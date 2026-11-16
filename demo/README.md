# デモ

`conductor` の主な動きを、1 本のスクリプトで順に見せます(UNIX 系は `sh` を、Windows は cmd を使います。Windows 版は、下の「Windows 版」を見てください)。

```sh
demo/run-demo.sh                 # リポジトリの根から。実行ファイルが無ければ、リリースビルドを作る
DEMO_PAUSE=1 demo/run-demo.sh    # 各場面のあとで Enter を待つ(画面を見せながら話すとき)
DEMO_LANG=en_US.UTF-8 demo/run-demo.sh   # 表示を英語にする(既定は日本語)
CONDUCTOR=/path/to/conductor demo/run-demo.sh   # 配布のバイナリを使う
```

一時ディレクトリで動くので、リポジトリには何も残りません。

| 場面 | 見せること |
|------|-----------|
| 0 | 定義ファイル(`conductor.toml`)の中身を表示する |
| 1 | 全部が成功する(書いた順に、1 つずつ実行) |
| 2 | 警告で分岐する(`extract` が警告 → `transform` と `load` をスキップして `notify` へ) |
| 3 | エラーで止まる(`load` が失敗。終了コードがそのまま返り、`watch` で止まった位置が見える) |
| 4 | 同じコマンドをもう一度 → 失敗した `load` から再開(済んだジョブは、再実行しない)。完了してもステートは残り、`watch` は表の下に完了を示す行(`completed`。日本語では `完了`)を出す。`--remove-on-complete` を付けると、完了でステートファイルを消す |
| 5 | ジョブを 1 つだけ実行する(`run`)と、`reset` |
| 6 | 定義ファイルを YAML に変換する(`convert`) |
| 7 | 定義を検査する(`validate`)。正しい定義は 0、`command` の中の引用符が閉じていない定義は 64。ジョブは実行されず、ステートも作られない |
| 8 | run ID でステートを分ける(`--run-id build-1` は `load` で止まり、環境変数 `DEMO_RUN_ID` の `build-2` は最後まで進む。同じ run ID で再実行すると `load` から再開。完了してもステートファイルは残り、`reset --run-id` で片づける) |
| 9 | 環境変数と後始末(`--run-id` は環境変数に勝つ。`--run-id ""` は run ID なしに固定。`reset --run-id` で 1 つ、`reset --all` で全部を消す) |

- 定義は `conductor.toml`、ジョブは `scripts/*.sh` です。ジョブの終了コードは、環境変数 `DEMO_<ジョブ名>_EXIT`(ジョブ名は大文字。例 `DEMO_LOAD_EXIT=2`。既定 0)で決まります。`DEMO_SLEEP` で、各ジョブの待ち時間(既定 0.5 秒。`sleep` に渡すので、小数も使えます)を変えられます。
- `DEMO_PAUSE` は、値が `1` のときだけ、各場面のあとで Enter を待ちます(既定は待ちません)。
- `notify` の `command` は、文字列の配列(`["sh", "scripts/notify.sh"]`)で書いています。ほかのジョブは、文字列です。
- 定義の `[pipeline]` に、`run_id_env = "DEMO_RUN_ID"` を書いています。run ID は、`--run-id` か、この環境変数で指定します(`--run-id` が優先。`--run-id ""` は、環境変数があっても、run ID なしに固定)。run ID ごとに、ステートファイルが別になります(`conductor.toml.<run ID>.state`)。規則の詳細は、ルートの [README](../README.md) の「[run ID でステートを分ける](../README.md#run-id-でステートを分けるv050-以降)」にあります。
- 場面 8 と 9 では、`DEMO_RUN_ID` を、デモが自分で設定します。デモの最初に、`DEMO_RUN_ID` と `CONDUCTOR_RUN_ID` は消すので、利用者の環境の値は影響しません。`LC_ALL` と `LC_MESSAGES` も消すので、`DEMO_LANG` の言語で表示されます。
- 場面 7 の誤りの定義(`bad.toml`)は、デモが一時ディレクトリの中に作ります。表示される原因の文は、日本語では `command の引用符が閉じていません`、英語では `command has an unclosed quote` です。

## Windows 版

Windows では、`sh` のスクリプトは動きません。代わりに、`demo\windows\` の cmd 版を使います(PowerShell でも、コマンドプロンプトでも動きます)。見せる場面は、UNIX 版と同じ(場面 0〜9)です。run ID の環境変数 `DEMO_RUN_ID` は、定義(`demo\windows\conductor.toml`)の `run_id_env` で受け取ります。

```bat
rem リポジトリの根から。実行ファイルが無ければ、リリースビルドを作る
demo\windows\run-demo.cmd

rem 各場面のあとで Enter を待つ(画面を見せながら話すとき)
set DEMO_PAUSE=1
demo\windows\run-demo.cmd

rem 表示を英語にする(既定は日本語)
set DEMO_LANG=en_US
demo\windows\run-demo.cmd

rem 配布のバイナリを使う
set CONDUCTOR=C:\path\to\conductor.exe
demo\windows\run-demo.cmd
```

PowerShell では、`set` の代わりに `$env:DEMO_PAUSE = "1"` のように書きます。

- 定義は `demo\windows\conductor.toml`、ジョブは `demo\windows\scripts\*.cmd` です。ジョブの終了コードは、UNIX 版と同じく `DEMO_<ジョブ名>_EXIT`(既定 0)で決まります。
- `DEMO_SLEEP`(各ジョブの待ち時間)は、整数の秒です(既定 1 秒。`0` で待たない)。小数は使えません。UNIX 版の既定(0.5 秒)とは違います。
- `DEMO_PAUSE` は、UNIX 版と同じく、値が `1` のときだけ Enter を待ちます。
- 画面に色は付きません(`watch` の出力も、色なしです)。
- `DEMO_LANG` には、`en_US` のようにロケール名を書きます(既定は `ja_JP.UTF-8`)。内部では `LANG` に設定し、`LC_ALL` と `LC_MESSAGES` は空にします。
- 日本語を正しく出すため、実行中だけコードページを UTF-8(65001)にして、終わりに元へ戻します。
