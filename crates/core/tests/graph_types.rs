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

mod common;

use conductor_core::JobStatus;
use conductor_core::graph::{
    EdgeKind, EdgeTarget, FlowEdge, FlowGraph, GraphLayout, GraphNode, LaneGlyph, LaneLayout,
    LaneRow, NodeKind, NodeState,
};

#[test]
fn 辺は_kinds_を1つ以上持ち_行き先はジョブか終点() {
    let edge = FlowEdge {
        from_job: "a".into(),
        to: EdgeTarget::Job("c".into()),
        kinds: vec![EdgeKind::Success, EdgeKind::Warning],
        skipped_jobs: vec!["b".into()],
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
    use conductor_core::graph::overlay_from_state;
    let state = common::state(
        &[
            ("a", JobStatus::Success),
            ("b", JobStatus::Warning),
            ("c", JobStatus::Skipped),
            ("d", JobStatus::Error),
            ("e", JobStatus::Pending),
        ],
        None,
        false,
    );
    let states: Vec<NodeState> = overlay_from_state(&state)
        .states
        .into_iter()
        .map(|(_, s)| s)
        .collect();
    assert_eq!(
        states,
        [
            NodeState::Ok,
            NodeState::Ok,
            NodeState::Skipped,
            NodeState::Error,
            NodeState::Pending,
        ]
    );
}
