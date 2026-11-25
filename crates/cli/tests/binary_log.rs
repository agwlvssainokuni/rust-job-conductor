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

//! ジョブの出力をファイルにも残す(`[pipeline] log_dir`。v0.10.0 の U5。FR5.1〜FR5.5、BR3.x、
//! BR4.x、BR5.1、BR5.4、NFR1.3、NFR1.8、NFR1.9)の、実際のバイナリのテスト。
//!
//! ジョブの出力は `conductor` の標準出力と標準エラーにも出るので、端末の側の中身と、ファイルの中身の
//! 両方を確かめる。

mod bin_support;

use std::fs;
use std::path::{Path, PathBuf};

use bin_support::*;

/// 定義(`pipeline_extra` は `[pipeline]` の追加の行)。
fn definition_with(pipeline_extra: &str, jobs: &[String]) -> String {
    format!(
        "format_version = 1\n[pipeline]\nname = \"p\"\n{pipeline_extra}\n\n{}",
        jobs.join("\n")
    )
}

/// ディレクトリの中のファイルの名前(並べ替え済み)。
fn names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .expect("読める")
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// 名前 `<stem>.<YYYYMMDDTHHMMSSZ>.<試行>.log` の時刻の部分(形が違えば `None`)。
fn time_part<'a>(name: &'a str, stem: &str, attempt: &str) -> Option<&'a str> {
    let rest = name.strip_prefix(stem)?.strip_prefix('.')?;
    let time = rest.strip_suffix(&format!(".{attempt}.log"))?;
    let b = time.as_bytes();
    let ok = b.len() == 16
        && b[8] == b'T'
        && b[15] == b'Z'
        && b[..8].iter().chain(&b[9..15]).all(u8::is_ascii_digit);
    ok.then_some(time)
}

/// 補助の `flood` が書く並び(64 バイトごとに改行する英小文字)。
fn flood_bytes(total: usize) -> Vec<u8> {
    (0..total)
        .map(|i| match i % 64 {
            63 => b'\n',
            r => b'a' + (r % 26) as u8,
        })
        .collect()
}

#[test]
fn fr5_2_ファイルの中身はジョブの出力で_名前は安全な_id_時刻_試行の番号() {
    let project = Project::new(&definition_with(
        "log_dir = \"logs\"",
        &[job_block("a/b", &helper_command("say hello oops"), "")],
    ));
    let before = conductor_core::format_compact(conductor_runtime::now());
    let output = project.run(&["pipeline", "p"]);
    let after = conductor_core::format_compact(conductor_runtime::now());
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    // 端末にも今と同じく出る。
    assert_eq!(text(&output.stdout), "hello");
    assert_eq!(text(&output.stderr), "oops");

    let logs = project.dir.path().join("logs");
    let files = names(&logs);
    assert_eq!(files.len(), 1, "{files:?}");
    let time = time_part(&files[0], "a_b", "1").expect("名前の形");
    assert!(
        before.as_str() <= time && time <= after.as_str(),
        "{before} {time} {after}"
    );
    // 2 つの流れの間の順は読んだ順(各流れの中身はそのまま。conductor は何も書き足さない)。
    let content = fs::read_to_string(logs.join(&files[0])).expect("読める");
    assert!(
        content == "hellooops" || content == "oopshello",
        "{content:?}"
    );
}

#[cfg(unix)]
#[test]
fn br3_4_unix_系のログのファイルは_0600() {
    use std::os::unix::fs::PermissionsExt;
    let project = Project::new(&definition_with(
        "log_dir = \"logs\"",
        &[job_block("a", &helper_command("exit 0"), "")],
    ));
    assert_eq!(project.run(&["run", "a"]).status.code(), Some(0));
    let logs = project.dir.path().join("logs");
    let file = logs.join(&names(&logs)[0]);
    let mode = fs::metadata(file).expect("読める").permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
}

#[test]
fn fr5_3_リトライで試行ごとに_3_つのファイルができる() {
    let project = Project::new(&definition_with(
        "log_dir = \"logs\"",
        &[job_block(
            "flaky",
            &helper_command("say try- '' 75"),
            "[job.retry]\nexit_codes = [75]\nmax = 2\ninterval = \"1s\"",
        )],
    ));
    let output = project.run(&["run", "flaky"]);
    assert_eq!(output.status.code(), Some(75));
    assert_eq!(text(&output.stdout), "try-try-try-");
    let logs = project.dir.path().join("logs");
    let files = names(&logs);
    assert_eq!(files.len(), 3, "{files:?}");
    for attempt in ["1", "2", "3"] {
        let file = files
            .iter()
            .find(|f| time_part(f, "flaky", attempt).is_some())
            .unwrap_or_else(|| panic!("試行 {attempt} のファイルが無い: {files:?}"));
        assert_eq!(fs::read_to_string(logs.join(file)).expect("読める"), "try-");
    }
}

#[test]
fn br5_1_ログを作れなければ起動せず_127_と_1_行() {
    let marker: PathBuf;
    let project = {
        let dir = tempfile::tempdir().expect("一時ディレクトリ");
        marker = dir.path().join("ran.txt");
        // 定義ファイルそのものを log_dir にする(ファイルなのでディレクトリを作れない)。
        let text = definition_with(
            "log_dir = \"conductor.toml\"",
            &[job_block("a", &helper_command_out(&marker, "run a 0"), "")],
        );
        fs::write(dir.path().join("conductor.toml"), text).expect("書ける");
        Project { dir }
    };
    let output = project.run(&["run", "a"]);
    assert_eq!(output.status.code(), Some(127));
    assert!(output.stdout.is_empty());
    let err = text(&output.stderr);
    assert_eq!(err.lines().count(), 1, "{err}");
    let prefix = format!(
        "conductor: error: cannot create the log file: {}",
        project.def_path().join("a.").display()
    );
    assert!(err.starts_with(&prefix), "{err}");
    assert!(err.ends_with(".1.log\n"), "{err}");
    assert!(!marker.exists(), "ジョブは起動されない");
}

#[test]
fn fr5_5_log_dir_のあり_なしで端末の出力が同じ() {
    let jobs = [
        job_block("one", &helper_command("say out-1 err-1"), ""),
        job_block("two", &helper_command("say out-2 err-2 3"), ""),
    ];
    let with_log = Project::new(&definition_with("log_dir = \"logs\"", &jobs));
    let without = Project::new(&definition_with("", &jobs));
    let a = with_log.run(&["pipeline", "p"]);
    let b = without.run(&["pipeline", "p"]);
    assert_eq!(a.status.code(), Some(3));
    assert_eq!(a.status.code(), b.status.code());
    assert_eq!(text(&a.stdout), text(&b.stdout));
    assert_eq!(text(&a.stderr), text(&b.stderr));
    assert_eq!(text(&a.stdout), "out-1out-2");
    assert_eq!(text(&a.stderr), "err-1err-2");
    assert!(!without.dir.path().join("logs").exists());
}

#[test]
fn nfr1_3_16_mib_の出力でも端末とファイルが一致する() {
    let total = 16 * 1024 * 1024;
    let project = Project::new(&definition_with(
        "log_dir = \"logs\"",
        &[job_block(
            "big",
            &helper_command(&format!("flood {total}")),
            "",
        )],
    ));
    let output = project.run(&["run", "big"]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    let expected = flood_bytes(total);
    assert_eq!(output.stdout.len(), total);
    assert!(output.stdout == expected, "端末の出力が並びと一致する");
    let logs = project.dir.path().join("logs");
    let file = fs::read(logs.join(&names(&logs)[0])).expect("読める");
    assert_eq!(file.len(), total);
    assert!(file == expected, "ファイルの中身が並びと一致する");
}

#[test]
fn nfr1_9_v_の行はジョブの出力の後に出る() {
    let project = Project::new(&definition_with(
        "log_dir = \"logs\"",
        &[
            job_block("a", &helper_command("say A-out A-err"), ""),
            job_block("b", &helper_command("say B-out B-err"), ""),
        ],
    ));
    let output = project.run(&["pipeline", "-v", "p"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        text(&output.stderr),
        "A-err[conductor] job=a result=success (exit code 0)\n\
         B-err[conductor] job=b result=success (exit code 0)\n"
    );
    assert_eq!(text(&output.stdout), "A-outB-out");
}

#[test]
fn br5_4_validate_graph_dry_run_はログのディレクトリを作らない() {
    let project = Project::new(&definition_with(
        "log_dir = \"logs\"",
        &[job_block("a", &helper_command("exit 0"), "")],
    ));
    for args in [
        &["validate"][..],
        &["graph"][..],
        &["run", "--dry-run", "a"][..],
        &["pipeline", "--dry-run", "p"][..],
    ] {
        let output = project.run(args);
        assert_eq!(output.status.code(), Some(0), "{args:?}");
    }
    assert!(!project.dir.path().join("logs").exists());
}

#[test]
fn br1_1_log_dir_の誤りは_64() {
    let project = Project::new(&definition_with(
        "log_dir = 1",
        &[job_block("a", &helper_command("exit 0"), "")],
    ));
    let output = project.run(&["run", "a"]);
    assert_eq!(output.status.code(), Some(64));
    assert!(text(&output.stderr).contains("`log_dir`"));
}
