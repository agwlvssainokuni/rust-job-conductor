# コードの計画(Code Generation Plan): 実行の結果の型、run_job の入力、定義ファイルの読み込みの順序

上流: `construction/functional-design/functional-spec.md`(§2 の段ごとの設計、§4 のテストの変更 T1〜T18、§5 の足すテスト N1〜N5、§6 のコミットの並び、§7 の確かめ方、§9 の決定 D1〜D5)、`rules.md`(BR1.1〜BR4.4)、`entities.md`、`inventory.md`(変える場所のファイルと行)。

方法: test-after(各段の実装のコミットのあと、その段のテストを足す)。小さなコミットで `main` に直接コミットし、各コミットでビルドとテストが通るようにする。push はしない。

## 守ること

- 既存のテストの変更は、承認済みの T1〜T18(★ の T3、T9、T11、T15〜T18 を含む。D3)だけ。期待する出力の文字列の変更は T18 の 1 行だけ。
- 新しい依存を足さない(`Cargo.lock` の差分は版の行だけ)。`unsafe` と `cfg(unix|windows)` を足さない(`nfr_source.rs`)。誤りの表示に入力の値を含めない。converter は変えない(版の行を除く)。
- 凍結した 5 つの仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)は変えない。language-agnostic は段 2 の最小限と段 3 だけ(D2)。
- ドキュメントコメントとテスト名は日本語。
- 想定外に、ほかの既存のテストを変えないと進めない項目が出たら、その項目は進めずに報告する。

## 段 1: `JobExecution` の型(BR1.1〜BR1.5、SM1)

- [x] Step 1: `JobExecution::not_started` と `start_failed` を足し、runtime の `run_job` と cli の `retry.rs` が使う(今の項目の形のまま。動きは同じ。テストは変えない)。コミット 1。
- [x] Step 2: `JobExecution { job_id, command, result: ExecutionResult }` に切り替える。`ExecutionResult` は `NotStarted { signal }`、`StartFailed`、`Exited { exit_code, terminated_by_signal, interrupted_by }`(D5)。`types.rs` に置き、`conductor_core` のルートから公開する。`classify`、`retry_decision`、`apply_outcome`、`run_job`、`step.rs`、`retry.rs` と、T1〜T12 を同じコミットで直す。コミット 2。
- [x] Step 3: N1(`classify.rs` の単体。§2.1.3 の表の 6 行)。コミット 3。
- [x] Step 4: 仕様書(rust の 02、05、07、11、12)。language-agnostic は変えない(D2)。コミット 4。段 1 の後の件数を記録(見込み 1,138)。

## 段 2: `run_job` の入力(BR2.1〜BR2.4、SM2)

- [x] Step 5: `run_job(job, line: &CommandLine, env, signal)` にし、`Runtime` トレイト、`run_job_with_retry`、`run_step` を合わせる。リトライでは `session.checked.commands[i]` をそのまま渡す。`run_job` の中の分割をなくし、「`line.program` が空・空白だけなら `StartFailed`」の 1 行を、起動前の合図の確認より前に置く(D4)。引数の名前は `line`(`nfr_source.rs` の `Command::new(&line.program)`)。T13〜T17。コミット 5。
- [x] Step 6: N2(`runner_basic.rs`。分けられない `job.command` と、補助の実行ファイルを指す `line` を渡すと `Exited { Some(3), .. }`)。コミット 6。
- [x] Step 7: 仕様書(rust の 02、05、07、11、12 と、language-agnostic の 05 §1.2 の手順 1・§7.5、07 §3.6・§4.1・§4.3 の最小限)。コミット 7。段 2 の後の件数を記録(見込み 1,139)。

## 段 3: 定義ファイルの読み込みの順序(BR3.1〜BR3.6、SM3、SM4)

- [x] Step 8: `read_definition` を形 B にする(D1): パスで種別だけ見る(無い → FileMissing、通常のファイルでない → Unreadable。開かない)→ 開く(権限など → Unreadable)→ 開いたハンドルから大きさ(上限超過 → TooLarge)→ 上限 + 1 バイトまで読む → UTF-8 の確認。理由の文は今のものを使い、入力の値を含めない。コミット 8。
- [x] Step 9: N3(UNIX。mode 000 の 1 MiB 超 → Unreadable。開けたら確かめない)、N4(上限超過は UTF-8 でないより先)、N5(UNIX。FIFO は開かずに Unreadable。別のスレッドで呼び、5 秒で打ち切る)。N3 は直す前の木で落ちること、N5 は直す前と後の両方で通ることを記録する。コミット 9。
- [x] Step 10: 仕様書(rust の 03・02、language-agnostic の 03)。「v0.9.5 までは、読む権限のない大きなファイルは tooLarge だった」の注記(`--to` の注記と同じ書き方)。FIFO の「止まらない」は「種別を見てから開くまでの差し替えの競合を除く」。コミット 10。
- [x] Step 11: 版を 0.9.6 にする(`ea09b46` と同じ範囲: `Cargo.toml` × 4、`Cargo.lock`、T18 の `binary_basic.rs`、仕様書の表記)。コミット 11。段 3 の後の件数を記録(見込み 1,142)。

## 記録

- [x] Step 12: `code-summary.md`(コミットの一覧、件数の移り変わり、テストの変更の一覧、計画からのずれ)、`traceability.json`(SM1〜SM5 と BR のすべてを、変えたファイルに結ぶ)、`memory.md`。

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
