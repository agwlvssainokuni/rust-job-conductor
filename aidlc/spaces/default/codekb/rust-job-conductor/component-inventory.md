# Component Inventory

### conductor-cli
実行ファイル `conductor` とライブラリ。引数の解釈(lexopt。`args.rs`。`--run-id`・`--all` を含む)、流れの組み立て(run・pipeline・watch・reset・validate・convert。`ctx.rs` が実行前の検査 `prepare` と、定義だけの検査 `check_definition`、run ID の決定を持つ)、シグナルの監視(UNIX: signal-hook、Windows: ctrlc)、表示(英語・日本語。`watch` の `Run ID:` の行、`reset --all` の警告)、終了コードの決定、パニックのフック(70)。環境変数は `Env` を通して、`Reporter`(言語・色)と、`Ctx`(run ID)が読む。依存: conductor-core、conductor-runtime、conductor-converter。補助バイナリ `conductor-cli-test-helper`(配布しない)。

### conductor-core
定義の読み込み・検証(Intake → Parse → Depth → VersionCheck → Walk)と、フローの判断(起動前の判断、分類、次の動き、再開位置)。純粋。共有する型(`PipelineDefinition`、`PipelineState`、`JobDefinition`、`Command`、`Transition`、`RunId`)を持つ。`[pipeline]` の許可キーの表(`name`、`description`、`run_id_env`)と、run ID・環境変数名の検査(`run_id.rs`)を持つ。依存: toml、thiserror。

### conductor-runtime
ジョブの直接起動と待ち(try_wait のくり返し)、シグナルの 1 回転送、ステートの場所の導出(`StateLocation`。run ID 対応)・読み・書き・照合・消去(原子的な置き換え、古い一時ファイルの掃除)、`reset --all`(`state/reset_all.rs`)、command の分割と検査(`check_command`)。`platform` に OS 固有の処理。依存: conductor-core、toml、thiserror、libc(unix)。補助バイナリ `conductor-test-helper`(配布しない)。

### conductor-converter
YAML と TOML の相互変換。アンカー等の展開に上限(ノード 100,000、16 MiB、深さ 32)。依存: toml、saphyr-parser、thiserror。conductor-core に依存しない。`[pipeline]` のキーを知らない(今回は概観のみ)。

### demo
デモ一式。UNIX 版(`run-demo.sh`、`conductor.toml`、`scripts/*.sh`)と、Windows 版(`windows/run-demo.cmd`、`windows/conductor.toml`、`windows/scripts/*.cmd`)、説明の `README.md`。Rust のコンポーネントではない。依存: UNIX 版は `sh`、`mktemp`、`sleep`。Windows 版は cmd(OS 標準。`ping`、`chcp`、`xcopy`)。どちらもビルド済みの `conductor`(または `cargo`)を使う。場面は 0〜6(run ID の場面はまだない)。Windows 版だけ、自動テスト `crates/cli/tests/windows_demo.rs`(`cfg(windows)`。`run-demo.cmd` ではなく、定義とスクリプトを写して `conductor` を直接呼ぶ)が付く。構造は code-structure.md を参照。
