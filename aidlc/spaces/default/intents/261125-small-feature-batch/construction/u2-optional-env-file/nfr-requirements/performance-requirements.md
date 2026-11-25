# 性能の要件: u2-optional-env-file

出典: 要件(`requirements.md` の NFR1)、機能の仕様(`functional-spec.md` の 3)、ビジネスルール(`rules.md` の BR2.1、BR2.2)、契約(`contract-summary.md` の C1)、技術スタック(`technology-stack.md`)。

## 要件

| ID | 要件 | 合否の基準 | 出典 |
|---|---|---|---|
| NFR1.3 | 読む対象の一覧の重複除き(パスの文字列)と、解決したパスでのまとめは、項目の数に比例する手間にする。今の `.env` の性能のテスト(`env_refs_perf.rs`)の上限を下げずに通る | `crates/core/tests/env_refs_perf.rs` が今の上限のまま通る | NFR1、BR2.1、BR2.2 |
