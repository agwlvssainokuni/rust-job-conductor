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

//! Windows 向けデモ(`demo/windows`)のテスト(FR1.3、FR2.1、FR2.2、FR6.1)。
//! 定義とスクリプトを一時ディレクトリへ写し、本物の `conductor` で場面 1〜14 の終了コードと結果を確かめる。
#![cfg(windows)]

mod bin_support;

use bin_support::*;
use std::path::{Path, PathBuf};
use std::process::Output;

fn demo_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../demo/windows")
}

/// デモ一式(定義とスクリプト)を写した一時ディレクトリ。
struct Demo {
    dir: tempfile::TempDir,
}

impl Demo {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("一時ディレクトリ");
        let src = demo_dir();
        std::fs::copy(
            src.join("conductor.toml"),
            dir.path().join("conductor.toml"),
        )
        .expect("定義を写せる");
        std::fs::copy(src.join("retry.toml"), dir.path().join("retry.toml"))
            .expect("リトライの定義を写せる");
        for name in ["env.toml", "demo.env"] {
            std::fs::copy(src.join(name), dir.path().join(name))
                .expect("環境変数の場面の定義を写せる");
        }
        std::fs::create_dir(dir.path().join("scripts")).expect("scripts を作れる");
        for name in [
            "extract",
            "transform",
            "load",
            "notify",
            "flaky",
            "show-env",
        ] {
            let file = format!("{name}.cmd");
            std::fs::copy(
                src.join("scripts").join(&file),
                dir.path().join("scripts").join(&file),
            )
            .expect("スクリプトを写せる");
        }
        Self { dir }
    }

    /// `conductor <args>` を、デモの一時ディレクトリで実行する(待ち時間 0、`envs` を追加)。
    fn run(&self, args: &[&str], envs: &[(&str, &str)]) -> Output {
        let mut command = conductor(args);
        command
            .current_dir(self.dir.path())
            .env("DEMO_SLEEP", "0")
            .env_remove("DEMO_EXTRACT_EXIT")
            .env_remove("DEMO_TRANSFORM_EXIT")
            .env_remove("DEMO_LOAD_EXIT")
            .env_remove("DEMO_NOTIFY_EXIT")
            .env_remove("DEMO_FLAKY_FAILS")
            .env_remove("CONDUCTOR_RUN_ID")
            .env_remove("DEMO_RUN_ID");
        for (key, value) in envs {
            command.env(key, value);
        }
        run_limited(command, || {})
    }
}

#[test]
fn 定義は_4_つのジョブと_extract_の警告の遷移を持つ() {
    let demo = Demo::new();
    let output = demo.run(&["convert", "conductor.toml", "--to", "yaml"], &[]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    let yaml = text(&output.stdout);
    for id in ["extract", "transform", "load", "notify"] {
        assert!(yaml.contains(&format!("id: {id}")), "{id}\n{yaml}");
    }
    assert!(yaml.contains("next: notify"), "{yaml}");
    assert!(yaml.contains("warning_exit_codes"), "{yaml}");
}

#[test]
fn 場面_1_全部が成功すると_0() {
    let demo = Demo::new();
    let output = demo.run(&["pipeline", "-v", "nightly"], &[]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    let out = text(&output.stdout);
    for id in ["extract", "transform", "load", "notify"] {
        assert!(out.contains(&format!("[{id}] 実行中")), "{id}\n{out}");
    }
}

#[test]
fn 場面_2_警告で_notify_へ進み_0() {
    let demo = Demo::new();
    let output = demo.run(
        &["pipeline", "-v", "nightly"],
        &[("DEMO_EXTRACT_EXIT", "1")],
    );
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    let out = text(&output.stdout);
    assert!(out.contains("[notify] 実行中"), "{out}");
    assert!(!out.contains("[transform] 実行中"), "{out}");
    assert!(!out.contains("[load] 実行中"), "{out}");
}

#[test]
fn 場面_3_と_4_load_の失敗は_2_で_再実行すると_load_から再開する() {
    let demo = Demo::new();
    let failed = demo.run(&["pipeline", "-v", "nightly"], &[("DEMO_LOAD_EXIT", "2")]);
    assert_eq!(failed.status.code(), Some(2), "{}", text(&failed.stderr));

    let resumed = demo.run(&["pipeline", "-v", "nightly"], &[]);
    assert_eq!(resumed.status.code(), Some(0), "{}", text(&resumed.stderr));
    let out = text(&resumed.stdout);
    assert!(!out.contains("[extract] 実行中"), "{out}");
    assert!(!out.contains("[transform] 実行中"), "{out}");
    assert!(out.contains("[load] 実行中"), "{out}");
    assert!(out.contains("[notify] 実行中"), "{out}");

    // 完了してもステートは残り、watch は完了を示す
    assert_eq!(state_files(&demo), ["conductor.toml.state"]);
    let watch = demo.run(&["watch"], &[]);
    assert_eq!(watch.status.code(), Some(0), "{}", text(&watch.stderr));
    assert!(
        text(&watch.stdout).trim_end().ends_with("completed"),
        "{}",
        text(&watch.stdout)
    );

    // --remove-on-complete を付けると、完了でステートファイルを消す
    let removed = demo.run(&["pipeline", "-v", "--remove-on-complete", "nightly"], &[]);
    assert_eq!(removed.status.code(), Some(0), "{}", text(&removed.stderr));
    assert!(state_files(&demo).is_empty());
}

#[test]
fn 場面_5_と_6_run_reset_convert_は_0() {
    let demo = Demo::new();
    for args in [
        &["run", "-v", "extract"][..],
        &["reset"][..],
        &["convert", "conductor.toml", "--to", "yaml"][..],
    ] {
        let output = demo.run(args, &[]);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{args:?}\n{}",
            text(&output.stderr)
        );
    }
}

/// 一時ディレクトリにあるステートファイルの名前(並べ替え済み)。
fn state_files(demo: &Demo) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(demo.dir.path())
        .expect("読める")
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("conductor.toml") && n.ends_with(".state"))
        .collect();
    names.sort();
    names
}

/// 標準出力と標準エラーを、つなげた文字列にする。
fn all_text(output: &Output) -> String {
    format!("{}{}", text(&output.stdout), text(&output.stderr))
}

const STOP_AT_LOAD: &[(&str, &str)] = &[("DEMO_LOAD_EXIT", "2")];

#[test]
fn 場面_7_validate_は正しい定義で_0_誤りの定義で_64_ジョブもステートもない() {
    let demo = Demo::new();
    let ok = demo.run(&["validate", "-v"], &[]);
    assert_eq!(ok.status.code(), Some(0), "{}", text(&ok.stderr));
    assert!(
        text(&ok.stderr).contains("definition is valid (4 jobs)"),
        "{}",
        text(&ok.stderr)
    );

    // command の中の引用符(')が閉じていない定義(TOML の文字列としては正しい)
    std::fs::write(
        demo.dir.path().join("bad.toml"),
        "format_version = 1\n\n[pipeline]\nname = \"bad\"\n\n[[job]]\nid = \"extract\"\ncommand = \"sh -c 'echo hi\"\n",
    )
    .expect("書ける");
    let bad = demo.run(&["validate", "--config", "bad.toml"], &[]);
    assert_eq!(bad.status.code(), Some(64), "{}", text(&bad.stderr));
    assert!(
        text(&bad.stderr).contains("unclosed quote"),
        "{}",
        text(&bad.stderr)
    );
    assert!(!all_text(&bad).contains("実行中"), "{}", all_text(&bad));
    assert!(state_files(&demo).is_empty());
}

#[test]
fn 場面_8_run_id_ごとにステートが分かれる() {
    let demo = Demo::new();
    let stopped = demo.run(
        &["pipeline", "-v", "--run-id", "build-1", "nightly"],
        STOP_AT_LOAD,
    );
    assert_eq!(stopped.status.code(), Some(2), "{}", text(&stopped.stderr));
    assert_eq!(state_files(&demo), ["conductor.toml.build-1.state"]);

    // 環境変数 DEMO_RUN_ID(定義の run_id_env)で指定した run は、別のステートで、最後まで進む
    let other = demo.run(
        &["pipeline", "-v", "nightly"],
        &[("DEMO_RUN_ID", "build-2")],
    );
    assert_eq!(other.status.code(), Some(0), "{}", text(&other.stderr));
    assert!(text(&other.stdout).contains("[notify] 実行中"));
    // build-2 は完了したが、完了のステートは残る
    assert_eq!(
        state_files(&demo),
        [
            "conductor.toml.build-1.state",
            "conductor.toml.build-2.state"
        ]
    );
}

#[test]
fn 場面_8_watch_は_run_id_を示し_同じ_run_id_で_load_から再開する() {
    let demo = Demo::new();
    let stopped = demo.run(
        &["pipeline", "-v", "--run-id", "build-1", "nightly"],
        STOP_AT_LOAD,
    );
    assert_eq!(stopped.status.code(), Some(2), "{}", text(&stopped.stderr));

    let watch = demo.run(&["watch", "--run-id", "build-1"], &[]);
    assert_eq!(watch.status.code(), Some(0), "{}", text(&watch.stderr));
    assert!(
        text(&watch.stdout).contains("Run ID: build-1"),
        "{}",
        text(&watch.stdout)
    );

    let resumed = demo.run(&["pipeline", "-v", "--run-id", "build-1", "nightly"], &[]);
    assert_eq!(resumed.status.code(), Some(0), "{}", text(&resumed.stderr));
    let all = all_text(&resumed);
    assert!(all.contains("skipped (already executed)"), "{all}");
    assert!(!all.contains("[extract] 実行中"), "{all}");
    assert!(!all.contains("[transform] 実行中"), "{all}");
    assert!(all.contains("[load] 実行中"), "{all}");
    // 完了してもステートファイルは残り、watch は完了を示す
    assert_eq!(state_files(&demo), ["conductor.toml.build-1.state"]);
    let done = demo.run(&["watch", "--run-id", "build-1"], &[]);
    assert_eq!(done.status.code(), Some(0), "{}", text(&done.stderr));
    assert!(
        text(&done.stdout).trim_end().ends_with("completed"),
        "{}",
        text(&done.stdout)
    );
    // 片づけは reset
    let reset = demo.run(&["reset", "--run-id", "build-1"], &[]);
    assert_eq!(reset.status.code(), Some(0), "{}", text(&reset.stderr));
    assert!(state_files(&demo).is_empty());
}

#[test]
fn 場面_9_run_id_は環境変数に勝ち_空の_run_id_は_run_id_なしに固定する() {
    let demo = Demo::new();
    let env = [("DEMO_RUN_ID", "env-1"), ("DEMO_LOAD_EXIT", "2")];

    let option = demo.run(&["pipeline", "-v", "--run-id", "opt-1", "nightly"], &env);
    assert_eq!(option.status.code(), Some(2), "{}", text(&option.stderr));
    assert_eq!(state_files(&demo), ["conductor.toml.opt-1.state"]);

    let fixed = demo.run(&["pipeline", "-v", "--run-id", "", "nightly"], &env);
    assert_eq!(fixed.status.code(), Some(2), "{}", text(&fixed.stderr));
    assert_eq!(
        state_files(&demo),
        ["conductor.toml.opt-1.state", "conductor.toml.state"]
    );
}

#[test]
fn 場面_9_reset_run_id_は_1_つだけ_reset_all_は残りをすべて消す() {
    let demo = Demo::new();
    for run_id in ["opt-1", "opt-2"] {
        let stopped = demo.run(
            &["pipeline", "-v", "--run-id", run_id, "nightly"],
            STOP_AT_LOAD,
        );
        assert_eq!(stopped.status.code(), Some(2), "{}", text(&stopped.stderr));
    }
    let by_env = demo.run(
        &["pipeline", "-v", "nightly"],
        &[("DEMO_RUN_ID", "env-1"), ("DEMO_LOAD_EXIT", "2")],
    );
    assert_eq!(by_env.status.code(), Some(2), "{}", text(&by_env.stderr));
    assert_eq!(
        state_files(&demo),
        [
            "conductor.toml.env-1.state",
            "conductor.toml.opt-1.state",
            "conductor.toml.opt-2.state"
        ]
    );

    let one = demo.run(&["reset", "--run-id", "opt-1"], &[]);
    assert_eq!(one.status.code(), Some(0), "{}", text(&one.stderr));
    assert_eq!(
        state_files(&demo),
        ["conductor.toml.env-1.state", "conductor.toml.opt-2.state"]
    );

    let all = demo.run(&["reset", "--all"], &[]);
    assert_eq!(all.status.code(), Some(0), "{}", text(&all.stderr));
    assert!(state_files(&demo).is_empty());

    let watch = demo.run(&["watch"], &[]);
    assert_eq!(watch.status.code(), Some(0), "{}", text(&watch.stderr));
    assert!(
        text(&watch.stdout).contains("no state"),
        "{}",
        text(&watch.stdout)
    );
}

#[test]
fn 場面_10_リトライで_2_回失敗してから成功する() {
    let demo = Demo::new();
    let output = demo.run(
        &["pipeline", "-v", "--config", "retry.toml", "retry-demo"],
        &[],
    );
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    let all = all_text(&output);
    // 知らせは、ロケールによらず、先頭が同じ(`conductor: retry: `)。2 回出る。
    assert_eq!(all.matches("conductor: retry: ").count(), 2, "{all}");
    assert!(all.contains("[flaky] 3 回目の実行"), "{all}");
    assert!(all.contains("job=flaky result=success"), "{all}");
}

#[test]
fn 場面_11_環境変数が子に渡り_優先順位は_job_env_が最も強い() {
    let demo = Demo::new();
    let output = demo.run(&["pipeline", "-v", "--config", "env.toml", "env-demo"], &[]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    let all = all_text(&output);
    assert!(
        all.contains("[show-env] DEMO_GREETING=hello from [pipeline.env]"),
        "{all}"
    );
    assert!(
        all.contains("[show-env] DEMO_FROM_FILE=from demo.env"),
        "{all}"
    );
    assert!(all.contains("[show-env] DEMO_QUOTED=two words"), "{all}");
    assert!(all.contains("[show-env] DEMO_STAGE=job"), "{all}");
    assert!(
        all.contains("[show-env] CONDUCTOR_PIPELINE=env-demo"),
        "{all}"
    );
    assert!(
        all.contains("[show-env] CONDUCTOR_JOB_ID=show-env"),
        "{all}"
    );
    assert!(all.contains("[show-env] CONDUCTOR_RUN_ID=(なし)"), "{all}");
    assert!(all.contains("[show-env] CONDUCTOR_ATTEMPT=1"), "{all}");
}

#[test]
fn 場面_11_env_が読めないと_64_で_validate_は重複キーを警告して_0() {
    let demo = Demo::new();
    std::fs::write(
        demo.dir.path().join("bad-env.toml"),
        "format_version = 1\n\n[pipeline]\nname = \"bad-env\"\nenv_file = \"missing.env\"\n\n[[job]]\nid = \"x\"\ncommand = 'scripts\\show-env.cmd'\n",
    )
    .expect("書ける");
    let bad = demo.run(&["pipeline", "--config", "bad-env.toml", "bad-env"], &[]);
    assert_eq!(bad.status.code(), Some(64), "{}", text(&bad.stderr));
    assert!(!text(&bad.stdout).contains("[show-env]"));

    std::fs::write(demo.dir.path().join("dup.env"), "A=1\nA=2\n").expect("書ける");
    std::fs::write(
        demo.dir.path().join("dup-env.toml"),
        "format_version = 1\n\n[pipeline]\nname = \"dup-env\"\nenv_file = \"dup.env\"\n\n[[job]]\nid = \"x\"\ncommand = 'scripts\\show-env.cmd'\n",
    )
    .expect("書ける");
    let dup = demo.run(&["validate", "--config", "dup-env.toml"], &[]);
    assert_eq!(dup.status.code(), Some(0), "{}", text(&dup.stderr));
    assert!(
        text(&dup.stderr).contains("duplicate key `A`"),
        "{}",
        text(&dup.stderr)
    );
}

#[test]
fn 場面_12_graph_は遷移を図にし_state_で状態と再開位置を重ねる() {
    let demo = Demo::new();
    let plain = demo.run(&["graph"], &[]);
    assert_eq!(plain.status.code(), Some(0), "{}", text(&plain.stderr));
    assert_eq!(
        text(&plain.stdout).replace("\r\n", "\n"),
        "*   extract\n|\\\n* | transform\n* | load\n|/\n* notify\no @end\n"
    );
    let verbose = demo.run(&["graph", "-v"], &[]);
    let out = text(&verbose.stdout);
    assert!(out.contains("success -> transform"), "{out}");
    let mermaid = demo.run(&["graph", "--format", "mermaid"], &[]);
    assert!(text(&mermaid.stdout).starts_with("flowchart TD"));
    // load で止めて、状態を重ねる。
    let failed = demo.run(&["pipeline", "nightly"], &[("DEMO_LOAD_EXIT", "2")]);
    assert_eq!(failed.status.code(), Some(2), "{}", text(&failed.stderr));
    let state = demo.run(&["graph", "--state"], &[]);
    let out = text(&state.stdout);
    assert!(out.contains("[OK] extract"), "{out}");
    assert!(out.contains("[NG] load  <- resume here"), "{out}");
    assert_eq!(demo.run(&["reset"], &[]).status.code(), Some(0));
}

#[test]
fn 場面_13_dry_run_は計画を出し_何も実行せず_ステートも作らない() {
    let demo = Demo::new();
    let plan = demo.run(&["pipeline", "nightly", "--dry-run"], &[]);
    assert_eq!(plan.status.code(), Some(0), "{}", text(&plan.stderr));
    let out = text(&plan.stdout).replace("\r\n", "\n");
    assert!(
        out.starts_with(
            "plan (--dry-run): pipeline nightly\nresume: from the start\n1. [run] extract\n"
        ),
        "{out}"
    );
    // extract の警告(終了コード 1)で、transform と load が飛ばされる。
    let warned = demo.run(
        &["pipeline", "nightly", "--dry-run", "--assume", "extract=1"],
        &[],
    );
    let out = text(&warned.stdout).replace("\r\n", "\n");
    assert!(out.contains("assume: extract=1\n"), "{out}");
    assert!(
        out.contains(
            "2. [skip (transition)] transform\n3. [skip (transition)] load\n4. [run] notify\n"
        ),
        "{out}"
    );
    // load の失敗(終了コード 2)の仮定で、そこで停止する。
    let stopped = demo.run(
        &["pipeline", "nightly", "--dry-run", "--assume", "load=2"],
        &[],
    );
    let out = text(&stopped.stdout).replace("\r\n", "\n");
    assert!(
        out.contains("result (simulated): stopped (error: load)\n"),
        "{out}"
    );
    // run の次の動き。
    let single = demo.run(&["run", "extract", "--dry-run"], &[]);
    let out = text(&single.stdout).replace("\r\n", "\n");
    assert!(out.contains("next step: next job transform\n"), "{out}");
    assert!(state_files(&demo).is_empty(), "ステートは作られない");
}

#[test]
fn 場面_14_init_はひな形を作り_2_回目は_69_で_force_で上書きできる() {
    let demo = Demo::new();
    let created = demo.run(&["init", "--config", "new.toml", "starter"], &[]);
    assert_eq!(created.status.code(), Some(0), "{}", text(&created.stderr));
    assert!(text(&created.stderr).contains("created new.toml"));
    let checked = demo.run(&["validate", "--config", "new.toml", "starter"], &[]);
    assert_eq!(checked.status.code(), Some(0), "{}", text(&checked.stderr));
    // 2 回目は、何も変えず、69 で終わる。
    let before = std::fs::read(demo.dir.path().join("new.toml")).expect("できている");
    let again = demo.run(&["init", "--config", "new.toml", "starter"], &[]);
    assert_eq!(again.status.code(), Some(69));
    assert!(text(&again.stderr).contains("already exists (use --force to overwrite)"));
    assert_eq!(
        std::fs::read(demo.dir.path().join("new.toml")).expect("読める"),
        before
    );
    // --force で上書き。`--config -` は、ファイルの内容と同じひな形を出す。
    let forced = demo.run(&["init", "--config", "new.toml", "--force", "starter"], &[]);
    assert_eq!(forced.status.code(), Some(0), "{}", text(&forced.stderr));
    assert!(text(&forced.stderr).contains("replaced new.toml"));
    let shown = demo.run(&["init", "--config", "-", "starter"], &[]);
    assert_eq!(shown.status.code(), Some(0), "{}", text(&shown.stderr));
    assert_eq!(
        shown.stdout,
        std::fs::read(demo.dir.path().join("new.toml")).expect("読める")
    );
}

#[test]
fn 利用者の環境の_run_id_は_デモの動作に影響しない() {
    // Demo::run は CONDUCTOR_RUN_ID と DEMO_RUN_ID を環境から外す。
    // この関数は、そのことを、親プロセスの環境に依らず確かめる(外した上で、run ID なしのステートになる)。
    let demo = Demo::new();
    let stopped = demo.run(&["pipeline", "-v", "nightly"], STOP_AT_LOAD);
    assert_eq!(stopped.status.code(), Some(2), "{}", text(&stopped.stderr));
    assert_eq!(state_files(&demo), ["conductor.toml.state"]);
}
