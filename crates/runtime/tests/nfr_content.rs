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

//! エラーの表示と、ステートファイルの内容に、入力の値が現れないこと(NFR4.5)。

mod common;

use std::path::Path;

use conductor_core::{
    Classification, JobExecution, JobOutcome, NextStep, PipelineDefinition, PipelineState,
};
use conductor_runtime::{
    CommandErrorKind, Durability, StateError, StateErrorKind, apply_outcome, check_command,
    initial_state as initial_state_with, read_state as read_state_with,
    reset_state as reset_state_with, write_state as write_state_with,
};

// run ID なしの呼び出し。
fn read_state(def: &Path) -> Result<Option<PipelineState>, StateError> {
    read_state_with(def, None)
}
fn reset_state(def: &Path) -> Result<(), StateError> {
    reset_state_with(def, None)
}
fn initial_state(d: &PipelineDefinition, def: &Path) -> PipelineState {
    initial_state_with(d, def, None)
}
fn write_state(def: &Path, s: &PipelineState, dur: Durability) -> Result<(), StateError> {
    write_state_with(def, None, s, dur)
}

const SECRET: &str = "S3CR3T-VALUE-123";

#[test]
fn 全_kind_のエラーの表示に入力の値が現れない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let def = dir.path().join("conductor.toml");
    let state = dir.path().join("conductor.toml.state");
    let mut seen: Vec<StateErrorKind> = Vec::new();

    let mut check = |e: conductor_runtime::StateError| {
        let text = e.to_string();
        assert!(!text.contains(SECRET), "{text}");
        assert!(
            text.contains("conductor.toml.state"),
            "パスは表示してよい: {text}"
        );
        let debug = format!("{e:?}");
        assert!(!debug.contains(SECRET), "{debug}");
        seen.push(e.kind);
    };

    // corrupt(構文、内容の矛盾)
    for content in [
        format!("{SECRET} = = x"),
        format!("format_version = 1\n{SECRET} = 1\n"),
        format!("format_version = 1\npipeline_name = \"{SECRET}\"\n"),
    ] {
        std::fs::write(&state, content).expect("書ける");
        check(read_state(&def).expect_err("拒否される"));
    }
    // versionMismatch
    std::fs::write(&state, format!("format_version = 7\n{SECRET} = 1\n")).expect("書ける");
    check(read_state(&def).expect_err("拒否される"));
    // tooLarge
    std::fs::write(&state, format!("# {SECRET}\n{}", "x".repeat(1024 * 1024))).expect("書ける");
    check(read_state(&def).expect_err("拒否される"));
    // unreadable(ディレクトリ)
    std::fs::remove_file(&state).expect("消せる");
    std::fs::create_dir(&state).expect("作れる");
    check(read_state(&def).expect_err("拒否される"));
    // removeFailed(中身のあるディレクトリ)
    std::fs::write(state.join(SECRET), "x").expect("書ける");
    check(reset_state(&def).expect_err("削除できない"));
    // writeFailed(置き換え先がディレクトリ)
    let d = definition();
    let s = initial_state(&d, &def);
    check(write_state(&def, &s, Durability::Fast).expect_err("書けない"));

    for kind in [
        StateErrorKind::Corrupt,
        StateErrorKind::VersionMismatch,
        StateErrorKind::TooLarge,
        StateErrorKind::Unreadable,
        StateErrorKind::RemoveFailed,
        StateErrorKind::WriteFailed,
    ] {
        assert!(seen.contains(&kind), "{kind:?} を試した");
    }
}

#[test]
fn コマンドエラーの表示に入力の値が現れない() {
    for input in [format!("{SECRET} 'unclosed"), format!("\"{SECRET}")] {
        let e = check_command(&input.into()).expect_err("分けられない");
        assert_eq!(e.kind, CommandErrorKind::UnclosedQuote);
        assert!(!e.to_string().contains(SECRET));
        assert!(!format!("{e:?}").contains(SECRET));
    }
    let e = check_command(&"   ".into()).expect_err("空");
    assert_eq!(e.kind, CommandErrorKind::Empty);
}

fn definition() -> PipelineDefinition {
    PipelineDefinition {
        name: "p".into(),
        description: None,
        run_id_env: None,
        remove_on_complete: false,
        format_version: 1,
        job: vec![common::job(
            "a",
            format!("prog --token={SECRET} 'with {SECRET}'"),
        )],
    }
}

#[test]
fn ステートファイルに_command_と環境変数の値が現れない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let def_path = dir.path().join("conductor.toml");
    let d = definition();
    let state = initial_state(&d, &def_path);
    let execution = JobExecution {
        job_id: "a".into(),
        command: d.job[0].command.clone(),
        started: true,
        exit_code: Some(3),
        terminated_by_signal: None,
        start_failure: false,
        interrupted: false,
        signal_number: None,
    };
    let outcome = JobOutcome {
        job_id: "a".into(),
        classification: Some(Classification::Error),
        reported_exit_code: 3,
        conductor_error: false,
    };
    let next = NextStep {
        job_id: "a".into(),
        next_job_id: None,
        skipped_job_ids: vec![],
        skip_reason_kind: None,
        end_reached: false,
        completed: false,
        resume_job_id: Some("a".into()),
    };
    let recorded = apply_outcome(&state, "a", &execution, &outcome, &next);
    write_state(&def_path, &recorded, Durability::Durable).expect("書ける");
    let text = std::fs::read_to_string(dir.path().join("conductor.toml.state")).expect("読める");
    assert!(!text.contains(SECRET), "{text}");
    assert!(!text.contains("prog"), "command は書かない: {text}");
    // 環境変数の値(PATH など)も書かない。
    if let Ok(path) = std::env::var("PATH") {
        assert!(!text.contains(&path));
    }
    assert!(text.contains("exit_code = 3"));
}
