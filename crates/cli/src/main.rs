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
