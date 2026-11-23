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

//! パニックしない(NFR4.7)。ランダムな引数と、壊れたステートを返す偽の `Runtime`。
//!
//! ファイルに触れない `MemRuntime` を使う(ランダムな `--config` が、実際のファイルを触らないように)。

mod common;

use common::*;
use conductor_core::{JobState, JobStatus, PipelineState, SkipReason};

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[self.below(items.len())]
    }
}

const WORDS: &[&str] = &[
    "run",
    "pipeline",
    "watch",
    "reset",
    "convert",
    "a",
    "b",
    "c",
    "p",
    "zzz",
    "-v",
    "--verbose",
    "-vv",
    "--config",
    "--config=x",
    "--to",
    "--to=yaml",
    "yaml",
    "toml",
    "--",
    "-",
    "--help",
    "-h",
    "-V",
    "--version",
    "",
    "\u{1b}[31m",
    "日本語",
    "--bogus",
    "-x",
    "--bogus=1",
    "=",
    "run\0",
    "a b",
];

#[test]
fn ランダムな引数でパニックしない() {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let runtime = MemRuntime::new(THREE_JOBS);
    for _ in 0..500 {
        let n = rng.below(7);
        let args: Vec<&str> = (0..n).map(|_| rng.pick(WORDS)).collect();
        let (code, sink) = run_mem(&runtime, &args);
        assert!(
            [0, 1, 64, 65, 66, 67, 68, 70].contains(&code) || (0..=255).contains(&code),
            "{args:?} -> {code}"
        );
        let err = sink.err_text();
        assert!(
            !err.chars()
                .any(|c| (c.is_control() && c != '\n') || c == '\u{202e}'),
            "{args:?}: {err:?}"
        );
        // 実行のたびに、ステートを空に戻す
        *runtime.state.borrow_mut() = None;
    }
}

#[cfg(unix)]
#[test]
fn utf8_でない引数でパニックしない() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    let bad = OsString::from_vec(vec![0xff, 0xfe, b'x']);
    let runtime = MemRuntime::new(THREE_JOBS);
    let mut sink = FakeSink::default();
    let env = FakeEnv::default();
    let signals = FakeSignals::new(SignalMode::Noop);
    for args in [
        vec![OsString::from("run"), bad.clone()],
        vec![bad.clone()],
        vec![
            OsString::from("--config"),
            bad.clone(),
            OsString::from("watch"),
        ],
        vec![
            OsString::from("run"),
            OsString::from("a"),
            OsString::from("--bogus"),
            bad.clone(),
        ],
        vec![OsString::from("--"), bad.clone()],
        vec![
            OsString::from("convert"),
            bad.clone(),
            OsString::from("--to"),
            bad.clone(),
        ],
        vec![bad.clone()],
    ] {
        let code = run_with_fakes(&runtime, &mut sink, &env, &signals, args);
        assert!((0..=255).contains(&code));
    }
}

/// ステートを、ランダムに壊す(定義と整合する並びの上で、状態・再開位置・フラグを乱す)。
fn random_state(rng: &mut Rng) -> PipelineState {
    let ids = ["a", "b", "c"];
    let statuses = [
        JobStatus::Pending,
        JobStatus::Success,
        JobStatus::Warning,
        JobStatus::Error,
        JobStatus::Skipped,
    ];
    let reasons = [
        None,
        Some(SkipReason::ByTransition),
        Some(SkipReason::ByEnd),
    ];
    let job_states = ids
        .iter()
        .map(|id| JobState {
            job_id: id.to_string(),
            status: statuses[rng.below(5)],
            exit_code: [None, Some(0), Some(-5), Some(300)][rng.below(4)],
            skip_reason: reasons[rng.below(3)],
        })
        .collect();
    let resume = ["a", "b", "c", "zzz", ""][rng.below(5)];
    PipelineState {
        definition_key: "k".into(),
        pipeline_name: "p".into(),
        definition_path: "mem.toml".into(),
        job_order: ids.iter().map(|s| s.to_string()).collect(),
        job_states,
        resume_job_id: if rng.below(3) == 0 {
            None
        } else {
            Some(resume.to_string())
        },
        end_reached: rng.below(2) == 0,
        completed: rng.below(4) == 0,
        run_id: None,
    }
}

#[test]
fn 壊れたステートを返す偽の_runtime_でもパニックしない() {
    let mut rng = Rng(0xDEAD_BEEF_CAFE_F00D);
    let runtime = MemRuntime::new(THREE_JOBS);
    let commands: [&[&str]; 6] = [
        &["run", "a"],
        &["run", "b"],
        &["run", "c"],
        &["pipeline", "p"],
        &["watch"],
        &["reset"],
    ];
    for _ in 0..600 {
        *runtime.state.borrow_mut() = Some(random_state(&mut rng));
        runtime.write_ok.set(rng.below(5) != 0);
        runtime
            .exit_codes
            .borrow_mut()
            .extend([rng.below(300) as i32 - 20, 0, 1, 256]);
        let command = commands[rng.below(commands.len())];
        let (code, sink) = run_mem(&runtime, command);
        assert!((0..=255).contains(&code), "{command:?} -> {code}");
        assert!(!sink.err_text().contains('\x1b'));
        runtime.exit_codes.borrow_mut().clear();
    }
}

#[test]
fn 空のジョブidや制御文字を含むジョブidでもパニックしない() {
    let runtime = MemRuntime::new(THREE_JOBS);
    for id in ["", "\u{1b}[2J", "a\nb", "\u{202e}evil", &"x".repeat(10_000)] {
        let (code, sink) = run_mem(&runtime, &["run", id]);
        assert_eq!(code, 68, "{id:?}");
        assert!(!sink.err_text().contains('\x1b'));
        // 使い方の一覧(機能が増えると長くなる)を含めても、1 万文字の入力は、そのまま出ない。
        assert!(sink.err_text().len() < 3_000, "長い入力は切って表示する");
    }
}
