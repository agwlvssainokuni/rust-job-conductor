# 変える場所の一覧(Inventory): cli の本番の小さなモジュールの統合

HEAD e124919(merge-cli-modules の意図の把握のコミット。版 0.9.8)の時点で、実物を読んで作った一覧。リポジトリのコードは変えていない。行番号はこの時点のもの。初期の記述と実物が違ったところは「(記述との違い)」と書く。設計の要点、テストの変更の一覧(T…)、件数の見込み、コミットの並び、決めてほしい点(D…)は `functional-spec.md`。

方針(ユーザーの判断。前の intent の判断と、この intent の Q1〜Q5): **テストのコードの都合で本番のコードを曲げない**。処理は変えず、変えるのは置き場所(ファイル)と可視性と `use` と、それに伴う説明のコメントだけ。利用者から見える動き(出力、終了コード、ステートファイル、`--dry-run` の出力、シグナルの扱い)は変えない。版は上げない。テストは `use` のパスの書き換えだけで、名前・確かめの行・期待の値は変えない(★ なし)。

決めてほしい点(`functional-spec.md` §10 の D1〜D7)は、ユーザーが決めた(すべて推奨のとおり。2026-11-25)。D1: `run_job_with_retry` は非公開の `fn`。D2: `escape_name`・`escape_full` は `pub(crate) fn`。D3: `pub(crate) use … spawn_watcher as start_signal_watcher;` で名前を変えない。D4: `run_job_with_retry` は `run_step` の直後。D5: エスケープの 2 関数は `push_line` の直前。D6: `run_convert` の後に `run_graph`・`read_overlay`・`run_init`、元のモジュールの説明は各関数の説明へ。D7: 仕様書に版の履歴の注記を足さない。この文書の「(D1)」〜「(D7)」は、その決定のとおりの形を書いている(写しの木にも同じ形を当てた)。T 表(T1)は、設計の承認で認めてもらう(承認待ち)。

範囲の外(初期の記述): core の小さなモジュール(`duration.rs`、`edges.rs`)、テストの統合、`platform` のファイルの統合。

## 0. 基準と、設計の検証

### 0.1 基準(`cargo test --workspace --locked`、macOS、rustc 1.100.0、debug、HEAD e124919)

| 項目 | 値 |
|---|---|
| テスト | **1,149 通過、1 無視、0 失敗**(core 381、runtime 197、converter 115 + 1 無視、cli 456。cli は単体 162、統合 294) |
| テスト名の一覧 | `cargo test --workspace --locked -- --list` の `: test` の行が 1,150 行(無視の 1 件を含む) |
| `cargo doc --workspace --no-deps --locked` の警告 | 5 件(core 1、runtime 4。この intent と無関係) |
| cli の `nfr_source` | 6 件 |
| 対象のファイルの行数 | `retry.rs` 93、`step.rs` 251、`report/escape.rs` 34、`report/mod.rs` 643、`graph.rs` 78、`init.rs` 67、`aux_commands.rs` 118、`platform/mod.rs` 52 |

### 0.2 設計の検証(写しの木での試し)

T 表と §1〜§5 を検索だけに頼らないために、HEAD e124919 の写し(`git archive HEAD` を `target/merge-cli/work` に展開。ビルドは `target/merge-cli/tgt`)に、この一覧の全項目(コードと仕様書 rust の S1〜S24)を推奨の形で当て、次を確かめた。写しは確かめたあと消した(`target/merge-cli` ごと)。リポジトリのコードは変えていない。

- 写しに当てる前に、同じ写しで基準を取り直した: 1,149 通過、1 無視、テスト名 1,150 行、`cargo doc` の警告 5 件(§0.1 と同じ)。
- `cargo fmt --all -- --check`: 1 か所だけ整え直しが要った(`step.rs` の `use conductor_core::{…}` の折り返し。名前を 4 つ足したため)。`cargo fmt --all` の後は通る。実装でも、各コミットで `cargo fmt --all` してから確かめる。
- `cargo clippy --workspace --all-targets --locked -- -D warnings` が、macOS と `--target x86_64-pc-windows-msvc` の両方で通る。本番の形 `cargo clippy -p conductor-cli --lib --bins --locked -- -D warnings` も、macOS と Windows の対象の両方で通る(Windows の `platform/windows.rs` の再公開がコンパイルされることを、ここで確かめた)。
- `cargo test --workspace --locked`: **1,149 通過、1 無視、0 失敗**(基準と同じ)。
- テスト名の一覧(パスつき、並べ替えて比べた): **差 0**。動かすテストは無く、パスも変わらない(T1 は `use` の 1 行だけ)。
- cli の `nfr_source` 6 件が通る(検査のコードは変えていない)。
- `cargo doc --workspace --no-deps --locked` の警告: 基準と同じ 5 件(同じ文)。あわせて `cargo doc -p conductor-cli --no-deps --document-private-items` で、足した説明のリンク(`[`run_step`]`、`[`run_graph`]`、`[`run_init`]`)に警告が出ないことを確かめた。
- 行カバレッジ(`cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --summary-only`): 全体の行 **98.06%**(80% 以上)。`step.rs` 98.79%、`aux_commands.rs` 98.06%、`report/mod.rs` 100%、`platform/mod.rs` 100%。
- §7 の検索(無くなるファイル、無くなったモジュールのパス、仕様書の古い名前): すべて 0 件。凍結した仕様書(go ほか)と言語に依らない仕様書は HEAD と同じ。
- 直した後の行数: `step.rs` 321、`report/mod.rs` 657、`aux_commands.rs` 222、`platform/mod.rs` 38。
- 使ったツールは rustc・cargo 1.100.0(MSRV と同じ版)。
- (検証の限り)写しは 4 項目をすべて当てた最後の形だけを確かめた。各項目は互いに別のファイル・別の名前を扱う(重なるのは `lib.rs` のモジュールの表の 1 行と、仕様書 02 章の表だけ)ので、途中のコミットの形も通る見込み。実装では各コミットで確かめる(`functional-spec.md` §8)。

---

## 1. `retry.rs` を `step.rs` へ(項目 1。Q2: A)

### 1.1 今の形

| 場所(HEAD) | 中身 |
|---|---|
| `crates/cli/src/retry.rs:15-20` | モジュールの説明(リトライを含む実行。規則の番号つき) |
| `crates/cli/src/retry.rs:22-28` | `use`(`conductor_core::{EnvContext, JobDefinition, JobExecution, RetryDecision, merge_env, retry_decision}`、`conductor_runtime::{CommandLine, InterruptSignal, WaitOutcome}`、`crate::ctx::{Ctx, Session}`、`crate::report::DisplayEvent`) |
| `crates/cli/src/retry.rs:30-93` | `pub(crate) fn run_job_with_retry(ctx, session, job, line) -> JobExecution` と説明(規則の番号つき) |
| `crates/cli/src/step.rs:43` | `use crate::retry::run_job_with_retry;`(呼ぶのは `run_step` の :143 の 1 か所だけ) |
| `crates/cli/src/lib.rs:53` | モジュールの表の「流れ」の行(`retry` と「リトライを含む実行(`retry`)」) |
| `crates/cli/src/lib.rs:102` | `mod retry;` |
| `crates/cli/src/run.rs:18` | モジュールの説明の `retry::run_job_with_retry` |

- `run_job_with_retry` を呼ぶのは `step.rs` だけ(`git grep -n run_job_with_retry -- crates` で、ほかは `pipeline.rs:105` の説明のコメントだけ。パスを含まないので変えない)。
- テストは `run_job_with_retry` を直接呼ばない(`flow_retry.rs` などの統合テストが公開の入口から確かめる)。
- (記述との違い)`retry.rs` のモジュールの説明は「`run`(個別実行)と `pipeline`(一括実行)の両方が呼ぶ」と書くが、実際に呼ぶのは `step.rs` の `run_step`(`run` と `pipeline` は `run_step` を呼ぶ)。まとめるときに「`run_step` が呼ぶ」に直す。

### 1.2 まとめた後

- `step.rs` の `run_step` の直後(`remove_if_requested` の前)に、`run_job_with_retry` を本体のまま置く(D4)。説明は、元のモジュールの説明(規則の番号を含む)を関数の説明の先頭の段落にし、元の関数の説明をその後に続ける。規則の番号はすべて残す。
- 可視性: `pub(crate) fn` → `fn`(非公開。使うのは同じモジュールの `run_step` だけ。D1)。
- `use`: `conductor_core` に `EnvContext`、`RetryDecision`、`merge_env`、`retry_decision` を、`conductor_runtime` に `InterruptSignal`、`WaitOutcome` を足す(`JobDefinition`、`JobExecution`、`CommandLine`、`Ctx`、`Session`、`DisplayEvent` は既にある)。`use crate::retry::run_job_with_retry;` を消す。名前の衝突は無い(`step.rs` の関数は `record_skip`、`run_step`、`remove_if_requested`、`report_execution`、`result_detail`)。
- `step.rs` のモジュールの説明の表に 1 行「`run_job_with_retry`(非公開) | `run_step` のリトライつきの実行。実行 → 判断 → 知らせ → 待機 → 繰り返し」を、`report_execution` の行の前に足す。
- `lib.rs`: `mod retry;` を消す。モジュールの表の「流れ」の行から `retry` を消し、「1 つのジョブの共通の手順(`step`)、リトライを含む実行(`retry`)」を「1 つのジョブの共通の手順とリトライを含む実行(`step`)」にする。
- `run.rs:18`: `retry::run_job_with_retry` → `` `step` の `run_job_with_retry` ``。
- `retry.rs` を `git rm` する。

## 2. `report/escape.rs` を `report/mod.rs` へ(項目 2。Q3: A)

### 2.1 今の形

| 場所(HEAD) | 中身 |
|---|---|
| `crates/cli/src/report/escape.rs:14-18` | モジュールの説明(本体は core の `escape_display`、converter は写しを持つ、`src/tests/escape_table.rs` と converter の単体テストの同じ表で揃える。規則の番号つき) |
| `crates/cli/src/report/escape.rs:20-34` | `use conductor_core::escape_display;`、`const MAX_DISPLAY_CHARS`(非公開)、`pub fn escape_name`、`pub fn escape_full` |
| `crates/cli/src/report/mod.rs:23-25` | モジュールの説明の「`escape` を通す」 |
| `crates/cli/src/report/mod.rs:28` | `pub mod escape;` |
| `crates/cli/src/report/mod.rs:50` | `use escape::{escape_full, escape_name};` |
| `crates/cli/src/report/plan_text.rs:26` | `use super::escape::escape_full;` |
| `crates/cli/src/report/watch.rs:22` | `use super::escape::escape_name;` |
| `crates/cli/src/report/graph_text.rs:29` | `use super::escape::escape_full;` |
| `crates/cli/src/tests/escape_table.rs:20` | `use crate::report::escape::{escape_full, escape_name};`(単体テスト。T1) |

- 使うのは `report` の 4 ファイル(`mod.rs`、`plan_text.rs`、`watch.rs`、`graph_text.rs`)と、単体テスト `escape_table.rs` だけ(`git grep -n "escape::\|escape_full\|escape_name" -- crates/cli`)。`graph_mermaid.rs` は自前の `escape_label` を使い、`escape` を使わない。
- 今の実効の可視性: `report` は `lib.rs` で非公開(`mod report;`)なので、`pub mod escape` の中の `pub fn` は、実効では crate の中。
- 名前の衝突: `report/mod.rs` に `MAX_DISPLAY_CHARS`、`escape_name`、`escape_full`、`escape_display` の名前は無い。
- `push_line`(`report/mod.rs:551`。`pub(crate) fn`)が、同じく下位のモジュール(graph・計画・watch の整形)が共有する補助で、`use super::push_line;` で読まれている。

### 2.2 まとめた後

- `report/mod.rs` の `push_line` の直前に、`MAX_DISPLAY_CHARS`(非公開のまま)、`escape_name`、`escape_full` を、本体と各説明のまま置く(D5)。
- 可視性: `pub fn` → `pub(crate) fn`(D2)。今の実効の範囲(crate の中)と同じで、広げも狭めもしない。並びの `push_line` と同じ書き方。
- `use`: `pub mod escape;` と `use escape::{escape_full, escape_name};` を消し、`conductor_core` の `use` に `escape_display` を足す。
- 下位のモジュールの `use super::escape::escape_full;`(`plan_text.rs`、`graph_text.rs`)を `use super::escape_full;` に、`use super::escape::escape_name;`(`watch.rs`)を `use super::escape_name;` にする(`use super::push_line;` と同じ形)。
- 単体テスト `escape_table.rs:20` を `use crate::report::{escape_full, escape_name};` にする(T1。テストの変更はこの 1 行だけ)。
- `report/mod.rs` のモジュールの説明の箇条(:23-25)を、「エスケープ(Escape。元の規則の番号)を通す」とし、元の `escape.rs` のモジュールの説明(本体は core の `escape_display`、ここは薄い包み、converter の写しと `src/tests/escape_table.rs` の同じ表)を、この箇条に続けて写す。2 つの関数がこのモジュールにあり、下位のモジュールも使うことを書く。
- `report/escape.rs` を `git rm` する。

## 3. `graph.rs` と `init.rs` を `aux_commands.rs` へ(項目 3。Q4: A)

### 3.1 今の形

| 場所(HEAD) | 中身 |
|---|---|
| `crates/cli/src/aux_commands.rs:15-17` | モジュールの説明「補助のコマンド(…流れの番号と規則の番号…): `watch`、`reset`、`validate`、`convert`。シグナルの準備は行わない(…)」 |
| `crates/cli/src/aux_commands.rs:19-25` | `use`(`std::path::Path`、`crate::ctx::{Ctx, definition_path}`、`crate::exit::{EXIT_CONVERSION, EXIT_OK}`、`crate::report::{DisplayEvent, StateContext}`、`crate::run_id::aux_run_id`、`crate::types::{CliCommand, UsageError, UsageErrorKind}`) |
| `crates/cli/src/aux_commands.rs:27-118` | `run_watch`、`run_reset`、`run_validate`、`run_convert`(すべて `pub(crate)`) |
| `crates/cli/src/graph.rs:15-19` | モジュールの説明(`graph` の流れ。流れの番号と規則の番号つき。シグナルの準備をしない) |
| `crates/cli/src/graph.rs:21-32` | `use`(`std::path::Path`、`conductor_core::PipelineDefinition`、`conductor_core::graph::{StateOverlay, build_graph, layout_graph, overlay_from_state, pending_overlay}`、`crate::ctx::{Ctx, StateRead}`、`crate::report::DisplayEvent`、`crate::report::graph_mermaid::render_mermaid`、`crate::report::graph_text::render_text`、`crate::types::{CliCommand, GraphFormat}`) |
| `crates/cli/src/graph.rs:34-78` | `pub(crate) fn run_graph`(説明「`conductor graph` を実行し、終了コードを返す。」)と非公開の `read_overlay`(規則の番号つきの説明) |
| `crates/cli/src/init.rs:15-18` | モジュールの説明(`init` の流れ。流れの番号と規則の番号つき。シグナルの準備をしない) |
| `crates/cli/src/init.rs:20-28` | `use`(`std::path::Path`、`conductor_runtime::{CreateError, CreateOutcome}`、`crate::ctx::Ctx`、`crate::exit::{EXIT_EXISTS, EXIT_OK, EXIT_STATE}`、`crate::report::DisplayEvent`、`crate::report::init_template::render_template`、`crate::types::{CliCommand, InitTarget}`) |
| `crates/cli/src/init.rs:30-67` | `pub(crate) fn run_init`(説明「`conductor init` を実行し、終了コードを返す。」) |
| `crates/cli/src/lib.rs:53` | モジュールの表の「流れ」の行(`graph`、`init`、「`graph`(図)、`init`(雛形)」) |
| `crates/cli/src/lib.rs:96-97` | `mod graph;`、`mod init;` |
| `crates/cli/src/lib.rs:187-188` | `CommandKind::Graph => graph::run_graph(…)`、`CommandKind::Init => init::run_init(…)` |

- 名前の衝突: `aux_commands.rs` に `run_graph`、`read_overlay`、`run_init` の名前は無い。`use` は和集合で、重なる `Path`・`Ctx`・`DisplayEvent`・`CliCommand`・`EXIT_OK` は 1 つにする。
- 3 つとも、シグナルの準備をしない(`Ctx::prepare_session` を呼ばない)。`graph` は `ctx.check_definition_and_env`(`validate` と共有)、`init` は `ctx.runtime.create_file`。
- **規則の番号の置き場所(記述との違い・足した点)**: `graph.rs`・`init.rs` の規則の番号は、主にモジュールの説明(`//!`)にあり、`run_graph`・`run_init` の説明は 1 行だけ。ファイルを消すとモジュールの説明が無くなるので、モジュールの説明の本文(流れの番号・規則の番号を含む)を、各関数の説明へ移す。

### 3.2 まとめた後

- `aux_commands.rs` の `run_convert` の後に、`run_graph`、`read_overlay`、`run_init` の順で、本体のまま置く(D6。`CommandKind` と `lib.rs` の `match` の並び `… Convert, Graph, Init` と同じ順)。
- 説明: `run_graph` の説明は「`conductor graph` を実行し、終了コードを返す(<`graph.rs` のモジュールの説明の括弧の中>)。」の 1 行のあとに、`graph.rs` のモジュールの説明の本文を段落として続ける。`run_init` も同じ(`init.rs` のモジュールの説明から)。`read_overlay` の説明は元のまま。各関数の説明の規則の番号は、すべて残る。
- `aux_commands.rs` のモジュールの説明(:15-17):
  - 1 行目の括弧の中に、`graph.rs`・`init.rs` のモジュールの説明の括弧の中の流れの番号を足し、コマンドの並びに `graph`、`init` を足す。
  - 「シグナルの準備は行わない(…)」は残し、「`graph` と `init` の規則の番号は [`run_graph`]・[`run_init`] の説明にある」と、「シグナルの準備をする `run`・`pipeline`・`--dry-run` は 1 コマンド 1 ファイル(`run.rs`、`pipeline.rs`、`dry_run.rs`)」を足す(分け方の基準を書き残す。SM3)。
- `use`(直した後): `std::path::Path`、`conductor_core::PipelineDefinition`、`conductor_core::graph::{…}`、`conductor_runtime::{CreateError, CreateOutcome}`、`crate::ctx::{Ctx, StateRead, definition_path}`、`crate::exit::{EXIT_CONVERSION, EXIT_EXISTS, EXIT_OK, EXIT_STATE}`、`crate::report::graph_mermaid::render_mermaid`、`crate::report::graph_text::render_text`、`crate::report::init_template::render_template`、`crate::report::{DisplayEvent, StateContext}`、`crate::run_id::aux_run_id`、`crate::types::{CliCommand, GraphFormat, InitTarget, UsageError, UsageErrorKind}`。
- 可視性: `run_graph`・`run_init` は `pub(crate)` のまま(`lib.rs` が呼ぶ)。`read_overlay` は非公開のまま。
- `lib.rs`: `mod graph;`、`mod init;` を消す。`CommandKind::Graph => aux_commands::run_graph(…)`、`CommandKind::Init => aux_commands::run_init(…)` にする。モジュールの表の「流れ」の行の、モジュールの列から `graph`、`init` を消し、責務の列の「`graph`(図)、`init`(雛形)、<補助の流れの番号>(watch・reset・validate・convert)」を「シグナルの準備をしない補助のコマンド(`aux_commands`。<補助の流れの番号>(watch・reset・validate・convert)、`graph`(図)、`init`(雛形))」にする。
- `graph.rs`、`init.rs` を `git rm` する。
- 直した後の `aux_commands.rs` は 222 行(写しで確かめた)。

## 4. `platform/mod.rs` の二重の中継(項目 4。Q5: A)

### 4.1 今の形

| 場所(HEAD) | 中身 |
|---|---|
| `crates/cli/src/platform/mod.rs:15-20` | モジュールの説明(OS 固有のコード、シグナルの監視、色) |
| `crates/cli/src/platform/mod.rs:22-26` | `use std::sync::Arc;`、`use conductor_runtime::InterruptSignal;`、`use crate::deps::SignalError;`(中継の関数のためだけ) |
| `crates/cli/src/platform/mod.rs:28-31` | `#[cfg(unix)] mod unix;`、`#[cfg(windows)] mod windows;` |
| `crates/cli/src/platform/mod.rs:33-37` | `pub(crate) fn start_signal_watcher(signal) { spawn_watcher(signal) }`(説明: プロセスで 1 回だけ呼ぶ、2 回目は契約の外) |
| `crates/cli/src/platform/mod.rs:39-47` | `#[cfg(unix)] fn spawn_watcher { unix::spawn_watcher(signal) }`、`#[cfg(windows)] fn spawn_watcher { windows::spawn_watcher(signal) }` |
| `crates/cli/src/platform/mod.rs:49-52` | `pub(crate) fn color_supported() -> bool { cfg!(unix) }` |
| `crates/cli/src/platform/unix.rs:29` | `pub(super) fn spawn_watcher(…)` |
| `crates/cli/src/platform/windows.rs:27` | `pub(super) fn spawn_watcher(…)` |
| `crates/cli/src/deps.rs:288` | `platform::start_signal_watcher(Arc::clone(signal))`(呼ぶのはここだけ) |
| `crates/runtime/src/platform/mod.rs:22-30` | 手本: `#[cfg(unix)] mod unix;`、`#[cfg(unix)] pub(crate) use unix::{…};`、Windows も同じ。OS ごとの関数は `pub(crate) fn` |

- **(記述との違い・足した点)** 再公開(`pub(crate) use`)は、元の項目の可視性を超えられない(`pub(super)` の関数を `pub(crate)` で再公開するとコンパイルが止まる)。runtime の `unix.rs`・`windows.rs` の関数が `pub(crate) fn` なのはこのため。cli でも `unix.rs`・`windows.rs` の `spawn_watcher` を `pub(super)` → `pub(crate)` にする(可視性だけ。名前と本体は変えない)。

### 4.2 まとめた後(D3。決定: 推奨のとおり。ユーザー、2026-11-25)

```rust
//! (今の説明。シグナルの監視の箇条の後ろに、次の 3 行を足す)
//!   入口は `start_signal_watcher`(OS ごとの `spawn_watcher` を、その名前で再公開したもの)。
//!   プロセスで 1 回だけ呼ぶ(本番は `Ctx::prepare_session` の 1 か所)。2 回呼んだときの動きは
//!   契約の外(UNIX 系は監視が 2 つになり、Windows は `Failed`)。

#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub(crate) use unix::spawn_watcher as start_signal_watcher;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub(crate) use windows::spawn_watcher as start_signal_watcher;

/// 端末に、色(ANSI の制御文字)を出してよい OS か。
pub(crate) fn color_supported() -> bool {
    cfg!(unix)
}
```

- 消すもの: 中継の関数 3 つ(`start_signal_watcher` の本体と、OS ごとの `spawn_watcher` の 2 つ)と、それだけが使う `use` 3 つ。中継の関数の説明(1 回だけ呼ぶ、2 回目は契約の外)は、モジュールの説明へ移す(再公開の `use` には説明を付けない)。
- `deps.rs:288` の `platform::start_signal_watcher(…)` は変えない(同じ名前で再公開する)。
- `platform` のディレクトリと `mod.rs`・`unix.rs`・`windows.rs` の名前は変えない。`platform/mod.rs` は 52 行 → 38 行。
- cli の `nfr_source.rs` の検査: `#[cfg(unix)]`/`#[cfg(windows)]` は `platform` の中だけ(パスの要素 `platform` で外す)。`unsafe`・`low_level` は足さない。`#[path`・`include!` も足さない。写しで 6 件が通る。
- Windows の対象のコンパイル: `windows.rs` の `spawn_watcher` を `pub(crate)` にし、`pub(crate) use windows::spawn_watcher as start_signal_watcher;` がコンパイルされることを、`--target x86_64-pc-windows-msvc` の clippy で確かめた(写し)。実行は CI の `test`(windows)で確かめる。

## 5. 仕様書

### 5.1 `docs/spec/rust`

| # | 場所(HEAD) | 今 | 直した後 | 項目 |
|---|---|---|---|---|
| S1 | `02:216`(`pipeline` のシーケンスの図) | `retry::run_job_with_retry(job, command)` | `step::run_job_with_retry(job, command)`(右のコメントの桁を合わせる) | 1 |
| S2 | `02:370`(`step.rs` の行) | 「…実行のあとの表示(`report_execution`、`result_detail`)。`run` と `pipeline` が呼ぶ。…」 | 「…実行のあとの表示(`report_execution`、`result_detail`)、`run_step` が呼ぶリトライを含む実行 `run_job_with_retry`(非公開。試行ごとに `merge_env`。どの試行も、検査で分割した同じコマンドを渡す)。`run` と `pipeline` が呼ぶ。…」 | 1 |
| S3 | `02:373` | `retry.rs` の行 | 消す | 1 |
| S4 | `05:5`(実装の所在) | `crates/cli/src/{ctx,run,pipeline,retry}.rs` | `crates/cli/src/{ctx,run,pipeline,step}.rs` | 1 |
| S5 | `05:634`(環境変数の注入) | 流れが `crates/cli/src/{ctx,retry}.rs` | `crates/cli/src/{ctx,step}.rs` | 1 |
| S6 | `07:4`(根拠) | `crates/cli/src/{platform/*,ctx,pipeline,run,retry,exit,panic_hook,main}.rs` | `retry` を `step` に | 1 |
| S7 | `07:25`(部品の表の Orchestrator) | `ctx/run/pipeline/retry.rs` | `ctx/run/pipeline/step.rs` | 1 |
| S8 | `12:85`(段 8 の作るもの) | 「…、`retry.rs`、`run.rs`、…」 | 「…、`step.rs`(1 つのジョブの共通の手順と、リトライを含む実行 `run_job_with_retry`)、`run.rs`、…」 | 1 |
| S9 | `12:107`(環境変数の段の作るもの) | `retry.rs`(試行ごとに `merge_env`、…) | `step.rs` の `run_job_with_retry`(試行ごとに `merge_env`、…) | 1 |
| S10 | `02:164`(cli の crate の中の関数の表) | `escape_name` / `escape_full`(`report::escape`。crate の中) | (`report`。crate の中) | 2 |
| S11 | `02:381`(`report/mod.rs` の行) | 「…、`push_line`(graph・計画・watch が共有)」 | 後ろに「、入力由来の文字列のエスケープ(`escape_name`、`escape_full`。conductor-core の `escape_display` の包み。計画・watch・図の整形も使う)」を足す | 2 |
| S12 | `02:387` | `report/escape.rs` の行 | 消す | 2 |
| S13 | `08:3`(実装) | `crates/cli/src/report/{mod,messages,watch,escape}.rs` | `report/{mod,messages,watch}.rs` | 2 |
| S14 | `08:564`(9 章の見出し) | 「## 9. 入力に由来する文字列のエスケープ(`escape.rs`)」 | 「(`report/mod.rs` の `escape_name`・`escape_full`)」 | 2 |
| S15 | `08:566` | 「(本体は conductor-core。`escape.rs` は包み)」 | 「(本体は conductor-core。`report/mod.rs` の `escape_name`・`escape_full` は包み)」 | 2 |
| S16 | `08:605` | 「cli の `report::escape` はそれを包む」 | 「cli の `report`(`report/mod.rs` の `escape_name`・`escape_full`)はそれを包む」 | 2 |
| S17 | `02:374`(`aux_commands.rs` の行) | `watch`・`reset`・`validate`・`convert` | 「シグナルの準備をしない補助のコマンド: `watch`・`reset`・`validate`・`convert`、`graph`(`run_graph`。v0.9.0 以降)、`init`(`run_init`。同)」 | 3 |
| S18 | `02:375`、`02:377` | `graph.rs`(v0.9.0 以降)の行、`init.rs`(同)の行 | 消す | 3 |
| S19 | `02:376`(`dry_run.rs` の行) | `dry_run.rs`(同) | `dry_run.rs`(v0.9.0 以降)(「同」が指す前の行が無くなるため) | 3 |
| S20 | `04:3`(実装) | `crates/cli/src/{main,lib,args,types,ctx,run,pipeline,aux_commands,graph,dry_run,init}.rs` | `graph`、`init` を消す | 3 |
| S21 | `04:203`、`04:218` | 実装は `crates/cli/src/graph.rs` の `run_graph`、`crates/cli/src/init.rs` の `run_init` | `crates/cli/src/aux_commands.rs` の `run_graph`、同じく `run_init` | 3 |
| S22 | `05:5`(実装の所在の後半) | それぞれの流れは `crates/cli/src/{graph,dry_run,init}.rs` | `crates/cli/src/aux_commands.rs`(`graph` と `init`)と `crates/cli/src/dry_run.rs` | 3 |
| S23 | `12:117`、`12:137` | `graph.rs`(`run_graph`)、`init.rs`(`run_init`) | `aux_commands.rs` の `run_graph`、`aux_commands.rs` の `run_init` | 3 |
| S24 | `02:380`(cli の `platform` の行) | 「…、`color_supported`。登録はプロセスで 1 回」 | 後ろに「。入口の `start_signal_watcher` は、`mod.rs` が OS ごとの `spawn_watcher` を `#[cfg(unix)]`/`#[cfg(windows)]` の `pub(crate) use … as start_signal_watcher` で再公開したもの(runtime の `platform` と同じ再公開の形)」を足す | 4 |

- 変えない: `02:310`・`02:319`(core の `walk/retry.rs`・`flow/retry.rs`。別のファイル)、`03:703`・`11:204`(core のテスト `definition_retry.rs`)、`05:277`(「残りの 3 つは cli の `step.rs`」。`record_skip`・`run_step`・`remove_if_requested` のことで、そのまま正しい)、`05:294`・`05:514`(関数の名前 `run_job_with_retry` は変わらない)、`10:3`(`aux_commands` を含み、`graph`・`init`・`retry` を含まない)、`11:263`(統合テストのファイル名)、`08` の `escape_name`・`escape_full` の名前を使う箇所(名前は変わらない)、`02:153`(`MAX_DISPLAY_CHARS` の版の履歴。名前は残る)、`07:23-28`・`11:173` などの `platform` の記述(ディレクトリとファイル名は変わらない)。
- 版の履歴の注記(「v0.9.8 までは `retry.rs`」など)は足さない(D7。版を上げないため)。

### 5.2 言語に依らない仕様書(`docs/spec/language-agnostic`)

- **直す所は無い**。該当の検索(§6)で当たるのは、関数の名前 `run_job_with_retry`(`05:320`、`05:357`、`05:517`)と、表示の規則の `escape_name`(`11:808`)だけで、どちらも名前は変わらない。Rust のファイル名・モジュールのパス(`retry.rs`、`report::escape`、`graph.rs`、`init.rs`)は書かれていない。

### 5.3 凍結した仕様書

- `docs/spec/{go,python,typescript,csharp,powershell}` は変えない。`go/02:551`(`escape.go`)、`python/02:723`・`python/08:688`(`escape.py`)、`typescript/02:645`・`typescript/08:618`(`escape.ts`)は、それぞれの言語のファイルで、この intent と無関係。

## 6. 網羅に使った検索のコマンド

```sh
# 対象のモジュールを指す所(コード、テスト、README、CI、仕様書)
git grep -n -E "retry\.rs|crate::retry|retry::|report/escape|report::escape|escape\.rs|escape::|graph\.rs|crate::graph|graph::run_graph|init\.rs|crate::init|init::run_init|spawn_watcher|start_signal_watcher|aux_commands" -- ':!aidlc' ':!docs/spec/go' ':!docs/spec/python' ':!docs/spec/typescript' ':!docs/spec/csharp' ':!docs/spec/powershell'
git grep -n "escape::\|escape_full\|escape_name" -- crates/cli          # 使う側(report の 4 ファイルと escape_table.rs)
git grep -n run_job_with_retry -- crates                                 # 呼ぶ側(step.rs だけ)
git grep -n "platform::" -- crates/cli/src                               # deps.rs:288、report/mod.rs:284
grep -n "^pub\|^fn\|^pub(crate)\|^const\|^use" crates/cli/src/report/mod.rs crates/cli/src/aux_commands.rs crates/cli/src/step.rs   # 名前の衝突
# 仕様書の、ファイル名とモジュールのパス
grep -n -E "retry\.rs|retry::run_job|escape\.rs|report::escape|report/escape|[^_]graph\.rs|[^_]init\.rs|\{[^}]*(retry|graph|init|escape)[^}]*\}\.rs|run_job_with_retry|run_graph|run_init|spawn_watcher|start_signal_watcher|cli/src/(retry|graph|init)" docs/spec/rust/*.md docs/spec/language-agnostic/*.md
grep -n -E "(^|[^_:a-z])(graph|init|retry|escape|aux_commands)::|platform::(start|spawn|color)" docs/spec/rust/*.md docs/spec/language-agnostic/*.md
git grep -n -E "retry\.rs|escape\.rs|cli/src/graph|cli/src/init|step\.rs|aux_commands" -- README.md docs/spec/rust docs/spec/language-agnostic .github
# 件数と名前
cargo test --workspace --locked 2>&1 | grep -E "^test result"
cargo test --workspace --locked -- --list 2>/dev/null | grep ": test$" | sort
```

- README と CI(`.github/workflows`)は、対象のファイル名とモジュールのパスを指さない(検索で 0 件)。

## 7. 範囲の外で、触らないもの(確かめた)

- core の小さなモジュール(`duration.rs`、`edges.rs`)、テストのファイル(T1 の 1 行のほか)、`platform` のファイルの統合(ファイルは 3 つのまま)。
- `Cargo.toml`、`Cargo.lock`、CI の設定、README。版(0.9.8)。
- `pipeline.rs:105` の説明のコメント(`run_job_with_retry` の名前だけで、パスを含まない)。`report/graph_mermaid.rs`(`escape` を使わない)。`deps.rs`(呼び出しの名前は同じ)。
- cli の `nfr_source.rs` と、ほかの製品のコードの検査のテスト。
- 凍結した仕様書と、言語に依らない仕様書。
