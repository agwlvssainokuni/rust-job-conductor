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

//! 「両方の飛び越し」(BR2.1)の図の例 D1〜D6 と、辺の一覧に残る形(BR1.5、BR3.1)のテスト。

mod graph_common;

use conductor_core::TransitionOn::{Success, Warning};
use conductor_core::graph::{
    EdgeKind, EdgeTarget, FlowEdge, FlowGraph, GraphLayout, GraphNode, is_lane_drawable,
    layout_graph,
};
use graph_common::{Spec, definition, lanes_text, letters};

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

#[test]
fn d1_両方の飛び越しは_線なしの行だけ() {
    assert_eq!(
        lanes(4, &both("a", Some("c"))).unwrap(),
        "* a\n* b\n* c\n* d\no @end\n"
    );
}

#[test]
fn d2_別の線の内側の両方の飛び越しは_線なし() {
    let mut transitions = vec![("a", Success, Some("f"))];
    transitions.extend(both("b", Some("d")));
    assert_eq!(
        lanes(6, &transitions).unwrap(),
        "*   a\n|\\\n* | b\n* | c\n* | d\n* | e\n|/\n* f\no @end\n"
    );
}

#[test]
fn d3_別の線の行き先を越える両方の飛び越しは_線なし() {
    let mut transitions = vec![("a", Success, Some("e"))];
    transitions.extend(both("b", Some("g")));
    assert_eq!(
        lanes(7, &transitions).unwrap(),
        "*   a\n|\\\n* | b\n* | c\n* | d\n|/\n* e\n* f\n* g\no @end\n"
    );
}

#[test]
fn d4_終点への両方の飛び越しは_線なし() {
    assert_eq!(
        lanes(3, &both("a", None)).unwrap(),
        "* a\n* b\n* c\no @end\n"
    );
}

#[test]
fn d5_飛び越し先のジョブが_両方の飛び越しを持つ() {
    let mut transitions = vec![("a", Success, Some("c"))];
    transitions.extend(both("c", Some("e")));
    assert_eq!(
        lanes(5, &transitions).unwrap(),
        "*   a\n|\\\n* | b\n|/\n* c\n* d\n* e\no @end\n"
    );
}

#[test]
fn d6_同じ行き先と両方の飛び越しの組み合わせ() {
    let mut transitions = vec![("a", Success, Some("e")), ("b", Success, Some("e"))];
    transitions.extend(both("c", Some("f")));
    assert_eq!(
        lanes(6, &transitions).unwrap(),
        "*   a\n|\\\n* | b\n|\\|\n* | c\n* | d\n|/\n* e\n* f\no @end\n"
    );
}

#[test]
fn 辺の一覧に残る形_交差() {
    assert_eq!(
        lanes(6, &[("a", Success, Some("d")), ("b", Success, Some("e"))]),
        None
    );
}

#[test]
fn 辺の一覧に残る形_異なる行き先が_2つ() {
    assert_eq!(
        lanes(5, &[("a", Success, Some("c")), ("a", Warning, Some("d"))]),
        None
    );
}

#[test]
fn 辺の一覧に残る形_同じ行き先を含んでも交差する() {
    // a→d と c→d は同じ行き先だが、b→e が a→d の線の行き先を越えるため、交差する。
    assert_eq!(
        lanes(
            6,
            &[
                ("a", Success, Some("d")),
                ("b", Success, Some("e")),
                ("c", Success, Some("d")),
            ]
        ),
        None
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
