# コード生成の計画: u2-optional-env-file(無くてもよい `.env`)

## Summary

- Builds: `env_file` に表 `{ path, required }` を許し、`required = false` のファイルが無ければ黙って読み飛ばす(core の型・検証・読む対象の一覧、runtime の読み取り、cli の差し替え口の型)
- Touches: `crates/core/src/types.rs`、`definition/walk/env.rs`、`envfile/mod.rs`・`merge.rs`、`plan/simulate.rs`、`lib.rs`、`crates/runtime/src/envfile.rs`、`crates/cli/src/deps.rs`・`ctx.rs`(と型に合わせる cli・core・runtime のテスト)、`README.md`、`demo/env.toml`・`demo/windows/env.toml`・`demo/README.md`
- Tests: 単体・統合テスト約 25(core の検証と一覧、runtime の読み取り)、非機能のテスト 3(秘密・性能)、cli の流れのテスト 3、実バイナリのテスト 3、デモ 1

出典: 機能の設計(`functional-spec.md`、`rules.md` の BR1.1〜BR4.2、`entities.md`)、非機能の設計(`security-design.md`、`performance-design.md`、`reliability-design.md`、`observability-design.md`、`logical-components.md`)、ユニットの定義(`unit-of-work.md` の U2)、要件(`requirements.md` の FR2.1〜FR2.5、FR8.1)、契約(`contract-summary.md` の C1、C3)。ユーザーストーリーは作っていないため、各ステップは要件と規則の ID に対応させる。

テストの進め方は Testing Contract(test-after)に従う。core → runtime → cli の順に、層を作ってからその層のテストを書いて走らせる。

## データの形の変更

- `EnvFileRef`(新規。`crates/core/src/types.rs`): `path: String`、`required: bool`。契約 C1 と `entities.md` の EnvFileRef。
- `PipelineDefinition.env_file` と `JobDefinition.env_file`: `Vec<String>` から `Vec<EnvFileRef>` に変える(`entities.md`)。
- `env_file_refs` の戻り値: `Vec<String>` から `Vec<EnvFileRef>` に変える。`entities.md` の EnvFileTarget は新しい型を作らず EnvFileRef を使う(`tech-stack-decisions.md`)。

## 設計のレビューからの持ち越し(このユニットで扱う)

- 読む対象は EnvFileRef の並びで返し、同じ形の別の型を作らない(機能の設計のレビュー)。
- cli の `Runtime::load_env_files` の引数の型、`ctx.rs` の呼び出し、偽の `Runtime` も変わる(機能の設計のレビュー)。
- 解決したパスでの required の集計は、読む前に 1 回の前処理で行う(機能の設計のレビュー、BR2.2)。
- 解決したパスの表は、誤りの並びが今と同じく参照の出現順になるよう、出現順を保つ形にする(非機能の設計のレビュー)。
- 要件の NFR1.5(飛ばしたことを出さない)、NFR6.4(誤りの表示は今のまま)、NFR7.1(fmt と clippy)は、Step 7、Step 8、Step 10 で確かめる(非機能の設計のレビュー)。

## ステップ

### Step 1: テストの準備を確かめる

- [x] 変更の前に、`unit-test-instructions.md` の 3 つのコマンドを走らせ、結果を記録する。

### Step 2: 型と定義の検証(core)

- [x] `crates/core/src/types.rs` に `EnvFileRef { path, required }` を足し(ドキュメントコメントは日本語で BR1.1〜BR1.3)、`PipelineDefinition.env_file` と `JobDefinition.env_file` を `Vec<EnvFileRef>` にする。`lib.rs` から再公開する。
- [x] `crates/core/src/definition/walk/env.rs` の `env_file` を、文字列・表・その配列を読む形にする。表は `check_keys` で `path`・`required` だけを許し、`path` は今の空の検査を共有し、`required` は省略時 true(BR1.1〜BR1.3)。理由の文は `security-design.md` の 2 の表のとおり(固定の文)。
- [x] `walk/mod.rs` の `PipelineHead`・`RawJob` の `env_file` の型を合わせる。

### Step 3: 読む対象の一覧と統合(core)

- [x] `crates/core/src/envfile/mod.rs` の `env_file_refs` が `Vec<EnvFileRef>` を返すようにする。パスの文字列で重複を除き、重複したときは required を論理和にする(文字列 → 位置の表で引く。BR2.1、`performance-design.md`)。範囲(`run` は `[pipeline]` とそのジョブ、ほかは全体)は今のまま。
- [x] `envfile/merge.rs`(`collect_files`)と `plan/simulate.rs` を、EnvFileRef の `path` で引く形に合わせる。引き当てに無い項目は今のまま何も足さない(BR2.3)。

### Step 4: core の層のテストを書いて走らせる

- [x] 型の変更に合わせて、定義を組み立てているテストと補助(`crates/core/src/test_support.rs`、`crates/core/tests/common/mod.rs`、`graph_common/mod.rs` など)の組み立て方だけを直す。確かめる内容は変えない。理由の文を見るテストは新しい文に合わせる。
- [x] `crates/core/tests/env_definition.rs` に、表の読み取り(path、required、省略時 true)、文字列と表の混在、表 1 つ、`[pipeline]` と `[job]` の両方、各誤り(path が無い・空・文字列でない、required が真偽でない、知らないキー、要素の型)を足す(BR1.1〜BR1.3、FR2.1)。
- [x] `crates/core/tests/env_merge.rs`(または `envfile/mod.rs` の `mod tests`)に、`env_file_refs` の重複除きと required の論理和、範囲、引き当てに無い項目が何も足さないことを足す(BR2.1、BR2.3)。
- [x] `crates/core/tests/nfr_security.rs` の `cases` に、表の型の誤り、知らないキー、`required` の型の誤りを足し、値の位置に目印を入れる(NFR6.1)。
- [x] `crates/core/tests/env_refs_perf.rs` に、表の項目を混ぜた入力のケースを足し、今と同じ上限で通ることを確かめる(NFR1.3)。上限は下げない。
- [x] core のコマンドを走らせ、通ることを確かめる。

### Step 5: 読み取り(runtime)

- [x] `crates/runtime/src/envfile.rs` の `load_env_files` の引数を `&[EnvFileRef]` にする。読む前に、解決したパスごとに required を論理和でまとめる表を、参照の出現順を保って作る(BR2.2)。
- [x] 読むときに `read_regular_file` が `Missing` を返し、そのパスの required が false なら、誤りにも警告にも入れず、引き当てにも載せない(BR2.3)。それ以外は今のまま(BR2.4、BR2.5)。誤りの並びは今と同じく参照の出現順。

### Step 6: runtime の層のテストを書いて走らせる

- [x] 型の変更に合わせて、`crates/runtime/tests/common/mod.rs`、`env_loader.rs`、`flow_with_state.rs` の組み立て方だけを直す。
- [x] `crates/runtime/tests/env_loader.rs` に足す: 無い × required の真偽、ディレクトリ・1 MiB 超・UTF-8 でない・書式の誤り × `required = false` で 64 の誤り、別の文字列が同じパスに解決されるときの論理和(片方だけ必ず要るなら FileMissing)、親のディレクトリが無いときは無いとして扱う、誤りの並びの順(BR2.2〜BR2.5、NFR6.3)。
- [x] runtime のコマンドを走らせ、通ることを確かめる。

### Step 7: cli の差し替え口と、cli のテスト

- [x] `crates/cli/src/deps.rs` の `Runtime::load_env_files` と `SystemRuntime` の実装、`crates/cli/src/ctx.rs` の呼び出しを `EnvFileRef` の並びに合わせる。流れの順番は変えない。偽の `Runtime`(`crates/cli/tests/common/mod.rs`)も引数の型だけ合わせる(NFR2.4)。
- [x] 型の変更に合わせて、cli のテスト(`flow_env.rs`、`flow_dry_run.rs`、`flow_graph.rs`、`src/tests/plan_report.rs` など)の組み立て方だけを直す。
- [x] `crates/cli/tests/flow_env.rs` と `flow_dry_run.rs` に、偽の `Runtime` で、無くてもよいファイルが無いときに `run` が 0 で変数が渡らないこと、`--dry-run` にその変数の名前が出ないことを足す(BR3.1、NFR1.5)。
- [x] `crates/cli/tests/binary_env.rs` に、実バイナリで、無くてもよいファイルが無いときに `run`・`pipeline`・`validate`・`--dry-run` が 0 で、標準出力・標準エラーがそのファイルを書かない定義と同じであること、あるが UTF-8 でないときは 64 で今と同じ誤りの表示であることを足す(NFR2.2、NFR1.5、NFR6.2、NFR6.4)。
- [x] cli のコマンドを走らせ、通ることを確かめる。

### Step 8: デモを更新する

- [x] `demo/env.toml` と `demo/windows/env.toml` の `env_file` に `{ path = "local.env", required = false }` を足す(`local.env` は置かない)。`demo/README.md` の環境変数の場面に一言足す(BR4.2、Q2 の A)。
- [x] `cargo test -p conductor-cli --test unix_demo --locked` を走らせ、デモの出力が変わらないことを確かめる(NFR3.2)。`windows_demo.rs` は CI の windows で確かめる。

### Step 9: README を更新する

- [x] `README.md` の `env_file` の説明に、表の形 `{ path, required }`、`required` の既定(true)、無いときの扱い(黙って進む)、あって読めないときは 64、同じファイルを両方で書いたときは必ず要るが優先、を足す。定義のキーの表と終了コード 64 の説明を合わせる(BR4.2、FR8.1)。`init` のひな形は変えない。
- [x] `cargo test -p conductor-cli --lib --locked -- tests::docs` が通ることを確かめる。

### Step 10: 書式と静的検査

- [x] `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通ることを確かめる(NFR7.1)。

## 規則と要件の対応

| ステップ | 規則 | 要件 |
|---|---|---|
| Step 2 | BR1.1〜BR1.3 | FR2.1、FR2.4 |
| Step 3 | BR2.1、BR2.3 | FR2.2、FR2.4 |
| Step 5 | BR2.2〜BR2.5 | FR2.2〜FR2.4 |
| Step 7 | BR3.1 | FR2.5 |
| Step 4、Step 6、Step 7 | BR1.1〜BR4.1 | FR2.1〜FR2.5、NFR1、NFR2、NFR6 |
| Step 8、Step 9 | BR4.2 | FR8.1 |
| Step 10 | | NFR7 |

## コミットの分け方

小さな単位でコミットする(件名は日本語、先頭に `core:`・`runtime:`・`cli:`・`docs:`)。各コミットで fmt と clippy が通るように、型の変更とそれに合わせる組み立ての修正は同じコミットに入れる。例: Step 2+3 の core の型と一覧(1〜2 コミット)、Step 4 のテスト、Step 5 の runtime とその型に合わせる cli の差し替え口(ビルドを通すため一緒に)、Step 6、Step 7 の cli のテスト、Step 8・Step 9 は `docs:`。版は上げない。

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
