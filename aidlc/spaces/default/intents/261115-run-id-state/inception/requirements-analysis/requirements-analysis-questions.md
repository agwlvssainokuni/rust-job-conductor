# Requirements Analysis — 質問

依頼で決まっていること(質問しません): `--run-id` と環境変数(変数名は定義ファイルの `[pipeline]` の `run_id_env`、省略時は `CONDUCTOR_RUN_ID`)で run ID を与える。オプション優先。値は `A-Za-z0-9._-` で 64 文字まで(先頭の `.` と `..` は禁止、違反は 68)。空は未設定と同じで、`--run-id ""` は、環境変数があっても run ID なしに固定する。ステートは `<定義ファイル>.<run-id>.state` で、ステートの中の `run_id` が、ファイル名と食い違えば、壊れたステート(65)。`watch` と `reset` は、その run のステートだけを対象にする。`reset --all` で、その定義のステートをすべて消す。自動削除はしない。置き場所は、これまでのまま。

調査(codekb)で分かった、決める必要のある点を 2 つ確認します。

## Q1. [watch と reset] 環境変数の名前を、どうやって知るか

`run_id_env`(環境変数の名前)は、定義ファイルに書きます。ところが `watch` と `reset` は、いま、定義ファイルを読まないコマンドです(定義が壊れていても、`reset` でステートを消せる、という性質のためです)。

- A. `watch` と `reset` は、定義ファイルから `[pipeline].run_id_env` だけを、ゆるく読む(ほかの誤りは無視する)。定義が読めない(壊れている)ときは、既定の名前 `CONDUCTOR_RUN_ID` を使う。`--run-id` を渡したときは、定義を読まない。これで、壊れた定義でも、`reset` が動く性質を保つ(推奨)
- B. `watch` と `reset` は、常に既定の名前 `CONDUCTOR_RUN_ID` だけを見る(`run_id_env` は、`run` と `pipeline` だけに効く)。定義の変数名を変えている場合は、`watch`・`reset` には `--run-id` を渡すか、既定の名前の変数を設定する必要がある
- C. `watch` と `reset` も、`run` と同じに、定義を検証つきで読む。壊れた定義では、`reset` も失敗する(`--run-id` を渡したときは、定義を読まない)
- X. Other (please specify)

[Answer]: A

## Q2. [`reset --all` の照合] どのファイルを消すか

定義 `x.toml` の run ID `a` のステートは `x.toml.a.state` で、これは、別の定義 `x.toml.a`(run ID なし)のステート `x.toml.a.state` と、同じファイル名になりえます。ファイル名だけでは、区別できません。

- A. 候補のファイル `x.toml.*.state` を読み、中身の `run_id` が、ファイル名の run ID と一致するものだけ消す(`x.toml.state` も消す)。`run_id` を持たないもの(別の定義のステート)と、読めないもの(壊れている)は、消さず、一覧を警告として表示する。壊れたものは `reset --run-id <ID>` で個別に消せる(推奨)
- B. ファイル名だけで判定する(`x.toml.<有効な run ID>.state` をすべて消す)。別の定義のステートを、巻き込む可能性を受け入れる
- X. Other (please specify)

[Answer]: A
