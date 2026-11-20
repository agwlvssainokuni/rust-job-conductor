# コード構造

## ワークスペース

`Cargo.toml`(`members = ["crates/*"]`、resolver 3、edition 2024、rust-version 1.100)。4 クレート。すべて Apache-2.0、`publish = false`(cli のみ明示)。

```text
.
├── Cargo.toml / Cargo.lock
├── crates/
│   ├── core/       conductor-core
│   ├── runtime/    conductor-runtime
│   ├── converter/  conductor-converter
│   └── cli/        conductor-cli(実行ファイル conductor)
├── demo/           定義の例とスクリプト(Unix と windows/)
├── docs/spec/      仕様書(7 版)
├── .github/workflows/  ci.yml, release.yml, audit.yml
└── README.md, LICENSE, .gitignore, .idea/(追跡)
```

## conductor-core(crates/core/src)

| ファイル | 分類 | 役割 |
|----------|------|------|
| `lib.rs` | Configuration | 公開関数の表と再エクスポート |
| `types.rs` | Model | `PipelineDefinition`、`JobDefinition`、`Command`、`RetryPolicy`、`Transition*`、`JobExecution`、`JobState`、`PipelineState`、`Decision*`、`Classification`、`JobOutcome`、`NextStep`、`RunMode`、`ResumePoint` |
| `error.rs` | Utility | `DefinitionErrorKind`(30 種、`code()`、`ALL`)、`DefinitionError`、`sanitize`、`quote_key`、`Locate`(行番号) |
| `run_id.rs` | Utility | `RunId`、`is_valid_env_name`、`peek_run_id_env`、`DEFAULT_RUN_ID_ENV` |
| `definition/mod.rs` | Service | `load_definition`、`parse_definition`(Intake → Parse → Depth → VersionCheck → Walk) |
| `definition/intake.rs` | Utility | 場所の決定と 1 MiB 上限の読み取り(core で唯一のファイル入出力) |
| `definition/parse.rs` | Utility | `toml::de::DeTable` で位置つきの木にする |
| `definition/depth.rs` | Utility | 入れ子 32 段の上限 |
| `definition/version.rs` | Utility | `format_version == 1` の確認 |
| `definition/duration.rs` | Utility | `<整数><s|m|h>` を秒に(1 秒から 24 時間) |
| `definition/walk.rs` | Service | キー・型・構造・遷移・終了コード・retry の検査と定義の組み立て(801 行) |
| `flow/classify.rs` | Service | 分類と終了コード(127・128+n・1) |
| `flow/decide.rs` | Service | `decide_before_run`(7 段の判断) |
| `flow/next_step.rs` | Service | `classify_and_decide`(遷移、`@end`、完了) |
| `flow/resume.rs` | Service | `resume_point_on_interrupt` |
| `flow/retry.rs` | Service | `retry_decision`、待ち時間の計算(飽和演算) |

## conductor-runtime(crates/runtime/src)

| ファイル | 分類 | 役割 |
|----------|------|------|
| `lib.rs` | Configuration | 公開関数の表、`#![deny(unsafe_code)]` |
| `runner.rs` | Service | `run_job`(起動と待ち。`wait_loop`、`Waitable` trait) |
| `split.rs` | Utility | `command_line`、`check_command`(状態機械による分割) |
| `wait.rs` | Utility | `wait_for`(50ms 間隔で中断を確認) |
| `types.rs` | Model | `CommandLine`、`InterruptSignal`、`StateLocation`、`ConsistencyResult`、`MismatchKind`、`Durability` |
| `error.rs` | Utility | `CommandError`、`StateError` と固定の理由の文 |
| `platform/{mod,unix,windows}.rs` | Configuration | `configure_spawn`、`forward_signal`、`termination_signal`、`sync_dir`。`unsafe` は unix.rs の `libc::kill` のみ |
| `state/reader.rs` | Repository | `read_state`、`read_state_raw`。1 MiB 上限、版の確認、キーの検査、矛盾の検査 |
| `state/writer.rs` | Repository | `write_state`(一時ファイル、同期、置き換え)、`reset_state` |
| `state/reset_all.rs` | Repository | `reset_all_states`(`--all`) |
| `state/consistency.rs` | Service | `check_consistency` |
| `state/record.rs` | Service | `initial_state`、`apply_outcome`、`apply_skip`、`apply_interrupt` |
| `bin/conductor-test-helper.rs` | Test | テスト専用バイナリ |

## conductor-converter(crates/converter/src)

`lib.rs`(`convert`、`convert_text`)、`types.rs`(`Value`、`TargetFormat`、`ConversionRequest`、`ConvertedText`)、`error.rs`(6 種)、`intake.rs`、`raw.rs`(定数 `MAX_NODES` 等)、`yaml_read.rs`、`toml_read.rs`、`resolve.rs`(展開、`root_check`)、`scalar.rs`、`yaml_write.rs`、`toml_write.rs`。

## conductor-cli(crates/cli/src)

| ファイル | 分類 | 役割 |
|----------|------|------|
| `main.rs` | Configuration | 薄い入口。パニックのフックを設定し `run_cli` の結果で exit |
| `lib.rs` | Controller | `run_cli`、`run_cli_with`(コマンドの振り分け) |
| `args.rs` | Controller | `parse_args`、`resolve_run_id`(`lexopt`、許可表の検査) |
| `types.rs` | Model | `CommandKind`、`CliCommand`、`ParsedArgs`、`UsageError(Kind)`、`Language`、`Level` |
| `ctx.rs` | Service | `Ctx`、`Session`、`prepare`、`check_definition`、`report_execution`、`aux_run_id` |
| `run.rs` | Service | `run_individual` |
| `pipeline.rs` | Service | `run_pipeline` |
| `retry.rs` | Service | `run_job_with_retry` |
| `aux_commands.rs` | Service | `run_watch`、`run_reset`、`run_validate`、`run_convert` |
| `exit.rs` | Utility | 終了コードの定数と `exit_code_for`、`signal_exit_code` |
| `deps.rs` | Configuration | trait と本番の実装(`SystemRuntime` など) |
| `report/mod.rs` | View | `Reporter`、`DisplayEvent`、`StateContext`、`ResultDetail` |
| `report/messages.rs` | View | 英日の文言表、`USAGE`、`definition_kind`、`usage_kind` など |
| `report/escape.rs`、`report/watch.rs` | View | escape(名前 64 文字)、`watch` の表 |
| `platform/{mod,unix,windows}.rs` | Configuration | シグナルの監視(プロセスで 1 回)、色の可否 |
| `panic_hook.rs` | Middleware | パニックで固定の文を出し 70 で終わる |
| `bin/conductor-cli-test-helper.rs` | Test | テスト専用バイナリ |

## コードのパターン

- 日本語のドキュメントコメントと日本語のテスト名。契約(C1〜C4)、BR/NFR/FR の番号がコメントに残る。
- 純粋関数 + 差し替え口の trait。エラーは値で返し、パニックしない。
- 固定の文だけでエラー表示を組み立てる(入力の値を入れない)。
- 状態機械(再帰を使わない): `split`、YAML の読み取りと展開。
- 定義の検査は「誤りをすべて集める」方式(`Walker.errors`)。ファイル・文法・版の誤りだけは 1 件で終える。

## テストの配置

| 場所 | 内容 |
|------|------|
| `crates/core/tests/` | `definition_{valid,errors,retry}.rs`、`flow_sequences.rs`、`retry_flow.rs`、`library_capability.rs`、`nfr_{performance,security}.rs`、fixtures(`valid_pipeline.toml`、`multiple_errors.toml`) |
| `crates/runtime/tests/` | `runner_{basic,signal}.rs`、`state_{read,read_raw,write}.rs`、`flow_with_state.rs`、`run_id_state.rs`、`toml_depth.rs`、`wait_interruptible.rs`、`nfr_{content,performance,robust,source}.rs` |
| `crates/cli/tests/` | `args_grammar.rs`、`flow_{run,pipeline,aux,retry,validate,run_id}.rs`、`report_messages.rs`、`escape_table.rs`、`exit_codes.rs`、`binary_{basic,run_id,signal}.rs`、`signals_{order,real}.rs`、`panic_hook.rs`、`unix_demo.rs`、`windows_demo.rs`、`nfr_{perf,robust,source}.rs`、`msrv_consistency.rs`、`docs_retry.rs`、`common/`、`bin_support/` |
| `crates/converter/tests/` | `convert_{ac,roundtrip}.rs`、`read_resolve.rs`、`nfr_security.rs`、fixtures |
| 各 `src/*.rs` | `#[cfg(test)]` の単体テスト |

## ドキュメントの配置

`README.md`、`docs/spec/{csharp,go,language-agnostic,powershell,python,rust,typescript}/00-README.md` から `12-rebuild-guide.md`(各 13 文書)、`demo/README.md`、`crates/cli/tests/WINDOWS_CTRL_C_CHECKLIST.md`。
