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

//! 1,000 ジョブでの性能(BR2.7、NFR1)。最悪の 2 つの形で、1 秒以内。

mod graph_common;

use std::time::{Duration, Instant};

use conductor_core::TransitionOn::Success;
use conductor_core::graph::{GraphLayout, build_graph, layout_graph};
use graph_common::definition;

const N: usize = 1000;

fn ids() -> Vec<String> {
    (0..N).map(|i| format!("j{i}")).collect()
}

fn timed(def: &conductor_core::PipelineDefinition) -> (GraphLayout, Duration) {
    let start = Instant::now();
    let graph = build_graph(def, None);
    let layout = layout_graph(&graph);
    (layout, start.elapsed())
}

#[test]
fn 入れ子が最大の形は_1000_ジョブでも_1秒以内() {
    let ids = ids();
    let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
    // 前半の各ジョブ i が、対応する後半のジョブ(N-1-i)へ、入れ子に飛び越す。
    let transitions: Vec<_> = (0..N / 2 - 1)
        .map(|i| (refs[i], Success, Some(refs[N - 1 - i])))
        .collect();
    let (layout, elapsed) = timed(&definition(&refs, &transitions));
    match layout {
        GraphLayout::Lanes(l) => assert!(l.rows.len() > N),
        GraphLayout::EdgeList => panic!("入れ子は、レーンの図になる"),
    }
    assert!(elapsed < Duration::from_secs(1), "{elapsed:?}");
}

#[test]
fn 交差する形は_辺の一覧に切り替わり_1000_ジョブでも_1秒以内() {
    let ids = ids();
    let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
    // j(i) → j(i+2) を、すべての i で。隣の区間と交差する。
    let transitions: Vec<_> = (0..N - 2)
        .map(|i| (refs[i], Success, Some(refs[i + 2])))
        .collect();
    let (layout, elapsed) = timed(&definition(&refs, &transitions));
    assert_eq!(layout, GraphLayout::EdgeList);
    assert!(elapsed < Duration::from_secs(1), "{elapsed:?}");
}
