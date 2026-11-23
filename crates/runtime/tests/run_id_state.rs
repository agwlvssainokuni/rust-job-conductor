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

//! run ID ごとのステート(FR3)と `reset --all`(FR5。NFR2)の統合テスト。実際のファイルを使う。

mod common;

use std::path::Path;

use common::{def, put};
use conductor_core::{JobState, PipelineState, RunId};
use conductor_runtime::{
    Durability, StateErrorKind, StateLocation, read_state, reset_all_states, reset_state,
    write_state,
};

fn id(v: &str) -> RunId {
    RunId::parse(v).expect("有効な run ID")
}

fn state(run_id: Option<&str>, completed: bool) -> PipelineState {
    PipelineState {
        pipeline_name: "p".into(),
        definition_path: "x.toml".into(),
        job_states: vec![JobState::pending("a"), JobState::pending("b")],
        resume_job_id: if completed { None } else { Some("a".into()) },
        end_reached: false,
        completed,
        run_id: run_id.map(str::to_string),
    }
}

/// 中身の `run_id`(あれば)つきの、正しいステートの文字列。
fn text(run_id: Option<&str>) -> String {
    let mut t = String::from(
        "format_version = 1\ndefinition_key = \"x.toml\"\ndefinition_path = \"x.toml\"\npipeline_name = \"p\"\njob_order = [\"a\"]\nend_reached = false\ncompleted = false\nresume_job_id = \"a\"\n",
    );
    if let Some(r) = run_id {
        t.push_str(&format!("run_id = \"{r}\"\n"));
    }
    t.push_str("\n[[job]]\nid = \"a\"\nstatus = \"pending\"\n");
    t
}

fn exists(dir: &Path, name: &str) -> bool {
    std::fs::symlink_metadata(dir.join(name)).is_ok()
}

fn names(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(dir)
        .expect("読める")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

// ---- パスと形(FR3.1、NFR1) ----

#[test]
fn run_id_の有無でパスが変わる() {
    let d = Path::new("dir/x.toml");
    assert_eq!(
        StateLocation::of(d, None).path(),
        Path::new("dir/x.toml.state")
    );
    assert_eq!(
        StateLocation::of(d, Some(&id("a"))).path(),
        Path::new("dir/x.toml.a.state")
    );
}

#[test]
fn run_id_ありで書いて読み戻せる() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let a = id("a");
    let s = state(Some("a"), false);
    write_state(&def(dir.path()), Some(&a), &s, Durability::Durable).expect("書ける");
    assert!(exists(dir.path(), "x.toml.a.state"));
    assert!(!exists(dir.path(), "x.toml.state"));
    let body = std::fs::read_to_string(dir.path().join("x.toml.a.state")).unwrap();
    assert!(body.contains("run_id = \"a\""), "{body}");
    assert_eq!(read_state(&def(dir.path()), Some(&a)), Ok(Some(s)));
}

#[test]
fn run_id_なしのステートの形は変わらない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let s = state(None, false);
    write_state(&def(dir.path()), None, &s, Durability::Durable).expect("書ける");
    assert_eq!(names(dir.path()), ["x.toml.state"]);
    let body = std::fs::read_to_string(dir.path().join("x.toml.state")).unwrap();
    assert!(!body.contains("run_id"), "{body}");
    assert_eq!(read_state(&def(dir.path()), None), Ok(Some(s)));
}

// ---- 照合(FR3.2) ----

fn corrupt_kind(r: Result<Option<PipelineState>, conductor_runtime::StateError>) -> bool {
    matches!(r, Err(e) if e.kind == StateErrorKind::Corrupt)
}

#[test]
fn 中身の_run_id_がファイル名と一致しなければ_corrupt() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let a = id("a");
    // 一致
    put(dir.path(), "x.toml.a.state", &text(Some("a")));
    assert!(matches!(
        read_state(&def(dir.path()), Some(&a)),
        Ok(Some(_))
    ));
    // 値が違う
    put(dir.path(), "x.toml.a.state", &text(Some("b")));
    assert!(corrupt_kind(read_state(&def(dir.path()), Some(&a))));
    // 中身にない(x.state を x.a.state にコピーした場合)
    put(dir.path(), "x.toml.a.state", &text(None));
    assert!(corrupt_kind(read_state(&def(dir.path()), Some(&a))));
    // run ID なしで読むのに、中身にある
    put(dir.path(), "x.toml.state", &text(Some("a")));
    assert!(corrupt_kind(read_state(&def(dir.path()), None)));
    // 大文字小文字だけが違っても一致しない
    put(dir.path(), "x.toml.a.state", &text(Some("A")));
    assert!(corrupt_kind(read_state(&def(dir.path()), Some(&a))));
}

#[test]
fn 完了のステートでも_run_id_は検査される() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let a = id("a");
    let done = |r: &str| {
        format!(
            "format_version = 1\ndefinition_key = \"x.toml\"\ndefinition_path = \"x.toml\"\npipeline_name = \"p\"\njob_order = [\"a\"]\nend_reached = true\ncompleted = true\nrun_id = \"{r}\"\n\n[[job]]\nid = \"a\"\nstatus = \"success\"\nexit_code = 0\n"
        )
    };
    put(dir.path(), "x.toml.a.state", &done("a"));
    assert_eq!(read_state(&def(dir.path()), Some(&a)), Ok(None));
    put(dir.path(), "x.toml.a.state", &done("b"));
    assert!(corrupt_kind(read_state(&def(dir.path()), Some(&a))));
}

#[test]
fn 照合の失敗の文に_run_id_の値が入らない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    put(dir.path(), "x.toml.sekret1.state", &text(Some("sekret2")));
    let e = read_state(&def(dir.path()), Some(&id("sekret1"))).expect_err("食い違い");
    let shown = e.to_string();
    // 中身の値は出さない。ステートファイルのパス(名前に run ID を含む)だけは、従来どおり出る。
    assert!(!shown.contains("sekret2"), "{shown}");
    assert_eq!(
        shown.replace(&e.path.display().to_string(), "<path>"),
        "ステートファイルが壊れています: <path> (run_id がファイル名の run ID と一致しません)"
    );
}

// ---- 独立(FR3.3) ----

#[test]
fn run_id_が違えばステートは独立する() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let d = def(dir.path());
    let (a, b) = (id("a"), id("b"));
    let mut sa = state(Some("a"), false);
    sa.resume_job_id = Some("a".into());
    let mut sb = state(Some("b"), false);
    sb.resume_job_id = Some("b".into());
    let none = state(None, false);
    write_state(&d, Some(&a), &sa, Durability::Durable).unwrap();
    write_state(&d, Some(&b), &sb, Durability::Durable).unwrap();
    write_state(&d, None, &none, Durability::Durable).unwrap();

    assert_eq!(read_state(&d, Some(&a)), Ok(Some(sa.clone())));
    assert_eq!(read_state(&d, Some(&b)), Ok(Some(sb.clone())));
    assert_eq!(read_state(&d, None), Ok(Some(none.clone())));

    // a を書き換えても、b と run ID なしは変わらない
    sa.resume_job_id = Some("b".into());
    write_state(&d, Some(&a), &sa, Durability::Durable).unwrap();
    assert_eq!(read_state(&d, Some(&b)), Ok(Some(sb.clone())));
    assert_eq!(read_state(&d, None), Ok(Some(none.clone())));

    // a を消しても、b と run ID なしは残る
    reset_state(&d, Some(&a)).unwrap();
    assert_eq!(read_state(&d, Some(&a)), Ok(None));
    assert_eq!(read_state(&d, Some(&b)), Ok(Some(sb)));
    assert_eq!(read_state(&d, None), Ok(Some(none)));
    // 存在しない run ID の reset は成功
    assert_eq!(reset_state(&d, Some(&id("zzz"))), Ok(()));
}

// ---- reset --all(FR5、NFR2) ----

#[test]
fn reset_all_は対象だけを消し_別の定義と壊れたものと一時ファイルを残す() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let p = dir.path();
    // 消すもの
    put(p, "x.toml.state", &text(None));
    put(p, "x.toml.a.state", &text(Some("a")));
    put(p, "x.toml.b.state", &text(Some("b")));
    // 消さないもの
    put(p, "x.toml.a.b.state", &text(Some("b"))); // 別の定義 `x.toml.a` の run ID `b`(値が名前と違う)
    put(p, "x.toml.c.state", "これは TOML ではない = = =");
    put(p, ".x.toml.state.tmp.1.2.3", "tmp");
    put(p, ".x.toml.a.state.tmp.1.2.3", "tmp");
    put(p, "x.toml.bak", "bak");
    put(p, "x.toml.a.statex", &text(Some("a")));
    put(p, "x.toml.state.bak", &text(None));
    put(p, "x.toml..state", &text(Some("")));
    put(p, "x.toml.a b.state", &text(Some("a b")));
    put(p, "x.toml", "定義");
    put(p, "y.toml.a.state", &text(Some("a")));
    put(p, "xx.toml.a.state", &text(Some("a")));

    let report = reset_all_states(&def(p)).expect("成功");
    assert_eq!(report.kept, ["x.toml.c.state"]);
    assert_eq!(
        names(p),
        [
            ".x.toml.a.state.tmp.1.2.3",
            ".x.toml.state.tmp.1.2.3",
            "x.toml",
            "x.toml..state",
            "x.toml.a b.state",
            "x.toml.a.b.state",
            "x.toml.a.statex",
            "x.toml.bak",
            "x.toml.c.state",
            "x.toml.state.bak",
            "xx.toml.a.state",
            "y.toml.a.state",
        ]
        .map(String::from)
    );
}

#[test]
fn reset_all_は別の定義のステートで_run_id_のないものを消さない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let p = dir.path();
    // 定義 `x.toml.a` の run ID なしのステートは `x.toml.a.state`。`x.toml` の run ID `a` と同じ名前。
    put(p, "x.toml.a.state", &text(None));
    let report = reset_all_states(&def(p)).expect("成功");
    assert!(report.kept.is_empty(), "壊れてはいないので、警告もしない");
    assert!(exists(p, "x.toml.a.state"));
}

#[test]
fn reset_all_は何もなくても成功し_ほかのディレクトリに触れない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let p = dir.path();
    assert_eq!(reset_all_states(&def(p)), Ok(Default::default()));
    // 隣のディレクトリ(定義と同じ名前のファイルを持つ)には触れない
    std::fs::create_dir(p.join("sub")).unwrap();
    put(&p.join("sub"), "x.toml.state", &text(None));
    put(&p.join("sub"), "x.toml.a.state", &text(Some("a")));
    // 名前がステートに見えるディレクトリは消さない
    std::fs::create_dir(p.join("x.toml.d.state")).unwrap();
    let report = reset_all_states(&def(p)).expect("成功");
    assert!(report.kept.is_empty());
    assert!(exists(&p.join("sub"), "x.toml.state"));
    assert!(exists(&p.join("sub"), "x.toml.a.state"));
    assert!(exists(p, "x.toml.d.state"));
}

#[test]
fn reset_all_は定義のあるディレクトリがなければ成功する() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let gone = dir.path().join("nothing").join("x.toml");
    assert_eq!(reset_all_states(&gone), Ok(Default::default()));
}

#[test]
fn reset_all_は完了のまま残った一致するステートを消す() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let p = dir.path();
    let done = "format_version = 1\ndefinition_key = \"x.toml\"\ndefinition_path = \"x.toml\"\npipeline_name = \"p\"\njob_order = [\"a\"]\nend_reached = true\ncompleted = true\nrun_id = \"a\"\n\n[[job]]\nid = \"a\"\nstatus = \"success\"\nexit_code = 0\n";
    put(p, "x.toml.a.state", done);
    reset_all_states(&def(p)).expect("成功");
    assert!(!exists(p, "x.toml.a.state"));
}

#[test]
fn reset_all_は大きすぎる_空_版違いのファイルを消さずに報告する() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let p = dir.path();
    put(p, "x.toml.big.state", &"a".repeat(1024 * 1024 + 1));
    put(
        p,
        "x.toml.ver.state",
        &text(Some("ver")).replace("format_version = 1", "format_version = 2"),
    );
    put(p, "x.toml.empty.state", "");
    let report = reset_all_states(&def(p)).expect("成功");
    assert_eq!(report.kept, ["x.toml.big.state", "x.toml.ver.state"]);
    assert!(exists(p, "x.toml.big.state"));
    assert!(exists(p, "x.toml.ver.state"));
    assert!(
        exists(p, "x.toml.empty.state"),
        "空は、だれのものか分からないので残す"
    );
}

#[cfg(unix)]
#[test]
fn reset_all_はシンボリックリンクを辿らず消さない() {
    use std::os::unix::fs::symlink;
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let outside = tempfile::tempdir().expect("外のディレクトリ");
    let p = dir.path();
    // 中身が一致する本物のファイルへのリンク(外にある)
    put(outside.path(), "real.state", &text(Some("a")));
    symlink(outside.path().join("real.state"), p.join("x.toml.a.state")).unwrap();
    // run ID なしのステートへのリンク
    put(outside.path(), "plain.state", &text(None));
    symlink(outside.path().join("plain.state"), p.join("x.toml.state")).unwrap();
    // ディレクトリへのリンク
    symlink(outside.path(), p.join("x.toml.dir.state")).unwrap();
    // 壊れたリンク
    symlink(p.join("nowhere"), p.join("x.toml.dead.state")).unwrap();

    let report = reset_all_states(&def(p)).expect("成功");
    assert!(report.kept.is_empty());
    assert!(exists(p, "x.toml.a.state"));
    assert!(exists(p, "x.toml.state"));
    assert!(exists(p, "x.toml.dir.state"));
    assert!(exists(p, "x.toml.dead.state"));
    assert!(exists(outside.path(), "real.state"));
    assert!(exists(outside.path(), "plain.state"));
}

#[cfg(unix)]
#[test]
fn reset_all_の削除の失敗は_remove_failed() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let p = dir.path();
    put(p, "x.toml.a.state", &text(Some("a")));
    std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o555)).unwrap();
    let result = reset_all_states(&def(p));
    std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o755)).unwrap();
    // root では、権限があっても消せてしまうので、その場合は確かめない
    if let Err(e) = result {
        assert_eq!(e.kind, StateErrorKind::RemoveFailed);
        assert!(exists(p, "x.toml.a.state"));
    }
}
