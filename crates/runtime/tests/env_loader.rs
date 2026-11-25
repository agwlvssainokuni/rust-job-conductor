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

use conductor_core::{EnvFileError, EnvFileErrorKind as K, EnvFileRef};
use conductor_runtime::load_env_files;

const SECRET: &str = "s3cr3t-VALUE-do-not-print";
const MAX: usize = 1_048_576;

/// 必ず要る(`required = true`)項目の並び。
fn refs(items: &[&str]) -> Vec<EnvFileRef> {
    items.iter().map(|s| env_ref(s, true)).collect()
}

fn env_ref(path: &str, required: bool) -> EnvFileRef {
    EnvFileRef {
        path: path.to_string(),
        required,
    }
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

#[test]
fn 無いファイルは必ず要るなら_file_missing_要らないなら誤りにせず引き当てにも載せない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let def = dir.path().join("c.toml");
    write(dir.path(), "ok.env", "A=1\n");

    // 必ず要る: 今までどおり FileMissing(v0.10.0 の BR2.5)
    let e = load_env_files(&def, &[env_ref("ok.env", true), env_ref("none.env", true)])
        .expect_err("誤り");
    assert_eq!(kinds(&e), [(None, K::FileMissing)]);

    // 要らない: 誤りにせず、その記述は引き当てに無い(v0.10.0 の BR2.3)
    let loaded = load_env_files(&def, &[env_ref("ok.env", true), env_ref("none.env", false)])
        .expect("読める");
    assert_eq!(loaded.files().len(), 1);
    assert!(loaded.get("ok.env").is_some());
    assert!(loaded.get("none.env").is_none());

    // 要らないファイルだけで、どれも無い: 空の結果
    let loaded = load_env_files(&def, &[env_ref("none.env", false)]).expect("読める");
    assert!(loaded.files().is_empty());
}

#[test]
fn 親のディレクトリが無いときも無いとして扱う() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let def = dir.path().join("c.toml");
    let loaded = load_env_files(&def, &[env_ref("nodir/x.env", false)]).expect("読める");
    assert!(loaded.get("nodir/x.env").is_none());
    let e = load_env_files(&def, &[env_ref("nodir/x.env", true)]).expect_err("誤り");
    assert_eq!(kinds(&e), [(None, K::FileMissing)]);
}

#[test]
fn 必ず要らないファイルでもあって読めないものは今までどおりの誤り() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let def = dir.path().join("c.toml");
    std::fs::create_dir(dir.path().join("d.env")).expect("ディレクトリ");
    write(dir.path(), "big.env", &"x".repeat(MAX + 1));
    let mut bytes = format!("A={SECRET}").into_bytes();
    bytes.push(0xff);
    std::fs::write(dir.path().join("bin.env"), bytes).expect("書き込み");
    write(dir.path(), "bad.env", &format!("NOEQUALS {SECRET}\n"));

    let e = load_env_files(
        &def,
        &[
            env_ref("d.env", false),
            env_ref("big.env", false),
            env_ref("bin.env", false),
            env_ref("bad.env", false),
        ],
    )
    .expect_err("誤り");
    assert_eq!(
        kinds(&e),
        [
            (None, K::Unreadable),
            (None, K::TooLarge),
            (None, K::NotUtf8),
            (Some(1), K::MissingEquals),
        ]
    );
    for err in &e {
        assert!(!err.to_string().contains(SECRET), "{err}");
    }
}

#[test]
fn 同じパスに解決される記述はどれか_1_つが必ず要るなら必ず要る() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let def = dir.path().join("c.toml");

    // 片方だけ必ず要る(前でも後ろでも): 無ければ FileMissing が 1 つだけ
    for refs in [
        [env_ref("x.env", false), env_ref("./x.env", true)],
        [env_ref("x.env", true), env_ref("./x.env", false)],
    ] {
        let e = load_env_files(&def, &refs).expect_err("誤り");
        assert_eq!(kinds(&e), [(None, K::FileMissing)], "{refs:?}");
    }

    // どちらも要らない: 誤りにしない
    let loaded = load_env_files(&def, &[env_ref("x.env", false), env_ref("./x.env", false)])
        .expect("読める");
    assert!(loaded.files().is_empty());

    // あれば、どちらの書き方からも同じ内容を引ける
    write(dir.path(), "x.env", "X=1\n");
    let loaded =
        load_env_files(&def, &[env_ref("x.env", false), env_ref("./x.env", true)]).expect("読める");
    assert_eq!(loaded.files().len(), 1);
    assert_eq!(loaded.get("x.env"), loaded.get("./x.env"));
}

#[test]
fn 誤りの並びは記述の出現順で読み飛ばしたファイルは入らない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let def = dir.path().join("c.toml");
    let mut bytes = b"A=1".to_vec();
    bytes.push(0xff);
    std::fs::write(dir.path().join("bin.env"), bytes).expect("書き込み");
    write(dir.path(), "ok.env", "B=1\n");

    let e = load_env_files(
        &def,
        &[
            env_ref("opt.env", false),
            env_ref("bin.env", false),
            env_ref("ok.env", true),
            env_ref("must.env", true),
            env_ref("./opt.env", false),
        ],
    )
    .expect_err("誤り");
    assert_eq!(kinds(&e), [(None, K::NotUtf8), (None, K::FileMissing)]);
    assert!(e[0].file.ends_with("bin.env") && e[1].file.ends_with("must.env"));
    assert!(
        e.iter().all(|err| !err.file.contains("opt.env")),
        "読み飛ばしたファイルは表示しない: {e:?}"
    );
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
