//! Windows の `platform`。
//!
//! ジョブは、同じコンソールで起動し、Ctrl+C は OS が同じコンソールのジョブに配る。この Unit は、
//! 追加の送信をしない(BR1.7)。

use std::path::Path;
use std::process::{Command, ExitStatus};

/// Windows では、ディレクトリの同期を行わない。
pub(crate) fn sync_dir(_dir: &Path) {}

/// Windows は、シグナルによる終了を持たない(終了は、すべて終了コード。Q4: A)。
pub(crate) fn termination_signal(_status: &ExitStatus) -> Option<i32> {
    None
}

/// 追加の指定はしない(同じコンソールで起動する)。
pub(crate) fn configure_spawn(_command: &mut Command) {}

/// 何もしない(Ctrl+C は、OS が配る)。
pub(crate) fn forward_signal(_pid: u32, _signal: i32) {}
