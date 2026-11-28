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
//!   2 つは `escape.rs` にあり、このモジュールが今の名前で出し、下位のモジュール(計画・watch・図の整形)も
//!   使う。本体は conductor-core の `escape_display` で、ここは名前・パスの薄い包みだけを持つ。
//!   conductor-converter は core に依存しないため、同じ規則の写しを持ち、単体テストの `src/tests/escape_table.rs` と converter の
//!   単体テストの、同じ期待値の表で揃える。
//! - 標準エラーへの書き込みの失敗は、無視する(本筋を止めない)。標準出力の失敗は、値で返す。
//!
//! このモジュールには、`Reporter` の本体(言語・色の判定、水準に従って出す `show`、変種ごとに振り分ける
//! `render`)だけを置く。中身は子のモジュールに分ける: `event.rs`(表示する出来事の型と水準。
//! `pub use` で今の名前のまま出す)、`render.rs`(変種ごとの組み立て。表示の形の節に並べる)、
//! `lines.rs`(1 行の組み立て)、`escape.rs`(エスケープと `push_line`)。

mod escape;
mod event;
pub mod graph_dot;
pub mod graph_mermaid;
pub mod graph_text;
pub mod init_template;
mod lines;
pub(crate) mod messages;
pub mod plan_text;
mod render;
mod watch;

pub(crate) use escape::{escape_full, escape_name, push_line};
pub use event::{DisplayEvent, FramePrefix, ResultDetail, StateContext};

use std::cell::OnceCell;

use crate::deps::{Env, Sink};
use crate::platform;
use crate::types::{Language, Level};

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
    ///
    /// 変種ごとに `render.rs` のメソッドを 1 つ呼ぶだけ。言語は、必要なメソッドだけで判定する
    /// (`language` は 1 回だけ判定して保つ)。
    pub fn render(&self, event: &DisplayEvent<'_>) -> String {
        match event {
            DisplayEvent::SignalSetupFailed => self.signal_setup_failed(),
            DisplayEvent::Version => self.version(),
            DisplayEvent::DefinitionErrors(errors) => self.definition_errors(errors),
            DisplayEvent::EnvFileErrors(errors) => self.env_file_errors(errors),
            DisplayEvent::EnvWarnings(warnings) => self.env_warnings(warnings),
            DisplayEvent::CommandErrors { definition, errors } => {
                self.command_errors(definition, errors)
            }
            DisplayEvent::Usage(error) => self.usage(error),
            DisplayEvent::State { error, context } => self.state(error, *context),
            DisplayEvent::ResetAllKept { names } => self.reset_all_kept(names),
            DisplayEvent::ConsistencyMismatch {
                state_path,
                kind,
                context,
            } => self.consistency_mismatch(state_path, *kind, *context),
            DisplayEvent::Rejected { job_id, reason } => self.rejected(job_id, *reason),
            DisplayEvent::StartFailure { job_id, command } => self.start_failure(job_id, command),
            DisplayEvent::SuccessCodeNotListed { job_id } => self.success_code_not_listed(job_id),
            DisplayEvent::Conversion(error) => self.conversion(error),
            DisplayEvent::OutputFailed => self.output_failed(),
            DisplayEvent::CompletionUndetermined { state_path } => {
                self.completion_undetermined(state_path)
            }
            DisplayEvent::RetryScheduled {
                job_id,
                last_exit_code,
                retry_number,
                max,
                wait_secs,
            } => self.retry_scheduled(job_id, *last_exit_code, *retry_number, *max, *wait_secs),
            DisplayEvent::JobResult {
                job_id,
                result,
                detail,
                conductor_exit,
            } => self.job_result(job_id, *result, *detail, *conductor_exit),
            DisplayEvent::JobSkipped { job_id, reason } => self.job_skipped(job_id, *reason),
            DisplayEvent::Transition {
                on,
                target,
                skipped,
            } => self.transition(*on, target, skipped),
            DisplayEvent::Interrupted { signal } => self.interrupted(*signal),
            DisplayEvent::DefinitionValid { job_count } => self.definition_valid(*job_count),
            DisplayEvent::WatchTable { state, now } => self.watch_table(state, *now),
            DisplayEvent::WatchEmpty => self.watch_empty(),
            DisplayEvent::WatchFrame { prefix, state } => self.watch_frame(*prefix, *state),
            DisplayEvent::InitCreated { path } => self.init_created(path),
            DisplayEvent::InitReplaced { path } => self.init_replaced(path),
            DisplayEvent::InitExists { path } => self.init_exists(path),
            DisplayEvent::InitFailed { path, error } => self.init_failed(path, *error),
            DisplayEvent::LogNotCreated { path } => self.log_not_created(path),
            DisplayEvent::LogWriteFailed { path } => self.log_write_failed(path),
            DisplayEvent::LogOutputCutShort { path } => self.log_output_cut_short(path),
            DisplayEvent::InitStdoutFailed => self.init_stdout_failed(),
            DisplayEvent::Graph(text)
            | DisplayEvent::Plan(text)
            | DisplayEvent::InitTemplate(text) => self.pass_through(text),
            DisplayEvent::Help => self.help(),
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
