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

//! 判断(Flow)。分類、終了コード、遷移、完了、起動前の判断、中断時の再開位置と、
//! ステートへの記録(`record`。本番と模擬が共に使う)。
//!
//! ここにある関数は、すべて純粋な関数である。ファイルにもプロセスにも触れず、引数だけから
//! 結果を決める(NFR5.1、ADR-005)。失敗せず、必ず結果を返す(契約 C2)。
//! 各ジョブの記録(`JobState`)が無いジョブは、pending と同じ扱いにする。

// 下位モジュールは crate の中。関数と型は、下の再公開の名前(`flow::classify_and_decide` など)で出る。
mod attempt;
mod classify;
mod decide;
mod next_step;
mod record;
mod resume;
mod retry;

pub use attempt::{cancel_attempt, end_attempt, start_attempt};
pub use decide::decide_from_state;
pub use next_step::classify_and_decide;
pub(crate) use next_step::{TransitionResolution, resolve_transition};
pub use record::{apply_interrupt, apply_outcome, apply_skip, initial_state};
pub(crate) use record::{apply_outcome_in, apply_skip_in};
pub use resume::resume_point_on_interrupt;
pub(crate) use retry::is_retry_target;
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

/// 記述順の位置 `index` のジョブ `job_id` の状態。記録が同じ位置にあればそれを使い(O(1))、
/// なければ [`record_of`] と同じに引く(記録が無ければ pending)。
///
/// ステートの記録は、ふつう定義の記述順に並ぶので、位置で引けばジョブの数に比例する時間で
/// 全ジョブを見られる(1,000 ジョブの計画の性能のため。U3 の BR6.3、BR2.3)。
pub(crate) fn status_at(states: &[JobState], index: usize, job_id: &str) -> JobStatus {
    match states.get(index) {
        Some(s) if s.job_id == job_id => s.status,
        _ => record_of(states, job_id).0,
    }
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
