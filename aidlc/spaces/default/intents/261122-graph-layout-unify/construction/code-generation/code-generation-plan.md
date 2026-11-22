# コード生成の計画(Code Generation Plan): graph の配置のリファクタリング

## Summary

- Builds: `graph` の配置を 1 つの仕組み(線の並びと寄せ)にし、古い経路を削除する。「両方の飛び越し」と分岐を、この仕組みの中で描き、交差・同じ行き先との組み合わせも図にする。主線の列 0 は、実行されうるジョブからの辺があるところだけ `|`(`-v` の詳細の行も同じ)
- Touches: `crates/core/src/graph/{layout,mod}.rs`、`crates/cli/src/report/graph_text.rs`(`-v` の列 0)、core と cli のテスト(書き換えと、正解データの差し替え・追加)、README、`docs/spec/rust`、`docs/spec/language-agnostic`
- Tests: 正解データ 2,748 + 871(作り直し)+ 300 例の照合、N1〜N5・B1・D1 の全文、`-v`、`--state`、性能(両方の飛び越し・分岐を含む 1,000 ジョブ)、実バイナリ(合計 12 本ほど)と、書き換える既存のテスト

設計の出典: `construction/functional-design/functional-spec.md`(§2〜§6b)、`rules.md`(BR1.1〜BR5.1)、**規範の参照実装 `layout-reference.py`**(Python。文と食い違うときは、こちらが正)。追跡の ID は成功の指標 `SM1`〜`SM4`。テストは Minimal、test-after、行カバレッジ 80% の下限(Build and Test が全体で確かめる)。新しい依存、`unsafe`、`cfg` は足さない。**git のソース(GPL v2)は読まない**(BR5.1)。

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

- [x] Step 1: 既存のテストの実行環境を確かめる(`cargo test -p conductor-core --test graph_layout_oracle` が動くこと。コマンドは `unit-test-instructions.md`)。
- [x] Step 2: 判定(core)を実装する: `crates/core/src/graph/layout.rs` の `skip_targets` を、functional-spec.md の §6b の 3(ジョブごとの辺の数と種類)で、レーンの図にするかを決める形にする。`SkipPlan` は、ジョブごとの「次のジョブへの辺の有無」「種類(飛び越しなし、通常、両方の飛び越し、分岐)」「行き先(0〜2 個)」を持つ。`ordinary` の欄と `is_ordinary_skip_graph` は削除する。(BR1.1。SM1、SM3)
- [x] Step 3: 配置(core)を実装する: `ordinary_layout` を唯一の配置にし、「両方の飛び越し」(線 1 本)と分岐(線 2 本、近い行き先が内側)の線の開き方と、主線の列 0 の規則(実行されうるジョブからの辺。実行されうるジョブは、先頭のジョブから辺をたどって求める)を足す。`move_rows`、分岐の行、合流の行、寄せの行の列 0 に、その記号を渡す。古い経路(`lane_layout`、`open_all_off_main`、「分岐が開いた」印、`debug_assert`、古い経路だけが使う補助)を削除する。`is_lane_drawable` の名前は変えない。規範は `layout-reference.py`。(BR1.2〜BR1.4。SM1、SM3)
- [x] Step 4: `-v` の詳細の行の列 0(core と cli): ノードごとの列 0 の記号を、配置の結果(`LaneLayout` か `LaneRow`)に持たせ、cli の `crates/cli/src/report/graph_text.rs` の `render_lanes` が、詳細の行の先頭に、それを使う(functional-spec.md の §6b の 1)。ほかの整形は変えない。(BR1.4。SM3)
- [x] Step 5: 配置の層のテストを書いて実行する(core): `crates/core/tests/graph_layout_oracle.rs` の読み取りに、`a>>d`(両方の飛び越し)を足す。分岐の正解データ `crates/core/tests/fixtures/graph_layout_oracle_branch.md` を、`layout-oracle-branch-v2.md`(871 例)に置き換える(`cp`)。`layout-oracle-offmain.md`(2,748 例)を `crates/core/tests/fixtures/graph_layout_oracle_offmain.md` に置く(`cp`)。ジョブ 6〜7 個の確認用に、`layout-reference.py` の `layout` で、乱数の種 20261122、ジョブ数 6 か 7、「両方の飛び越し」か分岐を 1 つ以上含む、重複のない 300 例を生成し、`crates/core/tests/fixtures/graph_layout_oracle_offmain_67.md` に置く(生成のコマンドは、この計画が名指しする。生成の手順は、ファイルの先頭に書く)。git の出力の正解データ(537、1,517、940、487 例)は、変えずに通ること。全部の正解データを、件数と 1 文字ずつ照合する。(BR3.1、BR4.1。SM2、SM3)
- [x] Step 6: 既存のテストを、新しい期待に書き換える: 「両方の飛び越し」と分岐の図の期待のうち、主線の列 0 の規則で変わるもの(`graph_lane_polish.rs`、`graph_branch_lanes.rs`、`graph_report.rs`、`binary_graph.rs`、`docs_graph.rs`、README の例)。組み合わせを「辺の一覧」とするテスト(`graph_drawable.rs`、`graph_crossing_lanes.rs`、`graph_mixed_shapes.rs`、`graph_report.rs` の辺の一覧の例、`binary_graph.rs`)を、図の期待にする。辺の一覧に残る形のテストは、3 本以上の辺を持つジョブなど(§6b の 3 の範囲の外)の形に差し替える。削除した名前を参照する `graph_types.rs`、`nfr_source.rs`、`crates/cli/src/graph.rs`、`graph_common` も直す。実際に失敗するものを確かめて直す。(BR2.1、BR3.1。SM2、SM3)
- [x] Step 7: 追加の例のテストを書いて実行する(core): N1〜N5、B1、D1 の全文(functional-spec.md の §4)、性能(「両方の飛び越し」と分岐を含む 1,000 ジョブで 1 秒以内)。`crates/core/tests/graph_layout_unify.rs`(新規)と `graph_perf.rs`。(BR1.1〜BR2.1。SM3)
- [x] Step 8: 整形の層のテストを書いて実行する(cli): `graph_report.rs` に、N1 の text の全文、N1・N2・N5 の `-v`(詳細の行の列 0)、`--state`。実バイナリ(`binary_graph.rs`)で N1。(BR1.4。SM3)
- [x] Step 9: README を直す: 「両方の飛び越し」と分岐の図の例(B1 など、変わる図)、辺の一覧に切り替わる形(3 本以上の辺を持つジョブなど)、N1 の例(最後に足す)。`docs_graph.rs` を合わせる。(BR4.2。SM4)
- [x] Step 10: 仕様書を直す: `docs/spec/rust` と `docs/spec/language-agnostic` の 05(15.4、15.5 を 1 つの仕組みとして書き直す)、08(図の例)、11(受け入れ基準、テストの表)、12。ほかの 5 つの仕様書と `demo/` は変えない。(BR4.2。SM4)
- [x] Step 11: 全体の確認: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --locked --workspace` が通ること(行カバレッジは Build and Test が確かめる)。
- [x] Step 12: 文書と追跡: `code-summary.md`、`traceability.json` を作る。

## 守ること

- 新しい外部ライブラリを足さない(`Cargo.toml`、`Cargo.lock` に差分を出さない)。`unsafe` と `cfg(unix)`/`cfg(windows)` を足さない。
- **git のソースを読まない**(BR5.1)。
- git の出力の正解データ(537、1,517、940、487 例)は、1 文字も変えない。
- コミットは小さな単位(判定、配置、`-v` の列 0、正解データ、書き換え、追加の例、README、仕様書 の順)。メッセージは日本語。
- ファイルは、ファイルの作成と編集のツールで書く(シェルの heredoc や Python で書かない。ただし、Step 5 の正解データの生成のコマンドは、計画が名指しする)。

## Assumptions

なし(データのモデルの要素は足さない。配置の結果に、ノードごとの列 0 の記号を持たせるのは、既存の `LaneLayout` か `LaneRow` の欄の追加で、設計の §6b の 1 に書いた)。
