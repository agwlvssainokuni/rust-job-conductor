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

//! 表示(Reporter。BR6.1〜BR6.5、NFR6.1〜NFR6.4、NFR4.2、NFR1.4)。
//!
//! 流れ(Orchestrator)が作った [`DisplayEvent`] から、文言(英語・日本語)を組み立て、
//! 水準(`level`)に従って、標準エラー・標準出力に出す。
//!
//! - 言語(Locale)と色(Color)は、**インスタンスごとの遅延のセル**で、最初に必要になったときだけ
//!   判定する(静的な `OnceLock` は使わない。`Env` を差し替える同じプロセスのテストが、互いに
//!   影響しないように)。成功・スキップで `-v` なしなら、表示は一度もなく、どちらも判定されない。
//! - 入力に由来する文字列(ジョブID、キー名、パス、command の最初の語)は、`escape` を通す
//!   (名前(使い方の誤りの対象、遷移の表示のジョブID など)は `escape_name` で 64 文字で切る。パスと、
//!   図・計画のジョブID・コマンドの語は `escape_full` で切らない)。
//! - 標準エラーへの書き込みの失敗は、無視する(本筋を止めない)。標準出力の失敗は、値で返す。

pub mod escape;
pub mod graph_mermaid;
pub mod graph_text;
pub mod init_template;
pub(crate) mod messages;
pub mod plan_text;
mod watch;

use std::cell::OnceCell;
use std::path::Path;

use conductor_converter::ConversionError;
use conductor_core::{
    Classification, DefinitionError, EnvFileError, EnvWarning, EnvWarningKind, PipelineState,
    RejectReason, RunId, SkipCause, TransitionOn, TransitionTarget, summarize_errors,
};
use conductor_runtime::{CommandError, CreateError, MismatchKind, StateError};

use crate::deps::{Env, Sink};
use crate::platform;
use crate::types::{Language, Level, UsageError};
use escape::{escape_full, escape_name};
use messages::{
    ERROR_PREFIX, RETRY_PREFIX, VERBOSE_PREFIX, WARNING_PREFIX, command_kind, conversion_kind,
    definition_kind, env_duplicate_key, env_errors_more, env_file_kind, env_file_reason,
    mismatch_kind, reject_reason, skip_reason, state_kind, state_kind_suggests_reset, usage_kind,
};

/// ステートの誤りを表示するときの文脈(原因に添える補足を決める)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateContext {
    /// ステートを読んだ(`reset` を案内する)。
    Read,
    /// run ID つきのステートを読んだ(`reset --run-id <ID>` を案内する)。
    ReadRunId,
    /// ジョブを起動する前の失敗(ジョブは起動していない)。
    JobNotStarted,
    /// ジョブのあとの失敗(ジョブは実行済み)。
    JobExecuted,
    /// 補足なし(`reset` と `reset --all`)。
    Plain,
}

impl StateContext {
    /// ステートを読んだときの表示の文脈。run ID があるときは、案内を `reset --run-id` にする。
    pub(crate) fn for_read(run_id: Option<&RunId>) -> Self {
        match run_id {
            Some(_) => Self::ReadRunId,
            None => Self::Read,
        }
    }
}

/// ジョブの結果の補足(`-v` の `(…)` の中)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultDetail {
    /// ジョブの終了コード。
    ExitCode(i32),
    /// ジョブが、シグナルで終了した。
    Signal(i32),
    /// ジョブを起動できなかった。
    StartFailure,
    /// 中断のシグナルを受け取った(`started` は、ジョブが起動していたか)。
    Interrupted { signal: i32, started: bool },
}

/// 表示する内容(entities の DisplayEvent。種類とデータ)。
#[derive(Debug, Clone)]
pub enum DisplayEvent<'a> {
    // ---- error(標準エラー。常に) ----
    DefinitionErrors(&'a [DefinitionError]),
    /// `.env` の誤り(20 件で打ち切り、残りは件数。値は含めない。BR2.8)。
    EnvFileErrors(&'a [EnvFileError]),
    /// `.env` の警告(重複キー。`validate` と `graph` が出す(`check_definition_and_env`)。BR7.2)。
    EnvWarnings(&'a [EnvWarning]),
    CommandErrors {
        definition: &'a Path,
        errors: &'a [(String, CommandError)],
    },
    Usage(&'a UsageError),
    State {
        error: &'a StateError,
        context: StateContext,
    },
    ConsistencyMismatch {
        state_path: &'a Path,
        kind: MismatchKind,
        /// 読んだときの文脈。`Read` か `ReadRunId`(run ID つきのステートなら、案内の文が変わる)。
        context: StateContext,
    },
    /// `reset --all` が、読めなくて残したファイル(警告。終了コードは変わらない)。
    ResetAllKept {
        names: &'a [String],
    },
    Rejected {
        job_id: &'a str,
        reason: RejectReason,
    },
    StartFailure {
        job_id: &'a str,
        command: &'a str,
    },
    SuccessCodeNotListed {
        job_id: &'a str,
    },
    Conversion(&'a ConversionError),
    /// 標準出力に書けなかった(`watch`・`graph`・`--dry-run` は 65、`convert` は 67 で終わる)。
    OutputFailed,
    CompletionUndetermined {
        state_path: &'a Path,
    },
    /// シグナルの準備に失敗した(固定の英語の文。終了コード 70)。
    SignalSetupFailed,
    /// `init` の作る先に、ファイルが既にある(終了コード 69。U4 の BR1.2)。
    InitExists {
        path: &'a Path,
    },
    /// `init` が、ファイルを作れなかった(`already_exists` 以外。終了コード 65。BR3.2)。
    InitFailed {
        path: &'a Path,
        error: CreateError,
    },
    /// `init --config -` で、ひな形を標準出力に書けなかった(終了コード 65。BR1.5)。
    InitStdoutFailed,
    // ---- notice(標準エラー。常に。エラーではない知らせ) ----
    /// リトライの前の知らせ(BR2.4、BR2.15)。待機の前に出す。
    RetryScheduled {
        job_id: &'a str,
        /// 直前の実行の終了コード。
        last_exit_code: i32,
        /// 何回目のリトライか(1 から)。
        retry_number: u32,
        /// リトライの最大回数。
        max: u32,
        /// 待ち時間(秒)。
        wait_secs: u64,
    },
    /// `init` が、ファイルを作った(BR1.4)。
    InitCreated {
        path: &'a Path,
    },
    /// `init --force` が、既にあったファイルを置き換えた(BR1.4)。
    InitReplaced {
        path: &'a Path,
    },
    // ---- verbose(標準エラー。`-v` のとき) ----
    JobResult {
        job_id: &'a str,
        /// 分類。`None` は、起動しなかった中断(表示は `success`・`warning`・`error`・`interrupted` のトークン)。
        result: Option<Classification>,
        detail: ResultDetail,
        /// `conductor` が返す終了コード(範囲を合わせたあと)。
        conductor_exit: i32,
    },
    JobSkipped {
        job_id: &'a str,
        reason: SkipCause,
    },
    Transition {
        /// 遷移を引いた結果(表示は `success` か `warning`)。
        on: TransitionOn,
        /// 遷移先(ジョブID、または `@end`。表示は [`TransitionTarget::label`] をエスケープしたもの)。
        target: &'a TransitionTarget,
        skipped: &'a [String],
    },
    Interrupted {
        signal: i32,
    },
    /// `validate` が通った(ジョブの数)。
    DefinitionValid {
        job_count: usize,
    },
    // ---- output(標準出力) ----
    WatchTable(&'a PipelineState),
    WatchEmpty,
    /// `graph` の図(組み立て済みの text か Mermaid。そのまま出す。BR6.3)。
    Graph(&'a str),
    /// `--dry-run` の計画(組み立て済み。そのまま出す。U3 の BR3.4)。
    Plan(&'a str),
    /// `init --config -` のひな形(組み立て済み。そのまま出す。BR1.5)。
    InitTemplate(&'a str),
    /// 使い方(`--help`)。
    Help,
    /// 版(`--version`)。
    Version,
}

impl DisplayEvent<'_> {
    /// 水準(出力先と、表示の条件を決める)。
    pub fn level(&self) -> Level {
        match self {
            Self::DefinitionErrors(_)
            | Self::EnvFileErrors(_)
            | Self::EnvWarnings(_)
            | Self::CommandErrors { .. }
            | Self::Usage(_)
            | Self::State { .. }
            | Self::ConsistencyMismatch { .. }
            | Self::ResetAllKept { .. }
            | Self::Rejected { .. }
            | Self::StartFailure { .. }
            | Self::SuccessCodeNotListed { .. }
            | Self::Conversion(_)
            | Self::OutputFailed
            | Self::SignalSetupFailed
            | Self::InitExists { .. }
            | Self::InitFailed { .. }
            | Self::InitStdoutFailed
            | Self::CompletionUndetermined { .. } => Level::Error,
            Self::RetryScheduled { .. } | Self::InitCreated { .. } | Self::InitReplaced { .. } => {
                Level::Notice
            }
            Self::JobResult { .. }
            | Self::JobSkipped { .. }
            | Self::Transition { .. }
            | Self::Interrupted { .. }
            | Self::DefinitionValid { .. } => Level::Verbose,
            Self::WatchTable(_)
            | Self::WatchEmpty
            | Self::Graph(_)
            | Self::Plan(_)
            | Self::InitTemplate(_)
            | Self::Help
            | Self::Version => Level::Output,
        }
    }
}

/// 表示の組み立てと出力。言語と色は、最初に必要になったときだけ判定する。
pub struct Reporter<'a> {
    env: &'a dyn Env,
    language: OnceCell<Language>,
    color: OnceCell<bool>,
    verbose: bool,
}

impl<'a> Reporter<'a> {
    pub fn new(env: &'a dyn Env, verbose: bool) -> Self {
        Self {
            env,
            language: OnceCell::new(),
            color: OnceCell::new(),
            verbose,
        }
    }

    /// 表示の言語(BR6.1)。`LC_ALL`、`LC_MESSAGES`、`LANG` の順に、最初に空でない値が `ja` で
    /// 始まれば日本語。最初の呼び出しで判定し、結果を保つ。
    pub fn language(&self) -> Language {
        *self.language.get_or_init(|| detect_language(self.env))
    }

    /// 色を使うか(BR6.5)。標準出力が端末で、UNIX 系で、`NO_COLOR` が未設定または空のときだけ。
    pub fn color_enabled(&self) -> bool {
        *self.color.get_or_init(|| {
            platform::color_supported()
                && self.env.var("NO_COLOR").is_none_or(|v| v.is_empty())
                && self.env.stdout_is_terminal()
        })
    }

    /// 水準に従って出す。`error` と `notice` は常に、`verbose` は `-v` のときだけ標準エラーへ(書き込みの失敗は
    /// 無視)。`output` は標準出力へ。戻り値は、`output` の書き込みの結果(失敗は `false`)。
    pub fn show(&self, sink: &mut dyn Sink, event: &DisplayEvent<'_>) -> bool {
        match event.level() {
            Level::Error | Level::Notice => {
                let _ = sink.write_err(self.render(event).as_bytes());
                true
            }
            Level::Verbose => {
                if self.verbose {
                    let _ = sink.write_err(self.render(event).as_bytes());
                }
                true
            }
            Level::Output => sink.write_out(self.render(event).as_bytes()).is_ok(),
        }
    }

    /// 文言を組み立てる(最後の改行を含む。複数の誤りは 1 行ずつ)。
    pub fn render(&self, event: &DisplayEvent<'_>) -> String {
        // 言語は、必要な枝だけで判定する(`language` は 1 回だけ判定して保つ)。
        let lang = || self.language();
        match event {
            // ロケールを判定しない、固定の文(パニックのフックと同じ。BR6.1 の差分)。
            DisplayEvent::SignalSetupFailed => format!("{}\n", messages::SIGNAL_SETUP_FAILED),
            DisplayEvent::Version => format!("conductor {}\n", env!("CARGO_PKG_VERSION")),
            DisplayEvent::DefinitionErrors(errors) => {
                let lang = lang();
                errors.iter().map(|e| definition_line(lang, e)).collect()
            }
            DisplayEvent::EnvFileErrors(errors) => env_file_lines(lang(), errors),
            DisplayEvent::EnvWarnings(warnings) => {
                let lang = lang();
                warnings.iter().map(|w| env_warning_line(lang, w)).collect()
            }
            DisplayEvent::CommandErrors { definition, errors } => {
                let lang = lang();
                let place = show_path(definition);
                errors
                    .iter()
                    .map(|(job_id, error)| command_line(lang, &place, job_id, error))
                    .collect()
            }
            DisplayEvent::Usage(error) => usage_text(lang(), error),
            DisplayEvent::State { error, context } => state_line(lang(), error, *context),
            DisplayEvent::ResetAllKept { names } => {
                let mut out = format!("{WARNING_PREFIX}{}\n", messages::RESET_ALL_KEPT.get(lang()));
                for name in names.iter() {
                    out.push_str(&format!("  {}\n", escape_full(name)));
                }
                out
            }
            DisplayEvent::ConsistencyMismatch {
                state_path,
                kind,
                context,
            } => {
                let lang = lang();
                let place = show_path(state_path);
                let hint = messages::reset_hint(*context).get(lang);
                let what = mismatch_kind(*kind).get(lang);
                error_line(
                    &place,
                    messages::MISMATCH.get(lang),
                    Some(&format!("{what}; {hint}")),
                )
            }
            DisplayEvent::Rejected { job_id, reason } => {
                let lang = lang();
                let place = job_place(job_id);
                error_line(
                    &place,
                    messages::REJECTED.get(lang),
                    Some(reject_reason(*reason).get(lang)),
                )
            }
            DisplayEvent::StartFailure { job_id, command } => {
                let place = job_place(job_id);
                let sentence = format!(
                    "{} '{}'",
                    messages::CANNOT_START.get(lang()),
                    escape_name(command)
                );
                error_line(&place, &sentence, None)
            }
            DisplayEvent::SuccessCodeNotListed { job_id } => {
                let place = job_place(job_id);
                error_line(&place, messages::SUCCESS_CODE_NOT_LISTED.get(lang()), None)
            }
            DisplayEvent::Conversion(error) => conversion_line(lang(), error),
            DisplayEvent::OutputFailed => {
                format!("{ERROR_PREFIX}{}\n", messages::OUTPUT_FAILED.get(lang()))
            }
            DisplayEvent::CompletionUndetermined { state_path } => {
                let place = show_path(state_path);
                error_line(&place, messages::COMPLETION_UNDETERMINED.get(lang()), None)
            }
            DisplayEvent::RetryScheduled {
                job_id,
                last_exit_code,
                retry_number,
                max,
                wait_secs,
            } => format!(
                "{RETRY_PREFIX}{}\n",
                messages::retry_scheduled(
                    lang(),
                    &escape_name(job_id),
                    *last_exit_code,
                    *retry_number,
                    *max,
                    *wait_secs
                )
            ),
            DisplayEvent::JobResult {
                job_id,
                result,
                detail,
                conductor_exit,
            } => job_result_line(lang(), job_id, *result, *detail, *conductor_exit),
            DisplayEvent::JobSkipped { job_id, reason } => format!(
                "{VERBOSE_PREFIX}job={} {} ({})\n",
                escape_name(job_id),
                messages::SKIPPED_WORD.get(lang()),
                skip_reason(*reason).get(lang())
            ),
            DisplayEvent::Transition {
                on,
                target,
                skipped,
            } => {
                let mut line = format!(
                    "{VERBOSE_PREFIX}transition: {} -> {}",
                    on.as_str(),
                    escape_name(target.label())
                );
                if !skipped.is_empty() {
                    let ids: Vec<String> = skipped.iter().map(|s| escape_name(s)).collect();
                    line.push_str(&format!(
                        " ({}: {})",
                        messages::SKIPPED_WORD.get(lang()),
                        ids.join(", ")
                    ));
                }
                line.push('\n');
                line
            }
            DisplayEvent::Interrupted { signal } => format!(
                "{VERBOSE_PREFIX}{} {signal}\n",
                messages::INTERRUPTED_BY_SIGNAL.get(lang())
            ),
            DisplayEvent::DefinitionValid { job_count } => format!(
                "{VERBOSE_PREFIX}{}\n",
                messages::definition_valid(lang(), *job_count)
            ),
            DisplayEvent::WatchTable(state) => {
                watch::render_table(lang(), state, self.color_enabled())
            }
            DisplayEvent::WatchEmpty => format!("{}\n", messages::WATCH_EMPTY.get(lang())),
            DisplayEvent::InitCreated { path } => {
                format!("{}\n", messages::init_created(lang(), &show_path(path)))
            }
            DisplayEvent::InitReplaced { path } => {
                format!("{}\n", messages::init_replaced(lang(), &show_path(path)))
            }
            DisplayEvent::InitExists { path } => format!(
                "{ERROR_PREFIX}{}\n",
                messages::init_exists(lang(), &show_path(path))
            ),
            DisplayEvent::InitFailed { path, error } => format!(
                "{ERROR_PREFIX}{}\n",
                messages::init_cannot_create(lang(), *error, &show_path(path))
            ),
            DisplayEvent::InitStdoutFailed => {
                format!(
                    "{ERROR_PREFIX}{}\n",
                    messages::INIT_STDOUT_FAILED.get(lang())
                )
            }
            DisplayEvent::Graph(text)
            | DisplayEvent::Plan(text)
            | DisplayEvent::InitTemplate(text) => (*text).to_string(),
            DisplayEvent::Help => messages::USAGE.get(lang()).to_string(),
        }
    }
}

/// パスの表示(制御文字をエスケープする。切らない。定義ファイルのパスと同じ扱い)。
fn show_path(path: &Path) -> String {
    escape_full(&path.display().to_string())
}

/// 環境のロケールから、言語を決める(BR6.1)。
fn detect_language(env: &dyn Env) -> Language {
    for name in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        match env.var(name) {
            Some(value) if !value.is_empty() => {
                return if value.starts_with("ja") {
                    Language::Ja
                } else {
                    Language::En
                };
            }
            _ => {}
        }
    }
    Language::En
}

/// `conductor: error: <場所>: <種類の文>[: <原因>]` の 1 行。
fn error_line(place: &str, sentence: &str, cause: Option<&str>) -> String {
    match cause {
        Some(cause) => format!("{ERROR_PREFIX}{place}: {sentence}: {cause}\n"),
        None => format!("{ERROR_PREFIX}{place}: {sentence}\n"),
    }
}

/// `<ファイル>[:<行>]`(ファイル名はエスケープする。切らない)。
fn place_of(file: &str, line: Option<usize>) -> String {
    let file = escape_full(file);
    match line {
        Some(line) => format!("{file}:{line}"),
        None => file,
    }
}

fn definition_line(lang: Language, e: &DefinitionError) -> String {
    let place = place_of(&e.file, e.line);
    let reason = escape_full(&e.reason);
    error_line(&place, definition_kind(e.kind).get(lang), Some(&reason))
}

/// `.env` の誤りの行(最大 20 件。残りは「ほか N 件」。値は含めない。BR2.8、BR8.1)。
fn env_file_lines(lang: Language, errors: &[EnvFileError]) -> String {
    let (shown, rest) = summarize_errors(errors);
    let mut out = String::new();
    for e in shown {
        let place = place_of(&e.file, e.line);
        out.push_str(&error_line(
            &place,
            env_file_kind(e.kind).get(lang),
            Some(env_file_reason(e.kind).get(lang)),
        ));
    }
    if rest > 0 {
        out.push_str(&format!("{ERROR_PREFIX}{}\n", env_errors_more(lang, rest)));
    }
    out
}

/// `conductor: warning: <ファイル>:<行>: <文>` の 1 行。
fn env_warning_line(lang: Language, w: &EnvWarning) -> String {
    let EnvWarningKind::DuplicateKey = w.kind;
    format!(
        "{WARNING_PREFIX}{}: {}\n",
        place_of(&w.file, Some(w.line)),
        env_duplicate_key(lang, &escape_name(&w.name))
    )
}

/// 1 行を足す(末尾の空白を取り除き、改行を足す)。graph・計画・watch の整形が使う。
pub(crate) fn push_line(out: &mut String, line: &str) {
    out.push_str(line.trim_end());
    out.push('\n');
}

/// ジョブの場所 `job '<ID>'`(ジョブ ID はエスケープする)。
fn job_place(job_id: &str) -> String {
    format!("job '{}'", escape_name(job_id))
}

fn command_line(lang: Language, definition: &str, job_id: &str, e: &CommandError) -> String {
    let place = format!("{definition}: {}", job_place(job_id));
    let sentence = command_kind(e.kind).get(lang);
    error_line(&place, sentence, None)
}

fn usage_text(lang: Language, e: &UsageError) -> String {
    let sentence = usage_kind(e.kind).get(lang);
    let mut out = match &e.subject {
        Some(subject) => format!("{ERROR_PREFIX}{sentence}: '{}'\n", escape_name(subject)),
        None => format!("{ERROR_PREFIX}{sentence}\n"),
    };
    out.push('\n');
    out.push_str(messages::USAGE.get(lang));
    out
}

fn state_line(lang: Language, e: &StateError, context: StateContext) -> String {
    let place = show_path(&e.path);
    let mut cause = escape_full(e.reason);
    match context {
        StateContext::Read | StateContext::ReadRunId => {
            if state_kind_suggests_reset(e.kind) {
                let hint = messages::reset_hint(context);
                cause.push_str(&format!("; {}", hint.get(lang)));
            }
        }
        StateContext::JobNotStarted => {
            cause.push_str(&format!(" ({})", messages::JOB_NOT_STARTED.get(lang)));
        }
        StateContext::JobExecuted => {
            cause.push_str(&format!(" ({})", messages::JOB_EXECUTED.get(lang)));
        }
        StateContext::Plain => {}
    }
    error_line(&place, state_kind(e.kind).get(lang), Some(&cause))
}

fn conversion_line(lang: Language, e: &ConversionError) -> String {
    let place = place_of(&e.file, e.line);
    let mut cause = escape_full(e.reason);
    if let Some(path) = &e.key_path {
        cause.push_str(&format!(
            " ({}: {})",
            messages::AT_KEY.get(lang),
            escape_full(path)
        ));
    }
    error_line(&place, conversion_kind(e.kind).get(lang), Some(&cause))
}

fn job_result_line(
    lang: Language,
    job_id: &str,
    result: Option<Classification>,
    detail: ResultDetail,
    conductor_exit: i32,
) -> String {
    let id = escape_name(job_id);
    let inner = match detail {
        ResultDetail::ExitCode(code) => {
            let mut text = format!("{} {code}", messages::EXIT_CODE.get(lang));
            if result == Some(Classification::Error) && code != conductor_exit {
                text.push_str(&format!(
                    ", {} {conductor_exit}",
                    messages::CONDUCTOR_EXITS_WITH.get(lang)
                ));
            }
            text
        }
        ResultDetail::Signal(n) => format!("{} {n}", messages::KILLED_BY_SIGNAL.get(lang)),
        ResultDetail::StartFailure => messages::COULD_NOT_START.get(lang).to_string(),
        ResultDetail::Interrupted { signal, started } => {
            let mut text = format!("{} {signal}", messages::INTERRUPTED_BY_SIGNAL.get(lang));
            if !started {
                text.push_str(&format!(", {}", messages::NOT_STARTED.get(lang)));
            }
            text
        }
    };
    let result = result.map_or("interrupted", Classification::as_str);
    format!("{VERBOSE_PREFIX}job={id} result={result} ({inner})\n")
}
