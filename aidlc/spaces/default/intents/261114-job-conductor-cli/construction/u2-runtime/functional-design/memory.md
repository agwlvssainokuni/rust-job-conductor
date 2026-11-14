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
- 2026-11-14T22:55:00Z — **2 回目のレビューは NOT-READY(Critical 1 件)のまま、最大回数に達して完了した。** Critical は、apply_skip の completed の導出(最後のジョブのスキップより前に completed になる)で、設計を直した(「いま skipped にしたジョブが、定義の順で最後のジョブ」の条件)が、直したあとの再レビューは、行っていない。Code Generation の計画のレビューと、実装のレビューで、AC4.2.2・AC4.2.3 の個別実行の経路(`@end`、最後のジョブのスキップ)を、重点的に見ること。ほかの引き継ぎ: (1) u1-core の設計の成果物は、`@end` で以降のジョブを先に skipped として記録する形のままだが、u1-core の実装は、個別実行では、呼ばれたときに記録する形である。u2 は、どちらでも動く条件にした。(2) UNIX 系で、ジョブを別のプロセスグループで起動する選択(対話的なジョブは対象外、孤児が残りやすい)は、人の確認がまだである。
