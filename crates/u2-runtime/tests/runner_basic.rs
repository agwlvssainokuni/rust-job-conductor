//! `run_job` の統合テスト(補助の実行ファイルを、実際に起動する)。AC2.1.1、AC2.1.2、AC2.1.5、NFR4.1、NFR4.6。

mod common;

use common::*;
use conductor_runtime::{InterruptSignal, run_job};

fn run(command: String) -> conductor_core::JobExecution {
    run_with_limit(
        || {},
        move || run_job(&job("j", command), &InterruptSignal::new()),
    )
}

#[test]
fn 引数のメタ文字がそのまま渡る() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let out = dir.path().join("out.txt");
    let command = helper_command(
        &out,
        r#"echo 'a;b' '&&' '|' '$HOME' '*' '>x' "q\"r" 'it works' '' "#,
    );
    let e = run(command);
    assert!(e.started && !e.start_failure && !e.interrupted);
    assert_eq!(e.exit_code, Some(0));
    assert_eq!(e.terminated_by_signal, None);
    let expected = ["a;b", "&&", "|", "$HOME", "*", ">x", "q\"r", "it works", ""];
    let want: Vec<String> = expected
        .iter()
        .enumerate()
        .map(|(i, a)| format!("arg[{i}]={a:?}"))
        .collect();
    assert_eq!(lines(&out), want);
}

#[test]
fn 環境変数と作業ディレクトリを引き継ぐ() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let out = dir.path().join("out.txt");
    let e = run(helper_command(
        &out,
        "env CARGO_PKG_NAME CONDUCTOR_NO_SUCH_ENV_ZZ",
    ));
    assert_eq!(e.exit_code, Some(0));
    let name = std::env::var("CARGO_PKG_NAME").ok();
    assert_eq!(
        lines(&out),
        [
            format!("env[CARGO_PKG_NAME]={name:?}"),
            "env[CONDUCTOR_NO_SUCH_ENV_ZZ]=None".to_string(),
        ]
    );

    let out2 = dir.path().join("out2.txt");
    run(helper_command(&out2, "cwd"));
    let cwd = std::env::current_dir().ok();
    assert_eq!(lines(&out2), [format!("cwd={cwd:?}")]);
}

#[test]
fn 存在しないコマンドは起動失敗() {
    let e = run("conductor-no-such-command-zzz a b".to_string());
    assert!(e.started && e.start_failure);
    assert_eq!((e.exit_code, e.terminated_by_signal), (None, None));
    assert!(!e.interrupted);
}

#[test]
fn 実行できないファイルは起動失敗() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let not_exec = dir.path().join("data.txt");
    std::fs::write(&not_exec, "plain text").expect("書ける");
    let e = run(q(&not_exec.display().to_string()));
    assert!(e.started && e.start_failure);
    assert_eq!(e.exit_code, None);
}

#[test]
fn 終了コードの返り方() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let out = dir.path().join("out.txt");
    for (arg, want) in [(0, 0), (3, 3), (255, 255)] {
        let e = run(helper_command(&out, &format!("exit {arg}")));
        assert_eq!(e.exit_code, Some(want), "exit {arg}");
        assert!(!e.start_failure && e.started);
    }
    // 大きな値: UNIX 系は下位 8 ビット、Windows はそのまま。
    let e = run(helper_command(&out, "exit 300"));
    let want = if cfg!(unix) { 300 & 0xff } else { 300 };
    assert_eq!(e.exit_code, Some(want));
}

#[test]
fn 起動前の合図が立っていれば起動しない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let out = dir.path().join("out.txt");
    let command = helper_command(&out, "echo should-not-run");
    let e = run_with_limit(
        || {},
        move || {
            let signal = InterruptSignal::new();
            signal.raise(2);
            run_job(&job("j", command), &signal)
        },
    );
    assert!(!e.started && e.interrupted && !e.start_failure);
    assert_eq!(e.signal_number, Some(2));
    assert_eq!((e.exit_code, e.terminated_by_signal), (None, None));
    assert!(lines(&out).is_empty(), "ジョブは起動されない");
}

#[test]
fn 実行中に合図が立たなければ_interrupted_にならない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let out = dir.path().join("out.txt");
    let e = run(helper_command(&out, "exit 1"));
    assert!(!e.interrupted);
    assert_eq!(e.signal_number, None);
}

#[test]
fn 終了の観測の直後に合図が立っていれば_interrupted_を足す() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let out = dir.path().join("out.txt");
    let command = helper_command(&out, "sleep 2");
    let out2 = out.clone();
    let e = run_with_limit(
        move || {
            if let Some(pid) = pid_in(&out2) {
                force_kill(pid);
            }
        },
        move || {
            let signal = std::sync::Arc::new(InterruptSignal::new());
            let s = signal.clone();
            let o = out.clone();
            let t = std::thread::spawn(move || {
                wait_for_line(&o, "ready", LIMIT);
                // 転送されない番号(Windows では、何も送らない)。ジョブは自分で終わる。
                s.raise(10);
            });
            let e = run_job(&job("j", command), &signal);
            let _ = t.join();
            e
        },
    );
    // ジョブ自身の終了コードは、そのまま持つ。
    assert_eq!(e.exit_code, Some(99));
    assert!(e.interrupted);
    assert_eq!(e.signal_number, Some(10));
}

#[cfg(windows)]
#[test]
fn cmd_への危険な引数は起動失敗() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let script = dir.path().join("echo-args.cmd");
    std::fs::write(&script, "@echo off\r\nexit /b 0\r\n").expect("書ける");
    // 引用符を壊す引数。標準のライブラリが、安全に渡せないと判断して、起動を拒否する(NFR4.9)。
    let command = format!("{} 'x\" & echo injected'", q(&script.display().to_string()));
    let e = run(command);
    assert!(e.started && e.start_failure, "{e:?}");
}

#[cfg(windows)]
#[test]
fn cmd_への安全な引数は起動できる() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let script = dir.path().join("ok.cmd");
    std::fs::write(&script, "@echo off\r\nexit /b 7\r\n").expect("書ける");
    let e = run(format!("{} plain", q(&script.display().to_string())));
    assert_eq!(e.exit_code, Some(7), "{e:?}");
}
