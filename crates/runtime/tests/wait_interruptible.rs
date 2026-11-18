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

//! リトライの待機 `wait_for` のテスト(U2。BR2.5、BR2.13、BR2.14。FR4.1、NFR7)。

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use conductor_runtime::{InterruptSignal, WaitOutcome, wait_for};

#[test]
fn 待ち時間が経つと_elapsed_を返す() {
    let signal = InterruptSignal::new();
    let started = Instant::now();
    assert_eq!(wait_for(1, &signal), WaitOutcome::Elapsed);
    assert!(started.elapsed() >= Duration::from_secs(1));
}

#[test]
fn 待ち_0_秒は_すぐ_elapsed_を返す() {
    let signal = InterruptSignal::new();
    let started = Instant::now();
    assert_eq!(wait_for(0, &signal), WaitOutcome::Elapsed);
    assert!(started.elapsed() < Duration::from_millis(500));
}

#[test]
fn すでに合図が立っていれば_すぐ_interrupted_を返す() {
    let signal = InterruptSignal::new();
    signal.raise(15);
    let started = Instant::now();
    assert_eq!(wait_for(3600, &signal), WaitOutcome::Interrupted(15));
    assert!(started.elapsed() < Duration::from_millis(500));
}

#[test]
fn 待ち_0_秒でも_合図が立っていれば_interrupted_を返す() {
    let signal = InterruptSignal::new();
    signal.raise(2);
    assert_eq!(wait_for(0, &signal), WaitOutcome::Interrupted(2));
}

#[test]
fn 長い待ちの途中の合図で_1_秒以内に_interrupted_を返す() {
    let signal = Arc::new(InterruptSignal::new());
    let raiser = {
        let signal = Arc::clone(&signal);
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(200));
            signal.raise(2);
        })
    };
    let started = Instant::now();
    assert_eq!(wait_for(3600, &signal), WaitOutcome::Interrupted(2));
    // 合図(200 ミリ秒後)から 1 秒以内、つまり全体で 1.2 秒以内。
    assert!(started.elapsed() < Duration::from_millis(1200));
    raiser.join().expect("合図のスレッドが終わる");
}

#[test]
fn 最初の合図の番号を返す() {
    let signal = InterruptSignal::new();
    signal.raise(1);
    signal.raise(15);
    assert_eq!(wait_for(10, &signal), WaitOutcome::Interrupted(1));
}
