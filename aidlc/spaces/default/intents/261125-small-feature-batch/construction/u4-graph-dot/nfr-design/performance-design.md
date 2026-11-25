# 性能の設計: u4-graph-dot

出典: 性能の要件(`performance-requirements.md` の NFR1.3)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 2)、契約(`contract-summary.md` の C5)。

## 設計

- `render_dot` は `render_mermaid` と同じ形にする: 節を 1 回走査してジョブ ID → 節の ID の表(`HashMap`)を作り、辺を 1 回走査して表で引く。手間は節と辺の数に比例する。
- 出力は 1 つの `String` に書き、最初に節と辺の数から見積もった大きさで領域を取る。
- 図の構造の計算(core)は今のものを使い、増やさない。

## 確かめ方

- 要件のレビューの指摘(今の `graph_perf.rs` は core の構造だけを測り、DOT の描画を通らない)に応えて、cli に DOT の描画の性能のテストを足す: 1 万のジョブが直線につながる `FlowGraph` を作り、`render_dot` が時間の上限の内に終わることを確かめる(`crates/cli/src/tests/graph_dot.rs`)。上限は、今の性能のテストと同じく、実測に余裕を持たせて Code Generation で決める。同じテストで mermaid の描画も測り、DOT が mermaid の数倍を超えないことも確かめる。
