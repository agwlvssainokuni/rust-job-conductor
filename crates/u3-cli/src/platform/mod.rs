//! OS 固有のコード(NFR3.1)。`cfg(unix)` / `cfg(windows)` は、このモジュールの中だけに置く。
//!
//! - シグナルの監視(SignalWatcher。NFR4.3): 専用のスレッドを 1 つ起動し、受け取った最初の
//!   番号を、中断の合図に書く。UNIX 系は `signal-hook` の `Signals`(イテレータ)、Windows は
//!   `ctrlc`。この Unit は `unsafe` を使わない(`low_level` も使わない)。
//! - 端末の色が使える OS か(UNIX 系だけ。Windows は色なし。BR6.5)。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use conductor_runtime::InterruptSignal;

use crate::deps::SignalError;

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

/// プロセスで、監視を登録したか(2 回目の呼び出しを見分ける)。
static REGISTERED: AtomicBool = AtomicBool::new(false);

/// シグナルの監視を始める。プロセスで 1 回だけ登録できる。2 回目は `AlreadyRegistered`。
pub fn start_signal_watcher(signal: Arc<InterruptSignal>) -> Result<(), SignalError> {
    if REGISTERED.swap(true, Ordering::SeqCst) {
        return Err(SignalError::AlreadyRegistered);
    }
    let result = spawn_watcher(signal);
    if result.is_err() {
        REGISTERED.store(false, Ordering::SeqCst);
    }
    result
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
pub fn color_supported() -> bool {
    cfg!(unix)
}
