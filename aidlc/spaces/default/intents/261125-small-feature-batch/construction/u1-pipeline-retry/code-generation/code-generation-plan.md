# コード生成の計画: u1-pipeline-retry(パイプライン共通のリトライの既定)

## Summary

- Builds: `[pipeline.retry]` を全ジョブの既定にし、ジョブの `retry = false` で打ち消す検証と組み立て(core の定義の検証だけ)
- Touches: `crates/core/src/definition/walk/mod.rs`、`walk/retry.rs`、`crates/core/src/types.rs`(文書)、core と cli のテスト、`README.md`、`demo/retry.toml`・`demo/windows/retry.toml`・`demo/README.md`
- Tests: 単体テスト約 20(core の検証)、非機能のテスト 4(秘密・性能)、cli の流れと表示のテスト 4、実バイナリのテスト 2(validate の 64、デモ)

出典: 機能の設計(`functional-spec.md`、`rules.md` の BR1.1〜BR4.2、`entities.md`)、非機能の設計(`security-design.md`、`performance-design.md`、`reliability-design.md`、`observability-design.md`)、ユニットの定義(`unit-of-work.md` の U1)、要件(`requirements.md` の FR1.1〜FR1.6、FR8.1)、契約(`contract-summary.md` の C3)。ユーザーストーリーは作っていないため、各ステップは要件と規則の ID に対応させる。

テストの進め方は Testing Contract(test-after)に従う。層ごとに実装してから、その層のテストを書いて走らせる。触る層は core の定義の検証(ビジネスロジック)だけで、データの層・API・画面は無い。cli と runtime の本番のコードは変えない(BR2.4)。

## 設計のレビューからの持ち越し(このユニットで扱う)

- `[pipeline]` の `name` が無いか空のとき、今の `pipeline_section` は早く返す。既定の読み取りは `env` と同じく `name` の判定より前に行い、`name` が誤りでも既定の誤りと、既定との重なりの検査を続ける(Step 3)。
- `retry = false` は v0.9.9 では誤り(RetryNotTable)で、このユニットから通る。これは FR1.4 による意図した変更で、README の変更点に書く(Step 9)。
- `retry = true` などの誤りは、種類の文「retry は表で書きます」を変えず、理由の文だけ「テーブルか `false`」にする(BR2.1、BR4.1)。種類の文と理由の文の言い回しの差は残る。種類の文を変えると、既定を書かない定義の表示まで変わるためである。

## ステップ

### Step 1: テストの準備を確かめる

- [x] 今のテストが通ることを、変更の前に確かめる。`cargo test -p conductor-core --test definition_retry --test definition_valid --test nfr_security --test definition_perf --locked` と `cargo test -p conductor-cli --test flow_retry --test flow_validate --test binary_basic --test unix_demo --locked` を走らせ、結果を記録する(`unit-test-instructions.md` のコマンド)。

### Step 2: ジョブの `retry` を 3 つの形で読む(core、ビジネスロジック)

- [x] `crates/core/src/definition/walk/retry.rs` に、ジョブの `retry` の読み取りの結果 `JobRetrySetting`(無し・`false`・表)を足す。`retry` が `false` なら disabled、表なら今の読み取り(RawRetry)、`true` やほかの型は種類 RetryNotTable、理由「`retry` はテーブルか `false` で書きます」の誤り(BR2.1、FR1.4)。
- [x] `crates/core/src/definition/walk/mod.rs` の `RawJob` の `retry` を `JobRetrySetting` にし、ジョブの表の始まりの位置 `start` を `RawJob` に足す(重なりの誤りの位置。BR3.2、`entities.md` の JobReading)。
- [x] 表 1 つを読む関数は、ジョブと既定で共有できる形(表を受け取る関数)にする(`security-design.md` の 1)。

### Step 3: `[pipeline.retry]` を読む(core、ビジネスロジック)

- [x] `PIPELINE_KEYS` に `retry` を足す。`pipeline_section` で、`name` の判定より前に `retry` を読む。表なら Step 2 の共有の関数で `PipelineRetryDefault` にし、表でなければ種類 RetryNotTable、理由「`[pipeline.retry]` はテーブルで書きます」の誤り(BR1.1、BR1.2、FR1.1)。
- [x] 既定は `PipelineHead` とは別に `walk` まで返し、`name` の誤りで既定が失われないようにする(設計のレビューの持ち越し)。
- [x] 既定の `max_interval` が `interval` より小さければ、`max_interval` の位置で RetryMaxIntervalTooSmall を 1 回だけ積む(BR3.5)。

### Step 4: 既定との重なりの検査(core、ビジネスロジック)

- [x] `cross_check` に既定を渡し、ジョブごとに、今の `check_retry` の隣で次を行う(BR3.1〜BR3.4、FR1.5):
  - 表のジョブは今のまま自分の表と成功・警告の重なりを調べる。`false` のジョブは調べない。
  - 無しのジョブは、既定があり、既定の `exit_codes` が読めていれば、既定の `exit_codes` と、そのジョブの成功(書かなければ 0 だけ。誤りなら調べない)・警告の終了コードを比べる。重なれば種類 RetryExitCodesOverlap、理由「既定のリトライ(`[pipeline.retry]`)の `exit_codes` に、このジョブの成功または警告の終了コードと同じ値があります」を、ジョブの表の始まりの位置で積む。
- [x] 理由の文は固定の定数にし、値・ジョブの ID を入れない(NFR6.1)。

### Step 5: 実効のリトライを組み立てる(core、ビジネスロジック)

- [x] `build_jobs` に既定(組み立てた `RetryPolicy`、無ければ無し)を渡し、表 → その表、`false` → 無し、無し → 既定の複製、既定も無し → 無し、で `JobDefinition.retry` を決める(BR2.2、BR2.3、FR1.2〜FR1.4)。既定の表で値が欠けていれば、今の `build_retry` と同じく組み立てない。
- [x] `PipelineDefinition` と `JobDefinition`・`RetryPolicy` の形は変えない(BR2.4)。`crates/core/src/types.rs` の `JobDefinition.retry` のドキュメントコメントを「実効のリトライ(ジョブの `[job.retry]`、無ければ `[pipeline.retry]`、`retry = false` なら `None`)」に直す。
- [x] `walk/mod.rs` と `walk/retry.rs` のモジュールのドキュメントコメントに、既定と打ち消しの規則の番号を足す(日本語)。

### Step 6: core の層のテストを書いて走らせる

- [x] `crates/core/tests/definition_retry.rs` に足す(テスト名は日本語):
  - 正しい `[pipeline.retry]` を読み、`retry` を書かないジョブに既定が入る(BR1.2、BR2.2)
  - `[pipeline.retry]` の各誤り(必須のキー無し、`max = 0`、空の `exit_codes`、片方だけの backoff、`max_interval` が小さい)が `[job.retry]` と同じ種類の誤りになり、上限と間隔の誤りは 1 回(BR1.2、BR3.5)
  - `[pipeline] retry = false` と `retry = 3` は RetryNotTable(BR1.1)
  - ジョブの `retry = true` は RetryNotTable、理由の文に「`false`」(BR2.1)
  - 既定とジョブの表があれば、ジョブの表を丸ごと使い、既定の倍率を混ぜない(BR2.2、FR1.3)
  - `retry = false` のジョブは既定があっても `None`、既定が無くても誤りにならず `None`(BR2.2、BR2.3)
  - 既定と警告・成功(省略時の 0 を含む)の重なりは、ジョブごとに 1 つ、そのジョブの行で RetryExitCodesOverlap。重なるジョブが 2 つなら 2 つ(BR3.1、BR3.2)
  - `retry = false` のジョブと自分の表を持つジョブは、既定と重なっても誤りにならない(BR3.3)
  - 既定の `exit_codes` が誤り、またはジョブの `success_exit_codes` が誤りのときは、重なりの誤りを重ねて出さない(BR3.4)
  - `name` が空でも、既定の誤りと既定との重なりの誤りが出る(設計のレビューの持ち越し)
- [x] 今の `br1_2_retry_が表でない` など、理由の文や `retry = false` の扱いに触れるテストがあれば、この変更に合わせて直す。
- [x] `crates/core/tests/definition_valid.rs` に、既定を書かない今の定義が今と同じ定義に組み立てられるテストを足す(BR4.1、NFR1.1)。
- [x] `crates/core/tests/nfr_security.rs` の `cases` に、既定の型の誤り、ジョブの `retry` の型の誤り、既定との重なりの 3 つを足し、値・ジョブの ID の位置に目印を入れる(NFR6.1)。
- [x] `crates/core/tests/definition_perf.rs` に、既定を持ち 1 万のジョブに既定を当てはめる定義の読み込みの、時間の上限つきのテストを足す。上限は、今のテストと同じく実測に余裕を持たせて決める(NFR1.3、NFR1.4)。
- [x] Step 1 の core のコマンドを走らせ、通ることを確かめる。

### Step 7: cli の流れと表示のテストを書いて走らせる(cli の本番は変えない)

- [x] `crates/cli/tests/flow_retry.rs` に、偽の `Runtime` で、既定によるリトライ(`retry` を書かないジョブが既定の終了コードで終わるとリトライして成功)と、`retry = false` のジョブがリトライしないことを足す(BR2.2、FR1.2、FR1.4)。
- [x] `crates/cli/tests/flow_validate.rs` に、既定との重なりが 64 で、`run` と同じ誤りの文になることを足す(FR1.5)。
- [x] `crates/cli/src/tests/graph_report.rs` と `crates/cli/src/tests/plan_report.rs` に、既定を当てはめたジョブと同じ値の `[job.retry]` を書いたジョブで、`graph -v` と `--dry-run` の表示が同じになることを足す(BR2.4、NFR1.5)。
- [x] `crates/cli/tests/binary_basic.rs` に、実バイナリの `validate` が既定との重なりで 64 になり、標準エラーに値が出ないことを足す(NFR2.2)。
- [x] Step 1 の cli のコマンドを走らせ、通ることを確かめる。

### Step 8: デモを更新する

- [x] `demo/retry.toml` と `demo/windows/retry.toml` の `flaky` の `[job.retry]` を `[pipeline.retry]` に移し、`flaky` は既定でリトライさせる。打ち消しの書き方 `retry = false` をコメントで示す(BR4.2)。デモの出力(リトライの知らせの行)は変わらない。
- [x] `demo/README.md` のリトライの場面の説明を、既定と打ち消しに合わせる。
- [x] `cargo test -p conductor-cli --test unix_demo --locked` を走らせ、実バイナリで既定によるリトライが動くことを確かめる(NFR2.2、NFR3.2)。`windows_demo.rs` は CI の windows で確かめる。

### Step 9: README を更新する

- [x] `README.md` のリトライの節に、`[pipeline.retry]`(全ジョブの既定、項目と検査は `[job.retry]` と同じ)、実効のリトライの決まり方(表 → 丸ごと、`retry = false` → 無し、無し → 既定)、既定との重なりの誤り、`retry = false` が v0.10.0 から書けることを足す。定義のキーの表に `[pipeline.retry]` と `retry = false` を足し、終了コード 64 の説明に `[pipeline.retry]` の誤りを足す(BR4.2、FR8.1)。`init` のひな形は変えない(Q4)。
- [x] 文書の検査(`crates/cli/src/tests/docs.rs`)と `cargo test -p conductor-cli --lib --locked tests::docs` が通ることを確かめる。

### Step 10: 書式と静的検査

- [x] `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通ることを確かめる(NFR7.1)。

## 規則と要件の対応

| ステップ | 規則 | 要件 |
|---|---|---|
| Step 2 | BR2.1 | FR1.4 |
| Step 3 | BR1.1、BR1.2、BR3.5 | FR1.1 |
| Step 4 | BR3.1〜BR3.4 | FR1.5 |
| Step 5 | BR2.2〜BR2.4 | FR1.2、FR1.3、FR1.4 |
| Step 6、Step 7 | BR1.1〜BR4.1 | FR1.1〜FR1.6、NFR1、NFR2、NFR6 |
| Step 8、Step 9 | BR4.2 | FR8.1 |

## コミットの分け方

チームの慣行に従い、小さな単位でコミットする(件名は日本語、先頭に `core:`・`cli:`・`docs:`)。例: Step 2、Step 3、Step 4、Step 5 は core の各 1 コミット、Step 6 のテストは 1〜2 コミット、Step 7 は cli のテスト、Step 8 と Step 9 は `docs:`。版は上げない(U7 で上げる)。

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
