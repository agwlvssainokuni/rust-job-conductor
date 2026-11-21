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

//! 判断(Flow)。分類、終了コード、遷移、完了、起動前の判断、中断時の再開位置。
//!
//! ここにある関数は、すべて純粋な関数である。ファイルにもプロセスにも触れず、引数だけから
//! 結果を決める(NFR5.1、ADR-005)。失敗せず、必ず結果を返す(契約 C2)。
//! 各ジョブの記録(`JobState`)が無いジョブは、pending と同じ扱いにする。

pub mod classify;
pub mod decide;
pub mod next_step;
pub mod resume;
pub mod retry;

pub use classify::classify;
pub use decide::decide_before_run;
pub use next_step::{TransitionResolution, classify_and_decide, resolve_transition};
pub use resume::resume_point_on_interrupt;
pub use retry::{RetryDecision, retry_decision};

use crate::types::{JobState, JobStatus, SkipReason};

/// ジョブの記録を引く。記録が無ければ pending(理由なし)として返す。
pub(crate) fn record_of(states: &[JobState], job_id: &str) -> (JobStatus, Option<SkipReason>) {
    states
        .iter()
        .find(|s| s.job_id == job_id)
        .map(|s| (s.status, s.skip_reason))
        .unwrap_or((JobStatus::Pending, None))
}

/// 成功・警告・スキップのいずれか(「済んだ」状態)か。
pub(crate) fn is_settled(status: JobStatus) -> bool {
    matches!(
        status,
        JobStatus::Success | JobStatus::Warning | JobStatus::Skipped
    )
}

/// `@end` によるスキップの記録があるか(`@end` に到達済みの印)。
pub(crate) fn has_end_skip(states: &[JobState]) -> bool {
    states
        .iter()
        .any(|s| s.status == JobStatus::Skipped && s.skip_reason == Some(SkipReason::ByEnd))
}
