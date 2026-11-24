# コードの計画(Code Generation Plan): テストのファイルの統合

上流: `construction/functional-design/functional-spec.md`(§2 の項目ごとの形、§4 の移すテスト T1〜T13、§6 の件数の見込み、§7 の確かめ方(D5 の最終確認を含む)、§8 のコミットの並び 1〜13 と §8.1 のファイルの移動と履歴、§10 の決定 D1〜D8)、`rules.md`(BR)、`entities.md`、`inventory.md`(まとめるファイル、仕様書の S1〜S16 と L1〜L10、完了の判定)。

方針: テストのコードの都合で本番のコードを曲げない(ユーザーの判断)。テストの名前・確かめの行・期待の値は変えず、ファイルをまとめ、補助とひな形の原本の写しを 1 つにする。本番のコードは変えない。利用者から見える動きは変えない。版は上げない。

方法: 1 項目 1 コミットで `main` に直接コミットする。各コミットで fmt、clippy、変えたクレートのテスト(件数がコミットごとの見込みと合うこと)を通す。新しいファイルへ移すときは `git mv`、既にあるファイルへまとめるときは同じコミットで足して元を `git rm` し、コミットの本文に「元: <元のパス>」を書く。push、タグ、リリースはしない。

## 守ること

- 利用者から見える動きを変えない。本番のコード(各クレートの `src/` のうち `src/tests/` の外)、`Cargo.toml`、`Cargo.lock`、CI の設定、README は変えない。
- テストの名前・確かめの行・期待の値は変えない。本体で変わるのは、D1・D2 の 5 行(`RETRY_HEADING`・`ENV_HEADING`・`long_id_definition`)と、D3 の `mod raw` の字下げだけ。消すテスト・足すテスト・名前を変えるテストは無い。
- 補助は `crates/cli/tests/common/fakes.rs` の 1 つにし、統合テストは `mod fakes;`、単体テストは `#[path = "../../../tests/common/fakes.rs"]` で読む(D は Q3: A)。先頭に `Language` の 2 つの定義を食い違わせない注意書きを置く。
- ひな形の原本は `crates/cli/tests/fixtures/init_ja.toml`・`init_en.toml` にし(D8)、両方のテストから `include_str!` で読む。中身は今の原本とバイト単位で同じ。リポジトリの根に `.gitattributes`(`crates/cli/tests/fixtures/*.toml text eol=lf` の 1 行)を置く(D5)。
- 製品のコードの検査(cli の `nfr_source` 6、runtime の `nfr_source` 5、converter の `nfr_security` 10 + 1 無視)は、数と中身を変えずに通す。CI が名前で指すテストのファイルは変えない。
- 仕様書は §8 の割り当てどおり直す。言語に依らない仕様書は L1〜L10(D6)だけで、772 行目は直さない(D7)。凍結した 5 つの仕様書は変えない。
- 新しい依存(dev-dependencies を含む)を足さない。`unsafe` と、製品のコードへの `cfg(unix|windows)` を足さない。
- ドキュメントコメントとテスト名は日本語。
- 想定外に、T1〜T13 の外の既存のテストを変えないと進めない項目が出たら、その項目は進めずに止めて報告する。

## 手順

- [x] Step 1: converter のテストを移す(T1: `convert_roundtrip.rs` の 1 件を `convert_ac.rs` へ、T2: 単体の `convert_ac.rs` の 2 件を `convert_roundtrip.rs` へ、T3: `tests/common` を `nfr_security.rs` へ)。コミット 1〜3。
- [x] Step 2: runtime のテストを移す(T4: `runner_env.rs` を `runner_basic.rs` へ、T5: `state_read_raw.rs` を `state_read.rs` の `mod raw` へ。D3)。コミット 4〜5。
- [x] Step 3: cli の小さな単体テストを移す(T6: `init_types.rs` を `args_grammar.rs` の init の節へ、T7・T8: `run_id_arg.rs` を `--run-id` の節へ移し、`fr2_5_` の 2 件を新しい `--remove-on-complete` の節へ(D4)、T9: `graph_long_id.rs` を `graph_report.rs` へ(D2))。コミット 6〜8。
- [x] Step 4: cli の README のテストを `src/tests/docs.rs` の 1 つにまとめる(T10: 単体の 3 つと補助、T11: 統合の 5 つと補助。D1)。コミット 9〜10。
- [x] Step 5: cli のテストの補助を `tests/common/fakes.rs` の 1 つにする(T12)。コミット 11。
- [x] Step 6: init のひな形の原本をデータのファイルにし、`.gitattributes` を置く(T13。D5、D8)。コミット 12。
- [x] Step 7: `functional-spec.md` §7 の確かめ方を走らせて結果を記録する: 完了の判定の検索(無くなったファイルの名前が仕様書と crates に残らない)、テスト名の比較(名前の最後の部分で差 0)、件数(1,149 のまま。cli 単体 172、統合 284)、移したテストの本体の比べ(変わるのは上の 5 行と字下げだけ)、`cargo doc` の警告(基準の 5 件のまま)、Windows の対象の clippy、本番の形の clippy、製品のコードの検査の数。fmt、clippy、全テスト、カバレッジを通す。D5 の最終確認の項目を「未確認(push の後の CI で確かめる)」として設ける。
- [x] Step 8: `code-summary.md`(コミットの一覧、件数の移り変わり、移したテストとまとめたファイル、検索の結果、D5 の最終確認の項目、計画からのずれ)、`traceability.json`(SM と BR のすべてを、変えたファイルに結ぶ)、`memory.md`。

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
