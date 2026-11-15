# Build Instructions

## 前提

- Rust のツールチェイン: 安定版(開発は 1.97)と、最小の対応版 1.85(`rust-version = "1.85"`、edition 2024)。`rustup` で入れる。
- カバレッジの測定: `cargo install cargo-llvm-cov --locked` と `rustup component add llvm-tools-preview`(開発機のみ)。
- 外部のサービス、環境変数、設定ファイルは要らない。C のコンパイラも要らない(`-sys` クレートを含まない)。

## ビルド

```bash
cargo build --workspace --locked
cargo build --release --locked --bin conductor     # 配布する実行ファイルだけ
target/release/conductor --version                  # conductor 0.2.0
```

## 最小の Rust(1.85)の確認

```bash
cargo +1.85 build --workspace --locked --all-targets
cargo +1.85 test --workspace --locked
```

- let チェーン(`if let ... && ...`)は 1.88 以降。使わない。

## 整形と静的検査

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
```

## Windows 向けの型検査(Windows 専用のコードを含む)

```bash
cargo check --workspace --tests --target x86_64-pc-windows-msvc --locked
```

## 依存の確認

```bash
cargo tree -p conductor-cli -e normal,build --locked | grep -E '(^|[ │├└─])(cc|cmake|pkg-config|bindgen) v'
```

何も出力しないこと。

## デモの確認

```bash
DEMO_SLEEP=0.1 sh demo/run-demo.sh      # 終了コード 0。配列の command のジョブ(notify)を含む
```

## よくあるビルドの問題

| 症状 | 原因と対処 |
|------|-----------|
| `error[E0658]: let expressions in this position are unstable` | 1.85 で let チェーンを使っている。`is_some_and` などに書き換える |
| デモが古い版で動く | `target/release/conductor` が古い。`cargo build --release --locked --bin conductor` で作り直す |
