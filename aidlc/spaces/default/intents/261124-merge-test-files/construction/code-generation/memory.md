<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->
- 2026-11-24T22:31:59Z — 途中のコミット(1、6、9)では、説明と仕様書の行に、その時点で正しい中間の文を書き、最後のコミットの後で設計の §2 と inventory §7 の形にそろえた。各コミットで説明が無いファイルを指さないようにするため。
- 2026-11-24T22:31:59Z — §7 の「ひな形の生の文字列 0」の検索は、テストの写しが残らないことを確かめる意図と読んだ。結果の 2 件は本番の `src/report/init_template.rs` の定数(基準から同じ)で、本番を除くと 0。

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->
- 2026-11-24T22:31:59Z — コミット 9 で、次のコミットで消える統合テストの `tests/docs_{graph,dry_run,init}.rs` の説明の 1 行ずつ(単体テストのファイル名)を `src/tests/docs.rs` に直した(計画の §8 の 9 に無い。最後の形には残らない)。

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->
- 2026-11-24T22:31:59Z — `docs.rs` の補助は、元の本体のまま `pub` を残した(本体を変えない方針を優先。私的なモジュールの中なので外へは出ない)。

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
- 2026-11-24T22:31:59Z — D5 の最終確認(Windows の CI の `test` で、ひな形のデータのファイルが LF のままで `flow_init`・`init_template` が通るか)は未確認。push の後の CI で確かめる。
- 2026-11-24T22:31:59Z — `docs/spec/rust/12-rebuild-guide.md` 142 行目の「`docs_*.rs` が、必須の語句と例を検査する」は、S1〜S16 に無く §7 の検索にも当たらないので直していない。直すかを決めてほしい。
