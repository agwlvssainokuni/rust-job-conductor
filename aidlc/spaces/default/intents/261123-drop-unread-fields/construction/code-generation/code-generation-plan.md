# コードの計画(Code Generation Plan): 読まれない項目の削除

上流: `construction/functional-design/functional-spec.md`(§2 の項目ごとの設計、§4 のテストの変更 T1〜T20、§4.5 の ★ 2 件、§4.6 の消す確かめの行 13 行、§6 の件数の見込み、§7 の確かめ方、§8 のコミットの並び 1〜6 と複数のコミットにまたがるファイルの形、§10 の決定 D1〜D3)、`rules.md`(BR)、`entities.md`、`inventory.md`(変える場所、§0 の基準、仕様書の S1〜S21、完了の判定)。

方針: テストのコードの都合で本番のコードを曲げない(ユーザーの判断)。型から読まれない項目を消し、テストは名前の並び、値、行き先など、残る項目で確かめる。本番に、テストのための要素を新しく足さない。

方法: 1 項目 1 コミットで `main` に直接コミットする。各コミットで fmt、clippy、全テストのコンパイル(`--no-run`)、変えたクレートのテストが通るようにする。push、タグ、リリースはしない。版は上げない。

## 守ること

- 利用者から見える動き(出力、終了コード、ステートファイル、`--dry-run` の出力、graph の出力、重複キーの警告の行、実行時の環境変数の並び)を変えない。
- 既存のテストの変更は、承認済みの T1〜T20 と、§4.6 の確かめの行 13 行の削除だけ。期待する出力の文字列は変えない。名前の変更は ★ 2 件だけ。テストは消さない。
- cli、runtime、converter の本番のコード(src)は変えない(D1 の代案)。
- 定義の `format_version` と `description` には触れない(Q3: B)。
- 新しい依存を足さない(`Cargo.toml`・`Cargo.lock` の差分なし)。`unsafe` と src の `cfg(unix|windows)` を足さない。converter は core に依存しない。誤りの表示に入力の値を含めない。
- 凍結した 5 つの仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)は変えない。
- ドキュメントコメントとテスト名は日本語。
- 想定外に、T1〜T20 の外の既存のテストを変えないと進めない項目が出たら、その項目は進めずに止めて報告する。

## 手順

- [ ] Step 1: `FlowEdge.explicit` を消し、`target_position` は位置だけを返す(項目 1。T1〜T5、§4.6 の T2。仕様書 S1〜S4)。コミット 1。
- [ ] Step 2: 計画の `EnvNameEntry` から `origin` を消し、名前だけの構造体にする(項目 2a。D1。T6、★1。仕様書 S6〜S11)。コミット 2。
- [ ] Step 3: `EnvOrigin` と `EffectiveEntry.origin` を消す(項目 2b。T7〜T9、★2、§4.6 の T7・T8。仕様書 S12〜S16)。コミット 3。
- [ ] Step 4: `EnvFileContent.path` を消す(項目 3a。T10〜T13、T19、§4.6 の T10。仕様書 S17、S18、S21(D3))。コミット 4。
- [ ] Step 5: `EnvEntry.line` を消す(項目 3b。T14〜T18、T20、§4.6 の T14。仕様書 S19、S20)。コミット 5。
- [ ] Step 6: `functional-spec.md` §7 の確かめ方を走らせて結果を記録する: 完了の判定の検索(仕様書の直し漏れと、残す記述が残ることの両方)、テスト名の一覧の前後の比較(差は ★ 2 件だけ)、件数(見込み 1,153)、`cargo doc` の警告(基準の 5 件のまま)、Windows の対象の clippy、`git diff -- crates/cli/src crates/runtime crates/converter` が空。fmt、clippy、全テスト、カバレッジを通す。
- [ ] Step 7: `code-summary.md`(コミットの一覧、件数、テストの変更・名前の変更・消した確かめの行の一覧、検索の結果、計画からのずれ)、`traceability.json`(SM と BR のすべてを、変えたファイルに結ぶ)、`memory.md`。

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
