# Component Inventory

### conductor-cli
実行ファイル `conductor` とライブラリ。引数の解釈(lexopt)、流れの組み立て(run・pipeline・watch・reset・convert)、シグナルの監視(UNIX: signal-hook、Windows: ctrlc)、表示(英語・日本語)、終了コードの決定、パニックのフック(70)。依存: conductor-core、conductor-runtime、conductor-converter。補助バイナリ `conductor-cli-test-helper`(配布しない)。

### conductor-core
定義の読み込み・検証(Intake → Parse → Depth → VersionCheck → Walk)と、フローの判断(起動前の判断、分類、次の動き、再開位置)。純粋。依存: toml、thiserror。

### conductor-runtime
ジョブの直接起動と待ち(try_wait のくり返し)、シグナルの 1 回転送、ステートの読み・書き・照合・消去(原子的な置き換え)、command の分割。`platform` に OS 固有の処理。依存: conductor-core、toml、thiserror、libc(unix)。補助バイナリ `conductor-test-helper`(配布しない)。

### conductor-converter
YAML と TOML の相互変換。アンカー等の展開に上限(ノード 100,000、16 MiB、深さ 32)。依存: toml、saphyr-parser、thiserror。conductor-core に依存しない。

### demo
UNIX 系専用のデモ一式(`run-demo.sh`、`conductor.toml`、`scripts/*.sh`)。Rust のコンポーネントではない。依存: `sh`、`mktemp`、`sleep`、ビルド済みの `conductor`(または `cargo`)。
