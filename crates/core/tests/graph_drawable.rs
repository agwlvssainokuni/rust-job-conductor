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

//! 「対応する形」の判定(BR1.1)と、辺の一覧に残る形、主線の列 0 の記号(BR1.4)のテスト。

mod graph_common;

use conductor_core::TransitionOn::{Success, Warning};
use conductor_core::graph::{
    EdgeKind, EdgeTarget, FlowEdge, FlowGraph, GraphLayout, GraphNode, LaneGlyph, build_graph,
    is_lane_drawable, layout_graph,
};
use graph_common::{Spec, both, definition, graph_with_extra_edge, lanes_text, letter_definition};

fn drawable(ids: &[&str], transitions: &[Spec<'_>]) -> bool {
    is_lane_drawable(&build_graph(&definition(ids, transitions), None))
}

const IDS: [&str; 6] = ["a", "b", "c", "d", "e", "f"];

#[test]
fn 対応する形_飛び越しなしとジョブ1個() {
    assert!(drawable(&IDS, &[]));
    assert!(drawable(&["only"], &[]));
    assert_eq!(
        lanes_text(&definition(&["only"], &[])).unwrap(),
        "* only\no @end\n"
    );
}

#[test]
fn 対応する形_飛び越しが1つと_終点への飛び越し() {
    assert!(drawable(&IDS, &[("a", Success, Some("d"))]));
    assert!(drawable(&IDS, &[("a", Success, None)]));
    // 警告だけが飛び越す形も同じ。
    assert!(drawable(&IDS, &[("b", Warning, Some("e"))]));
}

#[test]
fn 対応する形_入れ子() {
    assert!(drawable(
        &IDS,
        &[("a", Success, Some("f")), ("b", Success, Some("e"))]
    ));
}

#[test]
fn 対応する形_並びと_元が別の行き先になる連続() {
    assert!(drawable(
        &IDS,
        &[("a", Success, Some("c")), ("d", Success, Some("f"))]
    ));
    // 行き先のジョブから、次の飛び越しが始まる。
    assert!(drawable(
        &IDS,
        &[("a", Success, Some("c")), ("c", Success, Some("e"))]
    ));
}

#[test]
fn 対応する形_同じ行き先と終点の共有() {
    // a→d、b→d は、既存の線に合流する。
    assert!(drawable(
        &IDS,
        &[("a", Success, Some("d")), ("b", Success, Some("d"))]
    ));
    // 終点を共有する形(a→@end、b→@end)。
    assert!(drawable(
        &IDS,
        &[("a", Success, None), ("b", Success, None)]
    ));
}

#[test]
fn 対応する形_成功と警告の両方が飛び越す() {
    // 両方が d へ。通常の飛び越しと同じ線で描ける。
    assert!(drawable(
        &IDS,
        &[("a", Success, Some("d")), ("a", Warning, Some("d"))]
    ));
}

#[test]
fn 対応する形_両方の飛び越しが交差する() {
    // a の両方が d へ、b の成功が e へ。通常の飛び越しの交差と同じく、レーンの図で描く(BR2.1)。
    let crossing = [
        ("a", Success, Some("d")),
        ("a", Warning, Some("d")),
        ("b", Success, Some("e")),
    ];
    assert!(drawable(&IDS, &crossing));
    assert!(matches!(
        layout_graph(&build_graph(&definition(&IDS, &crossing), None)),
        GraphLayout::Lanes(_)
    ));
}

#[test]
fn 対応する形_交差する飛び越し() {
    // a→d、b→e は交差するが、各ジョブの飛び越しが 1 本で行き先が違うので、レーンの図で描く(BR1.1)。
    let crossing = [("a", Success, Some("d")), ("b", Success, Some("e"))];
    assert!(drawable(&IDS, &crossing));
    assert!(matches!(
        layout_graph(&build_graph(&definition(&IDS, &crossing), None)),
        GraphLayout::Lanes(_)
    ));
}

#[test]
fn 対応する形_交差に同じ行き先が混ざる() {
    // a→e、b→f は交差し、c→e が同じ行き先。通常の飛び越しだけなので、レーンの図で描く(BR1.1)。
    let ids = ["a", "b", "c", "d", "e", "f", "g"];
    let transitions = [
        ("a", Success, Some("e")),
        ("b", Success, Some("f")),
        ("c", Success, Some("e")),
    ];
    assert!(drawable(&ids, &transitions));
}

#[test]
fn 対応する形_交差に同じ行き先が混ざり_両方の飛び越しがある() {
    // 同じ形に、「両方の飛び越し」(d が成功も警告も f へ)が加わっても、図にする(BR2.1)。
    let ids = ["a", "b", "c", "d", "e", "f", "g"];
    let transitions = [
        ("a", Success, Some("e")),
        ("b", Success, Some("f")),
        ("d", Success, Some("f")),
        ("d", Warning, Some("f")),
    ];
    assert!(drawable(&ids, &transitions));
}

#[test]
fn 対応する形_異なる行き先が_2つの分岐() {
    // 成功が c、警告が e。次のジョブへの辺がなく、行き先が異なる 2 つ。2 本の線を同時に開く(BR1.1)。
    assert!(drawable(
        &IDS,
        &[("a", Success, Some("c")), ("a", Warning, Some("e"))]
    ));
    // 近い行き先が警告でも、終点への分岐でも同じ。
    assert!(drawable(
        &IDS,
        &[("a", Success, Some("e")), ("a", Warning, Some("c"))]
    ));
    assert!(drawable(
        &IDS,
        &[("a", Success, Some("c")), ("a", Warning, None)]
    ));
}

#[test]
fn 対応する形_分岐が開いている線より遠い行き先を持つ() {
    // b→d の線が開いている間に、c が e と f へ分岐する。交差するが、図にする(BR2.1)。
    assert!(drawable(
        &IDS,
        &[
            ("b", Success, Some("d")),
            ("c", Success, Some("e")),
            ("c", Warning, Some("f"))
        ]
    ));
}

#[test]
fn 対応する形_分岐の線を別の飛び越しが越える() {
    // a→c,e の分岐と、b→e。b の飛び越しは、分岐の線(内側の c)を越え、外側の線(e)と同じ行き先(BR2.1)。
    assert!(drawable(
        &IDS,
        &[
            ("a", Success, Some("c")),
            ("a", Warning, Some("e")),
            ("b", Success, Some("e"))
        ]
    ));
}

#[test]
fn 定義から作れる形は_すべて対応する形() {
    // 各ジョブの結果は成功と警告の 2 つなので、辺は 1 本か 2 本。どの組み合わせも 4 つの種類のどれか(BR1.1)。
    let targets = [Some("c"), Some("d"), Some("e"), None];
    for s in targets {
        for w in targets {
            for (s2, w2) in [(Some("d"), Some("e")), (Some("f"), None), (None, Some("e"))] {
                let mut transitions = vec![("a", Success, s), ("a", Warning, w)];
                transitions.push(("b", Success, s2));
                transitions.push(("b", Warning, w2));
                assert!(drawable(&IDS, &transitions), "{transitions:?}");
            }
        }
    }
}

#[test]
fn 対応しない形_辺が3本のジョブ() {
    // 図の構造を直接作る(定義からは作れない)。a に、次のジョブ、d、@end への 3 本の辺(BR1.1)。
    let def = definition(&IDS, &[("a", Success, Some("d"))]);
    let graph = graph_with_extra_edge(&def, "a", None);
    assert!(!is_lane_drawable(&graph));
    assert_eq!(layout_graph(&graph), GraphLayout::EdgeList);
}

#[test]
fn 対応しない形_次のジョブへの辺がなく_成功だけの辺が1本() {
    // 両方の飛び越し(a が c へ)の辺から、警告を除く(図の構造を直接作る。BR1.1)。
    let mut graph = build_graph(
        &definition(
            &IDS,
            &[("a", Success, Some("c")), ("a", Warning, Some("c"))],
        ),
        None,
    );
    graph.edges[0].kinds = vec![EdgeKind::Success];
    assert!(!is_lane_drawable(&graph));
    assert_eq!(layout_graph(&graph), GraphLayout::EdgeList);
}

#[test]
fn 辺の一覧に残る形_両方の飛び越しのジョブに辺を足して3本にする() {
    // 両方の飛び越し(a が c へ)に、次のジョブ(b)と @end への辺を足す。辺が 3 本で、種類が 4 つの
    // どれでもない(BR1.1。図の構造を直接作る)。
    let mut graph = graph_with_extra_edge(&letter_definition(4, &both("a", Some("c"))), "a", None);
    let extra = graph_with_extra_edge(&letter_definition(4, &[]), "a", Some("b"));
    graph
        .edges
        .push(extra.edges.last().cloned().expect("足した辺"));
    assert!(!is_lane_drawable(&graph));
    assert_eq!(layout_graph(&graph), GraphLayout::EdgeList);
}

#[test]
fn 辺の一覧に残る形_後ろ向きの辺() {
    // 定義の検証では退けられるが、図の構造としては、後ろ向きの辺は描けない。
    let graph = FlowGraph {
        nodes: ["a", "b", "c"]
            .iter()
            .map(|id| GraphNode {
                job_id: Some((*id).to_string()),
                is_end: false,
                state: None,
                is_resume: false,
            })
            .chain(std::iter::once(GraphNode {
                job_id: None,
                is_end: true,
                state: None,
                is_resume: false,
            }))
            .collect(),
        edges: vec![FlowEdge {
            from_job: "c".into(),
            to: EdgeTarget::Job("a".into()),
            kinds: vec![EdgeKind::Success, EdgeKind::Warning],
            skipped_jobs: Vec::new(),
            explicit: true,
        }],
    };
    assert!(!is_lane_drawable(&graph));
    assert_eq!(layout_graph(&graph), GraphLayout::EdgeList);
}

#[test]
fn 辺の一覧に残る形_辺が3本のジョブがある() {
    // 分岐のジョブ a に、3 本目の辺(@end へ)を足す。定義からは作れない、範囲の外の形(BR1.1)。
    let def = letter_definition(5, &[("a", Success, Some("c")), ("a", Warning, Some("d"))]);
    let graph = graph_with_extra_edge(&def, "a", None);
    assert_eq!(layout_graph(&graph), GraphLayout::EdgeList);
}

#[test]
fn 主線の列0の記号は_実行されうる_かつ次のジョブへの辺を持つジョブだけ線() {
    // N1 の形(「両方の飛び越し」a が d へと、通常の飛び越し b の成功が e へが交差。graph_shape_both.rs)。
    let mut transitions = both("a", Some("d")).to_vec();
    transitions.push(("b", Success, Some("e")));
    let GraphLayout::Lanes(layout) =
        layout_graph(&build_graph(&letter_definition(6, &transitions), None))
    else {
        panic!("N1 は、レーンの図になる");
    };
    // a は次のジョブへの辺がない。b、c は実行されない。d、e、f は線。終点は空白。
    use LaneGlyph::{Empty, Line};
    assert_eq!(
        layout.main_line,
        vec![Empty, Empty, Empty, Line, Line, Line, Empty]
    );
}

#[test]
fn 通常の飛び越しだけのグラフでは_主線の列0は終点を除いて常に線() {
    let def = letter_definition(5, &[("a", Success, Some("d")), ("b", Success, Some("e"))]);
    let GraphLayout::Lanes(layout) = layout_graph(&build_graph(&def, None)) else {
        panic!("通常の飛び越しは、レーンの図になる");
    };
    assert_eq!(layout.main_line.len(), 6);
    assert!(layout.main_line[..5].iter().all(|g| *g == LaneGlyph::Line));
    assert_eq!(layout.main_line[5], LaneGlyph::Empty);
}
