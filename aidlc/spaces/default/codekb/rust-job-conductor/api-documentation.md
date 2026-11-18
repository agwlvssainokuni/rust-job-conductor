# API Documentation

外部の面は 3 つ(コマンドライン、定義ファイル、ステートファイル)で、内部の面は crate 間のライブラリ API(契約 C2〜C4)と cli の差し替え口(trait)である。HTTP などのネットワーク API はない。

## 1. コマンドライン(cli)

| コマンド | 位置引数 | オプション | 動作 |
|----------|----------|------------|------|
| `conductor run <job-id>` | ジョブID(1) | `-v`、`--config`、`--run-id`、`--remove-on-complete` | 1 ジョブを実行(RunMode::Individual) |
| `conductor pipeline <name>` | パイプライン名(1) | 同上 | 記述順に全ジョブ(RunMode::Pipeline)。名前を定義の `name` と照合 |
| `conductor watch` | なし | `--config`、`--run-id` | ステートを表で 1 回表示して終わる(完了のステートも読む) |
| `conductor reset` | なし | `--config`、`--run-id` または `--all` | ステートを読まずに削除。`--all` は run ID の有無によらず全部 |
| `conductor validate [<name>]` | 0〜1 | `-v`、`--config` | 定義と全 `command` の検査だけ。成功は無表示(`-v` で 1 行) |
| `conductor convert <file> --to <yaml\|toml>` | 入力ファイル(1) | `--to`(必須) | 変換結果を標準出力へ 1 回で書く |
| `--help`/`-h`、`--version`/`-V` | | | `--` より前にあれば他の誤りより先に処理 |

- `--config` の既定は実行ディレクトリの `conductor.toml`。`--run-id` の値は空(run ID なしに固定)か、`A-Za-z0-9._-`・64 文字まで・先頭が `.` でない。2 回以上の `-v`・`--config`・`--to`・`--remove-on-complete` は `invalidValue`(68)。
- 引数は `OsString` のまま保つ(UTF-8 でないパスも使える)。シェルは介さない。

### 終了コード(`cli/src/exit.rs`、`flow/classify.rs`)

| 値 | 意味 |
|----|------|
| 0 | 成功・警告・スキップ、補助コマンドの成功 |
| ジョブの値 | エラーのジョブの終了コード(0〜255 に合わせる。範囲外は下位 8 ビット、エラーで 0 になるなら 1) |
| 1 | 成功コードにない `0`(`SuccessCodeNotListed` を表示) |
| 64 | 定義の誤り(ファイル・文法・検証・`command`・名前の不一致)。`validate` も |
| 65 | ステートの誤り(読めない・壊れ・版・食い違い・書き込み/初期化の失敗)、`watch` の標準出力の失敗 |
| 66 | 拒否(順序外。`Decision::Reject`) |
| 67 | 変換の誤り、`convert` の標準出力の失敗 |
| 68 | 使い方の誤り(run ID の値の誤りを含む) |
| 70 | 内部の誤り(パニック、シグナルの準備の失敗) |
| 127 | ジョブを起動できなかった |
| 128 + 番号 | シグナルで終了したジョブ、または conductor が中断された |

### 出力の水準(`DisplayEvent::level`)

| 水準 | 出力先 | 条件 | 例 |
|------|--------|------|-----|
| Error | 標準エラー | 常に | `DefinitionErrors`、`State`、`Rejected`、`StartFailure`、`SuccessCodeNotListed`、`Usage` ほか |
| Verbose | 標準エラー | `-v` のとき | `JobResult`、`JobSkipped`、`Transition`、`Interrupted`、`DefinitionValid` |
| Output | 標準出力 | 常に(失敗は値で返す) | `WatchTable`、`WatchEmpty`、`Help`、`Version` |

行の形式: エラーは `conductor: error: <場所>: <種類の文>[: <原因>]`、警告は `conductor: warning: ...`、`-v` は `[conductor] job=<id> ...` と `[conductor] transition: <on> -> <target>`。ジョブ本体の標準出力・標準エラーは加工せずに通す。言語は `LC_ALL`→`LC_MESSAGES`→`LANG` の最初の空でない値が `ja` で始まれば日本語、それ以外は英語(最初に必要になったときだけ判定し、結果を保持)。色は UNIX かつ標準出力が端末かつ `NO_COLOR` が空のときだけ(`watch` の状態の文字だけ)。トークン(`success`・`warning`・`error`・`skipped`・`pending`・`job=`・`result=`)は言語によらず英語。

## 2. 定義ファイル `conductor.toml`(format_version = 1)

| 位置 | キー | 型 | 必須 | 検査 |
|------|------|-----|------|------|
| 最上位 | `format_version` | 整数 | 必須 | `1` のみ。なければ `versionMissing`、他は `versionUnsupported`。通らなければ他の検査はしない |
| 最上位 | `pipeline` | テーブル(1 つ) | 必須 | 配列は `typeMismatch` |
| 最上位 | `job` | テーブルの配列 | 必須(1 つ以上) | なければ `noJobs` |
| `[pipeline]` | `name` | 文字列 | 必須 | 空は `emptyField`。`pipeline <name>` と不一致は `pipelineNameMismatch` |
| `[pipeline]` | `description` | 文字列 | 任意 | |
| `[pipeline]` | `run_id_env` | 文字列 | 任意 | `A-Za-z_` で始まり `A-Za-z0-9_`(`invalidEnvName`)。既定 `CONDUCTOR_RUN_ID` |
| `[pipeline]` | `remove_on_complete` | 真偽値 | 任意(既定 false) | 型の誤りは `typeMismatch` |
| `[[job]]` | `id` | 文字列 | 必須 | 空・`@` 始まりは `invalidJobId`、重複は `duplicateJobId` |
| `[[job]]` | `command` | 文字列、または文字列の配列 | 必須 | 空・空の配列・先頭が空は `emptyField`、配列の要素が文字列でないと `typeMismatch`。起動前に cli が `check_command`(引用符の閉じ忘れなど)で 64 |
| `[[job]]` | `success_exit_codes` | 整数の配列(i32) | 任意(省略 = `[0]`) | 書いたらその値だけが成功 |
| `[[job]]` | `warning_exit_codes` | 整数の配列(i32) | 任意 | `0` を含むと `zeroAsWarning`、成功と重なると `overlappingExitCodes` |
| `[[job.transitions]]` | `on` | `"success"` か `"warning"` | 必須 | `"error"` は `errorTransition`、他は `typeMismatch`、同じ `on` は `duplicateTransition` |
| `[[job.transitions]]` | `next` | 文字列(ジョブID か `@end`) | 必須 | 存在しなければ `unknownTarget`、前または自分は `backwardOrSelfTarget` |

- 未知のキーは、最上位・`[pipeline]`・`[[job]]`・`[[job.transitions]]` のどこでも `unknownKey`(位置つき)。誤りはすべて集めて位置順に返す(ファイルの取り込み・文法・版の誤りは 1 件で終える)。
- 上限: ファイル 1 MiB(`tooLarge`)、入れ子 32 段、UTF-8 のみ(`unreadable`)。
- 誤りの種類(20): `fileMissing`、`unreadable`、`tooLarge`、`syntax`、`typeMismatch`、`unknownKey`、`versionMissing`、`versionUnsupported`、`unknownTarget`、`errorTransition`、`backwardOrSelfTarget`、`overlappingExitCodes`、`zeroAsWarning`、`pipelineNameMismatch`、`duplicateJobId`、`invalidJobId`、`noJobs`、`emptyField`、`duplicateTransition`、`invalidEnvName`。表示は `<file>:<line>: <種類の文>: <固定の理由>`。
- 現在、再試行に関するキーはない(`retry` のようなキーを書くと `unknownKey`)。

## 3. ステートファイル(runtime が読み書き)

- 場所: 定義ファイルと同じディレクトリの `<定義ファイル名>.state`(run ID があれば `<定義ファイル名>.<run-id>.state`)。一時ファイルは `.<ステート名>.tmp.<pid>.<ns>.<seq>`(24 時間たった残りを後始末)。
- 内容(TOML、`format_version = 1`): `definition_key`、`definition_path`、`pipeline_name`、`job_order`(ジョブIDの配列)、`[[job]]`(`id`、`status` = pending/success/warning/error/skipped、`exit_code`、`skip_reason` = byTransition/byEnd)、`resume_job_id`、`run_id`(run ID ありのときだけ)、`end_reached`、`completed`。command・引数・環境変数・出力は書かない。
- 読み取りの検査: 上限 1 MiB、UTF-8、TOML、版(先に確認)、最上位 10 キー・ジョブ 4 キー以外は `corrupt`、status と skip_reason・exit_code の整合、ジョブの並びと `job_order` の一致、`resume_job_id` が `job_order` にあること、`completed` と `resume_job_id` が両立しないこと、`run_id` とファイル名の一致。
- `read_state`: 存在しない・長さ 0・`completed = true` は空(`None`)。`read_state_raw`(`watch` 用)は `completed` も返す。
- 照合(`check_consistency`): パイプライン名 → ジョブIDの集合 → 並びの順。食い違いは 65。

## 4. ライブラリ API(crate 間)

### conductor-core(契約 C2)

| 関数 | 入出力 | 備考 |
|------|--------|------|
| `load_definition(path: Option<&Path>, expected: Option<&str>)` | → `Result<PipelineDefinition, Vec<DefinitionError>>` | 唯一の IO は `intake` |
| `parse_definition(text, file_label, expected)` | 同上 | ファイルに触れない入口 |
| `decide_before_run(job_id, &definition, &[JobState], resume_job_id, end_reached)` | → `Decision`(`Run` / `Skip(reason)` / `Reject(reason)`) | 純粋。7 段の規則 |
| `classify_and_decide(&job, &JobExecution, &[JobState], &definition, RunMode)` | → `(JobOutcome, NextStep)` | 純粋。`classify` を内部で呼ぶ |
| `resume_point_on_interrupt(&definition, &[JobState])` | → `ResumePoint` | 純粋 |
| `RunId::parse`、`is_valid_env_name`、`peek_run_id_env` | | run ID と環境変数名の検査 |

主な型(`core/src/types.rs`): `PipelineDefinition`、`JobDefinition`(`id`・`command`・`warning_exit_codes`・`success_exit_codes: Option<Vec<i32>>`・`transitions`)、`Command`(`Line` / `Argv`)、`Transition`・`TransitionOn`・`TransitionTarget`、`JobExecution`(`started`・`exit_code`・`terminated_by_signal`・`start_failure`・`interrupted`・`signal_number`)、`JobStatus`、`SkipReason`、`JobState`、`PipelineState`、`Decision`・`DecisionReason`、`Classification`、`JobOutcome`(`classification: Option<_>`・`reported_exit_code`・`conductor_error`)、`NextStep`、`RunMode`、`ResumePoint`。終了コードは `i32`。

### conductor-runtime(契約 C3)

| 関数 | 役割 |
|------|------|
| `run_job(&JobDefinition, &InterruptSignal) -> JobExecution` | 直接起動し終了を待つ。失敗は戻り値(`start_failure`)。実行中の合図を 1 回だけ転送。起動前に合図が立っていれば起動しない(`started = false`、`interrupted = true`)。時間の上限・再試行はない |
| `check_command(&Command) -> Result<CommandLine, CommandError>` | 起動せずに分割だけ検査(閉じない引用符、空) |
| `read_state` / `read_state_raw` / `write_state(path, run_id, &state, Durability)` / `reset_state` / `reset_all_states` | ステートの入出力 |
| `check_consistency(&state, &definition) -> ConsistencyResult` | `Ok` か `Mismatch(PipelineName | JobIds | JobOrder)` |
| `initial_state`、`apply_outcome`、`apply_skip`、`apply_interrupt` | ステートの純粋な更新 |
| `InterruptSignal`(`raise`・`is_raised`・`signal_number`)、`Durability`(`Durable`/`Fast`)、`StateLocation` | 型 |

### conductor-converter(契約 C4)

`convert(&ConversionRequest) -> Result<ConvertedText, ConversionError>`、`convert_text(text, TargetFormat, file_label)`。入力の形式は `--to` と反対とみなす(拡張子は見ない)。往復で同じ木になることを合否の基準とし、コメントとキーの順序は保持しない。失敗の種類は `fileMissing`、`unreadable`、`tooLarge`、`syntax`、`unrepresentable`(null、文字列でないキー、タグ、日時、i64 超など)、`rootNotMap`。

### cli の差し替え口(`deps.rs`)

| trait | 本番 | 主なメソッド |
|-------|------|--------------|
| `Runtime` | `SystemRuntime` | `load_definition`、`check_command`、`run_job`、`read_state`、`read_state_raw`、`write_state`、`reset_state`、`reset_all_states`、`run_id_env`、`convert` |
| `Sink` | `StdSink` | `write_out`、`write_err`(失敗は `io::Result`) |
| `Env` | `SystemEnv` | `var(name)`、`stdout_is_terminal()` |
| `Signals` | `SystemSignals` | `start(&Arc<InterruptSignal>)`(プロセスで 1 回、2 回目は `AlreadyRegistered`) |

入口: `run_cli(args, &dyn Env, &mut dyn Sink) -> i32`、`run_cli_with(args, &mut Deps) -> i32`、`parse_args`、`install_panic_hook`。

時刻・待ち(スリープ)の trait は存在しない。
