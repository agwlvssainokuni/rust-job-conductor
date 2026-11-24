<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->
- 2026-11-25T12:00:00Z — 各コミットの確かめは、失敗で止まるスクリプト(`target/small-fixes/`。終わってから消した)で fmt、clippy(workspace、all-targets)、変えたクレートのテストを走らせ、単体・統合の件数を §8 の見込みと照らした。仕様書だけのコミット(3〜6)は §7 の該当の検索で確かめた。全テストと §7 の検索は最後に 1 回走らせた。
- 2026-11-25T12:00:00Z — 計画の「既知の限界を説明に書く」は、`nfr_source.rs` の確かめの前のコメントに書くと読んだ(`functional-spec.md` §2.4 のコードの形には無かった 2 行)。

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->
- 2026-11-25T12:00:00Z — 計画からのずれは無い。T1〜T4 の外の既存のテストを変える必要は出なかった。件数はすべてのコミットで見込みどおり(最後は 1,149 通過、1 無視。core 単体 112 / 統合 269、cli 単体 162 / 統合 294)。テスト名の差は消した 3 件だけ。細かな点(既知の限界のコメント、S4 の版の履歴の文の言い換え、S9 の括弧、T4 の一時的な確かめ)は `code-summary.md` の「計画からのずれ」に記録した。

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->
- 2026-11-25T12:00:00Z — T4 は空白を除いた部分一致にしたので、`include!` などの語が文字列や `/* */` のコメントの中にあっても失敗する(誤検出の側)。`#[cfg_attr(…, path = "…")]` の形は見逃す。今の製品のコードには、どの形も無い(D4)。
- 2026-11-25T12:00:00Z — T3 で、引数の解釈を通さずに `CliCommand::new(Init)` を直接呼ぶ確かめが無くなった。同じ等しさは `args_grammar.rs` が本番の解釈の経路で確かめる(`functional-spec.md` §4.6)。

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
