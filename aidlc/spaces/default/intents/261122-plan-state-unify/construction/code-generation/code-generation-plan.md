# コード生成の計画(Code Generation Plan): `--dry-run` の模擬の重複をなくすリファクタリング

## Summary

- Builds: ステートの更新の関数(`record.rs`)を runtime から core の `flow/record.rs` へ移し、runtime は再エクスポートする。`apply_skip` を O(n) にする。模擬(`plan_pipeline`)は `PipelineState` を持ち、本番と同じ `apply_skip`・`apply_outcome` で記録する(写しの `apply_result` をなくす)
- Touches: `crates/core/src/{lib.rs, flow/mod.rs, flow/record.rs, plan/simulate.rs}`、`crates/runtime/src/{lib.rs, state/mod.rs}`(`state/record.rs` は移動)、`crates/core/tests/plan_perf.rs`、`crates/core/tests/plan_pipeline.rs`(1 本足す)、`docs/spec/rust` の 05、02、12
- Tests: 既存のテストを変えずにすべて通す。足すのは、性能(終点の後の 1,000 個近いスキップ)と、承認した手書きのステートの形の 2 本。実装の後に、書き換える前の模擬と、90 万例で比べる(一時のテスト。削除する)

設計の出典: `construction/functional-design/functional-spec.md`(§1〜§5)、`rules.md`(BR1.1〜BR4.1)。追跡の ID は成功の指標 `SM1`〜`SM5`。テストは Minimal、test-after、行カバレッジ 80% の下限(Build and Test が全体で確かめる)。新しい依存、`unsafe`、`cfg` は足さない。

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

- [ ] Step 1: 既存のテストの実行環境を確かめる。変更の前に、`cargo test --locked --workspace` の件数を記録する(比較のため)。比べる確認のために、書き換える前の `plan_pipeline` の中身(`simulate.rs` の `plan_pipeline` と `apply_result`)を、一時のテスト `crates/runtime/tests/tmp_plan_compare.rs` に「今の模擬」として写す(設計の `plan-compare.rs.txt` を土台にし、`old_plan` を写した中身に、`new_plan` を本物の `conductor_core::plan_pipeline` にする)。このファイルはコミットしない。
- [ ] Step 2: 更新の関数を core へ移す: `git mv crates/runtime/src/state/record.rs crates/core/src/flow/record.rs`。`use conductor_core::...` を `crate::...` に直し、`flow/mod.rs` に `mod record;` と公開、`crates/core/src/lib.rs` の公開の一覧に `initial_state`、`apply_outcome`、`apply_skip`、`apply_interrupt` を足す。runtime の `lib.rs` は `pub use conductor_core::{apply_interrupt, apply_outcome, apply_skip, initial_state};` にし、`state/mod.rs` から `record` を外す。説明(runtime の `lib.rs` の表、`state/mod.rs`)を直す。単体のテストの中身は変えない。(BR1.1、BR1.2。SM1)
- [ ] Step 3: `apply_skip` を O(n) にする: 状態を定義の位置で引く(位置が一致すれば O(1)、一致しなければ探す。`decide.rs` の `status_at` と同じ考え)。結果は変えない。(BR2.3。SM3)
- [ ] Step 4: 模擬を本番の更新に寄せる: `plan_pipeline` は `PipelineState` を 1 つ持つ(ステートが無ければ `initial_state`。パスは模擬の中だけの値)。スキップは `apply_skip`、結果は `apply_outcome` で記録し、判断には、その状態の一覧、再開位置、`end_reached` を渡す。終わり方の「終点に着く」は、その状態の `end_reached`。`apply_result` と、別の変数 `resume`、`end_reached` をなくす。`simulate_job` は、記録に使う `JobExecution` と `JobOutcome` も返す。`simulate.rs` の冒頭の説明を直す。`plan_run` は変えない。(BR1.3。SM2)
- [ ] Step 5: 層のテストを書いて実行する: 既存のテストを変えずに、`cargo test --locked --workspace` がすべて通り、件数が Step 1 と同じこと(足したテストを除く)。Step 1 の一時のテストで、3 つの種(20261122、7、123456789)、各 30 万例を比べ、違いが 0 件であること。確かめたら、一時のテストを削除する。新しいテストを 2 本足す: `crates/core/tests/plan_perf.rs` に、終点の後に 1,000 個近いジョブをスキップする `--dry-run` の計画が 1 秒以内。`crates/core/tests/plan_pipeline.rs` に、`skipped(byEnd)` で `end_reached = false` の手書きのステートの計画が、終わり方「終点に着く」になること(承認した変化)。テスト名は日本語。(BR2.1〜BR2.3。SM3)
- [ ] Step 6: 仕様書を直す: `docs/spec/rust/05-execution-flow.md`(冒頭の所在、§3.3〜§3.5 の所在、§16 の擬似アルゴリズムを `apply_skip`・`apply_outcome` を呼ぶ形にし、`apply_result` の説明を §3.3〜§3.5 を指す 1 文に、`simulate_job` の形)、`02-architecture.md`(core のモジュールの表に `flow/record.rs`、runtime の `state/record.rs` の行の削除、runtime の再公開の一覧、データの流れの図の所在)、`12-rebuild-guide.md`(`state/record.rs` の行、`plan/simulate.rs` の行)。`docs/spec/language-agnostic` は変えない。(BR3.1。SM4)
- [ ] Step 7: 全体の確認: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --locked --workspace` が通ること(行カバレッジは Build and Test が確かめる)。(BR4.1。SM5)
- [ ] Step 8: 文書と追跡: `code-summary.md`、`traceability.json` を作る。

## 守ること

- 本番の出力・終了コード・ステート、`--dry-run` の出力を変えない(ただし、承認した手書きのステートの形 1 つを除く)。既存のテストは変えない。
- 新しい外部ライブラリを足さない(`Cargo.toml`、`Cargo.lock` に差分を出さない)。`unsafe` と `cfg(unix)`/`cfg(windows)` を足さない。core に入出力を持ち込まない。
- コミットは小さな単位(移動、`apply_skip` の速さ、模擬、テスト、仕様書 の順)。各コミットでビルドとテストが通ること。メッセージは日本語。一時のテストはコミットしない。
- ファイルは、ファイルの作成と編集のツールで書く(シェルの sed、perl、python で書き換えない。`git mv` は計画が名指しする)。

## Assumptions

なし(新しい型は足さない。模擬が既存の `PipelineState` を持つことは、設計の `entities.md` に書いた)。
