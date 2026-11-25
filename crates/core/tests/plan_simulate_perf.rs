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

//! 計画の模擬の性能(types-perf-visible の 2a。BR2.1、BR2.6)。
//!
//! ジョブ 1,000 個の計画を 1 回作る時間の、5 回の中央値が上限の中にあることを確かめる。上限は、
//! 直した後の見積もりの約 5 倍の余裕のある値で、大きな退行だけを止める(D6)。直す前の形に戻った
//! ことは、このテストでは検知できない(直す前と後の時間は、コード生成の記録に残す)。
//!
//! あわせて、すべてのジョブが `env` と `env_file` を持つ 1,000 ジョブの計画を、1 回の計測で
//! 1 秒以内に作れることを確かめる(U3。BR6.3、NFR1。plan-state-unify の BR2.3)。

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use common::{add_env_file, median};
use conductor_core::{
    EnvEntry, EnvFileContent, ExecutionPlan, JobExecution, LoadedEnvFiles, PipelineDefinition,
    PipelineState, PlanDisposition, PlanEnd, PlanInput, RunMode, apply_outcome,
    classify_and_decide, initial_state, parse_definition, plan_pipeline,
};

const N: usize = 1000;
const SAMPLES: usize = 5;
/// P1・P2 の上限(debug。直した後の見積もり約 31 ms の約 5 倍。D6)。
const LIMIT: Duration = Duration::from_millis(150);

/// ジョブ `j0`〜`j999` の定義。`end_after_first` なら、`j0` の成功で `@end` に遷移する。
fn definition(end_after_first: bool) -> PipelineDefinition {
    let mut text = String::from("format_version = 1\n[pipeline]\nname = \"p\"\n");
    for i in 0..N {
        text.push_str(&format!("[[job]]\nid = \"j{i}\"\ncommand = \"echo {i}\"\n"));
        if i == 0 && end_after_first {
            text.push_str("[[job.transitions]]\non = \"success\"\nnext = \"@end\"\n");
        }
    }
    parse_definition(&text, "t.toml", None).expect("正しい定義")
}

fn commands(def: &PipelineDefinition) -> Vec<Vec<String>> {
    def.job
        .iter()
        .map(|j| vec!["echo".to_string(), j.id.clone()])
        .collect()
}

/// 計画を `SAMPLES` 回作り、最後の計画と、時間の中央値を返す。
fn measure(
    def: &PipelineDefinition,
    state: Option<&PipelineState>,
    label: &str,
) -> (ExecutionPlan, Duration) {
    let commands = commands(def);
    let files = LoadedEnvFiles::new();
    let input = PlanInput {
        definition: def,
        state,
        commands: &commands,
        env_files: &files,
        run_id: None,
        assumptions: &[],
    };
    let mut last = None;
    let samples: Vec<Duration> = (0..SAMPLES)
        .map(|_| {
            let start = Instant::now();
            let plan = plan_pipeline(&input).expect("計画");
            let elapsed = start.elapsed();
            last = Some(plan);
            elapsed
        })
        .collect();
    let m = median(samples.clone());
    eprintln!("{label}: 中央値 {m:?}(各回 {samples:?})");
    (last.expect("計画を作った"), m)
}

#[test]
fn p1_千個のジョブがすべて実行する計画は_150ms_以内() {
    let def = definition(false);
    let (plan, m) = measure(&def, None, "P1");
    assert_eq!(plan.entries.len(), N);
    assert!(
        plan.entries
            .iter()
            .all(|e| matches!(e.disposition, PlanDisposition::WillRun(_)))
    );
    assert_eq!(plan.end, PlanEnd::Completed);
    // 最後のジョブまで、すべて結果を持つ(先頭と末尾を確かめる)。
    assert_eq!(plan.entries[0].job_id, "j0");
    assert_eq!(plan.entries[N - 1].job_id, format!("j{}", N - 1));
    assert!(m < LIMIT, "中央値 {m:?} が上限 {LIMIT:?} を超えた");
}

#[test]
fn p2_終点のあとの_999_個のスキップの計画は_150ms_以内() {
    // `run j0` で `@end` に着いた後のステート(j0 は成功、残りは未実行、`end_reached`)。
    let def = definition(true);
    let state = initial_state(&def, Path::new("t.toml"), None);
    let execution = JobExecution::exited(0);
    let (outcome, next) = classify_and_decide(
        &def.job[0],
        &execution,
        &state.job_states,
        &def,
        RunMode::Individual,
    );
    let state = apply_outcome(&state, "j0", &execution, &outcome, &next);
    assert!(state.end_reached && !state.completed);

    let (plan, m) = measure(&def, Some(&state), "P2");
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
    assert!(m < LIMIT, "中央値 {m:?} が上限 {LIMIT:?} を超えた");
}

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
            file: "p.env".into(),
            entries: (0..20)
                .map(|i| EnvEntry {
                    name: format!("F{i}"),
                    value: "x".into(),
                    line: i + 1,
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
