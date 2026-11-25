# 性能の要件: u7-watch-follow

出典: 要件(`requirements.md` の FR7.2、NFR2)、機能の仕様(`functional-spec.md` の 2、3.1)、ビジネスルール(`rules.md` の BR2.1、BR3.4)、契約(`contract-summary.md` の C5)、技術スタック(`technology-stack.md`)。

## 要件

| ID | 要件 | 合否の基準 | 出典 |
|---|---|---|---|
| NFR1.6 | 1 回の描画は、ステートの読み込み 1 回と表の組み立てだけで済ませ、待ちは `Runtime::wait` で眠る(忙しく回らない) | 流れのテストで、1 回ごとの読み込みと待ちの回数 | FR7.2、BR2.1 |
| NFR1.7 | リトライの書く回数は、リトライ 1 回で Fast 2 回、Durable 1 回(U6 より 1 回少ない) | U6 の流れのテストの期待を合わせて確かめる | BR3.4 |
