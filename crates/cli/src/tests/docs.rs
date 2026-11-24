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

//! README の検査。README を読むだけで、書き込まない。
//!
//! 節は README の並び(リトライ、環境変数、`graph`、`--dry-run`、`init`)。各節は、必須の語句と
//! 版の表記の検査、載っている例を実際の読み取り・解析に通すか、実際の出力と比べるテストを持つ。
//! 最初に、各節が使う補助(README の読み取り、必須語句の検査、見出しのあとの例の取り出し)を置く。

use std::path::{Path, PathBuf};

use crate::report::init_template::render_template;
use crate::types::Language;

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

// ---- リトライの説明(U3。FR8.1、FR6.2) ----

/// 小節の見出し(リトライの設定)の行。
const RETRY_HEADING: &str = "### リトライの設定(`[job.retry]`)";

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
    let example = block_after_heading(&readme(), RETRY_HEADING, "toml");
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

// ---- 環境変数の説明(U1。BR9.2、FR5.2) ----

const ENV_HEADING: &str = "### 環境変数の設定(`env`、`env_file`)(v0.9.0 以降)";

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
    let example = block_after_heading(&text, ENV_HEADING, "toml");
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let path = dir.path().join("conductor.toml");
    std::fs::write(&path, example).expect("書ける");
    let def = conductor_core::load_definition(Some(&path), None).expect("誤りなく読める");
    assert_eq!(def.env_file, [".env"]);
    assert_eq!(def.env[0].name, "TZ");
    assert_eq!(def.job[0].env_file, ["extract.env", "secret.env"]);
    assert_eq!(def.job[0].env[0].name, "LOG_LEVEL");

    let dotenv = block_after_heading(&text, ENV_HEADING, "text");
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

// ---- `graph` の説明(U2。BR7.2、FR5.2) ----

#[test]
fn readme_に_graph_の必須の語句と版の表記がある() {
    let text = readme();
    assert_readme_has(
        &text,
        &[
            "conductor graph",
            "--format mermaid",
            "--state",
            "(v0.9.0 以降)",
            "辺の一覧",
            "flowchart TD",
        ],
    );
    assert!(text.contains("## graph の使い方(v0.9.0 以降)"));
    // docs-check の必須語句は残っている
    assert_docs_check_words(&text);
}

#[test]
fn readme_の辺の一覧に切り替わる形は_4つの種類の外の形だけを述べる() {
    let text = readme();
    let line = text
        .lines()
        .find(|l| l.starts_with("- **辺の一覧に切り替わる形**"))
        .expect("記述がある");
    for word in [
        "飛び越しなし",
        "通常の飛び越し",
        "両方の飛び越し",
        "分岐",
        "どの組み合わせでも",
        "3 本以上",
    ] {
        assert!(line.contains(word), "{word} がない");
    }
    // 前回までの範囲(交差と、両方の飛び越し・分岐の組み合わせを辺の一覧にする)は、述べない(BR4.2)。
    assert!(!line.contains("グラフのどこかに"));
    assert!(!line.contains("同じ行き先で合流する形"));
    assert!(text.contains("`|\\|` の行"));
}

#[test]
fn readme_の図の例は_実際の出力と一致する() {
    let text = readme();
    let start = text.find("## graph の使い方").expect("節がある");
    let rest = &text[start..];
    let open = rest.find("```text\n").expect("例がある") + "```text\n".len();
    let body = &rest[open..];
    let example = &body[..body.find("```").expect("閉じている")];

    // 例の定義: a の成功が d へ飛ぶ 5 個のジョブ。
    let mut definition = String::from("format_version = 1\n\n[pipeline]\nname = \"p\"\n");
    for id in ["a", "b", "c", "d", "e"] {
        definition.push_str(&format!(
            "\n[[job]]\nid = \"{id}\"\ncommand = \"echo {id}\"\n"
        ));
        if id == "a" {
            definition.push_str("\n[[job.transitions]]\non = \"success\"\nnext = \"d\"\n");
        }
    }
    let def = conductor_core::parse_definition(&definition, "c.toml", None).expect("正しい定義");
    let graph = conductor_core::graph::build_graph(&def, None);
    let layout = conductor_core::graph::layout_graph(&graph);
    let actual = crate::report::graph_text::render_text(
        crate::types::Language::En,
        &def,
        &graph,
        &layout,
        false,
    );
    assert_eq!(example, actual);
}

/// 「graph の使い方」の節の `n` 番目(0 始まり)の ```text の例。
fn graph_example(text: &str, n: usize) -> String {
    let start = text.find("## graph の使い方").expect("節がある");
    let end = text[start..]
        .find("## dry-run の使い方")
        .map_or(text.len(), |i| start + i);
    let section = &text[start..end];
    let mut rest = section;
    let mut found = None;
    for _ in 0..=n {
        let open = rest.find("```text\n").expect("例がある") + "```text\n".len();
        let body = &rest[open..];
        let close = body.find("```").expect("閉じている");
        found = Some(body[..close].to_string());
        rest = &body[close..];
    }
    found.unwrap()
}

fn render(definition_text: &str) -> String {
    let def =
        conductor_core::parse_definition(definition_text, "c.toml", None).expect("正しい定義");
    let graph = conductor_core::graph::build_graph(&def, None);
    let layout = conductor_core::graph::layout_graph(&graph);
    crate::report::graph_text::render_text(crate::types::Language::En, &def, &graph, &layout, false)
}

fn jobs(ids: &[&str], extra_for: &[(&str, &str)]) -> String {
    let mut definition = String::from("format_version = 1\n\n[pipeline]\nname = \"p\"\n");
    for id in ids {
        definition.push_str(&format!(
            "\n[[job]]\nid = \"{id}\"\ncommand = \"echo {id}\"\n"
        ));
        for (from, extra) in extra_for {
            if from == id {
                definition.push_str(extra);
            }
        }
    }
    definition
}

#[test]
fn readme_の同じ行き先と両方の飛び越しの例は_実際の出力と一致する() {
    let text = readme();
    let to_d = "\n[[job.transitions]]\non = \"success\"\nnext = \"d\"\n";
    let same_target = jobs(&["a", "b", "c", "d", "e"], &[("a", to_d), ("b", to_d)]);
    assert_eq!(graph_example(&text, 1), render(&same_target));

    let both = "\n[[job.transitions]]\non = \"warning\"\nnext = \"c\"\n\n[[job.transitions]]\non = \"success\"\nnext = \"c\"\n";
    let both_skip = jobs(&["a", "b", "c", "d"], &[("a", both)]);
    assert_eq!(graph_example(&text, 2), render(&both_skip));
}

#[test]
fn readme_の分岐の例は_実際の出力と一致する() {
    let text = readme();
    let branch = "\n[[job.transitions]]\non = \"success\"\nnext = \"c\"\n\n[[job.transitions]]\non = \"warning\"\nnext = \"d\"\n";
    let definition = jobs(&["a", "b", "c", "d", "e"], &[("a", branch)]);
    assert_eq!(graph_example(&text, 3), render(&definition));
}

#[test]
fn readme_の交差の例は_実際の出力と一致する() {
    let text = readme();
    let to = |t: &str| format!("\n[[job.transitions]]\non = \"success\"\nnext = \"{t}\"\n");
    let definition = jobs(
        &["a", "b", "c", "d", "e"],
        &[("a", &to("d")), ("b", &to("e"))],
    );
    assert_eq!(graph_example(&text, 4), render(&definition));
}

#[test]
fn readme_の交差に同じ行き先が混ざる例は_実際の出力と一致する() {
    let text = readme();
    let to = |t: &str| format!("\n[[job.transitions]]\non = \"success\"\nnext = \"{t}\"\n");
    let definition = jobs(
        &["a", "b", "c", "d", "e"],
        &[("a", &to("e")), ("b", &to("d")), ("c", &to("e"))],
    );
    assert_eq!(graph_example(&text, 5), render(&definition));
}

#[test]
fn readme_の両方の飛び越しと交差が混ざる例は_実際の出力と一致する() {
    let text = readme();
    let both_to_d = "\n[[job.transitions]]\non = \"warning\"\nnext = \"d\"\n\n[[job.transitions]]\non = \"success\"\nnext = \"d\"\n";
    let to_e = "\n[[job.transitions]]\non = \"success\"\nnext = \"e\"\n";
    let definition = jobs(
        &["a", "b", "c", "d", "e", "f"],
        &[("a", both_to_d), ("b", to_e)],
    );
    assert_eq!(graph_example(&text, 6), render(&definition));
}

// ---- `--dry-run` の説明(U3。BR6.2、FR5.2) ----

#[test]
fn readme_に_dry_run_の必須の語句と版の表記がある() {
    let text = readme();
    assert_readme_has(
        &text,
        &[
            "--dry-run",
            "--assume",
            "## dry-run の使い方(v0.9.0 以降)",
            "使われなかった仮定",
            "計画が示さないこと",
            "再開位置",
        ],
    );
    // docs-check の必須語句は残っている
    assert_docs_check_words(&text);
}

#[test]
fn readme_の計画の例は_実際の出力と一致する() {
    let text = readme();
    let start = text.find("## dry-run の使い方").expect("節がある");
    let rest = &text[start..];
    let open = rest.find("```text\n").expect("例がある") + "```text\n".len();
    let body = &rest[open..];
    let example = &body[..body.find("```").expect("閉じている")];

    // 例の定義: a が終了コード 3 の警告で `@end` へ進む。`--assume a=3`。
    let definition = "format_version = 1\n\n[pipeline]\nname = \"demo\"\n\n\
        [[job]]\nid = \"a\"\ncommand = \"echo a\"\nwarning_exit_codes = [3]\n\n\
        [[job.transitions]]\non = \"warning\"\nnext = \"@end\"\n\n\
        [[job]]\nid = \"b\"\ncommand = \"echo b\"\n\n\
        [[job]]\nid = \"c\"\ncommand = \"echo c\"\n";
    let def = conductor_core::parse_definition(definition, "c.toml", None).expect("正しい定義");
    let commands: Vec<Vec<String>> = def
        .job
        .iter()
        .map(|j| vec!["echo".to_string(), j.id.clone()])
        .collect();
    let files = conductor_core::LoadedEnvFiles::new();
    let assumptions = [conductor_core::Assumption::new("a", 3)];
    let plan = conductor_core::plan_pipeline(&conductor_core::PlanInput {
        definition: &def,
        state: None,
        commands: &commands,
        env_files: &files,
        run_id: None,
        assumptions: &assumptions,
    })
    .expect("計画");
    let actual = crate::report::plan_text::render_plan(crate::types::Language::Ja, &plan);
    assert_eq!(example, actual);
}

// ---- `init` の説明(U4。BR4.2、FR5.2) ----

#[test]
fn readme_に_init_の必須の語句と版の表記と終了コード_69_がある() {
    let text = readme();
    assert_readme_has(
        &text,
        &[
            "conductor init",
            "--force",
            "--config -",
            "## init の使い方(v0.9.0 以降)",
            "| 69 |",
            "既にあります",
        ],
    );
    // docs-check の必須語句は残っている
    assert_docs_check_words(&text);
}

#[test]
fn readme_のひな形の例は_実際の日本語のひな形と一致する() {
    let text = readme();
    let start = text.find("## init の使い方").expect("節がある");
    let rest = &text[start..];
    let open = rest.find("```toml\n").expect("例がある") + "```toml\n".len();
    let body = &rest[open..];
    let example = &body[..body.find("```").expect("閉じている")];
    assert_eq!(example, render_template(Language::Ja, "my-pipeline"));
}
