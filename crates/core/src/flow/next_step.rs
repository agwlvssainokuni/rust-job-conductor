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

//! 次の動きと完了(`classify_and_decide`)。BR1.10、BR3.1〜BR3.4、BR3.6。

use super::classify::classify;
use super::{is_settled, record_of};
use crate::types::{
    Classification, JobDefinition, JobExecution, JobOutcome, JobState, NextStep,
    PipelineDefinition, RunMode, SkipReason, TransitionOn, TransitionTarget,
};

/// ジョブを実行した後の、分類と次の動きを決める(W3)。
///
/// 分類と終了コードは [`classify`](BR2.1〜BR2.4)で決め、次の動きは次のとおり。
///
/// - 起動しなかった中断、エラー(起動失敗・シグナル終了・中断を含む): 停止。次のジョブは無く、
///   そのジョブを再開位置(`resume_job_id`)にする(BR2.4、BR3.4)。自動の再試行はしない。
/// - 成功・警告: 対応する遷移を見る(BR3.1)。ジョブIDなら、そのジョブが次の対象で、間のジョブは
///   `ByTransition` のスキップ(BR3.2)。`@end` なら、次の対象は無く、以降の全ジョブが `ByEnd` の
///   スキップで、`end_reached` が `true`(BR3.3)。遷移がなければ、記述順で次のジョブ(BR1.10)。
/// - 完了(BR3.6): 記述順で最後のジョブが成功・警告で終われば、`run_mode` によらず `completed`。
///   `Pipeline` で `@end` に到達したときは、以降の全ジョブを `skipped_job_ids`(`ByEnd`)に列挙し、
///   `completed` は `true`。
///   `Individual` で `@end` に到達したときは、`end_reached = true`、`skip_reason_kind = ByEnd` で、
///   `skipped_job_ids` は空、`completed = false`。以降のジョブは、スケジューラが呼んだときに
///   [`decide_before_run`](super::decide_before_run) が `Skip(AfterEnd)` を返し、Cli が呼ばれた
///   ジョブだけをスキップとして記録する。最後のジョブが記録されたとき、
///   [`resume_point_on_interrupt`](super::resume_point_on_interrupt) が `completed = true` を返す。
///   (`@end` より後ろに未処理のジョブがなければ、`Individual` でも、その場で `completed`。)
///
/// 再実行で成功・警告になったときは、再開位置は無し(BR3.4)。`job_states` は、すでに成功・警告・
/// スキップとして記録されているジョブを、スキップの対象に加えないために使う。
/// 定義にないジョブは、停止として返す(パニックしない)。
pub fn classify_and_decide(
    job: &JobDefinition,
    execution: &JobExecution,
    job_states: &[JobState],
    definition: &PipelineDefinition,
    run_mode: RunMode,
) -> (JobOutcome, NextStep) {
    let outcome = classify(job, execution);
    let step = next_step(job, &outcome, job_states, definition, run_mode);
    (outcome, step)
}

fn stop(job: &JobDefinition) -> NextStep {
    NextStep {
        job_id: job.id.clone(),
        next_job_id: None,
        skipped_job_ids: Vec::new(),
        skip_reason_kind: None,
        end_reached: false,
        completed: false,
        resume_job_id: Some(job.id.clone()),
    }
}

fn next_step(
    job: &JobDefinition,
    outcome: &JobOutcome,
    job_states: &[JobState],
    definition: &PipelineDefinition,
    run_mode: RunMode,
) -> NextStep {
    let on = match outcome.classification {
        Some(Classification::Success) => TransitionOn::Success,
        Some(Classification::Warning) => TransitionOn::Warning,
        // エラー、起動しなかった中断
        _ => return stop(job),
    };
    let Some(index) = definition.job_index(&job.id) else {
        return stop(job);
    };
    let jobs = &definition.jobs;
    let is_last = index + 1 == jobs.len();

    let mut step = NextStep {
        job_id: job.id.clone(),
        next_job_id: None,
        skipped_job_ids: Vec::new(),
        skip_reason_kind: None,
        end_reached: false,
        completed: false,
        resume_job_id: None,
    };

    let pending_ids = |range: &[JobDefinition]| -> Vec<String> {
        range
            .iter()
            .filter(|j| !is_settled(record_of(job_states, &j.id).0))
            .map(|j| j.id.clone())
            .collect()
    };

    let target = job.transition_for(on).map(|t| &t.next);
    match target {
        Some(TransitionTarget::End) => {
            step.end_reached = true;
            let pending = pending_ids(&jobs[index + 1..]);
            if !pending.is_empty() {
                step.skip_reason_kind = Some(SkipReason::ByEnd);
            }
            match run_mode {
                // BR3.3: 一連の実行では、以降の全ジョブをここでスキップとして列挙し、完了する。
                RunMode::Pipeline => {
                    step.skipped_job_ids = pending;
                    step.completed = true;
                }
                // BR3.6: 個別実行では、以降のジョブはスケジューラが呼んだときに 1 つずつ
                // スキップされる。ここでは列挙せず、最後のジョブのスキップが記録されるまで
                // 完了しない(後ろに未処理のジョブがなければ、いま完了)。
                RunMode::Individual => {
                    step.completed = pending.is_empty();
                }
            }
        }
        Some(TransitionTarget::Job(target_id))
            if definition.job_index(target_id).is_some_and(|t| t > index) =>
        {
            let t = definition.job_index(target_id).unwrap_or(index + 1);
            step.skipped_job_ids = pending_ids(&jobs[index + 1..t]);
            if !step.skipped_job_ids.is_empty() {
                step.skip_reason_kind = Some(SkipReason::ByTransition);
            }
            step.next_job_id = Some(target_id.clone());
        }
        // 遷移がない(BR1.10)。検証済みの定義では起きないが、遷移先が後ろのジョブでないときも、同じ扱い。
        _ => {
            if is_last {
                step.completed = true;
            } else {
                step.next_job_id = Some(jobs[index + 1].id.clone());
            }
        }
    }
    step
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{JobStatus, Transition};

    fn tr(on: TransitionOn, next: TransitionTarget) -> Transition {
        Transition { on, next }
    }

    fn job(id: &str, transitions: Vec<Transition>) -> JobDefinition {
        JobDefinition {
            id: id.into(),
            command: "c".into(),
            warning_exit_codes: vec![3],
            success_exit_codes: None,
            transitions,
        }
    }

    /// j0 -> j1 -> j2 -> j3 の定義。j0 の遷移だけを差し替えられる。
    fn def(j0_transitions: Vec<Transition>) -> PipelineDefinition {
        PipelineDefinition {
            name: "p".into(),
            description: None,
            format_version: 1,
            jobs: vec![
                job("j0", j0_transitions),
                job("j1", vec![]),
                job("j2", vec![]),
                job("j3", vec![]),
            ],
        }
    }

    fn exec(job_id: &str, code: i32) -> JobExecution {
        JobExecution {
            job_id: job_id.into(),
            command: "c".into(),
            started: true,
            exit_code: Some(code),
            terminated_by_signal: None,
            start_failure: false,
            interrupted: false,
            signal_number: None,
        }
    }

    fn run(
        d: &PipelineDefinition,
        id: &str,
        code: i32,
        states: &[JobState],
        mode: RunMode,
    ) -> NextStep {
        let j = &d.jobs[d.job_index(id).unwrap()];
        classify_and_decide(j, &exec(id, code), states, d, mode).1
    }

    #[test]
    fn 既定は記述順で次のジョブ() {
        let d = def(vec![]);
        for mode in [RunMode::Individual, RunMode::Pipeline] {
            for code in [0, 3] {
                let s = run(&d, "j0", code, &[], mode);
                assert_eq!(s.next_job_id.as_deref(), Some("j1"));
                assert!(s.skipped_job_ids.is_empty() && s.skip_reason_kind.is_none());
                assert!(!s.end_reached && !s.completed && s.resume_job_id.is_none());
            }
        }
    }

    #[test]
    fn 遷移先を指定すると間のジョブはスキップ() {
        let d = def(vec![tr(
            TransitionOn::Success,
            TransitionTarget::Job("j3".into()),
        )]);
        let s = run(&d, "j0", 0, &[], RunMode::Pipeline);
        assert_eq!(s.next_job_id.as_deref(), Some("j3"));
        assert_eq!(s.skipped_job_ids, vec!["j1", "j2"]);
        assert_eq!(s.skip_reason_kind, Some(SkipReason::ByTransition));
        assert!(!s.completed);
    }

    #[test]
    fn 隣のジョブへの遷移はスキップなし() {
        let d = def(vec![tr(
            TransitionOn::Success,
            TransitionTarget::Job("j1".into()),
        )]);
        let s = run(&d, "j0", 0, &[], RunMode::Pipeline);
        assert_eq!(s.next_job_id.as_deref(), Some("j1"));
        assert!(s.skipped_job_ids.is_empty() && s.skip_reason_kind.is_none());
    }

    #[test]
    fn 警告の遷移は警告のときだけ使う() {
        let d = def(vec![tr(
            TransitionOn::Warning,
            TransitionTarget::Job("j2".into()),
        )]);
        assert_eq!(
            run(&d, "j0", 3, &[], RunMode::Individual)
                .next_job_id
                .as_deref(),
            Some("j2")
        );
        assert_eq!(
            run(&d, "j0", 0, &[], RunMode::Individual)
                .next_job_id
                .as_deref(),
            Some("j1")
        );
    }

    #[test]
    fn end_の遷移_pipeline_は完了_individual_は完了しない() {
        let d = def(vec![tr(TransitionOn::Success, TransitionTarget::End)]);
        let p = run(&d, "j0", 0, &[], RunMode::Pipeline);
        assert!(p.end_reached && p.completed && p.next_job_id.is_none());
        assert_eq!(p.skipped_job_ids, vec!["j1", "j2", "j3"]);
        assert_eq!(p.skip_reason_kind, Some(SkipReason::ByEnd));

        let i = run(&d, "j0", 0, &[], RunMode::Individual);
        assert!(i.end_reached && !i.completed && i.next_job_id.is_none());
        assert!(
            i.skipped_job_ids.is_empty(),
            "個別実行では、呼び出し時に 1 つずつスキップする"
        );
        assert_eq!(i.skip_reason_kind, Some(SkipReason::ByEnd));
    }

    #[test]
    fn 最後のジョブの_end_は_individual_でも完了する() {
        let mut d = def(vec![]);
        d.jobs[3].transitions = vec![tr(TransitionOn::Success, TransitionTarget::End)];
        let s = run(&d, "j3", 0, &[], RunMode::Individual);
        assert!(s.end_reached && s.completed);
        assert!(s.skipped_job_ids.is_empty() && s.skip_reason_kind.is_none());
    }

    #[test]
    fn 最後のジョブの成功は完了() {
        let d = def(vec![]);
        for mode in [RunMode::Individual, RunMode::Pipeline] {
            let s = run(&d, "j3", 0, &[], mode);
            assert!(s.completed && s.next_job_id.is_none() && s.resume_job_id.is_none());
        }
    }

    #[test]
    fn エラーは停止して再開位置はそのジョブ() {
        let d = def(vec![tr(TransitionOn::Success, TransitionTarget::End)]);
        let s = run(&d, "j0", 1, &[], RunMode::Pipeline);
        assert!(s.next_job_id.is_none() && s.skipped_job_ids.is_empty());
        assert_eq!(s.resume_job_id.as_deref(), Some("j0"));
        assert!(!s.completed && !s.end_reached);
    }

    #[test]
    fn 起動失敗とシグナルと中断も停止() {
        let d = def(vec![]);
        let j = &d.jobs[1];
        let variants = [
            JobExecution {
                exit_code: None,
                start_failure: true,
                ..exec("j1", 0)
            },
            JobExecution {
                exit_code: None,
                terminated_by_signal: Some(9),
                ..exec("j1", 0)
            },
            JobExecution {
                interrupted: true,
                signal_number: Some(2),
                ..exec("j1", 0)
            },
            JobExecution {
                started: false,
                exit_code: None,
                interrupted: true,
                signal_number: Some(2),
                ..exec("j1", 0)
            },
        ];
        for e in variants {
            let (_, s) = classify_and_decide(j, &e, &[], &d, RunMode::Pipeline);
            assert_eq!(s.resume_job_id.as_deref(), Some("j1"));
            assert!(s.next_job_id.is_none() && !s.completed);
        }
    }

    #[test]
    fn 再実行で成功したら再開位置は無しで次の対象が決まる() {
        let d = def(vec![]);
        let states = vec![
            JobState {
                job_id: "j0".into(),
                status: JobStatus::Success,
                exit_code: Some(0),
                skip_reason: None,
            },
            JobState {
                job_id: "j1".into(),
                status: JobStatus::Error,
                exit_code: Some(1),
                skip_reason: None,
            },
        ];
        let s = run(&d, "j1", 0, &states, RunMode::Individual);
        assert!(s.resume_job_id.is_none());
        assert_eq!(s.next_job_id.as_deref(), Some("j2"));
    }

    #[test]
    fn 記録済みのジョブはスキップの対象に加えない() {
        let d = def(vec![tr(
            TransitionOn::Success,
            TransitionTarget::Job("j3".into()),
        )]);
        let states = vec![JobState {
            job_id: "j1".into(),
            status: JobStatus::Skipped,
            exit_code: None,
            skip_reason: Some(SkipReason::ByTransition),
        }];
        let s = run(&d, "j0", 0, &states, RunMode::Pipeline);
        assert_eq!(s.skipped_job_ids, vec!["j2"]);
    }

    #[test]
    fn 定義にないジョブは停止() {
        let d = def(vec![]);
        let stranger = job("zz", vec![]);
        let (_, s) = classify_and_decide(&stranger, &exec("zz", 0), &[], &d, RunMode::Pipeline);
        assert!(s.next_job_id.is_none() && !s.completed);
        assert_eq!(s.resume_job_id.as_deref(), Some("zz"));
    }

    #[test]
    fn 後ろ向きの遷移は既定の次のジョブにする() {
        // 検証済みの定義では起きない。起きても、パニックせず、既定の動きにする。
        let mut d = def(vec![]);
        d.jobs[2].transitions = vec![tr(
            TransitionOn::Success,
            TransitionTarget::Job("j0".into()),
        )];
        let s = run(&d, "j2", 0, &[], RunMode::Pipeline);
        assert_eq!(s.next_job_id.as_deref(), Some("j3"));
    }
}
