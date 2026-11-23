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

//! 受け入れ基準(AC7.1.1〜AC7.1.3)と、`convert`(ファイルを読む入口)。

mod common;

use std::path::{Path, PathBuf};

use common::{toml, yaml};
use conductor_converter::{
    ConversionErrorKind as K, ConversionRequest, TargetFormat, convert, convert_text,
};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn request(path: PathBuf, target: TargetFormat) -> ConversionRequest {
    ConversionRequest {
        input_path: path,
        target_format: target,
    }
}

#[test]
fn ac_7_1_1_toml_から_yaml() {
    let out = convert(&request(fixture("pipeline.toml"), TargetFormat::Yaml)).unwrap();
    assert_eq!(
        yaml(&out.text).unwrap(),
        toml(include_str!("fixtures/pipeline.toml")).unwrap()
    );
    assert!(out.text.contains("format_version: 1"), "{}", out.text);
    assert!(out.text.contains("id: extract"), "{}", out.text);
}

#[test]
fn ac_7_1_2_アンカーつきの_yaml_から_toml() {
    let out = convert(&request(fixture("pipeline.yaml"), TargetFormat::Toml)).unwrap();
    assert_eq!(
        toml(&out.text).unwrap(),
        yaml(include_str!("fixtures/pipeline.yaml")).unwrap()
    );
    assert!(!out.text.contains("<<"), "{}", out.text);
    // 展開された結果が、定義の形(表の配列)で出力される
    assert!(out.text.contains("[[job]]"), "{}", out.text);
}

#[test]
fn ac_7_1_3_壊れた入力は種類と行を返し_出力しない() {
    let e = convert(&request(fixture("broken.yaml"), TargetFormat::Toml)).unwrap_err();
    assert_eq!(e.kind, K::Syntax);
    assert!(e.line.is_some());
    assert!(e.file.ends_with("broken.yaml"));
    let shown = e.to_string();
    assert!(shown.contains("broken.yaml:"), "{shown}");
    assert!(shown.contains("YAML として読めません"), "{shown}");

    let e = convert(&request(fixture("broken.toml"), TargetFormat::Yaml)).unwrap_err();
    assert_eq!(e.kind, K::Syntax);
    assert_eq!(e.line, Some(7));
}

#[test]
fn 入力の形式は変換後の反対で_拡張子は見ない() {
    // 拡張子が `.toml` でも、`--to toml` なら、YAML として読む(BR1.1)
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("looks-like.toml");
    std::fs::write(&path, "a: 1\nb: [x, y]\n").unwrap();
    let out = convert(&request(path.clone(), TargetFormat::Toml)).unwrap();
    assert!(out.text.contains("a = 1"), "{}", out.text);
    // 同じファイルを `--to yaml` にすると、TOML として読もうとして、文法の誤り
    let e = convert(&request(path, TargetFormat::Yaml)).unwrap_err();
    assert_eq!(e.kind, K::Syntax);
}

#[test]
fn 変換はファイルの失敗を決まった順で報告する() {
    let dir = tempfile::tempdir().unwrap();

    let missing = dir.path().join("none.yaml");
    assert_eq!(
        convert(&request(missing, TargetFormat::Toml))
            .unwrap_err()
            .kind,
        K::FileMissing
    );

    assert_eq!(
        convert(&request(dir.path().to_path_buf(), TargetFormat::Toml))
            .unwrap_err()
            .kind,
        K::Unreadable
    );

    // 1 MiB を超える入力は、中身が壊れていても tooLarge(BR1.12 の 3 は 5 より先)
    let big = dir.path().join("big.yaml");
    std::fs::write(&big, format!("a: [{}\n", "0,".repeat(600_000))).unwrap();
    assert_eq!(
        convert(&request(big, TargetFormat::Toml)).unwrap_err().kind,
        K::TooLarge
    );

    // UTF-8 でない入力は、文法を見る前に unreadable
    let bad = dir.path().join("bad.yaml");
    std::fs::write(&bad, [b'a', b':', b' ', 0xff, 0xfe, b'\n']).unwrap();
    assert_eq!(
        convert(&request(bad, TargetFormat::Toml)).unwrap_err().kind,
        K::Unreadable
    );
}

#[test]
fn ちょうど1_mibの正しい入力は変換できる() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("max.yaml");
    let head = "a: \"";
    let tail = "\"\n";
    let fill = 1024 * 1024 - head.len() - tail.len();
    std::fs::write(&path, format!("{head}{}{tail}", "x".repeat(fill))).unwrap();
    assert_eq!(std::fs::metadata(&path).unwrap().len(), 1024 * 1024);
    let out = convert(&request(path, TargetFormat::Toml)).unwrap();
    assert!(out.text.starts_with("a = \"xxx"));
}

#[test]
fn convert_text_はファイルに触れず_ラベルを使う() {
    let e = convert_text("a: [1\n", TargetFormat::Toml, "my-label.yaml").unwrap_err();
    assert_eq!(e.file, "my-label.yaml");
    let out = convert_text("a: 1\n", TargetFormat::Toml, "x").unwrap();
    assert_eq!(out.text, "a = 1\n");
}

#[test]
fn コメントつきの入力は変換でき_コメントは捨てる() {
    let out = convert_text(
        "# 先頭のコメント\na: 1 # 末尾のコメント\n",
        TargetFormat::Toml,
        "t",
    )
    .unwrap();
    assert!(!out.text.contains('#'), "{}", out.text);
    let out = convert_text("# comment\na = 1 # c\n", TargetFormat::Yaml, "t").unwrap();
    assert!(!out.text.contains('#'), "{}", out.text);
}
