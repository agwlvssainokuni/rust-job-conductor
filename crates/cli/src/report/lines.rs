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

//! 1 行の組み立て(Reporter の一部。BR6.1〜BR6.5、NFR4.2)。
//!
//! 誤り・警告・`-v` の 1 行(と、その並び)を、言語を引数で受け取って組み立てる。親のモジュールの
//! 表示の組み立てから使われる。

use std::path::Path;

use conductor_converter::ConversionError;
use conductor_core::{
    Classification, DefinitionError, EnvFileError, EnvWarning, EnvWarningKind, summarize_errors,
};
use conductor_runtime::{CommandError, StateError};

use super::messages::{
    self, ERROR_PREFIX, VERBOSE_PREFIX, WARNING_PREFIX, command_kind, conversion_kind,
    definition_kind, env_duplicate_key, env_errors_more, env_file_kind, env_file_reason,
    state_kind, state_kind_suggests_reset, usage_kind,
};
use super::{ResultDetail, StateContext, escape_full, escape_name};
use crate::types::{Language, UsageError};

/// パスの表示(制御文字をエスケープする。切らない。定義ファイルのパスと同じ扱い)。
pub(super) fn show_path(path: &Path) -> String {
    escape_full(&path.display().to_string())
}

/// `conductor: error: <場所>: <種類の文>[: <原因>]` の 1 行。
pub(super) fn error_line(place: &str, sentence: &str, cause: Option<&str>) -> String {
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

pub(super) fn definition_line(lang: Language, e: &DefinitionError) -> String {
    let place = place_of(&e.file, e.line);
    let reason = escape_full(&e.reason);
    error_line(&place, definition_kind(e.kind).get(lang), Some(&reason))
}

/// `.env` の誤りの行(最大 20 件。残りは「ほか N 件」。値は含めない。BR2.8、BR8.1)。
pub(super) fn env_file_lines(lang: Language, errors: &[EnvFileError]) -> String {
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
pub(super) fn env_warning_line(lang: Language, w: &EnvWarning) -> String {
    let EnvWarningKind::DuplicateKey = w.kind;
    format!(
        "{WARNING_PREFIX}{}: {}\n",
        place_of(&w.file, Some(w.line)),
        env_duplicate_key(lang, &escape_name(&w.name))
    )
}

/// ジョブの場所 `job '<ID>'`(ジョブ ID はエスケープする)。
pub(super) fn job_place(job_id: &str) -> String {
    format!("job '{}'", escape_name(job_id))
}

pub(super) fn command_line(
    lang: Language,
    definition: &str,
    job_id: &str,
    e: &CommandError,
) -> String {
    let place = format!("{definition}: {}", job_place(job_id));
    let sentence = command_kind(e.kind).get(lang);
    error_line(&place, sentence, None)
}

pub(super) fn usage_text(lang: Language, e: &UsageError) -> String {
    let sentence = usage_kind(e.kind).get(lang);
    let mut out = match &e.subject {
        Some(subject) => format!("{ERROR_PREFIX}{sentence}: '{}'\n", escape_name(subject)),
        None => format!("{ERROR_PREFIX}{sentence}\n"),
    };
    out.push('\n');
    out.push_str(messages::USAGE.get(lang));
    out
}

pub(super) fn state_line(lang: Language, e: &StateError, context: StateContext) -> String {
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

pub(super) fn conversion_line(lang: Language, e: &ConversionError) -> String {
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

pub(super) fn job_result_line(
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
