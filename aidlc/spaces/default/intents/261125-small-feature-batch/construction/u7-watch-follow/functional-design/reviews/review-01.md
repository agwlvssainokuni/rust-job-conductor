## Review

**Verdict:** NOT-READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-25T23:10:48Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Critical | aidlc/spaces/default/intents/261125-small-feature-batch/construction/u7-watch-follow/functional-design/functional-spec.md > 3. 動きの判定(`pipeline_activity`)の順 3、rules.md > BR3.1、entities.md > PipelineActivity の `Stopped` | `[job.retry]` の待ち時間のあいだ、`Stopped` と誤判定され、`watch --follow` が実行の途中で終わる。`run_job_with_retry`(`crates/cli/src/step.rs`)は、試行を閉じた状態(`ended_at` あり)を書いてから `wait` する。このとき `status` はまだ `apply_outcome` 前なので、初回なら `pending`、再開なら前回の `error` のままである。`resume_job_id` は、直前のジョブの `apply_outcome` が書いた「次に実行するジョブ」で、2 つ目以降のジョブでは今リトライ中のジョブ自身を指す(`record.rs` の `state.resume_job_id = next.resume_job_id`)。したがって、リトライ待ちの状態は「終わりの時刻の無い試行が無い」かつ「`resume_job_id` のジョブが試行を持つ」に当てはまり(再開のときは `error` のジョブもある)、順 3 の `Stopped` になる。表示は終了コード 0 で打ち切られ、FR7.3「パイプラインが止まったときに終わる」を、リトライを使うあらゆるパイプラインで破る。 | `Stopped` の条件を、リトライの待ちと中断・エラー停止の後とを区別できるように設計し直す。例: ステートに「待ち中」を見分ける材料が無い現状では、`resume_job_id` のジョブが試行を持つだけでは `Stopped` にせず、連続して同じ状態が一定回数(または一定時間)続いたときだけ止まったとみなす。あるいは、リトライを使うジョブでは待ち時間の上限を考慮する、`Stopped` を `error` が確定したとき(最後の試行が閉じており、かつ定義のリトライを使い切った)に限るなど。選んだ規則と、`[job.retry]` の待ち中に続けるという受け入れ条件を `rules.md` の BR3.1 と `functional-spec.md` の 3 に書く。 | New |
| R-02 | Minor | aidlc/spaces/default/intents/261125-small-feature-batch/construction/u7-watch-follow/functional-design/functional-spec.md > 2. 流れ(合図の準備)、entities.md > FollowEnd | 「合図の準備(今の run・pipeline と同じ)」とあるが、準備に失敗したときの扱いが無い。`run` は内部の誤り(`EXIT_INTERNAL` 70)で終わるが、`FollowEnd` と BR3.3 の終了コードの一覧にこの終わり方が載っていない。 | 合図の準備に失敗したときの終了コード(今の `run` と同じ 70 か)と表示を `FollowEnd` と BR3.3 に足す。 | New |
| R-03 | Minor | aidlc/spaces/default/intents/261125-small-feature-batch/construction/u7-watch-follow/functional-design/functional-spec.md > 2. 流れ(描く)、rules.md > BR2.2 | 画面を消すのが 1 回目の描画から(spec の「描く」)か 2 回目から(BR2.2)かが食い違う。 | BR2.2 と spec をそろえる。端末では 1 回目も消すのか、追記の区切りの空行が 2 回目からだけなのかを書く。 | New |
| R-04 | Minor | aidlc/spaces/default/intents/261125-small-feature-batch/construction/u7-watch-follow/functional-design/functional-design-questions.md > Q1 の A | 回答済みの Q1 A が使い方の誤りを終了コード 2 と書くが、このプロジェクトの使い方の誤りは 68(`EXIT_USAGE`)で、設計は 68 を使う。質問の文と設計が食い違ったまま残る。 | 質問の文を 68 に直す(または、設計が 68 を使うと注記する)。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| (この段階で指定された検証ツールは無い。`step.rs`、`record.rs`、`types.rs`、`deps.rs` を読んで突き合わせた) | 上の Critical を確認 | リトライ待ちの状態が `Stopped` の条件に当てはまることを、コードの流れで確かめた |

### Summary

継続表示の骨組み(引数、描き直し、終了コード、ステートが無いとき・読めないとき)は、既存のコードと契約に合っている。ただし、`Stopped` の判定がリトライ待ちの状態と区別できず、リトライを使うパイプラインで表示が途中で終わるため、このままでは実装に進めない。
