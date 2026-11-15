//! 表示(Reporter。BR6.1〜BR6.5、NFR6.1〜NFR6.4、NFR4.2、NFR1.4)。
//!
//! 流れ(Orchestrator)が作った [`DisplayEvent`] から、文言(英語・日本語)を組み立て、
//! 水準(`level`)に従って、標準エラー・標準出力に出す。
//!
//! - 言語(Locale)と色(Color)は、**インスタンスごとの遅延のセル**で、最初に必要になったときだけ
//!   判定する(静的な `OnceLock` は使わない。`Env` を差し替える同じプロセスのテストが、互いに
//!   影響しないように)。成功・スキップで `-v` なしなら、表示は一度もなく、どちらも判定されない。
//! - 入力に由来する文字列(ジョブID、キー名、パス、command の最初の語)は、`escape` を通す
//!   (名前は 64 文字で切り、パスは切らない)。
//! - 標準エラーへの書き込みの失敗は、無視する(本筋を止めない)。標準出力の失敗は、値で返す。

pub mod escape;
pub mod messages;
pub mod watch;

use std::cell::OnceCell;
use std::path::Path;

use conductor_converter::ConversionError;
use conductor_core::{DecisionReason, DefinitionError, PipelineState};
use conductor_runtime::{CommandError, MismatchKind, StateError};

use crate::deps::{Env, Sink};
use crate::platform;
use crate::types::{Language, Level, UsageError};
use escape::{escape_name, escape_path};
use messages::{
    ERROR_PREFIX, VERBOSE_PREFIX, command_kind, conversion_kind, definition_kind, mismatch_kind,
    reject_reason, skip_reason, state_kind, state_kind_suggests_reset, usage_kind,
};

/// ステートの誤りを表示するときの文脈(原因に添える補足を決める)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateContext {
    /// ステートを読んだ(`reset` を案内する)。
    Read,
    /// ジョブを起動する前の失敗(ジョブは起動していない)。
    JobNotStarted,
    /// ジョブのあとの失敗(ジョブは実行済み)。
    JobExecuted,
    /// 補足なし(`reset` コマンド、`watch`)。
    Plain,
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
    },
    Rejected {
        job_id: &'a str,
        reason: DecisionReason,
    },
    StartFailure {
        job_id: &'a str,
        command: &'a str,
    },
    SuccessCodeNotListed {
        job_id: &'a str,
    },
    Conversion(&'a ConversionError),
    /// 標準出力に書けなかった(`watch` は 65、`convert` は 67 で終わる)。
    OutputFailed,
    CompletionUndetermined {
        state_path: &'a Path,
    },
    /// シグナルの準備に失敗した(固定の英語の文。終了コード 70)。
    SignalSetupFailed,
    // ---- verbose(標準エラー。`-v` のとき) ----
    JobResult {
        job_id: &'a str,
        /// `success`・`warning`・`error`・`interrupted`(トークン)。
        result: &'static str,
        detail: ResultDetail,
        /// `conductor` が返す終了コード(範囲を合わせたあと)。
        conductor_exit: i32,
    },
    JobSkipped {
        job_id: &'a str,
        reason: DecisionReason,
    },
    Transition {
        /// `success` か `warning`。
        on: &'static str,
        /// 遷移先のジョブID、または `@end`。
        target: &'a str,
        skipped: &'a [String],
    },
    Interrupted {
        signal: i32,
    },
    // ---- output(標準出力) ----
    WatchTable(&'a PipelineState),
    WatchEmpty,
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
            | Self::CommandErrors { .. }
            | Self::Usage(_)
            | Self::State { .. }
            | Self::ConsistencyMismatch { .. }
            | Self::Rejected { .. }
            | Self::StartFailure { .. }
            | Self::SuccessCodeNotListed { .. }
            | Self::Conversion(_)
            | Self::OutputFailed
            | Self::SignalSetupFailed
            | Self::CompletionUndetermined { .. } => Level::Error,
            Self::JobResult { .. }
            | Self::JobSkipped { .. }
            | Self::Transition { .. }
            | Self::Interrupted { .. } => Level::Verbose,
            Self::WatchTable(_) | Self::WatchEmpty | Self::Help | Self::Version => Level::Output,
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

    /// 言語の判定を、すでに行ったか(遅延のテスト用)。
    pub fn language_resolved(&self) -> bool {
        self.language.get().is_some()
    }

    /// 色の判定を、すでに行ったか(遅延のテスト用)。
    pub fn color_resolved(&self) -> bool {
        self.color.get().is_some()
    }

    /// 水準に従って出す。`error` は常に、`verbose` は `-v` のときだけ標準エラーへ(書き込みの失敗は
    /// 無視)。`output` は標準出力へ。戻り値は、`output` の書き込みの結果(失敗は `false`)。
    pub fn show(&self, sink: &mut dyn Sink, event: &DisplayEvent<'_>) -> bool {
        match event.level() {
            Level::Error => {
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
        // ロケールを判定しない、固定の文(パニックのフックと同じ。BR6.1 の差分)。
        match event {
            DisplayEvent::SignalSetupFailed => {
                return format!("{}\n", messages::SIGNAL_SETUP_FAILED);
            }
            DisplayEvent::Version => return format!("conductor {}\n", env!("CARGO_PKG_VERSION")),
            _ => {}
        }
        let lang = self.language();
        match event {
            DisplayEvent::DefinitionErrors(errors) => {
                errors.iter().map(|e| definition_line(lang, e)).collect()
            }
            DisplayEvent::CommandErrors { definition, errors } => {
                let place = escape_path(&definition.display().to_string());
                errors
                    .iter()
                    .map(|(job_id, error)| command_line(lang, &place, job_id, error))
                    .collect()
            }
            DisplayEvent::Usage(error) => usage_text(lang, error),
            DisplayEvent::State { error, context } => state_line(lang, error, *context),
            DisplayEvent::ConsistencyMismatch { state_path, kind } => {
                let place = escape_path(&state_path.display().to_string());
                let hint = messages::RESET_HINT.get(lang);
                let what = mismatch_kind(*kind).get(lang);
                error_line(
                    &place,
                    messages::MISMATCH.get(lang),
                    Some(&format!("{what}; {hint}")),
                )
            }
            DisplayEvent::Rejected { job_id, reason } => {
                let place = format!("job '{}'", escape_name(job_id));
                error_line(
                    &place,
                    messages::REJECTED.get(lang),
                    Some(reject_reason(*reason).get(lang)),
                )
            }
            DisplayEvent::StartFailure { job_id, command } => {
                let place = format!("job '{}'", escape_name(job_id));
                let sentence = format!(
                    "{} '{}'",
                    messages::CANNOT_START.get(lang),
                    escape_name(command)
                );
                error_line(&place, &sentence, None)
            }
            DisplayEvent::SuccessCodeNotListed { job_id } => {
                let place = format!("job '{}'", escape_name(job_id));
                error_line(&place, messages::SUCCESS_CODE_NOT_LISTED.get(lang), None)
            }
            DisplayEvent::Conversion(error) => conversion_line(lang, error),
            DisplayEvent::OutputFailed => {
                format!("{ERROR_PREFIX}{}\n", messages::OUTPUT_FAILED.get(lang))
            }
            DisplayEvent::CompletionUndetermined { state_path } => {
                let place = escape_path(&state_path.display().to_string());
                error_line(&place, messages::COMPLETION_UNDETERMINED.get(lang), None)
            }
            DisplayEvent::JobResult {
                job_id,
                result,
                detail,
                conductor_exit,
            } => job_result_line(lang, job_id, result, *detail, *conductor_exit),
            DisplayEvent::JobSkipped { job_id, reason } => format!(
                "{VERBOSE_PREFIX}job={} {} ({})\n",
                escape_name(job_id),
                messages::SKIPPED_WORD.get(lang),
                skip_reason(*reason).get(lang)
            ),
            DisplayEvent::Transition {
                on,
                target,
                skipped,
            } => {
                let mut line = format!(
                    "{VERBOSE_PREFIX}transition: {on} -> {}",
                    escape_name(target)
                );
                if !skipped.is_empty() {
                    let ids: Vec<String> = skipped.iter().map(|s| escape_name(s)).collect();
                    line.push_str(&format!(
                        " ({}: {})",
                        messages::SKIPPED_LIST.get(lang),
                        ids.join(", ")
                    ));
                }
                line.push('\n');
                line
            }
            DisplayEvent::Interrupted { signal } => format!(
                "{VERBOSE_PREFIX}{} {signal}\n",
                messages::INTERRUPTED_BY_SIGNAL.get(lang)
            ),
            DisplayEvent::WatchTable(state) => {
                watch::render_table(lang, state, self.color_enabled())
            }
            DisplayEvent::WatchEmpty => format!("{}\n", messages::WATCH_EMPTY.get(lang)),
            DisplayEvent::Help => messages::USAGE.get(lang).to_string(),
            // 上で返している。
            DisplayEvent::SignalSetupFailed | DisplayEvent::Version => String::new(),
        }
    }
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

fn definition_line(lang: Language, e: &DefinitionError) -> String {
    let file = escape_path(&e.file);
    let place = match e.line {
        Some(line) => format!("{file}:{line}"),
        None => file,
    };
    let reason = escape_path(&e.reason);
    error_line(&place, &definition_kind(lang, e.kind), Some(&reason))
}

fn command_line(lang: Language, definition: &str, job_id: &str, e: &CommandError) -> String {
    let place = format!("{definition}: job '{}'", escape_name(job_id));
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
    let place = escape_path(&e.path.display().to_string());
    let mut cause = escape_path(e.reason);
    match context {
        StateContext::Read => {
            if state_kind_suggests_reset(e.kind) {
                cause.push_str(&format!("; {}", messages::RESET_HINT.get(lang)));
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
    let file = escape_path(&e.file);
    let place = match e.line {
        Some(line) => format!("{file}:{line}"),
        None => file,
    };
    let mut cause = escape_path(&e.reason);
    if let Some(path) = &e.key_path {
        cause.push_str(&format!(
            " ({}: {})",
            messages::AT_KEY.get(lang),
            escape_path(path)
        ));
    }
    error_line(&place, &conversion_kind(lang, e.kind), Some(&cause))
}

fn job_result_line(
    lang: Language,
    job_id: &str,
    result: &str,
    detail: ResultDetail,
    conductor_exit: i32,
) -> String {
    let id = escape_name(job_id);
    let inner = match detail {
        ResultDetail::ExitCode(code) => {
            let mut text = format!("{} {code}", messages::EXIT_CODE.get(lang));
            if result == "error" && code != conductor_exit {
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
    format!("{VERBOSE_PREFIX}job={id} result={result} ({inner})\n")
}
