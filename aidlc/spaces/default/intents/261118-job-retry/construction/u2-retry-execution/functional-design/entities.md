# Entities — u2-retry-execution

リトライの実行で扱うデータの形。技術には依存しない書き方で示す。u1 が作った `RetryPolicy`(ジョブの設定)を、実行側が読む。根拠: `requirements.md`(FR2〜FR6)、`functional-design-questions.md`(Q1〜Q3)、既存のコードの事実(`code-structure.md`)。

```yaml
entities:
  - name: RetryPolicy
    description: u1 が定義から読み取った、ジョブごとのリトライの設定。この Unit は読むだけで、変えない。
    attributes:
      - name: exit_codes
        logical_type: 整数の集合
        required: true
      - name: max
        logical_type: 整数
        required: true
        constraints: ["1 以上 100 以下"]
      - name: interval_secs
        logical_type: 整数(秒)
        required: true
      - name: multiplier
        logical_type: 整数
        required: false
      - name: max_interval_secs
        logical_type: 整数(秒)
        required: false
    relationships:
      - from: JobDefinition
        to: RetryPolicy
        cardinality: "0..1"
        direction: u1 が定義した関係を使う
  - name: RetryState
    description: 1 つのジョブの実行の間だけ持つ、リトライの途中の状態。ステートファイルには記録しない。
    attributes:
      - name: retries_done
        logical_type: 整数
        required: true
        defaults: 0
        constraints: ["0 以上 max 以下"]
        notes: すでに実行し直した回数。最初の実行の結果が出た時点では 0。
      - name: last_wait_secs
        logical_type: 整数(秒)
        required: false
        notes: 直前の待ち時間(回ごとに伸ばす方式の計算に使う)。
  - name: RetryDecision
    description: ジョブの 1 回の実行結果から、純粋に決まる次の動き。
    attributes:
      - name: kind
        logical_type: 列挙
        required: true
        allowed_values: [Stop, Retry]
      - name: wait_secs
        logical_type: 整数(秒)
        required: false
        notes: kind が Retry のときだけ。次の実行までの待ち時間。
  - name: RetryNotice
    description: リトライの前に利用者へ知らせる内容。DisplayEvent の新しい種類(`RetryScheduled`)として表示の仕組みに出す。
    attributes:
      - name: job_id
        logical_type: 文字列
        required: true
      - name: last_exit_code
        logical_type: 整数
        required: true
        notes: 直前の実行の終了コード。
      - name: retry_number
        logical_type: 整数
        required: true
        constraints: ["1 以上 max 以下"]
      - name: max
        logical_type: 整数
        required: true
      - name: wait_secs
        logical_type: 整数(秒)
        required: true
  - name: WaitOutcome
    description: 間隔の待機の結果。
    attributes:
      - name: kind
        logical_type: 列挙
        required: true
        allowed_values: [Elapsed, Interrupted]
      - name: signal_number
        logical_type: 整数
        required: false
        notes: kind が Interrupted のときだけ。最初に受け取ったシグナルの番号。
    relationships:
      - from: RetryDecision
        to: WaitOutcome
        cardinality: "1..1"
        direction: Retry の決定のあと、待機して WaitOutcome を得る
```

## 要約

- 新しいエンティティは 4 つ。`RetryState`(実行の間だけ持つ。永続化しない)、`RetryDecision`(純粋な判断の結果)、`RetryNotice`(表示の内容)、`WaitOutcome`(待機の結果)。`RetryPolicy` は u1 のもの。
- ステートファイルの形式は変えない。回数は、メモリの中だけで持つ。(FR5.1、FR5.2)
- 値はすべて整数(秒、回数、終了コード)。小数は使わない。
