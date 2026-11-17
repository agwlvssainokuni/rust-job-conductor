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

//! ジョブの一連の流れのテスト。conductor-core の判断(`decide_before_run`、`classify_and_decide`)と、
//! この Unit の `run_job`(補助の実行ファイル)・`apply_*`・`write_state`・`read_state`・
//! `reset_state` を組み合わせて、Cli の流れを、小さな試験用の関数で再現する(契約 C2 の順序の続き)。

mod common;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use common::*;
use conductor_core::{
    Decision, JobDefinition, JobStatus, PipelineDefinition, RunMode, Transition, TransitionOn,
    TransitionTarget, classify_and_decide, decide_before_run,
};
use conductor_runtime::{
    ConsistencyResult, Durability, InterruptSignal, MismatchKind, StateError, StateErrorKind,
    apply_outcome, apply_skip, check_consistency, initial_state as initial_state_with,
    read_state as read_state_with, reset_state as reset_state_with, run_job,
    write_state as write_state_with,
};

// run ID なしの呼び出し(この統合テストは、run ID を使わない流れ)。
fn read_state(def: &std::path::Path) -> Result<Option<conductor_core::PipelineState>, StateError> {
    read_state_with(def, None)
}
fn reset_state(def: &std::path::Path) -> Result<(), StateError> {
    reset_state_with(def, None)
}
fn initial_state(d: &PipelineDefinition, def: &std::path::Path) -> conductor_core::PipelineState {
    initial_state_with(d, def, None)
}
fn write_state(
    def: &std::path::Path,
    s: &conductor_core::PipelineState,
    dur: Durability,
) -> Result<(), StateError> {
    write_state_with(def, None, s, dur)
}

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

#[derive(Debug)]
struct Done {
    /// スケジューラへ返す終了コード。
    exit: i32,
    next_job: Option<String>,
    completed: bool,
    /// ジョブを起動したか。
    ran: bool,
}

/// Cli の 1 回の呼び出し(1 つのジョブ)を再現する。
/// `stop_before_record` が `true` なら、ジョブの終了のあと、記録の前に止まる(強制終了の再現)。
fn invoke_inner(
    def_path: &Path,
    definition: &PipelineDefinition,
    job_id: &str,
    mode: RunMode,
    signal: &InterruptSignal,
    stop_before_record: bool,
) -> Result<Done, Failure> {
    let loaded = read_state(def_path)?;
    let is_empty = loaded.is_none();
    if let Some(state) = &loaded
        && let ConsistencyResult::Mismatch(kind) = check_consistency(state, definition)
    {
        return Err(Failure::Mismatch(kind));
    }
    let state = loaded.unwrap_or_else(|| initial_state(definition, def_path));

    let decision = decide_before_run(
        job_id,
        definition,
        &state.job_states,
        state.resume_job_id.as_deref(),
        state.end_reached,
    );
    match decision {
        Decision::Reject(_) => Err(Failure::Rejected),
        Decision::Skip(reason) => {
            let next = apply_skip(&state, job_id, reason, definition);
            write_state(def_path, &next, Durability::Durable)?;
            let completed = next.completed;
            if completed {
                reset_state(def_path)?;
            }
            Ok(Done {
                exit: 0,
                next_job: None,
                completed,
                ran: false,
            })
        }
        Decision::Run => {
            if is_empty {
                write_state(def_path, &state, Durability::Fast)?;
            }
            let job = definition
                .job
                .iter()
                .find(|j| j.id == job_id)
                .expect("定義にある");
            let execution = run_job(job, signal);
            if stop_before_record {
                return Ok(Done {
                    exit: -1,
                    next_job: None,
                    completed: false,
                    ran: true,
                });
            }
            let (outcome, next) =
                classify_and_decide(job, &execution, &state.job_states, definition, mode);
            let recorded = apply_outcome(&state, job_id, &execution, &outcome, &next);
            write_state(def_path, &recorded, Durability::Durable)?;
            if next.completed {
                reset_state(def_path)?;
            }
            Ok(Done {
                exit: outcome.reported_exit_code,
                next_job: next.next_job_id,
                completed: next.completed,
                ran: execution.started,
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
    let def = scene.def_path();
    let definition = definition.clone();
    let job_id = job_id.to_string();
    run_with_limit(
        || {},
        move || {
            invoke_inner(
                &def,
                &definition,
                &job_id,
                mode,
                &InterruptSignal::new(),
                false,
            )
        },
    )
}

/// 一連の実行(`pipeline`): ステートの再開位置(なければ先頭)から、最後か停止まで。
fn pipeline(scene: &Scene, definition: &PipelineDefinition) -> Result<Vec<Done>, Failure> {
    let start = read_state(&scene.def_path())?
        .and_then(|s| s.resume_job_id)
        .unwrap_or_else(|| definition.job[0].id.clone());
    let mut done = Vec::new();
    let mut current = Some(start);
    while let Some(id) = current {
        let d = invoke(scene, definition, &id, RunMode::Pipeline)?;
        current = d.next_job.clone();
        done.push(d);
    }
    Ok(done)
}

#[test]
fn ac4_2_1_個別実行で最後まで行くとステートが初期化される() {
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
    assert!(!scene.state_path().exists(), "完了でステートが初期化される");
    assert_eq!(scene.ran(), ["a", "b", "c"]);
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
    assert!(!scene.state_path().exists());
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
    assert!(!scene.state_path().exists());
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
    assert!(!scene.state_path().exists());
}

#[test]
fn ac4_2_4_一連の実行の_end_で完了しステートが初期化される() {
    let scene = Scene::new();
    let def = scene.definition(vec![
        on_success_go(scene.job("a", 0), TransitionTarget::End),
        scene.job("b", 0),
        scene.job("c", 0),
    ]);
    let done = pipeline(&scene, &def).expect("実行できる");
    assert_eq!(done.len(), 1);
    assert!(done[0].completed);
    assert!(!scene.state_path().exists());
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
    assert!(!scene.state_path().exists());
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
