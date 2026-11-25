# 規模の要件: u4-graph-dot

出典: 要件(`requirements.md` の NFR1)、機能の仕様(`functional-spec.md` の 2)、ビジネスルール(`rules.md` の BR2.1)、契約(`contract-summary.md` の C5)、技術スタック(`technology-stack.md`)。

単体の CLI で、負荷で伸び縮みするものは無い。

## 要件

| ID | 要件 | 合否の基準 | 出典 |
|---|---|---|---|
| NFR1.4 | 扱える定義の大きさ(ジョブの数)は今の `graph` と同じ。DOT の出力の大きさはジョブと辺の数に比例し、上限は設けない | 今の上限のテストが通る | NFR1 |
