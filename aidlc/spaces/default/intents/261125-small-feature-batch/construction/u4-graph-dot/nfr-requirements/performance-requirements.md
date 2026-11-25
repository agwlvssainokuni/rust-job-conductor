# 性能の要件: u4-graph-dot

出典: 要件(`requirements.md` の NFR1)、機能の仕様(`functional-spec.md` の 2)、ビジネスルール(`rules.md` の BR2.1)、契約(`contract-summary.md` の C5)、技術スタック(`technology-stack.md`)。

## 要件

| ID | 要件 | 合否の基準 | 出典 |
|---|---|---|---|
| NFR1.3 | DOT の描画は、節と辺の数に比例する手間にする(mermaid と同じく、ジョブ ID から節の ID を表で引く)。図の構造の計算は今の core のものを使い、増やさない | 今の図の性能のテスト(`crates/core/tests/graph_perf.rs`)の上限が変わらない。DOT の描画は mermaid と同じ形の 1 回の走査で書く | NFR1、BR2.1 |
