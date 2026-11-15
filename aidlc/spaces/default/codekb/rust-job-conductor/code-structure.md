# Code Structure

## ディレクトリ構成
| パス | 区分 | 内容 |
|------|------|------|
| `Cargo.toml`、`Cargo.lock` | 設定 | ワークスペース |
| `crates/cli/` | コード | `src/`(args、ctx、run、pipeline、aux_commands、deps、exit、report/、platform/{mod,unix,windows}、panic_hook、types、main)、`src/bin/conductor-cli-test-helper.rs`、`tests/` |
| `crates/core/` | コード | `src/definition/`(intake、parse、depth、version、walk)、`src/flow/`(classify、decide、next_step、resume)、error、types |
| `crates/runtime/` | コード | `src/`(runner、split、types、error)、`state/`(reader、writer、consistency、record)、`platform/{unix,windows}`、`bin/conductor-test-helper.rs` |
| `crates/converter/` | コード | `src/`(intake、raw、resolve、scalar、toml_read/write、yaml_read/write、types、error)、`tests/fixtures/` |
| `demo/` | デモ(sh) | `run-demo.sh`、`conductor.toml`、`scripts/{extract,transform,load,notify}.sh`、`README.md` |
| `.github/workflows/` | CI | `ci.yml`、`release.yml`、`audit.yml` |
| `README.md`、`LICENSE` | 文書 | 日本語の利用説明、Apache-2.0 |

## 規約とパターン
- 各ファイルの冒頭に Apache-2.0 のヘッダ(Rust は `//`、sh・YAML は `#`)。コメントと説明は日本語。
- クレートの `lib.rs` に、公開関数・モジュールの対応を表で書く。`unsafe` は runtime の UNIX 系 `kill` だけ(cli は `forbid`)。
- テストは `crates/*/tests/` に統合テストで置き、`common/` に補助を置く。

## デモの構造(intent に関係)
- `run-demo.sh`: `mktemp -d` の一時ディレクトリに `conductor.toml` と `scripts/` をコピーし、6 つの場面を順に実行する。環境変数: `CONDUCTOR`、`DEMO_LANG`、`DEMO_PAUSE`、`DEMO_<ジョブ>_EXIT`、`DEMO_SLEEP`。
- `conductor.toml`: パイプライン `nightly`。ジョブは extract(警告=終了コード 1 で notify へ遷移)、transform、load、notify(配列形式の `command`)。
- `scripts/*.sh`: 各ジョブは、メッセージを出し、`sleep` し、`DEMO_<NAME>_EXIT` の値で終了する。
