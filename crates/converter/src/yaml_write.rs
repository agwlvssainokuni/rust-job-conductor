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

//! YAML の書き出し(Write)。
//!
//! 途中の表現を、ブロック形式の YAML にする。文字列は、値もキーも、読み戻したときに
//! 文字列のままになるように、必要なら二重引用符で囲む。キー `<<` は、必ず引用符で囲む
//! (マージキーとして読み戻されないように。BR1.2)。マップの順序は、`Value` の並び。
//! 深さは 32 までに抑えられている(Resolve)ため、再帰は浅い。

use crate::scalar::{Resolved, resolve_plain};
use crate::types::Value;

/// 他の YAML の処理系(YAML 1.1)が、真偽値や null として読む語。引用符で囲む。
const YAML11_WORDS: &[&str] = &[
    "y", "Y", "yes", "Yes", "YES", "n", "N", "no", "No", "NO", "on", "On", "ON", "off", "Off",
    "OFF",
];

/// 途中の表現を、YAML の文字列にする。最上位はマップ(RootCheck で確認済み)。
pub(crate) fn write(value: &Value) -> String {
    let mut out = String::new();
    match value {
        Value::Map(entries) if !entries.is_empty() => write_map(&mut out, entries, 0),
        other => {
            out.push_str(&inline(other));
            out.push('\n');
        }
    }
    out
}

fn pad(out: &mut String, n: usize) {
    for _ in 0..n {
        out.push(' ');
    }
}

/// 1行に収まる表現(スカラー、空の配列、空のマップ)。コレクションは `None`。
fn scalar_or_empty(value: &Value) -> Option<String> {
    match value {
        Value::Array(a) if a.is_empty() => Some("[]".to_string()),
        Value::Map(m) if m.is_empty() => Some("{}".to_string()),
        Value::Array(_) | Value::Map(_) => None,
        other => Some(inline(other)),
    }
}

fn inline(value: &Value) -> String {
    match value {
        Value::Str(s) => quote_string(s),
        Value::Int(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Float(f) => float_text(*f),
        Value::Array(_) => "[]".to_string(),
        Value::Map(_) => "{}".to_string(),
    }
}

fn float_text(f: f64) -> String {
    if f.is_nan() {
        ".nan".to_string()
    } else if f.is_infinite() {
        if f > 0.0 { ".inf" } else { "-.inf" }.to_string()
    } else {
        // `{:?}` は、整数値でも `1.0` の形になり、整数と区別できる。符号つきのゼロも保つ
        format!("{f:?}")
    }
}

fn write_map(out: &mut String, entries: &[(String, Value)], indent: usize) {
    for (key, value) in entries {
        pad(out, indent);
        out.push_str(&quote_string(key));
        out.push(':');
        match scalar_or_empty(value) {
            Some(text) => {
                out.push(' ');
                out.push_str(&text);
                out.push('\n');
            }
            None => {
                out.push('\n');
                match value {
                    Value::Map(m) => write_map(out, m, indent + 2),
                    Value::Array(a) => write_seq(out, a, indent + 2),
                    _ => {}
                }
            }
        }
    }
}

fn write_seq(out: &mut String, items: &[Value], indent: usize) {
    for item in items {
        pad(out, indent);
        out.push('-');
        match scalar_or_empty(item) {
            Some(text) => {
                out.push(' ');
                out.push_str(&text);
                out.push('\n');
            }
            None => {
                // 最初の要素は `- ` の同じ行に置き、残りは、2 桁深く揃える
                let mut inner = String::new();
                match item {
                    Value::Map(m) => write_map(&mut inner, m, indent + 2),
                    Value::Array(a) => write_seq(&mut inner, a, indent + 2),
                    _ => {}
                }
                out.push(' ');
                out.push_str(&inner[indent + 2..]);
            }
        }
    }
}

/// 文字列を、読み戻したときに文字列のままになる形にする。必要なら二重引用符で囲む。
fn quote_string(s: &str) -> String {
    if is_plain_safe(s) {
        s.to_string()
    } else {
        double_quoted(s)
    }
}

fn is_plain_safe(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    // 先頭は、英字、`_`、`/`(数字や `-` `.` で始まる語は、数値などに読まれうるため引用符で囲む)
    if !(first.is_ascii_alphabetic() || first == '_' || first == '/') {
        return false;
    }
    if s.ends_with(' ') {
        return false;
    }
    let body_ok = s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/' | ' '));
    if !body_ok || s.contains("  ") {
        return false;
    }
    if YAML11_WORDS.contains(&s) {
        return false;
    }
    matches!(resolve_plain(s), Resolved::Str(_))
}

fn double_quoted(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if needs_unicode_escape(c) => {
                let n = c as u32;
                if n <= 0xFFFF {
                    out.push_str(&format!("\\u{n:04X}"));
                } else {
                    out.push_str(&format!("\\U{n:08X}"));
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// YAML の印字できる文字にない(または、表示を乱す)文字。
fn needs_unicode_escape(c: char) -> bool {
    c.is_control()
        || matches!(
            c,
            '\u{2028}' | '\u{2029}' | '\u{FEFF}' | '\u{FFFE}' | '\u{FFFF}'
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(text: &str) -> Value {
        Value::Str(text.to_string())
    }

    fn map(entries: Vec<(&str, Value)>) -> Value {
        Value::Map(
            entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    #[test]
    fn strings_that_look_like_other_types_are_quoted() {
        for t in [
            "yes", "no", "on", "off", "true", "false", "null", "~", "1", "-1", "1.5", "0x1F", "",
            " lead", "trail ", "<<", "a: b", "a #b", ".nan", "1e3", "Yes", "Y",
        ] {
            let q = quote_string(t);
            assert!(q.starts_with('"') && q.ends_with('"'), "{t:?} -> {q}");
        }
        for t in [
            "hello",
            "echo hello world",
            "/bin/run.sh",
            "a-b_c.d",
            "cargo build --release",
        ] {
            assert_eq!(quote_string(t), t, "{t:?}");
        }
    }

    #[test]
    fn double_quoted_escapes() {
        assert_eq!(double_quoted("a\"b\\c\nd\te"), "\"a\\\"b\\\\c\\nd\\te\"");
        assert_eq!(double_quoted("\u{1}\u{7f}"), "\"\\u0001\\u007F\"");
        assert_eq!(double_quoted("日本語"), "\"日本語\"");
        assert_eq!(double_quoted("\u{2028}"), "\"\\u2028\"");
    }

    #[test]
    fn floats_keep_form() {
        assert_eq!(float_text(1.0), "1.0");
        assert_eq!(float_text(-0.0), "-0.0");
        assert_eq!(float_text(f64::NAN), ".nan");
        assert_eq!(float_text(f64::INFINITY), ".inf");
        assert_eq!(float_text(f64::NEG_INFINITY), "-.inf");
        assert_eq!(float_text(0.1), "0.1");
        assert_eq!(float_text(1e21), "1e21");
    }

    #[test]
    fn nested_structure_layout() {
        let v = map(vec![
            ("name", s("demo")),
            ("num", Value::Int(i64::MIN)),
            ("empty_a", Value::Array(vec![])),
            ("empty_m", Value::Map(vec![])),
            (
                "jobs",
                Value::Array(vec![
                    map(vec![("id", s("a")), ("cmd", s("echo hi"))]),
                    Value::Array(vec![Value::Int(1), Value::Array(vec![Value::Bool(true)])]),
                    s("x"),
                ]),
            ),
            ("on", map(vec![("<<", Value::Int(1))])),
        ]);
        let expected = "\
name: demo
num: -9223372036854775808
empty_a: []
empty_m: {}
jobs:
  - id: a
    cmd: echo hi
  - - 1
    - - true
  - x
\"on\":
  \"<<\": 1
";
        assert_eq!(write(&v), expected);
    }

    #[test]
    fn empty_root_map() {
        assert_eq!(write(&Value::Map(vec![])), "{}\n");
    }
}
