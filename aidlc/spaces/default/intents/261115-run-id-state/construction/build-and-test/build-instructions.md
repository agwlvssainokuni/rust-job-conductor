# Build Instructions

## 前提

- Rust のツールチェイン: 安定版(開発は 1.97)と、最小の対応版 1.85(`rust-version = "1.85"`、edition 2024)。`rustup` で入れる。
- カバレッジの測定: `cargo install cargo-llvm-cov --locked` と `rustup component add llvm-tools-preview`(開発機のみ)。
- 外部のサービス、環境変数、設定ファイルは要らない。C のコンパイラも要らない。

## ビルドと確認

```bash
cargo build --workspace --locked
cargo build --release --locked --bin conductor        # 配布する実行ファイルだけ
target/release/conductor --version                     # conductor 0.5.0
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo +1.85 build --workspace --locked --all-targets   # 最小の Rust。let チェーンは使わない
cargo +1.85 test --workspace --locked
cargo check --workspace --tests --target x86_64-pc-windows-msvc --locked   # Windows 専用のコードの型検査
cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only
```

## run ID の手動の確認(実際のバイナリ)

```bash
# 作業ディレクトリ(例: target/runid-check/)に、複数のジョブを持つ定義 c.toml を置く
conductor pipeline --run-id r1 p &     # r1 と r2 を並行して動かす
conductor pipeline --run-id r2 p &
wait
ls c.toml.*.state                       # 止まった run だけ、c.toml.<run-id>.state が残る
conductor watch --run-id r1 --config c.toml
conductor reset --all --config c.toml   # その定義のステートを、すべて消す
```

## README の文書チェック(CI の `docs-check` と同じ)

```bash
for kw in 'SHA-256' 'Gatekeeper' '終了コードの表' 'ステートファイルの場所' 'at-least-once' 'シグナル' 'convert'; do
  grep -q -- "$kw" README.md || echo "MISSING $kw"
done
```

何も出力しないこと。

## デモの確認

```bash
cargo build --release --locked --bin conductor         # 先に、新しい版を作る
DEMO_SLEEP=0.1 sh demo/run-demo.sh                      # 終了コード 0
```

## よくあるビルドの問題

| 症状 | 原因と対処 |
|------|-----------|
| `error[E0658]: let expressions in this position are unstable` | 1.85 で let チェーンを使っている。`is_some_and` などに書き換える |
| デモが古い版で動く | `target/release/conductor` が古い。作り直す |
| `docs-check` が失敗する | README から必須の語句が消えた。上の検索で、どれが無いかを確かめる |
| ステートが「壊れている(65)」と出る | run ID の食い違い(ファイル名と中身)や、古い形式。`conductor reset --run-id <ID>` で、個別に消す |
