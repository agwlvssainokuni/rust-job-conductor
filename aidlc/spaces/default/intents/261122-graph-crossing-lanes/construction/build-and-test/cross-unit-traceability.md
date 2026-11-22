# ユニットをまたぐ追跡(Cross-Unit Traceability): graph の text 表示の改善 その 3

要件の分析とユニットの分割は省いた(ユニットは 1 つ)。追跡の ID は、機能の設計の規則(`rules.md` の BR)で、`construction/code-generation/traceability.json` が正本。

| 規則 | 内容 | 実装 | テスト |
|------|------|------|--------|
| BR1.1 | 交差を図にする範囲(グラフ全体の条件) | `crates/core/src/graph/layout.rs` | `graph_crossing_lanes.rs`、`graph_drawable.rs` |
| BR1.2〜BR1.4 | 線の並び、合流の行(k = 0)、交差の行(k ≥ 1。横線 `_`) | `layout.rs`、`graph/mod.rs`、`graph_text.rs` | `graph_layout_oracle.rs`(940 例) |
| BR1.5 | ノードの行の記号の優先順 | `layout.rs` | `graph_layout_oracle.rs` |
| BR2.1 | 範囲の外(混ざった形)は辺の一覧 | `layout.rs` | `graph_crossing_lanes.rs`、`graph_lane_polish.rs`、`graph_branch_lanes.rs` |
| BR3.1 | ほかの形は変わらない | (変更なし) | `graph_layout_oracle.rs`(537、1,517、871 例) |
| BR4.1 | 正解データの一致 | `fixtures/graph_layout_oracle_cross.md` | `graph_layout_oracle.rs` |
| BR4.2 | README と仕様書の更新 | `README.md`、`docs/spec/rust`、`docs/spec/language-agnostic` | `docs_graph.rs` |
| BR5.1 | git のソースを読まない | (作業の進め方) | 設計と実装の確認 |

成功の指標 SM1〜SM3(`intent-statement.md`)も、すべて対応している。覆われていない規則: なし。
