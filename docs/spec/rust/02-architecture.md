# 02. アーキテクチャ(クレート・公開 API・実行の流れ・差し替え点・モジュール)

対象: `conductor` v0.10.0 の機能(試行の時刻、`[pipeline.retry]`、`env_file` の表、`unset_env`、`log_dir`、DOT、`watch --follow`)と v0.10.1 の直しを含む現在の実装(Rust 実装)。根拠は `crates/*/src/{lib,types,deps,...}.rs`。この文書は「どのクレートが何を受け持ち、何を公開し、どう組み合わさるか」を定める。各挙動の細部は 03〜10 章にある。

## 1. 4 クレートの責務と依存方向

```text
            ┌──────────────────┐
            │  conductor-cli   │  実行ファイル `conductor`(lib `conductor_cli` + bin)
            └───┬─────┬─────┬──┘
                │     │     │
      ┌─────────┘     │     └───────────┐
      ▼               ▼                 ▼
┌────────────┐  ┌──────────────┐  ┌───────────────────┐
│ conductor- │  │ conductor-   │  │ conductor-        │
│ runtime    │  │ core         │  │ converter         │
└─────┬──────┘  └──────────────┘  └───────────────────┘
      │ 依存            ▲                (core にも runtime にも依存しない)
      └─────────────────┘
```

| クレート | 種別 | 内部の依存 | 責務 | 入出力 |
|----------|------|------------|------|--------|
| `conductor-core` | lib | なし | 定義ファイルの読み込みと検証(`load_definition`、`parse_definition`)。共有する型。run ID の検査。フローの判断(分類、起動前の判断、次の動きと完了、リトライの判断、中断時の再開位置)。エラー型 `DefinitionError`。v0.9.0 以降: `.env` の解析と層の統合(`envfile`)、図の構造と配置(`graph`)、実行計画の模擬(`plan`)。v0.10.0 以降: 時刻の整形・読み取り・差(`time`)、試行の記録(`flow/attempt.rs`)、`watch --follow` の終わりの判定(`flow/activity.rs`)、`unset_env` と `.env` の重なりの検査(`check_unset_conflicts`)。すべて純粋 | ファイルに触れるのは定義の取り込み(`definition/intake.rs`)だけ。判断の関数は純粋(ファイル・プロセス・環境変数に触れず、失敗せず、必ず結果を返す)。`envfile`・`graph`・`plan` も、文字列と値だけを扱う |
| `conductor-runtime` | lib(+ テスト補助 bin) | core | `command` の分割と検査。ジョブの起動・待ち・シグナルの転送(`run_job`)。リトライの待機(`wait_for`)。ステートの読み込み・書き込み・初期化・照合・記録(`state`)。`reset --all`。v0.9.0 以降: `.env` ファイルの読み取り(`load_env_files`)、ファイルの排他的な作成(`create_file`。`init` が使う)。v0.10.0 以降: 現在の時刻(`now`。`clock`)、ログのファイル(`log`。`open_log_file`、`safe_log_stem`) | プロセス、ステートファイル、`.env`、ログのファイル、時計、OS(`platform`) |
| `conductor-converter` | lib | なし | 定義ファイルの TOML と YAML の相互変換(`convert`、`convert_text`)。定義の検証はしない | 入力ファイルの読み取りだけ(`intake.rs`) |
| `conductor-cli` | lib `conductor_cli` + bin `conductor`(+ テスト補助 bin) | core、runtime、converter | 引数の解釈、流れの組み立て(`run`・`pipeline`・`watch`・`reset`・`validate`・`convert`・`graph`・`init`、`--dry-run`)、図と計画とひな形の文字への整形、シグナルの監視、表示(英語・日本語)、終了コードの決定、パニックのフック | 標準入出力、環境変数、シグナル。ファイルとプロセスの操作は `Runtime` trait を介して runtime・core・converter に任せる |

依存方向の規則:

- `core` と `converter` は、他の内部クレートに依存しない。`runtime` は `core` だけに依存する。`cli` は 3 つすべてに依存する。逆向きの依存はない。
- 共有する型(`PipelineDefinition`、`JobExecution`、`PipelineState`、`JobOutcome`、`NextStep` など)は `core` が定める。`runtime` は、それを作る(`JobExecution`、`PipelineState`)・受け取る側で、型を追加定義しない(`InterruptSignal`、`CommandLine`、`StateLocation` などの実行側の型だけを足す)。
- 判断(core)・入出力(runtime)・流れと表示(cli)を、クレートの境界で分ける。純粋な判断は、ファイルシステムやプロセスを使わずにテストできる。
- OS 固有のコード(`cfg(unix)`/`cfg(windows)`)は、`runtime` と `cli` のそれぞれの `src/platform/` の中だけに置く(ソース検索のテストが強制)。

## 2. 公開 API

以下は、クレート直下から公開される主要な型・トレイト・関数。シグネチャは実装のとおり。

### 2.1 `conductor-core`

再公開(`lib.rs`): `consistency::{ConsistencyResult, MismatchKind, check_consistency}`、`definition::{load_definition, parse_definition}`、`envfile::{EffectiveEntry, EffectiveEnv, EnvContext, EnvEntry, EnvFileContent, EnvFileError, EnvFileErrorKind, EnvWarning, EnvWarningKind, LoadedEnvFiles, check_unset_conflicts, env_file_refs, merge_env, parse_env_file, summarize_errors}`、`error::{DefinitionError, DefinitionErrorKind, MAX_KEY_DISPLAY_CHARS, escape_display}`、`flow::{PipelineActivity, RetryDecision, apply_interrupt, apply_outcome, apply_skip, cancel_attempt, classify_and_decide, decide_from_state, end_attempt, initial_state, pipeline_activity, resume_point_on_interrupt, retry_decision, start_attempt}`、`plan::{Assumption, EnvNameEntry, ExecutionPlan, NextStepNote, PlanDisposition, PlanEnd, PlanEntry, PlanInput, PlanMode, PlanRejection, ResumeInfo, SimulatedResult, plan_pipeline, plan_run}`、`run_id::{DEFAULT_RUN_ID_ENV, RunId, RunIdError, peek_run_id_env}`、`time::{JobDuration, Timestamp, duration_secs, format_compact, format_duration, format_rfc3339, job_duration, parse_rfc3339}`、`types::{Attempt, Backoff, Classification, Command, Decision, EnvFileRef, EnvVar, ExecutionResult, JobDefinition, JobExecution, JobOutcome, JobState, JobStatus, NextStep, PipelineDefinition, PipelineState, RejectReason, ResumePoint, RetryPolicy, RunMode, SkipCause, SkipReason, Transition, TransitionOn, TransitionTarget}`(`graph` は `conductor_core::graph::…` のパスで使う)。公開モジュール: `consistency`、`definition`(公開の下位モジュールは `intake` だけ。`depth`・`parse`・`version`・`walk` は crate の中)、`envfile`、`error`、`flow`、`graph`、`plan`、`run_id`、`time`(v0.10.0 以降)、`types`。`flow`・`plan`・`envfile`・`graph` の下位モジュールは crate の中で、関数と型は各モジュールの再公開の名前(`flow::classify_and_decide`、`plan::plan_run`、`envfile::merge_env`、`graph::build_graph` など)で出る。終了コードの定数(`EXIT_START_FAILURE = 127`、`EXIT_SIGNAL_BASE = 128`、`EXIT_UNLISTED_ZERO = 1`)は `flow/classify.rs` の中だけの定数、`ASSUME_EXIT_CODE_MIN`・`ASSUME_EXIT_CODE_MAX`、`RESERVED_ENV_PREFIX`、`MAX_REPORTED_ERRORS` は crate の中の定数(`MAX_ENV_FILE_BYTES`・`MAX_ENV_ENTRIES` は公開)。

版の履歴(古い順):

- v0.9.0 以降は、`envfile::{…}` と `plan::{…}` も再公開する。
- v0.9.6 までは `error::*`・`types::*` で、`quote_key`・`Locate`・`TransitionResolution`・`resolve_transition` も出ていた(外から使われないので crate の中に狭めた)。
- v0.9.7 までは `flow` の下位モジュール、`plan::simulate`、`envfile::{merge, parse}`、`graph::{edges, layout, overlay}` と、上の crate の中の定数も公開だった。
- v0.10.0 以降は、`time` モジュールと `time::{…}`、`types::{Attempt, EnvFileRef}`、`flow::{PipelineActivity, pipeline_activity, start_attempt, end_attempt, cancel_attempt}`、`envfile::check_unset_conflicts` も再公開する。
- v0.10.1 で、計画の模擬のための内部の入口(`flow` の `decide_at`、`classify_and_decide_at`、`apply_skip_in`・`apply_outcome_in` の目印の引数、`record_at`、`position_of`、`SimulationMarks`)を足した。どれも `pub(crate)` で、公開の口は変わらない(05 の 16.2)。

#### 関数

| 関数 | シグネチャ | 意味 |
|------|-----------|------|
| `load_definition` | `(path: Option<&Path>, expected_pipeline_name: Option<&str>) -> Result<PipelineDefinition, Vec<DefinitionError>>` | パス(なければ `conductor.toml`)を読み、Intake → Parse → Depth → VersionCheck → Walk で検証する。03 章 |
| `parse_definition` | `(text: &str, file_label: &str, expected_pipeline_name: Option<&str>) -> Result<PipelineDefinition, Vec<DefinitionError>>` | 文字列に対して同じ検証(ファイルに触れない) |
| `peek_run_id_env` | `(text: &str) -> Option<String>` | 定義の `[pipeline].run_id_env` だけをゆるく取り出す(`watch`・`reset` 用)。03 の 12 章 |
| `is_valid_env_name`(crate の中) | `(name: &str) -> bool` | 環境変数名の形(`A-Za-z_` で始まり `A-Za-z0-9_` が続く) |
| `RunId::parse` | `(value: &str) -> Result<RunId, RunIdError>` | run ID の規則の検査(空・64 バイト超・不正な文字・先頭 `.`)。`as_str()` で文字列を返す |
| `classify`(`flow` の中だけ。`classify_and_decide` が呼ぶ) | `(job: &JobDefinition, execution: &JobExecution) -> JobOutcome` | 実行結果の分類と、返す終了コード。05 の 2.3 |
| `classify_and_decide` | `(job, execution, job_states: &[JobState], definition: &PipelineDefinition, run_mode: RunMode) -> (JobOutcome, NextStep)` | 分類して、次の動きと完了を決める。05 の 3 章 |
| `decide_before_run`(`flow/decide.rs` の中だけ。公開は `decide_from_state`) | `(job_id: &str, definition: &PipelineDefinition, job_states: &[JobState], resume_job_id: Option<&str>, end_reached: bool) -> Decision` | 起動前の判断(Run / Skip / Reject)。05 の 4 章 |
| `decide_from_state` | `(job_id: &str, definition: &PipelineDefinition, state: Option<&PipelineState>) -> Decision` | ステートから起動前の判断を決める。`None` は、記録が空、再開位置なし、`end_reached` は偽と同じ。`run`・`pipeline` と計画が使う。05 の 4.1 |
| `resume_point_on_interrupt` | `(definition: &PipelineDefinition, job_states: &[JobState]) -> ResumePoint` | ジョブのあいだの中断の再開位置。05 の 9 章 |
| `retry_decision` | `(job: &JobDefinition, execution: &JobExecution, retries_done: u32, last_wait_secs: u64) -> RetryDecision` | リトライするか(`Stop` または `Retry { wait_secs, last_exit_code, max }`)。05 の 7 章 |
| `resolve_transition`(v0.9.0 以降。v0.9.7 以降は `flow::next_step` の crate の中の関数で、公開しない) | `(job: &JobDefinition, on: TransitionOn, index: usize, position: impl Fn(&str) -> Option<usize>) -> TransitionResolution`(`position` はジョブ ID から記述順の位置を引く関数。v0.9.6 までは `(job, on, definition, index)`) | 結果 `on` の遷移の行き先の解決(`End` / `Job(位置)` / `Default`)。`classify_and_decide` と、図の辺の取り出し(`graph::flow_edges`)が共有する。05 の 3.2 |
| `parse_env_file`(v0.9.0 以降) | `(text: &str, file: &str) -> Result<EnvFileContent, Vec<EnvFileError>>` | `.env` の文字列の解析(純粋)。03 の 6.7 |
| `merge_env`(v0.9.0 以降) | `(def: &PipelineDefinition, job: &JobDefinition, files: &LoadedEnvFiles, ctx: &EnvContext<'_>) -> EffectiveEnv` | 層を統合して、ジョブに足す環境と、(v0.10.0 以降)引き継いだ環境から消す名前(`removals`)を作る(純粋)。05 の 14 章 |
| `env_file_refs`(v0.9.0 以降) | `(def: &PipelineDefinition, job_id: Option<&str>) -> Vec<EnvFileRef>`(v0.9.x までは `Vec<String>`) | 読む対象の `env_file` の記述(同じパスの文字列は 1 つにまとめ、`required` は論理和。書いた順)。`None` は全ジョブ分 |
| `check_unset_conflicts`(v0.10.0 以降) | `(def: &PipelineDefinition, job_id: Option<&str>, files: &LoadedEnvFiles) -> Vec<EnvFileError>` | 読めた `.env` の名前と、`unset_env` の名前の重なり(`unsetConflict`。純粋)。03 の 6.7 |
| `start_attempt` / `end_attempt` / `cancel_attempt`(v0.10.0 以降) | `(state: &PipelineState, job_id: &str, at: Timestamp) -> PipelineState` / 同 / `(state, job_id) -> PipelineState` | 試行を足す・閉じる・取り除く(純粋。`flow/attempt.rs`)。05 の 18.2 |
| `pipeline_activity`(v0.10.0 以降) | `(state: &PipelineState) -> PipelineActivity` | `watch --follow` の終わりの判定(`Completed`・`Running`・`Stopped`・`Between`。純粋。`flow/activity.rs`)。05 の 20 章 |
| `format_rfc3339` / `format_compact` / `parse_rfc3339` / `duration_secs` / `job_duration` / `format_duration`(v0.10.0 以降) | `(Timestamp) -> String` / 同 / `(&str) -> Option<Timestamp>` / `(start, end) -> u64` / `(job: &JobState, now: Timestamp) -> Option<JobDuration>` / `(secs: u64) -> String` | 時刻の整形・読み取り・差と、所要時間(純粋。`time.rs`)。05 の 18 章 |
| `is_reserved_env_name`(v0.9.0 以降。crate の中) | `(name: &str) -> bool` | `CONDUCTOR_` で始まるか(ASCII の大文字小文字を区別しない) |
| `summarize_errors`(v0.9.0 以降) | `(errors: &[EnvFileError]) -> (&[EnvFileError], usize)` | 表示する先頭の最大 20 件と、残りの件数 |
| `graph::build_graph` / `layout_graph` / `overlay_from_state` / `pending_overlay`(v0.9.0 以降) | 05 の 15 章 | 図の辺・構造・配置・状態の重ね合わせ(純粋)。辺の取り出し `graph::flow_edges` は crate の中、状態の対応 `node_state_of` は `graph/overlay.rs` の中だけ(公開しない。辺は `build_graph` の結果の `edges` で得る) |
| `plan_pipeline` / `plan_run`(v0.9.0 以降) | `(&PlanInput<'_>) -> Result<ExecutionPlan, PlanRejection>` / `(job_id: &str, &PlanInput<'_>) -> …` | `--dry-run` の計画の生成(純粋)。05 の 16 章 |
| `initial_state` / `apply_outcome` / `apply_skip` / `apply_interrupt` | `(definition: &PipelineDefinition, definition_path: &Path, run_id: Option<&RunId>) -> PipelineState` / `(state, job_id: &str, execution: &JobExecution, outcome: &JobOutcome, next: &NextStep) -> PipelineState` / `(state, job_id, reason: SkipCause, definition) -> PipelineState` / `(state, resume_point: &ResumePoint) -> PipelineState` | ステートの更新(純粋。`flow/record.rs`)。全ジョブ `pending` の初期ステート、結果・スキップ・中断の記録(メモリ上)。本番の流れと `--dry-run` の模擬が共に使う。05 の 3.3〜3.5 |
| `check_consistency` | `(state: &PipelineState, definition: &PipelineDefinition) -> ConsistencyResult` | ステートと定義の照合(純粋。`consistency.rs`。型 `ConsistencyResult`(`Ok` / `Mismatch(PipelineName \| JobIds \| JobOrder)`)、`MismatchKind` も同じ場所)。06 の 6 章 |

#### 主要な型(`types`)

| 型 | 内容 |
|----|------|
| `PipelineDefinition` | `name`、`description: Option<String>`、`run_id_env: Option<String>`、`remove_on_complete: bool`、`format_version: u32`、`job: Vec<JobDefinition>`、`env: Vec<EnvVar>`、`env_file: Vec<EnvFileRef>`(`env`・`env_file` は v0.9.0 以降。`env_file` は v0.9.x までは `Vec<String>`)、`log_dir: Option<String>`、`unset_env: Vec<String>`(後ろの 2 つは v0.10.0 以降)。`job_index(&str) -> Option<usize>`。`[pipeline.retry]` の既定は型に持たず、各ジョブの `retry` に複製して入れる(03 の 6.8) |
| `EnvVar`(v0.9.0 以降) | `{ name: String, value: String }`。`Debug` は値を `"***"` と表示する |
| `EnvFileRef`(v0.10.0 以降) | `{ path: String, required: bool }`。`env_file` の 1 項目(文字列の項目は `required = true`)。03 の 6.6 |
| `JobDefinition` | `id`、`command: Command`、`warning_exit_codes: Vec<i32>`、`success_exit_codes: Option<Vec<i32>>`、`transitions: Vec<Transition>`、`retry: Option<RetryPolicy>`(実効のリトライ。v0.10.0 以降は、書かなければ既定の複製、`retry = false` なら `None`)、`env: Vec<EnvVar>`、`env_file: Vec<EnvFileRef>`(`env`・`env_file` は v0.9.0 以降)、`unset_env: Vec<String>`(v0.10.0 以降)。`effective_success_codes() -> &[i32]`(crate の中。省略時は `DEFAULT_SUCCESS_EXIT_CODES` = `[0]`)、`transition_for(TransitionOn) -> Option<&Transition>` |
| `Command` | `Line(String)` / `Argv(Vec<String>)` |
| `Transition` / `TransitionOn` / `TransitionTarget` | `{ on, next }` / `Success`・`Warning`(`as_str()` で `success`・`warning`)/ `Job(String)`・`End` |
| `RetryPolicy` | `exit_codes: Vec<i32>`、`max: u32`、`interval_secs: u64`、`backoff: Option<Backoff>`(`Backoff { multiplier: u32, max_interval_secs: u64 }`。`Copy`。倍率と上限は両方あるときだけ) |
| `JobExecution` | `result: ExecutionResult`。組み立ての関数は `new(result)`、`exited(exit_code)`(起動して終了コードで終わった結果。`--dry-run` の模擬が使う)、`not_started(signal)`、`start_failed()`。v0.9.6 までは、読まれない `job_id`・`command` も持ち、組み立ての関数は先頭に `job` を受け取った。v0.9.5 までは `started`・`exit_code`・`terminated_by_signal`・`start_failure`・`interrupted`・`signal_number` の項目だった。05 の 1.2 |
| `ExecutionResult` | 起動の結果の 3 つの場合: `NotStarted { signal: i32 }`(起動しなかった中断)、`StartFailed`(起動の失敗)、`Exited { exit_code: Option<i32>, terminated_by_signal: Option<i32>, interrupted_by: Option<i32> }`(終了を観測した)。05 の 1.2 |
| `JobStatus` / `SkipReason` | `Pending`・`Success`・`Warning`・`Error`・`Skipped` / `ByTransition`・`ByEnd`。どちらも `as_str()` と `parse(&str) -> Option<Self>` で、ステートファイルの語(`pending` など、`byTransition`・`byEnd`)と行き来する |
| `JobState` | `job_id`、`status`、`exit_code: Option<i32>`、`skip_reason: Option<SkipReason>`、`attempts: Vec<Attempt>`(v0.10.0 以降。古い順)。`JobState::pending(id)`(試行なし) |
| `Attempt`(v0.10.0 以降) | `{ started_at: Timestamp, ended_at: Option<Timestamp> }`。ステートの `[[job.attempt]]`(06 の 2.3a) |
| `Timestamp`(v0.10.0 以降。`time`) | UTC の UNIX 時刻の秒(`u64`)。`EPOCH`(1970-01-01T00:00:00Z)〜`MAX`(9999-12-31T23:59:59Z)。`from_unix_seconds(secs) -> Option<Timestamp>`、`unix_seconds()`。`JobDuration { Finished(u64), Running(u64) }`(`secs()`) |
| `PipelineActivity`(v0.10.0 以降。`flow`) | `Completed`、`Running`、`Stopped`、`Between`(05 の 20 章) |
| `PipelineState` | `pipeline_name`、`definition_path`、`job_states: Vec<JobState>`、`resume_job_id: Option<String>`、`end_reached`、`completed`、`run_id: Option<String>`。ファイルの `definition_key` と `job_order` は、書き出しと読み込みの検査だけに使い、型には持たない(`job_order` は `job_states` のジョブIDの並び)。06 章 |
| `Decision` / `SkipCause` / `RejectReason` | `Run` / `Skip(SkipCause)` / `Reject(RejectReason)`。スキップの理由 `SkipCause` は `SkipTarget`・`AfterEnd`・`AlreadyExecuted`、拒否の理由 `RejectReason` は `OutOfOrder` だけ(ステートに書く `SkipReason { ByTransition, ByEnd }` とは別の型) |
| `TransitionTarget` / `TransitionOn` | `TransitionTarget::END_LABEL`(`@end`)、`label()`(ジョブID か `@end`)。`TransitionOn::parse`(`as_str` の逆)。`From<Classification> for JobStatus` |
| `Classification` / `JobOutcome` | `Success`・`Warning`・`Error`(`as_str()`、遷移を引く `transition_on() -> Option<TransitionOn>`。エラーは `None`)/ `{ classification: Option<Classification>, reported_exit_code: i32, conductor_error: bool }`(v0.9.6 までは読まれない `job_id` も持った) |
| `NextStep` | `{ next_job_id, skipped_job_ids, skip_reason_kind, end_reached, completed, resume_job_id }`(v0.9.6 までは読まれない `job_id` も持った) |
| `RunMode` | `Individual` / `Pipeline` |
| `ResumePoint` | `{ resume_job_id, end_reached, completed }` |
| `RetryDecision` | `Stop` / `Retry { wait_secs: u64, last_exit_code: i32, max: u32 }`(リトライの対象になった最後の終了コードと、最大回数を持つ) |

#### エラー(`error`)

| 型 | 内容 |
|----|------|
| `DefinitionErrorKind` | 34 種(v0.9.0 で `ReservedEnvName`・`EnvNameCaseConflict`・`InvalidEnvValue` を、v0.10.0 で `UnsetEnvConflict` を追加)。仕様の識別子(`fileMissing` など)は、変種の名前の先頭を小文字にしたもの。03 の 9.3、10 の 7.1 |
| `DefinitionError` | `{ file: String, line: Option<usize>, kind, reason: String }`。`Display` は `<file>[:<line>]: <種類の文>: <reason>` |
| `RunIdError` | 種類を持たない(`struct RunIdError;`)。4 つの条件(空・64 バイト超・不正な文字・先頭 `.`)のどれかで返す |
| `EnvFileErrorKind` / `EnvFileError`(`envfile`。v0.9.0 以降) | 14 種(v0.10.0 で `UnsetConflict` を追加。`reason()`。識別子は変種の名前の先頭を小文字にしたもの)/ `{ file: String, line: Option<usize>, kind }`。理由は `kind.reason()`(v0.9.6 までは同じ文の写しをフィールド `reason: String` に持った)。`Display` は `<file>[:<line>]: <種類の文>: <reason>`(書式は `DefinitionError` と共有する `error::render_error`)。03 の 6.7、10 の 7.1 |
| 補助 | `escape_display(input, limit)`(制御文字・書式文字のエスケープ。上限なしも選べる。表示のエスケープの本体)、`quote_key`、`Locate`(位置→行番号。`line_of(offset)`)、`MAX_KEY_DISPLAY_CHARS = 64` |

### 2.2 `conductor-runtime`

再公開(`lib.rs`): `error::{CommandError, CommandErrorKind, StateError, StateErrorKind}`、`run_job`、`check_command`、`read_state`、`read_state_raw`、`ResetAllReport`、`reset_all_states`、`reset_state`、`write_state`、`types::{CommandLine, Durability, InterruptSignal, StateLocation}`、`WaitOutcome`、`wait_for`、`load_env_files`、`CreateError`、`CreateOutcome`、`create_file`、`now`(`clock`)、`log::{LogOutcome, LogTarget, open_log_file, safe_log_stem}`。モジュールはすべて非公開(`bounded_read`、`clock`、`envfile`、`error`、`file_create`、`log`、`platform`、`runner`、`split`、`state`、`temp_file`、`types`、`wait`。`#![deny(unsafe_code)]`)。`StateErrorKind::label()`(種類の文の日本語)は公開(cli の表示が使う)。ステートの照合(`check_consistency`)と更新(`initial_state`、`apply_*`)は `conductor_core` の関数で、使う側は `conductor_core` から使う(2.1 の表)。

版の履歴(古い順):

- v0.9.0 以降は、`load_env_files`、`CreateError`、`CreateOutcome`、`create_file` も再公開する。
- v0.9.6 までは `error`・`types` も公開で、`error::*`・`types::*` で再公開した。
- v0.9.7 以降は、`StateErrorKind::label()` が公開。
- v0.10.0 以降は、`now`、`LogOutcome`、`LogTarget`、`open_log_file`、`safe_log_stem` も再公開する。

| 関数 | シグネチャ | 意味 |
|------|-----------|------|
| `run_job` | `(line: &CommandLine, env: &[(&str, &str)], unset: &[&str], signal: &InterruptSignal, log: Option<&LogTarget>) -> (JobExecution, LogOutcome)`(v0.10.0 以降。v0.9.7〜v0.9.9 は `(line, env, signal) -> JobExecution`、v0.9.6 までは `run_job(job, line, env, signal)`。`job` は使われなかった) | 分割済みのコマンド `line`(`check_command` の結果)で、シェルを介さず直接起動し、終了を待つ。中で `command` を分け直さない(`line.program` が空・空白だけなら起動せず `StartFailed`)。(v0.10.0 以降)`unset` の名前を `env_remove` で引き継いだ環境から除いてから、`env` を、並びの順に、引き継いだ環境へ足す(`env` は v0.9.0 以降。v0.8.0 以前は `env` 引数なし)。(v0.10.0 以降)`log` があれば、ログのファイルを作り、出力をパイプで読んで端末とファイルへ書く。作れなければ起動せず `StartFailed`。パニックしない。シグナルを子へ 1 回転送。v0.9.5 までは `run_job(job, env, signal)` で、中で分け直した。07 の 4 章 |
| `load_env_files`(v0.9.0 以降) | `(definition_path: &Path, refs: &[EnvFileRef]) -> Result<LoadedEnvFiles, Vec<EnvFileError>>`(v0.9.x までは `refs: &[String]`) | `env_file` の `.env` を、すべて読む(パスの解決、1 MiB の上限、UTF-8、解析。v0.10.0 以降は、`required = false` で無いファイルを読み飛ばす)。誤りは全ファイル分を集めて返す。03 の 6.7 |
| `now`(v0.10.0 以降。`clock`) | `() -> Timestamp` | 現在の時刻(UTC の秒)。1970 年より前なら `EPOCH`、9999 年を超えれば `MAX` に寄せる |
| `open_log_file`(v0.10.0 以降。`log`) | `(target: &LogTarget) -> Result<(File, PathBuf), PathBuf>` | 試行のログのファイルを作って開く(親を含めてディレクトリを作る、「無いときだけ作る」、`-2`〜`-1000` の番号、UNIX 系は 0600)。作れなければ、表示に使うパスを `Err` で返す。07 の 4.5 |
| `safe_log_stem`(同) | `(job_id: &str) -> String` | ジョブ ID を、ファイル名に使える安全な名前にする。07 の 4.5 |
| `create_file`(v0.9.0 以降) | `(path: &Path, bytes: &[u8], force: bool) -> Result<CreateOutcome, CreateError>` | 排他的な作成(`force` なら一時ファイルから置き換え)。`CreateOutcome::{Created, Replaced}`、`CreateError::{AlreadyExists, ParentMissing, IsDirectory, PermissionDenied, OtherIo}`。04 の 7.8 |
| `check_command` | `(command: &Command) -> Result<CommandLine, CommandError>` | 起動せずに `command` を検査(分割)する。07 の 3 章 |
| `wait_for` | `(secs: u64, signal: &InterruptSignal) -> WaitOutcome` | リトライ前の待機。合図を 50 ms 以下の間隔で確かめる。`WaitOutcome::{Elapsed, Interrupted(i32)}` |
| `read_state` | `(definition_path: &Path, run_id: Option<&RunId>) -> Result<Option<PipelineState>, StateError>` | ステートを読む。ファイルなし・長さ 0・**完了のもの**は `None`(空) |
| `read_state_raw` | 同上 | 完了のものも返す(`watch` 用)。ファイルなし・長さ 0 のときだけ `None` |
| `write_state` | `(definition_path, run_id: Option<&RunId>, state: &PipelineState, durability: Durability) -> Result<(), StateError>` | 一時ファイル → (Durable なら同期)→ rename。06 の 4 章 |
| `reset_state` | `(definition_path: &Path, run_id: Option<&RunId>) -> Result<(), StateError>` | ステートファイルを消す(なくても成功) |
| `reset_all_states` | `(definition_path: &Path) -> Result<ResetAllReport, StateError>` | 定義の全ステートを消す。`ResetAllReport { kept: Vec<String> }` |

型(`types`): `CommandLine { program: String, args: Vec<String> }`、`InterruptSignal`(`AtomicI32` 1 つ。`new`、`raise(n)`、`signal_number`。最初の番号が残る)、`StateLocation::of(definition_path, run_id)`(`path()`)、`Durability { Durable, Fast }`。(v0.10.0 以降。`log`)`LogTarget { dir: PathBuf, job_id: String, started_at: Timestamp, attempt: u32 }`、`LogOutcome { log_not_created: Option<PathBuf>, write_failed: Option<PathBuf>, output_cut_short: Option<PathBuf> }`(`Default` はすべて `None`)。エラー: `CommandError`/`CommandErrorKind`、`StateError`/`StateErrorKind`(10 の 7.2)。

### 2.3 `conductor-converter`

| 項目 | 内容 |
|------|------|
| `convert(request: &ConversionRequest) -> Result<ConvertedText, ConversionError>` | 入力ファイルを読み、変換する |
| `convert_text(text: &str, target: TargetFormat, file_label: &str) -> Result<ConvertedText, ConversionError>` | 文字列を変換する(ファイルに触れない) |
| `ConversionRequest { input_path: PathBuf, target_format: TargetFormat }` | 要求 |
| `TargetFormat { Yaml, Toml }` | 変換**後**の形式。入力の形式は、その反対とみなす |
| `ConvertedText { text: String }` | 結果 |
| `ConversionError { file, line, key_path, kind, reason }`(`reason: &'static str`。固定の文。v0.9.7 までは `String`)/ `ConversionErrorKind`(6 種) | 10 の 7.3 |

内部の途中の表現 `Value`: `Str` / `Int(i64)` / `Float(f64)` / `Bool` / `Array` / `Map(Vec<(String, Value)>)`(`null`・日時などは持たない)。等価の比べ方(09 の 6 節)は、テストのときだけの `PartialEq`(`#[cfg(test)]`)が持つ。本番の処理は木を比べない。

### 2.4 `conductor-cli`

再公開(`lib.rs`): `Deps`、`Env`、`Runtime`、`SignalError`、`Signals`、`Sink`、`StdSink`、`SystemEnv`、`SystemRuntime`、`SystemSignals`、`install_panic_hook`。公開の関数は `run_cli`、`run_cli_with`。公開するのは、`main` が使うもの(`run_cli`、`StdSink`、`SystemEnv`、`install_panic_hook`)と、依存の注入の継ぎ目(4 章)と、継ぎ目に出る型 `SignalError` だけで、公開モジュールは無い(`args`、`exit`、`report`、`run_id`、`types` などは、すべて crate の中)。

版の履歴(古い順):

- v0.9.6 までは `platform`・`report::messages`・`report::watch` も公開で、`status_word`・`state_symbol`・`glyph_char`・`MAX_DISPLAY_CHARS`・`DEFAULT_INIT_PATH`・`DEFAULT_INIT_PIPELINE_NAME` も外から見えた(外から使われないので crate の中に狭めた)。

| 関数 | シグネチャ | 意味 |
|------|-----------|------|
| `run_cli` | `<I: IntoIterator<Item = OsString>>(args: I, env: &dyn Env, sink: &mut dyn Sink) -> i32` | 本番の `Runtime`・`Signals` で `run_cli_with` を呼ぶ |
| `run_cli_with` | `<I>(args: I, deps: &mut Deps<'_>) -> i32` | 引数の解釈 → コマンドごとの処理 → 終了コード。`-h`/`-V` と使い方の誤りはここで処理。シグナルの監視の登録はプロセスで 1 回だけ。本番の `SystemSignals` を渡して、`run`・`pipeline` をプロセスで 2 回実行すると、2 回目の登録は契約の外(Windows は登録に失敗して 70 で終わる。UNIX 系は監視が 2 つになる)。`run_cli` も同じ |
| `parse_args`(`args`。crate の中) | `<I>(args: I) -> Result<ParsedArgs, UsageError>` | 引数の解釈。04 章 |
| `exit_code_for`(`exit`。crate の中) | `(reported: i32, is_error: bool) -> i32` | 0〜255 への整形。05 の 2.2 |
| `signal_exit_code`(同) | `(signal_number: i32) -> i32` | `exit_code_for(128 + n(飽和加算), true)` |
| `resolve_run_id`(`run_id`。crate の中) | `(option: Option<&RunIdArg>, env_name: &str, env: &dyn Env) -> Result<Option<RunId>, UsageError>` | run ID の決定。04 の 5 章 |
| `render_table`(`report::watch`。crate の中) | `(lang: Language, state: &PipelineState, color: bool, now: Timestamp) -> String`(`now` は v0.10.0 以降) | `watch` の表(v0.10.0 以降は所要時間の列を含む 4 列)。08 の 6 章 |
| `render_dot`(`report::graph_dot`。crate の中。v0.10.0 以降) | `(lang: Language, graph: &FlowGraph) -> String` | `graph --format dot` の DOT の文書。05 の 15.8 |
| `follow_watch`(`aux_commands`。crate の中。v0.10.0 以降) | `(ctx, path, run_id, interval: u32, signal) -> FollowEnd` | `watch --follow` の繰り返し。`FollowEnd { Completed, Removed, Stopped, Interrupted, StateError, OutputFailed }`(`exit_code()` で 0 か 65)。05 の 20 章 |
| `escape_name` / `escape_full`(`report`。crate の中) | `(input: &str) -> String` | 入力由来の名前(64 文字で切る)・パス(切らない)のエスケープ。本体は core の `escape_display`。08 の 9 章 |

終了コードの定数(`exit`。crate の中): `EXIT_OK = 0`、`EXIT_DEFINITION = 64`、`EXIT_STATE = 65`、`EXIT_REJECTED = 66`、`EXIT_CONVERSION = 67`、`EXIT_USAGE = 68`、`EXIT_EXISTS = 69`(v0.9.0 以降)、`EXIT_INTERNAL = 70`。

型(`types`): `CommandKind { Run, Pipeline, Watch, Reset, Validate, Convert, Graph, Init }`(`name()`、全種類の `ALL`、名前からの逆引き `from_name(name) -> Option<CommandKind>`。後ろの 2 つは v0.9.0 以降)、`CliCommand { kind, job_id, pipeline_name, convert_input: Option<OsString>, convert_target: Option<TargetFormat>, config_path: Option<OsString>, verbose, run_id: Option<RunIdArg>, all, remove_on_complete, graph: GraphOptions, dry_run: DryRunOptions, force, follow: bool, interval: Option<u32> }`(`graph`・`dry_run`・`force` は v0.9.0 以降、`follow`・`interval` は v0.10.0 以降。`init_options() -> InitOptions`)、`GraphFormat { Text(既定), Mermaid, Dot }`(`Dot` は v0.10.0 以降)、crate の中の定数 `DEFAULT_FOLLOW_INTERVAL_SECS = 2`、`MIN_FOLLOW_INTERVAL_SECS = 1`、`MAX_FOLLOW_INTERVAL_SECS = 3600`(v0.10.0 以降)、`FramePrefix { Nothing, ClearScreen, BlankLine }`(`report`。`watch --follow` の 1 回分の前に置くもの。08 の 6.4)、`GraphOptions { format, with_state }`、`DryRunOptions { dry_run, assumptions: Vec<Assumption> }`、`InitTarget { File(OsString), Stdout }`(`from_path`: `-` だけが `Stdout`)、`InitOptions { target, force, pipeline_name }`(既定は `conductor.toml` の `File`、`force = false`、名前 `my-pipeline`。crate の中の定数 `DEFAULT_INIT_PATH`、`DEFAULT_INIT_PIPELINE_NAME`)、`RunIdArg { Empty, Id(RunId) }`(`--run-id` の検査済みの指定。`parse`。`Empty` は run ID なしに固定)、`ParsedArgs { Help, Version, Command(CliCommand) }`、`UsageErrorKind`(8 種)、`UsageError { kind, subject: Option<String> }`、`Language { En, Ja }`、`Level { Error, Notice, Verbose, Output }`。

## 3. 実行時の全体の流れ

### 3.1 入口

```text
main():
    install_panic_hook()                       # パニック → 英語 1 行 + 終了コード 70
    code = run_cli(args_os().skip(1), &SystemEnv, &mut StdSink)
    std::process::exit(code)                   # exit は 1 回だけ
```

`run_cli_with`:

```text
parsed = parse_args(args)
  Err(e)            → 使い方の誤りを標準エラーに表示、return 68
  Ok(Help)          → 使い方全文を標準出力へ、return 0
  Ok(Version)       → `conductor <版>` を標準出力へ、return 0
  Ok(Command(cmd))  → ctx = Ctx { runtime, signals, sink, env, reporter(verbose) }
                      match cmd.kind: run / pipeline / watch / reset / validate / convert / graph / init
```

### 3.2 `pipeline` 実行のシーケンス

```text
cli::pipeline::run_pipeline
 ├─ dry_run::enter_flow(名前が無ければ 68、--dry-run なら run_dry_run へ)
 ├─ Ctx::prepare_session(check_pipeline を渡す)→ (Session { checked: Checked, signal }, ())
 │    ├─ Ctx::start_signals → Signals::start(signal)  → 失敗 70
 │    └─ 渡された検査 Ctx::check_pipeline(--assume のジョブIDは空。require_known は何もしない)
 │         ├─ Ctx::check_definition                         → CheckedDefinition(分割したコマンドを含む)
 │         │    ├─ Runtime::load_definition(path, name)    → core::load_definition   → 失敗 64
 │         │    └─ Runtime::check_command(各ジョブ)         → runtime::check_command  → 失敗 64(全件を集める)
 │         └─ Ctx::check_run_inputs                         → Checked
 │              ├─ Ctx::definition_run_id → run_id::resolve_run_id(--run-id | 環境変数) → 失敗 68
 │              ├─ Ctx::load_env_files(v0.9.0 以降)        → Runtime::load_env_files → runtime::load_env_files → 失敗 64
 │              │    └─ (v0.10.0 以降)core::check_unset_conflicts(.env の名前と unset_env の重なり) → 失敗 64
 │              └─ Ctx::read_checked_state(StateRead::Live)
 │                   ├─ Runtime::read_state(path, run_id)  → runtime::read_state     → 失敗 65(完了のものは None)
 │                   └─ check_consistency(state, definition) → 食い違い 65
 └─ for (job, command) in definition.job と session.checked.base.commands (記述順):
      ├─ core::decide_from_state(中で decide_before_run)       → Run / Skip / Reject
      │    ├─ Reject → 表示、return 66
      │    ├─ Skip   → step::record_skip(core::apply_skip → 変化があれば Runtime::write_state(Durable)。SkipRecord { state, written })
      │    │           → 表示 → 完了なら finish → step::remove_if_requested(remove_on_complete なら Runtime::reset_state)、return 0
      │    └─ Run    → step::run_step:
      │         ├─ 状態が空なら core::initial_state(ここでは書かない。v0.9.x までは Runtime::write_state(Fast))
      │         ├─ step::run_job_with_retry(job, command, &mut state)    # command は検査で分割済み。分け直さない
      │         │    loop { core::merge_env(試行ごと) → Runtime::now()
      │         │            →(リトライなら core::end_attempt(前の試行を pending_end で閉じる))
      │         │            → core::start_attempt → Runtime::write_state(Fast)(失敗 65。起動しない)
      │         │            → Runtime::run_job(command, 足す環境, 消す名前, signal, log_dir があれば LogTarget)
      │         │            → NotStarted なら core::cancel_attempt、そうでなければ Runtime::now()(終わりの時刻)
      │         │            → ログの警告 → core::retry_decision
      │         │            Stop  → core::end_attempt → (JobExecution, LogOutcome) を返す
      │         │            Retry → 終わりの時刻を pending_end に持つ → 知らせを表示 → Runtime::wait(secs)
      │         │                    → 中断なら core::end_attempt して「起動しなかった中断」の結果 }
      │         ├─ core::classify_and_decide(…, RunMode::Pipeline) → (JobOutcome, NextStep)
      │         ├─ step::report_execution: 表示(-v のときの結果行・遷移行。常時のエラー行。ログを作れなかった 1 行)
      │         └─ core::apply_outcome → Runtime::write_state(Durable) → StepResult
      │       (ここから pipeline)
      │         ├─ error または分類なし → return outcome_exit_code(...)
      │         ├─ completed → finish → step::remove_if_requested(remove_on_complete なら Runtime::reset_state)、return 0
      │         └─ 中断の合図が立っている → core::resume_point_on_interrupt → core::apply_interrupt
      │                                      → Runtime::write_state(Durable)、return 128 + n
 └─ ループを抜けても完了していない → "cannot determine whether the pipeline is complete"、return 65
```

`run` は、`prepare_session` に `check_job` を渡して(`start_signals` のあと `check_job`。ジョブIDの存在の検査を含み、ジョブの位置を返す)から、指定した 1 ジョブについて、同じ `step` の関数(`decide`、`record_skip`、`run_step`、`remove_if_requested`)で「判断 → 実行 → 分類 → 記録」を行い、`classify_and_decide` は `RunMode::Individual` で呼ぶ。いつ消すかと終了コードの決め方は、`run` と `pipeline` で違い、それぞれに残る(05 §5.1a〜§5.3)。`watch`・`reset`・`validate`・`convert` は、シグナルを準備せず、それぞれ `read_state_raw`/`reset_state`・`reset_all_states`/`check_definition`(v0.9.0 以降は、続けて `.env` の読み取り)/`convert` を呼ぶ(04 の 7 章)。v0.10.0 以降の `watch --follow` だけは、`start_signals` で準備してから、`follow_watch` で `read_state_raw` → `Runtime::now` → 表 → `pipeline_activity` → `Runtime::wait` を繰り返す(05 の 20 章)。

v0.9.0 以降の 3 つの流れ(05 の 15〜17 章、04 の 7.7〜7.9):

```text
graph:      Ctx::check_definition_and_env(定義 64 → command 64 → .env 64、重複キーの警告)
            → (--state)Ctx::definition_run_id 68 → Ctx::read_checked_state(StateRead::Raw。読み取り 65 → 照合 65)
            → overlay_from_state → build_graph → layout_graph → render_text | render_mermaid | render_dot(v0.10.0 以降)→ 標準出力 1 回
--dry-run:  Ctx::prepare_checked(FlowTarget で check_job | check_pipeline を呼び分ける。定義 64 → command 64 → ジョブID 68 → --assume のジョブID 68 → run ID 68
            → .env 64 → ステート 65 → 照合 65)
            → 定義の検査で分割したコマンドを渡す → plan_pipeline | plan_run(拒否 66)→ render_plan → 標準出力 1 回
            (シグナルの準備なし、ステートに書かない、ジョブを起動しない)
init:       render_template(言語, 名前) → Runtime::create_file | 標準出力(ファイル・ステート・.env を読まない)
```

### 3.3 責務の流れ(データ)

```text
定義ファイル ──load_definition──▶ PipelineDefinition ─┐
ステートファイル ──read_state──▶ PipelineState ───────┤
                                                       ▼
                    decide_from_state ──▶ Decision(Run/Skip/Reject)
Runtime::now ──▶ Timestamp ──start_attempt / end_attempt / cancel_attempt──▶ PipelineState(試行。Fast で書く)
Runtime::run_job ──▶ (JobExecution, LogOutcome) ──classify_and_decide──▶ (JobOutcome, NextStep)
                                          │
        apply_outcome / apply_skip / apply_interrupt ──▶ PipelineState(新) ──write_state──▶ ステートファイル
JobOutcome.reported_exit_code ──exit_code_for──▶ conductor の終了コード
```

判断(`decide_from_state`、`classify_and_decide`)と記録(`apply_outcome`、`apply_skip`、`apply_interrupt`)は、どちらも `conductor-core`(`flow/`)の純粋な関数で、`write_state` だけが `conductor-runtime` にある。`--dry-run` の模擬(`plan_pipeline`)も、同じ判断と記録の関数で、メモリ上のステートを進める(`write_state` は呼ばない。05 の 16 章)。

## 4. テスト用の差し替え点

`conductor-cli` の流れは、OS・ファイル・プロセスに直接触れず、`Deps` を介す。`run_cli_with(args, &mut Deps { runtime, sink, env, signals })` に、偽の実装を渡せる。

| trait | 本番の実装 | 責務(メソッド) |
|-------|-----------|----------------|
| `Runtime` | `SystemRuntime` | 14 のメソッド: `load_definition(path, expected_pipeline_name)`、`check_command(command)`、`load_env_files(definition_path, refs)`(v0.9.0 以降。`refs` は v0.10.0 以降 `&[EnvFileRef]`)、`now()`(v0.10.0 以降。本番は runtime の `now`)、`run_job(line, env, unset, signal, log) -> (JobExecution, LogOutcome)`(runtime の `run_job` と同じ形。`unset`・`log` と戻り値の `LogOutcome` は v0.10.0 以降。`env` は v0.9.0 以降。v0.9.5 までは分割済みの `line` なし。v0.9.7 までは `run_job(job, line, env, signal)` で、`job` は偽物がジョブ ID を記録するためだけに受け取った。v0.9.8〜v0.9.9 は `run_job(line, env, signal) -> JobExecution`)、`wait(secs, signal)`(リトライの待ちと、v0.10.0 以降の `watch --follow` の待ち)、`read_state(path, run_id)`、`read_state_raw`、`write_state(path, run_id, state, durability)`、`reset_state`、`reset_all_states`、`run_id_env(path)`(ゆるく読む)、`convert(input, target)`、`create_file(path, bytes, force)`(v0.9.0 以降) |
| `Sink` | `StdSink` | `write_out(bytes)`、`write_err(bytes)`(書くたびに flush) |
| `Env` | `SystemEnv` | `var(name) -> Option<String>`(UTF-8 でない値・未設定は `None`)、`stdout_is_terminal() -> bool` |
| `Signals` | `SystemSignals` | `start(signal: &Arc<InterruptSignal>) -> Result<(), SignalError>`(`Failed`)。プロセスで 1 回だけ呼ぶ(本番は `prepare_session` の 1 か所)。2 回呼んだときの動きは契約の外(UNIX 系は監視が 2 つになり、Windows は `Failed`) |

`SystemRuntime` は、`conductor_core`・`conductor_runtime`・`conductor_converter` の関数への薄い委譲である(`run_id_env` だけは、定義ファイルを取り込みと同じ大きさの上限で読み、`peek_run_id_env` を呼ぶ。読めなければ `None`)。

テストの偽の実装(`crates/cli/tests/common/mod.rs`。`FakeEnv`・`FakeSink` は `crates/cli/tests/common/fakes.rs` にあり、単体テストの `crates/cli/src/tests/common/mod.rs` も、写しを持たずに同じファイルを `#[path]` で読む):

| 名前 | 内容 |
|------|------|
| `FakeEnv` | 環境変数の表(`with(&[(名前, 値)])`)。ロケールを読んだかを記録する |
| `FakeSink` | 標準出力・標準エラーを文字列に溜める(`out_text()`、`err_text()`) |
| `FakeSignals` | `SignalMode`(`Noop`・`Fail`・`RaiseOnStart(番号)`)に応じて、登録の成功・失敗・開始時の合図を再現する |
| `ScriptedRuntime` | ジョブごとの結果(`Exec`)を台本で返す `Runtime`。呼び出しの記録(`log()`、`ran()`)、`run_job` に渡された環境(`run_envs`)と分割済みのコマンド(`run_lines`)。ジョブ ID は、渡された環境の `CONDUCTOR_JOB_ID` から取る(`job_id_of`。無ければパニック) |
| `MemRuntime` | ステートをメモリに持つ `Runtime`(ファイルなし。大きなジョブ数の流れの測定にも使う) |
| `Harness` | 実ファイルの一時ディレクトリで、定義の書き換え・実行・ステートの読み取りをする補助 |

`conductor-runtime` の差し替え点: 待ちの核 `wait_loop` は、`Waitable` trait(`id`、`try_wait`)、シグナルの転送の関数、sleep の関数を引数に取るので、偽の子プロセスで単体テストできる(待ちの間隔 1・2・4・8・10・10… ms、転送が 1 回だけ、`try_wait` の失敗)。`core` の判断は純粋な関数で、差し替え不要。

テスト補助の実行ファイル(配布しない): `conductor-test-helper`(runtime)と `conductor-cli-test-helper`(cli)。`<mode>` を渡して、終了コード・出力・待ち・シグナルへの反応などを再現する子プロセスになる(11 の 4.3)。

## 5. モジュール構成(ファイル単位の責務)

### 5.1 `crates/core/src`

| ファイル | 責務 |
|----------|------|
| `lib.rs` | 公開の再公開 |
| `types.rs` | 共有する型(2.1)。型と語の対応(`as_str`、`parse`)、`DEFAULT_SUCCESS_EXIT_CODES`、`JobExecution` の組み立ての関数と `ExecutionResult` |
| `consistency.rs` | ステートと定義の照合 `check_consistency`、`ConsistencyResult`、`MismatchKind`(純粋)。照合の単体テストも持つ |
| `error.rs` | `DefinitionErrorKind`(34 種)、`DefinitionError`、`escape_display`、`quote_key`、`Locate`(位置→行番号)、ファイル名の表示の上限 `MAX_FILE_DISPLAY_CHARS`(256。`envfile` と共通) |
| `envfile/mod.rs`(v0.9.0 以降) | `.env` の型(`EnvFileErrorKind` 14 種、`EnvFileError`、`EnvWarning`、`EnvEntry`、`EnvFileContent`、`LoadedEnvFiles`、`EnvContext`、`EffectiveEntry`、`EffectiveEnv`)、定数(`MAX_ENV_FILE_BYTES = 1_048_576`、`MAX_ENV_ENTRIES = 10_000`。crate の中の `MAX_REPORTED_ERRORS = 20`、`RESERVED_ENV_PREFIX`)、`is_reserved_env_name`(crate の中)、`env_file_refs`、`summarize_errors`、(v0.10.0 以降)`check_unset_conflicts` |
| `envfile/parse.rs`(同) | `parse_env_file`(`.env` の解析) |
| `envfile/merge.rs`(同) | `merge_env`(層の統合と自動の変数。v0.10.0 以降は消す名前 `removals` も)、crate の中の `merge_env_names`(計画の名前)、`merge_unset_names`(v0.10.0 以降。消す名前の並び) |
| `time.rs`(v0.10.0 以降) | `Timestamp`、`JobDuration`、`format_rfc3339`、`format_compact`、`parse_rfc3339`、`duration_secs`、`job_duration`、`format_duration`(年月日の変換を自前で書く。時計には触れない) |
| `graph/mod.rs`、`edges.rs`、`layout.rs`、`overlay.rs`(同) | 図の型(`FlowEdge`、`FlowGraph`、`GraphNode`、`NodeKind`(`TransitionTarget` の別名)、`NodeState`、`StateOverlay`、`LaneGlyph`、`LaneRow`、`LaneLayout`、`GraphLayout`)、辺の取り出し、「対応する形」の判定とレーンの配置、状態の重ね合わせ |
| `plan/mod.rs`、`simulate.rs`(同) | 計画の型と、模擬(`plan_pipeline`、`plan_run`)。模擬のステートの更新は `flow/record.rs` の関数を呼ぶ |
| `run_id.rs` | `RunId`、`RunIdError`、`is_valid_env_name`(crate の中)、`peek_run_id_env`、`DEFAULT_RUN_ID_ENV`、`MAX_RUN_ID_LEN`(このファイルの中だけ) |
| `definition/mod.rs` | `load_definition`、`parse_definition`(5 段階の流れ) |
| `definition/intake.rs` | 定義の場所の決定、パスで種別の確認(通常のファイルでなければ開かない)、開いて、開いたハンドルから種別と大きさを確かめる、1 MiB 上限の読み取り、UTF-8 の確認 |
| `definition/parse.rs` | `toml` で、位置(バイト範囲)を保つ木に読む。文法の誤りの固定の文 |
| `definition/depth.rs` | 入れ子の深さ(`MAX_NESTING_DEPTH` = 32 段)の検査。誤りの文の上限の数も、この定数から作る |
| `definition/version.rs` | `format_version` の確認 |
| `definition/walk/mod.rs` | 構造・型・遷移・終了コード・ジョブ間の検査と、`PipelineDefinition` の組み立て(`walk`) |
| `definition/walk/retry.rs` | `[job.retry]` の読み取り(`RETRY_KEYS`、`RawRetry`)、ほかの値との関係の検査(`check_retry`)、`RetryPolicy` の組み立て(`build_retry`)。v0.10.0 以降は、`[pipeline.retry]` の既定(`pipeline_retry`、`PipelineRetryDefault`)、ジョブの `retry` の 3 つの書き方(`job_retry`、`JobRetrySetting { Absent, Disabled, Table }`)、既定との重なり |
| `definition/walk/env.rs` | `env` の表と `env_file` の検査(`env_table`、`env_file`。v0.10.0 以降は表 `{ path, required }` も)。v0.10.0 以降: `unset_env` の検査(`unset_env`)と `env` との重なり(`check_unset_env`) |
| `definition/duration.rs` | リトライの間隔の文字列(`<整数><s\|m\|h>`)→ 秒 |
| `flow/mod.rs` | 判断の再公開。`record_of`(crate の中)。v0.10.1 以降は、計画の模擬の内部の入口の `record_at`、`position_of`、`SimulationMarks`(どれも crate の中。05 の 16.2) |
| `flow/attempt.rs`(v0.10.0 以降) | 試行の記録 `start_attempt`、`end_attempt`、`cancel_attempt`(純粋) |
| `flow/activity.rs`(v0.10.0 以降) | `watch --follow` の終わりの判定 `pipeline_activity`、`PipelineActivity`(純粋) |
| `flow/classify.rs` | 分類と報告する終了コード |
| `flow/decide.rs` | 起動前の判断(公開はステートからの `decide_from_state`。判定の本体 `decide_before_run` はこのファイルの中だけ。v0.10.1 以降は、計画の模擬の内部の入口 `decide_at`(crate の中)) |
| `flow/next_step.rs` | 遷移・完了・スキップ対象・再開位置(`classify_and_decide`)。v0.9.0 以降は、遷移の解決 `resolve_transition` を切り出し、図と共有する。分類から遷移への対応は `Classification::transition_on` を使う。v0.10.1 以降は、内部の入口 `classify_and_decide_at`(crate の中) |
| `flow/record.rs` | ステートの更新: `initial_state`、`apply_outcome`、`apply_skip`、`apply_interrupt`(純粋)と、その場で書き換える版 `apply_outcome_in`・`apply_skip_in`(crate の中。v0.10.1 以降は目印 `SimulationMarks` を受け取る)。本番の流れと `--dry-run` の模擬が共に使う。試行は変えない |
| `flow/resume.rs` | ジョブのあいだの中断の再開位置 |
| `flow/retry.rs` | リトライの判断と待ち時間の計算 |

### 5.2 `crates/runtime/src`

| ファイル | 責務 |
|----------|------|
| `lib.rs` | 公開の再公開 |
| `types.rs` | `CommandLine`、`InterruptSignal`、`StateLocation`、`Durability` |
| `error.rs` | `CommandError`、`StateError`(種類の文、理由の固定の日本語の文) |
| `split.rs` | `command` の分割(状態機械)、`check_command` |
| `runner.rs` | `run_job`(分割済みのコマンドでの起動・待ち・シグナルの転送。分け直さない。v0.9.0 以降は環境変数を足す。v0.10.0 以降は、消す名前の `env_remove` と、`log` があるときのパイプと読み取りのスレッド(`copy_stream`、2 秒の `DRAIN_LIMIT`))、`wait_loop` |
| `log.rs`(v0.10.0 以降) | `LogTarget`、`LogOutcome`、`safe_log_stem`、`open_log_file`(ログのファイルの名前と作成。07 の 4.5) |
| `clock.rs`(v0.10.0 以降) | `now`(現在の時刻。時計から秒を取るだけ) |
| `envfile.rs`(v0.9.0 以降) | `load_env_files`(パスの解決、種別の確認 → 開く → 開いたものから大きさの検査(v0.9.7 以降。定義ファイルと同じ順)、上限 + 1 バイトの読み取り(この 4 つは `bounded_read::read_regular_file`)、UTF-8、`parse_env_file` への委譲、同じパスの 1 回だけの読み取り) |
| `bounded_read.rs` | 上限つきの読み取り(`.env` とステートの読み込みが共有する)。`read_regular_file`(形 B: パスで種別 → 通常のファイルだけ開く → 開いたものの種別と大きさ → 上限 + 1 バイトまで。失敗は `ReadFailure`(`Missing`・`Unreadable`・`TooLarge`))、その 4 の段の `read_opened`、`read_limited` |
| `file_create.rs`(同) | `create_file`、`CreateOutcome`、`CreateError`(排他的な作成、`--force` の置き換え) |
| `wait.rs` | `wait_for`(リトライの待機) |
| `temp_file.rs` | 一時ファイルと親のディレクトリの扱い(`dir_or_current`、`temp_prefix`、`temp_name`、`create_temp`、連番 1 つ、`create_temp_beside`)と、一時ファイルに書いて置き換える `replace_via_temp`(ステートの書き込みと `init --force` が共有する。失敗は `ReplaceError`(一時ファイルを作れない / 書き込み・置き換えの失敗)で、誤りの種類の写し方は呼び出し側が決める)。`state`・`file_create`・`platform` が共有し、互いの内部を使わない |
| `platform/mod.rs`、`unix.rs`、`windows.rs` | プロセスグループ(`configure_spawn`)、シグナルの送信(`forward_signal`)、シグナル終了の読み取り(`termination_signal`)、ディレクトリの同期(`sync_dir`)、(v0.10.0 以降)ログのファイルの権限(`restrict_to_owner`。UNIX 系は 0600)。`unsafe` は `unix.rs` の `libc::kill` だけ |
| `state/mod.rs` | 下位モジュールの束ね |
| `state/reader.rs` | ステートの読み込み、検査(v0.10.0 以降は `[[job.attempt]]` の検査 `walk_attempts` を含む)、`read_state`/`read_state_raw`、`inspect_run_state`(`reset --all` 用)。上限つきの読み取りは `bounded_read::read_regular_file`(形 B。`.env` の読み込みと共有。v0.9.7 までは先に開く `read_bounded`) |
| `state/writer.rs` | 原子的な書き込み(一時ファイル、同期、rename、古い一時ファイルの掃除)、`reset_state`、TOML への書き出し(状態の語は `JobStatus::as_str` など) |
| `state/reset_all.rs` | `reset --all` の対象の判定と削除 |
| `bin/conductor-test-helper.rs` | テスト補助の実行ファイル |

### 5.3 `crates/converter/src`

| ファイル | 責務 |
|----------|------|
| `lib.rs` | `convert`、`convert_text`、読み取りの切り替え |
| `types.rs` | `ConversionRequest`、`TargetFormat`、`ConvertedText`、途中の表現 `Value` |
| `error.rs` | `ConversionError`、`ConversionErrorKind`、表示の組み立て |
| `intake.rs` | 入力ファイルを開いて読む(1 MiB 上限、UTF-8) |
| `raw.rs` | 素の木(アリーナ。展開前の構造。再帰なし) |
| `toml_read.rs` | TOML → 素の木 |
| `yaml_read.rs` | YAML のイベント列 → 素の木(明示のスタック。アンカー・エイリアス・重複キーの検出) |
| `scalar.rs` | YAML 1.2 コアスキーマによるスカラーの解決 |
| `resolve.rs` | 文書の確認(RootCheck)、アンカー・マージキーの展開、展開の上限、表せない値の検出 |
| `toml_write.rs` | `Value` → TOML(辞書順) |
| `yaml_write.rs` | `Value` → ブロック形式の YAML(辞書順) |
| `tests/`(`mod.rs`、`convert_roundtrip.rs`、`nfr_security.rs`、`read_resolve.rs`) | 途中の表現(`Value` の木)を比べる単体テスト(`lib.rs` の最後の `#[cfg(test)] mod tests;`) |

### 5.4 `crates/cli/src`

| ファイル | 責務 |
|----------|------|
| `main.rs` | 入口(パニックのフック、`run_cli`、`process::exit` 1 回) |
| `lib.rs` | 差し替え口と `install_panic_hook` の再公開、`run_cli`、`run_cli_with`。最後に単体テストの宣言 `#[cfg(test)] mod tests;` |
| `args.rs` | `lexopt` による引数の解釈(字句 → コマンドごとの文法の検査) |
| `run_id.rs` | run ID の解決(`resolve_run_id`。`watch`・`reset` の `aux_run_id`) |
| `types.rs` | `CommandKind`、`CliCommand`、`ParsedArgs`、`UsageError`、`Language`、`Level`、`GraphFormat`、`GraphOptions`、`DryRunOptions`、`InitTarget`、`InitOptions`(後ろの 5 つは v0.9.0 以降) |
| `deps.rs` | `Deps`、`Runtime`/`Sink`/`Env`/`Signals` と本番の実装 |
| `ctx.rs` | `run`・`pipeline` に共通の前処理(本番の入口は総称の `prepare_session` の 1 つ。シグナルの監視を始めてから、渡された検査を呼び、`(Session { checked: Checked, signal }, T)` を返す。`run` は `check_job` を渡してジョブの位置を、`pipeline` は `check_pipeline` を渡して `()` を受け取る。v0.9.6 までは `prepare_run`・`prepare_pipeline` の 2 つだった。`--dry-run` はシグナルを除く `prepare_checked` で `Checked` を受け取る。`Checked` は検査を終えた定義を `base: CheckedDefinition` に持つ。検査の並びは `check_job`(`run`)と `check_pipeline`(`pipeline`)の 1 か所で、部品は `start_signals`、`require_job`、`require_known`、`check_run_inputs`)、`FlowTarget::{Pipeline, Run}`、`check_definition`(`CheckedDefinition` を返す。分割したコマンドを含む)、`check_definition_and_env`(`validate` と `graph` が共有)、`definition_run_id`、`read_checked_state`(`StateRead::{Live, Raw}`)、`load_env_files`、表示・書き込み・削除の補助(`usage_error`、`show_output`、`state_error`、`write`、`reset`) |
| `step.rs` | 1 つのジョブの共通の手順(`record_skip`(`SkipRecord { state, written }` を返す。v0.9.6 までは組 `(PipelineState, bool)`)、`run_step`、`remove_if_requested`、`StepResult`)と、実行のあとの表示(`report_execution`、`result_detail`)、`run_step` が呼ぶリトライを含む実行 `run_job_with_retry`(非公開。試行ごとに `merge_env`。どの試行も、検査で分割した同じコマンドを渡す。v0.10.0 以降は、試行ごとに時刻を取り、試行を足して `Fast` で書き、`log_dir` を解決して `LogTarget` を作る(`resolve_log_dir`)。ログの警告は `report_log_warnings`)。`run` と `pipeline` が呼ぶ。起動前の判断は core の `decide_from_state` を直接呼ぶ。`run_step` は、定義の検査で分割したそのジョブのコマンドを受け取る |
| `run.rs` | `run_individual`(個別実行)。`prepare_session`(`check_job`)が返す位置でジョブを引く。いつ消すかと終了コードの決定 |
| `pipeline.rs` | `run_pipeline`(一括実行。ジョブのループ、エラーでの停止、ジョブのあいだの中断)、`finish`(`remove_if_requested` を呼んで 0 か 65) |
| `aux_commands.rs` | シグナルの準備をしない補助のコマンド: `watch`・`reset`・`validate`・`convert`、`graph`(`run_graph`。v0.9.0 以降)、`init`(`run_init`。同)。v0.10.0 以降の `watch --follow`(`follow_watch`、`FollowEnd`)だけは、シグナルを準備する |
| `dry_run.rs`(v0.9.0 以降) | `run_dry_run`(`FlowTarget::{Pipeline, Run}`)、`run` と `pipeline` の共通の入口 `enter_flow`(位置引数の欠けは 68、`--dry-run` なら計画へ) |
| `exit.rs` | 終了コードの定数、`exit_code_for`、`signal_exit_code` |
| `panic_hook.rs` | パニックのフック(英語 1 行、終了コード 70) |
| `platform/mod.rs`、`unix.rs`、`windows.rs` | シグナルの監視(UNIX: `signal-hook` のスレッド、Windows: `ctrlc`)、`color_supported`、(v0.10.0 以降)`clear_screen_supported`(`watch --follow` で画面を消してよい OS か。UNIX 系だけ)。登録はプロセスで 1 回。入口の `start_signal_watcher` は、`mod.rs` が OS ごとの `spawn_watcher` を `#[cfg(unix)]`/`#[cfg(windows)]` の `pub(crate) use … as start_signal_watcher` で再公開したもの(runtime の `platform` と同じ再公開の形) |
| `report/mod.rs` | `DisplayEvent`、`Reporter`(水準・言語・色の決定、文言の組み立て、`show`)、`StateContext`(`for_read(Option<&RunId>)` で読み取りの文脈 `Read`/`ReadRunId` を選ぶ。定義との食い違いの表示 `ConsistencyMismatch.context` と、案内の `messages::reset_hint(StateContext)` も同じ文脈を使う。v0.9.7 までは run ID の有無を真偽値 `with_run_id` で渡した)、`push_line`(graph・計画・watch が共有)、入力由来の文字列のエスケープ(`escape_name`、`escape_full`。conductor-core の `escape_display` の包み。計画・watch・図の整形も使う) |
| `report/messages.rs` | 英語と日本語の文言の表、種類ごとの文、`--help` 全文 |
| `report/watch.rs` | `watch` の表(列の幅、色。v0.10.0 以降は所要時間の列) |
| `report/graph_text.rs`、`graph_mermaid.rs`(v0.9.0 以降) | `render_text`(レーンの図・辺の一覧)、`render_mermaid`、`escape_label` |
| `report/graph_dot.rs`(v0.10.0 以降) | `render_dot`、`escape_dot_label` |
| `report/plan_text.rs`(同) | `render_plan` |
| `report/init_template.rs`(同) | `render_template`(ひな形の日本語・英語) |
| `bin/conductor-cli-test-helper.rs` | テスト補助の実行ファイル |
| `tests/`(`mod.rs`、`common/`、README の検査の `docs.rs` と、引数・表示などのファイル) | クレートの内部(引数の解析、表示、run ID、終了コード)と README を直接確かめる単体テスト(`lib.rs` の最後の `#[cfg(test)] mod tests;`) |

## 6. 設計上の要点(実装から読み取れること)

- `Ctx` は `runtime`・`signals`・`sink`・`env`・`reporter` を持つ。`Reporter` は、表示が最初に必要になったときだけ言語・色を判定し、結果を保つ(成功・スキップで `-v` なしなら、判定しない)。
- ステートの更新は、`apply_*`(複製して更新する純粋な関数)で作り、`write_state` で保存する。更新が元と等しいとき(`AlreadyExecuted` など)は書かない。
- `pipeline` は、次のジョブを `NextStep.next_job_id` で決めない。記述順に全ジョブを見て、起動前の判断で決める(スキップ・再開は、ステートが運ぶ)。
- 中断は、`InterruptSignal`(`AtomicI32`)の共有で伝える。監視のスレッドが `raise`、メインのスレッドが読む。
- 外部の入力による失敗は、すべて戻り値(`Result`)で返し、製品のコードに `unwrap`・`expect`・`panic!` を置かない(runtime と converter はテストが検査)。
