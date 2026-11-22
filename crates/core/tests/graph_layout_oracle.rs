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
//! 同じ行き先を含む形(`fixtures/graph_layout_oracle_same_target.md`。1,517 個)と、
//! 分岐を含む形(`fixtures/graph_layout_oracle_branch.md`。871 個。規則から導いた図で、git の出力ではない)と、
//! 飛び越しが交差する形(`fixtures/graph_layout_oracle_cross.md`。940 個。ジョブ 3〜7 個。git の出力)と、
//! 交差と同じ行き先が混ざる形(`fixtures/graph_layout_oracle_mixed.md`。487 個。ジョブ 4〜6 個。git の出力)。

mod graph_common;

use conductor_core::TransitionOn;
use conductor_core::graph::{build_graph, is_lane_drawable};
use graph_common::{definition, lanes_text, letters};

const ORACLE: &str = include_str!("fixtures/graph_layout_oracle.md");
const ORACLE_SAME_TARGET: &str = include_str!("fixtures/graph_layout_oracle_same_target.md");
const ORACLE_BRANCH: &str = include_str!("fixtures/graph_layout_oracle_branch.md");
const ORACLE_CROSS: &str = include_str!("fixtures/graph_layout_oracle_cross.md");
const ORACLE_MIXED: &str = include_str!("fixtures/graph_layout_oracle_mixed.md");

struct Example {
    heading: String,
    ids: Vec<String>,
    /// `(元のジョブ、行き先、分岐の遠いほうか)`。分岐(`b>d,f`)は、近いほうに成功、遠いほうに警告を割り当てる。
    skips: Vec<(String, Option<String>, bool)>,
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
        let to_target = |to: &str| (to != "END").then(|| to.to_string());
        let skips: Vec<(String, Option<String>, bool)> = if skips == "-" {
            Vec::new()
        } else {
            skips
                .split(' ')
                .flat_map(|s| {
                    let (from, to) = s.split_once('>').expect("飛び越しの形");
                    match to.split_once(',') {
                        Some((near, far)) => vec![
                            (from.to_string(), to_target(near), false),
                            (from.to_string(), to_target(far), true),
                        ],
                        None => vec![(from.to_string(), to_target(to), false)],
                    }
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
        .map(|(from, to, far)| {
            let on = if *far {
                TransitionOn::Warning
            } else {
                TransitionOn::Success
            };
            (from.as_str(), on, to.as_deref())
        })
        .collect();
    definition(&ids, &transitions)
}

fn all() -> Vec<Example> {
    let mut examples = parse(ORACLE);
    examples.extend(parse(ORACLE_SAME_TARGET));
    examples.extend(parse(ORACLE_BRANCH));
    examples.extend(parse(ORACLE_CROSS));
    examples.extend(parse(ORACLE_MIXED));
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
fn 分岐の正解データの例の数は_871_個() {
    assert_eq!(parse(ORACLE_BRANCH).len(), 871);
}

#[test]
fn 交差の正解データの例の数は_940_個() {
    assert_eq!(parse(ORACLE_CROSS).len(), 940);
}

#[test]
fn 交差と同じ行き先が混ざる正解データの例の数は_487_個() {
    assert_eq!(parse(ORACLE_MIXED).len(), 487);
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
