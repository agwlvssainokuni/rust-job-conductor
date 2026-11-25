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

//! 環境変数の注入の流れ(U1。偽の `Runtime`。BR5.2、BR6.1〜BR6.4、BR7.1、BR7.2、BR8.1、BR8.3)。
//!
//! `.env` は、定義ファイルと同じ一時ディレクトリの実ファイルで、読み取りは実際の関数を使う。
//! ジョブの起動だけが偽で、`run_job` に渡された環境の並びを記録する。
//! 消す名前(v0.10.0 の U3)は、偽の `Runtime` の `run_unsets` の記録で確かめる。

mod common;

use common::*;
use conductor_core::{EnvFileError, EnvFileErrorKind, LoadedEnvFiles};

const SECRET: &str = "s3cr3t-VALUE-do-not-print";

const DEF: &str = r#"format_version = 1
[pipeline]
name = "p"
env_file = "p.env"
[pipeline.env]
P_ENV = "pipe"
[[job]]
id = "a"
command = "echo a"
env_file = "a.env"
[job.env]
J_ENV = "ja"
[[job]]
id = "b"
command = "echo b"
env_file = "b.env"
"#;

const DEF_RETRY: &str = r#"format_version = 1
[pipeline]
name = "p"
[[job]]
id = "a"
command = "echo a"
[job.retry]
exit_codes = [75]
max = 2
interval = "1s"
"#;

/// 無くてもよいファイル(`required = false`)を、`[pipeline]` とジョブに書いた定義(v0.10.0 の BR3.1)。
const DEF_OPTIONAL: &str = r#"format_version = 1
[pipeline]
name = "p"
env_file = ["p.env", { path = "local.env", required = false }]
[[job]]
id = "a"
command = "echo a"
env_file = { path = "a-local.env", required = false }
"#;

fn put(h: &Harness, name: &str, text: &str) {
    std::fs::write(h.dir.path().join(name), text).expect("書ける");
}

fn harness_with_files() -> Harness {
    let h = Harness::new(DEF);
    put(&h, "p.env", "P_FILE=pf\n");
    put(&h, "a.env", "A_FILE=af\n");
    put(&h, "b.env", "B_FILE=bf\n");
    h
}

fn env_of(h: &Harness, index: usize) -> Vec<(String, String)> {
    h.runtime.run_envs.borrow()[index].1.clone()
}

fn value<'a>(env: &'a [(String, String)], name: &str) -> Option<&'a str> {
    env.iter().find(|(n, _)| n == name).map(|(_, v)| v.as_str())
}

#[test]
fn pipeline_は_env_の誤りがあるとジョブを起動せずステートにも触れず_64() {
    let mut h = harness_with_files();
    put(&h, "b.env", &format!("BAD {SECRET}\n"));
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 64, "{}", out.err);
    assert!(out.err.contains("b.env:1"), "{}", out.err);
    assert!(!out.err.contains(SECRET) && !out.out.contains(SECRET));
    assert_eq!(h.runtime.run_calls.get(), 0, "ジョブは 1 つも起動されない");
    assert!(!h.state_path().exists(), "ステートを作らない");

    // 既存のステートがあるときも、触れない(バイト列が同じ)
    put(&h, "b.env", "B_FILE=bf\n");
    h.runtime.script("a", &[Exec::Exit(1)]);
    assert_eq!(h.run(&["pipeline", "p"]).code, 1);
    let before = std::fs::read(h.state_path()).expect("ステートがある");
    put(&h, "a.env", "1BAD=x\n");
    let calls = h.runtime.run_calls.get();
    assert_eq!(h.run(&["pipeline", "p"]).code, 64);
    assert_eq!(h.runtime.run_calls.get(), calls);
    assert_eq!(std::fs::read(h.state_path()).unwrap(), before);
}

#[test]
fn run_は_pipeline_の_env_file_とそのジョブの分だけを読む() {
    let mut h = harness_with_files();
    std::fs::remove_file(h.dir.path().join("b.env")).expect("消せる");
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 0, "b.env が無くても a は動く: {}", out.err);
    let loads = h.runtime.env_loads.borrow();
    let paths: Vec<Vec<&str>> = loads
        .iter()
        .map(|refs| refs.iter().map(|r| r.path.as_str()).collect())
        .collect();
    assert_eq!(paths, [vec!["p.env", "a.env"]]);
    drop(loads);
    let env = env_of(&h, 0);
    assert_eq!(value(&env, "P_FILE"), Some("pf"));
    assert_eq!(value(&env, "A_FILE"), Some("af"));
    assert_eq!(value(&env, "B_FILE"), None);

    // pipeline なら、全ジョブの分を読むので、b.env が無いと 64
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 64);
    assert!(out.err.contains("b.env"), "{}", out.err);
}

#[test]
fn 再開でも_env_を読み直す() {
    let mut h = harness_with_files();
    h.runtime.script("a", &[Exec::Exit(1)]);
    assert_eq!(h.run(&["pipeline", "p"]).code, 1);
    put(&h, "p.env", "P_FILE=changed\n");
    assert_eq!(h.run(&["pipeline", "p"]).code, 0);
    let runs = h.runtime.run_envs.borrow().len();
    assert_eq!(runs, 3, "a(失敗)→ a(再開)→ b");
    assert_eq!(value(&env_of(&h, 0), "P_FILE"), Some("pf"));
    assert_eq!(value(&env_of(&h, 1), "P_FILE"), Some("changed"));
    assert_eq!(h.runtime.env_loads.borrow().len(), 2, "呼び出しごとに読む");
}

#[test]
fn 誤りは_20_件で打ち切り_残りの件数を示す() {
    let mut h = harness_with_files();
    let bad: String = (1..=25).map(|i| format!("BAD{i}\n")).collect();
    put(&h, "p.env", &bad);
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 64);
    let error_lines = out.err.lines().filter(|l| l.contains("p.env:")).count();
    assert_eq!(error_lines, 20, "{}", out.err);
    assert!(out.err.contains("p.env:20"));
    assert!(!out.err.contains("p.env:21"));
    assert!(out.err.contains("and 5 more"), "{}", out.err);

    // 日本語
    h.env = FakeEnv::with(&[("LANG", "ja_JP.UTF-8")]);
    let out = h.run(&["pipeline", "p"]);
    assert!(out.err.contains("ほか 5 件"), "{}", out.err);
}

#[test]
fn validate_は誤りで_64_重複キーの警告だけなら_0() {
    let mut h = harness_with_files();
    put(&h, "p.env", &format!("A=1\nB={SECRET}\nA=2\n"));
    let out = h.run(&["validate"]);
    assert_eq!(out.code, 0, "警告だけなら 0: {}", out.err);
    assert!(out.err.contains("warning"), "{}", out.err);
    assert!(out.err.contains("p.env:3"), "{}", out.err);
    assert!(out.err.contains("`A`"));
    assert!(!out.err.contains(SECRET));
    assert_eq!(out.out, "");
    assert_eq!(h.runtime.run_calls.get(), 0);
    assert!(!h.state_path().exists());

    // run と pipeline は、警告を表示しない
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 0);
    assert!(!out.err.contains("warning"), "{}", out.err);

    put(&h, "a.env", "NOEQ\n");
    let out = h.run(&["validate"]);
    assert_eq!(out.code, 64);
    assert!(out.err.contains("a.env:1"), "{}", out.err);
    // 定義の誤りが先に勝つ(`.env` は読まない)
    h.runtime.env_loads.borrow_mut().clear();
    h.set_definition(&DEF.replace("name = \"p\"", "name = \"p\"\nunknown = 1"));
    assert_eq!(h.run(&["validate"]).code, 64);
    assert!(h.runtime.env_loads.borrow().is_empty());
}

#[test]
fn 使い方の誤りの_68_が_env_の誤りの_64_より先に勝つ() {
    let mut h = harness_with_files();
    put(&h, "p.env", "BAD\n");
    let out = h.run(&["run", "nosuchjob"]);
    assert_eq!(out.code, 68, "{}", out.err);
    assert!(h.runtime.env_loads.borrow().is_empty(), "読む前に止まる");
    let out = h.run(&["run", "a", "--run-id", "bad id"]);
    assert_eq!(out.code, 68);
    assert!(h.runtime.env_loads.borrow().is_empty());
}

#[test]
fn リトライで_attempt_が_1_2_3_と増え再開で_1_に戻る() {
    let mut h = Harness::new(DEF_RETRY);
    h.runtime
        .script("a", &[Exec::Exit(75), Exec::Exit(75), Exec::Exit(0)]);
    h.env = FakeEnv::with(&[("CONDUCTOR_RUN_ID", "r9")]);
    assert_eq!(h.run(&["run", "a"]).code, 0);
    let attempts: Vec<String> = (0..3)
        .map(|i| {
            value(&env_of(&h, i), "CONDUCTOR_ATTEMPT")
                .unwrap()
                .to_string()
        })
        .collect();
    assert_eq!(attempts, ["1", "2", "3"]);
    let env = env_of(&h, 2);
    assert_eq!(value(&env, "CONDUCTOR_PIPELINE"), Some("p"));
    assert_eq!(value(&env, "CONDUCTOR_JOB_ID"), Some("a"));
    assert_eq!(value(&env, "CONDUCTOR_RUN_ID"), Some("r9"));

    // 失敗で止まったあとの再開は、また 1 から
    let mut h = Harness::new(DEF_RETRY);
    h.runtime
        .script("a", &[Exec::Exit(75), Exec::Exit(75), Exec::Exit(75)]);
    assert_eq!(h.run(&["run", "a"]).code, 75);
    h.runtime.script("a", &[Exec::Exit(0)]);
    assert_eq!(h.run(&["run", "a"]).code, 0);
    let last = h.runtime.run_envs.borrow().len() - 1;
    assert_eq!(value(&env_of(&h, last), "CONDUCTOR_ATTEMPT"), Some("1"));
    // run ID なしでは CONDUCTOR_RUN_ID を渡さない
    assert_eq!(value(&env_of(&h, last), "CONDUCTOR_RUN_ID"), None);
}

#[test]
fn run_job_に渡る環境の並びは層の順() {
    let mut h = harness_with_files();
    assert_eq!(h.run(&["run", "a"]).code, 0);
    let names: Vec<String> = env_of(&h, 0).into_iter().map(|(n, _)| n).collect();
    assert_eq!(
        names,
        [
            "P_FILE",
            "P_ENV",
            "A_FILE",
            "J_ENV",
            "CONDUCTOR_PIPELINE",
            "CONDUCTOR_JOB_ID",
            "CONDUCTOR_ATTEMPT"
        ]
    );
}

#[test]
fn 値は標準出力_標準エラー_ステートのどこにも出ない() {
    let mut h = harness_with_files();
    put(&h, "p.env", &format!("TOKEN={SECRET}\n"));
    h.set_definition(&DEF.replace("P_ENV = \"pipe\"", &format!("P_ENV = \"{SECRET}\"")));
    h.runtime.script("a", &[Exec::Exit(3)]);
    let first = h.run(&["pipeline", "-v", "p"]);
    h.runtime.script("a", &[Exec::Exit(0)]);
    let second = h.run(&["pipeline", "-v", "p"]);
    let validate = h.run(&["validate", "-v"]);
    for out in [&first, &second, &validate] {
        assert!(!out.out.contains(SECRET) && !out.err.contains(SECRET));
    }
    let state = std::fs::read_to_string(h.state_path()).unwrap_or_default();
    assert!(!state.contains(SECRET) && !state.contains("TOKEN"));
    // 値は、子に渡る環境にだけある
    assert_eq!(value(&env_of(&h, 0), "TOKEN"), Some(SECRET));
}

#[test]
fn 注入した読み取りの誤りも_64_で値を含まずに表示する() {
    let mut h = harness_with_files();
    *h.runtime.env_override.borrow_mut() = Some(Err(vec![
        EnvFileError::new("x.env", None, EnvFileErrorKind::FileMissing),
        EnvFileError::new("y.env", Some(4), EnvFileErrorKind::NulInValue),
    ]));
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 64);
    assert!(
        out.err.contains("x.env:") || out.err.contains("x.env"),
        "{}",
        out.err
    );
    assert!(out.err.contains("y.env:4"), "{}", out.err);
    *h.runtime.env_override.borrow_mut() = Some(Ok(LoadedEnvFiles::new()));
    assert_eq!(h.run(&["pipeline", "p"]).code, 0);
}

#[test]
fn 無くてもよいファイルが無ければ_run_と_pipeline_は_0_でその変数は渡らず何も表示しない() {
    let mut h = Harness::new(DEF_OPTIONAL);
    put(&h, "p.env", "P_FILE=pf\n");
    for args in [&["run", "a"][..], &["pipeline", "p"][..]] {
        let out = h.run(args);
        assert_eq!(out.code, 0, "{args:?}: {}", out.err);
        for shown in [&out.out, &out.err] {
            assert!(!shown.contains("local.env"), "{args:?}: {shown}");
        }
    }
    // 読む対象には、書いた required がそのまま渡る(run と pipeline で同じ入口)
    let loads = h.runtime.env_loads.borrow().clone();
    assert_eq!(loads.len(), 2);
    for refs in &loads {
        let got: Vec<(&str, bool)> = refs.iter().map(|r| (r.path.as_str(), r.required)).collect();
        assert_eq!(
            got,
            [
                ("p.env", true),
                ("local.env", false),
                ("a-local.env", false)
            ]
        );
    }
    let env = env_of(&h, 0);
    assert_eq!(value(&env, "P_FILE"), Some("pf"));
    assert_eq!(value(&env, "LOCAL"), None);

    // 置けば、その変数が後ろの層として渡る
    put(&h, "local.env", "LOCAL=l\nP_FILE=override\n");
    put(&h, "a-local.env", "A_LOCAL=al\n");
    assert_eq!(h.run(&["run", "a"]).code, 0);
    let env = env_of(&h, 2);
    assert_eq!(value(&env, "LOCAL"), Some("l"));
    assert_eq!(value(&env, "P_FILE"), Some("override"));
    assert_eq!(value(&env, "A_LOCAL"), Some("al"));
}

// ---- v0.10.0 の U3: 消す名前(`unset_env`)。BR2.2、BR3.1、BR3.2 ----

/// パイプラインとジョブ `a` に `unset_env` を書き、`.env` を共有する定義。
const DEF_UNSET: &str = r#"format_version = 1
[pipeline]
name = "p"
env_file = "shared.env"
unset_env = ["HTTP_PROXY"]
[[job]]
id = "a"
command = "echo a"
unset_env = ["SECRET_TOKEN", "HTTP_PROXY"]
[[job]]
id = "b"
command = "echo b"
env_file = "b.env"
"#;

fn unsets_of(h: &Harness) -> Vec<(String, Vec<String>)> {
    h.runtime.run_unsets.borrow().clone()
}

#[test]
fn run_job_に消す名前がパイプラインの次にジョブの順で渡る() {
    let mut h = Harness::new(DEF_UNSET);
    put(&h, "shared.env", "S=1\n");
    put(&h, "b.env", "B=1\n");
    assert_eq!(h.run(&["pipeline", "p"]).code, 0);
    assert_eq!(
        unsets_of(&h),
        [
            (
                "a".to_string(),
                vec!["HTTP_PROXY".to_string(), "SECRET_TOKEN".to_string()]
            ),
            ("b".to_string(), vec!["HTTP_PROXY".to_string()]),
        ]
    );
    // 足す変数は今のまま
    assert_eq!(value(&env_of(&h, 0), "S"), Some("1"));
    // unset_env を書かない定義は、空の並びが渡る(BR4.2)
    let mut h = harness_with_files();
    assert_eq!(h.run(&["run", "a"]).code, 0);
    assert_eq!(unsets_of(&h), [("a".to_string(), Vec::<String>::new())]);
}

#[test]
fn env_の名前と消す名前の重なりは_64_で範囲は読み取りの範囲() {
    let mut h = Harness::new(DEF_UNSET);
    put(&h, "shared.env", "S=1\n");
    // b.env の secret_token は、a の消す名前と重なるが、b は a の unset_env の範囲の外
    put(&h, "b.env", &format!("secret_token={SECRET}\n"));
    // run a は b.env を読まないので通る
    assert_eq!(h.run(&["run", "a"]).code, 0);
    // pipeline でも b の消す名前は HTTP_PROXY だけなので通る
    assert_eq!(h.run(&["pipeline", "p"]).code, 0);

    // shared.env の http_proxy は、a と b の両方の消す名前と重なる(大文字小文字を区別しない)
    put(&h, "shared.env", &format!("S=1\nhttp_proxy={SECRET}\n"));
    let calls = h.runtime.run_calls.get();
    for args in [&["run", "a"][..], &["run", "b"][..], &["pipeline", "p"][..]] {
        let out = h.run(args);
        assert_eq!(out.code, 64, "{args:?}: {}", out.err);
        assert!(out.err.contains("shared.env:2"), "{args:?}: {}", out.err);
        assert!(!out.err.contains(SECRET), "{}", out.err);
        assert!(
            !out.err.to_ascii_lowercase().contains("http_proxy"),
            "{}",
            out.err
        );
    }
    assert_eq!(h.runtime.run_calls.get(), calls, "ジョブは起動しない");
}

#[test]
fn 同じ_env_を_2_つのジョブが読んで重なっても誤りは_1_つ() {
    let mut h = Harness::new(DEF_UNSET);
    put(&h, "shared.env", "HTTP_PROXY=x\n");
    put(&h, "b.env", "B=1\n");
    let out = h.run(&["pipeline", "p"]);
    assert_eq!(out.code, 64, "{}", out.err);
    let lines = out
        .err
        .lines()
        .filter(|l| l.contains("shared.env:1"))
        .count();
    assert_eq!(lines, 1, "{}", out.err);
    assert!(
        out.err.contains("the same name is listed in `unset_env`"),
        "{}",
        out.err
    );
    // 日本語
    h.env = FakeEnv::with(&[("LANG", "ja_JP.UTF-8")]);
    let out = h.run(&["pipeline", "p"]);
    assert!(
        out.err.contains("`unset_env` の名前と同じ名前があります"),
        "{}",
        out.err
    );
}

#[test]
fn 同じファイルをどこかで必ず要るとしていれば無いとき_64() {
    let def = DEF_OPTIONAL.replace(
        "{ path = \"a-local.env\", required = false }",
        "[{ path = \"a-local.env\", required = false }, \"local.env\"]",
    );
    let mut h = Harness::new(&def);
    put(&h, "p.env", "P_FILE=pf\n");
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 64, "{}", out.err);
    assert!(out.err.contains("local.env"), "{}", out.err);
    assert!(!out.err.contains("a-local.env"), "{}", out.err);
    assert_eq!(h.runtime.run_calls.get(), 0);
}
