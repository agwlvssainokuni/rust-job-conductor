# ビルドの手順(Build Instructions): run と pipeline の重複をなくすリファクタリング

対象: ワークスペース `conductor`。この intent の変更は、cli の `step.rs`(新規)、`run.rs`、`pipeline.rs`、`lib.rs`、cli の流れのテスト 2 ファイル、仕様書 `docs/spec/rust` の 05 と 02 に限られる。

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

## このマシンで確かめられないもの

- Windows と ubuntu のテスト: CI(`ci.yml`。push で動く)で確かめる。`msrv-test` は、rustc が MSRV と同じ版なので、ここでの結果が兼ねる。
