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
fn 対応しない形_交差() {
    // a→d、b→e は交差する。
    let crossing = [("a", Success, Some("d")), ("b", Success, Some("e"))];
    assert!(!drawable(&IDS, &crossing));
    assert_eq!(
        layout_graph(&build_graph(&definition(&IDS, &crossing), None)),
        GraphLayout::EdgeList
    );
}

#[test]
fn 対応しない形_異なる行き先が_2つ() {
    // 成功が c、警告が e。次のジョブへの辺がなく、飛び越しが 2 つ。
    assert!(!drawable(
        &IDS,
        &[("a", Success, Some("c")), ("a", Warning, Some("e"))]
    ));
}
