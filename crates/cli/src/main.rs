//! `conductor` の実行ファイル。薄い入口。
//!
//! パニックのフックを最初に設定し、`run_cli` の結果で、1 回だけ `std::process::exit` を呼ぶ。

#![forbid(unsafe_code)]

use conductor_cli::{StdSink, SystemEnv, install_panic_hook, run_cli};

fn main() {
    install_panic_hook();
    let mut sink = StdSink;
    let code = run_cli(std::env::args_os().skip(1), &SystemEnv, &mut sink);
    std::process::exit(code);
}
