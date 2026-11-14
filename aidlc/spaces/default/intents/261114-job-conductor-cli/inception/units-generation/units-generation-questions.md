# Units Generation — 質問

Domain Design で決めた7つの部品を、Construction で1つずつ作る「作業の単位(Unit)」にまとめます。Unit ごとに、設計・実装・テストの一連のステージが繰り返されるため、Unit の数は、進め方の手間に直接かかわります。`conductor` は単一バイナリなので、すべての Unit は、1つのバイナリに組み込まれます(デプロイは分かれません)。チームの実践では、ウォーキングスケルトンは作らず、機能を1つずつ順に作る方針です。

## Q1. [設計] Unit のまとめ方(粒度)

部品: DefinitionLoader、Flow、JobRunner、StateStore、Reporter、Cli、Converter。

- A. 4つの Unit にまとめる(推奨)
  - `u1-core`(library): DefinitionLoader + Flow。ファイルと OS に深く触れない、定義と判断のロジック
  - `u2-runtime`(library): JobRunner + StateStore。OS のプロセス・シグナルとファイルに触れる部分
  - `u3-cli`(service): Cli + Reporter。コマンドの受け付けと、表示。`conductor` の実行ファイルになる
  - `u4-converter`(library): Converter。独立した補助機能
- B. 部品ごとに7つの Unit にする(1部品が1 Unit)。責務は明確だが、Unit ごとに繰り返すステージが増え、承認の回数も増える
- C. 2〜3つの粗い Unit にまとめる。ステージの繰り返しは少ないが、1つの Unit が大きく、レビューもしにくい
- D. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q2. [設計] 独立した Unit の進め方

依存のない Unit どうしは、DAG の上では並行に進められます(各 Unit は、自分のコードと、依存する Unit だけで、ビルドとテストが通る必要があります)。進め方の前提を確認します。

- A. 1人で、依存関係の順に1つずつ進める。ただし、Unit は互いに独立して、ビルドとテストが通るように作る(並行にできる余地を残す)
- B. 独立した Unit は、並行して進める(複数のコピーで同時に作る)ことを前提にする
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
