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
use conductor_core::{
    Classification, DefinitionErrorKind, EnvFileErrorKind, MismatchKind, NextStepNote,
    PlanDisposition, RejectReason, SkipCause,
};
use conductor_runtime::{CommandErrorKind, CreateError, StateErrorKind};

use crate::report::StateContext;
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

/// エラーの行の先頭の文字列のリテラル。`&str` の定数どうしは `const` で連結できないため、
/// 固定の文に `concat!` でつなぐときは、このマクロを使う。
macro_rules! error_prefix {
    () => {
        "conductor: error: "
    };
}

/// エラーの行の先頭(言語に関わらず同じ)。
pub const ERROR_PREFIX: &str = error_prefix!();
/// 警告の行の先頭(言語に関わらず同じ)。
pub const WARNING_PREFIX: &str = "conductor: warning: ";
/// リトライの知らせの行の先頭(言語に関わらず同じ。エラーではない。BR2.4)。
pub const RETRY_PREFIX: &str = "conductor: retry: ";
/// `-v` の行の先頭。
pub const VERBOSE_PREFIX: &str = "[conductor] ";
/// パニックのフックと、シグナルの準備の失敗の、固定の英語の文(ロケールを判定しない)。
pub const INTERNAL_ERROR: &str = concat!(error_prefix!(), "internal error");
pub const SIGNAL_SETUP_FAILED: &str = concat!(error_prefix!(), "failed to set up signal handling");

// ---- 使い方 ----

pub const USAGE: Text = Text::new(
    "\
Usage:
  conductor run [-v] [--config <path>] [--run-id <id>] [--remove-on-complete] [--dry-run [--assume <job-id>=<code>]] <job-id>
  conductor pipeline [-v] [--config <path>] [--run-id <id>] [--remove-on-complete] [--dry-run [--assume <job-id>=<code>]] <pipeline-name>
  conductor watch [--config <path>] [--run-id <id>]
  conductor reset [--config <path>] [--run-id <id> | --all]
  conductor validate [-v] [--config <path>] [<pipeline-name>]
  conductor graph [-v] [--config <path>] [--format <text|mermaid|dot>] [--state [--run-id <id>]] [<pipeline-name>]
  conductor init [--config <path>] [--force] [<pipeline-name>]
  conductor convert <file> --to <yaml|toml>
  conductor --help | --version

Options:
  -v, --verbose       print job results, skips and transitions to standard error (run, pipeline; validate prints one line);
                      graph prints the command, retry and skip details under each job
  --config <path>     definition file (default: conductor.toml in the current directory; init: - writes the template to standard output)
  --run-id <id>       run ID that separates the state file (run, pipeline, watch, reset, graph --state; default: the environment variable
                      named by run_id_env in the definition, or CONDUCTOR_RUN_ID; an empty value means no run ID)
  --remove-on-complete
                      remove the state file when the run completes (run, pipeline; default: keep it, or the
                      remove_on_complete setting of the definition)
  --all               remove every state file of the definition, with or without a run ID (reset only)
  --format <text|mermaid|dot>
                      output format of graph: text, mermaid or dot (Graphviz) (default: text)
  --state             overlay the state of the jobs on the graph (graph only; reads the state file, never writes it)
  --dry-run           print the execution plan without running jobs, writing the state or setting up signals (run, pipeline)
  --assume <job-id>=<code>
                      assume the exit code (0-255) of a job in the plan; repeatable (with --dry-run only; default: 0)
  --force             overwrite the file that already exists (init only; default: leave it and exit with 69)
  --to <yaml|toml>    target format of convert
  -h, --help          print this help
  -V, --version       print the version
",
    "\
使い方:
  conductor run [-v] [--config <パス>] [--run-id <ID>] [--remove-on-complete] [--dry-run [--assume <ジョブID>=<終了コード>]] <ジョブID>
  conductor pipeline [-v] [--config <パス>] [--run-id <ID>] [--remove-on-complete] [--dry-run [--assume <ジョブID>=<終了コード>]] <パイプライン名>
  conductor watch [--config <パス>] [--run-id <ID>]
  conductor reset [--config <パス>] [--run-id <ID> | --all]
  conductor validate [-v] [--config <パス>] [<パイプライン名>]
  conductor graph [-v] [--config <パス>] [--format <text|mermaid|dot>] [--state [--run-id <ID>]] [<パイプライン名>]
  conductor init [--config <パス>] [--force] [<パイプライン名>]
  conductor convert <ファイル> --to <yaml|toml>
  conductor --help | --version

オプション:
  -v, --verbose       ジョブの結果、スキップ、遷移を標準エラーに表示する(run、pipeline。validate は 1 行)。
                      graph は、ジョブごとに、コマンド、リトライ、飛び越しの詳細を表示する
  --config <パス>     定義ファイル(省略時: 実行ディレクトリの conductor.toml。init: - なら、ひな形を標準出力に出す)
  --run-id <ID>       ステートファイルを分ける run ID(run、pipeline、watch、reset、graph --state。省略時: 定義の run_id_env の環境変数、
                      なければ CONDUCTOR_RUN_ID。空の値は run ID なし)
  --remove-on-complete
                      完了のときにステートファイルを消す(run、pipeline。省略時: 残す。定義の remove_on_complete に従う)
  --all               定義のステートファイルを、run ID の有無によらず、すべて消す(reset のみ)
  --format <text|mermaid|dot>
                      graph の出力の形式。text、mermaid、dot(Graphviz)のどれか(省略時: text)
  --state             ジョブの状態を図に重ねる(graph のみ。ステートファイルを読むだけで、書かない)
  --dry-run           ジョブを実行せず、ステートに書かず、シグナルも準備せずに、実行計画を表示する(run、pipeline)
  --assume <ジョブID>=<終了コード>
                      計画の中で、ジョブの終了コード(0〜255)を仮定する。複数回指定できる(--dry-run と一緒のときだけ。省略時: 0)
  --force             既にあるファイルを上書きする(init のみ。省略時: 何も変えず、終了コード 69)
  --to <yaml|toml>    convert の変換後の形式
  -h, --help          この使い方を表示する
  -V, --version       版を表示する
",
);

// ---- init(U4。BR1.2、BR1.4、BR3.2、BR4.1) ----

/// `init` が、ファイルを作った(パスはエスケープ済み)。
pub fn init_created(lang: Language, path: &str) -> String {
    match lang {
        Language::En => format!("created {path}"),
        Language::Ja => format!("{path} を作成しました"),
    }
}

/// `init --force` が、既にあったファイルを置き換えた(パスはエスケープ済み)。
pub fn init_replaced(lang: Language, path: &str) -> String {
    match lang {
        Language::En => format!("replaced {path}"),
        Language::Ja => format!("{path} を置き換えました"),
    }
}

/// `init` の作る先に、ファイルが既にある(パスはエスケープ済み)。
pub fn init_exists(lang: Language, path: &str) -> String {
    match lang {
        Language::En => format!("{path} already exists (use --force to overwrite)"),
        Language::Ja => format!("{path} は既にあります(上書きするには --force)"),
    }
}

/// `init` が、ファイルを作れなかった(`already_exists` 以外。パスはエスケープ済み)。
/// 種類ごとの固定の文と、パスだけ。OS のエラーの文は含めない(BR3.2)。
pub fn init_cannot_create(lang: Language, kind: CreateError, path: &str) -> String {
    let reason = match kind {
        CreateError::ParentMissing => Text::new(
            "the parent directory does not exist",
            "親のディレクトリがありません",
        ),
        CreateError::IsDirectory => Text::new("it is a directory", "ディレクトリです"),
        CreateError::PermissionDenied => Text::new("permission denied", "書く権限がありません"),
        // `AlreadyExists` は、別の文(`init_exists`)で出す。ここでは、入出力の誤りと同じに扱う。
        CreateError::OtherIo | CreateError::AlreadyExists => {
            Text::new("I/O error", "入出力の誤りです")
        }
    };
    match lang {
        Language::En => format!("cannot create {path}: {}", reason.en),
        Language::Ja => format!("{path} を作れません: {}", reason.ja),
    }
}

/// `init --config -` で、ひな形を標準出力に書けなかった。
pub const INIT_STDOUT_FAILED: Text = Text::new(
    "cannot write the template to standard output",
    "ひな形を標準出力に書けません",
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

/// 定義の誤りの種類の文。日本語は、conductor-core の `DefinitionErrorKind::label()`(cli に持たない)。
pub fn definition_kind(kind: DefinitionErrorKind) -> Text {
    use DefinitionErrorKind as K;
    let en = match kind {
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
        K::ReservedEnvName => "reserved environment variable name",
        K::EnvNameCaseConflict => "environment variable names differ only in letter case",
        K::InvalidEnvValue => "invalid environment variable value",
        K::UnsetEnvConflict => "a name in unset_env is also set by env",
    };
    Text::new(en, kind.label())
}

/// `.env` の誤りの種類の文。日本語は、conductor-core の `EnvFileErrorKind::label()`(cli に持たない)。
pub fn env_file_kind(kind: EnvFileErrorKind) -> Text {
    use EnvFileErrorKind as K;
    let en = match kind {
        K::FileMissing => "environment file does not exist",
        K::Unreadable => "cannot read environment file",
        K::TooLarge => "environment file is too large",
        K::NotUtf8 => "environment file is not UTF-8",
        K::TooManyEntries => "environment file has too many entries",
        K::InvalidName => "invalid environment variable name",
        K::ReservedName => "reserved environment variable name",
        K::CaseConflict => "environment variable names differ only in letter case",
        K::MissingEquals => "not in KEY=VALUE form",
        K::UnterminatedQuote => "unterminated quote",
        K::InvalidEscape => "invalid escape sequence",
        K::TrailingGarbage => "unexpected characters after the closing quote",
        K::NulInValue => "value contains NUL",
    };
    Text::new(en, kind.label())
}

// 次の英語の文の「1 MiB」「10,000」と、conductor-core の上限の定数が食い違えば、コンパイルできない。
const _: () = assert!(conductor_core::envfile::MAX_ENV_FILE_BYTES == 1_048_576);
const _: () = assert!(conductor_core::envfile::MAX_ENV_ENTRIES == 10_000);

/// `.env` の誤りの理由(固定の文。値を含まない)。日本語は、conductor-core の
/// `EnvFileErrorKind::reason()`(cli に持たない)。
pub fn env_file_reason(kind: EnvFileErrorKind) -> Text {
    use EnvFileErrorKind as K;
    let en = match kind {
        K::FileMissing => "the file named in `env_file` was not found",
        K::Unreadable => "the file named in `env_file` cannot be read",
        K::TooLarge => "the file is larger than the limit (1 MiB)",
        K::NotUtf8 => "write the file in UTF-8",
        K::TooManyEntries => "the number of entries exceeds the limit (10,000)",
        K::InvalidName => {
            "a name starts with an ASCII letter or `_` and has only ASCII letters, digits and `_`"
        }
        K::ReservedName => "names starting with `CONDUCTOR_` cannot be written",
        K::CaseConflict => "the file has names that differ only in letter case",
        K::MissingEquals => "write a line as `KEY=VALUE`",
        K::UnterminatedQuote => "the quote is not closed on the same line",
        K::InvalidEscape => {
            "only `\\n`, `\\r`, `\\t`, `\\\"` and `\\\\` can be escaped in double quotes"
        }
        K::TrailingGarbage => "only spaces and a comment can follow the closing quote",
        K::NulInValue => "a value cannot contain NUL (U+0000)",
    };
    Text::new(en, kind.reason())
}

/// 誤りの打ち切りの行(`.env` の誤りが 20 件を超えたとき。BR2.8)。
pub fn env_errors_more(lang: Language, rest: usize) -> String {
    match lang {
        Language::En => format!("and {rest} more"),
        Language::Ja => format!("ほか {rest} 件"),
    }
}

/// 重複キーの警告の文(名前は、エスケープして入れる)。
pub fn env_duplicate_key(lang: Language, escaped_name: &str) -> String {
    match lang {
        Language::En => format!("duplicate key `{escaped_name}` (the last one wins)"),
        Language::Ja => format!("重複キー `{escaped_name}`(後ろの値を使います)"),
    }
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

/// ステートの誤りの種類の文。日本語は、conductor-runtime の `StateErrorKind::label()`(cli に持たない)。
pub fn state_kind(kind: StateErrorKind) -> Text {
    let en = match kind {
        StateErrorKind::Unreadable => "cannot read state file",
        StateErrorKind::TooLarge => "state file is too large",
        StateErrorKind::Corrupt => "state file is corrupt",
        StateErrorKind::VersionMismatch => "state file version is not supported",
        StateErrorKind::WriteFailed => "cannot write state file",
        StateErrorKind::RemoveFailed => "cannot remove state file",
    };
    Text::new(en, kind.label())
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

/// ステートの初期化の案内。run ID があるときは `reset --run-id <ID>`、無ければ `reset`(読み取りの誤りと、
/// 定義との食い違いが使う 1 か所)。
pub fn reset_hint(context: StateContext) -> Text {
    match context {
        StateContext::ReadRunId => RESET_HINT_RUN_ID,
        _ => RESET_HINT,
    }
}

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

/// 変換の誤りの種類の文。日本語は、conductor-converter の `ConversionErrorKind::label()`(cli に持たない)。
pub fn conversion_kind(kind: ConversionErrorKind) -> Text {
    use ConversionErrorKind as K;
    let en = match kind {
        K::FileMissing => "input file does not exist",
        K::Unreadable => "cannot read input file",
        K::TooLarge => "input is too large",
        K::Syntax => "syntax error",
        K::Unrepresentable => "value cannot be represented in the target format",
        K::RootNotMap => "top level is not a map",
    };
    Text::new(en, kind.label())
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

pub fn reject_reason(reason: RejectReason) -> Text {
    match reason {
        RejectReason::OutOfOrder => Text::new(
            "earlier jobs have not finished (out of order)",
            "前のジョブが終わっていません(順序外)",
        ),
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

// ---- ログのファイル(v0.10.0 の BR5.1〜BR5.3。後ろに `: <パス>` を付ける) ----

pub const LOG_NOT_CREATED: Text =
    Text::new("cannot create the log file", "ログのファイルを作れません");
pub const LOG_WRITE_FAILED: Text = Text::new(
    "failed to write to the log file; output after that point is not in the file",
    "ログのファイルへの書き込みに失敗しました。途中からファイルに残していません",
);
pub const LOG_OUTPUT_CUT_SHORT: Text = Text::new(
    "output continued after the job ended; the rest is not in the log file",
    "ジョブの終了の後も出力が続いたため、残りを残していません",
);

// ---- -v の文言 ----

pub const EXIT_CODE: Text = Text::new("exit code", "終了コード");
pub const CONDUCTOR_EXITS_WITH: Text = Text::new("conductor exits with", "conductor の終了コード");
pub const KILLED_BY_SIGNAL: Text = Text::new("terminated by signal", "シグナルで終了");
pub const COULD_NOT_START: Text = Text::new("could not start", "起動できませんでした");
pub const INTERRUPTED_BY_SIGNAL: Text = Text::new("interrupted by signal", "シグナルで中断");
pub const NOT_STARTED: Text = Text::new("not started", "起動せず");
/// スキップ行の語と、遷移行の `(…: ID, …)` の語。
pub const SKIPPED_WORD: Text = Text::new("skipped", "スキップ");
/// 遷移によるスキップの語(`-v` のスキップ行の理由と、watch の補足)。
pub const SKIPPED_BY_TRANSITION: Text = Text::new("skipped by a transition", "遷移によりスキップ");
/// `@end` の後ろのスキップの語(`-v` のスキップ行の理由と、watch の補足)。
pub const SKIPPED_AFTER_END: Text = Text::new("skipped after @end", "@end の後ろのためスキップ");
/// 実行済みの語(`-v` のスキップ行の理由と、計画のジョブの扱い)。
pub const ALREADY_EXECUTED: Text = Text::new("already executed", "実行済み");

pub fn skip_reason(reason: SkipCause) -> Text {
    match reason {
        SkipCause::SkipTarget => SKIPPED_BY_TRANSITION,
        SkipCause::AfterEnd => SKIPPED_AFTER_END,
        SkipCause::AlreadyExecuted => ALREADY_EXECUTED,
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
/// 完了の語(watch の完了のステートの表の下に足す 1 行(FR3.1)と、計画の模擬の最後の語)。
pub const COMPLETED: Text = Text::new("completed", "完了");
pub const HEADER_JOB: Text = Text::new("JOB", "ジョブ");
pub const HEADER_STATUS: Text = Text::new("STATUS", "状態");
pub const HEADER_NOTE: Text = Text::new("NOTE", "補足");

// ---- graph(U2。BR2.8、BR3.1〜BR3.3、BR4.2、BR5.3、BR7.1) ----

/// 辺の一覧に切り替えたときの、先頭の注記の行(BR2.8)。
pub const GRAPH_EDGE_LIST_NOTE: Text = Text::new(
    "note: the graph has a shape the lane diagram cannot draw, so it is replaced by a list of edges.",
    "注記: レーンの図で描けない形のため、レーンの図の代わりに、辺の一覧を出します。",
);
/// 辺の一覧の「ジョブ」の見出し(BR2.8)。
pub const GRAPH_JOBS_HEADING: Text = Text::new("Jobs:", "ジョブ:");
/// 辺の一覧の「辺」の見出し(BR2.8)。
pub const GRAPH_EDGES_HEADING: Text = Text::new("Edges:", "辺:");
/// 辺の種類(成功と警告の両方)。成功だけ・警告だけは [`class_word`] の語。
pub const GRAPH_KIND_BOTH: Text = Text::new("success, warning", "成功・警告");
/// 再開位置の注記(ジョブの行の末尾。先頭の 2 つの空白を含む。graph の BR4.2 と計画が使う)。
pub const RESUME_HERE: Text = Text::new("  <- resume here", "  <- 再開位置");
/// 詳細の行の見出し(コマンド。graph の `-v`(BR3.1)と計画が使う)。
pub const DETAIL_COMMAND: Text = Text::new("command: ", "コマンド: ");
/// 詳細の行の見出し(環境変数。計画が使う)。
pub const DETAIL_ENV: Text = Text::new("env: ", "環境変数: ");
/// 詳細の行の見出し(リトライ。graph の `-v`(BR3.1)と計画が使う)。
pub const DETAIL_RETRY: Text = Text::new("retry: ", "リトライ: ");

/// リトライの要約(`-v`。BR3.1)。例: 「終了コード 75、76 を最大 3 回、間隔 5 秒(倍率 2、上限 60 秒)」。
pub fn graph_retry_summary(lang: Language, retry: &conductor_core::RetryPolicy) -> String {
    let codes = |sep: &str| {
        retry
            .exit_codes
            .iter()
            .map(i32::to_string)
            .collect::<Vec<_>>()
            .join(sep)
    };
    match lang {
        Language::En => {
            let mut text = format!(
                "exit codes {}, up to {} times, every {} s",
                codes(", "),
                retry.max,
                retry.interval_secs
            );
            if let Some(b) = retry.backoff {
                let (m, max) = (b.multiplier, b.max_interval_secs);
                text.push_str(&format!(" (x{m}, max {max} s)"));
            }
            text
        }
        Language::Ja => {
            let mut text = format!(
                "終了コード {} を最大 {} 回、間隔 {} 秒",
                codes("、"),
                retry.max,
                retry.interval_secs
            );
            if let Some(b) = retry.backoff {
                let (m, max) = (b.multiplier, b.max_interval_secs);
                text.push_str(&format!("(倍率 {m}、上限 {max} 秒)"));
            }
            text
        }
    }
}

/// 辺の注記の 1 つ(`<種類> → <行き先>` と、飛び越すジョブ。BR3.2)。入力の値は、エスケープ済みで渡す。
pub fn graph_edge_note(
    lang: Language,
    kind_label: &str,
    target: &str,
    skipped: &[String],
) -> String {
    match lang {
        Language::En => {
            if skipped.is_empty() {
                format!("{kind_label} -> {target}")
            } else {
                format!("{kind_label} -> {target} (skips {})", skipped.join(", "))
            }
        }
        Language::Ja => {
            if skipped.is_empty() {
                format!("{kind_label} → {target}")
            } else {
                format!("{kind_label} → {target}({} を飛び越す)", skipped.join("、"))
            }
        }
    }
}

/// 辺の注記をつなぐ区切り(BR3.2)。
pub const GRAPH_NOTE_SEPARATOR: Text = Text::new(", ", "、");

// ---- `--dry-run` の計画(U3。BR3.1〜BR3.3、BR6.1)。語句の原本は functional-spec.md の例 P1〜P8、P2e、P4e ----

/// 計画の 1 行目の前半(あとに、パイプライン名が続く)。
pub const PLAN_HEADING: Text = Text::new(
    "plan (--dry-run): pipeline ",
    "計画(--dry-run): パイプライン ",
);
/// 再開位置の行の見出し。
pub const PLAN_RESUME_LABEL: Text = Text::new("resume: ", "再開位置: ");
/// 最初から(再開位置なし)。
pub const PLAN_FROM_START: Text = Text::new("from the start", "最初から");
/// `run` の、対象のジョブの行の見出し。
pub const PLAN_TARGET_LABEL: Text = Text::new("target job: ", "対象のジョブ: ");
/// 仮定の行の見出し。
pub const PLAN_ASSUME_LABEL: Text = Text::new("assume: ", "仮定: ");
/// 使われなかった仮定の行の見出し。
pub const PLAN_UNUSED_LABEL: Text = Text::new("unused assumptions: ", "使われなかった仮定: ");
/// 模擬の結果の行の見出し。
pub const PLAN_RESULT_LABEL: Text = Text::new("result (simulated): ", "結果(模擬): ");
/// 次の動きの行の見出し(`run`)。
pub const PLAN_NEXT_LABEL: Text = Text::new("next step: ", "次の動き: ");

/// 計画の 1 行目の run ID の部分(run ID は、`RunId` の規則で、表示に安全な文字だけ)。
pub fn plan_run_id_suffix(lang: Language, run_id: &str) -> String {
    match lang {
        Language::En => format!(" (run ID: {run_id})"),
        Language::Ja => format!("(run ID: {run_id})"),
    }
}

/// 前回の停止からの再開位置(エスケープ済みのジョブ ID を渡す)。
pub fn plan_resume_after_stop(lang: Language, escaped_job_id: &str) -> String {
    match lang {
        Language::En => format!("{escaped_job_id} (previous stop)"),
        Language::Ja => format!("{escaped_job_id}(前回の停止)"),
    }
}

/// ジョブの扱いの語(括弧の中の補足を除く)。
pub fn plan_disposition(disposition: PlanDisposition) -> Text {
    match disposition {
        PlanDisposition::WillRun(_) => Text::new("run", "実行する"),
        PlanDisposition::AlreadyExecuted => ALREADY_EXECUTED,
        PlanDisposition::SkippedByTransition => Text::new("skip (transition)", "スキップ(遷移)"),
        PlanDisposition::SkippedAfterEnd => Text::new("skip (after @end)", "スキップ(@end のあと)"),
        PlanDisposition::NotReached => {
            Text::new("not run (after stop)", "実行されない(停止のあと)")
        }
    }
}

/// 結果の分類の語(計画の仮定の補足と、graph の辺の種類が使う)。
pub fn class_word(classification: Classification) -> Text {
    match classification {
        Classification::Success => Text::new("success", "成功"),
        Classification::Warning => Text::new("warning", "警告"),
        Classification::Error => Text::new("error", "エラー"),
    }
}

/// 仮定(指定、または既定の終了コード 0)の結果の補足(括弧の中。BR3.2)。
/// `retry_target` が真なら、リトライの対象の注記を続ける。
pub fn plan_assumed(
    lang: Language,
    assumed: bool,
    exit_code: i32,
    classification: Classification,
    retry_target: bool,
) -> String {
    let class = class_word(classification).get(lang);
    match lang {
        Language::En => {
            let word = if assumed { "assumed" } else { "default" };
            let mut text = format!("{word}: exit code {exit_code} -> {class}");
            if retry_target {
                text.push_str(", retry target");
            }
            text
        }
        Language::Ja => {
            let word = if assumed { "仮定" } else { "既定" };
            let mut text = format!("{word}: 終了コード {exit_code} → {class}");
            if retry_target {
                text.push_str("、リトライの対象");
            }
            text
        }
    }
}

/// 模擬の最後の語(`@end` に到達して完了)。完了は [`COMPLETED`]。
pub const PLAN_END_REACHED: Text = Text::new("reached @end (completed)", "@end に到達(完了)");

/// 模擬の最後の語(エラーで停止。エスケープ済みのジョブ ID を渡す)。
pub fn plan_stopped_at_error(lang: Language, escaped_job_id: &str) -> String {
    match lang {
        Language::En => format!("stopped (error: {escaped_job_id})"),
        Language::Ja => format!("停止(エラー: {escaped_job_id})"),
    }
}

/// `run` の次の動きの語(BR1.6)。ジョブ ID は、渡された `escape` でエスケープして出す。
pub fn plan_next_step(
    lang: Language,
    note: &NextStepNote,
    escape: impl Fn(&str) -> String,
) -> String {
    let (next, skipped) = match note {
        NextStepNote::Stopped => return Text::new("stop", "停止").get(lang).to_string(),
        NextStepNote::EndReached => {
            return Text::new(
                "reached @end (the rest are skipped when called)",
                "@end に到達(残りのジョブは、呼ばれたときにスキップ)",
            )
            .get(lang)
            .to_string();
        }
        NextStepNote::Completed => return Text::new("done", "完了").get(lang).to_string(),
        NextStepNote::NextJob {
            job_id,
            skipped_job_ids,
        } => (job_id, skipped_job_ids),
    };
    let next = escape(next);
    let escaped_skipped: Vec<String> = skipped.iter().map(|id| escape(id)).collect();
    match lang {
        Language::En if escaped_skipped.is_empty() => format!("next job {next}"),
        Language::En => format!("next job {next} (skipped: {})", escaped_skipped.join(", ")),
        Language::Ja if escaped_skipped.is_empty() => format!("次のジョブ {next}"),
        Language::Ja => format!(
            "次のジョブ {next}(スキップ: {})",
            escaped_skipped.join("、")
        ),
    }
}
