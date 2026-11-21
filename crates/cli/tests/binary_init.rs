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

//! `init` の、実際のバイナリのテスト(U4。FR4.1〜FR4.5)。ロケールは英語に固定される。
//! 空の一時ディレクトリで、`conductor` を実行する(カレントディレクトリの既定の `conductor.toml`)。

mod bin_support;

use std::path::Path;
use std::process::Output;

use bin_support::*;

/// `dir` をカレントディレクトリにして、`conductor <args>` を実行する。
fn run_in(dir: &Path, args: &[&str]) -> Output {
    let mut command = conductor(args);
    command.current_dir(dir);
    run_limited(command, || {})
}

#[test]
fn 空のディレクトリで_init_すると_validate_に通り_dry_run_の計画が出る() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let out = run_in(dir.path(), &["init"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert_eq!(text(&out.stderr), "created conductor.toml\n");
    assert_eq!(text(&out.stdout), "");
    let written = std::fs::read(dir.path().join("conductor.toml")).expect("できている");
    assert!(!written.contains(&b'\r'), "改行は LF");
    assert!(
        String::from_utf8(written)
            .expect("UTF-8")
            .contains("name = \"my-pipeline\"\n")
    );

    let out = run_in(dir.path(), &["validate"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));

    // U3 との組み合わせ: 計画が出て、ステートは作られない。
    let out = run_in(dir.path(), &["pipeline", "my-pipeline", "--dry-run"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let plan = text(&out.stdout);
    assert!(
        plan.starts_with("plan (--dry-run): pipeline my-pipeline\n"),
        "{plan}"
    );
    assert!(
        plan.contains("[run] hello") && plan.contains("[run] flaky"),
        "{plan}"
    );
    assert!(!dir.path().join("conductor.toml.state").exists());
}

#[test]
fn もう一度_init_すると_69_でファイルは変わらない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let path = dir.path().join("conductor.toml");
    std::fs::write(&path, "# 利用者の定義\n").expect("書ける");
    let out = run_in(dir.path(), &["init"]);
    assert_eq!(out.status.code(), Some(69));
    assert_eq!(
        text(&out.stderr),
        "conductor: error: conductor.toml already exists (use --force to overwrite)\n"
    );
    assert_eq!(text(&out.stdout), "");
    assert_eq!(
        std::fs::read_to_string(&path).expect("読める"),
        "# 利用者の定義\n"
    );
}

#[test]
fn force_で上書きされ_validate_に通る() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let path = dir.path().join("conductor.toml");
    std::fs::write(&path, "# 利用者の定義\n").expect("書ける");
    let out = run_in(dir.path(), &["init", "--force", "nightly"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert_eq!(text(&out.stderr), "replaced conductor.toml\n");
    let written = std::fs::read_to_string(&path).expect("読める");
    assert!(written.contains("name = \"nightly\"\n"), "{written}");
    let out = run_in(dir.path(), &["validate", "nightly"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
}

#[test]
fn config_ハイフンの出力は_ファイルの内容と同じで_ファイルを作らない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let out = run_in(dir.path(), &["init", "--config", "-"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert_eq!(text(&out.stderr), "");
    assert!(!dir.path().join("conductor.toml").exists());

    let other = tempfile::tempdir().expect("一時ディレクトリ");
    let out_file = run_in(other.path(), &["init"]);
    assert_eq!(out_file.status.code(), Some(0));
    let content = std::fs::read(other.path().join("conductor.toml")).expect("できている");
    assert_eq!(out.stdout, content);
}

#[test]
fn 特殊な文字の名前で_init_したファイルも_validate_を通る() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    for (index, name) in ["a\"b", "a\\b", "夜間 バッチ", "# x"]
        .into_iter()
        .enumerate()
    {
        let file = format!("def{index}.toml");
        let out = run_in(dir.path(), &["init", "--config", &file, name]);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{name:?}: {}",
            text(&out.stderr)
        );
        let out = run_in(dir.path(), &["validate", "--config", &file, name]);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{name:?}: {}",
            text(&out.stderr)
        );
    }
    // 親のディレクトリが無いときは、作らず 65。
    let out = run_in(dir.path(), &["init", "--config", "missing/conductor.toml"]);
    assert_eq!(out.status.code(), Some(65));
    assert_eq!(
        text(&out.stderr),
        "conductor: error: cannot create missing/conductor.toml: the parent directory does not exist\n"
    );
    assert!(!dir.path().join("missing").exists());
}
