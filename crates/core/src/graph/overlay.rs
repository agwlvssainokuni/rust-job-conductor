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

//! 図の構造と状態の重ね合わせ(BR2.1、BR4.1、BR4.2)。

use super::{FlowGraph, GraphNode, NodeKind, NodeState, StateOverlay, flow_edges};
use crate::types::{JobStatus, PipelineDefinition, PipelineState};

/// ステートの内容から、重ね合わせを作る(BR4.1〜BR4.3)。
///
/// 完了したステートは、ジョブの状態を重ねるが、再開位置は使わない(BR4.2)。
pub fn overlay_from_state(state: &PipelineState) -> StateOverlay {
    StateOverlay {
        states: state
            .job_states
            .iter()
            .map(|s| (s.job_id.clone(), node_state_of(s.status)))
            .collect(),
        resume_job_id: if state.completed {
            None
        } else {
            state.resume_job_id.clone()
        },
    }
}

/// ステートが無いときの重ね合わせ。すべてのジョブが未実行(BR4.3)。
pub fn pending_overlay(definition: &PipelineDefinition) -> StateOverlay {
    StateOverlay {
        states: definition
            .job
            .iter()
            .map(|j| (j.id.clone(), NodeState::Pending))
            .collect(),
        resume_job_id: None,
    }
}

/// 図の構造を作る。節は、定義の記述順で、最後に終点(BR2.1)。`overlay` があれば、状態を重ねる。
/// 重ね合わせに無いジョブは、未実行として扱う。
pub fn build_graph(definition: &PipelineDefinition, overlay: Option<&StateOverlay>) -> FlowGraph {
    let mut nodes: Vec<GraphNode> = definition
        .job
        .iter()
        .map(|job| {
            let state = overlay.map(|o| {
                o.states
                    .iter()
                    .find(|(id, _)| *id == job.id)
                    .map_or(NodeState::Pending, |(_, state)| *state)
            });
            let is_resume = overlay.is_some_and(|o| o.resume_job_id.as_deref() == Some(&job.id));
            GraphNode {
                kind: NodeKind::Job(job.id.clone()),
                state,
                is_resume,
            }
        })
        .collect();
    nodes.push(GraphNode {
        kind: NodeKind::End,
        state: None,
        is_resume: false,
    });
    FlowGraph {
        nodes,
        edges: flow_edges(definition),
    }
}

/// ジョブの状態から、図の状態を決める(BR4.1)。成功と警告は `Ok` にまとめる。
pub fn node_state_of(status: JobStatus) -> NodeState {
    match status {
        JobStatus::Success | JobStatus::Warning => NodeState::Ok,
        JobStatus::Skipped => NodeState::Skipped,
        JobStatus::Error => NodeState::Error,
        JobStatus::Pending => NodeState::Pending,
    }
}
