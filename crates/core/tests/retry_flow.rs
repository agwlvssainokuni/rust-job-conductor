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

//! リトライの判断 `retry_decision` のテスト(U2。BR2.1〜BR2.3、BR2.9、BR2.16。FR1.2、FR2.1、FR2.2、FR3.1、FR3.2、FR3.6)。

use conductor_core::{
    Backoff, ExecutionResult, JobDefinition, JobExecution, RetryDecision, RetryPolicy,
    parse_definition, retry_decision,
};

const WITH_RETRY: &str = "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"c\"\n[job.retry]\nexit_codes = [75, 127]\nmax = 3\ninterval = \"30s\"\n";
const NO_RETRY: &str =
    "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"c\"\n";

fn job(text: &str) -> JobDefinition {
    parse_definition(text, "t.toml", None)
        .expect("正しい定義")
        .job
        .remove(0)
}

fn job_with_policy(policy: RetryPolicy) -> JobDefinition {
    let mut j = job(NO_RETRY);
    j.retry = Some(policy);
    j
}

fn exec(_j: &JobDefinition) -> JobExecution {
    JobExecution::new(ExecutionResult::Exited {
        exit_code: None,
        terminated_by_signal: None,
        interrupted_by: None,
    })
}

fn exited(_j: &JobDefinition, code: i32) -> JobExecution {
    JobExecution::exited(code)
}

/// 判断の待ち時間(`Retry` のときだけ `Some`)。
fn wait_of(d: RetryDecision) -> Option<u64> {
    match d {
        RetryDecision::Retry { wait_secs, .. } => Some(wait_secs),
        RetryDecision::Stop => None,
    }
}

// ---- 対象(BR2.1) ----

#[test]
fn 対象の終了コードなら_リトライする() {
    let j = job(WITH_RETRY);
    assert_eq!(wait_of(retry_decision(&j, &exited(&j, 75), 0, 0)), Some(30));
}

#[test]
fn リトライの設定がなければ_停止する() {
    let j = job(NO_RETRY);
    assert_eq!(
        retry_decision(&j, &exited(&j, 75), 0, 0),
        RetryDecision::Stop
    );
}

#[test]
fn 一覧にない終了コードは_停止する() {
    let j = job(WITH_RETRY);
    assert_eq!(
        retry_decision(&j, &exited(&j, 76), 0, 0),
        RetryDecision::Stop
    );
}

#[test]
fn 成功と警告の終了コードは_停止する() {
    let j = job(WITH_RETRY);
    assert_eq!(
        retry_decision(&j, &exited(&j, 0), 0, 0),
        RetryDecision::Stop
    );
    assert_eq!(
        retry_decision(&j, &exited(&j, 1), 0, 0),
        RetryDecision::Stop
    );
}

#[test]
fn 起動の失敗は_停止する() {
    let j = job(WITH_RETRY);
    // 起動の失敗は終了コードを持てない(一覧の 127 より優先することは、型で決まる)。
    let e = JobExecution::start_failed();
    assert_eq!(retry_decision(&j, &e, 0, 0), RetryDecision::Stop);
}

#[test]
fn シグナルでの終了は_停止する() {
    let j = job(WITH_RETRY);
    let e = JobExecution::new(ExecutionResult::Exited {
        exit_code: None,
        terminated_by_signal: Some(9),
        interrupted_by: None,
    });
    assert_eq!(retry_decision(&j, &e, 0, 0), RetryDecision::Stop);
}

#[test]
fn 中断された実行は_停止する() {
    let j = job(WITH_RETRY);
    let e = JobExecution::new(ExecutionResult::Exited {
        exit_code: Some(75),
        terminated_by_signal: None,
        interrupted_by: Some(2),
    });
    assert_eq!(retry_decision(&j, &e, 0, 0), RetryDecision::Stop);
}

#[test]
fn 起動前に中断された実行は_停止する() {
    let j = job(WITH_RETRY);
    let e = JobExecution::not_started(15);
    assert_eq!(retry_decision(&j, &e, 0, 0), RetryDecision::Stop);
}

#[test]
fn 終了コードがない実行は_停止する() {
    let j = job(WITH_RETRY);
    assert_eq!(retry_decision(&j, &exec(&j), 0, 0), RetryDecision::Stop);
}

#[test]
fn ジョブ自身の_127_は_一覧にあれば_リトライする() {
    let j = job(WITH_RETRY);
    assert_eq!(
        wait_of(retry_decision(&j, &exited(&j, 127), 0, 0)),
        Some(30)
    );
}

#[test]
fn ジョブ自身の_127_でも_一覧になければ_停止する() {
    let j = job(NO_RETRY);
    let mut j = j;
    j.retry = Some(RetryPolicy {
        exit_codes: vec![75],
        max: 3,
        interval_secs: 1,
        backoff: None,
    });
    assert_eq!(
        retry_decision(&j, &exited(&j, 127), 0, 0),
        RetryDecision::Stop
    );
}

// ---- 回数(BR2.2) ----

#[test]
fn max_回目までリトライし_その次で停止する() {
    let j = job(WITH_RETRY); // max = 3
    let e = exited(&j, 75);
    for done in 0..3 {
        assert_eq!(
            wait_of(retry_decision(&j, &e, done, 30)),
            Some(30),
            "done={done}"
        );
    }
    assert_eq!(retry_decision(&j, &e, 3, 30), RetryDecision::Stop);
    assert_eq!(retry_decision(&j, &e, 4, 30), RetryDecision::Stop);
}

// ---- 待ち時間(BR2.3) ----

fn backoff(interval: u64, multiplier: u32, max_interval: u64, max: u32) -> JobDefinition {
    job_with_policy(RetryPolicy {
        exit_codes: vec![75],
        max,
        interval_secs: interval,
        backoff: Some(Backoff {
            multiplier,
            max_interval_secs: max_interval,
        }),
    })
}

/// 判断の列を、呼び出し側と同じ手順(直前の待ち時間を引き継ぐ)で回し、待ち時間の列を返す。
fn waits(j: &JobDefinition, max: u32) -> Vec<u64> {
    let e = exited(j, 75);
    let mut last = 0;
    let mut out = Vec::new();
    for done in 0..max {
        match retry_decision(j, &e, done, last) {
            RetryDecision::Retry { wait_secs, .. } => {
                out.push(wait_secs);
                last = wait_secs;
            }
            RetryDecision::Stop => break,
        }
    }
    out
}

#[test]
fn 固定の間隔は_毎回同じ待ち時間() {
    let j = job(WITH_RETRY);
    assert_eq!(waits(&j, 3), vec![30, 30, 30]);
}

#[test]
fn 倍率と上限では_10_20_40_60_60_になる() {
    let j = backoff(10, 2, 60, 5);
    assert_eq!(waits(&j, 5), vec![10, 20, 40, 60, 60]);
}

#[test]
fn 上限と同じ間隔なら_ずっと同じ待ち時間() {
    let j = backoff(60, 3, 60, 3);
    assert_eq!(waits(&j, 3), vec![60, 60, 60]);
}

#[test]
fn 極端に大きい値でも_桁あふれしない() {
    let j = backoff(u64::MAX / 2, 10, u64::MAX, 3);
    let e = exited(&j, 75);
    assert_eq!(
        wait_of(retry_decision(&j, &e, 1, u64::MAX / 2)),
        Some(u64::MAX),
        "掛け算は飽和し、上限で止まる"
    );
    assert_eq!(wait_of(retry_decision(&j, &e, 2, u64::MAX)), Some(u64::MAX));
}

// ---- リトライの判断の中身(1f。BR2.4) ----

#[test]
fn リトライの判断は_最後の終了コードと最大回数を持つ() {
    let j = job(WITH_RETRY); // exit_codes = [75, 127]、max = 3、interval = 30s
    assert_eq!(
        retry_decision(&j, &exited(&j, 75), 0, 0),
        RetryDecision::Retry {
            wait_secs: 30,
            last_exit_code: 75,
            max: 3
        }
    );
    // ジョブ自身が返した 127 は、その終了コードを持つ。
    assert_eq!(
        retry_decision(&j, &exited(&j, 127), 0, 0),
        RetryDecision::Retry {
            wait_secs: 30,
            last_exit_code: 127,
            max: 3
        }
    );
    // 倍率つき(max 5)の 2 回目。
    let b = backoff(10, 2, 60, 5);
    assert_eq!(
        retry_decision(&b, &exited(&b, 75), 1, 10),
        RetryDecision::Retry {
            wait_secs: 20,
            last_exit_code: 75,
            max: 5
        }
    );
    // 一覧に無い終了コードと、回数を使い切ったときは停止。
    assert_eq!(
        retry_decision(&j, &exited(&j, 76), 0, 0),
        RetryDecision::Stop
    );
    assert_eq!(
        retry_decision(&j, &exited(&j, 75), 3, 30),
        RetryDecision::Stop
    );
}
