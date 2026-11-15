# ビジネス概要

## 目的

`conductor` は、既存のジョブスケジューラ(cron など)から呼び出される、ジョブの流れの制御ツールである。定義ファイル(TOML)に、ジョブと、結果(成功・警告・エラー)ごとの次の動きを書くと、`conductor` が記述順にジョブを実行し、遷移の規則に従ってスキップや終了を行い、実行の状態(ステート)をファイルに覚える。エラーで止まったあとは、エラーのジョブから再開できる(README.md 1-6 行)。

## 主な機能(利用者から見える動き)

| 機能 | 内容 | 根拠 |
|------|------|------|
| `run <ジョブID>` | ジョブ 1 つを実行する(個別実行)。起動前に、実行・スキップ・拒否を判断する | `crates/cli/src/run.rs` |
| `pipeline <名前>` | 定義の記述順に、ジョブを 1 つずつ実行する。エラーで止まり、再試行しない | `crates/cli/src/pipeline.rs` |
| `watch` | ステートを、ジョブ・状態・補足の 3 列の表で 1 回だけ表示する | `crates/cli/src/report/watch.rs` |
| `reset` | ステートを消す(`--run-id` で 1 つ、`--all` で全部) | `crates/cli/src/aux_commands.rs` |
| `validate` | 定義と全ジョブの `command` を検査する。実行も、ステートへの接触もしない | 同上 |
| `convert` | 定義を YAML と TOML のあいだで変換し、標準出力に書く | `crates/converter/src/lib.rs` |
| run ID | 実行単位ごとにステートを分ける(v0.5.0 以降) | `crates/core/src/run_id.rs` |
| 配列の `command` | 引数を配列で書く(v0.2.0 以降) | `crates/core/src/types.rs` の `Command` |

## 利用者と配布

- 利用者: 既存のスケジューラからバッチを動かす運用者。単一の実行ファイルを、Linux・macOS・Windows で同じ定義・同じ結果で使う。
- 配布: GitHub Releases(タグ `v*` で CI が 3 OS 向けのアーカイブと `SHA256SUMS` を作り、下書きのリリースを作る。人が確認して公開する)。署名・公証はしない(`.github/workflows/release.yml`)。
- 保証の範囲: ジョブの終了とステートへの記録のあいだに落ちると、ジョブが再実行されうる(at-least-once)。排他制御・自動の再試行・タイムアウトはない。ネットワークに接続せず、利用状況も収集しない。

## 非機能の性質

- 表示は、環境のロケールで英語・日本語を切り替える。色は UNIX 系の端末の `watch` だけ。
- 入力に由来する文字列は、制御文字をエスケープして表示する。外部の入力でパニックしない。
- 終了コードは、どの OS でも 0〜255。conductor 自身のエラーは 64〜68・70(詳細は api-documentation.md)。

## 今回の意図(readme-reorg)との関係

README.md の構成を整理する(run ID の記述の集約、長い表のセルの短縮、節の並べ替え、目次とクイックスタートの追加)。意味は変えない。CI の `docs-check` が README の 7 語を要求する(code-quality-assessment.md)。README の主張の照合結果は、`inception/reverse-engineering/developer-scan.md` の Handoff Summary にある。
