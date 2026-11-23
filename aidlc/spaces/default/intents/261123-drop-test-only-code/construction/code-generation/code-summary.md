# コード生成のまとめ(Code Summary): テストだけが使う本番のコードの削除

計画 `code-generation-plan.md` の Step 1〜11 を実装した。基準は HEAD 23b8975(コードの計画のコミット。コードは 9e91512 と同じ)。コードと仕様書のコミットは 24 件で、`functional-spec.md` §8 の並び 1〜24 と 1 対 1 に対応する(25 はコミットなしの確かめ)。ほかに、この記録のコミットが 1 件。push、タグ、リリース、版の更新はしていない(版は 0.9.8 のまま)。

方針(ユーザーの判断): テストのコードの都合で本番のコードを曲げない。本番から、テストだけが使う実装と公開範囲をなくし、テストは偽物の側か公開の入口で代わりを持つ。本番に、テストのための要素を新しく足さない。

## コミットの一覧

| # | §8 | 項目 | ハッシュ | 内容 |
|---|----|------|----------|------|
| 1 | 1 | 1a | `bcaff74` | core・runtime: `Command` の `From<&str>`・`From<String>` を消す(T2、T5、T6、T7、T9、T18、T19、T20 の一部、T21、T22、T23 の一部。§4.6 の `文字列から_line_を作れる`) |
| 2 | 2 | 1b | `77f374c` | cli: `RunIdArg::as_str` を消す(T27、T28。仕様書 rust 02) |
| 3 | 3 | 1c | `cd40010` | runtime・cli: `InterruptSignal` の `is_raised`・`PartialEq`・`Eq` を消す。`Default` は残す(D1)、`Debug` は `signal_number()` で書く(D8)(T17、T29、T30。仕様書 rust 02・07) |
| 4 | 4 | 1d-1 | `2a96695` | core: `JobStatus` の `Default` を消す(T1) |
| 5 | 5 | 1d-2 | `c2bfa1c` | runtime: `Durability` の `Default` を消す(§4.6 の `耐久性の既定は_durable`。仕様書 rust 02) |
| 6 | 6 | 1d-3 | `89ecdca` | core: `graph::StateOverlay` の `Default` を消す(T15、T16。§4.6 の `状態の重ね合わせの既定は空で_再開位置もない`) |
| 7 | 7 | 1d-4 | `2f8c3f2` | cli: `Language` の `Default` を消す(仕様書 rust 02) |
| 8 | 8 | 1e | `3cefdec` | core・cli: `sanitize` を消す(T3、T31 の説明の一部。§4.6 の `u1_core_の_sanitize_と同じ結果`。仕様書 rust 02・08・11・12) |
| 9 | 9 | 1f | `a3182a1` | core: `EnvFileError::reason()` を消す(T11、T12。仕様書 rust 02・10) |
| 10 | 10 | 2a | `6581226` | core・runtime: `decide_before_run` を `flow/decide.rs` の中だけに(T14、T25 の一部。ドキュメントコメント。仕様書 rust 02・12) |
| 11 | 11 | 2b | `45e0adb` | core: `flow::classify` を `flow` の中だけに(`pub(super)`。仕様書 rust 02) |
| 12 | 12 | 2c-1 | `df2d40b` | core: `MAX_RUN_ID_LEN` を `run_id.rs` の中だけに(仕様書 rust 02) |
| 13 | 13 | 2c-2 | `9e0e563` | core: `is_valid_env_name` を crate の中に(仕様書 rust 02) |
| 14 | 14 | 2c-3 | `48b7eed` | core: `is_reserved_env_name` を crate の中に(仕様書 rust 02) |
| 15 | 15 | 2c-4 | `3b5774d` | core: `JobDefinition::effective_success_codes` を crate の中に(T8。仕様書 rust 02) |
| 16 | 16 | 2d | `b2a14f7` | cli: `report::escape` の `escape_display` の再公開をやめる(T31 の import と説明の残り。仕様書 rust 02・08・11) |
| 17 | 17 | 2e | `dd15174` | cli: `pub mod` 4 つ(`args`・`deps`・`panic_hook`・`types`)を crate の中に(ドキュメントコメント。仕様書 rust 02) |
| 18 | 18 | 2f | `24420ad` | cli: 根の `GraphOptions`・`DryRunOptions` の再公開をやめる(仕様書 rust 02) |
| 19 | 19 | 3a | `6b73dc2` | core: `RunIdError` を種類のない誤りの型 1 つ(`pub struct RunIdError;`)に(T4。仕様書 rust 02・05・10・12。language-agnostic は D4 で変えない) |
| 20 | 20 | 3b | `b6dae26` | runtime・cli: runtime の core の再公開をやめ、使う所を `conductor_core` に(T20 の一部、T23 の一部、T24、T25 の一部、T32。ドキュメントコメント。仕様書 rust 02・05・06・10) |
| 21 | 21 | 3c-1、3c-2 | `fbc325b` | core: `DefinitionErrorKind`・`EnvFileErrorKind` の `Display` と `Error` を消す(T13。仕様書 rust 03・08・10) |
| 22 | 22 | 3c-3 | `6b0dd00` | converter: `ConversionErrorKind` の `Error` を消す(`Display` は残す) |
| 23 | 23 | 3d-1、3d-2 | `eb7303e` | core: `DefinitionErrorKind`・`EnvFileErrorKind` の `Hash` を消す(T10) |
| 24 | 24 | 3d-3、3d-4 | `e4a2bf1` | converter: `ConversionErrorKind`・`TargetFormat` の `Hash` を消す(T26) |
| — | 25 | 確かめ | (コミットなし) | §7 の検索、件数、`cargo doc` の警告、Windows の対象の clippy、カバレッジ(下の記録) |

各コミットで、失敗で止まる確認のスクリプト(`set -eo pipefail`)で、`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked --no-run`(全テストのコンパイル)と、そのコミットで変えたクレートのテストを通した(整形は `cargo fmt --all`)。各コミットで `git diff --stat -- '*Cargo.toml' Cargo.lock` が空であることも確かめた。2 つのコミットにまたがる import(T20・T23・T25)は、途中のコミット(1、10)でも `--no-run` と clippy が通った。最後に全テストを実行した。

## 件数の移り変わり(macOS、rustc 1.100.0 = MSRV、debug)

| 時点 | core 単体 | core 統合 | runtime 単体 | runtime 統合 | converter 単体 | converter 統合 | cli 単体 | cli 統合 | 合計 |
|------|-----------|-----------|--------------|--------------|----------------|----------------|----------|----------|------|
| 基準(23b8975) | 113 | 271 | 70 | 128 | 52 | 64 + 1 無視 | 2 | 457 | **1,157 通過、1 無視** |
| 最後(e4a2bf1) | 112 | 270 | 69 | 128 | 52 | 64 + 1 無視 | 2 | 456 | **1,153 通過、1 無視、0 失敗**(見込みどおり) |

コミットごとの、変えたクレートの件数(失敗は常に 0):

| # | core | runtime | converter | cli |
|---|------|---------|-----------|-----|
| 1 | 383 | 198 | — | — |
| 2 | — | — | — | 459 |
| 3 | — | 198 | — | 459 |
| 4 | 383 | — | — | — |
| 5 | — | 197 | — | — |
| 6 | 382 | — | — | — |
| 7 | — | — | — | 459 |
| 8 | 382 | — | — | 458 |
| 9 | 382 | — | — | 458 |
| 10 | 382 | 197 | — | 458 |
| 11〜15 | 382 | — | — | — |
| 16〜18 | — | — | — | 458 |
| 19 | 382 | 197 | — | 458 |
| 20 | — | 197 | — | 458 |
| 21 | 382 | — | — | 458 |
| 22 | — | — | 116 + 1 無視 | 458 |
| 23 | 382 | — | — | 458 |
| 24 | — | — | 116 + 1 無視 | 458 |

全体の合計は、1 の後 1,156、5 の後 1,155、6 の後 1,154、8 の後 1,153 で、そのあとは変わらない(足したテストは 0。BR8.1)。

## テストの変更

### テスト名の一覧の比較

`cargo test --locked --workspace -- --list 2>/dev/null | sort` を基準(23b8975)と最後(e4a2bf1)で比べた。テスト名の差は、次の 4 つの削除だけ(ほかの差は、各テストのバイナリの件数の行 `N tests, 0 benchmarks` の変化だけ)。名前を変えたテストは無い。

- `types::tests::文字列から_line_を作れる`(core 単体)
- `状態の重ね合わせの既定は空で_再開位置もない`(core `graph_types.rs`)
- `types::tests::耐久性の既定は_durable`(runtime 単体)
- `u1_core_の_sanitize_と同じ結果`(cli `escape_table.rs`)

### 変えたテスト(T 表)

変えたテストのファイルは、承認済みの T1〜T32 の範囲だけ(`git diff --stat 23b8975 HEAD -- 'crates/*/tests/*'` の 22 ファイルと、src の単体テストの core `types.rs`・`test_support.rs`・`error.rs`・`run_id.rs`、runtime `types.rs`・`runner.rs`・`split.rs`)。確かめる中身と期待する出力の文字列は変えていない。

- core(13): src `types.rs`(T1、§4.6)、`test_support.rs`(T2)、`error.rs`(T3)、`run_id.rs`(T4)、`tests/common/mod.rs`(T5)、`graph_common/mod.rs`(T6)、`definition_valid.rs`(T7、T8)、`nfr_security.rs`(T9、T10)、`env_file_parse.rs`(T11)、`env_types.rs`(T12、T13)、`flow_sequences.rs`(T14)、`graph_overlay.rs`(T15)、`graph_types.rs`(T16、§4.6)
- runtime(9): src `types.rs`(T17、§4.6)、`runner.rs`(T18)、`split.rs`(T19)、`tests/common/mod.rs`(T20)、`runner_basic.rs`(T21)、`nfr_robust.rs`(T22)、`nfr_content.rs`(T23)、`nfr_performance.rs`(T24)、`flow_with_state.rs`(T25)
- converter(1): `nfr_security.rs`(T26)
- cli(6): `run_id_arg.rs`(T27)、`args_grammar.rs`(T28)、`signals_real.rs`(T29)、`signals_order.rs`(T30)、`escape_table.rs`(T31、§4.6)、`report_messages.rs`(T32)

T 表の要点: T8 は `definition_valid.rs` の `effective_success_codes() == [0]` の 1 行を消し、もう 1 行を `success_exit_codes == Some(vec![0, 2])` に。T12 は写しを確かめる 1 行を消す。T14 は模型の項目から `PipelineState` を組んで `decide_from_state` に。T17 は 5 行を消す。T21 は `run` の引数を `Command` にし、文字列を渡す 10 か所(Windows でだけコンパイルされる 2 か所を含む)を `Command::Line(..)` で包む。T25 は写しの補助 `decide` を消して `decide_from_state` を直接呼ぶ。T27 は 1 行を消し、1 行を `matches!` に。T28 はファイルの中の補助 `run_id(text)` を置く。

### 消したテスト(§4.6 の 4 件)

| テスト | 消した理由 | 同じことを確かめて残るテスト |
|--------|------------|-------------------------------|
| core `types.rs` の `文字列から_line_を作れる` | `From` を消した(1a) | `definition_valid.rs` の `コマンドの文字列はそのまま保持される`、`nfr_security.rs` の `nfr4_5_…` |
| core `graph_types.rs` の `状態の重ね合わせの既定は空で_再開位置もない` | `StateOverlay` の `Default` を消した(1d-3。D3) | `graph_overlay.rs` の 5 件(T15 で空の値を項目で書く) |
| runtime `types.rs` の `耐久性の既定は_durable` | `Durability` の `Default` を消した(1d-2) | 既定の値の場面が無い。`Durable`・`Fast` の明示は `state_write.rs` と cli の流れのテスト |
| cli `escape_table.rs` の `u1_core_の_sanitize_と同じ結果` | `sanitize` と cli の再公開を消した(1e、2d) | 同じファイルの `期待値の表のとおりにエスケープされ_切り詰められる`、core の単体テスト(T3 の 4 件) |

足したテストは無い(BR8.1)。

## 完了の判定の検索(functional-spec §7。最後のコミット e4a2bf1 の後)

| 検索 | 結果 |
|------|------|
| `From<&str> for Command\|From<String> for Command\|Command::from(\|Into<Command>`(crates) | 0 件 |
| `RunIdArg::as_str\|empty.as_str()\|arg.as_str()`(crates) | 0 件 |
| `is_raised\|impl PartialEq for InterruptSignal\|impl Eq for InterruptSignal`(crates、docs/spec/rust) | 0 件 |
| `JobStatus::default\|Durability::default\|StateOverlay::default\|Language::default`(crates) | 0 件 |
| `#\[default\]`(crates/*/src) | 1 件(cli `types.rs:149` の `GraphFormat`) |
| `sanitize`(crates/core、crates/cli、docs/spec/rust) | 0 件(converter の写しは対象外) |
| `pub fn reason(&self)`(core `envfile/mod.rs`) | 0 件 |
| `decide_before_run`(crates/*/tests、crates/cli、crates/runtime) | 0 件 |
| `pub fn decide_before_run\|pub use classify::classify\|pub fn classify(`(core src) | 0 件 |
| `pub fn is_valid_env_name\|pub fn is_reserved_env_name\|pub fn effective_success_codes\|pub const MAX_RUN_ID_LEN`(crates) | 0 件 |
| `pub use conductor_core::escape_display\|report::escape::escape_display\|escape::{escape_display`(crates) | 0 件 |
| `^pub mod`(cli `lib.rs`) | 3 件(`exit`、`report`、`run_id`) |
| `GraphOptions\|DryRunOptions`(cli `lib.rs`) | 0 件 |
| `RunIdError::`(crates、docs/spec/rust) | 0 件 |
| `-w TooLong\|InvalidChar\|LeadingDot`(crates、docs/spec/rust) | 0 件 |
| `RunIdError.*Empty\|`Empty`[・,、] *`TooLong``(crates、docs/spec/rust) | 0 件 |
| `pub use conductor_core`(runtime src) | 0 件 |
| 誤りの種類の `Display`・`Error` の impl(5 つ) | 0 件 |
| `Hash)]` の次の行の 4 つの enum | 0 件 |
| `conductor_runtime` の `use` の塊の 7 つの名前(awk) | 0 件 |

ほかの確かめ:

- `cargo doc --workspace --no-deps --locked` の警告: core 1、runtime 4 の **5 件**(基準と同じ。この intent と無関係)。途中の 11、13〜18、20、21 の後にも、変えたクレートで確かめ、増えていない。
- `cargo clippy --workspace --all-targets --locked --target x86_64-pc-windows-msvc -- -D warnings`: 通る(`runner_basic.rs` の `cmd_` の Windows のテストを含む)。
- `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`: 通る(行カバレッジ 98.08%)。
- `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、全テスト: 通る。ローカルの rustc は 1.100.0(MSRV)。
- `git diff 23b8975 -- '*Cargo.toml' Cargo.lock`: 空(依存も版も変わらない。converter の `Cargo.toml` に core は無い)。
- 凍結した仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)と `docs/spec/language-agnostic`、`README.md`、`demo`: 差分なし。
- src に足した行に `unsafe`、`cfg(unix)`/`cfg(windows)` は無い。src に足した行の `.expect(` は、runtime `runner.rs` と `split.rs` の `#[cfg(test)]` の単体テストで、元の行の書き換え(T18、T19)だけ。製品のコードには無い。

## 計画からのずれ

- 仕様書 rust 02 の 3.2 のシーケンス(`runtime::apply_skip`、`runtime::initial_state`、`runtime::apply_outcome`、`runtime::apply_interrupt` の 4 行)を `core::` に直した。inventory の 3b の行の一覧に無かったが、再公開が無くなって誤りになるため、同じ項目(コミット 20)で合わせた(仕様書だけ。挙動の記述は変えない)。
- 仕様書 rust 02 の 2.2 の関数の表から 5 行を消すとき、シグネチャは 2.1 の表の `initial_state`/`apply_*`・`check_consistency` の行(「2.2 の表」と書いていた所)に移した(inventory の 3b のとおり)。2.2 の表の下に、使う側は `conductor_core` から使う旨の 1 文を足した。
- 2a で、core の `types.rs:440` と `plan/mod.rs:18` の説明(ふつうの文字の `decide_before_run`)は、crate の中の関数を指して正しいので変えていない(リンクでないので `cargo doc` の警告にもならない)。inventory の一覧にも無い。
- 1e の `escape_table.rs` のモジュールの説明は、8 で `sanitize` の部分を、16 で再公開の部分を消した(T31 を 2 つのコミットに分けて当てた。§8 の割り当てのとおり)。
- 指示に反した点は無い。T1〜T32 と §4.6 の外の既存のテストは変えていない。期待する出力の文字列、テストの名前は変えていない。作業のファイルは `target/drop-fd/` の下だけに置き、終わったあと消した。作業の前からある未コミットの audit の .md はコミットしていない。
