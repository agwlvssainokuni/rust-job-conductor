# 機能の仕様(Functional Spec): run と pipeline の重複をなくすリファクタリング

対象: `crates/cli/src/run.rs`(`run_individual`)と `crates/cli/src/pipeline.rs`(`run_pipeline`、`finish`)。意図: `ideation/intent-capture/intent-statement.md`(Q1: B、Q2: A、Q3: A)。利用者から見える動きは 1 文字も変えない。

## 1. 今の重複(読んで確かめた)

| 手順 | `run.rs` | `pipeline.rs` | 同じか |
|------|----------|---------------|--------|
| ステートから `(states, resume, end_reached)` を取り出し、`decide_before_run` を呼ぶ | 53-61 | 63-73 | 同じ |
| Skip: 土台(ステートか `initial_state`)に `apply_skip`、変わったときだけ `Durable` で書く(文脈は「起動前」) | 69-83 | 84-98 | 同じ |
| Run: ステートが空なら、初期ステートを `Fast` で書く(文脈は「起動前」) | 95-110 | 116-131 | 同じ |
| Run: `run_job_with_retry`、`classify_and_decide`(実行方式だけ違う)、`report_execution`、`apply_outcome`、`after` の文脈、`Durable` で書く | 117-136 | 133-152 | 同じ(`RunMode` が `Individual` か `Pipeline`) |
| 完了のあとの削除(`remove_on_complete` が真なら `reset`) | 84-89、137-142 | `finish`(193-207) | 削除の処理は同じ。**いつ消すかは違う**(§2) |

## 2. 残す違い(動きを変えないため、呼び出し側に残す)

1. **Skip のときの完了の扱い**: `run` は、記録が変わったときだけ、表示の**前に**消す。`pipeline` は、表示の**後に**、記録が変わらなくても `completed` なら消す(`finish`)。
2. **Run のあとの終了コード**: `run` は、完了なら消したうえで、常に `exit_code_for(報告の終了コード, エラーか)`。`pipeline` は、エラーか分類なしなら `exit_code_for` で止まり、完了なら `finish`(0)、そうでなければ中断の合図を見て、次のジョブへ進む。
3. **pipeline だけ**: ジョブのループ、ジョブのあいだの中断(BR4.2)、「完了を判定できない」(65)。
4. **run だけ**: ジョブIDの取り出しと、定義の中のジョブの位置の取り出し。

意図の記述は「完了の判断」も共通の手順に含めるとしていたが、上の 1、2 のとおり、いつ消すかは `run` と `pipeline` で違い、まとめると動きが変わる。そこで、**消す処理(`remove_on_complete` が真なら `reset`)を共通にし、いつ消すかの判断は、呼び出し側に残す**。

## 3. 新しい共通の手順(WF1。`crates/cli/src/step.rs`、新規)

| 関数 | 中身(§1 の同じ部分) | 戻り値 |
|------|----------------------|--------|
| `decide(job_id, definition, state: Option<&PipelineState>) -> Decision` | 状態の一覧、再開位置、`end_reached` を取り出して `decide_before_run` | 判断 |
| `record_skip(ctx, session, base: Option<&PipelineState>, job_id, reason) -> Result<(PipelineState, bool), i32>` | 土台(`base`、無ければ `initial_state`)に `apply_skip`、変わったときだけ `Durable` で書く | 記録後のステートと、土台から変わったか(`changed`。`run` は、これで削除するかを決める) |
| `run_step(ctx, session, base: Option<PipelineState>, job, mode: RunMode) -> Result<StepResult, i32>` | 空なら初期ステートを `Fast` で書く、`run_job_with_retry`、`classify_and_decide`、`report_execution`、`apply_outcome`、`Durable` で書く | `StepResult { recorded, outcome, next, after }` |
| `remove_if_requested(ctx, session, context) -> Result<(), i32>` | `remove_on_complete` が真なら `reset` | なし |

- 書き込みの失敗は、今と同じく、そのときの終了コード(65)を `Err` で返し、呼び出し側はそれで終わる。
- 書く順序、`Durability`、`StateContext` は、今と同じ。
- `run.rs` と `pipeline.rs` は、これらを呼ぶだけにする。`pipeline.rs` の `finish` は、`remove_if_requested` を使う形にする(または置き換える)。
- `run_step` の `base` は値で渡す。`run` は `session.state.clone()`、`pipeline` は `current.take()` を、呼び出し側で行い、`pipeline` は `StepResult.recorded` を `current` に戻す(今と同じ)。
- `run` の、定義の中のジョブの位置の取り出し(`job_index`。検査済みで、見つからない枝は通らない)は、`run_step` の前に移る。今は初期ステートの書き込みの後にあるが、通らない枝なので、動きは変わらない。

## 4. 仕様書(WF2)

`docs/spec/rust/05-execution-flow.md` の §5.2(`run`)と §5.3(`pipeline`)の擬似アルゴリズムを、共通の手順(§3 の 4 つ)を呼ぶ形に書き直し、共通の手順を新しい節に 1 回だけ書く。§2 の違いは、それぞれの節に残す。`02-architecture.md` は、モジュールの表に `step.rs` を足し、`pipeline.rs` の行(`finish`)と、呼び出しの木(180〜203 行あたり。`finish` を含む)を、新しい形に合わせる。ほかの仕様書(`docs/spec/language-agnostic` を含む)は、動きが変わらないので、変えない。

完了の基準: §5.2 と §5.3 が共通の手順を指し、同じ手順を 2 回書いていない。

## 5. テストの方針(Minimal の test-after)

- 既存のテストは、変えずに、すべて通る(`flow_run.rs`、`flow_pipeline.rs`、`flow_retry.rs`、`flow_run_id.rs`、`signals_order.rs` ほか)。
- 新規のテストの置き場所: 共通の関数は `pub(crate)` で、`tests/` から直接は呼べず、`Ctx` の偽物も要る。そこで、既存の流れのテスト(偽の `Runtime`/`Sink`/`Env`/`Signals` を注入する `flow_*.rs`)の形で、公開の入口から確かめる。今のテストで通っていない枝があれば、そこだけを足す(例: `pipeline` で、スキップの記録が変わらないのに完了しているときに消すこと、`run` で、変わらないときに消さないこと)。`decide` のような純粋な関数は、必要なら `step.rs` の中の `#[cfg(test)]` で確かめる。
- 合格の条件: `cargo fmt --check`、`clippy -D warnings`、全テスト、行カバレッジ 80% 以上、MSRV 1.100。

## 6. 成功の指標との対応(traceability.json の `SM1`〜`SM4`)

| ID | 成功の指標(`intent-statement.md`) | 対応する規則 |
|----|------------------------------------|--------------|
| SM1 | 1 つのジョブを処理する手順が 1 つの関数にまとまり、run と pipeline はそれを呼ぶだけ | BR1.1、BR1.2 |
| SM2 | 動きは 1 文字も変わらない。既存のテストを変えずに通る | BR2.1 |
| SM3 | 仕様書の実行の流れの章が、共通の手順として書き直される | BR3.1 |
| SM4 | 合格の条件(fmt、clippy、カバレッジ 80%、MSRV) | BR4.1 |
