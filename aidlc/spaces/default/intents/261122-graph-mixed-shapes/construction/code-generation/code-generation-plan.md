# コード生成の計画(Code Generation Plan): graph の text 表示の改善 その 4

## Summary

- Builds: `graph` の text で、通常の飛び越しだけのグラフ(交差に、同じ行き先が混ざる形を含む)を、`git log --graph` と同じ見た目の図で描く。配置を、重複を許す「線の並びと寄せ」の 1 つの仕組みに置き換える
- Touches: `crates/core/src/graph/layout.rs`、core と cli のテスト(書き換えと、正解データ 487 例の照合)、README、`docs/spec/rust`、`docs/spec/language-agnostic`
- Tests: 正解データ 487 例の照合 1 本、M2〜M4、`-v`、`--state`、辺の一覧に残る形、性能、実バイナリ(合計 10 本ほど)と、書き換える既存のテスト

設計の出典: `construction/functional-design/functional-spec.md`(§2〜§6b)、`rules.md`(BR1.1〜BR5.1)、実行できる参照実装 `layout-reference.py`(同じ記録。Python)。追跡の ID は成功の指標 `SM1`〜`SM3`。テストは Minimal、test-after、行カバレッジ 80% の下限(Build and Test が全体で確かめる)。新しい依存、`unsafe`、`cfg` は足さない。**git のソース(GPL v2)は読まない**(BR5.1)。なお、設計の確認(アーキテクトの読み直し)は、承認の時点で、まだ終わっていなかった。結果が届いたら、その指摘を、コード生成の確認で扱う。

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
- [x] Step 2: 判定(core)を実装する: `crates/core/src/graph/layout.rs` の `is_crossing_class` から、「飛び越しの行き先が、終点も含めて、どの 2 本でも違う」の条件を外し、「通常の飛び越しだけのグラフ」(すべてのジョブが次の辺を持つ、各ジョブの飛び越しは高々 1 本、「両方の飛び越し」と分岐がない)の判定にする(名前も直してよい)。(BR1.1。SM1)
- [x] Step 3: 配置(core)を実装する: 通常の飛び越しだけのグラフの配置を、`layout-reference.py` の仕組み(重複を許す `lanes`、`MOVE(c, d)`、寄せる順序、ノードの行の記号)に置き換える。**「両方の飛び越し」と分岐を含むグラフの配置は、今のまま残す。** 内側が先頭の仕様と、`open` の逆順の読み替えに注意。1,000 ジョブでも 1 秒以内に収まる計算量にする(重複の組の探索を、ノードごとに全探索しない)。(BR1.2〜BR1.5、BR2.1。SM1)
- [x] Step 4: 配置の層のテストを書いて実行する(core): `layout-oracle-mixed.md`(487 例)を `crates/core/tests/fixtures/graph_layout_oracle_mixed.md` に置き、`graph_layout_oracle.rs` で全例の一致を照合する。既存の 4 つの正解データ(537、1,517、871、4,757 例)は、そのまま通ること。ジョブ 7 個の 3,861 例は、手元で 1 回だけ、`/private/tmp/claude-501/-Users-agawa-Documents-project-git-rust-job-conductor/8e1e0ddf-1216-413b-8fe8-d56bac43f7ad/scratchpad/oracle-mixed7.md`(読み取りのみ。リポジトリに写さない。ジョブ 4〜7 個の全 4,348 例を含む)と照合して、結果を報告する(一時の `#[ignore]` のテストで。実行後に削除)。(BR4.1。SM1、SM2)
- [x] Step 5: 既存のテストを、新しい期待に書き換える(functional-spec.md の §6。実際に失敗するものを確かめる): 「交差と同じ行き先が混ざる形は辺の一覧」とするもの(`graph_lane_polish.rs`、`graph_drawable.rs`、`graph_crossing_lanes.rs`、`graph_report.rs` の E1 と英語の注記と `-v` の辺の一覧、`binary_graph.rs`、`graph_perf.rs`、`docs_graph.rs`)。辺の一覧が残るのは、「両方の飛び越し」と分岐を含むグラフ。(BR2.1、BR3.1。SM1、SM2)
- [x] Step 6: 追加の例のテストを書いて実行する(core): M2〜M4(functional-spec.md の §4)、辺の一覧に残る形(交差と、離れた「両方の飛び越し」、分岐)、性能(幅の広い、同じ行き先が重なる形で、1,000 ジョブでも 1 秒以内)。`crates/core/tests/graph_mixed_shapes.rs`(新規)。(BR1.1〜BR2.1。SM1、SM2)
- [x] Step 7: 整形の層のテストを書いて実行する(cli): `graph_report.rs` に、M2 の text の全文、`_` を含む M4、`-v`、`--state`。実バイナリ(`binary_graph.rs`)で M2。(SM1)
- [x] Step 8: README を直す: 辺の一覧に切り替わる形を、新しい範囲に改め、M2 の図の例を、`graph_example` の添字がずれないよう、最後に足す。`docs_graph.rs` を合わせる。(BR4.2。SM3)
- [x] Step 9: 仕様書を直す: functional-spec.md の §6a の場所(`docs/spec/rust` と `docs/spec/language-agnostic` の 05、08、11、12)。ほかの 5 つの仕様書と `demo/` は変えない。(BR4.2。SM3)
- [x] Step 10: 全体の確認: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --locked --workspace` が通ること(行カバレッジは Build and Test が確かめる)。
- [x] Step 11: 文書と追跡: `code-summary.md`、`traceability.json`(`SM1`〜`SM3` の各行に、実装またはテストのファイルを `target` に書く)を作る。

## 守ること

- 新しい外部ライブラリを足さない(`Cargo.toml`、`Cargo.lock` に差分を出さない)。`unsafe` と `cfg(unix)`/`cfg(windows)` を足さない。
- **git のソースを読まない**(BR5.1)。git の出力を実行して観察するのは構わない。
- 前回までの出力は、1 文字も変えない(既存の 4 つの正解データが、そのまま通る。「両方の飛び越し」と分岐を含むグラフの処理は、触らない)。
- コミットは小さな単位(判定、配置、正解データの照合、書き換え、追加の例、README、仕様書 の順)。メッセージは日本語。
- ファイルは、ファイルの作成と編集のツールで書く(シェルの heredoc や Python で書かない)。

## Assumptions

なし(データのモデルの要素は足さない)。
