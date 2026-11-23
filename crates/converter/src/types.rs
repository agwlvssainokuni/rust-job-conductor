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

//! 型(Types)。呼び出しの入力・出力の型と、途中の表現 `Value`。
//!
//! 入出力に依存しない。`Value` は、この Unit の内部の表現であり、他の Unit に公開しない
//! (テストのために `#[doc(hidden)]` で公開している)。

use std::collections::HashMap;
use std::path::PathBuf;

/// 変換後の形式(BR1.1)。入力の形式は、これと反対の形式とみなす。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetFormat {
    Yaml,
    Toml,
}

impl TargetFormat {
    /// 反対の形式(`Yaml` ↔ `Toml`)。変換後の形式から、入力の形式を求めるのに使う(BR1.1)。
    pub fn opposite(self) -> Self {
        match self {
            Self::Yaml => Self::Toml,
            Self::Toml => Self::Yaml,
        }
    }
}

/// 変換の呼び出しの入力(契約 C4 の input)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversionRequest {
    /// 変換する入力ファイルのパス。
    pub input_path: PathBuf,
    /// 変換後の形式。
    pub target_format: TargetFormat,
}

/// 変換の結果の文字列(契約 C4 の output)。書き出しは呼び出し側(Cli)の責務(BR1.9)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvertedText {
    /// 変換後の形式として、文法的に正しい文字列。
    pub text: String,
}

/// 途中の表現(D1)。文字列、整数(`i64`)、浮動小数、真偽値、配列、マップ(キーは文字列)だけを持つ。
///
/// マップは、入力に現れる順を保つ並びで持つ。等しさ(BR1.11)は、マップはキーの順序を問わず、
/// 浮動小数は NaN どうしを等しく、`0.0` と `-0.0` を区別して比べる。
#[derive(Debug, Clone)]
pub enum Value {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<Value>),
    Map(Vec<(String, Value)>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => {
                (a.is_nan() && b.is_nan()) || a.to_bits() == b.to_bits()
            }
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Map(a), Value::Map(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                let index: HashMap<&str, &Value> = b.iter().map(|(k, v)| (k.as_str(), v)).collect();
                if index.len() != b.len() {
                    return false;
                }
                a.iter()
                    .all(|(k, v)| index.get(k.as_str()).is_some_and(|other| *other == v))
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(entries: &[(&str, Value)]) -> Value {
        Value::Map(
            entries
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
        )
    }

    #[test]
    fn nan_は_nan_と等しく_符号の違う0は異なる() {
        assert_eq!(Value::Float(f64::NAN), Value::Float(-f64::NAN));
        assert_ne!(Value::Float(0.0), Value::Float(-0.0));
        assert_eq!(Value::Float(-0.0), Value::Float(-0.0));
        assert_ne!(Value::Float(f64::INFINITY), Value::Float(f64::NEG_INFINITY));
    }

    #[test]
    fn 種類が違えば等しくない() {
        assert_ne!(Value::Str("1".into()), Value::Int(1));
        assert_ne!(Value::Int(1), Value::Float(1.0));
        assert_ne!(Value::Bool(true), Value::Str("true".into()));
    }

    #[test]
    fn マップの等しさはキーの順を見ず_配列は順を見る() {
        let a = map(&[("x", Value::Int(1)), ("y", Value::Int(2))]);
        let b = map(&[("y", Value::Int(2)), ("x", Value::Int(1))]);
        assert_eq!(a, b);
        assert_ne!(a, map(&[("x", Value::Int(1))]));
        assert_ne!(a, map(&[("x", Value::Int(1)), ("y", Value::Int(3))]));
        assert_ne!(a, map(&[("x", Value::Int(1)), ("z", Value::Int(2))]));
        let arr1 = Value::Array(vec![Value::Int(1), Value::Int(2)]);
        let arr2 = Value::Array(vec![Value::Int(2), Value::Int(1)]);
        assert_ne!(arr1, arr2);
    }

    #[test]
    fn 反対の形式は_yaml_と_toml_を入れ替え_2_回で元に戻る() {
        assert_eq!(TargetFormat::Yaml.opposite(), TargetFormat::Toml);
        assert_eq!(TargetFormat::Toml.opposite(), TargetFormat::Yaml);
        for format in [TargetFormat::Yaml, TargetFormat::Toml] {
            assert_ne!(format.opposite(), format, "{format:?}");
            assert_eq!(format.opposite().opposite(), format, "{format:?}");
        }
    }
}
