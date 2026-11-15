# Component Inventory

### conductor-cli
実行ファイル `conductor` とライブラリ。引数の解釈(lexopt。`args.rs`)、流れの組み立て(run・pipeline・watch・reset・validate・convert。`ctx.rs` が実行前の検査 `prepare` と、定義だけの検査 `check_definition` を持つ)、シグナルの監視(UNIX: signal-hook、Windows: ctrlc)、表示(英語・日本語)、終了コードの決定、パニックのフック(70)。環境変数は `Env` を通して `Reporter` だけが読む。依存: conductor-core、conductor-runtime、conductor-converter。補助バイナリ `conductor-cli-test-helper`(配布しない)。

### conductor-core
定義の読み込み・検証(Intake → Parse → Depth → VersionCheck → Walk)と、フローの判断(起動前の判断、分類、次の動き、再開位置)。純粋。共有する型(`PipelineDefinition`、`PipelineState`、`JobDefinition`、`Command`、`Transition`)を持つ。`[pipeline]` の許可キーの表(`name`、`description`)を持つ。依存: toml、thiserror。

### conductor-runtime
ジョブの直接起動と待ち(try_wait のくり返し)、シグナルの 1 回転送、ステートの場所の導出(`StateLocation`)・読み・書き・照合・消去(原子的な置き換え、古い一時ファイルの掃除)、command の分割と検査(`check_command`)。`platform` に OS 固有の処理。依存: conductor-core、toml、thiserror、libc(unix)。補助バイナリ `conductor-test-helper`(配布しない)。

### conductor-converter
YAML と TOML の相互変換。アンカー等の展開に上限(ノード 100,000、16 MiB、深さ 32)。依存: toml、saphyr-parser、thiserror。conductor-core に依存しない。`[pipeline]` のキーを知らない(今回は概観のみ)。

### demo
デモ一式。UNIX 版(`run-demo.sh`、`conductor.toml`、`scripts/*.sh`)と、Windows 版(`windows/run-demo.cmd`、`windows/conductor.toml`、`windows/scripts/*.cmd`)。Rust のコンポーネントではない。依存: UNIX 版は `sh`、`mktemp`、`sleep`。Windows 版は cmd(OS 標準)。どちらもビルド済みの `conductor`(または `cargo`)を使う(今回は未走査。前回の記録)。
