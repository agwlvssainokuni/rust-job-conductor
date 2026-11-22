# テストの結果(Test Results): `--dry-run` の模擬の重複をなくすリファクタリング

実行日: 2026-11-22。macOS、`rustc 1.100.0`。コードは `b61d68a` まで。

| 検査 | 結果 |
|------|------|
| `cargo fmt --all -- --check` | 差分なし |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 警告なし |
| `cargo test --locked --workspace` | **1,123 件が成功、失敗 0 件**(この intent の前は 1,121 件。足した 2 本) |
| `cargo llvm-cov ... --fail-under-lines 80 --summary-only` | 行カバレッジ **97.96%**(9,447 行のうち未通過 193)。`flow/record.rs` 98.14%、`plan/simulate.rs` 99.58% |
| 比べる確認(書き換える前の模擬と新しい模擬) | 種 20261122、7、123456789 で各 30 万例、違い 0 件 |
| 性能(終点の後に 999 個のスキップ) | 約 0.1 秒(目安 1 秒。変更の前は 1.13 秒) |
| 依存の差分(`Cargo.toml`、`Cargo.lock`) | なし |

失敗したテスト: なし。
