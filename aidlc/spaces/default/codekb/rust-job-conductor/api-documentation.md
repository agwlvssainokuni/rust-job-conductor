# API Documentation

## CLI(外部の API)

```text
conductor run [-v] [--config <パス>] [--run-id <ID>] <ジョブID>
conductor pipeline [-v] [--config <パス>] [--run-id <ID>] <パイプライン名>
conductor watch [--config <パス>] [--run-id <ID>]
conductor reset [--config <パス>] [--run-id <ID> | --all]
conductor validate [-v] [--config <パス>] [<パイプライン名>]
conductor convert <入力> --to <yaml|toml>
```

文法の表は `crates/cli/src/args.rs` の先頭の説明(`-v`: run・pipeline・validate、`--config`: convert 以外、`--run-id`: run・pipeline・watch・reset、`--all`: reset のみ、`--to`: convert のみ)。許可されない場所のオプションは `optionNotAllowed`(68)。`-v`・`--config`・`--to`・`--run-id`・`--all` の重複は `invalidValue`(68)。`--help`/`-V` は先に処理する。**`--remove-on-complete` はまだ存在しない**(現状の確認)。

## 終了コード(`exit.rs`、`README.md`)

0 成功・警告・スキップ / ジョブの終了コード(0〜255 に丸める)/ 64 定義の誤り / 65 ステートの誤り(読み・壊れ・食い違い・書き込み・初期化の失敗、`watch` の出力失敗)/ 66 拒否(順序外)/ 67 変換の誤り / 68 使い方の誤り / 70 内部の誤り / 128+番号 中断。

## 定義ファイル(TOML。`core/definition/walk.rs`)

- ルート: `format_version = 1`、`[pipeline]`、`[[job]]`。
- `[pipeline]`: `name`(必須)、`description`、`run_id_env`(環境変数名。`A-Za-z_` 始まり)。知らないキーは `unknownKey`(64)。
- `[[job]]`: `id`、`command`(文字列または配列)、`warning_exit_codes` などと `transitions`(`on` = success/warning、`next` = ジョブID か `@end`)。

## ステートファイル(`runtime/state/{reader,writer}.rs`)

`<定義ファイル名>.state` または `<定義ファイル名>.<run-id>.state`。TOML。上限 1 MiB。キーは `format_version`(1)、`run_id`(run ID ありのみ)、`definition_key`、`definition_path`、`pipeline_name`、`job_order`、`[[job]]`(`id`、`status`、`exit_code`、`skip_reason`)、`resume_job_id`、`end_reached`、`completed`。

## ライブラリの公開関数(契約 C3。`conductor-runtime`)

| 関数 | 役割 |
|------|------|
| `read_state(path, run_id)` | ステートを読む。なし・長さ 0・`completed = true` は `Ok(None)` |
| `write_state(path, run_id, state, durability)` | 原子的に書く(`Fast`/`Durable`) |
| `reset_state(path, run_id)` | 読まずに削除。なければ成功。失敗は `removeFailed` |
| `reset_all_states(path)` | run ID の有無によらず、その定義のステートを消す(`ResetAllReport`) |
| `initial_state` / `apply_outcome` / `apply_skip` / `apply_interrupt` | 純粋な記録。`apply_skip` と `apply_outcome` が `completed` を決める |
| `check_consistency` | ステートと定義の食い違い |

`conductor-cli` の `Runtime` trait(`deps.rs`)は、上のうち `read_state`、`write_state`、`reset_state`、`reset_all_states` と、`load_definition`、`check_command`、`run_job`、`run_id_env`、`convert` の 9 メソッド。
