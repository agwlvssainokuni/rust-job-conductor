# 規模の要件: u5-log-dir

出典: 要件(`requirements.md` の FR5.3、NFR1)、機能の仕様(`functional-spec.md` の 2)、ビジネスルール(`rules.md` の BR3.3、BR6.1)、契約(`contract-summary.md` の C6)、技術スタック(`technology-stack.md`)。

## 要件

| ID | 要件 | 合否の基準 | 出典 |
|---|---|---|---|
| NFR1.6 | ログのファイルの数は実行と試行の数だけ増え、conductor は上限を設けず消さない(整理は範囲の外) | README に整理しないことが書かれている | BR6.1 |
| NFR1.7 | 名前の衝突の番号は 1000 まで試し、それを超えたら作れないとして扱う(無限に試さない) | 番号の上限の単体テスト | BR3.3 |
