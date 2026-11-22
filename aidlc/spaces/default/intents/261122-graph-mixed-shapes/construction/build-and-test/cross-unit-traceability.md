# ユニットをまたぐ追跡(Cross-Unit Traceability): graph の text 表示の改善 その 4

要件の分析とユニットの分割は省いた(ユニットは 1 つ)。追跡の ID は、機能の設計の規則(`rules.md` の BR)で、`construction/code-generation/traceability.json` が正本。

| 規則 | 内容 | 実装 | テスト |
|------|------|------|--------|
| BR1.1 | 通常の飛び越しだけのグラフを図にする範囲(グラフ全体の条件) | `crates/core/src/graph/layout.rs`(`is_ordinary_skip_graph`) | `graph_mixed_shapes.rs`、`graph_drawable.rs` |
| BR1.2〜BR1.4 | 線の並びと寄せ(寄せの行、寄せる順序) | `layout.rs`(`ordinary_layout`、`move_rows`、`settle_duplicates`) | `graph_layout_oracle.rs`(487 例ほか) |
| BR1.5 | ノードの行の記号 | `layout.rs` | `graph_layout_oracle.rs` |
| BR2.1 | 「両方の飛び越し」、分岐を含むグラフは今の処理のまま | `layout.rs`(古い経路) | `graph_mixed_shapes.rs`、`graph_lane_polish.rs`、`graph_branch_lanes.rs` |
| BR3.1 | 前回までの出力は変わらない | (古い経路は変更なし) | `graph_layout_oracle.rs`(537、1,517、871、940 例) |
| BR4.1 | 正解データの一致 | `fixtures/graph_layout_oracle_mixed.md` | `graph_layout_oracle.rs` |
| BR4.2 | README と仕様書の更新 | `README.md`、`docs/spec/rust`、`docs/spec/language-agnostic` | `docs_graph.rs` |
| BR5.1 | git のソースを読まない | (作業の進め方) | 設計と実装の確認 |

成功の指標 SM1〜SM3(`intent-statement.md`)も、すべて対応している。覆われていない規則: なし。
