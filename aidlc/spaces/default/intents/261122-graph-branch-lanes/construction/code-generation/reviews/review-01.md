## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-22
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | crates/core/tests/fixtures/graph_layout_oracle_branch.md and crates/core/tests/graph_layout_oracle.rs > ORACLE_BRANCH | 871 件の正解データは、規則から導いた図で、git の出力ではない。実装と一字一句一致するが、規則の解釈そのものの誤りは、このテストでは見つからない(同じ規則から実装と正解の両方を作ったため)。B1、B2 はユーザー承認済みで、code-summary.md にも明記されている。 | 対応は必須でない。可能なら、B3、B4 や三つ以上の線が重なる形を、ゲートで人が目視する。 | New |
| R-02 | Minor | aidlc/spaces/default/intents/261122-graph-branch-lanes/construction/code-generation/code-summary.md > 未確認の項目 | 行カバレッジ 80% の下限、CI(ubuntu、windows、MSRV 1.100)は未実行。macOS のローカルのテストだけが通っている。 | Build and Test で `cargo llvm-cov --fail-under-lines 80` を実行し、push 後に CI を確認する。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| cargo test --workspace | 1057 passed, 0 failed | code-summary.md の主張と一致する。 |
| cargo clippy --workspace --all-targets --locked -- -D warnings | 警告なし | 通る。 |
| cargo fmt --all -- --check | 差分なし | 通る。 |
| git diff 3ea7054 HEAD (Cargo.lock、各 Cargo.toml) | 差分 0 行 | 新しい依存はない。 |
| grep unsafe と cfg( (crates/core/src/graph) | 該当なし | unsafe も cfg も追加されていない。 |
| grep has_branch と古い B1 の `* \| c` | has_branch は無し。`* \| c` は分岐でない通常の例だけ(README 562、576 など) | 古い記述は残っていない。 |
| オラクルのテスト | ORACLE(537)、ORACLE_SAME_TARGET(1517)、ORACLE_BRANCH(871)をすべて読み込み、件数を assert し、`is_lane_drawable` も検査する | 空振りではない。既存の 537 と 1,517 は変更なしで通る。 |

### Summary

layout.rs の `skip_targets` は、分岐の検出(次の辺なし、辺はちょうど 2 本で行き先が異なる)、積み上げの規則(空か t2 が最も内側より近い)、分岐の線への後の合流と 3 本以上の辺は辺の一覧、列 0 の規則(`off_main`)、ノードの行の記号が通常どおりであることを、BR1.1 から BR4.2 のとおり実装している。テストは実際の挙動を検査し、依存、unsafe、cfg の違反もない。残るのは、正解データが git で確かめられないことと、カバレッジ・CI が未実行であることだけで、どちらも Minor である。
