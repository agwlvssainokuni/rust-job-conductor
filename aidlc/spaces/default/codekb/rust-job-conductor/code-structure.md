# Code Structure

## ワークスペース

```
Cargo.toml / Cargo.lock        ワークスペースの設定(edition 2024、resolver 3)
crates/core/       src/{definition/,flow/,error,run_id,types}.rs, tests/
crates/runtime/    src/{state/,platform/,runner,split,error,types}.rs, src/bin/conductor-test-helper.rs, tests/
crates/converter/  src/{yaml_read,yaml_write,toml_read,toml_write,resolve,scalar,raw,intake,error,types}.rs, tests/(fixtures あり)
crates/cli/        src/{args,ctx,run,pipeline,aux_commands,deps,exit,panic_hook,types}.rs, src/{report/,platform/}, src/bin/conductor-cli-test-helper.rs, src/main.rs, tests/
demo/              UNIX(sh)と Windows(cmd)のデモ
.github/workflows/ ci.yml, audit.yml, release.yml
```

## ファイルの分類

| 種類 | 場所 |
|------|------|
| 型 | `*/src/types.rs` |
| エラーの表示 | `*/src/error.rs` |
| 純粋な判断 | `core/src/flow/`、`runtime/src/state/{consistency,record}.rs` |
| 入出力 | `core/src/definition/intake.rs`、`runtime/src/state/{reader,writer,reset_all}.rs`、`runtime/src/runner.rs`、`converter/src/intake.rs` |
| OS 依存 | `runtime/src/platform/`、`cli/src/platform/` |
| 表示 | `cli/src/report/` |
| テスト | `crates/*/tests/`(約 11,600 行) |
| テスト補助バイナリ | `src/bin/conductor-*test-helper.rs`(配布しない) |

## コードの規模

`src` は約 7,000 行、`tests` は約 11,600 行(概数。`wc -l`)。

## パターン

- trait による依存の差し替え(cli の `Deps`)
- ファイル名は Rust 標準の snake_case、型は PascalCase
- 文書コメントに契約・モジュール構成を記載(日本語)
- テスト名に日本語を使用(例: `crates/cli/tests/nfr_source.rs`)
