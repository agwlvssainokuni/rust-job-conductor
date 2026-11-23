# コードの計画(Code Generation Plan): 型と構造の整理・計算量・利用者から見える変更

上流: `construction/functional-design/functional-spec.md`(§2 の段ごとの設計、§4 のテストの変更 T1〜T26、§5 の足すテスト N1〜N12 と性能の上限、§6 の件数の見込み、§7 の確かめ方、§8 のコミットの並び 1〜37、§10 の決定 D1〜D10 と既定で進めるもの)、`rules.md`(BR)、`entities.md`、`inventory.md`(変える場所、§0 の基準と性能の直す前の測定、§8 の検索のコマンド)。

方法: test-after(各項目の実装のコミットのあと、その項目のテストを足す)。小さなコミットで `main` に直接コミットし、各コミットでビルド、clippy、テストが通るようにする。push、タグ、リリースはしない。

## 守ること

- 段 1・2 では、利用者から見える動き(出力、終了コード、ステートファイル、`--dry-run` の出力)を変えない。文言の移動(`label()`、`messages.rs` の統一)で、日本語・英語の文字列を 1 文字も変えない。
- 既存のテストの変更は、承認済みの T1〜T26(T17a・T17b を含む)だけ。確かめる中身と期待する出力の文字列は変えない。例外は ★ T17a(消える項目の 1 行)、★ T24(文法の誤りの理由の期待)、★ T25(版)。テストの補助の使わなくなった引数は `_` で始めて残し、呼ぶ側は変えない。
- 新しい依存を足さない(`Cargo.lock` の差分は版の行だけ)。`unsafe` と src の `cfg(unix|windows)` を足さない。runtime の製品のコードに `.unwrap()`・`.expect(`・`panic!(`・`unreachable!(`・`todo!(`・`unimplemented!(` を書かない(`nfr_source.rs`)。converter は core に依存しない。誤りの表示に入力の値を含めない。
- 凍結した 5 つの仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)は変えない。
- ドキュメントコメントとテスト名は日本語。
- 想定外に、T1〜T26 の外の既存のテストを変えないと進めない項目が出たら、その項目は進めずに止めて報告する。

## 段 1: 型と構造(コミット 1〜21)

- [x] Step 1: 計画の型(`ResumeInfo`、`PlanDisposition::WillRun`、`NextStepNote`。1a、1b、1e)と図の節(`NodeKind`、D1。1c)。T1〜T8。
- [x] Step 2: `Backoff`(1d)と `RetryDecision::Retry` の `last_exit_code`・`max`(1f)、N1。T9、T10、T16、T26。
- [x] Step 3: 読まれない項目を消す(`JobOutcome`・`NextStep` の `job_id`、`JobExecution` の `job_id`・`command`、組み立ての関数と `run_job` の `job` の引数。1g、D2)。T5、T10〜T14、★ T17a、T17b、T18〜T23。仕様書に「v0.9.6 までは `run_job(job, line, env, signal)`」。
- [x] Step 4: `EnvFileError::reason()` と `render_error`(1h)、N2。T15。
- [x] Step 5: cli の `Checked.base`、`prepare_session`(D3)、`SkipRecord`(1i)。
- [x] Step 6: 種類の `label()`(D10)と `messages.rs` の関数の形の統一(1j)、N3。
- [x] Step 7: 外から使われない `pub` を狭める(1k、D9)。段 1 の後の件数を記録(見込み 1,149)。

## 段 2: 計算量(コミット 22〜30)

- [x] Step 8: 計画の模擬は複製しない(`apply_*_in`、D8。2a)と N4・N5。重ね合わせ(2b)と `flow_edges`(2c)を表で引くことと N6・N7。`.env` の重複を位置の表で(2d)と N8。`merge_env_names`(2e)。各項目で、直す前の木でもテストを走らせて時間を記録する。上限は §5.2 の表(P1・P2 150 ms、P3 350 ms、P4 600 ms、P5 100 ms。debug、5 回の中央値)。仕様書 11 章の性能の表。段 2 の後の件数を記録(見込み 1,154)。

## 段 3: 利用者から見える変更と版(コミット 31〜37)

- [x] Step 9: 文法の誤りの理由を形式ごとに(「TOML として読めません」「YAML として読めません」、内部の守り 2 か所は「入力の構造を解釈できません」。3a、D4、D5)、★ T24、N10、仕様書と「v0.9.6 までは…」の注記。
- [x] Step 10: `.env` の読み込みを形 B に(3b)、N11(直す前の木で落ちることを記録)、N12(直す前と後の両方で通ることを記録)、仕様書と「v0.9.6 までは…」の注記。
- [x] Step 11: 版を 0.9.7 にする(`ee9f409` と同じ範囲。★ T25)。段 3 の後の件数を記録(見込み 1,157)。

## 最後

- [x] Step 12: `functional-spec.md` §7 の確かめ方(検索と、段の前後の `cargo test --locked --workspace -- --list` の比較)を走らせて結果を記録する。fmt、clippy、全テスト、カバレッジを通す。
- [x] Step 13: `code-summary.md`(コミットの一覧、件数の移り変わり、テストの変更の一覧、性能の直す前と後の時間、検索の結果、計画からのずれ)、`traceability.json`(SM と BR のすべてを、変えたファイルに結ぶ)、`memory.md`。

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
