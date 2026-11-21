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
use conductor_core::Assumption;

/// コマンドの種類(BR1.1)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandKind {
    Run,
    Pipeline,
    Watch,
    Reset,
    Validate,
    Convert,
    /// 図の出力(U2。BR6.4)。
    Graph,
    /// 定義のひな形の作成(U4。BR1.1)。
    Init,
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
            Self::Graph => "graph",
            Self::Init => "init",
        }
    }
}

/// `init` のパイプライン名の既定(BR2.2)。
pub const DEFAULT_INIT_PIPELINE_NAME: &str = "my-pipeline";

/// `init` の定義のひな形の出力先の既定のパス(BR1.1)。
pub const DEFAULT_INIT_PATH: &str = "conductor.toml";

/// `conductor init` の出力先(entities の InitTarget)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InitTarget {
    /// ファイルを作る(既定は `conductor.toml`)。
    File(OsString),
    /// `--config -`: ファイルを作らず、標準出力にひな形だけを出す(BR1.5)。
    Stdout,
}

impl InitTarget {
    /// `--config` の値から、出力先を決める。`-` だけが標準出力で、それ以外はファイルのパス(BR1.5)。
    pub fn from_path(path: OsString) -> Self {
        if path == "-" {
            Self::Stdout
        } else {
            Self::File(path)
        }
    }
}

/// `init` のオプション(entities の InitOptions。BR1.1、BR1.5、BR2.2)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitOptions {
    pub target: InitTarget,
    /// `--force`。`Stdout` のときは、無視する(BR1.5)。
    pub force: bool,
    /// パイプライン名。省略したときは、固定の `my-pipeline`(BR2.2)。
    pub pipeline_name: String,
}

impl Default for InitOptions {
    fn default() -> Self {
        Self {
            target: InitTarget::File(OsString::from(DEFAULT_INIT_PATH)),
            force: false,
            pipeline_name: DEFAULT_INIT_PIPELINE_NAME.to_string(),
        }
    }
}

/// `graph` の出力の形式(`--format`。既定は `text`)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GraphFormat {
    #[default]
    Text,
    Mermaid,
}

/// `graph` のオプション(entities の GraphOptions)。`-v` と `--run-id` は、`CliCommand` の
/// `verbose` と `run_id` を使う。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GraphOptions {
    /// `--format`。`graph` のときのみ。
    pub format: GraphFormat,
    /// `--state`。`graph` のときのみ。ステートを読んで、状態を重ねる(BR4.1)。
    pub with_state: bool,
}

/// `--dry-run` のオプション(entities の DryRunOptions。U3)。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DryRunOptions {
    /// `--dry-run`。`run` と `pipeline` のときのみ。ジョブを実行せず、ステートに書かず、計画を出す。
    pub dry_run: bool,
    /// `--assume <ジョブID>=<終了コード>`(複数回)。`--dry-run` と一緒のときだけ(指定順)。
    pub assumptions: Vec<Assumption>,
}

/// 利用者が指定したコマンド(引数の解釈の結果。entities の CliCommand)。
///
/// パス(`config_path`、`convert_input`)は、`OsString` のまま保つ(UTF-8 でないパスも使える。NFR4.6)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliCommand {
    pub kind: CommandKind,
    /// `run` のときのみ。
    pub job_id: Option<String>,
    /// `pipeline` のとき(必須)と `validate`・`graph` のとき(省略可)。
    pub pipeline_name: Option<String>,
    /// `convert` のときのみ。変換する入力ファイルのパス。
    pub convert_input: Option<OsString>,
    /// `convert` のときのみ。`--to` の値。
    pub convert_target: Option<TargetFormat>,
    /// `--config <パス>`。run・pipeline・watch・reset のときのみ。
    pub config_path: Option<OsString>,
    /// `-v` / `--verbose`。run・pipeline・validate・graph のときのみ指定できる。
    pub verbose: bool,
    /// `--run-id <値>`。run・pipeline・watch・reset のときと、`--state` つきの graph のとき。検査済み(空は「run ID なしに固定」)。
    /// 指定がなければ `None`(環境変数を見る)。
    pub run_id: Option<String>,
    /// `--all`。reset のときのみ。
    pub all: bool,
    /// `--remove-on-complete`。run・pipeline のときのみ。指定があれば、完了でステートを削除する。
    pub remove_on_complete: bool,
    /// `graph` のときのみ。`--format` と `--state`。
    pub graph: GraphOptions,
    /// `run`・`pipeline` のときのみ。`--dry-run` と `--assume`。
    pub dry_run: DryRunOptions,
    /// `--force`。`init` のときのみ(U4。BR1.4)。出力先は `config_path`、名前は `pipeline_name` を使う
    /// ([`CliCommand::init_options`])。
    pub force: bool,
}

impl CliCommand {
    /// `init` のオプション(出力先、`--force`、パイプライン名)を、コマンドから取り出す(BR1.1、BR1.5、BR2.2)。
    pub fn init_options(&self) -> InitOptions {
        let defaults = InitOptions::default();
        InitOptions {
            target: match &self.config_path {
                Some(path) => InitTarget::from_path(path.clone()),
                None => defaults.target,
            },
            force: self.force,
            pipeline_name: self.pipeline_name.clone().unwrap_or(defaults.pipeline_name),
        }
    }

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
            remove_on_complete: false,
            graph: GraphOptions::default(),
            dry_run: DryRunOptions::default(),
            force: false,
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
    /// 常に表示する(標準エラー)。エラーではない知らせ(リトライの前の知らせ。BR2.4)。
    Notice,
    /// `-v` のときだけ表示する(標準エラー)。
    Verbose,
    /// 結果の出力(標準出力)。
    Output,
}
