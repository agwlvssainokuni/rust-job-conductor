# 性能の設計: u3-unset-env

出典: 性能の要件(`performance-requirements.md` の NFR1.3)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 2)、契約(`contract-summary.md` の C1)。

## 設計

- 重なりの検査は、消す名前を ASCII の小文字にした集合(`HashSet`)を作り、`env` と `.env` の名前を小文字にして引く。手間は名前の数に比例する。
- `removals` のまとめは、見た名前の集合で重複を除く(完全一致)。
- 確かめ方: 今の `env_merge_perf.rs`・`env_refs_perf.rs` が上限を変えずに通る。消す名前を多数持つ入力を `env_merge_perf.rs` に 1 つ足し、同じ上限で確かめる。
