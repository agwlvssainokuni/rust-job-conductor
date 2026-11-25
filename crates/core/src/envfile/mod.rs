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
//!
//! `parse`・`merge` は crate の中。関数は、下の再公開の名前(`envfile::merge_env` など)で出る。

mod merge;
mod parse;

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt;

use thiserror::Error;

use crate::error::render_error;
use crate::run_id::env_name_rule;
use crate::types::{EnvFileRef, PipelineDefinition};

pub use merge::merge_env;
pub(crate) use merge::{merge_env_names, merge_unset_names};
pub use parse::parse_env_file;

/// `.env` の大きさの上限(1 MiB。BR3.1)。
pub const MAX_ENV_FILE_BYTES: usize = 1_048_576;
/// 1 つの `.env` の項目の数の上限(BR3.2)。
pub const MAX_ENV_ENTRIES: usize = 10_000;
/// 誤りを表示する件数の上限(BR2.8)。
pub(crate) const MAX_REPORTED_ERRORS: usize = 20;

// 理由の文([`EnvFileErrorKind::reason`])の「1 MiB」「10,000」と、上限の定数を結ぶ(BR3.1、BR3.2)。
const _: () = assert!(MAX_ENV_FILE_BYTES == 1_048_576);
const _: () = assert!(MAX_ENV_ENTRIES == 10_000);
/// 予約名の接頭辞の文字列のリテラル(誤りの理由の文に `concat!` で組み込む。BR1.3)。
macro_rules! reserved_env_prefix {
    () => {
        "CONDUCTOR_"
    };
}
pub(crate) use reserved_env_prefix;

/// 予約名の接頭辞(BR1.3)。
pub(crate) const RESERVED_ENV_PREFIX: &str = reserved_env_prefix!();

/// 予約名(ASCII の大文字にして `CONDUCTOR_` で始まる名前)か(BR1.3)。
pub(crate) fn is_reserved_env_name(name: &str) -> bool {
    name.len() >= RESERVED_ENV_PREFIX.len()
        && name.as_bytes()[..RESERVED_ENV_PREFIX.len()]
            .eq_ignore_ascii_case(RESERVED_ENV_PREFIX.as_bytes())
}

/// `.env` の読み取りの誤りの種類(14 種)。
///
/// 種類の文(日本語)は [`EnvFileErrorKind::label`] の 1 か所に持ち、誤りの表示(`EnvFileError`)はそれを書く(D10)。
/// 仕様書の識別子(`kind`)は、変種の名前の先頭を小文字にしたもの(例: `NotUtf8` は `notUtf8`)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvFileErrorKind {
    FileMissing,
    Unreadable,
    TooLarge,
    NotUtf8,
    TooManyEntries,
    InvalidName,
    ReservedName,
    CaseConflict,
    MissingEquals,
    UnterminatedQuote,
    InvalidEscape,
    TrailingGarbage,
    NulInValue,
    /// 名前が、そのファイルを読むジョブに当てはまる `unset_env` の名前と重なる(U3 の BR2.2)。
    UnsetConflict,
}

impl EnvFileErrorKind {
    /// 種類の文(日本語。表示と、cli の日本語の表示が使う。D10)。
    pub fn label(self) -> &'static str {
        match self {
            Self::FileMissing => "環境変数ファイルが存在しません",
            Self::Unreadable => "環境変数ファイルを読めません",
            Self::TooLarge => "環境変数ファイルが大きすぎます",
            Self::NotUtf8 => "環境変数ファイルが UTF-8 ではありません",
            Self::TooManyEntries => "環境変数ファイルの項目が多すぎます",
            Self::InvalidName => "環境変数の名前が正しくありません",
            Self::ReservedName => "予約された環境変数の名前です",
            Self::CaseConflict => "大文字小文字だけが違う環境変数の名前があります",
            Self::MissingEquals => "`KEY=VALUE` の形ではありません",
            Self::UnterminatedQuote => "引用符が閉じていません",
            Self::InvalidEscape => "許されないエスケープです",
            Self::TrailingGarbage => "閉じた引用符のあとに余計な文字があります",
            Self::NulInValue => "値に NUL が含まれています",
            Self::UnsetConflict => "消す環境変数の名前と重なっています",
        }
    }

    /// 固定の理由の文(入力の値を含まない)。
    pub fn reason(self) -> &'static str {
        match self {
            Self::FileMissing => "`env_file` に書かれたファイルが、見つかりません",
            Self::Unreadable => "`env_file` に書かれたファイルを、読めません",
            Self::TooLarge => "ファイルの大きさが、上限(1 MiB)を超えています",
            Self::NotUtf8 => "ファイルは UTF-8 で書きます",
            Self::TooManyEntries => "項目の数が、上限(10,000)を超えています",
            Self::InvalidName => concat!("名前は、", env_name_rule!(), "で書きます"),
            Self::ReservedName => {
                concat!("`", reserved_env_prefix!(), "` で始まる名前は、書けません")
            }
            Self::CaseConflict => "大文字小文字だけが違う名前が、同じファイルにあります",
            Self::MissingEquals => "`KEY=VALUE` の形で書きます",
            Self::UnterminatedQuote => "引用符が、同じ行で閉じていません",
            Self::InvalidEscape => {
                "二重引用符の中で使えるエスケープは、`\\n`、`\\r`、`\\t`、`\\\"`、`\\\\` だけです"
            }
            Self::TrailingGarbage => "閉じた引用符のあとは、空白とコメントだけを書けます",
            Self::NulInValue => "値に NUL(U+0000)は書けません",
            Self::UnsetConflict => "`unset_env` の名前と同じ名前があります",
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
}

impl EnvFileError {
    /// 種類の固定の理由で、誤りを作る。
    pub fn new(file: impl Into<String>, line: Option<usize>, kind: EnvFileErrorKind) -> Self {
        Self {
            file: file.into(),
            line,
            kind,
        }
    }

    fn render(&self) -> String {
        render_error(&self.file, self.line, self.kind.label(), self.kind.reason())
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
    /// その項目の行番号(1 始まり。`unset_env` との重なりの誤りの位置。U3 の BR2.2)。
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
    /// 表示用のファイル名([`parse_env_file`] に渡した名前。`unset_env` との重なりの誤りに書く。
    /// U3 の BR2.2)。
    pub file: String,
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

/// 足す環境の 1 項目。値は秘密で、`Debug` は伏せ字にする(BR8.1)。
#[derive(Clone, PartialEq, Eq)]
pub struct EffectiveEntry {
    pub name: String,
    pub value: String,
}

impl fmt::Debug for EffectiveEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EffectiveEntry")
            .field("name", &self.name)
            .field("value", &"***")
            .finish()
    }
}

/// 1 つのジョブの 1 回の起動に渡す環境の変更(層を統合した結果)。引き継いだ環境へ「足す」環境変数と、
/// 引き継いだ環境から「消す」名前を持つ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveEnv {
    /// 層の順。完全に同じ名前は 1 つ(後ろの層が勝つ)。
    pub entries: Vec<EffectiveEntry>,
    /// 引き継いだ環境から、足す前に消す名前(U3 の BR3.1、BR3.2)。パイプライン、次にジョブの
    /// `unset_env` の書いた順で、完全に同じ名前は最初の 1 つ。`entries` の名前とは重ならない
    /// (定義と `.env` の検証で止める)。`unset_env` を書かなければ空。
    pub removals: Vec<String>,
}

impl EffectiveEnv {
    /// 起動の口へ渡す、足す環境(名前と値)。並びの順に足す(BR4.3)。
    pub fn additions(&self) -> Vec<(&str, &str)> {
        self.entries
            .iter()
            .map(|e| (e.name.as_str(), e.value.as_str()))
            .collect()
    }
}

/// 読んだ `.env` の名前と、そのファイルを読むジョブに当てはまる `unset_env` の名前の重なりを調べる
/// (U3 の BR2.2)。
///
/// 範囲は [`env_file_refs`] と同じ(`job_id` が `None` ならすべてのジョブ、`Some` ならそのジョブ)。
/// 範囲のジョブごとに、パイプラインとジョブの `env_file` で読めたファイルの名前を、パイプラインと
/// ジョブの `unset_env` の名前と比べる。比べ方は、どの OS でも ASCII の大文字小文字を区別しない
/// (security-design の 1)。重なった項目ごとに、そのファイルと行で `UnsetConflict` を 1 つ返す。同じ
/// ファイルを複数のジョブが読んでも、誤りは「ファイルと行」で 1 つにまとめ、ファイルの読み取り順、
/// 行の順に並べる(reliability-design の 1)。誤りに値も名前も入れない。
pub fn check_unset_conflicts(
    def: &PipelineDefinition,
    job_id: Option<&str>,
    files: &LoadedEnvFiles,
) -> Vec<EnvFileError> {
    // 重なった項目の (ファイルの位置, 行)。並べ替えと重複の除きを兼ねる。
    let mut found: BTreeSet<(usize, usize)> = BTreeSet::new();
    let jobs = def
        .job
        .iter()
        .filter(|job| job_id.is_none_or(|id| id == job.id));
    for job in jobs {
        let unset: HashSet<String> = def
            .unset_env
            .iter()
            .chain(&job.unset_env)
            .map(|name| name.to_ascii_lowercase())
            .collect();
        if unset.is_empty() {
            continue;
        }
        for r in def.env_file.iter().chain(&job.env_file) {
            let Some(&index) = files.by_ref.get(&r.path) else {
                continue;
            };
            let Some(content) = files.files.get(index) else {
                continue;
            };
            for entry in &content.entries {
                if unset.contains(&entry.name.to_ascii_lowercase()) {
                    found.insert((index, entry.line));
                }
            }
        }
    }
    found
        .into_iter()
        .map(|(index, line)| {
            EnvFileError::new(
                files.files[index].file.clone(),
                Some(line),
                EnvFileErrorKind::UnsetConflict,
            )
        })
        .collect()
}

/// 読む対象の `env_file` の記述の一覧を作る(BR6.1。v0.10.0 の BR2.1)。
///
/// `job_id` が `None` なら `[pipeline].env_file` とすべてのジョブの `env_file`(`pipeline`、`validate`)、
/// `Some` なら `[pipeline].env_file` とそのジョブの分(`run`)。同じパスの文字列は 1 つにまとめ、
/// 書いた順を保つ。まとめるときは `required` を論理和にする(どれか 1 つでも必ず要るなら必ず要る)。
pub fn env_file_refs(def: &PipelineDefinition, job_id: Option<&str>) -> Vec<EnvFileRef> {
    let jobs = def
        .job
        .iter()
        .filter(|job| job_id.is_none_or(|id| id == job.id))
        .map(|job| &job.env_file);
    let mut out: Vec<EnvFileRef> = Vec::new();
    // 見たパスの文字列 → `out` の位置(記述の数に比例する時間で、重複を除き required をまとめる)。
    let mut seen: HashMap<&str, usize> = HashMap::new();
    for refs in std::iter::once(&def.env_file).chain(jobs) {
        for r in refs {
            match seen.get(r.path.as_str()) {
                Some(&i) => out[i].required |= r.required,
                None => {
                    seen.insert(r.path.as_str(), out.len());
                    out.push(r.clone());
                }
            }
        }
    }
    out
}
