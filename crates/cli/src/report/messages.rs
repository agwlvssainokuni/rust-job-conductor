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

//! 文言の表(Messages)。英語と日本語の対の定数。
//!
//! 種類ごとの固定の文を持つ。入力の値(ジョブID、パス、キー名など)は、この表に含めず、
//! 組み立てるときに、エスケープして入れる。トークン(`success`・`warning`・`error`・`skipped`・
//! `pending`・`job=`・`result=`)は、言語に関わらず英語のまま。

use conductor_converter::ConversionErrorKind;
use conductor_core::{DecisionReason, DefinitionErrorKind};
use conductor_runtime::{CommandErrorKind, MismatchKind, StateErrorKind};

use crate::types::{Language, UsageErrorKind};

/// 英語と日本語の対。
#[derive(Debug, Clone, Copy)]
pub struct Text {
    pub en: &'static str,
    pub ja: &'static str,
}

impl Text {
    pub const fn new(en: &'static str, ja: &'static str) -> Self {
        Self { en, ja }
    }

    pub fn get(&self, lang: Language) -> &'static str {
        match lang {
            Language::En => self.en,
            Language::Ja => self.ja,
        }
    }
}

/// エラーの行の先頭(言語に関わらず同じ)。
pub const ERROR_PREFIX: &str = "conductor: error: ";
/// 警告の行の先頭(言語に関わらず同じ)。
pub const WARNING_PREFIX: &str = "conductor: warning: ";
/// リトライの知らせの行の先頭(言語に関わらず同じ。エラーではない。BR2.4)。
pub const RETRY_PREFIX: &str = "conductor: retry: ";
/// `-v` の行の先頭。
pub const VERBOSE_PREFIX: &str = "[conductor] ";
/// パニックのフックと、シグナルの準備の失敗の、固定の英語の文(ロケールを判定しない)。
pub const INTERNAL_ERROR: &str = "conductor: error: internal error";
pub const SIGNAL_SETUP_FAILED: &str = "conductor: error: failed to set up signal handling";

// ---- 使い方 ----

pub const USAGE: Text = Text::new(
    "\
Usage:
  conductor run [-v] [--config <path>] [--run-id <id>] [--remove-on-complete] <job-id>
  conductor pipeline [-v] [--config <path>] [--run-id <id>] [--remove-on-complete] <pipeline-name>
  conductor watch [--config <path>] [--run-id <id>]
  conductor reset [--config <path>] [--run-id <id> | --all]
  conductor validate [-v] [--config <path>] [<pipeline-name>]
  conductor convert <file> --to <yaml|toml>
  conductor --help | --version

Options:
  -v, --verbose       print job results, skips and transitions to standard error (run, pipeline; validate prints one line)
  --config <path>     definition file (default: conductor.toml in the current directory)
  --run-id <id>       run ID that separates the state file (run, pipeline, watch, reset; default: the environment variable
                      named by run_id_env in the definition, or CONDUCTOR_RUN_ID; an empty value means no run ID)
  --remove-on-complete
                      remove the state file when the run completes (run, pipeline; default: keep it, or the
                      remove_on_complete setting of the definition)
  --all               remove every state file of the definition, with or without a run ID (reset only)
  --to <yaml|toml>    target format of convert
  -h, --help          print this help
  -V, --version       print the version
",
    "\
使い方:
  conductor run [-v] [--config <パス>] [--run-id <ID>] [--remove-on-complete] <ジョブID>
  conductor pipeline [-v] [--config <パス>] [--run-id <ID>] [--remove-on-complete] <パイプライン名>
  conductor watch [--config <パス>] [--run-id <ID>]
  conductor reset [--config <パス>] [--run-id <ID> | --all]
  conductor validate [-v] [--config <パス>] [<パイプライン名>]
  conductor convert <ファイル> --to <yaml|toml>
  conductor --help | --version

オプション:
  -v, --verbose       ジョブの結果、スキップ、遷移を標準エラーに表示する(run、pipeline。validate は 1 行)
  --config <パス>     定義ファイル(省略時: 実行ディレクトリの conductor.toml)
  --run-id <ID>       ステートファイルを分ける run ID(run、pipeline、watch、reset。省略時: 定義の run_id_env の環境変数、
                      なければ CONDUCTOR_RUN_ID。空の値は run ID なし)
  --remove-on-complete
                      完了のときにステートファイルを消す(run、pipeline。省略時: 残す。定義の remove_on_complete に従う)
  --all               定義のステートファイルを、run ID の有無によらず、すべて消す(reset のみ)
  --to <yaml|toml>    convert の変換後の形式
  -h, --help          この使い方を表示する
  -V, --version       版を表示する
",
);

/// `validate` が通ったときの 1 行の本文(`-v` のとき。先頭の `[conductor] ` は含まない)。
pub fn definition_valid(lang: Language, job_count: usize) -> String {
    match lang {
        Language::En => {
            let unit = if job_count == 1 { "job" } else { "jobs" };
            format!("definition is valid ({job_count} {unit})")
        }
        Language::Ja => format!("定義は正しいです(ジョブ {job_count} 個)"),
    }
}

/// リトライの前の知らせの本文(先頭の `conductor: retry: ` と改行は含まない。BR2.4、BR2.15)。
///
/// 固定の文と整数と、エスケープ済みのジョブIDだけで組み立てる(入力の値は入れない)。
/// 最初の文言(最終の確定は U3)。
pub fn retry_scheduled(
    lang: Language,
    escaped_job_id: &str,
    last_exit_code: i32,
    retry_number: u32,
    max: u32,
    wait_secs: u64,
) -> String {
    match lang {
        Language::En => format!(
            "job '{escaped_job_id}' exited with code {last_exit_code}; \
             retry {retry_number} of {max} in {wait_secs} seconds"
        ),
        Language::Ja => format!(
            "ジョブ '{escaped_job_id}' が終了コード {last_exit_code} で終わりました。\
             {wait_secs} 秒後にリトライします({retry_number}/{max} 回目)"
        ),
    }
}

pub fn usage_kind(kind: UsageErrorKind) -> Text {
    match kind {
        UsageErrorKind::UnknownCommand => Text::new("unknown command", "知らないコマンドです"),
        UsageErrorKind::MissingArgument => Text::new("missing argument", "引数が足りません"),
        UsageErrorKind::UnexpectedArgument => {
            Text::new("unexpected argument", "余分な引数があります")
        }
        UsageErrorKind::UnknownOption => Text::new("unknown option", "知らないオプションです"),
        UsageErrorKind::OptionNotAllowed => Text::new(
            "option not allowed for this command",
            "このコマンドでは使えないオプションです",
        ),
        UsageErrorKind::MissingValue => {
            Text::new("option requires a value", "オプションに値がありません")
        }
        UsageErrorKind::InvalidValue => Text::new("invalid value", "値が正しくありません"),
        UsageErrorKind::UnknownJob => Text::new(
            "job is not defined in the definition file",
            "定義にないジョブIDです",
        ),
    }
}

// ---- 定義・command・ステート・変換の種類の文 ----

/// 定義の誤りの種類の文。日本語は、conductor-core の表示(`Display`)と同じ。
pub fn definition_kind(lang: Language, kind: DefinitionErrorKind) -> String {
    if lang == Language::Ja {
        return kind.to_string();
    }
    use DefinitionErrorKind as K;
    let text = match kind {
        K::FileMissing => "definition file does not exist",
        K::Unreadable => "cannot read definition file",
        K::TooLarge => "definition file is too large",
        K::Syntax => "syntax error",
        K::TypeMismatch => "wrong value type",
        K::UnknownKey => "unknown key",
        K::VersionMissing => "format_version is missing",
        K::VersionUnsupported => "unsupported format_version",
        K::UnknownTarget => "transition target job is not defined",
        K::ErrorTransition => "no transition can be written for the error result",
        K::BackwardOrSelfTarget => "transition target must be a later job or @end",
        K::OverlappingExitCodes => "success and warning exit codes overlap",
        K::ZeroAsWarning => "0 cannot be a warning exit code",
        K::PipelineNameMismatch => "pipeline name does not match the definition",
        K::DuplicateJobId => "duplicate job id",
        K::InvalidJobId => "invalid job id",
        K::NoJobs => "no jobs are defined",
        K::EmptyField => "required value is empty",
        K::DuplicateTransition => "duplicate transition for the same result",
        K::InvalidEnvName => "invalid environment variable name",
        K::RetryNotTable => "retry must be a table",
        K::RetryMissingKey => "a required key of retry is missing",
        K::RetryExitCodesEmpty => "retry exit_codes is empty",
        K::RetryExitCodeInvalid => "retry exit_codes has an invalid exit code",
        K::RetryExitCodesOverlap => "retry exit_codes overlap the success or warning exit codes",
        K::RetryMaxInvalid => "retry max is invalid",
        K::RetryDurationInvalid => "retry interval is invalid",
        K::RetryBackoffIncomplete => {
            "retry multiplier and max_interval must be given together or not at all"
        }
        K::RetryMultiplierInvalid => "retry multiplier is invalid",
        K::RetryMaxIntervalTooSmall => "retry max_interval is smaller than interval",
    };
    text.to_string()
}

pub fn command_kind(kind: CommandErrorKind) -> Text {
    match kind {
        CommandErrorKind::UnclosedQuote => Text::new(
            "command has an unclosed quote",
            "command の引用符が閉じていません",
        ),
        CommandErrorKind::Empty => Text::new("command is empty", "command が空です"),
    }
}

/// ステートの誤りの種類の文。日本語は、conductor-runtime の表示と同じ。
pub fn state_kind(kind: StateErrorKind) -> Text {
    match kind {
        StateErrorKind::Unreadable => {
            Text::new("cannot read state file", "ステートファイルを読めません")
        }
        StateErrorKind::TooLarge => {
            Text::new("state file is too large", "ステートファイルが大きすぎます")
        }
        StateErrorKind::Corrupt => {
            Text::new("state file is corrupt", "ステートファイルが壊れています")
        }
        StateErrorKind::VersionMismatch => Text::new(
            "state file version is not supported",
            "ステートファイルの版が合いません",
        ),
        StateErrorKind::WriteFailed => Text::new(
            "cannot write state file",
            "ステートファイルを書き込めません",
        ),
        StateErrorKind::RemoveFailed => Text::new(
            "cannot remove state file",
            "ステートファイルを削除できません",
        ),
    }
}

/// 読んだステートの誤りに、`reset` を案内する種類か。
pub fn state_kind_suggests_reset(kind: StateErrorKind) -> bool {
    matches!(
        kind,
        StateErrorKind::Unreadable
            | StateErrorKind::TooLarge
            | StateErrorKind::Corrupt
            | StateErrorKind::VersionMismatch
    )
}

pub const RESET_HINT: Text = Text::new(
    "run `conductor reset` to discard the state",
    "`conductor reset` でステートを初期化できます",
);

/// run ID つきのステートを読んだ誤りの案内(値は含めない)。
pub const RESET_HINT_RUN_ID: Text = Text::new(
    "run `conductor reset --run-id <ID>` to discard the state",
    "`conductor reset --run-id <ID>` でステートを初期化できます",
);

/// `reset --all` が、読めなくて残したファイルの警告(続けて、ファイル名を 1 行ずつ)。
pub const RESET_ALL_KEPT: Text = Text::new(
    "kept state files that could not be read (remove one with `conductor reset --run-id <ID>`):",
    "読めなかったため、次のステートファイルは残しました(`conductor reset --run-id <ID>` で個別に消せます):",
);

pub const JOB_NOT_STARTED: Text = Text::new("the job was not started", "ジョブは起動していません");
pub const JOB_EXECUTED: Text = Text::new(
    "the job has already been executed",
    "ジョブは実行されました",
);

/// 変換の誤りの種類の文。日本語は、conductor-converter の表示と同じ。
pub fn conversion_kind(lang: Language, kind: ConversionErrorKind) -> String {
    if lang == Language::Ja {
        return kind.to_string();
    }
    use ConversionErrorKind as K;
    let text = match kind {
        K::FileMissing => "input file does not exist",
        K::Unreadable => "cannot read input file",
        K::TooLarge => "input is too large",
        K::Syntax => "syntax error",
        K::Unrepresentable => "value cannot be represented in the target format",
        K::RootNotMap => "top level is not a map",
    };
    text.to_string()
}

pub const AT_KEY: Text = Text::new("at", "場所");

// ---- ステートの食い違い ----

pub fn mismatch_kind(kind: MismatchKind) -> Text {
    match kind {
        MismatchKind::PipelineName => Text::new("pipeline name", "パイプライン名"),
        MismatchKind::JobIds => Text::new("job ids", "ジョブID"),
        MismatchKind::JobOrder => Text::new("job order", "ジョブの並び"),
    }
}

pub const MISMATCH: Text = Text::new(
    "state does not match the definition",
    "ステートが定義と食い違っています",
);

// ---- その他のエラー ----

pub const REJECTED: Text = Text::new(
    "cannot run this job now",
    "このジョブは、いま実行できません",
);

pub fn reject_reason(reason: DecisionReason) -> Text {
    match reason {
        DecisionReason::OutOfOrder => Text::new(
            "earlier jobs have not finished (out of order)",
            "前のジョブが終わっていません(順序外)",
        ),
        DecisionReason::SkipTarget => Text::new("skip target", "スキップの対象です"),
        DecisionReason::AfterEnd => Text::new("after @end", "@end より後ろです"),
        DecisionReason::AlreadyExecuted => Text::new("already executed", "実行済みです"),
    }
}

pub const CANNOT_START: Text = Text::new("cannot start command", "コマンドを起動できません");
pub const SUCCESS_CODE_NOT_LISTED: Text = Text::new(
    "exit code 0 is not listed in success_exit_codes",
    "終了コード 0 は、success_exit_codes に含まれていません",
);
pub const OUTPUT_FAILED: Text = Text::new(
    "cannot write to standard output",
    "標準出力に書き込めません",
);
pub const COMPLETION_UNDETERMINED: Text = Text::new(
    "cannot determine whether the pipeline is complete",
    "パイプラインが完了したかを判定できません",
);

// ---- -v の文言 ----

pub const EXIT_CODE: Text = Text::new("exit code", "終了コード");
pub const CONDUCTOR_EXITS_WITH: Text = Text::new("conductor exits with", "conductor の終了コード");
pub const KILLED_BY_SIGNAL: Text = Text::new("terminated by signal", "シグナルで終了");
pub const COULD_NOT_START: Text = Text::new("could not start", "起動できませんでした");
pub const INTERRUPTED_BY_SIGNAL: Text = Text::new("interrupted by signal", "シグナルで中断");
pub const NOT_STARTED: Text = Text::new("not started", "起動せず");
pub const SKIPPED_WORD: Text = Text::new("skipped", "スキップ");
pub const SKIPPED_LIST: Text = Text::new("skipped", "スキップ");

pub fn skip_reason(reason: DecisionReason) -> Text {
    match reason {
        DecisionReason::SkipTarget => Text::new("skipped by a transition", "遷移によりスキップ"),
        DecisionReason::AfterEnd => Text::new("skipped after @end", "@end の後ろのためスキップ"),
        DecisionReason::AlreadyExecuted => Text::new("already executed", "実行済み"),
        DecisionReason::OutOfOrder => Text::new("out of order", "順序外"),
    }
}

// ---- watch ----

pub const WATCH_PIPELINE: Text = Text::new("Pipeline", "パイプライン");
/// run ID の行の見出し(run ID があるときだけ。英語・日本語とも同じ)。
pub const WATCH_RUN_ID: &str = "Run ID";
pub const WATCH_EMPTY: Text = Text::new(
    "no state (nothing has run yet)",
    "ステートはありません(まだ実行されていません)",
);
/// 完了のステートの表の下に足す 1 行(FR3.1)。
pub const WATCH_COMPLETED: Text = Text::new("completed", "完了");
pub const HEADER_JOB: Text = Text::new("JOB", "ジョブ");
pub const HEADER_STATUS: Text = Text::new("STATUS", "状態");
pub const HEADER_NOTE: Text = Text::new("NOTE", "補足");
pub const NOTE_BY_TRANSITION: Text = Text::new("skipped by a transition", "遷移によりスキップ");
pub const NOTE_BY_END: Text = Text::new("skipped after @end", "@end の後ろのためスキップ");
