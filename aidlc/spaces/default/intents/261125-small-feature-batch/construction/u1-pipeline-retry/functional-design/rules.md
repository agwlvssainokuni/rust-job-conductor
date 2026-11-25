# ビジネスルール: u1-pipeline-retry(パイプライン共通のリトライの既定)

出典: 要件(`requirements.md` の FR1.1〜FR1.6、FR8.1、NFR1、NFR6)、ユニットの定義(`unit-of-work.md` の U1)、要件とユニットの対応(`unit-of-work-story-map.md`)、コンポーネントの一覧(`components.md` の CoreDefinition)、設計の決定(`decisions.md` の ADR-003、ADR-008)、契約(`contract-summary.md` の C3)、機能の設計の質問(`functional-design-questions.md` の Q1〜Q4)。

規則の番号はこのユニットの中のもの。今のコードのドキュメントコメントにある、v0.8.0 の `[job.retry]` の規則の番号とは別である。

```yaml
rules:
  - id: BR1.1
    statement: "`[pipeline]` に `retry` を表(`[pipeline.retry]`)で書ける。表でない値(`false` を含む)は定義の誤り(64)"
    category: validation
    applies_to: PipelineRetryDefault
    trigger: 定義の検証で `[pipeline]` を読むとき
    logic: >
      IF `[pipeline]` に `retry` がある THEN 表なら PipelineRetryDefault を作る。
      表でなければ、種類 RetryNotTable(種類の文は今のまま)、理由「`[pipeline.retry]` はテーブルで書きます」の誤り
    violation: 定義の誤り(64)。どのジョブも動かない
    source: FR1.1

  - id: BR1.2
    statement: "`[pipeline.retry]` の項目と検査は `[job.retry]` と同じ。誤りの種類と理由の文も同じ"
    category: validation
    applies_to: PipelineRetryDefault
    trigger: BR1.1 で表を読んだとき
    logic: >
      書けるキー(exit_codes、max、interval、multiplier、max_interval)、必須(exit_codes、max、interval)、
      範囲(exit_codes は 1〜255 で空でない、max は 1〜100、interval と max_interval は 1 秒〜24 時間、
      multiplier は 2〜10)、multiplier と max_interval は両方か両方なし、max_interval は interval 以上。
      `[job.retry]` の読み取りと同じ手続きで検査する。誤りの位置は `[pipeline.retry]` の中の位置
    violation: "`[job.retry]` で同じ値を書いたときと同じ種類の定義の誤り(64)"
    source: FR1.1

  - id: BR2.1
    statement: "ジョブの `retry` は、表か `false`。`true` や、ほかの型は定義の誤り(64)"
    category: validation
    applies_to: JobRetrySetting
    trigger: 定義の検証でジョブを読むとき
    logic: >
      IF `retry` が無い THEN absent。IF `false` THEN disabled。IF 表 THEN table(BR1.2 と同じ検査)。
      ELSE 種類 RetryNotTable(種類の文は今のまま)、理由「`retry` はテーブルか `false` で書きます」の誤り
    violation: 定義の誤り(64)
    source: FR1.4

  - id: BR2.2
    statement: 各ジョブの実効のリトライは、ジョブの表があれば丸ごとそれ、false なら無し、書いていなければ既定
    category: calculation
    applies_to: JobDefinition.retry
    trigger: 検証で誤りが 1 つも無く、定義を組み立てるとき
    logic: >
      IF JobRetrySetting が table THEN その表から作った RetryPolicy(既定の項目と混ぜない)。
      IF disabled THEN 無し。
      IF absent AND 既定がある THEN 既定から作った RetryPolicy の複製。
      IF absent AND 既定が無い THEN 無し
    violation: 該当なし(誤りの無い定義だけを組み立てる)
    source: FR1.2, FR1.3, FR1.4

  - id: BR2.3
    statement: "既定が無い定義で `retry = false` を書いたジョブは、誤りにせず、リトライしない"
    category: policy
    applies_to: JobRetrySetting
    trigger: 定義の検証
    logic: IF 既定が無い AND disabled THEN 誤りにしない。実効のリトライは無し(BR2.2)
    violation: 該当なし
    source: FR1.4

  - id: BR2.4
    statement: "実効のリトライは検証の中で 1 度だけ決める。リトライの判断・流れ・`graph -v`・`--dry-run` はジョブの実効のリトライだけを見て、既定から来たかは見分けない"
    category: policy
    applies_to: JobDefinition, PipelineDefinition
    trigger: 定義を使うすべての場面
    logic: >
      組み立て後の定義は既定を持たず、各ジョブの retry に実効のリトライだけを持つ。
      表示は今の形のまま、実効のリトライを出す(既定由来の印は付けない)
    violation: 該当なし
    source: FR1.2

  - id: BR3.1
    statement: "既定の `exit_codes` が、既定を当てはめるジョブの成功または警告の終了コードと重なれば定義の誤り(64)"
    category: constraint
    applies_to: PipelineRetryDefault, JobReading
    trigger: 定義の検証の、ジョブどうしの関係の検査(cross_check)
    logic: >
      FOR 各ジョブ: IF retry が absent AND 既定の exit_codes が読めた AND
      (既定の exit_codes のどれかが、ジョブの成功の終了コード(書かなければ 0 だけ)に含まれる
       OR ジョブの警告の終了コードに含まれる) THEN 誤り。種類は RetryExitCodesOverlap(今の種類を使う)
    violation: 定義の誤り(64)。どのジョブも動かない
    source: FR1.5

  - id: BR3.2
    statement: 既定との重なりの誤りは、重なるジョブごとに 1 つずつ、そのジョブの表(`[[job]]`)の行で出す。理由の文に値もジョブの ID も含めない
    category: policy
    applies_to: BR3.1 の誤り
    trigger: BR3.1 の誤りを出すとき
    logic: >
      誤りの位置は、ジョブの表の始まり。理由の文は固定の文
      「既定のリトライ(`[pipeline.retry]`)の `exit_codes` に、このジョブの成功または警告の終了コードと同じ値があります」。
      ほかの誤りと一緒に集め、行の順に並べて返す(今の方式のまま)
    violation: 該当なし
    source: FR1.5

  - id: BR3.3
    statement: "`retry = false` のジョブと、自分の表を持つジョブは、既定との重なりを調べない"
    category: constraint
    applies_to: JobRetrySetting
    trigger: BR3.1 の検査
    logic: >
      IF disabled THEN 調べない。IF table THEN 既定とは調べず、今までどおりジョブの表の exit_codes と
      成功・警告の重なりを調べる
    violation: 該当なし
    source: FR1.5

  - id: BR3.4
    statement: 既定の `exit_codes` が誤り、またはジョブの成功の終了コードが誤りのときは、既定との重なりを調べない
    category: policy
    applies_to: BR3.1 の検査
    trigger: BR3.1 の検査
    logic: >
      IF 既定の exit_codes が読めていない THEN すべてのジョブで調べない。
      IF ジョブの success_exit_codes が誤り THEN そのジョブの成功との重なりを調べない(警告との重なりは調べる)。
      同じ原因の誤りを重ねて出さない(今の `[job.retry]` の扱いと同じ)
    violation: 該当なし
    source: FR1.5

  - id: BR3.5
    statement: 既定の間隔の上限と間隔の大小の誤りは、ジョブの数によらず `[pipeline.retry]` の位置で 1 回だけ出す
    category: validation
    applies_to: PipelineRetryDefault
    trigger: 定義の検証
    logic: IF 既定の max_interval が interval より小さい THEN 種類 RetryMaxIntervalTooSmall の誤りを 1 つ、max_interval の位置で
    violation: 定義の誤り(64)
    source: FR1.1

  - id: BR4.1
    statement: "`[pipeline.retry]` を書かない定義は、v0.9.9 と同じに動く(組み立てた定義、誤りの種類・文・行、表示)"
    category: constraint
    applies_to: 定義の検証の全体
    trigger: 定義の検証
    logic: >
      IF 既定が無い THEN 実効のリトライはジョブの表のとおり(BR2.2)。
      今ある誤り(`retry` が表でない等)の種類の文は変えない。
      変わるのは、ジョブの `retry` が表でも false でもないときの理由の文に「か `false`」が入ることだけ
    violation: 該当なし
    source: FR1.6

  - id: BR4.2
    statement: README とデモを更新し、`conductor init` のひな形は変えない
    category: policy
    applies_to: 文書
    trigger: このユニットの最後
    logic: >
      README のリトライの節に `[pipeline.retry]` と `retry = false` を足し、終了コード 64 の表の説明に
      `[pipeline.retry]` の誤りを足す。デモ(`demo/retry.toml` と `demo/windows/retry.toml`)に既定と
      打ち消しの例を足し、デモの説明を合わせる。`init` のひな形は今のまま
    violation: 該当なし
    source: FR8.1
```

## まとめ

| ID | 規則 | 種類 | 出典 |
|---|---|---|---|
| BR1.1 | `[pipeline.retry]` は表で書く | validation | FR1.1 |
| BR1.2 | 既定の項目と検査は `[job.retry]` と同じ | validation | FR1.1 |
| BR2.1 | ジョブの `retry` は表か `false` | validation | FR1.4 |
| BR2.2 | 実効のリトライ: 表→丸ごと、false→無し、無し→既定 | calculation | FR1.2、FR1.3、FR1.4 |
| BR2.3 | 既定が無い `retry = false` は許す | policy | FR1.4 |
| BR2.4 | 実効のリトライを 1 度だけ決め、既定由来は見分けない | policy | FR1.2 |
| BR3.1 | 既定と成功・警告の重なりは 64 | constraint | FR1.5 |
| BR3.2 | 重なりの誤りはジョブごとに、ジョブの行で | policy | FR1.5 |
| BR3.3 | false と自分の表のジョブは既定の重なりの対象外 | constraint | FR1.5 |
| BR3.4 | 前の誤りがあれば重なりを調べない | policy | FR1.5 |
| BR3.5 | 既定の上限と間隔の大小の誤りは 1 回 | validation | FR1.1 |
| BR4.1 | 既定を書かない定義は今と同じ | constraint | FR1.6 |
| BR4.2 | README とデモを更新、ひな形は変えない | policy | FR8.1 |
