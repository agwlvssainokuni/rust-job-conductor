# Test Results

実行日: 2026-11-15。環境: macOS(Darwin)、Rust 1.97(安定版)と 1.85.1。

## ビルドと静的検査

| 命令 | 終了コード | 結果 |
|------|-----------|------|
| `cargo build --release --locked --bin conductor` | 0 | 成功。`conductor --version` は `conductor 0.4.0` |
| `cargo fmt --all -- --check` | 0 | 差分なし |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 | 警告 0 |
| `cargo +1.85 build --workspace --locked --all-targets` | 0 | 成功 |
| `cargo check --workspace --tests --target x86_64-pc-windows-msvc --locked` | 0 | 成功 |
| README の必須の語句の検索(`docs-check` と同じ 7 語句) | 0 | すべてある |

## テスト

命令: `cargo test --workspace --locked`(安定版)と `cargo +1.85 test --workspace --locked`。どちらも終了コード 0。合計 597 件成功、失敗 0、無視 1(converter の以前からのもの)。

| パッケージ | 成功 | 失敗 | 前回(0.3.0)との差 |
|-----------|------|------|-------------------|
| conductor-core | 122 | 0 | 0 |
| conductor-runtime | 162 | 0 | 0 |
| conductor-converter | 114 | 0 | 0 |
| conductor-cli | 199 | 0 | +12 |

- 合計 585 → 597(+12)。改名(`jobs` → `job`)だけの段階で、585 件が、そのまま成功した(動作が変わっていない確認)。

## カバレッジ

命令: `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --summary-only`。終了コード 0。行 97.79%(6,344 行のうち未実行 140)、関数 97.99%、リージョン 97.60%。下限 80% を満たす(前回 97.80%)。

## 実際のバイナリでの確認(Code Generation の中で実施)

| 場合 | 終了コード | 出力 |
|------|-----------|------|
| 正しい定義 | 0 | なし |
| `-v` | 0 | 標準エラーに 1 行(`definition is valid (2 jobs)` / `定義は正しいです(ジョブ 2 個)`) |
| `command` の引用符が閉じていない | 64 | 誤りの行(入力の値は含まない) |
| 遷移先が不正 | 64 | 誤りの行 |
| パイプライン名が一致 / 不一致 | 0 / 64 | なし / 誤りの行 |
| 位置引数が 2 つ | 68 | 使い方の誤り |
| `--help` | 0 | `validate` の記載あり |

- `validate` は、`.state` を作らない。壊れた `.state` が隣にあっても、正しい定義なら 0 を返す(同じ状態で `run` は 65)。

## デモ

`DEMO_SLEEP=0.1 sh demo/run-demo.sh`: 終了コード 0(0.4.0 の `conductor` で実行)。

## 内部の名前

`PipelineDefinition` の Rust のフィールドに `jobs` は残っていない(`pub jobs` の検索で該当なし)。
