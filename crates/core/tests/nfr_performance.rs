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

//! NFR1.1: ジョブ 10 個の定義を、取り込みから結果まで 100 回読んで、p95 を測る。
//!
//! 基準(20ms)を超えても、失敗にはしない(Q1: A)。測定値は標準エラーに出し、
//! `GITHUB_STEP_SUMMARY` があれば、そこにも書き出す。

use std::io::Write;
use std::time::{Duration, Instant};

use conductor_core::load_definition;

const RUNS: usize = 100;
const BUDGET: Duration = Duration::from_millis(20);

fn ten_job_definition() -> String {
    let mut text = String::from(
        "format_version = 1\n[pipeline]\nname = \"bench\"\ndescription = \"ジョブ 10 個\"\n",
    );
    for i in 0..10 {
        text.push_str(&format!(
            "[[job]]\nid = \"job{i}\"\ncommand = \"run-step --index {i}\"\nwarning_exit_codes = [3]\nsuccess_exit_codes = [0, 2]\n"
        ));
        if i < 8 {
            text.push_str(&format!(
                "[[job.transitions]]\non = \"warning\"\nnext = \"job{}\"\n",
                i + 1
            ));
        }
    }
    text
}

#[test]
fn nfr1_1_ジョブ_10_個の定義の読み込みの_p95() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("conductor.toml");
    std::fs::write(&path, ten_job_definition()).unwrap();

    let mut samples: Vec<Duration> = Vec::with_capacity(RUNS);
    for _ in 0..RUNS {
        let start = Instant::now();
        let def = load_definition(Some(&path), None).expect("正しい定義");
        samples.push(start.elapsed());
        assert_eq!(def.jobs.len(), 10);
    }
    samples.sort();
    // 昇順の 95 番目(1 始まり)。
    let p95 = samples[(RUNS * 95).div_ceil(100) - 1];

    let verdict = if p95 > BUDGET {
        "基準を超えています(警告のみ。失敗にはしない)"
    } else {
        "基準以内"
    };
    let line = format!(
        "NFR1.1 定義の読み込み(ジョブ 10 個、{RUNS} 回): p95 = {:.3} ms、基準 {} ms: {verdict}",
        p95.as_secs_f64() * 1000.0,
        BUDGET.as_millis()
    );
    eprintln!("{line}");

    if let Some(summary) = std::env::var_os("GITHUB_STEP_SUMMARY") {
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(summary)
        {
            let _ = writeln!(f, "{line}");
        }
    }
}
