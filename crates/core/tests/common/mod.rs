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

//! core の統合テストの共通の補助(秘密の値、定義の検査の結果、計った時間の中央値)。
#![allow(dead_code)]

use std::time::Duration;

use conductor_core::DefinitionErrorKind as K;
use conductor_core::parse_definition;

/// 誤りの文やステートに現れてはいけない、秘密の値(NFR4.6)。
pub const SECRET: &str = "s3cr3t-VALUE-do-not-print";

/// 検査の結果を `(種類, 行)` の並びで返す。受理なら空。
pub fn run(text: &str) -> Vec<(K, Option<usize>)> {
    match parse_definition(text, "t.toml", None) {
        Ok(_) => Vec::new(),
        Err(errs) => errs.into_iter().map(|e| (e.kind, e.line)).collect(),
    }
}

/// 検査の誤りの種類の並び。受理なら空。
pub fn kinds(text: &str) -> Vec<K> {
    run(text).into_iter().map(|(k, _)| k).collect()
}

/// 計った時間の中央値。
pub fn median(mut samples: Vec<Duration>) -> Duration {
    samples.sort();
    samples[samples.len() / 2]
}
