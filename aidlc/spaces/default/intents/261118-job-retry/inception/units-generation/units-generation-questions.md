# Units Generation — 確認事項

この機能は、既存の 4 つの crate(core、runtime、converter、cli)の中を変える機能で、新しい構成要素は増えません。作業を、順に作って確かめられる単位(Unit)に分けます。

私の案(3 つの Unit):

- u1-retry-config: ジョブの定義に、リトライの設定を書けるようにし、検証する(core の定義の解析と検証、誤りの文言、`validate`、`convert` の確認)。要件 FR1、FR7。
- u2-retry-execution: 設定どおりにリトライを実行する(`run` と `pipeline`、固定の間隔と回ごとに伸ばす方式、待機中のシグナル、時間の差し込み口)。要件 FR2〜FR5、NFR3、NFR7。
- u3-retry-display-docs: リトライの途中の表示(英語と日本語)と、README。要件 FR6、FR8。

u2 は u1 に、u3 は u2 に、依存します。

## Q1. [開発者] 作業の分け方(境界の取り方)は、どれにしますか?

- A. 私の案(機能の段階で分ける: 設定、実行、表示と文書の 3 つ)
- B. crate ごとに分ける(core、runtime、cli、converter)
- C. 分けない(1 つの Unit にまとめる)
- D. Not yet defined
- X. Other (please specify)

[Answer]: A

## Q2. [開発者] 1 つの Unit の大きさは、どのくらいにしますか?

チームの方針は「変更を小さく保ち、小さな単位でレビューする」です。

- A. 小さめ(1 つの Unit が、レビューしやすい範囲に収まる)
- B. 大きめ(Unit の数を減らし、1 つの Unit にまとめて入れる)
- C. Not yet defined
- X. Other (please specify)

[Answer]: A

## Q3. [開発者] 独立した Unit は、同時に進めますか?

この案は、依存の順に 1 つずつ進める形になります(u1 → u2 → u3)。

- A. 1 つずつ順に進める(チームの方針「機能を 1 つずつ順に作る」に合わせる)
- B. 独立した Unit があれば、同時に進めてよい
- C. Not yet defined
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

- Looks correct
- Request changes

[Answer]: Looks correct
