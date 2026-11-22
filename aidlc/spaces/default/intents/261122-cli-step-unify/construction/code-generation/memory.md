<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->
- 2026-11-22 — `record_skip` の土台は `Option<&PipelineState>` で受け、中で `cloned()` する(まとめる前の `clone().unwrap_or_else(initial_state)` と同じ)。`run_step` の土台は計画どおり値で受ける。
- 2026-11-22 — `pipeline.rs` の `finish` は、名前と役割を残し(仕様書の呼び出しの木が `finish` を含むため)、中身を `remove_if_requested` の呼び出しにした。

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->
- 2026-11-22 — Step 2 のコミットだけ `mod step;` に `#[allow(dead_code)]` を付け、pipeline のコミットで外した。共通の手順の単独のコミットでも `clippy -D warnings` を通すため(`StepResult.recorded` を読むのは pipeline だけ)。
- 2026-11-22 — テストの候補「pipeline で、記録の変わらないスキップで完了しているときに消す」は到達できない枝(読み込んだ完了のステートは空として返り、ループ内の完了は直ちに終わる)なので、到達できる、通っていなかった枝(pipeline のスキップでの完了と削除、表示と削除の順序、削除の失敗)のテストに替えた。

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->
- 2026-11-22 — 表示と削除の順序(pipeline は表示のあと)は、`Sink` と `Runtime` の記録が別々で直接は並べられないため、削除を失敗させ、標準エラーの中で `skipped` の行が誤りの行より前にあることで確かめた。

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
