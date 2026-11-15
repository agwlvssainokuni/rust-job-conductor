//! 一括実行(`conductor pipeline`。W3、BR3.x、BR4.2)の流れ。AC3.1.x、AC3.2.x、AC4.3.2。

mod common;

use common::*;
use conductor_core::{JobState, JobStatus, PipelineState};
use conductor_runtime::{Durability, write_state};

fn status_of(h: &Harness, id: &str) -> JobStatus {
    h.state()
        .expect("ステートがある")
        .job_states
        .iter()
        .find(|j| j.job_id == id)
        .expect("ジョブがある")
        .status
}

#[test]
fn 全部成功なら_記述順に実行し_完了で初期化して_0() {
    let mut h = Harness::new(THREE_JOBS);
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 0);
    assert_eq!(out.out, "");
    assert_eq!(out.err, "");
    assert_eq!(h.runtime.ran(), ["a", "b", "c"]);
    assert!(!h.state_path().exists(), "完了したので初期化される");
    assert_eq!(h.env.var_calls.get(), 0, "ロケールの判定が行われない");
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
            "reset"
        ]
    );
}

#[test]
fn 警告の遷移で間のジョブをスキップする() {
    let mut h = Harness::new(WARNING_JUMPS);
    h.runtime.script("a", &[Exec::Exit(1)]);
    let out = h.run(&["pipeline", "p", "-v"]);
    assert_eq!(out.code, 0);
    assert_eq!(h.runtime.ran(), ["a", "c"]);
    assert!(!h.state_path().exists());
    assert!(
        out.err
            .contains("[conductor] transition: warning -> c (skipped: b)"),
        "{}",
        out.err
    );
    assert!(out.err.contains("[conductor] job=b skipped"), "{}", out.err);
}

#[test]
fn end_で完了して初期化する() {
    let mut h = Harness::new(END_AFTER_A);
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 0);
    assert_eq!(h.runtime.ran(), ["a"]);
    assert!(!h.state_path().exists());
    assert_eq!(h.runtime.log().iter().filter(|l| *l == "reset").count(), 1);
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
    assert!(!h.state_path().exists());
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
fn スキップで完了した場合も初期化する() {
    // a(@end)を個別に実行し、b・c は、一括実行のなかで、スキップとして記録される。
    let mut h = Harness::new(END_AFTER_A);
    assert_eq!(h.run(&["run", "a"]).code, 0);
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 0);
    assert_eq!(h.runtime.ran(), ["a"]);
    assert!(
        !h.state_path().exists(),
        "最後のスキップで完了し、初期化された"
    );
}

#[test]
fn 遷移で飛ばされた最後のジョブより前でも_スキップの記録で完了を確かめる() {
    let mut h = Harness::new(WARNING_JUMPS);
    h.runtime.script("a", &[Exec::Exit(1)]);
    assert_eq!(h.run(&["run", "a"]).code, 0);
    assert_eq!(h.run(&["pipeline", "p"]).code, 0);
    assert_eq!(h.runtime.ran(), ["a", "c"]);
    assert!(!h.state_path().exists());
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
fn 完了後の初期化の失敗は_65() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime.fail_reset.set(true);
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 65);
    assert_eq!(h.runtime.ran(), ["a", "b", "c"]);
    assert!(
        h.state().is_none(),
        "completed のまま残り、空として読まれる"
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
        definition_key: h.def_path().to_string_lossy().into_owned(),
        pipeline_name: "p".into(),
        definition_path: h.def_path().to_string_lossy().into_owned(),
        job_order: jobs.iter().map(|s| s.to_string()).collect(),
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
    };
    write_state(&h.def_path(), &state, Durability::Durable).expect("書ける");
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
