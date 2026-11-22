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

//! 形ごとの図: 通常の飛び越しと、その交差(BR1.1〜BR1.5、BR2.1)。
//!
//! 図の例 C1〜C3(`git log --graph` の出力と一致を確かめた形)と、交差を含まない形、次のジョブへの辺を
//! 持たないジョブがある交差。交差の性能は `graph_perf.rs`。

mod graph_common;

use conductor_core::TransitionOn::{Success, Warning};
use graph_common::{Spec, lanes};

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
fn 次のジョブへの辺を持たないジョブがある交差は_レーンの図になる() {
    // c は、成功も警告も終点へ飛ぶ(次のジョブへの辺がない)。c の下の行の列 0 は空白。
    // d、e は(a、b の飛び越しで)実行されうる、かつ次の節への辺を持つので、その下の行の列 0 は `|`(BR1.4)。
    let transitions: Vec<Spec<'_>> = vec![
        ("a", Success, Some("d")),
        ("b", Success, Some("e")),
        ("c", Success, None),
        ("c", Warning, None),
    ];
    assert_eq!(
        lanes(5, &transitions).as_deref(),
        Some(
            "\
*   a
|\\
* \\   b
|\\ \\
* \\ \\   c
 \\ \\ \\
  |_|/
 /| |
* | | d
| |/
|/|
* | e
|/
o @end
"
        )
    );
}
