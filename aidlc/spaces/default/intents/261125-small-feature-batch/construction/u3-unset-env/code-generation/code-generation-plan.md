# コード生成の計画: u3-unset-env(`unset_env`)

## Summary

- Builds: `[pipeline]` と `[job]` の `unset_env` で、親から引き継いだ環境変数を消してジョブに渡す。`env`・`.env` との重なりは 64、`--dry-run` は消す名前も出す
- Touches: core の `types.rs`・`error.rs`・`definition/walk/env.rs`・`walk/mod.rs`・`envfile/mod.rs`・`merge.rs`・`parse.rs`・`plan/`、runtime の `runner.rs`、cli の `deps.rs`・`ctx.rs`・`step.rs`・`report/plan_text.rs`・`messages.rs`、テスト、`README.md`、`demo/env.toml`・`demo/windows/env.toml`・`demo/README.md`
- Tests: 単体・統合テスト約 25(定義、重なり、統合、行、消す動き)、非機能のテスト 2、流れのテスト 5、実バイナリのテスト 4、デモ

出典: 機能の設計(`functional-spec.md`、`rules.md` の BR1.1〜BR4.3、`entities.md`)、非機能の設計(`security-design.md`、`reliability-design.md`、`performance-design.md`、`scalability-design.md`、`observability-design.md`、`logical-components.md`)、ユニットの定義(`unit-of-work.md` の U3)、要件(`requirements.md` の FR3.1〜FR3.5、FR8.1)、契約(`contract-summary.md` の C1、C3)、設計の決定(ADR-007)。ユーザーストーリーは作っていないため、各ステップは要件と規則の ID に対応させる。

テストの進め方は Testing Contract(test-after)に従う。core → runtime → cli の順に、層を作ってからその層のテストを書いて走らせる。

## データの形の変更

- `PipelineDefinition.unset_env`、`JobDefinition.unset_env`: 名前の並び(`entities.md`)。
- `EffectiveEnv.removals`: 消す名前の並び(`entities.md`、C1 の output_u3)。
- `EnvEntry.line`(行番号)、`EnvFileContent.file`(表示用のファイル名)(`reliability-design.md` の 1)。
- `DefinitionErrorKind::UnsetEnvConflict`、`EnvFileErrorKind::UnsetConflict`(`entities.md`)。
- 計画(`--dry-run`)の各ジョブに、消す名前の並びを足す(`observability-design.md`)。

## 設計のレビューからの持ち越し(このユニットで扱う)

- 重なりの検査の比べ方: 非機能の設計(`security-design.md` の 1)が機能の設計の BR2.1・BR2.2 の「完全一致」を上書きし、**どの OS でも ASCII の大文字小文字を区別せずに**比べる。UNIX 系で大文字小文字だけが違う名前を消して足す書き方も 64 になることを README に書く(非機能の設計のレビュー)。
- `.env` との重なりは `EnvEntry.line` と `EnvFileContent.file` を使い、core の `check_unset_conflicts` で「ファイル名と行」で重複を除いて返す(機能の設計のレビュー、非機能の要件のレビュー)。
- `unset_env` の名前の規則の誤りの理由の文は `unset_env` の文に分ける(機能の設計のレビュー)。
- `--dry-run` の消す名前は、名前だけの統合とは別の関数(`merge_unset_names`)で作る(非機能の設計のレビュー)。
- `validate` と `graph` も `.env` の重なりの検査を通ることを流れのテストで確かめる(非機能の設計のレビュー)。
- ステートには足す変数も消す名前も書かない(今のまま。非機能の要件のレビュー)。

## ステップ

### Step 1: テストの準備を確かめる

- [x] 変更の前に、`unit-test-instructions.md` の 3 つのコマンドを走らせ、結果を記録する。

### Step 2: 定義(core)

- [x] `types.rs` に `unset_env` を足す(パイプラインとジョブ)。`walk/env.rs` に `unset_env` の読み取り(配列と文字列の型、空・名前の規則・予約名。理由の文は `unset_env` の固定の文)を足し、`walk/mod.rs` の `PIPELINE_KEYS`・`JOB_KEYS` と組み立てに入れる。同じ名前は最初の 1 つにまとめる(BR1.1〜BR1.3)。
- [x] `error.rs` に `UnsetEnvConflict` を足し(種類の文と理由の文は固定)、`cross_check` で `env` との重なりを ASCII の大文字小文字を区別せずに調べる(パイプラインの `unset_env` はパイプラインとすべてのジョブの `env`、ジョブの `unset_env` はパイプラインとそのジョブの `env`。BR2.1、上の持ち越し)。

### Step 3: 統合と `.env` との重なり(core)

- [x] `envfile/parse.rs` で `EnvEntry.line` を埋め、`EnvFileContent.file` に表示用の名前を入れる。
- [x] `envfile/mod.rs` に `EnvFileErrorKind::UnsetConflict`(固定の文)と `check_unset_conflicts(def, job_scope, files) -> Vec<EnvFileError>` を足す(範囲は今の読み取りの範囲、大文字小文字を区別しない、ファイル名と行で重複を除く。BR2.2)。
- [x] `envfile/merge.rs` の `merge_env` が `removals`(パイプラインの次にジョブ、完全一致で重複を除く)を返し、`merge_unset_names` を足す(BR3.1、BR4.1)。`plan/` の計画の各ジョブに消す名前を入れる。

### Step 4: core の層のテストを書いて走らせる

- [x] 型の変更に合わせて、組み立てているテストの補助の組み立て方だけを直す。
- [x] `env_definition.rs`(型・名前の規則・予約名・重複のまとめ・`env` との重なり(`FOO`/`foo` を含む))、`env_file_parse.rs`(行)、`env_merge.rs`(`removals`、`entries` が今と同じ、`check_unset_conflicts` の範囲と重複除き)、`nfr_security.rs`(新しい誤りの表示に値が出ない)、`env_merge_perf.rs`(消す名前の多い入力を今の上限で)に足す。
- [x] core のコマンドを走らせ、通ることを確かめる。

### Step 5: 消す(runtime)とその層のテスト

- [x] `runner.rs` の `run_job` に消す名前を受け取らせ、足す前に `Command::env_remove` を呼ぶ(空なら呼ばない。BR3.2)。cli の差し替え口の型の変更は同じコミットにする。
- [x] `runner_basic.rs` に、親に置いた変数が消えること、無い名前は何もしないこと、消してから足した名前は足した値になること(UNIX 系では大文字小文字だけが違う名前が残ること)を足す。runtime のコマンドを走らせる。

### Step 6: 差し替え口と流れ・表示(cli)

- [x] `deps.rs` の `Runtime::run_job` と本番の実装、偽の `Runtime` に消す名前を足す。`step.rs` で統合の `removals` を渡す。
- [x] `ctx.rs` の `.env` の読み取りの後に `check_unset_conflicts` を呼び、誤りは今の `.env` の誤りと同じく表示して 64(BR2.2)。
- [x] `report/plan_text.rs` と `messages.rs` に、消す名前の行(英日。消す名前があるジョブだけ)を足す(BR4.1)。

### Step 7: cli の層のテストを書いて走らせる

- [x] `flow_env.rs`(消す名前が渡る、`run` と `pipeline` の範囲で `.env` の重なりの誤り、同じ `.env` を 2 つのジョブが読んでも誤りは 1 つ)、`flow_validate.rs` と `flow_graph.rs`(`.env` の重なりで 64)、`flow_dry_run.rs` と `src/tests/plan_report.rs`(消す名前の行の英日、無いジョブは今のまま)に足す。
- [x] `binary_env.rs` に、実バイナリで親の変数が消えること、`env` と `.env` の重なりで 64(値が出ない)、`--dry-run` の表示を足す。
- [x] cli のコマンドを走らせる。

### Step 8: デモと README

- [x] `demo/env.toml` と Windows 版に `unset_env` の例を足し、`demo/README.md` を合わせ、`unix_demo.rs`・`windows_demo.rs` の確認を足す。`unix_demo` を走らせる。
- [x] `README.md` に、書き方、名前の規則、重なりの誤り(大文字小文字を区別しない比べ方)、Windows の消し方、`--dry-run` の表示を書く。定義のキーの表と 64 の説明を合わせる。`cargo test -p conductor-cli --lib --locked -- tests::docs` を走らせる。

### Step 9: 書式と静的検査

- [x] `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通ることを確かめる(NFR7.1)。

## 規則と要件の対応

| ステップ | 規則 | 要件 |
|---|---|---|
| Step 2 | BR1.1〜BR1.3、BR2.1 | FR3.1、FR3.3、FR3.4 |
| Step 3 | BR2.2、BR3.1、BR4.1 | FR3.2、FR3.3 |
| Step 5 | BR3.2、BR3.3 | FR3.2、FR3.4 |
| Step 6 | BR2.2、BR4.1 | FR3.2、FR3.3 |
| Step 4、5、7 | すべて | FR3.1〜FR3.5、NFR1〜NFR6 |
| Step 8 | BR4.2、BR4.3 | FR3.5、FR8.1 |
| Step 9 | | NFR7 |

## コミットの分け方

小さな単位でコミットする(件名は日本語、先頭に `core:`・`runtime:`・`cli:`・`docs:`)。各コミットで fmt と clippy が通るようにする。例: Step 2、Step 3 は core の 2〜3 コミット、Step 4 のテスト、Step 5 は runtime と cli の型の変更、Step 6・7 は cli、Step 8 は `docs:`。版は上げない。

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
