# 単体テストの手順: u4-graph-dot

## テストの仕組み

- Rust の標準のテスト(`cargo test`)。新しい道具や依存は足さない。Graphviz は使わない(DOT の形は文字列の比較で確かめる)。
- cli の内部のテストは `crates/cli/src/tests/`、流れのテストは `crates/cli/tests/flow_*.rs`(偽の `Runtime`)、実バイナリのテストは `crates/cli/tests/binary_*.rs` と `unix_demo.rs`。
- テスト名は日本語。

## このユニットのテストの走らせ方

このユニットのテストだけを走らせる。どれも 1 つずつのコマンドで、後ろに別の手順を付けない。フィルタは `--` の後ろに書く。

```bash
cargo test -p conductor-cli --lib --locked -- tests::graph_dot tests::graph_report tests::docs
```

```bash
cargo test -p conductor-cli --test flow_graph --test binary_graph --test unix_demo --locked
```

- `windows_demo.rs` は Windows でだけ動くため、CI の windows の `test` で確かめる。

## 期待するカバレッジ

- 行カバレッジ 80% の下限はプロジェクト全体のもの。ユニットの確認のコマンド(`cargo llvm-cov --workspace ... --fail-under-lines 80`)と Build and Test が全体で測る。上のユニットのコマンドでは判定しない。下限は下げない。
- `graph_dot.rs` の新しい関数は、すべての分岐(記号の有無、太い枠、主線、種類の語、終点、行き先の分からない辺)をテストで通る。

## 偽物と差し替え

- 描画の単体テストは、今の `graph_report.rs` と同じく、定義の文字列から `FlowGraph` を組み立てて `render_dot` を呼ぶ(差し替え無し)。
- 流れのテストは今の偽の `Runtime` を使う。本番にテスト専用の引数・`pub`・関数を足さない。

## テストのデータ

- 定義は各テストの中の文字列で作る。分岐・飛び越し・終点を含む定義は、今の `graph_report.rs` の補助に倣う。
- 秘密のテストは、目印の文字列をコマンドとパイプライン名に入れる。
