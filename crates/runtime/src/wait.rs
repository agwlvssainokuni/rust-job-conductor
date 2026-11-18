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

//! リトライの待機(BR2.5、BR2.13、BR2.14。FR4.1、NFR7)。
//!
//! 標準ライブラリだけで、中断の合図を短い間隔で確かめながら待つ。OS 固有のコードは使わない。

use std::thread;
use std::time::{Duration, Instant};

use crate::types::InterruptSignal;

/// 合図を確かめる間隔の上限(BR2.14。1 秒以内に打ち切るため、100 ミリ秒以内にする)。
const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// 待機の結果(BR2.13)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitOutcome {
    /// 指定の時間が経った。
    Elapsed,
    /// 待機中、または待機の開始時に、中断の合図が立っていた(シグナルの番号つき)。
    Interrupted(i32),
}

/// `secs` 秒待つ。合図が立てば、すぐに `Interrupted(番号)` を返す(BR2.5)。
///
/// 合図は、待機の開始時と、50 ミリ秒以下の間隔で確かめる。長い待ちでも、合図から 1 秒以内に
/// 返る(BR2.14)。待ち 0 秒は、合図が立っていなければ、すぐ `Elapsed` を返す。
pub fn wait_for(secs: u64, signal: &InterruptSignal) -> WaitOutcome {
    let started = Instant::now();
    let total = Duration::from_secs(secs);
    loop {
        if let Some(number) = signal.signal_number() {
            return WaitOutcome::Interrupted(number);
        }
        let elapsed = started.elapsed();
        if elapsed >= total {
            return WaitOutcome::Elapsed;
        }
        thread::sleep((total - elapsed).min(POLL_INTERVAL));
    }
}
