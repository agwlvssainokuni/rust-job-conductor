<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->
- 2026-11-24T16:00:00Z — 各コミットの確かめは、失敗で止まるスクリプト(`target/cli-pub-work/`。終わってから消した)で fmt、clippy(workspace、all-targets)、cli のテストを走らせ、cli の単体・統合の件数を §8 の見込みと照らした。全テストと §7 の検索は最後に 1 回走らせた。
- 2026-11-24T16:00:00Z — S19 の `init_template.rs`・`init_types.rs` の 1 行は、移すコミットが違う(4 と 8)ので、コミット 4 で 2 行に分けてから 1 行ずつ単体テストの表へ移すと読んだ。

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->
- 2026-11-24T16:00:00Z — 計画からのずれは無い。T1〜T10 の外の既存のテストを変える必要は出なかった。件数はすべてのコミットで見込みどおり(最後は 1,152 通過、1 無視。cli 単体 164 / 統合 294)。細かな点(仕様書の表の行の並びの途中の入れ替え、`docs_*` が git の履歴で移動として出ないこと、コメントの折り返し)は `code-summary.md` の「計画からのずれ」に記録した。

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->
- 2026-11-24T16:00:00Z — `src/tests/` は `nfr_source.rs` の OS 分岐と `panic!(` の検査の外になる(D2)。代わりに同じテストで `lib.rs` の末尾の `#[cfg(test)] mod tests;` と、`src/tests/` の外に `#[path` が無いことを確かめ、`unsafe`・ネットワークの検査は `src/tests/` も対象に残した。検査の範囲は `code-summary.md` に記録した(BR3.2)。
- 2026-11-24T16:00:00Z — `flow_init.rs` の期待のひな形を原本の写しで書いたので、ひな形を変えるときは `src/tests/init_template.rs` と `tests/flow_init.rs` の 2 か所の原本を直す(D4 の代償)。

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
