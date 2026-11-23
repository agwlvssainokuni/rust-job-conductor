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

//! 図の重ね合わせと辺の取り出しの性能(types-perf-visible の 2b、2c。BR2.2、BR2.3、BR2.6)。
//!
//! ジョブ 1,000 個で、同じ処理を 100 回くり返す時間の、5 回の中央値が上限の中にあることを確かめる。
//! 上限は、直した後の見積もりの約 5 倍の余裕のある値で、大きな退行だけを止める(D6)。直す前の形に
//! 戻ったことは、このテストでは検知できない(直す前と後の時間は、コード生成の記録に残す)。

mod common;

use std::time::{Duration, Instant};

use common::median;
use conductor_core::graph::{
    EdgeKind, EdgeTarget, NodeState, StateOverlay, build_graph, flow_edges,
};
use conductor_core::{PipelineDefinition, parse_definition};

const N: usize = 1000;
const REPEAT: usize = 100;
const SAMPLES: usize = 5;
/// P3 の上限(debug。直した後の見積もり約 71 ms の約 5 倍。D6)。
const LIMIT_OVERLAY: Duration = Duration::from_millis(350);
/// P4 の上限(debug。直した後の見積もり約 118 ms の約 5 倍。D6)。
const LIMIT_EDGES: Duration = Duration::from_millis(600);

/// 同じ処理を `REPEAT` 回くり返す時間を `SAMPLES` 回測り、中央値を返す。
fn measure(label: &str, mut f: impl FnMut()) -> Duration {
    let samples: Vec<Duration> = (0..SAMPLES)
        .map(|_| {
            let start = Instant::now();
            for _ in 0..REPEAT {
                f();
            }
            start.elapsed()
        })
        .collect();
    let m = median(samples.clone());
    eprintln!("{label}: {REPEAT} 回の中央値 {m:?}(各回 {samples:?})");
    m
}

/// 遷移のないジョブ `j0`〜`j999` の定義。
fn plain_definition() -> PipelineDefinition {
    let mut text = String::from("format_version = 1\n[pipeline]\nname = \"p\"\n");
    for i in 0..N {
        text.push_str(&format!("[[job]]\nid = \"j{i}\"\ncommand = \"echo {i}\"\n"));
    }
    parse_definition(&text, "t.toml", None).expect("正しい定義")
}

/// 位置 `i` のジョブの重ね合わせの状態(成功・スキップ・エラーを順にくり返す)。
fn state_of(i: usize) -> NodeState {
    match i % 3 {
        0 => NodeState::Ok,
        1 => NodeState::Skipped,
        _ => NodeState::Error,
    }
}

#[test]
fn p3_千個のジョブの重ね合わせを_100_回くり返しても_350ms_以内() {
    let def = plain_definition();
    // `j999` は重ね合わせに無い(未実行として扱う)。
    let states: Vec<(String, NodeState)> =
        (0..N - 1).map(|i| (format!("j{i}"), state_of(i))).collect();
    let forward = StateOverlay {
        states: states.clone(),
        resume_job_id: Some("j999".into()),
    };
    let mut reversed_states = states;
    reversed_states.reverse();
    let reversed = StateOverlay {
        states: reversed_states,
        resume_job_id: Some("j999".into()),
    };

    // 結果の中身: 定義の順でも逆順でも、全ジョブの状態は重ね合わせのとおり。無いジョブは未実行。
    for overlay in [&forward, &reversed] {
        let graph = build_graph(&def, Some(overlay));
        assert_eq!(graph.nodes.len(), N + 1);
        for (i, node) in graph.nodes[..N - 1].iter().enumerate() {
            assert_eq!(node.state, Some(state_of(i)), "j{i}");
        }
        assert_eq!(graph.nodes[N - 1].state, Some(NodeState::Pending));
        assert!(graph.nodes[N - 1].is_resume);
        assert!(graph.nodes[N].is_end() && graph.nodes[N].state.is_none());
    }
    // 同じジョブ ID が 2 つある重ね合わせでは、最初のものが使われる。
    let duplicated = StateOverlay {
        states: vec![
            ("j0".into(), NodeState::Error),
            ("j0".into(), NodeState::Ok),
        ],
        resume_job_id: None,
    };
    let graph = build_graph(&def, Some(&duplicated));
    assert_eq!(graph.nodes[0].state, Some(NodeState::Error));
    assert_eq!(graph.nodes[1].state, Some(NodeState::Pending));

    let m = measure("P3", || {
        let graph = build_graph(&def, Some(&forward));
        assert_eq!(graph.nodes.len(), N + 1);
    });
    assert!(
        m < LIMIT_OVERLAY,
        "中央値 {m:?} が上限 {LIMIT_OVERLAY:?} を超えた"
    );
}

#[test]
fn p4_千個のジョブの辺の取り出しを_100_回くり返しても_600ms_以内() {
    // `j0`〜`j997` は、成功で次のジョブへ、警告で 2 つ先へ遷移を書く(遷移 1,996 個)。
    let mut text = String::from("format_version = 1\n[pipeline]\nname = \"p\"\n");
    for i in 0..N {
        text.push_str(&format!("[[job]]\nid = \"j{i}\"\ncommand = \"echo {i}\"\n"));
        if i + 2 < N {
            text.push_str(&format!(
                "[[job.transitions]]\non = \"success\"\nnext = \"j{}\"\n[[job.transitions]]\non = \"warning\"\nnext = \"j{}\"\n",
                i + 1,
                i + 2
            ));
        }
    }
    let def = parse_definition(&text, "t.toml", None).expect("正しい定義");

    // 結果の中身: 辺は 1,998 本(`j0`〜`j997` は 2 本ずつ、`j998` と `j999` は 1 本ずつ)。
    let edges = flow_edges(&def);
    assert_eq!(edges.len(), 1998);
    for i in 0..N - 2 {
        let success = &edges[2 * i];
        let warning = &edges[2 * i + 1];
        assert_eq!(success.from_job, format!("j{i}"));
        assert_eq!(success.to, EdgeTarget::Job(format!("j{}", i + 1)));
        assert_eq!(success.kinds, [EdgeKind::Success]);
        assert!(success.explicit);
        assert_eq!(warning.from_job, format!("j{i}"));
        assert_eq!(warning.to, EdgeTarget::Job(format!("j{}", i + 2)));
        assert_eq!(warning.kinds, [EdgeKind::Warning]);
        assert!(warning.explicit);
    }
    let j998 = &edges[2 * (N - 2)];
    assert_eq!(j998.from_job, format!("j{}", N - 2));
    assert_eq!(j998.to, EdgeTarget::Job(format!("j{}", N - 1)));
    assert_eq!(j998.kinds, [EdgeKind::Success, EdgeKind::Warning]);
    assert!(!j998.explicit);
    let j999 = &edges[2 * (N - 2) + 1];
    assert_eq!(j999.from_job, format!("j{}", N - 1));
    assert_eq!(j999.to, EdgeTarget::End);
    assert!(!j999.explicit);

    let m = measure("P4", || {
        assert_eq!(flow_edges(&def).len(), 1998);
    });
    assert!(
        m < LIMIT_EDGES,
        "中央値 {m:?} が上限 {LIMIT_EDGES:?} を超えた"
    );
}
