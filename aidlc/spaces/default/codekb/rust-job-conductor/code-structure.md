# コードの構成

## リポジトリの配置

```text
./
├── Cargo.toml              ワークスペース(members = crates/*、resolver 3、edition 2024、rust-version 1.100、release の最適化)
├── Cargo.lock              追跡する。CI は常に --locked
├── README.md               利用者向けの説明(日本語、955 行)。CI の docs-check が必須の語句を検査
├── LICENSE                 Apache-2.0
├── .gitattributes          crates/cli/tests/fixtures/*.toml を LF に固定
├── .github/workflows/      ci.yml、release.yml、audit.yml
├── crates/
│   ├── core/               conductor-core
│   ├── runtime/            conductor-runtime
│   ├── cli/                conductor-cli(実行ファイル conductor)
│   └── converter/          conductor-converter
├── demo/                   デモ(UNIX の sh と Windows の cmd)
└── docs/spec/              再構築用の仕様(7 版 × 13 文書)
```

`target/`(ビルド出力)と `local-docs/`(個人の参照文書)は `.gitignore` で除かれる。`.idea/`(IDE の設定)は追跡されているが、この走査では開いていない。

## 仕様書の配置(`docs/spec/`)

| ディレクトリ | 内容 | 大きさ | 版の記述 |
|--------------|------|--------|----------|
| `rust/` | Rust 版の構成(クレート、公開の口、モジュール)と挙動。00〜12 の 13 文書 | 7,370 行 | v0.10.3 |
| `language-agnostic/` | 言語に依らない挙動と受け入れテスト(AC 番号)。13 文書 | 8,467 行 | 「v0.10.3 までの挙動」 |
| `csharp/`、`go/`、`powershell/`、`python/`、`typescript/` | 各言語での再構築の案内。各 13 文書 | — | 冒頭は v0.8.0・v0.9.0 |

`rust/02-architecture.md` はモジュールごとの表(ファイルの名前と責務)を持ち、この intent の 3 つのファイルを名前で書く(`code-quality-assessment.md` の「ファイルのパスを書く文書」)。

## クレートごとのモジュール

分類は `code-analysis-guide.md` の分類(Model、Service、Repository、Utility、Configuration、Controller、Middleware、Static、Test)。責務と依存の詳細は `component-inventory.md` に 1 度だけ書く。

### crates/core/src(conductor-core)

| パス | 分類 | 内容 |
|------|------|------|
| `lib.rs` | 構成 | 公開の口の再公開とモジュールの表(`//!`)。`#![forbid(unsafe_code)]` |
| `types.rs` | Model | 共有の型 |
| `time.rs` | Utility | `Timestamp`、整形・読み取り、所要時間 |
| `error.rs` | Utility | `DefinitionErrorKind`(34 変種)、表示の escape、行番号の変換 |
| `run_id.rs` | Model/Utility | `RunId`、環境変数の名前の規則 |
| `consistency.rs` | Service | ステートと定義の照合 |
| `definition/mod.rs`、`intake.rs`、`parse.rs`、`depth.rs`、`version.rs`、`duration.rs` | Service/Repository | 定義の読み込みの段(`mod walk;` は private。`walk::walk` を `:69` で呼ぶ) |
| `definition/walk/mod.rs`(695 行) | Service | キー・型・構造・遷移・終了コード・ジョブをまたぐ検査と組み立て(`Walker`)。この intent の対象 |
| `definition/walk/retry.rs`(366 行)、`env.rs`(281 行) | Service | `Walker` の `impl` の子(retry、env・env_file・unset_env) |
| `envfile/mod.rs`、`parse.rs`、`merge.rs` | Model/Service | `.env` の型(`EnvFileErrorKind` 14 変種)、解析、層の統合 |
| `flow/`(`mod.rs`、`classify.rs`、`decide.rs`、`next_step.rs`、`resume.rs`、`retry.rs`、`record.rs`、`attempt.rs`、`activity.rs`) | Service | 判断・記録・試行・動きの様子 |
| `graph/`(`mod.rs`、`edges.rs`、`overlay.rs`、`layout.rs`) | Service/Model | 図の構造と配置 |
| `plan/`(`mod.rs`、`simulate.rs`) | Model/Service | `--dry-run` の計画 |
| `test_support.rs` | Test | 単体テスト用の補助(`#[cfg(test)]`) |

### crates/runtime/src(conductor-runtime)

| パス | 分類 | 内容 |
|------|------|------|
| `lib.rs` | 構成 | 公開の口と契約・呼び出しの順序の文書。`#![deny(unsafe_code)]` |
| `types.rs` | Model | `CommandLine`、`InterruptSignal`、`StateLocation`、`Durability` |
| `error.rs` | Utility | `CommandError`、`StateError`、固定の理由の文(`pub(crate) mod reason`) |
| `split.rs`、`runner.rs`、`wait.rs` | Service | command の分割、起動とシグナル、待機 |
| `log.rs`、`clock.rs`、`envfile.rs`、`bounded_read.rs`、`temp_file.rs`、`file_create.rs` | Repository/Utility | ログ、時計、`.env`、上限つきの読み取り、一時ファイル、`init` の作成 |
| `state/reader.rs`、`writer.rs`、`reset_all.rs` | Repository | ステートの読み書きと削除(キーの表 `TOP_KEYS`・`JOB_KEYS`・`ATTEMPT_KEYS` は `reader.rs` の private) |
| `platform/mod.rs`、`unix.rs`、`windows.rs` | Configuration/OS | プロセスグループ、シグナルの送信、同期、0600 |
| `bin/conductor-test-helper.rs` | Test | テスト専用の補助バイナリ(配布しない) |

### crates/cli/src(conductor-cli)

| パス | 分類 | 内容 |
|------|------|------|
| `main.rs` | Controller | パニックのフックと `run_cli` |
| `lib.rs`(191 行) | Controller | `run_cli`、`run_cli_with`。モジュールはすべて private(`:90-103`)。最後が `#[cfg(test)]\nmod tests;` |
| `args.rs`(679 行) | Controller | 引数の解釈(表 `OPTIONS`、字句、`build_command`、値の解釈)。この intent の対象 |
| `types.rs`(322 行) | Model | `CommandKind`、`CliCommand`、`GraphFormat`、`UsageErrorKind`、`Language`、`Level`、`--interval` の定数、`RUN_ID_OPTION` |
| `ctx.rs`、`step.rs`、`run.rs`、`pipeline.rs`、`dry_run.rs`、`aux_commands.rs` | Service | 流れ |
| `run_id.rs` | Service | run ID の解決 |
| `deps.rs` | Configuration | 差し替え口の trait と本番の実装 |
| `exit.rs`、`panic_hook.rs` | Utility/Middleware | 終了コード、パニックのフック |
| `platform/mod.rs`、`unix.rs`、`windows.rs` | Configuration/OS | シグナルの監視、色と画面の消去の可否 |
| `report/mod.rs`(746 行) | Controller(表示) | `StateContext`、`ResultDetail`、`DisplayEvent`(37 種)、`FramePrefix`、`Reporter`、1 行の組み立て、エスケープ。この intent の対象 |
| `report/messages.rs`(780 行) | Static | 英日の文言(`USAGE`、`HEADER_*` を含む) |
| `report/watch.rs`、`graph_text.rs`、`graph_mermaid.rs`、`graph_dot.rs`、`plan_text.rs`、`init_template.rs` | Utility(整形) | 表、図、計画、ひな形 |
| `bin/conductor-cli-test-helper.rs` | Test | テスト専用の補助バイナリ |
| `tests/`(`src/tests/`。12 ファイル) | Test | 内部の単体テスト(`args_grammar.rs`、`docs.rs`、`escape_table.rs`、`exit_codes.rs`、`graph_dot.rs`、`graph_report.rs`、`init_template.rs`、`plan_report.rs`、`report_messages.rs`、`mod.rs`、`common/mod.rs`・`common/kinds.rs`) |

### crates/converter/src(conductor-converter)

`lib.rs`(`convert`、`convert_text`)、`types.rs`、`error.rs`(`ConversionErrorKind` 6 種)、`intake.rs`、`raw.rs`、`yaml_read.rs`、`toml_read.rs`、`scalar.rs`、`resolve.rs`、`yaml_write.rs`、`toml_write.rs`、`tests/`(4 ファイル)。前回の走査から変わっていない。

## この intent の対象のファイル

行は `9376cb7` のもの。3 つのファイルには `#[test]` も `mod tests` も無い(テストは別のファイル)。外からの参照の一覧は `component-inventory.md`、パスに依存するテストは `code-quality-assessment.md`。

### `crates/cli/src/report/mod.rs`

| 行 | まとまり | 依頼の 3 つとの対応 |
|----|----------|----------------------|
| 1-31 | `//!` | — |
| 33-39 | 子モジュールの宣言(`graph_dot`、`graph_mermaid`、`graph_text`、`init_template`、`plan_text` は `pub`、`messages` は `pub(crate)`、`watch` は private) | — |
| 41-59 | `use` | — |
| 61-97 | `StateContext`(5 変種、`for_read`)、`ResultDetail`(4 変種) | 定義 |
| 99-242 | `DisplayEvent<'a>`(37 変種。error 19、notice 5、verbose 5、output 8) | 定義 |
| 244-278 | `FramePrefix`(3 変種)、`CLEAR_SCREEN`、`for_frame`、`text` | 定義 |
| 280-323 | `DisplayEvent::level` | 定義 |
| 325-374 | `Reporter`(`new`、`language`、`color_enabled`、`show`) | 本体 |
| 376-564 | `Reporter::render`(35 の枝) | 本体(表示の種類ごとの関数に分ける対象) |
| 567-570、589-638、659-746 | `show_path`、`error_line`、`place_of`、`definition_line`、`env_file_lines`、`env_warning_line`、`job_place`、`command_line`、`usage_text`、`state_line`、`conversion_line`、`job_result_line` | 1 行の組み立て |
| 572-587 | `detect_language` | 本体 |
| 640-651 | `MAX_DISPLAY_CHARS`、`escape_name`、`escape_full` | エスケープ |
| 653-657 | `push_line`(`graph_text`・`plan_text`・`watch` が共有) | 置き場所は決めること |

`render` の 35 の枝の分類(水準ごと 19・5・5・6、表示の形ごと 9 種)は、intent の記録 `inception/reverse-engineering/developer-scan.md` の 1.3 に持つ。

### `crates/core/src/definition/walk/mod.rs`

| 行 | まとまり | 依頼の 3 つとの対応 |
|----|----------|----------------------|
| 1-45 | `//!`、`mod env; mod retry;`、`use` | — |
| 47-73 | キーの表(`ROOT_KEYS`、`PIPELINE_KEYS` 48-58、`JOB_KEYS` 59-69、`TRANSITION_KEYS` 70)と `NO_JOBS_REASON` | `PIPELINE_KEYS`・`JOB_KEYS` は `spec_numbers.rs` が読む(動かさない) |
| 75-125 | 中間の型(`PipelineHead`、`RawTransition`、`SuccessCodes`、`RawJob`)と `Walker` | 子が `super::` で使う |
| 128-151 | `push`、`type_error`、`check_keys` | 基本(子が使う) |
| 153-277 | `string`、`boolean`、`exit_codes`、`command` | 値の読み取り |
| 279-368 | `pipeline_section`、`pipeline_head`、`log_dir` | `[pipeline]` |
| 370-427 | `transitions`、`transition_on` | 遷移 |
| 429-465 | `job` | ジョブ |
| 467-554 | `cross_check`、`check_exit_codes` | ジョブをまたぐ検査 |
| 557-568 | `exit_code_of`、`integer_of`(`retry.rs` が使う) | 値の読み取りの補助 |
| 570-643 | `pub fn walk` | 入口 |
| 645-695 | `build_jobs`、`names_of` | 組み立て |

### `crates/cli/src/args.rs`

| 行 | まとまり | 依頼の 3 つとの対応 |
|----|----------|----------------------|
| 1-55 | `//!`(文法の表と規則) | — |
| 57-67 | `use` | — |
| 69-268 | `Opt`、`Allowed`、`OptSpec`、`ALL_BUT_CONVERT`、`RUN_AND_PIPELINE`、`OPTIONS`、コンパイル時の検査、`spec`、`spec_by_long`、`allows`、`subject` | オプションの表 |
| 270-340 | `Check`、`CHECKS`、`Given` | 検査 |
| 342-448 | `Token`、`Special`、`pub fn parse_args`、`map_unexpected_value`、`to_text` | 字句 |
| 450-581 | `build_command`(コマンドの枝は 551-579 の 5 つ) | 組み立て(枝ごとに分ける対象) |
| 583-639、668-679 | `parse_assumptions`、`parse_graph_format`、`parse_interval`、`parse_run_id_option`、`parse_target` | 値の解釈 |
| 641-666 | `Operand`、`operand_rule`、`unexpected_argument` | 検査(位置引数) |

## テストの配置

| 場所 | 内容 |
|------|------|
| `crates/core/tests/`(36 ファイル。補助 `common/`、`graph_common/`、`fixtures/`) | 定義、`.env`、流れ、試行、時刻、図、計画、性能、NFR |
| `crates/runtime/tests/`(16 ファイル。補助 `common/`) | 起動、シグナル、ログ、ステート、`.env`、NFR |
| `crates/cli/tests/`(27 ファイル。補助 `common/`、`bin_support/`、`signal_support/`、`fixtures/`) | 流れのテスト、実バイナリのテスト、シグナル、デモ、NFR、MSRV、仕様書の数(`spec_numbers.rs`) |
| `crates/converter/tests/`(2 ファイル) | 受け入れ、安全性 |
| 各モジュールの `mod tests`、`crates/cli/src/tests/`(12)、`crates/converter/src/tests/`(4) | 単体テスト |

この intent の 3 つのファイルのテスト: `report` は `src/tests/report_messages.rs`(45 テスト)・`escape_table.rs`(3)・`docs.rs`(22)と流れのテスト、`args` は `src/tests/args_grammar.rs`(69)、`walk` は core の統合テスト(`definition_errors.rs`、`definition_valid.rs`、`definition_retry.rs`、`env_definition.rs` など。入口は `parse_definition`・`load_definition`)。

## コードの型(パターン)

- **純粋な中核と入出力の殻**: 判断・記録・時刻・図・計画は core の純粋な関数。入出力は runtime と cli の `deps` に限る。
- **差し替え口(依存の注入)**: `Deps { runtime, sink, env, signals }` を `run_cli_with` に渡す。
- **private のモジュールと crate の中の `pub`**: cli の全モジュールと core の `walk` は private で、中の `pub` は crate の中だけで見える。分けるときは `pub use` で今のパス(`crate::report::DisplayEvent` など)を保てる。
- **`impl` の子モジュール**: `walk/retry.rs`・`env.rs` は `impl Walker<'_>` を持ち、親が呼ぶメソッドだけを `pub(super)` にする。親の private の型・関数は子から `super::` で見える。
- **表に性質を 1 か所**: `args.rs` の `OPTIONS` と `CHECKS`、`walk` のキーの表。表と列挙の並びはコンパイル時の `const _` で確かめる。
- **すべての誤りを集める検証**: `Walker` が誤りを位置つきで集め、位置で安定に並べて返す。引数は最初の誤りだけを返す。
- **固定の文のエラー**: 種類ごとの文を `messages.rs` か `label()`/`reason()` の 1 か所に持ち、入力の値を含めない。
- **リポジトリのファイルを読むテスト**: `CARGO_MANIFEST_DIR` からの相対パスで根を求め、README・`Cargo.toml`・`ci.yml`・ソース・仕様書・デモを読む(`docs.rs`、`msrv_consistency.rs`、`nfr_source.rs`、`spec_numbers.rs`)。
- **上限 + 1 バイトの読み取り**、**原子的な置き換え**、**英日の文言の対**(`messages.rs` の `Text`)。
- **文書の規則番号**: `//!`・`///` に BR/NFR/FR の番号と、版の Unit を書く。
