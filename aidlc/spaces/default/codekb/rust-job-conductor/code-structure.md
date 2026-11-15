# コード構造

## ルートの構成

| パス | 内容 |
|------|------|
| `Cargo.toml` / `Cargo.lock` | ワークスペース(`crates/*`)。edition 2024、`rust-version = "1.85"`、`resolver = "3"`、release プロファイルは lto thin・strip |
| `crates/core/` | `conductor-core`(lib) |
| `crates/runtime/` | `conductor-runtime`(lib + テスト用 bin) |
| `crates/converter/` | `conductor-converter`(lib) |
| `crates/cli/` | `conductor-cli`(lib `conductor_cli` + bin `conductor` + テスト用 bin) |
| `demo/` | UNIX 向けデモ(`run-demo.sh`、`conductor.toml`、`scripts/*.sh`)と、Windows 向け `demo/windows/`(`run-demo.cmd`、`conductor.toml`、`scripts/*.cmd`) |
| `.github/workflows/` | `ci.yml`、`release.yml`、`audit.yml` |
| `README.md`、`demo/README.md`、`LICENSE` | 文書(日本語)、Apache-2.0 |

## モジュール(`src/`)

- **core**: `definition/{intake,parse,depth,version,walk}`(取り込み → 読み取り → 深さ(32 段)→ 版 → 構造の検査)、`flow/{classify,decide,next_step,resume}`、`error.rs`(`DefinitionError`、`Locate`)、`run_id.rs`、`types.rs`。
- **runtime**: `runner.rs`(起動と待ち)、`split.rs`(command の分割)、`state/{reader,writer,record,consistency,reset_all}`、`platform/{unix,windows}`、`types.rs`、`error.rs`、`bin/conductor-test-helper.rs`。
- **converter**: `intake`、`raw`、`resolve`(アンカー・マージの展開)、`scalar`、`{toml,yaml}_{read,write}`、`types`、`error`。
- **cli**: `main.rs`(薄い入口)、`lib.rs`(`run_cli`・`run_cli_with`)、`args.rs`、`ctx.rs`、`run.rs`、`pipeline.rs`、`aux_commands.rs`、`exit.rs`、`deps.rs`、`panic_hook.rs`、`platform/`、`report/{mod,messages,watch,escape}`、`bin/conductor-cli-test-helper.rs`。

## ファイルの分類

- 実行ファイルの入口: `crates/cli/src/main.rs`
- 流れ(調整): `ctx.rs`、`run.rs`、`pipeline.rs`、`aux_commands.rs`
- 純粋な判断: core の `flow/`、runtime の `state/{record,consistency}`、cli の `exit.rs`、`report/watch.rs`
- 入出力・OS: core の `definition/intake.rs`、runtime の `state/{reader,writer,reset_all}`・`runner.rs`・`platform/`、cli の `platform/`・`deps.rs`
- 文言: `report/messages.rs`(英語・日本語の対)
- テスト: 各クレートの `tests/`(41 ファイル)。cli の `tests/` には、バイナリを実行するもの(`binary_*`、`unix_demo`、`windows_demo`、`signals_real`)がある。

## コードのパターン

- 差し替え口 trait(`Deps`)による依存の注入。`run_cli_with` がテストの入口。
- 検査済みの値の型(`RunId`、`StateLocation`)。パス・名前は型を通したものだけを使う。
- エラーは、種類 + 固定の文。入力の値は文に含めず、表示のときにエスケープして足す。
- コメント・テスト名は日本語。命名は Rust 標準。
