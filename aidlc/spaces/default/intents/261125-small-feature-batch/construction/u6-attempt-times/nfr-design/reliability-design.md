# 信頼性の設計: u6-attempt-times

出典: 信頼性の要件(`reliability-requirements.md` の NFR1.1、NFR1.2、NFR1.6、NFR1.7、NFR1.8、NFR2.1〜NFR2.3、NFR3.1、NFR3.2)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 2〜5)、契約(`contract-summary.md` の C4)。

## 1. 試行の関数(core、純粋。`crates/core/src/flow/attempt.rs`)

| 関数 | 動き |
|---|---|
| `start_attempt(state, job_id, at) -> PipelineState` | そのジョブの最後の試行に `ended_at` が無ければ、自身の `started_at` で閉じる(BR1.4、NFR1.6)。その後ろに `Attempt { started_at: at, ended_at: None }` を足す |
| `end_attempt(state, job_id, at) -> PipelineState` | 最後の試行の `ended_at` を `at` にする |
| `cancel_attempt(state, job_id) -> PipelineState` | 最後の試行を取り除く(起動しなかった試行。下の 2 の表) |

`apply_outcome` と `apply_skip` は `attempts` を変えない(今のコードは `JobState` の項目を選んで書き換えるため、そのまま保たれる)。

## 2. 流れの形(cli の `step.rs`。機能の設計のレビューの指摘)

`run_job_with_retry` は、今の `(JobExecution, LogOutcome)` を返す形に、記録中のステートを `&mut PipelineState` で受け取り、書き込みの失敗を `Err(65)` で返す形にする。

```text
run_step:
  base(無ければ初期ステート。ここでは書かない)
  run_job_with_retry(ctx, session, job, line, &mut state)?
     試行ごと:
       started_at = now()
       state = start_attempt(state, job, started_at) → Fast で書く(初めの 1 回が初期化を兼ねる)
       run_job → (execution, log)
       結果ごとに下の表のとおり
  apply_outcome(state, ...) → Durable で書く(今と同じ)
```

| 試行の結果 | 試行の扱い | 理由 |
|---|---|---|
| 終わった(終了コード、シグナル、中断を受けて終わった) | `end_attempt(now())` | BR1.2 |
| 起動の失敗(コマンドが無いなど) | `end_attempt(now())` | 状態は `error` になるため、試行が要る(BR3.1) |
| ログを作れずに起動しなかった(U5 の `log_not_created`) | `end_attempt(now())` | 状態は `error` になるため、試行が要る。非機能の要件の NFR1.7 の「足さない」は BR3.1 と食い違うため、この設計で「足して閉じる」に改める |
| 起動の前に中断の合図が立っていた(`NotStarted`) | `cancel_attempt` | 起動していない(BR1.3)。状態は変わらない |
| リトライの待ちの中断 | 新しい試行を足さない | BR1.3 |

- リトライするときは、`end_attempt` の後に Fast で書いてから知らせて待つ。
- 書き込みの失敗は、今の初期化の書き込みの失敗と同じ表示と 65。ジョブは起動しない。

## 3. 読み込みの検査(runtime の `state/reader.rs`)

- `walk_job` に `attempt` のキーを足す(`JOB_KEYS` に `attempt`)。値は表の配列。各表のキーは `started_at`(必須)と `ended_at`(任意)。
- 新しい理由の文は 3 つ(`NO_ATTEMPT`、`BAD_TIME`、`UNFINISHED_ATTEMPT`)。知らないキー・欠け・型の誤りは今の `UNKNOWN_KEY`・`MISSING_KEY`・`WRONG_TYPE` を使う(非機能の要件のレビューの指摘。理由は新しく 3 つ、使うのは 6 つ)。
- 書き出し(`writer.rs` の `job_table`)は、試行があるときだけ `attempt` を書く(試行が無ければ今と同じ)。

## 4. 所要時間の出し方(NFR1.8)

| ジョブ | 所要時間 |
|---|---|
| 試行が無い | 空 |
| `success`・`warning`・`error` | 最初の `started_at` から最後の `ended_at` まで |
| `pending` で最後の試行に `ended_at` が無い | 最初の `started_at` から今まで(動いている最中とみなす。Q1) |
| `pending` で最後の試行が終わっている(中断の後など)、`skipped` | 空 |

## 5. テストの配置と、今のテストの直し

| テストの場所 | 確かめること |
|---|---|
| `crates/core/tests/` の新しいファイル(試行と所要時間) | `start_attempt`(閉じ忘れの試行を閉じる)、`end_attempt`、`cancel_attempt`、上の 4 の表、`format_duration` |
| `crates/runtime/tests/state_read.rs`、`state_write.rs` | 書いて読み戻すと等しい、3 つの理由、知らないキー、終わりの無い最後の試行は読める |
| `crates/runtime/tests/nfr_content.rs` | 時刻の 2 つのキーだけを許す |
| `crates/cli/tests/flow_*.rs` | 書く順と回数(偽の書き込み)、2 の表の場面ごと、65 と案内を 4 つのコマンドで |
| `crates/cli/tests/binary_*.rs` | ステートに時刻、古い形のステートで 65、`watch` の列 |
| `src/tests/` の `watch` の表 | 4 の表、英日 |
| `unix_demo.rs`、`windows_demo.rs` | 所要時間は形だけ |

- **今のテストの直し(非機能の要件のレビューの指摘)**: `success`・`warning`・`error` のジョブを持つ固定のステート(`state_read.rs` の `VALID` など)には、`[[job.attempt]]` を足す。直すのはテストのデータだけで、確かめる内容は変えない。NFR1.1 の例外に「固定のステートに試行を足す直し」を加える。
- 本番にテスト専用の項目を足さない。時刻は偽の `Runtime::now()` で固定する。
