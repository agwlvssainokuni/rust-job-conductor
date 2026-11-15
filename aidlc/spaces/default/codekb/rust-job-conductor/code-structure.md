# Code Structure

## ディレクトリ構成
| パス | 区分 | 内容 |
|------|------|------|
| `Cargo.toml`、`Cargo.lock` | 設定 | ワークスペース(`members = ["crates/*"]`)。各クレートの版は 0.5.0(`crates/*/Cargo.toml`) |
| `crates/cli/` | コード | `src/`(args、ctx、run、pipeline、aux_commands、deps、exit、report/{mod,messages,watch,escape}、platform/{mod,unix,windows}、panic_hook、types、main)、`src/bin/conductor-cli-test-helper.rs`、`tests/`(19 ファイル。run ID 関連は `flow_run_id.rs`・`binary_run_id.rs`。補助は `bin_support/`・`common/`) |
| `crates/core/` | コード | `src/definition/`(intake、parse、depth、version、walk)、`src/flow/`(classify、decide、next_step、resume)、`run_id.rs`(`RunId`、環境変数名の検査)、error、types。`tests/`(6 ファイル、`fixtures/`) |
| `crates/runtime/` | コード | `src/`(runner、split、types、error)、`state/`(reader、writer、consistency、record、`reset_all`)、`platform/{unix,windows}`、`bin/conductor-test-helper.rs`。`tests/`(`run_id_state.rs` を含む 11 ファイル) |
| `crates/converter/` | コード | `src/`(intake、raw、resolve、scalar、toml_read/write、yaml_read/write、types、error)、`tests/`(4 ファイル、`fixtures/`)。`[pipeline]` のキー名はコードに現れない(汎用の木)。今回は概観のみ |
| `demo/` | デモ | UNIX 版: `run-demo.sh`、`conductor.toml`、`scripts/{extract,transform,load,notify}.sh`。Windows 版: `windows/run-demo.cmd`、`windows/conductor.toml`、`windows/scripts/*.cmd`。`README.md`(Windows 版の節あり)。構造は下の「デモの構造」 |
| `.github/workflows/` | CI | `ci.yml`、`release.yml`、`audit.yml`(今回は未走査。前回の記録) |
| `README.md`、`LICENSE` | 文書 | 日本語の利用説明(run ID の節を含む)、Apache-2.0 |

## 規約とパターン
- 各ファイルの冒頭に Apache-2.0 のヘッダ(Rust は `//`、sh・YAML・TOML は `#`、cmd は `REM`)。コメントと説明は日本語。
- クレートの `lib.rs` に、公開関数・モジュールの対応を表で書く。`unsafe` は runtime の UNIX 系 `kill` だけ(cli は `forbid`)。
- 単体テストは各 `src/*.rs` の `mod tests` に、統合テストは `crates/*/tests/` に置き、`common/` に補助を置く。テスト名は日本語。
- 構造体の初期化は、ソース内の単体テストと統合テストでも、`PipelineDefinition { name, description, run_id_env, format_version, job }` と `PipelineState { definition_key, pipeline_name, definition_path, job_order, job_states, resume_job_id, end_reached, completed, run_id }` を、フィールド名で全部書く(`crates/core/src/{types,flow/*}.rs`、`crates/runtime/src/state/*`、`crates/runtime/tests/*`、`crates/cli/tests/*`)。フィールドを足すと、これらがすべて壊れる(run ID の追加で一度起きた)。

## CLI のコマンドの追加点(cli の構造)
1. `types.rs`: `CommandKind`(6 値。`Run`・`Pipeline`・`Watch`・`Reset`・`Validate`・`Convert`)と `name()`、`CliCommand`(`:53`。`kind`、`job_id`、`pipeline_name`、`convert_input`、`convert_target`、`config_path`、`verbose`、`run_id: Option<String>`(`:69`)、`all: bool`(`:71`))と `CliCommand::new`。
2. `args.rs`: `parse_args`(`:74`)が `lexopt` で字句(`Token`。`RunId`・`All` を含む)に分け、`build_command`(`:171`)がコマンド名を判定し、オプションの許可表と位置引数の数を検査する。`--run-id` は `run`・`pipeline`・`watch`・`reset` だけ、`--all` は `reset` だけ(それ以外は `OptionNotAllowed`)。`--all` と `--run-id` の併用、重複は `InvalidValue`。`--run-id` の値は `parse_run_id_option` で検査(空は有効=run ID なしに固定)。`resolve_run_id`(`:327`)が、オプションと環境変数から run ID を決める。
3. `lib.rs`: `run_cli_with` の `match cmd.kind`(`:160`)が、`run::run_individual`、`pipeline::run_pipeline`、`aux_commands::run_{watch,reset,validate,convert}` に振り分ける。
4. `ctx.rs`: `Ctx`(`runtime`、`signals`、`sink`、`reporter`、`env`)、`Ctx::prepare`・`check_definition`(architecture.md を参照)、`definition_path`(`--config` か `conductor.toml`。`:65`)、`aux_run_id`(`watch`・`reset` の run ID。`:303`)。
5. `report/`: `DisplayEvent`(水準は `Error`・`Verbose`・`Output`。`ResetAllKept`・`ConsistencyMismatch { with_run_id }` を含む)と、英語・日本語の文言(`messages.rs`。使い方の `USAGE`、`RESET_HINT_RUN_ID`、`RESET_ALL_KEPT`、`WATCH_RUN_ID`)。`watch.rs` の `render_table` が `Run ID: <ID>` の行を出す(run ID があるときだけ)。

## ステート関係のコード(runtime)
- `types.rs`: `StateLocation`(`of(definition_path, Option<&RunId>)`。`:114`)、`MismatchKind`、`ConsistencyResult`、`Durability`、`InterruptSignal`、`CommandLine`。
- `state/reader.rs`: `read_state(path, Option<&RunId>)` → `read_state_file` → `parse_state_for` → `walk`。`TOP_KEYS`(10 キー。`run_id` を含む。`:176-190` 付近)、`JOB_KEYS`(4 キー)、上限 1 MiB、`FORMAT_VERSION = 1`。`inspect_run_state`(`reset --all` 用。`Ours`・`NotOurs`・`Broken` を返す)。
- `state/writer.rs`: `write_state(path, run_id, state, Durability)`、`reset_state(path, run_id)`、`temp_name`、`create_temp`、`remove_stale_temps`、`serialize`(`run_id` を、あるときだけ書く。`:278`)。
- `state/reset_all.rs`: `reset_all_states`、`ResetAllReport { removed, kept }`、`run_id_in_name`。
- `state/record.rs`: `initial_state(definition, path, run_id)`・`apply_outcome`・`apply_skip`・`apply_interrupt`(純粋)。
- `state/consistency.rs`: `check_consistency`(パイプライン名 → ジョブIDの集合 → 並び)。
- `error.rs`: `StateErrorKind`(6 種類)、`StateError { path, kind, reason }`、固定の理由の文(`reason`。`RUN_ID_MISMATCH` を含む)。

## 定義の検証のコード(core)
- `definition/walk.rs`: `ROOT_KEYS`・`PIPELINE_KEYS`(`name`、`description`、`run_id_env`。`:34`)・`JOB_KEYS`・`TRANSITION_KEYS` の許可表、`Walker::check_keys`(未知のキー → `unknownKey`)、`Walker::string`(文字列の取り出し → `typeMismatch`)、`pipeline_section`(`run_id_env` の取り出しと形の検査。`:237-262`)。
- `run_id.rs`: `RunId::parse`、`is_valid_env_name`、`peek_run_id_env`(定義の文字列から `run_id_env` だけをゆるく取り出す)、`MAX_RUN_ID_LEN`(64)、`DEFAULT_RUN_ID_ENV`(`CONDUCTOR_RUN_ID`)。
- `error.rs`: `DefinitionErrorKind`、`quote_key`。

## デモの構造(今回、走査した)
詳細な行番号は、この intent の `developer-scan.md`(Handoff Summary)を参照。

- `demo/run-demo.sh`(POSIX sh、`set -u`): `mktemp -d` の一時ディレクトリ(`trap … EXIT` で消す)に `conductor.toml` と `scripts/` を写し、場面 0〜6 を順に実行する。補助関数: `scene`(太字の見出し)、`cmd`(シアンで `$ <コマンド>` を表示 → 実行 → 黄色で `→ 終了コード N`)、`pause`。環境変数: `CONDUCTOR`(既定は `target/release/conductor`、無ければ `cargo build`)、`DEMO_LANG`(既定 `ja_JP.UTF-8`、`LANG` に設定。`LC_ALL` は消さない)、`DEMO_PAUSE`、`DEMO_<JOB>_EXIT`、`DEMO_SLEEP`(既定 0.5)。1 コマンドだけの環境変数は、サブシェル `( VAR=値; export VAR; cmd … )`。
- 場面: 0 定義の表示、1 全部成功、2 警告で分岐(`DEMO_EXTRACT_EXIT=1`)、3 エラーで停止(`DEMO_LOAD_EXIT=2`)と `watch`、4 再開と、完了後の `watch`(空)、5 `run -v extract` と `reset`、6 `convert … --to yaml`。`validate`・`--run-id`・`reset --all` は使わない。
- `demo/conductor.toml`: パイプライン `nightly`、`run_id_env` なし。ジョブは extract(`sh scripts/extract.sh`、警告=終了コード 1 で notify へ遷移)、transform、load、notify(配列形式の `command`)。`scripts/*.sh` は、`[<job>] 実行中...` を出し、`sleep`、`DEMO_<NAME>_EXIT` の値で終了する。
- Windows 版(`demo/windows/`): `run-demo.cmd`(UTF-8、CRLF)が `%TEMP%\conductor-demo-<乱数>` で同じ場面 0〜6 を実行する。サブルーチン `:scene`・`:runcmd`(`echo $ conductor %*`、実行、`→ 終了コード %code%`)・`:pause`。1 コマンドだけの環境変数は `setlocal` / `set "VAR=値"` / `call :runcmd …` / `endlocal`。`DEMO_SLEEP` は整数の秒(既定 1。ジョブは `ping -n <秒+1> 127.0.0.1` で待つ)、`DEMO_LANG` は `en_US` 形式(`LANG` に設定し、`LC_ALL`・`LC_MESSAGES` は空にする)。実行中だけコードページを UTF-8(65001)にして戻す。色なし。`conductor.toml` は同じ `nightly`(`command` は `'scripts\extract.cmd'` のような文字列と、notify の配列 `["cmd", "/c", "scripts\\notify.cmd"]`)。
- `demo/README.md`: UNIX 版の使い方、場面の表(1〜6)、箇条書き、「## Windows 版」の節。
- `crates/cli/tests/windows_demo.rs`(`cfg(windows)` のみ): `run-demo.cmd` は実行せず、`demo/windows` の定義とスクリプトを一時ディレクトリへ写して、本物の `conductor`(`LANG=C`、`DEMO_SLEEP=0`)で、場面 1〜6 の終了コードとジョブの出力(`[<job>] 実行中`)を確かめる。`CONDUCTOR_RUN_ID` は外さない。
