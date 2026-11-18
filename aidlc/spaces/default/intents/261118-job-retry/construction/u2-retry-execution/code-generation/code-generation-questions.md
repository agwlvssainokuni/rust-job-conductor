# Code Generation Plan Approval

AI-DLC writes this file when it asks you to approve the plan. To answer here
instead of in chat, write your answer after `[Answer]:` and tell me in chat when you are done.

## Plan Approval

Approve the code plan for u2-retry-execution?

- Builds: 設定(u1 の `RetryPolicy`)に従って、対象の終了コードで終わったジョブを、間隔を空けて最大 N 回実行し直す(`run` と `pipeline` の共通の処理)。固定の間隔と、倍率と上限の方式、待機中のシグナルでの即時の中断、リトライの前の知らせ(常に標準エラー、英語と日本語)
- Touches: `crates/core/src/flow/retry.rs`(新規)と `flow/mod.rs`・`lib.rs`、`crates/runtime/src/wait.rs`(新規)と `lib.rs`、`crates/cli/src/deps.rs`(`Runtime` に「待つ」)、`crates/cli/src/retry.rs`(新規)、`crates/cli/s...
- Tests: 新規の統合テスト 3 ファイル(core の判断、runtime の待機、cli のリトライの流れ。合わせて約 45 件)と、cli の文言のテストへの追加。既存の約 755 件は変更なしで通す(偽の `Runtime` への追加を除く)
- Already changed before you approved (9 files): crates/core/src/definition/duration.rs, crates/core/src/definition/mod.rs, crates/core/src/flow/classify.rs, crates/core/src/flow/decide.rs, crates/core/src/flow/next_step.rs, crates/core/src/flow/resume.rs, crates/runtime/src/state/consistency.rs, crates/runtime/src/state/record.rs, crates/runtime/tests/common/mod.rs.

Full plan: aidlc/spaces/default/intents/261118-job-retry/construction/u2-retry-execution/code-generation/code-generation-plan.md
Test instructions: aidlc/spaces/default/intents/261118-job-retry/construction/u2-retry-execution/code-generation/unit-test-instructions.md

[Approval Fingerprint]: sha256:v3:19240ba7145d025cf970bbaf3cde33f57d64f9e3520e64913eb30cbc0a47b18a
[Planned Source]: d986f8b1e4f4d401bd61e5073111d404634ff72efcfa59b4f9f10f8429b080c7

- A. Approve Plan
- B. Request Changes
- C. I'll edit the files

[Answer]: A. Approve Plan
