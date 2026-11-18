# Component Inventory

見出しは、`reverse-engineering-timestamp.md` の `analyzed.components` と一致させている。各部品は「責務 / 主なファイル / 依存 / 再試行の変更で触れる点」の順に書く。

## 配布物(パッケージ)

| パッケージ | 種別 | 内容 |
|------------|------|------|
| conductor-core | lib | 定義と判断(純粋) |
| conductor-runtime | lib + 補助 bin | 起動・シグナル転送・ステート |
| conductor-converter | lib | YAML/TOML の変換 |
| conductor-cli | lib + bin `conductor` + 補助 bin | 実行ファイル本体 |

## core::Types

- **責務**: 共有するデータの形(契約 C1)。`PipelineDefinition`、`JobDefinition`、`Command`、`Transition`、`JobExecution`、`JobState`、`PipelineState`、`Decision`、`JobOutcome`、`NextStep`、`RunMode`、`ResumePoint`。
- **ファイル**: `crates/core/src/types.rs`
- **依存**: なし
- **再試行の変更で触れる点**: `JobDefinition` に再試行の設定を足す(`PartialEq, Eq` を導出しているため浮動小数は使いにくい)。`JobExecution` は 1 回の起動の結果で、試行回数の概念はない。`JobState`・`PipelineState` には試行回数のフィールドがない。

## core::DefinitionLoader

- **責務**: 定義ファイルの取り込み(1 MiB)、TOML の位置つきの木(`parse`)、入れ子 32 段(`depth`)、版(`version`)、構造の検査と組み立て(`walk`)。誤りはすべて集める。
- **ファイル**: `crates/core/src/definition/{mod,intake,parse,depth,version,walk}.rs`
- **依存**: toml(`DeTable`・`Spanned`)、core::Describe
- **再試行の変更で触れる点**: `walk.rs` の `JOB_KEYS`(未知のキーは `unknownKey`)、`RawJob`、`Walker::job`、`build_jobs`、`cross_check`。値の検査(型・範囲・組み合わせ)をここに足す。`exit_codes` と `exit_code_of` が整数(i32)の取り出しの前例。

## core::Flow

- **責務**: 起動前の判断(`decide_before_run`)、結果の分類と終了コード(`classify`)、次の動きと完了(`classify_and_decide`)、中断時の再開位置(`resume_point_on_interrupt`)。すべて純粋で、パニックせず必ず結果を返す。
- **ファイル**: `crates/core/src/flow/{mod,classify,decide,next_step,resume}.rs`
- **依存**: core::Types
- **再試行の変更で触れる点**: エラー(ジョブ自身の終了コード)の判定は `classify` の末尾の分岐。中断・起動失敗・シグナル終了・成功コードにない 0(`conductor_error`)は、ジョブ自身のエラーとは性質が違う。`next_step` の `stop()` が「エラーは停止、自動の再試行はしない」の現在の実装。

## core::RunId

- **責務**: run ID(`A-Za-z0-9._-`、64 文字まで、先頭 `.` 不可)と環境変数名の検査、定義からのゆるい `run_id_env` の取り出し。
- **ファイル**: `crates/core/src/run_id.rs`
- **依存**: toml
- **再試行の変更で触れる点**: なし(参考: 「ゆるく読む」`peek_run_id_env` は `watch`・`reset` が定義の検証なしに使う)。

## core::Describe

- **責務**: `DefinitionErrorKind`(20 種)と `DefinitionError` の固定の文の表示、`sanitize`(制御文字・書式文字のエスケープ、切り詰め)、`quote_key`、`Locate`(バイトの位置 → 行)。
- **ファイル**: `crates/core/src/error.rs`
- **依存**: thiserror
- **再試行の変更で触れる点**: 新しい誤りの種類を足すなら `ALL`・`code`・`#[error]` の 3 か所と、cli の `messages.rs::definition_kind`。

## runtime::JobRunner

- **責務**: `run_job`。command の分割 → 起動前の合図の確認 → 起動(UNIX は新しいプロセスグループ)→ `wait_loop`(`try_wait` を 1 ms から 10 ms の間隔でくり返し、まだのときだけ合図を見て 1 回だけ転送)→ `JobExecution`。
- **ファイル**: `crates/runtime/src/runner.rs`
- **依存**: runtime::CommandSplit、runtime::Platform、runtime::Types、core::Types
- **再試行の変更で触れる点**: 試行の間の待ちは、ここでも cli でも未実装。`wait_loop` は `sleep` を関数として受け取る形で、子プロセスの待ち専用(待ちの最中に合図を見るが、転送の相手(子)がいる前提)。

## runtime::CommandSplit

- **責務**: command の文字列を、シェルなしで、状態機械(再帰なし)で分ける。`check_command`(起動せずに検査)。
- **ファイル**: `crates/runtime/src/split.rs`
- **依存**: core::Types(`Command`)

## runtime::Platform

- **責務**: OS 固有の呼び出し(プロセスグループの指定、`kill` の 1 回の送信、シグナルによる終了の読み取り、ディレクトリの同期)。runtime で `unsafe` を許す唯一の場所(`kill` の 1 か所)。転送してよいのは SIGHUP・SIGINT・SIGQUIT・SIGTERM。
- **ファイル**: `crates/runtime/src/platform/{mod,unix,windows}.rs`
- **依存**: libc(UNIX)

## runtime::Types

- **責務**: `InterruptSignal`(原子的な整数。立ったら下りない、最初の番号を保持)、`CommandLine`、`StateLocation`、`MismatchKind`・`ConsistencyResult`、`Durability`。
- **ファイル**: `crates/runtime/src/types.rs`、`crates/runtime/src/error.rs`(`StateError`・`CommandError` と固定の理由)

## runtime::StateStore

- **責務**: ステートの読み込み(上限 1 MiB、版の確認が先、未知のキーは `corrupt`、整合の検査、`completed` は空)、書き込み(一時ファイルの新規作成 → `Durable` なら同期 → `rename` → ディレクトリの同期 → 古い一時ファイルの後始末)、`reset_state`、`reset_all_states`。
- **ファイル**: `crates/runtime/src/state/{reader,writer,reset_all,mod}.rs`
- **依存**: core::Types、runtime::Platform
- **再試行の変更で触れる点**: ジョブのキーは `id`・`status`・`exit_code`・`skip_reason` の 4 つ(`reader.rs::JOB_KEYS`)。ステートに何かを足すなら `reader` の `TOP_KEYS`/`JOB_KEYS`、`walk_job`、`writer::serialize`、`FORMAT_VERSION` に触れる。

## runtime::Recorder

- **責務**: `initial_state`、`apply_outcome`(分類のとおりに status、`exit_code` は `JobExecution` のまま、スキップの列挙、`resume_job_id`/`completed`、`end_reached` は戻さない)、`apply_skip`、`apply_interrupt`。入出力なしで新しい値を返す。
- **ファイル**: `crates/runtime/src/state/record.rs`
- **依存**: core::Types

## runtime::Consistency

- **責務**: ステートと定義の食い違いの検知(パイプライン名 → ジョブIDの集合 → 並び)。ジョブの設定の内容(コマンドや終了コード)は照合しない。
- **ファイル**: `crates/runtime/src/state/consistency.rs`

## cli::ArgParser

- **責務**: `lexopt` による字句分けと、コマンドごとの文法の表。`resolve_run_id`(`--run-id` > 環境変数)。
- **ファイル**: `crates/cli/src/args.rs`、`crates/cli/src/types.rs`(`CliCommand`・`ParsedArgs`・`UsageError`)
- **再試行の変更で触れる点**: 設定を定義ファイルに置く方針なら変更なし(オプションを足す場合のみ文法の表と `USAGE` の文言)。

## cli::Orchestrator

- **責務**: `run`・`pipeline` の流れ。`Ctx::prepare`(シグナルの準備 → 定義 → command の検査 → ジョブID → run ID → ステート → 照合の固定の順序)、`run_individual`、`run_pipeline`、`report_execution`、`write`、`reset`。
- **ファイル**: `crates/cli/src/{ctx,run,pipeline}.rs`
- **依存**: core::Flow、runtime::JobRunner、runtime::StateStore、runtime::Recorder、cli::Deps、cli::Reporter、cli::ExitMapper
- **再試行の変更で触れる点**: `run.rs` の `Decision::Run` の枝と `pipeline.rs` の `Decision::Run` の枝が、同じ形(`run_job` → `classify_and_decide` → `report_execution` → `apply_outcome` → `write_state`)を持つ。再試行のループと待ち(と待ちの最中の中断)は、ここに入る。`pipeline.rs` には、ジョブのあいだの中断の確認(`session.signal.signal_number()`)がある。

## cli::AuxCommands

- **責務**: `watch`(`read_state_raw` → 表)、`reset`(`--all` を含む)、`validate`、`convert`。シグナルの準備はしない。
- **ファイル**: `crates/cli/src/aux_commands.rs`
- **再試行の変更で触れる点**: `validate` は `check_definition`(定義 + command の検査)をそのまま使うため、定義の検査が増えれば自動で効く。`watch` は進行中の再試行を知らない(ステートに試行がないため)。

## cli::Deps

- **責務**: 差し替え口 `Runtime`・`Sink`・`Env`・`Signals` と本番の実装。
- **ファイル**: `crates/cli/src/deps.rs`
- **再試行の変更で触れる点**: 時刻・待ちの差し替え口はない。間隔を実時間なしでテストするには、ここに足すのが自然な位置。

## cli::Reporter

- **責務**: `DisplayEvent` から英語・日本語の行を組み立て、水準(Error / Verbose / Output)で標準エラー・標準出力へ出す。言語・色は遅延。文言は `Text{en, ja}` の定数と関数。
- **ファイル**: `crates/cli/src/report/{mod,messages}.rs`
- **再試行の変更で触れる点**: 進捗(何回目の再試行か、次まで何秒か)には新しい `DisplayEvent` の種類、`level()` の分類、`render` の分岐、`messages.rs` の英語・日本語の対が要る。`-v` なしでも見せるなら `Level::Error`(標準エラー、常に)か新しい水準になる。

## cli::WatchView

- **責務**: `watch` の表(ジョブ・状態・補足の 3 列、全角を幅 2 とみなす整列、状態は常に文字で、色は装飾)。完了のステートは完了の行を足す。
- **ファイル**: `crates/cli/src/report/watch.rs`

## cli::Escape

- **責務**: 入力由来の文字列(ジョブID・キー名・パス)の制御文字・書式文字のエスケープ(名前は 64 文字で切る、パスは切らない)。
- **ファイル**: `crates/cli/src/report/escape.rs`
- **備考**: core・runtime・converter に同じ規則の関数がある(重複。`tests/escape_table.rs` で揃えを検査)。

## cli::SignalWatcher

- **責務**: 専用スレッドで SIGHUP/INT/QUIT/TERM(UNIX は `signal-hook`)または Ctrl+C(Windows は `ctrlc`)を受け、最初の番号を `InterruptSignal` に `raise`。プロセスで 1 回だけ登録。
- **ファイル**: `crates/cli/src/platform/{mod,unix,windows}.rs`

## cli::ExitMapper

- **責務**: 終了コードの定数(0/64/65/66/67/68/70)と、0〜255 への合わせ(`exit_code_for`、`signal_exit_code`)。
- **ファイル**: `crates/cli/src/exit.rs`

## cli::PanicHook

- **責務**: パニックを固定の英語の文で標準エラーに書き、70 で終了。
- **ファイル**: `crates/cli/src/panic_hook.rs`、`crates/cli/src/main.rs`

## converter::Converter

- **責務**: YAML と TOML の相互変換。Intake → Read(YAML はイベント列を明示のスタックで読む)→ RootCheck(複数の文書・マップでない最上位を拒否)→ Resolve(アンカー・エイリアス・マージキーの展開。ノード 100,000・文字 16 MiB・深さ 32 の上限)→ Write。スキーマ(ジョブのキー)を一切知らない。
- **ファイル**: `crates/converter/src/*.rs`
- **依存**: toml、saphyr-parser、thiserror(core には依存しない)
- **再試行の変更で触れる点**: 通常なし。新しい設定が、表・配列・整数・文字列・真偽値・浮動小数で書かれる限り、そのまま往復する。`null`・日時・`i64` を超える整数・タグは `unrepresentable`。TOML への書き出しはキーを整列する(順序は保持しない)。

## TestHelpers

- **責務**: テストの共通部品。cli: `FakeEnv`・`FakeSink`・`FakeSignals`(`SignalMode`)・`ScriptedRuntime`(定義とステートは実物、ジョブの起動だけを台本 `Exec::{Exit, KilledBy, StartFailure}` で偽にし、`during_job`/`after_job` のフックで合図を起こす。呼び出しの順序を `log` に残す)、定義の文字列 `THREE_JOBS` など。バイナリを起動するテストは `bin_support`(`conductor()`、`definition`、`job_block`、30 秒の上限)と `conductor-cli-test-helper`(`exit`・`run`・`sleep`・`say`・`timing` など)。runtime: `conductor-test-helper`(`run <label> <n>`・`sleep`・`env`・`cwd`・`pid`)、`job()`、`run_with_limit`、`wait_for_line`。converter: `yaml()`・`toml()`・`s/i/m/a`・`with_timeout`。
- **ファイル**: `crates/cli/tests/common/mod.rs`、`crates/cli/tests/bin_support/mod.rs`、`crates/cli/src/bin/conductor-cli-test-helper.rs`、`crates/runtime/tests/common/mod.rs`、`crates/runtime/src/bin/conductor-test-helper.rs`、`crates/converter/tests/common/mod.rs`
- **再試行の変更で触れる点**: `ScriptedRuntime::scripts` は、ジョブごとに `VecDeque<Exec>` で結果を順に返す(同じジョブを複数回呼ぶ試験に向く)。待ちの差し替え口がないため、間隔のテストは実時間になる。`JobDefinition` のリテラルを持つ補助(`runtime::job()`、`bin_support::job_block`)は、フィールドが増えると更新が要る。
