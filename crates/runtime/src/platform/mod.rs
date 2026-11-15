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
