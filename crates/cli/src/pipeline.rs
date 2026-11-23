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

//! 一括実行(`conductor pipeline <pipeline-name>`。W3、BR3.1〜BR3.4、BR4.2)。
//!
//! 定義の記述順に、1 つずつ、`decide_from_state` で決める(次のジョブを `NextStep` で決めない。
//! 遷移先とスキップは、ステートが運ぶ)。結果、またはスキップを記録するたびに、`completed` を
//! 確かめ、true なら、完了のステートを残して 0 で終わる。削除を求められている(`--remove-on-complete`
//! または定義の `remove_on_complete`)ときだけ、`reset_state` する。エラー(`[job.retry]` のリトライを使い切った後の結果)で停止する。
//! ジョブのあいだで合図が立っていれば、再開位置を記録して、128 + 番号で終わる。
//!
//! 1 つのジョブの手順(判断、スキップの記録、実行と記録、削除)は、`run` と共通の
//! `crate::step` を呼ぶ(BR1.1)。ジョブのループ、いつ消すか、エラーでの停止、ジョブのあいだの
//! 中断、「完了を判定できない」は、ここに残す(BR1.2)。

use conductor_core::{
    Classification, Decision, PipelineState, RunMode, apply_interrupt, decide_from_state,
    resume_point_on_interrupt,
};
use conductor_runtime::{Durability, StateLocation};

use crate::ctx::{Ctx, FlowTarget, Session};
use crate::dry_run::enter_flow;
use crate::exit::{EXIT_OK, EXIT_REJECTED, EXIT_STATE, outcome_exit_code, signal_exit_code};
use crate::report::{DisplayEvent, StateContext};
use crate::step::{StepResult, record_skip, remove_if_requested, run_step};
use crate::types::CliCommand;

/// `conductor pipeline <pipeline-name>` を実行し、終了コードを返す。
pub(crate) fn run_pipeline(ctx: &mut Ctx<'_>, cmd: &CliCommand) -> i32 {
    let name = match enter_flow(ctx, cmd, cmd.pipeline_name.as_deref(), FlowTarget::Pipeline) {
        Ok(name) => name,
        Err(code) => return code,
    };
    let session = match ctx.prepare_session(|c| {
        c.check_pipeline(cmd, name, &[])
            .map(|checked| (checked, ()))
    }) {
        Ok((session, ())) => session,
        Err(code) => return code,
    };
    let definition = &session.checked.base.definition;
    let path = &session.checked.base.path;
    let run_id = session.checked.run_id.as_ref();

    // ステートが空のあいだは `None`。最初のジョブが `Run` と決まったあとに、初期ステートを書く。
    let mut current: Option<PipelineState> = session.checked.state.clone();

    for (job, command) in definition.job.iter().zip(&session.checked.base.commands) {
        match decide_from_state(&job.id, definition, current.as_ref()) {
            Decision::Reject(reason) => {
                ctx.show(&DisplayEvent::Rejected {
                    job_id: &job.id,
                    reason,
                });
                return EXIT_REJECTED;
            }
            Decision::Skip(reason) => {
                // pipeline は `written` を使わない。記録が変わらなくても、完了なら消す(run と違う。BR1.2)。
                let recorded = match record_skip(ctx, &session, current.as_ref(), &job.id, reason) {
                    Ok(record) => record.state,
                    Err(code) => return code,
                };
                ctx.show(&DisplayEvent::JobSkipped {
                    job_id: &job.id,
                    reason,
                });
                let completed = recorded.completed;
                current = Some(recorded);
                if completed {
                    return finish(ctx, &session, StateContext::JobNotStarted);
                }
            }
            Decision::Run => {
                let step = match run_step(
                    ctx,
                    &session,
                    current.take(),
                    job,
                    command,
                    RunMode::Pipeline,
                ) {
                    Ok(step) => step,
                    Err(code) => return code,
                };
                let StepResult {
                    recorded,
                    outcome,
                    next,
                    after,
                } = step;
                let is_error = outcome.classification == Some(Classification::Error);
                if is_error || outcome.classification.is_none() {
                    // エラーで停止する(リトライは `run_job_with_retry` で済んでいる)。中断で起動しなかった場合も、ここで終える。
                    return outcome_exit_code(&outcome);
                }
                if next.completed {
                    return finish(ctx, &session, after);
                }

                // ジョブのあいだの中断(BR4.2)。次のジョブを起動せず、再開位置を記録する。
                if let Some(number) = session.signal.signal_number() {
                    let point = resume_point_on_interrupt(definition, &recorded.job_states);
                    let interrupted = apply_interrupt(&recorded, &point);
                    if let Err(code) = ctx.write(
                        path,
                        run_id,
                        &interrupted,
                        Durability::Durable,
                        StateContext::JobNotStarted,
                    ) {
                        return code;
                    }
                    ctx.show(&DisplayEvent::Interrupted { signal: number });
                    return signal_exit_code(number);
                }
                current = Some(recorded);
            }
        }
    }

    // 記述順のジョブがなくなったのに、完了していない(起こらない想定)。初期化せずに終わる。
    let state_path = StateLocation::of(path, run_id).path().to_path_buf();
    ctx.show(&DisplayEvent::CompletionUndetermined {
        state_path: &state_path,
    });
    EXIT_STATE
}

/// 完了のあとの処理(BR3.3、FR1.1、FR2.1)。完了のステートは、すでに書いてある。
/// 削除は共通の `remove_if_requested` で行う(失敗は 65)。削除を求められていなければ、残して 0 で終える。
fn finish(ctx: &mut Ctx<'_>, session: &Session, context: StateContext) -> i32 {
    match remove_if_requested(ctx, session, context) {
        Ok(()) => EXIT_OK,
        Err(code) => code,
    }
}
