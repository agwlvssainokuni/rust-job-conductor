# 信頼性の設計: u4-graph-dot

出典: 信頼性の要件(`reliability-requirements.md` の NFR1.1、NFR1.2、NFR2.1〜NFR2.4、NFR3.1、NFR3.2)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 2、4、6)、契約(`contract-summary.md` の C5)。

## 1. 後方互換(NFR1.1)

- text と mermaid の描画のコードは変えない。共有する補助(`state_symbol`、`kind_label`、`is_main_edge`)は呼ぶだけで、振る舞いを変えない。
- 使い方の文(`--format <text|mermaid|dot>`)と、`--format` の誤りの文は `dot` が入るように変える。直す場所は機能の設計のレビューの指摘どおり、`args.rs`、`report/messages.rs` の該当するすべての文、README の使い方の表と例と `-v` の説明、`src/tests/docs.rs` の検査の語句。これらを見る今のテストだけを直す。
- `docs/spec/rust/` の仕様の文書は、過去の版の設計の記録であり、このユニットでは直さない(README が利用者向けの正本)。

## 2. テストの配置(NFR1.2、NFR2.1〜NFR2.4)

test-after。cli の描画を作ってから、その層のテストを書く。

| テストの場所 | 確かめること | 規則 |
|---|---|---|
| `crates/cli/src/tests/graph_dot.rs`(新規) | escape の対の表、記号(`--state`)、太い枠(再開位置だけ)、主線のラベルなし、種類の語、終点、`-v` で同じ出力、節と辺の数が mermaid と同じ(分岐・飛び越し・終点を含む定義)、性能 | BR2.1〜BR3.1 |
| `crates/cli/src/tests/graph_report.rs` | 今の text・mermaid の出力が変わらない(今のテストのまま) | BR4.1 |
| `crates/cli/tests/flow_graph.rs` | 偽の `Runtime` で、`--format dot` が出力を 1 回で書き 0 で終わる。定義の誤りは text と同じ表示 | BR2.1、NFR6.3 |
| `crates/cli/tests/binary_graph.rs` | 実バイナリで `--format dot`、`--state`、`-v`、目印が出ないこと、`--format svg` の誤り | NFR2.2、NFR6.2 |
| `crates/cli/src/tests/docs.rs` | 使い方の表の 3 か所が揃う | BR1.2 |
| `crates/cli/tests/unix_demo.rs`、`windows_demo.rs` | デモの `graph` の場面で `digraph` の行が出る | NFR3.2 |

- 本番にテスト専用の関数や `pub` を足さない。`render_dot` は今の `render_mermaid` と同じく、表示のモジュールの公開の関数として使う(NFR2.4)。
- 行カバレッジ 80% は確認のコマンド、3 つの OS と MSRV は CI(NFR2.3、NFR3.1)。

## 3. 失敗のときの動き

- 描画は失敗しない純粋な関数。標準出力への書き込みの失敗は、今の `graph` と同じ扱い。
