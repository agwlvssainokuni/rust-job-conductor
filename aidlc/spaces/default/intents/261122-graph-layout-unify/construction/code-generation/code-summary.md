# コード生成の要約(Code Summary): graph の配置のリファクタリング

計画 `code-generation-plan.md` の Step 1〜12 を、計画の順に、小さなコミットで `main` に積んだ(push はしていない)。規範は `functional-design/layout-reference.py`。git のソースは読んでいない(BR5.1)。

## コミット

| コミット | 内容 |
|----------|------|
| 4563ce8 | 判定: `SkipPlan` を「次のジョブへの辺の有無」「種類(`SkipKind`)」「行き先」にし、積み上げの走査と `ordinary` の欄を削除(BR1.1) |
| 37cbbe5 | 配置: `layout_lanes` を唯一の配置にし、「両方の飛び越し」と分岐の線の開き方、実行されうるジョブと主線の列 0 の規則を足す。`lane_layout`、`open_all_off_main`、`by_branch`、`debug_assert`、`merge_row` を削除(BR1.2〜BR1.4) |
| 6c09360 | `-v` の列 0: `LaneLayout.main_line` を足し、`render_lanes` の詳細の行の先頭に使う(BR1.4) |
| 06a61a3 | 正解データ: 分岐の第 2 版(871)に差し替え、`offmain`(2,748)と `offmain_67`(300)を追加。読み取りに `a>>d` を追加(BR4.1) |
| 344843a | 既存のテストの書き換え(BR2.1、BR3.1) |
| 4809b81 | 追加の例(N1〜N5、B1、D1、`main_line`)と性能(BR1.1〜BR2.1) |
| adc3972 | cli の整形(N1 の全文と `--state`、N1・N2・N5 の `-v`)と実バイナリ(N1) |
| fb5a08e | README |
| 6fb24e4 | 仕様書(rust と language-agnostic の 05、08、11、12) |

## 変えたファイル

- `crates/core/src/graph/layout.rs`: 判定(4 つの種類)、`reachable_nodes`(先頭のジョブからスタックでたどる。O(n+e))、`main_line_glyphs`、`move_rows`・`fork_row`・`join_lane_row`・`settle_duplicates` に列 0 の記号を渡す、`layout_lanes`(参照実装の `layout` の移植)。
- `crates/core/src/graph/mod.rs`: `LaneLayout` に `main_line: Vec<LaneGlyph>`(ノードごとの主線の列 0 の記号)。
- `crates/cli/src/report/graph_text.rs`: `render_lanes` の詳細の行の接頭辞の列 0 を `main_line` から取る。ほかの整形は変えていない。
- テスト(core): `graph_layout_oracle.rs`(読み取りと件数)、`graph_common/mod.rs`(`graph_with_extra_edge`)、`graph_drawable.rs`、`graph_lane_polish.rs`、`graph_branch_lanes.rs`、`graph_crossing_lanes.rs`、`graph_mixed_shapes.rs`、`graph_types.rs`、`graph_perf.rs`、新規 `graph_layout_unify.rs`。
- テスト(cli): `graph_report.rs`、`binary_graph.rs`、`docs_graph.rs`。
- 正解データ: `fixtures/graph_layout_oracle_branch.md`(差し替え)、`fixtures/graph_layout_oracle_offmain.md`、`fixtures/graph_layout_oracle_offmain_67.md`(新規)。git の出力の 4 つは無変更。
- 文書: `README.md`、`docs/spec/rust/{05,08,11,12}`、`docs/spec/language-agnostic/{05,08,11,12}`。
- 補助(この記録の中): `gen_offmain_67.py`(300 例の生成。乱数の種 20261122)、`print_layout.py`(テストの期待を参照実装で確かめる補助)。

## 確認

- `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`: 通過。
- `cargo test --locked --workspace`: 1,118 件が通過、失敗 0(macOS)。
- 正解データ 7 つ、計 7,400 例(537 + 1,517 + 940 + 487 + 871 + 2,748 + 300)が 1 文字も違わず一致。参照実装でも、`offmain`、`offmain_67`、`branch` の全例が一致することを確かめた。
- `Cargo.toml`、`Cargo.lock`、`demo/`、ほかの 5 つの仕様書は無変更。`unsafe` と `cfg(unix)`/`cfg(windows)` は足していない。

## 計画からの逸れ

- **定義から辺の一覧は出なくなった**: ジョブの結果は成功と警告の 2 つなので、定義から作れる図の構造では、どのジョブも 4 つの種類のどれかになる。そのため、辺の一覧に残る形のテストは、図の構造を直接作る形(辺が 3 本のジョブ、成功だけの辺が 1 本の非主線のジョブ、後ろ向きの辺)にし、cli の辺の一覧の整形(E1、E2、英語の注記、`-v`)は、整形の関数に `GraphLayout::EdgeList` を直接渡して確かめるようにした。実バイナリでは、辺の一覧を出す定義がもうないので、その確かめを、レーンの図の期待に替えた。
- **分岐の線の順**: 参照実装は `[t1, t2] + lanes` で、ラベルの順に並べる。正解データのラベルは常に近い行き先が先なので、実装は行き先を昇順に並べる(近い行き先が内側。成功と警告のどちらが近いかによらない。BR1.3 のとおり)。
- 関数名 `ordinary_layout` は、唯一の配置になったので `layout_lanes` に改めた(`is_lane_drawable` の名前は変えていない)。
- N4 は、仕様の §4 に図がない(質問の記録にだけある)ので、参照実装から導いた図(N1 と同じ見た目)でテストと仕様書に入れた。
- Step 2 のコミットの時点では、配置を分ける一時の補助 `is_ordinary_skip_graph` を残し、Step 3 で削除した(各コミットでビルドが通るようにするため)。
