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

//! 横断のテストのうち、ランダムな入力(NFR4.7。パニックしない)と、ランダムな構造の往復(BR1.11)。
//! 変換した結果を読んだ木を比べるので、ここに置く。ほかの横断のテストは、統合テストの
//! `tests/nfr_security.rs` にある。

use std::collections::BTreeSet;

use super::{toml, with_timeout, yaml};
use crate::{TargetFormat, convert_text};

/// 小さな疑似乱数(再現できる)。
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as usize
    }
}

#[test]
fn ランダムな入力と切り詰めた入力で_パニックしない() {
    let alphabet: Vec<&str> = vec![
        "a",
        "b",
        "1",
        "-",
        "?",
        ":",
        " ",
        "\n",
        "  ",
        "[",
        "]",
        "{",
        "}",
        ",",
        "#",
        "&x ",
        "*x",
        "!t ",
        "!!str ",
        "<<",
        "\"",
        "'",
        "|",
        ">",
        "---\n",
        "...\n",
        "=",
        ".",
        "e",
        "\\",
        "~",
        "[[",
        "]]",
        "0x",
        "9999999999999999999",
        "日",
        "\t",
        "%",
        "@",
        "`",
    ];
    let outcome = with_timeout(move || {
        let mut rng = Lcg(42);
        let mut count = 0usize;
        for _ in 0..3000 {
            let len = rng.next() % 40;
            let text: String = (0..len)
                .map(|_| alphabet[rng.next() % alphabet.len()])
                .collect();
            for target in [TargetFormat::Toml, TargetFormat::Yaml] {
                // 成功でも失敗でもよい。パニックしないこと、成功したら読み戻せること
                if let Ok(out) = convert_text(&text, target, "t") {
                    let back = match target {
                        TargetFormat::Toml => toml(&out.text),
                        TargetFormat::Yaml => yaml(&out.text),
                    };
                    assert!(back.is_ok(), "{text:?} -> {:?}", out.text);
                }
                count += 1;
            }
        }
        // 実際のファイルを、1 文字ずつ短くした入力
        for source in [
            include_str!("../../tests/fixtures/pipeline.yaml"),
            include_str!("../../tests/fixtures/pipeline.toml"),
        ] {
            for end in 0..=source.len() {
                if !source.is_char_boundary(end) {
                    continue;
                }
                for target in [TargetFormat::Toml, TargetFormat::Yaml] {
                    let _ = convert_text(&source[..end], target, "t");
                    count += 1;
                }
            }
        }
        count
    });
    assert!(outcome > 6000);
}

#[test]
fn 変換できたランダムな構造は往復する() {
    // 往復の確認(BR1.11)を、組み合わせの入力にも広げる
    let keys = ["a", "yes", "1", "null", "<<", "", "x y", "on", "true", "~"];
    let vals = [
        "yes", "1", "'1'", "\"null\"", "1.5", "-0.0", ".inf", "[]", "{}", "[1, [2]]", "{k: v}",
        "text", "'<<'", "\"\"", "0x10",
    ];
    let mut rng = Lcg(7);
    for _ in 0..300 {
        let n = 1 + rng.next() % 5;
        let mut text = String::new();
        let mut used = BTreeSet::new();
        for _ in 0..n {
            let k = keys[rng.next() % keys.len()];
            if !used.insert(k) {
                continue;
            }
            let v = vals[rng.next() % vals.len()];
            text.push_str(&format!("{}: {}\n", yaml_key(k), v));
        }
        let Ok(original) = yaml(&text) else {
            continue;
        };
        let toml_text = convert_text(&text, TargetFormat::Toml, "t")
            .unwrap_or_else(|e| panic!("{text:?}: {e}"));
        assert_eq!(
            toml(&toml_text.text).unwrap(),
            original,
            "{text:?}\n{}",
            toml_text.text
        );
        let back = convert_text(&toml_text.text, TargetFormat::Yaml, "t").unwrap();
        assert_eq!(
            yaml(&back.text).unwrap(),
            original,
            "{text:?}\n{}",
            back.text
        );
    }
}

fn yaml_key(k: &str) -> String {
    // キーは、必ず引用符で囲んで、文字列として書く
    format!("\"{k}\"")
}
