//! 個別実行(`conductor run <job-id>`。W2、BR2.1〜BR2.6、BR4.x、BR5.x)。
//!
//! 順序は固定(NFR2.4): 検査 → ステートの読み込みと照合 → `decide_before_run` →
//! (Run のとき、ステートが空なら)初期ステート(`Fast`)→ `run_job` → `classify_and_decide` →
//! `apply_outcome` → `write_state`(`Durable`)→ 完了なら `reset_state`。
//! 初期ステートは、`Run` が決まったあとにだけ書く(Reject・Skip の前には書かない)。

use conductor_core::{Classification, Decision, RunMode, classify_and_decide, decide_before_run};
use conductor_runtime::{Durability, apply_outcome, apply_skip, initial_state};

use crate::ctx::Ctx;
use crate::exit::{EXIT_OK, EXIT_REJECTED, EXIT_USAGE, exit_code_for};
use crate::report::{DisplayEvent, StateContext};
use crate::types::{CliCommand, UsageError, UsageErrorKind};

/// `conductor run <job-id>` を実行し、終了コードを返す。
pub(crate) fn run_individual(ctx: &mut Ctx<'_>, cmd: &CliCommand) -> i32 {
    let Some(job_id) = cmd.job_id.as_deref() else {
        let error = UsageError::new(UsageErrorKind::MissingArgument);
        ctx.show(&DisplayEvent::Usage(&error));
        return EXIT_USAGE;
    };
    let session = match ctx.prepare(cmd, None, Some(job_id)) {
        Ok(session) => session,
        Err(code) => return code,
    };
    let definition = &session.definition;
    let path = &session.definition_path;

    let (states, resume, end_reached) = match &session.state {
        Some(state) => (
            state.job_states.as_slice(),
            state.resume_job_id.as_deref(),
            state.end_reached,
        ),
        None => (&[][..], None, false),
    };
    let decision = decide_before_run(job_id, definition, states, resume, end_reached);

    match decision {
        Decision::Reject(reason) => {
            ctx.show(&DisplayEvent::Rejected { job_id, reason });
            EXIT_REJECTED
        }
        Decision::Skip(reason) => {
            let base = session
                .state
                .clone()
                .unwrap_or_else(|| initial_state(definition, path));
            let recorded = apply_skip(&base, job_id, reason, definition);
            if recorded != base {
                if let Err(code) = ctx.write(
                    path,
                    &recorded,
                    Durability::Durable,
                    StateContext::JobNotStarted,
                ) {
                    return code;
                }
                if recorded.completed {
                    if let Err(code) = ctx.reset(path, StateContext::JobNotStarted) {
                        return code;
                    }
                }
            }
            ctx.show(&DisplayEvent::JobSkipped { job_id, reason });
            EXIT_OK
        }
        Decision::Run => {
            let base = match session.state.clone() {
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
            let Some(index) = definition.job_index(job_id) else {
                // 検査済み(ジョブIDは定義にある)。ここには来ない。
                return EXIT_REJECTED;
            };
            let job = &definition.jobs[index];

            let execution = ctx.runtime.run_job(job, &session.signal);
            let (outcome, next) = classify_and_decide(
                job,
                &execution,
                &base.job_states,
                definition,
                RunMode::Individual,
            );
            ctx.report_execution(job, &execution, &outcome, &next);

            let recorded = apply_outcome(&base, job_id, &execution, &outcome, &next);
            if let Err(code) = ctx.write(
                path,
                &recorded,
                Durability::Durable,
                StateContext::JobExecuted,
            ) {
                return code;
            }
            if next.completed {
                if let Err(code) = ctx.reset(path, StateContext::JobExecuted) {
                    return code;
                }
            }
            let is_error = outcome.classification == Some(Classification::Error);
            exit_code_for(outcome.reported_exit_code, is_error)
        }
    }
}
