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

//! run ID の流れ(FR1〜FR6)。偽の `Runtime`(ジョブの起動だけ偽で、ステートは実際のファイル)で確かめる。

mod common;

use common::*;
use conductor_runtime::ResetAllReport;

/// `[pipeline]` に `run_id_env = "X_ID"` を足した、3 ジョブの定義。
fn with_env_name() -> String {
    THREE_JOBS.replacen("name = \"p\"\n", "name = \"p\"\nrun_id_env = \"X_ID\"\n", 1)
}

// ---- run ・ pipeline: ステートの分離(FR1.2、FR3.1、FR3.3) ----

#[test]
fn run_id_ごとに別のステートのファイルを読み書きする() {
    let mut h = Harness::new(THREE_JOBS);
    // r1 は b で止まる
    h.runtime.script("b", &[Exec::Exit(5)]);
    assert_eq!(h.run(&["pipeline", "p", "--run-id", "r1"]).code, 5);
    // r2 は a から始まる(r1 のステートに影響されない)
    let r2 = h.run(&["run", "a", "--run-id", "r2"]);
    assert_eq!(r2.code, 0, "{}", r2.err);

    let s1 = h.state_of("r1").expect("r1 のステート");
    let s2 = h.state_of("r2").expect("r2 のステート");
    assert_eq!(s1.run_id.as_deref(), Some("r1"));
    assert_eq!(s2.run_id.as_deref(), Some("r2"));
    assert_eq!(s1.resume_job_id.as_deref(), Some("b"));
    assert_eq!(s2.job_states[0].status, conductor_core::JobStatus::Success);
    assert_eq!(s2.job_states[1].status, conductor_core::JobStatus::Pending);
    assert_eq!(
        s1.job_states[1].status,
        conductor_core::JobStatus::Error,
        "r2 の実行は、r1 の記録を変えない"
    );
    assert!(h.state_path_for("r1").exists() && h.state_path_for("r2").exists());
    assert!(
        !h.state_path().exists(),
        "run ID なしのファイルは作られない"
    );
    assert!(h.state().is_none());
}

#[test]
fn 同じ_run_id_で呼び直すと止まったジョブから再開する() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime.script("b", &[Exec::Exit(5), Exec::Exit(0)]);
    assert_eq!(h.run(&["pipeline", "p", "--run-id", "r1"]).code, 5);
    h.runtime.log.borrow_mut().clear();
    assert_eq!(h.run(&["pipeline", "p", "--run-id", "r1"]).code, 0);
    assert_eq!(
        h.runtime.ran(),
        ["b", "c"],
        "a は実行済みなので、b から再開する"
    );
    assert!(
        h.state_raw_of("r1").expect("残る").completed,
        "完了のステートが残る"
    );
    assert!(
        h.state_of("r1").is_none(),
        "read_state は完了を空として返す"
    );
}

#[test]
fn fr2_1_run_id_ありでも_remove_on_complete_なら完了で消える() {
    let mut h = Harness::new(THREE_JOBS);
    assert_eq!(
        h.run(&["pipeline", "p", "--run-id", "r1", "--remove-on-complete"])
            .code,
        0
    );
    assert!(!h.state_path_for("r1").exists(), "完了で消える");
}

#[test]
fn run_id_なしは従来のファイル名と形のまま() {
    let mut h = Harness::new(THREE_JOBS);
    assert_eq!(h.run(&["run", "a"]).code, 0);
    assert!(h.state_path().exists());
    let body = std::fs::read_to_string(h.state_path()).unwrap();
    assert!(!body.contains("run_id"), "{body}");
}

// ---- run ID の決め方(FR1.2〜FR1.4) ----

#[test]
fn オプションは環境変数に優先し_環境変数だけでも使われる() {
    let mut h = Harness::new(THREE_JOBS);
    h.env
        .vars
        .insert("CONDUCTOR_RUN_ID".into(), "fromenv".into());
    assert_eq!(h.run(&["run", "a", "--run-id", "fromopt"]).code, 0);
    assert!(h.state_of("fromopt").is_some());
    assert!(h.state_of("fromenv").is_none());
    assert_eq!(h.run(&["run", "a"]).code, 0);
    assert!(h.state_of("fromenv").is_some(), "環境変数だけでも使う");
    assert!(h.state().is_none());
}

#[test]
fn 空の_run_id_のオプションは環境変数があっても_run_id_なしに固定する() {
    let mut h = Harness::new(THREE_JOBS);
    h.env
        .vars
        .insert("CONDUCTOR_RUN_ID".into(), "fromenv".into());
    assert_eq!(h.run(&["run", "a", "--run-id", ""]).code, 0);
    assert!(h.state().is_some(), "従来のステート");
    assert!(h.state_of("fromenv").is_none());
}

#[test]
fn 環境変数が空なら_run_id_なし() {
    let mut h = Harness::new(THREE_JOBS);
    h.env.vars.insert("CONDUCTOR_RUN_ID".into(), String::new());
    assert_eq!(h.run(&["run", "a"]).code, 0);
    assert!(h.state().is_some());
}

#[test]
fn 定義の_run_id_env_の名前の環境変数を読む() {
    let mut h = Harness::new(&with_env_name());
    h.env.vars.insert("X_ID".into(), "named".into());
    h.env
        .vars
        .insert("CONDUCTOR_RUN_ID".into(), "ignored".into());
    assert_eq!(h.run(&["run", "a"]).code, 0);
    assert!(h.state_of("named").is_some());
    assert!(h.state_of("ignored").is_none());
}

#[test]
fn 環境変数の値の違反は_68_で誤りの文に値を含めない() {
    for bad in ["a/b", "..", ".a", &"x".repeat(65), "a b", "é"] {
        let mut h = Harness::new(THREE_JOBS);
        h.env
            .vars
            .insert("CONDUCTOR_RUN_ID".into(), bad.to_string());
        let out = h.run(&["run", "a"]);
        assert_eq!(out.code, 68, "{bad}");
        assert!(out.err.contains("invalid value"), "{}", out.err);
        assert!(out.err.contains("environment variable"), "{}", out.err);
        assert!(!out.err.contains(bad), "値を含めない: {}", out.err);
        assert_eq!(h.runtime.ran(), Vec::<String>::new());
        assert!(!h.state_path().exists());
    }
}

#[test]
fn オプションの値の違反は_68_で誤りの文に値を含めない() {
    for bad in ["a/b", "..", ".a", &"y".repeat(65), "a b", "é"] {
        let mut h = Harness::new(THREE_JOBS);
        let out = h.run(&["run", "a", "--run-id", bad]);
        assert_eq!(out.code, 68, "{bad}");
        assert!(out.err.contains("--run-id"), "{}", out.err);
        assert!(!out.err.contains(bad), "値を含めない: {}", out.err);
        assert!(h.runtime.log().is_empty(), "何も実行しない");
    }
}

#[test]
fn run_id_の決定は定義の検査のあと_ステートの読み込みの前() {
    // 定義の誤り(64)が、環境変数の値の誤り(68)より先に報告される。
    let mut h = Harness::new("not a definition = =");
    h.env.vars.insert("CONDUCTOR_RUN_ID".into(), "a/b".into());
    assert_eq!(h.run(&["run", "a"]).code, 64);
    // 環境変数の値の誤りのとき、ステートは読まれない。
    let mut h = Harness::new(THREE_JOBS);
    h.env.vars.insert("CONDUCTOR_RUN_ID".into(), "a/b".into());
    assert_eq!(h.run(&["pipeline", "p"]).code, 68);
    assert_eq!(h.runtime.log(), ["load"]);
}

#[test]
fn ファイル名と中身の_run_id_の食い違いは_65_で_reset_の案内を出す() {
    let mut h = Harness::new(THREE_JOBS);
    assert_eq!(h.run(&["run", "a"]).code, 0);
    std::fs::copy(h.state_path(), h.state_path_for("a")).expect("コピーできる");
    h.runtime.log.borrow_mut().clear();
    let out = h.run(&["run", "b", "--run-id", "a"]);
    assert_eq!(out.code, 65);
    assert!(out.err.contains("state file is corrupt"), "{}", out.err);
    assert!(out.err.contains("reset --run-id <ID>"), "{}", out.err);
    assert!(h.runtime.ran().is_empty(), "ジョブは起動しない");
}

// ---- watch ・ reset(FR4) ----

#[test]
fn watch_は_run_id_のオプションがあれば定義を読まない() {
    let mut h = Harness::new("not a definition = =");
    let mut ok = Harness::new(THREE_JOBS);
    assert_eq!(ok.run(&["run", "a", "--run-id", "r1"]).code, 0);
    // 同じ場所に、正しい定義があるときのステートを、壊れた定義のディレクトリへ移す
    std::fs::copy(ok.state_path_for("r1"), h.state_path_for("r1")).unwrap();
    let out = h.run(&["watch", "--run-id", "r1"]);
    assert_eq!(out.code, 0, "{}", out.err);
    assert_eq!(h.runtime.run_id_env_calls.get(), 0, "定義を読まない");
    assert!(out.out.contains("Run ID: r1\n"), "{}", out.out);
    assert!(out.out.contains("Pipeline: p\n"), "{}", out.out);
}

#[test]
fn watch_は_run_id_なしなら表示は変わらず_run_id_の行を出さない() {
    let mut h = Harness::new(THREE_JOBS);
    h.run(&["run", "a"]);
    let out = h.run(&["watch"]);
    assert_eq!(out.code, 0);
    assert!(!out.out.contains("Run ID"), "{}", out.out);
    assert!(out.out.starts_with("Pipeline: p\nJOB"), "{}", out.out);
}

#[test]
fn watch_と_reset_は定義の_run_id_env_をゆるく読んで使う() {
    let mut h = Harness::new(&with_env_name());
    h.env.vars.insert("X_ID".into(), "named".into());
    h.env.vars.insert("CONDUCTOR_RUN_ID".into(), "other".into());
    assert_eq!(h.run(&["run", "a"]).code, 0);
    assert!(h.state_of("named").is_some());
    let out = h.run(&["watch"]);
    assert!(out.out.contains("Run ID: named\n"), "{}", out.out);
    assert_eq!(h.run(&["reset"]).code, 0);
    assert!(h.state_of("named").is_none(), "named のステートが消える");
}

#[test]
fn reset_は壊れた定義でも_既定の環境変数の名前で_その_run_のステートだけを消す() {
    let mut h = Harness::new("not a definition = =");
    std::fs::write(h.state_path_for("r1"), "garbage = = =").unwrap();
    std::fs::write(h.state_path_for("r2"), "garbage = = =").unwrap();
    std::fs::write(h.state_path(), "garbage = = =").unwrap();
    h.env.vars.insert("CONDUCTOR_RUN_ID".into(), "r1".into());
    let out = h.run(&["reset"]);
    assert_eq!(out.code, 0, "{}", out.err);
    assert!(!h.state_path_for("r1").exists());
    assert!(h.state_path_for("r2").exists(), "別の run ID は残る");
    assert!(h.state_path().exists(), "run ID なしも残る");
    assert_eq!(h.runtime.log(), ["reset"], "検証も読み込みもしない");
}

#[test]
fn reset_と_watch_の環境変数の値の違反は_68() {
    let mut h = Harness::new(THREE_JOBS);
    h.env.vars.insert("CONDUCTOR_RUN_ID".into(), "../x".into());
    assert_eq!(h.run(&["reset"]).code, 68);
    assert_eq!(h.run(&["watch"]).code, 68);
    assert!(h.runtime.log().is_empty(), "何も消さず、何も読まない");
}

#[test]
fn reset_の_run_id_の指定は別の_run_id_とステートに影響しない() {
    let mut h = Harness::new(THREE_JOBS);
    h.run(&["run", "a", "--run-id", "r1"]);
    h.run(&["run", "a", "--run-id", "r2"]);
    h.run(&["run", "a"]);
    assert_eq!(h.run(&["reset", "--run-id", "r1"]).code, 0);
    assert!(h.state_of("r1").is_none());
    assert!(h.state_of("r2").is_some());
    assert!(h.state().is_some());
    assert_eq!(
        h.run(&["reset", "--run-id", ""]).code,
        0,
        "run ID なしに固定"
    );
    assert!(h.state().is_none());
    assert!(h.state_of("r2").is_some());
}

// ---- reset --all(FR5) ----

#[test]
fn reset_all_は何も出力せず_0_で環境変数の値を見ない() {
    let mut h = Harness::new(THREE_JOBS);
    h.run(&["run", "a", "--run-id", "r1"]);
    h.run(&["run", "a"]);
    h.env
        .vars
        .insert("CONDUCTOR_RUN_ID".into(), "../bad".into());
    h.env.names.borrow_mut().clear();
    let out = h.run(&["reset", "--all"]);
    assert_eq!(out.code, 0, "{}", out.err);
    assert_eq!(out.out, "");
    assert_eq!(out.err, "");
    assert!(h.state_of("r1").is_none());
    assert!(h.state().is_none());
    assert_eq!(h.runtime.run_id_env_calls.get(), 0, "定義を読まない");
    assert!(!h.env.names.borrow().iter().any(|n| n == "CONDUCTOR_RUN_ID"));
}

#[test]
fn reset_all_は残したファイルを警告に出し_終了コードは_0() {
    let mut h = Harness::new(THREE_JOBS);
    *h.runtime.reset_all_override.borrow_mut() = Some(ResetAllReport {
        kept: vec!["conductor.toml.c.state".into(), "x\u{1b}[31m.state".into()],
    });
    let out = h.run(&["reset", "--all"]);
    assert_eq!(out.code, 0);
    assert_eq!(out.out, "");
    assert!(out.err.starts_with("conductor: warning: "), "{}", out.err);
    assert!(
        out.err.contains("conductor reset --run-id <ID>"),
        "{}",
        out.err
    );
    assert!(
        out.err.contains("  conductor.toml.c.state\n"),
        "{}",
        out.err
    );
    assert!(
        !out.err.contains('\u{1b}'),
        "エスケープされる: {:?}",
        out.err
    );
    // 日本語
    h.env.vars.insert("LANG".into(), "ja_JP.UTF-8".into());
    let out = h.run(&["reset", "--all"]);
    assert_eq!(out.code, 0);
    assert!(out.err.contains("残しました"), "{}", out.err);
}

#[test]
fn reset_all_の削除の失敗は_65() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime.fail_reset.set(true);
    let out = h.run(&["reset", "--all"]);
    assert_eq!(out.code, 65);
    assert!(out.err.contains("cannot remove state file"), "{}", out.err);
}

#[test]
fn メモリの_runtime_は_run_id_ごとにステートを分ける() {
    let runtime = MemRuntime::new(THREE_JOBS);
    assert_eq!(run_mem(&runtime, &["run", "a", "--run-id", "m1"]).0, 0);
    assert_eq!(run_mem(&runtime, &["run", "a"]).0, 0);
    assert!(runtime.run_states.borrow().contains_key("m1"));
    assert!(runtime.state.borrow().is_some());
    assert_eq!(run_mem(&runtime, &["reset", "--all"]).0, 0);
    assert!(runtime.run_states.borrow().is_empty());
    assert!(runtime.state.borrow().is_none());
}

// ---- validate(FR6) ----

#[test]
fn validate_は_run_id_env_の型と形の誤りを_64_で拒否し_正しければ通す() {
    for bad in ["5", "\"1A\"", "\"\"", "\"A-B\""] {
        let text = THREE_JOBS.replacen(
            "name = \"p\"\n",
            &format!("name = \"p\"\nrun_id_env = {bad}\n"),
            1,
        );
        let mut h = Harness::new(&text);
        let out = h.run(&["validate"]);
        assert_eq!(out.code, 64, "{bad}: {}", out.err);
        // run・pipeline も、同じ検査で拒否する
        assert_eq!(h.run(&["run", "a"]).code, 64, "{bad}");
        assert!(!h.state_path().exists());
    }
    let mut h = Harness::new(&with_env_name());
    assert_eq!(h.run(&["validate"]).code, 0);
}

#[test]
fn validate_は_run_id_のオプションと_all_を受け付けず_環境変数を見ない() {
    let mut h = Harness::new(THREE_JOBS);
    h.env
        .vars
        .insert("CONDUCTOR_RUN_ID".into(), "../bad".into());
    let out = h.run(&["validate", "--run-id", "x"]);
    assert_eq!(out.code, 68);
    assert!(out.err.contains("option not allowed"), "{}", out.err);
    assert_eq!(h.run(&["validate", "--all"]).code, 68);
    assert_eq!(h.run(&["validate"]).code, 0, "環境変数の値は検査しない");
    assert!(!h.env.names.borrow().iter().any(|n| n == "CONDUCTOR_RUN_ID"));
    assert!(
        !h.runtime.log().iter().any(|l| l == "read"),
        "ステートに触れない"
    );
}
