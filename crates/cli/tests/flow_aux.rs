//! 補助のコマンド(`watch`・`reset`・`convert`。W4〜W6、BR7.x)と、入口(`run_cli_with`)。
//! AC5.1.x、AC6.1.x、AC7.1.x。

mod common;

use common::*;
use conductor_converter::{ConversionError, ConversionErrorKind};
use conductor_runtime::StateErrorKind;

const YAML_DEFINITION: &str = "\
format_version: 1
pipeline:
  name: p
job:
  - id: a
    command: echo a
";

// ---- watch ----

#[test]
fn watch_は表を標準出力に_1_回出す() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime.script("a", &[Exec::Exit(0)]);
    h.run(&["run", "a"]);
    let out = h.run(&["watch"]);
    assert_eq!(out.code, 0);
    assert_eq!(out.err, "");
    assert_eq!(
        out.out,
        "Pipeline: p\nJOB  STATUS   NOTE\na    success  exit code 0\nb    pending\nc    pending\n"
    );
    assert_eq!(h.sink.out_writes, 1);
}

#[test]
fn watch_はステートが空なら空であることを表示して_0() {
    let mut h = Harness::new(THREE_JOBS);
    let out = h.run(&["watch"]);
    assert_eq!(out.code, 0);
    assert_eq!(out.out, "no state (nothing has run yet)\n");
}

#[test]
fn watch_は完了のまま残ったステートを空として扱う() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime.fail_reset.set(true);
    h.run(&["pipeline", "p"]);
    let out = h.run(&["watch"]);
    assert_eq!(out.code, 0);
    assert!(out.out.contains("no state"), "{}", out.out);
}

#[test]
fn watch_は定義ファイルを読まず_検証しない() {
    let mut h = Harness::new(THREE_JOBS);
    h.run(&["run", "a"]);
    h.set_definition("this is not even toml = = =");
    h.runtime.log.borrow_mut().clear();
    let out = h.run(&["watch"]);
    assert_eq!(out.code, 0);
    assert!(out.out.contains("Pipeline: p"));
    assert_eq!(h.runtime.log(), ["read"], "定義は読まない");
}

#[test]
fn watch_のステートの誤りは_65() {
    let mut h = Harness::new(THREE_JOBS);
    std::fs::write(h.state_path(), "garbage = = =").expect("書ける");
    let out = h.run(&["watch"]);
    assert_eq!(out.code, 65);
    assert_eq!(out.out, "");
    assert!(out.err.contains("state file is corrupt"), "{}", out.err);
}

#[test]
fn watch_のファイルへの出力には制御文字がない() {
    let mut h = Harness::new(THREE_JOBS);
    h.set_definition(&THREE_JOBS.replace("id = \"a\"", "id = \"x\\u001b[31my\""));
    h.run(&["run", "x\u{1b}[31my"]);
    h.env.terminal = false;
    let out = h.run(&["watch"]);
    assert_eq!(out.code, 0);
    assert!(!out.out.contains('\x1b'), "{:?}", out.out);
    assert!(out.out.contains("x\\u{1b}[31my"), "{}", out.out);
}

#[test]
fn watch_は端末なら色をつける_unix() {
    let mut h = Harness::new(THREE_JOBS);
    h.run(&["run", "a"]);
    h.env.terminal = true;
    let out = h.run(&["watch"]);
    assert_eq!(out.out.contains('\x1b'), cfg!(unix));
    h.env.vars.insert("NO_COLOR".into(), "1".into());
    let reporter_out = {
        // 新しい Reporter は、実行のたびに作られる
        h.run(&["watch"]).out
    };
    assert!(!reporter_out.contains('\x1b'), "NO_COLOR");
}

#[test]
fn watch_の標準出力が閉じていたら_65() {
    let mut h = Harness::new(THREE_JOBS);
    h.run(&["run", "a"]);
    h.sink.fail_out = true;
    let out = h.run(&["watch"]);
    assert_eq!(out.code, 65);
    assert!(
        out.err.contains("cannot write to standard output"),
        "{}",
        out.err
    );
    // 空の表示でも同じ
    let mut h = Harness::new(THREE_JOBS);
    h.sink.fail_out = true;
    assert_eq!(h.run(&["watch"]).code, 65);
}

// ---- reset ----

#[test]
fn reset_はステートを消す_何も表示しない() {
    let mut h = Harness::new(THREE_JOBS);
    h.run(&["run", "a"]);
    assert!(h.state_path().exists());
    let out = h.run(&["reset"]);
    assert_eq!(out.code, 0);
    assert_eq!(out.out, "");
    assert_eq!(out.err, "");
    assert!(!h.state_path().exists());
}

#[test]
fn reset_はステートがなくても成功する() {
    let mut h = Harness::new(THREE_JOBS);
    assert_eq!(h.run(&["reset"]).code, 0);
    assert_eq!(h.run(&["reset"]).code, 0);
}

#[test]
fn reset_は壊れたステートと定義の誤りを迂回する() {
    let mut h = Harness::new("not a definition = =");
    std::fs::write(h.state_path(), "garbage = = =").expect("書ける");
    let out = h.run(&["reset"]);
    assert_eq!(out.code, 0);
    assert!(!h.state_path().exists());
    assert_eq!(h.runtime.log(), ["reset"], "読み込みも検証もしない");
}

#[test]
fn reset_の失敗は_65() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime.fail_reset.set(true);
    let out = h.run(&["reset"]);
    assert_eq!(out.code, 65);
    assert!(out.err.contains("cannot remove state file"), "{}", out.err);
}

#[test]
fn reset_のあとは先頭から実行できる() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime.script("a", &[Exec::Exit(3)]);
    assert_eq!(h.run(&["run", "a"]).code, 3);
    h.run(&["reset"]);
    assert_eq!(h.run(&["run", "a"]).code, 0);
}

// ---- convert ----

#[test]
fn convert_は変換結果を全部_1_回で標準出力に書く() {
    let mut h = Harness::new(THREE_JOBS);
    let input = h.dir.path().join("in.yaml");
    std::fs::write(&input, YAML_DEFINITION).expect("書ける");
    let out = h.run_os(vec![
        "convert".into(),
        input.into_os_string(),
        "--to".into(),
        "toml".into(),
    ]);
    assert_eq!(out.code, 0);
    assert_eq!(out.err, "");
    assert_eq!(h.sink.out_writes, 1, "1 回で書く");
    assert!(out.out.contains("[[job]]"), "{}", out.out);
    assert!(out.out.contains("format_version = 1"));
    // 変換した結果は、conductor の定義として読める
    std::fs::write(h.def_path(), &out.out).expect("書ける");
    assert_eq!(h.run(&["run", "a"]).code, 0);
}

#[test]
fn convert_の失敗は_67_で標準出力は空() {
    let mut h = Harness::new(THREE_JOBS);
    let input = h.dir.path().join("bad.yaml");
    std::fs::write(&input, "a: [unclosed\n").expect("書ける");
    let out = h.run_os(vec![
        "convert".into(),
        input.into_os_string(),
        "--to".into(),
        "toml".into(),
    ]);
    assert_eq!(out.code, 67);
    assert_eq!(out.out, "");
    assert_eq!(h.sink.out_writes, 0);
    assert!(out.err.contains("syntax error"), "{}", out.err);
    assert!(out.err.starts_with("conductor: error: "));
}

#[test]
fn convert_の入力がないときも_67() {
    let mut h = Harness::new(THREE_JOBS);
    let out = h.run_raw(&["convert", "no-such-file.yaml", "--to", "toml"]);
    assert_eq!(out.code, 67);
    assert_eq!(out.out, "");
    assert!(out.err.contains("input file does not exist"), "{}", out.err);
}

#[test]
fn convert_の標準出力が閉じていたら_67() {
    let mut h = Harness::new(THREE_JOBS);
    *h.runtime.convert_result.borrow_mut() = Some(Ok("a = 1\n".to_string()));
    h.sink.fail_out = true;
    let out = h.run_raw(&["convert", "x.yaml", "--to", "toml"]);
    assert_eq!(out.code, 67);
    assert!(
        out.err.contains("cannot write to standard output"),
        "{}",
        out.err
    );
}

#[test]
fn convert_は表せない値を場所つきで表示する() {
    let mut h = Harness::new(THREE_JOBS);
    *h.runtime.convert_result.borrow_mut() = Some(Err(ConversionError {
        file: "in.yaml".into(),
        line: Some(4),
        key_path: Some("job[1].note".into()),
        kind: ConversionErrorKind::Unrepresentable,
        reason: "null は表せません".into(),
    }));
    let out = h.run_raw(&["convert", "in.yaml", "--to", "toml"]);
    assert_eq!(out.code, 67);
    assert_eq!(out.out, "");
    assert!(
        out.err
            .contains("in.yaml:4: value cannot be represented in the target format"),
        "{}",
        out.err
    );
    assert!(out.err.contains("(at: job[1].note)"), "{}", out.err);
}

#[test]
fn convert_は定義もステートも読まず_シグナルも準備しない() {
    let mut h = Harness::new(THREE_JOBS);
    *h.runtime.convert_result.borrow_mut() = Some(Ok("x = 1\n".to_string()));
    let out = h.run_raw(&["convert", "x.yaml", "--to", "toml"]);
    assert_eq!(out.code, 0);
    assert_eq!(out.out, "x = 1\n");
    assert_eq!(h.runtime.log(), ["convert"]);
    assert_eq!(h.signals.started.get(), 0);
}

// ---- 入口 ----

#[test]
fn help_は使い方を標準出力に出して_0() {
    let mut h = Harness::new(THREE_JOBS);
    let out = h.run_raw(&["--help"]);
    assert_eq!(out.code, 0);
    assert!(out.out.starts_with("Usage:\n"), "{}", out.out);
    assert_eq!(out.err, "");
    let mut h = Harness::new(THREE_JOBS);
    h.env = FakeEnv::with(&[("LC_ALL", "ja_JP.UTF-8")]);
    assert!(h.run_raw(&["-h"]).out.starts_with("使い方:\n"));
}

#[test]
fn version_は版を標準出力に出して_0() {
    let mut h = Harness::new(THREE_JOBS);
    let out = h.run_raw(&["--version"]);
    assert_eq!(out.code, 0);
    assert_eq!(
        out.out,
        format!("conductor {}\n", env!("CARGO_PKG_VERSION"))
    );
    assert_eq!(h.env.var_calls.get(), 0, "ロケールを判定しない");
}

#[test]
fn 引数がなければ使い方を標準エラーに出して_68() {
    let mut h = Harness::new(THREE_JOBS);
    let out = h.run_raw(&[]);
    assert_eq!(out.code, 68);
    assert_eq!(out.out, "");
    assert!(
        out.err
            .starts_with("conductor: error: missing argument\n\nUsage:\n"),
        "{}",
        out.err
    );
}

#[test]
fn 使い方の誤りは日本語のロケールで日本語() {
    let mut h = Harness::new(THREE_JOBS);
    h.env = FakeEnv::with(&[("LANG", "ja_JP.UTF-8")]);
    let out = h.run_raw(&["start"]);
    assert_eq!(out.code, 68);
    assert!(
        out.err
            .starts_with("conductor: error: 知らないコマンドです: 'start'\n\n使い方:\n"),
        "{}",
        out.err
    );
}

#[test]
fn v_は_watch_と_reset_と_convert_に付けると_68() {
    let mut h = Harness::new(THREE_JOBS);
    assert_eq!(h.run_raw(&["watch", "-v"]).code, 68);
    assert_eq!(h.run_raw(&["reset", "-v"]).code, 68);
    assert_eq!(h.run_raw(&["convert", "f", "--to", "yaml", "-v"]).code, 68);
    assert!(h.runtime.log().is_empty(), "ジョブも読み込みもしない");
}

#[test]
fn ステートの読み込みの失敗の種類ごとに_65() {
    for kind in [
        StateErrorKind::Unreadable,
        StateErrorKind::TooLarge,
        StateErrorKind::Corrupt,
        StateErrorKind::VersionMismatch,
    ] {
        let mut h = Harness::new(THREE_JOBS);
        *h.runtime.read_failure.borrow_mut() = Some(state_error(&h.state_path(), kind));
        let out = h.run(&["watch"]);
        assert_eq!(out.code, 65, "{kind:?}");
        assert!(out.err.contains("conductor.toml.state"));
    }
}
