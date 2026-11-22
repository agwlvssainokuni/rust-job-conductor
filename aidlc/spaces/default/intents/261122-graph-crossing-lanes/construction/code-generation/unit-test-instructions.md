# テストの手順(Unit Test Instructions): graph の text 表示の改善 その 3

Minimal の戦略。既存の Rust のテストの実行環境(`cargo test`)を使う。新しい設定は要らない。テスト名は日本語。

## 実行のコマンド(この作業の範囲だけ。プロジェクト全体の実行ではない)

```bash
# core: 正解データの照合(既存の 537、1,517、871 例と、新しい交差の 940 例)
cargo test --locked -p conductor-core --test graph_layout_oracle

# core: 図にできるかの判定(書き換えを含む)、追加の例(C1〜C3、辺の一覧に残る形)、性能
cargo test --locked -p conductor-core --test graph_drawable --test graph_lane_polish --test graph_branch_lanes --test graph_crossing_lanes --test graph_perf

# core: 図の型
cargo test --locked -p conductor-core --test graph_types --test graph_overlay

# cli: text の整形(交差、`_`、`-v`、`--state`)
cargo test --locked -p conductor-cli --test graph_report

# cli: 実バイナリと README の検査
cargo test --locked -p conductor-cli --test binary_graph --test docs_graph
```

## テストの内容

| 指標 | テスト | ファイル |
|------|--------|----------|
| SM1(交差する形が図で出る) | 交差の正解データ 940 例の照合。C1〜C3。text の全文。実バイナリ。性能 | `graph_layout_oracle.rs`、`graph_crossing_lanes.rs`、`graph_report.rs`、`binary_graph.rs`、`graph_perf.rs` |
| SM2(ほかの形は変わらない) | 既存の正解データ 537、1,517、871 例の照合。辺の一覧に残る混ざった例 | `graph_layout_oracle.rs`、`graph_crossing_lanes.rs`、`graph_drawable.rs`、`graph_lane_polish.rs`、`graph_branch_lanes.rs` |
| SM3(文書が実装に合う) | README の検査と、README の例が実出力と一致 | `docs_graph.rs` |

## 期待するカバレッジ

- プロジェクト全体の行カバレッジ 80% 以上。**全体のしきい値は、Build and Test が全体のテストで確かめる**(上のコマンドでは確かめない)。
- 新しいコード(`layout.rs` の交差の行)は、正解データの照合と追加の例で、すべての分岐を通す。

## モックとスタブ

- core のテストは、純粋な関数(定義から図の配置を作る)だけを動かす。cli の整形のテストは、定義を文字列から作り、整形の関数を直接呼ぶ。実バイナリのテストは、一時ディレクトリの定義ファイルで動かす。

## テストデータ

- 正解データ: `crates/core/tests/fixtures/graph_layout_oracle_cross.md`(`layout-oracle-cross.md` の写し。ジョブ 3〜7 個の 940 例。`git log --graph` の実際の出力。作り方は先頭に明記)。既存の 3 つは、そのまま残す。
- ジョブ 8 個の 3,817 例は、リポジトリに入れない。コード生成のときに、手元で 1 回だけ照合して、結果を報告する。
