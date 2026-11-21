# コード生成の計画(Code Generation Plan): U1 u1-env-injection(環境変数の注入と `.env`)

## Summary

- Builds: 環境変数の注入(`[pipeline.env]`、`[job.env]`、`env_file`)、`.env` の読み取りと検査、層の統合、`CONDUCTOR_*` の自動の変数、`validate` の重複キーの警告、ジョブの起動への反映
- Touches: `crates/core`(型、定義の検査、新しい `envfile` モジュール)、`crates/runtime`(`.env` の読み取り、`run_job` の環境)、`crates/cli`(`Runtime` の trait、`prepare`、リトライ、`validate`、表示、メッセージ)、`README.md`、`demo/`
- Tests: 約 55 個(単体、偽の `Runtime` を使う流れのテスト、実バイナリのテスト)。既存の約 45 か所の型の作り方と、メッセージの網羅のテストを、合わせて直す

## 前提と範囲

- 設計の原本: `construction/u1-env-injection/functional-design/` の `entities.md`、`rules.md`(BR の番号)、`functional-spec.md`(WF1〜WF7)。契約: `inception/contract-design/contract-summary.md` の C1、C3、C4、C5(機能の設計の「契約への反映」で補正済み)。
- 新しい外部ライブラリは足さない(`Cargo.toml` と `Cargo.lock` は変えない)。`unsafe` は書かない。`cfg(unix)`/`cfg(windows)` は `platform` の外に置かない。エラーの表示とステートに値を含めない(`project.md`)。
- 作らない: `unset_env`、optional な `env_file`、変数の展開、複数行の値(要件の範囲の外)。`graph`、`--dry-run`、`init`(U2〜U4)。
- 範囲の外(別の依頼): 仕様書(`docs/spec/rust`、`docs/spec/language-agnostic`)の更新、版 0.9.0 への引き上げ、リリース。
- 変更を受ける、この作業の外の既存ファイル(名前を明記する): `README.md`(新しい節を足す。`docs-check` の必須語句は残す)、`demo/` の台本とデモのテスト。
- コミットは小さな単位で、直接 `main` に作る(`team.md`)。各ステップは、できるだけ 1 つのコミットにする(型の追加、読み取りと検証、表示、テストを分ける)。

## Assumptions

なし(設計の `entities.md` にない、データモデルの要素は足さない)。

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

適用する層: 「データモデル」は型と定義の誤りの種類(`core::types`、`core::error`)、「リポジトリ/データアクセス」は `.env` の読み取り(`runtime`)、「ビジネスロジック」は定義の検査、`.env` の解析、層の統合(`core`)と、起動の口(`runtime::run_job`)、「API/エンドポイント」はコマンドの流れと表示(`cli`)。「フロントエンド」は、この単一の実行ファイルの CLI にはないので、該当しない(表示は「API」の層に含める)。

## 実装のステップ

### Step 1: 構造と設定の骨格(production configuration skeleton)

- [x] 1.1 新しいモジュールの空の枠を作る: `crates/core/src/envfile/mod.rs`、`parse.rs`、`merge.rs`、`crates/runtime/src/envfile.rs`。`crates/core/src/lib.rs` と `crates/runtime/src/lib.rs` に、`mod` と公開の `use` を足す(BR は後のステップ)。外部ライブラリは足さない(`Cargo.toml` は変えない)。ビルドが通ることを `cargo build --locked --workspace` で確かめる。

### Step 2: テストの実行環境の確認(test runner readiness)

- [x] 2.1 既存のテストの実行環境(`cargo test --locked`)が動くことを確かめ、この Unit の、範囲を絞ったコマンドを `unit-test-instructions.md` に記録する(新しい実行環境や設定は作らない。`conductor-cli-test-helper` の `env <NAME>...` モードは、既にある)。

### Step 3: データモデル — 実装(型と誤りの種類。`core`)

- [x] 3.1 `crates/core/src/types.rs`: `EnvVar`(`name`、`value`。`value` は `secret`。`Debug` は値を伏せ字にする。BR8.1)を足す。`PipelineDefinition` と `JobDefinition` に `env: Vec<EnvVar>`、`env_file: Vec<String>` を足す(`entities.md`。FR1.1、FR1.4、FR1.13)。
- [x] 3.2 既存の型の作り方を直す: `PipelineDefinition { .. }`(22 か所)と `JobDefinition { .. }`(23 か所)の、`crates/core`(`types.rs`、`definition/walk.rs`、`flow/*.rs`、`tests/definition_retry.rs`、`tests/retry_flow.rs`)、`crates/runtime`(`runner.rs`、`state/consistency.rs`、`state/record.rs`、`tests/flow_with_state.rs`、`tests/common/mod.rs`)の、すべてに `env: Vec::new(), env_file: Vec::new()` を足す(挙動は変えない。小さなコミット)。
- [x] 3.3 `crates/core/src/error.rs`: `DefinitionErrorKind` に `ReservedEnvName`、`EnvNameCaseConflict`、`InvalidEnvValue` を足す(固定の表示文、`code()`、`ALL` を 33 に更新。BR1.3〜BR1.5)。`ALL` の件数のテスト(30 → 33)を直す。
- [x] 3.4 `crates/core/src/envfile/mod.rs`: `EnvFileErrorKind`(13 種)、`EnvFileError`(固定の理由、値を含まない。表示は `<ファイル>:<行>: <種類>: <理由>`)、`EnvWarning`、`EnvEntry`、`EnvFileContent`、`LoadedEnvFiles`、`EnvContext`、`EnvOrigin`、`EffectiveEnv`、`EffectiveEntry`(値を持つ型の `Debug` は値を伏せる。BR8.1)の型だけを定義する。

### Step 4: データモデル — テストを書いて実行する

- [x] 4.1 `crates/core/tests/env_types.rs`(新規): `EnvVar`、`EnvEntry`、`EffectiveEntry` の `Debug` が値を含まない(`assert_eq!` の失敗の出力を想定して、`format!("{:?}")` を検査)。`DefinitionErrorKind::ALL` が 33 種で、識別子が重複せず、表示が空でない。`EnvFileError` の表示に、値が含まれない(行あり、行なし、ファイル名の制御文字のエスケープ)。(BR8.1)

### Step 5: ビジネスロジック(定義の検査)— 実装

- [x] 5.1 `crates/core/src/definition/walk.rs`: `[pipeline]` と `[[job]]` の許可するキーに `env`、`env_file` を足す。`env` の表の検査(名前の規則 `is_valid_env_name`、予約名(ASCII の大文字で `CONDUCTOR_` 始まり)、同じ表の中の大文字小文字だけが違う名前、値が文字列、NUL)と、`env_file` の検査(文字列または文字列の配列、要素が文字列、空でない)を、`functional-spec.md` の WF1 の順に実装し、定義の型に入れる。誤りの理由は固定の文で、キー名は `quote_key` を通す。(BR1.1〜BR1.6、BR1.8、FR1.1、FR1.2、FR1.4、FR1.9、FR1.11、FR1.13)

### Step 6: ビジネスロジック(定義の検査)— テストを書いて実行する

- [x] 6.1 `crates/core/tests/env_definition.rs`(新規、8 個): 正しい `env` と `env_file`(文字列、配列、空の配列)が読める、新しいキーのない既存の定義が変わらず読める、値が文字列でない、名前の規則に合わない、予約名(`CONDUCTOR_X`、`conductor_x`)、同じ表の中の大文字小文字だけが違う名前(別の表の同じ綴りは誤りでない)、NUL を含む値、`env_file` の要素が文字列でない/空の文字列。誤りの表示に値が含まれない。(BR1.1〜BR1.8)

### Step 7: ビジネスロジック(`.env` の解析)— 実装

- [x] 7.1 `crates/core/src/envfile/parse.rs`: `parse_env_file(text, file) -> Result<EnvFileContent, Vec<EnvFileError>>` を、WF2 の順に実装する(BOM、LF/CRLF、空行とコメント、`export`、名前の検査と予約名、引用符なし/単一/二重引用符とエスケープ、展開と複数行を受け付けない、NUL、重複(後ろが勝つ。警告)、同じファイルの大文字小文字だけが違う名前、項目 10,000 の上限、誤りを集める)。値は誤りに含めない。(BR1.3〜BR1.5、BR2.1〜BR2.8、BR3.2、FR1.6、FR1.7、FR1.9)
- [x] 7.2 誤りの打ち切りの補助(20 件まで、残りの件数): 表示の部品(`cli`)で使う、`EnvFileError` の並びを要約する純粋な関数を `core::envfile` に置く。(BR2.8)

### Step 8: ビジネスロジック(`.env` の解析)— テストを書いて実行する

- [x] 8.1 `crates/core/tests/env_file_parse.rs`(新規、8 個): 受け付ける書き方(`A=1`、`export A=1`、`A = 1`、`A=x=y`、`A=x #c`、`A=x#y`、`A=`)、単一引用符とエスケープ、二重引用符のエスケープ 5 種と許されないエスケープ、閉じていない引用符と、閉じたあとのごみ、BOM と CRLF、重複(後ろが勝つ。警告に行と名前)、同じファイルの大文字小文字だけが違う名前、予約名と名前の誤り、NUL、項目 10,000 の上限(10,000 は通り、10,001 は誤り)、複数の誤りが行番号つきで集まり、値を含まない。(BR2.1〜BR2.8、BR3.2)

### Step 9: ビジネスロジック(層の統合)— 実装

- [x] 9.1 `crates/core/src/envfile/merge.rs`: `merge_env(def, job, files, ctx) -> EffectiveEnv` を、WF4 の順に実装する(層の順、配列の順、完全一致で後ろが勝つ、大文字小文字だけが違う名前は別の名前として並べる、自動の 4 つの変数、`CONDUCTOR_RUN_ID` は run ID があるときだけ、`CONDUCTOR_ATTEMPT` は試行)。`EffectiveEnv::additions()` と `names()`(値を含まない。引き継いだ環境の名前を含まない)を実装する。(BR4.1〜BR4.4、BR5.1〜BR5.3、BR8.2、FR1.3、FR1.4、FR1.10、FR1.12)
- [x] 9.2 読む対象の `env_file` の一覧を作る純粋な関数(`pipeline` は全ジョブ、`run` は 1 ジョブ。重複する記述を除く)を `core::envfile` に置く。(BR6.1)

### Step 10: ビジネスロジック(層の統合)— テストを書いて実行する

- [x] 10.1 `crates/core/tests/env_merge.rs`(新規、8 個): 層の優先順位(5 層)、`env_file` の配列の順、完全に同じ名前は後ろが勝つ、層をまたぐ大文字小文字だけが違う名前は両方が並びに残り後ろが後(Windows の違いは並びの順で確かめる)、自動の 4 つの変数(run ID なしで `CONDUCTOR_RUN_ID` がない、試行が 1 と 3)、`names()` が値を含まず出どころを持つ、`additions()` の順、読む対象の一覧(`pipeline` と `run` の範囲、重複を除く)。(BR4.1〜BR4.4、BR5.1〜BR5.3、BR6.1、BR8.2)

### Step 11: リポジトリ/データアクセス(`.env` の読み取り。`runtime`)— 実装

- [x] 11.1 `crates/runtime/src/envfile.rs`: `load_env_files(definition_path, refs) -> Result<LoadedEnvFiles, Vec<EnvFileError>>` を、WF3 の順に実装する(相対は定義ファイルのディレクトリが基準、絶対はそのまま、解決後に同じパスは 1 回だけ読む、大きさの検査と上限 + 1 バイトまでの読み取り、UTF-8、無い/読めない、解析は `conductor-core` の `parse_env_file`、誤りは全ファイル分を集める)。`unsafe` は書かない。(BR1.7、BR3.1、BR3.3、BR3.4、BR6.4、FR1.5、FR1.8、FR1.9)

### Step 12: リポジトリ/データアクセス — テストを書いて実行する

- [x] 12.1 `crates/runtime/tests/env_loader.rs`(新規、7 個、一時ディレクトリ): 相対パス(定義ファイルのディレクトリが基準)と絶対パスと `..`、複数ファイルを書いた順に読む、同じパスの別の書き方を 1 回だけ読む、ファイルが無い/ディレクトリ/読めない、1 MiB ちょうどは通り 1 MiB + 1 は誤り(中身を読まずに判断)、UTF-8 でない、複数ファイルの誤りを集める(行番号つき、値を含まない)。(BR1.7、BR3.1〜BR3.4、BR6.4)

### Step 13: ビジネスロジック(起動の口。`runtime`)— 実装

- [x] 13.1 `crates/runtime/src/runner.rs`: `run_job(job, env, signal)` に、`env: &[(&str, &str)]` を足す。並びの順に `Command::env` で足し、引き継いだ環境は残す。(BR4.3、BR4.4、FR1.3)
- [x] 13.2 `run_job` の呼び出し元(`crates/runtime` のテスト、`crates/cli/src/deps.rs`)の引数を、空の環境で合わせる(挙動は変えない)。

### Step 14: ビジネスロジック(起動の口)— テストを書いて実行する

- [x] 14.1 `crates/runtime/tests/runner_env.rs`(新規、5 個、`conductor-cli-test-helper` の `env` モード): 足した値が子に見える、引き継いだ環境が残る(足さなかった名前)、足した値が引き継いだ値を上書きする、並びの後ろが勝つ(同じ名前が 2 回ある並び)、空の環境では従来どおり。(BR4.4)

### Step 15: API/エンドポイント(コマンドの流れと表示。`cli`)— 実装

- [x] 15.1 `crates/cli/src/deps.rs`: `Runtime` の trait に `load_env_files(definition_path, refs)` を足し、`run_job` の引数に環境を足す。`SystemRuntime` で `conductor_runtime` の関数を呼ぶ。
- [x] 15.2 `crates/cli/src/ctx.rs`: `Session` に `LoadedEnvFiles` を持たせる。`prepare` に、存在しないジョブの検査と run ID の決定のあと、`read_state` の前で、`.env` をすべて読む処理を足す(誤りは表示して 64。BR6.1〜BR6.3、BR2.8)。`validate`(`aux_commands.rs`)は、`command` の検査のあとに、同じ読み取りと、警告の表示(0)を足す。(BR7.1、BR7.2)
- [x] 15.3 `crates/cli/src/retry.rs`: `run_job_with_retry` が `&Session` を受け取り、試行ごとに `merge_env`(`attempt` は、この呼び出しのリトライの回数 + 1)で環境を作って、`run_job` に渡す。`pipeline.rs`、`run.rs` の呼び出しを合わせる。(BR5.2、FR1.10)
- [x] 15.4 `crates/cli/src/report/`(`mod.rs`、`messages.rs`): `DisplayEvent` に `.env` の誤りの表示(20 件で打ち切り、「ほか N 件」)と、重複キーの警告を足す。メッセージは、日本語と英語の両方(`messages.rs` の網羅の配列にも足す)。定義の新しい 3 種の誤りの表示も足す。値は表示しない。終了コードは、64(`EXIT_DEFINITION`)を使い、新しい値は足さない。(BR2.8、BR9.1、FR5.1)

### Step 16: API/エンドポイント — テストを書いて実行する

- [x] 16.1 既存のテストの偽の `Runtime`(`crates/cli/tests/common/mod.rs` など)と、メッセージの網羅のテスト(`report_messages.rs`、`escape_table.rs`)を、新しい trait の形と、新しいメッセージの種類に合わせて直す。
- [x] 16.2 `crates/cli/tests/flow_env.rs`(新規、8 個、偽の `Runtime`): `pipeline` で `.env` の誤りがあるとジョブが 1 つも起動されず、ステートに触れず 64。`run` が `[pipeline].env_file` と、そのジョブの分だけを読む。再開でも読み直す。20 件を超える誤りの打ち切り。`validate` が、誤りで 64、重複キーの警告だけなら 0。使い方の誤り(68)が先に勝つ。リトライで `CONDUCTOR_ATTEMPT` が 1、2、3 と増え、再開で 1 に戻る。`run_job` に渡る環境の並びが、層の順。値が、標準出力、標準エラー、ステートに出ない。(BR5.2、BR6.1〜BR6.4、BR7.1、BR7.2、BR8.1、BR8.3)
- [x] 16.3 `crates/cli/tests/binary_env.rs`(新規、5 個、実バイナリと `conductor-cli-test-helper env`): `[pipeline.env]`、`[job.env]`、`.env`、`env_file` の配列の値が、実際の子のプロセスの環境に見える。優先順位。`CONDUCTOR_PIPELINE`、`CONDUCTOR_JOB_ID`、`CONDUCTOR_RUN_ID`(`--run-id` あり/なし)、`CONDUCTOR_ATTEMPT`。`.env` の誤りで終了コード 64 と、値が出力に無い(秘密の値が、標準出力、標準エラー、ステートファイルに含まれない。NFR2)。`validate` の警告。(FR1.1〜FR1.12)

### Step 17: 環境/ビルドの設定

- [x] 17.1 `Cargo.toml`、`Cargo.lock`、CI の設定は、変えない(新しい依存なし)。`crates/cli/tests/nfr_source.rs` が、新しいファイルについても、`unsafe` が無く、`cfg` が `platform` の外に無く、新しい依存が無いことを検査することを確かめる(必要なら対象の一覧を直す。しきい値は下げない)。

### Step 18: 文書と追跡

- [x] 18.1 `README.md`: 新しい節を足す(`[pipeline.env]`、`[job.env]`、`env_file`、`.env` の書式と上限、優先順位、`CONDUCTOR_*` の 4 つの変数、`validate` の警告、終了コード 64、バージョン v0.9.0 以降、受け入れた OS の違い(層をまたぐ大文字小文字だけが違う名前)と、環境全体の大きさの限界)。`docs-check` の必須語句は、残す。`crates/cli/tests/docs_env.rs`(新規、2 個): README に新しい必須の語句(`env_file`、`CONDUCTOR_ATTEMPT`)がある。
- [x] 18.2 `demo/`: 環境変数の場面を足す(`demo/conductor.toml` と `demo/windows/conductor.toml` に `[pipeline.env]` を足し、台本 `run-demo.sh` と `windows/run-demo.cmd` で表示。`demo/scripts` と `demo/windows/scripts` の、値を表示するスクリプトを足す)。デモのテスト(`unix_demo.rs`、`windows_demo.rs`)を合わせる。(FR5.2)
- [ ] 18.3 `code-summary.md`、`traceability.json`(`aidlc engine sensor-traceability`)、`source-manifest.json` を作る。

## 要件・規則の対応(追跡)

| 要件 | 規則 | 実装のステップ | テスト |
|------|------|-----------------|--------|
| FR1.1、FR1.2 | BR1.1、BR1.2 | 3、5 | 6.1 |
| FR1.3、FR1.4 | BR1.6、BR4.1〜BR4.4 | 5、9、13 | 6.1、10.1、14.1 |
| FR1.5 | BR1.7 | 11 | 12.1 |
| FR1.6 | BR2.1〜BR2.8、BR3.3 | 7、11 | 8.1、12.1 |
| FR1.7 | BR2.7、BR7.1、BR7.2 | 7、15 | 8.1、16.2 |
| FR1.8 | BR3.4、BR6.1〜BR6.4 | 11、15 | 12.1、16.2 |
| FR1.9 | BR1.5、BR3.1、BR3.2 | 5、7、11 | 6.1、8.1、12.1 |
| FR1.10 | BR5.1〜BR5.3 | 9、15 | 10.1、16.2、16.3 |
| FR1.11 | BR1.3 | 5、7 | 6.1、8.1 |
| FR1.12 | BR8.1〜BR8.3 | 3、9、15 | 4.1、10.1、16.2、16.3 |
| FR1.13 | BR1.8 | 5 | 6.1 |
| FR5.1 | BR9.1 | 15 | 16.1 |
| FR5.2 | BR9.2 | 18 | 18.1、18.2 |

## 順序の注意

- 型の作り方の修正(3.2)は、挙動を変えないコミットとして、先に単独で行う(多くのテストに影響する。リスク R: `risk-and-sequencing-rationale.md`)。
- 各層は、「実装 → その層のテスト」の順(test-after)。
- 範囲を絞った実行のコマンドは、`unit-test-instructions.md` に書く。プロジェクト全体の行カバレッジ 80% は、Build and Test が、全体のテストで確かめる。
