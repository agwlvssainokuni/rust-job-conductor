# Code Generation Plan Approval

AI-DLC writes this file when it asks you to approve the plan. To answer here
instead of in chat, write your answer after `[Answer]:` and tell me in chat when you are done.

## Plan Approval

Approve the code plan for u3-cli?

- Builds: `conductor` の実行ファイル(パッケージ `conductor-cli`)。コマンド(run・pipeline・watch・reset・convert)、流れの組み立て、シグナル、表示(英語・日本語)、終了コード、利用者向けの README。あわせて、u1-core の定義ファイルのキーを `jobs` から `job` に直す
- Touches: `crates/u3-cli/`(新規。ソース、テスト、テスト専用の補助の実行ファイル)、`crates/u1-core/`(キー名の変更。`src/definition/walk.rs`、テスト、例)、ワークスペースの `Cargo.toml`(`[profile.release]`)と `Cargo.lock`、`README.md`(新規)
- Tests: 約 130 件(引数、終了コード、表示、流れ(偽の依存)、シグナル、binary を起動する少数のテスト、横断。Standard の方針と、行カバレッジ 80% の下限)

Full plan: aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/code-generation/code-generation-plan.md
Test instructions: aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u3-cli/code-generation/unit-test-instructions.md

[Approval Fingerprint]: sha256:v3:2e90dd17258da4bc20c4d547c0642efa8ea15699f17609a6bc5ead8f133d1832
[Planned Source]: 8e51d896ff555f1e39f237676ed89caef4673e5fa25de288a0294f2da89627bb

- A. Approve Plan
- B. Request Changes
- C. I'll edit the files

[Answer]: A. Approve Plan
