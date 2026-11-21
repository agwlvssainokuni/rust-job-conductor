# コード生成の計画(Code Generation Plan): U2 u2-graph(`conductor graph`)

## Summary

- Builds: `conductor graph`(text の図、`--format mermaid`、`-v`、`--state`)。辺の取り出し、「対応する形」のレーンの配置(git と同じ見た目。正解データ 537 例と一致)、辺の一覧(対応しない形)、状態の重ね合わせ、Mermaid の整形
- Touches: `crates/core`(`flow` の遷移の解決の切り出し、新しいモジュール `graph/`)、`crates/cli`(`args.rs`、`types.rs`、`report/`、新しい `graph.rs`、`aux_commands.rs` の `validate` の共通化、`lib.rs`)、`README.md`、`demo/`
- Tests: 約 50 個(単体、正解データ 537 例の一致、偽の `Runtime` を使う流れのテスト、実バイナリのテスト)。メッセージの網羅の既存テストを合わせて直す

## 前提と範囲

- 設計の原本: `construction/u2-graph/functional-design/` の `entities.md`、`rules.md`(BR の番号)、`functional-spec.md`(WF1〜WF6、期待する出力の例 G1〜G4、G6、E1、E2、`--state`、`-v`、Mermaid)、`layout-oracle.md`(レイアウトの正解データ。537 例)。契約: `inception/contract-design/contract-summary.md` の C2、C5、C6(機能の設計の「契約への反映」で補正済み)。
- 新しい外部ライブラリは足さない(`Cargo.toml` と `Cargo.lock` は変えない)。`unsafe` は書かない。`cfg(unix)`/`cfg(windows)` は `platform` の外に置かない。環境変数の値を、出力にもステートにも含めない(`project.md`)。`git` のソースコード(GPL)を写さない(BR2.6)。手順と正解データから、自前で書く。
- 作らない: 図のファイル出力、色、画像、`--dry-run`(U3)、`init`(U4)。
- 範囲の外(別の依頼): 仕様書(`docs/spec/**`)の更新、版 0.9.0 への引き上げ、リリース。
- 変更を受ける、この作業の外の既存ファイル(名前を明記する): `README.md`(新しい節を足す。`docs-check` の必須語句は残す)、`demo/` の台本とデモのテスト。
- コミットは小さな単位で、直接 `main` に作る(`team.md`)。各ステップは、できるだけ 1 つのコミットにする。

## Assumptions

なし(設計の `entities.md` にない、データモデルの要素は足さない。レビューの助言のうち、使われない `LaneGlyph::cross` と `LaneRow::note_target` は作らない。「レーンの図か、辺の一覧か」を表す型 `GraphLayout` は、設計の BR2.4 の判定の結果の表現として、実装が決める)。

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
      "text": "- **Methodology**: test-after\n- **Methodology evidence**: 基準(前回のインタビュー Q3)でチームは test-after を選んだ。履歴も整合する。たとえば `[job.retry]` では、型・読み取りの実装(d554ece、445d478)の後に、テストを追加するコミット(8ec98b5)が続き、実行の流れ(19d7b1a)の後に README のテスト(de4e3ad)が続く。`code-quality-assessment.md` も「チームの方針(test-after、行カバレッジ 80%)と一致する」と記す。\n- **Ordering**: 各テスト可能な層を実装してから、その層のテストを書いて実行する。\n- テストの種類: 各クレートの `tests/` に統合テスト(core 10、runtime 13、cli 21、converter 5 のファイル)。流れのテストは偽の `Runtime`/`Sink`/`Env`/`Signals` を注入する。実バイナリのテストは `conductor-cli-test-helper` を使う。非機能のテスト(`nfr_*.rs`)、網羅のテスト(`ALL` 配列)、デモのテスト(`unix_demo.rs`、`windows_demo.rs`)を含む。テスト名は日本語。\n- 行カバレッジ 80% を下限とする。CI の `coverage` ジョブが `cargo llvm-cov --fail-under-lines 80` で強制する(テスト専用の補助バイナリは除外)。\n- ローカルでは、主な開発環境の OS でテストを実行する。それ以外の OS のテストは GitHub Actions の CI(`test`: ubuntu、macos、windows)で実行する。マージ前に CI を通す。\n- 最小の Rust(1.100.0)でもテストを実行する(`msrv-test`)。\n- 起動の性能(`nfr_perf.rs`)は `startup-bench` で測るが、警告のみで失敗にしない(`STARTUP_BENCH_ENFORCE=false`、`continue-on-error`)。\n- 今回の機能にも、test-after と 80% の下限、単体テストと実バイナリを動かす統合テストを続ける(インタビュー Q3)。\n- 主な開発環境の OS は macOS で、テストはローカルの macOS で実行する。他の OS は CI で確認する(インタビュー Q6)。"
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
  "input_sha256": "sha256:826bac9fdf85fea4f3f7f79a7d3365e6615a41720c04ce8e867b1220ea1ec197",
  "contract_sha256": "sha256:0ca5a7b0baee94a8a8284941eb581edfac09ab7108c711149824fd0b57d005b4"
}
```

適用する層: 「データモデル」は図の型(`core::graph`)、「リポジトリ/データアクセス」は `--state` のステートの読み取り(既存の `read_state_raw` を使う。新しい読み取りは作らない)なので、独立の層としては該当しない(流れの層に含める)、「ビジネスロジック」は辺の取り出し、「対応する形」の判定、レーンの配置、状態の重ね合わせ(`core`)、「API/エンドポイント」はコマンドの引数、流れ、text と Mermaid と辺の一覧の整形(`cli`)。「フロントエンド」は該当しない。

## 実装のステップ

### Step 1: 構造と設定の骨格(production configuration skeleton)

- [x] 1.1 新しいモジュールの空の枠を作る: `crates/core/src/graph/mod.rs`(`edges.rs`、`layout.rs`、`overlay.rs`)、`crates/cli/src/graph.rs`。`crates/core/src/lib.rs` と `crates/cli/src/lib.rs` に、`mod` と公開の `use` を足す。外部ライブラリは足さない(`Cargo.toml` は変えない)。`cargo build --locked --workspace` が通ることを確かめる。

### Step 2: テストの実行環境の確認(test runner readiness)

- [x] 2.1 既存のテストの実行環境(`cargo test --locked`)が動くことを確かめ、この Unit の、範囲を絞ったコマンドを `unit-test-instructions.md` に記録する(新しい実行環境や設定は作らない)。

### Step 3: データモデル — 実装(図の型。`core`)

- [x] 3.1 `crates/core/src/graph/mod.rs`: `EdgeKind`、`EdgeTarget`、`FlowEdge`(`from_job`、`to`、`kinds`、`skipped_jobs`、`explicit`)、`NodeState`、`StateOverlay`、`GraphNode`、`FlowGraph`、`LaneGlyph`(`empty`、`node`、`end_node`、`line`、`fork`、`join`。`cross` は作らない)、`LaneRow`(`glyphs`、`node_index`。`note_target` は作らない)、`LaneLayout`、`GraphLayout`(レーンの図 `Lanes(LaneLayout)` か、辺の一覧 `EdgeList` か)の型だけを定義する(`entities.md`、BR2.4)。値を持つ型はない(環境変数は持たない。BR3.3)。

### Step 4: データモデル — テストを書いて実行する

- [x] 4.1 `crates/core/tests/graph_types.rs`(新規、約 4 個): `FlowEdge` の kinds が空でない、`StateOverlay` と `FlowGraph` の組み立て、`NodeState` の対応(成功と警告が `ok`、`skipped`、`error`、`pending`。BR4.1)。(BR4.1)

### Step 5: ビジネスロジック(辺と図の構造)— 実装

- [x] 5.1 `crates/core/src/flow/next_step.rs`: 遷移の解決(ジョブ、結果の種類、定義から、行き先と飛び越すジョブを決める部分)を、公開の関数に切り出す。`classify_and_decide` の結果は変えない(既存のテストが、そのまま通る)。(BR1.5、FR2.4)
- [x] 5.2 `crates/core/src/graph/edges.rs`: 5.1 の関数を使って、辺を取り出す(成功と警告の行き先、同じなら 1 つにまとめる、`skipped_jobs`、`explicit`、辺の並び(BR1.2)、エラーに辺を作らない)。(BR1.1〜BR1.4)
- [x] 5.3 `crates/core/src/graph/overlay.rs`: 図の構造(節は記述順で、最後に終点)と、`StateOverlay` の重ね合わせ(`JobStatus` の対応。完了したステートでは再開位置を使わない)。(BR2.1、BR4.1、BR4.2)
- [x] 5.4 `crates/core/src/graph/layout.rs`(判定の部分): 「対応する形」かの判定 `is_lane_drawable`(BR2.4 の 3 つの条件: 各ジョブの飛び越しの辺が高々 1 つで、もう 1 つが次のジョブ向き、飛び越しの行き先がどの 2 つの辺でも違う、区間が交差しない(厳密な不等号)。ジョブ 1 個の定義も対応する形)。(BR2.4)

### Step 6: ビジネスロジック(辺と図の構造)— テストを書いて実行する

- [x] 6.1 `crates/core/tests/graph_edges.rs`(新規、8 個): 遷移なし、成功だけの遷移、警告だけの遷移、両方が同じ行き先(1 つの辺。kinds が両方)、両方が違う行き先(辺の並び)、`@end` へ飛ぶ、最後のジョブ(`@end` への辺)、ジョブ 1 個。実行との一致: 同じ定義のすべてのジョブ、成功と警告のすべての結果で、`classify_and_decide` の次のジョブ(と、飛び越すジョブ)が、辺の行き先(と `skipped_jobs`)と一致する。(BR1.1〜BR1.5)
- [x] 6.2 `crates/core/tests/graph_overlay.rs`(新規、5 個): 状態の重ね合わせ(成功、警告、スキップ、エラー、未実行)、再開位置(完了していないステートだけ)、完了したステートでは再開位置なし、ステートなし(すべて未実行)。(BR4.1〜BR4.3)
- [x] 6.3 `crates/core/tests/graph_drawable.rs`(新規、6 個): 対応する形(飛び越しなし、1 つ、入れ子、並び、`@end` への飛び越し、ジョブ 1 個)と、対応しない形(交差、行き先の共有、成功と警告の両方が飛び越す)の判定。(BR2.4)

### Step 7: ビジネスロジック(レーンの配置)— 実装

- [x] 7.1 `crates/core/src/graph/layout.rs`: 対応する形のレーンの配置(`LaneLayout`)を、`functional-spec.md` の WF3 の手順で実装する(主線に記述順、分岐の行、列 1 への挿入と右へのずれ、合流の行、残りの線の左への詰まり、終点)。`git` のソースは写さない。`layout-oracle.md` のすべての例に一致するまで、直す。判定が偽なら、`GraphLayout::EdgeList` を返す。(BR2.1〜BR2.7)

### Step 8: ビジネスロジック(レーンの配置)— テストを書いて実行する

- [x] 8.1 `layout-oracle.md` を、`crates/core/tests/fixtures/graph_layout_oracle.md` にコピーし(内容は変えない)、`crates/core/tests/graph_layout_oracle.rs`(新規、約 4 個)が、それを読み込む。537 個の例のすべてで、(a) 見出しの形(`jobs=<n> skips=<...>`)から定義を作り、(b) 辺を取り出して `is_lane_drawable` が真であることを確かめ、(c) レーンの配置を文字にして(テスト側の最小の整形で。`cli` の整形と同じ規則)、期待する出力と 1 文字も違わないことを確かめる。例の数が 537 であることも確かめる。(BR2.3)
- [x] 8.2 `crates/core/tests/graph_perf.rs`(新規、2 個): ジョブ 1,000 個で、(a) 入れ子が最大の形(前半の各ジョブが、後半のジョブへ、入れ子に飛び越す)、(b) 交差する形(辺の一覧に切り替わる)で、辺の取り出しと配置が、1 秒以内に終わる。(BR2.7、NFR1)

### Step 9: API/エンドポイント(引数と型。`cli`)— 実装

- [x] 9.1 `crates/cli/src/types.rs` と `args.rs`: `graph` サブコマンド、`--format <text|mermaid>`、`--state`、`-v`(`graph` で使えるように)、`--run-id`(`graph` で、`--state` と一緒のときだけ)。重複、値の誤り、許されないオプション(68)。`GraphOptions`(`entities.md`)を `CliCommand` に足す。(BR4.5、BR6.4)

### Step 10: API/エンドポイント(表示。`cli`)— 実装

- [x] 10.1 `crates/cli/src/report/graph_text.rs`(新規。`report/` の下): text の整形(ジョブ `*`、終点 `o`、線、分岐、合流。ラベルの位置、行末の空白の削除、状態の記号と再開位置の注記、`-v` の詳細の行(コマンド、リトライの要約、飛び越しの注記)、辺の一覧(BR2.8))。コマンドの表示(配列の整形、制御文字のエスケープ)。環境変数は出さない。(BR2.2、BR2.5、BR2.8、BR3.1〜BR3.3、BR4.1、BR4.2)
- [x] 10.2 `crates/cli/src/report/graph_mermaid.rs`(新規): Mermaid の整形(`flowchart TD`、節の ID は連番、辺のラベル、エスケープ、状態の記号、`classDef`/`class`)。(BR5.1〜BR5.5)
- [x] 10.3 `crates/cli/src/report/messages.rs`: 新しいメッセージ(日本語と英語。辺の一覧の注記と見出し、種類のラベル、再開位置、`-v` の見出し、使い方)を足し、`ALL` の網羅の配列にも足す。(BR7.1)

### Step 11: API/エンドポイント(流れ。`cli`)— 実装

- [x] 11.1 `crates/cli/src/aux_commands.rs`: `run_validate` の、`DefinitionValid` を表示する前までを、共通の関数に切り出す(定義の検査、`command` の検査、`.env` の読み取り、警告の表示。戻り値は、定義とパス、または終了コード)。`validate` は、その関数を呼び、成功したら `DefinitionValid` を表示する。`validate` の既存のテスト(U1 のものを含む)が、そのまま通る。(BR6.1)
- [x] 11.2 `crates/cli/src/graph.rs`: `graph` の流れ(WF6): 引数の検査、11.1 の共通の関数(パイプライン名の照合を含む)、`--state` のとき run ID の決定(`prepare` と同じ)と、`read_state_raw`、`check_consistency`(65)、ステートが無いときは全部未実行、辺の取り出し、図の構造、重ね合わせ、配置(または辺の一覧)、整形、標準出力に 1 回で書く(失敗は 65)。ステートの書き込みと削除、シグナルの準備はしない。`crates/cli/src/lib.rs` で、コマンドを振り分ける。(BR4.3〜BR4.6、BR6.1〜BR6.3)

### Step 12: API/エンドポイント — テストを書いて実行する

- [x] 12.1 既存のテストの、メッセージの網羅(`report_messages.rs`、`escape_table.rs`)と、引数の文法(`args_grammar.rs`)を、新しいコマンドとメッセージに合わせて直す。
- [x] 12.2 `crates/cli/tests/graph_report.rs`(新規、8 個): text の例 G1〜G4、G6、辺の一覧 E1、E2、`--state`(再開位置)、完了したステート、`-v`(G2 と G4 の 2 つの例)、Mermaid(G2、`--state` と再開位置、`<`、`>`、`&`、`#`、`"`、バッククォートを含むジョブ ID)、日本語と英語。(BR2.8、BR3.x、BR4.x、BR5.x)
- [x] 12.3 `crates/cli/tests/flow_graph.rs`(新規、8 個、偽の `Runtime`): `graph` が、ジョブを起動せず、ステートに書かず、標準出力に図だけを書く。`validate` が 64 になる定義(定義の誤り、`command` の誤り、`.env` の誤り)で、`graph` も 64 で図を出さない。`graph <name>` の名前の不一致で 64。ステートの食い違いで 65(完了したステートにも)。`--run-id` だけで 68。`--format` の誤りと重複で 68。`graph` 以外での `--format` で 68。`--state` でステートが無いとき全部 `[  ]`。完了したステートで、状態の記号があって、再開位置が無い。出力の失敗で 65。(BR4.x、BR6.x)
- [x] 12.4 `crates/cli/tests/binary_graph.rs`(新規、4 個、実バイナリ): `conductor graph` の text と `--format mermaid` の出力が、例と一致する。`--state` で、実際に作ったステートの重ね合わせ。`.env` と `env` を持つ定義で、環境変数の値が、出力のどこにも無い(秘密の値を、標準出力、標準エラーで検査。FR1.12)。`validate` と同じ検査(`.env` の誤りで 64)。(FR2.x、FR1.12)

### Step 13: 環境/ビルドの設定

- [x] 13.1 `Cargo.toml`、`Cargo.lock`、CI の設定は、変えない(新しい依存なし)。`crates/cli/tests/nfr_source.rs` が、新しいファイルについても、`unsafe`、`cfg`、新しい依存、ネットワークの検査を通ることを確かめる(必要なら対象の一覧を直す。しきい値は下げない)。

### Step 14: 文書と追跡

- [x] 14.1 `README.md`: `graph` の節を足す(使い方、`--format`、`-v`、`--state`、出力例、辺の一覧に切り替わる形、終了コード、バージョン v0.9.0 以降)。`docs-check` の必須語句は残す。`crates/cli/tests/docs_graph.rs`(新規、2 個): README に新しい語句(`conductor graph`、`--format mermaid`)がある。
- [x] 14.2 `demo/`: `graph` の場面を足す(`demo/` の台本 `run-demo.sh` と `windows/run-demo.cmd` に、飛び越しのある定義の `graph` を足す)。デモのテスト(`unix_demo.rs`、`windows_demo.rs`)を合わせる。(FR5.2)
- [ ] 14.3 `code-summary.md`、`traceability.json`(`aidlc engine sensor-traceability`)、`source-manifest.json` を作る。

## 要件・規則の対応(追跡)

| 要件 | 規則 | 実装のステップ | テスト |
|------|------|-----------------|--------|
| FR2.1 | BR4.6、BR6.2〜BR6.4 | 9、11 | 12.3、12.4 |
| FR2.2 | BR1.1〜BR1.4、BR2.1〜BR2.8 | 3、5、7、10 | 6.1、6.3、8.1、8.2、12.2 |
| FR2.3 | BR3.1〜BR3.3 | 10 | 12.2 |
| FR2.4 | BR1.3、BR1.5 | 5 | 6.1 |
| FR2.5 | BR5.1〜BR5.5 | 10 | 12.2、12.4 |
| FR2.6 | BR6.1 | 11 | 12.3、12.4 |
| FR2.7 | BR4.1〜BR4.5 | 5、11 | 6.2、12.2、12.3 |
| FR1.12 | BR3.3、BR6.2、BR7.1 | 10、11 | 12.4 |
| FR5.1 | BR7.1 | 10 | 12.1 |
| FR5.2 | BR7.2 | 14 | 14.1、14.2 |

## 順序の注意

- 遷移の解決の切り出し(5.1)は、`classify_and_decide` の結果を変えない小さなコミットとして、先に単独で行う。
- 各層は、「実装 → その層のテスト」の順(test-after)。
- 範囲を絞った実行のコマンドは、`unit-test-instructions.md` に書く。プロジェクト全体の行カバレッジ 80% は、Build and Test が、全体のテストで確かめる。
