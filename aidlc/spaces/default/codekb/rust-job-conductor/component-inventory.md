# コンポーネント一覧

## conductor-core

- 責務: 定義ファイルの読み込み・検証と、ジョブの結果の分類・次の動き・再開位置・リトライの判断。
- 依存: 外部は `toml`、`thiserror` のみ。ほかのクレートには依存しない。
- 場所: `crates/core/`

## core::definition

- 責務: Intake → Parse → Depth → VersionCheck → Walk の順で定義を `PipelineDefinition` にする。キーの許可、型、構造、遷移、終了コード、retry、名前を検査し、誤りをすべて集める。
- 依存: `core::error`、`core::run_id`(環境変数の名前の検査)、`core::types`
- 場所: `crates/core/src/definition/`

## core::flow

- 責務: 純粋な判断(`classify`、`decide_before_run`、`classify_and_decide`、`resume_point_on_interrupt`、`retry_decision`)。
- 依存: `core::types`
- 場所: `crates/core/src/flow/`

## core::types

- 責務: 共有する型(定義、実行結果、ステートの内容、判断の結果)。
- 依存: なし
- 場所: `crates/core/src/types.rs`

## core::error

- 責務: `DefinitionErrorKind`(30 種)と `DefinitionError`、表示用の escape(`sanitize`、`quote_key`)、行番号の変換(`Locate`)。
- 依存: `thiserror`
- 場所: `crates/core/src/error.rs`

## core::run_id

- 責務: run ID(`RunId`)と環境変数の名前の検査、定義からの `run_id_env` のゆるい取り出し。
- 依存: `toml`
- 場所: `crates/core/src/run_id.rs`

## conductor-runtime

- 責務: ジョブの直接起動とシグナルの転送、リトライの待機、ステートの保持・照合・初期化・記録。
- 依存: `conductor-core`、`toml`、`thiserror`、`libc`(UNIX)
- 場所: `crates/runtime/`

## runtime::runner

- 責務: `run_job`。command を分け、起動前の合図を確認し、`Command::new` で起動して `try_wait` のループで待つ。環境は継承する。
- 依存: `runtime::split`、`runtime::platform`、`runtime::types`
- 場所: `crates/runtime/src/runner.rs`

## runtime::split

- 責務: command の分け方(空白区切り、`'...'`、`"..."` とバックスラッシュの規則)。`check_command`。
- 依存: `conductor-core::Command`、`runtime::error`
- 場所: `crates/runtime/src/split.rs`

## runtime::platform

- 責務: OS 固有の呼び出し(プロセスグループ、シグナルの送信、シグナル終了の読み取り、ディレクトリの同期)。`unsafe` はここだけ。
- 依存: `libc`(UNIX)
- 場所: `crates/runtime/src/platform/`

## runtime::state

- 責務: ステートの読み込み(reader)、書き込みと初期化(writer、reset_all)、照合(consistency)、記録(record)。
- 依存: `conductor-core`、`toml`
- 場所: `crates/runtime/src/state/`

## runtime::wait

- 責務: リトライの待機。中断の合図を 50ms 間隔で確認する。
- 依存: `runtime::types`
- 場所: `crates/runtime/src/wait.rs`

## conductor-converter

- 責務: 定義ファイルの YAML と TOML の相互変換。アンカー・エイリアス・マージキーを展開し、上限(100,000 ノード、16 MiB、深さ 32)を守る。
- 依存: `toml`、`thiserror`、`saphyr-parser`。core には依存しない。
- 場所: `crates/converter/`

## conductor-cli

- 責務: コマンドの受け付け、流れの組み立て、シグナル、表示、終了コード。
- 依存: `conductor-core`、`conductor-runtime`、`conductor-converter`、`lexopt`、`signal-hook`(UNIX)、`ctrlc`(Windows)
- 場所: `crates/cli/`

## cli::args

- 責務: 引数の解釈。コマンドごとの許可・重複・併用の検査。run ID の決定。
- 依存: `lexopt`、`conductor-core::RunId`、`conductor-converter::TargetFormat`
- 場所: `crates/cli/src/args.rs`

## cli::ctx

- 責務: 実行の共通部分(`Ctx`、`Session`、`prepare`、`check_definition`、`report_execution`、状態の書き込みの失敗の表示)。
- 依存: `cli::deps`、`cli::report`、`cli::args`、core、runtime
- 場所: `crates/cli/src/ctx.rs`

## cli::flows

- 責務: `run`(個別実行)、`pipeline`(一括実行)、`retry`(リトライを含む起動)、`aux_commands`(watch、reset、validate、convert)の流れ。
- 依存: `cli::ctx`、core の純粋関数、runtime の `apply_*`
- 場所: `crates/cli/src/{run,pipeline,retry,aux_commands}.rs`

## cli::report

- 責務: `Reporter` と `DisplayEvent`、英日の文言表、escape、`watch` の表。
- 依存: core、runtime、converter のエラー型
- 場所: `crates/cli/src/report/`

## cli::deps

- 責務: 差し替え口の trait と本番の実装。
- 依存: core、runtime、converter、`cli::platform`
- 場所: `crates/cli/src/deps.rs`

## cli::platform

- 責務: シグナルの監視(UNIX は `signal-hook`、Windows は `ctrlc`)。プロセスで 1 回だけ登録。
- 依存: `signal-hook`/`ctrlc`、`conductor-runtime::InterruptSignal`
- 場所: `crates/cli/src/platform/`

## cli::exit

- 責務: 終了コードの定数と、0 から 255 への合わせ。
- 依存: なし
- 場所: `crates/cli/src/exit.rs`

## cli::panic_hook

- 責務: パニックのときの固定の文と終了コード 70。
- 依存: `cli::exit`、`cli::report::messages`
- 場所: `crates/cli/src/panic_hook.rs`

## test-helpers

- 責務: テスト専用の補助バイナリ(`conductor-test-helper`、`conductor-cli-test-helper`)。引数・環境変数・終了コード・シグナルの記録。配布しない。
- 依存: 標準ライブラリのみ
- 場所: `crates/runtime/src/bin/`、`crates/cli/src/bin/`
