# 規模の要件: u2-optional-env-file

出典: 要件(`requirements.md` の NFR1)、機能の仕様(`functional-spec.md` の 3)、ビジネスルール(`rules.md` の BR2.1)、契約(`contract-summary.md` の C1)、技術スタック(`technology-stack.md`)。

単体の CLI で、負荷で伸び縮みするものは無い。

## 要件

| ID | 要件 | 合否の基準 | 出典 |
|---|---|---|---|
| NFR1.4 | `.env` の大きさ(1 MiB)と項目の数(10,000)の上限、定義の大きさの上限は今のまま。表の `env_file` で読む対象の数の上限は設けない(今の文字列と同じ) | 今の上限のテストが通る | NFR1 |
