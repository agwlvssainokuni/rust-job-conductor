# Code Generation Plan Approval

AI-DLC built this plan without asking because plan approval is off for this
piece of work. This file is the record and asks nothing; to look at a plan
before it is built, ask in chat.

## Plan Approval

Built without asking: plan approval is off for this piece of work (from scope express).

- Builds: ジョブの `command` に、文字列に加えて、文字列の配列を書けるようにする(定義の検証、起動、表示、`convert` の往復、README、デモ)。
- Touches: `crates/core`(`Command` 型、定義の読み込み)、`crates/runtime`(`check_command`、`run_job`)、`crates/cli`(`Runtime` の trait、起動失敗の名前)、`crates/converter`(テストのみ)、`README.md`、`demo/`、各 `Cargo.toml` の版(0.2.0...
- Tests: 新しいテスト 約 15 件(要件ごとに 1 件が目安。Minimal の方針)と、既存の 564 件を成功のまま保つ。行カバレッジ 80% 以上を保つ(チームの下限)。

Full plan: aidlc/spaces/default/intents/261115-command-array/construction/code-generation/code-generation-plan.md
Test instructions: aidlc/spaces/default/intents/261115-command-array/construction/code-generation/unit-test-instructions.md

[Approval Fingerprint]: sha256:v3:8f35d335077aa7f0f17844382d3707c2bbaf4baa2b7e268cf6c7848660d5e816
[Planned Source]: 1eef7432805597bede2a43c476d6876d2acd66405671fd4505de3ca4387a868c


[Answer]: Plan approval off
