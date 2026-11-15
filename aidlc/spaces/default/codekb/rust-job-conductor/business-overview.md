# Business Overview

## 目的

`conductor` は、既存のジョブスケジューラ(cron など)から呼び出される、ジョブの流れの制御ツールである(`README.md`)。定義ファイル(TOML)にジョブと、結果(成功・警告・エラー)ごとの次の動きを書くと、ジョブを記述順に実行し、遷移の規則でスキップや終了を行い、実行状態(ステート)を覚える。エラーで止まったあとは、エラーのジョブから再開できる。単一の実行ファイルで、Linux・macOS・Windows で同じ結果を返す。

## 主な機能

| 機能 | 入口 | 内容 |
|------|------|------|
| 個別実行 | `conductor run <job-id>` | 1 ジョブを実行し、結果をステートに記録する(`crates/cli/src/run.rs`) |
| 一括実行 | `conductor pipeline <name>` | 記述順に全ジョブを実行する。エラーで停止し、再試行しない(`pipeline.rs`) |
| 状況表示 | `conductor watch` | ステートを表で 1 回表示する(`aux_commands.rs`) |
| 初期化 | `conductor reset` | ステートを消す |
| 形式変換 | `conductor convert <file> --to <yaml\|toml>` | 定義ファイルを YAML と TOML のあいだで変換する(`conductor-converter`) |

## 業務ルール(コードから読み取れるもの)

- ジョブは `command` を、シェルを介さずに直接起動する。現行は、空白で分けた文字列で、`'...'` と `"..."` で空白を含む引数を書く(BR1.1、`crates/runtime/src/split.rs`)。変数・`*`・`|`・`;`・`&&`・`>` は解釈しない。
- 終了コードは 成功(既定 `[0]`)・警告・エラーに分類する。成功コードに含まれない 0 は、conductor 自身のエラー(終了コード 1)。
- 遷移は `success`・`warning` だけに書け、遷移先は記述順で後ろのジョブか `@end`。
- 終了コード: 0 成功、64 定義の誤り(command の誤りを含む)、65 ステート、66 拒否、67 変換、68 使い方、70 内部、127 起動失敗、128+番号 シグナル(`crates/cli/src/exit.rs`)。
- ステートは定義ファイルと同じディレクトリの `<定義ファイル名>.state`(TOML)。完了で消す。

## 今回の変更(command-array)との関係

ジョブの `command` に、現行の文字列に加えて、文字列の配列(1 要素 = 1 引数、分割・展開なし)を許す予定である。影響する業務ルールは BR1.1(分け方)と、command の誤りの報告(終了コード 64)である。詳細は `architecture.md` と `code-quality-assessment.md` を参照。
