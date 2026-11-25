# エンティティ: u4-graph-dot(`graph --format dot`)

出典: ユニットの定義(`unit-of-work.md` の U4)、要件とユニットの対応(`unit-of-work-story-map.md`。FR4.1〜FR4.4、FR8.1 がこのユニット)、要件(`requirements.md` の FR4.1〜FR4.4)、コンポーネントの一覧(`components.md` の CliArgs、CliReport)、契約(`contract-summary.md` の C5)、機能の設計の質問(`functional-design-questions.md` の Q1、Q2)。

このユニットは保存するデータを持たない。新しく足すのは、コマンドラインの値 1 つと、出力の DOT の文書の形だけである。入力は今の `FlowGraph`(core の図の構造と状態の重ね合わせ)をそのまま使う。

```yaml
entities:
  - name: GraphFormat
    kind: コマンドラインの値(既存の列挙。CliArgs)
    description: "`graph --format` の値。"
    attributes:
      - name: value
        type: 列挙
        required: true
        allowed_values: [text, mermaid, dot]
        default: text
        constraints: dot を足す。それ以外の値は今と同じ使い方の誤り

  - name: DotDocument
    kind: 出力(CliReport が作る文字列。保存しない)
    description: Graphviz の DOT 言語の有向グラフ 1 つ。
    attributes:
      - name: header
        type: 固定の文字列
        required: true
        constraints: "`digraph conductor {` で始め `}` で終わる。名前は固定で、入力の値を入れない"
      - name: graph_attributes
        type: 固定の属性
        required: true
        constraints: 上から下へ並べる(`rankdir=TB`)
      - name: nodes
        type: DotNode の並び
        required: true
      - name: edges
        type: DotEdge の並び
        required: true

  - name: DotNode
    kind: 出力の一部
    description: ジョブ 1 つ、または終点。
    attributes:
      - name: id
        type: 文字列
        required: true
        unique: true
        constraints: "ジョブは記述順の連番 `n0`、`n1`…、終点は `nend`(mermaid と同じ。ジョブ ID を ID に使わない)"
      - name: label
        type: DOT の文字列
        required: true
        constraints: >
          ジョブは「状態の記号 + ジョブ ID」(記号は `--state` のときだけ。text・mermaid と同じ記号)。
          終点は `@end`。escape は BR3.1
      - name: shape
        type: 列挙
        required: true
        allowed_values: [box, oval]
        constraints: ジョブは box、終点は oval
      - name: penwidth
        type: 整数
        required: false
        constraints: 再開位置のジョブだけ 3(mermaid の太い線と同じ)

  - name: DotEdge
    kind: 出力の一部
    description: 流れの辺 1 つ。
    attributes:
      - name: from
        type: DotNode の id
        required: true
      - name: to
        type: DotNode の id
        required: true
      - name: label
        type: DOT の文字列
        required: false
        constraints: 主線の辺(次のジョブへ、成功と警告の両方)はラベルなし。ほかは種類のラベル(mermaid と同じ語、言語に従う)
```

## まとめ

| エンティティ | 種類 | 内容 |
|---|---|---|
| GraphFormat | コマンドラインの値 | `dot` を足す |
| DotDocument / DotNode / DotEdge | 出力 | mermaid と同じ節・辺・ラベル・再開位置を、DOT の記法で書く |

入力の `FlowGraph`、`GraphNode`、`FlowEdge`、状態の重ね合わせは変えない。
