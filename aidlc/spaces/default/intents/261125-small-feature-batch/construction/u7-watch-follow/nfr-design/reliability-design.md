# 信頼性の設計: u7-watch-follow

出典: 信頼性の要件(`reliability-requirements.md` の NFR1.1〜NFR1.5、NFR2.1〜NFR2.3、NFR3.1〜NFR3.3)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 2、3、3.1、5)、契約(`contract-summary.md` の C4、C5)。

## 1. 終わりの判定(core。`crates/core/src/flow/activity.rs`)

- `pipeline_activity(state: &PipelineState) -> PipelineActivity`。判定の順は機能の仕様の 3 の表(完了 → 動いている試行 → 止まった → 間)。
- 確かめ方: 各値の単体テスト。リトライの待ちの間の形(最後の試行が開いている、`status` が `pending` または `error`)は `Running`。

## 2. リトライの待ちの間の試行(cli の `step.rs`。BR3.4)

- `run_job_with_retry` は、試行の終わりの時刻を `pending_end: Option<Timestamp>` に持つ。リトライするときは書かずに待つ。
- 次の試行の始め: `end_attempt(pending_end)` → `start_attempt(now)` → Fast で 1 回書く(文脈は起動前)。
- 待ちの途中の中断: `end_attempt(pending_end)` をしてから抜け、`run_step` の最後の Durable の書き込みに入れる。
- 確かめ方(非機能の要件のレビューの指摘): `flow_retry.rs` で、(1) 待ちの間に書かれたステートでは最後の試行が開いている、(2) 次の試行の書き込みで前の試行の `ended_at` が試行の後の `now()` の値、(3) 待ちの途中の中断の後の Durable のステートで、最後の試行の `ended_at` が持っていた時刻、(4) リトライ 1 回で Fast 2 回・Durable 1 回。

## 3. 継続表示の流れ(cli の `aux_commands.rs`)

```text
signal = 合図の準備(失敗なら 70)
seen = false、absent_once = false、read_failures = 0
繰り返し:
  合図 → 0
  読む:
    Ok(Some(state)) → seen = true、absent_once = false、read_failures = 0、now()、描く
                      Completed か Stopped → 0
    Ok(None)        → seen かつ absent_once → 0(Removed)
                      seen → absent_once = true(次の回でもう一度見る)
                      でなければ「ステートが無い」を描く
    Err(Unreadable) → read_failures += 1。3 回続けば 65。でなければ何も描かずに次の回へ
    Err(ほか)       → 誤りの表示、65(壊れた、古い形、大きすぎる)
  wait(interval, signal) が中断 → 0
```

- **読み込みの一時的な失敗(非機能の要件のレビューの指摘)**: `run` の書き込みは一時ファイルからの置き換えで、UNIX 系では読み手から途中の形は見えない。Windows では置き換えの瞬間に開けないことがあるため、開けない・読めない(`Unreadable`)は 3 回続くまで次の回に読み直す。消えたと判定するのも、2 回続けて無いときだけにする。
- 確かめ方: 偽の `Runtime` の読み込みの並び(有る → 無い → 有る で終わらない、有る → 無い → 無い で 0、読めない 2 回 → 有る で続く、読めない 3 回で 65、壊れたで 65)。

## 4. テストの配置

| テストの場所 | 確かめること |
|---|---|
| `crates/core/tests/` の新しいファイル | `pipeline_activity` の 4 つの値 |
| `crates/cli/src/tests/args_grammar.rs` | `--follow`、`--interval` の正しい場合と 68 |
| `crates/cli/tests/flow_aux.rs` | 上の 3 の並び、待ちと合図、描く回数、書き込みが 0 回 |
| `crates/cli/tests/flow_retry.rs` | 上の 2 |
| `crates/cli/tests/binary_*.rs` | 完了のステートで 1 回描いて 0、引数の誤りで 68、パイプに制御の文字が出ない |
| `src/tests/docs.rs` | 使い方の表の一致と必須の語句 |
| `unix_demo.rs`、`windows_demo.rs` | デモの場面(完了のステートを `--follow` で見て終わる) |

- 本番にテスト専用の項目を足さない。待ちと時刻と読み込みは差し替え口の偽物で動かす。
- 版を 0.10.0 にするコミットでは、版に関わるテスト(`--version`、README との一致)を同じコミットで合わせる。
