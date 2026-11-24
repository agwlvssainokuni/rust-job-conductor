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

//! 上限に近い `.env` の層の統合(`merge_env`)の性能(internal-cleanup-2 の A3。BR2.2、BR2.11)。
//!
//! - A3a: `[pipeline].env_file` の 1 ファイルに 10,000 項目(すべて別の名前)。5 回の中央値が 50 ms 以内。
//! - A3b: `[pipeline].env_file` と `[job].env_file` に 10,000 項目ずつ(同じ名前。後ろが勝つ)。
//!   5 回の中央値が 100 ms 以内。
//!
//! 項目の数に比例する時間。二乗の時間に戻ると、手元の機械でも上限を大きく超える。

mod common;

use std::time::{Duration, Instant};

use common::{add_env_file, median};
use conductor_core::envfile::MAX_ENV_ENTRIES;
use conductor_core::{
    EffectiveEnv, EnvContext, EnvEntry, EnvFileContent, LoadedEnvFiles, PipelineDefinition,
    merge_env, parse_definition,
};

const SAMPLES: usize = 5;
const BUILTINS: [&str; 4] = [
    "CONDUCTOR_PIPELINE",
    "CONDUCTOR_JOB_ID",
    "CONDUCTOR_RUN_ID",
    "CONDUCTOR_ATTEMPT",
];

/// 名前の並びから `.env` の内容を作る。
fn content(path: &str, names: &[String]) -> EnvFileContent {
    EnvFileContent {
        entries: names
            .iter()
            .enumerate()
            .map(|(i, n)| EnvEntry {
                name: n.clone(),
                value: format!("{path}-{i}"),
            })
            .collect(),
        warnings: Vec::new(),
    }
}

fn def(job_env_file: bool) -> PipelineDefinition {
    let job_line = if job_env_file {
        "env_file = \"j.env\"\n"
    } else {
        ""
    };
    let text = format!(
        "format_version = 1\n[pipeline]\nname = \"p\"\nenv_file = \"p.env\"\n[[job]]\nid = \"a\"\ncommand = \"c\"\n{job_line}"
    );
    parse_definition(&text, "t.toml", None).expect("正しい定義")
}

fn ctx() -> EnvContext<'static> {
    EnvContext {
        pipeline_name: "p",
        job_id: "a",
        run_id: Some("r1"),
        attempt: 1,
    }
}

/// `merge_env` を 5 回測り、中央値と最後の結果を返す。
fn measure(
    def: &PipelineDefinition,
    files: &LoadedEnvFiles,
) -> (Duration, Vec<Duration>, EffectiveEnv) {
    let mut last = None;
    let samples: Vec<Duration> = (0..SAMPLES)
        .map(|_| {
            let start = Instant::now();
            let env = merge_env(def, &def.job[0], files, &ctx());
            let elapsed = start.elapsed();
            last = Some(env);
            elapsed
        })
        .collect();
    (median(samples.clone()), samples, last.expect("1 回は測る"))
}

/// 結果の名前の並びが、`names` のあとに自動の変数 4 つが続く形か(BR4.3)。
fn assert_order(env: &EffectiveEnv, names: &[String]) {
    assert_eq!(env.entries.len(), names.len() + BUILTINS.len());
    for (e, n) in env.entries.iter().zip(names) {
        assert_eq!(&e.name, n);
    }
    for (e, b) in env.entries[names.len()..].iter().zip(BUILTINS) {
        assert_eq!(e.name, b);
    }
}

#[test]
fn a3a_一万項目の_env_file_一つの統合は_50ms_以内() {
    let names: Vec<String> = (0..MAX_ENV_ENTRIES).map(|i| format!("K{i}")).collect();
    let d = def(false);
    let mut files = LoadedEnvFiles::new();
    add_env_file(&mut files, "p.env", content("p.env", &names));

    let (m, samples, env) = measure(&d, &files);
    eprintln!("A3a: 中央値 {m:?}(各回 {samples:?})");
    assert_order(&env, &names);
    assert!(
        env.entries[..names.len()]
            .iter()
            .all(|e| e.value.starts_with("p.env-"))
    );
    assert!(
        m < Duration::from_millis(50),
        "中央値 {m:?} が上限 50ms を超えた"
    );
}

#[test]
fn a3b_同じ名前の一万項目の_env_file_二つの統合は_100ms_以内で_後ろの層の並びになる() {
    let names: Vec<String> = (0..MAX_ENV_ENTRIES).map(|i| format!("K{i}")).collect();
    // `[job].env_file` は逆の順に書く。同じ名前は最後に足した位置に並ぶ(BR4.3)。
    let reversed: Vec<String> = names.iter().rev().cloned().collect();
    let d = def(true);
    let mut files = LoadedEnvFiles::new();
    add_env_file(&mut files, "p.env", content("p.env", &names));
    add_env_file(&mut files, "j.env", content("j.env", &reversed));

    let (m, samples, env) = measure(&d, &files);
    eprintln!("A3b: 中央値 {m:?}(各回 {samples:?})");
    assert_order(&env, &reversed);
    assert!(
        env.entries[..names.len()]
            .iter()
            .all(|e| e.value.starts_with("j.env-"))
    );
    assert!(
        m < Duration::from_millis(100),
        "中央値 {m:?} が上限 100ms を超えた"
    );
}
