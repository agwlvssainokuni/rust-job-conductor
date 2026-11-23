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

mod docs_support;

use docs_support::{DOCS_CHECK_WORDS, assert_readme_has, block_after_heading, readme};

/// 小節の見出し(リトライの設定)の行。
const HEADING: &str = "### リトライの設定(`[job.retry]`)";

#[test]
fn readme_に_job_retry_と_5_つのキーが書かれている() {
    let text = readme();
    assert_readme_has(
        &text,
        &[
            "[job.retry]",
            "exit_codes",
            "max",
            "interval",
            "multiplier",
            "max_interval",
        ],
    );
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
    let example = block_after_heading(&readme(), HEADING, "toml");
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
    assert_eq!(retry.backoff.map(|b| b.multiplier), Some(2));
    assert_eq!(retry.backoff.map(|b| b.max_interval_secs), Some(60));
}

#[test]
fn ci_の_docs_check_の_7_語が_readme_にある() {
    let text = readme();
    assert_readme_has(&text, &DOCS_CHECK_WORDS);
}
