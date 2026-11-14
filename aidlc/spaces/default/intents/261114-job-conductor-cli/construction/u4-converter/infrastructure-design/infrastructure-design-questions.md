# Infrastructure Design(u4-converter)— 質問

u4-converter は library のため、サーバー、クラウド、監視、デプロイ先の環境はありません。この Unit で設計するのは、CI(GitHub Actions)で何を確かめるかだけです。u1-core の CI の設計(cicd-pipeline.md)と同じ方針で案を作りました。違いを生む2点を確認します。

## Q1. [CI] ワークフローの単位

u1-core の CI も、この Unit の CI も、同じ GitHub Actions で動きます。

- A. ワークスペース全体で、1つのワークフローにする。整形・静的解析・ビルド・テストは、`--workspace` で全 Unit をまとめて行う。ワークフローは、CI Pipeline のステージで1回だけ作る(推奨)
- B. Unit ごとに、別のワークフローにする
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q2. [CI] 入力の暴走(深い入れ子、展開で膨らむ YAML)のテストの扱い

NFR4.3・NFR4.4 のテスト(深さ 100,000 の入力、展開で膨らむ入力)が、まれにハングしても気づけるようにします。

- A. これらのテストに、時間の上限(各 30 秒)をつけ、超えたら失敗にする。時間の上限は、テストの中で測り、CI の全体の時間の上限(ジョブ 15 分)でも守る(推奨)
- B. 時間の上限はつけず、CI の全体の時間の上限だけに頼る
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
