# 性能の要件: u3-unset-env

出典: 要件(`requirements.md` の NFR1)、機能の仕様(`functional-spec.md` の 2)、ビジネスルール(`rules.md` の BR2.1、BR2.2、BR3.1)、契約(`contract-summary.md` の C1)、技術スタック(`technology-stack.md`)。

## 要件

| ID | 要件 | 合否の基準 | 出典 |
|---|---|---|---|
| NFR1.3 | 重なりの検査と消す名前のまとめは、名前の数に比例する手間(表で引く)。今の環境の統合の性能のテストの上限を下げずに通る | `env_merge_perf.rs`、`env_refs_perf.rs` が今の上限で通る | NFR1、BR2.1、BR2.2、BR3.1 |
