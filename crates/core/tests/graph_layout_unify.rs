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

//! 1 つの仕組みの配置の図の例(functional-spec.md の §4)。N1〜N5、B1、D1 の全文(BR1.1〜BR2.1)と、
//! 主線の列 0 の記号(BR1.4)。図は、規則から導いた図で、git の出力ではない。

mod graph_common;

use conductor_core::TransitionOn::{Success, Warning};
use conductor_core::graph::{GraphLayout, LaneGlyph, build_graph, layout_graph};
use graph_common::{Spec, definition, lanes_text, letters};

fn lanes(n: usize, transitions: &[Spec<'_>]) -> Option<String> {
    let ids = letters(n);
    let ids: Vec<&str> = ids.iter().map(String::as_str).collect();
    lanes_text(&definition(&ids, transitions))
}

/// `from` が、成功でも警告でも `to` へ飛ぶ遷移の組(両方の飛び越し)。
fn both<'a>(from: &'a str, to: Option<&'a str>) -> [Spec<'a>; 2] {
    [(from, Success, to), (from, Warning, to)]
}

/// N1 の遷移: 「両方の飛び越し」`a` が `d` へと、通常の飛び越し `b` の成功が `e` へが交差。
fn n1() -> Vec<Spec<'static>> {
    let mut transitions = both("a", Some("d")).to_vec();
    transitions.push(("b", Success, Some("e")));
    transitions
}

#[test]
fn n1_両方の飛び越しと通常の飛び越しの交差() {
    assert_eq!(
        lanes(6, &n1()).as_deref(),
        Some(
            "\
*   a
 \\
* \\   b
 \\ \\
* | | c
  |/
 /|
* | d
|/
* e
* f
o @end
"
        )
    );
}

#[test]
fn n2_分岐と通常の飛び越しの交差() {
    let transitions = [
        ("a", Success, Some("c")),
        ("a", Warning, Some("e")),
        ("b", Success, Some("d")),
    ];
    assert_eq!(
        lanes(6, &transitions).as_deref(),
        Some(
            "\
*     a
 \\ \\
* \\ \\   b
 \\ \\ \\
  |/ /
 /| |
* | | c
|/ /
* / d
|/
* e
* f
o @end
"
        )
    );
}

#[test]
fn n3_両方の飛び越しと通常の飛び越しの同じ行き先() {
    let mut transitions = both("a", Some("d")).to_vec();
    transitions.push(("b", Success, Some("d")));
    assert_eq!(
        lanes(5, &transitions).as_deref(),
        Some(
            "\
*   a
 \\
* | b
 \\|
* | c
 /
* d
* e
o @end
"
        )
    );
}

#[test]
fn n4_両方の飛び越しが2つ交差する() {
    // b は実行されないので、b の線も主線の列 0 を描かない。d は a から実行されうる(BR1.4)。
    let mut transitions = both("a", Some("d")).to_vec();
    transitions.extend(both("b", Some("e")));
    assert_eq!(
        lanes(6, &transitions).as_deref(),
        Some(
            "\
*   a
 \\
* \\   b
 \\ \\
* | | c
  |/
 /|
* | d
|/
* e
* f
o @end
"
        )
    );
}

#[test]
fn n5_分岐の線に後のジョブが同じ行き先で合流する() {
    let transitions = [
        ("a", Success, Some("c")),
        ("a", Warning, Some("e")),
        ("c", Success, Some("e")),
    ];
    assert_eq!(
        lanes(6, &transitions).as_deref(),
        Some(
            "\
*     a
 \\ \\
* | | b
 / /
* | c
|\\|
* | d
|/
* e
* f
o @end
"
        )
    );
}

#[test]
fn b1_分岐の図は_実行されうるジョブの下の合流の行だけ主線を描く() {
    let transitions = [("a", Success, Some("c")), ("a", Warning, Some("d"))];
    assert_eq!(
        lanes(5, &transitions).as_deref(),
        Some(
            "\
*     a
 \\ \\
* | | b
 / /
* / c
|/
* d
* e
o @end
"
        )
    );
}

#[test]
fn d1_両方の飛び越しの図は_変わらない() {
    // b は実行されないので、b と c のあいだの列 0 は空白のまま(BR1.4)。
    assert_eq!(
        lanes(4, &both("a", Some("c"))).as_deref(),
        Some(
            "\
*   a
 \\
* | b
 /
* c
* d
o @end
"
        )
    );
}

#[test]
fn 主線の列0の記号は_実行されうる_かつ次のジョブへの辺を持つジョブだけ線() {
    let ids = letters(6);
    let ids: Vec<&str> = ids.iter().map(String::as_str).collect();
    let GraphLayout::Lanes(layout) = layout_graph(&build_graph(&definition(&ids, &n1()), None))
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
    let ids = letters(5);
    let ids: Vec<&str> = ids.iter().map(String::as_str).collect();
    let def = definition(
        &ids,
        &[("a", Success, Some("d")), ("b", Success, Some("e"))],
    );
    let GraphLayout::Lanes(layout) = layout_graph(&build_graph(&def, None)) else {
        panic!("通常の飛び越しは、レーンの図になる");
    };
    assert_eq!(layout.main_line.len(), 6);
    assert!(layout.main_line[..5].iter().all(|g| *g == LaneGlyph::Line));
    assert_eq!(layout.main_line[5], LaneGlyph::Empty);
}

#[test]
fn ジョブ1個の両方の飛び越しは_終点への線になる() {
    // a は成功も警告も @end へ(次の節が @end なので、飛び越しではない)。
    assert_eq!(lanes(1, &both("a", None)).as_deref(), Some("* a\no @end\n"));
}
