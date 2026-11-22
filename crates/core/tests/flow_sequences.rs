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

//! Flow の一連の流れの統合テスト。公開の関数だけを使い、Cli のステートの更新を模す。
//!
//! Flow は、引数だけで試せる(ファイルにもプロセスにも触れない)。ここでは、
//! `parse_definition` で作った定義と、手元の `Vec<JobState>` だけを使う。

mod common;

use common::exited;
use conductor_core::{
    Classification, Decision, ExecutionResult, JobExecution, JobState, JobStatus, NextStep,
    PipelineDefinition, RejectReason, ResumePoint, RunMode, SkipCause, SkipReason,
    classify_and_decide, decide_before_run, parse_definition, resume_point_on_interrupt,
};

fn definition(extra_transitions: &str) -> PipelineDefinition {
    let text = format!(
        "format_version = 1\n[pipeline]\nname = \"p\"\n\
         [[job]]\nid = \"a\"\ncommand = \"c\"\nwarning_exit_codes = [3]\n{extra_transitions}\n\
         [[job]]\nid = \"b\"\ncommand = \"c\"\n\
         [[job]]\nid = \"c\"\ncommand = \"c\"\n\
         [[job]]\nid = \"d\"\ncommand = \"c\"\n"
    );
    parse_definition(&text, "t.toml", None).expect("正しい定義")
}

/// Cli が持つステートの、最小の模型。
struct Sim {
    def: PipelineDefinition,
    states: Vec<JobState>,
    resume: Option<String>,
    end_reached: bool,
    completed: bool,
}

impl Sim {
    fn new(def: PipelineDefinition) -> Self {
        let states = def.job.iter().map(|j| JobState::pending(&j.id)).collect();
        Self {
            def,
            states,
            resume: None,
            end_reached: false,
            completed: false,
        }
    }

    fn set(&mut self, id: &str, status: JobStatus, code: Option<i32>, reason: Option<SkipReason>) {
        let s = self.states.iter_mut().find(|s| s.job_id == id).unwrap();
        s.status = status;
        s.exit_code = code;
        s.skip_reason = reason;
    }

    fn decide(&self, id: &str) -> Decision {
        decide_before_run(
            id,
            &self.def,
            &self.states,
            self.resume.as_deref(),
            self.end_reached,
        )
    }

    /// ジョブを呼ぶ。run なら、`exec` の結果で分類して、ステートに反映する。
    /// 戻り値は、(判断, 返す終了コード)。
    fn call(&mut self, id: &str, exec: JobExecution, mode: RunMode) -> (Decision, Option<i32>) {
        let decision = self.decide(id);
        match decision {
            Decision::Reject(_) => (decision, None),
            Decision::Skip(_) => {
                if self
                    .states
                    .iter()
                    .any(|s| s.job_id == id && s.status == JobStatus::Pending)
                {
                    self.set(id, JobStatus::Skipped, None, Some(SkipReason::ByEnd));
                }
                let rp = resume_point_on_interrupt(&self.def, &self.states);
                self.apply_resume(rp);
                (decision, Some(0))
            }
            Decision::Run => {
                let job = self.def.job[self.def.job_index(id).unwrap()].clone();
                let (outcome, step) =
                    classify_and_decide(&job, &exec, &self.states, &self.def, mode);
                let status = match outcome.classification {
                    Some(Classification::Success) => JobStatus::Success,
                    Some(Classification::Warning) => JobStatus::Warning,
                    Some(Classification::Error) => JobStatus::Error,
                    None => JobStatus::Pending,
                };
                let exit_code = match exec.result {
                    ExecutionResult::Exited { exit_code, .. } => exit_code,
                    _ => None,
                };
                self.set(id, status, exit_code, None);
                self.apply_step(&step);
                (decision, Some(outcome.reported_exit_code))
            }
        }
    }

    fn apply_step(&mut self, step: &NextStep) {
        for s in &step.skipped_job_ids {
            self.set(s, JobStatus::Skipped, None, step.skip_reason_kind);
        }
        self.end_reached |= step.end_reached;
        self.completed = step.completed;
        self.resume = step.resume_job_id.clone();
    }

    fn apply_resume(&mut self, rp: ResumePoint) {
        self.resume = rp.resume_job_id;
        self.end_reached |= rp.end_reached;
        self.completed = rp.completed;
    }
}

#[test]
fn 一連の実行_警告の遷移で間をスキップして完了する() {
    let def = definition("[[job.transitions]]\non = \"warning\"\nnext = \"d\"\n");
    let mut sim = Sim::new(def);
    // a が警告(3)→ d へ。b、c はスキップ。
    let (d, code) = sim.call("a", exited("a", 3), RunMode::Pipeline);
    assert_eq!((d, code), (Decision::Run, Some(0)));
    assert_eq!(sim.states[1].skip_reason, Some(SkipReason::ByTransition));
    assert_eq!(sim.states[2].status, JobStatus::Skipped);
    assert!(!sim.completed);
    // d を実行して完了
    let (d, code) = sim.call("d", exited("d", 0), RunMode::Pipeline);
    assert_eq!((d, code), (Decision::Run, Some(0)));
    assert!(sim.completed && sim.resume.is_none());
}

#[test]
fn 個別実行_エラーから再実行して完了までの流れ() {
    let mut sim = Sim::new(definition(""));
    sim.call("a", exited("a", 0), RunMode::Individual);
    // b がエラー(5)。そのまま返り、停止し、再開位置は b。
    let (_, code) = sim.call("b", exited("b", 5), RunMode::Individual);
    assert_eq!(code, Some(5));
    assert_eq!(sim.resume.as_deref(), Some("b"));
    assert_eq!(sim.states[1].status, JobStatus::Error);
    // c を先に呼ぶと順序外で拒否され、ステートは変わらない。
    let before = sim.states.clone();
    let (d, code) = sim.call("c", exited("c", 0), RunMode::Individual);
    assert_eq!(
        (d, code),
        (Decision::Reject(RejectReason::OutOfOrder), None)
    );
    assert_eq!(sim.states, before);
    // b を再実行して成功。再開位置は解除される。
    let (d, _) = sim.call("b", exited("b", 0), RunMode::Individual);
    assert_eq!(d, Decision::Run);
    assert!(sim.resume.is_none());
    // a を再び呼ぶと、実行済みとしてスキップ(0)。
    let (d, code) = sim.call("a", exited("a", 0), RunMode::Individual);
    assert_eq!(
        (d, code),
        (Decision::Skip(SkipCause::AlreadyExecuted), Some(0))
    );
    // c、d を順に実行して完了。
    sim.call("c", exited("c", 0), RunMode::Individual);
    assert!(!sim.completed);
    sim.call("d", exited("d", 0), RunMode::Individual);
    assert!(sim.completed);
}

#[test]
fn 個別実行_end_のあとはスケジューラの呼び出しでスキップされ最後で完了する() {
    let def = definition("[[job.transitions]]\non = \"success\"\nnext = \"@end\"\n");
    let mut sim = Sim::new(def);
    sim.call("a", exited("a", 0), RunMode::Individual);
    assert!(
        sim.end_reached && !sim.completed,
        "個別実行では、@end に着いても、まだ完了しない"
    );
    assert!(
        sim.states[1..]
            .iter()
            .all(|s| s.status == JobStatus::Pending),
        "個別実行では、@end の時点で後ろのジョブを記録しない"
    );
    for id in ["b", "c"] {
        let (d, code) = sim.call(id, exited(id, 0), RunMode::Individual);
        assert_eq!(
            (d, code),
            (Decision::Skip(SkipCause::AfterEnd), Some(0)),
            "{id}"
        );
        assert!(!sim.completed, "{id} の記録のあとは、まだ完了しない");
        assert!(sim.end_reached);
    }
    assert_eq!(sim.resume.as_deref(), Some("d"), "次の未呼び出しのジョブ");
    let (d, code) = sim.call("d", exited("d", 0), RunMode::Individual);
    assert_eq!((d, code), (Decision::Skip(SkipCause::AfterEnd), Some(0)));
    assert!(sim.completed, "最後のジョブがスキップされた時点で完了");
    assert!(sim.resume.is_none());
}

#[test]
fn 一連の実行_end_で以降をまとめてスキップして完了する() {
    let def = definition("[[job.transitions]]\non = \"success\"\nnext = \"@end\"\n");
    let mut sim = Sim::new(def);
    sim.call("a", exited("a", 0), RunMode::Pipeline);
    assert!(sim.end_reached && sim.completed);
    for s in &sim.states[1..] {
        assert_eq!(s.status, JobStatus::Skipped, "{}", s.job_id);
        assert_eq!(s.skip_reason, Some(SkipReason::ByEnd));
    }
    assert!(sim.resume.is_none());
}

#[test]
fn 一連の実行_end_で即座に完了する() {
    let def = definition("[[job.transitions]]\non = \"success\"\nnext = \"@end\"\n");
    let mut sim = Sim::new(def);
    sim.call("a", exited("a", 0), RunMode::Pipeline);
    assert!(sim.end_reached && sim.completed);
}

#[test]
fn 成功コードを書いたジョブの_0_はエラーで_conductor_のエラー() {
    let text = "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"c\"\nsuccess_exit_codes = [2]\n";
    let def = parse_definition(text, "t.toml", None).unwrap();
    let job = &def.job[0];
    let (o, step) = classify_and_decide(job, &exited("a", 0), &[], &def, RunMode::Individual);
    assert_eq!(o.classification, Some(Classification::Error));
    assert_eq!((o.reported_exit_code, o.conductor_error), (1, true));
    assert_eq!(step.resume_job_id.as_deref(), Some("a"));
    let (o, step) = classify_and_decide(job, &exited("a", 2), &[], &def, RunMode::Individual);
    assert_eq!(o.classification, Some(Classification::Success));
    assert!(step.completed);
}

#[test]
fn 優先順位_中断と起動失敗とシグナル() {
    let def = definition("");
    let job = &def.job[0];
    // 起動失敗と中断・シグナルは、型の上で同時に作れない(起動失敗の優先は型で決まる)。
    let mut e = exited("a", 0);
    e.result = ExecutionResult::Exited {
        exit_code: Some(0),
        terminated_by_signal: Some(9),
        interrupted_by: Some(2),
    };
    let (o, _) = classify_and_decide(job, &e, &[], &def, RunMode::Pipeline);
    assert_eq!(o.reported_exit_code, 130, "中断が最優先");
    e.result = ExecutionResult::StartFailed;
    let (o, _) = classify_and_decide(job, &e, &[], &def, RunMode::Pipeline);
    assert_eq!(o.reported_exit_code, 127, "起動失敗がシグナル終了より先");
    e.result = ExecutionResult::Exited {
        exit_code: Some(0),
        terminated_by_signal: Some(9),
        interrupted_by: None,
    };
    let (o, _) = classify_and_decide(job, &e, &[], &def, RunMode::Pipeline);
    assert_eq!(o.reported_exit_code, 137);
}

#[test]
fn ジョブのあいだの中断の再開位置() {
    let mut sim = Sim::new(definition(""));
    sim.call("a", exited("a", 0), RunMode::Individual);
    let rp = resume_point_on_interrupt(&sim.def, &sim.states);
    assert_eq!(rp.resume_job_id.as_deref(), Some("b"));
    assert!(!rp.completed && !rp.end_reached);
    // 再開位置のジョブは、次の呼び出しで run になる。
    sim.apply_resume(rp);
    assert_eq!(sim.decide("b"), Decision::Run);
}

#[test]
fn 起動しなかった中断は再開位置になり_128_足す番号を返す() {
    let def = definition("");
    let job = &def.job[0];
    let e = JobExecution::not_started(job, 15);
    let (o, step) = classify_and_decide(job, &e, &[], &def, RunMode::Pipeline);
    assert_eq!(o.classification, None);
    assert_eq!(o.reported_exit_code, 143);
    assert_eq!(step.resume_job_id.as_deref(), Some("a"));
    assert!(step.next_job_id.is_none() && !step.completed);
}
