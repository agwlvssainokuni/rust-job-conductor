# テストの手順(Unit Test Instructions): graph の text 表示の改善

Minimal の戦略(必要なものだけ)。既存の Rust のテストの実行環境(`cargo test`)を使う。新しい設定は要らない。テスト名は日本語。

## 実行のコマンド(この作業の範囲だけ。どれも、プロジェクト全体の実行ではない)

```bash
# core: 正解データの照合(既存の 537 例と、新しい 1,517 例)
cargo test --locked -p conductor-core --test graph_layout_oracle

# core: 図にできるかの判定(書き換える 3 つの表明を含む)と、追加の例(D1〜D6、辺の一覧に残る形)
cargo test --locked -p conductor-core --test graph_drawable --test graph_lane_polish

# core: 図の型と、性能(入れ子と交差が、1,000 ジョブで 1 秒以内。変わらない)
cargo test --locked -p conductor-core --test graph_types --test graph_perf --test graph_overlay

# cli: text の整形(同じ行き先、両方の飛び越し、`-v`、`--state`、英語の注記)
cargo test --locked -p conductor-cli --test graph_report

# cli: 実バイナリと README の検査
cargo test --locked -p conductor-cli --test binary_graph --test docs_graph
```

これらは、この作業が触れるファイルのテストだけを動かす。ほかのテストの実行は、Build and Test が、プロジェクト全体で行う。

## テストの内容(Minimal: 成功の指標ごとに 1 つ以上)

| 指標 | テスト | ファイル |
|------|--------|----------|
| SM1(2 つの形が図で出る) | 同じ行き先の正解データ 1,517 例の照合。両方の飛び越しの例 D1〜D6。text の全文(S1、D1)。実バイナリ | `graph_layout_oracle.rs`、`graph_lane_polish.rs`、`graph_report.rs`、`binary_graph.rs` |
| SM2(ほかの形は変わらない) | 既存の正解データ 537 例の照合(変えない)。交差と、異なる行き先が 2 つの形が、辺の一覧のまま。書き換え対象でない既存のテストが通る | `graph_layout_oracle.rs`、`graph_drawable.rs`、`graph_perf.rs`、`graph_report.rs` |
| SM3(文書が実装に合う) | README の検査(新しい記述の語句) | `docs_graph.rs` |
| SM4(形ごとの正解の図で確かめる) | 同じ行き先の正解データ(git の実際の出力) | `graph_layout_oracle.rs` |

## 期待するカバレッジ

- プロジェクト全体の行カバレッジ 80% 以上(`team.md`、`project.md`)。**全体のしきい値は、Build and Test が、プロジェクト全体のテストで確かめる**(上のユニットの範囲のコマンドでは、確かめない)。
- 新しいコード(`layout.rs` の JoinLane と NoLane)は、正解データの照合と追加の例で、すべての分岐を通す。

## モックとスタブ

- core のテストは、純粋な関数(定義から図の配置を作る)だけを動かす。ファイルにもプロセスにも触れない。モックは要らない。
- cli の text の整形のテストは、定義を文字列から作り、整形の関数を直接呼ぶ。
- 実バイナリのテストは、`conductor` の実バイナリを、一時ディレクトリの定義ファイルで動かす。

## テストデータ

- 正解データ: `crates/core/tests/fixtures/graph_layout_oracle_same_target.md`(`layout-oracle-same-target.md` の写し。`git log --graph` の実際の出力。作り方は、そのファイルの先頭にある)。既存の `graph_layout_oracle.md` は、そのまま残す。
- 定義は、テストごとに、一時ディレクトリか文字列で作る(終わると消える)。
