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
mod activity;
mod attempt;
mod classify;
mod decide;
mod next_step;
mod record;
mod resume;
mod retry;

pub use activity::{PipelineActivity, pipeline_activity};
pub use attempt::{cancel_attempt, end_attempt, start_attempt};
pub(crate) use decide::decide_at;
pub use decide::decide_from_state;
pub use next_step::classify_and_decide;
pub(crate) use next_step::{TransitionResolution, classify_and_decide_at, resolve_transition};
pub use record::{apply_interrupt, apply_outcome, apply_skip, initial_state};
pub(crate) use record::{apply_outcome_in, apply_skip_in};
pub use resume::resume_point_on_interrupt;
pub(crate) use retry::is_retry_target;
pub use retry::{RetryDecision, retry_decision};

use crate::types::{JobState, JobStatus, PipelineDefinition, SkipReason};

/// ジョブの記録を引く。記録が無ければ pending(理由なし)として返す。
pub(crate) fn record_of(states: &[JobState], job_id: &str) -> (JobStatus, Option<SkipReason>) {
    states
        .iter()
        .find(|s| s.job_id == job_id)
        .map(|s| (s.status, s.skip_reason))
        .unwrap_or((JobStatus::Pending, None))
}

/// 記述順の位置 `index` のジョブ `job_id` の記録(状態とスキップの理由)。記録が同じ位置にあれば
/// それを使い(O(1))、なければ [`record_of`] と同じに引く(記録が無ければ pending)。
///
/// 記録の順が定義と違う、記録が欠ける、といったステートでも、[`record_of`] と同じ答えになる
/// (ステートのジョブ ID は一意という前提。定義の検証と読み込みが重複を拒む。v0.10.1 の BR3.4)。
pub(crate) fn record_at(
    states: &[JobState],
    index: usize,
    job_id: &str,
) -> (JobStatus, Option<SkipReason>) {
    match states.get(index) {
        Some(s) if s.job_id == job_id => (s.status, s.skip_reason),
        _ => record_of(states, job_id),
    }
}

/// 記述順の位置 `index` のジョブ `job_id` の状態。記録が同じ位置にあればそれを使い(O(1))、
/// なければ [`record_of`] と同じに引く(記録が無ければ pending)。
///
/// ステートの記録は、ふつう定義の記述順に並ぶので、位置で引けばジョブの数に比例する時間で
/// 全ジョブを見られる(1,000 ジョブの計画の性能のため。U3 の BR6.3、BR2.3)。
pub(crate) fn status_at(states: &[JobState], index: usize, job_id: &str) -> JobStatus {
    record_at(states, index, job_id).0
}

/// ジョブ `job_id` の、定義の記述順の位置。手がかりの位置 `hint` のジョブが `job_id` なら、それを
/// 使い(O(1))、違えば `job_index` で探す(v0.10.1 の BR3.4)。
///
/// 手がかりが正しくなくても、`definition.job_index(job_id)` と同じ答えになる(定義のジョブ ID は
/// 一意。定義の検証が重複を拒む)。
pub(crate) fn position_of(
    definition: &PipelineDefinition,
    hint: usize,
    job_id: &str,
) -> Option<usize> {
    match definition.job.get(hint) {
        Some(j) if j.id == job_id => Some(hint),
        _ => definition.job_index(job_id),
    }
}

/// 計画の模擬が、ジョブごとに判断と記録の本体へ渡す目印(`PlanSimulationLookup`。v0.10.1 の
/// BR3.4、FR3.3)。
///
/// 目印は、定義の記述順に進む模擬が、前のジョブから続けて走査するためのもの。各目印の意味と、
/// それが正しいままである理由は次のとおり。
///
/// - `settled_prefix`: 定義の位置 `0..settled_prefix` のジョブは、すべて済んでいる(成功・警告・
///   スキップ。規則 6 の前のジョブの走査に使う)。
/// - `pending_prefix`: 定義の位置 `0..pending_prefix` のジョブに、pending(記録なしを含む)は無い
///   (`apply_skip` の「最初の pending」に使う)。
/// - `error_records`: ステートの記録のうち、`error` の数(`apply_skip` の「エラーがあるか」に使う)。
///   `None` は、まだ数えていない(要るときに 1 回だけ数える)。
///
/// 目印は、1 つのステートに対して、`flow` の記録の関数([`apply_outcome_in`]・[`apply_skip_in`])
/// だけでそのステートを変えながら使う。これらの関数は、済んだジョブを済んでいない状態に戻さず
/// (実行するのは pending か error のジョブだけ。規則 1〜3)、どのジョブも pending に戻さない。
/// だから 2 つの前置きの目印は、記録を変えても正しいまま。`error` の数は、記録の関数が状態を
/// 変えるたびに [`SimulationMarks::note_status_change`] で直す。
///
/// 今の公開の入口(`decide_from_state`・`apply_skip`・`apply_outcome`)は、呼ぶたびに
/// [`SimulationMarks::new`] で作り直す。どの目印も先頭から(数は未計数から)始まるので、今までと
/// 同じ走査になる。フィールドは `flow` の外から見えず、`plan` は値を持ち回るだけ。
#[derive(Debug, Clone, Default)]
pub(crate) struct SimulationMarks {
    settled_prefix: usize,
    pending_prefix: usize,
    error_records: Option<usize>,
}

impl SimulationMarks {
    /// 先頭から始める目印(どの前置きも 0、`error` の数は未計数)。
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// 定義の位置 `index` より前のジョブが、すべて済んでいるか(規則 6)。`settled_prefix` を、
    /// 済んでいないジョブか `index` に着くまで進める(前のジョブから続けて走査する)。
    fn predecessors_settled(
        &mut self,
        definition: &PipelineDefinition,
        states: &[JobState],
        index: usize,
    ) -> bool {
        while self.settled_prefix < index {
            let job = &definition.job[self.settled_prefix];
            if !is_settled(status_at(states, self.settled_prefix, &job.id)) {
                return false;
            }
            self.settled_prefix += 1;
        }
        true
    }

    /// 定義の順で最初の pending のジョブの位置(無ければ `None`)。`pending_prefix` を、pending の
    /// ジョブか定義の終わりに着くまで進める。
    fn first_pending(
        &mut self,
        definition: &PipelineDefinition,
        states: &[JobState],
    ) -> Option<usize> {
        while let Some(job) = definition.job.get(self.pending_prefix) {
            if status_at(states, self.pending_prefix, &job.id) == JobStatus::Pending {
                return Some(self.pending_prefix);
            }
            self.pending_prefix += 1;
        }
        None
    }

    /// ステートに `error` の記録があるか。数えていなければ、ここで 1 回だけ数える。
    fn has_error(&mut self, states: &[JobState]) -> bool {
        let count = *self.error_records.get_or_insert_with(|| {
            states
                .iter()
                .filter(|j| j.status == JobStatus::Error)
                .count()
        });
        count > 0
    }

    /// 記録の状態が `old` から `new` に変わったことを、`error` の数に映す(数えていなければ何もしない)。
    fn note_status_change(&mut self, old: JobStatus, new: JobStatus) {
        if let Some(count) = self.error_records.as_mut() {
            if old == JobStatus::Error {
                *count -= 1;
            }
            if new == JobStatus::Error {
                *count += 1;
            }
        }
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
