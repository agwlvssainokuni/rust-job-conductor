# Test Results

実行日: 2026-11-17。実行環境: macOS(aarch64)、rustc 1.100.0、cargo-llvm-cov 0.9.1。ブランチ: `main`(コミット 3db0424 の時点のコードで実行)。

## ビルド

| コマンド | 結果 |
|---|---|
| `cargo build --workspace --all-targets --locked` | 成功(警告なし) |
| `cargo fmt --all -- --check` | 差分なし(終了コード 0) |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 警告 0 件(終了コード 0) |
| `cargo tree --workspace --locked -e normal,build,dev --prefix none` | `-sys` クレート、`cc`、`cmake`、`pkg-config`、`bindgen` は出ない |

## テスト

`unit-test-instructions.md` の全体のコマンドを 1 回だけ実行した。

| コマンド | 実行 | 成功 | 失敗 | 無視 |
|---|---|---|---|---|
| `cargo test --workspace --locked` | 705 | 704 | 0 | 1 |

- 変更前(基準): 成功 701、失敗 0、無視 1(開発担当が変更前のツリーで測った値)。変更後の 704 は、基準に `msrv_consistency.rs` の新規 3 件を足した数。
- 無視の 1 件は既存のもの(今回の変更とは無関係)。
- 失敗したテストはない。
- 計画と調査の記録にある「約 716 件」とは件数が合わない。この実行で数えた件数は上の 705 件で、差の理由は確かめていない(件数の見積もりの誤差とみられるが、確認はしていない)。

## カバレッジ

コマンド(CI の `coverage` ジョブと同じ):
`cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`

| 指標 | 値 |
|---|---|
| 行カバレッジ(TOTAL) | **91.81%**(7256 行中、未実行 594 行) |
| 関数カバレッジ | 93.77% |
| 領域カバレッジ | 69.59%(参考。下限の対象外) |
| 下限(`--fail-under-lines`) | 80%。満たしている(終了コード 0) |

出力の全文は、この実行のツール出力として保存されている。下限は変更していない。

## 実行していない検査

- Linux・Windows 上のテストと、macOS の CI ランナー上のテスト: ローカルは macOS のため。CI で実行する。
- CI の `build-check` と `msrv-test` の `@1.100.0` のジョブ: プッシュ後の CI でしか確かめられない。この時点では CI は実行されていない。
- Windows の Ctrl+C の手動確認(`WINDOWS_CTRL_C_CHECKLIST.md`): 自動化の対象外で、今回の変更とも関係しない。
