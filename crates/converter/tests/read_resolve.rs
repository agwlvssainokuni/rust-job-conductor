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

//! 文書の確認と展開(RootCheck、Resolve)、失敗の順序(BR1.12)、暴走の入力。

mod common;

use common::{a, i, m, s, toml, with_timeout, yaml};
use conductor_converter::ConversionErrorKind as K;

// ---- 文書の確認 ----

#[test]
fn 複数の文書は表せない() {
    let e = yaml("a: 1\n---\nb: 2\n").unwrap_err();
    assert_eq!(e.kind, K::Unrepresentable);
    assert!(e.key_path.is_none());
}

#[test]
fn 最上位はマップでなければならない() {
    for text in [
        "- 1\n- 2\n",
        "just a scalar\n",
        "42\n",
        "",
        "# コメントだけ\n",
        "---\n",
        "[1, 2]\n",
    ] {
        let e = yaml(text).unwrap_err();
        assert_eq!(e.kind, K::RootNotMap, "{text:?}");
    }
    // 空の TOML は、空のマップ
    assert_eq!(toml("").unwrap(), m(vec![]));
    assert_eq!(yaml("{}\n").unwrap(), m(vec![]));
}

// ---- アンカー・エイリアス・マージキー ----

#[test]
fn アンカーとエイリアスを展開する() {
    let v = yaml("base: &b\n  x: 1\n  y: [1, 2]\ncopy: *b\nn: &n 5\nm: *n\n").unwrap();
    let base = m(vec![("x", i(1)), ("y", a(vec![i(1), i(2)]))]);
    assert_eq!(
        v,
        m(vec![
            ("base", base.clone()),
            ("copy", base),
            ("n", i(5)),
            ("m", i(5))
        ])
    );
}

#[test]
fn キーに使ったエイリアス() {
    let v = yaml("k: &k name\n*k : 1\n").unwrap();
    assert_eq!(v, m(vec![("k", s("name")), ("name", i(1))]));
}

#[test]
fn マージキーの単独と並びと優先順位() {
    let v = yaml("a: &a {x: 1, y: 1}\nb: &b {y: 2, z: 2}\nc:\n  <<: *a\n  y: 9\nd:\n  <<: [*a, *b]\ne:\n  <<: [*b, *a]\n").unwrap();
    assert_eq!(
        v,
        m(vec![
            ("a", m(vec![("x", i(1)), ("y", i(1))])),
            ("b", m(vec![("y", i(2)), ("z", i(2))])),
            // マップ自身のキーが優先
            ("c", m(vec![("x", i(1)), ("y", i(9))])),
            // 並びは、先頭が優先
            ("d", m(vec![("x", i(1)), ("y", i(1)), ("z", i(2))])),
            ("e", m(vec![("x", i(1)), ("y", i(2)), ("z", i(2))])),
        ])
    );
}

#[test]
fn インラインのマップのマージキーと_マージより前の自身のキー() {
    let v = yaml("a:\n  y: 9\n  <<: {x: 1, y: 1}\n").unwrap();
    assert_eq!(v, m(vec![("a", m(vec![("y", i(9)), ("x", i(1))]))]));
}

#[test]
fn 引用符つきのマージキーは普通のキー() {
    let v = yaml("a: &a {x: 1}\nb:\n  \"<<\": *a\n").unwrap();
    assert_eq!(
        v,
        m(vec![
            ("a", m(vec![("x", i(1))])),
            ("b", m(vec![("<<", m(vec![("x", i(1))]))]))
        ])
    );
}

#[test]
fn 値がマップでないマージキーは表せない() {
    for text in [
        "a:\n  <<: 5\n",
        "a:\n  <<: [1, 2]\n",
        "a:\n  <<: ~\n",
        "a:\n  <<: text\n",
    ] {
        let e = yaml(text).unwrap_err();
        assert_eq!(e.kind, K::Unrepresentable, "{text:?}");
        assert_eq!(e.key_path.as_deref(), Some("a.<<"), "{text:?}");
        assert!(e.line.is_some());
    }
}

#[test]
fn エイリアスを通した重複するキーは文法の誤り() {
    let e = yaml("k: &k name\nname: 1\n*k : 2\n").unwrap_err();
    assert_eq!(e.kind, K::Syntax);
}

// ---- 上限(要素数、深さ) ----

fn flat_array_yaml(n: usize) -> String {
    // 根のマップ 1 + キー 1 + 配列 1 + 要素 n = n + 3 ノード
    format!("a: [{}]\n", vec!["0"; n].join(","))
}

#[test]
fn 要素数ちょうど100000は受理し_1つ多いと大きすぎる() {
    let ok = yaml(&flat_array_yaml(99_997)).expect("100,000 ちょうどは受理");
    assert!(matches!(ok, conductor_converter::testing::Value::Map(_)));
    let e = yaml(&flat_array_yaml(99_998)).unwrap_err();
    assert_eq!(e.kind, K::TooLarge);
    assert_eq!(e.reason, "展開後の要素数が上限(100000)を超えています");
}

#[test]
fn toml_の入力も要素数の上限は同じ() {
    let make = |n: usize| format!("a = [{}]\n", vec!["0"; n].join(","));
    assert!(toml(&make(99_997)).is_ok());
    assert_eq!(toml(&make(99_998)).unwrap_err().kind, K::TooLarge);
}

#[test]
fn エイリアスの爆発は_メモリを使い果たさずに大きすぎる() {
    let result = with_timeout(|| {
        // 大きなスカラー(10,000 バイト)を含めても、要素数と文字の大きさのどちらかの上限で止まる
        let mut text = format!("l0: &l0 [{}, x, x, x, x, x, x, x, x]\n", "y".repeat(10_000));
        for n in 1..=10 {
            let refs = vec![format!("*l{}", n - 1); 9].join(", ");
            text.push_str(&format!("l{n}: &l{n} [{refs}]\n"));
        }
        yaml(&text)
    });
    assert_eq!(result.unwrap_err().kind, K::TooLarge);
}

#[test]
fn 大きなスカラーのエイリアスの爆発は_大きさで大きすぎる() {
    // 20,000 バイトの文字列を 5,000 回参照する(入力は約 35 KB。ノード数は 5,001 で上限の内)
    let result = with_timeout(|| {
        let big = "x".repeat(20_000);
        let refs = vec!["*s"; 5_000].join(",");
        let text = format!("s: &s {big}\nl: [{refs}]\n");
        assert!(text.len() < 40_000);
        let start = std::time::Instant::now();
        let r = yaml(&text);
        (r, start.elapsed())
    });
    let (r, elapsed) = result;
    let e = r.unwrap_err();
    assert_eq!(e.kind, K::TooLarge);
    assert_eq!(e.reason, "展開後の文字の大きさが上限(16 MiB)を超えています");
    assert!(elapsed.as_secs() < 5, "{elapsed:?}");
}

#[test]
fn マージキーと値を通したエイリアスの爆発は_大きさで大きすぎる() {
    let result = with_timeout(|| {
        let big_key = "k".repeat(20_000);
        let refs = vec!["*b"; 2_000].join(", ");
        let text = format!("b: &b {{{big_key}: {big_key}}}\nm:\n  <<: [{refs}]\n");
        yaml(&text)
    });
    assert_eq!(result.unwrap_err().kind, K::TooLarge);
}

#[test]
fn 既定の16_mibでの展開後の大きさの境界() {
    // a, b, c が、`n` バイトの文字列(a はアンカー、b と c は参照)。キー 4 つ(a b c d)で 4 バイト。
    // 数値のスカラー(`d` の値)も、文字のバイト数で数える。3n + 4 + 3 = 16 MiB ちょうど
    let n = (16 * 1024 * 1024 - 7) / 3;
    assert_eq!(3 * n + 7, 16 * 1024 * 1024);
    let (ok, over) = with_timeout(move || {
        let s = "x".repeat(n);
        let ok = yaml(&format!("a: &a {s}\nb: *a\nc: *a\nd: 123\n"));
        // d の値を 1 バイト長くして、1 バイト超える
        let over = yaml(&format!("a: &a {s}\nb: *a\nc: *a\nd: 1234\n"));
        (ok, over)
    });
    assert!(ok.is_ok(), "ちょうど 16 MiB は受理");
    assert_eq!(over.unwrap_err().kind, K::TooLarge);
}

#[test]
fn 大きなマップを何度もマージすると_捨てた要素も数える() {
    let result = with_timeout(|| {
        let keys: Vec<String> = (0..1000).map(|n| format!("k{n}: 1")).collect();
        let refs = vec!["*b"; 200].join(", ");
        // 同じキーばかりなので、結果は小さい。それでも、訪れたノードを数える
        let text = format!("base: &b {{{}}}\nm:\n  <<: [{refs}]\n", keys.join(", "));
        yaml(&text)
    });
    assert_eq!(result.unwrap_err().kind, K::TooLarge);
}

#[test]
fn 読んだ深さが小さくても_展開後の深さが32を超えれば文法の誤り() {
    // 読み取り時の深さは 2 だが、エイリアスの入れ子で、展開後は深くなる
    let build = |levels: usize| {
        let mut text = String::from("l0: &l0 [x]\n");
        for n in 1..levels {
            text.push_str(&format!("l{n}: &l{n} [*l{}]\n", n - 1));
        }
        text
    };
    // l30: 根 1 + 配列 31 = 32 段
    assert!(yaml(&build(31)).is_ok());
    let e = yaml(&build(32)).unwrap_err();
    assert_eq!(e.kind, K::Syntax);
    assert_eq!(e.reason, "展開後の入れ子が深すぎます(上限 32)");
    assert!(e.key_path.is_some());
}

#[test]
fn 読み取りの深さの上限_32_と_33() {
    let ok = format!("a: {}{}\n", "[".repeat(31), "]".repeat(31));
    assert!(yaml(&ok).is_ok());
    let ng = format!("a: {}{}\n", "[".repeat(32), "]".repeat(32));
    assert_eq!(yaml(&ng).unwrap_err().kind, K::Syntax);
}

#[test]
fn 極端に深い入力は_異常終了せずに文法の誤りで失敗する() {
    let kinds = with_timeout(|| {
        let mut kinds = Vec::new();
        for n in [1000usize, 100_000] {
            kinds.push(
                yaml(&format!("a: {}{}", "[".repeat(n), "]".repeat(n)))
                    .unwrap_err()
                    .kind,
            );
            kinds.push(
                yaml(&format!("a: {}1{}", "{a: ".repeat(n), "}".repeat(n)))
                    .unwrap_err()
                    .kind,
            );
            kinds.push(yaml(&format!("{}x\n", "- ".repeat(n))).unwrap_err().kind);
            kinds.push(
                toml(&format!("a = {}{}", "[".repeat(n), "]".repeat(n)))
                    .unwrap_err()
                    .kind,
            );
            kinds.push(
                toml(&format!("a = {}1{}", "{a=".repeat(n), "}".repeat(n)))
                    .unwrap_err()
                    .kind,
            );
        }
        let mut indent = String::new();
        for level in 0..1300 {
            indent.push_str(&" ".repeat(level));
            indent.push_str("a:\n");
        }
        kinds.push(yaml(&indent).unwrap_err().kind);
        kinds
    });
    assert!(kinds.iter().all(|k| *k == K::Syntax), "{kinds:?}");
}

// ---- 表せない値(最初の 1 つ、経路) ----

#[test]
fn 入力の順で最初の表せない値を_キーの場所つきで報告する() {
    let e =
        yaml("a: 1\njob:\n  - id: x\n  - id: y\n    note: ~\n  - z: ~\nlater: ~\n").unwrap_err();
    assert_eq!(e.kind, K::Unrepresentable);
    assert_eq!(e.key_path.as_deref(), Some("job[1].note"));
    assert_eq!(e.reason, "null は表せません");
}

#[test]
fn null_の書き方と_ほかの表せない種類() {
    for (text, path, why) in [
        ("a: ~\n", "a", "null は表せません"),
        ("a: null\n", "a", "null は表せません"),
        ("a:\n", "a", "null は表せません"),
        ("a: !!binary aGk=\n", "a", "タグつきの値は表せません"),
        ("a: !mytag 1\n", "a", "タグつきの値は表せません"),
        ("a: !!set {x}\n", "a", "タグつきの値は表せません"),
        ("a: !!seq {x: 1}\n", "a", "タグつきの値は表せません"),
        (
            "a: 9223372036854775808\n",
            "a",
            "数値が表せる範囲を超えています",
        ),
        (
            "a: [1, 99999999999999999999]\n",
            "a[1]",
            "数値が表せる範囲を超えています",
        ),
        ("1: x\n", "(非文字列キー)", "文字列でないキーは表せません"),
        (
            "a:\n  true: x\n",
            "a.(非文字列キー)",
            "文字列でないキーは表せません",
        ),
        (
            "? [1, 2]\n: x\n",
            "(非文字列キー)",
            "文字列でないキーは表せません",
        ),
        (
            "? ~\n: x\n",
            "(非文字列キー)",
            "文字列でないキーは表せません",
        ),
        ("a: 1e999\n", "a", "数値が表せる範囲を超えています"),
    ] {
        let e = yaml(text).unwrap_err();
        assert_eq!(e.kind, K::Unrepresentable, "{text:?}");
        assert_eq!(e.key_path.as_deref(), Some(path), "{text:?}");
        assert_eq!(e.reason, why, "{text:?}");
    }
}

#[test]
fn toml_の日時は表せない() {
    let e = toml("name = \"x\"\n[a]\nwhen = 1979-05-27T07:32:00Z\n").unwrap_err();
    assert_eq!(e.kind, K::Unrepresentable);
    assert_eq!(e.key_path.as_deref(), Some("a.when"));
    for text in [
        "d = 1979-05-27\n",
        "t = 07:32:00\n",
        "o = 1979-05-27 07:32:00+09:00\n",
    ] {
        assert_eq!(toml(text).unwrap_err().kind, K::Unrepresentable, "{text:?}");
    }
}

#[test]
fn str_のタグと_一致するコアのタグは受理する() {
    let v = yaml("a: !!str 1\nb: !!int '5'\nc: !!float 1\nd: !!bool true\ne: !!str\nf: !!map {x: 1}\ng: !!seq [1]\n").unwrap();
    assert_eq!(
        v,
        m(vec![
            ("a", s("1")),
            ("b", i(5)),
            ("c", conductor_converter::testing::Value::Float(1.0)),
            ("d", conductor_converter::testing::Value::Bool(true)),
            ("e", s("")),
            ("f", m(vec![("x", i(1))])),
            ("g", a(vec![i(1)])),
        ])
    );
}

#[test]
fn コアスキーマのスカラー() {
    let v = yaml(
        "a: yes\nb: on\nc: no\nd: off\ne: true\nf: 0x1F\ng: 1.5\nh: '1'\ni: \"null\"\nj: 007\n",
    )
    .unwrap();
    assert_eq!(
        v,
        m(vec![
            ("a", s("yes")),
            ("b", s("on")),
            ("c", s("no")),
            ("d", s("off")),
            ("e", conductor_converter::testing::Value::Bool(true)),
            ("f", i(31)),
            ("g", conductor_converter::testing::Value::Float(1.5)),
            ("h", s("1")),
            ("i", s("null")),
            ("j", i(7)),
        ])
    );
}

// ---- 失敗の順序(BR1.12) ----

#[test]
fn 失敗の順_文法の誤りはその後のすべてより先() {
    // 5) syntax は、複数の文書(6)より先。2 つめの文書の誤りも見つける
    assert_eq!(yaml("a: 1\n---\nb: [1\n").unwrap_err().kind, K::Syntax);
    // 重複するキー(5)は、複数の文書(6)より先
    assert_eq!(yaml("a: 1\na: 2\n---\nb: 1\n").unwrap_err().kind, K::Syntax);
    // 再帰するアンカー(5)は、最上位がマップでないこと(7)より先
    assert_eq!(yaml("- &r [*r]\n").unwrap_err().kind, K::Syntax);
    // 展開前の深さの超過(5)は、表せない値(9)より先
    let deep = format!("n: ~\na: {}{}\n", "[".repeat(32), "]".repeat(32));
    assert_eq!(yaml(&deep).unwrap_err().kind, K::Syntax);
}

#[test]
fn 失敗の順_複数の文書_最上位がマップでない_値の順() {
    // 6) 複数の文書は、7) 最上位がマップでない より先
    assert_eq!(
        yaml("- 1\n---\n- 2\n").unwrap_err().kind,
        K::Unrepresentable
    );
    // 7) rootNotMap は、9) 表せない値 より先
    assert_eq!(yaml("- ~\n").unwrap_err().kind, K::RootNotMap);
    // 最初の文書が対象(最初がマップなら、通る)
    assert_eq!(
        yaml("---\n- 1\n---\na: 1\n").unwrap_err().kind,
        K::Unrepresentable
    );
}

#[test]
fn 失敗の順_展開の上限は表せない値より先() {
    // 8) 展開後の要素数の超過は、先に見つけた null(9)より優先する
    let text = format!("n: ~\na: [{}]\n", vec!["0"; 100_000].join(","));
    assert_eq!(yaml(&text).unwrap_err().kind, K::TooLarge);
    // 8) 展開後の深さの超過も、先の null より優先する
    let mut deep = String::from("n: ~\nl0: &l0 [x]\n");
    for n in 1..32 {
        deep.push_str(&format!("l{n}: &l{n} [*l{}]\n", n - 1));
    }
    assert_eq!(yaml(&deep).unwrap_err().kind, K::Syntax);
    // 8) 展開できないマージキーは、先の null より先に返す(経路で確かめる)
    let e = yaml("n: ~\nb:\n  <<: 5\n").unwrap_err();
    assert_eq!(e.kind, K::Unrepresentable);
    assert_eq!(e.key_path.as_deref(), Some("b.<<"));
}

#[test]
fn 失敗の順_同じ段の中では入力の順で最初のもの() {
    // 同じ段階で複数あるときは、入力に現れる順で最初のもの
    let e = yaml("a: ~\nb: !!binary x\n").unwrap_err();
    assert_eq!(e.key_path.as_deref(), Some("a"));
    let e = yaml("b: !!binary x\na: ~\n").unwrap_err();
    assert_eq!(e.key_path.as_deref(), Some("b"));
}

#[test]
fn 誤りの文は入力の値を含まない() {
    let secret = "SECRET-VALUE-123";
    for text in [
        format!("k: !!binary {secret}\n"),
        format!("{secret}:\n  - ~\n"),
        format!("a: [{secret}\n"),
        format!("{secret}: 1\n{secret}: 2\n"),
        format!("- {secret}\n"),
        format!("a:\n  <<: {secret}\n"),
        format!("1: {secret}\n"),
    ] {
        let e = yaml(&text).unwrap_err();
        let shown = format!("{e} {:?} {:?} {}", e.key_path, e.line, e.reason);
        // キー名は、エスケープされた形で現れてよい(最上位のキーが `secret` の場合)
        let key_named = text.starts_with(secret);
        assert!(key_named || !shown.contains(secret), "{text:?} -> {shown}");
        assert!(!e.reason.contains(secret));
    }
}
