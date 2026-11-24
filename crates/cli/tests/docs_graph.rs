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

//! README の `graph` の説明の検査(U2。BR7.2、FR5.2)。README を読むだけで、書き込まない。
//! 図の例と実際の出力を比べるテストは、単体テスト(`src/tests/docs_graph.rs`)にある。

mod docs_support;

use docs_support::{assert_docs_check_words, assert_readme_has, readme};

#[test]
fn readme_に_graph_の必須の語句と版の表記がある() {
    let text = readme();
    assert_readme_has(
        &text,
        &[
            "conductor graph",
            "--format mermaid",
            "--state",
            "(v0.9.0 以降)",
            "辺の一覧",
            "flowchart TD",
        ],
    );
    assert!(text.contains("## graph の使い方(v0.9.0 以降)"));
    // docs-check の必須語句は残っている
    assert_docs_check_words(&text);
}

#[test]
fn readme_の辺の一覧に切り替わる形は_4つの種類の外の形だけを述べる() {
    let text = readme();
    let line = text
        .lines()
        .find(|l| l.starts_with("- **辺の一覧に切り替わる形**"))
        .expect("記述がある");
    for word in [
        "飛び越しなし",
        "通常の飛び越し",
        "両方の飛び越し",
        "分岐",
        "どの組み合わせでも",
        "3 本以上",
    ] {
        assert!(line.contains(word), "{word} がない");
    }
    // 前回までの範囲(交差と、両方の飛び越し・分岐の組み合わせを辺の一覧にする)は、述べない(BR4.2)。
    assert!(!line.contains("グラフのどこかに"));
    assert!(!line.contains("同じ行き先で合流する形"));
    assert!(text.contains("`|\\|` の行"));
}
