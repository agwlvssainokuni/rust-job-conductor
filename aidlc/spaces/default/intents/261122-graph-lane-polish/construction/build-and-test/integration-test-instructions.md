# 統合テストの手順(Integration Test Instructions): graph の text 表示の改善

Minimal の戦略。境界をまたぐ確認は、core の配置から、cli の整形、実バイナリまでの 1 本の流れだけ。

| 境界 | 確認すること | ファイル |
|------|--------------|----------|
| core の配置 → cli の整形 | 同じ行き先(S1)と両方の飛び越し(D1)の text の全文。`-v`、`--state` を重ねた形。英語の注記の全文 | `crates/cli/tests/graph_report.rs` |
| 定義 → 実バイナリ | `conductor graph` を、同じ行き先と両方の飛び越しの定義で動かし、図を確かめる。交差は辺の一覧と注記 | `crates/cli/tests/binary_graph.rs` |
| README ↔ 実出力 | README の S1 と D1 の例が、実出力と一致する | `crates/cli/tests/docs_graph.rs` |
| git の出力 ↔ 配置 | 537 例と 1,517 例(`git log --graph` の実際の出力)が、1 文字も違わず一致する | `crates/core/tests/graph_layout_oracle.rs` |

実行:

```bash
cargo test --locked -p conductor-cli --test graph_report --test binary_graph --test docs_graph
cargo test --locked -p conductor-core --test graph_layout_oracle --test graph_drawable --test graph_lane_polish
cargo test --locked --workspace
```

カバレッジの目標: プロジェクト全体の行カバレッジ 80% 以上。
