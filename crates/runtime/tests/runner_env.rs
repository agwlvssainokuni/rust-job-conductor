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

//! `run_job` の環境変数の引数(U1。BR4.3、BR4.4、FR1.3)。補助の実行ファイルの `env` モードで、
//! 子のプロセスの環境を確かめる。環境変数は、プロセス全体には設定しない(並列のテストが干渉するため)。

mod common;

use common::*;
use conductor_core::ExecutionResult;
use conductor_runtime::{InterruptSignal, check_command, run_job};

/// 子で `env <names...>` を実行し、出力の行を返す。
fn child_env(env: &[(&str, &str)], names: &[&str]) -> Vec<String> {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let out = dir.path().join("out.txt");
    let command = helper_command(&out, &format!("env {}", names.join(" ")));
    let j = job("j", command);
    let line = check_command(&j.command).expect("分けられる");
    let e = run_job(&line, env, &InterruptSignal::new());
    assert!(matches!(
        e.result,
        ExecutionResult::Exited {
            exit_code: Some(0),
            ..
        }
    ));
    lines(&out)
}

#[test]
fn 足した値が子のプロセスに見える() {
    let got = child_env(
        &[
            ("CONDUCTOR_T_ADDED", "hello world"),
            ("CONDUCTOR_T_EMPTY", ""),
        ],
        &["CONDUCTOR_T_ADDED", "CONDUCTOR_T_EMPTY"],
    );
    assert_eq!(
        got,
        [
            "env[CONDUCTOR_T_ADDED]=Some(\"hello world\")",
            "env[CONDUCTOR_T_EMPTY]=Some(\"\")"
        ]
    );
}

#[test]
fn 足さなかった引き継いだ環境は残る() {
    // `cargo test` が子に渡す `CARGO_PKG_NAME` などではなく、必ずある `PATH` を使う。
    let inherited = std::env::var("PATH").expect("PATH がある");
    let got = child_env(&[("CONDUCTOR_T_OTHER", "x")], &["PATH"]);
    assert_eq!(got, [format!("env[PATH]=Some({inherited:?})")]);
}

#[test]
fn 足した値は引き継いだ値を上書きする() {
    let got = child_env(&[("PATH", "/overridden")], &["PATH"]);
    assert_eq!(got, ["env[PATH]=Some(\"/overridden\")"]);
}

#[test]
fn 同じ名前が並びに_2_回あれば後ろが勝つ() {
    let got = child_env(
        &[
            ("CONDUCTOR_T_TWICE", "first"),
            ("CONDUCTOR_T_TWICE", "second"),
        ],
        &["CONDUCTOR_T_TWICE"],
    );
    assert_eq!(got, ["env[CONDUCTOR_T_TWICE]=Some(\"second\")"]);
}

#[test]
fn 空の環境では従来どおりで足した名前は見えない() {
    let got = child_env(&[], &["CONDUCTOR_T_NOT_SET"]);
    assert_eq!(got, ["env[CONDUCTOR_T_NOT_SET]=None"]);
}
