# Business Overview

## 目的
`conductor`(v0.4.0)は、cron などの既存のジョブスケジューラから呼ばれる、ジョブの流れの制御ツールである。定義ファイル(`conductor.toml`)にジョブと、結果(成功・警告・エラー)ごとの遷移を書くと、記述順に実行し、スキップや終了を行い、ステートを保持して、エラーのジョブから再開できる(`README.md`)。

## 主な機能
- `run <ジョブID>`: 個別実行 / `pipeline <名前>`: 一括実行 / `watch`: ステートの表示 / `reset`: ステートの消去 / `validate [<名前>]`: 定義と `command` の検査だけ(ジョブは実行せず、ステートに触れない) / `convert`: YAML と TOML の変換。詳細は api-documentation.md。
- 単一の実行ファイル。ネットワーク接続なし、テレメトリなし。Linux・macOS・Windows で同じ定義に同じ結果。表示は環境のロケールで英語と日本語を切り替える。
- 保証は at-least-once(ジョブが 2 回動きうる)。自動の再試行・タイムアウト・排他制御はない。同じ定義への同時の実行は、後に書いた方のステートが勝つ(`README.md`「ステートファイルの場所」)。
- ステートは、定義ファイル 1 つにつき 1 つ(`<定義ファイル名>.state`)。同じ定義を、複数の実行単位(日付ごとなど)で別々に持つ手段は、現状ない。

## 利用者と配布
- 利用者は、バッチ運用の担当者。GitHub Releases から OS 別のアーカイブ(Linux musl、macOS arm64/x86_64、Windows msvc の zip)と `SHA256SUMS` を取得する。署名・公証はしていない。

## デモ
- 動きを見せるデモは 2 種類ある。UNIX 系は `demo/run-demo.sh`(`sh`)、Windows は `demo/windows/run-demo.cmd`(cmd。PowerShell からも動く)。どちらも同じ場面 0〜6 を見せる(`demo/README.md`)。構成は code-structure.md を参照(今回の走査の対象外。前回の記録)。

## 現在の意図(run-id-state)との関係
- 状態ファイルを、実行単位ごとに分けたい、という要望がある(run ID)。`--run-id <値>`(`run`・`pipeline`・`watch`・`reset`)と、定義ファイルの `[pipeline] run_id_env`(環境変数の名前。既定 `CONDUCTOR_RUN_ID`)で指定し、状態ファイル名は `<定義ファイル>.<run-id>.state` になる。run ID なしの動きは変わらない。`reset --all` で、その定義の状態ファイルをまとめて消す。`validate` は `run_id_env` の変数名の形だけを検査する。
- 現状のコードでの出発点は、architecture.md の「ステートファイルの場所と呼び出し元」と、この intent の `developer-scan.md` を参照。
- (前回の意図 validate-command は完了し、`validate` は実装済み。内部のフィールド名 `PipelineDefinition.jobs` も `job` に改名済み。)
