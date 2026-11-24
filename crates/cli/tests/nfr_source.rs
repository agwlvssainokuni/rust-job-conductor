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

//! ソースの検索と、依存の検査(NFR4.3、NFR4.5、NFR3.1、NFR2.1)。
//!
//! この Unit の `src/` に、`unsafe`、`low_level`、`std::net`、ネットワークのライブラリがないこと。
//! `cfg(unix)`・`cfg(windows)` が `platform` の外にないこと。依存に、C のコンパイルを要するものがないこと。
//! 単体テスト(`src/tests/`)は、OS 分岐と `panic!(` の検査の外(`lib.rs` の最後の `#[cfg(test)]` の
//! 宣言からだけ読み、製品のファイルに `#[path` が無いことを確かめる)。

use std::path::{Path, PathBuf};
use std::process::Command;

fn src_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// 単体テスト(`src/tests/` の下)のファイルか。`lib.rs` の最後の `#[cfg(test)] mod tests;` から
/// 読むテストのコードで、本番の成果物には入らない。
fn is_unit_test(path: &Path) -> bool {
    path.starts_with(src_dir().join("tests"))
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("読める") {
        let path = entry.expect("エントリ").path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// コメント(`//` 以降)を除いた、コードの行。
fn code_lines(path: &Path) -> Vec<(usize, String)> {
    let text = std::fs::read_to_string(path).expect("読める");
    text.lines()
        .enumerate()
        .map(|(i, line)| {
            let code = match line.find("//") {
                Some(at) => &line[..at],
                None => line,
            };
            (i + 1, code.to_string())
        })
        .collect()
}

fn all_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    rust_files(&src_dir(), &mut files);
    assert!(files.len() >= 10, "ソースが見つかる: {files:?}");
    files
}

fn find(pattern: &dyn Fn(&str) -> bool, skip: &dyn Fn(&Path) -> bool) -> Vec<String> {
    let mut hits = Vec::new();
    for file in all_files() {
        if skip(&file) {
            continue;
        }
        for (line_no, line) in code_lines(&file) {
            if pattern(&line) {
                hits.push(format!("{}:{line_no}: {line}", file.display()));
            }
        }
    }
    hits
}

#[test]
fn クレートは_forbid_unsafe_code_を宣言している() {
    let lib = std::fs::read_to_string(src_dir().join("lib.rs")).expect("読める");
    assert!(lib.contains("#![forbid(unsafe_code)]"));
}

#[test]
fn src_に_unsafe_と_low_level_がない() {
    let hits = find(
        &|line| {
            let cleaned = line.replace("forbid(unsafe_code)", "");
            cleaned.contains("unsafe") || cleaned.contains("low_level")
        },
        &|_| false,
    );
    assert!(hits.is_empty(), "{hits:#?}");
}

#[test]
fn src_にネットワークのコードがない() {
    let hits = find(
        &|line| {
            [
                "std::net",
                "TcpStream",
                "TcpListener",
                "UdpSocket",
                "reqwest",
                "hyper",
                "ureq",
                "tokio",
                "curl",
                "openssl",
            ]
            .iter()
            .any(|word| line.contains(word))
        },
        &|_| false,
    );
    assert!(hits.is_empty(), "{hits:#?}");
}

#[test]
fn os_ごとの分岐は_platform_の中だけ() {
    let hits = find(
        &|line| {
            let compact: String = line.chars().filter(|c| !c.is_whitespace()).collect();
            compact.contains("cfg(unix)")
                || compact.contains("cfg(windows)")
                || compact.contains("cfg!(unix)")
                || compact.contains("cfg!(windows)")
                || compact.contains("target_os")
                || compact.contains("cfg(not(unix))")
                || compact.contains("cfg(not(windows))")
        },
        &|path| path.components().any(|c| c.as_os_str() == "platform") || is_unit_test(path),
    );
    assert!(hits.is_empty(), "{hits:#?}");
}

#[test]
fn テスト専用の経路を実行ファイルに持たない() {
    // main.rs は、パニックのフックと run_cli だけ(パニックを起こすコマンドがない)
    let main = std::fs::read_to_string(src_dir().join("main.rs")).expect("読める");
    assert!(!main.contains("panic!"), "main.rs にパニックの経路がある");
    let hits = find(&|line| line.contains("panic!("), &|path| {
        path.components().any(|c| c.as_os_str() == "bin") || is_unit_test(path)
    });
    assert!(hits.is_empty(), "{hits:#?}");
    // 検査の外にする src/tests/ は、テストのときだけ読む(lib.rs の最後の宣言)
    let lib = std::fs::read_to_string(src_dir().join("lib.rs"))
        .expect("読める")
        .replace("\r\n", "\n");
    assert!(
        lib.trim_end().ends_with("#[cfg(test)]\nmod tests;"),
        "src/tests/ は #[cfg(test)] の宣言からだけ読む"
    );
    // 製品のファイル(src/tests/ の外)から、#[path] で src/tests/ を読む道が無い
    let paths = find(&|line| line.contains("#[path"), &|path| is_unit_test(path));
    assert!(paths.is_empty(), "{paths:#?}");
}

#[test]
fn 依存に_c_のコンパイルを要するものとネットワークのライブラリがない() {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let output = Command::new(cargo)
        .args([
            "tree",
            "-p",
            "conductor-cli",
            "-e",
            "normal,build",
            "--prefix",
            "none",
            "--offline",
            "--locked",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output();
    let Ok(output) = output else {
        eprintln!("スキップ: cargo を起動できない");
        return;
    };
    if !output.status.success() {
        eprintln!(
            "スキップ: cargo tree が失敗した: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    let names: Vec<&str> = text
        .lines()
        .filter_map(|l| l.split_whitespace().next())
        .collect();
    assert!(names.contains(&"lexopt"), "{names:?}");
    for forbidden in [
        "cc",
        "cmake",
        "pkg-config",
        "bindgen",
        "openssl",
        "reqwest",
        "hyper",
        "ureq",
        "tokio",
    ] {
        assert!(!names.contains(&forbidden), "{forbidden} が依存にある");
    }
    let allowed = ["windows-sys", "linux-raw-sys"];
    for name in &names {
        if name.ends_with("-sys") {
            assert!(allowed.contains(name), "許可されていない -sys: {name}");
        }
    }
}
