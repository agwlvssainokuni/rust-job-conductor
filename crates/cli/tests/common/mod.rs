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

//! 流れのテストの共通の補助(偽の `Runtime`・`Sink`・`Env`・`Signals`、定義の組み立て)。
#![allow(dead_code)]

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use conductor_cli::{Deps, Env, Runtime, SignalError, Signals, Sink, SystemRuntime, run_cli_with};
use conductor_converter::{ConversionError, TargetFormat};
use conductor_core::{
    Command, DefinitionError, EnvFileError, ExecutionResult, JobExecution, JobStatus,
    LoadedEnvFiles, PipelineDefinition, PipelineState, RunId,
};
use conductor_runtime::{
    CommandError, CommandLine, CreateError, CreateOutcome, Durability, InterruptSignal,
    ResetAllReport, StateError, StateErrorKind, WaitOutcome,
};

// ---- Env ----

/// 偽の環境のロケールを選ぶ、テストの言語(`FakeEnv::for_language` の引数)。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Language {
    En,
    Ja,
}

#[derive(Default)]
pub struct FakeEnv {
    pub vars: HashMap<String, String>,
    pub terminal: bool,
    pub var_calls: Cell<usize>,
    pub terminal_calls: Cell<usize>,
    /// 読まれた環境変数の名前(読んだ順)。
    pub names: RefCell<Vec<String>>,
}

impl FakeEnv {
    /// ロケールの環境変数(`LC_ALL`・`LC_MESSAGES`・`LANG`)が読まれたか(言語の判定の遅延を確かめる)。
    pub fn locale_read(&self) -> bool {
        self.names
            .borrow()
            .iter()
            .any(|n| matches!(n.as_str(), "LC_ALL" | "LC_MESSAGES" | "LANG"))
    }

    /// 色の判定のための問い合わせ(`NO_COLOR` を読んだ、または端末かを問い合わせた)があったか
    /// (色の判定の遅延を確かめる)。
    pub fn color_read(&self) -> bool {
        self.terminal_calls.get() > 0 || self.names.borrow().iter().any(|n| n == "NO_COLOR")
    }

    pub fn with(vars: &[(&str, &str)]) -> Self {
        Self {
            vars: vars
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Self::default()
        }
    }

    /// 言語 `lang` のロケール(`LANG`)だけを持つ環境。
    pub fn for_language(lang: Language) -> Self {
        match lang {
            Language::En => Self::with(&[("LANG", "en_US.UTF-8")]),
            Language::Ja => Self::with(&[("LANG", "ja_JP.UTF-8")]),
        }
    }
}

impl Env for FakeEnv {
    fn var(&self, name: &str) -> Option<String> {
        self.var_calls.set(self.var_calls.get() + 1);
        self.names.borrow_mut().push(name.to_string());
        self.vars.get(name).cloned()
    }

    fn stdout_is_terminal(&self) -> bool {
        self.terminal_calls.set(self.terminal_calls.get() + 1);
        self.terminal
    }
}

// ---- Sink ----

#[derive(Default)]
pub struct FakeSink {
    pub out: Vec<u8>,
    pub err: Vec<u8>,
    pub fail_out: bool,
    pub fail_err: bool,
    pub out_writes: usize,
}

impl FakeSink {
    pub fn out_text(&self) -> String {
        String::from_utf8_lossy(&self.out).into_owned()
    }

    pub fn err_text(&self) -> String {
        String::from_utf8_lossy(&self.err).into_owned()
    }
}

impl Sink for FakeSink {
    fn write_out(&mut self, bytes: &[u8]) -> io::Result<()> {
        if self.fail_out {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed"));
        }
        self.out_writes += 1;
        self.out.extend_from_slice(bytes);
        Ok(())
    }

    fn write_err(&mut self, bytes: &[u8]) -> io::Result<()> {
        if self.fail_err {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed"));
        }
        self.err.extend_from_slice(bytes);
        Ok(())
    }
}

// ---- Signals ----

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SignalMode {
    Noop,
    Fail,
    /// `start` の時点で、合図を立てる(検査より前に届いたシグナル)。
    RaiseOnStart(i32),
}

pub struct FakeSignals {
    pub mode: SignalMode,
    pub started: Cell<usize>,
}

impl FakeSignals {
    pub fn new(mode: SignalMode) -> Self {
        Self {
            mode,
            started: Cell::new(0),
        }
    }
}

impl Signals for FakeSignals {
    fn start(&self, signal: &Arc<InterruptSignal>) -> Result<(), SignalError> {
        self.started.set(self.started.get() + 1);
        match self.mode {
            SignalMode::Noop => Ok(()),
            SignalMode::Fail => Err(SignalError::Failed),
            SignalMode::RaiseOnStart(n) => {
                signal.raise(n);
                Ok(())
            }
        }
    }
}

// ---- Runtime ----

/// 1 回のジョブの起動の結果の指定。
#[derive(Clone, Copy, Debug)]
pub enum Exec {
    Exit(i32),
    KilledBy(i32),
    StartFailure,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum WriteFailure {
    #[default]
    Never,
    /// 書き込みの N 回目(0 始まり)が失敗する。
    OnCall(usize),
    /// その耐久性の書き込みは、すべて失敗する。
    OnDurability(Durability),
}

/// `run_job` に渡された、(ジョブID、足す環境の並び)。
pub type RunEnv = (String, Vec<(String, String)>);

/// `run_job` に渡された、(ジョブID、分割済みのコマンドの写し)。
pub type RunLine = (String, CommandLine);

/// ジョブの前後で呼ぶ関数(引数は、ジョブIDと、中断の合図)。
pub type JobHook = Box<dyn Fn(&str, &InterruptSignal)>;

/// 定義・ステートの入出力は実際の関数(`SystemRuntime`)に任せ、ジョブの起動だけを、台本で偽にする。
/// 失敗は、注入できる。呼び出しの順序を `log` に残す。
#[derive(Default)]
pub struct ScriptedRuntime {
    pub inner: SystemRuntime,
    pub log: RefCell<Vec<String>>,
    pub scripts: RefCell<HashMap<String, VecDeque<Exec>>>,
    pub write_failure: Cell<WriteFailure>,
    pub write_calls: Cell<usize>,
    pub fail_reset: Cell<bool>,
    pub read_failure: RefCell<Option<StateError>>,
    pub convert_result: RefCell<Option<Result<String, ConversionError>>>,
    /// ジョブの実行中に呼ぶ(合図を立てる)。引数は、ジョブID。
    pub during_job: RefCell<Option<JobHook>>,
    /// ジョブの終了を観測したあとに呼ぶ(ジョブのあいだの合図を起こす)。引数は、ジョブID。
    pub after_job: RefCell<Option<JobHook>>,
    pub run_calls: Cell<usize>,
    /// `reset --all` の結果を、実際の削除の代わりに返す(`kept` の表示を確かめる)。
    pub reset_all_override: RefCell<Option<ResetAllReport>>,
    /// `run_id_env`(定義のゆるい読み取り)の呼び出しの回数。
    pub run_id_env_calls: Cell<usize>,
    /// `wait` が呼ばれた待ち時間(秒)。待たずに記録するだけ。
    pub waits: RefCell<Vec<u64>>,
    /// `wait` の N 回目(0 始まり)で、合図(番号)を立てて `Interrupted` を返す。
    pub interrupt_wait: Cell<Option<(usize, i32)>>,
    /// `load_env_files` に渡された `env_file` の記述(呼び出しごと)。
    pub env_loads: RefCell<Vec<Vec<String>>>,
    /// `.env` の読み取りの結果を、実際の読み取りの代わりに返す(誤りの注入)。
    pub env_override: RefCell<Option<Result<LoadedEnvFiles, Vec<EnvFileError>>>>,
    /// `run_job` に渡された、(ジョブID、足す環境の並び)。起動のたびに記録する。
    pub run_envs: RefCell<Vec<RunEnv>>,
    /// `run_job` に渡された、(ジョブID、分割済みのコマンド)。起動のたびに記録する
    /// (合図で起動しないときも、`run_envs` と同じく記録する)。
    pub run_lines: RefCell<Vec<RunLine>>,
    /// `create_file` に渡された、(パス、内容、`force`)。呼び出しのたびに記録する(U4)。
    pub create_calls: RefCell<Vec<CreateCall>>,
    /// `create_file` の結果を、実際の作成の代わりに返す(`None` は実際に作る。U4)。
    pub create_override: RefCell<Option<Result<CreateOutcome, CreateError>>>,
}

/// `create_file` に渡された、(パス、内容、`force`)。
pub type CreateCall = (PathBuf, Vec<u8>, bool);

/// 偽の `run_job` が使う、足す環境の `CONDUCTOR_JOB_ID`(ジョブ ID)。
///
/// 本物の `run_job` はジョブ ID を受け取らないので、偽物は、core の `merge_env` が自動の変数の層で
/// いつも足す `CONDUCTOR_JOB_ID` から取る(利用者は予約の接頭辞 `CONDUCTOR_` を書けないので、ほかの値に
/// 上書きされない)。無いときは、契約が崩れたので、パニックで止める(D1)。
pub fn job_id_of(env: &[(&str, &str)]) -> String {
    env.iter()
        .find(|(name, _)| *name == "CONDUCTOR_JOB_ID")
        .map(|(_, value)| value.to_string())
        .expect("CONDUCTOR_JOB_ID がありません(core の merge_env がいつも足す)")
}

/// 偽の `wait` の共通の処理(`ScriptedRuntime` と `MemRuntime`)。待たずに、待ち時間を `waits` に記録する。
///
/// 合図がすでに立っていれば `Interrupted`。`interrupt_wait` が `(N, 番号)` で、これが N 回目(0 始まり)の
/// 呼び出しなら、合図を立てて `Interrupted`。どちらでもなければ `Elapsed`。
pub fn fake_wait(
    waits: &RefCell<Vec<u64>>,
    interrupt_wait: &Cell<Option<(usize, i32)>>,
    secs: u64,
    signal: &InterruptSignal,
) -> WaitOutcome {
    let n = waits.borrow().len();
    waits.borrow_mut().push(secs);
    if let Some(number) = signal.signal_number() {
        return WaitOutcome::Interrupted(number);
    }
    match interrupt_wait.get() {
        Some((at, number)) if at == n => {
            signal.raise(number);
            WaitOutcome::Interrupted(number)
        }
        _ => WaitOutcome::Elapsed,
    }
}

pub fn state_error(path: &Path, kind: StateErrorKind) -> StateError {
    StateError {
        path: path.to_path_buf(),
        kind,
        // 英語の表示を ASCII と確かめる単体テストの `src/tests/report_messages.rs` も同じ写しを使うので、
        // ASCII の文にする。
        reason: "reason",
    }
}

impl ScriptedRuntime {
    pub fn script(&self, job_id: &str, results: &[Exec]) {
        self.scripts
            .borrow_mut()
            .insert(job_id.to_string(), results.iter().copied().collect());
    }

    pub fn log(&self) -> Vec<String> {
        self.log.borrow().clone()
    }

    pub fn ran(&self) -> Vec<String> {
        self.log()
            .into_iter()
            .filter_map(|l| l.strip_prefix("run:").map(str::to_string))
            .collect()
    }

    fn push(&self, line: impl Into<String>) {
        self.log.borrow_mut().push(line.into());
    }
}

impl Runtime for ScriptedRuntime {
    fn load_definition(
        &self,
        path: &Path,
        expected: Option<&str>,
    ) -> Result<PipelineDefinition, Vec<DefinitionError>> {
        self.push("load");
        self.inner.load_definition(path, expected)
    }

    fn check_command(&self, command: &Command) -> Result<CommandLine, CommandError> {
        self.inner.check_command(command)
    }

    fn load_env_files(
        &self,
        definition_path: &Path,
        refs: &[String],
    ) -> Result<LoadedEnvFiles, Vec<EnvFileError>> {
        self.push("load-env");
        self.env_loads.borrow_mut().push(refs.to_vec());
        if let Some(result) = self.env_override.borrow().as_ref() {
            return result.clone();
        }
        self.inner.load_env_files(definition_path, refs)
    }

    fn run_job(
        &self,
        line: &CommandLine,
        env: &[(&str, &str)],
        signal: &InterruptSignal,
    ) -> JobExecution {
        let job_id = job_id_of(env);
        self.run_calls.set(self.run_calls.get() + 1);
        self.run_envs.borrow_mut().push((
            job_id.clone(),
            env.iter()
                .map(|(n, v)| (n.to_string(), v.to_string()))
                .collect(),
        ));
        self.run_lines
            .borrow_mut()
            .push((job_id.clone(), line.clone()));
        if let Some(number) = signal.signal_number() {
            self.push(format!("skip-run:{job_id}"));
            return JobExecution::not_started(number);
        }
        self.push(format!("run:{job_id}"));
        if let Some(hook) = self.during_job.borrow().as_ref() {
            hook(&job_id, signal);
        }
        let spec = self
            .scripts
            .borrow_mut()
            .get_mut(&job_id)
            .and_then(VecDeque::pop_front)
            .unwrap_or(Exec::Exit(0));
        // 結果は台本で決める。実行中の合図は、本物と同じく `Exited` のときだけ足す。
        let interrupted_by = signal.signal_number();
        let result = match spec {
            Exec::Exit(code) => ExecutionResult::Exited {
                exit_code: Some(code),
                terminated_by_signal: None,
                interrupted_by,
            },
            Exec::KilledBy(n) => ExecutionResult::Exited {
                exit_code: None,
                terminated_by_signal: Some(n),
                interrupted_by,
            },
            Exec::StartFailure => ExecutionResult::StartFailed,
        };
        let execution = JobExecution::new(result);
        if let Some(hook) = self.after_job.borrow().as_ref() {
            hook(&job_id, signal);
        }
        execution
    }

    fn wait(&self, secs: u64, signal: &InterruptSignal) -> WaitOutcome {
        self.push(format!("wait:{secs}"));
        fake_wait(&self.waits, &self.interrupt_wait, secs, signal)
    }

    fn read_state(
        &self,
        path: &Path,
        run_id: Option<&RunId>,
    ) -> Result<Option<PipelineState>, StateError> {
        self.push("read");
        if let Some(error) = self.read_failure.borrow().clone() {
            return Err(error);
        }
        self.inner.read_state(path, run_id)
    }

    fn read_state_raw(
        &self,
        path: &Path,
        run_id: Option<&RunId>,
    ) -> Result<Option<PipelineState>, StateError> {
        self.push("read-raw");
        if let Some(error) = self.read_failure.borrow().clone() {
            return Err(error);
        }
        self.inner.read_state_raw(path, run_id)
    }

    fn write_state(
        &self,
        path: &Path,
        run_id: Option<&RunId>,
        state: &PipelineState,
        durability: Durability,
    ) -> Result<(), StateError> {
        let n = self.write_calls.get();
        self.write_calls.set(n + 1);
        let label = match durability {
            Durability::Fast => "write:fast",
            Durability::Durable => "write:durable",
        };
        self.push(label);
        let fail = match self.write_failure.get() {
            WriteFailure::Never => false,
            WriteFailure::OnCall(k) => k == n,
            WriteFailure::OnDurability(d) => d == durability,
        };
        if fail {
            return Err(state_error(path, StateErrorKind::WriteFailed));
        }
        self.inner.write_state(path, run_id, state, durability)
    }

    fn reset_state(&self, path: &Path, run_id: Option<&RunId>) -> Result<(), StateError> {
        self.push("reset");
        if self.fail_reset.get() {
            return Err(state_error(path, StateErrorKind::RemoveFailed));
        }
        self.inner.reset_state(path, run_id)
    }

    fn reset_all_states(&self, path: &Path) -> Result<ResetAllReport, StateError> {
        self.push("reset-all");
        if self.fail_reset.get() {
            return Err(state_error(path, StateErrorKind::RemoveFailed));
        }
        match self.reset_all_override.borrow().clone() {
            Some(report) => Ok(report),
            None => self.inner.reset_all_states(path),
        }
    }

    fn run_id_env(&self, path: &Path) -> Option<String> {
        // ログには残さない(`reset` の呼び出しの記録を、変えないため)。
        self.run_id_env_calls.set(self.run_id_env_calls.get() + 1);
        self.inner.run_id_env(path)
    }

    fn convert(&self, input: &Path, target: TargetFormat) -> Result<String, ConversionError> {
        self.push("convert");
        match self.convert_result.borrow().clone() {
            Some(result) => result,
            None => self.inner.convert(input, target),
        }
    }

    fn create_file(
        &self,
        path: &Path,
        bytes: &[u8],
        force: bool,
    ) -> Result<CreateOutcome, CreateError> {
        self.push("create");
        self.create_calls
            .borrow_mut()
            .push((path.to_path_buf(), bytes.to_vec(), force));
        match *self.create_override.borrow() {
            Some(result) => result,
            None => self.inner.create_file(path, bytes, force),
        }
    }
}

// ---- 定義の組み立て ----

/// 3 ジョブ(a → b → c)の定義。
pub const THREE_JOBS: &str = r#"format_version = 1

[pipeline]
name = "p"

[[job]]
id = "a"
command = "echo a"

[[job]]
id = "b"
command = "echo b"

[[job]]
id = "c"
command = "echo c"
"#;

/// a の警告で c へ飛ぶ(b をスキップする)定義。
pub const WARNING_JUMPS: &str = r#"format_version = 1

[pipeline]
name = "p"

[[job]]
id = "a"
command = "echo a"
warning_exit_codes = [1]

[[job.transitions]]
on = "warning"
next = "c"

[[job]]
id = "b"
command = "echo b"

[[job]]
id = "c"
command = "echo c"
"#;

/// a の成功で `@end`(b、c は実行しない)の定義。
pub const END_AFTER_A: &str = r#"format_version = 1

[pipeline]
name = "p"

[[job]]
id = "a"
command = "echo a"

[[job.transitions]]
on = "success"
next = "@end"

[[job]]
id = "b"
command = "echo b"

[[job]]
id = "c"
command = "echo c"
"#;

/// 定義の `[pipeline]` に、`remove_on_complete = <値>` を足した文字列にする。
pub fn with_remove_on_complete(definition: &str, value: &str) -> String {
    definition.replacen(
        "name = \"p\"\n",
        &format!("name = \"p\"\nremove_on_complete = {value}\n"),
        1,
    )
}

// ---- 実行の枠 ----

pub struct Outcome {
    pub code: i32,
    pub out: String,
    pub err: String,
}

pub struct Harness {
    pub dir: tempfile::TempDir,
    pub runtime: ScriptedRuntime,
    pub env: FakeEnv,
    pub signals: FakeSignals,
    pub sink: FakeSink,
}

impl Harness {
    pub fn new(definition: &str) -> Self {
        let dir = tempfile::tempdir().expect("一時ディレクトリ");
        std::fs::write(dir.path().join("conductor.toml"), definition).expect("書ける");
        Self {
            dir,
            runtime: ScriptedRuntime::default(),
            env: FakeEnv::default(),
            signals: FakeSignals::new(SignalMode::Noop),
            sink: FakeSink::default(),
        }
    }

    pub fn def_path(&self) -> PathBuf {
        self.dir.path().join("conductor.toml")
    }

    pub fn state_path(&self) -> PathBuf {
        self.dir.path().join("conductor.toml.state")
    }

    /// 定義を書き換える。
    pub fn set_definition(&self, text: &str) {
        std::fs::write(self.def_path(), text).expect("書ける");
    }

    /// run ID つきのステートのファイルのパス(`conductor.toml.<run-id>.state`)。
    pub fn state_path_for(&self, run_id: &str) -> PathBuf {
        self.dir
            .path()
            .join(format!("conductor.toml.{run_id}.state"))
    }

    /// 読み込めるステート(空・完了は `None`)。
    pub fn state(&self) -> Option<PipelineState> {
        conductor_runtime::read_state(&self.def_path(), None).expect("読める")
    }

    /// run ID つきの、読み込めるステート(空・完了は `None`)。
    pub fn state_of(&self, run_id: &str) -> Option<PipelineState> {
        let id = RunId::parse(run_id).expect("有効な run ID");
        conductor_runtime::read_state(&self.def_path(), Some(&id)).expect("読める")
    }

    /// 完了のステートも含めて読む(ファイルなし・長さ 0 は `None`)。
    pub fn state_raw(&self) -> Option<PipelineState> {
        conductor_runtime::read_state_raw(&self.def_path(), None).expect("読める")
    }

    /// run ID つきの、完了のステートも含めて読む。
    pub fn state_raw_of(&self, run_id: &str) -> Option<PipelineState> {
        let id = RunId::parse(run_id).expect("有効な run ID");
        conductor_runtime::read_state_raw(&self.def_path(), Some(&id)).expect("読める")
    }

    /// `--config <定義>` を足して、実行する。
    pub fn run(&mut self, args: &[&str]) -> Outcome {
        let mut all: Vec<OsString> = args.iter().map(OsString::from).collect();
        all.push("--config".into());
        all.push(self.def_path().into_os_string());
        self.run_os(all)
    }

    /// そのままの引数で、実行する。
    pub fn run_raw(&mut self, args: &[&str]) -> Outcome {
        self.run_os(args.iter().map(OsString::from).collect())
    }

    pub fn run_os(&mut self, args: Vec<OsString>) -> Outcome {
        let code = run_with_fakes(
            &self.runtime,
            &mut self.sink,
            &self.env,
            &self.signals,
            args,
        );
        let outcome = Outcome {
            code,
            out: self.sink.out_text(),
            err: self.sink.err_text(),
        };
        self.sink.out.clear();
        self.sink.err.clear();
        outcome
    }
}

// ---- 入出力に触れない `Runtime`(横断のテストが使う) ----

/// ファイルにも、プロセスにも触れない `Runtime`。定義は固定で、ステートはメモリに持つ。
pub struct MemRuntime {
    pub definition: PipelineDefinition,
    pub state: RefCell<Option<PipelineState>>,
    /// run ID ごとのステート(`state` は、run ID なしの分)。
    pub run_states: RefCell<HashMap<String, PipelineState>>,
    /// ジョブの終了コード(先頭から使い、尽きたら 0)。
    pub exit_codes: RefCell<VecDeque<i32>>,
    pub write_ok: Cell<bool>,
    /// `wait` が呼ばれた待ち時間(秒)。待たずに記録するだけ。
    pub waits: RefCell<Vec<u64>>,
    /// `wait` の N 回目(0 始まり)で、合図(番号)を立てて `Interrupted` を返す。
    pub interrupt_wait: Cell<Option<(usize, i32)>>,
}

impl MemRuntime {
    pub fn new(definition_text: &str) -> Self {
        let definition = conductor_core::parse_definition(definition_text, "mem.toml", None)
            .expect("正しい定義");
        Self {
            definition,
            state: RefCell::new(None),
            run_states: RefCell::new(HashMap::new()),
            exit_codes: RefCell::new(VecDeque::new()),
            write_ok: Cell::new(true),
            waits: RefCell::new(Vec::new()),
            interrupt_wait: Cell::new(None),
        }
    }
}

impl Runtime for MemRuntime {
    fn load_definition(
        &self,
        _path: &Path,
        expected: Option<&str>,
    ) -> Result<PipelineDefinition, Vec<DefinitionError>> {
        match expected {
            Some(name) if name != self.definition.name => Err(vec![DefinitionError::new(
                "mem.toml",
                None,
                conductor_core::DefinitionErrorKind::PipelineNameMismatch,
                "名前が違います",
            )]),
            _ => Ok(self.definition.clone()),
        }
    }

    fn check_command(&self, command: &Command) -> Result<CommandLine, CommandError> {
        conductor_runtime::check_command(command)
    }

    fn load_env_files(
        &self,
        _definition_path: &Path,
        _refs: &[String],
    ) -> Result<LoadedEnvFiles, Vec<EnvFileError>> {
        Ok(LoadedEnvFiles::new())
    }

    fn run_job(
        &self,
        _line: &CommandLine,
        _env: &[(&str, &str)],
        signal: &InterruptSignal,
    ) -> JobExecution {
        // 合図を先に見て、起動するときだけ台本を消費する(`ScriptedRuntime` と同じ順)。
        match signal.signal_number() {
            Some(n) => JobExecution::not_started(n),
            None => JobExecution::exited(self.exit_codes.borrow_mut().pop_front().unwrap_or(0)),
        }
    }

    fn wait(&self, secs: u64, signal: &InterruptSignal) -> WaitOutcome {
        fake_wait(&self.waits, &self.interrupt_wait, secs, signal)
    }

    fn read_state(
        &self,
        path: &Path,
        run_id: Option<&RunId>,
    ) -> Result<Option<PipelineState>, StateError> {
        // 本物と同じく、完了のステートは空として返す。
        Ok(self
            .read_state_raw(path, run_id)?
            .filter(|state| !state.completed))
    }

    fn read_state_raw(
        &self,
        _path: &Path,
        run_id: Option<&RunId>,
    ) -> Result<Option<PipelineState>, StateError> {
        match run_id {
            None => Ok(self.state.borrow().clone()),
            Some(id) => Ok(self.run_states.borrow().get(id.as_str()).cloned()),
        }
    }

    fn write_state(
        &self,
        path: &Path,
        run_id: Option<&RunId>,
        state: &PipelineState,
        _durability: Durability,
    ) -> Result<(), StateError> {
        if !self.write_ok.get() {
            return Err(state_error(path, StateErrorKind::WriteFailed));
        }
        match run_id {
            None => *self.state.borrow_mut() = Some(state.clone()),
            Some(id) => {
                self.run_states
                    .borrow_mut()
                    .insert(id.as_str().to_string(), state.clone());
            }
        }
        Ok(())
    }

    fn reset_state(&self, _path: &Path, run_id: Option<&RunId>) -> Result<(), StateError> {
        match run_id {
            None => *self.state.borrow_mut() = None,
            Some(id) => {
                self.run_states.borrow_mut().remove(id.as_str());
            }
        }
        Ok(())
    }

    fn reset_all_states(&self, _path: &Path) -> Result<ResetAllReport, StateError> {
        *self.state.borrow_mut() = None;
        self.run_states.borrow_mut().clear();
        Ok(ResetAllReport { kept: Vec::new() })
    }

    fn run_id_env(&self, _path: &Path) -> Option<String> {
        self.definition.run_id_env.clone()
    }

    fn convert(&self, _input: &Path, _target: TargetFormat) -> Result<String, ConversionError> {
        Ok("converted\n".to_string())
    }

    fn create_file(
        &self,
        _path: &Path,
        _bytes: &[u8],
        _force: bool,
    ) -> Result<CreateOutcome, CreateError> {
        Ok(CreateOutcome::Created)
    }
}

/// 差し替え口を、メモリの `Runtime` で組んで実行する。
pub fn run_mem(runtime: &MemRuntime, args: &[&str]) -> (i32, FakeSink) {
    let mut sink = FakeSink::default();
    let env = FakeEnv::default();
    let signals = FakeSignals::new(SignalMode::Noop);
    let code = run_with_fakes(
        runtime,
        &mut sink,
        &env,
        &signals,
        args.iter().map(OsString::from).collect(),
    );
    (code, sink)
}

/// 偽の差し替え口(`Runtime`、出力、環境、シグナル)で `Deps` を組み、`run_cli_with` を呼ぶ。
pub fn run_with_fakes(
    runtime: &dyn Runtime,
    sink: &mut FakeSink,
    env: &FakeEnv,
    signals: &FakeSignals,
    args: Vec<OsString>,
) -> i32 {
    let mut deps = Deps {
        runtime,
        sink,
        env,
        signals,
    };
    run_cli_with(args, &mut deps)
}

/// ステートの中の、ジョブ `id` の状態(ジョブが無ければ失敗する)。
pub fn status_in(state: &PipelineState, id: &str) -> JobStatus {
    state
        .job_states
        .iter()
        .find(|j| j.job_id == id)
        .expect("ジョブがある")
        .status
}

/// 偽の実行環境が書いたステートの中の、ジョブ `id` の状態(ステートかジョブが無ければ失敗する)。
pub fn status_of(h: &Harness, id: &str) -> JobStatus {
    status_in(&h.state().expect("ステートがある"), id)
}
