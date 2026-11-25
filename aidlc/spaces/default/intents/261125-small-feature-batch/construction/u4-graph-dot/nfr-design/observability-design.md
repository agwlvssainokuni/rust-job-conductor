# 観測の設計: u4-graph-dot

出典: 観測の要件(`observability-requirements.md` の NFR1.5、NFR6.3)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 2、5)、契約(`contract-summary.md` の C5)。

監視・メトリクス・トレースは当てはまらない。

## 設計

- **状態の表し方を揃える(NFR1.5)**: DOT のラベルの記号は `state_symbol` をそのまま使う。再開位置は `GraphNode.is_resume` を見て `penwidth=3` を付ける(mermaid が `classDef resume` を付ける条件と同じ)。
  - 確かめ方: 同じ `FlowGraph`(状態を重ねたもの)を mermaid と DOT で描き、各ジョブのラベルの記号が一致し、太い枠が再開位置のジョブだけに付くことを `graph_dot.rs` で確かめる。
- **誤りの表示を揃える(NFR6.3)**: `--format` の値の違いは描画の直前の分岐だけで、定義やステートの読み込みと誤りの表示は共通の経路を通る。
  - 確かめ方: `flow_graph.rs` で、誤りのある定義を `--format dot` と `--format text` で読み、標準エラーと終了コードが同じことを確かめる。
