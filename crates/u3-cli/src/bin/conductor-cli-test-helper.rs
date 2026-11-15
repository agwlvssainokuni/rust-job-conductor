//! テスト専用の補助の実行ファイル(配布には使わない。NFR5.1)。`unsafe` も `libc` も使わない。
//!
//! `conductor-cli-test-helper [--out <file>] <mode> [args...]`
//!
//! `--out` を指定すると、`pid=`・`ready`・`ran=` などの記録を、そのファイルに追記する(テストは、
//! ジョブの標準出力を、`conductor` を介して読めないことがある)。指定がなければ、標準出力に書く。
//!
//! | モード | 動作 |
//! |--------|------|
//! | `exit <n>` | 終了コード n で終わる |
//! | `run <label> <n>` | `ran=<label>` を記録して、終了コード n で終わる(ジョブの流れのテスト用) |
//! | `echo <args...>` | 引数を、1 つずつ `arg[i]="..."` で標準出力に書く |
//! | `env <NAME>...` | 環境変数の値を、`env[NAME]=Some("...")` / `None` で標準出力に書く |
//! | `say <out-text> <err-text> [n]` | 標準出力と標準エラーに、そのまま書いて、終了コード n(既定 0)で終わる |
//! | `sleep [seconds]` | `pid=N` と `ready` を記録して眠り続ける(秒数があれば、その後 99 で終わる)。ハンドラは持たず、シグナルは既定の動作で終わる |
//! | `timing` | 起動の直後と終了の直前に、UNIX 時刻(ナノ秒)を、`start_ns=`・`end_ns=` で書いて終わる |
//! | `burn <ms>` | 指定のミリ秒、CPU を使い続けて終わる(CPU 時間の測定用) |
//!
//! 時刻は、プロセスをまたいで比べられる `SystemTime`(UNIX 時刻)で出す(`Instant` は、プロセスをまたいだ
//! 値を持たず、`unsafe` なしでは、取り出せないため)。

use std::fs::OpenOptions;
use std::io::Write;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn unix_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

fn main() {
    let started = unix_nanos();
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
    let number = |index: usize, default: i32| {
        args.get(index)
            .and_then(|a| a.parse::<i32>().ok())
            .unwrap_or(default)
    };

    match mode.as_str() {
        "exit" => std::process::exit(number(0, 2)),
        "run" => {
            let label = args.first().cloned().unwrap_or_default();
            emit(format!("ran={label}"));
            std::process::exit(number(1, 0));
        }
        "echo" => {
            for (i, a) in args.iter().enumerate() {
                println!("arg[{i}]={a:?}");
            }
        }
        "env" => {
            for name in &args {
                println!("env[{name}]={:?}", std::env::var(name).ok());
            }
        }
        "say" => {
            let out_text = args.first().cloned().unwrap_or_default();
            let err_text = args.get(1).cloned().unwrap_or_default();
            print!("{out_text}");
            eprint!("{err_text}");
            let _ = std::io::stdout().flush();
            let _ = std::io::stderr().flush();
            std::process::exit(number(2, 0));
        }
        "sleep" => {
            emit(format!("pid={}", std::process::id()));
            emit("ready".to_string());
            match args.first().and_then(|a| a.parse::<u64>().ok()) {
                Some(seconds) => {
                    std::thread::sleep(Duration::from_secs(seconds));
                    std::process::exit(99);
                }
                None => loop {
                    std::thread::sleep(Duration::from_secs(3600));
                },
            }
        }
        "timing" => {
            println!("start_ns={started}");
            println!("end_ns={}", unix_nanos());
        }
        "burn" => {
            let limit =
                Duration::from_millis(args.first().and_then(|a| a.parse().ok()).unwrap_or(100));
            let begin = Instant::now();
            let mut x: u64 = 1;
            while begin.elapsed() < limit {
                x = x
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                std::hint::black_box(x);
            }
        }
        _ => std::process::exit(2),
    }
}
