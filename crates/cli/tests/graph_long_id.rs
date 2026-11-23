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

//! `graph` の text の、65 文字以上のジョブ ID の表示(internal-cleanup-2 の BR2.4)。
//!
//! 節のラベルと辺の行き先は、`escape_full`(64 文字で切らない)でエスケープする。遷移の表示
//! (`escape_name`。64 文字で切る)とは使い分ける。行き先を型(`TransitionTarget`)で渡すように
//! 変えても、長いジョブ ID が切られないことを確かめる。

use conductor_cli::Language;
use conductor_cli::report::graph_text::render_text;
use conductor_core::graph::{GraphLayout, build_graph, layout_graph};
use conductor_core::{PipelineDefinition, parse_definition};

/// `a` の成功が、長い ID のジョブ(3 番目)へ飛ぶ定義。
fn definition(long: &str) -> PipelineDefinition {
    let text = format!(
        "format_version = 1\n[pipeline]\nname = \"p\"\n\
         [[job]]\nid = \"a\"\ncommand = \"echo a\"\n\
         [[job.transitions]]\non = \"success\"\nnext = \"{long}\"\n\
         [[job]]\nid = \"b\"\ncommand = \"echo b\"\n\
         [[job]]\nid = \"{long}\"\ncommand = \"echo c\"\n"
    );
    parse_definition(&text, "c.toml", None).expect("正しい定義")
}

fn long_id() -> String {
    format!("{}z", "x".repeat(69))
}

#[test]
fn レーンの図では_65_文字以上のジョブ_id_を切らずに出す() {
    let long = long_id();
    assert_eq!(long.chars().count(), 70);
    let def = definition(&long);
    let graph = build_graph(&def, None);
    let layout = layout_graph(&graph);
    for verbose in [false, true] {
        let out = render_text(Language::En, &def, &graph, &layout, verbose);
        assert!(out.contains(&long), "{out}");
        assert!(!out.contains('…'), "切らない: {out}");
    }
}

#[test]
fn 辺の一覧では_65_文字以上の行き先のジョブ_id_を切らずに出す() {
    let long = long_id();
    let def = definition(&long);
    let graph = build_graph(&def, None);
    for lang in [Language::En, Language::Ja] {
        let out = render_text(lang, &def, &graph, &GraphLayout::EdgeList, false);
        // 節のラベルと辺の行き先の両方に、全体が出る。
        assert!(out.matches(long.as_str()).count() >= 2, "{out}");
        assert!(!out.contains('…'), "切らない: {out}");
    }
}
