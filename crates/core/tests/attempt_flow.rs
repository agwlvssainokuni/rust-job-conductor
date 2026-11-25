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

//! 試行の時刻の記録(v0.10.0 の U6。BR1.1〜BR1.4)の統合テスト。公開の関数だけを使う。
//!
//! 時刻は `Timestamp::from_unix_seconds` で作る(差し替え無し)。

mod common;

use common::state;
use conductor_core::{
    Attempt, Classification, JobExecution, JobOutcome, JobStatus, NextStep, PipelineState,
    SkipCause, Timestamp, apply_outcome, apply_skip, cancel_attempt, end_attempt, parse_definition,
    start_attempt,
};

fn ts(secs: u64) -> Timestamp {
    Timestamp::from_unix_seconds(secs).expect("範囲の中")
}

fn attempt(start: u64, end: Option<u64>) -> Attempt {
    Attempt {
        started_at: ts(start),
        ended_at: end.map(ts),
    }
}

fn attempts_of<'a>(state: &'a PipelineState, id: &str) -> &'a [Attempt] {
    &state
        .job_states
        .iter()
        .find(|j| j.job_id == id)
        .expect("ある")
        .attempts
}

fn two_jobs() -> PipelineState {
    state(
        &[("a", JobStatus::Pending), ("b", JobStatus::Pending)],
        None,
        false,
    )
}

#[test]
fn br1_1_試行を始めると終わりの無い試行が後ろに足される() {
    let s0 = two_jobs();
    let s = start_attempt(&s0, "a", ts(100));
    assert_eq!(attempts_of(&s, "a"), [attempt(100, None)]);
    assert!(attempts_of(&s, "b").is_empty(), "ほかのジョブは変わらない");
    assert!(attempts_of(&s0, "a").is_empty(), "元のステートは変わらない");
    assert_eq!(s.job_states[0].status, JobStatus::Pending, "状態は変えない");
}

#[test]
fn br1_2_試行を閉じると最後の試行に終わりが入る() {
    let s = start_attempt(&two_jobs(), "a", ts(100));
    let s = end_attempt(&s, "a", ts(117));
    assert_eq!(attempts_of(&s, "a"), [attempt(100, Some(117))]);
    // リトライの 2 回目も、最後の試行だけを閉じる。
    let s = start_attempt(&s, "a", ts(130));
    let s = end_attempt(&s, "a", ts(165));
    assert_eq!(
        attempts_of(&s, "a"),
        [attempt(100, Some(117)), attempt(130, Some(165))]
    );
}

#[test]
fn br1_4_閉じ忘れの最後の試行は自身の始まりで閉じてから足す() {
    // 強制終了の後の再開: 最後の試行に終わりが無い。
    let s = start_attempt(&two_jobs(), "a", ts(100));
    let s = start_attempt(&s, "a", ts(500));
    assert_eq!(
        attempts_of(&s, "a"),
        [attempt(100, Some(100)), attempt(500, None)],
        "時刻を作らず、自身の始まりで閉じる"
    );
}

#[test]
fn br1_4_再開しても前の試行を残して後ろに足す() {
    let mut s = two_jobs();
    for (start, end) in [(100, 110), (200, 230), (300, 345)] {
        s = start_attempt(&s, "a", ts(start));
        s = end_attempt(&s, "a", ts(end));
    }
    assert_eq!(
        attempts_of(&s, "a"),
        [
            attempt(100, Some(110)),
            attempt(200, Some(230)),
            attempt(300, Some(345))
        ]
    );
}

#[test]
fn br1_3_起動しなかった試行は取り除く() {
    let s = start_attempt(&two_jobs(), "a", ts(100));
    let s = end_attempt(&s, "a", ts(110));
    let s = start_attempt(&s, "a", ts(200));
    let s = cancel_attempt(&s, "a");
    assert_eq!(
        attempts_of(&s, "a"),
        [attempt(100, Some(110))],
        "それまでの試行はそのまま"
    );
}

#[test]
fn 試行が無ければ閉じても取り除いても変わらない() {
    let s0 = two_jobs();
    assert_eq!(end_attempt(&s0, "a", ts(1)), s0);
    assert_eq!(cancel_attempt(&s0, "a"), s0);
}

#[test]
fn 知らないジョブidは無視する() {
    let s0 = two_jobs();
    assert_eq!(start_attempt(&s0, "zz", ts(1)), s0);
    assert_eq!(end_attempt(&s0, "zz", ts(1)), s0);
    assert_eq!(cancel_attempt(&s0, "zz"), s0);
}

#[test]
fn 結果の記録とスキップの記録は試行を保つ() {
    let def = parse_definition(
        "format_version = 1\n[pipeline]\nname = \"p\"\n\
         [[job]]\nid = \"a\"\ncommand = \"c\"\n\
         [[job]]\nid = \"b\"\ncommand = \"c\"\n",
        "t.toml",
        None,
    )
    .expect("正しい定義");
    let s = start_attempt(&two_jobs(), "a", ts(100));
    let s = end_attempt(&s, "a", ts(117));
    let outcome = JobOutcome {
        classification: Some(Classification::Success),
        reported_exit_code: 0,
        conductor_error: false,
    };
    let next = NextStep {
        next_job_id: Some("b".into()),
        skipped_job_ids: vec![],
        skip_reason_kind: None,
        end_reached: false,
        completed: false,
        resume_job_id: None,
    };
    let s = apply_outcome(&s, "a", &JobExecution::exited(0), &outcome, &next);
    assert_eq!(s.job_states[0].status, JobStatus::Success);
    assert_eq!(attempts_of(&s, "a"), [attempt(100, Some(117))]);

    // スキップも、そのジョブとほかのジョブの試行を変えない。
    let s = start_attempt(&s, "b", ts(200));
    let s = end_attempt(&s, "b", ts(210));
    let s = apply_skip(&s, "b", SkipCause::SkipTarget, &def);
    assert_eq!(s.job_states[1].status, JobStatus::Skipped);
    assert_eq!(attempts_of(&s, "b"), [attempt(200, Some(210))]);
    assert_eq!(attempts_of(&s, "a"), [attempt(100, Some(117))]);
}
