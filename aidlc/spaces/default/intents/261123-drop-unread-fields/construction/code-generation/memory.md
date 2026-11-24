<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->
- 2026-11-24T11:25:00Z — 仕様書 S14・S16 の擬似コードは、`inventory.md` の指示どおり層の名前を行末の注釈(`# [pipeline].env_file` など)で示し、自動の変数の層は最初の行に `# 自動の変数` と付けた。
- 2026-11-24T11:25:00Z — 各コミットで実行するテストは「変えたクレート」と読み、1〜3 は core、4・5 は core と cli にした(cli の src は変えないので、2 では念のため cli の `plan_report.rs` と `docs_dry_run.rs` も実行した)。

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->
- 2026-11-24T11:25:00Z — 計画からのずれは無い。T1〜T20 の外の既存のテストを変える必要は出なかった。T7 で確かめの 6 行を消したとき、後ろの空行 1 行も一緒に消した(空行の連続を避けるため。確かめの行ではない)。

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->
- 2026-11-24T11:25:00Z — `explicit` を消したので、辺が「書いた遷移」か「記述順の次」かの区別はテストで確かめられなくなった(設計 §4.6 で承認済み。本番はこの区別を読まず、位置は `to` で確かめる)。

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
