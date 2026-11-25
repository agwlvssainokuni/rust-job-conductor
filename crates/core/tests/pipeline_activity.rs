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

//! パイプラインの動きの様子の判定(v0.10.0 の U7 の BR3.1)の統合テスト。公開の関数だけを使う。

mod common;

use common::state;
use conductor_core::{
    Attempt, JobStatus, PipelineActivity, PipelineState, Timestamp, pipeline_activity,
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

fn with_attempts(mut s: PipelineState, id: &str, attempts: Vec<Attempt>) -> PipelineState {
    s.job_states
        .iter_mut()
        .find(|j| j.job_id == id)
        .expect("ある")
        .attempts = attempts;
    s
}

#[test]
fn br3_1_完了のステートは_completed() {
    let s = state(
        &[("a", JobStatus::Success), ("b", JobStatus::Success)],
        None,
        true,
    );
    assert_eq!(pipeline_activity(&s), PipelineActivity::Completed);
}

#[test]
fn br3_1_完了は開いた試行より先に判定する() {
    let s = state(&[("a", JobStatus::Success)], None, true);
    let s = with_attempts(s, "a", vec![attempt(100, None)]);
    assert_eq!(pipeline_activity(&s), PipelineActivity::Completed);
}

#[test]
fn br3_1_終わりの無い試行があれば_running() {
    let s = state(
        &[("a", JobStatus::Success), ("b", JobStatus::Pending)],
        Some("b"),
        false,
    );
    let s = with_attempts(s, "a", vec![attempt(100, Some(110))]);
    let s = with_attempts(s, "b", vec![attempt(111, None)]);
    assert_eq!(pipeline_activity(&s), PipelineActivity::Running);
}

#[test]
fn br3_4_リトライの待ちの間の形は_running_初回() {
    // 初回の試行が失敗してリトライを待つ間: 状態は pending のまま、再開の位置はそのジョブ、
    // 最後の試行は開いたまま。
    let s = state(&[("a", JobStatus::Pending)], Some("a"), false);
    let s = with_attempts(s, "a", vec![attempt(100, None)]);
    assert_eq!(pipeline_activity(&s), PipelineActivity::Running);
}

#[test]
fn br3_4_リトライの待ちの間の形は_running_再開() {
    // 前回のエラーから再開して、リトライを待つ間: 状態は error のまま、最後の試行は開いたまま。
    let s = state(&[("a", JobStatus::Error)], Some("a"), false);
    let s = with_attempts(s, "a", vec![attempt(10, Some(20)), attempt(100, None)]);
    assert_eq!(pipeline_activity(&s), PipelineActivity::Running);
}

#[test]
fn br3_1_エラーのジョブの後の間は_stopped() {
    // a がエラーで終わり、遷移で b へ進む定義の、a と b の間。
    let s = state(
        &[("a", JobStatus::Error), ("b", JobStatus::Pending)],
        Some("b"),
        false,
    );
    let s = with_attempts(s, "a", vec![attempt(100, Some(110))]);
    assert_eq!(pipeline_activity(&s), PipelineActivity::Stopped);
}

#[test]
fn br3_1_再開の位置のジョブが試行を持てば_stopped() {
    // 中断でジョブの途中から止まった(状態は pending のまま、試行は閉じている)。
    let s = state(&[("a", JobStatus::Pending)], Some("a"), false);
    let s = with_attempts(s, "a", vec![attempt(100, Some(105))]);
    assert_eq!(pipeline_activity(&s), PipelineActivity::Stopped);
}

#[test]
fn br3_1_始まる前は_between() {
    let s = state(
        &[("a", JobStatus::Pending), ("b", JobStatus::Pending)],
        None,
        false,
    );
    assert_eq!(pipeline_activity(&s), PipelineActivity::Between);
}

#[test]
fn br3_1_ジョブとジョブの間は_between() {
    let s = state(
        &[("a", JobStatus::Success), ("b", JobStatus::Pending)],
        Some("b"),
        false,
    );
    let s = with_attempts(s, "a", vec![attempt(100, Some(110))]);
    assert_eq!(pipeline_activity(&s), PipelineActivity::Between);
}

#[test]
fn br3_1_再開の位置が定義に無いジョブでも_between() {
    let s = state(&[("a", JobStatus::Success)], Some("zzz"), false);
    assert_eq!(pipeline_activity(&s), PipelineActivity::Between);
}
