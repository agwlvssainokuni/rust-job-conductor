# テストの手順(Unit Test Instructions): graph の text 表示の改善 その 2

Minimal の戦略。既存の Rust のテストの実行環境(`cargo test`)を使う。新しい設定は要らない。テスト名は日本語。

## 実行のコマンド(この作業の範囲だけ。プロジェクト全体の実行ではない)

```bash
# core: 正解データの照合(既存の 537 例、1,517 例と、新しい 871 例)
cargo test --locked -p conductor-core --test graph_layout_oracle

# core: 図にできるかの判定(書き換えを含む)と、分岐の追加の例(B1〜B4、辺の一覧に残る形)
cargo test --locked -p conductor-core --test graph_drawable --test graph_lane_polish --test graph_branch_lanes

# core: 図の型と、性能(変わらない)
cargo test --locked -p conductor-core --test graph_types --test graph_perf --test graph_overlay

# cli: text の整形(分岐、`-v`、`--state`)
cargo test --locked -p conductor-cli --test graph_report

# cli: 実バイナリと README の検査
cargo test --locked -p conductor-cli --test binary_graph --test docs_graph
```

## テストの内容

| 指標 | テスト | ファイル |
|------|--------|----------|
| SM1(この形が図で出る) | 分岐の正解データ 871 例の照合。B1〜B4。text の全文。実バイナリ | `graph_layout_oracle.rs`、`graph_branch_lanes.rs`、`graph_report.rs`、`binary_graph.rs` |
| SM2(ほかの形は変わらない) | 既存の正解データ 537 例と 1,517 例の照合。辺の一覧に残る形(交差、分岐の線への合流、`t2 >= top`) | `graph_layout_oracle.rs`、`graph_drawable.rs`、`graph_lane_polish.rs`、`graph_branch_lanes.rs` |
| SM3(文書が実装に合う) | README の検査と、README の例が実出力と一致 | `docs_graph.rs` |

## 期待するカバレッジ

- プロジェクト全体の行カバレッジ 80% 以上。**全体のしきい値は、Build and Test が全体のテストで確かめる**(上のコマンドでは確かめない)。
- 新しいコード(`layout.rs` の分岐)は、正解データの照合と追加の例で、すべての分岐を通す。

## モックとスタブ

- core のテストは、純粋な関数(定義から図の配置を作る)だけを動かす。cli の整形のテストは、定義を文字列から作り、整形の関数を直接呼ぶ。実バイナリのテストは、一時ディレクトリの定義ファイルで動かす。

## テストデータ

- 正解データ: `crates/core/tests/fixtures/graph_layout_oracle_branch.md`(`layout-oracle-branch.md` の写し。**git の出力ではなく、規則から導いた図**。ファイルの先頭に明記)。既存の `graph_layout_oracle.md` と `graph_layout_oracle_same_target.md` は、そのまま残す。
