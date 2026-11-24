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

//! 変換の出力が毎回同じであること。往復(BR1.11。変換した結果を読み戻した木が、入力の木と
//! 等しい)の木を比べるテストは、単体テストの `src/tests/convert_roundtrip.rs` にある。

use conductor_converter::{TargetFormat, convert_text};

const PIPELINE_TOML: &str = include_str!("fixtures/pipeline.toml");
const PIPELINE_YAML: &str = include_str!("fixtures/pipeline.yaml");

#[test]
fn 出力は毎回同じ() {
    let first = convert_text(PIPELINE_YAML, TargetFormat::Toml, "t").unwrap();
    let second = convert_text(PIPELINE_YAML, TargetFormat::Toml, "t").unwrap();
    assert_eq!(first, second);
    let first = convert_text(PIPELINE_TOML, TargetFormat::Yaml, "t").unwrap();
    let second = convert_text(PIPELINE_TOML, TargetFormat::Yaml, "t").unwrap();
    assert_eq!(first, second);
}
