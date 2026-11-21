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

//! 図のテストの共通の部品(定義の組み立てと、レーンの最小の文字化)。

#![allow(dead_code)]

use conductor_core::graph::{GraphLayout, LaneGlyph, LaneLayout, build_graph, layout_graph};
use conductor_core::{
    JobDefinition, PipelineDefinition, Transition, TransitionOn, TransitionTarget,
};

/// 遷移の指定。`(元のジョブ、結果、行き先)`。行き先 `None` は `@end`。
pub type Spec<'a> = (&'a str, TransitionOn, Option<&'a str>);

/// ジョブ ID の並びと遷移から、定義を作る。
pub fn definition(ids: &[&str], transitions: &[Spec<'_>]) -> PipelineDefinition {
    let job = ids
        .iter()
        .map(|id| JobDefinition {
            id: (*id).to_string(),
            command: format!("echo {id}").as_str().into(),
            warning_exit_codes: vec![3],
            success_exit_codes: None,
            transitions: transitions
                .iter()
                .filter(|(from, _, _)| from == id)
                .map(|(_, on, to)| Transition {
                    on: *on,
                    next: match to {
                        Some(t) => TransitionTarget::Job((*t).to_string()),
                        None => TransitionTarget::End,
                    },
                })
                .collect(),
            retry: None,
            env: Vec::new(),
            env_file: Vec::new(),
        })
        .collect();
    PipelineDefinition {
        name: "p".into(),
        description: None,
        run_id_env: None,
        remove_on_complete: false,
        format_version: 1,
        job,
        env: Vec::new(),
        env_file: Vec::new(),
    }
}

/// `a`、`b`、… の `n` 個のジョブ ID。
pub fn letters(n: usize) -> Vec<String> {
    (0..n)
        .map(|i| ((b'a' + i as u8) as char).to_string())
        .collect()
}

/// レーンの行を、文字にする(テスト側の最小の整形。cli の整形と同じ規則)。
pub fn render_lanes(layout: &LaneLayout, labels: &[String]) -> String {
    let mut out = String::new();
    for row in &layout.rows {
        let mut line: String = row
            .glyphs
            .iter()
            .map(|g| match g {
                LaneGlyph::Empty => ' ',
                LaneGlyph::Node => '*',
                LaneGlyph::EndNode => 'o',
                LaneGlyph::Line => '|',
                LaneGlyph::Fork => '\\',
                LaneGlyph::Join => '/',
            })
            .collect();
        if let Some(i) = row.node_index {
            line.push_str(&labels[i]);
        }
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}

/// 定義から、レーンの図の文字列を作る(辺の一覧になるときは `None`)。
pub fn lanes_text(def: &PipelineDefinition) -> Option<String> {
    let graph = build_graph(def, None);
    let labels: Vec<String> = def
        .job
        .iter()
        .map(|j| j.id.clone())
        .chain(std::iter::once("@end".to_string()))
        .collect();
    match layout_graph(&graph) {
        GraphLayout::Lanes(layout) => Some(render_lanes(&layout, &labels)),
        GraphLayout::EdgeList => None,
    }
}
