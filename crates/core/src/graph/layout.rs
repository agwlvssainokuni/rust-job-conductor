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

//! 「対応する形」の判定(BR1.1)と、レーンの配置(BR1.2〜BR1.4)。
//!
//! ジョブは主線(左の列)に記述順で並び、飛び越しの辺は、主線の右隣(列 1)から始まる線になる。
//! 配置は、1 つの仕組み(線の並びと寄せ)だけで行う(BR1.2)。規範は、設計の記録の参照実装
//! `layout-reference.py`。`git log --graph` の出力の観察から立てた規則を、新しく書いた純粋な関数である
//! (git のソースは読まない。BR5.1)。

use std::collections::{HashMap, HashSet};

use super::{EdgeTarget, FlowGraph, GraphLayout, LaneGlyph, LaneLayout, LaneRow};

/// ジョブの飛び越しの種類(BR1.1。functional-spec.md の §6b の 3)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SkipKind {
    /// 飛び越しなし: 辺は、次の節への 1 本。
    NoSkip,
    /// 通常の飛び越し: 次の節への辺と、それ以外への辺 1 本。
    Ordinary,
    /// 「両方の飛び越し」: 辺は 1 本(成功と警告の両方)で、行き先は次の節でない。
    BothSkip,
    /// 分岐: 辺は 2 本で、どちらも次の節でなく、行き先が異なる。
    Branch,
}

/// 配置の途中結果(core の内部)。ジョブごとの、次の節への辺の有無、飛び越しの種類、行き先(BR1.1)。
struct SkipPlan {
    /// 次の節(記述順の次のジョブ、または終点)への辺を持つか。
    has_next: Vec<bool>,
    /// 飛び越しの種類。
    kinds: Vec<SkipKind>,
    /// 飛び越しの行き先(節の位置。終点は、ジョブの数)の昇順の並び(0〜2 個)。
    /// 分岐では、近い行き先が先(内側の線になる。BR1.3)。
    targets: Vec<Vec<usize>>,
}

/// 各ジョブの、次の節への辺の有無、飛び越しの種類、行き先(BR1.1)。
///
/// すべてのジョブが、次のどれかなら、レーンの図にする。1 つでも違えば、全体が `None`(辺の一覧)。
/// 交差と同じ行き先は、どの組み合わせでも、図にする(BR2.1)。
///
/// - 飛び越しなし: 辺は、次の節への 1 本。
/// - 通常の飛び越し: 次の節への辺と、それ以外への辺 1 本。
/// - 「両方の飛び越し」: 辺は 1 本(成功と警告の両方)で、行き先は次の節でない。
/// - 分岐: 辺は 2 本で、どちらも次の節でなく、行き先が異なる(辺は行き先ごとにまとまるので、異なる)。
fn skip_targets(graph: &FlowGraph) -> Option<SkipPlan> {
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

    // 元のジョブごとに、次の節への辺と、飛び越しの辺を分ける。
    let mut has_next = vec![false; job_count];
    let mut targets: Vec<Vec<usize>> = vec![Vec::new(); job_count];
    let mut both_kinds = vec![false; job_count];
    for edge in &graph.edges {
        let from = *position.get(edge.from_job.as_str())?;
        let to = to_position(&edge.to)?;
        if from >= job_count || to <= from {
            return None;
        }
        if to == from + 1 {
            has_next[from] = true;
        } else {
            both_kinds[from] = edge.kinds.len() == 2;
            targets[from].push(to);
        }
    }
    let mut kinds = Vec::with_capacity(job_count);
    for i in 0..job_count {
        targets[i].sort_unstable();
        let kind = match (has_next[i], targets[i].as_slice()) {
            (true, []) => SkipKind::NoSkip,
            (true, [_]) => SkipKind::Ordinary,
            (false, [_]) if both_kinds[i] => SkipKind::BothSkip,
            (false, [t1, t2]) if t1 != t2 => SkipKind::Branch,
            // 次の節への辺も飛び越しもない、成功だけの辺が 1 本、辺が 3 本以上など(辺の一覧)。
            _ => return None,
        };
        kinds.push(kind);
    }
    Some(SkipPlan {
        has_next,
        kinds,
        targets,
    })
}

/// 図の形が、レーンの図(`git log --graph` と同じ見た目)で描ける「対応する形」か(BR1.1)。
///
/// 偽なら、text では、レーンの図の代わりに、辺の一覧を出す。Mermaid は、形によらず出せる。
pub fn is_lane_drawable(graph: &FlowGraph) -> bool {
    skip_targets(graph).is_some()
}

/// 図の配置を決める。「対応する形」ならレーンの図、そうでなければ辺の一覧(BR1.1、BR1.2)。
pub fn layout_graph(graph: &FlowGraph) -> GraphLayout {
    match skip_targets(graph) {
        Some(plan) => GraphLayout::Lanes(layout_lanes(&plan)),
        None => GraphLayout::EdgeList,
    }
}

/// 実行されうる節(先頭のジョブから、辺をたどって着ける。BR1.4)の印。節の位置ごと(終点を含む)。
/// 辺を 1 回ずつたどるので、ジョブと辺の数に比例する。
fn reachable_nodes(plan: &SkipPlan) -> Vec<bool> {
    let job_count = plan.targets.len();
    let mut reachable = vec![false; job_count + 1];
    reachable[0] = true;
    let mut stack = vec![0usize];
    while let Some(from) = stack.pop() {
        if from == job_count {
            continue;
        }
        let next = plan.has_next[from].then_some(from + 1);
        for to in next.into_iter().chain(plan.targets[from].iter().copied()) {
            if !reachable[to] {
                reachable[to] = true;
                stack.push(to);
            }
        }
    }
    reachable
}

/// 主線の列 0 の記号(BR1.4)。節の位置ごと。ノード `j` と次のノードのあいだの行(分岐の行、合流の行、
/// 寄せの行)の列 0 は、ジョブ `j` が実行されうる、かつ次の節への辺を持つときだけ `|`、それ以外は空白。
/// 終点は空白。
fn main_line_glyphs(plan: &SkipPlan) -> Vec<LaneGlyph> {
    let reachable = reachable_nodes(plan);
    let job_count = plan.targets.len();
    (0..=job_count)
        .map(|j| {
            if j < job_count && reachable[j] && plan.has_next[j] {
                LaneGlyph::Line
            } else {
                LaneGlyph::Empty
            }
        })
        .collect()
}

/// 線をつなぐ行(分岐): 新しい線 `new` 本(通常の飛び越しと「両方の飛び越し」は 1、分岐は 2)を列 1 から
/// 入れ、すでにある `existing` 本が右へずれる(BR1.3)。列 0 は `first`(BR1.4)。
fn fork_row(existing: usize, new: usize, first: LaneGlyph) -> Vec<LaneGlyph> {
    let mut glyphs = vec![first, LaneGlyph::Fork];
    for _ in 0..(existing + new - 1) {
        glyphs.extend([LaneGlyph::Empty, LaneGlyph::Fork]);
    }
    glyphs
}

/// 線をつなぐ行(同じ行き先への合流 `|\|`): 既存の最も内側の線(列 1)に重なる。外側の `outer` 本は、
/// そのまま。列 0 は `first`(BR1.4)。
fn join_lane_row(outer: usize, first: LaneGlyph) -> Vec<LaneGlyph> {
    let mut glyphs = vec![first, LaneGlyph::Fork, LaneGlyph::Line];
    for _ in 0..outer {
        glyphs.extend([LaneGlyph::Empty, LaneGlyph::Line]);
    }
    glyphs
}

/// 直前に作った「寄せ」の種類(ノードの行の、既存の線の記号を決める)。
#[derive(Clone, Copy, PartialEq, Eq)]
enum LastMove {
    /// 寄せなし。
    None,
    /// 1 行の寄せ(`c - d = 1`)。
    OneRow,
    /// 2 行の、同じ行き先の寄せ(`d >= 1`)。
    DuplicateTwoRows,
    /// 2 行の、到着する線の寄せ(`d = 0`、`k >= 1`)。
    ArrivalTwoRows,
}

/// マスの並びの、末尾の空白を取る。
fn trim_row(mut glyphs: Vec<LaneGlyph>) -> Vec<LaneGlyph> {
    while glyphs.last() == Some(&LaneGlyph::Empty) {
        glyphs.pop();
    }
    glyphs
}

/// 列 0 を `first` にして、末尾の空白を取る(BR1.4)。
fn finish_row(mut glyphs: Vec<LaneGlyph>, first: LaneGlyph) -> Vec<LaneGlyph> {
    if let Some(column0) = glyphs.first_mut() {
        *column0 = first;
    }
    trim_row(glyphs)
}

/// 「寄せ」`MOVE(c, d)` の行。列 `c`(1 始まり)の線を、列 `d`(0 は主線)へ寄せる。
/// `m` は寄せる前の線の数、`rest` は寄せる線を除いた線の並び(内側が先頭)、`next` は次のノードの位置、
/// `first` は列 0 の記号(BR1.4)。戻り値は、行と、1 行の寄せか。
fn move_rows(
    c: usize,
    d: usize,
    m: usize,
    rest: &[usize],
    next: usize,
    first: LaneGlyph,
) -> (Vec<Vec<LaneGlyph>>, bool) {
    if c - d == 1 {
        let mut row = Vec::with_capacity(2 * m);
        for _ in 0..c - 1 {
            row.extend([LaneGlyph::Line, LaneGlyph::Empty]);
        }
        row.extend([LaneGlyph::Line, LaneGlyph::Join]);
        for _ in 0..m - c {
            row.extend([LaneGlyph::Empty, LaneGlyph::Join]);
        }
        return (vec![finish_row(row, first)], true);
    }
    // 行 1: 列 0 〜 `c - 1` を `|` で、間の空白のうち、右の `c - d - 2` 個を `_` にする。
    let underscores = c - d - 2;
    let mut row1 = Vec::with_capacity(2 * m);
    for j in 0..c {
        row1.push(LaneGlyph::Line);
        if j < c - 1 {
            row1.push(if j >= (c - 1) - underscores {
                LaneGlyph::Underscore
            } else {
                LaneGlyph::Empty
            });
        }
    }
    row1.push(LaneGlyph::Join);
    for _ in 0..m - c {
        row1.extend([LaneGlyph::Empty, LaneGlyph::Join]);
    }
    // 行 2: 最初の重複から先は `/`。`d >= 1` のときは、寄せた線より外側で、次のノードが行き先の線から先も `/`。
    let first_duplicate = {
        let mut seen = HashSet::new();
        rest.iter()
            .position(|t| !seen.insert(*t))
            .unwrap_or(rest.len())
    };
    let mut row2 = Vec::with_capacity(2 * m);
    for _ in 0..d {
        row2.extend([LaneGlyph::Line, LaneGlyph::Empty]);
    }
    row2.extend([LaneGlyph::Line, LaneGlyph::Join]);
    let outer_start = if d >= 1 {
        rest.len() - (m - c)
    } else {
        rest.len()
    };
    let mut reached_next = false;
    for (j, &target) in rest.iter().enumerate().skip(d) {
        if j >= outer_start && target == next {
            reached_next = true;
        }
        let slash = j >= first_duplicate || (d >= 1 && j >= outer_start && reached_next);
        row2.extend([
            if slash {
                LaneGlyph::Join
            } else {
                LaneGlyph::Line
            },
            LaneGlyph::Empty,
        ]);
    }
    (
        vec![finish_row(row1, first), finish_row(row2, first)],
        false,
    )
}

/// 寄せる、同じ行き先の組 `(a, b)`(`a < b`)を 1 つ探す。`a` の小さい順、同じなら `b` の小さい順。
/// `arriving` は `(到着する線の位置, 到着するノードの位置)`。その行き先の組と、到着する線をまたぐ組は除く。
/// 各 `a` について、次の同じ行き先の線だけ見ればよい(より後ろの線も、またぐ)ので、線の数に比例する。
fn find_duplicate_pair(
    lanes: &[usize],
    arriving: Option<(usize, usize)>,
) -> Option<(usize, usize)> {
    let mut next_same = vec![usize::MAX; lanes.len()];
    let mut last_seen: HashMap<usize, usize> = HashMap::new();
    for (index, &target) in lanes.iter().enumerate().rev() {
        if let Some(later) = last_seen.insert(target, index) {
            next_same[index] = later;
        }
    }
    for (a, &b) in next_same.iter().enumerate() {
        if b == usize::MAX {
            continue;
        }
        if let Some((k, node)) = arriving
            && (lanes[a] == node || (a < k && k < b))
        {
            continue;
        }
        return Some((a, b));
    }
    None
}

/// 同じ行き先の重複を、なくなるまで寄せる。寄せるたびに、先頭から探し直す。`first` は列 0 の記号。
fn settle_duplicates(
    lanes: &mut Vec<usize>,
    arriving: Option<(usize, usize)>,
    node: usize,
    first: LaneGlyph,
    rows: &mut Vec<LaneRow>,
    last: &mut LastMove,
) {
    while let Some((a, b)) = find_duplicate_pair(lanes, arriving) {
        let m = lanes.len();
        lanes.remove(b);
        let (moved, one_row) = move_rows(b + 1, a + 1, m, lanes, node, first);
        rows.extend(moved.into_iter().map(|glyphs| LaneRow {
            glyphs,
            node_index: None,
        }));
        *last = if one_row {
            LastMove::OneRow
        } else {
            LastMove::DuplicateTwoRows
        };
    }
}

/// レーンの配置(BR1.2〜BR1.4。唯一の配置)。参照実装は `layout-reference.py` の `layout`。
///
/// 開いている線 `lanes` は、内側が先頭。同じ行き先の重複を許す。節ごとに、次の順で行を作る。
///
/// 1. 到着する線をまたがない重複を寄せ、到着する線を主線へ寄せ、残りの重複を寄せる。
/// 2. ノードの行。
/// 3. 通常の飛び越しと「両方の飛び越し」は、先頭の線と行き先が同じなら、合流の行 `|\|`(線を作らない)、
///    違えば、分岐の行と、新しい線 1 本。分岐は、分岐の行と、新しい線 2 本(近い行き先が内側。BR1.3)。
///
/// ノード `j` と次のノードのあいだの行の列 0 は、`main_line_glyphs` の記号(BR1.4)。
fn layout_lanes(plan: &SkipPlan) -> LaneLayout {
    let job_count = plan.targets.len();
    let main_line = main_line_glyphs(plan);
    let mut rows: Vec<LaneRow> = Vec::with_capacity(job_count * 3 + 2);
    let mut lanes: Vec<usize> = Vec::new();
    let mut after_fork = false;
    for i in 0..=job_count {
        let start_len = rows.len();
        let mut last = LastMove::None;
        // この節の手前の寄せの行は、上のノード(`i - 1`)と、このノードのあいだの行。
        let above = i.checked_sub(1).map_or(LaneGlyph::Empty, |j| main_line[j]);
        if let Some(first) = lanes.iter().position(|&t| t == i) {
            settle_duplicates(&mut lanes, Some((first, i)), i, above, &mut rows, &mut last);
            let m = lanes.len();
            let k = lanes.iter().position(|&t| t == i).unwrap_or(0);
            // 到着する線と、同じ行き先の別の線は、取り除く。
            let rest: Vec<usize> = lanes.iter().copied().filter(|&t| t != i).collect();
            let (moved, one_row) = move_rows(k + 1, 0, m, &rest, i, above);
            rows.extend(moved.into_iter().map(|glyphs| LaneRow {
                glyphs,
                node_index: None,
            }));
            last = if one_row {
                LastMove::OneRow
            } else {
                LastMove::ArrivalTwoRows
            };
            lanes = rest;
        }
        settle_duplicates(&mut lanes, None, i, above, &mut rows, &mut last);
        if rows.len() > start_len {
            after_fork = false;
        }
        let existing = lanes.len();
        if i == job_count {
            rows.push(LaneRow {
                glyphs: vec![LaneGlyph::EndNode, LaneGlyph::Empty],
                node_index: Some(i),
            });
            break;
        }
        let kind = plan.kinds[i];
        let targets = &plan.targets[i];
        let joins = matches!(kind, SkipKind::Ordinary | SkipKind::BothSkip)
            && lanes.first() == targets.first();
        let new_lanes = if joins { 0 } else { targets.len() };
        let new_lane = new_lanes > 0;
        // 次のノードが線の行き先なら、その線から先を `/` で描く。
        let (slash_from, slash_only_first) = match lanes.iter().position(|&t| t == i + 1) {
            Some(q) if !new_lane && last == LastMove::OneRow => (Some(q), false),
            Some(0) if kind == SkipKind::NoSkip && last == LastMove::DuplicateTwoRows => {
                (Some(0), true)
            }
            _ => (None, false),
        };
        let existing_glyph = |j: usize| {
            if new_lane {
                if after_fork {
                    LaneGlyph::Fork
                } else {
                    LaneGlyph::Line
                }
            } else if slash_from.is_some_and(|q| if slash_only_first { j == q } else { j >= q }) {
                LaneGlyph::Join
            } else {
                LaneGlyph::Line
            }
        };
        let width = 2 * (1 + existing + new_lanes);
        let mut glyphs = Vec::with_capacity(width);
        glyphs.push(LaneGlyph::Node);
        for j in 0..existing {
            glyphs.extend([LaneGlyph::Empty, existing_glyph(j)]);
        }
        glyphs.resize(width, LaneGlyph::Empty);
        rows.push(LaneRow {
            glyphs,
            node_index: Some(i),
        });
        if joins {
            rows.push(LaneRow {
                glyphs: join_lane_row(existing - 1, main_line[i]),
                node_index: None,
            });
        } else if new_lane {
            rows.push(LaneRow {
                glyphs: fork_row(existing, new_lanes, main_line[i]),
                node_index: None,
            });
            // 新しい線を先頭に入れる(分岐は、近い行き先が内側)。
            lanes.splice(0..0, targets.iter().copied());
        }
        after_fork = new_lane;
    }
    LaneLayout { rows, main_line }
}
