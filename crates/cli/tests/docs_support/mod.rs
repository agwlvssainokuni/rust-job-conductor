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

//! README を検査するテストの共通の補助(README の読み取り、必須語句の検査、見出しのあとの例の取り出し)。
//! README を読むだけで、書き込まない。
#![allow(dead_code)]

use std::path::{Path, PathBuf};

/// CI の `docs-check` が求める 7 つの語句(`.github/workflows/ci.yml` と同じ語と順)。
pub const DOCS_CHECK_WORDS: [&str; 7] = [
    "SHA-256",
    "Gatekeeper",
    "終了コードの表",
    "ステートファイルの場所",
    "at-least-once",
    "シグナル",
    "convert",
];

/// リポジトリのルートの README(改行は `\n` にそろえる)。
pub fn readme() -> String {
    let root: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    std::fs::read_to_string(root.join("README.md"))
        .expect("README.md を読める")
        .replace("\r\n", "\n")
}

/// 各語が README にあること。無ければ「README に {word} がない」で失敗する。
pub fn assert_readme_has(text: &str, words: &[&str]) {
    for word in words {
        assert!(text.contains(word), "README に {word} がない");
    }
}

/// `docs-check` の 7 つの語句が残っていること。無ければ「docs-check の語句 {word} がない」で失敗する。
pub fn assert_docs_check_words(text: &str) {
    for word in DOCS_CHECK_WORDS {
        assert!(text.contains(word), "docs-check の語句 {word} がない");
    }
}

/// 見出し `heading` のあとの、最初の `fence` のコードブロックの中身(末尾に改行を 1 つ足す)。
pub fn block_after_heading(text: &str, heading: &str, fence: &str) -> String {
    let start = text
        .find(heading)
        .unwrap_or_else(|| panic!("見出し `{heading}` がある"));
    let rest = &text[start..];
    let marker = format!("```{fence}\n");
    let open = rest.find(&marker).expect("例がある") + marker.len();
    let body = &rest[open..];
    let close = body.find("\n```").expect("コードブロックが閉じている");
    format!("{}\n", &body[..close])
}
