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

//! ジョブのあいだの中断の再開位置(`resume_point_on_interrupt`)。BR3.7。

use super::{has_end_skip, record_of};
use crate::types::{JobState, JobStatus, PipelineDefinition, ResumePoint};

/// ジョブとジョブのあいだで中断されたときの、再開位置を決める(W4、BR3.7)。
///
/// - 記述順で、まだ起動していない(pending、または記録なし)最初のジョブを、再開位置にする。
///   エラーで記録されたジョブが先にあれば、そのジョブが再開位置になる。
/// - 最後のジョブの直後なら、再開位置は無しで、`completed` が `true`。
/// - `end_reached` は、`@end` によるスキップの記録があれば `true`(引き継ぎ)。
/// - エラーとしては記録しない(この関数は、記録を作らない)。
///
/// Cli がこの関数を使うのは、`pipeline` のジョブのあいだの中断だけである。スキップを記録した
/// あとの完了と再開位置は [`apply_skip`](super::apply_skip) が書く(最後のジョブをスキップとして
/// 記録し、`pending` が無ければ `completed = true`。BR3.6)。
pub fn resume_point_on_interrupt(
    definition: &PipelineDefinition,
    job_states: &[JobState],
) -> ResumePoint {
    let resume = definition
        .job
        .iter()
        .find(|j| {
            matches!(
                record_of(job_states, &j.id).0,
                JobStatus::Pending | JobStatus::Error
            )
        })
        .map(|j| j.id.clone());
    ResumePoint {
        completed: resume.is_none(),
        resume_job_id: resume,
        end_reached: has_end_skip(job_states),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{JobDefinition, SkipReason};

    fn def(n: usize) -> PipelineDefinition {
        PipelineDefinition {
            name: "p".into(),
            description: None,
            run_id_env: None,
            remove_on_complete: false,
            format_version: 1,
            job: (0..n)
                .map(|i| JobDefinition {
                    id: format!("j{i}"),
                    command: "c".into(),
                    warning_exit_codes: vec![],
                    success_exit_codes: None,
                    transitions: vec![],
                    retry: None,
                    env: Vec::new(),
                    env_file: Vec::new(),
                })
                .collect(),
            env: Vec::new(),
            env_file: Vec::new(),
        }
    }

    fn st(id: &str, status: JobStatus, reason: Option<SkipReason>) -> JobState {
        JobState {
            job_id: id.into(),
            status,
            exit_code: None,
            skip_reason: reason,
        }
    }

    #[test]
    fn 記録がなければ最初のジョブ() {
        let r = resume_point_on_interrupt(&def(3), &[]);
        assert_eq!(r.resume_job_id.as_deref(), Some("j0"));
        assert!(!r.completed && !r.end_reached);
    }

    #[test]
    fn まだ起動していない最初のジョブ() {
        let states = vec![
            st("j0", JobStatus::Success, None),
            st("j1", JobStatus::Skipped, Some(SkipReason::ByTransition)),
        ];
        let r = resume_point_on_interrupt(&def(4), &states);
        assert_eq!(r.resume_job_id.as_deref(), Some("j2"));
        assert!(!r.completed);
    }

    #[test]
    fn 最後の直後は再開位置なしで完了() {
        let states = vec![
            st("j0", JobStatus::Success, None),
            st("j1", JobStatus::Warning, None),
        ];
        let r = resume_point_on_interrupt(&def(2), &states);
        assert_eq!(r.resume_job_id, None);
        assert!(r.completed);
    }

    #[test]
    fn 最後のジョブがスキップなら完了_end_reached_を引き継ぐ() {
        let states = vec![
            st("j0", JobStatus::Success, None),
            st("j1", JobStatus::Skipped, Some(SkipReason::ByEnd)),
        ];
        let r = resume_point_on_interrupt(&def(2), &states);
        assert!(r.completed && r.end_reached && r.resume_job_id.is_none());
    }

    #[test]
    fn end_reached_はスキップ待ちでも引き継ぐ() {
        let states = vec![
            st("j0", JobStatus::Success, None),
            st("j1", JobStatus::Skipped, Some(SkipReason::ByEnd)),
            st("j2", JobStatus::Pending, None),
        ];
        let r = resume_point_on_interrupt(&def(3), &states);
        assert_eq!(r.resume_job_id.as_deref(), Some("j2"));
        assert!(r.end_reached && !r.completed);
    }

    #[test]
    fn エラーのジョブが先にあればそのジョブ() {
        let states = vec![
            st("j0", JobStatus::Success, None),
            st("j1", JobStatus::Error, None),
        ];
        let r = resume_point_on_interrupt(&def(3), &states);
        assert_eq!(r.resume_job_id.as_deref(), Some("j1"));
    }
}
