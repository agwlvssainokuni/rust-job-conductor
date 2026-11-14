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
- 2026-11-14T20:24:00Z — 2回目のレビュー(最終)で残った指摘(Major 1件、Minor 2件)の引き継ぎ: (1) 個別実行で、最後のジョブがスキップされたときの `completed` と、再開位置の解除を、どの関数が出すかが決まっていない(W2 は Decision だけを返し、W3・W4 は呼ばれない)。W2 に「完了の判定」を出力として足すか、新しい W5 を設ける。契約 C2 の更新と合わせて、Code Generation の前に決める。(2) W1 に、BR1.13 の検査(ジョブIDの重複・空、ジョブが0、空の name・command、遷移の重複)の手順を足し、BR1.11 の受理の条件と規則の要約表(BR1.1〜BR1.13)を揃える。(3) W3 の手順番号の重複(5 が2つ)を直し、起動しなかった中断の JobOutcome.classification の値を定める(分類しない旨)。
