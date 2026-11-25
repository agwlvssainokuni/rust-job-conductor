# 規模の要件: u7-watch-follow

出典: 要件(`requirements.md` の FR7.2)、機能の仕様(`functional-spec.md` の 2)、ビジネスルール(`rules.md` の BR1.2)、契約(`contract-summary.md` の C5)、技術スタック(`technology-stack.md`)。

## 要件

| ID | 要件 | 合否の基準 | 出典 |
|---|---|---|---|
| NFR1.8 | 長く続けても記憶を増やさない(描くたびに前のステートと表を捨てる)。間隔は 1〜3600 秒 | 流れのテストで多数回繰り返しても状態を持ち越さないこと(`seen` のほかに持たない) | FR7.2、BR1.2 |
