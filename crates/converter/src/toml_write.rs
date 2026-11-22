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

//! TOML の書き出し(Write)。
//!
//! 途中の表現を `toml` のライブラリの値に写し、ライブラリの書き出しで文字列にする(T2)。
//! 文字列の引用符、浮動小数(`1.0`、`-0.0`、`nan`、`inf`)の形、表と表の配列の分け方は、
//! ライブラリに任せる。マップのキーの順序は、ライブラリの既定(整列)に従う(BR1.3)。

use crate::error::{ConversionError, ConversionErrorKind, reason};
use crate::types::Value;

/// 途中の表現を、`toml::Value` に写す。深さは 32 までに抑えられている(Resolve)ため、再帰は浅い。
fn to_toml(value: &Value) -> toml::Value {
    match value {
        Value::Str(s) => toml::Value::String(s.clone()),
        Value::Int(n) => toml::Value::Integer(*n),
        Value::Float(f) => toml::Value::Float(*f),
        Value::Bool(b) => toml::Value::Boolean(*b),
        Value::Array(items) => toml::Value::Array(items.iter().map(to_toml).collect()),
        Value::Map(entries) => {
            let mut table = toml::Table::new();
            for (k, v) in entries {
                table.insert(k.clone(), to_toml(v));
            }
            toml::Value::Table(table)
        }
    }
}

/// 途中の表現(最上位はマップ)を、TOML の文字列にする。
pub(crate) fn write(value: &Value, file: &str) -> Result<String, ConversionError> {
    let failed = || {
        ConversionError::new(
            file,
            None,
            None,
            ConversionErrorKind::Unrepresentable,
            reason::WRITE_FAILED,
        )
    };
    match to_toml(value) {
        toml::Value::Table(table) => toml::to_string(&table).map_err(|_| failed()),
        _ => Err(failed()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> toml::Table {
        text.parse().expect("書き出した TOML は読める")
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
    fn 文字列は引用符で囲み_そのまま残る() {
        let v = map(vec![
            ("a", Value::Str("yes".into())),
            ("b", Value::Str("1".into())),
            ("c", Value::Str("".into())),
            ("<<", Value::Str("null".into())),
            ("with space", Value::Str(" lead\n\"q\"\u{1}".into())),
        ]);
        let text = write(&v, "t").unwrap();
        let t = parse(&text);
        assert_eq!(t["a"].as_str(), Some("yes"));
        assert_eq!(t["b"].as_str(), Some("1"));
        assert_eq!(t["c"].as_str(), Some(""));
        assert_eq!(t["<<"].as_str(), Some("null"));
        assert_eq!(t["with space"].as_str(), Some(" lead\n\"q\"\u{1}"));
    }

    #[test]
    fn 浮動小数と整数の端() {
        let v = map(vec![
            ("one", Value::Float(1.0)),
            ("nz", Value::Float(-0.0)),
            ("nan", Value::Float(f64::NAN)),
            ("inf", Value::Float(f64::INFINITY)),
            ("ninf", Value::Float(f64::NEG_INFINITY)),
            ("max", Value::Int(i64::MAX)),
            ("min", Value::Int(i64::MIN)),
        ]);
        let text = write(&v, "t").unwrap();
        assert!(text.contains("one = 1.0"), "{text}");
        let t = parse(&text);
        assert!(t["one"].is_float());
        assert!(
            t["nz"]
                .as_float()
                .is_some_and(|f| f == 0.0 && f.is_sign_negative())
        );
        assert!(t["nan"].as_float().is_some_and(f64::is_nan));
        assert_eq!(t["inf"].as_float(), Some(f64::INFINITY));
        assert_eq!(t["ninf"].as_float(), Some(f64::NEG_INFINITY));
        assert_eq!(t["max"].as_integer(), Some(i64::MAX));
        assert_eq!(t["min"].as_integer(), Some(i64::MIN));
    }

    #[test]
    fn テーブルと_テーブルの配列と_空の値() {
        let v = map(vec![
            ("top", Value::Int(1)),
            ("empty_map", Value::Map(vec![])),
            ("empty_arr", Value::Array(vec![])),
            (
                "job",
                Value::Array(vec![
                    map(vec![("id", Value::Str("a".into()))]),
                    map(vec![
                        ("id", Value::Str("b".into())),
                        ("n", map(vec![("x", Value::Bool(true))])),
                    ]),
                ]),
            ),
            (
                "mixed",
                Value::Array(vec![Value::Int(1), Value::Str("a".into())]),
            ),
        ]);
        let text = write(&v, "t").unwrap();
        let t = parse(&text);
        assert_eq!(t["job"].as_array().map(Vec::len), Some(2));
        assert_eq!(t["mixed"].as_array().map(Vec::len), Some(2));
        assert!(t["empty_map"].as_table().is_some_and(|m| m.is_empty()));
    }

    #[test]
    fn 空の最上位は空の文字列() {
        assert_eq!(write(&Value::Map(vec![]), "t").unwrap(), "");
    }

    #[test]
    fn マップでない最上位は誤り() {
        let e = write(&Value::Int(1), "t").unwrap_err();
        assert_eq!(e.kind, ConversionErrorKind::Unrepresentable);
    }
}
