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

//! 交差する飛び越しのレーンの図(BR1.1〜BR1.5、BR2.1)。
//!
//! 図の例 C1〜C3(`git log --graph` の出力と一致を確かめた形)と、辺の一覧に残る混ざった形、性能。

mod graph_common;

use std::time::{Duration, Instant};

use conductor_core::TransitionOn::{Success, Warning};
use conductor_core::graph::{GraphLayout, build_graph, layout_graph};
use graph_common::{Spec, definition, lanes_text, letters};

fn lanes(n: usize, transitions: &[Spec<'_>]) -> Option<String> {
    let ids = letters(n);
    let ids: Vec<&str> = ids.iter().map(String::as_str).collect();
    lanes_text(&definition(&ids, transitions))
}

#[test]
fn c1_a_が_d_へ_b_が_e_へ飛ぶ交差は_行_a_と行_b_で描く() {
    let text = lanes(5, &[("a", Success, Some("d")), ("b", Success, Some("e"))]);
    assert_eq!(
        text.as_deref(),
        Some(
            "\
*   a
|\\
* \\   b
|\\ \\
* | | c
| |/
|/|
* | d
|/
* e
o @end
"
        )
    );
}

#[test]
fn c2_3_本の線が続けて交差すると_行_a_に横線を使う() {
    let text = lanes(
        7,
        &[
            ("a", Success, Some("d")),
            ("b", Success, Some("e")),
            ("c", Success, Some("f")),
        ],
    );
    assert_eq!(
        text.as_deref(),
        Some(
            "\
*   a
|\\
* \\   b
|\\ \\
* \\ \\   c
|\\ \\ \\
| |_|/
|/| |
* | | d
| |/
|/|
* | e
|/
* f
* g
o @end
"
        )
    );
}

#[test]
fn c3_終点への飛び越しを含む交差は_次のノードが到着する線の位置から_斜線になる() {
    let text = lanes(
        6,
        &[
            ("a", Success, Some("f")),
            ("b", Success, None),
            ("c", Success, Some("e")),
        ],
    );
    assert_eq!(
        text.as_deref(),
        Some(
            "\
*   a
|\\
* \\   b
|\\ \\
* \\ \\   c
|\\ \\ \\
* | | | d
|/ / /
* | / e
| |/
|/|
* | f
|/
o @end
"
        )
    );
}

#[test]
fn 交差を含まない図は_交差の行を持たない() {
    // 入れ子の飛び越しは、これまでどおり(行 A、行 B は出ない)。
    let text = lanes(5, &[("a", Success, Some("e")), ("b", Success, Some("d"))]).unwrap();
    assert!(!text.contains('_'));
    assert!(!text.contains("|/|"));
}

#[test]
fn 辺の一覧に残る形_交差と離れた別のジョブの両方の飛び越し() {
    let mut transitions: Vec<Spec<'_>> = vec![("a", Success, Some("d")), ("b", Success, Some("e"))];
    transitions.extend([("g", Success, Some("i")), ("g", Warning, Some("i"))]);
    assert_eq!(lanes(10, &transitions), None);
}

#[test]
fn 辺の一覧に残る形_交差と離れた別のジョブの分岐() {
    let transitions: Vec<Spec<'_>> = vec![
        ("a", Success, Some("d")),
        ("b", Success, Some("e")),
        ("f", Success, Some("h")),
        ("f", Warning, None),
    ];
    assert_eq!(lanes(9, &transitions), None);
}

#[test]
fn 辺の一覧に残る形_交差と離れた別のジョブの同じ行き先() {
    // g→i と h→i が同じ行き先(合流)。
    let transitions: Vec<Spec<'_>> = vec![
        ("a", Success, Some("d")),
        ("b", Success, Some("e")),
        ("f", Success, Some("i")),
        ("g", Success, Some("i")),
    ];
    assert_eq!(lanes(10, &transitions), None);
}

#[test]
fn 辺の一覧に残る形_次のジョブへの辺を持たないジョブがある交差() {
    // c は、成功も警告も終点へ飛ぶ(次のジョブへの辺がない)。
    let transitions: Vec<Spec<'_>> = vec![
        ("a", Success, Some("d")),
        ("b", Success, Some("e")),
        ("c", Success, None),
        ("c", Warning, None),
    ];
    assert_eq!(lanes(5, &transitions), None);
}

#[test]
fn 幅の広い交差の最悪の形は_1000_ジョブでも_1秒以内() {
    const N: usize = 1000;
    let ids: Vec<String> = (0..N).map(|i| format!("j{i}")).collect();
    let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
    // 前半の各ジョブ i が j(i + N/2) へ飛ぶ。500 本の線が同時に開き、すべてが交差する。
    let transitions: Vec<_> = (0..N / 2)
        .map(|i| (refs[i], Success, Some(refs[i + N / 2])))
        .collect();
    let start = Instant::now();
    let layout = layout_graph(&build_graph(&definition(&refs, &transitions), None));
    let elapsed = start.elapsed();
    assert!(matches!(layout, GraphLayout::Lanes(_)));
    assert!(elapsed < Duration::from_secs(1), "{elapsed:?}");
}
