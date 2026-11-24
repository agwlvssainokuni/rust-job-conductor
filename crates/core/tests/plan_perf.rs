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

//! 1,000 ジョブでの計画の生成の性能(U3。BR6.3、NFR1。plan-state-unify の BR2.3)。1 秒以内。

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use common::add_env_file;
use conductor_core::{
    EnvEntry, EnvFileContent, JobExecution, LoadedEnvFiles, PlanDisposition, PlanEnd, PlanInput,
    RunMode, apply_outcome, classify_and_decide, initial_state, parse_definition, plan_pipeline,
};

const N: usize = 1000;

#[test]
fn 千個のジョブがすべて実行する計画は_1秒以内に作れる() {
    // すべてのジョブが `env` と `env_file` を持つ。
    let mut text =
        String::from("format_version = 1\n[pipeline]\nname = \"p\"\nenv_file = \"p.env\"\n");
    for i in 0..N {
        text.push_str(&format!(
            "[[job]]\nid = \"j{i}\"\ncommand = \"echo {i}\"\nenv_file = \"p.env\"\n[job.env]\nK{i} = \"v\"\n"
        ));
    }
    let def = parse_definition(&text, "t.toml", None).expect("正しい定義");
    let commands: Vec<Vec<String>> = def
        .job
        .iter()
        .map(|j| vec!["echo".to_string(), j.id.clone()])
        .collect();
    let mut files = LoadedEnvFiles::new();
    add_env_file(
        &mut files,
        "p.env",
        EnvFileContent {
            entries: (0..20)
                .map(|i| EnvEntry {
                    name: format!("F{i}"),
                    value: "x".into(),
                })
                .collect(),
            warnings: Vec::new(),
        },
    );
    let start = Instant::now();
    let plan = plan_pipeline(&PlanInput {
        definition: &def,
        state: None,
        commands: &commands,
        env_files: &files,
        run_id: Some("r1"),
        assumptions: &[],
    })
    .expect("計画");
    let elapsed = start.elapsed();
    assert_eq!(plan.entries.len(), N);
    assert!(
        plan.entries
            .iter()
            .all(|e| matches!(e.disposition, PlanDisposition::WillRun(_)))
    );
    assert_eq!(plan.end, PlanEnd::Completed);
    assert!(elapsed < Duration::from_secs(1), "{elapsed:?}");
}

#[test]
fn 終点の後に千個近いジョブをスキップする計画は_1秒以内に作れる() {
    // 先頭のジョブの成功で `@end`。`run j0` の後のステート(j0 が成功、`end_reached`、残りは pending)
    // から計画すると、残りの 999 個は、それぞれ `apply_skip`(終点の後)で記録される。
    let mut text = String::from(
        "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"j0\"\ncommand = \"echo 0\"\n[[job.transitions]]\non = \"success\"\nnext = \"@end\"\n",
    );
    for i in 1..N {
        text.push_str(&format!("[[job]]\nid = \"j{i}\"\ncommand = \"echo {i}\"\n"));
    }
    let def = parse_definition(&text, "t.toml", None).expect("正しい定義");
    let s0 = initial_state(&def, Path::new("t.toml"), None);
    let execution = JobExecution::exited(0);
    let (outcome, next) = classify_and_decide(
        &def.job[0],
        &execution,
        &s0.job_states,
        &def,
        RunMode::Individual,
    );
    let state = apply_outcome(&s0, "j0", &execution, &outcome, &next);
    assert!(state.end_reached && !state.completed);
    let commands: Vec<Vec<String>> = def
        .job
        .iter()
        .map(|j| vec!["echo".to_string(), j.id.clone()])
        .collect();
    let files = LoadedEnvFiles::new();

    let start = Instant::now();
    let plan = plan_pipeline(&PlanInput {
        definition: &def,
        state: Some(&state),
        commands: &commands,
        env_files: &files,
        run_id: None,
        assumptions: &[],
    })
    .expect("計画");
    let elapsed = start.elapsed();
    eprintln!("終点の後の {} 個のスキップの計画: {elapsed:?}", N - 1);
    assert_eq!(plan.entries.len(), N);
    assert_eq!(
        plan.entries[0].disposition,
        PlanDisposition::AlreadyExecuted
    );
    assert!(
        plan.entries[1..]
            .iter()
            .all(|e| e.disposition == PlanDisposition::SkippedAfterEnd)
    );
    assert_eq!(plan.end, PlanEnd::EndReached);
    assert!(elapsed < Duration::from_secs(1), "{elapsed:?}");
}
