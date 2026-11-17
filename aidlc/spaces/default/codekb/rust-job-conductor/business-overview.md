# Business Overview

## ビジネス文脈

`conductor` は、cron などの既存のジョブスケジューラから呼ばれる、ジョブの流れの制御ツールである。定義ファイル(`conductor.toml`)にジョブと、結果(成功・警告・エラー)ごとの次の動きを書くと、ジョブを記述順に実行し、遷移の規則に従ってスキップや終了を行い、実行の状態(ステート)を覚える。エラーで止まったあとは、エラーのジョブから再開できる(README.md)。

## 主な機能

- `run` / `pipeline`: ジョブ 1 つ、またはパイプライン全体の実行(再開、スキップ、`@end`)
- `watch`: ステートの表の表示(完了行を含む、v0.7.0 以降)
- `reset`(`--run-id`、`--all`): ステートの消去
- `validate`: 定義と全ジョブの `command` の検査(ジョブは実行しない)
- `convert`: 定義の YAML と TOML の相互変換
- run ID によるステートの分離(v0.5.0 以降)、`--remove-on-complete`(v0.7.0 以降)

## 非機能の特徴

- 単一の実行ファイル。ネットワーク接続なし、テレメトリなし。Linux・macOS・Windows で同じ結果
- 表示は、ロケールに従い英語と日本語を切り替える
- 保証は at-least-once(落ちたときにジョブが再実行されうる)
- 配布は GitHub Releases(Linux musl x86_64、macOS arm64 / x86_64、Windows x86_64)。ライセンスは Apache-2.0
- 現在の版は 0.7.0(`crates/*/Cargo.toml`)

## 利用者

ジョブスケジューラの運用者(定義ファイルの作成者)。開発者は Rust に不慣れ(`aidlc/active-memory/team.md` の方針)。
