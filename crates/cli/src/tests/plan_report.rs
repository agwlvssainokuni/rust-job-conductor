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

//! 計画の整形(U3。BR3.1〜BR3.3、BR5.1、BR6.1)。期待する出力の例 P1〜P8、英語の例 P2e、P4e が、
//! 1 文字も違わず一致する。

use crate::report::plan_text::render_plan;
use crate::types::Language;
use conductor_core::{
    Assumption, EnvEntry, EnvFileContent, ExecutionPlan, JobState, JobStatus, LoadedEnvFiles,
    PipelineState, PlanInput, parse_definition, plan_pipeline, plan_run,
};

/// 定義の本文(`[pipeline]` の続きに、ジョブを書く)。
fn def_text(pipeline_extra: &str, jobs: &str) -> String {
    format!("format_version = 1\n[pipeline]\nname = \"demo\"\n{pipeline_extra}{jobs}")
}

fn job(id: &str, command: &str, extra: &str) -> String {
    format!("[[job]]\nid = \"{id}\"\ncommand = \"{command}\"\n{extra}")
}

/// 空白で分けた単語(テストの定義は、単純なコマンドだけ)。
fn words(command: &str) -> Vec<String> {
    command.split_whitespace().map(str::to_string).collect()
}

struct Case {
    text: String,
    commands: Vec<Vec<String>>,
    state: Option<PipelineState>,
    run_id: Option<&'static str>,
    assumptions: Vec<Assumption>,
}

impl Case {
    fn new(text: String, commands: &[&str]) -> Self {
        Self {
            text,
            commands: commands.iter().map(|c| words(c)).collect(),
            state: None,
            run_id: None,
            assumptions: Vec::new(),
        }
    }

    fn pipeline(&self) -> ExecutionPlan {
        self.plan(None)
    }

    fn run(&self, job_id: &str) -> ExecutionPlan {
        self.plan(Some(job_id))
    }

    fn plan(&self, run_job: Option<&str>) -> ExecutionPlan {
        let def = parse_definition(&self.text, "t.toml", None).expect("正しい定義");
        let files = LoadedEnvFiles::new();
        let input = PlanInput {
            definition: &def,
            state: self.state.as_ref(),
            commands: &self.commands,
            env_files: &files,
            run_id: self.run_id,
            assumptions: &self.assumptions,
        };
        match run_job {
            Some(id) => plan_run(id, &input),
            None => plan_pipeline(&input),
        }
        .expect("計画")
    }
}

fn state(jobs: &[(&str, JobStatus)], resume: Option<&str>, run_id: Option<&str>) -> PipelineState {
    PipelineState {
        pipeline_name: "demo".into(),
        definition_path: "t.toml".into(),
        job_states: jobs
            .iter()
            .map(|(id, status)| JobState {
                job_id: id.to_string(),
                status: *status,
                exit_code: None,
                skip_reason: None,
            })
            .collect(),
        resume_job_id: resume.map(str::to_string),
        end_reached: false,
        completed: false,
        run_id: run_id.map(str::to_string),
    }
}

fn echo_abc() -> Case {
    Case::new(
        def_text(
            "",
            &[
                job("a", "echo a", ""),
                job("b", "echo b", ""),
                job("c", "echo c", ""),
            ]
            .concat(),
        ),
        &["echo a", "echo b", "echo c"],
    )
}

/// P2・P2e の材料(a〜d。a は成功で記録済み、b がエラーで停止。run ID r1。c にリトライ。c=75)。
fn p2_case() -> Case {
    let jobs = [
        job("a", "echo a", ""),
        job("b", "echo b", ""),
        job(
            "c",
            "fetch --retry",
            "[job.retry]\nexit_codes = [75, 76]\nmax = 3\ninterval = \"5s\"\n",
        ),
        job("d", "echo d", ""),
    ]
    .concat();
    let mut case = Case::new(
        def_text(
            "[pipeline.env]\nAPI_URL = \"https://example.invalid\"\n",
            &jobs,
        ),
        &["echo a", "echo b", "fetch --retry", "echo d"],
    );
    case.state = Some(state(
        &[
            ("a", JobStatus::Success),
            ("b", JobStatus::Error),
            ("c", JobStatus::Pending),
            ("d", JobStatus::Pending),
        ],
        Some("b"),
        Some("r1"),
    ));
    case.run_id = Some("r1");
    case.assumptions = vec![Assumption::new("c", 75)];
    case
}

/// P4・P4e の材料(a が警告で `@end`。a=3)。
fn p4_case() -> Case {
    let jobs = [
        job(
            "a",
            "echo a",
            "warning_exit_codes = [3]\n[[job.transitions]]\non = \"warning\"\nnext = \"@end\"\n",
        ),
        job("b", "echo b", ""),
        job("c", "echo c", ""),
    ]
    .concat();
    let mut case = Case::new(def_text("", &jobs), &["echo a", "echo b", "echo c"]);
    case.assumptions = vec![Assumption::new("a", 3)];
    case
}

const P1: &str = "\
計画(--dry-run): パイプライン demo
再開位置: 最初から
1. [実行する] a
     コマンド: [\"echo\", \"a\"]
     環境変数: CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_ATTEMPT
2. [実行する] b
     コマンド: [\"echo\", \"b\"]
     環境変数: CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_ATTEMPT
3. [実行する] c
     コマンド: [\"echo\", \"c\"]
     環境変数: CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_ATTEMPT
結果(模擬): 完了
";

const P2: &str = "\
計画(--dry-run): パイプライン demo(run ID: r1)
再開位置: b(前回の停止)
仮定: c=75
1. [実行済み] a
2. [実行する] b  <- 再開位置
     コマンド: [\"echo\", \"b\"]
     環境変数: API_URL, CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_RUN_ID, CONDUCTOR_ATTEMPT
3. [実行する(仮定: 終了コード 75 → エラー、リトライの対象)] c
     コマンド: [\"fetch\", \"--retry\"]
     環境変数: API_URL, CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_RUN_ID, CONDUCTOR_ATTEMPT
     リトライ: 終了コード 75、76 を最大 3 回、間隔 5 秒
4. [実行されない(停止のあと)] d
結果(模擬): 停止(エラー: c)
";

const P2E: &str = "\
plan (--dry-run): pipeline demo (run ID: r1)
resume: b (previous stop)
assume: c=75
1. [already executed] a
2. [run] b  <- resume here
     command: [\"echo\", \"b\"]
     env: API_URL, CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_RUN_ID, CONDUCTOR_ATTEMPT
3. [run (assumed: exit code 75 -> error, retry target)] c
     command: [\"fetch\", \"--retry\"]
     env: API_URL, CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_RUN_ID, CONDUCTOR_ATTEMPT
     retry: exit codes 75, 76, up to 3 times, every 5 s
4. [not run (after stop)] d
result (simulated): stopped (error: c)
";

const P3: &str = "\
計画(--dry-run): パイプライン demo
再開位置: 最初から
仮定: b=1
使われなかった仮定: b
1. [実行する] a
     コマンド: [\"echo\", \"a\"]
     環境変数: CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_ATTEMPT
2. [スキップ(遷移)] b
3. [実行する] c
     コマンド: [\"echo\", \"c\"]
     環境変数: CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_ATTEMPT
結果(模擬): 完了
";

const P4: &str = "\
計画(--dry-run): パイプライン demo
再開位置: 最初から
仮定: a=3
1. [実行する(仮定: 終了コード 3 → 警告)] a
     コマンド: [\"echo\", \"a\"]
     環境変数: CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_ATTEMPT
2. [スキップ(@end のあと)] b
3. [スキップ(@end のあと)] c
結果(模擬): @end に到達(完了)
";

const P4E: &str = "\
plan (--dry-run): pipeline demo
resume: from the start
assume: a=3
1. [run (assumed: exit code 3 -> warning)] a
     command: [\"echo\", \"a\"]
     env: CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_ATTEMPT
2. [skip (after @end)] b
3. [skip (after @end)] c
result (simulated): reached @end (completed)
";

const P5: &str = "\
計画(--dry-run): パイプライン demo
対象のジョブ: b
1. [実行する] b
     コマンド: [\"echo\", \"b\"]
     環境変数: CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_ATTEMPT
次の動き: 次のジョブ c
";

const P6: &str = "\
計画(--dry-run): パイプライン demo
再開位置: 最初から
1. [実行する(既定: 終了コード 0 → エラー)] a
     コマンド: [\"make\"]
     環境変数: CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_ATTEMPT
2. [実行されない(停止のあと)] b
結果(模擬): 停止(エラー: a)
";

const P7: &str = "\
計画(--dry-run): パイプライン demo
対象のジョブ: a
1. [実行する] a
     コマンド: [\"echo\", \"a\"]
     環境変数: CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_ATTEMPT
次の動き: 次のジョブ c(スキップ: b)
";

const P8: &str = "\
計画(--dry-run): パイプライン demo
対象のジョブ: a
1. [実行する] a
     コマンド: [\"echo\", \"a\"]
     環境変数: CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_ATTEMPT
次の動き: @end に到達(残りのジョブは、呼ばれたときにスキップ)
";

#[test]
fn 例_p1_から_p4_と_p2e_p4e_が一文字も違わず一致する() {
    assert_eq!(render_plan(Language::Ja, &echo_abc().pipeline()), P1);

    let p2 = p2_case().pipeline();
    assert_eq!(render_plan(Language::Ja, &p2), P2);
    assert_eq!(render_plan(Language::En, &p2), P2E);

    let jobs = [
        job(
            "a",
            "echo a",
            "[[job.transitions]]\non = \"success\"\nnext = \"c\"\n",
        ),
        job("b", "echo b", ""),
        job("c", "echo c", ""),
    ]
    .concat();
    let mut p3 = Case::new(def_text("", &jobs), &["echo a", "echo b", "echo c"]);
    p3.assumptions = vec![Assumption::new("b", 1)];
    assert_eq!(render_plan(Language::Ja, &p3.pipeline()), P3);

    let p4 = p4_case().pipeline();
    assert_eq!(render_plan(Language::Ja, &p4), P4);
    assert_eq!(render_plan(Language::En, &p4), P4E);
}

#[test]
fn 既定のリトライを当てはめたジョブの計画は同じ値の表を書いたときと一文字も違わない() {
    // P2 の c の `[job.retry]` を `[pipeline.retry]` に移し、ほかのジョブは `retry = false` で打ち消す
    // (u1-pipeline-retry の BR2.4、NFR1.5)。
    let off = "retry = false\n";
    let jobs = [
        job("a", "echo a", off),
        job("b", "echo b", off),
        job("c", "fetch --retry", ""),
        job("d", "echo d", off),
    ]
    .concat();
    let mut by_default = p2_case();
    by_default.text = def_text(
        "[pipeline.env]\nAPI_URL = \"https://example.invalid\"\n\
         [pipeline.retry]\nexit_codes = [75, 76]\nmax = 3\ninterval = \"5s\"\n",
        &jobs,
    );
    let plan = by_default.pipeline();
    assert_eq!(render_plan(Language::Ja, &plan), P2);
    assert_eq!(render_plan(Language::En, &plan), P2E);
}

#[test]
fn 例_p5_から_p8_の_run_の計画が一文字も違わず一致する() {
    let mut p5 = echo_abc();
    p5.state = Some(state(
        &[
            ("a", JobStatus::Success),
            ("b", JobStatus::Pending),
            ("c", JobStatus::Pending),
        ],
        None,
        None,
    ));
    assert_eq!(render_plan(Language::Ja, &p5.run("b")), P5);

    let p6 = Case::new(
        def_text(
            "",
            &[
                job("a", "make", "success_exit_codes = [10]\n"),
                job("b", "echo b", ""),
            ]
            .concat(),
        ),
        &["make", "echo b"],
    );
    assert_eq!(render_plan(Language::Ja, &p6.pipeline()), P6);

    let jobs7 = [
        job(
            "a",
            "echo a",
            "[[job.transitions]]\non = \"success\"\nnext = \"c\"\n",
        ),
        job("b", "echo b", ""),
        job("c", "echo c", ""),
        job("d", "echo d", ""),
    ]
    .concat();
    let p7 = Case::new(
        def_text("", &jobs7),
        &["echo a", "echo b", "echo c", "echo d"],
    );
    assert_eq!(render_plan(Language::Ja, &p7.run("a")), P7);

    let jobs8 = [
        job(
            "a",
            "echo a",
            "[[job.transitions]]\non = \"success\"\nnext = \"@end\"\n",
        ),
        job("b", "echo b", ""),
        job("c", "echo c", ""),
    ]
    .concat();
    let p8 = Case::new(def_text("", &jobs8), &["echo a", "echo b", "echo c"]);
    assert_eq!(render_plan(Language::Ja, &p8.run("a")), P8);
}

#[test]
fn コマンドの語とジョブidは_制御文字がエスケープされる() {
    let mut plan = echo_abc().pipeline();
    plan.entries[0].command_words = vec!["echo".into(), "x\u{1b}[31my\nz".into()];
    plan.entries[1].job_id = "b\u{1b}evil".into();
    plan.unused_assumptions = vec!["u\u{7}id".into()];
    for lang in [Language::Ja, Language::En] {
        let text = render_plan(lang, &plan);
        assert!(
            !text.contains('\u{1b}') && !text.contains('\u{7}'),
            "{text}"
        );
        assert!(text.contains("\"x\\u{1b}[31my\\nz\""), "{text}");
        assert!(text.contains("b\\u{1b}evil"), "{text}");
        assert!(text.contains("u\\u{7}id"), "{text}");
    }
}

#[test]
fn 環境変数の名前は_層の順で_最後に_conductor_の変数() {
    let def = parse_definition(
        &def_text(
            "env_file = \"p.env\"\n[pipeline.env]\nP_ENV = \"v\"\n",
            &job(
                "a",
                "echo a",
                "env_file = \"a.env\"\n[job.env]\nJ_ENV = \"v\"\n",
            ),
        ),
        "t.toml",
        None,
    )
    .expect("正しい定義");
    let mut files = LoadedEnvFiles::new();
    for (reference, name) in [("p.env", "P_FILE"), ("a.env", "A_FILE")] {
        let i = files.push_file(EnvFileContent {
            file: reference.into(),
            entries: vec![EnvEntry {
                name: name.into(),
                value: "s3cr3t-VALUE-do-not-print".into(),
                line: 1,
            }],
            warnings: Vec::new(),
        });
        files.link(reference, i);
    }
    let commands = vec![words("echo a")];
    let plan = plan_pipeline(&PlanInput {
        definition: &def,
        state: None,
        commands: &commands,
        env_files: &files,
        run_id: Some("r9"),
        assumptions: &[],
    })
    .expect("計画");
    let text = render_plan(Language::En, &plan);
    assert!(
        text.contains("env: P_FILE, P_ENV, A_FILE, J_ENV, CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_RUN_ID, CONDUCTOR_ATTEMPT\n"),
        "{text}"
    );
    assert!(!text.contains("s3cr3t"), "値は出さない: {text}");
}

#[test]
fn 既定の成功と仮定の成功の見出しの違い() {
    // 指定した成功は、「仮定」と示す。指定のない成功は、補足を付けない。
    let mut case = echo_abc();
    case.assumptions = vec![Assumption::new("b", 0)];
    let text = render_plan(Language::En, &case.pipeline());
    assert!(
        text.contains("2. [run (assumed: exit code 0 -> success)] b\n"),
        "{text}"
    );
    assert!(text.contains("1. [run] a\n"), "{text}");
}

#[test]
fn 仮定のある_run_の次の動きは_先頭に仮定を添える() {
    let mut case = echo_abc();
    case.assumptions = vec![Assumption::new("a", 1)];
    let ja = render_plan(Language::Ja, &case.run("a"));
    assert!(
        ja.contains("次の動き: (仮定: 終了コード 1 → エラー) 停止\n"),
        "{ja}"
    );
    let en = render_plan(Language::En, &case.run("a"));
    assert!(
        en.contains("next step: (assumed: exit code 1 -> error) stop\n"),
        "{en}"
    );
    assert!(
        en.starts_with("plan (--dry-run): pipeline demo\ntarget job: a\n"),
        "{en}"
    );
    // 実行済みのジョブは、次の動きを持たない。
    let mut done = echo_abc();
    done.state = Some(state(
        &[("a", JobStatus::Success), ("b", JobStatus::Pending)],
        None,
        None,
    ));
    let text = render_plan(Language::Ja, &done.run("a"));
    assert!(
        text.contains("1. [実行済み] a\n") && !text.contains("次の動き"),
        "{text}"
    );
}

#[test]
fn 遷移とスキップの英語の語句() {
    let jobs = [
        job(
            "a",
            "echo a",
            "[[job.transitions]]\non = \"success\"\nnext = \"c\"\n",
        ),
        job("b", "echo b", ""),
        job("c", "echo c", ""),
    ]
    .concat();
    let mut case = Case::new(def_text("", &jobs), &["echo a", "echo b", "echo c"]);
    case.assumptions = vec![Assumption::new("b", 1)];
    let text = render_plan(Language::En, &case.pipeline());
    assert!(text.contains("unused assumptions: b\n"), "{text}");
    assert!(text.contains("2. [skip (transition)] b\n"), "{text}");
    assert!(text.ends_with("result (simulated): completed\n"), "{text}");
}
