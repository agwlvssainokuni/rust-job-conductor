# 機能の仕様(Functional Spec): cli の本番の小さなモジュールの統合

意図: `ideation/intent-capture/intent-statement.md`(Q1〜Q5 すべて A)。**変える場所の正確な一覧(項目ごとの今の形とまとめた後の形、名前の衝突、可視性、仕様書の S1〜S24、使った検索のコマンド)と、基準の件数、写しの木での検証は `inventory.md`**(HEAD e124919 で作った)。この文書は、方針、項目ごとの設計、**承認が要るテストの変更(§4 の T 表)**、件数の見込み、確かめ方、コミットの並び、範囲の外、決めてほしい点を書く。

決めてほしい点(§10 の D1〜D7)は、ユーザーが決めた(すべて推奨のとおり。2026-11-25)。この文書の D の箇所は、その決定のとおりの形を書いている(写しの木にも同じ形を当てた)。§4 の T 表(T1)は、ユーザーが設計の承認で認めた(2026-11-25)。

## 1. 方針

- **テストのコードの都合で本番のコードを曲げない**(ユーザーの判断)。**処理は変えない**。変えるのは、置き場所(ファイル)、可視性、`use`、それに伴う説明のコメントだけ。利用者から見える動き(出力、終了コード、ステートファイル、`--dry-run` の出力、シグナルの扱い)は変えない。版は上げない(`0.9.8` のまま)[desc]。
- 可視性は、使う範囲に合わせる。狭められるものは狭め(`run_job_with_retry` は同じモジュールだけ → 非公開)、テストのためだけに広げない(`escape_name`・`escape_full` は今の実効の範囲 crate の中のまま。D2)。
- **テストは `use` のパスの書き換えだけ**(1 行。T1)。名前・確かめの行・期待の値は変えない。消すテスト・足すテスト・名前を変えるテストは無い(★ なし)[Q1]。
- 各関数の説明の規則の番号は保つ。消すファイルのモジュールの説明(規則の番号を含む)は、まとめ先の関数の説明かモジュールの説明へ移す。
- `retry.rs` は `step.rs` へ [Q2]、`report/escape.rs` は `report/mod.rs` へ [Q3]、`graph.rs` と `init.rs` は `aux_commands.rs` へ [Q4]。`platform/mod.rs` の中継は、runtime と同じ再公開の形にする(ディレクトリとファイル名は変えない)[Q5]。
- 製品のコードの検査(cli の `nfr_source.rs`)は、テストの数と中身を変えずに通る。`cfg(unix)`/`cfg(windows)` は `platform` の中だけ、`unsafe` は足さない。
- 仕様書は `docs/spec/rust` を直す(S1〜S24)。言語に依らない仕様書は、該当が無いので直さない。凍結した仕様書は触らない。
- 小さなコミットの並び(1 項目 1 コミット。4 コミット)で、どのコミットでもビルド、`clippy -D warnings`、テストが通る [memory:M1] [memory:M4]。承認は、この設計とコードの計画の 1 回。結果は最後にまとめて見せる [Q1]。
- 新しい依存を足さない [memory:M5]。MSRV 1.100 [memory:M3]。ドキュメントコメントは日本語。

## 2. 項目ごとの設計(詳細は `inventory.md` の同じ番号)

| 項目 | 元 → まとめ先 | 可視性の変わり方 | 変わるテスト |
|------|---------------|------------------|--------------|
| 1 リトライを含む実行 | `src/retry.rs` → `src/step.rs`(`run_step` の直後) | `run_job_with_retry`: `pub(crate)` → 非公開(D1) | なし |
| 2 エスケープの包み | `src/report/escape.rs` → `src/report/mod.rs`(`push_line` の直前) | `escape_name`・`escape_full`: `pub mod escape` の中の `pub` → `pub(crate)`(実効の範囲は同じ。D2) | `src/tests/escape_table.rs` の `use` 1 行(T1) |
| 3 補助のコマンド | `src/graph.rs`、`src/init.rs` → `src/aux_commands.rs`(`run_convert` の後に `run_graph`・`read_overlay`・`run_init`) | 変わらない(`run_graph`・`run_init` は `pub(crate)`、`read_overlay` は非公開) | なし |
| 4 シグナルの監視の入口 | `src/platform/mod.rs` の中継 3 関数 → `#[cfg(unix)]`/`#[cfg(windows)]` の `pub(crate) use … as start_signal_watcher;` | `unix.rs`・`windows.rs` の `spawn_watcher`: `pub(super)` → `pub(crate)`(再公開に要る。D3) | なし |

ファイルの数: 無くなる 4(`retry.rs`、`report/escape.rs`、`graph.rs`、`init.rs`)、新しく作る 0。cli の `src/` の下の `.rs`(`src/tests/` と `bin/` を含む)は 40 → 36(cli の `nfr_source.rs` の「ソースが 10 ファイル以上」は満たす)。

### 2.1 項目 1 の形(`crates/cli/src/step.rs`)

```rust
//! (今の説明。関数の表の `report_execution` の行の前に 1 行を足す)
//! | `run_job_with_retry`(非公開) | `run_step` のリトライつきの実行。実行 → 判断 → 知らせ → 待機 → 繰り返し |

use conductor_core::{
    Classification, EnvContext, ExecutionResult, JobDefinition, JobExecution, JobOutcome, NextStep,
    PipelineState, RetryDecision, RunMode, SkipCause, apply_outcome, apply_skip,
    classify_and_decide, initial_state, merge_env, retry_decision,
};
use conductor_runtime::{CommandLine, Durability, InterruptSignal, WaitOutcome};

use crate::ctx::{Ctx, Session};
use crate::exit::outcome_exit_code;
use crate::report::{DisplayEvent, ResultDetail, StateContext};
// `use crate::retry::run_job_with_retry;` は消す

/* StepResult、SkipRecord、record_skip、run_step: 今のまま */

/// ジョブを、リトライを含めて実行する(<元の retry.rs のモジュールの説明の括弧の中>)。[`run_step`] が呼ぶ。
///
/// (元のモジュールの説明の 2 段落目以降。「`run` と `pipeline` の両方が呼ぶ」は、実際に呼ぶ
///  `run_step` に直す。規則の番号はすべて残す)
///
/// (元の関数の説明。規則の番号はすべて残す)
fn run_job_with_retry(/* 引数と本体は元のまま */) -> JobExecution { /* … */ }

/* remove_if_requested、report_execution、result_detail: 今のまま */
```

- `cargo fmt` が `use conductor_core::{…}` を上の形に折り返す(写しで確かめた)。
- `lib.rs` の `mod retry;` を消し、モジュールの表の「流れ」の行を直す(`inventory.md` §1.2)。`run.rs:18` の説明の `retry::run_job_with_retry` を `` `step` の `run_job_with_retry` `` にする。
- 完了の判定: `test ! -e crates/cli/src/retry.rs`。`grep -n "fn run_job_with_retry" crates/cli/src/step.rs` が 1 行で、先頭に `pub` が無い。

### 2.2 項目 2 の形(`crates/cli/src/report/mod.rs`)

```rust
//! (今の説明。エスケープの箇条を、元の escape.rs のモジュールの説明を合わせた形にする:
//!  「入力に由来する文字列(…)は、エスケープ(Escape。<元の番号>)を通す(…今の括弧のまま…)。
//!   2 つは、このモジュールにあり、下位のモジュール(計画・watch・図の整形)も使う。本体は
//!   conductor-core の `escape_display` で、ここは名前・パスの薄い包みだけを持つ。conductor-converter は
//!   core に依存しないため、同じ規則の写しを持ち、単体テストの `src/tests/escape_table.rs` と converter の
//!   単体テストの、同じ期待値の表で揃える。」)

// `pub mod escape;` は消す
pub mod graph_mermaid;
/* … */
use conductor_core::{ /* 今の名前 */, escape_display, summarize_errors };
// `use escape::{escape_full, escape_name};` は消す

/* … 今のまま … */

/// ジョブID・キー名などを表示するときの最大の文字数(<元の説明のまま>)。
const MAX_DISPLAY_CHARS: usize = conductor_core::MAX_KEY_DISPLAY_CHARS;

/// ジョブID・名前など(64 文字で切る)。
pub(crate) fn escape_name(input: &str) -> String {
    escape_display(input, Some(MAX_DISPLAY_CHARS))
}

/// 64 文字で切らずに、エスケープだけ(パス、図と計画のジョブID・コマンドの語など)。
pub(crate) fn escape_full(input: &str) -> String {
    escape_display(input, None)
}

/// 1 行を足す(…今のまま…)。
pub(crate) fn push_line(out: &mut String, line: &str) { /* 今のまま */ }
```

- 下位のモジュール: `plan_text.rs:26`・`graph_text.rs:29` は `use super::escape_full;`、`watch.rs:22` は `use super::escape_name;`。
- 単体テスト: `escape_table.rs:20` は `use crate::report::{escape_full, escape_name};`(T1)。
- 完了の判定: `test ! -e crates/cli/src/report/escape.rs`。`git grep -n "escape::" -- crates/cli` が 0 件。

### 2.3 項目 3 の形(`crates/cli/src/aux_commands.rs`)

```rust
//! 補助のコマンド(<今の番号>。<graph.rs と init.rs のモジュールの説明の括弧の中の流れの番号>): `watch`、`reset`、
//! `validate`、`convert`、`graph`、`init`。
//!
//! シグナルの準備は行わない(<今の括弧のまま>)。`graph` と `init` の規則の番号は、
//! [`run_graph`]・[`run_init`] の説明にある。シグナルの準備をする `run`・`pipeline`・`--dry-run` は、
//! 1 コマンド 1 ファイル(`run.rs`、`pipeline.rs`、`dry_run.rs`)。

/* use: 和集合(inventory.md §3.2) */

/* run_watch、run_reset、run_validate、run_convert: 今のまま */

/// `conductor graph` を実行し、終了コードを返す(<graph.rs のモジュールの説明の括弧の中>)。
///
/// (graph.rs のモジュールの説明の本文。規則の番号を含めてそのまま)
pub(crate) fn run_graph(ctx: &mut Ctx<'_>, cmd: &CliCommand) -> i32 { /* 元のまま */ }

/// (元の説明のまま)
fn read_overlay(/* 元のまま */) -> Result<StateOverlay, i32> { /* 元のまま */ }

/// `conductor init` を実行し、終了コードを返す(<init.rs のモジュールの説明の括弧の中>)。
///
/// (init.rs のモジュールの説明の本文。規則の番号を含めてそのまま)
pub(crate) fn run_init(ctx: &mut Ctx<'_>, cmd: &CliCommand) -> i32 { /* 元のまま */ }
```

- `lib.rs`: `mod graph;`・`mod init;` を消し、`CommandKind::Graph => aux_commands::run_graph(&mut ctx, &cmd)`、`CommandKind::Init => aux_commands::run_init(&mut ctx, &cmd)` にする。モジュールの表の「流れ」の行を直す(`inventory.md` §3.2)。
- 完了の判定: `test ! -e crates/cli/src/graph.rs && test ! -e crates/cli/src/init.rs`。`grep -c "^pub(crate) fn run_" crates/cli/src/aux_commands.rs` が 6。

### 2.4 項目 4 の形(`crates/cli/src/platform/`)

`inventory.md` §4.2 のとおり。`mod.rs` は、`#[cfg(unix)] mod unix;`・`#[cfg(unix)] pub(crate) use unix::spawn_watcher as start_signal_watcher;` と Windows の同じ 2 行、`color_supported` だけになる(runtime の `platform/mod.rs` と同じ並び: `mod` の直後に再公開)。`unix.rs:29`・`windows.rs:27` は `pub(super) fn spawn_watcher` → `pub(crate) fn spawn_watcher`(名前と本体と説明は変えない)。中継の関数の説明は、モジュールの説明へ移す。

- 完了の判定: `grep -c "^pub(crate) use .* as start_signal_watcher;" crates/cli/src/platform/mod.rs` が 2。`grep -n "fn start_signal_watcher\|fn spawn_watcher" crates/cli/src/platform/mod.rs` が 0 件。`grep -n "pub(crate) fn spawn_watcher" crates/cli/src/platform/{unix,windows}.rs` が 2 行。

## 3. 実物で確かめて初期の記述と違った点・足した点

- 項目 1: `retry.rs` のモジュールの説明の「`run` と `pipeline` の両方が呼ぶ」は、実際は `step.rs` の `run_step` が呼ぶ(まとめるときに直す)。行数は 251 + 93 → 321(`use` が 2 行増え、`use crate::retry` と著作権の見出しが減る)。
- 項目 2: 使う側は記述どおり report の 4 ファイルと、単体テスト `escape_table.rs` の 1 つ。可視性は、今の実効の範囲(crate の中)が、本番だけなら要らない広さ(下位のモジュールは親の非公開の項目を読める)で、crate の中の広さを使うのは単体テストだけ(D2)。
- 項目 3: `graph.rs`・`init.rs` の規則の番号はモジュールの説明にあり、関数の説明は 1 行だけ。ファイルを消すので、モジュールの説明を関数の説明へ移す(記述の「各関数のドキュメントコメントの規則の番号は保つ」を、この形で満たす)。
- 項目 4: 再公開には、OS ごとの関数の可視性を `pub(super)` → `pub(crate)` に広げる必要がある(runtime も `pub(crate) fn`)。初期の記述の「約 15 行減る」は、写しで 52 → 38 行(14 行減)。
- 仕様書: 言語に依らない仕様書には、ファイル名もモジュールのパスも書かれていない(直す所が無い)。仕様書 rust は 02・04・05・07・08・12 章の 24 か所(S1〜S24)。

## 4. 変えるテストの一覧(T1。ユーザーが設計の承認で認めた、2026-11-25)

Q1: A に従い、テストは `use` のパスの書き換えだけ。

### 4.0 網羅に使った検索のコマンド

全体は `inventory.md` §6。

```sh
git grep -n -E "crate::retry|retry::|report::escape|escape::|crate::graph|crate::init|graph::run_graph|init::run_init|platform::" -- crates/cli/src/tests crates/cli/tests
# → crates/cli/src/tests/escape_table.rs:20 の 1 件だけ
```

### 4.1 T 表

| ID | ファイル(HEAD) | 今 | 直した後 | テストの件数 | パスの変わり方(`--list`) |
|----|----------------|----|----------|--------------|---------------------------|
| T1 | `crates/cli/src/tests/escape_table.rs:20` | `use crate::report::escape::{escape_full, escape_name};` | `use crate::report::{escape_full, escape_name};` | 3(本体は変えない) | 変わらない(`tests::escape_table::…`) |

### 4.2 残すテスト(変えない)

- T1 の外のすべてのテスト。統合テスト(`flow_retry.rs`、`flow_graph.rs`、`flow_init.rs`、`binary_signal.rs`、`signals_order.rs`、`signals_real.rs` など)は公開の入口から確かめ、モジュールのパスを使わない。

### 4.3 消すテスト、名前を変えるテスト(★)

**なし**。

## 5. 足すテスト

なし。実装で足す必要が出たら、その数と理由を記録する。

## 6. 件数の見込み

| クレート | 単体(src) | 統合(tests/) | 合計 | 基準からの差 |
|---|---|---|---|---|
| core | 112 | 269 | 381 | 0 |
| runtime | 69 | 128 | 197 | 0 |
| converter | 96 | 19 + 1 無視 | 115 + 1 無視 | 0 |
| cli | 162 | 294 | 456 | 0 |
| 全体 | | | **1,149 通過、1 無視、0 失敗** | **0** |

テスト名の一覧(パスつき)も、前後で差 0(写しで確かめた)。各コミットで、全体と cli の件数は同じ。

## 7. 確かめ方

- 実装の前と後で、`cargo test --workspace --locked` の件数をクレートごとに記録する(基準 `inventory.md` §0.1)。合格は、失敗 0 で、全体が「**基準 1,149 − 消したテスト 0 + 足したテスト 0 = 1,149**(と無視 1)」に一致すること。
- テスト名の一覧の差: 前後で `cargo test --workspace --locked -- --list 2>/dev/null | grep ": test$" | sort` を比べ、**差が 0**(パスつきで 0)であること。
- テストの変更: `git diff <基準> -- crates/cli/src/tests crates/cli/tests` が、`escape_table.rs` の `use` の 1 行だけ(T1)。
- **完了の判定**。最後のコミットのあとで、次をすべて行う:

```sh
# 無くなるファイル(4)
for f in crates/cli/src/retry.rs crates/cli/src/report/escape.rs crates/cli/src/graph.rs crates/cli/src/init.rs; do
  test ! -e "$f" || echo "残る: $f"; done                                    # 出力なし
# 無くなったモジュールのパスの参照(コード)
git grep -n -E "crate::retry|retry::|report::escape|super::escape::|escape::\{|crate::graph::|crate::init::|graph::run_graph|init::run_init|mod (retry|graph|init|escape);" -- crates/cli   # 0 件
# まとめ先
grep -n "fn run_job_with_retry" crates/cli/src/step.rs                       # 1 行(pub なし)
grep -n "fn escape_name\|fn escape_full\|const MAX_DISPLAY_CHARS" crates/cli/src/report/mod.rs   # 3 行
grep -c "^pub(crate) fn run_" crates/cli/src/aux_commands.rs                 # 6
grep -n "start_signal_watcher\|spawn_watcher" -r crates/cli/src             # deps.rs:288、platform/mod.rs の説明 1 行と再公開 2 行、unix.rs・windows.rs の pub(crate) fn 各 1 行
ls crates/cli/src/platform                                                   # mod.rs unix.rs windows.rs
# 製品のコードの検査(数を変えずに通る)
cargo test -p conductor-cli --locked --test nfr_source                       # 6 通過
# 仕様書: 無くなったファイルの名前とモジュールのパスが残らない
grep -rn -E "cli/src/(retry|graph|init)\.rs|report/escape|escape\.rs|report::escape|crate::retry|retry::run_job|\{[^}]*\b(retry|graph|init|escape)\b[^}]*\}\.rs|ctx/run/pipeline/retry|\`(retry|graph|init)\.rs\`" docs/spec/rust docs/spec/language-agnostic \
  | grep -v "core/src\|definition/walk\|flow/retry\|definition_retry"        # 0 件
grep -n "step::run_job_with_retry" docs/spec/rust/02-architecture.md          # 1 行(S1)
grep -n "start_signal_watcher" docs/spec/rust/02-architecture.md              # 1 行(S24)
# 範囲の外が変わっていない
git diff --stat e124919 -- Cargo.lock '*Cargo.toml' .github README.md         # 空
git diff --stat e124919 -- crates/core crates/runtime crates/converter crates/cli/tests   # 空
git diff --stat e124919 -- docs/spec/language-agnostic docs/spec/go docs/spec/python docs/spec/typescript docs/spec/csharp docs/spec/powershell   # 空
```

- `cargo doc --workspace --no-deps --locked` の警告が、基準の 5 件(core 1、runtime 4)のまま。`cargo doc -p conductor-cli --no-deps --locked --document-private-items` に警告が出ない(足した説明のリンク)。
- Windows の対象のコンパイル: `cargo clippy --workspace --all-targets --locked --target x86_64-pc-windows-msvc -- -D warnings` と、本番の形 `cargo clippy -p conductor-cli --lib --bins --locked --target x86_64-pc-windows-msvc -- -D warnings` をローカルの macOS で走らせる(写しで通ることを確かめた)。実行は CI の `test`(windows)で確かめる。
- 本番の形: `cargo clippy -p conductor-cli --lib --bins --locked -- -D warnings` が通る(macOS)。
- 合格の条件: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、全テスト、行カバレッジ 80% 以上(写しで 98.06%)、MSRV 1.100 でのテスト(ローカルの rustc が 1.100.0)[memory:M2] [memory:M3] [memory:M4]。新しい依存を足さない(`Cargo.lock` と `Cargo.toml` の差分が無い)[memory:M5]。版の表記(`0.9.8`)は変えない。
- 利用者から見える動き: 処理(関数の本体)を変えないこと(下の本体の比べ)と、全テスト(シグナルの統合テスト `binary_signal.rs`・`signals_real.rs`・`signals_order.rs`、`flow_retry.rs`、`flow_graph.rs`、`flow_init.rs`、表示のテストを含む)が期待を変えずに通ることで確かめる。
- **関数の本体が変わっていないこと**: 移した関数(`run_job_with_retry`、`escape_name`、`escape_full`、`MAX_DISPLAY_CHARS`、`run_graph`、`read_overlay`、`run_init`、`unix.rs`・`windows.rs` の `spawn_watcher`)の本体を、元のファイル(基準)と移す先で、`fn` の行から閉じの `}` まで取り出して比べる。違うのは、`run_job_with_retry` の `pub(crate)` が消えること、`escape_name`・`escape_full` の `pub` が `pub(crate)` になること、`spawn_watcher` の `pub(super)` が `pub(crate)` になることだけ。

## 8. コミットの並び(小さな単位。各コミットでビルド、clippy、テストが通る)

各項目は別のファイル・別の名前を扱い、互いに依らないので、どの順でもビルドとテストは通る。重なるのは `lib.rs` のモジュールの表の 1 行(項目 1 と 3)と、仕様書 02 章の表(項目 1〜4。別の行)だけ。記述の順(1 → 2 → 3 → 4)にする。各コミットで、そのコミットで動かしたファイルを指す仕様書の行も直す。

**各コミットの確かめ**: `cargo fmt --all`(整えてから)、`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo clippy -p conductor-cli --lib --bins --locked -- -D warnings`、cli のテスト(`cargo test -p conductor-cli --locked`。456 件。`nfr_source` 6 件)、そのコミットの §7 の検索の該当部分。コミット 4 は、Windows の対象の clippy も行う(OS ごとのコードを変えるため)。最後の確かめ(5)で全テストと §7 のすべてを行う。

| # | コミット | 変えるもの | 件数の見込み |
|---|----------|------------|--------------|
| 1 | cli: リトライを含む実行を、1 つのジョブの手順のモジュールへまとめる | `step.rs`(`run_job_with_retry` を非公開で `run_step` の後に。説明の表に 1 行)、`lib.rs`(`mod retry;` とモジュールの表)、`run.rs:18` の説明、`retry.rs` を `git rm`。仕様書 S1〜S9 | 1,149 / cli 456 |
| 2 | cli: エスケープの包みを、表示のモジュールの本体へまとめる | `report/mod.rs`(`escape_name`・`escape_full`・`MAX_DISPLAY_CHARS` を `push_line` の前に。モジュールの説明)、`plan_text.rs`・`watch.rs`・`graph_text.rs` の `use`、`src/tests/escape_table.rs` の `use`(T1)、`report/escape.rs` を `git rm`。仕様書 S10〜S16 | 1,149 / cli 456 |
| 3 | cli: graph と init のコマンドを、補助のコマンドのモジュールへまとめる | `aux_commands.rs`(`run_graph`・`read_overlay`・`run_init` を `run_convert` の後に。モジュールの説明と、各関数の説明へ移した元のモジュールの説明)、`lib.rs`(`mod graph;`・`mod init;`、`match` の 2 行、モジュールの表)、`graph.rs`・`init.rs` を `git rm`。仕様書 S17〜S23 | 1,149 / cli 456 |
| 4 | cli: シグナルの監視の入口を、runtime と同じ再公開の形にする | `platform/mod.rs`(中継の 3 関数と `use` 3 つを消し、再公開 2 行。中継の説明をモジュールの説明へ)、`unix.rs`・`windows.rs` の `pub(super)` → `pub(crate)`。仕様書 S24 | 1,149 / cli 456 |
| 5 | (確かめ)§7 の検索、件数、テスト名の差、本体の比べ、`cargo doc` の警告、Windows の対象の clippy、本番の形の clippy、カバレッジの記録 | コミットなし。記録はコード生成の記録に書く | — |

合計 **4 コミット**(5 はコミットなし。実装で分けたりまとめたりしたら記録する)。版は上げない。タグとリリースはしない。

### 8.1 ファイルを消すときの履歴の追い方

- 4 つとも、移す先が既にあるファイルなので、`git mv` は使わない。同じコミットで、中身を移す先へ足し、元を `git rm` する(途中の状態で同じ関数が 2 つあったり、無かったりしない)。
- 元のファイルの履歴: `git log --oneline -- crates/cli/src/retry.rs`(`report/escape.rs`、`graph.rs`、`init.rs` も同じ)。消したコミットは `git log --diff-filter=D --oneline -- <元のパス>`、消す直前の中身は `git show <消したコミット>^:<元のパス>`。
- 行の出どころ: `git blame -w -C -C crates/cli/src/step.rs`(`aux_commands.rs`、`report/mod.rs` も同じ)。`-C -C` は、同じコミットで消した別のファイルからの写しを見つける(`-w` は字下げの差を無視する)。移した関数の行は、元のファイルのコミットに結び付く。
- コミットの本文に「元: <元のパス>」を書く(例: 「元: crates/cli/src/graph.rs、crates/cli/src/init.rs」)。

## 9. 範囲の外(書かない)

- core の小さなモジュール(`duration.rs`、`edges.rs`)、テストの統合、`platform` のファイルの統合。
- 関数の本体(処理)、関数の名前(`run_job_with_retry`、`escape_name`、`escape_full`、`run_graph`、`run_init`、`spawn_watcher`、`start_signal_watcher`)。
- `Cargo.toml`、`Cargo.lock`、CI の設定、README。版の更新、タグとリリース。
- 凍結した仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)と、言語に依らない仕様書(該当なし)。
- 製品のコードの検査のテストの中身。T1 の外のテスト。

## 10. 決めてほしい点

| ID | 決めること | 推奨 | 代案 | 推奨どおりでよいかの判断の手がかり | 決定 |
|----|------------|------|------|------------------------------------|------|
| D1 | `run_job_with_retry` の可視性 | **非公開の `fn` にする**(使うのは同じモジュールの `run_step` だけ) | `pub(crate)` のまま | 使う範囲に合わせる方針どおり。テストは直接呼ばないので、テストの都合は無い。写しで clippy・テストが通る | 決定: 非公開の `fn` にする(推奨のとおり。ユーザー、2026-11-25) |
| D2 | `escape_name`・`escape_full` の可視性 | **`pub(crate) fn` にする**(今の実効の範囲 crate の中と同じ。並びの `push_line` と同じ書き方)。単体テスト `escape_table.rs` は `use crate::report::{…}` で読む | (a) 非公開の `fn`: 本番の使う範囲(`report` と下位のモジュール。下位は親の非公開の項目を読める)に最も合う。ただし単体テスト `escape_table.rs`(`crate::tests` の下)から読めなくなり、テストの `use` の書き換えだけでは済まない(Q1 に反する。テストを `report` の中へ移すか、確かめ方を変える必要がある)/ (b) `pub fn`(書き方も今のまま) | crate の中の広さを使うのは単体テストだけで、本番だけなら非公開で足りる。ただし推奨は**今の範囲から広げない**(テストのために新しく広げるのではない)。converter の写しと同じ期待値の表で揃える `escape_table.rs` は、仕様書 08 章の 9 章が名指しする確かめで、包みの 2 関数を直接確かめるのはこのテストだけ | 決定: `pub(crate) fn` にする(推奨のとおり。ユーザー、2026-11-25) |
| D3 | `platform/mod.rs` の再公開の名前の付け方 | **`pub(crate) use unix::spawn_watcher as start_signal_watcher;`(`as` で今の名前のまま)**。`unix.rs`・`windows.rs` は可視性だけ変える | OS のファイルの関数の名前を `start_signal_watcher` に変え、`pub(crate) use unix::start_signal_watcher;`(runtime と全く同じく `as` なし) | 推奨は、置き場所・可視性・`use` だけを変える方針に合い、名前を 1 つも変えない。代案は runtime と字面までそろい、名前が 1 つになるが、OS のファイル 2 つの関数の名前が変わる。どちらも `deps.rs` は変わらない | 決定: `pub(crate) use … spawn_watcher as start_signal_watcher;` で名前を変えない(推奨のとおり。ユーザー、2026-11-25) |
| D4 | `step.rs` の中の `run_job_with_retry` の置き場所 | **`run_step` の直後**(呼ぶ順: `run_step` → `run_job_with_retry` → `report_execution`) | ファイルの最後 | 呼ぶ側の近くに置くと読みやすい。どちらも本体は同じ | 決定: `run_step` の直後(推奨のとおり。ユーザー、2026-11-25) |
| D5 | `report/mod.rs` の中の `escape_name`・`escape_full` の置き場所 | **`push_line` の直前**(下位のモジュールが共有する補助を並べる) | `use` の直後(ファイルの先頭) | `push_line` と同じく下位のモジュールが `use super::…` で読む補助。並べると、共有の補助の場所が 1 か所になる | 決定: `push_line` の直前(推奨のとおり。ユーザー、2026-11-25) |
| D6 | `aux_commands.rs` の中の並びと、モジュールの説明 | **`run_convert` の後に `run_graph`・`read_overlay`・`run_init`**(`CommandKind` と `lib.rs` の `match` の順)。モジュールの説明の見出しに `graph`・`init` と、元のモジュールの説明の括弧の中の流れの番号を足し、分け方の基準(準備をするコマンドは 1 コマンド 1 ファイル)を 1 文で書く。元のモジュールの説明の本文は各関数の説明へ移す | 見出しは今のままにし、`graph` と `init` の節の見出しのコメント(`// ---- graph ----`)を置いて、元のモジュールの説明をそこに写す | 推奨は、規則の番号が関数の説明に付き、`cargo doc --document-private-items` でも関数に結び付く。代案は元の説明の形を崩さないが、関数の外のコメントになる | 決定: `run_convert` の後に `run_graph`・`read_overlay`・`run_init`。元のモジュールの説明は各関数の説明へ移し、見出しに `graph`・`init` と分け方の基準を書く(推奨のとおり。ユーザー、2026-11-25) |
| D7 | 仕様書 rust に、版の履歴の注記(「v0.9.8 までは `retry.rs`・`report/escape.rs`・`graph.rs`・`init.rs` だった」など)を足すか | **足さない**(版を上げないので「v0.9.8 まで」と書けない。履歴は git の §8.1 の追い方で分かる) | 02 章の 5.4 の表の後に 1 文で「(版を上げない整理で)以前は別のファイルだった」と書く | 仕様書の表は今の形を書く。前の intent でも、版を上げない整理では版の注記を足していない | 決定: 足さない(推奨のとおり。ユーザー、2026-11-25) |

D1〜D7 は、ユーザーが決めた(すべて推奨のとおり。2026-11-25)。

**設計の承認で認めてもらうもの(ユーザーが設計の承認で認めた、2026-11-25)**: §4 の T 表(T1。消すテスト・名前を変えるテストは無い)、§2 の項目ごとの形、§8 のコミットの並び。

## 11. 成功の指標との対応(traceability.json)

`intent-statement.md` の「Success Metrics」の各項目に、上から ID を付けた。

| ID | 成功の指標(`intent-statement.md`) | 対応する規則 |
|----|------------------------------------|--------------|
| SM1 | `retry.rs` の中身は `step.rs` にあり、`retry.rs` は無くなる | BR1.1、BR1.2 |
| SM2 | `report/escape.rs` の中身は `report/mod.rs` にあり、`report/escape.rs` は無くなる | BR2.1、BR2.2 |
| SM3 | `graph`・`init` は `aux_commands.rs` にあり、`graph.rs`・`init.rs` は無くなる。準備をしないコマンドは `aux_commands.rs`、準備をするコマンドは 1 コマンド 1 ファイル | BR3.1、BR3.2 |
| SM4 | `platform/mod.rs` の中継は 1 段になり、runtime と同じ再公開の形。ディレクトリとファイル名は変わらない | BR4.1、BR4.2 |
| SM5 | 無くなるファイルが無いこと、無くなったモジュールのパスの参照が 0 件であることを検索で確かめる | BR5.2 |
| SM6 | テストは `use` のパスの書き換えだけ。テスト名の差 0。件数は変わらない | BR9.2、BR9.3 |
| SM7 | cli の `nfr_source.rs` が、テストの数と中身を変えずに通る | BR4.2、BR9.6 |
| SM8 | 仕様書 rust のモジュールの表と説明が実装と合い、無くなったファイルの名前が残らない | BR5.1、BR5.2 |
| SM9 | 利用者から見える動きは変わらない。版は上げない | BR9.1、BR9.5 |
| SM10 | 合格の条件(fmt、clippy(Windows の対象を含む)、カバレッジ 80%、MSRV、依存なし) | BR9.4 |
