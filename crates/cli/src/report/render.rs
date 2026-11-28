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

//! 表示の組み立ての中身(Reporter の一部。BR6.1〜BR6.5、NFR4.2)。
//!
//! 親のモジュールの `render` が、[`DisplayEvent`](super::DisplayEvent) の変種ごとに、ここのメソッドを
//! 1 つ呼ぶ。メソッドは表示の形(A〜I)の節に並べ、取り出した中身を引数で受け取る。
//!
//! - 言語は、メソッドの中で `self.language()` を呼んで、必要になったときに初めて判定する。
//!   固定の文(A)と、組み立て済みの文をそのまま出す `pass_through` は判定しない。
//! - 色は `watch_table` だけが判定する。`watch_frame` は `self.render` を呼び直す。

use std::path::Path;

use conductor_converter::ConversionError;
use conductor_core::{
    Classification, DefinitionError, EnvFileError, EnvWarning, MismatchKind, PipelineState,
    RejectReason, SkipCause, Timestamp, TransitionOn, TransitionTarget,
};
use conductor_runtime::{CommandError, CreateError, StateError};

use super::lines::{
    command_line, conversion_line, definition_line, env_file_lines, env_warning_line, error_line,
    job_place, job_result_line, show_path, state_line, usage_text,
};
use super::messages::{
    self, ERROR_PREFIX, RETRY_PREFIX, VERBOSE_PREFIX, WARNING_PREFIX, mismatch_kind, reject_reason,
    skip_reason,
};
use super::{
    DisplayEvent, FramePrefix, Reporter, ResultDetail, StateContext, escape_full, escape_name,
    watch,
};
use crate::types::UsageError;

impl Reporter<'_> {
    // ---- A. 固定の文 ----
    // ロケールを判定しない、固定の文(パニックのフックと同じ。BR6.1 の差分)。

    pub(super) fn signal_setup_failed(&self) -> String {
        format!("{}\n", messages::SIGNAL_SETUP_FAILED)
    }

    pub(super) fn version(&self) -> String {
        format!("conductor {}\n", env!("CARGO_PKG_VERSION"))
    }

    // ---- B. 誤りの一覧 ----

    pub(super) fn definition_errors(&self, errors: &[DefinitionError]) -> String {
        let lang = self.language();
        errors.iter().map(|e| definition_line(lang, e)).collect()
    }

    pub(super) fn env_file_errors(&self, errors: &[EnvFileError]) -> String {
        env_file_lines(self.language(), errors)
    }

    pub(super) fn env_warnings(&self, warnings: &[EnvWarning]) -> String {
        let lang = self.language();
        warnings.iter().map(|w| env_warning_line(lang, w)).collect()
    }

    pub(super) fn command_errors(
        &self,
        definition: &Path,
        errors: &[(String, CommandError)],
    ) -> String {
        let lang = self.language();
        let place = show_path(definition);
        errors
            .iter()
            .map(|(job_id, error)| command_line(lang, &place, job_id, error))
            .collect()
    }

    // ---- C. 関数に任せる誤りの 1 行 ----

    pub(super) fn usage(&self, error: &UsageError) -> String {
        usage_text(self.language(), error)
    }

    pub(super) fn state(&self, error: &StateError, context: StateContext) -> String {
        state_line(self.language(), error, context)
    }

    pub(super) fn conversion(&self, error: &ConversionError) -> String {
        conversion_line(self.language(), error)
    }

    // ---- D. 場所つきの誤りの 1 行 ----

    pub(super) fn consistency_mismatch(
        &self,
        state_path: &Path,
        kind: MismatchKind,
        context: StateContext,
    ) -> String {
        let lang = self.language();
        let place = show_path(state_path);
        let hint = messages::reset_hint(context).get(lang);
        let what = mismatch_kind(kind).get(lang);
        error_line(
            &place,
            messages::MISMATCH.get(lang),
            Some(&format!("{what}; {hint}")),
        )
    }

    pub(super) fn rejected(&self, job_id: &str, reason: RejectReason) -> String {
        let lang = self.language();
        let place = job_place(job_id);
        error_line(
            &place,
            messages::REJECTED.get(lang),
            Some(reject_reason(reason).get(lang)),
        )
    }

    pub(super) fn start_failure(&self, job_id: &str, command: &str) -> String {
        let place = job_place(job_id);
        let sentence = format!(
            "{} '{}'",
            messages::CANNOT_START.get(self.language()),
            escape_name(command)
        );
        error_line(&place, &sentence, None)
    }

    pub(super) fn success_code_not_listed(&self, job_id: &str) -> String {
        let place = job_place(job_id);
        error_line(
            &place,
            messages::SUCCESS_CODE_NOT_LISTED.get(self.language()),
            None,
        )
    }

    pub(super) fn completion_undetermined(&self, state_path: &Path) -> String {
        let place = show_path(state_path);
        error_line(
            &place,
            messages::COMPLETION_UNDETERMINED.get(self.language()),
            None,
        )
    }

    // ---- E. `ERROR_PREFIX` と文 ----

    pub(super) fn output_failed(&self) -> String {
        format!(
            "{ERROR_PREFIX}{}\n",
            messages::OUTPUT_FAILED.get(self.language())
        )
    }

    pub(super) fn init_exists(&self, path: &Path) -> String {
        format!(
            "{ERROR_PREFIX}{}\n",
            messages::init_exists(self.language(), &show_path(path))
        )
    }

    pub(super) fn init_failed(&self, path: &Path, error: CreateError) -> String {
        format!(
            "{ERROR_PREFIX}{}\n",
            messages::init_cannot_create(self.language(), error, &show_path(path))
        )
    }

    pub(super) fn init_stdout_failed(&self) -> String {
        format!(
            "{ERROR_PREFIX}{}\n",
            messages::INIT_STDOUT_FAILED.get(self.language())
        )
    }

    pub(super) fn log_not_created(&self, path: &Path) -> String {
        format!(
            "{ERROR_PREFIX}{}: {}\n",
            messages::LOG_NOT_CREATED.get(self.language()),
            show_path(path)
        )
    }

    // ---- F. `WARNING_PREFIX` と文 ----

    pub(super) fn reset_all_kept(&self, names: &[String]) -> String {
        let mut out = format!(
            "{WARNING_PREFIX}{}\n",
            messages::RESET_ALL_KEPT.get(self.language())
        );
        for name in names.iter() {
            out.push_str(&format!("  {}\n", escape_full(name)));
        }
        out
    }

    pub(super) fn log_write_failed(&self, path: &Path) -> String {
        format!(
            "{WARNING_PREFIX}{}: {}\n",
            messages::LOG_WRITE_FAILED.get(self.language()),
            show_path(path)
        )
    }

    pub(super) fn log_output_cut_short(&self, path: &Path) -> String {
        format!(
            "{WARNING_PREFIX}{}: {}\n",
            messages::LOG_OUTPUT_CUT_SHORT.get(self.language()),
            show_path(path)
        )
    }

    // ---- G. 通知 ----

    pub(super) fn retry_scheduled(
        &self,
        job_id: &str,
        last_exit_code: i32,
        retry_number: u32,
        max: u32,
        wait_secs: u64,
    ) -> String {
        format!(
            "{RETRY_PREFIX}{}\n",
            messages::retry_scheduled(
                self.language(),
                &escape_name(job_id),
                last_exit_code,
                retry_number,
                max,
                wait_secs
            )
        )
    }

    pub(super) fn init_created(&self, path: &Path) -> String {
        format!(
            "{}\n",
            messages::init_created(self.language(), &show_path(path))
        )
    }

    pub(super) fn init_replaced(&self, path: &Path) -> String {
        format!(
            "{}\n",
            messages::init_replaced(self.language(), &show_path(path))
        )
    }

    // ---- H. `-v` の行 ----

    pub(super) fn job_result(
        &self,
        job_id: &str,
        result: Option<Classification>,
        detail: ResultDetail,
        conductor_exit: i32,
    ) -> String {
        job_result_line(self.language(), job_id, result, detail, conductor_exit)
    }

    pub(super) fn job_skipped(&self, job_id: &str, reason: SkipCause) -> String {
        format!(
            "{VERBOSE_PREFIX}job={} {} ({})\n",
            escape_name(job_id),
            messages::SKIPPED_WORD.get(self.language()),
            skip_reason(reason).get(self.language())
        )
    }

    pub(super) fn transition(
        &self,
        on: TransitionOn,
        target: &TransitionTarget,
        skipped: &[String],
    ) -> String {
        let mut line = format!(
            "{VERBOSE_PREFIX}transition: {} -> {}",
            on.as_str(),
            escape_name(target.label())
        );
        if !skipped.is_empty() {
            let ids: Vec<String> = skipped.iter().map(|s| escape_name(s)).collect();
            line.push_str(&format!(
                " ({}: {})",
                messages::SKIPPED_WORD.get(self.language()),
                ids.join(", ")
            ));
        }
        line.push('\n');
        line
    }

    pub(super) fn interrupted(&self, signal: i32) -> String {
        format!(
            "{VERBOSE_PREFIX}{} {signal}\n",
            messages::INTERRUPTED_BY_SIGNAL.get(self.language())
        )
    }

    pub(super) fn definition_valid(&self, job_count: usize) -> String {
        format!(
            "{VERBOSE_PREFIX}{}\n",
            messages::definition_valid(self.language(), job_count)
        )
    }

    // ---- I. 標準出力 ----

    pub(super) fn watch_table(&self, state: &PipelineState, now: Timestamp) -> String {
        watch::render_table(self.language(), state, self.color_enabled(), now)
    }

    pub(super) fn watch_empty(&self) -> String {
        format!("{}\n", messages::WATCH_EMPTY.get(self.language()))
    }

    pub(super) fn watch_frame(
        &self,
        prefix: FramePrefix,
        state: Option<(&PipelineState, Timestamp)>,
    ) -> String {
        let body = match state {
            Some((state, now)) => DisplayEvent::WatchTable { state, now },
            None => DisplayEvent::WatchEmpty,
        };
        format!("{}{}", prefix.text(), self.render(&body))
    }

    /// 組み立て済みの文(`graph` の図、`--dry-run` の計画、`init --config -` のひな形)。言語を判定しない。
    pub(super) fn pass_through(&self, text: &str) -> String {
        text.to_string()
    }

    pub(super) fn help(&self) -> String {
        messages::USAGE.get(self.language()).to_string()
    }
}
