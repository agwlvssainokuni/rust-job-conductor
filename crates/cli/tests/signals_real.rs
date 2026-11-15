//! 本番のシグナルの監視(`SystemSignals`。NFR4.3)。UNIX 系だけ。
//!
//! このテストのプロセスは、シグナルの監視を登録する(プロセスで 1 回だけ登録できる)。
//! 登録は 1 つのテストだけで行う。2 つのシグナルを続けて送っても、最初の番号が残ること、
//! 2 回目の `start` が `AlreadyRegistered` になることを確かめる。

#![cfg(unix)]

use std::sync::Arc;
use std::time::{Duration, Instant};

use conductor_cli::{SignalError, Signals, SystemSignals};
use conductor_runtime::InterruptSignal;

fn send(signal: &str) {
    let pid = std::process::id().to_string();
    let status = std::process::Command::new("kill")
        .args([format!("-{signal}"), pid])
        .status()
        .expect("kill を起動できる");
    assert!(status.success());
}

fn wait_for(signal: &InterruptSignal) -> Option<i32> {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(10) {
        if let Some(n) = signal.signal_number() {
            return Some(n);
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    None
}

#[test]
fn 最初のシグナルの番号だけが残り_2_回目の登録は登録済みになる() {
    let signal = Arc::new(InterruptSignal::new());
    assert_eq!(SystemSignals.start(&signal), Ok(()));
    assert!(!signal.is_raised());

    send("TERM");
    assert_eq!(wait_for(&signal), Some(15));
    // 続けて別のシグナルを送っても、最初の番号が残る(`SIGINT` を受け取っても、このプロセスは終わらない)
    send("INT");
    send("HUP");
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(signal.signal_number(), Some(15));

    // 2 回目の登録は、登録済み(別の合図には、配線されない)
    let other = Arc::new(InterruptSignal::new());
    assert_eq!(
        SystemSignals.start(&other),
        Err(SignalError::AlreadyRegistered)
    );
    send("QUIT");
    std::thread::sleep(Duration::from_millis(200));
    assert!(!other.is_raised(), "配線されていない");
}
