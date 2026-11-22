# テストの結果(Test Results): graph の配置のリファクタリング

実行日: 2026-11-22。macOS、`rustc 1.100.0`。最終のコミット `b3aff70`(コードは `09686ff` まで)。

| 検査 | 結果 |
|------|------|
| `cargo fmt --all -- --check` | 差分なし |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 警告なし |
| `cargo test --locked --workspace` | **1,118 件が成功、失敗 0 件**(この intent の前は 1,091 件) |
| `cargo llvm-cov ... --fail-under-lines 80 --summary-only` | 行カバレッジ **97.99%**(9,482 行のうち未通過 191)。`graph/layout.rs` は 99.65%、`report/graph_text.rs` は 99.45% |

## 正解データ(7 つ、計 7,400 例。すべて 1 文字も違わず一致)

| 正解データ | 例の数 | 出どころ | 今回 |
|------------|--------|----------|------|
| `graph_layout_oracle.md` | 537 | git の出力 | 変更なし |
| `graph_layout_oracle_same_target.md` | 1,517 | git の出力 | 変更なし |
| `graph_layout_oracle_cross.md` | 940 | git の出力 | 変更なし |
| `graph_layout_oracle_mixed.md` | 487 | git の出力 | 変更なし |
| `graph_layout_oracle_branch.md` | 871 | 規則から導いた | 第 2 版に差し替え(列 0 の規則) |
| `graph_layout_oracle_offmain.md` | 2,748 | 規則から導いた | 新規(ジョブ 3〜5 個の全形) |
| `graph_layout_oracle_offmain_67.md` | 300 | 規則から導いた | 新規(ジョブ 6〜7 個の抜き出し) |

## 主なテストのファイル

- core: `graph_layout_oracle.rs`(11)、`graph_layout_unify.rs`(新規 10: N1〜N5、B1、D1、主線の記号)、`graph_perf.rs`(6)、`graph_drawable.rs`、`graph_lane_polish.rs`、`graph_branch_lanes.rs`、`graph_crossing_lanes.rs`、`graph_mixed_shapes.rs`
- cli: `graph_report.rs`(28)、`binary_graph.rs`(10)、`docs_graph.rs`(8)

失敗したテスト: なし。
