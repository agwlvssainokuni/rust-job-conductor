# 要件: v0.10.1 の整理・修正(F1〜F5)

出典: 依頼の文(`project-description.json`)、v0.10.0 の振り返り(`261125-small-feature-batch/operation/feedback-optimization/feedback-loop.md` の F1〜F5)、チームの実践(`team.md`)、コードの知識(`business-overview.md`、`architecture.md`、`code-structure.md`、`api-documentation.md`、`code-quality-assessment.md`)、要件の質問(`requirements-analysis-questions.md` の Q1〜Q5、まとめの確かめ済み)。

## 1. 意図の分析

- 目的: v0.10.0 の振り返りで残った小さな課題(文書の分かりにくさ、ログの名前の検査の漏れ、揺れる性能のテスト、原因不明の失敗、デモの見え方)を片づけ、版を 0.10.1 にする(依頼の文)。
- 種類: 既存の単体の CLI(`conductor`)の整理・修正。機能は足さない。整理・修正の intent なので版の 3 桁目を上げる(team.md の Way of Working)。
- 範囲: README、runtime(`log.rs`、`runner.rs`、`lib.rs` の文書)、core(計画の模擬、F3 で原因が確かめられたとき)、デモのスクリプト、`docs/spec/rust/` の数の誤り、この intent の記録(code-structure.md)。
- 成功の基準: F1〜F5 と文書のずれ(Q5 A)をすべて片づけ、v0.10.1 として出せる状態にする。チームの基準(test-after、行カバレッジ 80% 以上、3 つの OS と MSRV の CI)を満たす。

## 2. 機能の要件

### FR1. `watch --follow` の注意の書き直し(F1)

- **FR1.1** README の「エラーの後に続くジョブがあるときの注意」を書き直す。どの使い方で起きるか(エラーで止まったステートを `watch --follow` で見ながら、パイプラインを再開するとき)を、「パイプラインを始める前から見ておけます」の使い方と区別して書く(依頼の文、developer-scan の気になる点 1)。
- **FR1.2** 遷移で次のジョブへ続く実行の例(再開の位置のジョブが試行を書く前の、最初の描画で終わる例)を含める(依頼の文)。
- **FR1.3** `conductor` の動き(「止まった」の判定、`crates/core/src/flow/activity.rs`)は変えない。合否: `activity.rs` に差分が無く、README の必須語句の検査(`docs-check`)と `src/tests/docs.rs` が通る。

### FR2. `log_dir` のログの名前と読み取り(F2)

- **FR2.1** ログの名前で、最初の `.` より前の部分を Windows の予約名と比べるとき、前後の空白を除いて比べる(依頼の文)。合否: ジョブ ID `con ` のログの名前の頭が `_con_` になり、`con` は今までどおり `_con`、`console` は今までどおり `console` になる。
- **FR2.2** FR2.1 で名前が変わること(空白を除くと予約名になるジョブ ID だけ)を認める。README の「名前の置き換え」、`crates/runtime/tests/log_file.rs` の名前の表、リリースの注記を揃える(Q1 A)。
- **FR2.3** 出力を写す読み取りのスレッドで、待ち切れの「止める」印を、`read` を呼ぶ前にも見る。印が立っていたら、読まずに終わる(依頼の文、`crates/runtime/src/runner.rs` の `copy_stream`)。今の、読んだ後に見る確かめは残す。合否: 印が立った状態で `copy_stream` を呼ぶと、`read` が 1 回も呼ばれずに戻ることをテストで確かめる。

### FR3. `plan_simulate_perf.rs` の上限の扱い(F3)

- **FR3.1** `crates/core/tests/plan_simulate_perf.rs` の 150ms の上限は変えない(依頼の文)。
- **FR3.2** まず手元で測る。通常の `cargo test` と、カバレッジの計測(`cargo llvm-cov`)の下の両方で、所要時間を記録する(Q2 A)。
- **FR3.3** 計画の模擬がジョブごとに一覧を先頭から探すこと(二乗に近い比較)が遅さの原因と、測って確かめられたときに限り、探し方を直して速くする。計画の結果(出力)は変えない(Q2 A)。合否: 既存の計画のテストがすべて通り、カバレッジの計測の下でも 150ms に余裕をもって収まる。原因が別と分かったときは、直さずに測った結果と分かったことを記録する。

### FR4. 原因不明の失敗の記録の手順(F4)

- **FR4.1** 再び起きたときに残すもの(CI の実行の番号、落ちたテストの名前、出力、OS)と残し方を決めて、この intent の記録に書く。コードもテストも変えない(Q3 A)。

### FR5. デモの場面 4(F5)

- **FR5.1** `demo/run-demo.sh` の場面 4 で、`DEMO_PAUSE=1` のときは、`watch --follow` を呼ぶ前で一度止まり、それまでの出力(再開の出力と 1 回の `watch` の表)を読めるようにする(Q4 A)。
- **FR5.2** `DEMO_PAUSE` が無いとき(テストの `unix_demo.rs` を含む)の出力と動きは変えない。`conductor` の動きは変えない(Q4 A)。

### FR6. 文書のずれ(Q5 A)

- **FR6.1** `crates/runtime/src/lib.rs` の契約の表に、`run_job` の `unset` の引数を書き足す。
- **FR6.2** `docs/spec/rust/` の `DefinitionErrorKind` の数「33 種」を、実装の 34 種(v0.10.0 の `UnsetEnvConflict` を含む)に直す。対象は 02・10・00・12 の各章のうち、エラーの種類の数を指す箇所だけ(入れ子の深さの「33 段」は対象外)。

### FR7. 版

- **FR7.1** すべてを片づけてから、`crates/cli/Cargo.toml` の版を 0.10.1 にする(依頼の文)。件名は「版を 0.10.1 にする」の 1 コミット(team.md の Way of Working)。

## 3. 非機能の要件

- **NFR1 後方互換**: FR2.2 の名前の変化を除き、v0.10.0 と同じに動く。既存のテストはすべて通る。
- **NFR2 テスト**: test-after。コードを変える FR2.1・FR2.3・FR3.3 に単体テストを書く。行カバレッジ 80% 以上を保つ(project.md の Mandated)。
- **NFR3 3 つの OS**: ubuntu・macos・windows の CI と、最小の Rust(1.100)の CI が通る(project.md の Mandated)。
- **NFR4 依存**: 新しい外部ライブラリを加えない(project.md の Forbidden)。
- **NFR5 `unsafe` と `cfg`**: `unsafe` は runtime の `platform/unix.rs` の外に書かない。`cfg(unix)`/`cfg(windows)` は `platform` の外に置かない(project.md の Forbidden)。
- **NFR6 秘密の保護**: エラーの表示に入力の値を含めない(project.md の Forbidden)。
- **NFR7 品質の検査**: `cargo fmt` と `cargo clippy -D warnings` が通る(project.md の Mandated)。
- **NFR8 性能の基準**: 性能のテストの上限は緩めない(FR3.1、org.md の Testing Posture「基準を弱めて通さない」)。

## 4. 制約

- 変更は小さな単位でコミットし、直接 `main` に入れる(team.md の Way of Working)。
- テストの都合で本番のコードを曲げない(project.md の Forbidden)。FR2.3 のテストは、`copy_stream` の今の入口(モジュールの中の単体テスト)で書く。

## 5. 前提

- 空白は、ASCII の空白(` `)を指す。理由: Windows が名前の末尾から落とすのは `.` と空白で、今の `safe_log_stem` の末尾の処理も ASCII の空白だけを扱っているため(`crates/runtime/src/log.rs`)。
- F3 の測定は、開発の主な環境の macOS の手元で行う。理由: CI の機械の速さは選べず、チームはテストを手元の macOS で走らせるため(team.md の Testing Posture)。
- F4 の失敗の候補(実時計を挟んで比べるテスト)は推論であり、この intent では確かめない。

## 6. 範囲の外

- `watch --follow` の「止まった」の判定の変更(FR1.3)。
- `conductor` の画面の消し方の変更(Q4 A)。
- 実時計に頼るテストの見直し(Q3 A)。
- 性能のテストを `coverage` のジョブから外すこと(Q2 A)。
- `docs/spec/rust/` への `follow`・`log_dir`・`unset_env` の記述の書き足し(次の intent に回す。Q5 A)。
- `team.md` のテストの数の更新(チームの決まりの文書のため、学びの手順で人が判断する)。

## 7. 未決の事項(設計で決める)

- FR2.1 で、前だけ・後ろだけでなく前後の両方を除くか、の細部と、`con .txt` のように `.` の前に空白があるときの例(FR2.2 の表に載せる例)。
- FR3.3 で直すときの探し方(定義の位置と記録を、ジョブの ID から引ける形にするなど)。
- FR4.1 の手順を書く場所(この intent の記録のどの文書か)。
