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

//! `--dry-run` の流れ(偽の `Runtime`。U3。BR1.1、BR1.8、BR2.1、BR2.2、BR4.1〜BR4.5、BR3.4)。

mod common;

use common::*;
use conductor_runtime::StateErrorKind;

/// b、a の順の定義(`THREE_JOBS` で作ったステートと、ジョブの並びが食い違う)。
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

/// 計画の行から、実行するジョブの ID を、並び順に取り出す(英語の表示。`N. [run...] <id>`)。
fn will_run_ids(plan: &str) -> Vec<String> {
    plan.lines()
        .filter(|l| l.chars().next().is_some_and(|c| c.is_ascii_digit()))
        .filter_map(|l| {
            let (head, id) = l.rsplit_once("] ")?;
            let head = head.split_once(". [")?.1;
            let id = id.trim_end_matches("  <- resume here");
            (head == "run" || head.starts_with("run (")).then(|| id.to_string())
        })
        .collect()
}

#[test]
fn dry_run_はジョブを起動せず_ステートに書かず_消さず_シグナルも準備しない() {
    for args in [
        &["pipeline", "p", "--dry-run"][..],
        &["run", "a", "--dry-run"][..],
    ] {
        let mut h = Harness::new(THREE_JOBS);
        let out = h.run(args);
        assert_eq!(out.code, 0, "{args:?}: {}", out.err);
        assert!(
            out.out.starts_with("plan (--dry-run): pipeline p\n"),
            "{}",
            out.out
        );
        assert_eq!(out.err, "");
        assert_eq!(h.sink.out_writes, 1, "標準出力へは 1 回で書く");
        assert_eq!(h.runtime.run_calls.get(), 0);
        assert_eq!(h.runtime.write_calls.get(), 0);
        assert!(!h.runtime.log().iter().any(|l| l == "reset"), "削除しない");
        assert_eq!(h.signals.started.get(), 0, "シグナルを準備しない");
        assert!(!h.state_path().exists());
    }
}

#[test]
fn 定義と_command_と_env_の誤りは_64_で_計画を出さない() {
    let unclosed = "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"echo 'abc\"\n";
    let env_error = format!(
        "{THREE_JOBS}\n[[job]]\nid = \"z\"\ncommand = \"echo z\"\nenv_file = \"missing.env\"\n"
    );
    let bad_target = THREE_JOBS.replace(
        "command = \"echo b\"",
        "command = \"echo b\"\n[[job.transitions]]\non = \"success\"\nnext = \"nowhere\"",
    );
    for definition in [unclosed.to_string(), env_error, bad_target] {
        let mut h = Harness::new(&definition);
        let out = h.run(&["pipeline", "p", "--dry-run"]);
        assert_eq!(out.code, 64, "{}", out.err);
        assert_eq!(out.out, "");
        assert!(!out.err.is_empty());
        // 本番と同じ誤りの表示。
        let real = h.run(&["pipeline", "p"]);
        assert_eq!(real.code, 64);
        assert_eq!(out.err, real.err);
        assert_eq!(h.runtime.write_calls.get(), 0);
    }
}

#[test]
fn ステートの食い違いと読めないステートは_65() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime.script("b", &[Exec::Exit(1)]);
    assert_ne!(h.run(&["pipeline", "p"]).code, 0);
    h.set_definition(REORDERED);
    let out = h.run(&["pipeline", "p", "--dry-run"]);
    assert_eq!(out.code, 65, "{}", out.err);
    assert_eq!(out.out, "");
    assert!(out.err.contains("does not match"), "{}", out.err);

    let mut h = Harness::new(THREE_JOBS);
    *h.runtime.read_failure.borrow_mut() =
        Some(state_error(&h.state_path(), StateErrorKind::Corrupt));
    for args in [
        &["pipeline", "p", "--dry-run"][..],
        &["run", "a", "--dry-run"][..],
    ] {
        let out = h.run(args);
        assert_eq!(out.code, 65, "{}", out.err);
        assert_eq!(out.out, "");
    }
    assert_eq!(h.runtime.write_calls.get(), 0);
}

#[test]
fn 順序外の_run_は_66_で_本番と同じ表示() {
    let mut h = Harness::new(THREE_JOBS);
    let dry = h.run(&["run", "b", "--dry-run"]);
    assert_eq!(dry.code, 66, "{}", dry.err);
    assert_eq!(dry.out, "");
    let real = h.run(&["run", "b"]);
    assert_eq!(real.code, 66);
    assert_eq!(dry.err, real.err, "本番と同じ拒否の表示");
    assert_eq!(h.runtime.write_calls.get(), 0);
}

#[test]
fn assume_の不正な入力は_68_で_誤りの表示に入力の値を含まない() {
    let mut h = Harness::new(THREE_JOBS);
    for bad in [
        "job-secret-id",
        "job-secret-id=x",
        "job-secret-id=256",
        "job-secret-id=-1",
        "job-secret-id=1",
    ] {
        // 最後の 1 つは、定義にないジョブ。
        let out = h.run(&["pipeline", "p", "--dry-run", "--assume", bad]);
        assert_eq!(out.code, 68, "{bad}: {}", out.err);
        assert_eq!(out.out, "");
        assert!(!out.err.contains("job-secret-id"), "{}", out.err);
    }
    // 重複。
    let out = h.run(&[
        "pipeline",
        "p",
        "--dry-run",
        "--assume",
        "a=1",
        "--assume",
        "a=2",
    ]);
    assert_eq!(out.code, 68, "{}", out.err);
    // 定義にないジョブは、`run` でも 68。
    let out = h.run(&["run", "a", "--dry-run", "--assume", "job-secret-id=1"]);
    assert_eq!(out.code, 68);
    assert!(!out.err.contains("job-secret-id"), "{}", out.err);
    assert_eq!(h.runtime.run_calls.get(), 0);
    assert_eq!(h.runtime.write_calls.get(), 0);
}

#[test]
fn assume_だけと_dry_run_の付ける場所の誤りは_68() {
    let mut h = Harness::new(THREE_JOBS);
    assert_eq!(h.run(&["pipeline", "p", "--assume", "a=1"]).code, 68);
    assert_eq!(h.run(&["run", "a", "--assume", "a=1"]).code, 68);
    for args in [
        &["validate", "--dry-run"][..],
        &["graph", "--dry-run"][..],
        &["watch", "--dry-run"][..],
        &["reset", "--dry-run"][..],
    ] {
        let out = h.run(args);
        assert_eq!(out.code, 68, "{args:?}: {}", out.err);
    }
    assert_eq!(h.runtime.run_calls.get(), 0);
}

#[test]
fn 計画を出せたら_エラーで停止する模擬でも_0_で_出力できなければ_65() {
    let mut h = Harness::new(THREE_JOBS);
    let out = h.run(&["pipeline", "p", "--dry-run", "--assume", "b=1"]);
    assert_eq!(out.code, 0, "{}", out.err);
    assert!(
        out.out.contains("result (simulated): stopped (error: b)\n"),
        "{}",
        out.out
    );
    assert_eq!(will_run_ids(&out.out), ["a", "b"]);

    h.sink.fail_out = true;
    let out = h.run(&["pipeline", "p", "--dry-run"]);
    assert_eq!(out.code, 65, "{}", out.err);
    assert!(out.out.is_empty());
    assert!(!out.err.is_empty());
}

/// 同じ定義・同じステート・仮定どおりの終了コードを返す偽の `Runtime`(リトライなし、中断なし)で、
/// `--dry-run` の「実行する」の並びと、本番の `pipeline` が起動するジョブの並びが一致する(BR1.8)。
#[test]
fn 計画の実行するジョブは_本番の流れが起動するジョブと一致する() {
    let end_after_a = THREE_JOBS.replace(
        "command = \"echo a\"",
        "command = \"echo a\"\n[[job.transitions]]\non = \"success\"\nnext = \"@end\"",
    );
    type Case<'a> = (&'a str, String, Vec<(&'a str, i32)>);
    let cases: Vec<Case> = vec![
        ("すべて成功", THREE_JOBS.to_string(), vec![]),
        ("b でエラー", THREE_JOBS.to_string(), vec![("b", 1)]),
        ("警告で c へ", WARNING_JUMPS.to_string(), vec![("a", 1)]),
        (
            "警告でも進む",
            WARNING_JUMPS.to_string(),
            vec![("a", 1), ("c", 1)],
        ),
        ("@end", end_after_a, vec![]),
        ("成功コードにない 0", with_success_codes(), vec![]),
    ];
    for (name, definition, assumed) in cases {
        let mut h = Harness::new(&definition);
        let mut args: Vec<String> = ["pipeline", "p", "--dry-run"].map(String::from).to_vec();
        for (id, code) in &assumed {
            args.push("--assume".into());
            args.push(format!("{id}={code}"));
            h.runtime.script(id, &[Exec::Exit(*code)]);
        }
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let plan = h.run(&refs);
        assert_eq!(plan.code, 0, "{name}: {}", plan.err);
        assert_eq!(h.runtime.run_calls.get(), 0, "{name}");
        // 本番(`--dry-run` なし)。
        h.run(&["pipeline", "p"]);
        assert_eq!(
            h.runtime.ran(),
            will_run_ids(&plan.out),
            "{name}\n{}",
            plan.out
        );
    }
}

fn with_success_codes() -> String {
    THREE_JOBS.replace(
        "command = \"echo a\"",
        "command = \"echo a\"\nsuccess_exit_codes = [10]",
    )
}

#[test]
fn 前回の停止からの再開も_本番の流れが起動するジョブと一致する() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime.script("b", &[Exec::Exit(1)]);
    assert_ne!(h.run(&["pipeline", "p"]).code, 0);
    let before = h.runtime.ran();
    assert_eq!(before, ["a", "b"]);
    let state_bytes = std::fs::read(h.state_path()).expect("ステートがある");

    let plan = h.run(&["pipeline", "p", "--dry-run"]);
    assert_eq!(plan.code, 0, "{}", plan.err);
    assert!(
        plan.out.contains("resume: b (previous stop)\n"),
        "{}",
        plan.out
    );
    assert!(
        plan.out.contains("1. [already executed] a\n"),
        "{}",
        plan.out
    );
    assert!(
        plan.out.contains("2. [run] b  <- resume here\n"),
        "{}",
        plan.out
    );
    assert_eq!(
        std::fs::read(h.state_path()).expect("ステートがある"),
        state_bytes,
        "ステートは変わらない"
    );

    assert_eq!(h.run(&["pipeline", "p"]).code, 0);
    assert_eq!(h.runtime.ran()[before.len()..], will_run_ids(&plan.out));
}

#[test]
fn run_id_と_remove_on_complete_は併用でき_ステートは読むだけ() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime.script("a", &[Exec::Exit(0)]);
    h.runtime.script("b", &[Exec::Exit(1)]);
    assert_ne!(h.run(&["pipeline", "p", "--run-id", "r1"]).code, 0);
    let path = h.state_path_for("r1");
    let bytes = std::fs::read(&path).expect("run ID つきのステート");
    let writes = h.runtime.write_calls.get();

    let out = h.run(&[
        "pipeline",
        "p",
        "--dry-run",
        "--run-id",
        "r1",
        "--remove-on-complete",
    ]);
    assert_eq!(out.code, 0, "{}", out.err);
    assert!(
        out.out
            .starts_with("plan (--dry-run): pipeline p (run ID: r1)\nresume: b (previous stop)\n"),
        "{}",
        out.out
    );
    assert_eq!(std::fs::read(&path).expect("残っている"), bytes);
    assert_eq!(h.runtime.write_calls.get(), writes, "書かない");
    // run ID なしのステートは、読まない(ステートなしの「最初から」)。
    let out = h.run(&["pipeline", "p", "--dry-run", "--run-id", ""]);
    assert!(out.out.contains("resume: from the start\n"), "{}", out.out);
}
