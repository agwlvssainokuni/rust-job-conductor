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

//! 個別実行(`conductor run <job-id>`。W2、BR2.1〜BR2.6、BR4.x、BR5.x)。
//!
//! 順序は固定(NFR2.4): 検査 → ステートの読み込みと照合 → `decide_before_run` →
//! (Run のとき、ステートが空なら)初期ステート(`Fast`)→ `run_job` → `classify_and_decide` →
//! `apply_outcome` → `write_state`(`Durable`)→ 完了で、削除を求められていれば `reset_state`
//! (求められていなければ、完了のステートを残す)。
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
    let run_id = session.run_id.as_ref();

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
                .unwrap_or_else(|| initial_state(definition, path, run_id));
            let recorded = apply_skip(&base, job_id, reason, definition);
            if recorded != base {
                if let Err(code) = ctx.write(
                    path,
                    run_id,
                    &recorded,
                    Durability::Durable,
                    StateContext::JobNotStarted,
                ) {
                    return code;
                }
                if recorded.completed
                    && session.remove_on_complete
                    && let Err(code) = ctx.reset(path, run_id, StateContext::JobNotStarted)
                {
                    return code;
                }
            }
            ctx.show(&DisplayEvent::JobSkipped { job_id, reason });
            EXIT_OK
        }
        Decision::Run => {
            let base = match session.state.clone() {
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
            let Some(index) = definition.job_index(job_id) else {
                // 検査済み(ジョブIDは定義にある)。ここには来ない。
                return EXIT_REJECTED;
            };
            let job = &definition.job[index];

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
            // 起動前の中断(started = false)は、ジョブが実行されていない。
            let after = if execution.started {
                StateContext::JobExecuted
            } else {
                StateContext::JobNotStarted
            };
            if let Err(code) = ctx.write(path, run_id, &recorded, Durability::Durable, after) {
                return code;
            }
            if next.completed
                && session.remove_on_complete
                && let Err(code) = ctx.reset(path, run_id, after)
            {
                return code;
            }
            let is_error = outcome.classification == Some(Classification::Error);
            exit_code_for(outcome.reported_exit_code, is_error)
        }
    }
}
