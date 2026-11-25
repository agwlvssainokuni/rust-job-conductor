# コードの構成

## リポジトリの配置

```text
./
├── Cargo.toml              ワークスペース(members = crates/*、edition 2024、rust-version 1.100、release の最適化)
├── Cargo.lock              追跡する。CI は常に --locked
├── README.md               利用者向けの説明(日本語)。CI の docs-check が必須の語句を検査
├── LICENSE                 Apache-2.0
├── .gitattributes          crates/cli/tests/fixtures/*.toml を LF に固定
├── .github/workflows/      ci.yml、release.yml、audit.yml
├── .idea/                  IDE のプロジェクト設定(追跡。個人の設定は .idea/.gitignore で除く)
├── crates/
│   ├── core/               conductor-core
│   ├── runtime/            conductor-runtime
│   ├── cli/                conductor-cli(実行ファイル conductor)
│   └── converter/          conductor-converter
├── demo/                   デモ(UNIX の sh と Windows の cmd)。定義、.env、スクリプト
└── docs/spec/              再構築用の仕様(7 言語版 × 13 文書)
```

`target/`(ビルド出力)と `local-docs/`(個人の参照文書)は `.gitignore` で除かれる。

## クレートごとのモジュール

### crates/core/src(conductor-core)

| パス | 分類 | 内容 |
|------|------|------|
| `lib.rs` | 構成 | 公開の口(契約 C2)の再公開。`#![forbid(unsafe_code)]` |
| `types.rs` | Model | 共有の型(`PipelineDefinition`、`JobDefinition`、`Command`、`RetryPolicy`、`Backoff`、`EnvVar`、`Transition*`、`JobExecution`、`ExecutionResult`、`JobStatus`、`JobState`、`PipelineState`、`Decision`、`Classification`、`JobOutcome`、`NextStep`、`RunMode`、`ResumePoint` など) |
| `error.rs` | Utility | `DefinitionErrorKind`(33 種)、`DefinitionError`、`escape_display`、`quote_key`、`Locate`(バイトの位置 → 行) |
| `run_id.rs` | Model/Utility | `RunId`(`A-Za-z0-9._-`、64 文字まで、先頭が `.` でない)、環境変数の名前の規則、`peek_run_id_env` |
| `consistency.rs` | Service | `check_consistency`(パイプライン名、ジョブIDの集合、並び) |
| `definition/mod.rs` | Service | `load_definition`、`parse_definition`(Intake → Parse → Depth → VersionCheck → Walk) |
| `definition/intake.rs` | Repository | 定義ファイルの場所の決定と、上限(1 MiB)+ 1 バイトまでの読み取り。core で唯一ファイルに触れる |
| `definition/parse.rs`、`depth.rs`、`version.rs` | Service | 位置つきの木、入れ子 32 段、`format_version = 1` |
| `definition/duration.rs` | Utility | `<整数><s\|m\|h>` を秒に |
| `definition/walk/mod.rs` | Service | キー・型・構造・遷移・終了コード・名前の検査と組み立て(誤りはすべて集める) |
| `definition/walk/retry.rs` | Service | `[job.retry]` の検査と組み立て |
| `definition/walk/env.rs` | Service | `env`、`env_file` の検査 |
| `envfile/mod.rs` | Model | `.env` の型(`EnvFileErrorKind` 13 種、`EnvEntry`、`LoadedEnvFiles`、`EnvContext`、`EffectiveEnv`)、`env_file_refs`、上限(1 MiB、10,000 項目) |
| `envfile/parse.rs` | Service | `.env` の解析(純粋) |
| `envfile/merge.rs` | Service | 層の統合(`merge_env`)、名前だけの統合(`merge_env_names`) |
| `flow/mod.rs` | Service | 判断の再公開と共通の補助 |
| `flow/classify.rs`、`decide.rs`、`next_step.rs`、`resume.rs`、`retry.rs` | Service | 分類と終了コード、起動前の判断、次の動きと完了、ジョブのあいだの再開位置、リトライの判断 |
| `flow/record.rs` | Service | `initial_state`、`apply_outcome`、`apply_skip`、`apply_interrupt`(本番と模擬が共有) |
| `graph/mod.rs`、`edges.rs`、`overlay.rs`、`layout.rs` | Service/Model | 図の構造(`FlowGraph`)、辺、状態の重ね合わせ、レーンの配置(`GraphLayout`) |
| `plan/mod.rs`、`simulate.rs` | Service/Model | `--dry-run` の計画の型と、模擬(`plan_pipeline`、`plan_run`) |
| `test_support.rs` | Test | 単体テスト用の組み立ての補助(`#[cfg(test)]`) |

### crates/runtime/src(conductor-runtime)

| パス | 分類 | 内容 |
|------|------|------|
| `lib.rs` | 構成 | 公開の口(契約 C3)の再公開。`#![deny(unsafe_code)]` |
| `types.rs` | Model | `CommandLine`、`InterruptSignal`(Atomic)、`StateLocation`、`Durability` |
| `error.rs` | Utility | `CommandError`(2 種)、`StateError`(6 種)と固定の理由の文 |
| `split.rs` | Service | `check_command`(command の分割の状態機械) |
| `runner.rs` | Service | `run_job`(直接起動、`try_wait` のくり返し、シグナルの転送) |
| `wait.rs` | Service | `wait_for`(リトライの待機。中断を見る) |
| `envfile.rs` | Repository | `load_env_files`(パスの解決、上限つきの読み取り、UTF-8、解析は core) |
| `bounded_read.rs` | Utility | 通常のファイルだけを、上限 + 1 バイトまで読む(`.env` とステートが共有) |
| `temp_file.rs` | Utility | 一時ファイルの名前と作成、`replace_via_temp`(ステートと `init` が共有) |
| `file_create.rs` | Repository | `create_file`(`init` の排他的な作成、`--force` で置き換え) |
| `state/reader.rs` | Repository | `read_state`、`read_state_raw`(厳格なキーの表と内容の矛盾の検査) |
| `state/writer.rs` | Repository | `write_state`(原子的な置き換え)、`reset_state`、古い一時ファイルの後始末 |
| `state/reset_all.rs` | Repository | `reset_all_states`(`reset --all`) |
| `platform/mod.rs`、`unix.rs`、`windows.rs` | Configuration/OS | プロセスグループ、シグナルの送信、シグナル終了の読み取り、ディレクトリの同期。`unsafe` は `unix.rs` の 1 か所 |
| `bin/conductor-test-helper.rs` | Test | テスト専用の補助バイナリ(配布しない) |

### crates/cli/src(conductor-cli)

| パス | 分類 | 内容 |
|------|------|------|
| `main.rs` | Controller | パニックのフックを設定し、`run_cli` の結果で 1 回だけ終了 |
| `lib.rs` | Controller | `run_cli`、`run_cli_with`(コマンドごとの振り分け) |
| `args.rs` | Controller | 引数の解釈(`lexopt`、オプションの表 `OPTIONS`、コマンドごとの文法) |
| `types.rs` | Model | `CommandKind`(8 種)、`CliCommand`、`GraphFormat`、`InitTarget`、`UsageErrorKind`(8 種)など |
| `ctx.rs` | Service | `Ctx`、`Checked`、`Session`、実行の前の検査(定義 → command → ジョブID → run ID → `.env` → ステート)、ステートの書き込み・初期化の失敗の表示 |
| `step.rs` | Service | 1 つのジョブの手順(`record_skip`、`run_step`、`run_job_with_retry`、`remove_if_requested`) |
| `run.rs`、`pipeline.rs` | Service | 個別実行と一括実行の流れ |
| `dry_run.rs` | Service | `--dry-run` の流れ |
| `aux_commands.rs` | Service | `watch`、`reset`、`validate`、`convert`、`graph`、`init` |
| `run_id.rs` | Service | run ID の解決 |
| `deps.rs` | Configuration | 差し替え口の trait と本番の実装 |
| `exit.rs` | Utility | 終了コードの定数と、0〜255 への合わせ |
| `panic_hook.rs` | Middleware | パニックのとき固定の文と 70 |
| `platform/` | Configuration/OS | シグナルの監視、色が使える OS か |
| `report/mod.rs` | Controller(表示) | `DisplayEvent`(33 種)、`Reporter`(言語・色の遅延判定)、`escape_name`/`escape_full` |
| `report/messages.rs` | Static | 英日の文言の表 |
| `report/watch.rs`、`graph_text.rs`、`graph_mermaid.rs`、`plan_text.rs`、`init_template.rs` | Utility(整形) | 表、図(text、Mermaid)、計画、ひな形の整形(純粋) |
| `bin/conductor-cli-test-helper.rs` | Test | テスト専用の補助バイナリ(配布しない) |
| `tests/`(`src/tests/`) | Test | クレートの内部を確かめる単体テスト(文法、文言、図、計画、ひな形、README との一致) |

### crates/converter/src(conductor-converter)

| パス | 分類 | 内容 |
|------|------|------|
| `lib.rs` | 構成 | `convert`、`convert_text`。`#![forbid(unsafe_code)]` |
| `types.rs` | Model | `ConversionRequest`、`ConvertedText`、`TargetFormat`、内部の `Value` |
| `error.rs` | Utility | `ConversionError`、`ConversionErrorKind`(6 種) |
| `intake.rs` | Repository | 入力の読み取り(上限 1 MiB) |
| `raw.rs` | Model | 展開前の木(アリーナ) |
| `yaml_read.rs`、`toml_read.rs` | Service | 読み取り(YAML は明示のスタック) |
| `scalar.rs` | Utility | YAML 1.2 のコアスキーマ |
| `resolve.rs` | Service | アンカー・エイリアス・マージキーの展開と上限 |
| `yaml_write.rs`、`toml_write.rs` | Service | 書き出し |
| `tests/`(`src/tests/`) | Test | 往復、読み取りと解決、安全性の単体テスト |

## テストの配置

| 場所 | 内容 |
|------|------|
| `crates/core/tests/`(33 ファイル) | 定義(正常・誤り・retry・性能)、`.env`(解析、統合、型、性能)、流れ、図(辺、形、配置の参照出力との一致 `graph_layout_oracle.rs`、性能)、計画、NFR(安全性、性能)、ライブラリとしての使い方 |
| `crates/runtime/tests/`(14 ファイル) | 起動(基本、シグナル)、ステート(読み、書き、run ID、流れ)、`.env` の読み取り、ファイルの作成、待機、NFR(内容、性能、頑健さ、ソースの検査)、TOML の深さ |
| `crates/cli/tests/`(25 ファイル) | 流れのテスト(`flow_*`。偽の `Runtime`/`Sink`/`Env`/`Signals` を注入)、実バイナリのテスト(`binary_*`。テスト専用の補助バイナリを使う)、シグナル、デモ(`unix_demo.rs`、`windows_demo.rs`)、NFR(性能、頑健さ、ソースの検査)、MSRV と README の一致 |
| `crates/converter/tests/`(2 ファイル) | 受け入れ(`convert_ac.rs`)、安全性(`nfr_security.rs`) |
| 各モジュールの `#[cfg(test)] mod tests`、`crates/cli/src/tests/`、`crates/converter/src/tests/` | 単体テスト |

## コードの型(パターン)

- **純粋な中核と入出力の殻**: 判断・記録・図・計画は core の純粋な関数。入出力は runtime と cli の `deps` に限る。
- **差し替え口(依存の注入)**: `Deps { runtime, sink, env, signals }` を `run_cli_with` に渡す。テストは偽の実装で失敗を起こす。
- **すべての誤りを集める検証**: `Walker` が誤りを位置つきで集め、位置の順に返す。
- **固定の文のエラー**: 種類ごとの `label()`/`reason()` の 1 か所に文を持ち、入力の値を含めない。
- **網羅の配列**: `CommandKind::ALL` などで、追加漏れをテストで捕まえる。オプションの表と列挙の並びは `const` の検査で結ぶ。
- **上限 + 1 バイトの読み取り**: 定義、`.env`、ステート、変換の入力で共通(1 MiB)。
- **原子的な置き換え**: 一時ファイル → 同期 → `rename`(ステート、`init --force`)。
- **英日の文言の対**: `messages.rs` の定数を `Language` で引く。トークン(`success` など)は英語のまま。
- **文書の規則番号**: `//!`・`///` に BR/NFR/FR の番号を書き、設計の記録と対応させる。
