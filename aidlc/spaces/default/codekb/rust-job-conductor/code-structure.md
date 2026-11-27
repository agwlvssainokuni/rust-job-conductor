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
├── demo/                   デモ(UNIX の sh と Windows の cmd)。定義(conductor、retry、env、log)、.env、スクリプト
└── docs/spec/              再構築用の仕様(7 版 × 13 文書)
```

`target/`(ビルド出力)と `local-docs/`(個人の参照文書)は `.gitignore` で除かれる。`.idea/`(IDE の設定)は追跡されているが、この走査では開いていない。

## 仕様書の配置(`docs/spec/`)

| ディレクトリ | 内容 | 版の記述 |
|--------------|------|----------|
| `rust/` | Rust 版の構成(クレート、公開の口、モジュール)と挙動。00〜12 の 13 文書(計 6,638 行) | v0.10.1 と書くが、v0.10.0 の機能を書いていない(`code-quality-assessment.md`) |
| `language-agnostic/` | 言語に依らない挙動と受け入れ基準(AC 番号)。13 文書(計 7,634 行) | v0.10.1 までと書くが、v0.10.0 の機能を書いていない |
| `csharp/`、`go/`、`powershell/`、`python/`、`typescript/` | 各言語での再構築の案内。各 13 文書 | 冒頭は v0.8.0・v0.9.0 |

13 文書の割り振りは、00 案内、01 概要、02 アーキテクチャ、03 定義ファイル、04 コマンドライン、05 実行の意味論、06 ステート、07 プロセスとシグナル、08 出力と多言語化、09 変換、10 エラーと終了コード、11 品質とビルド、12 再構築の手引き。

## クレートごとのモジュール

分類は `code-analysis-guide.md` の分類(Model、Service、Repository、Utility、Configuration、Controller、Middleware、Static、Test)。各モジュールの責務と依存の詳細は `component-inventory.md` に 1 度だけ書く。

### crates/core/src(conductor-core)

| パス | 分類 | 内容 |
|------|------|------|
| `lib.rs` | 構成 | 公開の口(契約 C2)の再公開。`#![forbid(unsafe_code)]` |
| `types.rs` | Model | 共有の型(定義、実行結果、ステート、試行 `Attempt`、判断の結果) |
| `time.rs` | Utility | `Timestamp`、RFC 3339 と詰めた形の整形・読み取り、所要時間(v0.10.0) |
| `error.rs` | Utility | `DefinitionErrorKind`(34 種)、表示の escape、行番号の変換 |
| `run_id.rs` | Model/Utility | `RunId`、環境変数の名前の規則、`peek_run_id_env` |
| `consistency.rs` | Service | ステートと定義の照合 |
| `definition/mod.rs`、`intake.rs`、`parse.rs`、`depth.rs`、`version.rs`、`duration.rs` | Service/Repository | 定義の読み込みの段(`intake` だけがファイルに触れる) |
| `definition/walk/mod.rs`、`retry.rs`、`env.rs` | Service | キー・型・構造の検査と組み立て(`log_dir`、`unset_env`、`[pipeline.retry]`、`env_file` のテーブルを含む) |
| `envfile/mod.rs`、`parse.rs`、`merge.rs` | Model/Service | `.env` の型(`EnvFileErrorKind` 14 種)、解析、層の統合、`unset_env` の重なりの検査 |
| `flow/mod.rs` | Service | 判断の再公開と共通の補助(`record_of`、`record_at`、`status_at`、`position_of`、`SimulationMarks`。後ろの 3 つは v0.10.1) |
| `flow/classify.rs`、`decide.rs`、`next_step.rs`、`resume.rs`、`retry.rs` | Service | 分類、起動前の判断(公開の `decide_from_state` と内部の `decide_at`)、次の動き(公開の `classify_and_decide` と内部の `classify_and_decide_at`)、再開位置、リトライの判断 |
| `flow/record.rs` | Service | `initial_state`、`apply_*` と内部の `apply_outcome_in`・`apply_skip_in`(本番と模擬が共有) |
| `flow/attempt.rs` | Service | `start_attempt`、`end_attempt`、`cancel_attempt`(v0.10.0) |
| `flow/activity.rs` | Service | `pipeline_activity`(`watch --follow` の終わりの判定。v0.10.0) |
| `graph/mod.rs`、`edges.rs`、`overlay.rs`、`layout.rs` | Service/Model | 図の構造、辺、状態の重ね合わせ、レーンの配置 |
| `plan/mod.rs`、`simulate.rs` | Model/Service | `--dry-run` の計画の型と模擬(位置と目印を持ち回る。v0.10.1) |
| `test_support.rs` | Test | 単体テスト用の補助(`#[cfg(test)]`) |

### crates/runtime/src(conductor-runtime)

| パス | 分類 | 内容 |
|------|------|------|
| `lib.rs` | 構成 | 公開の口(契約 C3)の再公開。`#![deny(unsafe_code)]` |
| `types.rs` | Model | `CommandLine`、`InterruptSignal`、`StateLocation`、`Durability` |
| `error.rs` | Utility | `CommandError`、`StateError` と固定の理由の文 |
| `split.rs` | Service | `check_command`(command の分割の状態機械) |
| `runner.rs` | Service | `run_job`(直接起動、`try_wait` のくり返し、シグナルの転送、ログの複製のスレッド) |
| `log.rs` | Repository/Utility | `LogTarget`、`LogOutcome`、`safe_log_stem`、`open_log_file`(v0.10.0) |
| `clock.rs` | Utility | `now`(v0.10.0) |
| `wait.rs` | Service | `wait_for` |
| `envfile.rs` | Repository | `load_env_files` |
| `bounded_read.rs` | Utility | 上限 + 1 バイトまでの読み取り |
| `temp_file.rs` | Utility | 一時ファイルと `replace_via_temp` |
| `file_create.rs` | Repository | `create_file`(`init`) |
| `state/reader.rs`、`writer.rs`、`reset_all.rs` | Repository | ステートの読み書き(`[[job.attempt]]` を含む)と削除 |
| `platform/mod.rs`、`unix.rs`、`windows.rs` | Configuration/OS | プロセスグループ、シグナルの送信、ディレクトリの同期、0600 |
| `bin/conductor-test-helper.rs` | Test | テスト専用の補助バイナリ(配布しない) |

### crates/cli/src(conductor-cli)

| パス | 分類 | 内容 |
|------|------|------|
| `main.rs` | Controller | パニックのフックを設定し、`run_cli` の結果で 1 回だけ終了 |
| `lib.rs` | Controller | `run_cli`、`run_cli_with`(コマンドごとの振り分け) |
| `args.rs` | Controller | 引数の解釈(`lexopt`、オプションの表 `OPTIONS`) |
| `types.rs` | Model | `CommandKind`(8 種)、`CliCommand`(`follow`、`interval`)、`UsageErrorKind`(8 種)など |
| `ctx.rs` | Service | `Ctx`、`Checked`、`Session`、実行の前の検査、書き込み・初期化の失敗の表示 |
| `step.rs` | Service | 1 つのジョブの手順(試行の時刻、ログの指定、リトライ) |
| `run.rs`、`pipeline.rs`、`dry_run.rs` | Service | 個別実行、一括実行、`--dry-run` |
| `aux_commands.rs` | Service | `watch`(`follow_watch`)、`reset`、`validate`、`convert`、`graph`、`init` |
| `run_id.rs` | Service | run ID の解決 |
| `deps.rs` | Configuration | 差し替え口の trait と本番の実装 |
| `exit.rs` | Utility | 終了コードの定数と、0〜255 への合わせ |
| `panic_hook.rs` | Middleware | パニックのとき固定の文と 70 |
| `platform/mod.rs`、`unix.rs`、`windows.rs` | Configuration/OS | シグナルの監視、色と画面の消去が使える OS か |
| `report/mod.rs` | Controller(表示) | `DisplayEvent`(37 種)、`Reporter`、`FramePrefix`、escape |
| `report/messages.rs` | Static | 英日の文言の表(使い方の全文を含む) |
| `report/watch.rs`、`graph_text.rs`、`graph_mermaid.rs`、`graph_dot.rs`、`plan_text.rs`、`init_template.rs` | Utility(整形) | 表(4 列)、図(text、Mermaid、DOT)、計画、ひな形の整形(純粋) |
| `bin/conductor-cli-test-helper.rs` | Test | テスト専用の補助バイナリ(配布しない) |
| `tests/`(`src/tests/`) | Test | 内部の単体テスト(文法、文言、図、DOT、計画、ひな形、README との一致、終了コード)。`common/` に補助 |

### crates/converter/src(conductor-converter)

| パス | 分類 | 内容 |
|------|------|------|
| `lib.rs` | 構成 | `convert`、`convert_text`。`#![forbid(unsafe_code)]` |
| `types.rs` | Model | `ConversionRequest`、`ConvertedText`、`TargetFormat`、内部の `Value` |
| `error.rs` | Utility | `ConversionError`、`ConversionErrorKind`(6 種) |
| `intake.rs` | Repository | 入力の読み取り(上限 1 MiB) |
| `raw.rs` | Model | 展開前の木 |
| `yaml_read.rs`、`toml_read.rs`、`scalar.rs`、`resolve.rs` | Service/Utility | 読み取り、YAML のスカラー、アンカーとエイリアスの展開 |
| `yaml_write.rs`、`toml_write.rs` | Service | 書き出し |
| `tests/`(`src/tests/`) | Test | 往復、読み取りと解決、安全性 |

## テストの配置

| 場所 | 内容 |
|------|------|
| `crates/core/tests/`(36 ファイル。補助 `common/`、`graph_common/`、`fixtures/`) | 定義、`.env`、流れ、試行(`attempt_flow.rs`)、動きの様子(`pipeline_activity.rs`)、時刻(`time_format.rs`)、図(参照出力との一致 `graph_layout_oracle.rs` を含む)、計画、性能(`*_perf.rs`)、NFR |
| `crates/runtime/tests/`(16 ファイル。補助 `common/`) | 起動、シグナル、ログ(`log_file.rs`、`runner_log.rs`)、ステート、`.env`、ファイルの作成、待機、NFR(内容、性能、頑健さ、ソース) |
| `crates/cli/tests/`(26 ファイル。補助 `common/`、`bin_support/`、`signal_support/`、`fixtures/`) | 流れのテスト(`flow_*.rs`)、実バイナリのテスト(`binary_*.rs`。`binary_log.rs` を含む)、シグナル、デモ、NFR、MSRV と README の一致。手順書 `WINDOWS_CTRL_C_CHECKLIST.md` |
| `crates/converter/tests/`(2 ファイル。`fixtures/`) | 受け入れ、安全性 |
| 各モジュールの `#[cfg(test)] mod tests`、`crates/cli/src/tests/`(12 ファイル)、`crates/converter/src/tests/`(4 ファイル) | 単体テスト |

テストの方式と数(数え方を含む)は `code-quality-assessment.md` を参照。

## コードの型(パターン)

- **純粋な中核と入出力の殻**: 判断・記録・時刻・図・計画は core の純粋な関数。入出力は runtime と cli の `deps` に限る。現在の時刻も `Runtime::now` で注入する。
- **差し替え口(依存の注入)**: `Deps { runtime, sink, env, signals }` を `run_cli_with` に渡す。
- **公開の入口と内部の入口**: 公開の関数は内部の入口(`*_at`、`*_in`)を、作り直した目印で呼ぶ薄い殻。テストのための入口を本番に足さない方針(`project.md`)に沿い、内部の入口は `pub(crate)` で、性能のために計画の模擬が使う。
- **すべての誤りを集める検証**: `Walker` が誤りを位置つきで集め、位置の順に返す。
- **固定の文のエラー**: 種類ごとの `label()`/`reason()` の 1 か所に文を持ち、入力の値を含めない。
- **網羅の配列**: `CommandKind::ALL` などで、追加漏れをテストで捕まえる。
- **上限 + 1 バイトの読み取り**: 定義、`.env`、ステート、変換の入力で共通(1 MiB)。
- **原子的な置き換え**: 一時ファイル → 同期 → `rename`(ステート、`init --force`)。ログのファイルだけは置き換えず `create_new`。
- **英日の文言の対**: `messages.rs` の定数を `Language` で引く。
- **文書の規則番号**: `//!`・`///` に BR/NFR/FR の番号と、v0.10.0 の Unit(U1〜U7)、v0.10.1 の BR・FR を書き、設計の記録と対応させる。
