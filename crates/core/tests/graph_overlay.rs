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

//! 状態の重ね合わせ(BR4.1〜BR4.3)と、図の構造(BR2.1)のテスト。

mod common;
mod graph_common;

use common::state;
use conductor_core::JobStatus;
use conductor_core::graph::{NodeState, build_graph, overlay_from_state, pending_overlay};
use graph_common::definition;

#[test]
fn 図の節は記述順で最後に終点で_状態を重ねない図には状態がない() {
    let graph = build_graph(&definition(&["a", "b", "c"], &[]), None);
    assert_eq!(graph.nodes.len(), 4);
    let ids: Vec<_> = graph.nodes.iter().map(|n| n.job_id()).collect();
    assert_eq!(ids, [Some("a"), Some("b"), Some("c"), None]);
    assert!(graph.nodes[3].is_end() && !graph.nodes[0].is_end());
    assert!(
        graph
            .nodes
            .iter()
            .all(|n| n.state.is_none() && !n.is_resume)
    );
}

#[test]
fn 状態の重ね合わせは_成功と警告が_ok_でスキップとエラーと未実行を区別する() {
    let def = definition(&["a", "b", "c", "d", "e"], &[]);
    let st = state(
        &[
            ("a", JobStatus::Success),
            ("b", JobStatus::Warning),
            ("c", JobStatus::Skipped),
            ("d", JobStatus::Error),
            ("e", JobStatus::Pending),
        ],
        Some("d"),
        false,
    );
    let graph = build_graph(&def, Some(&overlay_from_state(&st)));
    let states: Vec<_> = graph.nodes.iter().map(|n| n.state).collect();
    assert_eq!(
        states,
        [
            Some(NodeState::Ok),
            Some(NodeState::Ok),
            Some(NodeState::Skipped),
            Some(NodeState::Error),
            Some(NodeState::Pending),
            None,
        ]
    );
}

#[test]
fn 再開位置は_完了していないステートのジョブだけに付く() {
    let def = definition(&["a", "b", "c"], &[]);
    let st = state(
        &[
            ("a", JobStatus::Success),
            ("b", JobStatus::Error),
            ("c", JobStatus::Pending),
        ],
        Some("b"),
        false,
    );
    let graph = build_graph(&def, Some(&overlay_from_state(&st)));
    let resume: Vec<_> = graph.nodes.iter().map(|n| n.is_resume).collect();
    assert_eq!(resume, [false, true, false, false]);
}

#[test]
fn 完了したステートでは_状態は重ねるが_再開位置は示さない() {
    let def = definition(&["a", "b"], &[]);
    let st = state(
        &[("a", JobStatus::Success), ("b", JobStatus::Success)],
        Some("b"),
        true,
    );
    let overlay = overlay_from_state(&st);
    assert!(overlay.resume_job_id.is_none());
    let graph = build_graph(&def, Some(&overlay));
    assert!(graph.nodes.iter().all(|n| !n.is_resume));
    assert_eq!(graph.nodes[0].state, Some(NodeState::Ok));
}

#[test]
fn ステートがないときは_すべてのジョブが未実行で_再開位置もない() {
    let def = definition(&["a", "b"], &[]);
    let overlay = pending_overlay(&def);
    let graph = build_graph(&def, Some(&overlay));
    assert_eq!(graph.nodes[0].state, Some(NodeState::Pending));
    assert_eq!(graph.nodes[1].state, Some(NodeState::Pending));
    assert!(graph.nodes.iter().all(|n| !n.is_resume));
    // 重ね合わせに無いジョブも未実行として扱う。
    let partial = conductor_core::graph::StateOverlay {
        states: vec![],
        resume_job_id: None,
    };
    let graph = build_graph(&def, Some(&partial));
    assert_eq!(graph.nodes[1].state, Some(NodeState::Pending));
}
