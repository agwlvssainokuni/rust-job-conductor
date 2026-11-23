# コードの計画(Code Generation Plan): テストだけが使う本番のコードの削除

上流: `construction/functional-design/functional-spec.md`(§2 の項目ごとの設計、§4 のテストの変更 T1〜T32、§4.6 の消すテスト 4 件、§6 の件数の見込み、§7 の確かめ方、§8 のコミットの並び 1〜25、§10 の決定 D1〜D8)、`rules.md`(BR)、`entities.md`、`inventory.md`(変える場所、§0 の基準、完了の判定)。

方針: テストのコードの都合で本番のコードを曲げない(ユーザーの判断)。本番から、テストだけが使う実装と公開範囲をなくし、テストは偽物の側か公開の入口で代わりを持つ。本番に、テストのための要素を新しく足さない。

方法: 1 項目 1 コミットで `main` に直接コミットする。各コミットで fmt、clippy、全テストのコンパイル(`--no-run`)、変えたクレートのテストが通るようにする。push、タグ、リリースはしない。版は上げない。

## 守ること

- 利用者から見える動き(出力、終了コード、ステートファイル、`--dry-run` の出力)を変えない。
- 既存のテストの変更は、承認済みの T1〜T32 と、消すテスト(§4.6 の 4 件)だけ。期待する出力の文字列は変えない。テストの名前は変えない。
- 新しい依存を足さない(`Cargo.toml`・`Cargo.lock` の差分なし)。`unsafe` と src の `cfg(unix|windows)` を足さない。runtime の製品のコードに `.unwrap()`・`.expect(`・`panic!(`・`unreachable!(`・`todo!(`・`unimplemented!(` を書かない。cli の src の単体テストでも `panic!(` を使わない。converter は core に依存しない。誤りの表示に入力の値を含めない。
- 凍結した 5 つの仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)は変えない。
- ドキュメントコメントとテスト名は日本語。
- 想定外に、T1〜T32 と §4.6 の外の既存のテストを変えないと進めない項目が出たら、その項目は進めずに止めて報告する。

## 削除(コミット 1〜9)

- [x] Step 1: `Command` の `From<&str>`・`From<String>` を消す(1a。テストの補助は D5 の形に)。コミット 1。
- [x] Step 2: `RunIdArg::as_str`(1b)と、`InterruptSignal` の `is_raised`・`PartialEq`・`Eq`(1c。`Default` は残す D1、`Debug` は `signal_number()` で書く D8)を消す。コミット 2〜3。
- [x] Step 3: 本番で使われない `Default` 4 つ(1d。`JobStatus`、`Durability`、`StateOverlay`、cli `Language`)を消す。コミット 4〜7。
- [x] Step 4: `sanitize`(1e。呼ぶ所は `escape_display` に)と `EnvFileError::reason()`(1f)を消す。コミット 8〜9。

## 公開範囲を狭める(コミット 10〜18)

- [x] Step 5: core の `decide_before_run`(2a。テストの補助 `decide` を消して `decide_from_state` を直接呼ぶ D7)、`flow::classify`(2b)、`MAX_RUN_ID_LEN`・`is_valid_env_name`・`is_reserved_env_name`・`effective_success_codes`(2c)を、使う範囲に合わせて最も狭くする(D6)。コミット 10〜15。
- [x] Step 6: cli の `escape_display` の再公開(2d)、`pub mod` 4 つ(2e)、根の `GraphOptions`・`DryRunOptions` の再公開(2f)をやめる。コミット 16〜18。

## 判断が要ったもの(コミット 19〜24)

- [x] Step 7: `RunIdError` を単位構造体 1 つに(3a。D2。language-agnostic の仕様書は変えない D4)。コミット 19。
- [x] Step 8: runtime の core の再公開をやめ、使う所を `conductor_core` に(3b。2 つのコミットにまたがる import は §8 の表の形)。コミット 20。
- [x] Step 9: 誤りの種類の `Display`・`Error`(3c)と、本番で使われない `Hash`(3d)を消す。コミット 21〜24。

## 最後

- [x] Step 10: `functional-spec.md` §7 の確かめ方を走らせて結果を記録する: 完了の判定の検索、テスト名の一覧の前後の比較、件数(見込み 1,153)、`cargo doc` の警告(基準の 5 件のまま)、Windows の対象の clippy。fmt、clippy、全テスト、カバレッジを通す。
- [x] Step 11: `code-summary.md`(コミットの一覧、件数の移り変わり、テストの変更・消したテストの一覧、検索の結果、計画からのずれ)、`traceability.json`(SM と BR のすべてを、変えたファイルに結ぶ)、`memory.md`。

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
