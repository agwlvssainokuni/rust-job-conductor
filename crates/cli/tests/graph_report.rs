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

//! `graph` の整形(text、辺の一覧、Mermaid。BR2.x、BR3.x、BR4.x、BR5.x)。functional-spec の例 G1〜G4、G6、E1、E2。

use conductor_cli::Language;
use conductor_cli::report::graph_mermaid::{escape_label, render_mermaid};
use conductor_cli::report::graph_text::render_text;
use conductor_core::graph::{
    NodeState, StateOverlay, build_graph, layout_graph, overlay_from_state,
};
use conductor_core::{PipelineDefinition, parse_definition};

/// `skips` は `(元、行き先)`。行き先 `"@end"` は終点。元の成功が、その行き先へ飛ぶ。
fn definition(ids: &[&str], skips: &[(&str, &str)], extra: &str) -> PipelineDefinition {
    let mut text = String::from("format_version = 1\n\n[pipeline]\nname = \"p\"\n");
    for id in ids {
        text.push_str(&format!(
            "\n[[job]]\nid = \"{id}\"\ncommand = \"echo {id}\"\n"
        ));
        if let Some((_, to)) = skips.iter().find(|(from, _)| from == id) {
            text.push_str(&format!(
                "\n[[job.transitions]]\non = \"success\"\nnext = \"{to}\"\n"
            ));
        }
        if id == &ids[0] {
            text.push_str(extra);
        }
    }
    parse_definition(&text, "c.toml", None).expect("正しい定義")
}

fn text_of(
    lang: Language,
    def: &PipelineDefinition,
    overlay: Option<&StateOverlay>,
    verbose: bool,
) -> String {
    let graph = build_graph(def, overlay);
    let layout = layout_graph(&graph);
    render_text(lang, def, &graph, &layout, verbose)
}

const A_TO_D: &[(&str, &str)] = &[("a", "d")];
const IDS5: [&str; 5] = ["a", "b", "c", "d", "e"];

#[test]
fn レーンの図の例_g1_から_g4_と_g6() {
    let g1 = definition(&["a", "b", "c"], &[], "");
    assert_eq!(
        text_of(Language::Ja, &g1, None, false),
        "* a\n* b\n* c\no @end\n"
    );
    let g2 = definition(&IDS5, A_TO_D, "");
    assert_eq!(
        text_of(Language::Ja, &g2, None, false),
        "*   a\n|\\\n* | b\n* | c\n|/\n* d\n* e\no @end\n"
    );
    let g3 = definition(&["a", "b", "c"], &[("a", "@end")], "");
    assert_eq!(
        text_of(Language::Ja, &g3, None, false),
        "*   a\n|\\\n* | b\n* | c\n|/\no @end\n"
    );
    let g4 = definition(
        &["a", "b", "c", "d", "e", "f"],
        &[("a", "f"), ("b", "e")],
        "",
    );
    assert_eq!(
        text_of(Language::Ja, &g4, None, false),
        "*   a\n|\\\n* \\   b\n|\\ \\\n* | | c\n* | | d\n|/ /\n* / e\n|/\n* f\no @end\n"
    );
    let g6 = definition(
        &["a", "b", "c", "d", "e", "f", "g"],
        &[("a", "c"), ("d", "f")],
        "",
    );
    assert_eq!(
        text_of(Language::Ja, &g6, None, false),
        "*   a\n|\\\n* | b\n|/\n* c\n*   d\n|\\\n* | e\n|/\n* f\n* g\no @end\n"
    );
}

#[test]
fn 辺の一覧の例_e1_交差と_e2_状態つき() {
    let e1 = definition(
        &["a", "b", "c", "d", "e", "f"],
        &[("a", "d"), ("b", "e")],
        "",
    );
    assert_eq!(
        text_of(Language::Ja, &e1, None, false),
        "注記: 飛び越しが交差するなどの形のため、レーンの図の代わりに、辺の一覧を出します。\n\
         ジョブ:\n  a\n  b\n  c\n  d\n  e\n  f\n\
         辺:\n  a -> b (警告)\n  a -> d (成功)\n  b -> c (警告)\n  b -> e (成功)\n  c -> d\n  d -> e\n  e -> f\n  f -> @end\n"
    );
    // E2: a の成功と警告が、どちらも d へ飛ぶ。
    let e2 = definition(
        &["a", "b", "c", "d"],
        &[],
        "\n[[job.transitions]]\non = \"warning\"\nnext = \"d\"\n\n[[job.transitions]]\non = \"success\"\nnext = \"d\"\n",
    );
    let overlay = StateOverlay {
        states: vec![
            ("a".into(), NodeState::Ok),
            ("b".into(), NodeState::Skipped),
            ("c".into(), NodeState::Skipped),
            ("d".into(), NodeState::Ok),
        ],
        resume_job_id: None,
    };
    assert_eq!(
        text_of(Language::Ja, &e2, Some(&overlay), false),
        "注記: 飛び越しが交差するなどの形のため、レーンの図の代わりに、辺の一覧を出します。\n\
         ジョブ:\n  [OK] a\n  [--] b\n  [--] c\n  [OK] d\n\
         辺:\n  a -> d (成功・警告)\n  b -> c\n  c -> d\n  d -> @end\n"
    );
}

fn g2_overlay(resume: Option<&str>) -> StateOverlay {
    StateOverlay {
        states: vec![
            ("a".into(), NodeState::Ok),
            ("b".into(), NodeState::Error),
            ("c".into(), NodeState::Pending),
            ("d".into(), NodeState::Pending),
            ("e".into(), NodeState::Pending),
        ],
        resume_job_id: resume.map(str::to_string),
    }
}

#[test]
fn state_の重ね合わせは_状態の記号と再開位置を行に添える() {
    let g2 = definition(&IDS5, A_TO_D, "");
    assert_eq!(
        text_of(Language::Ja, &g2, Some(&g2_overlay(Some("b"))), false),
        "*   [OK] a\n|\\\n* | [NG] b  <- 再開位置\n* | [  ] c\n|/\n* [  ] d\n* [  ] e\no @end\n"
    );
    // 英語の注記。
    assert!(
        text_of(Language::En, &g2, Some(&g2_overlay(Some("b"))), false)
            .contains("[NG] b  <- resume here\n")
    );
}

#[test]
fn 完了したステートは_状態の記号があり_再開位置は示さない() {
    let g2 = definition(&IDS5, A_TO_D, "");
    let completed = conductor_core::PipelineState {
        definition_key: "k".into(),
        pipeline_name: "p".into(),
        definition_path: "c.toml".into(),
        job_order: IDS5.iter().map(|s| s.to_string()).collect(),
        job_states: [
            ("a", conductor_core::JobStatus::Success),
            ("b", conductor_core::JobStatus::Skipped),
            ("c", conductor_core::JobStatus::Skipped),
            ("d", conductor_core::JobStatus::Success),
            ("e", conductor_core::JobStatus::Success),
        ]
        .iter()
        .map(|(id, status)| conductor_core::JobState {
            job_id: id.to_string(),
            status: *status,
            exit_code: None,
            skip_reason: None,
        })
        .collect(),
        resume_job_id: Some("e".into()),
        end_reached: false,
        completed: true,
        run_id: None,
    };
    assert_eq!(
        text_of(
            Language::Ja,
            &g2,
            Some(&overlay_from_state(&completed)),
            false
        ),
        "*   [OK] a\n|\\\n* | [--] b\n* | [--] c\n|/\n* [OK] d\n* [OK] e\no @end\n"
    );
}

#[test]
fn v_の詳細は_g2_と_g4_の例のとおり_分岐の行のあとに置く() {
    let g2 = definition(&IDS5, A_TO_D, "");
    assert_eq!(
        text_of(Language::Ja, &g2, None, true),
        "*   a\n|\\\n| | コマンド: echo a\n| | 成功 → d(b、c を飛び越す)、警告 → b\n\
         * | b\n| | コマンド: echo b\n* | c\n| | コマンド: echo c\n|/\n\
         * d\n| コマンド: echo d\n* e\n| コマンド: echo e\no @end\n"
    );
    let g4 = definition(
        &["a", "b", "c", "d", "e", "f"],
        &[("a", "f"), ("b", "e")],
        "",
    );
    assert_eq!(
        text_of(Language::Ja, &g4, None, true),
        "*   a\n|\\\n| | コマンド: echo a\n| | 成功 → f(b、c、d、e を飛び越す)、警告 → b\n\
         * \\   b\n|\\ \\\n| | | コマンド: echo b\n| | | 成功 → e(c、d を飛び越す)、警告 → c\n\
         * | | c\n| | | コマンド: echo c\n* | | d\n| | | コマンド: echo d\n\
         |/ /\n* / e\n| | コマンド: echo e\n|/\n* f\n| コマンド: echo f\no @end\n"
    );
}

#[test]
fn v_は_リトライと配列のコマンドと英語の文を出し_辺の一覧にも詳細を足す() {
    let text = "format_version = 1\n\n[pipeline]\nname = \"p\"\n\n[[job]]\nid = \"a\"\n\
        command = [\"echo\", \"hello world\", \"\", \"x\\ty\"]\n\n[job.retry]\n\
        exit_codes = [75, 76]\nmax = 3\ninterval = \"5s\"\nmultiplier = 2\nmax_interval = \"60s\"\n\n\
        [[job]]\nid = \"b\"\ncommand = \"echo b\"\n";
    let def = parse_definition(text, "c.toml", None).unwrap();
    let en = text_of(Language::En, &def, None, true);
    assert_eq!(
        en,
        "* a\n| command: echo \"hello world\" \"\" \"x\\ty\"\n\
         | retry: exit codes 75, 76, up to 3 times, every 5 s (x2, max 60 s)\n\
         * b\n| command: echo b\no @end\n"
    );
    let ja = text_of(Language::Ja, &def, None, true);
    assert!(
        ja.contains("| リトライ: 終了コード 75、76 を最大 3 回、間隔 5 秒(倍率 2、上限 60 秒)\n"),
        "{ja}"
    );

    // 辺の一覧(交差)の `-v` は、ジョブの行の下に、4 つの空白で字下げした詳細。
    let crossing = definition(&["a", "b", "c", "d", "e"], &[("a", "d"), ("b", "e")], "");
    let list = text_of(Language::En, &crossing, None, true);
    assert!(
        list.contains(
            "Jobs:\n  a\n    command: echo a\n    success -> d (skips b, c), warning -> b\n  b\n"
        ),
        "{list}"
    );
    assert!(list.contains("Edges:\n  a -> b (warning)\n"), "{list}");
}

#[test]
fn mermaid_の例と_state_の再開位置() {
    let g2 = definition(&IDS5, A_TO_D, "");
    let plain = render_mermaid(Language::Ja, &build_graph(&g2, None));
    assert_eq!(
        plain,
        "flowchart TD\n  n0[\"a\"]\n  n1[\"b\"]\n  n2[\"c\"]\n  n3[\"d\"]\n  n4[\"e\"]\n  nend([\"@end\"])\n\
         \x20 n0 -->|\"警告\"| n1\n  n0 -->|\"成功\"| n3\n  n1 --> n2\n  n2 --> n3\n  n3 --> n4\n  n4 --> nend\n"
    );
    let overlay = StateOverlay {
        states: vec![
            ("a".into(), NodeState::Ok),
            ("b".into(), NodeState::Error),
            ("c".into(), NodeState::Pending),
            ("d".into(), NodeState::Pending),
            ("e".into(), NodeState::Pending),
        ],
        resume_job_id: Some("b".into()),
    };
    let with_state = render_mermaid(Language::Ja, &build_graph(&g2, Some(&overlay)));
    assert!(with_state.contains("  n0[\"[OK] a\"]\n  n1[\"[NG] b\"]\n  n2[\"[  ] c\"]\n"));
    assert!(with_state.ends_with("  classDef resume stroke-width:3px\n  class n1 resume\n"));
    // 英語のラベル。
    let en = render_mermaid(Language::En, &build_graph(&g2, None));
    assert!(en.contains("n0 -->|\"warning\"| n1\n  n0 -->|\"success\"| n3\n"));
}

#[test]
fn mermaid_のラベルは_特別な文字をエスケープし_idを壊さない() {
    assert_eq!(
        escape_label("a<b>&c#d\"e`f\u{1}"),
        "a#lt;b#gt;#amp;c#35;d#quot;e#96;f#1;"
    );
    // `#` を最初に置き換える(置き換えで作った `#` を、さらに置き換えない)。
    assert_eq!(escape_label("#\""), "#35;#quot;");
    let def = parse_definition(
        "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"x<y>&z#w\\\"v`u\"\ncommand = \"echo\"\n",
        "c.toml",
        None,
    )
    .unwrap();
    let out = render_mermaid(Language::En, &build_graph(&def, None));
    assert!(
        out.contains("n0[\"x#lt;y#gt;#amp;z#35;w#quot;v#96;u\"]"),
        "{out}"
    );
    // 日本語のジョブ ID と予約語は、そのまま、引用符の中に書く。
    let def = parse_definition(
        "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"end\"\ncommand = \"echo\"\n[[job]]\nid = \"取得\"\ncommand = \"echo\"\n",
        "c.toml",
        None,
    )
    .unwrap();
    let out = render_mermaid(Language::En, &build_graph(&def, None));
    assert!(out.contains("n0[\"end\"]\n  n1[\"取得\"]\n"), "{out}");
}
