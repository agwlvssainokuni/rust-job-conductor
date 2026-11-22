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

//! ジョブを、リトライを含めて実行する共通の処理(U2。BR2.4〜BR2.13、FR3.5)。
//!
//! `run`(個別実行)と `pipeline`(一括実行)の両方が呼ぶ、1 つの関数。実行 → 判断(`retry_decision`)→
//! (リトライなら)知らせ → 待機 → 繰り返し、の流れで、最後の実行結果を返す。ステートの記録と、
//! 結果の表示は、呼び出し側が、その最後の結果について、これまでどおり 1 回だけ行う(BR2.10、BR2.12)。
//! リトライの回数と、直前の待ち時間は、この関数のローカルな値で、ステートファイルには書かない。

use conductor_core::{
    EnvContext, JobDefinition, JobExecution, RetryDecision, merge_env, retry_decision,
};
use conductor_runtime::{InterruptSignal, WaitOutcome};

use crate::ctx::{Ctx, Session};
use crate::report::DisplayEvent;

/// ジョブを実行し、リトライの対象なら、知らせて、待って、実行し直す。最後の実行結果を返す。
///
/// - リトライの設定がない、または対象外の結果なら、1 回実行して、その結果を返す(従来どおり。BR2.9)。
/// - 待機中に中断の合図が立てば、待機を打ち切り、「起動していない、中断された」実行結果を返す。
///   既存の分類で、分類なし・終了コード 128 + 番号・再開位置はそのジョブ、エラーとして記録しない
///   扱いになる(BR2.5)。
pub(crate) fn run_job_with_retry(
    ctx: &mut Ctx<'_>,
    session: &Session,
    job: &JobDefinition,
) -> JobExecution {
    let signal: &InterruptSignal = &session.signal;
    let mut retries_done: u32 = 0;
    let mut last_wait_secs: u64 = 0;
    loop {
        // 試行ごとに、足す環境を作り直す(`CONDUCTOR_ATTEMPT` は、この呼び出しのリトライの回数 + 1。
        // 再開すると 1 に戻る。BR5.2、FR1.10)。
        let checked = &session.checked;
        let env = merge_env(
            &checked.definition,
            job,
            &checked.env_files,
            &EnvContext {
                pipeline_name: &checked.definition.name,
                job_id: &job.id,
                run_id: checked.run_id.as_ref().map(|r| r.as_str()),
                attempt: retries_done + 1,
            },
        );
        let execution = ctx.runtime.run_job(job, &env.additions(), signal);
        let RetryDecision::Retry { wait_secs } =
            retry_decision(job, &execution, retries_done, last_wait_secs)
        else {
            return execution;
        };

        // 待機の前に、常に知らせる(BR2.4)。
        let max = job.retry.as_ref().map_or(0, |policy| policy.max);
        ctx.show(&DisplayEvent::RetryScheduled {
            job_id: &job.id,
            last_exit_code: execution.exit_code.unwrap_or_default(),
            retry_number: retries_done + 1,
            max,
            wait_secs,
        });

        match ctx.runtime.wait(wait_secs, signal) {
            WaitOutcome::Elapsed => {
                retries_done += 1;
                last_wait_secs = wait_secs;
            }
            WaitOutcome::Interrupted(number) => {
                return JobExecution {
                    job_id: job.id.clone(),
                    command: job.command.clone(),
                    started: false,
                    exit_code: None,
                    terminated_by_signal: None,
                    start_failure: false,
                    interrupted: true,
                    signal_number: Some(number),
                };
            }
        }
    }
}
