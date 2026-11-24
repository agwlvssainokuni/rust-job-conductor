# コード生成のまとめ(Code Summary): cli のライブラリの公開範囲を狭める

計画 `code-generation-plan.md` の Step 1〜10 を実装した。基準は HEAD b4510e3(コードの計画のコミット。コードは c35ada6 と同じ)。コードと仕様書のコミットは 12 件で、`functional-spec.md` §8 の並び 1〜12 と 1 対 1 に対応する(13 はコミットなしの確かめ)。ほかに、この記録のコミットが 1 件。push、タグ、リリース、版の更新はしていない(版は 0.9.8 のまま)。

方針(ユーザーの判断): テストのコードの都合で本番のコードを曲げない。cli のライブラリの公開範囲を、`main` が使う 4 つ、依存の注入の継ぎ目、継ぎ目に要る型(`SignalError`)の 13 項目だけにし、ほかの 15 項目を crate の中に狭めた。狭めた項目を直接使うテスト 162 件は `src/tests/` の単体テストへ移した。本番に足したのは `lib.rs` の最後の `#[cfg(test)] mod tests;`(と前の 1 行のコメント)だけ。本番の処理のコードは変えていない(変えたのは `lib.rs` の可視性・説明・末尾の宣言と、`src/report/escape.rs` の説明の文だけ)。

## コミットの一覧

| # | §8 | ハッシュ | 内容 |
|---|----|----------|------|
| 1 | 1 | `74932cd` | 単体テストの置き場所を作り、終了コードのテストを移す。`lib.rs` の最後に `#[cfg(test)] mod tests;`。`src/tests/mod.rs`、`src/tests/exit_codes.rs`(T6。`git mv`)。`tests/nfr_source.rs`(T10。D2)。仕様書 S8〜S11、S15〜S18、S19(`exit_codes.rs`)、S23、S24(`mod.rs`・`exit_codes.rs` の行) |
| 2 | 2 | `63426e7` | エスケープの表のテストを移す(`src/tests/escape_table.rs`。T6)。`src/report/escape.rs` の説明。仕様書 S12〜S14、S19、S24 |
| 3 | 3 | `ab6d60d` | run ID の指定の型のテストを移す(`src/tests/run_id_arg.rs`。T6)。仕様書 S19、S24 |
| 4 | 4 | `88b558f` | init の型のテストを移す(`src/tests/init_types.rs`。T6)。仕様書 S19(`init_template.rs`・`init_types.rs` の行を分け、`init_types.rs` だけを移す)、S24 |
| 5 | 5 | `be181e1` | 引数の文法のテストを移す(`src/tests/args_grammar.rs`。T1)。`src/tests/common/mod.rs` に `FakeEnv` の項目・`with`・`impl Env` だけを置く。仕様書 S19、S24 |
| 6 | 6 | `22c76e2` | graph の表示のテストを移す(`src/tests/graph_report.rs`、`graph_long_id.rs`。T3)。仕様書 S19、S24 |
| 7 | 7 | `0c3665f` | 計画の表示のテストを移す(`src/tests/plan_report.rs`。T4)。仕様書 S19、S24 |
| 8 | 8 | `1fc0a40` | ひな形のテストを移す(`src/tests/init_template.rs`。T5)。仕様書 S19、S24 |
| 9 | 9 | `0dfb8e3` | 表示の文言のテストを移す(`src/tests/report_messages.rs`。T2)。`src/tests/common/mod.rs` に `FakeEnv` の残り(`locale_read`・`color_read`・`for_language`)、`FakeSink`、`state_error`。`src/tests/common/kinds.rs`(T9b。`git mv`)。`tests/common/mod.rs` の `pub mod kinds;` とコメント(T9a の一部)。仕様書 S7、S19、S22、S24 |
| 10 | 10 | `a2ea890` | README の例と実際の出力を比べる 8 件を移す(`src/tests/docs_support.rs`、`docs_graph.rs`、`docs_dry_run.rs`、`docs_init.rs`。T7)。統合テストの 3 ファイルから 8 件を外し、説明を足す。仕様書 S20、S24 |
| 11 | 11 | `059bdb0` | `tests/common/mod.rs` にテストの言語 `Language`(T9a の残り。D5)。`tests/flow_init.rs`(期待のひな形を原本の写しで書く。D4)、`tests/flow_validate.rs`(T8)。仕様書 S21 |
| 12 | 12 | `ba27768` | 公開を狭める。`pub mod exit`・`report`・`run_id` を `mod` に、`pub use args::parse_args` と `pub use types::{…}`(11 個)を消し、`run_cli_with` が使う `parse_args`・`CommandKind`・`ParsedArgs` を非公開の `use` で取り込む。`lib.rs` の説明(`inventory.md` §1.3)。仕様書 S1〜S6 |
| — | 13 | (コミットなし) | §7 の検索、件数、テスト名の差、`cargo doc` の警告、Windows の対象の clippy、本番の形の clippy、カバレッジ(下の記録) |

各コミットで、失敗で止まる確認のスクリプト(`set -euo pipefail`。`target/cli-pub-work/` に置き、終わってから消した)で、`cargo fmt --all`(整えてから)、`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test -p conductor-cli --locked` を通し、cli の単体(`src/lib.rs`)・統合(`tests/`)の件数がコミットごとの見込みと一致し、ほかの対象(`main.rs` など)の件数が 0、失敗が 0 であることを確かめた。最後に全テストを実行した。

## 件数の移り変わり(macOS、rustc 1.100.0 = MSRV、debug)

| 時点 | cli 単体 | cli 統合 | 見込み(§8) |
|------|----------|----------|--------------|
| 基準(b4510e3) | 2 | 456 | — |
| 1(`74932cd`) | 9 | 449 | 9 / 449 |
| 2(`63426e7`) | 12 | 446 | 12 / 446 |
| 3(`ab6d60d`) | 13 | 445 | 13 / 445 |
| 4(`88b558f`) | 16 | 442 | 16 / 442 |
| 5(`be181e1`) | 76 | 382 | 76 / 382 |
| 6(`22c76e2`) | 106 | 352 | 106 / 352 |
| 7(`0c3665f`) | 113 | 345 | 113 / 345 |
| 8(`1fc0a40`) | 120 | 338 | 120 / 338 |
| 9(`0dfb8e3`) | 156 | 302 | 156 / 302 |
| 10(`a2ea890`) | 164 | 294 | 164 / 294 |
| 11(`059bdb0`) | 164 | 294 | 164 / 294 |
| 12(`ba27768`) | 164 | 294 | 164 / 294 |

全体(`cargo test --workspace --locked`。失敗は常に 0):

| 時点 | core 単体 | core 統合 | runtime 単体 | runtime 統合 | converter 単体 | converter 統合 | cli 単体 | cli 統合 | 合計 |
|------|-----------|-----------|--------------|--------------|----------------|----------------|----------|----------|------|
| 基準(b4510e3) | 112 | 270 | 69 | 128 | 96 | 19 + 1 無視 | 2 | 456 | **1,152 通過、1 無視、0 失敗** |
| 最後(ba27768) | 112 | 270 | 69 | 128 | 96 | 19 + 1 無視 | 164 | 294 | **1,152 通過、1 無視、0 失敗**(見込みどおり。1,152 − 0 + 0) |

消したテスト 0 件、足したテスト 0 件(BR8.1)。`nfr_source.rs` の件数は 6 のまま。

## 公開の判定の結果(`inventory.md` §1.1)

- 残した 13 項目: `run_cli`、`StdSink`、`SystemEnv`、`install_panic_hook`(`main` が使う)、`run_cli_with`、`Deps`、`Runtime`、`Sink`、`Env`、`Signals`、`SystemRuntime`、`SystemSignals`(継ぎ目)、`SignalError`(継ぎ目に出る型)。
- 狭めた 15 項目: `parse_args`、`CliCommand`、`CommandKind`、`GraphFormat`、`InitOptions`、`InitTarget`、`Language`、`Level`、`ParsedArgs`、`RunIdArg`、`UsageError`、`UsageErrorKind`、`exit`・`report`・`run_id`(`pub mod` を `mod` に)。
- 下位モジュールの中の `pub` は `pub` のまま(`pub(crate)` に書き換えていない)。本番で使われなくなる項目は 0 件(`cargo clippy -p conductor-cli --lib --bins --locked -- -D warnings` が警告なしで通る)。消した本番のコードは無い。
- 継ぎ目の形(trait のメソッドと引数、`Deps` の項目、`SignalError` の変種)、`run_cli`・`run_cli_with` のシグネチャと本体は変えていない。

## テストの変更

### 移したテスト(162 件。名前は変えていない)

| 移した先(単体) | 元(統合) | 件数 | 本体で変えた行 |
|------------------|------------|------|----------------|
| `src/tests/args_grammar.rs` | `tests/args_grammar.rs`(T1) | 60 | `mod common;` と後ろの空行を消す。import を `use super::common;`、`use crate::args::parse_args;`、`use crate::types::{…};` に。`:620` の `conductor_cli::run_id::resolve_run_id` を `crate::run_id::resolve_run_id` に(差 10 行) |
| `src/tests/report_messages.rs` | `tests/report_messages.rs`(T2) | 36 | `mod common;` と後ろの空行を消す。import を `use super::common;`、`use super::common::kinds::{…};`、`use super::common::{FakeEnv, FakeSink};`、`use crate::report::{…};`、`use crate::types::{…};` に(差 11 行) |
| `src/tests/graph_report.rs` | `tests/graph_report.rs`(T3) | 28 | import の 3 行(差 6 行) |
| `src/tests/graph_long_id.rs` | `tests/graph_long_id.rs`(T3) | 2 | import の 2 行(差 4 行) |
| `src/tests/plan_report.rs` | `tests/plan_report.rs`(T4) | 7 | import の 2 行(差 4 行) |
| `src/tests/init_template.rs` | `tests/init_template.rs`(T5) | 7 | import の 2 行(差 4 行)。原本 `ORIGINAL_JA`・`ORIGINAL_EN` とモジュールの説明はそのまま |
| `src/tests/exit_codes.rs` | `tests/exit_codes.rs`(T6) | 7 | `use crate::exit::*;`(差 2 行) |
| `src/tests/escape_table.rs` | `tests/escape_table.rs`(T6) | 3 | `use crate::report::escape::{escape_full, escape_name};`(差 2 行) |
| `src/tests/run_id_arg.rs` | `tests/run_id_arg.rs`(T6) | 1 | `use crate::types::{RunIdArg, UsageErrorKind};`(差 2 行) |
| `src/tests/init_types.rs` | `tests/init_types.rs`(T6) | 3 | `use crate::types::{CliCommand, CommandKind, InitOptions, InitTarget};`(差 2 行) |
| `src/tests/docs_graph.rs` | `tests/docs_graph.rs` の 6 件(T7) | 6 | 説明に 1 行。`mod docs_support;` と import を `use super::docs_support::readme;` に。`render_text` の呼び出し 2 か所のパスを `crate::report::graph_text::render_text(crate::types::Language::En, …)` に(補助 `render` の中の 1 か所は `cargo fmt` で 1 行になった)。補助 `graph_example`・`render`・`jobs` も一緒に移る |
| `src/tests/docs_dry_run.rs` | `tests/docs_dry_run.rs` の 1 件(T7) | 1 | 説明に 1 行。import。`render_plan` の呼び出しのパスを `crate::…` に |
| `src/tests/docs_init.rs` | `tests/docs_init.rs` の 1 件(T7) | 1 | 説明に 1 行。import を `use super::docs_support::readme;`、`use crate::report::init_template::render_template;`、`use crate::types::Language;` に |

- T1〜T6 の 10 ファイルと `common/kinds.rs` は `git mv` で移し、`git diff -M b4510e3` で名前の変更として検出される。差は上の import とパスの行だけ(`git diff -M b4510e3 -- …` の `+`/`-` の行を全部見て確かめた)。
- T7 の 3 ファイルは、`git mv` で移したあと、残る統合テストを元の場所に書き直したので、git は移動ではなく「元の場所の変更と新しいファイル」と見る(`git show -C` では写しとして検出される)。元の本体と移した本体を `diff` で比べ、差が上の行だけであることを確かめた。
- `#[cfg(unix)]`(`args_grammar.rs` の 4 か所)、`cfg!(unix)`(`report_messages.rs` の 2 か所)、`panic!(`(`args_grammar.rs` の 5 か所、`init_template.rs` の 2 か所)は、そのまま移した(D2)。

### 補助(T9)

- `src/tests/mod.rs`: §2.2 の形(下位のモジュールの宣言だけ。最後の形は §2.2 と同じ並び)。
- `src/tests/common/mod.rs`: `FakeEnv`(項目、`locale_read`、`color_read`、`with`、`for_language`、`impl Env`)、`FakeSink`(項目、`out_text`、`err_text`、`impl Sink`)、`state_error` を、統合テストの `tests/common/mod.rs` から本体を変えずに写した。コミット 5〜8 は `FakeEnv` の項目・`with`・`impl Env` だけ、コミット 9 で残りと `pub mod kinds;` を足した。`#![allow(dead_code)]` は付けていない(どのコミットでも `clippy -D warnings` が通った)。
- `src/tests/common/kinds.rs`: `git mv`。`use crate::types::UsageErrorKind as U;` の 1 行だけ変えた。
- `src/tests/docs_support.rs`: `readme()` の写しだけ。
- `tests/common/mod.rs`(T9a): `pub mod kinds;` を消した(9)。`state_error` のコメントを「単体テストの `src/tests/report_messages.rs` も同じ写しを使う」に直した(9。2 行に折り返した)。`conductor_cli` の import から `Language` を消し(`cargo fmt` で 1 行になった)、`// ---- Env ----` の後ろにテストの言語 `Language { En, Ja }` を足した(11)。`for_language` の本体と `#![allow(dead_code)]`・モジュールの説明は変えていない。

### 統合テストに残して書き方を変えたテスト(T8)

| ファイル | 変えた所 | 期待の値 |
|----------|----------|----------|
| `tests/flow_init.rs` | import を `use common::{FakeEnv, FakeSignals, FakeSink, Language, ScriptedRuntime, SignalMode, run_with_fakes};`(`cargo fmt` で折り返し)にし、`conductor_cli::Language` と `render_template` の import を消した。原本 `ORIGINAL_JA`・`ORIGINAL_EN`(単体テストの `src/tests/init_template.rs` の原本と同じ文字列。2 つの定数を `diff` で比べて同じことを確かめた)と補助 `expected_template(lang, name)`(`name = "my-pipeline"` を 1 回だけ置き換える)を足した。`既定の名前とパスで_ひな形が書かれ_標準エラーに作成の1行` の 2 か所と `config_ハイフンは標準出力にひな形だけを出し_force_を無視し_ファイルを作らない` の 2 か所の `render_template(…)` を `expected_template(…)` に | 同じバイト列(9 件すべてが通る) |
| `tests/flow_validate.rs` | `FakeEnv::for_language(conductor_cli::Language::En)` を `FakeEnv::for_language(common::Language::En)` に | 同じ |

- `tests/docs_graph.rs`(2 件)、`docs_dry_run.rs`(1 件)、`docs_init.rs`(1 件)は、移したテストを外し、モジュールの説明に移した先を書いた。残るテストの本体は変えていない。
- `tests/nfr_source.rs`(T10): §2.3 のとおり。下の「ソースの検査の範囲の記録」。

### 消したテスト・足したテスト・名前を変えたテスト

いずれも 0 件。

### テスト名の一覧の比較

`cargo test --workspace --locked -- --list 2>/dev/null | grep ": test$"` を基準(b4510e3。1,153 行)と最後(ba27768。1,153 行)で比べた。

- 名前の最後の部分で比べた差(`sed 's/: test$//; s/.*:://' | sort` を `diff`): **0 件**(消えた名前・増えた名前・名前の変わったテストは無い。★ なし)。
- パスで比べた差: **324 行**(移した 162 件の前後)。cli の単体テストの `tests::` のパスの内訳は `args_grammar` 60、`report_messages` 36、`graph_report` 28、`exit_codes` 7、`init_template` 7、`plan_report` 7、`docs_graph` 6、`escape_table` 3、`init_types` 3、`graph_long_id` 2、`docs_dry_run` 1、`docs_init` 1、`run_id_arg` 1(`inventory.md` §0.2 と同じ)。

## 完了の判定の検索(`functional-spec.md` §7。最後のコミット ba27768 のあと)

| 検索 | 結果 |
|------|------|
| `grep -n "^pub " crates/cli/src/lib.rs` | 4 件: `pub use deps::{…}`(10 個)、`pub use panic_hook::install_panic_hook`、`pub fn run_cli`、`pub fn run_cli_with` |
| `grep -n "^pub mod\|pub use args\|pub use types" crates/cli/src/lib.rs` | 0 件 |
| `tail -2 crates/cli/src/lib.rs` | `#[cfg(test)]` と `mod tests;` |
| `grep -rn "conductor_cli::\(parse_args\|CliCommand\|…\|run_id\)" crates/cli/tests` | 0 件 |
| `grep -rn -A3 "use conductor_cli::{" crates/cli/tests \| grep -E "parse_args\|…\|Init"` | 0 件 |
| `grep -rn "conductor_cli" crates/cli/src/tests` | 0 件 |
| `ls crates/cli/src/tests crates/cli/src/tests/common` | `mod.rs`、`docs_support.rs`、`common/` と 13 ファイル / `mod.rs`、`kinds.rs` |
| `test ! -e crates/cli/tests/common/kinds.rs` | 無い |
| `git diff --stat c35ada6 -- crates/core crates/runtime crates/converter Cargo.lock '*Cargo.toml'` | 空 |
| `grep -rn "unsafe\|low_level\|std::net\|…\|openssl" crates/cli/src/tests` | 0 件 |
| `grep -rln "cfg(unix)\|cfg!(unix)" crates/cli/src/tests` | `args_grammar.rs`、`report_messages.rs` だけ(移したテストのまま) |
| `grep -rn "#\[path" crates/cli/src \| grep -v "^crates/cli/src/tests/"` | 0 件 |
| `grep -n "is_unit_test\|#\[path\|ends_with(\"#\[cfg(test)\]" crates/cli/tests/nfr_source.rs` | 定義(31 行)、除く条件 2 か所(138、149 行)、末尾の宣言の確かめ(157 行)、`#[path` の確かめ(160〜161 行)、モジュールの説明(20 行) |
| `grep -n "再公開\|公開モジュール" docs/spec/rust/02-architecture.md` | 2.4 は 137 行の S1 の文(`parse_args`・型・`exit`・`report`・`run_id` を公開として書かない)。ほかは core・runtime の節と 5 章の表 |
| `grep -rn "cli/tests/escape_table\|\`tests/escape_table" docs/spec/rust` | 0 件 |
| `grep -n "args_grammar.rs\|…\|common/kinds.rs" docs/spec/rust/11-quality-and-build.md` | cli の単体テストの表(S24)の行(291〜302 行)と、3.3 の表の `cli/src/tests/escape_table.rs`(162 行)。core・converter の表の `common/kinds.rs`(221、248 行)は対象の外 |
| `grep -n "src/tests" docs/spec/rust/{02,07,08,11}-*.md` | 02:257(S7)、07:29(S10)、07:401(S11)、08:585(S12)、08:605(S13)、11:162(S14)、11:166(S15)、11:173(S16)、11:178(S17)、11:188(S18)、11:283(S23)、11:285(S24 の表の見出し)と、converter の既存の行(11:149、244、245、250)。02 の S9 の行は `tests/` と書くので、この検索には出ない |
| `git diff --stat c35ada6 -- docs/spec/language-agnostic docs/spec/{go,python,typescript,csharp,powershell}` | 空 |
| `grep -n "\[\`\(parse_args\|run_id\|exit\|report\)\`\]" crates/cli/src/lib.rs` | 0 件 |
| `git diff b4510e3 -- crates/core crates/runtime crates/converter` | 空 |
| `git diff --stat b4510e3 -- Cargo.toml Cargo.lock crates/cli/Cargo.toml README.md .github` | 空 |
| `grep -rn "cfg(unix)\|cfg(windows)\|cfg!(unix)\|cfg!(windows)" crates/cli/src \| grep -v "src/tests/\|src/platform/"` | 1 件(`lib.rs:55` の説明の文。既存) |
| コミット 12 の前の `git grep -n "conductor_cli::" -- crates/cli/tests` | `tests/common/mod.rs` の `Deps`・`Env`・`Runtime`・`SignalError`・`Signals`・`Sink`・`SystemRuntime`・`run_cli_with`、`signals_order.rs`・`signals_real.rs` の `Signals`・`SystemSignals` だけ(公開に残す名前) |

## 合格の条件の確かめ

- `cargo fmt --all -- --check`: 通る。
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: 通る。
- `cargo clippy -p conductor-cli --lib --bins --locked -- -D warnings`(テストを含まない本番の形): 通る(狭めた項目に `dead_code` が出ない)。
- `cargo clippy --workspace --all-targets --locked --target x86_64-pc-windows-msvc -- -D warnings`(macOS から): 通る。実行は CI の `test`(windows)で確かめる。
- `cargo test --workspace --locked`: 1,152 通過、1 無視、0 失敗。ローカルの rustc は 1.100.0(MSRV)なので、MSRV でのテストを兼ねる。
- `cargo doc --workspace --no-deps --locked`: 警告は基準と同じ 5 件(core 1、runtime 4。この intent と無関係)。cli は 0 件。
- `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`: 通る(終了コード 0)。全体の行 **98.06%**(`inventory.md` §0.2 の写しの値と同じ)。`src/tests/` の下のファイルは表に出ない(テストのコード)。
- 新しい依存なし(`Cargo.toml`・`Cargo.lock` の差分なし)。`unsafe` と製品のコードへの `cfg(unix|windows)` を足していない。cli は `#![forbid(unsafe_code)]` のまま。版は 0.9.8 のまま。
- 利用者から見える動き: 統合テスト(流れ・実バイナリ・デモ・非機能。294 件)と移した単体テスト(162 件)が、期待を変えずに通る。

## ソースの検査の範囲の記録(BR3.2。D2)

`tests/nfr_source.rs` は `src/` の下のすべての `.rs` を再帰で読み、各行の `//` 以降を除いて検査する。

| 検査 | 対象 | 変更 |
|------|------|------|
| `クレートは_forbid_unsafe_code_を宣言している` | `lib.rs` | 変えない |
| `src_に_unsafe_と_low_level_がない` | `src/` の全ファイル(`src/tests/` を含む) | 変えない |
| `src_にネットワークのコードがない` | `src/` の全ファイル(`src/tests/` を含む) | 変えない |
| `os_ごとの分岐は_platform_の中だけ` | `platform` と `src/tests/` を除く全ファイル | 除く条件に `\|\| is_unit_test(path)` を足した |
| `テスト専用の経路を実行ファイルに持たない` | `main.rs` と、`bin` と `src/tests/` を除く全ファイル | 除く条件に `\|\| is_unit_test(path)` を足し、`lib.rs` の末尾(改行を `\n` にそろえ、`trim_end` したあと)が `#[cfg(test)]\nmod tests;` であることと、`src/tests/` の外に `#[path` が無いことの 2 つの確かめを足した |
| `依存に_c_のコンパイルを要するものとネットワークのライブラリがない` | `cargo tree` | 変えない |

- `is_unit_test(path)` は `path.starts_with(src_dir().join("tests"))`。テストの数は 6 のまま。
- `src/tests/` は `lib.rs` の最後の `#[cfg(test)] mod tests;` からだけ読まれる(末尾の宣言の確かめ)。製品のファイルに `#[path` は 0 件(同じテストと上の `grep` で確かめた)ので、`src/tests/` を本番から読む道は無い。
- `src/tests/` の中で検査の外になったのは、`#[cfg(unix)]` 4 か所・`cfg!(unix)` 2 か所(`args_grammar.rs`、`report_messages.rs`)と `panic!(` 7 か所(`args_grammar.rs` 5、`init_template.rs` 2)。どれも移したテストのまま。
- `unsafe`・`low_level`・ネットワークの語の検査は `src/tests/` も対象のまま通る(上の `grep` も 0 件)。

## 計画からのずれ

- 計画からのずれは無い。T1〜T10 の外の既存のテストを変える必要は出なかった。件数はすべてのコミットで見込みどおり。
- `docs/spec/rust/11-quality-and-build.md` の単体テストの表(S24)は、行を足す順をコミットごとに決めたので、コミット 4 で `init_types.rs` の行を一度 `escape_table.rs` の前に置き、コミット 6 で元の統合テストの表と同じ並び(`init_types.rs` を最後)に直した。最後の並びは元の表と同じ。
- S19 の `init_template.rs`・`init_types.rs` の 1 行は、コミット 4 で 2 つに分け(`init_types.rs` の文「`InitOptions`・`InitTarget`」)、コミット 8 で `init_template.rs` の行を移した。
- S16 は `inventory.md` の語句のとおり「(cli の単体テスト `src/tests/` を除く)」を足したので、後ろの「(ソース検索のテストが強制。…)」と括弧が 2 つ並ぶ。
- `tests/common/mod.rs` の `state_error` のコメントは、直した文が長くなったので 2 行に折り返した。
- `src/tests/common/mod.rs` では、`state_error` を「`// ---- ステートの誤り ----`」の見出しの下に置いた(統合テストでは `// ---- Runtime ----` の下。本体とコメントは同じ)。モジュールの説明は写しであることを書いた新しい文。
- `src/tests/docs_*.rs` は、元のファイルを `git mv` してから、残る統合テストを元の場所に書いたので、git の履歴では移動として表示されない(`git show -C` で写しとして検出される)。
- `cargo fmt --all` が `src/tests/mod.rs` の `mod` の宣言を並べ替えた(最後の形は §2.2 と同じ)。
