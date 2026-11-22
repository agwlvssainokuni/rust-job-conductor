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

//! conductor-runtime: conductor の実行ライブラリ(Unit conductor-runtime)。
//!
//! ジョブの直接起動とシグナルの転送(JobRunner)と、ステートの保持・照合・初期化・記録
//! (StateStore)を受け持つ。判断(分類、次の動き、起動前の判断)は、`conductor-core`(conductor-core)が
//! 行い、コマンドと表示、シグナルの受け取りは、conductor-cli が行う。共有する型(`JobExecution`、
//! `PipelineState` など)は、`conductor-core` が定める。
//!
//! # 公開する関数(契約 C3)
//!
//! | 関数 | 役割 | 入出力 | 失敗 |
//! |------|------|--------|------|
//! | [`run_job`] | ジョブを、シェルを介さずに起動し、終了を待つ(W1。BR1.x) | `JobDefinition`、分割済みの [`CommandLine`]([`check_command`] の結果)、足す環境変数、[`InterruptSignal`] → `JobExecution` | なし(起動失敗は `StartFailed` の値) |
//! | [`load_env_files`] | `env_file` の `.env` を読む(U1。BR1.7、BR3.x、BR6.4) | 定義ファイルのパス、`env_file` の記述 → `LoadedEnvFiles` | `EnvFileError` の並び(複数) |
//! | [`wait_for`] | リトライの前の待機。中断の合図を短い間隔で確かめる(BR2.13、NFR7) | 秒数、[`InterruptSignal`] → [`WaitOutcome`] | なし |
//! | [`check_command`] | command を、起動せずに検査する(BR1.1) | [`Command`](文字列または配列) → [`CommandLine`] | [`CommandError`] |
//! | [`read_state`] | ステートを読む(W2。BR2.1〜BR2.3) | 定義ファイルのパス → `Option<PipelineState>`(`None` が空) | [`StateError`] |
//! | [`read_state_raw`] | ステートを、完了のものも含めて読む(`watch` と `graph --state` 用。FR3.1) | 定義ファイルのパス → `Option<PipelineState>`(`None` はファイルなし・長さ 0) | [`StateError`] |
//! | [`write_state`] | ステートを、原子的に書く(W3。BR2.4)。耐久性の指定つき | 定義ファイルのパス、ステート、[`Durability`] | [`StateError`](`writeFailed`) |
//! | [`check_consistency`] | ステートと定義の食い違いを検知する(W4。BR2.5) | ステート、定義 → [`ConsistencyResult`] | なし(純粋) |
//! | [`reset_state`] | ステートを、検査を迂回して消す(W5。BR2.6) | 定義ファイルのパス | [`StateError`](`removeFailed`) |
//! | [`reset_all_states`] | その定義のステートを、run ID の有無によらず、すべて消す(FR5) | 定義ファイルのパス → [`ResetAllReport`] | [`StateError`] |
//! | [`create_file`] | ファイルを排他的に作る、または置き換える(`init`。BR1.2、BR1.3、BR1.6) | パス、内容、`force` → [`CreateOutcome`] | [`CreateError`] |
//! | [`initial_state`] | 空のステートを、定義から作る(BR2.8) | 定義、パス、run ID → `PipelineState` | なし(純粋) |
//! | [`apply_outcome`] | ジョブの結果を記録する(BR2.8) | ステート、結果、次の動き → ステート | なし(純粋) |
//! | [`apply_skip`] | スキップを記録し、完了を判定する(BR2.8) | ステート、理由、定義 → ステート | なし(純粋) |
//! | [`apply_interrupt`] | 中断を記録する(BR2.8) | ステート、`ResumePoint` → ステート | なし(純粋) |
//!
//! `initial_state`、`apply_outcome`、`apply_skip`、`apply_interrupt` は、`conductor-core` の
//! `flow::record` にある(`--dry-run` の模擬と共に使う)。`check_consistency` と [`ConsistencyResult`]、
//! [`MismatchKind`] は、`conductor-core` の `consistency` にある。この Unit は、同じ名前で再公開する。
//!
//! `read_state`・`write_state`・`reset_state` は、定義ファイルのパスに加えて run ID(`Option<&RunId>`)を取る。
//! run ID があるときのステートは `<定義ファイル名>.<run-id>.state` で、中身の `run_id` と照合する。
//!
//! ファイルに触れるのは、`read_state`、`read_state_raw`、`write_state`、`reset_state`、
//! `reset_all_states`、`load_env_files`、`create_file` だけ。プロセスに触れるのは、
//! `run_job` だけ。それ以外は、引数だけから結果を決める純粋な処理で、ファイルにもプロセスにも
//! 環境変数にも触れない。
//!
//! # モジュールの構成(logical-components との対応)
//!
//! | モジュール | 部品 | 責務 |
//! |------------|------|------|
//! | `types` | 型(Types) | [`CommandLine`]、[`InterruptSignal`]、[`StateLocation`]、[`ConsistencyResult`]、[`Durability`] |
//! | `error` | 表示用の組み立て(Describe) | kind ごとの固定の文と、ステートファイルのパスだけでエラーを表示する |
//! | `split` | 分け方(Split) | BR1.1 の規則を、文字を 1 つずつ読む状態機械で実装する(再帰なし) |
//! | `runner` | 起動(Spawn)、待ちのくり返し(WaitLoop) | 直接起動し、`try_wait` のくり返し(1ms から 10ms)で待つ。回収はここだけ |
//! | `platform` | 送信(platform::signal)ほか | OS 固有のコードは、ここだけ。`unsafe` は、UNIX 系の `kill` の呼び出し 1 か所だけ |
//! | `state::reader` | ステートの読み込み(StateReader) | 上限 + 1 バイトの読み取り、版の確認(先)、走査、矛盾の検査 |
//! | `state::writer` | ステートの書き込み(StateWriter)、初期化(Reset) | 一時ファイルの新規作成、同期、置き換え、後始末 |
//! | `state::reset_all` | すべてのステートの初期化(ResetAll) | その定義のステート(run ID の有無によらない)を探して消す |
//! | `envfile` | `.env` の読み込み(EnvFileLoader) | `env_file` の場所の決定と読み取り。解析は `conductor-core` の `envfile` |
//! | `file_create` | ファイルの作成(RuntimeFileCreate) | `init` の出力先を、排他的に作る、または置き換える |
//! | `temp_file` | 一時ファイル(TempFile) | 一時ファイルの名前と新規作成、連番(1 つ)、親のディレクトリの扱い。`state`・`file_create`・`platform` が共有する |
//! | `wait` | 待機(Wait) | リトライの前の待機と、中断の合図の確認 |
//! | `src/bin/conductor-test-helper.rs` | テスト専用の補助の実行ファイル | 引数・環境変数・終了コード・シグナルの受信を出力する。配布には使わない |
//!
//! # 呼び出しの順序(契約 C2 の順序の続き)
//!
//! 1. [`read_state`] でステートを読む(失敗なら、ジョブを実行せず終わる)。ステートがあれば、
//!    [`check_consistency`] で定義と照合する。空なら [`initial_state`] で作る。
//! 2. ジョブごとに、`decide_before_run`(conductor-core)の判断に従う。
//!    - `Reject`: ジョブを実行せず、ステートも変えない。
//!    - `Skip`: [`apply_skip`] → [`write_state`]([`Durability::Durable`])。
//!    - `Run`: ステートが空だったときは、最初のジョブの**前**に、初期ステートを
//!      [`write_state`]([`Durability::Fast`])で書く(書けないディレクトリを、ジョブの前に検知する)。
//!      そのあと、[`run_job`] → `classify_and_decide`(conductor-core)→ [`apply_outcome`] →
//!      [`write_state`]([`Durability::Durable`])。
//! 3. 完了(`completed`)のとき、記録のあとに、呼び出し側が、削除を求められていれば [`reset_state`] を
//!    呼ぶ。この Unit は、完了を判断せず、初期化の手段だけを提供する(`apply_skip` と
//!    `apply_outcome` が、`completed` を書く)。完了のまま残ったステートは、[`read_state`] が空として
//!    返すため、次の実行は先頭から始まる(`watch` と `graph --state` だけは、[`read_state_raw`] で完了のステートを読む)。
//!
//! # 保証の範囲(at-least-once)
//!
//! ジョブの終了と、その結果の [`write_state`] のあいだに、プロセスが落ちる(kill される)と、
//! ジョブは終わっているのに、ステートに記録されない。次の呼び出しでは、そのジョブは未実行として
//! 扱われ、再実行される(冪等でないジョブは、2 回動きうる)。ステートが壊れた内容になることは
//! ない(原子的な置き換え)。conductor が SIGKILL などで強制終了されたとき、ジョブのプロセスは
//! 残って動き続けうる(この Unit は、検知も停止もしない)。排他制御はしない(同じ定義への同時の
//! 実行は、それぞれ動き、後に書いた方が勝つ)。
//!
//! # 守る性質
//!
//! - 外部の入力(command、ステートファイル、OS の結果)に対して、パニックしない。失敗は、
//!   戻り値のエラー(`JobExecution` の `StartFailed`、[`StateError`]、[`CommandError`])で返す(NFR4.7)。
//! - エラーの表示に、入力の値、ライブラリのエラー文を含めない。ステートファイルに、command、
//!   引数、環境変数、標準出力の内容を書かない(NFR4.5)。
//! - シグナルは、ジョブの子プロセスの pid だけに、回収の前に、1 回だけ、`SIGHUP`・`SIGINT`・
//!   `SIGQUIT`・`SIGTERM` のどれかを送る(NFR4.8)。プロセスグループ全体には送らない。
//!
//! # 公開しないもの
//!
//! - `platform` モジュール(OS 固有の呼び出し)、待ちのくり返し(WaitLoop)、一時ファイルの作成と
//!   後始末の内部関数。
//! - ジョブの強制終了(SIGKILL)、時間の上限、自動の再試行、ステートの場所の指定、ファイルのロック
//!   (いずれも、この版では提供しない)。

#![deny(unsafe_code)]

mod envfile;
pub mod error;
mod file_create;
mod platform;
mod runner;
mod split;
mod state;
mod temp_file;
pub mod types;
mod wait;

pub use conductor_core::check_consistency;
pub use conductor_core::{apply_interrupt, apply_outcome, apply_skip, initial_state};
pub use envfile::load_env_files;
pub use error::*;
pub use file_create::{CreateError, CreateOutcome, create_file};
pub use runner::run_job;
pub use split::check_command;
pub use state::reader::{read_state, read_state_raw};
pub use state::reset_all::{ResetAllReport, reset_all_states};
pub use state::writer::{reset_state, write_state};
pub use types::*;
pub use wait::{WaitOutcome, wait_for};
