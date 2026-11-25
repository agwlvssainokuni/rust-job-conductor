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

//! `graph` の流れ(偽の `Runtime`)。BR4.3〜BR4.6、BR6.1〜BR6.4。FR2.1、FR2.6、FR2.7。

mod common;

use common::*;

/// a の成功が d へ飛ぶ 5 個のジョブ(例 G2)。
const G2: &str = r#"format_version = 1

[pipeline]
name = "p"

[[job]]
id = "a"
command = "echo a"

[[job.transitions]]
on = "success"
next = "d"

[[job]]
id = "b"
command = "echo b"

[[job]]
id = "c"
command = "echo c"

[[job]]
id = "d"
command = "echo d"

[[job]]
id = "e"
command = "echo e"
"#;

const G2_PLAIN: &str = "*   a\n|\\\n* | b\n* | c\n|/\n* d\n* e\no @end\n";

/// 並びの違う定義(ジョブの並びが食い違う)。
const REORDERED: &str = r#"format_version = 1

[pipeline]
name = "p"

[[job]]
id = "b"
command = "echo b"

[[job]]
id = "a"
command = "echo a"
"#;

#[test]
fn graph_はジョブを起動せず_ステートに触れず_標準出力に図だけを1回で書く() {
    let mut h = Harness::new(G2);
    let out = h.run(&["graph"]);
    assert_eq!(out.code, 0, "{}", out.err);
    assert_eq!(out.out, G2_PLAIN);
    assert_eq!(out.err, "");
    assert_eq!(h.sink.out_writes, 1, "標準出力へは 1 回で書く");
    assert_eq!(h.runtime.run_calls.get(), 0);
    assert_eq!(h.runtime.write_calls.get(), 0);
    assert!(!h.state_path().exists());
    // `-v` なしでも、ある場合でも、ステートは作らない。Mermaid も同じ。
    let out = h.run(&["graph", "--format", "mermaid", "-v"]);
    assert_eq!(out.code, 0);
    assert!(out.out.starts_with("flowchart TD\n"));
    assert_eq!(h.runtime.run_calls.get(), 0);
    assert!(!h.state_path().exists());
}

fn with_job_extra(extra: &str) -> String {
    format!("{G2}\n[[job]]\nid = \"z\"\ncommand = \"echo z\"\n{extra}")
}

#[test]
fn validate_が_64_になる定義は_graph_も_64_で_図を出さず_同じ誤りを出す() {
    let unclosed = "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"echo 'abc\"\n";
    let bad_target = G2.replace("next = \"d\"", "next = \"nowhere\"");
    let env_error = with_job_extra("env_file = [\"missing.env\"]\n");
    // v0.10.0 の U3: `.env` の名前と `unset_env` の重なり(BR2.2)。
    let unset_conflict = with_job_extra("env_file = \"z.env\"\nunset_env = [\"Z_TOKEN\"]\n");
    for definition in [unclosed.to_string(), bad_target, env_error, unset_conflict] {
        let mut h = Harness::new(&definition);
        std::fs::write(h.dir.path().join("z.env"), "z_token=1\n").expect("書ける");
        let validate = h.run(&["validate"]);
        assert_eq!(validate.code, 64, "{}", validate.err);
        let graph = h.run(&["graph"]);
        assert_eq!(graph.code, 64, "{}", graph.err);
        assert_eq!(graph.out, "");
        assert_eq!(graph.err, validate.err, "validate と同じ誤り");
        let graph = h.run(&["graph", "--format", "mermaid", "--state"]);
        assert_eq!(graph.code, 64);
        assert_eq!(graph.out, "");
    }
}

#[test]
fn パイプライン名の不一致は_64() {
    let mut h = Harness::new(G2);
    let out = h.run(&["graph", "other"]);
    assert_eq!(out.code, 64, "{}", out.err);
    assert_eq!(out.out, "");
    assert_eq!(h.run(&["graph", "p"]).out, G2_PLAIN);
}

#[test]
fn state_のステートが定義と食い違えば_65_で_完了したステートにも照合する() {
    let mut h = Harness::new(G2);
    // 途中のステート(d でエラー)。
    h.runtime.script("d", &[Exec::Exit(1)]);
    assert_ne!(h.run(&["pipeline", "p"]).code, 0);
    h.set_definition(REORDERED);
    let out = h.run(&["graph", "--state"]);
    assert_eq!(out.code, 65, "{}", out.err);
    assert_eq!(out.out, "");
    assert!(out.err.contains("does not match"), "{}", out.err);

    // 完了したステートにも照合する。
    let mut h = Harness::new(G2);
    assert_eq!(h.run(&["pipeline", "p"]).code, 0);
    assert!(h.state_raw().is_some_and(|s| s.completed));
    h.set_definition(REORDERED);
    let out = h.run(&["graph", "--state"]);
    assert_eq!(out.code, 65, "{}", out.err);
    assert_eq!(out.out, "");
    // `--state` なしなら、ステートは見ないので、図が出る。
    assert_eq!(h.run(&["graph"]).code, 0);
}

#[test]
fn 使い方の誤りは_68_で_ジョブを起動しない() {
    let mut h = Harness::new(G2);
    for args in [
        &["graph", "--run-id", "r1"][..],
        &["graph", "--format", "svg"][..],
        &["graph", "--format", "text", "--format", "text"][..],
        &["graph", "--state", "--state"][..],
        &["run", "a", "--format", "text"][..],
        &["pipeline", "p", "--state"][..],
        &["watch", "--state"][..],
    ] {
        let out = h.run(args);
        assert_eq!(out.code, 68, "{args:?}: {}", out.err);
        assert_eq!(out.out, "");
        assert!(out.err.contains("Usage:"), "{}", out.err);
    }
    assert_eq!(h.runtime.run_calls.get(), 0);
    assert!(!h.state_path().exists());
}

#[test]
fn state_でステートがなければ_すべて_未実行() {
    let mut h = Harness::new(G2);
    let out = h.run(&["graph", "--state"]);
    assert_eq!(out.code, 0, "{}", out.err);
    assert_eq!(
        out.out,
        "*   [  ] a\n|\\\n* | [  ] b\n* | [  ] c\n|/\n* [  ] d\n* [  ] e\no @end\n"
    );
    assert!(!out.out.contains("resume here"));
    // 長さ 0 のステートも「ない」と同じ。
    std::fs::write(h.state_path(), "").unwrap();
    assert_eq!(h.run(&["graph", "--state"]).out, out.out);
    // run ID つき: そのステートがなければ、すべて未実行。
    let with_id = h.run(&["graph", "--state", "--run-id", "r1"]);
    assert_eq!(with_id.code, 0, "{}", with_id.err);
    assert_eq!(with_id.out, out.out);
}

#[test]
fn state_は_ステートの状態と再開位置を重ね_完了したステートには再開位置がない() {
    let mut h = Harness::new(G2);
    // a は成功で d へ飛び(b、c はスキップ)、d でエラーにして止める。
    h.runtime.script("a", &[Exec::Exit(0)]);
    h.runtime.script("d", &[Exec::Exit(1)]);
    assert_ne!(h.run(&["pipeline", "p"]).code, 0);
    let before = std::fs::read(h.state_path()).unwrap();
    let writes = h.runtime.write_calls.get();
    let out = h.run(&["graph", "--state"]);
    assert_eq!(out.code, 0, "{}", out.err);
    assert_eq!(
        out.out,
        "*   [OK] a\n|\\\n* | [--] b\n* | [--] c\n|/\n* [NG] d  <- resume here\n* [  ] e\no @end\n"
    );
    assert_eq!(
        std::fs::read(h.state_path()).unwrap(),
        before,
        "ステートは読むだけ"
    );
    assert_eq!(h.runtime.write_calls.get(), writes);
    // Mermaid の再開位置。
    let mermaid = h.run(&["graph", "--state", "--format", "mermaid"]);
    assert!(mermaid.out.contains("n3[\"[NG] d\"]"), "{}", mermaid.out);
    assert!(mermaid.out.ends_with("  class n3 resume\n"));

    // 完了したステート(d を成功にして完了させる)。
    let mut h = Harness::new(G2);
    assert_eq!(h.run(&["pipeline", "p"]).code, 0);
    assert!(h.state_raw().is_some_and(|s| s.completed));
    let out = h.run(&["graph", "--state"]);
    assert_eq!(out.code, 0, "{}", out.err);
    assert_eq!(
        out.out,
        "*   [OK] a\n|\\\n* | [--] b\n* | [--] c\n|/\n* [OK] d\n* [OK] e\no @end\n"
    );
    // run ID つき。
    h.env = FakeEnv::with(&[("CONDUCTOR_RUN_ID", "r9")]);
    assert_eq!(h.run(&["pipeline", "p"]).code, 0);
    assert!(h.state_raw_of("r9").is_some());
    assert!(h.run(&["graph", "--state"]).out.contains("[OK] a"));
}

#[test]
fn 標準出力に書けなければ_65() {
    let mut h = Harness::new(G2);
    h.sink.fail_out = true;
    let out = h.run(&["graph"]);
    assert_eq!(out.code, 65);
    assert_eq!(out.out, "");
    assert!(
        out.err.contains("cannot write to standard output"),
        "{}",
        out.err
    );
    assert_eq!(h.runtime.run_calls.get(), 0);
}

// ---- `--format dot`(U4。BR2.1、BR2.5、NFR6.3) ----

#[test]
fn format_dot_は_0_で終わり_標準出力にdotを1回で書き_v_で変わらない() {
    let mut h = Harness::new(G2);
    let out = h.run(&["graph", "--format", "dot"]);
    assert_eq!(out.code, 0, "{}", out.err);
    assert_eq!(out.err, "");
    assert!(out.out.starts_with("digraph conductor {\n  rankdir=TB;\n"));
    assert!(out.out.ends_with("  n4 -> nend;\n}\n"), "{}", out.out);
    assert_eq!(h.sink.out_writes, 1, "標準出力へは 1 回で書く");
    assert_eq!(h.runtime.run_calls.get(), 0);
    assert!(!h.state_path().exists());
    // `-v` は DOT に影響しない(BR2.5)。
    let verbose = h.run(&["graph", "--format", "dot", "-v"]);
    assert_eq!(verbose.code, 0);
    assert_eq!(verbose.out, out.out);
    // `--state` は、記号と再開位置の太い枠。
    h.runtime.script("d", &[Exec::Exit(1)]);
    assert_ne!(h.run(&["pipeline", "p"]).code, 0);
    let state = h.run(&["graph", "--format", "dot", "--state"]);
    assert_eq!(state.code, 0, "{}", state.err);
    assert!(
        state
            .out
            .contains("  n3 [label=\"[NG] d\", shape=box, penwidth=3];\n"),
        "{}",
        state.out
    );
}

#[test]
fn 誤りのある定義は_format_dot_でも_text_と同じ標準エラーと終了コード() {
    let unclosed = "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"echo 'abc\"\n";
    let bad_target = G2.replace("next = \"d\"", "next = \"nowhere\"");
    for definition in [unclosed.to_string(), bad_target] {
        let mut h = Harness::new(&definition);
        let text = h.run(&["graph", "--format", "text"]);
        let dot = h.run(&["graph", "--format", "dot"]);
        assert_eq!(dot.code, text.code);
        assert_eq!(dot.code, 64, "{}", dot.err);
        assert_eq!(dot.out, "");
        assert_eq!(dot.err, text.err);
    }
    // ステートが定義と食い違うときも同じ(65)。
    let mut h = Harness::new(G2);
    h.runtime.script("d", &[Exec::Exit(1)]);
    assert_ne!(h.run(&["pipeline", "p"]).code, 0);
    h.set_definition(REORDERED);
    let text = h.run(&["graph", "--state"]);
    let dot = h.run(&["graph", "--state", "--format", "dot"]);
    assert_eq!(dot.code, 65, "{}", dot.err);
    assert_eq!(dot.out, "");
    assert_eq!(dot.err, text.err);
}
