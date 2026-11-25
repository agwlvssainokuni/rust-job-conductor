# 信頼性の要件: u4-graph-dot

出典: 要件(`requirements.md` の NFR1、NFR2、NFR3、FR4.1、FR4.4)、機能の仕様(`functional-spec.md` の 2、4、6)、ビジネスルール(`rules.md` の BR2.1、BR4.1、BR4.2)、契約(`contract-summary.md` の C5)、技術スタック(`technology-stack.md`)、team.md の Testing Posture。

## 要件

| ID | 要件 | 合否の基準 | 出典 |
|---|---|---|---|
| NFR1.1 | `--format dot` を使わない `graph` の出力(text・mermaid)と使い方の誤りの扱いは今と同じ。使い方の表の文には `dot` が入る | 今の `graph` のテスト(`binary_graph.rs`、`flow_graph.rs`、`src/tests/graph_report.rs`、参照出力の `graph_layout_oracle.rs`)が、使い方の文を見るものを除いて変えずに通る | NFR1、FR4.4、BR4.1 |
| NFR1.2 | DOT のジョブの節の数と辺の数は、同じ定義の mermaid と一致する | 分岐・飛び越し・終点を含む定義で、DOT と mermaid の節と辺の数を比べるテスト | FR4.1、BR2.1 |
| NFR2.1 | test-after。cli の描画を作ってから、その単体テスト(escape の対の表、記号、太い枠、`-v`、主線のラベル)を書く | 規則ごとに正しい場合と境界の場合のテストがある | NFR2 |
| NFR2.2 | 実バイナリのテストで、`graph --format dot`、`--state`、`-v` の出力と、`--format` の誤りを確かめる | `binary_graph.rs` に足したテストが通る | NFR2 |
| NFR2.3 | 行カバレッジ 80% 以上を保つ | 確認のコマンドが通る | NFR2 |
| NFR2.4 | テストの都合で本番のコードを曲げない | 本番にテスト専用の項目が無い | NFR2、project.md |
| NFR3.1 | 3 つの OS と MSRV の CI が通る。DOT の改行は `\n` で、OS によらず同じ | CI の `test`、`build-check`、`msrv-test` | NFR3 |
| NFR3.2 | 更新したデモで、デモのテストが通る | `unix_demo.rs`(手元)、`windows_demo.rs`(CI) | NFR3、BR4.2 |
