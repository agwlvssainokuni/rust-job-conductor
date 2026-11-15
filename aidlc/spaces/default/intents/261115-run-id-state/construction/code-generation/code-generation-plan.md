# Code Generation Plan(run-id-state)

## Summary

- Builds: run ID によるステートの分離(`--run-id` と環境変数、定義ファイルの `[pipeline].run_id_env`、`<定義ファイル>.<run-id>.state`、中身の `run_id` の照合)、`watch`・`reset` の run ID 対応、`reset --all`、`validate` の `run_id_env` の検査、README。
- Touches: `crates/core`(`RunId` の検査、`PipelineDefinition.run_id_env`、`PipelineState.run_id`、定義の読み込み、誤りの種類)、`crates/runtime`(ステートのパス、読み書き・初期化の署名、`run_id` の照合、`reset_all_states`)、`crates/cli`(引数、`Runtime` trait と偽、環境変数の経路、`prepare`・`watch`・`reset` の流れ、表示と文言)、`README.md`、4 つの `Cargo.toml` の版(0.5.0)と `Cargo.lock`。
- Tests: 新しいテスト 約 30 件(要件ごと。名前の検査と `reset --all` の対象判定は、境界の入力を並べる)と、署名・フィールドの追加に伴う既存のテストの追従。それ以外の既存のテストは、そのまま成功。行カバレッジ 80% 以上を保つ。

## 前提(設計から)

- 要件: `inception/requirements-analysis/requirements.md`(FR1〜FR7、NFR1〜NFR4)。調査と変更箇所のファイル:行: `inception/reverse-engineering/developer-scan.md` の Handoff Summary((1)〜(7))。行番号は目安で、コードが正。zero-Unit(express)のため、この計画が設計を兼ねる。
- **`RunId`(core、新しいファイル `crates/core/src/run_id.rs`):** 検査済みの文字列の型 `RunId`(`parse(&str) -> Result<RunId, RunIdError>`、`as_str()`)。規則(FR1.4): 文字は `A-Za-z0-9._-`、1〜64 文字、先頭が `.` でない(`..` を含む)。`RunIdError` は、値を持たない種類だけ(空、長すぎる、使えない文字、先頭が `.`)。あわせて、環境変数の名前の検査 `is_valid_env_name(&str)`(`A-Za-z_` で始まり `A-Za-z0-9_` が続く、1 文字以上。FR2.2)を置く。どちらも純粋な関数で、ファイルや環境を読まない。`core/src/lib.rs` で公開する。
- **定義(FR2):** `PipelineDefinition.run_id_env: Option<String>` を足す。`definition/walk.rs` の `PIPELINE_KEYS` に `run_id_env` を足し、`description` と同じ形で文字列を取り出す(文字列でなければ `TypeMismatch`)。変数名の形が違えば(空を含む)、定義の誤りとして集める(新しい `DefinitionErrorKind`、例 `InvalidEnvName`。64。`messages.rs` の `definition_kind` に、英語と日本語の固定の文を足す。値を含めない)。`validate` も、同じ経路で検査される(FR6)。`format_version` は 1 のまま。
- **ステート(FR3):** `PipelineState.run_id: Option<String>` を足す。ステートの TOML には、`run_id` があるときだけ、キー `run_id` を書く(`TOP_KEYS` に足す。任意キー)。run ID なしのステートの形は、変えない(NFR1)。`initial_state` は、run ID を受け取って、`run_id` に入れる。パスは、`StateLocation::of(definition_path, run_id: Option<&RunId>)`: run ID があれば `<定義ファイル名>.<run-id>.state`、なければ `<定義ファイル名>.state`。`read_state`・`write_state`・`reset_state`(公開関数 3 つ)と、`Runtime` trait の対応するメソッド、`SystemRuntime`、テストの偽(`ScriptedRuntime`、`MemRuntime`)が、`run_id: Option<&RunId>` を受け取る形に揃える。
- **`run_id` の照合(FR3.2):** `read_state_file` が、期待する run ID を受け取り、`parse_state`/`walk` が `run_id` キーを読んだ直後に比べる: 期待が `Some(r)` なら、中身が `Some(r)` と等しいこと。期待が `None` なら、中身にキーがないこと。食い違いは、`corrupt`(新しい固定の理由 `RUN_ID_MISMATCH`。値を含めない)。**この検査は、`completed == true` を `None` として返す処理より前**に行う。
- **run ID の決め方(cli、FR1.1〜FR1.3、FR4.3):** 純粋な関数 `resolve_run_id(option: Option<&str>, env_name: &str, env: &dyn Env) -> Result<Option<RunId>, UsageError>`。`option` が `Some("")` なら `None`(固定)、`Some(v)` なら検査して `Some`、`None`(オプションなし)なら環境変数 `env_name` を読み、未定義と空は `None`、それ以外は検査(違反は 68)。検査の失敗の文には、値を含めない(出どころ=オプションか環境変数かだけ)。環境変数の名前は、`run`・`pipeline` では、`prepare` が定義を読んだあと(`check_definition` のあと、ジョブIDの確認とステートの読み込みの前)に、`definition.run_id_env.unwrap_or("CONDUCTOR_RUN_ID")`。`watch`・`reset` では、`--run-id` があれば定義を読まず、なければ `Runtime` の新しいメソッド `run_id_env(&self, definition_path) -> Option<String>`(ゆるく読む。ほかの誤りは無視し、読めない・キーがない・文字列でない・名前の形が違う、のときは `None` → 既定名)で名前を得る。ゆるい読み取りは、`core` に `peek_run_id_env(text: &str) -> Option<String>` を置き(TOML を解釈するが、`[pipeline].run_id_env` 以外を無視し、パニックしない)、`SystemRuntime` が、既存の取り込みと同じ大きさの上限でファイルを読んで渡す。`Ctx` へ `env` を渡す(`run_cli_with` から。`Reporter` が使う `Env` と同じものを借りる)。`--run-id`・`--all` は `CliCommand` のフィールドに足す(`--run-id` は、「指定なし」と「`""` で固定」を区別するため `Option<String>`)。
- **引数(FR1.1、FR5.3):** `Token::RunId(OsString)` と `--all`(フラグ)を、`lexopt` で解釈する。`--run-id` は `run`・`pipeline`・`watch`・`reset` だけ(`validate`・`convert` は `OptionNotAllowed` で 68)。`--all` は `reset` だけ(ほかは `OptionNotAllowed`)。`--all` と `--run-id` の併用、`--run-id` の重複、値の欠落、UTF-8 でない値は、使い方の誤り(68)。`--run-id` の値の検査は、引数の段階で行い(違反は `InvalidValue`)、環境変数の値の検査は、解決の段階で行う。`UsageErrorKind::ALL` と `usage_kind`、網羅のテストを揃える。
- **`reset --all`(FR5):** `Runtime` の新しいメソッド `reset_all_states(&self, definition_path) -> Result<ResetAllReport, StateError>`(`runtime` の公開関数 `reset_all_states` を呼ぶ)。`ResetAllReport { removed: usize, kept: Vec<String> }`(`kept` は、読めなくて残したファイル名)。実装: 定義ファイルと同じディレクトリを `read_dir` し、ファイル名が `<定義ファイル名>.state`(run ID なし。これは常に対象)か、`<定義ファイル名>.` で始まり `.state` で終わり、間が有効な `RunId` であるものだけを候補にする(隠しファイル・一時ファイル・ほかの名前は対象外)。候補は、シンボリックリンクを辿らず(`symlink_metadata`)、通常のファイルだけ。run ID 付きの候補は、読んで解釈し、`run_id` が間の文字列と一致すれば消す。`run_id` がない(別の定義のステート)ものは消さず、静かに無視する。読めない・壊れているものは消さず、`kept` に名前を入れる。`<定義ファイル名>.state` は、読まずに消す(`reset` と同じ)。削除に失敗したら、`RemoveFailed`(65)。ディレクトリの一覧に失敗したら、`Unreadable`(65)。cli は、`kept` があれば、標準エラーに警告(新しい `DisplayEvent`、ファイル名は既存のエスケープを通す。固定の文に `reset --run-id <ID>` で個別に消せることを添える)を出し、終了コードは 0。出力は、ほかに何もしない。
- **`watch`(FR4.1):** run ID があるときだけ、表の前に 1 行 `Run ID: <値>`(英語・日本語とも同じ形)を足す。`WatchTable` に `run_id: Option<String>` を足す(`PipelineState.run_id` を使う)。
- **文書(FR7):** `USAGE`(`messages.rs`、英語と日本語)の Usage と Options の行に `--run-id` と `reset --all`。README の使い方のコードブロック、コマンドの表、オプションの表、定義ファイルの書き方(`run_id_env`)、ステートファイルの場所(ファイル名、`run_id`)、終了コードの表、大文字小文字を区別しないファイルシステムでの注意(`A` と `a` を区別しない運用)。
- 1.85 でビルドできる書き方(let チェーンを使わない)。`unsafe` を増やさない。

## 実装の手順(test-after: 層ごとに、実装してから、その層のテストを書いて実行する)

- [x] **Step 1. 実行環境の確認:** `cargo test --workspace --locked` が成功する(597 件)ことを確かめ、`unit-test-instructions.md` のコマンドを確定する。
- [x] **Step 2. core を実装する(FR1.4、FR2、FR3.2 の型):** `run_id.rs`(`RunId`、`RunIdError`、`is_valid_env_name`)、`peek_run_id_env`、`PipelineDefinition.run_id_env`、`PipelineState.run_id`、`walk.rs` の `run_id_env`(取り出し、型と形の検査、新しい誤りの種類)。構造体リテラルを、コンパイルエラーで漏れなく直す。
- [x] **Step 3. core: テスト:** `RunId` の境界(`a`、`run-1.2_x`、長さ 64 と 65、空、`.a`、`..`、`a/b`、`a b`、`é`、制御文字、`CON` は有効)、`is_valid_env_name`、`peek_run_id_env`(正しい・壊れた・キーなし・型が違う・巨大な入力でパニックしない)、定義の `run_id_env`(正しい、`5`、`""`、`"1A"` が 64。FR2)。
- [x] **Step 4. runtime を実装する(FR3、FR5):** `StateLocation::of` の署名、`read_state`・`write_state`・`reset_state` の署名、`initial_state`、`serialize`・`TOP_KEYS` の `run_id`、`read_state_file` の照合(完了の判定より前)、`reset_all_states` と `ResetAllReport`、新しい固定の理由(`RUN_ID_MISMATCH`)、`lib.rs` の公開。
- [x] **Step 5. runtime: テスト:** パス(run ID の有無)、書いて読み戻す(`run_id` あり・なし)、run ID なしのステートの形が変わらない(NFR1)、照合(一致・不一致・中身にない・run ID なしで中身にある・完了のステートでも検査される)、run ID ごとの独立(`a` と `b` が互いに影響しない。FR3.3)、`reset_state` が run ID ごと、`reset_all_states` の境界(`x.toml.state`・`x.toml.a.state`・`x.toml.b.state` を消す、別の定義 `x.toml.a` のステート(`run_id` なし)を消さない、壊れた `x.toml.c.state` を消さず `kept` に入れる、一時ファイル `.x.toml.state.tmp.*`・`x.toml.bak`・`x.toml.a.statex`・無効な run ID(`x.toml...state`、`x.toml.a b.state`)を消さない、シンボリックリンクを辿らない(UNIX)、何もなくて成功、別のディレクトリのファイルに触れない。FR5、NFR2)。
- [x] **Step 6. cli を実装する(FR1、FR4、FR5、FR6):** `args.rs`(`--run-id`、`--all`、許可表、検査)、`types.rs`(`CliCommand`)、`deps.rs`(`Runtime` trait に `run_id_env`・`reset_all_states` を足し、既存の 3 メソッドの署名を変更。`SystemRuntime`)、`ctx.rs`(`env` の経路、`resolve_run_id`、`prepare` の順序、ステートのパスに run ID)、`run.rs`・`pipeline.rs`(run ID を使うステートの読み書き・初期化)、`aux_commands.rs`(`watch`・`reset`・`reset --all`)、`report/`(`WatchTable` の run ID の行、`reset --all` の警告の `DisplayEvent`)、`messages.rs`(usage、新しい文言、新しい誤りの種類)、`lib.rs`(`Env` の受け渡し)。`prepare` の観測できる順序(シグナル → 定義 → command → ジョブID → run ID の解決 → ステートの読み込みと照合)は、run ID の解決を足す以外、変えない。
- [x] **Step 7. cli: テスト:** 要件ごと — (a) 引数: `--run-id` が 4 つのコマンドで解釈、`validate`・`convert` で 68、`--all` は `reset` だけ、`--all --run-id` が 68、重複・欠落・UTF-8 でない値が 68、無効な値(`a/b`、`..`、`.a`、65 文字)が 68(FR1.1、FR1.4、FR5.3)、(b) `resolve_run_id`(オプション優先、環境変数、空、`--run-id ""` で固定、環境変数の値の違反は 68。FR1.2〜FR1.4)、(c) 流れ(偽の `Runtime`): `run`・`pipeline` が run ID ごとに別のステートを読み書きし、run ID なしは従来どおり。定義の `run_id_env` の名前を使う(FR1.2、FR3.1、FR3.3)、(d) `watch`・`reset`: `--run-id` があれば定義を読まない、なければ `run_id_env` をゆるく読み、壊れた定義でも `reset` が既定名で動く、`watch` が `Run ID:` の行を出す(FR4)、(e) `reset --all`: 警告の表示(`kept`)と終了コード 0、削除の失敗は 65(FR5)、(f) `validate` が `run_id_env` の型と形の誤りを 64 で拒否し、`--run-id` を受け付けない(FR6)、(g) 実際のバイナリ: 2 つの run ID の `pipeline` を並行して動かし、別々のステートファイルができ、互いの再開位置が変わらない、止まった run を同じ run ID で呼び直すと再開する、`reset --all` が `x.toml.*.state` を消し、無関係のファイルを消さない(FR3.3、FR3.4、FR5)。
- [x] **Step 8. 文書(FR7):** README と `--help` の文言。`docs-check` の必須の語句(SHA-256、Gatekeeper、終了コードの表、ステートファイルの場所、at-least-once、シグナル、convert)を消さないこと。
- [x] **Step 9. 版(NFR4):** 4 つの `crates/*/Cargo.toml` の `version` を 0.5.0 にし、`Cargo.lock` に追従させる。
- [x] **Step 10. 全体の確認:** `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked`、`cargo +1.85 build --workspace --locked --all-targets`、`cargo check --workspace --tests --target x86_64-pc-windows-msvc --locked`、`cargo llvm-cov` で行カバレッジ 80% 以上(補助の実行ファイルを除く)。README の語句検索(CI の `docs-check` と同じ)。`cargo build --release --locked --bin conductor` の実際のバイナリで、上の (g) と同じ流れを手で確かめる(`target/` の下の作業ディレクトリで)。デモ(`DEMO_SLEEP=0.1 sh demo/run-demo.sh`)が完走する。

## テストの方針(Minimal)

要件ごとに最も狭い層で 1 件を目安にし、名前の検査(`RunId`)と `reset --all` の対象判定には、NFR3 のとおり、境界の入力を並べたテストを加える(約 30 件)。署名・フィールドの追加のための既存のテストの修正は、追従だけ。使い方・文法を固定する既存のテストの期待値は、新しい引数に合わせて更新する。行カバレッジ 80% の下限は、Build and Test がワークスペース全体で確かめる。

## 設計からの追跡(要件 → 手順)

| 要件 | 手順 |
|------|------|
| FR1.1 | Step 6、7(a) |
| FR1.2 | Step 6、7(b)、7(c) |
| FR1.3 | Step 6、7(b) |
| FR1.4 | Step 2、3、6、7(a)、7(b) |
| FR2.1〜FR2.3 | Step 2、3、7(f) |
| FR3.1 | Step 4、5、7(c) |
| FR3.2 | Step 4、5 |
| FR3.3 | Step 5、7(c)、7(g) |
| FR3.4 | Step 7(g) |
| FR4.1〜FR4.3 | Step 6、7(d) |
| FR5.1〜FR5.4 | Step 4、5、6、7(e)、7(g) |
| FR6 | Step 2、7(f) |
| FR7 | Step 8 |
| NFR1 | Step 5、10(既存のテストが成功のまま) |
| NFR2 | Step 3、5(境界のテスト) |
| NFR3 | Step 10 |
| NFR4 | Step 9 |

## Assumptions

- 新しいファイル `crates/core/src/run_id.rs` を足す(検査の型は、独立した小さな単位が読みやすいため)。ほかは、既存のファイルの編集。
- `core` の公開の型 `RunId` と関数 `peek_run_id_env`、`runtime` の公開関数 `reset_all_states` と型 `ResetAllReport` を足す(要件の実現に必要)。

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
