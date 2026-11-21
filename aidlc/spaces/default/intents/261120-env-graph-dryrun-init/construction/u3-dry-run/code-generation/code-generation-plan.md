# コード生成の計画(Code Generation Plan): U3 u3-dry-run(`--dry-run` と `--assume`)

## Summary

- Builds: `pipeline` と `run` の `--dry-run`(実行せず、ステートに書かず、シグナルも準備せずに、実行計画を出す)と `--assume <ジョブID>=<終了コード>`(終了コードを仮定した流れの模擬)。計画の生成(`core`。実行と同じ判断)、仮定の検査、計画の整形(日本語と英語)
- Touches: `crates/core`(新しいモジュール `plan/`)、`crates/cli`(`args.rs`、`types.rs`、`ctx.rs` の準備の切り出し、新しい `dry_run.rs`、`pipeline.rs` と `run.rs` の分岐、`report/`、メッセージ)、`README.md`、`demo/`
- Tests: 約 45 個(単体、実行との一致、偽の `Runtime` を使う流れのテスト、実バイナリのテスト)。既存の引数の文法とメッセージの網羅のテストを合わせて直す

## 前提と範囲

- 設計の原本: `construction/u3-dry-run/functional-design/` の `entities.md`、`rules.md`(BR の番号)、`functional-spec.md`(WF1〜WF5、期待する出力の例 P1〜P8 と英語の例 P2e、P4e、契約への反映)。契約: `inception/contract-design/contract-summary.md` の C1、C2、C5、C7(機能の設計の「契約への反映」で補正済み)。U1 の名前の要約(`merge_env` の `EffectiveEnv::names()`)と、U2 のコマンドの表示とリトライの要約の整形を使う。
- 新しい外部ライブラリは足さない(`Cargo.toml` と `Cargo.lock` は変えない)。`unsafe` は書かない。`cfg(unix)`/`cfg(windows)` は `platform` の外に置かない。環境変数の値を、出力にもステートにも含めない(`project.md`)。`--assume` の誤りの表示に、入力の値(ジョブ ID)を含めない(`with_subject` を使わない)。
- 作らない: `--dry-run` なしの動きの変更、計画のファイル出力、`graph`(U2)、`init`(U4)。
- 範囲の外(別の依頼): 仕様書(`docs/spec/**`)の更新、版 0.9.0 への引き上げ、リリース。
- 変更を受ける、この作業の外の既存ファイル(名前を明記する): `README.md`(新しい節を足す。`docs-check` の必須語句は残す)、`demo/` の台本とデモのテスト。
- コミットは小さな単位で、直接 `main` に作る(`team.md`)。各ステップは、できるだけ 1 つのコミットにする。
- ファイルの改行を保つ: `demo/windows/run-demo.cmd` などの CRLF のファイルは、CRLF のまま編集する(U2 のレビューの指摘)。

## Assumptions

なし(設計の `entities.md` にない、データモデルの要素は足さない。計画を作る関数が、拒否(ジョブ ID と理由)を返す型 `PlanRejection` は、設計の BR4.3 の表現として、実装が決める)。

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

適用する層: 「データモデル」は計画の型(`core::plan`)、「リポジトリ/データアクセス」は、ステートと `.env` の読み取り(既存の `read_state` と U1 の読み取りを使う。新しい読み取りは作らない)なので、独立の層としては該当しない、「ビジネスロジック」は模擬と計画の生成(`core`)、「API/エンドポイント」はコマンドの引数、流れ、計画の整形(`cli`)。「フロントエンド」は該当しない。

## 実装のステップ

### Step 1: 構造と設定の骨格(production configuration skeleton)

- [x] 1.1 新しいモジュールの空の枠を作る: `crates/core/src/plan/mod.rs`(`simulate.rs`)、`crates/cli/src/dry_run.rs`。`crates/core/src/lib.rs` と `crates/cli/src/lib.rs` に、`mod` と公開の `use` を足す。外部ライブラリは足さない(`Cargo.toml` は変えない)。`cargo build --locked --workspace` が通ることを確かめる。

### Step 2: テストの実行環境の確認(test runner readiness)

- [x] 2.1 既存のテストの実行環境(`cargo test --locked`)が動くことを確かめ、この Unit の、範囲を絞ったコマンドを `unit-test-instructions.md` に記録する(新しい実行環境や設定は作らない)。

### Step 3: データモデル — 実装(計画の型。`core`)

- [x] 3.1 `crates/core/src/plan/mod.rs`: `Assumption`、`PlanDisposition`、`SimulatedResult`、`PlanEntry`、`ResumeInfo`、`PlanEnd`、`ExecutionPlan`、`NextStepNote`(`end_reached` を含む)、`PlanRejection`(拒否のジョブ ID と理由)の型だけを定義する(`entities.md`、BR4.3)。値を持つ型はない(環境変数は、名前と出どころだけ。BR5.1)。

### Step 4: データモデル — テストを書いて実行する

- [x] 4.1 `crates/core/tests/plan_types.rs`(新規、約 3 個): 型の組み立て、`PlanEntry` が環境変数の値を持たない(名前と出どころだけ)、`Assumption` の範囲の補助(0〜255)。(BR5.1、BR2.1)

### Step 5: ビジネスロジック(模擬と計画の生成)— 実装

- [x] 5.1 `crates/core/src/plan/simulate.rs`: 計画の生成(WF3)。`pipeline`: 模擬の状態をステートから作り、定義の記述順に、`decide_before_run`、(`Run` のとき)仮定(なければ 0)の `JobExecution` と `classify_and_decide`(`RunMode::Pipeline`)、模擬の状態の更新(ジョブの状態、再開位置、`end_reached`、スキップの反映。BR1.8)、エラーでの停止(以降は not_reached)、`PlanEnd`、使われなかった仮定、各ジョブの分割後のコマンド・環境変数の名前(U1 の `merge_env` の名前の要約。試行 1)・リトライの設定。`run`: `decide_before_run`(個別実行)と、`classify_and_decide`(`RunMode::Individual`)の `NextStepNote`。`Reject` は `PlanRejection` として返す。ファイルにも、プロセスにも、環境変数にも触れない。(BR1.1〜BR1.8、BR2.3、BR2.4、BR4.3)

### Step 6: ビジネスロジック(模擬と計画の生成)— テストを書いて実行する

- [x] 6.1 `crates/core/tests/plan_pipeline.rs`(新規、8 個): すべて成功、仮定で警告、仮定でエラー(停止。以降は not_reached)、遷移でスキップ、`@end` への到達、実行済み(再開)、ステートなし、完了したステート(「ステートなし」として扱う)、再開位置、拒否(順序外のステート)。(BR1.2〜BR1.4、BR4.3)
- [x] 6.2 `crates/core/tests/plan_assume.rs`(新規、5 個): 使われなかった仮定、リトライの対象の注記(retryable)、`success_exit_codes` に 0 を含まないジョブの既定(エラー)、127 以上の仮定が通常の終了コードとして分類される、同じ仮定を複数のジョブに。(BR1.7、BR2.3、BR2.4)
- [x] 6.3 `crates/core/tests/plan_run.rs`(新規、5 個): `run` の計画(`Run`、`Skip(AlreadyExecuted)`、拒否)、次の動き(次のジョブ、遷移のスキップ、`@end` に到達、完了、停止)、対象でないジョブへの仮定。(BR1.6)
- [x] 6.4 `crates/core/tests/plan_perf.rs`(新規、1 個): ジョブ 1,000 個(すべて実行する。`env` と `env_file` を持つ定義を含む)の計画の生成が、1 秒以内に終わる。(BR6.3、NFR1)

### Step 7: API/エンドポイント(引数と型。`cli`)— 実装

- [x] 7.1 `crates/cli/src/types.rs` と `args.rs`: `--dry-run`(`pipeline` と `run` だけ)、`--assume <ジョブID>=<終了コード>`(複数回。最初の `=` で分ける。書式、範囲 0〜255、重複の検査。`--dry-run` と一緒のときだけ)。`DryRunOptions`(`entities.md`)を `CliCommand` に足す。誤りは使い方の誤り(68)で、入力の値を含めない(新しい誤りの種類は、`with_subject` を使わない)。(BR2.1、BR2.2、BR4.5)

### Step 8: API/エンドポイント(流れと表示。`cli`)— 実装

- [x] 8.1 `crates/cli/src/ctx.rs`: `prepare` の、シグナルの準備を除いた部分(定義の検査、ジョブの存在、run ID の決定、`.env` の読み取り、`read_state`、照合)を、共通の関数に切り出す(本番の `prepare` は、その関数を呼んだあとに、シグナルを準備する。順序と、本番の動きは変えない)。(BR1.1、BR4.1、BR4.2)
- [x] 8.2 `crates/cli/src/dry_run.rs`: `--dry-run` の流れ(WF2〜WF4): 8.1 の関数、`--assume` のジョブ ID が定義にあるかの検査(なければ 68)、各ジョブのコマンドを `runtime` の `check_command` で分ける、計画の生成(`core`)、拒否は本番と同じ表示(`DisplayEvent::Rejected`)と 66、計画の整形、標準出力に 1 回で書く(失敗は 65)。成功は 0。ジョブの起動、ステートの書き込みと削除、シグナルの準備はしない。`pipeline.rs` と `run.rs` の入口で、`--dry-run` なら、この流れへ分岐する。(BR1.1、BR3.4、BR4.x)
- [x] 8.3 `crates/cli/src/report/plan_text.rs`(新規。`report/` の下)と `report/messages.rs`: 計画の整形(ヘッダー、再開位置または対象のジョブ、仮定、使われなかった仮定、ジョブの見出しの行、詳細の行、模擬の結果または次の動き)。日本語と英語の文言は、`functional-spec.md` の例と表のとおり。コマンドの表示とリトライの要約は、U2 の整形(`graph` の `-v`)の関数を、共通で使う。ジョブ ID は制御文字をエスケープして表示する。環境変数は、名前だけ。(BR3.1〜BR3.3、BR5.1、BR6.1)

### Step 9: API/エンドポイント — テストを書いて実行する

- [x] 9.1 既存のテストの、引数の文法(`args_grammar.rs`)と、メッセージの網羅(`report_messages.rs`、`escape_table.rs`)を、新しいオプションとメッセージに合わせて直す。
- [x] 9.2 `crates/cli/tests/plan_report.rs`(新規、7 個): 例 P1〜P8、英語の例 P2e、P4e の出力が、1 文字も違わず一致する。コマンドの表示のエスケープ、ジョブ ID のエスケープ、環境変数の名前の順(層の順、最後に `CONDUCTOR_*`)。(BR3.1〜BR3.3)
- [x] 9.3 `crates/cli/tests/flow_dry_run.rs`(新規、9 個、偽の `Runtime`): `--dry-run` が、ジョブを起動せず、ステートの書き込みと削除を呼ばず、シグナルを準備しない。定義の誤り、`command` の誤り、`.env` の誤りで 64。ステートの食い違いで 65。拒否で 66(`pipeline` と `run`)。`--assume` の不正な入力(`=` なし、整数でない、範囲外、重複、存在しないジョブ)で 68(誤りの表示に、入力の値が無いことも確かめる)。`--assume` だけ(`--dry-run` なし)で 68。`--dry-run` を `validate` などに付けて 68。計画を出せたら 0。実行との一致: 偽の `Runtime` が、仮定どおりの終了コードを返し(リトライなし、中断なし)、同じ定義、同じステートで、`--dry-run` なしの `pipeline` が起動するジョブの並びと、計画の「実行する」のジョブの並びが、一致する。(BR1.x、BR2.x、BR4.x、BR1.8)
- [x] 9.4 `crates/cli/tests/binary_dry_run.rs`(新規、4 個、実バイナリ): `conductor pipeline demo --dry-run` と `--assume` の出力が、例と一致する。実際に作ったステートから、再開位置が出て、ステートのファイルのバイト列が変わらない。何も実行されない(ジョブが書くはずのファイルができない)。`.env` と `env` を持つ定義で、環境変数の値が、標準出力、標準エラーのどこにも無い。(FR3.x、FR1.12)

### Step 10: 環境/ビルドの設定

- [x] 10.1 `Cargo.toml`、`Cargo.lock`、CI の設定は、変えない(新しい依存なし)。`crates/cli/tests/nfr_source.rs` が、新しいファイルについても、`unsafe`、`cfg`、新しい依存、ネットワークの検査を通ることを確かめる(必要なら対象の一覧を直す。しきい値は下げない)。

### Step 11: 文書と追跡

- [x] 11.1 `README.md`: `--dry-run` と `--assume` の節を足す(使い方、出力例、使われなかった仮定、終了コード(0、64、65、66、68)、計画が示さないこと(中断、起動の失敗、リトライの回数ごとの結果)、バージョン v0.9.0 以降)。`docs-check` の必須語句は残す。`crates/cli/tests/docs_dry_run.rs`(新規、2 個): README に新しい語句(`--dry-run`、`--assume`)がある。
- [x] 11.2 `demo/`: `--dry-run` の場面を足す(`demo/run-demo.sh` と `demo/windows/run-demo.cmd` に、`pipeline --dry-run` と `--assume` の例を足す。`run-demo.cmd` は CRLF のまま編集する)。デモのテスト(`unix_demo.rs`、`windows_demo.rs`)を合わせる。(FR5.2)
- [x] 11.3 `code-summary.md`、`traceability.json`(`aidlc engine sensor-traceability`)、`source-manifest.json` を作る。

## 要件・規則の対応(追跡)

| 要件 | 規則 | 実装のステップ | テスト |
|------|------|-----------------|--------|
| FR3.1 | BR1.1、BR3.4、BR4.4、BR4.5 | 7、8 | 9.3、9.4 |
| FR3.2 | BR1.2〜BR1.8、BR3.1〜BR3.3 | 3、5、8 | 6.1、6.2、6.3、9.2、9.3 |
| FR3.3 | BR4.1〜BR4.3 | 8 | 6.1、9.3 |
| FR3.4 | BR2.1〜BR2.4 | 5、7 | 6.2、9.3 |
| FR1.12 | BR3.3、BR5.1 | 3、8 | 4.1、9.4 |
| FR5.1 | BR6.1 | 8 | 9.1、9.2 |
| FR5.2 | BR6.2 | 11 | 11.1、11.2 |

## 順序の注意

- `prepare` の切り出し(8.1)は、本番の動きを変えない小さなコミットとして、単独で行う(既存のテストがそのまま通ること)。
- 各層は、「実装 → その層のテスト」の順(test-after)。
- 範囲を絞った実行のコマンドは、`unit-test-instructions.md` に書く。プロジェクト全体の行カバレッジ 80% は、Build and Test が、全体のテストで確かめる。
