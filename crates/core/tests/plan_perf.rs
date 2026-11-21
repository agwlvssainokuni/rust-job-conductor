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

//! 1,000 ジョブでの計画の生成の性能(U3。BR6.3、NFR1)。1 秒以内。

use std::time::{Duration, Instant};

use conductor_core::{
    EnvEntry, EnvFileContent, LoadedEnvFiles, PlanDisposition, PlanEnd, PlanInput,
    parse_definition, plan_pipeline,
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
    files.add(
        "p.env",
        EnvFileContent {
            path: "p.env".into(),
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
            .all(|e| e.disposition == PlanDisposition::WillRun)
    );
    assert_eq!(plan.end, PlanEnd::Completed);
    assert!(elapsed < Duration::from_secs(1), "{elapsed:?}");
}
