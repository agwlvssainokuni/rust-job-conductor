# Code Structure

## ワークスペースの構成

```
Cargo.toml                 ワークスペース(members = crates/*、edition 2024、rust-version 1.100)
crates/
  core/                    conductor-core (lib)
    src/
      lib.rs, types.rs, error.rs, run_id.rs
      definition/  mod.rs intake.rs parse.rs depth.rs version.rs walk.rs
      flow/        mod.rs classify.rs decide.rs next_step.rs resume.rs
    tests/         definition_valid / definition_errors / flow_sequences / library_capability /
                   nfr_security / nfr_performance + fixtures/(valid_pipeline.toml, multiple_errors.toml)
  runtime/                 conductor-runtime (lib + bin conductor-test-helper)
    src/
      lib.rs, types.rs, error.rs, runner.rs, split.rs
      platform/    mod.rs unix.rs windows.rs
      state/       mod.rs reader.rs writer.rs record.rs consistency.rs reset_all.rs
      bin/         conductor-test-helper.rs
    tests/         common/mod.rs + runner_basic / runner_signal / state_* / flow_with_state / run_id_state / toml_depth / nfr_*
  converter/               conductor-converter (lib)
    src/           lib.rs types.rs raw.rs intake.rs yaml_read.rs toml_read.rs resolve.rs scalar.rs yaml_write.rs toml_write.rs error.rs
    tests/         common/mod.rs + convert_ac / convert_roundtrip / read_resolve / nfr_security + fixtures/
  cli/                     conductor-cli (lib conductor_cli + bin conductor + bin conductor-cli-test-helper)
    src/
      main.rs, lib.rs, types.rs, args.rs, ctx.rs, run.rs, pipeline.rs, aux_commands.rs,
      deps.rs, exit.rs, panic_hook.rs
      platform/    mod.rs unix.rs windows.rs
      report/      mod.rs messages.rs watch.rs escape.rs
      bin/         conductor-cli-test-helper.rs
    tests/         common/mod.rs, bin_support/mod.rs + flow_* / binary_* / signals_* / report_messages / args_grammar / exit_codes / ... 
demo/                      デモ(conductor.toml、scripts/、windows/、run-demo.sh)
.github/workflows/         ci.yml release.yml audit.yml
README.md                  利用者向けの文書(日本語)
```

行数の目安(src のみ、`#[cfg(test)]` を含む): core 約 3,300、runtime 約 3,400(bin 含む)、converter 約 3,100、cli は `run.rs` 140、`report/mod.rs` 495、`report/messages.rs` 347 など。ワークスペース全体(tests を含む)で約 2 万行。

## 流れの層とファイルの対応

| 層 | ファイル | 役割 |
|----|----------|------|
| 入口 | `cli/src/main.rs` | パニックのフックを設定し、`run_cli` の結果で 1 回だけ `process::exit` |
| コマンド分岐 | `cli/src/lib.rs` | `run_cli`(本番の依存)/ `run_cli_with`(差し替え)。`CommandKind` で分岐 |
| 引数 | `cli/src/args.rs`, `types.rs` | `lexopt` で字句 → コマンドごとの文法の表。`CliCommand`、`UsageError` |
| 共通の前処理 | `cli/src/ctx.rs` | `Ctx::prepare`(検査の順序を固定)、`check_definition`、`report_execution`、`write`、`reset` |
| 個別実行 | `cli/src/run.rs` | `run_individual`(RunMode::Individual) |
| 一括実行 | `cli/src/pipeline.rs` | `run_pipeline`(RunMode::Pipeline)、`finish` |
| 補助 | `cli/src/aux_commands.rs` | `run_watch`、`run_reset`、`run_validate`、`run_convert` |
| 差し替え口 | `cli/src/deps.rs` | `Runtime`・`Sink`・`Env`・`Signals` と本番の実装 |
| 表示 | `cli/src/report/` | `DisplayEvent` + `Reporter`(言語・色は遅延)、文言の表 `Text{en, ja}`、`watch` の表、エスケープ |
| 終了コード | `cli/src/exit.rs` | `EXIT_*` 定数、`exit_code_for`、`signal_exit_code` |
| 定義 | `core/src/definition/` | Intake → Parse → Depth → VersionCheck → Walk |
| 判断 | `core/src/flow/` | `decide_before_run`、`classify`、`classify_and_decide`、`resume_point_on_interrupt` |
| 起動・待ち | `runtime/src/runner.rs`, `split.rs`, `platform/` | `run_job`、`wait_loop`、`command_line`/`split`、OS 固有 |
| ステート | `runtime/src/state/` | `reader`(読む)、`writer`(原子的に書く・reset)、`record`(`apply_*`)、`consistency`、`reset_all` |
| 変換 | `converter/src/` | Intake → Read → RootCheck → Resolve → Write |

## ファイルの分類

| 種別 | ファイル |
|------|----------|
| モデル(データの形) | `core/src/types.rs`(`PipelineDefinition`、`JobDefinition`、`Command`、`Transition*`、`JobExecution`、`JobState`、`PipelineState`、`Decision*`、`Classification`、`JobOutcome`、`NextStep`、`RunMode`、`ResumePoint`)、`runtime/src/types.rs`、`converter/src/types.rs`、`cli/src/types.rs` |
| 構文・検証 | `core/src/definition/*`、`core/src/run_id.rs` |
| 純粋な判断 | `core/src/flow/*`、`runtime/src/state/{record,consistency}.rs`、`cli/src/exit.rs`、`cli/src/args.rs` |
| IO(ファイル) | `core/src/definition/intake.rs`、`runtime/src/state/{reader,writer,reset_all}.rs`、`converter/src/intake.rs` |
| IO(プロセス・OS) | `runtime/src/runner.rs`、`runtime/src/platform/*`、`cli/src/platform/*` |
| 表示 | `cli/src/report/*`、`cli/src/panic_hook.rs` |
| エラー型 | `core/src/error.rs`、`runtime/src/error.rs`、`converter/src/error.rs`(いずれも固定の文) |
| テスト専用 | `runtime/src/bin/conductor-test-helper.rs`、`cli/src/bin/conductor-cli-test-helper.rs`、`*/tests/common/`、`cli/tests/bin_support/` |

## コードのパターン

- **crate の公開面は lib.rs の `pub use` で絞る**。runtime の `platform`・`runner`・`split`・`state` は非公開モジュールで、必要な関数だけを再公開する(`check_command`、`run_job` など)。
- **「位置を保つ木 → 検査 → 型付きの値」**: core の `walk.rs` は `toml::de::DeTable`(`Spanned`)を走査し、`RawJob` に集めてから `build_jobs` で `JobDefinition` にする。誤りは `Vec<(offset, DefinitionError)>` に集め、位置の順に並べて返す(`complete` が偽の子は後続の検査を省略)。
- **固定の文だけでエラーを組み立てる**: 理由は `&'static str` か固定の文。入力由来はキー名だけで、`quote_key`/`sanitize` で制御文字をエスケープし 64 文字で切る。
- **純粋な関数は「新しい値を返す」形**: `apply_outcome(&PipelineState, ...) -> PipelineState`(`clone` して更新)。
- **trait による差し替え + 本番の実装の薄い包み**: `SystemRuntime` は core・runtime・converter の関数を 1 行ずつ呼ぶだけ。
- **let-chain(`if let ... && ...`)**: 入れ子の `if` は let-chain に整理済み(edition 2024、MSRV 1.100)。例: `ctx.rs` の `prepare`、`run.rs` の `Skip` の枝、`walk.rs` の `pipeline_section`。
- **OS 固有部は `platform/` に閉じる**: `cfg(unix)`・`cfg(windows)` は各 crate の `platform` の外に置かない(`nfr_source` のテストが検査)。
- **日本語の doc コメントに要件 ID を添える**(BR/NFR/FR/ADR)。テスト名も日本語。
- **契約の番号**: C1(共有の型)、C2(core の関数)、C3(runtime の関数)、C4(converter の関数)。

## 新しいジョブ単位の設定を足すときに触れる場所(一覧)

| # | 場所 | 内容 |
|---|------|------|
| 1 | `core/src/types.rs` の `JobDefinition`(`PartialEq, Eq` 導出) | フィールドを足す。浮動小数は `Eq` と相容れない |
| 2 | `core/src/definition/walk.rs` の `JOB_KEYS`・`RawJob`・`Walker::job`・`build_jobs`・`cross_check` | キーの許可、型の検査、関係の検査、組み立て |
| 3 | `core/src/error.rs` の `DefinitionErrorKind`(`ALL`・`code`・`#[error]`) | 新しい誤りの種類を足す場合 |
| 4 | `cli/src/report/messages.rs` の `definition_kind` | 誤りの種類ごとの英語・日本語の文 |
| 5 | `core/src/flow/*` | 分類・次の動き(必要な場合) |
| 6 | `runtime/src/runner.rs`、`cli/src/run.rs`、`cli/src/pipeline.rs` | 実行の流れ |
| 7 | `runtime/src/state/{reader,writer,record}.rs` | ステートに記録する場合のみ(`TOP_KEYS`・`JOB_KEYS`、`FORMAT_VERSION`、`serialize`) |
| 8 | `cli/src/report/mod.rs`(`DisplayEvent`・`level`・`render`)、`report/messages.rs` | 表示の種類と文言 |
| 9 | `converter/src/*` | 通常は変更不要(スキーマ非依存) |
| 10 | テストの組み立て | `JobDefinition { ... }` のリテラルを持つ箇所(`core/src/types.rs` のテスト、`runtime/tests/common/mod.rs` の `job()`、各テスト)、定義の組み立て補助(`cli/tests/bin_support::job_block`、`cli/tests/common` の定義文字列) |
| 11 | `README.md`、`demo/` | 定義の表・終了コードの表・用語。CI の `docs-check` が README の必須の見出しを検査する |

## 命名・構成の慣習

- 命名は Rust 標準(snake_case、PascalCase、SCREAMING_SNAKE_CASE)。ファイルは機能の単位で 1 ファイル 1 責務。
- ライセンスヘッダー(Apache-2.0、`Copyright 2026 agwlvssainokuni`)を全ソースの先頭に置く。
- `rustfmt`・`clippy` は既定の設定(設定ファイルなし)。CI で `-D warnings`。
