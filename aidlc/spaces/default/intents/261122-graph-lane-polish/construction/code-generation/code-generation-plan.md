# コード生成の計画(Code Generation Plan): graph の text 表示の改善

## Summary

- Builds: `graph` の text で、「同じ行き先」と「両方の飛び越し」をレーンの図で描く(`|\|` の合流と、線なしの `*` の行)
- Touches: `crates/core/src/graph/layout.rs`、`crates/cli/src/report/messages.rs`、`crates/core/tests/`(`graph_drawable.rs` の書き換えと、正解データの照合の追加)、README、`docs/spec/rust`、`docs/spec/language-agnostic`
- Tests: 正解データ 1,517 例の照合 1 本、形ごとの例(D1〜D6)と `-v`、辺の一覧に残る形、実バイナリの確認(合計 10 本ほど)と、書き換える既存の表明 3 つ

設計の出典: `construction/functional-design/functional-spec.md`(§4 の図、§5a のコードの変更点、§8 のテスト)と `rules.md`(BR1.1〜BR4.2)。要件の分析と、ユニットの分割は、省略した計画のため、ない。追跡の ID は、`intent-statement.md` の成功の指標 `SM1`〜`SM4`(functional-spec.md の §7a)。

テストは Minimal の戦略(必要なものだけ)で、test-after(各層を実装してから、その層のテスト)、行カバレッジ 80% の下限(Build and Test が全体で確かめる)。新しい依存は足さない。`unsafe` と `cfg` を足さない。

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

- [x] Step 1: 既存のテストの実行環境を確かめる(`cargo test -p conductor-core --test graph_drawable` が動くこと)。コマンドは `unit-test-instructions.md` に記録済み。
- [x] Step 2: 配置の判定(core)を実装する: `crates/core/src/graph/layout.rs` の `skip_targets` を、(a) 「両方の飛び越し」のジョブでは、次のジョブへの辺の条件を要らなくする、(b) 行き先が同じ(`t == top`)を許し、積み上げに載せない、(c) 交差の判定を `t > top` にする、に変える。(BR1.1、BR1.5、BR2.1。SM1)
- [x] Step 3: レーンの行の作り方(core)を実装する: 同じ `layout.rs` の `lane_layout` に、JoinLane(`|\|` の行、ラベルの位置は 2 × (1 + 開いている線の数)、`after_fork` を偽に)と NoLane(「両方の飛び越し」は線を作らない)を足す。(BR1.2〜BR1.4、BR2.1。SM1)
- [x] Step 4: 配置の層のテストを書いて実行する(core): `crates/core/tests/graph_layout_oracle.rs` を一般化し、新しい正解データ `crates/core/tests/fixtures/graph_layout_oracle_same_target.md`(`layout-oracle-same-target.md` の写し。1,517 例)を、1 文字も違わず照合する。既存の正解データ(537 例)の照合も残す。(BR4.1。SM4)
- [x] Step 5: 配置の層の既存のテストを、新しい期待に書き換える(core): `crates/core/tests/graph_drawable.rs` の 3 つの表明(同じ行き先 `a→d、b→d`、終点を共有する形、成功と警告の両方が飛び越す形)を「図にできる」に。交差と、異なる行き先が 2 つの形は、そのまま「図にできない」。テストの名前を直す。(BR1.5、BR2.1、BR3.2。SM2)
- [x] Step 6: 「両方の飛び越し」と、同じ行き先の追加の例のテストを書いて実行する(core): D1〜D6(functional-spec.md の §4.2、§4.4)と、交差、異なる行き先が 2 つ、後ろ向きの辺などが辺の一覧に残ること。`crates/core/tests/graph_lane_polish.rs`(新規)。(BR1.1〜BR2.1、BR3.1。SM1、SM2)
- [x] Step 7: 整形(cli)を実装する: `crates/cli/src/report/messages.rs` の `GRAPH_EDGE_LIST_NOTE` の英語を、「note: the skip edges cross or form a shape the lane diagram cannot draw, so it is replaced by a list of edges.」に改める。`graph_text.rs` は変えない(合流の行が `Fork` を含むことに頼る。functional-spec.md の §5a の 5)。(BR3.1。SM1)
- [x] Step 8: 整形の層のテストを書いて実行する(cli): `crates/cli/tests/graph_report.rs` に、同じ行き先(S1)と両方の飛び越し(D1)の text の全文、`-v` の例(S1 と D1)、`--state` の重ね(D1)、英語の注記の全文を足す。既存の英語の注記の期待があれば、新しい文に合わせる。(BR2.2、BR3.1。SM1)
- [x] Step 9: 実バイナリのテストを書いて実行する(cli): `crates/cli/tests/binary_graph.rs` に、「同じ行き先」と「両方の飛び越し」の定義で `conductor graph` を動かして、図を確かめるテストを足す(英語の注記も確かめる)。(SM1)
- [x] Step 10: README を直す: `README.md` の「辺の一覧に切り替わる形」(569 行目)を、交差する形、異なる行き先への飛び越しを 2 つ持つ形、のみにし、「同じ行き先」と「両方の飛び越し」の例を足す。README の検査(`crates/cli/tests/docs_graph.rs`)を、新しい記述に合わせる。(BR4.2。SM3)
- [x] Step 11: 仕様書を直す: `docs/spec/rust/05-execution-flow.md` と `docs/spec/language-agnostic/05-execution-flow.md`(レーンの配置と「辺の一覧に切り替わる形」の節。「share a target」を含む記述)、および、配置に触れる他の節(02、08 の図の例、11 の受け入れ基準)を、実装に合わせる。ほかの 5 つの仕様書(go、typescript、python、csharp、powershell)は変えない。(BR4.2。SM3)
- [x] Step 12: 全体の確認: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --locked --workspace`、`cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only` が通ること(最後の 1 つは、Build and Test が確かめる)。
- [x] Step 13: 文書と追跡: `code-summary.md`、`traceability.json`(`SM1`〜`SM4` の各行に、実装またはテストのファイルを `target` に書く)を作る。

## 守ること(守る規則)

- 新しい外部ライブラリを足さない(`Cargo.toml`、`Cargo.lock` に差分を出さない)。
- `unsafe` を足さない。`cfg(unix)`/`cfg(windows)` を足さない(`nfr_source.rs`)。
- 環境変数の値を出力に出さない(この変更は、環境変数に触れない)。
- 既存のファイルは、この計画の Step に書いたものだけを変える(`demo/` は変えない)。
- コミットは小さな単位(配置の判定、レーンの行、正解データの照合、書き換え、追加の例、整形、README、仕様書 の順)。メッセージは日本語。

## Assumptions

なし(データのモデルの要素は足さない。`SkipPlan` は core の内部の途中結果で、公開しない)。
