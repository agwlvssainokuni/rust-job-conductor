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

//! リトライの流れ(U2。`run` と `pipeline` の共通の処理。BR2.1〜BR2.13、FR1.2、FR3.x、FR4.1、FR5.x、FR6.1)。
//!
//! 偽の `Runtime` が、待たずに、呼ばれた待ち時間を記録する(NFR3)。

mod common;

use common::*;
use conductor_core::JobStatus;

/// a だけが、終了コード 75 でリトライする(最大 3 回、30 秒おき)。b、c は設定なし。
const RETRY_A: &str = r#"format_version = 1
[pipeline]
name = "p"
[[job]]
id = "a"
command = "echo a"
[job.retry]
exit_codes = [75]
max = 3
interval = "30s"
[[job]]
id = "b"
command = "echo b"
[[job]]
id = "c"
command = "echo c"
"#;

/// a が、倍率 2、上限 60 秒、最初 10 秒、最大 5 回でリトライする。
const BACKOFF_A: &str = r#"format_version = 1
[pipeline]
name = "p"
[[job]]
id = "a"
command = "echo a"
[job.retry]
exit_codes = [75]
max = 5
interval = "10s"
multiplier = 2
max_interval = "1m"
[[job]]
id = "b"
command = "echo b"
"#;

/// a が、終了コード 75 と 127 でリトライする(最大 2 回、1 秒おき)。
const RETRY_75_127: &str = r#"format_version = 1
[pipeline]
name = "p"
[[job]]
id = "a"
command = "echo a"
[job.retry]
exit_codes = [75, 127]
max = 2
interval = "1s"
"#;

/// ジョブの実行・待機・ステートの書き込みだけを、順に並べたログ。
fn flow_log(h: &Harness) -> Vec<String> {
    h.runtime
        .log()
        .into_iter()
        .filter(|l| l.starts_with("run:") || l.starts_with("wait:") || l.starts_with("write:"))
        .collect()
}

// ---- run ----

#[test]
fn run_2_回目に成功すると_0_で終わり_知らせは_v_なしでも出る() {
    let mut h = Harness::new(RETRY_A);
    h.runtime.script("a", &[Exec::Exit(75), Exec::Exit(0)]);
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 0);
    assert_eq!(h.runtime.ran(), ["a", "a"]);
    assert_eq!(*h.runtime.waits.borrow(), [30]);
    assert_eq!(
        out.err,
        "conductor: retry: job 'a' exited with code 75; retry 1 of 3 in 30 seconds\n"
    );
    assert_eq!(out.out, "");
    assert_eq!(
        status_in(&h.state().expect("ステート"), "a"),
        JobStatus::Success
    );
}

#[test]
fn run_使い切ると_最後の終了コードで終わり_再開位置はそのジョブ() {
    let mut h = Harness::new(RETRY_A);
    h.runtime.script(
        "a",
        &[
            Exec::Exit(75),
            Exec::Exit(75),
            Exec::Exit(75),
            Exec::Exit(75),
        ],
    );
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 75);
    assert_eq!(h.runtime.ran().len(), 4, "合計で max + 1 回");
    assert_eq!(*h.runtime.waits.borrow(), [30, 30, 30]);
    assert_eq!(out.err.matches("conductor: retry: ").count(), 3);
    assert!(out.err.contains("retry 3 of 3"), "{}", out.err);
    let state = h.state().expect("ステート");
    assert_eq!(status_in(&state, "a"), JobStatus::Error);
    assert_eq!(state.job_states[0].exit_code, Some(75));
    assert_eq!(state.resume_job_id.as_deref(), Some("a"));
}

#[test]
fn run_ステートは_最終の結果で_1_回だけ書かれ_途中では書かれない() {
    let mut h = Harness::new(RETRY_A);
    h.runtime
        .script("a", &[Exec::Exit(75), Exec::Exit(75), Exec::Exit(0)]);
    h.run(&["run", "a"]);
    assert_eq!(
        flow_log(&h),
        [
            "write:fast",
            "run:a",
            "wait:30",
            "run:a",
            "wait:30",
            "run:a",
            "write:durable"
        ]
    );
}

#[test]
fn run_対象外の結果は_リトライしない() {
    for (exec, code) in [
        (Exec::StartFailure, 127),
        (Exec::KilledBy(9), 137),
        (Exec::Exit(76), 76),
    ] {
        let mut h = Harness::new(RETRY_A);
        h.runtime.script("a", &[exec, Exec::Exit(0)]);
        let out = h.run(&["run", "a"]);
        assert_eq!(out.code, code, "{exec:?}");
        assert_eq!(h.runtime.ran(), ["a"], "{exec:?}");
        assert!(h.runtime.waits.borrow().is_empty(), "{exec:?}");
        assert!(!out.err.contains("conductor: retry:"), "{}", out.err);
    }
}

#[test]
fn run_リトライの設定がないジョブは_従来どおり() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime.script("a", &[Exec::Exit(75), Exec::Exit(0)]);
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 75);
    assert_eq!(h.runtime.ran(), ["a"]);
    assert!(h.runtime.waits.borrow().is_empty());
    assert_eq!(out.err, "");
    assert_eq!(
        flow_log(&h),
        ["write:fast", "run:a", "write:durable"],
        "待機も追加の実行もない"
    );
}

#[test]
fn run_ジョブ自身の_127_は_一覧にあればリトライし_起動の失敗はしない() {
    let mut h = Harness::new(RETRY_75_127);
    h.runtime.script("a", &[Exec::Exit(127), Exec::Exit(0)]);
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 0);
    assert_eq!(h.runtime.ran(), ["a", "a"]);

    let mut h = Harness::new(RETRY_75_127);
    h.runtime.script("a", &[Exec::StartFailure, Exec::Exit(0)]);
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 127);
    assert_eq!(h.runtime.ran(), ["a"]);
}

#[test]
fn run_倍率と上限の待ち時間の列() {
    let mut h = Harness::new(BACKOFF_A);
    h.runtime.script("a", &[Exec::Exit(75); 6]);
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 75);
    assert_eq!(h.runtime.ran().len(), 6);
    assert_eq!(*h.runtime.waits.borrow(), [10, 20, 40, 60, 60]);
}

#[test]
fn run_結果の表示は_最終の結果について_1_回だけ() {
    let mut h = Harness::new(RETRY_A);
    h.runtime
        .script("a", &[Exec::Exit(75), Exec::Exit(75), Exec::Exit(0)]);
    let out = h.run(&["run", "-v", "a"]);
    assert_eq!(out.code, 0);
    assert_eq!(out.err.matches("result=").count(), 1, "{}", out.err);
    assert!(out.err.contains("result=success"), "{}", out.err);
    assert_eq!(out.err.matches("conductor: retry: ").count(), 2);
}

#[test]
fn run_知らせは日本語でも出る() {
    let mut h = Harness::new(RETRY_A);
    h.env = FakeEnv::with(&[("LANG", "ja_JP.UTF-8")]);
    h.runtime.script("a", &[Exec::Exit(75), Exec::Exit(0)]);
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 0);
    assert!(out.err.contains("リトライします(1/3 回目)"), "{}", out.err);
}

#[test]
fn run_待機中のシグナルで_128_足す番号で終わり_エラーとして記録しない() {
    let mut h = Harness::new(RETRY_A);
    h.runtime.script("a", &[Exec::Exit(75), Exec::Exit(0)]);
    h.runtime.interrupt_wait.set(Some((0, 15)));
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 143);
    assert_eq!(h.runtime.ran(), ["a"], "待機のあとの実行はしない");
    assert!(!out.err.contains("conductor: error"), "{}", out.err);
    let state = h.state().expect("ステート");
    assert_ne!(status_in(&state, "a"), JobStatus::Error);
    assert_eq!(state.resume_job_id.as_deref(), Some("a"));
}

#[test]
fn run_2_回目の待機中のシグナルでも_同じ扱い() {
    let mut h = Harness::new(RETRY_A);
    h.runtime.script("a", &[Exec::Exit(75); 4]);
    h.runtime.interrupt_wait.set(Some((1, 2)));
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 130);
    assert_eq!(h.runtime.ran(), ["a", "a"]);
    assert_eq!(
        h.state().expect("ステート").resume_job_id.as_deref(),
        Some("a")
    );
}

#[test]
fn run_再開すると_そのジョブを_最初の実行から数え直す() {
    let mut h = Harness::new(RETRY_A);
    h.runtime.script("a", &[Exec::Exit(75), Exec::Exit(75)]);
    h.runtime.interrupt_wait.set(Some((1, 15)));
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 143);

    h.runtime.interrupt_wait.set(None);
    h.runtime.waits.borrow_mut().clear();
    h.runtime.log.borrow_mut().clear();
    h.runtime.script("a", &[Exec::Exit(75); 4]);
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 75);
    assert_eq!(h.runtime.ran().len(), 4, "再開後も max + 1 回");
    assert_eq!(*h.runtime.waits.borrow(), [30, 30, 30]);
}

// ---- pipeline ----

#[test]
fn pipeline_リトライして成功すると_次のジョブへ進み_完了する() {
    let mut h = Harness::new(RETRY_A);
    h.runtime.script("a", &[Exec::Exit(75), Exec::Exit(0)]);
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 0);
    assert_eq!(h.runtime.ran(), ["a", "a", "b", "c"]);
    assert_eq!(*h.runtime.waits.borrow(), [30]);
    assert!(h.state_raw().expect("ステート").completed);
}

#[test]
fn pipeline_使い切ると_そこで停止し_再開位置はそのジョブ() {
    let mut h = Harness::new(RETRY_A);
    h.runtime.script("a", &[Exec::Exit(75); 4]);
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 75);
    assert_eq!(h.runtime.ran(), ["a", "a", "a", "a"]);
    let state = h.state().expect("ステート");
    assert_eq!(status_in(&state, "a"), JobStatus::Error);
    assert_eq!(state.resume_job_id.as_deref(), Some("a"));
}

#[test]
fn pipeline_ステートは_ジョブごとの最終の結果で書かれ_途中では書かれない() {
    let mut h = Harness::new(RETRY_A);
    h.runtime.script("a", &[Exec::Exit(75), Exec::Exit(0)]);
    h.run(&["pipeline", "p"]);
    assert_eq!(
        flow_log(&h),
        [
            "write:fast",
            "run:a",
            "wait:30",
            "run:a",
            "write:durable",
            "run:b",
            "write:durable",
            "run:c",
            "write:durable"
        ]
    );
}

#[test]
fn pipeline_待機中のシグナルで_以降を実行せず_再開位置を記録する() {
    let mut h = Harness::new(RETRY_A);
    h.runtime.script("a", &[Exec::Exit(75), Exec::Exit(0)]);
    h.runtime.interrupt_wait.set(Some((0, 15)));
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 143);
    assert_eq!(h.runtime.ran(), ["a"]);
    let state = h.state().expect("ステート");
    assert_ne!(status_in(&state, "a"), JobStatus::Error);
    assert_eq!(state.resume_job_id.as_deref(), Some("a"));
}

#[test]
fn pipeline_対象外のジョブと設定のないジョブは_従来どおり() {
    let mut h = Harness::new(RETRY_A);
    h.runtime.script("a", &[Exec::Exit(0)]);
    h.runtime.script("b", &[Exec::Exit(75), Exec::Exit(0)]);
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 75, "b は設定がないので、そのままエラー");
    assert_eq!(h.runtime.ran(), ["a", "b"]);
    assert!(h.runtime.waits.borrow().is_empty());
}

// ---- v0.10.0: 既定のリトライ(`[pipeline.retry]`)と打ち消し(`retry = false`) ----

/// 既定は終了コード 75 で最大 2 回、5 秒おき。a は `retry` を書かず既定でリトライし、b は `retry = false`
/// でリトライしない(u1-pipeline-retry の BR2.2、FR1.2、FR1.4)。
const DEFAULT_RETRY: &str = r#"format_version = 1
[pipeline]
name = "p"
[pipeline.retry]
exit_codes = [75]
max = 2
interval = "5s"
[[job]]
id = "a"
command = "echo a"
[[job]]
id = "b"
command = "echo b"
retry = false
"#;

#[test]
fn 既定_retry_を書かないジョブは既定の終了コードでリトライして成功する() {
    let mut h = Harness::new(DEFAULT_RETRY);
    h.runtime.script("a", &[Exec::Exit(75), Exec::Exit(0)]);
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 0, "{}", out.err);
    assert_eq!(h.runtime.ran(), ["a", "a"]);
    assert_eq!(*h.runtime.waits.borrow(), [5]);
    assert_eq!(
        out.err,
        "conductor: retry: job 'a' exited with code 75; retry 1 of 2 in 5 seconds\n"
    );
}

#[test]
fn 既定_retry_が_false_のジョブは既定があってもリトライしない() {
    let mut h = Harness::new(DEFAULT_RETRY);
    // b を単独で動かせるよう、先に a を成功させる。
    h.runtime.script("a", &[Exec::Exit(0)]);
    assert_eq!(h.run(&["run", "a"]).code, 0);
    h.runtime.script("b", &[Exec::Exit(75), Exec::Exit(0)]);
    let out = h.run(&["run", "b"]);
    assert_eq!(out.code, 75, "{}", out.err);
    assert_eq!(h.runtime.ran(), ["a", "b"]);
    assert!(h.runtime.waits.borrow().is_empty());
    assert!(!out.err.contains("conductor: retry:"), "{}", out.err);
}

#[test]
fn 既定_pipeline_では既定のジョブはリトライし_false_のジョブはそこで止まる() {
    let mut h = Harness::new(DEFAULT_RETRY);
    h.runtime.script("a", &[Exec::Exit(75), Exec::Exit(0)]);
    h.runtime.script("b", &[Exec::Exit(75), Exec::Exit(0)]);
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 75, "b はリトライせずエラー");
    assert_eq!(h.runtime.ran(), ["a", "a", "b"]);
    assert_eq!(*h.runtime.waits.borrow(), [5]);
    let state = h.state().expect("ステート");
    assert_eq!(status_in(&state, "a"), JobStatus::Success);
    assert_eq!(status_in(&state, "b"), JobStatus::Error);
    assert_eq!(state.resume_job_id.as_deref(), Some("b"));
}

// ---- 偽の MemRuntime ----

#[test]
fn メモリの_runtime_でも_同じ流れで待たずに試せる() {
    let runtime = MemRuntime::new(RETRY_A);
    *runtime.exit_codes.borrow_mut() = [75, 75, 0, 0, 0].into_iter().collect();
    let (code, sink) = run_mem(&runtime, &["pipeline", "p"]);
    assert_eq!(code, 0);
    assert_eq!(*runtime.waits.borrow(), [30, 30]);
    assert_eq!(sink.err_text().matches("conductor: retry: ").count(), 2);

    let runtime = MemRuntime::new(RETRY_A);
    *runtime.exit_codes.borrow_mut() = [75, 0, 0, 0].into_iter().collect();
    runtime.interrupt_wait.set(Some((0, 2)));
    let (code, _) = run_mem(&runtime, &["pipeline", "p"]);
    assert_eq!(code, 130);
    assert_eq!(*runtime.waits.borrow(), [30]);
}

#[test]
fn リトライのたびに同じ分割済みのコマンドを渡す() {
    let first = conductor_runtime::CommandLine {
        program: "echo".to_string(),
        args: vec!["a".to_string()],
    };

    // 2 回リトライして成功: 試行の数(3)だけ記録され、すべて最初の分割と等しい。
    let mut h = Harness::new(RETRY_A);
    h.runtime
        .script("a", &[Exec::Exit(75), Exec::Exit(75), Exec::Exit(0)]);
    assert_eq!(h.run(&["run", "a"]).code, 0);
    let lines = h.runtime.run_lines.borrow().clone();
    assert_eq!(lines.len(), 3, "試行の数だけ渡す");
    assert!(
        lines.iter().all(|(id, line)| id == "a" && *line == first),
        "{lines:?}"
    );

    // 合図で起動しないときも、同じ分割を渡したことが記録される。
    let mut h = Harness::new(RETRY_A);
    h.signals = FakeSignals::new(SignalMode::RaiseOnStart(15));
    assert_eq!(h.run(&["run", "a"]).code, 143);
    assert_eq!(h.runtime.ran(), Vec::<String>::new(), "起動しない");
    assert_eq!(
        *h.runtime.run_lines.borrow(),
        [("a".to_string(), first.clone())]
    );
}

// ---- ログのファイル(v0.10.0 の U5。BR2.3、BR3.1、BR5.1〜BR5.3) ----

#[test]
fn br2_3_リトライの試行ごとに時刻を取り_試行の番号が増える() {
    let mut h = Harness::new(&with_log_dir(RETRY_A, "\"logs\""));
    h.runtime
        .script("a", &[Exec::Exit(75), Exec::Exit(75), Exec::Exit(0)]);
    assert_eq!(h.run(&["run", "a"]).code, 0);
    assert_eq!(h.runtime.now_calls.get(), 3);
    assert_eq!(
        log_attempts(&h.runtime),
        [
            ("a".to_string(), 1, fake_now(0)),
            ("a".to_string(), 2, fake_now(1)),
            ("a".to_string(), 3, fake_now(2)),
        ]
    );
    // 再開すると、試行の番号は 1 に戻る(CONDUCTOR_ATTEMPT と同じ)。
    let mut h = Harness::new(&with_log_dir(RETRY_A, "\"logs\""));
    h.runtime.script("a", &[Exec::Exit(1), Exec::Exit(0)]);
    assert_eq!(h.run(&["run", "a"]).code, 1);
    assert_eq!(h.run(&["run", "a"]).code, 0);
    let attempts: Vec<u32> = log_attempts(&h.runtime).iter().map(|a| a.1).collect();
    assert_eq!(attempts, [1, 1]);
}

#[test]
fn br5_2_警告は試行ごとに_リトライの知らせの前に出る() {
    let mut h = Harness::new(&with_log_dir(RETRY_A, "\"logs\""));
    let first = h.dir.path().join("logs").join("a.1.log");
    let third = h.dir.path().join("logs").join("a.3.log");
    h.runtime
        .script("a", &[Exec::Exit(75), Exec::Exit(75), Exec::Exit(0)]);
    {
        let mut outcomes = h.runtime.log_outcomes.borrow_mut();
        outcomes.push_back(conductor_runtime::LogOutcome {
            write_failed: Some(first.clone()),
            ..Default::default()
        });
        outcomes.push_back(conductor_runtime::LogOutcome::default());
        outcomes.push_back(conductor_runtime::LogOutcome {
            output_cut_short: Some(third.clone()),
            ..Default::default()
        });
    }
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 0, "警告では終了コードは変わらない");
    let lines: Vec<&str> = out.err.lines().collect();
    assert_eq!(lines.len(), 4, "{}", out.err);
    assert!(lines[0].starts_with("conductor: warning: failed to write to the log file"));
    assert!(lines[0].ends_with(&first.display().to_string()));
    assert!(lines[1].starts_with("conductor: retry: "));
    assert!(lines[2].starts_with("conductor: retry: "));
    assert!(lines[3].starts_with("conductor: warning: output continued after the job ended"));
    assert!(lines[3].ends_with(&third.display().to_string()));
}

#[test]
fn br5_1_ログを作れない起動の失敗は_127_が一覧にあってもリトライしない() {
    let mut h = Harness::new(&with_log_dir(RETRY_75_127, "\"logs\""));
    let path = h.dir.path().join("logs").join("a.1.log");
    h.runtime.script("a", &[Exec::StartFailure, Exec::Exit(0)]);
    h.runtime
        .log_outcomes
        .borrow_mut()
        .push_back(conductor_runtime::LogOutcome {
            log_not_created: Some(path.clone()),
            ..Default::default()
        });
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 127);
    assert_eq!(h.runtime.ran(), ["a"]);
    assert_eq!(
        out.err,
        format!(
            "conductor: error: cannot create the log file: {}\n",
            path.display()
        )
    );
}

#[test]
fn br5_1_リトライの後の試行でログを作れなければ_その試行の_1_行で終わる() {
    let mut h = Harness::new(&with_log_dir(RETRY_A, "\"logs\""));
    let path = h.dir.path().join("logs").join("a.2.log");
    h.runtime.script("a", &[Exec::Exit(75), Exec::StartFailure]);
    {
        let mut outcomes = h.runtime.log_outcomes.borrow_mut();
        outcomes.push_back(conductor_runtime::LogOutcome::default());
        outcomes.push_back(conductor_runtime::LogOutcome {
            log_not_created: Some(path.clone()),
            ..Default::default()
        });
    }
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 127);
    assert_eq!(
        out.err,
        format!(
            "conductor: retry: job 'a' exited with code 75; retry 1 of 3 in 30 seconds\n\
             conductor: error: cannot create the log file: {}\n",
            path.display()
        )
    );
}
