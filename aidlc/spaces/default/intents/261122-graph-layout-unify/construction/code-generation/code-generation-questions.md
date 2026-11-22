# Code Generation Plan Approval

AI-DLC writes this file when it asks you to approve the plan. To answer here
instead of in chat, write your answer after `[Answer]:` and tell me in chat when you are done.

## Plan Approval

Approve the code plan?

- Builds: `graph` の配置を 1 つの仕組み(線の並びと寄せ)にし、古い経路を削除する。「両方の飛び越し」と分岐を、この仕組みの中で描き、交差・同じ行き先との組み合わせも図にする。主線の列 0 は、実行されうるジョブからの辺があるところだけ `|`(`-v` の詳細の行も同じ)
- Touches: `crates/core/src/graph/{layout,mod}.rs`、`crates/cli/src/report/graph_text.rs`(`-v` の列 0)、core と cli のテスト(書き換えと、正解データの差し替え・追加)、README、`docs/spec/rust`、`docs/spec/language-agnostic`
- Tests: 正解データ 2,748 + 871(作り直し)+ 300 例の照合、N1〜N5・B1・D1 の全文、`-v`、`--state`、性能(両方の飛び越し・分岐を含む 1,000 ジョブ)、実バイナリ(合計 12 本ほど)と、書き換える既存のテスト
- 設計の出典: `construction/functional-design/functional-spec.md`(§2〜§6b)、`rules.md`(BR1.1〜BR5.1)、**規範の参照実装 `layout-reference.py`**(Python。文と食い違うときは、こちらが正)。追跡の ID は成功の指標 `SM1`〜`SM4`。テストは Minimal、test-afte...

Full plan: aidlc/spaces/default/intents/261122-graph-layout-unify/construction/code-generation/code-generation-plan.md
Test instructions: aidlc/spaces/default/intents/261122-graph-layout-unify/construction/code-generation/unit-test-instructions.md

[Approval Fingerprint]: sha256:v3:e6241800a2297106ead95adafc78f66f48419b77574b276ae1555aee618c0520
[Planned Source]: 302085017571cab52295a856997a89f7ee16b23a98ee361899edbd00187ce952

- A. Approve Plan
- B. Request Changes
- C. I'll edit the files

[Answer]: A. Approve Plan
