# 技術の選択: u4-graph-dot

出典: 技術スタック(`technology-stack.md`)、要件(`requirements.md` の NFR4、NFR7)、機能の仕様(`functional-spec.md` の 1)、ビジネスルール(`rules.md` の BR2.1)、契約(`contract-summary.md` の C5)、team.md の Code Style。

新しい技術は選ばない。

| 項目 | 選択 | 理由 |
|---|---|---|
| DOT の整形 | cli の `report` に新しいモジュール `graph_dot.rs` を足し、自前で書く(ADR-006) | 新しい依存を加えない(team.md)。mermaid と同じ形で書ける |
| 共有の補助 | `graph_text.rs` の `state_symbol`、`kind_label`、`is_main_edge` と `report` の `escape_full` を使う | 形式の間で記号・語・判定を揃える |
| 引数 | `args.rs` の `parse_graph_format` に `dot` を足す | 今の形 |
| DOT の文法の確かめ方 | Graphviz を CI に入れず、escape の対の表と、出力の文字列の比較で確かめる | 依存と CI の道具を増やさない |

## 要件

| ID | 要件 | 合否の基準 | 出典 |
|---|---|---|---|
| NFR7.1 | `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通る | CI の `lint` | NFR7 |
