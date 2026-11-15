# Code Structure

## ディレクトリ構成
| パス | 区分 | 内容 |
|------|------|------|
| `Cargo.toml`、`Cargo.lock` | 設定 | ワークスペース(`members = ["crates/*"]`) |
| `crates/cli/` | コード | `src/`(args、ctx、run、pipeline、aux_commands、deps、exit、report/{mod,messages,watch,escape}、platform/{mod,unix,windows}、panic_hook、types、main)、`src/bin/conductor-cli-test-helper.rs`、`tests/`(16 ファイル、補助は `bin_support/`・`common/`) |
| `crates/core/` | コード | `src/definition/`(intake、parse、depth、version、walk)、`src/flow/`(classify、decide、next_step、resume)、error、types。`tests/`(6 ファイル、`fixtures/`) |
| `crates/runtime/` | コード | `src/`(runner、split、types、error)、`state/`(reader、writer、consistency、record)、`platform/{unix,windows}`、`bin/conductor-test-helper.rs`。`tests/`(10 ファイル) |
| `crates/converter/` | コード | `src/`(intake、raw、resolve、scalar、toml_read/write、yaml_read/write、types、error)、`tests/`(4 ファイル、`fixtures/`) |
| `demo/` | デモ | UNIX 版: `run-demo.sh`、`conductor.toml`、`scripts/{extract,transform,load,notify}.sh`。Windows 版: `windows/run-demo.cmd`、`windows/conductor.toml`、`windows/scripts/*.cmd`。`README.md`(Windows 版の節あり) |
| `.github/workflows/` | CI | `ci.yml`、`release.yml`、`audit.yml`(今回は未走査。前回の記録) |
| `README.md`、`LICENSE` | 文書 | 日本語の利用説明、Apache-2.0 |

## 規約とパターン
- 各ファイルの冒頭に Apache-2.0 のヘッダ(Rust は `//`、sh・YAML は `#`)。コメントと説明は日本語。
- クレートの `lib.rs` に、公開関数・モジュールの対応を表で書く。`unsafe` は runtime の UNIX 系 `kill` だけ(cli は `forbid`)。
- 単体テストは各 `src/*.rs` の `mod tests` に、統合テストは `crates/*/tests/` に置き、`common/` に補助を置く。
- 構造体の初期化は、ソース内の単体テストと統合テストでも、`PipelineDefinition { name, description, format_version, jobs }` をフィールド名で直接書く(`crates/core/src/{types,flow/*}.rs`、`crates/runtime/src/state/{consistency,record}.rs`、`crates/runtime/tests/*`)。

## CLI のコマンドの追加点(cli の構造)
1. `types.rs`: `CommandKind`(5 値)と `name()`、`CliCommand`(`kind`、`job_id`、`pipeline_name`、`convert_input`、`convert_target`、`config_path`、`verbose`)。
2. `args.rs`: `parse_args` が字句に分け、`build_command` がコマンド名(`"run"` など)を判定し、オプションの許可表と位置引数の数(1 つ必須か、0 個か)を検査する。
3. `lib.rs`: `run_cli_with` の `match cmd.kind` が、`run::run_individual`、`pipeline::run_pipeline`、`aux_commands::run_{watch,reset,convert}` に振り分ける。
4. `ctx.rs`: `Ctx`(`runtime`、`signals`、`sink`、`reporter`)と `Ctx::prepare`(実行前の検査。architecture.md を参照)、`definition_path`(`--config` か `conductor.toml`)。
5. `report/`: `DisplayEvent`(水準は `Error`・`Verbose`・`Output`)と、英語・日本語の文言(`messages.rs`。使い方の `USAGE` を含む)。

## `PipelineDefinition.jobs` の使われ方
`crates/core/src/types.rs` の定義(`jobs: Vec<JobDefinition>`、`job_index`)を、core の flow(`next_step`・`decide`・`resume`)、runtime の state(`consistency`・`record`)、cli(`ctx.rs`・`run.rs`・`pipeline.rs`)が読む。構築するのは `crates/core/src/definition/walk.rs` の 1 か所だけ。詳細な行の一覧は、この intent の `developer-scan.md` を参照。ステートのキー(`job`)と定義ファイルのキー(`job`)は、すでに揃っている。

## デモの構造
- `demo/run-demo.sh`: `mktemp -d` の一時ディレクトリに `conductor.toml` と `scripts/` をコピーし、6 つの場面を順に実行する。環境変数: `CONDUCTOR`、`DEMO_LANG`、`DEMO_PAUSE`、`DEMO_<ジョブ>_EXIT`、`DEMO_SLEEP`。
- `demo/conductor.toml`: パイプライン `nightly`。ジョブは extract(警告=終了コード 1 で notify へ遷移)、transform、load、notify(配列形式の `command`)。
- `demo/scripts/*.sh`: 各ジョブは、メッセージを出し、`sleep` し、`DEMO_<NAME>_EXIT` の値で終了する。
- Windows 版(`demo/windows/`): `run-demo.cmd` が `%TEMP%` の一時ディレクトリで同じ場面 0〜6 を実行する。`conductor.toml` は同じ `nightly`(`command` は `'scripts\extract.cmd'` のような文字列と、notify の配列 `["cmd", "/c", "scripts\\notify.cmd"]`)。`DEMO_SLEEP` は整数の秒(既定 1)、`DEMO_LANG` は `en_US` 形式(内部で `LANG` に設定)。実行中だけコードページを UTF-8(65001)にする。デモ自体は `validate` を使わない。
- `crates/cli/tests/windows_demo.rs`(`cfg(windows)` のみ)が、`demo/windows` を一時ディレクトリへ写して、場面 1〜6 の終了コードを確かめる。
