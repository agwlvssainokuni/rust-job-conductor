# Code Structure

## ワークスペース

```
Cargo.toml                 ワークスペース(members = crates/*)
crates/core/               conductor-core
  src/types.rs             共有型(PipelineDefinition, JobDefinition, Command, JobExecution, JobState, PipelineState, Decision ほか)
  src/error.rs             DefinitionError と Locate(行番号)
  src/definition/          intake, parse, depth, version, walk(検証と組み立て)
  src/flow/                classify, decide, next_step, resume(フロー判断)
crates/runtime/            conductor-runtime(v0.2.0)
  src/split.rs             split(文字列の分割)、command_line(Command → CommandLine)、check_command(&Command)
  src/runner.rs            run_job(起動と待ち)
  src/types.rs, error.rs   CommandLine, InterruptSignal, StateLocation, Durability, ConsistencyResult, CommandError, StateError
  src/state/               reader(読み込みと走査), writer(書き込みと reset), record(initial_state / apply_*), consistency(照合)
  src/platform/            unix.rs, windows.rs
  src/bin/                 conductor-test-helper(テスト用)
  tests/                   state_read, state_write, flow_with_state, runner_basic, runner_signal, toml_depth, nfr_content, nfr_robust, nfr_performance, nfr_source, common/mod.rs
crates/converter/          conductor-converter
  src/                     lib, types(Value), raw(素の木), intake, yaml_read, toml_read, resolve, scalar, yaml_write, toml_write, error
crates/cli/                conductor-cli(bin: conductor)
  src/                     main, lib, args, ctx, run, pipeline, aux_commands, deps, exit, types, panic_hook, platform/, report/
  src/bin/                 conductor-cli-test-helper(テスト用)
demo/                      デモ(conductor.toml, run-demo.sh, README.md, scripts/{extract,transform,load,notify}.sh)
.github/workflows/         ci, release, audit
```

## ファイルの分類

| 分類 | 主なファイル |
|------|-------------|
| Model | `core/src/types.rs`, `runtime/src/types.rs`, `converter/src/types.rs`, `converter/src/raw.rs`, `cli/src/types.rs` |
| Service(判断・実行) | `core/src/flow/*`, `runtime/src/runner.rs`, `runtime/src/split.rs`, `runtime/src/state/record.rs`, `cli/src/run.rs`, `pipeline.rs`, `aux_commands.rs`, `ctx.rs` |
| Repository(ステート入出力) | `runtime/src/state/reader.rs`, `writer.rs`, `consistency.rs` |
| Utility | `cli/src/report/escape.rs`, `converter/src/scalar.rs`, `core/src/definition/depth.rs` |
| Controller(入口) | `cli/src/main.rs`, `lib.rs`, `args.rs`, `exit.rs` |
| Configuration | `Cargo.toml`, `cli/src/deps.rs`(差し替え口), `.github/workflows/*` |
| Test | `crates/*/tests/`, 各 `src` の `#[cfg(test)]` |
| Static | `demo/`, `README.md`, 各 fixtures |

## コードのパターン

- trait による差し替え(`Runtime`, `Sink`, `Env`, `Signals`)。純粋な関数は trait を通さない。
- 入力の位置を保つ検証(`toml::Spanned`、`Locate` で行番号)と、誤りの集約(`Walker.errors`)。
- 状態機械による字句処理(`split.rs`、YAML 読み取りの明示スタック)で再帰を避ける。
- ステートの走査は、`toml::Table` と `toml::Value` を手で歩く(serde の派生ではない)。`TOP_KEYS`・`JOB_KEYS` の定数で未知のキーを拒む。書き込みも `toml::Table` を組んで `toml::to_string` で出す(`writer.rs` の `serialize`)。
- 日本語の `//!` 文書とテスト名。BR・NFR などの要件 ID をコメントで参照する。

## ステートのジョブ配列のキー(今回の変更 state-job-key の影響範囲)

ステートファイルの `jobs`(テーブルの配列)のキー名を、文字列リテラルで持つ箇所(runtime 内に閉じている)。

| 箇所 | 内容 |
|------|------|
| `runtime/src/state/reader.rs` L134 `TOP_KEYS`、L194 `walk` の `required(table, "jobs")` | 読み取りの本体 |
| `runtime/src/state/reader.rs` の `#[cfg(test)]`(L292・L297 の `VALID`、L396-400 の置換) | 単体テストの TOML と置換 |
| `runtime/src/state/writer.rs` L260 `serialize` の `"jobs".into()` | 書き込みの本体 |
| `runtime/src/error.rs` L117 `reason::ORDER_MISMATCH` | 利用者に見える文に `jobs` が入る |
| `runtime/tests/state_read.rs`(L33・L38)、`nfr_robust.rs`(L47・L52・L74-75・L130-131)、`toml_depth.rs`(L50・L55)、`state_write.rs`(L109・L116) | 統合テストの TOML、キー列、置換 |
| `README.md` | ステートの書式のキーは書かれていない(「ステートファイルの場所」の節は場所と動作だけ)。L168 の `[[jobs]]` は定義ファイルの旧書式の注記 |
| `demo/` | ステートの中身に触れない |

メモリ上の名前(`PipelineState.job_states`、`PipelineDefinition.jobs`)は別で、`consistency.rs` と `record.rs` の `.jobs` は定義側のフィールド。

## command に関わる箇所(command-array で実装済み)

| 箇所 | 役割 |
|------|------|
| `core/src/types.rs` `Command { Line(String), Argv(Vec<String>) }`、`Command::program_name` | 型の定義 |
| `core/src/definition/walk.rs` | 文字列または配列の読み取りと空判定 |
| `runtime/src/split.rs` `split`, `command_line`, `check_command(&Command)` | 文字列の分割、配列の変換、事前検査 |
| `runtime/src/runner.rs` `base_execution`, `run_job` | 実行(`command_line(&job.command)`) |
| `runtime/src/error.rs` `CommandError(Kind)` | 分割の誤り(`UnclosedQuote`, `Empty`) |
| `cli/src/deps.rs`, `ctx.rs`, `report/` | 差し替え口、事前検査、起動失敗の表示 |
| ステート(`runtime/src/state/`) | command を持たない |
