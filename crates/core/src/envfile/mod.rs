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

//! 環境変数の注入(`env`、`env_file`、`.env`)の型と純粋な処理(CoreEnvFile。U1)。
//!
//! - `parse`: `.env` の文字列を解析する(WF2。BR2.x、BR3.2)。
//! - `merge`: 層を統合して、ジョブに足す環境を作る(WF4。BR4.x、BR5.x)。
//!
//! ファイルにも環境変数にも触れない。値を持つ型(`EnvEntry`、`EffectiveEntry`)の `Debug` は、
//! 値を伏せ字にする。誤りの理由は固定の文だけで、入力の値を含めない(BR8.1)。

pub mod merge;
pub mod parse;

use std::collections::HashMap;
use std::fmt;

use thiserror::Error;

use crate::error::sanitize;
use crate::types::PipelineDefinition;

pub use merge::merge_env;
pub use parse::parse_env_file;

/// `.env` の大きさの上限(1 MiB。BR3.1)。
pub const MAX_ENV_FILE_BYTES: usize = 1_048_576;
/// 1 つの `.env` の項目の数の上限(BR3.2)。
pub const MAX_ENV_ENTRIES: usize = 10_000;
/// 誤りを表示する件数の上限(BR2.8)。
pub const MAX_REPORTED_ERRORS: usize = 20;
/// 予約名の接頭辞(BR1.3)。
pub const RESERVED_ENV_PREFIX: &str = "CONDUCTOR_";

/// ファイル名を表示するときの最大の文字数。
const MAX_FILE_DISPLAY_CHARS: usize = 256;

/// 予約名(ASCII の大文字にして `CONDUCTOR_` で始まる名前)か(BR1.3)。
pub fn is_reserved_env_name(name: &str) -> bool {
    name.len() >= RESERVED_ENV_PREFIX.len()
        && name.as_bytes()[..RESERVED_ENV_PREFIX.len()]
            .eq_ignore_ascii_case(RESERVED_ENV_PREFIX.as_bytes())
}

/// `.env` の読み取りの誤りの種類(13 種)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Error)]
pub enum EnvFileErrorKind {
    #[error("環境変数ファイルが存在しません")]
    FileMissing,
    #[error("環境変数ファイルを読めません")]
    Unreadable,
    #[error("環境変数ファイルが大きすぎます")]
    TooLarge,
    #[error("環境変数ファイルが UTF-8 ではありません")]
    NotUtf8,
    #[error("環境変数ファイルの項目が多すぎます")]
    TooManyEntries,
    #[error("環境変数の名前が正しくありません")]
    InvalidName,
    #[error("予約された環境変数の名前です")]
    ReservedName,
    #[error("大文字小文字だけが違う環境変数の名前があります")]
    CaseConflict,
    #[error("`KEY=VALUE` の形ではありません")]
    MissingEquals,
    #[error("引用符が閉じていません")]
    UnterminatedQuote,
    #[error("許されないエスケープです")]
    InvalidEscape,
    #[error("閉じた引用符のあとに余計な文字があります")]
    TrailingGarbage,
    #[error("値に NUL が含まれています")]
    NulInValue,
}

impl EnvFileErrorKind {
    /// 種類を表す識別子(機械が読む用)。
    pub fn code(self) -> &'static str {
        match self {
            Self::FileMissing => "fileMissing",
            Self::Unreadable => "unreadable",
            Self::TooLarge => "tooLarge",
            Self::NotUtf8 => "notUtf8",
            Self::TooManyEntries => "tooManyEntries",
            Self::InvalidName => "invalidName",
            Self::ReservedName => "reservedName",
            Self::CaseConflict => "caseConflict",
            Self::MissingEquals => "missingEquals",
            Self::UnterminatedQuote => "unterminatedQuote",
            Self::InvalidEscape => "invalidEscape",
            Self::TrailingGarbage => "trailingGarbage",
            Self::NulInValue => "nulInValue",
        }
    }

    /// すべての種類(網羅のテストに使う)。
    pub const ALL: [EnvFileErrorKind; 13] = [
        Self::FileMissing,
        Self::Unreadable,
        Self::TooLarge,
        Self::NotUtf8,
        Self::TooManyEntries,
        Self::InvalidName,
        Self::ReservedName,
        Self::CaseConflict,
        Self::MissingEquals,
        Self::UnterminatedQuote,
        Self::InvalidEscape,
        Self::TrailingGarbage,
        Self::NulInValue,
    ];

    /// 固定の理由の文(入力の値を含まない)。
    pub fn reason(self) -> &'static str {
        match self {
            Self::FileMissing => "`env_file` に書かれたファイルが、見つかりません",
            Self::Unreadable => "`env_file` に書かれたファイルを、読めません",
            Self::TooLarge => "ファイルの大きさが、上限(1 MiB)を超えています",
            Self::NotUtf8 => "ファイルは UTF-8 で書きます",
            Self::TooManyEntries => "項目の数が、上限(10,000)を超えています",
            Self::InvalidName => "名前は、英字または `_` で始まり、英数字と `_` だけで書きます",
            Self::ReservedName => "`CONDUCTOR_` で始まる名前は、書けません",
            Self::CaseConflict => "大文字小文字だけが違う名前が、同じファイルにあります",
            Self::MissingEquals => "`KEY=VALUE` の形で書きます",
            Self::UnterminatedQuote => "引用符が、同じ行で閉じていません",
            Self::InvalidEscape => {
                "二重引用符の中で使えるエスケープは、`\\n`、`\\r`、`\\t`、`\\\"`、`\\\\` だけです"
            }
            Self::TrailingGarbage => "閉じた引用符のあとは、空白とコメントだけを書けます",
            Self::NulInValue => "値に NUL(U+0000)は書けません",
        }
    }
}

/// `.env` の読み取りの誤り(契約 C1 の errors)。値を含まない。
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{}", self.render())]
pub struct EnvFileError {
    /// ファイルの名前(表示用)。
    pub file: String,
    /// 行番号(1 始まり)。位置が決まらない誤り(無い、大きすぎる)は `None`。
    pub line: Option<usize>,
    pub kind: EnvFileErrorKind,
    /// 固定の文から作った理由。入力の値を含まない。
    pub reason: String,
}

impl EnvFileError {
    /// 種類の固定の理由で、誤りを作る。
    pub fn new(file: impl Into<String>, line: Option<usize>, kind: EnvFileErrorKind) -> Self {
        Self {
            file: file.into(),
            line,
            kind,
            reason: kind.reason().to_string(),
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

/// 誤りの並びを、表示する先頭の最大 20 件と、残りの件数に分ける(BR2.8)。
pub fn summarize_errors(errors: &[EnvFileError]) -> (&[EnvFileError], usize) {
    let shown = errors.len().min(MAX_REPORTED_ERRORS);
    (&errors[..shown], errors.len() - shown)
}

/// 警告の種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvWarningKind {
    /// 同じファイルの中で、同じ名前が 2 回以上ある(BR2.7)。
    DuplicateKey,
}

/// 誤りではないが、知らせる事実(BR2.7)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvWarning {
    pub file: String,
    /// 後ろの(上書きする側の)項目の行。
    pub line: usize,
    /// 重複した名前(表示してよい)。
    pub name: String,
    pub kind: EnvWarningKind,
}

/// `.env` の 1 項目。値は秘密で、`Debug` は伏せ字にする(BR8.1)。
#[derive(Clone, PartialEq, Eq)]
pub struct EnvEntry {
    pub name: String,
    pub value: String,
    /// 項目を書いた行(1 始まり)。
    pub line: usize,
}

impl fmt::Debug for EnvEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EnvEntry")
            .field("name", &self.name)
            .field("value", &"***")
            .field("line", &self.line)
            .finish()
    }
}

/// 1 つの `.env` を読み取った結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvFileContent {
    /// 解決済みのパス(表示のときは、エスケープと切り詰めをする)。
    pub path: String,
    /// 書いた順。同じ名前の重複は、後ろの項目だけが、末尾側に残る。
    pub entries: Vec<EnvEntry>,
    pub warnings: Vec<EnvWarning>,
}

/// 1 回の呼び出しで読み取った、すべての `.env` の結果(BR6.4)。
///
/// 定義に書かれた `env_file` の文字列から、読み取った内容を引ける。同じ解決済みのパスは、
/// 1 つの内容を共有する。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LoadedEnvFiles {
    /// 解決済みのパスごとの内容。読み取った順。
    files: Vec<EnvFileContent>,
    /// 定義に書かれた文字列 → `files` の位置。
    by_ref: HashMap<String, usize>,
}

impl LoadedEnvFiles {
    /// 何も読んでいない状態。
    pub fn new() -> Self {
        Self::default()
    }

    /// 解決済みのパスの内容を足し、その位置を返す。
    pub fn push_file(&mut self, content: EnvFileContent) -> usize {
        self.files.push(content);
        self.files.len() - 1
    }

    /// 定義に書かれた文字列が、`files` の位置の内容を指すと記録する。
    pub fn link(&mut self, reference: impl Into<String>, index: usize) {
        self.by_ref.insert(reference.into(), index);
    }

    /// 内容を足し、同じ文字列から引けるようにする(偽の実装やテストの便宜)。
    pub fn add(&mut self, reference: impl Into<String>, content: EnvFileContent) {
        let index = self.push_file(content);
        self.link(reference, index);
    }

    /// 定義に書かれた文字列から、読み取った内容を引く。
    pub fn get(&self, reference: &str) -> Option<&EnvFileContent> {
        self.by_ref.get(reference).and_then(|&i| self.files.get(i))
    }

    /// 読み取った内容(解決済みのパスごと。読み取った順)。
    pub fn files(&self) -> &[EnvFileContent] {
        &self.files
    }
}

/// 自動の変数(`CONDUCTOR_*`)を決める入力(BR5.x)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnvContext<'a> {
    pub pipeline_name: &'a str,
    pub job_id: &'a str,
    /// 無いときは `CONDUCTOR_RUN_ID` を作らない(BR5.3)。
    pub run_id: Option<&'a str>,
    /// 1 から始まる試行の番号(BR5.2)。
    pub attempt: u32,
}

/// 名前の出どころ(契約 C1 の origin)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvOrigin {
    PipelineEnvFile,
    PipelineEnv,
    JobEnvFile,
    JobEnv,
    Builtin,
}

/// 足す環境の 1 項目。値は秘密で、`Debug` は伏せ字にする(BR8.1)。
#[derive(Clone, PartialEq, Eq)]
pub struct EffectiveEntry {
    pub name: String,
    pub value: String,
    pub origin: EnvOrigin,
}

impl fmt::Debug for EffectiveEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EffectiveEntry")
            .field("name", &self.name)
            .field("value", &"***")
            .field("origin", &self.origin)
            .finish()
    }
}

/// 1 つのジョブの 1 回の起動に、引き継いだ環境へ「足す」環境変数(層を統合した結果)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveEnv {
    pub job_id: String,
    /// 層の順。完全に同じ名前は 1 つ(後ろの層が勝つ)。
    pub entries: Vec<EffectiveEntry>,
}

impl EffectiveEnv {
    /// 起動の口へ渡す、足す環境(名前と値)。並びの順に足す(BR4.3)。
    pub fn additions(&self) -> Vec<(&str, &str)> {
        self.entries
            .iter()
            .map(|e| (e.name.as_str(), e.value.as_str()))
            .collect()
    }

    /// 名前だけの要約(値を含まない。引き継いだ環境の名前を含まない。BR8.2)。
    pub fn names(&self) -> Vec<(&str, EnvOrigin)> {
        self.entries
            .iter()
            .map(|e| (e.name.as_str(), e.origin))
            .collect()
    }
}

/// 読む対象の `env_file` の記述の一覧を作る(BR6.1)。
///
/// `job_id` が `None` なら `[pipeline].env_file` とすべてのジョブの `env_file`(`pipeline`、`validate`)、
/// `Some` ならそのジョブの分だけ(`run`)。同じ記述は 1 つにまとめ、書いた順を保つ。
pub fn env_file_refs(def: &PipelineDefinition, job_id: Option<&str>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut add = |refs: &[String]| {
        for r in refs {
            if !out.contains(r) {
                out.push(r.clone());
            }
        }
    };
    add(&def.env_file);
    for job in &def.job {
        if job_id.is_none_or(|id| id == job.id) {
            add(&job.env_file);
        }
    }
    out
}
