# コード生成の計画: u4-graph-dot(`graph --format dot`)

## Summary

- Builds: `graph --format dot` で Graphviz の DOT を出す(cli の引数・描画・使い方の文だけ。core は変えない)
- Touches: `crates/cli/src/types.rs`、`args.rs`、`aux_commands.rs`、`report/mod.rs`、`report/graph_dot.rs`(新規)、`report/messages.rs`、cli のテスト(`src/tests/graph_dot.rs` 新規、`tests/flow_graph.rs`、`tests/binary_graph.rs`、`src/tests/docs.rs`、デモのテスト)、`README.md`、`demo/run-demo.sh`、`demo/windows/run-demo.cmd`、`demo/README.md`
- Tests: 描画の単体テスト約 14(escape の対の表、記号、太い枠、ラベル、`-v`、mermaid との数の一致、性能)、流れのテスト 2、実バイナリのテスト 4、文書とデモ

出典: 機能の設計(`functional-spec.md`、`rules.md` の BR1.1〜BR4.2、`entities.md`)、非機能の設計(`security-design.md`、`performance-design.md`、`reliability-design.md`、`observability-design.md`、`logical-components.md`)、ユニットの定義(`unit-of-work.md` の U4)、要件(`requirements.md` の FR4.1〜FR4.4、FR8.1)、契約(`contract-summary.md` の C5)、設計の決定(ADR-006)。ユーザーストーリーは作っていないため、各ステップは要件と規則の ID に対応させる。

テストの進め方は Testing Contract(test-after)に従う。触る層は cli の表示と引数だけで、描画を作ってからその層のテストを書いて走らせる。

## 設計のレビューからの持ち越し(このユニットで扱う)

- 使い方の文の直す場所(機能の設計のレビュー): `args.rs` のモジュールの文書、`report/messages.rs` の使い方の文(英日の 2 つずつ、計 4 か所の `--format <text|mermaid>`)、README の 227 行・249 行・617 行あたりの使い方と表、README の `graph` の例と `-v` の説明、`src/tests/docs.rs` の検査の語句。
- `docs/spec/rust/` の仕様の文書は過去の版の記録として直さない(`reliability-design.md` の 1)。
- DOT の文法の確かめ方(非機能の設計のレビュー): 「二重引用符の数が偶数」ではなく、各行が決まった形(`  n<数> [label="<escape 済み>", ...];`、`  n<数> -> n<数>;`、`  n<数> -> n<数> [label="..."];` など)に一致し、ラベルの中の `"` がすべて `\` で打ち消されていることを確かめる。
- 行の数は「節の数(終点 `nend` を含む)+ 辺の数 + 3」(`digraph` の行、`rankdir` の行、`}` の行)。
- 性能のテストは絶対の上限だけにし、mermaid との比は 3 つの OS の揺れを考えて使わない。

## ステップ

### Step 1: テストの準備を確かめる

- [x] 変更の前に、`unit-test-instructions.md` の 2 つのコマンドを走らせ、結果を記録する。

### Step 2: 引数(cli)

- [x] `crates/cli/src/types.rs` の `GraphFormat` に `Dot` を足す(ドキュメントコメントは日本語)。
- [x] `crates/cli/src/args.rs` の `parse_graph_format` で `dot` を受け付け、`text`・`mermaid`・`dot` 以外は今と同じ使い方の誤り(文に `dot` を足す)。モジュールの文書の `--format <text|mermaid>` も直す(BR1.1)。

### Step 3: DOT の描画(cli)

- [x] `crates/cli/src/report/graph_dot.rs` を新しく作り、`report/mod.rs` で公開する。`render_dot(lang, &FlowGraph) -> String` と `escape_dot_label(&str) -> String` を書く(BR2.1〜BR3.2)。
  - `digraph conductor {`、`  rankdir=TB;`、節(ジョブは `n<i>`・`shape=box`、再開位置は `penwidth=3`、終点は `nend`・`shape=oval`)、辺(主線はラベルなし、他は `kind_label` の語)、`}`。
  - ラベルは `state_symbol` + ジョブ ID を `escape_dot_label` に通す。`escape_dot_label` は `escape_full` のあと `\` → `\\`、`"` → `\"`。
  - ジョブ ID → 節の ID の表は `HashMap` で引き、1 回の走査で書く(`performance-design.md`)。
- [x] `crates/cli/src/aux_commands.rs` の描画の分岐に `GraphFormat::Dot => render_dot(lang, &graph)` を足す。`-v` は mermaid と同じく影響しない(BR2.5)。
- [x] `crates/cli/src/report/messages.rs` の使い方の文 4 か所を `--format <text|mermaid|dot>` にし、`--format` の説明の行に `dot` を足す(BR1.2)。

### Step 4: cli の層のテストを書いて走らせる

- [x] `crates/cli/src/tests/graph_dot.rs` を新しく作る(`src/tests/mod.rs` に足す。テスト名は日本語):
  - `escape_dot_label` の入力と出力の対の表(`security-design.md` の 1 の表の 8 行)
  - `"`・`\`・改行を含むジョブ ID の図で、各行が決まった形に一致し、ラベルの中の `"` がすべて打ち消されていること、行の数が「節(終点を含む)+ 辺 + 3」であること
  - `--state` のときの記号が mermaid と同じ、再開位置だけに `penwidth=3`、`--state` でなければ記号も太い枠も無い(BR2.4、NFR1.5)
  - 主線はラベルなし、成功だけ・警告だけの辺は語のラベル(英日)(BR2.3)
  - 分岐・飛び越し・終点を含む定義で、ジョブの節の数と辺の数が mermaid と同じ(NFR1.2)
  - 1 万のジョブの直線の図の描画が時間の上限の内に終わり、行の数が期待どおり(NFR1.3、NFR1.4)。上限は実測に余裕を持たせて決め、下げない
- [x] `crates/cli/tests/flow_graph.rs` に、偽の `Runtime` で `--format dot` が 0 で 1 回書くこと、誤りのある定義では `--format text` と同じ標準エラーと終了コードになることを足す(NFR6.3)。
- [x] `crates/cli/tests/binary_graph.rs` に、実バイナリで `--format dot`、`--format dot --state`、`--format dot -v`(`-v` なしと同じ)、`--format svg` の誤り、目印をコマンドとパイプライン名に入れた定義で出力に目印が出ないことを足す(NFR2.2、NFR6.2)。
- [x] 使い方の文を見る今のテスト(`src/tests/docs.rs`、使い方の誤りの文を見るテスト)だけを直す。
- [x] cli のコマンドを走らせ、通ることを確かめる。

### Step 5: デモを更新する

- [x] `demo/run-demo.sh` と `demo/windows/run-demo.cmd` の場面 12 に `graph --format dot` を足し、`demo/README.md` の説明を合わせる(BR4.2)。
- [x] `crates/cli/tests/unix_demo.rs` と `windows_demo.rs` に、`digraph conductor {` の行が出ることの確認を足す。`cargo test -p conductor-cli --test unix_demo --locked` を走らせる。`windows_demo.rs` は CI の windows で確かめる。

### Step 6: README を更新する

- [x] `README.md` の使い方(2 か所)と表の `--format` を `<text|mermaid|dot>` にし、`graph` の節に `--format dot` の説明と `conductor graph --format dot | dot -Tsvg > flow.svg` の例、`-v` が DOT に影響しないことを足す(BR1.2、BR4.2、FR8.1)。
- [x] `cargo test -p conductor-cli --lib --locked -- tests::docs` が通ることを確かめる。

### Step 7: 書式と静的検査

- [x] `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通ることを確かめる(NFR7.1)。

## 規則と要件の対応

| ステップ | 規則 | 要件 |
|---|---|---|
| Step 2 | BR1.1 | FR4.1、FR4.4 |
| Step 3 | BR1.2、BR2.1〜BR3.2 | FR4.1〜FR4.3 |
| Step 4 | BR2.1〜BR4.1 | FR4.1〜FR4.4、NFR1、NFR2、NFR6 |
| Step 5、Step 6 | BR1.2、BR4.2 | FR8.1 |
| Step 7 | | NFR7 |

## コミットの分け方

小さな単位でコミットする(件名は日本語、先頭に `cli:`・`docs:`)。各コミットで fmt と clippy が通るようにする。例: Step 2+3 の引数と描画(1〜2 コミット。使われない関数で clippy が落ちないよう、分岐と描画は同じコミットに)、Step 4 のテスト、Step 5・Step 6 は `docs:`。版は上げない。

## Testing Contract

```json
{
  "version": 1,
  "methodology": "test-after",
  "source": "team",
  "ordering": "各テスト可能な層を実装してから、その層のテストを書いて実行する。",
  "scope": "feature",
  "test_strategy": "standard",
  "project_type": "brownfield",
  "applicable_notes": [
    {
      "layer": "org",
      "text": "We treat tests as a first-class deliverable in every Bolt. The specific\nmethodology (TDD, BDD, ATDD, or classic test-after) is affirmed at\npractices-discovery and recorded in `team.md` under this heading with explicit\n`Methodology` and `Ordering` fields; Code Generation resolves those fields\nindependently from coverage, tooling, and scope notes.\n\nWhen no posture has been affirmed, our default per scope is:\n- **Methodology**: test-after\n- **Ordering**: implement each applicable testable layer, then write and run\n  that layer's tests.\n- `mvp`, `enterprise`, `feature`, `infra` add an 80% line-coverage\n  floor and CI execution before merge.\n- `bugfix`, `security-patch` add a targeted regression for the specific\n  bug/vulnerability and require the existing suite to remain green.\n- `express` uses the Minimal strategy: requirement-driven unit tests (one per\n  requirement, with a happy-path floor per component); existing tests remain\n  green.\n- `poc`, `refactor`, `workshop`, `classic` add no extra new-test floor and require the\n  existing suite to remain green.\n\nThe active `Test Strategy` still applies in every scope and determines test\nvolume/types. Scope floors are additive; they never reduce or replace the\nselected strategy.\n\nBuild and Test verifies defined coverage floors and affirmed quality targets;\nthey may not be weakened to make a step pass.\n\nAffirm a stricter posture in `team.md` if the team commits to one."
    },
    {
      "layer": "team",
      "text": "- **Methodology**: test-after\n- **Methodology evidence**: チームは test-after を続けると決めた(インタビュー Q3)。履歴も整合する。v0.9.0 の U4 では、`create_file` の追加(602d15c)の後にそのテスト(90a27fa)、ひな形の生成(0fbd466)の後にひな形のテスト(5b750f8)が続く。U3 も流れ(a9eb237)の後に流れのテスト(365f45c)と実バイナリのテスト(062c627)が続く。`code-quality-assessment.md` も「チームの方針(test-after、行カバレッジ 80%)と一致する」と記す。\n- **Ordering**: 各テスト可能な層を実装してから、その層のテストを書いて実行する。\n- テストの種類: 各クレートの `tests/` に統合テスト(core 33、runtime 14、cli 25、converter 2 のファイル)。単体テストは各モジュールの `mod tests` と、`crates/cli/src/tests/`(11 ファイル)、`crates/converter/src/tests/`(4 ファイル)。`#[test]` は合計 1,170。\n- 流れのテストは偽の `Runtime`/`Sink`/`Env`/`Signals` を注入する。実バイナリのテストは `conductor` とテスト専用の補助バイナリを動かす。非機能のテスト(`nfr_*.rs`、`*_perf.rs`)、網羅のテスト(`ALL` 配列)、参照出力との一致(`graph_layout_oracle.rs`)、文書との一致(`src/tests/docs.rs`、`msrv_consistency.rs`)、デモのテスト(`unix_demo.rs`、`windows_demo.rs`)を含む。テスト名は日本語。\n- テストの都合で本番のコードを曲げない。テスト専用の引数・`pub`・関数を本番に残さず、流れのテストは公開の入口と差し替え口で書く(インタビュー Q6)。\n- 行カバレッジ 80% を下限とする。CI の `coverage` ジョブが `cargo llvm-cov --fail-under-lines 80` で強制する(テスト専用の補助バイナリは除外)。\n- 主な開発環境の OS は macOS で、テストはローカルの macOS で実行する。他の OS のテストは GitHub Actions の CI(`test`: ubuntu、macos、windows)で確認する。マージ前に CI を通す(インタビュー Q3)。\n- 最小の Rust(1.100.0)でもテストを実行する(`msrv-test`)。\n- 起動の性能(`nfr_perf.rs`)は `startup-bench` で測るが、警告のみで失敗にしない(`STARTUP_BENCH_ENFORCE=false`、`continue-on-error`)。\n- v0.10.0 の機能にも、test-after と 80% の下限、単体テストと実バイナリを動かす統合テストを続ける(インタビュー Q3)。\n- ステートの中身を調べるテスト(`nfr_content.rs`)は、B6 のために開始と終了の時刻だけを許すように直す。それ以外(コマンド・環境変数・出力)は今までどおり検査する(インタビュー Q7)。"
    }
  ],
  "obligations": {
    "strategy": "standard",
    "strategy_volume": [
      "Five to eight tests per component.",
      "Unit tests plus integration tests for key boundaries.",
      "Add E2E, performance, or security tests when requirements demand them."
    ],
    "scope_floor": [
      "Meet an 80% line-coverage floor.",
      "Run the selected tests in CI before merge."
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
  "input_sha256": "sha256:ed0086a4c967568436c54943aebacd308a988af279eee16ed426cf4f39a154fa",
  "contract_sha256": "sha256:3789717991ddc007f6cbca5d56d6b2d0b248de4ce1061637170c6787b5bf88e9"
}
```
