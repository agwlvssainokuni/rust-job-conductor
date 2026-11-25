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
//!
//! v0.10.0 の U6 で、ステートに試行の開始と終了の時刻だけを書くことを認めた。それ以外(コマンド・
//! 環境変数・出力)は今までどおり書かないことを確かめる。

mod common;

use common::without_run_id::{initial_state, read_state, reset_state, write_state};
use conductor_core::{
    Classification, Command, JobExecution, JobOutcome, NextStep, PipelineDefinition, Timestamp,
    apply_outcome, end_attempt, parse_rfc3339, start_attempt,
};
use conductor_runtime::{CommandErrorKind, Durability, StateErrorKind, check_command};

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
        let e = check_command(&Command::Line(input)).expect_err("分けられない");
        assert_eq!(e.kind, CommandErrorKind::UnclosedQuote);
        assert!(!e.to_string().contains(SECRET));
        assert!(!format!("{e:?}").contains(SECRET));
    }
    let e = check_command(&Command::Line("   ".into())).expect_err("空");
    assert_eq!(e.kind, CommandErrorKind::Empty);
}

fn definition() -> PipelineDefinition {
    common::definition(
        "p",
        vec![common::job(
            "a",
            format!("prog --token={SECRET} 'with {SECRET}'"),
        )],
    )
}

#[test]
fn ステートファイルに_command_と環境変数の値が現れない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let def_path = dir.path().join("conductor.toml");
    let d = definition();
    let state = initial_state(&d, &def_path);
    let execution = JobExecution::exited(3);
    let outcome = JobOutcome {
        classification: Some(Classification::Error),
        reported_exit_code: 3,
        conductor_error: false,
    };
    let next = NextStep {
        next_job_id: None,
        skipped_job_ids: vec![],
        skip_reason_kind: None,
        end_reached: false,
        completed: false,
        resume_job_id: Some("a".into()),
    };
    // 試行の時刻(v0.10.0 の U6)を記録してから、結果を記録する。
    let ts = |secs| Timestamp::from_unix_seconds(secs).expect("範囲の中");
    let state = start_attempt(&state, "a", ts(1_795_597_968));
    let state = end_attempt(&state, "a", ts(1_795_597_985));
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

    // 足してよいのは、開始と終了の時刻のキーと、その時刻の値だけ(U6 の BR2.2、FR6.3)。
    let table: toml::Table = text.parse().expect("TOML");
    let jobs = table["job"].as_array().expect("配列");
    for job in jobs {
        let job = job.as_table().expect("表");
        for key in job.keys() {
            assert!(
                ["id", "status", "exit_code", "skip_reason", "attempt"].contains(&key.as_str()),
                "ジョブの表のキー: {key}"
            );
        }
        for attempt in job
            .get("attempt")
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
        {
            let attempt = attempt.as_table().expect("表");
            for (key, value) in attempt {
                assert!(
                    ["started_at", "ended_at"].contains(&key.as_str()),
                    "試行のキー: {key}"
                );
                let value = value.as_str().expect("文字列");
                assert!(parse_rfc3339(value).is_some(), "時刻の値だけ: {value}");
            }
        }
    }
    assert!(
        text.contains("started_at = \"2026-11-25T09:12:48Z\""),
        "{text}"
    );
    assert!(
        text.contains("ended_at = \"2026-11-25T09:13:05Z\""),
        "{text}"
    );
}

#[test]
fn 試行の時刻の誤りの表示に入力の値が現れない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let def = dir.path().join("conductor.toml");
    let state = dir.path().join("conductor.toml.state");
    let content = format!(
        "format_version = 1\ndefinition_key = \"c\"\ndefinition_path = \"c\"\npipeline_name = \"p\"\n\
         job_order = [\"a\"]\nend_reached = false\ncompleted = false\n\n\
         [[job]]\nid = \"a\"\nstatus = \"success\"\nexit_code = 0\n\n\
         [[job.attempt]]\nstarted_at = \"{SECRET}\"\n"
    );
    std::fs::write(&state, content).expect("書ける");
    let e = read_state(&def).expect_err("拒否される");
    assert_eq!(e.kind, StateErrorKind::Corrupt);
    assert!(!e.to_string().contains(SECRET), "{e}");
    assert!(!format!("{e:?}").contains(SECRET));
}
