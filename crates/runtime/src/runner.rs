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

//! ジョブの起動と待ち(JobRunner。Spawn と WaitLoop)。BR1.2〜BR1.7、NFR4.1、NFR4.8。
//!
//! 待ちは、1 つのスレッドの、短い間隔(1ms から 10ms)の `try_wait` のくり返しで行う。
//! ジョブを回収する(`try_wait` が終了を返す)のは、このくり返しの中だけであり、シグナルを送るのは、
//! 直前の `try_wait` が「まだ終わっていない」を返した回に限る。
//!
//! ログの指定(`log_dir`。v0.10.0 の BR4.1〜BR4.5)があるときだけ、子の標準出力と標準エラーを
//! パイプで受け、それぞれ 1 つの読み取りのスレッド(Copier)が、読んだ分を端末(conductor の同じ流れ)と
//! ログのファイルの両方へ書く。待ちのくり返しとシグナルの転送は、ログの有無によらず同じ。

use std::fs::File;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use conductor_core::{ExecutionResult, JobExecution};

use crate::log::{LogOutcome, LogTarget, open_log_file};
use crate::platform;
use crate::types::{CommandLine, InterruptSignal};

/// 待ちの間隔の最初と最大(ミリ秒)。
const FIRST_INTERVAL_MS: u64 = 1;
const MAX_INTERVAL_MS: u64 = 10;

/// 読み取りの 1 回の枠(NFR1.3。出力の全体を溜めない)。
const COPY_CHUNK_BYTES: usize = 8 * 1024;

/// 子の終了の後、読み取りのスレッドの終わりを待つ時間の上限(合わせて。BR4.5)。
const DRAIN_LIMIT: Duration = Duration::from_secs(2);

/// `try_wait` と pid だけを持つ、子プロセスの見え方(待ちのくり返しを、プロセスなしで試すため)。
trait Waitable {
    fn id(&self) -> u32;
    fn try_wait(&mut self) -> io::Result<Option<ExitStatus>>;
}

impl Waitable for Child {
    fn id(&self) -> u32 {
        Child::id(self)
    }
    fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        Child::try_wait(self)
    }
}

/// 待ちのくり返しの結果。
enum Waited {
    /// 終了を観測した(回収した)。
    Exited(ExitStatus),
    /// 終了の状態を取得できなかった。
    WaitFailed,
}

/// 待ちのくり返し(WaitLoop)。
///
/// 直前の `try_wait` が「まだ」を返した回にだけ、合図を見て、立っていれば `forward` を 1 回だけ
/// 呼ぶ(`forward(pid, シグナル番号)`)。`try_wait` の失敗は、`WaitFailed`。
fn wait_loop<C: Waitable>(
    child: &mut C,
    signal: &InterruptSignal,
    forward: &mut dyn FnMut(u32, i32),
    sleep: &mut dyn FnMut(Duration),
) -> Waited {
    let mut forwarded = false;
    let mut interval = FIRST_INTERVAL_MS;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Waited::Exited(status),
            Ok(None) => {}
            Err(_) => return Waited::WaitFailed,
        }
        if !forwarded && let Some(number) = signal.signal_number() {
            forward(child.id(), number);
            forwarded = true;
        }
        sleep(Duration::from_millis(interval));
        interval = (interval * 2).min(MAX_INTERVAL_MS);
    }
}

/// ジョブを、シェルを介さずに起動し、終了を待つ(契約 C3 の `run_job`。W1)。
///
/// - 分割済みのコマンド `line`(Cli が、実行の前の検査で `check_command` で作ったもの)で起動する。
///   ここでは command を分け直さない(BR2.1。v0.9.5 までは `run_job(job, env, signal)` で、中で分け直した)。
///   ジョブの定義は受け取らない(以前の `run_job(job, line, env, signal)` の `job` は使われなかった)。
/// - `line.program` が空または空白だけなら、起動せず `StartFailed`(防御。BR2.3)。この確認は、
///   起動前の合図の確認より先に行う。
/// - 合図が、起動の前に立っていれば、起動せず、`NotStarted`(BR1.6)。
/// - 起動する(UNIX 系は新しいプロセスグループ)。作業ディレクトリ、環境変数、標準入出力は引き継ぐ。
///   `env` の名前と値を、並びの順に足す(引き継いだ環境は残し、同じ名前は上書きする。同じ名前が
///   並びに 2 回あれば、後ろが勝つ。U1 の BR4.3、BR4.4、FR1.3)。
///   起動できなければ `StartFailed`(BR1.2〜BR1.4)。
/// - 終了まで待つ。時間切れにしない。再試行しない(BR1.5)。実行中に合図が立てば、シグナルを
///   ジョブに 1 回だけ転送する(BR1.7)。
/// - 終了を観測したら `Exited`。その直後に合図が立っていれば、`interrupted_by` にその番号(BR1.7)。
///
/// ログの指定 `log`(v0.10.0 の `log_dir`。BR1.2、BR4.1〜BR4.5、BR5.1〜BR5.3):
///
/// - `None` なら、今と同じ経路で起動する(標準出力と標準エラーは引き継ぐ。パイプもスレッドも作らない)。
///   [`LogOutcome`] はすべて空。
/// - `Some` なら、合図の確認の後、起動の前に [`open_log_file`] でファイルを作る。作れなければ起動せず
///   `StartFailed` と `log_not_created`(BR5.1)。作れたら、標準出力と標準エラーをパイプにして起動し
///   (標準入力は引き継ぐ。BR4.3)、読んだ分を端末とファイルへ書く(BR4.1、BR4.2)。ファイルへの
///   書き込みに失敗したら、以後ファイルには書かず `write_failed`(BR5.2)。子の終了の後(中断のときも)、
///   読み取りの終わりを最大 2 秒待ち、待ち切れたら以後どこにも書かず `output_cut_short`(BR4.5)。
///
/// 失敗は、すべて戻り値(`JobExecution` と [`LogOutcome`])で返し、パニックしない(NFR4.7)。
pub fn run_job(
    line: &CommandLine,
    env: &[(&str, &str)],
    signal: &InterruptSignal,
    log: Option<&LogTarget>,
) -> (JobExecution, LogOutcome) {
    if line.program.trim().is_empty() {
        return (JobExecution::start_failed(), LogOutcome::default());
    }

    if let Some(number) = signal.signal_number() {
        return (JobExecution::not_started(number), LogOutcome::default());
    }

    let mut command = Command::new(&line.program);
    command.args(&line.args);
    for (name, value) in env {
        command.env(name, value);
    }
    platform::configure_spawn(&mut command);

    let Some(target) = log else {
        return (spawn_and_wait(&mut command, signal), LogOutcome::default());
    };
    let (file, path) = match open_log_file(target) {
        Ok(opened) => opened,
        Err(path) => {
            let outcome = LogOutcome {
                log_not_created: Some(path),
                ..LogOutcome::default()
            };
            return (JobExecution::start_failed(), outcome);
        }
    };
    run_with_log(&mut command, signal, file, path)
}

/// 起動して終了を待つ(標準出力と標準エラーは、`command` の指定のまま)。
fn spawn_and_wait(command: &mut Command, signal: &InterruptSignal) -> JobExecution {
    match command.spawn() {
        Ok(mut child) => wait_child(&mut child, signal),
        Err(_) => JobExecution::start_failed(),
    }
}

/// 子の終了を待ち、結果にする(シグナルの転送を含む。BR1.7)。
fn wait_child(child: &mut Child, signal: &InterruptSignal) -> JobExecution {
    let waited = wait_loop(
        child,
        signal,
        &mut platform::forward_signal,
        &mut std::thread::sleep,
    );
    match waited {
        Waited::Exited(status) => JobExecution::new(ExecutionResult::Exited {
            exit_code: status.code(),
            terminated_by_signal: platform::termination_signal(&status),
            interrupted_by: signal.signal_number(),
        }),
        // 終了の状態を取得できない。起動失敗として返し、conductor-core がエラー(127)として扱う(安全側)。
        Waited::WaitFailed => JobExecution::start_failed(),
    }
}

/// ログのファイル `file`(パス `path`)を用意できたときの起動と待ち(BR4.1〜BR4.5)。
fn run_with_log(
    command: &mut Command,
    signal: &InterruptSignal,
    file: File,
    path: PathBuf,
) -> (JobExecution, LogOutcome) {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(_) => return (JobExecution::start_failed(), LogOutcome::default()),
    };
    let shared = Arc::new(Shared::new(file));
    let (done_tx, done_rx) = mpsc::channel::<()>();
    let mut spawn_failed = false;
    if let Some(stdout) = child.stdout.take() {
        spawn_failed |= !start_copier(stdout, io::stdout(), &shared, &done_tx);
    }
    if let Some(stderr) = child.stderr.take() {
        spawn_failed |= !start_copier(stderr, io::stderr(), &shared, &done_tx);
    }
    // 送り手は読み取りのスレッドだけが持つ(全員が終われば、受け手は切断を知る)。
    drop(done_tx);
    if spawn_failed {
        // スレッドを作れず、その流れの出力をファイルに残せない(パイプは閉じた)。
        shared.mark_file_failed();
    }

    let execution = wait_child(&mut child, signal);
    let cut_short = !wait_copiers(&done_rx, DRAIN_LIMIT);
    if cut_short {
        shared.stop.store(true, Ordering::SeqCst);
    }
    let outcome = LogOutcome {
        log_not_created: None,
        write_failed: shared
            .file_failed
            .load(Ordering::SeqCst)
            .then(|| path.clone()),
        output_cut_short: cut_short.then_some(path),
    };
    (execution, outcome)
}

/// 読み取りのスレッドと共有する状態。
struct Shared<F> {
    /// 待ち切れたあとに立てる。立っていれば、読んだ分をどこにも書かない(BR4.5)。
    stop: AtomicBool,
    /// ファイルへの書き込みに一度でも失敗したか(BR5.2)。
    file_failed: AtomicBool,
    /// ログのファイル。2 つの流れは、この鍵で順に書く。書き込みに失敗したら `None` にする。
    file: Mutex<Option<F>>,
}

impl<F: Write> Shared<F> {
    fn new(file: F) -> Self {
        Self {
            stop: AtomicBool::new(false),
            file_failed: AtomicBool::new(false),
            file: Mutex::new(Some(file)),
        }
    }

    fn lock_file(&self) -> MutexGuard<'_, Option<F>> {
        // 鍵を持ったスレッドがパニックしても、ファイルへの書き込みは続けられる(中身は壊れていない)。
        self.file
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// 読んだ分をファイルへ書く。失敗したら印を立て、以後ファイルには書かない(BR5.2)。
    fn write_file(&self, chunk: &[u8]) {
        let mut guard = self.lock_file();
        if let Some(file) = guard.as_mut()
            && file.write_all(chunk).is_err()
        {
            *guard = None;
            self.file_failed.store(true, Ordering::SeqCst);
        }
    }

    fn mark_file_failed(&self) {
        *self.lock_file() = None;
        self.file_failed.store(true, Ordering::SeqCst);
    }
}

/// 1 つの流れの読み取りのスレッドを始める。スレッドを作れなければ `false`(パイプは閉じる)。
fn start_copier<R, T>(reader: R, terminal: T, shared: &Arc<Shared<File>>, done: &Sender<()>) -> bool
where
    R: Read + Send + 'static,
    T: Write + Send + 'static,
{
    let shared = Arc::clone(shared);
    let done = done.clone();
    std::thread::Builder::new()
        .name("conductor-log-copier".into())
        .spawn(move || {
            let mut terminal = terminal;
            copy_stream(reader, &mut terminal, &shared);
            // 終わりの知らせ(送り手を手放す。受け手は、全員が手放したことで終わりを知る)。
            drop(done);
        })
        .is_ok()
}

/// 1 つの流れを、終わりまで 8 KiB ずつ読み、端末とファイルへ書く(BR4.1、BR4.2、`reliability-design.md` の 2)。
///
/// - 読んだバイト列は、そのまま書く(conductor は何も書き足さない)。端末へは書くたびに flush する。
/// - 端末への書き込みに失敗したら、端末へ書くのをやめるが、読み続けてファイルへは書く。
/// - 「止める」印が立っていたら、読んだ分をどこにも書かずに終わる。
fn copy_stream<R: Read, T: Write, F: Write>(mut reader: R, terminal: &mut T, shared: &Shared<F>) {
    let mut buf = [0u8; COPY_CHUNK_BYTES];
    let mut terminal_ok = true;
    loop {
        let n = match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        };
        if shared.stop.load(Ordering::SeqCst) {
            break;
        }
        let chunk = &buf[..n];
        if terminal_ok {
            terminal_ok = terminal
                .write_all(chunk)
                .and_then(|()| terminal.flush())
                .is_ok();
        }
        shared.write_file(chunk);
    }
}

/// 読み取りのスレッドが全員終わるのを、合わせて `limit` まで待つ(BR4.5)。終われば `true`。
fn wait_copiers(done: &mpsc::Receiver<()>, limit: Duration) -> bool {
    let deadline = Instant::now() + limit;
    loop {
        let left = deadline.saturating_duration_since(Instant::now());
        match done.recv_timeout(left) {
            Ok(()) => continue,
            Err(RecvTimeoutError::Disconnected) => return true,
            Err(RecvTimeoutError::Timeout) => return false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FailingChild;
    impl Waitable for FailingChild {
        fn id(&self) -> u32 {
            1
        }
        fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
            Err(io::Error::other("fake"))
        }
    }

    struct NeverExits;
    impl Waitable for NeverExits {
        fn id(&self) -> u32 {
            4242
        }
        fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
            Ok(None)
        }
    }

    #[test]
    fn try_wait_の失敗は_wait_failed() {
        let signal = InterruptSignal::new();
        let mut forwards = 0;
        let result = wait_loop(
            &mut FailingChild,
            &signal,
            &mut |_, _| forwards += 1,
            &mut |_| {},
        );
        assert!(matches!(result, Waited::WaitFailed));
        assert_eq!(forwards, 0);
    }

    #[test]
    fn 合図が立てば_1回だけ転送し_間隔は_10ms_まで広がる() {
        let signal = InterruptSignal::new();
        signal.raise(15);
        let mut forwarded: Vec<(u32, i32)> = Vec::new();
        let mut sleeps: Vec<Duration> = Vec::new();
        // 6 回待ったところで、失敗にして抜ける(終わらない子の代わり)。
        let mut child = CountingChild {
            inner: NeverExits,
            left: 6,
        };
        let result = wait_loop(
            &mut child,
            &signal,
            &mut |pid, n| forwarded.push((pid, n)),
            &mut |d| sleeps.push(d),
        );
        assert!(matches!(result, Waited::WaitFailed));
        assert_eq!(forwarded, [(4242, 15)]);
        let ms: Vec<u128> = sleeps.iter().map(|d| d.as_millis()).collect();
        assert_eq!(ms, [1, 2, 4, 8, 10, 10]);
    }

    struct CountingChild {
        inner: NeverExits,
        left: u32,
    }
    impl Waitable for CountingChild {
        fn id(&self) -> u32 {
            self.inner.id()
        }
        fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
            if self.left == 0 {
                return Err(io::Error::other("stop"));
            }
            self.left -= 1;
            self.inner.try_wait()
        }
    }

    #[test]
    fn 合図が立たなければ転送しない() {
        let signal = InterruptSignal::new();
        let mut forwards = 0;
        let mut child = CountingChild {
            inner: NeverExits,
            left: 3,
        };
        let _ = wait_loop(&mut child, &signal, &mut |_, _| forwards += 1, &mut |_| {});
        assert_eq!(forwards, 0);
    }

    fn line(command: &str) -> CommandLine {
        crate::split::check_command(&conductor_core::Command::Line(command.into()))
            .expect("分けられる")
    }

    #[test]
    fn 空の_program_は起動失敗() {
        for program in ["", "  "] {
            let empty = CommandLine {
                program: program.into(),
                args: vec![],
            };
            let (e, log) = run_job(&empty, &[], &InterruptSignal::new(), None);
            assert_eq!(e.result, ExecutionResult::StartFailed);
            assert_eq!(log, LogOutcome::default());
        }
    }

    #[test]
    fn 起動前の合図は起動しない() {
        let s = InterruptSignal::new();
        s.raise(2);
        let (e, _) = run_job(&line("definitely-not-run"), &[], &s, None);
        assert_eq!(e.result, ExecutionResult::NotStarted { signal: 2 });
    }

    #[test]
    fn 存在しないコマンドは起動失敗() {
        let (e, log) = run_job(
            &line("conductor-no-such-command-zzz --flag"),
            &[],
            &InterruptSignal::new(),
            None,
        );
        assert_eq!(e.result, ExecutionResult::StartFailed);
        assert_eq!(log, LogOutcome::default());
    }
}
