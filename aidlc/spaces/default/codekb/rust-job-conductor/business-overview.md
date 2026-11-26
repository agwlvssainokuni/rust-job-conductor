# ビジネス概要

## 目的

`conductor` は、既存のジョブスケジューラ(cron など)から呼び出される、ジョブの流れの制御ツールである。定義ファイル(既定は `conductor.toml`)に、ジョブと、結果(成功・警告・エラー)ごとの次の動きを書いておくと、ジョブを記述順に実行し、遷移の規則に従ってスキップや終了を行い、実行の状態(ステート)をファイルに覚える。エラーで止まったあとは、エラーのジョブから再開できる。

- 単一の実行ファイル(`conductor`)。外部のランタイムは要らない。Linux・macOS・Windows で、同じ定義に同じ結果を返す。
- ネットワークに接続せず、利用状況の収集もしない(`crates/cli/tests/nfr_source.rs`、`crates/runtime/tests/nfr_source.rs` が検査)。
- 表示は、ロケール(`LC_ALL`、`LC_MESSAGES`、`LANG`)に従って、英語と日本語を切り替える。
- 現在の版は 0.10.0(4 クレートとも同じ版。`crates/*/Cargo.toml`)。ライセンスは Apache-2.0。

## 利用者とユースケース

| 利用者 | ユースケース |
|--------|--------------|
| スケジューラの運用者 | cron 等から `conductor pipeline <名前>` または `conductor run <ジョブID>` を呼び、夜間バッチなどのジョブ列を制御する。`--run-id` で実行の単位ごとにステートを分ける |
| ジョブの作者 | 定義ファイルに、ジョブ、警告・成功の終了コード、遷移(`@end` を含む)、リトライ(`[job.retry]`、全ジョブの既定 `[pipeline.retry]`、打ち消し `retry = false`)、環境変数(`env`、`env_file`、無くてもよい `.env`、`unset_env`)、ログの置き場所(`log_dir`)を書く。`init` でひな形を作る |
| 障害対応の担当者 | エラーで止まったジョブから再実行する。`watch` で状態と所要時間の表を見る。`watch --follow` で終わるまで表示し直す。`graph --state` で止まった位置を図で見る。`log_dir` のファイルで試行ごとの出力を読む。`reset`(`--all`)でステートを消す |
| 定義の整備者 | `validate` で定義と `.env` を実行せずに検査する。`--dry-run` で実行の計画を確かめる。`graph` で流れを図にする(text、Mermaid、DOT)。`convert` で YAML と TOML を相互に変換する |

## 主要な機能(0.10.0 時点)

1. 定義の読み込みと検証。誤りは 34 種(`DefinitionErrorKind`)、行番号つきで、まとめて報告する。
2. ジョブの直接起動(シェルを介さない)。`command` は文字列(空白と引用符の規則で分割)または文字列の配列。
3. 結果の分類(success / warning / error。起動失敗は 127、シグナルは 128 + 番号)と、遷移・`@end`・スキップ。
4. ステートファイル(TOML)による再開。run ID でステートを分ける。完了時の削除(`--remove-on-complete`、`remove_on_complete`)。v0.10.0 からは、試行ごとの開始・終了の時刻(`[[job.attempt]]`)を記録する。
5. リトライ: ジョブごとの `[job.retry]`、全ジョブの既定 `[pipeline.retry]`、打ち消しの `retry = false`(v0.10.0)。
6. 環境変数の注入と除去: `[pipeline.env]`、`[pipeline].env_file`、`[job.env]`、`[job].env_file`、自動の変数(`CONDUCTOR_PIPELINE`、`CONDUCTOR_JOB_ID`、`CONDUCTOR_RUN_ID`、`CONDUCTOR_ATTEMPT`)。v0.10.0 で、無くてもよい `.env`(`{ path, required = false }`)と、引き継いだ環境から消す `unset_env`。`.env` の誤りは 14 種。
7. ジョブの出力のファイル化(`[pipeline] log_dir`、v0.10.0)。端末に出しつつ、試行ごとに `<安全な ID>.<開始の時刻>.<試行の番号>.log` に残す。
8. `graph`(流れの図。`--format text|mermaid|dot`、`--state`、`-v`。`dot` は v0.10.0)。
9. `--dry-run`(ジョブを起動せず、ステートに書かず、計画を出す。`--assume <ジョブID>=<終了コード>`)。
10. `init`(定義のひな形。英日。既存のファイルは `--force` なしでは壊さない)。
11. シグナルの転送と中断の記録(UNIX は SIGHUP・SIGINT・SIGQUIT・SIGTERM、Windows は Ctrl+C)。
12. 補助のコマンド: `watch`(状態・所要時間・補足の表。`--follow [--interval <秒>]` で表示し直しを続ける。v0.10.0)、`reset`、`validate`、`convert`。
13. 終了コードの規約は `api-documentation.md` の 6 を参照。

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

## この intent(v0101-followups、v0.10.1)との関係

v0.10.0 の振り返りの課題 F1〜F5 を片づける整理・修正の作業で、新しい機能は足さない。場所の詳細は `code-quality-assessment.md` の「技術的負債と、この intent への影響」を参照。

| 課題 | 関わる利用者 | 現状の要点 |
|------|--------------|------------|
| F1 README の `watch --follow` の注意の書き直し | 障害対応の担当者 | エラーで止まったステートを見始めると、再開のジョブが試行を書く前の最初の 1 回で「止まった」と判定して終わる。README はこれを短い注意 1 つで書いている |
| F2 `log_dir` の名前と待ち切れの印 | ジョブの作者 | 予約名の比較に前後の空白が残る。読み取りのスレッドは「止める」印を読み取りの後にだけ見る |
| F3 `plan_simulate_perf.rs` の 150ms の上限 | 開発者(CI) | カバレッジの計測の下で揺れた。上限は下げない |
| F4 u6 の原因不明の失敗 | 開発者(CI) | コードに手がかりは無い。再び起きたら記録して調べる |
| F5 デモの場面 4 | デモを見せる人 | UNIX 系の端末では `watch --follow` が 1 回目から画面を消すため、`DEMO_PAUSE=1` で見せている前の出力が消える |

## 文書

利用者向けは `README.md`(日本語、953 行。必須の語句は CI の `docs-check` が検査)と `demo/README.md`。仕様書は `docs/spec/`(7 言語版 × 13 文書。`rust/` が Rust 版の構成を書く。v0.10.0 の機能の反映は一部だけ。`code-quality-assessment.md` を参照)。
