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

//! ジョブの一連の流れのテスト。conductor-core の判断(`decide_from_state`、`classify_and_decide`)と、
//! この Unit の `run_job`(補助の実行ファイル)・`apply_*`・`write_state`・`read_state`・
//! `reset_state` を組み合わせて、Cli の流れを、小さな試験用の関数で再現する(契約 C2 の順序の続き)。
//!
//! 補助は、今の cli(`run.rs`・`pipeline.rs`・`step.rs`)と同じ順にする: スキップは変わったときだけ
//! 書く、削除は `remove_on_complete` のときだけ(既定は完了のステートを残し、`read_state` は空として
//! 返す)、`pipeline` はステートを 1 回だけ読み、定義の記述順に決める。

mod common;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use common::without_run_id::{initial_state, read_state, reset_state, write_state};
use common::*;
use conductor_core::{
    Classification, ConsistencyResult, Decision, ExecutionResult, JobDefinition, JobOutcome,
    JobStatus, MismatchKind, NextStep, PipelineDefinition, PipelineState, RunMode, SkipCause,
    Transition, TransitionOn, TransitionTarget, apply_outcome, apply_skip, check_consistency,
    classify_and_decide, decide_from_state,
};
// run ID なしの呼び出し(この統合テストは、run ID を使わない流れ)は、`common::without_run_id`。
use conductor_runtime::{
    Durability, InterruptSignal, StateError, StateErrorKind, check_command, run_job,
};

/// 試験用の場面: 一時ディレクトリ、定義ファイルのパス、ジョブの起動の記録のファイル。
struct Scene {
    dir: tempfile::TempDir,
}

impl Scene {
    fn new() -> Self {
        Self {
            dir: tempfile::tempdir().expect("一時ディレクトリ"),
        }
    }
    fn def_path(&self) -> PathBuf {
        self.dir.path().join("conductor.toml")
    }
    fn state_path(&self) -> PathBuf {
        self.dir.path().join("conductor.toml.state")
    }
    fn log(&self) -> PathBuf {
        self.dir.path().join("ran.log")
    }
    /// 起動したジョブの記録(起動した順)。
    fn ran(&self) -> Vec<String> {
        lines(&self.log())
            .iter()
            .filter_map(|l| l.strip_prefix("ran=").map(str::to_string))
            .collect()
    }
    /// ジョブ `id` を、終了コード `code` で終わる、補助の実行ファイルとして定義する。
    fn job(&self, id: &str, code: i32) -> JobDefinition {
        job(id, helper_command(&self.log(), &format!("run {id} {code}")))
    }
    fn definition(&self, jobs: Vec<JobDefinition>) -> PipelineDefinition {
        PipelineDefinition {
            name: "nightly".into(),
            description: None,
            run_id_env: None,
            remove_on_complete: false,
            format_version: 1,
            job: jobs,
            env: vec![],
            env_file: vec![],
            log_dir: None,
        }
    }
}

fn on_success_go(mut j: JobDefinition, next: TransitionTarget) -> JobDefinition {
    j.transitions.push(Transition {
        on: TransitionOn::Success,
        next,
    });
    j
}

#[derive(Debug, PartialEq)]
enum Failure {
    State(StateErrorKind),
    Mismatch(MismatchKind),
    Rejected,
}

impl From<StateError> for Failure {
    fn from(e: StateError) -> Self {
        Failure::State(e.kind)
    }
}

/// 1 つのジョブの結果(スキップは `ran = false`)。
#[derive(Debug)]
struct Done {
    /// スケジューラへ返す終了コード。
    exit: i32,
    completed: bool,
    /// ジョブを起動したか。
    ran: bool,
}

/// 呼び出しの指定。
#[derive(Debug, Clone, Copy, Default)]
struct Call {
    /// 実効の `remove_on_complete`(真なら、完了でステートを消す。既定は偽で、完了のステートは残る)。
    remove_on_complete: bool,
    /// 真なら、ジョブの終了のあと、記録の前に止まる(強制終了の再現)。
    stop_before_record: bool,
}

/// ステートを読み、定義と照合する(cli の `read_checked_state`。完了のステートは空として読む)。
fn load_checked(
    def_path: &Path,
    definition: &PipelineDefinition,
) -> Result<Option<PipelineState>, Failure> {
    let loaded = read_state(def_path)?;
    if let Some(state) = &loaded
        && let ConsistencyResult::Mismatch(kind) = check_consistency(state, definition)
    {
        return Err(Failure::Mismatch(kind));
    }
    Ok(loaded)
}

/// スキップを記録する(cli の `step::record_skip`)。土台(無ければ初期ステート)に `apply_skip` し、
/// 変わったときだけ `Durable` で書く。記録と、変わったかを返す。
fn record_skip(
    def_path: &Path,
    definition: &PipelineDefinition,
    base: Option<&PipelineState>,
    job_id: &str,
    reason: SkipCause,
) -> Result<(PipelineState, bool), Failure> {
    let base = base
        .cloned()
        .unwrap_or_else(|| initial_state(definition, def_path));
    let recorded = apply_skip(&base, job_id, reason, definition);
    let changed = recorded != base;
    if changed {
        write_state(def_path, &recorded, Durability::Durable)?;
    }
    Ok((recorded, changed))
}

/// 実行した 1 つのジョブの記録(cli の `step::StepResult`)。
struct Step {
    recorded: PipelineState,
    outcome: JobOutcome,
    next: NextStep,
    started: bool,
}

/// ジョブを 1 つ実行して記録する(cli の `step::run_step`)。空なら初期ステートを `Fast` で書き、
/// 実行・分類・記録・`Durable` で書く。`stop_before_record` なら、記録の前に `None` で止まる。
fn run_step(
    def_path: &Path,
    definition: &PipelineDefinition,
    base: Option<PipelineState>,
    job: &JobDefinition,
    mode: RunMode,
    signal: &InterruptSignal,
    stop_before_record: bool,
) -> Result<Option<Step>, Failure> {
    let base = match base {
        Some(state) => state,
        None => {
            let initial = initial_state(definition, def_path);
            write_state(def_path, &initial, Durability::Fast)?;
            initial
        }
    };
    let line = check_command(&job.command).expect("分けられる");
    let execution = run_job(&line, &[], signal, None).0;
    if stop_before_record {
        return Ok(None);
    }
    let (outcome, next) = classify_and_decide(job, &execution, &base.job_states, definition, mode);
    let recorded = apply_outcome(&base, &job.id, &execution, &outcome, &next);
    write_state(def_path, &recorded, Durability::Durable)?;
    Ok(Some(Step {
        recorded,
        outcome,
        next,
        started: !matches!(execution.result, ExecutionResult::NotStarted { .. }),
    }))
}

/// 完了のあとの削除(cli の `step::remove_if_requested`)。`remove_on_complete` が真のときだけ消す。
fn remove_if_requested(def_path: &Path, remove_on_complete: bool) -> Result<(), Failure> {
    if remove_on_complete {
        reset_state(def_path)?;
    }
    Ok(())
}

/// Cli の `run <job-id>` の 1 回の呼び出しを再現する(既定の指定。完了のステートは残る)。
/// `stop_before_record` が `true` なら、ジョブの終了のあと、記録の前に止まる(強制終了の再現)。
fn invoke_inner(
    def_path: &Path,
    definition: &PipelineDefinition,
    job_id: &str,
    mode: RunMode,
    signal: &InterruptSignal,
    stop_before_record: bool,
) -> Result<Done, Failure> {
    let call = Call {
        stop_before_record,
        ..Call::default()
    };
    invoke_with(def_path, definition, job_id, mode, signal, call)
}

/// Cli の `run <job-id>`(`run_individual`)と同じ順で、1 つのジョブを処理する。
fn invoke_with(
    def_path: &Path,
    definition: &PipelineDefinition,
    job_id: &str,
    mode: RunMode,
    signal: &InterruptSignal,
    call: Call,
) -> Result<Done, Failure> {
    let state = load_checked(def_path, definition)?;
    match decide_from_state(job_id, definition, state.as_ref()) {
        Decision::Reject(_) => Err(Failure::Rejected),
        Decision::Skip(reason) => {
            let (recorded, changed) =
                record_skip(def_path, definition, state.as_ref(), job_id, reason)?;
            // run は、記録が変わって完了したときだけ、消すかを決める。
            if changed && recorded.completed {
                remove_if_requested(def_path, call.remove_on_complete)?;
            }
            Ok(Done {
                exit: 0,
                completed: recorded.completed,
                ran: false,
            })
        }
        Decision::Run => {
            let job = definition
                .job
                .iter()
                .find(|j| j.id == job_id)
                .expect("定義にある");
            let Some(step) = run_step(
                def_path,
                definition,
                state,
                job,
                mode,
                signal,
                call.stop_before_record,
            )?
            else {
                return Ok(Done {
                    exit: -1,
                    completed: false,
                    ran: true,
                });
            };
            if step.next.completed {
                remove_if_requested(def_path, call.remove_on_complete)?;
            }
            Ok(Done {
                exit: step.outcome.reported_exit_code,
                completed: step.next.completed,
                ran: step.started,
            })
        }
    }
}

fn invoke(
    scene: &Scene,
    definition: &PipelineDefinition,
    job_id: &str,
    mode: RunMode,
) -> Result<Done, Failure> {
    invoke_call(scene, definition, job_id, mode, Call::default())
}

fn invoke_call(
    scene: &Scene,
    definition: &PipelineDefinition,
    job_id: &str,
    mode: RunMode,
    call: Call,
) -> Result<Done, Failure> {
    let def = scene.def_path();
    let definition = definition.clone();
    let job_id = job_id.to_string();
    run_with_limit(
        || {},
        move || {
            invoke_with(
                &def,
                &definition,
                &job_id,
                mode,
                &InterruptSignal::new(),
                call,
            )
        },
    )
}

/// Cli の `pipeline <name>`(`run_pipeline`)と同じ順で、一連の実行を再現する。ステートを 1 回だけ読み、
/// 照合し、定義の記述順に各ジョブを決める(スキップ・実行・エラーでの停止・完了)。各ジョブの結果を返す。
fn pipeline_inner(
    def_path: &Path,
    definition: &PipelineDefinition,
    remove_on_complete: bool,
) -> Result<Vec<Done>, Failure> {
    let signal = InterruptSignal::new();
    let mut current = load_checked(def_path, definition)?;
    let mut done = Vec::new();
    for job in &definition.job {
        match decide_from_state(&job.id, definition, current.as_ref()) {
            Decision::Reject(_) => return Err(Failure::Rejected),
            Decision::Skip(reason) => {
                // pipeline は、記録が変わらなくても、完了なら消すかを決める。
                let (recorded, _changed) =
                    record_skip(def_path, definition, current.as_ref(), &job.id, reason)?;
                let completed = recorded.completed;
                done.push(Done {
                    exit: 0,
                    completed,
                    ran: false,
                });
                current = Some(recorded);
                if completed {
                    remove_if_requested(def_path, remove_on_complete)?;
                    return Ok(done);
                }
            }
            Decision::Run => {
                let step = run_step(
                    def_path,
                    definition,
                    current.take(),
                    job,
                    RunMode::Pipeline,
                    &signal,
                    false,
                )?
                .expect("記録の前に止めない");
                let stop = step.outcome.classification == Some(Classification::Error)
                    || step.outcome.classification.is_none();
                done.push(Done {
                    exit: step.outcome.reported_exit_code,
                    completed: step.next.completed,
                    ran: step.started,
                });
                if stop {
                    // エラーで停止する(再試行しない)。
                    return Ok(done);
                }
                if step.next.completed {
                    remove_if_requested(def_path, remove_on_complete)?;
                    return Ok(done);
                }
                current = Some(step.recorded);
            }
        }
    }
    panic!("記述順のジョブがなくなったのに、完了していない");
}

/// 一連の実行(既定の指定。完了のステートは残る)。
fn pipeline(scene: &Scene, definition: &PipelineDefinition) -> Result<Vec<Done>, Failure> {
    pipeline_with(scene, definition, false)
}

fn pipeline_with(
    scene: &Scene,
    definition: &PipelineDefinition,
    remove_on_complete: bool,
) -> Result<Vec<Done>, Failure> {
    let def = scene.def_path();
    let definition = definition.clone();
    run_with_limit(
        || {},
        move || pipeline_inner(&def, &definition, remove_on_complete),
    )
}

/// 完了のステートが残っていることを確かめる(ファイルはあり、`read_state` は空として返す)。
fn assert_completed_state_kept(scene: &Scene) {
    assert!(scene.state_path().exists(), "完了のステートが残る");
    assert_eq!(
        read_state(&scene.def_path()).expect("読める"),
        None,
        "完了のステートは空として読む"
    );
}

#[test]
fn ac4_2_1_個別実行で最後まで行くと完了のステートが残る() {
    let scene = Scene::new();
    let def = scene.definition(vec![
        scene.job("a", 0),
        scene.job("b", 0),
        scene.job("c", 0),
    ]);
    for id in ["a", "b"] {
        let d = invoke(&scene, &def, id, RunMode::Individual).expect("実行できる");
        assert_eq!((d.exit, d.completed), (0, false));
        assert!(scene.state_path().exists(), "{id} のあとステートがある");
    }
    let state = read_state(&scene.def_path())
        .expect("読める")
        .expect("ある");
    assert_eq!(state.job_states[1].status, JobStatus::Success);
    assert_eq!(state.job_states[2].status, JobStatus::Pending);

    let d = invoke(&scene, &def, "c", RunMode::Individual).expect("実行できる");
    assert!(d.completed);
    assert_completed_state_kept(&scene);
    assert_eq!(scene.ran(), ["a", "b", "c"]);

    // `remove_on_complete` が真なら、完了でステートが消える(完了のステートは空なので、a から動く)。
    let removing = Call {
        remove_on_complete: true,
        ..Call::default()
    };
    for id in ["a", "b"] {
        let d = invoke_call(&scene, &def, id, RunMode::Individual, removing).expect("実行できる");
        assert!(!d.completed);
    }
    let d = invoke_call(&scene, &def, "c", RunMode::Individual, removing).expect("実行できる");
    assert!(d.completed);
    assert!(!scene.state_path().exists(), "完了でステートが消える");
    assert_eq!(scene.ran(), ["a", "b", "c", "a", "b", "c"]);
}

#[test]
fn 順序外の呼び出しは拒否されステートを作らない() {
    let scene = Scene::new();
    let def = scene.definition(vec![scene.job("a", 0), scene.job("b", 0)]);
    assert_eq!(
        invoke(&scene, &def, "b", RunMode::Individual).expect_err("拒否される"),
        Failure::Rejected
    );
    assert!(!scene.state_path().exists());
    assert!(scene.ran().is_empty());
}

#[test]
fn ac4_2_2_end_のあと最後のジョブのスキップで完了する() {
    let scene = Scene::new();
    let def = scene.definition(vec![
        on_success_go(scene.job("a", 0), TransitionTarget::End),
        scene.job("b", 0),
    ]);
    let d = invoke(&scene, &def, "a", RunMode::Individual).expect("実行できる");
    assert!(!d.completed);
    let state = read_state(&scene.def_path())
        .expect("読める")
        .expect("ある");
    assert!(state.end_reached);

    let d = invoke(&scene, &def, "b", RunMode::Individual).expect("スキップできる");
    assert_eq!((d.exit, d.ran, d.completed), (0, false, true));
    assert_completed_state_kept(&scene);
    assert_eq!(scene.ran(), ["a"], "b は起動されない");
}

#[test]
fn ac4_2_3_end_のあとの個別実行は最後のジョブのスキップで初めて完了する() {
    let scene = Scene::new();
    let def = scene.definition(vec![
        on_success_go(scene.job("a", 0), TransitionTarget::End),
        scene.job("b", 0),
        scene.job("c", 0),
    ]);
    invoke(&scene, &def, "a", RunMode::Individual).expect("実行できる");

    let d = invoke(&scene, &def, "b", RunMode::Individual).expect("スキップできる");
    assert!(!d.completed, "C が残っている");
    let state = read_state(&scene.def_path())
        .expect("読める")
        .expect("ある");
    assert_eq!(state.job_states[1].status, JobStatus::Skipped);
    assert_eq!(state.resume_job_id.as_deref(), Some("c"));

    let d = invoke(&scene, &def, "c", RunMode::Individual).expect("スキップできる");
    assert!(d.completed);
    assert_completed_state_kept(&scene);
    assert_eq!(scene.ran(), ["a"]);
}

#[test]
fn 遷移によるスキップのあと最後のジョブの実行で完了する() {
    let scene = Scene::new();
    let def = scene.definition(vec![
        on_success_go(scene.job("a", 0), TransitionTarget::Job("c".into())),
        scene.job("b", 0),
        scene.job("c", 0),
    ]);
    invoke(&scene, &def, "a", RunMode::Individual).expect("実行できる");
    let state = read_state(&scene.def_path())
        .expect("読める")
        .expect("ある");
    assert_eq!(state.job_states[1].status, JobStatus::Skipped);

    let d = invoke(&scene, &def, "b", RunMode::Individual).expect("スキップできる");
    assert!(!d.ran && !d.completed);
    let d = invoke(&scene, &def, "c", RunMode::Individual).expect("実行できる");
    assert!(d.completed);
    assert_eq!(scene.ran(), ["a", "c"]);
    assert_completed_state_kept(&scene);
}

#[test]
fn ac4_2_4_一連の実行の_end_で完了し完了のステートが残る() {
    let scene = Scene::new();
    let def = scene.definition(vec![
        on_success_go(scene.job("a", 0), TransitionTarget::End),
        scene.job("b", 0),
        scene.job("c", 0),
    ]);
    let done = pipeline(&scene, &def).expect("実行できる");
    assert_eq!(done.len(), 1);
    assert!(done[0].completed);
    assert_completed_state_kept(&scene);
    assert_eq!(scene.ran(), ["a"]);
}

#[test]
fn ac4_3_1_ac4_3_2_エラーからの再開() {
    let scene = Scene::new();
    let failing = scene.definition(vec![
        scene.job("a", 0),
        scene.job("b", 3),
        scene.job("c", 0),
    ]);
    let done = pipeline(&scene, &failing).expect("実行できる");
    assert_eq!(done.last().map(|d| d.exit), Some(3));
    assert_eq!(scene.ran(), ["a", "b"], "エラーで止まる");
    let state = read_state(&scene.def_path())
        .expect("読める")
        .expect("ある");
    assert_eq!(state.resume_job_id.as_deref(), Some("b"));
    assert_eq!(state.job_states[1].status, JobStatus::Error);
    assert_eq!(state.job_states[1].exit_code, Some(3));

    // 直して、もう一度。エラーのジョブから再開し、成功済みの a は動かない。
    let fixed = scene.definition(vec![
        scene.job("a", 0),
        scene.job("b", 0),
        scene.job("c", 0),
    ]);
    let done = pipeline(&scene, &fixed).expect("実行できる");
    assert!(done.last().is_some_and(|d| d.completed));
    assert_eq!(scene.ran(), ["a", "b", "b", "c"]);
    assert_completed_state_kept(&scene);
}

#[test]
fn ac4_3_1_個別実行でも_エラーのジョブを再実行して続きを呼べる() {
    let scene = Scene::new();
    let failing = scene.definition(vec![scene.job("a", 0), scene.job("b", 2)]);
    invoke(&scene, &failing, "a", RunMode::Individual).expect("実行できる");
    let d = invoke(&scene, &failing, "b", RunMode::Individual).expect("実行できる");
    assert_eq!(d.exit, 2);
    // 成功済みの a は、alreadyExecuted でスキップされ、ステートは変わらない。
    let before = read_state(&scene.def_path()).expect("読める");
    let d = invoke(&scene, &failing, "a", RunMode::Individual).expect("スキップできる");
    assert!(!d.ran);
    assert_eq!(read_state(&scene.def_path()).expect("読める"), before);

    let fixed = scene.definition(vec![scene.job("a", 0), scene.job("b", 0)]);
    let d = invoke(&scene, &fixed, "b", RunMode::Individual).expect("実行できる");
    assert!(d.completed);
}

#[test]
fn ac4_3_3_reset_のあとは先頭から() {
    let scene = Scene::new();
    let failing = scene.definition(vec![scene.job("a", 0), scene.job("b", 3)]);
    pipeline(&scene, &failing).expect("実行できる");
    assert!(scene.state_path().exists());
    reset_state(&scene.def_path()).expect("初期化できる");
    pipeline(&scene, &failing).expect("実行できる");
    assert_eq!(scene.ran(), ["a", "b", "a", "b"], "a から、やり直す");
}

#[test]
fn ac4_3_4_自動の再実行はない() {
    let scene = Scene::new();
    let failing = scene.definition(vec![scene.job("a", 5), scene.job("b", 0)]);
    let done = pipeline(&scene, &failing).expect("実行できる");
    assert_eq!(done.len(), 1);
    assert_eq!(scene.ran(), ["a"], "失敗したジョブは、1 回だけ動く");
}

#[test]
fn ac4_4_1_から_3_食い違いはジョブを実行せず_reset_で回復する() {
    let scene = Scene::new();
    let base = scene.definition(vec![
        scene.job("a", 0),
        scene.job("b", 0),
        scene.job("c", 0),
    ]);
    invoke(&scene, &base, "a", RunMode::Individual).expect("実行できる");

    // 改名
    let renamed = scene.definition(vec![
        scene.job("a", 0),
        scene.job("x", 0),
        scene.job("c", 0),
    ]);
    assert_eq!(
        invoke(&scene, &renamed, "b", RunMode::Individual).expect_err("食い違い"),
        Failure::Mismatch(MismatchKind::JobIds)
    );
    // 追加
    let added = scene.definition(vec![
        scene.job("a", 0),
        scene.job("b", 0),
        scene.job("c", 0),
        scene.job("d", 0),
    ]);
    assert_eq!(
        invoke(&scene, &added, "b", RunMode::Individual).expect_err("食い違い"),
        Failure::Mismatch(MismatchKind::JobIds)
    );
    // 並びだけ
    let reordered = scene.definition(vec![
        scene.job("b", 0),
        scene.job("a", 0),
        scene.job("c", 0),
    ]);
    assert_eq!(
        invoke(&scene, &reordered, "b", RunMode::Individual).expect_err("食い違い"),
        Failure::Mismatch(MismatchKind::JobOrder)
    );
    // パイプライン名
    let mut other = base.clone();
    other.name = "other".into();
    assert_eq!(
        invoke(&scene, &other, "b", RunMode::Individual).expect_err("食い違い"),
        Failure::Mismatch(MismatchKind::PipelineName)
    );
    assert_eq!(scene.ran(), ["a"], "食い違いでは、ジョブを実行しない");

    reset_state(&scene.def_path()).expect("初期化できる");
    invoke(&scene, &renamed, "a", RunMode::Individual).expect("reset のあとは実行できる");
    assert_eq!(scene.ran(), ["a", "a"]);
}

#[test]
fn 壊れたステートではジョブを実行しない() {
    let scene = Scene::new();
    let def = scene.definition(vec![scene.job("a", 0)]);
    std::fs::write(scene.state_path(), "= = broken").expect("書ける");
    assert_eq!(
        invoke(&scene, &def, "a", RunMode::Individual).expect_err("拒否される"),
        Failure::State(StateErrorKind::Corrupt)
    );
    assert!(scene.ran().is_empty());
    reset_state(&scene.def_path()).expect("初期化できる");
    invoke(&scene, &def, "a", RunMode::Individual).expect("実行できる");
    assert_eq!(scene.ran(), ["a"]);
}

#[test]
fn 完了のまま残ったステートは先頭から再実行できる() {
    let scene = Scene::new();
    let def = scene.definition(vec![scene.job("a", 0), scene.job("b", 0)]);
    let mut done = initial_state(&def, &scene.def_path());
    done.job_states
        .iter_mut()
        .for_each(|j| j.status = JobStatus::Success);
    done.completed = true;
    write_state(&scene.def_path(), &done, Durability::Durable).expect("書ける");
    // 空として扱われるため、alreadyExecuted でスキップされず、a から動く。
    let d = invoke(&scene, &def, "a", RunMode::Individual).expect("実行できる");
    assert!(d.ran);
    assert_eq!(scene.ran(), ["a"]);
}

#[test]
fn ac2_1_6_成功コードに含まれない_0_はエラーでステートにエラーの位置が残る() {
    let scene = Scene::new();
    let mut a = scene.job("a", 0);
    a.success_exit_codes = Some(vec![2]);
    let def = scene.definition(vec![a, scene.job("b", 0)]);
    let d = invoke(&scene, &def, "a", RunMode::Individual).expect("実行できる");
    assert_ne!(d.exit, 0);
    let state = read_state(&scene.def_path())
        .expect("読める")
        .expect("ある");
    assert_eq!(state.job_states[0].status, JobStatus::Error);
    assert_eq!(state.job_states[0].exit_code, Some(0));
    assert_eq!(state.resume_job_id.as_deref(), Some("a"));
}

#[test]
fn 起動に失敗したジョブはエラーとして記録される() {
    let scene = Scene::new();
    let def = scene.definition(vec![
        job("a", "conductor-no-such-command-zzz"),
        scene.job("b", 0),
    ]);
    let d = invoke(&scene, &def, "a", RunMode::Individual).expect("実行できる");
    assert_ne!(d.exit, 0);
    let state = read_state(&scene.def_path())
        .expect("読める")
        .expect("ある");
    assert_eq!(state.job_states[0].status, JobStatus::Error);
    assert_eq!(state.job_states[0].exit_code, None);
    assert_eq!(state.resume_job_id.as_deref(), Some("a"));
}

#[test]
fn ステートの場所が読めなければジョブを起動しない() {
    let scene = Scene::new();
    let def = scene.definition(vec![scene.job("a", 0)]);
    std::fs::create_dir(scene.state_path()).expect("作れる");
    assert_eq!(
        invoke(&scene, &def, "a", RunMode::Individual).expect_err("失敗する"),
        // ディレクトリは、読もうとして失敗する(読めない)。
        Failure::State(StateErrorKind::Unreadable)
    );
    assert!(scene.ran().is_empty());
}

/// 読めるが書けない場所(読み取り専用のディレクトリ)では、最初のジョブの前に、writeFailed で止まる(BR2.9)。
#[cfg(unix)]
#[test]
fn 書けないディレクトリでは最初のジョブの前に止まる() {
    use std::os::unix::fs::PermissionsExt;
    if is_root() {
        eprintln!("スキップ: root のため、権限による書き込みの失敗を試せません");
        return;
    }
    let scene = Scene::new();
    let def = scene.definition(vec![scene.job("a", 0)]);
    let ro = scene.dir.path().join("ro");
    std::fs::create_dir(&ro).expect("作れる");
    std::fs::set_permissions(&ro, std::fs::Permissions::from_mode(0o555))
        .expect("権限を変えられる");
    let result = invoke_inner(
        &ro.join("conductor.toml"),
        &def,
        "a",
        RunMode::Individual,
        &InterruptSignal::new(),
        false,
    );
    std::fs::set_permissions(&ro, std::fs::Permissions::from_mode(0o755)).expect("戻せる");
    assert_eq!(
        result.expect_err("止まる"),
        Failure::State(StateErrorKind::WriteFailed)
    );
    assert!(scene.ran().is_empty(), "ジョブは起動されない");
}

#[test]
fn ac2_5_2_排他制御はなく_2_つの実行がそれぞれ動く() {
    let scene = Arc::new(Scene::new());
    let def = scene.definition(vec![scene.job("a", 0), scene.job("b", 0)]);
    let handles: Vec<_> = (0..2)
        .map(|_| {
            let (scene, def) = (scene.clone(), def.clone());
            std::thread::spawn(move || {
                invoke_inner(
                    &scene.def_path(),
                    &def,
                    "a",
                    RunMode::Individual,
                    &InterruptSignal::new(),
                    false,
                )
            })
        })
        .collect();
    for h in handles {
        let r = h.join().expect("スレッドが終わる");
        // 同時の読み書きでの失敗(置き換えの競合)は許すが、ステートが壊れた内容にはならない。
        if let Err(Failure::State(kind)) = &r {
            assert!(
                matches!(
                    kind,
                    StateErrorKind::WriteFailed | StateErrorKind::RemoveFailed
                ),
                "{kind:?}"
            );
        }
    }
    assert!(!scene.ran().is_empty());
    assert!(read_state(&scene.def_path()).is_ok(), "ステートは壊れない");
}

#[test]
fn at_least_once_終了のあと書き込みの前に止まるとジョブは再実行される() {
    let scene = Scene::new();
    let def = scene.definition(vec![scene.job("a", 0), scene.job("b", 0)]);
    // a が動いたが、記録の前に止まった。
    let d = invoke_inner(
        &scene.def_path(),
        &def,
        "a",
        RunMode::Individual,
        &InterruptSignal::new(),
        true,
    )
    .expect("実行できる");
    assert!(d.ran);
    let state = read_state(&scene.def_path())
        .expect("読める")
        .expect("初期ステートはある");
    assert!(
        state
            .job_states
            .iter()
            .all(|j| j.status == JobStatus::Pending)
    );

    // 次の呼び出しでは、a は未実行として扱われ、再実行される(冪等でないジョブは、2 回動きうる)。
    invoke(&scene, &def, "a", RunMode::Individual).expect("実行できる");
    assert_eq!(scene.ran(), ["a", "a"]);
    let state = read_state(&scene.def_path())
        .expect("読める")
        .expect("ある");
    assert_eq!(state.job_states[0].status, JobStatus::Success);
}

#[test]
fn 起動前の中断はジョブを起動せずエラーとして記録しない() {
    let scene = Scene::new();
    let def = scene.definition(vec![scene.job("a", 0), scene.job("b", 0)]);
    let signal = InterruptSignal::new();
    signal.raise(15);
    let d = invoke_inner(
        &scene.def_path(),
        &def,
        "a",
        RunMode::Individual,
        &signal,
        false,
    )
    .expect("実行できる");
    assert!(!d.ran);
    assert!(scene.ran().is_empty());
    let state = read_state(&scene.def_path())
        .expect("読める")
        .expect("ある");
    assert!(
        state
            .job_states
            .iter()
            .all(|j| j.status == JobStatus::Pending),
        "エラーとしては記録しない"
    );
}
