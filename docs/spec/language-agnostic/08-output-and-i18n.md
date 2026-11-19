# 08. 出力と多言語化

この章は、`conductor` が出す文字列の全文、出力先、言語の決め方、`watch` の表、エスケープの規則を規定する。文言は、この章の表のとおりに(文字単位で)出力する。コマンドラインは [04-cli.md](04-cli.md)、終了コードは [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md) を参照する。

## 1. 出力先の使い分け

表示は、処理の流れが作る「表示する内容」を、文言に直して、水準に従って出す。水準は 4 つある。

| 水準 | 出力先 | 出す条件 | 書き込み失敗 |
|------|--------|----------|--------------|
| `Error` | 標準エラー | 常に | 無視(本筋を止めない) |
| `Notice` | 標準エラー | 常に(エラーではない知らせ。リトライの前の知らせだけ) | 無視 |
| `Verbose` | 標準エラー | `-v` のときだけ | 無視 |
| `Output` | 標準出力 | 常に | 呼び出し元に失敗を返す(`watch` は 65、`convert` は 67 で終わる) |

- どの文言も、最後に改行(LF、`\n`)を含む。複数の誤りは 1 行ずつ。
- 書き込みのたびに flush する(行ごとに、すぐに見える)。
- 標準出力に出すのは、`watch` の表(と `no state`)、`--help`、`--version`、`convert` の変換結果だけ。
- ジョブ本体の標準出力・標準エラーは `conductor` が加工せず、そのまま流れる(`conductor` はジョブの標準入出力を継承させる)。`conductor` の表示は標準エラーなので、標準出力にはジョブの出力だけが残る。
- 成功・スキップで `-v` なしのときは、表示が一度もなく、言語も色も判定されない(必要になった最初の時点まで判定を遅らせる)。この遅延は、観測できる差(環境変数の読み取り順)を生まないなら、実装に任せてよい。

### 1.1 水準ごとのイベント

| 水準 | イベント |
|------|----------|
| Error | 定義の誤り、`command` の誤り、使い方の誤り(+使い方全文)、ステートの誤り、定義との食い違い、`reset --all` の警告(`conductor: warning:`)、拒否、起動失敗、成功コードに含まれない 0、変換の誤り、標準出力に書けない、完了を判定できない、シグナルの準備の失敗、内部の誤り |
| Notice | リトライの前の知らせ(`conductor: retry:`) |
| Verbose | ジョブの結果、スキップ、遷移、中断、`validate` が通った |
| Output | `watch` の表、`watch` の空、`--help`、`--version` |

## 2. `-v`(`--verbose`)

- `run`・`pipeline`・`validate` だけが受け付ける。1 回だけ(2 回は 68)。
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
- `<場所>`: 定義の誤りは `<ファイル>` または `<ファイル>:<行>`、`command` の誤りは `<定義ファイル>: job '<ID>'`、ステート・食い違いは `<ステートのパス>`、拒否・起動失敗・成功コードに含まれない 0 は `job '<ID>'`、変換の誤りは `<ファイル>` または `<ファイル>:<行>`。
- 使い方の誤りだけは場所を持たず、`conductor: error: <文>[: '<対象>']` のあとに空行と使い方全文(8 章)を続ける。

## 4. メッセージカタログ

「英語」「日本語」の列は、原文のとおり出す。`{…}`・`<…>` は実行時に入る値(エスケープ後。9 章)。

### 4.1 固定の英語の文(ロケールを判定しない)

| キー | 文 | 終了コード |
|------|----|------------|
| `INTERNAL_ERROR`(想定外の内部失敗) | `conductor: error: internal error` | 70 |
| `SIGNAL_SETUP_FAILED` | `conductor: error: failed to set up signal handling` | 70 |
| `--version` | `conductor {版}`(v0.8.0 では `conductor 0.8.0`) | 0 |

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

```text
Usage:
  conductor run [-v] [--config <path>] [--run-id <id>] [--remove-on-complete] <job-id>
  conductor pipeline [-v] [--config <path>] [--run-id <id>] [--remove-on-complete] <pipeline-name>
  conductor watch [--config <path>] [--run-id <id>]
  conductor reset [--config <path>] [--run-id <id> | --all]
  conductor validate [-v] [--config <path>] [<pipeline-name>]
  conductor convert <file> --to <yaml|toml>
  conductor --help | --version

Options:
  -v, --verbose       print job results, skips and transitions to standard error (run, pipeline; validate prints one line)
  --config <path>     definition file (default: conductor.toml in the current directory)
  --run-id <id>       run ID that separates the state file (run, pipeline, watch, reset; default: the environment variable
                      named by run_id_env in the definition, or CONDUCTOR_RUN_ID; an empty value means no run ID)
  --remove-on-complete
                      remove the state file when the run completes (run, pipeline; default: keep it, or the
                      remove_on_complete setting of the definition)
  --all               remove every state file of the definition, with or without a run ID (reset only)
  --to <yaml|toml>    target format of convert
  -h, --help          print this help
  -V, --version       print the version
```

### 8.2 日本語

```text
使い方:
  conductor run [-v] [--config <パス>] [--run-id <ID>] [--remove-on-complete] <ジョブID>
  conductor pipeline [-v] [--config <パス>] [--run-id <ID>] [--remove-on-complete] <パイプライン名>
  conductor watch [--config <パス>] [--run-id <ID>]
  conductor reset [--config <パス>] [--run-id <ID> | --all]
  conductor validate [-v] [--config <パス>] [<パイプライン名>]
  conductor convert <ファイル> --to <yaml|toml>
  conductor --help | --version

オプション:
  -v, --verbose       ジョブの結果、スキップ、遷移を標準エラーに表示する(run、pipeline。validate は 1 行)
  --config <パス>     定義ファイル(省略時: 実行ディレクトリの conductor.toml)
  --run-id <ID>       ステートファイルを分ける run ID(run、pipeline、watch、reset。省略時: 定義の run_id_env の環境変数、
                      なければ CONDUCTOR_RUN_ID。空の値は run ID なし)
  --remove-on-complete
                      完了のときにステートファイルを消す(run、pipeline。省略時: 残す。定義の remove_on_complete に従う)
  --all               定義のステートファイルを、run ID の有無によらず、すべて消す(reset のみ)
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

実装に任せてよい点:

- 内部で「表示する内容」をどう表現するか、文言の表をどう保持するか。
- 言語・色の判定の遅延評価(観測できる差がない範囲で)。
- 色の端末判定の方法(端末かどうかの判定が UNIX 系で正しく働くこと)。

未確認:

- 定義の誤りの `<原因>` の全文(03 章に属する)。
- `=` を含む短縮形の連結の細部(04 章 4.1)。
