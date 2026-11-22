# エンティティ(Entities): graph の配置のリファクタリング

新しい永続のデータも、新しい公開の型もない。既存の図の型(`crates/core/src/graph/mod.rs`)をそのまま使う。変わるのは、配置(core の `graph/layout.rs`)の内部の持ち方だけ。

```yaml
entities:
  - name: OpenLanes
    description: 配置の途中で持つ、開いている線の並び(core の内部。内側が先頭。前回の ordinary_layout と同じ)。同じ行き先の重複を許し、寄せで 1 本にする。
    attributes:
      - name: lanes
        logical_type: list of 行き先の節の位置
  - name: Reachable
    description: 先頭のジョブから、辺をたどって着けるジョブの集合(core の内部。新規)。主線の列 0 の記号(BR1.4)を決める。
    attributes:
      - name: reachable
        logical_type: set of ジョブの位置
  - name: SkipPlan
    description: 配置の途中結果(core の内部)。ジョブごとの飛び越しの行き先(0〜2 個)と、種類(飛び越しなし、通常、両方の飛び越し、分岐)。前回の `ordinary` の欄は、経路が 1 つになるので、なくす。古い経路のための `off_main` の扱いや、「分岐が開いた」印も、なくす。
```

要約: 型は変えない。配置の内部で、経路が 1 つになり、「実行されうるジョブ」の集合を新しく持つ。
