# コード生成の計画(Code Generation Plan): run と pipeline の重複をなくすリファクタリング

## Summary

- Builds: `crates/cli/src/step.rs`(新規)に、1 つのジョブの共通の手順(`decide`、`record_skip`、`run_step`、`remove_if_requested`)を置き、`run.rs` と `pipeline.rs` はそれを呼ぶだけにする。いつ消すかは呼び出し側に残す。動きは 1 文字も変えない
- Touches: `crates/cli/src/{step,run,pipeline,lib}.rs`、cli の流れのテスト(通っていない枝だけを足す)、`docs/spec/rust/05-execution-flow.md`(§5.2、§5.3 と共通の手順の節)、`docs/spec/rust/02-architecture.md`(モジュールの表、呼び出しの木)
- Tests: 既存のテストを変えずにすべて通す。足すのは、通っていない枝の流れのテスト(2〜3 本)だけ

設計の出典: `construction/functional-design/functional-spec.md`(§1〜§5)、`rules.md`(BR1.1〜BR4.1)。追跡の ID は成功の指標 `SM1`〜`SM4`。テストは Minimal、test-after、行カバレッジ 80% の下限(Build and Test が全体で確かめる)。新しい依存、`unsafe`、`cfg` は足さない。

## Testing Contract

```json
{
  "version": 1,
  "methodology": "test-after",
  "source": "team",
  "ordering": "各テスト可能な層を実装してから、その層のテストを書いて実行する。",
  "scope": "refactor",
  "test_strategy": "minimal",
  "project_type": "brownfield",
  "applicable_notes": [
    {
      "layer": "org",
      "text": "We treat tests as a first-class deliverable in every Bolt. The specific\nmethodology (TDD, BDD, ATDD, or classic test-after) is affirmed at\npractices-discovery and recorded in `team.md` under this heading with explicit\n`Methodology` and `Ordering` fields; Code Generation resolves those fields\nindependently from coverage, tooling, and scope notes.\n\nWhen no posture has been affirmed, our default per scope is:\n- **Methodology**: test-after\n- **Ordering**: implement each applicable testable layer, then write and run\n  that layer's tests.\n- `mvp`, `enterprise`, `feature`, `infra` add an 80% line-coverage\n  floor and CI execution before merge.\n- `bugfix`, `security-patch` add a targeted regression for the specific\n  bug/vulnerability and require the existing suite to remain green.\n- `express` uses the Minimal strategy: requirement-driven unit tests (one per\n  requirement, with a happy-path floor per component); existing tests remain\n  green.\n- `poc`, `refactor`, `workshop`, `classic` add no extra new-test floor and require the\n  existing suite to remain green.\n\nThe active `Test Strategy` still applies in every scope and determines test\nvolume/types. Scope floors are additive; they never reduce or replace the\nselected strategy.\n\nBuild and Test verifies defined coverage floors and affirmed quality targets;\nthey may not be weakened to make a step pass.\n\nAffirm a stricter posture in `team.md` if the team commits to one."
    },
    {
      "layer": "team",
      "text": "- **Methodology**: test-after\n- **Methodology evidence**: 基準(前回のインタビュー Q3)でチームは test-after を選んだ。履歴も整合する。たとえば `[job.retry]` では、型・読み取りの実装(d554ece、445d478)の後に、テストを追加するコミット(8ec98b5)が続き、実行の流れ(19d7b1a)の後に README のテスト(de4e3ad)が続く。`code-quality-assessment.md` も「チームの方針(test-after、行カバレッジ 80%)と一致する」と記す。\n- **Ordering**: 各テスト可能な層を実装してから、その層のテストを書いて実行する。\n- テストの種類: 各クレートの `tests/` に統合テスト(core 10、runtime 13、cli 21、converter 5 のファイル)。流れのテストは偽の `Runtime`/`Sink`/`Env`/`Signals` を注入する。実バイナリのテストは `conductor-cli-test-helper` を使う。非機能のテスト(`nfr_*.rs`)、網羅のテスト(`ALL` 配列)、デモのテスト(`unix_demo.rs`、`windows_demo.rs`)を含む。テスト名は日本語。\n- 行カバレッジ 80% を下限とする。CI の `coverage` ジョブが `cargo llvm-cov --fail-under-lines 80` で強制する(テスト専用の補助バイナリは除外)。\n- ローカルでは、主な開発環境の OS でテストを実行する。それ以外の OS のテストは GitHub Actions の CI(`test`: ubuntu、macos、windows)で実行する。マージ前に CI を通す。\n- 最小の Rust(1.100.0)でもテストを実行する(`msrv-test`)。\n- 起動の性能(`nfr_perf.rs`)は `startup-bench` で測るが、警告のみで失敗にしない(`STARTUP_BENCH_ENFORCE=false`、`continue-on-error`)。\n- 今回の機能にも、test-after と 80% の下限、単体テストと実バイナリを動かす統合テストを続ける(インタビュー Q3)。\n- 主な開発環境の OS は macOS で、テストはローカルの macOS で実行する。他の OS は CI で確認する(インタビュー Q6)。"
    }
  ],
  "obligations": {
    "strategy": "minimal",
    "strategy_volume": [
      "One verifiable test per requirement at the narrowest effective level.",
      "At least one happy-path unit test per component.",
      "Unit tests are the default; a bugfix/security scope floor may require an integration or E2E regression when that is the narrowest level that reproduces the defect."
    ],
    "scope_floor": [
      "Keep the existing test suite green.",
      "This scope adds no extra new-test floor beyond the selected test strategy."
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
  "input_sha256": "sha256:c5ec90c86cfde4bc3f3a63db01255b93f6cdb74000081f6d363ccea531e3e68a",
  "contract_sha256": "sha256:f4f95e97f799def5c5b22a16d75ab141601b6d1a9476cc0f7b322bd0d1aafcd6"
}
```

## 手順

- [x] Step 1: 既存のテストの実行環境を確かめる。変更の前に、`cargo test --locked -p conductor-cli` が通ることと、件数を記録する(比較のため。コマンドは `unit-test-instructions.md`)。
- [x] Step 2: 共通の手順を実装する: `crates/cli/src/step.rs` を新しく作り、`lib.rs` に `mod step;` を足す。`decide`、`record_skip`(`(PipelineState, bool)` を返す)、`run_step`(`StepResult { recorded, outcome, next, after }` を返す。`base` は値で受ける)、`remove_if_requested` を置く。中身は、今の `run.rs` と `pipeline.rs` の同じ部分を、順序、`Durability`、`StateContext` を変えずに移す(functional-spec.md §3)。ドキュメントコメントは日本語で、規則の番号(BR1.1、BR1.2)を書く。(SM1)
- [x] Step 3: `run.rs` を、共通の手順を呼ぶ形にする。Skip は `record_skip` の `changed` が真のときだけ、表示の前に `remove_if_requested`(完了しているとき)。Run は `job_index` の取り出しを `run_step` の前に置き、`run_step` の後、`next.completed` なら `remove_if_requested`、そのあと `exit_code_for`。(BR1.2、BR2.1。SM1、SM2)
- [x] Step 4: `pipeline.rs` を、共通の手順を呼ぶ形にする。ループ、Skip の表示の後の完了(変わらなくても)、エラーか分類なしの停止、完了、ジョブのあいだの中断、「完了を判定できない」は残す。`finish` は `remove_if_requested` を使う形にする。`current.take()` と、`StepResult.recorded` を `current` に戻すことは、今と同じ。(BR1.2、BR2.1。SM1、SM2)
- [x] Step 5: 層のテストを書いて実行する: 既存のテストを変えずに、`cargo test --locked -p conductor-cli` がすべて通り、件数が Step 1 と同じこと。今のテストで通っていない枝を確かめ(`cargo llvm-cov` の行の表示で `step.rs`、`run.rs`、`pipeline.rs` を見る)、通っていないものだけ、流れのテスト(`flow_run.rs` か `flow_pipeline.rs`)に足す。候補: `pipeline` で、スキップの記録が変わらないのに完了しているときに消すこと、`run` で、スキップの記録が変わらないときに消さないこと。テスト名は日本語。(BR2.1。SM2)
- [x] Step 6: 仕様書を直す: `docs/spec/rust/05-execution-flow.md` に、共通の手順の節を足し(4 つの関数と、書く順序)、§5.2 と §5.3 を、それを指す形に書き直す。run と pipeline の違い(functional-spec.md §2)は、それぞれの節に残す。`02-architecture.md` のモジュールの表に `step.rs` を足し、`pipeline.rs` の行と呼び出しの木(`finish` を含む)を直す。ほかの仕様書は変えない。(BR3.1。SM3)
- [x] Step 7: 全体の確認: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --locked --workspace` が通ること(行カバレッジは Build and Test が確かめる)。(BR4.1。SM4)
- [x] Step 8: 文書と追跡: `code-summary.md`、`traceability.json` を作る。

## 守ること

- 利用者から見える動き(出力、終了コード、ステートの書き込みの順序・`Durability`・文脈・内容)を 1 文字も変えない。既存のテストは変えない。
- 新しい外部ライブラリを足さない(`Cargo.toml`、`Cargo.lock` に差分を出さない)。`unsafe` と `cfg(unix)`/`cfg(windows)` を足さない。
- コミットは小さな単位(共通の手順、run、pipeline、テスト、仕様書 の順)。各コミットでテストが通ること。メッセージは日本語。
- ファイルは、ファイルの作成と編集のツールで書く(シェルの sed、perl、python で書き換えない)。

## Assumptions

なし(新しい型は、cli の内部の `StepResult` だけで、設計の `entities.md` に書いた)。
