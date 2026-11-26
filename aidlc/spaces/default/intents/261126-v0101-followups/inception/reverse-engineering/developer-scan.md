# 開発者のコードの走査(v0101-followups)

対象: `rust-job-conductor`(head `4ebd8e7a0d73445aa5bbd18dbb3773fe3eb64617`)。走査の幅は Full rescan(`./`)。コードは読むだけで、テスト・ビルド・プログラムは実行していない。

## Developer Code Scan Results

### Scan Coverage
- **Analyzed deeply**:
  - `Cargo.toml`、`Cargo.lock`(直接・推移的な依存の版)、`crates/core/Cargo.toml`、`crates/runtime/Cargo.toml`、`crates/cli/Cargo.toml`、`crates/converter/Cargo.toml`
  - `.github/workflows/ci.yml`、`.github/workflows/release.yml`、`.github/workflows/audit.yml`
  - `README.md`(全 953 行。特に「使い方」「watch で表示し直しを続ける」「ジョブの出力をファイルに残す」「ステートファイルの場所」「終了コードの表」「シグナルの扱い」)
  - `demo/run-demo.sh`、`demo/README.md`
  - `crates/core/src/`(`lib.rs`、`types.rs`、`time.rs`、`error.rs`、`flow/`(`activity.rs`、`attempt.rs`、`decide.rs`、`record.rs`、`mod.rs`)、`plan/simulate.rs`、`definition/walk/mod.rs` のキーの表と `log_dir`、`envfile/mod.rs` の誤りの種類)
  - `crates/runtime/src/`(`lib.rs`、`runner.rs`、`log.rs`、`clock.rs`、`wait.rs`、`platform/`、`state/reader.rs` のキーの表と試行の検査、`state/writer.rs` の書き込みの手順、`error.rs`、`types.rs`)
  - `crates/cli/src/`(`lib.rs`、`main.rs`、`deps.rs`、`types.rs`、`ctx.rs` の構成、`step.rs`、`run.rs`、`pipeline.rs`、`dry_run.rs`、`aux_commands.rs`、`run_id.rs`、`exit.rs`、`platform/`、`report/mod.rs` の `DisplayEvent` と `FramePrefix`、`report/watch.rs`、`report/graph_dot.rs` の冒頭、`args.rs` の `OPTIONS`)
  - `crates/converter/src/lib.rs`、`crates/converter/src/error.rs` の誤りの種類
  - `crates/core/tests/plan_simulate_perf.rs`、`crates/core/tests/definition_perf.rs` と `graph_lookup_perf.rs` の上限の決め方、`crates/core/tests/common/mod.rs` の `median`
  - `crates/runtime/tests/log_file.rs` の名前の表、`crates/cli/tests/flow_aux.rs` の U7 のテストの一覧、`crates/cli/tests/binary_basic.rs` と `binary_log.rs` の時刻を挟むテスト、`crates/cli/tests/unix_demo.rs` の環境の設定
- **Skimmed only**:
  - `crates/core/src/definition/`(`intake.rs`、`parse.rs`、`depth.rs`、`version.rs`、`duration.rs`、`walk/retry.rs`、`walk/env.rs`)、`crates/core/src/envfile/parse.rs`・`merge.rs`、`crates/core/src/graph/`、`crates/core/src/consistency.rs`、`crates/core/src/run_id.rs`、`crates/core/src/flow/classify.rs`・`next_step.rs`・`resume.rs`・`retry.rs`
  - `crates/runtime/src/split.rs`、`envfile.rs`、`bounded_read.rs`、`file_create.rs`、`temp_file.rs`、`state/reset_all.rs`、`bin/conductor-test-helper.rs`
  - `crates/cli/src/args.rs`(表の後の解釈の本体)、`report/messages.rs`、`report/graph_text.rs`、`report/graph_mermaid.rs`、`report/plan_text.rs`、`report/init_template.rs`、`panic_hook.rs`、`bin/conductor-cli-test-helper.rs`、`src/tests/`
  - `crates/converter/src/`(`intake.rs`、`raw.rs`、`yaml_read.rs`、`toml_read.rs`、`scalar.rs`、`resolve.rs`、`yaml_write.rs`、`toml_write.rs`、`types.rs`、`src/tests/`)
  - `crates/*/tests/` の残りのテスト(ファイル名と `fn` の一覧の粒度)
  - `demo/windows/`(`run-demo.cmd` の `watch --follow` の箇所)、`demo/scripts/`、`demo/*.toml`、`demo/demo.env`
  - `docs/spec/`(7 言語版 × 13 文書。`rust/` の目次と v0.10.0 の機能の記述の有無だけ)
  - `crates/core/tests/fixtures/`(図の参照出力。大きさだけ)、`crates/cli/tests/fixtures/`、`crates/converter/tests/fixtures/`
  - `.gitattributes`、`.gitignore`、`LICENSE`
- **Left out**:
  - `target/`(ビルド出力。`.gitignore` が無視)
  - `local-docs/`(個人の参照文書。`.gitignore` が無視)
  - `.idea/`(IDE の設定。What to Skip の IDE のキャッシュ)
  - `.git/`
  - `.claude/`、`aidlc/`、`aidlc.settings.local.json`、`.gitignore` の AI-DLC の節(AI-DLC 自身のもの)

### Packages Found
- `conductor-core` — library — Rust — 定義の読み込みと検証、純粋な判断・記録(試行の時刻を含む)、時刻の整形、`.env` の解析と統合、図、`--dry-run` の計画、`watch --follow` の終わりの判定(`pipeline_activity`)
- `conductor-runtime` — library(+ テスト専用の補助バイナリ `conductor-test-helper`)— Rust — ジョブの起動と待ち、シグナルの転送、ログのファイル(`log_dir`)、時計、ステートの読み書き、`.env` の読み取り、`init` のファイルの作成
- `conductor-cli` — binary `conductor`(+ ライブラリ `conductor_cli`、テスト専用の補助バイナリ `conductor-cli-test-helper`)— Rust — 引数、流れの組み立て、シグナル、表示(英日)、終了コード
- `conductor-converter` — library — Rust — YAML と TOML の相互変換(core に依存しない)

### Build System
- **Type**: Cargo ワークスペース(`resolver = "3"`、`members = ["crates/*"]`、edition 2024、`rust-version = "1.100"`)
- **Config Files**: `Cargo.toml`、`crates/*/Cargo.toml`、`Cargo.lock`(追跡。CI は常に `--locked`)、`.github/workflows/*.yml`
- **Build Dependencies**: `conductor-cli` → {`conductor-core`、`conductor-runtime`、`conductor-converter`}、`conductor-runtime` → `conductor-core`。core と converter はほかのクレートに依存しない。4 クレートとも版は 0.10.0。

### APIs Discovered
- CLI — `crates/cli/src/args.rs`、`types.rs` — 8 コマンド(`run`、`pipeline`、`watch`、`reset`、`validate`、`graph`、`init`、`convert`)と `--help`・`--version`。オプションの表 `OPTIONS` に `--follow`・`--interval`(v0.10.0)を含む。終了コード 0、ジョブの値、1、64〜70、127、128 + 番号
- ファイルの書式 — 定義(`PIPELINE_KEYS` 9 個、`JOB_KEYS` 9 個)、`.env`、ステート(`TOP_KEYS`、`JOB_KEYS` 5 個、`ATTEMPT_KEYS` 2 個)、ログのファイル名
- ライブラリ — core(`lib.rs` の契約 C2 の表。v0.10.0 で `start_attempt`・`end_attempt`・`cancel_attempt`・`pipeline_activity`・時刻の関数が増えた)、runtime(契約 C3。`run_job` は 5 引数、`now`、`open_log_file`、`safe_log_stem`)、converter(`convert`、`convert_text`)、cli(`run_cli`、`run_cli_with`、trait `Runtime` の 14 メソッドなど)
- ネットワークの API — 無し

### Frameworks & Libraries
- `toml` — 1.2.1(ロックは 1.2.1+spec-1.1.0)— 定義・ステートの読み取り、ステートと変換の書き出し
- `thiserror` — 2.0.21 — エラー型
- `lexopt` — 0.3.2 — 引数の字句の分割
- `saphyr-parser` — 0.3.4 — YAML の読み取り
- `signal-hook` — 0.4.6 — UNIX のシグナルの監視
- `ctrlc` — 3.5.2 — Windows の Ctrl+C
- `libc` — 0.2(ロックは 0.2.191)— UNIX の `kill`
- `tempfile` — 3.27.0 — 開発時のみ

### Test Coverage
- **Test Directories**: `crates/core/tests/`(36 ファイル)、`crates/runtime/tests/`(16)、`crates/cli/tests/`(26)、`crates/converter/tests/`(2)、各モジュールの `#[cfg(test)] mod tests`、`crates/cli/src/tests/`(10 ファイル + `common/`)、`crates/converter/src/tests/`(3 ファイル + `mod.rs`)。`#[test]` は合計 1,452
- **Test Frameworks**: Rust 標準の `#[test]` だけ(テスト名は日本語)。偽の `Runtime`/`Sink`/`Env`/`Signals` を注入する流れのテストと、実バイナリのテスト
- **Coverage Config**: present(`ci.yml` の `coverage`: `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`)

### Code Quality Indicators
- **Linting**: `rustfmt`(`cargo fmt --all -- --check`)、`clippy`(`--workspace --all-targets --locked -- -D warnings`)。設定ファイル無し(既定)
- **CI/CD**: `ci.yml`(lint、build-check、test(3 OS)、msrv-test、coverage、docs-check、startup-bench(警告のみ))、`release.yml`(verify-tag → checks(ci.yml の再利用)→ build(4 対象)→ draft-release)、`audit.yml`(警告のみ)
- **Documentation**: `README.md`(953 行、日本語。`docs-check` が必須の語句を検査)、`demo/README.md`、各モジュールの `//!` が規則の番号(BR/NFR/FR)と対応。`docs/spec/` は 7 言語版

### Technical Debt Signals
- `watch --follow` の「止まった」の判定が、再開の直前の短い間にも当てはまる。`crates/core/src/flow/activity.rs:49-63` は「動いている試行が無く、`error` のジョブがある」で `Stopped`。エラーで止まったステートのまま `watch --follow` を始めると、再開のジョブが試行を書く(`crates/cli/src/step.rs:231-238`)前の最初の 1 回で終わる。README は `README.md:299` で注意として書く(F1)
- Windows の予約名の検査が、名前の前後の空白を除かない。`crates/runtime/src/log.rs:88-94` は、最初の `.` より前の部分 `head` を、そのまま `RESERVED_NAMES` と比べる(`"CON .x"` の `head` は `"CON "` で一致しない)。名前の表のテスト `crates/runtime/tests/log_file.rs:49-80` に空白の例は無い(F2)
- 読み取りのスレッドの「止める」印は、読み取りの後にだけ見る。`crates/runtime/src/runner.rs:313-322` は `reader.read` が返ってから `shared.stop` を確かめるため、印が立った後も、次の読み取りが返るまでは読み続ける(F2)
- 性能のテストの上限が、カバレッジの計測の下での遅さを見込まない。`crates/core/tests/plan_simulate_perf.rs:39` の `LIMIT` は 150ms(「debug の見積もり約 31 ms の約 5 倍」)。計画の模擬はジョブごとに定義の位置と記録を前から探す(`crates/core/src/flow/decide.rs` の `job_index`・`record_of`、`crates/core/src/flow/record.rs:197-212` の `apply_skip_in` のエラーの確認と最初の `pending` の探索)ため、1,000 ジョブで二乗に近い回数の比較になる(推論)。`ci.yml` の `coverage` は同じテストを計測つきで走らせる(F3)
- `DEMO_PAUSE=1` のデモで、場面 4 の `watch --follow` が画面を消す。`demo/run-demo.sh:94-96` は端末で `watch --follow` を呼び、`crates/cli/src/report/mod.rs:262-268` の `FramePrefix::for_frame` は、UNIX 系の端末では 1 回目から `ClearScreen`(`ESC[H ESC[2J`)を置く。デモのテスト `crates/cli/tests/unix_demo.rs:44` は `DEMO_PAUSE` を消し、標準出力はパイプなので、この動きは検査されない(F5)
- u6 で 1 度だけ出た原因不明の失敗(F4)の手がかりは、コードからは確定できない。実時計を挟む検査は `crates/cli/tests/binary_basic.rs:414-417`、`crates/cli/tests/binary_log.rs:75-77` などにある(どれも `before <= 値 <= after` の形)
- 文書の古さ: `docs/spec/rust/00-README.md` は v0.10.0 を名乗るが、`follow`・`log_dir`・`unset_env` の記述は `05-execution-flow.md` などの数か所だけ。誤りの種類を「33 種」と書くが、実装の `DefinitionErrorKind` は 34 種、`EnvFileErrorKind` は 14 種
- runtime の `lib.rs` の契約の表の `run_job` の説明が「足す環境変数」だけで、消す名前(`unset`)の引数を書かない。呼び出しの順序の節は「初期ステートを `Fast` で書く」とあり、v0.10.0 の「試行を始めて `Fast` で書く」(`step.rs`)と言い方がずれている
- `team.md` の Testing Posture のテストの数(core 33、runtime 14、cli 25、`#[test]` 1,170)が、今の数(36、16、26、1,452)と合わない
- `TODO`/`FIXME`/`HACK` は無い。`#[allow]` は本体では `runtime/src/platform/unix.rs` の `unsafe_code` だけ(テストの `common` に `dead_code` が 5 つ)

## Handoff Summary
- **Intent-relevant finding**: F1〜F5 の箇所はすべて特定できた。F1 は `crates/core/src/flow/activity.rs:49-63` の `Stopped` の条件と `README.md:295`・`README.md:299` の説明。F2 は `crates/runtime/src/log.rs:88-94`(予約名の比較に前後の空白が残る)と `crates/runtime/src/runner.rs:313-322`(「止める」印を読み取りの後にだけ見る)。F3 は `crates/core/tests/plan_simulate_perf.rs:39` の 150ms の上限と、計画の模擬の二乗に近い探索(推論)。F5 は `demo/run-demo.sh:94-96` と `crates/cli/src/report/mod.rs:262-268`(端末では 1 回目から画面を消す)。F4 はコードに手がかりが無い。
- **Risks / follow-up**: (1) F2 の名前の規則を変えると、同じジョブ ID のログのファイル名が v0.10.0 と変わりうる(空白を含む ID だけ)。README の「名前の置き換え」(`README.md:625`)と `log_file.rs` の表を揃える必要がある。(2) F1・F5 は README・`demo/README.md`・`demo/windows/run-demo.cmd` の 3 か所に同じ説明があり、`crates/cli/src/tests/docs.rs` と `unix_demo.rs`・`windows_demo.rs` が文言の一部を検査する。(3) F3 は上限を下げない方針(brief)。`coverage` は `--fail-under-lines 80` で、テストの失敗はカバレッジの失敗と同じにジョブを落とす。(4) 新しい依存を足さない(`project.md` の NEVER)、テストの都合で本番を曲げない、`cfg` と `unsafe` の位置の制約は、F2 の修正でも守る必要がある(`platform` の外に OS の分岐を置かない)。
