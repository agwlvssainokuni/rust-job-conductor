# エンティティ(Entities): graph の text 表示の改善 その 3

新しい永続のデータはない。既存の図の型(`crates/core/src/graph/mod.rs`)をそのまま使い、レーンの配置(core の `graph/layout.rs`)の判定と行の作り方を広げる。`LaneGlyph` には、横線 `_` を足す(質問 Q1: A。git の出力が `_` を使うため)。

```yaml
entities:
  - name: LaneGlyph
    description: 既存の列挙型(`Empty`、`Node`、`EndNode`、`Line`、`Fork`、`Join`)に、横線を足す。
    allowed_values:
      - Empty
      - Node
      - EndNode
      - Line
      - Fork
      - Join
      - Underscore   # 新規。横線 `_`(3 本以上の線が続けて交差する行で使う)
    constraints:
      - cli の `glyph_char` は、Underscore を `_` にする(すべて ASCII のまま)
  - name: OpenLanes
    description: 配置の途中で持つ、開いている線の並び(core の内部)。内側(列 1)が先頭。
    attributes:
      - name: lanes
        logical_type: list of 行き先の節の位置(内側から外側の順)。同じ行き先は 2 つ持たない(交差の範囲)
    constraints:
      - 新しい線は、先頭(列 1)に入れる(既存の線は、1 つ外側へずれる)
      - 行き先のノードが来たとき、その線の位置 k(0 が内側)で、合流の行の形が決まる
```

要約: 新しい型は、`LaneGlyph::Underscore` だけ。判定の結果の型は、変えない。
