//! `watch` の表(WatchView。BR7.1)。ステートから、パイプライン名の 1 行と、
//! ジョブ・状態・補足の 3 列の表を組み立てる(純粋)。
//!
//! 状態は、色に関わらず、常に文字で書く。色は装飾で、色を使うときも、列の幅は、色を除いた文字で決める。

use conductor_core::{JobState, JobStatus, PipelineState, SkipReason};

use super::escape::escape_name;
use super::messages::{
    EXIT_CODE, HEADER_JOB, HEADER_NOTE, HEADER_STATUS, NOTE_BY_END, NOTE_BY_TRANSITION,
    WATCH_PIPELINE,
};
use crate::types::Language;

/// 状態を表す文字(言語に関わらず英語のまま)。
pub fn status_word(status: JobStatus) -> &'static str {
    match status {
        JobStatus::Pending => "pending",
        JobStatus::Success => "success",
        JobStatus::Warning => "warning",
        JobStatus::Error => "error",
        JobStatus::Skipped => "skipped",
    }
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
            Some(SkipReason::ByEnd) => NOTE_BY_END.get(lang).to_string(),
            Some(SkipReason::ByTransition) => NOTE_BY_TRANSITION.get(lang).to_string(),
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

/// 表の全体(最後の改行を含む)。
pub fn render_table(lang: Language, state: &PipelineState, color: bool) -> String {
    let rows: Vec<(String, JobStatus, String)> = state
        .job_states
        .iter()
        .map(|job| (escape_name(&job.job_id), job.status, note(lang, job)))
        .collect();

    let job_header = HEADER_JOB.get(lang);
    let status_header = HEADER_STATUS.get(lang);
    let note_header = HEADER_NOTE.get(lang);
    let job_width = rows
        .iter()
        .map(|r| width(&r.0))
        .chain([width(job_header)])
        .max()
        .unwrap_or(0);
    let status_width = rows
        .iter()
        .map(|r| width(status_word(r.1)))
        .chain([width(status_header)])
        .max()
        .unwrap_or(0);

    let mut out = format!(
        "{}: {}\n",
        WATCH_PIPELINE.get(lang),
        escape_name(&state.pipeline_name)
    );
    out.push_str(&format!(
        "{}  {}  {}\n",
        pad(job_header, job_width),
        pad(status_header, status_width),
        note_header
    ));
    for (job, status, note) in rows {
        let word = status_word(status);
        let padding = " ".repeat(status_width.saturating_sub(word.len()));
        let status_cell = match (color, status_color(status)) {
            (true, Some(code)) => format!("\x1b[{code}m{word}\x1b[0m{padding}"),
            _ => format!("{word}{padding}"),
        };
        let line = format!("{}  {}  {}", pad(&job, job_width), status_cell, note);
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}
