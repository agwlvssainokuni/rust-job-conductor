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

//! 本番のシグナルの監視の順序(UNIX 系)。時間を空けて届いたシグナルは、到着順で、最初の番号が残る
//! (INT のあとに HUP を送っても、INT の 2 が残る)。ほぼ同時に届く場合は、`signal-hook` が
//! 番号の小さい順に返すため、小さい番号が先になりうる(ここでは、時間を空けて、順番を作る)。
//!
//! 登録はプロセスで 1 回だけのため、このファイルには、テストを 1 つだけ置く。

#![cfg(unix)]

use std::sync::Arc;
use std::time::{Duration, Instant};

use conductor_cli::{Signals, SystemSignals};
use conductor_runtime::InterruptSignal;

fn send(signal: &str) {
    let status = std::process::Command::new("kill")
        .args([format!("-{signal}"), std::process::id().to_string()])
        .status()
        .expect("kill を起動できる");
    assert!(status.success());
}

#[test]
fn int_のあとに_hup_を送ると_int_の番号が残る() {
    let signal = Arc::new(InterruptSignal::new());
    assert_eq!(SystemSignals.start(&signal), Ok(()));
    send("INT");
    let start = Instant::now();
    while !signal.is_raised() && start.elapsed() < Duration::from_secs(10) {
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(signal.signal_number(), Some(2));
    // INT が合図に書かれたあとに、HUP を送る(時間を空けて、順番を作る)
    send("HUP");
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(signal.signal_number(), Some(2), "最初の番号が残る");
}
