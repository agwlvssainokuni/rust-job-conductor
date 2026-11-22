# コード生成の計画(Code Generation Plan): 動きを変えない整理のリファクタリング

## Summary

- Builds: 4 つの段の整理。段 1: cli の run ID とステートの照合の集約、コマンドの分割の使い回し、検査の関数の移動、入口の共通化。段 2: 分類と遷移を型で表示に渡す、状態の文字列の対応を core に、照合を core へ、上限つきの読み取りを 1 か所に。段 3: `walk.rs` の分割、小さな重複、`JobExecution::exited`。段 4: graph のテストの組み直し、converter のテスト名の日本語化、偽物の共通化、README の 68、5 つの仕様書の凍結の明記
- Touches: `crates/cli/src/{ctx,graph,aux_commands,dry_run,run,pipeline,step,report/mod,report/watch}.rs`、`crates/core/src/{types,lib,consistency(新規),error,envfile/mod,flow/next_step,plan/simulate,definition/depth,definition/walk(→ walk/)}.rs`、`crates/runtime/src/{lib,types,state/consistency,state/reader,state/writer}.rs`、`crates/converter/src/error.rs`、テスト(段 2 の `report_messages.rs` の組み立ての値、段 4 の graph・converter・cli の偽物)、`README.md`、`docs/spec/rust`、`docs/spec/language-agnostic/11`、5 つの仕様書の `00-README.md`
- Tests: 段 1〜3 は既存のテストを変えずに通す(例外は承認済みの `report_messages.rs` の組み立ての値)。段 4 はテストを組み直し、まとめた 6 件の一覧を記録する

設計の出典: `construction/functional-design/functional-spec.md`(§1〜§4a)、**`inventory.md`(変える場所、目標の形、テストへの影響。実装はこれに従う)**、`rules.md`(BR1.1〜BR5.3)。追跡の ID は成功の指標 `SM1`〜`SM6`。テストは Minimal、test-after、行カバレッジ 80% の下限(Build and Test が全体で確かめる)。新しい依存、`unsafe`、`cfg` は足さない。

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

- [x] Step 1: 既存のテストの実行環境を確かめ、基準の件数を記録する(`cargo test --locked --workspace`。基準: 1,123 通過、1 無視)。
- [x] Step 2: 段 1(cli)を実装する: 1a(`definition_run_id`、`read_checked_state`、`StateRead`。prepare と graph の順序と、`read_state`/`read_state_raw` の呼び分けを保つ。watch は `read_context`)、1b(`CheckedDefinition.commands`、`Checked`/`Session` の `commands`、dry_run と `report_execution` と `step::run_step` の使い回し。誤りは全件を集める)、1c(`check_definition_and_env` を `ctx.rs` へ)、1d(入口の共通化、prepare の部品化、run はジョブの位置を受け取る。順序を保つ)。項目ごとにコミット。(BR1.1〜BR1.4。SM1)
- [x] Step 3: 段 1 の層のテストを実行する: 既存のテストを変えずに全体が通り、件数が基準と同じ。`git diff` で段 1 のコミットがテストのファイルを変えていないことを確かめる。`inventory.md` の 1a〜1d の「今」の場所に重複が残っていないことを確かめる。(BR5.1、BR5.2。SM5)
- [x] Step 4: 段 2(型)を実装する: 2a(core に `Classification::as_str`、`TransitionOn::as_str`、`Classification::transition_on`。`DisplayEvent` の `JobResult.result` を `Option<Classification>`、`Transition.on` を `TransitionOn` に。表示の側で文字列に。**承認済みの例外として、`crates/cli/tests/report_messages.rs` の `with_all_events` と `v_の文言` の組み立ての値と import だけを直し、期待する出力の文字列は変えない**)、2b(`JobStatus`/`SkipReason` の `as_str`、`parse`。writer、reader、watch)、2c(`check_consistency` と型を core へ。runtime は再エクスポート。runtime の `consistency.rs` は薄いファイルにし、`mod tests` が使う名前をすべて use して、`mod tests` は 1 文字も変えない)、2d(`read_bounded`。誤りの写像を保ち、`read_limited` は残す)。項目ごとにコミット。(BR2.1〜BR2.4。SM2)
- [x] Step 5: 段 2 の層のテストを実行する: 全体が通り、件数が基準と同じ。テストのファイルの変更は `report_messages.rs` の組み立ての値と import だけ(期待の文字列に差分がないこと)。重複が残っていないことを確かめる。(BR5.1、BR5.2。SM5)
- [x] Step 6: 段 3(core)を実装する: 3a(`walk.rs` → `walk/{mod,retry,env}.rs`。`git mv` で `walk.rs` を `walk/mod.rs` にしてから分ける)、3b(`DEFAULT_SUCCESS_EXIT_CODES`)、3c(`exit_code_of`)、3d(`depth.rs` の文を `MAX_NESTING_DEPTH` から作る。converter は `const _: () = assert!(crate::raw::MAX_DEPTH == 32);`。表示の上限を error に寄せる。文は 1 文字も変えない)、3e(`empty_step`)、3f(`JobExecution::exited` と simulate での使用)。項目ごとにコミット。(BR3.1〜BR3.3。SM3)
- [x] Step 7: 段 3 の層のテストを実行する: 全体が通り、件数が基準と同じ。段 3 のコミットがテストのファイルを変えていないこと。重複が残っていないこと。(BR5.1、BR5.2。SM5)
- [x] Step 8: 段 1〜3 の仕様書を直す: `docs/spec/rust` の 02・04・05・06・07・10・12 章の内部の名前(`prepare_checked`、`check_definition_and_env`、`DryRunTarget`、`run_individual`、`aux_commands.rs`、`consistency.rs`、`walk.rs` など)を、新しい置き場所と名前に合わせる。(SM1〜SM3)
- [x] Step 9: 段 4(テストと文書)を実装し、テストを実行する: 4a(`crates/core/tests` の graph のテストを、形ごとのファイル `graph_shape_skip.rs`、`graph_shape_same_target.rs`、`graph_shape_both.rs`、`graph_shape_branch.rs` と `graph_drawable.rs` に組み直す。補助 `lanes`、`both`、`letter_definition` を `graph_common` に。`inventory.md` の 5 組と性能の 1 件をまとめ、ほかは入力と期待を落とさずに移す。64 件 → 58 件)、4b(converter のテスト名 117 件を日本語に。`env_probe_child` を改名するなら `nfr_security.rs` の `--exact` の文字列も同時に。ファイル単位でコミット)、4c(`fake_wait` と `JobExecution::exited` で偽物の重複をなくす。`flow_retry.rs` のフィールドは残す)、4d(README の終了コード 68 の行と関連の行を実装に合わせる。必須の語句を残す)、4e(5 つの仕様書の `00-README.md` の見出しの直後に凍結の明記)。テストのファイル名と件数を参照する仕様書(`docs/spec/rust/11-quality-and-build.md`、`docs/spec/rust/05-execution-flow.md`、`docs/spec/language-agnostic/11-quality-and-build.md` の AC-475、AC-476a〜e と converter のテスト名 24 個)も直す。全体が通ること(件数は基準から 6 件減る)。`docs_*.rs` と README の必須の語句が通ること。(BR4.1〜BR4.5。SM4)
- [x] Step 10: 全体の確認: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --locked --workspace` が通ること(行カバレッジは Build and Test が確かめる)。(BR5.3。SM6)
- [x] Step 11: 文書と追跡: `code-summary.md`(段ごとの件数、まとめた 6 件の一覧、テストの変更の一覧)、`traceability.json` を作る。

## 守ること

- 利用者から見える動き(出力、終了コード、ステート、`--dry-run` の出力)を変えない。
- 段 1〜3 は既存のテストを変えない(例外は承認済みの `report_messages.rs` の組み立ての値と import)。
- 新しい外部ライブラリを足さない(`Cargo.toml`、`Cargo.lock` に差分を出さない)。`unsafe` と `cfg(unix)`/`cfg(windows)` を足さない。converter を core に依存させない。core に入出力を持ち込まない。
- コミットは項目ごとの小さな単位。各コミットでビルド、clippy、全体のテストが通ること。メッセージは日本語。
- ファイルは、ファイルの作成と編集のツールで書く(シェルの sed、perl、python で書き換えない。`git mv` は計画が名指しする)。
- 範囲の外: `args.rs`、`runner.rs` の分割、converter と core の重複、5 つの仕様書の中身。

## Assumptions

なし(新しい内部の型 `CheckedDefinition`、`StateRead` と、型の移動は、設計の `entities.md` に書いた)。
