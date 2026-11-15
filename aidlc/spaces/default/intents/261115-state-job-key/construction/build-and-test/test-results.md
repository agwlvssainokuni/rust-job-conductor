# Test Results

実行日: 2026-11-15。環境: macOS(Darwin)、Rust 1.97(安定版)と 1.85.1。

## ビルドと静的検査

| 命令 | 終了コード | 結果 |
|------|-----------|------|
| `cargo build --release --locked --bin conductor` | 0 | 成功。`conductor --version` は `conductor 0.3.0` |
| `cargo fmt --all -- --check` | 0 | 差分なし |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 | 警告 0 |
| `cargo +1.85 build --workspace --locked --all-targets` | 0 | 成功 |
| `cargo check --workspace --tests --target x86_64-pc-windows-msvc --locked` | 0 | 成功 |

## テスト

命令: `cargo test --workspace --locked`(安定版)と `cargo +1.85 test --workspace --locked`。どちらも終了コード 0。合計 585 件成功、失敗 0、無視 1(converter の以前からのもの)。

| パッケージ | 成功 | 失敗 | 前回(0.2.0)との差 |
|-----------|------|------|-------------------|
| conductor-core | 122 | 0 | 0 |
| conductor-runtime | 162 | 0 | +5 |
| conductor-converter | 114 | 0 | 0 |
| conductor-cli | 187 | 0 | 0 |

- 合計 580 → 585(+5)。ステートの TOML を持つ既存のテストは、キー名の修正だけ。

## カバレッジ

命令: `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --summary-only`。終了コード 0。行 97.80%(6,310 行のうち未実行 139)、関数 97.98%、リージョン 97.62%。下限 80% を満たす(前回 97.79%)。

## 実際のバイナリでの確認(Code Generation の中で実施)

- 失敗するジョブで `conductor pipeline` を実行(終了コード 2)すると、`.state` が `[[job]]` と `format_version = 1` で書かれ、`jobs` はない。`conductor watch` が読める(終了コード 0)。
- 手書きの `[[jobs]]` のステートは、`watch` と `pipeline` のどちらでも、終了コード 65(壊れたステート)。メッセージに入力の値は出ない。

## デモ

`DEMO_SLEEP=0.1 sh demo/run-demo.sh`: 終了コード 0(0.3.0 の `conductor` で実行)。
