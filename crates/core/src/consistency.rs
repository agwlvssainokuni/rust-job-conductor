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

//! ステートと定義の照合(Consistency。BR2.5)。純粋な関数。
//!
//! conductor-runtime は、同じ名前で再エクスポートする(`conductor_runtime::check_consistency` など)。

use crate::types::{PipelineDefinition, PipelineState};

/// ステートと定義の食い違いの種類(BR2.5)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MismatchKind {
    /// パイプライン名の違い。
    PipelineName,
    /// ジョブIDの追加・削除・改名。
    JobIds,
    /// 記述順だけの違い。
    JobOrder,
}

/// ステートと定義の照合の結果(契約 C3 の `check_consistency`)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsistencyResult {
    Ok,
    Mismatch(MismatchKind),
}

/// ステートと定義の食い違いを検知する(契約 C3 の `check_consistency`。W4)。
///
/// パイプライン名 → ジョブIDの集合 → 並び、の順に比べる。`definition_key` と `definition_path` は、
/// 照合に使わない(定義ファイルの置き場所の移動で、食い違いにしないため)。
pub fn check_consistency(
    state: &PipelineState,
    definition: &PipelineDefinition,
) -> ConsistencyResult {
    if state.pipeline_name != definition.name {
        return ConsistencyResult::Mismatch(MismatchKind::PipelineName);
    }
    let defined: Vec<&str> = definition.job.iter().map(|j| j.id.as_str()).collect();
    let recorded: Vec<&str> = state.job_order.iter().map(String::as_str).collect();
    if defined == recorded {
        return ConsistencyResult::Ok;
    }
    let mut a = defined;
    let mut b = recorded;
    a.sort_unstable();
    b.sort_unstable();
    if a == b {
        ConsistencyResult::Mismatch(MismatchKind::JobOrder)
    } else {
        ConsistencyResult::Mismatch(MismatchKind::JobIds)
    }
}
