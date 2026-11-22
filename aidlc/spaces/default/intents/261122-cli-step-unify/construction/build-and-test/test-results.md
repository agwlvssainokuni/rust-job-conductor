# テストの結果(Test Results): run と pipeline の重複をなくすリファクタリング

実行日: 2026-11-22。macOS、`rustc 1.100.0`。コードは `d9f3823` まで。

| 検査 | 結果 |
|------|------|
| `cargo fmt --all -- --check` | 差分なし |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 警告なし |
| `cargo test --locked --workspace` | **1,121 件が成功、失敗 0 件**(この intent の前は 1,118 件) |
| `cargo test --locked -p conductor-cli` | 変更の前 440 件、後 443 件(既存のテストは無変更、新規 3 件) |
| `cargo llvm-cov ... --fail-under-lines 80 --summary-only` | 行カバレッジ **97.99%**(9,465 行のうち未通過 190)。`step.rs` 98.84%、`run.rs` 90.91%、`pipeline.rs` 88.89% |
| 依存の差分(`Cargo.toml`、`Cargo.lock`) | なし |

失敗したテスト: なし。
