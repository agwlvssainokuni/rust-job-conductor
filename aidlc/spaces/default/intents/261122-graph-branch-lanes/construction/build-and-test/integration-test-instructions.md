# 統合テストの手順(Integration Test Instructions): graph の text 表示の改善 その 2

Minimal の戦略。core の配置から、cli の整形、実バイナリまでの 1 本の流れ。

| 境界 | 確認すること | ファイル |
|------|--------------|----------|
| core の配置 → cli の整形 | 分岐(B1)の text の全文、`-v`、`--state` | `crates/cli/tests/graph_report.rs` |
| 定義 → 実バイナリ | 分岐の定義で `conductor graph` を動かし、B1 の図を確かめる | `crates/cli/tests/binary_graph.rs` |
| README ↔ 実出力 | README の分岐の例が、実出力と一致する | `crates/cli/tests/docs_graph.rs` |
| 規則 ↔ 配置 | 分岐の正解データ 871 例(規則から導いた図)と、既存の 537 例、1,517 例(git の出力)が、1 文字も違わず一致する | `crates/core/tests/graph_layout_oracle.rs` |

実行:

```bash
cargo test --locked -p conductor-cli --test graph_report --test binary_graph --test docs_graph
cargo test --locked -p conductor-core --test graph_layout_oracle --test graph_drawable --test graph_lane_polish --test graph_branch_lanes
cargo test --locked --workspace
```

カバレッジの目標: プロジェクト全体の行カバレッジ 80% 以上。
