# Code Generation Plan — u2-retry-execution(リトライの実行と、途中の知らせ)

## Summary

- Builds: 設定(u1 の `RetryPolicy`)に従って、対象の終了コードで終わったジョブを、間隔を空けて最大 N 回実行し直す(`run` と `pipeline` の共通の処理)。固定の間隔と、倍率と上限の方式、待機中のシグナルでの即時の中断、リトライの前の知らせ(常に標準エラー、英語と日本語)
- Touches: `crates/core/src/flow/retry.rs`(新規)と `flow/mod.rs`・`lib.rs`、`crates/runtime/src/wait.rs`(新規)と `lib.rs`、`crates/cli/src/deps.rs`(`Runtime` に「待つ」)、`crates/cli/src/retry.rs`(新規)、`crates/cli/src/run.rs`、`crates/cli/src/pipeline.rs`、`crates/cli/src/report/mod.rs` と `report/messages.rs`、`crates/cli/tests/common/mod.rs`(偽の `Runtime` 2 つに「待つ」)、新規のテスト 3 ファイル
- Tests: 新規の統合テスト 3 ファイル(core の判断、runtime の待機、cli のリトライの流れ。合わせて約 45 件)と、cli の文言のテストへの追加。既存の約 755 件は変更なしで通す(偽の `Runtime` への追加を除く)

## 根拠

- 設計: `construction/u2-retry-execution/functional-design/` の `functional-spec.md`、`rules.md`(BR2.1〜BR2.16)、`entities.md`
- 要件: FR1.2 と FR1.6 の動き、FR2.1、FR2.2、FR3.1〜FR3.6、FR4.1、FR4.2、FR5.1、FR5.2、FR6.1、FR6.2 の最初の文言、NFR3、NFR7(`requirements.md`)
- 既存のコード: `cli/src/run.rs`、`cli/src/pipeline.rs`、`cli/src/deps.rs`、`core/src/flow/classify.rs`、`runtime/src/types.rs`(`InterruptSignal`)、`report/mod.rs`(`DisplayEvent`)
- 前提: u1 の `JobDefinition::retry: Option<RetryPolicy>` が使える(完了済み)
- チームの方針: test-after、`cargo fmt` と `cargo clippy -D warnings`、行カバレッジ 80%、変更は小さく、こまめにコミットする

## 名前を付けて追加するデータの要素

設計(`entities.md`)が求めるものだけを足す。

- `RetryDecision`(`crates/core/src/flow/retry.rs`): `Stop` と `Retry { wait_secs }`(BR2.1〜BR2.3)
- `WaitOutcome`(`crates/runtime/src/wait.rs`): `Elapsed` と `Interrupted(signal_number)`(BR2.5、BR2.13)
- `DisplayEvent::RetryScheduled { job_id, last_exit_code, retry_number, max, wait_secs }`(BR2.4、BR2.15)
- `Runtime::wait(secs, signal) -> WaitOutcome`(BR2.13)
- `RetryState`(リトライの途中の状態: `retries_done`、`last_wait_secs`)は、ステートファイルに記録せず、共通の処理の中のローカルな値として持つ(BR2.10)
- ステートファイルの形式、`PipelineState`、`JobState` は変えない

## 実装ステップ(test-after: 各層を実装してから、その層のテストを書いて実行する)

- [ ] Step 1: テスト実行環境を確認し、変更前の基準を取る。`cargo test --workspace --locked` が全件通ることを記録する。コマンドは `unit-test-instructions.md` に記載
- [ ] Step 2: 純粋な判断(業務ロジックの層)の実装。`core/src/flow/retry.rs` に、`retry_decision(job, execution, retries_done, last_wait_secs) -> RetryDecision` を書く。対象の判定は、起動した、中断でない、起動失敗でない、シグナル終了でない、終了コードが設定の集合に含まれる、の 6 つの条件(BR2.1、BR2.16)。回数の判定(BR2.2)。待ち時間は、固定と、倍率と上限(1 回目は `interval`、以降は直前 × 倍率、上限で頭打ち、桁あふれは飽和)で計算する(BR2.3)。`core` の公開面(`lib.rs`)に出す
- [ ] Step 3: 純粋な判断のテストを書いて実行する。`crates/core/tests/retry_flow.rs`(新規)。対象と対象外の全パターン(設定なし、起動失敗、シグナル終了、中断、起動前の中断、一覧にない終了コード、成功・警告、ジョブ自身の 127)、回数の境界(`max` 回目まで Retry、その次で Stop)、待ち時間の列(固定、10→20→40→60→60 の例、上限で頭打ち、極端に大きい値でも桁あふれしない)。`cargo test -p conductor-core --locked`
- [ ] Step 4: 待機(runtime の層)の実装。`runtime/src/wait.rs` に、`wait_for(secs, signal) -> WaitOutcome` を書く。標準ライブラリだけを使い、100 ミリ秒以内の間隔で合図を確かめながら待つ。合図が立っていれば、すぐ `Interrupted(番号)` を返す。`runtime` の公開面に出す(`unsafe`、OS 別の `cfg`、新しい外部クレートは使わない)(BR2.13、BR2.14)
- [ ] Step 5: 待機のテストを書いて実行する。`crates/runtime/tests/wait_interruptible.rs`(新規)。待機が経つと `Elapsed`、待ち 0 秒ですぐ `Elapsed`、すでに合図が立っていればすぐ `Interrupted`、長い待ち(3600 秒)の途中で別のスレッドが合図を立てると、1 秒以内に `Interrupted` を返す(NFR7)。`cargo test -p conductor-runtime --test wait_interruptible --locked`
- [ ] Step 6: 差し込み口(cli の層)の実装。`deps.rs` の `Runtime` に `wait` を足し、`SystemRuntime` は `wait_for` を呼ぶ。`cli/tests/common/mod.rs` の偽の `Runtime`(`ScriptedRuntime`、`MemRuntime`)に `wait` を足す(待たずに、呼ばれた待ち時間を記録し、指定の回数目に `Interrupted` を返せるようにする)。ビルドが通ること
- [ ] Step 7: 表示の実装。`report/mod.rs` の `DisplayEvent` に `RetryScheduled` を足し、レベルは「常に標準エラー」、描画を書く。`report/messages.rs` に、英語と日本語の固定の文を足す(job_id は既存のエスケープを通し、整数と固定の文だけで組み立てる)(BR2.4、BR2.15)。表示のテスト: `crates/cli/tests/report_messages.rs` に、英語と日本語の文、`-v` なしで出ること(常に標準エラー)、入力の値が出ないことを確かめる 4 件程度を足し、`cargo test -p conductor-cli --test report_messages --locked`
- [ ] Step 8: 流れ(cli の層)の実装。`cli/src/retry.rs`(新規)に、共通の処理「ジョブを、リトライを含めて実行する」を書く: 実行 → 判断 → (Retry なら)知らせ → 待つ → (Elapsed なら)回数を 1 増やして繰り返す、(Interrupted なら)「起動していない、中断された」実行結果を作る。結果は最後の実行結果を返す。`run.rs` と `pipeline.rs` の `run_job` の呼び出しを、この処理に置き換える。結果の表示(`report_execution`)とステートの記録は、最終の結果について、これまでどおり 1 回だけ行う(BR2.7〜BR2.12)
- [ ] Step 9: 流れのテストを書いて実行する。`crates/cli/tests/flow_retry.rs`(新規)。偽の `Runtime` で、2 回目に成功、使い切ってエラー(最後の終了コード、再開位置)、対象外(起動失敗、シグナル終了)でリトライしない、リトライの設定がないジョブが従来どおり、倍率と上限の待ち時間の列、待機中のシグナルで 128 + 番号と再開位置と「エラーとして記録しない」、`pipeline` でも同じ、ステートが最終の結果で 1 回だけ書かれる(途中で書かれない)、知らせが `-v` なしで標準エラーに出る、を確かめる。`cargo test -p conductor-cli --test flow_retry --locked`
- [ ] Step 10: 全体の確認。次をすべて実行して通ること。`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked`、検査コマンド(`cargo llvm-cov ... --fail-under-lines 80`)。`cargo tree` で `-sys` と C のビルド依存、新しい外部クレートが増えていないこと。既存のテストのアサーションが変わっていないこと(偽の `Runtime` への `wait` の追加は、アサーションの変更ではない)
- [ ] Step 11: 文書と記録。コードの文書コメント(日本語、要件 ID つき)を書き、`code-summary.md`、`source-manifest.json`、`traceability.json` を作る

## 触れないもの(この Unit の外)

- ステートファイルの形式と、`runtime/src/state/*`
- `README.md` と `demo/`、表示の文言の最終の確定(U3 の担当)
- 定義の読み取りと検証(u1 で完了)
- 新しい外部クレートは、追加しない

## 要件との対応

| 要件 | ステップ |
|---|---|
| FR1.2、FR1.6 | 2、3、8、9(設定なしは従来どおり) |
| FR2.1、FR2.2 | 2、3、9 |
| FR3.1、FR3.2、FR3.3、FR3.4、FR3.6 | 2、3、8、9 |
| FR3.5 | 8、9(`run` と `pipeline`) |
| FR4.1 | 4、5、8、9 |
| FR4.2 | 9(既存のシグナルのテストが通る)、10 |
| FR5.1、FR5.2 | 8、9、10 |
| FR6.1、FR6.2(最初の文言) | 7、8、9 |
| NFR3 | 6、9(偽の `Runtime` で待たずに試す) |
| NFR7 | 4、5 |

## Assumptions

- 偽の `Runtime` が 2 つ(`ScriptedRuntime`、`MemRuntime`)ある。`wait` の追加は、この 2 つと `SystemRuntime` だけで済む見込み(`impl Runtime for` の検索で確認済み)
- 実際の時間を使う統合テスト(実プロセスで間隔 1 秒を待つ)は、時間がかかり、OS ごとの違いも出るため、この Unit では書かない。待機の実時間の確かめは、Step 5 の短い待機のテストと、偽の `Runtime` を使う Step 9 で行う
- `-v` の有無による表示の差は、既存の表示の仕組み(レベル)に合わせる。「常に標準エラー」は、既存のエラーのレベルと同じ出力先と条件で、文言はエラーとして扱わない

## Testing Contract

```json
{
  "version": 1,
  "methodology": "test-after",
  "source": "team",
  "ordering": "各テスト可能な層を実装してから、その層のテストを書いて実行する。",
  "scope": "feature",
  "test_strategy": "standard",
  "project_type": "brownfield",
  "applicable_notes": [
    {
      "layer": "org",
      "text": "We treat tests as a first-class deliverable in every Bolt. The specific\nmethodology (TDD, BDD, ATDD, or classic test-after) is affirmed at\npractices-discovery and recorded in `team.md` under this heading with explicit\n`Methodology` and `Ordering` fields; Code Generation resolves those fields\nindependently from coverage, tooling, and scope notes.\n\nWhen no posture has been affirmed, our default per scope is:\n- **Methodology**: test-after\n- **Ordering**: implement each applicable testable layer, then write and run\n  that layer's tests.\n- `mvp`, `enterprise`, `feature`, `infra` add an 80% line-coverage\n  floor and CI execution before merge.\n- `bugfix`, `security-patch` add a targeted regression for the specific\n  bug/vulnerability and require the existing suite to remain green.\n- `express` uses the Minimal strategy: requirement-driven unit tests (one per\n  requirement, with a happy-path floor per component); existing tests remain\n  green.\n- `poc`, `refactor`, `workshop`, `classic` add no extra new-test floor and require the\n  existing suite to remain green.\n\nThe active `Test Strategy` still applies in every scope and determines test\nvolume/types. Scope floors are additive; they never reduce or replace the\nselected strategy.\n\nBuild and Test verifies defined coverage floors and affirmed quality targets;\nthey may not be weakened to make a step pass.\n\nAffirm a stricter posture in `team.md` if the team commits to one."
    },
    {
      "layer": "team",
      "text": "- **Methodology**: test-after\n- **Methodology evidence**: インタビュー Q3 で、チームは test-after(実装してからテストを書く)を選んだ。\n- **Ordering**: 各テスト可能な層を実装してから、その層のテストを書いて実行する。\n- `cargo test` による単体テストと、バイナリを実行する統合テストを含める。\n- 行カバレッジ 80% を下限とする(インタビュー Q4: A)。\n- 主な開発環境の OS では、ローカルでテストを実行する。それ以外の OS(Linux・macOS・Windows のうち該当するもの。制約 C3)のテストは、GitHub Actions の CI で実行する。マージ前に CI を通す。"
    }
  ],
  "obligations": {
    "strategy": "standard",
    "strategy_volume": [
      "Five to eight tests per component.",
      "Unit tests plus integration tests for key boundaries.",
      "Add E2E, performance, or security tests when requirements demand them."
    ],
    "scope_floor": [
      "Meet an 80% line-coverage floor.",
      "Run the selected tests in CI before merge."
    ],
    "combination_rule": "Apply every selected-strategy obligation and every scope-floor obligation; neither replaces the other, and a targeted scope regression may add the narrowest necessary test type beyond the strategy default."
  },
  "plan_profile": {
    "methodology": "test-after",
    "runner_step": "Verify the existing test runner/configuration and record the exact unit-scoped command.",
    "runner_ready_before_first_test": true,
    "testable_layers": [
      "Data model / database behavior",
      "Repository / data access",
      "Business logic",
      "API / endpoint",
      "Frontend behavior"
    ],
    "steps": [
      "Project structure and production configuration skeleton.",
      "Verify the existing test runner/configuration and record the exact unit-scoped command.",
      "Data model / database behavior - implement.",
      "Data model / database behavior - write and run its tests after implementation.",
      "Repository / data access - implement.",
      "Repository / data access - write and run its tests after implementation.",
      "Business logic - implement.",
      "Business logic - write and run its tests after implementation.",
      "API / endpoint - implement.",
      "API / endpoint - write and run its tests after implementation.",
      "Frontend behavior - implement.",
      "Frontend behavior - write and run its tests after implementation.",
      "Environment/build configuration.",
      "Documentation and traceability."
    ]
  },
  "input_sha256": "sha256:d5b3fba95d247c0e12f7b2d508688f21b918b16d18e551864079baef0ef0ea0b",
  "contract_sha256": "sha256:d612e34b5f9d41c4f21a42502a2a8d564d0dfa60fd65fe4f577e5714d8511806"
}
```
