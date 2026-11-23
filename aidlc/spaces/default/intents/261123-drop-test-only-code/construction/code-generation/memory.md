<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->
- 2026-11-24T00:00:00Z — functional-spec §8 の並び 1〜24 のとおりに 24 件のコミットを積んだ(25 はコミットなしの確かめ)。各コミットで fmt・clippy(-D warnings)・全テストのコンパイル(--no-run)と、変えたクレートのテストを、失敗で止まる確認のスクリプト(target/drop-fd/check.sh。set -eo pipefail)で通した。最後に全テストを実行し、1,153 通過・1 無視・0 失敗で、見込みと一致した。
- 2026-11-24T00:00:00Z — テスト名の一覧の前後の差は §4.6 の 4 つの削除だけ。§7 の検索はすべて期待どおり(#[default] は GraphFormat の 1 件、cli の pub mod は exit・report・run_id の 3 件、ほかは 0 件)。cargo doc の警告は 5 件のまま、Windows の対象の clippy は通り、行カバレッジは 98.08%。
- 2026-11-24T00:00:00Z — 作業のファイル(基準と後のテストの出力、名前の一覧、確認と検索のスクリプト)は target/drop-fd/ の下にだけ置き、記録のあと消した。

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->
- 2026-11-24T00:00:00Z — 仕様書 rust 02 の 3.2 のシーケンスの runtime::apply_skip・initial_state・apply_outcome・apply_interrupt の 4 行を core:: に直した(inventory の 3b の行の一覧に無いが、再公開が無くなって誤りになるため。仕様書だけ)。
- 2026-11-24T00:00:00Z — core の types.rs:440 と plan/mod.rs:18 の説明のふつうの文字の decide_before_run は、crate の中の関数を指して正しいので変えなかった(inventory の一覧にも無い)。

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->
- 2026-11-24T00:00:00Z — 2a の decide.rs の公開の説明は、非公開の関数へのリンクを「`decide_before_run`(このモジュールの中だけの関数)」のふつうの文字にして、cargo doc の警告を増やさないようにした。

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
- 2026-11-24T00:00:00Z — Linux・Windows での全テストの実行は CI で確かめる(ローカルは macOS だけ。Windows はクロスの clippy だけ通した)。
