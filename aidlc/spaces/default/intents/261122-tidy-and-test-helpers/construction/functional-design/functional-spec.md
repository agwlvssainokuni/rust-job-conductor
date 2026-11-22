# 機能の仕様(Functional Spec): 動きを変えない小さな整理とテストの補助

意図: `ideation/intent-capture/intent-statement.md`(Q1〜Q4 すべて A)。**変える場所の正確な一覧(ファイルと行、目標の形、影響するテスト、使った検索のコマンド)は `inventory.md`**(HEAD af018bc で作った)。この文書は、方針、段ごとの設計、**承認が要るテストの変更(§4 の T 表)**、足すテスト(§5 の N 表)、確かめ方、コミットの並び、範囲の外、決めてほしい点を書く。

## 1. 方針

- 利用者から見える動き(標準出力と標準エラーの文、終了コード、ステートファイル、`--dry-run` の出力)を変えない [desc]。版は上げない。
- 7 つの段を順に進める。段 1(説明のずれと、使われないもの)、段 2(文言の定数)、段 3(処理の重複と境界)、段 4(到達しない分岐と効かない処理)、段 5(文字列での比較)、段 6(walk の整理)、段 7(テストの補助)。各段は小さなコミットの積み重ねで、どのコミットでもビルドとテストが通る [Q1] [memory:M1]。
- 承認は、この設計とコードの計画の 1 回。実装は段の順に進め、各段の終わりにテストの件数を記録し、結果は最後にまとめて見せる [Q1]。
- 新しい依存を足さない [memory:M5]。`unsafe` は runtime の `platform/unix.rs` だけ、`cfg(unix)`/`cfg(windows)` は src の `platform` の外に置かない(テストのファイルの `#![cfg(unix)]` は今のまま)、誤りの表示に入力の値を含めない、converter は core に依存しない、MSRV 1.100。テスト名とドキュメントコメントは日本語。
- 仕様書は `docs/spec/rust` だけを直す(`docs/spec/language-agnostic` は今回は直す必要が無い。凍結した `docs/spec/{go,python,typescript,csharp,powershell}` は触らない)。

## 2. 段ごとの設計(詳細は `inventory.md` の同じ番号)

| 段 | 項目 | 要点 |
|----|------|------|
| 1 | 1a | core `types.rs` の「生成は runtime」を今の姿(状態は core の `flow::record`、`JobExecution` は runtime・cli・core の模擬)に |
| 1 | 1b | `sanitize`・`escape_display` の説明を「本文は上限以内、切ったら `…` を足す(全体は最大で上限 + 1)」に。テスト名は変えず、注記だけ(D1。決定済み) |
| 1 | 1c | `args.rs` の文法の表に `--dry-run`・`--assume`・`--format`・`--state`・`--force` の列 |
| 1 | 1d | 仕様書 08 章の拒否の理由の 3 行を消す。05 章の `OutOfOrder` の 1 か所も |
| 1 | 1e | `Command::program_name` とその単体テスト 1 件を消す。仕様書 02 章の 1 行(Q4) |
| 2 | 2a | cli `messages.rs`: `SKIPPED_LIST` を消す、`SKIPPED_BY_TRANSITION`・`SKIPPED_AFTER_END`・`ALREADY_EXECUTED`・`RESUME_HERE`・`DETAIL_*`・`class_word`・`COMPLETED`、`error_prefix!()` と `concat!` |
| 2 | 2b | core: 環境変数の名前の規則の句をマクロ `env_name_rule!()` に(3 か所の文は 1 文字も変えない) |
| 2 | 2c | core: `reserved_env_prefix!()`。`RESERVED_ENV_PREFIX` と 2 つの文 |
| 2 | 2d | core: `ROOT_KEYS` に `VERSION_KEY` |
| 2 | 2e | core: 間隔の文の数と `MIN_SECS`・`MAX_SECS` を `const _: () = assert!(…)` で結ぶ |
| 2 | 2f | `CONDUCTOR_RUN_ID` の二重はまとめない(意味が別)。注記を足す(D2。決定済み) |
| 3 | 3a | runtime: 中立のモジュール `temp_file`(`dir_or_current`、`temp_prefix`、`temp_name`、`create_temp`、連番 1 つ、`create_temp_beside`)。`file_create`・`platform` が `state` の内部を使わない(D5。決定済み) |
| 3 | 3b | cli: `report::push_line` を 1 つに(graph・plan・watch) |
| 3 | 3c | cli: `job_place(job_id)`(4 か所) |
| 3 | 3d | cli: `messages::reset_hint(with_run_id)`、`StateContext::for_read(with_run_id)`(`read_context` を移す) |
| 3 | 3e | cli: `plan_end` を消し、`plan_text` で 1 回だけ `PlanEnd` を見る(`COMPLETED`、`PLAN_END_REACHED`、`plan_stopped_at_error`) |
| 3 | 3f | cli: `graph_mermaid.rs` の let-else を 1 段に |
| 4 | 4a | core: `decide.rs` の規則 5 のコードを消す(説明の番号は残し、「規則 6 に含まれる」と書く。仕様書 05 章の表にも) |
| 4 | 4b | cli: lexopt の到達しない `MissingValue` の枝を消し、守りの枝を 1 つにまとめて注記(D3。決定済み) |
| 4 | 4c | cli: `render` の `match` を 1 つにし、到達しない `=> String::new()` を消す(言語は必要な枝だけで判定) |
| 4 | 4d | cli: `retry.rs` の既定値 0 を消し、1 つの let-else に |
| 4 | 4e | core: `classify_exit_code(job, code) -> ExitCodeClass { classification, reported_exit_code, conductor_error }` を切り出す(返す終了コードと、`code == 0` の `conductor_error` まで 1 か所。`classify` の手順 5 と `simulate` の両方が使う)。`simulate.rs` の `unwrap_or` を消す(D4。決定済み)。`retryable` は `is_retry_target`(`pub(crate)`)を使う。出力と終了コードは変わらない(`inventory.md` 4e) |
| 4 | 4f | core: `retry_integer` が `u32` の範囲を受け取り、`try_from(..).ok()` の後始末を消す |
| 4 | 4g | core: `duration.rs` の余分な `n >= 1` を消す |
| 5 | 5a | cli: `CommandKind::ALL`・`from_name`。`args.rs` の逆引きの `match` を消す |
| 5 | 5b | cli: `Operand`(`Required`・`Optional`・`Forbidden`)と網羅の `match` |
| 5 | 5c | cli: `args.rs` の誤りの対象の直書き 4 か所を `Opt::…::subject()` に。`types.rs:43` は定数 `RUN_ID_OPTION`(`"--run-id"`)を使い、`args.rs` の `const _: () = assert!(…)` で `Opt::RunId` の表の綴りと結ぶ(`types` から `args` を参照しない) |
| 5 | 5d | runtime: `ParseFailure` に原因の列挙 `FailureCause`。`reset --all` の判定を理由の文で比べない |
| 6 | 6a | core: `SuccessCodes { Absent, Invalid, Valid(..) }`(`success` と `success_declared` を 1 つに) |
| 6 | 6b | core: `build_jobs(Vec<RawJob>)` で所有権を受け取り、clone をやめる |
| 6 | 6c | core: `definition::integer_value`(walk と version で共有) |
| 6 | 6d | core: `depth.rs` の 2 つの枝を `walk_children` で 1 つに |
| 7 | 7a | cli: `tests/docs_support`(`readme`、`DOCS_CHECK_WORDS`、`block_after_heading`、必須語句の検査 `assert_readme_has`・`assert_docs_check_words`。失敗のときの文は今のまま) |
| 7 | 7b | cli: `common` の `status_of`(Harness)と `status_in`(ステート) |
| 7 | 7c | cli: `tests/signal_support`(`send`、`send_self`) |
| 7 | 7d | cli: `args_grammar.rs` の `MapEnv` を `common::FakeEnv::with` に |
| 7 | 7e | cli の偽物: `MemRuntime.writes` を消す、台本は起動するときだけ消費、`ScriptedRuntime.run_lines`、`#[derive(Default)]` |
| 7 | 7f | runtime: `common::without_run_id` の 4 つの包み、`common::definition` |
| 7 | 7g | runtime の `src/state/consistency.rs` の単体テスト 4 件を core の `consistency.rs` へ |
| 7 | 7h | `JobExecution` の手組みを、`exited`・`not_started`・`start_failed`・`new` に |
| 7 | 7i | core の `tests/common`(`SECRET`、`run`、`kinds`、`median`、`state`、`job`、`exited`) |
| 7 | 7j | converter: `TargetFormat::opposite` |

各段のあと、`docs/spec/rust` の内部の名前と置き場所を合わせる(`inventory.md` の各項目の「仕様書」)。

## 3. 実物で確かめて調査と違った点、範囲から外したもの

- **調査と違った点**(実物に従った):
  - `skip_reason` の `AlreadyExecuted` と `plan_disposition` の `AlreadyExecuted` も同じ文(`already executed`/`実行済み`)。2a に入れた。
  - `messages.rs:755`(`plan_next_step` の `done`/`完了`)は、英語が `WATCH_COMPLETED` の `completed` と違うので重複ではない。まとめない。
  - `run_id_env` の文は `walk/mod.rs:285`(調査の行と同じ)。
  - `SECRET` の写しは core の 5 ファイル(`env_definition.rs` も)。cli の 4 ファイルと runtime の 1 ファイルにも同じ値があるが、クレートをまたいで `tests` の補助は共有できないので、範囲外。
  - 仕様書 05 章 `:185` にも `OutOfOrder` の同じ種類のずれがある(1d に入れた)。
  - `read_context` を呼ぶのは `ctx.rs:312` と `aux_commands.rs:36` の 2 か所(3d)。
- **範囲から外したもの**:
  - `docs_dry_run.rs`・`docs_graph.rs`・`docs_init.rs` の ```` ```text ```` の取り出し(探し方と閉じの探し方が `docs_env.rs`・`docs_retry.rs` と違い、まとめると結果が変わりうる。7a)。`docs_graph.rs:183-192` の行ごとの検査と、`docs_graph.rs:39`・`docs_retry.rs:54`・`:59-67` の個別の検査(形が違う)。
  - `types.rs` から `args::Opt` を呼ぶ形(`types` と `args` が互いを参照する。5c は定数と `const assert` にした)。
  - `version.rs:44`・`:53` の理由の文の中の `format_version` と、runtime の `state/reader.rs` の `"format_version"`(2d)。
  - `writer.rs` の `create_temp(dir, …)` に渡す `dir` を `dir_or_current` に通すこと(一時ファイルのパスの文字列が変わる。前の intent の決定。3a)。
  - `lexopt::Error` の守りの枝そのもの(列挙の網羅のため、消せない。4b)。
  - `RetryPolicy` に共有のメソッドを足す形(型の変更。4e)。
  - `StateContext` の値を `Read { with_run_id }` にする形(公開の型の変更で、テストが値を使う。3d)。

## 4. 変えるテストの一覧(ユーザーの承認が要る)

Q2: A に従い、変えるのは**補助の置き換え・移動・偽物の中身**だけで、**テストの名前、件数(移動したものも数える)、確かめる中身、期待する出力の文字列は変えない**。例外は T38(`program_name` の単体テスト 1 件の削除。Q4)だけ。src の中の単体テスト(`#[cfg(test)] mod tests`)の変更もここに挙げる。網羅に使った検索のコマンドは `inventory.md` §8。

「箇所」は変える行のまとまりの数。★ は、補助の置き換えを超えて、テストが見る値や順序に関わりうるもの(理由の欄に、結果が変わらない根拠を書く)。

### 4.1 cli(`crates/cli/tests/`)

| ID | ファイル:行 | 箇所 | 変える内容 | 理由(項目) |
|----|-------------|------|------------|------------|
| T1 | `docs_dry_run.rs:17`、`:19-25`、`:29-38`、`:40-50` | 4 | `readme()` の定義を消し、`mod docs_support; use docs_support::{assert_docs_check_words, assert_readme_has, readme};`。各機能の語句のループ(`:29-38`)は、語句の配列をこのファイルに残したまま `assert_readme_has(&text, &[..])` に。docs-check の 7 語のループ(`:40-50`)は `assert_docs_check_words(&text)` に。使わなくなった `use std::path::…` を消す。`readme()` を呼ぶ `:28`、`:55` は変えない | 7a |
| T2 | `docs_env.rs:19`、`:21-26`、`:31-40`、`:44-57`、`:59-69`、`:75`、`:85` | 7 | T1 と同じ。`first_block_after_heading` の定義を消し、`:75`・`:85` は `block_after_heading(&text, HEADING, "toml")`・`(…, "text")`。`HEADING`(`:28`)は残す | 7a |
| T3 | `docs_graph.rs:17`、`:19-25`、`:29-38`、`:41-51` | 4 | T1 と同じ(`readme()` を呼ぶ 8 か所、`:39` の見出しの検査、`:183-192` の行ごとの検査は変えない) | 7a |
| T4 | `docs_init.rs:17`、`:22-28`、`:32-41`、`:43-53` | 4 | T1 と同じ | 7a |
| T5 | `docs_retry.rs:19`、`:21-27`、`:32-40`、`:44-53`、`:72`、`:92-102` | 6 | T1 と同じ。`first_toml_block_after_heading` の定義を消し、`:72` は `block_after_heading(&readme(), HEADING, "toml")`。**docs-check の 7 語(`:92-102`)は、このファイルだけ失敗の文が「README に {word} がない」なので、`assert_docs_check_words` ではなく `assert_readme_has(&text, &DOCS_CHECK_WORDS)` を使い、文を変えない**。`:54`、`:59-67` の個別の検査は変えない | 7a |
| T6 | `docs_support/mod.rs`(新規) | 1 | `readme`、`DOCS_CHECK_WORDS`、`block_after_heading`、`assert_readme_has`(失敗の文「README に {word} がない」)、`assert_docs_check_words`(失敗の文「docs-check の語句 {word} がない」)。必須語句の検査の失敗の文は、5 つのファイルとも今と 1 文字も変わらない。変わるのは `block_after_heading` の `expect` の文(失敗のときだけ出る。今は「環境変数の設定の小節がある」「リトライの設定の小節がある」→「見出し `<見出し>` がある」)だけで、§10 の「既定で進めるもの」として承認待ち | 7a |
| T7 | `flow_run.rs:52-60` | 1 | `status_of` の定義を消す(`common` の同じ名前・同じ形を使う。呼ぶ 8 か所は変えない) | 7b |
| T8 | `flow_pipeline.rs:23-31` | 1 | T7 と同じ(呼ぶ 5 か所は変えない) | 7b |
| T9 ★ | `flow_retry.rs:74-80`、`:107`、`:131`、`:246`、`:303`、`:337` | 6 | `status_of` の定義を消し、5 か所を `status_in(&state, "a")` に。「無ければ `Pending`」が「無ければ失敗」になるが、対象の `a` はいつもステートにある(`initial_state` が全ジョブを入れる)ので、結果は同じ | 7b |
| T10 | `binary_signal.rs:31-37` | 1 | `send` の定義を消し、`mod signal_support; use signal_support::send;`(呼ぶ `:79`、`:149`、`:150` は変えない) | 7c |
| T11 | `signals_order.rs:29-35`、`:41`、`:48` | 3 | `send` の定義を消し、`signal_support::send_self` を使う(2 か所の呼び出しの名前を `send_self` に) | 7c |
| T12 | `signals_real.rs:29-36`、`:55`、`:58`、`:59`、`:69` | 5 | T11 と同じ(4 か所) | 7c |
| T13 | `signal_support/mod.rs`(新規) | 1 | `send(signal, pid)`、`send_self(signal)` | 7c |
| T14 | `args_grammar.rs:611-629`、`:633`、モジュールの宣言 | 3 | `MapEnv`・`env_of` を消し、`mod common;` を足し、`:633` は `&common::FakeEnv::with(env)` | 7d。副作用: 850 行の `common` の全体を一緒にコンパイルする(使わない部分は `common/mod.rs:16` の `#![allow(dead_code)]` で警告なし。動きと件数は変わらない)。別の道は `FakeEnv` を `common/fake_env.rs` に分けて `#[path]` で取り込む形(取り込む量は少ないが、`#[path]` の前例が無く、変えるファイルが 1 つ増える)。比べて推奨は前者(`inventory.md` 7d) |
| T15 | `common/mod.rs:171-178` | 1 | `WriteFailure` に `#[derive(Default)]`、`#[default] Never` | 7e |
| T16 | `common/mod.rs:187`、`:225-250` | 2 | `ScriptedRuntime` に `#[derive(Default)]`、手書きの `impl Default` を消す | 7e |
| T17 | `common/mod.rs:180-181`(型の別名)、`:187-222`(フィールド)、`:334-346`(`run_job`) | 3 | `RunLine` の別名、フィールド `run_lines`、`_line` を `line` にして記録 | 7e |
| T18 | `common/mod.rs:681`、`:698`、`:782` | 3 | `MemRuntime.writes` を消す | 7e |
| T19 ★ | `common/mod.rs:734-747` | 1 | `MemRuntime::run_job` は、合図を先に見て、起動するときだけ `exit_codes` を消費する。影響を受けうるのは `MemRuntime` を使う `flow_retry.rs:354-370`、`nfr_perf.rs:161-163`、`nfr_robust.rs:83-87`・`:109-125`・`:192-213`・`:218-220`、`flow_run_id.rs:350-355`。合図が立つのは `flow_retry.rs:365` の待機の中断だけで、確かめるのは終了コードと待ち時間。残りの台本の数を確かめるテストは無いので、結果は同じ | 7e |
| T20 | `common/mod.rs`(末尾に追加) | 1 | `status_in`、`status_of` | 7b |

### 4.2 runtime(`crates/runtime/`)

| ID | ファイル:行 | 箇所 | 変える内容 | 理由(項目) |
|----|-------------|------|------------|------------|
| T21 | `tests/common/mod.rs`(末尾に追加) | 1 | `pub mod without_run_id { read_state, reset_state, initial_state, write_state }`、`pub fn definition(name, jobs)` | 7f |
| T22 | `src/state/writer.rs:312-318`、`:329-344`、`:345-357`、`:358-367`(単体テスト) | 4 | `一時ファイルの名前の形`、`名前が重なったら連番を進める`、`名前が_16_回とも重なれば失敗`、`作成のそれ以外の失敗はすぐに返す` の 4 件を、名前と中身を変えずに `src/temp_file.rs` の単体テストへ移す(関数の道が `super::` で同じに引ける) | 3a |
| T23 | `tests/state_read.rs:23`、`:24-28` | 2 | 別名の import と包みを消し、`use common::without_run_id::read_state;`(`mod common;` は `:17` にある)。使わなくなった `StateError` などの import を消す | 7f |
| T24 | `tests/state_write.rs:25-28`、`:30-39` | 2 | T23 と同じ(3 つの包み) | 7f |
| T25 | `tests/nfr_content.rs:21-29`、`:31-43`、`:117-131`、`:139-146` | 4 | T23 と同じ(4 つの包み)。`definition()` は `common::definition("p", vec![common::job("a", …)])`(`:117-131` の本体を置き換え。関数名と呼ぶ側は変えない)。`:139-146` は `JobExecution::exited(&d.job[0], 3)` | 7f、7h |
| T26 | `tests/nfr_performance.rs:26-30`、`:32-41`、`:46-64` | 3 | T23 と同じ(3 つの包み)。`definition()` の本体を `common::definition("bench", …)` に | 7f |
| T27 | `tests/flow_with_state.rs:34-38`、`:41-57` | 2 | T23 と同じ(4 つの包み)。`use common::*;`(`:28`)は今のまま、`use common::without_run_id::{…};` を足す | 7f |
| T28 | `src/state/consistency.rs`(ファイル全体、4 件)、`src/state/mod.rs:15-18`・`:20`、`src/lib.rs:66` | 3 | ファイルを消し、モジュールの宣言、説明、表の行を直す(4 件は T29 で core へ移る) | 7g |

### 4.3 core(`crates/core/`)

| ID | ファイル:行 | 箇所 | 変える内容 | 理由(項目) |
|----|-------------|------|------------|------------|
| T29 | `src/consistency.rs`(末尾に追加) | 1 | runtime から移した 4 件(名前と中身は同じ。`use conductor_core::…` を `use crate::…` に。補助 `definition`・`state` も一緒に移す) | 7g |
| T30 | `tests/common/mod.rs`(新規) | 1 | `SECRET`、`run`、`kinds`、`median`、`state(jobs, resume, completed)`、`job(id)`、`exited(job_id, code)` | 7i |
| T31 | `tests/definition_errors.rs:22-28`、`:35-37`、モジュールの宣言 | 3 | `run`・`kinds` の定義を消し、`mod common; use common::{kinds, run};`(呼ぶ側は変えない。`with_header` `:30-33` は残す) | 7i |
| T32 | `tests/definition_retry.rs:21-31`、モジュールの宣言 | 2 | T31 と同じ | 7i |
| T33 ★ | `tests/env_definition.rs:19`、`:30-37`、モジュールの宣言 | 3 | `SECRET`・`kinds` を `common` のものに(`parse`・`ok` は残す)。今の `kinds` は受理された入力で失敗し、共通の `kinds` は空を返すが、`kinds` を受理される入力で呼ぶテストは無い(呼ぶ 11 か所はすべて誤りを期待する)。誤りの種類の並びは同じ | 7i |
| T34 | `tests/env_file_parse.rs:19`、`tests/env_merge.rs:22`、`tests/env_types.rs:24`、`tests/plan_types.rs:22`、各モジュールの宣言 | 8 | `const SECRET` を消し、`mod common; use common::SECRET;`(値は同じ) | 7i |
| T35 | `tests/env_merge_perf.rs:39-43`、`tests/flow_perf.rs:33-37`、各モジュールの宣言 | 4 | `median` を `common` のものに | 7i |
| T36 | `tests/plan_run.rs:53-74`、`:77`、`:112`、`:123`、`:133` | 5 | `state_with` を消し、4 か所を `state(&[…], None, false)` に | 7i |
| T37 ★ | `tests/plan_pipeline.rs:58-79`、`tests/graph_overlay.rs:23-44`、各モジュールの宣言 | 4 | `state` の定義を消し、`common::state` を使う(呼ぶ側は変えない)。値の違い: `plan_pipeline.rs` の `Skipped` は `skip_reason` が `None` から `ByTransition` になるが、`Skipped` を渡す `:182-191`・`:203-213` は直後に理由を書き直すので、書いた後の値は同じ。`graph_overlay.rs` の `definition_key`・`definition_path` は `"k"`・`"conductor.toml"` から `"t.toml"` になるが、図と計画の処理はこの 2 つを読まない | 7i |
| T38 | `src/types.rs:627-634`(単体テスト) | 1 | **テスト `プログラム名は_line_では先頭の語_argv_では先頭の要素` を消す(件数 −1。Q4)** | 1e |
| T39 | `tests/flow_sequences.rs:46-56`、`:300-304`、モジュールの宣言 | 3 | 補助 `exited` を消し `common::exited` を使う(呼ぶ 18 か所は変えない)。`:300-304` は `JobExecution::not_started(job, 15)` | 7h |
| T40 | `src/error.rs:383-387`(単体テスト) | 1 | 名前は変えず、「名前の『64 文字』は `…` を除いた本文の文字数」の注記を 1 行足す(D1。ユーザーが決定、2026-11-23) | 1b |
| T41 | `tests/retry_flow.rs:39-49`、`:51-60`、`:109-112`、`:119-126`、`:133-140`、`:147-150` | 6 | 補助 2 つの本体と、4 つの `..exec(&j)` を組み立ての関数に(`inventory.md` 7h の表) | 7h |
| T42 | `tests/graph_edges.rs:158-168`、`:176` | 2 | 補助 `exec` を消し、`JobExecution::exited(job, code)` | 7h |
| T43 | `src/flow/classify.rs:116-122`(単体テスト) | 1 | 補助 `exec` の本体を `JobExecution::new(&job(None, Vec::new()), result)` に | 7h |
| T44 | `src/flow/next_step.rs:226-236`、`:355-378`(単体テスト) | 2 | 補助 `exec` の本体を `JobExecution::exited(&job(job_id, vec![]), code)` に、4 つの `..exec("j1", 0)` を組み立ての関数に | 7h |
| T45 | `src/flow/record.rs:227-237`(単体テスト) | 1 | 補助 `exec` の本体を `JobExecution::new(&definition(&[job]).job[0], …)` に | 7h |

T39・T41〜T45 で、手組みの `command`(`"c"`・`"echo"`)はジョブの定義の `command` に変わる。`JobExecution.job_id`・`command` を読む処理は無い(`inventory.md` 7h)ので、確かめる中身は変わらない。

### 4.4 converter(`crates/converter/`)

| ID | ファイル:行 | 箇所 | 変える内容 | 理由(項目) |
|----|-------------|------|------------|------------|
| T46 | `tests/convert_roundtrip.rs:37-40` | 1 | `let to = from.opposite();` | 7j |

### 4.5 テストを変えずに通るもの(参考)

段 1〜6 の本体の変更は、`report::messages` の定数・関数の名前(テストは参照しない)、非公開のモジュールの中、または公開の関数の形を変えないものなので、既存のテストは T22・T38・T40 のほかは 1 行も変えずに通る。`CommandKind::ALL`・`from_name`、`TargetFormat::opposite` は足すだけ。`RunIdArg::parse` の形と誤りの対象(`--run-id`)は変えない。`StateContext` の 5 つの値は変えない。

**合計**: 変えるテストのファイルは 46(cli 15(新規 2: `docs_support`、`signal_support`)、runtime 8(削除 1: `src/state/consistency.rs`。src の単体テスト 2)、core 22(新規 1: `tests/common`。src の単体テスト 6)、converter 1)。このほか、移動先の `crates/runtime/src/temp_file.rs`(新規のモジュール。T22 の 4 件と N4)がある。件数が変わるのは T38(−1)だけで、T22・T28/T29 は移動(件数は移動先で数える)。

## 5. 足すテスト(Q3: A。整理で確かめられるようになったものだけ)

test-after(各項目の実装のコミットの後に、テストのコミット)。各テストは、正しい場合と、境界または誤りの場合を 2 つ以上確かめる。

| ID | 場所 | 確かめること | 件数 |
|----|------|--------------|------|
| N1 | `crates/cli/tests/args_grammar.rs` | `CommandKind::ALL` の全部で `from_name(kind.name()) == Some(kind)`(往復)、名前が重ならない、知らない名前(`""`、`"Run"`、`"runs"`)は `None` | 1 |
| N2 | `crates/cli/tests/flow_run.rs` | cli の層で、分割済みのコマンドを分け直さずに渡す: 配列の `command = ["prog", "a b"]` のジョブは `run_lines` に `CommandLine { program: "prog", args: ["a b"] }`、文字列の `command = "prog 'x y' z"` は `args: ["x y", "z"]`(`check_command` の分け方のとおりで、空白で分け直さない) | 1 |
| N3 | `crates/cli/tests/flow_retry.rs` | リトライのたびに、同じ分割済みのコマンドを渡す(`run_lines` が試行の数だけあり、すべて最初の `CommandLine` と等しい。合図で起動しないときの記録も同じ値) | 1 |
| N4 | `crates/runtime/src/temp_file.rs`(単体テスト) | `create_temp_beside` は、対象の名前の接頭辞 `.<名前>.tmp.` の新しいファイルを `dir` に作り、続けて呼ぶと別の名前になる(連番が進む)。存在しないディレクトリでは失敗する | 1 |
| N5 | `crates/converter/src/types.rs`(単体テスト) | `TargetFormat::opposite`: `Yaml` ↔ `Toml`、2 回で元に戻る | 1 |

- 合計の見込み: **+5 件**。実装で増減したら、その数と理由を記録する。

## 6. 件数の見込み

| クレート | 単体(src) | 統合(tests/) | 合計 | 基準からの差 |
|---|---|---|---|---|
| core | 112 − 1(T38)+ 4(T29) = **115** | 261 | **376** | +3 |
| runtime | 71 − 4(T28)+ 1(N4) = **68** | 125 | **193** | −3 |
| converter | 53 + 1(N5) = **54** | 63 + 1 無視 | **117 + 1 無視** | +1 |
| cli | 0 | 457 + 3(N1〜N3) = **460** | **460** | +3 |
| 全体 | | | **1,146 通過、1 無視、0 失敗** | +4(= −1 + 5) |

ファイルごと: `args_grammar.rs` 61、`flow_run.rs` 53、`flow_retry.rs` 19。ほかの統合テストのファイルの件数は変わらない。

## 7. 確かめ方

- 段の前と後で、`cargo test --locked --workspace` の件数をクレートごとに記録する(基準 §0 の表)。合格は、失敗 0 で、全体が「基準 1,142 − 1(T38)+ 足したテスト」に一致すること [Q1]。移動(T22、T28/T29)の段では、移動元と移動先のクレートの単体テストの差が打ち消し合うことも確かめる。
- テストのファイルの変更は §4 の T 表と §5 の N 表だけ。段ごとに `git diff --stat -- 'crates/*/tests'` と、src の単体テスト(`#[cfg(test)] mod tests`)の差分を見る。テスト名が変わっていないことは、段の前後で `cargo test --locked --workspace -- --list 2>/dev/null | sort` の差が「T38 の 1 件の削除と N1〜N5 の追加と、T22・T29 の道の違い(`state::writer::tests::` → `temp_file::tests::`、runtime の `state::consistency::tests::` → core の `consistency::tests::`)」だけであることで確かめる。
- 「重複・ずれ・到達しない分岐・文字列での比較が残っていない」は、`inventory.md` の「今」の場所を実装の後に 1 つずつ見て確かめる [Q1]。たとえば:
  - `grep -n "SKIPPED_LIST\|NOTE_BY_\|GRAPH_RESUME_HERE\|PLAN_RESUME_HERE\|GRAPH_DETAIL\|PLAN_DETAIL\|GRAPH_KIND_SUCCESS\|GRAPH_KIND_WARNING\|plan_class_word\|WATCH_COMPLETED\|fn plan_end" crates/cli/src`(0 件)
  - `grep -rn '"conductor: error: ' crates/cli/src`(`error_prefix!` の定義の 1 か所)
  - `grep -rn "英字または" crates/core/src`(マクロの定義の 1 か所)、`grep -rn 'CONDUCTOR_` で' crates/core/src`(0 件)
  - `grep -rn "static SEQUENCE" crates/runtime/src`(`temp_file.rs` の 1 か所)、`grep -rn "crate::state::\(dir_or_current\|writer::\)" crates/runtime/src/file_create.rs crates/runtime/src/platform`(0 件)
  - `grep -rn "fn push_line" crates/cli/src`(1 件)、`grep -rn "job '{}'" crates/cli/src`(`job_place` の 1 件)
  - `grep -n "nothing_started" crates/core/src/flow/decide.rs`(0 件)、`grep -n "MissingValue { .. }" crates/cli/src/args.rs`(0 件)、`grep -n "=> String::new()" crates/cli/src/report/mod.rs`(0 件)、`grep -n "_ => 0" crates/cli/src/retry.rs`(0 件)、`grep -n "unwrap_or(Classification::Error)" crates/core/src`(0 件)、`grep -n "try_from(m)" crates/core/src/definition/walk/retry.rs`(0 件)
  - `grep -n '"run" =>' crates/cli/src/args.rs`(0 件)、`grep -rn '"--\(to\|assume\|format\|run-id\)"' crates/cli/src`(`types.rs` の `RUN_ID_OPTION` の定義の 1 件だけ)、`grep -n "reason == reason::" crates/runtime/src`(0 件)
  - `grep -n "success_declared" crates/core/src`(0 件)、`grep -rn "from_str_radix" crates/core/src/definition`(`integer_value` の 1 件)
  - テスト: `grep -n "for word in" crates/cli/tests/docs_*.rs`(`docs_graph.rs:183` の 1 件だけ)、`grep -rn "fn readme\|fn status_of\|fn send\|struct MapEnv\|fn median\|const SECRET\|fn kinds" crates/*/tests`(共通の場所の 1 か所ずつ。cli と runtime の `SECRET` は範囲外で残る)、`grep -rn "JobExecution {" crates`(型の定義だけ)、`grep -rn "fn read_state(def" crates/runtime/tests`(`common` の 1 件)、`ls crates/runtime/src/state/consistency.rs`(無い)
  - `grep -rn "program_name" crates docs/spec/rust`(0 件)
- 合格の条件: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、全テスト、行カバレッジ 80% 以上(`cargo llvm-cov --fail-under-lines 80`)、MSRV 1.100 でのテスト [memory:M2] [memory:M3] [memory:M4]。新しい依存を足さない(`Cargo.lock` の差分なし)[memory:M5]。`unsafe` と src の `cfg` を足さない(`nfr_source.rs`)。converter の `Cargo.toml` に core を足さない。
- README と `docs-check` に触れる変更は無い。
- 段 4 の 4c: `flow_run.rs:71`、`flow_pipeline.rs:44` の「ロケールの判定が行われない」がそのまま通ること。段 3 の 3a: `state_write.rs`(一時ファイルが残らない、古い一時ファイルの後始末)、`file_create.rs`、`flow_init.rs` がそのまま通ること。

## 8. コミットの並び(小さな単位。各コミットでビルドとテストが通る)

段 1(5 コミット):
1. core: `types.rs` の生成元の説明(1a)
2. core: `sanitize`・`escape_display` の説明と、テストの注記(1b。T40)
3. cli: `args.rs` の文法の表(1c)
4. 仕様書: 08 章の拒否の理由の 3 行、05 章の `OutOfOrder`(1d)
5. core・仕様書: `program_name` とその単体テストを消す(1e。T38。件数 −1 を記録)

段 2(6 コミット):
6. cli: `SKIPPED_LIST`・`NOTE_BY_*`・`ALREADY_EXECUTED`(2a の前半)と仕様書 08 章の 1 行
7. cli: `RESUME_HERE`・`DETAIL_*`・`class_word`・`COMPLETED`(2a の後半)
8. cli: `error_prefix!` と `INTERNAL_ERROR`・`SIGNAL_SETUP_FAILED`(2a)
9. core: `env_name_rule!`・`reserved_env_prefix!`(2b、2c)
10. core: `ROOT_KEYS` の `VERSION_KEY`、間隔の文と定数の `assert`(2d、2e)
11. core: `CONDUCTOR_RUN_ID` の注記(2f)

段 3(8 コミット):
12. runtime: `temp_file` モジュール(`dir_or_current` を移す。`platform/unix.rs`・`reset_all.rs`・`writer.rs` の道を直す)
13. runtime: `temp_name`・`create_temp`・`temp_prefix` と単体テスト 4 件を移す(T22)
14. runtime: `create_temp_beside` と連番 1 つ(`writer.rs`・`file_create.rs`)、`lib.rs` の表、仕様書 02・06 章
15. runtime のテスト: `create_temp_beside` の単体テスト(N4)
16. cli: `push_line`(3b)と `job_place`(3c)
17. cli: `reset_hint`・`StateContext::for_read`(3d)
18. cli: `plan_end` を分ける(3e)
19. cli: `graph_mermaid.rs` の let-else(3f)

段 4(7 コミット):
20. core: 規則 5 のコードを消す、説明と仕様書 05 章の注記(4a)
21. cli: lexopt の枝(4b)
22. cli: `render` の `match` を 1 つに(4c)
23. cli: `retry.rs` の既定値(4d)
24. core: `classify_exit_code`、`is_retry_target` の共有、`simulate.rs`(4e)
25. core: `retry_integer` の `u32`(4f)
26. core: `duration.rs` の `n >= 1`(4g)

段 5(6 コミット):
27. cli: `CommandKind::ALL`・`from_name` と `args.rs` の逆引き、仕様書 02 章(5a)
28. cli のテスト: 名前の往復(N1)
29. cli: `Operand`(5b)
30. cli: 誤りの対象を `Opt::subject` に(5c)
31. runtime: `FailureCause`(5d)

段 6(4 コミット):
32. core: `SuccessCodes`(6a)
33. core: `build_jobs` の所有権(6b)
34. core: `integer_value`(6c)
35. core: `depth.rs` の `walk_children`(6d)

段 7(13 コミット):
36. cli のテスト: `docs_support` と 5 つの docs のテスト(T1〜T6)
37. cli のテスト: `status_of`・`status_in`(T7〜T9、T20)
38. cli のテスト: `signal_support`(T10〜T13)
39. cli のテスト: `args_grammar.rs` の `MapEnv`(T14)
40. cli のテスト: 偽物の `Default` の derive と `writes`(T15、T16、T18)
41. cli のテスト: `MemRuntime` の台本の消費の順(T19)
42. cli のテスト: `run_lines` の記録(T17)と、N2、N3
43. runtime のテスト: `without_run_id` と `definition`(T21、T23〜T27 の包みと `definition()`)
44. core・runtime: `consistency` の単体テストを core へ(T28、T29)、仕様書 02・12 章
45. core のテスト: `tests/common` の `SECRET`・`run`・`kinds`・`median`(T30 の一部、T31〜T35)
46. core のテスト: `state`(T30 の一部、T36、T37)
47. core・runtime: `JobExecution` の組み立ての関数(T39、T41〜T45、T25 の `:139-146`)
48. converter: `TargetFormat::opposite` と N5(T46)

最後:
49. 仕様書: 11 章のテストのファイルの表(`docs_support`、`signal_support`、core の `tests/common`、runtime の `common` の補助)と、02 章の偽物の表(`run_lines`)

合計 **49 コミット**ほど(実装で分けたりまとめたりしたら記録する)。版の更新はこの intent に含めない(利用者から見える変更が無いため)。

## 9. 範囲の外(書かない)

- 型と構造の大きな整理: `ResumeInfo`・`PlanEntry`・`GraphNode`・`RetryPolicy` の backoff・`NextStepNote` の列挙型化、`JobExecution.job_id`/`command` などの読まれないフィールド、`EnvFileError.reason`、`Checked` と `CheckedDefinition`、準備の入口、`record_skip`、`messages.rs` の関数の形の統一、公開範囲を狭める、`CliCommand` の列挙型化。
- 計算量。
- 利用者から見える変更(converter の文法の誤りの文言、`.env` の読み込みの順序)。converter のほかの細かな点と、文法の誤りの文言の重複。
- `EdgeList`、`enter_flow`、graph の配置、2 つのテスト用補助バイナリの統合。
- 凍結した仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)。
- §3 の「範囲から外したもの」。

## 10. 決めてほしい点

D1〜D5 は、ユーザーが推奨のとおりに決めた(2026-11-23)。

| ID | 決めること | 推奨 | 代案 | 決定 |
|----|------------|------|------|------|
| D1 | `error.rs` のテスト名 `エスケープ後も_64_文字を超えない`(実際は `…` を含めて 65) | **名前は変えず**、テストの中に注記を 1 行足す(T40。Q2 の「名前を変えない」を守る) | 名前を「エスケープ後は_64_文字で切り_省略記号を足す」などに変える(Q2 の例外を 1 件増やし、T 表に足す) | 決定: 名前は変えず、注記を 1 行足す(T40)(ユーザー、2026-11-23) |
| D2 | `CONDUCTOR_RUN_ID` の二重(`merge.rs:46` と `DEFAULT_RUN_ID_ENV`) | **まとめない**(渡す名前は固定、読む名前は `run_id_env` で変わる)。注記だけ | 渡す側に別の定数 `BUILTIN_RUN_ID_ENV` を置く(同じ値の定数が 2 つになるだけ) | 決定: まとめず、注記を足す(ユーザー、2026-11-23) |
| D3 | lexopt の守りの枝 | `Err(_)` の 1 つの枝にまとめ、「lexopt 0.3.2 の `next` は `UnexpectedValue` しか返さない。版を上げたときの守り」と注記して**残す**(列挙の網羅のため消せない) | (無し) | 決定: 1 つの枝にまとめ、注記して残す(ユーザー、2026-11-23) |
| D4 | `simulate.rs` の分類の `unwrap_or(Classification::Error)` | **`classify_exit_code` を切り出し**(`classify` の手順 5 の規則を 1 か所に)、`simulate` はそれで結果の語を決める | `unwrap_or` を残し、到達しない理由の注記だけ足す | 決定: `classify_exit_code` を切り出す(ユーザー、2026-11-23) |
| D5 | 一時ファイルの連番(`writer.rs` と `file_create.rs` で 2 つ) | **1 つにする**(`temp_file`。連番の値は利用者に見えず、名前の形は変わらない) | 呼び出し側ごとに連番を持ち、`create_temp_beside` に渡す | 決定: 1 つにする(ユーザー、2026-11-23) |

**既定で進めるもの**(ユーザーが設計の承認で認めた、2026-11-23): T9 の「無ければ失敗」、T19 の台本の消費の順、T33 の `kinds`、T37 のステートの補助の値、T6 の `expect` の文、T39・T41〜T45 の `command` の値(どれも §4 の理由のとおり、テストの結果は変わらない)。

## 11. 成功の指標との対応(traceability.json)

| ID | 成功の指標(`intent-statement.md`) | 対応する規則 |
|----|------------------------------------|--------------|
| SM1 | (1) 本体の整理: 重複が 1 か所、到達しない分岐・効かない処理がない、文字列の比較が型・定数、説明が実装と一致、`program_name` がない | BR1.1〜BR1.5、BR2.1〜BR2.6、BR3.1〜BR3.6、BR4.1〜BR4.7、BR5.1〜BR5.4、BR6.1〜BR6.4 |
| SM2 | (2) テストの補助: 写しが共通の場所に、古い組み立てが今の型の作り方に、cli の偽物が使われない状態を持たずコマンドを記録 | BR7.1〜BR7.10 |
| SM3 | 新しいテスト(整理で確かめられるようになったものに限って少数) | BR8.1 |
| SM4 | 利用者から見える動きは変わらない(版は上げない) | BR9.1 |
| SM5 | 既存のテストの扱いと件数(基準 − 1 + 足したテスト) | BR9.2、BR9.3 |
| SM6 | 合格の条件(fmt、clippy、カバレッジ 80%、MSRV、依存なし) | BR9.4 |
