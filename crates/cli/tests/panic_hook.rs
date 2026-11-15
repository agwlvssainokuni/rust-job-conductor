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

//! パニックのフック(NFR4.7、D3)。フックの本体は、終了の呼び出しを差し替えて確かめる。
//!
//! リリースの実行ファイルに、テスト専用の経路(パニックを起こすコマンド)は持たせない。

use std::sync::atomic::{AtomicI32, Ordering};

use conductor_cli::{install_panic_hook_with, on_panic};

static EXIT_CALLED_WITH: AtomicI32 = AtomicI32::new(-1);

#[test]
fn フックの本体は固定の英語の文を書き_70_で終わる() {
    let mut err: Vec<u8> = Vec::new();
    let mut code = None;
    on_panic(&mut err, &mut |c| code = Some(c));
    assert_eq!(
        String::from_utf8_lossy(&err),
        "conductor: error: internal error\n"
    );
    assert_eq!(code, Some(70));
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
    let mut code = None;
    on_panic(&mut Closed, &mut |c| code = Some(c));
    assert_eq!(code, Some(70));
}

#[test]
fn 設定したフックは_パニックのとき終了の関数を_70_で呼ぶ() {
    // このテストのプロセスのフックを差し替える(このファイルのほかのテストは、フックを使わない)。
    install_panic_hook_with(|code| EXIT_CALLED_WITH.store(code, Ordering::SeqCst));
    let result = std::panic::catch_unwind(|| {
        panic!("テストのための、意図したパニック");
    });
    assert!(result.is_err());
    assert_eq!(EXIT_CALLED_WITH.load(Ordering::SeqCst), 70);
    let _ = std::panic::take_hook();
}
