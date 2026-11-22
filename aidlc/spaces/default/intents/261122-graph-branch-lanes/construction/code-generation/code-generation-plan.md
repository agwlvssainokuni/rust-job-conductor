# コード生成の計画(Code Generation Plan): graph の text 表示の改善 その 2

## Summary

- Builds: `graph` の text で、「分岐」(成功と警告が、異なる、どちらも次でない行き先へ進む形)を、レーンの図で描く(2 本の線を同時に出し、主線の縦線を省く)
- Touches: `crates/core/src/graph/layout.rs`、`crates/core/tests/`(`graph_drawable.rs` ほかの書き換え、正解データ 871 例の照合の追加)、`crates/cli/tests/`(`docs_graph.rs` の書き換えと追加)、README、`docs/spec/rust`、`docs/spec/language-agnostic`
- Tests: 正解データ 871 例の照合 1 本、分岐の例(B1〜B4)、`-v`、`--state`、辺の一覧に残る形、実バイナリの確認(合計 10 本ほど)と、書き換える既存のテスト 3 つ

設計の出典: `construction/functional-design/functional-spec.md`(§2〜§6a)と `rules.md`(BR1.1〜BR4.2)。要件の分析とユニットの分割は省略した計画。追跡の ID は成功の指標 `SM1`〜`SM3`(functional-spec.md の §7)。テストは Minimal、test-after、行カバレッジ 80% の下限(Build and Test が全体で確かめる)。新しい依存、`unsafe`、`cfg` は足さない。

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

## 手順

- [x] Step 1: 既存のテストの実行環境を確かめる(`cargo test -p conductor-core --test graph_drawable` が動くこと。コマンドは `unit-test-instructions.md`)。
- [x] Step 2: 判定(core)を実装する: `crates/core/src/graph/layout.rs` の `skip_targets` を、ジョブごとの飛び越しの行き先の並び(0〜2 個)に広げ、分岐(次のジョブへの辺がなく、辺が 2 本で、どちらも次でなく、行き先が異なる)を許す。`!has_next` の判定から分岐を免除する。積み上げに「分岐が開いた」印を持たせる。分岐は、積み上げが空か、`t2 < top` のときだけ載せる。通常のジョブの合流が、分岐の開いた線に対して起きる形は、辺の一覧にする。(BR1.1、BR2.1、BR2.2。SM1)
- [x] Step 3: 線の行の作り方(core)を実装する: 同じ `layout.rs` の `lane_layout` に、分岐のノードの行(幅は 2 × (1 + 既存の線の数 + 2))と、分岐の行(列 0 を空にし、`\` を (既存の線の数 + 2) 個、空白 1 つおき)を足し、2 本の線(外側 t2、内側 t1)を積む。行き先での合流の行の列 0 は、その線を開いたジョブがすべて主線につながらない(分岐、「両方の飛び越し」)ときだけ空。`after_fork` は新しい線を作るジョブ(通常の新しい線と、分岐)のあとだけ真。(BR1.2〜BR1.5。SM1)
- [x] Step 4: 配置の層のテストを書いて実行する(core): `crates/core/tests/fixtures/graph_layout_oracle_branch.md`(`layout-oracle-branch.md` の写し。871 例)を、`graph_layout_oracle.rs` で、1 文字も違わず照合する。既存の正解データ(537 例と 1,517 例)の照合は、そのまま通ること。(BR4.1。SM1、SM2)
- [x] Step 5: 既存のテストを、新しい期待に書き換える(core): `graph_drawable.rs` の `対応しない形_異なる行き先が_2つ`(「a の成功が c、警告が e」は、図にできる)と、`graph_lane_polish.rs` の `辺の一覧に残る形_異なる行き先が_2つ`(交差の形、または分岐の線への合流の形に差し替える)。分岐の追加の判定(図にできる形、できない形)を足す。(BR2.1、BR2.2、BR3.1。SM1、SM2)
- [x] Step 6: 追加の例のテストを書いて実行する(core): B1〜B4(functional-spec.md の §4)と、辺の一覧に残る形(分岐の線への合流、`t2 >= top` の分岐、交差)。`crates/core/tests/graph_branch_lanes.rs`(新規)。(BR1.1〜BR2.2。SM1)
- [x] Step 7: 整形の層のテストを書いて実行する(cli): `crates/cli/tests/graph_report.rs` に、B1 の text の全文、`-v`、`--state`(B1)を足す。`graph_text.rs` は変えない(分岐の行が `Fork` の記号を含むことに頼る)。(SM1)
- [x] Step 8: 実バイナリのテストを書いて実行する(cli): `crates/cli/tests/binary_graph.rs` に、分岐の定義で `conductor graph` を動かして、B1 の図を確かめるテストを足す。(SM1)
- [x] Step 9: README を直す: `README.md` の「辺の一覧に切り替わる形」(595 行目あたり)を、交差する形と、分岐の線への合流を含む形などに改め、分岐の図の例(B1)を足す。`crates/cli/tests/docs_graph.rs` の `readme_の辺の一覧に切り替わる形は_交差と異なる行き先の_2_つだけを述べる` を、新しい記述に合わせて書き換え、分岐の例が実出力と一致するテストを足す。(BR4.2。SM3)
- [x] Step 10: 仕様書を直す: `docs/spec/rust` と `docs/spec/language-agnostic` の 05(配置、辺の一覧に切り替わる形)、08(図の例、辺の一覧の説明)、11(AC-474、AC-475、テスト数)。functional-spec.md の §6a の場所。ほかの 5 つの仕様書と `demo/` は変えない。(BR4.2。SM3)
- [x] Step 11: 全体の確認: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --locked --workspace` が通ること(行カバレッジは Build and Test が確かめる)。
- [x] Step 12: 文書と追跡: `code-summary.md`、`traceability.json`(`SM1`〜`SM3` の各行に、実装またはテストのファイルを `target` に書く)を作る。

## 守ること

- 新しい外部ライブラリを足さない(`Cargo.toml`、`Cargo.lock` に差分を出さない)。`unsafe` と `cfg(unix)`/`cfg(windows)` を足さない。
- 通常の飛び越し、同じ行き先、両方の飛び越し、交差の出力は、1 文字も変えない(既存の正解データ 537 例と 1,517 例が、そのまま通る)。
- コミットは小さな単位(判定、線の行、正解データの照合、書き換え、追加の例、整形、README、仕様書 の順)。メッセージは日本語。
- ファイルは、ファイルの作成と編集のツールで書く(シェルの heredoc や Python で書かない)。

## Assumptions

なし(データのモデルの要素は足さない。`SkipPlan` と「分岐が開いた」印は core の内部で、公開しない)。
