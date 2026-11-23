# 機能の仕様(Functional Spec): 型と構造の整理・計算量・利用者から見える変更

意図: `ideation/intent-capture/intent-statement.md`(Q1〜Q5 すべて A)。**変える場所の正確な一覧(ファイルと行、目標の形、完了の判定、使った検索のコマンド)と、性能の直す前の測定は `inventory.md`**(HEAD f1c9871 で作った)。この文書は、方針、段ごとの設計、**承認が要るテストの変更(§4 の T 表)**、足すテスト(§5 の N 表と性能の基準)、件数の見込み、確かめ方、コミットの並び、範囲の外、決めてほしい点を書く。

## 1. 方針

- 3 つの段を順に進める。段 1(型と構造)と段 2(計算量)は、利用者から見える動き(標準出力と標準エラーの文、終了コード、ステートファイル、`--dry-run` と `graph` の出力)を変えない [desc]。段 3(利用者から見える変更)を最後に置き、最後のコミットで版を 0.9.7 に上げる [desc] [Q4] [Q5]。
- 各段は小さなコミットの積み重ねで、どのコミットでもビルド、`clippy -D warnings`、テストが通る [Q1] [memory:M1] [memory:M4]。
- 承認は、この設計とコードの計画の 1 回。実装は段の順に進め、各段の終わりにテストの件数を記録し、結果は最後にまとめて見せる [Q1]。
- 既存のテストは、公開の型の変更に伴う組み立て、import、項目の読み方だけを変える。確かめる中身と期待する出力の文字列は変えない [Q2]。例外は §4 の ★(T17a の 1 行、段 3 の T24・T25)だけ。
- 新しい依存を足さない [memory:M5]。`unsafe` は runtime の `platform/unix.rs` だけ、`cfg(unix)`/`cfg(windows)` は src の `platform` の外に置かない(テストのファイルの `#[cfg(unix)]` は前例どおり可)、誤りの表示に入力の値を含めない、ステートに環境変数を書かない、runtime の製品のコード(src の `#[cfg(test)]` より前の部分)に `.unwrap()`・`.expect(`・`panic!(`・`unreachable!(`・`todo!(`・`unimplemented!(` を書かない(`crates/runtime/tests/nfr_source.rs` の `製品のコードに_パニックしうる書き方がない` が検査する。3b の形 B の `.env` の読み込みでも使わず、失敗はすべて `EnvFileErrorKind` で返す)、converter は core に依存しない、MSRV 1.100。テスト名とドキュメントコメントは日本語。
- 仕様書は `docs/spec/rust` と `docs/spec/language-agnostic` を直す。凍結した `docs/spec/{go,python,typescript,csharp,powershell}` は触らない [Q2]。

## 2. 段ごとの設計(詳細は `inventory.md` の同じ番号)

| 段 | 項目 | 要点 |
|----|------|------|
| 1 | 1a | `ResumeInfo` を `enum { FromStart, AfterStop(String), Individual(String) }` に。`ResumeKind` を消す。cli の `plan_text.rs` は網羅の `match`(`_` の枝なし) |
| 1 | 1b | `PlanDisposition::WillRun(SimulatedResult)`。`PlanEntry.result` を消し、読み方の補助 `PlanDisposition::result()` |
| 1 | 1c | `GraphNode { kind: NodeKind, state, is_resume }`。`NodeKind` は `TransitionTarget` の別名(`Job(String)`/`End`。D1)。補助 `job_id()`・`is_end()` |
| 1 | 1d | `RetryPolicy.backoff: Option<Backoff { multiplier, max_interval_secs }>`。`next_wait_secs` の片方だけの `_ =>` の枝が消える |
| 1 | 1e | `NextStepNote` を `enum { NextJob { job_id, skipped_job_ids }, EndReached, Completed, Stopped }` に。表示は網羅の `match`(作れない組は D7) |
| 1 | 1f | `RetryDecision::Retry { wait_secs, last_exit_code, max }`。cli の `retry.rs` の 2 つ目の let-else(到達しない else)を消す |
| 1 | 1g | `JobExecution` は `result` だけ(組み立ての関数は `job` を受け取らない)。runtime の `run_job(line, env, signal)`。`JobOutcome`・`NextStep` から `job_id` を消す。契約 C1 を直す(D2) |
| 1 | 1h | `EnvFileError.reason` をメソッド `reason()` に。`DefinitionError`・`EnvFileError` の表示の書式を `error::render_error` の 1 つに |
| 1 | 1i | cli: `Checked { base: CheckedDefinition, … }`(項目の二重をなくす)、準備の入口を `prepare_session`(本番)と `prepare_checked`(`--dry-run`)の 2 つに(D3)、`record_skip` は `SkipRecord { state, written }` |
| 1 | 1j | cli `messages.rs` の関数の形を 3 つ(固定の文の定数 / 種類ごとの `fn(kind) -> Text` / 値を埋める `fn(lang, …) -> String`)に。他のクレートが日本語を持つ種類は `Text::new(英語, kind.label())`。core・converter に `label()` を足し、runtime の `StateErrorKind::label` を公開(D10) |
| 1 | 1k | 外から使われない `pub` を狭める(`inventory.md` 1k の表の core 7 行、cli 6 行、runtime 1 行)。テストが使う `LoadedEnvFiles::add`・`Reporter::language_resolved`・`color_resolved` は狭めない(後者 2 つは `#[doc(hidden)]`。D9) |
| 2 | 2a | 計画の模擬は、crate の中の「その場で書き換える」版(`apply_outcome_in`・`apply_skip_in`・`apply_interrupt_in`)を使い、ステートを複製しない。公開の関数は「複製して `_in` を呼ぶ」だけ。探索は位置の手がかりつき(D8) |
| 2 | 2b | `build_graph` の重ね合わせを `HashMap`(最初のものを残す)で引く |
| 2 | 2c | `flow_edges` は遷移先の位置の表(最初の位置を残す)を 1 回作り、`resolve_transition` に引く関数を渡す。実行の経路は今のまま `job_index` |
| 2 | 2d | `.env` の解析の重複キーを、位置の表と「消した」印で扱う(`position` と `remove` をやめる) |
| 2 | 2e | 計画の名前の要約は、値を読まない `merge_env_names`(`merge_env` と並べ方・残し方を共有) |
| 3 | 3a | converter の文法の誤りの理由を「TOML として読めません」「YAML として読めません」に(D5)。`resolve.rs` の内部の守りの 2 か所は「入力の構造を解釈できません」(D4)。種類の文は変えない |
| 3 | 3b | `.env` の読み込みを定義ファイルと同じ形 B(種別をパスで見る → 開く → 開いたものから大きさ → 上限 + 1 まで読む → UTF-8)に。読む権限のない 1 MiB 超の `.env` は `unreadable` |
| 3 | 3c | 最後のコミットで版を 0.9.7 に(`ee9f409` と同じ 22 ファイル) |

各項目のコミットで、`docs/spec/rust` と `docs/spec/language-agnostic` の該当の行を合わせる(`inventory.md` の各項目の「仕様書」)。段 1・2 の仕様書の変更は内部の名前と形だけで、利用者から見える挙動の記述は変えない。

## 3. 実物で確かめて調査と違った点、範囲から外したもの

- **調査と違った点**(実物に従った):
  - 1f: 前の intent(4d)で cli の `retry.rs` の既定値 0 は消えていた。残っているのは「`Retry` が返るのは設定があり終了コードつきの `Exited` のときだけ」を呼ぶ側が let-else でもう一度書いている点(else は到達しない)。これを `Retry` の中身で置き換える。
  - 1g: `JobExecution` の `job_id`・`command` を消すと、runtime の `run_job` の `job` も使われなくなる(`runner.rs:101-144` は `job` を組み立ての関数に渡すだけ)。公開の関数の引数の変更が runtime のテスト 13 か所に及ぶ(T21)。cli の `Runtime::run_job` は偽物がジョブ ID を記録するので、`job` を残す。
  - 1g: `JobOutcome.job_id` を読むのは `classify.rs:177` の単体テストの 1 行だけ。フィールドを消すので、この 1 行の確認を消す(★ T17a)。
  - 1g: language-agnostic の仕様書は、`JobExecution`・`JobOutcome`・`NextStep` の `job_id`・`command` を書いていない(論理の形)。直すのは rust の仕様書だけ。
  - 1j: 統一の対象は `state_kind` の二重だけでなく、日本語を他のクレートの `Display` に任せて `String` を返す 4 つ(`definition_kind`・`env_file_kind`・`env_file_reason`・`conversion_kind`)と、`&'static str` を返す 2 つ(`plan_disposition`・`class_word`)。
  - 1k: `quote_key`・`Locate`(core の `error.rs`)も `error::*` で根に出ていたが、外から使われない。狭める。`LoadedEnvFiles::add`・`Reporter::language_resolved`・`color_resolved` はテストが使うので狭められない。
  - 2a: 計画は、複製をやめても `decide_before_run`・`classify_and_decide` の 1 回あたり O(n) の安い比較が残り、全体は二乗のまま(約 3 倍速くなる)。これらは公開の関数の契約の中の処理で、初期の記述の (4) に無い(範囲の外)。
  - 2e: 名前の要約は出力そのものが「ジョブ × 項目」の大きさで、値の複製をやめても約 2 割の改善(計算量の改善ではない)。
  - 3a: `resolve.rs` の 2 か所(`:290`、`:477-483`)は、どちらも形式に依らない内部の守りで、通常の入力では到達しない。
  - 3b: 期待が変わる既存のテストは無い(読む権限のない大きな `.env` を確かめるテストが無い)。
- **範囲から外したもの**:
  - `decide_before_run` の `job_index`・`record_of`・前のジョブの確認、`classify_and_decide` の `job_index`、`apply_skip` の走査(2a。公開の関数の契約の中の処理)。
  - runtime の `envfile.rs:43-47` の `seen` の線形の探索と、core の `env_file_refs` の `out.contains`(`.env` の記述の数は小さく、初期の記述に無い)。
  - converter の `ConversionError::render`(converter は core に依存しないので、`render_error` を共有しない。1h)。
  - `ExecutionPlan.end` の `run` のときの参考の値(1e。`pipeline` が使う。`NextStepNote` と意味が違う)。
  - cli の `Runtime::run_job` の `job` の引数(1g。偽物が使う)。
  - `enter_flow`(1i。初期の記述のとおり触らない)。

## 4. 変えるテストの一覧(ユーザーの承認が要る)

Q2: A に従い、変えるのは**組み立て、import、項目の読み方(同じ意味の新しい書き方への置き換え)**だけで、**テストの名前、件数、確かめる中身、期待する出力の文字列は変えない**。例外は ★ の 3 つ(T17a の 1 行の確認の削除、段 3 の期待の文字列の変更 T24・T25)。src の中の単体テスト(`#[cfg(test)] mod tests`)の変更もここに挙げる。

テストの補助の関数(`exec`、`exited`、`outcome`、`step` など)の引数が、組み立ての関数から `job` が外れて使われなくなったときは、**引数の名前を `_` で始めて残し、呼ぶ側は変えない**(差分を小さくし、「どのジョブの結果か」を読めるようにする。D2 の既定)。

### 4.0 網羅に使った検索のコマンド

全体は `inventory.md` §8。T 表を作るのに使った主な検索:

```sh
grep -rn "ResumeInfo\|ResumeKind\|\.resume\.\|NextStepNote\|next_step_note\|note\.\(next_job_id\|skipped_job_ids\|completed\|end_reached\|stopped\)\|SimulatedResult\|\]\.result\|entry\.result\|\.result\.expect\|\.result\.is_none\|PlanDisposition\|D::\|e\.disposition" crates
grep -rn "GraphNode\|is_end\b\|\.is_end\|node\.job_id\|n\.job_id" crates
grep -rn "multiplier\|max_interval_secs\|RetryPolicy {" crates
grep -rn "RetryDecision\|retry(" crates/core/tests/retry_flow.rs crates/cli/src
grep -rn "JobExecution::\(new\|exited\|not_started\|start_failed\)\|JobExecution {\|JobOutcome {\|NextStep {" crates
grep -rnoE "[a-z_\]\)]+\.job_id\b" crates | awk -F: '{print $3}' | sort | uniq -c
grep -rn "run_job(" crates
grep -rn "\.reason\b" crates/core/tests crates/cli/tests crates/runtime/tests
grep -rn "messages::\|report::messages\|report::watch\|status_word\|state_symbol\|glyph_char\|MAX_DISPLAY_CHARS\|DEFAULT_INIT_\|platform::" crates/cli/tests crates/cli/src/bin
grep -rn "文法の誤りです\|reason::SYNTAX" crates
grep -rn "TooLarge\|tooLarge" crates/runtime/tests crates/cli/tests
grep -rn "0\.9\.6" crates
```

### 4.1 core(`crates/core/`)

| ID | ファイル:行 | 箇所 | 変える内容 | 理由(項目) |
|----|-------------|------|------------|------------|
| T1 | `tests/plan_pipeline.rs:22`、`:64`、`:83`、`:90`、`:100`、`:110`、`:129`、`:157`、`:176`、`:66`、`:131-132`、`:140-141`、`:158`、`:69`、`:78`、`:92` | 16 | import の `ResumeKind` を `ResumeInfo` に。`WillRun` を含む扱いの並びの比較 8 か所は `assert!(matches!(dispositions(&plan).as_slice(), [D::WillRun(_), …]), "{:?}", …)` に(並びの各要素を同じ順に確かめる。`:199` の `WillRun` を含まない比較はそのまま)。再開位置 4 か所は `assert_eq!(plan.resume, ResumeInfo::FromStart)`・`ResumeInfo::AfterStop("b".into())`(`kind` と `job_id` の 2 つの確認を 1 つの値の比較に)。`.result.expect`・`.result.is_none()` 3 か所は `.disposition.result()…` | 1a、1b |
| T2 | `tests/plan_run.rs:21-22`、`:70`、`:72-73`、`:75-76`、`:86-87`、`:93-94`、`:109`、`:120`、`:132`、`:161` | 10 | import。`:70` は `matches!(…, D::WillRun(_))`。`:72-73` は `ResumeInfo::Individual("b".into())`。次の動き: `:75-76` は `NextStepNote::NextJob { job_id, .. }` で `job_id == "c"`、`:86-87` は `NextJob { job_id: "c", skipped_job_ids: ["b"] }` と等しい、`:93-94` は `NextStepNote::EndReached`(次のジョブなし・スキップなしは型が表す)、`:109` は `NextStepNote::Completed`、`:120` は `NextStepNote::Stopped`。`:132`・`:161` は `.disposition.result()` | 1a、1b、1e |
| T3 | `tests/plan_assume.rs:62`、`:68`、`:74`、`:90`、`:106`、`:107` | 6 | `.result.expect("結果")` を `.disposition.result().expect("結果")` に(`:79` の `D::NotReached` との比較はそのまま) | 1b |
| T4 | `tests/plan_types.rs:22`、`:29-40`、`:42` | 3 | import。項目の組み立てを `disposition: PlanDisposition::WillRun(SimulatedResult { … })` にして `result` を消す。`:42` は `entry.disposition.result().map(…)` | 1b |
| T5 | `tests/plan_perf.rs:73`、`:91` | 2 | `:73` は `matches!(e.disposition, PlanDisposition::WillRun(_))`。`:91` は `JobExecution::exited(0)` | 1b、1g |
| T6 | `tests/graph_overlay.rs:29`、`:31` | 2 | `n.job_id.as_deref()` を `n.job_id()`、`.is_end` を `.is_end()` に | 1c |
| T7 | `tests/graph_types.rs:18-20`、`:57-68`、`:73` | 3 | import に `NodeKind`。節の組み立てを `kind: NodeKind::Job("a".into())`・`kind: NodeKind::End` に。`:73` は `job_id().is_none() && is_end()` | 1c |
| T8 | `tests/graph_drawable.rs:20-22`、`:248-258` | 2 | T7 と同じ(節の組み立て) | 1c |
| T9 | `tests/definition_retry.rs:58-64`、`:71-72`、`:85-86`、`:98-99` | 4 | 期待の `RetryPolicy` の `multiplier: None, max_interval_secs: None` を `backoff: None` に。`p.multiplier`・`p.max_interval_secs` を `p.backoff.map(\|b\| b.multiplier)`・`p.backoff.map(\|b\| b.max_interval_secs)` に(確認の数は同じ) | 1d |
| T10 | `tests/retry_flow.rs:17-20`、`:39-48`、`:50-52`、`:54-56`、`:63`、`:149`、`:176`、`:233-237`、`:238`、`:101`、`:108-115`、`:122-129`、`:136`、`:156-162`、`:184-191`、`:201` | 16 | import に `Backoff`。補助 `exec`・`exited` は本体を `JobExecution::new(…)`・`JobExecution::exited(code)` に(引数は `_j`)。補助 `retry(wait_secs)` を `wait_of(d: RetryDecision) -> Option<u64>` に替え、5 か所の比較を `assert_eq!(wait_of(retry_decision(…)), Some(30))` に(確かめるのは今と同じ待ち時間だけ)。`:101`・`:108-115`・`:122-129`・`:136` は組み立ての関数から `&j` を外す。`RetryPolicy` の組み立て 2 か所を `backoff: None`・`backoff: Some(Backoff { multiplier, max_interval_secs: max_interval })` に。`:201` の型の分解に `..` を足す | 1d、1f、1g |
| T11 | `tests/flow_sequences.rs:291` | 1 | `JobExecution::not_started(15)`(補助 `exited` の 18 か所の呼び出しは T12 で変えない) | 1g |
| T12 | `tests/common/mod.rs:62-65` | 1 | `exited` の本体を `JobExecution::exited(code)` に(引数は `_job_id`。説明を「ジョブ ID は読みやすさのため。値は使わない」に) | 1g |
| T13 | `tests/graph_edges.rs:165` | 1 | `JobExecution::exited(code)` | 1g |
| T14 | `tests/flow_perf.rs:46` | 1 | `JobExecution::exited(0)` | 1g |
| T15 | `tests/env_file_parse.rs:179` | 1 | `err.reason` を `err.reason()` に | 1h |
| T16 | `src/types.rs:501-508`、`:527-529`(単体テスト) | 2 | `policy()` の `backoff: None`。`p.multiplier = Some(2); p.max_interval_secs = Some(600);` を `p.backoff = Some(Backoff { multiplier: 2, max_interval_secs: 600 });` に | 1d |
| T17a ★ | `src/flow/classify.rs:177`(単体テスト) | 1 | **`assert_eq!(o.job_id, "a")` を消す**(`JobOutcome.job_id` が無くなる。同じテストの分類・返す値・`conductor_error` の確認はそのまま)。**コミット 8**(`JobOutcome`・`NextStep` の `job_id` を消すコミット) | 1g |
| T17b | `src/flow/classify.rs:147-149`(単体テスト) | 1 | 補助 `exec` の本体を `JobExecution::new(result)` に。**コミット 9**(`JobExecution` の組み立ての関数から `job` を外すコミット)。T17a と分けるのは、各コミットでビルドが通るようにするため | 1g |
| T18 | `src/flow/next_step.rs:226-228`、`:347-364`(単体テスト) | 2 | 補助 `exec` の本体を `JobExecution::exited(code)` に(引数は `_job_id`)。`:347-364` の 4 つの組み立てから `j` を外す | 1g |
| T19 | `src/flow/record.rs:227-236`、`:238-245`、`:247-257`(単体テスト) | 3 | 補助 `exec` の本体を `JobExecution::new(…)` に(引数は `_job`)。補助 `outcome`・`step` から `job_id` の項目を消す(引数は `_job`) | 1g |

### 4.2 runtime(`crates/runtime/`)

| ID | ファイル:行 | 箇所 | 変える内容 | 理由(項目) |
|----|-------------|------|------------|------------|
| T20 | `tests/nfr_content.rs:113`、`:114-119`、`:120-128` | 3 | `JobExecution::exited(3)`。`JobOutcome`・`NextStep` の組み立てから `job_id: "a".into(),` を消す | 1g |
| T21 | `tests/runner_basic.rs:29`、`:111`、`:182`、`:229`、`:273`、`tests/runner_env.rs:31`、`tests/runner_signal.rs:40`、`:121`、`:158`、`:201`、`tests/nfr_robust.rs:182`、`:190`、`tests/flow_with_state.rs:206` | 13 | `run_job(&j, &line, …)` を `run_job(&line, …)` に(`j` は `check_command(&j.command)` などに使い続ける) | 1g |
| T22 | `src/runner.rs:260`、`:270`、`:277`(単体テスト) | 3 | T21 と同じ | 1g |

### 4.3 converter(`crates/converter/`)

| ID | ファイル:行 | 箇所 | 変える内容 | 理由(項目) |
|----|-------------|------|------------|------------|
| T24 ★ | `tests/convert_ac.rs:70` | 1 | **期待の文字列 `"文法の誤りです"` を `"YAML として読めません"` に**(`broken.yaml` の理由。今の検査は種類の文にも一致するので理由が変わっても通るが、理由の文を確かめる意図に合わせる。Q4) | 3a |

### 4.4 cli(`crates/cli/`)

| ID | ファイル:行 | 箇所 | 変える内容 | 理由(項目) |
|----|-------------|------|------------|------------|
| T23 | `tests/common/mod.rs:334`、`:361`、`:725`、`:727` | 4 | 偽物の `run_job` の中の組み立てから `job` を外す(`JobExecution::not_started(number)`、`::new(result)`、`::exited(…)`)。偽物の `run_job` の引数(`job`)は、ジョブ ID の記録に使うので今のまま | 1g |
| T25 ★ | `tests/binary_basic.rs:246` | 1 | **`"conductor 0.9.6\n"` を `"conductor 0.9.7\n"` に**(版の更新。前回 `ee9f409` と同じ) | 3c |
| T26 | `tests/docs_retry.rs:71-72` | 1 | `retry.multiplier`・`retry.max_interval_secs` を `retry.backoff.map(\|b\| b.multiplier)`・`retry.backoff.map(\|b\| b.max_interval_secs)` に | 1d |

### 4.5 テストを変えずに通るもの(参考)

- 1h の `render_error`、1i(非公開のモジュール)、1j(`report::messages` はテストが参照しない。表示は 1 文字も変わらない)、1k(狭めるのは外から参照の無いものだけ)、段 2 のすべて(公開の関数の形と結果は同じ)、3b(期待の変わるテストが無い)は、既存のテストを 1 行も変えずに通る。
- `PlanDisposition` の単位の値(`AlreadyExecuted` など)との `==` の比較は、そのままコンパイルできる。

**合計**: 変えるテストのファイルは 30(core 19(統合 15、src の単体テスト 4)、runtime 7(統合 6、src の単体テスト 1)、converter 1、cli 3)。件数が変わる変更は無い(★ T17a は 1 つのテストの中の 1 行。T17a はコミット 8、T17b はコミット 9)。期待の文字列が変わるのは ★ T24・T25 だけ。

## 5. 足すテスト

test-after(各項目の実装のコミットの後に、テストのコミット)。各テストは、正しい場合と、境界または誤りの場合を 2 つ以上確かめる(性能のテストは、時間と結果の中身を確かめる)。

### 5.1 一覧

| ID | 場所 | 確かめること | 件数 |
|----|------|--------------|------|
| N1 | `crates/core/tests/retry_flow.rs` | `Retry` が最後の終了コードと最大回数を持つ: 75 → `Retry { wait_secs: 30, last_exit_code: 75, max: 3 }`、ジョブ自身の 127 → `last_exit_code: 127`、倍率つき(max 5)の 2 回目 → `wait_secs: 20, last_exit_code: 75, max: 5`。一覧に無い 76 と回数を使い切ったときは `Stop` | 1 |
| N2 | `crates/core/tests/env_types.rs` | `EnvFileErrorKind::ALL` のすべてで、`EnvFileError::new(…, kind).reason() == kind.reason()`、表示に理由を含む。`EnvFileErrorKind` の 13 種と `DefinitionErrorKind` の 33 種の `label()` が、期待の文字列の表(変更前の `#[error("…")]` の文を、種類ごとにテストに固定の文字列として書いたもの。例: `FileMissing` → `環境変数ファイルが存在しません`、`Syntax` → `文法の誤りです`)と一致する(D10 で文を `#[error]` から `label()` へ移すときに、文が 1 文字も変わらないことを確かめる。`label() == to_string()` は構成上いつも真なので確かめない) | 1 |
| N3 | `crates/cli/tests/report_messages.rs` | 日本語のロケールで、定義(33 種)・`.env`(13 種)・変換(6 種)・ステート(6 種)の誤りの表示が、各クレートの `label()`(`.env` は `reason()` も)を含む(cli が日本語の文を別に持たない) | 1 |
| N4 | `crates/core/tests/plan_simulate_perf.rs`(新規) | 性能 P1(§5.2)。結果: 1,000 項目がすべて `WillRun`、`end == Completed` | 1 |
| N5 | 同上 | 性能 P2。結果: 先頭が `AlreadyExecuted`、残り 999 が `SkippedAfterEnd`、`end == EndReached` | 1 |
| N6 | `crates/core/tests/graph_lookup_perf.rs`(新規) | 性能 P3。結果: 定義の順と逆順の重ね合わせで、全ジョブの状態が重ね合わせのとおり、無いジョブは `Pending`、同じジョブ ID が 2 つある重ね合わせでは最初のものが使われる | 1 |
| N7 | 同上 | 性能 P4。定義はジョブ 1,000 個(`j0`〜`j999`)で、`j0`〜`j997` が成功で次のジョブ、警告で 2 つ先へ遷移を書く(遷移 1,996 個)。結果: **辺の数は 1,998**(`j0`〜`j997` は成功と警告の行き先が違うので 2 本ずつ = 1,996、`j998` は遷移なしで成功・警告とも `j999` へ 1 本、`j999` は遷移なしで終点へ 1 本)。`j0`〜`j997` の成功の辺は次のジョブ、警告の辺は 2 つ先で、どちらも `explicit`。`j998`・`j999` の辺は `explicit` でない | 1 |
| N8 | `crates/core/tests/env_parse_perf.rs`(新規) | 性能 P5。結果: 項目は 5,000 個で、並びは後半の順(`K4999` … `K0`)、値は後ろのもの、重複の警告が 5,000 件 | 1 |
| N9 | (欠番) | 性能 P6 は足さない(D6 の決定(ユーザー、2026-11-23))。番号は付け直さない | 0 |
| N10 | `crates/converter/tests/convert_ac.rs` | 文法の誤りの理由が形式ごとの文: `broken.yaml` → 理由 `YAML として読めません`、`broken.toml` → `TOML として読めません`。表示は `…: 文法の誤りです: YAML として読めません` の形で、`文法の誤りです: 文法の誤りです` を含まない。種類は `Syntax` のまま | 1 |
| N11 | `crates/runtime/tests/env_loader.rs`(`#[cfg(unix)]`) | 読む権限のない(mode 000)1 MiB 超の `.env` は `Unreadable`(root などで開けてしまうときは確かめない。前例 `intake.rs` の `権限のない_1_mib_超のファイルは_unreadable`)。直す前の木では `TooLarge` で落ちる | 1 |
| N12 | 同上(`#[cfg(unix)]`) | 書き手のいない名前付きパイプ(`mkfifo`)の `.env` は、開かずに `Unreadable`(別のスレッドで呼び、5 秒以内に返る。前例 `intake.rs` の `名前付きパイプは開かずに_unreadable`)。直す前と後の両方で通る(形 B が「止まらない」を保つことの確かめ) | 1 |

- 合計の見込み: **+11 件**(N1〜N8、N10〜N12。N9 は欠番)。実装で増減したら、その数と理由を記録する。

### 5.2 性能の基準(Q3: A)

測り方と直す前の結果は `inventory.md` §0.2(debug、5 回の中央値)。各テストは、同じ処理を **5 回測った中央値** を上限と比べ(`common::median`)、測った値を `eprintln!` で出す(前回の `flow_perf.rs`・`env_merge_perf.rs` と同じ形)。

上限は D6 の決定(ユーザー、2026-11-23)のとおり、**直した後の見積もりの約 5 倍(丸めた数)**の「余裕のある上限」にする。

| テスト | 測る処理 | 直す前 | 直した後の見積もり | 合格の上限 | 直した後の余裕 | 直す前の形との比 |
|--------|----------|--------|--------------------|------------|----------------|------------------|
| P1(N4) | `plan_pipeline`、ジョブ 1,000 個がすべて実行(ステート・`.env` なし)を 1 回 | 99.1 ms | ≒ 31 ms | **150 ms** | 4.8 倍 | 直す前も通る(0.66) |
| P2(N5) | `plan_pipeline`、`@end` の後の 999 個のスキップを 1 回 | 99.8 ms | ≒ 32 ms | **150 ms** | 4.7 倍 | 直す前も通る(0.67) |
| P3(N6) | `build_graph(def, Some(overlay))`、ジョブ 1,000 個を 100 回 | 361 ms | ≒ 71 ms | **350 ms** | 4.9 倍 | 手元でわずかに超える(1.03)。当てにならない |
| P4(N7) | `flow_edges`、ジョブ 1,000 個(遷移 1,996 個)を 100 回 | 658 ms | ≒ 118 ms | **600 ms** | 5.1 倍 | 手元でわずかに超える(1.10)。当てにならない |
| P5(N8) | `parse_env_file`、10,000 行(後半 5,000 行が逆順の重複)を 1 回 | 98.1 ms | ≒ 22 ms | **100 ms** | 4.5 倍 | 直す前も通る(0.98) |

- 余裕の考え方(D6 の決定): 手元(M2)より遅い CI の機械(GitHub Actions の ubuntu・macos・windows、debug。特に Windows。`coverage` は計測つき +3〜9%)で落ちないことを優先し、上限を直した後の見積もりの約 5 倍に置く。**大きな退行(見積もりの 5 倍を超える遅れ)だけを止める**。
- **直す前の形に戻ったことは、このテストでは検知できない**(P1・P2・P5 は直す前の時間でも上限の中、P3・P4 は手元でわずかに超えるだけで、機械の速さによっては通る)。改善が保たれていることは、テストの合否ではなく、直す前と後の時間の記録で示す。
- P6(計画の名前の要約。計算量の改善ではない)は足さない(D6 の決定)。2e の変更は既存の計画のテスト(`plan_types.rs`、`plan_pipeline.rs` の `env_names`)がそのまま確かめる。
- 前後の記録: 段 2 の実装で、各テストを直す前の木(直すコミットの 1 つ前)でも走らせた時間と、直した後の時間を、code-summary(コード生成の記録)と test-results(ビルドとテストの記録)に書く。2e は、§0.2 の M3b と同じ場面を一時に測って記録する(テストは足さない)。コミットは「直すコミット → テストを足すコミット」の順(test-after)。
- 今ある `plan_perf.rs`(1 秒)、`graph_perf.rs`(1 秒)、`flow_perf.rs`、`env_merge_perf.rs` は今のまま。起動の性能(`nfr_perf.rs`)の扱い(警告のみ)は変えない。

## 6. 件数の見込み

| クレート | 単体(src) | 統合(tests/) | 合計 | 基準からの差 |
|---|---|---|---|---|
| core | 115 | 261 + 7(N1、N2、N4〜N8)= **268** | **383** | +7 |
| runtime | 68 | 125 + 2(N11、N12)= **127** | **195** | +2 |
| converter | 54 | 63 + 1(N10)+ 1 無視 = **64 + 1 無視** | **118 + 1 無視** | +1 |
| cli | 0 | 460 + 1(N3)= **461** | **461** | +1 |
| 全体 | | | **1,157 通過、1 無視、0 失敗** | +11 |

ファイルごと: `retry_flow.rs` 17、`env_types.rs` 6、`env_loader.rs` 9(UNIX。Windows は 7)、`convert_ac.rs` 9、`report_messages.rs` 36、新規の `plan_simulate_perf.rs` 2、`graph_lookup_perf.rs` 2、`env_parse_perf.rs` 1。ほかのファイルの件数は変わらない。基準と見込みは macOS の件数(N11・N12 は Windows では数えない)。

## 7. 確かめ方

- 段の前と後で、`cargo test --locked --workspace` の件数をクレートごとに記録する(基準 `inventory.md` §0.1)。合格は、失敗 0 で、全体が「基準 1,146 + 足したテスト」に一致すること [Q1] [Q3]。
- テストのファイルの変更は §4 の T 表と §5 の N 表だけ。段ごとに `git diff --stat -- 'crates/*/tests'` と、src の単体テストの差分を見る。テスト名が変わっていないことは、段の前後で `cargo test --locked --workspace -- --list 2>/dev/null | sort` の差が N1〜N8・N10〜N12 の追加だけであることで確かめる。
- 段 1 の完了: `inventory.md` の各項目の「完了の判定」の検索を 1 つずつ行う [Q1]。たとえば:
  - `grep -rn "ResumeKind\|resume\.kind\|resume\.job_id" crates docs/spec/rust`(0 件)、`grep -rn "is_end:" crates`(0 件)、`grep -rn "multiplier: Option\|max_interval_secs: Option" crates`(0 件)、`grep -rn "note\.\(stopped\|completed\|end_reached\)" crates`(0 件)、`grep -n "ExecutionResult::Exited" crates/cli/src/retry.rs`(0 件)
  - `grep -rn "pub job_id" crates/core/src/types.rs`(`JobState` の 1 件)、`grep -rn "command: job.command.clone()" crates`(0 件)、`grep -rn "pub reason: String" crates/core/src/envfile`(0 件)
  - `grep -rn "fn prepare_run\|fn prepare_pipeline\|definition_path:" crates/cli/src`(0 件)、`grep -rn "Result<(PipelineState, bool)" crates/cli/src`(0 件)
  - `grep -n "to_string()" crates/cli/src/report/messages.rs`(F4 の 4 か所が無い)、`grep -rn '"ステートファイルを読めません"' crates`(runtime の 1 件)
  - 1k: `cargo build --workspace --all-targets --locked` と `cargo clippy … -D warnings` が通ること、`inventory.md` §8 の 1k の検索で外からの参照が 0 件のままであること
- 段 2 の完了: 性能のテスト N4〜N8 が通り、直す前の時間と直した後の時間(2e は M3b の場面の一時の測定)を code-summary と test-results に記録する(上限は大きな退行だけを止めるので、改善の確かめは記録で行う。D6)。`grep -n "state.clone()" crates/core/src/plan/simulate.rs` が出発点の 1 件だけ、`grep -n "\.position(\|\.remove(" crates/core/src/envfile/parse.rs` が 0 件、`grep -n "\.find(" crates/core/src/graph/overlay.rs` が 0 件、`grep -n "job_index" crates/core/src/graph/edges.rs crates/core/src/flow/next_step.rs` が `next_step` の `classify_and_decide` の経路だけ。
- 段 3: N11 が直す前の木で落ち(`TooLarge`)、直した後で通ることを記録する。N12 は直す前と後の両方で通ることを記録する。N10 と ★ T24 が通る。`grep -rn "reason::SYNTAX\|\"文法の誤りです\"" crates/converter/src`(種類の文の 1 件だけ)。仕様書の「v0.9.6 までは…」の注記が rust 03・09、language-agnostic 03・09 にある。
- 版: `grep -rn "0\.9\.6" crates Cargo.lock docs/spec/rust docs/spec/language-agnostic` が「v0.9.6 までは…」の注記だけ。`cargo run -q -p conductor-cli --bin conductor -- --version` が `conductor 0.9.7`。
- 合格の条件: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、全テスト、行カバレッジ 80% 以上(`cargo llvm-cov --fail-under-lines 80`)、MSRV 1.100 でのテスト [memory:M2] [memory:M3] [memory:M4]。新しい依存を足さない(`Cargo.lock` の差分は版の 4 行だけ)[memory:M5]。`unsafe` と src の `cfg` を足さない(runtime と cli の `nfr_source.rs`)。converter の `Cargo.toml` に core を足さない。README と `docs-check` の語句に触れる変更は無い。

## 8. コミットの並び(小さな単位。各コミットでビルド、clippy、テストが通る)

段 1(21 コミット):
1. core・cli: `ResumeInfo` を列挙型に(1a。T1・T2 の該当の行、仕様書 rust・language-agnostic 05)
2. core・cli: `PlanDisposition::WillRun(SimulatedResult)`(1b。T1・T2 の残り、T3、T4、T5 の `:73`、仕様書 05)
3. core・cli: `NextStepNote` を列挙型に(1e。T2 の残り、仕様書 05)
4. core・cli: `GraphNode` の `NodeKind`(1c。T6〜T8、仕様書 02・05)
5. core・cli: `Backoff`(1d。T9、T10 の `RetryPolicy`、T16、T26、仕様書 02・03)
6. core・cli: `RetryDecision::Retry` の `last_exit_code` と `max`(1f。T10 の比較、仕様書 02・05)
7. core のテスト: N1
8. core・runtime: `JobOutcome`・`NextStep` の `job_id` を消す(1g。★ T17a、T19 の `outcome`・`step`、T20 の組み立て、仕様書 02)
9. core・runtime・cli: `JobExecution` の `job_id`・`command` を消し、組み立ての関数から `job` を外す(1g。T5 の `:91`、T10〜T14、T17b、T18、T19 の `exec`、T20 の `:113`、T23、仕様書 02・05・07。runtime の `run_job` の `job` は、このコミットでは `_job` にしておく)
10. runtime・cli: `run_job(line, env, signal)`(1g。T21、T22、仕様書 02・05・07・12 に「v0.9.6 までは `run_job(job, line, env, signal)`」)
11. core: `EnvFileError::reason()` と `render_error`(1h。T15、仕様書 02・10)
12. core のテスト: N2
13. cli: `Checked.base`(1i)
14. cli: `prepare_session`(1i。仕様書 rust 02・04・05・07・10・12)
15. cli: `SkipRecord`(1i。仕様書 02・05)
16. core・converter・runtime: 種類の `label()`(1j の前半。表示は変わらない)
17. cli: `messages.rs` の関数の形の統一と `label()` の使用(1j の後半。仕様書 08)
18. cli のテスト: N3
19. core: `pub` を狭める、根の再公開を名前の一覧に(1k。仕様書 02)
20. cli: `pub` を狭める、`#[doc(hidden)]`(1k)
21. runtime: `pub` を狭める(1k。仕様書 02)

段 2(9 コミット):
22. core: `apply_*_in` と位置の手がかりつきの探索、模擬は複製しない(2a。仕様書 05)
23. core のテスト: N4、N5(直す前の木でも走らせ、時間を記録)
24. core: 重ね合わせを表で引く(2b)
25. core: `flow_edges` の位置の表と `resolve_transition` の引数(2c。仕様書 02・05)
26. core のテスト: N6、N7
27. core: `.env` の重複を位置の表で扱う(2d)
28. core のテスト: N8
29. core: `merge_env_names` と計画の名前の要約(2e。仕様書 05。テストは足さず、前後の時間を一時に測って記録。D6)
30. 仕様書: rust 11 の性能の表(3.1 節)とテストのファイルの表(新しい 3 ファイル)、language-agnostic 11 の性能の表(目安の行)

段 3(7 コミット):
31. converter: 文法の誤りの理由を形式ごとに(3a。★ T24)
32. converter のテスト: N10
33. 仕様書: 文法の誤りの理由(rust 08・09・10・12、language-agnostic 09・10・11・12。「v0.9.6 までは…」)
34. runtime: `.env` の読み込みを形 B に(3b)
35. runtime のテスト: N11、N12
36. 仕様書: `.env` の読み込みの順序(rust 02・03・11、language-agnostic 03・11。「v0.9.6 までは…」)
37. 版を 0.9.7 にする(3c。`Cargo.toml` × 4、`Cargo.lock`、★ T25、仕様書の `0.9.6` の表記 16 ファイル)

合計 **37 コミット**ほど(実装で分けたりまとめたりしたら記録する)。タグとリリースはしない。

## 9. 範囲の外(書かない)

- §3 の「範囲から外したもの」。
- `EdgeList`、`enter_flow`、graph の配置、2 つのテスト用補助バイナリの統合、定義の読み込みの順序(B3。別の intent)。
- converter のほかの細かな点(理由の文の英語版は作らない。今の規則のまま)。
- 凍結した仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)。
- タグとリリース(終わった後に頼まれたら行う)。

## 10. 決めてほしい点

D1〜D10 は、ユーザーが決めた(2026-11-23)。D1〜D5、D7〜D10 は推奨のとおり、D6 は推奨と違う(余裕のある上限。§5.2)。

| ID | 決めること | 推奨 | 代案 | 決定 |
|----|------------|------|------|------|
| D1 | `GraphNode` の `kind` の型(1c) | **`TransitionTarget` の別名 `NodeKind`**(`Job(String)`/`End`。`EdgeTarget` と同じ方法で、同じ形の型を増やさない。終点の語 `END_LABEL` と `label()` も使える) | 新しい列挙型 `NodeKind { Job(String), End }`(意味は「節」で、遷移の行き先と分けられるが、同じ形の型が 3 つ目になる) | 決定: `TransitionTarget` の別名 `NodeKind`(推奨のとおり。ユーザー、2026-11-23) |
| D2 | 読まれない `JobExecution.job_id`・`command` の消し方(1g) | **フィールドを消し、組み立ての関数と runtime の `run_job` から `job` を外す**(使わない引数を公開の関数に残さない)。テストの補助の関数は、引数の名前を `_` で始めて残し、呼ぶ側を変えない(§4 の冒頭) | (B)公開の関数の `job` を `_job` にして残す(呼ぶ側の差分は小さいが、意味の無い引数が残る)。(C)`JobExecution` をなくし `ExecutionResult` を直接使う(`Runtime` トレイト、`classify`、`retry_decision`、`apply_outcome` の型まで変わり、大きい) | 決定: フィールドを消し、組み立ての関数と `run_job` から `job` を外す(推奨のとおり。ユーザー、2026-11-23)。テストの補助の引数を `_` で残す点は「既定で進めるもの」として承認待ち |
| D3 | 準備の入口のまとめ方(1i) | **本番は 1 つの `prepare_session`**: シグナルの監視を始めてから(cli の既存の決まり)、渡された検査を呼ぶ(`fn prepare_session<T>(&mut self, check: impl FnOnce(&mut Self) -> Result<(Checked, T), i32>) -> Result<(Session, T), i32>`。`run` は `check_job` で位置を、`pipeline` は `check_pipeline` で `()` を受け取る)。`--dry-run` は今の `prepare_checked`。「シグナルが先」の決まりが 1 か所に残り、到達しない枝を作らない | (B)`prepare_run`・`prepare_pipeline` を消し、呼ぶ側(`run.rs`・`pipeline.rs`)に「`start_signals` → `check_*` → `Session`」の 3 行を書く(読みやすいが、決まりが 2 か所に書かれる)。(C)位置を `Option` で返す 1 つの関数(`run` に到達しない `None` の枝ができる) | 決定: 総称の `prepare_session` と `prepare_checked` の 2 つ(推奨のとおり。ユーザー、2026-11-23) |
| D4 | converter の `resolve.rs` の 2 か所(到達しない内部の守り)の理由の文(3a) | **`入力の構造を解釈できません`**(形式に依らない。仕様書の理由の一覧に「通常は使われない」と書く。`定義されていないアンカーを参照しています` と同じ扱い) | `YAML として読めません`(エイリアスは YAML だけだが、`Outcome::Internal` は TOML の入力でも通りうる)、または今の `文法の誤りです` を残す(種類の文との重なりが残る) | 決定: `入力の構造を解釈できません`(推奨のとおり。ユーザー、2026-11-23) |
| D5 | 文法の誤りの理由の文(3a) | **`TOML として読めません` / `YAML として読めません`**(Q4 の例のとおり。ステートの `TOML として読めません` と同じ言い方) | `TOML の文法に誤りがあります` / `YAML の文法に誤りがあります`(定義の読み込みの理由(`parse.rs:39`)と同じ言い方) | 決定: `TOML として読めません` / `YAML として読めません`(推奨のとおり。ユーザー、2026-11-23) |
| D6 | 性能のテストの上限(§5.2) | **表のとおり**(P1・P2 は 80 ms、P3 180 ms、P4 300 ms、P5 50 ms、P6 1,500 ms は目安)。P1・P2 は改善が約 3 倍で、手元の検知の幅が 1.24 倍と小さいことを承知で置く | P1・P2 を 60 ms にする(検知の幅 1.65 倍、CI の余裕は約 1.9 倍で、遅い CI で落ちうる)。または P6 を足さない(中身の確認は既存の計画のテストで足りるとみる。件数の見込みは 1,157) | 決定: **推奨と違い、余裕のある上限**(ユーザー、2026-11-23)。理由: 手元(M2)より遅い CI(特に Windows、debug)で落ちないため。各上限を直した後の見積もりの約 5 倍(丸めた数)にし、大きな退行だけを止める: P1・P2 は 150 ms、P3 350 ms、P4 600 ms、P5 100 ms。直す前の形に戻ったことは検知できない(§5.2)。直す前と後の時間は code-summary と test-results に記録する。P6(N9)は足さない(件数の見込みは 1,157) |
| D7 | `NextStepNote` で作れなくなる組(1e) | **「次のジョブが無く、完了でも `@end` でもない」組は `Completed` に写す**(今は `next job ` と空の ID を出すが、`classify_and_decide` はこの組を作らない) | 写す先を `Stopped` にする(どちらも到達しない) | 決定: `Completed` に写す(推奨のとおり。ユーザー、2026-11-23) |
| D8 | 模擬の探索の位置の手がかり(2a) | **ステートのジョブ ID は一意という前提で、定義の位置から探す**(定義の検証と読み込みが重複を拒む。模擬はすでに `status_at` で同じ前提に頼る)。公開の関数は今と同じ先頭からの探索 | 公開の関数と同じく先頭から探す(ジョブ数 × ジョブ数の比較が残り、2a の改善が減る) | 決定: 定義の位置から探す(推奨のとおり。ユーザー、2026-11-23) |
| D9 | `Reporter::language_resolved`・`color_resolved`(テスト専用。1k) | **`pub` のまま `#[doc(hidden)]` と「テスト用(契約の外)」の説明**(テストが使うので狭められない。converter の `code`・`ALL` と同じ扱い) | 何もしない | 決定: `#[doc(hidden)]` と説明を付ける(推奨のとおり。ユーザー、2026-11-23) |
| D10 | 他のクレートが日本語を持つ種類の文(1j) | **core(`DefinitionErrorKind`・`EnvFileErrorKind`)と converter(`ConversionErrorKind`)に `label() -> &'static str` を足し、`Display` はそれを書く**(種類の列挙型から `thiserror` の derive を外し、`Display` と空の `Error` を手で書く。日本語の文は各クレートの 1 か所、cli は `Text::new(英語, kind.label())`)。runtime の `StateErrorKind::label` は公開する | (B)この 4 つの関数は今の「`fn(lang, kind) -> String`、日本語は `Display`」のまま残し、`state_kind` の二重だけを `label()` で結ぶ(他のクレートは変えないが、形は 4 つのまま) | 決定: core と converter に `label()` を足し、`Display` をそれで書く。runtime の `StateErrorKind::label` を公開(推奨のとおり。ユーザー、2026-11-23) |

**既定で進めるもの**(ユーザーが設計の承認で認めた、2026-11-23): §4 の冒頭の「テストの補助の引数は `_` で残す」、T1・T2 の `matches!` と 1 つの値の比較への置き換え、T10 の `wait_of`、★ T17a の 1 行の削除(フィールドが無くなるため)、★ T24 の期待の文字列(Q4 に合わせる)、★ T25(版)。

## 11. 成功の指標との対応(traceability.json)

| ID | 成功の指標(`intent-statement.md`) | 対応する規則 |
|----|------------------------------------|--------------|
| SM1 | 段 1(型と構造): あり得る場合だけを表す型、読まれないフィールドと二重の項目が無い、重複した入口がまとまる、関数の形の統一、`pub` を狭める。動きは変えない | BR1.1〜BR1.11、BR9.1 |
| SM2 | 段 2(計算量): 上限に近い入力の性能のテストを足し、前後の時間を記録。上限は直す前の測定から数で決める | BR2.1〜BR2.6、BR9.1 |
| SM3 | 段 3(利用者から見える変更): 文法の誤りの理由を形式ごとに、`.env` を形 B に。テストで確かめ、値を含めない性質を保ち、仕様書に「v0.9.6 までは…」 | BR3.1、BR3.2、BR8.1 |
| SM4 | 版: 最後のコミットで 0.9.7(タグとリリースはしない) | BR3.3 |
| SM5 | 既存のテストの扱い(T 表と ★ を検索で網羅して承認)と件数(基準 + 足したテスト) | BR8.1、BR9.2、BR9.3 |
| SM6 | 合格の条件(fmt、clippy、カバレッジ 80%、MSRV、依存なし) | BR9.4 |
