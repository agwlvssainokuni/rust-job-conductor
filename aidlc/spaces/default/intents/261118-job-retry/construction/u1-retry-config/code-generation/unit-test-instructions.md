# Unit Test Instructions — u1-retry-config

## テストの方針

- 方法: test-after(各層を実装してから、その層のテストを書いて実行する)。テスト戦略は Standard(コンポーネントごとに 5〜8 件程度、主要な境界に統合テスト)、スコープは feature(行カバレッジ 80% の下限と、CI 実行を加える)
- 新しいテストは、`crates/core/tests/definition_retry.rs`(統合テスト、約 30 件)と、既存のファイルへの追加(converter の往復変換 1〜2 件、cli の `validate` と文言で約 4 件)。既存のテストは、アサーションを変えずに通す
- テストは、Rust 標準の `cargo test`。追加の設定は不要。テスト名は、既存に合わせて日本語でよい

## 実行環境

- Rust 1.100.0 以降(`rustc --version`)

## この Unit に絞った実行コマンド

| 目的 | コマンド |
|---|---|
| 変更前の基準(全体) | `cargo test --workspace --locked` |
| core(新規の統合テストを含む) | `cargo test -p conductor-core --locked` |
| core の新規テストだけ | `cargo test -p conductor-core --test definition_retry --locked` |
| converter の往復変換 | `cargo test -p conductor-converter --test convert_roundtrip --locked` |
| cli の `validate` の流れ | `cargo test -p conductor-cli --test flow_validate --locked` |
| cli の文言 | `cargo test -p conductor-cli --test report_messages --locked` |
| 書式 | `cargo fmt --all -- --check` |
| 静的解析(CI の `lint` と同じ) | `cargo clippy --workspace --all-targets --locked -- -D warnings` |

## カバレッジ

- 下限は行カバレッジ 80%(チームの方針と CI)。この下限は変えない
- 下限は、プロジェクト全体のテストを組み立てた状態で、Build and Test と各 Unit の完了の確認(検査コマンドの `cargo llvm-cov`)が測る。個別のテストの実行では判定しない
- 新しい検証のコード(`walk.rs` の `retry` の検査)の行は、新規のテストですべて通る(誤りの各分岐を含む)ことを目標にする

## モックとテストデータ

- モックは使わない。定義は、テストの中の文字列(TOML)として書く。既存の `crates/core/tests/` の書き方と補助(あれば)に合わせる
- 境界値の組(N=1/0/101、間隔 `"1s"`/`"0s"`/`"25h"`、倍率 2/1/11、終了コード 1/0/256)を、テストの表にまとめる

## OS ごとの扱い

- ローカル(macOS)で実行する。Linux と Windows は、CI で実行する(マージ前に CI を通す)
