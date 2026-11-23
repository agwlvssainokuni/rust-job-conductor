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

//! `read_state_raw`(完了のステートも返す読み取り)と `read_state` の差のテスト(FR3.1、FR3.2、FR3.4)。

mod common;

use common::{def, put};
use conductor_core::RunId;
use conductor_runtime::{StateErrorKind, read_state, read_state_raw};

const COMPLETED: &str = "format_version = 1\ndefinition_key = \"x.toml\"\ndefinition_path = \"x.toml\"\npipeline_name = \"p\"\njob_order = [\"a\"]\nend_reached = false\ncompleted = true\n\n[[job]]\nid = \"a\"\nstatus = \"success\"\nexit_code = 0\n";

#[test]
fn fr3_1_raw_は完了のステートを返し_read_state_は空として返す() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    put(dir.path(), "x.toml.state", COMPLETED);
    let raw = read_state_raw(&def(dir.path()), None)
        .expect("読める")
        .expect("完了のステートが返る");
    assert!(raw.completed);
    assert_eq!(raw.pipeline_name, "p");
    assert_eq!(read_state(&def(dir.path()), None), Ok(None));
    // 読み取りでは、ファイルを消さない
    assert!(dir.path().join("x.toml.state").exists());
}

#[test]
fn fr3_1_raw_は_run_id_ありの完了のステートも返す() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let a = RunId::parse("a").expect("有効な run ID");
    put(dir.path(), "x.toml.a.state", COMPLETED);
    // 中身の run_id がない(ファイル名の run ID と食い違う)ものは、両方とも同じ誤り
    let err_raw = read_state_raw(&def(dir.path()), Some(&a)).unwrap_err();
    let err = read_state(&def(dir.path()), Some(&a)).unwrap_err();
    assert_eq!(err_raw.kind, err.kind);
    // 正しい run_id を持つ完了のステート
    let with_id = COMPLETED.replace("completed = true\n", "completed = true\nrun_id = \"a\"\n");
    put(dir.path(), "x.toml.a.state", &with_id);
    let raw = read_state_raw(&def(dir.path()), Some(&a))
        .expect("読める")
        .expect("返る");
    assert!(raw.completed);
    assert_eq!(raw.run_id.as_deref(), Some("a"));
    assert_eq!(read_state(&def(dir.path()), Some(&a)), Ok(None));
}

#[test]
fn fr3_2_ファイルなしと長さ_0_は両方とも空() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    assert_eq!(read_state_raw(&def(dir.path()), None), Ok(None));
    put(dir.path(), "x.toml.state", "");
    assert_eq!(read_state_raw(&def(dir.path()), None), Ok(None));
    assert_eq!(read_state(&def(dir.path()), None), Ok(None));
}

#[test]
fn fr3_4_版違い_壊れたファイル_run_id_の食い違いは_raw_も_read_state_と同じ誤り() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let cases: [(&str, StateErrorKind); 2] = [
        (
            &COMPLETED.replace("format_version = 1", "format_version = 2"),
            StateErrorKind::VersionMismatch,
        ),
        ("これは TOML ではない = = =", StateErrorKind::Corrupt),
    ];
    for (body, kind) in cases {
        put(dir.path(), "x.toml.state", body);
        let raw = read_state_raw(&def(dir.path()), None).unwrap_err();
        let plain = read_state(&def(dir.path()), None).unwrap_err();
        assert_eq!(raw.kind, kind);
        assert_eq!(plain.kind, kind);
        assert_eq!(raw.to_string(), plain.to_string());
    }
    // run ID なしの読み取りで、中身に run_id がある(食い違い)
    let with_id = COMPLETED.replace("completed = true\n", "completed = true\nrun_id = \"zz\"\n");
    put(dir.path(), "x.toml.state", &with_id);
    let raw = read_state_raw(&def(dir.path()), None).unwrap_err();
    let plain = read_state(&def(dir.path()), None).unwrap_err();
    assert_eq!(raw.kind, StateErrorKind::Corrupt);
    assert_eq!(raw.kind, plain.kind);
}
