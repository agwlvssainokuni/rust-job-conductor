# コード生成の計画(Code Generation Plan): graph の text 表示の改善 その 3

## Summary

- Builds: `graph` の text で、交差する飛び越しを、`git log --graph` と同じ見た目の図で描く(交差の行 2 行と、横線 `_`)
- Touches: `crates/core/src/graph/{layout,mod}.rs`、`crates/cli/src/report/graph_text.rs`(`_` の 1 文字)、core と cli のテスト(書き換えと、正解データ 940 例の照合)、README、`docs/spec/rust`、`docs/spec/language-agnostic`
- Tests: 正解データ 940 例の照合 1 本、C1〜C3、`-v`、`--state`、辺の一覧に残る形(混ざった例)、性能、実バイナリ(合計 12 本ほど)と、書き換える既存のテスト

設計の出典: `construction/functional-design/functional-spec.md`(§2〜§6b)と `rules.md`(BR1.1〜BR5.1)。追跡の ID は成功の指標 `SM1`〜`SM3`。テストは Minimal、test-after、行カバレッジ 80% の下限(Build and Test が全体で確かめる)。新しい依存、`unsafe`、`cfg` は足さない。**git のソース(GPL v2)は読まない**(BR5.1)。規則は、設計の文書と、試作(読み取りのみ)に従う。

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
- [x] Step 2: 判定(core)を実装する: `crates/core/src/graph/layout.rs` の `skip_targets` で、BR1.1 の範囲の条件(グラフ全体: すべてのジョブが次の辺を持つ、各ジョブの飛び越しは高々 1 本、「両方の飛び越し」と分岐がない、行き先が終点も含めて全部違う)を満たすなら、交差を許す。満たさないグラフでは、従来どおり交差を辺の一覧にする。(BR1.1、BR2.1。SM1)
- [x] Step 3: 配置の部品(core)を実装する: `crates/core/src/graph/mod.rs` に `LaneGlyph::Underscore` を足し、`lane_layout` に、位置 `k` の線を取り除く処理、交差の行(行 A、行 B)、ノードの行の記号の優先順(BR1.5)を足す。線ごとの付随データを、同じ添字で取り除く。`after_fork` を、`k ≥ 1` の到着のあとで偽にする。内側が先頭の仕様と、`open` の逆順の読み替えに注意(functional-spec.md の §5a)。(BR1.2〜BR1.5。SM1)
- [x] Step 4: 整形(cli)を実装する: `crates/cli/src/report/graph_text.rs` の `glyph_char` に `Underscore` → `_` を足す。ほかは変えない。(BR1.4。SM1)
- [x] Step 5: 配置の層のテストを書いて実行する(core): `layout-oracle-cross.md`(940 例)を `crates/core/tests/fixtures/graph_layout_oracle_cross.md` に置き、`graph_layout_oracle.rs` で全例の一致を照合する。既存の 3 つの正解データ(537、1,517、871 例)は、そのまま通ること。ジョブ 8 個の 3,817 例は、手元で 1 回だけ、`/private/tmp/claude-501/-Users-agawa-Documents-project-git-rust-job-conductor/8e1e0ddf-1216-413b-8fe8-d56bac43f7ad/scratchpad/oracle-cross.md`(読み取りのみ。リポジトリに写さない)と照合して、結果を報告する。(BR4.1。SM1、SM2)
- [x] Step 6: 既存のテストを、新しい期待に書き換える(functional-spec.md の §6 の一覧。実際に失敗するものを確かめる): `graph_drawable.rs`、`graph_lane_polish.rs`、`graph_branch_lanes.rs`、`graph_report.rs`、`binary_graph.rs`、`graph_perf.rs`、`docs_graph.rs`。(BR2.1、BR3.1。SM1、SM2)
- [x] Step 7: 追加の例のテストを書いて実行する(core): C1〜C3(functional-spec.md の §4)、辺の一覧に残る混ざった例(交差と、離れた別のジョブの「両方の飛び越し」、分岐、同じ行き先)、性能(幅の広い交差の最悪の形で、1,000 ジョブでも 1 秒以内)。`crates/core/tests/graph_crossing_lanes.rs`(新規)。(BR1.1〜BR2.1。SM1、SM2)
- [x] Step 8: 整形の層のテストを書いて実行する(cli): `graph_report.rs` に、C1 の text の全文、`_` を含む C2、`-v`、`--state`。実バイナリ(`binary_graph.rs`)で C1。(SM1)
- [x] Step 9: README を直す: 辺の一覧に切り替わる形(569 行目あたり)を、新しい範囲に改め、交差の図の例(C1)を、`graph_example` の添字がずれないよう、最後に足す。`docs_graph.rs` を合わせる。(BR4.2。SM3)
- [x] Step 10: 仕様書を直す: functional-spec.md の §6a の場所(`docs/spec/rust` と `docs/spec/language-agnostic` の 05、08、11、12)。ほかの 5 つの仕様書と `demo/` は変えない。(BR4.2。SM3)
- [x] Step 11: 全体の確認: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --locked --workspace` が通ること(行カバレッジは Build and Test が確かめる)。
- [x] Step 12: 文書と追跡: `code-summary.md`、`traceability.json`(`SM1`〜`SM3` の各行に、実装またはテストのファイルを `target` に書く)を作る。

## 守ること

- 新しい外部ライブラリを足さない(`Cargo.toml`、`Cargo.lock` に差分を出さない)。`unsafe` と `cfg(unix)`/`cfg(windows)` を足さない。
- **git のソースを読まない**(BR5.1)。git の出力を実行して観察する(読み取りのコマンド)のは構わない。
- 交差以外(通常の飛び越しで交差しない形、同じ行き先、両方の飛び越し、分岐)の出力は、1 文字も変えない(既存の正解データ 537、1,517、871 例が、そのまま通る)。
- コミットは小さな単位(判定、配置、整形の `_`、正解データの照合、書き換え、追加の例、README、仕様書 の順)。メッセージは日本語。
- ファイルは、ファイルの作成と編集のツールで書く(シェルの heredoc や Python で書かない)。

## Assumptions

なし(データのモデルの要素は足さない。`LaneGlyph::Underscore` は、既存の列挙型の値の追加で、設計の entities.md に書いた)。
