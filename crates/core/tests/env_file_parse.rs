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

//! `.env` の解析(U1。BR1.3〜BR1.5、BR2.1〜BR2.8、BR3.2、FR1.6、FR1.7、FR1.9)。

mod common;

use common::SECRET;
use conductor_core::{EnvFileError, EnvFileErrorKind as K, EnvWarningKind, parse_env_file};

fn pairs(text: &str) -> Vec<(String, String)> {
    parse_env_file(text, "t.env")
        .unwrap_or_else(|e| panic!("受理されること: {e:?}"))
        .entries
        .into_iter()
        .map(|e| (e.name, e.value))
        .collect()
}

fn errs(text: &str) -> Vec<(Option<usize>, K)> {
    parse_env_file(text, "t.env")
        .expect_err("誤りになること")
        .into_iter()
        .map(|e: EnvFileError| (e.line, e.kind))
        .collect()
}

fn one(text: &str) -> String {
    let p = pairs(text);
    assert_eq!(p.len(), 1, "{p:?}");
    p[0].1.clone()
}

#[test]
fn 受け付ける書き方() {
    assert_eq!(one("A=1"), "1");
    assert_eq!(one("export A=1"), "1");
    assert_eq!(one("export\tA=1"), "1");
    assert_eq!(one("  A = 1  "), "1");
    assert_eq!(one("A=x=y"), "x=y");
    assert_eq!(one("A=x #c"), "x");
    assert_eq!(one("A=x#y"), "x#y");
    assert_eq!(one("A="), "");
    assert_eq!(one("A= #c"), "");
    assert_eq!(one("A=${HOME}/$X"), "${HOME}/$X", "展開しない");
    assert_eq!(one("A= \"x\""), "x", "引用符の前の空白は読み飛ばす");
    assert_eq!(
        pairs("# c\n\n  # d\nA=1\nexport=2\n"),
        [("A".into(), "1".into()), ("export".into(), "2".into())]
    );
}

#[test]
fn 単一引用符はそのままで二重引用符はエスケープを解く() {
    assert_eq!(one("A='a\\nb # $X'"), "a\\nb # $X");
    assert_eq!(one("A='x' # c"), "x");
    assert_eq!(one("A=\"a\\nb\\rc\\td\\\"e\\\\f\""), "a\nb\rc\td\"e\\f");
    assert_eq!(one("A=\"x # y\" # c"), "x # y");
    assert_eq!(one("A=\"\""), "");
    assert_eq!(errs("A=\"\\q\""), [(Some(1), K::InvalidEscape)]);
    assert_eq!(errs("A=\"\\$\""), [(Some(1), K::InvalidEscape)]);
}

#[test]
fn 閉じていない引用符と閉じたあとのごみ() {
    assert_eq!(errs("A='x"), [(Some(1), K::UnterminatedQuote)]);
    assert_eq!(errs("A=\"x"), [(Some(1), K::UnterminatedQuote)]);
    assert_eq!(errs("A=\"x\\"), [(Some(1), K::UnterminatedQuote)]);
    assert_eq!(
        errs("A=\"x\nB=1\"\n"),
        [(Some(1), K::UnterminatedQuote)],
        "引用符が行をまたぐ値は受け付けない(2 行目は、引用符を含む通常の値)"
    );
    assert_eq!(errs("A='x' y"), [(Some(1), K::TrailingGarbage)]);
    assert_eq!(errs("A=\"x\"y"), [(Some(1), K::TrailingGarbage)]);
}

#[test]
fn bom_と_crlf() {
    assert_eq!(pairs("\u{FEFF}A=1\r\nB=\"2\"\r\n").len(), 2);
    assert_eq!(one("\u{FEFF}A=1\r\n"), "1");
    assert_eq!(
        pairs("A=x\r\nB=y\r\n")[0].1,
        "x",
        "行末の CR は値に入らない"
    );
    assert_eq!(
        errs("\u{FEFF}A\r\nB=1\r\nC\r\n"),
        [(Some(1), K::MissingEquals), (Some(3), K::MissingEquals)]
    );
}

#[test]
fn 重複は後ろが勝ち警告に行と名前を持つ() {
    let c = parse_env_file("A=1\nB=2\nA=3\n", "t.env").unwrap();
    let p: Vec<_> = c
        .entries
        .iter()
        .map(|e| (e.name.as_str(), e.value.as_str()))
        .collect();
    assert_eq!(p, [("B", "2"), ("A", "3")], "後ろの項目は末尾側に並ぶ");
    assert_eq!(c.warnings.len(), 1);
    let w = &c.warnings[0];
    assert_eq!(
        (w.file.as_str(), w.line, w.name.as_str()),
        ("t.env", 3, "A")
    );
    assert_eq!(w.kind, EnvWarningKind::DuplicateKey);
}

#[test]
fn 同じファイルの大文字小文字だけが違う名前は誤り() {
    assert_eq!(errs("foo=1\nFOO=2\n"), [(Some(2), K::CaseConflict)]);
}

#[test]
fn 名前の誤りと予約名() {
    assert_eq!(errs("=1"), [(Some(1), K::InvalidName)]);
    assert_eq!(errs("1A=x"), [(Some(1), K::InvalidName)]);
    assert_eq!(errs("A-B=x"), [(Some(1), K::InvalidName)]);
    assert_eq!(errs("A"), [(Some(1), K::MissingEquals)]);
    assert_eq!(errs("export"), [(Some(1), K::MissingEquals)]);
    assert_eq!(errs("CONDUCTOR_X=1"), [(Some(1), K::ReservedName)]);
    assert_eq!(errs("conductor_x=1"), [(Some(1), K::ReservedName)]);
    assert_eq!(
        errs("export CONDUCTOR_RUN_ID=1"),
        [(Some(1), K::ReservedName)]
    );
}

#[test]
fn nul_を含む値は誤り() {
    assert_eq!(errs("A=x\0y"), [(Some(1), K::NulInValue)]);
    assert_eq!(errs("A=\"x\0\""), [(Some(1), K::NulInValue)]);
}

#[test]
fn 項目の上限は_10000_で超えると誤り() {
    let make = |n: usize| -> String { (0..n).map(|i| format!("K{i}=v\n")).collect() };
    assert_eq!(
        parse_env_file(&make(10_000), "t.env")
            .unwrap()
            .entries
            .len(),
        10_000
    );
    assert_eq!(errs(&make(10_001)), [(Some(10_001), K::TooManyEntries)]);
}

#[test]
fn 複数の誤りが行番号つきで集まり値を含まない() {
    let text =
        format!("A={SECRET}\nBAD {SECRET}\n1X={SECRET}\nC=\"{SECRET}\\q\"\nD='{SECRET}\nE=1\n");
    let e = parse_env_file(&text, "t.env").unwrap_err();
    let got: Vec<_> = e.iter().map(|e| (e.line, e.kind)).collect();
    assert_eq!(
        got,
        [
            (Some(2), K::MissingEquals),
            (Some(3), K::InvalidName),
            (Some(4), K::InvalidEscape),
            (Some(5), K::UnterminatedQuote),
        ]
    );
    for err in &e {
        assert!(!err.to_string().contains(SECRET), "{err}");
        assert!(!err.kind.reason().contains(SECRET));
    }
}
