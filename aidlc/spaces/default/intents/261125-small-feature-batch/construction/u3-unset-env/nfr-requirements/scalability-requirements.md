# 規模の要件: u3-unset-env

出典: 要件(`requirements.md` の NFR1)、機能の仕様(`functional-spec.md` の 2)、ビジネスルール(`rules.md` の BR2.2)、契約(`contract-summary.md` の C1)、技術スタック(`technology-stack.md`)。

## 要件

| ID | 要件 | 合否の基準 | 出典 |
|---|---|---|---|
| NFR1.4 | `unset_env` の名前の数の上限は設けない(定義の大きさの上限の中)。`.env` との重なりの誤りは、今の `.env` の誤りと同じく表示の上限(20 件)に従い、同じファイルを複数のジョブが読んでも同じ誤りを重ねて数えない | 同じ `.env` を 2 つのジョブが読む定義で、重なりの誤りが 1 つだけ出ること | NFR1、BR2.2 |
