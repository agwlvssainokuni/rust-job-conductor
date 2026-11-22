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
//! ステートの書き込み・初期化とその失敗の表示を持つ。ジョブを実行したあとの表示は `step` にある。

use std::path::{Path, PathBuf};
use std::sync::Arc;

use conductor_core::DEFAULT_RUN_ID_ENV;
use conductor_core::{
    DefinitionError, LoadedEnvFiles, PipelineDefinition, PipelineState, RunId, env_file_refs,
};
use conductor_runtime::ConsistencyResult;
use conductor_runtime::{
    CommandLine, Durability, InterruptSignal, StateError, StateLocation, check_consistency,
};

use crate::deps::{Env, Runtime, SignalError, Signals, Sink};
use crate::exit::{EXIT_DEFINITION, EXIT_INTERNAL, EXIT_OK, EXIT_STATE, EXIT_USAGE};
use crate::report::{DisplayEvent, Reporter, StateContext};
use crate::run_id::resolve_run_id;
use crate::types::{CliCommand, UsageError, UsageErrorKind};

/// 流れが使う、差し替え口と表示の束。
pub(crate) struct Ctx<'a> {
    pub runtime: &'a dyn Runtime,
    pub signals: &'a dyn Signals,
    pub sink: &'a mut dyn Sink,
    /// 環境変数の読み取り(run ID。`Reporter` が使う `Env` と同じもの)。
    pub env: &'a dyn Env,
    pub reporter: Reporter<'a>,
}

/// 検査を終えた定義(`check_definition` の結果)。
pub(crate) struct CheckedDefinition {
    pub path: PathBuf,
    pub definition: PipelineDefinition,
    /// 各ジョブの `command` を分割したもの(ジョブと同じ並び。分割は、定義の検査の 1 回だけ)。
    pub commands: Vec<CommandLine>,
}

/// シグナルの準備を除く、実行の前の検査を終えた材料(`prepare_checked` の結果)。
pub(crate) struct Checked {
    pub definition_path: PathBuf,
    pub definition: PipelineDefinition,
    /// 各ジョブの `command` を分割したもの(ジョブと同じ並び)。
    pub commands: Vec<CommandLine>,
    /// この実行の run ID(`None` は、run ID なし)。ステートのパスと、`run_id` の照合に使う。
    pub run_id: Option<RunId>,
    /// ステート(`None` は空。完了のまま残ったものも空として返る)。
    pub state: Option<PipelineState>,
    /// 実効の `remove_on_complete`(FR2.3): `--remove-on-complete` があれば `true`、なければ
    /// 定義の `[pipeline].remove_on_complete`。環境変数は見ない(FR2.6)。
    pub remove_on_complete: bool,
    /// 読み取った `.env`(この呼び出しの中だけで持つ。ステートには書かない。BR6.3、BR8.3)。
    pub env_files: LoadedEnvFiles,
}

/// 実行の前の検査を終えた、1 回の実行の材料。
pub(crate) struct Session {
    /// 実行の前の検査を終えた材料。
    pub checked: Checked,
    /// 中断の合図(シグナルの監視が書き、`run_job` が読む)。
    pub signal: Arc<InterruptSignal>,
}

/// `run` と `pipeline` の対象(本番と `--dry-run` で共通)。
#[derive(Debug, Clone, Copy)]
pub(crate) enum FlowTarget<'a> {
    /// `pipeline <name>`。
    Pipeline(&'a str),
    /// `run <job-id>`。
    Run(&'a str),
}

/// ステートを読む方法(`read_checked_state`)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StateRead {
    /// `read_state`: 完了のまま残ったステートは、空として読む(`run`・`pipeline`)。
    Live,
    /// `read_state_raw`: 完了のステートも読む(`graph --state`)。
    Raw,
}

/// 定義ファイルの場所(BR1.4)。`--config` があればそのパス、なければ実行ディレクトリの `conductor.toml`
/// (conductor-core の `resolve_path`)。
pub(crate) fn definition_path(cmd: &CliCommand) -> PathBuf {
    conductor_core::definition::intake::resolve_path(cmd.config_path.as_deref().map(Path::new))
}

impl Ctx<'_> {
    pub(crate) fn show(&mut self, event: &DisplayEvent<'_>) -> bool {
        self.reporter.show(&mut *self.sink, event)
    }

    /// 使い方の誤りを表示して、68 を返す。
    pub(crate) fn usage_error(&mut self, error: &UsageError) -> i32 {
        self.show(&DisplayEvent::Usage(error));
        EXIT_USAGE
    }

    /// 標準出力に `output` を書く。書けたら 0、書けなければ(閉じたパイプなど。パニックにしない)
    /// `failed` を表示して 65 を返す(`watch`・`graph`・`--dry-run`・`init`。BR6.3)。
    pub(crate) fn show_output(
        &mut self,
        output: &DisplayEvent<'_>,
        failed: &DisplayEvent<'_>,
    ) -> i32 {
        if self.show(output) {
            EXIT_OK
        } else {
            self.show(failed);
            EXIT_STATE
        }
    }

    /// `run <job-id>` の実行の前の検査(BR2.1、BR2.2、BR3.1)。誤りは表示して、終了コードを `Err` で返す。
    /// 材料と、定義の中のそのジョブの位置を返す。
    ///
    /// 順序: シグナルの準備(定義の検査より前。BR4.1)→ 定義(64)→ command(64)→ ジョブID(68)→
    /// run ID の決定(68)→ `.env` の読み取り(64)→ ステートの読み込み(65)→ 照合(65)。
    pub(crate) fn prepare_run(
        &mut self,
        cmd: &CliCommand,
        job_id: &str,
    ) -> Result<(Session, usize), i32> {
        let signal = self.start_signals()?;
        let (checked, index) = self.check_job(cmd, job_id, &[])?;
        Ok((Session { checked, signal }, index))
    }

    /// `pipeline <name>` の実行の前の検査(BR2.1、BR2.2、BR3.1)。順序は [`Ctx::prepare_run`] から
    /// ジョブIDの検査を除いたもの。`name` は、定義のパイプライン名の照合に渡す。
    pub(crate) fn prepare_pipeline(
        &mut self,
        cmd: &CliCommand,
        name: &str,
    ) -> Result<Session, i32> {
        let signal = self.start_signals()?;
        let checked = self.check_pipeline(cmd, name, &[])?;
        Ok(Session { checked, signal })
    }

    /// 本番の検査の、シグナルの準備を除いた部分(定義の検査、ジョブの存在、run ID の決定、`.env` の
    /// 読み取り、ステートの読み込みと照合。U3 の BR1.1、BR4.1、BR4.2)。`--dry-run` が使う。
    ///
    /// `also_known_jobs` は、ジョブIDの検査と同じ位置(定義の検査のあと、run ID の決定の前)で、
    /// 定義にあるかを確かめるジョブID(`--assume` の指定。ない値は使い方の誤り 68。誤りの表示に、
    /// ジョブIDは含めない。BR2.1)。
    pub(crate) fn prepare_checked(
        &mut self,
        cmd: &CliCommand,
        target: FlowTarget<'_>,
        also_known_jobs: &[&str],
    ) -> Result<Checked, i32> {
        match target {
            FlowTarget::Pipeline(name) => self.check_pipeline(cmd, name, also_known_jobs),
            FlowTarget::Run(job_id) => self
                .check_job(cmd, job_id, also_known_jobs)
                .map(|(checked, _)| checked),
        }
    }

    /// `run <job-id>` の検査(定義 64 → command 64 → ジョブID 68 → `--assume` 68 → run ID 68 →
    /// `.env` 64 → ステート 65)。材料と、定義の中のそのジョブの位置を返す。
    ///
    /// `also_known_jobs` が空なら、`--assume` の検査は何もしない(表示もしない)。
    fn check_job(
        &mut self,
        cmd: &CliCommand,
        job_id: &str,
        also_known_jobs: &[&str],
    ) -> Result<(Checked, usize), i32> {
        let checked = self.check_definition(cmd, None)?;
        let index = self.require_job(&checked.definition, job_id)?;
        self.require_known(&checked.definition, also_known_jobs)?;
        let checked = self.check_run_inputs(cmd, checked, Some(job_id))?;
        Ok((checked, index))
    }

    /// `pipeline <name>` の検査([`Ctx::check_job`] からジョブIDの検査を除いた同じ順)。`name` は、
    /// 定義のパイプライン名の照合に渡す。
    fn check_pipeline(
        &mut self,
        cmd: &CliCommand,
        name: &str,
        also_known_jobs: &[&str],
    ) -> Result<Checked, i32> {
        let checked = self.check_definition(cmd, Some(name))?;
        self.require_known(&checked.definition, also_known_jobs)?;
        self.check_run_inputs(cmd, checked, None)
    }

    /// シグナルの監視を始める(BR4.1)。すでに登録済みなら、そのまま使う。失敗は表示して、70 を `Err` で返す。
    fn start_signals(&mut self) -> Result<Arc<InterruptSignal>, i32> {
        let signal = Arc::new(InterruptSignal::new());
        match self.signals.start(&signal) {
            Ok(()) | Err(SignalError::AlreadyRegistered) => Ok(signal),
            Err(SignalError::Failed) => {
                self.show(&DisplayEvent::SignalSetupFailed);
                Err(EXIT_INTERNAL)
            }
        }
    }

    /// ジョブIDが定義にあることを確かめ、その位置を返す。無ければ、ジョブIDを添えた使い方の誤り(68)。
    fn require_job(&mut self, definition: &PipelineDefinition, job_id: &str) -> Result<usize, i32> {
        match definition.job_index(job_id) {
            Some(index) => Ok(index),
            None => {
                let error = UsageError::with_subject(UsageErrorKind::UnknownJob, job_id);
                Err(self.usage_error(&error))
            }
        }
    }

    /// `--assume` のジョブIDが、すべて定義にあることを確かめる。無ければ、使い方の誤り(68。ジョブIDは
    /// 表示に含めない。BR2.1)。
    fn require_known(
        &mut self,
        definition: &PipelineDefinition,
        job_ids: &[&str],
    ) -> Result<(), i32> {
        if job_ids.iter().any(|id| definition.job_index(id).is_none()) {
            let error = UsageError::new(UsageErrorKind::UnknownJob);
            return Err(self.usage_error(&error));
        }
        Ok(())
    }

    /// ジョブの検査のあとの部分: run ID の決定(68)→ `.env` の読み取り(64)→ ステートの読み込みと
    /// 照合(65)。`env_job` は、読む `.env` の範囲(`run` のジョブID。`pipeline` は `None`)。
    fn check_run_inputs(
        &mut self,
        cmd: &CliCommand,
        checked: CheckedDefinition,
        env_job: Option<&str>,
    ) -> Result<Checked, i32> {
        let CheckedDefinition {
            path,
            definition,
            commands,
        } = checked;

        let run_id = self.definition_run_id(cmd, &definition)?;

        // `.env` の読み取り(BR6.1〜BR6.3)。誤りがあれば、ジョブを実行せず、ステートにも触れずに 64 で止まる。
        let env_files = self.load_env_files(&path, &definition, env_job)?;

        let state =
            self.read_checked_state(&path, &definition, run_id.as_ref(), StateRead::Live)?;

        let remove_on_complete = cmd.remove_on_complete || definition.remove_on_complete;
        Ok(Checked {
            definition_path: path,
            definition,
            commands,
            run_id,
            state,
            remove_on_complete,
            env_files,
        })
    }

    /// 定義を読んだあとの run ID の決定(FR1.2〜FR1.4)。環境変数の名前は、定義の `run_id_env`
    /// (なければ既定)。誤りは表示して、68 を `Err` で返す。`prepare_checked` と `graph --state` が使う。
    pub(crate) fn definition_run_id(
        &mut self,
        cmd: &CliCommand,
        definition: &PipelineDefinition,
    ) -> Result<Option<RunId>, i32> {
        let env_name = definition
            .run_id_env
            .as_deref()
            .unwrap_or(DEFAULT_RUN_ID_ENV);
        match resolve_run_id(cmd.run_id.as_ref(), env_name, self.env) {
            Ok(run_id) => Ok(run_id),
            Err(error) => Err(self.usage_error(&error)),
        }
    }

    /// ステートを読み、定義と照合する(BR2.5)。読めなければ 65、食い違えば 65 を、表示して `Err` で返す。
    /// ステートが無ければ `Ok(None)`。`read` で、完了のステートを空として読むか(`Live`)、
    /// 完了のものも読むか(`Raw`)を選ぶ。
    pub(crate) fn read_checked_state(
        &mut self,
        path: &Path,
        definition: &PipelineDefinition,
        run_id: Option<&RunId>,
        read: StateRead,
    ) -> Result<Option<PipelineState>, i32> {
        let result = match read {
            StateRead::Live => self.runtime.read_state(path, run_id),
            StateRead::Raw => self.runtime.read_state_raw(path, run_id),
        };
        let state = match result {
            Ok(state) => state,
            Err(error) => {
                return Err(self.state_error(&error, StateContext::for_read(run_id.is_some())));
            }
        };
        if let Some(state) = &state
            && let ConsistencyResult::Mismatch(kind) = check_consistency(state, definition)
        {
            let state_path = StateLocation::of(path, run_id).path().to_path_buf();
            self.show(&DisplayEvent::ConsistencyMismatch {
                state_path: &state_path,
                kind,
                with_run_id: run_id.is_some(),
            });
            return Err(EXIT_STATE);
        }
        Ok(state)
    }

    /// 読む対象の `env_file`(`job_id` があればそのジョブの分、なければ全ジョブの分と
    /// `[pipeline].env_file`)を、すべて読む(BR6.1、BR7.1)。誤りは表示して、64 を `Err` で返す(BR2.8)。
    pub(crate) fn load_env_files(
        &mut self,
        definition_path: &Path,
        definition: &PipelineDefinition,
        job_id: Option<&str>,
    ) -> Result<LoadedEnvFiles, i32> {
        let refs = env_file_refs(definition, job_id);
        if refs.is_empty() {
            return Ok(LoadedEnvFiles::new());
        }
        match self.runtime.load_env_files(definition_path, &refs) {
            Ok(loaded) => Ok(loaded),
            Err(errors) => {
                self.show(&DisplayEvent::EnvFileErrors(&errors));
                Err(EXIT_DEFINITION)
            }
        }
    }

    /// 定義の読み込みと検証(64)、全ジョブの `command` の検査(64)。`run`・`pipeline` の準備と `validate`・`graph` が共有する。
    ///
    /// シグナル・ステートには触れない。誤りは表示して、終了コードを `Err` で返す。
    /// `command` の誤りは、全ジョブの分を集めてから表示する。分割したコマンドは、ジョブと同じ並びで返す。
    pub(crate) fn check_definition(
        &mut self,
        cmd: &CliCommand,
        expected_pipeline: Option<&str>,
    ) -> Result<CheckedDefinition, i32> {
        let path = definition_path(cmd);
        let definition = match self.runtime.load_definition(&path, expected_pipeline) {
            Ok(definition) => definition,
            Err(errors) => return Err(self.definition_errors(&errors)),
        };

        let mut commands = Vec::with_capacity(definition.job.len());
        let mut command_errors = Vec::new();
        for job in &definition.job {
            match self.runtime.check_command(&job.command) {
                Ok(line) => commands.push(line),
                Err(error) => command_errors.push((job.id.clone(), error)),
            }
        }
        if !command_errors.is_empty() {
            self.show(&DisplayEvent::CommandErrors {
                definition: &path,
                errors: &command_errors,
            });
            return Err(EXIT_DEFINITION);
        }
        Ok(CheckedDefinition {
            path,
            definition,
            commands,
        })
    }

    /// `validate` と `graph` が共有する検査(BR6.1): 定義の検証(64)、全ジョブの `command` の検査(64)、
    /// `.env` の読み取り(64)。重複キーの警告は、誤りでないので、標準エラーに出して続ける(BR7.2)。
    ///
    /// ジョブは実行しない。シグナルの準備、ステートの読み書き、リセットもしない。値は、どこにも出さない。
    /// 誤りは、`run` と同じ表示で、終了コードを `Err` で返す。
    pub(crate) fn check_definition_and_env(
        &mut self,
        cmd: &CliCommand,
    ) -> Result<CheckedDefinition, i32> {
        let checked = self.check_definition(cmd, cmd.pipeline_name.as_deref())?;
        // `pipeline` と同じ範囲の `.env` を読む(BR7.1)。重複キーの警告は、誤りでないので 0(BR7.2)。
        let loaded = self.load_env_files(&checked.path, &checked.definition, None)?;
        let warnings: Vec<_> = loaded
            .files()
            .iter()
            .flat_map(|f| f.warnings.iter().cloned())
            .collect();
        if !warnings.is_empty() {
            self.show(&DisplayEvent::EnvWarnings(&warnings));
        }
        Ok(checked)
    }

    /// 定義の誤りを表示して、64 を返す。
    fn definition_errors(&mut self, errors: &[DefinitionError]) -> i32 {
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
        run_id: Option<&RunId>,
        state: &PipelineState,
        durability: Durability,
        context: StateContext,
    ) -> Result<(), i32> {
        match self.runtime.write_state(path, run_id, state, durability) {
            Ok(()) => Ok(()),
            Err(error) => Err(self.state_error(&error, context)),
        }
    }

    /// ステートを初期化する。失敗は、表示して、65 を `Err` で返す。
    pub(crate) fn reset(
        &mut self,
        path: &Path,
        run_id: Option<&RunId>,
        context: StateContext,
    ) -> Result<(), i32> {
        match self.runtime.reset_state(path, run_id) {
            Ok(()) => Ok(()),
            Err(error) => Err(self.state_error(&error, context)),
        }
    }
}
