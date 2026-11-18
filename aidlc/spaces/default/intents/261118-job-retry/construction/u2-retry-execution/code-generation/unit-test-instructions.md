# Unit Test Instructions — u2-retry-execution

## テストの方針

- 方法: test-after(各層を実装してから、その層のテストを書いて実行する)。テスト戦略は Standard(コンポーネントごとに 5〜8 件程度、主要な境界に統合テスト)、スコープは feature(行カバレッジ 80% の下限と、CI 実行を加える)
- 新しいテストは、`crates/core/tests/retry_flow.rs`(約 15 件)、`crates/runtime/tests/wait_interruptible.rs`(約 6 件)、`crates/cli/tests/flow_retry.rs`(約 20 件)の 3 ファイルと、`crates/cli/tests/report_messages.rs` への追加(約 4 件)。既存のテストは、アサーションを変えずに通す
- テストは、Rust 標準の `cargo test`。追加の設定は不要。テスト名は、既存に合わせて日本語でよい

## 実行環境

- Rust 1.100.0 以降(`rustc --version`)

## この Unit に絞った実行コマンド

| 目的 | コマンド |
|---|---|
| 変更前の基準(全体) | `cargo test --workspace --locked` |
| 純粋な判断(core) | `cargo test -p conductor-core --test retry_flow --locked` |
| 待機(runtime) | `cargo test -p conductor-runtime --test wait_interruptible --locked` |
| cli の文言 | `cargo test -p conductor-cli --test report_messages --locked` |
| cli のリトライの流れ | `cargo test -p conductor-cli --test flow_retry --locked` |
| 書式 | `cargo fmt --all -- --check` |
| 静的解析(CI の `lint` と同じ) | `cargo clippy --workspace --all-targets --locked -- -D warnings` |

## カバレッジ

- 下限は行カバレッジ 80%(チームの方針と CI)。この下限は変えない
- 下限は、プロジェクト全体のテストを組み立てた状態で、Build and Test と各 Unit の完了の確認(検査コマンドの `cargo llvm-cov`)が測る。個別のテストの実行では判定しない
- 新しいコード(`flow/retry.rs`、`wait.rs`、`cli/src/retry.rs`)の行は、新規のテストで、各分岐を含めて通ることを目標にする

## モックとテストデータ

- 待機は、偽の `Runtime`(`ScriptedRuntime`、`MemRuntime`)が、待たずに、呼ばれた待ち時間を記録する。指定の回数目に `Interrupted` を返せる。これで、間隔が 1 時間のリトライも、実際には待たずに試せる
- 実時間を使うのは、`wait_interruptible.rs` の短い待機(最大でも 1 秒程度)だけにする
- 定義は、テストの中の文字列(TOML)として書く。既存の `crates/cli/tests/common/mod.rs` の補助に合わせる

## OS ごとの扱い

- ローカル(macOS)で実行する。Linux と Windows は、CI で実行する(マージ前に CI を通す)
