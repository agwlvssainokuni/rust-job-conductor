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

//! Windows のシグナルの監視。`ctrlc` のハンドラ(ライブラリが専用のスレッドで呼ぶ)から、
//! 番号 2(Ctrl+C)で、合図に `raise` する。

use std::sync::Arc;

use conductor_runtime::InterruptSignal;

use crate::deps::SignalError;

/// Ctrl+C の番号(BR4.1)。
const CTRL_C_NUMBER: i32 = 2;

pub(super) fn spawn_watcher(signal: Arc<InterruptSignal>) -> Result<(), SignalError> {
    ctrlc::set_handler(move || signal.raise(CTRL_C_NUMBER)).map_err(|e| match e {
        ctrlc::Error::MultipleHandlers => SignalError::AlreadyRegistered,
        _ => SignalError::Failed,
    })
}
