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
//! 純粋な関数(`decide_before_run`、`classify_and_decide`、`apply_*` など)は、trait を通さず、
//! 流れが直接呼ぶ。

use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use conductor_converter::{ConversionError, ConversionRequest, TargetFormat};
use conductor_core::{
    DefinitionError, JobDefinition, JobExecution, PipelineDefinition, PipelineState,
};
use conductor_runtime::{CommandError, CommandLine, Durability, InterruptSignal, StateError};

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
    /// すでに登録済み(プロセスで 1 回だけ登録できる)。流れは、成功と同様に続ける。
    AlreadyRegistered,
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
    fn check_command(&self, command: &str) -> Result<CommandLine, CommandError>;
    /// ジョブを起動し、終了を待つ(`run_job`)。
    fn run_job(&self, job: &JobDefinition, signal: &InterruptSignal) -> JobExecution;
    /// ステートを読む(`read_state`。`None` は空)。
    fn read_state(&self, definition_path: &Path) -> Result<Option<PipelineState>, StateError>;
    /// ステートを書く(`write_state`)。
    fn write_state(
        &self,
        definition_path: &Path,
        state: &PipelineState,
        durability: Durability,
    ) -> Result<(), StateError>;
    /// ステートを消す(`reset_state`)。
    fn reset_state(&self, definition_path: &Path) -> Result<(), StateError>;
    /// 入力ファイルを、`target` の形式の文字列にする(`convert`)。
    fn convert(&self, input: &Path, target: TargetFormat) -> Result<String, ConversionError>;
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

    fn check_command(&self, command: &str) -> Result<CommandLine, CommandError> {
        conductor_runtime::check_command(command)
    }

    fn run_job(&self, job: &JobDefinition, signal: &InterruptSignal) -> JobExecution {
        conductor_runtime::run_job(job, signal)
    }

    fn read_state(&self, definition_path: &Path) -> Result<Option<PipelineState>, StateError> {
        conductor_runtime::read_state(definition_path)
    }

    fn write_state(
        &self,
        definition_path: &Path,
        state: &PipelineState,
        durability: Durability,
    ) -> Result<(), StateError> {
        conductor_runtime::write_state(definition_path, state, durability)
    }

    fn reset_state(&self, definition_path: &Path) -> Result<(), StateError> {
        conductor_runtime::reset_state(definition_path)
    }

    fn convert(&self, input: &Path, target: TargetFormat) -> Result<String, ConversionError> {
        let request = ConversionRequest {
            input_path: PathBuf::from(input),
            target_format: target,
        };
        conductor_converter::convert(&request).map(|converted| converted.text)
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
