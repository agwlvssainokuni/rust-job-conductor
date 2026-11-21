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
//! 64 文字までに切って表示する。

use thiserror::Error;

/// キー名を表示するときの最大の文字数(NFR4.6)。
pub const MAX_KEY_DISPLAY_CHARS: usize = 64;

/// ファイル名を表示するときの最大の文字数。
const MAX_FILE_DISPLAY_CHARS: usize = 256;

/// 定義の誤りの種類(BR1.1〜BR1.13、security-design の追加分を含む)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Error)]
pub enum DefinitionErrorKind {
    #[error("定義ファイルが存在しません")]
    FileMissing,
    #[error("定義ファイルを読めません")]
    Unreadable,
    #[error("定義ファイルが大きすぎます")]
    TooLarge,
    #[error("文法の誤りです")]
    Syntax,
    #[error("値の型が正しくありません")]
    TypeMismatch,
    #[error("定義にないキーです")]
    UnknownKey,
    #[error("書式の版がありません")]
    VersionMissing,
    #[error("対応していない書式の版です")]
    VersionUnsupported,
    #[error("遷移先のジョブが定義にありません")]
    UnknownTarget,
    #[error("エラー結果には遷移先を書けません")]
    ErrorTransition,
    #[error("遷移先は、記述順で後ろのジョブまたは @end のみです")]
    BackwardOrSelfTarget,
    #[error("成功と警告の終了コードが重なっています")]
    OverlappingExitCodes,
    #[error("0 は警告の終了コードにできません")]
    ZeroAsWarning,
    #[error("パイプライン名が定義と一致しません")]
    PipelineNameMismatch,
    #[error("ジョブIDが重複しています")]
    DuplicateJobId,
    #[error("ジョブIDが正しくありません")]
    InvalidJobId,
    #[error("ジョブが 1 つもありません")]
    NoJobs,
    #[error("必須の値が空です")]
    EmptyField,
    #[error("同じ結果の遷移が重複しています")]
    DuplicateTransition,
    #[error("環境変数の名前が正しくありません")]
    InvalidEnvName,
    /// `[job.retry]` が表でない(BR1.2)。
    #[error("retry は表で書きます")]
    RetryNotTable,
    /// `[job.retry]` の必須のキーがない(BR1.3)。
    #[error("retry の必須のキーがありません")]
    RetryMissingKey,
    /// `retry.exit_codes` が空(BR1.5)。
    #[error("retry の終了コードの一覧が空です")]
    RetryExitCodesEmpty,
    /// `retry.exit_codes` に、整数でない値、または 1〜255 の範囲外の値がある(BR1.5)。
    #[error("retry の終了コードが正しくありません")]
    RetryExitCodeInvalid,
    /// `retry.exit_codes` が、成功または警告の終了コードと重なっている(BR1.7)。
    #[error("retry の終了コードが、成功または警告の終了コードと重なっています")]
    RetryExitCodesOverlap,
    /// `retry.max` が 1〜100 の整数でない(BR1.9)。
    #[error("retry の最大回数が正しくありません")]
    RetryMaxInvalid,
    /// `retry.interval` または `retry.max_interval` の書式や範囲が正しくない(BR1.10)。
    #[error("retry の間隔が正しくありません")]
    RetryDurationInvalid,
    /// `retry.multiplier` と `retry.max_interval` の片方だけがある(BR1.11)。
    #[error("retry の倍率と間隔の上限は、両方を書くか、両方とも書きません")]
    RetryBackoffIncomplete,
    /// `retry.multiplier` が 2〜10 の整数でない(BR1.12)。
    #[error("retry の倍率が正しくありません")]
    RetryMultiplierInvalid,
    /// `retry.max_interval` が `retry.interval` より小さい(BR1.13)。
    #[error("retry の間隔の上限が、間隔より小さくなっています")]
    RetryMaxIntervalTooSmall,
    /// `env` の名前が、予約名(`CONDUCTOR_` で始まる)(BR1.3)。
    #[error("予約された環境変数の名前です")]
    ReservedEnvName,
    /// `env` の表の中に、大文字小文字だけが違う名前がある(BR1.4)。
    #[error("大文字小文字だけが違う環境変数の名前があります")]
    EnvNameCaseConflict,
    /// `env` の値に NUL が含まれる(BR1.5)。
    #[error("環境変数の値が正しくありません")]
    InvalidEnvValue,
}

impl DefinitionErrorKind {
    /// 種類を表す識別子(機械が読む用。entities.md の `kind` の名前)。
    pub fn code(self) -> &'static str {
        match self {
            Self::FileMissing => "fileMissing",
            Self::Unreadable => "unreadable",
            Self::TooLarge => "tooLarge",
            Self::Syntax => "syntax",
            Self::TypeMismatch => "typeMismatch",
            Self::UnknownKey => "unknownKey",
            Self::VersionMissing => "versionMissing",
            Self::VersionUnsupported => "versionUnsupported",
            Self::UnknownTarget => "unknownTarget",
            Self::ErrorTransition => "errorTransition",
            Self::BackwardOrSelfTarget => "backwardOrSelfTarget",
            Self::OverlappingExitCodes => "overlappingExitCodes",
            Self::ZeroAsWarning => "zeroAsWarning",
            Self::PipelineNameMismatch => "pipelineNameMismatch",
            Self::DuplicateJobId => "duplicateJobId",
            Self::InvalidJobId => "invalidJobId",
            Self::NoJobs => "noJobs",
            Self::EmptyField => "emptyField",
            Self::DuplicateTransition => "duplicateTransition",
            Self::InvalidEnvName => "invalidEnvName",
            Self::RetryNotTable => "retryNotTable",
            Self::RetryMissingKey => "retryMissingKey",
            Self::RetryExitCodesEmpty => "retryExitCodesEmpty",
            Self::RetryExitCodeInvalid => "retryExitCodeInvalid",
            Self::RetryExitCodesOverlap => "retryExitCodesOverlap",
            Self::RetryMaxInvalid => "retryMaxInvalid",
            Self::RetryDurationInvalid => "retryDurationInvalid",
            Self::RetryBackoffIncomplete => "retryBackoffIncomplete",
            Self::RetryMultiplierInvalid => "retryMultiplierInvalid",
            Self::RetryMaxIntervalTooSmall => "retryMaxIntervalTooSmall",
            Self::ReservedEnvName => "reservedEnvName",
            Self::EnvNameCaseConflict => "envNameCaseConflict",
            Self::InvalidEnvValue => "invalidEnvValue",
        }
    }

    /// すべての種類(網羅のテストと、表示の確認に使う)。
    pub const ALL: [DefinitionErrorKind; 33] = [
        Self::FileMissing,
        Self::Unreadable,
        Self::TooLarge,
        Self::Syntax,
        Self::TypeMismatch,
        Self::UnknownKey,
        Self::VersionMissing,
        Self::VersionUnsupported,
        Self::UnknownTarget,
        Self::ErrorTransition,
        Self::BackwardOrSelfTarget,
        Self::OverlappingExitCodes,
        Self::ZeroAsWarning,
        Self::PipelineNameMismatch,
        Self::DuplicateJobId,
        Self::InvalidJobId,
        Self::NoJobs,
        Self::EmptyField,
        Self::DuplicateTransition,
        Self::InvalidEnvName,
        Self::RetryNotTable,
        Self::RetryMissingKey,
        Self::RetryExitCodesEmpty,
        Self::RetryExitCodeInvalid,
        Self::RetryExitCodesOverlap,
        Self::RetryMaxInvalid,
        Self::RetryDurationInvalid,
        Self::RetryBackoffIncomplete,
        Self::RetryMultiplierInvalid,
        Self::RetryMaxIntervalTooSmall,
        Self::ReservedEnvName,
        Self::EnvNameCaseConflict,
        Self::InvalidEnvValue,
    ];
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
        let file = sanitize(&self.file, MAX_FILE_DISPLAY_CHARS);
        match self.line {
            Some(line) => format!("{file}:{line}: {}: {}", self.kind, self.reason),
            None => format!("{file}: {}: {}", self.kind, self.reason),
        }
    }
}

/// 入力に由来する文字列を、表示用に整える。制御文字をエスケープし、`max_chars` 文字で切る。
///
/// 出力は、エスケープ後の文字数で `max_chars` 文字までに収め、切ったときは末尾に `…` を付ける。
pub fn sanitize(input: &str, max_chars: usize) -> String {
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

/// キー名を、エラーの理由に入れる形(引用符つき、エスケープと 64 文字の切り詰め)にする。
pub fn quote_key(key: &str) -> String {
    format!("`{}`", sanitize(key, MAX_KEY_DISPLAY_CHARS))
}

/// バイトの位置から、行番号(1 始まり)を求める(Locate)。
///
/// 改行は LF と CRLF のどちらも、LF の数で数える。
#[derive(Debug, Clone)]
pub struct Locate {
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
            let out = sanitize(&format!("a{c}b"), 64);
            assert_eq!(out, format!("a\\u{{{:x}}}b", c as u32));
            assert!(!out.contains(c));
        }
        assert_eq!(sanitize("日本語 abc", 64), "日本語 abc");
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
        assert_eq!(sanitize("a\nb\x1b[0m", 64), "a\\nb\\u{1b}[0m");
        assert_eq!(quote_key("k\tx"), "`k\\tx`");
    }

    #[test]
    fn キー名は_64_文字で切られる() {
        let long = "k".repeat(200);
        let s = sanitize(&long, MAX_KEY_DISPLAY_CHARS);
        assert_eq!(s.chars().count(), MAX_KEY_DISPLAY_CHARS + 1);
        assert!(s.ends_with('…'));
        // ちょうど 64 文字は切らない
        let exact = "k".repeat(64);
        assert_eq!(sanitize(&exact, 64), exact);
    }

    #[test]
    fn エスケープ後も_64_文字を超えない() {
        let s = sanitize(&"\n".repeat(100), 64);
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
    fn 全種類に識別子と表示があり重複しない() {
        let mut codes: Vec<_> = DefinitionErrorKind::ALL.iter().map(|k| k.code()).collect();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), 33);
        for k in DefinitionErrorKind::ALL {
            assert!(!k.to_string().is_empty());
        }
    }

    #[test]
    fn ファイル名の制御文字もエスケープされる() {
        let e = DefinitionError::new("a\nb.toml", None, DefinitionErrorKind::FileMissing, "r");
        assert!(!e.to_string().contains('\n'));
    }
}
