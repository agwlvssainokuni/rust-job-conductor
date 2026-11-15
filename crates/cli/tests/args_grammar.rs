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

use std::ffi::OsString;

use conductor_cli::{CliCommand, CommandKind, ParsedArgs, UsageError, UsageErrorKind, parse_args};
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
    ] {
        assert_eq!(kind.name(), name);
    }
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
        assert_eq!(cmd.run_id.as_deref(), Some("r1"), "{args:?}");
        assert!(!cmd.all);
    }
    // 指定なし、空(固定)の区別
    assert_eq!(command(&["run", "a"]).run_id, None);
    assert_eq!(
        command(&["run", "a", "--run-id", ""]).run_id.as_deref(),
        Some("")
    );
    assert_eq!(
        command(&["run", "a", "--run-id="]).run_id.as_deref(),
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

struct MapEnv(std::collections::HashMap<String, String>);

impl conductor_cli::Env for MapEnv {
    fn var(&self, name: &str) -> Option<String> {
        self.0.get(name).cloned()
    }
    fn stdout_is_terminal(&self) -> bool {
        false
    }
}

fn env_of(pairs: &[(&str, &str)]) -> MapEnv {
    MapEnv(
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    )
}

fn resolved(option: Option<&str>, env: &[(&str, &str)]) -> Result<Option<String>, UsageError> {
    conductor_cli::args::resolve_run_id(option, "RID", &env_of(env))
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
