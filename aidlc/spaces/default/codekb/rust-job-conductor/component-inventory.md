# コンポーネント一覧

見出しの名前は、`reverse-engineering-timestamp.md` の Scope of Analysis の `components` と一致させる。ファイルの配置と分類は `code-structure.md`、外部の版は `technology-stack.md` を参照。

## conductor-core

- 責務: 定義ファイルの読み込み・検証と、純粋な判断(分類・次の動き・再開位置・リトライ)、ステートの更新と試行の時刻、時刻の整形、照合、`.env` の解析と層の統合、図の構造と配置、`--dry-run` の計画、`watch --follow` の終わりの判定。
- 依存: 外部は `toml`、`thiserror` のみ。ほかのクレートには依存しない。`#![forbid(unsafe_code)]`。
- 場所: `crates/core/`

## core::types

- 責務: 共有の型(`PipelineDefinition`(`log_dir`、`unset_env` を含む)、`JobDefinition`(実効の `retry`、`unset_env`)、`EnvFileRef { path, required }`、`JobExecution`、`JobState`(`attempts`)、`Attempt { started_at, ended_at }`、`PipelineState`、判断の結果)。`EnvVar` の `Debug` は値を伏せ字にする。フィールドは公開。
- 依存: `core::time`
- 場所: `crates/core/src/types.rs`

## core::time

- 責務: `Timestamp`(UTC の UNIX 秒。1970〜9999 年)、`format_rfc3339`・`parse_rfc3339`(ステート)、`format_compact`(ログの名前)、`duration_secs`、`job_duration`・`format_duration`(`watch` の所要時間)。暦の計算は自前。時計には触れない。
- 依存: `core::types`
- 場所: `crates/core/src/time.rs`

## core::error

- 責務: `DefinitionErrorKind`(34 種)と `DefinitionError`、表示の escape(`escape_display`)、行番号の変換。
- 依存: `thiserror`
- 場所: `crates/core/src/error.rs`

## core::run_id

- 責務: `RunId` の検査、環境変数の名前の規則、定義からの `run_id_env` のゆるい取り出し(`peek_run_id_env`)。
- 依存: `toml`
- 場所: `crates/core/src/run_id.rs`

## core::definition

- 責務: Intake → Parse → Depth → VersionCheck → Walk の順で、定義を `PipelineDefinition` にする。キーの許可、型、構造、遷移、終了コード、retry(`[pipeline.retry]` の既定の当てはめと `retry = false`)、env・env_file(テーブルの項目)・unset_env、`log_dir` を検査し、誤りをすべて集める。core で唯一ファイルに触れる(`intake`)。
- 依存: `core::error`、`core::run_id`、`core::envfile`(予約名)、`core::types`、`toml`
- 場所: `crates/core/src/definition/`

## core::envfile

- 責務: `.env` の解析(`parse_env_file`)、層の統合(`merge_env`、名前だけの `merge_env_names`、消す名前の `merge_unset_names`)、読む対象の一覧(`env_file_refs`)、`unset_env` と足す名前の重なりの検査(`check_unset_conflicts`)、誤りの種類(14 種)と要約。
- 依存: `core::error`、`core::run_id`、`core::types`、`thiserror`
- 場所: `crates/core/src/envfile/`

## core::flow

- 責務: 純粋な判断(`decide_from_state`、`classify_and_decide`、`resume_point_on_interrupt`、`retry_decision`)、ステートの記録(`initial_state`、`apply_outcome`、`apply_skip`、`apply_interrupt`)、試行の時刻(`start_attempt`、`end_attempt`、`cancel_attempt`)、動きの様子(`pipeline_activity`)。v0.10.1 から、計画の模擬のための crate の中の入口(`decide_at`、`classify_and_decide_at`、`apply_outcome_in`、`apply_skip_in`)と、位置で引く補助(`record_at`、`status_at`、`position_of`)と目印(`SimulationMarks`)を持つ。公開の関数は目印を作り直して同じ本体を呼ぶ。
- 依存: `core::types`、`core::time`
- 場所: `crates/core/src/flow/`(`activity.rs`、`attempt.rs`、`classify.rs`、`decide.rs`、`mod.rs`、`next_step.rs`、`record.rs`、`resume.rs`、`retry.rs`)

## core::consistency

- 責務: ステートと定義の照合(パイプライン名、ジョブIDの集合、並び)。
- 依存: `core::types`
- 場所: `crates/core/src/consistency.rs`

## core::graph

- 責務: 図の構造(`FlowGraph`)、辺の取り出し(実行と同じ遷移の解決)、状態の重ね合わせ、レーンの配置(`GraphLayout`)。文字への変換は持たない。
- 依存: `core::flow`(`resolve_transition`)、`core::types`
- 場所: `crates/core/src/graph/`

## core::plan

- 責務: `--dry-run` の計画の型(`ExecutionPlan`、`PlanEntry`(足す名前と消す名前)、`PlanDisposition`、`PlanEnd`)と模擬(`plan_pipeline`、`plan_run`)。本番と同じ判断と記録を使い、試行は記録しない。`plan_pipeline` は位置と目印を持ち回り、`flow` の内部の入口を呼ぶ(v0.10.1)。`plan_run` は公開の入口を使う。
- 依存: `core::flow`、`core::envfile`、`core::types`
- 場所: `crates/core/src/plan/`

## conductor-runtime

- 責務: ジョブの起動と待ち、シグナルの転送、ログのファイル、時計、ステートの読み書きと削除、`.env` の読み取り、`init` のファイルの作成。
- 依存: `conductor-core`、`toml`、`thiserror`、`libc`(UNIX)。`#![deny(unsafe_code)]`。
- 場所: `crates/runtime/`

## runtime::types

- 責務: `CommandLine`、`InterruptSignal`(Atomic。立ったら下りない)、`StateLocation`、`Durability`(`Fast`・`Durable`)。
- 依存: `conductor-core`(`RunId`)
- 場所: `crates/runtime/src/types.rs`

## runtime::error

- 責務: `CommandError`(2 種)、`StateError`(6 種)と固定の理由の文(試行の 3 つ `NO_ATTEMPT`・`BAD_TIME`・`UNFINISHED_ATTEMPT` を含む)。
- 依存: `thiserror`
- 場所: `crates/runtime/src/error.rs`

## runtime::split

- 責務: `check_command`(文字列の command を状態機械で分割。配列はそのまま)。
- 依存: `conductor-core`(`Command`)、`runtime::types`、`runtime::error`
- 場所: `crates/runtime/src/split.rs`

## runtime::runner

- 責務: `run_job`(消してから足す環境、直接起動、1〜10ms の `try_wait` のくり返し、シグナルの 1 回の転送)。ログの指定があるときは、出力をパイプにし、流れごとの読み取りのスレッドで端末とファイルへ書き、終了後に最大 2 秒待つ。「止める」印は読む前と読んだ後に見る(v0.10.1。`runner.rs:315`・`324`)。
- 依存: `runtime::log`、`runtime::platform`、`runtime::types`、`conductor-core`
- 場所: `crates/runtime/src/runner.rs`

## runtime::log

- 責務: `LogTarget`、`LogOutcome`、`safe_log_stem`(安全な名前。予約名は前後の空白を除いて比べる。v0.10.1、`log.rs:90`)、`open_log_file`(ディレクトリの作成、`create_new`、0600、`-2`〜`-1000`)。
- 依存: `conductor-core`(`Timestamp`、`format_compact`)、`runtime::platform`
- 場所: `crates/runtime/src/log.rs`

## runtime::clock

- 責務: `now`(`SystemTime` → `Timestamp`。範囲の外は端に寄せる)。時計に触れる唯一の場所。
- 依存: `conductor-core`(`Timestamp`)
- 場所: `crates/runtime/src/clock.rs`

## runtime::wait

- 責務: `wait_for`(リトライと `watch --follow` の待ち。合図を 50ms 以下の間隔で確認)。
- 依存: `runtime::types`
- 場所: `crates/runtime/src/wait.rs`

## runtime::envfile

- 責務: `load_env_files`(パスの解決、`required = false` の無いファイルの読み飛ばし、上限つきの読み取り、UTF-8。解析は core)。
- 依存: `conductor-core`(`envfile`)、`runtime::bounded_read`
- 場所: `crates/runtime/src/envfile.rs`

## runtime::bounded_read

- 責務: 通常のファイルだけを、上限 + 1 バイトまで読む(`.env` とステートが共有)。
- 依存: 標準ライブラリ
- 場所: `crates/runtime/src/bounded_read.rs`

## runtime::temp_file

- 責務: 一時ファイルの名前と作成、`replace_via_temp`(ステートと `init` が共有)。
- 依存: `runtime::platform`
- 場所: `crates/runtime/src/temp_file.rs`

## runtime::file_create

- 責務: `create_file`(`init` の排他的な作成、`--force` で置き換え)。
- 依存: `runtime::temp_file`
- 場所: `crates/runtime/src/file_create.rs`

## runtime::state

- 責務: `read_state`・`read_state_raw`(厳格なキーの表、試行の検査)、`write_state`(原子的な置き換え。試行を書く)、`reset_state`、`reset_all_states`、古い一時ファイルの後始末。
- 依存: `conductor-core`(型、`parse_rfc3339`、`format_rfc3339`)、`runtime::bounded_read`、`runtime::temp_file`、`toml`
- 場所: `crates/runtime/src/state/`

## runtime::platform

- 責務: プロセスグループ、シグナルの送信(`unsafe` の `libc::kill` 1 か所)、シグナル終了の読み取り、ディレクトリの同期、新しいファイルの 0600。
- 依存: `libc`(UNIX)
- 場所: `crates/runtime/src/platform/`

## conductor-converter

- 責務: YAML と TOML の相互変換(アンカー・エイリアス・マージキーの展開と上限)。定義のキーは知らない。
- 依存: `toml`、`thiserror`、`saphyr-parser`。`#![forbid(unsafe_code)]`。
- 場所: `crates/converter/`

## conductor-cli

- 責務: 実行ファイル `conductor`。引数、流れの組み立て、シグナル、表示(英日)、終了コード。
- 依存: `conductor-core`、`conductor-runtime`、`conductor-converter`、`lexopt`、`signal-hook`(UNIX)、`ctrlc`(Windows)。`#![forbid(unsafe_code)]`。
- 場所: `crates/cli/`

## cli::types

- 責務: `CommandKind`(8 種、`ALL`)、`CliCommand`(`follow`、`interval` を含む)、`GraphFormat`(`Text`・`Mermaid`・`Dot`)、`InitTarget`、`UsageErrorKind`(8 種)、`Language`、`Level`、`--interval` の既定と範囲の定数。
- 依存: `conductor-core`、`conductor-converter`(`TargetFormat`)
- 場所: `crates/cli/src/types.rs`

## cli::args

- 責務: 引数の解釈(`lexopt`、オプションの表 `OPTIONS`、コマンドごとの文法)。
- 依存: `cli::types`、`lexopt`
- 場所: `crates/cli/src/args.rs`

## cli::ctx

- 責務: `Ctx`、`Checked`、`Session`、実行の前の検査(定義 → command → ジョブID → run ID → `.env` → `unset_env` の重なり → ステート)、シグナルの準備、ステートの書き込み・初期化とその失敗の表示。
- 依存: `cli::deps`、`cli::report`、`cli::run_id`、`conductor-core`、`conductor-runtime`
- 場所: `crates/cli/src/ctx.rs`

## cli::step

- 責務: 1 つのジョブの手順(`record_skip`、`run_step`、`run_job_with_retry`、`remove_if_requested`)。試行ごとに時刻を取り、`start_attempt` → `Fast` の書き込み → 起動(ログの指定)→ `end_attempt`/`cancel_attempt`。リトライの待ちの間は試行を開いたまま(`pending_end`)。`log_dir` の解決とログの警告。
- 依存: `cli::ctx`、`cli::report`、`cli::exit`、`conductor-core`、`conductor-runtime`
- 場所: `crates/cli/src/step.rs`

## cli::flows

- 責務: `run`(個別実行)、`pipeline`(一括実行)、`dry_run`(計画)、`aux_commands`(`watch` と `follow_watch`、`reset`、`validate`、`convert`、`graph`、`init`)の流れ。
- 依存: `cli::ctx`、`cli::step`、`cli::report`、`cli::platform`、`conductor-core`、`conductor-runtime`
- 場所: `crates/cli/src/run.rs`、`pipeline.rs`、`dry_run.rs`、`aux_commands.rs`

## cli::run_id

- 責務: run ID の解決(`--run-id`、環境変数、`watch`・`reset` のゆるい読み取り)。
- 依存: `cli::ctx`、`cli::deps`、`conductor-core`
- 場所: `crates/cli/src/run_id.rs`

## cli::report

- 責務: `DisplayEvent`(37 種)、`Reporter`(言語・色の遅延判定)、`FramePrefix`(`Nothing`・`ClearScreen`・`BlankLine`)、英日の文言(`messages.rs`。使い方の全文、ログの 3 つの文、所要時間の見出しを含む)、整形(`watch.rs`(4 列)、`graph_text.rs`、`graph_mermaid.rs`、`graph_dot.rs`、`plan_text.rs`、`init_template.rs`)。
- 依存: `cli::types`、`conductor-core`、`conductor-runtime`(エラーの型)、`conductor-converter`(エラーの型)
- 場所: `crates/cli/src/report/`

## cli::deps

- 責務: 差し替え口の trait(`Runtime` 14 メソッド、`Sink`、`Env`、`Signals`)と本番の実装。
- 依存: `cli::platform`、`conductor-core`、`conductor-runtime`、`conductor-converter`
- 場所: `crates/cli/src/deps.rs`

## cli::platform

- 責務: シグナルの監視(UNIX は `signal-hook`、Windows は `ctrlc`)、`color_supported`、`clear_screen_supported`(どちらも `cfg!(unix)`)。
- 依存: `signal-hook`、`ctrlc`、`conductor-runtime`(`InterruptSignal`)
- 場所: `crates/cli/src/platform/`

## cli::exit

- 責務: 終了コードの定数と、0〜255 への合わせ。
- 依存: `conductor-core`
- 場所: `crates/cli/src/exit.rs`

## cli::panic_hook

- 責務: パニックのとき固定の英語の文を出し、70 で終わる。
- 依存: 標準ライブラリ
- 場所: `crates/cli/src/panic_hook.rs`

## test-helpers

- 責務: テスト専用の補助バイナリ(`conductor-test-helper`、`conductor-cli-test-helper`)。引数・環境変数・終了コード・シグナルの受信・出力を決めた形で返す。配布せず、カバレッジから除く。
- 依存: 標準ライブラリ
- 場所: `crates/runtime/src/bin/conductor-test-helper.rs`、`crates/cli/src/bin/conductor-cli-test-helper.rs`

## demo

- 責務: 主な動きを場面 0〜15 で見せるスクリプト(UNIX の `run-demo.sh`、Windows の `run-demo.cmd`)と、その定義(`conductor.toml`、`retry.toml`、`env.toml`、`log.toml`)・`.env`・ジョブのスクリプト。`DEMO_PAUSE=1` で場面ごとに Enter を待つ。場面 4 で `watch --follow` を呼び、その前に待つ(v0.10.1。`run-demo.sh:94-98`)。場面 15 はログのファイル。
- 依存: ビルドした `conductor`(無ければ `cargo build --release`)
- 場所: `demo/`

## spec-docs

- 責務: 再構築用の仕様書。`rust/`(Rust 版の構成と挙動)、`language-agnostic/`(言語に依らない挙動と受け入れ基準)、5 つの言語版の案内。各 13 文書。コードからもテスト・CI からも読まれない(文書だけ)。
- 依存: なし(実装を書き起こしたもの。実装が正)
- 場所: `docs/spec/`

## ci-workflows

- 責務: マージの関門(`ci.yml`)、リリース(`release.yml`)、脆弱性の検査(`audit.yml`)。
- 依存: GitHub Actions(`actions/checkout@v6`、`dtolnay/rust-toolchain`、`taiki-e/install-action@v2`、`actions/upload-artifact@v7`、`actions/download-artifact@v8`)
- 場所: `.github/workflows/`
