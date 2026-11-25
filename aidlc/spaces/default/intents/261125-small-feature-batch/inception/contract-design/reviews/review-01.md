## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-25T10:02:18Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261125-small-feature-batch/inception/contract-design/contract-summary.md > C4 ステートファイルの試行(`status = "running"` の例)と Open questions の C4 行 | C4 の例は、動いている最中のジョブを `status = "running"` と書き、「今の状態の値のまま(名前は既存のものを使う)」と注記している。しかし実コードの `JobStatus`(`crates/core/src/types.rs`)の値は `pending`、`success`、`warning`、`error`、`skipped` の 5 つだけで、`running` は無い。リーダーは知らない値を拒否する(`reader.rs` の `JobStatus::parse`)。この例のままだと、U6 の実装者が存在しない値を契約として受け取る。Open questions の「今の状態の値で足りるか」への答えも、実コードからは「足りない」(動いている最中も `pending` のまま)になる。 | 例を実在する値(`pending` と、`ended_at` の無い `[[job.attempt]]`)に直す。あるいは `running` を足すかどうかを決め、足すなら u6 の責務(`JobStatus`、`parse`、`as_str`、`watch` の表)に入れ、`format_version` 据え置きとの整合を書く。「動いている最中」と「中断で止まった」の見分け方は、U6 の機能の設計へ送る前に、この契約で決められる範囲(判断材料の候補)まで書く。 | New |
| R-02 | Minor | aidlc/spaces/default/intents/261125-small-feature-batch/inception/contract-design/contract-summary.md > C4 の「動いたジョブに `[[job.attempt]]` が 1 つも無いステートは 65」 | 「動いたジョブ」の判定の基準が書かれていない。ステートは `status` と `exit_code` を持つだけなので、`pending` かつ試行が無いジョブは正常(未着手)で、`success` などで試行が無いジョブだけが壊れていると読める。この読み方が書かれていないと、ステートの読み取りの検査(FR6.4)の実装が割れる。 | 「`pending` 以外のジョブ(`skipped` を除くか含むかを明記)に試行が 1 つも無ければ 65」のように、状態の値で判定を書く。`skipped` のジョブに試行が要るかも明記する。 | New |
| R-03 | Minor | aidlc/spaces/default/intents/261125-small-feature-batch/inception/contract-design/contract-summary.md > C2、C6 | ログのファイル名の `<開始の時刻>` は試行ごとの `started_at`(C2)と同じ値とされるが、その名前の `<試行の番号>` と、ジョブの ID の文字の制約(区切りの `.` や、パスに使えない文字)が契約に無い。ジョブの ID が `.` や `/` を含み得るなら、ファイル名の解釈と、`log_dir` の外への書き出しの余地が生じる。 | ジョブの ID の既存の文字の規則を確認して C6 に引用する。ファイル名に使えない文字が入り得るなら、その扱いを決める。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| (なし。ステージに検証ツールは無い) | n/a | 実コードの `crates/core/src/types.rs` の `JobStatus` と `crates/runtime/src/state/reader.rs` を直接確認した。 |

### Summary

契約の境界(u2 → u3、u5 → u6)と、利用者に見える面(定義ファイル、コマンドライン、ログ)は、依存が循環せず、足すだけの変更として一貫している。ただし C4 の例が実在しない `running` 状態を使っており、U6 の実装前に直すべきである。この 1 件は Major で、他は Minor のため、判定は READY とする。
