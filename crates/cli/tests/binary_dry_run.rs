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

//! `--dry-run` の、実際のバイナリのテスト(U3。FR3.1〜FR3.4、FR1.12)。ロケールは英語に固定される。

mod bin_support;

use bin_support::*;

const SECRET: &str = "s3cr3t-VALUE-do-not-print";

/// 補助の実行ファイルを呼ぶ、計画の「command」の表示(空白で分けた単語。パスは 1 語)。
fn shown(rest: &str) -> String {
    let words: Vec<String> = std::iter::once(helper_path().to_string())
        .chain(rest.split_whitespace().map(str::to_string))
        .map(|w| format!("\"{w}\""))
        .collect();
    format!("[{}]", words.join(", "))
}

/// a、b、c の定義。b は、終了コード 3 を警告にする。
fn abc_definition() -> String {
    definition(
        "demo",
        &[
            job_block("a", &helper_command("exit 0"), ""),
            job_block("b", &helper_command("exit 0"), "warning_exit_codes = [3]"),
            job_block("c", &helper_command("exit 0"), ""),
        ],
    )
}

const ENV3: &str = "CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_ATTEMPT";

#[test]
fn pipeline_の計画と_assume_の出力が例と一致し_何も作られない() {
    let project = Project::new(&abc_definition());
    let out = project.run(&["pipeline", "demo", "--dry-run"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let command = shown("exit 0");
    let expected = format!(
        "plan (--dry-run): pipeline demo\nresume: from the start\n\
         1. [run] a\n     command: {command}\n     env: {ENV3}\n\
         2. [run] b\n     command: {command}\n     env: {ENV3}\n\
         3. [run] c\n     command: {command}\n     env: {ENV3}\n\
         result (simulated): completed\n"
    );
    assert_eq!(text(&out.stdout), expected);
    assert_eq!(text(&out.stderr), "");
    assert!(!project.state_path().exists());

    let out = project.run(&["pipeline", "demo", "--dry-run", "--assume", "b=3"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let expected = format!(
        "plan (--dry-run): pipeline demo\nresume: from the start\nassume: b=3\n\
         1. [run] a\n     command: {command}\n     env: {ENV3}\n\
         2. [run (assumed: exit code 3 -> warning)] b\n     command: {command}\n     env: {ENV3}\n\
         3. [run] c\n     command: {command}\n     env: {ENV3}\n\
         result (simulated): completed\n"
    );
    assert_eq!(text(&out.stdout), expected);
    assert!(!project.state_path().exists());

    // `run` と、エラーで停止する仮定。
    let out = project.run(&["run", "a", "--dry-run", "--assume", "a=1"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let expected = format!(
        "plan (--dry-run): pipeline demo\ntarget job: a\nassume: a=1\n\
         1. [run (assumed: exit code 1 -> error)] a\n     command: {command}\n     env: {ENV3}\n\
         next step: (assumed: exit code 1 -> error) stop\n"
    );
    assert_eq!(text(&out.stdout), expected);
}

#[test]
fn 実際に作ったステートから再開位置が出て_ステートのバイト列は変わらない() {
    let definition = definition(
        "demo",
        &[
            job_block("a", &helper_command("exit 0"), ""),
            job_block("b", &helper_command("exit 1"), ""),
            job_block("c", &helper_command("exit 0"), ""),
        ],
    );
    let project = Project::new(&definition);
    let run = project.run(&["pipeline", "demo"]);
    assert_eq!(run.status.code(), Some(1), "{}", text(&run.stderr));
    let before = std::fs::read(project.state_path()).expect("ステートがある");

    let out = project.run(&["pipeline", "demo", "--dry-run"]);
    let stdout = text(&out.stdout);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(stdout.contains("resume: b (previous stop)\n"), "{stdout}");
    assert!(stdout.contains("1. [already executed] a\n"), "{stdout}");
    assert!(stdout.contains("2. [run] b  <- resume here\n"), "{stdout}");
    assert!(stdout.contains("3. [run] c\n"), "{stdout}");
    assert_eq!(
        std::fs::read(project.state_path()).expect("残っている"),
        before
    );

    let out = project.run(&["run", "b", "--dry-run"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    // 個別実行は、再開位置を示さず、対象のジョブと次の動きを示す。
    let stdout = text(&out.stdout);
    assert!(stdout.contains("target job: b\n"), "{stdout}");
    assert!(stdout.ends_with("next step: next job c\n"), "{stdout}");
    assert_eq!(
        std::fs::read(project.state_path()).expect("残っている"),
        before
    );
}

#[test]
fn 何も実行されない_ジョブが書くはずのファイルができない() {
    let project = Project::new("format_version = 1\n[pipeline]\nname = \"demo\"\n");
    let marker = project.dir.path().join("ran.txt");
    let definition = definition(
        "demo",
        &[
            job_block("a", &helper_command_out(&marker, "run a 0"), ""),
            job_block("b", &helper_command_out(&marker, "run b 0"), ""),
        ],
    );
    std::fs::write(project.def_path(), definition).expect("書ける");

    for args in [
        &["pipeline", "demo", "--dry-run"][..],
        &["run", "a", "--dry-run"][..],
    ] {
        let out = project.run(args);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{args:?}: {}",
            text(&out.stderr)
        );
        assert!(!marker.exists(), "ジョブが実行された: {args:?}");
        assert!(!project.state_path().exists());
    }
    // 確認: `--dry-run` なしなら、実行される。
    let out = project.run(&["pipeline", "demo"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(marker.exists());
}

#[test]
fn env_と_dotenv_の値は_標準出力と標準エラーのどこにも無く_名前だけが出る() {
    let definition = format!(
        "format_version = 1\n[pipeline]\nname = \"demo\"\nenv_file = \"p.env\"\n[pipeline.env]\nT_PENV = \"{SECRET}\"\n\
         [[job]]\nid = \"a\"\ncommand = '''{}'''\n[job.env]\nT_JENV = \"{SECRET}\"\n",
        helper_command("exit 0")
    );
    let project = Project::new(&definition);
    std::fs::write(
        project.dir.path().join("p.env"),
        format!("T_TOKEN={SECRET}\n"),
    )
    .expect("書ける");
    for args in [
        &["pipeline", "demo", "--dry-run", "-v"][..],
        &["run", "a", "--dry-run", "-v"][..],
        &["run", "a", "--dry-run", "--assume", "a=1", "--run-id", "r1"][..],
    ] {
        let out = project.run(args);
        let (stdout, stderr) = (text(&out.stdout), text(&out.stderr));
        assert_eq!(out.status.code(), Some(0), "{args:?}: {stderr}");
        assert!(
            !stdout.contains(SECRET) && !stderr.contains(SECRET),
            "{args:?}"
        );
        assert!(
            stdout.contains("env: T_TOKEN, T_PENV, T_JENV, CONDUCTOR_PIPELINE"),
            "{stdout}"
        );
    }
    assert!(!project.state_path().exists());
}
