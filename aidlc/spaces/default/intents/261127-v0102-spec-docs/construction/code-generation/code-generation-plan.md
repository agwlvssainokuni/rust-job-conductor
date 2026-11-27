# コード生成の計画: v0.10.2 の仕様書の整理

## Summary

- Builds: `docs/spec/rust/` と `docs/spec/language-agnostic/` を v0.10.1 の実装に合わせる(v0.10.0 の機能の書き足し、実装と逆の記述の書き換え、誤りの数、language-agnostic の AC-519 からの新しい節と既存の AC の書き換え)、`team.md` のテストの数の 1 行、runtime の呼び出しの順序のドキュメントコメント、版を 0.10.2 にする
- Touches: `docs/spec/rust/` の 13 章、`docs/spec/language-agnostic/` の 13 章、`aidlc/spaces/default/memory/team.md` の 1 行、`crates/runtime/src/lib.rs` のドキュメントコメント、版の 22 ファイル
- Tests: 新しいテストは無い(文書の変更)。既存のテストすべて、`cargo doc`、`grep` の確かめ(付録 B)

出典: 機能の設計(`functional-spec.md` の 1〜5 章、`rules.md` の BR1.1〜BR6.3、`entities.md`)、要件(`requirements.md` の FR1.1〜FR5.1、NFR1〜NFR5)、設計の承認で直すと決めたレビューの指摘(R-01〜R-04。付録 A・B に反映)、引き渡し(`inception/reverse-engineering/developer-scan.md` の章ごとの差の表)。Units とユーザーストーリーは無いので、各ステップは要件と規則の ID に対応させる。

テストの進め方は Testing Contract(test-after)に従う。この作業は文書が主で、コードの動きを変えない。各段の後に、その段の確かめ(付録 B のうち関わるもの)と、コードに触れた段では関わるテストを走らせる。

## データの形の変更

- 無い。コードの型・ステートの形・定義の書式は変えない(BR6.1)。

## 設計のレビューからの持ち越し(この段階で扱う)

- **R-01**: language-agnostic の実装と逆の記述も、行ごとに付録 A.2 に挙げ、書き換えを確かめる。付録 B に、逆の記述の語の `grep` を足す。
- **R-02**: 既存の AC の洗い出しの語を広げ(付録 A.3)、AC-146・147・241・305〜308 を名指しで確かめる。
- **R-03**: 「ほか」をやめ、`grep` で見つけた「13 種」「33 種」の行をすべて付録 A.1・A.2 に挙げ、履歴として残す行に印を付ける。
- **R-04**: language-agnostic の 11 章の新しい AC は、機能の節ごとに 1 コミットにする。AC の番号は、Step 13 で節と番号の範囲を先に決めて(付録 C)から、各章で参照する。

## ステップ

### Step 1: 準備

- [x] `unit-test-instructions.md` のコマンドをそのまま走らせ、結果を `code-summary.md` に記録する(変更の前)。
- [x] 付録 B の確かめを変更の前に走らせ、出発点の結果を `code-summary.md` に記録する。

### Step 2〜12: `docs/spec/rust/`(章ごとに 1 コミット。BR1.1〜BR1.3、BR6.2)

各章は `functional-spec.md` の 2 章の手順(W1)で書く。差は引き渡しの「章ごとの差」の表。書き換えは付録 A.1。

- [x] Step 2: 03(定義の書式。FR1.3)
- [x] Step 3: 06(ステート。FR1.6)
- [x] Step 4: 07(プロセスとシグナル。FR1.7)
- [x] Step 5: 05(実行の流れ。FR1.5。v0.10.1 の内部の入口を 16.2 に)
- [x] Step 6: 04(CLI。FR1.4)
- [x] Step 7: 08(出力。FR1.8。`--help` の全文は `crates/cli/src/report/messages.rs` から写す)
- [x] Step 8: 10(誤りと終了コード。FR1.9)
- [x] Step 9: 02(構成。FR1.2)
- [x] Step 10: 11(品質とビルド。FR1.10)
- [x] Step 11: 12(作り直しの手引き。FR1.11。履歴の段は残す)
- [x] Step 12: 00・01(案内と概要。FR1.1)と 09 の版の記述
- [x] rust の付録 A.1 の各行が書き換わり、付録 B の rust の確かめが通ることを確かめる(FR1.12、NFR3)。

### Step 13: language-agnostic の AC の節と番号の範囲を決める(R-04、BR2.2)

- [x] 11 章に足す新しい節(4.21 以降、見出しに「(v0.10.0 以降)」)と、各節の AC の番号の範囲(AC-519 から)を付録 C の形で決め、`code-summary.md` に書く。以降の章は、この範囲の番号を参照する。

### Step 14〜23: `docs/spec/language-agnostic/`(章ごとに 1 コミット。BR2.1、BR2.4、BR1.3)

言語に依らない言い方で書く(型名・モジュール名・関数名を使わない)。書き換えは付録 A.2。

- [x] Step 14: 03
- [x] Step 15: 06
- [x] Step 16: 07
- [x] Step 17: 05(v0.10.1 の内部の入口は「計画の模擬は実行と同じ判断を、1 つの判断あたり定数の手間で行う」と性能の目安として)
- [x] Step 18: 04
- [x] Step 19: 08
- [x] Step 20: 10
- [x] Step 21: 02
- [x] Step 22: 00・01
- [x] Step 23: 09 の版の記述(変換は変わっていない)

### Step 24: language-agnostic 11 章の既存の AC の書き換え(1 コミット。BR2.3、Q2 A)

- [x] 付録 A.3 の語で 11 章の AC の表のすべての行を洗い出し、合わない行の表を `code-summary.md` に残す。少なくとも AC-146・147・170・174・175・241・278・305〜308・355・389・453 と、5 章(デモ)の AC を確かめる。
- [x] 合わない AC は、番号を残して中身を今の実装に書き換え、「(v0.10.0 で変更)」を添え、根拠の列のテストの名前を今のテストの名前に直す(テストの名前は `grep` で実在を確かめる)。
- [x] 11 章の冒頭の表(5 の「場面 0〜14」)と非機能の表(性能の目安)を直す。

### Step 25〜(節の数): language-agnostic 11 章の新しい節(機能の節ごとに 1 コミット。BR2.2、R-04)

- [x] 付録 C の節ごとに、AC を AC-519 からの続きで書く(前提・実行・期待・根拠の列。根拠はテストの名前か区分)。節の数だけコミットを分ける。

### Step 26: language-agnostic 12 章(1 コミット。BR2.2)

- [x] v0.10 の機能の段を足し、段の表に新しい AC の範囲を書く。履歴の段の数は残す。誤りの数(39・121・248 行)を直す。

### Step 27: runtime の呼び出しの順序(1 コミット。BR4.1)

- [ ] `crates/runtime/src/lib.rs` の「# 呼び出しの順序」を `functional-spec.md` の 5 章のとおり直す。コードは変えない。
- [ ] `cargo test -p conductor-runtime --lib --locked` と `cargo doc -p conductor-runtime --no-deps --locked` を走らせる。

### Step 28: team.md のテストの数(1 コミット。BR3.1、FR3)

- [ ] テストの数を数え直す(付録 B の数のコマンド)。
- [ ] `aidlc/spaces/default/memory/team.md` の Testing Posture の「テストの種類」の 1 行の数だけを今の値に置き換え、数え方を括弧で添える。その 1 行の外は変えない。`git diff` でその 1 行だけが変わったことを確かめる。

### Step 29: 全体の確かめ

- [ ] 付録 B の確かめをすべて走らせ、結果を `code-summary.md` に書く。
- [ ] `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked` を走らせる(NFR1)。

### Step 30: 版を 0.10.2 にする(1 コミット。BR5.1)

- [ ] v0.10.1 の 152c4a1 と同じ 22 ファイルの範囲で、版を 0.10.2 にする。件名は「版を 0.10.2 にする」。
- [ ] `cargo test --workspace --locked` を走らせる。

### Step 31: 記録

- [ ] `code-summary.md`(章ごとの変更、付録 B の結果、既存の AC の洗い出しの表、新しい AC の範囲、テストの結果、コミット)と `traceability.json` を書く。

## コミット

各ステップで 1 コミット(仕様書は `docs:`、team.md と記録は `aidlc:`、lib.rs は `runtime:`)。版の更新は「版を 0.10.2 にする」の 1 コミット。push はしない(人が行う)。

## 守ること

- ファイルの作成と編集はファイルの道具だけで行い、シェル(`sed -i`、`python3`、heredoc)で書き換えない。
- 書く事柄は実装の行から写す(BR1.1)。根拠の見つからない事柄は書かず、`code-summary.md` に差として書く。
- 履歴の記述(その時点の数や形)は書き換えない。
- `crates/` は、ドキュメントコメント(Step 27)と版(Step 30)の外を変えない(BR6.1)。新しい依存を足さない。

## 付録 A: 書き換える記述の一覧

### A.1 rust(`docs/spec/rust/`)

| 行 | 今の記述 | 扱い |
|---|---|---|
| `00-README.md:31` | 11 章の要約に「場面 0〜14」 | 0〜15 に |
| `00-README.md:75` | 環境変数の層は「足すだけ(取り除かない)」 | `unset_env` を書く |
| `01-overview.md:64` | 非目標に「ジョブの出力の保存」「`unset` はない」 | 非目標から外し、機能として書く |
| `02-architecture.md:106` | `EnvFileErrorKind` 13 種 | 14 種 |
| `02-architecture.md:121` ほかの `run_job` | `(line, env, signal) -> JobExecution` | `(line, env, unset, signal, log) -> (JobExecution, LogOutcome)` |
| `02-architecture.md:298` | `EnvFileErrorKind` 13 種 | 14 種 |
| `03-definition-format.md:9` | 前置きに v0.10.0 のキーが無い(13 種の語を含むなら 14 種に) | 書き足す |
| `03-definition-format.md:391` | 「`.env` の誤りの種類(13 種)」 | 14 種(`unsetConflict` を足す) |
| `03-definition-format.md` 9.3(523〜561) | 見出し 34 種、表 33 行 | `unsetEnvConflict` を足して 34 行 |
| `03-definition-format.md:563` | `.env` の誤りの説明(13 種を含むなら) | 14 種 |
| `05-execution-flow.md:531` | 「ステートは最終の結果で 1 回だけ書かれ、途中では書かれない」 | 試行の開始で `Fast`、結果で `Durable` |
| `05-execution-flow.md:634` | 「足す」だけ。`unset` はない | `unset_env` と `env_remove` |
| `05-execution-flow.md:652` 付近(層の図) | `unset_env` が無い | 足す |
| `07-process-and-signals.md:205` | 「すべて引き継ぐ。足すだけ」 | 消す名前を除いてから足す |
| `07-process-and-signals.md:206` | 「出力を読まない・保存しない」 | `log_dir` があればパイプで読む |
| `10-errors-and-exit-codes.md:12` | 64 の説明(13 種を含む) | 14 種、`unsetEnvConflict`・`unsetConflict` |
| `10-errors-and-exit-codes.md:121` | `EnvFileErrorKind`(13 種) | 14 種 |
| `11-quality-and-build.md:291` | `report_messages.rs` の説明に `.env` の誤り 13 種 | 14 種 |
| `12-rebuild-guide.md:107` | v0.9.0 の段の `run_job` | **履歴。残す**(v0.9.0 の段の時点の形) |
| `12-rebuild-guide.md:105` | 33 種 | **履歴。残す** |
| `12-rebuild-guide.md:161` | `EnvFileError`(13 種) | 14 種(v0.10.0 で 1 種足したと書く) |
| `12-rebuild-guide.md:182` | 「環境変数は足すだけ(取り除かない)」 | `unset_env` を含めて書く |
| `12-rebuild-guide.md:321` | `.env` の 13 種 | 14 種 |

残してよい記述(書き換えない): `05-execution-flow.md:170`・`277`(手順の説明で、逆ではない)、`05-execution-flow.md:691`(`CONDUCTOR_RUN_ID` を取り除かない。今も正しい)、`03-definition-format.md:326`・`350`・`352`・`370`・`375`(`.env` の読み方)、`04-cli.md:133`・`06-state-file.md:330`(定義を読まない)、`02-architecture.md:242`・`10-errors-and-exit-codes.md:54`(`init` は読まない)、`12-rebuild-guide.md:105`・`107`(履歴)。

### A.2 language-agnostic(`docs/spec/language-agnostic/`)

| 行 | 今の記述 | 扱い |
|---|---|---|
| `00-README.md:24` | 03 章の要約に 33 種・13 種 | 34 種・14 種 |
| `00-README.md:32` | 11 章の要約に「場面 0〜14」(あれば) | 0〜15 |
| `00-README.md:91` | 環境変数の層(足すだけ) | `unset_env` を書く |
| `01-overview.md:67` | 非目標に「ジョブの出力の保存」「`unset` はない」 | 非目標から外し、機能として書く |
| `03-definition-format.md:557` | 「`.env` の誤りの種類(13 種)」 | 14 種 |
| `03-definition-format.md:688` | 「9.3 種類の一覧(33 種)」 | 34 種(表に `unsetEnvConflict`) |
| `03-definition-format.md:9`・`499`・`728` | 13 種・33 種の語を含むなら | 14 種・34 種 |
| `05-execution-flow.md:535` | 「ステートは最終の結果で 1 回だけ書かれ、途中では書かれない」 | 試行の開始で書く |
| `05-execution-flow.md:646` | 「足す」だけ | `unset_env` |
| `07-process-and-signals.md:243` | 「出力を読まない」 | `log_dir` があれば読む |
| `07-process-and-signals.md:471` | 「足す環境変数を…足す(取り除かない)【規定】」 | 消す名前を除いてから足す |
| `10-errors-and-exit-codes.md:12` | 64 の説明(13 種を含む) | 14 種 |
| `10-errors-and-exit-codes.md:139` | 33 種 | 34 種 |
| `10-errors-and-exit-codes.md:147` | `.env` の誤り 13 種 | 14 種 |
| `11-quality-and-build.md:11` | 「場面 0〜14」 | 0〜15 |
| `11-quality-and-build.md:675` | AC-278 | Step 24 で書き換え |
| `12-rebuild-guide.md:38`・`39`・`121`・`248` | 33 種・13 種 | 34 種・14 種(履歴の段の数なら残し、印を付ける) |

残してよい記述: `05-execution-flow.md:700`(`CONDUCTOR_RUN_ID`)、`03-definition-format.md:536`(`export`)、`11-quality-and-build.md:911`(AC-452。今も正しい)。

### A.3 既存の AC の洗い出しの語(R-02)

`watch`、`--follow`、`--interval`、リトライ、ステート、`[[job]]`、試行、時刻、`env`、`env_file`、`unset_env`、`log_dir`、ログ、出力、`graph`、`--format`、デモ、場面、終了コード、`.env` の誤り、定義の誤り、種。

## 付録 B: 確かめ(作業の前と後)

- `grep -rn '33 種\|13 種' docs/spec/rust/ docs/spec/language-agnostic/` の結果が、付録 A で「履歴。残す」とした行だけ。
- `grep -rn '足すだけ\|取り除かない\|取り除く書き方\|読まない・保存しない\|1 回だけ書\|0〜14' docs/spec/rust/ docs/spec/language-agnostic/` の結果が、付録 A の「残してよい記述」の行だけ(R-01)。
- `grep -rln 'log_dir\|unset_env\|--follow\|attempt\|dot' docs/spec/rust/` と `docs/spec/language-agnostic/` が、要件の FR1 の各章(02・03・04・05・06・07・08・10・11・12)を含む。
- 数(`functional-spec.md` の 4 章): `[pipeline]`・`[[job]]` のキー 9・9、誤り 34 種・14 種、ステートのキー 10・5・2、試行の理由 3 つ、`watch` の列 4、`--interval` 1〜3600 と 2 秒、`--format` 3 つ、場面 0〜15。仕様書の該当の記述と一致する。
- テストの数: 各 `crates/*/tests/` の直下の `.rs` の数、`crates/cli/src/tests/` と `crates/converter/src/tests/` のすべての `.rs` の数、`grep -rE '^\s*#\[test\]' crates --include='*.rs'` の行の数。

## 付録 C: language-agnostic 11 章の新しい節(Step 13 で確定する案)

| 節 | 機能 | AC の番号 |
|---|---|---|
| 4.21 | パイプライン共通のリトライの既定と `retry = false`(v0.10.0 以降) | AC-519 から |
| 4.22 | 無くてもよい `.env`(`env_file` の表)(v0.10.0 以降) | 4.21 の続き |
| 4.23 | `unset_env`(v0.10.0 以降) | 続き |
| 4.24 | `graph --format dot`(v0.10.0 以降) | 続き |
| 4.25 | ジョブの出力をファイルに残す(`log_dir`)(v0.10.0 以降。v0.10.1 の名前の空白・読む前の印を含む) | 続き |
| 4.26 | 試行ごとの時刻と `watch` の所要時間(v0.10.0 以降) | 続き |
| 4.27 | `watch --follow`(v0.10.0 以降) | 続き |
| (4.16 の非機能の表) | 計画の模擬の性能(v0.10.1。1,000 ジョブ) | 続き |

各節の番号の範囲は、Step 13 で AC の行を数えてから確定し、`code-summary.md` に書く。

## Testing Contract

```json
{
  "version": 1,
  "methodology": "test-after",
  "source": "team",
  "ordering": "各テスト可能な層を実装してから、その層のテストを書いて実行する。",
  "scope": "refactor",
  "test_strategy": "minimal",
  "project_type": "brownfield",
  "applicable_notes": [
    {
      "layer": "org",
      "text": "We treat tests as a first-class deliverable in every Bolt. The specific\nmethodology (TDD, BDD, ATDD, or classic test-after) is affirmed at\npractices-discovery and recorded in `team.md` under this heading with explicit\n`Methodology` and `Ordering` fields; Code Generation resolves those fields\nindependently from coverage, tooling, and scope notes.\n\nWhen no posture has been affirmed, our default per scope is:\n- **Methodology**: test-after\n- **Ordering**: implement each applicable testable layer, then write and run\n  that layer's tests.\n- `mvp`, `enterprise`, `feature`, `infra` add an 80% line-coverage\n  floor and CI execution before merge.\n- `bugfix`, `security-patch` add a targeted regression for the specific\n  bug/vulnerability and require the existing suite to remain green.\n- `express` uses the Minimal strategy: requirement-driven unit tests (one per\n  requirement, with a happy-path floor per component); existing tests remain\n  green.\n- `poc`, `refactor`, `workshop`, `classic` add no extra new-test floor and require the\n  existing suite to remain green.\n\nThe active `Test Strategy` still applies in every scope and determines test\nvolume/types. Scope floors are additive; they never reduce or replace the\nselected strategy.\n\nBuild and Test verifies defined coverage floors and affirmed quality targets;\nthey may not be weakened to make a step pass.\n\nAffirm a stricter posture in `team.md` if the team commits to one."
    },
    {
      "layer": "team",
      "text": "- **Methodology**: test-after\n- **Methodology evidence**: チームは test-after を続けると決めた(インタビュー Q3)。履歴も整合する。v0.9.0 の U4 では、`create_file` の追加(602d15c)の後にそのテスト(90a27fa)、ひな形の生成(0fbd466)の後にひな形のテスト(5b750f8)が続く。U3 も流れ(a9eb237)の後に流れのテスト(365f45c)と実バイナリのテスト(062c627)が続く。`code-quality-assessment.md` も「チームの方針(test-after、行カバレッジ 80%)と一致する」と記す。\n- **Ordering**: 各テスト可能な層を実装してから、その層のテストを書いて実行する。\n- テストの種類: 各クレートの `tests/` に統合テスト(core 33、runtime 14、cli 25、converter 2 のファイル)。単体テストは各モジュールの `mod tests` と、`crates/cli/src/tests/`(11 ファイル)、`crates/converter/src/tests/`(4 ファイル)。`#[test]` は合計 1,170。\n- 流れのテストは偽の `Runtime`/`Sink`/`Env`/`Signals` を注入する。実バイナリのテストは `conductor` とテスト専用の補助バイナリを動かす。非機能のテスト(`nfr_*.rs`、`*_perf.rs`)、網羅のテスト(`ALL` 配列)、参照出力との一致(`graph_layout_oracle.rs`)、文書との一致(`src/tests/docs.rs`、`msrv_consistency.rs`)、デモのテスト(`unix_demo.rs`、`windows_demo.rs`)を含む。テスト名は日本語。\n- テストの都合で本番のコードを曲げない。テスト専用の引数・`pub`・関数を本番に残さず、流れのテストは公開の入口と差し替え口で書く(インタビュー Q6)。\n- 行カバレッジ 80% を下限とする。CI の `coverage` ジョブが `cargo llvm-cov --fail-under-lines 80` で強制する(テスト専用の補助バイナリは除外)。\n- 主な開発環境の OS は macOS で、テストはローカルの macOS で実行する。他の OS のテストは GitHub Actions の CI(`test`: ubuntu、macos、windows)で確認する。マージ前に CI を通す(インタビュー Q3)。\n- 最小の Rust(1.100.0)でもテストを実行する(`msrv-test`)。\n- 起動の性能(`nfr_perf.rs`)は `startup-bench` で測るが、警告のみで失敗にしない(`STARTUP_BENCH_ENFORCE=false`、`continue-on-error`)。\n- v0.10.0 の機能にも、test-after と 80% の下限、単体テストと実バイナリを動かす統合テストを続ける(インタビュー Q3)。\n- ステートの中身を調べるテスト(`nfr_content.rs`)は、B6 のために開始と終了の時刻だけを許すように直す。それ以外(コマンド・環境変数・出力)は今までどおり検査する(インタビュー Q7)。"
    }
  ],
  "obligations": {
    "strategy": "minimal",
    "strategy_volume": [
      "One verifiable test per requirement at the narrowest effective level.",
      "At least one happy-path unit test per component.",
      "Unit tests are the default; a bugfix/security scope floor may require an integration or E2E regression when that is the narrowest level that reproduces the defect."
    ],
    "scope_floor": [
      "Keep the existing test suite green.",
      "This scope adds no extra new-test floor beyond the selected test strategy."
    ],
    "combination_rule": "Apply every selected-strategy obligation and every scope-floor obligation; neither replaces the other, and a targeted scope regression may add the narrowest necessary test type beyond the strategy default."
  },
  "plan_profile": {
    "methodology": "test-after",
    "runner_step": "Verify the existing test runner/configuration and record the exact unit-scoped command.",
    "runner_ready_before_first_test": true,
    "testable_layers": [
      "Data model / database behavior",
      "Repository / data access",
      "Business logic",
      "API / endpoint",
      "Frontend behavior"
    ],
    "steps": [
      "Project structure and production configuration skeleton.",
      "Verify the existing test runner/configuration and record the exact unit-scoped command.",
      "Data model / database behavior - implement.",
      "Data model / database behavior - write and run its tests after implementation.",
      "Repository / data access - implement.",
      "Repository / data access - write and run its tests after implementation.",
      "Business logic - implement.",
      "Business logic - write and run its tests after implementation.",
      "API / endpoint - implement.",
      "API / endpoint - write and run its tests after implementation.",
      "Frontend behavior - implement.",
      "Frontend behavior - write and run its tests after implementation.",
      "Environment/build configuration.",
      "Documentation and traceability."
    ]
  },
  "input_sha256": "sha256:7e9d968db97062fa027a5e051451ae9a45cf716ab34c770cf5bad3d7158ab724",
  "contract_sha256": "sha256:728ba30aa54828358fc5a5af6a5cb1934005ee548cf618e9d67080e927739be0"
}
```
