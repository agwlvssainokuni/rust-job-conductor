# 開発者のコードスキャン(rust-job-conductor、全体の再スキャン)

- 対象: ワークスペースの根(`./`)。対象のコミット: `1f88e59b2453ed80ea4d53b4275c85dcdb310550`(2026-11-25、版 0.9.9)
- 深さ: Standard。読むだけで、テスト・ビルド・プログラムは実行していない。
- intent: small-feature-batch(v0.10.0 の 7 つの小さな機能)。スキャンは、この範囲に限らず、リポジトリ全体を対象にした。

## Developer Code Scan Results

### Scan Coverage
- **Analyzed deeply**:
  - `Cargo.toml`、`Cargo.lock`
  - `crates/core/`(`Cargo.toml`、`src/` の全モジュール: `lib.rs`、`types.rs`、`error.rs`、`run_id.rs`、`consistency.rs`、`definition/`(`walk/` を含む)、`envfile/`、`flow/`、`graph/`、`plan/`)
  - `crates/runtime/`(`Cargo.toml`、`src/` の全モジュール: `runner.rs`、`split.rs`、`envfile.rs`、`file_create.rs`、`temp_file.rs`、`bounded_read.rs`、`wait.rs`、`types.rs`、`error.rs`、`platform/`、`state/`、`bin/`)
  - `crates/cli/`(`Cargo.toml`、`src/` の全モジュール: `lib.rs`、`main.rs`、`args.rs`、`types.rs`、`ctx.rs`、`step.rs`、`run.rs`、`pipeline.rs`、`dry_run.rs`、`aux_commands.rs`、`run_id.rs`、`deps.rs`、`exit.rs`、`panic_hook.rs`、`platform/`、`report/`、`bin/`)
  - `crates/converter/`(`Cargo.toml`、`src/` の全モジュールの文書と公開の口)
  - `.github/workflows/`(`ci.yml`、`release.yml`、`audit.yml`)
  - `README.md`(目次、使い方、終了コード、保証、シグナル)
  - `.gitignore`(プロジェクトの部分)、`.gitattributes`
  - 各クレートの `tests/` と `src/tests/`(ファイルの構成と、`nfr_source.rs` の検査の中身。個々のテストの本体は抜き取りで確認)
- **Skimmed only**:
  - `docs/spec/`(7 言語版 × 13 文書。ファイル名と構成だけ)
  - `demo/`(`README.md`、`conductor.toml` の冒頭、ファイルの一覧)
  - `.idea/`(IDE のプロジェクト設定。追跡されているファイルの一覧だけ)
  - `crates/core/tests/fixtures/`(配置の参照の出力。大きなファイルは開かず、名前と行数だけ)
  - `LICENSE`
- **Left out**:
  - `target/`(Rust のビルド出力。`.gitignore` で無視)
  - `local-docs/`(`.gitignore` で無視される個人の参照文書)
  - `.idea/workspace.xml` など、`.idea/.gitignore` が無視する IDE の個人ファイル
  - AI-DLC の導入物(`.claude/`、`aidlc/`、`aidlc.settings.local.json`、`.gitignore` の AI-DLC の区画)

### Packages Found
- `conductor-core` — ライブラリ — Rust — 定義の読み込み・検証、純粋な判断(分類・次の動き・再開位置・リトライ)、ステートの更新、`.env` の解析と層の統合、図の構造と配置、`--dry-run` の計画
- `conductor-runtime` — ライブラリ(+ テスト専用の補助バイナリ `conductor-test-helper`) — Rust — ジョブの直接起動とシグナルの転送、リトライの待機、ステートの読み書き、`.env` の読み取り、`init` のファイルの作成
- `conductor-converter` — ライブラリ — Rust — 定義ファイルの YAML と TOML の相互変換(core に依存しない)
- `conductor-cli` — ライブラリ + 実行ファイル `conductor`(+ テスト専用の補助バイナリ `conductor-cli-test-helper`) — Rust — 引数の解釈、流れの組み立て、シグナル、表示(英日)、終了コード

### Build System
- **Type**: Cargo ワークスペース(`members = ["crates/*"]`、`resolver = "3"`、`edition = "2024"`、`rust-version = "1.100"`)
- **Config Files**: `Cargo.toml`(根)、`crates/*/Cargo.toml`、`Cargo.lock`(追跡。CI は常に `--locked`)。`[profile.release]` は `opt-level = 3`、`lto = "thin"`、`codegen-units = 1`、`strip = true`。`rustfmt.toml`・`clippy.toml` は無い(既定)。
- **Build Dependencies**: `conductor-cli` → {`conductor-core`、`conductor-runtime`、`conductor-converter`}、`conductor-runtime` → `conductor-core`。`conductor-converter` と `conductor-core` は、ほかのクレートに依存しない。循環なし。

### APIs Discovered
- CLI のコマンド — `crates/cli/src/args.rs`、`crates/cli/src/types.rs` — 8 コマンド(`run`、`pipeline`、`watch`、`reset`、`validate`、`convert`、`graph`、`init`)+ `--help`/`--version`。オプション 11 種(`-v`、`--config`、`--to`、`--run-id`、`--all`、`--remove-on-complete`、`--dry-run`、`--force`、`--assume`、`--format`、`--state`)
- 定義ファイルの書式(TOML、`format_version = 1`) — `crates/core/src/definition/walk/` — 根のキー 3、`[pipeline]` のキー 6、`[[job]]` のキー 8、`[job.retry]` のキー 5、遷移のキー 2
- ステートファイルの書式(TOML、`format_version = 1`) — `crates/runtime/src/state/` — 最上位のキー 10、ジョブのキー 4。知らないキーは拒否(65)
- `.env` の書式 — `crates/core/src/envfile/parse.rs` — `KEY=VALUE`、引用符、コメント。誤りは 13 種
- ライブラリの公開の口 — core(`load_definition` など 10 関数 + `envfile`・`graph`・`plan`・`run_id` の関数)、runtime(11 関数)、converter(2 関数)、cli(`run_cli`、`run_cli_with`、`install_panic_hook` と差し替え口の trait 4 つ)
- ジョブに渡す自動の環境変数 — `crates/core/src/envfile/merge.rs` — `CONDUCTOR_PIPELINE`、`CONDUCTOR_JOB_ID`、`CONDUCTOR_RUN_ID`(run ID があるとき)、`CONDUCTOR_ATTEMPT`
- 終了コード — `crates/cli/src/exit.rs` — 0、ジョブの値、64〜70、127、128 + 番号
- ネットワークの API は無い(`nfr_source.rs` が `std::net`、`TcpStream`、`reqwest`、`hyper` などの不在を検査)

### Frameworks & Libraries
- `toml` — 1.2.1(機能を絞る) — 定義・ステートの読み書き(core、runtime、converter)
- `thiserror` — 2.0.21 — エラー型(core、runtime、converter)
- `lexopt` — 0.3.2 — 引数の字句の分割(cli)
- `saphyr-parser` — 0.3.4 — YAML のイベントの読み取り(converter)
- `signal-hook` — 0.4.6(`iterator` のみ。UNIX) — シグナルの監視(cli)
- `ctrlc` — 3.5.2(Windows) — Ctrl+C の監視(cli)
- `libc` — 0.2(ロックは 0.2.191。UNIX) — `kill` の 1 か所(runtime)
- `tempfile` — 3.27.0(開発時のみ) — テストの一時ディレクトリ
- 標準ライブラリ以外の非同期ランタイム、ログ、シリアライズの枠組み(serde の derive の直接使用)は無い

### Test Coverage
- **Test Directories**: `crates/core/tests/`(統合テスト 33 ファイル、`common/`、`graph_common/`、`fixtures/`)、`crates/runtime/tests/`(14 ファイル、`common/`)、`crates/cli/tests/`(25 ファイル、`common/`、`bin_support/`、`signal_support/`、`fixtures/`、`WINDOWS_CTRL_C_CHECKLIST.md`)、`crates/converter/tests/`(2 ファイル、`fixtures/`)、`crates/cli/src/tests/`(単体テスト 8 ファイル + `common/`)、`crates/converter/src/tests/`(単体テスト 3 ファイル)、各モジュールの `#[cfg(test)] mod tests`
- **Test Frameworks**: Rust 標準の `#[test]`(`#[test]` の数: core 381、runtime 200、cli 473、converter 116)。外部のテストの枠組みは無い。テスト名は日本語。
- **Coverage Config**: あり(CI の `coverage` ジョブ。`cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80`)

### Code Quality Indicators
- **Linting**: `rustfmt`(`cargo fmt --all -- --check`)、`clippy`(`cargo clippy --workspace --all-targets --locked -- -D warnings`)。設定ファイルなし(既定)。CI の `lint` ジョブ
- **CI/CD**: `.github/workflows/ci.yml`(`lint`、`build-check`(1.100.0、ubuntu と windows。C のビルドと `-sys` の検査)、`test`(ubuntu・macos・windows)、`msrv-test`、`coverage`、`docs-check`、`startup-bench`(警告のみ))、`release.yml`(`verify-tag` → `checks`(ci.yml の再利用)→ `build`(4 対象)→ `draft-release`)、`audit.yml`(`cargo audit`。PR と毎週月曜。警告のみ)
- **Documentation**: `README.md`(772 行、日本語。`docs-check` が必須の語句を検査)、`demo/README.md`、`docs/spec/`(7 言語版の再構築用の仕様)、各モジュールの冒頭の `//!` 文書(日本語。規則の番号 BR/NFR/FR と対応)

### Technical Debt Signals
- 本体のコードに `TODO`/`FIXME`/`HACK` は無い(`XXXX` の一致は `init_template.rs:91` の `\uXXXX` の説明で、負債ではない)。
- `#[allow]` は `crates/runtime/src/platform/unix.rs:20` の `#![allow(unsafe_code)]` の 1 つだけ。`unsafe` のブロックは `unix.rs:68` の `libc::kill` 1 か所。
- 500 行を超える本体のファイル(テストを含む): `cli/src/report/messages.rs`(754)、`cli/src/report/mod.rs`(659)、`converter/src/resolve.rs`(630)、`cli/src/args.rs`(626)、`core/src/definition/walk/mod.rs`(621)、`core/src/types.rs`(583)、`core/src/flow/record.rs`(575)、`runtime/src/state/reader.rs`(572)。責務は単一で、多くは表(文言、文法)やテストが長さの原因。
- `team.md` の Testing Posture のテストファイルの数(core 10、runtime 13、cli 21、converter 5)は古い。現状は core 33、runtime 14、cli 25、converter 2(統合テスト。converter の単体テストは `src/tests/` へ移っている)。
- ステートの読み取りが知らないキーを拒否する(`state/reader.rs` の `TOP_KEYS`・`JOB_KEYS`)。キーを足すと、古い版の `conductor` が新しいステートを読めない(65)。版(`FORMAT_VERSION = 1`)の扱いの判断が要る。
- `JobDefinition` などの公開の構造体はフィールドが公開で、テストがリテラルで組み立てる(`core/src/test_support.rs` などの補助で一部を吸収)。フィールドを足すと、テストの変更が広がる。
- 使い方の表が、`args.rs` の `//!`、`report/messages.rs` の USAGE、`README.md` の 3 か所にある(`src/tests/docs.rs` が一部の一致を検査)。

## Handoff Summary
- **Intent-relevant finding**:
  - `[pipeline.retry]`: リトライの設定はジョブだけが持つ(`core/src/types.rs` の `JobDefinition.retry: Option<RetryPolicy>`、`core/src/definition/walk/mod.rs` の `PIPELINE_KEYS` に `retry` は無い)。判断は `core/src/flow/retry.rs` の `retry_decision(job, …)` が `job.retry` だけを見る。パイプラインの既定は、読み取り(`walk`)で各ジョブへ合成するか、型に持たせるかの選択になる。前例は `remove_on_complete`(定義の既定と `--remove-on-complete` の合成。`cli/src/ctx.rs` の `check_run_inputs`)。
  - `graph --format dot`: 形式は `cli/src/types.rs` の `GraphFormat { Text, Mermaid }` と `cli/src/args.rs:567` の値の検査、`cli/src/aux_commands.rs` の `run_graph` の分岐。整形の前例は `cli/src/report/graph_mermaid.rs`(`FlowGraph` から純粋に作り、ラベルを escape)。core の `graph` は形式に依存しない。
  - 任意の `.env`: 読み取りは `runtime/src/envfile.rs` の `load_env_files` → `read_one` で、無ければ `EnvFileErrorKind::FileMissing` として全体を失敗にする(`run`・`pipeline`・`validate`・`graph`・`--dry-run` が 64 で止まる)。`env_file` の型は `Vec<String>`(`core/src/types.rs`)で、任意かどうかの情報を持たない。
  - `unset_env`: 起動は `runtime/src/runner.rs` の `run_job(line, env: &[(&str, &str)], signal)` で、`Command::env` で足すだけ(`env_remove` は無い)。統合は `core/src/envfile/merge.rs` の `merge_env` が「足す」項目だけを返す(`EffectiveEnv`)。`--dry-run` の名前の要約(`merge_env_names`)も同じ並べ方に従う。
  - `log_dir`: 起動は標準入出力を引き継ぐ(`runner.rs` の文書)。`Stdio` の指定は本体のどこにも無い。ファイルに触れるのは runtime だけで、cli は `deps.rs` の `Runtime` trait を通す。リトライのたびの出力の扱い(追記か、試行ごとか)と、`.env` と同じパスの解決(定義ファイルのディレクトリが基準)が論点。
  - 開始・終了の時刻と `watch` の所要時間: `PipelineState`・`JobState`(`core/src/types.rs`)に時刻は無い。時刻を取る処理も本体に無い(`SystemTime` は一時ファイルの名前と古さの判定だけ)。ステートの書き出しは `runtime/src/state/writer.rs` の `serialize`/`job_table`、読み取りは `state/reader.rs` の厳格なキーの表。core は純粋(時刻に触れない)ので、時刻は cli か runtime で取り、`apply_*` に渡す形になる。
  - `watch` の連続表示: `cli/src/aux_commands.rs` の `run_watch` は 1 回だけ表示して終わる。シグナルの準備をしない(既定の動作で終わる)。表は `cli/src/report/watch.rs` の `render_table`(純粋。色は UNIX 系の端末で `NO_COLOR` が無いときだけ)。
- **Risks / follow-up**:
  - ステートのキーを足すと、旧版がそのステートを拒否する(`UNKNOWN_KEY` で 65)。`format_version` を上げるか、読み取りの寛容さを変えるかを設計で決める必要がある。
  - 守る性質(コードで確認済み): `unsafe` は runtime の `platform/unix.rs` の 1 か所(`libc::kill`)。runtime は `#![deny(unsafe_code)]`、core・converter・cli は `#![forbid(unsafe_code)]`。本体のコードの `cfg(unix)`/`cfg(windows)` は `runtime/src/platform/` と `cli/src/platform/` の中だけ(ほかは `#[cfg(test)]` のテストの中)。ネットワークのライブラリは無い。エラーの表示は固定の文で、入力の値を含めない。ステートに command・環境変数・出力を書かない。環境変数の値を持つ型の `Debug` は伏せ字。
  - 新しい依存は加えない方針(`project.md`)。DOT の整形、ログのファイル、時刻の書式は標準ライブラリで書く必要がある。時刻の書式(RFC 3339 など)を標準ライブラリだけで作るには、暦の計算を自前で持つことになる。
  - ログのファイルを runtime で作ると、`nfr_source.rs`(OS 固有のコードの位置)と、`.env` と同じ上限・パスの扱いの一貫性を保つ必要がある。
  - 連続の `watch` は、端末の消去の制御文字と Windows の扱い(色を使わない方針。`report/mod.rs`)、中断の扱い(今は既定の動作で終わる)を決める必要がある。
