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

//! 辺の取り出し(BR1.1〜BR1.5)のテスト。

mod graph_common;

use conductor_core::graph::{EdgeKind, EdgeTarget, FlowEdge, flow_edges};
use conductor_core::{
    JobExecution, PipelineDefinition, RunMode, TransitionOn, classify_and_decide,
};
use graph_common::{Spec, definition};

use EdgeKind::{Success, Warning};
use TransitionOn::{Success as OnSuccess, Warning as OnWarning};

fn job(id: &str) -> EdgeTarget {
    EdgeTarget::Job(id.to_string())
}

fn edge(
    from: &str,
    to: EdgeTarget,
    kinds: &[EdgeKind],
    skipped: &[&str],
    explicit: bool,
) -> FlowEdge {
    FlowEdge {
        from_job: from.into(),
        to,
        kinds: kinds.to_vec(),
        skipped_jobs: skipped.iter().map(|s| s.to_string()).collect(),
        explicit,
    }
}

#[test]
fn 遷移がなければ_記述順の次のジョブへの辺が1つで_最後は終点() {
    let edges = flow_edges(&definition(&["a", "b", "c"], &[]));
    assert_eq!(
        edges,
        vec![
            edge("a", job("b"), &[Success, Warning], &[], false),
            edge("b", job("c"), &[Success, Warning], &[], false),
            edge("c", EdgeTarget::End, &[Success, Warning], &[], false),
        ]
    );
}

#[test]
fn 成功だけの遷移は_成功の辺と警告の辺に分かれ_次のジョブへの辺が先() {
    let edges = flow_edges(&definition(
        &["a", "b", "c", "d"],
        &[("a", OnSuccess, Some("d"))],
    ));
    assert_eq!(
        edges[..2],
        [
            edge("a", job("b"), &[Warning], &[], false),
            edge("a", job("d"), &[Success], &["b", "c"], true),
        ]
    );
}

#[test]
fn 警告だけの遷移も_次のジョブへの辺が先() {
    let edges = flow_edges(&definition(
        &["a", "b", "c"],
        &[("a", OnWarning, Some("c"))],
    ));
    assert_eq!(
        edges[..2],
        [
            edge("a", job("b"), &[Success], &[], false),
            edge("a", job("c"), &[Warning], &["b"], true),
        ]
    );
}

#[test]
fn 成功と警告が同じ行き先なら_1つの辺で_kinds_が両方() {
    let edges = flow_edges(&definition(
        &["a", "b", "c"],
        &[("a", OnSuccess, Some("c")), ("a", OnWarning, Some("c"))],
    ));
    assert_eq!(
        edges[0],
        edge("a", job("c"), &[Success, Warning], &["b"], true)
    );
    // 次のジョブ b には、a からの辺がない。
    assert!(edges.iter().filter(|e| e.from_job == "a").count() == 1);
}

#[test]
fn 成功と警告が違う遠い行き先なら_近い辺が先で_終点が最後() {
    let edges = flow_edges(&definition(
        &["a", "b", "c", "d"],
        &[("a", OnSuccess, None), ("a", OnWarning, Some("c"))],
    ));
    assert_eq!(
        edges[..2],
        [
            edge("a", job("c"), &[Warning], &["b"], true),
            edge("a", EdgeTarget::End, &[Success], &["b", "c", "d"], true),
        ]
    );
}

#[test]
fn 終点へ飛ぶと_元より後ろのすべてのジョブを飛び越す() {
    let edges = flow_edges(&definition(&["a", "b", "c"], &[("a", OnSuccess, None)]));
    assert_eq!(
        edges[..2],
        [
            edge("a", job("b"), &[Warning], &[], false),
            edge("a", EdgeTarget::End, &[Success], &["b", "c"], true),
        ]
    );
}

#[test]
fn 最後のジョブから終点への辺は_飛び越しなし() {
    let edges = flow_edges(&definition(&["a", "b"], &[("b", OnSuccess, None)]));
    let last = edges.last().unwrap();
    // 成功は明示、警告は既定だが、どちらも終点なので 1 つにまとまる。
    assert_eq!(
        last,
        &edge("b", EdgeTarget::End, &[Success, Warning], &[], true)
    );
}

#[test]
fn ジョブが1個なら_終点への辺が1つだけ() {
    let edges = flow_edges(&definition(&["only"], &[]));
    assert_eq!(
        edges,
        vec![edge(
            "only",
            EdgeTarget::End,
            &[Success, Warning],
            &[],
            false
        )]
    );
}

/// 実行の判断(`classify_and_decide`)の次のジョブと、図の辺の行き先が、同じになること(BR1.5)。
fn assert_same_as_execution(def: &PipelineDefinition) {
    let edges = flow_edges(def);
    for job in &def.job {
        for (kind, code) in [(Success, 0), (Warning, 3)] {
            let (_, step) = classify_and_decide(
                job,
                &JobExecution::exited(job, code),
                &[],
                def,
                RunMode::Pipeline,
            );
            let from_edges: Vec<&FlowEdge> = edges
                .iter()
                .filter(|e| e.from_job == job.id && e.kinds.contains(&kind))
                .collect();
            assert_eq!(from_edges.len(), 1, "結果ごとに辺は 1 つ: {}", job.id);
            let e = from_edges[0];
            match (&e.to, &step.next_job_id) {
                (EdgeTarget::Job(id), Some(next)) => {
                    assert_eq!(id, next);
                    assert_eq!(e.skipped_jobs, step.skipped_job_ids);
                }
                (EdgeTarget::End, None) => {
                    // `@end` への遷移、または最後のジョブの完了。
                    assert!(step.completed);
                    assert_eq!(e.skipped_jobs, step.skipped_job_ids);
                }
                other => panic!("実行と図が食い違う: {other:?}"),
            }
        }
    }
}

#[test]
fn 実行の判断と図の辺は_すべてのジョブとすべての結果で一致する() {
    let ids = ["a", "b", "c", "d", "e"];
    let patterns: Vec<Vec<Spec<'_>>> = vec![
        vec![],
        vec![("a", OnSuccess, Some("d"))],
        vec![("a", OnWarning, Some("c"))],
        vec![("a", OnSuccess, Some("e")), ("a", OnWarning, Some("e"))],
        vec![("a", OnSuccess, None), ("b", OnWarning, Some("d"))],
        vec![
            ("a", OnSuccess, Some("e")),
            ("b", OnSuccess, Some("d")),
            ("c", OnWarning, None),
        ],
        vec![("e", OnSuccess, None)],
        vec![("d", OnSuccess, Some("e")), ("d", OnWarning, None)],
    ];
    for transitions in &patterns {
        assert_same_as_execution(&definition(&ids, transitions));
    }
}
