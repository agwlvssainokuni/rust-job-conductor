<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->
- 2026-11-28T08:27:45Z — Step 7 の `check_options` は R-02 のとおり `given.count(Opt::RunId) > 0` で見る。`CHECKS` の後は 0 回か 1 回で、値を取るオプションはいつも `Some` を積むので、前の `run_id.is_some()` と同じ判定と解した(`args_grammar` の 69 のテストが通る)。
- 2026-11-28T08:27:45Z — 新しい子のファイルには、前例(`c34d3f7` の `retry.rs`・`env.rs`)と同じく、ライセンスと短い `//!` を付けた。移した項目の中身の行は変えていない。

- 2026-11-28T10:30:00Z — Step 12 の `render.rs` のメソッドは中身を値で受け取るため、枝の `*last_exit_code` などの `*`、`WatchFrame` の `now: *now`、`(*text)` を外した。文・改行・`format!` の形は変えていない。
- 2026-11-28T10:30:00Z — `lines.rs` の `place_of` は兄弟が使わないので、前例に合わせて private にした(設計の表は `pub(super)`)。

## Deviations
- 2026-11-28T10:30:00Z — Step 13 の FR4.4 で `ResetAllKept` を足すと `エラーの行は_conductor_error_で始まり_場所を含む` が落ちた(水準は Error、表示は warning)。止めて報告し、人の判断 A(2026-11-28)により、そのテストの例外の条件に `ResetAllKept` を加えた。
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->
- 2026-11-28T08:27:45Z — Step 5 の `git mv` の前に、移す先の `crates/cli/src/args/` を `mkdir` で作った。`git mv` は無いディレクトリへは移せないため。ファイルは書いていない。
- 2026-11-28T08:27:45Z — Step 7 の新しい関数の文書コメントは「手順 N:」で始めた。「4. …」で始めると、clippy の `doc_lazy_continuation` が番号つきの箇条とみなして落ちるため。

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->
- 2026-11-28T08:27:45Z — `args/mod.rs` は 341 行で、参考値の 340 行を 1 行超えた。新しい関数の文書コメントを削って合わせるより、手順の番号と誤りの種類の説明を残すほうを選んだ(R-01 で行数は参考値)。`code-summary.md` に実測を書いた。

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
