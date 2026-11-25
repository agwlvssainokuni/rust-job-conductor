# Code Generation Plan Approval

AI-DLC writes this file when it asks you to approve the plan. To answer here
instead of in chat, write your answer after `[Answer]:` and tell me in chat when you are done.

## Plan Approval

Approve the code plan for u7-watch-follow?

- Builds: `conductor watch --follow [--interval <秒>]` で、ステートを一定の間隔(既定 2 秒、1〜3600)で描き直す。完了・消えた・止まったら最後に描いて 0、Ctrl-C でも 0。UNIX 系の端末は画面を消して描き直し、ほかは追記。リトライの待ちの間は試行を開いたままにする(U6 の流れの直し)。最後に版を 0.10.0 にする
- Touches: core の `flow/activity.rs`(新規)・`flow/mod.rs`・`lib.rs`、cli の `args.rs`・`types.rs`・`aux_commands.rs`・`step.rs`・`platform/`・`report/mod.rs`・`report/messages.rs`、テスト、`README.md`、デモ、版の更新(4 つの `C...
- Tests: 単体テスト約 15(判定の 4 つの値、引数)、流れのテスト約 12(継続表示の読み込みの並び、合図、リトライの待ち)、実バイナリのテスト 3、docs のテスト、デモ
- 出典: 機能の設計(`functional-spec.md`、`rules.md` の BR1.1〜BR5.2、`entities.md`)、非機能の設計(`reliability-design.md`、`security-design.md`、`performance-design.md`、`scalability-design.md`、`observability-design.md`、...
- テストの進め方は Testing Contract(test-after)に従う。core → cli の順に、層を作ってからその層のテストを書いて走らせる。
- Already changed before you approved (7 files): crates/cli/src/report/mod.rs, crates/cli/tests/nfr_robust.rs, crates/core/src/flow/attempt.rs, crates/core/src/flow/decide.rs, crates/core/src/flow/mod.rs, crates/core/src/flow/next_step.rs, crates/core/src/flow/resume.rs.

Full plan: aidlc/spaces/default/intents/261125-small-feature-batch/construction/u7-watch-follow/code-generation/code-generation-plan.md
Test instructions: aidlc/spaces/default/intents/261125-small-feature-batch/construction/u7-watch-follow/code-generation/unit-test-instructions.md

[Approval Fingerprint]: sha256:v3:4a686e35be89ca7bfbede641d2e61e207bb6192ecb65ab74e4077e7602f38b8c
[Planned Source]: 4689c235ebb9de0cf4fff4c330de7fbdbcf048ec56783faffb2162afccc5c368

- A. Approve Plan
- B. Request Changes
- C. I'll edit the files

[Answer]: A. Approve Plan
