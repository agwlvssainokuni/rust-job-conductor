# Code Generation Plan Approval

AI-DLC writes this file when it asks you to approve the plan. To answer here
instead of in chat, write your answer after `[Answer]:` and tell me in chat when you are done.

## Plan Approval

Approve the code plan?

- Builds: 4 つの段の整理。段 1: 説明のずれ(`apply_skip`、`Plain`、escape、`lib.rs` の表、`config_path`・65・`read_state_raw`、`definition_key`、仕様書 11 章の件数)と制約(core・converter の `forbid(unsafe_code)`、上限の数の const assert)。段 2...
- Touches: `crates/core/src`(flow、envfile、definition、graph、plan、types、error、lib)、`crates/converter/src`(lib、error)、`crates/runtime/src`(lib、state、error、file_create、platform/unix.rs の「親が空なら .」)、`crate...
- Tests: 既存のテストは変えずに通す。例外は承認済みの T1〜T9(期待する出力の文字列は変えない)。足すのは性能のテスト 3 件と小さな関数の単体テスト 4 件(1,128 → 1,135 の見込み)
- 設計の出典: `construction/functional-design/functional-spec.md`(§1〜§9。**§4 のテストの変更はユーザーが承認済み**、§5 の性能の基準、§8 のコミットの分け方)、**`inventory.md`(変える場所、目標の形、影響するテスト。実装はこれに従う)**、`rules.md`(BR1.1〜BR5.4)、`entities.m...

Full plan: aidlc/spaces/default/intents/261122-internal-cleanup-2/construction/code-generation/code-generation-plan.md
Test instructions: aidlc/spaces/default/intents/261122-internal-cleanup-2/construction/code-generation/unit-test-instructions.md

[Approval Fingerprint]: sha256:v3:43ede47da7c8e34e8dc5a852068fca16c1ce812f78eb4b83fa4499882e18b0e1
[Planned Source]: 770be047c3271f2264bd796cbb14fe1f3c0b2ab8d93f94b1caa8ee316df0d0d7

- A. Approve Plan
- B. Request Changes
- C. I'll edit the files

[Answer]: A. Approve Plan
