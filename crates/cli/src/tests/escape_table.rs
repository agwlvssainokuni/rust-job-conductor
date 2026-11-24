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

//! `escape_display` の、同じ入力と期待値の表(conductor-core・conductor-converter と揃える。NFR4.2)。
//!
//! このテストは期待値の表を固定する。conductor-converter の同じ規則の写しは、非公開のため、
//! converter の単体テストにある、同じ期待値の表で揃える。

use crate::report::{escape_full, escape_name};
use conductor_core::escape_display;

/// (入力, 上限, 期待値)。conductor-core・conductor-converter の単体テストの期待値と同じ規則。
fn table() -> Vec<(String, usize, String)> {
    vec![
        ("a\nb\x1b[0m".into(), 64, "a\\nb\\u{1b}[0m".into()),
        ("k\tx".into(), 64, "k\\tx".into()),
        ("a\rb".into(), 64, "a\\rb".into()),
        ("日本語 abc".into(), 64, "日本語 abc".into()),
        ("a\u{200b}b".into(), 64, "a\\u{200b}b".into()),
        ("a\u{202e}b".into(), 64, "a\\u{202e}b".into()),
        ("a\u{2066}b".into(), 64, "a\\u{2066}b".into()),
        ("a\u{feff}b".into(), 64, "a\\u{feff}b".into()),
        (
            "a\u{2028}b\u{2029}".into(),
            64,
            "a\\u{2028}b\\u{2029}".into(),
        ),
        ("\x00\x7f\u{85}".into(), 64, "\\u{0}\\u{7f}\\u{85}".into()),
        ("k".repeat(64), 64, "k".repeat(64)),
        ("k".repeat(200), 64, format!("{}…", "k".repeat(64))),
        ("\n".repeat(100), 64, format!("{}…", "\\n".repeat(32))),
        ("abcdef".into(), 3, "abc…".into()),
        ("".into(), 64, "".into()),
    ]
}

#[test]
fn 期待値の表のとおりにエスケープされ_切り詰められる() {
    for (input, limit, expected) in table() {
        assert_eq!(escape_display(&input, Some(limit)), expected, "{input:?}");
    }
}

#[test]
fn パスは切らず_エスケープだけ() {
    let long = format!("dir\n/{}", "p".repeat(300));
    let out = escape_full(&long);
    assert!(out.starts_with("dir\\n/"));
    assert!(out.ends_with(&"p".repeat(300)));
    assert_eq!(escape_name(&long).chars().count(), 65);
}

#[test]
fn エスケープ後の文字数で切る() {
    // `\n` は 2 文字になる。64 文字を超えないところで切れる。
    let out = escape_display(&"\n".repeat(100), Some(64));
    assert_eq!(out.chars().count(), 65);
}
