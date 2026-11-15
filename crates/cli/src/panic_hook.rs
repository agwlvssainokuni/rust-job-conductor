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

//! パニックのフック(PanicHook。NFR4.7、D3)。
//!
//! 内部の誤り(パニック)は、固定の英語の文(ロケールを判定しない)を標準エラーに書き、
//! 終了コード 70 で終わる。終了の呼び出しは、差し替えられる(テストが、フックの関数を直接呼ぶ)。
//! リリースの実行ファイルに、テスト専用の経路(パニックを起こすコマンド)は持たせない。
//!
//! ジョブの実行中にパニックすると、子プロセスの終了を待たずに終わり、ジョブが孤児として残りうる。
//! ステートは、直前の完全な内容のままで、次の実行は、そのジョブを再実行しうる(at-least-once の
//! 範囲に含める)。

use std::io::Write;

use crate::exit::EXIT_INTERNAL;
use crate::report::messages::INTERNAL_ERROR;

/// フックの本体。固定の文を `err` に書き、終了コード 70 で `exit` を呼ぶ(書き込みの失敗は無視する)。
pub fn on_panic(err: &mut dyn Write, exit: &mut dyn FnMut(i32)) {
    let _ = writeln!(err, "{INTERNAL_ERROR}");
    let _ = err.flush();
    exit(EXIT_INTERNAL);
}

/// パニックのフックを設定する。`exit` は、プロセスを終える関数(本番は `std::process::exit`)。
///
/// `main` の最初に呼ぶ。`exit` が戻る(テスト)と、パニックの伝播は、通常どおり続く。
pub fn install_panic_hook_with(exit: fn(i32)) {
    std::panic::set_hook(Box::new(move |_info| {
        let mut stderr = std::io::stderr();
        on_panic(&mut stderr, &mut |code| exit(code));
    }));
}

/// 本番のパニックのフックを設定する(`std::process::exit` で終わる)。
pub fn install_panic_hook() {
    install_panic_hook_with(|code| std::process::exit(code));
}
