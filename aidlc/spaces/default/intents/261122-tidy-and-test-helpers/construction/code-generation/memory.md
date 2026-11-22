<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->
- 2026-11-23T00:00:00Z — 49 件のコミットを §8 の並びのとおりに積んだ。各コミットの前に fmt・clippy・全テストを通した。件数は段 1 の後 1,141、段 3 の後 1,142、段 5 の後 1,143、段 7 の後 1,146 で、見込みと一致した。
- 2026-11-23T00:00:00Z — 文言の定数をまとめたところは、テストに加えて、組み立てた文が元の文字列のままバイナリにあることを `grep -a` で確かめた(出力は 1 文字も変えない)。
- 2026-11-23T00:00:00Z — N3 の「合図で起動しないとき」は、`SignalMode::RaiseOnStart(15)` で `run_job` が合図の立った状態で呼ばれる場合とした(待機の中断のあとは `run_job` が呼ばれないため)。
- 2026-11-23T00:00:00Z — 作業の一時ファイル(件数の記録、テスト名の一覧、検索の結果)は、プロジェクトの中の `target/tidy-work/`(.gitignore の対象)に置いた。

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->
- 2026-11-23T00:00:00Z — 6a の `SuccessCodes::Valid` は位置を持たない形にした(位置はどこからも読まれず、clippy の dead_code で止まったため)。
- 2026-11-23T00:00:00Z — 5b の関数は `operand_rule`、4b の関数は `map_unexpected_value(option: &str)` にした(名前の紛れと、扱う範囲に合わせるため)。
- 2026-11-23T00:00:00Z — 4c で言語を必要な枝だけで判定した結果、`Graph`・`Plan`・`InitTemplate` と、スキップの無い `Transition` でもロケールを読まなくなった。出力は同じ。
- 2026-11-23T00:00:00Z — T6 の `expect` の文は、`docs_retry.rs` の「TOML の例がある」も共通の「例がある」になった(失敗のときだけの文)。
- 2026-11-23T00:00:00Z — 仕様書は §8 の行のほか、02 章の `read_context`・`report/mod.rs` の行、05 章の `simulate_job`、02 章の `TargetFormat` も直した(名前と置き場所を合わせるため)。
- 2026-11-23T00:00:00Z — コミット 10 は fmt の差を見落としてコミットしたため、直して `--amend` した(push 前)。以後、fmt・clippy が通らなければ止まる手順にした。

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->
- 2026-11-23T00:00:00Z — `build_retry` は参照を受け取る形のまま残した(6b の範囲は `build_jobs` だけで、差分を小さく保つため)。
- 2026-11-23T00:00:00Z — 4d の let-else の else(到達しない)は、リトライしないのと同じ結果を返す形で残した。カバレッジの行が 1 つ増えるが、行カバレッジは 98.14%。

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
- 2026-11-23T00:00:00Z — Linux・Windows での全テスト(特に `signal_support` を使う UNIX のテストと、Windows でコンパイルされる `temp_file`)は CI で確かめる(ローカルは macOS だけ)。
