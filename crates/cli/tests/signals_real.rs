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

//! 本番のシグナルの監視(`SystemSignals`。NFR4.3)。UNIX 系だけ。
//!
//! このテストのプロセスは、シグナルの監視を登録する(プロセスで 1 回だけ登録できる)。
//! 登録は 1 つのテストだけで行う。2 つのシグナルを続けて送っても、最初の番号が残ること、
//! 2 回目の `start` が `AlreadyRegistered` になることを確かめる。

#![cfg(unix)]

mod signal_support;

use std::sync::Arc;
use std::time::{Duration, Instant};

use conductor_cli::{SignalError, Signals, SystemSignals};
use conductor_runtime::InterruptSignal;
use signal_support::send_self;

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

    send_self("TERM");
    assert_eq!(wait_for(&signal), Some(15));
    // 続けて別のシグナルを送っても、最初の番号が残る(`SIGINT` を受け取っても、このプロセスは終わらない)
    send_self("INT");
    send_self("HUP");
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(signal.signal_number(), Some(15));

    // 2 回目の登録は、登録済み(別の合図には、配線されない)
    let other = Arc::new(InterruptSignal::new());
    assert_eq!(
        SystemSignals.start(&other),
        Err(SignalError::AlreadyRegistered)
    );
    send_self("QUIT");
    std::thread::sleep(Duration::from_millis(200));
    assert!(!other.is_raised(), "配線されていない");
}
