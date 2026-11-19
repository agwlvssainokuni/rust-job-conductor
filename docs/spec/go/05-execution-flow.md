# 05. 実行の意味論(終了コード・分岐・再開・run ID・リトライ・パイプライン)

対象: Go で実装する `conductor` v0.8.0 と同じ挙動の実行ツール。この文書は、`run`(ジョブを 1 つ実行)と `pipeline`(定義の全ジョブを順に実行)の実行の流れを、判定表・状態遷移・擬似コードで定める。数値の計算(待ち時間の倍率・上限・飽和・変換)は、Go の型に即した式と境界の例で定める。この文書だけで、Go でゼロから同じ挙動を実装できることを目標にする。

ステートファイルの論理項目・保存の性質・読み込みの検査は [06-state-file.md](06-state-file.md) にある。この文書の「ステートへの記録」は、06 の形式のファイルを書く前の、メモリ上のステート(以下「ステート値」)の更新を指す。定義ファイルの書式は [03-definition-format.md](03-definition-format.md)、コマンドラインは [04-cli.md](04-cli.md)、プロセス起動とシグナルは [07-process-and-signals.md](07-process-and-signals.md)、メッセージの文言は [08-output-and-i18n.md](08-output-and-i18n.md)、終了コードの総覧は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md) を参照。

## 0. 構成の決定(Go 版)

「判断」(分類・次の動き・起動前の判断・リトライの判断・再開位置)は、ファイルにもプロセスにも触れない**純粋な関数**にし、失敗させず(`error` を返さず)必ず結果を返す。置き場所は次のとおり固定する。

| パッケージ | 置くもの |
|---|---|
| `internal/flow` | 共有の値の型(`JobStatus`、`SkipReason`、`JobState`、`PipelineState`、`JobExecution` ほか。`JobExecution` が 1 回の実行結果の型)、`Classify`、`ClassifyAndDecide`、`NextStep`、`DecideBeforeRun`、`RetryDecision`、`NextWaitSecs`、`ResumePointOnInterrupt`、`ExitCodeFor`、`SignalExitCode`。標準ライブラリのうち `math`・`math/bits` だけを使う。ファイル・時計・環境変数・シグナルには触れない |
| `internal/state` | 記録の関数(`ApplyOutcome`、`ApplySkip`、`ApplyInterrupt`、`InitialState`。純粋。複製して更新)、読み書き([06](06-state-file.md)) |
| `internal/runid` | run ID の検査と決定(§6) |
| `internal/runner` | `RunJob`(子プロセスの起動と待機)、`WaitFor`(§7.8) |
| `internal/cli` | `prepare`、`run`・`pipeline` の流れ(§5)、リトライの実行ループ(§7.7)。`Deps` を介して時計・環境・ファイル・プロセスに触れる |

依存の向きは `definition` ← `flow` ← (`state`、`runner`) ← `cli`(`flow` は `definition` の型だけを使い、`state` と `runner` は `flow` の共有の型を使う。`state` と `runner` は互いに依存しない。`cli` が最上位で、循環させない。[02-architecture.md](02-architecture.md) 3 章)。

---

## 1. 用語とデータ

### 1.1 ジョブの状態(`JobStatus`)

| 値(ステート上の文字列) | 意味 |
|---|---|
| `pending` | まだ起動していない。記録がないジョブも pending と同じに扱う |
| `success` | 成功(成功コードで終わった) |
| `warning` | 警告(警告コードで終わった) |
| `error` | エラー |
| `skipped` | スキップ。理由 `skip_reason` が必ず付く |

- 「済んだ(settled)」状態 = `success` / `warning` / `skipped`。
- 「結果のある」状態 = `success` / `warning` / `error`(終了コードを持てる)。
- スキップ理由(`SkipReason`): `byTransition`(遷移によるスキップ)、`byEnd`(`@end` によるスキップ)。値が無いことは、Go では空文字列 `""` で表す。

### 1.2 1 回の実行結果(`JobExecution`)

ジョブを 1 回起動して終わるまでの観測結果。

| フィールド | Go の型 | 意味 |
|---|---|---|
| `Started` | `bool` | 起動した(子プロセスの生成に成功した)か。「中断の合図が起動の前に立っていたため起動しなかった」ときだけ `false`。ほかの起動失敗でも `Started` は `true` のまま(`StartFailure` を見る) |
| `ExitCode` | `*int32` | プロセスの終了コード。取れないとき(シグナル終了、起動失敗)は `nil` |
| `TerminatedBySignal` | `*int32` | ジョブがシグナルで終了したときの番号(UNIX 系のみ) |
| `StartFailure` | `bool` | 起動に失敗した。command を引数に分けられない、プロセス生成の失敗、終了状態を取得できない、のいずれか |
| `Interrupted` | `bool` | conductor がシグナルを受け取り、中断の合図が立っている(ジョブの終了を観測した直後に立っていた場合を含む) |
| `SignalNumber` | `*int32` | `Interrupted` のとき、最初に受け取ったシグナルの番号 |

ジョブの起動手順(`RunJob`):

1. command を引数に分ける(規則は [07-process-and-signals.md](07-process-and-signals.md))。失敗なら `StartFailure = true`(`Started = true`)で終わる。
2. 中断の合図が立っていれば、`Started = false, Interrupted = true, SignalNumber = 番号` で終わる(起動しない)。
3. 子プロセスを生成する(`os/exec`)。失敗なら `StartFailure = true` で終わる。
4. 終了まで待つ。待機中に合図が立てば、子プロセスに同じシグナルを**1 回だけ**転送して、終了を待ち続ける。
5. 終了コードとシグナル終了を記録する。終了状態を取得できなければ `StartFailure = true`。
6. 終了を観測した直後に合図が立っていれば `Interrupted = true, SignalNumber = 番号`。

終了コードの型の変換(Go 版の決定): `exec.ExitError` の `ExitCode()` は `int` を返す。`ExitCode` フィールドは `int32` なので、`0 <= code <= math.MaxInt32` ならそのまま、`math.MaxInt32 < code <= math.MaxUint32`(Windows の NTSTATUS 型の値)なら `int32(uint32(code))`(2 の補数の再解釈で負数になる。例: `0xC0000005` → `-1073741819`)、それ以外(UNIX 系でシグナル終了のとき `-1`)は `ExitCode = nil` にして、シグナル終了の判定(07)へ進む。この変換の Windows での実機の値は未確認(§13)。

### 1.3 結果(`JobOutcome`)

| フィールド | 意味 |
|---|---|
| `Class` | `ClassSuccess` / `ClassWarning` / `ClassError`、または `ClassNone`(**分類なし**: 「起動しなかった中断」) |
| `ReportedExitCode` | スケジューラへ返す終了コード(`int32`)。後で 0〜255 に整える(§2.2) |
| `ConductorError` | 「成功コードに含まれない 0」など、conductor 自身が原因を標準エラーに表示するエラーのとき `true` |

### 1.4 実行方式(`RunMode`)

`ModeIndividual`(`conductor run <job-id>`)と `ModePipeline`(`conductor pipeline <name>`)。`@end` の扱いと完了の判定だけが違う(§3、§5)。

---

## 2. 終了コード

### 2.1 conductor が返す終了コード(プロセスの終了コード)

| 値 | 名前 | 意味 |
|---|---|---|
| 0 | OK | 成功、警告、スキップ、正常終了 |
| ジョブの値(1〜255) | - | エラーになったジョブの終了コードをそのまま(範囲外は §2.2) |
| 1 | - | 成功コードに含まれない 0 で終わったジョブ(`ConductorError`)。終了コードを取れなかったエラーも 1 |
| 64 | DEFINITION | 定義の誤り(ファイルがない・読めない・検証の誤り・パイプライン名の不一致・command の誤り) |
| 65 | STATE | ステートの誤り(読めない・壊れている・版の不一致・定義との食い違い・書き込み/初期化/削除の失敗)、完了を判定できない、`watch` の出力失敗 |
| 66 | REJECTED | 拒否(順序外) |
| 67 | CONVERSION | convert の失敗(この文書の対象外) |
| 68 | USAGE | 使い方の誤り(知らないジョブID、run ID の値の誤りを含む) |
| 70 | INTERNAL | 内部の誤り(想定外の異常終了、シグナル処理の準備の失敗) |
| 128 + 番号 | - | ジョブがシグナルで終了した、または conductor が中断された(SIGINT = 130、SIGTERM = 143) |

64〜68・70 は、ジョブ自身の終了コードと重なりうる。区別は標準エラーの `conductor: error: ...` の行で行う。

### 2.2 `ExitCodeFor(reported int32, isError bool) int`: 0〜255 への整形

プロセスの終了コードは 0〜255 に収める(`os.Exit` に渡す値)。

```text
if 0 <= reported <= 255:  return reported
low = reported AND 0xFF            # 負数も含め、2 の補数表現の下位 8 ビット
if low == 0 and isError: return 1  # エラーを成功に見せない
return low
```

Go での書き方の注意: `int32` に対するビット積 `reported & 0xFF` は、Go の言語仕様どおり 2 の補数として働くので、負数でも上の式のまま下位 8 ビットが得られる(例: `int32(-1) & 0xFF == 255`)。`uint32` へ変換してから `& 0xFF` しても同じ結果になる。どちらでもよい。

| reported | isError | 結果 |
|---|---|---|
| 3 | true | 3 |
| 256 | true | 1(下位 8 ビット 0 → 1) |
| 256 | false | 0 |
| 257 | true | 1 |
| 511 | true | 255 |
| -1 | true | 255 |
| -256 | true | 1 |
| 2147483647 | true | 255 |
| -2147483648 | true | 1 |

`isError` は `Class == ClassError`。分類なし(起動しなかった中断)と成功・警告は `false`。

中断の終了コード `SignalExitCode(n int32) int` = `ExitCodeFor(SatAddInt32(128, n), true)`(パイプラインのジョブのあいだの中断で使う)。

`SatAddInt32(a, b int32) int32`(32 ビット符号付き整数の飽和加算): `int64(a)+int64(b)` を計算し、`math.MaxInt32`(2147483647)を超えるなら 2147483647、`math.MinInt32`(-2147483648)を下回るなら -2147483648 にして `int32` へ戻す。64 ビットで計算するので、途中でオーバーフローしない。例: `SatAddInt32(128, 2147483647) = 2147483647`、`SatAddInt32(128, 2) = 130`、`SatAddInt32(128, -2147483648) = -2147483520`。実在のシグナル番号(1〜64 程度)では飽和は起きない。

### 2.3 成功・警告・エラーの分類と返す値(`Classify`)

- **成功コードの集合** = 定義の `success_exit_codes` があればその値だけ(0 を書かなければ 0 でも成功にならない)、無ければ `[0]`。
- **警告コードの集合** = `warning_exit_codes`(0 を含まず、成功コードと重ならないことは定義の検証で保証済み)。

判定は次の順に、最初に当てはまるものを採る(上ほど優先)。入力は 1 回の実行結果(§1.2)とジョブ定義。

| 順 | 条件 | Class | ReportedExitCode | ConductorError |
|---|---|---|---|---|
| 1 | `Started = false` かつ `Interrupted` | ClassNone | `SatAddInt32(128, SignalNumber)`(`SignalNumber` が `nil` なら `SatAddInt32(128, 0)` = 128) | false |
| 1' | `Started = false` かつ `Interrupted` でない(あり得ない組み合わせ) | ClassError | 127 | false |
| 2 | `Interrupted`(起動済み) | ClassError | `SatAddInt32(128, SignalNumber)`(`nil` なら 128)。ジョブ自身の終了コードは見ない | false |
| 3 | `StartFailure` | ClassError | 127 | false |
| 4 | `TerminatedBySignal` が `nil` でない(値 n) | ClassError | `SatAddInt32(128, n)` | false |
| 5 | `ExitCode` が `nil`(起動して終わったのに終了コードがない) | ClassError | 1 | **true** |
| 6 | `ExitCode` ∈ 成功コード | ClassSuccess | 0 | false |
| 7 | `ExitCode` ∈ 警告コード | ClassWarning | 0 | false |
| 8 | `ExitCode = 0`(成功コードに無い) | ClassError | 1 | **true** |
| 9 | それ以外 | ClassError | `ExitCode` そのまま(負数も、255 超もそのまま保持。整形は §2.2) | false |

全組み合わせの例(定義 `warning_exit_codes = [3, 4]`、`success_exit_codes` 省略):

| 実行結果 | 分類 | 返す値 | プロセス終了コード |
|---|---|---|---|
| exit 0 | success | 0 | 0 |
| exit 3 / 4 | warning | 0 | 0 |
| exit 1 / 2 / 255 | error | 1 / 2 / 255 | 同じ |
| exit -1(Windows 等) | error | -1 | 255 |
| 起動失敗 / command 不正 | error | 127 | 127 |
| SIGKILL(9)で終了 | error | 137 | 137 |
| 実行中に SIGTERM(15)を受信(ジョブが exit 0 でも) | error | 143 | 143 |
| 起動前に SIGINT(2)が立っていた | 分類なし | 130 | 130 |

`success_exit_codes = [2]` のとき: exit 2 → success(0)、exit 0 → error(1, `ConductorError`)。`success_exit_codes = [0, 2]` のとき exit 0 も success。

標準エラーへの常時の表示(`-v` なしでも):

- `ConductorError = true`: `conductor: error: job '<id>': exit code 0 is not listed in success_exit_codes`(日本語ロケールでは日本語。文言は [08-output-and-i18n.md](08-output-and-i18n.md))。`ConductorError = true` なら、順 5(終了コードが無い)の場合も同じ文言を出す(`ConductorError` が真なら、理由を区別せず同じ行を出す)。
- 起動失敗: `conductor: error: job '<id>': cannot start command '<program>'`。

---

## 3. 遷移と次の動き(`ClassifyAndDecide` / `NextStep`)

ジョブ実行後に、分類と次の動き(`NextStep`)を決める。

### 3.1 `NextStep` のフィールド

| フィールド | Go の型 | 意味 |
|---|---|---|
| `NextJobID` | `string`(`""` = 無し) | 次に実行するジョブ。完了・停止のとき無し(`pipeline` コマンドは、この値を使わない。§5.3)。ジョブIDは定義の検証で空でないことが保証される(`@` で始まらず、前後の空白を除いて空でない)ので、`""` を「無し」に使える |
| `SkippedJobIDs` | `[]string` | この時点でスキップとして記録するジョブ |
| `SkipReasonKind` | `SkipReason`(`""` = なし) | その理由(`byTransition` / `byEnd` / なし) |
| `EndReached` | `bool` | `@end` に到達した |
| `Completed` | `bool` | 完了した |
| `ResumeJobID` | `string`(`""` = 無し) | エラー等で停止したときの再開位置(そのジョブ)。無しもある |

### 3.2 判定表

入力: 分類、そのジョブの該当する遷移(`on = success` は成功のとき、`on = warning` は警告のときに参照する。`error` の遷移は書けない)、`RunMode`、そのジョブの位置(index。定義の記述順、0 始まり)、各ジョブの現在の状態。

「未完了のジョブ」= 指定した区間のジョブのうち、状態が success / warning / skipped **でない**もの(pending・error・記録なし)。区間は、`@end` なら「index より後ろ全部」、遷移先 `T` なら「index と T のあいだ(両端を含まない)」。

| 分類 | 遷移 | RunMode | NextJobID | SkippedJobIDs / kind | EndReached | Completed | ResumeJobID |
|---|---|---|---|---|---|---|---|
| error | (見ない) | 両方 | なし | 空 / なし | false | false | **このジョブ** |
| 分類なし(起動しなかった中断) | (見ない) | 両方 | なし | 空 / なし | false | false | **このジョブ** |
| success / warning | `@end` | Pipeline | なし | 後ろの未完了ジョブ全部 / `byEnd`(空なら kind なし) | true | **true** | なし |
| success / warning | `@end` | Individual | なし | **空** / kind = `byEnd`(後ろに未完了が 1 つでもあるとき。無ければ kind なし) | true | 後ろに未完了が無ければ true、あれば false | なし |
| success / warning | ジョブ `T`(index より後ろ) | 両方 | `T` | index と T の間の未完了ジョブ / `byTransition`(空なら kind なし) | false | false | なし |
| success / warning | 遷移なし(または遷移先が後ろのジョブでない。検証済み定義では起きない) | 両方 | 最後のジョブなら なし、そうでなければ次のジョブ | 空 / なし | false | 最後のジョブなら true、そうでなければ false | なし |
| 定義にないジョブ | - | - | error と同じ(停止) | | | | |

要点:

- 成功・警告のあと再開位置は無し。エラー・分類なしのあとは必ずそのジョブ。
- Individual の `@end` では、後ろのジョブを今は記録せず(`SkippedJobIDs` は空)、`EndReached = true` だけを記録する。後ろのジョブは、スケジューラが 1 つずつ `run` を呼んだときに `Skip(AfterEnd)` が返り、呼ばれたジョブだけ `skipped(byEnd)` と記録される(§4.2、§5.4)。
- 遷移先 `T` の遷移では、`T` へ飛ぶ。`T` が index+1 なら間は空で、スキップは無い。
- 自動のリトライ(ジョブ失敗の再実行)は、`retry` を書かない限りしない(§7)。

### 3.3 `ApplyOutcome`(結果の記録。メモリ上)

入力: 現在のステート値 `S`、ジョブID、実行結果、結果(`JobOutcome`)、`NextStep`。`S` を複製して更新し、返す。**複製は深く行う**(`JobStates` のスライスを新しく作る。元の `S` を書き換えない)。

1. 対象ジョブ(`S` のジョブ状態一覧から id で探す。見つからなければこの手順 1 を飛ばす):
   - `Class` が `ClassNone` でなければ `Status` を success / warning / error にし、`SkipReason` を消す。`ClassNone`(起動前の中断)なら `Status` は**変えない**。
   - `ExitCode` = 実行結果の `ExitCode`(`nil` なら、以前の値も消す)。そのあと `Status` が success / warning / error のいずれでもなければ `ExitCode` を消す。
2. `SkippedJobIDs` の各ジョブ(対象ジョブ自身は除く)を `Status = skipped`、`ExitCode` 無し、`SkipReason = SkipReasonKind`(`""` なら `byTransition`)にする。
3. `ResumeJobID = NextStep.ResumeJobID`、`Completed = NextStep.Completed`。
4. `EndReached = S.EndReached || NextStep.EndReached`(一度 true になったら戻さない)。

### 3.4 `ApplySkip`(スキップの記録。メモリ上)

入力: ステート値 `S`、ジョブID、判断の理由(`DecisionReason`: `AlreadyExecuted` / `SkipTarget` / `AfterEnd` / `OutOfOrder`)、定義。

- 理由が `AlreadyExecuted` / `OutOfOrder` → 何も変えない(複製を返す)。
- `SkipTarget` → そのジョブを `skipped(byTransition)`、`AfterEnd` → `skipped(byEnd)` にし(`ExitCode` 無し)、`AfterEnd` なら `EndReached = true`。ジョブが `S` に無ければ、更新せず複製を返す。
- そのあと、`S` に `error` のジョブが 1 つでもあれば、`ResumeJobID` と `Completed` は**変えない**(エラーの再開位置を守る)。無ければ:
  - `firstPending` = 定義の記述順で最初の `pending`(記録なしを含む)のジョブ(無ければ無し)
  - `Completed` = 「いまスキップしたジョブが定義の最後のジョブ、かつ `firstPending` が無い」
  - `ResumeJobID = firstPending`

### 3.5 `ApplyInterrupt`(中断の記録。メモリ上)

再開位置(§9)の値で更新する: `ResumeJobID = ResumePoint.ResumeJobID`、`Completed = ResumePoint.Completed`、`EndReached = S.EndReached || ResumePoint.EndReached`。ジョブの状態は変えない(エラーとして記録しない)。

---

## 4. 起動前の判断(`DecideBeforeRun`)

### 4.1 入力と出力

入力: ジョブID、定義、各ジョブの状態の一覧、`ResumeJobID`(ステートの同名の値)、`EndReached`。ステートが空(ファイルなし・長さ 0・完了のまま残ったもの)のときは、状態一覧は空、`ResumeJobID = ""`、`EndReached = false` を渡す。

出力: `Run`(実行する)/ `Skip(理由)`(実行せず 0 で終わる。理由は `SkipTarget` / `AfterEnd` / `AlreadyExecuted`)/ `Reject(OutOfOrder)`(実行せず、ステートも変えず、66)。

### 4.2 判定表(上から最初に当てはまるものを採る)

状態 `st` = そのジョブの記録(無ければ pending)、`reason` = そのスキップ理由。

| 順 | 条件 | 結果 |
|---|---|---|
| 0 | ジョブIDが定義にない(`run` では、事前に 68 で止まるので到達しない) | `Reject(OutOfOrder)` |
| 1 | `st = skipped` かつ `reason = byEnd` | `Skip(AfterEnd)` |
| 1' | `st = skipped` かつ `reason` が byTransition または無し | `Skip(SkipTarget)` |
| 2 | `EndReached` かつ `st = pending` | `Skip(AfterEnd)` |
| 3 | `st` が success または warning | `Skip(AlreadyExecuted)` |
| 4 | `st = error`、または `ResumeJobID` がこのジョブID | `Run` |
| 5 | 全ジョブが pending(または記録なし)で、このジョブが記述順で最初 | `Run` |
| 6 | `st = pending` で、記述順で前のジョブがすべて success / warning / skipped | `Run` |
| 7 | 上のどれでもない | `Reject(OutOfOrder)` |

例(4 ジョブ j0〜j3):

| 呼ぶジョブ | 状態 | resume | EndReached | 結果 |
|---|---|---|---|---|
| j1 | j0=success, j1=skipped(byTransition) | - | false | Skip(SkipTarget) |
| j2 | j2=skipped(byEnd) | - | true | Skip(AfterEnd) |
| j3 | j0=success(j3 記録なし) | - | true | Skip(AfterEnd) |
| j0 | j0=success | - | false | Skip(AlreadyExecuted) |
| j0 | j0=warning | - | false | Skip(AlreadyExecuted) |
| j1 | j0=success, j1=error | j1 | false | Run |
| j1 | j1=error | - | false | Run |
| j2 | j0=success, j1=skipped | j2 | false | Run |
| j0 | (空) | - | false | Run |
| j0 | 全部 pending | - | false | Run |
| j2 | j0=success, j1=warning | - | false | Run |
| j3 | j0=success, j1=skipped, j2=skipped | - | false | Run |
| j1 | (空) | - | false | Reject |
| j2 | j0=success | - | false | Reject |
| j2 | j0=success, j1=error | - | false | Reject |

補足: 規則 1' は、理由が書かれていない `skipped`(ステートファイル上は読み込みで弾かれるが、関数としては入りうる)を SkipTarget とする。規則 2 は `pending`(記録なしを含む)のときだけ。`error` のジョブは規則 4 で、`EndReached` でも `Run` になる。規則 6 は「記述順で前のすべて」を見るので、後ろのジョブが済んでいても影響しない。

### 4.3 判断の結果とステート・終了コード(呼び出し側)

| 結果 | ステートの変更 | 標準エラー | 終了コード |
|---|---|---|---|
| `Reject(r)` | しない(初期ステートも書かない) | `conductor: error: job '<id>': cannot run this job now: earlier jobs have not finished (out of order)`(日本語: `このジョブは、いま実行できません: 前のジョブが終わっていません(順序外)`) | 66 |
| `Skip(r)` | `ApplySkip` の結果が元と**等しくなければ**、耐久性ありで書く。`AlreadyExecuted` は常に等しい(書かない) | `-v` のとき `[conductor] job=<id> skipped (<理由>)`。理由文: SkipTarget = `skipped by a transition`、AfterEnd = `skipped after @end`、AlreadyExecuted = `already executed` | 0(pipeline では続行) |
| `Run` | ステートが空なら先に初期ステートを、耐久性を省いて書く(§5) | - | ジョブの結果による |

「等しい」はステート値の全項目の比較(Go では `reflect.DeepEqual` か、型ごとの `Equal` メソッド。`ExitCode` の `*int32` は指す値で比べる。ポインタの同一性で比べてはならない)。「耐久性あり(Durable)」「省く(Fast)」の定義は [06-state-file.md](06-state-file.md) の書き込み。

---

## 5. 実行の流れ

### 5.1 共通の前処理 `prepare`(`run`・`pipeline`)

順序は固定。最初に誤りがあった時点で、その終了コードで終わる(以降は行わない)。

| 順 | 処理 | 誤りのときの終了コード |
|---|---|---|
| 1 | 中断の合図(「立った/立っていない」と最初のシグナル番号を保持する共有の旗)を作り、シグナルの監視を開始(定義の検査より前。`signal.Notify`)。`Notify` は失敗を返さないが、監視の準備(旗の生成・チャネルの用意・OS 別の初期化)に失敗したら失敗として扱う | 準備の失敗: 70(`conductor: error: failed to set up signal handling`) |
| 2 | 定義ファイルの読み込みと検証(書式は拡張子で決める。[03](03-definition-format.md))。`pipeline` は、引数の名前と定義の `pipeline.name` を照合 | 64 |
| 3 | 全ジョブの `command` の検査(引用符の閉じ忘れ・空) | 64 |
| 4 | `run` のとき、ジョブIDが定義にあるか | 68(`unknown job`) |
| 5 | run ID の決定(§6) | 68 |
| 6 | ステートの読み込み([06-state-file.md](06-state-file.md) §6)。完了のまま残ったものは「空」 | 65 |
| 7 | 空でなければ、定義との照合(06 §7) | 65 |
| 8 | 実効の `remove_on_complete` = `--remove-on-complete` があれば true、なければ定義の `pipeline.remove_on_complete`(省略時 false)。環境変数は見ない | - |

- 定義ファイルは `--config <パス>`、なければ実行ディレクトリの `conductor.yaml`。
- シグナルを受け取っても、1〜7 は最後まで進む。64・65・66・スキップ(0)の終了コードが、中断の終了コードに優先する。
- `pipeline` で引数の名前が定義の名前と違う場合は 64(順 2)。引数の名前が無いのは使い方の誤り 68(コマンドライン解析。[04-cli.md](04-cli.md))。

### 5.2 `conductor run <job-id>` の擬似アルゴリズム

```text
session = prepare(...)                      # 5.1。誤りなら終了
(states, resume, endReached) = session.state の JobStates / ResumeJobID / EndReached
                               (state が空なら [], "", false)
d = DecideBeforeRun(jobID, def, states, resume, endReached)

match d:
  Reject(r):  表示、return 66
  Skip(r):
    base = session.state か InitialState(...)(空なら)
    rec  = ApplySkip(base, jobID, r, def)
    if rec != base:
        write(rec, Durable)                  # 失敗: 65、文脈=ジョブ起動前
        if rec.Completed and removeOnComplete:
            resetState()                     # 失敗: 65
    -v なら skipped 表示
    return 0
  Run:
    base = session.state か、(空なら) InitialState を Fast で書いたもの   # 書けなければ 65、ジョブは起動しない
    exec = runJobWithRetry(job)              # §7。ステートは書かない
    (outcome, next) = ClassifyAndDecide(job, exec, base.JobStates, def, ModeIndividual)
    表示(§5.5)
    rec = ApplyOutcome(base, jobID, exec, outcome, next)
    write(rec, Durable)                      # 失敗: 65、文脈=exec.Started なら「ジョブは実行されました」、そうでなければ「起動していません」
    if next.Completed and removeOnComplete:
        resetState()                         # 失敗: 65
    return ExitCodeFor(outcome.ReportedExitCode, outcome.Class == ClassError)
```

注意:

- Individual では、遷移でスキップが決まったジョブ(`byTransition`)は、`ApplyOutcome` で記録済みなので、後で `run` されると `Skip(SkipTarget)`。
- 最後のジョブの `run` 完了、または `@end` 後の最後のジョブのスキップ記録で `Completed = true`。
- 初期ステートは `Run` と決まった後にだけ書く(Reject・Skip の前には書かない)。ただし Skip のときは、ステートが空なら `InitialState`(06 §2.4)を土台に記録が書かれる。
- `ClassifyAndDecide(job, exec, states, def, mode)` は、`Classify`(§2.3)で結果を作り、その分類を使って `NextStep`(§3.2)で次の動きを決めて、組で返す。

### 5.3 `conductor pipeline <name>` の擬似アルゴリズム

`pipeline` は、「次のジョブ」を `NextStep.NextJobID` で決めない。定義の記述順に全ジョブを 1 つずつ見て、`DecideBeforeRun` で決める(遷移先・スキップは、ステートが運ぶ)。

```text
session = prepare(...)                       # 引数の名前が無ければ 68
current = session.state                      # 空なら 無し
for job in def.jobs (記述順):
  d = DecideBeforeRun(job.ID, def, current の状態一覧 (無しなら []), current.ResumeJobID, current.EndReached)
  match d:
    Reject(r): 表示、return 66
    Skip(r):
      base = current か InitialState(...)
      rec  = ApplySkip(base, job.ID, r, def)
      if rec != base: write(rec, Durable)    # 失敗: 65
      -v なら skipped 表示
      current = rec
      if rec.Completed: return finish(...)   # 下記
    Run:
      base = current、無ければ InitialState を Fast で書いたもの   # 失敗: 65
      exec = runJobWithRetry(job)
      (outcome, next) = ClassifyAndDecide(job, exec, base.JobStates, def, ModePipeline)
      表示
      rec = ApplyOutcome(base, job.ID, exec, outcome, next)
      write(rec, Durable)                    # 失敗: 65
      if outcome は error、または分類なし:
          return ExitCodeFor(outcome.ReportedExitCode, outcome.Class == ClassError)   # エラーで停止
      if next.Completed: return finish(...)
      if 中断の合図が立っている(番号 n):      # ジョブのあいだの中断
          point = ResumePointOnInterrupt(def, rec.JobStates)
          write(ApplyInterrupt(rec, point), Durable)     # 失敗: 65
          -v なら「interrupted by signal n」
          return SignalExitCode(n)             # ExitCodeFor(SatAddInt32(128, n), true)
      current = rec
# ループを抜けた(完了していない。起こらない想定)
表示 "cannot determine whether the pipeline is complete"; return 65   # ステートは初期化しない

finish: if not removeOnComplete: return 0
        resetState() が成功 → 0、失敗 → 65
```

- 完了のステートは、`finish` の前に必ず Durable で書き込み済み。`remove_on_complete` が真のときだけ削除する(06 §9)。
- 「完了を判定できない」は、全ジョブが `AlreadyExecuted` などで通過し、どこでも `Completed` が真にならなかったとき。標準エラー `conductor: error: <ステートファイルのパス>: cannot determine whether the pipeline is complete`(日本語: `パイプラインが完了したかを判定できません`)、終了コード 65。
- ジョブ実行後、`next.Completed` の判定は、中断の合図の判定より**先**。完了していれば中断の記録はせず、`finish` へ進む。
- エラーのジョブで止まったときの再実行: 同じコマンドを再度呼ぶと、`success`/`warning` のジョブは `Skip(AlreadyExecuted)`(`-v` に `already executed`)、`error` のジョブから `Run`。
- `Skip` で `rec.Completed` が真になるのは §3.4 の条件(スキップしたジョブが最後のジョブで、pending が無い)のとき。

#### 状態遷移の例(`pipeline`)

定義 `a → b → c`(遷移なし)。`b` が exit 3 でエラー。

| 時点 | job_states | resume_job_id | end_reached | completed | 終了コード |
|---|---|---|---|---|---|
| 初期(Fast で書く) | a=pending, b=pending, c=pending | なし | false | false | - |
| a 成功後 | a=success(0), b=pending, c=pending | なし | false | false | - |
| b エラー後 | a=success(0), b=error(3), c=pending | b | false | false | 3 |
| 再実行: a は Skip(AlreadyExecuted)、b 成功 | a=success, b=success(0), c=pending | なし | false | false | - |
| c 成功後 | 全部 success | なし | false | true | 0 |

定義 `a --success--> @end` のとき(Pipeline): a 成功で b, c は `skipped(byEnd)`、`end_reached = true`、`completed = true`、0 で終わる([06-state-file.md](06-state-file.md) §4.2 の例 2 のステート)。

### 5.4 `@end` 後の `run`(Individual)の流れ

`run a`(遷移 `@end`)→ `a=success`、`end_reached = true`、`completed = false`(後ろに未完了がある場合)。続けて `run b` → 規則 2 で `Skip(AfterEnd)` → `b=skipped(byEnd)`、`resume_job_id` = 最初の pending(`c`)。`run c` → `Skip(AfterEnd)` → `c=skipped(byEnd)`、最後のジョブで pending が無いので `completed = true`(`remove_on_complete` なら削除)。`@end` の後ろに未完了が無ければ、`run a` の時点で `completed = true`。

### 5.5 表示(`-v` と常時)

- 常に標準エラー: エラー行(`conductor: error: ...`)、起動失敗、成功コードに無い 0、リトライの知らせ(§7.7)、`reset --all` の警告。
- `-v` のときだけ標準エラー(先頭 `[conductor] `): `job=<id> result=<success|warning|error|interrupted> (...)`、`transition: <success|warning> -> <target>[ (skipped: <id>, ...)]`、`job=<id> skipped (<理由>)`、`interrupted by signal <n>`。括弧内の詳細の厳密な文言は [08-output-and-i18n.md](08-output-and-i18n.md) に従う。
- ジョブの結果の行と遷移の行は、**最終の結果について 1 回**だけ(リトライの途中の結果は出さない)。
- 言語: 環境変数 `LC_ALL`、`LC_MESSAGES`、`LANG` の順で、最初の空でない値が `ja` で始まれば日本語、それ以外は英語(どれも空・未設定なら英語)。トークン(`success` `warning` `error` `skipped` `pending` `job=` `result=`)は常に英語。

---

## 6. run ID

run ID は、同じ定義を別々の実行単位で並行して使うときに、ステートファイルを分ける識別子。実装は `internal/runid`。

### 6.1 値の規則(`runid.Parse(s string) (RunID, error)`)

次の順に検査し、最初に当てはまる誤りを返す(誤りは使い方の誤り 68。誤りの文に値は含めない)。

| 順 | 誤り(エラー種別) | 条件 |
|---|---|---|
| 1 | `ErrEmpty` | 長さ 0 |
| 2 | `ErrTooLong` | 64 バイトを超える(`len(s) > 64`。許す文字は ASCII だけなので、文字数と同じ) |
| 3 | `ErrInvalidChar` | `A-Za-z0-9._-` 以外のバイトがある(バイト単位で見る。`for i := 0; i < len(s); i++`。`range` で rune に分けない) |
| 4 | `ErrLeadingDot` | 先頭が `.`(`..` を含む) |

有効: `1`, `r1`, `12345`, `build-2026.11_a`, `a.b`, 64 文字ちょうど。無効: `""`(ただし §6.2 の規則で空は「なし」)、65 文字、`a/b`、`a b`、`.x`、`..`、`日本語`(バイト数は 9 で 64 以下だが、非 ASCII なので順 3 で `ErrInvalidChar`)。

型の骨格:

```go
package runid

type RunID string // Parse が成功した値だけが取れる(非公開のフィールドを持つ構造体にしてもよい)

var (
    ErrEmpty       = errors.New("empty")
    ErrTooLong     = errors.New("too long")
    ErrInvalidChar = errors.New("invalid character")
    ErrLeadingDot  = errors.New("leading dot")
)

func Parse(s string) (RunID, error)
func (r RunID) String() string
```

### 6.2 決定(`runid.Resolve`、`prepare` の 5)

```text
envName = 定義の pipeline.run_id_env があればそれ、なければ "CONDUCTOR_RUN_ID"
if --run-id が指定された(値は空でもよい):
    value = そのオプションの値(subject = "--run-id")      # 環境変数は見ない
else:
    value = 環境変数 envName の値(未定義なら 無し)(subject = "run ID (environment variable)")
if value が 無し または "": run ID なし
else: Parse(value) が失敗なら 68(`invalid value: '<subject>'`)、成功なら run ID
```

- 環境変数の読み取りは `Deps.LookupEnv(name) (string, bool)`(`os.LookupEnv` 相当)。未定義と空文字列は、どちらも「なし」になる(区別しない)。
- `--run-id ""` は、環境変数があっても「run ID なし」に固定する。
- `run_id_env` の形は定義の検証で保証される(`A-Za-z_` で始まり `A-Za-z0-9_` が続く 1 文字以上)。
- 環境変数名は定義で決まるので、定義の検査(64)の**後**、ステートの読み込み(65)の**前**に run ID を決める。
- `watch`・`reset` は定義を検証しない: `--run-id` があれば定義を読まず(環境変数名は既定)、なければ定義の `pipeline.run_id_env` だけをゆるく読む([03](03-definition-format.md)のゆるい読み取り)。読めない・キーが無い・文字列でない・名前の形が違う、のどれでも「なし」= 既定の名前を使う。ほかの誤りは無視する。異常終了してはならない(ゆるい読み取りの関数は、構文の誤り・想定外の型で `panic` しないこと。YAML ライブラリの `panic` を `recover` で「なし」に変えてよい)。
- 環境変数の値が規則に合わなければ、`run`・`pipeline`・`watch`・`reset`(`--all` を除く)で 68。`reset --all` は環境変数を見ない。

### 6.3 run ID とステートの関係

- run ID あり: ステートは `<定義ファイル名>.<run-id>.state`。中身に `run_id` = run ID を書く。run ID なし: `<定義ファイル名>.state`、`run_id` の項目無し。
- 読み込み時、中身の `run_id`(あるか、値)がファイル名側の run ID と一致しなければ壊れたステート(65)。詳細は 06 §6、§7。
- run ID は同じ定義でも run ごとに独立したステート・再開位置を持つ。同じ run ID で呼び直すと、止まったジョブから再開する。

---

## 7. リトライ(`retry`、v0.8.0)

### 7.1 設定(検証済みの値。`RetryPolicy`)

| キー | 型・範囲 | 意味 |
|---|---|---|
| `exit_codes` | 整数の一覧。空でない。各値は 1〜255。昇順・重複なしに正規化済み | リトライの対象にするジョブ自身の終了コード。このジョブの成功コード・警告コードと重ならない |
| `max` | 整数 1〜100 | リトライの最大回数(最初の実行を除く)。実行は合計で最大 `max + 1` 回 |
| `interval` | 文字列 `<整数><s|m|h>`。結果が 1 秒〜24 時間(86400 秒) | 1 回目のリトライの前の待ち時間。数字は半角の数字だけ、整数は 1 以上。単位は小文字 1 文字(`s` 秒、`m` 分 = 60 秒、`h` 時間 = 3600 秒)。例 `"30s"`、`"2m"`、`"1h"` |
| `multiplier` | 整数 2〜10 | 待ち時間にかける倍率。`max_interval` と同時に指定 |
| `max_interval` | `interval` と同じ書式。`interval` 以上 | 待ち時間の上限。`multiplier` と同時に指定 |

- `multiplier` と `max_interval` は両方書くか両方書かない。検証の詳細は [03-definition-format.md](03-definition-format.md)。
- `retry` を書かないジョブは、1 回だけ実行する。

### 7.2 数値の型(Go 版の決定)

| 値 | Go の型 | 理由 |
|---|---|---|
| 待ち時間(秒。`interval`、`max_interval`、各回の待ち) | `uint64` | 64 ビット符号なし整数の秒とする。乗算の飽和も 64 ビット符号なしで定める |
| 倍率 `multiplier` | `uint32`(範囲 2〜10) | 32 ビット符号なし整数に収まる値とする |
| `max`、`retriesDone`、`n`(知らせの回数) | `uint32`(`max` は 1〜100) | |
| `exit_codes` の各値 | `int32`(1〜255) | ジョブの終了コードの型と同じ |
| `time.Duration`(待機の実行時だけ) | `int64` のナノ秒 | §7.3 の変換だけで使う。**待ち時間の計算には使わない** |

浮動小数点は使わない。丸めは無い(整数の秒だけ)。計算は `uint64` の秒で完結し、`time.Duration` には待機の直前で一度だけ変換する。

`RetryPolicy` の骨格:

```go
type RetryPolicy struct {
    ExitCodes    []int32  // 昇順・重複なし・各 1〜255・空でない
    Max          uint32   // 1〜100
    IntervalSecs uint64   // 1〜86400
    Backoff      *Backoff // nil なら固定間隔。multiplier と max_interval は必ず同時に有る/無い
}

type Backoff struct {
    Multiplier      uint32 // 2〜10
    MaxIntervalSecs uint64 // IntervalSecs 以上、86400 以下
}
```

(`Backoff` を 1 つの構造体にまとめるのは、「両方有るか両方無い」を型で保証するための Go 版の決定。定義の検証で片方だけは `retryBackoffIncomplete` として拒否されるので、`Backoff` が部分的に有る状態は作れない。)

### 7.3 `interval` / `max_interval` の文字列から秒への変換と、`time.Duration` への変換

文字列 → 秒(`ParseDurationSecs(s string) (secs uint64, ok bool)`。定義の検証で使う。置き場所は `internal/definition`):

```text
1. len(s) < 2 なら ok=false
2. 最後の 1 バイトが 's' / 'm' / 'h' のどれでもなければ ok=false(大文字 'S' や 'ms' の 'm' の前の 's' は、数字でない文字が残るので次で弾かれる)
3. 残り(digits)が、ASCII '0'〜'9' だけの 1 バイト以上でなければ ok=false
   (符号・空白・小数点・'_'・全角数字・"0x" は、ここで弾かれる。strconv の解釈には頼らず、バイトで検査する)
4. digits の先頭の '0' を取り除く(すべて '0' なら "0" を残す)。取り除いたあと:
     - 長さが 20 を超えるなら ok=false(uint64 の最大値は 20 桁。86400 秒を超えるので、どのみち拒否される)
     - n = strconv.ParseUint(digits, 10, 64)。エラー(範囲外を含む)なら ok=false
5. 単位ごとの上限で、乗算の前に比べる(乗算のオーバーフローを避ける):
     's': 1 <= n <= 86400
     'm': 1 <= n <= 1440
     'h': 1 <= n <= 24
   外れたら ok=false
6. secs = n * 単位の秒数(1 / 60 / 3600)。上限を先に比べたので、オーバーフローしない
```

例: `"1s"`→1、`"01s"`→1、`"0000000000000000000000000001s"`→1、`"1440m"`→86400、`"24h"`→86400、`"86400s"`→86400。拒否: `""`、`"s"`、`"10"`、`"1.5s"`、`"-1s"`、`"+1s"`、`" 1s"`、`"1s "`、`"1S"`、`"１s"`(全角 1)、`"1ms"`、`"0x10s"`、`"1_0s"`、`"0s"`、`"0h"`、`"86401s"`、`"25h"`、`"99999999999999999999999s"`、`"18446744073709551615h"`(手順 5 で拒否。`18446744073709551615 * 3600` を計算しない)。

秒 → `time.Duration`(`DurationFromSecs(secs uint64) time.Duration`。待機の直前だけで使う):

```text
const maxSecs = uint64(math.MaxInt64 / int64(time.Second))   # = 9223372036(約 292 年)
if secs > maxSecs: return time.Duration(math.MaxInt64)        # 飽和。折り返さない
return time.Duration(secs) * time.Second
```

- `time.Duration` は `int64` のナノ秒。`time.Duration(secs) * time.Second` は `secs > 9223372036` でオーバーフローして負数または不正な値になるので、上の上限判定を必ず先に行う。
- 検証済みの定義では待ち時間は 86400 秒以下(`DurationFromSecs(86400) = 86400 * 10^9 = 86,400,000,000,000` ns で、`int64` の範囲に十分収まる)なので、飽和の分岐は実際の定義では通らない。単体テストでだけ検証する(境界: `DurationFromSecs(9223372036)` は `9223372036000000000`、`DurationFromSecs(9223372037)` と `DurationFromSecs(math.MaxUint64)` は `math.MaxInt64`)。
- 待機は `Duration` を直接 `time.Sleep` へ渡さず、§7.6 の `WaitFor` を使う(シグナルから 1 秒以内に打ち切るため)。

### 7.4 リトライの対象か(`IsRetryTarget`)

次の**すべて**が成り立つときだけ真。

1. `Started = true`
2. `Interrupted = false`
3. `StartFailure = false`
4. `TerminatedBySignal == nil`
5. `ExitCode != nil` かつ `*ExitCode` が `ExitCodes` に含まれる

分類(成功・警告・エラー)は見ない。対象の終了コードは成功・警告と重ならない(検証済み)ので、結果として対象はエラーの終了コードだけ。ジョブ自身が 127 で終わった場合は、`exit_codes` に 127 を書いていれば対象(conductor が付けた起動失敗の 127 は、`ExitCode` を持たないので対象にならない)。

### 7.5 判断(`RetryDecision(job, execution, retriesDone, lastWaitSecs)`)

```text
policy = job.Retry が nil なら return Stop
if not IsRetryTarget(policy, execution) or retriesDone >= policy.Max: return Stop
return Retry{ WaitSecs: NextWaitSecs(policy, retriesDone, lastWaitSecs) }
```

### 7.6 待ち時間の式

```text
NextWaitSecs(policy, retriesDone uint32, lastWaitSecs uint64) uint64:
  if policy.Backoff != nil:
      if retriesDone == 0: return min(policy.IntervalSecs, policy.Backoff.MaxIntervalSecs)   # = IntervalSecs(検証で IntervalSecs <= MaxIntervalSecs)
      else:                return min(SatMulU64(lastWaitSecs, policy.Backoff.Multiplier), policy.Backoff.MaxIntervalSecs)
  else:
      return policy.IntervalSecs                                                              # 毎回同じ(固定間隔)
```

- k 回目のリトライ(k = `retriesDone + 1`)の前の待ち時間を `W(k)` とすると: 固定間隔なら `W(k) = interval`。倍率ありなら `W(1) = interval`、`W(k) = min(W(k-1) × multiplier, max_interval)`(k ≥ 2)。
- 倍率の掛け算は、直前の**実際の**待ち時間(上限で頭打ちされた後の値)に対して行う。
- `min` は 2 つの `uint64` の小さい方(Go 1.21 以降の組み込み関数 `min` が使える。go.mod の Go の版は 1.25 以降なので使ってよい)。

`SatMulU64(a uint64, b uint32) uint64`(64 ビット符号なし整数の飽和乗算):

```text
hi, lo = bits.Mul64(a, uint64(b))      # math/bits。128 ビットの積の上位・下位 64 ビット
if hi != 0: return math.MaxUint64      # 18446744073709551615 を超えた
return lo
```

`a × b` が 2^64 - 1(18446744073709551615)を超えるなら 18446744073709551615、そうでなければ `a × b`。折り返し(`a * b` を直接書いて `uint64` の桁あふれに任せる)は禁止。

境界の例(関数単体の挙動):

| a | b | 結果 | 理由 |
|---|---|---|---|
| 0 | 10 | 0 | |
| 86400 | 10 | 864000 | 検証済みの定義で起こりうる最大の積 |
| 1844674407370955161 | 10 | 18446744073709551610 | 飽和しない最大に近い値(`2^64-1` を 10 で割って切り捨てた値 × 10) |
| 1844674407370955162 | 10 | 18446744073709551615 | 積 `18446744073709551620` が `2^64-1` を超えるので飽和 |
| 18446744073709551615 | 2 | 18446744073709551615 | 飽和 |
| 9223372036854775807 | 10 | 18446744073709551615 | 飽和 |

`NextWaitSecs` の境界例(関数単体。検証済みの定義では起きない入力):

| IntervalSecs | Multiplier | MaxIntervalSecs | retriesDone | lastWaitSecs | 結果 |
|---|---|---|---|---|---|
| 10 | 2 | 60 | 0 | 0 | 10 |
| 10 | 2 | 60 | 3 | 40 | 60(80 を 60 で頭打ち) |
| 10 | 2 | 60 | 4 | 60 | 60 |
| 1 | 10 | 18446744073709551615 | 1 | 9223372036854775807 | 18446744073709551615(積が飽和し、上限で止まる) |
| 30 | (Backoff なし) | - | 5 | 30 | 30 |

検証済みの定義では `interval` と `max_interval` が 86400 秒以下なので、`SatMulU64` が飽和する場面は無い(最大でも 86400 × 10)。再実装でこの極端な入力まで再現する必要は実用上ないが、乗算は桁あふれで折り返さないこと。

待ち時間の列の例:

| interval | multiplier | max_interval | max | 待ち時間の列(秒) | 合計 |
|---|---|---|---|---|---|
| 30s | なし | なし | 3 | 30, 30, 30 | 90 |
| 10s | 2 | 60s | 5 | 10, 20, 40, 60, 60 | 190 |
| 60s | 3 | 60s | 3 | 60, 60, 60 | 180 |
| 1s | 10 | 3600s(1h) | 6 | 1, 10, 100, 1000, 3600, 3600 | 8311 |
| 1m | 2 | 24h | 4 | 60, 120, 240, 480 | 900 |
| 1h | 10 | 24h | 4 | 3600, 36000, 86400, 86400 | 212400 |

### 7.7 実行ループ(`runJobWithRetry`)。`run` と `pipeline` の共通

```text
retriesDone = 0; lastWaitSecs = 0
loop:
    exec = RunJob(job, interrupt)
    match RetryDecision(job, exec, retriesDone, lastWaitSecs):
        Stop: return exec                          # 最後の結果
        Retry{waitSecs}:
            表示: 常に 1 行(§7.9)。retryNumber = retriesDone + 1
            match WaitFor(waitSecs, interrupt):
                Elapsed:        retriesDone += 1; lastWaitSecs = waitSecs; continue
                Interrupted(n): return JobExecution{ Started=false, Interrupted=true, SignalNumber=&n,
                                                      ExitCode=nil, StartFailure=false, TerminatedBySignal=nil }
```

- 返された最後の結果だけが、分類・表示・ステート記録の対象(ステートは最終の結果で 1 回だけ書かれ、途中では書かれない)。リトライの回数・直前の待ち時間はローカルな値で、ステートファイルに書かない。
- したがって、conductor が落ちたあとの再開では、そのジョブを最初の実行から数え直す。
- 使い切った(`retriesDone == Max` で、まだ対象の終了コード): `Stop` → 通常のエラーと同じ(最後の実行の終了コードで終わり、再開位置はそのジョブ)。再実行の結果が成功・警告なら通常どおり遷移へ進む。
- 総実行回数: 最大 `Max + 1`。知らせの回数: 実際に実行し直す(または待機に入る)回数。
- 起動前に中断された(合図が立っていた)ジョブはリトライせず、その場で `Started = false` の結果を返す(`RetryDecision` は `Started = false` で Stop)。

### 7.8 待機とシグナル(`WaitFor(secs uint64, interrupt) WaitResult`)

実装は `internal/runner`。時計は `Deps` の時計(`Now() time.Time`、`NewTimer`)で差し替えられるようにする。

```text
d = DurationFromSecs(secs)
if 中断の合図が立っている: return Interrupted(最初の番号)        # 待機の開始時にも確かめる(select の乱択に頼らない)
timer = 時計.NewTimer(d)
select:
    case <-timer.C:           # 合図のチャネルも同時に準備できていたら、合図を優先して再確認する
        if 中断の合図が立っている: return Interrupted(最初の番号)
        return Elapsed
    case <-interrupt.Done():  # 最初のシグナルで閉じられるチャネル
        timer を停止
        return Interrupted(最初の番号)
```

- `select` は準備ができている複数の case から擬似乱数で 1 つ選ぶ。このため、タイマー側が選ばれたあとにもう一度合図を確認する(上の擬似コード)。守るべきは、「合図から 1 秒以内に打ち切る」「待ち時間が `secs` 秒以上になるまで `Elapsed` を返さない」。
- 経過時間の測定は、単調増加の時計を使う。Go の `time.Now()` が返す `time.Time` は単調時計の値を持ち、`time.Since`・`Sub` はそれを使う(実時計の調整に影響されない)。タイマーを使う場合は `time.Timer` が「少なくとも `d` 後」に発火することに頼ってよい。ポーリングで実装する場合は、50 ミリ秒以下の刻みで合図を確かめ、`time.Since(start) >= d` で `Elapsed` とする。どちらの実装でもよい。
- 待機中に中断された場合は、次の実行をせず、分類なし(起動しなかった中断)として扱う: 終了コード `128 + 番号`、ジョブの状態は変えず(エラーとして記録しない)、再開位置はそのジョブ。`pipeline` でも、そこで止まって終わる(§5.3: 分類なしは停止)。
- 再開すると、そのジョブを最初の実行から数え直す。

### 7.9 リトライの知らせ

待機の**前**に、`-v` の有無によらず、標準エラーに 1 行(エラーではない。`conductor: retry: ` で始まる)。待機が中断された場合も、その回の知らせは出ている。

- 英語: `conductor: retry: job '<id>' exited with code <last_exit_code>; retry <n> of <max> in <wait_secs> seconds`
- 日本語: `conductor: retry: ジョブ '<id>' が終了コード <last_exit_code> で終わりました。<wait_secs> 秒後にリトライします(<n>/<max> 回目)`
- `<last_exit_code>` は直前の実行の終了コード(`ExitCode` が `nil` なら 0)。`<n>` は 1 から。`<wait_secs>` は秒の整数(10 進、桁区切りなし。`strconv.FormatUint(waitSecs, 10)`)で、1 のときも `seconds`(単数形の処理はない)。`<id>` は表示用にエスケープ・長さ制限される(規則は [08-output-and-i18n.md](08-output-and-i18n.md))。
- 例: `conductor: retry: job 'extract' exited with code 75; retry 2 of 5 in 20 seconds`

---

## 8. シグナルと中断(流れへの影響)

| 受信のタイミング | ジョブの結果 | ステートの記録 | 終了コード |
|---|---|---|---|
| ジョブの起動の前(`run`) | 起動しない。分類なし | ジョブの状態は変えない。`resume_job_id` = そのジョブ(初期ステートは、空だった場合に書かれている) | 128 + n |
| ジョブの実行中 | 子に同じシグナルを 1 回転送。終了を待つ。`Interrupted`。分類 error | そのジョブを `error` で記録(`exit_code` はジョブの値があればそれ) | 128 + n |
| リトライの待機中 | 分類なし(§7.8) | 状態は変えない。再開位置 = そのジョブ | 128 + n |
| `pipeline` で、ジョブを実行・記録した直後(まだ完了していない) | - | 再開位置(§9)を記録 | `SignalExitCode(n)` = 128 + n |
| 定義の検査・ステートの確認の途中 | 検査は最後まで進む | 64・65・66・スキップ(0)が優先 | それぞれ |

- シグナルは SIGHUP・SIGINT・SIGQUIT・SIGTERM(UNIX 系)、Ctrl+C(Windows。`os.Interrupt`)。2 回目以降のシグナルは追加の動作をしない。終了コードに使う番号は、最初に受け取ったもの。詳細は [07-process-and-signals.md](07-process-and-signals.md)。
- 受け取った `os.Signal` から番号を得る方法: UNIX 系では `sig.(syscall.Signal)` の `int32` への変換。Windows の `os.Interrupt` は `syscall.SIGINT`(番号 2)で、終了コードは 130 になるはず(Windows での実機は未確認。§13)。型アサーションが失敗する(想定外の `os.Signal` の実装)場合は、番号を 0 として扱い、終了コードを 128 にする。
- 実行中の中断ではジョブ自身の終了コードは見ない(分類 error、返す値は 128 + n)。

---

## 9. 再開位置(`ResumePointOnInterrupt`)

入力: 定義、各ジョブの状態。

```text
resume_job_id = 記述順で最初の「pending(または記録なし)または error」のジョブ(無ければ 無し)
completed     = (resume_job_id == 無し)
end_reached   = 「skipped(byEnd)」のジョブが 1 つでもあれば true
```

エラー・pending を区別せず、記述順で最初のものが再開位置。エラーのジョブが前にあれば、そのジョブ。この関数は記録を作らない(呼び出し側が `ApplyInterrupt`(§3.5)で書く)。

---

## 10. 完了と完了後

- 完了(`completed = true`)になるのは: 最後のジョブが success / warning で終わった、`@end` に到達した(Pipeline、または Individual で後ろに未完了が無い)、スキップの記録で最後のジョブまで済んだ(§3.4)。
- 完了のステートは `completed = true` のまま**残る**(`remove_on_complete` が真でない限り)。書式は変わらない。
- 完了のステートは、読み込み(`run`・`pipeline` の `prepare`)では「空」として扱う(定義との照合もしない)。次の `pipeline`・`run <最初のジョブ>` は先頭から始まり、初期ステートで上書きする。先頭以外のジョブを `run` で単独に呼ぶと順序外(66)。
- `remove_on_complete` が真のとき、完了のステートを書いた後に `resetState` で削除する(失敗は 65。完了のステートは残る)。偽のときは削除を試みず、0 で終わる(`run` ではジョブの結果による終了コード)。
- `watch` は完了のステートも表示する(表の下に `completed`/`完了` の行)。

---

## 11. 保証の範囲(at-least-once)

- 結果の記録は、ジョブの終了の**後**。終了と書き込みのあいだに conductor が落ちると、ジョブは未実行として扱われ、再実行されうる(ジョブが 2 回動きうる)。
- 結果の書き込みに失敗したときは、ジョブの終了コードではなく 65 で終わり、「ジョブは実行されました」と表示する。
- `retry` のジョブは、最大 `max + 1` 回動く。
- 時間の上限(タイムアウト)はない。ジョブが終わらなければ待ち続ける。
- 排他制御はしない([06-state-file.md](06-state-file.md) §11)。

---

## 12. 再現すべき特記事項

| # | 項目 | 挙動 |
|---|---|---|
| 1 | リトライの知らせの単数形 | 待ちが 1 秒のとき `in 1 seconds`(英語)。単数形の処理はない |
| 2 | エラーのジョブを再実行して、起動前・待機中に中断されたとき | 分類なしなので `status` は error のまま変わらないが、`exit_code` は消える(`ApplyOutcome` が `ExitCode` に実行結果の値 = 無しを書く) |
| 3 | 成功直後の `resume_job_id` | 成功・警告のあとの `NextStep.ResumeJobID` は無し。ただし後続のスキップ記録(`ApplySkip`)は、エラーが無いとき `ResumeJobID` を最初の pending にする。そのため、遷移で飛ばした後のステートには、成功直後でも `resume_job_id` が入りうる |
| 4 | 分類なし(起動前の中断)の終了コードの整形 | `isError = false` で整形する。`128 + n` が 256 以上かつ下位 8 ビットが 0 になる番号(n = 128 など)は、0 になりうる(実在のシグナル番号では起きない) |
| 5 | `pipeline` の「完了を判定できない」 | 全ジョブを通過しても `Completed` が真にならなければ 65(ステートは初期化しない) |
| 6 | ステートの誤りの原因の文 | 種類の文はロケールに従うが、理由(例: `JSON として読めません`)は常に日本語の固定文(06 §6.2) |

---

## 13. `internal/flow` の純粋関数の署名の骨格

実装は書かない。型とシグネチャだけを示す(名前は推奨。外から見える挙動が同じなら変えてよい)。`flow` の関数は、`error` を返さず、`panic` もせず、入力を書き換えない。

```go
package flow

// ---- 共有の値の型(state も使う) ----

type JobStatus string

const (
    StatusPending JobStatus = "pending"
    StatusSuccess JobStatus = "success"
    StatusWarning JobStatus = "warning"
    StatusError   JobStatus = "error"
    StatusSkipped JobStatus = "skipped"
)

type SkipReason string // "" = 無し

const (
    SkipByTransition SkipReason = "byTransition"
    SkipByEnd        SkipReason = "byEnd"
)

type JobState struct {
    ID         string
    Status     JobStatus
    ExitCode   *int32 // success / warning / error のときだけ有りうる
    SkipReason SkipReason
}

type RunMode int

const (
    ModeIndividual RunMode = iota
    ModePipeline
)

type JobExecution struct {
    Started            bool
    ExitCode           *int32
    TerminatedBySignal *int32
    StartFailure       bool
    Interrupted        bool
    SignalNumber       *int32
}

type Class int

const (
    ClassNone Class = iota // 分類なし(起動しなかった中断)
    ClassSuccess
    ClassWarning
    ClassError
)

type JobOutcome struct {
    Class            Class
    ReportedExitCode int32
    ConductorError   bool
}

type NextStepResult struct {
    NextJobID      string // "" = 無し
    SkippedJobIDs  []string
    SkipReasonKind SkipReason // "" = なし
    EndReached     bool
    Completed      bool
    ResumeJobID    string // "" = 無し
}

type DecisionKind int

const (
    DecisionRun DecisionKind = iota
    DecisionSkip
    DecisionReject
)

type DecisionReason int

const (
    ReasonNone DecisionReason = iota
    ReasonAlreadyExecuted
    ReasonSkipTarget
    ReasonAfterEnd
    ReasonOutOfOrder
)

type Decision struct {
    Kind   DecisionKind
    Reason DecisionReason // Skip / Reject のとき
}

type ResumePoint struct {
    ResumeJobID string // "" = 無し
    Completed   bool
    EndReached  bool
}

type RetryKind int

const (
    RetryStop RetryKind = iota
    RetryAgain
)

type RetryResult struct {
    Kind     RetryKind
    WaitSecs uint64 // RetryAgain のとき
}

// ---- 数値 ----

func ExitCodeFor(reported int32, isError bool) int
func SatAddInt32(a, b int32) int32
func SignalExitCode(signalNumber int32) int
func SatMulU64(a uint64, b uint32) uint64

// ---- 判断(definition の型を使う) ----

func Classify(job *definition.Job, exec JobExecution) JobOutcome

// states は定義のジョブ数と同じ長さとは限らない(記録なしは pending として扱う)。
func ClassifyAndDecide(job *definition.Job, exec JobExecution, states []JobState,
    def *definition.Definition, mode RunMode) (JobOutcome, NextStepResult)

func NextStep(job *definition.Job, class Class, states []JobState,
    def *definition.Definition, mode RunMode) NextStepResult

func DecideBeforeRun(jobID string, def *definition.Definition, states []JobState,
    resumeJobID string, endReached bool) Decision

func IsRetryTarget(policy *definition.RetryPolicy, exec JobExecution) bool
func RetryDecision(job *definition.Job, exec JobExecution, retriesDone uint32, lastWaitSecs uint64) RetryResult
func NextWaitSecs(policy *definition.RetryPolicy, retriesDone uint32, lastWaitSecs uint64) uint64

func ResumePointOnInterrupt(def *definition.Definition, states []JobState) ResumePoint
```

テストの指針: 上の表(§2.2、§2.3、§3.2、§4.2、§7.6)の行を、そのままテーブル駆動テストの 1 行にできる粒度にしてある。

---

## 14. 実装に任せてよい点と未確認

実装に任せてよい点(外から見える挙動が同じなら自由):

- 判断関数・記録関数の内部構造、データ型の表現(上の骨格は推奨)。
- 待機の実現方法(§7.8。タイマーでもポーリングでもよい)、シグナル監視の仕組み。
- 標準エラーの出力のバッファリング(ただし行単位で、順序を保つ)。

### この版が採った選択

| # | 点 | 内容 |
|---|---|---|
| 1 | 既定の定義ファイル名 | `conductor.yaml`(拡張子 `.yaml`/`.yml`/`.json` で書式を決める) |
| 2 | ステートの書式 | JSON(06) |
| 3 | 理由文のうち書式に関わるもの | `JSON として読めません` ほか(06 §6.4 の全件表) |
| 4 | 「なし」の表現 | `nil` ポインタと空文字列 `""`(ジョブIDは空でないことが保証される) |
| 5 | 待機 | `select` とタイマーで実装してよい(合図から 1 秒以内の打ち切りは同じ) |
| 6 | `Backoff` の構造体 | `multiplier` と `max_interval` を 1 つの `*Backoff` にまとめ、「両方有るか両方無い」を型で保証 |
| 7 | Windows の終了コード | `int` で返る値(0〜`MaxUint32`)を `int32` へ 2 の補数で再解釈して負数にする(§1.2) |

### 未確認

- Windows での、実行中ジョブへのシグナル転送の細部、`exec.ExitError.ExitCode()` が `MaxInt32` を超えたときの実機の値、`os.Interrupt` の番号(`syscall.SIGINT` = 2 のはずだが、実機で確認していない)。
- ジョブの標準入出力の扱い(子プロセスが conductor の入出力を引き継ぐか)。この文書の対象外。実装が採る方式は [07-process-and-signals.md](07-process-and-signals.md) を参照。
- 複数シグナルがほぼ同時に届いたときの「最初の番号」の決まり方。`signal.Notify` のチャネルに届く順序は、到着の順になるはずだが、ほぼ同時の到着の再現は未確認。再実装では「最初に観測した 1 つ」を保持すれば足りる(2 つ目以降は無視する)。
- `bits.Mul64` と組み込み関数 `min` は Go の標準の機能(それぞれ Go 1.12、1.21 以降)として使うが、採用する Go の版で利用できることを、実装時に確認する。
