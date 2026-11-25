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

//! 補助のコマンド(`watch`・`reset`・`convert`。W4〜W6、BR7.x)と、入口(`run_cli_with`)。
//! AC5.1.x、AC6.1.x、AC7.1.x。

mod common;

use common::*;
use conductor_converter::{ConversionError, ConversionErrorKind};
use conductor_core::{Attempt, JobState, JobStatus, PipelineState};
use conductor_runtime::StateErrorKind;

const YAML_DEFINITION: &str = "\
format_version: 1
pipeline:
  name: p
job:
  - id: a
    command: echo a
";

// ---- watch ----

#[test]
fn watch_は表を標準出力に_1_回出す() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime.script("a", &[Exec::Exit(0)]);
    h.run(&["run", "a"]);
    let out = h.run(&["watch"]);
    assert_eq!(out.code, 0);
    assert_eq!(out.err, "");
    assert_eq!(
        out.out,
        "Pipeline: p\nJOB  STATUS   DURATION  NOTE\na    success  1s        exit code 0\nb    pending\nc    pending\n"
    );
    assert_eq!(h.sink.out_writes, 1);
}

#[test]
fn u6_br4_1_watch_の所要時間は偽の時計の始まりから終わりまで() {
    // 偽の now は呼ぶたびに 1 秒進む。a の試行は始まり(0 回目)と終わり(1 回目)で 1 秒。
    // b はリトライで 2 回の試行(2〜3 回目、4〜5 回目)。所要時間は最初の始まりから最後の終わりまで 3 秒。
    let mut h = Harness::new(&THREE_JOBS.replace(
        "id = \"b\"\ncommand = \"echo b\"\n",
        "id = \"b\"\ncommand = \"echo b\"\n\n[job.retry]\nexit_codes = [75]\nmax = 1\ninterval = \"1s\"\n",
    ));
    h.runtime.script("b", &[Exec::Exit(75), Exec::Exit(0)]);
    assert_eq!(h.run(&["pipeline", "p"]).code, 0);
    let out = h.run(&["watch"]);
    assert_eq!(out.code, 0);
    assert!(out.out.contains("a    success  1s "), "{}", out.out);
    assert!(out.out.contains("b    success  3s "), "{}", out.out);
}

#[test]
fn u6_br4_1_watch_は動いている最中のジョブに今までの経過時間を出す() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime.script("a", &[Exec::Exit(0)]);
    assert_eq!(h.run(&["run", "a"]).code, 0);
    // b の試行を始めたまま止まったステート(強制終了の後と同じ形)を作る。
    let mut state = h.state().expect("ある");
    // 始まりは偽の時計の最初の時刻の 63 秒前。
    let started = conductor_core::Timestamp::from_unix_seconds(FAKE_NOW_BASE - 63).expect("範囲");
    state = conductor_core::start_attempt(&state, "b", started);
    conductor_runtime::write_state(
        &h.def_path(),
        None,
        &state,
        conductor_runtime::Durability::Fast,
    )
    .expect("書ける");
    // watch の now は 3 回目の呼び出し(a の始まりと終わりの後。fake_now(2))。b は 65 秒の経過。
    let out = h.run(&["watch"]);
    assert_eq!(out.code, 0);
    assert!(out.out.contains("b    pending  1m05s\n"), "{}", out.out);
    assert!(out.out.contains("c    pending\n"), "{}", out.out);
    assert_eq!(
        h.runtime.now_calls.get(),
        3,
        "watch は今の時刻を 1 回だけ取る"
    );
}

#[test]
fn u6_br3_1_watch_は古い形のステートを_65_にして_reset_を案内する() {
    let mut h = Harness::new(THREE_JOBS);
    std::fs::write(h.state_path(), OLD_STATE).expect("書ける");
    let out = h.run(&["watch"]);
    assert_eq!(out.code, 65);
    assert_eq!(out.out, "");
    assert!(out.err.contains("state file is corrupt"), "{}", out.err);
    assert!(out.err.contains("conductor reset"), "{}", out.err);
    assert!(!out.err.contains("echo"), "{}", out.err);
}

#[test]
fn watch_はステートが空なら空であることを表示して_0() {
    let mut h = Harness::new(THREE_JOBS);
    let out = h.run(&["watch"]);
    assert_eq!(out.code, 0);
    assert_eq!(out.out, "no state (nothing has run yet)\n");
}

#[test]
fn fr3_1_watch_は完了のステートの表の下に完了の行を出して_0() {
    let mut h = Harness::new(THREE_JOBS);
    assert_eq!(h.run(&["pipeline", "p"]).code, 0);
    let out = h.run(&["watch"]);
    assert_eq!(out.code, 0);
    assert_eq!(out.err, "");
    assert_eq!(
        out.out,
        "Pipeline: p\nJOB  STATUS   DURATION  NOTE\na    success  1s        exit code 0\nb    success  1s        exit code 0\nc    success  1s        exit code 0\ncompleted\n"
    );
    assert!(!out.out.contains("no state"), "{}", out.out);
}

#[test]
fn fr3_1_watch_の完了の行は日本語では完了() {
    let mut h = Harness::new(THREE_JOBS);
    assert_eq!(h.run(&["pipeline", "p"]).code, 0);
    h.env = FakeEnv::with(&[("LANG", "ja_JP.UTF-8")]);
    let out = h.run(&["watch"]);
    assert_eq!(out.code, 0);
    assert!(
        out.out
            .ends_with("c       success  1s        終了コード 0\n完了\n"),
        "{}",
        out.out
    );
    assert!(out.out.contains("success"), "状態の列の語は英語のまま");
}

#[test]
fn fr3_2_watch_は_run_id_ありの完了のステートにも完了の行を出す() {
    let mut h = Harness::new(THREE_JOBS);
    assert_eq!(h.run(&["pipeline", "p", "--run-id", "r1"]).code, 0);
    let out = h.run(&["watch", "--run-id", "r1"]);
    assert_eq!(out.code, 0);
    assert!(out.out.contains("Run ID: r1\n"), "{}", out.out);
    assert!(out.out.ends_with("completed\n"), "{}", out.out);
    // 別の run ID には、ステートがない
    let other = h.run(&["watch", "--run-id", "r2"]);
    assert_eq!(other.out, "no state (nothing has run yet)\n");
}

#[test]
fn fr3_3_no_state_はファイルなしと長さ_0_のときだけ() {
    let mut h = Harness::new(THREE_JOBS);
    assert_eq!(h.run(&["watch"]).out, "no state (nothing has run yet)\n");
    std::fs::write(h.state_path(), "").expect("書ける");
    let out = h.run(&["watch"]);
    assert_eq!(out.code, 0);
    assert_eq!(out.out, "no state (nothing has run yet)\n");
}

#[test]
fn fr3_4_watch_は壊れた完了のステートを_65_にする() {
    let mut h = Harness::new(THREE_JOBS);
    assert_eq!(h.run(&["pipeline", "p"]).code, 0);
    let body = std::fs::read_to_string(h.state_path()).expect("読める");
    std::fs::write(h.state_path(), body.replace("job_order", "job_ordr")).expect("書ける");
    let out = h.run(&["watch"]);
    assert_eq!(out.code, 65);
    assert!(out.err.contains("state file is corrupt"), "{}", out.err);
    assert_eq!(out.out, "");
}

#[test]
fn fr3_1_完了の削除に失敗して残ったステートも_watch_は完了として表示する() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime.fail_reset.set(true);
    assert_eq!(h.run(&["pipeline", "p", "--remove-on-complete"]).code, 65);
    let out = h.run(&["watch"]);
    assert_eq!(out.code, 0);
    assert!(out.out.ends_with("completed\n"), "{}", out.out);
}

#[test]
fn watch_は定義ファイルを読まず_検証しない() {
    let mut h = Harness::new(THREE_JOBS);
    h.run(&["run", "a"]);
    h.set_definition("this is not even toml = = =");
    h.runtime.log.borrow_mut().clear();
    let out = h.run(&["watch"]);
    assert_eq!(out.code, 0);
    assert!(out.out.contains("Pipeline: p"));
    assert_eq!(h.runtime.log(), ["read-raw"], "定義は読まない");
}

#[test]
fn watch_のステートの誤りは_65() {
    let mut h = Harness::new(THREE_JOBS);
    std::fs::write(h.state_path(), "garbage = = =").expect("書ける");
    let out = h.run(&["watch"]);
    assert_eq!(out.code, 65);
    assert_eq!(out.out, "");
    assert!(out.err.contains("state file is corrupt"), "{}", out.err);
}

#[test]
fn watch_のファイルへの出力には制御文字がない() {
    let mut h = Harness::new(THREE_JOBS);
    h.set_definition(&THREE_JOBS.replace("id = \"a\"", "id = \"x\\u001b[31my\""));
    h.run(&["run", "x\u{1b}[31my"]);
    h.env.terminal = false;
    let out = h.run(&["watch"]);
    assert_eq!(out.code, 0);
    assert!(!out.out.contains('\x1b'), "{:?}", out.out);
    assert!(out.out.contains("x\\u{1b}[31my"), "{}", out.out);
}

#[test]
fn watch_は端末なら色をつける_unix() {
    let mut h = Harness::new(THREE_JOBS);
    h.run(&["run", "a"]);
    h.env.terminal = true;
    let out = h.run(&["watch"]);
    assert_eq!(out.out.contains('\x1b'), cfg!(unix));
    h.env.vars.insert("NO_COLOR".into(), "1".into());
    let reporter_out = {
        // 新しい Reporter は、実行のたびに作られる
        h.run(&["watch"]).out
    };
    assert!(!reporter_out.contains('\x1b'), "NO_COLOR");
}

#[test]
fn watch_の標準出力が閉じていたら_65() {
    let mut h = Harness::new(THREE_JOBS);
    h.run(&["run", "a"]);
    h.sink.fail_out = true;
    let out = h.run(&["watch"]);
    assert_eq!(out.code, 65);
    assert!(
        out.err.contains("cannot write to standard output"),
        "{}",
        out.err
    );
    // 空の表示でも同じ
    let mut h = Harness::new(THREE_JOBS);
    h.sink.fail_out = true;
    assert_eq!(h.run(&["watch"]).code, 65);
}

// ---- reset ----

#[test]
fn reset_はステートを消す_何も表示しない() {
    let mut h = Harness::new(THREE_JOBS);
    h.run(&["run", "a"]);
    assert!(h.state_path().exists());
    let out = h.run(&["reset"]);
    assert_eq!(out.code, 0);
    assert_eq!(out.out, "");
    assert_eq!(out.err, "");
    assert!(!h.state_path().exists());
}

#[test]
fn fr4_1_reset_は完了のステートも消す_run_id_ありなしの両方() {
    let mut h = Harness::new(THREE_JOBS);
    assert_eq!(h.run(&["pipeline", "p"]).code, 0);
    assert_eq!(h.run(&["pipeline", "p", "--run-id", "r1"]).code, 0);
    assert!(h.state_path().exists() && h.state_path_for("r1").exists());
    assert_eq!(h.run(&["reset"]).code, 0);
    assert!(!h.state_path().exists());
    assert!(h.state_path_for("r1").exists(), "run ID ありは別");
    assert_eq!(h.run(&["reset", "--run-id", "r1"]).code, 0);
    assert!(!h.state_path_for("r1").exists());
}

#[test]
fn fr4_1_reset_all_は完了のステートもすべて消す() {
    let mut h = Harness::new(THREE_JOBS);
    assert_eq!(h.run(&["pipeline", "p"]).code, 0);
    assert_eq!(h.run(&["pipeline", "p", "--run-id", "r1"]).code, 0);
    assert_eq!(h.run(&["reset", "--all"]).code, 0);
    assert!(!h.state_path().exists());
    assert!(!h.state_path_for("r1").exists());
}

#[test]
fn fr2_5_remove_on_complete_は_watch_reset_validate_convert_で_68() {
    for args in [
        &["watch", "--remove-on-complete"][..],
        &["reset", "--remove-on-complete"],
        &["reset", "--all", "--remove-on-complete"],
        &["validate", "--remove-on-complete"],
        &["convert", "x.yaml", "--to", "toml", "--remove-on-complete"],
    ] {
        let mut h = Harness::new(THREE_JOBS);
        let out = h.run(args);
        assert_eq!(out.code, 68, "{args:?}: {}", out.err);
        assert!(
            out.err.contains("--remove-on-complete"),
            "{args:?}: {}",
            out.err
        );
        assert_eq!(h.runtime.run_calls.get(), 0);
    }
}

#[test]
fn reset_はステートがなくても成功する() {
    let mut h = Harness::new(THREE_JOBS);
    assert_eq!(h.run(&["reset"]).code, 0);
    assert_eq!(h.run(&["reset"]).code, 0);
}

#[test]
fn reset_は壊れたステートと定義の誤りを迂回する() {
    let mut h = Harness::new("not a definition = =");
    std::fs::write(h.state_path(), "garbage = = =").expect("書ける");
    let out = h.run(&["reset"]);
    assert_eq!(out.code, 0);
    assert!(!h.state_path().exists());
    assert_eq!(h.runtime.log(), ["reset"], "読み込みも検証もしない");
}

#[test]
fn reset_の失敗は_65() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime.fail_reset.set(true);
    let out = h.run(&["reset"]);
    assert_eq!(out.code, 65);
    assert!(out.err.contains("cannot remove state file"), "{}", out.err);
}

#[test]
fn reset_のあとは先頭から実行できる() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime.script("a", &[Exec::Exit(3)]);
    assert_eq!(h.run(&["run", "a"]).code, 3);
    h.run(&["reset"]);
    assert_eq!(h.run(&["run", "a"]).code, 0);
}

// ---- convert ----

#[test]
fn convert_は変換結果を全部_1_回で標準出力に書く() {
    let mut h = Harness::new(THREE_JOBS);
    let input = h.dir.path().join("in.yaml");
    std::fs::write(&input, YAML_DEFINITION).expect("書ける");
    let out = h.run_os(vec![
        "convert".into(),
        input.into_os_string(),
        "--to".into(),
        "toml".into(),
    ]);
    assert_eq!(out.code, 0);
    assert_eq!(out.err, "");
    assert_eq!(h.sink.out_writes, 1, "1 回で書く");
    assert!(out.out.contains("[[job]]"), "{}", out.out);
    assert!(out.out.contains("format_version = 1"));
    // 変換した結果は、conductor の定義として読める
    std::fs::write(h.def_path(), &out.out).expect("書ける");
    assert_eq!(h.run(&["run", "a"]).code, 0);
}

#[test]
fn convert_の失敗は_67_で標準出力は空() {
    let mut h = Harness::new(THREE_JOBS);
    let input = h.dir.path().join("bad.yaml");
    std::fs::write(&input, "a: [unclosed\n").expect("書ける");
    let out = h.run_os(vec![
        "convert".into(),
        input.into_os_string(),
        "--to".into(),
        "toml".into(),
    ]);
    assert_eq!(out.code, 67);
    assert_eq!(out.out, "");
    assert_eq!(h.sink.out_writes, 0);
    assert!(out.err.contains("syntax error"), "{}", out.err);
    assert!(out.err.starts_with("conductor: error: "));
}

#[test]
fn convert_の入力がないときも_67() {
    let mut h = Harness::new(THREE_JOBS);
    let out = h.run_raw(&["convert", "no-such-file.yaml", "--to", "toml"]);
    assert_eq!(out.code, 67);
    assert_eq!(out.out, "");
    assert!(out.err.contains("input file does not exist"), "{}", out.err);
}

#[test]
fn convert_の標準出力が閉じていたら_67() {
    let mut h = Harness::new(THREE_JOBS);
    *h.runtime.convert_result.borrow_mut() = Some(Ok("a = 1\n".to_string()));
    h.sink.fail_out = true;
    let out = h.run_raw(&["convert", "x.yaml", "--to", "toml"]);
    assert_eq!(out.code, 67);
    assert!(
        out.err.contains("cannot write to standard output"),
        "{}",
        out.err
    );
}

#[test]
fn convert_は表せない値を場所つきで表示する() {
    let mut h = Harness::new(THREE_JOBS);
    *h.runtime.convert_result.borrow_mut() = Some(Err(ConversionError {
        file: "in.yaml".into(),
        line: Some(4),
        key_path: Some("job[1].note".into()),
        kind: ConversionErrorKind::Unrepresentable,
        reason: "null は表せません",
    }));
    let out = h.run_raw(&["convert", "in.yaml", "--to", "toml"]);
    assert_eq!(out.code, 67);
    assert_eq!(out.out, "");
    assert!(
        out.err
            .contains("in.yaml:4: value cannot be represented in the target format"),
        "{}",
        out.err
    );
    assert!(out.err.contains("(at: job[1].note)"), "{}", out.err);
}

#[test]
fn convert_は定義もステートも読まず_シグナルも準備しない() {
    let mut h = Harness::new(THREE_JOBS);
    *h.runtime.convert_result.borrow_mut() = Some(Ok("x = 1\n".to_string()));
    let out = h.run_raw(&["convert", "x.yaml", "--to", "toml"]);
    assert_eq!(out.code, 0);
    assert_eq!(out.out, "x = 1\n");
    assert_eq!(h.runtime.log(), ["convert"]);
    assert_eq!(h.signals.started.get(), 0);
}

// ---- 入口 ----

#[test]
fn help_は使い方を標準出力に出して_0() {
    let mut h = Harness::new(THREE_JOBS);
    let out = h.run_raw(&["--help"]);
    assert_eq!(out.code, 0);
    assert!(out.out.starts_with("Usage:\n"), "{}", out.out);
    assert_eq!(out.err, "");
    let mut h = Harness::new(THREE_JOBS);
    h.env = FakeEnv::with(&[("LC_ALL", "ja_JP.UTF-8")]);
    assert!(h.run_raw(&["-h"]).out.starts_with("使い方:\n"));
}

#[test]
fn version_は版を標準出力に出して_0() {
    let mut h = Harness::new(THREE_JOBS);
    let out = h.run_raw(&["--version"]);
    assert_eq!(out.code, 0);
    assert_eq!(
        out.out,
        format!("conductor {}\n", env!("CARGO_PKG_VERSION"))
    );
    assert_eq!(h.env.var_calls.get(), 0, "ロケールを判定しない");
}

#[test]
fn 引数がなければ使い方を標準エラーに出して_68() {
    let mut h = Harness::new(THREE_JOBS);
    let out = h.run_raw(&[]);
    assert_eq!(out.code, 68);
    assert_eq!(out.out, "");
    assert!(
        out.err
            .starts_with("conductor: error: missing argument\n\nUsage:\n"),
        "{}",
        out.err
    );
}

#[test]
fn 使い方の誤りは日本語のロケールで日本語() {
    let mut h = Harness::new(THREE_JOBS);
    h.env = FakeEnv::with(&[("LANG", "ja_JP.UTF-8")]);
    let out = h.run_raw(&["start"]);
    assert_eq!(out.code, 68);
    assert!(
        out.err
            .starts_with("conductor: error: 知らないコマンドです: 'start'\n\n使い方:\n"),
        "{}",
        out.err
    );
}

#[test]
fn v_は_watch_と_reset_と_convert_に付けると_68() {
    let mut h = Harness::new(THREE_JOBS);
    assert_eq!(h.run_raw(&["watch", "-v"]).code, 68);
    assert_eq!(h.run_raw(&["reset", "-v"]).code, 68);
    assert_eq!(h.run_raw(&["convert", "f", "--to", "yaml", "-v"]).code, 68);
    assert!(h.runtime.log().is_empty(), "ジョブも読み込みもしない");
}

#[test]
fn ステートの読み込みの失敗の種類ごとに_65() {
    for kind in [
        StateErrorKind::Unreadable,
        StateErrorKind::TooLarge,
        StateErrorKind::Corrupt,
        StateErrorKind::VersionMismatch,
    ] {
        let mut h = Harness::new(THREE_JOBS);
        *h.runtime.read_failure.borrow_mut() = Some(state_error(&h.state_path(), kind));
        let out = h.run(&["watch"]);
        assert_eq!(out.code, 65, "{kind:?}");
        assert!(out.err.contains("conductor.toml.state"));
    }
}

// ---- watch --follow(v0.10.0 の U7。BR2.1〜BR4.2) ----

/// 継続表示のテストのステート。a と b の 2 つのジョブ。
fn follow_state(
    a: (JobStatus, Option<Option<usize>>),
    b: (JobStatus, Option<Option<usize>>),
    resume: Option<&str>,
    completed: bool,
) -> PipelineState {
    let job = |id: &str, (status, attempt): (JobStatus, Option<Option<usize>>)| {
        let mut job = JobState::pending(id);
        job.status = status;
        if status != JobStatus::Pending {
            job.exit_code = Some(if status == JobStatus::Error { 1 } else { 0 });
        }
        if let Some(end) = attempt {
            job.attempts.push(Attempt {
                started_at: fake_now(0),
                ended_at: end.map(fake_now),
            });
        }
        job
    };
    PipelineState {
        pipeline_name: "p".into(),
        definition_path: "conductor.toml".into(),
        job_states: vec![job("a", a), job("b", b)],
        resume_job_id: resume.map(str::to_string),
        end_reached: false,
        completed,
        run_id: None,
    }
}

/// ジョブとジョブの間(a が成功、b はまだ。表示を続ける)。
fn between() -> RawRead {
    Ok(Some(follow_state(
        (JobStatus::Success, Some(Some(1))),
        (JobStatus::Pending, None),
        Some("b"),
        false,
    )))
}

/// b が動いている(表示を続ける)。
fn running() -> RawRead {
    Ok(Some(follow_state(
        (JobStatus::Success, Some(Some(1))),
        (JobStatus::Pending, Some(None)),
        Some("b"),
        false,
    )))
}

/// 完了(表示して終わる)。
fn completed() -> RawRead {
    Ok(Some(follow_state(
        (JobStatus::Success, Some(Some(1))),
        (JobStatus::Success, Some(Some(2))),
        None,
        true,
    )))
}

/// a がエラーで止まった(表示して終わる)。
fn stopped() -> RawRead {
    Ok(Some(follow_state(
        (JobStatus::Error, Some(Some(1))),
        (JobStatus::Pending, None),
        Some("a"),
        false,
    )))
}

fn absent() -> RawRead {
    Ok(None)
}

fn unreadable(h: &Harness) -> RawRead {
    Err(state_error(&h.state_path(), StateErrorKind::Unreadable))
}

fn queue(h: &Harness, reads: Vec<RawRead>) {
    h.runtime.raw_reads.borrow_mut().extend(reads);
}

/// 表を描いた回数。
fn tables(out: &str) -> usize {
    out.matches("Pipeline: p\n").count()
}

fn reads(h: &Harness) -> usize {
    h.runtime.log().iter().filter(|l| *l == "read-raw").count()
}

/// ステートを書いた・消した呼び出しが無い(読むだけ。NFR6.2)。
fn assert_read_only(h: &Harness) {
    let log = h.runtime.log();
    assert!(
        !log.iter()
            .any(|l| l.starts_with("write:") || l.starts_with("reset")),
        "{log:?}"
    );
    assert_eq!(h.runtime.write_calls.get(), 0);
}

#[test]
fn u7_br3_1_完了のステートは_1_回描いて_0() {
    let mut h = Harness::new(THREE_JOBS);
    queue(&h, vec![completed()]);
    let out = h.run(&["watch", "--follow"]);
    assert_eq!(out.code, 0);
    assert_eq!(out.err, "");
    assert_eq!(tables(&out.out), 1);
    assert!(out.out.ends_with("completed\n"), "{}", out.out);
    assert!(h.runtime.waits.borrow().is_empty(), "待たない");
    assert_eq!(h.signals.started.get(), 1, "合図の準備をする");
    assert_read_only(&h);
}

#[test]
fn u7_br3_1_止まったステートは_1_回描いて_0() {
    let mut h = Harness::new(THREE_JOBS);
    queue(&h, vec![stopped()]);
    let out = h.run(&["watch", "--follow"]);
    assert_eq!(out.code, 0);
    assert_eq!(tables(&out.out), 1);
    assert!(h.runtime.waits.borrow().is_empty());
}

#[test]
fn u7_br2_1_動いている間は既定の_2_秒おきに描き直す() {
    let mut h = Harness::new(THREE_JOBS);
    queue(&h, vec![between(), running(), running(), completed()]);
    let out = h.run(&["watch", "--follow"]);
    assert_eq!(out.code, 0);
    assert_eq!(tables(&out.out), 4);
    assert_eq!(*h.runtime.waits.borrow(), [2, 2, 2]);
    // 描くたびに今の時刻を 1 回取る(BR2.1)。
    assert_eq!(h.runtime.now_calls.get(), 4);
    assert_read_only(&h);
}

#[test]
fn u7_br1_2_interval_の秒数で待つ() {
    let mut h = Harness::new(THREE_JOBS);
    queue(&h, vec![running(), completed()]);
    let out = h.run(&["watch", "--follow", "--interval", "5"]);
    assert_eq!(out.code, 0);
    assert_eq!(*h.runtime.waits.borrow(), [5]);
}

#[test]
fn u7_br2_2_端末でないときは_2_回目から空の_1_行を置いて追記する() {
    let mut h = Harness::new(THREE_JOBS);
    h.env.terminal = false;
    queue(&h, vec![running(), completed()]);
    let out = h.run(&["watch", "--follow"]);
    assert!(out.out.starts_with("Pipeline: p\n"), "{:?}", out.out);
    assert!(out.out.contains("\n\nPipeline: p\n"), "{:?}", out.out);
    assert!(!out.out.contains('\x1b'), "制御の文字を出さない");
    assert_eq!(h.sink.out_writes, 2, "1 回分ずつ 1 回で書く");
}

#[test]
fn u7_br2_2_unix_の端末では毎回画面を消してから描く() {
    let mut h = Harness::new(THREE_JOBS);
    h.env.terminal = true;
    h.env.vars.insert("NO_COLOR".into(), "1".into());
    queue(&h, vec![running(), completed()]);
    let out = h.run(&["watch", "--follow"]);
    assert_eq!(out.code, 0);
    let clear = "\x1b[H\x1b[2J";
    if cfg!(unix) {
        assert!(out.out.starts_with(clear), "{:?}", out.out);
        assert_eq!(out.out.matches(clear).count(), 2);
        assert!(!out.out.contains("\n\nPipeline"), "空の行は置かない");
    } else {
        // Windows は端末でも追記する。
        assert!(!out.out.contains('\x1b'), "{:?}", out.out);
        assert!(out.out.contains("\n\nPipeline: p\n"), "{:?}", out.out);
    }
}

#[test]
fn u7_br4_1_ステートが無ければ待ち_できたら表を描く() {
    let mut h = Harness::new(THREE_JOBS);
    h.env.terminal = false;
    queue(&h, vec![absent(), absent(), running(), completed()]);
    let out = h.run(&["watch", "--follow"]);
    assert_eq!(out.code, 0);
    assert_eq!(
        out.out.matches("no state (nothing has run yet)\n").count(),
        2,
        "見る前の「無い」は毎回描く"
    );
    assert!(
        out.out.starts_with(
            "no state (nothing has run yet)\n\nno state (nothing has run yet)\n\nPipeline: p\n"
        ),
        "{:?}",
        out.out
    );
    assert_eq!(tables(&out.out), 2);
    assert_eq!(*h.runtime.waits.borrow(), [2, 2, 2]);
    assert_eq!(h.runtime.now_calls.get(), 2, "「無い」では時刻を取らない");
}

#[test]
fn u7_br3_1_一度見えたステートが_1_回だけ無いなら続ける() {
    let mut h = Harness::new(THREE_JOBS);
    queue(&h, vec![running(), absent(), running(), completed()]);
    let out = h.run(&["watch", "--follow"]);
    assert_eq!(out.code, 0);
    assert_eq!(tables(&out.out), 3, "1 回だけの「無い」は何も描かない");
    assert_eq!(reads(&h), 4);
}

#[test]
fn u7_br3_1_一度見えたステートが_2_回続けて無いなら_0_で終わる() {
    let mut h = Harness::new(THREE_JOBS);
    queue(&h, vec![running(), absent(), absent(), completed()]);
    let out = h.run(&["watch", "--follow"]);
    assert_eq!(out.code, 0);
    assert_eq!(out.err, "");
    assert_eq!(tables(&out.out), 1);
    assert_eq!(reads(&h), 3, "消えたと判定して終わる");
    assert_eq!(*h.runtime.waits.borrow(), [2, 2]);
}

#[test]
fn u7_読めないときは何も描かず_1_秒後に読み直し_読めたら続ける() {
    let mut h = Harness::new(THREE_JOBS);
    let reads_in = vec![running(), unreadable(&h), unreadable(&h), completed()];
    queue(&h, reads_in);
    let out = h.run(&["watch", "--follow", "--interval", "30"]);
    assert_eq!(out.code, 0);
    assert_eq!(out.err, "");
    assert_eq!(tables(&out.out), 2);
    assert_eq!(*h.runtime.waits.borrow(), [30, 1, 1]);
}

#[test]
fn u7_読めないのが_3_回続けば_65() {
    let mut h = Harness::new(THREE_JOBS);
    let reads_in = vec![
        running(),
        unreadable(&h),
        unreadable(&h),
        unreadable(&h),
        completed(),
    ];
    queue(&h, reads_in);
    let out = h.run(&["watch", "--follow"]);
    assert_eq!(out.code, 65);
    assert_eq!(out.err.matches("cannot read state file").count(), 1);
    assert!(out.err.contains("conductor.toml.state"), "{}", out.err);
    assert_eq!(*h.runtime.waits.borrow(), [2, 1, 1]);
}

#[test]
fn u7_読めた後は読めない回数を数え直す() {
    // 読めない 2 回 → 無い(読めた)→ 読めない 2 回 → 完了。3 回続かないので 65 にならない。
    let mut h = Harness::new(THREE_JOBS);
    let reads_in = vec![
        unreadable(&h),
        unreadable(&h),
        absent(),
        unreadable(&h),
        unreadable(&h),
        completed(),
    ];
    queue(&h, reads_in);
    let out = h.run(&["watch", "--follow"]);
    assert_eq!(out.code, 0);
    assert_eq!(*h.runtime.waits.borrow(), [1, 1, 2, 1, 1]);
}

#[test]
fn u7_読めないときは無いの続きを戻す() {
    // 有る → 無い → 読めない → 無い → 完了。「無い」が 2 回続いていないので、消えたとしない。
    let mut h = Harness::new(THREE_JOBS);
    let reads_in = vec![running(), absent(), unreadable(&h), absent(), completed()];
    queue(&h, reads_in);
    let out = h.run(&["watch", "--follow"]);
    assert_eq!(out.code, 0);
    assert_eq!(tables(&out.out), 2);
    assert_eq!(reads(&h), 5);
}

#[test]
fn u7_br4_2_壊れたステートは今の_watch_と同じ表示で_65() {
    for kind in [
        StateErrorKind::Corrupt,
        StateErrorKind::TooLarge,
        StateErrorKind::VersionMismatch,
    ] {
        let mut h = Harness::new(THREE_JOBS);
        let error = Err(state_error(&h.state_path(), kind));
        queue(&h, vec![running(), error]);
        let out = h.run(&["watch", "--follow"]);
        assert_eq!(out.code, 65, "{kind:?}");
        assert!(out.err.contains("conductor.toml.state"), "{}", out.err);
        assert_eq!(*h.runtime.waits.borrow(), [2], "{kind:?}");
    }
}

#[test]
fn u7_br4_2_古い形のステートは_reset_を案内して_65() {
    let mut h = Harness::new(THREE_JOBS);
    std::fs::write(h.state_path(), OLD_STATE).expect("書ける");
    let out = h.run(&["watch", "--follow"]);
    assert_eq!(out.code, 65);
    assert!(out.err.contains("conductor reset"), "{}", out.err);
    assert_eq!(out.out, "");
}

#[test]
fn u7_br3_2_待ちの途中の合図で_0() {
    let mut h = Harness::new(THREE_JOBS);
    queue(&h, vec![running(), running(), completed()]);
    h.runtime.interrupt_wait.set(Some((1, 2)));
    let out = h.run(&["watch", "--follow"]);
    assert_eq!(out.code, 0);
    assert_eq!(out.err, "");
    assert_eq!(tables(&out.out), 2);
    assert_eq!(reads(&h), 2, "合図の後は読まない");
}

#[test]
fn u7_br3_2_描く前に合図が立っていれば何も読まずに_0() {
    let mut h = Harness::new(THREE_JOBS);
    h.signals = FakeSignals::new(SignalMode::RaiseOnStart(15));
    queue(&h, vec![running()]);
    let out = h.run(&["watch", "--follow"]);
    assert_eq!(out.code, 0);
    assert_eq!(out.out, "");
    assert_eq!(reads(&h), 0);
}

#[test]
fn u7_br3_3_合図の準備に失敗したら_70_で何も読まない() {
    let mut h = Harness::new(THREE_JOBS);
    h.signals = FakeSignals::new(SignalMode::Fail);
    queue(&h, vec![completed()]);
    let out = h.run(&["watch", "--follow"]);
    assert_eq!(out.code, 70);
    assert_eq!(
        out.err,
        "conductor: error: failed to set up signal handling\n"
    );
    assert_eq!(out.out, "");
    assert_eq!(reads(&h), 0);
}

#[test]
fn u7_br1_1_follow_の無い_watch_は合図の準備をしない() {
    let mut h = Harness::new(THREE_JOBS);
    h.signals = FakeSignals::new(SignalMode::Fail);
    queue(&h, vec![running()]);
    let out = h.run(&["watch"]);
    assert_eq!(out.code, 0);
    assert_eq!(tables(&out.out), 1);
    assert_eq!(h.signals.started.get(), 0);
    assert!(h.runtime.waits.borrow().is_empty());
}

#[test]
fn u7_標準出力に書けなければ_65() {
    let mut h = Harness::new(THREE_JOBS);
    h.sink.fail_out = true;
    queue(&h, vec![running(), completed()]);
    let out = h.run(&["watch", "--follow"]);
    assert_eq!(out.code, 65);
    assert!(
        out.err.contains("cannot write to standard output"),
        "{}",
        out.err
    );
    assert!(h.runtime.waits.borrow().is_empty());
    // 「無い」の表示でも同じ。
    let mut h = Harness::new(THREE_JOBS);
    h.sink.fail_out = true;
    queue(&h, vec![absent()]);
    assert_eq!(h.run(&["watch", "--follow"]).code, 65);
}

#[test]
fn u7_run_id_のステートを読む() {
    let mut h = Harness::new(THREE_JOBS);
    h.runtime.script("a", &[Exec::Exit(0)]);
    assert_eq!(h.run(&["pipeline", "p", "--run-id", "r1"]).code, 0);
    let out = h.run(&["watch", "--follow", "--run-id", "r1"]);
    assert_eq!(out.code, 0);
    assert!(out.out.contains("Run ID: r1\n"), "{}", out.out);
    assert!(out.out.ends_with("completed\n"), "{}", out.out);
}

#[test]
fn u7_1000_回の繰り返しでも動き_書き込みをしない() {
    let mut h = Harness::new(THREE_JOBS);
    h.env.terminal = false;
    let mut reads_in: Vec<RawRead> = (0..999).map(|_| running()).collect();
    reads_in.push(completed());
    queue(&h, reads_in);
    let out = h.run(&["watch", "--follow", "--interval", "1"]);
    assert_eq!(out.code, 0);
    assert_eq!(tables(&out.out), 1000);
    assert_eq!(h.runtime.waits.borrow().len(), 999);
    assert_read_only(&h);
}
