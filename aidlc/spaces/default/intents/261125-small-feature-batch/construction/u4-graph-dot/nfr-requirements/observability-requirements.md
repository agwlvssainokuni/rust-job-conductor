# 観測の要件: u4-graph-dot

出典: 要件(`requirements.md` の FR4.2、NFR6)、機能の仕様(`functional-spec.md` の 2、5)、ビジネスルール(`rules.md` の BR2.4、BR2.5)、契約(`contract-summary.md` の C5)、技術スタック(`technology-stack.md`)。

監視・メトリクスは当てはまらない。利用者が見るのは DOT の出力そのものである。

## 要件

| ID | 要件 | 合否の基準 | 出典 |
|---|---|---|---|
| NFR1.5 | `--state` のときの状態の記号と再開位置の表し方は、text・mermaid と同じ情報を持つ(記号は同じ文字列、再開位置は太い枠) | 同じステートで、mermaid と DOT の各ジョブの記号が一致し、再開位置のジョブだけに `penwidth=3` が付くことを確かめる | FR4.2、BR2.4 |
| NFR6.3 | 誤りの表示(定義・ステートの誤り、使い方の誤り)は text・mermaid と同じ経路で、同じ形 | 今の誤りの表示のテストが通る。`--format dot` での定義の誤りの表示が `--format text` と同じ | NFR6 |
