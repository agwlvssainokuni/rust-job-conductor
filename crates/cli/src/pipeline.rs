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
//! 定義の記述順に、1 つずつ、`decide_before_run` で決める(次のジョブを `NextStep` で決めない。
//! 遷移先とスキップは、ステートが運ぶ)。結果、またはスキップを記録するたびに、`completed` を
//! 確かめ、true なら、完了のステートを残して 0 で終わる。削除を求められている(`--remove-on-complete`
//! または定義の `remove_on_complete`)ときだけ、`reset_state` する。エラーで停止し、再試行しない。
//! ジョブのあいだで合図が立っていれば、再開位置を記録して、128 + 番号で終わる。

use conductor_core::{
    Classification, Decision, PipelineState, RunId, RunMode, classify_and_decide,
    decide_before_run, resume_point_on_interrupt,
};
use conductor_runtime::{
    Durability, StateLocation, apply_interrupt, apply_outcome, apply_skip, initial_state,
};

use crate::ctx::Ctx;
use crate::exit::{
    EXIT_OK, EXIT_REJECTED, EXIT_STATE, EXIT_USAGE, exit_code_for, signal_exit_code,
};
use crate::report::{DisplayEvent, StateContext};
use crate::retry::run_job_with_retry;
use crate::types::{CliCommand, UsageError, UsageErrorKind};

/// `conductor pipeline <pipeline-name>` を実行し、終了コードを返す。
pub(crate) fn run_pipeline(ctx: &mut Ctx<'_>, cmd: &CliCommand) -> i32 {
    let Some(name) = cmd.pipeline_name.as_deref() else {
        let error = UsageError::new(UsageErrorKind::MissingArgument);
        ctx.show(&DisplayEvent::Usage(&error));
        return EXIT_USAGE;
    };
    let session = match ctx.prepare(cmd, Some(name), None) {
        Ok(session) => session,
        Err(code) => return code,
    };
    let definition = &session.definition;
    let path = &session.definition_path;
    let run_id = session.run_id.as_ref();

    // ステートが空のあいだは `None`。最初のジョブが `Run` と決まったあとに、初期ステートを書く。
    let mut current: Option<PipelineState> = session.state.clone();

    for job in &definition.job {
        let decision = {
            let (states, resume, end_reached) = match &current {
                Some(state) => (
                    state.job_states.as_slice(),
                    state.resume_job_id.as_deref(),
                    state.end_reached,
                ),
                None => (&[][..], None, false),
            };
            decide_before_run(&job.id, definition, states, resume, end_reached)
        };

        match decision {
            Decision::Reject(reason) => {
                ctx.show(&DisplayEvent::Rejected {
                    job_id: &job.id,
                    reason,
                });
                return EXIT_REJECTED;
            }
            Decision::Skip(reason) => {
                let base = current
                    .clone()
                    .unwrap_or_else(|| initial_state(definition, path, run_id));
                let recorded = apply_skip(&base, &job.id, reason, definition);
                if recorded != base
                    && let Err(code) = ctx.write(
                        path,
                        run_id,
                        &recorded,
                        Durability::Durable,
                        StateContext::JobNotStarted,
                    )
                {
                    return code;
                }
                ctx.show(&DisplayEvent::JobSkipped {
                    job_id: &job.id,
                    reason,
                });
                let completed = recorded.completed;
                current = Some(recorded);
                if completed {
                    return finish(
                        ctx,
                        path,
                        run_id,
                        session.remove_on_complete,
                        StateContext::JobNotStarted,
                    );
                }
            }
            Decision::Run => {
                let base = match current.take() {
                    Some(state) => state,
                    None => {
                        let initial = initial_state(definition, path, run_id);
                        if let Err(code) = ctx.write(
                            path,
                            run_id,
                            &initial,
                            Durability::Fast,
                            StateContext::JobNotStarted,
                        ) {
                            return code;
                        }
                        initial
                    }
                };

                let execution = run_job_with_retry(ctx, job, &session.signal);
                let (outcome, next) = classify_and_decide(
                    job,
                    &execution,
                    &base.job_states,
                    definition,
                    RunMode::Pipeline,
                );
                ctx.report_execution(job, &execution, &outcome, &next);

                let recorded = apply_outcome(&base, &job.id, &execution, &outcome, &next);
                // 起動前の中断(started = false)は、ジョブが実行されていない。
                let after = if execution.started {
                    StateContext::JobExecuted
                } else {
                    StateContext::JobNotStarted
                };
                if let Err(code) = ctx.write(path, run_id, &recorded, Durability::Durable, after) {
                    return code;
                }
                let is_error = outcome.classification == Some(Classification::Error);
                if is_error || outcome.classification.is_none() {
                    // エラーで停止する(再試行しない)。中断で起動しなかった場合も、ここで終える。
                    return exit_code_for(outcome.reported_exit_code, is_error);
                }
                if next.completed {
                    return finish(ctx, path, run_id, session.remove_on_complete, after);
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
/// `remove_on_complete` が真のときだけ、ステートを削除する(失敗は 65)。偽なら、残して 0 で終える。
fn finish(
    ctx: &mut Ctx<'_>,
    path: &std::path::Path,
    run_id: Option<&RunId>,
    remove_on_complete: bool,
    context: StateContext,
) -> i32 {
    if !remove_on_complete {
        return EXIT_OK;
    }
    match ctx.reset(path, run_id, context) {
        Ok(()) => EXIT_OK,
        Err(code) => code,
    }
}
