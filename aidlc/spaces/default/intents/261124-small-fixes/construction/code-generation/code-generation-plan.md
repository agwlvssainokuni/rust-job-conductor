# コードの計画(Code Generation Plan): 小さな直し

上流: `construction/functional-design/functional-spec.md`(§1〜§2 の項目ごとの設計、§4 のテストの変更 T1〜T4、§4.6 の消すテスト 3 件、§6 の件数の見込み、§7 の確かめ方、§8 のコミットの並び 1〜11、§10 の決定 D1〜D8)、`rules.md`(BR)、`entities.md`、`inventory.md`(直す説明 C1〜C4、仕様書の S1〜S14 と X1〜X5、完了の判定)。

方針: テストのコードの都合で本番のコードを曲げない(ユーザーの判断)。本番のコードで変えるのは説明の文だけ。利用者から見える動きは変えない。版は上げない。

方法: 1 項目 1 コミットで `main` に直接コミットする。各コミットで fmt、clippy、変えたクレートのテスト(件数がコミットごとの見込みと合うこと)を通す。仕様書だけのコミットは §7 の該当の検索で確かめる。push、タグ、リリースはしない。

## 守ること

- 利用者から見える動き(出力、終了コード、ステートファイル、`--dry-run` の出力)を変えない。本番のコードは説明の文(C1〜C4)だけを変える。
- 既存のテストの変更は、承認済みの T1〜T4 だけ。消すのは §4.6 の 3 件だけ。移すテスト 1 件は名前と本体を変えない。名前を変えるテストは無い。`signals_order.rs`・`signals_real.rs` は変えない。
- ソースの検査(T4)は D4 のとおり、`src/tests/` の外の製品のコードで、`#[path` と、`include!`・`include_str!`・`include_bytes!`(マクロの名前と `!`。行の空白を除いて探す)を止める。テストの数と名前は変えない。既知の限界を説明に書く。
- 仕様書は §8 の割り当てどおり直す。版の注記は rust 02 の 3 段落だけを「今の形」と「版の履歴」に分ける(D7)。言語に依らない仕様書は S12(D3)だけ。凍結した 5 つの仕様書は変えない。
- 新しい依存を足さない(`Cargo.toml`・`Cargo.lock` の差分なし)。`unsafe` と、製品のコードへの `cfg(unix|windows)` を足さない。
- ドキュメントコメントとテスト名は日本語。
- 想定外に、T1〜T4 の外の既存のテストを変えないと進めない項目が出たら、その項目は進めずに止めて報告する。

## 手順

- [x] Step 1: core の計画と次の動きの説明の、起動前の判断の関数の名前を直す(C1、C2。仕様書 rust 05 の 2 か所。D5)。コミット 1。
- [x] Step 2: core の「runtime が再公開する」の説明を直す(C3。D1)。コミット 2。
- [x] Step 3: 仕様書 rust 02 を直す(`resolve_run_id` の型 S1、`consistency` の一覧 S2・S3、版の注記を版の履歴に分ける S4〜S7。D2)。コミット 3〜5。
- [x] Step 4: 仕様書 rust 06 の実装の所在を直す(X2。D6)。コミット 6。
- [x] Step 5: core の重なる性能のテスト 1 件を消し(T1)、残る 1 件を `plan_simulate_perf.rs` へ移して `plan_perf.rs` を消す(T2。仕様書 S8〜S10、S12。D3、D8)。コミット 7〜8。
- [x] Step 6: cli の重なる init の型のテスト 2 件を消す(T3、C4。仕様書 S11)。コミット 9。
- [x] Step 7: cli のソースの検査で、製品のコードの include の 3 つのマクロも止める(T4。仕様書 S13、S14。D4)。コミット 10。
- [x] Step 8: `functional-spec.md` §7 の確かめ方を走らせて結果を記録する: 完了の判定の検索(古い説明、仕様書の型と一覧、版の注記、`plan_perf` の参照)、テスト名の比較(名前の最後の部分で比べ、差は消した 3 件だけ)、件数(見込み 1,149)、`cargo doc` の警告(基準の 5 件のまま)、Windows の対象の clippy、本番の形の clippy。fmt、clippy、全テスト、カバレッジを通す。
- [x] Step 9: `code-summary.md`(コミットの一覧、件数の移り変わり、消したテストと移したテスト、検索の結果、計画からのずれ)、`traceability.json`(SM と BR のすべてを、変えたファイルに結ぶ)、`memory.md`。

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
