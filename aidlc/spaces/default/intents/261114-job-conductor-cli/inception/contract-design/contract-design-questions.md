# Contract Design — 質問

`conductor` は、1つのバイナリの中の4つの Unit が、プログラムの呼び出しで連携します(通信はありません)。そのため、ネットワークの API ではなく、次の2種類の約束事が対象です。

- **Unit どうしの境界**: u1-core、u2-runtime、u3-cli、u4-converter のあいだで受け渡す型と呼び出し(unit-of-work-dependency の接点の表)
- **外部との境界**: バイナリの外から見える約束事。スケジューラが見る終了コードとコマンド、利用者が書く `conductor.toml`、`conductor` が書くステートファイル

## Q1. [設計] 外部との境界として、正式な約束事にするもの

- A. 3つとも、約束事として固定する(コマンドと終了コード、`conductor.toml` の書式、ステートファイルの書式)(推奨)
- B. コマンドと終了コード、`conductor.toml` の書式の2つだけを固定する。ステートファイルは内部の都合として、固定しない
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q2. [設計] 書式の変更に対する方針(互換性)

`conductor.toml` と、ステートファイルの書式を、あとで変更するときの扱いを確認します。

- A. どちらのファイルにも、書式の版(バージョン)を持たせる。古い版のステートを読んだときは、変換せず、エラーにして `reset` を促す。定義ファイルの版が新しすぎるときも、エラーにする(推奨)
- B. 版は持たせない。キーの追加だけで拡張し、読むときは、不明なキーを無視する
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q3. [設計] 定義ファイルの、知らないキーの扱い

`conductor.toml` に、定義にないキーが書かれていたときの扱いを確認します(綴りの誤りを、早く見つけるためにも関わります)。

- A. エラーにして、ジョブを実行しない(ファイル名・行・キーを表示する)(推奨)
- B. 無視して実行する
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
