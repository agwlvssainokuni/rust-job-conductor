# コードの計画(Code Generation Plan): 動きを変えない小さな整理とテストの補助

上流: `construction/functional-design/functional-spec.md`(§2 の段ごとの設計、§4 のテストの変更 T1〜T46、§5 の足すテスト N1〜N5、§6 の件数の見込み、§7 の確かめ方、§8 のコミットの並び 1〜49、§10 の決定 D1〜D5 と既定で進めるもの)、`rules.md`(BR)、`entities.md`、`inventory.md`(変える場所のファイルと行、§0 の基準の件数、§8 の検索のコマンド)。

方法: test-after(各項目の実装のコミットのあと、その項目のテストを足す)。小さなコミットで `main` に直接コミットし、各コミットでビルドとテストが通るようにする。push はしない。版は上げない。

## 守ること

- 利用者から見える動き(出力、終了コード、ステートファイル、`--dry-run` の出力)を変えない。文言の定数をまとめるときは、文字列が 1 文字も変わらないことを確かめる。
- 既存のテストの変更は、承認済みの T1〜T46 だけ(★ の T9、T19、T33、T37 と、§10 の既定で進めるものを含む)。テストの名前、件数(移動したものも数える)、確かめる中身、期待する出力の文字列は変えない。例外は T38(`program_name` の単体テスト 1 件の削除)だけ。
- 新しい依存を足さない(`Cargo.lock` の差分なし)。`unsafe` と src の `cfg(unix|windows)` を足さない(`nfr_source.rs`)。converter は core に依存しない。誤りの表示に入力の値を含めない。
- 凍結した 5 つの仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)は変えない。
- ドキュメントコメントとテスト名は日本語。
- 想定外に、T1〜T46 の外の既存のテストを変えないと進めない項目が出たら、その項目は進めずに止めて報告する。

## 段 1: 説明のずれと使われないもの(コミット 1〜5)

- [x] Step 1: 生成元の説明(1a)、`sanitize` の説明とテストの注記(1b、T40、D1)、`args.rs` の文法の表(1c)、仕様書 08・05 章(1d)、`program_name` とその単体テストの削除(1e、T38)。段の後の件数を記録(見込み 1,141)。

## 段 2: 文言の定数(コミット 6〜11)

- [x] Step 2: cli の `messages.rs` の文言の対をまとめる(2a)、`error_prefix!`(2a)、core の `env_name_rule!`・`reserved_env_prefix!`(2b、2c)、`ROOT_KEYS` の `VERSION_KEY` と間隔の定数の `assert`(2d、2e)、`CONDUCTOR_RUN_ID` の注記(2f、D2)。段の後の件数は 1,141 のまま。

## 段 3: 処理の重複と境界(コミット 12〜19)

- [x] Step 3: runtime の `temp_file` モジュール(`dir_or_current`、`temp_name`・`create_temp`・`temp_prefix` と単体テスト 4 件の移動 T22、`create_temp_beside` と連番 1 つ D5、`lib.rs` の表、仕様書 02・06 章)、N4。cli の `push_line`・`job_place`(3b、3c)、`reset_hint`・`StateContext::for_read`(3d)、`plan_end`(3e)、`graph_mermaid.rs` の let-else(3f)。段の後の件数を記録(見込み 1,142)。

## 段 4: 到達しない分岐と効かない処理(コミット 20〜26)

- [x] Step 4: decide の規則 5(4a)、lexopt の枝を 1 つの守りにまとめて注記(4b、D3)、`render` の `match`(4c)、`retry.rs` の既定値(4d)、`ExitCodeClass` を返す `classify_exit_code` と `is_retry_target` の共有(4e、D4)、`retry_integer` の `u32`(4f)、`duration.rs` の `n >= 1`(4g)。段の後の件数は 1,142 のまま。

## 段 5: 文字列での比較(コミット 27〜31)

- [x] Step 5: `CommandKind::ALL`・`from_name`(5a)と N1、`Operand`(5b)、誤りの対象を `Opt::subject` と `RUN_ID_OPTION` に(5c。`RUN_ID_OPTION` は `args.rs` の `const` の検査で `OPTIONS` と結ぶ)、runtime の `FailureCause`(5d)。段の後の件数を記録(見込み 1,143)。

## 段 6: walk の整理(コミット 32〜35)

- [x] Step 6: `SuccessCodes`(6a)、`build_jobs` の所有権(6b)、`integer_value`(6c)、`walk_children`(6d)。段の後の件数は 1,143 のまま。

## 段 7: テストの補助(コミット 36〜48)

- [x] Step 7: cli の `docs_support`(T1〜T6)、`status_of`・`status_in`(T7〜T9、T20)、`signal_support`(T10〜T13)、`args_grammar.rs` の `MapEnv`(T14)、偽物の `Default`・`writes`(T15、T16、T18)、台本の消費の順(T19)、`run_lines`(T17)と N2・N3。runtime の包みと `definition`(T21、T23〜T27)、`consistency` の単体テストを core へ(T28、T29)。core の `tests/common`(T30〜T37)、`JobExecution` の組み立ての関数(T39、T41〜T45)。converter の `TargetFormat::opposite` と N5(T46)。段の後の件数を記録(見込み 1,146)。

## 最後

- [x] Step 8: 仕様書 11 章のテストのファイルの表と、02 章の偽物の表(コミット 49)。
- [x] Step 9: 確かめ方(`functional-spec.md` §7)の検索を実装の後に走らせて結果を記録し、`cargo test --locked --workspace -- --list` の前後の差がテスト名の約束どおりであることを確かめる。fmt、clippy、全テスト、カバレッジを通す。
- [x] Step 10: `code-summary.md`(コミットの一覧、件数の移り変わり、テストの変更の一覧、確かめ方の検索の結果、計画からのずれ)、`traceability.json`(SM と BR のすべてを、変えたファイルに結ぶ)、`memory.md`。

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
