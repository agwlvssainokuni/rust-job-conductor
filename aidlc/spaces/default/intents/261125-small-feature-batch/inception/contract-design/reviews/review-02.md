## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-25T10:05:14Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261125-small-feature-batch/inception/contract-design/contract-summary.md > C4 ステートファイルの試行(`status = "running"` の例)と Open questions の C4 行 | C4 の例は、動いている最中のジョブを `status = "running"` と書き、「今の状態の値のまま(名前は既存のものを使う)」と注記している。しかし実コードの `JobStatus`(`crates/core/src/types.rs`)の値は `pending`、`success`、`warning`、`error`、`skipped` の 5 つだけで、`running` は無い。リーダーは知らない値を拒否する(`reader.rs` の `JobStatus::parse`)。この例のままだと、U6 の実装者が存在しない値を契約として受け取る。Open questions の「今の状態の値で足りるか」への答えも、実コードからは「足りない」(動いている最中も `pending` のまま)になる。 | 例を実在する値(`pending` と、`ended_at` の無い `[[job.attempt]]`)に直す。あるいは `running` を足すかどうかを決め、足すなら u6 の責務(`JobStatus`、`parse`、`as_str`、`watch` の表)に入れ、`format_version` 据え置きとの整合を書く。「動いている最中」と「中断で止まった」の見分け方は、U6 の機能の設計へ送る前に、この契約で決められる範囲(判断材料の候補)まで書く。 | Resolved (reviewer) |
| R-02 | Minor | aidlc/spaces/default/intents/261125-small-feature-batch/inception/contract-design/contract-summary.md > C4 の「動いたジョブに `[[job.attempt]]` が 1 つも無いステートは 65」 | 「動いたジョブ」の判定の基準が書かれていない。ステートは `status` と `exit_code` を持つだけなので、`pending` かつ試行が無いジョブは正常(未着手)で、`success` などで試行が無いジョブだけが壊れていると読める。この読み方が書かれていないと、ステートの読み取りの検査(FR6.4)の実装が割れる。 | 「`pending` 以外のジョブ(`skipped` を除くか含むかを明記)に試行が 1 つも無ければ 65」のように、状態の値で判定を書く。`skipped` のジョブに試行が要るかも明記する。 | Resolved (reviewer) |
| R-03 | Minor | aidlc/spaces/default/intents/261125-small-feature-batch/inception/contract-design/contract-summary.md > C2、C6 | ログのファイル名の `<開始の時刻>` は試行ごとの `started_at`(C2)と同じ値とされるが、その名前の `<試行の番号>` と、ジョブの ID の文字の制約(区切りの `.` や、パスに使えない文字)が契約に無い。ジョブの ID が `.` や `/` を含み得るなら、ファイル名の解釈と、`log_dir` の外への書き出しの余地が生じる。 | ジョブの ID の既存の文字の規則を確認して C6 に引用する。ファイル名に使えない文字が入り得るなら、その扱いを決める。 | Unresolved |
| R-04 | Minor | aidlc/spaces/default/intents/261125-small-feature-batch/inception/contract-design/contract-summary.md > C4 の 3 つ目の箇条 | 「ended_at が無くてよいのは各ジョブの最後の試行だけ」とあるが、`pending` 以外の状態(`success`/`warning`/`error`)の最後の試行に `ended_at` が無い場合の扱いが書かれていない。完了したジョブで `ended_at` が欠けたステートが 65 になるのか許容されるのかが、読み手によって分かれる。 | u6 の機能の設計で、完了状態の最後の試行に `ended_at` が必須かどうかを決め、必須なら 65 の対象に加える。契約には一文足すだけでよい。 | New |

> R-01 Reviewer note: C4 の例は実在する 5 つの状態の値だけを使い、`running` を足さないと明記した。`pending` で最後の試行に `ended_at` が無いものを「動いている最中か、中断で止まった」と読む規則を書き、見分け方は u6 の機能の設計に回して Open questions に載せた。契約として十分である。

> R-02 Reviewer note: 試行が無いジョブの扱いを状態の値で書いた。`success`/`warning`/`error` に試行が無ければ 65、`pending` と `skipped` は無くてよい。`skipped` の扱いも明記された。

> R-03 Reviewer note: 利用者の指示どおり機能の設計に回し、Open questions に載せた(C6、u5 が対象)。参考として、`crates/core/src/run_id.rs` の 74 行目付近の検査は英数字と `_` だけを通すので、ジョブの ID に `.` や `/` は入らない見込みである。u5 の設計でこの検査が実際にジョブの ID に使われているかを確かめて引用すれば、すぐ閉じられる。ブロックしない。

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| (なし) | 該当なし | この段階に検証ツールは無い。ジョブの ID の文字の規則は `crates/core/src/run_id.rs` を目で確認した。 |

### Summary

R-01 と R-02 は契約の本文で解消され、状態の値を増やさない方針は既存の 5 つの値と矛盾しない。残るのは R-03(機能の設計に回した)と、完了状態の `ended_at` の扱いの小さな補足だけで、どれも実装をブロックしない。
