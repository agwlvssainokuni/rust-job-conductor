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

//! ログの指定つきの `run_job`(出力の複製。v0.10.0 の BR1.2、BR4.1〜BR4.5、BR5.1)の統合テスト
//! (補助の実行ファイルを、実際に起動する)。
//!
//! ジョブの出力は、このテストのプロセスの端末(標準出力と標準エラー)にも出る。端末の側の中身と、
//! 大量の出力の長さの一致は、`conductor` の出力を受け取れる cli の `binary_log.rs` で確かめる。
//! ここではファイルの中身と、結果の値を確かめる(出力は小さく保つ)。

mod common;

use std::fs;
use std::path::Path;

use common::*;
use conductor_core::{ExecutionResult, JobExecution, Timestamp};
use conductor_runtime::{InterruptSignal, LogOutcome, LogTarget, run_job};

/// 2026-11-25T09:12:48Z。
const STARTED: u64 = 1_795_597_968;
const TIME: &str = "20261125T091248Z";

fn target(dir: &Path, job_id: &str) -> LogTarget {
    LogTarget {
        dir: dir.to_path_buf(),
        job_id: job_id.to_string(),
        started_at: Timestamp::from_unix_seconds(STARTED).expect("範囲の中"),
        attempt: 1,
    }
}

fn run_logged(command: String, target: Option<LogTarget>) -> (JobExecution, LogOutcome) {
    run_with_limit(
        || {},
        move || {
            run_job(
                &command_line(command),
                &[],
                &[],
                &InterruptSignal::new(),
                target.as_ref(),
            )
        },
    )
}

fn exited(code: i32) -> ExecutionResult {
    ExecutionResult::Exited {
        exit_code: Some(code),
        terminated_by_signal: None,
        interrupted_by: None,
    }
}

/// 補助の実行ファイルを `--out` なしで呼ぶ command(出力はジョブの標準出力と標準エラー)。
fn helper(rest: &str) -> String {
    format!("{} {rest}", q(helper_path()))
}

#[test]
fn br4_1_標準出力と標準エラーの両方がファイルに残り_各流れの中の順を保つ() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (e, log) = run_logged(helper("streams 20"), Some(target(dir.path(), "both")));
    assert_eq!(e.result, exited(0));
    assert_eq!(log, LogOutcome::default());
    let text = fs::read_to_string(dir.path().join(format!("both.{TIME}.1.log"))).expect("読める");
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 40, "{text}");
    let outs: Vec<&str> = lines
        .iter()
        .copied()
        .filter(|l| l.starts_with("out "))
        .collect();
    let errs: Vec<&str> = lines
        .iter()
        .copied()
        .filter(|l| l.starts_with("err "))
        .collect();
    let want_out: Vec<String> = (0..20).map(|i| format!("out {i}")).collect();
    let want_err: Vec<String> = (0..20).map(|i| format!("err {i}")).collect();
    assert_eq!(outs, want_out);
    assert_eq!(errs, want_err);
}

#[test]
fn br4_2_ファイルにはジョブの出力だけを書く() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (e, _) = run_logged(helper("echo 'a b' c"), Some(target(dir.path(), "echo")));
    assert_eq!(e.result, exited(0));
    let bytes = fs::read(dir.path().join(format!("echo.{TIME}.1.log"))).expect("読める");
    assert_eq!(bytes, b"arg[0]=\"a b\"\narg[1]=\"c\"\n");
}

#[test]
fn br4_4_終了コードはログの有無で変わらない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (with_log, log) = run_logged(helper("exit 7"), Some(target(dir.path(), "seven")));
    let (without, none) = run_logged(helper("exit 7"), None);
    assert_eq!(with_log.result, exited(7));
    assert_eq!(without.result, with_log.result);
    assert_eq!(log, LogOutcome::default());
    assert_eq!(none, LogOutcome::default());
    // 出力の無いジョブでも、試行のファイルは空で残る。
    let path = dir.path().join(format!("seven.{TIME}.1.log"));
    assert_eq!(fs::read(path).expect("読める"), b"");
}

#[test]
fn br1_2_ログの指定が無ければ結果は空でファイルを作らない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let out = dir.path().join("out.txt");
    let (e, log) = run_logged(helper_command(&out, "run plain 0"), None);
    assert_eq!(e.result, exited(0));
    assert_eq!(log, LogOutcome::default());
    assert_eq!(lines(&out), ["ran=plain"]);
    assert_eq!(fs::read_dir(dir.path()).expect("読める").count(), 1);
}

#[test]
fn br5_1_ログを作れなければ起動せず_作れないパスを返す() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let blocker = dir.path().join("file-not-dir");
    fs::write(&blocker, b"").expect("書ける");
    let out = dir.path().join("out.txt");
    let (e, log) = run_logged(
        helper_command(&out, "run never 0"),
        Some(target(&blocker, "j")),
    );
    assert_eq!(e.result, ExecutionResult::StartFailed);
    assert_eq!(
        log,
        LogOutcome {
            log_not_created: Some(blocker.join(format!("j.{TIME}.1.log"))),
            ..LogOutcome::default()
        }
    );
    assert!(!out.exists(), "ジョブは起動されない");
}

#[test]
fn br3_1_ディレクトリが無ければ作ってから起動する() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let logs = dir.path().join("new").join("logs");
    let (e, log) = run_logged(helper("streams 1"), Some(target(&logs, "mk")));
    assert_eq!(e.result, exited(0));
    assert_eq!(log, LogOutcome::default());
    // 2 つの流れの間の順は読んだ順なので、行の集まりで比べる(BR4.1)。
    let text = fs::read_to_string(logs.join(format!("mk.{TIME}.1.log"))).expect("読める");
    let mut lines: Vec<&str> = text.lines().collect();
    lines.sort_unstable();
    assert_eq!(lines, ["err 0", "out 0"], "{text:?}");
}

#[cfg(unix)]
#[test]
fn br4_5_孫が出力を持ち続けても_10_秒以内に戻り_待ち切れを返す() {
    use std::time::{Duration, Instant};
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let started = Instant::now();
    let (e, log) = run_logged(helper("orphan 30"), Some(target(dir.path(), "orphan")));
    let elapsed = started.elapsed();
    let path = dir.path().join(format!("orphan.{TIME}.1.log"));
    // 孫の後始末(`sleep` は自分の pid を出力に書く。書く前に止めた場合は 30 秒で終わる)。
    let text = fs::read_to_string(&path).expect("読める");
    if let Some(pid) = text
        .lines()
        .find_map(|l| l.strip_prefix("pid=").and_then(|n| n.parse().ok()))
    {
        force_kill(pid);
    }
    assert_eq!(e.result, exited(0));
    assert!(elapsed < Duration::from_secs(10), "{elapsed:?}");
    assert!(elapsed >= Duration::from_secs(2), "2 秒は待つ: {elapsed:?}");
    assert_eq!(
        log,
        LogOutcome {
            output_cut_short: Some(path),
            ..LogOutcome::default()
        }
    );
    assert!(text.contains("parent done\n"), "{text}");
}

#[cfg(unix)]
#[test]
fn br4_4_ログありでもシグナルは今と同じく転送され_出力を待って戻る() {
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let out = dir.path().join("out.txt");
    let logs = dir.path().join("logs");
    let signal = Arc::new(InterruptSignal::new());
    let (tx, rx) = std::sync::mpsc::channel();
    {
        let signal = signal.clone();
        let command = helper_command(&out, "sleep 25");
        let target = target(&logs, "sleepy");
        std::thread::spawn(move || {
            let _ = tx.send(run_job(
                &command_line(command),
                &[],
                &[],
                &signal,
                Some(&target),
            ));
        });
    }
    assert!(wait_for_line(&out, "ready", Duration::from_secs(20)));
    let started = Instant::now();
    signal.raise(15);
    let (e, log) = match rx.recv_timeout(LIMIT) {
        Ok(result) => result,
        Err(_) => {
            if let Some(pid) = pid_in(&out) {
                force_kill(pid);
            }
            panic!("時間の上限(30 秒)を超えました");
        }
    };
    assert_eq!(
        e.result,
        ExecutionResult::Exited {
            exit_code: None,
            terminated_by_signal: Some(15),
            interrupted_by: Some(15),
        }
    );
    assert_eq!(log, LogOutcome::default());
    assert!(started.elapsed() < Duration::from_secs(10));
    assert!(logs.join(format!("sleepy.{TIME}.1.log")).exists());
}
