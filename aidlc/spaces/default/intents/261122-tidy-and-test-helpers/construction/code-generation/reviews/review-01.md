## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-23T00:00:00Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | crates/cli/tests/common/mod.rs:status_in(T9 の移行先)、crates/cli/tests/flow_retry.rs の 5 か所 | flow_retry.rs の旧 `status_of` はジョブが無いとき `Pending` を返したが、共通の `status_in` は `expect("ジョブがある")` で失敗する。`assert_ne!(…, Error)` の 2 か所は、ジョブが記録に無い場合に以前は通り、今は失敗する。確かめる中身は強くなる方向の変化で、現在のテストは通っている(code-summary の ★ の承認の範囲)。ただし code-summary の「確かめる中身は変えていない」の文は、この点では厳密には正しくない。 | code-summary の T9 の行に、「ジョブが無いときは Pending ではなく失敗にした(厳しくなる方向)」と一言足す。コードの変更は要らない。 | New |
| R-02 | Minor | crates/cli/tests/docs_support/mod.rs:block_after_heading、aidlc/.../code-generation/code-summary.md(ずれ 5) | docs_retry.rs の `expect("TOML の例がある")` は共通の `expect("例がある")` になり、見出しが見つからないときの文も「リトライの設定の小節がある」から「見出し `…` がある」に変わった。どちらも失敗のときだけ出る文で、合否と期待の文字列は同じ。ずれ 5 は T6 の「既定で進めるもの」の延長として妥当。 | 対応は不要。承認の記録に、ずれ 5 が T6 の既定の範囲であることが残っていれば足りる。 | New |
| R-03 | Minor | crates/cli/src/report/mod.rs:render(code-summary のずれ 4。メモの番号は 3 と書かれる場合あり) | `render` が言語を必要な枝でだけ判定するようになり、`Graph`・`Plan`・`InitTemplate` などでロケールを読まなくなった。差分を読む限り、各枝が出す文字列は元と同じ(`SKIPPED_LIST` を `SKIPPED_WORD` に置き換えた遷移行も、両者とも `skipped`/`スキップ` で同じ)。`Transition` の枝は `lang()` を 2 回呼ぶが、`language` が 1 回だけ判定して保つため、結果は同じ。ロケールを読む回数が減るのは、利用者から見える出力の変化ではない。 | 対応は不要。依頼文の「ずれ 3」と code-summary の「ずれ 4」の番号の食い違いだけ、記録で揃えるとよい。 | New |
| R-04 | info | crates/core/src/flow/decide.rs:decide_before_run(規則 5 の削除) | 旧規則 5(`nothing_started && index == 0` のとき `Run`)は、規則 1〜4 を通った時点で `status` が pending になるため、`index == 0` の規則 6(前のジョブが無く、`status == Pending`)に含まれる。削除後も同じ結果になる。ドキュメントコメントの規則 5 の行にもその旨がある。 | 対応は不要。 | New |
| R-05 | info | crates/core/src/flow/classify.rs:classify_exit_code、crates/core/src/plan/simulate.rs:simulate_job | `classify_exit_code` は旧 `classify` の手順 5 の if 連鎖と同じ順序・同じ値(成功 0、警告 0、非成功の 0 は 1 と `conductor_error`、他は元の終了コード)。simulate は `unwrap_or(Error)` をやめ、`exit_codes.contains` を `is_retry_target` にしたが、`JobExecution::exited` で作った実行は `Some(code)` を持つため結果は同じ。 | 対応は不要。 | New |
| R-06 | info | crates/runtime/src/temp_file.rs、crates/runtime/src/state/writer.rs、crates/runtime/src/file_create.rs | 一時ファイル名は `.<名前>.tmp.<pid>.<nanos>.<seq>` の形のまま(`temp_name` と、`temp_prefix` を介した後始末の接頭辞)。連番の `SEQUENCE` が 2 つから 1 つになった(設計の D5)。名前が重なったときの 16 回の再試行も同じ。`dir_or_current` の移動で、`reset_all.rs`・`writer.rs`・`file_create.rs` の import も解決している。 | 対応は不要。 | New |
| R-07 | info | crates/runtime/src/state/reader.rs:FailureCause、inspect_run_state | `FailureCause::RunIdMismatch` は `walk` の `run_id` 不一致だけで立つ。`reason == RUN_ID_MISMATCH` の比較を型の比較に置き換えたもので、`reset --all` の `NotOurs`/`Broken` の振り分けは同じ。 | 対応は不要。 | New |
| R-08 | info | crates/cli/src/report/messages.rs、crates/core/src/run_id.rs、crates/core/src/envfile/mod.rs | `error_prefix!`・`env_name_rule!`・`reserved_env_prefix!` の `concat!` は、元の文字列と文字単位で同じ(`conductor: error: internal error`、`…failed to set up signal handling`、`名前は、英字または `_` で始まり、英数字と `_` だけで書きます`、``` `CONDUCTOR_` で始まる名前は、書けません ```)。`SKIPPED_LIST`・`NOTE_BY_*`・`PLAN_*`・`GRAPH_*` の統合も、日英の文字列は同じ。`plan_end` の置き換えは、`PlanEnd` の 3 つの値(`completed`/`reached @end (completed)`/`stopped (error: …)` と日本語版)を同じ文字列で出す。 | 対応は不要。 | New |
| R-09 | info | crates/cli/src/args.rs:operand_rule、map_unexpected_value | `operand_rule` は旧 `wants_positional`/`allows_operand` と同じ分類(Run・Pipeline・Convert は必須、Validate・Graph・Init は省略可、Watch・Reset は不可)。`lexopt::Error` の `MissingValue` の枝を消したあとの守りの枝は、旧 `_ => InvalidValue` と同じ結果。 | 対応は不要。 | New |
| R-10 | info | git diff 37210ca..HEAD の全体(制約) | 新しい依存なし(`Cargo.lock`・各 `Cargo.toml` に差分なし。版は 0.9.6 のまま)。足した行の `unsafe`・`cfg(unix\|windows)` は `signal_support` のドキュメントコメントの 1 行だけ。仕様書の差分は `docs/spec/rust` の 6 ファイルだけで、凍結した go/python/typescript/csharp/powershell と language-agnostic は変更なし。converter から core への依存の追加も無い(`opposite` は converter 内の型に足している)。 | 対応は不要。 | New |
| R-11 | info | aidlc/.../code-generation/code-summary.md(計画からのずれ 1〜8) | ずれ 1(`Valid` が位置を持たない)・2・3(名前の変更)・6(`#[cfg(test)] use`)・8(`--amend`)は動きに影響せず妥当。ずれ 7(§8 に無い仕様書の 3 行の追加)は、名前と置き場所を合わせる直しで、`docs/spec/rust` の範囲内。 | 対応は不要。 | New |
| R-12 | info | aidlc/.../code-generation/traceability.json | `coverage` の 55 件(SM1〜6、BR1.1〜9.4)の `target` はすべて実在するファイル。`upstream_ids` と `coverage` の ID は過不足なく一致し、設計側にも存在する。`reverse` は空の配列。 | 対応は不要。`reverse` を空にしておく取り決めなら、そのままでよい。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
| --- | --- | --- |
| traceability.json の存在確認(自作の照合) | PASS: target の欠落 0、ID の過不足 0 | 実在する |
| git diff 37210ca..HEAD の読み取り(core・runtime・cli の主な変更、messages・render・decide・classify・temp_file・reader・args、テストの共通化) | 動きの変化なし | 規則・文字列・名前の形は同じ |
| cargo test・clippy・llvm-cov | 実行していない(レビューは差分の読み取りで行った) | code-summary の 1,146 通過・カバレッジ 98.14% は、報告をそのまま引用したもので、この review では再確認していない。判定を左右する指摘は無い |

### Summary

利用者から見える動き(文言、終了コード、ステート、`--dry-run` の出力)は、読んだ差分の範囲で変わっていない。文言の定数の統合、`decide` の規則 5 の削除、`classify_exit_code`、`temp_file`、`FailureCause`、`render` の変更は、どれも元と同じ結果になる。既存のテストの変更は T1〜T46 の範囲で、ずれ 4(`docs_retry.rs` の `expect` の文)は失敗のときだけの文のため承認の範囲として妥当。制約(新しい依存なし、unsafe と cfg の追加なし、凍結した仕様書に差分なし、版は同じ)も守られ、traceability.json の ID と target も実在する。残るのは、T9 の `status_in` が厳しくなる点の記録だけで、READY とする。
