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
    GraphLayout, NodeState, StateOverlay, build_graph, layout_graph, overlay_from_state,
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

/// 辺の一覧の整形(BR1.1 の範囲の外の形)。定義から作れる形は、すべてレーンの図になるので、
/// 整形の関数に、辺の一覧の配置を直接渡す。
fn edge_list_of(
    lang: Language,
    def: &PipelineDefinition,
    overlay: Option<&StateOverlay>,
    verbose: bool,
) -> String {
    let graph = build_graph(def, overlay);
    render_text(lang, def, &graph, &GraphLayout::EdgeList, verbose)
}

const A_TO_D: &[(&str, &str)] = &[("a", "d")];
const IDS5: [&str; 5] = ["a", "b", "c", "d", "e"];
/// 交差し、かつ「両方の飛び越し」(a が成功も警告も e へ)を含む形(前回までは辺の一覧、今回からレーンの図)。
const E1_IDS: [&str; 7] = ["a", "b", "c", "d", "e", "f", "g"];
const E1_SKIPS: [(&str, &str); 2] = [("b", "f"), ("c", "e")];

/// E1 の定義。a の成功と警告が、どちらも e へ飛ぶ。
fn e1_definition() -> PipelineDefinition {
    definition(&E1_IDS, &E1_SKIPS, &both_skip_extra("e"))
}

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
fn e1_の形は_レーンの図になる() {
    // 交差し、さらに「両方の飛び越し」(a が成功も警告も e へ)を含む形も、今回から図にする(BR2.1)。
    // a は次のジョブへの辺がなく、b、c、d は実行されないので、その下の行の列 0 は空白(BR1.4)。
    let e1 = e1_definition();
    assert_eq!(
        text_of(Language::Ja, &e1, None, false),
        "*   a\n \\\n* \\   b\n \\ \\\n* \\ \\   c\n \\ \\ \\\n  | |/\n  |/|\n* / | d\n / /\n* / e\n|/\n* f\n* g\no @end\n"
    );
}

#[test]
fn 辺の一覧の例_e1_交差と_状態つき() {
    // 辺の一覧の整形は、前回までのとおり(範囲の外の形で使う。BR1.1)。
    let e1 = e1_definition();
    assert_eq!(
        edge_list_of(Language::Ja, &e1, None, false),
        "注記: 飛び越しが交差するなどの形のため、レーンの図の代わりに、辺の一覧を出します。\n\
         ジョブ:\n  a\n  b\n  c\n  d\n  e\n  f\n  g\n\
         辺:\n  a -> e (成功・警告)\n  b -> c (警告)\n  b -> f (成功)\n  c -> d (警告)\n  c -> e (成功)\n  d -> e\n  e -> f\n  f -> g\n  g -> @end\n"
    );
    // E2: 辺の一覧の形に、状態を重ねる。
    let overlay = StateOverlay {
        states: ["a", "b", "c", "d", "e", "f", "g"]
            .iter()
            .map(|id| (id.to_string(), NodeState::Pending))
            .collect(),
        resume_job_id: None,
    };
    assert_eq!(
        edge_list_of(Language::Ja, &e1, Some(&overlay), false),
        "注記: 飛び越しが交差するなどの形のため、レーンの図の代わりに、辺の一覧を出します。\n\
         ジョブ:\n  [  ] a\n  [  ] b\n  [  ] c\n  [  ] d\n  [  ] e\n  [  ] f\n  [  ] g\n\
         辺:\n  a -> e (成功・警告)\n  b -> c (警告)\n  b -> f (成功)\n  c -> d (警告)\n  c -> e (成功)\n  d -> e\n  e -> f\n  f -> g\n  g -> @end\n"
    );
}

/// a の成功と警告が、どちらも `to` へ飛ぶ(両方の飛び越し)。
fn both_skip_extra(to: &str) -> String {
    format!(
        "\n[[job.transitions]]\non = \"warning\"\nnext = \"{to}\"\n\n[[job.transitions]]\non = \"success\"\nnext = \"{to}\"\n"
    )
}

#[test]
fn 同じ行き先の図_s1_と_両方の飛び越しの図_d1() {
    // S1: a→d と b→d が、同じ線に合流する。
    let s1 = definition(&IDS5, &[("a", "d"), ("b", "d")], "");
    assert_eq!(
        text_of(Language::Ja, &s1, None, false),
        "*   a\n|\\\n* | b\n|\\|\n* | c\n|/\n* d\n* e\no @end\n"
    );
    // D1: a の成功と警告が、どちらも c へ飛ぶ。通常の飛び越しと同じ線で、主線の線だけ省く。
    let d1 = definition(&["a", "b", "c", "d"], &[], &both_skip_extra("c"));
    assert_eq!(
        text_of(Language::Ja, &d1, None, false),
        "*   a\n \\\n* | b\n /\n* c\n* d\no @end\n"
    );
}

#[test]
fn 同じ行き先と両方の飛び越しに_state_を重ねても_行の形は変わらない() {
    let d1 = definition(&["a", "b", "c", "d"], &[], &both_skip_extra("c"));
    let overlay = StateOverlay {
        states: vec![
            ("a".into(), NodeState::Ok),
            ("b".into(), NodeState::Skipped),
            ("c".into(), NodeState::Ok),
            ("d".into(), NodeState::Pending),
        ],
        resume_job_id: Some("d".into()),
    };
    assert_eq!(
        text_of(Language::Ja, &d1, Some(&overlay), false),
        "*   [OK] a\n \\\n* | [--] b\n /\n* [OK] c\n* [  ] d  <- 再開位置\no @end\n"
    );
}

#[test]
fn v_の詳細は_合流の行のあとにも_分岐の行と同じように置く() {
    let s1 = definition(&IDS5, &[("a", "d"), ("b", "d")], "");
    assert_eq!(
        text_of(Language::En, &s1, None, true),
        "*   a\n|\\\n| | command: echo a\n| | success -> d (skips b, c), warning -> b\n\
         * | b\n|\\|\n| | command: echo b\n| | success -> d (skips c), warning -> c\n\
         * | c\n| | command: echo c\n|/\n\
         * d\n| command: echo d\n* e\n| command: echo e\no @end\n"
    );
    let d1 = definition(&["a", "b", "c", "d"], &[], &both_skip_extra("c"));
    assert_eq!(
        text_of(Language::En, &d1, None, true),
        "*   a\n \\\n  | command: echo a\n  | success, warning -> c (skips b)\n\
         * | b\n  | command: echo b\n /\n* c\n| command: echo c\n* d\n| command: echo d\no @end\n"
    );
}

/// a の成功が `near`、警告が `far` へ飛ぶ(分岐)。
fn branch_extra(near: &str, far: &str) -> String {
    format!(
        "\n[[job.transitions]]\non = \"success\"\nnext = \"{near}\"\n\n[[job.transitions]]\non = \"warning\"\nnext = \"{far}\"\n"
    )
}

const B1_TEXT: &str = "*     a\n \\ \\\n* | | b\n / /\n* / c\n|/\n* d\n* e\no @end\n";

#[test]
fn 分岐の図_b1_の全文() {
    // B1: a の成功が c、警告が d。2 本の線を同時に開く。a、b の下の行の列 0 は空白で、
    // 実行されうる c の下の合流の行は `|/`(BR1.4)。
    let b1 = definition(&IDS5, &[], &branch_extra("c", "d"));
    assert_eq!(text_of(Language::Ja, &b1, None, false), B1_TEXT);
    // 近い行き先が警告でも、同じ図になる。
    let swapped = definition(&IDS5, &[], &branch_extra("d", "c"));
    assert_eq!(text_of(Language::Ja, &swapped, None, false), B1_TEXT);
}

#[test]
fn 分岐の図に_v_の詳細と_state_を重ねる() {
    let b1 = definition(&IDS5, &[], &branch_extra("c", "d"));
    assert_eq!(
        text_of(Language::En, &b1, None, true),
        "*     a\n \\ \\\n  | | command: echo a\n  | | success -> c (skips b), warning -> d (skips b, c)\n\
         * | | b\n  | | command: echo b\n / /\n* / c\n| | command: echo c\n|/\n* d\n| command: echo d\n* e\n| command: echo e\no @end\n"
    );
    let overlay = StateOverlay {
        states: vec![
            ("a".into(), NodeState::Ok),
            ("b".into(), NodeState::Skipped),
            ("c".into(), NodeState::Ok),
            ("d".into(), NodeState::Pending),
            ("e".into(), NodeState::Pending),
        ],
        resume_job_id: Some("d".into()),
    };
    assert_eq!(
        text_of(Language::Ja, &b1, Some(&overlay), false),
        "*     [OK] a\n \\ \\\n* | | [--] b\n / /\n* / [OK] c\n|/\n* [  ] d  <- 再開位置\n* [  ] e\no @end\n"
    );
}

#[test]
fn 英語の注記は_レーンの図で描けない形を含む言い方() {
    let e1 = e1_definition();
    let text = edge_list_of(Language::En, &e1, None, false);
    assert!(
        text.starts_with(
            "note: the skip edges cross or form a shape the lane diagram cannot draw, so it is replaced by a list of edges.\nJobs:\n"
        ),
        "{text}"
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

    // 辺の一覧の `-v` は、ジョブの行の下に、4 つの空白で字下げした詳細。
    let crossing = e1_definition();
    let list = edge_list_of(Language::En, &crossing, None, true);
    assert!(
        list.contains(
            "Jobs:\n  a\n    command: echo a\n    success, warning -> e (skips b, c, d)\n  b\n"
        ),
        "{list}"
    );
    assert!(
        list.contains("Edges:\n  a -> e (success, warning)\n"),
        "{list}"
    );
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

const C1_TEXT: &str = "*   a\n|\\\n* \\   b\n|\\ \\\n* | | c\n| |/\n|/|\n* | d\n|/\n* e\no @end\n";

#[test]
fn 交差の例_c1_は_交差の行を含むレーンの図で出る() {
    let c1 = definition(&IDS5, &[("a", "d"), ("b", "e")], "");
    assert_eq!(text_of(Language::Ja, &c1, None, false), C1_TEXT);
    // 英語でも図は同じ(ラベルはジョブ ID)。
    assert_eq!(text_of(Language::En, &c1, None, false), C1_TEXT);
}

#[test]
fn 交差の例_c2_は_3_本の線が続けて交差し_横線を含む() {
    let c2 = definition(
        &["a", "b", "c", "d", "e", "f", "g"],
        &[("a", "d"), ("b", "e"), ("c", "f")],
        "",
    );
    let text = text_of(Language::Ja, &c2, None, false);
    assert_eq!(
        text,
        "*   a\n|\\\n* \\   b\n|\\ \\\n* \\ \\   c\n|\\ \\ \\\n| |_|/\n|/| |\n* | | d\n| |/\n|/|\n* | e\n|/\n* f\n* g\no @end\n"
    );
}

#[test]
fn 交差の図の_v_は_交差の行の置き場所を変えず_詳細の行を足す() {
    let c1 = definition(&IDS5, &[("a", "d"), ("b", "e")], "");
    let text = text_of(Language::En, &c1, None, true);
    assert_eq!(
        text,
        "*   a\n|\\\n| | command: echo a\n| | success -> d (skips b, c), warning -> b\n\
         * \\   b\n|\\ \\\n| | | command: echo b\n| | | success -> e (skips c, d), warning -> c\n\
         * | | c\n| | | command: echo c\n| |/\n|/|\n\
         * | d\n| | command: echo d\n|/\n* e\n| command: echo e\no @end\n"
    );
}

#[test]
fn 交差の図に_state_の状態を重ねる() {
    let c1 = definition(&IDS5, &[("a", "d"), ("b", "e")], "");
    let overlay = StateOverlay {
        states: vec![
            ("a".into(), NodeState::Ok),
            ("b".into(), NodeState::Ok),
            ("c".into(), NodeState::Error),
            ("d".into(), NodeState::Pending),
            ("e".into(), NodeState::Pending),
        ],
        resume_job_id: Some("c".into()),
    };
    let text = text_of(Language::Ja, &c1, Some(&overlay), false);
    assert_eq!(
        text,
        "*   [OK] a\n|\\\n* \\   [OK] b\n|\\ \\\n* | | [NG] c  <- 再開位置\n| |/\n|/|\n* | [  ] d\n|/\n* [  ] e\no @end\n"
    );
}

/// M2: a→e、b→d、c→e(5 ジョブ)。離れて同じ行き先を持つ線が、別の線と交差する(BR1.1)。
const M2_TEXT: &str = "\
*   a
|\\
* \\   b
|\\ \\
* \\ \\   c
|\\ \\ \\
| |/ /
|/| /
| |/
* / d
|/
* e
o @end
";

fn m2_definition() -> PipelineDefinition {
    definition(&IDS5, &[("a", "e"), ("b", "d"), ("c", "e")], "")
}

#[test]
fn 同じ行き先が混ざる交差の図_m2_は_text_の全文で出る() {
    assert_eq!(
        text_of(Language::Ja, &m2_definition(), None, false),
        M2_TEXT
    );
}

#[test]
fn 同じ行き先が混ざる交差で_横線を含む図_m4_が出る() {
    // M4: a→d、b→f、c→e(6 ジョブ)。3 本の線が続けて交差し(`_`)、終わりに同じ行き先が重なる。
    let m4 = definition(
        &["a", "b", "c", "d", "e", "f"],
        &[("a", "d"), ("b", "f"), ("c", "e")],
        "",
    );
    assert_eq!(
        text_of(Language::Ja, &m4, None, false),
        "*   a\n|\\\n* \\   b\n|\\ \\\n* \\ \\   c\n|\\ \\ \\\n| |_|/\n|/| |\n* | | d\n|/ /\n* / e\n|/\n* f\no @end\n"
    );
}

#[test]
fn 同じ行き先が混ざる交差の図の_v_は_寄せの行の置き場所を変えず_詳細の行を足す() {
    let text = text_of(Language::En, &m2_definition(), None, true);
    assert!(
        text.contains(
            "*   a\n|\\\n| | command: echo a\n| | success -> e (skips b, c, d), warning -> b\n"
        ),
        "{text}"
    );
    // 詳細の行を除くと、`-v` なしの図と同じ(寄せの行は、そのまま)。
    let without_details: String = text
        .lines()
        .filter(|l| {
            !l.contains("command:") && !l.contains("success ->") && !l.contains("warning ->")
        })
        .map(|l| format!("{l}\n"))
        .collect();
    assert_eq!(without_details, M2_TEXT);
}

#[test]
fn 同じ行き先が混ざる交差の図に_state_の状態を重ねる() {
    let overlay = StateOverlay {
        states: vec![
            ("a".into(), NodeState::Ok),
            ("b".into(), NodeState::Ok),
            ("c".into(), NodeState::Error),
            ("d".into(), NodeState::Pending),
            ("e".into(), NodeState::Pending),
        ],
        resume_job_id: Some("c".into()),
    };
    let text = text_of(Language::Ja, &m2_definition(), Some(&overlay), false);
    assert_eq!(
        text,
        "*   [OK] a\n|\\\n* \\   [OK] b\n|\\ \\\n* \\ \\   [NG] c  <- 再開位置\n|\\ \\ \\\n| |/ /\n|/| /\n| |/\n* / [  ] d\n|/\n* [  ] e\no @end\n"
    );
}

const IDS6: [&str; 6] = ["a", "b", "c", "d", "e", "f"];

/// N1: 「両方の飛び越し」`a` が `d` へと、通常の飛び越し `b` の成功が `e` へが交差する。
fn n1_definition() -> PipelineDefinition {
    definition(&IDS6, &[("b", "e")], &both_skip_extra("d"))
}

const N1_TEXT: &str =
    "*   a\n \\\n* \\   b\n \\ \\\n* | | c\n  |/\n /|\n* | d\n|/\n* e\n* f\no @end\n";

#[test]
fn n1_の図は_text_の全文で出る() {
    // a は次のジョブへの辺がなく、b、c は実行されないので、その下の行の列 0 は空白(BR1.4)。
    assert_eq!(
        text_of(Language::Ja, &n1_definition(), None, false),
        N1_TEXT
    );
}

#[test]
fn n1_の_v_の詳細の行の列0は_図の行と同じ規則() {
    // 詳細の行の列 0 も、そのジョブが実行されうる、かつ次のジョブへの辺を持つときだけ `|`(BR1.4)。
    assert_eq!(
        text_of(Language::En, &n1_definition(), None, true),
        "*   a\n \\\n  | command: echo a\n  | success, warning -> d (skips b, c)\n\
         * \\   b\n \\ \\\n  | | command: echo b\n  | | success -> e (skips c, d), warning -> c\n\
         * | | c\n  | | command: echo c\n  |/\n /|\n\
         * | d\n| | command: echo d\n|/\n\
         * e\n| command: echo e\n* f\n| command: echo f\no @end\n"
    );
}

#[test]
fn n2_の_v_の詳細の行の列0は_図の行と同じ規則() {
    // N2: 分岐 a の成功が c、警告が e と、通常の飛び越し b の成功が d へが交差する。
    let n2 = definition(&IDS6, &[("b", "d")], &branch_extra("c", "e"));
    assert_eq!(
        text_of(Language::En, &n2, None, true),
        "*     a\n \\ \\\n  | | command: echo a\n  | | success -> c (skips b), warning -> e (skips b, c, d)\n\
         * \\ \\   b\n \\ \\ \\\n  | | | command: echo b\n  | | | success -> d (skips c), warning -> c\n\
         \x20 |/ /\n /| |\n\
         * | | c\n| | | command: echo c\n|/ /\n\
         * / d\n| | command: echo d\n|/\n\
         * e\n| command: echo e\n* f\n| command: echo f\no @end\n"
    );
}

#[test]
fn n5_の_v_の詳細は_合流の行のあとに置き_列0は図の行と同じ規則() {
    // N5: 分岐 a の成功が c、警告が e の線に、c の成功が e へ合流する。
    let n5 = definition(&IDS6, &[("c", "e")], &branch_extra("c", "e"));
    assert_eq!(
        text_of(Language::En, &n5, None, true),
        "*     a\n \\ \\\n  | | command: echo a\n  | | success -> c (skips b), warning -> e (skips b, c, d)\n\
         * | | b\n  | | command: echo b\n / /\n\
         * | c\n|\\|\n| | command: echo c\n| | success -> e (skips d), warning -> d\n\
         * | d\n| | command: echo d\n|/\n\
         * e\n| command: echo e\n* f\n| command: echo f\no @end\n"
    );
}

#[test]
fn n1_の図に_state_の状態を重ねる() {
    let overlay = StateOverlay {
        states: vec![
            ("a".into(), NodeState::Ok),
            ("b".into(), NodeState::Skipped),
            ("c".into(), NodeState::Skipped),
            ("d".into(), NodeState::Error),
            ("e".into(), NodeState::Pending),
            ("f".into(), NodeState::Pending),
        ],
        resume_job_id: Some("d".into()),
    };
    assert_eq!(
        text_of(Language::Ja, &n1_definition(), Some(&overlay), false),
        "*   [OK] a\n \\\n* \\   [--] b\n \\ \\\n* | | [--] c\n  |/\n /|\n* | [NG] d  <- 再開位置\n|/\n* [  ] e\n* [  ] f\no @end\n"
    );
}
