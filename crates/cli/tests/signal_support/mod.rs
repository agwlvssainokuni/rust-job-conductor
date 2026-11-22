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

//! 実際のシグナルを送るテストの共通の補助(UNIX 系。`kill` を起動して送る)。
//! 取り込むテストのファイルは、どれも `#![cfg(unix)]`。
#![allow(dead_code)]

use std::process::Command;

/// プロセス `pid` に、シグナル `signal`(`TERM` などの名前)を送る。
pub fn send(signal: &str, pid: u32) {
    let status = Command::new("kill")
        .args([format!("-{signal}"), pid.to_string()])
        .status()
        .expect("kill を起動できる");
    assert!(status.success());
}

/// このテストのプロセス自身に、シグナル `signal` を送る。
pub fn send_self(signal: &str) {
    send(signal, std::process::id());
}
