# Entities — u1-retry-config

ジョブの定義に書く「リトライの設定」のデータの形。技術には依存しない書き方で示す。既存の `JobDefinition`(ジョブの定義)に、0 または 1 個の `RetryPolicy` を持たせる。根拠: `requirements.md`(FR1、FR7)、`functional-design-questions.md`(Q1〜Q6)。

```yaml
entities:
  - name: RetryPolicy
    description: ジョブごとのリトライの設定。ジョブの定義に表 `retry` があるときだけ存在する。
    attributes:
      - name: exit_codes
        logical_type: 整数の集合
        required: true
        unique: true
        constraints: [空でない, "各値は 1 以上 255 以下"]
        notes: 重複して書いても 1 つとして扱う。成功と警告の終了コードとは重ならない。
      - name: max
        logical_type: 整数
        required: true
        constraints: ["1 以上 100 以下"]
        notes: 最初の実行に加えて、実行し直す最大の回数。
      - name: interval
        logical_type: Duration
        required: true
        constraints: ["1 秒以上 24 時間以下"]
      - name: multiplier
        logical_type: 整数
        required: false
        constraints: ["2 以上 10 以下", max_interval と同時に書く(両方あるか、両方ない)]
      - name: max_interval
        logical_type: Duration
        required: false
        constraints: ["1 秒以上 24 時間以下", "interval 以上", multiplier と同時に書く]
    entity_constraints:
      - exit_codes と、そのジョブの成功の終了コードの集合、警告の終了コードの集合は、どれも重ならない
      - multiplier と max_interval は、両方あるか、両方ない
      - max_interval >= interval
    relationships:
      - from: JobDefinition
        to: RetryPolicy
        cardinality: "0..1"
        direction: JobDefinition が RetryPolicy を持つ
  - name: Duration
    description: 時間の長さを表す値。定義ファイルでは、整数と単位からなる文字列で書く。
    attributes:
      - name: text
        logical_type: 文字列
        required: true
        constraints: ["`<整数><単位>`。単位は s、m、h のどれか", "整数は 1 以上", "小数、空、単位なし、ほかの単位は不可"]
      - name: seconds
        logical_type: 整数
        required: true
        notes: text から計算した秒数(s は 1、m は 60、h は 3600 倍)。実行側(u2)は、この整数の秒を使う。
  - name: RetryDefinitionError
    description: リトライの設定の誤り。既存の定義の誤り(終了コード 64)の一種として扱う。位置(行、桁)とキー名を伴い、固定の文だけで表す。
    attributes:
      - name: kind
        logical_type: 列挙
        required: true
        allowed_values:
          - retryNotTable
          - retryMissingKey
          - retryUnknownKey
          - retryExitCodesEmpty
          - retryExitCodeInvalid
          - retryExitCodesOverlap
          - retryMaxInvalid
          - retryDurationInvalid
          - retryBackoffIncomplete
          - retryMultiplierInvalid
          - retryMaxIntervalTooSmall
      - name: location
        logical_type: 位置
        required: true
```

## 要約

- 新しいエンティティは 3 つ。`RetryPolicy`(設定の本体)、`Duration`(時間の長さ)、`RetryDefinitionError`(誤りの種類)。
- `RetryPolicy` は、ジョブの定義に表 `retry` があるときだけ存在する。ないジョブは、これまでと同じ(リトライなし)。
- 実行側(u2)に渡すのは、`exit_codes`(整数の集合)、`max`、`interval` の秒、`multiplier` と `max_interval` の秒(あれば)。値はすべて整数で、小数は使わない。(要件の制約 C1)
- 誤りの種類は、設計上の識別名である。実際の名前と数(既存は 20 種類)は、コードの生成で、既存の命名に合わせて決める。
