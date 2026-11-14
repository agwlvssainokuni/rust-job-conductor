//! YAML 1.2 のコアスキーマによる、スカラーの解決。
//!
//! イベントの列は、スカラーを文字列として返すため、型の解決は、この Unit が行う。
//! `true` / `false` だけが真偽値、`null` / `~` / 空は null、整数、浮動小数、それ以外は文字列。
//! `yes` / `no` / `on` / `off` は文字列である(BR1.2)。

use crate::raw::{CoreTag, RawScalar, Tag};

/// 解決したスカラー。
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Resolved {
    Null,
    Bool(bool),
    Int(i64),
    /// 整数の書式だが、`i64` を超える。
    IntOutOfRange,
    Float(f64),
    /// 浮動小数の書式だが、有限の `f64` で表せない(あふれて無限大になる)。
    FloatOutOfRange,
    Str(String),
}

/// 引用符なしのスカラー(plain)を、コアスキーマで解決する。
pub(crate) fn resolve_plain(text: &str) -> Resolved {
    match text {
        "" | "~" | "null" | "Null" | "NULL" => return Resolved::Null,
        "true" | "True" | "TRUE" => return Resolved::Bool(true),
        "false" | "False" | "FALSE" => return Resolved::Bool(false),
        ".inf" | ".Inf" | ".INF" | "+.inf" | "+.Inf" | "+.INF" => {
            return Resolved::Float(f64::INFINITY);
        }
        "-.inf" | "-.Inf" | "-.INF" => return Resolved::Float(f64::NEG_INFINITY),
        ".nan" | ".NaN" | ".NAN" => return Resolved::Float(f64::NAN),
        _ => {}
    }
    if let Some(r) = resolve_int(text) {
        return r;
    }
    if is_core_float(text) {
        return match text.parse::<f64>() {
            Ok(f) if f.is_finite() => Resolved::Float(f),
            _ => Resolved::FloatOutOfRange,
        };
    }
    Resolved::Str(text.to_string())
}

fn resolve_int(text: &str) -> Option<Resolved> {
    let bytes = text.as_bytes();
    if let Some(rest) = text.strip_prefix("0x") {
        if !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Some(from_radix(rest, 16));
        }
        return None;
    }
    if let Some(rest) = text.strip_prefix("0o") {
        if !rest.is_empty() && rest.bytes().all(|b| (b'0'..=b'7').contains(&b)) {
            return Some(from_radix(rest, 8));
        }
        return None;
    }
    let digits = match bytes.first() {
        Some(b'+') | Some(b'-') => &text[1..],
        _ => text,
    };
    if !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) {
        return Some(match text.parse::<i64>() {
            Ok(n) => Resolved::Int(n),
            Err(_) => Resolved::IntOutOfRange,
        });
    }
    None
}

fn from_radix(digits: &str, radix: u32) -> Resolved {
    match i64::from_str_radix(digits, radix) {
        Ok(n) => Resolved::Int(n),
        Err(_) => Resolved::IntOutOfRange,
    }
}

/// `[-+]? ( \.[0-9]+ | [0-9]+ (\.[0-9]*)? ) ([eE] [-+]? [0-9]+)?`(整数だけの書式は、呼び出し側で先に除く)。
fn is_core_float(text: &str) -> bool {
    let b = text.as_bytes();
    let mut i = 0;
    if matches!(b.first(), Some(b'+') | Some(b'-')) {
        i += 1;
    }
    let int_start = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    let int_digits = i - int_start;
    let mut frac_digits = 0;
    if i < b.len() && b[i] == b'.' {
        i += 1;
        let s = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        frac_digits = i - s;
    } else if int_digits == 0 {
        return false;
    }
    if int_digits == 0 && frac_digits == 0 {
        return false;
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        i += 1;
        if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
            i += 1;
        }
        let s = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == s {
            return false;
        }
    }
    i == b.len()
}

/// 表せない理由(BR1.6)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Unrep {
    Null,
    Tagged,
    NumberRange,
    Datetime,
}

/// 素のスカラーを、解決した値にする。表せないものは `Unrep`。
pub(crate) fn scalar_value(scalar: &RawScalar, tag: Tag) -> Result<crate::types::Value, Unrep> {
    use crate::types::Value;
    match scalar {
        RawScalar::Str(s) => Ok(Value::Str(s.clone())),
        RawScalar::Int(n) => Ok(Value::Int(*n)),
        RawScalar::Float(f) => Ok(Value::Float(*f)),
        RawScalar::Bool(b) => Ok(Value::Bool(*b)),
        RawScalar::Datetime => Err(Unrep::Datetime),
        RawScalar::Yaml { text, plain } => match yaml_resolved(text, *plain, tag)? {
            Resolved::Null => Err(Unrep::Null),
            Resolved::Bool(b) => Ok(Value::Bool(b)),
            Resolved::Int(n) => Ok(Value::Int(n)),
            Resolved::Float(f) => Ok(Value::Float(f)),
            Resolved::IntOutOfRange | Resolved::FloatOutOfRange => Err(Unrep::NumberRange),
            Resolved::Str(s) => Ok(Value::Str(s)),
        },
    }
}

/// タグを考えて、YAML のスカラーを解決する。
///
/// `!!str` は、書き方によらず文字列。`!!int` `!!float` `!!bool` `!!null` は、解決した型が
/// タグと合うときだけ、タグなしと同じに扱う。それ以外のタグは、表せない(`Tagged`)。
pub(crate) fn yaml_resolved(text: &str, plain: bool, tag: Tag) -> Result<Resolved, Unrep> {
    match tag {
        Tag::None => Ok(if plain {
            resolve_plain(text)
        } else {
            Resolved::Str(text.to_string())
        }),
        Tag::Core(CoreTag::Str) => Ok(Resolved::Str(text.to_string())),
        Tag::Core(core) => {
            let r = resolve_plain(text);
            let matches = match core {
                CoreTag::Int => matches!(r, Resolved::Int(_) | Resolved::IntOutOfRange),
                CoreTag::Float => matches!(
                    r,
                    Resolved::Float(_) | Resolved::FloatOutOfRange | Resolved::Int(_)
                ),
                CoreTag::Bool => matches!(r, Resolved::Bool(_)),
                CoreTag::Null => matches!(r, Resolved::Null),
                _ => false,
            };
            if !matches {
                return Err(Unrep::Tagged);
            }
            Ok(match (core, r) {
                (CoreTag::Float, Resolved::Int(n)) => Resolved::Float(n as f64),
                (_, r) => r,
            })
        }
        Tag::Other => Err(Unrep::Tagged),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(text: &str) -> Resolved {
        resolve_plain(text)
    }

    #[test]
    fn core_schema_bool_and_null() {
        assert_eq!(s("true"), Resolved::Bool(true));
        assert_eq!(s("False"), Resolved::Bool(false));
        for t in ["yes", "no", "on", "off", "Yes", "ON", "y", "n"] {
            assert_eq!(s(t), Resolved::Str(t.to_string()), "{t}");
        }
        for t in ["", "~", "null", "Null", "NULL"] {
            assert_eq!(s(t), Resolved::Null, "{t:?}");
        }
        assert_eq!(s("nUll"), Resolved::Str("nUll".into()));
    }

    #[test]
    fn core_schema_integers() {
        assert_eq!(s("0"), Resolved::Int(0));
        assert_eq!(s("-12"), Resolved::Int(-12));
        assert_eq!(s("+7"), Resolved::Int(7));
        assert_eq!(s("007"), Resolved::Int(7));
        assert_eq!(s("0x1F"), Resolved::Int(31));
        assert_eq!(s("0o17"), Resolved::Int(15));
        assert_eq!(s("9223372036854775807"), Resolved::Int(i64::MAX));
        assert_eq!(s("-9223372036854775808"), Resolved::Int(i64::MIN));
        assert_eq!(s("9223372036854775808"), Resolved::IntOutOfRange);
        assert_eq!(s("0xFFFFFFFFFFFFFFFFFF"), Resolved::IntOutOfRange);
        assert_eq!(s("0x"), Resolved::Str("0x".into()));
        assert_eq!(s("0o8"), Resolved::Str("0o8".into()));
        assert_eq!(s("1_000"), Resolved::Str("1_000".into()));
    }

    #[test]
    fn core_schema_floats() {
        assert_eq!(s("1.5"), Resolved::Float(1.5));
        assert_eq!(s("-0.0"), Resolved::Float(-0.0));
        assert_eq!(s(".5"), Resolved::Float(0.5));
        assert_eq!(s("1."), Resolved::Float(1.0));
        assert_eq!(s("1e3"), Resolved::Float(1000.0));
        assert_eq!(s("1.5E-2"), Resolved::Float(0.015));
        assert_eq!(s(".inf"), Resolved::Float(f64::INFINITY));
        assert_eq!(s("-.inf"), Resolved::Float(f64::NEG_INFINITY));
        assert!(matches!(s(".nan"), Resolved::Float(f) if f.is_nan()));
        assert_eq!(s("1e999"), Resolved::FloatOutOfRange);
        for t in [
            ".", "e3", "1e", "1.2.3", "-", "+", ".e1", "1 .5", "inf", "nan", "NaN",
        ] {
            assert_eq!(s(t), Resolved::Str(t.to_string()), "{t:?}");
        }
    }

    #[test]
    fn quoted_scalars_are_always_strings() {
        assert_eq!(
            yaml_resolved("1", false, Tag::None),
            Ok(Resolved::Str("1".into()))
        );
        assert_eq!(
            yaml_resolved("null", false, Tag::None),
            Ok(Resolved::Str("null".into()))
        );
    }

    #[test]
    fn tags() {
        assert_eq!(
            yaml_resolved("1", true, Tag::Core(CoreTag::Str)),
            Ok(Resolved::Str("1".into()))
        );
        assert_eq!(
            yaml_resolved("1", false, Tag::Core(CoreTag::Int)),
            Ok(Resolved::Int(1))
        );
        assert_eq!(
            yaml_resolved("1", true, Tag::Core(CoreTag::Float)),
            Ok(Resolved::Float(1.0))
        );
        assert_eq!(
            yaml_resolved("x", true, Tag::Core(CoreTag::Int)),
            Err(Unrep::Tagged)
        );
        assert_eq!(
            yaml_resolved("x", true, Tag::Core(CoreTag::Unsupported)),
            Err(Unrep::Tagged)
        );
        assert_eq!(yaml_resolved("x", true, Tag::Other), Err(Unrep::Tagged));
        assert_eq!(
            yaml_resolved("x", true, Tag::Core(CoreTag::Map)),
            Err(Unrep::Tagged)
        );
    }

    #[test]
    fn scalar_value_maps_unrepresentable() {
        let y = |t: &str| RawScalar::Yaml {
            text: t.into(),
            plain: true,
        };
        assert_eq!(scalar_value(&y("~"), Tag::None), Err(Unrep::Null));
        assert_eq!(
            scalar_value(&y("99999999999999999999"), Tag::None),
            Err(Unrep::NumberRange)
        );
        assert_eq!(
            scalar_value(&RawScalar::Datetime, Tag::None),
            Err(Unrep::Datetime)
        );
        assert_eq!(
            scalar_value(&y("on"), Tag::None),
            Ok(crate::types::Value::Str("on".into()))
        );
    }
}
