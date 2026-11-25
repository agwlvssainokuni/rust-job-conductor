## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-25T23:46:46Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | README.md > 「watch で表示し直しを続ける」の「エラーの後に続くジョブがあるときの注意」、crates/core/src/flow/activity.rs > `pipeline_activity` | `Stopped` は「動いている試行が無く `error` のジョブがある」だけで決まる。そのため `error` の後に遷移で続くパイプラインでは、後ろのジョブの境目(前のジョブの試行が閉じて次の試行が始まるまでの間)に描いた回が、止まったと判定して終わる。README の注意は再開のときの例に偏っていて、遷移で続く通常の実行でも同じことが起き得る点は書いていない。設計(rules.md の BR3.1)どおりの動きなので、ブロックはしない | 次の版の改良として、README の注意に「遷移で `error` の後にジョブが続く実行でも、境目に描くと終わることがある」を 1 文足すか、`Stopped` の判定を絞る(例: `error` のジョブの試行が最後に閉じた時刻より後に動きが無い、など)ことを検討する | New |
| R-02 | Minor | crates/cli/src/ctx.rs > `start_signals` | `fn` を `pub(crate)` に広げた。テスト専用ではなく、`watch --follow` が `run`・`pipeline` と同じ関数で合図を準備するための本番の利用で、「テストの都合で本番を曲げない」には当たらない。確認のみ | 対応は不要 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| `cargo test -p conductor-cli --test flow_aux --test flow_retry --locked` | PASS(58 件と 33 件) | 継続表示の流れと U6 の直しのテストが通る |
| `cargo test -p conductor-core --test pipeline_activity --locked` | PASS(10 件) | 動きの様子の判定が設計どおり |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS | 警告なし |
| 版の更新の比較(`1f88e59` と `e112426` の変更ファイル) | 同じ 22 ファイル | 版の更新に漏れなし。残る `0.9.9` は「v0.9.9 までの形」の注記だけ |

### Summary

継続表示の数え方と終わる条件(Completed、2 回続けて無いときの Removed、Stopped、中断、Unreadable の 3 回で 65、合図の失敗で 70)は rules.md の BR3.1〜BR3.3、BR4.1、BR4.2 と一致する。UNIX の端末だけが画面を消し、ほかは追記である。`watch` の `--follow` なしの経路は変わっていない。U6 の直しは、終わりの時刻をメモリに持って次の試行の始めに閉じ、待ちの途中の中断でも持っていた時刻で閉じて呼び出し側の Durable に入る。書く回数は Fast 2 回・Durable 1 回で、置き換えたテストは新しい動きを確かめていて弱めてはいない(削除は、その書き込み自体が無くなった 1 本だけ)。重大な欠陥は見つからず、READY とする。Minor は、`error` 後の遷移で続く実行での誤判定の説明が README に足りない点のみ。報告された手順のずれ(python3 での編集、一時ファイル、未 push のコミットの amend)は、最終の差分と結果に影響していない。
