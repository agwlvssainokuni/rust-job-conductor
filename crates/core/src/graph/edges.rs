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

//! 辺の取り出し(BR1.1〜BR1.5)。
//!
//! 行き先は、実行のときに次のジョブを決める判断と同じ関数([`resolve_transition`])で決める。
//! 図と実行で、判断を二重に書かない(BR1.5、FR2.4)。

use super::{EdgeKind, EdgeTarget, FlowEdge};
use crate::flow::{TransitionResolution, resolve_transition};
use crate::types::{PipelineDefinition, TransitionOn};

/// 結果 `on` で終わったときの行き先の位置を決める。ジョブは 0 から、終点は `jobs.len()`。
/// 戻り値の 2 つ目は、遷移が書かれていたか。
fn target_position(
    definition: &PipelineDefinition,
    index: usize,
    on: TransitionOn,
) -> (usize, bool) {
    let job = &definition.job[index];
    match resolve_transition(job, on, definition, index) {
        TransitionResolution::End => (definition.job.len(), true),
        TransitionResolution::Job(t) => (t, true),
        // 遷移がない。記述順で次のジョブ(最後のジョブなら終点。BR1.1)。
        TransitionResolution::Default => (index + 1, false),
    }
}

/// 定義から、図の辺を取り出す(BR1.1〜BR1.4)。
///
/// - 成功と警告のものだけを作る。エラーに辺はない(BR1.3)。
/// - 成功と警告の行き先が同じなら、1 つにまとめ、`kinds` に両方を持つ(BR1.2)。
/// - 並びは、元のジョブの記述順。同じ元の中では、行き先が近い辺が先(記述順の次のジョブへの辺が
///   先、終点が最後。BR1.2)。
/// - `skipped_jobs` は、元と行き先のあいだのジョブ(終点なら、末尾まで。BR1.4)。
pub fn flow_edges(definition: &PipelineDefinition) -> Vec<FlowEdge> {
    let jobs = &definition.job;
    let mut edges = Vec::with_capacity(jobs.len() + 1);
    for (index, job) in jobs.iter().enumerate() {
        let (on_success, explicit_success) =
            target_position(definition, index, TransitionOn::Success);
        let (on_warning, explicit_warning) =
            target_position(definition, index, TransitionOn::Warning);
        let mut pairs: Vec<(usize, Vec<EdgeKind>, bool)> = if on_success == on_warning {
            vec![(
                on_success,
                vec![EdgeKind::Success, EdgeKind::Warning],
                explicit_success || explicit_warning,
            )]
        } else {
            vec![
                (on_success, vec![EdgeKind::Success], explicit_success),
                (on_warning, vec![EdgeKind::Warning], explicit_warning),
            ]
        };
        // 行き先が近い辺を先に(終点は、位置が最後)。
        pairs.sort_by_key(|(position, _, _)| *position);
        for (position, kinds, explicit) in pairs {
            let to = if position >= jobs.len() {
                EdgeTarget::End
            } else {
                EdgeTarget::Job(jobs[position].id.clone())
            };
            let skipped_jobs = jobs[index + 1..position.min(jobs.len())]
                .iter()
                .map(|j| j.id.clone())
                .collect();
            edges.push(FlowEdge {
                from_job: job.id.clone(),
                to,
                kinds,
                skipped_jobs,
                explicit,
            });
        }
    }
    edges
}
