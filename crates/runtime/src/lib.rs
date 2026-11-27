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
//! | [`run_job`] | ジョブを、シェルを介さずに起動し、終了を待つ(W1。BR1.x)。ログの指定があれば出力をファイルにも残す(v0.10.0 の BR4.x) | 分割済みの [`CommandLine`]([`check_command`] の結果)、足す環境変数、消す環境変数の名前の並び(`unset`。v0.10.0 の `unset_env`)、[`InterruptSignal`]、`Option<&`[`LogTarget`]`>` → (`JobExecution`, [`LogOutcome`]) | なし(起動失敗は `StartFailed` の値、ログの失敗は [`LogOutcome`] の値) |
//! | [`now`] | 現在の時刻(v0.10.0 の BR2.3) | なし → `Timestamp` | なし(範囲の外は端に寄せる) |
//! | [`open_log_file`] | 試行のログのファイルを、無いときだけ作る(v0.10.0 の BR3.1〜BR3.4) | [`LogTarget`] → ファイルとパス | 表示用のパス |
//! | [`safe_log_stem`] | ジョブ ID をファイル名に使える名前にする(v0.10.0 の BR3.2) | ジョブ ID → 文字列 | なし(純粋) |
//! | [`load_env_files`] | `env_file` の `.env` を読む(U1。BR1.7、BR3.x、BR6.4) | 定義ファイルのパス、`env_file` の記述 → `LoadedEnvFiles` | `EnvFileError` の並び(複数) |
//! | [`wait_for`] | リトライの前の待機。中断の合図を短い間隔で確かめる(BR2.13、NFR7) | 秒数、[`InterruptSignal`] → [`WaitOutcome`] | なし |
//! | [`check_command`] | command を、起動せずに検査する(BR1.1) | [`Command`](文字列または配列) → [`CommandLine`] | [`CommandError`] |
//! | [`read_state`] | ステートを読む(W2。BR2.1〜BR2.3) | 定義ファイルのパス → `Option<PipelineState>`(`None` が空) | [`StateError`] |
//! | [`read_state_raw`] | ステートを、完了のものも含めて読む(`watch` と `graph --state` 用。FR3.1) | 定義ファイルのパス → `Option<PipelineState>`(`None` はファイルなし・長さ 0) | [`StateError`] |
//! | [`write_state`] | ステートを、原子的に書く(W3。BR2.4)。耐久性の指定つき | 定義ファイルのパス、ステート、[`Durability`] | [`StateError`](`writeFailed`) |
//! | [`reset_state`] | ステートを、検査を迂回して消す(W5。BR2.6) | 定義ファイルのパス | [`StateError`](`removeFailed`) |
//! | [`reset_all_states`] | その定義のステートを、run ID の有無によらず、すべて消す(FR5) | 定義ファイルのパス → [`ResetAllReport`] | [`StateError`] |
//! | [`create_file`] | ファイルを排他的に作る、または置き換える(`init`。BR1.2、BR1.3、BR1.6) | パス、内容、`force` → [`CreateOutcome`] | [`CreateError`] |
//!
//! ステートの照合(`check_consistency`)と更新(`initial_state`、`apply_*`)は conductor-core の関数を使う
//! (`consistency` と `flow::record`。`--dry-run` の模擬と共に使う)。
//!
//! `read_state`・`read_state_raw`・`write_state`・`reset_state` は、定義ファイルのパスに加えて run ID(`Option<&RunId>`)を取る。
//! run ID があるときのステートは `<定義ファイル名>.<run-id>.state` で、中身の `run_id` と照合する。
//!
//! ファイルに触れるのは、`read_state`、`read_state_raw`、`write_state`、`reset_state`、
//! `reset_all_states`、`load_env_files`、`create_file`、`open_log_file`(と、ログの指定があるときの
//! `run_job`)だけ。プロセスに触れるのは、`run_job` だけ。時計に触れるのは、`now` だけ。それ以外は、引数だけから結果を決める純粋な処理で、ファイルにもプロセスにも
//! 環境変数にも触れない。
//!
//! # モジュールの構成(logical-components との対応)
//!
//! | モジュール | 部品 | 責務 |
//! |------------|------|------|
//! | `types` | 型(Types) | [`CommandLine`]、[`InterruptSignal`]、[`StateLocation`]、[`Durability`] |
//! | `error` | 表示用の組み立て(Describe) | kind ごとの固定の文と、ステートファイルのパスだけでエラーを表示する |
//! | `split` | 分け方(Split) | BR1.1 の規則を、文字を 1 つずつ読む状態機械で実装する(再帰なし) |
//! | `runner` | 起動(Spawn)、待ちのくり返し(WaitLoop) | 直接起動し、`try_wait` のくり返し(1ms から 10ms)で待つ。回収はここだけ。ログの指定があれば、出力を端末とファイルへ複製する |
//! | `platform` | 送信(platform::signal)ほか | OS 固有のコードは、ここだけ。`unsafe` は、UNIX 系の `kill` の呼び出し 1 か所だけ |
//! | `bounded_read` | 上限つきの読み取り(BoundedRead) | 通常のファイルだけを開き、上限 + 1 バイトまで読む(形 B)。`.env` とステートが共有する |
//! | `state::reader` | ステートの読み込み(StateReader) | 読み取りは `bounded_read`。長さ 0 は空、版の確認(先)、走査、矛盾の検査 |
//! | `state::writer` | ステートの書き込み(StateWriter)、初期化(Reset) | 内容の組み立てと同期。置き換えは `temp_file::replace_via_temp`。古い一時ファイルの後始末 |
//! | `state::reset_all` | すべてのステートの初期化(ResetAll) | その定義のステート(run ID の有無によらない)を探して消す |
//! | `envfile` | `.env` の読み込み(EnvFileLoader) | `env_file` の場所の決定と読み取り。解析は `conductor-core` の `envfile` |
//! | `file_create` | ファイルの作成(RuntimeFileCreate) | `init` の出力先を、排他的に作る、または置き換える |
//! | `temp_file` | 一時ファイル(TempFile) | 一時ファイルの名前と新規作成、連番(1 つ)、親のディレクトリの扱い、一時ファイルに書いて置き換える処理(`replace_via_temp`)。`state`・`file_create`・`platform` が共有する |
//! | `wait` | 待機(Wait) | リトライの前の待機と、中断の合図の確認 |
//! | `clock` | 時計(RuntimeClock) | 現在の時刻(v0.10.0 の BR2.3) |
//! | `log` | ログのファイル(RuntimeLog) | 安全な名前、衝突の番号、ディレクトリの作成、0600(v0.10.0 の BR3.x) |
//! | `src/bin/conductor-test-helper.rs` | テスト専用の補助の実行ファイル | 引数・環境変数・終了コード・シグナルの受信を出力する。配布には使わない |
//!
//! # 呼び出しの順序(契約 C2 の順序の続き)
//!
//! 1. [`read_state`] でステートを読む(失敗なら、ジョブを実行せず終わる)。ステートがあれば、
//!    `conductor_core::check_consistency` で定義と照合する。空なら `conductor_core::initial_state` で作る。
//! 2. ジョブごとに、`decide_from_state`(conductor-core)の判断に従う。
//!    - `Reject`: ジョブを実行せず、ステートも変えない。
//!    - `Skip`: `conductor_core::apply_skip` → [`write_state`]([`Durability::Durable`])。
//!    - `Run`: 試行ごとに、[`now`] で取った時刻で `conductor_core::start_attempt`(試行を足す)→
//!      起動の**前**に [`write_state`]([`Durability::Fast`])→ [`run_job`](足す環境変数、消す名前、
//!      `log_dir` があれば [`LogTarget`])→ 試行の後に [`now`] で取った時刻で `conductor_core::end_attempt`
//!      (試行を閉じる)。ステートが空だったときは、初期ステート(`conductor_core::initial_state`)に最初の
//!      試行を足した書き込みが、初期化の書き込みを兼ねる(書けないディレクトリを、ジョブの前に検知する。
//!      v0.10.0 以降)。そのあと、`classify_and_decide`(conductor-core)→ `conductor_core::apply_outcome` →
//!      [`write_state`]([`Durability::Durable`])。
//!      リトライのときは、試行を開いたまま(終わりの時刻はメモリに持ち、書かずに)[`wait_for`] で待ち、次の
//!      試行の始めに閉じてから新しい試行を足す。起動の前に中断の合図が立っていた試行は
//!      `conductor_core::cancel_attempt` で取り除き、待ちの中断では持っていた時刻で閉じてから
//!      `Durable` の書き込みに入る(呼び出すのは conductor-cli)。
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
//! - ジョブの強制終了(SIGKILL)、時間の上限、自動の再試行(リトライは cli が `[job.retry]` に従い、
//!   [`wait_for`] で待って [`run_job`] を呼び直す)、ステートの場所の指定、ファイルのロック
//!   (いずれも、この版では提供しない)。

#![deny(unsafe_code)]

mod bounded_read;
mod clock;
mod envfile;
mod error;
mod file_create;
mod log;
mod platform;
mod runner;
mod split;
mod state;
mod temp_file;
mod types;
mod wait;

pub use clock::now;
pub use envfile::load_env_files;
pub use error::{CommandError, CommandErrorKind, StateError, StateErrorKind};
pub use file_create::{CreateError, CreateOutcome, create_file};
pub use log::{LogOutcome, LogTarget, open_log_file, safe_log_stem};
pub use runner::run_job;
pub use split::check_command;
pub use state::reader::{read_state, read_state_raw};
pub use state::reset_all::{ResetAllReport, reset_all_states};
pub use state::writer::{reset_state, write_state};
pub use types::{CommandLine, Durability, InterruptSignal, StateLocation};
pub use wait::{WaitOutcome, wait_for};
