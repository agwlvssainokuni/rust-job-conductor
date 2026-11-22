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

//! 形ごとの図: 「両方の飛び越し」(成功でも警告でも同じ行き先へ飛ぶ形。BR1.3、BR1.4)と、交差・同じ
//! 行き先との組み合わせ(BR2.1)。
//!
//! 図の例 D1〜D7、N1、N3、N4。図は、規則から導いた図で、git の出力ではない。

mod graph_common;

use conductor_core::TransitionOn::{Success, Warning};
use graph_common::{Spec, both, lanes};

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

fn ordinary(n: usize, skips: &[(&str, &str)]) -> String {
    let transitions: Vec<Spec<'_>> = skips.iter().map(|(f, t)| (*f, Success, Some(*t))).collect();
    lanes(n, &transitions).unwrap()
}

fn both_skips(n: usize, skips: &[(&str, &str)]) -> String {
    let transitions: Vec<Spec<'_>> = skips.iter().flat_map(|(f, t)| both(f, Some(t))).collect();
    lanes(n, &transitions).unwrap()
}

/// N1 の遷移: 「両方の飛び越し」`a` が `d` へと、通常の飛び越し `b` の成功が `e` へが交差。
fn n1() -> Vec<Spec<'static>> {
    let mut transitions = both("a", Some("d")).to_vec();
    transitions.push(("b", Success, Some("e")));
    transitions
}

#[test]
fn d1_両方の飛び越しは_通常の飛び越しと同じ線で描き_主線の線だけ省く() {
    // b は実行されないので、b と c のあいだの列 0 は空白のまま(BR1.4)。
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
fn n1_両方の飛び越しと通常の飛び越しの交差() {
    // 通常の飛び越しが交差する形と同じく、レーンの図になる。
    // b、c は実行されないので、その下の行の列 0 は空白。d は実行されうる(BR1.4)。
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
fn ジョブ1個の両方の飛び越しは_終点への線になる() {
    // a は成功も警告も @end へ(次の節が @end なので、飛び越しではない)。
    assert_eq!(lanes(1, &both("a", None)).as_deref(), Some("* a\no @end\n"));
}

#[test]
fn 交差と離れた別のジョブの両方の飛び越しは_レーンの図になる() {
    // 交差の部分は、通常の飛び越しだけのグラフと同じ。g の両方の飛び越しの下は、列 0 が空白(BR1.4、BR2.1)。
    let mut transitions: Vec<Spec<'_>> = vec![("a", Success, Some("d")), ("b", Success, Some("e"))];
    transitions.extend([("g", Success, Some("i")), ("g", Warning, Some("i"))]);
    assert_eq!(
        lanes(10, &transitions).as_deref(),
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
* f
*   g
 \\
* | h
 /
* i
* j
o @end
"
        )
    );
}

#[test]
fn 交差に_両方の飛び越しが混ざる形は_レーンの図になる() {
    // 交差と同じ行き先に、「両方の飛び越し」(d が成功も警告も f へ)が加わる(BR2.1)。d は次のジョブへの
    // 辺がない。d の線は、b の線(f へ)と同じ行き先で、次のノードの手前で寄せる。d の下の行の列 0 は
    // 空白(BR1.4)。
    let transitions: Vec<Spec<'_>> = vec![
        ("a", Success, Some("e")),
        ("b", Success, Some("f")),
        ("c", Success, Some("e")),
        ("d", Success, Some("f")),
        ("d", Warning, Some("f")),
    ];
    assert_eq!(
        lanes(7, &transitions).as_deref(),
        Some(
            "\
*   a
|\\
* \\   b
|\\ \\
* \\ \\   c
|\\ \\ \\
| | |/
| |/|
* | |   d
 \\ \\ \\
  |/ /
 /| /
  |/
* / e
|/
* f
* g
o @end
"
        )
    );
}
