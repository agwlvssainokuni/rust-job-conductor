<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->
- 2026-11-23T00:00:00Z — Step 5 の「65 文字以上のジョブ ID の graph の text の出力」は既存のテストに無かったため、新しいファイル `crates/cli/tests/graph_long_id.rs`(2 件)で確かめた。件数の見込み +7 が +9(1,137)になった。
- 2026-11-23T00:00:00Z — A3b の入力は、`[job].env_file` を逆の順に並べた(BR4.3 の「最後に足した位置」を確かめるため)。§5 の見積もり(同じ順)と直す前の時間が違う(1,704 ms 対 2,166 ms)。
- 2026-11-23T00:00:00Z — 性能の「直す前」は、足すテストを直す前の木で走らせて落ちたときの値を使った(§5.3 の前後の記録の手順どおり)。

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->
- 2026-11-23T00:00:00Z — コミット 15(`.env` の解析の重複の除去。任意)を見送った。上限の入力でも 40.6 ms で、性能のテストも無い。
- 2026-11-23T00:00:00Z — 段 3 のコミット 29 で `args::resolve_run_id` を移行の再公開として残し、段 4 のコミット 31(T7)で消した。各コミットでテストを変えずに通すため。
- 2026-11-23T00:00:00Z — `flow_with_state.rs` は、ほかのテストの呼び出しを変えないよう、`invoke_inner` と `pipeline` を今の引数のまま既定の指定で包み、`remove_on_complete` を渡す入口(`invoke_call`・`invoke_with`・`pipeline_with`)を足した。inventory 4d の `pipeline(scene, definition, remove_on_complete)` とは形が違う。
- 2026-11-23T00:00:00Z — T2 の `Rejected` は、要素 1 つのループが clippy の `single_element_loop` に触れるため、1 回の呼び出しにした(組み立ての値だけで、期待の文字列は同じ)。

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->
- 2026-11-23T00:00:00Z — `apply_outcome` は `<JobStatus as From<Classification>>::from(classification)` と書いた。`JobStatus::from(classification)` だと `Classification` の import が非テストのコードで使われなくなり、T6 の範囲の外の単体テストの import を変える必要が出るため、少し冗長な完全修飾の形を選んだ。
- 2026-11-23T00:00:00Z — graph の text の行き先は、段 2 では `EdgeTarget::End => TransitionTarget::END_LABEL`、段 4 で別名になってから `escape_path(to.label())` にした。遷移の表示(`escape_name`、64 文字で切る)と graph(`escape_path`、切らない)の使い分けは保った。
- 2026-11-23T00:00:00Z — `dir_or_current` は新しいファイルを作らず `runtime/src/state/mod.rs` に置いた。`platform/unix.rs` から `crate::state::dir_or_current` を呼ぶ(`unsafe` の個所と `cfg` には触れない)。

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
