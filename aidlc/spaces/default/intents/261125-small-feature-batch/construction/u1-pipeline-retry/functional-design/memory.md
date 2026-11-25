<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->

- 2026-11-25T10:24:46Z — 既定の重なりの誤りの種類は、新しい種類を足さず既存の RetryExitCodesOverlap を使う; 種類の文(label)は「retry の終了コードが、成功または警告の終了コードと重なっています」で既定にもそのまま当てはまる。既定由来であることは理由の文で示す。
- 2026-11-25T10:24:46Z — `retry = true` などの誤りは既存の RetryNotTable を使い、種類の文(「retry は表で書きます」)は変えず、理由の文だけ「表か false」にする; 種類の文を変えると、既定を書かない定義の誤りの表示まで変わり FR1.6 に触れるため。

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->
- 2026-11-25T10:24:46Z — 定義の型(PipelineDefinition、JobDefinition、RetryPolicy)は変えず、既定は検証の中でだけ持つ; Q3 の A(見分けない)により、既定由来の区別も既定そのものも組み立て後に要らない。graph -v・--dry-run・リトライの判断は今の `job.retry` をそのまま読むので、cli と flow の変更が要らない。
- 2026-11-25T10:24:46Z — 読み取りの途中の形(RawJob の retry)を「無し・false・表」の 3 つにする; Option のままでは false と無しを見分けられず、実効の決定ができないため。

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
