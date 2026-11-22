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

//! 分岐(成功と警告が、異なる、どちらも次でない行き先へ進む形)の図の例 B1〜B4(BR1.3、BR1.4)と、
//! 分岐が同じ行き先・交差と組み合わさる形(BR2.1)、辺の一覧に残る形(BR1.1)のテスト。

mod graph_common;

use conductor_core::TransitionOn::{Success, Warning};
use conductor_core::graph::{GraphLayout, layout_graph};
use graph_common::{Spec, definition, graph_with_extra_edge, lanes_text, letters};

/// `n` 個のジョブ(`a`、`b`、…)の図。辺の一覧になるときは `None`。
fn lanes(n: usize, transitions: &[Spec<'_>]) -> Option<String> {
    let ids = letters(n);
    let ids: Vec<&str> = ids.iter().map(String::as_str).collect();
    lanes_text(&definition(&ids, transitions))
}

#[test]
fn b1_分岐は_2本の線を同時に開き_実行されうるジョブの下だけ主線を描く() {
    // b は実行されないので、b の下の行の列 0 は空白。c は実行されうるので、c の下の合流の行は `|/`(BR1.4)。
    assert_eq!(
        lanes(5, &[("a", Success, Some("c")), ("a", Warning, Some("d"))]).unwrap(),
        "*     a\n \\ \\\n* | | b\n / /\n* / c\n|/\n* d\n* e\no @end\n"
    );
}

#[test]
fn b2_終点への分岐() {
    assert_eq!(
        lanes(4, &[("a", Success, Some("c")), ("a", Warning, None)]).unwrap(),
        "*     a\n \\ \\\n* | | b\n / /\n* | c\n* | d\n|/\no @end\n"
    );
}

#[test]
fn b3_外側に通常の飛び越しがある分岐() {
    assert_eq!(
        lanes(
            7,
            &[
                ("a", Success, Some("g")),
                ("b", Success, Some("d")),
                ("b", Warning, Some("e")),
            ]
        )
        .unwrap(),
        "*   a\n|\\\n* \\     b\n \\ \\ \\\n* | | | c\n / / /\n* / / d\n|/ /\n* | e\n* | f\n|/\n* g\no @end\n"
    );
}

#[test]
fn b4_線が離れる分岐() {
    assert_eq!(
        lanes(6, &[("a", Success, Some("d")), ("a", Warning, Some("f"))]).unwrap(),
        "*     a\n \\ \\\n* | | b\n* | | c\n / /\n* | d\n* | e\n|/\n* f\no @end\n"
    );
}

#[test]
fn 成功と警告のどちらが近くても_同じ図になる() {
    assert_eq!(
        lanes(5, &[("a", Success, Some("d")), ("a", Warning, Some("c"))]),
        lanes(5, &[("a", Success, Some("c")), ("a", Warning, Some("d"))])
    );
}

#[test]
fn 分岐の線へ後のジョブが同じ行き先で合流する形は_合流の行で描く() {
    // a は c と f へ分岐し、d が f へ飛ぶ。d の線は、分岐の外側の線(f)に合流の行 `|\|` で重なる(BR2.1)。
    assert_eq!(
        lanes(
            7,
            &[
                ("a", Success, Some("c")),
                ("a", Warning, Some("f")),
                ("d", Success, Some("f")),
            ]
        )
        .unwrap(),
        "*     a\n \\ \\\n* | | b\n / /\n* | c\n* | d\n|\\|\n* | e\n|/\n* f\n* g\no @end\n"
    );
}

#[test]
fn 分岐の遠い行き先が最も内側の線より遠い形は_交差の行で描く() {
    // b→d の線が開いている間に、c が e と f へ分岐する(BR2.1)。
    assert_eq!(
        lanes(
            6,
            &[
                ("b", Success, Some("d")),
                ("c", Success, Some("e")),
                ("c", Warning, Some("f")),
            ]
        )
        .unwrap(),
        "* a\n*   b\n|\\\n* \\     c\n \\ \\ \\\n  |_|/\n /| |\n* | | d\n|/ /\n* / e\n|/\n* f\no @end\n"
    );
    // 2 つ目の分岐の近い行き先が、1 つ目の分岐の線と同じ。重複の線は、次のノードの手前で寄せる(BR1.3)。
    assert_eq!(
        lanes(
            7,
            &[
                ("a", Success, Some("d")),
                ("a", Warning, Some("f")),
                ("b", Success, Some("d")),
                ("b", Warning, Some("e")),
            ]
        )
        .unwrap(),
        "*     a\n \\ \\\n* \\ \\     b\n \\ \\ \\ \\\n  | |/ /\n  |/| |\n* / | | c\n / / /\n* / / d\n|/ /\n* / e\n|/\n* f\n* g\no @end\n"
    );
}

#[test]
fn 分岐と交差する飛び越しは_交差の行で描く() {
    // a→d、b→e は交差し、離れた f が分岐する(BR2.1)。
    assert_eq!(
        lanes(
            9,
            &[
                ("a", Success, Some("d")),
                ("b", Success, Some("e")),
                ("f", Success, Some("h")),
                ("f", Warning, None),
            ]
        )
        .unwrap(),
        "*   a\n|\\\n* \\   b\n|\\ \\\n* | | c\n| |/\n|/|\n* | d\n|/\n* e\n*     f\n \\ \\\n* | | g\n / /\n* | h\n* | i\n|/\no @end\n"
    );
    // 分岐の線の間に、別の飛び越しの線が割り込む。
    assert_eq!(
        lanes(
            7,
            &[
                ("a", Success, Some("c")),
                ("a", Warning, Some("f")),
                ("b", Success, Some("e")),
            ]
        )
        .unwrap(),
        "*     a\n \\ \\\n* \\ \\   b\n \\ \\ \\\n  |/ /\n /| |\n* | | c\n* | | d\n|/ /\n* / e\n|/\n* f\n* g\no @end\n"
    );
}

#[test]
fn 辺の一覧に残る形_辺が3本のジョブがある() {
    // 分岐のジョブ a に、3 本目の辺(@end へ)を足す。定義からは作れない、範囲の外の形(BR1.1)。
    let ids = letters(5);
    let ids: Vec<&str> = ids.iter().map(String::as_str).collect();
    let def = definition(
        &ids,
        &[("a", Success, Some("c")), ("a", Warning, Some("d"))],
    );
    let graph = graph_with_extra_edge(&def, "a", None);
    assert_eq!(layout_graph(&graph), GraphLayout::EdgeList);
}
