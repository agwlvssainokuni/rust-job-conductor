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
//! - 入力に由来する文字列(ジョブID、キー名、パス、command の最初の語)は、表示の安全のエスケープ
//!   (Escape。NFR4.2。端末に出せる形にする)を通す
//!   (名前(使い方の誤りの対象、遷移の表示のジョブID など)は `escape_name` で 64 文字で切る。パスと、
//!   図・計画のジョブID・コマンドの語は `escape_full` で切らない)。
//!   2 つは、このモジュールにあり、下位のモジュール(計画・watch・図の整形)も使う。本体は
//!   conductor-core の `escape_display` で、ここは名前・パスの薄い包みだけを持つ。conductor-converter は
//!   core に依存しないため、同じ規則の写しを持ち、単体テストの `src/tests/escape_table.rs` と converter の
//!   単体テストの、同じ期待値の表で揃える。
//! - 標準エラーへの書き込みの失敗は、無視する(本筋を止めない)。標準出力の失敗は、値で返す。

mod escape;
mod event;
pub mod graph_dot;
pub mod graph_mermaid;
pub mod graph_text;
pub mod init_template;
mod lines;
pub(crate) mod messages;
pub mod plan_text;
mod watch;

pub(crate) use escape::{escape_full, escape_name, push_line};
pub use event::{DisplayEvent, FramePrefix, ResultDetail, StateContext};

use std::cell::OnceCell;

use crate::deps::{Env, Sink};
use crate::platform;
use crate::types::{Language, Level};
use lines::{
    command_line, conversion_line, definition_line, env_file_lines, env_warning_line, error_line,
    job_place, job_result_line, show_path, state_line, usage_text,
};
use messages::{
    ERROR_PREFIX, RETRY_PREFIX, VERBOSE_PREFIX, WARNING_PREFIX, mismatch_kind, reject_reason,
    skip_reason,
};

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
            DisplayEvent::WatchTable { state, now } => {
                watch::render_table(lang(), state, self.color_enabled(), *now)
            }
            DisplayEvent::WatchEmpty => format!("{}\n", messages::WATCH_EMPTY.get(lang())),
            DisplayEvent::WatchFrame { prefix, state } => {
                let body = match state {
                    Some((state, now)) => DisplayEvent::WatchTable { state, now: *now },
                    None => DisplayEvent::WatchEmpty,
                };
                format!("{}{}", prefix.text(), self.render(&body))
            }
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
            DisplayEvent::LogNotCreated { path } => format!(
                "{ERROR_PREFIX}{}: {}\n",
                messages::LOG_NOT_CREATED.get(lang()),
                show_path(path)
            ),
            DisplayEvent::LogWriteFailed { path } => format!(
                "{WARNING_PREFIX}{}: {}\n",
                messages::LOG_WRITE_FAILED.get(lang()),
                show_path(path)
            ),
            DisplayEvent::LogOutputCutShort { path } => format!(
                "{WARNING_PREFIX}{}: {}\n",
                messages::LOG_OUTPUT_CUT_SHORT.get(lang()),
                show_path(path)
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
