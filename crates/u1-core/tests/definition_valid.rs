//! 正しい定義の受理と、`load_definition`(ファイルから読む入口)の統合テスト。

use std::path::{Path, PathBuf};

use conductor_core::{
    DefinitionErrorKind, JobDefinition, Transition, TransitionOn, TransitionTarget,
    load_definition, parse_definition,
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
    let ids: Vec<_> = def.jobs.iter().map(|j| j.id.as_str()).collect();
    assert_eq!(
        ids,
        ["extract", "transform", "load", "notify"],
        "記述順を保つ"
    );

    let extract: &JobDefinition = &def.jobs[0];
    assert_eq!(extract.command, "extract --all --since yesterday");
    assert!(extract.warning_exit_codes.is_empty());
    assert_eq!(extract.success_exit_codes, None);
    assert_eq!(extract.effective_success_codes(), &[0], "既定の成功コード");
    assert!(extract.transitions.is_empty());

    let transform = &def.jobs[1];
    assert_eq!(transform.warning_exit_codes, vec![3, 4]);
    assert_eq!(transform.effective_success_codes(), &[0, 2]);
    assert_eq!(
        transform.transitions,
        vec![Transition {
            on: TransitionOn::Warning,
            next: TransitionTarget::Job("notify".into())
        }]
    );

    assert_eq!(
        def.jobs[2].transitions,
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
    assert_eq!(def.jobs[1].command, "transform \"in dir\" $HOME");
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
    assert_eq!(def.jobs.len(), 1);
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
    assert_eq!(def.jobs[0].transitions.len(), 2);
}
