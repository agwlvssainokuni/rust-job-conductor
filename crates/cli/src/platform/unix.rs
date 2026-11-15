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

//! UNIX 系のシグナルの監視。`signal-hook` の `Signals`(イテレータ)を、専用のスレッドで回す。

use std::sync::Arc;

use conductor_runtime::InterruptSignal;
use signal_hook::consts::{SIGHUP, SIGINT, SIGQUIT, SIGTERM};
use signal_hook::iterator::Signals;

use crate::deps::SignalError;

/// SIGHUP・SIGINT・SIGQUIT・SIGTERM を受け取り、合図に `raise` する(最初の番号が残る)専用のスレッドを起動する。
///
/// ほぼ同時に届いた複数のシグナルは、`signal-hook` が番号の小さい順に返すため、「最初の番号」が、
/// 到着順でなく、小さい番号になりうる。
pub(super) fn spawn_watcher(signal: Arc<InterruptSignal>) -> Result<(), SignalError> {
    let mut signals =
        Signals::new([SIGHUP, SIGINT, SIGQUIT, SIGTERM]).map_err(|_| SignalError::Failed)?;
    std::thread::Builder::new()
        .name("conductor-signals".to_string())
        .spawn(move || {
            for number in signals.forever() {
                signal.raise(number);
            }
        })
        .map(|_| ())
        .map_err(|_| SignalError::Failed)
}
