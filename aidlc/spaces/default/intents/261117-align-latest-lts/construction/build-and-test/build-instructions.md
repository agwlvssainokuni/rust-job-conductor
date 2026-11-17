# Build Instructions

対象: 最新の Rust に合わせたリファクタリング(`code-generation-plan.md`、`unit-test-instructions.md`、`code-summary.md` の作業の結果)。

## 前提

- Rust 1.100.0 以降(`rustc --version` で確認)。MSRV は `Cargo.toml` の `rust-version = "1.100"`。
- `git`(テストの一部が使う)。
- 外部のサービスや環境変数は不要。

## 依存の取得とビルド

すべて `Cargo.lock` に固定されたまま(`--locked`)で実行する。

| 目的 | コマンド |
|---|---|
| 全ターゲットのビルド | `cargo build --workspace --all-targets --locked` |
| 書式の検査 | `cargo fmt --all -- --check` |
| 静的解析(CI の `lint` と同じ) | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| 依存の確認(`-sys` と C のビルド依存がないこと) | `cargo tree --workspace --locked -e normal,build,dev --prefix none` |

## ビルドの確認

- `cargo build` が成功し、警告がない。
- `cargo clippy ... -D warnings` が警告 0 件で終わる。
- `cargo tree` の出力に、`-sys` で終わるクレート(ホストの macOS では `linux-raw-sys`、`windows-sys` も出ない)と、`cc`、`cmake`、`pkg-config`、`bindgen` がない。

## テストの実行

テストは `unit-test-instructions.md` のコマンドと同じ。

| 目的 | コマンド |
|---|---|
| 全体 | `cargo test --workspace --locked` |
| MSRV の記載の一致 | `cargo test -p conductor-cli --test msrv_consistency --locked` |
| カバレッジ(CI の `coverage` ジョブと同じ) | `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only` |

`cargo-llvm-cov` が無い場合は `cargo install cargo-llvm-cov --locked` で導入する(この作業では導入済みの 0.9.1 を使った)。

## 困ったとき

- `rustc` の版が 1.100 より古い: `rustup update stable` で更新する。MSRV が 1.100 のため、それより古いと `Cargo.toml` の `rust-version` の検査で止まる。
- `--locked` で失敗する: `Cargo.lock` が `Cargo.toml` と食い違っている。この作業では `Cargo.lock` を変更していないため、食い違いが出たら変更の原因を確認する。
- Windows のテストで Ctrl+C を確かめる場合は、`crates/cli/tests/WINDOWS_CTRL_C_CHECKLIST.md` の手順に従う(自動テストでは確かめない)。
