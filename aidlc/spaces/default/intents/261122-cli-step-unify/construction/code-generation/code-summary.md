# コード生成の要約(Code Summary): run と pipeline の重複をなくすリファクタリング

計画 `code-generation-plan.md` の Step 1〜8 を、計画の順に、小さなコミットで `main` に積んだ(push はしていない)。利用者から見える動き(出力、終了コード、ステートの書き込みの順序・`Durability`・文脈・内容)は変えていない。

## コミット

| コミット | 内容 |
|----------|------|
| f9b9523 | 共通の手順: `crates/cli/src/step.rs` を新規に作り、`decide`、`record_skip`、`run_step`、`remove_if_requested`、`StepResult` を置く(BR1.1、BR1.2) |
| b76be14 | run: `run_individual` を、共通の手順を呼ぶ形にする(BR1.2、BR2.1) |
| 2e60886 | pipeline: `run_pipeline` を、共通の手順を呼ぶ形にし、`finish` は `remove_if_requested` を使う(BR1.2、BR2.1) |
| 738f3cd | テスト: 今のテストで通っていなかった、削除の時機の枝を 3 本足す(BR1.2、BR2.1) |
| d9f3823 | 仕様書: `docs/spec/rust/05-execution-flow.md` に §5.1a を足し、§5.2・§5.3 をそれを指す形に。`02-architecture.md` のモジュールの表と呼び出しの木(BR3.1) |

## 変えたファイル

- `crates/cli/src/step.rs`(新規): 1 つのジョブの共通の手順。
  - `decide(job_id, definition, state: Option<&PipelineState>) -> Decision`
  - `record_skip(ctx, session, base: Option<&PipelineState>, job_id, reason) -> Result<(PipelineState, bool), i32>`(`bool` は土台から変わったか)
  - `run_step(ctx, session, base: Option<PipelineState>, job, mode) -> Result<StepResult, i32>`(`base` は値で受ける)
  - `remove_if_requested(ctx, session, context) -> Result<(), i32>`
  - `StepResult { recorded, outcome, next, after }`
- `crates/cli/src/run.rs`: 共通の手順を呼ぶだけにした。Skip は `changed` かつ完了のときだけ、表示の前に消す。Run は、`job_index` の取り出しを `run_step` の前に置き、完了なら消してから `exit_code_for`。
- `crates/cli/src/pipeline.rs`: 共通の手順を呼ぶだけにした。ループ、Skip の表示のあとの完了(記録が変わらなくても)、エラーか分類なしでの停止、完了、ジョブのあいだの中断、「完了を判定できない」は残る。`current.take()` を `run_step` に渡し、`StepResult.recorded` を `current` に戻す。`finish(ctx, session, context)` は `remove_if_requested` を呼んで 0 か 65。
- `crates/cli/src/lib.rs`: `mod step;` と、モジュールの表のドキュメントコメントに `step` を足した。
- テスト: `crates/cli/tests/flow_run.rs`(1 本)、`crates/cli/tests/flow_pipeline.rs`(2 本)。既存のテストは 1 行も変えていない(追加だけ)。
- 文書: `docs/spec/rust/05-execution-flow.md`、`docs/spec/rust/02-architecture.md`。ほかの仕様書、README は変えていない。

## 足したテスト

| テスト | ファイル | 確かめること |
|--------|----------|--------------|
| `br1_2_記録の変わらないスキップは_remove_on_complete_でも消さない` | `flow_run.rs` | run は、記録の変わらないスキップ(`AlreadyExecuted`)では、`--remove-on-complete` でも書かず、消さない |
| `fr2_1_スキップで完了して_remove_on_complete_なら消して_0` | `flow_pipeline.rs` | pipeline で、スキップで完了したとき、記録を書いてから消し、0 |
| `br1_2_スキップで完了したときは_表示のあとで消し_削除の失敗は_65` | `flow_pipeline.rs` | pipeline は、`skipped` の表示のあとで消す。削除の失敗は 65 で、文脈は「起動前」 |

## 確認

- 変更の前: `cargo test --locked -p conductor-cli` は 440 件が通過。
- 変更の後: 同じコマンドで 443 件が通過(既存の 440 件 + 新規の 3 件)、失敗 0。各コミットで、このコマンドが通ることを確かめた。
- `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`: 通過。
- `cargo test --locked --workspace`: 1,121 件が通過、失敗 0(macOS)。
- `Cargo.toml`、`Cargo.lock` は無変更。`unsafe` と `cfg(unix)`/`cfg(windows)` は足していない。
- 行カバレッジの全体のしきい値は、Build and Test で確かめる。`cargo llvm-cov` の行の表示では、`step.rs` で通らない行は `record_skip` の「土台が空なら初期ステート」の 1 行だけ(空のステートでは Skip にならないので、まとめる前から通らない枝)。

## 計画からの逸れ

- **Step 2 のコミットだけ、`mod step;` に `#[allow(dead_code)]` を付けた**: 共通の手順を単独でコミットすると、まだ誰も呼ばないので未使用の警告が出る(`clippy -D warnings` で止まる)。各コミットを警告なしに保つため、Step 2 で付け、Step 4(pipeline)のコミットで外した(`StepResult.recorded` を読むのは pipeline だけなので、run のコミットでは外せなかった)。
- **テストの候補の差し替え**: 計画の候補「pipeline で、スキップの記録が変わらないのに完了しているときに消すこと」は、通らない枝だった。読み込んだ完了のステートは空(`None`)として返り、ループの中で完了すれば直ちに終わるので、土台が完了していることがなく、変わらないスキップで完了することがない。そこで、通っていなかった、到達できる枝(pipeline のスキップでの完了と削除、表示と削除の順序、削除の失敗)を確かめるテストにした。run の候補(変わらないときに消さない)は、そのまま足した。
  根拠: `prepare` の 6(`docs/spec/rust/05-execution-flow.md` §5.1 の「完了のまま残ったものは「空」」)。
- **`run` の `job_index` の取り出しの位置**: 初期ステートの書き込み(`run_step` の中)の前に移った。検査済みのジョブIDなので、見つからない枝は通らず、動きは変わらない(設計の §3 のとおり)。
- 新しいテストは、計画の「2〜3 本」の上限の 3 本。
