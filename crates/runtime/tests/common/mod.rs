//! 統合テストの共通の補助(補助の実行ファイル、時間の上限、定義の組み立て)。
#![allow(dead_code)]

use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use conductor_core::JobDefinition;

/// プロセスを起動するテストの、時間の上限。
pub const LIMIT: Duration = Duration::from_secs(30);

/// 補助の実行ファイルのパス(`cargo test` が自動でビルドする)。
pub fn helper_path() -> &'static str {
    env!("CARGO_BIN_EXE_conductor-test-helper")
}

/// 一重引用符で囲んだ語(command の文字列に、そのまま埋め込める)。
pub fn q(text: &str) -> String {
    format!("'{text}'")
}

/// 補助の実行ファイルを呼ぶ command の文字列。`--out` に出力先を渡し、`rest` を続ける。
pub fn helper_command(out: &Path, rest: &str) -> String {
    format!(
        "{} --out {} {}",
        q(helper_path()),
        q(&out.display().to_string()),
        rest
    )
}

pub fn job(id: &str, command: String) -> JobDefinition {
    JobDefinition {
        id: id.to_string(),
        command,
        warning_exit_codes: vec![],
        success_exit_codes: None,
        transitions: vec![],
    }
}

/// 出力のファイルの行(まだ無ければ空)。
pub fn lines(out: &Path) -> Vec<String> {
    std::fs::read_to_string(out)
        .map(|s| s.lines().map(str::to_string).collect())
        .unwrap_or_default()
}

/// 出力のファイルに、その行が現れるまで待つ(上限つき)。現れなければ `false`。
pub fn wait_for_line(out: &Path, wanted: &str, limit: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < limit {
        if lines(out).iter().any(|l| l == wanted) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    false
}

/// 出力のファイルから、補助の実行ファイルのプロセス番号を読む。
pub fn pid_in(out: &Path) -> Option<u32> {
    lines(out)
        .iter()
        .find_map(|l| l.strip_prefix("pid=").and_then(|n| n.parse().ok()))
}

/// 補助の実行ファイルのプロセスを、強制終了する(テストが持つ子プロセスの後始末)。
pub fn force_kill(pid: u32) {
    #[cfg(unix)]
    let _ = std::process::Command::new("kill")
        .args(["-KILL", &pid.to_string()])
        .status();
    #[cfg(windows)]
    let _ = std::process::Command::new("taskkill")
        .args(["/F", "/PID", &pid.to_string()])
        .status();
}

/// root(権限の検査をすり抜ける)で動いているか。UNIX 系以外は `false`。
pub fn is_root() -> bool {
    #[cfg(unix)]
    {
        std::process::Command::new("id")
            .arg("-u")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "0")
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        false
    }
}

/// 別のスレッドで動かし、上限(30 秒)を超えたら、`cleanup`(子プロセスの強制終了)を呼んでから
/// 失敗にする。
pub fn run_with_limit<T, F>(cleanup: impl FnOnce(), f: F) -> T
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    match rx.recv_timeout(LIMIT) {
        Ok(v) => v,
        Err(_) => {
            cleanup();
            panic!("時間の上限(30 秒)を超えました");
        }
    }
}
