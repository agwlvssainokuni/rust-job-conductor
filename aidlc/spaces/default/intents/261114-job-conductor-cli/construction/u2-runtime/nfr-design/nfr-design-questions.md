# NFR Design(u2-runtime)— 質問

u2-runtime は library のため、可用性・スケーラビリティ・可観測性の設計は、この Unit の種類には当てはまりません。ここで設計するのは、ジョブの待ち方とシグナルの送り方の安全、ステートの書き込みの安全、テストの作り方です。判断が必要な2点を確認します。

## Q1. [設計] ジョブの終了とシグナルの待ち方

ジョブの終了を待ちながら、中断の合図が立つのを見る必要があります。シグナルを送る相手の pid が、終了後に別のプロセスに使い回される事故(NFR4.8)を避けたいです。

- A. 1つのスレッドで、ジョブの終了を、短い間隔(最初は 1ms、徐々に 10ms まで)で確かめるくり返しにし、同じくり返しの中で合図を見る。自分でジョブを回収するのは、このくり返しだけなので、終了後の pid に、シグナルを送らない。ジョブの終了の観測が、最大 10ms 遅れる(推奨)
- B. 専用のスレッドが、ジョブの終了を待ち(ブロックして待つ)、メインのスレッドが、合図を見る。終了の観測は即時だが、回収と送信の競合を、ロックで守る必要があり、競合の隙間が残る
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q2. [テスト] テスト専用の補助の実行ファイル

引数・環境変数・終了コード・シグナルの受信を出力する、テスト専用の小さな実行ファイルが要ります(3つの OS で、同じ動きにする)。

- A. この Unit のクレートの中に、テスト専用の実行ファイル(bin)として置く。`cargo test` が、自動でビルドする(推奨)
- B. ワークスペースに、別のクレート(`crates/test-helper`)として置く
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
