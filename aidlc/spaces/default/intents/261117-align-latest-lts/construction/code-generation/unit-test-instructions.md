# Unit Test Instructions — 最新の Rust に合わせたリファクタリング

## テストの方針

- 方法: test-after(各層を実装してから、その層のテストを書いて実行する)。テスト戦略は Minimal、スコープは refactor。
- 新しく書くテストは 1 件(MSRV の記載の一致を確かめる `crates/cli/tests/msrv_consistency.rs`)。そのほかは、既存のテスト(約 716 件)を変更なしで通す。コードの書き換え(let チェーン化)は挙動を変えないため、既存のテストが検証する。
- テストコードは書き換えない。アサーションは変えない。(FR3.3)

## テストの実行環境(設定)

- テストフレームワークは Rust 標準の `cargo test`。追加の設定ファイルは不要。
- 前提: Rust 1.100.0 以降(`rustc --version` で確認)。

## 実行コマンド(この作業に絞ったもの)

テストの実行は、次のコマンドを、それぞれ単独で実行する。

| 目的 | コマンド |
|---|---|
| 変更前の基準(全体) | `cargo test --workspace --locked` |
| MSRV の記載の一致(新規テスト) | `cargo test -p conductor-cli --test msrv_consistency --locked` |
| conductor-core | `cargo test -p conductor-core --locked` |
| conductor-runtime | `cargo test -p conductor-runtime --locked` |
| conductor-converter | `cargo test -p conductor-converter --locked` |
| conductor-cli | `cargo test -p conductor-cli --locked` |
| 書式 | `cargo fmt --all -- --check` |
| 静的解析(CI の `lint` と同じ) | `cargo clippy --workspace --all-targets --locked -- -D warnings` |

## カバレッジ

- 下限は行カバレッジ 80%(チームの方針と `ci.yml` の `coverage` ジョブ)。下限は変えない。
- 下限は、プロジェクト全体のテストを組み立てた状態で、Build and Test と CI(`cargo llvm-cov --fail-under-lines 80`)が測る。crate ごとの実行では判定しない。
- `cargo-llvm-cov` が手元にある場合だけ、ローカルで実行して値を記録する。

## モック・テストデータ

- モックは使わない。既存のテストが使う補助バイナリ(`src/bin/conductor-*test-helper.rs`)と fixtures(`crates/converter/tests/` の fixtures)をそのまま使う。
- 新規テストは、リポジトリのファイル(`Cargo.toml`、`README.md`、`.github/workflows/ci.yml`)を読み取るだけで、書き込まない。

## OS ごとの扱い

- ローカル(macOS)で実行する。Linux と Windows のテストは、CI で実行する。マージ前に CI を通す。
