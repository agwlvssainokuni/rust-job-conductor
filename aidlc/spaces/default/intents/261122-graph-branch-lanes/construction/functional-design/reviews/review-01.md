## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-22
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261122-graph-branch-lanes/construction/functional-design/functional-spec.md > §2 WF1 手順 3 と §5a.2、rules.md > BR1.4 / BR2.1 | 分岐が開いた線(t1 または t2)へ、あとのジョブが合流する形(例: `a` が `c` と `e` へ分岐し、`d` の成功も `e` へ飛ぶ。同じ行き先が t == top の合流になる)の扱いが決まっていない。BR2.1 は「分岐の線が既存の線の行き先と同じ」の向きだけを辺の一覧にする。逆向き(通常のジョブまたは「両方の飛び越し」が分岐の線に合流)は、§2 手順 3 の「合流の判定は前回までと同じ」と、現行の `skip_targets` の `t == top` 分岐と `lane_layout` の JoinLane により、そのまま図にできる形として通ってしまう。正解データ 871 例は「同じ行き先を含まない」形だけなので、この形の出力は未検証のまま出荷される。しかも BR1.4 の `all_off_main` の `&=` により、合流の行の列 0 が何になるかも、規則から一意に決まらない。 | 次のどちらかを規則に書く。(a) 分岐が開いた線への、あとからの合流は辺の一覧にする(`skip_targets` で、線ごとに「分岐が開いた」印を持ち、`t == top` の合流を拒む)。(b) 合流を図にするなら、その形を正解データに加えて照合する。テストにも、この形(`a` が `c`、`e` へ分岐し、`d` が `e` へ)を入れる。 | New |
| R-02 | Minor | aidlc/spaces/default/intents/261122-graph-branch-lanes/construction/functional-design/functional-spec.md > §6 テストの方針 | 書き換えが要るテストを「あれば」と曖昧に書いている。実際は確定している。`crates/core/tests/graph_lane_polish.rs` の `辺の一覧に残る形_異なる行き先が_2つ`(`a` が `c`、`d` へ)は、新しい仕様では図になるので必ず失敗する。`crates/core/tests/graph_drawable.rs` の `対応しない形_異なる行き先が_2つ`(118 行)も同じ。さらに、`crates/cli/tests/docs_graph.rs` の `readme_の辺の一覧に切り替わる形は_交差と異なる行き先の_2_つだけを述べる` は、README の文言に「異なる行き先」が残っていることを要求するので、README を直すと失敗する。この 3 件のうち docs_graph.rs の件は、spec のどこにも出てこない。`binary_graph.rs` と `graph_report.rs` は、交差の形だけを使っているので、変更は不要。 | §6 に、書き換える 3 件(graph_drawable.rs、graph_lane_polish.rs、docs_graph.rs)の名前と新しい期待(交差の形への差し替え、README の語句検査の更新)を明記する。 | New |
| R-03 | Minor | aidlc/spaces/default/intents/261122-graph-branch-lanes/construction/functional-design/functional-spec.md > §5a.6 と rules.md > BR4.2 | 文書の更新範囲が「README と仕様書」とだけある。「異なる行き先への飛び越しを 2 つ持つ形は辺の一覧」を述べる箇所は、README.md の 595 行、`docs/spec/rust/08-output-and-i18n.md` の 866 行、`docs/spec/rust/05-execution-flow.md` の 765 行、`docs/spec/language-agnostic/05-execution-flow.md` の 803 行、`08-output-and-i18n.md` の 895 行、`11-quality-and-build.md` の AC-474、AC-475 にある(AC-474 と AC-475 は、この形を辺の一覧と明記しており、テスト参照も含む)。 | BR4.2 の対象に、この箇所を列挙する。AC-474、AC-475 の更新と、分岐の新しい受け入れ基準の追加も入れる。 | New |
| R-04 | Minor | aidlc/spaces/default/intents/261122-graph-branch-lanes/construction/functional-design/functional-spec.md > §5a.1 と entities.md > SkipPlan | 実装の変更点の書き方が、現行の `skip_targets` と合っていない。現行は、`skip: Vec<Option<usize>>` と `both_skip` の 2 本で、第 2 の飛び越しで `skip[from].replace(to).is_some()` により即 `None` を返す(69 行)。また、条件 1 の検査(74〜79 行)は `!has_next[i]` で `None` を返すので、分岐のジョブを、この検査から明示的に除く必要がある。entities.md は `SkipKind`(None、NewLane、JoinLane、Branch)を述べるが、現行コードにこの型はなく、判定が `lane_layout` 側(`joins` の計算)にある。どちらを作るのかが書かれていない。edge.kinds は、分岐の辺では必ず 1 要素(行き先が異なれば合流しない)なので、`both_skip` と衝突しない点は問題ない。 | 設計に、(1) 第 2 の飛び越しは、辺が 2 本・どちらも次でない・行き先が異なる、のときだけ分岐として受ける、(2) `has_next` の検査から分岐を除く、(3) 戻り値の形(`Vec<Option<(usize, Option<usize>)>>` か `SkipKind` の列か)、の 3 点を書く。 | New |
| R-05 | Minor | aidlc/spaces/default/intents/261122-graph-branch-lanes/construction/functional-design/functional-spec.md > §2 手順 2、§5a.2 | 積み上げの確認は筋が通っている(t2 < top なら t1 < t2 < top なので、2 本とも既存の線のどれとも異なり、交差もしない。t1 == i+2 や終点への分岐も、押す順で矛盾しない。分岐ジョブ自身が線の行き先なら、先に合流の行で閉じるので、積み上げの先頭は閉じたあとの値になる)。ただし、§2 は「すでに開いている線の行き先と同じとき」を `t2 >= top` と別に書いており、実装者は同じ条件を二重に書く恐れがある。`t2 >= top` の判定だけで足りる(積み上げは単調に減るので、top より近い行き先が、他の開いた線と一致することはない)。 | 「t2 < top のとき、t1、t2 は開いている線のどれとも異なる」と 1 つにまとめる。積み上げが空のときは、`t2 < top` を常に真とする(終点への分岐を含む)ことも書く。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| validate-domain-model, validate-entities | 未実行(このステージの定義に、読み取り専用で走らせる検査が無い) | 規則 ID(BR1.1〜BR4.2)は rules.md と functional-spec.md で相互に参照できることを、目視で確認した |
| 現行コードとの照合(layout.rs、graph_text.rs) | 実施 | `-v` の詳細の置き場所は、分岐の行が `Fork` を含むことと、列数 = ノードの行の幅 ÷ 2 に頼る。これは新しい幅(1 + 既存 + 2)で正しく働く。`--state` は node_index で重ねるので影響なし。`graph_text.rs` は変更不要という主張は妥当 |

### Summary

分岐の判定、積み上げ、幅、分岐の行、`-v` と `--state` の扱いは、現行の layout.rs と整合しており、実装できる。重大な穴は 1 つで、分岐が開いた線へ、あとから通常のジョブが合流する形が、規則でも正解データでも未定義のまま図になってしまう点である(Major 1 件)。ほかは、書き換えが要るテストと文書の列挙が不足している点(Minor)で、承認前に直せば足りる。
