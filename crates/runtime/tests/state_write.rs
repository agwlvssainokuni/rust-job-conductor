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

//! `write_state`、`reset_state` の統合テスト(ファイルを使う)。BR2.2、BR2.4、BR2.6、NFR4.4、
//! AC4.1.2、AC4.3.3、AC6.1.1、AC6.1.2。失敗は、権限に頼らず、置き換え先をディレクトリにして起こす。

mod common;

use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use common::without_run_id::{read_state, reset_state, write_state};
use conductor_core::{JobState, JobStatus, PipelineState, SkipReason};
use conductor_runtime::{Durability, StateErrorKind};

fn paths(dir: &Path) -> (PathBuf, PathBuf) {
    (dir.join("conductor.toml"), dir.join("conductor.toml.state"))
}

fn sample() -> PipelineState {
    PipelineState {
        definition_key: "dir/conductor.toml".into(),
        pipeline_name: "nightly".into(),
        definition_path: "dir/conductor.toml".into(),
        job_order: vec!["a".into(), "b".into(), "c".into(), "d".into()],
        job_states: vec![
            JobState {
                job_id: "a".into(),
                status: JobStatus::Success,
                exit_code: Some(0),
                skip_reason: None,
            },
            JobState {
                job_id: "b".into(),
                status: JobStatus::Warning,
                exit_code: Some(2),
                skip_reason: None,
            },
            JobState {
                job_id: "c".into(),
                status: JobStatus::Skipped,
                exit_code: None,
                skip_reason: Some(SkipReason::ByEnd),
            },
            JobState {
                job_id: "d".into(),
                status: JobStatus::Error,
                exit_code: Some(-1),
                skip_reason: None,
            },
        ],
        resume_job_id: Some("d".into()),
        end_reached: true,
        completed: false,
        run_id: None,
    }
}

fn temp_files(dir: &Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .expect("読める")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.contains(".tmp."))
        .collect()
}

#[test]
fn 書いたステートを読み戻すと等しい() {
    for durability in [Durability::Durable, Durability::Fast] {
        let dir = tempfile::tempdir().expect("一時ディレクトリ");
        let (def, state_path) = paths(dir.path());
        let mut state = sample();
        write_state(&def, &state, durability).expect("書ける");
        assert!(state_path.exists());
        assert_eq!(read_state(&def), Ok(Some(state.clone())), "{durability:?}");

        // 上書きできて、一時ファイルは残らない。
        state.resume_job_id = None;
        state.job_states[3].status = JobStatus::Success;
        write_state(&def, &state, durability).expect("上書きできる");
        assert_eq!(read_state(&def), Ok(Some(state)));
        assert!(temp_files(dir.path()).is_empty());
    }
}

#[test]
fn ステートファイルの内容は版つきで_e3_のキーを持つ() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (def, state_path) = paths(dir.path());
    write_state(&def, &sample(), Durability::Durable).expect("書ける");
    let text = std::fs::read_to_string(state_path).expect("読める");
    let table: toml::Table = text.parse().expect("TOML");
    assert_eq!(table["format_version"].as_integer(), Some(1));
    assert_eq!(table["definition_key"], table["definition_path"]);
    for key in [
        "pipeline_name",
        "job_order",
        "job",
        "resume_job_id",
        "end_reached",
        "completed",
    ] {
        assert!(table.contains_key(key), "{key}");
    }
    assert!(!table.contains_key("jobs"), "旧キーは書かない");
    let jobs = table["job"].as_array().expect("配列");
    assert_eq!(jobs[2]["skip_reason"].as_str(), Some("byEnd"));
    assert_eq!(jobs[3]["exit_code"].as_integer(), Some(-1));
    assert!(text.contains("[[job]]"));
    assert!(!text.contains("[[jobs]]"));
}

#[test]
fn 置き換え先がディレクトリなら_write_failed_でステートは変わらず一時ファイルも残らない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (def, state_path) = paths(dir.path());
    std::fs::create_dir(&state_path).expect("作れる");
    std::fs::write(state_path.join("keep.txt"), "keep").expect("書ける");
    for durability in [Durability::Durable, Durability::Fast] {
        let e = write_state(&def, &sample(), durability).expect_err("失敗する");
        assert_eq!(e.kind, StateErrorKind::WriteFailed);
        assert_eq!(e.path, state_path);
        assert!(state_path.is_dir(), "ステートの場所は変わらない");
        assert!(state_path.join("keep.txt").exists());
        assert!(
            temp_files(dir.path()).is_empty(),
            "自分が作った一時ファイルは消す"
        );
    }
}

#[test]
fn 失敗しても以前のステートは壊れない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (def, _) = paths(dir.path());
    let first = sample();
    write_state(&def, &first, Durability::Durable).expect("書ける");
    // 存在しないディレクトリへの書き込みは失敗する(別の定義の場所)。
    let missing = dir.path().join("no-such-dir").join("conductor.toml");
    let e = write_state(&missing, &first, Durability::Durable).expect_err("失敗する");
    assert_eq!(e.kind, StateErrorKind::WriteFailed);
    assert_eq!(read_state(&def), Ok(Some(first)));
}

#[test]
fn 残った一時ファイルは読み込みに影響しない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (def, _) = paths(dir.path());
    let state = sample();
    write_state(&def, &state, Durability::Durable).expect("書ける");
    std::fs::write(
        dir.path().join(".conductor.toml.state.tmp.1.1.1"),
        "garbage = = =",
    )
    .expect("書ける");
    assert_eq!(read_state(&def), Ok(Some(state.clone())));
    write_state(&def, &state, Durability::Fast).expect("書ける");
    assert_eq!(read_state(&def), Ok(Some(state)));
}

#[test]
fn 古い一時ファイルは_durable_の書き込みのあとだけ消える() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (def, _) = paths(dir.path());
    let old = dir.path().join(".conductor.toml.state.tmp.9.9.9");
    let fresh = dir.path().join(".conductor.toml.state.tmp.8.8.8");
    std::fs::write(&old, "x").expect("書ける");
    std::fs::write(&fresh, "x").expect("書ける");
    let f = OpenOptions::new().write(true).open(&old).expect("開ける");
    f.set_modified(SystemTime::now() - Duration::from_secs(25 * 60 * 60))
        .expect("更新時刻を変えられる");
    drop(f);

    write_state(&def, &sample(), Durability::Fast).expect("書ける");
    assert!(old.exists(), "Fast では後始末をしない");

    write_state(&def, &sample(), Durability::Durable).expect("書ける");
    assert!(!old.exists(), "24 時間以上たった一時ファイルは消える");
    assert!(fresh.exists(), "新しい一時ファイルは残る");
}

#[cfg(unix)]
#[test]
fn 書けないディレクトリは_write_failed() {
    use std::os::unix::fs::PermissionsExt;
    if common::is_root() {
        eprintln!("スキップ: root のため、権限による書き込みの失敗を試せません");
        return;
    }
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let sub = dir.path().join("ro");
    std::fs::create_dir(&sub).expect("作れる");
    let def = sub.join("conductor.toml");
    std::fs::set_permissions(&sub, std::fs::Permissions::from_mode(0o555))
        .expect("権限を変えられる");
    let result = write_state(&def, &sample(), Durability::Fast);
    std::fs::set_permissions(&sub, std::fs::Permissions::from_mode(0o755)).expect("戻せる");
    assert_eq!(
        result.expect_err("失敗する").kind,
        StateErrorKind::WriteFailed
    );
    assert!(!sub.join("conductor.toml.state").exists());
}

#[test]
fn 初期化はステートを消し_無くても成功する() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (def, state_path) = paths(dir.path());
    assert_eq!(reset_state(&def), Ok(()), "存在しなくても成功");
    write_state(&def, &sample(), Durability::Durable).expect("書ける");
    assert_eq!(reset_state(&def), Ok(()));
    assert!(!state_path.exists());
    assert_eq!(read_state(&def), Ok(None));
}

#[test]
fn 初期化は破損や版の違いの検査を迂回する() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (def, state_path) = paths(dir.path());
    for content in ["= = broken", "format_version = 9\n", "\u{0}\u{1}"] {
        std::fs::write(&state_path, content).expect("書ける");
        assert!(read_state(&def).is_err());
        assert_eq!(reset_state(&def), Ok(()));
        assert!(!state_path.exists());
    }
}

#[test]
fn 削除に失敗したら_remove_failed() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (def, state_path) = paths(dir.path());
    std::fs::create_dir(&state_path).expect("作れる");
    std::fs::write(state_path.join("child"), "x").expect("書ける");
    let e = reset_state(&def).expect_err("削除できない");
    assert_eq!(e.kind, StateErrorKind::RemoveFailed);
    assert_eq!(e.path, state_path);
}
