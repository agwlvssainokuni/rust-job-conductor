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

//! 型(Types)。この Unit が足す値の形だけを持つ(entities.md)。入出力と OS に依存しない。
//!
//! 共有する型(`PipelineDefinition`、`JobExecution`、`PipelineState` など)は、
//! conductor-core(`conductor-core`)が定める。

use std::ffi::OsString;

use conductor_converter::TargetFormat;

/// コマンドの種類(BR1.1)。6つだけ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandKind {
    Run,
    Pipeline,
    Watch,
    Reset,
    Validate,
    Convert,
}

impl CommandKind {
    /// コマンドの名前(引数に書く語)。
    pub fn name(self) -> &'static str {
        match self {
            Self::Run => "run",
            Self::Pipeline => "pipeline",
            Self::Watch => "watch",
            Self::Reset => "reset",
            Self::Validate => "validate",
            Self::Convert => "convert",
        }
    }
}

/// 利用者が指定したコマンド(引数の解釈の結果。entities の CliCommand)。
///
/// パス(`config_path`、`convert_input`)は、`OsString` のまま保つ(UTF-8 でないパスも使える。NFR4.6)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliCommand {
    pub kind: CommandKind,
    /// `run` のときのみ。
    pub job_id: Option<String>,
    /// `pipeline` のとき(必須)と `validate` のとき(省略可)。
    pub pipeline_name: Option<String>,
    /// `convert` のときのみ。変換する入力ファイルのパス。
    pub convert_input: Option<OsString>,
    /// `convert` のときのみ。`--to` の値。
    pub convert_target: Option<TargetFormat>,
    /// `--config <パス>`。run・pipeline・watch・reset のときのみ。
    pub config_path: Option<OsString>,
    /// `-v` / `--verbose`。run・pipeline・validate のときのみ指定できる。
    pub verbose: bool,
    /// `--run-id <値>`。run・pipeline・watch・reset のときのみ。検査済み(空は「run ID なしに固定」)。
    /// 指定がなければ `None`(環境変数を見る)。
    pub run_id: Option<String>,
    /// `--all`。reset のときのみ。
    pub all: bool,
}

impl CliCommand {
    /// 指定のない値で、コマンドを作る(引数の解釈と、テストが使う)。
    pub fn new(kind: CommandKind) -> Self {
        Self {
            kind,
            job_id: None,
            pipeline_name: None,
            convert_input: None,
            convert_target: None,
            config_path: None,
            verbose: false,
            run_id: None,
            all: false,
        }
    }
}

/// 引数の解釈の結果。`--help`・`--version` は、コマンドの解釈より先に処理する(BR1.6)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedArgs {
    Help,
    Version,
    Command(CliCommand),
}

/// 使い方の誤りの種類(entities の UsageError.kind)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsageErrorKind {
    UnknownCommand,
    MissingArgument,
    UnexpectedArgument,
    UnknownOption,
    OptionNotAllowed,
    MissingValue,
    InvalidValue,
    UnknownJob,
}

impl UsageErrorKind {
    /// すべての種類(網羅のテストに使う)。
    pub const ALL: [UsageErrorKind; 8] = [
        Self::UnknownCommand,
        Self::MissingArgument,
        Self::UnexpectedArgument,
        Self::UnknownOption,
        Self::OptionNotAllowed,
        Self::MissingValue,
        Self::InvalidValue,
        Self::UnknownJob,
    ];
}

/// コマンドの使い方の誤り。ジョブは実行されない。終了コードは 68。
///
/// 表示の文は、種類ごとの固定の文である。`subject` は、誤りの対象の語(知らないコマンド、
/// オプション、ジョブIDなど)で、表示のときにエスケープする。入力の値は、それ以外に含めない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageError {
    pub kind: UsageErrorKind,
    pub subject: Option<String>,
}

impl UsageError {
    pub fn new(kind: UsageErrorKind) -> Self {
        Self {
            kind,
            subject: None,
        }
    }

    pub fn with_subject(kind: UsageErrorKind, subject: impl Into<String>) -> Self {
        Self {
            kind,
            subject: Some(subject.into()),
        }
    }
}

/// 表示の言語(entities の Language)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    En,
    Ja,
}

/// 表示の水準(entities の DisplayEvent.level)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    /// 常に表示する(標準エラー)。
    Error,
    /// `-v` のときだけ表示する(標準エラー)。
    Verbose,
    /// 結果の出力(標準出力)。
    Output,
}
