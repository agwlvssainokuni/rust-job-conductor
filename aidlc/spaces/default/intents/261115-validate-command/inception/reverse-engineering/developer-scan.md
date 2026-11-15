## Developer Code Scan Results

### Scan Coverage
- **Analyzed deeply**:
  - `crates/cli/src/`(args.rs、lib.rs、run.rs、pipeline.rs、aux_commands.rs、ctx.rs、deps.rs、exit.rs、types.rs、main.rs、report/mod.rs と report/messages.rs の使い方・表示まわり)
  - `crates/core/src/`(lib.rs、types.rs、definition/mod.rs、definition/walk.rs の `jobs` と名前照合の箇所、flow/decide.rs・next_step.rs・resume.rs の `jobs` の使用箇所)
  - `crates/runtime/src/`(lib.rs、split.rs の `check_command`、state/consistency.rs・record.rs の `jobs` の使用箇所)
  - `crates/converter/src/`(`"jobs"` を含む error.rs、yaml_write.rs、toml_write.rs の単体テスト)と `crates/converter/tests/` の `jobs` を含む箇所
  - `README.md`(使い方・終了コード・ステートの節)
  - `demo/`(README.md、`windows/conductor.toml`、`windows/run-demo.cmd` の構造)
  - `crates/*/Cargo.toml`、ワークスペースの `Cargo.toml`
- **Skimmed only**:
  - `crates/*/tests/`(ファイル数とテスト数のみ。cli 16 ファイル/192 件、core 6/122、runtime 10/165、converter 4/115。`args_grammar.rs`、`report_messages.rs`、`exit_codes.rs` は、引数の文法と使い方の文言に関わる箇所だけ確認)
  - `crates/cli/src/platform/`、`report/watch.rs`、`report/escape.rs`、`panic_hook.rs`
  - `crates/core/src/definition/{depth,intake,parse,version}.rs`、`flow/classify.rs`
  - `crates/runtime/src/runner.rs`、`platform/`、`state/{reader,writer}.rs`(`reader.rs` は、ステートの `job` キーの確認のみ)
  - `crates/converter/src/` の変換本体(intake、raw、resolve、scalar、toml_read、yaml_read)
  - `demo/run-demo.sh` と `demo/scripts/`、`demo/windows/scripts/`
- **Left out**: `.claude/`、`aidlc/` 作業領域、`.idea/`、`target/`、`local-docs/`、`.github/`(スナップショット範囲外)、.gitignore で除外されたもの

### Packages Found
- conductor-cli(`crates/cli`)— バイナリ+ライブラリ — Rust — 引数の解釈、流れの組み立て、シグナル、表示、終了コード
- conductor-core(`crates/core`)— ライブラリ — Rust — 定義の読み込み・検証、フローの判断
- conductor-runtime(`crates/runtime`)— ライブラリ — Rust — ジョブの起動、ステート、command の検査
- conductor-converter(`crates/converter`)— ライブラリ — Rust — YAML と TOML の変換
- demo(`demo/`)— スクリプト — sh と cmd — UNIX 版と Windows 版(`demo/windows/`)のデモ

### Build System
- **Type**: Cargo ワークスペース(`members = ["crates/*"]`、edition 2024、rust-version 1.85)
- **Config Files**: `Cargo.toml`、`crates/*/Cargo.toml`
- **Build Dependencies**: cli → core、runtime、converter / runtime → core / converter は独立

### APIs Discovered
- CLI — `crates/cli/src/args.rs` — 5 コマンド(run、pipeline、watch、reset、convert)+ `--help`・`--version`。`CommandKind` は 5 値
- 内部 trait — `crates/cli/src/deps.rs` — `Runtime`(7 メソッド)、`Sink`、`Env`、`Signals`
- ライブラリ — core 5 関数、runtime 11 関数、converter 2 関数

### Frameworks & Libraries
- lexopt 0.3.2(引数)、toml 1.2.1、saphyr-parser 0.3.4、thiserror 2.0.21、signal-hook 0.4.6(unix)、ctrlc 3.5.2(windows)、libc 0.2(unix)、tempfile 3.27.0(dev)

### Test Coverage
- **Test Directories**: `crates/{cli,core,runtime,converter}/tests/`、各 `src/` 内の単体テスト
- **Test Frameworks**: 標準の `cargo test`(統合テストと単体テスト)
- **Coverage Config**: 前回の記録では CI(cargo-llvm-cov、行 80%)。今回は `.github/` を開いていない

### Code Quality Indicators
- **Linting**: rustfmt、clippy(前回の記録による。今回は未確認)
- **CI/CD**: 今回は未確認(範囲外)
- **Documentation**: `README.md` は詳細。各 `lib.rs` に設計の表。`demo/README.md` に Windows 版の節あり

### Technical Debt Signals
- `PipelineDefinition.jobs` が、定義ファイルのキー `job` と食い違っている(`crates/core/src/types.rs:34`)。ステートファイルのキーは `job` で、旧 `[[jobs]]` はステートと定義の両方で拒否される(`README.md:168`、`crates/runtime/tests/state_read.rs:205`)
- 実行前の検査(`Ctx::prepare`)が、シグナルの準備・ステートの読み込みと一続きになっている(`crates/cli/src/ctx.rs:75-143`)。ステートに触れずに検査だけを行う入口がない
- `DisplayEvent` に、成功を表す(検査が通った)出力がない。既定では conductor 自身の出力はなく、`-v` のときだけ結果を標準エラーに出す
- 使い方の文言が 2 か所に重複している(`crates/cli/src/report/messages.rs:57-90` と `README.md:75-81`)

## Handoff Summary
- **Intent-relevant finding**:
  - **サブコマンドの解釈と振り分け**:
    - 字句分け: `crates/cli/src/args.rs:65-127`(`parse_args`)。
    - コマンド名の判定: `args.rs:181-188`(`"run"`、`"pipeline"`、`"watch"`、`"reset"`、`"convert"`。それ以外は `UnknownCommand`)。`CommandKind` の定義は `crates/cli/src/types.rs:26-32`(`name()` は `types.rs:34-44`)。
    - オプションの許可表: `args.rs:191-193`(`takes_verbose`: run と pipeline のみ。`takes_config`: convert 以外。`takes_to`: convert のみ)。`-v`・`--config`・`--to` は、2 回以上で `InvalidValue`(`args.rs:204-212`)。
    - 位置引数: `wants_positional` は `args.rs:218`(Watch・Reset 以外が 1 つ必須)。`CliCommand` へ詰める処理は `args.rs:232-243`(`job_id`、`pipeline_name`、`convert_input`)。`CliCommand` は `types.rs:51-65`。
    - 文法の表のドキュメントは `args.rs:21-34`。
    - 振り分け: `crates/cli/src/lib.rs:159-165`(`match cmd.kind`)。`Run` → `run::run_individual`、`Pipeline` → `pipeline::run_pipeline`、`Watch`・`Reset`・`Convert` → `aux_commands::run_*`。`Ctx` の組み立ては `lib.rs:153-158`。
    - `validate` の追加で触る場所: `CommandKind` に値を足す(`types.rs`)、`name()`、`args.rs` の判定(`:181-188`)・許可表(`:191-193`)・位置引数の判定(`:218`、`:232-243`)、`lib.rs:159-165`、`UsageErrorKind` の網羅(`types.rs:105` の `ALL`)は種類を増やさなければ変更不要。`crates/cli/tests/args_grammar.rs:323-327` が `CommandKind` と名前の対応を列挙している。
    - 使い方の文言: `crates/cli/src/report/messages.rs:57-90`(英語と日本語。`--config` の説明文は 69 行・85 行)。`README.md:75-81` にも同じ表がある。
  - **`run`・`pipeline` が実行の前に行うこと**: すべて `Ctx::prepare`(`crates/cli/src/ctx.rs:75-143`)にある。順序は次のとおり。
    1. シグナルの準備: `ctx.rs:81-88`(`self.signals.start`)。失敗は `SignalSetupFailed` を表示して 70(`EXIT_INTERNAL`)。`AlreadyRegistered` は成功と同じ扱い。
    2. 定義の読み込みと検証: `ctx.rs:90-94`(`definition_path(cmd)` は `ctx.rs:59-64`。`--config` があればそのパス、なければ `conductor.toml`)。`self.runtime.load_definition(&path, expected_pipeline)` → core の `load_definition`(`crates/core/src/definition/mod.rs:38-45`。Intake → Parse → Depth → VersionCheck → Walk)。失敗は `DefinitionErrors` を表示して 64(`definition_errors`、`ctx.rs:145-148`)。
    3. パイプライン名の照合: `pipeline` だけ、`expected_pipeline = Some(name)` を `load_definition` に渡す(`pipeline.rs:44`)。core の walk が、定義の `name` と比べ、不一致は `PipelineNameMismatch` の定義の誤りとして集める(`crates/core/src/definition/walk.rs:235-241`)。したがって不一致は 64。`run` と `validate` で名前なし(`None`)なら照合しない。
    4. 全ジョブの command の検査: `ctx.rs:96-112`。`definition.jobs.iter()` の各ジョブに `self.runtime.check_command(&job.command)`(runtime の `check_command`、`crates/runtime/src/split.rs:157`)。1 件でも失敗なら `CommandErrors { definition, errors }` を表示して 64。起動はしない。
    5. 不明なジョブID: `ctx.rs:114-120`。`require_job`(`run` のみ `Some(job_id)`)で `definition.job_index(job_id).is_none()` なら `UsageError(UnknownJob)` を表示して 68(`EXIT_USAGE`)。`pipeline` は `None`。
    6. ステートの読み込み: `ctx.rs:122-125`(`runtime.read_state`)。失敗は 65。
    7. ステートの照合: `ctx.rs:126-135`(`check_consistency`)。不一致は `ConsistencyMismatch` を表示して 65。
    - 上の 1・6・7 は、`validate` に含めない(ステートに触れず、シグナルも使わない)。2〜5 が `validate` に該当する。`prepare` は、2〜5 を切り出せる形にはなっていない(1 から 7 まで一続き)。
    - 呼び出し側: `run.rs:37`(`ctx.prepare(cmd, None, Some(job_id))`)、`pipeline.rs:44`(`ctx.prepare(cmd, Some(name), None)`)。
    - `validate` に必要な `Runtime` の既存メソッド: `load_definition` と `check_command` のみ(`crates/cli/src/deps.rs:76-82`)。`Runtime` trait に追加は不要。ステート系(`read_state`・`write_state`・`reset_state`)を呼ばないことを、偽の `Runtime` で確かめるテストを書ける。
  - **終了コードと文言**(`crates/cli/src/exit.rs:19-32`):
    - 定義の誤りと command の誤り: 64(`EXIT_DEFINITION`)。表示は `DefinitionErrors`(`report/mod.rs:243` の前、`definition_line`)と `CommandErrors`(`report/mod.rs:243-249`)。標準エラー。
    - 不明なジョブID: 68(`EXIT_USAGE`)。文言は `usage_kind(UnknownJob)`(`messages.rs:108-111`)。
    - 使い方の誤り(引数の文法): 68(`lib.rs:133-139`)。`--help`・`--version` は 0(`lib.rs:141-149`)。
    - 検査が通ったときに出す文言は、現在、存在しない。`DisplayEvent`(`report/mod.rs:75-140`)の `level()`(`:144-164`)は `Error`・`Verbose`・`Output` の 3 水準。成功を伝えるなら新しい `DisplayEvent` と文言(`messages.rs`、英語と日本語)が必要。標準出力(`Output`)に出すか、`-v` のときだけ標準エラー(`Verbose`)に出すかは、未決の設計事項。
    - `--config`(省略時 `conductor.toml`)と、`-v` の可否は、`args.rs:191-193` の許可表で決まる。`validate` に `-v` を許すかは未決。README の終了コードの表は `README.md:188-206`。
  - **`PipelineDefinition` のフィールド `jobs` を使っている箇所**(リネーム `jobs` → `job` の対象。ファイル:行):
    - 定義:
      - `crates/core/src/types.rs:34`(`pub jobs: Vec<JobDefinition>`)
      - `crates/core/src/types.rs:40`(`self.jobs.iter()`。`job_index`)
    - 構築:
      - `crates/core/src/definition/walk.rs:456-461`(`build_jobs` の結果を `jobs,` の省略記法で渡している。`let Some(jobs) = build_jobs(...)` の変数名は変えなくてもよい。フィールド名を変えるなら `jobs,` を `job: jobs,` にする)
    - ソース内の読み取り(製品コード):
      - `crates/core/src/flow/next_step.rs:86`(`let jobs = &definition.jobs;`)
      - `crates/core/src/flow/decide.rs:72`(`.jobs` は改行の後ろ)
      - `crates/core/src/flow/decide.rs:79`(`definition.jobs[..index]`)
      - `crates/core/src/flow/resume.rs:37`(`.jobs` は改行の後ろ)
      - `crates/runtime/src/state/consistency.rs:32`
      - `crates/runtime/src/state/record.rs:37`、`:39`(`.jobs` は改行の後ろ)、`:152`(`.jobs` は改行の後ろ)、`:156`
      - `crates/cli/src/ctx.rs:97`(`.jobs` は改行の後ろ)
      - `crates/cli/src/run.rs:103`(`definition.jobs[index]`)
      - `crates/cli/src/pipeline.rs:54`(`for job in &definition.jobs`)
    - ソース内の単体テスト(同じファイルの `mod tests`。フィールド名の初期化 `jobs:` と読み取り):
      - `crates/core/src/types.rs:329`(`jobs: vec![...]`)
      - `crates/core/src/flow/next_step.rs:176`(`jobs: vec![`)、`:205`、`:286`、`:313`、`:396`(`d.jobs[..]`)
      - `crates/core/src/flow/decide.rs:99`(`jobs: (0..n)`)
      - `crates/core/src/flow/resume.rs:63`(`jobs: (0..n)`)
      - `crates/runtime/src/state/consistency.rs:58`(`jobs: ids`)
      - `crates/runtime/src/state/record.rs:184`(`jobs: ids`)
    - 結合テスト(`tests/`):
      - `crates/core/tests/flow_sequences.rs:61`、`:107`、`:249`、`:262`、`:293`
      - `crates/core/tests/definition_valid.rs:41`、`:48`、`:55`、`:67`、`:79`、`:118`、`:138`、`:146`、`:154`
      - `crates/core/tests/nfr_performance.rs:57`
      - `crates/core/tests/nfr_security.rs:31`
      - `crates/runtime/tests/nfr_content.rs:102`(`jobs: vec![`)、`:117`(`d.jobs[0]`)
      - `crates/runtime/tests/nfr_performance.rs:37`(`jobs: (0..10)`)、`:77`(`&d.jobs`)
      - `crates/runtime/tests/flow_with_state.rs:71`(`jobs,` の省略記法)、`:154`(`.jobs` は改行の後ろ)、`:212`(`definition.jobs[0]`)
    - `crates/cli/tests/` は `PipelineDefinition.jobs` を使っていない(ヒットはローカル変数 `jobs`)。
    - **別物なので変更しない `jobs`**:
      - ローカル変数、引数名(`walk.rs:323-349` の `jobs: &[RawJob]`、`build_jobs`、`cli/tests/nfr_perf.rs`、`bin_support/mod.rs:49-52`、`definition_errors.rs:31-136`、`runtime/tests/state_write.rs:116-121`、`toml_depth.rs:55`、`convert_roundtrip.rs` のローカル変数)
      - `Kind::NoJobs`、`"no jobs are defined"`(`messages.rs:140`)
      - 旧キーを拒否するテスト(`cli/tests/flow_run.rs:251`、`runtime/src/state/reader.rs:336`、`runtime/tests/state_read.rs:205`、`state_write.rs:116`・`:121` の `[[jobs]]`)と、`README.md:168` の旧キーの注意書き
  - **converter の見本のキー `"jobs"` → `"job"`**(単体テストの見本。いずれも製品コードの動作は変わらない):
    - `crates/converter/src/yaml_write.rs:261`(`"jobs"`)と `:275`(期待の文字列 `jobs:`)
    - `crates/converter/src/toml_write.rs:128`(`"jobs"`)と `:144`(`t["jobs"]`)
    - `crates/converter/src/error.rs:284`(`PathSeg::Key("jobs")`)、`:291`(`"jobs[2].transitions[0]"`)、`:23-25`(説明コメントの `jobs`、`jobs[2]...`)
    - `crates/converter/tests/read_resolve.rs:294`(入力 YAML の `jobs:`)と `:296`(`"jobs[1].note"`)
    - `crates/converter/tests/convert_roundtrip.rs` は、すでにキー `"job"` を使っている(`:66`、`:113`、`:152`)。変更不要。
    - 定義ファイルのキーは `[[job]]`。`crates/converter` は core に依存せず、`jobs` を意味づけて扱わない(任意のキーとして通す)。
  - **README とデモ**:
    - `README.md:73-81`(使い方のコードブロック)、`:83-91`(コマンドの表)、`:93-100`(オプションの表。`--config` の対象コマンド列挙は `:95`)、`:103-111`(例)、`:188-206`(終了コードの表。64・68 の説明)に `validate` の追記先がある。
    - `README.md:168` の `[[jobs]]` の注意書きはそのまま(ファイルのキーは変わらない)。
    - デモ: `demo/README.md`(UNIX 版の場面 1〜6 と、`## Windows 版`)、`demo/run-demo.sh`、`demo/windows/run-demo.cmd`。デモは `validate` を使っていない。`demo/windows/conductor.toml` は `[[job]]` を使い、`jobs` を含まない。`crates/cli/tests/windows_demo.rs` は `cfg(windows)` 限定で、`demo/windows` を一時ディレクトリへ写して場面 1〜6 を確かめる。
- **Risks / follow-up**:
  - `validate` は `prepare` の 1(シグナル)と 6・7(ステート)を呼ばないこと。`Ctx::prepare` を共通化するときは、2〜5 を切り出す。シグナルの準備を `validate` で行わない場合、`validate` の実行中の Ctrl+C は既定の動作で終わる(`aux_commands.rs` の `watch`・`reset`・`convert` と同じ方針。`aux_commands.rs:17` の注記)。
  - `run`・`pipeline` の順序は、定義(64)→ command(64)→ ジョブID(68)の順。`validate <pipeline-name>` はこのうち、名前の照合(64)を受け持つ。`validate` で「ジョブID」を扱う必要はない。
  - 名前なしの `validate` は、`load_definition(&path, None)` を呼べば照合なしで検査できる(`Runtime::load_definition` の第 2 引数が `Option<&str>` のため、変更不要)。
  - 引数の位置引数 `<pipeline-name>` は任意(0 か 1 個)になる。現在の `args.rs:218-230` は、コマンドごとに 1 つ必須か 0 個かの 2 通りだけなので、「0 か 1」を表す分岐が新たに必要。
  - リネームは、フィールドの定義(`types.rs:34`)の変更後、コンパイルエラーで漏れを検出できる。`jobs: vec![...]` と省略記法 `jobs,` の初期化(`walk.rs:461`、`flow_with_state.rs:71`)、改行の後ろの `.jobs`(grep の行頭では見つからないもの)に注意する。
  - テストの総数: cli 192、core 122、runtime 165、converter 115(単体テストと統合テストの `#[test]` の合計)。`cli` の `args_grammar.rs` と `report_messages.rs` が、文法と使い方の文言を固定している(文言を変えると、これらのテストの期待値の更新が必要)。
  - `PipelineState`・ステートファイルのキー `job`(`reader.rs:194`)は変わらない。リネームはフィールド名だけで、ステートの互換に影響しない。
