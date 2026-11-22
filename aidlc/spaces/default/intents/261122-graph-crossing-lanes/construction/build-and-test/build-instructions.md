# ビルドの手順(Build Instructions): graph の text 表示の改善 その 3

対象: ワークスペース `conductor`。この intent の変更は、core の `graph/layout.rs` と `graph/mod.rs`、cli の `graph_text.rs`(`_` の 1 文字)、テスト、README、仕様書。

## 前提

- Rust 1.100 以上(MSRV 1.100、edition 2024)。このマシンは `rustc 1.100.0`。`cargo-llvm-cov`(`llvm-tools-preview`)。新しい依存は足していない。

## コマンド

```bash
cargo build --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --locked --workspace
cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only
```

README の必須語句(CI の `docs-check`): `SHA-256`、`Gatekeeper`、`終了コードの表`、`ステートファイルの場所`、`at-least-once`、`シグナル`、`convert`。

## このマシンで確かめられないもの

- Windows と ubuntu のテスト、`msrv-test`(rustc が MSRV と同じ版なので、ここでの結果が兼ねる): CI(`ci.yml`。push で動く)で確かめる。

## よくある問題

- 計測つきの実行は遅く、マシンが忙しいと、時間のかかるテストが 30 秒の上限を超えることがある。単独で再実行すると通る。
