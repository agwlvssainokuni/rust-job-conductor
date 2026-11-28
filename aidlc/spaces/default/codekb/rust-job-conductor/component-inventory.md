# コンポーネント一覧

見出しの名前は、`reverse-engineering-timestamp.md` の Scope of Analysis の `components` と一致させる。ファイルの配置と行の範囲は `code-structure.md`、外部の版は `technology-stack.md` を参照。

## conductor-core

- 責務: 定義ファイルの読み込み・検証と、純粋な判断、ステートの更新と試行の時刻、時刻の整形、照合、`.env` の解析と層の統合、図の構造と配置、`--dry-run` の計画、`watch --follow` の終わりの判定。
- 依存: 外部は `toml`、`thiserror` のみ。ほかのクレートには依存しない。`#![forbid(unsafe_code)]`。
- 場所: `crates/core/`

## core::types

- 責務: 共有の型(`PipelineDefinition`、`JobDefinition`、`EnvFileRef`、`JobExecution`、`JobState`、`Attempt`、`PipelineState`、`Transition`・`TransitionOn`・`TransitionTarget`、`RetryPolicy`、判断の結果)。`EnvVar` の `Debug` は値を伏せ字にする。
- 依存: `core::time`
- 場所: `crates/core/src/types.rs`

## core::time

- 責務: `Timestamp`、`format_rfc3339`・`parse_rfc3339`、`format_compact`、`duration_secs`、`job_duration`・`format_duration`。暦の計算は自前。
- 依存: `core::types`
- 場所: `crates/core/src/time.rs`

## core::error

- 責務: `DefinitionErrorKind`(34 変種)と `DefinitionError`、`Locate`(行番号の変換)、`quote_key`、表示の escape(`escape_display`、`MAX_KEY_DISPLAY_CHARS`)。`:176` の文書は cli の包みを古い名前 `report::escape` で書く。
- 依存: `thiserror`
- 場所: `crates/core/src/error.rs`

## core::run_id

- 責務: `RunId` の検査、環境変数の名前の規則(`is_valid_env_name`、`env_name_rule!`)、`peek_run_id_env`。
- 依存: `toml`
- 場所: `crates/core/src/run_id.rs`

## core::definition

- 責務: Intake → Parse → Depth → VersionCheck → Walk の順で、定義を `PipelineDefinition` にする。core で唯一ファイルに触れる(`intake`)。
- Walk(`definition/walk/`。private。外の入口は `walk::walk` だけで、`definition/mod.rs:69` が呼ぶ):
  - `walk/mod.rs`(695 行。この intent の対象): キーの表(`ROOT_KEYS`、`PIPELINE_KEYS` 9、`JOB_KEYS` 9、`TRANSITION_KEYS` 2)、中間の型(`PipelineHead`、`RawTransition`、`SuccessCodes`、`RawJob`)、`Walker { file, locate, errors }` と、そのメソッド(基本 `push`・`type_error`・`check_keys`、値の読み取り `string`・`boolean`・`exit_codes`・`command`、`[pipeline]` の `pipeline_section`・`pipeline_head`・`log_dir`、遷移 `transitions`・`transition_on`、`job`、ジョブをまたぐ検査 `cross_check`・`check_exit_codes`)、自由関数 `exit_code_of`・`integer_of`、入口 `walk`、組み立て `build_jobs`・`names_of`。
  - `walk/retry.rs`(366 行): `RETRY_KEYS` 5、`PipelineRetryDefault`・`RawRetry`・`JobRetrySetting`(`pub(super)`)、`impl Walker` の `job_retry`・`pipeline_retry`・`check_retry`(`pub(super)`)と補助、`build_retry`(`pub(super)`)。親から `push`・`check_keys`、`RawJob`・`SuccessCodes`、`exit_code_of`・`integer_of` を使う(`retry.rs:28`)。
  - `walk/env.rs`(281 行): `impl Walker` の `env_table`・`env_file`・`unset_env`・`check_unset_env`(`pub(super)`)と補助、`ENV_FILE_TABLE_KEYS`。親から `push`・`type_error`・`string`・`boolean`・`check_keys`、`PipelineHead`・`RawJob` を使う(`env.rs:24`)。
  - 誤りは位置で安定に並べるので、同じ位置の誤りの並びは呼ぶ順で決まる(`walk/mod.rs:641`)。
- 依存: `core::error`、`core::run_id`、`core::envfile`(予約名)、`core::types`、`toml`
- 外からの参照: `walk/mod.rs` の `PIPELINE_KEYS`・`JOB_KEYS` を `crates/cli/tests/spec_numbers.rs` がソースの文字列として読む(`:440`・`:639`・`:651`)。
- 場所: `crates/core/src/definition/`

## core::envfile

- 責務: `.env` の解析、層の統合(`merge_env`、`merge_env_names`、`merge_unset_names`)、`env_file_refs`、`check_unset_conflicts`、`EnvFileErrorKind`(14 変種)、予約名(`is_reserved_env_name`、`reserved_env_prefix`)。
- 依存: `core::error`、`core::run_id`、`core::types`、`thiserror`
- 場所: `crates/core/src/envfile/`

## core::flow

- 責務: 純粋な判断、ステートの記録、試行の時刻、動きの様子(`pipeline_activity`)。計画の模擬のための crate の中の入口(`decide_at`、`classify_and_decide_at`、`apply_outcome_in`、`apply_skip_in`)と目印(`SimulationMarks`)。
- 依存: `core::types`、`core::time`
- 場所: `crates/core/src/flow/`

## core::consistency

- 責務: ステートと定義の照合(`MismatchKind` 3 種)。
- 依存: `core::types`
- 場所: `crates/core/src/consistency.rs`

## core::graph

- 責務: 図の構造(`FlowGraph`)、辺、状態の重ね合わせ、レーンの配置。文字への変換は持たない(cli の `report`)。
- 依存: `core::flow`(`resolve_transition`)、`core::types`
- 場所: `crates/core/src/graph/`

## core::plan

- 責務: `--dry-run` の計画の型と模擬(`plan_pipeline`、`plan_run`)。
- 依存: `core::flow`、`core::envfile`、`core::types`
- 場所: `crates/core/src/plan/`

## conductor-runtime

- 責務: ジョブの起動と待ち、シグナルの転送、ログのファイル、時計、ステートの読み書きと削除、`.env` の読み取り、`init` のファイルの作成。
- 依存: `conductor-core`、`toml`、`thiserror`、`libc`(UNIX)。`#![deny(unsafe_code)]`。
- 場所: `crates/runtime/`

## runtime::types

- 責務: `CommandLine`、`InterruptSignal`、`StateLocation`、`Durability`。
- 依存: `conductor-core`(`RunId`)
- 場所: `crates/runtime/src/types.rs`

## runtime::error

- 責務: `CommandError`(2 種)、`StateError`(6 種)と固定の理由の文(`pub(crate) mod reason`。試行の 3 つを含む。`spec_numbers.rs` が読む)。
- 依存: `thiserror`
- 場所: `crates/runtime/src/error.rs`

## runtime::split

- 責務: `check_command`(文字列の command を状態機械で分割)。
- 依存: `conductor-core`、`runtime::types`、`runtime::error`
- 場所: `crates/runtime/src/split.rs`

## runtime::runner

- 責務: `run_job`(直接起動、`try_wait` のくり返し、シグナルの 1 回の転送、ログの複製のスレッド)。
- 依存: `runtime::log`、`runtime::platform`、`runtime::types`、`conductor-core`
- 場所: `crates/runtime/src/runner.rs`

## runtime::log

- 責務: `LogTarget`、`LogOutcome`、`safe_log_stem`、`open_log_file`。
- 依存: `conductor-core`、`runtime::platform`
- 場所: `crates/runtime/src/log.rs`

## runtime::clock

- 責務: `now`。時計に触れる唯一の場所。
- 依存: `conductor-core`(`Timestamp`)
- 場所: `crates/runtime/src/clock.rs`

## runtime::wait

- 責務: `wait_for`(合図を 50ms 以下の間隔で確認)。
- 依存: `runtime::types`
- 場所: `crates/runtime/src/wait.rs`

## runtime::envfile

- 責務: `load_env_files`(パスの解決、無くてもよいファイル、上限つきの読み取り)。
- 依存: `conductor-core`、`runtime::bounded_read`
- 場所: `crates/runtime/src/envfile.rs`

## runtime::bounded_read

- 責務: 通常のファイルだけを、上限 + 1 バイトまで読む。
- 依存: 標準ライブラリ
- 場所: `crates/runtime/src/bounded_read.rs`

## runtime::temp_file

- 責務: 一時ファイルと `replace_via_temp`。
- 依存: `runtime::platform`
- 場所: `crates/runtime/src/temp_file.rs`

## runtime::file_create

- 責務: `create_file`(`init` の排他的な作成、`--force` で置き換え。`CreateError` 5 種)。
- 依存: `runtime::temp_file`
- 場所: `crates/runtime/src/file_create.rs`

## runtime::state

- 責務: `read_state`・`read_state_raw`(厳格なキーの表 `TOP_KEYS` 10・`JOB_KEYS` 5・`ATTEMPT_KEYS` 2。`spec_numbers.rs` が読む)、`write_state`、`reset_state`、`reset_all_states`。
- 依存: `conductor-core`、`runtime::bounded_read`、`runtime::temp_file`、`toml`
- 場所: `crates/runtime/src/state/`

## runtime::platform

- 責務: プロセスグループ、シグナルの送信(`unsafe` の `libc::kill` 1 か所)、ディレクトリの同期、0600。
- 依存: `libc`(UNIX)
- 場所: `crates/runtime/src/platform/`

## conductor-converter

- 責務: YAML と TOML の相互変換(アンカー・エイリアス・マージキーの展開と上限)。`ConversionErrorKind` 6 種。
- 依存: `toml`、`thiserror`、`saphyr-parser`。`#![forbid(unsafe_code)]`。
- 場所: `crates/converter/`

## conductor-cli

- 責務: 実行ファイル `conductor`。引数、流れの組み立て、シグナル、表示(英日)、終了コード。モジュールはすべて private(`lib.rs:90-103`)。
- 依存: `conductor-core`、`conductor-runtime`、`conductor-converter`、`lexopt`、`signal-hook`(UNIX)、`ctrlc`(Windows)。`#![forbid(unsafe_code)]`。
- 場所: `crates/cli/`

## cli::types

- 責務: `CommandKind`(8 種、公開の `ALL`、`from_name`)、`CliCommand`、`ParsedArgs`、`GraphFormat`(3)、`InitTarget`、`RunIdArg`、`UsageError`・`UsageErrorKind`(8 種)、`Language`、`Level`(4)、`RUN_ID_OPTION`、`--interval` の定数(`pub(crate)`。`spec_numbers.rs` が読む)。
- 依存: `conductor-core`、`conductor-converter`(`TargetFormat`)
- 場所: `crates/cli/src/types.rs`

## cli::args

- 責務: 引数の解釈(`args.rs`、679 行。この intent の対象)。オプションの表(`Opt` 13、`Allowed` 3、`OptSpec`、`OPTIONS`、`ALL_BUT_CONVERT`、`RUN_AND_PIPELINE`、コンパイル時の検査 2 つと `is_long_spelling`、`spec`・`spec_by_long`・`allows`・`subject`)、検査(`Check`、`CHECKS` 25 行、`Given`、`Operand`・`operand_rule`)、字句(`Token`、`Special`、`parse_args`、`map_unexpected_value`、`to_text`)、組み立て(`build_command`。コマンドの枝は `match kind` の 5 つ)、値の解釈(`parse_assumptions`、`parse_graph_format`、`parse_interval`、`parse_run_id_option`、`parse_target`、`unexpected_argument`)。
- 依存: `cli::types`、`lexopt`、`conductor-core`(`Assumption`)、`conductor-converter`(`TargetFormat`)
- 外からの参照: `parse_args` だけ(`lib.rs:113`・`:150`、`src/tests/args_grammar.rs:23`)。ほかの項目は `args` の中だけ。
- 場所: `crates/cli/src/args.rs`

## cli::ctx

- 責務: `Ctx`、`Checked`、`Session`、実行の前の検査、シグナルの準備、ステートの書き込み・初期化とその失敗の表示。
- 依存: `cli::deps`、`cli::report`、`cli::run_id`、`conductor-core`、`conductor-runtime`
- 場所: `crates/cli/src/ctx.rs`

## cli::step

- 責務: 1 つのジョブの手順(`record_skip`、`run_step`、`run_job_with_retry`、`remove_if_requested`)と、実行の結果から `ResultDetail` を作る `result_detail`。
- 依存: `cli::ctx`、`cli::report`、`cli::exit`、`conductor-core`、`conductor-runtime`
- 場所: `crates/cli/src/step.rs`

## cli::flows

- 責務: `run`、`pipeline`、`dry_run`、`aux_commands`(`watch` と `follow_watch`、`reset`、`validate`、`convert`、`graph`、`init`)の流れ。
- 依存: `cli::ctx`、`cli::step`、`cli::report`、`cli::platform`、`conductor-core`、`conductor-runtime`
- 場所: `crates/cli/src/run.rs`、`pipeline.rs`、`dry_run.rs`、`aux_commands.rs`

## cli::run_id

- 責務: run ID の解決。
- 依存: `cli::ctx`、`cli::deps`、`conductor-core`
- 場所: `crates/cli/src/run_id.rs`

## cli::report

- 責務: 表示。`report/mod.rs`(746 行。この intent の対象)に、型(`StateContext` 5、`ResultDetail` 4、`DisplayEvent` 37、`FramePrefix` 3、`CLEAR_SCREEN`)、水準(`level`)、本体(`Reporter`。言語・色の遅延の判定、`show`、`render` の 35 の枝、`detect_language`)、1 行の組み立て(`show_path`、`error_line`、`place_of`、`definition_line`、`env_file_lines`、`env_warning_line`、`job_place`、`command_line`、`usage_text`、`state_line`、`conversion_line`、`job_result_line`)、エスケープ(`MAX_DISPLAY_CHARS`、`escape_name`、`escape_full`)、整形の補助(`push_line`)。子は `messages.rs`(英日の文言)、`watch.rs`、`graph_text.rs`、`graph_mermaid.rs`、`graph_dot.rs`、`plan_text.rs`、`init_template.rs`。
- 依存: `cli::types`、`cli::deps`(`Env`、`Sink`)、`cli::platform`(`color_supported`)、`conductor-core`、`conductor-runtime`(エラーの型)、`conductor-converter`(エラーの型)
- 外からの参照(`use` の場所):

| 名前 | 使う場所 |
|------|----------|
| `DisplayEvent` | `lib.rs:118`、`ctx.rs:32`、`run.rs:31`、`pipeline.rs:36`、`step.rs:48`、`dry_run.rs:26`、`aux_commands.rs:40`、`src/tests/report_messages.rs:25` |
| `Reporter` | `lib.rs:118`、`ctx.rs:32`、`src/tests/report_messages.rs:25` |
| `StateContext` | `ctx.rs:32`、`run.rs:31`、`pipeline.rs:36`、`step.rs:48`、`aux_commands.rs:40`、`report/messages.rs:28`、`src/tests/report_messages.rs:25` |
| `ResultDetail` | `step.rs:48`、`src/tests/report_messages.rs:25` |
| `FramePrefix` | `aux_commands.rs:40`、`src/tests/report_messages.rs:25` |
| `escape_name`・`escape_full` | `src/tests/escape_table.rs:20`、子の `watch.rs:25`・`graph_text.rs:29`・`graph_dot.rs:28`・`plan_text.rs:26`(`super::`) |
| `push_line` | 子の `graph_text.rs:35`・`plan_text.rs:34`・`watch.rs:30`(`super::`) |
| `messages::*` | `panic_hook.rs:32`、`src/tests/docs.rs:212`・`:523`、子 |
| 子モジュールの関数 | `aux_commands.rs:36-39`、`dry_run.rs:27`、`src/tests/{docs,graph_dot,graph_report,init_template,plan_report}.rs` |

- 場所: `crates/cli/src/report/`

## cli::deps

- 責務: 差し替え口の trait(`Runtime`、`Sink`、`Env`、`Signals`)と本番の実装。
- 依存: `cli::platform`、`conductor-core`、`conductor-runtime`、`conductor-converter`
- 場所: `crates/cli/src/deps.rs`

## cli::platform

- 責務: シグナルの監視、`color_supported`、`clear_screen_supported`。cli の `cfg(unix)`/`cfg(windows)` はここだけ。
- 依存: `signal-hook`、`ctrlc`、`conductor-runtime`(`InterruptSignal`)
- 場所: `crates/cli/src/platform/`

## cli::exit

- 責務: 終了コードの定数と、0〜255 への合わせ。
- 依存: `conductor-core`
- 場所: `crates/cli/src/exit.rs`

## cli::panic_hook

- 責務: パニックのとき固定の英語の文(`messages::INTERNAL_ERROR`)を出し、70 で終わる。
- 依存: `cli::report`(`messages`)
- 場所: `crates/cli/src/panic_hook.rs`

## test-helpers

- 責務: テスト専用の補助バイナリ(`conductor-test-helper`、`conductor-cli-test-helper`)。配布せず、カバレッジから除く。
- 依存: 標準ライブラリ
- 場所: `crates/runtime/src/bin/conductor-test-helper.rs`、`crates/cli/src/bin/conductor-cli-test-helper.rs`

## repo-file-tests

- 責務: リポジトリのファイル(ソース、README、仕様書、CI、デモ)を文字列として読み、実装と文書の一致や、ソースの性質を確かめるテスト。`crates/cli/tests/spec_numbers.rs`(仕様書の数。v0.10.3)、`crates/cli/src/tests/docs.rs`(README)、`crates/cli/tests/msrv_consistency.rs`(MSRV)、`crates/cli/tests/nfr_source.rs`・`crates/runtime/tests/nfr_source.rs`・`crates/converter/tests/nfr_security.rs`(ソースの検査)。ファイルのパスや行の形に依存する(`code-quality-assessment.md` の「パスに依存するテスト」)。
- 依存: 標準ライブラリだけ(`CARGO_MANIFEST_DIR` からの相対のパス)
- 場所: 上の各ファイル

## demo

- 責務: 主な動きを場面 0〜15 で見せるスクリプト(UNIX の `run-demo.sh`、Windows の `run-demo.cmd`。どちらも 16 個)と、その定義・`.env`・ジョブのスクリプト。場面の数は `spec_numbers.rs` が数える。
- 依存: ビルドした `conductor`
- 場所: `demo/`

## spec-docs

- 責務: 再構築用の仕様書。`rust/`(Rust 版の構成と挙動。モジュールの表でファイルの名前を書く)、`language-agnostic/`(言語に依らない挙動と受け入れ基準)、5 つの言語版の案内。各 13 文書。数の記述は `spec_numbers.rs` が実装と突き合わせる(モジュールの表は検査しない)。
- 依存: なし(実装を書き起こしたもの。実装が正)
- 場所: `docs/spec/`

## ci-workflows

- 責務: マージの関門(`ci.yml`)、リリース(`release.yml`)、脆弱性の検査(`audit.yml`)。この intent の 3 つのファイルのパスは書いていない。
- 依存: GitHub Actions(`actions/checkout@v6`、`dtolnay/rust-toolchain`、`taiki-e/install-action@v2`、`actions/upload-artifact@v7`、`actions/download-artifact@v8`)
- 場所: `.github/workflows/`
