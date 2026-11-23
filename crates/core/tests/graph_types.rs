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

//! 図の型(entities.md。BR4.1)のテスト。

use conductor_core::JobStatus;
use conductor_core::graph::{
    EdgeKind, EdgeTarget, FlowEdge, FlowGraph, GraphLayout, GraphNode, LaneGlyph, LaneLayout,
    LaneRow, NodeKind, NodeState, StateOverlay,
};

#[test]
fn 辺は_kinds_を1つ以上持ち_行き先はジョブか終点() {
    let edge = FlowEdge {
        from_job: "a".into(),
        to: EdgeTarget::Job("c".into()),
        kinds: vec![EdgeKind::Success, EdgeKind::Warning],
        skipped_jobs: vec!["b".into()],
        explicit: true,
    };
    assert!(!edge.kinds.is_empty());
    assert_eq!(edge.to, EdgeTarget::Job("c".into()));
    let end = FlowEdge {
        to: EdgeTarget::End,
        ..edge.clone()
    };
    assert_ne!(end, edge);
}

#[test]
fn 状態の重ね合わせの既定は空で_再開位置もない() {
    let overlay = StateOverlay::default();
    assert!(overlay.states.is_empty());
    assert!(overlay.resume_job_id.is_none());
    let overlay = StateOverlay {
        states: vec![("a".into(), NodeState::Ok)],
        resume_job_id: Some("b".into()),
    };
    assert_eq!(overlay.states[0].1, NodeState::Ok);
}

#[test]
fn 図の構造は節と辺を持ち_終点の節にはジョブidがない() {
    let graph = FlowGraph {
        nodes: vec![
            GraphNode {
                kind: NodeKind::Job("a".into()),
                state: Some(NodeState::Pending),
                is_resume: false,
            },
            GraphNode {
                kind: NodeKind::End,
                state: None,
                is_resume: false,
            },
        ],
        edges: Vec::new(),
    };
    assert_eq!(graph.nodes.len(), 2);
    assert!(graph.nodes[1].job_id().is_none() && graph.nodes[1].is_end());
}

#[test]
fn レーンの配置は行の並びで_配置の結果はレーンか辺の一覧() {
    let row = LaneRow {
        glyphs: vec![LaneGlyph::Node, LaneGlyph::Empty],
        node_index: Some(0),
    };
    let layout = GraphLayout::Lanes(LaneLayout {
        rows: vec![row],
        main_line: vec![LaneGlyph::Line, LaneGlyph::Empty],
    });
    assert!(
        matches!(layout, GraphLayout::Lanes(ref l) if l.rows.len() == 1 && l.main_line.len() == 2)
    );
    assert_ne!(layout, GraphLayout::EdgeList);
}

#[test]
fn ジョブの状態は成功と警告を_ok_にまとめる() {
    use conductor_core::graph::node_state_of;
    assert_eq!(node_state_of(JobStatus::Success), NodeState::Ok);
    assert_eq!(node_state_of(JobStatus::Warning), NodeState::Ok);
    assert_eq!(node_state_of(JobStatus::Skipped), NodeState::Skipped);
    assert_eq!(node_state_of(JobStatus::Error), NodeState::Error);
    assert_eq!(node_state_of(JobStatus::Pending), NodeState::Pending);
}
