## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-18T22:27:29Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | aidlc/spaces/default/intents/261118-job-retry/construction/u2-retry-execution/functional-design/functional-spec.md > 待ち時間の計算(BR2.3) の表 | 表は「1 回目は interval」とだけ書くが、rules.md の BR2.3 は「1 回目は min(interval, max_interval)」である。コード(crates/core/src/flow/retry.rs の next_wait_secs)は BR2.3 の min を実装しており、interval が max_interval より大きい設定では表と食い違う。 | 表の 1 回目を「interval と max_interval の小さい方」に直す。または、interval が max_interval 以下であることを u1 の設定の検証が保証しているなら、その旨を書く。 | New |
| R-02 | Minor | aidlc/spaces/default/intents/261118-job-retry/construction/u2-retry-execution/functional-design/functional-spec.md > 全体の流れ、手順 7 と 2 | 待機が Elapsed で終わってから次の実行を始めるまでの間にシグナルが立つ場合の扱いが書かれていない。BR2.5 は待機の開始時と待機中だけを扱い、再実行の前に合図を確かめる前提は BR2.6 の「既存の run_job をそのまま使う」に暗黙に頼っている。 | 再実行の直前に、既存の起動前の中断の確認が働くことを 1 行で明記する。これで、知らせと待機のあいだ、および待機と再実行のあいだのどちらでシグナルを受けても、取りこぼしがないことが読み取れる。 | New |
| R-03 | Minor | aidlc/spaces/default/intents/261118-job-retry/construction/u2-retry-execution/functional-design/entities.md > RetryPolicy と RetryNotice | RetryNotice.last_exit_code は必須の整数だが、対象の結果は必ず終了コードを持つ(BR2.1 の 6)、という前提が entities.md に書かれていない。コードは unwrap_or_default で埋めている。 | last_exit_code の notes に、Retry と決まった結果は必ず終了コードを持つため常に値がある、と書く。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| コードとの突き合わせ(crates/core/src/flow/retry.rs、crates/cli/src/retry.rs) | 判断の 6 条件、BR2.2 の上限、BR2.3 の飽和計算、知らせ、待機、Interrupted の実行結果(started=false、interrupted=true、signal_number つき)は、設計の記述と一致した。食い違いは上記 Minor の 1 件目だけ。 | 設計は実装可能で、実装と矛盾しない。 |

### Summary

要件(FR1.2、FR2、FR3、FR4、FR5、FR6、NFR3、NFR7)に対する規則 BR2.1〜BR2.16 は一貫していて、待機中のシグナル、ステートの扱い、`run` と `pipeline` の共通化、127 の扱いも、既存の流れに無理なくつながる。見つかったのは文言の食い違いと明記の不足だけで、Critical と Major はなく、READY とする。
