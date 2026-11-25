## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-25T23:14:26Z
**Iteration:** 2
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Critical | aidlc/spaces/default/intents/261125-small-feature-batch/construction/u7-watch-follow/functional-design/functional-spec.md > 3. 動きの判定(`pipeline_activity`)の順 3、rules.md > BR3.1、entities.md > PipelineActivity の `Stopped` | `[job.retry]` の待ち時間のあいだ、`Stopped` と誤判定され、`watch --follow` が実行の途中で終わる。`run_job_with_retry`(`crates/cli/src/step.rs`)は、試行を閉じた状態(`ended_at` あり)を書いてから `wait` する。このとき `status` はまだ `apply_outcome` 前なので、初回なら `pending`、再開なら前回の `error` のままである。`resume_job_id` は、直前のジョブの `apply_outcome` が書いた「次に実行するジョブ」で、2 つ目以降のジョブでは今リトライ中のジョブ自身を指す(`record.rs` の `state.resume_job_id = next.resume_job_id`)。したがって、リトライ待ちの状態は「終わりの時刻の無い試行が無い」かつ「`resume_job_id` のジョブが試行を持つ」に当てはまり(再開のときは `error` のジョブもある)、順 3 の `Stopped` になる。表示は終了コード 0 で打ち切られ、FR7.3「パイプラインが止まったときに終わる」を、リトライを使うあらゆるパイプラインで破る。 | `Stopped` の条件を、リトライの待ちと中断・エラー停止の後とを区別できるように設計し直す。例: ステートに「待ち中」を見分ける材料が無い現状では、`resume_job_id` のジョブが試行を持つだけでは `Stopped` にせず、連続して同じ状態が一定回数(または一定時間)続いたときだけ止まったとみなす。あるいは、リトライを使うジョブでは待ち時間の上限を考慮する、`Stopped` を `error` が確定したとき(最後の試行が閉じており、かつ定義のリトライを使い切った)に限るなど。選んだ規則と、`[job.retry]` の待ち中に続けるという受け入れ条件を `rules.md` の BR3.1 と `functional-spec.md` の 3 に書く。 | Resolved (reviewer) |
| R-02 | Minor | aidlc/spaces/default/intents/261125-small-feature-batch/construction/u7-watch-follow/functional-design/functional-spec.md > 2. 流れ(合図の準備)、entities.md > FollowEnd | 「合図の準備(今の run・pipeline と同じ)」とあるが、準備に失敗したときの扱いが無い。`run` は内部の誤り(`EXIT_INTERNAL` 70)で終わるが、`FollowEnd` と BR3.3 の終了コードの一覧にこの終わり方が載っていない。 | 合図の準備に失敗したときの終了コード(今の `run` と同じ 70 か)と表示を `FollowEnd` と BR3.3 に足す。 | Resolved (reviewer) |
| R-03 | Minor | aidlc/spaces/default/intents/261125-small-feature-batch/construction/u7-watch-follow/functional-design/functional-spec.md > 2. 流れ(描く)、rules.md > BR2.2 | 画面を消すのが 1 回目の描画から(spec の「描く」)か 2 回目から(BR2.2)かが食い違う。 | BR2.2 と spec をそろえる。端末では 1 回目も消すのか、追記の区切りの空行が 2 回目からだけなのかを書く。 | Resolved (reviewer) |
| R-04 | Minor | aidlc/spaces/default/intents/261125-small-feature-batch/construction/u7-watch-follow/functional-design/functional-design-questions.md > Q1 の A | 回答済みの Q1 A が使い方の誤りを終了コード 2 と書くが、このプロジェクトの使い方の誤りは 68(`EXIT_USAGE`)で、設計は 68 を使う。質問の文と設計が食い違ったまま残る。 | 質問の文を 68 に直す(または、設計が 68 を使うと注記する)。 | Resolved (reviewer) |
| R-05 | Minor | aidlc/spaces/default/intents/261125-small-feature-batch/construction/u7-watch-follow/functional-design/functional-spec.md > 3 動きの判定(`pipeline_activity`) | `error` になったジョブは結果が出た後も `error` のままなので、`error` の後に続くジョブがある定義では、次のジョブが始まるまでの短い間が `Stopped` と判定され、継続表示が早く終わりうる。Q2 の A の範囲内の制約だが、制約として書かれていない。 | この制約を spec と README の継続表示の説明に書く(`error` の後に続くジョブがある定義では、短い間に表示が終わることがある)。 | New |

> R-01 Reviewer note: functional-spec.md §3.1 と BR3.4 で、リトライの待ちの間は最後の試行を開いたままにし、終わりの時刻をメモリに持つ形に直した。待ちの途中の中断では end_attempt をしてから抜ける。これで待ちが Stopped に見えない。

> R-02 Reviewer note: 合図の準備の失敗は 70(spec §2、BR3.3、entities.md)に揃った。

> R-03 Reviewer note: 画面を消す描画は 1 回目から毎回(BR2.2、spec §2)に揃った。

> R-04 Reviewer note: Q1 の「2」と 68 の食い違いは spec §4 に注記された。

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| (なし) | - | この段階に検証ツールの指定は無い。upstream の参照(FR7.1〜FR7.5、FR8.1、FR8.2、C5)と core の `decide.rs` の `error`/`resume_job_id` の扱いを突き合わせて確認した。 |

### Summary

前回の Critical と 3 つの Minor はすべて直った。新しい Minor は 1 つで、`error` ジョブの後に続くジョブがある場合の `Stopped` の誤判定を、制約として文書化すれば足りる。実装を阻むものは無い。
