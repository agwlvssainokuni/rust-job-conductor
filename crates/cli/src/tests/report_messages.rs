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

//! 表示(Reporter。BR6.1〜BR6.5、NFR6.x、NFR4.2): 全種類の文言(英語・日本語)、ロケール、遅延、色。

use std::path::Path;

use super::common;
use super::common::kinds::{
    CONVERSION_ERROR_KINDS, CREATE_ERRORS, DEFINITION_ERROR_KINDS, ENV_FILE_ERROR_KINDS,
    USAGE_ERROR_KINDS,
};
use super::common::{FakeEnv, FakeSink};
use crate::report::{DisplayEvent, Reporter, ResultDetail, StateContext};
use crate::types::{Language, Level, UsageError, UsageErrorKind};
use conductor_converter::ConversionError;
use conductor_core::{
    Attempt, Classification, DefinitionError, DefinitionErrorKind, EnvFileError, EnvFileErrorKind,
    EnvWarning, EnvWarningKind, JobState, JobStatus, MismatchKind, PipelineState, RejectReason,
    SkipCause, SkipReason, Timestamp, TransitionOn, TransitionTarget,
};
use conductor_runtime::{CommandError, CommandErrorKind, CreateError, StateError, StateErrorKind};

fn env_for(lang: Language) -> FakeEnv {
    FakeEnv::for_language(lang)
}

fn state_error(kind: StateErrorKind) -> StateError {
    common::state_error(Path::new("dir/c.toml.state"), kind)
}

fn pipeline_state() -> PipelineState {
    let job = |id: &str, status, exit_code, skip_reason| JobState {
        job_id: id.to_string(),
        status,
        exit_code,
        skip_reason,
        attempts: vec![],
    };
    PipelineState {
        pipeline_name: "nightly".into(),
        definition_path: "c.toml".into(),
        job_states: vec![
            JobState {
                // 2026-11-25T09:12:48Z から 2026-11-25T09:13:05Z までの 1 回の試行(17 秒)。
                attempts: vec![attempt(1_795_597_968, Some(1_795_597_985))],
                ..job("extract", JobStatus::Warning, Some(1), None)
            },
            job(
                "transform",
                JobStatus::Skipped,
                None,
                Some(SkipReason::ByTransition),
            ),
            job("load", JobStatus::Skipped, None, Some(SkipReason::ByEnd)),
            job("x", JobStatus::Pending, None, None),
        ],
        resume_job_id: Some("x".into()),
        end_reached: false,
        completed: false,
        run_id: None,
    }
}

/// `start` 秒に始まり、`end` 秒に終わった(`None` は終わりの無い)試行。
fn attempt(start: u64, end: Option<u64>) -> Attempt {
    let ts = |secs| Timestamp::from_unix_seconds(secs).expect("範囲の中");
    Attempt {
        started_at: ts(start),
        ended_at: end.map(ts),
    }
}

/// `watch` の表のイベント。今の時刻は 2026-11-25T09:13:53Z(`pipeline_state` の試行の時刻と合わせる)。
fn watch_table(state: &PipelineState) -> DisplayEvent<'_> {
    DisplayEvent::WatchTable {
        state,
        now: Timestamp::from_unix_seconds(1_795_598_033).expect("範囲の中"),
    }
}

/// 全種類のイベントを作って、`f` に渡す。
fn with_all_events(f: impl Fn(&DisplayEvent<'_>)) {
    let definition_errors: Vec<DefinitionError> = DEFINITION_ERROR_KINDS
        .iter()
        .map(|kind| DefinitionError::new("c.toml", Some(7), *kind, "reason `k`"))
        .collect();
    f(&DisplayEvent::DefinitionErrors(&definition_errors));
    f(&DisplayEvent::DefinitionErrors(&[DefinitionError::new(
        "c.toml",
        None,
        DefinitionErrorKind::FileMissing,
        "reason",
    )]));

    // `.env` の誤りと警告(U1)。全種類と、20 件を超える打ち切り。
    let env_errors: Vec<EnvFileError> = ENV_FILE_ERROR_KINDS
        .iter()
        .map(|kind| EnvFileError::new("x.env", Some(3), *kind))
        .collect();
    f(&DisplayEvent::EnvFileErrors(&env_errors));
    f(&DisplayEvent::EnvFileErrors(&[EnvFileError::new(
        "x.env",
        None,
        EnvFileErrorKind::FileMissing,
    )]));
    let many: Vec<EnvFileError> = (1..=25)
        .map(|i| EnvFileError::new("x.env", Some(i), EnvFileErrorKind::MissingEquals))
        .collect();
    f(&DisplayEvent::EnvFileErrors(&many));
    f(&DisplayEvent::EnvWarnings(&[EnvWarning {
        file: "x.env".into(),
        line: 5,
        name: "A".into(),
        kind: EnvWarningKind::DuplicateKey,
    }]));

    let command_errors = vec![
        (
            "a".to_string(),
            CommandError {
                kind: CommandErrorKind::UnclosedQuote,
            },
        ),
        (
            "b".to_string(),
            CommandError {
                kind: CommandErrorKind::Empty,
            },
        ),
    ];
    f(&DisplayEvent::CommandErrors {
        definition: Path::new("c.toml"),
        errors: &command_errors,
    });

    for kind in USAGE_ERROR_KINDS {
        f(&DisplayEvent::Usage(&UsageError::new(kind)));
        f(&DisplayEvent::Usage(&UsageError::with_subject(
            kind, "subj",
        )));
    }

    for kind in [
        StateErrorKind::Unreadable,
        StateErrorKind::TooLarge,
        StateErrorKind::Corrupt,
        StateErrorKind::VersionMismatch,
        StateErrorKind::WriteFailed,
        StateErrorKind::RemoveFailed,
    ] {
        let error = state_error(kind);
        for context in [
            StateContext::Read,
            StateContext::JobNotStarted,
            StateContext::JobExecuted,
            StateContext::Plain,
        ] {
            f(&DisplayEvent::State {
                error: &error,
                context,
            });
        }
    }

    for kind in [
        MismatchKind::PipelineName,
        MismatchKind::JobIds,
        MismatchKind::JobOrder,
    ] {
        f(&DisplayEvent::ConsistencyMismatch {
            state_path: Path::new("c.toml.state"),
            kind,
            context: StateContext::Read,
        });
    }
    f(&DisplayEvent::Rejected {
        job_id: "b",
        reason: RejectReason::OutOfOrder,
    });
    for reason in [
        SkipCause::SkipTarget,
        SkipCause::AfterEnd,
        SkipCause::AlreadyExecuted,
    ] {
        f(&DisplayEvent::JobSkipped {
            job_id: "b",
            reason,
        });
    }
    f(&DisplayEvent::StartFailure {
        job_id: "a",
        command: "python",
    });
    f(&DisplayEvent::SuccessCodeNotListed { job_id: "a" });
    for kind in CONVERSION_ERROR_KINDS {
        f(&DisplayEvent::Conversion(&ConversionError {
            file: "in.yaml".into(),
            line: Some(2),
            key_path: Some("job[1].note".into()),
            kind,
            reason: "reason",
        }));
        f(&DisplayEvent::Conversion(&ConversionError {
            file: "in.yaml".into(),
            line: None,
            key_path: None,
            kind,
            reason: "reason",
        }));
    }
    f(&DisplayEvent::OutputFailed);
    f(&DisplayEvent::CompletionUndetermined {
        state_path: Path::new("c.toml.state"),
    });
    f(&DisplayEvent::SignalSetupFailed);

    let skipped = vec!["b".to_string(), "c".to_string()];
    for detail in [
        ResultDetail::ExitCode(0),
        ResultDetail::ExitCode(256),
        ResultDetail::Signal(9),
        ResultDetail::StartFailure,
        ResultDetail::Interrupted {
            signal: 15,
            started: true,
        },
        ResultDetail::Interrupted {
            signal: 15,
            started: false,
        },
    ] {
        for result in [
            Some(Classification::Success),
            Some(Classification::Warning),
            Some(Classification::Error),
            None,
        ] {
            f(&DisplayEvent::JobResult {
                job_id: "a",
                result,
                detail,
                conductor_exit: 1,
            });
        }
    }
    f(&DisplayEvent::Transition {
        on: TransitionOn::Warning,
        target: &TransitionTarget::Job("c".into()),
        skipped: &skipped,
    });
    f(&DisplayEvent::Transition {
        on: TransitionOn::Success,
        target: &TransitionTarget::End,
        skipped: &[],
    });
    f(&DisplayEvent::Interrupted { signal: 15 });
    f(&DisplayEvent::RetryScheduled {
        job_id: "a",
        last_exit_code: 75,
        retry_number: 1,
        max: 3,
        wait_secs: 30,
    });
    let state = pipeline_state();
    f(&watch_table(&state));
    f(&DisplayEvent::WatchEmpty);
    f(&DisplayEvent::Graph("* a\no @end\n"));
    f(&DisplayEvent::Plan("plan\n"));
    f(&DisplayEvent::InitTemplate("template\n"));
    let init_path = Path::new("dir/conductor.toml");
    f(&DisplayEvent::InitCreated { path: init_path });
    f(&DisplayEvent::InitReplaced { path: init_path });
    f(&DisplayEvent::InitExists { path: init_path });
    for error in CREATE_ERRORS {
        f(&DisplayEvent::InitFailed {
            path: init_path,
            error,
        });
    }
    f(&DisplayEvent::InitStdoutFailed);
    let log_path = Path::new("logs/a.20261125T091248Z.1.log");
    f(&DisplayEvent::LogNotCreated { path: log_path });
    f(&DisplayEvent::LogWriteFailed { path: log_path });
    f(&DisplayEvent::LogOutputCutShort { path: log_path });
    f(&DisplayEvent::Help);
    f(&DisplayEvent::Version);
}

fn has_control(text: &str) -> bool {
    text.chars()
        .any(|c| (c.is_control() && c != '\n') || c == '\u{202e}')
}

#[test]
fn 全種類のイベントが空でなく_改行で終わり_生の制御文字を含まない() {
    for lang in [Language::En, Language::Ja] {
        let env = env_for(lang);
        let reporter = Reporter::new(&env, true);
        with_all_events(|event| {
            let text = reporter.render(event);
            assert!(!text.is_empty(), "{event:?}");
            assert!(text.ends_with('\n'), "{event:?}: {text:?}");
            assert!(!has_control(&text), "{event:?}: {text:?}");
        });
    }
}

#[test]
fn エラーの行は_conductor_error_で始まり_場所を含む() {
    let env = env_for(Language::En);
    let reporter = Reporter::new(&env, false);
    with_all_events(|event| {
        if event.level() != Level::Error || matches!(event, DisplayEvent::EnvWarnings(_)) {
            return;
        }
        let text = reporter.render(event);
        for line in text.lines().take_while(|l| !l.is_empty()) {
            assert!(line.starts_with("conductor: error: "), "{event:?}: {line}");
            assert!(line.len() > "conductor: error: ".len(), "{event:?}");
        }
    });
}

#[test]
fn 英語の表示は英語の文だけで日本語の文字を含まない() {
    // 上流(u1・u2・u4)の理由の文は日本語の固定の文のため、テストでは英語の理由を渡している。
    let env = env_for(Language::En);
    let reporter = Reporter::new(&env, true);
    with_all_events(|event| {
        let text = reporter.render(event);
        assert!(text.is_ascii(), "{event:?}: {text}");
    });
}

#[test]
fn 日本語の表示は日本語の文を含む() {
    let env = env_for(Language::Ja);
    let reporter = Reporter::new(&env, true);
    with_all_events(|event| {
        let text = reporter.render(event);
        let japanese = !text.is_ascii();
        // 固定の英語の文、または、トークンだけでできている行
        let token_only = match event {
            DisplayEvent::SignalSetupFailed
            | DisplayEvent::Version
            | DisplayEvent::Graph(_)
            | DisplayEvent::Plan(_)
            | DisplayEvent::InitTemplate(_) => true,
            DisplayEvent::Transition { skipped, .. } => skipped.is_empty(),
            _ => false,
        };
        assert!(japanese || token_only, "{event:?}: {text}");
    });
}

#[test]
fn 定義の誤りは_ファイル_行_種類_原因を_1_行ずつ() {
    let env = env_for(Language::En);
    let reporter = Reporter::new(&env, false);
    let errors = vec![
        DefinitionError::new("c.toml", Some(3), DefinitionErrorKind::UnknownKey, "R1"),
        DefinitionError::new("c.toml", None, DefinitionErrorKind::NoJobs, "R2"),
    ];
    let text = reporter.render(&DisplayEvent::DefinitionErrors(&errors));
    assert_eq!(
        text,
        "conductor: error: c.toml:3: unknown key: R1\nconductor: error: c.toml: no jobs are defined: R2\n"
    );
}

#[test]
fn 定義の誤りの日本語は_u1_の種類の文を使う() {
    let env = env_for(Language::Ja);
    let reporter = Reporter::new(&env, false);
    let errors = vec![DefinitionError::new(
        "c.toml",
        Some(3),
        DefinitionErrorKind::UnknownKey,
        "R1",
    )];
    assert_eq!(
        reporter.render(&DisplayEvent::DefinitionErrors(&errors)),
        "conductor: error: c.toml:3: 定義にないキーです: R1\n"
    );
}

/// リトライの設定の誤りの種類(U1。BR1.14、FR7.1)。
fn retry_kinds() -> [DefinitionErrorKind; 10] {
    use DefinitionErrorKind as K;
    [
        K::RetryNotTable,
        K::RetryMissingKey,
        K::RetryExitCodesEmpty,
        K::RetryExitCodeInvalid,
        K::RetryExitCodesOverlap,
        K::RetryMaxInvalid,
        K::RetryDurationInvalid,
        K::RetryBackoffIncomplete,
        K::RetryMultiplierInvalid,
        K::RetryMaxIntervalTooSmall,
    ]
}

#[test]
fn リトライの設定の誤りは英語の固定の文を持つ() {
    let env = env_for(Language::En);
    let reporter = Reporter::new(&env, false);
    let mut seen = std::collections::HashSet::new();
    for kind in retry_kinds() {
        let errors = vec![DefinitionError::new("c.toml", Some(5), kind, "R")];
        let text = reporter.render(&DisplayEvent::DefinitionErrors(&errors));
        assert!(text.is_ascii(), "{kind:?}: {text}");
        assert!(text.starts_with("conductor: error: c.toml:5: "), "{text}");
        assert!(text.contains("retry"), "{text}");
        assert!(seen.insert(text), "{kind:?} の文が重複している");
    }
}

#[test]
fn リトライの設定の誤りは日本語の固定の文を持つ() {
    let env = env_for(Language::Ja);
    let reporter = Reporter::new(&env, false);
    let mut seen = std::collections::HashSet::new();
    for kind in retry_kinds() {
        let errors = vec![DefinitionError::new("c.toml", Some(5), kind, "R")];
        let text = reporter.render(&DisplayEvent::DefinitionErrors(&errors));
        assert!(!text.is_ascii(), "{kind:?}: {text}");
        assert!(text.contains("retry"), "{text}");
        assert!(seen.insert(text), "{kind:?} の文が重複している");
    }
}

#[test]
fn 入力に由来する文字列は_エスケープされる() {
    let env = env_for(Language::En);
    let reporter = Reporter::new(&env, true);
    let evil = "a\x1b[31m\n\u{202e}b";
    let expected = "a\\u{1b}[31m\\n\\u{202e}b";
    let texts = [
        reporter.render(&DisplayEvent::Rejected {
            job_id: evil,
            reason: RejectReason::OutOfOrder,
        }),
        reporter.render(&DisplayEvent::StartFailure {
            job_id: evil,
            command: evil,
        }),
        reporter.render(&DisplayEvent::SuccessCodeNotListed { job_id: evil }),
        reporter.render(&DisplayEvent::JobSkipped {
            job_id: evil,
            reason: SkipCause::AfterEnd,
        }),
        reporter.render(&DisplayEvent::Usage(&UsageError::with_subject(
            UsageErrorKind::UnknownCommand,
            evil,
        ))),
        reporter.render(&DisplayEvent::DefinitionErrors(&[DefinitionError::new(
            evil,
            Some(1),
            DefinitionErrorKind::Syntax,
            evil,
        )])),
        reporter.render(&DisplayEvent::State {
            error: &StateError {
                path: Path::new(evil).to_path_buf(),
                kind: StateErrorKind::Corrupt,
                reason: "r",
            },
            context: StateContext::Read,
        }),
        reporter.render(&DisplayEvent::CommandErrors {
            definition: Path::new(evil),
            errors: &[(
                evil.to_string(),
                CommandError {
                    kind: CommandErrorKind::Empty,
                },
            )],
        }),
    ];
    for text in texts {
        assert!(text.contains(expected), "{text}");
        assert!(!has_control(&text), "{text:?}");
    }
}

#[test]
fn ジョブidは_64_文字で切り_パスは切らない() {
    let env = env_for(Language::En);
    let reporter = Reporter::new(&env, true);
    let long = "k".repeat(200);
    let text = reporter.render(&DisplayEvent::SuccessCodeNotListed { job_id: &long });
    assert!(text.contains(&format!("{}…", "k".repeat(64))), "{text}");
    assert!(!text.contains(&"k".repeat(65)));
    let text = reporter.render(&DisplayEvent::State {
        error: &StateError {
            path: Path::new(&long).to_path_buf(),
            kind: StateErrorKind::WriteFailed,
            reason: "r",
        },
        context: StateContext::Plain,
    });
    assert!(text.contains(&long), "パスは切らない");
}

#[test]
fn ステートの誤りの補足は_文脈で決まる() {
    let env = env_for(Language::En);
    let reporter = Reporter::new(&env, false);
    let corrupt = state_error(StateErrorKind::Corrupt);
    let write = state_error(StateErrorKind::WriteFailed);
    let render =
        |error: &StateError, context| reporter.render(&DisplayEvent::State { error, context });
    assert!(render(&corrupt, StateContext::Read).contains("conductor reset"));
    assert!(!render(&write, StateContext::Read).contains("conductor reset"));
    assert!(render(&write, StateContext::JobExecuted).contains("already been executed"));
    assert!(render(&write, StateContext::JobNotStarted).contains("was not started"));
    let plain = render(&write, StateContext::Plain);
    assert!(
        !plain.contains("executed") && !plain.contains("not started"),
        "{plain}"
    );
    assert!(plain.contains("dir/c.toml.state"));
}

#[test]
fn v_の文言() {
    let env = env_for(Language::En);
    let reporter = Reporter::new(&env, true);
    assert_eq!(
        reporter.render(&DisplayEvent::JobResult {
            job_id: "extract",
            result: Some(Classification::Warning),
            detail: ResultDetail::ExitCode(1),
            conductor_exit: 0,
        }),
        "[conductor] job=extract result=warning (exit code 1)\n"
    );
    assert_eq!(
        reporter.render(&DisplayEvent::JobResult {
            job_id: "x",
            result: Some(Classification::Error),
            detail: ResultDetail::ExitCode(256),
            conductor_exit: 1,
        }),
        "[conductor] job=x result=error (exit code 256, conductor exits with 1)\n"
    );
    let skipped = vec!["transform".to_string(), "load".to_string()];
    assert_eq!(
        reporter.render(&DisplayEvent::Transition {
            on: TransitionOn::Warning,
            target: &TransitionTarget::Job("notify".into()),
            skipped: &skipped,
        }),
        "[conductor] transition: warning -> notify (skipped: transform, load)\n"
    );
    assert_eq!(
        reporter.render(&DisplayEvent::JobSkipped {
            job_id: "transform",
            reason: SkipCause::SkipTarget,
        }),
        "[conductor] job=transform skipped (skipped by a transition)\n"
    );
    assert_eq!(
        reporter.render(&DisplayEvent::Interrupted { signal: 15 }),
        "[conductor] interrupted by signal 15\n"
    );
}

#[test]
fn 定義が正しいときの_v_の文言は_ジョブの数を含み_入力の値を含まない() {
    let env_en = env_for(Language::En);
    let env_ja = env_for(Language::Ja);
    let reporter = Reporter::new(&env_en, true);
    assert_eq!(
        reporter.render(&DisplayEvent::DefinitionValid { job_count: 4 }),
        "[conductor] definition is valid (4 jobs)\n"
    );
    assert_eq!(
        reporter.render(&DisplayEvent::DefinitionValid { job_count: 1 }),
        "[conductor] definition is valid (1 job)\n"
    );
    let reporter = Reporter::new(&env_ja, true);
    assert_eq!(
        reporter.render(&DisplayEvent::DefinitionValid { job_count: 4 }),
        "[conductor] 定義は正しいです(ジョブ 4 個)\n"
    );
    // -v がなければ何も出さない。
    let reporter = Reporter::new(&env_en, false);
    let mut sink = FakeSink::default();
    reporter.show(&mut sink, &DisplayEvent::DefinitionValid { job_count: 4 });
    assert!(sink.out.is_empty() && sink.err.is_empty());
}

#[test]
fn level_ごとに出力先が決まる() {
    let env = env_for(Language::En);
    // error: 常に標準エラー
    let reporter = Reporter::new(&env, false);
    let mut sink = FakeSink::default();
    reporter.show(&mut sink, &DisplayEvent::OutputFailed);
    assert!(sink.out.is_empty() && !sink.err.is_empty());
    // verbose: -v のときだけ
    let mut sink = FakeSink::default();
    reporter.show(&mut sink, &DisplayEvent::Interrupted { signal: 2 });
    assert!(sink.out.is_empty() && sink.err.is_empty());
    let reporter = Reporter::new(&env, true);
    reporter.show(&mut sink, &DisplayEvent::Interrupted { signal: 2 });
    assert!(sink.out.is_empty() && !sink.err.is_empty());
    // output: 標準出力。失敗は戻り値
    let mut sink = FakeSink::default();
    assert!(reporter.show(&mut sink, &DisplayEvent::WatchEmpty));
    assert!(!sink.out.is_empty() && sink.err.is_empty());
    sink.fail_out = true;
    assert!(!reporter.show(&mut sink, &DisplayEvent::WatchEmpty));
    // 標準エラーの失敗は無視する
    sink.fail_err = true;
    assert!(reporter.show(&mut sink, &DisplayEvent::OutputFailed));
}

#[test]
fn 計画は標準出力に出て_使い方に_dry_run_と_assume_がある() {
    for lang in [Language::En, Language::Ja] {
        let env = env_for(lang);
        let reporter = Reporter::new(&env, false);
        let mut sink = FakeSink::default();
        assert!(reporter.show(&mut sink, &DisplayEvent::Plan("plan\n")));
        assert_eq!(sink.out_text(), "plan\n");
        assert!(sink.err.is_empty());
        let usage = reporter.render(&DisplayEvent::Help);
        assert!(
            usage.contains("--dry-run") && usage.contains("--assume"),
            "{usage}"
        );
    }
}

#[test]
fn ロケールの判定() {
    let cases: [(&[(&str, &str)], Language); 8] = [
        (&[("LANG", "ja_JP.UTF-8")], Language::Ja),
        (&[("LANG", "en_US.UTF-8")], Language::En),
        (&[], Language::En),
        (
            &[("LC_ALL", "en_US.UTF-8"), ("LANG", "ja_JP.UTF-8")],
            Language::En,
        ),
        (
            &[("LC_ALL", "ja_JP.UTF-8"), ("LANG", "en_US.UTF-8")],
            Language::Ja,
        ),
        (
            &[("LC_ALL", ""), ("LC_MESSAGES", "ja"), ("LANG", "en")],
            Language::Ja,
        ),
        (&[("LC_MESSAGES", "fr_FR"), ("LANG", "ja_JP")], Language::En),
        (&[("LANG", "C")], Language::En),
    ];
    for (vars, expected) in cases {
        let env = FakeEnv::with(vars);
        assert_eq!(Reporter::new(&env, false).language(), expected, "{vars:?}");
    }
}

#[test]
fn 言語と色の判定は_最初に必要になるまで行われない() {
    let env = FakeEnv::with(&[("LANG", "ja_JP.UTF-8")]);
    let reporter = Reporter::new(&env, false);
    assert!(!env.locale_read() && !env.color_read());
    assert_eq!(env.var_calls.get() + env.terminal_calls.get(), 0);
    // -v なしの verbose イベントは、組み立てない
    let mut sink = FakeSink::default();
    reporter.show(&mut sink, &DisplayEvent::Interrupted { signal: 2 });
    assert!(!env.locale_read());
    assert_eq!(env.var_calls.get(), 0);
    // 最初の表示で、言語だけが判定され、結果が保たれる
    reporter.show(&mut sink, &DisplayEvent::OutputFailed);
    assert!(env.locale_read() && !env.color_read());
    let calls = env.var_calls.get();
    assert!(calls > 0);
    reporter.show(&mut sink, &DisplayEvent::OutputFailed);
    assert_eq!(env.var_calls.get(), calls, "判定は 1 回だけ");
    assert_eq!(env.terminal_calls.get(), 0, "端末の判定は、watch の表だけ");
}

#[test]
fn インスタンスどうしは互いに影響しない() {
    let ja = FakeEnv::with(&[("LANG", "ja_JP.UTF-8")]);
    let en = FakeEnv::with(&[("LANG", "en_US.UTF-8")]);
    let a = Reporter::new(&ja, false);
    let b = Reporter::new(&en, false);
    assert_eq!(a.language(), Language::Ja);
    assert_eq!(b.language(), Language::En);
    assert_eq!(a.language(), Language::Ja);
}

#[test]
fn バージョンとシグナルの準備の失敗は_ロケールを判定しない() {
    let env = FakeEnv::with(&[("LANG", "ja_JP.UTF-8")]);
    let reporter = Reporter::new(&env, false);
    let text = reporter.render(&DisplayEvent::SignalSetupFailed);
    assert_eq!(text, "conductor: error: failed to set up signal handling\n");
    assert!(
        reporter
            .render(&DisplayEvent::Version)
            .starts_with("conductor ")
    );
    assert_eq!(env.var_calls.get(), 0);
    assert!(!env.locale_read());
}

#[test]
fn 色は端末で_no_color_が無いか空で_unix_のときだけ() {
    let unix = cfg!(unix);
    let cases = [
        (true, None, unix),
        (true, Some(""), unix),
        (true, Some("1"), false),
        (false, None, false),
        (false, Some("1"), false),
    ];
    for (terminal, no_color, expected) in cases {
        let mut env = FakeEnv {
            terminal,
            ..FakeEnv::default()
        };
        if let Some(v) = no_color {
            env.vars.insert("NO_COLOR".into(), v.into());
        }
        assert_eq!(
            Reporter::new(&env, false).color_enabled(),
            expected,
            "terminal={terminal} NO_COLOR={no_color:?}"
        );
    }
}

#[test]
fn watch_の表は_4_列で_状態は文字で書く() {
    let env = env_for(Language::En);
    let reporter = Reporter::new(&env, false);
    let text = reporter.render(&watch_table(&pipeline_state()));
    let expected = "\
Pipeline: nightly
JOB        STATUS   DURATION  NOTE
extract    warning  17s       exit code 1
transform  skipped            skipped by a transition
load       skipped            skipped after @end
x          pending
";
    assert_eq!(text, expected);
    assert!(!text.contains('\x1b'));
}

/// ジョブ 1 つ(状態 `status`、試行 `attempts`)のステートの `watch` の表の、そのジョブの所要時間の欄。
fn duration_of(status: JobStatus, attempts: Vec<Attempt>, now: u64) -> String {
    let env = env_for(Language::En);
    let reporter = Reporter::new(&env, false);
    let mut state = pipeline_state();
    state.job_states.truncate(1);
    state.job_states[0].status = status;
    state.job_states[0].attempts = attempts;
    let text = reporter.render(&DisplayEvent::WatchTable {
        state: &state,
        now: Timestamp::from_unix_seconds(now).expect("範囲の中"),
    });
    let row = text.lines().nth(2).expect("ジョブの行");
    // 列: ジョブ、状態、所要時間、補足(2 つの空白で区切る)。
    row.split("  ")
        .map(str::trim)
        .filter(|cell| !cell.is_empty())
        .nth(2)
        .filter(|cell| !cell.starts_with("exit code"))
        .unwrap_or("")
        .to_string()
}

#[test]
fn u6_br4_1_watch_の所要時間は表のとおり() {
    let t0 = 1_795_597_968;
    // 終わったジョブは、最初の始まりから最後の終わりまで(リトライの待ちを含む)。
    assert_eq!(
        duration_of(
            JobStatus::Success,
            vec![attempt(t0, Some(t0 + 17))],
            t0 + 999
        ),
        "17s"
    );
    assert_eq!(
        duration_of(
            JobStatus::Error,
            vec![attempt(t0, Some(t0 + 10)), attempt(t0 + 40, Some(t0 + 65))],
            t0 + 999
        ),
        "1m05s"
    );
    assert_eq!(
        duration_of(JobStatus::Warning, vec![attempt(t0, Some(t0 + 7384))], t0),
        "2h03m04s"
    );
    // 動いている最中の pending は、今までの経過時間。
    assert_eq!(
        duration_of(JobStatus::Pending, vec![attempt(t0, None)], t0 + 60),
        "1m00s"
    );
    assert_eq!(
        duration_of(JobStatus::Pending, vec![attempt(t0, None)], t0 + 3600),
        "1h00m00s"
    );
    // 終わりが始まりより前(時計の戻り)は 0 秒。
    assert_eq!(
        duration_of(JobStatus::Pending, vec![attempt(t0, None)], t0 - 5),
        "0s"
    );
    assert_eq!(
        duration_of(JobStatus::Success, vec![attempt(t0, Some(t0 + 59))], t0),
        "59s"
    );
    // 空にするもの: 試行が無い、最後の試行が終わった pending、結果のあるジョブで終わりの無い試行。
    assert_eq!(duration_of(JobStatus::Pending, vec![], t0), "");
    assert_eq!(
        duration_of(JobStatus::Pending, vec![attempt(t0, Some(t0 + 3))], t0 + 9),
        ""
    );
    assert_eq!(
        duration_of(JobStatus::Success, vec![attempt(t0, None)], t0 + 9),
        ""
    );
}

#[test]
fn u6_br4_1_watch_の所要時間の見出しは日本語では所要時間() {
    let env = env_for(Language::Ja);
    let reporter = Reporter::new(&env, false);
    let text = reporter.render(&watch_table(&pipeline_state()));
    let header = text.lines().nth(1).expect("見出しの行");
    assert!(header.contains("所要時間"), "{header}");
    let order: Vec<usize> = ["ジョブ", "状態", "所要時間", "補足"]
        .iter()
        .map(|h| header.find(h).expect("見出しがある"))
        .collect();
    assert!(order.windows(2).all(|w| w[0] < w[1]), "列の順: {header}");
    assert!(text.contains("17s"), "{text}");
}

#[test]
fn watch_の表は端末のとき_状態に色をつけても_文字は残る() {
    let mut env = env_for(Language::En);
    env.terminal = true;
    let reporter = Reporter::new(&env, false);
    let text = reporter.render(&watch_table(&pipeline_state()));
    if cfg!(unix) {
        assert!(text.contains("\x1b[33mwarning\x1b[0m"), "{text:?}");
        assert!(text.contains("\x1b[36mskipped\x1b[0m"));
        assert!(!text.contains("\x1b[0mpending"), "pending は色なし");
        let plain: String = strip_ansi(&text);
        assert!(
            plain.contains("extract    warning  17s       exit code 1"),
            "{plain}"
        );
    } else {
        assert!(!text.contains('\x1b'));
    }
}

fn strip_ansi(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            for n in chars.by_ref() {
                if n == 'm' {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[test]
fn watch_の表は日本語の見出しと補足() {
    let env = env_for(Language::Ja);
    let reporter = Reporter::new(&env, false);
    let text = reporter.render(&watch_table(&pipeline_state()));
    assert!(text.starts_with("パイプライン: nightly\n"), "{text}");
    assert!(text.contains("ジョブ") && text.contains("状態") && text.contains("補足"));
    assert!(text.contains("終了コード 1"));
    assert!(text.contains("遷移によりスキップ") && text.contains("@end の後ろのためスキップ"));
    // 状態は、言語に関わらず英語の文字
    assert!(text.contains("warning") && text.contains("pending"));
}

#[test]
fn watch_の空の表示() {
    let env = env_for(Language::En);
    let reporter = Reporter::new(&env, false);
    assert_eq!(
        reporter.render(&DisplayEvent::WatchEmpty),
        "no state (nothing has run yet)\n"
    );
}

#[test]
fn ジョブidの制御文字は表の中でもエスケープされる() {
    let env = env_for(Language::En);
    let reporter = Reporter::new(&env, false);
    let mut state = pipeline_state();
    state.job_states[0].job_id = "e\x1b[31mx".to_string();
    state.pipeline_name = "n\nm".to_string();
    let text = reporter.render(&watch_table(&state));
    assert!(text.contains("e\\u{1b}[31mx"));
    assert!(text.contains("n\\nm"));
    assert!(!text.contains('\x1b'));
}

#[test]
fn 使い方は種類ごとの固定の文と使い方の一覧() {
    let env = env_for(Language::En);
    let reporter = Reporter::new(&env, false);
    let text = reporter.render(&DisplayEvent::Usage(&UsageError::with_subject(
        UsageErrorKind::UnknownCommand,
        "start",
    )));
    assert!(text.starts_with("conductor: error: unknown command: 'start'\n\nUsage:\n"));
    for command in [
        "run", "pipeline", "watch", "reset", "validate", "convert", "graph",
    ] {
        assert!(text.contains(&format!("conductor {command}")), "{command}");
    }
}

// ---- リトライの知らせ(U2。BR2.4、BR2.15。FR6.1、FR6.2) ----

fn retry_event(job_id: &str) -> DisplayEvent<'_> {
    DisplayEvent::RetryScheduled {
        job_id,
        last_exit_code: 75,
        retry_number: 2,
        max: 3,
        wait_secs: 40,
    }
}

#[test]
fn リトライの知らせの英語の文は_要素をすべて含み_エラーの形ではない() {
    let env = env_for(Language::En);
    let reporter = Reporter::new(&env, false);
    let text = reporter.render(&retry_event("a"));
    assert_eq!(
        text,
        "conductor: retry: job 'a' exited with code 75; retry 2 of 3 in 40 seconds\n"
    );
    assert!(!text.contains("error"), "{text}");
}

#[test]
fn リトライの知らせの日本語の文は_要素をすべて含む() {
    let env = env_for(Language::Ja);
    let reporter = Reporter::new(&env, false);
    let text = reporter.render(&retry_event("a"));
    assert_eq!(
        text,
        "conductor: retry: ジョブ 'a' が終了コード 75 で終わりました。40 秒後にリトライします(2/3 回目)\n"
    );
}

#[test]
fn リトライの知らせは_v_なしでも標準エラーに出て_標準出力には出ない() {
    let env = env_for(Language::En);
    let reporter = Reporter::new(&env, false);
    let mut sink = FakeSink::default();
    assert_eq!(retry_event("a").level(), Level::Notice);
    assert!(reporter.show(&mut sink, &retry_event("a")));
    assert!(sink.out.is_empty());
    assert!(sink.err_text().starts_with("conductor: retry: "));
}

#[test]
fn リトライの知らせの_job_id_はエスケープされ_64_文字で切られる() {
    let env = env_for(Language::En);
    let reporter = Reporter::new(&env, false);
    let id = format!("x\u{1b}[31m\n{}", "y".repeat(100));
    let text = reporter.render(&retry_event(&id));
    assert!(!has_control(text.trim_end_matches('\n')), "{text:?}");
    assert_eq!(text.matches('\n').count(), 1, "{text:?}");
    assert!(!text.contains(&"y".repeat(65)), "{text:?}");
}

#[test]
fn env_の誤りは_20_件で打ち切り_警告は行と名前を持つ_日本語と英語() {
    let many: Vec<EnvFileError> = (1..=25)
        .map(|i| EnvFileError::new("x.env", Some(i), EnvFileErrorKind::MissingEquals))
        .collect();
    let warn = [EnvWarning {
        file: "x.env".into(),
        line: 5,
        name: "A\nB".into(),
        kind: EnvWarningKind::DuplicateKey,
    }];
    let en_env = env_for(Language::En);
    let en = Reporter::new(&en_env, false);
    let text = en.render(&DisplayEvent::EnvFileErrors(&many));
    assert_eq!(text.lines().count(), 21, "{text}");
    assert!(
        text.lines().last().unwrap().ends_with("and 5 more"),
        "{text}"
    );
    let text = en.render(&DisplayEvent::EnvWarnings(&warn));
    assert!(
        text.starts_with("conductor: warning: x.env:5: duplicate key `A\\nB`"),
        "{text}"
    );

    let ja_env = env_for(Language::Ja);
    let ja = Reporter::new(&ja_env, false);
    let text = ja.render(&DisplayEvent::EnvFileErrors(&many));
    assert!(
        text.lines().last().unwrap().ends_with("ほか 5 件"),
        "{text}"
    );
    let text = ja.render(&DisplayEvent::EnvWarnings(&warn));
    assert!(text.contains("重複キー"), "{text}");
}

// ---- init(U4。BR1.2、BR1.4、BR3.2、BR4.1) ----

#[test]
fn init_のメッセージは_仕様の表のとおり() {
    let path = Path::new("a/conductor.toml");
    let cases = [
        (
            DisplayEvent::InitCreated { path },
            "created a/conductor.toml\n",
            "a/conductor.toml を作成しました\n",
        ),
        (
            DisplayEvent::InitReplaced { path },
            "replaced a/conductor.toml\n",
            "a/conductor.toml を置き換えました\n",
        ),
        (
            DisplayEvent::InitExists { path },
            "conductor: error: a/conductor.toml already exists (use --force to overwrite)\n",
            "conductor: error: a/conductor.toml は既にあります(上書きするには --force)\n",
        ),
        (
            DisplayEvent::InitFailed {
                path,
                error: CreateError::ParentMissing,
            },
            "conductor: error: cannot create a/conductor.toml: the parent directory does not exist\n",
            "conductor: error: a/conductor.toml を作れません: 親のディレクトリがありません\n",
        ),
        (
            DisplayEvent::InitFailed {
                path,
                error: CreateError::IsDirectory,
            },
            "conductor: error: cannot create a/conductor.toml: it is a directory\n",
            "conductor: error: a/conductor.toml を作れません: ディレクトリです\n",
        ),
        (
            DisplayEvent::InitFailed {
                path,
                error: CreateError::PermissionDenied,
            },
            "conductor: error: cannot create a/conductor.toml: permission denied\n",
            "conductor: error: a/conductor.toml を作れません: 書く権限がありません\n",
        ),
        (
            DisplayEvent::InitFailed {
                path,
                error: CreateError::OtherIo,
            },
            "conductor: error: cannot create a/conductor.toml: I/O error\n",
            "conductor: error: a/conductor.toml を作れません: 入出力の誤りです\n",
        ),
        (
            DisplayEvent::InitStdoutFailed,
            "conductor: error: cannot write the template to standard output\n",
            "conductor: error: ひな形を標準出力に書けません\n",
        ),
    ];
    for (event, en, ja) in cases {
        for (lang, expected) in [(Language::En, en), (Language::Ja, ja)] {
            let env = env_for(lang);
            let reporter = Reporter::new(&env, false);
            assert_eq!(reporter.render(&event), expected, "{event:?} {lang:?}");
        }
    }
}

#[test]
fn init_のパスは_制御文字をエスケープして表示する() {
    let env = env_for(Language::En);
    let reporter = Reporter::new(&env, false);
    let path = Path::new("a\nb\u{1b}[31m.toml");
    for event in [
        DisplayEvent::InitCreated { path },
        DisplayEvent::InitExists { path },
        DisplayEvent::InitFailed {
            path,
            error: CreateError::OtherIo,
        },
    ] {
        let text = reporter.render(&event);
        assert_eq!(text.matches('\n').count(), 1, "{text:?}");
        assert!(!text.contains('\u{1b}'), "{text:?}");
    }
}

#[test]
fn init_の水準は_成功が通知_誤りがエラー_ひな形が標準出力() {
    let path = Path::new("c.toml");
    assert_eq!(DisplayEvent::InitCreated { path }.level(), Level::Notice);
    assert_eq!(DisplayEvent::InitReplaced { path }.level(), Level::Notice);
    assert_eq!(DisplayEvent::InitExists { path }.level(), Level::Error);
    assert_eq!(DisplayEvent::InitStdoutFailed.level(), Level::Error);
    assert_eq!(DisplayEvent::InitTemplate("x").level(), Level::Output);
}

#[test]
fn 使い方に_init_と_force_がある() {
    for lang in [Language::En, Language::Ja] {
        let env = env_for(lang);
        let usage = Reporter::new(&env, false).render(&DisplayEvent::Help);
        assert!(usage.contains("conductor init"), "{usage}");
        assert!(usage.contains("--force"), "{usage}");
    }
}

// ---- 種類の文の出どころ(1j、D10) ----

#[test]
fn 日本語の誤りの表示は_各クレートの_label_を使う() {
    let env = env_for(Language::Ja);
    let reporter = Reporter::new(&env, false);
    // 定義の誤り(33 種)。
    for kind in DEFINITION_ERROR_KINDS {
        let errors = [DefinitionError::new("c.toml", Some(1), kind, "R")];
        let text = reporter.render(&DisplayEvent::DefinitionErrors(&errors));
        assert_eq!(
            text,
            format!("conductor: error: c.toml:1: {}: R\n", kind.label()),
            "{kind:?}"
        );
    }
    // `.env` の誤り(13 種)。種類の文と理由の両方。
    for kind in ENV_FILE_ERROR_KINDS {
        let errors = [EnvFileError::new("x.env", Some(2), kind)];
        let text = reporter.render(&DisplayEvent::EnvFileErrors(&errors));
        assert_eq!(
            text,
            format!(
                "conductor: error: x.env:2: {}: {}\n",
                kind.label(),
                kind.reason()
            ),
            "{kind:?}"
        );
    }
    // 変換の誤り(6 種)。
    for kind in CONVERSION_ERROR_KINDS {
        let error = ConversionError {
            file: "in.yaml".into(),
            line: Some(3),
            key_path: None,
            kind,
            reason: "R",
        };
        let text = reporter.render(&DisplayEvent::Conversion(&error));
        assert_eq!(
            text,
            format!("conductor: error: in.yaml:3: {}: R\n", kind.label()),
            "{kind:?}"
        );
    }
    // ステートの誤り(6 種)。
    for kind in [
        StateErrorKind::Unreadable,
        StateErrorKind::TooLarge,
        StateErrorKind::Corrupt,
        StateErrorKind::VersionMismatch,
        StateErrorKind::WriteFailed,
        StateErrorKind::RemoveFailed,
    ] {
        let error = state_error(kind);
        let text = reporter.render(&DisplayEvent::State {
            error: &error,
            context: StateContext::Plain,
        });
        assert!(
            text.starts_with(&format!(
                "conductor: error: dir/c.toml.state: {}: ",
                kind.label()
            )),
            "{kind:?}: {text}"
        );
    }
}

// ---- ログのファイルの 3 つの文(v0.10.0 の BR5.1〜BR5.3、NFR6.6) ----

#[test]
fn ログの_3_つの文の英語() {
    let env = env_for(Language::En);
    let reporter = Reporter::new(&env, false);
    let path = Path::new("logs/a_b.20261125T091248Z.1.log");
    assert_eq!(
        reporter.render(&DisplayEvent::LogNotCreated { path }),
        "conductor: error: cannot create the log file: logs/a_b.20261125T091248Z.1.log\n"
    );
    assert_eq!(
        reporter.render(&DisplayEvent::LogWriteFailed { path }),
        "conductor: warning: failed to write to the log file; output after that point is not in the file: logs/a_b.20261125T091248Z.1.log\n"
    );
    assert_eq!(
        reporter.render(&DisplayEvent::LogOutputCutShort { path }),
        "conductor: warning: output continued after the job ended; the rest is not in the log file: logs/a_b.20261125T091248Z.1.log\n"
    );
}

#[test]
fn ログの_3_つの文の日本語() {
    let env = env_for(Language::Ja);
    let reporter = Reporter::new(&env, false);
    let path = Path::new("logs/a.20261125T091248Z.1.log");
    assert_eq!(
        reporter.render(&DisplayEvent::LogNotCreated { path }),
        "conductor: error: ログのファイルを作れません: logs/a.20261125T091248Z.1.log\n"
    );
    assert_eq!(
        reporter.render(&DisplayEvent::LogWriteFailed { path }),
        "conductor: warning: ログのファイルへの書き込みに失敗しました。途中からファイルに残していません: logs/a.20261125T091248Z.1.log\n"
    );
    assert_eq!(
        reporter.render(&DisplayEvent::LogOutputCutShort { path }),
        "conductor: warning: ジョブの終了の後も出力が続いたため、残りを残していません: logs/a.20261125T091248Z.1.log\n"
    );
}

#[test]
fn ログの文のパスはエスケープされ_1_行に収まる() {
    let env = env_for(Language::En);
    let reporter = Reporter::new(&env, false);
    let path = Path::new("logs/x\u{1b}[31m\ny.log");
    for event in [
        DisplayEvent::LogNotCreated { path },
        DisplayEvent::LogWriteFailed { path },
        DisplayEvent::LogOutputCutShort { path },
    ] {
        let text = reporter.render(&event);
        assert_eq!(text.matches('\n').count(), 1, "{text:?}");
        assert!(!has_control(text.trim_end_matches('\n')), "{text:?}");
    }
}

#[test]
fn ログを作れない文はエラーで_警告の_2_つは知らせの水準() {
    let path = Path::new("l.log");
    assert_eq!(DisplayEvent::LogNotCreated { path }.level(), Level::Error);
    assert_eq!(DisplayEvent::LogWriteFailed { path }.level(), Level::Notice);
    assert_eq!(
        DisplayEvent::LogOutputCutShort { path }.level(),
        Level::Notice
    );
    // `-v` なしでも標準エラーに出て、標準出力には出ない。
    let env = env_for(Language::En);
    let reporter = Reporter::new(&env, false);
    let mut sink = FakeSink::default();
    assert!(reporter.show(&mut sink, &DisplayEvent::LogWriteFailed { path }));
    assert!(sink.out.is_empty());
    assert!(sink.err_text().starts_with("conductor: warning: "));
}
