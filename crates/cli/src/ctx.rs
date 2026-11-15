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

//! 流れの共通部分(Orchestrator の土台)。
//!
//! `run` と `pipeline` に共通の、実行の前の検査(W2 の 1〜5。BR2.1、BR2.2、BR3.1)と、
//! ジョブを実行したあとの表示、ステートの書き込みの失敗の表示を持つ。

use std::path::{Path, PathBuf};
use std::sync::Arc;

use conductor_core::{
    Classification, DefinitionError, JobDefinition, JobExecution, JobOutcome, NextStep,
    PipelineDefinition, PipelineState, TransitionOn, TransitionTarget,
};
use conductor_runtime::ConsistencyResult;
use conductor_runtime::{
    Durability, InterruptSignal, StateError, StateLocation, check_consistency,
};

use crate::deps::{Runtime, SignalError, Signals, Sink};
use crate::exit::{EXIT_DEFINITION, EXIT_INTERNAL, EXIT_STATE, EXIT_USAGE, exit_code_for};
use crate::report::{DisplayEvent, Reporter, ResultDetail, StateContext};
use crate::types::{CliCommand, UsageError, UsageErrorKind};

/// 既定の定義ファイル(BR1.4)。
const DEFAULT_DEFINITION: &str = "conductor.toml";

/// 流れが使う、差し替え口と表示の束。
pub(crate) struct Ctx<'a> {
    pub runtime: &'a dyn Runtime,
    pub signals: &'a dyn Signals,
    pub sink: &'a mut dyn Sink,
    pub reporter: Reporter<'a>,
}

/// 実行の前の検査を終えた、1 回の実行の材料。
pub(crate) struct Session {
    pub definition_path: PathBuf,
    pub definition: PipelineDefinition,
    /// ステート(`None` は空。完了のまま残ったものも空として返る)。
    pub state: Option<PipelineState>,
    /// 中断の合図(シグナルの監視が書き、`run_job` が読む)。
    pub signal: Arc<InterruptSignal>,
}

/// 定義ファイルの場所(BR1.4)。`--config` があればそのパス、なければ実行ディレクトリの `conductor.toml`。
pub(crate) fn definition_path(cmd: &CliCommand) -> PathBuf {
    match &cmd.config_path {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from(DEFAULT_DEFINITION),
    }
}

impl Ctx<'_> {
    pub(crate) fn show(&mut self, event: &DisplayEvent<'_>) -> bool {
        self.reporter.show(&mut *self.sink, event)
    }

    /// 実行の前の検査(BR2.1、BR2.2、BR3.1)。誤りは表示して、終了コードを `Err` で返す。
    ///
    /// 順序: シグナルの準備(定義の検査より前。BR4.1)→ 定義(64)→ command(64)→ ジョブID(68)→
    /// ステートの読み込み(65)→ 照合(65)。`expected_pipeline` は、`pipeline` の名前の照合に渡す。
    pub(crate) fn prepare(
        &mut self,
        cmd: &CliCommand,
        expected_pipeline: Option<&str>,
        require_job: Option<&str>,
    ) -> Result<Session, i32> {
        let signal = Arc::new(InterruptSignal::new());
        match self.signals.start(&signal) {
            Ok(()) | Err(SignalError::AlreadyRegistered) => {}
            Err(SignalError::Failed) => {
                self.show(&DisplayEvent::SignalSetupFailed);
                return Err(EXIT_INTERNAL);
            }
        }

        let path = definition_path(cmd);
        let definition = match self.runtime.load_definition(&path, expected_pipeline) {
            Ok(definition) => definition,
            Err(errors) => return Err(self.definition_errors(&errors)),
        };

        let command_errors: Vec<_> = definition
            .jobs
            .iter()
            .filter_map(|job| {
                self.runtime
                    .check_command(&job.command)
                    .err()
                    .map(|e| (job.id.clone(), e))
            })
            .collect();
        if !command_errors.is_empty() {
            self.show(&DisplayEvent::CommandErrors {
                definition: &path,
                errors: &command_errors,
            });
            return Err(EXIT_DEFINITION);
        }

        if let Some(job_id) = require_job {
            if definition.job_index(job_id).is_none() {
                let error = UsageError::with_subject(UsageErrorKind::UnknownJob, job_id);
                self.show(&DisplayEvent::Usage(&error));
                return Err(EXIT_USAGE);
            }
        }

        let state = match self.runtime.read_state(&path) {
            Ok(state) => state,
            Err(error) => return Err(self.state_error(&error, StateContext::Read)),
        };
        if let Some(state) = &state {
            if let ConsistencyResult::Mismatch(kind) = check_consistency(state, &definition) {
                let state_path = StateLocation::of(&path).path().to_path_buf();
                self.show(&DisplayEvent::ConsistencyMismatch {
                    state_path: &state_path,
                    kind,
                });
                return Err(EXIT_STATE);
            }
        }

        Ok(Session {
            definition_path: path,
            definition,
            state,
            signal,
        })
    }

    pub(crate) fn definition_errors(&mut self, errors: &[DefinitionError]) -> i32 {
        self.show(&DisplayEvent::DefinitionErrors(errors));
        EXIT_DEFINITION
    }

    /// ステートの誤りを表示して、65 を返す。
    pub(crate) fn state_error(&mut self, error: &StateError, context: StateContext) -> i32 {
        self.show(&DisplayEvent::State { error, context });
        EXIT_STATE
    }

    /// ステートを書く。失敗は、表示して、65 を `Err` で返す。
    pub(crate) fn write(
        &mut self,
        path: &Path,
        state: &PipelineState,
        durability: Durability,
        context: StateContext,
    ) -> Result<(), i32> {
        match self.runtime.write_state(path, state, durability) {
            Ok(()) => Ok(()),
            Err(error) => Err(self.state_error(&error, context)),
        }
    }

    /// ステートを初期化する。失敗は、表示して、65 を `Err` で返す。
    pub(crate) fn reset(&mut self, path: &Path, context: StateContext) -> Result<(), i32> {
        match self.runtime.reset_state(path) {
            Ok(()) => Ok(()),
            Err(error) => Err(self.state_error(&error, context)),
        }
    }

    /// ジョブを実行したあとの表示(BR2.6、BR6.3、BR6.4)。`-v` の結果・遷移・中断と、
    /// 常に表示する起動の失敗・成功コードに含まれない 0。
    pub(crate) fn report_execution(
        &mut self,
        job: &JobDefinition,
        execution: &JobExecution,
        outcome: &JobOutcome,
        next: &NextStep,
    ) {
        let is_error = outcome.classification == Some(Classification::Error);
        let conductor_exit = exit_code_for(outcome.reported_exit_code, is_error);
        let result = match outcome.classification {
            Some(Classification::Success) => "success",
            Some(Classification::Warning) => "warning",
            Some(Classification::Error) => "error",
            None => "interrupted",
        };
        let detail = result_detail(execution);
        self.show(&DisplayEvent::JobResult {
            job_id: &job.id,
            result,
            detail,
            conductor_exit,
        });

        if detail == ResultDetail::StartFailure {
            let name = self
                .runtime
                .check_command(&job.command)
                .map(|line| line.program)
                .unwrap_or_else(|_| {
                    job.command
                        .split_whitespace()
                        .next()
                        .unwrap_or_default()
                        .to_string()
                });
            self.show(&DisplayEvent::StartFailure {
                job_id: &job.id,
                command: &name,
            });
        }
        if outcome.conductor_error {
            self.show(&DisplayEvent::SuccessCodeNotListed { job_id: &job.id });
        }

        let on = match outcome.classification {
            Some(Classification::Success) => Some(TransitionOn::Success),
            Some(Classification::Warning) => Some(TransitionOn::Warning),
            _ => None,
        };
        if let Some(on) = on {
            if let Some(transition) = job.transition_for(on) {
                let target = match &transition.next {
                    TransitionTarget::Job(id) => id.clone(),
                    TransitionTarget::End => "@end".to_string(),
                };
                let on_word = match on {
                    TransitionOn::Success => "success",
                    TransitionOn::Warning => "warning",
                };
                self.show(&DisplayEvent::Transition {
                    on: on_word,
                    target: &target,
                    skipped: &next.skipped_job_ids,
                });
            }
        }
        if execution.interrupted {
            self.show(&DisplayEvent::Interrupted {
                signal: execution.signal_number.unwrap_or(0),
            });
        }
    }
}

/// 実行の結果の補足(`classify` の判定の順序と同じ)。
fn result_detail(execution: &JobExecution) -> ResultDetail {
    if !execution.started {
        return if execution.interrupted {
            ResultDetail::Interrupted {
                signal: execution.signal_number.unwrap_or(0),
                started: false,
            }
        } else {
            ResultDetail::StartFailure
        };
    }
    if execution.interrupted {
        return ResultDetail::Interrupted {
            signal: execution.signal_number.unwrap_or(0),
            started: true,
        };
    }
    if execution.start_failure {
        return ResultDetail::StartFailure;
    }
    if let Some(signal) = execution.terminated_by_signal {
        return ResultDetail::Signal(signal);
    }
    ResultDetail::ExitCode(execution.exit_code.unwrap_or(0))
}
