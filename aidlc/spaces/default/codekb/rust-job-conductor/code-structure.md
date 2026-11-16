# Code Structure

## ワークスペース

`Cargo.toml`(members = `crates/*`)。ほかに `demo/`(デモのスクリプトと定義)、`.github/workflows/`、`README.md`。

## クレートとモジュール(責務は `component-inventory.md`)

- `crates/core/src/`: `lib.rs`、`types.rs`(`PipelineDefinition`、`PipelineState`、`JobState` ほか)、`run_id.rs`(`RunId`、`DEFAULT_RUN_ID_ENV`、`peek_run_id_env`)、`error.rs`、`definition/`(`mod`、`intake`、`parse`、`version`、`depth`、`walk`)、`flow/`(`mod`、`classify`、`decide`、`next_step`、`resume`)
- `crates/runtime/src/`: `lib.rs`、`types.rs`(`StateLocation`、`Durability`、`InterruptSignal`)、`error.rs`、`runner.rs`、`split.rs`、`platform/`、`state/`(`mod`、`reader`、`writer`、`record`、`consistency`、`reset_all`)、`bin/conductor-test-helper.rs`
- `crates/converter/src/`: `lib.rs`、`intake`、`raw`、`resolve`、`scalar`、`toml_read`・`toml_write`、`yaml_read`・`yaml_write`、`types`、`error`
- `crates/cli/src/`: `main.rs`、`lib.rs`、`args.rs`、`ctx.rs`、`run.rs`、`pipeline.rs`、`aux_commands.rs`、`deps.rs`、`exit.rs`、`types.rs`、`panic_hook.rs`、`platform/`、`report/`(`mod`、`messages`、`watch`、`escape`)、`bin/conductor-cli-test-helper.rs`
- テスト: 各クレートの `tests/`(`cli` は `flow_*`、`binary_*`、`args_grammar`、`report_messages`、`exit_codes`、`nfr_*`、`signals_*`、`unix_demo`、`windows_demo`、`common/`、`bin_support/`)

## ファイルの分類

- Model: `core/types.rs`、`runtime/types.rs`、`cli/types.rs`
- Controller 相当: `cli/{run,pipeline,aux_commands}.rs`(流れ)、`cli/args.rs`(引数)
- Service: `core/flow/*`(判断)、`runtime/state/record.rs`(記録、純粋)
- Repository 相当: `runtime/state/{reader,writer,reset_all}.rs`(ステートのファイル)
- Utility: `runtime/split.rs`、`cli/exit.rs`、`cli/report/escape.rs`
- Configuration: `core/definition/*`(定義の検証)
- Test: 上記の `tests/` と各 `#[cfg(test)]`

## コードの型(パターン)

- 差し替え口(trait)と偽の実装でテストする。
- 純粋な判断と、副作用の境界を分ける。
- エラーは、種類ごとの固定の文と、パス・対象の語だけで表示する(入力の値は含めない)。
- 日本語のコメント・テスト名。表示は `Text::new(英語, 日本語)`(`report/messages.rs`)。
