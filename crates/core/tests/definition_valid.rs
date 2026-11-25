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

//! 正しい定義の受理と、`load_definition`(ファイルから読む入口)の統合テスト。

use std::path::{Path, PathBuf};

use conductor_core::{
    Backoff, Command, DefinitionErrorKind, JobDefinition, PipelineDefinition, RetryPolicy,
    Transition, TransitionOn, TransitionTarget, load_definition, parse_definition,
};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn 正しい定義をファイルから読める() {
    let def = load_definition(Some(&fixture("valid_pipeline.toml")), None).expect("受理される");

    assert_eq!(def.name, "nightly");
    assert_eq!(
        def.description.as_deref(),
        Some("夜間バッチ(抽出、変換、読み込み、通知)")
    );
    assert_eq!(def.format_version, 1);
    let ids: Vec<_> = def.job.iter().map(|j| j.id.as_str()).collect();
    assert_eq!(
        ids,
        ["extract", "transform", "load", "notify"],
        "記述順を保つ"
    );

    let extract: &JobDefinition = &def.job[0];
    assert_eq!(
        extract.command,
        Command::Line("extract --all --since yesterday".into())
    );
    assert!(extract.warning_exit_codes.is_empty());
    assert_eq!(extract.success_exit_codes, None);
    assert!(extract.transitions.is_empty());

    let transform = &def.job[1];
    assert_eq!(transform.warning_exit_codes, vec![3, 4]);
    assert_eq!(transform.success_exit_codes, Some(vec![0, 2]));
    assert_eq!(
        transform.transitions,
        vec![Transition {
            on: TransitionOn::Warning,
            next: TransitionTarget::Job("notify".into())
        }]
    );

    assert_eq!(
        def.job[2].transitions,
        vec![Transition {
            on: TransitionOn::Success,
            next: TransitionTarget::End
        }]
    );
}

#[test]
fn コマンドの文字列はそのまま保持される() {
    let def = load_definition(Some(&fixture("valid_pipeline.toml")), None).unwrap();
    // 引用符も `$HOME` も、展開や解釈をしない(NFR4.5)。
    assert_eq!(
        def.job[1].command,
        Command::Line("transform \"in dir\" $HOME".into())
    );
}

#[test]
fn パイプライン名が一致すれば受理される() {
    assert!(load_definition(Some(&fixture("valid_pipeline.toml")), Some("nightly")).is_ok());
}

#[test]
fn パイプライン名が違うと_pipeline_name_mismatch_で値は出ない() {
    let errs =
        load_definition(Some(&fixture("valid_pipeline.toml")), Some("other_secret")).unwrap_err();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].kind, DefinitionErrorKind::PipelineNameMismatch);
    assert_eq!(errs[0].line, Some(4));
    let shown = errs[0].to_string();
    assert!(!shown.contains("other_secret") && !shown.contains("nightly"));
}

#[test]
fn 存在しないファイルは_file_missing_で名前が出る() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("none.toml");
    let errs = load_definition(Some(&p), None).unwrap_err();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].kind, DefinitionErrorKind::FileMissing);
    assert!(errs[0].file.ends_with("none.toml"));
}

#[test]
fn 一時ディレクトリの定義を読める() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("my.toml");
    std::fs::write(
        &p,
        "format_version = 1\r\n[pipeline]\r\nname = \"x\"\r\n[[job]]\r\nid = \"a\"\r\ncommand = \"c\"\r\n",
    )
    .unwrap();
    let def = load_definition(Some(&p), None).unwrap();
    assert_eq!(def.job.len(), 1);
}

#[test]
fn 最小の定義は説明なしで受理される() {
    let def = parse_definition(
        "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"c\"\n",
        "m.toml",
        None,
    )
    .unwrap();
    assert_eq!(def.description, None);
}

#[test]
fn 定義の外の遷移も_next_が_end_なら受理される() {
    let text = "format_version = 1\n[pipeline]\nname = \"p\"\n\
        [[job]]\nid = \"a\"\ncommand = \"c\"\ntransitions = [{ on = \"success\", next = \"b\" }, { on = \"warning\", next = \"@end\" }]\n\
        [[job]]\nid = \"b\"\ncommand = \"c\"\n";
    let def = parse_definition(text, "m.toml", None).unwrap();
    assert_eq!(def.job[0].transitions.len(), 2);
}

#[test]
fn br4_1_既定を書かない定義は今と同じ定義に組み立てられる() {
    // `[pipeline.retry]` を書かない、v0.9.9 で受理される定義。`[job.retry]` を書いたジョブだけがリトライを持つ。
    let text = "format_version = 1\n[pipeline]\nname = \"p\"\n\
        [[job]]\nid = \"a\"\ncommand = \"c\"\nwarning_exit_codes = [4]\n\
        [job.retry]\nexit_codes = [75]\nmax = 3\ninterval = \"30s\"\nmultiplier = 2\nmax_interval = \"10m\"\n\
        [[job]]\nid = \"b\"\ncommand = \"c\"\n";
    let def = parse_definition(text, "t.toml", None).expect("受理される");
    let expected = PipelineDefinition {
        name: "p".into(),
        description: None,
        run_id_env: None,
        remove_on_complete: false,
        format_version: 1,
        job: vec![
            JobDefinition {
                id: "a".into(),
                command: Command::Line("c".into()),
                warning_exit_codes: vec![4],
                success_exit_codes: None,
                transitions: Vec::new(),
                retry: Some(RetryPolicy {
                    exit_codes: vec![75],
                    max: 3,
                    interval_secs: 30,
                    backoff: Some(Backoff {
                        multiplier: 2,
                        max_interval_secs: 600,
                    }),
                }),
                env: Vec::new(),
                env_file: Vec::new(),
                unset_env: Vec::new(),
            },
            JobDefinition {
                id: "b".into(),
                command: Command::Line("c".into()),
                warning_exit_codes: Vec::new(),
                success_exit_codes: None,
                transitions: Vec::new(),
                retry: None,
                env: Vec::new(),
                env_file: Vec::new(),
                unset_env: Vec::new(),
            },
        ],
        env: Vec::new(),
        env_file: Vec::new(),
        log_dir: None,
        unset_env: Vec::new(),
    };
    assert_eq!(def, expected);
}

#[test]
fn fr1_2_配列の_command_は要素のまま保持される() {
    let text = "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = [\"my prog\", \"a b\", \"$HOME\", \"\"]\n[[job]]\nid = \"b\"\ncommand = \"echo x\"\n";
    let def = parse_definition(text, "t.toml", None).expect("受理される");
    assert_eq!(
        def.job[0].command,
        Command::Argv(vec![
            "my prog".into(),
            "a b".into(),
            "$HOME".into(),
            String::new()
        ])
    );
    assert_eq!(def.job[1].command, Command::Line("echo x".into()));
}

/// `[pipeline]` に `log_dir` を足した定義。`value` は TOML の右辺をそのまま書く。
fn with_log_dir(value: &str) -> String {
    format!(
        "format_version = 1\n[pipeline]\nname = \"p\"\nlog_dir = {value}\n[[job]]\nid = \"a\"\ncommand = \"c\"\n"
    )
}

#[test]
fn br1_1_log_dir_は書いた文字列のまま読める() {
    // 相対・絶対・空白や日本語を含むパスも、解決せずそのまま持つ(解決は使う側)。
    for (value, expected) in [
        ("\"logs\"", "logs"),
        ("\"logs/nightly\"", "logs/nightly"),
        ("\"/var/log/conductor\"", "/var/log/conductor"),
        ("\"../ログ dir\"", "../ログ dir"),
        ("\" \"", " "),
        ("'C:\\logs'", "C:\\logs"),
    ] {
        let def = parse_definition(&with_log_dir(value), "t.toml", None).expect(value);
        assert_eq!(def.log_dir.as_deref(), Some(expected), "{value}");
    }
}

#[test]
fn br1_2_log_dir_を書かなければ_none() {
    let text =
        "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"c\"\n";
    let def = parse_definition(text, "t.toml", None).unwrap();
    assert_eq!(def.log_dir, None);
}

#[test]
fn br1_1_log_dir_はほかの_pipeline_のキーと共存する() {
    let text = "format_version = 1\n[pipeline]\nname = \"p\"\nlog_dir = \"logs\"\nremove_on_complete = true\nrun_id_env = \"X_ID\"\n\
        [pipeline.retry]\nexit_codes = [75]\nmax = 1\ninterval = \"1s\"\n[[job]]\nid = \"a\"\ncommand = \"c\"\n";
    let def = parse_definition(text, "t.toml", None).unwrap();
    assert_eq!(def.log_dir.as_deref(), Some("logs"));
    assert!(def.remove_on_complete);
    assert!(def.job[0].retry.is_some());
}
