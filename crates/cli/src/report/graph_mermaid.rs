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

//! `graph --format mermaid` の整形(WF5。BR5.1〜BR5.5)。
//!
//! `flowchart TD`、節の定義、辺の定義、(再開位置があるときだけ)`classDef` と `class` の順。
//! 節の ID は連番(`n0`、`n1`、…。終点は `nend`)で、ジョブ ID は、二重引用符つきのラベルに書く
//! (どのジョブ ID でも図が壊れない。BR5.2)。形によらず、常に出せる。`-v` は影響しない(BR3.1)。

use std::collections::HashMap;
use std::fmt::Write;

use conductor_core::graph::{EdgeTarget, FlowGraph, NodeKind};

use super::graph_text::{is_main_edge, kind_label, state_symbol};
use crate::types::Language;

/// ラベルの中の文字を、Mermaid の実体の記法にする(BR5.4)。`#` は、最初に置き換える
/// (置き換えで作った `#` を、さらに置き換えない)。
pub fn escape_label(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '#' => out.push_str("#35;"),
            '"' => out.push_str("#quot;"),
            '<' => out.push_str("#lt;"),
            '>' => out.push_str("#gt;"),
            '&' => out.push_str("#amp;"),
            '`' => out.push_str("#96;"),
            c if c.is_control() => {
                let _ = write!(out, "#{};", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

/// Mermaid の図を作る(最後の改行を含む)。
pub fn render_mermaid(lang: Language, graph: &FlowGraph) -> String {
    let mut out = String::from("flowchart TD\n");
    // ジョブ ID から節の ID を引く。
    let mut ids: HashMap<&str, String> = HashMap::new();
    let mut resume: Option<String> = None;
    for (index, node) in graph.nodes.iter().enumerate() {
        match &node.kind {
            NodeKind::Job(job_id) => {
                let node_id = format!("n{index}");
                let label = format!("{}{}", state_symbol(node.state), escape_label(job_id));
                let _ = writeln!(out, "  {node_id}[\"{label}\"]");
                if node.is_resume {
                    resume = Some(node_id.clone());
                }
                ids.insert(job_id.as_str(), node_id);
            }
            NodeKind::End => {
                let _ = writeln!(out, "  nend([\"{}\"])", NodeKind::END_LABEL);
            }
        }
    }
    for edge in &graph.edges {
        let from = ids.get(edge.from_job.as_str());
        let to = match &edge.to {
            EdgeTarget::Job(id) => ids.get(id.as_str()).map(String::as_str),
            EdgeTarget::End => Some("nend"),
        };
        let (Some(from), Some(to)) = (from, to) else {
            continue;
        };
        // 主線の辺(次のジョブへ、成功と警告の両方)は、ラベルなし(BR5.3)。
        if is_main_edge(edge) {
            let _ = writeln!(out, "  {from} --> {to}");
        } else {
            let label = kind_label(lang, &edge.kinds);
            let _ = writeln!(out, "  {from} -->|\"{label}\"| {to}");
        }
    }
    if let Some(node_id) = resume {
        out.push_str("  classDef resume stroke-width:3px\n");
        let _ = writeln!(out, "  class {node_id} resume");
    }
    out
}
