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

//! `conductor graph` の、実際のバイナリのテスト(FR2.1〜FR2.7、FR1.12)。

mod bin_support;

use bin_support::*;

const SECRET: &str = "s3cr3t-VALUE-do-not-print";

const A_TO_D: &str = "[[job.transitions]]\non = \"success\"\nnext = \"d\"\n";

/// a の成功が d へ飛ぶ 5 個のジョブ(例 G2)。`d_exit` は、d の終了コード。
fn g2_definition(d_exit: i32) -> String {
    let ok = helper_command("exit 0");
    let jobs = [
        job_block("a", &ok, A_TO_D),
        job_block("b", &ok, ""),
        job_block("c", &ok, ""),
        job_block("d", &helper_command(&format!("exit {d_exit}")), ""),
        job_block("e", &ok, ""),
    ];
    definition("p", &jobs)
}

const G2_TEXT: &str = "*   a\n|\\\n* | b\n* | c\n|/\n* d\n* e\no @end\n";

const G2_MERMAID: &str = "flowchart TD\n  n0[\"a\"]\n  n1[\"b\"]\n  n2[\"c\"]\n  n3[\"d\"]\n  n4[\"e\"]\n  nend([\"@end\"])\n  n0 -->|\"warning\"| n1\n  n0 -->|\"success\"| n3\n  n1 --> n2\n  n2 --> n3\n  n3 --> n4\n  n4 --> nend\n";

#[test]
fn text_と_mermaid_の出力が例と一致し_ジョブは動かない() {
    let project = Project::new(&g2_definition(0));
    let text_out = project.run(&["graph"]);
    assert_eq!(
        text_out.status.code(),
        Some(0),
        "{}",
        text(&text_out.stderr)
    );
    assert_eq!(text(&text_out.stdout), G2_TEXT);
    assert_eq!(text(&text_out.stderr), "");

    let mermaid = project.run(&["graph", "--format", "mermaid"]);
    assert_eq!(mermaid.status.code(), Some(0));
    assert_eq!(text(&mermaid.stdout), G2_MERMAID);
    // `-v` は、Mermaid には影響しない。
    let verbose = project.run(&["graph", "--format", "mermaid", "-v"]);
    assert_eq!(text(&verbose.stdout), G2_MERMAID);
    // ステートは作られない。
    assert!(!project.state_path().exists());
}

#[test]
fn state_は_実際に作ったステートの状態と再開位置を重ねる() {
    let project = Project::new(&g2_definition(1));
    // d がエラーで止まる(a は d へ飛び、b と c はスキップ)。
    let run = project.run(&["pipeline", "p"]);
    assert_eq!(run.status.code(), Some(1), "{}", text(&run.stderr));
    let before = std::fs::read(project.state_path()).unwrap();

    let out = project.run(&["graph", "--state"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert_eq!(
        text(&out.stdout),
        "*   [OK] a\n|\\\n* | [--] b\n* | [--] c\n|/\n* [NG] d  <- resume here\n* [  ] e\no @end\n"
    );
    assert_eq!(std::fs::read(project.state_path()).unwrap(), before);

    // Mermaid。
    let mermaid = project.run(&["graph", "--state", "--format", "mermaid"]);
    let mermaid = text(&mermaid.stdout);
    assert!(mermaid.contains("n3[\"[NG] d\"]"), "{mermaid}");
    assert!(mermaid.ends_with("  classDef resume stroke-width:3px\n  class n3 resume\n"));

    // ステートが定義と食い違うと、65。
    std::fs::write(
        project.def_path(),
        g2_definition(1).replace("id = \"e\"", "id = \"z\""),
    )
    .unwrap();
    let out = project.run(&["graph", "--state"]);
    assert_eq!(out.status.code(), Some(65), "{}", text(&out.stderr));
    assert_eq!(text(&out.stdout), "");
}

#[test]
fn 環境変数の値は_どの出力にも出ない() {
    let ok = helper_command("exit 0");
    let extra = format!("env_file = [\"j.env\"]\n[job.env]\nFROM_JOB_ENV = \"{SECRET}\"\n{A_TO_D}");
    let jobs = [
        job_block("a", &ok, &extra),
        job_block("b", &ok, ""),
        job_block("c", &ok, ""),
        job_block("d", &ok, ""),
    ];
    let mut text_definition = definition("p", &jobs);
    text_definition = text_definition.replace(
        "[pipeline]\nname = \"p\"\n",
        &format!("[pipeline]\nname = \"p\"\nenv_file = \"p.env\"\n[pipeline.env]\nFROM_PIPELINE_ENV = \"{SECRET}\"\n"),
    );
    let project = Project::new(&text_definition);
    std::fs::write(
        project.dir.path().join("p.env"),
        format!("FROM_PIPELINE_FILE={SECRET}\n"),
    )
    .unwrap();
    std::fs::write(
        project.dir.path().join("j.env"),
        format!("FROM_JOB_FILE={SECRET}\n"),
    )
    .unwrap();
    for args in [
        &["graph"][..],
        &["graph", "-v"][..],
        &["graph", "--format", "mermaid"][..],
        &["graph", "--state", "-v"][..],
    ] {
        let out = project.run(args);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{args:?}: {}",
            text(&out.stderr)
        );
        for stream in [text(&out.stdout), text(&out.stderr)] {
            assert!(!stream.contains(SECRET), "{args:?}: 値が出ている");
            for name in [
                "FROM_JOB_ENV",
                "FROM_PIPELINE_ENV",
                "FROM_PIPELINE_FILE",
                "FROM_JOB_FILE",
            ] {
                assert!(!stream.contains(name), "{args:?}: 名前 {name} が出ている");
            }
        }
    }
    // `-v` は、コマンドだけを出す。
    let verbose = project.run(&["graph", "-v"]);
    assert!(text(&verbose.stdout).contains("command: "));
}

#[test]
fn validate_と同じ検査をする_env_の誤りは_64_で_重複キーは警告で_0() {
    let ok = helper_command("exit 0");
    let with_env_file = |extra: &str| definition("p", &[job_block("a", &ok, extra)]);

    // `.env` が無い: validate も graph も 64。
    let project = Project::new(&with_env_file("env_file = [\"missing.env\"]\n"));
    let validate = project.run(&["validate"]);
    let graph = project.run(&["graph"]);
    assert_eq!(validate.status.code(), Some(64));
    assert_eq!(graph.status.code(), Some(64), "{}", text(&graph.stderr));
    assert_eq!(text(&graph.stdout), "");
    assert_eq!(text(&graph.stderr), text(&validate.stderr));

    // 重複キーは警告(標準エラー)で、図は出る。値は出ない。
    let project = Project::new(&with_env_file("env_file = [\"d.env\"]\n"));
    std::fs::write(
        project.dir.path().join("d.env"),
        format!("DUP=one\nDUP={SECRET}\n"),
    )
    .unwrap();
    let graph = project.run(&["graph"]);
    assert_eq!(graph.status.code(), Some(0), "{}", text(&graph.stderr));
    assert_eq!(text(&graph.stdout), "* a\no @end\n");
    let err = text(&graph.stderr);
    assert!(err.contains("conductor: warning: "), "{err}");
    assert!(err.contains("DUP") && !err.contains(SECRET), "{err}");
}

/// 同じ行き先(a→d、b→d。例 S1)の定義の text が、合流の行で出る。
#[test]
fn 同じ行き先の定義は_合流の行のレーンの図で出る() {
    let ok = helper_command("exit 0");
    let jobs = [
        job_block("a", &ok, A_TO_D),
        job_block("b", &ok, A_TO_D),
        job_block("c", &ok, ""),
        job_block("d", &ok, ""),
        job_block("e", &ok, ""),
    ];
    let project = Project::new(&definition("p", &jobs));
    let out = project.run(&["graph"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert_eq!(
        text(&out.stdout),
        "*   a\n|\\\n* | b\n|\\|\n* | c\n|/\n* d\n* e\no @end\n"
    );
    assert_eq!(text(&out.stderr), "");
}

/// 成功と警告の両方が飛び越す定義(例 D1)の text が、通常の飛び越しと同じ線(主線の線だけ省く)で出る。
/// 交差と「両方の飛び越し」が混ざる形も、今回からレーンの図で出る(BR2.1)。
#[test]
fn 両方の飛び越しの定義は_通常の飛び越しと同じ線で描き_交差と混ざる形もレーンの図で出る() {
    let ok = helper_command("exit 0");
    let both = "[[job.transitions]]\non = \"warning\"\nnext = \"c\"\n\n[[job.transitions]]\non = \"success\"\nnext = \"c\"\n";
    let jobs = [
        job_block("a", &ok, both),
        job_block("b", &ok, ""),
        job_block("c", &ok, ""),
        job_block("d", &ok, ""),
    ];
    let project = Project::new(&definition("p", &jobs));
    let out = project.run(&["graph"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert_eq!(
        text(&out.stdout),
        "*   a\n \\\n* | b\n /\n* c\n* d\no @end\n"
    );

    // 交差し、かつ「両方の飛び越し」(a が成功も警告も e へ)を含む形。b、c、d は実行されないので、
    // その下の行の列 0 は空白(BR1.4)。
    let both_to_e = "[[job.transitions]]\non = \"warning\"\nnext = \"e\"\n\n[[job.transitions]]\non = \"success\"\nnext = \"e\"\n";
    let to_e = "[[job.transitions]]\non = \"success\"\nnext = \"e\"\n";
    let to_f = "[[job.transitions]]\non = \"success\"\nnext = \"f\"\n";
    let crossing = [
        job_block("a", &ok, both_to_e),
        job_block("b", &ok, to_f),
        job_block("c", &ok, to_e),
        job_block("d", &ok, ""),
        job_block("e", &ok, ""),
        job_block("f", &ok, ""),
    ];
    let project = Project::new(&definition("p", &crossing));
    let out = project.run(&["graph"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert_eq!(
        text(&out.stdout),
        "*   a\n \\\n* \\   b\n \\ \\\n* \\ \\   c\n \\ \\ \\\n  | |/\n  |/|\n* / | d\n / /\n* / e\n|/\n* f\no @end\n"
    );
    assert_eq!(text(&out.stderr), "");
}

/// 交差する飛び越し(a→d、b→e。例 C1)の定義の text が、交差の行を含むレーンの図で出る。
#[test]
fn 交差する飛び越しの定義は_交差の行を含むレーンの図で出る() {
    let ok = helper_command("exit 0");
    let to = |t: &str| format!("[[job.transitions]]\non = \"success\"\nnext = \"{t}\"\n");
    let jobs = [
        job_block("a", &ok, &to("d")),
        job_block("b", &ok, &to("e")),
        job_block("c", &ok, ""),
        job_block("d", &ok, ""),
        job_block("e", &ok, ""),
    ];
    let project = Project::new(&definition("p", &jobs));
    let out = project.run(&["graph"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert_eq!(
        text(&out.stdout),
        "*   a\n|\\\n* \\   b\n|\\ \\\n* | | c\n| |/\n|/|\n* | d\n|/\n* e\no @end\n"
    );
    assert_eq!(text(&out.stderr), "");
}

/// 交差に同じ行き先が混ざる飛び越し(a→e、b→d、c→e。例 M2)の定義の text が、寄せの行を含むレーンの図で出る。
#[test]
fn 交差に同じ行き先が混ざる定義は_寄せの行を含むレーンの図で出る() {
    let ok = helper_command("exit 0");
    let to = |t: &str| format!("[[job.transitions]]\non = \"success\"\nnext = \"{t}\"\n");
    let jobs = [
        job_block("a", &ok, &to("e")),
        job_block("b", &ok, &to("d")),
        job_block("c", &ok, &to("e")),
        job_block("d", &ok, ""),
        job_block("e", &ok, ""),
    ];
    let project = Project::new(&definition("p", &jobs));
    let out = project.run(&["graph"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert_eq!(
        text(&out.stdout),
        "*   a\n|\\\n* \\   b\n|\\ \\\n* \\ \\   c\n|\\ \\ \\\n| |/ /\n|/| /\n| |/\n* / d\n|/\n* e\no @end\n"
    );
    assert_eq!(text(&out.stderr), "");
}

/// 分岐(a の成功が c、警告が d。例 B1)の定義の text が、2 本の線を同時に開くレーンの図で出る。
#[test]
fn 分岐の定義は_2本の線を同時に開くレーンの図で出る() {
    let ok = helper_command("exit 0");
    let branch = "[[job.transitions]]\non = \"success\"\nnext = \"c\"\n\n[[job.transitions]]\non = \"warning\"\nnext = \"d\"\n";
    let jobs = [
        job_block("a", &ok, branch),
        job_block("b", &ok, ""),
        job_block("c", &ok, ""),
        job_block("d", &ok, ""),
        job_block("e", &ok, ""),
    ];
    let project = Project::new(&definition("p", &jobs));
    let out = project.run(&["graph"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert_eq!(
        text(&out.stdout),
        "*     a\n \\ \\\n* | | b\n / /\n* / c\n|/\n* d\n* e\no @end\n"
    );
    assert_eq!(text(&out.stderr), "");
}

/// 「両方の飛び越し」(a が d へ)と通常の飛び越し(b の成功が e へ)が交差する定義(例 N1)の text と `-v` が、
/// レーンの図で出る。詳細の行の列 0 も、図の行と同じ規則(BR1.4、BR2.1)。
#[test]
fn 両方の飛び越しと通常の飛び越しが交差する定義_n1_はレーンの図で出る() {
    let ok = helper_command("exit 0");
    let both_to_d = "[[job.transitions]]\non = \"warning\"\nnext = \"d\"\n\n[[job.transitions]]\non = \"success\"\nnext = \"d\"\n";
    let to_e = "[[job.transitions]]\non = \"success\"\nnext = \"e\"\n";
    let jobs = [
        job_block("a", &ok, both_to_d),
        job_block("b", &ok, to_e),
        job_block("c", &ok, ""),
        job_block("d", &ok, ""),
        job_block("e", &ok, ""),
        job_block("f", &ok, ""),
    ];
    let project = Project::new(&definition("p", &jobs));
    let out = project.run(&["graph"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert_eq!(
        text(&out.stdout),
        "*   a\n \\\n* \\   b\n \\ \\\n* | | c\n  |/\n /|\n* | d\n|/\n* e\n* f\no @end\n"
    );
    assert_eq!(text(&out.stderr), "");

    // `-v`: 実行されない b、c と、次のジョブへの辺がない a の下の詳細の行は、列 0 が空白。
    let verbose = project.run(&["graph", "-v"]);
    assert_eq!(verbose.status.code(), Some(0), "{}", text(&verbose.stderr));
    let stdout = text(&verbose.stdout);
    // 詳細のうち、コマンドの行(ジョブごとに 1 行。補助の実行ファイルの `exit 0`)の列 0: a、b、c は空白、d、e、f は `|`。
    let firsts: Vec<char> = stdout
        .lines()
        .filter(|l| !l.starts_with('*') && l.ends_with("exit 0"))
        .filter_map(|l| l.chars().next())
        .collect();
    assert_eq!(firsts, vec![' ', ' ', ' ', '|', '|', '|'], "{stdout}");
}
