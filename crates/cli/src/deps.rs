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

//! 差し替え口(Deps)と、本番の実装(Q2: A)。
//!
//! 流れ(Orchestrator)は、小さな trait(`Runtime`、`Sink`、`Env`、`Signals`)を受け取る。
//! 本番は conductor-core・conductor-runtime・conductor-converter の実際の関数と、標準出力・標準エラー、実際の環境、
//! 実際のシグナルの監視。テストは、失敗を起こす偽の実装を渡す。
//!
//! 純粋な関数(`decide_from_state`、`classify_and_decide`、`apply_*` など)は、trait を通さず、
//! 流れが直接呼ぶ。

use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use conductor_converter::{ConversionError, ConversionRequest, TargetFormat};
use conductor_core::{
    Command, DefinitionError, EnvFileError, EnvFileRef, JobExecution, LoadedEnvFiles,
    PipelineDefinition, PipelineState, RunId, Timestamp,
};
use conductor_runtime::{
    CommandError, CommandLine, CreateError, CreateOutcome, Durability, InterruptSignal, LogOutcome,
    LogTarget, ResetAllReport, StateError, WaitOutcome,
};

use crate::platform;

/// 環境(環境変数、標準出力が端末かどうか)。
pub trait Env {
    /// 環境変数の値(UTF-8 でない値、未設定は `None`)。
    fn var(&self, name: &str) -> Option<String>;
    /// 標準出力が端末か(色の判定。BR6.5)。
    fn stdout_is_terminal(&self) -> bool;
}

/// 標準出力・標準エラーへの書き込みの出口。失敗は、値で返す(パニックにしない。NFR4.7)。
pub trait Sink {
    /// 標準出力に書き、すぐに見えるようにする(行ごとに、直接書く)。
    fn write_out(&mut self, bytes: &[u8]) -> io::Result<()>;
    /// 標準エラーに書き、すぐに見えるようにする。
    fn write_err(&mut self, bytes: &[u8]) -> io::Result<()>;
}

/// シグナルの準備の失敗。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalError {
    /// 登録できなかった。内部の誤り(終了コード 70)。
    Failed,
}

/// シグナルの監視(SignalWatcher)の差し替え口。
pub trait Signals {
    /// 受け取ったシグナルのうち、最初の 1 つの番号を、`signal` に書く監視を始める。
    ///
    /// `signal` は、監視のスレッドに渡すため、共有する(`Arc`)。
    ///
    /// 「最初の番号」は、監視が `raise` を呼んだ順の最初である。ほぼ同時に複数のシグナルが届くと、
    /// `signal-hook` の `Signals` が、番号の小さい順に返すため、到着順ではなく、小さい番号が
    /// 先になりうる(例: INT(2)と HUP(1)なら HUP)。時間を空けて届いたときは、到着順である。
    ///
    /// プロセスで 1 回だけ呼ぶ(本番は `prepare_session` の 1 か所)。2 回呼んだときの動きは契約の外
    /// (UNIX 系は監視が 2 つになり、Windows は `Failed`)。
    fn start(&self, signal: &Arc<InterruptSignal>) -> Result<(), SignalError>;
}

/// conductor-core・conductor-runtime・conductor-converter の、入出力に触れる関数の薄い包み。
pub trait Runtime {
    /// 定義ファイルを読み、検証する(`load_definition`)。
    fn load_definition(
        &self,
        path: &Path,
        expected_pipeline_name: Option<&str>,
    ) -> Result<PipelineDefinition, Vec<DefinitionError>>;
    /// command を、起動せずに検査する(`check_command`)。
    fn check_command(&self, command: &Command) -> Result<CommandLine, CommandError>;
    /// `env_file` の `.env` を読む(`load_env_files`。BR1.7、BR3.x、BR6.4。無くてもよいファイルは
    /// v0.10.0 の BR2.2〜BR2.5)。
    fn load_env_files(
        &self,
        definition_path: &Path,
        refs: &[EnvFileRef],
    ) -> Result<LoadedEnvFiles, Vec<EnvFileError>>;
    /// 現在の時刻(`now`。v0.10.0 の BR2.3、契約 C2)。流れが試行の前に 1 回だけ呼ぶ。
    fn now(&self) -> Timestamp;
    /// ジョブを、分割済みのコマンド `line` で、`env` を足した環境で起動し、終了を待つ(`run_job`。BR2.2)。
    /// `log` があれば、出力をそのログのファイルにも残す(v0.10.0 の BR4.x)。ログの結果を添えて返す。
    fn run_job(
        &self,
        line: &CommandLine,
        env: &[(&str, &str)],
        signal: &InterruptSignal,
        log: Option<&LogTarget>,
    ) -> (JobExecution, LogOutcome);
    /// リトライの前に `secs` 秒待つ(`wait_for`。BR2.13)。合図が立てば、すぐ `Interrupted` を返す。
    fn wait(&self, secs: u64, signal: &InterruptSignal) -> WaitOutcome;
    /// ステートを読む(`read_state`。`None` は空)。`run_id` は、ファイル名の run ID。
    fn read_state(
        &self,
        definition_path: &Path,
        run_id: Option<&RunId>,
    ) -> Result<Option<PipelineState>, StateError>;
    /// ステートを、完了のものも含めて読む(`read_state_raw`。`watch` と `graph --state` が使う。`None` はファイルなし・長さ 0)。
    fn read_state_raw(
        &self,
        definition_path: &Path,
        run_id: Option<&RunId>,
    ) -> Result<Option<PipelineState>, StateError>;
    /// ステートを書く(`write_state`)。
    fn write_state(
        &self,
        definition_path: &Path,
        run_id: Option<&RunId>,
        state: &PipelineState,
        durability: Durability,
    ) -> Result<(), StateError>;
    /// ステートを消す(`reset_state`)。
    fn reset_state(&self, definition_path: &Path, run_id: Option<&RunId>)
    -> Result<(), StateError>;
    /// その定義のステートを、run ID の有無によらず、すべて消す(`reset_all_states`)。
    fn reset_all_states(&self, definition_path: &Path) -> Result<ResetAllReport, StateError>;
    /// 定義ファイルの `[pipeline].run_id_env` を、ゆるく読む(ほかの誤りは無視。読めない、
    /// キーがない、文字列でない、名前の形が違う、のどれでも `None`)。
    fn run_id_env(&self, definition_path: &Path) -> Option<String>;
    /// 入力ファイルを、`target` の形式の文字列にする(`convert`)。
    fn convert(&self, input: &Path, target: TargetFormat) -> Result<String, ConversionError>;
    /// ファイルを、排他的に作る(`create_file`。`init` が使う。U4 の BR1.3)。`force` なら、置き換える。
    fn create_file(
        &self,
        path: &Path,
        bytes: &[u8],
        force: bool,
    ) -> Result<CreateOutcome, CreateError>;
}

/// 流れに渡す、差し替え口の束。
pub struct Deps<'a> {
    pub runtime: &'a dyn Runtime,
    pub sink: &'a mut dyn Sink,
    pub env: &'a dyn Env,
    pub signals: &'a dyn Signals,
}

/// 本番の `Runtime`: 他の Unit の実際の関数を呼ぶ。
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemRuntime;

impl Runtime for SystemRuntime {
    fn load_definition(
        &self,
        path: &Path,
        expected_pipeline_name: Option<&str>,
    ) -> Result<PipelineDefinition, Vec<DefinitionError>> {
        conductor_core::load_definition(Some(path), expected_pipeline_name)
    }

    fn check_command(&self, command: &Command) -> Result<CommandLine, CommandError> {
        conductor_runtime::check_command(command)
    }

    fn load_env_files(
        &self,
        definition_path: &Path,
        refs: &[EnvFileRef],
    ) -> Result<LoadedEnvFiles, Vec<EnvFileError>> {
        conductor_runtime::load_env_files(definition_path, refs)
    }

    fn now(&self) -> Timestamp {
        conductor_runtime::now()
    }

    fn run_job(
        &self,
        line: &CommandLine,
        env: &[(&str, &str)],
        signal: &InterruptSignal,
        log: Option<&LogTarget>,
    ) -> (JobExecution, LogOutcome) {
        conductor_runtime::run_job(line, env, signal, log)
    }

    fn wait(&self, secs: u64, signal: &InterruptSignal) -> WaitOutcome {
        conductor_runtime::wait_for(secs, signal)
    }

    fn read_state(
        &self,
        definition_path: &Path,
        run_id: Option<&RunId>,
    ) -> Result<Option<PipelineState>, StateError> {
        conductor_runtime::read_state(definition_path, run_id)
    }

    fn read_state_raw(
        &self,
        definition_path: &Path,
        run_id: Option<&RunId>,
    ) -> Result<Option<PipelineState>, StateError> {
        conductor_runtime::read_state_raw(definition_path, run_id)
    }

    fn write_state(
        &self,
        definition_path: &Path,
        run_id: Option<&RunId>,
        state: &PipelineState,
        durability: Durability,
    ) -> Result<(), StateError> {
        conductor_runtime::write_state(definition_path, run_id, state, durability)
    }

    fn reset_state(
        &self,
        definition_path: &Path,
        run_id: Option<&RunId>,
    ) -> Result<(), StateError> {
        conductor_runtime::reset_state(definition_path, run_id)
    }

    fn reset_all_states(&self, definition_path: &Path) -> Result<ResetAllReport, StateError> {
        conductor_runtime::reset_all_states(definition_path)
    }

    fn run_id_env(&self, definition_path: &Path) -> Option<String> {
        // 取り込みと同じ大きさの上限で読む。読めなければ、既定の名前に任せる(`None`)。
        let text = conductor_core::definition::intake::read_definition(definition_path).ok()?;
        conductor_core::peek_run_id_env(&text)
    }

    fn convert(&self, input: &Path, target: TargetFormat) -> Result<String, ConversionError> {
        let request = ConversionRequest {
            input_path: PathBuf::from(input),
            target_format: target,
        };
        conductor_converter::convert(&request).map(|converted| converted.text)
    }

    fn create_file(
        &self,
        path: &Path,
        bytes: &[u8],
        force: bool,
    ) -> Result<CreateOutcome, CreateError> {
        conductor_runtime::create_file(path, bytes, force)
    }
}

/// 本番の `Env`: 実際の環境。
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemEnv;

impl Env for SystemEnv {
    fn var(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }

    fn stdout_is_terminal(&self) -> bool {
        io::stdout().is_terminal()
    }
}

/// 本番の `Sink`: 実際の標準出力・標準エラー。
#[derive(Debug, Default, Clone, Copy)]
pub struct StdSink;

impl Sink for StdSink {
    fn write_out(&mut self, bytes: &[u8]) -> io::Result<()> {
        let mut out = io::stdout().lock();
        out.write_all(bytes)?;
        out.flush()
    }

    fn write_err(&mut self, bytes: &[u8]) -> io::Result<()> {
        let mut err = io::stderr().lock();
        err.write_all(bytes)?;
        err.flush()
    }
}

/// 本番の `Signals`: UNIX 系は `signal-hook`、Windows は `ctrlc`(`platform` モジュール)。
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemSignals;

impl Signals for SystemSignals {
    fn start(&self, signal: &Arc<InterruptSignal>) -> Result<(), SignalError> {
        platform::start_signal_watcher(Arc::clone(signal))
    }
}
