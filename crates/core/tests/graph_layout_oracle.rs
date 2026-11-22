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

//! レーンの配置の正解データとの一致(BR2.3、BR4.1)。
//!
//! 行き先がすべて違う形(`fixtures/graph_layout_oracle.md`。537 個)と、
//! 同じ行き先を含む形(`fixtures/graph_layout_oracle_same_target.md`。1,517 個)。

mod graph_common;

use conductor_core::TransitionOn;
use conductor_core::graph::{build_graph, is_lane_drawable};
use graph_common::{definition, lanes_text, letters};

const ORACLE: &str = include_str!("fixtures/graph_layout_oracle.md");
const ORACLE_SAME_TARGET: &str = include_str!("fixtures/graph_layout_oracle_same_target.md");

struct Example {
    heading: String,
    ids: Vec<String>,
    skips: Vec<(String, Option<String>)>,
    expected: String,
}

fn parse(source: &str) -> Vec<Example> {
    let mut examples = Vec::new();
    let mut lines = source.lines();
    while let Some(line) = lines.next() {
        let Some(heading) = line.strip_prefix("### ") else {
            continue;
        };
        // `jobs=<n> skips=<...>`
        let rest = heading.strip_prefix("jobs=").expect("見出しの形");
        let (count, skips) = rest.split_once(" skips=").expect("見出しの形");
        let count: usize = count.parse().expect("ジョブの数");
        let skips: Vec<(String, Option<String>)> = if skips == "-" {
            Vec::new()
        } else {
            skips
                .split(' ')
                .map(|s| {
                    let (from, to) = s.split_once('>').expect("飛び越しの形");
                    let to = if to == "END" {
                        None
                    } else {
                        Some(to.to_string())
                    };
                    (from.to_string(), to)
                })
                .collect()
        };
        assert_eq!(lines.next(), Some("```text"));
        let mut expected = String::new();
        for body in lines.by_ref() {
            if body == "```" {
                break;
            }
            expected.push_str(body);
            expected.push('\n');
        }
        examples.push(Example {
            heading: heading.to_string(),
            ids: letters(count),
            skips,
            expected,
        });
    }
    examples
}

fn build(example: &Example) -> conductor_core::PipelineDefinition {
    let ids: Vec<&str> = example.ids.iter().map(String::as_str).collect();
    let transitions: Vec<_> = example
        .skips
        .iter()
        .map(|(from, to)| (from.as_str(), TransitionOn::Success, to.as_deref()))
        .collect();
    definition(&ids, &transitions)
}

fn all() -> Vec<Example> {
    let mut examples = parse(ORACLE);
    examples.extend(parse(ORACLE_SAME_TARGET));
    examples
}

#[test]
fn 正解データの例の数は_537_個() {
    assert_eq!(parse(ORACLE).len(), 537);
}

#[test]
fn 同じ行き先の正解データの例の数は_1517_個() {
    assert_eq!(parse(ORACLE_SAME_TARGET).len(), 1517);
}

#[test]
fn 正解データのすべての形は_対応する形と判定される() {
    for example in all() {
        let graph = build_graph(&build(&example), None);
        assert!(is_lane_drawable(&graph), "{}", example.heading);
    }
}

#[test]
fn 正解データのすべての例に_1文字も違わず一致する() {
    let mut failures = Vec::new();
    for example in all() {
        let actual = lanes_text(&build(&example));
        if actual.as_deref() != Some(example.expected.as_str()) {
            failures.push(format!(
                "### {}\n--- 期待\n{}--- 実際\n{}",
                example.heading,
                example.expected,
                actual.unwrap_or_else(|| "(辺の一覧)\n".into())
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} 個が不一致\n{}",
        failures.len(),
        failures
            .iter()
            .take(5)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn 正解データの各行は_末尾に空白を持たない() {
    for example in all() {
        assert!(
            example.expected.lines().all(|l| l == l.trim_end()),
            "{}",
            example.heading
        );
    }
}
