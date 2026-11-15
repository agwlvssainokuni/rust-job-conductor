//! 往復(BR1.11): 変換した結果を読み戻した木が、入力の木と等しい。

mod common;

use common::{a, i, m, s, toml, yaml};
use conductor_converter::testing::Value;
use conductor_converter::{TargetFormat, convert_text};

const PIPELINE_TOML: &str = include_str!("fixtures/pipeline.toml");
const PIPELINE_YAML: &str = include_str!("fixtures/pipeline.yaml");

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
fn definition_like_toml_roundtrips_through_yaml() {
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
fn definition_like_yaml_roundtrips_through_toml() {
    let toml_text = assert_roundtrip(PIPELINE_YAML, TargetFormat::Yaml);
    assert!(toml_text.contains("[[job]]"), "{toml_text}");
    // アンカー、エイリアス、マージキーは、出力に残らない
    assert!(!toml_text.contains("<<") && !toml_text.contains('*') && !toml_text.contains('&'));
}

#[test]
fn merged_values_are_expanded_with_priority() {
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
fn scalar_boundaries_roundtrip_from_yaml() {
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
fn scalar_boundaries_roundtrip_from_toml() {
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
fn floats_roundtrip_both_ways() {
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
fn i64_edges_roundtrip() {
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
fn empty_and_nested_collections_roundtrip() {
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

#[test]
fn deepest_allowed_structure_roundtrips() {
    // 32 段(根のマップ 1 + 配列 31)
    let yaml_text = format!("a: {}x{}\n", "[".repeat(31), "]".repeat(31));
    assert_roundtrip(&yaml_text, TargetFormat::Yaml);
}

#[test]
fn output_is_deterministic() {
    let first = convert_text(PIPELINE_YAML, TargetFormat::Toml, "t").unwrap();
    let second = convert_text(PIPELINE_YAML, TargetFormat::Toml, "t").unwrap();
    assert_eq!(first, second);
    let first = convert_text(PIPELINE_TOML, TargetFormat::Yaml, "t").unwrap();
    let second = convert_text(PIPELINE_TOML, TargetFormat::Yaml, "t").unwrap();
    assert_eq!(first, second);
}
