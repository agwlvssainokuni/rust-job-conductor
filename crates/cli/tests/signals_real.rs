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

//! 本番のシグナルの監視(`SystemSignals`。NFR4.3)。UNIX 系だけ。
//!
//! このテストのプロセスは、シグナルの監視を登録する(登録はプロセスで 1 回だけ)。
//! 登録は 1 つのテストだけで行う。2 つのシグナルを続けて送っても、最初の番号が残ることを確かめる。

#![cfg(unix)]

mod bin_support;
mod signal_support;

use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};

use bin_support::{
    LIMIT, conductor, force_kill, helper_command, helper_command_out, job_block, pid_in, text,
    wait_for_line,
};
use conductor_cli::{Signals, SystemSignals};
use conductor_runtime::InterruptSignal;
use signal_support::{send, send_self};

fn wait_for(signal: &InterruptSignal) -> Option<i32> {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(10) {
        if let Some(n) = signal.signal_number() {
            return Some(n);
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    None
}

#[test]
fn 最初のシグナルの番号だけが残る() {
    let signal = Arc::new(InterruptSignal::new());
    assert_eq!(SystemSignals.start(&signal), Ok(()));
    assert_eq!(signal.signal_number(), None);

    send_self("TERM");
    assert_eq!(wait_for(&signal), Some(15));
    // 続けて別のシグナルを送っても、最初の番号が残る(`SIGINT` を受け取っても、このプロセスは終わらない)
    send_self("INT");
    send_self("HUP");
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(signal.signal_number(), Some(15));
}

#[test]
fn log_dir_ありでも中断の_sigterm_は今と同じく_143_で_ログのファイルが残る() {
    // v0.10.0 の BR4.4、BR4.5、NFR1.11: 別のプロセスの conductor に送る(このプロセスの監視には触れない)。
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let out = dir.path().join("job.out");
    let def_path = dir.path().join("conductor.toml");
    std::fs::write(
        &def_path,
        format!(
            "format_version = 1\n[pipeline]\nname = \"p\"\nlog_dir = \"logs\"\n\n{}\n{}",
            job_block("sleeper", &helper_command_out(&out, "sleep"), ""),
            job_block("after", &helper_command("exit 0"), "")
        ),
    )
    .expect("書ける");

    let mut command = conductor(&["pipeline", "p", "--config"]);
    command
        .arg(&def_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let child = command.spawn().expect("起動できる");
    let conductor_pid = child.id();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(child.wait_with_output());
    });
    let cleanup = |out: &std::path::Path| {
        if let Some(pid) = pid_in(out) {
            force_kill(pid);
        }
        force_kill(conductor_pid);
    };
    if !wait_for_line(&out, "ready", Duration::from_secs(20)) {
        cleanup(&out);
        panic!("ジョブが始まらなかった");
    }
    let sent = Instant::now();
    send("TERM", conductor_pid);
    let output = match rx.recv_timeout(LIMIT) {
        Ok(result) => result.expect("待てる"),
        Err(_) => {
            cleanup(&out);
            panic!("時間の上限(30 秒)を超えました");
        }
    };
    assert_eq!(
        output.status.code(),
        Some(143),
        "stderr: {}",
        text(&output.stderr)
    );
    assert!(sent.elapsed() < Duration::from_secs(10));
    let logs: Vec<_> = std::fs::read_dir(dir.path().join("logs"))
        .expect("ログのディレクトリがある")
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        logs.len(),
        1,
        "sleeper の 1 つだけ(after は動かない): {logs:?}"
    );
    assert!(logs[0].starts_with("sleeper."), "{logs:?}");
}
