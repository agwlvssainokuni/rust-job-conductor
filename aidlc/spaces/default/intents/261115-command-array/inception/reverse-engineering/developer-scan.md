## Developer Code Scan Results

### Scan Coverage
- **Analyzed deeply**:
  - `crates/core/src/types.rs`, `crates/core/src/lib.rs`
  - `crates/core/src/definition/` (`mod.rs`, `walk.rs`, `intake.rs`, `depth.rs`。`parse.rs`・`version.rs` は構成のみ)
  - `crates/runtime/src/split.rs`, `runner.rs`, `types.rs`, `lib.rs`, `error.rs`
  - `crates/runtime/src/state/record.rs`, `writer.rs`(先頭部)
  - `crates/converter/src/lib.rs`, `types.rs`, `raw.rs`, `toml_write.rs`
  - `crates/cli/src/lib.rs`, `ctx.rs`, `run.rs`, `pipeline.rs`, `aux_commands.rs`, `deps.rs`(trait 部), `exit.rs`, `main.rs`, `report/mod.rs`(DisplayEvent と command 関連), `report/watch.rs`, `report/escape.rs`
  - `Cargo.toml`(ルートと 4 クレート), `Cargo.lock`, `.github/workflows/ci.yml`, `README.md`, `demo/conductor.toml`, `.gitignore`
- **Skimmed only**:
  - `crates/core/src/flow/`(`classify.rs`, `decide.rs`, `next_step.rs`, `resume.rs`: `command` を使わないことを grep で確認)
  - `crates/core/src/error.rs`, `crates/core/src/definition/parse.rs`, `version.rs`
  - `crates/runtime/src/state/reader.rs`, `consistency.rs`, `platform/`
  - `crates/converter/src/` の `error.rs`, `intake.rs`, `resolve.rs`, `scalar.rs`, `yaml_read.rs`, `toml_read.rs`, `yaml_write.rs`(ヘッダの文書と関数名のみ)
  - `crates/cli/src/args.rs`, `types.rs`, `panic_hook.rs`, `platform/`, `report/messages.rs`, `bin/conductor-cli-test-helper.rs`
  - `crates/runtime/src/bin/conductor-test-helper.rs`
  - `crates/*/tests/`(ファイル名・件数・`command` の出現数だけ。内容は一部のみ)
  - `.github/workflows/release.yml`, `audit.yml`, `demo/`(`run-demo.sh`, `scripts/`)
- **Left out**: `target/`(`.gitignore` 対象。開かず)、`local-docs/`(`.gitignore` 対象かつ指示で対象外)、`.idea/`、`.claude/`、`aidlc/`(指示で対象外)。

### Packages Found
- conductor-core — ライブラリ(Cargo クレート) — Rust — 定義の読み込み・検証(TOML)とフロー判断(純粋、入出力は intake のみ)
- conductor-runtime — ライブラリ — Rust — command の分割(split)、ジョブの起動と待ち、シグナル転送、ステート(TOML)の読み書き・照合
- conductor-converter — ライブラリ — Rust — YAML と TOML の相互変換(スキーマを知らない汎用の木。core に依存しない)
- conductor-cli — ライブラリ + バイナリ `conductor`(+ テスト用 `conductor-cli-test-helper`) — Rust — 引数解析、流れの組み立て、シグナル、表示、終了コード

### Build System
- **Type**: Cargo ワークスペース(`members = ["crates/*"]`, resolver 3, edition 2024, rust-version 1.85)
- **Config Files**: `Cargo.toml`, `crates/*/Cargo.toml`, `Cargo.lock`(`--locked` で使用)。`rustfmt.toml`・`clippy.toml` は無し
- **Build Dependencies**: cli → core, runtime, converter / runtime → core / converter → (core に依存しない) / core → (内部依存なし)

### APIs Discovered
- ライブラリ API(Rust 公開関数) — `crates/core/src/lib.rs` — `load_definition`, `parse_definition`, `classify_and_decide`, `decide_before_run`, `resume_point_on_interrupt`(+ 型一式)
- ライブラリ API — `crates/runtime/src/lib.rs` — `run_job`, `check_command`, `read_state`, `write_state`, `reset_state`, `check_consistency`, `initial_state`, `apply_outcome`/`apply_skip`/`apply_interrupt`
- ライブラリ API — `crates/converter/src/lib.rs` — `convert`, `convert_text`
- CLI — `crates/cli/src/args.rs` — 5 サブコマンド(`run`, `pipeline`, `watch`, `reset`, `convert`)と `--help`/`--version`/`-v`/`--config`
- 定義ファイル(TOML)の書式 — `crates/core/src/definition/walk.rs` — `format_version`, `[pipeline]`, `[[job]]`(`id`, `command`, `warning_exit_codes`, `success_exit_codes`, `transitions`)

### Frameworks & Libraries
- toml — 1.2.1 — 定義・ステートの読み書き(core は parse のみ、runtime/converter は serde・display も)
- thiserror — 2.0.21 — エラー型
- saphyr-parser — 0.3.4 — YAML のイベント読み取り(converter)
- lexopt — 0.3.2 — 引数解析(cli)
- signal-hook — 0.4.6 — UNIX のシグナル監視(cli)
- ctrlc — 3.5.2 — Windows の Ctrl+C(cli)
- libc — 0.2 — UNIX のプロセスグループ・シグナル送信(runtime)
- tempfile — 3.27.0 — テスト(dev)

### Test Coverage
- **Test Directories**: `crates/{core,runtime,converter,cli}/tests/`(統合テスト・NFR テスト・fixtures)、各 `src/` 内の `#[cfg(test)]`
- **Test Frameworks**: 標準の `cargo test`(`#[test]` 約 567 件: core 115, runtime 154, converter 113, cli 185。テスト名は日本語・英語混在)。バイナリ実行のテストあり(`bin_support`、テスト用ヘルパー)
- **Coverage Config**: CI の `cargo llvm-cov --fail-under-lines 80`(設定ファイルは無し)

### Code Quality Indicators
- **Linting**: `cargo fmt --check` と `cargo clippy --workspace --all-targets -D warnings`(CI の lint ジョブ)。設定ファイルは既定
- **CI/CD**: `.github/workflows/ci.yml`(lint, build-check[ubuntu/windows, MSRV 1.85], test[3 OS], msrv-test, coverage, docs-check, startup-bench)、`release.yml`(タグで 4 ターゲットのバイナリ)、`audit.yml`(cargo audit、警告のみ)
- **Documentation**: `README.md`(利用者向け、日本語、CI の docs-check が必須語を検査)、`demo/`。各モジュールに日本語の `//!` 文書。`unsafe` は runtime の `platform/unix.rs` の 1 か所だけ(テストが検査)

### Technical Debt Signals
- `command` は `JobDefinition.command: String` と `JobExecution.command: String` の 2 か所で型が固定され、分割は runtime の `split`(`run_job` と `check_command`)に集中している(`crates/core/src/types.rs:50,101`、`crates/runtime/src/runner.rs:90,115`)。
- `ctx.rs` の起動失敗の表示は、`check_command` が失敗したときに `job.command.split_whitespace().next()` で名前を作る(`crates/cli/src/ctx.rs:204-213`)。command が文字列でなくなると、この経路の前提が崩れる。
- `escape_display` 相当の関数が 4 クレートに重複している(`crates/cli/src/report/escape.rs` の文書に明記)。
- TODO/FIXME/HACK は 0 件。`#[allow]` はテストの共通モジュールの `dead_code` と runtime の `unsafe_code`(1 か所)のみ。

## Handoff Summary
- **Intent-relevant finding**: 変更の中心は 4 点。(1) `crates/core/src/definition/walk.rs` の `Walker::string(table, "command", ...)`(L254)と `RawJob.command: Option<String>`、空判定 `c.trim().is_empty()`(L256)、`build_jobs`(L447)。配列の取り出しと要素ごとの型エラー報告は、同ファイルの `exit_codes`(L115-140)が先行する型である。(2) `crates/core/src/types.rs` の `JobDefinition.command`(L50)と `JobExecution.command`(L101)。`JobDefinition {` を組み立てる箇所は core の `flow/*`・`types.rs`、runtime の `runner.rs`・`state/record.rs`・`state/consistency.rs`、runtime テストの `common/mod.rs`・`flow_with_state.rs` に及ぶ。(3) `crates/runtime/src/split.rs` の `split`/`check_command`(`CommandLine { program, args }` を返す)と `runner.rs` の `run_job`(L115 `split(&job.command)` → `Command::new(program).args(args)`)。`CommandError` は `UnclosedQuote`/`Empty` の 2 種。(4) cli: `ctx.rs` の `prepare`(L96-103、全ジョブを `check_command(&job.command)` で事前検査)と `report_execution`(L202-216、起動失敗時にプログラム名を表示)、`deps.rs` の `Runtime::check_command(&self, command: &str)` trait(差し替え口。テストの偽物も影響を受ける)。
- **Risks / follow-up**:
  - ステートファイル(`runtime/src/state/`)は command を保存しない(`PipelineState`/`JobState` に command の欄は無い)。`watch` の表(`report/watch.rs`)も command を出さない。`-v` の表示も command を出さず、command の文字列が表に出るのは起動失敗時のプログラム名だけ(`DisplayEvent::StartFailure`、`escape_name` で 64 文字に切る)。したがってステートの書式と版(`format_version = 1`)は変更の対象外になりうる。
  - converter はスキーマを知らない汎用の木(`Value::Str/Array/Map`)で変換するため、`command = ["a", "b"]` の配列は、コード変更なしで往復できる見込みだが、未確認(テストは未実行)。TOML の書き出しはライブラリ任せで、整列したキー順になる。YAML の文字列の引用符(`yaml_write.rs`)の扱いは要確認。
  - 深さの上限 32(`depth.rs`)は配列を 1 段数える。`[[job]]` 配下の `command` 配列で 1 段増えるだけで、通常は問題にならない。
  - 空配列、空文字列の要素、先頭要素が空の場合の扱いは、現行の `Empty`(コマンド名がない)との整合を決める必要がある。要素に空白・引用符を含む場合は分割・展開をしない(文字列の分割規則との差)。
  - テストの影響が大きい: `command` を含むテストファイルは約 30、`definition_errors.rs`(37 件)・`binary_basic.rs`(31 件)・`nfr_perf.rs`・`runner_basic.rs` が多い。fixtures(`core/tests/fixtures/*.toml`, `converter/tests/fixtures/*`)と `demo/conductor.toml`、README の「command の書き方」節も文書の更新対象。
  - `README.md` は CI の docs-check が必須語を検査する(`command` の語は必須語に含まれない)。カバレッジ 80% 未満は CI で失敗する。
  - 実行ファイルのビルド・テストは実行していない(読み取りのみ)。
