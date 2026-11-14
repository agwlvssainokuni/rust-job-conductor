# Infrastructure Design(u2-runtime)— 質問

u2-runtime は library のため、サーバー、クラウド、監視、デプロイ先の環境はありません。この Unit で設計するのは、CI(GitHub Actions)で何を確かめるかだけです。u1-core・u4-converter の CI の設計と同じ方針(ワークスペース全体で、1つのワークフロー)で案を作りました。この Unit は、実際のプロセスを起動し、OS ごとに動きが違う部分(シグナル、Windows の `.cmd`)を持つため、次の2点を確認します。

## Q1. [CI] プロセスを起動するテストの時間の上限

テストが、ジョブ(補助の実行ファイル)の終了を待ち続けてハングすることを、見つけられるようにします。

- A. プロセスを起動するテストに、各 30 秒の時間の上限をつけ、超えたら失敗にする(別のスレッドで動かし、結果を待つ)。CI のジョブ全体は 15 分で打ち切る(推奨)
- B. 時間の上限はつけず、CI のジョブ全体の時間の上限だけに頼る
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q2. [CI] OS 固有のテストの扱い

シグナルの転送(UNIX 系)と、Windows の `.cmd` の安全な起動は、その OS の CI でしか確かめられません。

- A. 各 OS の CI で、その OS 固有のテストも、マージを止める必須のチェックにする。Windows の `.cmd` のテストの結果が、期待と異なったら、設計を見直す(推奨)
- B. OS 固有のテストは、情報として記録するだけで、失敗しても、マージを止めない
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
