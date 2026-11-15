# Dependencies

## 外部の依存
| クレート | 版 | 使うクレート | 条件 |
|----------|-----|--------------|------|
| lexopt | 0.3.2 | cli | 全 OS |
| signal-hook | 0.4.6(iterator のみ) | cli | cfg(unix) |
| ctrlc | 3.5.2 | cli | cfg(windows) |
| libc | 0.2 | runtime | cfg(unix) |
| toml | 1.2.1 | core、converter、runtime | 全 OS |
| saphyr-parser | 0.3.4 | converter | 全 OS |
| thiserror | 2.0.21 | core、converter、runtime | 全 OS |
| tempfile | 3.27.0 | 全クレート(dev) | テストのみ |

(版は前回の記録。今回は `Cargo.toml` の版の再確認をしていない。`crates/cli/Cargo.toml` の `signal-hook = "0.4.6"` と、クレートの版 0.5.0 だけを確認した。)

## 内部の依存
cli → core、runtime、converter / runtime → core / converter は core に依存しない。`PipelineDefinition`(core)のフィールドを使うのは、core 自身、runtime(`state/consistency`・`state/record`)、cli(`ctx`・`run`・`pipeline`・`aux_commands`)。`PipelineState`(core の型。`run_id` を含む)を、runtime が生成・保持し(`state/{reader,writer,record,reset_all}`)、cli が `run`・`pipeline`・`report/watch` で読む。`RunId`(core)は、cli(`args`・`ctx`)と runtime(`types`・`state/*`)が使う。converter は、定義の型を使わない。

## デモの依存
- UNIX 版: `sh`、`mktemp`、`cp`、`sleep`(小数を受け付けるもの)、`conductor`(または `cargo`)。
- Windows 版: cmd、`chcp`、`xcopy`、`ping`、`conductor.exe`(または `cargo`)。
- デモの定義は `conductor` の定義の書式(`format_version = 1`)に従い、`[pipeline]` の許可キーの変更(run ID の `run_id_env`)の影響を受ける。

## 制約
CI が、C のビルドを要する依存(cc、cmake、pkg-config、bindgen)と、`windows-sys`・`linux-raw-sys` 以外の `-sys` クレートを禁止している(`ci.yml` の build-check。前回の記録)。`cargo audit` を毎週月曜と PR で実行(警告のみ)。run ID の実装は、標準ライブラリの `std::fs`・`std::path` だけで足りた(新しい依存は増えていない)。
