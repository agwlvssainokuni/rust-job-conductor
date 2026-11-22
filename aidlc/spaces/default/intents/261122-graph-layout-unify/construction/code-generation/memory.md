<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->
- 2026-11-22 — 分岐の線の順は、参照実装の `[t1, t2] + lanes` を、行き先の昇順(近い行き先が内側)と解した。正解データのラベルは常に近い行き先が先で、BR1.3 と一致する。
- 2026-11-22 — N4 は §4 に図がないので、参照実装から導いた図(N1 と同じ見た目)を期待にした。

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->
- 2026-11-22 — 定義から作れる形は、すべて 4 つの種類に入るので、辺の一覧は定義からは出なくなった。辺の一覧に残る形のテストは、図の構造を直接作る形にし、cli の辺の一覧の整形は `GraphLayout::EdgeList` を直接渡して確かめた。実バイナリの辺の一覧の確かめは、レーンの図の期待に替えた。
- 2026-11-22 — `ordinary_layout` を `layout_lanes` に改名した(唯一の配置になったため)。Step 2 のコミットでは、ビルドを通すための一時の補助を残し、Step 3 で削除した。

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->
- 2026-11-22 — 主線の列 0 は、`LaneLayout` に `main_line`(ノードごと)として持たせた(`LaneRow` に足すと、すべての行の構築と既存のテストが変わるため)。
- 2026-11-22 — Step 3(37cbbe5)から Step 6(344843a)の前までのコミットでは、期待の書き換え前のテストが失敗し、README の例のテスト(`docs_graph`)は Step 9(fb5a08e)まで失敗する(test-after と、計画のコミットの順のため)。各コミットのビルドは通る。最後のコミットでは、全テストが通る。

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
- 2026-11-22 — 辺の一覧の整形と注記の文(「飛び越しが交差するなどの形のため」)は、定義からは出なくなった。残すか、文を改めるかは、次の作業で決める。
- 2026-11-22 — 版(0.9.3)は上げていない(計画に版の更新の手順がない)。
