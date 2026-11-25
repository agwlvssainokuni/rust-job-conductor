# エンティティ: u1-pipeline-retry(パイプライン共通のリトライの既定)

出典: ユニットの定義(`unit-of-work.md` の U1)、要件(`requirements.md` の FR1.1〜FR1.6)、コンポーネントの一覧(`components.md` の CoreTypes、CoreDefinition)、契約(`contract-summary.md` の C3)、機能の設計の質問(`functional-design-questions.md` の Q1〜Q4)。要件とユニットの対応(`unit-of-work-story-map.md`)では、FR1.1〜FR1.6 と FR8.1 がこのユニットに当たる。

このユニットは、組み立て後の定義の型(`PipelineDefinition`、`JobDefinition`、`RetryPolicy`)の形を変えない。既定(`[pipeline.retry]`)は検証の中でだけ持ち、各ジョブの `retry` に「実効のリトライ」を入れて返す(ADR-003)。既定から来たかどうかは残さない(Q3 の A)。新しく足すのは、検証の途中の形(読み取りの結果)だけである。

```yaml
entities:
  - name: PipelineRetryDefault
    kind: 検証の途中の値(CoreDefinition の中だけ。組み立て後には残らない)
    description: >
      `[pipeline]` の `retry` の表(`[pipeline.retry]`)を読んだ結果。項目と検査は
      `[job.retry]` の読み取りの結果(RetryReading)と同じ。
    attributes:
      - name: reading
        type: RetryReading
        required: true
        constraints: 表で書いたときだけ作る。表でなければ作らず、定義の誤り(64)
    relationships:
      - to: PipelineDefinitionReading
        cardinality: "0..1 対 1"
        direction: パイプラインの読み取りが高々 1 つ持つ

  - name: RetryReading
    kind: 検証の途中の値(既存の RawRetry)
    description: >
      `retry` の表 1 つを読んだ結果。値が正しい項目だけを持ち、誤りの位置を持つ。
      ジョブの表と既定の表で同じ形を使う。
    attributes:
      - name: exit_codes
        type: 終了コードの並びと位置
        required: true
        constraints: 1〜255 の整数、空でない、昇順で重複なし。誤りなら値を持たない
      - name: max
        type: 整数
        required: true
        min: 1
        max: 100
      - name: interval
        type: 秒と位置
        required: true
        constraints: 1 秒〜24 時間
      - name: multiplier
        type: 整数
        required: false
        min: 2
        max: 10
        constraints: max_interval と両方あるか、両方ないか
      - name: max_interval
        type: 秒と位置
        required: false
        constraints: interval 以上

  - name: JobRetrySetting
    kind: 検証の途中の値(既存の RawJob の retry を 3 つの場合に広げる)
    description: ジョブに書いた `retry` の読み取りの結果。
    attributes:
      - name: case
        type: 列挙
        required: true
        allowed_values: [absent, disabled, table]
        default: absent
        constraints: >
          absent は `retry` を書いていない。disabled は `retry = false`。
          table は `[job.retry]` の表(RetryReading を持つ)。
          `retry = true` と、表でも false でもない値は誤り(64)で、この値を作らない
      - name: reading
        type: RetryReading
        required: false
        constraints: case が table のときだけ持つ
    relationships:
      - to: JobReading
        cardinality: "1 対 1"
        direction: ジョブの読み取りが 1 つ持つ

  - name: JobReading
    kind: 検証の途中の値(既存の RawJob)
    description: ジョブ 1 つの読み取りの結果。retry 以外の項目は今のまま。
    attributes:
      - name: start
        type: 位置
        required: true
        constraints: ジョブの表(`[[job]]`)の始まり。既定との重なりの誤りの行に使う(Q1 の A)
      - name: success
        type: 成功の終了コードの読み取り(absent、invalid、valid)
        required: true
      - name: warning
        type: 警告の終了コードと位置
        required: false
      - name: retry
        type: JobRetrySetting
        required: true

  - name: JobDefinition
    kind: 組み立て後の定義(既存。CoreTypes。形は変えない)
    description: ジョブ 1 つの定義。
    attributes:
      - name: retry
        type: RetryPolicy
        required: false
        constraints: >
          実効のリトライ。ジョブの表があればそれ、disabled なら無し、absent なら既定の複製
          (既定が無ければ無し)。既定から来たかは持たない
    relationships:
      - to: PipelineDefinition
        cardinality: "多 対 1"
        direction: 各ジョブは 1 つのパイプラインに属する

  - name: RetryPolicy
    kind: 組み立て後の定義(既存。CoreTypes。形は変えない)
    description: 1 つのジョブのリトライの設定。
    attributes:
      - name: exit_codes
        type: 終了コードの並び
        required: true
      - name: max
        type: 整数
        required: true
      - name: interval_secs
        type: 秒
        required: true
      - name: backoff
        type: 倍率と上限の組
        required: false
    relationships:
      - to: JobDefinition
        cardinality: "0..1 対 1"
        direction: ジョブが高々 1 つ持つ。既定から来たものは、ジョブごとに別の複製

  - name: PipelineDefinition
    kind: 組み立て後の定義(既存。CoreTypes。形は変えない)
    description: パイプラインの定義。既定(`[pipeline.retry]`)は持たない(Q3 の A)。
    attributes:
      - name: job
        type: JobDefinition の並び
        required: true
```

## まとめ

| エンティティ | 種類 | このユニットでの変化 |
|---|---|---|
| PipelineRetryDefault | 検証の途中 | 新規。`[pipeline.retry]` を読んだ結果 |
| RetryReading | 検証の途中 | 既存(RawRetry)。既定の表にも使う |
| JobRetrySetting | 検証の途中 | 新規。ジョブの `retry` を「無し・false・表」の 3 つで持つ |
| JobReading | 検証の途中 | 既存(RawJob)。retry を JobRetrySetting に、ジョブの表の始まりの位置を足す |
| JobDefinition / RetryPolicy / PipelineDefinition | 組み立て後 | 形は変えない。`JobDefinition.retry` の意味が「実効のリトライ」になる |

変換のツール(`converter`)は、定義を型に依らない木として読み書きするため、`[pipeline.retry]` と `retry = false` は手を入れずに通る(このユニットの対象外)。
