# 開発者のコードの走査(v0104-split-files)

- 日付: 2026-11-28
- コミット: `9376cb7b3791a48bd584dab803c43ecbff6b976d`(走査の前の snapshot の head_commit。作業ツリーの差は AI-DLC の記録だけで、コードの差は無い)
- 広さ: Full rescan(リポジトリ全体 `./`)。深さ: Minimal
- 方法: 読むだけのコマンド(`grep`、`wc`、`find`、`git log`、`git diff`)とファイルの読み取り。テスト・ビルド・プログラムは走らせていない。

## Developer Code Scan Results

### Scan Coverage

- **Analyzed deeply**:
  - `./`(リポジトリ全体。下の行は、この intent のために行の単位で読んだもの)
  - `crates/cli/src/report/mod.rs`(746 行。全行)
  - `crates/core/src/definition/walk/mod.rs`(695 行。全行)
  - `crates/core/src/definition/walk/env.rs`、`crates/core/src/definition/walk/retry.rs`(親の項目を使う箇所)
  - `crates/cli/src/args.rs`(679 行。全行)
  - `crates/cli/tests/spec_numbers.rs`(930 行。読むパスと定数、取り出しの補助)
  - `crates/cli/src/tests/docs.rs`、`crates/cli/src/tests/mod.rs`、`crates/cli/src/tests/report_messages.rs`(イベントの一覧)、`crates/cli/src/tests/args_grammar.rs`(入口)、`crates/cli/src/tests/escape_table.rs`(入口)
  - `crates/cli/tests/nfr_source.rs`、`crates/runtime/tests/nfr_source.rs`、`crates/cli/tests/msrv_consistency.rs`
  - `crates/cli/src/lib.rs`、`crates/cli/src/types.rs`(項目の一覧)、`crates/cli/src/report/messages.rs`(項目の一覧)、`crates/core/src/definition/mod.rs`
  - `docs/spec/rust/`(この 3 つのファイルのパスを書く箇所)、`.github/workflows/`
  - 前回の知識ベース(v0.10.3、`8c5c21e`)からの差 `git diff 8c5c21e 9376cb7`(コードは `crates/cli/tests/spec_numbers.rs` の追加、版、テスト名 2 つ、仕様書だけ)。ほかのモジュールは前回の知識ベースと突き合わせた
- **Skimmed only**:
  - `docs/spec/csharp/`、`docs/spec/go/`、`docs/spec/powershell/`、`docs/spec/python/`、`docs/spec/typescript/`(ファイルの数と版の記述だけ)
  - `crates/converter/src/`(前回から変わっていない。構成だけ)
  - `crates/core/tests/fixtures/`、`crates/cli/tests/fixtures/`、`crates/converter/tests/fixtures/`、`demo/scripts/`、`demo/windows/scripts/`(構成だけ)
- **Left out**:
  - `target/`(ビルド出力。`.gitignore`)
  - `local-docs/`(個人の参照文書。`.gitignore`)
  - `.idea/`(IDE の設定)
  - `.claude/`、`aidlc/`(AI-DLC 自身。前回の知識ベースとこの intent の記録だけを読んだ)、`.mcp.json` の AI-DLC の部分、`.gitignore` の AI-DLC の区画、`aidlc.settings.local.json`

### Packages Found

- `conductor-core` — lib — Rust — 定義の検証と純粋な判断・記録・時刻・図・計画(`crates/core/`)
- `conductor-runtime` — lib + テスト補助 bin — Rust — 起動、ログ、時計、ステートとファイルの入出力(`crates/runtime/`)
- `conductor-converter` — lib — Rust — YAML と TOML の相互変換(`crates/converter/`)
- `conductor-cli` — lib + bin `conductor` + テスト補助 bin — Rust — 引数、流れ、表示、終了コード(`crates/cli/`)

4 クレートとも版 0.10.3(`crates/*/Cargo.toml:3`)。

### Build System

- **Type**: Cargo ワークスペース(`resolver = "3"`、`members = ["crates/*"]`、edition 2024、`rust-version = "1.100"`)
- **Config Files**: `Cargo.toml`、`Cargo.lock`(追跡。CI は `--locked`)、`crates/*/Cargo.toml`、`.github/workflows/{ci,release,audit}.yml`
- **Build Dependencies**: cli → {core, runtime, converter}、runtime → core。core と converter はほかのクレートに依存しない

### APIs Discovered

- ネットワークの API は無い。
- CLI — `crates/cli/src/args.rs` — 8 コマンド(`CommandKind::ALL`)、オプション 13(`Opt`)と `-h`/`--help`/`-V`/`--version`
- ライブラリの公開の口 — `crates/{core,runtime,converter,cli}/src/lib.rs` — 前回の知識ベースから変わっていない(`api-documentation.md` の 8)

### Frameworks & Libraries

- `toml` — 1.2.1 — 定義・ステートの読み書き
- `thiserror` — 2.0.21 — エラー型
- `lexopt` — 0.3.2 — 引数の字句(`args.rs` だけが使う)
- `saphyr-parser` — 0.3.4 — YAML
- `signal-hook` — 0.4.6 — UNIX のシグナル(cli の `platform`)
- `ctrlc` — 3.5.2 — Windows の Ctrl+C(cli の `platform`)
- `libc` — 0.2 — `kill`(runtime の `platform/unix.rs`)
- `tempfile` — 3.27.0 — 開発時

`Cargo.lock` の差は v0.10.2 → v0.10.3 で 4 クレートの版だけ。

### Test Coverage

- **Test Directories**: `crates/core/tests/`、`crates/runtime/tests/`、`crates/cli/tests/`、`crates/converter/tests/`、`crates/cli/src/tests/`、`crates/converter/src/tests/`、各モジュールの `mod tests`
- **Test Frameworks**: Rust 標準の `#[test]` だけ(テスト名は日本語)
- **Coverage Config**: 有り(CI の `coverage`。`cargo llvm-cov --fail-under-lines 80`、テスト専用の補助バイナリを除く)

テストの数(`9376cb7`。読むだけのコマンドで数えた):

| クレート | `tests/` の直下の `.rs` | `src/tests/` の `.rs`(`mod.rs`・`common/` を含む) | `#[test]`(`grep -rE '^\s*#\[test\]'`) |
|----------|------------------------|---------------------------------------------------|----------------------------------------|
| core | 36 | — | 472 |
| runtime | 16 | — | 259 |
| cli | 27 | 12 | 625 |
| converter | 2 | 4 | 116 |
| 合計 | 81 | 16 | 1,472 |

v0.10.3 で cli に `tests/spec_numbers.rs`(12 テスト)が加わり、cli は 26 → 27 ファイル、613 → 625 テストになった。3 つの対象のファイルには `#[test]` も `mod tests` も無い(テストはすべて別のファイル)。

### Code Quality Indicators

- **Linting**: `rustfmt`・`clippy -D warnings`(設定ファイル無し)。CI の `lint`
- **CI/CD**: `.github/workflows/ci.yml`(lint、build-check、test 3 OS、msrv-test、coverage、docs-check、startup-bench は警告のみ)、`release.yml`、`audit.yml`。ワークフローは 3 つの対象のファイルのパスを書いていない
- **Documentation**: README(日本語)、各モジュールの `//!`。仕様書 `docs/spec/rust/` が対象のファイルのパスを書く(下の 6)

### Technical Debt Signals

- 大きなファイル(この intent の対象): `report/mod.rs` 746 行(`render` が 188 行、35 の枝)、`walk/mod.rs` 695 行(`impl Walker` が 429 行)、`args.rs` 679 行(`build_command` が 132 行)。
- `TODO`/`FIXME`/`HACK` は 0。本体の `#[allow]` は `runtime/src/platform/unix.rs:20` の 1 つだけ。
- 古い名前の文書: `crates/core/src/error.rs:176` が「conductor-cli の `report::escape` が包む」と書く(実際の名前は `escape_name`・`escape_full`)。

## この intent のための詳細

### 1. `crates/cli/src/report/mod.rs`(746 行)

#### 1.1 まとまり(行の範囲)

| 行 | まとまり | 項目 |
|----|----------|------|
| 1-31 | ライセンスと `//!` | 表示の責務、遅延の判定、エスケープ(`escape_name` 64 文字、`escape_full` 切らない)、converter の写しと `src/tests/escape_table.rs` |
| 33-39 | 子モジュール | `pub mod graph_dot`、`pub mod graph_mermaid`、`pub mod graph_text`、`pub mod init_template`、`pub(crate) mod messages`、`pub mod plan_text`、`mod watch` |
| 41-59 | `use` | `OnceCell`、`Path`、converter の `ConversionError`、core の 13 項目(`escape_display`、`summarize_errors` を含む)、runtime の `CommandError`・`CreateError`・`StateError`、`crate::deps::{Env, Sink}`、`crate::platform`、`crate::types::{Language, Level, UsageError}`、`messages` の 17 項目 |
| 61-84 | `pub enum StateContext`(5 変種)と `pub(crate) fn for_read` | 定義 |
| 86-97 | `pub enum ResultDetail`(4 変種) | 定義 |
| 99-242 | `pub enum DisplayEvent<'a>`(37 変種) | 定義。区切りの注記で error 19(103-161)、notice 5(162-190)、verbose 5(191-217)、output 8(218-241) |
| 244-278 | `pub enum FramePrefix`(3 変種)、`pub(crate) const CLEAR_SCREEN`(258)、`for_frame`・`text` | 定義 |
| 280-323 | `impl DisplayEvent { pub fn level }` | 定義(水準の対応の `match`。37 変種を 4 つの水準へ) |
| 325-374 | `pub struct Reporter`、`new`、`language`、`color_enabled`、`show` | 表示の本体 |
| 376-564 | `pub fn render` | 表示の本体(35 の枝。下の 1.3) |
| 567-570 | `fn show_path` | 1 行の組み立て(パスを `escape_full`) |
| 572-587 | `fn detect_language` | 表示の本体(ロケール) |
| 589-604 | `fn error_line`、`fn place_of` | 1 行の組み立て |
| 606-638 | `fn definition_line`、`fn env_file_lines`、`fn env_warning_line` | 1 行の組み立て |
| 640-651 | `const MAX_DISPLAY_CHARS`、`pub(crate) fn escape_name`、`pub(crate) fn escape_full` | エスケープ |
| 653-657 | `pub(crate) fn push_line` | 子モジュールが共有する整形の補助 |
| 659-746 | `fn job_place`、`fn command_line`、`fn usage_text`、`fn state_line`、`fn conversion_line`、`fn job_result_line` | 1 行の組み立て |

#### 1.2 まとまりの間の依存

- `render` → 1 行の組み立ての全関数、`messages`、`watch::render_table`、`self.language()`、`self.color_enabled()`、`self.render`(`WatchFrame` の再帰)。
- 1 行の組み立て → エスケープ(`escape_name`・`escape_full`)、`messages`、`StateContext`(`state_line`)、`ResultDetail`(`job_result_line`)。
- エスケープ → core の `escape_display`・`MAX_KEY_DISPLAY_CHARS` だけ。
- `DisplayEvent` → `StateContext`、`ResultDetail`、`FramePrefix` の型と、外のクレートの型。`level` → `crate::types::Level`。
- `messages.rs:28` が `use crate::report::StateContext`(`reset_hint` の引数)。`messages` と `mod.rs` は互いに使う(モジュールの循環。Rust では問題ない)。
- 子モジュールが親から使うもの: `graph_text.rs:29,35`(`escape_full`、`push_line`)、`graph_dot.rs:28`(`escape_full`)、`plan_text.rs:26,34`(`escape_full`、`push_line`)、`watch.rs:25,30`(`escape_name`、`push_line`)。どれも `super::` で引く。

#### 1.3 `Reporter::render` の枝(35 の枝、37 の変種)

`match` の枝は 35(`Graph | Plan | InitTemplate` が 1 つの枝)。依頼の「40」は、この数え方では 35 になる。`DisplayEvent::` の現れは `render` の中で 39(`WatchFrame` の中で `WatchTable`・`WatchEmpty` を作る 2 つを含む)。

水準ごと(`level` の対応):

| 水準 | 枝の数(変種) | 枝 |
|------|----------------|----|
| Error | 19(19) | `SignalSetupFailed`、`DefinitionErrors`、`EnvFileErrors`、`EnvWarnings`、`CommandErrors`、`Usage`、`State`、`ResetAllKept`、`ConsistencyMismatch`、`Rejected`、`StartFailure`、`SuccessCodeNotListed`、`Conversion`、`OutputFailed`、`CompletionUndetermined`、`InitExists`、`InitFailed`、`InitStdoutFailed`、`LogNotCreated` |
| Notice | 5(5) | `RetryScheduled`、`InitCreated`、`InitReplaced`、`LogWriteFailed`、`LogOutputCutShort` |
| Verbose | 5(5) | `JobResult`、`JobSkipped`、`Transition`、`Interrupted`、`DefinitionValid` |
| Output | 6(8) | `Version`、`WatchTable`、`WatchEmpty`、`WatchFrame`、`Graph \| Plan \| InitTemplate`、`Help` |

表示の形ごと(分ける関数の候補の単位):

| 形 | 枝 | 中身 |
|----|----|------|
| A. 固定の文(言語を判定しない) | `SignalSetupFailed`、`Version` | `messages::SIGNAL_SETUP_FAILED`、`CARGO_PKG_VERSION` |
| B. 誤りの一覧(1 件 1 行) | `DefinitionErrors`、`EnvFileErrors`、`EnvWarnings`、`CommandErrors` | `definition_line`、`env_file_lines`(20 件で打ち切り)、`env_warning_line`(接頭辞は warning)、`command_line` |
| C. 関数に任せる誤りの 1 行 | `Usage`、`State`、`Conversion` | `usage_text`(使い方の全文を足す)、`state_line`、`conversion_line` |
| D. `error_line` で場所つきの 1 行 | `ConsistencyMismatch`、`Rejected`、`StartFailure`、`SuccessCodeNotListed`、`CompletionUndetermined` | 場所は `show_path` か `job_place` |
| E. `ERROR_PREFIX` + 文 | `OutputFailed`、`InitExists`、`InitFailed`、`InitStdoutFailed`、`LogNotCreated` | `messages` の定数か関数 |
| F. `WARNING_PREFIX` + 文 | `ResetAllKept`(名前を 1 行ずつ)、`LogWriteFailed`、`LogOutputCutShort` | `ResetAllKept` の水準は Error |
| G. 通知 | `RetryScheduled`(`RETRY_PREFIX`)、`InitCreated`、`InitReplaced`(接頭辞なし) | `messages` の関数 |
| H. `-v` の行(`VERBOSE_PREFIX`) | `JobResult`、`JobSkipped`、`Transition`、`Interrupted`、`DefinitionValid` | `job_result_line` ほか、枝の中で組み立て |
| I. 標準出力 | `WatchTable`、`WatchEmpty`、`WatchFrame`、`Graph \| Plan \| InitTemplate`、`Help` | `watch::render_table`(色を使う)、再帰、そのまま、`messages::USAGE` |

枝の中に組み立てを直に書いているのは、`ResetAllKept`、`ConsistencyMismatch`、`Rejected`、`StartFailure`、`RetryScheduled`、`JobSkipped`、`Transition`(15 行)、`WatchFrame` など。水準が Error なのに `conductor: warning:` で始まるのは `EnvWarnings` と `ResetAllKept`。

分けるときに保つこと:

- 言語の判定の遅延: `render` は閉包 `lang` で、必要な枝だけが `self.language()` を呼ぶ。A と `Graph | Plan | InitTemplate` は言語を判定しない。テスト `言語と色の判定は_最初に必要になるまで行われない`(`src/tests/report_messages.rs:694`)、`バージョンとシグナルの準備の失敗は_ロケールを判定しない`(`:726`)が確かめる。
- 色は `WatchTable` だけが `self.color_enabled()` で判定する。`WatchFrame` は `self.render` を再帰で呼ぶ(関数に分けても `Reporter` を渡す必要がある)。

#### 1.4 外からの参照(`report` は `lib.rs:99` の `mod report;` で private。`pub` は crate の中で見える)

| 名前 | 使う場所(`use` の行) |
|------|------------------------|
| `DisplayEvent` | `lib.rs:118`、`ctx.rs:32`、`run.rs:31`、`pipeline.rs:36`、`step.rs:48`、`dry_run.rs:26`、`aux_commands.rs:40`、`src/tests/report_messages.rs:25` |
| `Reporter` | `lib.rs:118`、`ctx.rs:32`、`src/tests/report_messages.rs:25` |
| `StateContext`(`for_read` を含む) | `ctx.rs:32`(`for_read` は `:295`・`:305`)、`run.rs:31`、`pipeline.rs:36`、`step.rs:48`、`aux_commands.rs:40`(`for_read` は `:71`・`:143`)、`messages.rs:28`、`src/tests/report_messages.rs:25` |
| `ResultDetail` | `step.rs:48`(`:367`〜`:421`)、`src/tests/report_messages.rs:25` |
| `FramePrefix` | `aux_commands.rs:40`、`src/tests/report_messages.rs:25` |
| `escape_name`、`escape_full` | `src/tests/escape_table.rs:20`(`crate::report::{escape_full, escape_name}`)、子モジュール(1.2) |
| `push_line` | `graph_text.rs`、`plan_text.rs`、`watch.rs`(`super::push_line`) |
| `CLEAR_SCREEN`、`MAX_DISPLAY_CHARS`、`detect_language`、1 行の組み立ての関数 | `mod.rs` の中だけ |
| `report::messages::*` | `panic_hook.rs:32`(`INTERNAL_ERROR`)、`src/tests/docs.rs:212`(`LOG_*`)・`:523`(`USAGE`) |
| `report::graph_text::render_text` ほか子モジュール | `aux_commands.rs:36-39`、`dry_run.rs:27`、`src/tests/{docs,graph_dot,graph_report,init_template,plan_report}.rs` |

Reporter のメソッド(`render`・`show`・`language`・`color_enabled`・`new`)を呼ぶファイル: `aux_commands.rs`(12)、`ctx.rs`(11)、`step.rs`(9)、`lib.rs`(5)、`pipeline.rs`(4)、`run.rs`(2)、`dry_run.rs`(2)、`src/tests/report_messages.rs`(128)。統合テスト(`tests/`)は `report` を使えない(公開の口は `lib.rs:107-111` の差し替え口と `run_cli`・`run_cli_with`・`install_panic_hook` だけ)。

全種類のイベントのテスト `with_all_events`(`src/tests/report_messages.rs:99`)は手で並べた一覧で、37 変種のうち 35 を作る(`DefinitionValid` と `ResetAllKept` は入っていない。`WatchTable` は補助 `watch_table` で作る)。網羅の `match` ではないので、変種を分けても一覧はコンパイルで止まらない。

### 2. `crates/core/src/definition/walk/mod.rs`(695 行)

#### 2.1 まとまり(行の範囲)

| 行 | まとまり | 項目 |
|----|----------|------|
| 1-27 | `//!` | 検査する規則、`retry`・`env` の子モジュール(「どちらも `Walker` のメソッドで、ここから呼ぶ」) |
| 29-30 | 子モジュール | `mod env;`、`mod retry;` |
| 32-45 | `use` | `HashMap`、`toml::Spanned`、`DeTable`・`DeValue`、`crate::definition::integer_value`、`VERSION_KEY`、`DefinitionError`・`Kind`・`Locate`・`quote_key`、`env_name_rule`・`is_valid_env_name`、types の 10 項目、`retry::{JobRetrySetting, PipelineRetryDefault, build_retry}` |
| 47-70 | キーの表 | `ROOT_KEYS`(47)、`PIPELINE_KEYS`(48-58。9 個)、`JOB_KEYS`(59-69。9 個)、`TRANSITION_KEYS`(70) |
| 72-73 | `NO_JOBS_REASON` | 定数 |
| 75-119 | 中間の型 | `PipelineHead`(76-86)、`RawTransition`(88-91)、`SuccessCodes`(94-101)、`RawJob`(103-119) |
| 121-125 | `struct Walker` | `file`、`locate`、`errors` |
| 128-138 | 基本 | `push`、`type_error` |
| 140-151 | キーの検査 | `check_keys` |
| 153-219 | 値の読み取り | `string`(154-174)、`boolean`(177-191)、`exit_codes`(194-219) |
| 221-277 | 値の読み取り(`command`) | `command` |
| 279-368 | `[pipeline]` | `pipeline_section`(281-308)、`pipeline_head`(311-357)、`log_dir`(361-368) |
| 370-427 | 遷移 | `transitions`(370-410)、`transition_on`(412-427) |
| 429-465 | ジョブ | `job` |
| 467-554 | ジョブをまたぐ検査 | `cross_check`(469-526。ID の一意、遷移先)、`check_exit_codes`(528-554) |
| 557-568 | 自由関数 | `exit_code_of`、`integer_of` |
| 570-643 | 入口 | `pub fn walk`(唯一の外の入口。`definition/mod.rs:69` から呼ぶ) |
| 645-695 | 組み立て | `build_jobs`(649-690)、`names_of`(693-695) |

依頼の 3 つとの対応: 値の読み取り = 153-277(`string`・`boolean`・`exit_codes`・`command`。`log_dir` と自由関数 557-568 の置き場所は決めること)、遷移 = 370-427(と `RawTransition`・`TRANSITION_KEYS`)、ジョブをまたぐ検査 = 467-554。

#### 2.2 `env.rs`・`retry.rs` が親から使うもの

| 子 | `use super::…` | 使う `Walker` のメソッド(回数) | 使う親の型のフィールド |
|----|----------------|--------------------------------|-------------------------|
| `env.rs:24` | `PipelineHead`、`RawJob`、`Walker` | `push`(9)、`type_error`(6)、`string`(1。`env_file_table`)、`boolean`(1。同)、`check_keys`(1) | `head.env`、`head.unset_env`、`job.env`、`job.unset_env` |
| `retry.rs:28` | `RawJob`、`SuccessCodes`、`Walker`、`exit_code_of`、`integer_of` | `push`(12)、`check_keys`(1) | `job.retry`、`job.start`、`job.success`、`job.warning`、`DeTable::get` |

親が子から呼ぶもの: `env_table`・`env_file`・`unset_env`(`pipeline_head:331-334`、`job:460-462`)、`check_unset_env`(`cross_check:525`)、`pipeline_retry`(`pipeline_section:305`)、`job_retry`(`job:459`)、`check_retry`(`cross_check:523`)、`PipelineRetryDefault::into_policy`(`walk:612`)、`build_retry`(`build_jobs:680`)。子の公開は `pub(super)`(子の `impl Walker<'_>` のメソッドと、`PipelineRetryDefault`・`RawRetry`・`JobRetrySetting`・`build_retry`)。

形の決まり(前例): 子は `impl Walker<'_>` を持ち、外に出すメソッドだけ `pub(super)`、補助は private。親の private の項目(型・自由関数・`use`)は子から `super::` で見える。前例のコミットは `c34d3f7`(`walk.rs` を `walk/{mod,retry,env}.rs` に分ける。1 コミット)。

`string`・`boolean`・`check_keys` を子モジュール(たとえば値の読み取りの子)に移すと、`env.rs` から呼ぶために `pub(super)` が要る(兄弟の子の `pub(super)` は `walk` の中のすべてで見える)。`exit_code_of`・`integer_of` を移すなら、`retry.rs:28` の `use super::{…}` を直すか、`mod.rs` に `use` を置く。

#### 2.3 保つこと

- 誤りの順: `walk` は誤りを位置(offset)で `sort_by_key`(安定)する(`:641`)。同じ位置の誤りは積んだ順のまま残るので、呼ぶ順(`pipeline_section` で `pipeline_retry` → `pipeline_head`。`pipeline_head` で `check_keys` → `name` → `description` → `run_id_env` → `remove_on_complete` → `env_table` → `env_file` → `log_dir` → `unset_env` → 名前の検査。`job` で `check_keys` → `id` → `command` → `warning` → `success` → `transitions` → `job_retry` → `env_table` → `env_file` → `unset_env`。`cross_check` で ID → ジョブごとに遷移先・`check_exit_codes`・`check_retry` → `check_unset_env`)を変えないこと。
- `RawJob::complete` は `job` の中の誤りの数の差で決まる(`:448`・`:463`)。
- テストは公開の入口(`parse_definition`・`load_definition`)だけから(`walk` に `mod tests` は無い。core の統合テスト `definition_*.rs`・`env_definition.rs` など)。

### 3. `crates/cli/src/args.rs`(679 行)

#### 3.1 まとまり(行の範囲)

| 行 | まとまり | 項目 |
|----|----------|------|
| 1-55 | `//!` | 文法の表(8 コマンド × 14 列)と規則の注記(使い方の表の 1 か所。`team.md` の「`args.rs`、`messages.rs`、README の 3 か所」) |
| 57-67 | `use` | `OsString`、`TargetFormat`、`Assumption`、`lexopt::prelude::*`、`crate::types` の 11 項目(`C` の別名を含む) |
| 69-107 | オプションの表の型 | `enum Opt`(13 変種)、`enum Allowed`(3)、`struct OptSpec` |
| 109-119 | 定数 | `ALL_BUT_CONVERT`、`RUN_AND_PIPELINE` |
| 121-203 | 表 | `const OPTIONS: &[OptSpec]`(13 行) |
| 205-236 | コンパイル時の検査 | `const _`(表と `Opt` の並び)、`const fn is_long_spelling`、`const _`(`RUN_ID_OPTION` の綴り) |
| 238-268 | 表を引く | `spec`、`spec_by_long`、`OptSpec::allows`、`Opt::subject` |
| 270-309 | 検査の順 | `enum Check`(2)、`const CHECKS`(25 行) |
| 311-340 | 指定の集計 | `struct Given`(`new`・`push`・`count`・`once`・`take`) |
| 342-356 | 字句の型 | `enum Token`(3)、`enum Special`(2) |
| 358-425 | 字句 | `pub fn parse_args`(唯一の外の入口。`lexopt` のくり返し、`-h`/`-V` の先取り、字句の誤り) |
| 427-448 | 字句の補助 | `map_unexpected_value`、`to_text` |
| 450-581 | 組み立て | `fn build_command`(下の 3.2) |
| 583-639 | 値の解釈 | `parse_assumptions`、`parse_graph_format`、`parse_interval`、`parse_run_id_option` |
| 641-666 | 位置引数 | `enum Operand`(3)、`operand_rule`、`unexpected_argument` |
| 668-679 | 値の解釈 | `parse_target` |

依頼の 3 つとの対応: オプションの表と検査 = 69-340(と `Operand`・`operand_rule`)、字句 = 342-448、値の解釈 = 583-639・668-679(`to_text`・`unexpected_argument` の置き場所は決めること)。

#### 3.2 `build_command`(450-581、132 行)の形

1. 字句を集める(454-462): `Given`、最初の知らないオプション、位置引数。
2. 知らないオプション → `UnknownOption`(465-467)。
3. コマンドの名前(469-476): 無ければ `MissingArgument`、知らなければ `UnknownCommand`。
4. `CHECKS` の順に「使えない」と「重複」(478-497)。
5. `--all` と `--run-id` の併用 → `InvalidValue`(500-507)。
6. `graph` の `--run-id` は `--state` と一緒のときだけ → `OptionNotAllowed`(509-515)。
7. 共通の値の組み立て(517-538): `verbose`、`force`、`config_path`、`all`、`remove_on_complete`、`dry_run`、`assumptions`(`parse_assumptions`)、`with_state`、`format`(`parse_graph_format`)、`follow`、`interval`(`parse_interval`)、`run_id`(`parse_run_id_option`)、`to` の取り出し。
8. 位置引数の数(540-549): `operand_rule` と余りの引数。
9. **コマンドの枝**(551-579)は `match kind` の **5 つの枝**(8 コマンド): `Run`(`job_id`)、`Pipeline | Validate | Graph`(`pipeline_name`)、`Convert`(`convert_input`、`--to` が無ければ `MissingArgument`、`parse_target`)、`Init`(空・空白だけの名前は `InvalidValue`(対象 `"pipeline name"`))、`Watch | Reset`(何もしない)。

誤りの報告は最初の 1 つで、順は上の 1〜9 と 7 の中の並び(`--assume` → `--format` → `--interval` → `--run-id` → 位置引数 → コマンドの枝)で決まる。`src/tests/args_grammar.rs`(69 テスト)が誤りの種類と対象を確かめる。

#### 3.3 まとまりの間の依存

- `parse_args` → `spec_by_long`、`Opt::subject`、`Token`、`Special`、`map_unexpected_value`、`build_command`。
- `map_unexpected_value` → `spec_by_long`。
- `build_command` → `Given`、`CHECKS`、`spec`、`OptSpec::allows`、`Opt::subject`、`operand_rule`、`unexpected_argument`、`to_text`、値の解釈の 5 関数、`CommandKind::from_name`、`CliCommand::new`。
- 値の解釈 → `Opt::subject`(誤りの対象)、`crate::types`(`MIN_`/`MAX_FOLLOW_INTERVAL_SECS`、`RunIdArg::parse`)、core の `Assumption`、converter の `TargetFormat`。
- `Given::new` → `OPTIONS`。コンパイル時の検査 → `OPTIONS`、`Opt`、`RUN_ID_OPTION`。

#### 3.4 外からの参照

- `parse_args` だけ: `lib.rs:113`(`use args::parse_args`、`:150` で呼ぶ)、`src/tests/args_grammar.rs:23`(`use crate::args::parse_args`)。ほかの項目はすべて `args` の中だけ(`lib.rs:90` の `mod args;` は private)。
- `dry_run.rs:32-33`、`aux_commands.rs:265`・`:342` は文書の中で `parse_args` の名前を書くだけ。

### 4. パスや行に依存するテスト

#### 4.1 `crates/cli/tests/spec_numbers.rs`(v0.10.3 で追加。12 テスト)

リポジトリの根(`CARGO_MANIFEST_DIR/../..`。`repo_path` `:46`)からの相対のパスで読む。読むファイルと取り出し方:

| 定数・関数 | 読むファイル | 取り出し方(探す文字列) |
|------------|--------------|--------------------------|
| `WALK_SRC`(`:440`) | `crates/core/src/definition/walk/mod.rs` | `array_strings`(`:150`): `const PIPELINE_KEYS`(`:639`)と `const JOB_KEYS`(`:651`)を含む最初の行から、`= ` の後ろ、`];` までの `"` の数 ÷ 2 |
| `STATE_READER_SRC`(`:441`) | `crates/runtime/src/state/reader.rs` | 同じ形で `const TOP_KEYS`、`const JOB_KEYS`、`const ATTEMPT_KEYS` |
| `attempt_reasons`(`:445`) | `crates/runtime/src/error.rs` | `mod reason {` の中の `pub const` で、直前の `///` に「試行」 |
| `watch_columns`(`:477`) | `crates/cli/src/report/messages.rs` | 行の先頭が `pub const HEADER_` の行の数(今は `:548-552` の 4 つ) |
| `cli_const`(`:489`) | `crates/cli/src/types.rs` | `const MIN_FOLLOW_INTERVAL_SECS:` などを含む行の `=` の後ろ(`types.rs:215-219`) |
| `enum_variants`(`:113`) | `crates/core/src/error.rs`(`pub enum DefinitionErrorKind {`)、`crates/core/src/envfile/mod.rs`(`pub enum EnvFileErrorKind {`)、`crates/cli/src/types.rs`(`pub enum GraphFormat {`) | 見出しの行から `}` だけの行まで |
| `demo_scenes`(`:508`) | `demo/run-demo.sh`、`demo/windows/run-demo.cmd` | `scene "`、`call :scene ` |
| `SPEC_DIRS`・`R`・`L`(`:27-30`) | `docs/spec/rust/`、`docs/spec/language-agnostic/` | 文書 |

この intent との関わり:

- **`walk/mod.rs` の分け方**: `PIPELINE_KEYS` と `JOB_KEYS` を `walk/mod.rs` に残すなら、テストは変わらない。別のファイルへ移すなら `WALK_SRC` を直す必要がある。`TRANSITION_KEYS`・`ROOT_KEYS` はテストが読まない。`array_strings` は「最初に見つかった行」を使うので、同じファイルに `const PIPELINE_KEYS` を含む別の行(たとえば文書の中の語)を前に置くと、取り違える。
- **`report/mod.rs` の分け方**: テストが読むのは `report/messages.rs` の `pub const HEADER_` だけ。`mod.rs` を分けても影響しない(`messages.rs` を動かさない限り)。
- **`args.rs` の分け方**: テストは `args.rs` を読まない(`--interval` の数は `types.rs`、`--format` は `types.rs` の `GraphFormat`)。

#### 4.2 `crates/cli/src/tests/docs.rs`(22 テスト)

- 読むファイルは `README.md` だけ(`CARGO_MANIFEST_DIR/../../README.md`、`:38`)。ソースのファイルは読まない。
- モジュールのパスで使う項目: `crate::report::init_template::render_template`(`:23`)、`crate::report::messages::{LOG_NOT_CREATED, LOG_OUTPUT_CUT_SHORT, LOG_WRITE_FAILED}`(`:212`)、`crate::report::graph_text::render_text`(`:299`・`:333`)、`crate::report::plan_text::render_plan`(`:457`)、`crate::report::messages::USAGE`(`:523`)、`crate::types::Language`。分けたあとも、これらのパスが今の名前で引けること。
- テスト名 `u7_nfr1_10_使い方の_watch_の行は_args_messages_readme_で揃う`(`:521`)は `args` の名前を含むが、`args.rs` は読まない(`USAGE` と README だけ)。

#### 4.3 `crates/cli/tests/nfr_source.rs` の判定(cli の `src/` の全 `.rs`)

- `os_ごとの分岐は_platform_の中だけ`(`:127`): 行の `//` 以降を除き、空白を除いて `cfg(unix)`・`cfg(windows)`・`cfg!(unix)`・`cfg!(windows)`・`target_os`・`cfg(not(unix))`・`cfg(not(windows))` を探す。除外はパスの部品に `platform` があるファイルと `src/tests/` の下。新しいファイルを `report/` や `args/` の下に作るとき、ディレクトリ名に `platform` を使わない限り検査の対象になる。
- `src_に_unsafe_と_low_level_がない`(`:91`)、`src_にネットワークのコードがない`(`:103`): 新しいファイルも対象(除外なし)。
- `テスト専用の経路を実行ファイルに持たない`(`:145`): `panic!(` は `bin/` と `src/tests/` の外で禁止。`#[path`・`include!`・`include_str!`・`include_bytes!` は `src/tests/` の外で禁止。`lib.rs` の最後が `#[cfg(test)]\nmod tests;` であること。
- `クレートは_forbid_unsafe_code_を宣言している`(`:85`): `lib.rs` だけ。
- `crates/runtime/tests/nfr_source.rs` は runtime の `src/` だけで、この intent の 3 ファイルは対象の外。core には `nfr_source.rs` が無い(`#![forbid(unsafe_code)]` は `core/src/lib.rs`)。

#### 4.4 `crates/cli/tests/msrv_consistency.rs`(3 テスト)

- 読むのは `Cargo.toml`、`README.md`、`.github/workflows/ci.yml`(`repo_file`、`:21`)。この intent の分割とは関わらない。版を 0.10.4 にしても関わらない(`rust-version` だけを見る)。

#### 4.5 そのほか

- `crates/cli/tests/binary_basic.rs:267` が `conductor 0.10.3\n` を期待する(版を 0.10.4 にするとき直す)。
- `crates/core/tests/definition_errors.rs`・`definition_valid.rs` は `tests/fixtures/` の定義の行番号を比べる(ソースの行ではない)。`converter/tests/nfr_security.rs` は converter の `src/` だけ。
- `line!()`・`file!()` を使うコードは無い。

### 5. 版を 0.10.4 にするときの場所(`9376cb7` で `0.10.3` を含む箇所)

`crates/{core,runtime,converter,cli}/Cargo.toml:3`、`Cargo.lock`(4 か所)、`crates/cli/tests/binary_basic.rs:267`、仕様書 `docs/spec/rust/` の `00-README.md:3・10`、`03-definition-format.md:1`、`04-cli.md:1・268・274`、`09-converter.md:1`、`11-quality-and-build.md:23-28・562`、`12-rebuild-guide.md:13・362`、`docs/spec/language-agnostic/` の `00-README.md:3・10`、`01-overview.md:3`、`02-architecture.md:3`、`04-cli.md:306・310`、`05-execution-flow.md:3`、`06-state-file.md:3`、`07-process-and-signals.md:3`、`08-output-and-i18n.md:111`、`11-quality-and-build.md:3・781`、`12-rebuild-guide.md:3`。README とデモには無い。

### 6. 対象のファイルのパスを書く文書(分けると古くなりうる)

- `docs/spec/rust/02-architecture.md:335-337`(`definition/walk/mod.rs`・`retry.rs`・`env.rs` のモジュールの表)、`:395`(`args.rs`)、`:408-414`(`report/mod.rs` の責務と子モジュールの表)。
- `docs/spec/rust/08-output-and-i18n.md:3`(`crates/cli/src/report/{mod,messages,watch}.rs`)、`:604`・`:606`・`:645`(エスケープは `report/mod.rs` の `escape_name`・`escape_full`)。
- `docs/spec/rust/07-process-and-signals.md:44`(定義の読み込み時の検査は `definition/walk/mod.rs`)。
- `docs/spec/rust/12-rebuild-guide.md:76・105・117・127・137・150・159・169・179・201・211`(`args.rs`、`report/`、`definition/walk/mod.rs` の `PIPELINE_KEYS`・`JOB_KEYS`)。
- `crates/cli/src/lib.rs:47-59`(モジュールの表。`args`、`report`)、`crates/core/src/lib.rs:57`(`definition::walk`)、`crates/cli/src/report/mod.rs:15-31`、`crates/core/src/definition/walk/mod.rs:22-23`(子モジュールの説明)。
- `aidlc/spaces/default/memory/team.md` の Code Style「使い方の表は `args.rs`、`report/messages.rs`、README の 3 か所を揃える」。
- language-agnostic 版はこの 3 つのファイルのパスを書かない。

## Handoff Summary

- **Intent-relevant finding**: 3 つのファイルはどれも crate の中の private のモジュールで、外の名前の参照は少ない。`args` は `parse_args` だけ(`lib.rs:113`、`src/tests/args_grammar.rs:23`)。`walk` は `walk` だけ(`definition/mod.rs:69`)。`report` は `DisplayEvent`・`Reporter`・`StateContext`・`ResultDetail`・`FramePrefix`・`escape_name`・`escape_full`・`push_line` と子モジュール(上の 1.4)。`Reporter::render` の `match` は 35 の枝(37 変種。依頼の「40」ではない)で、水準で 19・5・5・6、表示の形で A〜I の 9 種に分けられる(1.3)。`build_command` のコマンドの枝は `match kind` の 5 つ(8 コマンド。3.2)。`env.rs`・`retry.rs` が使う `Walker` のメソッドは `push`・`type_error`・`check_keys`・`string`・`boolean`、自由関数は `exit_code_of`・`integer_of`(2.2)。`spec_numbers.rs` が読む対象のファイルは `walk/mod.rs`(`const PIPELINE_KEYS`・`const JOB_KEYS`。`spec_numbers.rs:440`・`:639`・`:651`)と `report/messages.rs`(`pub const HEADER_`。`:478`)。
- **Risks / follow-up**:
  - `PIPELINE_KEYS`・`JOB_KEYS` を `walk/mod.rs` から動かすと `spec_numbers.rs` の `WALK_SRC` が壊れる。`messages.rs` は動かさないこと。
  - `render` を分けても、言語の判定の遅延(必要な枝だけ)と、`WatchFrame` の再帰・`WatchTable` の色の判定を保つこと(`src/tests/report_messages.rs:694`・`:726` が確かめる)。
  - `walk` の誤りは位置で安定に並べるので、同じ位置の誤りは呼ぶ順で決まる。呼ぶ順を変えないこと。`build_command` も最初の誤りだけを返すので、検査の順を変えないこと。
  - 新しいファイルは `nfr_source.rs` の検査の対象(`cfg(unix)` など、`panic!(`、`include!`・`#[path` は `platform`・`src/tests/` の外で禁止)。`#[path]` で別のファイルを読む分け方はできない。
  - `pub use` で今の名前を保つとき、`report` は `crate::report::{escape_full, escape_name}`(`escape_table.rs`)と `crate::report::StateContext`(`messages.rs:28`)を、`walk` は子からの `super::{…}` の参照を保つこと。テストのための `pub` を足さない(`project.md` の NEVER)。
  - 仕様書 `docs/spec/rust/02-architecture.md`・`08`・`07`・`12` と `lib.rs` の `//!` がファイルのパスを書く(6)。直すかを要件で決めること(spec_numbers はそれらを検査しない)。
  - `args.rs` を `args/mod.rs` にすると、`team.md` の「`args.rs`」の書き方も古くなる(規則の文の直しは learnings の手続き)。
  - `with_all_events` は手で並べた一覧で、`DefinitionValid` と `ResetAllKept` を含まない(分割の影響は無いが、網羅の守りではない)。
