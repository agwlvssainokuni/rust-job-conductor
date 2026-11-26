<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->
- 2026-11-27T00:00:00Z — 知識ベースが STALE だったため、範囲を絞らず全体を読み直した(人の選択)。v0.10.0 で 3 クレートすべてに手が入り、絞ると範囲外が shallow に下がるため。比較の結果は COVERS。

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
- 2026-11-27T00:00:00Z — F2 で予約名の検査の前後の空白を除くと、空白を含むジョブ ID のログの名前が v0.10.0 と変わる。互換の扱いを要件で決める。
- 2026-11-27T00:00:00Z — F1〜F5 の外の文書のずれ(`docs/spec/rust/` の follow・log_dir・unset_env の記述不足と誤りの種類 33/34、runtime `lib.rs` の契約表の `unset`、team.md のテストの数)を今回の範囲に入れるかを要件で決める。
