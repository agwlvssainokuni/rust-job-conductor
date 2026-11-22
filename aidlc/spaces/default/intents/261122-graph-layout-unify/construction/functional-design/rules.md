# ビジネスルール(Rules): graph の配置のリファクタリング

**規範は、参照実装 `layout-reference.py`。** 文と食い違うときは、参照実装が正。

```yaml
rules:
  - id: BR1.1
    statement: レーンの図にするのは、すべてのジョブが「飛び越しなし」「通常の飛び越し」「両方の飛び越し」「分岐」のどれかのグラフ。交差と同じ行き先は、どの組み合わせでも図にする。それ以外のジョブ(辺が 3 本以上など)があれば、辺の一覧。
    category: policy
    applies_to: 「レーンの図で描ける形」の判定(core の `graph/layout.rs`)
    trigger: 常に
    logic: functional-spec.md の WF1
    violation: 図にできる形が辺の一覧になる、または描けない形を図にする
    source: intent-statement.md(Q1: B、Q2: A)
  - id: BR1.2
    statement: 配置は、前回の「線の並びと寄せ」の 1 つの仕組みだけで行う。古い経路(`lane_layout` と、その特例)は、なくす。
    category: policy
    applies_to: 配置の全体
    trigger: 常に
    logic: functional-spec.md の WF2、§5a
    violation: 経路が 2 つ残る
    source: intent-statement.md(成功の指標)
  - id: BR1.3
    statement: 「両方の飛び越し」は線を 1 本、分岐は線を 2 本(近い行き先が内側)、先頭に入れる。分岐の行の `\` は (既存の線の数 + 新しい線の数) 個。通常の飛び越しと「両方の飛び越し」は、先頭の線と行き先が同じなら、合流の行 `|\|` で、線を作らない。分岐の線の行き先が既存の線と同じでも、2 本を入れ、次のノードの手前の寄せで 1 本にする。
    category: calculation
    applies_to: 線を開く行
    trigger: 「両方の飛び越し」、分岐
    logic: layout-reference.py のとおり
    violation: 正解データと不一致
    source: layout-oracle-offmain.md、layout-oracle-branch-v2.md
  - id: BR1.4
    statement: ノード `j` と `j + 1` のあいだの行(分岐の行、合流の行、寄せの行)の列 0 は、ジョブ `j` が実行されうる(先頭のジョブから辺をたどって着ける)、かつ次のジョブへの辺を持つときだけ `|`、それ以外は空白(質問 Q1: B)。
    category: calculation
    applies_to: 主線の列 0
    trigger: 常に
    logic: 実行されうるジョブの集合を、先頭のジョブからたどって求める
    violation: 正解データと不一致
    source: functional-design-questions.md Q1: B
  - id: BR2.1
    statement: 「両方の飛び越し」か分岐を含むグラフの、交差と同じ行き先の組み合わせは、図にする(今は辺の一覧)。
    category: policy
    applies_to: 判定と配置
    trigger: 組み合わせの形
    logic: BR1.1〜BR1.4 で描く
    violation: 辺の一覧に落ちる(従来)
    source: intent-statement.md(Q2: A)
  - id: BR3.1
    statement: git の出力の正解データ(537、1,517、940、487 例)と、通常の飛び越しだけのグラフの出力は、1 文字も変わらない。分岐の正解データは、BR1.4 で作り直した版(`layout-oracle-branch-v2.md`)に差し替える。`--format mermaid`、`--state`、`-v` の仕様は変えない。
    category: constraint
    applies_to: graph の出力全体
    trigger: 常に
    logic: 正解データの照合
    violation: 回帰
    source: intent-statement.md(成功の指標)、functional-design-questions.md Q1: B
  - id: BR4.1
    statement: 「両方の飛び越し」か分岐を含む形は、`layout-oracle-offmain.md`(ジョブ 3〜5 個の 2,748 例)と、`layout-oracle-branch-v2.md`(871 例)と、1 文字も違わず一致する。
    category: validation
    applies_to: テスト
    trigger: テストの実行
    logic: 正解データを読み込み、形ごとに配置し、行を比べる
    violation: テストの失敗
    source: functional-design-questions.md Q2: A
  - id: BR4.2
    statement: README と仕様書(`docs/spec/rust`、`docs/spec/language-agnostic`)の配置と「辺の一覧に切り替わる形」の記述を、1 つの仕組みに合わせて書き直す。
    category: policy
    applies_to: 文書
    trigger: 実装の完了
    logic: BR1.1〜BR3.1 を反映する
    violation: 文書と実装の食い違い
    source: intent-statement.md(Q1: B)
  - id: BR5.1
    statement: git のソース(GPL v2)は読まない。規則は、`git log --graph` の出力の観察と、ユーザーが承認した図だけから立てる。
    category: policy
    applies_to: 作業の進め方
    trigger: 常に
    logic: 出力の観察と、正解データとの照合だけを根拠にする
    violation: GPL v2 のコードの派生物になる恐れ
    source: 前回までの intent の確認(ユーザー)、intent-statement.md
```

要約:

| ID | 内容 |
|----|------|
| BR1.1、BR1.2 | 判定と、1 つの仕組み(古い経路をなくす) |
| BR1.3、BR1.4 | 両方の飛び越しと分岐の線の開き方、主線の列 0 の規則(実行されうるジョブからの辺) |
| BR2.1 | 組み合わせの形を図にする |
| BR3.1 | git の出力は変えない。分岐の正解データは作り直した版 |
| BR4.1、BR4.2 | 正解データの一致。README と仕様書の書き直し |
| BR5.1 | git のソースは読まない |
