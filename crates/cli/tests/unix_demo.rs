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

//! UNIX 向けデモ(`demo/run-demo.sh`)のテスト(FR6.2)。
//! スクリプトを `sh` で実行し、場面 0〜9 の見出しと要点の行が出て、0 で終わることを確かめる。
#![cfg(unix)]

mod bin_support;

use bin_support::*;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("リポジトリの根")
        .to_path_buf()
}

/// `sh demo/run-demo.sh` を、リポジトリの根から実行する。
/// 利用者の環境に左右されないよう、関係する環境変数は、すべて明示する。
fn run_demo(tmp: &Path, user_env: &[(&str, &str)]) -> Output {
    let mut command = Command::new("sh");
    command
        .arg("demo/run-demo.sh")
        .current_dir(repo_root())
        .env("CONDUCTOR", env!("CARGO_BIN_EXE_conductor"))
        .env("DEMO_LANG", "en_US.UTF-8")
        .env("DEMO_SLEEP", "0")
        .env("TMPDIR", tmp)
        .env_remove("DEMO_PAUSE")
        .env_remove("CONDUCTOR_RUN_ID")
        .env_remove("DEMO_RUN_ID")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env_remove("NO_COLOR");
    for (key, value) in user_env {
        command.env(key, value);
    }
    run_limited(command, || {})
}

fn assert_demo_output(output: &Output) {
    let err = text(&output.stderr);
    assert_eq!(output.status.code(), Some(0), "{err}");
    // `-v` の行やエラーは標準エラーに出るので、標準出力と合わせて見る。
    let out = format!("{}{err}", text(&output.stdout));
    for n in 0..=9 {
        assert!(out.contains(&format!("=== {n}. ")), "場面 {n}\n{out}");
    }
    // 場面 7: validate
    assert!(out.contains("definition is valid"), "{out}");
    assert!(out.contains("→ 終了コード 64"), "{out}");
    // 場面 8: run ID ごとにステートが分かれる
    assert!(out.contains("Run ID: build-1"), "{out}");
    assert!(out.contains("conductor.toml.build-1.state"), "{out}");
    assert!(out.contains("skipped (already executed)"), "{out}");
    // 場面 9: --run-id が環境変数に勝つ、reset --all のあとは空
    assert!(out.contains("conductor.toml.opt-1.state"), "{out}");
    assert!(out.contains("(なし)"), "{out}");
    assert!(out.contains("no state"), "{out}");
}

#[test]
fn デモは_0_で完走し_場面_0_から_9_の要点の行が出る() {
    let tmp = tempfile::tempdir().expect("一時ディレクトリ");
    let output = run_demo(tmp.path(), &[]);
    assert_demo_output(&output);
}

#[test]
fn 利用者の環境の_run_id_と_ロケールに左右されない() {
    let tmp = tempfile::tempdir().expect("一時ディレクトリ");
    let output = run_demo(
        tmp.path(),
        &[
            ("CONDUCTOR_RUN_ID", "zzz"),
            ("DEMO_RUN_ID", "zzz"),
            ("LC_ALL", "C"),
            ("LC_MESSAGES", "C"),
            ("NO_COLOR", "1"),
        ],
    );
    assert_demo_output(&output);
    let out = text(&output.stdout);
    // 利用者の run ID は使われない
    assert!(!out.contains("zzz"), "{out}");
}

#[test]
fn デモの一時ディレクトリは_終わると残らない() {
    let tmp = tempfile::tempdir().expect("一時ディレクトリ");
    let output = run_demo(tmp.path(), &[]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    let left: Vec<_> = std::fs::read_dir(tmp.path())
        .expect("読める")
        .filter_map(Result::ok)
        .map(|e| e.file_name())
        .collect();
    assert!(left.is_empty(), "残ったもの: {left:?}");
}
