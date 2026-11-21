# ユニットの生成: 質問

「ユニット」は、Construction で、1 つずつ設計と実装を回す作業の単位です。これまでの決定(依存の順に作る、変更は小さな単位に保つ、機能を 1 つずつ順に作る)を土台に、分け方を決めます。

## Q1. [アーキテクト] ユニットの分け方(粒度)

4 つの機能を、どの粒度のユニットにしますか。

- A. 機能ごとに 4 つ: (1) 環境変数と `.env`、(2) `graph`、(3) `--dry-run`、(4) `init`
- B. 5 つに細かく分ける: (1) 環境変数の表と組み込みの変数、(2) `.env` ファイル、(3) `graph`、(4) `--dry-run`、(5) `init`(「環境変数と `.env`」が最も大きいので、2 つに分ける)
- C. 層ごとに 3 つ: `core`、`runtime`、`cli`(機能をまたいで、層で分ける)
- D. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q2. [アーキテクト] 第 2 段階の拡張の置き場所

`graph --state`(状態を重ねた図)と、`--dry-run --assume`(終了コードを仮定した模擬)は、v0.9.0 の範囲に含めると決めています。どのユニットに入れますか。

- A. それぞれの基本の機能と同じユニットに入れる(`graph --state` は `graph` のユニット、`--assume` は `--dry-run` のユニット)
- B. 別のユニットにして、基本の機能のユニットのあとに作る
- C. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q3. [アーキテクト] README、デモ、メッセージの置き場所

README の更新、デモの場面の追加、日本語と英語のメッセージは、どのユニットで作りますか。

- A. 各ユニットが、自分の機能の README、デモ、メッセージを作る(今の履歴と同じ。機能ごとに、一通り完結する)
- B. 最後に、README とデモだけを、別のユニットで、まとめて作る(メッセージは各ユニット)
- C. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
