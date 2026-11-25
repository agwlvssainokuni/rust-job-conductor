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

//! Windows の `platform`。
//!
//! ジョブは、同じコンソールで起動し、Ctrl+C は OS が同じコンソールのジョブに配る。この Unit は、
//! 追加の送信をしない(BR1.7)。

use std::fs::OpenOptions;
use std::path::Path;
use std::process::{Command, ExitStatus};

/// Windows は、新しく作るファイルの権限を既定のままにする(ログのファイル。v0.10.0 の BR3.4)。
pub(crate) fn restrict_to_owner(_options: &mut OpenOptions) {}

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
