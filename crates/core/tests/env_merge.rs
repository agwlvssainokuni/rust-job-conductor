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
//! 消す名前と `.env` との重なり(v0.10.0 の U3。BR2.2、BR3.1)。

mod common;

use common::{SECRET, add_env_file, env_ref, paths};
use conductor_core::{
    EffectiveEnv, EnvContext, EnvFileContent, EnvFileError, EnvFileErrorKind, LoadedEnvFiles,
    PipelineDefinition, check_unset_conflicts, env_file_refs, merge_env, parse_definition,
    parse_env_file,
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

// ---- v0.10.0 の U3: 消す名前(`removals`)と `.env` との重なり(BR2.2、BR3.1) ----

#[test]
fn removals_はパイプラインの次にジョブの順で完全に同じ名前は最初の_1_つ() {
    let d = def(
        "format_version = 1\n[pipeline]\nname = \"p\"\nunset_env = [\"A\", \"HTTP_PROXY\"]\n\
                 [[job]]\nid = \"a\"\ncommand = \"c\"\nunset_env = [\"B\", \"A\", \"a\"]\n",
    );
    let env = merge_env(&d, &d.job[0], &LoadedEnvFiles::new(), &ctx(None, 1));
    assert_eq!(env.removals, ["A", "HTTP_PROXY", "B", "a"]);
}

#[test]
fn unset_env_を書かなければ_removals_は空で_entries_は今と同じ() {
    // BR4.2、契約 C1: 足す変数の形は変えない
    let d = def(LAYERED);
    let mut files = LoadedEnvFiles::new();
    add_env_file(&mut files, "p.env", content("P=1\n"));
    let env = merge_env(&d, &d.job[0], &files, &ctx(None, 1));
    assert!(env.removals.is_empty());
    // unset_env を足した定義でも、足す変数の並びと値は同じ
    let with_unset = def(&LAYERED.replace(
        "command = \"c\"\n",
        "command = \"c\"\nunset_env = [\"HTTP_PROXY\"]\n",
    ));
    let env2 = merge_env(&with_unset, &with_unset.job[0], &files, &ctx(None, 1));
    assert_eq!(env2.entries, env.entries);
    assert_eq!(env2.removals, ["HTTP_PROXY"]);
}

const UNSET_FILES: &str = "format_version = 1
[pipeline]
name = \"p\"
env_file = \"shared.env\"
[[job]]
id = \"a\"
command = \"c\"
unset_env = [\"foo\"]
[[job]]
id = \"b\"
command = \"c\"
env_file = \"b.env\"
unset_env = [\"FOO\", \"B_ONLY\"]
";

fn unset_files() -> LoadedEnvFiles {
    let mut files = LoadedEnvFiles::new();
    let i = files.push_file(parse_env_file("X=1\nFOO=2\n", "/abs/shared.env").unwrap());
    files.link("shared.env", i);
    let i = files.push_file(parse_env_file("# c\nB_ONLY=1\nY=2\n", "/abs/b.env").unwrap());
    files.link("b.env", i);
    files
}

fn shown(errs: &[EnvFileError]) -> Vec<(String, Option<usize>, EnvFileErrorKind)> {
    errs.iter()
        .map(|e| (e.file.clone(), e.line, e.kind))
        .collect()
}

#[test]
fn env_との重なりはファイルと行の誤りで同じファイルと行は_1_つにまとめる() {
    let d = def(UNSET_FILES);
    let files = unset_files();
    // 全ジョブ: shared.env の FOO は a と b の両方で重なるが 1 つ。b.env の B_ONLY は b で重なる。
    let errs = check_unset_conflicts(&d, None, &files);
    assert_eq!(
        shown(&errs),
        [
            (
                "/abs/shared.env".into(),
                Some(2),
                EnvFileErrorKind::UnsetConflict
            ),
            (
                "/abs/b.env".into(),
                Some(2),
                EnvFileErrorKind::UnsetConflict
            ),
        ]
    );
    // 表示に値も名前も出ない(理由の文は固定)
    for e in &errs {
        let text = e.to_string();
        assert!(
            text.contains("`unset_env` の名前と同じ名前があります"),
            "{text}"
        );
        assert!(!text.contains("FOO") && !text.contains("B_ONLY"), "{text}");
    }
}

#[test]
fn 重なりの範囲は_run_ならそのジョブだけ() {
    let d = def(UNSET_FILES);
    let files = unset_files();
    // a は shared.env だけを読み、消す名前は foo(大文字小文字を区別しないで FOO と重なる)
    assert_eq!(
        shown(&check_unset_conflicts(&d, Some("a"), &files)),
        [(
            "/abs/shared.env".into(),
            Some(2),
            EnvFileErrorKind::UnsetConflict
        )]
    );
    assert_eq!(check_unset_conflicts(&d, Some("b"), &files).len(), 2);
    assert!(check_unset_conflicts(&d, Some("zz"), &files).is_empty());
}

#[test]
fn パイプラインの_unset_env_は全ジョブの_env_file_と比べ_重ならなければ空() {
    let d = def(
        "format_version = 1\n[pipeline]\nname = \"p\"\nunset_env = [\"Y\"]\n\
                 [[job]]\nid = \"a\"\ncommand = \"c\"\n\
                 [[job]]\nid = \"b\"\ncommand = \"c\"\nenv_file = \"b.env\"\n",
    );
    let files = unset_files();
    assert_eq!(
        shown(&check_unset_conflicts(&d, None, &files)),
        [(
            "/abs/b.env".into(),
            Some(3),
            EnvFileErrorKind::UnsetConflict
        )]
    );
    assert!(check_unset_conflicts(&d, Some("a"), &files).is_empty());
    // unset_env が無い定義と、読めていないファイルは何も返さない
    assert!(check_unset_conflicts(&def(LAYERED), None, &files).is_empty());
    assert!(check_unset_conflicts(&d, None, &LoadedEnvFiles::new()).is_empty());
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
