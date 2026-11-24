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

//! `validate`(定義の検証と全ジョブの `command` の検査だけを行う補助のコマンド)の流れ。FR1.1〜FR1.8。

mod common;

use common::*;

const UNCLOSED_QUOTE: &str = r#"format_version = 1

[pipeline]
name = "p"

[[job]]
id = "a"
command = "echo 'abc"
"#;

const BAD_TARGET: &str = r#"format_version = 1

[pipeline]
name = "p"

[[job]]
id = "a"
command = "echo a"

[[job.transitions]]
on = "success"
next = "nowhere"
"#;

const MISSING_PROGRAM: &str = r#"format_version = 1

[pipeline]
name = "p"

[[job]]
id = "a"
command = "/no/such/program-for-conductor-test --flag"
"#;

/// リトライの設定を持つ定義。`retry` に、ジョブ a の `[job.retry]` の本体を書く(8 行目から)。
fn with_retry(retry_body: &str) -> String {
    format!(
        "format_version = 1\n\n[pipeline]\nname = \"p\"\n\n[[job]]\nid = \"a\"\ncommand = \"echo a\"\n\n[job.retry]\n{retry_body}"
    )
}

#[test]
fn 正しい_retry_を持つ定義は_0_で_何も表示しない() {
    let mut h = Harness::new(&with_retry(
        "exit_codes = [75]\nmax = 3\ninterval = \"30s\"\nmultiplier = 2\nmax_interval = \"10m\"\n",
    ));
    let out = h.run(&["validate"]);
    assert_eq!(out.code, 0, "{}", out.err);
    assert_eq!(out.out, "");
    assert_eq!(out.err, "");
    assert_eq!(h.runtime.run_calls.get(), 0);
    assert!(!h.state_path().exists());
}

#[test]
fn retry_の誤りは_64_で_run_と同じ誤りの文() {
    let mut h = Harness::new(&with_retry(
        "exit_codes = [75]\nmax = 0\ninterval = \"30s\"\n",
    ));
    let validate = h.run(&["validate"]);
    assert_eq!(validate.code, 64);
    assert_eq!(validate.out, "");
    assert!(
        validate.err.starts_with("conductor: error: "),
        "{}",
        validate.err
    );
    assert!(validate.err.contains(":12:"), "行番号: {}", validate.err);
    let run = h.run(&["run", "a"]);
    assert_eq!(run.code, 64);
    assert_eq!(validate.err, run.err, "run と同じ文");
    assert_eq!(h.runtime.run_calls.get(), 0, "ジョブは実行しない");
}

#[test]
fn retry_の誤りは複数あれば_1_行ずつ_位置の順() {
    let mut h = Harness::new(&with_retry(
        "exit_codes = [0]\nmax = 101\ninterval = \"25h\"\n",
    ));
    let out = h.run(&["validate"]);
    assert_eq!(out.code, 64);
    let lines: Vec<&str> = out.err.lines().collect();
    assert_eq!(lines.len(), 3, "{}", out.err);
    assert!(lines[0].contains(":11:"), "{}", out.err);
    assert!(lines[1].contains(":12:"), "{}", out.err);
    assert!(lines[2].contains(":13:"), "{}", out.err);
}

#[test]
fn retry_の許可されていないキーは_64() {
    let mut h = Harness::new(&with_retry(
        "exit_codes = [75]\nmax = 3\ninterval = \"30s\"\nwait = 1\n",
    ));
    let out = h.run(&["validate"]);
    assert_eq!(out.code, 64);
    assert!(out.err.contains("unknown key"), "{}", out.err);
}

#[test]
fn 正しい定義は_0_で_何も表示せず_ステートにもジョブにもシグナルにも触れない() {
    let mut h = Harness::new(THREE_JOBS);
    let out = h.run(&["validate"]);
    assert_eq!(out.code, 0);
    assert_eq!(out.out, "");
    assert_eq!(out.err, "");
    assert_eq!(h.runtime.log(), ["load"], "定義を読むだけ");
    assert_eq!(h.runtime.run_calls.get(), 0);
    assert_eq!(h.signals.started.get(), 0, "シグナルの準備をしない");
    assert!(!h.state_path().exists());
}

#[test]
fn 定義の誤りは_64_で_run_と同じ誤りの文() {
    let mut h = Harness::new(BAD_TARGET);
    let validate = h.run(&["validate"]);
    assert_eq!(validate.code, 64);
    assert_eq!(validate.out, "");
    assert!(
        validate.err.starts_with("conductor: error: "),
        "{}",
        validate.err
    );
    let run = h.run(&["run", "a"]);
    assert_eq!(run.code, 64);
    assert_eq!(validate.err, run.err, "run と同じ文");
    assert_eq!(h.runtime.run_calls.get(), 0);
}

#[test]
fn command_の誤りは_64_で_run_と同じ誤りの文() {
    let mut h = Harness::new(UNCLOSED_QUOTE);
    let validate = h.run(&["validate"]);
    assert_eq!(validate.code, 64);
    assert_eq!(validate.out, "");
    let run = h.run(&["run", "a"]);
    assert_eq!(run.code, 64);
    assert_eq!(validate.err, run.err, "run と同じ文");
    assert!(validate.err.contains("job 'a'"), "{}", validate.err);
    assert!(
        !validate.err.contains("abc"),
        "入力の値を含めない: {}",
        validate.err
    );
}

#[test]
fn 空白だけのコマンド名は_64_で_ジョブを起動しない() {
    // 文字列の command の先頭の語が、引用符の中の空白だけ(2c)。
    let mut h = Harness::new(
        "format_version = 1\n\n[pipeline]\nname = \"p\"\n\n[[job]]\nid = \"a\"\ncommand = \"'   ' x\"\n",
    );
    h.env = FakeEnv::for_language(common::Language::En);
    let validate = h.run(&["validate"]);
    assert_eq!(validate.code, 64, "{}", validate.err);
    assert_eq!(validate.out, "");
    assert!(
        validate.err.contains("command is empty"),
        "{}",
        validate.err
    );
    assert!(validate.err.contains("job 'a'"), "{}", validate.err);
    assert!(
        !validate.err.contains("'   '"),
        "入力の値を含めない: {}",
        validate.err
    );
    let run = h.run(&["run", "a"]);
    assert_eq!(run.code, 64, "{}", run.err);
    assert_eq!(validate.err, run.err, "run と同じ文");
    assert_eq!(h.runtime.run_calls.get(), 0, "ジョブを起動しない");
}

#[test]
fn パイプライン名は_一致で_0_不一致で_64_省略で照合しない() {
    let mut h = Harness::new(THREE_JOBS);
    assert_eq!(h.run(&["validate", "p"]).code, 0);
    let mismatch = h.run(&["validate", "other"]);
    assert_eq!(mismatch.code, 64);
    assert!(
        !mismatch.err.contains("other"),
        "入力の値を含めない: {}",
        mismatch.err
    );
    assert_eq!(h.run(&["validate"]).code, 0, "省略は照合しない");
}

#[test]
fn v_は標準エラーに_1_行だけ出す() {
    let mut h = Harness::new(THREE_JOBS);
    let out = h.run(&["validate", "-v"]);
    assert_eq!(out.code, 0);
    assert_eq!(out.out, "");
    assert_eq!(out.err, "[conductor] definition is valid (3 jobs)\n");

    h.env.vars.insert("LANG".into(), "ja_JP.UTF-8".into());
    let out = h.run(&["validate", "-v"]);
    assert_eq!(out.err, "[conductor] 定義は正しいです(ジョブ 3 個)\n");
}

#[test]
fn 起動できないプログラムの_command_でも_0() {
    let mut h = Harness::new(MISSING_PROGRAM);
    let out = h.run(&["validate"]);
    assert_eq!(out.code, 0);
    assert_eq!(out.err, "");
    assert_eq!(h.runtime.run_calls.get(), 0);
}

#[test]
fn ステートが壊れていても_ステートを読まず_0() {
    let mut h = Harness::new(THREE_JOBS);
    std::fs::write(h.state_path(), "garbage = = =").expect("書ける");
    let out = h.run(&["validate"]);
    assert_eq!(out.code, 0);
    assert_eq!(out.err, "");
    assert_eq!(h.runtime.log(), ["load"]);
    assert_eq!(
        std::fs::read_to_string(h.state_path()).expect("残っている"),
        "garbage = = ="
    );
}

#[test]
fn 位置引数が_2_個や_to_は使い方の誤りで_68() {
    let mut h = Harness::new(THREE_JOBS);
    for args in [
        &["validate", "a", "b"][..],
        &["validate", "--to", "yaml"][..],
    ] {
        let out = h.run(args);
        assert_eq!(out.code, 68, "{args:?}");
        assert!(out.err.contains("Usage:"), "{args:?}");
    }
    assert_eq!(h.runtime.log(), Vec::<String>::new());
}
