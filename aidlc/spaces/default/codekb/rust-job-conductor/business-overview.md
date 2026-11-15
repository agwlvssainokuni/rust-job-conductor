# Business Overview

## 目的
`conductor`(v0.3.0)は、cron などの既存のジョブスケジューラから呼ばれる、ジョブの流れの制御ツールである。定義ファイル(`conductor.toml`)にジョブと、結果(成功・警告・エラー)ごとの遷移を書くと、記述順に実行し、スキップや終了を行い、ステートを保持して、エラーのジョブから再開できる(`README.md`)。

## 主な機能
- `run <ジョブID>`: 個別実行 / `pipeline <名前>`: 一括実行 / `watch`: ステートの表示 / `reset`: ステートの消去 / `convert`: YAML と TOML の変換。詳細は api-documentation.md。
- 単一の実行ファイル。ネットワーク接続なし、テレメトリなし。Linux・macOS・Windows で同じ定義に同じ結果。表示は環境のロケールで英語と日本語を切り替える。
- 保証は at-least-once(ジョブが 2 回動きうる)。自動の再試行・タイムアウト・排他制御はない。

## 利用者と配布
- 利用者は、バッチ運用の担当者。GitHub Releases から OS 別のアーカイブ(Linux musl、macOS arm64/x86_64、Windows msvc の zip)と `SHA256SUMS` を取得する。署名・公証はしていない。

## 現在の意図(windows-demo)との関係
- 動きを見せるデモが `demo/` にあるが、UNIX 系専用(`sh`)である(`demo/README.md` に「Windows では動かない」と明記)。Windows 版のデモを求められている。デモの構成は code-structure.md、制約は code-quality-assessment.md を参照。
