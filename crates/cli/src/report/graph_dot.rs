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

//! `graph --format dot` の整形(U4。BR2.1〜BR3.2)。
//!
//! `digraph conductor {`、`rankdir=TB;`、節の定義、辺の定義、`}` の順。Mermaid と同じ `FlowGraph` から、
//! 同じ節と辺を、同じ順で描く(BR2.1)。節の ID は連番(`n0`、`n1`、…。終点は `nend`)で、ジョブ ID は、
//! 二重引用符つきのラベルにだけ書く(BR2.2)。`--state` のときは、ラベルの先頭に状態の記号を付け、
//! 再開位置のジョブを太い枠(`penwidth=3`)にする(BR2.4)。`-v` は影響しない(BR2.5)。
//! 書く入力の値は、ジョブ ID だけ(BR3.2)。

use std::collections::HashMap;
use std::fmt::Write;

use conductor_core::graph::{EdgeTarget, FlowGraph, NodeKind};

use super::escape_full;
use super::graph_text::{is_main_edge, kind_label, state_symbol};
use crate::types::Language;

/// ラベルの文字列を、DOT の二重引用符の文字列の中に書ける形にする(BR3.1)。
///
/// まず text と同じ見える形(`escape_full`。改行は `\n`、書式の制御文字は `\u{..}` など)にし、
/// そのあと `\` を `\\`、`"` を `\"` にする。これで、DOT の `\n`・`\l` などの特別な列がラベルの中で
/// 働かず、どのジョブ ID でも文書が壊れない。
pub fn escape_dot_label(text: &str) -> String {
    let shown = escape_full(text);
    let mut out = String::with_capacity(shown.len());
    for c in shown.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            c => out.push(c),
        }
    }
    out
}

/// DOT の文書を作る(最後の改行を含む。BR2.1)。
pub fn render_dot(lang: Language, graph: &FlowGraph) -> String {
    // 1 行の大きさの目安から、最初に領域を取る(1 回の書き込みで済むように)。
    let mut out = String::with_capacity(40 + 48 * graph.nodes.len() + 32 * graph.edges.len());
    out.push_str("digraph conductor {\n");
    out.push_str("  rankdir=TB;\n");
    // ジョブ ID から節の ID を引く。
    let mut ids: HashMap<&str, String> = HashMap::with_capacity(graph.nodes.len());
    for (index, node) in graph.nodes.iter().enumerate() {
        match &node.kind {
            NodeKind::Job(job_id) => {
                let node_id = format!("n{index}");
                let label = escape_dot_label(&format!("{}{job_id}", state_symbol(node.state)));
                // 再開位置のジョブだけ、太い枠(BR2.4)。
                let penwidth = if node.is_resume { ", penwidth=3" } else { "" };
                let _ = writeln!(out, "  {node_id} [label=\"{label}\", shape=box{penwidth}];");
                ids.insert(job_id.as_str(), node_id);
            }
            NodeKind::End => {
                let _ = writeln!(
                    out,
                    "  nend [label=\"{}\", shape=oval];",
                    escape_dot_label(NodeKind::END_LABEL)
                );
            }
        }
    }
    for edge in &graph.edges {
        let from = ids.get(edge.from_job.as_str());
        let to = match &edge.to {
            EdgeTarget::Job(id) => ids.get(id.as_str()).map(String::as_str),
            EdgeTarget::End => Some("nend"),
        };
        // 行き先の分からない辺は、Mermaid と同じく出さない(BR2.1)。
        let (Some(from), Some(to)) = (from, to) else {
            continue;
        };
        // 主線の辺(次のジョブへ、成功と警告の両方)は、ラベルなし(BR2.3)。
        if is_main_edge(edge) {
            let _ = writeln!(out, "  {from} -> {to};");
        } else {
            let label = escape_dot_label(kind_label(lang, &edge.kinds));
            let _ = writeln!(out, "  {from} -> {to} [label=\"{label}\"];");
        }
    }
    out.push_str("}\n");
    out
}
