# 変える場所の一覧(Inventory): 型と構造の整理・計算量・利用者から見える変更

HEAD f1c9871(v0.9.6 のあとの記録のコミット)の時点で、実物を読んで作った一覧。コードは変えていない。行番号はこの時点のもの。直前の調査の候補と実物が違ったところは「(調査との違い)」と書く。各段の設計の要点、テストの変更の一覧(T…)、足すテスト(N…)、コミットの並び、決めてほしい点(D…)は `functional-spec.md`。

## 0. 基準

### 0.1 テストの件数(`cargo test --locked --workspace`、macOS(Apple M2)、rustc 1.100.0、debug、HEAD f1c9871)

| クレート | 単体(src) | 統合(tests/) | 合計 |
|---|---|---|---|
| core | 115 | 261 | 376 |
| runtime | 68 | 125 | 193 |
| converter | 54 | 63 通過 + 1 無視 | 117 通過 + 1 無視 |
| cli | 0 | 460 | 460 |
| 全体 | | | **1,146 通過、1 無視、0 失敗** |

この intent に関わるファイルの件数(基準): core `plan_pipeline.rs` 10、`plan_run.rs` 5、`plan_assume.rs` 5、`plan_types.rs` 3、`plan_perf.rs` 2、`graph_overlay.rs` 5、`graph_types.rs` 5、`graph_drawable.rs` 21、`definition_retry.rs` 36、`retry_flow.rs` 16、`flow_sequences.rs` 9、`graph_edges.rs` 9、`flow_perf.rs` 1、`env_file_parse.rs` 10、`env_types.rs` 5。runtime `nfr_content.rs` 3、`runner_basic.rs` 12、`runner_env.rs` 5、`runner_signal.rs` 6、`nfr_robust.rs` 6、`flow_with_state.rs` 20、`env_loader.rs` 7。converter `convert_ac.rs` 8。cli `report_messages.rs` 35、`binary_basic.rs` 19、`docs_retry.rs` 4。

### 0.2 性能の直す前の測定(段 2。Q3)

- 測り方: macOS(Apple M2)、rustc 1.100.0、HEAD f1c9871、**debug**(CI の `test` ジョブと同じ `cargo test`)。一時的なテストのファイル `crates/core/tests/tmp_measure_tpv.rs` を作って `cargo test -p conductor-core --locked --test tmp_measure_tpv -- --nocapture --test-threads=1` で測り、**測り終えたあと消した**(`git status` で残っていないことを確かめた。コミットしていない)。
- 各場面を **5 回測った中央値**(と最小・最大)。前回(internal-cleanup-2 の §5)と同じ方法。
- 「直した後の見積もり」は、直した後の形(表で引く、複製しない)を一時のテストの中に書いて測った部品の時間から見積もった(E の行)。

| 場面 | 入力(上限の定数) | 直す前 debug 中央値(最小〜最大) |
|------|------------------|----------------------------------|
| M1: `plan_pipeline`、すべて実行(ステートなし、`.env` なし) | ジョブ 1,000 個(`plan_perf.rs:23` の `N`、仕様書 11 章の「1,000 ジョブ」) | **99.1 ms**(97.3〜107.5) |
| M2: `plan_pipeline`、`run j0` で `@end` に着いた後のステートから(残り 999 個を `apply_skip`) | 同上 | **99.8 ms**(98.9〜100.8) |
| M3: `plan_pipeline`、すべて実行、`[pipeline].env_file` に 1 ファイル | ジョブ 1,000 個、`.env` 10,000 項目(`envfile/mod.rs:41` の `MAX_ENV_ENTRIES`) | **6,207 ms**(6,166〜6,551) |
| M3b: 同上(ジョブを減らす) | ジョブ 100 個、`.env` 10,000 項目 | **611 ms**(601〜622)。2 回目の測定は 593 ms |
| M4: `build_graph(def, Some(overlay))` を 100 回(重ね合わせは定義と同じ順) | ジョブ 1,000 個 | **361 ms**(360〜363) |
| M4r: 同上(重ね合わせが逆順) | 同上 | **360 ms**(360〜369) |
| M4': `build_graph(def, None)` を 100 回(参考。重ね合わせなし) | 同上 | 33.0 ms |
| M5: `flow_edges` を 100 回(各ジョブが成功と警告で後ろのジョブへ遷移を書く) | ジョブ 1,000 個、遷移 1,996 個 | **658 ms**(657〜659) |
| M6: `parse_env_file`(前半 5,000 行が別の名前、後半 5,000 行が逆順の重複) | 10,000 行(`MAX_ENV_ENTRIES`) | **98.1 ms**(97.7〜99.1) |
| M6b: 同上(後半の重複が同じ順) | 同上 | 42.3 ms |
| M6c: 同上(重複なし。参考) | 同上 | 19.6 ms(2 回目 18.3 ms) |

| 見積もりの部品(E) | 測った時間 | 意味 |
|--------------------|------------|------|
| E1a: ジョブ 1,000 個のステートの複製を 1,000 回 | 67.9 ms | M1・M2 のうち、`apply_outcome`・`apply_skip` の複製の分 |
| E1b: `job_states` の線形の探索を 1,000 回 | 3.3 ms | `find_mut` の分 |
| E1c/E1d: `decide_before_run`・`classify_and_decide` を 1,000 回 | 6.3 ms / 3.0 ms | 直さない部分(範囲の外) |
| E3: 名前だけの要約(ジョブ 100 個、`.env` 10,000 項目) | 473 ms | M3b の直した後 |
| E4: 重ね合わせを表(`HashMap`)にして引く、を 100 回 | 38.1 ms | M4 の直した後 = M4' 33.0 + 38.1 ≒ 71 ms |
| E5a / E5b: 遷移先の位置の線形の探索 2,000 回を 100 回 / 表を作って 2,000 回引く、を 100 回 | 597 ms / 56.7 ms | M5 の直した後 = 658 − 597 + 57 ≒ 118 ms |
| E6: 重複を表で扱う(最後の位置だけ残す) | 7.9 ms | M6 の直した後 ≒ M6c 18〜20 + 数 ms ≒ 22 ms |

- **直した後の見積もり**(debug 中央値): M1 ≒ 31 ms(= 99.1 − 67.9)、M2 ≒ 32 ms、M3b ≒ 473 ms、M4 ≒ 71 ms、M5 ≒ 118 ms、M6 ≒ 22 ms。
- **性能のテストの上限**(D6。決定(ユーザー、2026-11-23)): 直した後の見積もりの約 5 倍の余裕のある上限(P1・P2 150 ms、P3 350 ms、P4 600 ms、P5 100 ms。`functional-spec.md` §5.2)。遅い CI でも落ちず、大きな退行だけを止める。直す前の形に戻ったことはテストでは検知できないので、直す前と後の時間を code-summary と test-results に記録する。M3(名前の要約)の性能のテストは足さない。
- **実物で分かったこと(調査との違い)**:
  - 計画(M1・M2)は、複製を除いても、`decide_before_run`(`definition.job_index`、`record_of`、前のジョブの確認が `O(位置)`)と `classify_and_decide`(`job_index`)が、ジョブごとに線形の探索をする。これは公開の関数の契約(引数はジョブ ID)の中の処理で、この intent の範囲(初期の記述の (4))の外。したがって計画は、**複製をやめても、安い比較の二乗の時間が残る**。改善は約 3 倍(複製の割り当てがなくなる)で、二乗が線形になるわけではない。§段 2 の 2a と `functional-spec.md` の D6 に書く。
  - 名前の要約(M3)は、出力そのもの(ジョブごとに 10,004 個の名前)が「ジョブ × 項目」の大きさなので、値を複製しなくても約 2 割しか速くならない(611 → 473 ms)。計算量の改善ではなく、無駄な複製(秘密の値の写し)をなくす変更である。
  - 重ね合わせ(M4)は、逆順でも同じ時間(線形の探索の平均は順に依らない)。表で引くと、順に依らず線形になる。

---

## 段 1(型と構造。利用者から見える動きは変えない)

### 1a. `ResumeInfo` の `kind` と `job_id: Option`

- 今: `crates/core/src/plan/mod.rs:116-125`(`ResumeKind { FromStart, AfterStop, Individual }`)、`:127-133`(`ResumeInfo { kind, job_id: Option<String> }`。`job_id` は「`AfterStop` と `Individual` のとき」)。作る所: `plan/simulate.rs:136-145`、`:225-228`。読む所: `crates/cli/src/report/plan_text.rs:54-75`(`match (plan.resume.kind, &plan.resume.job_id)` の 2 つの枝と `_` の枝。`_` は `FromStart` と、あり得ない「`AfterStop`/`Individual` で `None`」をまとめる)。
- 目標: `pub enum ResumeInfo { FromStart, AfterStop(String), Individual(String) }`。`ResumeKind` を消す。`plan_text.rs` は網羅の `match &plan.resume`(`_` の枝なし)。
- 再公開: `crates/core/src/lib.rs:110` から `ResumeKind` を消す。
- 仕様書: rust `05-execution-flow.md:859`、language-agnostic `05-execution-flow.md:869`(論理の形を「3 つの場合。`afterStop`・`individual` はジョブ ID を持つ」に)。
- 影響するテスト: T1(`plan_pipeline.rs`)、T2(`plan_run.rs`)。
- 完了の判定: `grep -rn "ResumeKind\|resume\.kind\|resume\.job_id" crates docs/spec/rust` が 0 件。

### 1b. `PlanEntry.result: Option<SimulatedResult>`(`WillRun` のときだけ `Some`)

- 今: `plan/mod.rs:58-71`(`PlanDisposition`。`Copy`)、`:73-84`(`SimulatedResult`。`Copy`)、`:108-109`(`result`)。作る所: `simulate.rs:76-82`、`:96-102`、`:120-126`、`:192`、`:216`、`:304-336`(`entry` の引数 `result`)。読む所: `plan_text.rs:145`(`!= PlanDisposition::WillRun`)、`:188`、`:210`、`messages.rs:683-694`(`plan_disposition`)。
- 目標: `PlanDisposition::WillRun(SimulatedResult)`(`Copy`・`Eq` のまま)。`PlanEntry.result` を消す。読み方の補助 `impl PlanDisposition { pub fn result(self) -> Option<SimulatedResult> }`(`WillRun(r)` だけ `Some(r)`)。`simulate.rs` の `entry` は `disposition` だけを受け取る。`plan_text.rs` は `let PlanDisposition::WillRun(result) = entry.disposition else { return }` と `entry.disposition.result()`。
- 仕様書: rust `05:854-857`、`05:893-895`、language-agnostic `05:865-867`。
- 影響するテスト: T1、T2、T3(`plan_assume.rs`)、T4(`plan_types.rs`)、T5(`plan_perf.rs:73`)。`== D::AlreadyExecuted` などの単位の値との比較は、そのままコンパイルできる(変えない)。
- 完了の判定: `grep -rn "\.result\b" crates/core/src/plan crates/cli/src/report/plan_text.rs` が 0 件、`grep -rn "result: Option<SimulatedResult>" crates` が 0 件。

### 1c. `GraphNode { job_id: Option, is_end: bool }`

- 今: `crates/core/src/graph/mod.rs:79-89`(`is_end` は `job_id.is_none()` と同じ)。作る所: `graph/overlay.rs:64-77`。読む所: `graph/layout.rs:65`、`cli/src/report/graph_text.rs:49-58`(`node_label`)、`:196`、`:238`、`graph_mermaid.rs:58-70`。
- 目標: `GraphNode { kind: NodeKind, state, is_resume }`。`NodeKind` は遷移の行き先の型の別名 `pub use crate::types::TransitionTarget as NodeKind;`(`Job(String)` / `End`。`EdgeTarget` と同じ方法。D1。決定(ユーザー、2026-11-23))。読み方の補助 `impl GraphNode { pub fn job_id(&self) -> Option<&str>; pub fn is_end(&self) -> bool }`。`graph_text.rs` の `node_label` は `match &node.kind`(終点は `NodeKind::END_LABEL`)。
- 仕様書: rust `02-architecture.md:290`、`05:738`、`05:767`、language-agnostic `05:750`、`05:778`。
- 影響するテスト: T6(`graph_overlay.rs`)、T7(`graph_types.rs`)、T8(`graph_drawable.rs`)。
- 完了の判定: `grep -rn "is_end:" crates` が 0 件、`grep -rn "job_id: Option<String>" crates/core/src/graph` が 0 件。

### 1d. `RetryPolicy` の `multiplier` と `max_interval_secs`

- 今: `crates/core/src/types.rs:114-129`(2 つの `Option`。両方あるか両方ないかは `definition/walk/retry.rs:69-75` の `RetryBackoffIncomplete` で検証済み)。作る所: `walk/retry.rs:249-259`(`build_retry`)。読む所: `flow/retry.rs:50-63`(`next_wait_secs` の `(Some, Some)` と `_ =>` の枝。片方だけの組は `_` に落ちる)、`cli/src/report/messages.rs:596`、`:608`(`graph_retry_summary`)。
- 目標: `pub struct Backoff { pub multiplier: u32, pub max_interval_secs: u64 }`(`Copy`)と `RetryPolicy { exit_codes, max, interval_secs, backoff: Option<Backoff> }`。`build_retry` は `raw.multiplier.zip(raw.max_interval.map(|(s, _)| s)).map(|(multiplier, max_interval_secs)| Backoff { .. })`(誤りがあれば定義の組み立ての前に失敗しているので、片方だけの組はここに来ない。来ても `None` で、今の「片方だけなら固定の間隔」と同じ)。`next_wait_secs` は `match policy.backoff { Some(b) => …, None => policy.interval_secs }`。
- 再公開: `Backoff` を `lib.rs` の型の一覧に足す(1k で `types::*` を名前の一覧にするとき)。
- 仕様書: rust `02:78`、`03-definition-format.md:241`(型の列)、`:601-606`、language-agnostic `03:771-776`。
- 影響するテスト: T9(`definition_retry.rs`)、T10(`retry_flow.rs`)、T16(`types.rs` の単体テスト)、T26(`cli/tests/docs_retry.rs:71-72`)。
- 完了の判定: `grep -rn "multiplier: Option\|max_interval_secs: Option\|(policy.multiplier" crates` が 0 件。

### 1e. `NextStepNote` の 3 つの真偽値

- 今: `plan/mod.rs:146-156`(`next_job_id`、`skipped_job_ids`、`completed`、`end_reached`、`stopped`)。作る所: `simulate.rs:200-206`。読む所: `cli/src/report/messages.rs:748-778`(`plan_next_step`。`stopped` → 停止、`end_reached && !completed` → 「呼ばれたときにスキップ」、`completed` → 完了、それ以外 → 次のジョブ。次のジョブが無いときは `unwrap_or_default()` の空文字)、`plan_text.rs:202-229`。
- 目標:

  ```rust
  pub enum NextStepNote {
      /// 次のジョブ(遷移で飛び越すジョブ。無ければ空)。
      NextJob { job_id: String, skipped_job_ids: Vec<String> },
      /// `@end` に到達した(後ろのジョブは、呼ばれたときにスキップされる)。完了していない。
      EndReached,
      /// 完了する。
      Completed,
      /// エラーで停止する。
      Stopped,
  }
  ```

  作り方(`simulate.rs`): エラー → `Stopped`、`next.next_job_id` が `Some(id)` → `NextJob { id, next.skipped_job_ids }`、`next.end_reached && !next.completed` → `EndReached`、それ以外 → `Completed`。表示(`plan_next_step`)は網羅の `match`。
- 動きが変わらない根拠: `classify_and_decide` の `NextStep` は、成功・警告では「次のジョブがある」「`@end`(個別実行で残りあり → 未完了)」「完了」のどれか 1 つ(`next_step.rs:145-181`)。エラーは次のジョブなし。今の表示の 4 つの枝と同じ順で同じ文になる。今の「次のジョブが無く、完了でも `@end` でもない」組(表示は `next job `)は作られない。新しい型では作れず、仮に来ても `Completed` になる(D7。決定(ユーザー、2026-11-23))。
- `ExecutionPlan.end`(`PlanEnd`)は今のまま(`pipeline` が使う。`run` では参考の値)。`NextStepNote::EndReached` は「未完了の `@end`」で、`PlanEnd::EndReached`(「`@end` に到達して完了」)と意味が違うため、型を共有しない。
- 仕様書: rust `05:861`、`05:910`、language-agnostic `05:871`、`05:919`。
- 影響するテスト: T2(`plan_run.rs:74-120`)。
- 完了の判定: `grep -rn "note\.\(stopped\|completed\|end_reached\)" crates` が 0 件。

### 1f. `RetryDecision::Retry` に最後の終了コードと回数

- 今: `crates/core/src/flow/retry.rs:22-28`(`Retry { wait_secs }`)。`cli/src/retry.rs:65-90`: `Retry` を受けたあと、2 つ目の let-else で `job.retry` と `ExecutionResult::Exited { exit_code: Some(..) }` を取り出し直す(前の intent の 4d で既定値は消えたが、「`Retry` が返るのはこの組のときだけ」を呼ぶ側がもう一度書いている。else は到達しない)。
- 目標: `Retry { wait_secs: u64, last_exit_code: i32, max: u32 }`。`retry_decision` は `is_retry_target` の代わりに、対象の終了コードを返す内部の関数(`retry_target_code(policy, execution) -> Option<i32>`。`is_retry_target` はこれの `is_some()` にして、`--dry-run` の模擬が使い続ける)から `last_exit_code` を、`policy.max` から `max` を入れる。`cli/src/retry.rs` は `let RetryDecision::Retry { wait_secs, last_exit_code, max } = … else { return execution; };` の 1 つにし、2 つ目の let-else を消す。
- 仕様書: rust `02:57`、`02:90`、`05` の擬似コード(`retry_decision` の行)。
- 影響するテスト: T10(`retry_flow.rs` の比較の書き方)。新しいテスト N1。
- 完了の判定: `grep -n "ExecutionResult::Exited" crates/cli/src/retry.rs` が 0 件。

### 1g. 読まれないフィールド(契約 C1)

- 今:
  - `JobExecution.job_id`・`command`(`types.rs:203-211`。`new` が毎回 `job.command.clone()`。`:232-263`)。読む所は無い(`grep -rnE "(execution|exec|e)\.(job_id|command)\b" crates` で 0 件。`JobExecution` はステートに書かれない)。
  - `JobOutcome.job_id`(`types.rs:437-447`。作る所 `flow/classify.rs:32-45`)。読む所は `classify.rs:177` の単体テストの 1 行だけ。
  - `NextStep.job_id`(`types.rs:449-465`。作る所 `flow/next_step.rs:92-102` の `empty_step`)。読む所は無い。
- 目標:
  - `JobExecution { result: ExecutionResult }`。組み立ての関数は `job` を受け取らない: `new(result)`、`exited(exit_code)`、`not_started(signal)`、`start_failed()`(D2。決定(ユーザー、2026-11-23))。runtime の `run_job` も `job` を使わなくなるので、`run_job(line, env, signal)` にする(`crates/runtime/src/runner.rs:101-144`。仕様書に「v0.9.6 までは `run_job(job, line, env, signal)`」)。cli の `Runtime::run_job(job, line, env, signal)`(偽物がジョブ ID を記録する)は今のまま、`SystemRuntime` の実装だけ `job` を渡さない(`deps.rs:182`)。
  - `JobOutcome` と `NextStep` から `job_id` を消す。`next_step.rs` の `empty_step(job)` は `empty_step()` に。
- 仕様書(契約 C1): rust `02:32`、`02:79`、`02:86`、`02:87`、`02:108`、`02:255`、`05:27`、`05:518`、`05:525`、`05:893`、`07:177-179`、`07:208-220`、`12:106`。language-agnostic は、これらのフィールドを書いていない(`05:27-35`、`05:49-56`、`05:155-164` を確かめた)ので、変えない。
- 影響するテスト: T5(`plan_perf.rs:91`)、T10〜T14、T17b〜T23(組み立ての関数と `run_job` の引数)。★ T17a の `classify.rs:177`(`o.job_id` の確認の 1 行を消す。`JobOutcome.job_id` を消すコミット 8 で。T17b の `exec` の本体はコミット 9 で)。
- 完了の判定: `grep -rn "pub job_id" crates/core/src/types.rs` が `JobState` の 1 件だけ、`grep -rn "command: job.command.clone()" crates` が 0 件。

### 1h. `EnvFileError.reason: String` と 2 つの `render`

- 今: `crates/core/src/envfile/mod.rs:159-189`(`reason` は `new` の中でいつも `kind.reason().to_string()`。`:178`)。cli は `env_file_reason(lang, e.kind)`(`messages.rs:328-353`)を使い、`e.reason` を読まない。読むのはテスト `env_file_parse.rs:179` だけ。`render` は `envfile/mod.rs:182-188` と `error.rs:220-226`(`DefinitionError`)で同じ書式。
- 目標: フィールドを消し、`pub fn reason(&self) -> &'static str { self.kind.reason() }`。書式は `error.rs` の `pub(crate) fn render_error(file: &str, line: Option<usize>, kind: &str, reason: &str) -> String`(ファイル名の `sanitize` を含む)を 2 つの `render` が使う。converter の同じ形(`converter/src/error.rs:111-121`)は、converter が core に依存しないので、そのまま(範囲の外)。
- 仕様書: rust `02:99`、`10-errors-and-exit-codes.md:122`。
- 影響するテスト: T15(`env_file_parse.rs:179`)。新しいテスト N2。
- 完了の判定: `grep -rn "pub reason: String" crates/core/src/envfile` が 0 件、`grep -rn '{file}:{line}: {}: {}' crates/core/src` が 1 件。

### 1i. cli の `Checked` と `CheckedDefinition`、準備の入口、`record_skip`

- 今:
  - `crates/cli/src/ctx.rs:48-54`(`CheckedDefinition { path, definition, commands }`)と `:56-71`(`Checked { definition_path, definition, commands, run_id, state, remove_on_complete, env_files }`)。`check_run_inputs`(`:247-277`)が分解して詰め直す。
  - 入口 3 つ: `prepare_run`(`:136-144`。シグナル → `check_job(.., &[])` → `(Session, usize)`)、`prepare_pipeline`(`:148-156`。シグナル → `check_pipeline(.., &[])` → `Session`)、`prepare_checked`(`:164-176`。`FlowTarget` で `check_pipeline`・`check_job` を呼び分け、位置を捨てる)。呼ぶ所: `run.rs:42`、`pipeline.rs:43`、`dry_run.rs:56`。`enter_flow`(`dry_run.rs:35-51`)は触らない。
  - `step.rs:79-104` の `record_skip` が `(PipelineState, bool)` を返す。呼ぶ所: `run.rs:54-60`(`changed` で削除を決める)、`pipeline.rs:66-69`(`_changed`)。
- 目標:
  - `Checked { base: CheckedDefinition, run_id, state, remove_on_complete, env_files }`(項目の二重をなくす)。読む所は `session.checked.base.definition` などに(`run.rs:46`、`:83`、`step.rs:86-87`、`:126-127`、`:176`、`pipeline.rs:47-48`、`:54`、`dry_run.rs:60`、`:63`、`retry.rs:54`、`:58`)。
  - 入口は 2 つ: 本番の `prepare_session`(シグナルの監視を始めてから(cli の既存の決まり)、渡された検査を呼ぶ。`run` は `check_job` で位置も、`pipeline` は `check_pipeline` で `()` を受け取る。D3。決定(ユーザー、2026-11-23))と、`--dry-run` の `prepare_checked`(今のまま)。`prepare_run`・`prepare_pipeline` を消す。
  - `record_skip` は `SkipRecord { state: PipelineState, written: bool }` を返す(`written` は「土台から変わり、書いた」)。
- 仕様書: rust `02:186-227`(流れの図)、`02:356-358`、`05:257-259`、`05:285`、`05:311-315`、`05:344-350`、`07:287-289`、`10:18`、`12:85`、`12:127`、`04:238`。
- 影響するテスト: なし(非公開のモジュール)。
- 完了の判定: `grep -rn "fn prepare_run\|fn prepare_pipeline\|definition_path:" crates/cli/src` が 0 件、`grep -rn "Result<(PipelineState, bool)" crates/cli/src` が 0 件。

### 1j. cli `report/messages.rs` の関数の形と、ステートの誤りの日本語の二重

- 今の形(`crates/cli/src/report/messages.rs`):
  - 形 F1(固定の文の定数 `pub const X: Text`): 多数。問題なし。
  - 形 F2(種類ごとの固定の文 `fn(kind) -> Text`。言語を受け取らない): `usage_kind`(`:228`)、`command_kind`(`:371`)、`state_kind`(`:382`)、`reset_hint`(`:432`)、`mismatch_kind`(`:473`)、`reject_reason`(`:493`)、`skip_reason`(`:533`)。
  - 形 F3(値を埋める文 `fn(lang, 値…) -> String`、`match lang` で `format!`): `init_created`(`:143`)、`init_replaced`(`:151`)、`init_exists`(`:159`)、`init_cannot_create`(`:168`)、`definition_valid`(`:194`)、`retry_scheduled`(`:208`)、`env_errors_more`(`:356`)、`env_duplicate_key`(`:364`)、`graph_retry_summary`(`:579`)、`graph_edge_note`(`:617`)、`plan_run_id_suffix`(`:667`)、`plan_resume_after_stop`(`:675`)、`plan_assumed`(`:708`)、`plan_stopped_at_error`(`:740`)、`plan_next_step`(`:748`)。
  - **形 F4**(固定の文なのに `fn(lang, kind) -> String` で、日本語は他のクレートの `Display` に任せる): `definition_kind`(`:254-297`。`kind.to_string()`)、`env_file_kind`(`:300-321`)、`env_file_reason`(`:328-353`。`kind.reason().to_string()`)、`conversion_kind`(`:453-467`)。
  - **形 F5**(固定の文なのに `fn(lang, x) -> &'static str`): `plan_disposition`(`:683-694`)、`class_word`(`:697-704`)。
  - 述語(文でない): `state_kind_suggests_reset`(`:409`)。形の統一の対象外。
  - 二重: `state_kind` の日本語 6 つ(`:383-405`)は、runtime の `StateErrorKind::label()`(`crates/runtime/src/error.rs:61-72`。非公開)と同じ文を別に持つ。一致を確かめる仕組みが無い。
- 目標(統一後の形は 3 つだけ):
  - F1: 固定の文の定数 `Text`(今のまま)。
  - F2: 種類ごとの固定の文は `fn(kind) -> Text`。**日本語を他のクレートが持つ種類は、`Text::new(英語, kind.label())` で、日本語の文を cli に書かない**(一致は型で結ぶ)。
  - F3: 値を埋める文は `fn(lang, 値…) -> String`(今のまま)。
  - 変える関数: `definition_kind(kind) -> Text`(日本語 `DefinitionErrorKind::label()`)、`env_file_kind(kind) -> Text`(`EnvFileErrorKind::label()`)、`env_file_reason(kind) -> Text`(`EnvFileErrorKind::reason()`)、`conversion_kind(kind) -> Text`(`ConversionErrorKind::label()`)、`state_kind(kind) -> Text`(`StateErrorKind::label()`)、`plan_disposition(d) -> Text`、`class_word(c) -> Text`。呼ぶ側は `.get(lang)`(`report/mod.rs:528`、`:539-540`、`:605`、`:618`、`graph_text.rs:83-84`、`plan_text.rs:132`、`messages.rs:715`)。
  - 他のクレートに足すもの(D10。決定(ユーザー、2026-11-23)): core `DefinitionErrorKind::label(self) -> &'static str`、`EnvFileErrorKind::label(self) -> &'static str`、converter `ConversionErrorKind::label(self) -> &'static str`(今の `#[error("…")]` の文を `label` に移し、`Display` は `f.write_str(self.label())`、`std::error::Error` は空の実装。種類の列挙型から `thiserror` の derive だけを外す。構造体の誤りの型は今のまま)。runtime `StateErrorKind::label` を `pub` に。表示の文は 1 文字も変わらない。
- 仕様書: rust `08-output-and-i18n.md:126-128`、`:184`、`:247`、`:323`(関数の名前と「日本語は … の `label()`」)。
- 影響するテスト: なし(`report::messages` はテストから参照されない。`grep -rn "messages::" crates/cli/tests` で 0 件)。表示は `report_messages.rs` などがそのまま確かめる。新しいテスト N3。
- 完了の判定: `grep -n "to_string()" crates/cli/src/report/messages.rs` で F4 の 4 か所が消える、`grep -n '"ステートファイルを読めません"' crates` が runtime の 1 件だけ、`grep -n "fn plan_disposition\|fn class_word" crates/cli/src/report/messages.rs` の戻り値が `Text`。

### 1k. 外から使われない `pub` を狭める

調べ方: 各名前について、`grep -rn "<名前>" crates --include='*.rs' | grep -v "^crates/<自分のクレート>/src"` で、ほかのクレートの src と、すべてのクレートの `tests/`・`src/bin` から参照が無いことを確かめた(§8 に検索のコマンド)。テストは別のクレートなので、テストが使う `pub` は狭められない。

| クレート | 今 | 目標 | 外からの参照 |
|----------|----|------|--------------|
| core | `definition/mod.rs:20` `pub mod depth`、`:23` `pub mod parse`、`:24` `pub mod version`、`:25` `pub mod walk` | `mod`(`pub(crate)`)。`intake` は `pub` のまま(cli の `resolve_path`・`read_definition`・`DEFAULT_DEFINITION_FILE`) | 0 件 |
| core | `definition/depth.rs:31` `pub fn find_too_deep` | 非公開の `fn` | 0 件 |
| core | `definition/intake.rs:42` `pub fn file_label` | `pub(crate)`(converter の `file_label` は自前の別の関数) | 0 件 |
| core | `flow/next_step.rs:62` `TransitionResolution`、`:75` `resolve_transition`、`flow/mod.rs:31` と `lib.rs:104` の再公開 | `pub(crate)`。再公開を消す | 0 件 |
| core | `envfile/mod.rs:274` `LoadedEnvFiles::add` | **狭めない**(テスト `plan_types.rs:63`、`env_merge.rs`、`env_merge_perf.rs`、`plan_perf.rs:44`、cli `plan_report.rs:417` が使う) | テストが使う |
| core | `lib.rs:100` `pub use error::*` | 名前を並べる: `DefinitionError, DefinitionErrorKind, MAX_KEY_DISPLAY_CHARS, escape_display, sanitize` | — |
| core | (調査との違い)`error.rs:286` `quote_key`、`:294` `Locate` | `pub(crate)`(`error::*` で根に出ていたが、外から使われない) | 0 件 |
| core | `lib.rs:116` `pub use types::*` | 名前を並べる: `Backoff, Classification, Command, Decision, EnvVar, ExecutionResult, JobDefinition, JobExecution, JobOutcome, JobState, JobStatus, NextStep, PipelineDefinition, PipelineState, RejectReason, ResumePoint, RetryPolicy, RunMode, SkipCause, SkipReason, Transition, TransitionOn, TransitionTarget`(今 `*` で出ている名前と同じ + `Backoff`) | — |
| cli | `report/mod.rs:31` `pub mod messages` | `pub(crate) mod`(`panic_hook.rs:28` が crate の中から使う) | 0 件 |
| cli | `report/mod.rs:33` `pub mod watch` | `mod`(`render_table` は `pub(crate)`) | 0 件 |
| cli | `report/watch.rs:31` `status_word`、`graph_text.rs:38` `state_symbol`、`:163` `glyph_char` | 非公開(`state_symbol` は `graph_mermaid.rs:27` が使うので `pub(crate)`) | 0 件 |
| cli | `report/escape.rs:24` `MAX_DISPLAY_CHARS` | 非公開の `const` | 0 件 |
| cli | `types.rs:108` `DEFAULT_INIT_PIPELINE_NAME`、`:111` `DEFAULT_INIT_PATH` | `pub(crate)` | 0 件 |
| cli | `lib.rs:92` `pub mod platform` | `mod`(関数は `pub(crate)`) | 0 件(`main.rs`、`src/bin` も使わない) |
| cli | `report/mod.rs:290` `language_resolved`、`:295` `color_resolved`(テスト専用) | **狭めない**(`report_messages.rs` の「言語と色の判定は_最初に必要になるまで行われない」が使う)。`#[doc(hidden)]` を付け、説明に「テスト用(契約の外)」と書く(converter の `code`・`ALL` と同じ扱い) | テストが使う |
| runtime | `lib.rs:122` `pub mod error`、`:129` `pub mod types` と `:132`・`:137` の `pub use …::*` | `mod` にし、名前を並べる: `pub use error::{CommandError, CommandErrorKind, StateError, StateErrorKind}`、`pub use types::{CommandLine, ConsistencyResult, Durability, InterruptSignal, MismatchKind, StateLocation}` | `conductor_runtime::error::`・`::types::` は 0 件 |

- 狭めたあとの確かめ方: (1) `cargo build --workspace --all-targets --locked` と `cargo test --locked --workspace` が通る(テストと他のクレートは別のクレートなので、参照が残っていればコンパイルできない)。(2) `cargo clippy --workspace --all-targets --locked -- -D warnings` が通る(狭めて使われなくなった項目があれば `dead_code` で止まる。止まったら、その項目を記録して消す)。(3) §8 の検索をもう一度行い、表の「0 件」が変わらないこと。
- `lib.rs:41-54` のモジュールの表の内部リンク(`[`definition::parse`]` など)は、非公開になるのでリンクをやめて `` `definition::parse` `` にする(`cargo doc` の `private_intra_doc_links` の警告を出さない。CI は `cargo doc` を走らせないが、そろえる)。
- 仕様書: rust `02-architecture.md:42`(再公開の一覧)、`02:104`。

---

## 段 2(計算量。利用者から見える動きは変えない)

### 2a. 計画を作るときのステートの複製と `find_mut`

- 今: `crates/core/src/flow/record.rs:82-120`(`apply_outcome`。`state.clone()` してから更新)、`:130-169`(`apply_skip`。`AlreadyExecuted` でも `state.clone()`)、`:175-181`(`apply_interrupt`)、`:67-69`(`find_mut` の線形の探索。`apply_outcome` はスキップのジョブごとにも呼ぶ。`:110`)。`plan/simulate.rs:103`、`:116` がジョブごとに呼ぶ(ジョブ数 × ステートの大きさ = 二乗の複製)。M1・M2 の約 68% が複製(§0.2)。
- 目標:
  - crate の中だけの「その場で書き換える」版 `pub(crate) fn apply_outcome_in(state: &mut PipelineState, hint: usize, job_id, execution, outcome, next)`、`apply_skip_in(state: &mut, hint, job_id, reason, definition)`、`apply_interrupt_in(state: &mut, resume_point)` を置き、公開の `apply_outcome`・`apply_skip`・`apply_interrupt` は「`state.clone()` して `_in` を呼び、返す」だけにする(公開の型と結果は同じ)。
  - 探索は `locate(states, from, job_id) -> Option<usize>`(`states[from..]` を先に、なければ `states[..from]` を探す)。公開の関数は `from = 0`(今の `find` と同じ、最初に一致したもの)。模擬は `from = 定義の位置`、スキップの並びは「直前に見つけた位置 + 1」から探す(ステートが定義の順に並んでいれば、各探索は O(1))。
  - `simulate.rs` は `apply_outcome_in(&mut state, index, …)`、`apply_skip_in(&mut state, index, …)` を使い、ステートを複製しない(出発点の 1 回の `clone` は残る)。
- 動きが変わらない根拠: 公開の関数は今と同じ値を返す(複製してから同じ手順)。模擬の `from = 位置` が今の `find` と違う結果になるのは、ステートに同じジョブ ID が 2 つあるときだけ。ステートは `initial_state`(定義のジョブ ID は一意。`DuplicateJobId`)か、読み込み(`state/reader.rs:318` が `DUPLICATE_JOB` で拒む)から来る。模擬はすでに `status_at`(`flow/mod.rs:52-57`。位置で引く)に同じ前提で頼っている(前の intent の決定。D8。決定(ユーザー、2026-11-23))。
- 残る二乗(範囲の外): `decide_before_run` の前のジョブの確認と `job_index`・`record_of`、`classify_and_decide` の `job_index`、`apply_skip` の「エラーがあるか」と「最初の pending」の走査(どれも 1 回あたり O(n) の安い比較)。§0.2 のとおり、計画は約 3 倍速くなるが、二乗の比較は残る。
- 仕様書: rust `05:172`(「`S` を複製して更新する」は公開の関数の説明として残し、「模擬は、crate の中のその場で書き換える版を使う(結果は同じ)」を足す)、`05:5`。
- 影響するテスト: なし(公開の関数の形と結果は同じ)。新しいテスト N4、N5。

### 2b. 図の重ね合わせの線形の探索

- 今: `crates/core/src/graph/overlay.rs:52-82`(`build_graph`。ジョブごとに `o.states.iter().find(..)`。`:57-62`)。
- 目標: `overlay` があれば、先に `HashMap<&str, NodeState>` を作り(**最初に現れたものを残す**: `entry(id).or_insert(state)`)、ジョブごとに引く。重ね合わせに無いジョブは今と同じく `Pending`。
- 動きが変わらない根拠: `find` は最初に一致したものを返す。表も最初のものを残すので、同じジョブ ID が重なる入力でも結果は同じ。
- 影響するテスト: なし。新しいテスト N6。

### 2c. 遷移先の解決の `job_index` と `flow_edges`

- 今: `crates/core/src/flow/next_step.rs:75-89`(`resolve_transition` が遷移先ごとに `definition.job_index(target)`。線形)。`graph/edges.rs:26-38`、`:47-89`(`flow_edges` が全ジョブ × 成功・警告の 2 回呼ぶ)。M5 の約 91% が `job_index`(§0.2)。`next_step.rs:126` の `definition.job_index(&job.id)` は、1 回の実行で 1 回なので今のまま。
- 目標: `resolve_transition(job, on, index, position: impl Fn(&str) -> Option<usize>)`(`pub(crate)`。1k で公開をやめる)。`classify_and_decide` の `next_step` は `|id| definition.job_index(id)` を渡す(今と同じ)。`flow_edges` は最初に `HashMap<&str, usize>`(最初に現れた位置を残す)を作って、それを引く関数を渡す。
- 動きが変わらない根拠: `job_index` は最初に一致した位置を返す。表も最初の位置を残す。`classify_and_decide` の経路は今と同じ関数を使う。
- 仕様書: rust `02:58`(1k で公開をやめるので、行を「`flow::next_step` の crate の中の関数」に)、`05:166`、`05:751`。language-agnostic `05:188`、`05:762` は論理の説明(関数の引数の形は書き方の自由)なので、そのまま。
- 影響するテスト: なし。新しいテスト N7。

### 2d. `.env` の重複キーの位置の探索と削除

- 今: `crates/core/src/envfile/parse.rs:77-96`(重複のたびに `entries.iter().position(..)` と `entries.remove(pos)`。`:79-81`)。
- 目標: `seen` の表(小文字の名前 → 綴り)に、項目の位置も持たせる(`HashMap<String, (String, usize)>`)。完全に同じ名前の重複は、前の位置の項目を「消した」印にし(`entries: Vec<Option<EnvEntry>>`)、新しい項目を末尾に足して位置を更新する。最後に印の付いた項目を除く。
- 動きが変わらない根拠: 今は「前の項目を取り除き、末尾に足す」ので、残る並びは「各名前の最後の項目を、最後に現れた順に並べたもの」。印で消して最後に除いても同じ並びになる。警告(`DuplicateKey`)、`CaseConflict`、件数の上限(`taken`)の扱いは変えない。
- 影響するテスト: なし(`env_file_parse.rs` の重複の並びの検査がそのまま通ること)。新しいテスト N8。

### 2e. 計画の名前の要約が値まで複製する

- 今: `crates/core/src/plan/simulate.rs:304-336`(`entry` が `merge_env(..)` で値まで `to_string` した `EffectiveEnv` を作り、`.names()` で名前だけ取り、さらに `name.to_string()`)。`envfile/merge.rs:28-56`、`:65-80`(`keep_last`)。
- 目標: `merge.rs` で、層を並べる部分(`collect_items`)と「名前ごとに最後の位置だけ残す」部分(`keep_last_positions(items) -> impl Iterator<Item = &Item>`)を分け、`merge_env` と、新しい `pub(crate) fn merge_env_names<'a>(def: &'a PipelineDefinition, job: &'a JobDefinition, files: &'a LoadedEnvFiles, ctx: &EnvContext<'_>) -> Vec<(&'a str, EnvOrigin)>`(値を読まない)が共有する。自動の変数の名前は固定の文字列(`CONDUCTOR_RUN_ID` は `ctx.run_id` があるときだけ)。`entry` は `merge_env_names` を使う。
- 動きが変わらない根拠: 名前と出どころと並びは、同じ並べ方と同じ「最後に残す」規則で決まる。値は計画に入らない(今も入らない。BR5.1)。
- 効果: 約 2 割(§0.2 の M3b 611 ms → E3 473 ms)。秘密の値の写しを作らなくなる(計画は値を読まない)。
- 仕様書: rust `05:895`(`env_names` の作り方)。
- 影響するテスト: なし。性能のテスト(P6・N9)は足さない(D6。決定(ユーザー、2026-11-23))。既存の計画のテスト(`plan_types.rs`、`plan_pipeline.rs` の `env_names`)がそのまま確かめる。前後の時間は §0.2 の M3b と同じ場面を一時に測り、code-summary と test-results に記録する。

---

## 段 3(利用者から見える変更と版)

### 3a. converter の文法の誤りの理由の文(Q4: A)

- 今: `crates/converter/src/error.rs:138` `reason::SYNTAX = "文法の誤りです"`(種類の表示 `ConversionErrorKind::Syntax` の `#[error("文法の誤りです")]`(`:45`)と同じ文)。使う所: `toml_read.rs:67`(TOML の読み取り器の誤り)、`yaml_read.rs:216`(YAML のパーサーの誤り)、`resolve.rs:290`(エイリアスの参照先がエイリアス。読み取りで保証され到達しない)、`resolve.rs:477-483`(`Outcome::Internal`。フレームの状態の守り。到達しない)。表示は「`x.yaml:3: 文法の誤りです: 文法の誤りです`」、英語のロケールでは「`x.yaml:3: syntax error: 文法の誤りです`」(理由は日本語のまま。rust `09-converter.md` の「`<reason>` は日本語のまま」。定義の誤りの `<原因>` が日本語のままなのと同じ規則で、`08-output-and-i18n.md:128` の定義の誤りと同じ扱い)。
- 目標(D5。決定(ユーザー、2026-11-23)): `reason::TOML_SYNTAX = "TOML として読めません"`(`toml_read.rs:67`)、`reason::YAML_SYNTAX = "YAML として読めません"`(`yaml_read.rs:216`)。`resolve.rs` の 2 か所は形式が決まらない内部の守りなので、`reason::BROKEN_STRUCTURE = "入力の構造を解釈できません"`(通常は使われない。D4。決定(ユーザー、2026-11-23))。`reason::SYNTAX` を消す。種類の表示(`文法の誤りです`、英語 `syntax error`)は変えない。
- 表示: 日本語「`x.yaml:3: 文法の誤りです: YAML として読めません`」、英語「`conductor: error: x.yaml:3: syntax error: YAML として読めません`」(理由は日本語のまま。英語の理由の文は作らない。今の規則のまま)。入力の値は含めない。
- 期待が変わる既存のテスト: ★ T24(`crates/converter/tests/convert_ac.rs:70` の `"文法の誤りです"`。`e.to_string()` は種類の文「文法の誤りです」も含むので、今の検査は理由が変わっても通るが、理由の文を確かめる意図に合わせて `"YAML として読めません"` にする)。検索: `grep -rn "文法の誤りです\|reason::SYNTAX" crates`(core の `error.rs:38` は定義の誤りの種類の文で無関係。cli のテストは `flow_aux.rs:329` が `syntax error` だけを確かめ、偽物の変換結果を使う)。
- 仕様書(「v0.9.6 までは、文法の誤りの理由は種類の文と同じ `文法の誤りです` だった」と注記):
  - rust `09-converter.md:72`、`:78`、`:88`、`:94`、`:330`、`:342`、`:432-433`、`10-errors-and-exit-codes.md:192`、`12-rebuild-guide.md:303`。`08-output-and-i18n.md:256` は種類の表で変えない(理由が日本語のままであることを、`:247` の節に 1 文足す)。
  - language-agnostic `09-converter.md:86`、`:93`、`:121`、`:128`、`:136`、`:139`、`:142-143`、`:428`、`:440`、`:538-539`、`10-errors-and-exit-codes.md:204`、`11-quality-and-build.md:584`(AC-210 の期待の行)、`12-rebuild-guide.md:285-286`。`09:81`(core の定義の理由との比較の表)は converter の列を新しい文に。`08:269` は種類の表で変えない。

### 3b. `.env` の読み込みの順序(Q5: A)

- 今: `crates/runtime/src/envfile.rs:85-113`(`read_one`): パスの `metadata`(`:96`)→ 通常でない → `Unreadable`(`:97-99`)→ **パスのメタデータの大きさ**が上限超 → `TooLarge`(`:100-102`)→ `File::open`(`:103`)→ 上限 + 1 まで読む(`:104-110`)→ UTF-8(`:111`)。読む権限のない 1 MiB 超のファイルは `TooLarge`。
- 目標(定義の `read_definition`(`crates/core/src/definition/intake.rs:71-129`)と同じ形 B): (1) パスの `metadata`: `NotFound` → `FileMissing`、他 → `Unreadable`。通常のファイルでない → `Unreadable`(開かない。大きさは見ない)。(2) `File::open`: `NotFound`(1 のあとに消えた)→ `FileMissing`、他 → `Unreadable`。(3) 開いたハンドルの `metadata()`: 取れない → `Unreadable`、通常でない → `Unreadable`、大きさが上限超 → `TooLarge`。(4) 上限 + 1 まで読む(失敗 → `Unreadable`、超えれば `TooLarge`)。(5) UTF-8 でない → `NotUtf8`。
- 変わること: 読む権限のない 1 MiB 超の `.env` → `Unreadable`(今は `TooLarge`)。変わらないこと: 無い(`FileMissing`)、ディレクトリ・名前付きパイプ(`Unreadable`。開かないので FIFO で止まらない)、読める 1 MiB 超(`TooLarge`)、1 MiB 超で UTF-8 でない(`TooLarge`)、UTF-8 でない(`NotUtf8`)、読む権限のない小さなファイル(`Unreadable`)。誤りの理由は種類ごとの固定の文のまま(値を含めない)。
- 守ること: runtime の製品のコードには `.unwrap()`・`.expect(`・`panic!(`・`unreachable!(`・`todo!(`・`unimplemented!(` を書かない(`crates/runtime/tests/nfr_source.rs:163-180` の `製品のコードに_パニックしうる書き方がない`)。形 B の各段の失敗は `map_err` と `?` で `EnvFileErrorKind` にし、パニックしうる書き方を使わない。
- 期待が変わる既存のテスト: なし(読む権限のない大きな `.env` を確かめるテストが無い。`grep -rn "TooLarge" crates/runtime/tests crates/cli/tests` で確かめた: `env_loader.rs` の「ちょうど 1 MiB」は読めるファイル)。
- 仕様書: rust `03-definition-format.md:345-352`(順の表を形 B に)と、表の後に「読む権限のない(mode 000 の)`.env` は、大きさにかかわらず `unreadable`。v0.9.6 までは、パスのメタデータで大きさを先に見たため、1 MiB を超える読む権限のない `.env` は `tooLarge` だった」「通常のファイルでないもの(名前付きパイプを含む)は、開かずに `unreadable`」。rust `02-architecture.md:319`(`envfile.rs` の行)、`11-quality-and-build.md:217`(`env_loader.rs` の説明)。language-agnostic `03-definition-format.md:512-520`(同じ表と注記)、`11-quality-and-build.md:914`(AC-464 に「読む権限のない 1 MiB 超 → 64(`unreadable`)」を足す)。
- 新しいテスト: N11、N12。

### 3c. 版を 0.9.7 に(最後のコミット)

- `git show --stat ee9f409`(0.9.6 への版の更新)と同じ範囲(22 ファイル): `Cargo.lock`(4 クレートの版)、`crates/{cli,converter,core,runtime}/Cargo.toml`、★ T25 `crates/cli/tests/binary_basic.rs:246`(`"conductor 0.9.6\n"` → `"conductor 0.9.7\n"`)、仕様書の `0.9.6` の表記(16 ファイル: rust `00-README.md:3`、`:10`、`03:1`、`04:1`、`:250`、`:256`、`09:1`、`11:23-28`、`:513`、`12:13`、`:272`。language-agnostic `00:3`、`:10`、`01:3`、`02:3`、`04:288`、`:292`、`05:3`、`06:3`、`07:3`、`08:111`、`11:3`、`:760`、`12:3`)。
- 段 1〜3 で足す「v0.9.6 までは…」の注記は昔の版を指すので上げない。今ある「v0.9.5 までは…」も変えない。
- タグとリリースはしない [memory:M6]。

---

## 8. 使った検索のコマンド(テストの変更の一覧の網羅のため)

リポジトリのルートで実行した。結果は §0〜§3 と `functional-spec.md` §4 に反映した。

```sh
# 基準の件数(クレートごとの合計)
cargo test --locked --workspace 2>&1 | awk '/Running |Doc-tests/{n=$0} /^test result/{print n " | " $0}'
# 1a・1b・1e: 計画の型
grep -rn "ResumeInfo\|ResumeKind\|\.resume\.\|NextStepNote\|next_step_note\|note\.\(next_job_id\|skipped_job_ids\|completed\|end_reached\|stopped\)\|SimulatedResult\|\]\.result\|entry\.result\|\.result\.expect\|\.result\.is_none\|PlanDisposition\|D::\|e\.disposition\|fn dispositions" crates
# 1c: 図の節
grep -rn "GraphNode\|is_end\b\|\.is_end\|node\.job_id\|n\.job_id" crates
# 1d: リトライの設定
grep -rn "multiplier\|max_interval_secs\|RetryPolicy {" crates docs/spec/rust docs/spec/language-agnostic
# 1f: リトライの判断
grep -rn "RetryDecision" crates docs/spec/rust docs/spec/language-agnostic
# 1g: 読まれないフィールドと組み立ての関数
grep -rnoE "[a-z_\]\)]+\.job_id\b" crates | awk -F: '{print $3}' | sort | uniq -c
grep -rnE "(execution|exec|e|outcome|o|next|step|n)\.job_id\b" crates
grep -rn "\.command\b" crates | grep -v "job\.command\|j\.command\|def\.job\|definition\.job\|\.command_words"
grep -rn "JobExecution::\(new\|exited\|not_started\|start_failed\)" crates
grep -rn "JobExecution {\|JobOutcome {\|NextStep {" crates
grep -rn "run_job(" crates
grep -n "exited(\|exec(" crates/core/tests/flow_sequences.rs crates/core/tests/retry_flow.rs crates/core/src/flow/next_step.rs crates/core/src/flow/record.rs crates/core/src/flow/classify.rs
grep -n "outcome(\|step(" crates/core/src/flow/record.rs
# 1h: `.env` の誤りの理由
grep -rn "\.reason\b\|reason:" crates | grep -iv "skip_reason\|reject\|SkipCause"
grep -rn "EnvFileError {\|EnvFileError::new" crates
# 1i: cli の準備
grep -rn "prepare_run\|prepare_pipeline\|prepare_checked\|check_definition\|CheckedDefinition\|Checked\b\|enter_flow\|record_skip\|checked\.\(definition_path\|path\|definition\|commands\)" crates/cli/src
# 1j: messages の関数と呼ぶ所
grep -n "fn \|pub const\|match lang\|-> " crates/cli/src/report/messages.rs
grep -rn "definition_kind\|env_file_kind\|env_file_reason\|conversion_kind\|state_kind\|plan_disposition\|class_word\|messages::" crates/cli/src crates/cli/tests
# 1k: 外から使われない pub(core)
for p in "definition::depth" "definition::parse" "definition::version" "definition::walk" find_too_deep file_label resolve_transition TransitionResolution "\.add(" "definition::intake" quote_key Locate MAX_NESTING_DEPTH VERSION_KEY SUPPORTED_FORMAT_VERSION "conductor_core::error::" "conductor_core::types::"; do grep -rn "$p" crates --include='*.rs' | grep -v "^crates/core/src"; done
# 1k: cli(テストと補助バイナリから)
for n in "report::messages" "messages::" "report::watch" status_word state_symbol glyph_char MAX_DISPLAY_CHARS DEFAULT_INIT_PIPELINE_NAME DEFAULT_INIT_PATH "platform::" language_resolved color_resolved; do grep -rln "$n" crates/cli/tests crates/cli/src/bin crates/cli/src/main.rs; done
# 1k: runtime
grep -rn "conductor_runtime::\(error\|types\)::" crates
# 段 2: 計算量の場所
grep -n "clone()\|fn find_mut\|find_mut(" crates/core/src/flow/record.rs crates/core/src/plan/simulate.rs
grep -n "find(\|job_index\|position(\|remove(" crates/core/src/graph/overlay.rs crates/core/src/graph/edges.rs crates/core/src/flow/next_step.rs crates/core/src/envfile/parse.rs
# 段 3: 文法の誤りの理由と、版
grep -rn "文法の誤りです\|reason::SYNTAX\|SYNTAX\b" crates docs/spec/rust docs/spec/language-agnostic README.md
grep -rn "syntax error\|文法" crates README.md
grep -rn "TooLarge\|tooLarge" crates/runtime/tests crates/cli/tests
git show --stat ee9f409
grep -rno "0\.9\.6" docs/spec/rust docs/spec/language-agnostic crates README.md
# 仕様書の該当の行
grep -n "ResumeInfo\|ResumeKind\|NextStepNote\|SimulatedResult\|PlanEntry\|PlanDisposition\|GraphNode\|is_end\|RetryDecision\|JobOutcome\|NextStep\b\|JobExecution\|EnvFileError\b\|prepare_run\|prepare_pipeline\|prepare_checked\|CheckedDefinition\|record_skip\|resolve_transition\|TransitionResolution\|state_kind\|definition_kind\|conversion_kind\|env_file_kind\|env_file_reason\|class_word\|plan_disposition" docs/spec/rust/*.md docs/spec/language-agnostic/*.md
```
