# エンティティ(Entities): graph の text 表示の改善 その 2

新しい永続のデータはない。既存の図の型(`crates/core/src/graph/mod.rs`)を使い、配置の途中結果(`SkipPlan`)に「分岐」の種類を足す(core の内部。公開しない)。

```yaml
entities:
  - name: SkipPlan
    description: 配置の途中結果。ジョブごとに、レーンの図で、飛び越しの線をどう描くかを決める。
    attributes:
      - name: per_job
        logical_type: list of SkipKind(ジョブの記述順。長さ = ジョブの数)
        required: true
    allowed_values:
      SkipKind:
        - None          # 線を作らない(次のジョブへ進むだけ)
        - NewLane       # 新しい線を 1 本作る(通常の飛び越し。前回までと同じ)
        - JoinLane      # 最も内側の線に合流する(同じ行き先。前回)
        - Branch        # 新しい線を 2 本、同時に作る(分岐。今回)。行き先は 2 つ、近いほうが内側
    constraints:
      - Branch のジョブは、次のジョブへの辺を持たない(辺は、次でない 2 つの行き先への 2 本だけ)
      - Branch の 2 つの行き先は、異なる(近い順に t1 < t2)
      - 「主線につながらない」ジョブ(Branch と、前回の「両方の飛び越し」)の線は、分岐の行と合流の行で、列 0 の記号を空にする
  - name: OpenLane
    description: 配置の途中で持つ、開いている線(core の内部)。
    attributes:
      - name: target
        logical_type: 行き先の節の位置
      - name: all_off_main
        logical_type: bool(この線を開いたジョブが、すべて主線につながらないか。合流の行の列 0 を空にするかを決める)
```

要約: 新しい型は足さない。判定の結果が、「線なし」「新しい線」「合流」に、「分岐(新しい線 2 本)」が加わる。
