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

//! `run_job` の統合テスト(補助の実行ファイルを、実際に起動する)。AC2.1.1、AC2.1.2、AC2.1.5、NFR4.1、NFR4.6。
//!
//! 環境変数の引数(U1。BR4.3、BR4.4、FR1.3)は、補助の実行ファイルの `env` モードで、子のプロセスの
//! 環境を確かめる。環境変数は、プロセス全体には設定しない(並列のテストが干渉するため)。

mod common;

use common::*;
use conductor_core::{Command, ExecutionResult};
use conductor_runtime::{CommandLine, InterruptSignal, check_command, run_job};

fn run(command: Command) -> conductor_core::JobExecution {
    run_with_limit(
        || {},
        move || {
            let line = check_command(&command).expect("分けられる");
            run_job(&line, &[], &[], &InterruptSignal::new(), None).0
        },
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
    let e = run(Command::Line(command));
    assert_eq!(
        e.result,
        ExecutionResult::Exited {
            exit_code: Some(0),
            terminated_by_signal: None,
            interrupted_by: None,
        }
    );
    let expected = ["a;b", "&&", "|", "$HOME", "*", ">x", "q\"r", "it works", ""];
    let want: Vec<String> = expected
        .iter()
        .enumerate()
        .map(|(i, a)| format!("arg[{i}]={a:?}"))
        .collect();
    assert_eq!(lines(&out), want);
}

#[test]
fn 配列の_command_の要素は分割も展開もされずにそのまま渡る() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let out = dir.path().join("out.txt");
    let expected = [
        "a b",
        "'q'",
        "\"d\"",
        "$HOME",
        "a;b",
        "*",
        "",
        "  lead and trail  ",
    ];
    let mut argv: Vec<String> = vec![
        helper_path().to_string(),
        "--out".into(),
        out.display().to_string(),
        "echo".into(),
    ];
    argv.extend(expected.iter().map(|a| a.to_string()));
    let e = run(Command::Argv(argv));
    assert!(matches!(
        e.result,
        ExecutionResult::Exited {
            exit_code: Some(0),
            interrupted_by: None,
            ..
        }
    ));
    let want: Vec<String> = expected
        .iter()
        .enumerate()
        .map(|(i, a)| format!("arg[{i}]={a:?}"))
        .collect();
    assert_eq!(lines(&out), want);
}

#[test]
fn 配列の_command_の先頭が空なら起動失敗() {
    for argv in [vec![String::new()], vec!["  ".into(), "x".into()], vec![]] {
        // `check_command` は誤りを返すので、分割済みのコマンドを直接作る(先頭を `program`、
        // 無ければ空、残りを `args`)。`run_job` は、空の `program` を起動失敗にする。
        let mut iter = argv.clone().into_iter();
        let line = CommandLine {
            program: iter.next().unwrap_or_default(),
            args: iter.collect(),
        };
        let e = run_with_limit(
            || {},
            move || run_job(&line, &[], &[], &InterruptSignal::new(), None).0,
        );
        assert_eq!(e.result, ExecutionResult::StartFailed);
    }
}

#[test]
fn 配列の_command_で存在しないプログラムは起動失敗() {
    let e = run(Command::Argv(vec![
        "conductor-no-such-command-zzz".into(),
        "--flag".into(),
    ]));
    assert_eq!(e.result, ExecutionResult::StartFailed);
}

#[test]
fn 環境変数と作業ディレクトリを引き継ぐ() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let out = dir.path().join("out.txt");
    let e = run(Command::Line(helper_command(
        &out,
        "env CARGO_PKG_NAME CONDUCTOR_NO_SUCH_ENV_ZZ",
    )));
    assert!(matches!(
        e.result,
        ExecutionResult::Exited {
            exit_code: Some(0),
            ..
        }
    ));
    let name = std::env::var("CARGO_PKG_NAME").ok();
    assert_eq!(
        lines(&out),
        [
            format!("env[CARGO_PKG_NAME]={name:?}"),
            "env[CONDUCTOR_NO_SUCH_ENV_ZZ]=None".to_string(),
        ]
    );

    let out2 = dir.path().join("out2.txt");
    run(Command::Line(helper_command(&out2, "cwd")));
    let cwd = std::env::current_dir().ok();
    assert_eq!(lines(&out2), [format!("cwd={cwd:?}")]);
}

#[test]
fn 存在しないコマンドは起動失敗() {
    let e = run(Command::Line(
        "conductor-no-such-command-zzz a b".to_string(),
    ));
    assert_eq!(e.result, ExecutionResult::StartFailed);
}

#[test]
fn 実行できないファイルは起動失敗() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let not_exec = dir.path().join("data.txt");
    std::fs::write(&not_exec, "plain text").expect("書ける");
    let e = run(Command::Line(q(&not_exec.display().to_string())));
    assert_eq!(e.result, ExecutionResult::StartFailed);
}

/// `run_job` は渡した分割済みのコマンドで起動し、`job.command` を分け直さない(BR2.1)。
#[test]
fn 渡した分割済みのコマンドで起動し_command_を分け直さない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let out = dir.path().join("out.txt");
    let line = command_line(helper_command(&out, "exit 3"));
    // 定義の command は分けられない文字列。分け直せば起動失敗になる。
    assert!(check_command(&Command::Line("'".into())).is_err());
    let e = run_with_limit(
        || {},
        move || run_job(&line, &[], &[], &InterruptSignal::new(), None).0,
    );
    assert_eq!(
        e.result,
        ExecutionResult::Exited {
            exit_code: Some(3),
            terminated_by_signal: None,
            interrupted_by: None,
        }
    );
}

#[test]
fn 終了コードの返り方() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let out = dir.path().join("out.txt");
    for (arg, want) in [(0, 0), (3, 3), (255, 255)] {
        let e = run(Command::Line(helper_command(&out, &format!("exit {arg}"))));
        assert!(
            matches!(
                e.result,
                ExecutionResult::Exited { exit_code: Some(c), .. } if c == want
            ),
            "exit {arg}"
        );
    }
    // 大きな値: UNIX 系は下位 8 ビット、Windows はそのまま。
    let e = run(Command::Line(helper_command(&out, "exit 300")));
    let want = if cfg!(unix) { 300 & 0xff } else { 300 };
    assert!(matches!(
        e.result,
        ExecutionResult::Exited { exit_code: Some(c), .. } if c == want
    ));
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
            let line = command_line(command);
            run_job(&line, &[], &[], &signal, None).0
        },
    );
    assert_eq!(e.result, ExecutionResult::NotStarted { signal: 2 });
    assert!(lines(&out).is_empty(), "ジョブは起動されない");
}

#[test]
fn 実行中に合図が立たなければ_interrupted_にならない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let out = dir.path().join("out.txt");
    let e = run(Command::Line(helper_command(&out, "exit 1")));
    assert!(matches!(
        e.result,
        ExecutionResult::Exited {
            interrupted_by: None,
            ..
        }
    ));
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
            let line = command_line(command);
            let e = run_job(&line, &[], &[], &signal, None).0;
            let _ = t.join();
            e
        },
    );
    // ジョブ自身の終了コードは、そのまま持つ。
    assert!(matches!(
        e.result,
        ExecutionResult::Exited {
            exit_code: Some(99),
            interrupted_by: Some(10),
            ..
        }
    ));
}

// ---- 環境変数の引数(U1。BR4.3、BR4.4、FR1.3) ----

/// 子で `env <names...>` を実行し、出力の行を返す。
fn child_env(env: &[(&str, &str)], names: &[&str]) -> Vec<String> {
    child_env_unset(env, &[], names)
}

/// `unset` の名前を消し、`env` を足した子で `env <names...>` を実行し、出力の行を返す(U3)。
fn child_env_unset(env: &[(&str, &str)], unset: &[&str], names: &[&str]) -> Vec<String> {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let out = dir.path().join("out.txt");
    let command = helper_command(&out, &format!("env {}", names.join(" ")));
    let line = command_line(command);
    let e = run_job(&line, env, unset, &InterruptSignal::new(), None).0;
    assert!(matches!(
        e.result,
        ExecutionResult::Exited {
            exit_code: Some(0),
            ..
        }
    ));
    lines(&out)
}

#[test]
fn 足した値が子のプロセスに見える() {
    let got = child_env(
        &[
            ("CONDUCTOR_T_ADDED", "hello world"),
            ("CONDUCTOR_T_EMPTY", ""),
        ],
        &["CONDUCTOR_T_ADDED", "CONDUCTOR_T_EMPTY"],
    );
    assert_eq!(
        got,
        [
            "env[CONDUCTOR_T_ADDED]=Some(\"hello world\")",
            "env[CONDUCTOR_T_EMPTY]=Some(\"\")"
        ]
    );
}

#[test]
fn 足さなかった引き継いだ環境は残る() {
    // `cargo test` が子に渡す `CARGO_PKG_NAME` などではなく、必ずある `PATH` を使う。
    let inherited = std::env::var("PATH").expect("PATH がある");
    let got = child_env(&[("CONDUCTOR_T_OTHER", "x")], &["PATH"]);
    assert_eq!(got, [format!("env[PATH]=Some({inherited:?})")]);
}

#[test]
fn 足した値は引き継いだ値を上書きする() {
    let got = child_env(&[("PATH", "/overridden")], &["PATH"]);
    assert_eq!(got, ["env[PATH]=Some(\"/overridden\")"]);
}

#[test]
fn 同じ名前が並びに_2_回あれば後ろが勝つ() {
    let got = child_env(
        &[
            ("CONDUCTOR_T_TWICE", "first"),
            ("CONDUCTOR_T_TWICE", "second"),
        ],
        &["CONDUCTOR_T_TWICE"],
    );
    assert_eq!(got, ["env[CONDUCTOR_T_TWICE]=Some(\"second\")"]);
}

#[test]
fn 空の環境では従来どおりで足した名前は見えない() {
    let got = child_env(&[], &["CONDUCTOR_T_NOT_SET"]);
    assert_eq!(got, ["env[CONDUCTOR_T_NOT_SET]=None"]);
}

// ---- Windows の cmd(Windows だけ) ----

#[cfg(windows)]
#[test]
fn cmd_への危険な引数は注入にならない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let script = dir.path().join("echo-args.cmd");
    std::fs::write(&script, "@echo off\r\nexit /b 0\r\n").expect("書ける");
    let marker = dir.path().join("injected.txt");
    // 引用符を壊して、別のコマンドを続ける引数。標準のライブラリは、安全にエスケープして渡すか、
    // 渡せないと判断して起動を拒否する。どちらでも、続くコマンドは実行されない(NFR4.9)。
    let command = format!(
        "{} 'x\" & echo injected > {}'",
        q(&script.display().to_string()),
        marker.display()
    );
    let e = run(Command::Line(command));
    assert!(
        !matches!(e.result, ExecutionResult::NotStarted { .. }),
        "{e:?}"
    );
    assert!(
        !marker.exists(),
        "引数のメタ文字がコマンドとして実行された: {e:?}"
    );
}

#[cfg(windows)]
#[test]
fn cmd_への配列の危険な引数は注入にならない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let script = dir.path().join("echo-args.cmd");
    std::fs::write(&script, "@echo off\r\nexit /b 0\r\n").expect("書ける");
    let marker = dir.path().join("injected.txt");
    // 配列の要素は、そのまま引数になる。標準のライブラリが安全にエスケープして渡すか、渡せないと
    // 判断して起動を拒否する。どちらでも、続くコマンドは実行されない(FR2.3)。
    let e = run(Command::Argv(vec![
        script.display().to_string(),
        format!("x\" & echo injected > {}", marker.display()),
    ]));
    assert!(
        !matches!(e.result, ExecutionResult::NotStarted { .. }),
        "{e:?}"
    );
    assert!(
        !marker.exists(),
        "引数のメタ文字がコマンドとして実行された: {e:?}"
    );
}

#[cfg(windows)]
#[test]
fn cmd_への安全な引数は起動できる() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let script = dir.path().join("ok.cmd");
    std::fs::write(&script, "@echo off\r\nexit /b 7\r\n").expect("書ける");
    let e = run(Command::Line(format!(
        "{} plain",
        q(&script.display().to_string())
    )));
    assert!(
        matches!(
            e.result,
            ExecutionResult::Exited {
                exit_code: Some(7),
                ..
            }
        ),
        "{e:?}"
    );
}
