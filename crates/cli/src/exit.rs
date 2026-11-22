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

//! 終了コードの決定(ExitMapper。BR5.1〜BR5.3、NFR2.2)。純粋な関数と定数だけ。
//!
//! 終了コードは、どの OS でも、0〜255 の範囲に合わせる(BR5.2)。

use conductor_core::{Classification, JobOutcome};

/// 成功、警告、スキップ。
pub const EXIT_OK: i32 = 0;
/// 定義の誤り(ファイルがない・読めない・検証の誤り・command の誤り)。
pub const EXIT_DEFINITION: i32 = 64;
/// ステートの誤り(読めない・壊れている・食い違い・書き込み・初期化の失敗)と、`watch`・`graph`・
/// `--dry-run` の標準出力の失敗、`init` の書き込みの失敗。
pub const EXIT_STATE: i32 = 65;
/// 拒否(順序外)。
pub const EXIT_REJECTED: i32 = 66;
/// 変換の誤りと、`convert` の出力の失敗。
pub const EXIT_CONVERSION: i32 = 67;
/// 使い方の誤り。
pub const EXIT_USAGE: i32 = 68;
/// `init` の作る先に、ファイルが既にある(`--force` なし。U4 の BR1.2)。
pub const EXIT_EXISTS: i32 = 69;
/// 内部の誤り(パニック、シグナルの準備の失敗)。
pub const EXIT_INTERNAL: i32 = 70;

/// ジョブの結果の `reported_exit_code` を、プロセスの終了コード(0〜255)にする(BR5.2)。
///
/// 範囲内の値は、そのまま。範囲外(255 を超える、または負)は、下位 8 ビット。ただし、
/// エラーのジョブで、下位 8 ビットが 0 になる場合は 1(エラーを成功に見せない)。
pub fn exit_code_for(reported: i32, is_error: bool) -> i32 {
    if (0..=255).contains(&reported) {
        return reported;
    }
    let low = reported & 0xFF;
    if low == 0 && is_error { 1 } else { low }
}

/// ジョブの結果(分類と `reported_exit_code`)から、プロセスの終了コードを決める(BR5.2)。
/// エラーの分類のときだけ、下位 8 ビットの 0 を 1 にする([`exit_code_for`])。
pub fn outcome_exit_code(outcome: &JobOutcome) -> i32 {
    exit_code_for(
        outcome.reported_exit_code,
        outcome.classification == Some(Classification::Error),
    )
}

/// 中断(シグナル)の終了コード: 128 + 番号。0〜255 に合わせる。
pub fn signal_exit_code(signal_number: i32) -> i32 {
    exit_code_for(128i32.saturating_add(signal_number), true)
}
