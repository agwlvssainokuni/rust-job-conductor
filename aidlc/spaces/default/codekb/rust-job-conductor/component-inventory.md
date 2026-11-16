# Component Inventory

## conductor-core

- 責務: 定義の読み込みと検証(`definition/`、`[pipeline]` は `walk.rs` の `PipelineHead`・`PIPELINE_KEYS`)、型(`PipelineDefinition`、`PipelineState`、`JobState`、`NextStep`)、run ID(`RunId`、`DEFAULT_RUN_ID_ENV`、`peek_run_id_env`)、フローの判断(`decide_before_run`、`classify_and_decide`、`resume_point_on_interrupt`)。純粋。
- 依存: toml、thiserror。

## conductor-runtime

- 責務: ジョブの起動とシグナル転送(`runner.rs`)、`command` の分割(`split.rs`)、ステートの読み(`reader.rs`、完了は空)・書き(`writer.rs`、`reset_state`)・記録(`record.rs`)・照合(`consistency.rs`)・一括削除(`reset_all.rs`)、`StateLocation`。
- 依存: conductor-core、toml、thiserror、libc(UNIX)。

## conductor-converter

- 責務: 定義の TOML と YAML の変換(`convert`、`convert_text`)。`[pipeline]` のキーを知らない汎用変換。conductor-core に依存しない。
- 依存: toml、thiserror、saphyr-parser。

## conductor-cli

- 責務: 引数の解釈(`args.rs`)、流れ(`run.rs`、`pipeline.rs`、`aux_commands.rs`、共通の `ctx.rs`)、差し替え口(`deps.rs`)、終了コード(`exit.rs`)、表示(`report/`: `Reporter`、`DisplayEvent`、`StateContext`、`watch.rs` の表、`messages.rs` の英日の文言)、シグナルの監視(`platform/`)、パニックのフック。バイナリ `conductor`。
- 依存: conductor-core、conductor-runtime、conductor-converter、lexopt、signal-hook(UNIX)、ctrlc(Windows)。

## demo

- 責務: 利用者向けのデモ(`run-demo.sh`、`windows/run-demo.cmd`、`conductor.toml`、ジョブのスクリプト、`README.md`)。場面 0〜9。`unix_demo.rs`・`windows_demo.rs` が実行して確かめる。
