# エンティティ(Entities): graph の text 表示の改善

この変更は、新しい永続のデータを持たない。既存の図の型(`FlowGraph`、`FlowEdge`、`LaneLayout`、`LaneRow`、`LaneGlyph`、`GraphLayout`。`crates/core/src/graph/mod.rs`)をそのまま使い、レーンの配置の途中結果(`SkipPlan`)の意味を広げる。

```yaml
entities:
  - name: SkipPlan
    description: 配置の途中結果。ジョブごとに、レーンの図で、飛び越しの線をどう描くかを決める(core の内部。公開しない)。
    attributes:
      - name: per_job
        logical_type: list of SkipKind (ジョブの記述順。長さ = ジョブの数)
        required: true
    allowed_values:
      SkipKind:
        - NoLane      # 線を作らない。次のジョブへ進むだけのジョブと、「両方の飛び越し」のジョブ
        - NewLane     # 新しい線を作り、行き先は、それより内側の線の行き先より近い(従来)
        - JoinLane    # 最も内側の線と行き先が同じ。新しい線を作らず、その線に合流する(新規)
    constraints:
      - 「両方の飛び越し」のジョブは NoLane。他のジョブの線にも、交差の判定にも影響しない
      - JoinLane の行き先は、その時点の最も内側の線の行き先と等しい
  - name: FlowEdge
    description: 既存(変更なし)。成功と警告の行き先が同じなら 1 つにまとめ、kinds に両方を持つ。
    relationships:
      - SkipPlan は FlowEdge の組(kinds の数、行き先)から導く
  - name: LaneRow
    description: 既存(変更なし)。合流の行(`|\|`)も、既存の LaneGlyph(Line、Fork)の並びで表す。新しい種類の記号は足さない。
```

要約: 新しい型は足さない。判定の結果(`SkipPlan`)だけが、「新しい線」「合流」「線なし」の 3 種になる。
