# 08. 出力と多言語化

この章は、`conductor` が出す文字列の全文、出力先、言語の決め方、`watch` の表、エスケープの規則を規定する。文言は、この章の表のとおりに(文字単位で)出力する。コマンドラインは [04-cli.md](04-cli.md)、終了コードは [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md) を参照する。

## 1. 出力先の使い分け

表示は、処理の流れが作る「表示する内容」を、文言に直して、水準に従って出す。水準は 4 つある。

| 水準 | 出力先 | 出す条件 | 書き込み失敗 |
|------|--------|----------|--------------|
| `Error` | 標準エラー | 常に | 無視(本筋を止めない) |
| `Notice` | 標準エラー | 常に(エラーではない知らせ。リトライの前の知らせと、v0.9.0 以降の `init` の成功の知らせ) | 無視 |
| `Verbose` | 標準エラー | `-v` のときだけ | 無視 |
| `Output` | 標準出力 | 常に | 呼び出し元に失敗を返す(`watch` は 65、`convert` は 67、v0.9.0 以降の `graph`・`--dry-run`・`init --config -` は 65 で終わる) |

- どの文言も、最後に改行(LF、`\n`)を含む。複数の誤りは 1 行ずつ。
- 書き込みのたびに flush する(行ごとに、すぐに見える)。
- 標準出力に出すのは、`watch` の表(と `no state`)、`--help`、`--version`、`convert` の変換結果と、v0.9.0 以降の `graph` の図、`--dry-run` の計画、`init --config -` のひな形だけ。この 3 つは、組み立て済みの文字列を、そのまま 1 回で書く。
- ジョブ本体の標準出力・標準エラーは `conductor` が加工せず、そのまま流れる(`conductor` はジョブの標準入出力を継承させる)。`conductor` の表示は標準エラーなので、標準出力にはジョブの出力だけが残る。
- 成功・スキップで `-v` なしのときは、表示が一度もなく、言語も色も判定されない(必要になった最初の時点まで判定を遅らせる)。この遅延は、観測できる差(環境変数の読み取り順)を生まないなら、実装に任せてよい。

### 1.1 水準ごとのイベント

| 水準 | イベント |
|------|----------|
| Error | 定義の誤り、`command` の誤り、使い方の誤り(+使い方全文)、ステートの誤り、定義との食い違い、`reset --all` の警告(`conductor: warning:`)、拒否、起動失敗、成功コードに含まれない 0、変換の誤り、標準出力に書けない、完了を判定できない、シグナルの準備の失敗、内部の誤り。v0.9.0 以降: `.env` の誤り、`.env` の重複キーの警告(`conductor: warning:`)、`init` の既存・作れない・標準出力に書けない |
| Notice | リトライの前の知らせ(`conductor: retry:`)。v0.9.0 以降: `init` の成功(作った、置き換えた) |
| Verbose | ジョブの結果、スキップ、遷移、中断、`validate` が通った |
| Output | `watch` の表、`watch` の空、`--help`、`--version`。v0.9.0 以降: `graph` の図、`--dry-run` の計画、`init --config -` のひな形 |

## 2. `-v`(`--verbose`)

- `run`・`pipeline`・`validate`、v0.9.0 以降は `graph` が受け付ける(`graph` では、標準出力の図に詳細の行を足す。11 章)。1 回だけ(2 回は 68)。
- 常に表示するものは、`-v` に関係なく出る(エラー、起動失敗、成功コードに含まれない 0、リトライの知らせ)。
- `-v` の行は、先頭が `[conductor] `(角かっこ、`conductor`、角かっこ閉じ、半角空白)。

### 2.1 `-v` の行の形式

| 種類 | 形式(英語) | 形式(日本語) |
|------|-------------|----------------|
| ジョブの結果 | `[conductor] job=<ID> result=<success\|warning\|error\|interrupted> (<補足>)` | 同左(補足だけ日本語) |
| スキップ | `[conductor] job=<ID> skipped (<理由>)` | `[conductor] job=<ID> スキップ (<理由>)` |
| 遷移 | `[conductor] transition: <success\|warning> -> <遷移先ID または @end>`。スキップしたジョブがあれば、続けて ` (skipped: <ID>, <ID>)` | 同じ形で、` (スキップ: <ID>, <ID>)` |
| 中断 | `[conductor] interrupted by signal <N>` | `[conductor] シグナルで中断 <N>` |
| `validate` 成功 | `[conductor] definition is valid (<N> job)` / `(<N> jobs)`(`N == 1` のとき `job`、それ以外は `jobs`) | `[conductor] 定義は正しいです(ジョブ <N> 個)` |

- `job=`・`result=` と、`success`・`warning`・`error`・`interrupted` の状態語、`transition:`・`->`、遷移先の `@end` は、言語によらず英語(固定の語)。
- 遷移行の ` (skipped: ...)` の ID は、`, `(コンマ+半角空白)で区切る。ID は、名前用エスケープ(9 章)を通す。

結果行の補足 `(<補足>)`:

| 状況 | 英語 | 日本語 |
|------|------|--------|
| ジョブの終了コード | `exit code <C>` | `終了コード <C>` |
| `result=error` で、ジョブの終了コード `C` と `conductor` の終了コード `E` が異なるとき(例: `C = 0` で成功コードに含まれない、`C = 256`) | `exit code <C>, conductor exits with <E>` | `終了コード <C>, conductor の終了コード <E>` |
| ジョブがシグナルで終了 | `terminated by signal <N>` | `シグナルで終了 <N>` |
| 起動できなかった | `could not start` | `起動できませんでした` |
| 中断(ジョブは起動済み) | `interrupted by signal <N>` | `シグナルで中断 <N>` |
| 中断(起動前) | `interrupted by signal <N>, not started` | `シグナルで中断 <N>, 起動せず` |

- 区切りは、言語によらず ASCII の `, `(コンマ+半角空白)。日本語でも全角の読点は使わない。起動前の中断の `, not started` / `, 起動せず` も同じ。
- 補足が出る場面の決め方は、`result` が `error` のとき、`C` が conductor の終了コード `E` と同じなら 1 つ目の形、異なれば 2 つ目の形。

`result` の決め方: ジョブの分類が成功 → `success`、警告 → `warning`、エラー → `error`、分類なし(起動前の中断)→ `interrupted`。起動済みで中断された場合は、分類がエラーなので `error` として表示する。

スキップの理由(`-v` のスキップ行)と、拒否の理由は、別の表(5 章)。

実際の出力(`LANG=C`):

```text
$ conductor pipeline -v nightly --config c.toml --run-id r1
[conductor] job=extract result=warning (exit code 1)
[conductor] transition: warning -> load (skipped: transform)
[conductor] job=transform skipped (skipped by a transition)
[conductor] job=load result=success (exit code 0)

$ conductor run -v a --config e.toml        # success_exit_codes = [5] で、ジョブが 0 で終了
[conductor] job=a result=error (exit code 0, conductor exits with 1)
conductor: error: job 'a': exit code 0 is not listed in success_exit_codes

$ conductor pipeline -v p --config d.toml   # ジョブが SIGKILL で終了
[conductor] job=a result=success (exit code 0)
[conductor] job=b result=error (terminated by signal 9)

$ conductor validate -v
[conductor] definition is valid (3 jobs)
```

出す順序: ジョブの結果行・遷移行・中断行は、ジョブの実行後(ステートの書き込みの前)に出す。起動の失敗と、成功コードに含まれない 0 は、結果行のあとに、常に出す誤り行。

## 3. 誤りの行の形式

```text
conductor: error: <場所>: <種類の文>[: <原因>]
```

- 先頭 `conductor: error: ` は言語によらず同じ。警告は `conductor: warning: `、リトライの知らせは `conductor: retry: `。
- `<場所>`: 定義の誤りは `<ファイル>` または `<ファイル>:<行>`、`.env` の誤りと警告(v0.9.0 以降)は `<.env のパス>` または `<.env のパス>:<行>`(パス用エスケープ)、`command` の誤りは `<定義ファイル>: job '<ID>'`、ステート・食い違いは `<ステートのパス>`、拒否・起動失敗・成功コードに含まれない 0 は `job '<ID>'`、変換の誤りは `<ファイル>` または `<ファイル>:<行>`。
- 使い方の誤りだけは場所を持たず、`conductor: error: <文>[: '<対象>']` のあとに空行と使い方全文(8 章)を続ける。

## 4. メッセージカタログ

「英語」「日本語」の列は、原文のとおり出す。`{…}`・`<…>` は実行時に入る値(エスケープ後。9 章)。

### 4.1 固定の英語の文(ロケールを判定しない)

| キー | 文 | 終了コード |
|------|----|------------|
| `INTERNAL_ERROR`(想定外の内部失敗) | `conductor: error: internal error` | 70 |
| `SIGNAL_SETUP_FAILED` | `conductor: error: failed to set up signal handling` | 70 |
| `--version` | `conductor {版}`(v0.9.2 では `conductor 0.9.2`) | 0 |

### 4.2 使い方の誤り

| 種類 | 英語 | 日本語 |
|------|------|--------|
| `UnknownCommand` | `unknown command` | `知らないコマンドです` |
| `MissingArgument` | `missing argument` | `引数が足りません` |
| `UnexpectedArgument` | `unexpected argument` | `余分な引数があります` |
| `UnknownOption` | `unknown option` | `知らないオプションです` |
| `OptionNotAllowed` | `option not allowed for this command` | `このコマンドでは使えないオプションです` |
| `MissingValue` | `option requires a value` | `オプションに値がありません` |
| `InvalidValue` | `invalid value` | `値が正しくありません` |
| `UnknownJob` | `job is not defined in the definition file` | `定義にないジョブIDです` |

表示: 対象あり `conductor: error: <文>: '<対象>'`、なし `conductor: error: <文>`。対象は名前用エスケープ(64 文字で切る)。終了コードはすべて 68。

### 4.3 定義の誤り

表示は `conductor: error: <ファイル>[:<行>]: <種類の文>: <原因>`。`<原因>` は、定義の検証が作る日本語の固定の文で、言語を切り替えない(英語ロケールでも日本語のまま)。原因の文の一覧は [03-definition-format.md](03-definition-format.md) にある。この章では、種類の文だけを規定する。終了コードはすべて 64。

| 識別子 | 英語 | 日本語 |
|--------|------|--------|
| `fileMissing` | `definition file does not exist` | `定義ファイルが存在しません` |
| `unreadable` | `cannot read definition file` | `定義ファイルを読めません` |
| `tooLarge` | `definition file is too large` | `定義ファイルが大きすぎます` |
| `syntax` | `syntax error` | `文法の誤りです` |
| `typeMismatch` | `wrong value type` | `値の型が正しくありません` |
| `unknownKey` | `unknown key` | `定義にないキーです` |
| `versionMissing` | `format_version is missing` | `書式の版がありません` |
| `versionUnsupported` | `unsupported format_version` | `対応していない書式の版です` |
| `unknownTarget` | `transition target job is not defined` | `遷移先のジョブが定義にありません` |
| `errorTransition` | `no transition can be written for the error result` | `エラー結果には遷移先を書けません` |
| `backwardOrSelfTarget` | `transition target must be a later job or @end` | `遷移先は、記述順で後ろのジョブまたは @end のみです` |
| `overlappingExitCodes` | `success and warning exit codes overlap` | `成功と警告の終了コードが重なっています` |
| `zeroAsWarning` | `0 cannot be a warning exit code` | `0 は警告の終了コードにできません` |
| `pipelineNameMismatch` | `pipeline name does not match the definition` | `パイプライン名が定義と一致しません` |
| `duplicateJobId` | `duplicate job id` | `ジョブIDが重複しています` |
| `invalidJobId` | `invalid job id` | `ジョブIDが正しくありません` |
| `noJobs` | `no jobs are defined` | `ジョブが 1 つもありません` |
| `emptyField` | `required value is empty` | `必須の値が空です` |
| `duplicateTransition` | `duplicate transition for the same result` | `同じ結果の遷移が重複しています` |
| `invalidEnvName` | `invalid environment variable name` | `環境変数の名前が正しくありません` |
| `retryNotTable` | `retry must be a table` | `retry は表で書きます` |
| `retryMissingKey` | `a required key of retry is missing` | `retry の必須のキーがありません` |
| `retryExitCodesEmpty` | `retry exit_codes is empty` | `retry の終了コードの一覧が空です` |
| `retryExitCodeInvalid` | `retry exit_codes has an invalid exit code` | `retry の終了コードが正しくありません` |
| `retryExitCodesOverlap` | `retry exit_codes overlap the success or warning exit codes` | `retry の終了コードが、成功または警告の終了コードと重なっています` |
| `retryMaxInvalid` | `retry max is invalid` | `retry の最大回数が正しくありません` |
| `retryDurationInvalid` | `retry interval is invalid` | `retry の間隔が正しくありません` |
| `retryBackoffIncomplete` | `retry multiplier and max_interval must be given together or not at all` | `retry の倍率と間隔の上限は、両方を書くか、両方とも書きません` |
| `retryMultiplierInvalid` | `retry multiplier is invalid` | `retry の倍率が正しくありません` |
| `retryMaxIntervalTooSmall` | `retry max_interval is smaller than interval` | `retry の間隔の上限が、間隔より小さくなっています` |
| `reservedEnvName`(v0.9.0 以降) | `reserved environment variable name` | `予約された環境変数の名前です` |
| `envNameCaseConflict`(v0.9.0 以降) | `environment variable names differ only in letter case` | `大文字小文字だけが違う環境変数の名前があります` |
| `invalidEnvValue`(v0.9.0 以降) | `invalid environment variable value` | `環境変数の値が正しくありません` |

実出力:

```text
$ conductor pipeline wrongname --config c.toml        # LANG=C
conductor: error: c.toml:3: pipeline name does not match the definition: 指定されたパイプライン名が、定義の `name` と一致しません
$ LANG=ja_JP.UTF-8 conductor validate --config nonexist.toml
conductor: error: nonexist.toml: 定義ファイルが存在しません: 指定の場所にファイルがありません
```

定義に複数の誤りがあるとき、検出した誤りを 1 行ずつ出す(順序と検出の範囲は [03-definition-format.md](03-definition-format.md))。

### 4.4 `command` の誤り

表示: `conductor: error: <定義ファイル>: job '<ID>': <文>`(原因は付けない)。全ジョブを検査し、誤りのあるジョブごとに 1 行。終了コード 64。

| 種類 | 英語 | 日本語 |
|------|------|--------|
| `UnclosedQuote` | `command has an unclosed quote` | `command の引用符が閉じていません` |
| `Empty` | `command is empty` | `command が空です` |

`command` の解釈の規則は [03-definition-format.md](03-definition-format.md) と [05-execution-flow.md](05-execution-flow.md) にある。

### 4.5 ステートの誤り

表示: `conductor: error: <ステートのパス>: <種類の文>: <原因>[<補足>]`。`<原因>` は、ステートの読み書きが作る日本語の固定の文で、言語を切り替えない(一覧は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md) 7.2)。終了コードはすべて 65。

| 種類 | 英語 | 日本語 |
|------|------|--------|
| `Unreadable` | `cannot read state file` | `ステートファイルを読めません` |
| `TooLarge` | `state file is too large` | `ステートファイルが大きすぎます` |
| `Corrupt` | `state file is corrupt` | `ステートファイルが壊れています` |
| `VersionMismatch` | `state file version is not supported` | `ステートファイルの版が合いません` |
| `WriteFailed` | `cannot write state file` | `ステートファイルを書き込めません` |
| `RemoveFailed` | `cannot remove state file` | `ステートファイルを削除できません` |

`<補足>`(文脈による):

| 文脈 | 補足の付き方 | 英語 | 日本語 |
|------|----------------|------|--------|
| `Read`(run ID なしで読んだ) | 種類が `Unreadable`/`TooLarge`/`Corrupt`/`VersionMismatch` のときだけ `; <案内>` | ``run `conductor reset` to discard the state`` | `` `conductor reset` でステートを初期化できます `` |
| `ReadRunId`(run ID つきで読んだ) | 同上 | ``run `conductor reset --run-id <ID>` to discard the state``(`<ID>` は、文字どおり `<ID>` という文字列) | `` `conductor reset --run-id <ID>` でステートを初期化できます `` |
| `JobNotStarted` | 常に ` (<文>)` | `the job was not started` | `ジョブは起動していません` |
| `JobExecuted` | 常に ` (<文>)` | `the job has already been executed` | `ジョブは実行されました` |
| `Plain`(`reset`・`watch` の一部) | なし | | |

文脈の使い分け:

- `watch`: run ID なしなら `Read`、ありなら `ReadRunId`(案内つき)。
- `reset`・`reset --all`: `Plain`(補足なし)。
- `run`・`pipeline`: 読み込みは `Read`/`ReadRunId`。書き込み・削除は、ジョブを起動する前の失敗なら `JobNotStarted`、ジョブの実行後の失敗なら `JobExecuted`。

実出力:

```text
conductor: error: c.toml.state: state file is corrupt: TOML として読めません; run `conductor reset` to discard the state
conductor: error: c.toml.state: ステートファイルが壊れています: TOML として読めません; `conductor reset` でステートを初期化できます
```

### 4.6 定義との食い違い

表示: `conductor: error: <ステートのパス>: <MISMATCH>: <食い違いの種類>; <案内>`。案内は run ID の有無で、4.5 の `Read` / `ReadRunId` の案内と同じ文。終了コード 65。

| 種類 | 英語 | 日本語 |
|------|------|--------|
| `MISMATCH` | `state does not match the definition` | `ステートが定義と食い違っています` |
| `PipelineName` | `pipeline name` | `パイプライン名` |
| `JobIds` | `job ids` | `ジョブID` |
| `JobOrder` | `job order` | `ジョブの並び` |

### 4.7 拒否・起動失敗・そのほか

| キー | 表示 | 英語 | 日本語 | 終了コード |
|------|------|------|--------|------------|
| `REJECTED` | `conductor: error: job '<ID>': <REJECTED>: <理由>` | `cannot run this job now` | `このジョブは、いま実行できません` | 66 |
| 拒否の理由 `OutOfOrder` | | `earlier jobs have not finished (out of order)` | `前のジョブが終わっていません(順序外)` | |
| 拒否の理由 `SkipTarget` | | `skip target` | `スキップの対象です` | |
| 拒否の理由 `AfterEnd` | | `after @end` | `@end より後ろです` | |
| 拒否の理由 `AlreadyExecuted` | | `already executed` | `実行済みです` | |
| `CANNOT_START` | `conductor: error: job '<ID>': <文> '<コマンド名>'` | `cannot start command` | `コマンドを起動できません` | 127 |
| `SUCCESS_CODE_NOT_LISTED` | `conductor: error: job '<ID>': <文>` | `exit code 0 is not listed in success_exit_codes` | `終了コード 0 は、success_exit_codes に含まれていません` | 1 |
| `OUTPUT_FAILED` | `conductor: error: <文>`(場所なし) | `cannot write to standard output` | `標準出力に書き込めません` | 65(`watch`)/ 67(`convert`) |
| `COMPLETION_UNDETERMINED` | `conductor: error: <ステートのパス>: <文>` | `cannot determine whether the pipeline is complete` | `パイプラインが完了したかを判定できません` | 65 |

`CANNOT_START` の `<コマンド名>` は、`command` を解釈して得た実行ファイル名(解釈できなければ `command` の先頭の語。それもなければ空)を、名前用エスケープしたもの。

実出力:

```text
conductor: error: job 'load': cannot run this job now: earlier jobs have not finished (out of order)
conductor: error: job 'step3': cannot start command 'nonexistent_prog_x'
```

### 4.8 変換の誤り

表示: `conductor: error: <ファイル>[:<行>]: <種類の文>: <原因>[ (<AT_KEY>: <キーの位置>)]`。`<原因>` は、変換が作る日本語の固定の文で、言語を切り替えない(一覧は 10 章 7.3)。`AT_KEY` は英語 `at`、日本語 `場所`。キーの位置があるときだけ付く(` (at: job[2].transitions[0])`)。終了コードはすべて 67。

| 種類 | 英語 | 日本語 |
|------|------|--------|
| `fileMissing` | `input file does not exist` | `入力ファイルが存在しません` |
| `unreadable` | `cannot read input file` | `入力ファイルを読めません` |
| `tooLarge` | `input is too large` | `入力が大きすぎます` |
| `syntax` | `syntax error` | `文法の誤りです` |
| `unrepresentable` | `value cannot be represented in the target format` | `相手の形式で表せない値があります` |
| `rootNotMap` | `top level is not a map` | `最上位がキーと値の集まり(マップ)ではありません` |

実出力:

```text
$ conductor convert nofile --to toml
conductor: error: nofile: input file does not exist: 指定の場所にファイルがありません
```

### 4.9 リトライの知らせ(`Notice`。常に標準エラー)

形式: `conductor: retry: <本文>`。

| 言語 | 本文 |
|------|------|
| 英語 | `job '{ID}' exited with code {最後の終了コード}; retry {n} of {max} in {秒} seconds` |
| 日本語 | `ジョブ '{ID}' が終了コード {最後の終了コード} で終わりました。{秒} 秒後にリトライします({n}/{max} 回目)` |

- `{n}` は、1 から始まる「何回目のリトライか」。`{max}` は最大回数。`{秒}` は、次の待ち時間(整数の秒。リトライの待ち時間は整数秒で扱う)。1 秒でも `seconds`(単複の区別はしない)。
- 待機の前に必ず出す。`{最後の終了コード}` は、直前の試行の終了コード。終了コードがなければ 0 と表示する。
- 待ち時間の求め方は [05-execution-flow.md](05-execution-flow.md)(リトライ)に従う。

実出力:

```text
conductor: retry: job 'load' exited with code 75; retry 1 of 2 in 1 seconds
conductor: retry: ジョブ 'load' が終了コード 75 で終わりました。1 秒後にリトライします(1/2 回目)
```

### 4.10 `reset --all` の警告

形式(標準エラー。終了コードは 0):

```text
conductor: warning: <RESET_ALL_KEPT>
  <ファイル名>
  <ファイル名>
```

| 言語 | `RESET_ALL_KEPT` |
|------|------------------|
| 英語 | ``kept state files that could not be read (remove one with `conductor reset --run-id <ID>`):`` |
| 日本語 | ``読めなかったため、次のステートファイルは残しました(`conductor reset --run-id <ID>` で個別に消せます):`` |

ファイル名の行は、先頭に半角空白 2 文字を置き、パス用エスケープ(切らない)を通したファイル名(ディレクトリを含まない名前)を書く。1 行 1 ファイル。

### 4.11 `-v` の補助語

| キー | 英語 | 日本語 |
|------|------|--------|
| `EXIT_CODE` | `exit code` | `終了コード` |
| `CONDUCTOR_EXITS_WITH` | `conductor exits with` | `conductor の終了コード` |
| `KILLED_BY_SIGNAL` | `terminated by signal` | `シグナルで終了` |
| `COULD_NOT_START` | `could not start` | `起動できませんでした` |
| `INTERRUPTED_BY_SIGNAL` | `interrupted by signal` | `シグナルで中断` |
| `NOT_STARTED` | `not started` | `起動せず` |
| `SKIPPED_WORD`(スキップ行の語) | `skipped` | `スキップ` |
| `SKIPPED_LIST`(遷移行の `(…: ID, …)`) | `skipped` | `スキップ` |

組み立ては、`<語> <数>`(半角空白 1 つで連結)。日本語でも数との連結は半角空白。

### 4.12 `.env` の誤りと警告(v0.9.0 以降)

**誤り**(標準エラー。終了コード 64): 誤り 1 件につき 1 行。

```text
conductor: error: <ファイル>[:<行>]: <種類の文>: <理由>
```

- `<ファイル>` は解決済みのパス(パス用エスケープ。切らない)。`<行>` は、行番号があるときだけ(無い誤り: `fileMissing`、`unreadable`、`tooLarge`、`notUtf8`)。
- 種類の文と理由は、**言語に従う**(定義の誤りの `<原因>` が常に日本語なのと違う)。日本語は 03 の 7.7 の表の「日本語の種類の文」と「日本語の理由」。英語は次の表。
- 先頭の最大 20 件を表示し、超えたら、最後に 1 行 `conductor: error: and <N> more`(日本語 `conductor: error: ほか <N> 件`)を出す(`<N>` = 残りの件数)。
- **値は、どの行にも含めない**(名前も含めない)。

| 識別子 | 種類の文(英語) | 理由(英語) |
|--------|-----------------|--------------|
| `fileMissing` | `environment file does not exist` | ``the file named in `env_file` was not found`` |
| `unreadable` | `cannot read environment file` | ``the file named in `env_file` cannot be read`` |
| `tooLarge` | `environment file is too large` | `the file is larger than the limit (1 MiB)` |
| `notUtf8` | `environment file is not UTF-8` | `write the file in UTF-8` |
| `tooManyEntries` | `environment file has too many entries` | `the number of entries exceeds the limit (10,000)` |
| `invalidName` | `invalid environment variable name` | ``a name starts with an ASCII letter or `_` and has only ASCII letters, digits and `_` `` |
| `reservedName` | `reserved environment variable name` | ``names starting with `CONDUCTOR_` cannot be written`` |
| `caseConflict` | `environment variable names differ only in letter case` | `the file has names that differ only in letter case` |
| `missingEquals` | `not in KEY=VALUE form` | ``write a line as `KEY=VALUE` `` |
| `unterminatedQuote` | `unterminated quote` | `the quote is not closed on the same line` |
| `invalidEscape` | `invalid escape sequence` | ``only `\n`, `\r`, `\t`, `\"` and `\\` can be escaped in double quotes`` |
| `trailingGarbage` | `unexpected characters after the closing quote` | `only spaces and a comment can follow the closing quote` |
| `nulInValue` | `value contains NUL` | `a value cannot contain NUL (U+0000)` |

(理由の中の `` ` `` は、文の一部である。)日本語の種類の文と理由は、03 の 6.7 の表。

実出力(英語。`x.env` の 3 行目に `=` がない場合):

```text
conductor: error: x.env:3: not in KEY=VALUE form: write a line as `KEY=VALUE`
```

**警告**(標準エラー。`validate` だけが出す。重複キー): 警告 1 件につき 1 行。誤りではないので、終了コードに影響しない(`validate` の終了コードは 0)。

```text
conductor: warning: <ファイル>:<後ろの行>: duplicate key `<名前>` (the last one wins)
conductor: warning: <ファイル>:<後ろの行>: 重複キー `<名前>`(後ろの値を使います)
```

`<名前>` は名前用エスケープ(64 文字で切る)。例: ``conductor: warning: x.env:5: duplicate key `A\nB` (the last one wins)``(名前が `A<LF>B` のとき)。警告は、ファイルの読み取り順、ファイルの中では行の順に出す。

### 4.13 `init` のメッセージ(v0.9.0 以降)

パス(`<path>`)はパス用エスケープ(切らない)。成功は標準エラー(通知の水準)、誤りは `conductor: error: ` つきの標準エラー。

| 場面 | 英語 | 日本語 | 終了コード |
|------|------|--------|------------|
| 作った(`created`) | `created <path>` | `<path> を作成しました` | 0 |
| 置き換えた(`replaced`) | `replaced <path>` | `<path> を置き換えました` | 0 |
| 既にある(`alreadyExists`) | `conductor: error: <path> already exists (use --force to overwrite)` | `conductor: error: <path> は既にあります(上書きするには --force)` | 69 |
| 親のディレクトリが無い(`parentMissing`) | `conductor: error: cannot create <path>: the parent directory does not exist` | `conductor: error: <path> を作れません: 親のディレクトリがありません` | 65 |
| 作る先がディレクトリ(`isDirectory`) | `conductor: error: cannot create <path>: it is a directory` | `conductor: error: <path> を作れません: ディレクトリです` | 65 |
| 権限がない(`permissionDenied`) | `conductor: error: cannot create <path>: permission denied` | `conductor: error: <path> を作れません: 書く権限がありません` | 65 |
| そのほか(`otherIo`) | `conductor: error: cannot create <path>: I/O error` | `conductor: error: <path> を作れません: 入出力の誤りです` | 65 |
| `--config -` で、標準出力に書けない | `conductor: error: cannot write the template to standard output` | `conductor: error: ひな形を標準出力に書けません` | 65 |

OS のエラーの文は含めない。`--config -` の成功では、ひな形だけを標準出力に出し、成功のメッセージは出さない。

## 5. スキップ・拒否の理由の対応

| 理由 | `-v` のスキップ行 `(…)` 英語 | 日本語 | 拒否の理由(英語 / 日本語) |
|------|--------------------------------|--------|------------|
| `SkipTarget` | `skipped by a transition` | `遷移によりスキップ` | `skip target` / `スキップの対象です` |
| `AfterEnd` | `skipped after @end` | `@end の後ろのためスキップ` | `after @end` / `@end より後ろです` |
| `AlreadyExecuted` | `already executed` | `実行済み` | `already executed` / `実行済みです` |
| `OutOfOrder` | `out of order` | `順序外` | `earlier jobs have not finished (out of order)` / `前のジョブが終わっていません(順序外)` |

起動前の判断(実行・スキップ・拒否)の決め方は [05-execution-flow.md](05-execution-flow.md)。

## 6. `watch` の表

### 6.1 構成

出力(標準出力):

```text
<Pipeline|パイプライン>: <パイプライン名>
Run ID: <ID>                       ← run ID があるときだけ。見出しは言語によらず "Run ID"
<ジョブ見出し>  <状態見出し>  <補足見出し>
<ジョブID>  <状態>  <補足>
...
<完了|completed>                  ← ステートが完了のときだけ
```

| 要素 | 英語 | 日本語 |
|------|------|--------|
| パイプライン行の見出し | `Pipeline` | `パイプライン` |
| run ID 行の見出し | `Run ID` | `Run ID` |
| 列見出し(ジョブ) | `JOB` | `ジョブ` |
| 列見出し(状態) | `STATUS` | `状態` |
| 列見出し(補足) | `NOTE` | `補足` |
| 完了を示す行 | `completed` | `完了` |
| ステートなし(ファイルなし・長さ 0) | `no state (nothing has run yet)` | `ステートはありません(まだ実行されていません)` |

- パイプライン名・run ID・ジョブIDは、名前用エスケープ(9 章。64 文字で切り、制御文字を `\u{..}` に)を通す。
- パイプライン行は `<見出し>: <名前>`(コロンのあとに半角空白 1 つ)、run ID 行は `Run ID: <ID>`。
- ステートなしの文は、表の代わりにこの 1 行だけ(末尾改行つき)を標準出力に出す。
- 行はすべて LF で終わる。

### 6.2 列と整形

状態の語(言語によらず英語): `pending`, `success`, `warning`, `error`, `skipped`。

補足の決め方:

| 状態 | 補足 |
|------|------|
| `success` / `warning` / `error` | 終了コードがあれば `exit code <C>`(日本語 `終了コード <C>`)。なければ空 |
| `skipped` | `skip_reason` が `ByEnd` → `skipped after @end` / `@end の後ろのためスキップ`。`ByTransition` → `skipped by a transition` / `遷移によりスキップ`。なければ空 |
| `pending` | 空 |

整形の規則:

- 列は、2 つの半角空白で区切る。
- ジョブの列の幅 = 見出しと、全ジョブID(エスケープ後)の表示幅の最大。状態の列の幅 = 見出しと、全状態語(5 語の固定集合ではなく、実際に現れる語と見出し)の表示幅の最大。
- 表示幅は、文字ごとに、ASCII(U+0000〜U+007F)なら 1、それ以外は 2 として足す近似(全角を 2 とみなす)。実際の端末幅(結合文字、絵文字など)は考慮しない。
- 見出し行: ジョブ見出しと状態見出しを、右に半角空白を足して列の幅にそろえ、補足見出しは埋めない。行末の空白は取らない(補足見出しは空でないので、実際には取る空白がない)。
- データ行: ジョブIDを埋め(列の表示幅にそろえる)、状態語を埋め(埋めは状態語の文字数で数える。状態語は ASCII)、補足を続ける。行全体の行末の空白を削る(補足が空なら、状態語のあとの埋めと区切りも削られる)。
- `completed`: ステートの `completed` が真のときだけ、表の下に 1 行足す(`completed` / `完了`)。

### 6.3 色

- 色を使う条件(最初に必要になったとき一度だけ判定): UNIX 系の OS である(Windows は常に色なし)、かつ環境変数 `NO_COLOR` が未設定または空、かつ標準出力が端末。
- 色は、状態語だけに、ANSI の SGR で付ける: `ESC[<番号>m<語>ESC[0m`(ESC は U+001B)のあとに埋めの空白。`success` = 32(緑)、`warning` = 33(黄)、`error` = 31(赤)、`skipped` = 36(シアン)、`pending` = 色なし(制御列を付けない)。
- 列の幅は、色の制御文字を除いた文字で決める(状態語の埋めの空白は、制御列の外側に置く)。ファイルやパイプへの出力には制御文字を含めない。状態は色によらず文字でも表す。

実出力(`LANG=C`):

```text
$ conductor watch
Pipeline: hello
JOB    STATUS   NOTE
step1  success  exit code 0
step2  success  exit code 0
step3  error
```

run ID つき・完了(英語 / 日本語):

```text
Pipeline: nightly
Run ID: r1
JOB        STATUS   NOTE
extract    warning  exit code 1
transform  skipped  skipped by a transition
load       success  exit code 0
completed
```

```text
パイプライン: nightly
Run ID: r1
ジョブ     状態     補足
extract    warning  終了コード 1
transform  skipped  遷移によりスキップ
load       success  終了コード 0
完了
```

(日本語の表で `ジョブ` は表示幅 6、列の幅は `transform` の 9 なので、埋めは 3 空白、区切りの 2 空白と合わせて 5 空白になる。`状態` は幅 4、列の幅 7 で、同様に 5 空白。)

## 7. ロケールの決め方

- 環境変数 `LC_ALL`, `LC_MESSAGES`, `LANG` を、この順に見る。最初に空でない値を採り、その値が `ja` で始まれば日本語、それ以外は英語。どれも未設定または空なら英語。値は大文字小文字を区別する(`JA_JP` は英語)。値が文字として解釈できない(UTF-8 でない)ときは、未設定と同じ扱い。
- 最初の空でない値で決まり、後の変数は見ない(`LC_ALL=C` かつ `LANG=ja_JP` は英語。`LC_ALL=` が空で `LANG=ja_JP.UTF-8` なら日本語)。
- 判定は、1 回の実行につき、最初に表示が必要になったときだけ行い、結果を保つ。
- ロケールを判定しない固定の文: `--version`、内部の誤りの文、シグナルの準備の失敗の文(英語のみ)。
- 言語によらず英語: 状態語、`job=`・`result=`、`Run ID`、`conductor: error: `・`conductor: warning: `・`conductor: retry: `・`[conductor] ` の接頭辞、`--help` の中のオプション名。定義・ステート・変換の誤りの `<原因>` は日本語のまま。
- 「実装に任せてよい点」: ロケールの判定にオペレーティングシステムのロケール API を使わず、上の 3 つの環境変数だけで決めること(これは必須)。

## 8. 使い方(`--help`)全文

標準出力、終了コード 0。使い方の誤りのときは、誤りの行と空行のあとに、同じ全文を標準エラーに出す。言語は 7 章で決める。各全文の末尾は改行で終わる。行の先頭の空白は、半角空白 2 文字(オプションの説明の折り返しの行は 22 文字)。

### 8.1 英語

v0.9.0 以降の全文(v0.8.0 以前は、`graph`・`init` の行、`--dry-run`・`--assume`・`--format`・`--state`・`--force` の行を持たない):

```text
Usage:
  conductor run [-v] [--config <path>] [--run-id <id>] [--remove-on-complete] [--dry-run [--assume <job-id>=<code>]] <job-id>
  conductor pipeline [-v] [--config <path>] [--run-id <id>] [--remove-on-complete] [--dry-run [--assume <job-id>=<code>]] <pipeline-name>
  conductor watch [--config <path>] [--run-id <id>]
  conductor reset [--config <path>] [--run-id <id> | --all]
  conductor validate [-v] [--config <path>] [<pipeline-name>]
  conductor graph [-v] [--config <path>] [--format <text|mermaid>] [--state [--run-id <id>]] [<pipeline-name>]
  conductor init [--config <path>] [--force] [<pipeline-name>]
  conductor convert <file> --to <yaml|toml>
  conductor --help | --version

Options:
  -v, --verbose       print job results, skips and transitions to standard error (run, pipeline; validate prints one line);
                      graph prints the command, retry and skip details under each job
  --config <path>     definition file (default: conductor.toml in the current directory; init: - writes the template to standard output)
  --run-id <id>       run ID that separates the state file (run, pipeline, watch, reset, graph --state; default: the environment variable
                      named by run_id_env in the definition, or CONDUCTOR_RUN_ID; an empty value means no run ID)
  --remove-on-complete
                      remove the state file when the run completes (run, pipeline; default: keep it, or the
                      remove_on_complete setting of the definition)
  --all               remove every state file of the definition, with or without a run ID (reset only)
  --format <text|mermaid>
                      output format of graph (default: text)
  --state             overlay the state of the jobs on the graph (graph only; reads the state file, never writes it)
  --dry-run           print the execution plan without running jobs, writing the state or setting up signals (run, pipeline)
  --assume <job-id>=<code>
                      assume the exit code (0-255) of a job in the plan; repeatable (with --dry-run only; default: 0)
  --force             overwrite the file that already exists (init only; default: leave it and exit with 69)
  --to <yaml|toml>    target format of convert
  -h, --help          print this help
  -V, --version       print the version
```

### 8.2 日本語

```text
使い方:
  conductor run [-v] [--config <パス>] [--run-id <ID>] [--remove-on-complete] [--dry-run [--assume <ジョブID>=<終了コード>]] <ジョブID>
  conductor pipeline [-v] [--config <パス>] [--run-id <ID>] [--remove-on-complete] [--dry-run [--assume <ジョブID>=<終了コード>]] <パイプライン名>
  conductor watch [--config <パス>] [--run-id <ID>]
  conductor reset [--config <パス>] [--run-id <ID> | --all]
  conductor validate [-v] [--config <パス>] [<パイプライン名>]
  conductor graph [-v] [--config <パス>] [--format <text|mermaid>] [--state [--run-id <ID>]] [<パイプライン名>]
  conductor init [--config <パス>] [--force] [<パイプライン名>]
  conductor convert <ファイル> --to <yaml|toml>
  conductor --help | --version

オプション:
  -v, --verbose       ジョブの結果、スキップ、遷移を標準エラーに表示する(run、pipeline。validate は 1 行)。
                      graph は、ジョブごとに、コマンド、リトライ、飛び越しの詳細を表示する
  --config <パス>     定義ファイル(省略時: 実行ディレクトリの conductor.toml。init: - なら、ひな形を標準出力に出す)
  --run-id <ID>       ステートファイルを分ける run ID(run、pipeline、watch、reset、graph --state。省略時: 定義の run_id_env の環境変数、
                      なければ CONDUCTOR_RUN_ID。空の値は run ID なし)
  --remove-on-complete
                      完了のときにステートファイルを消す(run、pipeline。省略時: 残す。定義の remove_on_complete に従う)
  --all               定義のステートファイルを、run ID の有無によらず、すべて消す(reset のみ)
  --format <text|mermaid>
                      graph の出力の形式(省略時: text)
  --state             ジョブの状態を図に重ねる(graph のみ。ステートファイルを読むだけで、書かない)
  --dry-run           ジョブを実行せず、ステートに書かず、シグナルも準備せずに、実行計画を表示する(run、pipeline)
  --assume <ジョブID>=<終了コード>
                      計画の中で、ジョブの終了コード(0〜255)を仮定する。複数回指定できる(--dry-run と一緒のときだけ。省略時: 0)
  --force             既にあるファイルを上書きする(init のみ。省略時: 何も変えず、終了コード 69)
  --to <yaml|toml>    convert の変換後の形式
  -h, --help          この使い方を表示する
  -V, --version       版を表示する
```

## 9. 入力に由来する文字列のエスケープ

表示するジョブID・名前・パス・キー名・コマンド名・使い方の誤りの対象は、エスケープを通してから出す(端末の制御列の注入や、表示の偽装を防ぐため)。ジョブ本体の出力は通さない(そのまま)。

規則(文字は Unicode のコードポイント単位で見る):

| 文字 | 出力 |
|------|------|
| LF(U+000A) | `\n`(バックスラッシュと `n`) |
| CR(U+000D) | `\r` |
| TAB(U+0009) | `\t` |
| 制御文字(Unicode の一般カテゴリ Cc: U+0000〜U+001F、U+007F〜U+009F。上の 3 つを除く) | `\u{<小文字の 16 進、先頭の 0 なし>}`(例: ESC は `\u{1b}`、NUL は `\u{0}`、U+0085 は `\u{85}`) |
| 書式・双方向の制御文字: U+200B〜U+200F、U+2028、U+2029、U+202A〜U+202E、U+2060〜U+2064、U+2066〜U+2069、U+FEFF | `\u{...}`(同じ形式。例: `\u{202e}`) |
| それ以外 | そのまま(日本語もそのまま) |

切り詰め:

- 名前用(`escapeName`): 上限 64 文字。数えるのは、エスケープ後の文字数(コードポイント単位)。次の 1 文字(エスケープ列は 1 かたまり)を足すと 64 を超えるなら、そこで打ち切って末尾に `…`(U+2026)を 1 文字付ける。したがって出力は最大 65 文字。エスケープ列(`\u{202e}` など)は途中で切らない。
- パス用(`escapePath`): 上限なし(エスケープだけ)。
- 名前用を使うもの: 使い方の誤りの対象、ジョブID、遷移先、スキップのID、リトライのジョブID、パイプライン名、run ID、コマンド名。パス用を使うもの: パス(定義ファイル、ステートのパス、`reset --all` のファイル名、変換の入力ファイル、原因、キーの位置)。
- 非 UTF-8 などで文字に直せない部分は、置換文字(U+FFFD)に置き換えてから通す(使い方の誤りの対象について確認済み。ほかのパスは、実装に任せてよい)。

期待値の表(入力 → 上限 → 出力):

| 入力 | 上限 | 出力 |
|------|------|------|
| `a<LF>b<ESC>[0m` | 64 | `a\nb\u{1b}[0m` |
| `k<TAB>x` | 64 | `k\tx` |
| `a<CR>b` | 64 | `a\rb` |
| `日本語 abc` | 64 | `日本語 abc` |
| `a<U+200B>b` | 64 | `a\u{200b}b` |
| `a<U+202E>b` | 64 | `a\u{202e}b` |
| `a<U+2066>b` | 64 | `a\u{2066}b` |
| `a<U+FEFF>b` | 64 | `a\u{feff}b` |
| `a<U+2028>b<U+2029>` | 64 | `a\u{2028}b\u{2029}` |
| `<U+0000><U+007F><U+0085>` | 64 | `\u{0}\u{7f}\u{85}` |
| `k` × 64 | 64 | `k` × 64(切らない) |
| `k` × 200 | 64 | `k` × 64 + `…` |
| LF × 100 | 64 | `\n` × 32 + `…`(65 文字) |
| `abcdef` | 3 | `abc…` |
| (空) | 64 | (空) |

定義の検証・ステート・変換の各処理が、誤りの文を内部で作るときにも、同じ規則のエスケープを使う。その場合、定義とパスのファイル名には 256 文字の上限を使う実装がありうるが、`conductor` の表示では、ファイル名をパス用エスケープ(切らない)で出す。したがって、表示されるファイル名は切り詰められない(必須)。

## 10. 必須の挙動と、実装に任せてよい点

必須:

- 4 章・5 章・6 章・8 章の全文、`-v` の行の形式、出力先(標準出力か標準エラーか)と水準、行の改行。
- 言語の決め方(7 章)、エスケープの規則と切り詰め(9 章)、色の条件(6.3)。
- 日本語・英語ともに、固定の語とトークンの区別(2.1、7 章)。
- (v0.9.0 以降)`.env` の誤りと警告の文言(4.12)、`init` のメッセージ(4.13)、`graph` の図・`--dry-run` の計画・`init` のひな形の文字列(11、12 章)。1 文字単位で同じにする。

実装に任せてよい点:

- 内部で「表示する内容」をどう表現するか、文言の表をどう保持するか。
- 言語・色の判定の遅延評価(観測できる差がない範囲で)。
- 色の端末判定の方法(端末かどうかの判定が UNIX 系で正しく働くこと)。

未確認:

- 定義の誤りの `<原因>` の全文(03 章に属する)。
- `=` を含む短縮形の連結の細部(04 章 4.1)。

## 11. `graph` の図と `--dry-run` の計画の出力(v0.9.0 以降)

図(`graph`)と計画(`--dry-run`)は、組み立て済みの文字列を、標準出力に 1 回で書く。各行の末尾の空白は取り除く。ジョブID、コマンドの語、仮定のジョブIDは、制御文字をパス用エスケープ(切らない)でエスケープする。環境変数の値は、どこにも出さない。以降の例は、テスト(11 の 4.18、4.19 節の AC)の期待値のとおり。配置と整形の規則は、05 の 15、16 章。

### 11.1 `graph` の text(レーンの図)

例 G1(`a`、`b`、`c`。飛び越しなし):

```text
* a
* b
* c
o @end
```

例 G2(`a` の成功が `d` へ飛ぶ。5 ジョブ `a`〜`e`):

```text
*   a
|\
* | b
* | c
|/
* d
* e
o @end
```

例 G3(`a` の成功が `@end` へ。3 ジョブ):

```text
*   a
|\
* | b
* | c
|/
o @end
```

例 G4(入れ子の 2 本。`a` → `f`、`b` → `e`。6 ジョブ):

```text
*   a
|\
* \   b
|\ \
* | | c
* | | d
|/ /
* / e
|/
* f
o @end
```

例 G6(続く 2 本。`a` → `c`、`d` → `f`。7 ジョブ):

```text
*   a
|\
* | b
|/
* c
*   d
|\
* | e
|/
* f
* g
o @end
```

### 11.2 `graph --state`(G2 に、`a` 成功、`b` エラー、そのほか未実行、再開位置 `b`)

日本語:

```text
*   [OK] a
|\
* | [NG] b  <- 再開位置
* | [  ] c
|/
* [  ] d
* [  ] e
o @end
```

英語は、`b` の行が `* | [NG] b  <- resume here`。完了したステート(`a` 成功、`b`・`c` スキップ、`d`・`e` 成功)は、再開位置を示さず、次のようになる。

```text
*   [OK] a
|\
* | [--] b
* | [--] c
|/
* [OK] d
* [OK] e
o @end
```

### 11.3 `graph -v`(G2。日本語)

```text
*   a
|\
| | コマンド: echo a
| | 成功 → d(b、c を飛び越す)、警告 → b
* | b
| | コマンド: echo b
* | c
| | コマンド: echo c
|/
* d
| コマンド: echo d
* e
| コマンド: echo e
o @end
```

G4 の `-v`(日本語。分岐の行のあとに詳細を置き、列の数だけ `|` を重ねる):

```text
*   a
|\
| | コマンド: echo a
| | 成功 → f(b、c、d、e を飛び越す)、警告 → b
* \   b
|\ \
| | | コマンド: echo b
| | | 成功 → e(c、d を飛び越す)、警告 → c
* | | c
| | | コマンド: echo c
* | | d
| | | コマンド: echo d
|/ /
* / e
| | コマンド: echo e
|/
* f
| コマンド: echo f
o @end
```

リトライと配列の `command` の `-v`(英語。`command = ["echo", "hello world", "", "x\ty"]`、`exit_codes = [75, 76]`、`max = 3`、`interval = "5s"`、`multiplier = 2`、`max_interval = "60s"`。`\t` は、バックスラッシュと `t` の 2 文字):

```text
* a
| command: echo "hello world" "" "x\ty"
| retry: exit codes 75, 76, up to 3 times, every 5 s (x2, max 60 s)
* b
| command: echo b
o @end
```

日本語のリトライの行: `| リトライ: 終了コード 75、76 を最大 3 回、間隔 5 秒(倍率 2、上限 60 秒)`。

### 11.3a `graph` の同じ行き先と、両方の飛び越し

例 S1(`a` → `d`、`b` → `d`。5 ジョブ `a`〜`e`)。同じ行き先への飛び越しは、既存の線に合流する(`|\|` の行):

```text
*   a
|\
* | b
|\|
* | c
|/
* d
* e
o @end
```

例 D1(`a` の成功と警告が、どちらも `c` へ。4 ジョブ `a`〜`d`)。成功と警告の両方が飛び越すジョブは、通常の飛び越しと同じ線を引くが、飛び越されるジョブ(`b`)の上下では、主線の `|` を描かず、`\` と `/` だけにする(`a` から `b`、`b` から `c` へ進む辺は、実際にはない):

```text
*   a
 \
* | b
 /
* c
* d
o @end
```

規則: 「両方の飛び越し」のジョブが作る分岐の行は列 0 を空にする(` \`)。同じ行き先へ合流する行は ` \|`。行き先の合流の行は、その線に合流した辺がすべて「両方の飛び越し」なら ` /`、通常の飛び越しが 1 つでも含まれるなら `|/`。通常の飛び越しの図は変わらない。

`-v` の詳細は、S1 の合流の行(`|\|`)のあとに、列の数だけ `| ` を重ねて置く(英語。`a` の分岐の行 `|\` のあとと同じ扱い):

```text
*   a
|\
| | command: echo a
| | success -> d (skips b, c), warning -> b
* | b
|\|
| | command: echo b
| | success -> d (skips c), warning -> c
* | c
...
```

### 11.3b `graph` の分岐

成功と警告が、異なる、どちらも次でない行き先へ進むジョブ(分岐)は、2 本の線を同時に開く(外側が遠い行き先 `t2`、内側が近い行き先 `t1`。成功と警告のどちらが近いかは問わない)。このジョブは次のジョブへ進まないので、ノードの行の幅は `2 × (1 + 既存の線の数 + 2)`、分岐の行は列 0 を空にして `\` を `既存の線の数 + 2` 個、空白 1 つおきに並べる。行き先の合流の行は、その線を開いたジョブがすべて主線につながらない(分岐、「両方の飛び越し」)ときだけ列 0 を空にする。これらの図は、`git log --graph` の出力ではなく、規則から導いた図(参照実装の正解データ。871 例)である。

例 B1(`a` の成功が `c`、警告が `d`。5 ジョブ `a`〜`e`):

```text
*     a
 \ \
* | | b
 / /
* / c
 /
* d
* e
o @end
```

例 B2(`a` の成功が `c`、警告が `@end`。4 ジョブ `a`〜`d`):

```text
*     a
 \ \
* | | b
 / /
* | c
* | d
 /
o @end
```

例 B3(`a` の成功が `g`、`b` の成功が `d`、警告が `e`。外側に通常の飛び越しがある。7 ジョブ `a`〜`g`):

```text
*   a
|\
* \     b
 \ \ \
* | | | c
 / / /
* / / d
 / /
* | e
* | f
|/
* g
o @end
```

例 B4(`a` の成功が `d`、警告が `f`。線が離れる。6 ジョブ `a`〜`f`):

```text
*     a
 \ \
* | | b
* | | c
 / /
* | d
* | e
 /
* f
o @end
```

`-v` の詳細は、分岐の行(` \ \`)のあとに、列の数(ノードの行の幅 ÷ 2。B1 の `a` は 3)だけ `| ` を重ねて置く(英語。B1):

```text
*     a
 \ \
| | | command: echo a
| | | success -> c (skips b), warning -> d (skips b, c)
* | | b
...
```

`--state` の重ね合わせも、行の形を変えない(状態の記号は、ノードの行のラベルに付く)。

辺の一覧に残る形: 分岐の遠い行き先が、すでに開いている最も内側の線の行き先以上(`b` の成功が `d` へ飛ぶ間に、`c` が `e` と `f` へ分岐する)と、後のジョブが分岐の線へ同じ行き先で合流する形(`a` が `c` と `f` へ分岐し、`d` が `f` へ飛ぶ)。

### 11.3c `graph` の交差する飛び越しのレーンの図

飛び越しが交差しても、交差を許す範囲(05 の 15.4。すべてのジョブが次のジョブへの辺を持ち、各ジョブの飛び越しが 1 本以下で、「両方の飛び越し」と分岐がなく、行き先が終点も含めてすべて違う)なら、レーンの図で描く。到着する線が最も内側でないとき、ノードの行の前に、交差の行を 2 行置く。例 C1(`a` → `d`、`b` → `e`。5 ジョブ `a`〜`e`):

```text
*   a
|\
* \   b
|\ \
* | | c
| |/
|/|
* | d
|/
* e
o @end
```

例 C2(`a` → `d`、`b` → `e`、`c` → `f`。7 ジョブ `a`〜`g`)。3 本の線が続けて交差すると、交差の行の行 A に、横線 `_` が入る。

```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
| |_|/
|/| |
* | | d
| |/
|/|
* | e
|/
* f
* g
o @end
```

例 C3(`a` → `f`、`b` → `@end`、`c` → `e`。6 ジョブ `a`〜`f`)。合流の行(`|/ / /`)のあとの `e` の行で、`/` になるのは、次のノード `f` が到着する線の位置以降の線。

```text
*   a
|\
* \   b
|\ \
* \ \   c
|\ \ \
* | | | d
|/ / /
* | / e
| |/
|/|
* | f
|/
o @end
```

C1〜C3 は、`git log --graph` の実際の出力と一致する(正解データ `graph_layout_oracle_cross.md`、940 例)。`-v` の詳細の行は、交差の行の置き場所を変えず、ノードの行(と続く分岐の行)のあとに置く。`--state` の重ね合わせも、行の形を変えない。

### 11.4 `graph` の辺の一覧

例 E1(`a` → `e`、`b` → `f`、`c` → `e`。7 ジョブ `a`〜`g`。飛び越しが交差し、かつ、同じ行き先(`a` と `c` が `e`)を含むので、交差を許す範囲の外で、レーンの図でなく辺の一覧。日本語):

```text
注記: 飛び越しが交差するなどの形のため、レーンの図の代わりに、辺の一覧を出します。
ジョブ:
  a
  b
  c
  d
  e
  f
  g
辺:
  a -> b (警告)
  a -> e (成功)
  b -> c (警告)
  b -> f (成功)
  c -> d (警告)
  c -> e (成功)
  d -> e
  e -> f
  f -> g
  g -> @end
```

例 E2(E1 と同じ定義に、`--state` で、すべて未実行。状態の記号が付く):

```text
注記: 飛び越しが交差するなどの形のため、レーンの図の代わりに、辺の一覧を出します。
ジョブ:
  [  ] a
  [  ] b
  [  ] c
  [  ] d
  [  ] e
  [  ] f
  [  ] g
辺:
  a -> b (警告)
  a -> e (成功)
  b -> c (警告)
  b -> f (成功)
  c -> d (警告)
  c -> e (成功)
  d -> e
  e -> f
  f -> g
  g -> @end
```

辺の一覧に切り替わる形は、飛び越しが交差し、かつ、グラフのどこかに、同じ行き先へ飛ぶ辺、「両方の飛び越し」、分岐を含む形(離れたジョブにあっても同じ。交差を許す範囲の外。11.3c)と、分岐の線に関わる、描けない形(分岐の遠い行き先が最も内側の線の行き先以上の形、後のジョブが分岐の線へ同じ行き先で合流する形。11.3b)と、レーンで描けない形(後ろ向きの辺など)だけ。交差する通常の飛び越し(11.3c)や、成功が `c`、警告が `e` のような分岐そのものは、レーンの図で描く。英語の注記は `note: the skip edges cross or form a shape the lane diagram cannot draw, so it is replaced by a list of edges.`。

英語の見出しは `Jobs:`、`Edges:`、種類は `(success)`、`(warning)`、`(success, warning)`。`-v` の詳細は、ジョブの行の下に、4 つの空白で字下げして足す(英語。E1 と同じ定義):

```text
Jobs:
  a
    command: echo a
    success -> e (skips b, c, d), warning -> b
  b
  ...
Edges:
  a -> b (warning)
  ...
```

### 11.5 `graph --format mermaid`(G2)

```text
flowchart TD
  n0["a"]
  n1["b"]
  n2["c"]
  n3["d"]
  n4["e"]
  nend(["@end"])
  n0 -->|"警告"| n1
  n0 -->|"成功"| n3
  n1 --> n2
  n2 --> n3
  n3 --> n4
  n4 --> nend
```

(ラベルの語は、英語では `"warning"`、`"success"`。)`--state` で `a` 成功、`b` エラー、そのほか未実行、再開位置 `b` のときは、節の定義が `n0["[OK] a"]`、`n1["[NG] b"]`、`n2["[  ] c"]`、…になり、末尾に次の 2 行が付く。

```text
  classDef resume stroke-width:3px
  class n1 resume
```

エスケープ: `a<b>&c#d"e`f<SOH>` → `a#lt;b#gt;#amp;c#35;d#quot;e#96;f#1;`。日本語のジョブID(`取得`)と予約語(`end`)は、そのまま、引用符の中に書く(`n0["end"]`、`n1["取得"]`)。

### 11.6 `--dry-run` の計画(`pipeline`)

形(1 行目から順に): ヘッダー、再開位置(または `run` の対象のジョブ)、仮定(あれば)、使われなかった仮定(あれば)、ジョブごとの項目、模擬の結果(`pipeline`)または次の動き(`run`)。

| 行 | 日本語 | 英語 |
|----|--------|------|
| ヘッダー | `計画(--dry-run): パイプライン <名前>`(run ID があれば、続けて `(run ID: <ID>)`。空白なし) | `plan (--dry-run): pipeline <名前>`(あれば ` (run ID: <ID>)`。空白あり) |
| 再開位置 | `再開位置: 最初から` / `再開位置: <ID>(前回の停止)` | `resume: from the start` / `resume: <ID> (previous stop)` |
| `run` の対象 | `対象のジョブ: <ID>` | `target job: <ID>` |
| 仮定 | `仮定: a=3, b=1`(指定順。`, `) | `assume: a=3, b=1` |
| 使われなかった仮定 | `使われなかった仮定: b` | `unused assumptions: b` |
| ジョブの項目 | `<番号>. [<扱い>] <ID>`(再開位置のジョブは行末に `  <- 再開位置`) | `<番号>. [<扱い>] <ID>`(`  <- resume here`) |
| コマンド(`WillRun` のみ) | 5 つの空白 + `コマンド: ["echo", "a"]` | 5 つの空白 + `command: ["echo", "a"]` |
| 環境変数(同) | 5 つの空白 + `環境変数: A, B, ...`(名前だけ。層の順) | 5 つの空白 + `env: A, B, ...` |
| リトライ(同。あれば) | 5 つの空白 + `リトライ: 終了コード 75、76 を最大 3 回、間隔 5 秒` | 5 つの空白 + `retry: exit codes 75, 76, up to 3 times, every 5 s` |
| 模擬の結果(`pipeline`) | `結果(模擬): <終わり>` | `result (simulated): <終わり>` |
| 次の動き(`run`) | `次の動き: <動き>` | `next step: <動き>` |

- 扱い(`[ ]` の中): `実行する`/`run`、`実行済み`/`already executed`、`スキップ(遷移)`/`skip (transition)`、`スキップ(@end のあと)`/`skip (after @end)`、`実行されない(停止のあと)`/`not run (after stop)`。
- 仮定の補足: `WillRun` のジョブで、`--assume` で指定したもの、または既定の結果が成功でないものに、扱いの語のあとに括弧で付ける(日本語は語のすぐあと `実行する(仮定: 終了コード 3 → 警告)`、英語は空白を置いて `run (assumed: exit code 3 -> warning)`)。指定のない既定の成功は、補足なし。補足の語: `仮定`/`assumed`(指定)、`既定`/`default`(既定の 0)、`終了コード <C> → <分類>`/`exit code <C> -> <class>`、分類の語 `成功`/`警告`/`エラー`、`success`/`warning`/`error`。リトライの対象の終了コードなら、続けて `、リトライの対象`/`, retry target`。
- 模擬の終わり: `完了`/`completed`、`@end に到達(完了)`/`reached @end (completed)`、`停止(エラー: <ID>)`/`stopped (error: <ID>)`。
- 次の動き(`run`): 先頭に、結果が警告かエラーのとき `(<仮定の補足>) `(リトライの注記なし)を添える。そのあと: 停止なら `停止`/`stop`、`@end` に到達して未完了なら `@end に到達(残りのジョブは、呼ばれたときにスキップ)`/`reached @end (the rest are skipped when called)`、完了なら `完了`/`done`、そうでなければ `次のジョブ <ID>`(あれば `(スキップ: <ID>、<ID>)`)/`next job <ID>`(`(skipped: <ID>, <ID>)`)。
- 実行済みなどで `WillRun` でない `run` の対象は、次の動きの行を出さない。

例 P1(日本語。ステートなし、すべて成功):

```text
計画(--dry-run): パイプライン demo
再開位置: 最初から
1. [実行する] a
     コマンド: ["echo", "a"]
     環境変数: CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_ATTEMPT
2. [実行する] b
     コマンド: ["echo", "b"]
     環境変数: CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_ATTEMPT
3. [実行する] c
     コマンド: ["echo", "c"]
     環境変数: CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_ATTEMPT
結果(模擬): 完了
```

例 P2(日本語。`[pipeline.env]` に `API_URL`、run ID `r1`、`a` 成功済みで `b` がエラーで停止、`c` にリトライ、`--assume c=75`):

```text
計画(--dry-run): パイプライン demo(run ID: r1)
再開位置: b(前回の停止)
仮定: c=75
1. [実行済み] a
2. [実行する] b  <- 再開位置
     コマンド: ["echo", "b"]
     環境変数: API_URL, CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_RUN_ID, CONDUCTOR_ATTEMPT
3. [実行する(仮定: 終了コード 75 → エラー、リトライの対象)] c
     コマンド: ["fetch", "--retry"]
     環境変数: API_URL, CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_RUN_ID, CONDUCTOR_ATTEMPT
     リトライ: 終了コード 75、76 を最大 3 回、間隔 5 秒
4. [実行されない(停止のあと)] d
結果(模擬): 停止(エラー: c)
```

例 P2e(P2 の英語):

```text
plan (--dry-run): pipeline demo (run ID: r1)
resume: b (previous stop)
assume: c=75
1. [already executed] a
2. [run] b  <- resume here
     command: ["echo", "b"]
     env: API_URL, CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_RUN_ID, CONDUCTOR_ATTEMPT
3. [run (assumed: exit code 75 -> error, retry target)] c
     command: ["fetch", "--retry"]
     env: API_URL, CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_RUN_ID, CONDUCTOR_ATTEMPT
     retry: exit codes 75, 76, up to 3 times, every 5 s
4. [not run (after stop)] d
result (simulated): stopped (error: c)
```

例 P3(日本語。`a` の成功が `c` へ飛ぶ。`--assume b=1` は使われない):

```text
計画(--dry-run): パイプライン demo
再開位置: 最初から
仮定: b=1
使われなかった仮定: b
1. [実行する] a
     コマンド: ["echo", "a"]
     環境変数: CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_ATTEMPT
2. [スキップ(遷移)] b
3. [実行する] c
     コマンド: ["echo", "c"]
     環境変数: CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_ATTEMPT
結果(模擬): 完了
```

例 P4(日本語。`a` が警告 `[3]` で `@end` へ。`--assume a=3`)と P4e(英語):

```text
計画(--dry-run): パイプライン demo
再開位置: 最初から
仮定: a=3
1. [実行する(仮定: 終了コード 3 → 警告)] a
     コマンド: ["echo", "a"]
     環境変数: CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_ATTEMPT
2. [スキップ(@end のあと)] b
3. [スキップ(@end のあと)] c
結果(模擬): @end に到達(完了)
```

```text
plan (--dry-run): pipeline demo
resume: from the start
assume: a=3
1. [run (assumed: exit code 3 -> warning)] a
     command: ["echo", "a"]
     env: CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_ATTEMPT
2. [skip (after @end)] b
3. [skip (after @end)] c
result (simulated): reached @end (completed)
```

例 P6(日本語。`a` が `success_exit_codes = [10]` で、既定の 0 がエラーになる):

```text
計画(--dry-run): パイプライン demo
再開位置: 最初から
1. [実行する(既定: 終了コード 0 → エラー)] a
     コマンド: ["make"]
     環境変数: CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_ATTEMPT
2. [実行されない(停止のあと)] b
結果(模擬): 停止(エラー: a)
```

環境変数の名前は、層の順(`[pipeline].env_file`、`[pipeline.env]`、`[job].env_file`、`[job.env]`、自動の変数)に並ぶ。例: `env: P_FILE, P_ENV, A_FILE, J_ENV, CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_RUN_ID, CONDUCTOR_ATTEMPT`。

### 11.7 `--dry-run` の計画(`run`)

例 P5(日本語。`a` 成功済み、`b` と `c` が未実行。`run b --dry-run`):

```text
計画(--dry-run): パイプライン demo
対象のジョブ: b
1. [実行する] b
     コマンド: ["echo", "b"]
     環境変数: CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_ATTEMPT
次の動き: 次のジョブ c
```

例 P7(`a` の成功が `c` へ飛ぶ。`run a --dry-run`)の最後の行: `次の動き: 次のジョブ c(スキップ: b)`。例 P8(`a` の成功が `@end` へ)の最後の行: `次の動き: @end に到達(残りのジョブは、呼ばれたときにスキップ)`。`--assume a=1`(エラー)なら、最後の行は、日本語 `次の動き: (仮定: 終了コード 1 → エラー) 停止`、英語 `next step: (assumed: exit code 1 -> error) stop`(`target job: a` の行から始まる)。

## 12. `init` のひな形(v0.9.0 以降)

ひな形は、(名前の前まで)+ エスケープした名前 + (名前のあと)の文字列である。UTF-8、LF、末尾に改行 1 つ。言語は、ロケールの判定結果(日本語か英語。コメントだけが違う)。`name` は、既定 `my-pipeline`。

日本語(名前が既定のとき):

```toml
# conductor の定義ファイルのひな形(conductor init が作った)。
# 検査: conductor validate / 実行計画: conductor pipeline <名前> --dry-run / 図: conductor graph
format_version = 1

[pipeline]
name = "my-pipeline"
description = "ひな形のパイプライン"

# 全ジョブに足す環境変数。値は、書いたとおりに渡す(展開しない)。
# .env ファイルから読むときは、[pipeline] に env_file = ".env" を足す。
[pipeline.env]
GREETING = "hello"

# hello: 最初のジョブ。
# Windows では、echo は実行ファイルではないので、command = ["cmd", "/c", "echo", "hello"] のように書く。
[[job]]
id = "hello"
command = ["echo", "hello"]

# flaky: 終了コード 75 は「一時的な失敗」。5 秒待って、最大 3 回、実行し直す。
# 待ち時間を伸ばすには、multiplier と max_interval を、一緒に書く(例: multiplier = 2、max_interval = "1m")。
[[job]]
id = "flaky"
command = ["echo", "flaky"]

[job.retry]
exit_codes = [75]
max = 3
interval = "5s"
```

英語(名前が既定のとき):

```toml
# A starter conductor definition (created by conductor init).
# Check: conductor validate / plan: conductor pipeline <name> --dry-run / diagram: conductor graph
format_version = 1

[pipeline]
name = "my-pipeline"
description = "Starter pipeline"

# Environment variables added to every job. Values are passed as written (no expansion).
# To read a .env file, add env_file = ".env" to [pipeline].
[pipeline.env]
GREETING = "hello"

# hello: the first job.
# On Windows echo is not an executable; write command = ["cmd", "/c", "echo", "hello"] instead.
[[job]]
id = "hello"
command = ["echo", "hello"]

# flaky: exit code 75 means "a temporary failure". Wait 5 seconds and run again, up to 3 times.
# To lengthen the wait, write multiplier and max_interval together (for example multiplier = 2, max_interval = "1m").
[[job]]
id = "flaky"
command = ["echo", "flaky"]

[job.retry]
exit_codes = [75]
max = 3
interval = "5s"
```

- 名前のエスケープ: TOML の基本文字列として、`"` → `\"`、`\` → `\\`、制御文字(U+0000〜U+001F と U+007F)→ `\uXXXX`(大文字 16 進 4 桁)。それ以外はそのまま(日本語の名前もそのまま)。名前は、コメントには書かない。
- ひな形は、`validate` に必ず通る(名前の空白だけは 68 で先に拒否される)。`[pipeline.env]` と `[job.retry]` の例を、有効な行として含む。
- 入力は言語と名前だけで、利用者の環境変数や、`.env`、既存のファイルの内容は入らない。
