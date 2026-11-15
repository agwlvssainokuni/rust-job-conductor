# コンポーネント一覧

見出しの名前は、`reverse-engineering-timestamp.md` の `analyzed.components` と同じ。

## conductor-core

- 種類・場所: lib、`crates/core/`(版 0.6.0)
- 責務: 定義ファイル(TOML)の読み込みと検証、ジョブの結果の分類・起動前の判断・完了・再開位置、run ID の検査。純粋(ファイルに触れるのは `definition/intake.rs` だけ)。
- 主な公開: `load_definition`、`parse_definition`、`decide_before_run`、`classify_and_decide`、`resume_point_on_interrupt`、`RunId`、`peek_run_id_env`、型(`PipelineDefinition` ほか)。
- 依存: `toml`、`thiserror`。内部の依存なし。
- テスト: `tests/` 6 ファイル。

## conductor-runtime

- 種類・場所: lib + テスト用 bin `conductor-test-helper`、`crates/runtime/`
- 責務: ジョブの直接起動(シェルなし)と終了の待ち(`try_wait` のくり返し)、シグナルの転送、command の分割、ステートの読み込み・原子的な書き込み・初期化・`reset --all`、照合、記録。
- 主な公開: `run_job`、`check_command`、`read_state`、`write_state`、`reset_state`、`reset_all_states`、`check_consistency`、`initial_state`、`apply_*`。
- 依存: `conductor-core`、`toml`、`thiserror`、`libc`(UNIX)。`unsafe` は UNIX の `kill` 1 か所。
- テスト: `tests/` 11 ファイル。

## conductor-converter

- 種類・場所: lib、`crates/converter/`
- 責務: 定義ファイルの YAML と TOML の変換。展開(アンカー・エイリアス・マージキー)の上限つき。
- 主な公開: `convert`、`convert_text`、`TargetFormat`、`ConversionError`。
- 依存: `toml`、`thiserror`、`saphyr-parser`。`conductor-core` には依存しない。
- テスト: `tests/` 4 ファイル(フィクスチャあり)。

## conductor-cli

- 種類・場所: lib `conductor_cli` + bin `conductor`(配布)+ テスト用 bin `conductor-cli-test-helper`、`crates/cli/`
- 責務: 引数の解釈、`run`/`pipeline`/`watch`/`reset`/`validate`/`convert` の流れ、シグナルの受け取り、表示(英語・日本語・色・エスケープ)、終了コード、パニックのフック。
- 主な公開: `run_cli`、`run_cli_with`、`parse_args`、`Deps` と trait(`Runtime`・`Sink`・`Env`・`Signals`)、`exit` の定数。
- 依存: core・runtime・converter、`lexopt`、`signal-hook`(UNIX)、`ctrlc`(Windows)。
- テスト: `tests/` 20 ファイル(バイナリ実行、シグナル、デモ、文言、性能の測定)。

## CI とリリースのワークフロー

- 場所: `.github/workflows/{ci,release,audit}.yml`
- 責務: lint、ビルド(Linux・Windows。MSRV 1.85)、テスト(Linux・macOS・Windows)、MSRV のテスト、カバレッジ 80%、`docs-check`(README の 7 語)、起動の測定、`cargo audit`、タグ `v*` での 4 ターゲットのビルドと下書きのリリース。

## デモ

- 場所: `demo/`(UNIX: `run-demo.sh`、`conductor.toml`、`scripts/*.sh`)、`demo/windows/`(`run-demo.cmd`、`conductor.toml`、`scripts/*.cmd`)、`demo/README.md`
- 責務: 9 場面(+ 場面 0 の定義の表示)で、conductor の主な動きを見せる。一時ディレクトリで動く。`crates/cli/tests/unix_demo.rs`・`windows_demo.rs` が動作を固定する。

## ドキュメント(README.md)

- 場所: `README.md`(283 行)
- 責務: 利用者向けの唯一の文書(インストール、使い方、定義、終了コード、ステート、保証、シグナル、convert、ロケール)。CI の `docs-check` が 7 語を要求する。構成は developer-scan.md の Handoff Summary (1) を参照。
