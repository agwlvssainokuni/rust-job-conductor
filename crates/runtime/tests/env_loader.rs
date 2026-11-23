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

//! `.env` の読み取り(U1。BR1.7、BR3.1〜BR3.4、BR6.4)。一時ディレクトリの実ファイルを使う。

use std::path::Path;

use conductor_core::{EnvFileError, EnvFileErrorKind as K};
use conductor_runtime::load_env_files;

const SECRET: &str = "s3cr3t-VALUE-do-not-print";
const MAX: usize = 1_048_576;

fn refs(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn kinds(errors: &[EnvFileError]) -> Vec<(Option<usize>, K)> {
    errors.iter().map(|e| (e.line, e.kind)).collect()
}

fn write(dir: &Path, name: &str, text: &str) {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("親のディレクトリ");
    }
    std::fs::write(path, text).expect("書き込み");
}

#[test]
fn 相対パスは定義ファイルのディレクトリが基準で絶対パスと_dotdot_も使える() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    write(dir.path(), "conf/a.env", "A=rel\n");
    write(dir.path(), "up.env", "U=up\n");
    write(dir.path(), "abs.env", "B=abs\n");
    let def = dir.path().join("conf/conductor.toml");
    let abs = dir.path().join("abs.env").display().to_string();

    // 実行ディレクトリは、定義ファイルのディレクトリではない(基準が定義ファイルであることの確認)
    let loaded = load_env_files(&def, &refs(&["a.env", "../up.env", &abs])).expect("読める");
    let value = |r: &str| loaded.get(r).expect("読んである").entries[0].value.clone();
    assert_eq!(value("a.env"), "rel");
    assert_eq!(value("../up.env"), "up");
    assert_eq!(value(&abs), "abs");
}

#[test]
fn 複数のファイルを書いた順に読む() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    write(dir.path(), "1.env", "A=1\n");
    write(dir.path(), "2.env", "A=2\n");
    write(dir.path(), "0.env", "A=0\n");
    let def = dir.path().join("c.toml");
    let loaded = load_env_files(&def, &refs(&["2.env", "0.env", "1.env"])).expect("読める");
    let order: Vec<_> = loaded
        .files()
        .iter()
        .map(|f| f.entries[0].value.as_str())
        .collect();
    assert_eq!(order, ["2", "0", "1"]);
}

#[test]
fn 同じパスの別の書き方は_1_回だけ読み同じ内容を共有する() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    // 1 回だけ読むことは、誤りが 1 度しか報告されないことで確かめる
    write(dir.path(), "bad.env", "NOEQUALS\n");
    write(dir.path(), "ok.env", "A=1\n");
    let def = dir.path().join("c.toml");
    let e = load_env_files(&def, &refs(&["bad.env", "./bad.env"])).expect_err("誤り");
    assert_eq!(
        kinds(&e),
        [(Some(1), K::MissingEquals)],
        "同じファイルの誤りは重複しない"
    );

    let loaded = load_env_files(&def, &refs(&["ok.env", "./ok.env"])).expect("読める");
    assert_eq!(loaded.files().len(), 1);
    assert_eq!(loaded.get("ok.env"), loaded.get("./ok.env"));
}

#[test]
fn 無い_ディレクトリ_読めないファイルは誤りで中身を読まない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    std::fs::create_dir(dir.path().join("d.env")).expect("ディレクトリ");
    let def = dir.path().join("c.toml");
    let e = load_env_files(&def, &refs(&["nothing.env", "d.env"])).expect_err("誤り");
    assert_eq!(kinds(&e), [(None, K::FileMissing), (None, K::Unreadable)]);
    assert!(e[0].to_string().contains("nothing.env"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        write(dir.path(), "locked.env", &format!("A={SECRET}\n"));
        let path = dir.path().join("locked.env");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).expect("権限");
        // root では読めてしまうため、読めなかったときだけ検査する
        if std::fs::read(&path).is_err() {
            let e = load_env_files(&def, &refs(&["locked.env"])).expect_err("誤り");
            assert_eq!(kinds(&e), [(None, K::Unreadable)]);
        }
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).expect("権限");
    }
}

#[test]
fn ちょうど_1_mib_は通り_1_mib_足す_1_は誤りで中身を読まずに判断する() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let def = dir.path().join("c.toml");
    // 1 MiB ちょうど: コメント行で埋める(項目の上限に触れない)
    let mut exact = String::from("A=1\n#");
    exact.push_str(&"x".repeat(MAX - exact.len()));
    assert_eq!(exact.len(), MAX);
    write(dir.path(), "exact.env", &exact);
    let loaded = load_env_files(&def, &refs(&["exact.env"])).expect("通る");
    assert_eq!(loaded.get("exact.env").unwrap().entries.len(), 1);

    // 1 MiB + 1: 中身が UTF-8 でなくても、大きさで判断する(読まない)
    write(dir.path(), "big.env", &"x".repeat(MAX + 1));
    std::fs::write(dir.path().join("bigbin.env"), vec![0xffu8; MAX + 1]).expect("書き込み");
    let e = load_env_files(&def, &refs(&["big.env", "bigbin.env"])).expect_err("誤り");
    assert_eq!(kinds(&e), [(None, K::TooLarge), (None, K::TooLarge)]);
}

#[test]
fn utf8_でないファイルは誤り() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let def = dir.path().join("c.toml");
    let mut bytes = format!("A={SECRET}").into_bytes();
    bytes.push(0xff);
    std::fs::write(dir.path().join("bin.env"), bytes).expect("書き込み");
    let e = load_env_files(&def, &refs(&["bin.env"])).expect_err("誤り");
    assert_eq!(kinds(&e), [(None, K::NotUtf8)]);
    assert!(!e[0].to_string().contains(SECRET));
}

#[test]
fn 複数のファイルの誤りを集めて行番号つきで返し値を含まない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let def = dir.path().join("c.toml");
    write(
        dir.path(),
        "a.env",
        &format!("OK=1\nBAD {SECRET}\nC=\"{SECRET}\\q\"\n"),
    );
    write(dir.path(), "b.env", &format!("1X={SECRET}\n"));
    write(dir.path(), "good.env", "G=1\n");
    let e = load_env_files(&def, &refs(&["a.env", "good.env", "b.env", "missing.env"]))
        .expect_err("誤り");
    assert_eq!(
        kinds(&e),
        [
            (Some(2), K::MissingEquals),
            (Some(3), K::InvalidEscape),
            (Some(1), K::InvalidName),
            (None, K::FileMissing),
        ]
    );
    for err in &e {
        assert!(!err.to_string().contains(SECRET), "{err}");
    }
    assert!(e[0].file.ends_with("a.env") && e[2].file.ends_with("b.env"));
}

/// 読む権限のない 1 MiB 超の `.env` は、大きさより先に開けないことで `unreadable`(3b。定義ファイルの
/// 読み込みと同じ順。v0.9.6 までは、パスのメタデータで大きさを先に見たため `tooLarge` だった)。
#[cfg(unix)]
#[test]
fn 権限のない_1_mib_超の_env_は_unreadable() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let def = dir.path().join("c.toml");
    let path = dir.path().join("big-locked.env");
    std::fs::write(&path, format!("A={SECRET}\n#{}", "x".repeat(MAX))).expect("書き込み");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).expect("権限");
    // root などで開けてしまうときは、確かめない。
    if std::fs::File::open(&path).is_ok() {
        return;
    }
    let e = load_env_files(&def, &refs(&["big-locked.env"])).expect_err("誤り");
    assert_eq!(kinds(&e), [(None, K::Unreadable)]);
    assert!(!e[0].to_string().contains(SECRET));
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).expect("権限");
}

/// 書き手のいない名前付きパイプの `.env` は、開かずに `unreadable`(待ち続けない。3b)。
#[cfg(unix)]
#[test]
fn 名前付きパイプの_env_は開かずに_unreadable() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let def = dir.path().join("c.toml");
    let path = dir.path().join("pipe.env");
    let status = std::process::Command::new("mkfifo")
        .arg(&path)
        .status()
        .expect("mkfifo を起動できる");
    assert!(status.success(), "mkfifo が失敗した");
    // 止まったときに落ちるよう、別のスレッドで呼び、5 秒で打ち切る。
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(load_env_files(&def, &refs(&["pipe.env"])));
    });
    let r = rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("5 秒以内に返る(開いて待たない)");
    let e = r.expect_err("誤り");
    assert_eq!(kinds(&e), [(None, K::Unreadable)]);
}
