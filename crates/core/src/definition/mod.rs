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

//! 定義の読み込みと検証(DefinitionLoader)。
//!
//! 流れは Intake → Parse → Depth → VersionCheck → Walk(Locate を使う)。
//! 入出力に触れるのは Intake だけで、それ以外は、文字列だけを受け取る純粋な処理である。

pub mod depth;
pub(crate) mod duration;
pub mod intake;
pub mod parse;
pub mod version;
pub mod walk;

use std::path::Path;

use toml::de::DeInteger;

use crate::error::{DefinitionError, Locate};
use crate::types::PipelineDefinition;

/// TOML の整数を `i64` で取り出す(`i64` に収まらなければ `None`)。走査と版の確認が共有する。
pub(crate) fn integer_value(int: &DeInteger<'_>) -> Option<i64> {
    i64::from_str_radix(int.as_str(), int.radix()).ok()
}

/// 定義ファイルを読み込み、検証して、`PipelineDefinition` を返す(契約 C2 の `load_definition`)。
///
/// - `path`: 定義ファイルのパス。`None` のときは、実行ディレクトリの `conductor.toml`(BR4.1)。
/// - `expected_pipeline_name`: `pipeline` コマンドで渡された名前。あれば、定義の `name` と比べる(BR1.8)。
///
/// 失敗のときは、見つかった誤りをすべて返す(ファイルの取り込みや文法・版の誤りは、1 件だけ)。
/// ジョブは起動しない。ファイルへの書き込みもしない。
pub fn load_definition(
    path: Option<&Path>,
    expected_pipeline_name: Option<&str>,
) -> Result<PipelineDefinition, Vec<DefinitionError>> {
    let path = intake::resolve_path(path);
    let text = intake::read_definition(&path).map_err(|e| vec![e])?;
    parse_definition(&text, &intake::file_label(&path), expected_pipeline_name)
}

/// 文字列として渡された定義を、検証して `PipelineDefinition` にする。
///
/// ファイルに触れないため、検証だけを試せる。`file_label` は、エラーの表示に使う名前。
/// 定義の内容は、実行も評価もしない(コマンドの文字列は、そのまま保持する。NFR4.5)。
pub fn parse_definition(
    text: &str,
    file_label: &str,
    expected_pipeline_name: Option<&str>,
) -> Result<PipelineDefinition, Vec<DefinitionError>> {
    let locate = Locate::new(text);
    let doc = parse::parse_document(text, file_label, &locate).map_err(|e| vec![e])?;
    let root = doc.get_ref();
    depth::check_depth(root, file_label, &locate).map_err(|e| vec![e])?;
    let format_version = version::check_version(root, file_label, &locate).map_err(|e| vec![e])?;
    walk::walk(
        root,
        format_version,
        file_label,
        &locate,
        expected_pipeline_name,
    )
}
