# Component Inventory

## conductor-core

- 責務: 定義(TOML)の取り込み・検証(Intake → Parse → Depth → VersionCheck → Walk)と、フロー判断(起動前判断、分類、次の動き、再開位置)。run ID の型と規則
- 場所: `crates/core/`
- 依存: `toml`、`thiserror`(内部クレートへの依存なし)

## conductor-runtime

- 責務: ジョブの直接起動と終了待ち、シグナル転送(子 pid のみ)、ステートの読み書き・照合・初期化・記録、`command` の分割
- 場所: `crates/runtime/`(テスト補助バイナリ `conductor-test-helper`)
- 依存: `conductor-core`、`toml`、`thiserror`、`libc`(unix)

## conductor-converter

- 責務: 定義の YAML と TOML の相互変換(アンカー等の展開の上限つき)
- 場所: `crates/converter/`
- 依存: `toml`、`thiserror`、`saphyr-parser`

## conductor-cli

- 責務: 引数解釈、各コマンドの流れの組み立て、シグナルの受け取り、表示(英語・日本語)、終了コードの決定、パニックのフック。バイナリ `conductor`
- 場所: `crates/cli/`(テスト補助バイナリ `conductor-cli-test-helper`)
- 依存: 上の 3 クレート、`lexopt`、`signal-hook`(unix)、`ctrlc`(windows)

## CI / リリース(コンポーネント外の構成要素)

`.github/workflows/{ci,audit,release}.yml`(詳細は technology-stack.md)。
