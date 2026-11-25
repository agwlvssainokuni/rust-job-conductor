# API の文書

`conductor` には、ネットワークの API(HTTP、gRPC など)は無い。外部の面は、(1) コマンドライン、(2) 定義ファイルの書式、(3) `.env` の書式、(4) ステートファイルの書式、(5) ジョブへ渡す環境変数、(6) 終了コードと標準出力・標準エラー、の 6 つ。内部の面は、4 クレートの公開の関数と型。

## 1. コマンドライン(`crates/cli/src/args.rs`、`types.rs`)

```text
conductor run      [-v] [--config <パス>] [--run-id <ID>] [--remove-on-complete] [--dry-run [--assume <ジョブID>=<終了コード>]...] <ジョブID>
conductor pipeline [-v] [--config <パス>] [--run-id <ID>] [--remove-on-complete] [--dry-run [--assume <ジョブID>=<終了コード>]...] <パイプライン名>
conductor watch    [--config <パス>] [--run-id <ID>]
conductor reset    [--config <パス>] [--run-id <ID> | --all]
conductor validate [-v] [--config <パス>] [<パイプライン名>]
conductor graph    [-v] [--config <パス>] [--format <text|mermaid>] [--state [--run-id <ID>]] [<パイプライン名>]
conductor init     [--config <パス>|-] [--force] [<パイプライン名>]
conductor convert  <ファイル> --to <yaml|toml>
conductor --help | -h | --version | -V
```

- オプションの性質は `OPTIONS`(`args.rs`)の 1 か所に持つ。並びは `Opt` の列挙と `const` の検査で結ぶ。
- `--name value` と `--name=value` の両方を受け付ける。`--` 以降は位置引数。同じオプションの重複は `invalidValue`(68)。
- `--format` の値は `text`・`mermaid` だけ(`args.rs` の検査。`GraphFormat { Text, Mermaid }`)。
- `--run-id` は空(run ID なしに固定)か、`RunId` の規則(`A-Za-z0-9._-`、64 文字まで、先頭が `.` でない)。
- `--assume` は `--dry-run` と一緒のときだけ。0〜255、同じジョブIDの重複は誤り。ジョブIDの存在は定義を読んだあとに検査。
- 使い方の誤り(`UsageErrorKind`、8 種): `UnknownCommand`、`MissingArgument`、`UnexpectedArgument`、`UnknownOption`、`OptionNotAllowed`、`MissingValue`、`InvalidValue`、`UnknownJob`。表示に入力の値は含めない(ジョブIDだけは escape して添える)。

## 2. 定義ファイル(TOML。`crates/core/src/definition/`)

| 場所 | キー | 型と規則 |
|------|------|----------|
| 根 | `format_version` | 整数。1 だけ(無ければ `versionMissing`、違えば `versionUnsupported`。通らなければ他を検査しない) |
| 根 | `pipeline` | テーブル(1 つだけ) |
| 根 | `job` | テーブルの配列(1 つ以上) |
| `[pipeline]` | `name` | 文字列(必須、空でない。`pipeline <名前>` と一致) |
| `[pipeline]` | `description` | 文字列(任意) |
| `[pipeline]` | `run_id_env` | 文字列(環境変数の名前の規則) |
| `[pipeline]` | `remove_on_complete` | 真偽値(既定 `false`) |
| `[pipeline]` | `env` | 表(名前 → 文字列。予約名 `CONDUCTOR_*`、大文字小文字だけ違う名前、NUL は誤り) |
| `[pipeline]` | `env_file` | 文字列、または文字列の配列(空の文字列は誤り。相対パスは定義ファイルのディレクトリが基準) |
| `[[job]]` | `id` | 文字列(必須、一意、空でなく `@` で始めない) |
| `[[job]]` | `command` | 文字列、または文字列の配列(先頭が空でない) |
| `[[job]]` | `warning_exit_codes` | 整数の配列(0 は不可、成功と重ならない) |
| `[[job]]` | `success_exit_codes` | 整数の配列(省略時は `[0]`) |
| `[[job]]` | `transitions` | テーブルの配列。`on`(`success`・`warning`。`error` は不可)、`next`(後ろのジョブIDか `@end`)。同じ `on` は 1 つ |
| `[[job]]` | `retry` | テーブル。`exit_codes`(1〜255、空でない。成功・警告と重ならない)、`max`(1〜100)、`interval`(`<整数><s\|m\|h>`、1 秒〜24 時間)、`multiplier`(2〜10)と `max_interval`(両方か、両方とも無し。`interval` 以上) |
| `[[job]]` | `env`、`env_file` | `[pipeline]` と同じ |

- 許されないキーは `unknownKey`。`[pipeline]` に `retry` は無い(リトライはジョブだけ)。
- 誤りの種類は `DefinitionErrorKind` の 33 種。表示は `<ファイル>:<行>: <種類>: <理由>`。キー名は escape して 64 文字まで。
- 大きさの上限 1 MiB、入れ子 32 段。

## 3. `.env` の書式(`crates/core/src/envfile/parse.rs`)

- `KEY=VALUE` の行、`#` のコメント、空行。単引用符・二重引用符(二重引用符の中のエスケープは `\n`、`\r`、`\t`、`\"`、`\\` だけ)。展開はしない。
- 上限: 1 MiB、10,000 項目。UTF-8 だけ。
- 誤りは `EnvFileErrorKind` の 13 種(`FileMissing`、`Unreadable`、`TooLarge`、`NotUtf8`、`TooManyEntries`、`InvalidName`、`ReservedName`、`CaseConflict`、`MissingEquals`、`UnterminatedQuote`、`InvalidEscape`、`TrailingGarbage`、`NulInValue`)。表示は最大 20 件と残りの件数。値は表示しない。
- 同じファイルでの名前の重複は警告(`DuplicateKey`)で、後ろが勝つ。
- ファイルが無いことは常に誤り(`FileMissing`。任意の `.env` の仕組みは無い)。

## 4. ステートファイル(TOML。`crates/runtime/src/state/`)

- 場所: 定義ファイルと同じディレクトリの `<定義ファイル名>.state`、run ID があれば `<定義ファイル名>.<run-id>.state`。
- 最上位のキー(`TOP_KEYS`、10 個): `format_version`(1)、`run_id`(run ID があるときだけ)、`definition_key`、`definition_path`、`pipeline_name`、`job_order`、`job`、`resume_job_id`(任意)、`end_reached`、`completed`。
- ジョブのキー(`JOB_KEYS`、4 個): `id`、`status`(`pending`・`success`・`warning`・`error`・`skipped`)、`exit_code`(任意)、`skip_reason`(`byTransition`・`byEnd`)。
- 読み取りは厳格: 知らないキー、型の誤り、`status` と `exit_code`/`skip_reason` の不整合、`job_order` との不一致、`completed` と `resume_job_id` の両立、ファイル名との run ID の不一致は、すべて誤り(65)。版が違えば `VersionMismatch`(reset が必要)。
- 書き込みは原子的(一時ファイル `.<名前>.tmp.<pid>.<時刻>.<連番>` → `Durable` なら同期 → `rename`)。24 時間以上前の一時ファイルは後始末。
- 時刻、リトライの回数、command、環境変数、出力は書かない。
- `StateErrorKind`: `Unreadable`、`TooLarge`、`Corrupt`、`VersionMismatch`、`WriteFailed`、`RemoveFailed`。

## 5. ジョブへ渡す環境変数(`crates/core/src/envfile/merge.rs`、`crates/runtime/src/runner.rs`)

- 親の環境を引き継ぎ、そこへ「足す」: `[pipeline].env_file` < `[pipeline.env]` < `[job].env_file` < `[job.env]` < 自動の変数(後ろが勝つ。同じ名前は 1 つ)。
- 自動の変数: `CONDUCTOR_PIPELINE`、`CONDUCTOR_JOB_ID`、`CONDUCTOR_RUN_ID`(run ID があるときだけ)、`CONDUCTOR_ATTEMPT`(試行の番号。1 から)。
- 消す(unset)手段は無い。作業ディレクトリ、標準入出力は引き継ぐ。

## 6. 終了コードと出力(`crates/cli/src/exit.rs`、`report/`)

| コード | 意味 |
|--------|------|
| 0 | 成功、警告、スキップ、補助のコマンドの成功 |
| ジョブの値 | エラーのジョブの終了コード(0〜255 に合わせる。エラーで 0 になるなら 1) |
| 1 | 成功コードに含まれない 0 |
| 64 | 定義の誤り(`.env` の誤り、command の誤りを含む) |
| 65 | ステートの誤り、標準出力に書けない(`watch`・`graph`・`--dry-run`・`init -`)、`init` が作れない |
| 66 | 拒否(順序外) |
| 67 | 変換の誤り |
| 68 | 使い方の誤り |
| 69 | `init` の作る先が既にある |
| 70 | 内部の誤り(パニック、シグナルの準備の失敗) |
| 127 | 起動できなかった |
| 128 + 番号 | シグナルによる終了、または中断 |

- 標準出力: `watch` の表、`graph` の図、`--dry-run` の計画、`init -` のひな形、`convert` の結果(どれも 1 回で書く)。
- 標準エラー: `conductor: error: ...` などの固定の文(英日)、`-v` の結果・スキップ・遷移、リトライの知らせ。
- 色(ANSI)は `watch` の表だけで、UNIX 系の端末かつ `NO_COLOR` が未設定・空のとき。

## 7. ライブラリの公開の口(内部の API)

### conductor-core(`crates/core/src/lib.rs`)

| 関数 | 役割 |
|------|------|
| `load_definition(path, expected_name)` / `parse_definition(text, …)` | 定義の読み込みと検証 → `PipelineDefinition` か `Vec<DefinitionError>` |
| `decide_from_state(job_id, def, state)` | 起動前の判断 → `Decision`(`Run`・`Skip(SkipCause)`・`Reject(RejectReason)`) |
| `classify_and_decide(job, execution, states, def, mode)` | 分類と次の動き → (`JobOutcome`, `NextStep`) |
| `resume_point_on_interrupt(def, states)` | ジョブのあいだの中断の再開位置 |
| `retry_decision(job, execution, retries_done, last_wait)` | `RetryDecision::{Stop, Retry{wait_secs, last_exit_code, max}}`。`job.retry` だけを見る |
| `initial_state`、`apply_outcome`、`apply_skip`、`apply_interrupt` | ステートの更新(純粋) |
| `check_consistency(state, def)` | `ConsistencyResult::{Ok, Mismatch(MismatchKind)}`(`PipelineName`・`JobIds`・`JobOrder`) |
| `parse_env_file`、`merge_env`、`env_file_refs`、`summarize_errors` | `.env` の解析、層の統合、読む対象の一覧、誤りの要約 |
| `graph::build_graph`、`overlay_from_state`、`pending_overlay`、`layout_graph` | 図の構造、状態の重ね合わせ、レーンの配置(`GraphLayout::{Lanes, EdgeList}`) |
| `plan_pipeline`、`plan_run` | `--dry-run` の計画(`ExecutionPlan` か `PlanRejection`) |
| `RunId::parse`、`peek_run_id_env`、`escape_display` | run ID、定義からの `run_id_env` のゆるい取り出し、表示の escape |

### conductor-runtime(`crates/runtime/src/lib.rs`)

| 関数 | 役割 |
|------|------|
| `check_command(&Command)` | command を起動せずに分割・検査 → `CommandLine` か `CommandError`(`UnclosedQuote`・`Empty`) |
| `run_job(&CommandLine, env: &[(&str, &str)], &InterruptSignal)` | 直接起動して待つ → `JobExecution`(`NotStarted`・`StartFailed`・`Exited`) |
| `wait_for(secs, &InterruptSignal)` | `WaitOutcome::{Elapsed, Interrupted(番号)}` |
| `load_env_files(definition_path, refs)` | `.env` の読み取り → `LoadedEnvFiles` か `Vec<EnvFileError>` |
| `read_state`、`read_state_raw` | ステートの読み込み(`read_state` は完了を空として返す) |
| `write_state(path, run_id, state, Durability)` | 原子的な書き込み(`Fast`・`Durable`) |
| `reset_state`、`reset_all_states` | 削除(`ResetAllReport { kept }`) |
| `create_file(path, bytes, force)` | `CreateOutcome::{Created, Replaced}` か `CreateError`(`AlreadyExists`・`ParentMissing`・`IsDirectory`・`PermissionDenied`・`OtherIo`) |

### conductor-converter(`crates/converter/src/lib.rs`)

| 関数 | 役割 |
|------|------|
| `convert(&ConversionRequest)` | ファイルを読み、`TargetFormat::{Yaml, Toml}` へ変換 → `ConvertedText` |
| `convert_text(text, target, file_label)` | 文字列の変換 |

誤りは `ConversionErrorKind`(`FileMissing`・`Unreadable`・`TooLarge`・`Syntax`・`Unrepresentable`・`RootNotMap`)。converter は定義の書式を知らない(キーを検査しない)ため、定義のキーを足しても、変換は影響を受けない。

### conductor-cli(`crates/cli/src/lib.rs`)

| 公開 | 役割 |
|------|------|
| `run_cli(args, env, sink)` | 本番の依存で実行し、終了コードを返す |
| `run_cli_with(args, &mut Deps)` | 差し替えた依存で実行する(テスト用) |
| `install_panic_hook()` | パニックのフック |
| trait `Runtime`(13 メソッド)、`Sink`、`Env`、`Signals` と、本番の `SystemRuntime`、`StdSink`、`SystemEnv`、`SystemSignals`、`Deps`、`SignalError` | 差し替え口 |
