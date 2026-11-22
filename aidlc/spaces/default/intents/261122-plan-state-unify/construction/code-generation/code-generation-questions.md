# Code Generation Plan Approval

AI-DLC writes this file when it asks you to approve the plan. To answer here
instead of in chat, write your answer after `[Answer]:` and tell me in chat when you are done.

## Plan Approval

Approve the code plan?

- Builds: ステートの更新の関数(`record.rs`)を runtime から core の `flow/record.rs` へ移し、runtime は再エクスポートする。`apply_skip` を O(n) にする。模擬(`plan_pipeline`)は `PipelineState` を持ち、本番と同じ `apply_skip`・`apply_outcome` で記録する(...
- Touches: `crates/core/src/{lib.rs, flow/mod.rs, flow/record.rs, plan/simulate.rs}`、`crates/runtime/src/{lib.rs, state/mod.rs}`(`state/record.rs` は移動)、`crates/core/tests/plan_perf.rs`、`crates/core/t...
- Tests: 既存のテストを変えずにすべて通す。足すのは、性能(終点の後の 1,000 個近いスキップ)と、承認した手書きのステートの形の 2 本。実装の後に、書き換える前の模擬と、90 万例で比べる(一時のテスト。削除する)
- 設計の出典: `construction/functional-design/functional-spec.md`(§1〜§5)、`rules.md`(BR1.1〜BR4.1)。追跡の ID は成功の指標 `SM1`〜`SM5`。テストは Minimal、test-after、行カバレッジ 80% の下限(Build and Test が全体で確かめる)。新しい依存、`unsafe`、`c...

Full plan: aidlc/spaces/default/intents/261122-plan-state-unify/construction/code-generation/code-generation-plan.md
Test instructions: aidlc/spaces/default/intents/261122-plan-state-unify/construction/code-generation/unit-test-instructions.md

[Approval Fingerprint]: sha256:v3:394844fc52ddc50db924d149888b6af85088d4ef083baacb231a6f9013ff08de
[Planned Source]: 15ed1ed7a2b6829744c2a02b8e347f8547aff3ff8d02aa8434997d2c00443fe8

- A. Approve Plan
- B. Request Changes
- C. I'll edit the files

[Answer]: A. Approve Plan
