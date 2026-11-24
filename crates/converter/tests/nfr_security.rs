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

//! 横断のテスト: NFR4.5(副作用なし)、NFR4.6(表示)、NFR4.7(パニックしない)、NFR1.1(静的な初期化なし)。
//! ランダムな入力とランダムな構造の往復の 2 件は、変換した結果を読んだ木を比べるので、単体テストの
//! `src/tests/nfr_security.rs` にある。

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use conductor_converter::{
    ConversionError, ConversionErrorKind as K, ConversionRequest, TargetFormat, convert,
    convert_text,
};

const SECRET: &str = "SECRET-VALUE-9f3a";

fn src_files() -> Vec<(PathBuf, String)> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("src を読める") {
        let path = entry.expect("entry").path();
        if path.extension().is_some_and(|e| e == "rs") {
            let text = std::fs::read_to_string(&path).expect("読める");
            out.push((path, text));
        }
    }
    assert!(out.len() >= 10, "ソースの数: {}", out.len());
    out
}

// ---- NFR4.5 ----

#[test]
fn ソースはプロセスとネットワークと環境変数に触れない() {
    // 検索の語は、分けて組み立てる(このファイル自身は、検索の対象外)
    let banned = [
        ["std::", "process"].concat(),
        ["std::", "net"].concat(),
        ["std::", "env"].concat(),
        ["use std::", "os"].concat(),
        ["Command", "::new"].concat(),
    ];
    // 書き込みの API は、本体(`#[cfg(test)]` より前)にない(テストのモジュールは、一時ディレクトリに作ってよい)
    let write_apis = [
        ["File::", "create"].concat(),
        ["fs::", "write"].concat(),
        ["OpenOptions"].concat(),
        ["create_", "dir"].concat(),
        ["remove_", "file"].concat(),
        ["remove_", "dir"].concat(),
        ["fs::", "rename"].concat(),
        ["fs::", "copy"].concat(),
    ];
    for (path, text) in src_files() {
        for word in &banned {
            assert!(
                !text.contains(word.as_str()),
                "{} に `{word}` がある",
                path.display()
            );
        }
        let production = text.split("#[cfg(test)]").next().unwrap_or("");
        for word in &write_apis {
            assert!(
                !production.contains(word.as_str()),
                "{} の本体に `{word}` がある",
                path.display()
            );
        }
    }
}

#[test]
fn ソースは静的な初期化を持たない() {
    let banned = [
        "lazy_static",
        "once_cell",
        "LazyLock",
        "LazyCell",
        "OnceLock",
        "OnceCell",
        "thread_local!",
        "static mut",
        "#[ctor",
    ];
    for (path, text) in src_files() {
        for word in banned {
            assert!(
                !text.contains(word),
                "{} に `{word}` がある",
                path.display()
            );
        }
        for (n, line) in text.lines().enumerate() {
            let t = line.trim_start();
            let is_static_item = t.starts_with("static ")
                || t.starts_with("pub static ")
                || t.starts_with("pub(crate) static ");
            assert!(
                !is_static_item,
                "{}:{}: 静的な値がある",
                path.display(),
                n + 1
            );
        }
    }
}

#[test]
fn ソースは入力の経路で_unwrap_や_panic_を使わない() {
    // テストのモジュール(`#[cfg(test)]` 以降)を除いて、`unwrap()` `panic!` `expect(` を使わない
    for (path, text) in src_files() {
        let production = text.split("#[cfg(test)]").next().unwrap_or("");
        for word in [
            ".unwrap()",
            "panic!",
            ".expect(",
            "unreachable!",
            "todo!",
            "unimplemented!",
        ] {
            assert!(
                !production.contains(word),
                "{} の本体に `{word}` がある",
                path.display()
            );
        }
    }
}

fn listing(dir: &Path) -> BTreeSet<String> {
    std::fs::read_dir(dir)
        .expect("読める")
        .map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
        .collect()
}

#[test]
fn 変換はファイルを作らず_カレントディレクトリの変更に左右されない() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("in.yaml");
    std::fs::write(&input, "a: 1\nb: [x, y]\n").unwrap();
    let broken = temp.path().join("broken.yaml");
    std::fs::write(&broken, "a: [1\n").unwrap();

    let temp_before = listing(temp.path());
    let cwd = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cwd_before = listing(cwd);
    let tests_before = listing(&cwd.join("tests"));
    let system_temp = std::env::temp_dir();
    let system_before = listing(&system_temp);

    for target in [TargetFormat::Toml, TargetFormat::Yaml] {
        for path in [&input, &broken, &temp.path().join("none.yaml")] {
            let _ = convert(&ConversionRequest {
                input_path: path.clone(),
                target_format: target,
            });
        }
    }

    assert_eq!(listing(temp.path()), temp_before);
    assert_eq!(listing(cwd), cwd_before);
    assert_eq!(listing(&cwd.join("tests")), tests_before);
    // 他のテストが一時ファイルを作るため、システムの一時ディレクトリは、この変換が作った名前がないことだけ見る
    let created: Vec<_> = listing(&system_temp)
        .difference(&system_before)
        .cloned()
        .collect();
    assert!(
        created.iter().all(|name| name.starts_with(".tmp")),
        "変換が作った疑いのあるファイル: {created:?}"
    );
}

/// 子プロセスとして動く(環境変数を変えた実行の結果を、親が比べる)。
#[test]
#[ignore = "環境変数に左右されない から、子プロセスとして動かす"]
fn 環境を調べる子プロセス() {
    let yaml = "a: &x [1, 2]\nb: *x\nc: yes\nd: 1.0\n";
    let toml = "a = [1, 2]\nb = \"yes\"\nc = 1.0\n";
    let r1 = convert_text(yaml, TargetFormat::Toml, "t").map(|o| o.text);
    let r2 = convert_text(toml, TargetFormat::Yaml, "t").map(|o| o.text);
    println!("PROBE-BEGIN\n{r1:?}\n{r2:?}\nPROBE-END");
}

#[test]
fn 環境変数に左右されない() {
    let exe = std::env::current_exe().expect("実行ファイル");
    let run = |envs: &[(&str, &str)]| -> String {
        let mut cmd = std::process::Command::new(&exe);
        cmd.args([
            "--ignored",
            "--exact",
            "環境を調べる子プロセス",
            "--nocapture",
            "--test-threads=1",
        ]);
        for (k, v) in envs {
            cmd.env(k, v);
        }
        let out = cmd.output().expect("子プロセスを動かせる");
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        let begin = stdout.find("PROBE-BEGIN").expect("結果がある");
        let end = stdout.find("PROBE-END").expect("結果がある");
        stdout[begin..end].to_string()
    };
    let base = run(&[]);
    assert!(base.contains("a = [1, 2]"), "{base}");
    let varied = run(&[
        ("LANG", "ja_JP.UTF-8"),
        ("LC_ALL", "C"),
        ("TZ", "Asia/Tokyo"),
        ("HOME", "/nonexistent"),
        ("NO_COLOR", "1"),
        ("RUST_LOG", "trace"),
        ("CONDUCTOR_DEBUG", "1"),
    ]);
    assert_eq!(base, varied);
}

#[test]
fn タグは値を組み立てない() {
    for text in [
        "a: !!binary aGVsbG8=\n",
        "a: !mytag value\n",
        "a: !!python/object:os.system x\n",
        "a: !<tag:example.com,2000:t> x\n",
    ] {
        let e = convert_text(text, TargetFormat::Toml, "t").unwrap_err();
        assert_eq!(e.kind, K::Unrepresentable, "{text:?}");
    }
    let out = convert_text("a: !!str 1\n", TargetFormat::Toml, "t").unwrap();
    assert_eq!(out.text, "a = \"1\"\n");
}

// ---- NFR4.6 ----

// 誤りの種類の一覧(テストの写し。BR1.7)。
//
// 一覧の定数、種類の位置を返す網羅の `match`(`_` を使わない)、一覧の i 番目の位置が i であることの
// コンパイル時の確かめを並べて置く。種類を足すと `match` がコンパイルで止まり、一覧の並びの誤りや
// 重複も、コンパイルで止まる。

/// 変換の失敗の種類のすべて(テストの写し。種類を足したら、下の `match` がコンパイルで止まるので、
/// ここにも足す)。
const CONVERSION_ERROR_KINDS: [K; 6] = [
    K::FileMissing,
    K::Unreadable,
    K::TooLarge,
    K::Syntax,
    K::Unrepresentable,
    K::RootNotMap,
];

/// 変換の失敗の種類の、一覧の中の位置(網羅の `match`。`_` を使わない)。
const fn conversion_position(kind: K) -> usize {
    match kind {
        K::FileMissing => 0,
        K::Unreadable => 1,
        K::TooLarge => 2,
        K::Syntax => 3,
        K::Unrepresentable => 4,
        K::RootNotMap => 5,
    }
}

// 一覧の i 番目の位置が i であること(並びの誤りと重複を、コンパイルで止める)。
const _: () = {
    let mut i = 0;
    while i < CONVERSION_ERROR_KINDS.len() {
        assert!(conversion_position(CONVERSION_ERROR_KINDS[i]) == i);
        i += 1;
    }
};

fn all_kind_errors() -> Vec<ConversionError> {
    let dir = tempfile::tempdir().unwrap();
    let write = |name: &str, bytes: &[u8]| -> PathBuf {
        let p = dir.path().join(name);
        std::fs::write(&p, bytes).unwrap();
        p
    };
    let req = |path: PathBuf, target| ConversionRequest {
        input_path: path,
        target_format: target,
    };

    let mut big = format!("k: {SECRET}\n").into_bytes();
    big.extend(std::iter::repeat_n(b'#', 1024 * 1024));
    let mut bad_utf8 = format!("k: {SECRET} ").into_bytes();
    bad_utf8.extend([0xff, 0xfe]);

    vec![
        convert(&req(dir.path().join("missing.yaml"), TargetFormat::Toml)).unwrap_err(),
        convert(&req(dir.path().to_path_buf(), TargetFormat::Toml)).unwrap_err(),
        convert(&req(write("big.yaml", &big), TargetFormat::Toml)).unwrap_err(),
        convert(&req(write("bad.yaml", &bad_utf8), TargetFormat::Toml)).unwrap_err(),
        convert_text(&format!("k: [{SECRET}\n"), TargetFormat::Toml, "x.yaml").unwrap_err(),
        convert_text(
            &format!("k: {SECRET}\nk: 2\n"),
            TargetFormat::Toml,
            "x.yaml",
        )
        .unwrap_err(),
        convert_text(
            &format!("k: !!binary {SECRET}\n"),
            TargetFormat::Toml,
            "x.yaml",
        )
        .unwrap_err(),
        convert_text(
            &format!("k: ~\nz: {SECRET}\n"),
            TargetFormat::Toml,
            "x.yaml",
        )
        .unwrap_err(),
        convert_text(&format!("- {SECRET}\n"), TargetFormat::Toml, "x.yaml").unwrap_err(),
        convert_text(
            &format!("k:\n  <<: {SECRET}\n"),
            TargetFormat::Toml,
            "x.yaml",
        )
        .unwrap_err(),
        convert_text(&format!("k = [{SECRET}\n"), TargetFormat::Yaml, "x.toml").unwrap_err(),
        convert_text("d = 1979-05-27\n", TargetFormat::Yaml, "x.toml").unwrap_err(),
        convert_text("a: 99999999999999999999\n", TargetFormat::Toml, "x.yaml").unwrap_err(),
    ]
}

#[test]
fn 表示は_どの種類でも入力の値を含まない() {
    let errors = all_kind_errors();
    let kinds: Vec<K> = errors.iter().map(|e| e.kind).collect();
    for kind in CONVERSION_ERROR_KINDS {
        assert!(kinds.contains(&kind), "{kind:?} のテストがない");
    }
    for e in &errors {
        let shown = format!("{e}");
        assert!(!shown.contains(SECRET), "{shown}");
        assert!(!e.reason.contains(SECRET), "{}", e.reason);
        assert!(!format!("{:?}", e.key_path).contains(SECRET));
        // 固定の文と場所だけ: 理由は、種類ごとの固定の文
        assert!(!e.reason.is_empty());
        assert!(shown.contains(e.reason), "{shown}");
    }
}

#[test]
fn ライブラリの誤りの文は現れない() {
    // ライブラリのエラー文の語(YAML パーサー、TOML パーサー)が、表示に現れない
    let toml_err = convert_text("a = = 1\n", TargetFormat::Yaml, "t")
        .unwrap_err()
        .to_string();
    let yaml_err = convert_text("a: [1\nb: 2\n", TargetFormat::Toml, "t")
        .unwrap_err()
        .to_string();
    for shown in [toml_err, yaml_err] {
        for word in [
            "expected",
            "found",
            "invalid",
            "unexpected",
            "while parsing",
            "TOML parse error",
        ] {
            assert!(!shown.contains(word), "{shown}");
        }
    }
}

#[test]
fn キーの場所のキー名は_エスケープし切り詰める() {
    let long = "k".repeat(100);
    let e = convert_text(&format!("\"{long}\": ~\n"), TargetFormat::Toml, "t").unwrap_err();
    let path = e.key_path.unwrap();
    assert_eq!(path.chars().count(), 65, "{path}");
    assert!(path.ends_with('…'));

    let e = convert_text("\"a\\e[31m\\u202e\\n\": ~\n", TargetFormat::Toml, "t").unwrap_err();
    let path = e.key_path.unwrap();
    assert!(
        !path.contains('\u{1b}') && !path.contains('\u{202e}') && !path.contains('\n'),
        "{path:?}"
    );
    assert!(
        path.contains("\\u{1b}") && path.contains("\\u{202e}") && path.contains("\\n"),
        "{path}"
    );

    // 表示全体も、制御文字を含まない
    let shown = format!(
        "{}",
        convert_text("\"x\\ry\": ~\n", TargetFormat::Toml, "t").unwrap_err()
    );
    assert!(!shown.contains('\r'));
}

// ---- NFR4.7 ----

#[test]
fn 範囲外と空と壊れた入力で_パニックしない() {
    let cases: Vec<(&str, TargetFormat, K)> = vec![
        (
            "a: 9223372036854775808\n",
            TargetFormat::Toml,
            K::Unrepresentable,
        ),
        (
            "a: -9223372036854775809\n",
            TargetFormat::Toml,
            K::Unrepresentable,
        ),
        (
            "a: 0xFFFFFFFFFFFFFFFFFFFFFFFF\n",
            TargetFormat::Toml,
            K::Unrepresentable,
        ),
        ("a = 9223372036854775808\n", TargetFormat::Yaml, K::Syntax),
        (
            "a = 0xFFFFFFFFFFFFFFFFFFFF\n",
            TargetFormat::Yaml,
            K::Syntax,
        ),
        ("", TargetFormat::Toml, K::RootNotMap),
        ("   \n\n", TargetFormat::Toml, K::RootNotMap),
        ("\0", TargetFormat::Toml, K::Syntax),
        ("a: \"\\xZZ\"\n", TargetFormat::Toml, K::Syntax),
        ("a: \"\\uD800\"\n", TargetFormat::Toml, K::Syntax),
        ("a = \"\\uD800\"\n", TargetFormat::Yaml, K::Syntax),
    ];
    for (text, target, kind) in cases {
        let e = convert_text(text, target, "t").unwrap_err();
        assert_eq!(e.kind, kind, "{text:?}");
    }
    // TOML の空は、空のマップ
    assert_eq!(
        convert_text("", TargetFormat::Yaml, "t").unwrap().text,
        "{}\n"
    );
}
