# Functional Design (u1-retry-config) — 確認事項

この Unit は、ジョブの定義に「リトライの設定」を書けるようにし、読み取って検証します。要件(`requirements.md` の FR1、FR7)で決まっていないことを、ここで決めます。既存の定義では、ジョブごとに `success_exit_codes`、`warning_exit_codes`、`[[job.transitions]]` を書きます。

## Q1. [開発者] リトライの設定は、どんな書き方にしますか?

- A. ジョブの下に、専用の表(テーブル)を 1 つ書く。表があればリトライを有効にし、表の中に `exit_codes`(終了コードの一覧)、`max`(最大回数)、`interval`(間隔)、`multiplier`(倍率)、`max_interval`(間隔の上限)を書く
  ```toml
  [[job]]
  id = "load"
  command = "..."
  [job.retry]
  exit_codes = [75]
  max = 3
  interval = "30s"
  ```
- B. ジョブの直下に、平らなキーを並べる(`retry_exit_codes`、`retry_max`、`retry_interval`、`retry_multiplier`、`retry_max_interval`)
- C. Not yet defined
- X. Other (please specify)

[Answer]: A

## Q2. [開発者] リトライの終了コードの一覧が空(`[]`)の定義は、どうしますか?

- A. 定義の誤りにする(リトライを使うなら、1 つ以上のコードを書く)
- B. 受け付けて、「リトライしない」と同じ扱いにする
- C. Not yet defined
- X. Other (please specify)

[Answer]: A

## Q3. [開発者] 間隔の上限が、最初の間隔より小さい定義(例: `interval = "30s"`、`max_interval = "10s"`)は、どうしますか?

- A. 定義の誤りにする
- B. 受け付けて、上限を優先する(最初の間隔も上限で頭打ちにする)
- C. Not yet defined
- X. Other (please specify)

[Answer]: A

## Q4. [開発者] 「上側」の最大値は、どうしますか?(極端に大きい値や、倍率を掛けたときの桁あふれを防ぐため)

- A. 上限を決めて、超えたら定義の誤りにする(最大回数は 100 以下、間隔と間隔の上限は 24 時間以下、倍率は 10 以下)
- B. 上限は決めず、計算で桁あふれしないように、頭打ちで扱う
- C. Not yet defined
- X. Other (please specify)

[Answer]: A

## Q5. [開発者] リトライの終了コードに書ける値は、どうしますか?

- A. 1〜255 の整数だけ(0 は誤り。0 は成功を表すため)
- B. 0〜255 の整数(0 は、成功の終了コードの一覧に 0 がない場合だけ書ける)
- C. Not yet defined
- X. Other (please specify)

[Answer]: A

## Q6. [開発者] 起動できなかった場合(終了コード 127、conductor が付ける)と、ジョブ自身が 127 で終わる場合は、どう扱いますか?

- A. 起動できなかった場合は、リトライの対象にしない(一覧に 127 があっても再実行しない)。ジョブ自身が 127 で終わった場合は、一覧にあれば対象にする
- B. 127 を一覧に書いたら、起動できなかった場合も対象にする
- C. Not yet defined
- X. Other (please specify)

[Answer]: A
