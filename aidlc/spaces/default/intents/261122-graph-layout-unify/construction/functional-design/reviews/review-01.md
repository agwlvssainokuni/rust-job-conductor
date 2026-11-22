## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-22
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261122-graph-layout-unify/construction/functional-design/functional-spec.md > §3 項目 3 と §6(`-v`)、crates/cli/src/report/graph_text.rs の render_lanes | `-v` の詳細の行は、列の数だけ `\| ` を無条件に描く(prefix は `"\| ".repeat(columns)`)。Q1: B の規則で、実行されないジョブ(例: N1 の `b`、`c`)や次のジョブへの辺を持たないジョブの前後の行は列 0 が空白になる。そのジョブの詳細の行だけ列 0 が `\|` になり、図の主線が途切れたところに線が現れる。仕様は「cli は変更なし」としていて、詳細の行の列 0 の規則が書かれていない。古い経路にも同じ癖はあるが、列 0 が空白になる図が約半分に増える。 | 詳細の行の列 0 を、ノードの列 0 の記号(BR1.4 と同じ判定)に従わせるか、現状のまま `\|` を描くかを、仕様に明記する。従わせるなら、cli の変更(行の列 0 の記号を core から渡す)を §5a に足し、N1、N2、N5 の `-v` の全文をテストの期待に入れる。現状のままにするなら、その理由を書き、`-v` のテストで固定する。 | New |
| R-02 | Minor | aidlc/spaces/default/intents/261122-graph-layout-unify/construction/functional-design/layout-reference.py > layout() の `off` リスト、`lastkind='cross2'` | 規範の参照実装に、出力に影響しない状態 `off`(代入だけで、読まれない)と `offm` が残っている。Rust に移す人が、古い経路の「主線につながらない」印を残す手がかりに見える。BR1.2(古い経路の印をなくす)と食い違う。 | `off`、`offm` を削って、正解データとの一致(`bad 0`)を取り直す。 | New |
| R-03 | Minor | aidlc/spaces/default/intents/261122-graph-layout-unify/construction/functional-design/functional-spec.md > §2 WF1、§1 の表の 4 行目、§5a | 判定の文が不正確。表の 4 行目は「次のジョブへの辺も飛び越しもないジョブ」と書くが、実コードは、飛び越しが 1 本で次への辺がなく「両方の飛び越し」でもないジョブ(成功だけの飛び越し)も辺の一覧にする。また、実行されうるジョブの計算に `has_next`(ジョブごとの次への辺)が要るが、`SkipPlan` の説明に書かれていない(`off_main` を消すなら、代わりに持つものを書く必要がある)。 | WF1 を、ジョブごとの述語(辺の数、次への辺、飛び越しの数、種類)で書く。`skip_targets` の条件 1 をそのまま使い、条件 2、3 と `is_ordinary_skip_graph` を消すと明記する。`SkipPlan` が持つもの(`targets`、`has_next`、種類)を書く。 | New |
| R-04 | Minor | aidlc/spaces/default/intents/261122-graph-layout-unify/construction/functional-design/functional-spec.md > §6 テストの方針 | 変更するテストの一覧が足りない。(1) `crates/core/tests/graph_layout_oracle.rs` の parse は `a>d` と `b>d,f` だけを読み、新しい正解データの `>>`(両方の飛び越し)を読めない。(2) `graph_perf.rs` の 4 つの性能テストは通常の飛び越しだけで、「両方の飛び越し」と分岐の入れ子の 1,000 ジョブの性能テストが無い(仕様は新規に書くとしているが、既存のファイルの変更として挙がっていない)。(3) `graph_types.rs` の 84 行目、`nfr_source.rs`、`crates/cli/src/graph.rs`、`graph_common` も、削除する `lane_layout` などの名前の検索に当たる。(4) traceability.json の SM1 は BR1.2 だけ、SM3 は BR2.1 だけだが、spec §7 は BR1.1、BR1.2、BR1.3、BR2.1 と書く。 | テストの一覧に上の 4 点を足す。parse に `>>` を足す。traceability.json と spec §7 を一致させる。 | New |
| R-05 | Minor | aidlc/spaces/default/intents/261122-graph-layout-unify/construction/functional-design/layout-oracle-offmain.md(394 KB、ジョブ 3〜5 個) | 正解データは参照実装から作った(git の出力ではない)ので、検証できるのは「Rust への移し間違い」であり、規則の正しさではない。ジョブ 6 個以上の「両方の飛び越し」と分岐(3 重の入れ子、複数の重複の寄せ)は、固定の正解データに入らない。サイズは許容範囲(既存の 5 ファイル計 563 KB に近い)。 | 現状のサイズは受け入れてよい。ジョブ 6〜7 個の、固定の乱数で選んだ数百例(約 50 KB)を足すか、不変条件(通常の飛び越しだけなら git の正解データと同じ、全例の最後が `o @end`)のテストを足すことを検討する。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| python3 layout-reference.py layout-oracle-offmain.md layout-oracle-branch-v2.md | 2748 bad 0、871 bad 0 | 申告どおり、全例が一致する。 |
| 同じ参照実装を、既存の git の正解データ 4 ファイル(537、1,517、940、487 例)に当てた | すべて bad 0 | 通常の飛び越しだけのグラフでは、全ジョブが先頭から次への辺でたどれるので列 0 は常に `\|` になり、git の出力は変わらない。 |
| 前回の参照実装(graph-mixed-shapes)と新しい参照実装を、通常の飛び越しだけのランダム 30,000 例(ジョブ 6〜14 個)で比べた | 差 0 | 大きい形でも、通常の飛び越しだけのグラフの出力は変わらない。 |
| 新しい参照実装を、「両方の飛び越し」と分岐を含むランダム 20,000 例(ジョブ 6〜14 個)で実行した | 例外なし、全例の最後が `o @end` | ジョブ 6 個以上でも、参照実装は破綻しない(正解データとの一致までは確かめていない)。 |
| ジョブ 3〜5 個の、「両方の飛び越し」か分岐を含む形の数え上げ | 12 + 156 + 2,580 = 2,748 | offmain の件数と一致し、取りこぼしがない。 |
| 参照実装と現在の `ordinary_layout`/`settle_duplicates`/`find_duplicate_pair`/`move_rows` の比較(読み合わせ) | 同じ構造。差は、列 0 の記号の引数、分岐の線 2 本の開き方、到着側の設定だけ | Rust への移しは実装可能。`move_rows`、`fork_row`、`join_lane_row` が列 0 の記号を引数で受ければよい。参照実装の `settle` は `mainok(i-1)` を使うので、Rust の `settle_duplicates` にも列 0 の記号を渡す必要がある。 |

### Summary

規則は実行できる参照実装に固定されていて、全正解データ(git の 4 ファイルと新しい 2 ファイル)と一致し、1 つの仕組みへの統合と、git の出力が変わらないという主張は、私の追加の照合でも成り立つ。実装の前に決めるべき主な点は、`-v` の詳細の行の列 0 を新しい規則に合わせるかどうか(Major 1 件)で、残りは参照実装の掃除、判定の文の正確さ、テストの一覧の補足(Minor)。複雑さは、ジョブ数 1,000 でも、ジョブごとに線の数に比例した処理なので問題ない(既存の `find_duplicate_pair` は線の数に比例)。
