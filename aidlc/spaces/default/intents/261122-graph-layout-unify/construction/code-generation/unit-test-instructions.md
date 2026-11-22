# テストの手順(Unit Test Instructions): graph の配置のリファクタリング

Minimal の戦略。既存の Rust のテストの実行環境(`cargo test`)を使う。新しい設定は要らない。テスト名は日本語。

## 実行のコマンド(この作業の範囲だけ。プロジェクト全体の実行ではない)

```bash
# core: 正解データの照合(git の出力 537、1,517、940、487 例と、規則から導いた 871、2,748、300 例)
cargo test --locked -p conductor-core --test graph_layout_oracle

# core: 判定、追加の例(N1〜N5、B1、D1)、性能
cargo test --locked -p conductor-core --test graph_drawable --test graph_lane_polish --test graph_branch_lanes --test graph_crossing_lanes --test graph_mixed_shapes --test graph_layout_unify --test graph_perf

# core: 図の型
cargo test --locked -p conductor-core --test graph_types --test graph_overlay

# cli: text の整形(`-v` の詳細の行の列 0、`--state`)
cargo test --locked -p conductor-cli --test graph_report

# cli: 実バイナリと README の検査
cargo test --locked -p conductor-cli --test binary_graph --test docs_graph
```

## テストの内容

| 指標 | テスト | ファイル |
|------|--------|----------|
| SM1(1 つの仕組み) | 古い経路の削除(ビルドと clippy が通ること)、全正解データの照合 | `graph_layout_oracle.rs` |
| SM2(既存の出力は変わらない) | git の出力の正解データ 537、1,517、940、487 例 | `graph_layout_oracle.rs` |
| SM3(組み合わせの形が図になる) | 2,748、300 例、N1〜N5、`-v`、性能、実バイナリ | `graph_layout_oracle.rs`、`graph_layout_unify.rs`、`graph_report.rs`、`binary_graph.rs`、`graph_perf.rs` |
| SM4(文書が実装に合う) | README の検査と、README の例が実出力と一致 | `docs_graph.rs` |

## 期待するカバレッジ

- プロジェクト全体の行カバレッジ 80% 以上。**全体のしきい値は、Build and Test が全体のテストで確かめる**(上のコマンドでは確かめない)。

## モックとスタブ

- core のテストは、純粋な関数だけを動かす。cli の整形のテストは、整形の関数を直接呼ぶ。実バイナリのテストは、一時ディレクトリの定義ファイルで動かす。

## テストデータ

- `crates/core/tests/fixtures/graph_layout_oracle_branch.md`: 作り直した 871 例(`layout-oracle-branch-v2.md`)に差し替える。
- `crates/core/tests/fixtures/graph_layout_oracle_offmain.md`(新規、2,748 例)、`graph_layout_oracle_offmain_67.md`(新規、300 例): **git の出力ではなく、規則から導いた図**(先頭に明記)。
- git の出力の正解データ 4 つは、そのまま。
