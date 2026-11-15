//! 性能と同時の実行(NFR1.1〜NFR1.3、NFR1.5、NFR1.6)。
//!
//! 測定の結果は、標準エラーと、`GITHUB_STEP_SUMMARY`(あれば)に出す。基準を超えても、
//! **失敗にしない**。環境変数 `STARTUP_BENCH_ENFORCE` が `true` のときだけ、基準を超えたら失敗にする
//! (配分の確定のあと、CI が切り替える。cicd-pipeline の `startup-bench`)。

mod bin_support;
mod common;

use std::io::Write;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use bin_support::*;
use common::MemRuntime;

const ITERATIONS: usize = 20;
/// `run` の自身のオーバーヘッド(起動前 + ジョブのあと)の基準(ミリ秒)。
const OVERHEAD_LIMIT_MS: f64 = 100.0;
/// `pipeline` の、ジョブ 1 つあたりの目標(ミリ秒)。
const PER_JOB_LIMIT_MS: f64 = 50.0;
/// ジョブ 1,000 個の定義で、`run` が終わるまでの目標(ミリ秒。仮)。
const LARGE_LIMIT_MS: f64 = 1000.0;

fn enforce() -> bool {
    std::env::var("STARTUP_BENCH_ENFORCE").is_ok_and(|v| v == "true")
}

fn now_ns() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

/// 昇順の 19 番目(20 回のうちの p95)。
fn p95(values: &mut [f64]) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let index = ((values.len() as f64) * 0.95).ceil() as usize;
    values[index.saturating_sub(1).min(values.len() - 1)]
}

fn report(line: &str) {
    eprintln!("[bench] {line}");
    if let Ok(path) = std::env::var("GITHUB_STEP_SUMMARY") {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(file, "- {line}");
        }
    }
}

fn warn_or_fail(name: &str, value_ms: f64, limit_ms: f64) {
    if value_ms > limit_ms {
        let message = format!("{name}: {value_ms:.1} ms が基準 {limit_ms:.0} ms を超えた");
        if enforce() {
            panic!("{message}(STARTUP_BENCH_ENFORCE=true)");
        }
        report(&format!("警告: {message}"));
    }
}

fn ms(ns: i128) -> f64 {
    ns as f64 / 1_000_000.0
}

fn parse_field(text: &str, name: &str) -> Option<i128> {
    text.lines()
        .find_map(|l| l.strip_prefix(&format!("{name}=")))
        .and_then(|v| v.trim().parse().ok())
}

fn ten_job_definition(command: &str) -> String {
    let jobs: Vec<String> = (0..10)
        .map(|i| job_block(&format!("j{i}"), command, ""))
        .collect();
    definition("p", &jobs)
}

#[test]
fn 起動のオーバーヘッドの測定_run_ジョブ_10_個() {
    let project = Project::new(&ten_job_definition(&helper_command("timing")));
    let (mut totals, mut pres, mut posts) = (Vec::new(), Vec::new(), Vec::new());
    for _ in 0..ITERATIONS {
        let _ = std::fs::remove_file(project.state_path());
        let mut command = conductor(&["run", "j0", "--config"]);
        command.arg(project.def_path());
        let t0 = now_ns() as i128;
        let output = run_limited(command, || {});
        let t1 = now_ns() as i128;
        assert_eq!(output.status.code(), Some(0));
        let out = text(&output.stdout);
        let start = parse_field(&out, "start_ns").expect("起動の時刻");
        let end = parse_field(&out, "end_ns").expect("終了の直前の時刻");
        let job_time = end - start;
        totals.push(ms((t1 - t0) - job_time));
        pres.push(ms(start - t0));
        posts.push(ms(t1 - end));
    }
    let (total, pre, post) = (p95(&mut totals), p95(&mut pres), p95(&mut posts));
    report(&format!(
        "run(ジョブ 10 個、{ITERATIONS} 回、debug ビルド): オーバーヘッド p95 = {total:.1} ms(起動前 p95 = {pre:.1} ms、ジョブのあと p95 = {post:.1} ms)"
    ));
    warn_or_fail("run のオーバーヘッド", total, OVERHEAD_LIMIT_MS);
}

#[test]
fn pipeline_のジョブ_1_つあたりの測定() {
    let project = Project::new(&ten_job_definition(&helper_command("exit 0")));
    let mut per_job = Vec::new();
    for _ in 0..5 {
        let _ = std::fs::remove_file(project.state_path());
        let started = Instant::now();
        let output = project.run(&["pipeline", "p"]);
        let elapsed = started.elapsed();
        assert_eq!(output.status.code(), Some(0));
        per_job.push(elapsed.as_secs_f64() * 1000.0 / 10.0);
    }
    let worst = p95(&mut per_job);
    report(&format!(
        "pipeline(ジョブ 10 個、5 回、debug ビルド): ジョブ 1 つあたり p95 = {worst:.1} ms(ジョブ自身の起動を含む)"
    ));
    warn_or_fail("pipeline のジョブ 1 つあたり", worst, PER_JOB_LIMIT_MS);
}

fn thousand_job_definition(command: &str) -> String {
    let jobs: Vec<String> = (0..1000)
        .map(|i| job_block(&format!("job{i}"), command, ""))
        .collect();
    definition("big", &jobs)
}

#[test]
fn ジョブ_1000_個の定義の時間() {
    let project = Project::new(&thousand_job_definition(&helper_command("exit 0")));
    let started = Instant::now();
    let output = project.run(&["run", "job0"]);
    let elapsed = started.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    report(&format!(
        "run(ジョブ 1,000 個、debug ビルド): ジョブを含む全体 = {elapsed:.1} ms"
    ));
    warn_or_fail("ジョブ 1,000 個の run", elapsed, LARGE_LIMIT_MS);

    // pipeline の流れの費用(ジョブの起動とファイルを除く。ジョブごとのステートの複製を含む)
    let runtime = MemRuntime::new(&thousand_job_definition("echo x"));
    let started = Instant::now();
    let (code, _) = common::run_mem(&runtime, &["pipeline", "big"]);
    let elapsed = started.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(code, 0);
    assert!(runtime.state.borrow().is_none(), "完了して、初期化された");
    report(&format!(
        "pipeline(ジョブ 1,000 個、メモリの Runtime、debug ビルド): 流れの全体 = {elapsed:.1} ms"
    ));
    warn_or_fail("ジョブ 1,000 個の pipeline の流れ", elapsed, 10_000.0);
}

#[cfg(unix)]
fn cpu_seconds(pid: u32) -> Option<f64> {
    let output = std::process::Command::new("ps")
        .args(["-o", "time=", "-p", &pid.to_string()])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    // `HH:MM:SS`、`MM:SS.ss`、`D-HH:MM:SS` のいずれも、':' で分けて足す
    let text = text.rsplit('-').next()?.to_string();
    let mut seconds = 0.0;
    for part in text.split(':') {
        seconds = seconds * 60.0 + part.trim().parse::<f64>().ok()?;
    }
    Some(seconds)
}

#[cfg(unix)]
#[test]
fn ジョブの実行中の_cpu_時間は無視できる() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let out = dir.path().join("job.out");
    let def_path = dir.path().join("conductor.toml");
    std::fs::write(
        &def_path,
        definition(
            "p",
            &[job_block(
                "sleeper",
                &helper_command_out(&out, "sleep 5"),
                "",
            )],
        ),
    )
    .expect("書ける");
    let mut command = conductor(&["run", "sleeper", "--config"]);
    command
        .arg(&def_path)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let child = command.spawn().expect("起動できる");
    let pid = child.id();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(child.wait_with_output());
    });
    let cleanup = || {
        if let Some(job) = pid_in(&out) {
            force_kill(job);
        }
        force_kill(pid);
    };
    if !wait_for_line(&out, "ready", Duration::from_secs(20)) {
        cleanup();
        panic!("ジョブが始まらなかった");
    }
    std::thread::sleep(Duration::from_secs(3));
    let cpu = cpu_seconds(pid);
    match rx.recv_timeout(LIMIT) {
        Ok(result) => {
            result.expect("待てる");
        }
        Err(_) => {
            cleanup();
            let _ = rx.recv_timeout(Duration::from_secs(10));
            panic!("時間の上限(30 秒)を超えました");
        }
    }
    match cpu {
        Some(seconds) => {
            report(&format!(
                "ジョブの実行中(約 3 秒)の conductor の CPU 時間 = {seconds:.2} s"
            ));
            assert!(
                seconds <= 0.1,
                "CPU 時間が基準(100 ms)を超えた: {seconds} s"
            );
        }
        None => eprintln!("スキップ: ps で CPU 時間を読めなかったため、測っていない"),
    }
}

#[test]
fn 同じ定義への_2_つの_run_は_どちらも実行され_ステートは壊れない() {
    let dir = tempfile::tempdir().expect("一時ディレクトリ");
    let out = dir.path().join("job.out");
    let def_path = dir.path().join("conductor.toml");
    std::fs::write(
        &def_path,
        definition(
            "p",
            &[
                job_block("a", &helper_command_out(&out, "sleep 2"), ""),
                job_block("b", &helper_command("exit 0"), ""),
            ],
        ),
    )
    .expect("書ける");

    let spawn = || {
        let mut command = conductor(&["run", "a", "--config"]);
        command
            .arg(&def_path)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        let child = command.spawn().expect("起動できる");
        let pid = child.id();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(child.wait_with_output());
        });
        (pid, rx)
    };
    let (pid1, rx1) = spawn();
    let kill_all = |pids: &[u32]| {
        for pid in pids {
            force_kill(*pid);
        }
    };
    if !wait_for_line(&out, "ready", Duration::from_secs(20)) {
        kill_all(&[pid1]);
        panic!("最初のジョブが始まらなかった");
    }
    let (pid2, rx2) = spawn();
    // 2 つ目のジョブも始まる(`ready` の行が 2 つになる)
    let start = Instant::now();
    loop {
        let readies = std::fs::read_to_string(&out)
            .map(|s| s.lines().filter(|l| *l == "ready").count())
            .unwrap_or(0);
        if readies >= 2 {
            break;
        }
        if start.elapsed() > Duration::from_secs(20) {
            kill_all(&[pid1, pid2]);
            panic!("2 つ目のジョブが始まらなかった");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    for rx in [rx1, rx2] {
        match rx.recv_timeout(LIMIT) {
            Ok(result) => {
                let output = result.expect("待てる");
                // ジョブは、2 秒のあと 99 で終わる(エラー)。どちらも、ジョブの終了コードで終わる。
                assert_eq!(output.status.code(), Some(99), "{}", text(&output.stderr));
            }
            Err(_) => {
                kill_all(&[pid1, pid2]);
                panic!("時間の上限(30 秒)を超えました");
            }
        }
    }
    // ステートは、読み戻せる(壊れた内容になっていない)
    let state = conductor_runtime::read_state(&def_path).expect("読み戻せる");
    assert!(state.is_some());
}
