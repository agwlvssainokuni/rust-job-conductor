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

//! 読む対象の `env_file` の一覧の性能(leftovers-and-fifo の 1h-3。BR2.3)。
//!
//! `[pipeline].env_file` に別々の記述 10,000 個と、ジョブの `env_file` に同じ 10,000 個を書いた定義で、
//! `env_file_refs(def, None)` の時間の 5 回の中央値が上限の中にあることを確かめる。上限は、直した後の
//! 見積もりの約 5 倍(数 ms の揺れを見込んで丸めた)の余裕のある値で、大きな退行だけを止める(D9)。

mod common;

use std::time::{Duration, Instant};

use common::{env_ref, median};
use conductor_core::envfile::MAX_ENV_ENTRIES;
use conductor_core::{EnvFileRef, PipelineDefinition, env_file_refs, parse_definition};

const SAMPLES: usize = 5;
/// P3 の上限(debug。直した後の見積もり約 7 ms の約 5 倍を丸めた。D9)。
const LIMIT: Duration = Duration::from_millis(40);

#[test]
fn p3_一万個の_env_file_の記述の一覧は_40ms_以内() {
    let refs: Vec<EnvFileRef> = (0..MAX_ENV_ENTRIES)
        .map(|i| env_ref(&format!("e{i}.env"), true))
        .collect();
    let mut definition = empty_definition();
    definition.env_file = refs.clone();
    definition.job[0].env_file = refs.clone();

    let mut last = None;
    let samples: Vec<Duration> = (0..SAMPLES)
        .map(|_| {
            let start = Instant::now();
            let out = env_file_refs(&definition, None);
            let elapsed = start.elapsed();
            last = Some(out);
            elapsed
        })
        .collect();
    let m = median(samples.clone());
    eprintln!("P3: 中央値 {m:?}(各回 {samples:?})");

    // 結果の中身: 10,000 個で、`[pipeline].env_file` に書いた順(ジョブの記述は全部重複)。
    assert_eq!(last.expect("作った"), refs);
    assert_eq!(env_file_refs(&definition, Some("a")), refs);
    // ジョブの記述に新しい 1 個を足すと、末尾に 1 個増える。
    definition.job[0].env_file.push(env_ref("new.env", true));
    let out = env_file_refs(&definition, Some("a"));
    assert_eq!(out.len(), MAX_ENV_ENTRIES + 1);
    assert_eq!(out.last(), Some(&env_ref("new.env", true)));
    assert_eq!(out[..MAX_ENV_ENTRIES], refs[..]);

    assert!(m < LIMIT, "中央値 {m:?} が上限 {LIMIT:?} を超えた");
}

/// ジョブが 1 つ(`a`)で、`env_file` を書いていない定義。
fn empty_definition() -> PipelineDefinition {
    parse_definition(
        "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"c\"\n",
        "t.toml",
        None,
    )
    .expect("正しい定義")
}
