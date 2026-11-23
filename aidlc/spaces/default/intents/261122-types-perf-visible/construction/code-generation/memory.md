<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->
- 2026-11-23T00:00:00Z — 37 件のコミットを functional-spec §8 の並びのとおりに積んだ。各コミットの前に fmt・build・clippy・全テストを通した。件数は段 1 の後 1,149、段 2 の後 1,154、段 3 の後 1,157 で、見込みと一致した。テスト名の差は N1〜N8・N10〜N12 の追加だけ。
- 2026-11-23T00:00:00Z — N2 の固定の表は、コミット 12 では `to_string()`(変更前の `#[error]` の文)と比べ、`label()` が入るコミット 16 で `label()` と比べる形に替えた。文を移す前に表で固定し、移したあとも同じ表で 1 文字も違わないことを確かめるため。
- 2026-11-23T00:00:00Z — 直す前の性能は、各テストのファイルを直す前の木に置いて測った。直すコミットの間はテストのファイルを `target/tpv/` に退避し(stash・reset は使わない)、テストのコミットで戻した。直す前の木では P3・P4・P5 が時間の上限で落ち、結果の中身はすべて通った。
- 2026-11-23T00:00:00Z — 作業の一時ファイル(件数、テスト名の一覧、検索のスクリプト)は `target/tpv/`(.gitignore の対象)に置いた。2e の一時の測定のテストは測ったあと消した。

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->
- 2026-11-23T00:00:00Z — 整形は `cargo fmt --all` で当てた(内容の変更はファイルのツール)。N1 のテストの追加だけ、誤ってシェルのヒアドキュメントで書き足した。以後はファイルのツールで行った。
- 2026-11-23T00:00:00Z — `apply_outcome_in` の手がかりは `Option<usize>` にした(公開の関数は `None` で、すべての探索を先頭から行い、今と同じ結果を保つ)。
- 2026-11-23T00:00:00Z — T21・T22・T23 のファイルで、使われなくなった変数・引数を `_` で始め(`runner_basic.rs:108`、cli の 2 つ目の偽物)、`runner.rs` の単体テストに import を 1 行足した。確かめる中身は変えていない。
- 2026-11-23T00:00:00Z — `plan_next_step` はエスケープの関数を受け取る形に、`check_job`・`check_pipeline` は `pub(crate)` に、`classify.rs` の内部の補助 `outcome` は `job` の引数なしにした。
- 2026-11-23T00:00:00Z — §7 の検索で 0 件にならないもの 2 つ(読み取りの層の `RawRetry.multiplier: Option<u32>`、`Runtime` の関数の引数の名前 `definition_path:`)は前からあり、目的の二重は消えている。

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->
- 2026-11-23T00:00:00Z — `flow_edges` は遷移の無い定義でも位置の表を毎回作る。そのため P3(`build_graph` の中で `flow_edges` を呼ぶ)は 2b の後の 72 ms から 2c の後に約 97 ms へ増えた。上限 350 ms の中で、直す前(369 ms)の約 1/4。表を遅らせて作る工夫は設計に無いので入れなかった。

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
- 2026-11-23T00:00:00Z — Linux・Windows での全テスト(特に新しい性能のテストの時間と、Windows で数えない N11・N12)は CI で確かめる(ローカルは macOS だけ)。
