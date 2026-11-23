<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->
- 2026-11-23T00:00:00Z — functional-spec §8 の並び 1〜25 のとおりに 24 件のコミットを積んだ(14 はコミットなしの確かめ)。各コミットで build・clippy(-D warnings)・全テストを、失敗で止まる確認のスクリプトで通した。件数は段 1 の後 1,156、段 2 の後 1,157 で、見込みと一致した。テスト名の差は §4.6 の 5 つの削除、★ T1・T2・T30 の名前の変更、T29 の 2 件の置き場所の変更、N1 の追加だけ。
- 2026-11-23T00:00:00Z — 2b のステートファイルの比較は、`target/unbend-cg/` に同じ定義を置き、2b の前と後のバイナリで pipeline・run・run-id つきの run を実行して、3 つのファイルを cmp で比べた(すべて同じ)。P4 は 117.2 ms から 121.3〜132.3 ms(上限 600 ms)。作業のファイルは確かめたあと消した。
- 2026-11-23T00:00:00Z — D1 のコンパイル時の確かめは、core の写しの 2 つの要素を一時的に入れ替えて、cargo test --no-run が止まることを確かめ、すぐに戻した。

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->
- 2026-11-23T00:00:00Z — inventory に無かった rust の仕様書の記載(1g の 06・10 の ALL、1i の 02・05・10 の「登録済み」、2f の 05 の擬似コード)も、同じ項目の記載として合わせた(仕様書だけ)。
- 2026-11-23T00:00:00Z — language-agnostic の「すでに登録済み」の契約の文(02・05・07・10・12 章)は、承認済みの範囲の外なので変えなかった。11 章のテストの名前の欄(AC-285・AC-299・AC-348)だけを §8 の 25 として合わせた。
- 2026-11-23T00:00:00Z — runtime の tests/flow_with_state.rs:140 のコメントが、消した cli の step::decide を指したまま残る(T 表の外のテストのファイルなので変えていない)。

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->
- 2026-11-23T00:00:00Z — 2f の plan_run は、判断を decide_from_state にまとめたうえで、模擬の分類に渡す状態の一覧だけを Run の枝で取り出す(map_or)。判断の match の重複は残らない。

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
- 2026-11-23T00:00:00Z — (解決)language-agnostic の「すでに登録済み」の契約の文は、ユーザーの決定で直した(6dac72e。AC-299 は番号を残して該当なし)。runtime の flow_with_state.rs:140 のコメントも、ユーザーの決定で直した(e47877a)。
- 2026-11-23T00:00:00Z — Linux・Windows での全テストと P4 の CI での時間は CI で確かめる(ローカルは macOS だけ。Windows はクロスの clippy だけ通した)。
