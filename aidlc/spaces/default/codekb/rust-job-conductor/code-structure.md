# Code Structure

## ディレクトリ構成
| パス | 区分 | 内容 |
|------|------|------|
| `Cargo.toml`、`Cargo.lock` | 設定 | ワークスペース(`members = ["crates/*"]`、版 0.4.0) |
| `crates/cli/` | コード | `src/`(args、ctx、run、pipeline、aux_commands、deps、exit、report/{mod,messages,watch,escape}、platform/{mod,unix,windows}、panic_hook、types、main)、`src/bin/conductor-cli-test-helper.rs`、`tests/`(17 ファイル、補助は `bin_support/`・`common/`) |
| `crates/core/` | コード | `src/definition/`(intake、parse、depth、version、walk)、`src/flow/`(classify、decide、next_step、resume)、error、types。`tests/`(6 ファイル、`fixtures/`) |
| `crates/runtime/` | コード | `src/`(runner、split、types、error)、`state/`(reader、writer、consistency、record)、`platform/{unix,windows}`、`bin/conductor-test-helper.rs`。`tests/`(10 ファイル) |
| `crates/converter/` | コード | `src/`(intake、raw、resolve、scalar、toml_read/write、yaml_read/write、types、error)、`tests/`(4 ファイル、`fixtures/`)。`[pipeline]` のキー名はコードに現れない(汎用の木) |
| `demo/` | デモ | UNIX 版: `run-demo.sh`、`conductor.toml`、`scripts/{extract,transform,load,notify}.sh`。Windows 版: `windows/run-demo.cmd`、`windows/conductor.toml`、`windows/scripts/*.cmd`。`README.md`(Windows 版の節あり)(今回は未走査。前回の記録) |
| `.github/workflows/` | CI | `ci.yml`、`release.yml`、`audit.yml`(今回は未走査。前回の記録) |
| `README.md`、`LICENSE` | 文書 | 日本語の利用説明、Apache-2.0 |

## 規約とパターン
- 各ファイルの冒頭に Apache-2.0 のヘッダ(Rust は `//`、sh・YAML は `#`)。コメントと説明は日本語。
- クレートの `lib.rs` に、公開関数・モジュールの対応を表で書く。`unsafe` は runtime の UNIX 系 `kill` だけ(cli は `forbid`)。
- 単体テストは各 `src/*.rs` の `mod tests` に、統合テストは `crates/*/tests/` に置き、`common/` に補助を置く。テスト名は日本語。
- 構造体の初期化は、ソース内の単体テストと統合テストでも、`PipelineDefinition { name, description, format_version, job }` と `PipelineState { definition_key, pipeline_name, definition_path, job_order, job_states, resume_job_id, end_reached, completed }` を、フィールド名で全部書く(`crates/core/src/{types,flow/*}.rs`、`crates/runtime/src/state/{consistency,record,reader}.rs`、`crates/runtime/tests/*`、`crates/cli/tests/*`)。フィールドを足すと、これらがすべて壊れる。

## CLI のコマンドの追加点(cli の構造)
1. `types.rs`: `CommandKind`(6 値。`Run`・`Pipeline`・`Watch`・`Reset`・`Validate`・`Convert`。`:26`)と `name()`、`CliCommand`(`:53`。`kind`、`job_id`、`pipeline_name`、`convert_input`、`convert_target`、`config_path`、`verbose`)と `CliCommand::new`(全フィールドを列挙)。
2. `args.rs`: `parse_args`(`:66`)が `lexopt` で字句(`Token`)に分け、`build_command`(`:154`)がコマンド名を判定し、オプションの許可表(`takes_verbose`・`takes_config`・`takes_to`)と位置引数の数を検査する。値を取るオプションは `Long("config") => match parser.value() {…}` の形(`:98`)。
3. `lib.rs`: `run_cli_with`(`:129`)の `match cmd.kind`(`:159`)が、`run::run_individual`、`pipeline::run_pipeline`、`aux_commands::run_{watch,reset,validate,convert}` に振り分ける。
4. `ctx.rs`: `Ctx`(`runtime`、`signals`、`sink`、`reporter`。`env` は持たない)、`Ctx::prepare`・`check_definition`(architecture.md を参照)、`definition_path`(`--config` か `conductor.toml`)。
5. `report/`: `DisplayEvent`(水準は `Error`・`Verbose`・`Output`)と、英語・日本語の文言(`messages.rs`。使い方の `USAGE` を含む)。

## ステート関係のコード(runtime)
- `types.rs`: `StateLocation`(`of`・`path`)、`MismatchKind`、`ConsistencyResult`、`Durability`、`InterruptSignal`、`CommandLine`。
- `state/reader.rs`: `read_state`(`:49`)→ `read_state_file`(`:54`)→ `parse_state`(`:100`)→ `walk`(`:176`)。`TOP_KEYS`(9 キー。`:128`)、`JOB_KEYS`(4 キー。`:140`)、上限 1 MiB、`FORMAT_VERSION = 1`。
- `state/writer.rs`: `write_state`(`:53`)→ `write_state_file`(`:61`)、`reset_state`(`:110`)、`temp_name`(`:124`)、`create_temp`(`:130`)、`remove_stale_temps`(`:162`)、`serialize`(`:231`)。
- `state/record.rs`: `initial_state`・`apply_outcome`・`apply_skip`・`apply_interrupt`(純粋。`PipelineState` を複製して変える)。
- `state/consistency.rs`: `check_consistency`(パイプライン名 → ジョブIDの集合 → 並び)。
- `error.rs`: `StateErrorKind`(6 種類)、`StateError { path, kind, reason }`、固定の理由の文(`reason`)。

## 定義の検証のコード(core)
- `definition/walk.rs`: `ROOT_KEYS`・`PIPELINE_KEYS`(`name`、`description`)・`JOB_KEYS`・`TRANSITION_KEYS` の許可表(`:32-39`)、`Walker::check_keys`(未知のキー → `unknownKey`)、`Walker::string`(文字列の取り出し → `typeMismatch`)、`pipeline_section`(`:202`)、`walk`(`:416`。`PipelineDefinition` を作るのは `:457` の 1 か所)。
- `error.rs`: `DefinitionErrorKind`(19 種類)、`quote_key`(キー名のエスケープと切り詰め)。

## デモの構造(今回は未走査。前回の記録)
- `demo/run-demo.sh`: `mktemp -d` の一時ディレクトリに `conductor.toml` と `scripts/` をコピーし、6 つの場面を順に実行する。環境変数: `CONDUCTOR`、`DEMO_LANG`、`DEMO_PAUSE`、`DEMO_<ジョブ>_EXIT`、`DEMO_SLEEP`。
- `demo/conductor.toml`: パイプライン `nightly`。ジョブは extract(警告=終了コード 1 で notify へ遷移)、transform、load、notify(配列形式の `command`)。
- `demo/scripts/*.sh`: 各ジョブは、メッセージを出し、`sleep` し、`DEMO_<NAME>_EXIT` の値で終了する。
- Windows 版(`demo/windows/`): `run-demo.cmd` が `%TEMP%` の一時ディレクトリで同じ場面 0〜6 を実行する。`conductor.toml` は同じ `nightly`(`command` は `'scripts\extract.cmd'` のような文字列と、notify の配列 `["cmd", "/c", "scripts\\notify.cmd"]`)。`DEMO_SLEEP` は整数の秒(既定 1)、`DEMO_LANG` は `en_US` 形式(内部で `LANG` に設定)。実行中だけコードページを UTF-8(65001)にする。デモ自体は `validate` を使わない。
- `crates/cli/tests/windows_demo.rs`(`cfg(windows)` のみ)が、`demo/windows` を一時ディレクトリへ写して、場面 1〜6 の終了コードを確かめる。
