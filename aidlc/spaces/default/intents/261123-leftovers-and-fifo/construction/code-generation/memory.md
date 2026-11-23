<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->
- 2026-11-23T00:00:00Z — functional-spec §8 の並びのとおりに 45 件のコミットを積んだ(§8 のコミット 23 は進めなかった)。各コミットの前に fmt・build・clippy(-D warnings)・全テストを通した。件数は段 1 の後 1,156、段 2 の後 1,161 で、見込みと一致した。テスト名の差は §4.6 の 4 つの削除、N1〜N8 の追加、★ T23 の名前の変更、T4 の置き場所の移動だけ。
- 2026-11-23T00:00:00Z — 直す前の性能(P1〜P3)は、テストのファイルを直す前の木に置いて測り、直すコミットの間は `target/lf/` に退避した(stash・reset は使わない)。段 2 の N4〜N7 の直す前の確かめは、`git worktree add --detach target/lf/before aeaec20` にファイルのツールでテストを書き足して走らせ、確かめたあと worktree を消した。
- 2026-11-23T00:00:00Z — 1h-4・1h-5(D8)の前後の時間と、§4.6 の閉じない形の確かめは、一時のテストで行い、測ったあと消した(コミットしていない)。作業の一時ファイルは `target/lf/` に置いた。

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->
- 2026-11-23T00:00:00Z — 1g-4(ConversionError.reason を &'static str に)は進めなかった。T 表の外の converter tests/nfr_security.rs:328 の `shown.contains(&e.reason)` が clippy::needless_borrows_for_generic_args で止まり、そのテストを変えないと進めないため(指示どおり止めて報告)。T19 も行っていない。
- 2026-11-23T00:00:00Z — 1c-4c で record.rs の製品の import から Classification を外したため、同じファイルの単体テストの import の 1 行に Classification を足した(T2 の対象のファイル。中身は変えていない)。このコミットは clippy の失敗を見落として一度作り、直して --amend した(push 前)。以後、確認のスクリプトはどれかが失敗したら止まるようにした。
- 2026-11-23T00:00:00Z — 1a-2 の注記は、コードのコメントに版の番号を書かない指示に従い、版の番号なしにした。
- 2026-11-23T00:00:00Z — P2 の入力は誤りの位置の順を行で確かめるため 1 行に 1 つの遷移にした(1,000,146 バイト)。

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->
- 2026-11-23T00:00:00Z — 1h-5 の record_skip は crate の中の関数なので、前後の時間は同じ処理(複製する形と借りる形)を一時のテストで並べて測った(162.8 ms → 97.5 ms)。

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
- 2026-11-23T00:00:00Z — 1g-4 を進めるには、converter tests/nfr_security.rs:328 の `&e.reason` を `e.reason` にする変更の承認が要る。
- 2026-11-23T00:00:00Z — Linux・Windows での全テストと、新しい性能のテストの CI での時間(上限との余裕が 3 倍を切らないか)は CI で確かめる(ローカルは macOS だけ)。
