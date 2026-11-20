# 要件分析: 質問

## Q1. [プロダクトオーナー] 要件の土台にする、これまでの決定

機能の検討で、次の詳細を決めています(メモ `project_v090_feature_plan.md` にも記録)。これを、要件の土台にしてよいですか。

環境変数と `.env`:
- `[pipeline.env]` と `[job.env]` の文字列の表。値は文字列だけで、展開(`${X}`)はしない。名前の規則は `run_id_env` と同じ。
- 優先順位(後ろが勝つ): 引き継いだ環境 < `[pipeline].env_file` < `[pipeline.env]` < `[job].env_file` < `[job.env]`。
- `env_file` は、文字列または配列(後ろのファイルが勝つ)。パスの基準は、定義ファイルのあるディレクトリ。
- `.env` は自前の厳格な書式(`KEY=VALUE`、`export ` の接頭辞可、単一引用符と二重引用符、エスケープは `\n \r \t \" \\` だけ、展開なし、複数行なし、上限 1 MiB)。
- 同じファイルの中で、同じキーが重複したら、後ろが勝つ。`validate` が警告を出す(誤りにはしない)。
- 呼び出しごとに、ジョブを起動する前に、すべての `.env` を読む。ファイルが無い、書式の誤りなら、何も実行せずに止まる。
- 値は、`-v`、`--dry-run`、ステートに出さない(名前だけ)。
- `CONDUCTOR_` で始まる名前の利用者定義は、予約名として拒否する。`unset_env` と、optional なファイルは見送り。

`graph`: `git log --graph` 風の text(既定)と Mermaid。`-v` で詳細。`--state` で状態を重ねる。

`--dry-run`: ステートを読んで、再開位置まで示す。何も実行せず、ステートも書かない。`--assume` で終了コードを仮定した模擬。

`init`: ひな形を作る。既にあるときは終了コード 69。`--force` で上書き。`-` で標準出力。

- A. この内容を、要件の土台にする
- B. 変更したい点がある(「その他」に書く)
- C. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q2. [開発者] 定義の書式の版(`format_version`)

今の `conductor` は、`format_version = 1` だけを受け入れます。`env` や `env_file` などの新しいキーを足すとき、版をどうしますか(互換を壊してもよい、と決めています)。

- A. 1 のままにする(新しいキーを足すだけ。旧版の `conductor` は、新しいキーを「未知のキー」として拒否する)
- B. 2 に上げる(新しいキーは 2 でだけ書ける。1 の定義も、これまでどおり読める)
- C. 2 に上げて、1 は読めなくする
- D. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q3. [開発者] ジョブに渡す組み込みの変数

ジョブに、`conductor` が自動で渡す変数を決めます。`CONDUCTOR_ATTEMPT` は、リトライを含めた「何回目の実行か」を表します。

- A. `CONDUCTOR_PIPELINE`、`CONDUCTOR_JOB_ID`、`CONDUCTOR_RUN_ID`、`CONDUCTOR_ATTEMPT` の 4 つ。`ATTEMPT` は 1 から始まり、リトライのたびに増える。run ID が無いときは `CONDUCTOR_RUN_ID` を渡さない
- B. 4 つとも渡す。`ATTEMPT` は 0 から始まる(リトライの回数)。run ID が無いときは、空文字列で渡す
- C. `CONDUCTOR_PIPELINE`、`CONDUCTOR_JOB_ID`、`CONDUCTOR_RUN_ID` の 3 つだけ(`ATTEMPT` は渡さない)
- D. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q4. [開発者] `--dry-run` の対象

`--dry-run` を、どのコマンドで使えるようにしますか。

- A. `pipeline` と `run` の両方
- B. `pipeline` だけ
- C. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q5. [開発者] `graph` で、定義に誤りがあるとき

`conductor graph` に渡した定義が、検証で誤りになるときの動きを決めます。

- A. `validate` と同じ誤りを出し(終了コードも同じ 64)、図は出さない
- B. 誤りを出しつつ、読めた範囲の図を出す
- C. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q6. [開発者] `init` のひな形

`conductor init` が作るひな形は、どうしますか。

- A. 1 種類のひな形。コメントはロケール(日本語/英語)に従い、`[pipeline.env]` と `[job.retry]` の例を含む。作ったものは、必ず `validate` に通る
- B. 最小のひな形(ジョブが 1 つだけ。例は含めない)
- C. `--template` で、複数のひな形から選べる
- D. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q7. [開発者] `.env` の上限

`.env` ファイルの大きさや、項目の数に、上限を設けますか(定義ファイルは、1 MiB が上限です)。

- A. ファイルの大きさは 1 MiB(定義と同じ)、項目の数は 10,000 を上限にする
- B. ファイルの大きさだけ 1 MiB を上限にする(項目の数は制限しない)
- C. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q8. [品質] 性能の基準

`graph` と `--dry-run` の応答の速さに、基準を設けますか。

- A. ジョブが 1,000 個ある定義でも、1 秒以内に終わる(テストで確かめる)
- B. 基準は設けない(測るだけで、強制しない)
- C. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
