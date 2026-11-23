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

//! `.env` の解析の重複キーの扱いの性能(types-perf-visible の 2d。BR2.4、BR2.6)。
//!
//! 項目の数の上限(10,000 行)の `.env` で、後半 5,000 行が前半の名前の逆順の重複のとき、1 回の解析の
//! 時間の、5 回の中央値が上限の中にあることを確かめる。上限は、直した後の見積もりの約 5 倍の余裕の
//! ある値で、大きな退行だけを止める(D6)。直す前の形に戻ったことは、このテストでは検知できない
//! (直す前と後の時間は、コード生成の記録に残す)。

mod common;

use std::time::{Duration, Instant};

use common::median;
use conductor_core::envfile::MAX_ENV_ENTRIES;
use conductor_core::{EnvWarningKind, parse_env_file};

const HALF: usize = MAX_ENV_ENTRIES / 2;
const SAMPLES: usize = 5;
/// P5 の上限(debug。直した後の見積もり約 22 ms の約 5 倍。D6)。
const LIMIT: Duration = Duration::from_millis(100);

#[test]
fn p5_一万行で後半が逆順の重複の_env_の解析は_100ms_以内() {
    // 前半: `K0=a0` … `K4999=a4999`。後半: `K4999=b4999` … `K0=b0`(逆順の重複)。
    let mut text = String::new();
    for i in 0..HALF {
        text.push_str(&format!("K{i}=a{i}\n"));
    }
    for i in (0..HALF).rev() {
        text.push_str(&format!("K{i}=b{i}\n"));
    }

    let mut last = None;
    let samples: Vec<Duration> = (0..SAMPLES)
        .map(|_| {
            let start = Instant::now();
            let content = parse_env_file(&text, "big.env").expect("正しい .env");
            let elapsed = start.elapsed();
            last = Some(content);
            elapsed
        })
        .collect();
    let m = median(samples.clone());
    eprintln!("P5: 中央値 {m:?}(各回 {samples:?})");

    // 結果の中身: 項目は 5,000 個で、後半の順(`K4999` … `K0`)、値は後ろのもの。重複の警告が 5,000 件。
    let content = last.expect("解析した");
    assert_eq!(content.entries.len(), HALF);
    for (k, entry) in content.entries.iter().enumerate() {
        let i = HALF - 1 - k;
        assert_eq!(entry.name, format!("K{i}"));
        assert_eq!(entry.value, format!("b{i}"));
        assert_eq!(entry.line, HALF + k + 1);
    }
    assert_eq!(content.warnings.len(), HALF);
    assert!(
        content
            .warnings
            .iter()
            .all(|w| w.kind == EnvWarningKind::DuplicateKey)
    );
    assert_eq!(content.warnings[0].name, format!("K{}", HALF - 1));
    assert!(m < LIMIT, "中央値 {m:?} が上限 {LIMIT:?} を超えた");
}
