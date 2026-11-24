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

//! 受け入れ基準(AC7.1.1、AC7.1.2)のうち、変換の結果を読んだ木を、入力の木と比べるもの。
//! ほかの受け入れ基準は、統合テストの `tests/convert_ac.rs` にある。

use std::path::{Path, PathBuf};

use super::{toml, yaml};
use crate::{ConversionRequest, TargetFormat, convert};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn request(path: PathBuf, target: TargetFormat) -> ConversionRequest {
    ConversionRequest {
        input_path: path,
        target_format: target,
    }
}

#[test]
fn ac_7_1_1_toml_から_yaml() {
    let out = convert(&request(fixture("pipeline.toml"), TargetFormat::Yaml)).unwrap();
    assert_eq!(
        yaml(&out.text).unwrap(),
        toml(include_str!("../../tests/fixtures/pipeline.toml")).unwrap()
    );
    assert!(out.text.contains("format_version: 1"), "{}", out.text);
    assert!(out.text.contains("id: extract"), "{}", out.text);
}

#[test]
fn ac_7_1_2_アンカーつきの_yaml_から_toml() {
    let out = convert(&request(fixture("pipeline.yaml"), TargetFormat::Toml)).unwrap();
    assert_eq!(
        toml(&out.text).unwrap(),
        yaml(include_str!("../../tests/fixtures/pipeline.yaml")).unwrap()
    );
    assert!(!out.text.contains("<<"), "{}", out.text);
    // 展開された結果が、定義の形(表の配列)で出力される
    assert!(out.text.contains("[[job]]"), "{}", out.text);
}
