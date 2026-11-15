<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
- 2026-11-15T00:30:00Z — 2 回目のレビュー(READY)で残った Major(R-01)は、traceability.json の upstream_ids に、NFR6.1〜6.4、NFR2.1、NFR3.1、NFR5.1 が無いこと(上流の一覧は、検査の道具が作るため、手では足していない)。これらの設計は、security-design の「他の NFR の設計」と observability-design にある。Code Generation と Build and Test で、これらの要件を、抜けなく確かめること。NFR4.8(リリース)は、CI Pipeline・Deployment Pipeline のステージに委ねる(取り決め 3 点: --locked でのビルド、実行ファイル名 conductor、補助の実行ファイルを含めない)。
