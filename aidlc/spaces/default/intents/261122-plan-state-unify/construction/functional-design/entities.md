# エンティティ(Entities): `--dry-run` の模擬の重複をなくすリファクタリング

新しい型、永続のデータ、ステートファイルの形は変えない。既存の `PipelineState`(core の `types.rs`)を、模擬の中でも使うようになるだけ。

```yaml
entities:
  - name: PipelineState
    description: パイプラインの実行の途中経過(既存。core)。模擬(plan_pipeline)は、これまでの「状態の一覧、再開位置、end_reached の 3 つの変数」の代わりに、これを 1 つ持ち、本番と同じ更新の関数で進める(保存はしない)。
    attributes:
      - name: job_states
        logical_type: list of JobState
      - name: resume_job_id
        logical_type: optional job id
      - name: end_reached
        logical_type: boolean
      - name: completed
        logical_type: boolean
```
