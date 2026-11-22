# ユニットをまたぐ追跡(Cross-Unit Traceability): graph の配置のリファクタリング

要件の分析とユニットの分割は省いた(ユニットは 1 つ)。追跡の ID は、成功の指標(`intent-statement.md` の SM1〜SM4)と、機能の設計の規則(`rules.md` の BR)で、`construction/code-generation/traceability.json` が正本。

| ID | 内容 | 実装 | テスト |
|----|------|------|--------|
| SM1、BR1.1、BR1.2 | 判定(ジョブごとの 4 つの種類)と、1 つの仕組み(古い経路の削除) | `crates/core/src/graph/layout.rs`(`skip_targets`、`layout_lanes`) | `graph_drawable.rs`、`graph_layout_oracle.rs` |
| BR1.3 | 「両方の飛び越し」と分岐の線の開き方 | `layout.rs` | `graph_layout_oracle.rs`(offmain 2,748、offmain_67 300、branch 871) |
| BR1.4 | 主線の列 0(実行されうるジョブからの辺)、`-v` の詳細の行も同じ | `layout.rs`、`graph/mod.rs`(`LaneLayout.main_line`)、`crates/cli/src/report/graph_text.rs` | `graph_layout_unify.rs`、`graph_report.rs`、`binary_graph.rs` |
| SM3、BR2.1 | 組み合わせの形を図にする | `layout.rs` | `graph_layout_unify.rs`、`graph_crossing_lanes.rs`、`graph_mixed_shapes.rs` |
| SM2、BR3.1 | git の出力の正解データは変わらない | (変更なし) | `graph_layout_oracle.rs`(537、1,517、940、487 例) |
| BR4.1 | 規則から導いた正解データの一致 | `crates/core/tests/fixtures/` の 3 つ | `graph_layout_oracle.rs` |
| SM4、BR4.2 | README と仕様書の書き直し | `README.md`、`docs/spec/rust`、`docs/spec/language-agnostic` | `docs_graph.rs` |
| BR5.1 | git のソースを読まない | (作業の進め方) | 設計と実装の確認 |

覆われていない規則・指標: なし。
