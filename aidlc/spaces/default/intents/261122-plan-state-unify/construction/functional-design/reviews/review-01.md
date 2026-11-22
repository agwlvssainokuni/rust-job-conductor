## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-22
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261122-plan-state-unify/construction/functional-design/functional-spec.md > §3 変更 3 と §5 テストの方針 | 1,000 ジョブの計画を 1 秒以内に作る性能の要件(`plan_perf.rs`、`decide.rs` の `status_at`)への影響が検討されていない。`apply_outcome` と `apply_skip` は呼ぶたびにステート全体を複製する。さらに `apply_skip` は、最初の pending を探すのに、ジョブごとに状態を線形に引く(`status_of` の `find`)ので、1 回が O(n^2) になる。スキップが多い計画(終点の後に大量のスキップ)では、全体が O(n^3) に近づく。既存の `plan_perf.rs` が測るのは、全ジョブが実行される 1 つの形だけで、スキップの多い形は測っていない。本番では 1 回の呼び出しだったので問題にならなかったが、模擬では全ジョブ分を回す | 実装の前に、終点の後で約 1,000 ジョブをスキップする形の計画の時間を測る(使い捨てでよい)。1 秒に収まらなければ、`apply_skip` の探索を位置の添字でたどる形に直すか、模擬のスキップだけ別の手当てを設計に書く。性能の確認を §5 の合格の条件に加える | New |
| R-02 | Minor | functional-spec.md > §2 比べる確認(`plan-compare.rs.txt` の `new_plan`) | 比べた「新しい模擬」は、テストの中に書いた別の再実装で、実際にリファクタリングした `plan_pipeline` ではない。比べ終わって削除されたので、実装後の本物には同じ比較を再実行できない。実装が `simulate_job` の戻り値の変更などで再実装とずれても、既存テストだけが頼りになる | 実装の間は、今の `plan_pipeline` の写しを一時のオラクルとして残し、実装後の本物と同じ 90 万例を 1 回比べてから消す。または、小さく絞った版を `#[ignore]` のテストとして残す | New |
| R-03 | Minor | functional-spec.md > §2 のステートの作り方(`real_step`)と結論 | 生成したステートは、本番の更新の関数だけで作っているので、現実の形は広く網羅するが、手で編集したステートは含まない。例として、`skipped(byEnd)` の記録があるのに `end_reached = false` のステート(`check_consistency` はこの組み合わせを拒否しない)では、今の模擬は終わり方が `Completed`、新しい模擬は `apply_skip(AfterEnd)` が `end_reached` を立てるので `EndReached` になる。「出力は変わらない」は、本番で作れるステートに限った主張である。run_id、リトライ、`on = "error"` の遷移、`success_exit_codes` も、定義の生成に入っていない(出力には効かないが、書かれていない) | §2 に、「比べた範囲は本番で作れるステートに限る。手編集の `byEnd` と `end_reached=false` の組み合わせは、新しい方が `EndReached` になる(より整合した結果)」と明記する。受け入れるかどうかを人が決められるようにする | New |
| R-04 | Minor | functional-spec.md > §2 規則 4 の理由 | 理由の説明が不完全。実際は次の 2 点による。(a) 新しい模擬の `apply_skip` は `resume_job_id` を「最初の pending」に書き換えるが、古い模擬は古い再開位置のまま、という違いがある。(b) ただし記述順のループで後のジョブに着く時点では、それ以前のジョブはすべて実行済み、スキップ済み、または拒否で返却済みなので、後のジョブは規則 6 で同じ判断になり、規則 4 は結果を変えない。現在の文は「前のジョブが pending のまま残るのは終点の後のスキップだけ」と書くが、旧実装ではその場合も規則 2 が先に当たる、という点が実質の根拠である。論理は成り立つが、(a) を書いていない | 理由を (a)(b) の形に書き直す。比べる確認が、これを経験的に裏づけていることも添える | New |
| R-05 | Minor | functional-spec.md > §4 仕様書(WF2)の一覧 | 書き直しの一覧に抜けがある。仕様書では 12 の `state/record.rs` の行(58 行目。runtime の「作るもの」からコアの行へ移す)と `simulate_job` の戻り値の変更(05 §16.2 の記述、12 の 126 行目)が明記されていない。コードのコメントでは `crates/runtime/src/lib.rs` の表(60 行目)、`crates/runtime/src/state/mod.rs`(`pub(crate) mod record` の削除と 16 行目の説明)、`crates/core/src/plan/simulate.rs` の冒頭のコメント(「core は呼べない」「apply_result」)、`record.rs` の冒頭のコメントの所在が古くなる。core の `lib.rs` と `flow/mod.rs` の再エクスポート(`pub use flow::{...}`)の追加も、一覧に無い | §3 と §4 の一覧に、これらを加える。`docs/spec/language-agnostic` を変えない方針は、模擬の出力が変わらず、状態の扱いを書いた箇所が無い、という確認(私は全文の確認はしていない)を前提に受け入れられる | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| 名前の衝突の確認(core の `lib.rs`、`error::*`、`types::*`) | `initial_state`、`apply_*` の名前は core に無い | 衝突なし。`pub use conductor_core::{...}` の再エクスポートは、runtime の `types::*` と衝突しない |
| 依存の確認(`record.rs` の import) | `std::path::Path` と core の型だけ(`RunId` は `crate::run_id`) | 移動は成立する。入出力なし。core の `nfr_security.rs` は、`std::path` や `fs` の使用を検査していない |
| 利用箇所の確認 | cli(`pipeline.rs`、`step.rs`、`run.rs`)と runtime のテストは `conductor_runtime::` 経由 | 再エクスポートで変えずに動く。`state::record` は `pub(crate)` で外から名前を使われていない |
| 比べる確認(読み取りでの検査。実行はしていない) | ハーネスは、今の公開の `plan_pipeline` と、独立のループを比べる。0 件の主張はそのまま信頼できるが、上の限界がある | 確認した範囲で健全。ただし本物の新実装は未検証 |

### Summary

設計は成立している: `record.rs` は core の型だけに依存し、移動と再エクスポートに名前の衝突も依存の逆転も無く、比べる確認は本番で作れるステートの範囲では妥当である。主な懸念は、全ジョブ分の `apply_skip` と複製による 1,000 ジョブの計画の性能(Major 1 件)と、比べた対象が本物の実装でないこと、手編集のステートを含まないことの記録である。
