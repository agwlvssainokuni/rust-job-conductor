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

//! NFR1.1: 起動前の処理の時間(ジョブ 10 個の定義)。基準(p95 20ms。仮)を超えても、失敗にしない。
//! 測定値は標準エラーに出し、`GITHUB_STEP_SUMMARY` があれば、そこにも書き出す。

mod common;

use std::io::Write;
use std::time::{Duration, Instant};

use std::path::Path;

use conductor_core::{PipelineDefinition, PipelineState};
use conductor_runtime::{
    ConsistencyResult, Durability, StateError, check_command, check_consistency,
    initial_state as initial_state_with, read_state as read_state_with,
    write_state as write_state_with,
};

// run ID なしの呼び出し(この測定は run ID を使わない)。
fn read_state(def: &Path) -> Result<Option<PipelineState>, StateError> {
    read_state_with(def, None)
}
fn initial_state(d: &PipelineDefinition, def: &Path) -> PipelineState {
    initial_state_with(d, def, None)
}
fn write_state(def: &Path, s: &PipelineState, dur: Durability) -> Result<(), StateError> {
    write_state_with(def, None, s, dur)
}

const RUNS: usize = 100;
const TARGET: Duration = Duration::from_millis(20);

fn definition() -> PipelineDefinition {
    PipelineDefinition {
        name: "bench".into(),
        description: None,
        run_id_env: None,
        remove_on_complete: false,
        format_version: 1,
        job: (0..10)
            .map(|i| {
                common::job(
                    &format!("job{i}"),
                    format!("prog --opt 'value {i}' \"x y\" z{i}"),
                )
            })
            .collect(),
        env: vec![],
        env_file: vec![],
    }
}

fn p95(samples: &mut [Duration]) -> Duration {
    samples.sort();
    // 昇順の 95 番目(100 回のとき、添字 94)。
    let rank = (samples.len() * 95).div_ceil(100).max(1);
    samples[rank - 1]
}

fn report(line: &str) {
    eprintln!("{line}");
    if let Ok(path) = std::env::var("GITHUB_STEP_SUMMARY")
        && let Ok(mut f) = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
    {
        let _ = writeln!(f, "{line}");
    }
}

#[test]
fn 起動前の処理の_p95_を測る() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let def_path = dir.path().join("conductor.toml");
    let d = definition();

    let mut fast = Vec::with_capacity(RUNS);
    for _ in 0..RUNS {
        let start = Instant::now();
        for job in &d.job {
            assert!(check_command(&job.command).is_ok());
        }
        let existing = read_state(&def_path).expect("読める");
        if let Some(state) = &existing {
            assert_eq!(check_consistency(state, &d), ConsistencyResult::Ok);
        }
        let state = existing.unwrap_or_else(|| initial_state(&d, &def_path));
        write_state(&def_path, &state, Durability::Fast).expect("書ける");
        fast.push(start.elapsed());
    }

    let mut durable = Vec::with_capacity(RUNS);
    let state = initial_state(&d, &def_path);
    for _ in 0..RUNS {
        let start = Instant::now();
        write_state(&def_path, &state, Durability::Durable).expect("書ける");
        durable.push(start.elapsed());
    }

    let fast_p95 = p95(&mut fast);
    let durable_p95 = p95(&mut durable);
    report(&format!(
        "NFR1.1 conductor-runtime: 起動前の処理(ジョブ 10 個、{RUNS} 回、同期なし)p95 = {fast_p95:?}(基準 {TARGET:?}、仮)"
    ));
    report(&format!(
        "NFR1.1 conductor-runtime: 結果の書き込み(Durable、{RUNS} 回)p95 = {durable_p95:?}(基準なし。記録のみ)"
    ));
    if fast_p95 > TARGET {
        report(&format!(
            "警告: 起動前の処理の p95 が基準を超えました({fast_p95:?} > {TARGET:?})。失敗にはしません"
        ));
    }
}
