# API ドキュメント

外部の API(HTTP など)は無い。公開面は、(1) コマンドライン、(2) 定義ファイルの書式、(3) ステートファイルの書式、(4) 終了コード、(5) 4 クレートの Rust API である。

## 1. コマンドライン(`crates/cli/src/args.rs`)

```text
conductor run [-v] [--config <path>] [--run-id <id>] [--remove-on-complete] <job-id>
conductor pipeline [-v] [--config <path>] [--run-id <id>] [--remove-on-complete] <pipeline-name>
conductor watch [--config <path>] [--run-id <id>]
conductor reset [--config <path>] [--run-id <id> | --all]
conductor validate [-v] [--config <path>] [<pipeline-name>]
conductor convert <file> --to <yaml|toml>
conductor --help | --version
```

| コマンド | 位置引数 | `-v` | `--config` | `--to` | `--run-id` | `--all` | `--remove-on-complete` |
|----------|----------|------|------------|--------|------------|---------|------------------------|
| run | ジョブID(1) | 可 | 可 | 不可 | 可 | 不可 | 可 |
| pipeline | 名前(1) | 可 | 可 | 不可 | 可 | 不可 | 可 |
| watch | なし | 不可 | 可 | 不可 | 可 | 不可 | 不可 |
| reset | なし | 不可 | 可 | 不可 | 可 | 可(`--run-id` と併用不可) | 不可 |
| validate | 名前(0 か 1) | 可 | 可 | 不可 | 不可 | 不可 | 不可 |
| convert | 入力ファイル(1) | 不可 | 不可 | 必須 | 不可 | 不可 | 不可 |

解釈の順序(`parse_args` → `build_command`):

1. `lexopt` で字句に分ける。`--help`/`-h`/`--version`/`-V` は、ほかの誤りより先に処理する。
2. 知らないオプションを先に誤りにする(`UnknownOption`)。
3. 最初の位置引数がコマンド名(`run`/`pipeline`/`watch`/`reset`/`validate`/`convert`。それ以外は `UnknownCommand`)。
4. コマンドごとに許されないオプション(`OptionNotAllowed`)、重複(`InvalidValue`)、`--all` と `--run-id` の併用(`InvalidValue`)を検査する。
5. `--run-id` の値は、空、または `RunId` の規則(`A-Za-z0-9._-`、64 文字まで、先頭が `.` でない)。
6. 位置引数の過不足(`MissingArgument`/`UnexpectedArgument`)。`convert` は `--to` が必須で、値は `yaml` か `toml`。

新しい `-v` 型のフラグや値つきオプションを足すときは、`Token` 列挙、`parse_args` の match、`build_command` の許可表、重複検査、`CliCommand` のフィールド、`messages.rs` の `USAGE`(英日)が連動する。新しいサブコマンドは、さらに `CommandKind`(`name()` を含む)と `run_cli_with` の match が連動する。

使い方の誤り(`UsageErrorKind`、8 種): `UnknownCommand`、`MissingArgument`、`UnexpectedArgument`、`UnknownOption`、`OptionNotAllowed`、`MissingValue`、`InvalidValue`、`UnknownJob`。終了コード 68。

run ID の決定(`ctx.rs::prepare`、`args.rs::resolve_run_id`): `--run-id`(空は「run ID なし」に固定)、なければ環境変数(名前は定義の `run_id_env`、既定 `CONDUCTOR_RUN_ID`)。`watch`/`reset` は定義を検証せず、`peek_run_id_env` で環境変数の名前だけをゆるく読む(`aux_run_id`)。

## 2. 定義ファイル(`conductor.toml`、`format_version = 1`)

検査は `crates/core/src/definition/walk.rs`。許されたキー以外は `unknownKey`。

| 場所 | キー | 型・規則 |
|------|------|----------|
| ルート | `format_version` | 整数 1(必須。無ければ `versionMissing`、他は `versionUnsupported`。通らなければ他の検査をしない) |
| ルート | `pipeline` | テーブル 1 つ(必須。配列は不可) |
| ルート | `job` | テーブルの配列(1 つ以上。無ければ `noJobs`) |
| `[pipeline]` | `name` | 文字列、空でない(必須。`pipeline` コマンドで渡した名前と一致) |
| `[pipeline]` | `description` | 文字列(任意) |
| `[pipeline]` | `run_id_env` | 環境変数の名前の形(`A-Za-z_` 始まり、英数字と `_`。違えば `invalidEnvName`) |
| `[pipeline]` | `remove_on_complete` | 真偽値(既定 false) |
| `[[job]]` | `id` | 文字列、空でなく `@` で始まらない、一意 |
| `[[job]]` | `command` | 文字列(空白だけは不可)または文字列の配列(空配列不可、先頭要素は空白だけ不可) |
| `[[job]]` | `warning_exit_codes` | 整数(32 ビット)の配列。0 を含まない。成功コードと重ならない |
| `[[job]]` | `success_exit_codes` | 整数の配列(省略時は `[0]`) |
| `[[job]]` | `transitions` | テーブルの配列 |
| `[[job.transitions]]` | `on` | `success` または `warning`(`error` は `errorTransition`)。同じ `on` の重複は `duplicateTransition` |
| `[[job.transitions]]` | `next` | 記述順で後ろのジョブID、または `@end` |
| `[job.retry]` | `exit_codes` | 1 から 255 の整数の非空の配列(昇順、重複をまとめる)。成功・警告のコードと重ならない |
| `[job.retry]` | `max` | 1 から 100 の整数(必須) |
| `[job.retry]` | `interval` | `<整数><s|m|h>`、1 秒から 24 時間(必須) |
| `[job.retry]` | `multiplier` | 2 から 10 の整数。`max_interval` と同時に書く |
| `[job.retry]` | `max_interval` | `interval` 以上の間隔 |

注意(この intent 向け): 現在の定義には、環境変数の注入のためのキーは無い。許可キーの定数 `PIPELINE_KEYS`、`JOB_KEYS` に無いキーは、すべて `unknownKey` になる。

大きさと深さの上限: 1 MiB(`tooLarge`)、入れ子 32 段(`syntax`)。

### 定義の誤りの種類(`DefinitionErrorKind`、`crates/core/src/error.rs`、30 種)

| 分類 | `code()` の識別子 |
|------|-------------------|
| 取り込み | `fileMissing`、`unreadable`、`tooLarge` |
| 文法・版 | `syntax`、`versionMissing`、`versionUnsupported` |
| 型・キー | `typeMismatch`、`unknownKey`、`emptyField`、`invalidEnvName` |
| ジョブ・遷移 | `invalidJobId`、`duplicateJobId`、`noJobs`、`duplicateTransition`、`unknownTarget`、`errorTransition`、`backwardOrSelfTarget`、`pipelineNameMismatch` |
| 終了コード | `overlappingExitCodes`、`zeroAsWarning` |
| retry | `retryNotTable`、`retryMissingKey`、`retryExitCodesEmpty`、`retryExitCodeInvalid`、`retryExitCodesOverlap`、`retryMaxInvalid`、`retryDurationInvalid`、`retryBackoffIncomplete`、`retryMultiplierInvalid`、`retryMaxIntervalTooSmall` |

表示の形: `<file>:<line>: <kind の文>: <理由>`(行が無いときは `<file>: ...`)。日本語の kind の文は core の `#[error("...")]`、英語は cli の `report/messages.rs::definition_kind`。理由の文は core が固定の日本語で組み立てる(入力の値を入れない。キー名は `quote_key` で escape と 64 文字の切り詰め)。

## 3. ステートファイル(`<定義ファイル名>[.<run-id>].state`、TOML)

上位キー 10 個: `format_version`、`run_id`、`definition_key`、`definition_path`、`pipeline_name`、`job_order`、`job`(配列)、`resume_job_id`、`end_reached`、`completed`。ジョブのキー 4 個: `id`、`status`(pending/success/warning/error/skipped)、`exit_code`、`skip_reason`(byTransition/byEnd)。1 MiB 上限。完了のステートは `read_state` が空として返し(次の実行は先頭から)、`watch` だけが `read_state_raw` で読む。`reset_all_states` は同じディレクトリの `<名前>.state` と `<名前>.<有効な run ID>.state` だけを対象にする。

## 4. 終了コード(`crates/cli/src/exit.rs`)

| コード | 意味 |
|--------|------|
| 0 | 成功、警告、スキップ、`validate` の成功 |
| ジョブのコード | エラーのジョブ(0 から 255 に合わせる。下位 8 ビットで 0 になるなら 1) |
| 127 | 起動失敗 |
| 128+n | シグナルによる終了・中断 |
| 1 | 成功コードに含まれない 0 |
| 64 | 定義の誤り(ファイル・検証・command) |
| 65 | ステートの誤り(読み書き、壊れている、食い違い)、`watch` の出力失敗 |
| 66 | 拒否(順序外) |
| 67 | 変換の誤り、`convert` の出力失敗 |
| 68 | 使い方の誤り |
| 70 | 内部の誤り(パニック、シグナルの準備の失敗) |

## 5. メッセージカタログ(`crates/cli/src/report/`)

- `messages.rs`: `Text { en, ja }` の対と、種類ごとの関数。接頭辞: `conductor: error: `、`conductor: warning: `、`conductor: retry: `、`[conductor] `(`-v`)。固定の英語: `INTERNAL_ERROR`、`SIGNAL_SETUP_FAILED`。
- `report/mod.rs::DisplayEvent` の種類: error 系(`DefinitionErrors`、`CommandErrors`、`Usage`、`State`、`ConsistencyMismatch`、`ResetAllKept`、`Rejected`、`StartFailure`、`SuccessCodeNotListed`、`Conversion`、`OutputFailed`、`CompletionUndetermined`、`SignalSetupFailed`)、notice 系(`RetryScheduled`)、verbose 系(`JobResult`、`JobSkipped`、`Transition`、`Interrupted`、`DefinitionValid`)、出力系(`WatchTable`、`WatchEmpty`、`Help`、`Version`)。
- 言語は `LC_ALL`/`LC_MESSAGES`/`LANG` から遅延して決める。色は標準出力が端末のとき(UNIX のみ `color_supported`)。
- トークン(`success`、`warning`、`error`、`skipped`、`pending`、`job=`、`result=`)は言語によらず英語。
- 入力に由来する文字列は `escape_name`(64 文字)/`escape_path` を通す。

## 6. Rust API(契約 C2 から C4)

### conductor-core(`crates/core/src/lib.rs`)

| 関数 | 入出力 | 性質 |
|------|--------|------|
| `load_definition(path: Option<&Path>, expected_name: Option<&str>)` | → `Result<PipelineDefinition, Vec<DefinitionError>>` | 唯一ファイルに触れる |
| `parse_definition(text, file_label, expected_name)` | 同上 | 純粋 |
| `decide_before_run(job_id, &definition, &job_states, resume_job_id, end_reached)` | → `Decision`(`Run`/`Skip(reason)`/`Reject(reason)`) | 純粋 |
| `classify_and_decide(&job, &execution, &job_states, &definition, RunMode)` | → `(JobOutcome, NextStep)` | 純粋 |
| `resume_point_on_interrupt(&definition, &job_states)` | → `ResumePoint` | 純粋 |
| `retry_decision(&job, &execution, retries_done, last_wait_secs)` | → `RetryDecision` | 純粋 |
| `is_valid_env_name(&str)`、`peek_run_id_env(&str)`、`RunId::parse` | | 純粋 |

### conductor-runtime(`crates/runtime/src/lib.rs`)

`run_job(&JobDefinition, &InterruptSignal) -> JobExecution`(失敗は `start_failure`)、`wait_for(secs, &InterruptSignal) -> WaitOutcome`、`check_command(&Command) -> Result<CommandLine, CommandError>`、`read_state` / `read_state_raw` / `write_state(…, Durability)` / `reset_state` / `reset_all_states`(定義のパスと `Option<&RunId>`)、`check_consistency`、`initial_state`、`apply_outcome`、`apply_skip`、`apply_interrupt`。

### conductor-converter(`crates/converter/src/lib.rs`)

`convert(&ConversionRequest) -> Result<ConvertedText, ConversionError>`、`convert_text(text, TargetFormat, file_label)`。エラー(`ConversionErrorKind`、6 種): `fileMissing`、`unreadable`、`tooLarge`、`syntax`、`unrepresentable`、`rootNotMap`。入力の形式は出力の反対とみなす(拡張子は見ない)。

### conductor-cli(`crates/cli/src/lib.rs`)

`run_cli`、`run_cli_with(args, &mut Deps)`、`parse_args`、`install_panic_hook`。差し替え口 trait: `Runtime`(`load_definition`、`check_command`、`run_job`、`wait`、`read_state`、`read_state_raw`、`write_state`、`reset_state`、`reset_all_states`、`run_id_env`、`convert`)、`Sink`、`Env`、`Signals`。
