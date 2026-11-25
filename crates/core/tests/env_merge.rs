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

//! 層の統合と自動の変数(U1。BR4.1〜BR4.4、BR5.1〜BR5.3、BR6.1、BR8.2)。

mod common;

use common::{SECRET, add_env_file, env_ref, paths};
use conductor_core::{
    EffectiveEnv, EnvContext, EnvFileContent, LoadedEnvFiles, PipelineDefinition, env_file_refs,
    merge_env, parse_definition, parse_env_file,
};

fn def(text: &str) -> PipelineDefinition {
    parse_definition(text, "t.toml", None).unwrap_or_else(|e| panic!("{e:?}"))
}

fn content(text: &str) -> EnvFileContent {
    parse_env_file(text, "x.env").unwrap()
}

fn ctx(run_id: Option<&str>, attempt: u32) -> EnvContext<'_> {
    EnvContext {
        pipeline_name: "p",
        job_id: "a",
        run_id,
        attempt,
    }
}

fn get<'a>(env: &'a EffectiveEnv, name: &str) -> Option<&'a str> {
    env.additions()
        .into_iter()
        .find(|(n, _)| *n == name)
        .map(|(_, v)| v)
}

fn names(env: &EffectiveEnv) -> Vec<&str> {
    env.additions().into_iter().map(|(n, _)| n).collect()
}

const LAYERED: &str = "format_version = 1
[pipeline]
name = \"p\"
env_file = \"p.env\"
[pipeline.env]
V = \"pipeline-env\"
[[job]]
id = \"a\"
command = \"c\"
env_file = [\"j1.env\", \"j2.env\"]
[job.env]
V = \"job-env\"
";

#[test]
fn 層の優先順位は_pipeline_env_file_から_job_env_の順に後ろが勝つ() {
    let d = def(LAYERED);
    let mut files = LoadedEnvFiles::new();
    add_env_file(
        &mut files,
        "p.env",
        content("V=pipeline-file\nP=p\nW=p-file\n"),
    );
    add_env_file(&mut files, "j1.env", content("W=job-file-1\nQ=1\n"));
    add_env_file(&mut files, "j2.env", content("W=job-file-2\n"));
    let env = merge_env(&d, &d.job[0], &files, &ctx(None, 1));
    assert_eq!(get(&env, "V"), Some("job-env"), "job.env が最も強い");
    assert_eq!(
        get(&env, "W"),
        Some("job-file-2"),
        "配列は後ろのファイルが勝つ"
    );
    assert_eq!(get(&env, "P"), Some("p"));
    assert_eq!(get(&env, "Q"), Some("1"));

    // pipeline.env が pipeline の env_file に勝つ(job.env を外した定義)
    let d = def(&LAYERED.replace("[job.env]\nV = \"job-env\"\n", ""));
    let env = merge_env(&d, &d.job[0], &files, &ctx(None, 1));
    assert_eq!(get(&env, "V"), Some("pipeline-env"));
}

#[test]
fn 並びは層の順で完全に同じ名前は後ろの層の位置に移る() {
    let d = def(LAYERED);
    let mut files = LoadedEnvFiles::new();
    add_env_file(&mut files, "p.env", content("V=pf\nP=p\n"));
    add_env_file(&mut files, "j1.env", content("J1=1\n"));
    add_env_file(&mut files, "j2.env", content("P=again\n"));
    let env = merge_env(&d, &d.job[0], &files, &ctx(Some("r1"), 1));
    assert_eq!(
        names(&env),
        // p.env: [V, P] → pipeline.env の V が末尾へ: [P, V] → j1.env: [P, V, J1]
        // → j2.env の P が末尾へ: [V, J1, P] → job.env の V が末尾へ: [J1, P, V]
        [
            "J1",
            "P",
            "V",
            "CONDUCTOR_PIPELINE",
            "CONDUCTOR_JOB_ID",
            "CONDUCTOR_RUN_ID",
            "CONDUCTOR_ATTEMPT"
        ]
    );
}

#[test]
fn 層をまたぐ大文字小文字だけが違う名前は両方が並びに残り後ろが後() {
    let d = def(
        "format_version = 1\n[pipeline]\nname = \"p\"\n[pipeline.env]\nfoo = \"lower\"\n\
         [[job]]\nid = \"a\"\ncommand = \"c\"\n[job.env]\nFOO = \"UPPER\"\n",
    );
    let env = merge_env(&d, &d.job[0], &LoadedEnvFiles::new(), &ctx(None, 1));
    let n = names(&env);
    let pos = |x: &str| n.iter().position(|v| *v == x).unwrap();
    assert!(
        pos("foo") < pos("FOO"),
        "後ろの層の名前が後(Windows は、これで後ろが勝つ): {n:?}"
    );
    assert_eq!(get(&env, "foo"), Some("lower"));
    assert_eq!(get(&env, "FOO"), Some("UPPER"));
}

#[test]
fn 自動の変数は_run_id_がないとき_conductor_run_id_を作らず試行を反映する() {
    let d = def(LAYERED);
    let files = LoadedEnvFiles::new();
    let e1 = merge_env(&d, &d.job[0], &files, &ctx(None, 1));
    assert_eq!(get(&e1, "CONDUCTOR_PIPELINE"), Some("p"));
    assert_eq!(get(&e1, "CONDUCTOR_JOB_ID"), Some("a"));
    assert_eq!(get(&e1, "CONDUCTOR_RUN_ID"), None);
    assert_eq!(get(&e1, "CONDUCTOR_ATTEMPT"), Some("1"));
    let e3 = merge_env(&d, &d.job[0], &files, &ctx(Some("run-7"), 3));
    assert_eq!(get(&e3, "CONDUCTOR_RUN_ID"), Some("run-7"));
    assert_eq!(get(&e3, "CONDUCTOR_ATTEMPT"), Some("3"));
    assert_eq!(get(&e3, "CONDUCTOR_JOB_ID"), Some("a"));
}

#[test]
fn 読み込まれていない_env_file_の記述は何も足さない() {
    let d = def(LAYERED);
    let env = merge_env(&d, &d.job[0], &LoadedEnvFiles::new(), &ctx(None, 1));
    assert_eq!(get(&env, "V"), Some("job-env"));
    assert_eq!(get(&env, "P"), None);
}

#[test]
fn names_は値を含まず_additions_は並びの順() {
    let d = def(LAYERED);
    let mut files = LoadedEnvFiles::new();
    add_env_file(&mut files, "p.env", content(&format!("P={SECRET}\n")));
    let env = merge_env(&d, &d.job[0], &files, &ctx(None, 1));
    let shown = format!("{:?} {:?}", names(&env), env);
    assert!(!shown.contains(SECRET), "{shown}");
    assert!(names(&env).contains(&"P"));
    assert_eq!(env.additions().len(), env.entries.len());
    // 引き継いだ環境(PATH など)の名前は含まれない
    assert!(!names(&env).contains(&"PATH"));
}

const MULTI: &str = "format_version = 1
[pipeline]
name = \"p\"
env_file = [\"p.env\", \"shared.env\"]
[[job]]
id = \"a\"
command = \"c\"
env_file = \"a.env\"
[[job]]
id = \"b\"
command = \"c\"
env_file = [\"b.env\", \"shared.env\", \"a.env\"]
";

#[test]
fn 読む対象の一覧は_pipeline_が全ジョブ_run_が_1_ジョブで重複を除く() {
    let d = def(MULTI);
    assert_eq!(
        paths(&env_file_refs(&d, None)),
        ["p.env", "shared.env", "a.env", "b.env"]
    );
    assert_eq!(
        paths(&env_file_refs(&d, Some("a"))),
        ["p.env", "shared.env", "a.env"]
    );
    assert_eq!(
        paths(&env_file_refs(&d, Some("b"))),
        ["p.env", "shared.env", "b.env", "a.env"]
    );
    assert!(
        env_file_refs(&d, Some("zz")).len() == 2,
        "存在しないジョブは pipeline の分だけ"
    );
    let none = def(
        "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"c\"\nenv_file = []\n",
    );
    assert!(env_file_refs(&none, None).is_empty());
}

const OPTIONAL: &str = "format_version = 1
[pipeline]
name = \"p\"
env_file = [\"p.env\", { path = \"local.env\", required = false }]
[[job]]
id = \"a\"
command = \"c\"
env_file = [{ path = \"opt.env\", required = false }, { path = \"p.env\", required = false }]
[[job]]
id = \"b\"
command = \"c\"
env_file = [\"local.env\", { path = \"opt.env\", required = false }]
";

#[test]
fn 読む対象の一覧は同じパスの文字列を_1_つにまとめ_required_を論理和にする() {
    let d = def(OPTIONAL);
    // 全体: local.env は b が必ず要るとするので必ず要る。opt.env はどこでも要らない。
    assert_eq!(
        env_file_refs(&d, None),
        [
            env_ref("p.env", true),
            env_ref("local.env", true),
            env_ref("opt.env", false),
        ]
    );
    // run a: b の記述は範囲の外なので、local.env は要らないまま。p.env は pipeline が必ず要る。
    assert_eq!(
        env_file_refs(&d, Some("a")),
        [
            env_ref("p.env", true),
            env_ref("local.env", false),
            env_ref("opt.env", false),
        ]
    );
    // run b: 後ろの記述が必ず要るなら、前の要らない記述も必ず要るに変わる(位置は前のまま)。
    assert_eq!(
        env_file_refs(&d, Some("b")),
        [
            env_ref("p.env", true),
            env_ref("local.env", true),
            env_ref("opt.env", false),
        ]
    );
}

#[test]
fn 無くてもよいファイルが無く引き当てに無い記述は何も足さない() {
    let d = def(OPTIONAL);
    let mut files = LoadedEnvFiles::new();
    // p.env だけ読めた(local.env と opt.env は無くて飛ばした)状態
    add_env_file(&mut files, "p.env", content("P=1\n"));
    let env = merge_env(&d, &d.job[0], &files, &ctx(None, 1));
    assert_eq!(
        names(&env),
        [
            "P",
            "CONDUCTOR_PIPELINE",
            "CONDUCTOR_JOB_ID",
            "CONDUCTOR_ATTEMPT"
        ]
    );
    // 読めたファイルは、表で書いた記述からも引ける
    add_env_file(&mut files, "opt.env", content("O=2\n"));
    let env = merge_env(&d, &d.job[0], &files, &ctx(None, 1));
    assert_eq!(get(&env, "O"), Some("2"));
}

#[test]
fn 同じ内容を指す複数の記述は_1_つの内容を共有する() {
    let mut files = LoadedEnvFiles::new();
    let i = files.push_file(content("A=1\n"));
    files.link("x.env", i);
    files.link("./x.env", i);
    assert_eq!(files.files().len(), 1);
    assert_eq!(files.get("x.env"), files.get("./x.env"));
    assert!(files.get("none").is_none());
}
