# Test Results

実行日: 2026-11-15。環境: macOS(Darwin)、Rust 1.97(安定版)と 1.85.1。

## ビルドと静的検査

| 命令 | 終了コード | 結果 |
|------|-----------|------|
| `cargo build --release --locked --bin conductor` | 0 | 成功。971,152 バイト。`conductor --version` は `conductor 0.2.0` |
| `cargo fmt --all -- --check` | 0 | 差分なし |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 | 警告 0 |
| `cargo +1.85 build --workspace --locked --all-targets` | 0 | 成功 |
| `cargo check --workspace --tests --target x86_64-pc-windows-msvc --locked` | 0 | 成功(Windows 専用のテストの型検査) |
| `cargo tree -p conductor-cli -e normal,build --locked` の C ビルド依存の検索 | 1(該当なし) | `cc`・`cmake`・`pkg-config`・`bindgen` はない |

## テスト

命令: `cargo test --workspace --locked`(安定版)と `cargo +1.85 test --workspace --locked`。どちらも終了コード 0。合計 580 件成功、失敗 0、無視 1(converter の以前からのもの)。

| パッケージ | 成功 | 失敗 | 前回(0.1.0)との差 |
|-----------|------|------|-------------------|
| conductor-core | 122 | 0 | +7 |
| conductor-runtime | 157 | 0 | +5 |
| conductor-converter | 114 | 0 | +2 |
| conductor-cli | 187 | 0 | +2 |

- 合計 564 → 580(+16)。既存のテストの修正は、型の変更に伴う署名の追従だけ。Windows 専用のテスト 1 件(`cmd_への配列の危険な引数は注入にならない`)は、`cfg(windows)` のため、この環境では実行せず、型検査まで。

## カバレッジ

命令: `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --summary-only`。終了コード 0。行 97.79%(6,285 行のうち未実行 139)、関数 97.97%、リージョン 97.61%。下限 80% を満たす(前回 97.66%)。

## デモ

`DEMO_SLEEP=0.1 sh demo/run-demo.sh`: 終了コード 0。全 6 場面が完走。配列の `command` のジョブ(`notify`)を含む。(0.2.0 の `conductor` で実行)
