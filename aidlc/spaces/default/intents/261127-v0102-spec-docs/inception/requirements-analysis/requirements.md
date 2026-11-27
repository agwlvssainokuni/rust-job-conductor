# 要件: v0.10.2 の仕様書の整理

出典: 依頼の文(`project-description.json`)、要件の質問(`requirements-analysis-questions.md` の Q1〜Q6、まとめの確かめ済み)、コードの知識(`business-overview.md`、`architecture.md`、`code-structure.md`、`code-quality-assessment.md` の「仕様書と実装の差」「テストの数」)、引き渡し(`inception/reverse-engineering/developer-scan.md`)、チームの実践(`team.md`)。

## 1. 意図の分析

- 目的: 仕様書(`docs/spec/rust/`、`docs/spec/language-agnostic/`)を v0.10.1 の実装に合わせる。仕様書だけで作り直せる、という今の仕様書の目的を保つ(依頼の文、Q2 A)。あわせて、`team.md` のテストの数と、runtime の呼び出しの順序のドキュメントコメントを直す(Q3 A、Q5 A)。最後に版を 0.10.2 にする。
- 種類: 文書の整理。`conductor` の動きは変えない。整理の intent なので版の 3 桁目を上げる(team.md の Way of Working)。
- 今の差: v0.10.0 の機能(`watch --follow`/`--interval`、`log_dir`、`unset_env`、`[pipeline.retry]` と `retry = false`、表の `env_file`、`graph --format dot`、`[[job.attempt]]` と所要時間の列、時刻)は、2 つの仕様書のどこにも書かれていない。実装と逆のことを書いた文もある(`code-quality-assessment.md`)。
- 成功の基準: 2 つの仕様書に、対象の機能がすべて今の章と同じ細かさで書かれ、実装と逆の文が無く、誤りの数が実装と一致する。コードの動きは変わらず、既存のテストと CI がすべて通る。

## 2. 機能の要件

### FR1. `docs/spec/rust/` を実装に合わせる(Q1 B、Q2 A)

対象の機能(以下「v0.10 の機能」): v0.10.0 の `watch --follow`/`--interval`、`[pipeline] log_dir`(試行ごとのログのファイル、名前、0600、作れないとき 127 扱い、ログの 3 つの表示)、`unset_env`(パイプラインとジョブ、重なりの検査、`env_remove`)、`[pipeline.retry]` の既定と `retry = false`、`env_file` の表 `{ path, required }`(無くてもよい `.env`)、`graph --format dot`、ステートの `[[job.attempt]]`(`started_at`、`ended_at`)と `watch` の所要時間の列、時刻(UTC の RFC 3339)、v0.10.1 のログの名前の空白・読む前の「止める」印・`flow` の内部の入口(`decide_at`、`classify_and_decide_at`、`SimulationMarks` など)。

- **FR1.1** 00・01: 版の記述を v0.10.2(v0.10.0 の機能を含む)にし、ユースケースと用語に v0.10 の機能を足す。00 の章の要約(11 章の場面、12 章の段)を実装に合わせる。
- **FR1.2** 02: core・runtime・cli の関数と型の表、`Runtime` の trait のメソッド、モジュールの表、シーケンスの図に、v0.10 の機能の関数・型・モジュール(`time.rs`、`flow/attempt.rs`、`flow/activity.rs`、`log.rs`、`clock.rs`、`graph_dot.rs` ほか)と、`run_job` の今の形 `(line, env, unset, signal, log) -> (JobExecution, LogOutcome)` を書く。
- **FR1.3** 03: `[pipeline]`(9 キー)と `[[job]]`(9 キー)の表、`[pipeline.retry]` と `retry = false`、`env_file` の表と理由の文、`unset_env` の検査、`.env` の誤り 14 種、検査の順序、`DefinitionErrorKind` 34 種の一覧(9.3 の表に `unsetEnvConflict` を足す)、データモデル、例を実装に合わせる。
- **FR1.4** 04: 構文、オプションの表(`--follow`、`--interval`、`--format dot`)、書き込みの時機(試行ごとに `Fast`)、`watch --follow` の動き(終わりの条件、描き方、終了コード)、実行例を書く。
- **FR1.5** 05: 起動の手順、前処理(`check_unset_conflicts`)、試行の開始と終わり(`start_attempt`・`end_attempt`・`cancel_attempt`、`pending_end`)、既定のリトライ、`unset_env` の層と `env_remove`、`required = false`、DOT の節、`watch --follow` の終わりの判定(`pipeline_activity`)、v0.10.1 の内部の入口(16.2 の計画の模擬)を書く。
- **FR1.6** 06: ジョブのキー(5 個)、`[[job.attempt]]`、矛盾の条件(試行が無い、最後でない試行に終わりが無い、時刻の書式の誤り)、例、v0.9.9 までのステートが 65 になること、書き込みの契機を書く。
- **FR1.7** 07: `run_job` の形、`env_remove`、`log_dir` があるときのパイプと読み取りのスレッド、2 秒の待ち、「止める」印(読む前と読んだ後)、`watch --follow` の待ち、OS の差(0600、名前の比べ方、画面の消去)を書く。
- **FR1.8** 08: `watch` の 4 列の表(`DURATION`/`所要時間`)と描き方(UNIX 系の端末で `ESC[H ESC[2J`、ほかは空の 1 行で追記)、`--help` の全文(`report/messages.rs` と一致)、ログの 3 つの文、`.env` の誤り 14 種、DOT の例を書く。
- **FR1.9** 10: 0・64・65・127 に v0.10 の場合を足し、誤りの列挙を 34 種・14 種にし、ステートの誤りの理由に試行の 3 つ(`NO_ATTEMPT`、`BAD_TIME`、`UNFINISHED_ATTEMPT`)を足す。
- **FR1.10** 11: テストの一覧に v0.10.0〜v0.10.1 のテストのファイルを足し、デモを場面 0〜15(場面 4 の `watch --follow` と `DEMO_PAUSE` の待ち、場面 15 のログのファイル)にし、`.env` の誤りの数を 14 種にする。
- **FR1.11** 12: v0.10 の機能を作り直す段を足す(時刻と試行、既定のリトライ、`env_file` の表、`unset_env`、`log_dir`、DOT、`watch --follow`、v0.10.1 の直し)。履歴の段の、その時点の数(105 行の 33 種など)は残す。
- **FR1.12** 実装と逆のことを書いた文(00:75、01:64、05:531、05:634、07:205-206、12:182 ほか、`code-quality-assessment.md` の一覧の「古」)は、書き足しではなく書き換える(依頼の文)。
- 09(変換)は差が無いので、版の記述のほかは変えない。

### FR2. `docs/spec/language-agnostic/` を実装に合わせる(Q1 B、Q6 A)

- **FR2.1** FR1 と同じ v0.10 の機能を、language-agnostic の今の章と同じ細かさで、言語に依らない言い方(型名やモジュール名ではなく、挙動と形)で書く。v0.10.1 の内部の入口は、挙動に表れない作りの事柄なので、計画の模擬の結果が実行の判断と一致すること(と性能の目安)として書く。
- **FR2.2** 受け入れの条件に、今の体系の続きの新しい AC の番号を振る。既存の AC の番号は変えない。
- **FR2.3** 誤りの数を 34 種(定義)・14 種(`.env`)に直す(00:24、03:499、03:688、10:147、12:38、12:248 ほか)。
- **FR2.4** 「v0.10.1 までの挙動」の記述を、v0.10.2 の版に合わせる。

### FR3. `team.md` のテストの数(Q3 A、Q4 A)

- **FR3.1** `team.md` の Testing Posture の「テストの種類」の 1 行の数だけを、今の値に書き換える: 統合テストのファイル core 36、runtime 16、cli 26、converter 2。`crates/cli/src/tests/` 12、`crates/converter/src/tests/` 4。`#[test]` 合計 1,460(コード生成の時点で数え直し、違えばその値)。数え方を括弧で添える(統合テストは `tests/` の直下の `.rs`、`src/tests/` は `mod.rs` と補助を含むすべての `.rs`、`#[test]` は行の数)。
- **FR3.2** その行の他の文と、`team.md` の他の節は変えない。書き換えは、コード生成の計画の承認(人の確かめ)を経て行う。

### FR4. runtime の呼び出しの順序(Q5 A)

- **FR4.1** `crates/runtime/src/lib.rs` の「呼び出しの順序」(72〜82 行あたり)を、v0.10.0 の後の形(試行の開始で `Fast` で書く、`run_job` の今の引数、ログ、試行の終わり)に直す。ドキュメントコメントだけで、コードは変えない。

### FR5. 版

- **FR5.1** すべてを片づけてから、`crates/cli/Cargo.toml` ほかの版を 0.10.2 にする(v0.10.1 の 152c4a1 と同じ 22 ファイルの範囲)。件名は「版を 0.10.2 にする」の 1 コミット。

## 3. 非機能の要件

- **NFR1 動きを変えない**: `crates/` の変更は、ドキュメントコメント(FR4.1)、版の更新(FR5.1)、版の期待のテスト(`binary_basic.rs`)だけ。既存のテストがすべて通る。
- **NFR2 仕様書の正しさ**: 書いた内容は実装の該当の行と一致する。数(キー、誤りの種類、列、場面、段)と文(理由の文、表示の文、`--help`)は実装から写し、推測で書かない。合否: レビューで、各 FR の主な事柄を実装の行と照らして確かめる。
- **NFR3 実装と逆の文が無い**: `grep` で、`取り除かない`・`出力を読まない`・`保存しない`・`途中では書かれない`・`足すだけ`・`13 種`・`33 種` などが、履歴として残す箇所の他に残らない。
- **NFR4 3 つの OS と品質の検査**: CI(ubuntu・macos・windows の `test`、`msrv-test`、`lint`、`coverage`、`docs-check`)が通る(project.md の Mandated)。
- **NFR5 依存**: 新しい依存を足さない。

## 4. 制約

- 変更は小さな単位でコミットし、直接 `main` に入れる(team.md)。仕様書は章ごと(またはまとまりごと)にコミットする。
- `team.md` の書き換えは FR3 の 1 行だけ。

## 5. 前提

- 仕様書の検査(テストや CI)は無い。正しさは、レビューと `grep` の確かめに頼る。理由: `docs/spec/` を読むテストも CI も今は無く(`code-quality-assessment.md`)、検査を足すことは依頼に無いため。
- language-agnostic の AC の番号の体系は、11 章(`11-quality-and-build.md`)と 12 章にまとまっている(AC の語の数: 11 章 568、12 章 40)。番号の振り方は設計で決める。
- `#[test]` の数は、コード生成の時点の値を使う(テストは変えないので、1,460 のままの見込み)。

## 6. 範囲の外

- ほかの 5 つの言語版(csharp、go、powershell、python、typescript)の仕様書(Q1 B)。
- `conductor` の動きの変更。README の変更(README は v0.10.0〜v0.10.1 で実装に合わせてある)。
- 仕様書を読むテストや CI の検査を足すこと。
- `team.md` の他の古い記述(例: タグの範囲「v0.1.0 から v0.9.9」)。

## 7. 未決の事項(設計で決める)

- language-agnostic の新しい AC の番号の振り方(章ごとの番号の続きか、機能ごとの番号の帯か)。
- 章ごとの書き足しの位置(新しい節を足すか、今の節に足すか)と、12 章の新しい段の番号。
- 仕様書の分量が大きいので、コード生成の作業をどう分けるか(rust と language-agnostic、章のまとまり)。
