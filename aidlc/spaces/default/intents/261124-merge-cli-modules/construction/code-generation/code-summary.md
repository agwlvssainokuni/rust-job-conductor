# コード生成のまとめ(Code Summary): cli の本番の小さなモジュールの統合

計画 `code-generation-plan.md` の Step 1〜6 を実装した。基準は HEAD 9fe86f5(コードの計画のコミット。コードと仕様書は e124919 と同じ)。コードと仕様書のコミットは 4 件で、`functional-spec.md` §8 の並び 1〜4 と 1 対 1 に対応する(5 はコミットなしの確かめ)。ほかに、この記録のコミットが 1 件。push、タグ、リリース、版の更新はしていない(版は 0.9.8 のまま)。

方針(ユーザーの判断): テストのコードの都合で本番のコードを曲げない。処理(関数の本体)は変えず、置き場所、可視性、`use`、説明の文だけを変えた。利用者から見える動き(出力、終了コード、ステートファイル、`--dry-run`、シグナルの扱い)は変わらない。`Cargo.toml`、`Cargo.lock`、CI の設定、README、core・runtime・converter、凍結した仕様書、言語に依らない仕様書は 1 行も変えていない。

## コミットの一覧

| # | §8 | ハッシュ | 内容 |
|---|----|----------|------|
| 1 | 1 | `931be4f` | cli: リトライを含む実行を、1 つのジョブの手順のモジュールへまとめる。`run_job_with_retry` を `step.rs` の `run_step` の直後へ、非公開で(D1、D4)。元のモジュールの説明を関数の説明へ移し、呼ぶ側を `run_step` に直した。`step.rs` の説明の表に 1 行。`lib.rs` の `mod retry;` とモジュールの表、`run.rs` の説明。仕様書 S1〜S9。元: `crates/cli/src/retry.rs` |
| 2 | 2 | `c1b261e` | cli: エスケープの包みを、表示のモジュールの本体へまとめる。`MAX_DISPLAY_CHARS`(非公開)、`escape_name`・`escape_full`(`pub(crate)`。D2)を `report/mod.rs` の `push_line` の直前へ(D5)。モジュールの説明のエスケープの箇条に元の説明を写した。`plan_text.rs`・`graph_text.rs`・`watch.rs` の `use`、T1。仕様書 S10〜S16。元: `crates/cli/src/report/escape.rs` |
| 3 | 3 | `6fb74e3` | cli: graph と init のコマンドを、補助のコマンドのモジュールへまとめる。`run_graph`・`read_overlay`・`run_init` を `aux_commands.rs` の `run_convert` の後へ(D6。可視性は変えない)。元のモジュールの説明は各関数の説明へ。見出しに `graph`・`init` と流れの番号、分け方の基準。`lib.rs` の `mod`、呼び出し、モジュールの表。仕様書 S17〜S23。元: `crates/cli/src/graph.rs`、`crates/cli/src/init.rs` |
| 4 | 4 | `9c35e43` | cli: シグナルの監視の入口を、runtime と同じ再公開の形にする。`platform/mod.rs` の中継の 3 関数と `use` 3 つを消し、`#[cfg(unix)]`/`#[cfg(windows)]` の `pub(crate) use … spawn_watcher as start_signal_watcher;`(D3)。`unix.rs`・`windows.rs` の `spawn_watcher` を `pub(super)` → `pub(crate)`。中継の説明はモジュールの説明へ。仕様書 S24(ファイルを消していないので「元:」なし) |
| — | 5 | (コミットなし) | §7 の検索、件数、テスト名の差、本体の比べ、`cargo doc` の警告、Windows の対象の clippy、本番の形の clippy、カバレッジ(下の記録) |

各コミットで、失敗で止まる確認のスクリプト(`set -eu`。`target/merge-cli-cg/` に置き、終わってから消した)で、`cargo fmt --all`(整えてから)、`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo clippy -p conductor-cli --lib --bins --locked -- -D warnings`、`cargo clippy --workspace --all-targets --locked --target x86_64-pc-windows-msvc -- -D warnings`、`cargo test -p conductor-cli --locked`(失敗 0 かつ 456 件通過でなければ止まる)、`Cargo.lock`・`crates/cli/Cargo.toml`・core・runtime・converter の差分なし、を通した。Windows の対象の clippy は、計画ではコミット 4 だけだったが、指示に従い全コミットで走らせた。

## 消したファイルとまとめ先

| 消したファイル | まとめ先 | コミット |
|----------------|----------|----------|
| `crates/cli/src/retry.rs` | `crates/cli/src/step.rs`(`run_job_with_retry`) | `931be4f` |
| `crates/cli/src/report/escape.rs` | `crates/cli/src/report/mod.rs`(`MAX_DISPLAY_CHARS`、`escape_name`、`escape_full`) | `c1b261e` |
| `crates/cli/src/graph.rs` | `crates/cli/src/aux_commands.rs`(`run_graph`、`read_overlay`) | `6fb74e3` |
| `crates/cli/src/init.rs` | `crates/cli/src/aux_commands.rs`(`run_init`) | `6fb74e3` |

cli の `src/` の下の `.rs` は 40 → 36。履歴は `git log --oneline -- <元のパス>`、行の出どころは `git blame -w -C -C <まとめ先>`(`functional-spec.md` §8.1)。

直した後の行数: `step.rs` 323(見込み 321)、`report/mod.rs` 659(見込み 657)、`aux_commands.rs` 222(見込みどおり)、`platform/mod.rs` 38(見込みどおり)。差の 2 行ずつは、移した説明の折り返し(`step.rs`)と、エスケープの箇条の文の直し(`report/mod.rs`。R-04)による。

## 件数(macOS、rustc 1.100.0 = MSRV、debug)

| 時点 | cli | 全体 |
|------|-----|------|
| 基準(9fe86f5) | 456 | 1,149 通過、1 無視(テスト名 1,150 行) |
| 1〜4 の各コミット | 456 | —(cli だけを各コミットで実行) |
| 最後(9c35e43) | 456 | **1,149 通過、1 無視、0 失敗** |

基準 1,149 − 消したテスト 0 + 足したテスト 0 = 1,149 に一致。

## テストの変更

- `git diff 9fe86f5 -- crates/cli/src/tests crates/cli/tests`: `crates/cli/src/tests/escape_table.rs` の `use` の 1 行だけ(T1。`use crate::report::escape::{escape_full, escape_name};` → `use crate::report::{escape_full, escape_name};`)。名前・確かめの行・期待の値は変えていない。
- テスト名の一覧(`cargo test --workspace --locked -- --list 2>/dev/null | grep ": test$" | sort`)を基準と最後で比べた: 1,150 行と 1,150 行、**差 0**(パスつき)。
- T1 の外の既存のテストを変えないと進めない項目は、出なかった。

## 完了の判定(`functional-spec.md` §7。最後のコミット 9c35e43 のあと)

| 検索・確かめ | 結果 |
|--------------|------|
| 無くなるファイル 4 つ | 4 つとも無い(出力なし) |
| 無くなったモジュールのパスの参照(コード。`git grep … -- crates/cli`) | 0 件 |
| `grep -n "fn run_job_with_retry" step.rs` | 1 行(179 行目。`pub` なし) |
| `grep -n "fn escape_name\|fn escape_full\|const MAX_DISPLAY_CHARS" report/mod.rs` | 3 行(554、557、562) |
| `grep -c "^pub(crate) fn run_" aux_commands.rs` | 6 |
| `start_signal_watcher\|spawn_watcher` | `deps.rs:288`、`platform/mod.rs` の説明 1 行(20)と再公開 2 行(28、33)、`unix.rs:29`・`windows.rs:27` の `pub(crate) fn` |
| `grep -c "^pub(crate) use .* as start_signal_watcher;" platform/mod.rs` | 2 |
| `grep -n "fn start_signal_watcher\|fn spawn_watcher" platform/mod.rs` | 0 件 |
| `ls crates/cli/src/platform` | `mod.rs unix.rs windows.rs` |
| 仕様書の古い名前(最後の `grep -v` の除外の**前**) | 1 件: `docs/spec/rust/05-execution-flow.md:5` の `crates/core/src/flow/{classify,decide,next_step,resume,retry}.rs`(core の別のファイル。正しく除外の対象) |
| 仕様書の古い名前(除外の後) | 0 件 |
| `step::run_job_with_retry`(02 章) | 1 行(216。S1) |
| `start_signal_watcher`(02 章) | 1 行(377。S24) |
| `git diff --stat e124919 -- Cargo.lock '*Cargo.toml' .github README.md` | 空 |
| `git diff --stat e124919 -- crates/core crates/runtime crates/converter crates/cli/tests` | 空 |
| `git diff --stat e124919 -- docs/spec/language-agnostic docs/spec/{go,python,typescript,csharp,powershell}` | 空 |
| `git diff 9fe86f5 -- crates/core crates/runtime crates/converter` | 空 |
| `cargo test -p conductor-cli --locked --test nfr_source` | 6 通過 |
| `cargo fmt --all -- --check` | 通過 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 通過 |
| `cargo clippy -p conductor-cli --lib --bins --locked -- -D warnings`(本番の形) | 通過 |
| `cargo clippy --workspace --all-targets --locked --target x86_64-pc-windows-msvc -- -D warnings` | 通過 |
| `cargo clippy -p conductor-cli --lib --bins --locked --target x86_64-pc-windows-msvc -- -D warnings` | 通過 |
| `cargo doc --workspace --no-deps --locked` の警告 | 5 件(core 1、runtime 4。基準と同じ文) |
| `cargo doc -p conductor-cli --no-deps --locked --document-private-items` | 警告なし(足した説明のリンク `[`run_step`]`、`[`run_graph`]`、`[`run_init`]` を含む) |
| `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only` | 通過。全体の行 **98.06%**。`step.rs` 98.79%、`aux_commands.rs` 98.06%、`report/mod.rs` 100%、`platform/mod.rs` 100% |

### 関数の本体の比べ

移した関数を、基準(9fe86f5)の元のファイルと移す先で、`fn` の行から閉じの `}` まで取り出して比べた。

- `run_job_with_retry`: 違いは 1 行目の `pub(crate) fn` → `fn` だけ。
- `escape_name`・`escape_full`: 違いは `pub fn` → `pub(crate) fn` だけ。`MAX_DISPLAY_CHARS` は同じ。
- `run_graph`(27 行)、`read_overlay`(14 行)、`run_init`(37 行): 同じ。
- `unix.rs`・`windows.rs` の `spawn_watcher`: 違いは `pub(super)` → `pub(crate)` の 1 行だけ。

### `report/mod.rs` の説明の文(レビューの R-04。目で確かめた)

コミット 2 の後の `report/mod.rs` のモジュールの説明のエスケープの箇条は、「入力に由来する文字列(…)は、表示の安全のエスケープ(Escape。NFR4.2。端末に出せる形にする)を通す」で始まり、消えたモジュール `escape` の名前を指していない。続けて「2 つは、このモジュールにあり、下位のモジュール(計画・watch・図の整形)も使う。本体は conductor-core の `escape_display` で、…」と、元の `escape.rs` の説明を写した。

## 計画からのずれ

- 行数が見込みより 2 行ずつ多い(`step.rs` 323、`report/mod.rs` 659)。説明の折り返しと R-04 の文の直しによる。処理には関係しない。
- Windows の対象の clippy を、計画のコミット 4 だけでなく、全コミットで走らせた(指示に従った。結果はすべて通過)。
- 確認のスクリプトは `set -eu`(`pipefail` なし)。テストの出力は `grep`・`tee` を通すため、テストの終了コードの代わりに、`FAILED`・`panicked` の有無と、通過 456・失敗 0 の数で止めた。
- ほかに、計画と設計からのずれは無い。
