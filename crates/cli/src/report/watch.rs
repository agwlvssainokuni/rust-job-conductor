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

//! `watch` の表(WatchView。BR7.1)。ステートから、パイプライン名の 1 行と、
//! ジョブ・状態・所要時間・補足の 4 列の表を組み立てる(純粋。所要時間の列は v0.10.0 の U6 の BR4.1)。
//!
//! 状態は、色に関わらず、常に文字で書く。色は装飾で、色を使うときも、列の幅は、色を除いた文字で決める。
//! 所要時間の計算と書き方は core の `job_duration`・`format_duration`(U7 の継続表示も使う)。

use conductor_core::{
    JobState, JobStatus, PipelineState, SkipReason, Timestamp, format_duration, job_duration,
};

use super::escape_name;
use super::messages::{
    COMPLETED, EXIT_CODE, HEADER_DURATION, HEADER_JOB, HEADER_NOTE, HEADER_STATUS,
    SKIPPED_AFTER_END, SKIPPED_BY_TRANSITION, WATCH_PIPELINE, WATCH_RUN_ID,
};
use super::push_line;
use crate::types::Language;

/// 表の 1 行(ジョブ ID はエスケープ済み)。
struct Row {
    job: String,
    status: JobStatus,
    duration: String,
    note: String,
}

/// 状態を表す文字(言語に関わらず英語のまま。[`JobStatus::as_str`] と同じ)。
fn status_word(status: JobStatus) -> &'static str {
    status.as_str()
}

/// 状態の色(ANSI の SGR の番号)。`pending` は色なし。
fn status_color(status: JobStatus) -> Option<&'static str> {
    match status {
        JobStatus::Success => Some("32"),
        JobStatus::Warning => Some("33"),
        JobStatus::Error => Some("31"),
        JobStatus::Skipped => Some("36"),
        JobStatus::Pending => None,
    }
}

fn note(lang: Language, job: &JobState) -> String {
    match job.status {
        JobStatus::Success | JobStatus::Warning | JobStatus::Error => match job.exit_code {
            Some(code) => format!("{} {code}", EXIT_CODE.get(lang)),
            None => String::new(),
        },
        JobStatus::Skipped => match job.skip_reason {
            Some(SkipReason::ByEnd) => SKIPPED_AFTER_END.get(lang).to_string(),
            Some(SkipReason::ByTransition) => SKIPPED_BY_TRANSITION.get(lang).to_string(),
            None => String::new(),
        },
        JobStatus::Pending => String::new(),
    }
}

/// 表示の幅(全角の文字は 2 とみなす)。列をそろえるための近似。
fn width(text: &str) -> usize {
    text.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum()
}

fn pad(text: &str, to: usize) -> String {
    let w = width(text);
    let mut out = text.to_string();
    for _ in w..to {
        out.push(' ');
    }
    out
}

/// 所要時間の列の文字(v0.10.0 の U6 の BR4.1、BR4.2)。空にするジョブは空の文字列。
fn duration_cell(job: &JobState, now: Timestamp) -> String {
    job_duration(job, now)
        .map(|d| format_duration(d.secs()))
        .unwrap_or_default()
}

/// 表の全体(最後の改行を含む)。`now` は表示の時点の時刻(動いている最中のジョブの経過時間に使う)。
pub(crate) fn render_table(
    lang: Language,
    state: &PipelineState,
    color: bool,
    now: Timestamp,
) -> String {
    let rows: Vec<Row> = state
        .job_states
        .iter()
        .map(|job| Row {
            job: escape_name(&job.job_id),
            status: job.status,
            duration: duration_cell(job, now),
            note: note(lang, job),
        })
        .collect();

    let job_header = HEADER_JOB.get(lang);
    let status_header = HEADER_STATUS.get(lang);
    let duration_header = HEADER_DURATION.get(lang);
    let note_header = HEADER_NOTE.get(lang);
    let job_width = rows
        .iter()
        .map(|r| width(&r.job))
        .chain([width(job_header)])
        .max()
        .unwrap_or(0);
    let status_width = rows
        .iter()
        .map(|r| width(status_word(r.status)))
        .chain([width(status_header)])
        .max()
        .unwrap_or(0);
    let duration_width = rows
        .iter()
        .map(|r| width(&r.duration))
        .chain([width(duration_header)])
        .max()
        .unwrap_or(0);

    let mut out = format!(
        "{}: {}\n",
        WATCH_PIPELINE.get(lang),
        escape_name(&state.pipeline_name)
    );
    // run ID があるときだけ、パイプライン名の次の行に足す(run ID なしの表示は変えない)。
    if let Some(run_id) = &state.run_id {
        out.push_str(&format!("{WATCH_RUN_ID}: {}\n", escape_name(run_id)));
    }
    out.push_str(&format!(
        "{}  {}  {}  {}\n",
        pad(job_header, job_width),
        pad(status_header, status_width),
        pad(duration_header, duration_width),
        note_header
    ));
    for row in rows {
        let word = status_word(row.status);
        let padding = " ".repeat(status_width.saturating_sub(word.len()));
        let status_cell = match (color, status_color(row.status)) {
            (true, Some(code)) => format!("\x1b[{code}m{word}\x1b[0m{padding}"),
            _ => format!("{word}{padding}"),
        };
        let line = format!(
            "{}  {}  {}  {}",
            pad(&row.job, job_width),
            status_cell,
            pad(&row.duration, duration_width),
            row.note
        );
        push_line(&mut out, &line);
    }
    // 完了のステートは、表の下に、完了を示す行を足す(FR3.1)。
    if state.completed {
        out.push_str(COMPLETED.get(lang));
        out.push('\n');
    }
    out
}
