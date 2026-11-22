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

//! 通常の飛び越しだけのグラフの、交差と同じ行き先が混ざる形(BR1.1〜BR1.5、BR2.1)。
//!
//! 図の例 M2〜M4(`git log --graph` の出力と一致を確かめた形)と、辺の一覧に残る形。

mod graph_common;

use conductor_core::TransitionOn::{Success, Warning};
use graph_common::{Spec, definition, lanes_text, letters};

fn lanes(n: usize, transitions: &[Spec<'_>]) -> Option<String> {
    let ids = letters(n);
    let ids: Vec<&str> = ids.iter().map(String::as_str).collect();
    lanes_text(&definition(&ids, transitions))
}

#[test]
fn m2_離れて同じ行き先を持つ線が_別の線と交差する() {
    let text = lanes(
        5,
        &[
            ("a", Success, Some("e")),
            ("b", Success, Some("d")),
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
| |/ /
|/| /
| |/
* / d
|/
* e
o @end
"
        )
    );
}

#[test]
fn m3_隣り合って同じ行き先を持つ線が_別の線と交差する() {
    let text = lanes(
        5,
        &[
            ("a", Success, Some("d")),
            ("b", Success, Some("e")),
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
* | | c
|\\| |
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
fn m4_3_本の線が続けて交差し_終わりに同じ行き先が重なる() {
    let text = lanes(
        6,
        &[
            ("a", Success, Some("d")),
            ("b", Success, Some("f")),
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
| |_|/
|/| |
* | | d
|/ /
* / e
|/
* f
o @end
"
        )
    );
}

#[test]
fn 辺の一覧に残る形_交差に_両方の飛び越しが混ざる() {
    // d は成功も警告も f へ飛ぶ(次のジョブへの辺がない)。交差と同じ行き先があっても、前回までのとおり(BR2.1)。
    let transitions: Vec<Spec<'_>> = vec![
        ("a", Success, Some("e")),
        ("b", Success, Some("f")),
        ("c", Success, Some("e")),
        ("d", Success, Some("f")),
        ("d", Warning, Some("f")),
    ];
    assert_eq!(lanes(7, &transitions), None);
}

#[test]
fn 辺の一覧に残る形_交差に_分岐が混ざる() {
    // e は h と終点へ分岐する(次のジョブへの辺がない)。
    let transitions: Vec<Spec<'_>> = vec![
        ("a", Success, Some("d")),
        ("b", Success, Some("e")),
        ("c", Success, Some("d")),
        ("f", Success, Some("h")),
        ("f", Warning, None),
    ];
    assert_eq!(lanes(9, &transitions), None);
}

#[test]
fn 辺の一覧に残る形_次のジョブへの辺がないジョブを含む同じ行き先() {
    // c は成功も警告も終点へ飛ぶ。a と b も終点へ飛ぶので、同じ行き先と交差があっても、範囲の外。
    let transitions: Vec<Spec<'_>> = vec![
        ("a", Success, Some("d")),
        ("b", Success, None),
        ("c", Success, None),
        ("c", Warning, None),
    ];
    assert_eq!(lanes(5, &transitions), None);
}
