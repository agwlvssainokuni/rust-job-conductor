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

//! 定義の検査の性能(leftovers-and-fifo の 1h-1、1h-2。BR2.1、BR2.2)。
//!
//! 定義の `env` の表の大文字小文字の検査(P1)と、遷移の重複の確認(P2)が、大きな入力でも二乗の
//! 時間にならないことを、1 回の `parse_definition` の時間の 5 回の中央値で確かめる。上限は、直した
//! 後の見積もりの約 5 倍の余裕のある値で、大きな退行だけを止める(D9)。直す前の形に戻ったことは、
//! このテストでは確実には検知できない(直す前と後の時間は、コード生成の記録に残す)。
//!
//! P3 は、v0.10.0 の既定のリトライ(`[pipeline.retry]`)を 1 万のジョブに当てはめる定義の読み込みが、
//! 同じ数のジョブで既定を持たない定義と同じ程度に収まることを確かめる(u1-pipeline-retry の NFR1.3)。

mod common;

use std::time::{Duration, Instant};

use common::median;
use conductor_core::envfile::MAX_ENV_ENTRIES;
use conductor_core::{DefinitionErrorKind as K, parse_definition};

const SAMPLES: usize = 5;
/// P1 の上限(debug。直した後の見積もり約 72 ms の約 5 倍。D9)。
const LIMIT_P1: Duration = Duration::from_millis(350);
/// P2 の上限(debug。直した後の見積もり約 324 ms の約 5 倍。D9)。
const LIMIT_P2: Duration = Duration::from_millis(1_600);
/// P2 の、1 つのジョブに書く `success` と `warning` の遷移の数(それぞれ)。
const TRANSITIONS: usize = 20_000;
/// P3 の上限(debug。u1-pipeline-retry の NFR1.3、NFR1.4。実測の約 140 ms(既定なしの同じ数のジョブで
/// 約 137 ms)の約 5 倍。D9 と同じ決め方)。
const LIMIT_P3: Duration = Duration::from_millis(700);
/// P3 の、既定を当てはめるジョブの数。
const DEFAULT_RETRY_JOBS: usize = 10_000;

/// `[pipeline.env]` に `K0` … `K<n-1>` を書いた定義(ジョブは 1 つ)。
fn env_definition(n: usize, extra: &str) -> String {
    let mut text = String::from("format_version = 1\n[pipeline]\nname = \"p\"\n[pipeline.env]\n");
    for i in 0..n {
        text.push_str(&format!("K{i} = \"v\"\n"));
    }
    text.push_str(extra);
    text.push_str("[[job]]\nid = \"a\"\ncommand = \"c\"\n");
    text
}

#[test]
fn p1_一万項目の_pipeline_env_の定義の読み込みは_350ms_以内() {
    let text = env_definition(MAX_ENV_ENTRIES, "");

    let mut last = None;
    let samples: Vec<Duration> = (0..SAMPLES)
        .map(|_| {
            let start = Instant::now();
            let definition = parse_definition(&text, "t.toml", None).expect("正しい定義");
            let elapsed = start.elapsed();
            last = Some(definition);
            elapsed
        })
        .collect();
    let m = median(samples.clone());
    eprintln!("P1: 中央値 {m:?}(各回 {samples:?}、{} バイト)", text.len());

    // 結果の中身: `[pipeline.env]` は 10,000 項目で、書いた順。
    let definition = last.expect("読んだ");
    assert_eq!(definition.env.len(), MAX_ENV_ENTRIES);
    for (i, var) in definition.env.iter().enumerate() {
        assert_eq!(var.name, format!("K{i}"));
    }

    // 大文字小文字だけが違う名前を最後に足すと、誤りは 1 件だけ(検査が働く。時間は測らない)。
    let conflicting = env_definition(MAX_ENV_ENTRIES, "k0 = \"x\"\n");
    let errors = parse_definition(&conflicting, "t.toml", None).expect_err("誤りになる");
    let kinds: Vec<K> = errors.iter().map(|e| e.kind).collect();
    assert_eq!(kinds, [K::EnvNameCaseConflict]);

    assert!(m < LIMIT_P1, "中央値 {m:?} が上限 {LIMIT_P1:?} を超えた");
}

/// `[pipeline.retry]` を書き(`with_default` のとき)、`retry` を書かないジョブを `n` 個並べた定義。
/// 各ジョブは警告の終了コードを持ち、既定との重なりの検査をすべてのジョブで行う(重ならない値)。
fn default_retry_definition(n: usize, with_default: bool) -> String {
    let mut text = String::from("format_version = 1\n[pipeline]\nname = \"p\"\n");
    if with_default {
        text.push_str(
            "[pipeline.retry]\nexit_codes = [75, 76, 77]\nmax = 3\ninterval = \"30s\"\nmultiplier = 2\nmax_interval = \"10m\"\n",
        );
    }
    for i in 0..n {
        text.push_str(&format!(
            "[[job]]\nid = \"j{i}\"\ncommand = \"c\"\nwarning_exit_codes = [3, 4]\n"
        ));
    }
    text
}

/// 1 回の `parse_definition` の時間を `SAMPLES` 回測り、中央値と最後の結果を返す。
fn measure(text: &str) -> (Duration, conductor_core::PipelineDefinition) {
    let mut last = None;
    let samples: Vec<Duration> = (0..SAMPLES)
        .map(|_| {
            let start = Instant::now();
            let definition = parse_definition(text, "t.toml", None).expect("正しい定義");
            let elapsed = start.elapsed();
            last = Some(definition);
            elapsed
        })
        .collect();
    (median(samples), last.expect("読んだ"))
}

#[test]
fn p3_一万のジョブに既定のリトライを当てはめる定義の読み込みは_700ms_以内() {
    let with_default = default_retry_definition(DEFAULT_RETRY_JOBS, true);
    let without_default = default_retry_definition(DEFAULT_RETRY_JOBS, false);

    let (m, definition) = measure(&with_default);
    let (base, _) = measure(&without_default);
    eprintln!(
        "P3: 既定あり 中央値 {m:?}、既定なし 中央値 {base:?}({} バイト)",
        with_default.len()
    );

    // 結果の中身: すべてのジョブに、既定と同じリトライが入る(NFR1.3)。
    assert_eq!(definition.job.len(), DEFAULT_RETRY_JOBS);
    let first = definition.job[0].retry.clone().expect("既定が入る");
    assert_eq!(first.exit_codes, vec![75, 76, 77]);
    assert!(
        definition
            .job
            .iter()
            .all(|j| j.retry.as_ref() == Some(&first))
    );

    assert!(m < LIMIT_P3, "中央値 {m:?} が上限 {LIMIT_P3:?} を超えた");
}

#[test]
fn p2_一つのジョブに重複した遷移が四万個の定義の読み込みは_1600ms_以内() {
    // 1 つのジョブに、`success` の遷移 20,000 個、続けて `warning` の遷移 20,000 個(インラインの配列。
    // 1 行に 1 つ)。後半の各確認が前半をすべて読む、最悪の並び。
    let mut text = String::from(
        "format_version = 1\n[pipeline]\nname = \"p\"\n\
         [[job]]\nid = \"a\"\ncommand = \"c\"\nwarning_exit_codes = [3]\ntransitions = [\n",
    );
    let first_line = text.lines().count() + 1;
    for on in ["success", "warning"] {
        for _ in 0..TRANSITIONS {
            text.push_str(&format!("{{on=\"{on}\",next=\"b\"}},\n"));
        }
    }
    text.push_str("]\n[[job]]\nid = \"b\"\ncommand = \"c\"\n");

    let mut last = None;
    let samples: Vec<Duration> = (0..SAMPLES)
        .map(|_| {
            let start = Instant::now();
            let result = parse_definition(&text, "t.toml", None);
            let elapsed = start.elapsed();
            last = Some(result);
            elapsed
        })
        .collect();
    let m = median(samples.clone());
    eprintln!("P2: 中央値 {m:?}(各回 {samples:?}、{} バイト)", text.len());

    // 結果の中身: 誤りは 39,998 件で、すべて `duplicateTransition`(最初の `success` と最初の
    // `warning` は誤りでない)。誤りは位置の順(行の順)。
    let errors = last.expect("読んだ").expect_err("誤りになる");
    assert_eq!(errors.len(), 2 * (TRANSITIONS - 1));
    assert!(errors.iter().all(|e| e.kind == K::DuplicateTransition));
    let lines: Vec<usize> = errors.iter().map(|e| e.line.expect("行がある")).collect();
    let success_dups = (first_line + 1)..(first_line + TRANSITIONS);
    let warning_dups = (first_line + TRANSITIONS + 1)..(first_line + 2 * TRANSITIONS);
    let expected: Vec<usize> = success_dups.chain(warning_dups).collect();
    assert_eq!(lines, expected);

    assert!(m < LIMIT_P2, "中央値 {m:?} が上限 {LIMIT_P2:?} を超えた");
}
