# Developer Code Scan(v0102-spec-docs、Full rescan)

- 対象: リポジトリの根(`./`)。snapshot の head_commit は `55958fd0454647339d353ef04e576fc4204a7291`。
- 走査の日: 2026-11-28。テスト・ビルド・プログラムは実行していない(数えるのは `find`・`grep` だけ)。
- 前回の知識ベース(v0101-followups、`4ebd8e7`)から後の、AI-DLC の記録を除くコミットは 14 個(F1〜F5 の直し、テスト、`docs/spec/` の「34 種」と版、版の更新)。差分は 39 ファイル、+668/-99 行。

## Developer Code Scan Results

### Scan Coverage
- **Analyzed deeply**:
  - `docs/spec/rust/`(13 文書。見出しの全部と、v0.10.0〜v0.10.1 の機能に関わる節を行の単位で照合)
  - `crates/core/src/`(`lib.rs`、`flow/`(`mod.rs` の `SimulationMarks`・`record_at`・`position_of`、`decide.rs`、`next_step.rs`、`record.rs`、`activity.rs`、`attempt.rs`)、`plan/simulate.rs`、`definition/walk/{mod,env,retry}.rs`、`error.rs`、`envfile/mod.rs`、`time.rs` の公開の口)
  - `crates/runtime/src/`(`lib.rs`、`log.rs`、`runner.rs` の読み取りのスレッド、`state/reader.rs`、`error.rs`)
  - `crates/cli/src/`(`args.rs` の `--follow`・`--interval`・`--format`、`types.rs`、`step.rs` の試行の書き込み、`aux_commands.rs` の `follow_watch`、`deps.rs` の trait、`report/{mod,messages,watch}.rs`)
  - `crates/*/tests/` と `crates/cli/src/tests/`、`crates/converter/src/tests/`(ファイルの一覧と `#[test]` の数)
  - `Cargo.toml`、`crates/*/Cargo.toml`、`Cargo.lock` の差分、`.github/workflows/`(ci.yml、release.yml、audit.yml の構成)
  - `demo/run-demo.sh`(場面の見出しと場面 4 の待ち)、`README.md`(`watch --follow` の節と使い方の表)
  - `aidlc/spaces/default/memory/team.md` の Testing Posture(読むだけ。brief で許された範囲)
- **Skimmed only**:
  - `docs/spec/language-agnostic/`(キーワードの検索と冒頭の版の記述だけ)
  - `docs/spec/csharp/`、`docs/spec/go/`、`docs/spec/powershell/`、`docs/spec/python/`、`docs/spec/typescript/`(ファイルの数と冒頭の版だけ)
  - `crates/converter/src/`(前回から変わっていない。公開の口だけ)
  - `crates/core/src/graph/`、`crates/cli/src/report/graph_*.rs`、`plan_text.rs`、`init_template.rs`(前回から変わっていない)
  - `demo/windows/`、`demo/README.md`、`crates/*/tests/fixtures/`
- **Left out**:
  - `target/`(ビルド出力。`.gitignore`)
  - `local-docs/`(`.gitignore`)
  - `.idea/`(IDE の設定)
  - AI-DLC の導入物(`.claude/`、`aidlc/` の作業場(上の `team.md` を除く)、`aidlc.settings.local.json`、`.gitignore` の AI-DLC の節)

### Packages Found
- `conductor-core` — lib — Rust — 定義の読み込みと検証、純粋な判断・記録・試行の時刻・時刻・照合・`.env`・図・計画
- `conductor-runtime` — lib(+ テスト専用の bin `conductor-test-helper`)— Rust — 起動・待ち・シグナル・ログのファイル・時計・ステート・`.env` の読み取り・`init` のファイルの作成
- `conductor-cli` — lib + bin `conductor`(+ テスト専用の bin `conductor-cli-test-helper`)— Rust — 引数、流れ、表示(英日)、終了コード
- `conductor-converter` — lib — Rust — YAML と TOML の相互変換

### Build System
- **Type**: Cargo ワークスペース(`members = ["crates/*"]`、`resolver = "3"`、edition 2024、`rust-version = "1.100"`)
- **Config Files**: `Cargo.toml`、`crates/{core,runtime,cli,converter}/Cargo.toml`、`Cargo.lock`(追跡。CI は `--locked`)
- **Build Dependencies**: cli → {core, runtime, converter}、runtime → core。4 クレートの版は 0.10.1。v0.10.1 で `Cargo.lock` が変わったのは 4 クレートの版だけ(外部の依存は増減なし)

### APIs Discovered
- コマンドライン — `crates/cli/src/args.rs` — 8 コマンド(`run`、`pipeline`、`watch`、`reset`、`validate`、`graph`、`init`、`convert`)と `--help`・`--version`
- 定義ファイル(TOML)— `crates/core/src/definition/` — `[pipeline]` 9 キー、`[[job]]` 9 キー、誤り 34 種
- `.env` — `crates/core/src/envfile/` — 誤り 14 種
- ステートファイル(TOML)— `crates/runtime/src/state/` — 最上位 10 キー、ジョブ 5 キー、試行 2 キー
- ログのファイル — `crates/runtime/src/log.rs`
- ライブラリの公開の口 — 各クレートの `lib.rs`(cli の差し替え口 `Runtime` は 14 メソッド)
- ネットワークの API は無い

### Frameworks & Libraries
- `toml` — 1.2.1 — 定義・ステートの読み書き、変換
- `thiserror` — 2.0.21 — エラー型
- `lexopt` — 0.3.2 — 引数の字句の分割
- `saphyr-parser` — 0.3.4 — YAML の読み取り
- `signal-hook` — 0.4.6 — UNIX のシグナルの監視
- `ctrlc` — 3.5.2 — Windows の Ctrl+C
- `libc` — 0.2.191 — UNIX の `kill`
- `tempfile` — 3.27.0 — 開発時(テストの一時ディレクトリ)

### Test Coverage
- **Test Directories**: `crates/{core,runtime,cli,converter}/tests/`、`crates/cli/src/tests/`、`crates/converter/src/tests/`、各モジュールの `#[cfg(test)] mod tests`
- **Test Frameworks**: Rust 標準の `#[test]` だけ
- **Coverage Config**: CI の `coverage` ジョブ(`cargo llvm-cov --fail-under-lines 80`。テスト専用の補助バイナリを除く)。設定ファイルは無い

### Code Quality Indicators
- **Linting**: `rustfmt`・`clippy -D warnings`(設定ファイルなし。CI の `lint`)。本体の `#[allow]` は `runtime/src/platform/unix.rs:20` の `unsafe_code` だけ。テストの補助 6 つに `#![allow(dead_code)]`
- **CI/CD**: `.github/workflows/ci.yml`、`release.yml`、`audit.yml`
- **Documentation**: `README.md`(955 行)、`demo/README.md`(69 行)、`docs/spec/`(7 版 × 13 文書)、各モジュールの `//!` が規則の番号(BR/NFR/FR、U1〜U7、v0.10.1 の BR)と結ぶ

### Technical Debt Signals
- `TODO`/`FIXME`/`HACK` は 0 件。
- 仕様書 `docs/spec/rust/` が v0.10.0 の機能をほぼ書いていない(下の「仕様書と実装の差」)。この intent の主対象。
- `crates/runtime/src/lib.rs:72-82` の「呼び出しの順序」は、v0.10.0 の「試行を始めて `Fast` で書く」を反映していない(初期ステートを `Fast` で書く、のまま)。v0.10.1 で直したのは契約の表の `run_job` の行(`unset`)だけ。
- `team.md` のテストの数が古い(下の「テストの数」)。
- 使い方の表が 3 か所(`args.rs`、`report/messages.rs`、`README.md`)にある。仕様書の `--help` の全文(08 の 8 章)を含めると 4 か所。

## 仕様書と実装の差

凡例: 「無」= 記述が無い、「古」= v0.9.x の形のまま書いてあり実装と食い違う、「誤」= 数や名前が実装と合わない。行番号は `docs/spec/rust/` の各ファイルの行。実装の根拠は末尾の「実装の要点」。

### 機能ごとの記述の有無(`docs/spec/rust/`)

| 機能(版) | 記述 | 根拠(検索の結果) |
|------------|------|--------------------|
| `watch --follow` / `--interval`(v0.10.0) | 無(13 文書のどこにも無い) | `follow` は 08:340 の `.env` の英文の 1 語だけ。`interval` は retry の間隔だけ |
| `[pipeline] log_dir`(v0.10.0) | 無 | `log_dir`・`LogTarget`・`log.rs` の一致は 0 件 |
| `unset_env`(v0.10.0) | 無。しかも「取り除かない」と逆を書く | 00:75、01:64、05:634、07:205、12:182 |
| `[pipeline.retry]` と `retry = false`(v0.10.0) | 無 | 03:175 は `retry` を「テーブル、既定 None」とだけ書く |
| `env_file` の表 `{ path, required }`(無くてもよい `.env`。v0.10.0) | 無。型は「文字列、または文字列の配列」のまま | 03:139、03:173、03:322、02:80、02:82、03:588 |
| `graph --format dot`(v0.10.0) | 無。`text|mermaid` のまま | 01:30、04:23、04:41、04:74、04:108、04:209、05:729、08:500・515・536・550 |
| ステートの `[[job.attempt]]`(`started_at`、`ended_at`。v0.10.0) | 無 | 06 に `attempt`・`started_at` の一致は 0 件。06:51-58 のジョブのキーは 4 個 |
| `watch` の所要時間の列(v0.10.0) | 無。列は 3 つのまま | 08:399-418 |
| 時刻(`Timestamp`、RFC 3339、詰めた形、`clock::now`。v0.10.0) | 無 | `Timestamp`・`clock`・`time.rs` の一致は 0 件 |
| `UnsetEnvConflict`(定義の誤り。v0.10.0) | 誤: 数は 34 種と書くが一覧は 33 行 | 02:103 だけが名前を書く。03:523 の見出しは 34 種、表(03:527-561)は 33 行。10:128 の列挙も 33 個 |
| `UnsetConflict`(`.env` の誤り。v0.10.0) | 誤: 13 種のまま | 02:106、02:298、03:9、03:391、03:563、10:12、10:121、11:213、11:291、12:107、12:161、12:321 |
| ステートの誤りの理由 3 つ(`NO_ATTEMPT`、`BAD_TIME`、`UNFINISHED_ATTEMPT`) | 無 | 10:142-170 の表に無い。06:60-67 の矛盾の条件にも無い |
| ログの表示(`LOG_NOT_CREATED`、`LOG_WRITE_FAILED`、`LOG_OUTPUT_CUT_SHORT`) | 無 | 08 に「ログ」の一致は 0 件 |
| v0.10.1 の内部の入口(`decide_at`、`classify_and_decide_at`、`SimulationMarks`、`apply_*_in`、`record_at`、`position_of`) | 無 | 一致は 0 件。05:869-900(16.2 `plan_pipeline`)は公開の入口だけで書く |
| 版の記述(v0.10.1) | 一部だけ直してある | 00:3・10、03:1、04:1・250・256、09:1、11:23-28・542、12:13・272 は 0.10.1。02:3 は「v0.9.0 の機能を含む」 |

### 章ごとの差

| 章 | 差(行) | 種別 |
|----|----------|------|
| 00-README | 3・10 は「v0.10.1(v0.9.0 の機能を含む)」で、v0.10.0 の機能を挙げない。31 の 11 章の要約は「場面 0〜14」(実装は 0〜15)。32 の 12 章の要約は「段 0〜13」(v0.10.0 の段が無い)。38 の「特定の挙動を確かめる」に v0.10.0 の機能の行き先が無い。75 の用語「環境変数の層」は「足すだけ(取り除かない)」。84 の耐久性の用語は試行の `Fast` に触れない | 無・古 |
| 01-overview | 16-32 のユースケースに、`watch --follow`、ログのファイル、`unset_env`、既定のリトライ、DOT が無い。23 は `[job.retry]` だけ。30 は `text\|mermaid`。64 の非目標が「ジョブの出力の保存」「取り除く書き方 `unset` はない」を挙げる(どちらも v0.10.0 で実装した) | 無・古 |
| 02-architecture | 3 は「v0.9.0 の機能を含む」。50-72 の core の関数の表に `start_attempt`・`end_attempt`・`cancel_attempt`・`pipeline_activity`・時刻の関数・`check_unset_conflicts` が無い。80・82 の `PipelineDefinition`・`JobDefinition` に `log_dir`・`unset_env` が無く、`env_file: Vec<String>` のまま(実装は `Vec<EnvFileRef>`)。`JobState` の `attempts`、`Attempt` が無い。103 は 34 種(正しい)。106 は 13 種。111 の runtime の再公開に `now`・`LogTarget`・`LogOutcome`・`open_log_file`・`safe_log_stem` が無い。121 の `run_job` は `(line, env, signal) -> JobExecution`(実装は `(line, env, unset, signal, log) -> (JobExecution, LogOutcome)`)。168 の cli の型に `follow`・`interval`・`GraphFormat::Dot` が無い。192-243 のシーケンスに試行と `Fast` の書き込みが無い。266 の `Runtime` の trait に `now` が無い(実装は 14 メソッド)。290-320 の core のモジュールの表に `time.rs`、`flow/attempt.rs`、`flow/activity.rs` が無い。321-341 の runtime の表に `log.rs`、`clock.rs` が無い。381 の report に `graph_dot.rs` が無い | 無・古 |
| 03-definition-format | 9-10 の前置きに v0.10.0 のキーが無い。6.1(131-138)の `[pipeline]` の表は 6 キー(実装は 9: `retry`、`log_dir`、`unset_env` が無い)。6.2(166-175)の `[[job]]` の表は 8 キー(`unset_env` が無い)。175 の `retry` は表だけ(`false` が無い)。6.5(230-279)に `[pipeline.retry]`、既定の当てはめ、`retry = false`、既定との重なりの誤りが無い。6.6(280-328)の `env_file` は「文字列、または文字列の配列」で、表 `{ path, required }` と、その誤りの文(`ENV_FILE_TYPE_REASON` など)が無い。`unset_env` の検査の節が無い。6.7(391-413)の `.env` の誤りは 13 種(実装は 14。`unsetConflict` が無い)で、`required = false` の無いファイルの読み飛ばしが無い。7(415-454)の検査の順序に `retry`(パイプライン)・`log_dir`・`unset_env` と重なりの検査が無い。9.3(523-563)は見出しが 34 種で表が 33 行(`unsetEnvConflict` が無い)。563 は 13 種。11(578-618)のデータモデルに `log_dir`・`unset_env`・`EnvFileRef` が無い。13(628-726)の例に v0.10.0 のキーが無い | 無・誤 |
| 04-cli | 1 は v0.10.1。2(17-27)の構文の `watch` に `--follow [--interval]` が無く、`graph` は `text\|mermaid`。29 の版の注記に v0.10.0 が無い。3(33-47)のオプションの表に `--follow`・`--interval` の行と、`watch` の列の可否が無い。41・74・108 の `--format` は `text`・`mermaid` だけ。7.1(158)・7.2 は「初期ステートを `Fast` で書き、結果を 1 回 `Durable` で書く」(実装は試行ごとに `Fast`)。7.3(174-176)の `watch` は 1 回だけの表示で、`--follow` の動き(終わりの条件、描き方、65・70)が無い。7.7(201-214)に `dot` が無い。9 の実行例に v0.10.0 の例が無い | 無・古 |
| 05-execution-flow | 37 の起動の手順は `run_job(line, env, signal)`。5.1(257-273)の前処理に `unset_env` の重なりの検査(`check_unset_conflicts`)が無い。5.1a(275-307)の共通の手順に `start_attempt`・`end_attempt`・`cancel_attempt` と時刻が無い。7(452-561)のリトライは `[job.retry]` だけで、`[pipeline.retry]` と `retry = false` が無い。7.5(514-536)の実行ループに試行の時刻と `pending_end` が無く、531 は「ステートは最終の結果で 1 回だけ書かれ、途中では書かれない」(v0.10.0 で変わった)。14(632-726)は「取り除く書き方(`unset`)はない」(634)、層の図(652)に `unset_env` が無く、14.4(697-701)は `run_job(line, env, signal)` で `env_remove` が無い。14.5 に `required = false` が無い。15(727-847)は text と Mermaid だけで、DOT の節が無い。16.2(869-900)に v0.10.1 の内部の入口(位置と目印)が無い。`watch --follow` の終わりの判定(`pipeline_activity`)の節が無い | 無・古 |
| 06-state-file | 2.2(34-49)の最上位のキーは正しい(10 個)。2.3(51-58)のジョブのキーは 4 個(`attempt` が無い)。2.4(60-67)の矛盾の条件に「動いたジョブに試行が無い」「最後でない試行に終わりが無い」「時刻の書式の誤り」が無い。2.5(69-160)の例に `[[job.attempt]]` が無い。v0.9.9 までのステートが 65 になる旨が無い。3(167-181)の書き込みの契機に「試行を始めたとき(`Fast`)」が無く、171 は初期ステートだけを `Fast` とする | 無・古 |
| 07-process-and-signals | 4.1(181-196)の `run_job` は `(line, env, signal) -> JobExecution` で、消す名前とログの指定が無い。188 に `env_remove` が無い。4.2(205)は「すべて引き継ぐ。足すだけ」。206 は「標準入出力はすべて引き継ぐ(`Stdio` を指定しない)。出力を読まない・保存しない」(`log_dir` があればパイプにして読む)。読み取りのスレッド、2 秒の待ち、「止める」印(v0.10.1 で読む前にも見る)が無い。7.5(369-377)の待機は「リトライの待機」だけで、`watch --follow` の待ちが無い。8.1(381-394)の OS 差に、0600、`env_remove` の比べ方、画面の消去が無い | 無・古 |
| 08-output-and-i18n | 3 の実装のファイルに `graph_dot.rs` が無い。4.12(314-358)の `.env` の誤りは 13 種。ログの 3 つの文が無い。6(393-437)の `watch` の表は 3 列(実装は 4 列。`DURATION`/`所要時間` が無い)。`watch --follow` の描き方(`ESC[H ESC[2J`、空の 1 行)が無い。8(485-563)の `--help` の全文は、`watch` の行(497・533)に `--follow` が無く、`--format <text\|mermaid>`(500・515・536・550)で、`--follow`・`--interval` の行が無い(実装は `messages.rs:78-136`)。11.5(1203-1229)は Mermaid だけで、DOT の例が無い。12(1379-1453)のひな形は、実装と一致するか未照合(`init_template.rs` は v0.10.0 で変わっていない) | 無・古 |
| 09-converter | 1 は v0.10.1。変換は v0.10.0 で変わっていない。差は見つからない | 差なし |
| 10-errors-and-exit-codes | 1(9-20)の 0 に `watch --follow` の終わり(完了・止まった・消えた・中断)が無い。12 の 64 に `unsetEnvConflict`・`unsetConflict` が無く、`.env` は 13 種。13 の 65 に `watch --follow` の読めないの 3 回が無い。19 の 127 に「ログのファイルを作れず起動しなかった」が無い。121 は 13 種。128 の列挙は 33 個。138-170 の理由の表に試行の理由 3 つが無い | 無・誤 |
| 11-quality-and-build | 23-28 は 0.10.1。4.2(198-299)のテストの一覧に、core の `attempt_flow.rs`・`pipeline_activity.rs`・`time_format.rs`、runtime の `log_file.rs`・`runner_log.rs`、cli の `binary_log.rs`、`src/tests/graph_dot.rs` が無い。213 は `EnvFileErrorKind` 13 種。263 の `flow_aux.rs` に `--follow` の試験が無い。9(439-516)のデモは場面 0〜14(実装は 0〜15。場面 15 はログのファイル、定義 `log.toml`)で、場面 4 の `watch --follow` と `DEMO_PAUSE` の待ち(v0.10.1)が無い。9.5 の `unix_demo.rs` の見出しの範囲は `=== 14.` まで | 無・古 |
| 12-rebuild-guide | 13 は 0.10.1。1(5-142)の段は 0〜13 で、v0.10.0 の機能の段(時刻と試行、既定のリトライ、`env_file` の表、`unset_env`、`log_dir`、DOT、`watch --follow`)が無い。105 は「33 種」(v0.9.0 の段の時点の数としては正しい)。182 は「足すだけ」。321 は `.env` 13 種 | 無・古 |

### `docs/spec/language-agnostic/` の差(軽く見た範囲)

- 00:3・10 と 01:3・02:3 は「v0.10.1 までの挙動」を名乗るが、`log_dir`・`unset_env`・`[pipeline.retry]`・`retry = false`・`[[job.attempt]]`・所要時間・`--follow` の一致は 13 文書で 0 件。`dot` は 11:924 の無関係の 1 語だけ。
- 誤りの数は 33 種と 13 種(00:24、03:499、03:688、10:147、12:38、12:248)。v0.9.x の時点の数としては一貫しているが、v0.10.0 の実装(34 種、14 種)とは合わない。rust 版は v0.10.1 で「34 種」に直したが、この版は直していない(コミット `134a9cf` は rust 版だけ)。
- ほかの 5 版(csharp、go、powershell、python、typescript)は冒頭が v0.8.0/v0.9.0 で、`log_dir` の一致は 0 件(版の扱いが rust・language-agnostic と違う)。

### 実装の要点(差の根拠)

- 定義のキー: `crates/core/src/definition/walk/mod.rs:47-70`(`PIPELINE_KEYS` 9、`JOB_KEYS` 9)。`log_dir` は空なら `emptyField`(`mod.rs:359-367`)。`unset_env` の検査と重なり(`walk/env.rs:107-208`、理由の文 `env.rs:266-281`)。`env_file` の表 `{ path, required }`(`env.rs:210-262`。`required` の既定は `true`)。`[pipeline.retry]` と `retry = false`(`walk/retry.rs:20-120`。理由の文 `retry.rs:43-50`)。
- 誤りの数: `DefinitionErrorKind` 34(`crates/core/src/error.rs:82` が `UnsetEnvConflict`)、`EnvFileErrorKind` 14(`crates/core/src/envfile/mod.rs:89` が `UnsetConflict`)。
- ステート: `crates/runtime/src/state/reader.rs:192-208`(最上位 10、ジョブ 5、試行 2 のキー)、`walk_job`・`walk_attempts`(動いたジョブに試行が無ければ `NO_ATTEMPT`、最後でない試行に終わりが無ければ `UNFINISHED_ATTEMPT`、時刻の誤りは `BAD_TIME`。理由の文は `crates/runtime/src/error.rs:124-128`)。
- 書き込みの時機: `crates/cli/src/step.rs:178-296`(試行ごとに `now` → `start_attempt` → `Fast` で書く → 起動 → `end_attempt`/`cancel_attempt`、リトライの間は `pending_end`)。
- CLI: `crates/cli/src/args.rs:23-36`・`193-199`・`529-534`・`608`・`616-628`(`--follow`、`--interval` 1〜3600、`dot`)。既定の間隔は `types.rs:215`(2 秒)。使い方の全文は `report/messages.rs:78-136`。`watch` の 4 列は `report/watch.rs:16-161`(`HEADER_DURATION` は `messages.rs:551`)。ログの文は `messages.rs:501-508`。
- `watch --follow`: `crates/cli/src/aux_commands.rs:95-172`(`follow_watch`、画面の消去の判定は 142 行)、終わりの判定は `crates/core/src/flow/activity.rs:38-63`。
- ログ: `crates/runtime/src/log.rs:82-105`(`safe_log_stem`。v0.10.1 で予約名の比較の前後の空白を除く、90 行)、`log.rs:134`(`open_log_file`)、`crates/runtime/src/runner.rs:315`・`324`(「止める」印を読む前と読んだ後に見る。v0.10.1)。
- v0.10.1 の内部の入口: `crates/core/src/flow/mod.rs:23-185`(`record_at`、`position_of`、`SimulationMarks`)、`flow/decide.rs:128`(`decide_at`)、`flow/next_step.rs:67`(`classify_and_decide_at`)、`flow/record.rs:110`・`189`(`apply_outcome_in`・`apply_skip_in`)、`plan/simulate.rs:23-37`・`77-106`・`269-282`。どれも `pub(crate)` で、公開の口(`lib.rs`)は変わっていない。

## テストの数

数え方(読むだけのコマンド):

- 統合テストのファイル: `find crates/<crate>/tests -maxdepth 1 -type f -name '*.rs'`。`tests/` の直下の `.rs` だけを数え、補助のディレクトリ(`common/`、`graph_common/`、`bin_support/`、`signal_support/`、`fixtures/`)は含めない。
- `src/tests/` のファイル: `find crates/<crate>/src/tests -name '*.rs'`。直下だけ(`-maxdepth 1`)と、`common/` を含む全部の 2 通りで数えた。どちらも `mod.rs` を含む。
- `#[test]`: `grep -rh --include='*.rs' '^[[:space:]]*#\[test\][[:space:]]*$' crates/<crate>`。行に `#[test]` だけがあるものを数えた。`grep -o '#\[test\]'` で文字列の中を含めて数えても同じ値(文書の中の `#[test]` の書き込みは無い)。`target/` は `crates/` の外にあるため含まれない。`#[ignore]` つきは 1 件(`converter/tests/nfr_security.rs:190`)で、数に含めた。補助のディレクトリと `src/bin/` に `#[test]` は無い。

| 項目 | `team.md` の値 | 今の値(`55958fd`) | 差 |
|------|----------------|---------------------|----|
| core の統合テストのファイル | 33 | 36 | +3(`attempt_flow.rs`、`pipeline_activity.rs`、`time_format.rs`。v0.10.0) |
| runtime の統合テストのファイル | 14 | 16 | +2(`log_file.rs`、`runner_log.rs`。v0.10.0) |
| cli の統合テストのファイル | 25 | 26 | +1(`binary_log.rs`。v0.10.0) |
| converter の統合テストのファイル | 2 | 2 | 0 |
| `crates/cli/src/tests/` のファイル | 11 | 直下 10(`mod.rs` を含む。テストを持つのは 9)、`common/` を含む全部 12 | 全部で数えると +1(`graph_dot.rs`。v0.10.0)。`team.md` の 11 は全部で数えた値と読める(推論) |
| `crates/converter/src/tests/` のファイル | 4 | 4(`mod.rs` を含む。テストを持つのは 3) | 0 |
| `#[test]`(core) | — | 472(`tests/` 352、`src/` 120) | 前回の知識ベースの 467 から +5(v0.10.1) |
| `#[test]`(runtime) | — | 259(`tests/` 171、`src/` 88) | 前回の 257 から +2(v0.10.1) |
| `#[test]`(cli) | — | 613(`tests/` 408、`src/` 205。うち `src/tests/` 203) | 前回の 612 から +1(v0.10.1) |
| `#[test]`(converter) | — | 116(`tests/` 20、`src/` 96。うち `src/tests/` 45) | 前回と同じ |
| `#[test]` の合計 | 1,170 | 1,460 | +290 |

- `team.md` の数は v0.10.0 の前(v0.9.x)の時点の数と読める(推論)。前回の知識ベース(v0.10.1 の intent)は 1,452 と書いた。

## Handoff Summary
- **Intent-relevant finding**: `docs/spec/rust/` は、版の表記(00:3・10、03:1、04:1、11:23-28 など)と「34 種」(02:103、03:523、10:120、12:22)だけを v0.10.1 に直してあり、v0.10.0 の機能(`watch --follow`/`--interval`、`log_dir`、`unset_env`、`[pipeline.retry]`・`retry = false`、`env_file` の表、`graph --format dot`、`[[job.attempt]]`、`watch` の所要時間)は 13 文書のどこにも書いていない。`unset_env` については 00:75・01:64・05:634・07:205・12:182 が「取り除かない」と逆を書き、ログについては 01:64・07:206 が「出力を保存しない・読まない」と書く。数の誤りは 3 つ: 03:523 の見出し 34 種に表が 33 行、`.env` の誤り 13 種(実装 14)の 12 か所、ステートの誤りの理由の表(10:142-170)に試行の 3 つが無い。`docs/spec/language-agnostic/` も v0.10.0 の機能を持たず、数は 33・13 のまま。テストの数は、統合テストのファイルが core 36・runtime 16・cli 26・converter 2、`src/tests/` が cli 12(直下 10)・converter 4、`#[test]` が 1,460。
- **Risks / follow-up**: (1) 仕様書の対象を rust 版だけにするか、language-agnostic も含めるかを要件で決める必要がある。language-agnostic は受け入れ基準(AC 番号)の体系を持ち、直す量が多い。(2) 追記は 12 章の段の追加、02 の型と関数の表、03 の 4 つの節、04・05・06・07・08・10・11 の多くの節に及ぶ。小さな単位でのコミット(team.md)に合わせ、章ごと・機能ごとに分ける必要がある。(3) `crates/cli/src/tests/docs.rs` は README の語句を検査するが、`docs/spec/` を読むテストと CI は無い(`crates/` と `.github/` で `docs/spec` の一致は 0 件)。仕様書の直しはテストで守られない。(4) `crates/runtime/src/lib.rs:72-82` の「呼び出しの順序」も v0.10.0 の書き込みの時機とずれる(コードの文書の直しで、仕様書の範囲の外)。(5) `team.md` は読むだけで直していない。直す値は上の表。数え方(`src/tests/` を直下で数えるか全部で数えるか)を人に確かめる必要がある。
