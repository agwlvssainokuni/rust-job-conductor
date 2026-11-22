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

//! リトライの判断(BR2.1〜BR2.3、BR2.16。FR1.2、FR2.1、FR2.2、FR3.1、FR3.2、FR3.6)。
//!
//! 引数だけから結果を決める純粋な関数。待つことも、起動することも、表示することもしない。

use crate::types::{ExecutionResult, JobDefinition, JobExecution, RetryPolicy};

/// リトライするかの判断(BR2.1〜BR2.3)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryDecision {
    /// 実行し直さない(対象外、または回数を使い切った)。
    Stop,
    /// `wait_secs` 秒待ってから、実行し直す。
    Retry { wait_secs: u64 },
}

/// 1 回の実行結果が、リトライの対象か(BR2.1、BR2.16)。
///
/// 起動して終了を観測し(`Exited`)、conductor の中断でない、シグナルで終了していない、
/// 終了コードが設定の集合に含まれる、の全部が成り立つときだけ真。
/// ジョブ自身が返した 127 は、一覧にあれば対象になる(起動の失敗は終了コードを持たないので、対象にならない)。
fn is_retry_target(policy: &RetryPolicy, execution: &JobExecution) -> bool {
    match execution.result {
        ExecutionResult::Exited {
            exit_code: Some(code),
            terminated_by_signal: None,
            interrupted_by: None,
        } => policy.exit_codes.contains(&code),
        _ => false,
    }
}

/// `retries_done` 回目まで実行し直した後の、k = `retries_done + 1` 回目のリトライの待ち時間(BR2.3)。
///
/// 固定の間隔(倍率と上限がない)なら `interval_secs`。倍率と上限があるときは、1 回目は
/// `interval_secs`、2 回目以降は「直前の待ち時間 × 倍率」を上限で頭打ちにする。計算は飽和で行う。
fn next_wait_secs(policy: &RetryPolicy, retries_done: u32, last_wait_secs: u64) -> u64 {
    match (policy.multiplier, policy.max_interval_secs) {
        (Some(multiplier), Some(max_interval)) => {
            if retries_done == 0 {
                policy.interval_secs.min(max_interval)
            } else {
                last_wait_secs
                    .saturating_mul(u64::from(multiplier))
                    .min(max_interval)
            }
        }
        _ => policy.interval_secs,
    }
}

/// ジョブの 1 回の実行結果から、リトライするかと、待ち時間を決める(BR2.1〜BR2.3、BR2.16)。
///
/// - `retries_done`: ここまでに実行し直した回数(最初の実行の直後は 0)。
/// - `last_wait_secs`: 直前のリトライの前の待ち時間(`retries_done` が 0 のときは使わない)。
///
/// リトライの設定がない、または対象外の結果なら `Stop`(これまでと同じ動き。BR2.9)。
/// 対象でも、`retries_done` が `max` に達していれば `Stop`(合計の実行は最大 `max + 1` 回。BR2.2)。
pub fn retry_decision(
    job: &JobDefinition,
    execution: &JobExecution,
    retries_done: u32,
    last_wait_secs: u64,
) -> RetryDecision {
    let Some(policy) = job.retry.as_ref() else {
        return RetryDecision::Stop;
    };
    if !is_retry_target(policy, execution) || retries_done >= policy.max {
        return RetryDecision::Stop;
    }
    RetryDecision::Retry {
        wait_secs: next_wait_secs(policy, retries_done, last_wait_secs),
    }
}
