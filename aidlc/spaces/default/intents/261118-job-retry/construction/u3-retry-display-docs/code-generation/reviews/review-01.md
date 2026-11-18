## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-18T22:36:19Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | crates/cli/tests/docs_retry.rs > 全テスト(README の知らせの引用) | README が引用する知らせの英語・日本語の文は、`crates/cli/src/report/messages.rs` の `retry_scheduled` の書式と一致している(目視で確認した)。ただし、docs テストはこの引用を検査しないため、u2 側の文言が将来変わっても README の引用が古いまま通る | 任意。`retry_scheduled` の出力を README の引用と比べる検査を足す、または引用を更新する運用を記録する | New |
| R-02 | Minor | README.md > リトライの動き > リトライの前の知らせ(英語の例)と code-summary.md の注記 | 英語の知らせは、待ち時間が 1 秒のとき `in 1 seconds` になる(`messages.rs` が単数・複数を分けないことを確認した。code-summary の注記は正しい)。README の例は 20 秒で、この点に触れていない | 対応は不要(既知で範囲外)。直すときは、`report_messages.rs` のテストと README の引用を、一緒に更新する | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| cargo test -p conductor-cli --test docs_retry --locked | PASS(4 passed) | README の例の TOML を実際の `load_definition` に通し、5 つの値を検査している。小節や例が消えると、`expect` で失敗する。既存の dev-dependency(`tempfile`)だけを使う |
| cargo fmt --all -- --check | PASS | 整形の差分なし |
| cargo clippy --workspace --all-targets --locked -- -D warnings | PASS | 警告なし |
| cargo test --workspace --locked | PASS(全 `test result: ok`、失敗 0) | 既存のテストは緑のまま |
| git diff acd80e4 HEAD --stat -- . ':!aidlc' | README.md と crates/cli/tests/docs_retry.rs の 2 ファイルだけ | ソースコード、既存のテスト、クレートの追加はない。source-manifest.json の `writes` はこの 2 つと一致する |

**確認した README の主張(コードとテストに対して)**

- 範囲: `exit_codes` は 1〜255 で、重複は昇順整理と `dedup` でまとまる。`max` は 1〜100(`walk.rs`)。`multiplier` は 2〜10。`interval` と `max_interval` は 1 秒〜24 時間。`max_interval < interval` は誤り。`multiplier` と `max_interval` は片方だけだと誤り。成功・警告の終了コードとの重なりも誤り。
- リトライの条件(`flow/retry.rs`): 起動していて、中断でなく、起動の失敗でなく、シグナル終了でなく、自身の終了コードが一覧にあるときだけ。ジョブ自身の 127 は一覧にあれば対象になる。合計は最大 `max + 1` 回。
- 待ち時間: 1 回目は `interval`、以降は直前 × 倍率で `max_interval` が上限。例の 10/20/40/60/60 は `max = 5` で合っている。
- 使い切り: 最後の結果をそのまま返すため、通常のエラーと同じ。通知は `run_job_with_retry` で `-v` に関係なく待機の前に出す。待機の中断は、起動なし・中断の実行結果を返すので 128+番号になり、エラーとして記録されず、再開位置はそのジョブ。待機は 50ms ごとに確認するので、1 秒以内は成り立つ。リトライの回数はローカル変数で、ステートに書かない。
- `[job.retry]` の位置の記述は、「`id`・`command` などのあと、`[[job.transitions]]` の前に書きます」という書き方の案内であり、後ろに置いても TOML として読めるため、「必須」と読める強さではない。
- 古い一文「自動の再試行はしません」は、「定義に `[job.retry]` を書かなければ」の条件つきに直っている。終了コードの表の 64 と 128+番号の追記、保証の範囲、シグナルの節の追記も、コードと合っている。
- CI の docs-check の 7 語(SHA-256、Gatekeeper、終了コードの表、ステートファイルの場所、at-least-once、シグナル、convert)は README にあり、テストでも確認している。MSRV の記述は差分に含まれない。

### Summary

README の新しい説明は、検査したすべての事実(範囲、条件、待ち時間、通知の文言、中断の扱い、ステートの扱い)がコードとテストと一致し、古い一文も正しく条件づけられている。docs テストは意味があり、全体のテスト・fmt・clippy も通るため、ブロックする指摘はなく READY とする(Minor 2 件のみ)。Linux/Windows の CI は、ローカルで確認できない。
