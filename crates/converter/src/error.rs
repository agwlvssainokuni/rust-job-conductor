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

//! エラーの型と表示(Describe)。
//!
//! 表示の文は、kind ごとの固定の文と、ファイル名・行・キーの経路だけから組み立てる。
//! ライブラリのエラー文、ソースの抜粋、入力の値は含めない(NFR4.6、BR1.10)。
//! 入力に由来するキー名は、制御文字と書式・双方向の制御文字をエスケープし、64 文字までに切る。
//!
//! # `key_path` の書式
//!
//! 最上位のキーは `jobs`、入れ子のキーは `.` で、配列の要素は `[添字]`(0 始まり)でつなぐ
//! (例: `jobs[2].transitions[0]`)。キーが文字列でないときは、そのマップの経路に
//! `(非文字列キー)` を付ける(最上位なら `(非文字列キー)`、入れ子なら `jobs[2].(非文字列キー)`)。
//! 各キー名は、エスケープと 64 文字の切り詰めを行ってから、そのまま(引用符なしで)つなぐ。

use thiserror::Error;

/// キー名を表示するときの最大の文字数(NFR4.6)。
pub(crate) const MAX_KEY_DISPLAY_CHARS: usize = 64;

/// ファイル名を表示するときの最大の文字数。
const MAX_FILE_DISPLAY_CHARS: usize = 256;

/// 変換の失敗の種類(entities の `kind`)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Error)]
pub enum ConversionErrorKind {
    #[error("入力ファイルが存在しません")]
    FileMissing,
    #[error("入力ファイルを読めません")]
    Unreadable,
    #[error("入力が大きすぎます")]
    TooLarge,
    #[error("文法の誤りです")]
    Syntax,
    #[error("相手の形式で表せない値があります")]
    Unrepresentable,
    #[error("最上位がキーと値の集まり(マップ)ではありません")]
    RootNotMap,
}

impl ConversionErrorKind {
    /// 種類を表す識別子(機械が読む用。entities.md の `kind` の名前)。契約 C4 の外(テスト用)。
    #[doc(hidden)]
    pub fn code(self) -> &'static str {
        match self {
            Self::FileMissing => "fileMissing",
            Self::Unreadable => "unreadable",
            Self::TooLarge => "tooLarge",
            Self::Syntax => "syntax",
            Self::Unrepresentable => "unrepresentable",
            Self::RootNotMap => "rootNotMap",
        }
    }

    /// すべての種類(網羅のテストに使う)。契約 C4 の外(テスト用)。
    #[doc(hidden)]
    pub const ALL: [ConversionErrorKind; 6] = [
        Self::FileMissing,
        Self::Unreadable,
        Self::TooLarge,
        Self::Syntax,
        Self::Unrepresentable,
        Self::RootNotMap,
    ];
}

/// 変換の失敗の報告(契約 C4 の failure)。変換は行われず、出力は作られない。
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{}", self.render())]
pub struct ConversionError {
    /// 入力ファイルの名前(表示用)。
    pub file: String,
    /// 行番号(1 始まり)。位置が分かるときのみ。
    pub line: Option<usize>,
    /// 表せない値などの場所(キーの経路)。値そのものは含まない。
    pub key_path: Option<String>,
    pub kind: ConversionErrorKind,
    /// 固定の文。入力の値と、ライブラリのエラー文は含まない。
    pub reason: String,
}

impl ConversionError {
    pub(crate) fn new(
        file: &str,
        line: Option<usize>,
        key_path: Option<String>,
        kind: ConversionErrorKind,
        reason: &str,
    ) -> Self {
        Self {
            file: file.to_string(),
            line,
            key_path,
            kind,
            reason: reason.to_string(),
        }
    }

    fn render(&self) -> String {
        let file = sanitize(&self.file, MAX_FILE_DISPLAY_CHARS);
        let mut out = match self.line {
            Some(line) => format!("{file}:{line}: {}: {}", self.kind, self.reason),
            None => format!("{file}: {}: {}", self.kind, self.reason),
        };
        if let Some(path) = &self.key_path {
            out.push_str(&format!("(場所: {path})"));
        }
        out
    }
}

/// 固定の理由の文(入力の値を含まない)。
pub(crate) mod reason {
    pub const FILE_MISSING: &str = "指定の場所にファイルがありません";
    pub const PERMISSION: &str = "読み取りの権限がありません";
    pub const READ_FAILED: &str = "読み取りに失敗しました";
    pub const NOT_REGULAR: &str = "通常のファイルではありません";
    pub const NOT_UTF8: &str = "UTF-8 として読めません";
    pub const INPUT_TOO_LARGE: &str = "入力が上限(1 MiB)を超えています";
    pub const EXPANDED_TOO_LARGE: &str = "展開後の要素数が上限(100000)を超えています";
    pub const EXPANDED_BYTES_TOO_LARGE: &str = "展開後の文字の大きさが上限(16 MiB)を超えています";
    pub const SYNTAX: &str = "文法の誤りです";
    pub const DUPLICATE_KEY: &str = "キーが重複しています";
    pub const RECURSIVE_ANCHOR: &str = "アンカーが自分自身を含んでいます";
    pub const UNKNOWN_ANCHOR: &str = "定義されていないアンカーを参照しています";
    pub const TOO_DEEP: &str = "入れ子が深すぎます(上限 32)";
    pub const TOO_DEEP_EXPANDED: &str = "展開後の入れ子が深すぎます(上限 32)";
    pub const MULTI_DOCUMENT: &str = "複数の文書は表せません";
    pub const NULL_VALUE: &str = "null は表せません";
    pub const NON_STRING_KEY: &str = "文字列でないキーは表せません";
    pub const TAGGED: &str = "タグつきの値は表せません";
    pub const DATETIME: &str = "日時の値は表せません";
    pub const NUMBER_RANGE: &str = "数値が表せる範囲を超えています";
    pub const MERGE_NOT_MAP: &str = "マージキーの値がマップではありません";
    pub const ROOT_NOT_MAP: &str = "最上位がマップではありません";
    pub const EMPTY_DOCUMENT: &str = "文書が空です";
    pub const WRITE_FAILED: &str = "変換後の形式として書き出せません";
}

/// 入力に由来する文字列を、表示用に整える。制御文字をエスケープし、`max_chars` 文字で切る。
///
/// 出力は、エスケープ後の文字数で `max_chars` 文字までに収め、切ったときは末尾に `…` を付ける。
pub(crate) fn sanitize(input: &str, max_chars: usize) -> String {
    let mut out = String::new();
    let mut count = 0usize;
    for c in input.chars() {
        let piece = escape_char(c);
        let len = piece.chars().count();
        if count + len > max_chars {
            out.push('…');
            return out;
        }
        out.push_str(&piece);
        count += len;
    }
    out
}

fn escape_char(c: char) -> String {
    match c {
        '\n' => "\\n".to_string(),
        '\r' => "\\r".to_string(),
        '\t' => "\\t".to_string(),
        c if c.is_control() || is_format_char(c) => format!("\\u{{{:x}}}", c as u32),
        c => c.to_string(),
    }
}

/// 表示を乱す書式文字・双方向制御文字(ゼロ幅、方向指定、行・段落区切り、BOM など)。
fn is_format_char(c: char) -> bool {
    matches!(
        c,
        '\u{200B}'..='\u{200F}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2060}'..='\u{2064}'
            | '\u{2066}'..='\u{2069}'
            | '\u{FEFF}'
    )
}

/// キーの経路の1区間。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PathSeg {
    Key(String),
    Index(usize),
    NonStringKey,
}

/// 経路を、`key_path` の書式の文字列にする。経路が空のときは `None`。
pub(crate) fn format_key_path(segs: &[PathSeg]) -> Option<String> {
    if segs.is_empty() {
        return None;
    }
    let mut out = String::new();
    for seg in segs {
        match seg {
            PathSeg::Key(k) => {
                if !out.is_empty() {
                    out.push('.');
                }
                out.push_str(&sanitize(k, MAX_KEY_DISPLAY_CHARS));
            }
            PathSeg::Index(i) => out.push_str(&format!("[{i}]")),
            PathSeg::NonStringKey => {
                if !out.is_empty() {
                    out.push('.');
                }
                out.push_str("(非文字列キー)");
            }
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_has_no_input_values_for_every_kind() {
        for kind in ConversionErrorKind::ALL {
            let e = ConversionError::new("in.yaml", Some(3), None, kind, "固定の文");
            let s = e.to_string();
            assert!(s.contains("in.yaml:3:"), "{s}");
            assert!(s.contains("固定の文"), "{s}");
            assert!(!kind.code().is_empty());
        }
    }

    #[test]
    fn display_without_line_and_with_key_path() {
        let e = ConversionError::new(
            "in.toml",
            None,
            Some("a.b".into()),
            ConversionErrorKind::Unrepresentable,
            "固定の文",
        );
        let s = e.to_string();
        assert!(s.starts_with("in.toml: "), "{s}");
        assert!(s.ends_with("(場所: a.b)"), "{s}");
    }

    #[test]
    fn sanitize_escapes_control_and_format_chars() {
        assert_eq!(sanitize("a\nb\tc\rd", 64), "a\\nb\\tc\\rd");
        assert_eq!(sanitize("a\u{1b}[31m", 64), "a\\u{1b}[31m");
        assert_eq!(
            sanitize("x\u{202e}y\u{200b}z\u{feff}", 64),
            "x\\u{202e}y\\u{200b}z\\u{feff}"
        );
        assert_eq!(sanitize("日本語", 64), "日本語");
    }

    #[test]
    fn sanitize_truncates_to_64_chars() {
        let long = "k".repeat(100);
        let s = sanitize(&long, MAX_KEY_DISPLAY_CHARS);
        assert_eq!(s.chars().count(), 65);
        assert!(s.ends_with('…'));
        assert_eq!(sanitize(&"k".repeat(64), 64), "k".repeat(64));
        // エスケープ後の長さで数える(切れ目で、エスケープを分けない)
        let s = sanitize(&"\n".repeat(40), 64);
        assert_eq!(s, format!("{}…", "\\n".repeat(32)));
    }

    #[test]
    fn key_path_format() {
        let segs = vec![
            PathSeg::Key("jobs".into()),
            PathSeg::Index(2),
            PathSeg::Key("transitions".into()),
            PathSeg::Index(0),
        ];
        assert_eq!(
            format_key_path(&segs).as_deref(),
            Some("jobs[2].transitions[0]")
        );
        assert_eq!(format_key_path(&[]), None);
        assert_eq!(
            format_key_path(&[PathSeg::NonStringKey]).as_deref(),
            Some("(非文字列キー)")
        );
        assert_eq!(
            format_key_path(&[PathSeg::Key("a".into()), PathSeg::NonStringKey]).as_deref(),
            Some("a.(非文字列キー)")
        );
        assert_eq!(
            format_key_path(&[PathSeg::Key("a\nb".into())]).as_deref(),
            Some("a\\nb")
        );
    }
}
