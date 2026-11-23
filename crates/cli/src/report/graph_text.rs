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

//! `graph` の text の整形(WF4。BR2.2、BR2.5、BR2.8、BR3.1〜BR3.3、BR4.1、BR4.2)。
//!
//! 配置(`GraphLayout`)を、ASCII の文字(`*`、`o`、`|`、`\`、`/`)にし、ラベルと詳細の行を
//! 足す(純粋)。環境変数は、名前も値も、出さない(BR3.3、FR1.12)。入力に由来する文字列
//! (ジョブ ID、コマンド)は、制御文字をエスケープして出す。

use std::collections::HashMap;

use conductor_core::graph::{
    EdgeKind, EdgeTarget, FlowEdge, FlowGraph, GraphLayout, GraphNode, LaneGlyph, NodeKind,
    NodeState,
};
use conductor_core::{Classification, Command, JobDefinition, PipelineDefinition};

use super::escape::escape_full;
use super::messages::{
    DETAIL_COMMAND, DETAIL_RETRY, GRAPH_EDGE_LIST_NOTE, GRAPH_EDGES_HEADING, GRAPH_JOBS_HEADING,
    GRAPH_KIND_BOTH, GRAPH_NOTE_SEPARATOR, RESUME_HERE, class_word, graph_edge_note,
    graph_retry_summary,
};
use super::push_line;
use crate::types::Language;

/// 状態の記号(BR4.1)。`--state` でなければ、空。
pub(crate) fn state_symbol(state: Option<NodeState>) -> &'static str {
    match state {
        Some(NodeState::Ok) => "[OK] ",
        Some(NodeState::Skipped) => "[--] ",
        Some(NodeState::Error) => "[NG] ",
        Some(NodeState::Pending) => "[  ] ",
        None => "",
    }
}

/// ジョブの節のラベル(状態の記号、ジョブ ID、再開位置の注記)。終点は `@end`(BR4.1、BR4.2)。
fn node_label(lang: Language, node: &GraphNode) -> String {
    let id = match &node.kind {
        NodeKind::Job(id) => id,
        NodeKind::End => return NodeKind::END_LABEL.to_string(),
    };
    let mut label = format!("{}{}", state_symbol(node.state), escape_full(id));
    if node.is_resume {
        label.push_str(RESUME_HERE.get(lang));
    }
    label
}

/// コマンドの表示(BR3.3)。文字列は、そのまま(エスケープのみ)。配列は、要素ごとに、空白か空の
/// ときだけ二重引用符で囲み、空白でつなぐ。
fn command_text(command: &Command) -> String {
    match command {
        Command::Line(line) => escape_full(line),
        Command::Argv(argv) => argv
            .iter()
            .map(|arg| {
                let shown = escape_full(arg);
                if arg.is_empty() || arg.chars().any(char::is_whitespace) {
                    format!("\"{shown}\"")
                } else {
                    shown
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
    }
}

/// 辺の種類のラベル(成功だけ、警告だけ、両方)。text と Mermaid が使う。
pub(crate) fn kind_label(lang: Language, kinds: &[EdgeKind]) -> &'static str {
    match kinds {
        [EdgeKind::Success] => class_word(Classification::Success).get(lang),
        [EdgeKind::Warning] => class_word(Classification::Warning).get(lang),
        _ => GRAPH_KIND_BOTH.get(lang),
    }
}

/// 主線の辺(次のジョブへ、成功と警告の両方)か(BR3.2、BR5.3)。text と Mermaid が使う。
pub(crate) fn is_main_edge(edge: &FlowEdge) -> bool {
    edge.kinds.len() == 2 && edge.skipped_jobs.is_empty()
}

/// 行き先の表示(ジョブ ID はエスケープ。終点は `@end`)。
/// 64 文字で切らない `escape_full` を使う(遷移の表示の `escape_name` とは使い分ける)。`@end` はエスケープしても同じ。
fn target_text(to: &EdgeTarget) -> String {
    escape_full(to.label())
}

/// 飛び越しの注記(BR3.2)。辺が 2 つあるとき、または飛び越すジョブがあるときだけ。
fn skip_note(lang: Language, edges: &[&FlowEdge]) -> Option<String> {
    if edges.len() < 2 && edges.iter().all(|e| e.skipped_jobs.is_empty()) {
        return None;
    }
    // 成功の辺を先に、警告の辺を後に。
    let mut ordered: Vec<&&FlowEdge> = edges.iter().collect();
    ordered.sort_by_key(|e| e.kinds.first() != Some(&EdgeKind::Success));
    let notes: Vec<String> = ordered
        .iter()
        .map(|e| {
            let skipped: Vec<String> = e.skipped_jobs.iter().map(|s| escape_full(s)).collect();
            graph_edge_note(
                lang,
                kind_label(lang, &e.kinds),
                &target_text(&e.to),
                &skipped,
            )
        })
        .collect();
    Some(notes.join(GRAPH_NOTE_SEPARATOR.get(lang)))
}

/// `-v` の詳細のために、辺を出どころのジョブ ID で 1 回まとめる(各ジョブの中の並びは `graph.edges` の順)。
fn edges_by_job(graph: &FlowGraph) -> HashMap<&str, Vec<&FlowEdge>> {
    let mut by_job: HashMap<&str, Vec<&FlowEdge>> = HashMap::new();
    for edge in &graph.edges {
        by_job.entry(edge.from_job.as_str()).or_default().push(edge);
    }
    by_job
}

/// `-v` の詳細の文(コマンド、リトライ、飛び越しの順。該当しない項目は出さない。BR3.1)。
/// `by_job` は [`edges_by_job`] でまとめた辺。
fn detail_sentences(
    lang: Language,
    job: &JobDefinition,
    by_job: &HashMap<&str, Vec<&FlowEdge>>,
) -> Vec<String> {
    let mut lines = vec![format!(
        "{}{}",
        DETAIL_COMMAND.get(lang),
        command_text(&job.command)
    )];
    if let Some(retry) = &job.retry {
        lines.push(format!(
            "{}{}",
            DETAIL_RETRY.get(lang),
            graph_retry_summary(lang, retry)
        ));
    }
    let edges = by_job.get(job.id.as_str()).map_or(&[][..], Vec::as_slice);
    if let Some(note) = skip_note(lang, edges) {
        lines.push(note);
    }
    lines
}

/// text の図を作る(最後の改行を含む)。`layout` が辺の一覧なら、辺の一覧にする(BR2.8)。
pub fn render_text(
    lang: Language,
    definition: &PipelineDefinition,
    graph: &FlowGraph,
    layout: &GraphLayout,
    verbose: bool,
) -> String {
    match layout {
        GraphLayout::Lanes(lanes) => render_lanes(lang, definition, graph, lanes, verbose),
        GraphLayout::EdgeList => render_edge_list(lang, definition, graph, verbose),
    }
}

/// レーンの 1 マスの文字(BR2.2。すべて ASCII)。
fn glyph_char(glyph: &LaneGlyph) -> char {
    match glyph {
        LaneGlyph::Empty => ' ',
        LaneGlyph::Node => '*',
        LaneGlyph::EndNode => 'o',
        LaneGlyph::Line => '|',
        LaneGlyph::Fork => '\\',
        LaneGlyph::Join => '/',
        LaneGlyph::Underscore => '_',
    }
}

fn render_lanes(
    lang: Language,
    definition: &PipelineDefinition,
    graph: &FlowGraph,
    lanes: &conductor_core::graph::LaneLayout,
    verbose: bool,
) -> String {
    let mut out = String::new();
    let by_job = if verbose {
        edges_by_job(graph)
    } else {
        HashMap::new()
    };
    let rows = &lanes.rows;
    let mut i = 0;
    while i < rows.len() {
        let row = &rows[i];
        let mut line: String = row.glyphs.iter().map(glyph_char).collect();
        let Some(node_index) = row.node_index else {
            push_line(&mut out, &line);
            i += 1;
            continue;
        };
        line.push_str(&node_label(lang, &graph.nodes[node_index]));
        push_line(&mut out, &line);
        i += 1;
        if !verbose || graph.nodes[node_index].is_end() {
            continue;
        }
        // 詳細の行は、ノードの行と、続く分岐の行のあと(合流の行の前)に置く(BR3.1)。
        if let Some(next) = rows.get(i)
            && next.node_index.is_none()
            && next.glyphs.contains(&LaneGlyph::Fork)
        {
            let fork: String = next.glyphs.iter().map(glyph_char).collect();
            push_line(&mut out, &fork);
            i += 1;
        }
        // 列の数だけ `|` を描き、(列の数 × 2)の位置に、詳細の文を置く。列 0 は、配置が決めた主線の記号
        // (実行されないジョブや、次のジョブへの辺がないジョブの下は空白。BR1.4)。
        let columns = row.glyphs.len() / 2;
        let column0 = lanes
            .main_line
            .get(node_index)
            .map_or(LaneGlyph::Line, |g| *g);
        let prefix = format!(
            "{} {}",
            glyph_char(&column0),
            "| ".repeat(columns.saturating_sub(1))
        );
        if let Some(job) = definition.job.get(node_index) {
            for sentence in detail_sentences(lang, job, &by_job) {
                push_line(&mut out, &format!("{prefix}{sentence}"));
            }
        }
    }
    out
}

fn render_edge_list(
    lang: Language,
    definition: &PipelineDefinition,
    graph: &FlowGraph,
    verbose: bool,
) -> String {
    let mut out = String::new();
    let by_job = if verbose {
        edges_by_job(graph)
    } else {
        HashMap::new()
    };
    push_line(&mut out, GRAPH_EDGE_LIST_NOTE.get(lang));
    push_line(&mut out, GRAPH_JOBS_HEADING.get(lang));
    for (index, node) in graph.nodes.iter().enumerate().filter(|(_, n)| !n.is_end()) {
        push_line(&mut out, &format!("  {}", node_label(lang, node)));
        if verbose && let Some(job) = definition.job.get(index) {
            for sentence in detail_sentences(lang, job, &by_job) {
                push_line(&mut out, &format!("    {sentence}"));
            }
        }
    }
    push_line(&mut out, GRAPH_EDGES_HEADING.get(lang));
    for edge in &graph.edges {
        let mut line = format!(
            "  {} -> {}",
            escape_full(&edge.from_job),
            target_text(&edge.to)
        );
        // 主線の辺(次のジョブへ、成功と警告の両方)以外は、種類を添える。
        if !is_main_edge(edge) {
            line.push_str(&format!(" ({})", kind_label(lang, &edge.kinds)));
        }
        push_line(&mut out, &line);
    }
    out
}
