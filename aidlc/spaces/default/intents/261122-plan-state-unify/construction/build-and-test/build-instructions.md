# ビルドの手順(Build Instructions): `--dry-run` の模擬の重複をなくすリファクタリング

対象: ワークスペース `conductor`。この intent の変更は、core の `flow/record.rs`(runtime から移動)、`flow/mod.rs`、`lib.rs`、`plan/simulate.rs`、runtime の `lib.rs`、`state/mod.rs`、core のテスト 2 ファイル、仕様書 `docs/spec/rust` の 05、02、12 に限られる。

## 前提

- Rust 1.100 以上(MSRV 1.100、edition 2024)。このマシンは `rustc 1.100.0`。`cargo-llvm-cov`。新しい依存は足していない。

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
