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
