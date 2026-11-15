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
| 1 | 全部が成功する(書いた順に、1 つずつ実行) |
| 2 | 警告で分岐する(`extract` が警告 → `transform` と `load` をスキップして `notify` へ) |
| 3 | エラーで止まる(`load` が失敗。終了コードがそのまま返り、`watch` で止まった位置が見える) |
| 4 | 同じコマンドをもう一度 → 失敗した `load` から再開(済んだジョブは、再実行しない) |
| 5 | ジョブを 1 つだけ実行する(`run`)と、`reset` |
| 6 | 定義ファイルを YAML に変換する(`convert`) |

- 定義は `conductor.toml`、ジョブは `scripts/*.sh` です。ジョブの終了コードは、環境変数 `DEMO_<ジョブ名>_EXIT`(例 `DEMO_LOAD_EXIT=2`)で決まります。`DEMO_SLEEP` で、各ジョブの待ち時間(既定 0.5 秒)を変えられます。
- `notify` の `command` は、文字列の配列(`["sh", "scripts/notify.sh"]`)で書いています。ほかのジョブは、文字列です。

## Windows 版

Windows では、`sh` のスクリプトは動きません。代わりに、`demo\windows\` の cmd 版を使います(PowerShell でも、コマンドプロンプトでも動きます)。見せる場面は、UNIX 版と同じ(場面 0〜6)です。

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

- 定義は `demo\windows\conductor.toml`、ジョブは `demo\windows\scripts\*.cmd` です。ジョブの終了コードは、UNIX 版と同じく `DEMO_<ジョブ名>_EXIT` で決まります。
- `DEMO_SLEEP`(各ジョブの待ち時間)は、整数の秒です(既定 1 秒。`0` で待たない)。小数は使えません。
- 画面に色は付きません(`watch` の出力も、色なしです)。
- `DEMO_LANG` には、`en_US` のようにロケール名を書きます(既定は `ja_JP.UTF-8`)。内部では `LANG` に設定し、`LC_ALL` と `LC_MESSAGES` は空にします。
- 日本語を正しく出すため、実行中だけコードページを UTF-8(65001)にして、終わりに元へ戻します。
