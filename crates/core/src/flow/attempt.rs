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

//! 試行の時刻の記録(v0.10.0 の U6。BR1.1〜BR1.4)。入出力のない純粋な関数。
//!
//! 本番の流れ(Cli の `run_job_with_retry`)が、試行ごとに [`start_attempt`] → 起動 →
//! [`end_attempt`](起動しなかった試行は [`cancel_attempt`])の順に呼ぶ。時刻は呼び出し側が
//! runtime の時計から取って渡す。[`super::apply_outcome`] と [`super::apply_skip`] は試行を変えない。
//!
//! どの関数も、ステートに無いジョブ ID は無視する(パニックしない。ほかの記録の関数と同じ)。

use crate::time::Timestamp;
use crate::types::{Attempt, JobState, PipelineState};

/// ジョブ `job_id` の記録を、書き換えられる形で引く。無ければ `None`。
fn job_mut<'a>(state: &'a mut PipelineState, job_id: &str) -> Option<&'a mut JobState> {
    state.job_states.iter_mut().find(|j| j.job_id == job_id)
}

/// 試行を始める(BR1.1、BR1.4)。
///
/// そのジョブの最後の試行に `ended_at` が無ければ(conductor が強制終了された後の再開)、その試行を
/// 自身の `started_at` で閉じる(時刻を作らない。FR6.9)。その後ろに `started_at = at` で終わりの無い
/// 試行を足す。前の試行は消さない(Q2)。
pub fn start_attempt(state: &PipelineState, job_id: &str, at: Timestamp) -> PipelineState {
    let mut out = state.clone();
    if let Some(job) = job_mut(&mut out, job_id) {
        if let Some(last) = job.attempts.last_mut()
            && last.ended_at.is_none()
        {
            last.ended_at = Some(last.started_at);
        }
        job.attempts.push(Attempt {
            started_at: at,
            ended_at: None,
        });
    }
    out
}

/// 最後の試行を閉じる(BR1.2)。最後の試行の `ended_at` を `at` にする。試行が無ければ何もしない。
pub fn end_attempt(state: &PipelineState, job_id: &str, at: Timestamp) -> PipelineState {
    let mut out = state.clone();
    if let Some(last) = job_mut(&mut out, job_id).and_then(|job| job.attempts.last_mut()) {
        last.ended_at = Some(at);
    }
    out
}

/// 最後の試行を取り除く(起動の前に中断の合図が立っていて、起動しなかった試行。BR1.3)。
/// 試行が無ければ何もしない。
pub fn cancel_attempt(state: &PipelineState, job_id: &str) -> PipelineState {
    let mut out = state.clone();
    if let Some(job) = job_mut(&mut out, job_id) {
        job.attempts.pop();
    }
    out
}
