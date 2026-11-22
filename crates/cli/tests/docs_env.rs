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

//! README の環境変数の説明の検査(U1。BR9.2、FR5.2)。
//!
//! README を読むだけで、書き込まない。例は、実際の読み取りと解析に通す。

mod docs_support;

use docs_support::{assert_docs_check_words, assert_readme_has, block_after_heading, readme};

const HEADING: &str = "### 環境変数の設定(`env`、`env_file`)(v0.9.0 以降)";

#[test]
fn readme_に必須の語句と版の表記がある() {
    let text = readme();
    assert_readme_has(
        &text,
        &[
            "env_file",
            "CONDUCTOR_ATTEMPT",
            "CONDUCTOR_PIPELINE",
            "CONDUCTOR_JOB_ID",
            "CONDUCTOR_RUN_ID",
            "[pipeline.env]",
            "[job.env]",
            "(v0.9.0 以降)",
            "1 MiB",
            "10,000",
        ],
    );
    // docs-check の必須語句は残っている
    assert_docs_check_words(&text);
}

#[test]
fn readme_の例の定義と_env_を実際の読み取りと解析に通せる() {
    let text = readme();
    let example = block_after_heading(&text, HEADING, "toml");
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let path = dir.path().join("conductor.toml");
    std::fs::write(&path, example).expect("書ける");
    let def = conductor_core::load_definition(Some(&path), None).expect("誤りなく読める");
    assert_eq!(def.env_file, [".env"]);
    assert_eq!(def.env[0].name, "TZ");
    assert_eq!(def.job[0].env_file, ["extract.env", "secret.env"]);
    assert_eq!(def.job[0].env[0].name, "LOG_LEVEL");

    let dotenv = block_after_heading(&text, HEADING, "text");
    let content = conductor_core::parse_env_file(&dotenv, ".env").expect("誤りなく解析できる");
    let get = |n: &str| {
        content
            .entries
            .iter()
            .find(|e| e.name == n)
            .map(|e| e.value.clone())
    };
    assert_eq!(get("GREETING").as_deref(), Some("hello\nworld"));
    assert_eq!(get("RAW").as_deref(), Some("no $expansion here"));
    assert_eq!(get("EMPTY").as_deref(), Some(""));
    assert_eq!(get("DB_PORT").as_deref(), Some("5432"));
}
