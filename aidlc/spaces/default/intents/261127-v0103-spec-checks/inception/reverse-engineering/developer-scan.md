# 開発者のコードの走査(v0103-spec-checks)

- 日付: 2026-11-28
- コミット: `8c5c21e5e2d8ad08768891a8381ec4594a044d14`(版 0.10.2)
- 走査の幅: 全体の再スキャン(Full rescan、`./`)。テスト・ビルド・プログラムは走らせていない。数は `grep`・`find`・`git rev-list`・`git tag`・`awk` で数えた(読むだけ)。

## Developer Code Scan Results

### Scan Coverage
- **Analyzed deeply**:
  - `Cargo.toml`、`Cargo.lock`、`crates/*/Cargo.toml`
  - `crates/core/src/`(`error.rs`、`envfile/mod.rs`、`definition/walk/mod.rs`、`definition/walk/retry.rs` を行の単位で。ほかは前回の知識ベースと突き合わせ、`55958fd..HEAD` で変更が無いことを `git diff --stat` で確かめた)
  - `crates/runtime/src/`(`lib.rs` の契約と呼び出しの順序、`state/reader.rs` のキーの表、`error.rs` の理由の文)
  - `crates/cli/src/`(`types.rs` の `CommandKind::ALL`・`--interval` の定数・`GraphFormat`、`report/mod.rs` の `DisplayEvent`、`deps.rs` の `Runtime`)
  - `crates/cli/src/tests/docs.rs`、`crates/cli/tests/msrv_consistency.rs`、`crates/cli/tests/unix_demo.rs`、`crates/cli/tests/windows_demo.rs`(関数の一覧)、`crates/cli/tests/nfr_source.rs`(関数の一覧)、`crates/core/tests/common/kinds.rs`、`crates/core/tests/env_types.rs`、`crates/core/tests/*_perf.rs`(上限の値)
  - `docs/spec/rust/`、`docs/spec/language-agnostic/`(全 26 章。数の記述、11 章の性能の表と受け入れテストの表、12 章の段の見出しを行の単位で)
  - `demo/run-demo.sh`、`demo/windows/run-demo.cmd`(場面の見出し)、`demo/README.md`
  - `.github/workflows/ci.yml`、`release.yml`、`audit.yml`
  - `aidlc/spaces/default/memory/team.md`(読むだけ。A-2 の事実のため。直していない)
- **Skimmed only**:
  - `docs/spec/csharp/`、`docs/spec/go/`、`docs/spec/powershell/`、`docs/spec/python/`、`docs/spec/typescript/`(ファイルの数と冒頭の版だけ)
  - `crates/converter/src/`(前回から変更なし。構成だけ)
  - `crates/*/tests/fixtures/`、`demo/scripts/`、`demo/windows/scripts/`(構成だけ)
  - `README.md`(必須の語句と版の記述だけ)
- **Left out**:
  - `target/`(ビルド出力。`.gitignore`)
  - `local-docs/`(個人の参照文書。`.gitignore`)
  - `.idea/`(IDE の設定。追跡はされているが、What to Skip の IDE の設定)
  - `.claude/`、`aidlc/`(AI-DLC の導入物と作業場。`team.md` だけ例外として読んだ)、`aidlc.settings.local.json`(`.gitignore`)

### Packages Found
- `conductor-core` — lib — Rust — 定義の読み込みと検証、純粋な判断・記録・時刻・図・計画、`.env` の解析
- `conductor-runtime` — lib + テスト専用の bin — Rust — ジョブの起動、シグナルの転送、ログ、時計、ステートの読み書き
- `conductor-converter` — lib — Rust — YAML と TOML の相互変換
- `conductor-cli` — lib + bin `conductor` + テスト専用の bin — Rust — 引数、流れ、表示(英日)、終了コード

### Build System
- **Type**: Cargo ワークスペース(`resolver = "3"`、edition 2024、`rust-version = "1.100"`、版 0.10.2)
- **Config Files**: `Cargo.toml`、`Cargo.lock`、`crates/{core,runtime,converter,cli}/Cargo.toml`
- **Build Dependencies**: cli → core・runtime・converter、runtime → core。core と converter はほかのクレートに依存しない。

### APIs Discovered
- コマンドライン — `crates/cli/src/args.rs` — 8 コマンド(`CommandKind::ALL`)と `--help`・`--version`
- 定義ファイル(TOML) — `crates/core/src/definition/` — 根 3 キー、`[pipeline]` 9 キー、`[[job]]` 9 キー、`[job.retry]` 5 キー、遷移 2 キー
- ステートファイル(TOML) — `crates/runtime/src/state/` — 最上位 10 キー、ジョブ 5 キー、試行 2 キー
- ライブラリの公開の口 — 各クレートの `lib.rs`(ネットワークの API は無い)

### Frameworks & Libraries
- `toml` — 1.2.1 — 定義・ステートの読み書き
- `thiserror` — 2.0.21 — エラー型
- `lexopt` — 0.3.2 — 引数の字句
- `saphyr-parser` — 0.3.4 — YAML の読み取り
- `signal-hook` — 0.4.6 — UNIX のシグナル
- `ctrlc` — 3.5.2 — Windows の Ctrl+C
- `libc` — 0.2(ロック 0.2.191) — UNIX の `kill`
- `tempfile` — 3.27.0 — テスト(開発時)

### Test Coverage
- **Test Directories**: `crates/core/tests/`(36)、`crates/runtime/tests/`(16)、`crates/cli/tests/`(26)、`crates/converter/tests/`(2)、`crates/cli/src/tests/`(12)、`crates/converter/src/tests/`(4)。`#[test]` は 1,460(core 472、runtime 259、cli 613、converter 116)
- **Test Frameworks**: Rust 標準の `#[test]` だけ
- **Coverage Config**: 有り(`ci.yml` の `coverage`。`cargo llvm-cov --fail-under-lines 80`、テスト専用の補助バイナリを除く)

### Code Quality Indicators
- **Linting**: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`(設定ファイルなし。`ci.yml` の `lint`)
- **CI/CD**: `.github/workflows/ci.yml`(lint、build-check、test、msrv-test、coverage、docs-check、startup-bench)、`release.yml`、`audit.yml`
- **Documentation**: `README.md`(955 行)、`demo/README.md`(69 行)、`docs/spec/`(7 版 × 13 文書)。モジュールの `//!` は日本語で規則の番号つき

### Technical Debt Signals
- `docs/spec/` を読むテストも CI のジョブも無い(下の 1)。仕様書の数は人の確かめだけに頼る。
- `crates/core/tests/env_types.rs:55` のテスト名が `定義の誤りの種類は_33_種で表示が空でない` のまま(本文のコメントは 34 種。数は `tests/common/kinds.rs` の `[D; 34]` が保つ)。
- 語の数を取り出せる公開の配列は `CommandKind::ALL` だけ。`DefinitionErrorKind`・`EnvFileErrorKind` の一覧は、テスト側の写し(`crates/core/tests/common/kinds.rs`)にある。定義とステートのキーの表は、どれも private の `const`(下の 2)。
- `TODO`/`FIXME`/`HACK` は無い。本体の `#[allow]` は `runtime/src/platform/unix.rs:20` の 1 つ(テストの補助に `#![allow(dead_code)]` が 6 つ)。

## 1. 今ある文書のテスト

`docs/spec/` を読むテストは **無い**。`docs/spec` という文字列は、`crates/` のどの `.rs` にも現れない。CI(`ci.yml`)にも仕様書を見るジョブは無い。文書と実装を結ぶテストは次のとおり。

| ファイル | 読むもの | 確かめること(比べる値) |
|----------|----------|--------------------------|
| `crates/cli/src/tests/docs.rs`(cli の単体テスト。`crate::` の非公開の項目を使える) | `README.md`(`CARGO_MANIFEST_DIR/../../README.md`、CRLF を LF にそろえる。補助 `readme()`) | (a) 必須の語句: `assert_readme_has` で語の一覧を `contains`。`DOCS_CHECK_WORDS`(7 語。`ci.yml` の `docs-check` と同じ語と順)を `assert_docs_check_words` で各節の後に確かめる。(b) 版の表記の見出し(`(v0.8.0 以降)`、`(v0.9.0 以降)`、`(v0.10.0 以降)`)。(c) 例を実際の処理に通す: `block_after_heading(見出し, "toml")` で取り出した例を `load_definition`/`parse_definition` に通し、値(`[75, 111]`、`max = 5`、`log_dir = "logs"` など)を比べる。`.env` の例を `parse_env_file` に通す。(d) 例と実際の出力の一致: `graph` の text の図 7 例(`render_text`)、`--dry-run` の計画(`render_plan`)、`init` のひな形(`render_template`)を、1 文字ずつ `assert_eq!`。(e) ログの 3 つの文(`messages::LOG_*` の `ja`・`en`)が README にそのままあること。(f) `watch` の使い方の行が `messages::USAGE`(英日)と README の「## 使い方」の `text` の例で揃うこと(`u7_nfr1_10_…`) |
| `crates/cli/tests/msrv_consistency.rs`(統合テスト) | `Cargo.toml`、`README.md`、`.github/workflows/ci.yml`(補助 `repo_file`) | `Cargo.toml` の `rust-version` の値(`cargo_msrv()`)が、README の `Rust({msrv} 以降)`、`ci.yml` の `dtolnay/rust-toolchain@{msrv}.0`(`stable` 以外の全部)、`ci.yml` のコメント `rust-version = {msrv})` と `最小の Rust {msrv} でも` に一致する。3 つの関数 |
| `crates/cli/tests/unix_demo.rs`(`#![cfg(unix)]`) | `demo/run-demo.sh` を `sh` で実行した出力 | 見出し `=== 0. ` から `=== 15. ` の各場面(`for n in 0..=15`)と要点の行、終了コード 0。数は文字列のリテラル(15)で持つ。仕様書は読まない |
| `crates/cli/tests/windows_demo.rs` | `demo/windows/` の定義とスクリプトを写して本物の `conductor` で実行 | 場面 1〜15 の終了コードと結果。仕様書は読まない |
| `crates/cli/tests/nfr_source.rs`、`crates/runtime/tests/nfr_source.rs`、`crates/converter/tests/nfr_security.rs` | 各クレートの `src/` の `.rs` | ソースの文字列の検査(`unsafe`、`cfg(unix)`/`cfg(windows)` の位置、ネットワークのコード、`#![forbid(unsafe_code)]`、`lib.rs` の最後が `#[cfg(test)]\nmod tests;`)。**ソースを読んで検査する前例**で、文書ではない |
| `crates/core/tests/env_types.rs` + `crates/core/tests/common/kinds.rs` | (ファイルは読まない) | テスト側の写し `DEFINITION_ERROR_KINDS: [D; 34]`・`ENV_FILE_ERROR_KINDS: [E; 14]` と、`_` を使わない網羅の `match`(`definition_position`・`env_file_position`)を `const _: ()` で突き合わせ、変種を足すとコンパイルが止まる。種類の文の表 `DEFINITION_KIND_TEXTS: [_; 34]`・`ENV_FILE_KIND_TEXTS: [_; 14]` |
| CI `docs-check`(`ci.yml:120-133`) | `README.md` | 7 語の `grep -q`(`SHA-256`、`Gatekeeper`、`終了コードの表`、`ステートファイルの場所`、`at-least-once`、`シグナル`、`convert`) |

A-3 のテストを置く場所の手がかり:

- 仕様書を読むなら、`docs.rs` と同じく `CARGO_MANIFEST_DIR` からの相対パスでリポジトリの根を求める。`msrv_consistency.rs` の `repo_file` がそのまま使える形。
- 定義の誤りと `.env` の誤りの数は、`crates/core/tests/` に置けば `common::kinds` の配列の長さ(34・14)を使える。cli に置くなら、core のテストの補助は見えない。
- `cli/src/tests/`(単体)からは `crate::types::{MIN_FOLLOW_INTERVAL_SECS, MAX_FOLLOW_INTERVAL_SECS, DEFAULT_FOLLOW_INTERVAL_SECS}`(`pub(crate)`)、`CommandKind::ALL`、`GraphFormat` が使える。core・runtime の private の `const`(`PIPELINE_KEYS`、`TOP_KEYS` など)は、どのテストからも見えない。

## 2. 仕様書の中の数の記述

行は `8c5c21e` の各ファイルの行。`R` = `docs/spec/rust/`、`L` = `docs/spec/language-agnostic/`。

### 2.1 数と、実装で取り出せる場所

| 数 | 実装の場所(取り出し方) | 仕様書の記述(ファイル:行と書き方) |
|----|---------------------------|--------------------------------------|
| 定義の誤り 34 種 | `crates/core/src/error.rs:34-83` の `DefinitionErrorKind`(公開の列挙。`ALL` は無い)。テスト側の写し `crates/core/tests/common/kinds.rs:26` の `DEFINITION_ERROR_KINDS: [D; 34]` | R00:23「34 種のエラーの種類」、R02:113「34 種」、R02:322「(34 種)」、R03:644「### 9.3 種類の一覧(34 種)」(表 34 行)、R08:126-166 の 4.3 の表 34 行と R08:167「(34 種。…)」、R10:121「(34 種)」、R10:129 の識別子の並び(34 個。行末「34 種。」)、R11:219「34 種」、R12:22・24・169・413「34 種」。L00:24、L03:513「第 9 節の 34 種」、L03:814「### 9.3 種類の一覧(34 種)」(表 34 行)、L08:128-168 の表 34 行と L08:169、L10:140「34 種」、L10:142 の識別子の並び(34 個)、L12:46・47・193・267・349 |
| `.env` の誤り 14 種 | `crates/core/src/envfile/mod.rs:74-90` の `EnvFileErrorKind`。写し `common/kinds.rs:114` の `[E; 14]` | R00:23、R02:116、R02:323、R03:415「#### `.env` の誤りの種類(14 種。…)」(表 14 行)、R08:347「(14 種。…)」、R10:122(識別子 14 個)、R11:219・303、R12:169・244・413。L00:24、L03:582(表 14 行)、L08:365、L10:148(識別子 14 個)、L12:193・267・349 |
| `[pipeline]` のキー 9 | `crates/core/src/definition/walk/mod.rs:48-58` の `PIPELINE_KEYS`(private `&[&str]`) | R03:144「上の 9 つ(`PIPELINE_KEYS`)」(直前の表 9 行)、R12:150・188(足すキーの名前)。L03:312「上の 9 つ」(表 9 行) |
| `[[job]]` のキー 9 | 同 `walk/mod.rs:59-69` の `JOB_KEYS`(private) | R03:184「上の 9 つ(`JOB_KEYS`)」(表 9 行)。L03:348「上の 9 つ」(表 9 行) |
| `[job.retry]` のキー 5 | `walk/retry.rs:34-40` の `RETRY_KEYS`(private) | R03:239「次の 5 つだけ」(表 5 行)。L03:405 |
| 根のキー 3・遷移のキー 2 | `walk/mod.rs:47` の `ROOT_KEYS`、`:70` の `TRANSITION_KEYS`(private) | 数の語は無く、表だけ |
| ステートの最上位のキー 10 | `crates/runtime/src/state/reader.rs:192` の `TOP_KEYS: [&str; 10]`(private) | R06:34-47「### 2.2 最上位のキー」の表 10 行(数の語は無い)。L06:36-49 の表 10 行 |
| ステートのジョブのキー 5 | `reader.rs:205` の `JOB_KEYS: [&str; 5]`(private) | R06:61「上の 5 つ(`JOB_KEYS`)」、R12:200「`JOB_KEYS` 5 個」。L06:63「上の 5 つ」、L12:224「ジョブの項目 5 個」 |
| 試行のキー 2 | `reader.rs:208` の `ATTEMPT_KEYS: [&str; 2]`(private) | R06:70「上の 2 つ(`ATTEMPT_KEYS`)」、R12:200「`ATTEMPT_KEYS` 2 個」。L06:72「上の 2 つ」、L12:224「試行の項目 2 個」 |
| 試行の誤りの理由 3 つ | `crates/runtime/src/error.rs:124-128` の `reason::NO_ATTEMPT`・`BAD_TIME`・`UNFINISHED_ATTEMPT`(`pub(crate) mod reason`) | R10:168-170 の 3 行(数の語は無い)、R12:200 の 3 つの名前。L12:224「試行の 3 つの理由」 |
| `watch` の列 4 | `crates/cli/src/report/watch.rs`(見出し `JOB STATUS DURATION NOTE`。`messages.rs` の見出しの文) | R08:425「(v0.10.0 以降は 4 列。…3 列…)」、R08:470・481 の例。L00:29「`watch` の表(4 列)」、L08:491 の例 |
| `--interval` 1〜3600、既定 2 秒 | `crates/cli/src/types.rs:215-219` の `DEFAULT_FOLLOW_INTERVAL_SECS = 2`・`MIN_FOLLOW_INTERVAL_SECS = 1`・`MAX_FOLLOW_INTERVAL_SECS = 3600`(`pub(crate)`)、`args.rs:625` | R04:47「秒(1〜3600 の整数)」、R04:82「1〜3600 の整数の秒…」と既定の列「2 秒」、R04:186「(既定 2 秒)」、R08:554・592(`--help` の全文の「1 to 3600」「default: 2」「1〜3600」「省略時: 2」)、R10:236「1〜3600」。L04:48・88(同じ形) |
| `--format` の値 3 つ | `crates/cli/src/types.rs` の `GraphFormat { Text, Mermaid, Dot }`、`args.rs:608` | R04:112「`text`・`mermaid`・`dot`」(数の語は無い)。L04:130 |
| コマンド 8 | `crates/cli/src/types.rs:68` の `CommandKind::ALL: [CommandKind; 8]`(公開) | R04:107「8 コマンド」、R02:184 の列挙。L04:125「8 コマンド」 |
| 使い方の誤り 8 種 | `crates/cli/src/types.rs` の `UsageErrorKind`(8 変種) | R10:222「(8 種)」。L10:232「8 種」、L11:784(AC-348)「8 種類」 |
| ステートの誤り 6 種 | `crates/runtime/src/error.rs` の `StateErrorKind`(6 変種) | R10:139「(6 種)」。L10:156 |
| command の誤り 2 種 | `runtime/src/error.rs` の `CommandErrorKind`(2) | R10:137。L10:154 |
| 変換の誤り 6 種 | `crates/converter/src/error.rs` の `ConversionErrorKind`(6) | R02:157、R09:303、R10:181。L09:400、L10:194 |
| ファイルの作成の誤り 5 種 | `runtime/src/file_create.rs` の `CreateError`(5) | R10:142。L06:616、L10:150 |
| 食い違い 3 種 | `crates/core/src/consistency.rs` の `MismatchKind`(3) | R10:141。L10:158 |
| デモの場面 0〜15 | `demo/run-demo.sh:69-277` の `scene "0. …"`〜`scene "15. …"`(16 個)、`demo/windows/run-demo.cmd:67-330` の `call :scene` 16 個。テストは `unix_demo.rs` の `0..=15` | R00:31「場面 0〜15」、R11:509-524 の場面の表、R11:533「場面 1〜15」、R12:216。L00:32、L11:11「場面 0〜15」、L11:1226(AC-424)「`=== 0. ` から `=== 15. `」、L11:1245(AC-518)「場面 11〜15」、L12:355 |
| 12 章の段 0〜21 | 実装に対応する値は無い(仕様書の中の構成)。R12・L12 の `### 段 N.` の見出しが 0〜21 の 22 個 | R00:32「段 0〜21。…段 10〜13、…段 14〜20、…段 21」、R12:9-218 の見出し、R12:101・146。L00:33、L12:32 の表、L12:34-242 の見出し、L12:125・170 |

### 2.2 突き合わせで気をつけること

- 古い版の数を、版の注記つきで残している箇所がある。数の検査は、これらを誤りとして拾わないように書く必要がある: R12:105「(33 種)」(段 10 の時点)、L12:129「33 種。v0.9.0 の段の時点の数で…」、R08:425「3 列」(v0.9.x まで)、R04:107・L04:125「6 コマンド」(v0.8.0 以前)、R11:533・L11:1226「1〜14」「`=== 14. `」(v0.9.x まで)、R12:60「理由文は…(22 種)」(段 5 の時点。今の `reason` の状態の文は 25、command の 2 を合わせて 27)。
- `DefinitionErrorKind` の識別子は、変種の名前の先頭を小文字にしたもの(`error.rs:31` のドキュメントコメント)。R10:129・L10:142 の識別子の並びは、変種の宣言の順と同じ。
- 数の語が無く、表の行の数だけで数を表す箇所がある(ステートの最上位 10、試行の理由 3、`--format` の 3)。行を数えるなら、表の範囲(見出しから空行まで)を決めて数える形になる。
- 数の取り出し口が private の `const` のもの(定義のキー、ステートのキー、理由の文)は、テストから値を取れない。取り出すには (a) ソースの文字列を読む(`nfr_source.rs` の前例)、(b) 動きで数える(全部のキーを書いた定義が通り、1 つ足すと `unknownKey`)、(c) 公開の口を足す、のどれか。(c) は `project.md` の NEVER「テストの都合で本番のコードを曲げない」に当たりうる。

## 3. A-1 の事実

### 3.1 11 章(language-agnostic)の根拠の列のテスト名の書き方

- 決まりの文: L11:26「受け入れテストの根拠は、参照実装のテストの名前を、各項目の『根拠』の欄に併記した」、L11:235「根拠は、参照実装のテストの名前(`ファイル名::テスト名`、日本語の名前の `_` は空白に当たる)」。
- 根拠の列の `…::…` の形の名前は 743 個(AC の行 580)。そのうち空白で書いたものが 588、空白の無いもの 155。空白の無いもののうち、日本語の語を `_` でつないだもの(書き方が不揃いのもの)は **14 個** で、次の 14 の AC にある: AC-097a(L11:387)、AC-206・207・209・212a・216・217・221・229・230・231・232・233・234(L11:599-628。4.7 `convert` の節に集中)。残りの空白の無い名前は、ファイル名だけ(`core::nfr_security` など)か、日本語の 1 語の名前(`args_grammar::引数がない`)。
- 実例(`_` のまま): `flow_validate::空白だけのコマンド名は_64_で_ジョブを起動しない`(AC-097a)、`convert_roundtrip::ac_7_1_1_toml_から_yaml`、`convert_ac::入力の形式は変換後の反対で_拡張子は見ない`、`read_resolve::読み取りの深さの上限_32_と_33`、`read_resolve::null_の書き方と_ほかの表せない種類 / toml_の日時は表せない`(空白と `_` が混ざる)。
- 実例(空白にしたもの): `flow_validate::v は標準エラーに 1 行だけ出す`(AC-002。実際は `v_は標準エラーに_1_行だけ出す` と読める)、`flow_run::br3_1 log_dir があれば 定義のディレクトリで解決した指定を試行の時刻とともに渡す`(実際の関数は `crates/cli/tests/flow_run.rs:518` の `br3_1_log_dir_があれば_定義のディレクトリで解決した指定を試行の時刻とともに渡す`)、`binary_basic::validate は正しい定義で 0…`(`…` で切った形。実際は `binary_basic.rs:98`)。
- 傾向: 空白が大多数(588 対 14)。空白にする形は機械的ではない。`br3_1`・`log_dir`・`u6_br1_1`・`run_id` のような ASCII の識別子の中の `_` は残し、語の区切りの `_` だけを空白にしている(空白の形から元の名前へは一意に戻らない)。末尾を `…` で切った名前、`同上`、`派生(…)`、`/` で 2 つ並べた名前も混ざる。
- `docs/spec/rust/11-quality-and-build.md` の根拠の列は、テストのファイル名(`core/tests/flow_perf.rs` など)で書き、関数の名前は書かない。

### 3.2 AC-430 と、実際のテスト

- 仕様(L11:1232): AC-430「UNIX 系: 利用者の環境に左右されない」の前提は `CONDUCTOR_RUN_ID=zzz`、`DEMO_RUN_ID=zzz`、**`LC_ALL=ja_JP.UTF-8`、`LC_MESSAGES=ja_JP.UTF-8`**、`NO_COLOR=1`。期待は「出力に `zzz` が現れない。英語の表示(`DEMO_LANG` のとおり)。結果が上と同じ」。根拠は「同上」(= `unix_demo`)。
- 実際のテスト(`crates/cli/tests/unix_demo.rs:236-251` の `利用者の環境の_run_id_と_ロケールに左右されない`): **`LC_ALL=C`、`LC_MESSAGES=C`**、ほかは仕様と同じ。`run_demo` は `DEMO_LANG=en_US.UTF-8` を渡す。確かめるのは `assert_demo_output`(英語の要点の行を含む)と、標準出力に `zzz` が無いこと。
- デモのスクリプト(`demo/run-demo.sh:35-36`)は `unset LC_ALL LC_MESSAGES` で消してから `DEMO_LANG` を `LANG` に入れる。
- 差の意味: `LC_ALL=C` は英語なので、スクリプトが `LC_ALL` を消さなくても英語の出力になり、「消すこと」をこのテストは見分けられない。仕様の `ja_JP.UTF-8` なら、消さなければ日本語になり、見分けられる。直す向きは 2 つ(仕様を `C` にする、またはテストを `ja_JP.UTF-8` にする)。後者はテストを強くする変更で、本番のコードは変わらない。
- 同じ表のほかの言語の記述(R11:477、L11:1142「`LC_ALL`、`LC_MESSAGES` は外す」)は実装と合う。

### 3.3 AC-573 と、性能の表の「強制」「目安」

- L11:54(2.1 の表の行): 計画の模擬の基準「参照実装の基準(開発用の版、5 回の中央値): 150 ms / 150 ms(v0.10.1 でも変えていない)」、扱い **目安**、受け入れテスト AC-573。
- L11:913(AC-573): 期待の中で「どちらも 150 ms 以内(**目安。参照実装は開発用の版で強制として検査する**。2.1 節)」。根拠は `plan_simulate_perf::p1 …`・`p2 …`・`flow_dry_run::…`、区分: 計測。L05:949 も「【目安】」、L02:198 も「目安」、L12:247 も「目安の 150 ms 以内」。
- 参照実装: `crates/core/tests/plan_simulate_perf.rs:39` の `const LIMIT: Duration = Duration::from_millis(150)`、`:105`・`:136` の `assert!(m < LIMIT, …)`(5 回の中央値)。CI の `test`・`msrv-test`・`coverage` で普通に走るため、超えれば不合格(強制)。
- R11:118(rust 版の 3.1 の表): 同じ項目を **強制** と書く(「5 回の中央値が 150 ms / 150 ms 以内(debug)」)。
- L11 の 2.1 の表の「扱い」の列の使われ方:
  - **強制**: AC-403(CPU 時間)、AC-404(同時の 2 つの `run`)、AC-275(待機の中断 1 秒)、AC-486(図の生成 1 秒)、AC-501(計画の生成 1 秒)、AC-540(DOT 1 秒)、AC-548(ログの中身)、AC-571(`watch --follow` の 1,000 回)。参照実装でも `assert!` で落とすもの。
  - **目安**: AC-397〜402(起動の測定。参照実装も「計測のみ」で、`STARTUP_BENCH_ENFORCE=true` のときだけ強制)、待ちの間隔の 10 ms、そして「参照実装の基準(同)」と書く v0.9.7〜v0.9.8 の 6 行(重ね合わせ 350 ms、辺 600 ms、`.env` の重複キー 100 ms、`env` の表 350 ms、遷移の重複 1,600 ms、`env_file` の一覧 40 ms。どれも受け入れテストは「—」)と AC-573。
  - この「参照実装の基準(同)」の 6 行も、参照実装では `assert!` で強制している(`graph_lookup_perf.rs:33・35`、`env_parse_perf.rs:33`、`definition_perf.rs:35・37`、`env_refs_perf.rs:31`)。R11:119-124 はこれらを **強制** と書く。つまり language-agnostic 版は「debug の版の時間の上限は実装と機械に依るので、再実装には目安」という使い方で、AC-573 だけでなくこの 6 行も同じ扱い。AC-573 を「強制」にするなら、6 行との揃いも決める必要がある。
  - L11 の 2.1 の表には、R11:116-117 の `flow_perf.rs`(50 ms)と `env_merge_perf.rs`(50 ms / 100 ms)の行が無い。`definition_perf.rs:42` の P3(700 ms)は R11・L11 のどちらの表にも無い。

## 4. A-2 の事実(`aidlc/spaces/default/memory/team.md`。読むだけ)

数え方: `git rev-list --count HEAD`(1,088)、`git rev-list --count --merges HEAD`(0)、`git log --format=%an | sort -u`(1 人)、`git tag | sort -V`(21 個)、`git branch -r`(`origin/main` だけ)。いずれも `8c5c21e` の時点。

| 行 | 今の記述 | 実際の値 | 種類 |
|----|----------|----------|------|
| 11 | 「867 コミットすべてが非マージ、作者は 1 人」 | 1,088 コミット、マージ 0、作者 1 人 | 数が古い(性質は今も正しい) |
| 14 | 「タグは `v0.1.0` から `v0.9.9` まで付いている」 | `v0.1.0`〜`v0.10.2`(21 個。v0.10.0・v0.10.1・v0.10.2 は `main` の上) | 範囲が古い |
| 15 | 「機能を足す intent は版の 2 桁目を上げ(v0.8.0、v0.9.0)、整理・修正の intent は 3 桁目を上げる(v0.9.1 から v0.9.9)。v0.10.0 の intent は、7 つの機能を作り終えてから版を 0.10.0 にする」 | 例は v0.10.0(機能)、v0.10.1・v0.10.2(整理)まで増えた。後ろの文は v0.10.0 の intent だけの決まりで、終わっている | 例が古い / 1 つの intent だけの決まり |
| 24 | 「v0.10.0 では B1〜B7 を 1 つずつ作る。…」 | v0.10.0 の intent だけの決まり(終わった) | 1 つの intent だけの決まり |
| 25 | 「各機能の最後に README とデモを更新し、7 つを作り終えてから版を 0.10.0 にする」 | 前半(各機能の最後に README とデモ)は一般の慣行として残せる。後半は v0.10.0 だけ | 一部が 1 つの intent だけの決まり |
| 32 | テストの数(core 36、runtime 16、cli 26、converter 2、`src/tests/` 12・4、`#[test]` 1,460) | 同じ(数え直して一致) | 古くない |
| 39 | 「v0.10.0 の機能にも、test-after と 80% の下限、…を続ける」 | 方針は 29・35 行と同じで、v0.10.0 に限る必要が無い | 1 つの intent だけの言い回し |
| 40 | 「ステートの中身を調べるテスト(`nfr_content.rs`)は、B6 のために開始と終了の時刻だけを許すように直す」 | 直し終わっている(66 行の Code Style と `nfr_content.rs` が今の決まり) | 済んだ作業の指示 |
| 55 | 「署名・公証、Intel macOS の動作確認、`audit` の強制、`startup-bench` の強制は、v0.10.0 でも見送る」 | 今も見送っている(`release.yml` に署名なし、`x86_64-apple-darwin` は確認なし、`audit.yml` は警告のみ、`ci.yml:140` の `STARTUP_BENCH_ENFORCE: "false"`)。「v0.10.0 でも」が古い | 版の限定が古い |
| 60 | 「lint の抑止(`#[allow]`)は `platform/unix.rs` の `unsafe_code` の 1 つだけ」 | 本体では正しい。テストの補助に `#![allow(dead_code)]` が 6 つ(`core/tests/common/mod.rs:17`、`core/tests/graph_common/mod.rs:18`、`runtime/tests/common/mod.rs:16`、`cli/tests/common/mod.rs:16`、`cli/tests/bin_support/mod.rs:16`、`cli/tests/signal_support/mod.rs:17`) | 範囲の書き方があいまい(推論) |
| 68 | 「v0.10.0 の機能でも、新しい外部ライブラリ(依存)を追加しない。時刻の書式、DOT の整形、`watch` の表示し直しは自前で書く」 | 依存は増えていない(`git diff v0.9.9 HEAD -- Cargo.lock` の差は 4 クレートの版の 8 行だけで、パッケージの増減は無い)。「v0.10.0 の機能でも」が古い。また、この行の後に空行が無く、すぐ `## Forbidden` が続く | 版の限定が古い / 書式 |

ほかの行(10、12、13、16〜18、22、23、29〜31、33〜38、48〜54、59、61〜67)は、今の実装と CI に合う。

## Handoff Summary
- **Intent-relevant finding**: 仕様書(`docs/spec/`)を読むテストは無い(`crates/` の `.rs` に `docs/spec` の文字列が 0 件)。文書のテストは README(`crates/cli/src/tests/docs.rs`)と MSRV(`crates/cli/tests/msrv_consistency.rs`)だけで、どちらも `CARGO_MANIFEST_DIR/../..` からリポジトリのファイルを読む形。A-3 の数のうち、定義の誤り 34・`.env` の誤り 14 はテスト側の写し(`crates/core/tests/common/kinds.rs:26`・`:114`)で、コマンド 8 は `CommandKind::ALL`(`crates/cli/src/types.rs:68`)で、`--interval` は `types.rs:215-219` の `pub(crate)` の定数で取れる。定義のキー(`walk/mod.rs:48`・`:59`)とステートのキー(`state/reader.rs:192`・`:205`・`:208`)は private の `const` で、テストから値を取れない。A-1 は、根拠の列の `_` のままの名前が 14 個(AC-097a と 4.7 の 13 個)、AC-430 は仕様の `LC_ALL=ja_JP.UTF-8` に対してテストは `LC_ALL=C`(`unix_demo.rs:243`)、AC-573 は language-agnostic 版が「目安」、rust 版と参照実装(`plan_simulate_perf.rs:39`・`:105`・`:136`)が「強制」。
- **Risks / follow-up**:
  - 数の検査は、版の注記つきの古い数(R12:105 と L12:129 の 33 種、R08:425 の 3 列、R04:107 と L04:125 の 6 コマンド、R11:533 と L11:1226 の 14 まで、R12:60 の 22 種)を誤りと見なさない書き方が要る。
  - private の `const` を数えるために公開の口を足すのは、`project.md` の NEVER(テストの都合で本番を曲げない)に当たりうる。ソースの文字列を読む(`nfr_source.rs` の前例)か、動きで数えるかを設計で決める。
  - AC-573 を「強制」に直すなら、L11 の 2.1 の表の「参照実装の基準(同)」の 6 行(参照実装はどれも強制)も同じ扱いにするかを決める。
  - AC-430 は、仕様を `C` に合わせるか、テストを `ja_JP.UTF-8` にして強くするかを決める(テストを変えても本番のコードは変わらない)。
  - `crates/core/tests/env_types.rs:55` のテスト名が「33 種」のまま。直すならテストの名前の変更だけ。
  - 版を 0.10.3 にするときの範囲: 4 つの `Cargo.toml`、`Cargo.lock`(4 か所)、`crates/cli/tests/binary_basic.rs:267`、R00:3・10、R03:1、R04:1・268・274、R09:1、R11:23-28・562、R12:13・362、L00:3・10、L01:3、L02:3、L04:306・310、L05:3、L06:3、L07:3、L08:111、L11:3・781、L12:3。
  - `team.md` は、この段では直していない。直すのは学びの作法(規則の書き込みの道具)による。
