# ユニットをまたぐ追跡(Cross-Unit Traceability): graph の text 表示の改善

この計画は、要件の分析とユニットの分割を省いた(ユニットは 1 つ)。追跡の ID は、機能の設計の規則(`rules.md` の BR)で、各規則を、実装とテストに結びつけた(`construction/code-generation/traceability.json` が正本)。

| 規則 | 内容 | 実装 | テスト |
|------|------|------|--------|
| BR1.1〜BR1.4 | 同じ行き先は、既存の線に合流(`|\|`)。ラベルの位置、直後の記号 | `crates/core/src/graph/layout.rs` | `graph_layout_oracle.rs`(1,517 例)、`graph_lane_polish.rs` |
| BR1.5 | 交差の判定(`t > top`) | `layout.rs` | `graph_drawable.rs` |
| BR2.1 | 両方の飛び越しは線なし | `layout.rs` | `graph_lane_polish.rs`(D1〜D6) |
| BR2.2 | 図に印を付けない。注記は `-v` | `crates/cli/src/report/graph_text.rs`(変更なし) | `graph_report.rs` |
| BR3.1 | 辺の一覧に落ちる形と英語の注記 | `crates/cli/src/report/messages.rs` | `graph_report.rs`、`binary_graph.rs` |
| BR3.2 | ほかの形は変わらない | (変更なし) | `graph_layout_oracle.rs`(既存の 537 例) |
| BR4.1 | 正解データの全例の一致 | `fixtures/graph_layout_oracle_same_target.md` | `graph_layout_oracle.rs` |
| BR4.2 | README と仕様書の更新 | `README.md`、`docs/spec/rust`、`docs/spec/language-agnostic` | `docs_graph.rs` |

成功の指標 SM1〜SM4(`intent-statement.md`)も、すべて対応している。覆われていない規則: なし。
