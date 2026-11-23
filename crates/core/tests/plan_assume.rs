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

//! 仮定(`--assume`)の扱い(U3。BR1.7、BR2.3、BR2.4)。

use conductor_core::{
    Assumption, Classification, ExecutionPlan, LoadedEnvFiles, PlanDisposition as D, PlanEnd,
    PlanInput, parse_definition, plan_pipeline,
};

/// a の警告(3)で c へ飛ぶ(b は飛び越される)。a にリトライ(75、76)。
const DEF: &str = "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"echo a\"\nwarning_exit_codes = [3]\n[job.retry]\nexit_codes = [75, 76]\nmax = 3\ninterval = \"5s\"\n[[job.transitions]]\non = \"warning\"\nnext = \"c\"\n[[job]]\nid = \"b\"\ncommand = \"echo b\"\n[[job]]\nid = \"c\"\ncommand = \"echo c\"\n";

/// a の成功コードは 10 だけ(0 は成功ではない)。
const NO_ZERO: &str = "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"make\"\nsuccess_exit_codes = [10]\n[[job]]\nid = \"b\"\ncommand = \"echo b\"\n";

fn make_plan(text: &str, assumptions: &[Assumption]) -> ExecutionPlan {
    let def = parse_definition(text, "t.toml", None).expect("正しい定義");
    let commands: Vec<Vec<String>> = def.job.iter().map(|j| vec![j.id.clone()]).collect();
    let files = LoadedEnvFiles::new();
    plan_pipeline(&PlanInput {
        definition: &def,
        state: None,
        commands: &commands,
        env_files: &files,
        run_id: None,
        assumptions,
    })
    .expect("計画")
}

#[test]
fn 実行されないジョブの仮定は_使われなかった仮定になる_指定順() {
    // a が警告で c へ飛ぶので、b は実行されない。定義にないジョブの仮定も、使われない。
    let plan = make_plan(
        DEF,
        &[
            Assumption::new("c", 0),
            Assumption::new("a", 3),
            Assumption::new("b", 1),
            Assumption::new("zzz", 2),
        ],
    );
    assert_eq!(plan.unused_assumptions, ["b", "zzz"]);
    assert_eq!(plan.assumptions.len(), 4);
}

#[test]
fn リトライの対象の終了コードを仮定すると_注記される() {
    let plan = make_plan(DEF, &[Assumption::new("a", 75)]);
    let r = plan.entries[0].disposition.result().expect("結果");
    assert_eq!(r.classification, Classification::Error);
    assert!(r.retryable);
    assert_eq!(plan.end, PlanEnd::StoppedAtError("a".into()));
    // リトライの対象でない終了コード。
    let plan = make_plan(DEF, &[Assumption::new("a", 1)]);
    assert!(
        !plan.entries[0]
            .disposition
            .result()
            .expect("結果")
            .retryable
    );
}

#[test]
fn 成功コードに_0_を含まないジョブは_既定の_0_でエラーになる() {
    let plan = make_plan(NO_ZERO, &[]);
    let r = plan.entries[0].disposition.result().expect("結果");
    assert_eq!(
        (r.exit_code, r.classification, r.assumed),
        (0, Classification::Error, false)
    );
    assert_eq!(plan.entries[1].disposition, D::NotReached);
    assert_eq!(plan.end, PlanEnd::StoppedAtError("a".into()));
    // 成功コードを仮定すれば、進む。
    let plan = make_plan(NO_ZERO, &[Assumption::new("a", 10)]);
    assert_eq!(plan.end, PlanEnd::Completed);
}

#[test]
fn 終了コード_127_以上の仮定は_通常の終了コードとして分類される() {
    for code in [127, 130, 255] {
        let plan = make_plan(DEF, &[Assumption::new("a", code)]);
        let r = plan.entries[0].disposition.result().expect("結果");
        assert_eq!(
            (r.exit_code, r.classification),
            (code, Classification::Error),
            "{code}"
        );
        assert!(!r.retryable);
    }
}

#[test]
fn 複数のジョブへの仮定は_それぞれのジョブに効く() {
    let plan = make_plan(
        NO_ZERO,
        &[Assumption::new("a", 10), Assumption::new("b", 3)],
    );
    let a = plan.entries[0].disposition.result().expect("結果");
    let b = plan.entries[1].disposition.result().expect("結果");
    assert_eq!(
        (a.classification, b.classification),
        (Classification::Success, Classification::Error)
    );
    assert!(plan.unused_assumptions.is_empty());
}
