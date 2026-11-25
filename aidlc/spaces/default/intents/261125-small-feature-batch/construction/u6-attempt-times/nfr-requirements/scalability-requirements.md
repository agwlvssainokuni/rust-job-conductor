# 規模の要件: u6-attempt-times

出典: 要件(`requirements.md` の FR6.5)、機能の仕様(`functional-spec.md` の 4)、ビジネスルール(`rules.md` の BR1.4)、契約(`contract-summary.md` の C4)、技術スタック(`technology-stack.md`)。

## 要件

| ID | 要件 | 合否の基準 | 出典 |
|---|---|---|---|
| NFR1.4 | 試行は再開のたびに足されるが、ステートの大きさの上限(1 MiB)は変えない。リトライの最大回数と再開の回数が多くても、試行 1 つは 2 行で済む。上限を超えれば今の `TooLarge`(65)と `reset` の案内 | 試行を多く持つステートを読めること(性能のテストの上限の中) | BR1.4 |
