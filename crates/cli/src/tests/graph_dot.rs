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

//! `graph --format dot` の整形(U4。BR2.1〜BR3.2、NFR1.2〜NFR1.5、NFR6.1)。
//!
//! escape の入力と出力の対の表、各行の形、`--state` の記号と太い枠、辺のラベル、Mermaid との数の一致、
//! 1 万のジョブでの描画の時間の順。

use std::time::{Duration, Instant};

use crate::report::graph_dot::{escape_dot_label, render_dot};
use crate::report::graph_mermaid::render_mermaid;
use crate::types::Language;
use conductor_core::graph::{
    EdgeKind, EdgeTarget, FlowEdge, FlowGraph, GraphNode, NodeKind, NodeState, StateOverlay,
    build_graph,
};
use conductor_core::{PipelineDefinition, parse_definition};

/// ジョブを並べた定義。`first_extra` は最初のジョブの後ろに足す(遷移など)。ジョブ ID は TOML の
/// 基本文字列の中身として書く(`\"`、`\\`、`\n` の escape をそのまま使える)。
fn definition(ids: &[&str], first_extra: &str) -> PipelineDefinition {
    let mut text = String::from("format_version = 1\n\n[pipeline]\nname = \"p\"\n");
    for (i, id) in ids.iter().enumerate() {
        text.push_str(&format!(
            "\n[[job]]\nid = \"{id}\"\ncommand = \"echo {i}\"\n"
        ));
        if i == 0 {
            text.push_str(first_extra);
        }
    }
    parse_definition(&text, "c.toml", None).expect("正しい定義")
}

/// 最初のジョブの遷移(`on` の結果で `next` へ)。
fn transition(on: &str, next: &str) -> String {
    format!("\n[[job.transitions]]\non = \"{on}\"\nnext = \"{next}\"\n")
}

const IDS5: [&str; 5] = ["a", "b", "c", "d", "e"];

/// a の成功が d へ飛ぶ 5 個のジョブ(`graph_report.rs` の G2 と同じ形)。
fn g2() -> PipelineDefinition {
    definition(&IDS5, &transition("success", "d"))
}

fn g2_overlay() -> StateOverlay {
    StateOverlay {
        states: vec![
            ("a".into(), NodeState::Ok),
            ("b".into(), NodeState::Error),
            ("c".into(), NodeState::Pending),
            ("d".into(), NodeState::Skipped),
            ("e".into(), NodeState::Pending),
        ],
        resume_job_id: Some("b".into()),
    }
}

/// 二重引用符の文字列の中身を読み、閉じる `"` の後ろを返す。中の `\` の後ろは `\` か `"` だけを許す
/// (escape の済んだ形)。打ち消されていない `"` は、文字列の終わり。
fn quoted(rest: &str) -> Option<(&str, &str)> {
    let mut chars = rest.char_indices();
    while let Some((i, c)) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some((_, '\\' | '"')) => {}
                _ => return None,
            },
            '"' => return Some((&rest[..i], &rest[i + 1..])),
            c if c.is_control() => return None,
            _ => {}
        }
    }
    None
}

/// `n<数>` か `nend` で始まるなら、その後ろ。
fn node_id(text: &str) -> Option<&str> {
    if let Some(rest) = text.strip_prefix("nend") {
        return Some(rest);
    }
    let rest = text.strip_prefix('n')?;
    let digits = rest.chars().take_while(char::is_ascii_digit).count();
    (digits > 0).then(|| &rest[digits..])
}

/// 節か辺の 1 行が、決まった形に一致するか。
fn is_valid_body_line(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("  ").and_then(node_id) else {
        return false;
    };
    // 節: `  n<数> [label="...", shape=box(, penwidth=3)?];`、`  nend [label="...", shape=oval];`
    if let Some(label) = rest.strip_prefix(" [label=\"") {
        let Some((_, after)) = quoted(label) else {
            return false;
        };
        return matches!(
            after,
            ", shape=box];" | ", shape=box, penwidth=3];" | ", shape=oval];"
        );
    }
    // 辺: `  n<数> -> n<数>;`、`  n<数> -> n<数> [label="..."];`
    let Some(to) = rest.strip_prefix(" -> ").and_then(node_id) else {
        return false;
    };
    if to == ";" {
        return true;
    }
    let Some(label) = to.strip_prefix(" [label=\"") else {
        return false;
    };
    matches!(quoted(label), Some((_, "];")))
}

/// 文書全体が DOT の決まった形か(最初の 2 行、各行、最後の `}` と改行)。
fn assert_valid_dot(dot: &str) {
    assert!(dot.ends_with("}\n"), "{dot}");
    let lines: Vec<&str> = dot.lines().collect();
    assert_eq!(lines[0], "digraph conductor {");
    assert_eq!(lines[1], "  rankdir=TB;");
    assert_eq!(lines[lines.len() - 1], "}");
    for line in &lines[2..lines.len() - 1] {
        assert!(is_valid_body_line(line), "形が違う行: {line:?}\n{dot}");
    }
}

/// 節(終点を含む)と辺の行の数。
fn counts(dot: &str) -> (usize, usize) {
    let nodes = dot
        .lines()
        .filter(|l| l.contains(" [label=\"") && l.contains("shape="))
        .count();
    let edges = dot.lines().filter(|l| l.contains(" -> ")).count();
    (nodes, edges)
}

/// ジョブ `n<i>` のラベル(DOT)。
fn dot_label(dot: &str, index: usize) -> String {
    let head = format!("  n{index} [label=\"");
    let line = dot
        .lines()
        .find(|l| l.starts_with(&head))
        .expect("節がある");
    quoted(&line[head.len()..])
        .expect("閉じている")
        .0
        .to_string()
}

/// ジョブ `n<i>` のラベル(Mermaid)。
fn mermaid_label(mermaid: &str, index: usize) -> String {
    let head = format!("  n{index}[\"");
    let line = mermaid
        .lines()
        .find(|l| l.starts_with(&head))
        .expect("節がある");
    line[head.len()..line.len() - 2].to_string()
}

#[test]
fn escape_dot_label_の入力と出力の対の表() {
    let table: [(&str, &str); 8] = [
        ("a", "a"),
        ("a\"b", "a\\\"b"),
        ("a\\b", "a\\\\b"),
        ("a\nb", "a\\\\nb"),
        ("a\\lb", "a\\\\lb"),
        ("\t", "\\\\t"),
        ("\u{202E}", "\\\\u{202e}"),
        ("日本語", "日本語"),
    ];
    for (input, expected) in table {
        assert_eq!(escape_dot_label(input), expected, "{input:?}");
    }
}

#[test]
fn g2_の全文は_mermaid_と同じ節と辺を_dot_の記法で書く() {
    let dot = render_dot(Language::Ja, &build_graph(&g2(), None));
    assert_eq!(
        dot,
        "digraph conductor {\n  rankdir=TB;\n\
         \x20 n0 [label=\"a\", shape=box];\n  n1 [label=\"b\", shape=box];\n  n2 [label=\"c\", shape=box];\n\
         \x20 n3 [label=\"d\", shape=box];\n  n4 [label=\"e\", shape=box];\n  nend [label=\"@end\", shape=oval];\n\
         \x20 n0 -> n1 [label=\"警告\"];\n  n0 -> n3 [label=\"成功\"];\n  n1 -> n2;\n  n2 -> n3;\n  n3 -> n4;\n\
         \x20 n4 -> nend;\n}\n"
    );
    assert_valid_dot(&dot);
}

#[test]
fn 主線の辺はラベルなしで_成功だけ_警告だけ_両方の辺は語のラベル_英日() {
    let en = render_dot(Language::En, &build_graph(&g2(), None));
    assert!(
        en.contains(
            "  n0 -> n1 [label=\"warning\"];\n  n0 -> n3 [label=\"success\"];\n  n1 -> n2;\n"
        ),
        "{en}"
    );
    // a の成功も警告も c へ(両方の飛び越し)。
    let both = definition(
        &["a", "b", "c"],
        &format!(
            "{}{}",
            transition("success", "c"),
            transition("warning", "c")
        ),
    );
    let ja = render_dot(Language::Ja, &build_graph(&both, None));
    assert!(ja.contains("  n0 -> n2 [label=\"成功・警告\"];\n"), "{ja}");
    let en = render_dot(Language::En, &build_graph(&both, None));
    assert!(
        en.contains("  n0 -> n2 [label=\"success, warning\"];\n"),
        "{en}"
    );
    // 主線(b から c)はラベルなし。
    assert!(ja.contains("  n1 -> n2;\n"), "{ja}");
}

#[test]
fn state_のときは_mermaid_と同じ記号で_再開位置だけが太い枠() {
    let graph = build_graph(&g2(), Some(&g2_overlay()));
    let dot = render_dot(Language::Ja, &graph);
    let mermaid = render_mermaid(Language::Ja, &graph);
    assert!(
        dot.contains(
            "  n0 [label=\"[OK] a\", shape=box];\n  n1 [label=\"[NG] b\", shape=box, penwidth=3];\n\
             \x20 n2 [label=\"[  ] c\", shape=box];\n  n3 [label=\"[--] d\", shape=box];\n"
        ),
        "{dot}"
    );
    for index in 0..IDS5.len() {
        assert_eq!(dot_label(&dot, index), mermaid_label(&mermaid, index));
    }
    // 太い枠は、Mermaid が `class ... resume` を付ける節だけ。
    assert_eq!(dot.matches("penwidth=3").count(), 1);
    assert!(mermaid.contains("  class n1 resume\n"));
    assert_valid_dot(&dot);
}

#[test]
fn state_でなければ_記号も太い枠も無い() {
    let dot = render_dot(Language::Ja, &build_graph(&g2(), None));
    for symbol in ["[OK]", "[NG]", "[--]", "[  ]"] {
        assert!(!dot.contains(symbol), "{symbol}: {dot}");
    }
    assert!(!dot.contains("penwidth"));
    // 再開位置の無い(完了した)重ね合わせでも、太い枠は無い。
    let mut overlay = g2_overlay();
    overlay.resume_job_id = None;
    let dot = render_dot(Language::Ja, &build_graph(&g2(), Some(&overlay)));
    assert!(dot.contains("[OK] a"));
    assert!(!dot.contains("penwidth"));
}

#[test]
fn 引用符と_バックスラッシュと_改行を含むジョブ_id_でも_各行が決まった形になる() {
    // TOML の中身として書く: `q"1`、`b\s`、`l\ln`(バックスラッシュと l)、改行を含む `x<改行>y`。
    let ids = ["q\\\"1", "b\\\\s", "l\\\\ln", "x\\ny"];
    let def = definition(&ids, &transition("warning", "x\\ny"));
    let graph = build_graph(&def, None);
    let dot = render_dot(Language::Ja, &graph);
    assert_valid_dot(&dot);
    assert!(
        dot.contains("  n0 [label=\"q\\\"1\", shape=box];\n"),
        "{dot}"
    );
    assert!(
        dot.contains("  n1 [label=\"b\\\\s\", shape=box];\n"),
        "{dot}"
    );
    assert!(
        dot.contains("  n2 [label=\"l\\\\ln\", shape=box];\n"),
        "{dot}"
    );
    assert!(
        dot.contains("  n3 [label=\"x\\\\ny\", shape=box];\n"),
        "{dot}"
    );
    // 行の数は、節(終点を含む)+ 辺 + 3。
    assert_eq!(
        dot.lines().count(),
        graph.nodes.len() + graph.edges.len() + 3
    );
}

#[test]
fn 分岐と飛び越しと終点を含む定義で_ジョブの節と辺の数が_mermaid_と同じ() {
    let cases = [
        // 終点への飛び越し。
        definition(&["a", "b", "c"], &transition("success", "@end")),
        // 分岐(成功は c、警告は e)。
        definition(
            &IDS5,
            &format!(
                "{}{}",
                transition("success", "c"),
                transition("warning", "e")
            ),
        ),
        // 両方の飛び越し。
        definition(
            &IDS5,
            &format!(
                "{}{}",
                transition("success", "d"),
                transition("warning", "d")
            ),
        ),
        g2(),
    ];
    for def in &cases {
        let graph = build_graph(def, None);
        let dot = render_dot(Language::En, &graph);
        let mermaid = render_mermaid(Language::En, &graph);
        assert_valid_dot(&dot);
        let (nodes, edges) = counts(&dot);
        let mermaid_jobs = mermaid
            .lines()
            .filter(|l| {
                l.starts_with("  n")
                    && !l.starts_with("  nend")
                    && l.contains("[\"")
                    && !l.contains("-->")
            })
            .count();
        let mermaid_edges = mermaid.lines().filter(|l| l.contains("-->")).count();
        // DOT の節の数は、終点を含む。
        assert_eq!(nodes, mermaid_jobs + 1, "{dot}\n{mermaid}");
        assert_eq!(edges, mermaid_edges, "{dot}\n{mermaid}");
        assert_eq!(dot.lines().count(), nodes + edges + 3);
    }
}

#[test]
fn 行き先の分からない辺は_mermaid_と同じく出さない() {
    let node = |id: &str| GraphNode {
        kind: NodeKind::Job(id.into()),
        state: None,
        is_resume: false,
    };
    let edge = |from: &str, to: EdgeTarget| FlowEdge {
        from_job: from.into(),
        to,
        kinds: vec![EdgeKind::Success],
        skipped_jobs: Vec::new(),
    };
    let graph = FlowGraph {
        nodes: vec![
            node("a"),
            GraphNode {
                kind: NodeKind::End,
                state: None,
                is_resume: false,
            },
        ],
        edges: vec![
            edge("a", EdgeTarget::Job("zz".into())),
            edge("zz", EdgeTarget::End),
            edge("a", EdgeTarget::End),
        ],
    };
    let dot = render_dot(Language::Ja, &graph);
    assert_eq!(
        dot,
        "digraph conductor {\n  rankdir=TB;\n  n0 [label=\"a\", shape=box];\n\
         \x20 nend [label=\"@end\", shape=oval];\n  n0 -> nend [label=\"成功\"];\n}\n"
    );
    let mermaid = render_mermaid(Language::Ja, &graph);
    assert_eq!(
        dot.lines().filter(|l| l.contains(" -> ")).count(),
        mermaid.lines().filter(|l| l.contains("-->")).count()
    );
}

/// 1 万のジョブが直線につながる図(成功と警告の両方で次のジョブへ)。
fn straight_graph(n: usize) -> FlowGraph {
    let mut nodes: Vec<GraphNode> = (0..n)
        .map(|i| GraphNode {
            kind: NodeKind::Job(format!("job{i}")),
            state: Some(NodeState::Pending),
            is_resume: i == n / 2,
        })
        .collect();
    nodes.push(GraphNode {
        kind: NodeKind::End,
        state: None,
        is_resume: false,
    });
    let edges = (0..n)
        .map(|i| FlowEdge {
            from_job: format!("job{i}"),
            to: if i + 1 < n {
                EdgeTarget::Job(format!("job{}", i + 1))
            } else {
                EdgeTarget::End
            },
            kinds: vec![EdgeKind::Success, EdgeKind::Warning],
            skipped_jobs: Vec::new(),
        })
        .collect();
    FlowGraph { nodes, edges }
}

/// 描画の時間の上限(NFR1.3)。手元の macOS のデバッグ版の実測は数十ミリ秒で、3 つの OS の CI の揺れに
/// 余裕を持たせる。下げない。
const RENDER_LIMIT: Duration = Duration::from_secs(1);

#[test]
fn 一万のジョブの直線の図も_時間の上限の内に描け_行の数が期待どおり() {
    const N: usize = 10_000;
    let graph = straight_graph(N);
    let start = Instant::now();
    let dot = render_dot(Language::Ja, &graph);
    let elapsed = start.elapsed();
    assert!(elapsed < RENDER_LIMIT, "{elapsed:?}");
    // 節(N + 終点)+ 辺(N)+ 3。
    assert_eq!(dot.lines().count(), (N + 1) + N + 3);
    assert_eq!(dot.matches("penwidth=3").count(), 1);
    assert!(dot.contains(&format!("  n{} -> nend;\n", N - 1)));
}
