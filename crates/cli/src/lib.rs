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

//! conductor-cli: `conductor` の実行ファイル(Unit conductor-cli)のライブラリ部分。
//!
//! コマンド(`run`・`pipeline`・`watch`・`reset`・`validate`・`convert`・`graph`・`init`)の受け付け、1 回の実行の流れの組み立て
//! (Orchestrator)、シグナル、表示(英語・日本語)、終了コードを受け持つ。ジョブの起動と
//! ステートは `conductor-runtime`(conductor-runtime)、定義の検証と判断は `conductor-core`(conductor-core)、
//! 定義ファイルの変換は `conductor-converter`(conductor-converter)が行い、この Unit は、それらを、
//! 決まった順に呼ぶ(ADR-002。唯一の調整役)。`main.rs`(実行ファイル `conductor`)は、
//! パニックのフックを設定して、[`run_cli`] を呼び、返った終了コードで 1 回だけ終了する薄い入口である。
//!
//! # 入口の関数
//!
//! | 関数 | 役割 |
//! |------|------|
//! | [`run_cli`] | 引数を受け取り、本番の依存(実際の他の Unit の関数、標準出力・標準エラー、環境、シグナル)で動かし、終了コードを返す |
//! | [`run_cli_with`] | 依存([`Deps`]。`Runtime`・`Sink`・`Env`・`Signals`)を引数で受け取る。テストが、偽の実装を渡して使う |
//! | [`parse_args`] | 引数の解釈だけ(純粋) |
//! | [`install_panic_hook`] | パニックのフック(固定の文を出し、終了コード 70) |
//!
//! # 差し替え口(trait)
//!
//! | trait | 本番の実装 | 責務 |
//! |-------|------------|------|
//! | [`Runtime`] | [`SystemRuntime`] | `load_definition`・`check_command`・`load_env_files`・`run_job`・`wait`・`read_state`・`read_state_raw`・`write_state`・`reset_state`・`reset_all_states`・`run_id_env`・`convert`・`create_file` |
//! | [`Sink`] | [`StdSink`] | 標準出力・標準エラーへの書き込み(失敗は値で返す) |
//! | [`Env`] | [`SystemEnv`] | 環境変数、標準出力が端末か |
//! | [`Signals`] | [`SystemSignals`] | シグナルの監視の開始(プロセスで 1 回。2 回目は `AlreadyRegistered`) |
//!
//! 純粋な関数(`decide_before_run`、`classify_and_decide`、`apply_*`、`check_consistency` など)は、
//! trait を通さず、流れが直接呼ぶ。
//!
//! # モジュールの構成(logical-components との対応)
//!
//! | モジュール | 部品 | 責務 |
//! |------------|------|------|
//! | `main.rs` | 入口(main) | パニックのフックを設定し、`run_cli` の結果で、1 回だけ `std::process::exit` |
//! | [`args`] | 引数の解釈(ArgParser) | `lexopt` で字句に分け、コマンドごとの文法の表で検査する |
//! | `ctx`、`step`、`retry`、`run`、`pipeline`、`dry_run`、`graph`、`init`、`aux_commands` | 流れ(Orchestrator) | W2(run)、W3(pipeline)、1 つのジョブの共通の手順(`step`)、リトライを含む実行(`retry`)、`--dry-run`(計画。U3)、`graph`(図)、`init`(雛形)、W4〜W6(watch・reset・validate・convert) |
//! | [`run_id`] | run ID の解決 | `--run-id` の指定と環境変数から、この実行の run ID を決める(FR1.2〜FR1.4、FR4.3) |
//! | [`platform`] | シグナルの監視(SignalWatcher)ほか | OS 固有のコードは、ここだけ(`cfg(unix)`/`cfg(windows)`) |
//! | [`exit`] | 終了コードの決定(ExitMapper) | 純粋な関数。0〜255 に合わせる(BR5.2) |
//! | [`report`] | 表示(Reporter)、watch の表(WatchView)、エスケープ(Escape) | 文言(英語・日本語)の組み立て。言語・色は遅延 |
//! | [`panic_hook`] | パニックのフック(PanicHook) | 固定の英語の文を出し、70 で終了(終了の呼び出しは差し替えられる) |
//! | [`deps`] | 差し替え口(Deps) | 上の trait と、本番の実装 |
//!
//! # 終了コード
//!
//! 成功・警告・スキップは 0。エラーのジョブは、ジョブの終了コード(起動失敗は 127、シグナル終了・
//! 中断は 128 + 番号、成功コードに含まれない 0 は 1)を、どの OS でも 0〜255 に合わせた値
//! (範囲外は下位 8 ビット。エラーで 0 になるなら 1)。`conductor` 自身のエラーは、定義の誤り 64、
//! ステートの誤り 65、拒否 66、変換の誤り 67、使い方の誤り 68、`init` の作る先が既にある 69、
//! 内部の誤り 70。
//! 70 とジョブ自身の終了コードは重なりうる(区別は、標準エラーの固定の文で行う)。
//!
//! # 保証の範囲(at-least-once)
//!
//! 流れの順序は固定である: 判断 → (必要なら)初期ステート → ジョブ → 記録 → 書き込み → 初期化。
//! ジョブの終了と、その結果の書き込みのあいだに、プロセスが落ちる(kill される)と、ジョブは
//! 終わっているのに、ステートに記録されない。次の呼び出しでは、そのジョブは未実行として扱われ、
//! 再実行される(冪等でないジョブは、2 回動きうる)。結果の書き込みが失敗したときは、ジョブの
//! 終了コードではなく 65 で終わり、「ジョブは実行されました」と表示する。ステートが壊れた
//! 内容になることはない(conductor-runtime の原子的な置き換え)。排他制御はしない。自動の再試行もしない。
//!
//! # 守る性質
//!
//! - `#![forbid(unsafe_code)]`。ネットワークに接続せず、テレメトリもない。
//! - 外部の入力(引数、環境変数、ファイル、シグナル)に対して、パニックしない。
//! - 入力に由来する文字列は、制御文字・書式文字・双方向の制御文字をエスケープして表示する。
//!   ジョブ本体の出力は、そのまま通す(`conductor` は、ジョブの標準出力・標準エラーを加工しない)。

#![forbid(unsafe_code)]

pub mod args;
mod aux_commands;
mod ctx;
pub mod deps;
mod dry_run;
pub mod exit;
mod graph;
mod init;
pub mod panic_hook;
mod pipeline;
pub mod platform;
pub mod report;
mod retry;
mod run;
pub mod run_id;
mod step;
pub mod types;

use std::ffi::OsString;

pub use args::parse_args;
pub use deps::{
    Deps, Env, Runtime, SignalError, Signals, Sink, StdSink, SystemEnv, SystemRuntime,
    SystemSignals,
};
pub use panic_hook::{install_panic_hook, install_panic_hook_with, on_panic};
pub use types::{
    CliCommand, CommandKind, DryRunOptions, GraphFormat, GraphOptions, InitOptions, InitTarget,
    Language, Level, ParsedArgs, RunIdArg, UsageError, UsageErrorKind,
};

use ctx::Ctx;
use exit::{EXIT_OK, EXIT_USAGE};
use report::{DisplayEvent, Reporter};

/// 本番の依存で、`args`(プログラム名を除く)のコマンドを実行し、終了コードを返す。
///
/// `main` は、この結果で、1 回だけ `std::process::exit` を呼ぶ。
pub fn run_cli<I>(args: I, env: &dyn Env, sink: &mut dyn Sink) -> i32
where
    I: IntoIterator<Item = OsString>,
{
    let runtime = SystemRuntime;
    let signals = SystemSignals;
    let mut deps = Deps {
        runtime: &runtime,
        sink,
        env,
        signals: &signals,
    };
    run_cli_with(args, &mut deps)
}

/// 差し替えた依存で、コマンドを実行し、終了コードを返す(W1〜W6)。
pub fn run_cli_with<I>(args: I, deps: &mut Deps<'_>) -> i32
where
    I: IntoIterator<Item = OsString>,
{
    let parsed = match parse_args(args) {
        Ok(parsed) => parsed,
        Err(error) => {
            let reporter = Reporter::new(deps.env, false);
            reporter.show(&mut *deps.sink, &DisplayEvent::Usage(&error));
            return EXIT_USAGE;
        }
    };
    let cmd = match parsed {
        ParsedArgs::Help => {
            Reporter::new(deps.env, false).show(&mut *deps.sink, &DisplayEvent::Help);
            return EXIT_OK;
        }
        ParsedArgs::Version => {
            Reporter::new(deps.env, false).show(&mut *deps.sink, &DisplayEvent::Version);
            return EXIT_OK;
        }
        ParsedArgs::Command(cmd) => cmd,
    };

    let mut ctx = Ctx {
        runtime: deps.runtime,
        signals: deps.signals,
        sink: &mut *deps.sink,
        env: deps.env,
        reporter: Reporter::new(deps.env, cmd.verbose),
    };
    match cmd.kind {
        CommandKind::Run => run::run_individual(&mut ctx, &cmd),
        CommandKind::Pipeline => pipeline::run_pipeline(&mut ctx, &cmd),
        CommandKind::Watch => aux_commands::run_watch(&mut ctx, &cmd),
        CommandKind::Reset => aux_commands::run_reset(&mut ctx, &cmd),
        CommandKind::Validate => aux_commands::run_validate(&mut ctx, &cmd),
        CommandKind::Convert => aux_commands::run_convert(&mut ctx, &cmd),
        CommandKind::Graph => graph::run_graph(&mut ctx, &cmd),
        CommandKind::Init => init::run_init(&mut ctx, &cmd),
    }
}
