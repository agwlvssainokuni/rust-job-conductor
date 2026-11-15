# Code Generation Plan(command-array)

## Summary

- Builds: ジョブの `command` に、文字列に加えて、文字列の配列を書けるようにする(定義の検証、起動、表示、`convert` の往復、README、デモ)。
- Touches: `crates/core`(`Command` 型、定義の読み込み)、`crates/runtime`(`check_command`、`run_job`)、`crates/cli`(`Runtime` の trait、起動失敗の名前)、`crates/converter`(テストのみ)、`README.md`、`demo/`、各 `Cargo.toml` の版(0.2.0)、`Cargo.lock`。
- Tests: 新しいテスト 約 15 件(要件ごとに 1 件が目安。Minimal の方針)と、既存の 564 件を成功のまま保つ。行カバレッジ 80% 以上を保つ(チームの下限)。

## 前提(設計から)

- 要件: `inception/requirements-analysis/requirements.md`(FR1〜FR5、NFR1〜NFR4)。調査: `aidlc/spaces/default/codekb/rust-job-conductor/`。zero-Unit(express)のため、設計の成果物はなく、この計画が設計を兼ねる。
- **型:** `conductor-core` に `Command` を足す(`enum Command { Line(String), Argv(Vec<String>) }`、`Debug, Clone, PartialEq, Eq`)。`JobDefinition.command` と `JobExecution.command` を `Command` にする。`From<&str>` と `From<String>`(`Line` を作る)を実装して、既存のテストの `command: "echo".into()` が、そのまま動くようにする。`Command::program_name(&self) -> Option<&str>`(`Line` は `split_whitespace().next()`、`Argv` は先頭の要素)を足す(起動失敗の表示用)。
- **検証(FR1.3、FR1.4、FR1.5):** 定義を読むとき、`command` が TOML/YAML の文字列なら `Line`(空・空白だけは、従来どおり `EmptyField`)。配列なら、各要素が文字列であること(そうでなければ、型の誤り)、配列が空でないこと、先頭の要素が空・空白だけでないこと(いずれも `EmptyField`)。2 番目以降の空の文字列は有効。それ以外の型は、型の誤り。誤りのメッセージに、入力の値を入れない(NFR2)。
- **実行(FR2.1):** `conductor-runtime` の `check_command` は `&Command` を受ける。`Line` は従来の `split`、`Argv` は要素をそのまま引数の列にする(`CommandLine` を、同じ型で返す)。先頭が空のときは、`CommandErrorKind::Empty`(防御の二重の検査)。`run_job` は `check_command` と同じ経路を使い、引数の作り方以外は変えない(作業ディレクトリ、環境変数、シグナル、終了コードの扱いも同じ)。
- **表示(FR2.2):** `cli` の起動失敗の名前の取り出しは、`job.command.program_name()` を使う。長さ(64 文字)とエスケープは、従来の `escape_name` のまま。
- **変換(FR3):** `converter` は、値の木を変換する作りのため、配列はそのまま往復できる見込み。テストで確かめ、足りなければ直す。
- **版(NFR4):** 4 つのクレートの `version` を 0.2.0 にする(`release.yml` のタグの確認が、`crates/cli/Cargo.toml` の版を読む)。
- 1.85 でビルドできる書き方(let チェーンを使わない)。`unsafe` を増やさない。

## 実装の手順(test-after: 層ごとに、実装してから、その層のテストを書いて実行する)

- [x] **Step 1. 実行環境の確認:** 既存のテストの実行コマンドが動くことを確かめ、`unit-test-instructions.md` のコマンドを確定する(`cargo test --workspace --locked` が 564 件成功)。
- [x] **Step 2. core: `Command` 型を実装する(FR1.1、FR1.2):** `crates/core/src/types.rs` に `Command`、`From<&str>`・`From<String>`、`program_name` を足し、`JobDefinition.command` と `JobExecution.command` を `Command` にする。`crates/core/src/lib.rs` で公開する。
- [x] **Step 3. core: 型のテスト:** `Command` の `From` と `program_name`(`Line` と `Argv`、空の `Argv`)の単体テスト。既存の core のテストが成功する。
- [x] **Step 4. core: 定義の読み込みを実装する(FR1.3、FR1.4、FR1.5):** `crates/core/src/definition/walk.rs` の `command` の読み取りを、文字列と配列の両方にする(`exit_codes` の配列の読み取りを手本にする)。`RawJob.command` を `Option<Command>` にし、`JobDefinition` への組み立てを直す。
- [x] **Step 5. core: 定義のテスト:** 要件の受け入れ基準ごと: 配列が読める(FR1.2)、型の誤り(`5`、`["a", 5]`、テーブル。FR1.3)、`[]`・`[""]`・`["  ", "x"]` が `EmptyField`、`["echo", ""]` が有効(FR1.4)、空の文字列は従来どおり誤り(FR1.5)。誤りのメッセージに値を含まない(NFR2)。
- [x] **Step 6. runtime: `check_command` と `run_job` を実装する(FR2.1):** `crates/runtime/src/split.rs`、`runner.rs`、`lib.rs` を、`&Command` を受ける形にする。`Argv` は分割も展開もしない。
- [x] **Step 7. runtime: テスト:** 配列の引数が、そのまま渡る(空白、引用符、`$`、空の文字列を含む。補助の実行ファイルで、受け取った引数を確かめる。FR1.2、FR2.1)、先頭が空・要素が空の配列(`Empty`)、存在しないプログラムの配列が起動失敗(`start_failure`。FR2.2)。Windows の `.cmd` に配列でメタ文字を渡しても、続くコマンドが実行されない(FR2.3。既存の `cmd_への危険な引数は注入にならない` と同じ性質を、配列でも確かめる。`cfg(windows)`)。
- [x] **Step 8. cli: 実装する(FR2.2):** `crates/cli/src/deps.rs` の `Runtime::check_command` を `&Command` を受ける形にし、`ctx.rs` の起動失敗の名前を `program_name()` にする。テスト用の偽の `Runtime`(`tests/common/mod.rs` など)の署名を直す。
- [x] **Step 9. cli: テスト:** 配列のジョブの起動失敗(127)で、先頭の要素の名前が表示される。配列のジョブが、`run` と `pipeline` で、文字列のジョブと同じ結果になる(FR2.1。実際のバイナリを起動するテストで、1 件)。
- [x] **Step 10. converter: 往復のテスト(FR3):** 文字列と配列の `command` を含む定義を、TOML → YAML → TOML と YAML → TOML → YAML に変換して、同じ木になる。実装の修正は、テストが失敗したときだけ。
- [x] **Step 11. 設定: 版を 0.2.0 にする(NFR4):** 4 つの `crates/*/Cargo.toml` の `version` と `Cargo.lock`。
- [x] **Step 12. 文書とデモ(FR4、FR5):** `README.md` の「command の書き方」に、配列の節(1 要素 1 引数、分割も展開もしない、先頭の検査、空の引数、`format_version = 1` のまま、v0.1.x は配列を拒否すること)を書く。`demo/conductor.toml` に、配列の `command` のジョブ(`notify`)を入れ、`demo/run-demo.sh` の説明に 1 行足す。`demo/run-demo.sh` を実行して、最後まで成功することを確かめる。コードの doc コメントを、新しい型に合わせる。
- [x] **Step 13. 全体の確認:** `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked`、`cargo +1.85 build --workspace --locked --all-targets`、`cargo llvm-cov` で行カバレッジ 80% 以上(補助の実行ファイルを除く)。

## テストの方針(Minimal)

要件ごとに、最も狭い層で 1 件を目安に、約 15 件を足す。既存の 564 件は、型の変更に伴う修正(`check_command` の署名など)を除き、そのまま成功する。行カバレッジ 80% の下限(チームの実践)は、Build and Test がワークスペース全体で確かめる。

## 設計からの追跡(要件 → 手順)

| 要件 | 手順 |
|------|------|
| FR1.1 | Step 1、2、3、13 |
| FR1.2 | Step 2、4、5、7 |
| FR1.3 | Step 4、5 |
| FR1.4 | Step 4、5、6、7 |
| FR1.5 | Step 4、5 |
| FR2.1 | Step 6、7、9 |
| FR2.2 | Step 7、8、9 |
| FR2.3 | Step 7 |
| FR3 | Step 10 |
| FR4 | Step 12 |
| FR5 | Step 12 |
| NFR1 | Step 1、13(既存のテストが成功のまま) |
| NFR2 | Step 5(値を含まないメッセージ) |
| NFR3 | Step 13 |
| NFR4 | Step 11 |

## Assumptions

- 新しい型 `Command`(`conductor-core` の公開の型)を足す。要件が求める書き方(文字列または配列)を、型で表すために必要で、依頼の範囲内。
- 版を 0.2.0 にするのは、`Cargo.toml` の `version` だけ(タグ付けと公開は、この作業の範囲外)。

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
