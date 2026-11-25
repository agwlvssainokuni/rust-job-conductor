# コンポーネント一覧

見出しの名前は、`reverse-engineering-timestamp.md` の Scope of Analysis の `components` と一致させる。

## conductor-core

- 責務: 定義ファイルの読み込み・検証と、純粋な判断(分類・次の動き・再開位置・リトライ)、ステートの更新、照合、`.env` の解析と層の統合、図の構造と配置、`--dry-run` の計画。
- 依存: 外部は `toml`、`thiserror` のみ。ほかのクレートには依存しない。`#![forbid(unsafe_code)]`。
- 場所: `crates/core/`

## core::types

- 責務: 共有の型(定義、実行結果、ステートの内容、判断の結果)。`EnvVar` の `Debug` は値を伏せ字にする。
- 依存: なし
- 場所: `crates/core/src/types.rs`

## core::error

- 責務: `DefinitionErrorKind`(33 種)と `DefinitionError`、表示の escape(`escape_display`、`quote_key`)、行番号の変換(`Locate`)、`.env` と共有する表示の書式(`render_error`)。
- 依存: `thiserror`
- 場所: `crates/core/src/error.rs`

## core::run_id

- 責務: `RunId` の検査、環境変数の名前の規則、定義からの `run_id_env` のゆるい取り出し(`peek_run_id_env`)。
- 依存: `toml`
- 場所: `crates/core/src/run_id.rs`

## core::definition

- 責務: Intake → Parse → Depth → VersionCheck → Walk の順で、定義を `PipelineDefinition` にする。キーの許可(`ROOT_KEYS`、`PIPELINE_KEYS`、`JOB_KEYS`、`RETRY_KEYS`)、型、構造、遷移、終了コード、retry、env、名前を検査し、誤りをすべて集める。core で唯一ファイルに触れる(`intake`)。
- 依存: `core::error`、`core::run_id`、`core::envfile`(予約名)、`core::types`、`toml`
- 場所: `crates/core/src/definition/`

## core::envfile

- 責務: `.env` の解析(`parse_env_file`)、層の統合(`merge_env`、`merge_env_names`)、読む対象の一覧(`env_file_refs`)、誤りの種類(13 種)と要約。値を持つ型の `Debug` は伏せ字。
- 依存: `core::error`、`core::run_id`、`core::types`、`thiserror`
- 場所: `crates/core/src/envfile/`

## core::flow

- 責務: 純粋な判断(`decide_from_state`、`classify_and_decide`、`resume_point_on_interrupt`、`retry_decision`)と、ステートの記録(`initial_state`、`apply_outcome`、`apply_skip`、`apply_interrupt`)。
- 依存: `core::types`
- 場所: `crates/core/src/flow/`

## core::consistency

- 責務: ステートと定義の照合(パイプライン名、ジョブIDの集合、並び)。
- 依存: `core::types`
- 場所: `crates/core/src/consistency.rs`

## core::graph

- 責務: 図の構造(`FlowGraph`、`FlowEdge`、`GraphNode`)、辺の取り出し(実行と同じ遷移の解決)、状態の重ね合わせ(`StateOverlay`)、レーンの配置(`GraphLayout::{Lanes, EdgeList}`)。文字への変換は持たない。
- 依存: `core::flow`(`resolve_transition`)、`core::types`
- 場所: `crates/core/src/graph/`

## core::plan

- 責務: `--dry-run` の計画の型(`ExecutionPlan`、`PlanEntry`、`Assumption` など)と、模擬(`plan_pipeline`、`plan_run`)。環境変数は名前だけ持つ。
- 依存: `core::flow`、`core::envfile`(`merge_env_names`)、`core::types`
- 場所: `crates/core/src/plan/`

## conductor-runtime

- 責務: ジョブの直接起動とシグナルの転送、リトライの待機、command の分割、ステートの保持・初期化、`.env` の読み取り、`init` のファイルの作成。
- 依存: `conductor-core`、`toml`、`thiserror`、`libc`(UNIX)。`#![deny(unsafe_code)]`。
- 場所: `crates/runtime/`

## runtime::types

- 責務: `CommandLine`(分割済みのプログラムと引数)、`InterruptSignal`(最初のシグナルの番号を持つ Atomic)、`StateLocation`(ステートのパスの決定)、`Durability`(`Fast`・`Durable`)。
- 依存: `conductor-core::RunId`
- 場所: `crates/runtime/src/types.rs`

## runtime::error

- 責務: `CommandError`(2 種)、`StateError`(6 種)と、固定の理由の文。
- 依存: `thiserror`
- 場所: `crates/runtime/src/error.rs`

## runtime::split

- 責務: command の分割(空白区切り、`'...'`、`"..."` とバックスラッシュの規則。状態機械)。`check_command`。
- 依存: `conductor-core::Command`、`runtime::error`、`runtime::types`
- 場所: `crates/runtime/src/split.rs`

## runtime::runner

- 責務: `run_job`。分割済みの command で `Command::new` を起動し、`try_wait` のくり返し(1ms から 10ms)で待ち、合図を子の pid に 1 回だけ転送する。環境は引き継いで足すだけ、標準入出力は引き継ぐ。
- 依存: `runtime::platform`、`runtime::types`、`conductor-core::JobExecution`
- 場所: `crates/runtime/src/runner.rs`

## runtime::wait

- 責務: リトライの待機(`wait_for`)。合図を 50ms 以下の間隔で確認する。
- 依存: `runtime::types`
- 場所: `crates/runtime/src/wait.rs`

## runtime::envfile

- 責務: `load_env_files`。`env_file` のパスを定義ファイルのディレクトリ基準で解決し、同じパスは 1 回だけ読み、誤りを集める。解析は core。
- 依存: `runtime::bounded_read`、`conductor-core::envfile`
- 場所: `crates/runtime/src/envfile.rs`

## runtime::bounded_read

- 責務: 通常のファイルだけを、上限 + 1 バイトまで読む(名前付きパイプで待たない)。`.env` とステートが共有する。
- 依存: 標準ライブラリ
- 場所: `crates/runtime/src/bounded_read.rs`

## runtime::temp_file

- 責務: 一時ファイルの名前と排他の作成、`replace_via_temp`(書いて置き換える)。ステートと `init` が共有する。
- 依存: `runtime::platform`
- 場所: `crates/runtime/src/temp_file.rs`

## runtime::file_create

- 責務: `create_file`(`init` の出力先を排他的に作る、`--force` で置き換える)。親のディレクトリは作らない。
- 依存: `runtime::temp_file`
- 場所: `crates/runtime/src/file_create.rs`

## runtime::state

- 責務: ステートの読み込み(`reader`。厳格なキーの表と矛盾の検査)、書き込みと初期化(`writer`)、すべての初期化(`reset_all`)。
- 依存: `conductor-core`、`toml`、`runtime::bounded_read`、`runtime::temp_file`、`runtime::platform`
- 場所: `crates/runtime/src/state/`

## runtime::platform

- 責務: OS 固有の呼び出し(プロセスグループ、シグナルの送信、シグナル終了の読み取り、ディレクトリの同期)。`unsafe` は `unix.rs` の `libc::kill` 1 か所だけ。
- 依存: `libc`(UNIX)
- 場所: `crates/runtime/src/platform/`

## conductor-converter

- 責務: 定義ファイルの YAML と TOML の相互変換。アンカー・エイリアス・マージキーを展開し、上限(100,000 ノード、16 MiB、深さ 32)を守る。定義の書式は知らない。
- 依存: `toml`、`thiserror`、`saphyr-parser`。core には依存しない。`#![forbid(unsafe_code)]`。
- 場所: `crates/converter/`

## conductor-cli

- 責務: コマンドの受け付け、流れの組み立て、シグナル、表示(英日)、終了コード。実行ファイル `conductor`。
- 依存: `conductor-core`、`conductor-runtime`、`conductor-converter`、`lexopt`、`signal-hook`(UNIX)、`ctrlc`(Windows)。`#![forbid(unsafe_code)]`。
- 場所: `crates/cli/`

## cli::types

- 責務: `CommandKind`(8 種と `ALL`)、`CliCommand`、`GraphFormat`、`GraphOptions`、`DryRunOptions`、`InitOptions`、`InitTarget`、`RunIdArg`、`UsageError`、`ParsedArgs`、`Language`。
- 依存: `conductor-core`(`RunId`、`Assumption`)、`conductor-converter::TargetFormat`
- 場所: `crates/cli/src/types.rs`

## cli::args

- 責務: 引数の解釈。オプションの表(`OPTIONS`)、コマンドごとの許可・重複・併用の検査、値の検査(`--format`、`--run-id`、`--assume`、`--to`、`init` の名前)。
- 依存: `lexopt`、`cli::types`
- 場所: `crates/cli/src/args.rs`

## cli::ctx

- 責務: 流れの共通部分(`Ctx`、`Checked`、`Session`、`FlowTarget`)。実行の前の検査の順序(シグナル → 定義 → command → ジョブID → run ID → `.env` → ステート → 照合)、`check_definition_and_env`(`validate`・`graph`)、ステートの書き込み・初期化とその失敗の表示、実効の `remove_on_complete`。
- 依存: `cli::deps`、`cli::report`、`cli::run_id`、core、runtime
- 場所: `crates/cli/src/ctx.rs`

## cli::step

- 責務: 1 つのジョブの手順(`record_skip`、`run_step`、リトライつきの実行 `run_job_with_retry`、`remove_if_requested`、実行後の表示)。試行ごとに `merge_env` で環境を作り直す。
- 依存: `cli::ctx`、`cli::report`、`cli::exit`、core(`merge_env`、`retry_decision`、`classify_and_decide`、`apply_*`)
- 場所: `crates/cli/src/step.rs`

## cli::flows

- 責務: コマンドごとの流れ。`run`(個別実行)、`pipeline`(一括実行、ジョブのあいだの中断)、`dry_run`(計画)、`aux_commands`(`watch`、`reset`、`validate`、`convert`、`graph`、`init`)。
- 依存: `cli::ctx`、`cli::step`、`cli::report`、core の純粋な関数
- 場所: `crates/cli/src/{run,pipeline,dry_run,aux_commands}.rs`

## cli::run_id

- 責務: `--run-id` と環境変数から run ID を決める(`resolve_run_id`、`aux_run_id`)。
- 依存: `cli::deps::Env`、`core::run_id`
- 場所: `crates/cli/src/run_id.rs`

## cli::report

- 責務: `Reporter` と `DisplayEvent`(33 種)、英日の文言の表、escape、`watch` の表、図(text、Mermaid)、計画、ひな形の整形。言語(`LC_ALL`・`LC_MESSAGES`・`LANG`)と色(UNIX 系の端末、`NO_COLOR`)は遅延で判定。
- 依存: core・runtime・converter のエラー型と図・計画の型
- 場所: `crates/cli/src/report/`

## cli::deps

- 責務: 差し替え口の trait(`Runtime` 13 メソッド、`Sink`、`Env`、`Signals`)と本番の実装。
- 依存: core、runtime、converter、`cli::platform`
- 場所: `crates/cli/src/deps.rs`

## cli::platform

- 責務: シグナルの監視(UNIX は `signal-hook`、Windows は `ctrlc`)。プロセスで 1 回だけ登録。色が使える OS か。
- 依存: `signal-hook`/`ctrlc`、`conductor-runtime::InterruptSignal`
- 場所: `crates/cli/src/platform/`

## cli::exit

- 責務: 終了コードの定数(0、64〜70)と、0〜255 への合わせ、シグナルの終了コード。
- 依存: `conductor-core::JobOutcome`
- 場所: `crates/cli/src/exit.rs`

## cli::panic_hook

- 責務: パニックのときの固定の英語の文と終了コード 70。
- 依存: `cli::exit`
- 場所: `crates/cli/src/panic_hook.rs`

## test-helpers

- 責務: テスト専用の補助バイナリ(`conductor-test-helper`、`conductor-cli-test-helper`)。引数・環境変数・終了コード・シグナル・時刻・出力の記録。配布しない(`release.yml` は `--bin conductor` だけをビルド)。カバレッジの対象外。
- 依存: 標準ライブラリのみ
- 場所: `crates/runtime/src/bin/`、`crates/cli/src/bin/`

## ci-workflows

- 責務: マージの関門(`ci.yml`: lint、build-check、test、msrv-test、coverage、docs-check、startup-bench)、リリース(`release.yml`: verify-tag、checks、build、draft-release)、脆弱性の検査(`audit.yml`)。
- 依存: GitHub Actions(`actions/checkout@v6`、`dtolnay/rust-toolchain`、`taiki-e/install-action@v2`、`actions/upload-artifact@v7`、`actions/download-artifact@v8`)
- 場所: `.github/workflows/`
