# 統合テストの手順(Integration Test Instructions): graph の text 表示の改善 その 3

Minimal の戦略。core の配置から、cli の整形、実バイナリまでの 1 本の流れ。

| 境界 | 確認すること | ファイル |
|------|--------------|----------|
| core の配置 → cli の整形 | 交差の text の全文(C1、`_` を含む C2)、`-v`、`--state` | `crates/cli/tests/graph_report.rs` |
| 定義 → 実バイナリ | 交差の定義で `conductor graph` を動かし、C1 の図を確かめる。辺の一覧に残る形(交差と同じ行き先)の注記 | `crates/cli/tests/binary_graph.rs` |
| README ↔ 実出力 | README の C1 の例が、実出力と一致する | `crates/cli/tests/docs_graph.rs` |
| git の出力 ↔ 配置 | 交差の 940 例と、既存の 537、1,517 例(git の出力)、871 例(規則から導いた図)が、1 文字も違わず一致する | `crates/core/tests/graph_layout_oracle.rs` |

実行:

```bash
cargo test --locked -p conductor-cli --test graph_report --test binary_graph --test docs_graph
cargo test --locked -p conductor-core --test graph_layout_oracle --test graph_drawable --test graph_lane_polish --test graph_branch_lanes --test graph_crossing_lanes --test graph_perf
cargo test --locked --workspace
```

カバレッジの目標: プロジェクト全体の行カバレッジ 80% 以上。
