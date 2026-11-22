# Code Generation Plan Approval

AI-DLC writes this file when it asks you to approve the plan. To answer here
instead of in chat, write your answer after `[Answer]:` and tell me in chat when you are done.

## Plan Approval

Approve the code plan?

- Builds: `crates/cli/src/step.rs`(新規)に、1 つのジョブの共通の手順(`decide`、`record_skip`、`run_step`、`remove_if_requested`)を置き、`run.rs` と `pipeline.rs` はそれを呼ぶだけにする。いつ消すかは呼び出し側に残す。動きは 1 文字も変えない
- Touches: `crates/cli/src/{step,run,pipeline,lib}.rs`、cli の流れのテスト(通っていない枝だけを足す)、`docs/spec/rust/05-execution-flow.md`(§5.2、§5.3 と共通の手順の節)、`docs/spec/rust/02-architecture.md`(モジュールの表、呼び出しの木)
- Tests: 既存のテストを変えずにすべて通す。足すのは、通っていない枝の流れのテスト(2〜3 本)だけ
- 設計の出典: `construction/functional-design/functional-spec.md`(§1〜§5)、`rules.md`(BR1.1〜BR4.1)。追跡の ID は成功の指標 `SM1`〜`SM4`。テストは Minimal、test-after、行カバレッジ 80% の下限(Build and Test が全体で確かめる)。新しい依存、`unsafe`、`c...

Full plan: aidlc/spaces/default/intents/261122-cli-step-unify/construction/code-generation/code-generation-plan.md
Test instructions: aidlc/spaces/default/intents/261122-cli-step-unify/construction/code-generation/unit-test-instructions.md

[Approval Fingerprint]: sha256:v3:fc1bc64e612c2147b13f087c5154d79447f30c14601f44641d803929de58edfb
[Planned Source]: 3549f5b8377df59b0786e1d9203874c46de9cd722dd865ad3201de1f9b7bab9f

- A. Approve Plan
- B. Request Changes
- C. I'll edit the files

[Answer]: A. Approve Plan
