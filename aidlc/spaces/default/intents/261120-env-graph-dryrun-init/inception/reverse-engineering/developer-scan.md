## Developer Code Scan Results

対象: rust-job-conductor(Rust ワークスペース、4 クレート、版 0.8.0)。スナップショットのパスは `./`、head_commit は `1a77179273aed7763b89a69fed1b383c9b4ba8b2`。スキャンは読み取りのみで、テスト・ビルド・プログラムは実行していない。

### Scan Coverage
- **Analyzed deeply**:
  - `Cargo.toml`
  - `crates/core/src/`(lib.rs、types.rs、error.rs、run_id.rs、definition/{mod,intake,parse,depth,version,duration,walk}.rs、flow/{mod,classify,decide,next_step,resume,retry}.rs)
  - `crates/runtime/src/`(lib.rs、runner.rs、split.rs、types.rs、error.rs、wait.rs、platform/、state/ の構造と公開関数)
  - `crates/cli/src/`(lib.rs、main.rs、args.rs、types.rs、ctx.rs、run.rs、pipeline.rs、aux_commands.rs、retry.rs、exit.rs、deps.rs、report/ の構造と文言表、platform/mod.rs、panic_hook.rs)
  - `crates/converter/src/`(lib.rs、types.rs、error.rs の種類、resolve.rs の方針)
  - `crates/*/Cargo.toml`
  - `.github/workflows/`(ci.yml、release.yml、audit.yml)
  - `crates/core/tests/`、`crates/runtime/tests/`、`crates/cli/tests/`、`crates/converter/tests/` の配置と共通補助(cli の common/mod.rs、nfr_source.rs の先頭)
- **Skimmed only**:
  - `crates/*/tests/*.rs` の各テスト本体(ファイル名と行数、先頭の説明だけ)
  - `crates/*/src/` の `#[cfg(test)]` の単体テストの本体
  - `crates/converter/src/{yaml_read,yaml_write,toml_read,toml_write,scalar,raw,intake}.rs`(モジュールの先頭の説明だけ)
  - `crates/runtime/src/state/{reader,writer,reset_all}.rs` の内部、`crates/runtime/src/platform/{unix,windows}.rs`、`crates/cli/src/platform/{unix,windows}.rs`、`crates/cli/src/report/{escape,watch}.rs`
  - `crates/*/src/bin/*-test-helper.rs`(モード表だけ)
  - `demo/`(定義の例、スクリプト、Windows 版)
  - `README.md`、`docs/spec/`(91 ファイル、7 版。`rust/00-README.md` の範囲・目次・用語だけ。他の言語版 csharp・go・language-agnostic・powershell・python・typescript は開いていない)
  - `.idea/`(追跡されている IDE の設定。中身は開いていない)
- **Left out**:
  - `target/`(.gitignore で除外。ビルド出力)
  - `local-docs/`(.gitignore で除外)
  - `.claude/`、`.kiro/`、`.codex/`、`.cursor/`、`.opencode/`、`.aidlc/`、`aidlc/`(AI-DLC 自身の導入物。対象外)

### Packages Found
- conductor-core — ライブラリ(crates/core) — Rust — 定義ファイルの読み込み・検証(definition/)と、ジョブの結果の分類・次の動き・再開位置・リトライの判断(flow/)。入出力に触れるのは definition/intake.rs のみ。run_id.rs(run ID の検査)、types.rs(共有型)、error.rs(エラーの種類と表示)。
- conductor-runtime — ライブラリ(crates/runtime) — Rust — ジョブの直接起動と待ち(runner.rs)、command の分割(split.rs)、シグナルの転送(platform/)、リトライの待機(wait.rs)、ステートの読み書き・照合・記録・初期化(state/)。
- conductor-converter — ライブラリ(crates/converter) — Rust — YAML と TOML のあいだの定義ファイルの変換。conductor-core に依存しない。キーと値の汎用の木(`Value`)で変換するため、定義のキーの意味は知らない。
- conductor-cli — バイナリ + ライブラリ(crates/cli、実行ファイル名 `conductor`) — Rust — 引数の解釈(args.rs)、流れの組み立て(ctx.rs、run.rs、pipeline.rs、aux_commands.rs、retry.rs)、表示(report/)、終了コード(exit.rs)、シグナルの監視(platform/)、差し替え口(deps.rs)。
- conductor-test-helper / conductor-cli-test-helper — テスト専用バイナリ(runtime、cli の src/bin/) — Rust — 引数・環境変数・終了コード・シグナルを出力する補助。配布しない。

### Build System
- **Type**: Cargo ワークスペース(`members = ["crates/*"]`、resolver 3、edition 2024、rust-version 1.100、license Apache-2.0)
- **Config Files**: `Cargo.toml`、`crates/*/Cargo.toml`、`Cargo.lock`。`rustfmt.toml`・`clippy.toml`・`rust-toolchain*` は無い(既定の設定)。release プロファイル: opt-level 3、lto thin、codegen-units 1、strip。
- **Build Dependencies**: cli → core、runtime、converter。runtime → core。converter は独立(core に依存しない)。core は外部のクレートだけ。

### APIs Discovered
- CLI の文法 — `crates/cli/src/args.rs` — 6 サブコマンド(`run`、`pipeline`、`watch`、`reset`、`validate`、`convert`)と、オプション `-v`、`--config`、`--run-id`、`--all`、`--remove-on-complete`、`--to`、`-h`、`-V`。`parse_args` は `lexopt` で字句に分け、`build_command` が許可・重複・併用不可を検査する純粋な関数。
- conductor-core の公開関数 — `lib.rs` — `load_definition`、`parse_definition`、`decide_before_run`、`classify_and_decide`、`resume_point_on_interrupt`、`retry_decision`、`peek_run_id_env`、`is_valid_env_name`。
- conductor-runtime の公開関数 — `lib.rs` — `run_job`、`wait_for`、`check_command`、`read_state`、`read_state_raw`、`write_state`、`check_consistency`、`reset_state`、`reset_all_states`、`initial_state`、`apply_outcome`、`apply_skip`、`apply_interrupt`。
- conductor-converter の公開関数 — `lib.rs` — `convert`、`convert_text`。
- 差し替え口 — `crates/cli/src/deps.rs` — trait `Runtime`(11 メソッド)、`Sink`、`Env`、`Signals`。本番は `SystemRuntime`、`StdSink`、`SystemEnv`、`SystemSignals`。
- 定義ファイル(TOML、format_version = 1) — `crates/core/src/definition/walk.rs` — キー: ルート `format_version`、`pipeline`、`job`。`[pipeline]`: `name`、`description`、`run_id_env`、`remove_on_complete`。`[[job]]`: `id`、`command`、`warning_exit_codes`、`success_exit_codes`、`transitions`、`retry`。`[[job.transitions]]`: `on`、`next`。`[job.retry]`: `exit_codes`、`max`、`interval`、`multiplier`、`max_interval`。許可キーの定数は `ROOT_KEYS`、`PIPELINE_KEYS`、`JOB_KEYS`、`RETRY_KEYS`、`TRANSITION_KEYS`(walk.rs の先頭)。
- ステートファイル(TOML) — `crates/runtime/src/state/reader.rs` — 上位キー 10 個(`TOP_KEYS`)、ジョブのキー 4 個(`JOB_KEYS`)。場所は `<定義ファイル名>[.<run-id>].state`。環境変数・command・出力は書かない(NFR4.5)。

### Frameworks & Libraries
- toml — 1.2.1 — 定義・ステートの TOML 読み取り(core は `parse`、`std`。`Spanned`/`DeTable` で位置を保つ。runtime と converter は `display`、`serde` も)
- thiserror — 2.0.21 — エラー型(core、runtime、converter)
- saphyr-parser — 0.3.4 — YAML のイベント列の読み取り(converter)
- lexopt — 0.3.2 — 引数の字句分け(cli)
- signal-hook — 0.4.6(default-features 無効、`iterator`) — UNIX のシグナル監視(cli、cfg(unix))
- ctrlc — 3.5.2 — Windows の Ctrl+C(cli、cfg(windows))
- libc — 0.2 — UNIX の `kill`(runtime、cfg(unix)。`unsafe` はここ 1 か所)
- tempfile — 3.27.0 — テスト用(dev-dependencies)

### Test Coverage
- **Test Directories**: `crates/core/tests/`(10 ファイル + fixtures)、`crates/runtime/tests/`(13 ファイル + common)、`crates/cli/tests/`(21 ファイル + common、bin_support、WINDOWS_CTRL_C_CHECKLIST.md)、`crates/converter/tests/`(5 ファイル + fixtures + common)。各 `src/*.rs` の内側にも `#[cfg(test)]` の単体テスト(日本語のテスト名)。
- **Test Frameworks**: 標準の `cargo test`(#[test] と統合テスト)。cli の統合テストは、偽の `Runtime`/`Sink`/`Env`/`Signals`(tests/common/mod.rs)で流れを検査する `flow_*.rs` と、実バイナリを起動する `binary_*.rs`、`signals_real.rs`、`unix_demo.rs`、`windows_demo.rs` に分かれる。`nfr_*.rs` は非機能の検査(ソース検索、性能、堅牢性、セキュリティ)。`msrv_consistency.rs` は Cargo.toml と README の MSRV 一致、`docs_retry.rs` は README の記述の確認。
- **Coverage Config**: CI の `coverage` ジョブで `cargo llvm-cov --workspace --locked --fail-under-lines 80`(テスト補助バイナリは除外)。手元の設定ファイルは無い。

### Code Quality Indicators
- **Linting**: `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings`(CI の `lint` ジョブ)。設定ファイルは無く既定。
- **CI/CD**: `.github/workflows/ci.yml`(lint、build-check[ubuntu・windows、MSRV 1.100、C ビルド依存と -sys クレートの検査]、test[ubuntu・macos・windows]、msrv-test、coverage、docs-check[README の必須語句]、startup-bench[警告のみ])、`release.yml`(タグ `v*`、main 上のタグと版の一致を確認、musl Linux・aarch64/x86_64 macOS・Windows の 4 対象をビルド、スモークテスト、SHA256SUMS、下書きリリース)、`audit.yml`(cargo audit、継続、警告のみ)。
- **Documentation**: README.md(利用者向け。インストール、使い方、定義の書き方、リトライ、ステートの場所、run ID、終了コード、at-least-once、シグナル、convert、ロケール)。各 `src/lib.rs` と各モジュールの先頭に、日本語の詳しいモジュール文書(契約・呼び出し順序・守る性質)。`docs/spec/`: 7 版(csharp、go、language-agnostic、powershell、python、rust、typescript)× 13 文書(00-README から 12-rebuild-guide)の仕様書。`docs/spec/rust/` が現在の実装(v0.8.0)の忠実な仕様。`demo/`: Unix 版と Windows 版の定義の例とスクリプト。

### Technical Debt Signals
- 定義の型 `JobDefinition`・`PipelineDefinition`(crates/core/src/types.rs)は構造体リテラルで直接作られているテストが多数ある(core、runtime、cli の `#[cfg(test)]` と tests の `common/mod.rs`)。フィールドを足す変更(環境変数の追加など)は、リテラルの全箇所の修正を要する。
- `crates/cli/src/report/messages.rs` の `definition_kind` は `DefinitionErrorKind` を網羅して match する(英語)。日本語は `Display`(core 側の `#[error]`)をそのまま使うため、新しい種類は core の `error.rs`(`#[error]`、`code()`、`ALL` の配列の長さ 30)と cli の `messages.rs` の 2 か所と、網羅の検査(tests の `report_messages.rs`)を同時に直す必要がある。
- `crates/cli/src/ctx.rs` の `prepare`、`run.rs` の `run_individual`、`pipeline.rs` の `run_pipeline` は、似た前処理・記録の流れを持つ(重複)。`--dry-run` を足すと、どこで止めるかが 3 か所に及ぶ。
- ジョブ起動の `run_job`(runtime)は `JobDefinition` だけを受け取り、環境変数を `Command` に渡す口が無い(`Command::new` のあと `args` と `configure_spawn` のみ。環境は親のまま継承)。
- `peek_run_id_env`(core/run_id.rs)は定義をゆるく読む別経路で、`walk.rs` の検証と重複する知識(`[pipeline].run_id_env`)を持つ。
- `.idea/` が追跡されている(IDE の設定。`.gitignore` に無い)。
- 使い方の表が、args.rs の文書コメント、messages.rs の USAGE(英日)、README の 3 か所に重複して書かれている。
- unsafe は runtime/src/platform/unix.rs の `libc::kill` 1 か所だけ(runtime は `#![deny(unsafe_code)]` で platform だけ許可)。cli は `#![forbid(unsafe_code)]`。

## Handoff Summary
- **Intent-relevant finding**: この intent(環境変数と .env、`conductor graph`、`--dry-run`、`conductor init`)が触れる場所は次のとおり。
  - 定義のキー: 読み取りと検証は `crates/core/src/definition/walk.rs`。許可キーの定数 `PIPELINE_KEYS`、`JOB_KEYS`、`RETRY_KEYS`(ファイルの先頭)。環境変数の名前の検査 `is_valid_env_name` は `crates/core/src/run_id.rs`。型は `crates/core/src/types.rs` の `PipelineDefinition`、`JobDefinition`。
  - エラーの種類: `DefinitionErrorKind`(30 種、`crates/core/src/error.rs`。`#[error]`、`code()`、`ALL: [_; 30]`)。英語の文は `crates/cli/src/report/messages.rs` の `definition_kind`、日本語は core の `Display`。使い方の誤りは `UsageErrorKind`(8 種、`crates/cli/src/types.rs`)と `usage_kind`。
  - ジョブの起動: `crates/runtime/src/runner.rs` の `run_job(job, signal)`。`Command::new(&line.program)`、`command.args(&line.args)`、`platform::configure_spawn`(UNIX は `process_group(0)`)のあと `spawn()`。環境は親を継承するだけで、環境変数を足す口は無い。command の分割は `crates/runtime/src/split.rs`(変数を展開しない)。
  - CLI: `crates/cli/src/args.rs` の `parse_args` / `build_command`(許可表と重複検査)、`crates/cli/src/types.rs` の `CommandKind`・`CliCommand`、`crates/cli/src/lib.rs` の `run_cli_with` の match でコマンドを振り分け。新しいサブコマンド(`graph`、`init`)や `--dry-run` は、これらと `messages.rs` の `USAGE`(英日)、README の使い方の表に及ぶ。
  - 流れ: `crates/cli/src/ctx.rs` の `prepare`(検証、run ID、ステートの読み込みと照合)と `check_definition`(`load_definition` と `check_command`)、`run.rs`、`pipeline.rs`。`validate` は `aux_commands.rs` の `run_validate` が `check_definition` だけを呼ぶ(ジョブは実行しない。`--dry-run` の参考になる)。
  - 純粋な流れの関数(`decide_before_run`、`classify_and_decide`、`resume_point_on_interrupt`、`retry_decision`)は `crates/core/src/flow/`。ステート: `crates/runtime/src/state/{record,reader,writer,consistency,reset_all}.rs`。
  - converter(`crates/converter`)は、定義の意味を知らない汎用の木の変換なので、新しいキー(環境変数の表など)が文字列・表・配列で書かれる限り、そのまま通る。core に依存しない。
- **Risks / follow-up**:
  - 新しい `DefinitionErrorKind` を足すと、core の `ALL` の長さ(30)、`code()`、`#[error]`、cli の英語の `definition_kind`、`report_messages.rs` などの網羅のテストが連動して変わる。
  - 定義の構造体にフィールドを足すと、多数のテストのリテラルが壊れる。
  - ステートファイルは環境変数を書かないという性質(NFR4.5、`nfr_content.rs`)と、エラーの表示に入力の値を含めないという性質(NFR4.6)があり、`.env` の値の扱いはこれを守る必要がある。
  - クレート間の純粋/入出力の境界(core は intake 以外でファイルに触れない)、`unsafe` の位置の制約(`nfr_source.rs`)、cfg の位置の制約(`platform` の外に `cfg(unix)`/`cfg(windows)` を置かない)は、テストで強制されている。`.env` の読み取りや `graph` の出力は、この境界を意識して置く場所を決める必要がある。
  - README の語句(SHA-256、Gatekeeper、終了コードの表、ステートファイルの場所、at-least-once、シグナル、convert)は CI の `docs-check` で必須。MSRV の README 表記は `msrv_consistency.rs` で検査される。
  - 既存の `docs/spec/rust/` は v0.8.0 の記述で、この intent の変更後は更新が要る。
