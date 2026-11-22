# ビジネスルール(Rules): graph の text 表示の改善 その 3

内側の線を先頭とした、開いている線の並びを `lanes`、その数を `m`、行き先のノードが来たとき、その線の位置(0 が内側)を `k` とする。列の位置(0 始まりの文字の位置)は、主線が 0、線 `j`(0 始まり)が `2(j+1)`。

```yaml
rules:
  - id: BR1.1
    statement: 「交差する形」を、レーンの図にする。範囲の条件は、**グラフ全体**で 1 つ(正解データと試作が覆う範囲): すべてのジョブが、次のジョブへの辺を持ち、各ジョブの飛び越しは高々 1 本、「両方の飛び越し」と分岐がなく、飛び越しの行き先が、終点も含めて、どの 2 本でも違う。この条件を満たすグラフは、交差の有無によらず、レーンの図にする。
    category: policy
    applies_to: 「レーンの図で描ける形」の判定(core の `graph/layout.rs`)
    trigger: 通常の飛び越しの線が交差する
    logic: 通常の飛び越しだけで、行き先が重ならない形なら、交差の有無によらず、レーンの図
    violation: 辺の一覧に落ちる(従来)
    source: intent-statement.md、functional-design-questions.md Q1: A
  - id: BR1.2
    statement: 新しい線は、列 1 に入れ、既存の線は 1 つ外側へずれる(前回までと同じ)。行き先のノードが来たら、その線を取り除く(残りの順序は変えない)。
    category: calculation
    applies_to: 線の並びの更新
    trigger: 飛び越しを持つジョブ、行き先のノード
    logic: lanes の先頭に追加。到着で、k 番目を削除
    violation: 正解データと不一致
    source: layout-oracle-cross.md
  - id: BR1.3
    statement: 行き先のノードが来たとき、到着する線が最も内側(k = 0)なら、従来の合流の行 `|/` + ` /` を (m − 1) 回、を 1 行置く(前回までと同じ)。
    category: calculation
    applies_to: 合流の行
    trigger: k = 0
    logic: 1 行
    violation: 正解データと不一致
    source: layout-oracle-cross.md(既存の 537 例、1,517 例も同じ)
  - id: BR1.4
    statement: 到着する線が最も内側でない(k ≥ 1)とき、ノードの行の前に、2 行(行 A、行 B)を置く。行 A: 列 0 に `|`。続けて、j = 0 〜 k−1 について、列 2(j+1) に `|`、その右に、j = k−1 のときだけ `/`、それ以外は `_`。続けて、到着した線より外側の線(m − 1 − k 本)について、` /`(空白と `/`)を 1 本ずつ。行 B: `|/` に続けて、残りの線(m − 1 本)について `|` を、空白 1 つおきに。各行の末尾の空白は取る。
    category: calculation
    applies_to: 交差の行
    trigger: k ≥ 1
    logic: 行 A と行 B の 2 行(実際の文字は functional-spec.md の §4 の例)
    violation: 正解データと不一致
    source: layout-oracle-cross.md(940 例と、ジョブ 8 個の 3,817 例)
  - id: BR1.5
    statement: ノードの行の、既存の線の記号。次の優先順で決める。(a) そのノードが新しい線を作り、直前の行が分岐の行(新しい線を作るジョブのあと)なら `\`。(b) そのノードが**新しい線を作らず**、そのノードの前の合流の行が k = 0 で、次のノードも線の行き先(その線の位置 q)なら、位置 q 以降の線を `/`、それより内側を `|`。(c) それ以外は `|`(新しい線を作るノードで、直前が分岐の行でないときも `|`)。交差の行(k ≥ 1)のあとのノードの行は、(b) を使わず `|`。この (b) の条件(新しい線を作らない)を落とすと、交差の 4,757 例のうち 484 例が不一致になる(レビューの確認)。
    category: calculation
    applies_to: ノードの行
    trigger: 常に
    logic: 上のとおり(前回までの `/` の規則の、複数の線への一般化)
    violation: 正解データと不一致
    source: layout-oracle-cross.md
  - id: BR2.1
    statement: 次のグラフは、従来どおり辺の一覧にする(後回し。質問 Q2: A)。線が交差し、かつ、グラフ全体のどこかに「両方の飛び越し」、分岐、同じ行き先を含むもの(離れた別のジョブにあっても同じ。BR1.1 の範囲の条件を満たさないため)。混ざった例を、辺の一覧のテストにする。
    category: constraint
    applies_to: 「レーンの図で描ける形」の判定
    trigger: 交差と、これらの形の組み合わせ
    logic: 範囲の外は辺の一覧
    violation: 描き方が決まっていない形を図にする
    source: functional-design-questions.md Q2: A
  - id: BR3.1
    statement: BR1.1 に当たらない形の出力(通常の飛び越しで交差しない形、同じ行き先、両方の飛び越し、分岐、辺の一覧)は、1 文字も変わらない。`--format mermaid`、`--state`、`-v` の仕様も変えない。既存の 3 つの正解データ(537 例、1,517 例、871 例)が、そのまま通る。
    category: constraint
    applies_to: graph の出力全体
    trigger: 常に
    logic: 既存の正解データと、書き換え対象でない既存のテストが、そのまま通る
    violation: 回帰
    source: intent-statement.md(質問 Q1: B)
  - id: BR4.1
    statement: 交差する形は、`layout-oracle-cross.md`(ジョブ 3〜7 個の 940 例)と、1 文字も違わず一致する。ジョブ 8 個の 3,817 例は、コード生成のときに、手元で 1 回だけ照合する(質問 Q2: A)。
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
    source: functional-design-questions.md Q1(ユーザーの確認 2026-11-22)
```

要約:

| ID | 内容 |
|----|------|
| BR1.1〜BR1.5 | 交差する形を図にする。線の並び、合流の行(k = 0 と k ≥ 1)、ノードの行の記号 |
| BR2.1 | 両方の飛び越し・分岐・同じ行き先を含む交差は辺の一覧 |
| BR3.1 | ほかの形は変えない |
| BR4.1、BR4.2 | 正解データの一致。README と仕様書の更新 |
| BR5.1 | git のソースは読まない(観察だけ) |
