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

//! run ID の、実際のバイナリのテスト(FR3.3、FR3.4、FR5)。

mod bin_support;

use std::process::Output;

use bin_support::*;
use conductor_core::JobStatus;

/// a は成功、b は終了コード `b_exit`。
fn two_jobs(b_exit: i32) -> String {
    definition(
        "p",
        &[
            job_block("a", &helper_command("exit 0"), ""),
            job_block("b", &helper_command(&format!("exit {b_exit}")), ""),
        ],
    )
}

fn run_with_env(project: &Project, args: &[&str], env: &[(&str, &str)]) -> Output {
    let mut command = conductor(args);
    command.arg("--config").arg(project.def_path());
    for (k, v) in env {
        command.env(k, v);
    }
    run_limited(command, || {})
}

fn names(project: &Project) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(project.dir.path())
        .expect("読める")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

fn id(v: &str) -> conductor_core::RunId {
    conductor_core::RunId::parse(v).expect("有効な run ID")
}

#[test]
fn 並行する_2_つの_run_id_は別のステートで_互いの再開位置を変えず_同じ_run_id_で再開する() {
    let project = Project::new(&two_jobs(7));
    // 2 つの run ID の pipeline を、同時に動かす。
    let results: Vec<Output> = std::thread::scope(|s| {
        let handles: Vec<_> = ["r1", "r2"]
            .into_iter()
            .map(|r| s.spawn(|| project.run(&["pipeline", "p", "--run-id", r])))
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for output in &results {
        assert_eq!(output.status.code(), Some(7), "{}", text(&output.stderr));
        assert!(output.stdout.is_empty());
    }
    assert_eq!(
        names(&project),
        [
            "conductor.toml",
            "conductor.toml.r1.state",
            "conductor.toml.r2.state"
        ]
    );
    let def = project.def_path();
    for r in ["r1", "r2"] {
        let s = conductor_runtime::read_state(&def, Some(&id(r)))
            .expect("読める")
            .expect("ステートがある");
        assert_eq!(s.run_id.as_deref(), Some(r));
        assert_eq!(s.resume_job_id.as_deref(), Some("b"));
        assert_eq!(s.job_states[0].status, JobStatus::Success);
    }
    let r2_before =
        std::fs::read(project.dir.path().join("conductor.toml.r2.state")).expect("読める");

    // 定義を直して(b が成功する)、r1 だけを、同じ run ID で呼び直す: b から再開して、完了する。
    std::fs::write(project.def_path(), two_jobs(0)).expect("書ける");
    let output = project.run(&["pipeline", "p", "--run-id", "r1"]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    assert_eq!(
        names(&project),
        ["conductor.toml", "conductor.toml.r2.state"],
        "r1 だけ完了して消える"
    );
    let r2_after =
        std::fs::read(project.dir.path().join("conductor.toml.r2.state")).expect("読める");
    assert_eq!(r2_before, r2_after, "r2 の再開位置は変わらない");
}

#[test]
fn 環境変数と空のオプションと定義の_run_id_env_はバイナリでも効く() {
    let project = Project::new(&two_jobs(7));
    // 環境変数
    let output = run_with_env(
        &project,
        &["pipeline", "p"],
        &[("CONDUCTOR_RUN_ID", "fromenv")],
    );
    assert_eq!(output.status.code(), Some(7));
    assert!(
        project
            .dir
            .path()
            .join("conductor.toml.fromenv.state")
            .exists()
    );
    // 空のオプションは、環境変数があっても、run ID なしに固定する
    let output = run_with_env(
        &project,
        &["pipeline", "p", "--run-id", ""],
        &[("CONDUCTOR_RUN_ID", "fromenv")],
    );
    assert_eq!(output.status.code(), Some(7));
    assert!(project.state_path().exists());
    // オプションは環境変数に優先する
    run_with_env(
        &project,
        &["pipeline", "p", "--run-id", "fromopt"],
        &[("CONDUCTOR_RUN_ID", "fromenv")],
    );
    assert!(
        project
            .dir
            .path()
            .join("conductor.toml.fromopt.state")
            .exists()
    );

    // 定義の run_id_env の名前(run も watch も reset も)
    let named = definition("p", &[job_block("a", &helper_command("exit 7"), "")]).replacen(
        "name = \"p\"\n",
        "name = \"p\"\nrun_id_env = \"MY_RUN\"\n",
        1,
    );
    let project = Project::new(&named);
    let output = run_with_env(
        &project,
        &["run", "a"],
        &[("MY_RUN", "m1"), ("CONDUCTOR_RUN_ID", "ignored")],
    );
    assert_eq!(output.status.code(), Some(7));
    assert!(project.dir.path().join("conductor.toml.m1.state").exists());
    let output = run_with_env(&project, &["watch"], &[("MY_RUN", "m1")]);
    assert_eq!(output.status.code(), Some(0));
    assert!(
        text(&output.stdout).contains("Run ID: m1\n"),
        "{}",
        text(&output.stdout)
    );
    let output = run_with_env(&project, &["reset"], &[("MY_RUN", "m1")]);
    assert_eq!(output.status.code(), Some(0));
    assert!(!project.dir.path().join("conductor.toml.m1.state").exists());
}

#[test]
fn 無効な値は_68_で_validate_は_run_id_を受け付けず_run_id_env_の誤りは_64() {
    let project = Project::new(&two_jobs(0));
    for bad in ["a/b", "..", &"x".repeat(65)] {
        let output = project.run(&["pipeline", "p", "--run-id", bad]);
        assert_eq!(output.status.code(), Some(68), "{bad}");
        assert!(!text(&output.stderr).contains(bad));
        let output = run_with_env(&project, &["pipeline", "p"], &[("CONDUCTOR_RUN_ID", bad)]);
        assert_eq!(output.status.code(), Some(68), "{bad}");
        assert!(!text(&output.stderr).contains(bad));
    }
    assert_eq!(names(&project), ["conductor.toml"], "ステートは作られない");
    let output = project.run(&["validate", "--run-id", "x"]);
    assert_eq!(output.status.code(), Some(68));
    let bad_env = two_jobs(0).replacen("name = \"p\"\n", "name = \"p\"\nrun_id_env = \"1A\"\n", 1);
    let bad = Project::new(&bad_env);
    assert_eq!(bad.run(&["validate"]).status.code(), Some(64));
    assert_eq!(bad.run(&["run", "a"]).status.code(), Some(64));
}

#[test]
fn reset_all_は定義のステートだけを消し_別の定義と壊れたものと一時ファイルを残す() {
    let project = Project::new(&two_jobs(7));
    let dir = project.dir.path();
    // 本物のステート: run ID なしと、2 つの run ID
    assert_eq!(project.run(&["pipeline", "p"]).status.code(), Some(7));
    for r in ["a", "b"] {
        assert_eq!(
            project.run(&["pipeline", "p", "--run-id", r]).status.code(),
            Some(7)
        );
    }
    // 別の定義 conductor.toml.a(run ID なし)のステート。名前は、run ID `a` のものと同じ。
    let other = dir.join("conductor.toml.a.state");
    let other_text = "format_version = 1\ndefinition_key = \"x\"\ndefinition_path = \"x\"\npipeline_name = \"p\"\njob_order = [\"a\"]\nend_reached = false\ncompleted = false\n\n[[job]]\nid = \"a\"\nstatus = \"pending\"\n";
    std::fs::write(&other, other_text).unwrap();
    // 壊れたもの、一時ファイル、無関係のファイル
    std::fs::write(dir.join("conductor.toml.c.state"), "garbage = = =").unwrap();
    std::fs::write(dir.join(".conductor.toml.state.tmp.1.2.3"), "tmp").unwrap();
    std::fs::write(dir.join("conductor.toml.bak"), "bak").unwrap();
    std::fs::write(dir.join("other.toml.state"), "other").unwrap();

    // `a` の本物のステートは、`conductor.toml.a.state` を上書きしたので、別の定義のものに置き換わった。
    // 本物の `a` を作り直す。
    std::fs::remove_file(&other).unwrap();
    assert_eq!(
        project
            .run(&["pipeline", "p", "--run-id", "a"])
            .status
            .code(),
        Some(7)
    );
    // 別の定義のステートは、別の名前で置く(名前が重なる場合の判定は、runtime のテストで確かめる)。
    std::fs::write(dir.join("conductor.toml.a.b.state"), other_text).unwrap();

    let output = project.run(&["reset", "--all"]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    assert!(output.stdout.is_empty());
    let err = text(&output.stderr);
    assert!(err.starts_with("conductor: warning: "), "{err}");
    assert!(err.contains("conductor.toml.c.state"), "{err}");
    assert!(
        !err.contains("conductor.toml.a.b.state"),
        "壊れていないものは警告しない: {err}"
    );
    assert_eq!(
        names(&project),
        [
            ".conductor.toml.state.tmp.1.2.3",
            "conductor.toml",
            "conductor.toml.a.b.state",
            "conductor.toml.bak",
            "conductor.toml.c.state",
            "other.toml.state",
        ]
    );
    // 壊れたものは、run ID を指定して、個別に消せる。
    assert_eq!(
        project.run(&["reset", "--run-id", "c"]).status.code(),
        Some(0)
    );
    assert!(!dir.join("conductor.toml.c.state").exists());
}

#[test]
fn reset_all_は何もなくても_0_で何も出力しない() {
    let project = Project::new(&two_jobs(0));
    let output = project.run(&["reset", "--all"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
    assert_eq!(
        project
            .run(&["reset", "--all", "--run-id", "x"])
            .status
            .code(),
        Some(68)
    );
}
