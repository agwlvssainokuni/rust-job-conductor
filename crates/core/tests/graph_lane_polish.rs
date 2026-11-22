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

//! 「両方の飛び越し」(BR1.3、BR1.4)の図の例 D1〜D7 と、交差・同じ行き先との組み合わせ(BR2.1)、
//! 辺の一覧に残る形(BR1.1)のテスト。

mod graph_common;

use conductor_core::TransitionOn::{Success, Warning};
use conductor_core::graph::{
    EdgeKind, EdgeTarget, FlowEdge, FlowGraph, GraphLayout, GraphNode, is_lane_drawable,
    layout_graph,
};
use graph_common::{Spec, definition, graph_with_extra_edge, lanes_text, letters};

/// `n` 個のジョブ(`a`、`b`、…)の図。辺の一覧になるときは `None`。
fn lanes(n: usize, transitions: &[Spec<'_>]) -> Option<String> {
    let ids = letters(n);
    let ids: Vec<&str> = ids.iter().map(String::as_str).collect();
    lanes_text(&definition(&ids, transitions))
}

/// `from` が、成功でも警告でも `to` へ飛ぶ遷移の組。
fn both<'a>(from: &'a str, to: Option<&'a str>) -> [Spec<'a>; 2] {
    [(from, Success, to), (from, Warning, to)]
}

/// 「両方の飛び越し」の図を、通常の飛び越しの図と比べるために、飛び越される間の主線の `|` が
/// 空になる行(` \\`、` /`、` \\|`)の先頭を `|` に戻す。
fn with_main_line(text: &str) -> String {
    text.lines()
        .map(|line| {
            if line.starts_with(" \\") || line.starts_with(" /") {
                format!("|{}\n", &line[1..])
            } else {
                format!("{line}\n")
            }
        })
        .collect()
}

#[test]
fn d1_両方の飛び越しは_通常の飛び越しと同じ線で描き_主線の線だけ省く() {
    assert_eq!(
        lanes(4, &both("a", Some("c"))).unwrap(),
        "*   a\n \\\n* | b\n /\n* c\n* d\no @end\n"
    );
}

#[test]
fn d2_別の線の内側の両方の飛び越しは_通常の飛び越しと同じ線で描く() {
    let mut transitions = vec![("a", Success, Some("f"))];
    transitions.extend(both("b", Some("d")));
    assert_eq!(
        lanes(6, &transitions).unwrap(),
        "*   a\n|\\\n* \\   b\n \\ \\\n* | | c\n / /\n* | d\n* | e\n|/\n* f\no @end\n"
    );
}

#[test]
fn d3_別の線の行き先を越える両方の飛び越しは_交差の行で描く() {
    // b は実行されうるが、次のジョブへの辺がないので、b の下の行の列 0 は空白(BR1.4)。
    // c、d は実行されないので、e の手前の交差の行の列 0 も空白。
    let mut transitions = vec![("a", Success, Some("e"))];
    transitions.extend(both("b", Some("g")));
    assert_eq!(
        lanes(7, &transitions).unwrap(),
        "*   a\n|\\\n* \\   b\n \\ \\\n* | | c\n* | | d\n  |/\n /|\n* | e\n* | f\n|/\n* g\no @end\n"
    );
}

#[test]
fn d4_終点への両方の飛び越しは_終点へ描く() {
    assert_eq!(
        lanes(3, &both("a", None)).unwrap(),
        "*   a\n \\\n* | b\n* | c\n /\no @end\n"
    );
}

#[test]
fn d5_飛び越し先のジョブが_両方の飛び越しを持つ() {
    let mut transitions = vec![("a", Success, Some("c"))];
    transitions.extend(both("c", Some("e")));
    assert_eq!(
        lanes(5, &transitions).unwrap(),
        "*   a\n|\\\n* | b\n|/\n*   c\n \\\n* | d\n /\n* e\no @end\n"
    );
}

#[test]
fn d6_同じ行き先への合流は_両方の飛び越しでも合流の線になる() {
    // a、b は通常の飛び越しで e へ。c は両方の飛び越しで e へ合流する。d は実行されない(c は d へ進まない)ので、
    // e の手前の合流の行の列 0 は空白(BR1.4)。
    let mut transitions = vec![("a", Success, Some("e")), ("b", Success, Some("e"))];
    transitions.extend(both("c", Some("e")));
    assert_eq!(
        lanes(6, &transitions).unwrap(),
        "*   a\n|\\\n* | b\n|\\|\n* | c\n \\|\n* | d\n /\n* e\n* f\no @end\n"
    );
}

#[test]
fn d7_両方の飛び越しだけが合流する線は_合流の行の主線も空() {
    let mut transitions = both("a", Some("e")).to_vec();
    transitions.extend(both("b", Some("e")));
    assert_eq!(
        lanes(5, &transitions).unwrap(),
        "*   a\n \\\n* | b\n \\|\n* | c\n* | d\n /\n* e\no @end\n"
    );
}

fn ordinary(n: usize, skips: &[(&str, &str)]) -> String {
    let transitions: Vec<Spec<'_>> = skips.iter().map(|(f, t)| (*f, Success, Some(*t))).collect();
    lanes(n, &transitions).unwrap()
}

fn both_skips(n: usize, skips: &[(&str, &str)]) -> String {
    let transitions: Vec<Spec<'_>> = skips.iter().flat_map(|(f, t)| both(f, Some(t))).collect();
    lanes(n, &transitions).unwrap()
}

#[test]
fn 両方の飛び越しの図は_同じ飛び越しの通常の図と_主線の線を除いて一致する() {
    let shapes: [(usize, Vec<(&str, &str)>); 4] = [
        (4, vec![("a", "c")]),
        (6, vec![("a", "f"), ("b", "d")]),
        (5, vec![("a", "c"), ("c", "e")]),
        (6, vec![("a", "e"), ("b", "e"), ("c", "e")]),
    ];
    for (n, skips) in shapes {
        assert_eq!(
            with_main_line(&both_skips(n, &skips)),
            ordinary(n, &skips),
            "{skips:?}"
        );
    }
}

#[test]
fn 両方の飛び越しが交差する形は_通常の飛び越しが交差する形と同じくレーンの図になる() {
    let transitions: Vec<Spec<'_>> = both("a", Some("d"))
        .into_iter()
        .chain([("b", Success, Some("e"))])
        .collect();
    // b、c は実行されないので、その下の行の列 0 は空白。d は実行されうる(BR1.4)。
    assert_eq!(
        lanes(6, &transitions).unwrap(),
        "*   a\n \\\n* \\   b\n \\ \\\n* | | c\n  |/\n /|\n* | d\n|/\n* e\n* f\no @end\n"
    );
}

#[test]
fn 交差する通常の飛び越しは_レーンの図になる() {
    assert_eq!(
        lanes(5, &[("a", Success, Some("d")), ("b", Success, Some("e"))]).as_deref(),
        Some("*   a\n|\\\n* \\   b\n|\\ \\\n* | | c\n| |/\n|/|\n* | d\n|/\n* e\no @end\n")
    );
}

#[test]
fn 辺の一覧に残る形_両方の飛び越しのジョブに辺を足して3本にする() {
    // 両方の飛び越し(a が c へ)に、次のジョブ(b)と @end への辺を足す。辺が 3 本で、種類が 4 つの
    // どれでもない(BR1.1。図の構造を直接作る)。
    let ids = letters(4);
    let ids: Vec<&str> = ids.iter().map(String::as_str).collect();
    let mut graph = graph_with_extra_edge(&definition(&ids, &both("a", Some("c"))), "a", None);
    let extra = graph_with_extra_edge(&definition(&ids, &[]), "a", Some("b"));
    graph
        .edges
        .push(extra.edges.last().cloned().expect("足した辺"));
    assert!(!is_lane_drawable(&graph));
    assert_eq!(layout_graph(&graph), GraphLayout::EdgeList);
}

#[test]
fn 同じ行き先を含んで交差する形は_レーンの図になる() {
    // a→e と c→e は同じ行き先で、b→f が a→e の線の行き先を越えて交差する。通常の飛び越しだけなので図にする(BR1.1)。
    let text = lanes(
        7,
        &[
            ("a", Success, Some("e")),
            ("b", Success, Some("f")),
            ("c", Success, Some("e")),
        ],
    )
    .expect("通常の飛び越しだけのグラフは、レーンの図になる");
    assert!(text.starts_with("*   a\n|\\\n* \\   b\n"), "{text}");
    assert!(text.ends_with("o @end\n"), "{text}");
}

#[test]
fn 交差と同じ行き先に_両方の飛び越しが混ざる形は_レーンの図になる() {
    // 交差と同じ行き先に、「両方の飛び越し」(d が成功も警告も f へ)が加わる(BR2.1)。
    // d の線は、次のノードの手前で、同じ行き先の線に寄せる。
    assert_eq!(
        lanes(
            7,
            &[
                ("a", Success, Some("e")),
                ("b", Success, Some("f")),
                ("c", Success, Some("e")),
                ("d", Success, Some("f")),
                ("d", Warning, Some("f")),
            ]
        )
        .unwrap(),
        "*   a\n|\\\n* \\   b\n|\\ \\\n* \\ \\   c\n|\\ \\ \\\n| | |/\n| |/|\n* | |   d\n \\ \\ \\\n  |/ /\n /| /\n  |/\n* / e\n|/\n* f\n* g\no @end\n"
    );
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
