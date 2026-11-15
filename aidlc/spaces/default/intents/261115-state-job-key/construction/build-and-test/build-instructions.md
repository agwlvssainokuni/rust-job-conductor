# Build Instructions

## 前提

- Rust のツールチェイン: 安定版(開発は 1.97)と、最小の対応版 1.85(`rust-version = "1.85"`、edition 2024)。`rustup` で入れる。
- カバレッジの測定: `cargo install cargo-llvm-cov --locked` と `rustup component add llvm-tools-preview`(開発機のみ)。
- 外部のサービス、環境変数、設定ファイルは要らない。C のコンパイラも要らない。

## ビルドと確認

```bash
cargo build --workspace --locked
cargo build --release --locked --bin conductor        # 配布する実行ファイルだけ
target/release/conductor --version                     # conductor 0.3.0
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo +1.85 build --workspace --locked --all-targets   # 最小の Rust。let チェーンは使わない
cargo +1.85 test --workspace --locked
cargo check --workspace --tests --target x86_64-pc-windows-msvc --locked   # Windows 専用のコードの型検査
```

## デモの確認

```bash
cargo build --release --locked --bin conductor         # 先に、新しい版を作る(古い版で動かないため)
DEMO_SLEEP=0.1 sh demo/run-demo.sh                      # 終了コード 0
```

## よくあるビルドの問題

| 症状 | 原因と対処 |
|------|-----------|
| `error[E0658]: let expressions in this position are unstable` | 1.85 で let チェーンを使っている。`is_some_and` などに書き換える |
| デモが古い版で動く | `target/release/conductor` が古い。作り直す |
| 古い `.state`(`[[jobs]]`)が「壊れたステート」(65)になる | v0.3.0 でステートのキーが `[[job]]` に変わった(互換なし)。`conductor reset` で消してやり直す |
