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

//! 引数の文法の表(NFR4.1、BR1.1〜BR1.6)。全コマンドの正しい引数と誤り。

mod common;

use std::ffi::OsString;

use conductor_cli::{
    CliCommand, CommandKind, GraphFormat, InitOptions, InitTarget, ParsedArgs, RunIdArg,
    UsageError, UsageErrorKind, parse_args,
};
use conductor_converter::TargetFormat;

fn os(args: &[&str]) -> Vec<OsString> {
    args.iter().map(OsString::from).collect()
}

fn parse(args: &[&str]) -> Result<ParsedArgs, UsageError> {
    parse_args(os(args))
}

fn command(args: &[&str]) -> CliCommand {
    match parse(args) {
        Ok(ParsedArgs::Command(cmd)) => cmd,
        other => panic!("コマンドになるはず: {args:?} -> {other:?}"),
    }
}

fn kind_of(args: &[&str]) -> UsageErrorKind {
    match parse(args) {
        Err(e) => e.kind,
        other => panic!("使い方の誤りになるはず: {args:?} -> {other:?}"),
    }
}

#[test]
fn run_はジョブidを受け取る() {
    let cmd = command(&["run", "extract"]);
    assert_eq!(cmd.kind, CommandKind::Run);
    assert_eq!(cmd.job_id.as_deref(), Some("extract"));
    assert!(!cmd.verbose);
    assert!(cmd.config_path.is_none());
}

#[test]
fn pipeline_はパイプライン名を受け取る() {
    let cmd = command(&["pipeline", "nightly"]);
    assert_eq!(cmd.kind, CommandKind::Pipeline);
    assert_eq!(cmd.pipeline_name.as_deref(), Some("nightly"));
}

#[test]
fn validate_はパイプライン名を省略でき_v_と_config_を受け付ける() {
    let cmd = command(&["validate"]);
    assert_eq!(cmd.kind, CommandKind::Validate);
    assert!(cmd.pipeline_name.is_none());
    assert!(!cmd.verbose);
    let cmd = command(&["validate", "nightly"]);
    assert_eq!(cmd.pipeline_name.as_deref(), Some("nightly"));
    let cmd = command(&["validate", "-v", "--config", "c.toml", "nightly"]);
    assert!(cmd.verbose);
    assert_eq!(cmd.config_path, Some(OsString::from("c.toml")));
    assert_eq!(cmd.pipeline_name.as_deref(), Some("nightly"));
}

#[test]
fn validate_の位置引数は_2_個以上と_to_が使い方の誤り() {
    assert_eq!(
        kind_of(&["validate", "a", "b"]),
        UsageErrorKind::UnexpectedArgument
    );
    assert_eq!(
        kind_of(&["validate", "--to", "yaml"]),
        UsageErrorKind::OptionNotAllowed
    );
    assert_eq!(kind_of(&["validate", "-vv"]), UsageErrorKind::InvalidValue);
}

#[test]
fn watch_と_reset_は位置引数を受け取らない() {
    assert_eq!(command(&["watch"]).kind, CommandKind::Watch);
    assert_eq!(command(&["reset"]).kind, CommandKind::Reset);
    assert_eq!(
        kind_of(&["watch", "extra"]),
        UsageErrorKind::UnexpectedArgument
    );
    assert_eq!(
        kind_of(&["reset", "extra"]),
        UsageErrorKind::UnexpectedArgument
    );
}

#[test]
fn convert_は入力と形式を受け取る() {
    let cmd = command(&["convert", "in.yaml", "--to", "toml"]);
    assert_eq!(cmd.kind, CommandKind::Convert);
    assert_eq!(cmd.convert_input, Some(OsString::from("in.yaml")));
    assert_eq!(cmd.convert_target, Some(TargetFormat::Toml));
    let cmd = command(&["convert", "--to=yaml", "in.toml"]);
    assert_eq!(cmd.convert_target, Some(TargetFormat::Yaml));
}

#[test]
fn config_は空白と等号の両方の形を受け付ける() {
    for cmd in ["run", "pipeline", "watch", "reset"] {
        let operand: &[&str] = if cmd == "watch" || cmd == "reset" {
            &[]
        } else {
            &["x"]
        };
        for form in [vec!["--config", "a/b.toml"], vec!["--config=a/b.toml"]] {
            let mut args = vec![cmd];
            args.extend(form);
            args.extend(operand);
            let parsed = command(&args);
            assert_eq!(
                parsed.config_path,
                Some(OsString::from("a/b.toml")),
                "{args:?}"
            );
        }
    }
}

#[test]
fn オプションはコマンドの前にも書ける() {
    let cmd = command(&["--config", "c.toml", "-v", "run", "x"]);
    assert_eq!(cmd.kind, CommandKind::Run);
    assert!(cmd.verbose);
    assert_eq!(cmd.config_path, Some(OsString::from("c.toml")));
}

#[test]
fn v_は_run_と_pipeline_だけ() {
    assert!(command(&["run", "-v", "x"]).verbose);
    assert!(command(&["run", "--verbose", "x"]).verbose);
    assert!(command(&["pipeline", "x", "-v"]).verbose);
    for args in [
        vec!["watch", "-v"],
        vec!["reset", "-v"],
        vec!["convert", "f", "--to", "yaml", "-v"],
    ] {
        assert_eq!(kind_of(&args), UsageErrorKind::OptionNotAllowed, "{args:?}");
    }
}

#[test]
fn v_はちょうど_1_回() {
    for args in [
        vec!["run", "-vv", "x"],
        vec!["run", "-v", "-v", "x"],
        vec!["run", "-v", "--verbose", "x"],
    ] {
        assert_eq!(kind_of(&args), UsageErrorKind::InvalidValue, "{args:?}");
    }
}

#[test]
fn config_は_convert_に付けられない() {
    assert_eq!(
        kind_of(&["convert", "f", "--to", "yaml", "--config", "c"]),
        UsageErrorKind::OptionNotAllowed
    );
}

#[test]
fn to_は_convert_だけで必須() {
    assert_eq!(
        kind_of(&["run", "x", "--to", "yaml"]),
        UsageErrorKind::OptionNotAllowed
    );
    assert_eq!(kind_of(&["convert", "f"]), UsageErrorKind::MissingArgument);
}

#[test]
fn to_の値は_yaml_か_toml_だけ() {
    for bad in ["json", "YAML", "", "yaml "] {
        assert_eq!(
            kind_of(&["convert", "f", "--to", bad]),
            UsageErrorKind::InvalidValue,
            "{bad:?}"
        );
    }
}

#[test]
fn 値の欠落は_missing_value() {
    assert_eq!(
        kind_of(&["run", "x", "--config"]),
        UsageErrorKind::MissingValue
    );
    assert_eq!(
        kind_of(&["convert", "f", "--to"]),
        UsageErrorKind::MissingValue
    );
}

#[test]
fn 重複する_config_と_to_は誤り() {
    assert_eq!(
        kind_of(&["run", "x", "--config", "a", "--config", "b"]),
        UsageErrorKind::InvalidValue
    );
    assert_eq!(
        kind_of(&["convert", "f", "--to", "yaml", "--to", "toml"]),
        UsageErrorKind::InvalidValue
    );
}

#[test]
fn 引数がない() {
    assert_eq!(kind_of(&[]), UsageErrorKind::MissingArgument);
    assert_eq!(kind_of(&["-v"]), UsageErrorKind::MissingArgument);
}

#[test]
fn 知らないコマンドとオプション() {
    assert_eq!(kind_of(&["start"]), UsageErrorKind::UnknownCommand);
    assert_eq!(
        kind_of(&["run", "x", "--bogus"]),
        UsageErrorKind::UnknownOption
    );
    assert_eq!(kind_of(&["run", "x", "-x"]), UsageErrorKind::UnknownOption);
    assert_eq!(
        kind_of(&["run", "x", "--bogus=1"]),
        UsageErrorKind::UnknownOption
    );
    assert_eq!(
        kind_of(&["run", "x", "--verbose=1"]),
        UsageErrorKind::InvalidValue
    );
}

#[test]
fn 必須の引数の欠落と余分な引数() {
    assert_eq!(kind_of(&["run"]), UsageErrorKind::MissingArgument);
    assert_eq!(kind_of(&["pipeline"]), UsageErrorKind::MissingArgument);
    assert_eq!(
        kind_of(&["convert", "--to", "yaml"]),
        UsageErrorKind::MissingArgument
    );
    assert_eq!(
        kind_of(&["run", "a", "b"]),
        UsageErrorKind::UnexpectedArgument
    );
    assert_eq!(
        kind_of(&["convert", "a", "b", "--to", "yaml"]),
        UsageErrorKind::UnexpectedArgument
    );
}

#[test]
fn ダブルハイフンのあとは位置引数() {
    let cmd = command(&["run", "--", "-weird"]);
    assert_eq!(cmd.job_id.as_deref(), Some("-weird"));
    let cmd = command(&["run", "--config", "c", "--", "--help"]);
    assert_eq!(cmd.job_id.as_deref(), Some("--help"));
    // `--` のあとの `-v` は、位置引数(余分な引数)
    assert_eq!(
        kind_of(&["run", "x", "--", "-v"]),
        UsageErrorKind::UnexpectedArgument
    );
}

#[test]
fn help_と_version_は先に処理される() {
    for args in [["--help"], ["-h"]] {
        assert_eq!(parse(&args), Ok(ParsedArgs::Help));
    }
    for args in [["--version"], ["-V"]] {
        assert_eq!(parse(&args), Ok(ParsedArgs::Version));
    }
    // ほかの誤りより先
    assert_eq!(parse(&["run", "-vv", "--help"]), Ok(ParsedArgs::Help));
    assert_eq!(parse(&["bogus", "--bogus", "-V"]), Ok(ParsedArgs::Version));
    assert_eq!(parse(&["watch", "-v", "-h"]), Ok(ParsedArgs::Help));
    // 先に現れた方
    assert_eq!(parse(&["-V", "-h"]), Ok(ParsedArgs::Version));
    assert_eq!(parse(&["-h", "-V"]), Ok(ParsedArgs::Help));
}

#[test]
fn ダブルハイフンのあとの_help_は処理しない() {
    assert_eq!(kind_of(&["--", "--help"]), UsageErrorKind::UnknownCommand);
}

#[test]
fn 使い方の誤りは対象の語を持つ() {
    let e = parse(&["start"]).unwrap_err();
    assert_eq!(e.subject.as_deref(), Some("start"));
    let e = parse(&["run", "x", "--bogus"]).unwrap_err();
    assert_eq!(e.subject.as_deref(), Some("--bogus"));
    let e = parse(&[]).unwrap_err();
    assert!(e.subject.is_none());
}

#[cfg(unix)]
#[test]
fn utf8_でないパスは_os_string_のまま保つ() {
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    let bad = OsString::from_vec(vec![b'a', 0xff, b'.', b't']);
    let args = vec![
        OsString::from("run"),
        OsString::from("x"),
        OsString::from("--config"),
        bad.clone(),
    ];
    let Ok(ParsedArgs::Command(cmd)) = parse_args(args) else {
        panic!("受け付けるはず")
    };
    assert_eq!(
        cmd.config_path.as_ref().map(|p| p.as_bytes()),
        Some(bad.as_bytes())
    );

    let args = vec![
        OsString::from("convert"),
        bad.clone(),
        OsString::from("--to"),
        OsString::from("yaml"),
    ];
    let Ok(ParsedArgs::Command(cmd)) = parse_args(args) else {
        panic!("受け付けるはず")
    };
    assert_eq!(cmd.convert_input, Some(bad.clone()));
}

#[cfg(unix)]
#[test]
fn utf8_でないジョブidとコマンド名は_invalid_value() {
    use std::os::unix::ffi::OsStringExt;
    let bad = OsString::from_vec(vec![0xff, 0xfe]);
    let e = parse_args(vec![OsString::from("run"), bad.clone()]).unwrap_err();
    assert_eq!(e.kind, UsageErrorKind::InvalidValue);
    let e = parse_args(vec![bad]).unwrap_err();
    assert_eq!(e.kind, UsageErrorKind::InvalidValue);
}

#[test]
fn コマンドの名前() {
    for (kind, name) in [
        (CommandKind::Run, "run"),
        (CommandKind::Pipeline, "pipeline"),
        (CommandKind::Watch, "watch"),
        (CommandKind::Reset, "reset"),
        (CommandKind::Validate, "validate"),
        (CommandKind::Convert, "convert"),
        (CommandKind::Graph, "graph"),
        (CommandKind::Init, "init"),
    ] {
        assert_eq!(kind.name(), name);
    }
}

#[test]
fn コマンドの名前から種類を引ける_往復_重なりなし_知らない名前は_none() {
    for kind in CommandKind::ALL {
        assert_eq!(CommandKind::from_name(kind.name()), Some(kind), "{kind:?}");
    }
    let mut names: Vec<&str> = CommandKind::ALL.iter().map(|k| k.name()).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), CommandKind::ALL.len(), "名前は重ならない");
    for unknown in ["", "Run", "runs"] {
        assert_eq!(CommandKind::from_name(unknown), None, "{unknown:?}");
    }
}

// ---- init(U4。BR1.1、BR1.5、BR2.2、BR3.1) ----

#[test]
fn init_の既定は_conductor_toml_と_my_pipeline_で_force_なし() {
    let cmd = command(&["init"]);
    assert_eq!(cmd.kind, CommandKind::Init);
    let options = cmd.init_options();
    assert_eq!(options, InitOptions::default());
    assert_eq!(
        options.target,
        InitTarget::File(OsString::from("conductor.toml"))
    );
    assert_eq!(options.pipeline_name, "my-pipeline");
    assert!(!options.force);
}

#[test]
fn init_は_config_と_force_と名前を受け取り_ハイフンは標準出力() {
    let options = command(&["init", "--config", "x.toml", "--force", "夜間バッチ"]).init_options();
    assert_eq!(options.target, InitTarget::File(OsString::from("x.toml")));
    assert!(options.force);
    assert_eq!(options.pipeline_name, "夜間バッチ");

    let options = command(&["--force", "init", "--config=x.toml"]).init_options();
    assert!(options.force);
    assert_eq!(options.target, InitTarget::File(OsString::from("x.toml")));

    let options = command(&["init", "--config", "-", "--force"]).init_options();
    assert_eq!(options.target, InitTarget::Stdout);
    // 標準出力のときの `--force` は、誤りにしない(流れが無視する)。
    assert!(options.force);
    assert_eq!(
        command(&["init", "--config=-"]).init_options().target,
        InitTarget::Stdout
    );
}

#[test]
fn init_の名前が空か空白だけなら_invalid_value_で_値を含めない() {
    for name in ["", " ", "  \t "] {
        let e = parse(&["init", name]).unwrap_err();
        assert_eq!(e.kind, UsageErrorKind::InvalidValue, "{name:?}");
        assert_eq!(e.subject.as_deref(), Some("pipeline name"), "{name:?}");
    }
}

#[test]
fn init_の位置引数が_2_つ以上なら_unexpected_argument() {
    assert_eq!(
        kind_of(&["init", "a", "b"]),
        UsageErrorKind::UnexpectedArgument
    );
}

#[test]
fn init_は_config_と_force_以外のオプションをすべて許さない() {
    for args in [
        &["init", "-v"][..],
        &["init", "--dry-run"],
        &["init", "--assume", "a=1"],
        &["init", "--format", "text"],
        &["init", "--state"],
        &["init", "--run-id", "r"],
        &["init", "--remove-on-complete"],
        &["init", "--all"],
        &["init", "--to", "yaml"],
    ] {
        assert_eq!(kind_of(args), UsageErrorKind::OptionNotAllowed, "{args:?}");
    }
}

#[test]
fn force_は_init_だけで使え_重複と値つきは誤り() {
    for kind in ["run", "pipeline", "watch", "reset", "validate", "graph"] {
        let mut args = vec![kind, "--force"];
        if matches!(kind, "run" | "pipeline") {
            args.push("x");
        }
        let e = parse(&args).unwrap_err();
        assert_eq!(e.kind, UsageErrorKind::OptionNotAllowed, "{args:?}");
        assert_eq!(e.subject.as_deref(), Some("--force"), "{args:?}");
    }
    let e = parse(&["convert", "f", "--to", "yaml", "--force"]).unwrap_err();
    assert_eq!(e.kind, UsageErrorKind::OptionNotAllowed);
    let e = parse(&["init", "--force", "--force"]).unwrap_err();
    assert_eq!(e.kind, UsageErrorKind::InvalidValue);
    assert_eq!(e.subject.as_deref(), Some("--force"));
    let e = parse(&["init", "--force=1"]).unwrap_err();
    assert_eq!(e.kind, UsageErrorKind::InvalidValue);
}

#[test]
fn init_の_config_の重複と値なしは誤り() {
    assert_eq!(
        kind_of(&["init", "--config", "a", "--config", "b"]),
        UsageErrorKind::InvalidValue
    );
    assert_eq!(kind_of(&["init", "--config"]), UsageErrorKind::MissingValue);
}

#[test]
fn 使い方の誤りの全種類が網羅されている() {
    assert_eq!(UsageErrorKind::ALL.len(), 8);
}

// ---- --run-id と --all(FR1.1、FR1.4、FR5.3) ----

#[test]
fn run_id_は_4_つのコマンドで解釈される() {
    for (args, kind) in [
        (&["run", "a", "--run-id", "r1"][..], CommandKind::Run),
        (&["pipeline", "p", "--run-id=r1"][..], CommandKind::Pipeline),
        (&["watch", "--run-id", "r1"][..], CommandKind::Watch),
        (&["reset", "--run-id", "r1"][..], CommandKind::Reset),
    ] {
        let cmd = command(args);
        assert_eq!(cmd.kind, kind, "{args:?}");
        assert_eq!(
            cmd.run_id.as_ref().map(RunIdArg::as_str),
            Some("r1"),
            "{args:?}"
        );
        assert!(!cmd.all);
    }
    // 指定なし、空(固定)の区別
    assert_eq!(command(&["run", "a"]).run_id, None);
    assert_eq!(
        command(&["run", "a", "--run-id", ""])
            .run_id
            .as_ref()
            .map(RunIdArg::as_str),
        Some("")
    );
    assert_eq!(
        command(&["run", "a", "--run-id="])
            .run_id
            .as_ref()
            .map(RunIdArg::as_str),
        Some("")
    );
}

#[test]
fn run_id_は_validate_と_convert_で使えない() {
    for args in [
        &["validate", "--run-id", "x"][..],
        &["validate", "p", "--run-id=x"][..],
        &["convert", "f.yaml", "--to", "toml", "--run-id", "x"][..],
    ] {
        let e = parse(args).unwrap_err();
        assert_eq!(e.kind, UsageErrorKind::OptionNotAllowed, "{args:?}");
        assert_eq!(e.subject.as_deref(), Some("--run-id"));
    }
}

#[test]
fn all_は_reset_だけが受け付ける() {
    let cmd = command(&["reset", "--all"]);
    assert!(cmd.all);
    assert_eq!(cmd.run_id, None);
    for args in [
        &["run", "a", "--all"][..],
        &["pipeline", "p", "--all"][..],
        &["watch", "--all"][..],
        &["validate", "--all"][..],
        &["convert", "f.yaml", "--to", "toml", "--all"][..],
    ] {
        assert_eq!(kind_of(args), UsageErrorKind::OptionNotAllowed, "{args:?}");
    }
}

#[test]
fn all_と_run_id_の併用と重複と値の欠落は誤り() {
    assert_eq!(
        kind_of(&["reset", "--all", "--run-id", "x"]),
        UsageErrorKind::InvalidValue
    );
    assert_eq!(
        kind_of(&["reset", "--run-id", "", "--all"]),
        UsageErrorKind::InvalidValue,
        "空の値でも併用はできない"
    );
    assert_eq!(
        kind_of(&["reset", "--all", "--all"]),
        UsageErrorKind::InvalidValue
    );
    assert_eq!(
        kind_of(&["run", "a", "--run-id", "x", "--run-id", "y"]),
        UsageErrorKind::InvalidValue
    );
    assert_eq!(
        kind_of(&["run", "a", "--run-id"]),
        UsageErrorKind::MissingValue
    );
    assert_eq!(
        kind_of(&["reset", "--all=x"]),
        UsageErrorKind::InvalidValue,
        "--all は値を取らない"
    );
}

#[test]
fn run_id_の無効な値は_68_で誤りの文に値を含めない() {
    let long = "a".repeat(65);
    for bad in [
        "a/b",
        "..",
        ".a",
        ".",
        "a b",
        "é",
        "a\\b",
        "a:b",
        long.as_str(),
    ] {
        let e = parse(&["run", "a", "--run-id", bad]).unwrap_err();
        assert_eq!(e.kind, UsageErrorKind::InvalidValue, "{bad}");
        assert_eq!(
            e.subject.as_deref(),
            Some("--run-id"),
            "値を subject にしない: {bad}"
        );
    }
    // 境界の有効な値
    for ok in ["a", "run-1.2_x", "A", &"a".repeat(64), "CON", "a..b"] {
        assert!(parse(&["run", "a", "--run-id", ok]).is_ok(), "{ok}");
    }
    // UTF-8 でない値
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        let args = vec![
            OsString::from("run"),
            OsString::from("a"),
            OsString::from("--run-id"),
            OsString::from_vec(vec![0x66, 0xff]),
        ];
        let e = parse_args(args).unwrap_err();
        assert_eq!(e.kind, UsageErrorKind::InvalidValue);
        assert_eq!(e.subject.as_deref(), Some("--run-id"));
    }
}

// ---- resolve_run_id(FR1.2〜FR1.4) ----

fn resolved(option: Option<&str>, env: &[(&str, &str)]) -> Result<Option<String>, UsageError> {
    let option = option.map(RunIdArg::parse).transpose()?;
    conductor_cli::run_id::resolve_run_id(option.as_ref(), "RID", &common::FakeEnv::with(env))
        .map(|r| r.map(|id| id.as_str().to_string()))
}

#[test]
fn resolve_run_id_のオプション_環境変数_空_固定() {
    // オプションが環境変数に優先する(環境変数が不正でも見ない)
    assert_eq!(resolved(Some("a"), &[("RID", "b")]), Ok(Some("a".into())));
    assert_eq!(
        resolved(Some("a"), &[("RID", "../bad")]),
        Ok(Some("a".into()))
    );
    // オプションがなければ環境変数
    assert_eq!(resolved(None, &[("RID", "b")]), Ok(Some("b".into())));
    // 未定義と空は run ID なし
    assert_eq!(resolved(None, &[]), Ok(None));
    assert_eq!(resolved(None, &[("RID", "")]), Ok(None));
    // 名前が違う環境変数は見ない
    assert_eq!(resolved(None, &[("OTHER", "x")]), Ok(None));
    // 空のオプションは、環境変数があっても、run ID なしに固定する
    assert_eq!(resolved(Some(""), &[("RID", "b")]), Ok(None));
}

#[test]
fn fr2_5_remove_on_complete_は_run_と_pipeline_だけが受け付ける() {
    assert!(command(&["run", "a", "--remove-on-complete"]).remove_on_complete);
    assert!(command(&["pipeline", "--remove-on-complete", "p"]).remove_on_complete);
    assert!(!command(&["pipeline", "p"]).remove_on_complete);
    for args in [
        &["watch", "--remove-on-complete"][..],
        &["reset", "--remove-on-complete"],
        &["validate", "--remove-on-complete"],
        &["convert", "x", "--to", "toml", "--remove-on-complete"],
    ] {
        assert_eq!(kind_of(args), UsageErrorKind::OptionNotAllowed, "{args:?}");
    }
}

#[test]
fn fr2_5_remove_on_complete_は値を取らず_2_回の指定は誤り() {
    assert_eq!(
        kind_of(&["run", "a", "--remove-on-complete", "--remove-on-complete"]),
        UsageErrorKind::InvalidValue
    );
    assert_eq!(
        kind_of(&["run", "a", "--remove-on-complete=1"]),
        UsageErrorKind::InvalidValue
    );
}

#[test]
fn resolve_run_id_の環境変数の値の違反は_invalid_value_で値を含めない() {
    let long = "z".repeat(65);
    for bad in ["a/b", "..", ".a", "a b", "é", long.as_str()] {
        let e = resolved(None, &[("RID", bad)]).unwrap_err();
        assert_eq!(e.kind, UsageErrorKind::InvalidValue, "{bad}");
        let subject = e.subject.expect("対象がある");
        assert!(!subject.contains(bad) || bad.len() < 2, "{subject}");
        assert!(subject.contains("environment variable"), "{subject}");
    }
    let e = resolved(Some("a/b"), &[]).unwrap_err();
    assert_eq!(e.subject.as_deref(), Some("--run-id"));
}

// ---- graph(U2。BR4.5、BR6.4) ----

#[test]
fn graph_は_全オプションを受け取り_既定は_text_で_状態なし() {
    let cmd = command(&["graph"]);
    assert_eq!(cmd.kind, CommandKind::Graph);
    assert_eq!(cmd.graph.format, GraphFormat::Text);
    assert!(!cmd.graph.with_state && !cmd.verbose && cmd.pipeline_name.is_none());
    let cmd = command(&[
        "graph", "-v", "--config", "c.toml", "--format", "mermaid", "--state", "--run-id", "r1",
        "nightly",
    ]);
    assert_eq!(cmd.graph.format, GraphFormat::Mermaid);
    assert!(cmd.graph.with_state && cmd.verbose);
    assert_eq!(cmd.run_id.as_ref().map(RunIdArg::as_str), Some("r1"));
    assert_eq!(cmd.pipeline_name.as_deref(), Some("nightly"));
    // `--format=text` の形も受け付ける。
    assert_eq!(
        command(&["graph", "--format=text"]).graph.format,
        GraphFormat::Text
    );
}

#[test]
fn graph_の_run_id_は_state_と一緒のときだけ() {
    for args in [
        &["graph", "--run-id", "r1"][..],
        &["graph", "--run-id", ""][..],
    ] {
        let e = parse(args).unwrap_err();
        assert_eq!(e.kind, UsageErrorKind::OptionNotAllowed, "{args:?}");
        assert_eq!(e.subject.as_deref(), Some("--run-id"));
    }
    // 形の誤りは、既存の誤り。
    assert_eq!(
        kind_of(&["graph", "--state", "--run-id", ".bad"]),
        UsageErrorKind::InvalidValue
    );
}

#[test]
fn graph_の_format_は_text_か_mermaid_だけで_重複と値の欠けは誤り() {
    assert_eq!(
        kind_of(&["graph", "--format", "dot"]),
        UsageErrorKind::InvalidValue
    );
    assert_eq!(
        kind_of(&["graph", "--format", "text", "--format", "mermaid"]),
        UsageErrorKind::InvalidValue
    );
    assert_eq!(
        kind_of(&["graph", "--format"]),
        UsageErrorKind::MissingValue
    );
    assert_eq!(
        kind_of(&["graph", "--state", "--state"]),
        UsageErrorKind::InvalidValue
    );
    assert_eq!(
        kind_of(&["graph", "--state=x"]),
        UsageErrorKind::InvalidValue
    );
    assert_eq!(kind_of(&["graph", "-vv"]), UsageErrorKind::InvalidValue);
    assert_eq!(
        kind_of(&["graph", "a", "b"]),
        UsageErrorKind::UnexpectedArgument
    );
}

#[test]
fn format_と_state_は_graph_以外では許されないオプション() {
    for kind in ["run", "pipeline", "watch", "reset", "validate", "convert"] {
        for option in [&["--format", "text"][..], &["--state"][..]] {
            let mut args = vec![kind];
            if matches!(kind, "run" | "pipeline" | "convert") {
                args.push("x");
            }
            args.extend_from_slice(option);
            assert_eq!(kind_of(&args), UsageErrorKind::OptionNotAllowed, "{args:?}");
        }
    }
}

// ---- `--dry-run` と `--assume`(U3。BR2.1、BR2.2、BR4.5) ----

#[test]
fn dry_run_と_assume_は_run_と_pipeline_で_複数回指定できる() {
    for kind in ["run", "pipeline"] {
        let cmd = command(&[
            kind,
            "x",
            "--dry-run",
            "--assume",
            "a=3",
            "--assume=b=255",
            "--run-id",
            "r1",
            "--remove-on-complete",
        ]);
        assert!(cmd.dry_run.dry_run, "{kind}");
        let pairs: Vec<(&str, i32)> = cmd
            .dry_run
            .assumptions
            .iter()
            .map(|a| (a.job_id.as_str(), a.exit_code))
            .collect();
        assert_eq!(pairs, [("a", 3), ("b", 255)], "{kind}");
        assert_eq!(cmd.run_id.as_ref().map(RunIdArg::as_str), Some("r1"));
        assert!(cmd.remove_on_complete);
    }
    // 指定がなければ、既定は偽で空。値の `=` は、最初の `=` で分ける(終了コードに `=` があれば誤り)。
    let cmd = command(&["pipeline", "x"]);
    assert!(!cmd.dry_run.dry_run && cmd.dry_run.assumptions.is_empty());
    assert_eq!(
        kind_of(&["pipeline", "x", "--dry-run", "--assume", "a=1=2"]),
        UsageErrorKind::InvalidValue
    );
}

#[test]
fn assume_の不正な入力は_invalid_value_で_入力の値を含まない() {
    for bad in [
        "a",
        "a=",
        "a=x",
        "a=1.5",
        "a=-1",
        "a=256",
        "a=99999999999",
        "a=1 ",
    ] {
        let e = parse(&["pipeline", "x", "--dry-run", "--assume", bad]).unwrap_err();
        assert_eq!(e.kind, UsageErrorKind::InvalidValue, "{bad:?}");
        assert_eq!(e.subject.as_deref(), Some("--assume"), "{bad:?}");
    }
    // 同じジョブ ID の重複。
    let e = parse(&[
        "run",
        "x",
        "--dry-run",
        "--assume",
        "job-secret-id=1",
        "--assume",
        "job-secret-id=2",
    ])
    .unwrap_err();
    assert_eq!(e.kind, UsageErrorKind::InvalidValue);
    assert_eq!(e.subject.as_deref(), Some("--assume"));
    // 値の欠け。
    assert_eq!(
        kind_of(&["run", "x", "--dry-run", "--assume"]),
        UsageErrorKind::MissingValue
    );
}

#[test]
fn assume_は_dry_run_なしでは使えず_dry_run_は_run_と_pipeline_だけ() {
    let e = parse(&["run", "x", "--assume", "a=1"]).unwrap_err();
    assert_eq!(e.kind, UsageErrorKind::OptionNotAllowed);
    assert_eq!(e.subject.as_deref(), Some("--assume"));
    for kind in ["watch", "reset", "validate", "graph", "convert"] {
        let mut args = vec![kind];
        if kind == "convert" {
            args.extend(["f", "--to", "yaml"]);
        }
        args.push("--dry-run");
        let e = parse(&args).unwrap_err();
        assert_eq!(e.kind, UsageErrorKind::OptionNotAllowed, "{args:?}");
        assert_eq!(e.subject.as_deref(), Some("--dry-run"), "{args:?}");
    }
    // 重複と値つきの指定は誤り。
    assert_eq!(
        kind_of(&["run", "x", "--dry-run", "--dry-run"]),
        UsageErrorKind::InvalidValue
    );
    assert_eq!(
        kind_of(&["run", "x", "--dry-run=1"]),
        UsageErrorKind::InvalidValue
    );
}

// ---- 誤りの報告の順(04 章 4.2 の 6・7。表化の前の動きを固定する) ----

fn error_of(args: &[&str]) -> (UsageErrorKind, Option<String>) {
    match parse(args) {
        Err(e) => (e.kind, e.subject),
        other => panic!("使い方の誤りになるはず: {args:?} -> {other:?}"),
    }
}

fn assert_error(args: &[&str], kind: UsageErrorKind, subject: &str) {
    assert_eq!(
        error_of(args),
        (kind, Some(subject.to_string())),
        "{args:?}"
    );
}

#[test]
fn 報告の順_使えないの検査は仕様の順に並ぶ() {
    use UsageErrorKind::OptionNotAllowed as NotAllowed;
    // -v、--config、--to、--run-id、--all、--remove-on-complete、--dry-run、--force の順。
    assert_error(
        &["convert", "f", "--to", "yaml", "--config", "c", "-v"],
        NotAllowed,
        "-v",
    );
    assert_error(
        &["validate", "--run-id", "a", "--to", "yaml"],
        NotAllowed,
        "--to",
    );
    assert_error(
        &["validate", "--all", "--run-id", "a"],
        NotAllowed,
        "--run-id",
    );
    assert_error(
        &["watch", "--remove-on-complete", "--all"],
        NotAllowed,
        "--all",
    );
    assert_error(
        &["watch", "--dry-run", "--remove-on-complete"],
        NotAllowed,
        "--remove-on-complete",
    );
    assert_error(&["watch", "--force", "--dry-run"], NotAllowed, "--dry-run");
    // --format は --state より先。
    assert_error(
        &["run", "x", "--state", "--format", "text"],
        NotAllowed,
        "--format",
    );
}

#[test]
fn 報告の順_force_の使えないは重複より先で_force_の重複は_assume_の不可より先() {
    assert_error(
        &["watch", "--force", "--force"],
        UsageErrorKind::OptionNotAllowed,
        "--force",
    );
    assert_error(
        &["init", "--assume", "a=1", "--force", "--force"],
        UsageErrorKind::InvalidValue,
        "--force",
    );
}

#[test]
fn 報告の順_assume_の不可と_dry_run_の重複は_format_と_state_の不可より先() {
    assert_error(
        &["run", "x", "--format", "text", "--assume", "a=1"],
        UsageErrorKind::OptionNotAllowed,
        "--assume",
    );
    assert_error(
        &["run", "x", "--state", "--dry-run", "--dry-run"],
        UsageErrorKind::InvalidValue,
        "--dry-run",
    );
    assert_error(
        &["run", "x", "-v", "-v", "--dry-run", "--dry-run"],
        UsageErrorKind::InvalidValue,
        "--dry-run",
    );
}

#[test]
fn 報告の順_使えないはすべての重複より先() {
    assert_error(
        &["watch", "-v", "-v"],
        UsageErrorKind::OptionNotAllowed,
        "-v",
    );
    assert_error(
        &["run", "x", "--config", "a", "--config", "b", "--state"],
        UsageErrorKind::OptionNotAllowed,
        "--state",
    );
    assert_error(
        &["graph", "--format", "a", "--format", "b", "-v", "-v"],
        UsageErrorKind::InvalidValue,
        "--format",
    );
}

#[test]
fn 報告の順_重複の検査は仕様の順に並ぶ() {
    use UsageErrorKind::InvalidValue as Invalid;
    // --format、--state、-v、--config、--to、--run-id、--all、--remove-on-complete の順。
    assert_error(
        &[
            "graph", "--state", "--state", "--format", "a", "--format", "b",
        ],
        Invalid,
        "--format",
    );
    assert_error(
        &["graph", "-v", "-v", "--state", "--state"],
        Invalid,
        "--state",
    );
    assert_error(
        &["run", "x", "--config", "a", "--config", "b", "-v", "-v"],
        Invalid,
        "-v",
    );
    assert_error(
        &[
            "run", "x", "--run-id", "a", "--run-id", "b", "--config", "a", "--config", "b",
        ],
        Invalid,
        "--config",
    );
    assert_error(
        &["reset", "--all", "--all", "--run-id", "a", "--run-id", "b"],
        Invalid,
        "--run-id",
    );
    assert_error(
        &[
            "run",
            "x",
            "--remove-on-complete",
            "--remove-on-complete",
            "--run-id",
            "a",
            "--run-id",
            "b",
        ],
        Invalid,
        "--run-id",
    );
    assert_error(
        &[
            "init", "--config", "a", "--config", "b", "--force", "--force",
        ],
        Invalid,
        "--force",
    );
}

#[test]
fn 報告の順_重複は_all_と_run_id_の併用と値の検査より先() {
    // 併用の誤りの対象は --all、重複の対象は --run-id。
    assert_error(
        &["reset", "--all", "--run-id", "a", "--run-id", "b"],
        UsageErrorKind::InvalidValue,
        "--run-id",
    );
    // --format の値は、重複のあとで検査する(値の誤りでも対象は同じだが、種類の順を確かめる)。
    assert_error(
        &["graph", "--format", "dot", "-v", "-v"],
        UsageErrorKind::InvalidValue,
        "-v",
    );
    // --assume の値の誤りは、--run-id の重複より後。
    assert_error(
        &[
            "run",
            "x",
            "--dry-run",
            "--assume",
            "bad",
            "--run-id",
            "a",
            "--run-id",
            "b",
        ],
        UsageErrorKind::InvalidValue,
        "--run-id",
    );
}

#[test]
fn 報告の順_字句の誤りは知らないオプションより先で_最初のものを報告する() {
    assert_error(
        &["run", "x", "--bogus", "--verbose=1"],
        UsageErrorKind::InvalidValue,
        "--verbose",
    );
    assert_error(
        &["run", "x", "--bogus", "-y"],
        UsageErrorKind::UnknownOption,
        "--bogus",
    );
    assert_error(
        &["run", "x", "--force=1", "--bogus=2"],
        UsageErrorKind::InvalidValue,
        "--force",
    );
    assert_error(&["--bogus"], UsageErrorKind::UnknownOption, "--bogus");
}

#[test]
fn 値の欠落の対象はオプションの綴り() {
    for option in ["--config", "--run-id", "--format", "--assume", "--to"] {
        assert_error(&["run", "x", option], UsageErrorKind::MissingValue, option);
    }
}

#[test]
fn 値を取らないオプションに値を付けると対象はその綴り() {
    for option in [
        "--verbose",
        "--all",
        "--remove-on-complete",
        "--state",
        "--dry-run",
        "--force",
    ] {
        let arg = format!("{option}=1");
        assert_error(&["run", "x", &arg], UsageErrorKind::InvalidValue, option);
    }
}

// ---- `--to` の値の誤りの対象(BR3.1) ----

#[test]
fn to_の値の誤りの対象は_to_で値を含まない() {
    for bad in ["json", "YAML", ""] {
        assert_error(
            &["convert", "f", "--to", bad],
            UsageErrorKind::InvalidValue,
            "--to",
        );
        assert_error(
            &["convert", &format!("--to={bad}"), "f"],
            UsageErrorKind::InvalidValue,
            "--to",
        );
    }
}

#[cfg(unix)]
#[test]
fn utf8_でない_to_の値は_invalid_value_で対象は_to() {
    use std::os::unix::ffi::OsStringExt;
    let args = vec![
        OsString::from("convert"),
        OsString::from("f"),
        OsString::from("--to"),
        OsString::from_vec(vec![b'y', 0xff]),
    ];
    let e = parse_args(args).unwrap_err();
    assert_eq!(e.kind, UsageErrorKind::InvalidValue);
    assert_eq!(e.subject.as_deref(), Some("--to"));
}
