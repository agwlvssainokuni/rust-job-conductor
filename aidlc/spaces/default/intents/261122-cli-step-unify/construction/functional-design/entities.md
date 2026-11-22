# エンティティ(Entities): run と pipeline の重複をなくすリファクタリング

永続のデータ、公開の型、ステートファイルの形は変えない。cli の内部に、共通の手順の戻り値の型を 1 つ足すだけ。

```yaml
entities:
  - name: StepResult
    description: 1 つのジョブを実行し、記録したあとの結果(cli の内部。新規。step.rs)。呼び出し側(run、pipeline)が、終了コードと、完了・中断の扱いを決めるのに使う。
    attributes:
      - name: recorded
        logical_type: PipelineState(記録後のステート。書き込み済み)
      - name: outcome
        logical_type: JobOutcome(分類と、報告の終了コード)
      - name: next
        logical_type: NextStep(完了したか、など)
      - name: after
        logical_type: StateContext(ジョブが起動したか。後の書き込みと削除の誤りの文脈)
```
