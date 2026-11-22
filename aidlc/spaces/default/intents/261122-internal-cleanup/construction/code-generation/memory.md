<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->
- 2026-11-22 — 1d の「入口の共通の関数」は、`FlowTarget` を ctx.rs に置き、入口 `enter_flow(ctx, cmd, arg, FlowTarget::Run|Pipeline)` を dry_run.rs に置いた(`--dry-run` の振り分けを持つため)。位置引数をそのまま返すので、run 側に通らない枝が残らない。
- 2026-11-22 — 4a の主線の列 0 のテスト 2 件は graph_drawable.rs に寄せた(`graph_main_line.rs` は作らない。unit-test-instructions.md の 5 ファイルに合わせた)。N1 の遷移は、ヘルパーを graph_common に増やさず、その場で組み立てた。
- 2026-11-22 — 4b の converter のテスト名は、言語ごとの識別子(toml、yaml、utf8、null、mib など)と、`ac_7_1_x` の接頭辞を ASCII のまま残した。language-agnostic の 11 章の `convert_ac::ac_7_1_3` は接頭辞の参照として有効なので直していない。

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->
- 2026-11-22 — 3a で `git mv` の前に `mkdir -p crates/core/src/definition/walk` を実行した(無いディレクトリへは `git mv` できない)。
- 2026-11-22 — 4a で、移す先が 1 対 1 でない 5 つのテストのファイルを `git rm` で消した。
- 2026-11-22 — 2b で runtime の writer.rs の `mod tests` が `use super::*` で `JobStatus` を使うため、本体の側に `#[cfg(test)] use conductor_core::JobStatus;` を置いた(`mod tests` は変えていない)。

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->
- 2026-11-22 — 4a は 1 つのコミットにした(テストをファイルのあいだで移すので、途中の状態では件数が合わない)。4b は計画どおりファイル単位の 13 コミット。
- 2026-11-22 — 段 2・3 で足した core の関数(`as_str`、`parse`、`transition_on`、`JobExecution::exited`)に新しい単体テストは足していない。refactor の範囲で、既存のテスト(ステートの読み書き、表示、模擬)が同じ値を通して確かめているため。件数を基準のまま保ち、比べやすくした。

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
- 2026-11-22 — 凍結した 5 つの仕様書の 11 章は、converter の旧いテスト名を参照したまま(中身を変えない方針)。Build and Test で問題にしないか確かめる。
- 2026-11-22 — Windows で、日本語のテスト名を `--exact` で子プロセスに渡す `環境変数に左右されない` が通るかは、CI の windows で確かめる(ローカルは macOS で通過)。
