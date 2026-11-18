# Code Generation Plan Approval

AI-DLC writes this file when it asks you to approve the plan. To answer here
instead of in chat, write your answer after `[Answer]:` and tell me in chat when you are done.

## Plan Approval

Approve the code plan for u3-retry-display-docs?

- Builds: リトライの前の知らせの文言(英語と日本語)の確定と、README へのリトライの説明(項目の書き方、動き、シグナル、at-least-once との関係)の追加。README の例が実際に通ることを確かめるテストを 1 つ足す
- Touches: `README.md`(目次、定義ファイルの書き方の表と新しい小節、ステートファイルの場所の「自動の再試行はしません」の一文、終了コードの表の 64、保証の範囲、シグナルの扱い)、`crates/cli/src/report/messages.rs`(文言を直す必要が出たときだけ)、`crates/cli/tests/report_messages.rs`(文言を直したとき...
- Tests: 新規の統合テスト 1 ファイル(約 4 件)。既存のテストは変更なしで通す。CI の `docs-check`(README の必須の語の検査)と同じ検査を、ローカルで実行する
- Already changed before you approved: crates/core/src/flow/mod.rs.

Full plan: aidlc/spaces/default/intents/261118-job-retry/construction/u3-retry-display-docs/code-generation/code-generation-plan.md
Test instructions: aidlc/spaces/default/intents/261118-job-retry/construction/u3-retry-display-docs/code-generation/unit-test-instructions.md

[Approval Fingerprint]: sha256:v3:6a2c750b20579accbd56e3450f489f1def7c508102565f9e0726960104b54b11
[Planned Source]: 1cfc43e42c15f6021e8ad45b5b2291aaaf64b7f69ef599ce728b2cce6e43fb87

- A. Approve Plan
- B. Request Changes
- C. I'll edit the files

[Answer]: A. Approve Plan
