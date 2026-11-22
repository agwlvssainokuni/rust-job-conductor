<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->
- 2026-11-22 — Step 5 の「`plan_perf.rs` に足す」は、既存の `plan_perf.rs` にテストを 1 本追加すると解した。終点の後の形は、`run j0` の後(j0 が成功、`end_reached`、残りは pending)のステートを本番の関数で作り、999 個すべてが `apply_skip` を通る形にした。
- 2026-11-22 — 模擬の初期ステートのパスは、模擬の中だけの値として定数 `<dry-run>` にした(出力にも保存にも使わない)。

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->
- 2026-11-22 — 移した `record.rs` の単体のテストで、`use conductor_core::JobDefinition;` を `use crate::types::JobDefinition;` に 1 行だけ直した(core の中では `conductor_core` の名前が使えないため。テストの中身は無変更)。
- 2026-11-22 — `crates/runtime/src/lib.rs` の公開する関数の表で、`apply_*` の 3 行が段落の後に離れていたのを、表の中へ戻した(説明の修正の範囲)。

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->
- 2026-11-22 — `apply_skip` の位置で引く処理は、`decide.rs` の `status_at` を共有せず、`record.rs` の中の小さなクロージャにした(`decide.rs` を変えずに済み、差分が小さい)。
- 2026-11-22 — `simulate_job` の戻り値は、新しい型を足さず、4 つの組にした(計画の「新しい型は足さない」に合わせた)。

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
- 2026-11-22 — `docs/spec/{go,python,typescript}/02-architecture.md` にも `state/record` の記述が残る。計画の範囲(`docs/spec/rust` の 05、02、12)の外なので、変えていない。
