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

//! `read_state` の統合テスト(ファイルを使う)。BR2.1〜BR2.3、NFR4.3、NFR4.5、AC4.1.3、AC5.1.2。

mod common;

use common::paths;
use common::without_run_id::read_state;
use conductor_core::JobStatus;
use conductor_runtime::StateErrorKind;

const VALID: &str = r#"format_version = 1
definition_key = "conductor.toml"
definition_path = "conductor.toml"
pipeline_name = "p"
job_order = ["a", "b"]
end_reached = false
completed = false
resume_job_id = "b"

[[job]]
id = "a"
status = "success"
exit_code = 0

[[job]]
id = "b"
status = "pending"
"#;

#[test]
fn ステートファイルが無ければ空() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (def, _) = paths(dir.path());
    assert_eq!(read_state(&def), Ok(None));
}

#[test]
fn 長さ_0_のファイルは空() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (def, state) = paths(dir.path());
    std::fs::write(&state, b"").expect("書ける");
    assert_eq!(read_state(&def), Ok(None));
}

#[test]
fn 定義ファイルと同じディレクトリのステートを読む() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (def, state) = paths(dir.path());
    std::fs::write(&state, VALID).expect("書ける");
    let s = read_state(&def).expect("読める").expect("空ではない");
    assert_eq!(s.pipeline_name, "p");
    assert_eq!(s.job_states[0].status, JobStatus::Success);
    // 別の定義ファイル名は、別のステート。
    assert_eq!(read_state(&dir.path().join("other.toml")), Ok(None));
}

#[test]
fn completed_のまま残ったステートは空として返しファイルは消さない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (def, state) = paths(dir.path());
    let text = VALID
        .replace("completed = false", "completed = true")
        .replace("resume_job_id = \"b\"\n", "");
    std::fs::write(&state, text).expect("書ける");
    assert_eq!(read_state(&def), Ok(None));
    assert!(state.exists(), "読み取りでは消さない");
}

#[test]
fn 上限ちょうどは受理し_1_バイト超えは_too_large() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (def, state) = paths(dir.path());
    let limit = 1024 * 1024;

    let mut exact = String::from(VALID);
    exact.push('#');
    let pad = limit - exact.len();
    exact.push_str(&"x".repeat(pad));
    assert_eq!(exact.len(), limit);
    std::fs::write(&state, &exact).expect("書ける");
    assert!(
        matches!(read_state(&def), Ok(Some(_))),
        "1 MiB ちょうどは受理"
    );

    exact.push('x');
    std::fs::write(&state, &exact).expect("書ける");
    let e = read_state(&def).expect_err("上限を超える");
    assert_eq!(e.kind, StateErrorKind::TooLarge);
    assert_eq!(e.path, state);
}

#[cfg(unix)]
#[test]
fn 文字デバイスへのリンクのステートは開かずに_unreadable() {
    // `/dev/zero`(通常のファイルでないもの)へのシンボリックリンクのステートは、開かずに `unreadable`
    // (形 B。終わりのない読み取りに入らない)。
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (def, state) = paths(dir.path());
    std::os::unix::fs::symlink("/dev/zero", &state).expect("リンクを作れる");
    let e = common::run_with_limit(|| {}, move || read_state(&def)).expect_err("拒否される");
    assert_eq!(e.kind, StateErrorKind::Unreadable);
}

#[test]
fn 不正な_utf8_は_corrupt() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (def, state) = paths(dir.path());
    std::fs::write(&state, [0xff, 0xfe, 0x00, b'a']).expect("書ける");
    let e = read_state(&def).expect_err("拒否される");
    assert_eq!(e.kind, StateErrorKind::Corrupt);
}

#[test]
fn 壊れた_toml_は_corrupt_で内容を表示に出さない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (def, state) = paths(dir.path());
    std::fs::write(&state, "secret-value = = oops\n").expect("書ける");
    let e = read_state(&def).expect_err("拒否される");
    assert_eq!(e.kind, StateErrorKind::Corrupt);
    let text = e.to_string();
    assert!(text.contains("conductor.toml.state"), "{text}");
    assert!(
        !text.contains("secret-value") && !text.contains("oops"),
        "{text}"
    );
}

#[test]
fn 将来の版は_version_mismatch_で未知のキーに影響されない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (def, state) = paths(dir.path());
    std::fs::write(
        &state,
        "format_version = 2\nbrand_new = [1, 2]\n[x]\ny = 1\n",
    )
    .expect("書ける");
    assert_eq!(
        read_state(&def).expect_err("拒否される").kind,
        StateErrorKind::VersionMismatch
    );
    std::fs::write(&state, "format_version = 0\n").expect("書ける");
    assert_eq!(
        read_state(&def).expect_err("拒否される").kind,
        StateErrorKind::VersionMismatch
    );
}

#[test]
fn 内容の矛盾は_corrupt() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (def, state) = paths(dir.path());
    let cases = [
        VALID.replace("completed = false", "completed = false\nextra = 1"),
        VALID.replace("end_reached = false\n", ""),
        VALID.replace("status = \"pending\"", "status = \"unknown\""),
        VALID.replace("status = \"pending\"", "status = \"skipped\""),
        VALID.replace(
            "status = \"pending\"",
            "status = \"pending\"\nexit_code = 1",
        ),
        VALID.replace("id = \"b\"", "id = \"a\""),
        VALID.replace("job_order = [\"a\", \"b\"]", "job_order = [\"b\", \"a\"]"),
        VALID.replace("resume_job_id = \"b\"", "resume_job_id = \"z\""),
        VALID.replace("completed = false", "completed = true"),
    ];
    for (i, text) in cases.iter().enumerate() {
        std::fs::write(&state, text).expect("書ける");
        let e = read_state(&def).expect_err("拒否される");
        assert_eq!(e.kind, StateErrorKind::Corrupt, "case {i}");
    }
}

#[test]
fn job_のテーブルを持つ版_1_のステートを読める() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (def, state) = paths(dir.path());
    assert!(VALID.contains("format_version = 1") && VALID.contains("[[job]]"));
    std::fs::write(&state, VALID).expect("書ける");
    let s = read_state(&def).expect("読める").expect("空ではない");
    assert_eq!(s.job_states.len(), 2);
}

#[test]
fn 旧キー_jobs_のステートは_corrupt_で拒否され内容を表示に出さない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (def, state) = paths(dir.path());
    std::fs::write(&state, VALID.replace("[[job]]", "[[jobs]]")).expect("書ける");
    let e = read_state(&def).expect_err("拒否される");
    assert_eq!(e.kind, StateErrorKind::Corrupt);
    let text = e.to_string();
    assert!(!text.contains("jobs"), "{text}");
}

#[test]
fn 読めないファイルは_unreadable() {
    // ステートの場所がディレクトリのとき(開けても、読めない)。
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (def, state) = paths(dir.path());
    std::fs::create_dir(&state).expect("作れる");
    let e = read_state(&def).expect_err("拒否される");
    assert_eq!(e.kind, StateErrorKind::Unreadable);
    assert_eq!(e.path, state);
}

#[cfg(unix)]
#[test]
fn 権限のないファイルは_unreadable() {
    use std::os::unix::fs::PermissionsExt;
    if common::is_root() {
        eprintln!("スキップ: root のため、権限による読み取りの失敗を試せません");
        return;
    }
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (def, state) = paths(dir.path());
    std::fs::write(&state, VALID).expect("書ける");
    std::fs::set_permissions(&state, std::fs::Permissions::from_mode(0o000))
        .expect("権限を変えられる");
    let result = read_state(&def);
    std::fs::set_permissions(&state, std::fs::Permissions::from_mode(0o600)).expect("戻せる");
    assert_eq!(
        result.expect_err("拒否される").kind,
        StateErrorKind::Unreadable
    );
}

/// 5 秒の上限つきで、別のスレッドで `f` を呼ぶ(止まったときに落ちるように)。
#[cfg(unix)]
fn within_5s<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    rx.recv_timeout(std::time::Duration::from_secs(5))
        .expect("5 秒以内に返る(開いて待たない)")
}

/// 書き手のいない名前付きパイプのステートは、開かずに `unreadable`。`reset --all` は止まらず、消さない
/// (2b)。読む権限のない 1 MiB 超のステートは `unreadable`(`tooLarge` ではない)。
#[cfg(unix)]
#[test]
fn 名前付きパイプと権限のない大きなステートは開かずに_unreadable() {
    use conductor_core::RunId;
    use conductor_runtime::{read_state_raw, reset_all_states};
    use std::os::unix::fs::PermissionsExt;

    let mkfifo = |path: &std::path::Path| {
        let status = std::process::Command::new("mkfifo")
            .arg(path)
            .status()
            .expect("mkfifo を起動できる");
        assert!(status.success(), "mkfifo が失敗した");
    };

    // run ID なしのステートの名前付きパイプ: read_state と read_state_raw。
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (def, state) = paths(dir.path());
    mkfifo(&state);
    let d = def.clone();
    let e = within_5s(move || read_state(&d)).expect_err("拒否される");
    assert_eq!(e.kind, StateErrorKind::Unreadable);
    assert_eq!(e.path, state);
    let d = def.clone();
    let e = within_5s(move || read_state_raw(&d, None)).expect_err("拒否される");
    assert_eq!(e.kind, StateErrorKind::Unreadable);

    // run ID つきのステートの名前の名前付きパイプ: reset --all は止まらず、消さず、残したものにも入れない。
    let other = tempfile::tempdir().expect("一時ディレクトリ");
    let (def2, _) = paths(other.path());
    let pipe = other.path().join("conductor.toml.r1.state");
    mkfifo(&pipe);
    let run_id = RunId::parse("r1").expect("有効な run ID");
    let d = def2.clone();
    let e = within_5s(move || read_state_raw(&d, Some(&run_id))).expect_err("拒否される");
    assert_eq!(e.kind, StateErrorKind::Unreadable);
    let report = within_5s(move || reset_all_states(&def2)).expect("成功");
    assert!(report.kept.is_empty(), "{:?}", report.kept);
    assert!(pipe.exists(), "名前付きパイプは消さない");

    // 読む権限のない 1 MiB 超のステートは unreadable(root などで開けてしまうときは飛ばす)。
    if common::is_root() {
        eprintln!("スキップ: root のため、権限による読み取りの失敗を試せません");
        return;
    }
    let big = tempfile::tempdir().expect("一時ディレクトリ");
    let (def3, state3) = paths(big.path());
    std::fs::write(&state3, vec![b'#'; 1024 * 1024 + 1]).expect("書ける");
    std::fs::set_permissions(&state3, std::fs::Permissions::from_mode(0o000))
        .expect("権限を変えられる");
    let result = read_state(&def3);
    std::fs::set_permissions(&state3, std::fs::Permissions::from_mode(0o600)).expect("戻せる");
    assert_eq!(
        result.expect_err("拒否される").kind,
        StateErrorKind::Unreadable
    );
}

/// `read_state_raw`(完了のステートも返す読み取り)と `read_state` の差のテスト(FR3.1、FR3.2、FR3.4)。
///
/// run ID つきの `read_state`(引数 2 つ)を使うので、run ID なしの `read_state`(上のテストが使う
/// `common::without_run_id::read_state`)と名前が重ならないよう、子のモジュールに置く。
mod raw {
    use super::common::{def, put};
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
        let with_id =
            COMPLETED.replace("completed = true\n", "completed = true\nrun_id = \"zz\"\n");
        put(dir.path(), "x.toml.state", &with_id);
        let raw = read_state_raw(&def(dir.path()), None).unwrap_err();
        let plain = read_state(&def(dir.path()), None).unwrap_err();
        assert_eq!(raw.kind, StateErrorKind::Corrupt);
        assert_eq!(raw.kind, plain.kind);
    }
}
