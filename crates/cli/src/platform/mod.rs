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

//! OS 固有のコード(NFR3.1)。`cfg(unix)` / `cfg(windows)` は、このモジュールの中だけに置く。
//!
//! - シグナルの監視(SignalWatcher。NFR4.3): 専用のスレッドを 1 つ起動し、受け取った最初の
//!   番号を、中断の合図に書く。UNIX 系は `signal-hook` の `Signals`(イテレータ)、Windows は
//!   `ctrlc`。この Unit は `unsafe` を使わない(`low_level` も使わない)。
//! - 端末の色が使える OS か(UNIX 系だけ。Windows は色なし。BR6.5)。

use std::sync::Arc;

use conductor_runtime::InterruptSignal;

use crate::deps::SignalError;

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

/// シグナルの監視を始める。プロセスで 1 回だけ呼ぶ(本番は `Ctx::prepare_session` の 1 か所)。
/// 2 回呼んだときの動きは契約の外(UNIX 系は監視が 2 つになり、Windows は `Failed`)。
pub(crate) fn start_signal_watcher(signal: Arc<InterruptSignal>) -> Result<(), SignalError> {
    spawn_watcher(signal)
}

#[cfg(unix)]
fn spawn_watcher(signal: Arc<InterruptSignal>) -> Result<(), SignalError> {
    unix::spawn_watcher(signal)
}

#[cfg(windows)]
fn spawn_watcher(signal: Arc<InterruptSignal>) -> Result<(), SignalError> {
    windows::spawn_watcher(signal)
}

/// 端末に、色(ANSI の制御文字)を出してよい OS か。
pub(crate) fn color_supported() -> bool {
    cfg!(unix)
}
