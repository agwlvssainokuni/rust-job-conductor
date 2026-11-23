# コードの計画(Code Generation Plan): 残りの整理と読み込みの順序のそろえ

上流: `construction/functional-design/functional-spec.md`(§2 の段ごとの設計、§4 のテストの変更 T1〜T24 と消すテスト §4.6、§5 の足すテスト N1〜N8 と性能の上限、§6 の件数の見込み、§7 の確かめ方、§8 のコミットの並び 1〜46、§10 の決定 D1〜D12 と承認済みの項目)、`rules.md`(BR)、`entities.md`、`inventory.md`(変える場所、§0 の基準と性能の直す前の測定、§8 の検索のコマンド)。

方法: test-after(各項目の実装のコミットのあと、その項目のテストを足す)。小さなコミットで `main` に直接コミットし、各コミットでビルド、clippy、テストが通るようにする。push、タグ、リリースはしない。

## 守ること

- 段 1 では、利用者から見える動き(出力、終了コード、ステートファイル、`--dry-run` の出力)を変えない。名前や渡し方の変更で、表示の文字列を 1 文字も変えない。
- 既存のテストの変更は、承認済みの T1〜T24 と、消すテスト(§4.6 の 4 つ)だけ。確かめる中身と期待する出力の文字列は変えない。例外は ★ T1b(確かめ方の置き換え)、★ T22(版)、★ T23(期待を `Unreadable` に、名前を `文字デバイスへのリンクのステートは開かずに_unreadable` に)。
- cli の `Runtime::run_job` から `job` を外す(Q3)。テストの偽物は環境変数 `CONDUCTOR_JOB_ID` からジョブ ID を取り、無いときはパニックで止める(D1)。今の記録と台本の引き当てを保つ。
- 新しい依存を足さない(`Cargo.lock` の差分は版の行だけ)。`unsafe` と src の `cfg(unix|windows)` を足さない。core・converter の src のテストに `cfg(unix)` を増やさない(名前付きパイプのテストは `tests/` に置く)。runtime の製品のコードに `.unwrap()`・`.expect(`・`panic!(`・`unreachable!(`・`todo!(`・`unimplemented!(` を書かない。converter は core に依存しない。誤りの表示に入力の値を含めない。コードのコメントに版の番号を書かない。
- 凍結した 5 つの仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)は変えない。
- ドキュメントコメントとテスト名は日本語。
- 想定外に、T1〜T24 と §4.6 の外の既存のテストを変えないと進めない項目が出たら、その項目は進めずに止めて報告する。

## 段 1: 動きを変えない整理(コミット 1〜36)

- [x] Step 1: 古くなった説明と守りの理由(1a)。コミット 1〜2。
- [x] Step 2: 使われないもの・二重の計算・常に真の条件(1b、`apply_interrupt_in`、`EffectiveEnv` の `job_id`・`names()`、補助の使わない引数(D2)、模擬の分類、`decide`、`interrupted_signal`、`command_line`)と、記録の `hint`(D4)。T1a、★ T1b、T2、T3。コミット 3〜10。
- [x] Step 3: 二重の処理(`replace_via_temp`、ステートの読み込みの段、`bounded_read.rs`(`read_limited` は段 2 まで `pub(crate)`)、`build_retry`(D3)、ごく小さい整理)。T4。コミット 11〜15。
- [x] Step 4: cli の `Runtime::run_job` から `job` を外す(1d、Q3、D1)。T15 の偽物。コミット 16。
- [x] Step 5: 公開範囲を狭める(1e、1f。`read_limited` を除く)。T5。コミット 17〜19。
- [x] Step 6: 型と名前(`escape_full`(D6)、`StateContext`(D5)、converter の行番号と `reason`)。T16〜T19。コミット 20〜23。
- [x] Step 7: 計算量(1h)と N1〜N3。上限は P1 350 ms、P2 1,600 ms、P3 40 ms(D9。debug、5 回の中央値)。直す前の木でも走らせて時間を記録する。`-v` の図と `record_skip` は前後の時間だけ記録する(D8)。コミット 24〜29。
- [x] Step 8: テストの補助の整理(1i。T6〜T13、T15 の残り、T20、T21、T24)と、重複したテストを消す(§4.6 の 4 つ、D10)。仕様書 11 章。コミット 30〜36。段 1 の後の件数を記録(見込み 1,156。基準 1,157 − 4 + 3)。

## 段 2: 利用者から見える変更と版(コミット 37〜46)

- [x] Step 9: converter の読み込みを形 B に(2a)と N4(直す前の木で落ちることを記録)、仕様書と「v0.9.7 までは…」の注記。コミット 37〜39。
- [x] Step 10: ステートの読み込みを形 B に(2b、D7)と ★ T23、`read_limited` を非公開に、N5・N8、仕様書 06 章と注記。コミット 40〜42。
- [x] Step 11: 空白だけのコマンド名を `Empty`(64)に(2c)と N6・N7、仕様書と注記。コミット 43〜45。
- [x] Step 12: 版を 0.9.8 にする(`7239aa2` と同じ範囲。★ T22)。コミット 46。段 2 の後の件数を記録(見込み 1,161)。

## 最後

- [x] Step 13: `functional-spec.md` §7 の確かめ方(検索と、段の前後の `cargo test --locked --workspace -- --list` の比較)を走らせて結果を記録する。fmt、clippy、全テスト、カバレッジを通す。
- [x] Step 14: `code-summary.md`(コミットの一覧、件数の移り変わり、テストの変更と消したテストの一覧、性能の直す前と後の時間、検索の結果、計画からのずれ)、`traceability.json`(SM と BR のすべてを、変えたファイルに結ぶ)、`memory.md`。

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
