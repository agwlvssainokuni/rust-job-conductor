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

//! 形ごとの図: 同じ行き先(合流)と、それが交差と混ざる形(BR1.1〜BR1.5、BR2.1)。
//!
//! 図の例 M2〜M4(`git log --graph` の出力と一致を確かめた形)と、次のジョブへの辺がないジョブを含む形、
//! 交差と離れた同じ行き先。

mod graph_common;

use conductor_core::TransitionOn::{Success, Warning};
use graph_common::{Spec, lanes};

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
fn 交差と離れた別のジョブの同じ行き先は_レーンの図になる() {
    // g→i と f→i が同じ行き先(合流)。通常の飛び越しだけなので、図にする(BR1.1)。
    let transitions: Vec<Spec<'_>> = vec![
        ("a", Success, Some("d")),
        ("b", Success, Some("e")),
        ("f", Success, Some("i")),
        ("g", Success, Some("i")),
    ];
    let text = lanes(10, &transitions).expect("通常の飛び越しだけのグラフは、図になる");
    assert!(text.ends_with("o @end\n"), "{text}");
}

#[test]
fn 次のジョブへの辺がないジョブを含む同じ行き先は_レーンの図になる() {
    // c は成功も警告も終点へ飛ぶ。b の線(終点へ)に、合流の行で重なる。c の下の行の列 0 は空白(BR1.4)。
    let transitions: Vec<Spec<'_>> = vec![
        ("a", Success, Some("d")),
        ("b", Success, None),
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
* | | c
 \\| |
  |/
 /|
* | d
* | e
|/
o @end
"
        )
    );
}
