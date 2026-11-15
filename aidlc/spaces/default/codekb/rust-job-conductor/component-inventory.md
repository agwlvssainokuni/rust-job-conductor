# Component Inventory

## conductor-core
- **責務**: 定義ファイルの読み込みと検証(`definition/`: Intake, Parse, Depth, Version, Walk)、共有型(`types.rs`)、フローの判断(`flow/`: 分類、実行前の判断、次の動き、再開位置)。
- **依存**: `toml`, `thiserror`。内部依存なし。入出力は `definition/intake.rs` だけ。
- **command との関係**: `Command { Line, Argv }`、`JobDefinition.command`、`JobExecution.command`、`walk.rs` の読み取りと空判定。`flow/` は command を使わない。
- **ステートとの関係**: `PipelineState`・`JobState`(型のみ。保持は runtime)。フィールド名は `job_states` で、ファイルのキー `jobs` とは別。(今回は浅い確認)

## conductor-runtime
- **責務**: command の分割と変換(`split.rs`: `split`、`command_line`、`check_command(&Command)`)、ジョブの起動と待ち・シグナル転送(`runner.rs`)、ステートの読み書き・記録・照合(`state/`: `reader`、`writer`、`record`、`consistency`)、OS 固有(`platform/`)、中断の合図 `InterruptSignal`。
- **依存**: conductor-core, `toml`, `thiserror`, `libc`(UNIX)。版は v0.2.0。
- **command との関係**: 文字列は `split`、配列は要素をそのまま `CommandLine` にする。`CommandError`。ステートは command を持たない。
- **ステートとの関係(今回の変更の対象)**: ステートファイルの形を決めるのは `state/reader.rs`(`TOP_KEYS`、`JOB_KEYS`、`walk`、`walk_job`)と `state/writer.rs`(`serialize`、`job_table`)の 2 つ。`record.rs` と `consistency.rs` はファイルのキーを知らない。エラー文は `error.rs` の `reason`。テストは `tests/state_read.rs`、`state_write.rs`、`nfr_robust.rs`、`toml_depth.rs`、`flow_with_state.rs`。

## conductor-converter
- **責務**: YAML と TOML の相互変換。素の木(`raw.rs`)→ 展開(`resolve.rs`)→ `Value` → 書き出し。展開の上限(ノード 100,000、16 MiB、深さ 32)。
- **依存**: `toml`, `thiserror`, `saphyr-parser`。core に依存しない。
- **command・ステートとの関係**: 無し(スキーマを知らない。ステートは変換の対象ではない)。(今回は浅い確認)

## conductor-cli
- **責務**: 引数解析(`args.rs`, lexopt)、流れ(`run.rs`, `pipeline.rs`, `aux_commands.rs`, 共通部 `ctx.rs`)、シグナル(`platform/`)、表示(`report/`: メッセージ、watch の表、エスケープ)、終了コード(`exit.rs`)、パニックフック。差し替え口は `deps.rs`。
- **依存**: core, runtime, converter, `lexopt`, `signal-hook`(UNIX), `ctrlc`(Windows)。
- **command との関係**: `ctx.rs` の事前検査と起動失敗の表示、`deps.rs` の `check_command(&Command)`、`report/mod.rs` の `CommandErrors`・`StartFailure`。
- **ステートとの関係**: `read_state` などを呼ぶだけで、ステートのキーを知らない(`watch` は型から表を作る)。(今回は浅い確認)

## 補助
- `crates/runtime/src/bin/conductor-test-helper.rs`、`crates/cli/src/bin/conductor-cli-test-helper.rs`: テスト専用で配布に含めない。
- `demo/`: 動作のデモ(`run-demo.sh`、`conductor.toml`、`scripts/`、`README.md`)。`notify` の `command` は配列、ほかは文字列。ステートの中身には触れない(`watch` の表示を見せるだけ)。
