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

//! Windows 向けデモ(`demo/windows`)のテスト(FR1.3、FR2.1、FR2.2)。
//! 定義とスクリプトを一時ディレクトリへ写し、本物の `conductor` で場面 1〜6 の終了コードを確かめる。
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
        std::fs::create_dir(dir.path().join("scripts")).expect("scripts を作れる");
        for name in ["extract", "transform", "load", "notify"] {
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
            .env_remove("DEMO_NOTIFY_EXIT");
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
