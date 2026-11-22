# ビジネスルール(Rules): graph の text 表示の改善 その 4

内側を先頭とした、開いている線の並びを `lanes`、その数を `m` とする。列は、主線が 0、`lanes[j]` が列 `j+1`。列の文字の位置は `2 × 列`。

```yaml
rules:
  - id: BR1.1
    statement: 「通常の飛び越しだけのグラフ」を、レーンの図にする。範囲の条件は、グラフ全体で 1 つ: すべてのジョブが次のジョブへの辺を持ち、各ジョブの飛び越しは高々 1 本で、「両方の飛び越し」と分岐がない。飛び越しの行き先は、重なってよく(同じ行き先)、線は交差してよい。この条件を満たすグラフは、行き先の重なりと交差の有無によらず、レーンの図にする。
    category: policy
    applies_to: 「レーンの図で描ける形」の判定(core の `graph/layout.rs`)
    trigger: 通常の飛び越しだけのグラフ
    logic: 範囲の条件を満たすなら、レーンの図(前回までの「行き先がすべて違う」の条件を外す)
    violation: 辺の一覧に落ちる(従来)
    source: intent-statement.md、functional-design-questions.md Q1: A
  - id: BR1.2
    statement: 配置は、「線の並び」と「寄せ」の 1 つの仕組みで決める。ジョブの飛び越しは、新しい線を `lanes` の先頭に入れる(先頭の線と行き先が同じなら、合流の行 `|\|` で、新しい線を作らない)。同じ行き先の線が 2 本以上になったら、「寄せ」で 1 本にする。
    category: calculation
    applies_to: 配置の全体
    trigger: 飛び越しを持つジョブ、行き先のノード、同じ行き先の線の重複
    logic: functional-spec.md の WF2(参照実装 `layout-reference.py`)のとおり
    violation: 正解データと不一致
    source: layout-oracle-mixed.md、layout-reference.py
  - id: BR1.3
    statement: 「寄せ」`MOVE(c, d)`(列 `c` の線を、列 `d`(0 は主線)へ寄せる)の行を、次のとおり決める。`c − d = 1` のときは 1 行: `| ` を `c − 1` 回、続けて `|/`、続けて ` /` を `m − c` 回。`c − d ≥ 2` のときは 2 行。行 1: 列 0 〜 `c − 1` を `|` で、間の空白のうち、右の `c − d − 2` 個を `_` に、続けて `/`、続けて ` /` を `m − c` 回。行 2: `| ` を `d` 回、続けて `|/`、続けて、残りの線(寄せる線を除く)を、`d` 番目から、`| ` で(重複の 2 本目以降は `/ `。`d ≥ 1` のとき、寄せた線より外側の線は、次のノードへ到着する線から `/ `)。各行の末尾の空白は取る。
    category: calculation
    applies_to: 到着の行(`d = 0`)と、同じ行き先の寄せの行(`d ≥ 1`)
    trigger: BR1.2 の「寄せ」
    logic: 上のとおり
    violation: 正解データと不一致
    source: layout-oracle-mixed.md、layout-reference.py、前回の交差の正解データ
  - id: BR1.4
    statement: 寄せる順序。行き先のノードが来たとき、(1) 到着する線(行き先 = そのノード)をまたがない、同じ行き先の重複の組を、先に寄せる。(2) 到着する線の位置 `k` の 1 本を、主線へ寄せる(同じ行き先の別の線は、その後、静かに取り除く)。(3) 残りの重複の組を寄せる。到着がなくても、重複の組があれば、寄せる。重複の組は、内側の線から順に(前の線 `a`、後ろの線 `b` の組で、`a` の小さい順、同じなら `b` の小さい順)寄せる。
    category: calculation
    applies_to: 配置の途中
    trigger: 各ノードの手前
    logic: 上のとおり
    violation: 正解データと不一致
    source: layout-oracle-mixed.md、layout-reference.py
  - id: BR1.5
    statement: ノードの行の、既存の線の記号。(a) そのノードが新しい線を作り、直前の行が分岐の行なら `\`。(b) 直前の寄せが 1 行の合流(`c − d = 1`)で、そのノードが新しい線を作らず、次のノードも線の行き先(位置 `q`)なら、位置 `q` 以降を `/`、内側を `|`。(c) 直前の寄せが 2 行の、同じ行き先の寄せ(`d ≥ 1`)で、そのノードが飛び越しを持たず、次のノードが線の行き先で `q = 0` なら、位置 0 だけ `/`、ほかは `|`。(d) それ以外は `|`(到着の 2 行の寄せ(`d = 0`、`k ≥ 1`)の直後も `|`)。
    category: calculation
    applies_to: ノードの行
    trigger: 常に
    logic: 上の優先順
    violation: 正解データと不一致
    source: layout-oracle-mixed.md、layout-reference.py
  - id: BR2.1
    statement: 次のグラフは、従来どおり辺の一覧にする(後回し。意図の把握 Q2: A)。グラフのどこかに、「両方の飛び越し」、分岐があるもの(交差や同じ行き先が混ざる、混ざらないにかかわらず、今の判定のまま)。
    category: constraint
    applies_to: 「レーンの図で描ける形」の判定
    trigger: 範囲の条件を満たさないグラフ
    logic: 今の処理のまま(前回までの出力を変えない)
    violation: 描き方が決まっていない形を図にする
    source: intent-statement.md(範囲)
  - id: BR3.1
    statement: 通常の飛び越しだけのグラフで、前回までに図になった形(行き先がすべて違う、交差の有無。同じ行き先で交差しない形)の出力は、1 文字も変わらない。「両方の飛び越し」、分岐を含むグラフと、辺の一覧の出力も、変わらない。`--format mermaid`、`--state`、`-v` の仕様も変えない。既存の正解データ(537、1,517、871、4,757 例)が、そのまま通る。
    category: constraint
    applies_to: graph の出力全体
    trigger: 常に
    logic: 既存の正解データと、書き換え対象でない既存のテストが、そのまま通る
    violation: 回帰
    source: intent-statement.md(Q1: B)
  - id: BR4.1
    statement: 交差と同じ行き先が混ざる形は、`layout-oracle-mixed.md`(ジョブ 4〜6 個の 487 例)と、1 文字も違わず一致する。ジョブ 7 個の 3,861 例は、コード生成のときに、手元で 1 回だけ照合する(質問 Q2: A)。
    category: validation
    applies_to: テスト
    trigger: テストの実行
    logic: 正解データを読み込み、形ごとに配置し、行を比べる
    violation: テストの失敗
    source: functional-design-questions.md Q2: A
  - id: BR4.2
    statement: README と仕様書(`docs/spec/rust`、`docs/spec/language-agnostic`)の「辺の一覧に切り替わる形」と配置の記述を、実装に合わせて直す。
    category: policy
    applies_to: 文書
    trigger: 実装の完了
    logic: BR1.1〜BR3.1 を反映する
    violation: 文書と実装の食い違い
    source: intent-statement.md(成功の指標)
  - id: BR5.1
    statement: 規則は、`git log --graph` の出力だけの観察から立てる。git のソース(GPL v2)は読まない。出力と一字一句一致しない形は、辺の一覧のままにする。
    category: policy
    applies_to: 作業の進め方
    trigger: 規則の導出
    logic: 出力の観察と、正解データとの照合だけを根拠にする
    violation: GPL v2 のコードの派生物になる恐れ
    source: 前回までの intent の確認(ユーザー。2026-11-22)、intent-statement.md
```

要約:

| ID | 内容 |
|----|------|
| BR1.1 | 通常の飛び越しだけのグラフは、同じ行き先と交差の有無によらず、図にする |
| BR1.2〜BR1.5 | 線の並びと寄せの 1 つの仕組み(寄せの行、順序、ノードの行の記号) |
| BR2.1、BR3.1 | 両方の飛び越し・分岐を含むグラフと、前回までの出力は変えない |
| BR4.1、BR4.2 | 正解データの一致。README と仕様書の更新 |
| BR5.1 | git のソースは読まない(観察だけ) |
