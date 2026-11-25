# 性能の設計: u7-watch-follow

出典: 性能の要件(`performance-requirements.md` の NFR1.6、NFR1.7)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 2、3.1)、契約(`contract-summary.md` の C5)。

## 設計

- **1 回の描画(NFR1.6)**: 読み込み 1 回、`now()` 1 回、表の組み立て 1 回、書き出し 1 回。待ちは `Runtime::wait`(50 ミリ秒ごとに合図を見て眠る)で、忙しく回らない。
- **書く回数(NFR1.7)**: リトライ 1 回で Fast 2 回(各試行の始め)、Durable 1 回。
- 確かめ方: 流れのテストで、描画 1 回あたりの読み込みと待ちの回数、リトライの書く回数。
