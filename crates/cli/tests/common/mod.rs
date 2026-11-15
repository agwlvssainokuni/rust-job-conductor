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
    DefinitionError, JobDefinition, JobExecution, PipelineDefinition, PipelineState,
};
use conductor_runtime::{
    CommandError, CommandLine, Durability, InterruptSignal, StateError, StateErrorKind,
};

// ---- Env ----

#[derive(Default)]
pub struct FakeEnv {
    pub vars: HashMap<String, String>,
    pub terminal: bool,
    pub var_calls: Cell<usize>,
    pub terminal_calls: Cell<usize>,
}

impl FakeEnv {
    pub fn with(vars: &[(&str, &str)]) -> Self {
        Self {
            vars: vars
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Self::default()
        }
    }
}

impl Env for FakeEnv {
    fn var(&self, name: &str) -> Option<String> {
        self.var_calls.set(self.var_calls.get() + 1);
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
    Already,
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
            SignalMode::Already => Err(SignalError::AlreadyRegistered),
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

#[derive(Clone, Copy, Debug)]
pub enum WriteFailure {
    Never,
    /// 書き込みの N 回目(0 始まり)が失敗する。
    OnCall(usize),
    /// その耐久性の書き込みは、すべて失敗する。
    OnDurability(Durability),
}

/// ジョブの前後で呼ぶ関数(引数は、ジョブIDと、中断の合図)。
pub type JobHook = Box<dyn Fn(&str, &InterruptSignal)>;

/// 定義・ステートの入出力は実際の関数(`SystemRuntime`)に任せ、ジョブの起動だけを、台本で偽にする。
/// 失敗は、注入できる。呼び出しの順序を `log` に残す。
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
}

impl Default for ScriptedRuntime {
    fn default() -> Self {
        Self {
            inner: SystemRuntime,
            log: RefCell::new(Vec::new()),
            scripts: RefCell::new(HashMap::new()),
            write_failure: Cell::new(WriteFailure::Never),
            write_calls: Cell::new(0),
            fail_reset: Cell::new(false),
            read_failure: RefCell::new(None),
            convert_result: RefCell::new(None),
            during_job: RefCell::new(None),
            after_job: RefCell::new(None),
            run_calls: Cell::new(0),
        }
    }
}

pub fn state_error(path: &Path, kind: StateErrorKind) -> StateError {
    StateError {
        path: path.to_path_buf(),
        kind,
        reason: "テスト用の固定の理由",
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

    fn check_command(&self, command: &str) -> Result<CommandLine, CommandError> {
        self.inner.check_command(command)
    }

    fn run_job(&self, job: &JobDefinition, signal: &InterruptSignal) -> JobExecution {
        self.run_calls.set(self.run_calls.get() + 1);
        let mut execution = JobExecution {
            job_id: job.id.clone(),
            command: job.command.clone(),
            started: true,
            exit_code: None,
            terminated_by_signal: None,
            start_failure: false,
            interrupted: false,
            signal_number: None,
        };
        if let Some(number) = signal.signal_number() {
            execution.started = false;
            execution.interrupted = true;
            execution.signal_number = Some(number);
            self.push(format!("skip-run:{}", job.id));
            return execution;
        }
        self.push(format!("run:{}", job.id));
        if let Some(hook) = self.during_job.borrow().as_ref() {
            hook(&job.id, signal);
        }
        let spec = self
            .scripts
            .borrow_mut()
            .get_mut(&job.id)
            .and_then(VecDeque::pop_front)
            .unwrap_or(Exec::Exit(0));
        match spec {
            Exec::Exit(code) => execution.exit_code = Some(code),
            Exec::KilledBy(n) => execution.terminated_by_signal = Some(n),
            Exec::StartFailure => execution.start_failure = true,
        }
        if let Some(number) = signal.signal_number() {
            execution.interrupted = true;
            execution.signal_number = Some(number);
        }
        if let Some(hook) = self.after_job.borrow().as_ref() {
            hook(&job.id, signal);
        }
        execution
    }

    fn read_state(&self, path: &Path) -> Result<Option<PipelineState>, StateError> {
        self.push("read");
        if let Some(error) = self.read_failure.borrow().clone() {
            return Err(error);
        }
        self.inner.read_state(path)
    }

    fn write_state(
        &self,
        path: &Path,
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
        self.inner.write_state(path, state, durability)
    }

    fn reset_state(&self, path: &Path) -> Result<(), StateError> {
        self.push("reset");
        if self.fail_reset.get() {
            return Err(state_error(path, StateErrorKind::RemoveFailed));
        }
        self.inner.reset_state(path)
    }

    fn convert(&self, input: &Path, target: TargetFormat) -> Result<String, ConversionError> {
        self.push("convert");
        match self.convert_result.borrow().clone() {
            Some(result) => result,
            None => self.inner.convert(input, target),
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

    /// 読み込めるステート(空・完了は `None`)。
    pub fn state(&self) -> Option<PipelineState> {
        conductor_runtime::read_state(&self.def_path()).expect("読める")
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
        let code = {
            let mut deps = Deps {
                runtime: &self.runtime,
                sink: &mut self.sink,
                env: &self.env,
                signals: &self.signals,
            };
            run_cli_with(args, &mut deps)
        };
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
    /// ジョブの終了コード(先頭から使い、尽きたら 0)。
    pub exit_codes: RefCell<VecDeque<i32>>,
    pub write_ok: Cell<bool>,
    pub writes: Cell<usize>,
}

impl MemRuntime {
    pub fn new(definition_text: &str) -> Self {
        let definition = conductor_core::parse_definition(definition_text, "mem.toml", None)
            .expect("正しい定義");
        Self {
            definition,
            state: RefCell::new(None),
            exit_codes: RefCell::new(VecDeque::new()),
            write_ok: Cell::new(true),
            writes: Cell::new(0),
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

    fn check_command(&self, command: &str) -> Result<CommandLine, CommandError> {
        conductor_runtime::check_command(command)
    }

    fn run_job(&self, job: &JobDefinition, signal: &InterruptSignal) -> JobExecution {
        let mut execution = JobExecution {
            job_id: job.id.clone(),
            command: job.command.clone(),
            started: true,
            exit_code: Some(self.exit_codes.borrow_mut().pop_front().unwrap_or(0)),
            terminated_by_signal: None,
            start_failure: false,
            interrupted: false,
            signal_number: None,
        };
        if let Some(n) = signal.signal_number() {
            execution.started = false;
            execution.exit_code = None;
            execution.interrupted = true;
            execution.signal_number = Some(n);
        }
        execution
    }

    fn read_state(&self, _path: &Path) -> Result<Option<PipelineState>, StateError> {
        Ok(self.state.borrow().clone())
    }

    fn write_state(
        &self,
        path: &Path,
        state: &PipelineState,
        _durability: Durability,
    ) -> Result<(), StateError> {
        self.writes.set(self.writes.get() + 1);
        if !self.write_ok.get() {
            return Err(state_error(path, StateErrorKind::WriteFailed));
        }
        *self.state.borrow_mut() = Some(state.clone());
        Ok(())
    }

    fn reset_state(&self, _path: &Path) -> Result<(), StateError> {
        *self.state.borrow_mut() = None;
        Ok(())
    }

    fn convert(&self, _input: &Path, _target: TargetFormat) -> Result<String, ConversionError> {
        Ok("converted\n".to_string())
    }
}

/// 差し替え口を、メモリの `Runtime` で組んで実行する。
pub fn run_mem(runtime: &MemRuntime, args: &[&str]) -> (i32, FakeSink) {
    let mut sink = FakeSink::default();
    let env = FakeEnv::default();
    let signals = FakeSignals::new(SignalMode::Noop);
    let code = {
        let mut deps = Deps {
            runtime,
            sink: &mut sink,
            env: &env,
            signals: &signals,
        };
        run_cli_with(args.iter().map(OsString::from), &mut deps)
    };
    (code, sink)
}
