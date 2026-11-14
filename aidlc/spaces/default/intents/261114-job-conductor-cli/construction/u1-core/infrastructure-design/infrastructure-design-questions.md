# Infrastructure Design(u1-core)— 質問

u1-core は library のため、サーバー、クラウド、監視、デプロイ先の環境はありません。この Unit で設計するのは、CI(GitHub Actions)で何を確かめるかだけです(案は cicd-pipeline.md にあります)。判断が必要な2点を確認します。

## Q1. [CI] 起動時間の確認(NFR1.1 の 20ms)の扱い

仮の基準 20ms(p95)を、CI でどう扱うかを確認します。CI の実行環境は、時間のばらつきが大きいためです。

- A. 当面は、基準を超えても警告として記録するだけにし、Code Generation で実測してから、失敗にするかを決める(推奨)
- B. 最初から、基準を超えたら CI を失敗にする
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q2. [CI] 行カバレッジ 80% の測定の場所

`cargo-llvm-cov` による行カバレッジの測定を、どの OS で行うかを確認します。

- A. Linux だけで測り、80% 未満なら失敗にする。他の OS ではテストの成功だけを確かめる(推奨)
- B. 3つの OS すべてで測り、それぞれ 80% 以上を要求する
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
