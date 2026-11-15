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
        !project.state_path().exists(),
        "最後のジョブなので、完了して初期化される"
    );
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
    let state = conductor_runtime::read_state(&project.def_path())
        .expect("読める")
        .expect("ステートがある");
    assert_eq!(state.resume_job_id.as_deref(), Some("fail"));
    assert_eq!(state.job_states[1].status, JobStatus::Error);
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
fn pipeline_は最後まで実行して_初期化する() {
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
    assert!(!project.state_path().exists());
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
