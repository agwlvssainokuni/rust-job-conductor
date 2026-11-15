//! 一括実行(`conductor pipeline <pipeline-name>`。W3、BR3.1〜BR3.4、BR4.2)。
//!
//! 定義の記述順に、1 つずつ、`decide_before_run` で決める(次のジョブを `NextStep` で決めない。
//! 遷移先とスキップは、ステートが運ぶ)。結果、またはスキップを記録するたびに、`completed` を
//! 確かめ、true なら `reset_state` して 0 で終わる。エラーで停止し、再試行しない。
//! ジョブのあいだで合図が立っていれば、再開位置を記録して、128 + 番号で終わる。

use conductor_core::{
    Classification, Decision, PipelineState, RunMode, classify_and_decide, decide_before_run,
    resume_point_on_interrupt,
};
use conductor_runtime::{
    Durability, StateLocation, apply_interrupt, apply_outcome, apply_skip, initial_state,
};

use crate::ctx::Ctx;
use crate::exit::{
    EXIT_OK, EXIT_REJECTED, EXIT_STATE, EXIT_USAGE, exit_code_for, signal_exit_code,
};
use crate::report::{DisplayEvent, StateContext};
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

    // ステートが空のあいだは `None`。最初のジョブが `Run` と決まったあとに、初期ステートを書く。
    let mut current: Option<PipelineState> = session.state.clone();

    for job in &definition.jobs {
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
                    .unwrap_or_else(|| initial_state(definition, path));
                let recorded = apply_skip(&base, &job.id, reason, definition);
                if recorded != base {
                    if let Err(code) = ctx.write(
                        path,
                        &recorded,
                        Durability::Durable,
                        StateContext::JobNotStarted,
                    ) {
                        return code;
                    }
                }
                ctx.show(&DisplayEvent::JobSkipped {
                    job_id: &job.id,
                    reason,
                });
                let completed = recorded.completed;
                current = Some(recorded);
                if completed {
                    return finish(ctx, path, StateContext::JobNotStarted);
                }
            }
            Decision::Run => {
                let base = match current.take() {
                    Some(state) => state,
                    None => {
                        let initial = initial_state(definition, path);
                        if let Err(code) = ctx.write(
                            path,
                            &initial,
                            Durability::Fast,
                            StateContext::JobNotStarted,
                        ) {
                            return code;
                        }
                        initial
                    }
                };

                let execution = ctx.runtime.run_job(job, &session.signal);
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
                if let Err(code) = ctx.write(path, &recorded, Durability::Durable, after) {
                    return code;
                }
                let is_error = outcome.classification == Some(Classification::Error);
                if is_error || outcome.classification.is_none() {
                    // エラーで停止する(再試行しない)。中断で起動しなかった場合も、ここで終える。
                    return exit_code_for(outcome.reported_exit_code, is_error);
                }
                if next.completed {
                    return finish(ctx, path, after);
                }

                // ジョブのあいだの中断(BR4.2)。次のジョブを起動せず、再開位置を記録する。
                if let Some(number) = session.signal.signal_number() {
                    let point = resume_point_on_interrupt(definition, &recorded.job_states);
                    let interrupted = apply_interrupt(&recorded, &point);
                    if let Err(code) = ctx.write(
                        path,
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
    let state_path = StateLocation::of(path).path().to_path_buf();
    ctx.show(&DisplayEvent::CompletionUndetermined {
        state_path: &state_path,
    });
    EXIT_STATE
}

/// 完了のあとの初期化(BR3.3)。失敗は 65。
fn finish(ctx: &mut Ctx<'_>, path: &std::path::Path, context: StateContext) -> i32 {
    match ctx.reset(path, context) {
        Ok(()) => EXIT_OK,
        Err(code) => code,
    }
}
