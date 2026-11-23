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

//! `run <job> --dry-run` の計画の生成(U3。BR1.6、BR4.3)。

mod common;

use common::state;
use conductor_core::{
    Assumption, ExecutionPlan, JobStatus, LoadedEnvFiles, NextStepNote, PipelineState,
    PlanDisposition as D, PlanInput, PlanMode, PlanRejection, RejectReason, ResumeInfo,
    parse_definition, plan_run,
};

const THREE: &str = "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"echo a\"\nwarning_exit_codes = [3]\n[[job]]\nid = \"b\"\ncommand = \"echo b\"\n[[job]]\nid = \"c\"\ncommand = \"echo c\"\n";

/// a の成功で c へ飛ぶ(b は飛び越す)。
const JUMP: &str = "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"echo a\"\n[[job.transitions]]\non = \"success\"\nnext = \"c\"\n[[job]]\nid = \"b\"\ncommand = \"echo b\"\n[[job]]\nid = \"c\"\ncommand = \"echo c\"\n";

/// a の成功で `@end`。
const END: &str = "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"echo a\"\n[[job.transitions]]\non = \"success\"\nnext = \"@end\"\n[[job]]\nid = \"b\"\ncommand = \"echo b\"\n[[job]]\nid = \"c\"\ncommand = \"echo c\"\n";

fn run_plan(
    text: &str,
    job_id: &str,
    state: Option<&PipelineState>,
    assumptions: &[Assumption],
) -> Result<ExecutionPlan, PlanRejection> {
    let def = parse_definition(text, "t.toml", None).expect("正しい定義");
    let commands: Vec<Vec<String>> = def.job.iter().map(|j| vec![j.id.clone()]).collect();
    let files = LoadedEnvFiles::new();
    plan_run(
        job_id,
        &PlanInput {
            definition: &def,
            state,
            commands: &commands,
            env_files: &files,
            run_id: None,
            assumptions,
        },
    )
}

#[test]
fn run_の計画は_対象の_1_ジョブと次の動きを持ち_再開位置は示さない() {
    let s = state(
        &[
            ("a", JobStatus::Success),
            ("b", JobStatus::Pending),
            ("c", JobStatus::Pending),
        ],
        None,
        false,
    );
    let plan = run_plan(THREE, "b", Some(&s), &[]).expect("計画");
    assert_eq!(plan.mode, PlanMode::Individual);
    assert_eq!(plan.entries.len(), 1);
    assert_eq!(plan.entries[0].job_id, "b");
    assert!(
        matches!(plan.entries[0].disposition, D::WillRun(_)),
        "{:?}",
        plan.entries[0].disposition
    );
    assert!(!plan.entries[0].is_resume);
    assert_eq!(plan.resume, ResumeInfo::Individual("b".into()));
    let note = plan.next_step_note.expect("次の動き");
    assert!(
        matches!(&note, NextStepNote::NextJob { job_id, .. } if job_id == "c"),
        "{note:?}"
    );
}

#[test]
fn run_の次の動きは_遷移のスキップ_end_完了_停止を区別する() {
    // 遷移でスキップ。
    let note = run_plan(JUMP, "a", None, &[])
        .expect("計画")
        .next_step_note
        .expect("次の動き");
    assert_eq!(
        note,
        NextStepNote::NextJob {
            job_id: "c".into(),
            skipped_job_ids: vec!["b".into()],
        }
    );
    // `@end` に到達(個別実行では、後ろは列挙しない)。
    let note = run_plan(END, "a", None, &[])
        .expect("計画")
        .next_step_note
        .expect("次の動き");
    assert_eq!(note, NextStepNote::EndReached);
    // 最後のジョブの成功は、完了。
    let s = state(
        &[
            ("a", JobStatus::Success),
            ("b", JobStatus::Success),
            ("c", JobStatus::Pending),
        ],
        None,
        false,
    );
    let note = run_plan(THREE, "c", Some(&s), &[])
        .expect("計画")
        .next_step_note
        .expect("次の動き");
    assert_eq!(note, NextStepNote::Completed);
    // 仮定でエラーなら、停止。
    let s = state(
        &[("a", JobStatus::Success), ("b", JobStatus::Pending)],
        None,
        false,
    );
    let note = run_plan(THREE, "b", Some(&s), &[Assumption::new("b", 1)])
        .expect("計画")
        .next_step_note
        .expect("次の動き");
    assert_eq!(note, NextStepNote::Stopped);
}

#[test]
fn run_で実行済みのジョブは_実行済みで_次の動きを持たない() {
    let s = state(
        &[("a", JobStatus::Success), ("b", JobStatus::Pending)],
        None,
        false,
    );
    let plan = run_plan(THREE, "a", Some(&s), &[]).expect("計画");
    assert_eq!(plan.entries[0].disposition, D::AlreadyExecuted);
    assert!(plan.entries[0].disposition.result().is_none());
    assert!(plan.next_step_note.is_none());
}

#[test]
fn run_で順序外のジョブは_拒否になる() {
    // a が未実行のまま b を呼ぶ。
    let rejection = run_plan(THREE, "b", None, &[]).expect_err("拒否");
    assert_eq!(
        rejection,
        PlanRejection {
            job_id: "b".into(),
            reason: RejectReason::OutOfOrder
        }
    );
    // 定義にないジョブも、パニックせず拒否。
    assert!(run_plan(THREE, "zzz", None, &[]).is_err());
}

#[test]
fn run_の対象でないジョブへの仮定は_使われなかった仮定になる() {
    let plan = run_plan(
        THREE,
        "a",
        None,
        &[Assumption::new("a", 3), Assumption::new("c", 1)],
    )
    .expect("計画");
    assert_eq!(plan.unused_assumptions, ["c"]);
    let r = plan.entries[0].disposition.result().expect("結果");
    assert_eq!(r.exit_code, 3);
    assert!(r.assumed);
}
