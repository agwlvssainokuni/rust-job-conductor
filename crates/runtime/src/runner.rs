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

use std::io;
use std::process::{Child, Command, ExitStatus};
use std::time::Duration;

use conductor_core::{JobDefinition, JobExecution};

use crate::platform;
use crate::split::command_line;
use crate::types::InterruptSignal;

/// 待ちの間隔の最初と最大(ミリ秒)。
const FIRST_INTERVAL_MS: u64 = 1;
const MAX_INTERVAL_MS: u64 = 10;

/// `try_wait` と pid だけを持つ、子プロセスの見え方(待ちのくり返しを、プロセスなしで試すため)。
pub(crate) trait Waitable {
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
pub(crate) enum Waited {
    /// 終了を観測した(回収した)。
    Exited(ExitStatus),
    /// 終了の状態を取得できなかった。
    WaitFailed,
}

/// 待ちのくり返し(WaitLoop)。
///
/// 直前の `try_wait` が「まだ」を返した回にだけ、合図を見て、立っていれば `forward` を 1 回だけ
/// 呼ぶ(`forward(pid, シグナル番号)`)。`try_wait` の失敗は、`WaitFailed`。
pub(crate) fn wait_loop<C: Waitable>(
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

fn base_execution(job: &JobDefinition) -> JobExecution {
    JobExecution {
        job_id: job.id.clone(),
        command: job.command.clone(),
        started: true,
        exit_code: None,
        terminated_by_signal: None,
        start_failure: false,
        interrupted: false,
        signal_number: None,
    }
}

/// ジョブを、シェルを介さずに起動し、終了を待つ(契約 C3 の `run_job`。W1)。
///
/// - command を分ける(BR1.1。配列は、要素をそのまま引数にする)。分けられないとき(Cli が、実行の前に検査するため、通常は来ない)は、
///   `start_failure`(防御)。
/// - 合図が、起動の前に立っていれば、起動せず、`started = false`、`interrupted = true`(BR1.6)。
/// - 起動する(UNIX 系は新しいプロセスグループ)。作業ディレクトリ、環境変数、標準入出力は引き継ぐ。
///   起動できなければ `start_failure`(BR1.2〜BR1.4)。
/// - 終了まで待つ。時間切れにしない。再試行しない(BR1.5)。実行中に合図が立てば、シグナルを
///   ジョブに 1 回だけ転送する(BR1.7)。
/// - 終了の観測の直後に、合図が立っていれば、`interrupted = true` と `signal_number`(BR1.7)。
///
/// 失敗は、すべて戻り値(`JobExecution`)で返し、パニックしない(NFR4.7)。
pub fn run_job(job: &JobDefinition, signal: &InterruptSignal) -> JobExecution {
    let mut execution = base_execution(job);

    let line = match command_line(&job.command) {
        Ok(line) => line,
        Err(_) => {
            execution.start_failure = true;
            return execution;
        }
    };

    if let Some(number) = signal.signal_number() {
        execution.started = false;
        execution.interrupted = true;
        execution.signal_number = Some(number);
        return execution;
    }

    let mut command = Command::new(&line.program);
    command.args(&line.args);
    platform::configure_spawn(&mut command);
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(_) => {
            execution.start_failure = true;
            return execution;
        }
    };

    let waited = wait_loop(
        &mut child,
        signal,
        &mut platform::forward_signal,
        &mut std::thread::sleep,
    );
    match waited {
        Waited::Exited(status) => {
            execution.exit_code = status.code();
            execution.terminated_by_signal = platform::termination_signal(&status);
            if let Some(number) = signal.signal_number() {
                execution.interrupted = true;
                execution.signal_number = Some(number);
            }
        }
        Waited::WaitFailed => {
            // 終了の状態を取得できない。起動失敗として返し、conductor-core がエラー(127)として扱う(安全側)。
            execution.start_failure = true;
        }
    }
    execution
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

    fn job(command: &str) -> JobDefinition {
        JobDefinition {
            id: "j".into(),
            command: command.into(),
            warning_exit_codes: vec![],
            success_exit_codes: None,
            transitions: vec![],
        }
    }

    #[test]
    fn 分けられない_command_は起動失敗() {
        let e = run_job(&job("echo 'x"), &InterruptSignal::new());
        assert!(e.started && e.start_failure && !e.interrupted);
        let e = run_job(&job("  "), &InterruptSignal::new());
        assert!(e.start_failure);
    }

    #[test]
    fn 起動前の合図は起動しない() {
        let s = InterruptSignal::new();
        s.raise(2);
        let e = run_job(&job("definitely-not-run"), &s);
        assert!(!e.started && e.interrupted && !e.start_failure);
        assert_eq!(e.signal_number, Some(2));
        assert_eq!((e.exit_code, e.terminated_by_signal), (None, None));
    }

    #[test]
    fn 存在しないコマンドは起動失敗() {
        let e = run_job(
            &job("conductor-no-such-command-zzz --flag"),
            &InterruptSignal::new(),
        );
        assert!(e.started && e.start_failure);
        assert_eq!((e.exit_code, e.terminated_by_signal), (None, None));
    }
}
