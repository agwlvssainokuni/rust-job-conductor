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
//! 仮定した終了コード(なければ 0)の実行結果で進める。模擬の途中のステート(`PipelineState`)は、
//! この関数の中だけで持ち、返さず、保存しない。ステートの更新は、本番と同じ関数
//! (`flow::record` の `initial_state`、`apply_skip`、`apply_outcome`)で行う(BR1.8)。

use std::collections::HashSet;
use std::path::Path;

use super::{
    Assumption, EnvNameEntry, ExecutionPlan, NextStepNote, PlanDisposition, PlanEnd, PlanEntry,
    PlanRejection, ResumeInfo, ResumeKind, SimulatedResult,
};
use crate::envfile::{EnvContext, LoadedEnvFiles, merge_env};
use crate::flow::{
    apply_outcome, apply_skip, classify_and_decide, decide_before_run, initial_state,
};
use crate::types::{
    Classification, Decision, JobDefinition, JobExecution, JobOutcome, JobState, NextStep,
    PipelineDefinition, PipelineState, RejectReason, RunMode, SkipCause,
};

/// 模擬の初期ステートに入れる定義ファイルのパス。模擬の中だけで使い、出力にも保存にも使わない。
const SIMULATED_DEFINITION_PATH: &str = "<dry-run>";

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
    // ステートが無いときは、本番と同じ初期ステート(すべて pending)から始める。
    let mut state: PipelineState = match active_state(input) {
        Some(state) => state.clone(),
        None => initial_state(def, Path::new(SIMULATED_DEFINITION_PATH), None),
    };
    let initial_resume = state.resume_job_id.clone();
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
        let decision = decide_before_run(
            &job.id,
            def,
            &state.job_states,
            state.resume_job_id.as_deref(),
            state.end_reached,
        );
        match decision {
            Decision::Reject(reason) => return Err(rejection(&job.id, reason)),
            Decision::Skip(reason) => {
                // スキップは、本番と同じ `apply_skip` で記録する(BR1.8)。
                entries.push(entry(
                    input,
                    index,
                    skip_disposition(reason),
                    is_resume,
                    None,
                ));
                state = apply_skip(&state, &job.id, reason, def);
            }
            Decision::Run => {
                let (code, assumed) = assumed_exit_code(input.assumptions, &job.id);
                let (result, execution, outcome, next) = simulate_job(
                    job,
                    def,
                    &state.job_states,
                    code,
                    assumed,
                    RunMode::Pipeline,
                );
                // 結果は、本番と同じ `apply_outcome` で記録する(再開位置と `end_reached` も含む)。
                state = apply_outcome(&state, &job.id, &execution, &outcome, &next);
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
        None if state.end_reached => PlanEnd::EndReached,
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
        mode: RunMode::Pipeline,
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
        return Err(rejection(job_id, RejectReason::OutOfOrder));
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
            let (result, _, _, next) =
                simulate_job(job, def, states, code, assumed, RunMode::Individual);
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
        mode: RunMode::Individual,
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

fn rejection(job_id: &str, reason: RejectReason) -> PlanRejection {
    PlanRejection {
        job_id: job_id.to_string(),
        reason,
    }
}

/// 起動前の判断(スキップ)から、扱いを決める(BR1.3)。
fn skip_disposition(reason: SkipCause) -> PlanDisposition {
    match reason {
        SkipCause::AlreadyExecuted => PlanDisposition::AlreadyExecuted,
        SkipCause::AfterEnd => PlanDisposition::SkippedAfterEnd,
        SkipCause::SkipTarget => PlanDisposition::SkippedByTransition,
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
///
/// 計画の結果のほかに、ステートの記録(`apply_outcome`)に使う、仮定の実行(`JobExecution`)と
/// 分類の結果(`JobOutcome`)も返す(分類を 2 回しない)。
fn simulate_job(
    job: &JobDefinition,
    definition: &PipelineDefinition,
    states: &[JobState],
    exit_code: i32,
    assumed: bool,
    mode: RunMode,
) -> (SimulatedResult, JobExecution, JobOutcome, NextStep) {
    let execution = JobExecution::exited(job, exit_code);
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
        execution,
        outcome,
        next,
    )
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
