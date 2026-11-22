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
//! (求められていなければ、完了のステートを残す)。`run_job` の呼び出しは、リトライを含む共通の処理
//! (`retry::run_job_with_retry`)に置き換わっている。ステートは、最終の結果について 1 回だけ書く。
//! 初期ステートは、`Run` が決まったあとにだけ書く(Reject・Skip の前には書かない)。
//!
//! 1 つのジョブの手順(判断、スキップの記録、実行と記録、削除)は、`pipeline` と共通の
//! `crate::step` を呼ぶ(BR1.1)。いつ消すかと、終了コードの決め方は、ここに残す(BR1.2)。

use conductor_core::{Classification, Decision, RunMode};

use crate::ctx::Ctx;
use crate::dry_run::{DryRunTarget, run_dry_run};
use crate::exit::{EXIT_OK, EXIT_REJECTED, EXIT_USAGE, exit_code_for};
use crate::report::{DisplayEvent, StateContext};
use crate::step::{decide, record_skip, remove_if_requested, run_step};
use crate::types::{CliCommand, UsageError, UsageErrorKind};

/// `conductor run <job-id>` を実行し、終了コードを返す。
pub(crate) fn run_individual(ctx: &mut Ctx<'_>, cmd: &CliCommand) -> i32 {
    let Some(job_id) = cmd.job_id.as_deref() else {
        let error = UsageError::new(UsageErrorKind::MissingArgument);
        ctx.show(&DisplayEvent::Usage(&error));
        return EXIT_USAGE;
    };
    // `--dry-run`: ジョブを実行せず、ステートに書かず、シグナルも準備せずに、計画を出す(U3)。
    if cmd.dry_run.dry_run {
        return run_dry_run(ctx, cmd, DryRunTarget::Run(job_id));
    }
    let session = match ctx.prepare(cmd, None, Some(job_id)) {
        Ok(session) => session,
        Err(code) => return code,
    };
    let definition = &session.definition;

    match decide(job_id, definition, session.state.as_ref()) {
        Decision::Reject(reason) => {
            ctx.show(&DisplayEvent::Rejected { job_id, reason });
            EXIT_REJECTED
        }
        Decision::Skip(reason) => {
            let (recorded, changed) =
                match record_skip(ctx, &session, session.state.as_ref(), job_id, reason) {
                    Ok(result) => result,
                    Err(code) => return code,
                };
            // run は、記録が変わって完了したときだけ、表示の前に消す(pipeline と違う。BR1.2)。
            if changed
                && recorded.completed
                && let Err(code) = remove_if_requested(ctx, &session, StateContext::JobNotStarted)
            {
                return code;
            }
            ctx.show(&DisplayEvent::JobSkipped { job_id, reason });
            EXIT_OK
        }
        Decision::Run => {
            let Some(index) = definition.job_index(job_id) else {
                // 検査済み(ジョブIDは定義にある)。ここには来ない。
                return EXIT_REJECTED;
            };
            let job = &definition.job[index];

            let step = match run_step(
                ctx,
                &session,
                session.state.clone(),
                job,
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
            let is_error = step.outcome.classification == Some(Classification::Error);
            exit_code_for(step.outcome.reported_exit_code, is_error)
        }
    }
}
