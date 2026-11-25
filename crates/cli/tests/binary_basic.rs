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

//! バイナリを起動するテスト(少数)。`env!("CARGO_BIN_EXE_conductor")` で、終了コードと、
//! 標準出力・標準エラーの分け方を確かめる(AC2.3.x、AC2.4.x、AC5.1.x、AC7.1.x)。

mod bin_support;

use bin_support::*;
use conductor_core::JobStatus;

#[test]
fn version_は_0_で版を標準出力に出す() {
    let output = run_limited(conductor(&["--version"]), || {});
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        text(&output.stdout),
        format!("conductor {}\n", env!("CARGO_PKG_VERSION"))
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn 使い方の誤りは_68_で標準エラーに使い方を出す() {
    for args in [&["bogus"][..], &[][..], &["run"][..], &["watch", "-v"][..]] {
        let output = run_limited(conductor(args), || {});
        assert_eq!(output.status.code(), Some(68), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        assert!(text(&output.stderr).contains("Usage:"), "{args:?}");
    }
}

#[test]
fn 成功は_0_で_標準出力も標準エラーも空() {
    let project = Project::new(&definition(
        "p",
        &[job_block("a", &helper_command("exit 0"), "")],
    ));
    let output = project.run(&["run", "a"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty(), "{}", text(&output.stdout));
    assert!(output.stderr.is_empty(), "{}", text(&output.stderr));
    assert!(
        project.state_path().exists(),
        "最後のジョブなので完了するが、完了のステートは残る"
    );
    let state = conductor_runtime::read_state_raw(&project.def_path(), None)
        .expect("読める")
        .expect("完了のステートがある");
    assert!(state.completed);
}

#[test]
fn 警告は_0_スキップは_0_エラーはジョブの値() {
    let project = Project::new(&definition(
        "p",
        &[
            job_block(
                "warn",
                &helper_command("exit 3"),
                "warning_exit_codes = [3]",
            ),
            job_block("fail", &helper_command("exit 7"), ""),
            job_block("last", &helper_command("exit 0"), ""),
        ],
    ));
    // 警告
    let output = project.run(&["run", "warn"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
    // スキップ(実行済み)
    let output = project.run(&["run", "warn"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
    // エラー: ジョブの終了コード
    let output = project.run(&["run", "fail"]);
    assert_eq!(output.status.code(), Some(7));
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
    let state = conductor_runtime::read_state(&project.def_path(), None)
        .expect("読める")
        .expect("ステートがある");
    assert_eq!(state.resume_job_id.as_deref(), Some("fail"));
    assert_eq!(state.job_states[1].status, JobStatus::Error);
}

#[test]
fn validate_は正しい定義で_0_誤りで_64_ステートを作らず_壊れたステートでも_0() {
    let project = Project::new(&definition(
        "p",
        &[job_block("a", &helper_command("exit 0"), "")],
    ));
    let output = project.run(&["validate"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
    assert!(!project.state_path().exists(), "ステートを作らない");

    std::fs::write(project.state_path(), "garbage = = =").expect("書ける");
    let output = project.run(&["validate", "p"]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));

    let bad = Project::new(&definition("p", &[job_block("a", "echo 'unclosed", "")]));
    let output = bad.run(&["validate"]);
    assert_eq!(output.status.code(), Some(64));
    assert!(output.stdout.is_empty());
    assert!(text(&output.stderr).starts_with("conductor: error: "));
    assert!(!bad.state_path().exists());
}

#[test]
fn validate_は既定のリトライとの重なりで_64_になり_値を出さない() {
    // 既定(`[pipeline.retry]`)の 173 が、`retry` を書かないジョブの警告の終了コードと重なる
    // (u1-pipeline-retry の BR3.1、NFR2.2、NFR6.1)。ジョブ ID と終了コードは目印。
    const ID: &str = "job_id_marker_7q";
    let project = Project::new(&format!(
        "format_version = 1\n[pipeline]\nname = \"p\"\n[pipeline.retry]\nexit_codes = [173]\nmax = 1\ninterval = \"1s\"\n[[job]]\nid = \"{ID}\"\ncommand = \"echo x\"\nwarning_exit_codes = [173]\n"
    ));
    let output = project.run(&["validate"]);
    assert_eq!(output.status.code(), Some(64));
    assert!(output.stdout.is_empty());
    let err = text(&output.stderr);
    assert!(err.starts_with("conductor: error: "), "{err}");
    assert_eq!(err.lines().count(), 1, "{err}");
    // 一時ディレクトリの名前に目印の数字が偶然入っても誤らないよう、定義のパスを除いて調べる。
    let shown = err.replace(&project.def_path().display().to_string(), "<path>");
    assert!(!shown.contains(ID), "ジョブ ID を出さない: {err}");
    assert!(!shown.contains("173"), "終了コードの値を出さない: {err}");
    assert!(!project.state_path().exists(), "ステートを作らない");
}

#[test]
fn ジョブを起動できないと_127() {
    let project = Project::new(&definition(
        "p",
        &[job_block("a", "/no/such/program-for-conductor-test", "")],
    ));
    let output = project.run(&["run", "a"]);
    assert_eq!(output.status.code(), Some(127));
    assert!(
        text(&output.stderr).contains("cannot start command"),
        "{}",
        text(&output.stderr)
    );
    assert!(output.stdout.is_empty());
}

#[test]
fn 定義の誤りは_64() {
    let project = Project::new("format_version = 1\n");
    let output = project.run(&["run", "a"]);
    assert_eq!(output.status.code(), Some(64));
    assert!(text(&output.stderr).starts_with("conductor: error: "));
    assert!(output.stdout.is_empty());
}

#[test]
fn ジョブの出力は_そのまま通り_conductor_の表示が混ざらない() {
    let project = Project::new(&definition(
        "p",
        &[job_block(
            "a",
            &helper_command("say 'OUT to stdout' 'ERR to stderr' 0"),
            "",
        )],
    ));
    let output = project.run(&["run", "a"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(text(&output.stdout), "OUT to stdout");
    assert_eq!(text(&output.stderr), "ERR to stderr");
}

#[test]
fn v_は標準エラーだけに出し_標準出力はジョブのまま() {
    let project = Project::new(&definition(
        "p",
        &[job_block("a", &helper_command("say hello ''"), "")],
    ));
    let output = project.run(&["run", "-v", "a"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(text(&output.stdout), "hello");
    assert_eq!(
        text(&output.stderr),
        "[conductor] job=a result=success (exit code 0)\n"
    );
}

#[test]
fn pipeline_は最後まで実行して_完了のステートを残し_watch_が完了を示す() {
    let project = Project::new(&definition(
        "p",
        &[
            job_block("a", &helper_command("exit 0"), ""),
            job_block("b", &helper_command("exit 0"), ""),
        ],
    ));
    let output = project.run(&["pipeline", "p"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
    assert!(project.state_path().exists(), "完了のステートが残る");

    let output = project.run(&["watch"]);
    assert_eq!(output.status.code(), Some(0));
    let shown = text(&output.stdout);
    assert!(shown.starts_with("Pipeline: p\n"), "{shown}");
    assert!(shown.ends_with("completed\n"), "{shown}");
    assert!(!shown.contains("no state"), "{shown}");

    // 再実行は先頭から始まる
    let output = project.run(&["pipeline", "p"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(project.state_path().exists());
}

#[test]
fn fr2_1_pipeline_の_remove_on_complete_は完了でファイルを消す() {
    let project = Project::new(&definition(
        "p",
        &[
            job_block("a", &helper_command("exit 0"), ""),
            job_block("b", &helper_command("exit 0"), ""),
        ],
    ));
    let output = project.run(&["pipeline", "p", "--remove-on-complete"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
    assert!(!project.state_path().exists());
    let output = project.run(&["watch"]);
    assert!(text(&output.stdout).contains("no state"));
}

#[test]
fn fr2_1_run_の_remove_on_complete_も完了でファイルを消す() {
    let project = Project::new(&definition(
        "p",
        &[job_block("a", &helper_command("exit 0"), "")],
    ));
    let output = project.run(&["run", "a", "--remove-on-complete"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(!project.state_path().exists());
}

#[test]
fn fr2_5_remove_on_complete_は_watch_で_68() {
    let output = run_limited(conductor(&["watch", "--remove-on-complete"]), || {});
    assert_eq!(output.status.code(), Some(68));
    assert!(output.stdout.is_empty());
    assert!(
        text(&output.stderr).contains("--remove-on-complete"),
        "{}",
        text(&output.stderr)
    );
}

#[test]
fn fr6_1_version_は_0_8_0() {
    let output = run_limited(conductor(&["--version"]), || {});
    assert_eq!(text(&output.stdout), "conductor 0.9.9\n");
}

/// 補助の実行ファイルを、`command` の配列で呼ぶジョブ(`run <label> <code>`)。
fn array_job_block(id: &str, label: &str, code: i32, out: &std::path::Path) -> String {
    format!(
        "[[job]]\nid = \"{id}\"\ncommand = ['{}', '--out', '{}', 'run', '{label}', '{code}']\n",
        helper_path(),
        out.display()
    )
}

#[test]
fn 配列の_command_は_run_と_pipeline_で文字列と同じ結果になる() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let out_line = dir.path().join("line.txt");
    let out_argv = dir.path().join("argv.txt");

    // 文字列の定義と配列の定義で、終了コードと表示が同じ。
    for (form, out) in [("line", &out_line), ("argv", &out_argv)] {
        let jobs = if form == "line" {
            vec![
                job_block("a", &helper_command_out(out, "run a 0"), ""),
                job_block("b", &helper_command_out(out, "run b 7"), ""),
            ]
        } else {
            vec![
                array_job_block("a", "a", 0, out),
                array_job_block("b", "b", 7, out),
            ]
        };
        let project = Project::new(&definition("p", &jobs));
        let output = project.run(&["run", "a"]);
        assert_eq!(output.status.code(), Some(0), "{form}");
        let output = project.run(&["pipeline", "p"]);
        assert_eq!(output.status.code(), Some(7), "{form}");
        assert!(
            output.stdout.is_empty() && output.stderr.is_empty(),
            "{form}"
        );
    }
    let ran = |p: &std::path::Path| std::fs::read_to_string(p).expect("出力がある");
    assert_eq!(ran(&out_line), ran(&out_argv));
    assert_eq!(ran(&out_argv), "ran=a\nran=b\n");
}

#[test]
fn convert_は変換結果だけを標準出力に出す() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let input = dir.path().join("in.yaml");
    std::fs::write(
        &input,
        "format_version: 1\npipeline:\n  name: p\njob:\n  - id: a\n    command: echo a\n",
    )
    .expect("書ける");
    let mut command = conductor(&["convert"]);
    command.arg(&input).args(["--to", "toml"]);
    let output = run_limited(command, || {});
    assert_eq!(output.status.code(), Some(0));
    let out = text(&output.stdout);
    assert!(
        out.contains("[[job]]") && out.contains("format_version = 1"),
        "{out}"
    );
    assert!(output.stderr.is_empty());

    // 失敗: 標準出力は空、67
    std::fs::write(&input, "a: [unclosed\n").expect("書ける");
    let mut command = conductor(&["convert"]);
    command.arg(&input).args(["--to", "toml"]);
    let output = run_limited(command, || {});
    assert_eq!(output.status.code(), Some(67));
    assert!(output.stdout.is_empty());
    assert!(text(&output.stderr).starts_with("conductor: error: "));
}

#[test]
fn watch_をファイルに出力しても_制御文字がない() {
    let project = Project::new(&definition(
        "p",
        &[
            job_block("a", &helper_command("exit 0"), ""),
            job_block("b", &helper_command("exit 0"), ""),
        ],
    ));
    assert_eq!(project.run(&["run", "a"]).status.code(), Some(0));
    let file_path = project.dir.path().join("watch.out");
    let file = std::fs::File::create(&file_path).expect("作れる");
    let mut command = conductor(&["watch", "--config"]);
    command.arg(project.def_path());
    command.stdout(file);
    // `run_limited` は標準出力をパイプにするため、ここでは直接起動して、上限つきで待つ
    let status = {
        let mut child = command
            .stdin(std::process::Stdio::null())
            .spawn()
            .expect("起動できる");
        let started = std::time::Instant::now();
        loop {
            if let Some(status) = child.try_wait().expect("待てる") {
                break status;
            }
            if started.elapsed() > LIMIT {
                let _ = child.kill();
                let _ = child.wait();
                panic!("時間の上限(30 秒)を超えました");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    };
    assert_eq!(status.code(), Some(0));
    let content = std::fs::read(&file_path).expect("読める");
    let content = text(&content);
    assert!(!content.contains('\x1b'), "{content:?}");
    assert!(
        content.contains("a") && content.contains("success") && content.contains("pending"),
        "{content}"
    );
}

#[test]
fn watch_と_reset_は定義ファイルがなくても動く() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let missing = dir.path().join("missing.toml");
    let mut command = conductor(&["watch", "--config"]);
    command.arg(&missing);
    let output = run_limited(command, || {});
    assert_eq!(output.status.code(), Some(0));
    assert!(text(&output.stdout).contains("no state"));
    let mut command = conductor(&["reset", "--config"]);
    command.arg(&missing);
    let output = run_limited(command, || {});
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
}

// ---- 試行の時刻と所要時間(v0.10.0 の U6。FR6.1〜FR6.8) ----

#[test]
fn u6_fr6_1_ステートに試行の始まりと終わりの時刻が書かれる() {
    let project = Project::new(&definition(
        "p",
        &[
            job_block("a", &helper_command("exit 0"), ""),
            job_block("b", &helper_command("exit 3"), "warning_exit_codes = [3]"),
        ],
    ));
    let before = conductor_runtime::now();
    let output = project.run(&["pipeline", "p"]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    let after = conductor_runtime::now();

    let body = std::fs::read_to_string(project.state_path()).expect("読める");
    assert_eq!(body.matches("[[job.attempt]]").count(), 2, "{body}");
    let state = conductor_runtime::read_state_raw(&project.def_path(), None)
        .expect("読める")
        .expect("完了のステートがある");
    for job in &state.job_states {
        assert_eq!(job.attempts.len(), 1, "{}", job.job_id);
        let attempt = job.attempts[0];
        let ended = attempt.ended_at.expect("終わりがある");
        assert!(before <= attempt.started_at && attempt.started_at <= ended);
        assert!(ended <= after, "{}", job.job_id);
        // 書いた形は秒までの UTC(`Z`)。
        let started_text = conductor_core::format_rfc3339(attempt.started_at);
        assert!(
            body.contains(&format!("started_at = \"{started_text}\"")),
            "{body}"
        );
    }
}

#[test]
fn u6_fr6_4_古い形のステートは_65_で_reset_を案内し_reset_の後は動く() {
    let project = Project::new(&definition(
        "p",
        &[
            job_block("a", &helper_command("exit 0"), ""),
            job_block("b", &helper_command("exit 0"), ""),
        ],
    ));
    // v0.9.9 までの形のステート(a は成功で、試行の記録が無い)。
    std::fs::write(
        project.state_path(),
        "format_version = 1\ndefinition_key = \"conductor.toml\"\ndefinition_path = \"conductor.toml\"\n\
         pipeline_name = \"p\"\njob_order = [\"a\", \"b\"]\nresume_job_id = \"b\"\n\
         end_reached = false\ncompleted = false\n\n\
         [[job]]\nid = \"a\"\nstatus = \"success\"\nexit_code = 0\n\n\
         [[job]]\nid = \"b\"\nstatus = \"pending\"\n",
    )
    .expect("書ける");
    for args in [&["run", "b"][..], &["pipeline", "p"][..], &["watch"][..]] {
        let output = project.run(args);
        assert_eq!(output.status.code(), Some(65), "{args:?}");
        let err = text(&output.stderr);
        assert!(err.contains("state file is corrupt"), "{args:?}: {err}");
        assert!(err.contains("conductor reset"), "{args:?}: {err}");
        assert!(output.stdout.is_empty(), "{args:?}");
    }
    assert_eq!(project.run(&["reset"]).status.code(), Some(0));
    assert_eq!(project.run(&["pipeline", "p"]).status.code(), Some(0));
}

#[test]
fn u6_fr6_5_watch_の表に所要時間の列がある() {
    let project = Project::new(&definition(
        "p",
        &[
            job_block("a", &helper_command("exit 0"), ""),
            job_block("b", &helper_command("exit 0"), ""),
        ],
    ));
    assert_eq!(project.run(&["run", "a"]).status.code(), Some(0));
    let output = project.run(&["watch"]);
    assert_eq!(output.status.code(), Some(0));
    let shown = text(&output.stdout);
    let lines: Vec<&str> = shown.lines().collect();
    assert_eq!(
        lines[1].split_whitespace().collect::<Vec<_>>(),
        ["JOB", "STATUS", "DURATION", "NOTE"]
    );
    // a は終わったジョブ。所要時間は秒の形(短いジョブなので 1 分未満)。
    let a: Vec<&str> = lines[2].split_whitespace().collect();
    assert_eq!(a[..2], ["a", "success"], "{shown}");
    assert!(
        a[2].ends_with('s') && a[2][..a[2].len() - 1].bytes().all(|b| b.is_ascii_digit()),
        "{shown}"
    );
    // b は動いていないので空。
    assert_eq!(lines[3].trim_end(), "b    pending", "{shown}");

    let mut command = conductor(&["watch", "--config"]);
    command.arg(project.def_path()).env("LANG", "ja_JP.UTF-8");
    let output = run_limited(command, || {});
    assert!(
        text(&output.stdout).contains("所要時間"),
        "{}",
        text(&output.stdout)
    );
}

#[test]
fn 日本語のロケールでは_エラーが日本語になる() {
    let mut command = conductor(&["bogus"]);
    command.env("LANG", "ja_JP.UTF-8");
    let output = run_limited(command, || {});
    assert_eq!(output.status.code(), Some(68));
    assert!(
        text(&output.stderr).contains("知らないコマンドです"),
        "{}",
        text(&output.stderr)
    );
}
