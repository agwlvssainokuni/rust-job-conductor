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

/// 「graph の使い方」の節の `n` 番目(0 始まり)の ```text の例。
fn graph_example(text: &str, n: usize) -> String {
    let start = text.find("## graph の使い方").expect("節がある");
    let end = text[start..]
        .find("## dry-run の使い方")
        .map_or(text.len(), |i| start + i);
    let section = &text[start..end];
    let mut rest = section;
    let mut found = None;
    for _ in 0..=n {
        let open = rest.find("```text\n").expect("例がある") + "```text\n".len();
        let body = &rest[open..];
        let close = body.find("```").expect("閉じている");
        found = Some(body[..close].to_string());
        rest = &body[close..];
    }
    found.unwrap()
}

fn render(definition_text: &str) -> String {
    let def =
        conductor_core::parse_definition(definition_text, "c.toml", None).expect("正しい定義");
    let graph = conductor_core::graph::build_graph(&def, None);
    let layout = conductor_core::graph::layout_graph(&graph);
    conductor_cli::report::graph_text::render_text(
        conductor_cli::Language::En,
        &def,
        &graph,
        &layout,
        false,
    )
}

fn jobs(ids: &[&str], extra_for: &[(&str, &str)]) -> String {
    let mut definition = String::from("format_version = 1\n\n[pipeline]\nname = \"p\"\n");
    for id in ids {
        definition.push_str(&format!(
            "\n[[job]]\nid = \"{id}\"\ncommand = \"echo {id}\"\n"
        ));
        for (from, extra) in extra_for {
            if from == id {
                definition.push_str(extra);
            }
        }
    }
    definition
}

#[test]
fn readme_の同じ行き先と両方の飛び越しの例は_実際の出力と一致する() {
    let text = readme();
    let to_d = "\n[[job.transitions]]\non = \"success\"\nnext = \"d\"\n";
    let same_target = jobs(&["a", "b", "c", "d", "e"], &[("a", to_d), ("b", to_d)]);
    assert_eq!(graph_example(&text, 1), render(&same_target));

    let both = "\n[[job.transitions]]\non = \"warning\"\nnext = \"c\"\n\n[[job.transitions]]\non = \"success\"\nnext = \"c\"\n";
    let both_skip = jobs(&["a", "b", "c", "d"], &[("a", both)]);
    assert_eq!(graph_example(&text, 2), render(&both_skip));
}

#[test]
fn readme_の分岐の例は_実際の出力と一致する() {
    let text = readme();
    let branch = "\n[[job.transitions]]\non = \"success\"\nnext = \"c\"\n\n[[job.transitions]]\non = \"warning\"\nnext = \"d\"\n";
    let definition = jobs(&["a", "b", "c", "d", "e"], &[("a", branch)]);
    assert_eq!(graph_example(&text, 3), render(&definition));
}

#[test]
fn readme_の交差の例は_実際の出力と一致する() {
    let text = readme();
    let to = |t: &str| format!("\n[[job.transitions]]\non = \"success\"\nnext = \"{t}\"\n");
    let definition = jobs(
        &["a", "b", "c", "d", "e"],
        &[("a", &to("d")), ("b", &to("e"))],
    );
    assert_eq!(graph_example(&text, 4), render(&definition));
}

#[test]
fn readme_の交差に同じ行き先が混ざる例は_実際の出力と一致する() {
    let text = readme();
    let to = |t: &str| format!("\n[[job.transitions]]\non = \"success\"\nnext = \"{t}\"\n");
    let definition = jobs(
        &["a", "b", "c", "d", "e"],
        &[("a", &to("e")), ("b", &to("d")), ("c", &to("e"))],
    );
    assert_eq!(graph_example(&text, 5), render(&definition));
}

#[test]
fn readme_の辺の一覧に切り替わる形は_交差と分岐の線に関わる形だけを述べる() {
    let text = readme();
    let line = text
        .lines()
        .find(|l| l.starts_with("- **辺の一覧に切り替わる形**"))
        .expect("記述がある");
    assert!(line.contains("交差"));
    assert!(line.contains("分岐"));
    assert!(line.contains("同じ行き先で合流する形"));
    assert!(line.contains("グラフのどこかに"));
    assert!(!line.contains("同じ行き先へ飛ぶ辺(同じ行き先で合流する形)を含む形"));
    assert!(!line.contains("異なる行き先への飛び越しを 2 つ持つ形"));
    assert!(!line.contains("同じ行き先へ飛ぶ辺が 2 つある"));
    assert!(text.contains("`|\\|` の行"));
}
