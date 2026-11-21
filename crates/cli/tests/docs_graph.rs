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

use std::path::{Path, PathBuf};

fn readme() -> String {
    let root: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    std::fs::read_to_string(root.join("README.md"))
        .expect("README.md を読める")
        .replace("\r\n", "\n")
}

#[test]
fn readme_に_graph_の必須の語句と版の表記がある() {
    let text = readme();
    for word in [
        "conductor graph",
        "--format mermaid",
        "--state",
        "(v0.9.0 以降)",
        "辺の一覧",
        "flowchart TD",
    ] {
        assert!(text.contains(word), "README に {word} がない");
    }
    assert!(text.contains("## graph の使い方(v0.9.0 以降)"));
    // docs-check の必須語句は残っている
    for word in [
        "SHA-256",
        "Gatekeeper",
        "終了コードの表",
        "ステートファイルの場所",
        "at-least-once",
        "シグナル",
        "convert",
    ] {
        assert!(text.contains(word), "docs-check の語句 {word} がない");
    }
}

#[test]
fn readme_の図の例は_実際の出力と一致する() {
    let text = readme();
    let start = text.find("## graph の使い方").expect("節がある");
    let rest = &text[start..];
    let open = rest.find("```text\n").expect("例がある") + "```text\n".len();
    let body = &rest[open..];
    let example = &body[..body.find("```").expect("閉じている")];

    // 例の定義: a の成功が d へ飛ぶ 5 個のジョブ。
    let mut definition = String::from("format_version = 1\n\n[pipeline]\nname = \"p\"\n");
    for id in ["a", "b", "c", "d", "e"] {
        definition.push_str(&format!(
            "\n[[job]]\nid = \"{id}\"\ncommand = \"echo {id}\"\n"
        ));
        if id == "a" {
            definition.push_str("\n[[job.transitions]]\non = \"success\"\nnext = \"d\"\n");
        }
    }
    let def = conductor_core::parse_definition(&definition, "c.toml", None).expect("正しい定義");
    let graph = conductor_core::graph::build_graph(&def, None);
    let layout = conductor_core::graph::layout_graph(&graph);
    let actual = conductor_cli::report::graph_text::render_text(
        conductor_cli::Language::En,
        &def,
        &graph,
        &layout,
        false,
    );
    assert_eq!(example, actual);
}
