## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-22
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | crates/core/src/graph/layout.rs > skip_targets の `to <= from` 判定と `GraphLayout::EdgeList`(および graph_text.rs の英語の注記) | 辺の一覧は、定義からは出ない。ジョブごとの辺は 1 本か 2 本で、遷移先は `walk.rs:727` の検査で記述順の後ろ(または `@end`)に限られ、既定の遷移も次のジョブになるので、後ろ向きの辺や自分への辺は作れない。残りの経路は、図の構造を直接作ったときの防御としてだけ生きる。これは 05 の 15.4 と 08 の 11.4 に書かれ、テストは `graph_with_extra_edge` で到達を確かめている。ただし注記の文言は「飛び越しが交差するなどの形」のままで、今の理由(後ろ向きの辺、辺が 3 本以上)と合わない。 | 経路は残してよい(消すと、不正な図の構造で panic か誤った図になる)。注記の文言を実際の理由(図の構造が 4 つの種類を外れる)に合わせるか、次の改訂まで持ち越すと記録する。 | New |
| R-02 | Minor | crates/core/src/graph/layout.rs > layout_lanes / skip_targets(分岐の線の並び) | 分岐の 2 本は行き先の昇順(近い方が内側)に並べる。参照実装は入力順のままで、正解データは近い方が先のものだけを含むので、「成功と警告のどちらが近いか問わない」は、参照実装が直接は検証していない正規化である。図に辺の種類の表示はなく、線は行き先だけで決まるので、見た目は変わらない。移植は参照実装と一致する(判定、寄せ、`q`/`qonly`、`after_fork`、列 0 を、行ごとに突き合わせて確認した)。 | 対応は不要。AC-476e の文言に、「正規化であって、正解データの範囲外」と一言足すとよい。 | New |
| R-03 | Minor | git log 37cbbe5..344843a、および fb5a08e より前の docs_graph | 中間のコミットで、テストが失敗する(37cbbe5 から 344843a まで、docs_graph は fb5a08e まで)。チームは「マージ前に CI を通す」と決めており、`main` へ直接コミットするので、この区間は `git bisect` を妨げる。最終の HEAD は、テスト 1,118 件が通り、`cargo fmt --check` と `clippy -D warnings` もそのまま通ることを確認した。 | 履歴を書き換えない。code-summary.md に、既知の区間として 1 行記録する。次からは、配置とテストの更新を同じコミットにする。 | New |
| R-04 | Minor | aidlc/spaces/default/intents/261122-graph-layout-unify/construction/code-generation/gen_offmain_67.py, print_layout.py | 正解データの生成と表示の補助スクリプトを intent の記録の下にコミットした。リリースの成果物やクレートには含まれず、依存も増えない。ただし 7 ジョブまでを生成する手順が、再現できる形で残るのはこの 2 つだけで、正解データ(offmain は 48,527 行)の由来は、コードの側から見えない。 | 残してよい。正解データのヘッダかテストのコメントに、生成元のスクリプトの場所を 1 行書く。 | New |
| R-05 | Minor | crates/core/tests/graph_perf.rs(1,000 ジョブ) | 1,000 ジョブの性能は、時間(1 秒以内)だけを確かめる。正解データの正しさは 7 ジョブまでで、大きい入力での正しさは、小さい入力と同じ仕組みであることに頼る。計算量は、到達の判定が辺の数に比例し、寄せの重複探しが線の数に比例する(寄せの回数は作った線の数以下なので、全体は高々 2 乗)ので、1,000 ジョブでは問題ない。 | 対応は不要。余裕があれば、1,000 ジョブの出力に、行数と末尾の `o @end` の確認を足す。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| cargo fmt --all -- --check | PASS | 差分なし |
| cargo clippy --workspace --all-targets --locked -- -D warnings | PASS | 警告なし |
| 旧い経路の残りの検索(`lane_layout`、`ordinary_layout` など) | コード、テストに 0 件。05 の 15.5 に、「なくした」という説明だけ 1 件 | 古い経路は消えている |
| 依存、`unsafe`、`cfg(` の差分(510382e..HEAD、crates と Cargo.toml、Cargo.lock) | 追加なし | 新しい依存、`unsafe`、`cfg` はない |
| 新しいテスト関数の名前 | テスト関数は日本語。ASCII の名前は補助関数だけ | 規約どおり |
| テスト全体(開発者の報告) | 1,118 件が通り、失敗 0 | 私は全体を再実行していない(fmt と clippy は実行した) |

### Summary

layout.rs は layout-reference.py と判定、列 0、寄せ、重複の探し方まで一致し、旧い経路は削除され、1,000 ジョブでも計算量は問題ない。-v の列 0 は `main_line` から取るので配置と同じ規則に従い、README と仕様(05、08、11、12)も実装と整合する。辺の一覧は定義からは到達せず(遷移は後ろ向き不可)、防御として残すのが妥当で、残るのは注記の文言、中間コミットの失敗、補助スクリプトの記録という、軽い指摘だけである。
