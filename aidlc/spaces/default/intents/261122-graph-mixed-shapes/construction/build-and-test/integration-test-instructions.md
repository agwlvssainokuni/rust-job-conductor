# 統合テストの手順(Integration Test Instructions): graph の text 表示の改善 その 4

Minimal の戦略。core の配置から、cli の整形、実バイナリまでの 1 本の流れ。

| 境界 | 確認すること | ファイル |
|------|--------------|----------|
| core の配置 → cli の整形 | M2、M4(`_` を含む)の text の全文、`-v`、`--state`。辺の一覧に残る形(両方の飛び越しを含む交差)の注記と一覧 | `crates/cli/tests/graph_report.rs` |
| 定義 → 実バイナリ | 交差と同じ行き先が混ざる定義で `conductor graph` を動かし、M2 の図を確かめる。辺の一覧に残る形の注記 | `crates/cli/tests/binary_graph.rs` |
| README ↔ 実出力 | README の M2 の例が、実出力と一致する | `crates/cli/tests/docs_graph.rs` |
| 正解データ ↔ 配置 | 487 例(混ざる形)、940 例(交差)、1,517 例(同じ行き先)、537 例(通常)は git の出力、871 例(分岐)は規則から導いた図が、1 文字も違わず一致する | `crates/core/tests/graph_layout_oracle.rs` |

実行:

```bash
cargo test --locked -p conductor-cli --test graph_report --test binary_graph --test docs_graph
cargo test --locked -p conductor-core --test graph_layout_oracle --test graph_drawable --test graph_lane_polish --test graph_branch_lanes --test graph_crossing_lanes --test graph_mixed_shapes --test graph_perf
cargo test --locked --workspace
```

カバレッジの目標: プロジェクト全体の行カバレッジ 80% 以上。
