# デモ

`conductor` の主な動きを、1 本のスクリプトで順に見せます(UNIX 系。`sh` を使います)。

```sh
demo/run-demo.sh                 # リポジトリの根から。実行ファイルが無ければ、リリースビルドを作る
DEMO_PAUSE=1 demo/run-demo.sh    # 各場面のあとで Enter を待つ(画面を見せながら話すとき)
DEMO_LANG=en_US.UTF-8 demo/run-demo.sh   # 表示を英語にする(既定は日本語)
CONDUCTOR=/path/to/conductor demo/run-demo.sh   # 配布のバイナリを使う
```

一時ディレクトリで動くので、リポジトリには何も残りません。

| 場面 | 見せること |
|------|-----------|
| 1 | 全部が成功する(書いた順に、1 つずつ実行) |
| 2 | 警告で分岐する(`extract` が警告 → `transform` と `load` をスキップして `notify` へ) |
| 3 | エラーで止まる(`load` が失敗。終了コードがそのまま返り、`watch` で止まった位置が見える) |
| 4 | 同じコマンドをもう一度 → 失敗した `load` から再開(済んだジョブは、再実行しない) |
| 5 | ジョブを 1 つだけ実行する(`run`)と、`reset` |
| 6 | 定義ファイルを YAML に変換する(`convert`) |

- 定義は `conductor.toml`、ジョブは `scripts/*.sh` です。ジョブの終了コードは、環境変数 `DEMO_<ジョブ名>_EXIT`(例 `DEMO_LOAD_EXIT=2`)で決まります。`DEMO_SLEEP` で、各ジョブの待ち時間(既定 0.5 秒)を変えられます。
- Windows では、`sh` のスクリプトは動きません。`conductor.toml` の `command` を、Windows のコマンドに書き換えてください。
