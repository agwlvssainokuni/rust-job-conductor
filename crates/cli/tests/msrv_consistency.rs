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

//! MSRV(最小の Rust の版)の記載が、`Cargo.toml`・`README.md`・`ci.yml` で一致すること。
//!
//! リポジトリのファイルを読むだけで、書き込まない。

use std::path::{Path, PathBuf};

fn repo_file(relative: &str) -> String {
    let root: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    std::fs::read_to_string(root.join(relative)).expect("読める")
}

/// `Cargo.toml` の `rust-version = "x.y"` の値。
fn cargo_msrv() -> String {
    let text = repo_file("Cargo.toml");
    let line = text
        .lines()
        .find(|l| l.trim_start().starts_with("rust-version"))
        .expect("rust-version がある");
    line.split('"')
        .nth(1)
        .expect("引用符で囲まれている")
        .to_string()
}

#[test]
fn msrv_は_cargo_toml_と_readme_で一致する() {
    let msrv = cargo_msrv();
    let readme = repo_file("README.md");
    assert!(
        readme.contains(&format!("Rust({msrv} 以降)")),
        "README の MSRV が Cargo.toml({msrv})と違う"
    );
}

#[test]
fn msrv_は_cargo_toml_と_ci_のツールチェーン指定で一致する() {
    let msrv = cargo_msrv();
    let ci = repo_file(".github/workflows/ci.yml");
    let pinned: Vec<&str> = ci
        .lines()
        .filter_map(|l| l.split("dtolnay/rust-toolchain@").nth(1))
        .map(str::trim)
        .filter(|v| *v != "stable")
        .collect();
    assert!(!pinned.is_empty(), "固定した版のジョブがある");
    for version in pinned {
        assert_eq!(
            version,
            format!("{msrv}.0"),
            "ci.yml の固定した版が Cargo.toml({msrv})と違う"
        );
    }
}

#[test]
fn msrv_は_ci_のコメントとも一致する() {
    let msrv = cargo_msrv();
    let ci = repo_file(".github/workflows/ci.yml");
    assert!(
        ci.contains(&format!("rust-version = {msrv})")),
        "ci.yml のコメント(rust-version)が Cargo.toml({msrv})と違う"
    );
    assert!(
        ci.contains(&format!("最小の Rust {msrv} でも")),
        "ci.yml のコメント(最小の Rust)が Cargo.toml({msrv})と違う"
    );
}
