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

//! README のリトライの説明の検査(U3。FR8.1、FR6.2)。
//!
//! README を読むだけで、書き込まない。例の定義は、一時ファイルに書いて、実際の読み取りに通す。

use std::path::{Path, PathBuf};

fn readme() -> String {
    let root: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    std::fs::read_to_string(root.join("README.md"))
        .expect("README.md を読める")
        .replace("\r\n", "\n")
}

/// 小節の見出し(リトライの設定)の行。
const HEADING: &str = "### リトライの設定(`[job.retry]`)";

/// 見出しのあとの、最初の ```toml ブロックの中身。
fn first_toml_block_after_heading(text: &str) -> String {
    let start = text.find(HEADING).expect("リトライの設定の小節がある");
    let rest = &text[start..];
    let open = rest.find("```toml\n").expect("TOML の例がある") + "```toml\n".len();
    let body = &rest[open..];
    let close = body.find("\n```").expect("コードブロックが閉じている");
    format!("{}\n", &body[..close])
}

#[test]
fn readme_に_job_retry_と_5_つのキーが書かれている() {
    let text = readme();
    for word in [
        "[job.retry]",
        "exit_codes",
        "max",
        "interval",
        "multiplier",
        "max_interval",
    ] {
        assert!(text.contains(word), "README に {word} がない");
    }
    assert!(text.contains("(v0.8.0 以降)"), "版の表記がない");
}

#[test]
fn 古い一文はそのままの形では残っていない() {
    let text = readme();
    assert!(
        !text.contains("エラーのジョブから再開します。**自動の再試行はしません**。"),
        "古い一文が残っている"
    );
    assert!(
        text.contains("定義に `[job.retry]` を書かなければ、**自動の再試行はしません**"),
        "条件つきの一文がない"
    );
}

#[test]
fn readme_の例の定義を実際の読み取りに通すとリトライの設定が入っている() {
    let example = first_toml_block_after_heading(&readme());
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let path = dir.path().join("conductor.toml");
    std::fs::write(&path, example).expect("書ける");

    let def = conductor_core::load_definition(Some(&path), None).expect("誤りなく読める");
    let retry = def.job[0]
        .retry
        .as_ref()
        .expect("リトライの設定が入っている");
    assert_eq!(retry.exit_codes, vec![75, 111]);
    assert_eq!(retry.max, 5);
    assert_eq!(retry.interval_secs, 10);
    assert_eq!(retry.multiplier, Some(2));
    assert_eq!(retry.max_interval_secs, Some(60));
}

#[test]
fn ci_の_docs_check_の_7_語が_readme_にある() {
    let text = readme();
    for word in [
        "SHA-256",
        "Gatekeeper",
        "終了コードの表",
        "ステートファイルの場所",
        "at-least-once",
        "シグナル",
        "convert",
    ] {
        assert!(text.contains(word), "README に {word} がない");
    }
}
