# コード生成の計画: u6-attempt-times(試行ごとの時刻と所要時間)

## Summary

- Builds: ステートの各ジョブに `[[job.attempt]]`(`started_at`・`ended_at`)を試行ごとに書き、`watch` の表に所要時間の列(`17s`・`1m05s`・`2h03m04s`)を出す。試行の記録の無い動いたジョブのステートは 65 と `reset` の案内
- Touches: core の `types.rs`・`flow/attempt.rs`(新規)・`flow/mod.rs`・`time.rs`・`lib.rs`、runtime の `state/reader.rs`・`state/writer.rs`・`error.rs`、cli の `step.rs`・`aux_commands.rs`・`report/watch.rs`・`report/messages.rs`、テスト(固定のステートに試行を足す直しを含む)、`README.md`、デモの `watch` の場面
- Tests: 単体・統合テスト約 30(試行の関数、所要時間と書き方、読み書き、3 つの理由)、非機能のテスト 2(`nfr_content.rs`、表示に値が出ない)、流れのテスト 6、実バイナリのテスト 3、デモ

出典: 機能の設計(`functional-spec.md`、`rules.md` の BR1.1〜BR5.2、`entities.md`)、非機能の設計(`reliability-design.md`、`security-design.md`、`performance-design.md`、`scalability-design.md`、`observability-design.md`、`logical-components.md`)、ユニットの定義(`unit-of-work.md` の U6)、要件(`requirements.md` の FR6.1〜FR6.9、FR8.1)、契約(`contract-summary.md` の C4、C5)、設計の決定(ADR-004)。ユーザーストーリーは作っていないため、各ステップは要件と規則の ID に対応させる。

テストの進め方は Testing Contract(test-after)に従う。core → runtime → cli の順に、層を作ってからその層のテストを書いて走らせる。

## データの形の変更

- `Attempt { started_at: Timestamp, ended_at: Option<Timestamp> }`、`JobState.attempts: Vec<Attempt>`(`entities.md`)。
- ステートファイルの `[[job.attempt]]`(`started_at` 必須、`ended_at` 任意)。`format_version` は 1 のまま(C4)。
- runtime の `reason` に `NO_ATTEMPT`、`BAD_TIME`、`UNFINISHED_ATTEMPT`(`entities.md`)。
- 表示だけの `JobDuration`(`Finished(秒)`・`Running(秒)`)。`messages.rs` の `HEADER_DURATION`。

## 設計のレビューからの持ち越し(このユニットで扱う)

- ログを作れずに起動しなかった試行は「足して閉じる」(`reliability-design.md` の 2。状態が `error` になるため。非機能の要件の NFR1.7 の「足さない」はこの設計で改めた)。起動の前に合図が立っていた試行(`NotStarted`)は `cancel_attempt` で取り除く。
- 所要時間は `reliability-design.md` の 4 の表のとおり。`success`・`warning`・`error` なのに最後の試行に `ended_at` が無い(手で書き換えたなど。conductor は書かない)ときは空にする(非機能の設計のレビュー)。
- 開始の書き込み(Fast)の表示の文脈は「起動前」(`StateContext::JobNotStarted`)、リトライの前の書き込み(Fast)は「起動した」(`StateContext::JobExecuted`)(非機能の設計のレビュー)。
- 今のテストの固定のステートで、`success`・`warning`・`error` のジョブを持つものには `[[job.attempt]]` を足す。直すのはデータだけで、確かめる内容は変えない(非機能の要件のレビュー)。
- 所要時間の関数は core に置き、U7 も使う(機能の設計のレビュー)。

## ステップ

### Step 1: テストの準備を確かめる

- [x] 変更の前に、`unit-test-instructions.md` の 3 つのコマンドを走らせ、結果を記録する。

### Step 2: 型と試行の関数、所要時間(core)

- [x] `types.rs` に `Attempt` と `JobState.attempts` を足し、`JobState::pending` と組み立ての箇所を合わせる(BR1.1)。
- [x] `flow/attempt.rs` に `start_attempt`(閉じ忘れの最後の試行を自身の開始の時刻で閉じてから足す)、`end_attempt`、`cancel_attempt` を足し、`lib.rs` から公開する(BR1.1〜BR1.4)。`apply_outcome`・`apply_skip` が `attempts` を保つことを確かめる。
- [x] `time.rs` に `job_duration(job, now) -> Option<JobDuration>`(4 の表)と `format_duration(秒)` を足す(BR4.1、BR4.2)。

### Step 3: core の層のテストを書いて走らせる

- [x] 型の変更に合わせて、テストの補助と組み立ての直しだけをする。
- [x] 新しい `crates/core/tests/attempt_flow.rs`(3 つの関数、閉じ忘れの試行、`apply_outcome` と `apply_skip` が試行を保つ)と、`time_format.rs` への追加(`format_duration` の 0、59、60、3599、3600、3661 秒。`job_duration` の 4 の表の各行、終わりが始まりより前で 0)。
- [x] core のコマンドを走らせる。

### Step 4: ステートの読み書き(runtime)とその層のテスト

- [x] `writer.rs` の `job_table` に、試行があるときだけ `attempt` を書く。`reader.rs` の `JOB_KEYS` に `attempt` を足し、試行の表を読む(知らないキー、欠け、型、`BAD_TIME`、`UNFINISHED_ATTEMPT`、`NO_ATTEMPT`)。`error.rs` に 3 つの理由の文(BR2.1、BR2.2、BR3.1、BR3.2)。
- [x] 固定のステート(`state_read.rs` の `VALID` など)に試行を足す。`state_read.rs`(3 つの理由、知らないキー、最後の試行の終わりが無くても読める)、`state_write.rs`(書いて読み戻すと等しい、試行が無ければ今と同じ形、試行 1,000 個)、`nfr_content.rs`(時刻の 2 つのキーだけを許す)に足す。
- [x] runtime のコマンドを走らせる。

### Step 5: 流れと表示(cli)

- [x] `step.rs` の `run_job_with_retry` が記録中のステートを `&mut PipelineState` で受け取り、試行ごとに `start_attempt` → Fast で書く → 起動 → 結果ごとに `end_attempt` か `cancel_attempt` → リトライなら Fast で書く、とする。書き込みの失敗は `Err(65)`。`run_step` は空のステートの初期化の書き込みを、最初の開始の書き込みにまとめる(BR1.1〜BR1.3、NFR1.3)。
- [x] `aux_commands.rs` の `run_watch` が `ctx.runtime.now()` を 1 回取り、`watch.rs` の `render_table` に渡す。`watch.rs` に所要時間の列、`messages.rs` に `HEADER_DURATION`(英日)(BR4.1、BR4.2)。

### Step 6: cli の層のテストを書いて走らせる

- [x] 固定のステートを使う cli のテストに試行を足す。`watch` の表の期待に所要時間の列を足す。
- [x] `flow_run.rs`・`flow_pipeline.rs`・`flow_retry.rs`(書く順と回数、時刻が偽の `now()` の値、起動の失敗・ログを作れない・起動の前の合図・リトライの待ちの中断の各場面)、`flow_aux.rs`(`watch` の所要時間、動いている最中の経過時間、古い形のステートで 65 と案内)、`flow_graph.rs` と `flow_dry_run.rs`(古い形のステートで 65 と案内)に足す。
- [x] `binary_basic.rs` か新しい実バイナリのテストに、ステートに時刻が書かれること、古い形のステートで 65 と `reset` の案内、`watch` の列を足す。
- [x] cli のコマンドを走らせる。

### Step 7: デモと README

- [x] デモの `watch` の場面と `demo/README.md` を合わせ、`unix_demo.rs`・`windows_demo.rs` で所要時間の列は形だけを確かめる。`unix_demo` を走らせる。
- [x] `README.md` に、ステートの `[[job.attempt]]`、所要時間の列、動いている最中の経過時間と強制終了の後の扱い(`conductor reset` か再実行で直る)、v0.9.9 までのステートは 65 で `reset` が要ることを書く。`cargo test -p conductor-cli --lib --locked -- tests::docs` を走らせる。

### Step 8: 書式と静的検査

- [x] `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通ることを確かめる(NFR7.1)。

## 規則と要件の対応

| ステップ | 規則 | 要件 |
|---|---|---|
| Step 2 | BR1.1〜BR1.4、BR4.1、BR4.2 | FR6.1、FR6.5、FR6.9 |
| Step 4 | BR2.1、BR2.2、BR3.1、BR3.2 | FR6.2〜FR6.4、FR6.6〜FR6.9 |
| Step 5 | BR1.1〜BR1.3、BR4.1、BR4.2 | FR6.1、FR6.5 |
| Step 3、4、6 | すべて | FR6.1〜FR6.9、NFR1〜NFR6 |
| Step 7 | BR5.1、BR5.2 | FR8.1 |
| Step 8 | | NFR7 |

## コミットの分け方

小さな単位でコミットする(件名は日本語、先頭に `core:`・`runtime:`・`cli:`・`docs:`)。各コミットで fmt と clippy が通るようにする。例: Step 2 は core の 2〜3 コミット、Step 3 のテスト、Step 4 は runtime の読み書きとテスト、Step 5 は流れと表示、Step 6 のテスト、Step 7 は `docs:`。型の変更で壊れる組み立ては、型を変えるコミットで一緒に直す。版は上げない。

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
      "text": "- **Methodology**: test-after\n- **Methodology evidence**: チームは test-after を続けると決めた(インタビュー Q3)。履歴も整合する。v0.9.0 の U4 では、`create_file` の追加(602d15c)の後にそのテスト(90a27fa)、ひな形の生成(0fbd466)の後にひな形のテスト(5b750f8)が続く。U3 も流れ(a9eb237)の後に流れのテスト(365f45c)と実バイナリのテスト(062c627)が続く。`code-quality-assessment.md` も「チームの方針(test-after、行カバレッジ 80%)と一致する」と記す。\n- **Ordering**: 各テスト可能な層を実装してから、その層のテストを書いて実行する。\n- テストの種類: 各クレートの `tests/` に統合テスト(core 33、runtime 14、cli 25、converter 2 のファイル)。単体テストは各モジュールの `mod tests` と、`crates/cli/src/tests/`(11 ファイル)、`crates/converter/src/tests/`(4 ファイル)。`#[test]` は合計 1,170。\n- 流れのテストは偽の `Runtime`/`Sink`/`Env`/`Signals` を注入する。実バイナリのテストは `conductor` とテスト専用の補助バイナリを動かす。非機能のテスト(`nfr_*.rs`、`*_perf.rs`)、網羅のテスト(`ALL` 配列)、参照出力との一致(`graph_layout_oracle.rs`)、文書との一致(`src/tests/docs.rs`、`msrv_consistency.rs`)、デモのテスト(`unix_demo.rs`、`windows_demo.rs`)を含む。テスト名は日本語。\n- テストの都合で本番のコードを曲げない。テスト専用の引数・`pub`・関数を本番に残さず、流れのテストは公開の入口と差し替え口で書く(インタビュー Q6)。\n- 行カバレッジ 80% を下限とする。CI の `coverage` ジョブが `cargo llvm-cov --fail-under-lines 80` で強制する(テスト専用の補助バイナリは除外)。\n- 主な開発環境の OS は macOS で、テストはローカルの macOS で実行する。他の OS のテストは GitHub Actions の CI(`test`: ubuntu、macos、windows)で確認する。マージ前に CI を通す(インタビュー Q3)。\n- 最小の Rust(1.100.0)でもテストを実行する(`msrv-test`)。\n- 起動の性能(`nfr_perf.rs`)は `startup-bench` で測るが、警告のみで失敗にしない(`STARTUP_BENCH_ENFORCE=false`、`continue-on-error`)。\n- v0.10.0 の機能にも、test-after と 80% の下限、単体テストと実バイナリを動かす統合テストを続ける(インタビュー Q3)。\n- ステートの中身を調べるテスト(`nfr_content.rs`)は、B6 のために開始と終了の時刻だけを許すように直す。それ以外(コマンド・環境変数・出力)は今までどおり検査する(インタビュー Q7)。"
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
  "input_sha256": "sha256:ed0086a4c967568436c54943aebacd308a988af279eee16ed426cf4f39a154fa",
  "contract_sha256": "sha256:3789717991ddc007f6cbca5d56d6b2d0b248de4ce1061637170c6787b5bf88e9"
}
```
