## Developer Code Scan Results

対象 intent: `run-id-state`(状態ファイルを実行単位に分ける run ID の導入)。焦点を絞った走査(FOCUSED)。HEAD は `3785d31beaf30964b4adb07ffd8e2bc248673496`。行番号は HEAD の時点。

### Scan Coverage
- **Analyzed deeply**:
  - `crates/runtime/src/`(`types.rs`、`error.rs`、`lib.rs`、`state/{mod,reader,writer,record,consistency}.rs`)
  - `crates/core/src/`(`types.rs`、`lib.rs`、`definition/{mod,walk}.rs`、`error.rs` の `DefinitionErrorKind`)
  - `crates/cli/src/`(`args.rs`、`ctx.rs`、`aux_commands.rs`、`deps.rs`、`lib.rs`、`run.rs`、`pipeline.rs`、`exit.rs`、`types.rs`、`report/{mod,messages}.rs` の関連部分、`report/watch.rs` の冒頭)
  - `README.md`(使い方、オプション表、終了コード表、ステートファイルの場所)
- **Skimmed only**:
  - `crates/converter/src/`(定義を汎用の木として扱う。`[pipeline]` のキー名はコードに現れない)
  - `crates/cli/tests/`、`crates/core/tests/`、`crates/runtime/tests/`、`crates/converter/tests/`(ファイル名・`#[test]` の数・ステート名/使い方/引数の文法を固定する箇所の grep のみ)
  - `crates/runtime/src/{runner,split,platform}`、`crates/core/src/{definition/{intake,parse,depth,version},flow}`、`crates/cli/src/{platform,panic_hook,main,report/{escape,watch(本体)}}`、`src/bin/*`
  - `demo/`、`Cargo.toml`(版 0.4.0、edition 2024 の確認のみ。snapshot の外)
- **Left out**: `.claude/`、`aidlc/`(workspace)、`.idea/`、`target/`、`local-docs/`、`.github/`(CI は前回の記録を引き継ぐ)

### Packages Found
- conductor-cli — bin+lib(`conductor`) — Rust — 引数・流れ・表示・終了コード
- conductor-core — lib — Rust — 定義の検証と判断(純粋)。`PipelineDefinition`・`PipelineState` の型
- conductor-runtime — lib — Rust — ジョブの起動、ステートの入出力
- conductor-converter — lib — Rust — YAML/TOML 変換(`[pipeline]` のキーを知らない)

### Build System
- **Type**: Cargo ワークスペース(`members = ["crates/*"]`、edition 2024、全クレート 0.4.0)
- **Config Files**: `Cargo.toml`、`Cargo.lock`
- **Build Dependencies**: cli → core・runtime・converter / runtime → core

### APIs Discovered
- CLI — `conductor run|pipeline|watch|reset|validate|convert`(6 コマンド)
- 内部ライブラリ API — `read_state`・`write_state`・`reset_state`・`initial_state`・`check_consistency`(runtime)、`load_definition`・`parse_definition`(core)

### Frameworks & Libraries
- lexopt 0.3.2(引数)、toml 1.2.1(定義とステート)、thiserror 2.0.21、signal-hook/ctrlc(シグナル)

### Test Coverage
- **Test Directories**: 各クレートの `tests/`。`#[test]` の数(grep、ソース内の単体テストを含む): cli 17 ファイル/204、core 6/122、runtime 10/165、converter 4/115
- **Test Frameworks**: 標準の `cargo test`、`tempfile`
- **Coverage Config**: 前回の記録(cargo-llvm-cov、行 80% 未満で失敗)。今回は未確認

### Code Quality Indicators
- **Linting**: `rustfmt`・`clippy`(前回の記録。設定ファイルなし)
- **CI/CD**: 今回は未走査(`.github/` は対象外。前回の記録を引き継ぐ)
- **Documentation**: README は日本語で詳細。各 `lib.rs` に設計の表

### Technical Debt Signals
- 使い方の文言が 2 か所(`crates/cli/src/report/messages.rs:57-92` の `USAGE` と README の「使い方」)。
- `PipelineState.definition_key` は「仮」とコメントされ、`definition_path` と同じ文字列で書かれる(`crates/core/src/types.rs:192`、`crates/runtime/src/state/writer.rs:237-240`)。照合には使わない。
- 前回の記録の「`PipelineDefinition.jobs`」は、すでに `job` に改名済み(`crates/core/src/types.rs:33`)。`validate` も追加済み。

## Handoff Summary

### (1) ステートファイルのパスの決め方と呼び出し元
- **導出は 1 か所**: `StateLocation::of(definition_path)`(`crates/runtime/src/types.rs:109-120`)。定義ファイル名(`file_name`)に `.state` を足し、`parent` の下に置く。`file_name` がない(`/`、空)ときは `.state`(テスト `types.rs:209-218`)。`StateLocation` は `path()` だけを持つ(フィールド 1 つ)。
- **runtime 内の呼び出し元**: `read_state`(`state/reader.rs:49-52` → `read_state_file` `:54`)、`write_state`(`state/writer.rs:53-59` → `write_state_file` `:61`)、`reset_state`(`writer.rs:110-121`)。`check_consistency`(`state/consistency.rs:25`)は**パスを使わない**(名前・ジョブIDだけ)。
- **cli の呼び出し元**: `Ctx::prepare`(`ctx.rs:100` `read_state`、`:106` `StateLocation::of`(食い違いの表示用))、`Ctx::write`(`ctx.rs:176`)、`Ctx::reset`(`ctx.rs:184`)、`run_watch`(`aux_commands.rs:29`)、`run_reset`(`aux_commands.rs:49`)、`pipeline.rs:167`(`CompletionUndetermined` の表示用)。`run.rs`(`:63`、`:87`、`:66`、`:75`、`:88`、`:122`、`:126`)と `pipeline.rs`(`:78`、`:81`、`:104-105`、`:134`、`:150`、`:176`)は、`path`(定義ファイルのパス)を `ctx.write`・`ctx.reset`・`initial_state` に渡す。
- **すべての経路が「定義ファイルのパス」だけをキーにする**: `Runtime` trait(`crates/cli/src/deps.rs:74-98`)の `read_state(&Path)`・`write_state(&Path, &PipelineState, Durability)`・`reset_state(&Path)` が、いずれも定義パスを受け取る。run ID を渡すには、この 3 メソッドと `SystemRuntime`(`deps.rs:129-144`)、公開関数 3 つ、テストの偽(`crates/cli/tests/common/mod.rs` の `ScriptedRuntime`(`:237`〜)と `MemRuntime`(`:519`〜))の署名を揃えて変える必要がある。
- **一時ファイルの掃除**: `remove_stale_temps(dir, state_name, now)`(`writer.rs:162-188`)。`write_state_file` の `Durable` の成功後だけ呼ぶ(`writer.rs:100-103`)。`state_name` は `StateLocation` のファイル名そのもの。
- **一時ファイルの名前**: `.<state_name>.tmp.<pid>.<nanos>.<seq>`(`temp_name` `writer.rs:124-126`)。run ID 付きなら `.<def>.<run-id>.state.tmp.…` になり、自動で別名になる。

### (2) `PipelineState` のフィールドと、直列化・解釈・照合の箇所
- **型**(`crates/core/src/types.rs:189-202`): `definition_key: String`、`pipeline_name: String`、`definition_path: String`、`job_order: Vec<String>`、`job_states: Vec<JobState>`、`resume_job_id: Option<String>`、`end_reached: bool`、`completed: bool`(8 フィールド)。`run_id` は無い。
- **構造体リテラルで全フィールドを書く箇所(フィールド追加で壊れる)**: 製品コード `runtime/src/state/reader.rs:~226-235`(`walk` の戻り値)、`runtime/src/state/record.rs:33-46`(`initial_state`)。テスト: `runtime/src/state/consistency.rs:72-81`、`runtime/tests/state_write.rs`、`cli/tests/{report_messages,nfr_robust,flow_pipeline}.rs`、`core/src/types.rs` のテスト。`apply_*`(`record.rs`)は `state.clone()` から作るので、フィールドを足しても変更不要。
- **直列化**: `serialize`(`writer.rs:231-263`)。`toml::Table` に、`format_version`(= `FORMAT_VERSION` 1、`reader.rs:35`)、`definition_key`(`definition_path` と同じ値)、`definition_path`、`pipeline_name`、`job_order`、`job`(配列のテーブル。`job_table` `writer.rs:211-228`)、`resume_job_id`(`Some` のときだけ)、`end_reached`、`completed` を入れる。`run_id` は、ここへ `Option` で足す。
- **解釈**: `parse_state`(`reader.rs:100-126`)が、空 → UTF-8 → TOML → `format_version`(先)→ `walk`。`completed == true` なら `Ok(None)`(`reader.rs:121`)。`TOP_KEYS: [&str; 9]`(`reader.rs:128-138`)にない最上位キーは `corrupt(UNKNOWN_KEY)`(`reader.rs:177-179`)。`JOB_KEYS`(`:140`)は `[[job]]` の中。必須キーは `required`(`:142`)、型の誤りは `WRONG_TYPE`。任意キーの前例は `resume_job_id`(`reader.rs:~204-207` の `table.get`)。**`TOP_KEYS` の要素数(9)と `[&str; 9]` を直すこと**。
- **「`run_id` が**ファイル名と一致すること**」を検査するには、`parse_state` / `walk` はパスを持たない**(`ParseFailure` はパスなし。パスは `read_state_file` が `StateError::new(path, …)` で付ける。`reader.rs:54-81`)。期待する run ID は、`read_state_file` の引数として足し、`walk` の後(または `completed` の判定の前か後かを決めたうえで)に比べる。`completed` のまま残ったステートは `None` で返るので、その場合も一致検査をするかどうかは設計で決める。
- **読み込んだ状態と定義の照合**: `check_consistency(state, definition)`(`consistency.rs:25-46`)は、パイプライン名 → ジョブIDの集合 → 並び、の順(`MismatchKind::{PipelineName, JobIds, JobOrder}` `types.rs:130-137`)。run ID の食い違いはここでは扱わない設計になっている(パスの違いは見ない、と明記: `consistency.rs:23-24`)。run ID の一致は、ファイル名との照合(read 側)で足すのが自然で、`MismatchKind` の追加は不要。
- **エラー**: `StateErrorKind::{Unreadable, TooLarge, Corrupt, VersionMismatch, WriteFailed, RemoveFailed}`(`error.rs:53-60`)と固定の理由(`error.rs:98-124` の `reason`)。`reason` のテスト(`error.rs:131-141`)が、理由の文に `jobs` を含まないことを確かめる。新しい理由(例: run ID の不一致)は、`reason` に足し、`corrupt` で返す(終了コード 65)。
- **`format_version`**: 変更なし(1)。run ID なしのファイルに `run_id` キーを書くかどうかは設計で決める(キーを省略すれば旧ファイルはそのまま読める。`TOP_KEYS` に足しても、任意キーなら互換)。

### (3) `[pipeline]` のキーの検査(`run_id_env` の足し方)
- **既知キーの一覧**: `crates/core/src/definition/walk.rs:32-33` — `ROOT_KEYS = ["format_version","pipeline","job"]`、`PIPELINE_KEYS = ["name","description"]`。
- **未知キーの扱い**: `Walker::check_keys`(`walk.rs:81-91`)が、許可表にないキーごとに `DefinitionErrorKind::UnknownKey`(「キー `x` は定義にありません」。キー名は `quote_key` でエスケープし 64 文字で切る)を集める。`[pipeline]` では `pipeline_section`(`walk.rs:202-250`)の `self.check_keys(table, PIPELINE_KEYS)`(`:226`)。誤りは**止まらずにすべて集め**、最後に位置の順で返す(`walk.rs:416-481`)。
- **型の誤り**: `Walker::string`(`walk.rs:94-113`)。必須でない文字列は `string(table, "description", start, false)`(`:227-228`)の形。型が文字列でなければ `TypeMismatch`(「`description` は文字列で書きます」)。空文字の扱いは `name` だけ(`:233-236`、`EmptyField`)。`run_id_env` は、`description` と同じ形で取り出し、変数名の形の検査(空・使える文字)を足せる。
- **戻り値**: `pipeline_section` は `Option<(String, Option<String>)>`(名前と説明)。`walk` は `PipelineDefinition { name, description, format_version, job }`(`walk.rs:457-462`)を作る。**この構築は製品コードでは 1 か所だけ**。`PipelineDefinition`(`types.rs:26-35`)に `run_id_env: Option<String>` を足すと、テストの構築(`core/src/flow/{next_step,decide,resume}.rs`、`runtime/src/state/{consistency,record}.rs`、`core/src/types.rs`、`runtime/tests/*`、`cli/tests/*` の `PipelineDefinition { … }`)も直す必要がある。
- **誤りの種類**: 変数名の形の誤りに専用の `DefinitionErrorKind`(`core/src/error.rs:31-70`、19 種類)を足すか、`TypeMismatch`/`EmptyField` を使う。**64** で終わる(`EXIT_DEFINITION`)。表示の英語・日本語は `crates/cli/src/report/messages.rs:~130-160` の `definition_kind`(全 kind を網羅する `match`)に足す。
- **converter**: `crates/converter/src/` に `pipeline`・`name`・`description` のキー名は無い。定義を汎用の木として扱うので、`run_id_env` の追加で変更は不要(任意の追加キーを素通しする)。
- **既存テスト**: `core/tests/definition_errors.rs:155`(`[pipeline]` に `description` だけで `name` がない)、`:166`(`br1_6_知らないキー`)、`core/tests/definition_valid.rs`。

### (4) 引数の解釈と `Env`
- **字句**: `lexopt`。`parse_args`(`crates/cli/src/args.rs:66-128`)が、`Token::{Verbose, Config(OsString), To(OsString), Unknown(String), Positional(OsString)}`(`args.rs:45-53`)に分ける。**値を取るオプションの作り方**(`--config`、`args.rs:98-106`): `Long("config") => match parser.value() { Ok(v) => tokens.push(Token::Config(v)), Err(_) => { lex_error.get_or_insert(UsageError::with_subject(Kind::MissingValue, "--config")); break; } }`。`--to` も同形(`:107-113`)。`--config=path` と `--config path` の両方を `lexopt` が処理する。`--run-id` は、同じ形(`Token::RunId(OsString)`)で足せる。値が空(`--run-id ""`、`--run-id=`)は、`parser.value()` が空の `OsString` を返す(`--config` と同じ経路)。
- **文法の表**: `build_command`(`args.rs:154-266`)。コマンドごとの許可は、`takes_verbose`(`:193`。Run・Pipeline・Validate)、`takes_config`(`:197`。Convert 以外)、`takes_to`(`:198`)。許されない場所は `OptionNotAllowed`(subject は `-v`・`--config`・`--to`)。重複は `InvalidValue`(`config.len() > 1` `:212`)。`takes_run_id` は Run・Pipeline・Watch・Reset(`validate` と `convert` は不可 → `OptionNotAllowed`)。`reset --all` は、`--all` の新しいトークンと、`reset` だけの許可、`--all` と `--run-id` の併用を `InvalidValue`(または専用)で 68 にする。
- **位置引数**: `watch`・`reset` は 0 個(`wants_positional` `:224-227`、`allows_operand`)。余分は `UnexpectedArgument`(`unexpected_argument` `:259-264`)。
- **結果の型**: `CliCommand`(`crates/cli/src/types.rs:53-70`): `kind`、`job_id`、`pipeline_name`、`convert_input`、`convert_target`、`config_path: Option<OsString>`、`verbose`。`CliCommand::new`(`:72-84`)が全フィールドを列挙する。run ID と `--all` のフィールドを足す(「未指定」と「`""` で明示的になし」を区別するため `Option<OsString>`/`Option<String>` が要る)。
- **使い方の誤りの種類**: 8 つ(`UsageErrorKind` `types.rs:94-103`、`ALL` の配列 `:105-114` が網羅のテスト `args_grammar.rs:362` で固定)。表示は `report/messages.rs:105-126` の `usage_kind`。run ID の値の規則違反(文字・長さ 64・先頭の `.`・`..`)は `InvalidValue`(subject に値)で 68。新しい種類を足すなら、`ALL`・`usage_kind`・テストを揃える。
- **`Env`**: `crates/cli/src/deps.rs:37-42` — `trait Env { fn var(&self, name: &str) -> Option<String>; fn stdout_is_terminal(&self) -> bool; }`。本番は `SystemEnv`(`deps.rs:156-167`、`std::env::var(name).ok()`。UTF-8 でない値は `None`)。`Deps { runtime, sink, env, signals }`(`deps.rs:101-106`)に入り、`run_cli_with`(`lib.rs:129`)へ渡る。**`Ctx` は `env` を持たない**(`ctx.rs:41-46`: `runtime`・`signals`・`sink`・`reporter`)。`env` は `Reporter::new(deps.env, cmd.verbose)`(`lib.rs:~157`)でだけ使われ、`LC_ALL`・`LC_MESSAGES`・`LANG`(`report/mod.rs:348-362`)と `NO_COLOR`(`report/mod.rs:200`)を読む。run ID の環境変数を読むには、`Ctx` に `env: &dyn Env` を足す(または `run_cli_with` で先に値を決めて `CliCommand` へ入れる)。
- **テストの偽**: `FakeEnv { vars: HashMap<String,String> }` + `FakeEnv::with(&[(k,v)])`(`crates/cli/tests/common/mod.rs:37-60`)。`Harness`(`:420-466`)は `env: FakeEnv::default()` を持つ。環境変数は、`h.env.vars.insert(…)` で設定できる。本番バイナリのテスト(`bin_support/mod.rs:57` の `conductor(args)`)は、実際の環境を使う(`Command::env`/`env_remove` で制御)。
- **定義を読む順序(重要)**: 環境変数の名前は `[pipeline] run_id_env`(定義ファイル)にある。`run`・`pipeline` は `prepare` の定義の読み込み(`ctx.rs:90`/`:126-155`)の**後**でステートを読むので、順序は成り立つ。しかし `watch`(`aux_commands.rs:27-44`。**定義を読まない**)と `reset`(`:47-53`。**定義を読まない**。壊れた定義でも成功する。テスト `flow_aux.rs:157-163`、`runtime.log() == ["reset"]`)は、いま定義を読まない。`watch`/`reset` で定義の `run_id_env` を使うなら、定義を(軽く)読む必要があり、既存の「定義の誤りを迂回する」性質と衝突する。要件の側で、`watch`/`reset` が環境変数の名前をどう決めるか(既定名 `CONDUCTOR_RUN_ID` だけか、定義から読むか、読めないときは既定名か)を決める必要がある。

### (5) `reset` の現状の削除と、`reset --all` の設計材料
- **削除**: `reset_state`(`writer.rs:110-121`)。`fs::remove_file(StateLocation::of(def).path())`。`NotFound` は成功(`Ok(())`)、それ以外は `StateError::new(path, RemoveFailed, REMOVE_FAILED)`。読まず、検査しない。`run_reset`(`aux_commands.rs:47-53`)は `definition_path(cmd)` → `runtime.reset_state` → 失敗は `ctx.state_error(&e, StateContext::Plain)`(65)。成功は何も出さない(`flow_aux.rs:126-135`)。完了時の自動削除は `Ctx::reset`(`ctx.rs:183-188`)経由で同じ関数を呼ぶ。
- **ディレクトリの一覧に使う API**: 製品コードで `fs::read_dir` を使うのは **`remove_stale_temps`(`writer.rs:162-188`)だけ**。`fs::read_dir(dir)`(空のパスは `.`)→ `entries.flatten()` → `entry.file_name()` を `to_string_lossy()` して `starts_with(prefix)` → `entry.metadata()`、`meta.is_file()` を確かめ、24 時間(`STALE_AFTER` `:~22`)以上古いものを `fs::remove_file`。失敗は無視する。`reset --all` はこれに倣えるが、**エラーは無視せず `RemoveFailed` で返す**必要がある(`reset` は失敗を 65 にする)。
- **`remove_stale_temps` の照合**: 接頭辞 `.{state_name}.tmp.`(`writer.rs:~163`)。先頭が `.` の隠しファイルだけが対象で、状態ファイル(`x.state`)自体には一致しない。`reset --all` の照合(`<def>.state` と `<def>.<有効な run ID>.state` だけ)とは、別の規則で、混ぜない。
- **照合の曖昧さ(要注意)**: 定義 `x.toml` の `<def>.<run-id>.state` は `x.toml.<run-id>.state`。**別の定義 `x.toml.a`(run ID なし)のステートは `x.toml.a.state` で、これも `x.toml.<a>.state` の形に一致する**。run ID の規則(`A-Za-z0-9._-`、64 文字、先頭 `.` と `..` の禁止)を満たす `a` なので、`x.toml` の `reset --all` が別の定義のステートを消しうる。要件の「他の定義のステートは消さない」と矛盾しうる。run ID 付きファイルの `run_id` キーは中身で確かめられる(`reset` は読まない設計と衝突する)ので、設計で決める(接頭辞が定義ファイル名に完全一致し、間の run ID が有効であることに加え、`.state` で終わることだけで足りるか、中身を読んで確かめるか)。
- **一時ファイルは、`.` で始まる名前**(`.<state_name>.tmp.…`)なので、`<def>.<run-id>.state` の照合には当たらない(先頭が定義名でないため)。ただし、`--all` の照合を「先頭が `<def>.`」とすると、隠しファイルは当たらない。
- **`Runtime` trait に口を足す必要**: `reset_state` の戻りは `Result<(), StateError>`。`--all` は複数ファイルを扱うので、別のメソッド(例: `reset_all_states(definition_path)`)が要り、`SystemRuntime`・`ScriptedRuntime`・`MemRuntime` に足す。

### (6) 終了コードとメッセージ
- 定数(`crates/cli/src/exit.rs:19-32`): `EXIT_OK` 0、`EXIT_DEFINITION` 64、`EXIT_STATE` 65、`EXIT_REJECTED` 66、`EXIT_CONVERSION` 67、`EXIT_USAGE` 68、`EXIT_INTERNAL` 70。
- **64**: 定義の誤り。`Ctx::definition_errors`(`ctx.rs:157-160`)→ `DisplayEvent::DefinitionErrors`。`validate` も同じ(`run_validate` `aux_commands.rs:59-69`)。`run_id_env` の変数名の形の誤りは、定義の検証(core の `walk`)で返せば、`run`・`pipeline`・`validate` すべてで 64 になる。
- **65**: `Ctx::state_error`(`ctx.rs:163-166`)。`DisplayEvent::State { error, context }`(`report/mod.rs:~85`)。文は `state_line`(`report/mod.rs:399-418`)が、kind ごとの固定の文 + パス + 理由 + 文脈の補足(`StateContext::{Read, JobNotStarted, JobExecuted, Plain}` `:49-58`)で作る。`Read` の `Corrupt`/`Unreadable`/`TooLarge`/`VersionMismatch` には「`conductor reset` で初期化できます」の案内が付く(`RESET_HINT` `messages.rs:208-211`、`state_kind_suggests_reset` `:198-206`)。run ID の不一致(`corrupt`)は、この経路で 65 になる。ただし、案内が `conductor reset`(run ID なし)だけで、run ID 付きの `reset --run-id` を案内しない(文の見直しの対象)。食い違いは `ConsistencyMismatch`(`report/mod.rs:87`、`ctx.rs:107-111`、65)。
- **68**: `parse_args` の `UsageError` → `run_cli_with`(`lib.rs:133-137`)が `DisplayEvent::Usage` を出して `EXIT_USAGE`。表示は、固定の 1 行 + 空行 + `USAGE`(`report/mod.rs:~236-250`)。run ID の値の規則違反、`--all` と `--run-id` の併用、`validate`/`convert` への `--run-id`(`OptionNotAllowed`)は、すべて 68。`UnknownJob`(`run` の未知のジョブID)だけは `prepare` の中で 68(`ctx.rs:92-98`)。
- **70**: シグナルの準備の失敗(`ctx.rs:84-88`)とパニック。run ID には関係しない。
- **使い方**: `USAGE`(英語・日本語。`messages.rs:57-92`)の「Usage」の行と「Options」の行を、run ID と `--all` に合わせて直す。README の「使い方」(`README.md:75-83`)のコードブロックとオプション表(`:94-103`)、「ステートファイルの場所」(`:212-221`)も直す。
- **`watch` の表示**: `WatchTable`(`report/mod.rs:138`、`report/watch.rs`)はパイプライン名の 1 行 + 3 列の表。run ID の表示を足すなら、`PipelineState.run_id` を `watch.rs` が読む。

### (7) 固定しているテストとフィクスチャ(ファイル名)
- **ステートのファイル名 `conductor.toml.state`**: `crates/cli/tests/common/mod.rs:445-447`(`Harness::state_path`)、`crates/cli/tests/bin_support/mod.rs:146`(`state_path`)、`flow_run.rs:322`、`flow_aux.rs:356`(文字列で `"conductor.toml.state"` を含むことを確認)、`report_messages.rs:40,150,192,423`(`c.toml.state`)。`runtime/src/types.rs:202-218` の単体テスト、`runtime/tests/state_read.rs:44-47`(`paths`)・`state_write.rs`・`nfr_content.rs`・`nfr_robust.rs`・`flow_with_state.rs`。
- **ステートの TOML の形(`TOP_KEYS`、`[[job]]`)**: `runtime/src/state/reader.rs` の単体テスト(`VALID` の定数 `:~285-305`)、`runtime/tests/state_read.rs:24-42`(`VALID`)、`:147-211`(版の不一致・内容の矛盾・旧キー `jobs` は `corrupt`)、`state_write.rs`(書いた内容を読み戻す)、`runtime/tests/nfr_content.rs`(書く内容に入れてはならないもの)。**`run_id` の足し方で、`nfr_content` が「ステートに書くものを限る」性質を固定していないか、確認が要る**。
- **使い方の文言**: `cli/tests/report_messages.rs:709-720`(`Usage:` の前の固定の文)、`flow_aux.rs:284-330`(`--help` が `Usage:\n` で始まる。日本語は `使い方:\n`)、`binary_basic.rs:35-40`、`flow_validate.rs:153-161`。`USAGE` の本文そのものを逐語で固定しているテストは見つからなかった(先頭の語だけ)。
- **引数の文法**: `cli/tests/args_grammar.rs`(26 件。`v_は_run_と_pipeline_だけ`(`:142`)、`config_は…`(`:112`)、`重複する_config_と_to_は誤り`(`:207`)、`watch_と_reset_は位置引数を受け取らない`(`:88`)、`使い方の誤りの全種類が網羅されている`(`:362`)、`コマンドの名前`(`:348`)、UTF-8 でないパス(`:307`))。`exit_codes.rs`(定数の値)。`nfr_source.rs`(cli、ソースの性質を grep する可能性。未確認)。
- **定義の `[pipeline]` のキー**: `core/tests/definition_errors.rs`(`:155`、`:166` 未知のキー、`:361` キー名の切り詰め)、`definition_valid.rs`、`fixtures/{valid_pipeline,multiple_errors}.toml`、`core/tests/nfr_security.rs`。
- **`reset`/`watch` が定義を読まないこと**: `flow_aux.rs:157-163`(`reset_は壊れたステートと定義の誤りを迂回する`。`h.runtime.log() == ["reset"]`)。`watch` の定義に触れないテストは `flow_aux.rs` の watch 節(`:~80-120`)。
- **一時ファイルの名前の照合**: `runtime/src/state/writer.rs` の単体テスト(`:281` 以降、9 件)、`runtime/tests/state_write.rs:69`(`temp_files`)。

### Risks / follow-up
- **`watch`/`reset` と定義の読み込み**: 上の (4) の通り、`run_id_env` を定義から読むには、`watch`/`reset` が定義を読む必要があり、「壊れた定義でも `reset` は動く」という既存の性質(README:218、`flow_aux.rs:157-163`)と衝突する。要件で決める。
- **`reset --all` の照合の曖昧さ**: (5) の通り、別の定義(`x.toml.a`)の run ID なしのステートと、`x.toml` の run ID `a` のステートが、同じ名前になる。
- **`Runtime` trait の署名の変更**が、本番・2 種類の偽(`ScriptedRuntime`、`MemRuntime`)・公開関数・多数のテストに波及する。
- **`PipelineDefinition` と `PipelineState` へのフィールド追加**は、製品コードの構築が各 1 か所(`walk.rs:457`、`reader.rs` の `walk`)+ `initial_state` だが、テストの構築が多数(上記の一覧)。
- **`run_id` が `completed` のステート**: `read_state` は `completed == true` を空として返す(`reader.rs:121`)ので、ファイル名との一致検査の位置(前か後か)で、完了のまま残ったファイルの扱いが変わる。
- 前回の記録(`.github/`、CI、カバレッジ)は今回は未確認で、そのまま引き継ぐ。
