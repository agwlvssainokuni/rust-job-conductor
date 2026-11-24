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

//! README の検査。README を読むだけで、書き込まない。
//!
//! 節は README の並び(`graph`、`--dry-run`、`init`)。各節は、README の例と実際の出力を比べる。
//! 必須の語句と版の表記の検査は、統合テストの `tests/docs_*.rs` にある。最初に、各節が使う補助
//! (README の読み取り)を置く。

use std::path::{Path, PathBuf};

use crate::report::init_template::render_template;
use crate::types::Language;

/// リポジトリのルートの README(改行は `\n` にそろえる)。
pub fn readme() -> String {
    let root: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    std::fs::read_to_string(root.join("README.md"))
        .expect("README.md を読める")
        .replace("\r\n", "\n")
}

// ---- `graph` の説明(U2。BR7.2、FR5.2) ----

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

// ---- `--dry-run` の説明(U3。BR6.2、FR5.2) ----

#[test]
fn readme_の計画の例は_実際の出力と一致する() {
    let text = readme();
    let start = text.find("## dry-run の使い方").expect("節がある");
    let rest = &text[start..];
    let open = rest.find("```text\n").expect("例がある") + "```text\n".len();
    let body = &rest[open..];
    let example = &body[..body.find("```").expect("閉じている")];

    // 例の定義: a が終了コード 3 の警告で `@end` へ進む。`--assume a=3`。
    let definition = "format_version = 1\n\n[pipeline]\nname = \"demo\"\n\n\
        [[job]]\nid = \"a\"\ncommand = \"echo a\"\nwarning_exit_codes = [3]\n\n\
        [[job.transitions]]\non = \"warning\"\nnext = \"@end\"\n\n\
        [[job]]\nid = \"b\"\ncommand = \"echo b\"\n\n\
        [[job]]\nid = \"c\"\ncommand = \"echo c\"\n";
    let def = conductor_core::parse_definition(definition, "c.toml", None).expect("正しい定義");
    let commands: Vec<Vec<String>> = def
        .job
        .iter()
        .map(|j| vec!["echo".to_string(), j.id.clone()])
        .collect();
    let files = conductor_core::LoadedEnvFiles::new();
    let assumptions = [conductor_core::Assumption::new("a", 3)];
    let plan = conductor_core::plan_pipeline(&conductor_core::PlanInput {
        definition: &def,
        state: None,
        commands: &commands,
        env_files: &files,
        run_id: None,
        assumptions: &assumptions,
    })
    .expect("計画");
    let actual = crate::report::plan_text::render_plan(crate::types::Language::Ja, &plan);
    assert_eq!(example, actual);
}

// ---- `init` の説明(U4。BR4.2、FR5.2) ----

#[test]
fn readme_のひな形の例は_実際の日本語のひな形と一致する() {
    let text = readme();
    let start = text.find("## init の使い方").expect("節がある");
    let rest = &text[start..];
    let open = rest.find("```toml\n").expect("例がある") + "```toml\n".len();
    let body = &rest[open..];
    let example = &body[..body.find("```").expect("閉じている")];
    assert_eq!(example, render_template(Language::Ja, "my-pipeline"));
}
