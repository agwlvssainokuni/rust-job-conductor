# コードの計画(Code Generation Plan): cli の本番の小さなモジュールの統合

上流: `construction/functional-design/functional-spec.md`(§2 の項目ごとの形、§4 の変えるテスト T1、§6 の件数の見込み、§7 の確かめ方、§8 のコミットの並びと §8.1 の履歴の追い方、§10 の決定 D1〜D7)、`rules.md`(BR)、`entities.md`、`inventory.md`(変える場所、仕様書の S1〜S24、完了の判定)。

方針: テストのコードの都合で本番のコードを曲げない(ユーザーの判断)。処理は変えず、置き場所と可視性と `use` だけを変える。利用者から見える動き(出力、終了コード、ステートファイル、`--dry-run`、シグナルの扱い)は変えない。版は上げない。

方法: 1 項目 1 コミットで `main` に直接コミットする。各コミットで fmt、clippy(macOS と Windows の対象)、cli のテストを通す。ファイルを消したコミットは、本文に「元: <元のパス>」を書く。push、タグ、リリースはしない。

## 守ること

- 処理のコードは変えない。変えるのは置き場所、可視性(D1〜D3)、`use`、説明の文だけ。
- 可視性: `run_job_with_retry` は非公開(D1)。`escape_name`・`escape_full` は `pub(crate)`(D2。`mod report;` が非公開なので今の実効の範囲と同じ)。`platform` は `pub(crate) use unix::spawn_watcher as start_signal_watcher;` の形で、`unix.rs`・`windows.rs` の `spawn_watcher` を `pub(crate)` にする(D3。runtime と同じ)。
- 置き場所: `run_job_with_retry` は `run_step` の直後(D4)。エスケープの 2 関数は `push_line` の直前(D5)。`aux_commands.rs` は `run_convert` の後に `run_graph`・`read_overlay`・`run_init` を置き、元のモジュールの説明(規則の番号)を各関数の説明へ移し、見出しに graph と init と分け方の基準を書く(D6)。
- `platform` のディレクトリと `mod.rs`・`unix.rs`・`windows.rs` の名前は変えない。`cfg(unix|windows)` は `platform` の中だけ。
- 既存のテストの変更は T1(`src/tests/escape_table.rs` の `use` の 1 行)だけ。名前・確かめの行・期待の値は変えない。
- 仕様書 rust は S1〜S24 を直す。版の履歴の注記は足さない(D7)。言語に依らない仕様書と凍結した仕様書は変えない。
- 新しい依存を足さない。`unsafe` を足さない。
- ドキュメントコメントは日本語。
- 想定外に、T1 の外の既存のテストを変えないと進めない項目が出たら、その項目は進めずに止めて報告する。

## 手順

- [x] Step 1: `retry.rs` を `step.rs` へまとめ、`retry.rs` を消す(項目 1。`lib.rs` の `mod retry;` とモジュールの表、`run.rs` の説明)。仕様書の該当の S。コミット 1。
- [x] Step 2: `report/escape.rs` を `report/mod.rs` へまとめ、`report/escape.rs` を消す(項目 2。report の 3 ファイルの `use`、T1)。`report/mod.rs` の説明の「`escape` を通す」を、消えるモジュールの名前を指さない文に直したことを目で確かめる(レビューの R-04)。仕様書の該当の S。コミット 2。
- [x] Step 3: `graph.rs` と `init.rs` を `aux_commands.rs` へまとめ、2 つを消す(項目 3。`lib.rs` の `mod` と呼び出し)。仕様書の該当の S。コミット 3。
- [x] Step 4: `platform/mod.rs` の中継を再公開の形にする(項目 4)。仕様書の該当の S。コミット 4。
- [x] Step 5: `functional-spec.md` §7 の確かめ方を走らせて結果を記録する: 消したファイルが無いこと、消えたモジュールのパスの参照が 0 件、仕様書の古い名前が 0 件(最後の `grep -v` の除外をかける前の出力も一度見る。レビューの R-03)、テスト名の比較(差 0)、件数(1,149 のまま)、`cargo doc` の警告(基準の 5 件のまま)、Windows の対象の clippy、本番の形の clippy、cli の `nfr_source` の 6 件。fmt、clippy、全テスト、カバレッジを通す。
- [x] Step 6: `code-summary.md`(コミットの一覧、件数、消したファイルとまとめ先、検索の結果、計画からのずれ)、`traceability.json`(SM と BR のすべてを、変えたファイルに結ぶ)、`memory.md`。

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
