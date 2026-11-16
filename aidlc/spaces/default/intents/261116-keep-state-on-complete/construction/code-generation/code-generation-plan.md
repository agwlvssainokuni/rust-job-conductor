# Code Generation Plan: 完了後もステートを残す(0.7.0)

## Summary

- Builds: 完了後もステートを残す標準の動作と、`--remove-on-complete` / `[pipeline].remove_on_complete` による削除、`watch` の完了表示
- Touches: `crates/core`(定義の項目)、`crates/runtime`(完了のステートを返す読み取り)、`crates/cli`(引数・完了処理・`watch`・文言)、`README.md`、`demo/`、4 クレートの版と `Cargo.lock`
- Tests: 要件ごとに 1 件以上の単体テストと、既存テストの前提の修正(`flow_*`・`binary_*`・`unix_demo`・`windows_demo`)、バイナリを実行する統合テストの追加

## 前提

- ゼロ Unit(express)。成果物は `construction/code-generation/` に置く。コードはワークスペースのルートの該当ファイルを、その場で直す(複製を作らない)。
- 方式は test-after。各層を実装してから、その層のテストを書いて実行する。
- 完了のステートは、今の `pipeline.rs` と `run.rs` が、削除の前に、`completed = true` で書いている(`Durability::Durable`)。だから、削除の呼び出し(`ctx.reset`)を、条件つきにするだけで、FR1 は満たせる。ステートの形式は変えない。
- `read_state` は、完了のステートを空として返す挙動のまま、変えない(FR1.3)。`watch` だけが、完了のステートを区別して読む。
- ソースの規約: Apache-2.0 のヘッダー、`conductor-cli` に `forbid(unsafe_code)`、let チェーン不可(Rust 1.85)、利用者向けの文章は日本語(英語の出力は、既存どおり、言語で切り替える)。

## 実装の手順

### 層 0: 準備

- [ ] **Step 1**: 既存のテストの実行環境を確かめ、層ごとの実行コマンドを `unit-test-instructions.md` に記録する(新しい依存や設定ファイルは足さない)。ベースラインとして、`cargo test --workspace --locked` が成功することを、変更の前に確かめる。

### 層 1: 定義(`crates/core`)— FR2.2、FR2.4

- [ ] **Step 2**: `PipelineDefinition`(`crates/core/src/types.rs`)に、`remove_on_complete: bool`(省略時は `false`)を足す。`Default`・既存のコンストラクターの初期値を合わせる。
- [ ] **Step 3**: `crates/core/src/definition/walk.rs` の `[pipeline]` の読み取り(`PIPELINE_KEYS`、`PipelineHead`)に、`remove_on_complete` を足す。真偽値でなければ、既存の「型が違う」の誤りの経路で、定義ファイルの誤り(64)にする。`validate` も、同じ検査を通る(`Ctx::check_definition` が共通)。
- [ ] **Step 4**: 層 1 のテスト(単体): 省略時は `false`、`true`・`false` が読める、文字列・数値は誤り、他のキーとの共存、未知のキーの拒否が今までどおり(FR2.2、FR2.4)。

### 層 2: ステートの読み取り(`crates/runtime`)— FR3.1、FR3.2、FR3.4

- [ ] **Step 5**: `crates/runtime/src/state/reader.rs` に、`read_state_raw(definition_path, run_id) -> Result<Option<PipelineState>, StateError>` を足す。完了のステートも、そのまま返す。ファイルがない・長さ 0 は `Ok(None)`。版・破損・run ID の食い違いの検査は、`read_state` と同じ(FR3.4)。`read_state` は、`read_state_raw` を呼び、`completed` なら `None` にする形に直す(挙動は変えない)。`lib.rs` の表と再公開を合わせる。
- [ ] **Step 6**: 層 2 のテスト(単体): `read_state_raw` が完了のステートを返す、`read_state` は同じファイルを空として返す、ファイルなし・長さ 0 は両方 `None`、版違い・壊れたファイル・run ID の食い違いは両方とも同じ誤り。

### 層 3: CLI(`crates/cli`)— FR1、FR2.1、FR2.3、FR2.5、FR3、FR4

- [ ] **Step 7**: `args.rs`: `--remove-on-complete`(値なし)を、字句の段階と表に足す。`pipeline` と `run` だけが受け付ける。`watch`・`reset`・`validate`・`convert` に付けたら、`OptionNotAllowed`(68)。2 回以上の指定は、既存の重複の扱い(`-v` の 2 回以上と同じ規則)に合わせる。`CliCommand` に `remove_on_complete: bool` を足す。冒頭の表のコメントを直す。
- [ ] **Step 8**: `report/messages.rs`: 使い方(`--help` と使い方の誤りの表示、英語・日本語の両方)に、`--remove-on-complete` を足す。`watch` の完了の行の文言(`WATCH_COMPLETED`: 英語 `completed`、日本語 `完了`)を足す。
- [ ] **Step 9**: `ctx.rs`: `Session` に、実効の `remove_on_complete`(オプションが指定されたら `true`、なければ定義の値)を持たせ、`prepare` で決める(FR2.3)。
- [ ] **Step 10**: `pipeline.rs` の `finish`(完了後の初期化。2 か所から呼ばれる)と、`run.rs` の完了の 2 か所の `ctx.reset` を、実効の `remove_on_complete` が `true` のときだけ呼ぶ形にする。`false` なら、削除せず、そのまま 0 で終える(完了のステートは、すでに書いてある)。モジュールの説明コメントと、`StateContext` の記述も合わせる(FR1.1、FR2.1)。
- [ ] **Step 11**: `deps.rs` の `Runtime` trait に `read_state_raw` を足し、`SystemRuntime` は runtime の `read_state_raw` を呼ぶ。テスト用のフェイクの実装(`tests/` の共通部)も合わせる。
- [ ] **Step 12**: `aux_commands.rs` の `watch`: `read_state_raw` で読み、状態があれば、今までの表を出し、`state.completed` のときだけ、表の下に完了の行を 1 行足す(`report/watch.rs` の組み立てに、完了の行を足すか、`WatchView` で組み立てる)。ファイルなし・長さ 0 のときだけ `no state`。run ID あり・なしの両方で同じ(FR3.1〜FR3.3)。終了コードは 0。
- [ ] **Step 13**: 層 3 のテスト(単体・統合): 下の「テスト」の表のとおり。

### 層 4: 既存テストの前提の修正

- [ ] **Step 14**: 完了後にステートがなくなる前提のテストを、新しい挙動に直す(FR5.3): `flow_pipeline.rs`(完了後の初期化の失敗が 65 になるテストは、`--remove-on-complete` を付けた場合のテストに直し、付けない場合は、完了のステートが残ることを確かめるテストを足す)、`flow_run_id.rs`、`flow_aux.rs`(`watch_は完了のまま残ったステートを空として扱う` は、完了を示す行を確かめるテストに直す)、`binary_basic.rs`、`binary_run_id.rs`、`unix_demo.rs`(`no state` の期待)、`windows_demo.rs`。

### 層 5: 文書とデモ

- [ ] **Step 15**: `README.md`: クイックスタートの 5(完了後もステートが残り、`watch` が完了を示す)、`pipeline` の説明、`ステートファイルの場所` の節(完了で消える、という記述を、残る、に直し、`--remove-on-complete` で消す、を書く)、run ID の小節(完了した run は消える、という記述、「ステートは、自動では消えません」との整合)、`watch` の説明と出力例、オプションの表(`--remove-on-complete`)、定義ファイルの書き方(`[pipeline].remove_on_complete`)、終了コードの表(68 の例)、保証の範囲、版の注記(0.7.0 以降)。`docs-check` の 7 語句と、それを含む見出しは変えない。
- [ ] **Step 16**: デモ: `demo/run-demo.sh` と `demo/windows/run-demo.cmd` の場面 4・8・9(完了後に `watch` が完了を示す、ステートが残る、`--remove-on-complete` で消える、片づけは `reset`)と、2 つの `conductor.toml`(必要なら)と、`demo/README.md` を合わせる。

### 層 6: 版

- [ ] **Step 17**: 4 クレートの `Cargo.toml` の版と `Cargo.lock` を 0.7.0 にする(`cargo build --locked` が通ること)。タグは付けない。

### 層 7: 確認

- [ ] **Step 18**: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked`、`cargo +1.85 build --workspace --locked --all-targets` が成功すること。README の `docs-check` の 7 語句が残ること。
- [ ] **Step 19**: `code-summary.md` と `traceability.json` を書く。

## テスト(要件との対応)

| 要件 | テスト(層・場所) |
|------|-------------------|
| FR1.1 | 統合(`flow_pipeline`): 最後のジョブの完了、`@end`、スキップでの完了のそれぞれで、ステートが残り `completed = true`。`run` の完了(単独のジョブ)でも残る。run ID あり・なしの両方(`flow_run_id`) |
| FR1.2 | 統合: 完了後のファイルを `read_state_raw` で読んで、形式が今までどおり(`format_version = 1`)。書き込みは既存の原子的な手順 |
| FR1.3 | 統合: 完了のステートが残った状態で `pipeline` を再実行すると、先頭から始まる。先頭以外のジョブを `run` で単独に呼ぶと 66 |
| FR1.4 | 統合: 完了のステートの書き込みが失敗すると 65(既存のフェイクの書き込み失敗) |
| FR2.1 | 統合: `--remove-on-complete` を付けると、完了でファイルが消える。`pipeline` と `run` の両方 |
| FR2.2・FR2.4 | 単体(core)と統合: 定義の `remove_on_complete = true` で消える。型の誤りは 64。`validate` も 64 |
| FR2.3 | 統合: オプションあり・定義 false → 消える。オプションなし・定義 true → 消える。両方なし → 残る |
| FR2.5 | 単体(args)と統合: `watch`・`reset`・`validate`・`convert` に付けると 68 |
| FR2.6 | (環境変数を見ないことは、設計。テストは、環境変数を設定しても残ることを 1 件) |
| FR3.1〜FR3.4 | 単体(runtime、report)と統合(`flow_aux`、`binary_*`): 表の下に完了の行(英語・日本語)。`no state` は、ファイルなし・長さ 0 のときだけ。run ID あり。完了のステートが壊れていると 65 |
| FR4.1 | 統合: `reset`・`reset --all` が、完了のステートも消す(run ID あり・なし) |
| FR5.1・FR5.2 | `docs-check` の 7 語句、デモのテスト(`unix_demo`、`windows_demo`)、README のクイックスタートの手順の再実行 |
| FR5.3 | 既存テストの修正と追加(Step 14) |
| FR6.1 | `conductor --version` が `conductor 0.7.0`(バイナリのテスト) |
| NFR1〜NFR5 | fmt・clippy・テスト・1.85 のビルド・カバレッジ(Build and Test で測る) |

## Testing Contract

```json
{
  "version": 1,
  "methodology": "test-after",
  "source": "team",
  "ordering": "各テスト可能な層を実装してから、その層のテストを書いて実行する。",
  "scope": "express",
  "test_strategy": "minimal",
  "project_type": "brownfield",
  "applicable_notes": [
    {
      "layer": "org",
      "text": "We treat tests as a first-class deliverable in every Bolt. The specific\nmethodology (TDD, BDD, ATDD, or classic test-after) is affirmed at\npractices-discovery and recorded in `team.md` under this heading with explicit\n`Methodology` and `Ordering` fields; Code Generation resolves those fields\nindependently from coverage, tooling, and scope notes.\n\nWhen no posture has been affirmed, our default per scope is:\n- **Methodology**: test-after\n- **Ordering**: implement each applicable testable layer, then write and run\n  that layer's tests.\n- `mvp`, `enterprise`, `feature`, `infra` add an 80% line-coverage\n  floor and CI execution before merge.\n- `bugfix`, `security-patch` add a targeted regression for the specific\n  bug/vulnerability and require the existing suite to remain green.\n- `express` uses the Minimal strategy: requirement-driven unit tests (one per\n  requirement, with a happy-path floor per component); existing tests remain\n  green.\n- `poc`, `refactor`, `workshop`, `classic` add no extra new-test floor and require the\n  existing suite to remain green.\n\nThe active `Test Strategy` still applies in every scope and determines test\nvolume/types. Scope floors are additive; they never reduce or replace the\nselected strategy.\n\nBuild and Test verifies defined coverage floors and affirmed quality targets;\nthey may not be weakened to make a step pass.\n\nAffirm a stricter posture in `team.md` if the team commits to one."
    },
    {
      "layer": "team",
      "text": "- **Methodology**: test-after\n- **Methodology evidence**: インタビュー Q3 で、チームは test-after(実装してからテストを書く)を選んだ。\n- **Ordering**: 各テスト可能な層を実装してから、その層のテストを書いて実行する。\n- `cargo test` による単体テストと、バイナリを実行する統合テストを含める。\n- 行カバレッジ 80% を下限とする(インタビュー Q4: A)。\n- 主な開発環境の OS では、ローカルでテストを実行する。それ以外の OS(Linux・macOS・Windows のうち該当するもの。制約 C3)のテストは、GitHub Actions の CI で実行する。マージ前に CI を通す。"
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
  "input_sha256": "sha256:47c2e6e6f234580fd227d246415c56f693b43f2a7953fee56ef7217b54cbd097",
  "contract_sha256": "sha256:3e75c1c022f49e57c8d1612ebaf8b3e7016754454c19abb2dc7f313d6288a4e3"
}
```
