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

//! ログのファイルの名前と作り方(RuntimeLog。v0.10.0 の BR3.1〜BR3.4、NFR6.1、NFR6.3)の統合テスト
//! (一時ディレクトリの実ファイル)。

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use conductor_core::Timestamp;
use conductor_runtime::{LogTarget, open_log_file, safe_log_stem};

/// 2026-11-25T09:12:48Z。
const STARTED: u64 = 1_795_597_968;
const TIME: &str = "20261125T091248Z";

fn target(dir: &Path, job_id: &str, attempt: u32) -> LogTarget {
    LogTarget {
        dir: dir.to_path_buf(),
        job_id: job_id.to_string(),
        started_at: Timestamp::from_unix_seconds(STARTED).expect("範囲の中"),
        attempt,
    }
}

fn names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .expect("読める")
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn br3_2_安全な名前の対の表() {
    let cases: &[(&str, &str)] = &[
        ("a", "a"),
        ("build", "build"),
        ("a/b", "a_b"),
        ("a\\b", "a_b"),
        ("../x", "_._x"),
        (".env", "_env"),
        ("..", "__"),
        ("a.", "a_"),
        ("a ", "a_"),
        ("a.b", "a.b"),
        ("CON", "_CON"),
        ("con", "_con"),
        ("com1", "_com1"),
        ("LPT9", "_LPT9"),
        ("NUL.x", "_NUL.x"),
        ("nul.tar.gz", "_nul.tar.gz"),
        ("CONX", "CONX"),
        ("COM0", "COM0"),
        ("x.CON", "x.CON"),
        ("日本語", "日本語"),
        ("a\tb", "a_b"),
        ("a\u{0}b\u{1f}c\u{7f}d\u{9f}e", "a_b_c_d_e"),
        ("a\u{a0}b", "a\u{a0}b"),
        ("a:b*c?d\"e<f>g|h", "a_b_c_d_e_f_g_h"),
    ];
    for (input, expected) in cases {
        assert_eq!(safe_log_stem(input), *expected, "{input:?}");
    }
}

#[test]
fn br3_2_末尾の点は_置き換えた後に見る() {
    // 先頭の `.` を置き換えた後に、予約名と末尾を見る。
    assert_eq!(safe_log_stem("."), "_");
    assert_eq!(safe_log_stem("CON."), "_CON_");
    assert_eq!(safe_log_stem("x "), "x_");
}

#[test]
fn br3_2_長い_id_は_100_バイトまでで_文字の途中で切らない() {
    let ascii = "a".repeat(200);
    assert_eq!(safe_log_stem(&ascii), "a".repeat(100));
    // 3 バイトの文字が 100 バイトの境界をまたぐ(99 バイトまで)。
    let wide = "あ".repeat(70);
    let stem = safe_log_stem(&wide);
    assert_eq!(stem, "あ".repeat(33));
    assert!(stem.len() <= 100);
    // 切った後の末尾の `.` も置き換える。
    let dotted = format!("{}.{}", "b".repeat(99), "c".repeat(20));
    assert_eq!(safe_log_stem(&dotted), format!("{}_", "b".repeat(99)));
    // 予約名の `_` を足しても 100 バイトを超えない。
    let reserved = format!("CON.{}", "d".repeat(200));
    assert_eq!(safe_log_stem(&reserved).len(), 100);
    assert!(safe_log_stem(&reserved).starts_with("_CON."));
}

#[test]
fn nfr6_3_置き換えた名前にディレクトリの区切りが残らない() {
    for input in [
        "../../etc/passwd",
        "a/../../b",
        "..\\..\\x",
        "/abs",
        "C:\\x",
    ] {
        let stem = safe_log_stem(input);
        assert!(!stem.contains('/') && !stem.contains('\\'), "{input:?}");
        assert!(!stem.starts_with('.'), "{input:?}");
    }
}

#[test]
fn br3_1_ディレクトリを親を含めて作り_名前の形で作る() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let logs = dir.path().join("a").join("b").join("logs");
    let (mut file, path) = open_log_file(&target(&logs, "build", 1)).expect("作れる");
    assert_eq!(path, logs.join(format!("build.{TIME}.1.log")));
    file.write_all(b"x").expect("書ける");
    drop(file);
    assert_eq!(fs::read(&path).expect("読める"), b"x");
    // 試行の番号は名前に入る。
    let (_, second) = open_log_file(&target(&logs, "build", 2)).expect("作れる");
    assert_eq!(second, logs.join(format!("build.{TIME}.2.log")));
}

#[test]
fn br3_1_ファイルは_log_dir_の直下にできる() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (_, path) = open_log_file(&target(dir.path(), "../x/y", 1)).expect("作れる");
    assert_eq!(path.parent(), Some(dir.path()));
    assert_eq!(names(dir.path()), [format!("_._x_y.{TIME}.1.log")]);
}

#[test]
fn br3_3_同じ名前があれば番号を足し_元のファイルは変わらない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let first: PathBuf = dir.path().join(format!("j.{TIME}.1.log"));
    fs::write(&first, b"original").expect("書ける");
    let (mut file, second) = open_log_file(&target(dir.path(), "j", 1)).expect("作れる");
    assert_eq!(second, dir.path().join(format!("j.{TIME}.1-2.log")));
    file.write_all(b"new").expect("書ける");
    drop(file);
    // 2 つあるときは -3。
    let (_, third) = open_log_file(&target(dir.path(), "j", 1)).expect("作れる");
    assert_eq!(third, dir.path().join(format!("j.{TIME}.1-3.log")));
    assert_eq!(
        fs::read(&first).expect("読める"),
        b"original",
        "上書きしない"
    );
    assert_eq!(fs::read(&second).expect("読める"), b"new", "追記しない");
    assert_eq!(fs::read(&third).expect("読める"), b"");
}

#[test]
fn br5_1_ディレクトリを作れなければ_番号なしの名前を返す() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    // ファイルを置いた場所を、ログのディレクトリにする。
    let blocker = dir.path().join("not-a-dir");
    fs::write(&blocker, b"keep").expect("書ける");
    let err = open_log_file(&target(&blocker, "j", 1)).expect_err("作れない");
    assert_eq!(err, blocker.join(format!("j.{TIME}.1.log")));
    assert_eq!(fs::read(&blocker).expect("読める"), b"keep");
    // その下のディレクトリも作れない。
    let deeper = blocker.join("logs");
    assert!(open_log_file(&target(&deeper, "j", 1)).is_err());
}

#[test]
fn br5_1_同じ名前のディレクトリがあれば番号を足すか作れない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    fs::create_dir(dir.path().join(format!("j.{TIME}.1.log"))).expect("作れる");
    // OS によって「すでにある」か別の誤りになる。どちらでも、ディレクトリには触れない。
    match open_log_file(&target(dir.path(), "j", 1)) {
        Ok((_, path)) => assert_eq!(path, dir.path().join(format!("j.{TIME}.1-2.log"))),
        Err(path) => assert_eq!(path, dir.path().join(format!("j.{TIME}.1.log"))),
    }
    assert!(dir.path().join(format!("j.{TIME}.1.log")).is_dir());
}

#[cfg(unix)]
#[test]
fn br3_4_unix_系のファイルの権限は_0600() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (_, path) = open_log_file(&target(dir.path(), "j", 1)).expect("作れる");
    let mode = fs::metadata(&path).expect("読める").permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
    // 番号つきの名前も同じ。
    let (_, numbered) = open_log_file(&target(dir.path(), "j", 1)).expect("作れる");
    let mode = fs::metadata(&numbered)
        .expect("読める")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600);
}

#[test]
fn 日本語の_id_と_試行の番号の大きな値でも作れる() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let (_, path) = open_log_file(&target(dir.path(), "夜間の処理", 101)).expect("作れる");
    assert_eq!(names(dir.path()), [format!("夜間の処理.{TIME}.101.log")]);
    assert!(path.exists());
    let long = "長".repeat(200);
    let (_, path) = open_log_file(&target(dir.path(), &long, 1)).expect("長い ID でも作れる");
    assert!(path.file_name().expect("名前").len() <= 255);
}
