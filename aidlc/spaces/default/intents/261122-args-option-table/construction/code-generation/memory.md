<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->
- 2026-11-22 — `spec(opt)` を添字で引くため、`OPTIONS` の行を `Opt` の並びと同じ順に置き、`const _: () = { ... }` でコンパイル時に順を検査した。`src/` に `#[cfg(test)]` を置かない(`cfg` を足さない)制約と、cli の `src/` に `expect`/`unwrap` が無い今の書き方に合わせるため。
- 2026-11-22 — 誤りの対象の綴りは `Opt::subject()` にまとめた。`Verbose` だけ `-v` にする `match` は、仕様の「`-v` だけ短い形」を写したもので、オプションごとの分岐ではないと解釈した。
- 2026-11-22 — `CHECKS` のあとの値の検査の関数(`parse_assumptions` など)の対象の文字列は、範囲の外として今のまま残した。`--to` の対象だけ `Opt::To.subject()` にした。

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->
- 2026-11-22 — Step 3 で、使える場所の条件(`takes_verbose` などの `matches!`)も `OptSpec::allows` で表から読む形にした。`allowed` の列が Step 4 まで読まれないと `clippy -D warnings` の dead code で止まるため。`if` の並びの置き換えは計画どおり Step 4。
- 2026-11-22 — Step 4 と Step 5(確かめだけ)を 1 つのコミットにした。
- 2026-11-22 — Step 2 の「`--assume` の不可が `--dry-run` の重複より先」はテストにできない(同時に起きない)。`--format`・`--state` の不可より先であることで代えた。字句の段階の対象の綴りと、字句の誤りと知らないオプションの順のテストも足した(計画の例より広い)。
- 2026-11-22 — `functional-spec.md` 5 の「`args_grammar.rs`(106 件)」は実際の 49 件と合わない。全体の基準 1,117 は一致した。

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->
- 2026-11-22 — `args.rs` は 538 → 627 行に増えた(表の行を `rustfmt` が 1 項目 1 行にするため)。行数より、オプションの性質が 1 か所に集まることを優先した。
- 2026-11-22 — オプションごとの指定は `Vec<Vec<Option<OsString>>>`(`Given`)にした。値を取らないオプションも `None` を積むので、数は並びの長さで一様に扱える。

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
- 2026-11-22 — `--to` の誤りの表示の変更で版を上げるかは、リリースのときにユーザーが決める(版の更新の候補として `code-summary.md` に記録)。
- 2026-11-22 — 行カバレッジ 80% の確認は Build and Test で行う(この段階では測っていない)。
