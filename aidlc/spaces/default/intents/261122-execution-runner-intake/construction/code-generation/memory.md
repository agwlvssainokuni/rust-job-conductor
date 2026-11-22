<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->
- 2026-11-23T00:00:00Z — N3〜N5 の「直す前」は、Step 8 の前の木(c49ff0b)にテストを一時的に足して走らせ、結果を記録してから Edit で外した(stash や reset は使わない)。N3 は TooLarge で落ち、N4・N5 は通った。
- 2026-11-23T00:00:00Z — T14 の `CommandLine` は、runtime の tests/common に補助を足さず、各呼び出しの前で `check_command(&j.command).expect("分けられる")` で作った(承認の一覧に無い common の変更を避けるため)。

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->
- 2026-11-23T00:00:00Z — Step 2 で、T1〜T12 の一覧に無いテストの変更(`flow_sequences.rs` の `優先順位_中断と起動失敗とシグナル` と、読み方だけの 9 か所)が要ったため、コミットせずに止めて報告した。ユーザーの承認を受けて T19・T20 として functional-spec.md §4 に足し、続けた。
- 2026-11-23T00:00:00Z — classify の内部の `signal_exit` の引数を `Option<i32>` から `i32` にした(中断の番号が型で必ずあるため。返す値は同じ)。
- 2026-11-23T00:00:00Z — language-agnostic の 05 §1.2 は、手順 1 と同じ節の `start_failure` の行も直した(手順 1 と食い違わないため)。

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->
- 2026-11-23T00:00:00Z — `split.rs` の `command_line` は `check_command` に畳まずに残した(呼ぶのは `check_command` だけになった)。差分を小さく保つため。
- 2026-11-23T00:00:00Z — 偽物の `ScriptedRuntime` は、台本の結果を `ExecutionResult` に写してから `JobExecution::new` で作る形にした。`StartFailed` には実行中の合図を足さない(本物と同じ)。

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
- 2026-11-23T00:00:00Z — Windows の `ディレクトリは_unreadable` と、Linux での N3・N5 は、CI で確かめる(ローカルは macOS だけ)。
