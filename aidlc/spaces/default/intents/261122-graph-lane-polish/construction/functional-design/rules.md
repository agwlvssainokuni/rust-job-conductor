# ビジネスルール(Rules): graph の text 表示の改善

```yaml
rules:
  - id: BR1.1
    statement: 飛び越しの行き先が、いま開いている線のうち最も内側(列 1)の行き先と同じなら、新しい線を作らず、その線に合流する。
    category: policy
    applies_to: レーンの配置(core の `graph/layout.rs`)
    trigger: ジョブに飛び越しの辺が 1 つあり、その行き先が最も内側の線の行き先と等しい
    logic: IF 行き先 == 最も内側の線の行き先 THEN SkipKind = JoinLane(線の数は増えない)
    violation: 新しい線を作ってしまうと、git log --graph と違う図になる(正解データと不一致)
    source: intent-statement.md(同じ行き先は、既存の線に合流させる)
  - id: BR1.2
    statement: 合流するジョブの、ノードの行の直後に、合流の行を置く。合流の行は、主線の `|`、`\`、最も内側の線の `|` の順に並べ、外側の線(列 2 以降)は、空白 1 つおきに `|` を続ける。
    category: calculation
    applies_to: 合流の行
    trigger: BR1.1 が成り立つ
    logic: 合流の行 = `|\|` + 外側の線の数だけ ` |`
    violation: 正解データと不一致
    source: intent-statement.md、layout-oracle-same-target.md
  - id: BR1.3
    statement: 合流するジョブの、ノードの行のラベルの位置は、2 × (1 + 開いている線の数)。新しい線を作らないので、線の数が変わらない。
    category: calculation
    applies_to: ノードの行
    trigger: BR1.1 が成り立つ
    logic: width = 2 × (1 + 開いている線の数)
    violation: ラベルの位置がずれる
    source: layout-oracle-same-target.md
  - id: BR1.4
    statement: 合流の行は、線が右へずれる分岐の行ではない。合流の行の直後のノードの行で、既存の線は `|` のままにする(`\` にしない)。
    category: calculation
    applies_to: 合流の行の直後のノードの行
    trigger: BR1.1 が成り立つジョブの次のジョブ
    logic: 直前が合流の行なら、既存の線の記号は `|`
    violation: 正解データと不一致
    source: layout-oracle-same-target.md
  - id: BR1.5
    statement: 飛び越しの行き先が、最も内側の線の行き先より遠いときは、交差とみなし、辺の一覧にする(行き先が、最も内側でない線の行き先と同じときも、これにあたる)。最も内側の線と等しいときは、交差ではない。
    category: constraint
    applies_to: 「レーンの図で描ける形」の判定
    trigger: 飛び越しの辺が、開いている線と重なる
    logic: IF 行き先 > 最も内側の線の行き先 THEN 辺の一覧。IF 行き先 == 最も内側の線の行き先 THEN 合流(BR1.1)
    violation: 描けない形を図にしてしまう
    source: intent-statement.md(交差は対象外)
  - id: BR2.1
    statement: そのジョブから出る辺がちょうど 1 つで、その辺が成功と警告の両方で、行き先が記述順の次のジョブでないとき(「両方の飛び越し」)、そのジョブは線を作らず、ノードの行だけを描く。他のジョブの線の描き方にも、交差の判定にも影響しない。
    category: policy
    applies_to: レーンの配置、「レーンの図で描ける形」の判定
    trigger: 「両方の飛び越し」のジョブ
    logic: IF 辺が 1 つ AND kinds が成功と警告 AND 行き先 != 次のジョブ THEN SkipKind = NoLane(飛び越し先は無視する)
    violation: 辺の一覧に落ちてしまう(従来の挙動)
    source: intent-statement.md(両方の飛び越しは、上下の `|` なしで `*` のみ)
  - id: BR2.2
    statement: 「両方の飛び越し」は、図には印を付けない(質問 Q1: A)。飛び越しの事実は、`-v` の注記(既存の書式。「成功・警告 → c(b を飛び越す)」)で示す。
    category: policy
    applies_to: text の整形
    trigger: `-v`
    logic: 既存の飛び越しの注記を、そのまま使う
    violation: 注記が消える、または図に余計な印が付く
    source: functional-design-questions.md Q1
  - id: BR3.1
    statement: 辺の一覧に切り替わるのは、(a) 飛び越しの線が交差する、(b) 1 つのジョブが、異なる行き先への飛び越しを 2 つ持つ、の 2 つの形だけにする。先頭の注記の英語の文言は、辺の一覧に落ちるすべての形を含む言い方に改める(「note: the skip edges cross or form a shape the lane diagram cannot draw, so it is replaced by a list of edges.」。「行き先が同じ」を含めない)。
    category: policy
    applies_to: 辺の一覧への切り替えと、その注記
    trigger: レーンの図で描けない形
    logic: BR1.5 または (b) のときだけ辺の一覧
    violation: 注記が実態と合わない
    source: intent-statement.md
  - id: BR3.2
    statement: BR1.1〜BR2.1 に当たらない形の出力は、1 文字も変わらない(`--format mermaid`、`--state` の重ね表示、`-v` の書式も変えない)。この規則が逆にする「図にできるか」の表明(`graph_drawable.rs` の 3 つ)は、新しい期待に書き換える(functional-spec.md の §8)。
    category: constraint
    applies_to: graph の出力全体
    trigger: 常に
    logic: 既存の正解データ(537 例)と、書き換え対象でない既存のテストが、そのまま通る
    violation: 回帰
    source: intent-statement.md(ほかの形の出力は変わらない。Q2)
  - id: BR4.1
    statement: 同じ行き先を含む形は、layout-oracle-same-target.md の全 1,517 例(ジョブ 2〜8 個)と、1 文字も違わず一致する(Q2: A)。
    category: validation
    applies_to: テスト
    trigger: テストの実行
    logic: 正解データを読み込み、形ごとに配置し、行を比べる
    violation: テストの失敗
    source: functional-design-questions.md Q2
  - id: BR4.2
    statement: README と仕様書(`docs/spec/rust`、`docs/spec/language-agnostic`)の「辺の一覧に切り替わる形」と配置の記述を、実装に合わせて直す。
    category: policy
    applies_to: 文書
    trigger: 実装の完了
    logic: BR1.1〜BR3.1 を反映する
    violation: 文書と実装の食い違い
    source: intent-statement.md(成功の指標)
```

要約:

| ID | 内容 |
|----|------|
| BR1.1〜BR1.5 | 同じ行き先は、既存の線に合流する(`|\|`)。線の数、ラベルの位置、直後の行、交差の判定 |
| BR2.1、BR2.2 | 両方の飛び越しは、線なしの `*` の行。注記は `-v` だけ |
| BR3.1、BR3.2 | 辺の一覧は交差と、異なる行き先を 2 つ持つ形だけ。ほかの形は変えない |
| BR4.1、BR4.2 | 正解データの全例の一致。README と仕様書の更新 |
