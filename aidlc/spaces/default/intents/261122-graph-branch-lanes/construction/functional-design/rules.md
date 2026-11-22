# ビジネスルール(Rules): graph の text 表示の改善 その 2

```yaml
rules:
  - id: BR1.1
    statement: 次のジョブへの辺を持たず、記述順の次でない、異なる 2 つの行き先(近い順に t1、t2)への辺を持つジョブを「分岐」とする(成功と警告のどちらが近いかは問わない)。分岐のジョブは、新しい線を 2 本、同時に作る。近い行き先 t1 の線が内側(列 1)、遠い行き先 t2 の線が外側(列 2)。
    category: policy
    applies_to: レーンの配置(core の `graph/layout.rs`)
    trigger: ジョブから出る辺が 2 本で、どちらも次のジョブへの辺でなく、行き先が異なる
    logic: SkipKind = Branch。開いている線に、外側(t2)、内側(t1)の順に積む
    violation: 辺の一覧に落ちる(従来)
    source: intent-statement.md(この形をレーンの図で描く)、functional-design-questions.md Q1: A
  - id: BR1.2
    statement: 分岐のジョブの、ノードの行のラベルの位置は、2 × (1 + 開いている線の数 + 2)。
    category: calculation
    applies_to: ノードの行
    trigger: BR1.1 が成り立つ
    logic: width = 2 × (1 + 既存の線の数 + 2)
    violation: ラベルの位置がずれる
    source: layout-oracle-branch.md
  - id: BR1.3
    statement: 分岐のジョブの、ノードの行の直後に、分岐の行を 1 つ置く。分岐の行は、列 0 を空にし、列 1 から、(既存の線の数 + 2)個の `\` を、空白 1 つおきに並べる。
    category: calculation
    applies_to: 分岐の行
    trigger: BR1.1 が成り立つ
    logic: 分岐の行 = 空白、`\`、(空白、`\`)を (既存の線の数 + 1) 回
    violation: 正解データと不一致
    source: functional-design-questions.md Q1: A、layout-oracle-branch.md
  - id: BR1.4
    statement: 行き先での合流の行は、その線を開いたジョブが、すべて「主線につながらない」ジョブ(分岐、または前回の「両方の飛び越し」)のときだけ、列 0 を空にする(前回の BR と同じ)。通常の飛び越しが 1 つでも同じ線にあるなら、従来どおり `|/`。
    category: calculation
    applies_to: 行き先での合流の行
    trigger: 線の行き先のノード
    logic: 列 0 = 空白 if 線の all_off_main else `|`
    violation: 正解データと不一致
    source: layout-oracle-branch.md
  - id: BR1.5
    statement: 分岐の直後のノードの行で、既存の線の記号は、そのノードが分岐、または新しい線を作るジョブなら `\`、それ以外は `|`(前回までの `after_fork` の規則と同じ)。
    category: calculation
    applies_to: 分岐の行の直後のノードの行
    trigger: 分岐のジョブの次のノード
    logic: after_fork は、新しい線を作るジョブ(NewLane、Branch)のあとだけ真
    violation: 正解データと不一致
    source: layout-oracle-branch.md
  - id: BR2.1
    statement: 分岐の線は、交差してはならず、行き先が同じ(既存の線との合流を含む)ものを含んではならない(質問 Q2: A)。分岐の 2 つの行き先は、どちらも、最も内側の開いている線の行き先より、近くなければならない(`t2 < top`)。この条件を満たさない形は、従来どおり辺の一覧にする。
    category: constraint
    applies_to: 「レーンの図で描ける形」の判定
    trigger: 分岐のジョブがある
    logic: IF t2 >= top(最も内側の開いている線の行き先)THEN 辺の一覧。分岐の線と他の線の行き先が同じなら、辺の一覧
    violation: 描けない形を図にしてしまう
    source: functional-design-questions.md Q2: A
  - id: BR2.2
    statement: 分岐が開いた線に、後のジョブが同じ行き先で合流する形(例: `a` が `c` と `e` へ分岐し、`d` が `e` へ飛ぶ。`t == top` で、積み上げの上の線が分岐の開いた線)は、辺の一覧にする。
    category: constraint
    applies_to: 「レーンの図で描ける形」の判定
    trigger: 通常のジョブの合流が、分岐の開いた線に対して起きる
    logic: IF 積み上げの上の線が「分岐が開いた」印を持つ AND t == top THEN 辺の一覧
    violation: 正解データにない形(合流の行の列 0 の扱いなど)を、未検証のまま図にしてしまう
    source: functional-design-questions.md Q2: A(同じ行き先を含む分岐は一覧)、設計のレビューの Major
  - id: BR3.1
    statement: 分岐のジョブ以外の出力(通常の飛び越し、同じ行き先、両方の飛び越し、交差、1 つのジョブが 3 本以上の辺を持つ形など)は、1 文字も変わらない。`--format mermaid`、`--state` の重ね表示、`-v` の書式も変えない。この規則が逆にする「図にできるか」の表明は、新しい期待に書き換える(functional-spec.md の §6)。
    category: constraint
    applies_to: graph の出力全体
    trigger: 常に
    logic: 既存の正解データ(537 例と 1,517 例)と、書き換え対象でない既存のテストが、そのまま通る
    violation: 回帰
    source: intent-statement.md(ほかの形の出力は変わらない)
  - id: BR4.1
    statement: 分岐を含む形は、layout-oracle-branch.md の全 871 例(ジョブ 3〜8 個)と、1 文字も違わず一致する(Q3: A)。
    category: validation
    applies_to: テスト
    trigger: テストの実行
    logic: 正解データを読み込み、形ごとに配置し、行を比べる
    violation: テストの失敗
    source: functional-design-questions.md Q3: A
  - id: BR4.2
    statement: README と仕様書(`docs/spec/rust`、`docs/spec/language-agnostic`)の「辺の一覧に切り替わる形」と配置の記述を、実装に合わせて直す。
    category: policy
    applies_to: 文書
    trigger: 実装の完了
    logic: BR1.1〜BR3.1 を反映する
    violation: 文書と実装の食い違い
    source: intent-statement.md(成功の指標)、functional-design-questions.md
```

要約:

| ID | 内容 |
|----|------|
| BR1.1〜BR1.5 | 分岐は新しい線を 2 本同時に作る。ラベルの位置、分岐の行、合流の行、直後の記号 |
| BR2.1 | 交差と、同じ行き先を含む分岐は、辺の一覧のまま |
| BR3.1 | ほかの形は変えない |
| BR4.1、BR4.2 | 正解データ 871 例の一致。README と仕様書の更新 |
