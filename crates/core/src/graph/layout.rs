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

//! 「対応する形」の判定(BR2.4)と、レーンの配置(BR2.1〜BR2.7)。
//!
//! ジョブは主線(左の列)に記述順で並び、飛び越しの辺は、主線の右隣(列 1)から始まる線になる。
//! 配置は、`layout-oracle.md` の例と BR2.3 の手順から、新しく書いた純粋な関数である(BR2.6)。

use std::collections::HashMap;

use super::{EdgeTarget, FlowGraph, GraphLayout, LaneGlyph, LaneLayout, LaneRow};

/// 各ジョブの飛び越しの行き先(節の位置。終点は、ジョブの数)。飛び越しがなければ `None`。
///
/// 「対応する形」でなければ、全体が `None`(BR2.4)。条件は 3 つ:
///
/// 1. 各ジョブから出る辺は、記述順の次の節への辺を持ち、それ以外の辺(飛び越し)は、多くて 1 つ。
/// 2. 飛び越しの行き先が、どの 2 つの辺でも違う(最後のジョブの、終点への辺は、飛び越しではない)。
/// 3. 飛び越しの区間が交差しない(元 1 < 元 2 < 行き先 1 < 行き先 2 になる組がない)。
///
/// 条件 2 と 3 は、記述順に節を見ながら、開いている飛び越しの行き先を積んだ列(積み上げ)で、
/// 1 回の走査で確かめる。
fn skip_targets(graph: &FlowGraph) -> Option<Vec<Option<usize>>> {
    let job_count = graph.nodes.len().checked_sub(1)?;
    let position: HashMap<&str, usize> = graph
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(i, n)| n.job_id.as_deref().map(|id| (id, i)))
        .collect();
    let to_position = |to: &EdgeTarget| -> Option<usize> {
        match to {
            EdgeTarget::End => Some(job_count),
            EdgeTarget::Job(id) => position.get(id.as_str()).copied(),
        }
    };

    // 条件 1: 元のジョブごとに、次の節への辺と、飛び越しの辺を分ける。
    let mut has_next = vec![false; job_count];
    let mut skip: Vec<Option<usize>> = vec![None; job_count];
    for edge in &graph.edges {
        let from = *position.get(edge.from_job.as_str())?;
        let to = to_position(&edge.to)?;
        if from >= job_count || to <= from {
            return None;
        }
        if to == from + 1 {
            has_next[from] = true;
        } else if skip[from].replace(to).is_some() {
            return None;
        }
    }
    if has_next.iter().any(|next| !next) {
        return None;
    }

    // 条件 2、3: 開いている行き先の積み上げ(上が最も内側)。
    let mut open: Vec<usize> = Vec::new();
    for (i, target) in skip.iter().enumerate() {
        match open.last() {
            Some(&top) if top == i => {
                open.pop();
            }
            _ => {}
        }
        // 最も内側でない線が、ここで終わるなら、交差している。
        if open.contains(&i) {
            return None;
        }
        if let Some(t) = *target {
            if open.last().is_some_and(|&top| t >= top) {
                return None;
            }
            open.push(t);
        }
    }
    Some(skip)
}

/// 図の形が、レーンの図(`git log --graph` と同じ見た目)で描ける「対応する形」か(BR2.4)。
///
/// 偽なら、text では、レーンの図の代わりに、辺の一覧を出す。Mermaid は、形によらず出せる。
pub fn is_lane_drawable(graph: &FlowGraph) -> bool {
    skip_targets(graph).is_some()
}

/// 図の配置を決める。「対応する形」ならレーンの図、そうでなければ辺の一覧(BR2.4、BR2.8)。
pub fn layout_graph(graph: &FlowGraph) -> GraphLayout {
    match skip_targets(graph) {
        Some(skip) => GraphLayout::Lanes(lane_layout(&skip)),
        None => GraphLayout::EdgeList,
    }
}

/// 線をつなぐ行(合流): 主線の線と、閉じる線(列 1)の合流、残りの `remaining` 本の左へのずれ。
fn merge_row(remaining: usize) -> Vec<LaneGlyph> {
    let mut glyphs = vec![LaneGlyph::Line, LaneGlyph::Join];
    for _ in 0..remaining {
        glyphs.extend([LaneGlyph::Empty, LaneGlyph::Join]);
    }
    glyphs
}

/// 線をつなぐ行(分岐): 新しい線を列 1 に入れ、すでにある `existing` 本が右へずれる。
fn fork_row(existing: usize) -> Vec<LaneGlyph> {
    let mut glyphs = vec![LaneGlyph::Line, LaneGlyph::Fork];
    for _ in 0..existing {
        glyphs.extend([LaneGlyph::Empty, LaneGlyph::Fork]);
    }
    glyphs
}

/// レーンの配置(BR2.1〜BR2.3、BR2.5)。`skip[i]` は、ジョブ i の飛び越しの行き先の位置。
///
/// 節ごとに、次の順で行を作る。
///
/// 1. その節が飛び越しの行き先なら、合流の行(列 1 の線が主線へ合流し、残りは左へ詰まる)。
/// 2. ノードの行。すでにある右の線は、この節が分岐を持ち、かつ直前の行が分岐の行なら `\`、
///    この節が合流の行き先(直前が合流の行)で、次の節も合流の行き先なら `/`、それ以外は `|`。
///    ラベルの位置は、分岐のあとの列の数 × 2(正解データの例が根拠)。
/// 3. 飛び越しがあれば、分岐の行。
fn lane_layout(skip: &[Option<usize>]) -> LaneLayout {
    let job_count = skip.len();
    let mut rows: Vec<LaneRow> = Vec::with_capacity(job_count * 3 + 2);
    // 開いている飛び越しの行き先。最後が列 1(最も内側)。
    let mut open: Vec<usize> = Vec::new();
    // 直前の行が分岐の行か(分岐の行の直後のノードの行では、ずれる線を `\` で描く)。
    let mut after_fork = false;
    for i in 0..=job_count {
        let is_target = open.last() == Some(&i);
        if is_target {
            rows.push(LaneRow {
                glyphs: merge_row(open.len() - 1),
                node_index: None,
            });
            open.pop();
            after_fork = false;
        }
        let existing = open.len();
        if i == job_count {
            rows.push(LaneRow {
                glyphs: vec![LaneGlyph::EndNode, LaneGlyph::Empty],
                node_index: Some(i),
            });
            break;
        }
        let target = skip.get(i).copied().flatten();
        let existing_glyph = if target.is_some() {
            if after_fork {
                LaneGlyph::Fork
            } else {
                LaneGlyph::Line
            }
        } else if is_target && open.last() == Some(&(i + 1)) {
            LaneGlyph::Join
        } else {
            LaneGlyph::Line
        };
        let width = 2 * (1 + existing + usize::from(target.is_some()));
        let mut glyphs = Vec::with_capacity(width);
        glyphs.push(LaneGlyph::Node);
        for _ in 0..existing {
            glyphs.extend([LaneGlyph::Empty, existing_glyph]);
        }
        glyphs.resize(width, LaneGlyph::Empty);
        rows.push(LaneRow {
            glyphs,
            node_index: Some(i),
        });
        if let Some(t) = target {
            rows.push(LaneRow {
                glyphs: fork_row(existing),
                node_index: None,
            });
            open.push(t);
        }
        after_fork = target.is_some();
    }
    LaneLayout { rows }
}
