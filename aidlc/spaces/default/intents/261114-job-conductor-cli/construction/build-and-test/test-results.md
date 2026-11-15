# Test Results

実行日: 2026-11-15。環境: macOS(Darwin)、Rust 1.97(安定版)と 1.85.1。

## ビルドと静的検査

| 命令 | 終了コード | 結果 |
|------|-----------|------|
| `cargo build --workspace --locked`(1.97) | 0 | 成功 |
| `cargo build --release --locked --bin conductor` | 0 | 成功。`conductor` は 971,056 バイト |
| `cargo fmt --all -- --check` | 0 | 差分なし |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 | 警告 0 |
| `cargo +1.85 build --workspace --locked` | 101 | **失敗**。`crates/u1-core/src/definition/walk.rs:160` と `:193` の let チェーン(`error[E0658]: let expressions in this position are unstable`) |

- 失敗の診断: 1.85 の最小のバージョンでは、let チェーン(1.88 で安定化)を使えない。2 か所を `Option::is_some_and` に書き換えたコピーで、`cargo +1.85 build --workspace --locked --all-targets` が成功した(コピーはプロジェクトの外で試した。プロジェクトのソースは未変更)。

## テスト

命令: `cargo test --workspace --locked`。終了コード 0。合計 564 件成功、失敗 0、無視 1(u1 の以前からのもの)。

| パッケージ | 成功 | 失敗 |
|-----------|------|------|
| conductor-core(u1) | 115 | 0 |
| conductor-runtime(u2) | 152 | 0 |
| conductor-converter(u4) | 112 | 0 |
| conductor-cli(u3) | 185 | 0 |

## カバレッジ

命令: `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --summary-only`。終了コード 0。

- 全体: 行 97.66%(6,206 行のうち未実行 145)、関数 97.94%、リージョン 97.56%。下限 80% を満たす。
- 最小のファイル: `u3-cli/src/aux.rs` 91.18%、`u4-converter/src/resolve.rs` 91.11%、`u3-cli/src/pipeline.rs` 91.23%。すべてのファイルが 91% 以上のため、各 Unit の範囲でも 80% を満たす。
- `cfg(windows)` のコード(`platform/windows.rs` など)は、macOS の測定に入らない。

## 依存

- `cargo tree -p conductor-cli -e normal,build` に `cc`・`cmake`・`pkg-config`・`bindgen` と `-sys` の依存はない。

## 性能(u3-cli、macOS の debug ビルド。`unit` の測定)

`run` のオーバーヘッド p95 21.4 ms(基準 100 ms)、`pipeline` のジョブ 1 つあたり p95 24.9 ms(目標 50 ms)、ジョブ 1,000 個の定義の `run` 151 ms(基準 1 秒)、CPU 0.01 秒(基準 0.1 秒)。すべて基準内。

## 修復後の再実行(loop-back 1 のあと)

| 命令 | 終了コード | 結果 |
|------|-----------|------|
| `cargo +1.85 build --workspace --locked --all-targets` | 0 | 成功 |
| `cargo +1.85 test --workspace --locked` | 0 | 564 件成功、失敗 0 |
| `cargo test --workspace --locked`(1.97) | 0 | 564 件成功、失敗 0 |
| `cargo llvm-cov --workspace ...` | 0 | 行 97.66% |
| `cargo fmt --all -- --check`、`cargo clippy ... -D warnings` | 0、0 | 問題なし |

## Loop-Back Log

### Loop-back 1 — 2026-11-15T01:30:00Z

- **Diagnosis:** 最小の Rust のバージョン 1.85 で、ワークスペースがビルドできない。`crates/u1-core/src/definition/walk.rs` の 160 行目と 193 行目が let チェーンを使っている(1.88 で安定化)。u2・u3・u4 には同じ構文はない(1.85 の全ビルドで、エラーはこの 2 か所だけ)。
- **Root-cause stage:** code-generation(`u1-core`)。nfr-requirements の rust-version 1.85 の目標に、実装が反している。
- **Planned fix:** `u1-core` だけを Code Generation に戻し、2 か所を `Option::is_some_and` に書き換える(動作は変えない)。最小のバージョンを上げる案(1.88 に引き上げ)は、利用者のツールチェインの条件を変えるため、採らない。
- **Estimated impact:** 書き換えは約 10 行。工数は小さい(テストの再実行を含めて 1 時間未満)。費用はなし。リスクは低い(`is_some_and` は 1.70 から。既存の `u1-core` のテスト 115 件で確かめる)。Plan Approval と review が `u1-core` の分だけ、もう一度要る。
