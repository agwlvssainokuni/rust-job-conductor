# Business Rules — u1-retry-config

リトライの設定を、定義から読み取るときの規則。根拠は、`requirements.md` の要件(FR)と、`functional-design-questions.md` の回答(Q)。誤りは、すべて定義の誤り(終了コード 64)になり、`validate` でも検出される。

```yaml
rules:
  - id: BR1.1
    statement: ジョブの定義に表 `retry` がなければ、リトライしない。ジョブの動作は、これまでと変わらない。
    category: policy
    applies_to: JobDefinition
    trigger: 定義の読み取り
    logic: IF `retry` がない THEN RetryPolicy を持たない
    violation_behaviour: なし(誤りではない)
    source: FR1.2, FR1.6, NFR1
  - id: BR1.2
    statement: `retry` は表(テーブル)でなければならない。
    category: validation
    applies_to: retry
    trigger: 定義の読み取り
    logic: IF `retry` が表でない THEN 誤り retryNotTable
    violation_behaviour: 定義の誤り(64)
    source: FR1.1
  - id: BR1.3
    statement: 表 `retry` には、`exit_codes`、`max`、`interval` を必ず書く。どれかを省略したら誤りにする。
    category: validation
    applies_to: retry
    trigger: 定義の読み取り
    logic: IF exit_codes、max、interval のどれかがない THEN 誤り retryMissingKey(足りないキー名を示す)
    violation_behaviour: 定義の誤り(64)
    source: FR1.1, FR1.7
  - id: BR1.4
    statement: 表 `retry` に書けるキーは、`exit_codes`、`max`、`interval`、`multiplier`、`max_interval` だけである。
    category: validation
    applies_to: retry
    trigger: 定義の読み取り
    logic: IF 上記以外のキーがある THEN 誤り retryUnknownKey(既存の未知のキーと同じ扱い)
    violation_behaviour: 定義の誤り(64)
    source: FR1.1
  - id: BR1.5
    statement: `exit_codes` は空でない整数の一覧で、各値は 1 以上 255 以下とする。0 は誤りにする。
    category: validation
    applies_to: retry.exit_codes
    trigger: 定義の読み取り
    logic: IF 一覧が空 THEN 誤り retryExitCodesEmpty; IF 整数でない、または 1 未満、または 255 超の値がある THEN 誤り retryExitCodeInvalid
    violation_behaviour: 定義の誤り(64)
    source: FR1.1, FR1.8
  - id: BR1.6
    statement: `exit_codes` に重複があっても誤りにしない。集合として扱う。
    category: policy
    applies_to: retry.exit_codes
    trigger: 定義の読み取り
    logic: IF 同じ値が複数ある THEN 1 つにまとめる
    violation_behaviour: なし
    source: FR1.1
  - id: BR1.7
    statement: `exit_codes` は、そのジョブの成功の終了コードの集合(省略時は 0 だけ)と、警告の終了コードの集合と、重なってはならない。
    category: constraint
    applies_to: retry.exit_codes
    trigger: 定義の読み取り(成功と警告の集合が決まったあと)
    logic: IF exit_codes と成功の集合、または exit_codes と警告の集合が共通の値を持つ THEN 誤り retryExitCodesOverlap
    violation_behaviour: 定義の誤り(64)
    source: FR1.3
  - id: BR1.8
    statement: `exit_codes` に 127 を書いてもよい。定義の誤りにはしない。起動できなかった場合の扱いは、実行側(u2)の規則である。
    category: policy
    applies_to: retry.exit_codes
    trigger: 定義の読み取り
    logic: IF 127 がある THEN 受け付ける
    violation_behaviour: なし
    source: FR3.6
  - id: BR1.9
    statement: `max` は 1 以上 100 以下の整数でなければならない。
    category: validation
    applies_to: retry.max
    trigger: 定義の読み取り
    logic: IF 整数でない、または 1 未満、または 100 超 THEN 誤り retryMaxInvalid
    violation_behaviour: 定義の誤り(64)
    source: FR1.1, FR1.8
  - id: BR1.10
    statement: 間隔(`interval`、`max_interval`)は、整数と単位 `s`、`m`、`h` のいずれかからなる文字列で書く。整数は 1 以上で、結果は 1 秒以上 24 時間以下とする。
    category: validation
    applies_to: retry.interval, retry.max_interval
    trigger: 定義の読み取り
    logic: IF 文字列でない、形が違う(小数、空、単位なし、ほかの単位)、1 秒未満、24 時間超 THEN 誤り retryDurationInvalid
    violation_behaviour: 定義の誤り(64)
    source: FR1.4, FR1.8
  - id: BR1.11
    statement: `multiplier` と `max_interval` は、両方を書くか、両方とも書かないかのどちらかにする。
    category: constraint
    applies_to: retry
    trigger: 定義の読み取り
    logic: IF 一方だけがある THEN 誤り retryBackoffIncomplete
    violation_behaviour: 定義の誤り(64)
    source: FR1.5, FR1.9
  - id: BR1.12
    statement: `multiplier` は 2 以上 10 以下の整数でなければならない。
    category: validation
    applies_to: retry.multiplier
    trigger: 定義の読み取り
    logic: IF 整数でない、または 2 未満、または 10 超 THEN 誤り retryMultiplierInvalid
    violation_behaviour: 定義の誤り(64)
    source: FR1.5, FR1.8
  - id: BR1.13
    statement: `max_interval` は `interval` 以上でなければならない。
    category: constraint
    applies_to: retry
    trigger: 定義の読み取り(両方の値が決まったあと)
    logic: IF max_interval が interval より小さい THEN 誤り retryMaxIntervalTooSmall
    violation_behaviour: 定義の誤り(64)
    source: FR1.5
  - id: BR1.14
    statement: リトライの設定の誤りは、ほかの定義の誤りと同じ仕組みで集め、位置の順に報告する。`validate` も同じ誤りを検出し、ジョブは実行しない。
    category: policy
    applies_to: 定義の検査
    trigger: 定義の読み取り、validate
    logic: IF リトライの設定に誤りがある THEN 既存の誤りと同じく、位置の順に集めて報告し、実行しない
    violation_behaviour: 定義の誤り(64)
    source: FR7.1
  - id: BR1.15
    statement: `convert`(YAML と TOML の相互変換)は、`retry` の表を、キーと値(整数と文字列)のまま、失わずに変換する。
    category: policy
    applies_to: convert
    trigger: convert の実行
    logic: IF `retry` の表を含む定義を往復変換する THEN 内容が同じになる
    violation_behaviour: 変換の誤り(既存)
    source: FR7.2
  - id: BR1.16
    statement: リトライの設定が正しい定義の、リトライと関係のない動作(成功・警告・エラーの分類、遷移、終了コード)は、これまでと変わらない。
    category: constraint
    applies_to: 定義の検査
    trigger: 定義の読み取り
    logic: IF 定義にリトライの表があっても THEN リトライ以外の読み取りの結果は変わらない
    violation_behaviour: なし
    source: FR1.6, NFR1
```

## 規則の要約

| ID | 内容 | カテゴリ | 要件 |
|---|---|---|---|
| BR1.1 | `retry` がなければリトライしない | policy | FR1.2、FR1.6 |
| BR1.2 | `retry` は表 | validation | FR1.1 |
| BR1.3 | `exit_codes`、`max`、`interval` は必須 | validation | FR1.1、FR1.7 |
| BR1.4 | 許可するキーだけ | validation | FR1.1 |
| BR1.5 | `exit_codes` は空でなく 1〜255 | validation | FR1.1、FR1.8 |
| BR1.6 | `exit_codes` の重複は集合として扱う | policy | FR1.1 |
| BR1.7 | 成功・警告の終了コードと重ならない | constraint | FR1.3 |
| BR1.8 | 127 を書いてもよい | policy | FR3.6 |
| BR1.9 | `max` は 1〜100 | validation | FR1.1、FR1.8 |
| BR1.10 | 間隔の書式と範囲(1 秒〜24 時間) | validation | FR1.4、FR1.8 |
| BR1.11 | `multiplier` と `max_interval` は両方か、両方なし | constraint | FR1.5、FR1.9 |
| BR1.12 | `multiplier` は 2〜10 | validation | FR1.5、FR1.8 |
| BR1.13 | `max_interval` は `interval` 以上 | constraint | FR1.5 |
| BR1.14 | 誤りは位置の順に集め、`validate` も検出する | policy | FR7.1 |
| BR1.15 | `convert` が `retry` を失わない | policy | FR7.2 |
| BR1.16 | リトライ以外の読み取りの結果は変わらない | constraint | FR1.6、NFR1 |
