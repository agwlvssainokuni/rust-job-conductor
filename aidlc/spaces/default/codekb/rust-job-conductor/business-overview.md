# ビジネス概要

## 目的

`conductor` は、既存のジョブスケジューラ(cron など)から呼び出される、ジョブの流れの制御ツールである。定義ファイル(既定は `conductor.toml`)に、ジョブと、結果(成功・警告・エラー)ごとの次の動きを書いておくと、ジョブを記述順に実行し、遷移の規則に従ってスキップや終了を行い、実行の状態(ステート)をファイルに覚える。エラーで止まったあとは、エラーのジョブから再開できる。

- 単一の実行ファイル(`conductor`)。外部のランタイムは要らない。Linux・macOS・Windows で、同じ定義に同じ結果を返す。
- ネットワークに接続せず、利用状況の収集もしない(`nfr_source.rs` が検査)。
- 表示は、ロケール(`LC_ALL`、`LC_MESSAGES`、`LANG`)に従って、英語と日本語を切り替える。
- 現在の版は 0.9.9(4 クレートとも同じ版。`crates/*/Cargo.toml`)。ライセンスは Apache-2.0。

## 利用者とユースケース

| 利用者 | ユースケース |
|--------|--------------|
| スケジューラの運用者 | cron 等から `conductor pipeline <名前>` または `conductor run <ジョブID>` を呼び、夜間バッチなどのジョブ列を制御する。`--run-id` で実行の単位ごとにステートを分ける |
| ジョブの作者 | 定義ファイルに、ジョブ、警告・成功の終了コード、遷移(`@end` を含む)、リトライ(`[job.retry]`)、環境変数(`env`、`env_file`)を書く。`init` でひな形を作る |
| 障害対応の担当者 | エラーで止まったジョブから再実行する。`watch` で状態の表を見る。`graph --state` で止まった位置を図で見る。`reset`(`--all`)でステートを消す |
| 定義の整備者 | `validate` で定義と `.env` を実行せずに検査する。`--dry-run` で実行の計画を確かめる。`graph` で流れを図にする(text、Mermaid)。`convert` で YAML と TOML を相互に変換する |

## 主要な機能(0.9.9 時点)

1. 定義の読み込みと検証。誤りは 33 種(`DefinitionErrorKind`)、行番号つきで、まとめて報告する。
2. ジョブの直接起動(シェルを介さない)。`command` は文字列(空白と引用符の規則で分割)または文字列の配列。
3. 結果の分類(success / warning / error。起動失敗は 127、シグナルは 128 + 番号)と、遷移・`@end`・スキップ。
4. ステートファイル(TOML)による再開。run ID でステートを分ける(`--run-id`、定義の `run_id_env`、既定の `CONDUCTOR_RUN_ID`)。完了時の削除(`--remove-on-complete`、`remove_on_complete`)。
5. リトライ(`[job.retry]`。対象の終了コード、最大回数、間隔、倍率と上限)。ジョブごとの設定だけ。
6. 環境変数の注入: `[pipeline.env]`、`[pipeline].env_file`、`[job.env]`、`[job].env_file` と、自動の変数(`CONDUCTOR_PIPELINE`、`CONDUCTOR_JOB_ID`、`CONDUCTOR_RUN_ID`、`CONDUCTOR_ATTEMPT`)。`.env` の書式の誤りは 13 種。
7. `graph`(流れの図。`--format text|mermaid`、`--state`、`-v`)。
8. `--dry-run`(ジョブを起動せず、ステートに書かず、計画を出す。`--assume <ジョブID>=<終了コード>`)。
9. `init`(定義のひな形。英日。既存のファイルは `--force` なしでは壊さない)。
10. シグナルの転送と中断の記録(UNIX は SIGHUP・SIGINT・SIGQUIT・SIGTERM、Windows は Ctrl+C)。
11. 補助のコマンド: `watch`(状態の表を 1 回)、`reset`、`validate`、`convert`。
12. 終了コードの規約: 成功・警告・スキップは 0。ジョブのエラーはジョブの終了コード。`conductor` 自身のエラーは 64(定義)、65(ステート・出力)、66(拒否)、67(変換)、68(使い方)、69(`init` の作る先が既にある)、70(内部)。

## 保証と非目標

- at-least-once: ジョブの終了とステートの書き込みのあいだに落ちると、そのジョブは再実行されうる。排他制御はしない。時間の上限(タイムアウト)は無い。
- 入力に対してパニックしない。エラー表示に入力の値(誤った値、ライブラリのエラー文)を含めない。ステートに command・環境変数・出力を書かない。環境変数の値を持つ型の `Debug` は伏せ字。
- ジョブの標準出力・標準エラーは加工せず、そのまま流す(ファイルには残さない)。

## 業務上の主な取引(トランザクション)

| 取引 | 入口 | 結果 |
|------|------|------|
| ジョブ列の一括実行 | `conductor pipeline <名前>` | 全ジョブを記述順に処理し、完了のステートを残す(または削除する) |
| 1 ジョブの個別実行 | `conductor run <ジョブID>` | 起動前の判断に従い、実行・スキップ・拒否 |
| 再開 | 同じコマンドの再実行 | エラーのジョブ(`resume_job_id`)から再開 |
| 状態の確認 | `conductor watch` | ステートの表(1 回) |
| 流れの確認 | `conductor graph [--state]` | 図(text または Mermaid)を標準出力へ |
| 計画の確認 | `conductor pipeline <名前> --dry-run` | 実行の計画を標準出力へ(ジョブは起動しない) |
| 定義の検査 | `conductor validate` | 誤りの一覧(64)または成功 |
| ひな形の作成 | `conductor init [<名前>]` | 定義ファイル(または標準出力) |
| 形式の変換 | `conductor convert <file> --to <yaml\|toml>` | 変換した文字列を標準出力へ |

## この intent(small-feature-batch、v0.10.0)との関係

現在の実装に無く、この intent が足す予定の 7 つの機能(要求の確定は後続のステージ)。影響する場所は `architecture.md` の「改善の余地」と `code-quality-assessment.md` を参照。

| 機能 | 現状 |
|------|------|
| `[pipeline.retry]`(全ジョブの既定のリトライ) | リトライはジョブ(`[job.retry]`)だけ。`[pipeline]` に `retry` のキーは無い |
| `graph --format dot` | 形式は `text` と `mermaid` だけ |
| 無くてもよい `.env`(`env_file`) | `env_file` のファイルが無いと、定義の誤り(64)で止まる |
| `unset_env` | 環境は親から引き継ぎ、足す・上書きするだけ。消す手段は無い |
| `log_dir`(ジョブの出力をファイルに残す) | ジョブの標準出力・標準エラーは引き継ぐだけ |
| ステートの開始・終了の時刻と、`watch` の所要時間 | ステートに時刻は無い |
| `watch` の連続表示 | `watch` は 1 回だけ表示して終わる |

## 文書

利用者向けは `README.md`(日本語。必須の語句は CI の `docs-check` が検査)と `demo/README.md`。仕様書は `docs/spec/`(7 言語版 × 13 文書。`rust/` が Rust 版の構成を書く)。
