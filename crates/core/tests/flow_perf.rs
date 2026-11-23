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

//! `@end` に着いたときの残りのジョブの列挙の性能(internal-cleanup-2 の A2。BR2.1、BR2.11)。
//!
//! ジョブ 1,000 個の一連の実行で、先頭のジョブの成功で `@end` に着き、残りの 999 個を
//! `skipped_job_ids` に列挙する。この判断を 100 回くり返す時間の、5 回の中央値が 50 ms 以内
//! (ジョブの数に比例する時間。二乗の時間に戻ると、手元の機械でも上限を大きく超える)。

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use common::median;
use conductor_core::{
    JobExecution, RunMode, SkipReason, classify_and_decide, initial_state, parse_definition,
};

const N: usize = 1000;
const REPEAT: usize = 100;
const SAMPLES: usize = 5;
const LIMIT: Duration = Duration::from_millis(50);

#[test]
fn a2_千個のジョブで終点に着いたときの残りの列挙を_100_回くり返しても_50ms_以内() {
    let mut text = String::from(
        "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"j0\"\ncommand = \"echo 0\"\n[[job.transitions]]\non = \"success\"\nnext = \"@end\"\n",
    );
    for i in 1..N {
        text.push_str(&format!("[[job]]\nid = \"j{i}\"\ncommand = \"echo {i}\"\n"));
    }
    let def = parse_definition(&text, "t.toml", None).expect("正しい定義");
    let state = initial_state(&def, Path::new("t.toml"), None);
    let execution = JobExecution::exited(0);

    // 結果の中身: 残りの 999 個が記述順に `ByEnd` で並び、完了する。
    let (_, next) = classify_and_decide(
        &def.job[0],
        &execution,
        &state.job_states,
        &def,
        RunMode::Pipeline,
    );
    let expected: Vec<String> = (1..N).map(|i| format!("j{i}")).collect();
    assert_eq!(next.skipped_job_ids, expected);
    assert_eq!(next.skip_reason_kind, Some(SkipReason::ByEnd));
    assert!(next.completed && next.end_reached);

    let samples: Vec<Duration> = (0..SAMPLES)
        .map(|_| {
            let start = Instant::now();
            for _ in 0..REPEAT {
                let (_, next) = classify_and_decide(
                    &def.job[0],
                    &execution,
                    &state.job_states,
                    &def,
                    RunMode::Pipeline,
                );
                assert_eq!(next.skipped_job_ids.len(), N - 1);
            }
            start.elapsed()
        })
        .collect();
    let m = median(samples.clone());
    eprintln!("A2: {REPEAT} 回の中央値 {m:?}(各回 {samples:?})");
    assert!(m < LIMIT, "中央値 {m:?} が上限 {LIMIT:?} を超えた");
}
