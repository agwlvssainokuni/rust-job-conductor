//! `read_state` の統合テスト(ファイルを使う)。BR2.1〜BR2.3、NFR4.3、NFR4.5、AC4.1.3、AC5.1.2。

mod common;

use std::path::{Path, PathBuf};

use conductor_core::JobStatus;
use conductor_runtime::{StateErrorKind, read_state};

const VALID: &str = r#"format_version = 1
definition_key = "conductor.toml"
definition_path = "conductor.toml"
pipeline_name = "p"
job_order = ["a", "b"]
end_reached = false
completed = false
resume_job_id = "b"

[[jobs]]
id = "a"
status = "success"
exit_code = 0

[[jobs]]
id = "b"
status = "pending"
"#;

/// 定義ファイルのパス(ファイルそのものは無くてよい)と、ステートファイルのパス。
fn paths(dir: &Path) -> (PathBuf, PathBuf) {
    (dir.join("conductor.toml"), dir.join("conductor.toml.state"))
}

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
fn 終わりのない入力でも上限で止まる() {
    // 大きさが 0 と報告されるが、読むと終わらないファイル。
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (def, state) = paths(dir.path());
    std::os::unix::fs::symlink("/dev/zero", &state).expect("リンクを作れる");
    let e = common::run_with_limit(|| {}, move || read_state(&def)).expect_err("拒否される");
    assert_eq!(e.kind, StateErrorKind::TooLarge);
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
