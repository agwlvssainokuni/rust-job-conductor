# API の文書

`conductor` には、ネットワークの API(HTTP、gRPC など)は無い。外部の面は、(1) コマンドライン、(2) 定義ファイルの書式、(3) `.env` の書式、(4) ステートファイルの書式、(5) ジョブへ渡す環境変数、(6) 終了コードと標準出力・標準エラー、(7) ログのファイル、の 7 つ。内部の面は、4 クレートの公開の関数と型(8)。仕様書(`docs/spec/`)がこれらをどこまで書いているかは `code-quality-assessment.md` の「仕様書と実装の差」を参照。

## 1. コマンドライン(`crates/cli/src/args.rs`、`types.rs`)

```text
conductor run      [-v] [--config <パス>] [--run-id <ID>] [--remove-on-complete] [--dry-run [--assume <ジョブID>=<終了コード>]...] <ジョブID>
conductor pipeline [-v] [--config <パス>] [--run-id <ID>] [--remove-on-complete] [--dry-run [--assume <ジョブID>=<終了コード>]...] <パイプライン名>
conductor watch    [--config <パス>] [--run-id <ID>] [--follow [--interval <秒>]]
conductor reset    [--config <パス>] [--run-id <ID> | --all]
conductor validate [-v] [--config <パス>] [<パイプライン名>]
conductor graph    [-v] [--config <パス>] [--format <text|mermaid|dot>] [--state [--run-id <ID>]] [<パイプライン名>]
conductor init     [--config <パス>|-] [--force] [<パイプライン名>]
conductor convert  <ファイル> --to <yaml|toml>
conductor --help | -h | --version | -V
```

- オプションの性質(綴り、値の有無、許すコマンド)は `OPTIONS`(`args.rs`)の 1 か所に持つ。`--name value` と `--name=value` の両方を受け付ける。`--` 以降は位置引数。
- `--follow`(値なし)は `watch` だけ。`--interval <秒>` は `--follow` と一緒のときだけで、1〜3600 の整数(数字だけ)。既定は 2(`DEFAULT_FOLLOW_INTERVAL_SECS`、`types.rs:215`)。`watch` 以外の `--follow --interval` は `--follow` の誤りとして報告する(`args.rs:295`)。
- `--format` は `text`(既定)・`mermaid`・`dot`(`GraphFormat`。`args.rs:608`)。
- `--run-id` は空(run ID なしに固定)か、`RunId` の規則(`A-Za-z0-9._-`、1〜64 文字、先頭が `.` でない)。
- `--assume` は `--dry-run` と一緒のときだけ。0〜255、同じジョブIDの重複は誤り。
- 使い方の誤り(`UsageErrorKind`、8 種): `UnknownCommand`、`MissingArgument`、`UnexpectedArgument`、`UnknownOption`、`OptionNotAllowed`、`MissingValue`、`InvalidValue`、`UnknownJob`。表示に入力の値は含めない。
- 使い方の全文(英日)は `report/messages.rs:78-136`。

### `watch --follow` の契約(`crates/cli/src/aux_commands.rs`、`crates/core/src/flow/activity.rs`)

| 項目 | 動き |
|------|------|
| 準備 | 最初にシグナルの準備(失敗は 70)。定義は検証せず、`run_id_env` だけをゆるく読む |
| 1 回分 | 合図を見る → `read_state_raw` → 有れば今の時刻で表を描く → 終わりの判定 → `--interval` 秒待つ(合図で打ち切る) |
| 描き方 | UNIX 系で標準出力が端末なら、1 回目から毎回 `ESC[H ESC[2J` を前に置く(`aux_commands.rs:142`)。ほか(Windows、パイプ、ファイル)は 2 回目から空の 1 行を前に置く |
| 完了で終わる | `completed = true`(0) |
| 止まったで終わる | 終わりの時刻の無い試行が無く、かつ `error` のジョブがある、または `resume_job_id` のジョブが試行を持つ(0) |
| 続ける | 終わりの時刻の無い試行がある(実行中、リトライの待ち、強制終了の後)、またはどれでもない(始まる前、ジョブのあいだ) |
| 消えたで終わる | 一度見えたステートが 2 回続けて無い(0) |
| 中断 | 合図で 0 |
| 誤り | `Unreadable` は描かずに 1 秒後に読み直し、3 回続けば 65。壊れた・古い形・大きすぎるは 65。標準出力に書けなければ 65 |

## 2. 定義ファイル(TOML。`crates/core/src/definition/`)

| 場所 | キー | 型と規則 |
|------|------|----------|
| 根 | `format_version` | 整数。1 だけ(通らなければ他を検査しない) |
| 根 | `pipeline` | テーブル(1 つ) |
| 根 | `job` | テーブルの配列(1 つ以上) |
| `[pipeline]` | `name` | 文字列(必須、空でない) |
| `[pipeline]` | `description` | 文字列(任意) |
| `[pipeline]` | `run_id_env` | 文字列(環境変数の名前の規則) |
| `[pipeline]` | `remove_on_complete` | 真偽値(既定 `false`) |
| `[pipeline]` | `env` | 表(名前 → 文字列。予約名 `CONDUCTOR_*`、大文字小文字だけ違う名前、NUL は誤り) |
| `[pipeline]` | `env_file` | 文字列かテーブル `{ path, required }`、またはその配列(`required` の既定は `true`。表のキーは `path` と `required` だけ) |
| `[pipeline]` | `retry` | テーブル(全ジョブの既定。キーと検査は `[job.retry]` と同じ。`false` も誤り) |
| `[pipeline]` | `log_dir` | 空でない文字列(相対は定義ファイルのディレクトリが基準) |
| `[pipeline]` | `unset_env` | 文字列の配列(名前の規則は `env` と同じ。同じ名前は 1 つにまとめる。`env`・`.env` の名前と重なると誤り) |
| `[[job]]` | `id` | 文字列(必須、一意、空でなく `@` で始めない) |
| `[[job]]` | `command` | 文字列、または文字列の配列(先頭が空でない) |
| `[[job]]` | `warning_exit_codes` | 整数の配列(0 は不可、成功と重ならない) |
| `[[job]]` | `success_exit_codes` | 整数の配列(省略時は `[0]`) |
| `[[job]]` | `transitions` | テーブルの配列。`on`(`success`・`warning`)、`next`(後ろのジョブIDか `@end`) |
| `[[job]]` | `retry` | テーブル(`exit_codes`、`max`、`interval`、任意の `multiplier` と `max_interval`)か `false`。書かなければ `[pipeline.retry]` の複製、表なら丸ごとそれ(既定と混ぜない)、`false` なら無し |
| `[[job]]` | `env`、`env_file`、`unset_env` | `[pipeline]` と同じ(`log_dir` は書けない) |

- キーの表: `PIPELINE_KEYS` 9 個、`JOB_KEYS` 9 個(`crates/core/src/definition/walk/mod.rs:47-70`)。許されないキーは `unknownKey`。
- 誤りの種類は `DefinitionErrorKind` の 34 種(v0.10.0 で `UnsetEnvConflict` が増えた)。表示は `<ファイル>:<行>: <種類>: <理由>`。
- v0.10.0 で増えた理由の文(固定): `` `log_dir` が空です ``(`emptyField`)、`` `unset_env` は、文字列の配列で書きます ``・`` `unset_env` の要素は、文字列で書きます ``(`typeMismatch`)、`` `unset_env` に空の文字列は書けません ``(`emptyField`)、`` `unset_env` の名前が、`env` の名前と重なっています ``(`unsetEnvConflict`)、`` `env_file` は、文字列かテーブル、またはその配列で書きます ``・`` `env_file` の配列の要素は、文字列かテーブルで書きます ``(`typeMismatch`)、`` `retry` はテーブルか `false` で書きます ``・`` `[pipeline.retry]` はテーブルで書きます ``(`retryNotTable`)、既定の `exit_codes` とジョブの成功・警告の重なり(`retryExitCodesOverlap`)。場所は `walk/mod.rs:359-367`、`walk/env.rs:107-281`、`walk/retry.rs:43-50`。
- 大きさの上限 1 MiB、入れ子 32 段。

## 3. `.env` の書式(`crates/core/src/envfile/parse.rs`)

- `KEY=VALUE` の行、`#` のコメント、空行。単引用符・二重引用符(二重引用符の中のエスケープは `\n`、`\r`、`\t`、`\"`、`\\` だけ)。展開はしない。上限は 1 MiB、10,000 項目。UTF-8 だけ。
- 誤りは `EnvFileErrorKind` の 14 種(`FileMissing`、`Unreadable`、`TooLarge`、`NotUtf8`、`TooManyEntries`、`InvalidName`、`ReservedName`、`CaseConflict`、`MissingEquals`、`UnterminatedQuote`、`InvalidEscape`、`TrailingGarbage`、`NulInValue`、`UnsetConflict`)。表示は最大 20 件と残りの件数。値は表示しない。`UnsetConflict` は v0.10.0(`envfile/mod.rs:89`)。
- `required = false` のファイルが無いだけなら誤りでも警告でもない。あって読めないときは誤り。

## 4. ステートファイル(TOML。`crates/runtime/src/state/`)

- 場所: 定義ファイルと同じディレクトリの `<定義ファイル名>.state`、run ID があれば `<定義ファイル名>.<run-id>.state`。
- 最上位のキー(`TOP_KEYS`、10 個): `format_version`(1)、`run_id`(run ID があるときだけ)、`definition_key`、`definition_path`、`pipeline_name`、`job_order`、`job`、`resume_job_id`(任意)、`end_reached`、`completed`。
- ジョブのキー(`JOB_KEYS`、5 個): `id`、`status`、`exit_code`、`skip_reason`、`attempt`。試行のキー(`ATTEMPT_KEYS`、2 個): `started_at`(必須)、`ended_at`(任意)。時刻は UTC の RFC 3339(秒まで、`Z`)。場所は `state/reader.rs:192-208`。
- 読み取りは厳格: 知らないキー、型の誤り、状態と終了コードの不整合、`success`・`warning`・`error` のジョブに試行が無い(`NO_ATTEMPT`)、最後でない試行に終わりが無い(`UNFINISHED_ATTEMPT`)、時刻の書式の誤り(`BAD_TIME`)、`job_order` との不一致、ファイル名との run ID の不一致は、すべて誤り(65)。終わりが始まりより前でも誤りにしない(時計の戻り)。理由の文は `crates/runtime/src/error.rs:99-132`。
- 書き込みは原子的(一時ファイル → `Durable` なら同期 → `rename`)。試行を始めたときは `Fast`、結果は `Durable`。
- `StateErrorKind`: `Unreadable`、`TooLarge`、`Corrupt`、`VersionMismatch`、`WriteFailed`、`RemoveFailed`。

## 5. ジョブへ渡す環境変数(`crates/core/src/envfile/merge.rs`、`crates/runtime/src/runner.rs`)

- 親の環境を引き継ぎ、`unset_env`(`[pipeline]` と `[[job]]` の和)の名前を除いてから、`[pipeline].env_file` < `[pipeline.env]` < `[job].env_file` < `[job.env]` < 自動の変数、の順に足す(後ろが勝つ)。
- 自動の変数: `CONDUCTOR_PIPELINE`、`CONDUCTOR_JOB_ID`、`CONDUCTOR_RUN_ID`(run ID があるときだけ)、`CONDUCTOR_ATTEMPT`(試行の番号。1 から)。自動の変数は消せない。
- 作業ディレクトリと標準入力は引き継ぐ。標準出力・標準エラーは、`log_dir` が無ければ引き継ぎ、あればパイプ。

## 6. 終了コードと出力(`crates/cli/src/exit.rs`、`report/`)

| コード | 意味 |
|--------|------|
| 0 | 成功、警告、スキップ、補助のコマンドの成功、`watch --follow` の完了・止まった・消えた・中断 |
| ジョブの値 | エラーのジョブの終了コード(0〜255 に合わせる。エラーで 0 になるなら 1) |
| 1 | 成功コードに含まれない 0 |
| 64 | 定義の誤り(`.env` の誤り、command の誤り、`unset_env` の重なりを含む) |
| 65 | ステートの誤り、標準出力に書けない(`watch`・`graph`・`--dry-run`・`init -`)、`init` が作れない |
| 66 | 拒否(順序外) |
| 67 | 変換の誤り |
| 68 | 使い方の誤り |
| 69 | `init` の作る先が既にある |
| 70 | 内部の誤り(パニック、シグナルの準備の失敗) |
| 127 | 起動できなかった(ログのファイルを作れず起動しなかったときを含む) |
| 128 + 番号 | シグナルによる終了、または中断 |

- 標準出力: `watch` の表(ジョブ・状態・所要時間・補足の 4 列。`--follow` の各回を含む)、`graph` の図、`--dry-run` の計画、`init -` のひな形、`convert` の結果。
- 標準エラー: `conductor: error: ...` などの固定の文(英日)、`-v` の結果・スキップ・遷移、リトライの知らせ、ログの警告(`LogWriteFailed`、`LogOutputCutShort`)。ログの文は `messages.rs:501-508`。
- 所要時間の列の見出しは `DURATION`/`所要時間`(`messages.rs:551`)。計算と書き方は core の `job_duration`・`format_duration`。
- 色(ANSI)は `watch` の表だけで、UNIX 系の端末かつ `NO_COLOR` が未設定・空のとき。

## 7. ログのファイル(`crates/runtime/src/log.rs`。v0.10.0、v0.10.1)

- 置き場所: 解決した `log_dir` の直下(無ければ親を含めて作る)。名前: `<safe_log_stem(ジョブID)>.<format_compact(開始の時刻)>.<試行の番号>.log`。衝突は `-2`〜`-1000` を試行の番号の後に足す。
- `safe_log_stem`(`log.rs:82-105`)の手順: (1) `/` `\` `:` `*` `?` `"` `<` `>` `|` と制御文字を `_`、(2) 先頭の `.` を `_`、(3) 最初の `.` より前の部分から前後の半角空白を除いた値が Windows の予約名(`CON`、`PRN`、`AUX`、`NUL`、`COM1`〜`COM9`、`LPT1`〜`LPT9`。大文字小文字を問わない)なら先頭に `_`(空白を除くのは比べるときだけ。v0.10.1、`log.rs:90`)、(4) 100 バイトまでに切る、(5) 末尾の `.` か空白を `_`。
- UNIX 系は 0600、Windows は既定の権限。中身はジョブの出力のバイト列だけ。`conductor` は消さない。
- 作れなければ起動せず 127 と `conductor: error:` の 1 行。書き込みの失敗と、終了後 2 秒の待ち切れは警告の 1 行(終了コードは変えない)。

## 8. ライブラリの公開の口(内部の API)

### conductor-core(`crates/core/src/lib.rs`)

| 関数 | 役割 |
|------|------|
| `load_definition`、`parse_definition` | 定義の読み込みと検証 → `PipelineDefinition` か `Vec<DefinitionError>` |
| `decide_from_state`、`classify_and_decide`、`resume_point_on_interrupt`、`retry_decision` | 純粋な判断 |
| `initial_state`、`apply_outcome`、`apply_skip`、`apply_interrupt` | ステートの更新(本番と模擬が共有) |
| `start_attempt`、`end_attempt`、`cancel_attempt` | 試行の時刻の記録(本番だけ。v0.10.0) |
| `pipeline_activity` → `PipelineActivity::{Completed, Running, Stopped, Between}` | 動きの様子(v0.10.0) |
| `check_consistency` | ステートと定義の照合 |
| `parse_env_file`、`merge_env`、`env_file_refs`、`summarize_errors`、`check_unset_conflicts` | `.env` と環境の統合 |
| `Timestamp`、`format_rfc3339`、`format_compact`、`parse_rfc3339`、`duration_secs`、`job_duration`、`format_duration`、`JobDuration` | 時刻(v0.10.0) |
| `graph::build_graph`、`overlay_from_state`、`pending_overlay`、`layout_graph` | 図の構造と配置 |
| `plan_pipeline`、`plan_run` | `--dry-run` の計画 |
| `RunId::parse`、`peek_run_id_env`、`escape_display` | run ID と表示の escape |

v0.10.1 の crate の中の入口(公開しない。`flow/mod.rs:23-185`): `decide_at`(`flow/decide.rs:128`)、`classify_and_decide_at`(`flow/next_step.rs:67`)、`apply_outcome_in`・`apply_skip_in`(`flow/record.rs:110`・`189`)、`record_at`、`status_at`、`position_of`、`SimulationMarks`(`settled_prefix`、`pending_prefix`、`error_records`)。使うのは `plan/simulate.rs` の `plan_pipeline` だけ。公開の口の並び(`lib.rs`)は v0.10.0 と同じ。

### conductor-runtime(`crates/runtime/src/lib.rs`)

| 関数 | 役割 |
|------|------|
| `check_command(&Command)` | → `CommandLine` か `CommandError`(`UnclosedQuote`・`Empty`) |
| `run_job(&CommandLine, env: &[(&str, &str)], unset: &[&str], &InterruptSignal, Option<&LogTarget>)` | → (`JobExecution`, `LogOutcome`) |
| `now()` | → `Timestamp`(範囲の外は端に寄せる) |
| `open_log_file(&LogTarget)`、`safe_log_stem(&str)` | ログのファイルと名前 |
| `wait_for(secs, &InterruptSignal)` | → `WaitOutcome::{Elapsed, Interrupted(番号)}` |
| `load_env_files(definition_path, refs)` | → `LoadedEnvFiles` か `Vec<EnvFileError>` |
| `read_state`、`read_state_raw`、`write_state`、`reset_state`、`reset_all_states` | ステートの読み書きと削除 |
| `create_file(path, bytes, force)` | → `CreateOutcome::{Created, Replaced}` か `CreateError`(`AlreadyExists`・`ParentMissing`・`IsDirectory`・`PermissionDenied`・`OtherIo`) |

契約の表(`lib.rs` の `//!`)の `run_job` の行は v0.10.1 で `unset` を書き足した。同じファイルの「呼び出しの順序」(`lib.rs:72-82`)は、まだ「初期ステートを `Fast` で書く」の形。

### conductor-converter(`crates/converter/src/lib.rs`)

| 関数 | 役割 |
|------|------|
| `convert(&ConversionRequest)` | ファイルを読み、`TargetFormat::{Yaml, Toml}` へ変換 → `ConvertedText` |
| `convert_text(text, target, file_label)` | 文字列の変換 |

誤りは `ConversionErrorKind`(`FileMissing`・`Unreadable`・`TooLarge`・`Syntax`・`Unrepresentable`・`RootNotMap`)。converter は定義のキーを知らない。

### conductor-cli(`crates/cli/src/lib.rs`)

| 公開 | 役割 |
|------|------|
| `run_cli(args, env, sink)` | 本番の依存で実行し、終了コードを返す |
| `run_cli_with(args, &mut Deps)` | 差し替えた依存で実行する(テスト用) |
| `install_panic_hook()` | パニックのフック |
| trait `Runtime`(14 メソッド。`now` を含む)、`Sink`、`Env`(`var`、`stdout_is_terminal`)、`Signals` と、本番の `SystemRuntime`、`StdSink`、`SystemEnv`、`SystemSignals`、`Deps`、`SignalError` | 差し替え口 |
