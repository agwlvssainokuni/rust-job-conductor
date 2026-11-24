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

//! 往復(BR1.11): 変換した結果を読み戻した木が、入力の木と等しい。
//!
//! あわせて、受け入れ基準(AC7.1.1、AC7.1.2)のうち、変換の結果を読んだ木を、入力の木と比べるもの。
//! ほかの受け入れ基準は、統合テストの `tests/convert_ac.rs` にある。

use std::path::{Path, PathBuf};

use super::{a, i, m, s, toml, yaml};
use crate::types::Value;
use crate::{ConversionRequest, TargetFormat, convert, convert_text};

const PIPELINE_TOML: &str = include_str!("../../tests/fixtures/pipeline.toml");
const PIPELINE_YAML: &str = include_str!("../../tests/fixtures/pipeline.yaml");

fn read(text: &str, format: TargetFormat) -> Value {
    match format {
        TargetFormat::Yaml => yaml(text),
        TargetFormat::Toml => toml(text),
    }
    .unwrap_or_else(|e| panic!("読めない: {e}\n{text}"))
}

/// `text`(`from` の形式)を、反対の形式へ変換し、読み戻した木が等しいことを確かめる。
/// さらに、元の形式へ戻した結果も、等しいことを確かめる。変換後の文字列を返す。
fn assert_roundtrip(text: &str, from: TargetFormat) -> String {
    let to = match from {
        TargetFormat::Yaml => TargetFormat::Toml,
        TargetFormat::Toml => TargetFormat::Yaml,
    };
    let original = read(text, from);
    let converted =
        convert_text(text, to, "in").unwrap_or_else(|e| panic!("変換できない: {e}\n{text}"));
    assert_eq!(
        read(&converted.text, to),
        original,
        "往復で木が変わった:\n{}",
        converted.text
    );
    let back = convert_text(&converted.text, from, "mid").expect("戻せる");
    assert_eq!(
        read(&back.text, from),
        original,
        "戻した結果の木が変わった:\n{}",
        back.text
    );
    converted.text
}

#[test]
fn 定義の形の_toml_は_yaml_を通って往復する() {
    let yaml_text = assert_roundtrip(PIPELINE_TOML, TargetFormat::Toml);
    // `on` キーは、文字列のキーとして保たれる
    let tree = read(&yaml_text, TargetFormat::Yaml);
    let Value::Map(top) = tree else { panic!("map") };
    let jobs = &top.iter().find(|(k, _)| k == "job").expect("job").1;
    let Value::Array(jobs) = jobs else {
        panic!("array")
    };
    assert_eq!(jobs.len(), 4);
}

#[test]
fn 定義の形の_yaml_は_toml_を通って往復する() {
    let toml_text = assert_roundtrip(PIPELINE_YAML, TargetFormat::Yaml);
    assert!(toml_text.contains("[[job]]"), "{toml_text}");
    // アンカー、エイリアス、マージキーは、出力に残らない
    assert!(!toml_text.contains("<<") && !toml_text.contains('*') && !toml_text.contains('&'));
}

/// 文字列と配列の `command` を持つ定義(配列には、空白・引用符・`$`・空の要素を含む)。
const ARRAY_COMMAND_TOML: &str = r#"format_version = 1

[pipeline]
name = "p"

[[job]]
id = "a"
command = "echo a"

[[job]]
id = "b"
command = ["my prog", "a b", "'q'", "$HOME", ""]
"#;

const ARRAY_COMMAND_YAML: &str = r#"format_version: 1
pipeline:
  name: p
job:
  - id: a
    command: echo a
  - id: b
    command:
      - my prog
      - a b
      - "'q'"
      - $HOME
      - ""
"#;

fn command_of(tree: &Value, index: usize) -> Value {
    let Value::Map(top) = tree else { panic!("map") };
    let Value::Array(jobs) = &top.iter().find(|(k, _)| k == "job").expect("job").1 else {
        panic!("array")
    };
    let Value::Map(job) = &jobs[index] else {
        panic!("map")
    };
    job.iter()
        .find(|(k, _)| k == "command")
        .expect("command")
        .1
        .clone()
}

#[test]
fn 配列形式の_command_は往復する_toml_yaml_toml() {
    let yaml_text = assert_roundtrip(ARRAY_COMMAND_TOML, TargetFormat::Toml);
    let tree = read(&yaml_text, TargetFormat::Yaml);
    assert_eq!(command_of(&tree, 0), s("echo a"));
    assert_eq!(
        command_of(&tree, 1),
        a(vec![s("my prog"), s("a b"), s("'q'"), s("$HOME"), s("")])
    );
}

#[test]
fn 配列形式の_command_は往復する_yaml_toml_yaml() {
    let toml_text = assert_roundtrip(ARRAY_COMMAND_YAML, TargetFormat::Yaml);
    let tree = read(&toml_text, TargetFormat::Toml);
    assert_eq!(command_of(&tree, 0), s("echo a"));
    assert_eq!(
        command_of(&tree, 1),
        a(vec![s("my prog"), s("a b"), s("'q'"), s("$HOME"), s("")])
    );
}

#[test]
fn マージした値は優先順位どおりに展開する() {
    let tree = read(PIPELINE_YAML, TargetFormat::Yaml);
    let Value::Map(top) = tree else { panic!("map") };
    let Value::Array(jobs) = &top.iter().find(|(k, _)| k == "job").unwrap().1 else {
        panic!("array")
    };
    let transform = &jobs[1];
    let expected = m(vec![
        ("id", s("transform")),
        ("command", s("transform \"in dir\" $HOME")),
        // マップ自身のキーが優先、ほかはマージから
        ("success_exit_codes", a(vec![i(0)])),
        ("warning_exit_codes", a(vec![i(3), i(4)])),
        (
            "transitions",
            a(vec![
                m(vec![("on", s("warning")), ("next", s("notify"))]),
                m(vec![("on", s("error")), ("next", s("@end"))]),
            ]),
        ),
    ]);
    assert_eq!(transform, &expected);
}

#[test]
fn スカラーの境界は往復する_yaml_から() {
    let text = r##"
"yes": "yes"
"no": "no"
"on": "on"
"off": "off"
"1": "1"
"1.5": "1.5"
"null": "null"
"~": "~"
"true": "true"
"false": "false"
"<<": "<<"
"": ""
" lead": "trail "
"a: b": "x #y"
nl: "line1\nline2\ttab \"q\" \\ back"
ctrl: "\u0001\u007f\u0085 "
unicode: 日本語のテキスト😀
plain: echo hello world
path: /usr/bin/env
dash: "- x"
colon: "k:"
hash: "#c"
amp: "&a"
star: "*a"
bang: "!t"
brace: "{x}"
bracket: "[x]"
percent: "%x"
at: "@end"
backtick: "`x"
long: "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
"##;
    assert_roundtrip(text, TargetFormat::Yaml);
}

#[test]
fn スカラーの境界は往復する_toml_から() {
    let text = r#"
"yes" = "yes"
"1" = "1"
"null" = "null"
"true" = "true"
"<<" = "<<"
"" = ""
" lead" = "trail "
"a b" = "line1\nline2\ttab \"q\" \\ back"
ctrl = "\u0001\u007f\u0085 "
unicode = "日本語のテキスト😀"
"#;
    assert_roundtrip(text, TargetFormat::Toml);
}

#[test]
fn 浮動小数は両方向に往復する() {
    let yaml_text = "one: 1.0\nneg_zero: -0.0\nnan: .nan\ninf: .inf\nninf: -.inf\nsmall: 1.5e-7\nbig: 1e21\nint: 1\nfrac: 0.1\n";
    let toml_text = assert_roundtrip(yaml_text, TargetFormat::Yaml);
    let tree = read(&toml_text, TargetFormat::Toml);
    let Value::Map(entries) = tree else {
        panic!("map")
    };
    let get = |k: &str| &entries.iter().find(|(key, _)| key == k).unwrap().1;
    assert!(matches!(get("one"), Value::Float(f) if *f == 1.0));
    assert!(matches!(get("int"), Value::Int(1)));
    assert!(matches!(get("neg_zero"), Value::Float(f) if f.is_sign_negative() && *f == 0.0));

    let toml_in =
        "one = 1.0\nneg_zero = -0.0\nnan = nan\ninf = inf\nninf = -inf\nint = 1\nsmall = 1.5e-7\n";
    let yaml_out = assert_roundtrip(toml_in, TargetFormat::Toml);
    assert!(yaml_out.contains("one: 1.0"), "{yaml_out}");
    assert!(yaml_out.contains("neg_zero: -0.0"), "{yaml_out}");
    assert!(yaml_out.contains("nan: .nan"), "{yaml_out}");
    assert!(yaml_out.contains("inf: .inf"), "{yaml_out}");
    assert!(yaml_out.contains("ninf: -.inf"), "{yaml_out}");
}

#[test]
fn i64_の端は往復する() {
    let toml_in = "max = 9223372036854775807\nmin = -9223372036854775808\nzero = 0\n";
    let yaml_out = assert_roundtrip(toml_in, TargetFormat::Toml);
    assert!(yaml_out.contains("max: 9223372036854775807"), "{yaml_out}");
    assert!(yaml_out.contains("min: -9223372036854775808"), "{yaml_out}");
    assert_roundtrip(
        "max: 9223372036854775807\nmin: -9223372036854775808\nhex: 0x7FFFFFFFFFFFFFFF\n",
        TargetFormat::Yaml,
    );
}

#[test]
fn 空と入れ子の集まりは往復する() {
    assert_roundtrip("{}\n", TargetFormat::Yaml);
    assert_eq!(
        convert_text("{}\n", TargetFormat::Toml, "t").unwrap().text,
        ""
    );
    assert_eq!(
        convert_text("", TargetFormat::Yaml, "t").unwrap().text,
        "{}\n"
    );
    assert_roundtrip("", TargetFormat::Toml);

    let yaml_text = "\
empty_arr: []
empty_map: {}
nested:
  - [1, [2, [3, []]]]
  - {a: {b: {c: []}}}
  - []
  - {}
  - - x
    - y
matrix:
  - [1, 2]
  - [3, 4]
mixed: [1, a, true, 1.5, {k: v}, [z]]
";
    assert_roundtrip(yaml_text, TargetFormat::Yaml);

    let toml_text = "\
empty_arr = []
empty_map = {}
nested = [[1, [2, [3, []]]], { a = { b = { c = [] } } }, [], {}, [\"x\", \"y\"]]
mixed = [1, \"a\", true, 1.5, { k = \"v\" }, [\"z\"]]

[deep.er.still]
x = 1

[[arr.of.tables]]
n = 1

[[arr.of.tables]]
n = 2
";
    assert_roundtrip(toml_text, TargetFormat::Toml);
}

/// `[job.retry]` を含む定義(FR7.2、BR1.15)。整数・文字列・整数の配列を持つ。
const RETRY_TOML: &str = "format_version = 1

[pipeline]
name = \"p\"

[[job]]
id = \"a\"
command = \"echo a\"

[job.retry]
exit_codes = [75, 127]
max = 3
interval = \"30s\"
multiplier = 2
max_interval = \"10m\"
";

#[test]
fn retry_の表は往復する_yaml_を通って_toml_に戻る() {
    let yaml_text = assert_roundtrip(RETRY_TOML, TargetFormat::Toml);
    // 値(整数と文字列)が、型を保って YAML に出る
    let tree = read(&yaml_text, TargetFormat::Yaml);
    let Value::Map(top) = tree else { panic!("map") };
    let Value::Array(jobs) = &top.iter().find(|(k, _)| k == "job").expect("job").1 else {
        panic!("array")
    };
    let Value::Map(job) = &jobs[0] else {
        panic!("map")
    };
    let Value::Map(retry) = &job.iter().find(|(k, _)| k == "retry").expect("retry").1 else {
        panic!("map")
    };
    assert_eq!(
        retry.iter().find(|(k, _)| k == "max").map(|(_, v)| v),
        Some(&i(3))
    );
    assert_eq!(
        retry.iter().find(|(k, _)| k == "interval").map(|(_, v)| v),
        Some(&s("30s"))
    );
}

#[test]
fn retry_の表は往復する_yaml_から() {
    let yaml_text = convert_text(RETRY_TOML, TargetFormat::Yaml, "t")
        .unwrap()
        .text;
    assert_roundtrip(&yaml_text, TargetFormat::Yaml);
}

#[test]
fn 許される最も深い構造は往復する() {
    // 32 段(根のマップ 1 + 配列 31)
    let yaml_text = format!("a: {}x{}\n", "[".repeat(31), "]".repeat(31));
    assert_roundtrip(&yaml_text, TargetFormat::Yaml);
}

// ---- 受け入れ基準。ファイルを読む入口 `convert` の結果の木 ----

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
        toml(include_str!("../../tests/fixtures/pipeline.toml")).unwrap()
    );
    assert!(out.text.contains("format_version: 1"), "{}", out.text);
    assert!(out.text.contains("id: extract"), "{}", out.text);
}

#[test]
fn ac_7_1_2_アンカーつきの_yaml_から_toml() {
    let out = convert(&request(fixture("pipeline.yaml"), TargetFormat::Toml)).unwrap();
    assert_eq!(
        toml(&out.text).unwrap(),
        yaml(include_str!("../../tests/fixtures/pipeline.yaml")).unwrap()
    );
    assert!(!out.text.contains("<<"), "{}", out.text);
    // 展開された結果が、定義の形(表の配列)で出力される
    assert!(out.text.contains("[[job]]"), "{}", out.text);
}
