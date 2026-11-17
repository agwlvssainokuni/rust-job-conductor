# API Documentation

## 外部 API: コマンドライン

```
conductor run [-v] [--config <パス>] [--run-id <ID>] [--remove-on-complete] <ジョブID>
conductor pipeline [-v] [--config <パス>] [--run-id <ID>] [--remove-on-complete] <パイプライン名>
conductor watch [--config <パス>] [--run-id <ID>]
conductor reset [--config <パス>] [--run-id <ID> | --all]
conductor validate [-v] [--config <パス>] [<パイプライン名>]
conductor convert <ファイル> --to <yaml|toml>
conductor --help | --version
```

詳細な動作は README.md「使い方」。

### 終了コード

0 成功/警告/スキップ、ジョブの値(エラー)、1、64 定義の誤り、65 ステートの誤り、66 拒否、67 変換の誤り、68 使い方の誤り、70 内部の誤り、127 起動失敗、128+番号 シグナル(README.md「終了コードの表」)。

### 定義ファイル(`conductor.toml`)

`format_version = 1`、`[pipeline]`(`name`、`description`、`run_id_env`、`remove_on_complete`)、`[[job]]`(`id`、`command`、`success_exit_codes`、`warning_exit_codes`、`[[job.transitions]]`)。ステートファイルは `<定義名>[.<run-id>].state`。

## 内部 API(ライブラリ)

| クレート | 公開する主なもの |
|----------|------------------|
| conductor-core | `load_definition`、`parse_definition`、`decide_before_run`、`classify_and_decide`、`resume_point_on_interrupt`、`RunId` |
| conductor-runtime | `run_job`、`check_command`、`read_state`、`read_state_raw`、`write_state`、`check_consistency`、`reset_state`、`reset_all_states`、`initial_state`、`apply_outcome` / `apply_skip` / `apply_interrupt` |
| conductor-converter | `convert`、`convert_text`、`ConversionRequest`、`ConvertedText`、`TargetFormat`、`ConversionError` |
| conductor-cli | `run_cli`、`run_cli_with`、`parse_args`、`install_panic_hook`、trait `Runtime` / `Sink` / `Env` / `Signals` |

すべて `publish = false`(クレートとしては公開しない)。
