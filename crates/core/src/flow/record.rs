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

//! ステートへの記録(Recorder。BR2.8)。入出力のない純粋な関数。
//!
//! 本番の流れ(Cli)と、`--dry-run` の模擬(`plan::simulate`)の両方が呼ぶ、ステートの更新の
//! 1 か所。本番では、呼び出し側(Cli)が、結果を受け取り、記録して、`write_state`
//! (`conductor-runtime`)を呼ぶ。`end_reached` は、一度 `true` になったら、`false` に戻さない
//! (monotonic)。

use std::path::Path;

use crate::run_id::RunId;
use crate::types::{
    Classification, DecisionReason, JobExecution, JobOutcome, JobState, JobStatus, NextStep,
    PipelineDefinition, PipelineState, ResumePoint, SkipReason,
};

/// 空のステートを、定義から作る(`initial_state`)。
///
/// すべてのジョブが `pending`、`resume_job_id` は無し、`end_reached` と `completed` は `false`。
/// `definition_key` と `definition_path` は、同じ文字列(定義ファイルのパス)。
///
/// `run_id` があれば、`run_id` に入れる(ファイル名の run ID と同じ値)。
pub fn initial_state(
    definition: &PipelineDefinition,
    definition_path: &Path,
    run_id: Option<&RunId>,
) -> PipelineState {
    let path = definition_path.to_string_lossy().into_owned();
    PipelineState {
        definition_key: path.clone(),
        pipeline_name: definition.name.clone(),
        definition_path: path,
        job_order: definition.job.iter().map(|j| j.id.clone()).collect(),
        job_states: definition
            .job
            .iter()
            .map(|j| JobState::pending(j.id.clone()))
            .collect(),
        resume_job_id: None,
        end_reached: false,
        completed: false,
        run_id: run_id.map(|r| r.as_str().to_string()),
    }
}

fn has_result(status: JobStatus) -> bool {
    matches!(
        status,
        JobStatus::Success | JobStatus::Warning | JobStatus::Error
    )
}

fn find_mut<'a>(state: &'a mut PipelineState, job_id: &str) -> Option<&'a mut JobState> {
    state.job_states.iter_mut().find(|j| j.job_id == job_id)
}

/// ジョブの結果を記録する(`apply_outcome`。BR2.8)。
///
/// - そのジョブの `status` は、分類のとおり(分類が無い、起動しなかった中断のときは、変えない)。
///   `exit_code` は、`JobExecution` の `exit_code` そのまま(無ければ、以前の値を消す)。
/// - `NextStep` の `skipped_job_ids` の各ジョブを `skipped` にし、`skip_reason` に
///   `skip_reason_kind` を書く。
/// - `resume_job_id` と `completed` は、`NextStep` のとおり。`end_reached` は、一度 `true` に
///   なったら戻さない。
///
/// 定義にない(状態にないジョブ)IDは、無視する(パニックしない)。
pub fn apply_outcome(
    state: &PipelineState,
    job_id: &str,
    execution: &JobExecution,
    outcome: &JobOutcome,
    next: &NextStep,
) -> PipelineState {
    let mut out = state.clone();
    if let Some(job) = find_mut(&mut out, job_id) {
        if let Some(classification) = outcome.classification {
            job.status = match classification {
                Classification::Success => JobStatus::Success,
                Classification::Warning => JobStatus::Warning,
                Classification::Error => JobStatus::Error,
            };
            job.skip_reason = None;
        }
        job.exit_code = execution.exit_code;
        if !has_result(job.status) {
            // 状態の整合(結果の無い状態には、終了コードを持たせない)。
            job.exit_code = None;
        }
    }
    let reason = next.skip_reason_kind.unwrap_or(SkipReason::ByTransition);
    for skipped in &next.skipped_job_ids {
        if skipped == job_id {
            continue;
        }
        if let Some(job) = find_mut(&mut out, skipped) {
            job.status = JobStatus::Skipped;
            job.exit_code = None;
            job.skip_reason = Some(reason);
        }
    }
    out.resume_job_id = next.resume_job_id.clone();
    out.completed = next.completed;
    out.end_reached = state.end_reached || next.end_reached;
    out
}

/// スキップを記録する(`apply_skip`。BR2.8)。
///
/// - [`DecisionReason::AlreadyExecuted`](と、スキップでない理由)は、何も変えない。
/// - `SkipTarget`(`byTransition`)と `AfterEnd`(`byEnd`)は、そのジョブを `skipped` にする。
///   そのあと、`error` のジョブがあれば、`resume_job_id` と `completed` を変えない(エラーの再開位置を
///   守る)。なければ、`completed` を「いま `skipped` にしたジョブが、定義の順で最後であり、かつ
///   `pending` が無い」として、`resume_job_id` を「定義の順で最初の `pending`(無ければ無し)」として
///   書く。`AfterEnd` は、`end_reached` を `true` にする。
pub fn apply_skip(
    state: &PipelineState,
    job_id: &str,
    reason: DecisionReason,
    definition: &PipelineDefinition,
) -> PipelineState {
    let skip_reason = match reason {
        DecisionReason::SkipTarget => SkipReason::ByTransition,
        DecisionReason::AfterEnd => SkipReason::ByEnd,
        DecisionReason::AlreadyExecuted | DecisionReason::OutOfOrder => return state.clone(),
    };
    let mut out = state.clone();
    match find_mut(&mut out, job_id) {
        Some(job) => {
            job.status = JobStatus::Skipped;
            job.exit_code = None;
            job.skip_reason = Some(skip_reason);
        }
        None => return out,
    }
    if reason == DecisionReason::AfterEnd {
        out.end_reached = true;
    }

    let status_of = |id: &str| {
        out.job_states
            .iter()
            .find(|j| j.job_id == id)
            .map_or(JobStatus::Pending, |j| j.status)
    };
    if out.job_states.iter().any(|j| j.status == JobStatus::Error) {
        return out;
    }
    let first_pending = definition
        .job
        .iter()
        .find(|j| status_of(&j.id) == JobStatus::Pending)
        .map(|j| j.id.clone());
    let is_last = definition.job.last().is_some_and(|j| j.id == job_id);
    out.completed = is_last && first_pending.is_none();
    out.resume_job_id = first_pending;
    out
}

/// ジョブを起動しない中断、またはジョブのあいだの中断を記録する(`apply_interrupt`)。
///
/// `resume_job_id`、`completed` を `ResumePoint` のとおりに、`end_reached` を(戻さずに)書く。
/// エラーとしては記録しない。
pub fn apply_interrupt(state: &PipelineState, resume_point: &ResumePoint) -> PipelineState {
    let mut out = state.clone();
    out.resume_job_id = resume_point.resume_job_id.clone();
    out.completed = resume_point.completed;
    out.end_reached = state.end_reached || resume_point.end_reached;
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::JobDefinition;

    /// run ID なしの初期ステート(既存のテストの書き方を保つ)。
    fn initial_state(definition: &PipelineDefinition, path: &Path) -> PipelineState {
        super::initial_state(definition, path, None)
    }

    #[test]
    fn 初期ステートは_run_id_を持つ() {
        let d = definition(&["a"]);
        let id = RunId::parse("r1").unwrap();
        let s = super::initial_state(&d, Path::new("c.toml"), Some(&id));
        assert_eq!(s.run_id.as_deref(), Some("r1"));
        assert_eq!(initial_state(&d, Path::new("c.toml")).run_id, None);
    }

    fn definition(ids: &[&str]) -> PipelineDefinition {
        PipelineDefinition {
            name: "p".into(),
            description: None,
            run_id_env: None,
            remove_on_complete: false,
            format_version: 1,
            job: ids
                .iter()
                .map(|id| JobDefinition {
                    id: (*id).into(),
                    command: "echo".into(),
                    warning_exit_codes: vec![],
                    success_exit_codes: None,
                    transitions: vec![],
                    retry: None,
                    env: vec![],
                    env_file: vec![],
                })
                .collect(),
            env: vec![],
            env_file: vec![],
        }
    }

    fn exec(job: &str, code: Option<i32>) -> JobExecution {
        JobExecution {
            job_id: job.into(),
            command: "echo".into(),
            started: true,
            exit_code: code,
            terminated_by_signal: None,
            start_failure: false,
            interrupted: false,
            signal_number: None,
        }
    }

    fn outcome(job: &str, c: Option<Classification>) -> JobOutcome {
        JobOutcome {
            job_id: job.into(),
            classification: c,
            reported_exit_code: 0,
            conductor_error: false,
        }
    }

    fn step(job: &str) -> NextStep {
        NextStep {
            job_id: job.into(),
            next_job_id: None,
            skipped_job_ids: vec![],
            skip_reason_kind: None,
            end_reached: false,
            completed: false,
            resume_job_id: None,
        }
    }

    fn status(state: &PipelineState, id: &str) -> JobStatus {
        state
            .job_states
            .iter()
            .find(|j| j.job_id == id)
            .map(|j| j.status)
            .expect("ある")
    }

    #[test]
    fn 初期ステートはすべて_pending() {
        let s = initial_state(&definition(&["a", "b"]), Path::new("dir/c.toml"));
        assert_eq!(s.job_order, ["a", "b"]);
        assert!(
            s.job_states
                .iter()
                .all(|j| *j == JobState::pending(j.job_id.clone()))
        );
        assert_eq!(s.definition_key, "dir/c.toml");
        assert_eq!(s.definition_key, s.definition_path);
        assert_eq!(
            (s.resume_job_id, s.end_reached, s.completed),
            (None, false, false)
        );
    }

    #[test]
    fn 成功を記録して終了コードを書く() {
        let d = definition(&["a", "b"]);
        let s0 = initial_state(&d, Path::new("c.toml"));
        let s = apply_outcome(
            &s0,
            "a",
            &exec("a", Some(0)),
            &outcome("a", Some(Classification::Success)),
            &step("a"),
        );
        assert_eq!(status(&s, "a"), JobStatus::Success);
        assert_eq!(s.job_states[0].exit_code, Some(0));
        assert_eq!(status(&s, "b"), JobStatus::Pending);
        assert_eq!(
            s0.job_states[0].status,
            JobStatus::Pending,
            "元のステートは変わらない"
        );
    }

    #[test]
    fn 警告とエラーを記録する() {
        let d = definition(&["a", "b"]);
        let s0 = initial_state(&d, Path::new("c.toml"));
        let w = apply_outcome(
            &s0,
            "a",
            &exec("a", Some(2)),
            &outcome("a", Some(Classification::Warning)),
            &step("a"),
        );
        assert_eq!(
            (status(&w, "a"), w.job_states[0].exit_code),
            (JobStatus::Warning, Some(2))
        );
        let mut next = step("b");
        next.resume_job_id = Some("b".into());
        let e = apply_outcome(
            &w,
            "b",
            &exec("b", Some(7)),
            &outcome("b", Some(Classification::Error)),
            &next,
        );
        assert_eq!(status(&e, "b"), JobStatus::Error);
        assert_eq!(e.resume_job_id.as_deref(), Some("b"));
    }

    #[test]
    fn 起動失敗とシグナル終了は終了コードなし() {
        let d = definition(&["a"]);
        let s0 = initial_state(&d, Path::new("c.toml"));
        let s = apply_outcome(
            &s0,
            "a",
            &exec("a", None),
            &outcome("a", Some(Classification::Error)),
            &step("a"),
        );
        assert_eq!(
            (status(&s, "a"), s.job_states[0].exit_code),
            (JobStatus::Error, None)
        );
    }

    #[test]
    fn 再実行で古い終了コードが消え_エラーが解ければ再開位置も消える() {
        let d = definition(&["a", "b"]);
        let mut s = initial_state(&d, Path::new("c.toml"));
        let mut next = step("a");
        next.resume_job_id = Some("a".into());
        s = apply_outcome(
            &s,
            "a",
            &exec("a", Some(5)),
            &outcome("a", Some(Classification::Error)),
            &next,
        );
        assert_eq!(s.job_states[0].exit_code, Some(5));
        // 再実行で、終了コードのないエラー(シグナル終了)。古い 5 は消える。
        s = apply_outcome(
            &s,
            "a",
            &exec("a", None),
            &outcome("a", Some(Classification::Error)),
            &next,
        );
        assert_eq!(s.job_states[0].exit_code, None);
        // 再実行で成功。再開位置が無しに戻る。
        s = apply_outcome(
            &s,
            "a",
            &exec("a", Some(0)),
            &outcome("a", Some(Classification::Success)),
            &step("a"),
        );
        assert_eq!(
            (status(&s, "a"), s.resume_job_id),
            (JobStatus::Success, None)
        );
    }

    #[test]
    fn 起動しなかった中断は_status_を変えない() {
        let d = definition(&["a"]);
        let s0 = initial_state(&d, Path::new("c.toml"));
        let mut e = exec("a", None);
        e.started = false;
        e.interrupted = true;
        let s = apply_outcome(&s0, "a", &e, &outcome("a", None), &step("a"));
        assert_eq!(s.job_states[0], JobState::pending("a"));
    }

    #[test]
    fn 次の動きのスキップと理由を記録する() {
        let d = definition(&["a", "b", "c"]);
        let s0 = initial_state(&d, Path::new("c.toml"));
        let mut next = step("a");
        next.skipped_job_ids = vec!["b".into(), "c".into()];
        next.skip_reason_kind = Some(SkipReason::ByEnd);
        next.end_reached = true;
        next.completed = true;
        let s = apply_outcome(
            &s0,
            "a",
            &exec("a", Some(0)),
            &outcome("a", Some(Classification::Success)),
            &next,
        );
        assert_eq!(s.job_states[1].skip_reason, Some(SkipReason::ByEnd));
        assert_eq!(status(&s, "c"), JobStatus::Skipped);
        assert!(s.end_reached && s.completed);
    }

    #[test]
    fn end_reached_は一度立てば戻らない() {
        let d = definition(&["a", "b"]);
        let mut s = initial_state(&d, Path::new("c.toml"));
        s.end_reached = true;
        let s = apply_outcome(
            &s,
            "a",
            &exec("a", Some(0)),
            &outcome("a", Some(Classification::Success)),
            &step("a"),
        );
        assert!(s.end_reached);
        let rp = ResumePoint {
            resume_job_id: Some("b".into()),
            end_reached: false,
            completed: false,
        };
        assert!(apply_interrupt(&s, &rp).end_reached);
    }

    #[test]
    fn 知らないジョブidは無視する() {
        let d = definition(&["a"]);
        let s0 = initial_state(&d, Path::new("c.toml"));
        let s = apply_outcome(
            &s0,
            "zz",
            &exec("zz", Some(0)),
            &outcome("zz", Some(Classification::Success)),
            &step("zz"),
        );
        assert_eq!(s, s0);
        assert_eq!(apply_skip(&s0, "zz", DecisionReason::SkipTarget, &d), s0);
    }

    #[test]
    fn already_executed_は不変() {
        let d = definition(&["a", "b"]);
        let mut s = initial_state(&d, Path::new("c.toml"));
        s.job_states[0].status = JobStatus::Success;
        s.job_states[0].exit_code = Some(0);
        assert_eq!(apply_skip(&s, "a", DecisionReason::AlreadyExecuted, &d), s);
        assert_eq!(apply_skip(&s, "a", DecisionReason::OutOfOrder, &d), s);
    }

    #[test]
    fn error_があるときはスキップしても再開位置と完了を変えない() {
        let d = definition(&["a", "b", "c"]);
        let mut s = initial_state(&d, Path::new("c.toml"));
        s.job_states[0].status = JobStatus::Error;
        s.job_states[0].exit_code = Some(1);
        s.resume_job_id = Some("a".into());
        let s2 = apply_skip(&s, "c", DecisionReason::SkipTarget, &d);
        assert_eq!(status(&s2, "c"), JobStatus::Skipped);
        assert_eq!(
            (s2.resume_job_id.as_deref(), s2.completed),
            (Some("a"), false)
        );
    }

    #[test]
    fn 最後のジョブのスキップで_completed() {
        let d = definition(&["a", "b"]);
        let mut s = initial_state(&d, Path::new("c.toml"));
        s.job_states[0].status = JobStatus::Success;
        s.job_states[0].exit_code = Some(0);
        let s2 = apply_skip(&s, "b", DecisionReason::SkipTarget, &d);
        assert!(s2.completed);
        assert_eq!(s2.resume_job_id, None);
        assert_eq!(s2.job_states[1].skip_reason, Some(SkipReason::ByTransition));
        assert!(!s2.end_reached);
    }

    #[test]
    fn 最後でないジョブのスキップでは再開位置が最初の_pending() {
        let d = definition(&["a", "b", "c"]);
        let s = initial_state(&d, Path::new("c.toml"));
        let s2 = apply_skip(&s, "a", DecisionReason::SkipTarget, &d);
        assert!(!s2.completed);
        assert_eq!(s2.resume_job_id.as_deref(), Some("b"));
    }

    #[test]
    fn 最後のジョブが先にスキップされ_前が_pending_なら完了しない() {
        let d = definition(&["a", "b", "c"]);
        let s = initial_state(&d, Path::new("c.toml"));
        let s2 = apply_skip(&s, "c", DecisionReason::SkipTarget, &d);
        assert!(!s2.completed);
        assert_eq!(s2.resume_job_id.as_deref(), Some("a"));
    }

    #[test]
    fn end_のあとの個別実行() {
        // A → @end。B と C は pending のまま(conductor-core の個別実行は、skipped_job_ids が空)。
        let d = definition(&["a", "b", "c"]);
        let mut next = step("a");
        next.skip_reason_kind = Some(SkipReason::ByEnd);
        next.end_reached = true;
        let s = apply_outcome(
            &initial_state(&d, Path::new("c.toml")),
            "a",
            &exec("a", Some(0)),
            &outcome("a", Some(Classification::Success)),
            &next,
        );
        assert!(s.end_reached && !s.completed);
        assert_eq!(status(&s, "b"), JobStatus::Pending);

        let s = apply_skip(&s, "b", DecisionReason::AfterEnd, &d);
        assert!(!s.completed, "C が残っている");
        assert_eq!(s.resume_job_id.as_deref(), Some("c"));
        assert_eq!(s.job_states[1].skip_reason, Some(SkipReason::ByEnd));
        assert!(s.end_reached);

        let s = apply_skip(&s, "c", DecisionReason::AfterEnd, &d);
        assert!(s.completed);
        assert_eq!(s.resume_job_id, None);
    }

    #[test]
    fn 中断を記録する() {
        let d = definition(&["a", "b"]);
        let s = initial_state(&d, Path::new("c.toml"));
        let rp = ResumePoint {
            resume_job_id: Some("b".into()),
            end_reached: true,
            completed: false,
        };
        let s2 = apply_interrupt(&s, &rp);
        assert_eq!(s2.resume_job_id.as_deref(), Some("b"));
        assert!(s2.end_reached && !s2.completed);
        assert!(
            s2.job_states.iter().all(|j| j.status == JobStatus::Pending),
            "エラーとしては記録しない"
        );
    }
}
