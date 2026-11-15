//! UNIX 系のシグナルの監視。`signal-hook` の `Signals`(イテレータ)を、専用のスレッドで回す。

use std::sync::Arc;

use conductor_runtime::InterruptSignal;
use signal_hook::consts::{SIGHUP, SIGINT, SIGQUIT, SIGTERM};
use signal_hook::iterator::Signals;

use crate::deps::SignalError;

/// SIGHUP・SIGINT・SIGQUIT・SIGTERM を受け取り、合図に `raise` する(最初の番号が残る)専用のスレッドを起動する。
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
