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

//! 受け入れ基準と、`convert`(ファイルを読む入口)、変換の出力が毎回同じであること。変換の結果を
//! 読んだ木を入力の木と比べる 2 件と、往復の木を比べるテストは、単体テストの
//! `src/tests/convert_roundtrip.rs` にある。

use std::path::{Path, PathBuf};

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

const PIPELINE_TOML: &str = include_str!("fixtures/pipeline.toml");
const PIPELINE_YAML: &str = include_str!("fixtures/pipeline.yaml");

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
fn 出力は毎回同じ() {
    let first = convert_text(PIPELINE_YAML, TargetFormat::Toml, "t").unwrap();
    let second = convert_text(PIPELINE_YAML, TargetFormat::Toml, "t").unwrap();
    assert_eq!(first, second);
    let first = convert_text(PIPELINE_TOML, TargetFormat::Yaml, "t").unwrap();
    let second = convert_text(PIPELINE_TOML, TargetFormat::Yaml, "t").unwrap();
    assert_eq!(first, second);
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

#[test]
fn 文法の誤りの理由は形式ごとの文で_種類の文と重ならない() {
    // YAML: 理由は「YAML として読めません」。表示は「<ファイル>:<行>: 文法の誤りです: YAML として読めません」。
    let e = convert(&request(fixture("broken.yaml"), TargetFormat::Toml)).unwrap_err();
    assert_eq!(e.kind, K::Syntax);
    assert_eq!(e.reason, "YAML として読めません");
    let shown = e.to_string();
    assert!(
        shown.contains(": 文法の誤りです: YAML として読めません"),
        "{shown}"
    );
    assert!(!shown.contains("文法の誤りです: 文法の誤りです"), "{shown}");

    // TOML: 理由は「TOML として読めません」。
    let e = convert(&request(fixture("broken.toml"), TargetFormat::Yaml)).unwrap_err();
    assert_eq!(e.kind, K::Syntax);
    assert_eq!(e.reason, "TOML として読めません");
    let shown = e.to_string();
    assert!(
        shown.contains(":7: 文法の誤りです: TOML として読めません"),
        "{shown}"
    );
    assert!(!shown.contains("文法の誤りです: 文法の誤りです"), "{shown}");
}

/// 書き手のいない名前付きパイプの入力は、開かずに `unreadable`(待ち続けない。2a)。
#[cfg(unix)]
#[test]
fn 名前付きパイプの入力は開かずに_unreadable() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let path = dir.path().join("pipe.toml");
    let status = std::process::Command::new("mkfifo")
        .arg(&path)
        .status()
        .expect("mkfifo を起動できる");
    assert!(status.success(), "mkfifo が失敗した");
    // 止まったときに落ちるよう、別のスレッドで呼び、5 秒で打ち切る。
    let (tx, rx) = std::sync::mpsc::channel();
    let input = path.clone();
    std::thread::spawn(move || {
        let _ = tx.send(convert(&request(input, TargetFormat::Yaml)));
    });
    let r = rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("5 秒以内に返る(開いて待たない)");
    let e = r.expect_err("誤り");
    assert_eq!(e.kind, K::Unreadable);
    assert_eq!(e.reason, "通常のファイルではありません");
    assert!(e.line.is_none());
    // 名前付きパイプは、そのまま残る(読まない・消さない)。
    assert!(path.exists());
}
