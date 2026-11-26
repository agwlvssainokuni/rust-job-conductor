## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-26T23:29:23Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | aidlc/spaces/default/intents/261126-v0101-followups/construction/code-generation/code-summary.md > Step 5 の「2 倍ごとの伸び」 | F3 を直さない判定(BR3.3)は、250 → 500 の伸びが 1.52〜2.29 倍で「どの段でも 3 倍以上」を満たさない、という点から正しく導かれている。ただ 500 → 1,000 と 1,000 → 2,000 は 3.3〜3.7 倍で、基準の数字だけに頼ると境界に近い。判定の理由に、この 2 段の伸びがある事実と、1,000 ジョブが約 24ms で上限 150ms の約 6 分の 1 という事実の両方が書かれているので、結論は妥当。 | 対応は不要。Build and Test の `build-and-test-summary.md` に表を写すときも、この 2 段の 3 倍超を省かずに書く。 | Accepted risk |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| `cargo test --workspace --locked` | 失敗 0 件(出力に `FAILED` が 0 件) | R-06 のテスト 5 件を含め、全体が通る |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 警告なしで終了 | 通る |
| `cargo fmt --all -- --check` | 差分なし | 通る |
| `git diff 152c4a1..HEAD --stat -- crates/core/tests crates/core/src/types.rs crates/core/src/lib.rs crates/cli` | 出力なし | `plan_simulate_perf.rs`、`job_index`(`types.rs`)、公開の再公開(`lib.rs`)、cli は変わっていない |

### 確かめた点

- BR3.4(写しが無いこと): 規則 1〜7 は `decide.rs` の `decide_with` の 1 か所のままです。公開の `decide_before_run` と、模擬の `decide_at` は、どちらも `decide_with` を呼びます。`next_step` と `classify` も 1 か所のままです。記録は `apply_skip_in`・`apply_outcome_in` の 1 か所で、公開の `apply_skip`・`apply_outcome` が同じ関数を呼びます。`plan/simulate.rs` に規則の写しはありません。
- NFR1(公開の入口): `decide_from_state`、`classify_and_decide`、`apply_skip`、`apply_outcome` は、呼ぶたびに `SimulationMarks::new()` を作ります。前置きは 0 から、`error` の数は未計数から始まり、走査は変更前と同じです。`git diff` では cli も `lib.rs` も変わっていません。
- 目印の前提: `settled_prefix` と `pending_prefix` の前提は、済んだジョブが戻らず、どのジョブも pending に戻らないことです。`record.rs` で `status` を書く箇所は 125 行(分類から。`Success`・`Warning`・`Error` のみ)、146 行と 206 行(`Skipped`)の 3 つだけで、`Pending` を書く箇所はありません。`error` の数は、これらすべてで `note_status_change` を通ります。数える前の変更は、数えるときに現在のステートから数えるので漏れません。`record_at` と `position_of` は、手がかりが外れても `record_of`・`job_index` と同じ答えに戻ります(ジョブ ID の一意という前提は、定義の検証と読み込みが拒むと文書に書かれています)。
- R-06 のテスト: `walk` は公開と内部の 2 通りで同じ入力を進め、判断・次の動き・記録(`assert_eq!(internal, public)`)を各ジョブで比べます。比較の入力に、逆順の記録、記録の欠け、空のステート、再開位置と error の記録、途中のエラーの後の順序外、`end_reached`、手がかりの違いが入っています。Reject の後も止めずに進むので、目印がジョブをまたいで正しいことも見えます。比較は偽の実装でも通る形ではありません。
- BR3.5: `code-summary.md` の表で、llvm-cov の下の 1,000 ジョブは P1 3.53〜3.56ms・P2 2.75〜2.98ms(上限 75ms)です。2 倍ごとの伸びは最大 2.01 倍で、上限 2.5 倍以下です。判定は表から導けます。CI の `coverage` で通ることは、人の push の後に Build and Test で確かめる、と書かれていて、約束を越える主張はしていません。
- project の決まり: 新しい依存なし、`unsafe` なし、`cfg` は増えていません(`git diff` で `flow`・`plan` のみ)。テスト名とドキュメントコメントは日本語です。テスト専用の `pub` や引数は本番に無く、内部の入口は `pub(crate)` です。

### Summary

規則と記録の本体は 1 か所のままで、公開の入口は新しい目印で今までと同じ走査になります。目印の前提は、`status` を書く 3 箇所だけで保たれています。R-06 の一致のテストは、意味のある入力で公開と内部の両方を比べ、全体のテスト、clippy、fmt が通りました。BR3.5 の合格は測定の表から導けます。READY です。
