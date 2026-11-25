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
//! | `run_job_with_retry`(非公開) | `run_step` のリトライつきの実行。実行 → 判断 → 知らせ → 待機 → 繰り返し |
//! | `report_execution`(非公開) | `run_step` の表示。結果・起動の失敗・成功コードに含まれない 0・遷移・中断 |
//!
//! 書く順序、`Durability`、表示の文脈(`StateContext`)は、各関数の説明のとおり。
//! 書き込みの失敗は、表示したうえで、終了コード(65)を `Err` で返す。呼び出し側は、それで終わる。
//!
//! いつ消すか(完了の扱い)と、終了コードの決め方は、`run` と `pipeline` で違うため、呼び出し側に残す
//! (BR1.2)。

use conductor_core::{
    Classification, EnvContext, ExecutionResult, JobDefinition, JobExecution, JobOutcome, NextStep,
    PipelineState, RetryDecision, RunMode, SkipCause, apply_outcome, apply_skip,
    classify_and_decide, initial_state, merge_env, retry_decision,
};
use std::path::{Path, PathBuf};

use conductor_runtime::{
    CommandLine, Durability, InterruptSignal, LogOutcome, LogTarget, WaitOutcome,
};

use crate::ctx::{Ctx, Session};
use crate::exit::outcome_exit_code;
use crate::report::{DisplayEvent, ResultDetail, StateContext};

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

    let (execution, log) = run_job_with_retry(ctx, session, job, command);
    let (outcome, next) = classify_and_decide(job, &execution, &base.job_states, definition, mode);
    report_execution(ctx, job, command, &execution, &log, &outcome, &next);

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

/// ジョブを、リトライを含めて実行する(U2。BR2.4〜BR2.13、FR3.5)。[`run_step`] が呼ぶ。
///
/// 実行 → 判断(`retry_decision`)→(リトライなら)知らせ → 待機 → 繰り返し、の流れで、最後の実行結果を
/// 返す。ステートの記録と、結果の表示は、呼び出し側が、その最後の結果について、これまでどおり 1 回だけ
/// 行う(BR2.10、BR2.12)。リトライの回数と、直前の待ち時間は、この関数のローカルな値で、ステートファイル
/// には書かない。
///
/// `line` は、実行の前の検査で分割したそのジョブのコマンド。リトライの何回目でも、同じものを渡す
/// (分け直さない。BR2.2)。
/// - リトライの設定がない、または対象外の結果なら、1 回実行して、その結果を返す(従来どおり。BR2.9)。
/// - 待機中に中断の合図が立てば、待機を打ち切り、「起動していない、中断された」実行結果
///   (`NotStarted`)を返す。
///   既存の分類で、分類なし・終了コード 128 + 番号・再開位置はそのジョブ、エラーとして記録しない
///   扱いになる(BR2.5)。
///
/// 試行ごとに、起動の前に現在の時刻を 1 回だけ取る(v0.10.0 の BR2.3)。定義に `log_dir` があれば、
/// その時刻と試行の番号でログの指定([`LogTarget`])を作って渡す(BR3.1)。試行の後、ファイルへの
/// 書き込みの失敗と、出力の待ち切れを、1 行ずつ警告する(BR5.2、BR5.3。終了コードは変えない)。
/// 最後の試行のログの結果を、実行結果に添えて返す(ログを作れなかった表示に使う。BR5.1)。
fn run_job_with_retry(
    ctx: &mut Ctx<'_>,
    session: &Session,
    job: &JobDefinition,
    line: &CommandLine,
) -> (JobExecution, LogOutcome) {
    let signal: &InterruptSignal = &session.signal;
    let log_dir = session
        .checked
        .base
        .definition
        .log_dir
        .as_deref()
        .map(|dir| resolve_log_dir(&session.checked.base.path, dir));
    let mut retries_done: u32 = 0;
    let mut last_wait_secs: u64 = 0;
    loop {
        // 試行ごとに、足す環境を作り直す(`CONDUCTOR_ATTEMPT` は、この呼び出しのリトライの回数 + 1。
        // 再開すると 1 に戻る。BR5.2、FR1.10)。
        let checked = &session.checked;
        let env = merge_env(
            &checked.base.definition,
            job,
            &checked.env_files,
            &EnvContext {
                pipeline_name: &checked.base.definition.name,
                job_id: &job.id,
                run_id: checked.run_id.as_ref().map(|r| r.as_str()),
                attempt: retries_done + 1,
            },
        );
        let started_at = ctx.runtime.now();
        let target = log_dir.as_ref().map(|dir| LogTarget {
            dir: dir.clone(),
            job_id: job.id.clone(),
            started_at,
            attempt: retries_done + 1,
        });
        // 引き継いだ環境から消す名前(U3 の BR3.1、BR3.2。足す前に消す)。
        let unset: Vec<&str> = env.removals.iter().map(String::as_str).collect();
        let (execution, log) =
            ctx.runtime
                .run_job(line, &env.additions(), &unset, signal, target.as_ref());
        report_log_warnings(ctx, &log);
        // `Retry` は、リトライの対象になった最後の終了コードと、最大回数を持つ(BR2.4)。
        let RetryDecision::Retry {
            wait_secs,
            last_exit_code,
            max,
        } = retry_decision(job, &execution, retries_done, last_wait_secs)
        else {
            return (execution, log);
        };

        // 待機の前に、常に知らせる(BR2.4)。
        ctx.show(&DisplayEvent::RetryScheduled {
            job_id: &job.id,
            last_exit_code,
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
                return (JobExecution::not_started(number), LogOutcome::default());
            }
        }
    }
}

/// `log_dir` を解決する(相対なら (定義ファイルのディレクトリ)/(パス)、絶対ならそのまま。v0.10.0 の
/// BR1.1。`env_file` と同じ規則)。`~` と変数は展開しない。
fn resolve_log_dir(definition_path: &Path, log_dir: &str) -> PathBuf {
    let dir = Path::new(log_dir);
    if dir.is_absolute() {
        dir.to_path_buf()
    } else {
        definition_path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(dir)
    }
}

/// 試行の後のログの警告(v0.10.0 の BR5.2、BR5.3)。書き込みの失敗と待ち切れを、それぞれ 1 行。
fn report_log_warnings(ctx: &mut Ctx<'_>, log: &LogOutcome) {
    if let Some(path) = &log.write_failed {
        ctx.show(&DisplayEvent::LogWriteFailed { path });
    }
    if let Some(path) = &log.output_cut_short {
        ctx.show(&DisplayEvent::LogOutputCutShort { path });
    }
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
///
/// ログのファイルを作れずに起動しなかったとき(`log.log_not_created`)は、起動の失敗の表示の代わりに、
/// ログを作れない 1 行だけを出す(v0.10.0 の BR5.1)。
fn report_execution(
    ctx: &mut Ctx<'_>,
    job: &JobDefinition,
    command: &CommandLine,
    execution: &JobExecution,
    log: &LogOutcome,
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
        match &log.log_not_created {
            Some(path) => ctx.show(&DisplayEvent::LogNotCreated { path }),
            None => ctx.show(&DisplayEvent::StartFailure {
                job_id: &job.id,
                command: &command.program,
            }),
        };
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
