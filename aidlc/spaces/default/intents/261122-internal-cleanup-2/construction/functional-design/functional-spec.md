# 機能の仕様(Functional Spec): 動きを変えない整理の第 2 弾

意図: `ideation/intent-capture/intent-statement.md`(Q1〜Q4 すべて A)。**変える場所の正確な一覧(ファイルと行、目標の形、影響するテスト)は `inventory.md`**(HEAD a2f036f で作った)。この文書は、方針、決めたこと、**承認が要るテストの変更(§4)**、**性能の基準(§5)**、確かめ方、コミットの分け方を書く。

## 1. 方針

- 利用者から見える動き(標準出力と標準エラーの文、終了コード、ステートファイル、`--dry-run` の出力)を変えない [desc]。
- 4 つの段を順に進める。段 1(説明と制約)、段 2(小さな重複と計算量)、段 3(cli の ctx)、段 4(型とテスト)。各段は小さなコミットの積み重ねで、どのコミットでもビルドとテストが通る [Q1] [memory:M1]。
- 承認は、この設計とコードの計画の 1 回。実装は段の順に進め、各段の終わりにテストが通ることを確かめ、結果は最後にまとめて見せる [Q1]。
- 新しい依存を足さない。`unsafe` は runtime の `platform/unix.rs` だけ、`cfg(unix)`/`cfg(windows)` は `platform` の外に置かない、エラーに入力の値を含めない、ステートに環境変数を書かない、MSRV 1.100、converter は core に依存しない。テスト名とドキュメントコメントは日本語。
- 範囲の外: B3(core の `read_definition` の順序)、F(`run_job` に `CommandLine` を渡す)、`JobExecution` の真偽値の列挙型化、`EdgeList` の削除、凍結した 5 つの仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)[desc]。

## 2. 段ごとの変更(詳細は `inventory.md`)

| 段 | 項目 | 要点 |
|----|------|------|
| 1 | 1a | `apply_skip` の説明の誤り 4 か所を直す(完了の判定は `apply_skip`。`resume_point_on_interrupt` は pipeline のジョブのあいだの中断だけ) |
| 1 | 1b | `Plain`(reset だけ)、`OutputFailed`・`EXIT_STATE`(graph・dry-run・init も 65)、`config_path`(convert 以外)、`read_state_raw`(graph --state も)の説明。仕様書 08 章(rust と language-agnostic) |
| 1 | 1c | escape の「4 つ」「runtime」を「3 つ」に(最終の文は 2i のあと) |
| 1 | 1d | core・runtime・cli の `lib.rs` の表と、runtime の `state/mod.rs` の説明 |
| 1 | 1e | `definition_key` の「仮」を、確定した説明に(フィールドは消さない) |
| 1 | 1f | 仕様書 `docs/spec/rust/11` のテストの件数を**書くのをやめる**(ずれる、OS で変わる) |
| 1 | 1g | core と converter に `#![forbid(unsafe_code)]` |
| 1 | 1h | 理由の文の上限の数と定数を `const _: () = assert!(…)` で結ぶ(core 4、converter 3、調査に無かった runtime 1 と cli の英語の文 2) |
| 2 | 2a | `status_at` を `flow` で共有。`pending_ids`(と `resume_point_on_interrupt`)を位置で引き、O(n) に |
| 2 | 2b | `merge_env` を「最後の位置だけ残す」で O(E) に(BR4.3 の並びを保つ)。任意で `.env` の解析の重複の除去も |
| 2 | 2c | 既定の定義ファイル名を core の `DEFAULT_DEFINITION_FILE`・`resolve_path` に寄せる |
| 2 | 2d | `TransitionTarget::END_LABEL`・`label()`。`DisplayEvent::Transition.target` を `&TransitionTarget` に |
| 2 | 2e | 表示の `place_of(file, line)`。`show_path` を 4 か所で使う |
| 2 | 2f | graph の text と Mermaid の `kind_label`・`is_main_edge` を共通に |
| 2 | 2g | `Ctx::usage_error`(7 か所)、`Ctx::show_output`(4 か所)、`exit::outcome_exit_code`(3 か所) |
| 2 | 2h | runtime の `remove_state_file` と `dir_or_current` |
| 2 | 2i | escape の本体を core の `escape_display` に。cli は包みと再公開。converter の写しは残す |
| 2 | 2j | `aux_commands.rs` の型の書き直しとフルパス |
| 3 | 3a | `report_execution`・`result_detail` を `step.rs` へ。`definition_errors` を非公開に |
| 3 | 3b | `Session { checked: Checked, signal }` |
| 3 | 3c | `check_job`・`check_pipeline` で `prepare_run`・`prepare_pipeline`・`prepare_checked` の重複をなくす(**順序と表示は変えない**) |
| 3 | 3d | `resolve_run_id`・`aux_run_id` を新しい `run_id` モジュールへ |
| 4 | 4a | `CliCommand.run_id: Option<RunIdArg>`(`Empty` / `Id(RunId)`)。`RunId::parse` は 1 回 |
| 4 | 4b | `DecisionReason` を `SkipCause`(3 つ)と `RejectReason`(1 つ)に分ける。到達しない分岐 4 つが消える |
| 4 | 4c | `PlanMode`・`EdgeKind`・`EdgeTarget` を `RunMode`・`TransitionOn`・`TransitionTarget` の別名(`pub use … as`)に。`From<Classification> for JobStatus`、`TransitionOn::parse` |
| 4 | 4d | `runtime/tests/flow_with_state.rs` を今の cli の流れ、名前、期待に(Q3: A) |

各段のあと、`docs/spec/rust` の内部の名前と置き場所を合わせる(`inventory.md` の 2k、3e、4e)。`docs/spec/language-agnostic` は 1b の `08:211` だけ直す。

## 3. 実物で確かめて調査と違った点、範囲から外したもの

- **調査と違った点**(実物に従った):
  - `show_path`(`crates/cli/src/report/mod.rs:477-480`)は未使用ではない(`init` の 4 つの表示が使う)。重複は、4 か所が同じ式を書いていること(2e)。
  - 「使い方の誤りを表示して 68」は 9 か所でなく 7 か所(`ctx.rs` 3、`dry_run.rs` 1、`aux_commands.rs` 3)。`lib.rs:144-146` は `Ctx` を作る前なので対象外(2g)。
  - 「出力に失敗して 65」の `init` は、失敗の表示が `InitStdoutFailed`(別の文)。補助は失敗の表示を引数で受ける(2g)。
  - `intake.rs` の 1 MiB の文は `:128`(調査の `:135` は関数の終わり)。
  - 上限の数の文は、調査の場所のほかに `runtime/src/error.rs:103`(1 MiB)と cli の英語の文 `messages.rs:324`・`:326` にもある。同じ検査を足す(1h)。
  - 「位置で状態を引く」重複は、`resume.rs:36-45` にもある(O(n²)。2a で一緒に直す)。
  - 「親が空なら `.`」は 2 か所でなく 4 か所(`reset_all.rs`、`writer.rs` の `remove_stale_temps`、`file_create.rs`、`platform/unix.rs`)(2h)。
  - 仕様書 11 章の件数は、挙がった 3 件のほかに `plan_perf.rs` 1→2、`plan_pipeline.rs` 9→10 もずれ、OS でも変わる → 件数を書くのをやめる(1f)。
  - 仕様書 02 章で `resolve_run_id` の置き場所を書くのは `:349` のほかに `:149`、`12-rebuild-guide.md:76` も(3d)。
- **範囲から外した、または任意にしたもの**:
  - `.env` の解析(`envfile/parse.rs:83-85`)の重複の除去は任意(2b)。上限の入力(10,000 行のうち 5,000 が重複)でも debug 40.6 ms で、利用者の手元の 1 回の読み取りでは問題にならない。直すなら同じ「最後の位置だけ残す」形で、警告の行と順を変えない。性能のテストは足さない(Q4 の対象は 2 か所)。
  - `write_state_file` の `create_temp(dir, …)`(親が空のまま `join`)は、一時ファイルのパスの文字列が変わるので 2h から外す。
  - `lib.rs:144-146` の使い方の誤り(`Ctx` を作る前)は 2g の補助を使わない。

## 4. 公開の型の変更で直す必要がある既存のテスト(ユーザーの承認が要る)

Q2: A に従い、変えるのは**組み立ての値と import(と、型の値の名前の置き換え)だけ**で、**期待する出力の文字列は 1 文字も変えない** [Q2]。T9 は意図の段で決めた Q3: A の範囲 [Q3]。

| ID | ファイル:行 | 変える内容 | 理由(項目) | テストを変えずに済む代案 |
|----|-------------|------------|------------|--------------------------|
| T1 | `crates/cli/tests/report_messages.rs:25-29`(import)、`:252-261`、`:553-557` | `target: "c"` → `target: &TransitionTarget::Job("c".into())`、`target: "@end"` → `target: &TransitionTarget::End`、`target: "notify"` → `target: &TransitionTarget::Job("notify".into())`。import に `TransitionTarget` を足す。期待の `"[conductor] transition: warning -> notify (skipped: transform, load)\n"` は変えない | 2d: 公開の型 `DisplayEvent::Transition.target` を `&str` から `&TransitionTarget` に変える | `target` を `&str` のまま残し、`step`(段 3 の後)で `transition.next.label()` を渡す。`"@end"` の文字列は core の 1 か所にまとまるが、行き先を型で渡す目的は満たさない |
| T2 | `crates/cli/tests/report_messages.rs:26`(import)、`:181-195`、`:444`、`:453`、`:563` | `:181-195` の 1 つのループ(4 つの理由を `Rejected` と `JobSkipped` の両方に入れる)を、`Rejected` は `RejectReason::OutOfOrder` だけ、`JobSkipped` は `SkipCause::{SkipTarget, AfterEnd, AlreadyExecuted}` の 2 つのループに分ける(到達しない組み合わせ 4 つ、`Rejected` にスキップの理由 3 つと `JobSkipped` に `OutOfOrder`、は型の上で作れなくなるので、組み立てから消える)。`:444` → `RejectReason::OutOfOrder`、`:453` → `SkipCause::AfterEnd`、`:563` → `SkipCause::SkipTarget`。期待の `"[conductor] job=transform skipped (skipped by a transition)\n"` などは変えない | 4b: `DecisionReason` を分ける | 無い(分けること自体が組み合わせを作れなくする)。分けないなら 4b を範囲から外す |
| T3 | `crates/core/tests/flow_sequences.rs:21`(import)、`:169`、`:180`、`:208`、`:218` | `Decision::Reject(DecisionReason::OutOfOrder)` → `Decision::Reject(RejectReason::OutOfOrder)`、`Decision::Skip(DecisionReason::AlreadyExecuted)` → `Decision::Skip(SkipCause::AlreadyExecuted)`、`AfterEnd` も同じ | 4b | 同上 |
| T4 | `crates/core/tests/plan_run.rs:18`(import)、`:148` | `reason: DecisionReason::OutOfOrder` → `reason: RejectReason::OutOfOrder` | 4b(`PlanRejection.reason`) | 同上 |
| T5 | `crates/core/src/flow/decide.rs:141`、`:283`、`:302`(単体テスト) | `use DecisionReason::*;` → `use RejectReason::*; use SkipCause::*;`(表の中の裸の `SkipTarget`・`OutOfOrder` などは、そのまま通る)。`:283` → `RejectReason::OutOfOrder`、`:302` → `SkipCause::SkipTarget` | 4b | 同上 |
| T6 | `crates/core/src/flow/record.rs:461`、`:470`、`:471`、`:481`、`:495`、`:506`、`:515`、`:537`、`:543`(単体テスト) | `DecisionReason::SkipTarget` などを `SkipCause::…` に。**`:471`(`apply_skip` に `OutOfOrder` を渡すと何も変えない)の 1 行は消す**: `apply_skip` が `SkipCause` を受け取るので、`OutOfOrder` は型の上で渡せなくなる(この行が確かめていた「到達しない分岐」自体が消える)。テスト名 `already_executed_は不変` と `:470` は残る | 4b(`record.rs:137` の到達しない分岐) | 同上 |
| T7 | `crates/cli/tests/args_grammar.rs:483`、`:488`、`:492`、`:700`、`:793`、`:621-624`(補助 `resolved`)と import | `cmd.run_id.as_deref()` → `cmd.run_id.as_ref().map(RunIdArg::as_str)`(期待の `Some("r1")`、`Some("")` は変えない。`:486`、`:515` の `run_id, None` は変えない)。補助 `resolved` は、`option.map(RunIdArg::parse).transpose()?` で組み立ててから `conductor_cli::run_id::resolve_run_id(option.as_ref(), "RID", …)` を呼ぶ(`:684` の `resolved(Some("a/b"), &[])` は `RunIdArg::parse` が今と同じ `invalidValue`・`--run-id` を返すので、期待は変えない) | 4a(`CliCommand.run_id` の型)、3d(`resolve_run_id` の置き場所) | (a) `RunIdArg` に `Deref<Target = str>` を実装すれば、`:483`〜`:793` の 5 行は変えずに済む(列挙型に `Deref` を持たせるのは型の意味をぼかすので勧めない)。(b) `args::resolve_run_id(Option<&str>, …)` を古い形の包みとして残せば補助も変えずに済むが、テストしか使わない入口と二重の検査が残る |
| T8 | `crates/cli/tests/escape_table.rs:15`、`:17`、`:22`(ドキュメントコメントだけ) | 「conductor-core・conductor-runtime・conductor-converter と揃える」から runtime を消し、「本体は conductor-core、converter は写し」に。2i のあと cli の `escape_display` は core の `sanitize` そのものを呼ぶので、`:17-18` の「core の `sanitize` と直接揃える」は「cli は core の `sanitize` を使う。このテストは期待値の表を固定し、converter の写しは converter の単体テストの同じ表で揃える」の趣旨に書き直す(テストの中身は変えない) | 1c、2i(説明のずれ) | コメントを残す(テストの中身には影響しないが、説明は誤ったまま) |
| T9 | `crates/runtime/tests/flow_with_state.rs:130-244`(補助)、`:247`・`:350`(名前)、`:266`・`:299`・`:322`・`:345`・`:360`・`:390`(期待) | `inventory.md` の 4d のとおり。補助を今の cli の流れに(スキップは変わったときだけ書く、削除は `remove_on_complete` のときだけ、pipeline は記述順に決める)。2 件の名前を「完了のステートが残る」に。完了の後の期待を「ファイルが残り、`read_state` は空」に。件数は 20 のまま | 意図の Q3: A(古い流れの再現を直す) | (意図の段で決定済み) |

**ユーザーは、T1〜T8 のテストの変更(T7 は代案を採らず、この表のとおり)を認めて、設計を承認した**(2026-11-22)。

**テストを変えずに通るもの**(参考): `PlanMode`・`EdgeKind`・`EdgeTarget` は別名(`pub use … as`)にするので、`graph_edges.rs:19`・`:25`(`use EdgeKind::{Success, Warning}`)、`graph_types.rs`、`graph_drawable.rs`、`graph_common/mod.rs`、`plan_run.rs:19`・`:83` は変えずに通る。escape は `conductor_cli::report::escape::{escape_display, escape_name, escape_path}` の道を再公開で保つ。`DEFAULT_INIT_PATH` の名前と値は保つ。これら以外の段 1〜3 の変更は、非公開のモジュールの中なので、既存のテストは 1 行も変えずに通る。

**4c の代案(別名を置かない)**: 古い名前を消すなら、次も直す必要がある(承認の対象を増やすので勧めない): `core/tests/graph_edges.rs:19`・`:25`・`:28-35`・`:56`・`:115`・`:127`・`:139`・`:150`・`:185`・`:189`、`graph_types.rs:19`・`:27-35`、`graph_drawable.rs:21`・`:224`・`:263-264`、`graph_common/mod.rs:21`・`:130-133`、`plan_run.rs:19`・`:83`。

## 5. 性能の基準(Q4: A)

### 5.1 測り方

- 測った機械と条件: macOS(開発者の主な環境)、rustc 1.100.0、HEAD a2f036f。一時的なテストのファイルを `crates/core/tests/` に作って測り、**測り終えたあと消した**(`git status` で作業ツリーに残っていないことを確かめた。コミットしていない)。
- 形: `cargo test -p conductor-core --locked --test <一時のファイル> -- --nocapture --test-threads=1`(**debug**。CI の `test` ジョブと同じ `cargo test`)。参考に `--release` と、CI の `coverage` と同じ計測つき(`cargo llvm-cov --no-report`)でも測った。
- 各場面を **11 回測り、中央値**(と最小・最大)を取った。
- 「直した後」は、直した後の形(位置で引く、最後の位置だけ残す)を一時のテストの中に書いて見積もった。

### 5.2 結果(直す前の実物、ミリ秒)

| 場面 | 入力 | debug 中央値(最小〜最大) | 計測つき | release |
|------|------|--------------------------|----------|---------|
| A2: `@end` に着いたときの残りのジョブの列挙(`classify_and_decide`、`RunMode::Pipeline`、先頭のジョブの成功で `@end`、残り 999 個) | ジョブ 1,000 個(`plan_perf.rs:25` の `N`)。1 回 | 2.764(2.743〜4.751) | 2.871 | 0.900 |
| 同上を 100 回 | 同上 | **278.0**(276.4〜292.6) | 286.8 | 90.9 |
| A3a: `merge_env`、`[pipeline].env_file` の 1 ファイル | `.env` 10,000 項目(`envfile/mod.rs:40` の `MAX_ENV_ENTRIES`)、すべて別の名前 | **353.3**(350.6〜364.2) | 383.5 | 90.6 |
| A3b: `merge_env`、`[pipeline].env_file` と `[job].env_file` の 2 ファイル | 10,000 項目ずつ、同じ名前(後ろが勝つ) | **2,166.4**(2,161.8〜2,457.1) | 2,227.0 | 308.3 |
| (参考)`.env` の解析 | 10,000 行、うち 5,000 が重複 | 40.6 | 40.6 | 26.4 |

直した後の見積もり(同じ入力、debug 中央値): A2 を 100 回 **6.1 ms**(計測つき 6.2、release 3.1)、A3b **8.7 ms**(計測つき 8.8、release 1.0)。A3a(項目は A3b の半分)は測っておらず、A3b の見積もり以下と**推定**した。A3a の直した後の時間は、実装の段で測って記録する。

### 5.3 決めた基準(新しいテストの合格の条件)

各テストは、同じ処理を **5 回測った中央値** を上限と比べる(1 回の揺れで落ちないように)。測った値は `eprintln!` で出す(`--nocapture` で見える)。

| テスト | 測る処理 | 合格の上限 | 直す前に対する上限の割合 | 直した後の見積もりに対する余裕 |
|--------|----------|------------|--------------------------|--------------------------------|
| A2(`crates/core/tests/flow_perf.rs`) | A2 を 100 回 | **50 ms** | 18%(直す前の 278 ms は上限の 5.6 倍で、落ちる) | 8.2 倍(6.1 ms) |
| A3a(`crates/core/tests/env_merge_perf.rs`) | A3a を 1 回 | **50 ms** | 14%(直す前の 353 ms は 7.1 倍) | 5.7 倍以上(A3b の見積もり 8.7 ms を上限の目安にした場合) |
| A3b(同じファイル) | A3b を 1 回 | **100 ms** | 4.6%(直す前の 2,166 ms は 21.7 倍) | 11.5 倍(8.7 ms) |

- 余裕の考え方: CI の機械(GitHub Actions の ubuntu・macos・windows、debug、`coverage` は計測つき)が手元の 2〜3 倍遅くても、直した後は上限の半分に届かない(最も小さい余裕の A3a でも 5.7 倍以上)。一方、直す前の形に戻ると、手元の機械でも上限を 5 倍以上超えて落ちる(二乗の時間に戻ったことを検知できる)。計測つきの値は debug とほぼ同じ(+3〜9%)。
- 合格の確認: テストは結果の中身も確かめる(A2 は `skipped_job_ids` が 999 個で記述順、A3 は項目が 10,004 個(10,000 + 自動の変数 4)で、並びが BR4.3 のとおり、A3b の出どころは `JobEnvFile`)。
- 前後の記録: 段 2 の実装で、直す前の木でテストを走らせて落ちる時間を記録し、直した後に通る時間を記録する(Build and Test の記録に残す)。コミットは「直すコミット → テストを足すコミット」の順(test-after)。
- `plan_perf.rs`(1,000 ジョブの計画、1 秒以内)と `graph_perf.rs` は今のまま。起動の性能(`nfr_perf.rs`)の扱い(警告のみ)は変えない。

## 6. 足すテスト

- 性能: §5 の 3 件(A2 1、A3 2)。
- 新しい小さな関数の単体テスト(test-after。各 1 件、正しい場合と境界の 2 つ以上を確かめる): `TransitionOn::parse`(`as_str` との往復と、知らない語・`error` が `None`)、`TransitionTarget::label`(ジョブと `@end`)、`From<Classification> for JobStatus`(3 つ)、`RunIdArg::parse`・`as_str`(空、規則どおり、規則違反は `invalidValue` で値を含めない。`crates/cli/tests/` の新しいファイル)。
- 合計の見込み: **+7 件**(基準 1,128 → 1,135)。実装で増減したら、その数と理由を記録する。

## 7. 確かめ方

- 段の前と後で、`cargo test --workspace --locked` の件数を記録する(基準: **1,128 通過、1 無視、0 失敗**。`inventory.md` §0)。合格は、失敗 0 で、件数が「基準 + 足したテスト」に一致すること [Q1]。T6 で消す 1 行は 1 つのテストの中の 1 つの確かめなので、件数は変わらない。`flow_with_state.rs` は 20 件のまま [Q3]。
- テストのファイルの変更は §4 の一覧と §6 の新しいファイルだけ。段ごとに `git diff --stat -- 'crates/*/tests'` と、`src` の中の単体テスト(`#[cfg(test)] mod tests`)の差分で確かめる(既存の単体テストの変更は T5、T6 だけ。足すのは §6)。
- 「重複・ずれ・到達しない分岐・文字列での比較が残っていない」は、`inventory.md` の「今」の場所を実装の後に 1 つずつ見て確かめる [Q1]。たとえば: `grep -rn '"@end"' crates/*/src`(core の `END_LABEL` の 1 か所だけ)、`grep -rn "DecisionReason" crates docs/spec/rust`(0 件)、`grep -rn '"success" =>' crates/core/src/definition`(0 件)、`grep -rn "fn is_format_char" crates`(core と converter の 2 件)、`grep -rn 'Path::new(".")' crates/runtime/src`(`dir_or_current` の 1 か所)。
- 段 3 の順序: `flow_run.rs`・`flow_pipeline.rs`・`flow_dry_run.rs`・`flow_run_id.rs`・`flow_env.rs` が誤りの順序(64・68・65 の勝ち方)と表示を確かめている。これらがそのまま通ること。
- 合格の条件: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、全テスト、行カバレッジ 80% 以上(`cargo llvm-cov --fail-under-lines 80`)、MSRV 1.100 でのテスト [memory:M2] [memory:M3] [memory:M4]。新しい依存を足さない(`Cargo.lock` の差分なし)[memory:M5]。`unsafe` と `cfg` を足さない(`nfr_source.rs`)。
- README と `docs-check` に触れる変更は無い(終了コードの表、必須の語句は変えない)。

## 8. コミットの分け方(小さな単位。各コミットでビルドとテストが通る)

段 1(9 コミットほど):
1. core: `apply_skip` の説明を直す(1a。`lib.rs`、`decide.rs`、`next_step.rs`、`resume.rs`)
2. cli: `Plain`・`OutputFailed`・`EXIT_STATE`・`config_path`・`read_state_raw` の説明と、仕様書 08 章(1b)
3. cli: escape の説明を直す(1c。`escape.rs`、`escape_table.rs` のコメント(T8)、仕様書 11 章の 2 か所)
4. core・runtime・cli: `lib.rs` の表と `state/mod.rs` の説明(1d)
5. core: `definition_key` の注記(1e)
6. 仕様書: 11 章のテストの件数を消す(1f)
7. core・converter: `#![forbid(unsafe_code)]`(1g)
8. core: 上限の数と定数をコンパイル時に結ぶ(1h の core)
9. converter・runtime・cli: 同じ(1h の残り)

段 2(16 コミットほど):
10. core: `status_at` を `flow` で共有し、`record.rs`・`resume.rs` も使う(2a)
11. core: `pending_ids` を位置で引く(2a。直す前の A2 の時間を先に記録)
12. core のテスト: `flow_perf.rs`(A2)
13. core: `merge_env` を最後の位置だけ残す形に(2b。直す前の A3 の時間を先に記録)
14. core のテスト: `env_merge_perf.rs`(A3a、A3b)
15. (任意)core: `.env` の解析の重複の除去(2b)
16. cli: 既定の定義ファイル名を core に寄せる(2c)
17. core: `TransitionTarget::END_LABEL`・`label`、walk が使う(2d)。単体テスト
18. cli: `DisplayEvent::Transition.target` を型に、graph の `@end` も(2d。T1)
19. cli: `place_of` と `show_path`(2e)
20. cli: graph の `kind_label`・`is_main_edge` を共通に(2f)
21. cli: `usage_error`・`show_output`・`outcome_exit_code`(2g)
22. runtime: `remove_state_file` と `dir_or_current`(2h)
23. core・cli: escape の本体を core に(2i。escape の説明の最終の文も)
24. cli: `aux_commands.rs` の冗長(2j)
25. 仕様書: 段 2 の置き場所(2k)

段 3(5 コミットほど):
26. cli: `report_execution`・`result_detail` を `step.rs` へ、`definition_errors` を非公開に(3a)
27. cli: `Session` が `Checked` を持つ(3b)
28. cli: `check_job`・`check_pipeline` で prepare の重複をなくす(3c)
29. cli: `run_id` モジュールに `resolve_run_id`・`aux_run_id` を移す(3d。T7 の呼ぶ道は 30 と一緒でもよい)
30. 仕様書: 段 3 の置き場所(3e)

段 4(6 コミットほど):
31. cli: `RunIdArg`(4a。T7)。単体テストの新しいファイル
32. core・runtime・cli: `SkipCause` と `RejectReason`(4b。T2〜T6)
33. core: `PlanMode`・`EdgeKind`・`EdgeTarget` を別名に(4c)
34. core: `From<Classification> for JobStatus`、`TransitionOn::parse`(4c)。単体テスト
35. runtime のテスト: `flow_with_state.rs` を今の流れに(4d。T9)
36. 仕様書: 段 4 の型(4e)、11 章の `flow_with_state.rs` の目的の文

版の更新(0.9.6 など)はこの intent に含めない(利用者から見える変更が無いため。必要なら別に決める)。

## 9. 成功の指標との対応(traceability.json)

| ID | 成功の指標(`intent-statement.md`) | 対応する規則 |
|----|------------------------------------|--------------|
| SM1 | 段 1(説明と制約) | BR1.1〜BR1.8 |
| SM2 | 段 2(小さな重複と計算量、性能のテスト) | BR2.1〜BR2.11 |
| SM3 | 段 3(cli の ctx) | BR3.1〜BR3.4 |
| SM4 | 段 4(型とテスト) | BR4.1〜BR4.4 |
| SM5 | 動きを変えない、テストの扱い | BR5.1〜BR5.3 |
| SM6 | 合格の条件 | BR5.4 |
