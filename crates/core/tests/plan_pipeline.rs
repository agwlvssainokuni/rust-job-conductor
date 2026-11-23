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

//! `pipeline --dry-run` の計画の生成(U3。BR1.2〜BR1.4、BR1.8)。

mod common;

use common::state;
use conductor_core::{
    Assumption, Classification, ExecutionPlan, JobStatus, LoadedEnvFiles, PipelineState,
    PlanDisposition as D, PlanEnd, PlanInput, ResumeInfo, SkipReason, parse_definition,
    plan_pipeline,
};

const THREE: &str = "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"echo a\"\nwarning_exit_codes = [3]\n[[job]]\nid = \"b\"\ncommand = \"echo b\"\n[[job]]\nid = \"c\"\ncommand = \"echo c\"\n";

/// a の警告(3)で c へ飛ぶ。
const JUMP: &str = "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"echo a\"\nwarning_exit_codes = [3]\n[[job.transitions]]\non = \"warning\"\nnext = \"c\"\n[[job]]\nid = \"b\"\ncommand = \"echo b\"\n[[job]]\nid = \"c\"\ncommand = \"echo c\"\n";

/// a の警告(3)で `@end`。
const END: &str = "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"echo a\"\nwarning_exit_codes = [3]\n[[job.transitions]]\non = \"warning\"\nnext = \"@end\"\n[[job]]\nid = \"b\"\ncommand = \"echo b\"\n[[job]]\nid = \"c\"\ncommand = \"echo c\"\n";

fn plan_with(
    text: &str,
    state: Option<&PipelineState>,
    assumptions: &[Assumption],
) -> ExecutionPlan {
    let def = parse_definition(text, "t.toml", None).expect("正しい定義");
    let commands: Vec<Vec<String>> = def
        .job
        .iter()
        .map(|j| vec!["echo".to_string(), j.id.clone()])
        .collect();
    let files = LoadedEnvFiles::new();
    let input = PlanInput {
        definition: &def,
        state,
        commands: &commands,
        env_files: &files,
        run_id: None,
        assumptions,
    };
    plan_pipeline(&input).expect("計画を作れる")
}

fn dispositions(plan: &ExecutionPlan) -> Vec<D> {
    plan.entries.iter().map(|e| e.disposition).collect()
}

#[test]
fn すべて成功する流れは_全ジョブを実行して完了する() {
    let plan = plan_with(THREE, None, &[]);
    assert_eq!(dispositions(&plan), [D::WillRun, D::WillRun, D::WillRun]);
    assert_eq!(plan.end, PlanEnd::Completed);
    assert_eq!(plan.resume, ResumeInfo::FromStart);
    assert!(plan.unused_assumptions.is_empty());
    for entry in &plan.entries {
        let result = entry.result.expect("実行するジョブは結果を持つ");
        assert_eq!(result.classification, Classification::Success);
        assert!(!result.assumed);
    }
}

#[test]
fn 仮定で警告になっても_次のジョブへ進み_完了する() {
    let plan = plan_with(THREE, None, &[Assumption::new("a", 3)]);
    let a = plan.entries[0].result.expect("結果");
    assert_eq!(
        (a.exit_code, a.classification, a.assumed),
        (3, Classification::Warning, true)
    );
    assert_eq!(dispositions(&plan), [D::WillRun, D::WillRun, D::WillRun]);
    assert_eq!(plan.end, PlanEnd::Completed);
}

#[test]
fn 仮定でエラーになると停止し_以降は実行されない() {
    let plan = plan_with(THREE, None, &[Assumption::new("b", 1)]);
    assert_eq!(dispositions(&plan), [D::WillRun, D::WillRun, D::NotReached]);
    assert_eq!(plan.end, PlanEnd::StoppedAtError("b".into()));
    assert!(plan.entries[2].result.is_none());
}

#[test]
fn 遷移で飛び越されたジョブは_遷移でスキップになる() {
    let plan = plan_with(JUMP, None, &[Assumption::new("a", 3)]);
    assert_eq!(
        dispositions(&plan),
        [D::WillRun, D::SkippedByTransition, D::WillRun]
    );
    assert_eq!(plan.end, PlanEnd::Completed);
}

#[test]
fn end_に到達すると_以降は_end_のあとでスキップになる() {
    let plan = plan_with(END, None, &[Assumption::new("a", 3)]);
    assert_eq!(
        dispositions(&plan),
        [D::WillRun, D::SkippedAfterEnd, D::SkippedAfterEnd]
    );
    assert_eq!(plan.end, PlanEnd::EndReached);
}

#[test]
fn 前回の停止から再開する_実行済みと再開位置() {
    let s = state(
        &[
            ("a", JobStatus::Success),
            ("b", JobStatus::Error),
            ("c", JobStatus::Pending),
        ],
        Some("b"),
        false,
    );
    let plan = plan_with(THREE, Some(&s), &[]);
    assert_eq!(
        dispositions(&plan),
        [D::AlreadyExecuted, D::WillRun, D::WillRun]
    );
    assert_eq!(plan.resume, ResumeInfo::AfterStop("b".into()));
    let resumes: Vec<bool> = plan.entries.iter().map(|e| e.is_resume).collect();
    assert_eq!(resumes, [false, true, false]);
}

#[test]
fn ステートが無ければ最初から_再開位置のジョブは無い() {
    let plan = plan_with(THREE, None, &[]);
    assert_eq!(plan.resume, ResumeInfo::FromStart);
    assert!(plan.entries.iter().all(|e| !e.is_resume));
}

#[test]
fn 完了したステートは_ステートなしと同じ扱いで最初から() {
    let s = state(
        &[
            ("a", JobStatus::Success),
            ("b", JobStatus::Success),
            ("c", JobStatus::Success),
        ],
        None,
        true,
    );
    let plan = plan_with(THREE, Some(&s), &[]);
    assert_eq!(dispositions(&plan), [D::WillRun, D::WillRun, D::WillRun]);
    assert_eq!(plan.resume, ResumeInfo::FromStart);
}

#[test]
fn ステートの遷移のスキップの記録は_遷移でスキップとして引き継がれる() {
    let mut s = state(
        &[
            ("a", JobStatus::Success),
            ("b", JobStatus::Skipped),
            ("c", JobStatus::Pending),
        ],
        Some("c"),
        false,
    );
    s.job_states[1].skip_reason = Some(SkipReason::ByTransition);
    let plan = plan_with(JUMP, Some(&s), &[]);
    assert_eq!(
        dispositions(&plan),
        [D::AlreadyExecuted, D::SkippedByTransition, D::WillRun]
    );
}

#[test]
fn 手で書き換えた_end_のスキップがあり_end_reached_が偽のステートは_終点に着くで終わる() {
    // 本番の流れでは作られない形(`check_consistency` は通る)。模擬は、本番と同じ `apply_skip` で
    // 終点の後のスキップを記録し、`end_reached` を立てるので、終わり方は「終点に着く」(本番と一致)。
    let mut s = state(
        &[
            ("a", JobStatus::Success),
            ("b", JobStatus::Skipped),
            ("c", JobStatus::Pending),
        ],
        None,
        false,
    );
    s.job_states[0].exit_code = Some(0);
    s.job_states[1].skip_reason = Some(SkipReason::ByEnd);
    assert!(!s.end_reached);
    let plan = plan_with(THREE, Some(&s), &[]);
    assert_eq!(
        dispositions(&plan),
        [D::AlreadyExecuted, D::SkippedAfterEnd, D::SkippedAfterEnd]
    );
    assert_eq!(plan.end, PlanEnd::EndReached);
}
