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

//! 計画の型(U3。BR2.1、BR5.1、FR1.12)。

mod common;

use common::{SECRET, add_env_file};
use conductor_core::{
    Assumption, Classification, EnvEntry, EnvFileContent, LoadedEnvFiles, PlanDisposition,
    PlanEntry, PlanInput, SimulatedResult, parse_definition, plan_pipeline,
};

#[test]
fn 計画の項目は_扱いと結果と名前の要約を持つ() {
    let entry = PlanEntry {
        job_id: "a".into(),
        disposition: PlanDisposition::WillRun(SimulatedResult {
            exit_code: 3,
            classification: Classification::Warning,
            assumed: true,
            retryable: false,
        }),
        is_resume: true,
        command_words: vec!["echo".into(), "a".into()],
        env_names: Vec::new(),
        unset_names: Vec::new(),
        retry: None,
    };
    assert_eq!(entry.clone(), entry);
    assert_eq!(entry.disposition.result().map(|r| r.exit_code), Some(3));
}

#[test]
fn 仮定の終了コードの範囲は_0_から_255() {
    assert!(Assumption::exit_code_in_range(0));
    assert!(Assumption::exit_code_in_range(255));
    assert!(!Assumption::exit_code_in_range(-1));
    assert!(!Assumption::exit_code_in_range(256));
    assert_eq!(Assumption::new("a", 7).exit_code, 7);
}

#[test]
fn 計画は環境変数の値を持たない_名前だけ() {
    let def = parse_definition(
        "format_version = 1\n[pipeline]\nname = \"p\"\nenv_file = \"p.env\"\n[[job]]\nid = \"a\"\ncommand = \"echo a\"\n[job.env]\nTOKEN = \"s3cr3t-VALUE-do-not-print\"\n",
        "t.toml",
        None,
    )
    .expect("正しい定義");
    let mut files = LoadedEnvFiles::new();
    add_env_file(
        &mut files,
        "p.env",
        EnvFileContent {
            file: "p.env".into(),
            entries: vec![EnvEntry {
                name: "FROM_FILE".into(),
                value: SECRET.into(),
                line: 1,
            }],
            warnings: Vec::new(),
        },
    );
    let commands = vec![vec!["echo".to_string(), "a".to_string()]];
    let input = PlanInput {
        definition: &def,
        state: None,
        commands: &commands,
        env_files: &files,
        run_id: None,
        assumptions: &[],
    };
    let plan = plan_pipeline(&input).expect("計画");
    let debug = format!("{plan:?}");
    assert!(!debug.contains(SECRET), "{debug}");
    let names: Vec<&str> = plan.entries[0]
        .env_names
        .iter()
        .map(|e| e.name.as_str())
        .collect();
    assert_eq!(
        names,
        [
            "FROM_FILE",
            "TOKEN",
            "CONDUCTOR_PIPELINE",
            "CONDUCTOR_JOB_ID",
            "CONDUCTOR_ATTEMPT",
        ]
    );
}
