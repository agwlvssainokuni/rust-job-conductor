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

//! 実際のシグナル(UNIX 系。AC2.5.6、BR4.x、NFR4.3): ジョブの実行中に SIGTERM を送ると、
//! 128 + 15 で終わり、ステートに再開位置が残る。
//!
//! Windows の Ctrl+C の経路は、自動化せず、リリース前の手動確認のチェックリストにした
//! (`WINDOWS_CTRL_C_CHECKLIST.md`)。

#![cfg(unix)]

mod bin_support;

use std::process::{Command, Stdio};
use std::time::Duration;

use bin_support::*;
use conductor_core::JobStatus;

fn send(signal: &str, pid: u32) {
    let status = Command::new("kill")
        .args([format!("-{signal}"), pid.to_string()])
        .status()
        .expect("kill を起動できる");
    assert!(status.success());
}

#[test]
fn ジョブの実行中の_sigterm_は_143_で_再開位置が残る() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let out = dir.path().join("job.out");
    let definition_text = definition(
        "p",
        &[
            job_block("sleeper", &helper_command_out(&out, "sleep"), ""),
            job_block("after", &helper_command("exit 0"), ""),
        ],
    );
    let def_path = dir.path().join("conductor.toml");
    std::fs::write(&def_path, definition_text).expect("書ける");

    let mut command = conductor(&["run", "sleeper", "--config"]);
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

    // ジョブが動き始めるまで待つ
    if !wait_for_line(&out, "ready", Duration::from_secs(20)) {
        cleanup(&out);
        panic!("ジョブが始まらなかった");
    }
    send("TERM", conductor_pid);

    let output = match rx.recv_timeout(LIMIT) {
        Ok(result) => result.expect("待てる"),
        Err(_) => {
            cleanup(&out);
            let _ = rx.recv_timeout(Duration::from_secs(10));
            panic!("時間の上限(30 秒)を超えました");
        }
    };
    assert_eq!(
        output.status.code(),
        Some(143),
        "stderr: {}",
        text(&output.stderr)
    );
    assert!(output.stdout.is_empty());

    let state = conductor_runtime::read_state(&def_path, None)
        .expect("読める")
        .expect("ステートが残る");
    assert_eq!(state.resume_job_id.as_deref(), Some("sleeper"));
    assert_eq!(state.job_states[0].status, JobStatus::Error);
    assert_eq!(state.job_states[1].status, JobStatus::Pending);
    // ジョブも終わっている(孤児が残らない)
    if let Some(pid) = pid_in(&out) {
        let alive = Command::new("kill")
            .args(["-0", &pid.to_string()])
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        assert!(!alive, "ジョブが残っている");
    }
}

#[test]
fn 続けて_2_つのシグナルを送っても_終了の処理は_1_回() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let out = dir.path().join("job.out");
    let def_path = dir.path().join("conductor.toml");
    std::fs::write(
        &def_path,
        definition(
            "p",
            &[job_block("sleeper", &helper_command_out(&out, "sleep"), "")],
        ),
    )
    .expect("書ける");

    let mut command = conductor(&["run", "sleeper", "--config"]);
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
    if !wait_for_line(&out, "ready", Duration::from_secs(20)) {
        if let Some(pid) = pid_in(&out) {
            force_kill(pid);
        }
        force_kill(conductor_pid);
        panic!("ジョブが始まらなかった");
    }
    // 最初の番号(HUP = 1)が残る。続けて INT を送っても、終了の処理は、メインの流れで 1 回だけ。
    send("HUP", conductor_pid);
    send("INT", conductor_pid);
    let output = match rx.recv_timeout(LIMIT) {
        Ok(result) => result.expect("待てる"),
        Err(_) => {
            if let Some(pid) = pid_in(&out) {
                force_kill(pid);
            }
            force_kill(conductor_pid);
            let _ = rx.recv_timeout(Duration::from_secs(10));
            panic!("時間の上限(30 秒)を超えました");
        }
    };
    assert_eq!(
        output.status.code(),
        Some(129),
        "stderr: {}",
        text(&output.stderr)
    );
    assert!(read_state_error_job(&def_path));
}

fn read_state_error_job(def_path: &std::path::Path) -> bool {
    conductor_runtime::read_state(def_path, None)
        .ok()
        .flatten()
        .map(|s| s.job_states[0].status == JobStatus::Error)
        .unwrap_or(false)
}
