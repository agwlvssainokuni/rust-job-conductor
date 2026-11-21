# ドメイン設計: 質問

今ある構成(要件: `requirements.md`、既存の構成: コード知識ベースの `component-inventory.md`)に、4 つの機能を足すときの、部品の境界を決めます。今は、`conductor-core`(定義の読み込みと検証、判断の純粋関数)、`conductor-runtime`(プロセスの起動、ステート)、`conductor-cli`(引数、流れの組み立て、表示)の 3 つの層があります。`core` は、定義の取り込み(`intake`)以外ではファイルに触れない、という制約があります。

## Q1. [アーキテクト] `.env` の読み取りと、環境変数の統合の置き場所

`.env` の文字列を読む処理(書式の検査と項目の取り出し)と、各層(引き継いだ環境、`.env`、`[pipeline.env]`、`[job.env]`)を優先順位で統合する処理を、どこに置きますか。

- A. 書式の検査と統合は、`core` に新しい部品(`envfile`)として置く(文字列を受け取るだけの純粋な関数)。ファイルを開いて文字列にする処理は、`runtime` に置く
- B. すべて `runtime` に置く(ファイルの読み取りも、書式の検査も、統合も)
- C. すべて `core` に置く(`intake` と同じく、`envfile` もファイルに触れる)
- D. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q2. [アーキテクト] `graph` の図を作る処理の置き場所

次のジョブを決める判断(`core::flow`)から辺を作り、`git log --graph` 風の配置(レーンの割り当て)と、text と Mermaid の文字列を作る処理を、どこに置きますか。

- A. 辺の取り出しとレーンの配置は、`core` の新しい部品(`graph`。副作用のない純粋な関数)に置く。text と Mermaid の整形は、`cli` の表示に置く
- B. 辺の取り出し、配置、整形のすべてを、`core` の新しい部品に置く
- C. すべて `cli` に置く
- D. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q3. [アーキテクト] `--dry-run` の計画を作る処理の置き場所

「定義と、読み込んだステートから、どのジョブを実行する・スキップするか、再開位置はどこか」を決める処理(`--assume` の模擬を含む)を、どこに置きますか。

- A. `core` の新しい部品(`plan`。定義、ステートの内容、仮定の終了コードを受け取って、計画を返す純粋な関数)に置く。ステートの読み込みは、今のとおり `runtime`、表示は `cli`
- B. `cli` の流れの組み立てに置く(`pipeline` と `run` の中に、dry-run の分岐を足す)
- C. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q4. [アーキテクト] `init` のひな形と、ファイルの作成の置き場所

ひな形の文字列(日本語/英語)と、排他的なファイルの作成を、どこに置きますか。

- A. ひな形の文字列は、`cli` の表示(メッセージの置き場所)に置く。排他的な作成は、`runtime` のファイル書き込みの部品に置く(ステートの書き込みと同じ流儀)
- B. ひな形も、作成も、`cli` に置く
- C. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
