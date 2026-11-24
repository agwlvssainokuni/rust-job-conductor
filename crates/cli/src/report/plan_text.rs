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

//! `--dry-run` の計画の整形(WF4。BR3.1〜BR3.3、BR5.1、BR6.1)。純粋な関数。
//!
//! 形は、functional-spec.md の例 P1〜P8(日本語)、P2e、P4e(英語)で固定する。各行の末尾の空白は
//! 取り除く。ジョブ ID(使われなかった仮定のジョブ ID を含む)とコマンドの語は、制御文字を
//! エスケープして出す。環境変数は、名前だけを出す(値は、計画の型が持たない。BR5.1、FR1.12)。

use conductor_core::{
    Assumption, Classification, ExecutionPlan, NextStepNote, PlanDisposition, PlanEnd, PlanEntry,
    PlanMode, ResumeInfo,
};

use super::escape_full;
use super::messages::{
    COMPLETED, DETAIL_COMMAND, DETAIL_ENV, DETAIL_RETRY, PLAN_ASSUME_LABEL, PLAN_END_REACHED,
    PLAN_FROM_START, PLAN_HEADING, PLAN_NEXT_LABEL, PLAN_RESULT_LABEL, PLAN_RESUME_LABEL,
    PLAN_TARGET_LABEL, PLAN_UNUSED_LABEL, RESUME_HERE, graph_retry_summary, plan_assumed,
    plan_disposition, plan_next_step, plan_resume_after_stop, plan_run_id_suffix,
    plan_stopped_at_error,
};
use super::push_line;
use crate::types::Language;

/// 詳細の行の字下げ(5 つの空白。BR3.3)。
const DETAIL_INDENT: &str = "     ";

/// 計画の文字列を作る(最後の改行を含む)。
pub fn render_plan(lang: Language, plan: &ExecutionPlan) -> String {
    let mut out = String::new();

    let mut heading = format!(
        "{}{}",
        PLAN_HEADING.get(lang),
        escape_full(&plan.pipeline_name)
    );
    if let Some(run_id) = &plan.run_id {
        heading.push_str(&plan_run_id_suffix(lang, run_id));
    }
    push_line(&mut out, &heading);

    match &plan.resume {
        ResumeInfo::Individual(id) => push_line(
            &mut out,
            &format!("{}{}", PLAN_TARGET_LABEL.get(lang), escape_full(id)),
        ),
        ResumeInfo::AfterStop(id) => push_line(
            &mut out,
            &format!(
                "{}{}",
                PLAN_RESUME_LABEL.get(lang),
                plan_resume_after_stop(lang, &escape_full(id))
            ),
        ),
        ResumeInfo::FromStart => push_line(
            &mut out,
            &format!(
                "{}{}",
                PLAN_RESUME_LABEL.get(lang),
                PLAN_FROM_START.get(lang)
            ),
        ),
    }

    if !plan.assumptions.is_empty() {
        push_line(
            &mut out,
            &format!(
                "{}{}",
                PLAN_ASSUME_LABEL.get(lang),
                assumptions_text(&plan.assumptions)
            ),
        );
    }
    if !plan.unused_assumptions.is_empty() {
        let ids: Vec<String> = plan
            .unused_assumptions
            .iter()
            .map(|id| escape_full(id))
            .collect();
        push_line(
            &mut out,
            &format!("{}{}", PLAN_UNUSED_LABEL.get(lang), ids.join(", ")),
        );
    }

    for (index, entry) in plan.entries.iter().enumerate() {
        push_entry(&mut out, lang, index + 1, entry);
    }

    match plan.mode {
        PlanMode::Pipeline => {
            let end = match &plan.end {
                PlanEnd::Completed => COMPLETED.get(lang).to_string(),
                PlanEnd::EndReached => PLAN_END_REACHED.get(lang).to_string(),
                PlanEnd::StoppedAtError(id) => plan_stopped_at_error(lang, &escape_full(id)),
            };
            push_line(&mut out, &format!("{}{end}", PLAN_RESULT_LABEL.get(lang)));
        }
        PlanMode::Individual => {
            if let (Some(note), Some(entry)) = (&plan.next_step_note, plan.entries.first()) {
                push_line(&mut out, &next_step_line(lang, note, entry));
            }
        }
    }
    out
}

/// `<ジョブID>=<終了コード>` を指定順に、`, ` でつなぐ。
fn assumptions_text(assumptions: &[Assumption]) -> String {
    assumptions
        .iter()
        .map(|a| format!("{}={}", escape_full(&a.job_id), a.exit_code))
        .collect::<Vec<_>>()
        .join(", ")
}

/// ジョブの見出しの行と、詳細の行(BR3.2、BR3.3)。
fn push_entry(out: &mut String, lang: Language, number: usize, entry: &PlanEntry) {
    let mut word = plan_disposition(entry.disposition).get(lang).to_string();
    if let Some(note) = assumption_note(lang, entry) {
        // 「実行する」「run」に、括弧の補足を続ける。
        // 英語は、語と括弧のあいだに空白を置く(`run (assumed: ...)`)。日本語は置かない。
        let gap = if lang == Language::En { " " } else { "" };
        word.push_str(&format!("{gap}({note})"));
    }
    let mut line = format!("{number}. [{word}] {}", escape_full(&entry.job_id));
    if entry.is_resume {
        line.push_str(RESUME_HERE.get(lang));
    }
    push_line(out, &line);

    let PlanDisposition::WillRun(_) = entry.disposition else {
        return;
    };
    let words: Vec<String> = entry
        .command_words
        .iter()
        .map(|w| format!("\"{}\"", escape_full(w)))
        .collect();
    push_line(
        out,
        &format!(
            "{DETAIL_INDENT}{}[{}]",
            DETAIL_COMMAND.get(lang),
            words.join(", ")
        ),
    );
    let names: Vec<String> = entry
        .env_names
        .iter()
        .map(|e| escape_full(&e.name))
        .collect();
    push_line(
        out,
        &format!(
            "{DETAIL_INDENT}{}{}",
            DETAIL_ENV.get(lang),
            names.join(", ")
        ),
    );
    if let Some(retry) = &entry.retry {
        push_line(
            out,
            &format!(
                "{DETAIL_INDENT}{}{}",
                DETAIL_RETRY.get(lang),
                graph_retry_summary(lang, retry)
            ),
        );
    }
}

/// 仮定の補足(括弧の中)。指定したジョブ、または既定の結果が成功でないジョブに付ける(BR3.2)。
fn assumption_note(lang: Language, entry: &PlanEntry) -> Option<String> {
    let result = entry.disposition.result()?;
    if !result.assumed && result.classification == Classification::Success {
        return None;
    }
    Some(plan_assumed(
        lang,
        result.assumed,
        result.exit_code,
        result.classification,
        result.retryable,
    ))
}

/// 次の動きの行(`run`。BR1.6)。仮定(または既定)の結果が警告かエラーのときは、先頭に補足を添える。
fn next_step_line(lang: Language, note: &NextStepNote, entry: &PlanEntry) -> String {
    let mut text = String::new();
    if let Some(result) = entry.disposition.result()
        && result.classification != Classification::Success
    {
        let inner = plan_assumed(
            lang,
            result.assumed,
            result.exit_code,
            result.classification,
            false,
        );
        text.push_str(&format!("({inner}) "));
    }
    text.push_str(&plan_next_step(lang, note, escape_full));
    format!("{}{text}", PLAN_NEXT_LABEL.get(lang))
}
