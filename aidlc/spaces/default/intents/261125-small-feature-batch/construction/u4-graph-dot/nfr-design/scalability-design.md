# 規模の設計: u4-graph-dot

出典: 規模の要件(`scalability-requirements.md` の NFR1.4)、性能の要件(`performance-requirements.md`)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 2)、契約(`contract-summary.md` の C5)。

## 設計

- 扱える定義の大きさは、今の定義の読み込みの上限(`crates/core/tests/definition_perf.rs` などが確かめる大きさ)と同じ。DOT の出力の大きさは、節と辺の数に比例する(1 行に 1 つ)。
- 確かめ方: 性能の設計の 1 万のジョブのテストで、出力の行の数が「節の数 + 辺の数 + 3」であることも確かめ、大きな図でも出力が欠けないことを見る。
