//! OS ごとの違いを閉じ込めるモジュール(NFR3.1)。
//!
//! プロセスグループの指定と、シグナルの送信と、シグナルによる終了の読み取りと、ディレクトリの同期
//! だけがここにある。
//! ほかの部品は、OS を知らない。`cfg(unix)`・`cfg(windows)` と、OS 固有の呼び出しは、
//! このモジュールの外に置かない。

#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub(crate) use unix::{configure_spawn, forward_signal, sync_dir, termination_signal};

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub(crate) use windows::{configure_spawn, forward_signal, sync_dir, termination_signal};
