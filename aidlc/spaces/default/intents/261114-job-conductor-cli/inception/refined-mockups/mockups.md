# Mockups(コマンド体験の具体化)

上流: wireframes と user-flow(Rough Mockups)、stories(US1.1〜US7.1)、requirements(FR1〜FR7、NFR1〜NFR6)、team-practices。`conductor` は画面を持たない CLI のため、ここでは画面ではなく、コマンドの入出力と定義ファイルの書き方を、ストーリーごとに具体化する。表示文言は**イメージ**であり、確定した文言ではない。文言の言語は、環境のロケールに従う(英語と日本語を切り替える)。 [Q5]

## 1. 定義ファイル(`conductor.toml`)の例

US1.1、US1.2、US1.3 に対応する。キー名と値は、確認済みの回答に基づく。

```toml
[pipeline]
name = "production-nightly-flow"
description = "夜間バッチ"

[[job]]
id = "extract"
command = "python scripts/extract.py"
warning_exit_codes = [1]
# success_exit_codes を書かなければ、終了コード 0 のみが成功

[[job.transitions]]
on = "warning"
next = "slack_notify_and_skip"    # 警告のとき、間の transform と load をスキップして移る

[[job]]
id = "transform"
command = "python scripts/transform.py"
success_exit_codes = [0, 3]       # 書いた値だけが成功。0 を書かなければ 0 でも成功にならない

[[job]]
id = "load"
command = "python scripts/load.py"

[[job]]
id = "slack_notify_and_skip"
command = "python scripts/notify.py --msg='Warning in extraction'"

[[job.transitions]]
on = "success"
next = "@end"                     # パイプラインをここで終える
```

- `success_exit_codes` は、書かなければ 0 のみが成功で、書けば書いた値だけが成功になる。 [Q4]
- 遷移先は、記述順で後ろのジョブID、または `@end`(パイプラインをここで終える)に限る。エラー(`on = "error"`)には書けない。 [Q3]、requirements の FR1.3、FR1.4
- 遷移を書かない成功・警告は、記述順で次のジョブに進む。 requirements の FR1.3
- 定義ファイルは、既定で実行ディレクトリの `conductor.toml` を使い、`--config <パス>` で別のパスを指定できる。 [Q2]

## 2. `conductor run <job-id>`(US2.1〜US2.5)

### 2.1 成功(既定では `conductor` 自身の表示なし)

```
$ conductor run extract
(ジョブ本体の標準出力・標準エラーが、そのまま流れる)
$ echo $?
0
```

- 既定では、`conductor` 自身の表示は出さない。 [Rough Mockups の Q2、Q3]

### 2.2 `-v` / `--verbose` を付けたとき(表示を有効にする)

```
$ conductor run -v extract
[conductor] job=extract result=warning (exit code 1)
[conductor] transition: warning -> slack_notify_and_skip (skipped: transform, load)
$ echo $?
0
```

- `-v` / `--verbose` の1段階で、ジョブの結果、スキップ、遷移を表示する。 [Q1]
- 警告で終わったジョブは、`conductor` が終了コード 0 で返す。 [requirements の FR2.5]

### 2.3 スキップ対象のジョブ

```
$ conductor run transform
$ echo $?
0
```

- 何も表示せず、ジョブのコマンドも実行せず、終了コード 0 で終わる。 [Rough Mockups の Q3]
- `-v` を付けると、`[conductor] job=transform skipped` のような表示が出る(イメージ)。 [Q1]

### 2.4 エラー

```
$ conductor run extract
(ジョブ本体の出力)
$ echo $?
2
```

- エラーのときは、ジョブの終了コードをそのまま返す。 [requirements の FR2.5]
- ジョブのコマンドを起動できないときは、`conductor` 自身が原因を表示して 127 で、シグナルで終了したときは 128 + シグナル番号で終了する。 [Q7]

```
$ conductor run extract
conductor: error: cannot start command 'python': No such file or directory
$ echo $?
127
```

### 2.5 定義ファイルの誤り(`conductor` 自身のエラーは常に表示する)

```
$ conductor run transform
conductor: error: conductor.toml:21: job 'transform': transition target 'extract' is not after the current job
$ echo $?
(非0)
```

- 誤りのある定義では、ジョブを一切実行せず、ファイル名・行・原因を表示して異常終了する。 [Rough Mockups の Q5、Q7]、requirements の FR1.4

## 3. `conductor pipeline <pipeline-name>`(US3.1、US3.2)

```
$ conductor pipeline production-nightly-flow
(各ジョブの標準出力・標準エラーが、順に流れる)
$ echo $?
0
```

- ジョブを記述順に実行し、遷移ルールに従ってスキップ・ジャンプする。エラーが起きたら、そこで止まり、ジョブの終了コードで終了する。自動の再試行はしない。 [requirements の FR3.1、FR3.2]
- 一連の実行の表示のイメージは、個別実行に準じる。`-v` で結果・スキップ・遷移が出る。 [Q1]
- `@end` に到達したときは、その場で完了とし、ステートを初期化する。 [requirements の FR4.2]

## 4. `conductor watch`(US5.1)

```
$ conductor watch
JOB                    STATUS    NOTE
extract                warning   exit code 1
transform              skipped   by warning of 'extract'
load                   skipped   by warning of 'extract'
slack_notify_and_skip  success
```

- 現在の状態を、ジョブID、状態、補足の3列で、一覧として1回表示して終了する。 [Q6]、[Rough Mockups の Q4]
- 状態は、成功(success)・警告(warning)・スキップ(skipped)・エラー(error)・未実行(pending)のように、文字で表す。 [Q6]
- ステートが空のときは、空であることが表示され、エラーにならない。 [US5.1]

## 5. `conductor reset`(US6.1)

```
$ conductor reset
$ echo $?
0
```

- ステートがあれば初期化し、なくてもエラーにしない。 [US6.1]
- 既定では、`conductor` 自身の表示は出さない。 [Rough Mockups の Q2、Q3]

## 6. `conductor convert <file> --to <format>`(US7.1)

```
$ conductor convert conductor.yaml --to toml
(変換後の内容が、標準出力、または指定した出力に書き出される)
```

- キーと値の意味が同じものが得られる。コメントと記述順は保持されない。 [requirements の FR7.1]
- 出力先の指定の具体は、設計で定める。

## Assumptions & Open Questions

- 表示の文言(`[conductor] ...`、`conductor: error: ...`、`watch` の列見出し)は、確認の仕方を示すイメージであり、確定した仕様ではない。設計で定める。 [assumption]
- `convert` の出力先(標準出力か、ファイルか)は、設計で定める。 [assumption]
- 一連の実行(`conductor pipeline`)の表示のイメージは、個別実行に準じるものとして書いた。詳細は設計で定める。 [assumption]
