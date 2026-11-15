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

//! 個別実行(`conductor run`。W2、BR2.x、BR4.x、BR5.x)の流れ。偽のジョブの起動・書き込みの失敗・
//! シグナルで、reliability-design の表の全行を確かめる。AC1.3.x、AC2.3.x、AC2.4.x。

mod common;

use common::*;
use conductor_core::{JobStatus, SkipReason};
use conductor_runtime::{Durability, StateErrorKind};

/// 1 ジョブだけの定義(最初のジョブが、そのまま最後のジョブ)。
const ONE_JOB: &str = r#"format_version = 1
[pipeline]
name = "p"
[[job]]
id = "only"
command = "echo only"
"#;

/// 成功コードに 2 だけを書いた定義(0 は成功に含まれない)。
const SUCCESS_IS_TWO: &str = r#"format_version = 1
[pipeline]
name = "p"
[[job]]
id = "a"
command = "echo a"
success_exit_codes = [2]
"#;

/// `command` が配列のジョブ 1 つだけの定義(先頭の要素は、存在しないプログラム)。
const ARRAY_JOB: &str = r#"format_version = 1
[pipeline]
name = "p"
[[job]]
id = "a"
command = ["conductor-array-prog", "x y"]
"#;

fn status_of(h: &Harness, id: &str) -> JobStatus {
    h.state()
        .expect("ステートがある")
        .job_states
        .iter()
        .find(|j| j.job_id == id)
        .expect("ジョブがある")
        .status
}

// ---- 成功と順序 ----

#[test]
fn 成功は何も表示せず_ロケールも判定しない() {
    let mut h = Harness::new(THREE_JOBS);
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 0);
    assert_eq!(out.out, "");
    assert_eq!(out.err, "");
    assert!(!h.env.locale_read(), "ロケールの判定が行われない");
    assert_eq!(h.env.terminal_calls.get(), 0);
    assert_eq!(status_of(&h, "a"), JobStatus::Success);
}

#[test]
fn 順序は_判断_初期ステート_ジョブ_記録の書き込み() {
    let mut h = Harness::new(THREE_JOBS);
    h.run(&["run", "a"]);
    assert_eq!(
        h.runtime.log(),
        ["load", "read", "write:fast", "run:a", "write:durable"]
    );
}

#[test]
fn 空でないステートでは_初期ステートを書かない() {
    let mut h = Harness::new(THREE_JOBS);
    h.run(&["run", "a"]);
    h.runtime.log.borrow_mut().clear();
    let out = h.run(&["run", "b"]);
    assert_eq!(out.code, 0);
    assert_eq!(
        h.runtime.log(),
        ["load", "read", "run:b", "write:durable"],
        "Fast の書き込みがない"
    );
}

#[test]
fn 警告は_0_で終わり_記録される() {
    let mut h = Harness::new(WARNING_JUMPS);
    h.runtime.script("a", &[Exec::Exit(1)]);
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 0);
    assert_eq!(status_of(&h, "a"), JobStatus::Warning);
    assert_eq!(status_of(&h, "b"), JobStatus::Skipped);
    let state = h.state().expect("ステート");
    let b = state
        .job_states
        .iter()
        .find(|j| j.job_id == "b")
        .expect("b");
    assert_eq!(b.skip_reason, Some(SkipReason::ByTransition));
}

#[test]
fn エラーはジョブの終了コードで終わり_再開位置が残る() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime.script("a", &[Exec::Exit(2)]);
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 2);
    assert_eq!(out.err, "", "ジョブ自身のエラーを、conductor は表示しない");
    assert_eq!(status_of(&h, "a"), JobStatus::Error);
    assert_eq!(
        h.state().expect("ステート").resume_job_id.as_deref(),
        Some("a")
    );
}

#[test]
fn エラーからの再開は_そのジョブを再実行する() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime.script("a", &[Exec::Exit(2), Exec::Exit(0)]);
    assert_eq!(h.run(&["run", "a"]).code, 2);
    assert_eq!(h.run(&["run", "a"]).code, 0);
    assert_eq!(h.runtime.ran(), ["a", "a"]);
    assert_eq!(status_of(&h, "a"), JobStatus::Success);
}

// ---- 起動前の判断 ----

#[test]
fn 順序外は_66_で_ステートを書かない() {
    let mut h = Harness::new(THREE_JOBS);
    let out = h.run(&["run", "b"]);
    assert_eq!(out.code, 66);
    assert!(
        out.err
            .contains("conductor: error: job 'b': cannot run this job now"),
        "{}",
        out.err
    );
    assert!(out.err.contains("out of order"));
    assert_eq!(h.runtime.run_calls.get(), 0);
    assert!(
        !h.state_path().exists(),
        "Reject の前に、初期ステートを書かない"
    );
    assert!(!h.runtime.log().iter().any(|l| l.starts_with("write")));
}

#[test]
fn 順序外は_ステートがあっても変えない() {
    let mut h = Harness::new(THREE_JOBS);
    h.run(&["run", "a"]);
    let before = std::fs::read(h.state_path()).expect("ステート");
    let out = h.run(&["run", "c"]);
    assert_eq!(out.code, 66);
    assert_eq!(std::fs::read(h.state_path()).expect("ステート"), before);
}

#[test]
fn 実行済みのスキップは_0_で_書き込まない() {
    let mut h = Harness::new(THREE_JOBS);
    h.run(&["run", "a"]);
    h.runtime.log.borrow_mut().clear();
    let out = h.run(&["--verbose", "run", "a"]);
    assert_eq!(out.code, 0);
    assert_eq!(h.runtime.ran(), Vec::<String>::new(), "再実行しない");
    assert!(
        !h.runtime.log().iter().any(|l| l.starts_with("write")),
        "変更がないので書かない"
    );
    assert_eq!(out.err, "[conductor] job=a skipped (already executed)\n");
}

#[test]
fn スキップの記録は_書き込む() {
    let mut h = Harness::new(WARNING_JUMPS);
    h.runtime.script("a", &[Exec::Exit(1)]);
    h.run(&["run", "a"]);
    h.runtime.log.borrow_mut().clear();
    let out = h.run(&["run", "b"]);
    assert_eq!(out.code, 0);
    assert_eq!(h.runtime.ran(), Vec::<String>::new(), "b は起動しない");
    assert!(h.runtime.log().contains(&"write:durable".to_string()));
    assert_eq!(out.err, "", "既定では何も表示しない(AC2.4.3)");
}

#[test]
fn スキップで完了したら_初期化する() {
    let mut h = Harness::new(END_AFTER_A);
    assert_eq!(h.run(&["run", "a"]).code, 0);
    let state = h.state().expect("ステート");
    assert!(state.end_reached && !state.completed);
    assert_eq!(h.run(&["run", "b"]).code, 0, "@end の後ろは、スキップ");
    assert!(h.state_path().exists(), "まだ完了しない");
    assert_eq!(h.run(&["run", "c"]).code, 0);
    assert!(
        !h.state_path().exists(),
        "最後のスキップで完了し、初期化される"
    );
    assert_eq!(h.runtime.ran(), ["a"]);
    assert_eq!(h.runtime.log().iter().filter(|l| *l == "reset").count(), 1);
}

#[test]
fn 最後のジョブの完了で_初期化する() {
    let mut h = Harness::new(ONE_JOB);
    let out = h.run(&["run", "only"]);
    assert_eq!(out.code, 0);
    assert!(!h.state_path().exists());
    let log = h.runtime.log();
    assert_eq!(log[log.len() - 2..], ["write:durable", "reset"]);
}

// ---- 検査の失敗 ----

#[test]
fn 定義の誤りは_64_ですべて表示し_ジョブを起動しない() {
    let mut h = Harness::new(
        "format_version = 1\nbogus = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"c\"\nextra = 1\n",
    );
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 64);
    let lines: Vec<&str> = out.err.lines().collect();
    assert_eq!(lines.len(), 2, "1 行ずつ: {}", out.err);
    assert!(
        lines
            .iter()
            .all(|l| l.starts_with("conductor: error: ") && l.contains("conductor.toml:"))
    );
    assert_eq!(h.runtime.run_calls.get(), 0);
    assert_eq!(h.runtime.log(), ["load"], "ステートを読まない");
}

#[test]
fn 古い_jobs_のキーは定義の誤り() {
    let mut h = Harness::new(
        "format_version = 1\n[pipeline]\nname = \"p\"\n[[jobs]]\nid = \"a\"\ncommand = \"c\"\n",
    );
    assert_eq!(h.run(&["run", "a"]).code, 64);
}

#[test]
fn 定義ファイルがないのも_64() {
    let mut h = Harness::new(THREE_JOBS);
    std::fs::remove_file(h.def_path()).expect("消せる");
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 64);
    assert!(
        out.err.contains("definition file does not exist"),
        "{}",
        out.err
    );
}

#[test]
fn command_の誤りは_64_でジョブidを表示する() {
    let mut h = Harness::new(
        "format_version = 1\n[pipeline]\nname = \"p\"\n[[job]]\nid = \"a\"\ncommand = \"echo 'oops\"\n[[job]]\nid = \"b\"\ncommand = \"ok\"\n",
    );
    let out = h.run(&["run", "b"]);
    assert_eq!(out.code, 64);
    assert!(
        out.err.contains("job 'a': command has an unclosed quote"),
        "{}",
        out.err
    );
    assert_eq!(h.runtime.run_calls.get(), 0);
}

#[test]
fn 知らないジョブidは_68() {
    let mut h = Harness::new(THREE_JOBS);
    let out = h.run(&["run", "zzz"]);
    assert_eq!(out.code, 68);
    assert!(
        out.err
            .contains("job is not defined in the definition file: 'zzz'"),
        "{}",
        out.err
    );
    assert_eq!(h.runtime.log(), ["load"], "ステートは読まない");
}

#[test]
fn 定義の誤りは_知らないジョブidより優先する() {
    let mut h = Harness::new("format_version = 1\n");
    assert_eq!(h.run(&["run", "zzz"]).code, 64);
}

#[test]
fn ステートが読めないときは_65_で_reset_を案内する() {
    let mut h = Harness::new(THREE_JOBS);
    *h.runtime.read_failure.borrow_mut() =
        Some(state_error(&h.state_path(), StateErrorKind::Unreadable));
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 65);
    assert!(out.err.contains("conductor reset"), "{}", out.err);
    assert_eq!(h.runtime.run_calls.get(), 0);
}

#[test]
fn ステートが壊れているときは_65() {
    let mut h = Harness::new(THREE_JOBS);
    std::fs::write(h.state_path(), "this is not toml = = =").expect("書ける");
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 65);
    assert!(out.err.contains("state file is corrupt"), "{}", out.err);
    assert!(out.err.contains("conductor.toml.state"));
    assert_eq!(h.runtime.run_calls.get(), 0);
}

#[test]
fn 定義との食い違いは_65_で_reset_を案内する() {
    let mut h = Harness::new(THREE_JOBS);
    h.run(&["run", "a"]);
    for (definition, what) in [
        (
            THREE_JOBS.replace("name = \"p\"", "name = \"q\""),
            "pipeline name",
        ),
        (THREE_JOBS.replace("id = \"c\"", "id = \"z\""), "job ids"),
    ] {
        h.set_definition(&definition);
        let out = h.run(&["run", "a"]);
        assert_eq!(out.code, 65, "{what}");
        assert!(out.err.contains(what), "{}", out.err);
        assert!(out.err.contains("conductor reset"));
    }
    h.set_definition(
        &THREE_JOBS
            .replace("id = \"a\"", "id = \"TMP\"")
            .replace("id = \"b\"", "id = \"a\"")
            .replace("id = \"TMP\"", "id = \"b\""),
    );
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 65);
    assert!(out.err.contains("job order"), "{}", out.err);
}

// ---- 書き込み・初期化の失敗 ----

#[test]
fn 初期ステートが書けないときは_65_でジョブを起動しない() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime
        .write_failure
        .set(WriteFailure::OnDurability(Durability::Fast));
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 65);
    assert!(out.err.contains("the job was not started"), "{}", out.err);
    assert_eq!(h.runtime.run_calls.get(), 0);
}

#[test]
fn ジョブのあとの書き込みの失敗は_65_でジョブは実行済み() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime
        .write_failure
        .set(WriteFailure::OnDurability(Durability::Durable));
    h.runtime.script("a", &[Exec::Exit(0)]);
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 65, "ジョブの終了コードを返さない");
    assert!(
        out.err.contains("the job has already been executed"),
        "{}",
        out.err
    );
    assert_eq!(h.runtime.ran(), ["a"]);
    // 次の呼び出しでは、ジョブは未実行として扱われ、再実行される(at-least-once)
    h.runtime.write_failure.set(WriteFailure::Never);
    assert_eq!(h.run(&["run", "a"]).code, 0);
    assert_eq!(h.runtime.ran(), ["a", "a"]);
}

#[test]
fn ジョブがエラーでも_書き込みの失敗は_65() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime
        .write_failure
        .set(WriteFailure::OnDurability(Durability::Durable));
    h.runtime.script("a", &[Exec::Exit(7)]);
    assert_eq!(h.run(&["run", "a"]).code, 65);
}

#[test]
fn 完了後の初期化の失敗は_65_で_completed_のまま残る() {
    let mut h = Harness::new(ONE_JOB);
    h.runtime.fail_reset.set(true);
    let out = h.run(&["run", "only"]);
    assert_eq!(out.code, 65);
    assert!(out.err.contains("cannot remove state file"), "{}", out.err);
    assert!(h.state_path().exists());
    assert!(
        h.state().is_none(),
        "完了のまま残ったステートは、空として読まれる"
    );
    // 次の実行は、先頭から始まる
    h.runtime.fail_reset.set(false);
    assert_eq!(h.run(&["run", "only"]).code, 0);
    assert_eq!(h.runtime.ran(), ["only", "only"]);
}

#[test]
fn 起動前の中断の結果の書き込みの失敗は_65_でジョブは起動していない() {
    let mut h = Harness::new(THREE_JOBS);
    h.signals = FakeSignals::new(SignalMode::RaiseOnStart(15));
    h.runtime
        .write_failure
        .set(WriteFailure::OnDurability(Durability::Durable));
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 65);
    assert!(out.err.contains("the job was not started"), "{}", out.err);
    assert!(!out.err.contains("already been executed"), "{}", out.err);
    assert_eq!(h.runtime.ran(), Vec::<String>::new());
}

#[test]
fn スキップの記録の書き込みの失敗は_65_でジョブは起動しない() {
    let mut h = Harness::new(WARNING_JUMPS);
    h.runtime.script("a", &[Exec::Exit(1)]);
    h.run(&["run", "a"]); // 書き込みは 2 回(0、1)
    h.runtime.write_failure.set(WriteFailure::OnCall(2));
    let out = h.run(&["run", "b"]);
    assert_eq!(out.code, 65);
    assert!(out.err.contains("the job was not started"), "{}", out.err);
    assert_eq!(h.runtime.ran(), ["a"]);
}

#[test]
fn スキップで完了したときの初期化の失敗は_65() {
    let mut h = Harness::new(END_AFTER_A);
    h.run(&["run", "a"]);
    h.run(&["run", "b"]);
    h.runtime.fail_reset.set(true);
    let out = h.run(&["run", "c"]);
    assert_eq!(out.code, 65);
    assert_eq!(h.runtime.ran(), ["a"]);
}

// ---- ジョブの結果の表示 ----

#[test]
fn 起動できないときは_127_で_cannot_start_command() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime.script("a", &[Exec::StartFailure]);
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 127);
    assert_eq!(
        out.err,
        "conductor: error: job 'a': cannot start command 'echo'\n"
    );
    assert_eq!(status_of(&h, "a"), JobStatus::Error);
}

#[test]
fn 配列の_command_の起動失敗は先頭の要素の名前を表示する() {
    let mut h = Harness::new(ARRAY_JOB);
    h.runtime.script("a", &[Exec::StartFailure]);
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 127);
    assert_eq!(
        out.err, "conductor: error: job 'a': cannot start command 'conductor-array-prog'\n",
        "引数 `x y` は表示に出ない"
    );
}

#[test]
fn 成功コードに含まれない_0_は_1_で原因を常に表示する() {
    let mut h = Harness::new(SUCCESS_IS_TWO);
    h.runtime.script("a", &[Exec::Exit(0)]);
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 1);
    assert!(
        out.err
            .contains("exit code 0 is not listed in success_exit_codes"),
        "{}",
        out.err
    );
    assert_eq!(status_of(&h, "a"), JobStatus::Error);
}

#[test]
fn シグナルで終了したジョブは_128_足す番号() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime.script("a", &[Exec::KilledBy(9)]);
    assert_eq!(h.run(&["run", "a"]).code, 137);
}

#[test]
fn 範囲外の終了コードは_どの_os_でも下位_8_ビット() {
    for (job_code, expected) in [(256, 1), (300, 44), (-1, 255), (512, 1), (255, 255)] {
        let mut h = Harness::new(THREE_JOBS);
        h.runtime.script("a", &[Exec::Exit(job_code)]);
        let out = h.run(&["-v", "run", "a"]);
        assert_eq!(out.code, expected, "{job_code}");
        if !(0..=255).contains(&job_code) {
            // 元の値は、-v の表示にそのまま示す
            assert!(
                out.err.contains(&format!(
                    "exit code {job_code}, conductor exits with {expected}"
                )),
                "{}",
                out.err
            );
        }
    }
}

#[test]
fn v_はジョブの結果と遷移を標準エラーに表示する() {
    let mut h = Harness::new(WARNING_JUMPS);
    h.runtime.script("a", &[Exec::Exit(1)]);
    let out = h.run(&["run", "-v", "a"]);
    assert_eq!(out.code, 0);
    assert_eq!(out.out, "");
    assert_eq!(
        out.err,
        "[conductor] job=a result=warning (exit code 1)\n[conductor] transition: warning -> c (skipped: b)\n"
    );
}

#[test]
fn v_は日本語のロケールで日本語になる() {
    let mut h = Harness::new(THREE_JOBS);
    h.env = FakeEnv::with(&[("LANG", "ja_JP.UTF-8")]);
    let out = h.run(&["run", "-v", "a"]);
    assert_eq!(out.err, "[conductor] job=a result=success (終了コード 0)\n");
    let out = h.run(&["run", "-v", "a"]);
    assert!(out.err.contains("実行済み"), "{}", out.err);
}

#[test]
fn v_は_end_への遷移を表示する() {
    let mut h = Harness::new(END_AFTER_A);
    let out = h.run(&["run", "-v", "a"]);
    assert_eq!(
        out.err,
        "[conductor] job=a result=success (exit code 0)\n[conductor] transition: success -> @end\n"
    );
}

// ---- シグナル(BR4.x) ----

#[test]
fn 起動の前に合図が立っていれば_起動せず_再開位置を記録して_128_足す番号() {
    let mut h = Harness::new(THREE_JOBS);
    h.signals = FakeSignals::new(SignalMode::RaiseOnStart(15));
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 143);
    assert_eq!(h.runtime.ran(), Vec::<String>::new());
    assert_eq!(
        h.runtime.log(),
        ["load", "read", "write:fast", "skip-run:a", "write:durable"],
        "初期ステートを書いてから、再開位置を記録する(BR4.2a)"
    );
    let state = h.state().expect("ステートが残る");
    assert_eq!(state.resume_job_id.as_deref(), Some("a"));
    assert_eq!(
        state.job_states[0].status,
        JobStatus::Pending,
        "エラーとしては記録しない"
    );
}

#[test]
fn 検査中の合図より_定義の誤りの終了コードが優先する() {
    let mut h = Harness::new("format_version = 1\n");
    h.signals = FakeSignals::new(SignalMode::RaiseOnStart(2));
    assert_eq!(h.run(&["run", "a"]).code, 64);
    let mut h = Harness::new(THREE_JOBS);
    h.signals = FakeSignals::new(SignalMode::RaiseOnStart(2));
    assert_eq!(h.run(&["run", "zzz"]).code, 68);
}

#[test]
fn 検査中の合図より_reject_と_skip_が優先する() {
    let mut h = Harness::new(THREE_JOBS);
    h.signals = FakeSignals::new(SignalMode::RaiseOnStart(15));
    assert_eq!(h.run(&["run", "b"]).code, 66, "Reject");
    assert!(!h.state_path().exists());

    let mut h = Harness::new(THREE_JOBS);
    h.run(&["run", "a"]);
    h.signals = FakeSignals::new(SignalMode::RaiseOnStart(15));
    assert_eq!(h.run(&["run", "a"]).code, 0, "Skip");
    assert_eq!(h.runtime.ran(), ["a"]);
}

#[test]
fn ジョブの実行中の合図は_エラーとして記録して_128_足す番号() {
    let mut h = Harness::new(THREE_JOBS);
    *h.runtime.during_job.borrow_mut() = Some(Box::new(|_, signal| signal.raise(2)));
    let out = h.run(&["run", "-v", "a"]);
    assert_eq!(out.code, 130);
    assert!(out.err.contains("interrupted by signal 2"), "{}", out.err);
    assert_eq!(status_of(&h, "a"), JobStatus::Error);
}

#[test]
fn ジョブの実行中に_2_つの合図が来ても_最初の番号が残る() {
    let mut h = Harness::new(THREE_JOBS);
    *h.runtime.during_job.borrow_mut() = Some(Box::new(|_, signal| {
        signal.raise(15);
        signal.raise(2);
    }));
    assert_eq!(h.run(&["run", "a"]).code, 143);
}

#[test]
fn シグナルの準備は_定義の検査より前に行う() {
    let mut h = Harness::new("format_version = 1\n");
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 64);
    assert_eq!(
        h.signals.started.get(),
        1,
        "検査に失敗しても、準備は済んでいる"
    );
}

#[test]
fn シグナルの準備に失敗したら_70_で_固定の英語の文() {
    let mut h = Harness::new(THREE_JOBS);
    h.signals = FakeSignals::new(SignalMode::Fail);
    h.env = FakeEnv::with(&[("LANG", "ja_JP.UTF-8")]);
    let out = h.run(&["run", "a"]);
    assert_eq!(out.code, 70);
    assert_eq!(
        out.err,
        "conductor: error: failed to set up signal handling\n"
    );
    assert_eq!(h.runtime.log(), Vec::<String>::new(), "何も実行しない");
    assert_eq!(h.env.var_calls.get(), 0, "ロケールを判定しない");
}

#[test]
fn すでに登録済みでも_流れは成功と同様に続ける() {
    let mut h = Harness::new(THREE_JOBS);
    h.signals = FakeSignals::new(SignalMode::Already);
    assert_eq!(h.run(&["run", "a"]).code, 0);
    assert_eq!(h.runtime.ran(), ["a"]);
}

#[test]
fn 標準エラーが閉じていても_本来の終了コードで終わる() {
    let mut h = Harness::new(THREE_JOBS);
    h.sink.fail_err = true;
    h.runtime.script("a", &[Exec::Exit(2)]);
    assert_eq!(h.run(&["run", "-v", "a"]).code, 2);
    assert_eq!(h.run(&["run", "zzz"]).code, 68);
}

#[test]
fn 使い方の誤りでは_何も実行せず_シグナルも準備しない() {
    let mut h = Harness::new(THREE_JOBS);
    let out = h.run_raw(&["run"]);
    assert_eq!(out.code, 68);
    assert!(out.err.contains("missing argument"));
    assert_eq!(h.signals.started.get(), 0);
    assert!(h.runtime.log().is_empty());
}

#[test]
fn config_を省くと_実行ディレクトリの_conductor_toml_を読む() {
    // テストの実行ディレクトリ(クレートの根)には、conductor.toml がない。
    let mut h = Harness::new(THREE_JOBS);
    let out = h.run_raw(&["run", "a"]);
    assert_eq!(out.code, 64);
    assert!(out.err.contains("conductor.toml"), "{}", out.err);
    assert_eq!(h.runtime.log(), ["load"]);
}
