# ユニットをまたぐ追跡(Cross-Unit Traceability): graph の text 表示の改善 その 2

要件の分析とユニットの分割は省いた(ユニットは 1 つ)。追跡の ID は、機能の設計の規則(`rules.md` の BR)で、`construction/code-generation/traceability.json` が正本。

| 規則 | 内容 | 実装 | テスト |
|------|------|------|--------|
| BR1.1〜BR1.4 | 分岐は新しい線を 2 本同時に作る。ラベルの位置、分岐の行、合流の行の列 0 | `crates/core/src/graph/layout.rs` | `graph_layout_oracle.rs`(871 例)、`graph_branch_lanes.rs` |
| BR1.5 | ノードの行の既存の線の記号(既存と同じ `/` の規則) | `layout.rs` | `graph_layout_oracle.rs` |
| BR2.1 | 交差と `t2 >= top` の分岐は辺の一覧 | `layout.rs` | `graph_drawable.rs` |
| BR2.2 | 分岐の線への後のジョブの合流は辺の一覧 | `layout.rs` | `graph_lane_polish.rs` |
| BR3.1 | ほかの形は変わらない | (変更なし) | `graph_layout_oracle.rs`(537 例、1,517 例)、`graph_branch_lanes.rs` |
| BR4.1 | 正解データ 871 例の一致 | `fixtures/graph_layout_oracle_branch.md` | `graph_layout_oracle.rs` |
| BR4.2 | README と仕様書の更新 | `README.md`、`docs/spec/rust`、`docs/spec/language-agnostic` | `docs_graph.rs` |

成功の指標 SM1〜SM3(`intent-statement.md`)も、すべて対応している。覆われていない規則: なし。
