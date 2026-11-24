<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->
- 2026-11-24T14:00:00Z — T4 の `common::with_timeout`・`common::toml`・`common::yaml` の `common::` を外すことは、§4.4 の T4 に書かれた変更と読み、本体の 6 行を直した(確かめの中身は同じ)。
- 2026-11-24T14:00:00Z — 各コミットの確かめは、失敗で止まるスクリプトで fmt、clippy(workspace、all-targets)、converter のテストを走らせ、単体・統合・無視の件数を見込みと照らした。全テストは最後に 1 回走らせた。

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->
- 2026-11-24T14:00:00Z — 計画からのずれは無い。T1〜T7 の外の既存のテストを変える必要は出なかった。件数はすべてのコミットで見込みどおり(最後は 1,152 通過、1 無視)。T1 の import の書き換えで、`mod common;` の後ろの空行 1 行が消えた(確かめの行ではない)。

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->
- 2026-11-24T14:00:00Z — 移したテスト(`src/tests/`)と `Value` の `eq` の本体は、`tests/nfr_security.rs` のソースの検査(`src/*.rs` の直下、最初の `#[cfg(test)]` より前)の外になる。代わりに `src/tests/` の禁止の語の検索(0 件)と、`types.rs` の最初の `#[cfg(test)]` が `impl PartialEq` の直前であることを記録に残した(BR3.2)。

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
