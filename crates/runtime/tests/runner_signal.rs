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

//! シグナルの転送の統合テスト(UNIX 系だけ。NFR4.8、BR1.7、AC2.5.6)。
//!
//! 「眠り続ける」補助の実行ファイルに、合図を立てる。ハンドラなしの既定の動作で終了するため、
//! `terminated_by_signal` で、シグナルが届いたことが分かる。
#![cfg(unix)]

mod common;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use common::*;
use conductor_core::{ExecutionResult, JobExecution};
use conductor_runtime::{InterruptSignal, check_command, run_job};

/// ジョブを別のスレッドで動かし、起動して準備ができたところで合図を立て、結果を受け取る。
fn run_and_raise(command: String, out: &Path, number: i32) -> JobExecution {
    let signal = Arc::new(InterruptSignal::new());
    let (tx, rx) = std::sync::mpsc::channel();
    {
        let signal = signal.clone();
        std::thread::spawn(move || {
            let j = job("j", command);
            let line = check_command(&j.command).expect("分けられる");
            let _ = tx.send(run_job(&line, &[], &signal));
        });
    }
    assert!(
        wait_for_line(out, "ready", Duration::from_secs(20)),
        "ジョブが準備できない"
    );
    signal.raise(number);
    match rx.recv_timeout(LIMIT) {
        Ok(e) => e,
        Err(_) => {
            if let Some(pid) = pid_in(out) {
                force_kill(pid);
            }
            panic!("時間の上限(30 秒)を超えました");
        }
    }
}

fn out_file() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let out = dir.path().join("out.txt");
    (dir, out)
}

#[test]
fn 実行中の合図は_sigterm_として届く() {
    let (_dir, out) = out_file();
    let e = run_and_raise(helper_command(&out, "sleep 25"), &out, 15);
    assert_eq!(
        e.result,
        ExecutionResult::Exited {
            exit_code: None,
            terminated_by_signal: Some(15),
            interrupted_by: Some(15),
        }
    );
}

#[test]
fn sigint_も転送される() {
    let (_dir, out) = out_file();
    let e = run_and_raise(helper_command(&out, "sleep 25"), &out, 2);
    assert!(matches!(
        e.result,
        ExecutionResult::Exited {
            terminated_by_signal: Some(2),
            interrupted_by: Some(2),
            ..
        }
    ));
}

#[test]
fn 転送しない番号の合図では送られず_interrupted_だけが立つ() {
    let (_dir, out) = out_file();
    // SIGUSR1 は、転送してよい集合にない。ジョブは、自分の時間(1 秒)で終わる。
    let usr1 = if cfg!(target_os = "macos") { 30 } else { 10 };
    let e = run_and_raise(helper_command(&out, "sleep 1"), &out, usr1);
    assert_eq!(
        e.result,
        ExecutionResult::Exited {
            exit_code: Some(99),
            terminated_by_signal: None,
            interrupted_by: Some(usr1),
        }
    );
}

#[test]
fn ジョブは新しいプロセスグループで起動される() {
    let (_dir, out) = out_file();
    let out2 = out.clone();
    let command = helper_command(&out, "sleep 25");
    let signal = Arc::new(InterruptSignal::new());
    let (tx, rx) = std::sync::mpsc::channel();
    {
        let signal = signal.clone();
        std::thread::spawn(move || {
            let j = job("j", command);
            let line = check_command(&j.command).expect("分けられる");
            let _ = tx.send(run_job(&line, &[], &signal));
        });
    }
    assert!(wait_for_line(&out2, "ready", Duration::from_secs(20)));
    let child = pid_in(&out2).expect("pid が出力される");
    let child_group = pgid_of(child);
    let own_group = pgid_of(std::process::id());
    signal.raise(15);
    if rx.recv_timeout(LIMIT).is_err() {
        force_kill(child);
        panic!("時間の上限(30 秒)を超えました");
    }
    assert_eq!(child_group, Some(child), "子のグループは、子の pid");
    assert_ne!(child_group, own_group);
}

#[test]
fn シグナルは_1回だけ転送される() {
    let (dir, out) = out_file();
    let hits = dir.path().join("hits.txt");
    let stop = dir.path().join("stop");
    // sh は、シグナルを受けるたびに 1 行書き、stop ができるまで動き続ける(数えるため)。
    let script = format!(
        "trap \"echo x >> {hits}\" TERM; echo pid=$$ >> {out}; echo ready >> {out}; \
         while [ ! -f {stop} ]; do sleep 0.02; done",
        hits = hits.display(),
        out = out.display(),
        stop = stop.display()
    );
    let command = format!("sh -c {}", q(&script));
    let signal = Arc::new(InterruptSignal::new());
    let (tx, rx) = std::sync::mpsc::channel();
    {
        let signal = signal.clone();
        std::thread::spawn(move || {
            let j = job("j", command);
            let line = check_command(&j.command).expect("分けられる");
            let _ = tx.send(run_job(&line, &[], &signal));
        });
    }
    assert!(wait_for_line(&out, "ready", Duration::from_secs(20)));
    signal.raise(15);
    signal.raise(15);
    assert!(
        wait_for_line(&hits, "x", Duration::from_secs(20)),
        "シグナルが届く"
    );
    // 追加の転送があれば、この間に届く(待ちの間隔は、最大 10ms)。
    std::thread::sleep(Duration::from_millis(400));
    std::fs::write(&stop, "").expect("書ける");
    let e = match rx.recv_timeout(LIMIT) {
        Ok(e) => e,
        Err(_) => {
            if let Some(pid) = pid_in(&out) {
                force_kill(pid);
            }
            panic!("時間の上限(30 秒)を超えました");
        }
    };
    assert_eq!(lines(&hits), ["x"], "転送は 1 回だけ");
    assert!(matches!(
        e.result,
        ExecutionResult::Exited {
            exit_code: Some(0),
            interrupted_by: Some(_),
            ..
        }
    ));
}

/// 終了済みのジョブにも、安全に動く(送信の判断は、回収の前だけ)。
#[test]
fn 先に終わったジョブは転送なしで観測される() {
    let (_dir, out) = out_file();
    let e = run_with_limit(|| {}, {
        let command = helper_command(&out, "exit 4");
        move || {
            let s = InterruptSignal::new();
            let j = job("j", command);
            let line = check_command(&j.command).expect("分けられる");
            let e = run_job(&line, &[], &s);
            s.raise(15);
            e
        }
    });
    assert!(
        matches!(
            e.result,
            ExecutionResult::Exited {
                exit_code: Some(4),
                interrupted_by: None,
                ..
            }
        ),
        "観測の後に立った合図は、次の呼び出しの前のものとして扱う"
    );
}

fn pgid_of(pid: u32) -> Option<u32> {
    let output = std::process::Command::new("ps")
        .args(["-o", "pgid=", "-p", &pid.to_string()])
        .output()
        .ok()?;
    String::from_utf8_lossy(&output.stdout).trim().parse().ok()
}
