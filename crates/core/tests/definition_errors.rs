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

//! 定義の検証の誤り(BR1.1〜BR1.13)の網羅テスト。違反の例と、受理の例。

use std::path::{Path, PathBuf};

use conductor_core::DefinitionErrorKind as K;
use conductor_core::{load_definition, parse_definition};

/// 検査の結果を `(種類, 行)` の並びで返す。受理なら空。
fn run(text: &str) -> Vec<(K, Option<usize>)> {
    match parse_definition(text, "t.toml", None) {
        Ok(_) => Vec::new(),
        Err(errs) => errs.into_iter().map(|e| (e.kind, e.line)).collect(),
    }
}

/// 先頭に版とパイプラインを付けた定義(3 行)。ジョブは 4 行目から。
fn with_header(jobs: &str) -> String {
    format!("format_version = 1\n[pipeline]\nname = \"p\"\n{jobs}")
}

fn kinds(text: &str) -> Vec<K> {
    run(text).into_iter().map(|(k, _)| k).collect()
}

#[test]
fn br1_1_存在しない遷移先() {
    let t = with_header(
        "[[job]]\nid = \"a\"\ncommand = \"c\"\n[[job.transitions]]\non = \"success\"\nnext = \"zzz\"\n",
    );
    assert_eq!(run(&t), vec![(K::UnknownTarget, Some(9))]);
    // 受理: 後ろのジョブへ
    let ok = with_header(
        "[[job]]\nid = \"a\"\ncommand = \"c\"\n[[job.transitions]]\non = \"success\"\nnext = \"b\"\n[[job]]\nid = \"b\"\ncommand = \"c\"\n",
    );
    assert!(run(&ok).is_empty());
}

#[test]
fn br1_2_エラー結果への遷移() {
    let t = with_header(
        "[[job]]\nid = \"a\"\ncommand = \"c\"\n[[job.transitions]]\non = \"error\"\nnext = \"@end\"\n",
    );
    assert_eq!(run(&t), vec![(K::ErrorTransition, Some(8))]);
}

#[test]
fn br1_3_自分自身と前のジョブへの遷移() {
    let t = with_header(
        "[[job]]\nid = \"a\"\ncommand = \"c\"\n[[job.transitions]]\non = \"success\"\nnext = \"a\"\n\
         [[job]]\nid = \"b\"\ncommand = \"c\"\n[[job.transitions]]\non = \"warning\"\nnext = \"a\"\n",
    );
    assert_eq!(
        run(&t),
        vec![
            (K::BackwardOrSelfTarget, Some(9)),
            (K::BackwardOrSelfTarget, Some(15))
        ]
    );
}

#[test]
fn br1_4_終了コードの重複と_0_の警告() {
    let overlap = with_header(
        "[[job]]\nid = \"a\"\ncommand = \"c\"\nwarning_exit_codes = [2, 3]\nsuccess_exit_codes = [0, 3]\n",
    );
    assert_eq!(run(&overlap), vec![(K::OverlappingExitCodes, Some(7))]);

    let zero = with_header("[[job]]\nid = \"a\"\ncommand = \"c\"\nwarning_exit_codes = [0]\n");
    assert_eq!(run(&zero), vec![(K::ZeroAsWarning, Some(7))]);

    // 0 を明示的に成功に含め、さらに警告に 0 を書いたときは、0 の指定の誤りだけを報告する
    let both = with_header(
        "[[job]]\nid = \"a\"\ncommand = \"c\"\nwarning_exit_codes = [0]\nsuccess_exit_codes = [0]\n",
    );
    assert_eq!(kinds(&both), vec![K::ZeroAsWarning]);

    // 受理: 重ならない
    let ok = with_header(
        "[[job]]\nid = \"a\"\ncommand = \"c\"\nwarning_exit_codes = [2]\nsuccess_exit_codes = [0, 3]\n",
    );
    assert!(run(&ok).is_empty());
}

#[test]
fn br1_5_型の誤り() {
    let cases = [
        (
            "終了コードが文字列",
            "[[job]]\nid = \"a\"\ncommand = \"c\"\nwarning_exit_codes = [\"x\"]\n",
            7,
        ),
        (
            "終了コードがリストでない",
            "[[job]]\nid = \"a\"\ncommand = \"c\"\nsuccess_exit_codes = 0\n",
            7,
        ),
        ("idが整数", "[[job]]\nid = 1\ncommand = \"c\"\n", 5),
        ("commandが無い", "[[job]]\nid = \"a\"\n", 4),
        (
            "終了コードが範囲外",
            "[[job]]\nid = \"a\"\ncommand = \"c\"\nwarning_exit_codes = [4294967296]\n",
            7,
        ),
        (
            "transitionsが配列でない",
            "[[job]]\nid = \"a\"\ncommand = \"c\"\ntransitions = 3\n",
            7,
        ),
        (
            "onが未知の値",
            "[[job]]\nid = \"a\"\ncommand = \"c\"\n[[job.transitions]]\non = \"later\"\nnext = \"@end\"\n",
            8,
        ),
        (
            "nextが無い",
            "[[job]]\nid = \"a\"\ncommand = \"c\"\n[[job.transitions]]\non = \"success\"\n",
            7,
        ),
    ];
    for (name, jobs, line) in cases {
        assert_eq!(
            run(&with_header(jobs)),
            vec![(K::TypeMismatch, Some(line))],
            "{name}"
        );
    }
}

#[test]
fn br1_5_パイプラインの型の誤りと欠落() {
    assert_eq!(
        run("format_version = 1\n[[job]]\nid = \"a\"\ncommand = \"c\"\n"),
        vec![(K::TypeMismatch, Some(1))]
    );
    assert_eq!(
        run("format_version = 1\n[pipeline]\nname = 5\n[[job]]\nid = \"a\"\ncommand = \"c\"\n"),
        vec![(K::TypeMismatch, Some(3))]
    );
    assert_eq!(
        run(
            "format_version = 1\n[pipeline]\ndescription = \"d\"\n[[job]]\nid = \"a\"\ncommand = \"c\"\n"
        ),
        vec![(K::TypeMismatch, Some(2))]
    );
    assert_eq!(
        run("format_version = 1\npipeline = 1\n[[job]]\nid = \"a\"\ncommand = \"c\"\n"),
        vec![(K::TypeMismatch, Some(2))]
    );
}

#[test]
fn br1_6_知らないキー() {
    let top = "format_version = 1\ntypo = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"c\"\n";
    assert_eq!(run(top), vec![(K::UnknownKey, Some(2))]);
    let pipe = "format_version = 1\n[pipeline]\nname = \"p\"\nowner = \"x\"\n[[job]]\nid = \"a\"\ncommand = \"c\"\n";
    assert_eq!(run(pipe), vec![(K::UnknownKey, Some(4))]);
    let job = with_header("[[job]]\nid = \"a\"\ncommand = \"c\"\nwarn_exit_codes = [1]\n");
    assert_eq!(run(&job), vec![(K::UnknownKey, Some(7))]);
    let tr = with_header(
        "[[job]]\nid = \"a\"\ncommand = \"c\"\n[[job.transitions]]\non = \"success\"\nnext = \"@end\"\nwhen = 1\n",
    );
    assert_eq!(run(&tr), vec![(K::UnknownKey, Some(10))]);
}

#[test]
fn br1_7_版の確認が先で他の誤りは出ない() {
    let missing = "unknown = 1\n[pipeline]\nname = 1\n";
    assert_eq!(kinds(missing), vec![K::VersionMissing]);
    let unsupported = "format_version = 2\nunknown = 1\n[pipeline]\nname = 1\n";
    assert_eq!(run(unsupported), vec![(K::VersionUnsupported, Some(1))]);
    // 文法の誤りは版より先
    assert_eq!(kinds("format_version = = 1"), vec![K::Syntax]);
}

#[test]
fn br1_8_パイプライン名の不一致() {
    let t = with_header("[[job]]\nid = \"a\"\ncommand = \"c\"\n");
    let errs = parse_definition(&t, "t.toml", Some("other")).unwrap_err();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].kind, K::PipelineNameMismatch);
    assert_eq!(errs[0].line, Some(3));
    assert!(parse_definition(&t, "t.toml", Some("p")).is_ok());
}

#[test]
fn br1_9_取り込みの誤りはファイルを使って確かめる() {
    let dir = tempfile::tempdir().unwrap();
    let missing = load_definition(Some(&dir.path().join("x.toml")), None).unwrap_err();
    assert_eq!(missing[0].kind, K::FileMissing);

    let big = dir.path().join("big.toml");
    std::fs::write(&big, "#".repeat(1024 * 1024 + 1)).unwrap();
    assert_eq!(
        load_definition(Some(&big), None).unwrap_err()[0].kind,
        K::TooLarge
    );

    let exact = dir.path().join("exact.toml");
    let mut body = String::from(
        "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"c\"\n",
    );
    body.push_str(&"#".repeat(1024 * 1024 - body.len()));
    assert_eq!(body.len(), 1024 * 1024);
    std::fs::write(&exact, &body).unwrap();
    assert!(
        load_definition(Some(&exact), None).is_ok(),
        "ちょうど 1 MiB は受理"
    );

    let bin = dir.path().join("bin.toml");
    std::fs::write(&bin, [0xff, 0xfe, 0x00]).unwrap();
    assert_eq!(
        load_definition(Some(&bin), None).unwrap_err()[0].kind,
        K::Unreadable
    );

    assert_eq!(
        load_definition(Some(dir.path()), None).unwrap_err()[0].kind,
        K::Unreadable
    );
}

#[test]
fn br1_12_パイプラインが_2_つ以上() {
    let t = "format_version = 1\n[[pipeline]]\nname = \"a\"\n[[pipeline]]\nname = \"b\"\n[[job]]\nid = \"a\"\ncommand = \"c\"\n";
    assert_eq!(run(t), vec![(K::TypeMismatch, Some(2))]);
}

#[test]
fn br1_13_構造の制約() {
    let dup =
        with_header("[[job]]\nid = \"a\"\ncommand = \"c\"\n[[job]]\nid = \"a\"\ncommand = \"c\"\n");
    assert_eq!(run(&dup), vec![(K::DuplicateJobId, Some(8))]);

    let empty_id = with_header("[[job]]\nid = \"\"\ncommand = \"c\"\n");
    assert_eq!(run(&empty_id), vec![(K::InvalidJobId, Some(5))]);
    let at_id = with_header("[[job]]\nid = \"@end\"\ncommand = \"c\"\n");
    assert_eq!(run(&at_id), vec![(K::InvalidJobId, Some(5))]);

    assert_eq!(
        kinds("format_version = 1\n[pipeline]\nname = \"p\"\n"),
        vec![K::NoJobs]
    );
    assert_eq!(
        kinds("format_version = 1\njob = []\n[pipeline]\nname = \"p\"\n"),
        vec![K::NoJobs]
    );

    let empty_cmd = with_header("[[job]]\nid = \"a\"\ncommand = \"  \"\n");
    assert_eq!(run(&empty_cmd), vec![(K::EmptyField, Some(6))]);
    assert_eq!(
        run("format_version = 1\n[pipeline]\nname = \"\"\n[[job]]\nid = \"a\"\ncommand = \"c\"\n"),
        vec![(K::EmptyField, Some(3))]
    );

    let dup_tr = with_header(
        "[[job]]\nid = \"a\"\ncommand = \"c\"\n[[job.transitions]]\non = \"success\"\nnext = \"@end\"\n[[job.transitions]]\non = \"success\"\nnext = \"@end\"\n",
    );
    assert_eq!(run(&dup_tr), vec![(K::DuplicateTransition, Some(10))]);
    // 受理: success と warning は別
    let ok = with_header(
        "[[job]]\nid = \"a\"\ncommand = \"c\"\n[[job.transitions]]\non = \"success\"\nnext = \"@end\"\n[[job.transitions]]\non = \"warning\"\nnext = \"@end\"\n",
    );
    assert!(run(&ok).is_empty());
}

#[test]
fn 複数の誤りが一度にすべて返る() {
    let path: PathBuf =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/multiple_errors.toml");
    let errs = load_definition(Some(&path), None).unwrap_err();
    let got: Vec<_> = errs.iter().map(|e| (e.kind, e.line)).collect();
    assert_eq!(
        got,
        vec![
            (K::UnknownKey, Some(2)),
            (K::TypeMismatch, Some(10)),
            (K::UnknownKey, Some(15)),
            (K::UnknownTarget, Some(19)),
        ]
    );
    // 表示の文に、誤った値(zero、nowhere)は入らない。キー名は入る。
    let shown: String = errs
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!shown.contains("zero") && !shown.contains("nowhere"));
    assert!(shown.contains("unknown_top") && shown.contains("colour"));
}

#[test]
fn 行番号は_crlf_でも一致する() {
    let t = with_header("[[job]]\nid = \"a\"\ncommand = \"c\"\nwarn = 1\n").replace('\n', "\r\n");
    assert_eq!(run(&t), vec![(K::UnknownKey, Some(7))]);
}

#[test]
fn 文法の誤りは_syntax_で行つき() {
    assert_eq!(
        run("format_version = 1\n[pipeline\n"),
        vec![(K::Syntax, Some(2))]
    );
}

#[test]
fn 深さの上限と巨大な入れ子() {
    let ok = format!("a = {}{}\n", "[".repeat(32), "]".repeat(32));
    // 深さ 32 は、構造の検査で unknownKey になる(深さの拒否ではない)ため、版の確認まで進む
    assert_eq!(kinds(&ok), vec![K::VersionMissing]);
    let ng = format!("a = {}{}\n", "[".repeat(33), "]".repeat(33));
    assert_eq!(kinds(&ng), vec![K::Syntax]);
    for depth in [1000, 100_000] {
        let arr = format!("a = {}{}\n", "[".repeat(depth), "]".repeat(depth));
        assert_eq!(kinds(&arr), vec![K::Syntax], "配列 {depth}");
        let inline = format!("a = {}1{}\n", "{b = ".repeat(depth), "}".repeat(depth));
        assert_eq!(
            kinds(&inline),
            vec![K::Syntax],
            "インラインテーブル {depth}"
        );
        let header = format!("[{}]\nx = 1\n", vec!["a"; depth].join("."));
        assert_eq!(kinds(&header), vec![K::Syntax], "見出し {depth}");
    }
}

#[test]
fn 深さ_32_は受け入れ_33_と_1000_は_syntax() {
    for (label, build) in [
        (
            "配列",
            (|d: usize| format!("a = {}{}\n", "[".repeat(d), "]".repeat(d))) as fn(usize) -> String,
        ),
        ("インラインテーブル", |d: usize| {
            format!("a = {}1{}\n", "{b = ".repeat(d), "}".repeat(d))
        }),
    ] {
        // 深さの検査は通り、構造の検査で別の誤り(版の欠落)になる。syntax ではない。
        assert_eq!(kinds(&build(32)), vec![K::VersionMissing], "{label} 32");
        for depth in [33, 1000] {
            assert_eq!(kinds(&build(depth)), vec![K::Syntax], "{label} {depth}");
        }
    }
}

#[test]
fn キー名は_64_文字で切られて表示される() {
    let long = "k".repeat(300);
    let t = format!(
        "format_version = 1\n{long} = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"c\"\n"
    );
    let errs = parse_definition(&t, "t.toml", None).unwrap_err();
    assert_eq!(errs[0].kind, K::UnknownKey);
    assert!(errs[0].reason.contains(&"k".repeat(64)));
    assert!(!errs[0].reason.contains(&"k".repeat(65)));
}

// ---- command の配列(FR1.3、FR1.4、FR1.5、NFR2) ----

/// `command` の値だけを差し替えた、1 ジョブの定義。`command` は 6 行目。
fn with_command(value: &str) -> String {
    with_header(&format!("[[job]]\nid = \"a\"\ncommand = {value}\n"))
}

#[test]
fn fr1_3_command_の型の誤り() {
    for value in ["5", "true", "{ a = 1 }", "[\"a\", 5]", "[[\"a\"]]"] {
        assert_eq!(
            run(&with_command(value)),
            vec![(K::TypeMismatch, Some(6))],
            "{value}"
        );
    }
}

#[test]
fn fr1_4_空の配列と先頭の要素の誤り() {
    for value in ["[]", "[\"\"]", "[\"  \", \"x\"]"] {
        assert_eq!(
            run(&with_command(value)),
            vec![(K::EmptyField, Some(6))],
            "{value}"
        );
    }
    // 受理: 2 番目以降の空の文字列は有効
    assert!(run(&with_command("[\"echo\", \"\"]")).is_empty());
}

#[test]
fn fr1_5_文字列の_command_の空は従来どおり誤り() {
    for value in ["\"\"", "\"   \""] {
        assert_eq!(
            run(&with_command(value)),
            vec![(K::EmptyField, Some(6))],
            "{value}"
        );
    }
}

#[test]
fn nfr2_command_の誤りのメッセージに入力の値が出ない() {
    let secret = "SENTINEL_CMD_7b1e";
    for value in [
        format!("[\"{secret}\", 5]"),
        format!("{{ {secret} = 1 }}"),
        format!("[\"  \", \"{secret}\"]"),
    ] {
        let errs = parse_definition(&with_command(&value), "t.toml", None).unwrap_err();
        assert!(!errs.is_empty());
        for e in errs {
            assert!(!e.to_string().contains(secret), "{value}: {e}");
        }
    }
}

/// `[pipeline]` に `run_id_env` を足した定義。`value` は TOML の右辺をそのまま書く。
fn with_run_id_env(value: &str) -> String {
    format!(
        "format_version = 1\n[pipeline]\nname = \"p\"\nrun_id_env = {value}\n[[job]]\nid = \"a\"\ncommand = \"c\"\n"
    )
}

#[test]
fn fr2_run_id_env_は正しい名前なら受理される() {
    let def = parse_definition(&with_run_id_env("\"GITHUB_RUN_ID\""), "t.toml", None).unwrap();
    assert_eq!(def.run_id_env.as_deref(), Some("GITHUB_RUN_ID"));
    assert_eq!(def.format_version, 1);
    let none = parse_definition(
        &with_header("[[job]]\nid = \"a\"\ncommand = \"c\"\n"),
        "t",
        None,
    )
    .unwrap();
    assert_eq!(none.run_id_env, None);
}

#[test]
fn fr2_run_id_env_の型と形の誤りは定義の誤り() {
    assert_eq!(run(&with_run_id_env("5")), vec![(K::TypeMismatch, Some(4))]);
    for bad in ["\"1A\"", "\"\"", "\"A-B\"", "\"A B\"", "\"é\"", "\"A=B\""] {
        assert_eq!(
            run(&with_run_id_env(bad)),
            vec![(K::InvalidEnvName, Some(4))],
            "{bad}"
        );
    }
}

#[test]
fn fr2_誤りの文に_run_id_env_の値を含めない() {
    let errs = parse_definition(&with_run_id_env("\"9secret_value\""), "t.toml", None).unwrap_err();
    for e in errs {
        assert!(!e.to_string().contains("secret_value"), "{e}");
    }
}
