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

//! エラーの型と表示(Describe)、位置の変換(Locate)。
//!
//! 表示の文は、固定の文だけから組み立てる。ライブラリのエラー文、ソースの抜粋、
//! 誤った値は含めない(NFR4.6)。入力に由来するキー名は、制御文字をエスケープし、
//! 本文を 64 文字以内に切って表示する(切ったときは末尾に `…` を足す)。

use thiserror::Error;

/// キー名を表示するときの最大の文字数(NFR4.6)。
pub const MAX_KEY_DISPLAY_CHARS: usize = 64;

/// ファイル名を表示するときの最大の文字数(定義の誤りと `.env` の誤りで共通)。
pub(crate) const MAX_FILE_DISPLAY_CHARS: usize = 256;

/// 定義の誤りの種類(BR1.1〜BR1.13、security-design の追加分を含む)。
///
/// 種類の文(日本語)は [`DefinitionErrorKind::label`] の 1 か所に持ち、誤りの表示(`DefinitionError`)はそれを書く(D10)。
/// 仕様書の識別子(`kind`)は、変種の名前の先頭を小文字にしたもの(例: `FileMissing` は `fileMissing`)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefinitionErrorKind {
    FileMissing,
    Unreadable,
    TooLarge,
    Syntax,
    TypeMismatch,
    UnknownKey,
    VersionMissing,
    VersionUnsupported,
    UnknownTarget,
    ErrorTransition,
    BackwardOrSelfTarget,
    OverlappingExitCodes,
    ZeroAsWarning,
    PipelineNameMismatch,
    DuplicateJobId,
    InvalidJobId,
    NoJobs,
    EmptyField,
    DuplicateTransition,
    InvalidEnvName,
    /// `[job.retry]` が表でない(BR1.2)。
    RetryNotTable,
    /// `[job.retry]` の必須のキーがない(BR1.3)。
    RetryMissingKey,
    /// `retry.exit_codes` が空(BR1.5)。
    RetryExitCodesEmpty,
    /// `retry.exit_codes` に、整数でない値、または 1〜255 の範囲外の値がある(BR1.5)。
    RetryExitCodeInvalid,
    /// `retry.exit_codes` が、成功または警告の終了コードと重なっている(BR1.7)。
    RetryExitCodesOverlap,
    /// `retry.max` が 1〜100 の整数でない(BR1.9)。
    RetryMaxInvalid,
    /// `retry.interval` または `retry.max_interval` の書式や範囲が正しくない(BR1.10)。
    RetryDurationInvalid,
    /// `retry.multiplier` と `retry.max_interval` の片方だけがある(BR1.11)。
    RetryBackoffIncomplete,
    /// `retry.multiplier` が 2〜10 の整数でない(BR1.12)。
    RetryMultiplierInvalid,
    /// `retry.max_interval` が `retry.interval` より小さい(BR1.13)。
    RetryMaxIntervalTooSmall,
    /// `env` の名前が、予約名(`CONDUCTOR_` で始まる)(BR1.3)。
    ReservedEnvName,
    /// `env` の表の中に、大文字小文字だけが違う名前がある(BR1.4)。
    EnvNameCaseConflict,
    /// `env` の値に NUL が含まれる(BR1.5)。
    InvalidEnvValue,
    /// `unset_env` の名前が、同じジョブに当てはまる `env` の名前と重なる(U3 の BR2.1)。
    UnsetEnvConflict,
}

impl DefinitionErrorKind {
    /// 種類の文(日本語。表示と、cli の日本語の表示が使う。D10)。
    pub fn label(self) -> &'static str {
        match self {
            Self::FileMissing => "定義ファイルが存在しません",
            Self::Unreadable => "定義ファイルを読めません",
            Self::TooLarge => "定義ファイルが大きすぎます",
            Self::Syntax => "文法の誤りです",
            Self::TypeMismatch => "値の型が正しくありません",
            Self::UnknownKey => "定義にないキーです",
            Self::VersionMissing => "書式の版がありません",
            Self::VersionUnsupported => "対応していない書式の版です",
            Self::UnknownTarget => "遷移先のジョブが定義にありません",
            Self::ErrorTransition => "エラー結果には遷移先を書けません",
            Self::BackwardOrSelfTarget => "遷移先は、記述順で後ろのジョブまたは @end のみです",
            Self::OverlappingExitCodes => "成功と警告の終了コードが重なっています",
            Self::ZeroAsWarning => "0 は警告の終了コードにできません",
            Self::PipelineNameMismatch => "パイプライン名が定義と一致しません",
            Self::DuplicateJobId => "ジョブIDが重複しています",
            Self::InvalidJobId => "ジョブIDが正しくありません",
            Self::NoJobs => "ジョブが 1 つもありません",
            Self::EmptyField => "必須の値が空です",
            Self::DuplicateTransition => "同じ結果の遷移が重複しています",
            Self::InvalidEnvName => "環境変数の名前が正しくありません",
            Self::RetryNotTable => "retry は表で書きます",
            Self::RetryMissingKey => "retry の必須のキーがありません",
            Self::RetryExitCodesEmpty => "retry の終了コードの一覧が空です",
            Self::RetryExitCodeInvalid => "retry の終了コードが正しくありません",
            Self::RetryExitCodesOverlap => {
                "retry の終了コードが、成功または警告の終了コードと重なっています"
            }
            Self::RetryMaxInvalid => "retry の最大回数が正しくありません",
            Self::RetryDurationInvalid => "retry の間隔が正しくありません",
            Self::RetryBackoffIncomplete => {
                "retry の倍率と間隔の上限は、両方を書くか、両方とも書きません"
            }
            Self::RetryMultiplierInvalid => "retry の倍率が正しくありません",
            Self::RetryMaxIntervalTooSmall => "retry の間隔の上限が、間隔より小さくなっています",
            Self::ReservedEnvName => "予約された環境変数の名前です",
            Self::EnvNameCaseConflict => "大文字小文字だけが違う環境変数の名前があります",
            Self::InvalidEnvValue => "環境変数の値が正しくありません",
            Self::UnsetEnvConflict => "消す環境変数の名前が、足す環境変数の名前と重なっています",
        }
    }
}

/// 定義の誤りの報告(契約 C2 の `DefinitionError`)。1 回の検証で、複数を返してよい。
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{}", self.render())]
pub struct DefinitionError {
    /// 定義ファイルの名前(表示用)。
    pub file: String,
    /// 行番号(1 始まり)。位置が決まらないときは `None`。
    pub line: Option<usize>,
    pub kind: DefinitionErrorKind,
    /// 固定の文から組み立てた理由。入力の値は含まない(キー名は、エスケープと切り詰めを行う)。
    pub reason: String,
}

impl DefinitionError {
    /// 誤りを作る。`reason` は、呼び出し側が固定の文から組み立てたものを渡す。
    pub fn new(
        file: impl Into<String>,
        line: Option<usize>,
        kind: DefinitionErrorKind,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            file: file.into(),
            line,
            kind,
            reason: reason.into(),
        }
    }

    fn render(&self) -> String {
        render_error(&self.file, self.line, self.kind.label(), &self.reason)
    }
}

/// 誤りの表示の書式(`<ファイル>:<行>: <種類>: <理由>`。行が無ければ `<ファイル>: <種類>: <理由>`)。
/// ファイル名は表示用に整える(NFR4.2)。定義の誤りと `.env` の誤りが共有する。
pub(crate) fn render_error(file: &str, line: Option<usize>, kind: &str, reason: &str) -> String {
    let file = escape_display(file, Some(MAX_FILE_DISPLAY_CHARS));
    match line {
        Some(line) => format!("{file}:{line}: {kind}: {reason}"),
        None => format!("{file}: {kind}: {reason}"),
    }
}

/// 入力に由来する文字列を、表示用に整える(NFR4.2、NFR4.6)。表示のエスケープの本体
/// (conductor-cli の `report::escape_full`(`report/escape.rs`)が包む)。
///
/// - 制御文字(U+0000〜U+001F、U+007F〜U+009F)と、書式・双方向の制御文字(U+200B〜U+200F、
///   U+2028、U+2029、U+202A〜U+202E、U+2060〜U+2064、U+2066〜U+2069、U+FEFF)を、`\u{..}` にする。
///   改行・復帰・タブは、`\n`・`\r`・`\t`。
/// - `limit` があれば、本文はエスケープ後の文字数で `limit` 文字以内。切ったときは末尾に `…` を足す
///   (全体は最大で `limit + 1` 文字)。`None` は、切らない(パス。エスケープだけ)。
pub fn escape_display(input: &str, limit: Option<usize>) -> String {
    let mut out = String::with_capacity(input.len());
    let mut count = 0usize;
    for c in input.chars() {
        let piece = escape_char(c);
        let len = piece.chars().count();
        if limit.is_some_and(|max| count + len > max) {
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

/// キー名を、エラーの理由に入れる形(引用符つき、エスケープと 64 文字の切り詰め)にする。
pub(crate) fn quote_key(key: &str) -> String {
    format!("`{}`", escape_display(key, Some(MAX_KEY_DISPLAY_CHARS)))
}

/// バイトの位置から、行番号(1 始まり)を求める(Locate)。
///
/// 改行は LF と CRLF のどちらも、LF の数で数える。
#[derive(Debug, Clone)]
pub(crate) struct Locate {
    /// 各行の先頭のバイトの位置。
    line_starts: Vec<usize>,
}

impl Locate {
    pub fn new(text: &str) -> Self {
        let mut line_starts = vec![0];
        line_starts.extend(
            text.bytes()
                .enumerate()
                .filter(|(_, b)| *b == b'\n')
                .map(|(i, _)| i + 1),
        );
        Self { line_starts }
    }

    /// バイトの位置の行番号(1 始まり)。範囲外の位置は、最後の行として扱う。
    pub fn line_of(&self, offset: usize) -> usize {
        match self.line_starts.binary_search(&offset) {
            Ok(i) => i + 1,
            Err(i) => i,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 書式文字と双方向制御文字もエスケープする() {
        for c in [
            '\u{200B}', '\u{200F}', '\u{2028}', '\u{2029}', '\u{202A}', '\u{202E}', '\u{2066}',
            '\u{2069}', '\u{FEFF}',
        ] {
            let out = escape_display(&format!("a{c}b"), Some(64));
            assert_eq!(out, format!("a\\u{{{:x}}}b", c as u32));
            assert!(!out.contains(c));
        }
        assert_eq!(escape_display("日本語 abc", Some(64)), "日本語 abc");
    }

    #[test]
    fn 行番号_lf() {
        let l = Locate::new("a\nb\nc");
        assert_eq!(l.line_of(0), 1);
        assert_eq!(l.line_of(1), 1);
        assert_eq!(l.line_of(2), 2);
        assert_eq!(l.line_of(4), 3);
        assert_eq!(l.line_of(5), 3); // 末尾
        assert_eq!(l.line_of(999), 3); // 範囲外
    }

    #[test]
    fn 行番号_crlf() {
        let l = Locate::new("a\r\nb\r\nc");
        assert_eq!(l.line_of(0), 1);
        assert_eq!(l.line_of(1), 1); // CR
        assert_eq!(l.line_of(3), 2);
        assert_eq!(l.line_of(6), 3);
    }

    #[test]
    fn 行番号_空のテキストと末尾の改行() {
        assert_eq!(Locate::new("").line_of(0), 1);
        let l = Locate::new("a\n");
        assert_eq!(l.line_of(2), 2);
    }

    #[test]
    fn キー名の制御文字はエスケープされる() {
        assert_eq!(escape_display("a\nb\x1b[0m", Some(64)), "a\\nb\\u{1b}[0m");
        assert_eq!(quote_key("k\tx"), "`k\\tx`");
    }

    #[test]
    fn キー名は_64_文字で切られる() {
        let long = "k".repeat(200);
        let s = escape_display(&long, Some(MAX_KEY_DISPLAY_CHARS));
        assert_eq!(s.chars().count(), MAX_KEY_DISPLAY_CHARS + 1);
        assert!(s.ends_with('…'));
        // ちょうど 64 文字は切らない
        let exact = "k".repeat(64);
        assert_eq!(escape_display(&exact, Some(64)), exact);
    }

    #[test]
    fn エスケープ後も_64_文字を超えない() {
        // 名前の「64 文字」は、`…` を除いた本文の文字数。
        let s = escape_display(&"\n".repeat(100), Some(64));
        assert_eq!(s.chars().count(), 64 + 1);
    }

    #[test]
    fn 表示は_ファイル_行_種類_理由() {
        let e = DefinitionError::new(
            "conductor.toml",
            Some(7),
            DefinitionErrorKind::UnknownKey,
            "キー `x` は定義にありません",
        );
        assert_eq!(
            e.to_string(),
            "conductor.toml:7: 定義にないキーです: キー `x` は定義にありません"
        );
        let e = DefinitionError::new("c.toml", None, DefinitionErrorKind::NoJobs, "理由");
        assert_eq!(e.to_string(), "c.toml: ジョブが 1 つもありません: 理由");
    }

    #[test]
    fn ファイル名の制御文字もエスケープされる() {
        let e = DefinitionError::new("a\nb.toml", None, DefinitionErrorKind::FileMissing, "r");
        assert!(!e.to_string().contains('\n'));
    }
}
