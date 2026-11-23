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
//! 終了コード 70 で終わる。リリースの実行ファイルに、テスト専用の経路(パニックを起こすコマンド)は
//! 持たせない。
//!
//! 本体(文を書き、終了コードを返す)は非公開の関数で、このモジュールの単体テストが確かめる。
//! フックの設定そのもの(パニックのとき 70 で終わること)は、自動のテストでは確かめない(テストの
//! プロセスが終わってしまうため。`main` が最初に [`install_panic_hook`] を呼ぶことは、読んで確かめる)。
//!
//! ジョブの実行中にパニックすると、子プロセスの終了を待たずに終わり、ジョブが孤児として残りうる。
//! ステートは、直前の完全な内容のままで、次の実行は、そのジョブを再実行しうる(at-least-once の
//! 範囲に含める)。

use std::io::Write;

use crate::exit::EXIT_INTERNAL;
use crate::report::messages::INTERNAL_ERROR;

/// フックの本体。固定の英語の文を `err` に書き、終了コード 70 を返す(書き込みの失敗は無視する)。
fn write_internal_error(err: &mut dyn Write) -> i32 {
    let _ = writeln!(err, "{INTERNAL_ERROR}");
    let _ = err.flush();
    EXIT_INTERNAL
}

/// パニックのフックを設定する(`main` の最初に呼ぶ)。パニックのとき、本体を標準エラーに書き、
/// 終了コード 70 で終わる(`std::process::exit`)。
pub fn install_panic_hook() {
    std::panic::set_hook(Box::new(|_info| {
        std::process::exit(write_internal_error(&mut std::io::stderr()));
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn フックの本体は固定の英語の文を書き_70_で終わる() {
        let mut err: Vec<u8> = Vec::new();
        let code = write_internal_error(&mut err);
        assert_eq!(
            String::from_utf8_lossy(&err),
            "conductor: error: internal error\n"
        );
        assert_eq!(code, 70);
    }

    #[test]
    fn 標準エラーに書けなくても_70_で終わる() {
        struct Closed;
        impl std::io::Write for Closed {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(std::io::ErrorKind::BrokenPipe.into())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Err(std::io::ErrorKind::BrokenPipe.into())
            }
        }
        let code = write_internal_error(&mut Closed);
        assert_eq!(code, 70);
    }
}
