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

//! テスト専用の補助の実行ファイル(配布には使わない。NFR5.1、D2)。
//!
//! `conductor-test-helper [--out <file>] <mode> [args...]`
//!
//! 出力は、`--out` で指定したファイルに追記する(指定がなければ標準出力)。テストは、ジョブの標準出力を
//! 読めないため、ファイルで結果を受け取る。値は `{:?}` の形で書き、改行や引用符を含んでも 1 行になる。
//!
//! | モード | 動作 |
//! |--------|------|
//! | `echo <args...>` | 引数を 1 つずつ `arg[i]="..."` で出力して、終了コード 0 で終わる |
//! | `run <label> <n>` | `ran=<label>` を出力して、終了コード n で終わる(ジョブの流れのテスト用) |
//! | `exit <n>` | 終了コード n で終わる |
//! | `env <NAME>` | 環境変数の値を `env[NAME]=Some("...")` / `None` で出力する |
//! | `cwd` | 作業ディレクトリを `cwd="..."` で出力する |
//! | `pid` | 自分のプロセス番号を `pid=N` で出力する |
//! | `sleep [seconds]` | `pid=N` と `ready` を出力し、眠り続ける(秒数があれば、その後、終了コード 99 で終わる)。シグナルの受信は、ハンドラなしで、既定の動作による終了で確かめる |
//! | `streams <n>` | `out <i>` を標準出力へ、`err <i>` を標準エラーへ、1 行ずつ交互に n 回書いて(各行の後で flush)、終了コード 0 で終わる(ログの出力の複製の確かめ) |
//! | `orphan <seconds>` | 標準出力と標準エラーを引き継いだ孫(`sleep <seconds>`)を残し、`parent done` を書いてすぐ終了コード 0 で終わる(ログの読み取りの待ち切りの確かめ) |

use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;

fn main() {
    let mut args: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|a| a.to_string_lossy().into_owned())
        .collect();

    let mut out_path: Option<String> = None;
    if args.first().map(String::as_str) == Some("--out") && args.len() >= 2 {
        args.remove(0);
        out_path = Some(args.remove(0));
    }

    let emit = |line: String| match &out_path {
        Some(path) => {
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                let _ = writeln!(file, "{line}");
            }
        }
        None => println!("{line}"),
    };

    let mode = if args.is_empty() {
        String::new()
    } else {
        args.remove(0)
    };

    match mode.as_str() {
        "echo" => {
            for (i, a) in args.iter().enumerate() {
                emit(format!("arg[{i}]={a:?}"));
            }
        }
        "run" => {
            // ジョブの流れのテスト用。`run <label> <code>`: 起動の記録を残して、終了コード code で終わる。
            let label = args.first().cloned().unwrap_or_default();
            emit(format!("ran={label}"));
            let code = args.get(1).and_then(|a| a.parse::<i32>().ok()).unwrap_or(0);
            std::process::exit(code);
        }
        "exit" => {
            let code = args
                .first()
                .and_then(|a| a.parse::<i32>().ok())
                .unwrap_or(2);
            std::process::exit(code);
        }
        "env" => {
            for name in &args {
                emit(format!("env[{name}]={:?}", std::env::var(name).ok()));
            }
        }
        "cwd" => {
            let cwd = std::env::current_dir().ok();
            emit(format!("cwd={cwd:?}"));
        }
        "pid" => emit(format!("pid={}", std::process::id())),
        "sleep" => {
            emit(format!("pid={}", std::process::id()));
            emit("ready".to_string());
            let limit = args.first().and_then(|a| a.parse::<u64>().ok());
            match limit {
                Some(seconds) => {
                    std::thread::sleep(Duration::from_secs(seconds));
                    std::process::exit(99);
                }
                None => loop {
                    std::thread::sleep(Duration::from_secs(3600));
                },
            }
        }
        "streams" => {
            let n = args
                .first()
                .and_then(|a| a.parse::<u32>().ok())
                .unwrap_or(1);
            let mut out = std::io::stdout();
            let mut err = std::io::stderr();
            for i in 0..n {
                let _ = writeln!(out, "out {i}").and_then(|()| out.flush());
                let _ = writeln!(err, "err {i}").and_then(|()| err.flush());
            }
        }
        "orphan" => {
            let seconds = args.first().cloned().unwrap_or_else(|| "30".into());
            let spawned = std::env::current_exe().and_then(|exe| {
                std::process::Command::new(exe)
                    .args(["sleep", &seconds])
                    .spawn()
            });
            if spawned.is_err() {
                std::process::exit(3);
            }
            println!("parent done");
        }
        _ => std::process::exit(2),
    }
}
