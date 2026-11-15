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

//! バイナリを起動するテストの共通の補助(30 秒の時間の上限、定義の組み立て)。
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// プロセスを起動するテストの時間の上限。検知するのは、ハングだけである。
pub const LIMIT: Duration = Duration::from_secs(30);

pub fn conductor_path() -> &'static str {
    env!("CARGO_BIN_EXE_conductor")
}

pub fn helper_path() -> &'static str {
    env!("CARGO_BIN_EXE_conductor-cli-test-helper")
}

/// 補助の実行ファイルを呼ぶ command(パスは、一重引用符で囲む)。
pub fn helper_command(rest: &str) -> String {
    format!("'{}' {rest}", helper_path())
}

/// 補助の実行ファイルを `--out` つきで呼ぶ command。
pub fn helper_command_out(out: &Path, rest: &str) -> String {
    format!("'{}' --out '{}' {rest}", helper_path(), out.display())
}

/// ジョブ 1 つ分の定義(`extra` は、`warning_exit_codes` などの追加の行)。
pub fn job_block(id: &str, command: &str, extra: &str) -> String {
    format!("[[job]]\nid = \"{id}\"\ncommand = '''{command}'''\n{extra}\n")
}

pub fn definition(name: &str, jobs: &[String]) -> String {
    format!(
        "format_version = 1\n[pipeline]\nname = \"{name}\"\n\n{}",
        jobs.join("\n")
    )
}

/// `conductor` の Command(ロケールを英語に固定し、色の指定を外す)。
pub fn conductor(args: &[&str]) -> Command {
    let mut command = Command::new(conductor_path());
    command
        .args(args)
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env("LANG", "C")
        .env_remove("NO_COLOR");
    command
}

/// プロセスを強制終了する(テストが持つ子プロセスの後始末)。
pub fn force_kill(pid: u32) {
    #[cfg(unix)]
    let _ = Command::new("kill")
        .args(["-KILL", &pid.to_string()])
        .status();
    #[cfg(windows)]
    let _ = Command::new("taskkill")
        .args(["/F", "/PID", &pid.to_string()])
        .status();
}

/// 時間の上限(30 秒)つきで、子プロセスを起動して、出力を集める。
///
/// 超えたら、`cleanup`(テストが持つ、ほかの子プロセスの強制終了)を呼び、子プロセスを強制終了して、
/// 回収されるのを待ってから、失敗にする。
pub fn run_limited(mut command: Command, cleanup: impl FnOnce()) -> Output {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let child = command.spawn().expect("起動できる");
    let pid = child.id();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(child.wait_with_output());
    });
    match rx.recv_timeout(LIMIT) {
        Ok(result) => result.expect("終わりを待てる"),
        Err(_) => {
            cleanup();
            force_kill(pid);
            // 回収されるのを待つ(待っても、すぐに戻る)
            let _ = rx.recv_timeout(Duration::from_secs(10));
            panic!("時間の上限(30 秒)を超えました");
        }
    }
}

/// 出力のファイルに、その行が現れるまで待つ(上限つき)。
pub fn wait_for_line(out: &Path, wanted: &str, limit: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < limit {
        let found = std::fs::read_to_string(out)
            .map(|s| s.lines().any(|l| l == wanted))
            .unwrap_or(false);
        if found {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    false
}

/// 出力のファイルから、`pid=` の値を読む。
pub fn pid_in(out: &Path) -> Option<u32> {
    std::fs::read_to_string(out).ok().and_then(|s| {
        s.lines()
            .find_map(|l| l.strip_prefix("pid=").and_then(|n| n.parse().ok()))
    })
}

/// 一時ディレクトリに定義を書く。
pub struct Project {
    pub dir: tempfile::TempDir,
}

impl Project {
    pub fn new(definition_text: &str) -> Self {
        let dir = tempfile::tempdir().expect("一時ディレクトリ");
        std::fs::write(dir.path().join("conductor.toml"), definition_text).expect("書ける");
        Self { dir }
    }

    pub fn def_path(&self) -> PathBuf {
        self.dir.path().join("conductor.toml")
    }

    pub fn state_path(&self) -> PathBuf {
        self.dir.path().join("conductor.toml.state")
    }

    /// `conductor <args> --config <定義>` を、上限つきで実行する。
    pub fn run(&self, args: &[&str]) -> Output {
        let mut command = conductor(args);
        command.arg("--config").arg(self.def_path());
        run_limited(command, || {})
    }
}

pub fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}
