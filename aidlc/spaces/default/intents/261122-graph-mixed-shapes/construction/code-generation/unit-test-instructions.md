# テストの手順(Unit Test Instructions): graph の text 表示の改善 その 4

Minimal の戦略。既存の Rust のテストの実行環境(`cargo test`)を使う。新しい設定は要らない。テスト名は日本語。

## 実行のコマンド(この作業の範囲だけ。プロジェクト全体の実行ではない)

```bash
# core: 正解データの照合(既存の 537、1,517、871、4,757 例と、新しい 487 例)
cargo test --locked -p conductor-core --test graph_layout_oracle

# core: 図にできるかの判定(書き換えを含む)、追加の例(M2〜M4、辺の一覧に残る形)、性能
cargo test --locked -p conductor-core --test graph_drawable --test graph_lane_polish --test graph_branch_lanes --test graph_crossing_lanes --test graph_mixed_shapes --test graph_perf

# core: 図の型
cargo test --locked -p conductor-core --test graph_types --test graph_overlay

# cli: text の整形(交差と同じ行き先、`_`、`-v`、`--state`)
cargo test --locked -p conductor-cli --test graph_report

# cli: 実バイナリと README の検査
cargo test --locked -p conductor-cli --test binary_graph --test docs_graph
```

## テストの内容

| 指標 | テスト | ファイル |
|------|--------|----------|
| SM1(範囲の形が図で出る) | 正解データ 487 例の照合。M2〜M4。text の全文。実バイナリ。性能 | `graph_layout_oracle.rs`、`graph_mixed_shapes.rs`、`graph_report.rs`、`binary_graph.rs`、`graph_perf.rs` |
| SM2(ほかの形は変わらない) | 既存の 4 つの正解データの照合。辺の一覧に残る形 | `graph_layout_oracle.rs`、`graph_mixed_shapes.rs`、`graph_drawable.rs`、`graph_lane_polish.rs`、`graph_branch_lanes.rs`、`graph_crossing_lanes.rs` |
| SM3(文書が実装に合う) | README の検査と、README の例が実出力と一致 | `docs_graph.rs` |

## 期待するカバレッジ

- プロジェクト全体の行カバレッジ 80% 以上。**全体のしきい値は、Build and Test が全体のテストで確かめる**(上のコマンドでは確かめない)。
- 新しいコード(`layout.rs` の寄せ)は、正解データの照合と追加の例で、すべての分岐を通す。

## モックとスタブ

- core のテストは、純粋な関数(定義から図の配置を作る)だけを動かす。cli の整形のテストは、定義を文字列から作り、整形の関数を直接呼ぶ。実バイナリのテストは、一時ディレクトリの定義ファイルで動かす。

## テストデータ

- 正解データ: `crates/core/tests/fixtures/graph_layout_oracle_mixed.md`(`layout-oracle-mixed.md` の写し。ジョブ 4〜6 個の 487 例。`git log --graph` の実際の出力。作り方は先頭に明記)。既存の 4 つは、そのまま残す。
- ジョブ 7 個の 3,861 例は、リポジトリに入れない。コード生成のときに、手元で 1 回だけ照合して、結果を報告する。
