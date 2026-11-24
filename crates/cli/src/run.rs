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
//! 順序は固定(NFR2.4): 検査 → ステートの読み込みと照合 → `decide_from_state` →
//! (Run のとき、ステートが空なら)初期ステート(`Fast`)→ `step` の `run_job_with_retry`(リトライを含む)→
//! `classify_and_decide` → `apply_outcome` → `write_state`(`Durable`)→ 完了で、削除を求められていれば
//! `reset_state`(求められていなければ、完了のステートを残す)。ステートは、最終の結果について 1 回だけ書く。
//! 初期ステートは、`Run` が決まったあとにだけ書く(Reject・Skip の前には書かない)。
//!
//! 1 つのジョブの手順(判断、スキップの記録、実行と記録、削除)は、`pipeline` と共通の
//! `crate::step` を呼ぶ(BR1.1)。いつ消すかと、終了コードの決め方は、ここに残す(BR1.2)。

use conductor_core::{Decision, RunMode, decide_from_state};

use crate::ctx::{Ctx, FlowTarget};
use crate::dry_run::enter_flow;
use crate::exit::{EXIT_OK, EXIT_REJECTED, outcome_exit_code};
use crate::report::{DisplayEvent, StateContext};
use crate::step::{record_skip, remove_if_requested, run_step};
use crate::types::CliCommand;

/// `conductor run <job-id>` を実行し、終了コードを返す。
pub(crate) fn run_individual(ctx: &mut Ctx<'_>, cmd: &CliCommand) -> i32 {
    let job_id = match enter_flow(ctx, cmd, cmd.job_id.as_deref(), FlowTarget::Run) {
        Ok(job_id) => job_id,
        Err(code) => return code,
    };
    let (session, index) = match ctx.prepare_session(|c| c.check_job(cmd, job_id, &[])) {
        Ok(prepared) => prepared,
        Err(code) => return code,
    };
    let definition = &session.checked.base.definition;

    match decide_from_state(job_id, definition, session.checked.state.as_ref()) {
        Decision::Reject(reason) => {
            ctx.show(&DisplayEvent::Rejected { job_id, reason });
            EXIT_REJECTED
        }
        Decision::Skip(reason) => {
            let record = match record_skip(
                ctx,
                &session,
                session.checked.state.as_ref(),
                job_id,
                reason,
            ) {
                Ok(result) => result,
                Err(code) => return code,
            };
            // run は、記録が変わって完了したときだけ、表示の前に消す(pipeline と違う。BR1.2)。
            if record.written
                && record.state.completed
                && let Err(code) = remove_if_requested(ctx, &session, StateContext::JobNotStarted)
            {
                return code;
            }
            ctx.show(&DisplayEvent::JobSkipped { job_id, reason });
            EXIT_OK
        }
        Decision::Run => {
            // 位置は、検査(`prepare_session` に渡した `check_job`)で確かめたもの。
            let job = &definition.job[index];

            let step = match run_step(
                ctx,
                &session,
                session.checked.state.clone(),
                job,
                &session.checked.base.commands[index],
                RunMode::Individual,
            ) {
                Ok(step) => step,
                Err(code) => return code,
            };
            if step.next.completed
                && let Err(code) = remove_if_requested(ctx, &session, step.after)
            {
                return code;
            }
            outcome_exit_code(&step.outcome)
        }
    }
}
