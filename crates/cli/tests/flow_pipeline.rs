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

//! 一括実行(`conductor pipeline`。W3、BR3.x、BR4.2)の流れ。AC3.1.x、AC3.2.x、AC4.3.2。

mod common;

use common::*;
use conductor_core::{JobState, JobStatus, PipelineState};
use conductor_runtime::{Durability, write_state};

#[test]
fn fr1_1_全部成功なら_記述順に実行し_完了のステートを残して_0() {
    let mut h = Harness::new(THREE_JOBS);
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 0);
    assert_eq!(out.out, "");
    assert_eq!(out.err, "");
    assert_eq!(h.runtime.ran(), ["a", "b", "c"]);
    assert!(h.state_path().exists(), "完了のステートが残る");
    assert!(h.state_raw().expect("残っている").completed);
    assert!(h.state().is_none(), "read_state は完了を空として返す");
    assert!(!h.env.locale_read(), "ロケールの判定が行われない");
    assert_eq!(
        h.runtime.log(),
        [
            "load",
            "read",
            "write:fast",
            "run:a",
            "write:durable",
            "run:b",
            "write:durable",
            "run:c",
            "write:durable",
        ],
        "削除(reset)を呼ばない"
    );
}

#[test]
fn fr1_2_完了のステートの形式は今までどおり() {
    let mut h = Harness::new(THREE_JOBS);
    assert_eq!(h.run(&["pipeline", "p"]).code, 0);
    let text = std::fs::read_to_string(h.state_path()).expect("読める");
    assert!(text.contains("format_version = 1"), "{text}");
    assert!(text.contains("completed = true"), "{text}");
    assert!(text.contains("pipeline_name = \"p\""), "{text}");
}

#[test]
fn fr2_1_remove_on_complete_を付けると_完了でステートを消して_0() {
    let mut h = Harness::new(THREE_JOBS);
    let out = h.run(&["pipeline", "p", "--remove-on-complete"]);
    assert_eq!(out.code, 0);
    assert_eq!(h.runtime.ran(), ["a", "b", "c"]);
    assert!(!h.state_path().exists(), "完了したので消える");
    assert_eq!(h.runtime.log().iter().filter(|l| *l == "reset").count(), 1);
}

#[test]
fn fr2_2_定義の_remove_on_complete_が_true_なら_オプションなしでも消える() {
    let mut h = Harness::new(&with_remove_on_complete(THREE_JOBS, "true"));
    assert_eq!(h.run(&["pipeline", "p"]).code, 0);
    assert!(!h.state_path().exists());
}

#[test]
fn fr2_3_オプションと定義の組み合わせで_オプションが優先して消す() {
    // オプションあり・定義 false → 消える
    let mut h = Harness::new(&with_remove_on_complete(THREE_JOBS, "false"));
    assert_eq!(h.run(&["pipeline", "p", "--remove-on-complete"]).code, 0);
    assert!(!h.state_path().exists());
    // オプションなし・定義 false → 残る
    let mut h = Harness::new(&with_remove_on_complete(THREE_JOBS, "false"));
    assert_eq!(h.run(&["pipeline", "p"]).code, 0);
    assert!(h.state_path().exists());
    // オプションあり・定義 true → 消える
    let mut h = Harness::new(&with_remove_on_complete(THREE_JOBS, "true"));
    assert_eq!(h.run(&["pipeline", "p", "--remove-on-complete"]).code, 0);
    assert!(!h.state_path().exists());
}

#[test]
fn fr2_6_環境変数では削除を切り替えない() {
    let mut h = Harness::new(THREE_JOBS);
    h.env = FakeEnv::with(&[
        ("CONDUCTOR_REMOVE_ON_COMPLETE", "1"),
        ("REMOVE_ON_COMPLETE", "true"),
    ]);
    assert_eq!(h.run(&["pipeline", "p"]).code, 0);
    assert!(h.state_path().exists(), "環境変数は見ないので残る");
}

#[test]
fn fr1_3_完了のステートが残っていても_pipeline_の再実行は先頭から始まる() {
    let mut h = Harness::new(THREE_JOBS);
    assert_eq!(h.run(&["pipeline", "p"]).code, 0);
    h.runtime.log.borrow_mut().clear();
    assert_eq!(h.run(&["pipeline", "p"]).code, 0);
    assert_eq!(h.runtime.ran(), ["a", "b", "c"], "先頭から全部を実行する");
    assert!(h.state_raw().expect("残る").completed);
}

#[test]
fn fr1_3_完了のステートが残っていても_先頭以外のジョブの_run_は_66() {
    let mut h = Harness::new(THREE_JOBS);
    assert_eq!(h.run(&["pipeline", "p"]).code, 0);
    h.runtime.log.borrow_mut().clear();
    let out = h.run(&["run", "b"]);
    assert_eq!(out.code, 66, "{}", out.err);
    assert_eq!(h.runtime.ran(), Vec::<String>::new());
}

#[test]
fn fr1_4_完了のステートの書き込みの失敗は_65() {
    let mut h = Harness::new(THREE_JOBS);
    // 0: 初期(Fast)、1: a、2: b、3: c の記録(完了)
    h.runtime.write_failure.set(WriteFailure::OnCall(3));
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 65);
    assert_eq!(h.runtime.ran(), ["a", "b", "c"]);
    assert!(
        !h.state_raw().is_some_and(|s| s.completed),
        "完了のステートは書けていない"
    );
}

#[test]
fn 警告の遷移で間のジョブをスキップする() {
    let mut h = Harness::new(WARNING_JUMPS);
    h.runtime.script("a", &[Exec::Exit(1)]);
    let out = h.run(&["pipeline", "p", "-v"]);
    assert_eq!(out.code, 0);
    assert_eq!(h.runtime.ran(), ["a", "c"]);
    assert!(h.state_raw().expect("残る").completed);
    assert!(
        out.err
            .contains("[conductor] transition: warning -> c (skipped: b)"),
        "{}",
        out.err
    );
    assert!(out.err.contains("[conductor] job=b skipped"), "{}", out.err);
}

#[test]
fn fr1_1_end_で完了して_完了のステートを残す() {
    let mut h = Harness::new(END_AFTER_A);
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 0);
    assert_eq!(h.runtime.ran(), ["a"]);
    assert!(h.state_raw().expect("残る").completed);
    assert_eq!(h.runtime.log().iter().filter(|l| *l == "reset").count(), 0);
}

#[test]
fn エラーで停止し_再試行せず_ジョブの終了コードで終わる() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime.script("b", &[Exec::Exit(5)]);
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 5);
    assert_eq!(
        h.runtime.ran(),
        ["a", "b"],
        "c は実行せず、b は再試行しない"
    );
    assert_eq!(status_of(&h, "a"), JobStatus::Success);
    assert_eq!(status_of(&h, "b"), JobStatus::Error);
    assert_eq!(
        h.state().expect("ステート").resume_job_id.as_deref(),
        Some("b")
    );
}

#[test]
fn エラーからの再開は_成功済みを飛ばして_エラーのジョブから() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime.script("b", &[Exec::Exit(5), Exec::Exit(0)]);
    assert_eq!(h.run(&["pipeline", "p"]).code, 5);
    h.runtime.log.borrow_mut().clear();
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 0);
    assert_eq!(h.runtime.ran(), ["b", "c"], "a は再実行しない");
    assert!(h.state_raw().expect("残る").completed);
    assert_eq!(
        h.runtime
            .log()
            .iter()
            .filter(|l| *l == "write:fast")
            .count(),
        0,
        "ステートがあるので、初期ステートを書かない"
    );
}

#[test]
fn 個別実行の続きから一括実行できる() {
    let mut h = Harness::new(THREE_JOBS);
    assert_eq!(h.run(&["run", "a"]).code, 0);
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 0);
    assert_eq!(h.runtime.ran(), ["a", "b", "c"], "a は 1 回だけ");
}

#[test]
fn fr1_1_スキップで完了した場合も_完了のステートを残す() {
    // a(@end)を個別に実行し、b・c は、一括実行のなかで、スキップとして記録される。
    let mut h = Harness::new(END_AFTER_A);
    assert_eq!(h.run(&["run", "a"]).code, 0);
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 0);
    assert_eq!(h.runtime.ran(), ["a"]);
    assert!(
        h.state_raw().expect("残る").completed,
        "最後のスキップで完了し、完了のステートが残った"
    );
}

#[test]
fn 遷移で飛ばされた最後のジョブより前でも_スキップの記録で完了を確かめる() {
    let mut h = Harness::new(WARNING_JUMPS);
    h.runtime.script("a", &[Exec::Exit(1)]);
    assert_eq!(h.run(&["run", "a"]).code, 0);
    assert_eq!(h.run(&["pipeline", "p"]).code, 0);
    assert_eq!(h.runtime.ran(), ["a", "c"]);
    assert!(h.state_raw().expect("残る").completed);
}

#[test]
fn 書き込みの失敗は_65_で_以降のジョブを起動しない() {
    let mut h = Harness::new(THREE_JOBS);
    // 0: 初期(Fast)、1: a の記録、2: b の記録
    h.runtime.write_failure.set(WriteFailure::OnCall(2));
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 65);
    assert!(
        out.err.contains("the job has already been executed"),
        "{}",
        out.err
    );
    assert_eq!(h.runtime.ran(), ["a", "b"]);
}

#[test]
fn 初期ステートが書けないときは_65_でジョブを起動しない() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime
        .write_failure
        .set(WriteFailure::OnDurability(Durability::Fast));
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 65);
    assert_eq!(h.runtime.run_calls.get(), 0);
}

#[test]
fn fr2_1_remove_on_complete_のとき_完了後の削除の失敗は_65() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime.fail_reset.set(true);
    let out = h.run(&["pipeline", "p", "--remove-on-complete"]);
    assert_eq!(out.code, 65);
    assert_eq!(h.runtime.ran(), ["a", "b", "c"]);
    assert!(
        h.state().is_none() && h.state_raw().is_some_and(|s| s.completed),
        "completed のまま残り、空として読まれる"
    );
}

#[test]
fn fr1_1_削除しないときは_削除の失敗の設定があっても_0() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime.fail_reset.set(true);
    assert_eq!(h.run(&["pipeline", "p"]).code, 0, "reset を呼ばない");
}

#[test]
fn fr2_1_スキップで完了して_remove_on_complete_なら消して_0() {
    // a(@end)を個別に実行し、b・c は、一括実行のなかで、スキップとして記録される。
    let mut h = Harness::new(END_AFTER_A);
    assert_eq!(h.run(&["run", "a"]).code, 0);
    h.runtime.log.borrow_mut().clear();
    let out = h.run(&["pipeline", "p", "--remove-on-complete"]);
    assert_eq!(out.code, 0);
    assert_eq!(h.runtime.ran(), Vec::<String>::new(), "a は再実行しない");
    assert!(!h.state_path().exists(), "最後のスキップで完了し、消える");
    let log = h.runtime.log();
    assert_eq!(
        log[log.len() - 2..],
        ["write:durable", "reset"],
        "c のスキップを記録してから消す: {log:?}"
    );
}

#[test]
fn br1_2_スキップで完了したときは_表示のあとで消し_削除の失敗は_65() {
    // pipeline は、スキップを表示してから消す(run は、表示の前に消す)。
    let mut h = Harness::new(END_AFTER_A);
    assert_eq!(h.run(&["run", "a"]).code, 0);
    h.runtime.fail_reset.set(true);
    let out = h.run(&["pipeline", "p", "--remove-on-complete", "-v"]);
    assert_eq!(out.code, 65);
    assert_eq!(h.runtime.ran(), ["a"], "a は個別実行の 1 回だけ");
    let skipped = out.err.find("job=c skipped").expect("c のスキップの表示");
    let failed = out
        .err
        .find("the job was not started")
        .expect("起動前の文脈の、削除の失敗の表示");
    assert!(skipped < failed, "表示のあとで消す: {}", out.err);
    assert!(
        h.state().is_none() && h.state_raw().is_some_and(|s| s.completed),
        "completed のまま残る"
    );
}

#[test]
fn ジョブのあいだの合図は_再開位置を記録して_128_足す番号() {
    let mut h = Harness::new(THREE_JOBS);
    *h.runtime.after_job.borrow_mut() = Some(Box::new(|id, signal| {
        if id == "a" {
            signal.raise(15);
        }
    }));
    let out = h.run(&["pipeline", "p", "-v"]);
    assert_eq!(out.code, 143);
    assert_eq!(h.runtime.ran(), ["a"], "b は起動しない");
    let state = h.state().expect("再開位置が残る");
    assert_eq!(state.resume_job_id.as_deref(), Some("b"));
    assert_eq!(status_of(&h, "a"), JobStatus::Success);
    assert_eq!(
        status_of(&h, "b"),
        JobStatus::Pending,
        "エラーとしては記録しない"
    );
    assert!(
        out.err.contains("[conductor] interrupted by signal 15"),
        "{}",
        out.err
    );

    // 再実行すると、b から続く
    *h.runtime.after_job.borrow_mut() = None;
    assert_eq!(h.run(&["pipeline", "p"]).code, 0);
    assert_eq!(h.runtime.ran(), ["a", "b", "c"]);
}

#[test]
fn ジョブのあいだの再開位置を書けないときは_65() {
    let mut h = Harness::new(THREE_JOBS);
    *h.runtime.after_job.borrow_mut() = Some(Box::new(|id, signal| {
        if id == "a" {
            signal.raise(2);
        }
    }));
    // 0: 初期、1: a の記録、2: 再開位置
    h.runtime.write_failure.set(WriteFailure::OnCall(2));
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 65);
    assert!(out.err.contains("the job was not started"), "{}", out.err);
    assert_eq!(h.runtime.ran(), ["a"]);
}

#[test]
fn ジョブの実行中の合図は_そのジョブをエラーとして記録して停止する() {
    let mut h = Harness::new(THREE_JOBS);
    *h.runtime.during_job.borrow_mut() = Some(Box::new(|id, signal| {
        if id == "b" {
            signal.raise(15);
        }
    }));
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 143);
    assert_eq!(h.runtime.ran(), ["a", "b"]);
    assert_eq!(status_of(&h, "b"), JobStatus::Error);
}

#[test]
fn 起動前の中断の結果の書き込みの失敗は_65_でジョブは起動していない() {
    let mut h = Harness::new(THREE_JOBS);
    h.signals = FakeSignals::new(SignalMode::RaiseOnStart(15));
    h.runtime
        .write_failure
        .set(WriteFailure::OnDurability(Durability::Durable));
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 65);
    assert!(out.err.contains("the job was not started"), "{}", out.err);
    assert!(!out.err.contains("already been executed"), "{}", out.err);
    assert_eq!(h.runtime.ran(), Vec::<String>::new());
}

#[test]
fn 起動の前の合図は_最初のジョブを起動せず_再開位置を記録する() {
    let mut h = Harness::new(THREE_JOBS);
    h.signals = FakeSignals::new(SignalMode::RaiseOnStart(2));
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 130);
    assert_eq!(h.runtime.ran(), Vec::<String>::new());
    assert_eq!(
        h.state().expect("ステート").resume_job_id.as_deref(),
        Some("a")
    );
}

#[test]
fn パイプライン名の不一致は_64() {
    let mut h = Harness::new(THREE_JOBS);
    let out = h.run(&["pipeline", "other"]);
    assert_eq!(out.code, 64);
    assert!(
        out.err.contains("pipeline name does not match"),
        "{}",
        out.err
    );
    assert_eq!(h.runtime.run_calls.get(), 0);
}

#[test]
fn 起動できないジョブで止まる() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime.script("a", &[Exec::StartFailure]);
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 127);
    assert!(out.err.contains("cannot start command 'echo'"));
    assert_eq!(h.runtime.ran(), ["a"]);
}

#[test]
fn 完了の判定ができないときは_65_で_初期化しない() {
    let mut h = Harness::new(THREE_JOBS);
    // すべて成功・警告で記録済みなのに、completed でない(起こらない想定の、壊れた内容)
    let jobs = ["a", "b", "c"];
    let state = PipelineState {
        pipeline_name: "p".into(),
        definition_path: h.def_path().to_string_lossy().into_owned(),
        job_states: jobs
            .iter()
            .map(|id| JobState {
                job_id: id.to_string(),
                status: JobStatus::Success,
                exit_code: Some(0),
                skip_reason: None,
            })
            .collect(),
        resume_job_id: None,
        end_reached: false,
        completed: false,
        run_id: None,
    };
    write_state(&h.def_path(), None, &state, Durability::Durable).expect("書ける");
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 65);
    assert!(
        out.err
            .contains("cannot determine whether the pipeline is complete"),
        "{}",
        out.err
    );
    assert!(h.state_path().exists(), "初期化しない");
    assert_eq!(h.runtime.run_calls.get(), 0);
}

#[test]
fn ステートの誤りと食い違いは_65_ジョブは起動しない() {
    let mut h = Harness::new(THREE_JOBS);
    std::fs::write(h.state_path(), "garbage = = =").expect("書ける");
    assert_eq!(h.run(&["pipeline", "p"]).code, 65);
    assert_eq!(h.runtime.run_calls.get(), 0);
}

#[test]
fn pipeline_は引数のパイプライン名がないと_68() {
    let mut h = Harness::new(THREE_JOBS);
    assert_eq!(h.run_raw(&["pipeline"]).code, 68);
    assert_eq!(h.signals.started.get(), 0);
}

#[test]
fn 個別実行と一括実行だけがシグナルを準備する() {
    let mut h = Harness::new(THREE_JOBS);
    h.run(&["pipeline", "p"]);
    assert_eq!(h.signals.started.get(), 1);
    h.run(&["watch"]);
    h.run(&["reset"]);
    assert_eq!(h.signals.started.get(), 1, "watch・reset では準備しない");
}
