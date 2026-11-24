# コードの計画(Code Generation Plan): cli のライブラリの公開範囲を狭める

上流: `construction/functional-design/functional-spec.md`(§1 公開の判定(残す 13・狭める 15)、§2 項目ごとの設計(§2.3 のソースの検査の範囲 D2)、§4 の移す・変えるテスト T1〜T10、§6 の件数の見込み、§7 の確かめ方、§8 のコミットの並び 1〜12 とコミットごとの件数と補助の置き方、§10 の決定 D1〜D7)、`rules.md`(BR。特に BR3.2)、`entities.md`、`inventory.md`(変える場所、仕様書の S1〜S24、完了の判定)。

方針: テストのコードの都合で本番のコードを曲げない(ユーザーの判断)。cli のライブラリの公開範囲を、`main` が使う 4 つ、依存の注入の継ぎ目、継ぎ目に要る型(`SignalError`)だけにし、それ以外は crate の中に狭める。狭めたものを直接呼ぶテストは `src/tests/` の単体テストへ移す。本番に足すのは `lib.rs` の最後の `#[cfg(test)] mod tests;` だけ。

方法: 小さなコミットで `main` に直接コミットする。テストを先に移し(コミット 1〜11)、最後に公開を狭める(12)。各コミットで fmt、clippy、cli のテスト(件数がコミットごとの見込みと合うこと)を通す。push、タグ、リリースはしない。版は上げない。

## 守ること

- 利用者から見える動き(出力、終了コード、ステートファイル、`--dry-run` の出力)を変えない。本番の処理のコードは変えない(変えるのは `lib.rs` の公開範囲・説明・末尾の宣言と、説明の文だけ)。
- 既存のテストの変更は、承認済みの T1〜T10 だけ。移すテストの名前、確かめの行、期待の値は変えない(変わるのは import、`conductor_cli::` を `crate::` にする所、補助の参照)。消すテストと足すテストは無い。
- 単体テストは `src/tests/` に置き(元と同じファイル名)、`#[cfg(test)] mod tests;` は `lib.rs` の最後に置く(D1)。
- `nfr_source.rs` の変更は D2 のとおり: `src/tests/` を OS 分岐と `panic!(` の検査の外にし、代わりに `lib.rs` の末尾が `#[cfg(test)] mod tests;` であることと、`src/tests/` の外に `#[path` が無いことを同じテストで確かめる。`unsafe`・`low_level`・ネットワークの検査は `src/tests/` も対象のまま。テストの数は変えない。
- `src/tests/common/mod.rs` に `#![allow(dead_code)]` を付けない。各コミットには、そのコミットで使う補助だけを置く。
- core、runtime、converter は変えない。
- 新しい依存を足さない(`Cargo.toml`・`Cargo.lock` の差分なし)。`unsafe` と、製品のコードへの `cfg(unix|windows)` を足さない。cli は `#![forbid(unsafe_code)]` のまま。
- 凍結した 5 つの仕様書と言語に依らない仕様書は変えない(D6)。
- ドキュメントコメントとテスト名は日本語。
- 想定外に、T1〜T10 の外の既存のテストを変えないと進めない項目が出たら、その項目は進めずに止めて報告する。

## 手順

- [ ] Step 1: `src/tests/` の置き場所を作り、`exit_codes.rs` を移す。`nfr_source.rs` を D2 の形に直す(コミット 1)。
- [ ] Step 2: `escape_table.rs`、`run_id_arg.rs`、`init_types.rs` を移す(コミット 2〜4)。
- [ ] Step 3: `args_grammar.rs` を移す(コミット 5)。
- [ ] Step 4: `graph_report.rs` と `graph_long_id.rs`、`plan_report.rs`、`init_template.rs` を移す(コミット 6〜8)。
- [ ] Step 5: `report_messages.rs` と `kinds.rs` を移し、補助(`FakeSink`、`state_error` など)を足す(コミット 9。仕様書 S7)。
- [ ] Step 6: `docs_*` の 8 件を移す(コミット 10)。
- [ ] Step 7: `flow_init.rs`(期待のひな形を原本の写しで書く。D4)と `flow_validate.rs` を直し、`tests/common` にテスト用の `Language` を足す(D5)(コミット 11)。
- [ ] Step 8: 公開を狭める(`parse_args`、`types` の 11 個、`pub mod exit`・`report`・`run_id` を crate の中に。`lib.rs` の説明)(コミット 12。仕様書 S1〜S6)。
- [ ] Step 9: `functional-spec.md` §7 の確かめ方を走らせて結果を記録する: 完了の判定の検索(公開の項目が一覧と一致、`tests/` から狭めた名前の参照が 0 件、仕様書に古い記述が残らない)、テスト名の比較(名前の最後の部分で比べて差 0)、件数(1,152 のまま)、`cargo doc` の警告(基準の 5 件のまま)、Windows の対象の clippy、本番の形(`--lib --bins`)の clippy、core・runtime・converter の差分が空。fmt、clippy、全テスト、カバレッジを通す。
- [ ] Step 10: `code-summary.md`(コミットの一覧、件数の移り変わり、公開の判定、移したテストと書き方を変えたテスト、検索の結果、ソースの検査の範囲の記録、計画からのずれ)、`traceability.json`(SM と BR のすべてを、変えたファイルに結ぶ)、`memory.md`。

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
