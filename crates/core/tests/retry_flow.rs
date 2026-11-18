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
    JobDefinition, JobExecution, RetryDecision, RetryPolicy, parse_definition, retry_decision,
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

fn exec(j: &JobDefinition) -> JobExecution {
    JobExecution {
        job_id: j.id.clone(),
        command: j.command.clone(),
        started: true,
        exit_code: None,
        terminated_by_signal: None,
        start_failure: false,
        interrupted: false,
        signal_number: None,
    }
}

fn exited(j: &JobDefinition, code: i32) -> JobExecution {
    JobExecution {
        exit_code: Some(code),
        ..exec(j)
    }
}

fn retry(wait_secs: u64) -> RetryDecision {
    RetryDecision::Retry { wait_secs }
}

// ---- 対象(BR2.1) ----

#[test]
fn 対象の終了コードなら_リトライする() {
    let j = job(WITH_RETRY);
    assert_eq!(retry_decision(&j, &exited(&j, 75), 0, 0), retry(30));
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
    let e = JobExecution {
        start_failure: true,
        exit_code: Some(127),
        ..exec(&j)
    };
    assert_eq!(retry_decision(&j, &e, 0, 0), RetryDecision::Stop);
}

#[test]
fn シグナルでの終了は_停止する() {
    let j = job(WITH_RETRY);
    let e = JobExecution {
        terminated_by_signal: Some(9),
        ..exec(&j)
    };
    assert_eq!(retry_decision(&j, &e, 0, 0), RetryDecision::Stop);
}

#[test]
fn 中断された実行は_停止する() {
    let j = job(WITH_RETRY);
    let e = JobExecution {
        interrupted: true,
        signal_number: Some(2),
        exit_code: Some(75),
        ..exec(&j)
    };
    assert_eq!(retry_decision(&j, &e, 0, 0), RetryDecision::Stop);
}

#[test]
fn 起動前に中断された実行は_停止する() {
    let j = job(WITH_RETRY);
    let e = JobExecution {
        started: false,
        interrupted: true,
        signal_number: Some(15),
        ..exec(&j)
    };
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
    assert_eq!(retry_decision(&j, &exited(&j, 127), 0, 0), retry(30));
}

#[test]
fn ジョブ自身の_127_でも_一覧になければ_停止する() {
    let j = job(NO_RETRY);
    let mut j = j;
    j.retry = Some(RetryPolicy {
        exit_codes: vec![75],
        max: 3,
        interval_secs: 1,
        multiplier: None,
        max_interval_secs: None,
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
        assert_eq!(retry_decision(&j, &e, done, 30), retry(30), "done={done}");
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
        multiplier: Some(multiplier),
        max_interval_secs: Some(max_interval),
    })
}

/// 判断の列を、呼び出し側と同じ手順(直前の待ち時間を引き継ぐ)で回し、待ち時間の列を返す。
fn waits(j: &JobDefinition, max: u32) -> Vec<u64> {
    let e = exited(j, 75);
    let mut last = 0;
    let mut out = Vec::new();
    for done in 0..max {
        match retry_decision(j, &e, done, last) {
            RetryDecision::Retry { wait_secs } => {
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
        retry_decision(&j, &e, 1, u64::MAX / 2),
        retry(u64::MAX),
        "掛け算は飽和し、上限で止まる"
    );
    assert_eq!(retry_decision(&j, &e, 2, u64::MAX), retry(u64::MAX));
}
