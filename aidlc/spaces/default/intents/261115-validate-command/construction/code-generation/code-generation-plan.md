# Code Generation Plan(validate-command)

## Summary

- Builds: `conductor validate` サブコマンド(実行せず、ステートに触れずに、定義の検証と全ジョブの `command` の検査だけを行う)、`PipelineDefinition.jobs` → `job` の改名、README のタイトルと `validate` の説明。
- Touches: `crates/cli`(引数解釈、`Ctx` の検査の切り出し、`validate` の流れ、表示とメッセージ、使い方の文言、テスト)、`crates/core`・`crates/runtime`(フィールド名の追従)、`crates/converter`(単体テストのサンプルのキー)、`README.md`、4 つの `Cargo.toml` の版(0.4.0)と `Cargo.lock`。
- Tests: 新しいテスト 約 12 件(要件ごと。Minimal の方針)と、使い方・文法を固定する既存テストの期待値の更新。それ以外の既存のテストは、そのまま成功。行カバレッジ 80% 以上を保つ。

## 前提(設計から)

- 要件: `inception/requirements-analysis/requirements.md`(FR1〜FR3、NFR1〜NFR4)。調査と、変更箇所のファイル:行: `inception/reverse-engineering/developer-scan.md` の Handoff Summary、`aidlc/spaces/default/codekb/rust-job-conductor/`。zero-Unit(express)のため、この計画が設計を兼ねる。
- **改名(FR2):** `PipelineDefinition.jobs` → `job`。製品コードとテストの全使用箇所は、調査の一覧のとおり(コンパイルエラーで漏れを検出する。`jobs,` の省略記法の初期化は `job: jobs,` にする)。ローカル変数・引数名の `jobs`、`Kind::NoJobs`、英語の文面(`no jobs are defined`)、旧キーを拒否するテスト(`[[jobs]]`)、README の旧キーの注意書きは、変えない。`converter` の単体テストのサンプルのキー `"jobs"`(`yaml_write.rs`、`toml_write.rs`、`error.rs`、`tests/read_resolve.rs`)は `"job"` に直す(期待の文字列 `jobs:`、`jobs[1].note`、`jobs[2].transitions[0]`、説明コメントも)。動作は変えない。
- **`validate` の流れ(FR1):** `CommandKind::Validate` を足す(`types.rs`、`name()` は `"validate"`)。`Ctx::prepare` の「定義の読み込みと検証、`command` の検査」(調査の 2〜4)を、`prepare` と `validate` が共有する小さな関数に切り出す(誤りの表示と終了コード 64 は、いまのままにする)。`prepare` は、シグナルの準備 → 切り出した検査 → ジョブIDの確認 → ステートの読み込みと照合、の順(いまの順序を変えない)。`validate` は、切り出した検査だけを呼ぶ: シグナルの準備、ステート(`read_state`・`write_state`・`reset_state`)、`run_job` を呼ばない。名前は `load_definition(&path, cmd.pipeline_name.as_deref())` で、渡せば照合、なければ照合しない(`Runtime` trait に追加は不要)。
- **引数(FR1.1、FR1.5):** `validate` は `-v` と `--config` を受け付け(許可表に追加)、位置引数は 0 か 1 個(任意)。2 個以上、`--to` などの許可されないオプションは、使い方の誤り(68)。`CliCommand.pipeline_name` に詰める。
- **表示(FR1.6):** `DisplayEvent::DefinitionValid { job_count }` を足し、水準は `Verbose`(標準エラー。`-v` のときだけ)。文言は `messages.rs` に英語と日本語(例: `definition is valid (4 jobs)` / `定義は正しいです(ジョブ 4 個)`)。成功時は標準出力に何も書かない。誤りの表示は、いまの `DefinitionErrors` と `CommandErrors` を使う。
- **文言(FR1.1):** 使い方(`--help`)の文言(`messages.rs`)に `validate` を足す。
- 1.85 でビルドできる書き方(let チェーンを使わない)。`unsafe` を増やさない。

## 実装の手順(test-after: 層ごとに、実装してから、その層のテストを書いて実行する)

- [x] **Step 1. 実行環境の確認:** `cargo test --workspace --locked` が成功する(585 件)ことを確かめ、`unit-test-instructions.md` のコマンドを確定する。
- [x] **Step 2. 改名を実装する(FR2):** `crates/core/src/types.rs` のフィールドを `job` に改名し、`core`・`runtime`・`cli` の製品コード、ソース内の単体テスト、結合テストの使用箇所を追従させる。`cargo build --workspace --all-targets --locked` でコンパイルエラーがなくなるまで直す。
- [x] **Step 3. converter のテストのサンプルのキーを直す(FR2):** `yaml_write.rs`、`toml_write.rs`、`error.rs`、`tests/read_resolve.rs` のサンプルの `"jobs"` を `"job"` に(期待値も)。
- [x] **Step 4. 改名のテスト:** `cargo test --workspace --locked` が 585 件、すべて成功する(動作が変わっていないことの確認)。`PipelineDefinition` の Rust のフィールドに `jobs` が残らないことを、検索で確かめる。
- [x] **Step 5. cli: `validate` を実装する(FR1.1〜FR1.8):** `types.rs`(`CommandKind::Validate`)、`args.rs`(コマンド名の判定、許可表、位置引数 0 か 1 個)、`ctx.rs`(検査の切り出し)、`lib.rs`(振り分け)、`report/mod.rs` と `messages.rs`(`DefinitionValid` と文言、使い方の文言)。`validate` の流れの本体は、`aux_commands.rs`(`watch`・`reset`・`convert` と同じ、シグナルを使わない補助のコマンドの置き場)に置く。
- [x] **Step 6. cli: テスト:** 要件ごと — (a) 引数: `validate` / `validate NAME` / `-v` / `--config` が解釈され、名前 2 個・`--to` は 68(FR1.1、FR1.5)、`args_grammar.rs` の `CommandKind` の列挙と使い方の文言の期待値(`report_messages.rs`)の更新、(b) 偽の `Runtime` で: 正しい定義は 0 で、`read_state`・`write_state`・`reset_state`・`run_job` が 1 回も呼ばれず、シグナルの準備もされない(FR1.3、FR1.8)、(c) 定義の誤りと `command` の誤り(引用符が閉じていない)は 64 で、`run` と同じ誤りの文(FR1.2、FR1.5)、(d) 名前の照合: 一致で 0、不一致で 64、省略で照合なし(FR1.4)、(e) 表示: 既定は標準出力・標準エラーとも空、`-v` で標準エラーに 1 行(ジョブ数を含む)(FR1.6)、(f) 存在しないプログラムの `command` が 0(FR1.7)、(g) 実際のバイナリで、正しい定義が 0、誤りの定義が 64、`.state` が作られない、壊れた `.state` が隣にあっても 0(FR1.3)。
- [x] **Step 7. 文書(FR3):** `README.md` の 1 行目を `# rust-job-conductor — Rust製のジョブ実行制御ツール` にし(本文の `conductor` は変えない)、使い方のコードブロック、コマンドの表、オプションの表(`--config`・`-v` の対象に `validate`)、例、終了コードの表に `validate` を足す(実行しないこと、ステートに触れないこと、`-v` の動作)。`docs-check` の必須の語句を消さないこと(SHA-256、Gatekeeper、終了コードの表、ステートファイルの場所、at-least-once、シグナル、convert)。
- [x] **Step 8. 版(NFR4):** 4 つの `crates/*/Cargo.toml` の `version` を 0.4.0 にし、`Cargo.lock` に追従させる。
- [x] **Step 9. 全体の確認:** `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked`、`cargo +1.85 build --workspace --locked --all-targets`、`cargo check --workspace --tests --target x86_64-pc-windows-msvc --locked`、`cargo llvm-cov` で行カバレッジ 80% 以上(補助の実行ファイルを除く)。README の `docs-check` の語句検索(CI と同じ)を手元で実行する。実際のバイナリ(`cargo build --release --locked --bin conductor`)で、`conductor validate` を手で確かめる(正しい定義 0、誤りの定義 64、`.state` が作られない、`--help` に `validate`)。デモ(`DEMO_SLEEP=0.1 sh demo/run-demo.sh`)が完走する。

## テストの方針(Minimal)

要件ごとに、最も狭い層で 1 件を目安に、約 12 件を足す。改名のための既存のテストの修正は、フィールド名の追従だけ。使い方・文法を固定する既存のテストの期待値は、`validate` の追加に合わせて更新する。行カバレッジ 80% の下限は、Build and Test がワークスペース全体で確かめる。

## 設計からの追跡(要件 → 手順)

| 要件 | 手順 |
|------|------|
| FR1.1 | Step 5、6(a) |
| FR1.2 | Step 5、6(c) |
| FR1.3 | Step 5、6(b)、6(g) |
| FR1.4 | Step 5、6(d) |
| FR1.5 | Step 5、6(a)、6(c) |
| FR1.6 | Step 5、6(e) |
| FR1.7 | Step 6(f) |
| FR1.8 | Step 5、6(b) |
| FR2 | Step 2、3、4 |
| FR3.1 | Step 7 |
| FR3.2 | Step 7 |
| NFR1 | Step 4、9(既存のテストが成功のまま) |
| NFR2 | Step 5、6(c) |
| NFR3 | Step 9 |
| NFR4 | Step 8 |

## Assumptions

- `validate` の流れの本体を `aux_commands.rs` に置く(新しいファイルは作らない)。`validate` は、シグナルを使わない補助のコマンドと同じ種類のため。変更の範囲は、既存のファイルの編集だけ。

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
