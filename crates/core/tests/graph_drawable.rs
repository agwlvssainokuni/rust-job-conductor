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

//! 「対応する形」の判定(BR2.4)のテスト。

mod graph_common;

use conductor_core::TransitionOn::{Success, Warning};
use conductor_core::graph::{GraphLayout, build_graph, is_lane_drawable, layout_graph};
use graph_common::{Spec, definition, lanes_text};

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
fn 対応しない形_両方の飛び越しが交差する() {
    // a の両方が d へ、b の成功が e へ。通常の飛び越しと同じく交差する。
    let crossing = [
        ("a", Success, Some("d")),
        ("a", Warning, Some("d")),
        ("b", Success, Some("e")),
    ];
    assert!(!drawable(&IDS, &crossing));
    assert_eq!(
        layout_graph(&build_graph(&definition(&IDS, &crossing), None)),
        GraphLayout::EdgeList
    );
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
fn 対応しない形_交差に同じ行き先が混ざる() {
    // a→e、b→f は交差し、c→e が同じ行き先なので、範囲の外(辺の一覧。BR2.1)。
    let ids = ["a", "b", "c", "d", "e", "f", "g"];
    let transitions = [
        ("a", Success, Some("e")),
        ("b", Success, Some("f")),
        ("c", Success, Some("e")),
    ];
    assert!(!drawable(&ids, &transitions));
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
fn 対応しない形_分岐が積み上げの上の線より遠い() {
    // b→d の線が開いている間に、c が e と f へ分岐する。t2 が最も内側の線以上なので、交差する(BR2.1)。
    assert!(!drawable(
        &IDS,
        &[
            ("b", Success, Some("d")),
            ("c", Success, Some("e")),
            ("c", Warning, Some("f"))
        ]
    ));
}

#[test]
fn 対応しない形_分岐の線を別の飛び越しが越える() {
    // a→c,e の分岐と、b→e。b の飛び越しは、分岐の線(内側の c)を越えるので、交差する。
    assert!(!drawable(
        &IDS,
        &[
            ("a", Success, Some("c")),
            ("a", Warning, Some("e")),
            ("b", Success, Some("e"))
        ]
    ));
}
