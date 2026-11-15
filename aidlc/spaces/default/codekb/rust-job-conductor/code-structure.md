# Code Structure

## ワークスペース

```
Cargo.toml                 ワークスペース(members = crates/*)
crates/core/               conductor-core
  src/types.rs             共有型(PipelineDefinition, JobDefinition, JobExecution, JobState, PipelineState, Decision ほか)
  src/error.rs             DefinitionError と Locate(行番号)
  src/definition/          intake, parse, depth, version, walk(検証と組み立て)
  src/flow/                classify, decide, next_step, resume(フロー判断)
crates/runtime/            conductor-runtime
  src/split.rs, runner.rs  command の分割、起動と待ち
  src/types.rs, error.rs   CommandLine, InterruptSignal, StateLocation, CommandError, StateError
  src/state/               reader, writer, record, consistency
  src/platform/            unix.rs, windows.rs
  src/bin/                 conductor-test-helper(テスト用)
crates/converter/          conductor-converter
  src/                     lib, types(Value), raw(素の木), intake, yaml_read, toml_read, resolve, scalar, yaml_write, toml_write, error
crates/cli/                conductor-cli(bin: conductor)
  src/                     main, lib, args, ctx, run, pipeline, aux_commands, deps, exit, types, panic_hook, platform/, report/
  src/bin/                 conductor-cli-test-helper(テスト用)
demo/                      デモ(conductor.toml, run-demo.sh, scripts/)
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
- 日本語の `//!` 文書とテスト名。BR・NFR などの要件 ID をコメントで参照する。

## command に関わる箇所(今回の変更の影響範囲)

| 箇所 | 役割 |
|------|------|
| `core/src/definition/walk.rs` `RawJob.command`, `job()`, `build_jobs()` | 読み取り・空判定・組み立て |
| `core/src/types.rs` `JobDefinition.command`, `JobExecution.command` | 型の定義 |
| `runtime/src/split.rs` `split`, `check_command` | 文字列の分割と事前検査 |
| `runtime/src/runner.rs` `base_execution`, `run_job` | 実行 |
| `runtime/src/error.rs` `CommandError(Kind)` | 分割の誤り(`UnclosedQuote`, `Empty`) |
| `cli/src/deps.rs` `Runtime::check_command(&str)` | 差し替え口 |
| `cli/src/ctx.rs` `prepare`, `report_execution` | 事前検査と起動失敗の表示 |
| `cli/src/report/mod.rs`, `messages.rs` | `CommandErrors`, `StartFailure` の表示と文 |
| ステート(`runtime/src/state/`) | command を持たない(変更不要の見込み) |
| `converter/` | スキーマを知らない汎用の木(配列は既に扱える見込み) |
