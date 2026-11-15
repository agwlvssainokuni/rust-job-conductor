# API Documentation

## CLI(外部 API)
```text
conductor run [-v] [--config <パス>] <ジョブID>
conductor pipeline [-v] [--config <パス>] <パイプライン名>
conductor watch [--config <パス>]
conductor reset [--config <パス>]
conductor convert <ファイル> --to <yaml|toml>
conductor --help | --version
```
- 既定の定義は実行ディレクトリの `conductor.toml`。ステートは定義と同じディレクトリの `<定義名>.state`。
- 終了コード: 0 成功/警告/スキップ、ジョブの値(エラー)、1(成功コード外の 0)、64 定義、65 ステート、66 拒否、67 変換、68 使い方、70 内部、127 起動失敗、128+番号 シグナル。全 OS で 0〜255 に丸める。
- 出力: 既定では conductor 自身の出力なし。エラー時のみ標準エラー。`-v` で結果と遷移を標準エラーへ。言語は `LC_ALL`/`LC_MESSAGES`/`LANG` の `ja` で日本語。色は UNIX 系の端末で `NO_COLOR` 未設定の `watch` だけ。

## 定義ファイル(TOML)
`format_version = 1`、`[pipeline] name`、`[[job]]`(`id`、`command`(文字列/配列)、`success_exit_codes`、`warning_exit_codes`、`[[job.transitions]] on/next`)。`next` は後ろのジョブID か `@end`。詳細は `README.md`。

## ライブラリ API(内部)
- conductor-core: `load_definition`、`parse_definition`、`decide_before_run`、`classify_and_decide`、`resume_point_on_interrupt`。
- conductor-runtime: `run_job`、`check_command`、`read_state`、`write_state`、`check_consistency`、`reset_state`、`initial_state`、`apply_outcome`、`apply_skip`、`apply_interrupt`。
- conductor-converter: `convert`、`convert_text`。
- conductor-cli: `run_cli`、`run_cli_with`、`parse_args`、`install_panic_hook`。
