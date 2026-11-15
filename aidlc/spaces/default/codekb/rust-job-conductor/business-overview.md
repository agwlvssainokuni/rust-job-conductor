# Business Overview

## 目的
`conductor`(v0.5.0)は、cron などの既存のジョブスケジューラから呼ばれる、ジョブの流れの制御ツールである。定義ファイル(`conductor.toml`)にジョブと、結果(成功・警告・エラー)ごとの遷移を書くと、記述順に実行し、スキップや終了を行い、ステートを保持して、エラーのジョブから再開できる(`README.md`)。

## 主な機能
- `run <ジョブID>`: 個別実行 / `pipeline <名前>`: 一括実行 / `watch`: ステートの表示 / `reset`: ステートの消去 / `validate [<名前>]`: 定義と `command` の検査だけ(ジョブは実行せず、ステートに触れない) / `convert`: YAML と TOML の変換。詳細は api-documentation.md。
- 単一の実行ファイル。ネットワーク接続なし、テレメトリなし。Linux・macOS・Windows で同じ定義に同じ結果。表示は環境のロケールで英語と日本語を切り替える。
- 保証は at-least-once(ジョブが 2 回動きうる)。自動の再試行・タイムアウト・排他制御はない。同じ定義(同じ run ID)への同時の実行は、後に書いた方のステートが勝つ(`README.md`「ステートファイルの場所」)。
- run ID(v0.5.0 で実装済み): `--run-id <ID>`(`run`・`pipeline`・`watch`・`reset`)か、環境変数(名前は定義の `[pipeline].run_id_env`、既定 `CONDUCTOR_RUN_ID`)で、同じ定義を実行単位(日付ごと、CI の実行ごとなど)で別々のステートに分ける。ステートは `<定義ファイル名>.<run-id>.state`。同じ run ID で呼び直すと、止まったジョブから再開する。`watch` は `Run ID: <ID>` の行を出し、`reset --run-id <ID>` はその run ID だけ、`reset --all` はその定義のステートをすべて消す(`README.md:227-237`)。run ID なしの動きと、ステートの名前・形式は、変わらない。

## 利用者と配布
- 利用者は、バッチ運用の担当者。GitHub Releases から OS 別のアーカイブ(Linux musl、macOS arm64/x86_64、Windows msvc の zip)と `SHA256SUMS` を取得する。署名・公証はしていない。

## デモ
- 動きを見せるデモは 2 種類ある。UNIX 系は `demo/run-demo.sh`(`sh`)、Windows は `demo/windows/run-demo.cmd`(cmd。PowerShell からも動く)。どちらも同じ場面 0〜6(定義の表示、全部成功、警告で分岐、エラーで停止と `watch`、再開、個別実行と `reset`、`convert`)を見せる(`demo/README.md`)。**run ID の場面は、まだない**: `--run-id`・`run_id_env`・`reset --all`・`validate` を使う場面も、定義の `run_id_env` も、デモにはない。構成の詳細は code-structure.md を参照。

## 現在の意図(demo-run-id)との関係
- デモを、実装済みの run ID(v0.5.0)に合わせて更新する。対象は、UNIX 版(`demo/run-demo.sh`、`demo/conductor.toml`、`demo/scripts/*.sh`)、Windows 版(`demo/windows/run-demo.cmd`、`conductor.toml`、`scripts/*.cmd`)、`demo/README.md`、Windows 版の自動テスト `crates/cli/tests/windows_demo.rs`。見せたい場面: `--run-id` と環境変数、run ID ごとに別のステート、同じ run ID での再開、`watch` の `Run ID:` の行、`reset --run-id`、`reset --all`(`validate` も候補)。`conductor` 本体のソースは変えない。
- 見せ方の制約(逐次実行が安定、利用者の `CONDUCTOR_RUN_ID` が既存の場面に混ざる、など)は、この intent の `developer-scan.md` を参照。
- (前回までの意図: validate-command(完了。`validate` は実装済み)、run-id-state(完了。run ID を実装した)。)
