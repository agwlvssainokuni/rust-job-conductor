// Copyright 2026 agwlvssainokuni
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! 計画の生成(模擬。WF3。BR1.2〜BR1.8、BR2.3、BR2.4、BR4.3)。純粋な関数。
//!
//! 実行と同じ判断(`decide_before_run`、`classify_and_decide`)を、ジョブを起動する代わりに、
//! 仮定した終了コード(なければ 0)の実行結果で進める。模擬の途中の状態(ジョブの状態の
//! コピー、再開位置、`end_reached`)は、この関数の中だけで持ち、返さず、保存しない。
//! ステートの更新は、本番の `apply_outcome`(`conductor-runtime`。`core` は呼べない)と
//! 同じ結果になるよう、小さな関数 [`apply_result`] に書く(BR1.8)。

use std::collections::HashSet;

use super::{
    Assumption, EnvNameEntry, ExecutionPlan, NextStepNote, PlanDisposition, PlanEnd, PlanEntry,
    PlanMode, PlanRejection, ResumeInfo, ResumeKind, SimulatedResult,
};
use crate::envfile::{EnvContext, LoadedEnvFiles, merge_env};
use crate::flow::{classify_and_decide, decide_before_run};
use crate::types::{
    Classification, Decision, DecisionReason, JobDefinition, JobExecution, JobState, JobStatus,
    NextStep, PipelineDefinition, PipelineState, RunMode, SkipReason,
};

/// 計画の入力。値(環境変数、`.env` の内容)は、名前の要約を作るためだけに読み、計画に持たない。
#[derive(Debug, Clone, Copy)]
pub struct PlanInput<'a> {
    pub definition: &'a PipelineDefinition,
    /// 読んだステート(完了したステートは `None` として渡される。BR1.4)。
    pub state: Option<&'a PipelineState>,
    /// 各ジョブの分割後のコマンド(定義の記述順。`CliFlows` が分けて渡す。BR1.5)。
    pub commands: &'a [Vec<String>],
    pub env_files: &'a LoadedEnvFiles,
    pub run_id: Option<&'a str>,
    /// `--assume` の指定(指定順。同じジョブ ID は呼び出し側が重複させない。BR2.1)。
    pub assumptions: &'a [Assumption],
}

/// `pipeline <name> --dry-run` の計画を作る(BR1.2〜BR1.8)。
///
/// # Errors
///
/// 本番なら順序外として拒否される(`Reject`)ときは、拒否を返す(BR4.3)。
pub fn plan_pipeline(input: &PlanInput<'_>) -> Result<ExecutionPlan, PlanRejection> {
    let def = input.definition;
    // ステートが無いときは、すべて pending の状態から始める(本番の初期ステートと同じ判断になる)。
    let mut states: Vec<JobState> = match active_state(input) {
        Some(state) => state.job_states.clone(),
        None => def
            .job
            .iter()
            .map(|j| JobState::pending(j.id.clone()))
            .collect(),
    };
    let initial_resume = active_state(input).and_then(|s| s.resume_job_id.clone());
    let mut resume = initial_resume.clone();
    let mut end_reached = active_state(input).is_some_and(|s| s.end_reached);
    let mut stopped: Option<String> = None;
    let mut entries: Vec<PlanEntry> = Vec::with_capacity(def.job.len());

    for (index, job) in def.job.iter().enumerate() {
        let is_resume = initial_resume.as_deref() == Some(job.id.as_str());
        if stopped.is_some() {
            entries.push(entry(
                input,
                index,
                PlanDisposition::NotReached,
                is_resume,
                None,
            ));
            continue;
        }
        let decision = decide_before_run(&job.id, def, &states, resume.as_deref(), end_reached);
        match decision {
            Decision::Reject(reason) => return Err(rejection(&job.id, reason)),
            Decision::Skip(reason) => {
                // スキップは、すでに決まっている。模擬の状態は変えない(BR1.8)。
                entries.push(entry(
                    input,
                    index,
                    skip_disposition(reason),
                    is_resume,
                    None,
                ));
            }
            Decision::Run => {
                let (code, assumed) = assumed_exit_code(input.assumptions, &job.id);
                let (result, next) =
                    simulate_job(job, def, &states, code, assumed, RunMode::Pipeline);
                apply_result(&mut states, &job.id, code, result.classification, &next);
                resume = next.resume_job_id.clone();
                end_reached = end_reached || next.end_reached;
                if result.classification == Classification::Error {
                    stopped = Some(job.id.clone());
                }
                entries.push(entry(
                    input,
                    index,
                    PlanDisposition::WillRun,
                    is_resume,
                    Some(result),
                ));
            }
        }
    }

    let end = match stopped {
        Some(job_id) => PlanEnd::StoppedAtError(job_id),
        None if end_reached => PlanEnd::EndReached,
        None => PlanEnd::Completed,
    };
    let resume_info = match &initial_resume {
        Some(id) => ResumeInfo {
            kind: ResumeKind::AfterStop,
            job_id: Some(id.clone()),
        },
        None => ResumeInfo {
            kind: ResumeKind::FromStart,
            job_id: None,
        },
    };
    let ran: HashSet<&str> = entries
        .iter()
        .filter(|e| e.disposition == PlanDisposition::WillRun)
        .map(|e| e.job_id.as_str())
        .collect();
    let unused = unused_assumptions(input.assumptions, &ran);
    Ok(ExecutionPlan {
        pipeline_name: def.name.clone(),
        run_id: input.run_id.map(str::to_string),
        mode: PlanMode::Pipeline,
        resume: resume_info,
        assumptions: input.assumptions.to_vec(),
        unused_assumptions: unused,
        entries,
        end,
        next_step_note: None,
    })
}

/// `run <job> --dry-run` の計画を作る(BR1.6)。対象の 1 ジョブの項目と、次の動き。
///
/// # Errors
///
/// 対象が順序外で拒否される(定義にないジョブを含む)ときは、拒否を返す(BR4.3)。
pub fn plan_run(job_id: &str, input: &PlanInput<'_>) -> Result<ExecutionPlan, PlanRejection> {
    let def = input.definition;
    let Some(index) = def.job_index(job_id) else {
        return Err(rejection(job_id, DecisionReason::OutOfOrder));
    };
    let job = &def.job[index];
    let (states, resume, end_reached): (&[JobState], Option<&str>, bool) = match active_state(input)
    {
        Some(state) => (
            state.job_states.as_slice(),
            state.resume_job_id.as_deref(),
            state.end_reached,
        ),
        None => (&[][..], None, false),
    };
    let decision = decide_before_run(job_id, def, states, resume, end_reached);

    let mut note = None;
    let mut ran = HashSet::new();
    let (item, end) = match decision {
        Decision::Reject(reason) => return Err(rejection(job_id, reason)),
        Decision::Skip(reason) => (
            entry(input, index, skip_disposition(reason), false, None),
            PlanEnd::Completed,
        ),
        Decision::Run => {
            let (code, assumed) = assumed_exit_code(input.assumptions, job_id);
            let (result, next) = simulate_job(job, def, states, code, assumed, RunMode::Individual);
            let stopped = result.classification == Classification::Error;
            note = Some(NextStepNote {
                next_job_id: next.next_job_id.clone(),
                skipped_job_ids: next.skipped_job_ids.clone(),
                completed: next.completed,
                end_reached: next.end_reached,
                stopped,
            });
            let end = if stopped {
                PlanEnd::StoppedAtError(job_id.to_string())
            } else if next.end_reached {
                PlanEnd::EndReached
            } else {
                PlanEnd::Completed
            };
            ran.insert(job_id);
            (
                entry(input, index, PlanDisposition::WillRun, false, Some(result)),
                end,
            )
        }
    };
    Ok(ExecutionPlan {
        pipeline_name: def.name.clone(),
        run_id: input.run_id.map(str::to_string),
        mode: PlanMode::Individual,
        resume: ResumeInfo {
            kind: ResumeKind::Individual,
            job_id: Some(job_id.to_string()),
        },
        assumptions: input.assumptions.to_vec(),
        unused_assumptions: unused_assumptions(input.assumptions, &ran),
        entries: vec![item],
        end,
        next_step_note: note,
    })
}

fn rejection(job_id: &str, reason: DecisionReason) -> PlanRejection {
    PlanRejection {
        job_id: job_id.to_string(),
        reason,
    }
}

/// 起動前の判断(スキップ)から、扱いを決める(BR1.3)。
fn skip_disposition(reason: DecisionReason) -> PlanDisposition {
    match reason {
        DecisionReason::AlreadyExecuted => PlanDisposition::AlreadyExecuted,
        DecisionReason::AfterEnd => PlanDisposition::SkippedAfterEnd,
        DecisionReason::SkipTarget | DecisionReason::OutOfOrder => {
            PlanDisposition::SkippedByTransition
        }
    }
}

/// ジョブの仮定した終了コード。指定がなければ 0(成功。BR2.4)。戻り値の 2 つ目は、指定したか。
fn assumed_exit_code(assumptions: &[Assumption], job_id: &str) -> (i32, bool) {
    match assumptions.iter().find(|a| a.job_id == job_id) {
        Some(a) => (a.exit_code, true),
        None => (0, false),
    }
}

/// 模擬で `WillRun` にならなかった仮定のジョブ ID(指定順。BR2.3)。
fn unused_assumptions(assumptions: &[Assumption], ran: &HashSet<&str>) -> Vec<String> {
    assumptions
        .iter()
        .filter(|a| !ran.contains(a.job_id.as_str()))
        .map(|a| a.job_id.clone())
        .collect()
}

/// 仮定した終了コードで、実行の代わりに、分類と次の動きを決める(BR1.2、BR1.7)。
fn simulate_job(
    job: &JobDefinition,
    definition: &PipelineDefinition,
    states: &[JobState],
    exit_code: i32,
    assumed: bool,
    mode: RunMode,
) -> (SimulatedResult, NextStep) {
    let execution = JobExecution {
        job_id: job.id.clone(),
        command: job.command.clone(),
        started: true,
        exit_code: Some(exit_code),
        terminated_by_signal: None,
        start_failure: false,
        interrupted: false,
        signal_number: None,
    };
    let (outcome, next) = classify_and_decide(job, &execution, states, definition, mode);
    // 起動したので、分類は必ずある。念のため、無ければエラーとして扱う。
    let classification = outcome.classification.unwrap_or(Classification::Error);
    let retryable = job
        .retry
        .as_ref()
        .is_some_and(|r| r.exit_codes.contains(&exit_code));
    (
        SimulatedResult {
            exit_code,
            classification,
            assumed,
            retryable,
        },
        next,
    )
}

/// 模擬の状態に、ジョブの結果を反映する(BR1.8)。本番の `apply_outcome` と同じ結果にする:
/// ジョブの状態(分類のとおり)、次の動きのスキップ(`skipped` と理由)。
/// 再開位置と `end_reached` は、呼び出し側が `NextStep` から更新する。
fn apply_result(
    states: &mut [JobState],
    job_id: &str,
    exit_code: i32,
    classification: Classification,
    next: &NextStep,
) {
    if let Some(job) = states.iter_mut().find(|s| s.job_id == job_id) {
        job.status = match classification {
            Classification::Success => JobStatus::Success,
            Classification::Warning => JobStatus::Warning,
            Classification::Error => JobStatus::Error,
        };
        job.exit_code = Some(exit_code);
        job.skip_reason = None;
    }
    let reason = next.skip_reason_kind.unwrap_or(SkipReason::ByTransition);
    for skipped in &next.skipped_job_ids {
        if skipped == job_id {
            continue;
        }
        if let Some(job) = states.iter_mut().find(|s| &s.job_id == skipped) {
            job.status = JobStatus::Skipped;
            job.exit_code = None;
            job.skip_reason = Some(reason);
        }
    }
}

/// 計画の項目を作る。コマンド、環境変数の名前と出どころ(試行 1。値は持たない)、リトライの設定を持たせる(BR1.5)。
fn entry(
    input: &PlanInput<'_>,
    index: usize,
    disposition: PlanDisposition,
    is_resume: bool,
    result: Option<SimulatedResult>,
) -> PlanEntry {
    let def = input.definition;
    let job = &def.job[index];
    let ctx = EnvContext {
        pipeline_name: &def.name,
        job_id: &job.id,
        run_id: input.run_id,
        attempt: 1,
    };
    let env_names = merge_env(def, job, input.env_files, &ctx)
        .names()
        .into_iter()
        .map(|(name, origin)| EnvNameEntry {
            name: name.to_string(),
            origin,
        })
        .collect();
    PlanEntry {
        job_id: job.id.clone(),
        disposition,
        is_resume,
        command_words: input.commands.get(index).cloned().unwrap_or_default(),
        env_names,
        retry: job.retry.clone(),
        result,
    }
}

/// 模擬の出発点にするステート。完了したステートは、本番と同じに、ステートなしとして扱う(BR1.4)。
fn active_state<'a>(input: &PlanInput<'a>) -> Option<&'a PipelineState> {
    input.state.filter(|s| !s.completed)
}
