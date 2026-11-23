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

//! run ID(実行単位の識別子)と、その環境変数の名前の検査。
//!
//! run ID はステートファイルの名前の一部になるので、ここの検査を通った値だけを、
//! パスの組み立てに使う(NFR2)。どの関数も純粋で、ファイルにも環境にも触れない。
//! エラーの型は、入力の値を持たない。

use toml::de::{DeTable, DeValue};

/// run ID の最大の長さ(文字数。すべて ASCII なので、バイト数と同じ)。
const MAX_RUN_ID_LEN: usize = 64;

/// 環境変数の名前を省略したときの既定。
pub const DEFAULT_RUN_ID_ENV: &str = "CONDUCTOR_RUN_ID";

/// run ID の検査に通らなかった理由(値は持たない)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunIdError {
    /// 空。
    Empty,
    /// 64 文字を超える。
    TooLong,
    /// `A-Za-z0-9._-` 以外の文字がある。
    InvalidChar,
    /// 先頭が `.`(`..` を含む)。
    LeadingDot,
}

/// 検査済みの run ID。`A-Za-z0-9._-` の 1〜64 文字で、先頭が `.` でない。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RunId(String);

impl RunId {
    /// 値を検査して、run ID にする。
    pub fn parse(value: &str) -> Result<RunId, RunIdError> {
        if value.is_empty() {
            return Err(RunIdError::Empty);
        }
        if value.len() > MAX_RUN_ID_LEN {
            return Err(RunIdError::TooLong);
        }
        if !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
        {
            return Err(RunIdError::InvalidChar);
        }
        if value.starts_with('.') {
            return Err(RunIdError::LeadingDot);
        }
        Ok(RunId(value.to_string()))
    }

    /// 検査済みの文字列。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// [`is_valid_env_name`] の規則を言う句(誤りの理由の文に `concat!` で組み込む文字列のリテラル)。
/// 3 か所の文(`env` の名前、`run_id_env`、`.env` の名前)が使う。
macro_rules! env_name_rule {
    () => {
        "英字または `_` で始まり、英数字と `_` だけ"
    };
}
pub(crate) use env_name_rule;

/// 環境変数の名前の形(`A-Za-z_` で始まり、`A-Za-z0-9_` が続く。1 文字以上)か。
pub fn is_valid_env_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    match bytes.next() {
        Some(b) if b.is_ascii_alphabetic() || b == b'_' => {}
        _ => return false,
    }
    bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

/// 定義の文字列から、`[pipeline].run_id_env` だけをゆるく取り出す。
///
/// ほかの誤りは無視する。読めない・キーがない・文字列でない・名前の形が違う、のどれでも `None`。
/// パニックしない。
pub fn peek_run_id_env(text: &str) -> Option<String> {
    let doc = DeTable::parse(text).ok()?;
    let pipeline = doc.get_ref().get("pipeline")?;
    let DeValue::Table(table) = pipeline.get_ref() else {
        return None;
    };
    let value = table.get("run_id_env")?;
    match value.get_ref() {
        DeValue::String(s) if is_valid_env_name(s) => Some(s.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 有効な_run_id() {
        let long = "a".repeat(64);
        for v in [
            "a",
            "run-1.2_x",
            "A",
            "0",
            "a.b",
            "CON",
            "x-",
            long.as_str(),
        ] {
            let id = RunId::parse(v).unwrap_or_else(|_| panic!("有効のはず: {v}"));
            assert_eq!(id.as_str(), v);
        }
    }

    #[test]
    fn 長さの境界は_64_と_65() {
        assert!(RunId::parse(&"a".repeat(64)).is_ok());
        assert_eq!(RunId::parse(&"a".repeat(65)), Err(RunIdError::TooLong));
        assert_eq!(RunId::parse(""), Err(RunIdError::Empty));
    }

    #[test]
    fn 先頭のドットは拒否する() {
        assert_eq!(RunId::parse(".a"), Err(RunIdError::LeadingDot));
        assert_eq!(RunId::parse(".."), Err(RunIdError::LeadingDot));
        assert_eq!(RunId::parse("."), Err(RunIdError::LeadingDot));
        assert!(RunId::parse("a..b").is_ok());
    }

    #[test]
    fn 使えない文字は拒否する() {
        for v in [
            "a/b", "a\\b", "a b", "é", "a\u{0}b", "a\nb", "a\tb", "a:b", "a*", "日本", "a\u{7f}",
            "../a", "a/..", "~",
        ] {
            assert_eq!(RunId::parse(v), Err(RunIdError::InvalidChar), "{v:?}");
        }
    }

    #[test]
    fn 大文字と小文字は別の値として扱う() {
        assert_ne!(RunId::parse("A").unwrap(), RunId::parse("a").unwrap());
    }

    #[test]
    fn 環境変数の名前の形() {
        for ok in ["A", "_", "_a1", "GITHUB_RUN_ID", "a"] {
            assert!(is_valid_env_name(ok), "{ok}");
        }
        for ng in ["", "1A", "A-B", "A B", "A=B", "é", "A\u{0}", "A.B"] {
            assert!(!is_valid_env_name(ng), "{ng:?}");
        }
    }

    #[test]
    fn peek_は正しい名前だけを返す() {
        let ok = "[pipeline]\nname = \"p\"\nrun_id_env = \"X_ID\"\n";
        assert_eq!(peek_run_id_env(ok), Some("X_ID".to_string()));
        assert_eq!(peek_run_id_env("[pipeline]\nname = \"p\"\n"), None);
        assert_eq!(peek_run_id_env("[pipeline]\nrun_id_env = 5\n"), None);
        assert_eq!(peek_run_id_env("[pipeline]\nrun_id_env = \"1A\"\n"), None);
        assert_eq!(peek_run_id_env("[pipeline]\nrun_id_env = \"\"\n"), None);
        assert_eq!(peek_run_id_env("pipeline = 3\n"), None);
        assert_eq!(peek_run_id_env(""), None);
    }

    #[test]
    fn peek_は他の誤りを無視し壊れた入力でもパニックしない() {
        let other_errors = "format_version = 9\n[pipeline]\nrun_id_env = \"X\"\nzzz = 1\n";
        assert_eq!(peek_run_id_env(other_errors), Some("X".to_string()));
        for bad in [
            "[pipeline\nrun_id_env",
            "= = =",
            "\u{0}",
            "a = [[[[",
            "[[pipeline]]\n",
        ] {
            assert_eq!(peek_run_id_env(bad), None, "{bad:?}");
        }
        let huge = format!("[pipeline]\nrun_id_env = \"{}\"\n", "A".repeat(2_000_000));
        assert!(peek_run_id_env(&huge).is_some());
        let deep = format!("a = {}1{}\n", "[".repeat(100_000), "]".repeat(100_000));
        assert_eq!(peek_run_id_env(&deep), None);
    }
}
