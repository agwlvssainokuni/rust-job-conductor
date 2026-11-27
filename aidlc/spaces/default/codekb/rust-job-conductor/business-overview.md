# ビジネス概要

## 目的

`conductor` は、既存のジョブスケジューラ(cron など)から呼び出される、ジョブの流れの制御ツールである。定義ファイル(既定は `conductor.toml`)に、ジョブと、結果(成功・警告・エラー)ごとの次の動きを書いておくと、ジョブを記述順に実行し、遷移の規則に従ってスキップや終了を行い、実行の状態(ステート)をファイルに覚える。エラーで止まったあとは、エラーのジョブから再開できる。

- 単一の実行ファイル(`conductor`)。外部のランタイムは要らない。Linux・macOS・Windows で、同じ定義に同じ結果を返す。
- ネットワークに接続せず、利用状況の収集もしない(`crates/cli/tests/nfr_source.rs`、`crates/runtime/tests/nfr_source.rs` が検査)。
- 表示は、ロケール(`LC_ALL`、`LC_MESSAGES`、`LANG`)に従って、英語と日本語を切り替える。
- 現在の版は 0.10.1(4 クレートとも同じ版。`crates/*/Cargo.toml`)。ライセンスは Apache-2.0。

## 利用者とユースケース

| 利用者 | ユースケース |
|--------|--------------|
| スケジューラの運用者 | cron 等から `conductor pipeline <名前>` または `conductor run <ジョブID>` を呼び、夜間バッチなどのジョブ列を制御する。`--run-id` で実行の単位ごとにステートを分ける |
| ジョブの作者 | 定義ファイルに、ジョブ、警告・成功の終了コード、遷移(`@end` を含む)、リトライ(`[job.retry]`、全ジョブの既定 `[pipeline.retry]`、打ち消し `retry = false`)、環境変数(`env`、`env_file`、無くてもよい `.env`、`unset_env`)、ログの置き場所(`log_dir`)を書く。`init` でひな形を作る |
| 障害対応の担当者 | エラーで止まったジョブから再実行する。`watch` で状態と所要時間の表を見る。`watch --follow` で終わるまで表示し直す。`graph --state` で止まった位置を図で見る。`log_dir` のファイルで試行ごとの出力を読む。`reset`(`--all`)でステートを消す |
| 定義の整備者 | `validate` で定義と `.env` を実行せずに検査する。`--dry-run` で実行の計画を確かめる。`graph` で流れを図にする(text、Mermaid、DOT)。`convert` で YAML と TOML を相互に変換する |
| 再実装する人(仕様書の読み手) | `docs/spec/` の仕様書だけを読んで、同じ挙動のツールを作る。この intent の主な読み手 |

## 主要な機能(0.10.1 時点)

1. 定義の読み込みと検証。誤りは 34 種(`DefinitionErrorKind`)、行番号つきで、まとめて報告する。
2. ジョブの直接起動(シェルを介さない)。`command` は文字列(空白と引用符の規則で分割)または文字列の配列。
3. 結果の分類(success / warning / error。起動失敗は 127、シグナルは 128 + 番号)と、遷移・`@end`・スキップ。
4. ステートファイル(TOML)による再開。run ID でステートを分ける。完了時の削除(`--remove-on-complete`、`remove_on_complete`)。v0.10.0 から、試行ごとの開始・終了の時刻(`[[job.attempt]]`)を記録する。
5. リトライ: ジョブごとの `[job.retry]`、全ジョブの既定 `[pipeline.retry]`、打ち消しの `retry = false`(v0.10.0)。
6. 環境変数の注入と除去: `[pipeline.env]`、`[pipeline].env_file`、`[job.env]`、`[job].env_file`、自動の変数(`CONDUCTOR_PIPELINE`、`CONDUCTOR_JOB_ID`、`CONDUCTOR_RUN_ID`、`CONDUCTOR_ATTEMPT`)。v0.10.0 で、無くてもよい `.env`(`{ path, required = false }`)と、引き継いだ環境から消す `unset_env`。`.env` の誤りは 14 種。
7. ジョブの出力のファイル化(`[pipeline] log_dir`、v0.10.0)。端末に出しつつ、試行ごとに `<安全な ID>.<開始の時刻>.<試行の番号>.log` に残す。
8. `graph`(流れの図。`--format text|mermaid|dot`、`--state`、`-v`。`dot` は v0.10.0)。
9. `--dry-run`(ジョブを起動せず、ステートに書かず、計画を出す。`--assume <ジョブID>=<終了コード>`)。
10. `init`(定義のひな形。英日。既存のファイルは `--force` なしでは壊さない)。
11. シグナルの転送と中断の記録(UNIX は SIGHUP・SIGINT・SIGQUIT・SIGTERM、Windows は Ctrl+C)。
12. 補助のコマンド: `watch`(状態・所要時間・補足の表。`--follow [--interval <秒>]` で表示し直しを続ける。v0.10.0)、`reset`、`validate`、`convert`。
13. 終了コードの規約は `api-documentation.md` の 6 を参照。

v0.10.1 は整理・修正の版で、利用者から見える新しい機能は無い(README の `watch --follow` の注意、ログの名前の予約名の比べ方、読み取りのスレッドの止め方、計画の模擬の内部の入口、デモの場面 4 の待ち)。

## 保証と非目標

- at-least-once: ジョブの終了とステートの書き込みのあいだに落ちると、そのジョブは再実行されうる。排他制御はしない。時間の上限(タイムアウト)は無い。
- 入力に対してパニックしない。エラー表示に入力の値を含めない。ステートには開始と終了の時刻だけを足し、command・環境変数・出力は書かない。環境変数の値を持つ型の `Debug` は伏せ字。
- ジョブの標準出力・標準エラーは加工しない。`log_dir` があるときだけ、パイプで受けて端末とファイルへ同じバイト列を書く(子からは端末に見えなくなる)。古いログは消さない。

## 業務上の主な取引(トランザクション)

| 取引 | 入口 | 結果 |
|------|------|------|
| ジョブ列の一括実行 | `conductor pipeline <名前>` | 全ジョブを記述順に処理し、完了のステートを残す(または削除する) |
| 1 ジョブの個別実行 | `conductor run <ジョブID>` | 起動前の判断に従い、実行・スキップ・拒否 |
| 再開 | 同じコマンドの再実行 | エラーのジョブ(`resume_job_id`)から再開 |
| 状態の確認 | `conductor watch` | ステートの表(1 回) |
| 状態の見守り | `conductor watch --follow` | 完了・止まった・消えた・中断まで表を描き直す(終了コード 0) |
| 流れの確認 | `conductor graph [--state]` | 図(text、Mermaid、DOT)を標準出力へ |
| 計画の確認 | `conductor pipeline <名前> --dry-run` | 実行の計画を標準出力へ(ジョブは起動しない) |
| 定義の検査 | `conductor validate` | 誤りの一覧(64)または成功 |
| ひな形の作成 | `conductor init [<名前>]` | 定義ファイル(または標準出力) |
| 形式の変換 | `conductor convert <file> --to <yaml\|toml>` | 変換した文字列を標準出力へ |

## この intent(v0102-spec-docs、v0.10.2)との関係

整理の intent で、コードの挙動は変えない。対象は 3 つ。

| 対象 | 関わる読み手 | 現状の要点 |
|------|--------------|------------|
| 仕様書 `docs/spec/rust/` を実装に合わせる | 再実装する人 | 版の表記と「34 種」だけが v0.10.1 で、v0.10.0 の機能(`watch --follow`、`log_dir`、`unset_env`、既定のリトライ、`env_file` の表、DOT、試行の時刻、所要時間)は書いていない。差の一覧は `code-quality-assessment.md` の「仕様書と実装の差」 |
| `team.md` のテストの数を直す | チーム(AI-DLC の規則の読み手) | 書かれた数は v0.10.0 の前の数。今の数は `code-quality-assessment.md` の「テスト」 |
| 版を 0.10.2 にする | 利用者 | 4 クレートの `Cargo.toml`、`Cargo.lock`、仕様書の版の表記 |

## 文書

利用者向けは `README.md`(日本語、955 行。必須の語句は CI の `docs-check` が検査)と `demo/README.md`(69 行)。仕様書は `docs/spec/`(7 版 × 13 文書。`rust/` が Rust 版の構成を、`language-agnostic/` が言語に依らない挙動を書く。ほかの 5 版は、冒頭の版の記述が v0.8.0・v0.9.0 で、v0.10.0 の機能を持たない)。仕様書を読むテストと CI は無い。
