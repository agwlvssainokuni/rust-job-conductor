# Domain Design — 質問

`conductor` を構成する「作るコード」の部品(コンポーネント)と、その責務の境界を決めます。データベースやファイルなどは、部品ではなく、部品が使うもの(外部依存)として扱います。配置の形(単一バイナリ内のどこに置くか)と技術スタックは、このステージでは決めません。

候補となる部品の例(論理的な責務):

- DefinitionLoader: `conductor.toml` を読み、検証する(FR1)
- Flow(遷移の判断): 結果の分類、次に実行・スキップするジョブ、完了の判定(FR2.2〜FR2.4、FR4.2)
- JobRunner: ジョブのコマンドを起動し、終了コードとシグナルを扱う(FR2.1、FR2.9)
- StateStore: ステートファイルの保持、書き込みの安全性、破損・食い違いの検知(FR4)
- Converter: YAML と TOML の変換(FR7)
- Reporter: 表示(`-v`、`watch`、エラー)と文言の言語(FR2.6、FR5.1)
- Cli: コマンド・オプションの受け付けと、各部品の呼び出し(`run`、`pipeline`、`watch`、`reset`、`convert`)

## Q1. [設計] 部品の分け方(粒度)

- A. 細かく分ける(上の7つをそれぞれ独立した部品にし、さらに「結果の分類」を Flow から分ける)。責務は明確だが、部品間のやり取りが多い
- B. 中間(上の7つをそのまま部品にする)。責務の境界が分かりやすく、部品どうしのやり取りも少ない(推奨)
- C. 粗く分ける(DefinitionLoader + StateStore を1つ、Flow + JobRunner を1つ、Converter、Cli + Reporter の4つ)。部品は少ないが、1つの責務が大きくなる
- D. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: B

## Q2. [設計] 表示(Reporter)の位置づけ

表示と文言の言語(英語・日本語)の切り替えを、どこで受け持つかを確認します。

- A. 独立した Reporter 部品にする。他の部品は、表示の内容(種類とデータ)だけを渡し、文言は Reporter が組み立てる(推奨)
- B. Cli の一部にする。部品が少なくなる
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q3. [設計] ステートと定義の食い違い検知の担当

「ステートに記録されたジョブ構成と、現在の定義のジョブ構成が食い違う」ことの検知(FR4.6)を、どの部品が受け持つかを確認します。

- A. StateStore が、定義のジョブ構成を受け取って照合する(ステートの整合は StateStore の責務)(推奨)
- B. Flow が、ステートと定義を比べる(遷移の判断に関わる整合は Flow の責務)
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
