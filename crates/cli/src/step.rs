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

//! 1 つのジョブを処理する共通の手順(BR1.1、BR1.2)。`run`(個別実行)と `pipeline`(一括実行)が呼ぶ。
//!
//! 起動前の判断は、`conductor_core::decide_from_state`(ステートが無いときの扱いを含む)を、呼び出し側が
//! 直接呼ぶ。手順は 3 つ(と、実行のあとの表示):
//!
//! | 関数 | 中身 |
//! |------|------|
//! | [`record_skip`] | 土台(ステート、無ければ初期ステート)に `apply_skip`。変わったときだけ `Durable` で書く |
//! | [`run_step`] | 空なら初期ステートを `Fast` で書く → リトライつきの実行 → 分類 → 表示 → 記録 → `Durable` で書く |
//! | [`remove_if_requested`] | `remove_on_complete` が真なら、ステートを削除する |
//! | `report_execution`(非公開) | `run_step` の表示。結果・起動の失敗・成功コードに含まれない 0・遷移・中断 |
//!
//! 書く順序、`Durability`、表示の文脈(`StateContext`)は、各関数の説明のとおり。
//! 書き込みの失敗は、表示したうえで、終了コード(65)を `Err` で返す。呼び出し側は、それで終わる。
//!
//! いつ消すか(完了の扱い)と、終了コードの決め方は、`run` と `pipeline` で違うため、呼び出し側に残す
//! (BR1.2)。

use conductor_core::{
    Classification, ExecutionResult, JobDefinition, JobExecution, JobOutcome, NextStep,
    PipelineState, RunMode, SkipCause, apply_outcome, apply_skip, classify_and_decide,
    initial_state,
};
use conductor_runtime::{CommandLine, Durability};

use crate::ctx::{Ctx, Session};
use crate::exit::outcome_exit_code;
use crate::report::{DisplayEvent, ResultDetail, StateContext};
use crate::retry::run_job_with_retry;

/// 1 つのジョブを実行し、記録したあとの結果([`run_step`] の戻り値)。
pub(crate) struct StepResult {
    /// 記録後のステート(書き込み済み)。
    pub recorded: PipelineState,
    /// 分類と、報告の終了コード。
    pub outcome: JobOutcome,
    /// 次の動き(完了したか、など)。
    pub next: NextStep,
    /// ジョブが起動したかによる文脈(この後の書き込みと削除の誤りの表示に使う)。
    pub after: StateContext,
}

/// スキップの記録の結果([`record_skip`] が返す)。
pub(crate) struct SkipRecord {
    /// 記録後のステート。
    pub state: PipelineState,
    /// 土台から変わり、書いたか。`run` はこれで削除するかを決め、`pipeline` は使わない(BR1.2)。
    pub written: bool,
}

/// スキップを記録する(BR1.1)。土台は `base`(無ければ初期ステート)。`apply_skip` の結果が土台から
/// 変わったときだけ、`Durable` で書く(文脈は「起動前」)。
///
/// 記録後のステートと、書いたかを [`SkipRecord`] で返す。書き込みの失敗は、終了コードを `Err` で返す。
pub(crate) fn record_skip(
    ctx: &mut Ctx<'_>,
    session: &Session,
    base: Option<&PipelineState>,
    job_id: &str,
    reason: SkipCause,
) -> Result<SkipRecord, i32> {
    let definition = &session.checked.base.definition;
    let path = &session.checked.base.path;
    let run_id = session.checked.run_id.as_ref();
    // 土台は借りる(無いときだけ、初期ステートを作る)。
    let initial;
    let base = match base {
        Some(base) => base,
        None => {
            initial = initial_state(definition, path, run_id);
            &initial
        }
    };
    let recorded = apply_skip(base, job_id, reason, definition);
    let changed = recorded != *base;
    if changed {
        ctx.write(
            path,
            run_id,
            &recorded,
            Durability::Durable,
            StateContext::JobNotStarted,
        )?;
    }
    Ok(SkipRecord {
        state: recorded,
        written: changed,
    })
}

/// ジョブを 1 つ実行して記録する(BR1.1)。順序は固定:
///
/// 1. `base` が空なら、初期ステートを `Fast` で書く(文脈は「起動前」)。
/// 2. `run_job_with_retry` で、`command` を渡して実行する(リトライを含む)。
/// 3. `classify_and_decide` で分類し、次の動きを決める(`mode` は実行方式)。
/// 4. `report_execution` で表示する。
/// 5. `apply_outcome` で記録し、`Durable` で書く(文脈は、起動したかで決まる)。
///
/// `base` は値で受ける(`run` は `session.checked.state.clone()`、`pipeline` は `current.take()` を渡す)。
/// `command` は、定義の検査で分割したそのジョブのコマンド(`session.checked.commands` の同じ位置)。
/// 起動にも表示にも、これを使う(分け直さない。BR2.2)。
/// 書き込みの失敗は、終了コードを `Err` で返す。
pub(crate) fn run_step(
    ctx: &mut Ctx<'_>,
    session: &Session,
    base: Option<PipelineState>,
    job: &JobDefinition,
    command: &CommandLine,
    mode: RunMode,
) -> Result<StepResult, i32> {
    let definition = &session.checked.base.definition;
    let path = &session.checked.base.path;
    let run_id = session.checked.run_id.as_ref();
    let base = match base {
        Some(state) => state,
        None => {
            let initial = initial_state(definition, path, run_id);
            ctx.write(
                path,
                run_id,
                &initial,
                Durability::Fast,
                StateContext::JobNotStarted,
            )?;
            initial
        }
    };

    let execution = run_job_with_retry(ctx, session, job, command);
    let (outcome, next) = classify_and_decide(job, &execution, &base.job_states, definition, mode);
    report_execution(ctx, job, command, &execution, &outcome, &next);

    let recorded = apply_outcome(&base, &job.id, &execution, &outcome, &next);
    // 起動前の中断(`NotStarted`)は、ジョブが実行されていない。
    let after = if matches!(execution.result, ExecutionResult::NotStarted { .. }) {
        StateContext::JobNotStarted
    } else {
        StateContext::JobExecuted
    };
    ctx.write(path, run_id, &recorded, Durability::Durable, after)?;
    Ok(StepResult {
        recorded,
        outcome,
        next,
        after,
    })
}

/// 完了のあとの削除(BR1.2、BR3.3、FR1.1、FR2.1)。完了のステートは、すでに書いてある。
/// 実効の `remove_on_complete` が真のときだけ、ステートを削除する(失敗は、終了コードを `Err` で返す)。
/// 偽なら、何もしない(完了のステートを残す)。いつ呼ぶかは、呼び出し側が決める。
pub(crate) fn remove_if_requested(
    ctx: &mut Ctx<'_>,
    session: &Session,
    context: StateContext,
) -> Result<(), i32> {
    let checked = &session.checked;
    if !checked.remove_on_complete {
        return Ok(());
    }
    ctx.reset(&checked.base.path, checked.run_id.as_ref(), context)
}

/// ジョブを実行したあとの表示(BR2.6、BR6.3、BR6.4)。`-v` の結果・遷移・中断と、
/// 常に表示する起動の失敗・成功コードに含まれない 0。`command` は、定義の検査で分割したそのジョブの
/// コマンド(起動の失敗の表示に、コマンド名を使う)。
fn report_execution(
    ctx: &mut Ctx<'_>,
    job: &JobDefinition,
    command: &CommandLine,
    execution: &JobExecution,
    outcome: &JobOutcome,
    next: &NextStep,
) {
    let conductor_exit = outcome_exit_code(outcome);
    let detail = result_detail(execution);
    ctx.show(&DisplayEvent::JobResult {
        job_id: &job.id,
        result: outcome.classification,
        detail,
        conductor_exit,
    });

    if detail == ResultDetail::StartFailure {
        ctx.show(&DisplayEvent::StartFailure {
            job_id: &job.id,
            command: &command.program,
        });
    }
    if outcome.conductor_error {
        ctx.show(&DisplayEvent::SuccessCodeNotListed { job_id: &job.id });
    }

    if let Some(on) = outcome
        .classification
        .and_then(Classification::transition_on)
        && let Some(transition) = job.transition_for(on)
    {
        ctx.show(&DisplayEvent::Transition {
            on,
            target: &transition.next,
            skipped: &next.skipped_job_ids,
        });
    }
    // 起動しなかった中断と、終了の観測の直後に合図が立っていた実行のときだけ(`result_detail` が作る)。
    if let ResultDetail::Interrupted { signal, .. } = detail {
        ctx.show(&DisplayEvent::Interrupted { signal });
    }
}

/// 実行の結果の補足(`classify` の判定の順序と同じ)。
fn result_detail(execution: &JobExecution) -> ResultDetail {
    match execution.result {
        ExecutionResult::NotStarted { signal } => ResultDetail::Interrupted {
            signal,
            started: false,
        },
        ExecutionResult::Exited {
            interrupted_by: Some(signal),
            ..
        } => ResultDetail::Interrupted {
            signal,
            started: true,
        },
        ExecutionResult::StartFailed => ResultDetail::StartFailure,
        ExecutionResult::Exited {
            terminated_by_signal: Some(signal),
            interrupted_by: None,
            ..
        } => ResultDetail::Signal(signal),
        ExecutionResult::Exited {
            exit_code,
            terminated_by_signal: None,
            interrupted_by: None,
        } => ResultDetail::ExitCode(exit_code.unwrap_or(0)),
    }
}
