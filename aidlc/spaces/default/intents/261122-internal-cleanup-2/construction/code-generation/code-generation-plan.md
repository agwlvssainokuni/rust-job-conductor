# コード生成の計画(Code Generation Plan): 動きを変えない整理の第 2 弾

## Summary

- Builds: 4 つの段の整理。段 1: 説明のずれ(`apply_skip`、`Plain`、escape、`lib.rs` の表、`config_path`・65・`read_state_raw`、`definition_key`、仕様書 11 章の件数)と制約(core・converter の `forbid(unsafe_code)`、上限の数の const assert)。段 2: 小さな重複と計算量(`status_at` の共有と `pending_ids`、`merge_env`、既定の定義ファイル名、`TransitionTarget` の表示、`<file>:<line>`、graph の共通、使い方の誤り・出力の失敗・終了コードの補助、ステートの削除、escape を core に、性能のテスト 3 件)。段 3: cli の ctx(`report_execution` の移動、`Session` と `Checked`、prepare の重複、`run_id` モジュール)。段 4: 型とテスト(`RunIdArg`、`SkipCause`/`RejectReason`、別名、`From`、`TransitionOn::parse`、`flow_with_state.rs`)
- Touches: `crates/core/src`(flow、envfile、definition、graph、plan、types、error、lib)、`crates/converter/src`(lib、error)、`crates/runtime/src`(lib、state、error、file_create、platform/unix.rs の「親が空なら .」)、`crates/cli/src`(ctx、step、run、pipeline、dry_run、aux_commands、graph、init、args、types、exit、deps、lib、run_id(新規)、report/*)、テスト(承認済みの T1〜T9 と §6 の新しいテスト)、`docs/spec/rust`、`docs/spec/language-agnostic/08`
- Tests: 既存のテストは変えずに通す。例外は承認済みの T1〜T9(期待する出力の文字列は変えない)。足すのは性能のテスト 3 件と小さな関数の単体テスト 4 件(1,128 → 1,135 の見込み)

設計の出典: `construction/functional-design/functional-spec.md`(§1〜§9。**§4 のテストの変更はユーザーが承認済み**、§5 の性能の基準、§8 のコミットの分け方)、**`inventory.md`(変える場所、目標の形、影響するテスト。実装はこれに従う)**、`rules.md`(BR1.1〜BR5.4)、`entities.md`。追跡の ID は成功の指標 `SM1`〜`SM6`。テストは Minimal、test-after、行カバレッジ 80% の下限(Build and Test が全体で確かめる)。新しい依存、`unsafe` は足さず、`src/` の `platform` の外に `cfg` を足さない。

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

この intent の層は、各クレートの内部の論理(Business logic にあたる)と、その説明・仕様書。データモデルの形(ステートファイルの書式)、API、画面の層は変えない。

## 手順

- [x] Step 1: 既存のテストの実行環境を確かめ、基準の件数を記録する(`cargo test --locked --workspace`。基準: 1,128 通過、1 無視、0 失敗)。
- [x] Step 2: 段 1(説明と制約)を実装する: `functional-spec.md` §8 のコミット 1〜9(1a〜1h)。T8 はコミット 3。`#![forbid(unsafe_code)]` の説明文に語 `unsafe` を書いて `nfr_source.rs` の検査に触れないように注意。項目ごとにコミット。(BR1.1〜BR1.8。SM1)
- [x] Step 3: 段 1 の層のテストを実行する: 全体が通り、件数が基準と同じ。テストのファイルの変更は T8 のコメントだけ。(BR5.1〜BR5.3。SM5)
- [x] Step 4: 段 2(小さな重複と計算量)を実装する: §8 のコミット 10〜25(2a〜2k)。計算量の 2 件は、**直す前に、§5 の測り方(debug、5 回の中央値)で今の時間を測って記録してから直す**。graph の text の行き先は今と同じ `escape_path`(64 文字で切らない)を保ち、遷移の表示の `escape_name` との使い分けを変えない。T1 はコミット 18。15(`.env` の解析の重複の除去)は任意。項目ごとにコミット。(BR2.1〜BR2.11。SM2)
- [x] Step 5: 段 2 の層のテストを書いて実行する: 性能のテスト `crates/core/tests/flow_perf.rs`(A2。100 回で上限 50 ms)と `crates/core/tests/env_merge_perf.rs`(A3a 上限 50 ms、A3b 上限 100 ms)を足す。直した後の時間(A3a を含む)を測って記録する。65 文字以上のジョブ ID の graph の text の出力が今と同じであることを、既存のテストで確かめているか見て、無ければ足す。全体が通ること。件数は基準 + 足した分。テストのファイルの変更は T1 と新しいファイルだけ。(BR2.x、BR5.x。SM2、SM5)
- [x] Step 6: 段 3(cli の ctx)を実装する: §8 のコミット 26〜30(3a〜3e)。prepare の検査の順序(シグナル → 定義 → command → ジョブ → `--assume` → run ID → `.env` → ステート)と、graph の順序を変えない。項目ごとにコミット。(BR3.1〜BR3.4。SM3)
- [x] Step 7: 段 3 の層のテストを実行する: `flow_run.rs`・`flow_pipeline.rs`・`flow_dry_run.rs`・`flow_run_id.rs`・`flow_env.rs` を含む全体が変えずに通ること。(BR5.x。SM3、SM5)
- [x] Step 8: 段 4(型とテスト)を実装する: §8 のコミット 31〜36(4a〜4e)。T2〜T7、T9 はここ。`PlanMode`・`EdgeKind`・`EdgeTarget` は別名(`pub use … as`)にして graph と plan のテストは変えない。項目ごとにコミット。(BR4.1〜BR4.4。SM4)
- [x] Step 9: 段 4 の層のテストを書いて実行する: §6 の単体テスト 4 件(`TransitionOn::parse`、`TransitionTarget::label`、`From<Classification> for JobStatus`、`RunIdArg::parse`・`as_str`)。テスト名は日本語。全体が通ること。`flow_with_state.rs` は 20 件のまま。(BR4.x、BR5.x。SM4、SM5)
- [x] Step 10: 全体の確認: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --locked --workspace`(失敗 0、件数は基準 + 足した分)。`functional-spec.md` §7 の grep の確かめ。テストのファイルの差分が T1〜T9 と新しいファイルだけであること。(BR5.4。SM6)
- [x] Step 11: 文書と追跡: `code-summary.md`(段ごとの件数、性能の前後の時間、テストの変更の一覧、計画からのずれ、見送ったこと)、`traceability.json` を作る。

## 守ること

- 利用者から見える動き(出力、終了コード、ステートファイル、`--dry-run` の出力、誤りの報告の順)を変えない。
- 既存のテストを変えるのは、承認済みの T1〜T9 だけ。期待する出力の文字列は変えない。
- 新しい外部ライブラリを足さない(`Cargo.toml`、`Cargo.lock` に差分を出さない)。`unsafe` を足さない。`src/` の `platform` の外に `cfg` を足さない。converter を core に依存させない。エラーに入力の値を含めない。ステートに環境変数を書かない。
- コミットは項目ごとの小さな単位。各コミットでビルド、clippy、全体のテストが通ること。メッセージは日本語。
- ファイルは、ファイルの作成と編集のツールで書く(シェルの sed、perl、python で書き換えない)。
- 範囲の外: B3(core の `read_definition` の順序)、F(`run_job` に分割済みのコマンドを渡す)、`JobExecution` の列挙型化、`EdgeList` の削除、凍結した 5 つの仕様書、版の更新。

## Assumptions

なし(新しい型 `RunIdArg`、`SkipCause`、`RejectReason`、`TransitionTarget::END_LABEL` などは、設計の `entities.md` に書いた)。
