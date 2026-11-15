# Component Inventory

## conductor-core
- **責務**: 定義ファイルの読み込みと検証(`definition/`: Intake, Parse, Depth, Version, Walk)、共有型(`types.rs`)、フローの判断(`flow/`: 分類、実行前の判断、次の動き、再開位置)。
- **依存**: `toml`, `thiserror`。内部依存なし。入出力は `definition/intake.rs` だけ。
- **command との関係**: `JobDefinition.command`(L50)、`JobExecution.command`(L101)、`walk.rs` の読み取りと空判定。`flow/` は command を使わない。

## conductor-runtime
- **責務**: command の分割(`split.rs`)、ジョブの起動と待ち・シグナル転送(`runner.rs`)、ステートの読み書き・記録・照合(`state/`)、OS 固有(`platform/`)、中断の合図 `InterruptSignal`。
- **依存**: conductor-core, `toml`, `thiserror`, `libc`(UNIX)。
- **command との関係**: `split`/`check_command`/`run_job`、`CommandLine`、`CommandError`。ステートは command を持たない。

## conductor-converter
- **責務**: YAML と TOML の相互変換。素の木(`raw.rs`)→ 展開(`resolve.rs`)→ `Value` → 書き出し。展開の上限(ノード 100,000、16 MiB、深さ 32)。
- **依存**: `toml`, `thiserror`, `saphyr-parser`。core に依存しない。
- **command との関係**: 無し(スキーマを知らず、配列を一般の値として往復する。確認はテスト未実行)。

## conductor-cli
- **責務**: 引数解析(`args.rs`, lexopt)、流れ(`run.rs`, `pipeline.rs`, `aux_commands.rs`, 共通部 `ctx.rs`)、シグナル(`platform/`)、表示(`report/`: メッセージ、watch の表、エスケープ)、終了コード(`exit.rs`)、パニックフック。差し替え口は `deps.rs`。
- **依存**: core, runtime, converter, `lexopt`, `signal-hook`(UNIX), `ctrlc`(Windows)。
- **command との関係**: `ctx.rs` の事前検査と起動失敗の表示、`deps.rs` の `check_command`、`report/mod.rs` の `CommandErrors`・`StartFailure`。

## 補助
- `crates/runtime/src/bin/conductor-test-helper.rs`、`crates/cli/src/bin/conductor-cli-test-helper.rs`: テスト専用で配布に含めない。
- `demo/`: 動作のデモ(`run-demo.sh`、`conductor.toml`、`scripts/`)。
