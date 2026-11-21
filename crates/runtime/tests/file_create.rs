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

//! `create_file` の統合テスト(一時ディレクトリの実ファイル)。U4 の BR1.2、BR1.3、BR1.6。
//! 権限とシンボリックリンクのテストは、実行時に効果を確かめ、効かない環境では確かめずに通す。

use std::fs;
use std::path::Path;

use conductor_runtime::{CreateError, CreateOutcome, create_file};

/// ディレクトリの中の、一時ファイル(先頭が `.`)の数。
fn hidden_files(dir: &Path) -> usize {
    fs::read_dir(dir)
        .expect("読める")
        .filter_map(Result::ok)
        .filter(|e| e.file_name().to_string_lossy().starts_with('.'))
        .count()
}

#[test]
fn 新規に作る_内容は書いたとおりで_改行は_lf_のまま() {
    let dir = tempfile::tempdir().expect("作れる");
    let path = dir.path().join("conductor.toml");
    let bytes = "a = 1\nb = \"夜間\"\n".as_bytes();
    assert_eq!(create_file(&path, bytes, false), Ok(CreateOutcome::Created));
    assert_eq!(fs::read(&path).expect("読める"), bytes);
    assert!(!fs::read(&path).expect("読める").contains(&b'\r'));
}

#[test]
fn 既にあるファイルは壊さない() {
    let dir = tempfile::tempdir().expect("作れる");
    let path = dir.path().join("conductor.toml");
    fs::write(&path, "元の内容").expect("書ける");
    assert_eq!(
        create_file(&path, b"new", false),
        Err(CreateError::AlreadyExists)
    );
    assert_eq!(fs::read_to_string(&path).expect("読める"), "元の内容");
    assert_eq!(hidden_files(dir.path()), 0);
}

#[test]
fn force_で置き換える_内容が新しくなり_一時ファイルは残らない() {
    let dir = tempfile::tempdir().expect("作れる");
    let path = dir.path().join("conductor.toml");
    fs::write(&path, "元の内容").expect("書ける");
    assert_eq!(
        create_file(&path, b"new\n", true),
        Ok(CreateOutcome::Replaced)
    );
    assert_eq!(fs::read_to_string(&path).expect("読める"), "new\n");
    assert_eq!(hidden_files(dir.path()), 0);

    // 無いファイルへの force は、新規の作成。
    let other = dir.path().join("other.toml");
    assert_eq!(create_file(&other, b"x", true), Ok(CreateOutcome::Created));
    assert_eq!(fs::read_to_string(&other).expect("読める"), "x");
    assert_eq!(hidden_files(dir.path()), 0);
}

#[test]
fn 親のディレクトリが無いときは_作らず_parent_missing() {
    let dir = tempfile::tempdir().expect("作れる");
    let path = dir.path().join("missing").join("conductor.toml");
    for force in [false, true] {
        assert_eq!(
            create_file(&path, b"x", force),
            Err(CreateError::ParentMissing),
            "force={force}"
        );
    }
    assert!(!dir.path().join("missing").exists());
}

#[test]
fn 作る先がディレクトリなら_is_directory() {
    let dir = tempfile::tempdir().expect("作れる");
    let target = dir.path().join("sub");
    fs::create_dir(&target).expect("作れる");
    for force in [false, true] {
        assert_eq!(
            create_file(&target, b"x", force),
            Err(CreateError::IsDirectory),
            "force={force}"
        );
    }
    assert!(target.is_dir());
    assert_eq!(hidden_files(dir.path()), 0);
}

#[test]
fn 宙に浮いたシンボリックリンクは_already_exists_で_force_ならリンクそのものを置き換える() {
    let dir = tempfile::tempdir().expect("作れる");
    let link = dir.path().join("conductor.toml");
    let target = dir.path().join("gone.toml");
    #[cfg(unix)]
    let made = std::os::unix::fs::symlink(&target, &link).is_ok();
    #[cfg(not(unix))]
    let made = false;
    if !made {
        // リンクを作れない環境では、確かめずに通す。
        return;
    }
    assert_eq!(
        create_file(&link, b"x", false),
        Err(CreateError::AlreadyExists)
    );
    assert!(!target.exists(), "リンク先を作らない");
    assert!(fs::symlink_metadata(&link).expect("ある").is_symlink());

    assert_eq!(create_file(&link, b"y", true), Ok(CreateOutcome::Replaced));
    let meta = fs::symlink_metadata(&link).expect("ある");
    assert!(!meta.is_symlink(), "リンクそのものが置き換わる");
    assert_eq!(fs::read_to_string(&link).expect("読める"), "y");
    assert!(!target.exists(), "リンク先は変わらない");
}

#[test]
fn 書く権限がないディレクトリでは_permission_denied() {
    let dir = tempfile::tempdir().expect("作れる");
    let locked = dir.path().join("locked");
    fs::create_dir(&locked).expect("作れる");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o555)).expect("変えられる");
    }
    // 権限が効くかを、実行時に確かめる(root などでは、効かない)。
    let probe = locked.join("probe");
    let effective = fs::write(&probe, b"x").is_err();
    let outcome = (
        create_file(&locked.join("a.toml"), b"x", false),
        create_file(&locked.join("b.toml"), b"x", true),
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // 後始末のために、書けるように戻す。
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).expect("戻せる");
    }
    if !effective {
        return;
    }
    assert_eq!(outcome.0, Err(CreateError::PermissionDenied));
    assert_eq!(outcome.1, Err(CreateError::PermissionDenied));
    assert_eq!(hidden_files(&locked), 0);
}

#[test]
fn 同時に作る_2_つのうち_ちょうど_1_つだけが_created_になる() {
    for _ in 0..20 {
        let dir = tempfile::tempdir().expect("作れる");
        let path = dir.path().join("conductor.toml");
        let results: Vec<Result<CreateOutcome, CreateError>> = std::thread::scope(|scope| {
            let a = scope.spawn(|| create_file(&path, b"from-a\n", false));
            let b = scope.spawn(|| create_file(&path, b"from-b\n", false));
            vec![a.join().expect("終わる"), b.join().expect("終わる")]
        });
        let created = results
            .iter()
            .filter(|r| **r == Ok(CreateOutcome::Created))
            .count();
        let exists = results
            .iter()
            .filter(|r| **r == Err(CreateError::AlreadyExists))
            .count();
        assert_eq!(created, 1, "{results:?}");
        assert_eq!(exists, 1, "{results:?}");
    }
}
