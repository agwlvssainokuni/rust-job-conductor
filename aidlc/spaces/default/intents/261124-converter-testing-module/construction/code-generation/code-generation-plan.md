# コードの計画(Code Generation Plan): converter の testing モジュールの削除

上流: `construction/functional-design/functional-spec.md`(§2 の項目ごとの設計、§4 の移す・変える・消すテスト T1〜T7、§4.5 の消すテスト 1 件、§6 の件数の見込み、§7 の確かめ方、§8 のコミットの並び 1〜7、§10 の決定 D1〜D4)、`rules.md`(BR。特に BR3.2 のソースの検査との関係)、`entities.md`、`inventory.md`(変える場所、仕様書の S1〜S13、完了の判定)。

方針: テストのコードの都合で本番のコードを曲げない(ユーザーの判断)。converter の `testing` モジュールと `TargetFormat::opposite` を本番から消し、途中の表現を比べるテストは `src/tests/` の単体テストへ移す。`Value` の `PartialEq` は `#[cfg(test)]` の中だけにする。本番に、テストのための要素を新しく足さない。

方法: 小さなコミットで `main` に直接コミットする。テストを先に移し(コミット 1〜4)、移し終えてから本番を変える(5、6)。各コミットで fmt、clippy、converter のテスト(件数がコミットごとの見込みと合うこと)を通す。push、タグ、リリースはしない。版は上げない。

## 守ること

- 利用者から見える動き(変換の出力、誤りの表示、終了コード)を変えない。
- 既存のテストの変更は、承認済みの T1〜T7 と、消すテスト 1 件(`反対の形式は_yaml_と_toml_を入れ替え_2_回で元に戻る`)だけ。移すテストの名前と確かめの行と期待の値は変えない(変わるのは import、`testing::Value::` を `Value::` にする行、`include_str!` のパス、往復の補助の `opposite()` を `match` にする所)。
- 単体テストは `src/tests/` に置き、`#[cfg(test)] mod tests;` は `lib.rs` の最後に置く。src の直下のファイルに単体テストを足さない。製品のコードより前の説明やコメントに `#[cfg(test)]` の文字を書かない(BR3.2。`tests/nfr_security.rs` のソースの検査との関係)。
- core、runtime、cli は変えない。
- 新しい依存を足さない(`Cargo.toml`・`Cargo.lock` の差分なし)。`unsafe` と src の `cfg(unix|windows)` を足さない。converter は core に依存しない。
- 凍結した 5 つの仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)と言語に依らない仕様書は変えない(D4)。
- ドキュメントコメントとテスト名は日本語。
- 想定外に、T1〜T7 の外の既存のテストを変えないと進めない項目が出たら、その項目は進めずに止めて報告する。

## 手順

- [x] Step 1: `read_resolve.rs` の 28 件を `src/tests/read_resolve.rs` へ移し、`src/tests/mod.rs` に補助(`yaml`・`toml`・`s`・`i`・`m`・`a`・`with_timeout`)を置く。`lib.rs` の最後に `#[cfg(test)] mod tests;`。`tests/read_resolve.rs` を消す(T1。仕様書 S4、S6、S7、S8、S12)。コミット 1。
- [x] Step 2: 往復の木を比べる 13 件を `src/tests/convert_roundtrip.rs` へ移す(T2。仕様書 S9、S12)。コミット 2。
- [x] Step 3: 受け入れ基準の 2 件を `src/tests/convert_ac.rs` へ移す(T3。仕様書 S10、S12)。コミット 3。
- [x] Step 4: ランダムな入力の 2 件と補助を `src/tests/nfr_security.rs` へ移し、`tests/common/mod.rs` を `pub mod kinds;` だけにする(T4、T6。仕様書 S11、S12)。コミット 4。
- [x] Step 5: `testing` モジュールと `TargetFormat::opposite` を消し、`opposite` の単体テストを消す(T5。仕様書 S1、S2、S3)。コミット 5。
- [x] Step 6: `Value` の `PartialEq` を `#[cfg(test)]` にする(仕様書 S5)。コミット 6。
- [x] Step 7: `functional-spec.md` §7 の確かめ方を走らせて結果を記録する: 完了の判定の検索(`src/tests/` の禁止の語の検索を含む)、テスト名の比較(名前の最後の部分で比べ、差は消した 1 件だけ)、件数(見込み 1,152)、`cargo doc` の警告(基準の 5 件のまま)、Windows の対象の clippy、`cargo clippy -p conductor-converter --lib` が `PartialEq` なしで通ること、core・runtime・cli の差分が空。fmt、clippy、全テスト、カバレッジを通す。ソースの検査の範囲の記録(BR3.2)を残す。
- [x] Step 8: `code-summary.md`(コミットの一覧、件数の移り変わり、移したテストと消したテストの一覧、検索の結果、ソースの検査の範囲の記録、計画からのずれ)、`traceability.json`(SM と BR のすべてを、変えたファイルに結ぶ)、`memory.md`。

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
