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

//! ひな形(`render_template`)のテスト。U4 の BR2.1〜BR2.4。
//! 期待値は、機能の仕様(`functional-spec.md`)の原本から書いたもので、実装の出力から作っていない。

use conductor_cli::Language;
use conductor_cli::report::init_template::render_template;
use conductor_core::parse_definition;
use conductor_runtime::check_command;

/// 日本語のひな形の原本(名前は既定の `my-pipeline`)。
const ORIGINAL_JA: &str = r##"# conductor の定義ファイルのひな形(conductor init が作った)。
# 検査: conductor validate / 実行計画: conductor pipeline <名前> --dry-run / 図: conductor graph
format_version = 1

[pipeline]
name = "my-pipeline"
description = "ひな形のパイプライン"

# 全ジョブに足す環境変数。値は、書いたとおりに渡す(展開しない)。
# .env ファイルから読むときは、[pipeline] に env_file = ".env" を足す。
[pipeline.env]
GREETING = "hello"

# hello: 最初のジョブ。
# Windows では、echo は実行ファイルではないので、command = ["cmd", "/c", "echo", "hello"] のように書く。
[[job]]
id = "hello"
command = ["echo", "hello"]

# flaky: 終了コード 75 は「一時的な失敗」。5 秒待って、最大 3 回、実行し直す。
# 待ち時間を伸ばすには、multiplier と max_interval を、一緒に書く(例: multiplier = 2、max_interval = "1m")。
[[job]]
id = "flaky"
command = ["echo", "flaky"]

[job.retry]
exit_codes = [75]
max = 3
interval = "5s"
"##;

/// 英語のひな形の原本(名前は既定の `my-pipeline`)。
const ORIGINAL_EN: &str = r##"# A starter conductor definition (created by conductor init).
# Check: conductor validate / plan: conductor pipeline <name> --dry-run / diagram: conductor graph
format_version = 1

[pipeline]
name = "my-pipeline"
description = "Starter pipeline"

# Environment variables added to every job. Values are passed as written (no expansion).
# To read a .env file, add env_file = ".env" to [pipeline].
[pipeline.env]
GREETING = "hello"

# hello: the first job.
# On Windows echo is not an executable; write command = ["cmd", "/c", "echo", "hello"] instead.
[[job]]
id = "hello"
command = ["echo", "hello"]

# flaky: exit code 75 means "a temporary failure". Wait 5 seconds and run again, up to 3 times.
# To lengthen the wait, write multiplier and max_interval together (for example multiplier = 2, max_interval = "1m").
[[job]]
id = "flaky"
command = ["echo", "flaky"]

[job.retry]
exit_codes = [75]
max = 3
interval = "5s"
"##;

#[test]
fn 日本語と英語のひな形は_原本と_1_文字も違わず一致する() {
    assert_eq!(render_template(Language::Ja, "my-pipeline"), ORIGINAL_JA);
    assert_eq!(render_template(Language::En, "my-pipeline"), ORIGINAL_EN);
}

#[test]
fn 名前は_toml_の基本文字列としてエスケープされる() {
    let name_line = |name: &str| {
        let text = render_template(Language::En, name);
        text.lines()
            .find(|l| l.starts_with("name = "))
            .expect("name の行がある")
            .to_string()
    };
    assert_eq!(name_line("a\"b"), r#"name = "a\"b""#);
    assert_eq!(name_line("a\\b"), r#"name = "a\\b""#);
    assert_eq!(name_line("a\nb"), r#"name = "a\u000Ab""#);
    assert_eq!(name_line("a\u{7f}b"), r#"name = "a\u007Fb""#);
    assert_eq!(name_line("a\tb"), r#"name = "a\u0009b""#);
    assert_eq!(name_line("夜間バッチ"), r#"name = "夜間バッチ""#);
}

#[test]
fn ひな形は_utf8_で_lf_だけで_末尾に改行が_1_つ() {
    for lang in [Language::Ja, Language::En] {
        let text = render_template(lang, "夜間\nバッチ");
        assert!(!text.contains('\r'));
        assert!(text.ends_with("\"\n"));
        assert!(!text.ends_with("\n\n"));
    }
}

#[test]
fn どのひな形も_定義として読め_command_も分けられる() {
    let names = [
        "my-pipeline",
        "夜間バッチ",
        "a\"b",
        "a\\b",
        "a\nb",
        "a\u{7f}b",
        "  spaced  ",
        "# コメント風",
    ];
    for lang in [Language::Ja, Language::En] {
        for name in names {
            let text = render_template(lang, name);
            let definition = parse_definition(&text, "conductor.toml", Some(name))
                .unwrap_or_else(|e| panic!("{lang:?} {name:?}: {e:?}"));
            assert_eq!(definition.name, name);
            assert_eq!(definition.job.len(), 2);
            for job in &definition.job {
                check_command(&job.command)
                    .unwrap_or_else(|e| panic!("{lang:?} {name:?} {}: {e:?}", job.id));
            }
        }
    }
}

#[test]
fn ひな形の環境変数とリトライの例は_有効な行として検査を通る() {
    for lang in [Language::Ja, Language::En] {
        let text = render_template(lang, "my-pipeline");
        // コメントアウトされていない(U1 の `[pipeline.env]` の検査と、リトライの検査を通っている)。
        let definition = parse_definition(&text, "conductor.toml", None).expect("読める");
        assert_eq!(definition.env.len(), 1);
        assert_eq!(definition.env[0].name, "GREETING");
        assert_eq!(definition.env[0].value, "hello");
        assert!(definition.env_file.is_empty());
        let hello = &definition.job[0];
        let flaky = &definition.job[1];
        assert_eq!((hello.id.as_str(), flaky.id.as_str()), ("hello", "flaky"));
        assert!(hello.retry.is_none());
        let retry = flaky.retry.as_ref().expect("リトライの例がある");
        assert_eq!(retry.exit_codes, vec![75]);
        assert_eq!(retry.max, 3);
        assert_eq!(retry.interval_secs, 5);
    }
}

#[test]
fn ひな形に_名前以外の利用者の値は入らない() {
    for lang in [Language::Ja, Language::En] {
        let a = render_template(lang, "AAAAAAAA");
        let b = render_template(lang, "BBBBBBBB");
        assert_eq!(a.replace("AAAAAAAA", ""), b.replace("BBBBBBBB", ""));
        assert_eq!(a.matches("AAAAAAAA").count(), 1, "名前は 1 か所だけ");
    }
}

#[test]
fn 名前は_コメントに書かれない() {
    for lang in [Language::Ja, Language::En] {
        let text = render_template(lang, "UNIQUE-NAME\nINJECTED");
        for line in text.lines().filter(|l| l.starts_with('#')) {
            assert!(!line.contains("UNIQUE-NAME"), "{line}");
            assert!(!line.contains("INJECTED"), "{line}");
        }
        assert!(!text.lines().any(|l| l.starts_with("INJECTED")));
    }
}
