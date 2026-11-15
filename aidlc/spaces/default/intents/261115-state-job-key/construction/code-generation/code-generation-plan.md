# Code Generation Plan(state-job-key)

## Summary

- Builds: ステートファイル(`.state`)のジョブのテーブルのキーを `jobs` から `job` に改名する(書き出し、読み込み、エラー文面、テスト、版)。
- Touches: `crates/runtime/src/state/{reader,writer}.rs`、`crates/runtime/src/error.rs`、`crates/runtime/tests/`(ステートの TOML を持つテスト)、README とデモ(ステートのキーの記述があれば)、4 つの `Cargo.toml` の版(0.3.0)と `Cargo.lock`。
- Tests: 新しいテスト 約 4 件(要件ごと)と、ステートの TOML を持つ既存のテストの修正。既存のテストは、それ以外そのまま成功。行カバレッジ 80% 以上を保つ。

## 前提(設計から)

- 要件: `inception/requirements-analysis/requirements.md`(FR1〜FR5、NFR1〜NFR3)。調査: `aidlc/spaces/default/codekb/rust-job-conductor/`。zero-Unit(express)のため、この計画が設計を兼ねる。
- **変更の範囲:** ステートのファイル上のキー名だけ(TOML のキー `jobs` → `job`)。`consistency.rs` と `record.rs` の `.jobs` は定義側のフィールドなので、変えない。Rust の型・フィールド名(`JobState`、`PipelineState.job_states`)、定義ファイルの形式、`cli` の表示、`convert` は変えない(NFR1)。
- **互換なし(FR2):** 読み込みは `job` だけ。`jobs` を持つステートは、`TOP_KEYS` にないキーとして、既存の「壊れたステート」(`corrupt`、65)で拒否される。専用のコードは足さない。`format_version` は 1 のまま(FR3)。
- **名前の衝突に注意:** ステートのトップレベルの `job` キーは、配列のテーブル(`[[job]]`)。`TOP_KEYS` の `"jobs"` → `"job"`、`walk` の `required(table, "jobs")` → `"job"`、`writer.rs` の `serialize` の `"jobs".into()` → `"job".into()`。
- 1.85 でビルドできる書き方(let チェーンを使わない)。`unsafe` を増やさない。

## 実装の手順(test-after: 実装してから、その層のテストを書いて実行する)

- [x] **Step 1. 実行環境の確認:** `cargo test --workspace --locked` が成功する(580 件)ことを確かめ、`unit-test-instructions.md` のコマンドを確定する。
- [x] **Step 2. runtime: ステートの読み書きを実装する(FR1、FR2、FR3):** `crates/runtime/src/state/writer.rs` の `serialize` が書くキーを `job` にし、`crates/runtime/src/state/reader.rs` の `TOP_KEYS` と `walk` の読むキーを `job` にする。`format_version` は変えない。
- [x] **Step 3. runtime: エラー文面を直す(FR4):** `crates/runtime/src/error.rs` の `ORDER_MISMATCH` など、ステートのキーとして `jobs` に触れている固定の文を、`job` に直す(入力の値を含まない原則は変えない)。ほかに、ステートのキーとしての `jobs` の記述(doc コメントなど)があれば、直す。
- [x] **Step 4. runtime: テストを直し、足す(FR1〜FR4):** ステートの TOML を持つ既存のテスト(`state/reader.rs` の単体テスト、`tests/state_read.rs`、`nfr_robust.rs`、`toml_depth.rs`、`state_write.rs`)の `[[jobs]]` とキー列を、`[[job]]` / `job` に直す。新しいテスト: (a) 書き出したステートに `[[job]]` があり `jobs` がない、書いて読み戻すと等しい(FR1)、(b) `[[job]]` のステートが読める、`format_version = 1`(FR2、FR3)、(c) `[[jobs]]` のステートは `corrupt` で拒否される(FR2)、(d) エラー文面に、ステートのキーとしての「jobs」が残らない(FR4)。
- [x] **Step 5. 文書とデモ(FR5):** README とデモに、ステートのキーとしての `[[jobs]]` の記述があれば `[[job]]` に直す(定義ファイルの旧書式の注記 `[[jobs]]` → `[[job]]` は、そのまま。ステートの記述ではない)。なければ変更しない。
- [x] **Step 6. 版(NFR3):** 4 つの `crates/*/Cargo.toml` の `version` を 0.3.0 にし、`Cargo.lock` に追従させる。
- [x] **Step 7. 全体の確認:** `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked`、`cargo +1.85 build --workspace --locked --all-targets`、`cargo llvm-cov` で行カバレッジ 80% 以上(補助の実行ファイルを除く)。実際のバイナリで、`run` を失敗させて `.state` を作り、`[[job]]` で書かれていること、`watch` が読めること、`DEMO_SLEEP=0.1 sh demo/run-demo.sh` が完走することも確かめる。

## テストの方針(Minimal)

要件ごとに、最も狭い層で 1 件を目安に、約 4 件を足す。既存のテストは、ステートの TOML に触れるものの修正を除き、そのまま成功する。行カバレッジ 80% の下限は、Build and Test がワークスペース全体で確かめる。

## 設計からの追跡(要件 → 手順)

| 要件 | 手順 |
|------|------|
| FR1 | Step 2、4 |
| FR2 | Step 2、4 |
| FR3 | Step 2、4 |
| FR4 | Step 3、4 |
| FR5 | Step 5 |
| NFR1 | Step 2、7(変更の範囲の確認) |
| NFR2 | Step 1、7 |
| NFR3 | Step 6 |

## Assumptions

None.

## Testing Contract

```json
{
  "version": 1,
  "methodology": "test-after",
  "source": "team",
  "ordering": "各テスト可能な層を実装してから、その層のテストを書いて実行する。",
  "scope": "express",
  "test_strategy": "minimal",
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
  "input_sha256": "sha256:47c2e6e6f234580fd227d246415c56f693b43f2a7953fee56ef7217b54cbd097",
  "contract_sha256": "sha256:3e75c1c022f49e57c8d1612ebaf8b3e7016754454c19abb2dc7f313d6288a4e3"
}
```
