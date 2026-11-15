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

//! NFR4.5(検証は副作用を持たず、コマンドを解釈しない)と
//! NFR4.6(エラーの表示に入力の値が出ない)、NFR4.7(パニックしない)のテスト。

use std::collections::HashSet;

use conductor_core::{DefinitionErrorKind, load_definition, parse_definition};

const SENTINEL: &str = "SENTINEL_VALUE_9f3a";

#[test]
fn nfr4_5_コマンドの文字列はそのまま保持され展開も起動もされない() {
    let command = "sh -c 'touch SHOULD_NOT_EXIST' ; `id` $(id) ${CONDUCTOR_TEST_UNSET} $HOME ~/x";
    let text = format!(
        "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = {command:?}\n"
    );
    let def = parse_definition(&text, "t.toml", None).unwrap();
    assert_eq!(def.jobs[0].command, command);
    assert!(!std::path::Path::new("SHOULD_NOT_EXIST").exists());
}

#[test]
fn nfr4_5_ファイルから読んでも定義のあるディレクトリは変わらない() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("conductor.toml");
    std::fs::write(
        &path,
        "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"touch created\"\n",
    )
    .unwrap();
    let before = std::fs::read(&path).unwrap();
    load_definition(Some(&path), None).unwrap();
    let entries: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(entries.len(), 1, "ファイルを作らない");
    assert_eq!(std::fs::read(&path).unwrap(), before, "書き換えない");
}

/// 各種類を起こす入力。値の位置に、目印の文字列を入れる。
fn cases() -> Vec<(DefinitionErrorKind, String)> {
    use DefinitionErrorKind as K;
    let head = "format_version = 1\n[pipeline]\nname = \"p\"\n";
    let job = |extra: &str| format!("{head}[[job]]\nid = \"a\"\ncommand = \"c\"\n{extra}");
    vec![
        (K::Syntax, format!("a = = \"{SENTINEL}\"\n")),
        (
            K::TypeMismatch,
            job(&format!("warning_exit_codes = [\"{SENTINEL}\"]\n")),
        ),
        (
            K::UnknownKey,
            format!(
                "format_version = 1\nextra = \"{SENTINEL}\"\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"c\"\n"
            ),
        ),
        (K::VersionMissing, format!("x = \"{SENTINEL}\"\n")),
        (
            K::VersionUnsupported,
            format!("format_version = \"{SENTINEL}\"\n"),
        ),
        (
            K::UnknownTarget,
            job(&format!(
                "[[job.transitions]]\non = \"success\"\nnext = \"{SENTINEL}\"\n"
            )),
        ),
        (
            K::ErrorTransition,
            job(&format!(
                "[[job.transitions]]\non = \"error\"\nnext = \"{SENTINEL}\"\n"
            )),
        ),
        (
            K::BackwardOrSelfTarget,
            format!(
                "{head}[[job]]\nid = \"{SENTINEL}\"\ncommand = \"c\"\n[[job.transitions]]\non = \"success\"\nnext = \"{SENTINEL}\"\n"
            ),
        ),
        (
            K::OverlappingExitCodes,
            job("warning_exit_codes = [7]\nsuccess_exit_codes = [7]\n"),
        ),
        (K::ZeroAsWarning, job("warning_exit_codes = [0]\n")),
        (
            K::DuplicateJobId,
            format!(
                "{head}[[job]]\nid = \"{SENTINEL}\"\ncommand = \"c\"\n[[job]]\nid = \"{SENTINEL}\"\ncommand = \"c\"\n"
            ),
        ),
        (
            K::InvalidJobId,
            format!("{head}[[job]]\nid = \"@{SENTINEL}\"\ncommand = \"c\"\n"),
        ),
        (K::NoJobs, head.to_string()),
        (
            K::EmptyField,
            format!("{head}[[job]]\nid = \"{SENTINEL}\"\ncommand = \"\"\n"),
        ),
        (
            K::DuplicateTransition,
            job(
                "[[job.transitions]]\non = \"success\"\nnext = \"@end\"\n[[job.transitions]]\non = \"success\"\nnext = \"@end\"\n",
            ),
        ),
    ]
}

#[test]
fn nfr4_6_すべての種類で表示の文に入力の値が出ない() {
    let mut seen: HashSet<DefinitionErrorKind> = HashSet::new();
    for (kind, text) in cases() {
        let errs = parse_definition(&text, "t.toml", None).unwrap_err();
        assert!(
            errs.iter().any(|e| e.kind == kind),
            "{kind:?} が返ること: {errs:?}"
        );
        for e in &errs {
            let shown = e.to_string();
            assert!(!shown.contains(SENTINEL), "{kind:?}: {shown}");
            seen.insert(e.kind);
        }
    }

    // パイプライン名の不一致: 両方の名前が表示に出ない。
    let text = format!(
        "format_version = 1\n[pipeline]\nname = \"{SENTINEL}_def\"\n[[job]]\nid = \"a\"\ncommand = \"c\"\n"
    );
    let errs = parse_definition(&text, "t.toml", Some(&format!("{SENTINEL}_arg"))).unwrap_err();
    assert_eq!(errs[0].kind, DefinitionErrorKind::PipelineNameMismatch);
    assert!(!errs[0].to_string().contains(SENTINEL));
    seen.insert(errs[0].kind);

    // ファイルの取り込みの種類: 内容の値は出ない。
    let dir = tempfile::tempdir().unwrap();
    let missing = load_definition(Some(&dir.path().join("none.toml")), None).unwrap_err();
    seen.insert(missing[0].kind);
    let big = dir.path().join("big.toml");
    std::fs::write(&big, format!("# {SENTINEL}\n{}", "#".repeat(1024 * 1024))).unwrap();
    let too_large = load_definition(Some(&big), None).unwrap_err();
    assert!(!too_large[0].to_string().contains(SENTINEL));
    seen.insert(too_large[0].kind);
    let bad = dir.path().join("bad.toml");
    let mut bytes = SENTINEL.as_bytes().to_vec();
    bytes.push(0xff);
    std::fs::write(&bad, bytes).unwrap();
    let unreadable = load_definition(Some(&bad), None).unwrap_err();
    assert!(!unreadable[0].to_string().contains(SENTINEL));
    seen.insert(unreadable[0].kind);

    for kind in DefinitionErrorKind::ALL {
        assert!(seen.contains(&kind), "{kind:?} のテストがない");
    }
}

#[test]
fn nfr4_6_キー名は制御文字をエスケープして表示される() {
    let text = "format_version = 1\n\"bad\\u001b[31mkey\" = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"c\"\n";
    let errs = parse_definition(text, "t.toml", None).unwrap_err();
    let shown = errs[0].to_string();
    assert!(!shown.contains('\u{1b}'));
    assert!(shown.contains("\\u{1b}"));
}

#[test]
fn nfr4_7_壊れた入力でもパニックしない() {
    let inputs: Vec<String> = vec![
        String::new(),
        "\u{feff}format_version = 1".into(),
        "[".into(),
        "[[".into(),
        "a = \"unterminated".into(),
        "a = 0x".into(),
        "a = 1979-05-27T07:32:00Z\n".into(),
        "a = { b = [ { c = 1 } ] }".into(),
        "\0\0\0".into(),
        "format_version = 1\njob = 5\npipeline = []\n".into(),
        "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"c\"\ntransitions = [1, 2]\nwarning_exit_codes = [[1]]\n".into(),
        format!("a = {}{}", "[".repeat(1000), "]".repeat(1000)),
        format!("a = {}1{}", "{b = ".repeat(1000), "}".repeat(1000)),
        format!("[{}]\n", vec!["k"; 1000].join(".")),
        "日本語のキー = 1\n".into(),
    ];
    for text in inputs {
        let r = parse_definition(&text, "t.toml", None);
        assert!(r.is_err(), "どれも定義としては受理されない");
    }
}
