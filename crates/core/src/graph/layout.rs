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

/// 配置の途中結果(core の内部)。ジョブごとの飛び越しの行き先と、主線につながらないかの印。
struct SkipPlan {
    /// 飛び越しの行き先(節の位置。終点は、ジョブの数)の昇順の並び。0 個は飛び越しなし、
    /// 1 個は通常の飛び越し、2 個は分岐(BR1.1)。
    targets: Vec<Vec<usize>>,
    /// 主線につながらないジョブか(分岐と「両方の飛び越し」)。その線の主線側の `|` を描かない。
    off_main: Vec<bool>,
}

/// 開いている線(積み上げの 1 要素)。判定の走査の中だけで使う。
struct OpenLane {
    target: usize,
    /// 分岐が開いた線か。後のジョブが同じ行き先で合流する形は、辺の一覧にする(BR2.2)。
    by_branch: bool,
}

/// 各ジョブの飛び越しの行き先と、主線につながらないかの印。
///
/// 「対応する形」でなければ、全体が `None`(BR2.4)。ただし、交差を許す範囲(BR1.1。`is_crossing_class`)の
/// グラフは、条件 2、3 を調べず、交差してもよい。条件は 3 つ:
///
/// 1. 各ジョブから出る辺は、記述順の次の節への辺を持ち、それ以外の辺(飛び越し)は、多くて 1 つ。
///    ただし、次の節への辺を要らないジョブが 2 種類ある。
///    - 「両方の飛び越し」: 辺が 1 つだけで、種類が成功と警告の両方で、行き先が次の節でない(BR2.1)。
///      その辺は、通常の飛び越しと同じ線で描く。
///    - 「分岐」: 辺が 2 本で、どちらも次の節でなく、行き先が異なる(BR1.1)。2 本の線を同時に開く。
/// 2. 飛び越しの区間が交差しない(元 1 < 元 2 < 行き先 1 < 行き先 2 になる組がない)。
///    行き先が同じ辺は、交差ではなく、合流になる(BR1.1)。
/// 3. 分岐は、積み上げが空か、遠いほうの行き先が最も内側の線の行き先より近いときだけ載せる(BR2.1)。
///    分岐が開いた線に、後のジョブが同じ行き先で合流する形は、辺の一覧にする(BR2.2)。
///
/// 条件 2 と 3 は、記述順に節を見ながら、開いている飛び越しの行き先を積んだ列(積み上げ)で、
/// 1 回の走査で確かめる。合流する辺は、積み上げに載せない。
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

    // 条件 1: 元のジョブごとに、次の節への辺と、飛び越しの辺を分ける。
    let mut has_next = vec![false; job_count];
    let mut targets: Vec<Vec<usize>> = vec![Vec::new(); job_count];
    let mut edge_count = vec![0usize; job_count];
    let mut both_skip = vec![false; job_count];
    for edge in &graph.edges {
        let from = *position.get(edge.from_job.as_str())?;
        let to = to_position(&edge.to)?;
        if from >= job_count || to <= from {
            return None;
        }
        edge_count[from] += 1;
        if to == from + 1 {
            has_next[from] = true;
        } else {
            if edge.kinds.len() == 2 {
                both_skip[from] = true;
            }
            targets[from].push(to);
        }
    }
    let mut off_main = vec![false; job_count];
    for i in 0..job_count {
        targets[i].sort_unstable();
        match targets[i].len() {
            0 => {
                if !has_next[i] {
                    return None;
                }
            }
            1 => {
                if both_skip[i] && edge_count[i] == 1 {
                    // 「両方の飛び越し」: 次の節への辺は要らない(通常の飛び越しと同じ線で描く)。
                    off_main[i] = true;
                } else if !has_next[i] {
                    return None;
                }
            }
            2 => {
                // 「分岐」: 次の節への辺がなく、辺は 2 本だけ(行き先は、辺がまとまるので異なる)。
                if edge_count[i] != 2 || targets[i][0] == targets[i][1] {
                    return None;
                }
                off_main[i] = true;
            }
            _ => return None,
        }
    }

    // 交差を許す範囲(BR1.1): グラフ全体で、すべてのジョブが次の節への辺を持ち、飛び越しは高々 1 本、
    // 「両方の飛び越し」と分岐がなく、飛び越しの行き先が、終点も含めて、どの 2 本でも違う。
    if is_crossing_class(&targets, &has_next, &off_main) {
        return Some(SkipPlan { targets, off_main });
    }

    // 条件 2、3: 開いている行き先の積み上げ(上が最も内側)。
    let mut open: Vec<OpenLane> = Vec::new();
    for (i, ts) in targets.iter().enumerate() {
        if open.last().is_some_and(|l| l.target == i) {
            open.pop();
        }
        // 最も内側でない線が、ここで終わるなら、交差している。
        if open.iter().any(|l| l.target == i) {
            return None;
        }
        match ts.as_slice() {
            [] => {}
            &[t] => match open.last() {
                Some(top) if t > top.target => return None,
                // 最も内側の線と行き先が同じなら、合流する(積み上げは変えない)。
                // 分岐が開いた線への合流は、辺の一覧にする(BR2.2)。
                Some(top) if t == top.target => {
                    if top.by_branch {
                        return None;
                    }
                }
                _ => open.push(OpenLane {
                    target: t,
                    by_branch: false,
                }),
            },
            &[t1, t2] => {
                // 積み上げが空でなければ、t2 が最も内側の線より近いときだけ載せる(BR2.1)。
                if open.last().is_some_and(|top| t2 >= top.target) {
                    return None;
                }
                for target in [t2, t1] {
                    open.push(OpenLane {
                        target,
                        by_branch: true,
                    });
                }
            }
            _ => return None,
        }
    }
    Some(SkipPlan { targets, off_main })
}

/// 交差を許す範囲か(BR1.1)。すべてのジョブが次の節への辺を持ち、飛び越しは高々 1 本で、
/// 主線につながらないジョブ(「両方の飛び越し」、分岐)がなく、行き先が(終点も含めて)すべて違う。
fn is_crossing_class(targets: &[Vec<usize>], has_next: &[bool], off_main: &[bool]) -> bool {
    if !has_next.iter().all(|&h| h) || off_main.iter().any(|&o| o) {
        return false;
    }
    if targets.iter().any(|ts| ts.len() > 1) {
        return false;
    }
    let mut seen = std::collections::HashSet::new();
    targets.iter().flatten().all(|t| seen.insert(*t))
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
        Some(plan) => GraphLayout::Lanes(lane_layout(&plan)),
        None => GraphLayout::EdgeList,
    }
}

/// 線をつなぐ行(合流): 主線の線と、閉じる線(列 1)の合流、残りの `remaining` 本の左へのずれ。
/// `off_main` なら、列 0 を空にする(その線を開いたジョブが、すべて主線につながらない。BR1.4)。
fn merge_row(remaining: usize, off_main: bool) -> Vec<LaneGlyph> {
    let first = if off_main {
        LaneGlyph::Empty
    } else {
        LaneGlyph::Line
    };
    let mut glyphs = vec![first, LaneGlyph::Join];
    for _ in 0..remaining {
        glyphs.extend([LaneGlyph::Empty, LaneGlyph::Join]);
    }
    glyphs
}

/// 線をつなぐ行(分岐): 新しい線 `new` 本(通常は 1、分岐は 2)を列 1 から入れ、
/// すでにある `existing` 本が右へずれる(BR1.3)。
fn fork_row(existing: usize, new: usize, off_main: bool) -> Vec<LaneGlyph> {
    let first = if off_main {
        LaneGlyph::Empty
    } else {
        LaneGlyph::Line
    };
    let mut glyphs = vec![first, LaneGlyph::Fork];
    for _ in 0..(existing + new - 1) {
        glyphs.extend([LaneGlyph::Empty, LaneGlyph::Fork]);
    }
    glyphs
}

/// 交差の行(BR1.4)。開いている線が `m` 本、到着する線が内側から `k` 番目(`k >= 1`)のときの、行 A と行 B。
fn crossing_rows(m: usize, k: usize) -> (Vec<LaneGlyph>, Vec<LaneGlyph>) {
    let mut a = vec![LaneGlyph::Line, LaneGlyph::Empty];
    for j in 0..k {
        a.push(LaneGlyph::Line);
        a.push(if j == k - 1 {
            LaneGlyph::Join
        } else {
            LaneGlyph::Underscore
        });
    }
    for _ in 0..(m - 1 - k) {
        a.extend([LaneGlyph::Empty, LaneGlyph::Join]);
    }
    let mut b = vec![LaneGlyph::Line, LaneGlyph::Join, LaneGlyph::Line];
    for _ in 0..(m - 2) {
        b.extend([LaneGlyph::Empty, LaneGlyph::Line]);
    }
    (a, b)
}

/// 線をつなぐ行(同じ行き先への合流): 既存の最も内側の線(列 1)に重なる。外側の `outer` 本は、そのまま。
fn join_lane_row(outer: usize, both: bool) -> Vec<LaneGlyph> {
    let first = if both {
        LaneGlyph::Empty
    } else {
        LaneGlyph::Line
    };
    let mut glyphs = vec![first, LaneGlyph::Fork, LaneGlyph::Line];
    for _ in 0..outer {
        glyphs.extend([LaneGlyph::Empty, LaneGlyph::Line]);
    }
    glyphs
}

/// レーンの配置(BR1.1〜BR1.5、BR2.1〜BR2.3、BR2.5)。
///
/// 節ごとに、次の順で行を作る。
///
/// 1. その節が飛び越しの行き先なら、その線の位置 `k`(内側から数える)で、`k = 0` は合流の行(列 1 の線が主線へ
///    合流し、残りは左へ詰まる)、`k >= 1` は交差の行(行 A、行 B の 2 行。BR1.4)。その線は、位置 `k` から取り除く。
/// 2. ノードの行。すでにある右の線は、この節が新しい線(NewLane、分岐)を持ち、かつ直前の行が分岐の行なら `\`、
///    この節が合流の行き先(直前が `k = 0` の合流の行)で、次の節も合流の行き先(位置 `q`)なら、`q` 以降を `/`、
///    それ以外は `|`(BR1.5)。
///    ラベルの位置は、2 × (1 + 開いている線の数 + 新しい線の数(NewLane なら 1、分岐なら 2))。
/// 3. 新しい線なら分岐の行。最も内側の線と行き先が同じ飛び越し(JoinLane)なら、合流の行 `|\|`
///    (線を増やさない)。飛び越しなし(NoLane)は、行を足さない。分岐は、線を 2 本(外側が遠い行き先、
///    内側が近い行き先)同時に開く。
///    「両方の飛び越し」は、通常の飛び越しと同じ扱い(NewLane か JoinLane)。
fn lane_layout(plan: &SkipPlan) -> LaneLayout {
    let job_count = plan.targets.len();
    let mut rows: Vec<LaneRow> = Vec::with_capacity(job_count * 3 + 2);
    // 開いている飛び越しの行き先。最後が列 1(最も内側)。
    let mut open: Vec<usize> = Vec::new();
    // 開いている線ごとに、その線を開いたジョブがすべて主線につながらないか(その線の主線側の `|` を描かない)。
    let mut open_all_off_main: Vec<bool> = Vec::new();
    // 直前の行が分岐の行か(分岐の行の直後のノードの行では、ずれる線を `\` で描く)。
    let mut after_fork = false;
    for i in 0..=job_count {
        // この節に到着する線(最後が最も内側。内側から数えた位置 `k` は `open.len() - 1 - 添字`)。
        let arrival = open.iter().position(|&t| t == i);
        let mut arrived_innermost = false;
        if let Some(index) = arrival {
            let k = open.len() - 1 - index;
            if k == 0 {
                rows.push(LaneRow {
                    glyphs: merge_row(open.len() - 1, open_all_off_main[index]),
                    node_index: None,
                });
                arrived_innermost = true;
            } else {
                let (row_a, row_b) = crossing_rows(open.len(), k);
                rows.push(LaneRow {
                    glyphs: row_a,
                    node_index: None,
                });
                rows.push(LaneRow {
                    glyphs: row_b,
                    node_index: None,
                });
            }
            open.remove(index);
            open_all_off_main.remove(index);
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
        let targets = &plan.targets[i];
        let joins = targets.len() == 1 && open.last() == Some(&targets[0]);
        let new_lanes = if joins { 0 } else { targets.len() };
        let new_lane = new_lanes > 0;
        // 次の節も線の行き先なら、その線(内側から数えて `q` 番目)以降の線は `/`(BR1.5 の (b))。
        let join_from = if !new_lane && arrived_innermost {
            open.iter()
                .position(|&t| t == i + 1)
                .map(|p| open.len() - 1 - p)
        } else {
            None
        };
        let existing_glyph = |j: usize| {
            if new_lane {
                if after_fork {
                    LaneGlyph::Fork
                } else {
                    LaneGlyph::Line
                }
            } else if join_from.is_some_and(|q| j >= q) {
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
        let off_main = plan.off_main[i];
        if joins {
            if let Some(last) = open_all_off_main.last_mut() {
                *last &= off_main;
            }
            rows.push(LaneRow {
                glyphs: join_lane_row(existing - 1, off_main),
                node_index: None,
            });
        } else if new_lane {
            rows.push(LaneRow {
                glyphs: fork_row(existing, new_lanes, off_main),
                node_index: None,
            });
            // 行き先の昇順の並びを逆順に積む(外側が遠い行き先、内側が近い行き先)。
            for &t in targets.iter().rev() {
                open.push(t);
                open_all_off_main.push(off_main);
            }
        }
        after_fork = new_lane;
    }
    LaneLayout { rows }
}
