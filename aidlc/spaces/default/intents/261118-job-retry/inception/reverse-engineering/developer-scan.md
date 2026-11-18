## Developer Code Scan Results

### Scan Coverage
- **Analyzed deeply**:
  - `Cargo.toml`, `Cargo.lock`
  - `crates/core/src/`(types.rs、error.rs、lib.rs、run_id.rs、definition/ 全 6 ファイル、flow/ 全 5 ファイル)
  - `crates/runtime/src/`(types.rs、error.rs、lib.rs、runner.rs、split.rs、platform/ 全 3 ファイル、state/ 全 6 ファイル)
  - `crates/cli/src/`(lib.rs、main.rs、types.rs、ctx.rs、deps.rs、exit.rs、run.rs、pipeline.rs、aux_commands.rs、panic_hook.rs、platform/ 全 3 ファイル、report/ の mod.rs・watch.rs・escape.rs・messages.rs の冒頭、args.rs の文法部)
  - `crates/converter/src/`(lib.rs、types.rs、raw.rs、resolve.rs の方針部、toml_write.rs、yaml_write.rs の冒頭)
  - `crates/*/Cargo.toml`
  - `.github/workflows/ci.yml`
  - `README.md`(定義ファイルの書き方、終了コード、at-least-once、シグナルの節)
  - `demo/conductor.toml`
  - テストの共通補助: `crates/cli/tests/common/mod.rs`、`crates/cli/tests/bin_support/mod.rs`、`crates/runtime/tests/common/mod.rs`、`crates/converter/tests/common/mod.rs`、`crates/cli/tests/nfr_source.rs`、`crates/cli/tests/msrv_consistency.rs`、`crates/runtime/tests/nfr_source.rs`
- **Skimmed only**:
  - `crates/converter/src/`(yaml_read.rs、scalar.rs、toml_read.rs、intake.rs、error.rs)
  - `crates/cli/src/args.rs`(関数の本体)、`crates/cli/src/report/messages.rs`(個々の文言)
  - `crates/core/src/definition/depth.rs`
  - `crates/runtime/src/state/reader.rs` の走査の細部、`crates/runtime/src/state/reset_all.rs` の後半
  - `crates/*/tests/`(ファイル名・テスト数・共通補助のみ。個々のテスト本体は読んでいない)
  - `crates/core/tests/fixtures/`、`crates/converter/tests/fixtures/`
  - `.github/workflows/release.yml`、`.github/workflows/audit.yml`(冒頭のみ)
  - `demo/`(README、scripts、windows/)
  - `crates/cli/tests/WINDOWS_CTRL_C_CHECKLIST.md`
  - `README.md` のインストールとクイックスタートの節
  - `local-docs/requirements.md`(題名のみ。ユーザー管理の参考資料)
- **Left out**:
  - `target/`(ビルド成果物。`.gitignore` 済み。開いていない)
  - `.git/`
  - `.idea/`(IDE の設定)
  - `LICENSE`
  - `aidlc.settings.local.json`(AI-DLC の設定)
  - AI-DLC 自身の導入物(`.claude/`、`aidlc/` など。指示により対象外)

### Packages Found
- conductor-core — ライブラリ — Rust — 定義の読み込み・検証(DefinitionLoader)と、純粋な判断(Flow: 起動前の判断、分類、次の動き、再開位置)。IO はファイル取り込み(`definition/intake.rs`)だけ
- conductor-runtime — ライブラリ(+ テスト専用バイナリ `conductor-test-helper`) — Rust — ジョブの直接起動と待ち(`run_job`)、シグナルの転送、ステートの読み書き・照合・記録・初期化。OS 固有部は `platform/`
- conductor-converter — ライブラリ — Rust — YAML と TOML の相互変換。スキーマに依存しない汎用の木(`Value`)で動く。conductor-core に依存しない
- conductor-cli — ライブラリ + バイナリ `conductor`(+ テスト専用バイナリ `conductor-cli-test-helper`) — Rust — 引数の解釈、`run`・`pipeline`・`watch`・`reset`・`validate`・`convert` の流れ、シグナルの監視、表示(英語・日本語)、終了コード

### Build System
- **Type**: Cargo ワークスペース(`resolver = "3"`、`edition = "2024"`、`rust-version = "1.100"`、ライセンス Apache-2.0)
- **Config Files**: `Cargo.toml`(ルート、`[profile.release]` は `opt-level = 3`、`lto = "thin"`、`codegen-units = 1`、`strip = true`)、`crates/*/Cargo.toml`、`Cargo.lock`。`rustfmt.toml`・`clippy.toml`・`rust-toolchain` は存在しない
- **Build Dependencies**: conductor-cli → conductor-core、conductor-runtime、conductor-converter。conductor-runtime → conductor-core。conductor-core と conductor-converter は、ほかの crate に依存しない

### APIs Discovered
- CLI(コマンドライン)— `crates/cli/src/args.rs` — 6 コマンド(`run`、`pipeline`、`watch`、`reset`、`validate`、`convert`)と `--help`・`--version`
- 定義ファイル(`conductor.toml`、`format_version = 1`)— `crates/core/src/definition/walk.rs` — `[pipeline]` 4 キー、`[[job]]` 5 キー、`[[job.transitions]]` 2 キー
- ステートファイル(TOML、`format_version = 1`)— `crates/runtime/src/state/{reader,writer}.rs` — 最上位 10 キー、ジョブ 4 キー(未知のキーは `corrupt`)
- ライブラリ API(契約 C2: core)— `load_definition`、`parse_definition`、`decide_before_run`、`classify_and_decide`、`resume_point_on_interrupt` ほか
- ライブラリ API(契約 C3: runtime)— `run_job`、`check_command`、`read_state`、`read_state_raw`、`write_state`、`check_consistency`、`reset_state`、`reset_all_states`、`initial_state`、`apply_outcome`、`apply_skip`、`apply_interrupt`
- ライブラリ API(契約 C4: converter)— `convert`、`convert_text`
- 差し替え口(trait)— `crates/cli/src/deps.rs` — `Runtime`、`Sink`、`Env`、`Signals`(4 つ)

### Frameworks & Libraries
- toml — 1.2.1 — 定義・ステートの読み取り(`DeTable`、位置つき)と書き出し(core は `parse`・`std`、runtime と converter は `display`・`serde` つき)
- thiserror — 2.0.21 — エラー型の Display
- saphyr-parser — 0.3.4 — YAML のイベント列(converter)
- lexopt — 0.3.2 — 引数の字句分け(cli)
- signal-hook — 0.4.6(UNIX のみ)、ctrlc — 3.5.2(Windows のみ)— シグナルの監視(cli)
- libc — 0.2(UNIX のみ、runtime)— `kill` の 1 か所
- tempfile — 3.27.0(dev)— テスト

### Test Coverage
- **Test Directories**: `crates/{core,runtime,converter,cli}/tests/`、各 `src/` 内の `#[cfg(test)]` モジュール
- **Test Frameworks**: 標準の `cargo test`(`#[test]`)。テスト名は日本語。外部のモックのライブラリはなく、cli は `Runtime`・`Sink`・`Env`・`Signals` の偽実装(`crates/cli/tests/common/mod.rs` の `ScriptedRuntime` など)と、テスト専用バイナリ(`conductor-test-helper`、`conductor-cli-test-helper`)で動かす
- **Coverage Config**: CI の `coverage` ジョブが `cargo llvm-cov --fail-under-lines 80`(テスト用バイナリは除外)。ローカルの設定ファイルはない
- 件数の目安: `tests/` 配下の `#[test]` 約 480、`src/` 内の `#[cfg(test)]` 約 200

### Code Quality Indicators
- **Linting**: `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings`(CI の `lint`)。設定ファイルは置かず既定値。`#![forbid(unsafe_code)]`(cli)、`#![deny(unsafe_code)]`(runtime。`platform/unix.rs` の `kill` 1 か所だけ許可)
- **CI/CD**: `.github/workflows/ci.yml`(lint、build-check(ubuntu・windows、MSRV 1.100)、test(ubuntu・macos・windows)、msrv-test、coverage、docs-check、startup-bench)、`release.yml`(タグ `v*` で 3 OS のリリース成果物を作り、下書きで公開)、`audit.yml`(cargo audit)
- **Documentation**: README.md(440 行、日本語。使い方・定義の書き方・終了コード・at-least-once・シグナル・convert)。ソースは日本語の doc コメントが密で、要件 ID(BR・NFR・FR)と対応づけてある。CI の `docs-check` が README の必須の見出しを検査する。`msrv_consistency` テストが、MSRV の記載の一致を検査する

### Technical Debt Signals
- 表示のエスケープ関数が、core(`sanitize`)・runtime・converter・cli(`report/escape.rs`)の 4 か所に重複(`crates/cli/src/report/escape.rs` の doc コメントが認めており、`tests/escape_table.rs` で揃えを確かめる)
- 自動の再試行・時間の上限・ファイルのロックは、設計上持たない(`conductor-runtime/src/lib.rs`、`crates/cli/src/pipeline.rs` の doc コメント)。`classify_and_decide` の doc コメントに「自動の再試行はしない」とある(`crates/core/src/flow/next_step.rs`)
- cli の `Deps` に、時刻・待ち(スリープ)の差し替え口がない。待ちは `crates/runtime/src/runner.rs` の `wait_loop` の内部(`std::thread::sleep` を引数で渡す形)に閉じている
- `crates/cli/src/lib.rs` などの doc コメントが、`entities.md`・`logical-components` など、リポジトリにない設計文書を参照している(設計文書は `local-docs/`(git 管理外)にあるとみられる)

## Handoff Summary
- **Intent-relevant finding**: 再試行を足す入口は 5 層にまたがる。(1) 定義: ジョブのキー一覧が `crates/core/src/definition/walk.rs:35-41` の `JOB_KEYS` と `RawJob`・`job()`・`build_jobs`、型が `crates/core/src/types.rs:81-92` の `JobDefinition`。未知のキーは `unknownKey` で拒否されるため、新しいキーは必ずここに足す。誤りの種類は `crates/core/src/error.rs` の `DefinitionErrorKind`(20 種。`ALL`・`code`・表示文と、cli の `report/messages.rs` の `definition_kind` に日本語・英語の対応がある)。(2) 判断: 終了コードの分類は `crates/core/src/flow/classify.rs` の `classify`(成功 → 警告 → 0 の特例 → エラーの順)、エラーは `crates/core/src/flow/next_step.rs` の `stop()`(`resume_job_id` を自分にして停止)。再試行の対象にするのは、`terminated_by_signal`・`start_failure`・`interrupted` ではなく、「エラーと分類された、ジョブ自身の終了コード」だけにするのが自然(中断は再試行してはならない)。(3) 実行の流れ: `crates/cli/src/run.rs:1026-1034` と `crates/cli/src/pipeline.rs:107-115` が、`run_job` → `classify_and_decide` → `report_execution` → `apply_outcome` → `write_state(Durable)` を 1 回ずつ行う(同じ形が 2 か所に複製)。再試行のループは、ここ(または共通化した関数)に入る。(4) 待ち: 間隔を空ける仕組みと、待ちの最中の中断(`InterruptSignal`)の扱いは、いまどこにもない。`Deps` に時刻・スリープの差し替え口がなく、テストは `ScriptedRuntime`(`crates/cli/tests/common/mod.rs`)で `run_job` だけを台本にしている。(5) 表示: 進捗の表示は `DisplayEvent`(`crates/cli/src/report/mod.rs`)に種類を足し、`messages.rs` に英語・日本語の対を足す。`-v` のときだけ出す水準(`Level::Verbose`)と、常に出す水準(`Level::Error`)がある。要件の「進捗を表示する」を、`-v` なしでも見せるかは、水準の選択として決める必要がある。converter は定義のスキーマを知らない(汎用の木)ため、新しいキーのための変更は原則不要だが、表(table)・浮動小数の往復は既存のテストの範囲を確認する必要がある。
- **Risks / follow-up**:
  - 既定の挙動の保持: 再試行を設定しないジョブは、現行どおり 1 回で結果を確定する(後方互換)。`PipelineDefinition`・`JobDefinition` は `PartialEq, Eq` を導出しているため、浮動小数を足すと `Eq` が使えなくなる(整数のミリ秒・秒にすれば避けられる)。`JobDefinition` を構造体リテラルで組み立てているテスト・補助が多数あり(`crates/*/tests/`、`crates/runtime/tests/common/mod.rs` の `job()`、`core/src/types.rs` のテスト)、フィールドを足すと全部の更新が要る。
  - at-least-once: 再試行中にステートへ何を残すかで、再開の意味が変わる。ステートのジョブのキーは 4 つに固定(`reader.rs` の `JOB_KEYS`、未知のキーは `corrupt`)で、足せば `format_version` の扱いと、旧版の conductor による読み取りの拒否を決める必要がある。最後の試行の結果だけを記録するなら、ステートの形は変えずにすむ。
  - 中断との相互作用: 待ちの最中にシグナルが立ったら、次の試行を起動せず、128 + 番号で終わる(`started = false` の中断と同じ扱い、BR2.4 が近い)。待ち自体はシグナルを転送しない(ジョブがいないため)。`pipeline` ではジョブのあいだの中断(`pipeline.rs` の `session.signal.signal_number()`)の前後との順序を決める必要がある。
  - 検査の順序とテストの不変条件: cli の `nfr_source` テストが、`src/` に `unsafe`・ネットワーク・`cfg(unix)`/`cfg(windows)`(platform の外)がないことを検査する。runtime の `nfr_source` は、`src/` にシェルの呼び出し・パニックしうる書き方がないことを検査する。待ちの実装は、この制約の中に収める。
  - 表示の安全: ジョブID など入力由来の文字列は、必ず `escape_name`/`escape_path` を通す。
  - README の `docs-check`(必須の見出し)と MSRV 一致テストがあるため、README を更新するとき、これらの見出しを消さない。
  - 一部のテストは本物のバイナリ・シグナル・待ちの時間を使う(`signals_real.rs`、`binary_signal.rs`、`runner_signal.rs`。30 秒の上限つき)。再試行の間隔を実時間で待つテストは遅くなるため、間隔の差し替え口が要る。
