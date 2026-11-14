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
- 2026-11-14T20:36:00Z — 2回目のレビュー(最終、READY)で残った指摘の引き継ぎ(Code Generation で拾う): (1) traceability.json は NFR4.1〜NFR4.7 のみで、NFR1.1・NFR2.1・NFR3.1・NFR5.1 の行がない(測定できる基準は security-design に記載済み。NFR5.1 の基準は、Code Generation のテストの計画で、行カバレッジ 80% として明記する)。(2) 位置を持つ要素のない誤り(必須キーの欠落、`noJobs`、パイプライン名の不一致)の行は、親の要素(テーブル、ファイル全体)の位置を使う。型の誤りで検査できなくなった子の後続の検査は、その子の範囲だけ省略し、他の検査は続ける。(3) D1 の「serde と toml」の表記は、実際の方式(位置を保つ文書の木を、自分で走査する)では、serde を読み取りに使わないことを明記する。serde は、変換(u4-converter)とステート(u2-runtime)で使う。
