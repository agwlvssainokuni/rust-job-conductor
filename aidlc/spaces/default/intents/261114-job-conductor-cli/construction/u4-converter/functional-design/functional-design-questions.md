# Functional Design(u4-converter)— 質問

u4-converter は、YAML と TOML のあいだで、定義ファイルの内容を変換する小さな部品です(US7.1、AC7.1.1〜AC7.1.3)。要件で決まっていないが、変換の結果を左右する3点を確認します。

## Q1. [変換の範囲] 変換の対象は何ですか

- A. 内容を問わない、一般のデータの変換にする。`conductor` の定義として正しいかは調べない(定義の検証は、`conductor run` などの読み込み時に行われる)(推奨)
- B. `conductor` の定義(`format_version`、`pipeline`、`jobs` など)として正しいことを確かめてから変換し、正しくなければ変換しない
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q2. [変換元の形式] 入力が YAML か TOML かを、どう決めますか

`conductor convert <file> --to <format>` は、出力の形式だけを指定します。

- A. `--to` と反対の形式を、入力とみなす(`--to toml` なら YAML として読む。`--to yaml` なら TOML として読む)。拡張子は見ない(推奨)
- B. 拡張子(`.yaml`・`.yml`・`.toml`)で決め、決められなければエラーにする
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q3. [変換できない値] 相手の形式で表せない値があったときは、どうしますか

たとえば、YAML の `null`、日付でない特殊な値、キーが文字列でないマップ、1つのファイルに複数の文書、アンカーとエイリアスなどです。TOML には `null` がありません。

- A. 意味を変えずに変換できないものは、変換せず、原因(どのキーのどの値か)を表示してエラーにする。アンカーとエイリアスは、展開して変換する(推奨)
- B. 表せない値は、黙って取り除く、または近い値に置き換えて、変換を続ける
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
