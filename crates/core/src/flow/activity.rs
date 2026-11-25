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

//! パイプラインの動きの様子の判定(v0.10.0 の U7。BR3.1)。入出力のない純粋な関数。
//!
//! `watch --follow` が、描くたびに読み直したステートから、表示を続けるか終わるかを決めるのに使う。

use crate::types::{JobStatus, PipelineState};

/// ステートから読み取る、パイプラインの動きの様子(U7 の BR3.1)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineActivity {
    /// `completed` が真。
    Completed,
    /// 終わりの時刻の無い試行を持つジョブがある(実行中、リトライの待ち、または強制終了の後)。
    Running,
    /// 動いている試行が無く、`error` のジョブがあるか、再開の位置のジョブが試行を持っている。
    Stopped,
    /// 上のどれでもない(まだ始まっていない、ジョブとジョブの間)。
    Between,
}

/// ステートの動きの様子を判定する(U7 の BR3.1)。
///
/// 判定の順は、完了 → 動いている試行 → 止まった → 間。リトライの待ちの間は最後の試行が開いたままなので
/// (BR3.4)、`Running` になる。
pub fn pipeline_activity(state: &PipelineState) -> PipelineActivity {
    if state.completed {
        return PipelineActivity::Completed;
    }
    let running = state
        .job_states
        .iter()
        .any(|j| j.attempts.last().is_some_and(|a| a.ended_at.is_none()));
    if running {
        return PipelineActivity::Running;
    }
    let has_error = state
        .job_states
        .iter()
        .any(|j| j.status == JobStatus::Error);
    let resume_tried = state.resume_job_id.as_deref().is_some_and(|id| {
        state
            .job_states
            .iter()
            .any(|j| j.job_id == id && !j.attempts.is_empty())
    });
    if has_error || resume_tried {
        PipelineActivity::Stopped
    } else {
        PipelineActivity::Between
    }
}
