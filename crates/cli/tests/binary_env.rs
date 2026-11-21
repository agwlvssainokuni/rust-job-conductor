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

//! 環境変数の注入の、実際のバイナリのテスト(U1。FR1.1〜FR1.12、NFR2)。
//!
//! 子のプロセスの環境は、補助の実行ファイルの `env <NAME>...` モードで確かめる(標準出力に、
//! `env[NAME]=Some("...")` / `None` を書く)。

mod bin_support;

use std::process::Output;

use bin_support::*;

const SECRET: &str = "s3cr3t-VALUE-do-not-print";

/// 子の標準出力から、`env[NAME]` の値を読む(`None` は、子に見えなかった)。
fn seen(stdout: &str, name: &str) -> Option<Option<String>> {
    let prefix = format!("env[{name}]=");
    let rest = stdout
        .lines()
        .find_map(|l| l.strip_prefix(prefix.as_str()))?;
    if rest == "None" {
        return Some(None);
    }
    let inner = rest.strip_prefix("Some(\"")?.strip_suffix("\")")?;
    Some(Some(inner.replace("\\\"", "\"").replace("\\\\", "\\")))
}

fn value(stdout: &str, name: &str) -> Option<String> {
    seen(stdout, name).unwrap_or_else(|| panic!("{name} の行がない: {stdout}"))
}

fn put(project: &Project, name: &str, text: &str) {
    std::fs::write(project.dir.path().join(name), text).expect("書ける");
}

/// 定義(`pipeline_extra` は `[pipeline]` の追加の行)。ジョブ `show` が、`names` を子の環境から読む。
fn definition_with(pipeline_extra: &str, job_extra: &str, names: &str) -> String {
    format!(
        "format_version = 1\n[pipeline]\nname = \"p\"\n{pipeline_extra}\n\
         [[job]]\nid = \"show\"\ncommand = '''{}'''\n{job_extra}\n",
        helper_command(&format!("env {names}"))
    )
}

fn run_clean(project: &Project, args: &[&str]) -> Output {
    let mut command = conductor(args);
    command
        .arg("--config")
        .arg(project.def_path())
        .env_remove("CONDUCTOR_RUN_ID")
        .env("T_INHERITED", "from-parent")
        .env("T_OVERRIDDEN", "from-parent");
    run_limited(command, || {})
}

#[test]
fn env_と_env_file_の値と優先順位が子のプロセスの環境に見える() {
    let project = Project::new(&definition_with(
        "env_file = \"p.env\"\n[pipeline.env]\nLAYER = \"pipeline-env\"\nT_PENV = \"pe\"\n",
        "env_file = [\"j1.env\", \"j2.env\"]\n[job.env]\nLAYER = \"job-env\"\nT_JENV = \"je w\"\n",
        "LAYER T_PFILE T_PENV T_J1 T_J2 T_JENV T_SHARED T_QUOTED T_NONE T_INHERITED T_OVERRIDDEN",
    ));
    // 優先順位(後ろが勝つ): .env(p) < [pipeline.env] < .env(job、配列の順) < [job.env]
    put(
        &project,
        "p.env",
        "LAYER=pipeline-file\nT_PFILE=pf\nT_SHARED=from-p\nT_OVERRIDDEN=from-env-file\n",
    );
    put(&project, "j1.env", "LAYER=j1\nT_J1=one\nT_SHARED=from-j1\n");
    put(
        &project,
        "j2.env",
        "export T_J2=two\nT_SHARED=from-j2\nT_QUOTED=\"a b\\n\"\n",
    );

    let output = run_clean(&project, &["run", "show"]);
    let out = text(&output.stdout);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    assert_eq!(value(&out, "LAYER").as_deref(), Some("job-env"));
    assert_eq!(value(&out, "T_PFILE").as_deref(), Some("pf"));
    assert_eq!(value(&out, "T_PENV").as_deref(), Some("pe"));
    assert_eq!(value(&out, "T_J1").as_deref(), Some("one"));
    assert_eq!(value(&out, "T_J2").as_deref(), Some("two"));
    assert_eq!(value(&out, "T_JENV").as_deref(), Some("je w"));
    assert_eq!(
        value(&out, "T_SHARED").as_deref(),
        Some("from-j2"),
        "配列は後ろが勝つ"
    );
    assert_eq!(
        value(&out, "T_QUOTED").as_deref(),
        Some("a b\\n"),
        "改行は 1 行に出る"
    );
    assert_eq!(value(&out, "T_NONE"), None);
    // 引き継いだ環境は残り、`.env` が上書きできる
    assert_eq!(value(&out, "T_INHERITED").as_deref(), Some("from-parent"));
    assert_eq!(
        value(&out, "T_OVERRIDDEN").as_deref(),
        Some("from-env-file")
    );
}

#[test]
fn 自動の変数が_run_id_の有無で渡る() {
    let project = Project::new(&definition_with(
        "",
        "",
        "CONDUCTOR_PIPELINE CONDUCTOR_JOB_ID CONDUCTOR_RUN_ID CONDUCTOR_ATTEMPT",
    ));
    let with_id = text(&run_clean(&project, &["pipeline", "p", "--run-id", "r-42"]).stdout);
    assert_eq!(value(&with_id, "CONDUCTOR_PIPELINE").as_deref(), Some("p"));
    assert_eq!(value(&with_id, "CONDUCTOR_JOB_ID").as_deref(), Some("show"));
    assert_eq!(value(&with_id, "CONDUCTOR_RUN_ID").as_deref(), Some("r-42"));
    assert_eq!(value(&with_id, "CONDUCTOR_ATTEMPT").as_deref(), Some("1"));

    let without = text(&run_clean(&project, &["run", "show"]).stdout);
    assert_eq!(value(&without, "CONDUCTOR_JOB_ID").as_deref(), Some("show"));
    assert_eq!(
        value(&without, "CONDUCTOR_RUN_ID"),
        None,
        "run ID がなければ渡さない"
    );
    assert_eq!(value(&without, "CONDUCTOR_ATTEMPT").as_deref(), Some("1"));
}

#[test]
fn env_の誤りは_64_で値が出力に無くジョブもステートも作らない() {
    let project = Project::new(&definition_with(
        "env_file = [\"ok.env\", \"bad.env\", \"missing.env\"]\n",
        "",
        "ANY",
    ));
    put(&project, "ok.env", &format!("GOOD={SECRET}\n"));
    put(
        &project,
        "bad.env",
        &format!("NOEQ {SECRET}\nT_OK=1\n1BAD={SECRET}\nQ=\"{SECRET}\\q\"\n"),
    );
    for args in [
        &["pipeline", "p"][..],
        &["run", "show"][..],
        &["validate"][..],
    ] {
        let output = run_clean(&project, args);
        let (out, err) = (text(&output.stdout), text(&output.stderr));
        assert_eq!(output.status.code(), Some(64), "{args:?}: {err}");
        assert!(out.is_empty(), "ジョブが動いていない: {out}");
        assert!(err.contains("bad.env:1"), "{err}");
        assert!(
            err.contains("bad.env:3") && err.contains("bad.env:4"),
            "{err}"
        );
        assert!(err.contains("missing.env"), "{err}");
        assert!(!err.contains(SECRET), "値が出ている: {err}");
        assert!(!project.state_path().exists());
    }
}

#[test]
fn validate_は重複キーの警告を標準エラーに出して_0() {
    let project = Project::new(&definition_with("env_file = \"p.env\"\n", "", "ANY"));
    put(&project, "p.env", &format!("A=1\nB={SECRET}\nA=2\n"));
    let output = run_clean(&project, &["validate"]);
    let err = text(&output.stderr);
    assert_eq!(output.status.code(), Some(0), "{err}");
    assert!(text(&output.stdout).is_empty());
    assert!(
        err.contains("warning") && err.contains("p.env:3") && err.contains("`A`"),
        "{err}"
    );
    assert!(!err.contains(SECRET));
    assert!(!project.state_path().exists());
}

#[test]
fn 秘密の値は標準出力_標準エラー_ステートファイルに含まれない() {
    // 値を表示しないジョブ(`exit 3`)で、失敗のステートも書かせる
    let project = Project::new(&format!(
        "format_version = 1\n[pipeline]\nname = \"p\"\nenv_file = \"p.env\"\n[pipeline.env]\nT_PENV = \"{SECRET}\"\n\
         [[job]]\nid = \"quiet\"\ncommand = '''{}'''\n[job.env]\nT_JENV = \"{SECRET}\"\n",
        helper_command("exit 3")
    ));
    put(&project, "p.env", &format!("T_TOKEN={SECRET}\n"));
    for args in [
        &["pipeline", "-v", "p"][..],
        &["run", "-v", "quiet"][..],
        &["validate", "-v"][..],
    ] {
        let output = run_clean(&project, args);
        let (out, err) = (text(&output.stdout), text(&output.stderr));
        assert!(
            !out.contains(SECRET) && !err.contains(SECRET),
            "{args:?}: {out}{err}"
        );
    }
    let state = std::fs::read_to_string(project.state_path()).expect("失敗のステートがある");
    assert!(!state.contains(SECRET) && !state.contains("T_TOKEN") && !state.contains("T_PENV"));
}
