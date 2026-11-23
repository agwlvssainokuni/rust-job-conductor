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
