# Code Generation Plan — u1-retry-config(リトライの設定と検証)

## Summary

- Builds: ジョブの定義に表 `[job.retry]` を書けるようにし、読み取って検証する(終了コード、最大回数、間隔、倍率、上限)。誤りは定義の誤り(64)になり、`validate` が検出する
- Touches: `crates/core/src/types.rs`、`crates/core/src/error.rs`、`crates/core/src/definition/walk.rs`(と、必要なら同じ階層の新しいファイル)、`crates/cli/src/report/messages.rs`、`JobDefinition` のリテラルを持つ既存のテスト(core、runtime、cli)、`crates/core/tests/definition_retry.rs`(新規)、`crates/converter/tests/convert_roundtrip.rs` と `crates/cli/tests/flow_validate.rs` と `crates/cli/tests/report_messages.rs` への追加
- Tests: 新規の統合テスト 1 ファイル(core、約 30 件)に加えて、converter・cli の既存ファイルへの追加が約 6 件。既存の約 705 件は変更なしで通す

## 根拠

- 設計: `construction/u1-retry-config/functional-design/` の `functional-spec.md`、`rules.md`(BR1.1〜BR1.16)、`entities.md`
- 要件: FR1.1〜FR1.9、FR7.1、FR7.2(`requirements.md`)
- 既存のコードの触れる場所: `code-structure.md` の「新しいジョブ単位の設定を足すときに触れる場所」の 1〜4、9、10(5〜8 と 11 は、この Unit では触れない)
- チームの方針: test-after(各層を実装してから、その層のテストを書いて実行する)、`cargo fmt` と `cargo clippy -D warnings`、行カバレッジ 80%、変更は小さく、こまめにコミットする

## 名前を付けて追加するデータの要素

要件や設計が求めるもの(`entities.md`)だけを足す。

- 型 `RetryPolicy`(`crates/core/src/types.rs`)。項目: `exit_codes`(終了コードの集合、BR1.5〜1.8)、`max`(最大回数、BR1.9)、`interval_secs`(間隔の秒、BR1.10)、`multiplier`(倍率、省略可、BR1.12)、`max_interval_secs`(間隔の上限の秒、省略可、BR1.10、BR1.13)。値はすべて整数(要件 C1)
- `JobDefinition` の項目 `retry: Option<RetryPolicy>`(BR1.1)
- 誤りの種類(`DefinitionErrorKind`)を 10 個追加(BR1.2〜BR1.13): `RetryNotTable`(BR1.2)、`RetryMissingKey`(BR1.3)、`RetryExitCodesEmpty`(BR1.5)、`RetryExitCodeInvalid`(BR1.5)、`RetryExitCodesOverlap`(BR1.7)、`RetryMaxInvalid`(BR1.9)、`RetryDurationInvalid`(BR1.10)、`RetryBackoffIncomplete`(BR1.11)、`RetryMultiplierInvalid`(BR1.12)、`RetryMaxIntervalTooSmall`(BR1.13)。許可されていないキー(BR1.4)は、既存の未知のキーの種類を使う(新しい種類は足さない)

## 実装ステップ(test-after: 各層を実装してから、その層のテストを書いて実行する)

- [x] Step 1: テスト実行環境を確認し、変更前の基準を取る。`cargo test --workspace --locked` が全件通ることを記録する。コマンドは `unit-test-instructions.md` に記載
- [x] Step 2: データの層の実装。`RetryPolicy` と `JobDefinition::retry` を追加し、`JobDefinition { .. }` のリテラルを持つ既存の箇所(core の `types.rs` のテスト、runtime の `tests/common/mod.rs` の `job()`、cli のテストの定義の組み立て)に、`retry: None` を足す。ビルドが通ること。(BR1.1、FR1.1)
- [x] Step 3: データの層のテストを書いて実行する。`RetryPolicy` を持つ定義と持たない定義の値が、期待どおりに作れ、比較できること(`Eq`)を、core の単体テストで確かめる。`cargo test -p conductor-core --locked`
- [x] Step 4: 誤りの種類の実装。`DefinitionErrorKind` に 10 個の種類を足し(`ALL`、`code`、表示)、`cli/src/report/messages.rs` の `definition_kind` に、英語と日本語の固定の文を足す。入力の値そのものは入れず、キー名だけを使う(既存の方針)。(BR1.14、FR7.1)
- [x] Step 5: 誤りの種類のテストを書いて実行する。全種類の表示にコードと固定の文があり、重複しないこと(既存のテストが、種類の数を自動で拾う)と、cli の `report_messages` で、10 個の英語と日本語の文があることを確かめる。`cargo test -p conductor-core --locked` と `cargo test -p conductor-cli --test report_messages --locked`
- [x] Step 6: 検証(業務ロジック)の実装。`walk.rs` の `JOB_KEYS` に `retry` を足し、`RawJob` に読み取り結果を持たせ、`Walker::job` で `[job.retry]` を検査する(表であること BR1.2、許可するキー BR1.4、必須のキー BR1.3、終了コード BR1.5〜BR1.6、最大回数 BR1.9、間隔 BR1.10、倍率 BR1.12、倍率と上限の組 BR1.11)。`cross_check` で、終了コードと成功・警告の集合の重なり(BR1.7)と、上限が間隔以上(BR1.13)を検査する。`build_jobs` で `RetryPolicy` を組み立てる。間隔の文字列(`<整数><s|m|h>`)を秒に直す処理は、独立した小さな関数にして、単体で試せるようにする
- [x] Step 7: 検証のテストを書いて実行する。`crates/core/tests/definition_retry.rs` を新規に作る。正しい定義(固定の間隔、倍率と上限つき、最小値と最大値の境界、`exit_codes` の重複、127 を含む一覧)と、10 個の誤りの種類それぞれ、境界の内側と外側(N=1 と 0 と 101、間隔 `"1s"` と `"0s"` と `"25h"`、倍率 2 と 1 と 11、終了コード 1 と 0 と 256)、`retry` を持たない定義が、これまでと同じ結果になること(BR1.1、BR1.16)、誤りが位置の順に集められること(BR1.14)を確かめる。`cargo test -p conductor-core --locked`
- [x] Step 8: `validate` と `convert` の確認。converter の `convert_roundtrip.rs` に、`[job.retry]` を含む定義の往復変換(TOML→YAML→TOML)のテストを足す(BR1.15、変換のコードは変えない見込み。変える必要が出たら、計画に追記して承認を取る)。cli の `flow_validate.rs` に、正しい `[job.retry]` を含む定義で `validate` が通ることと、誤りのある定義で終了コード 64 と、誤りの文が出ることを確かめるテストを足す。`cargo test -p conductor-converter --locked` と `cargo test -p conductor-cli --test flow_validate --locked`
- [x] Step 9: 全体の確認。次をすべて実行して通ること。`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked`、検査コマンド(`cargo llvm-cov ... --fail-under-lines 80`)。`cargo tree` で `-sys` と C のビルド依存が増えていないこと。テストコードの既存のアサーションが変わっていないこと
- [ ] Step 10: 文書と記録。コードの文書コメント(日本語、要件 ID つき)を書き、`code-summary.md`、`source-manifest.json`、`traceability.json` を作る

## 触れないもの(この Unit の外)

- 実行の流れ(`run.rs`、`pipeline.rs`、`runner.rs`、`flow/*`)、ステートファイル(`runtime/src/state/*`)、表示の種類(`DisplayEvent`)は、U2、U3 の担当
- `README.md` と `demo/` は、U3 の担当
- 新しい外部クレートは、追加しない

## 要件との対応

| 要件 | ステップ |
|---|---|
| FR1.1、FR1.2 | 2、3、6、7 |
| FR1.3 | 6、7(BR1.7) |
| FR1.4 | 6、7(BR1.10) |
| FR1.5、FR1.9 | 6、7(BR1.11〜BR1.13) |
| FR1.6 | 2、7、9(既存のテストが通る) |
| FR1.7 | 6、7(BR1.3) |
| FR1.8 | 6、7(境界値) |
| FR7.1 | 4、5、7、8 |
| FR7.2 | 8 |

## Assumptions

- 間隔の文字列を秒に直す処理を、新しいファイルに分けるか `walk.rs` の中に置くかは、コードの大きさを見て決める(どちらでも、公開する形は変わらない)
- converter は、スキーマに依存しない作りのため、コードの変更は不要の見込み。必要になったら、計画の変更として扱う

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
