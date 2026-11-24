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
//! README の例と実際の出力を比べる単体テスト。必須の語句の検査は統合テストの同じ名前のファイル。

use super::docs_support::readme;

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
    let actual = crate::report::graph_text::render_text(
        crate::types::Language::En,
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
    crate::report::graph_text::render_text(crate::types::Language::En, &def, &graph, &layout, false)
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
fn readme_の両方の飛び越しと交差が混ざる例は_実際の出力と一致する() {
    let text = readme();
    let both_to_d = "\n[[job.transitions]]\non = \"warning\"\nnext = \"d\"\n\n[[job.transitions]]\non = \"success\"\nnext = \"d\"\n";
    let to_e = "\n[[job.transitions]]\non = \"success\"\nnext = \"e\"\n";
    let definition = jobs(
        &["a", "b", "c", "d", "e", "f"],
        &[("a", both_to_d), ("b", to_e)],
    );
    assert_eq!(graph_example(&text, 6), render(&definition));
}
