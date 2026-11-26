<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->
- 2026-11-26T23:10:00Z — Loop-back 1: CI の coverage で plan_simulate_perf.rs が上限 150ms を超えた(P1 161ms、P2 155ms)。F3 を「直さない」とした判定(手元の速い機械で BR3.3 を当てた)が根。人が Retry with fix を選び、コード生成に戻って計画の Step 6 の直しを行う。

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
- 2026-11-26T23:10:00Z — 性能の判定の基準(BR3.3)を手元の機械だけで当てると、CI の遅い機械での結果を見誤る。次からは、時間の上限を持つテストの判断に CI の計測の値も使うか(学びの候補)。
