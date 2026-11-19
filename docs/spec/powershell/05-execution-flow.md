# 05. 実行の意味論(終了コード・分岐・再開・run ID・リトライ・パイプライン)

対象: PowerShell 7.x(`pwsh`)で実装する `conductor` v0.8.0 と同じ挙動の実行ツール。この文書は、`run`(ジョブを 1 つ実行)と `pipeline`(定義の全ジョブを順に実行)の実行の流れを、判定表・状態遷移・擬似コードで定める。数値の計算(待ち時間の倍率・上限・飽和・秒から待機への変換)は、`[bigint]` で計算して `[uint64]` の範囲に収める飽和演算として、式・境界の例・落とし穴まで定める。待機は同期で、シグナルのフラグを短い間隔で見る方式に決める(§7.8)。この文書だけで、PowerShell でゼロから同じ挙動を実装できることを目標にする。

ステートファイルの論理項目・保存の性質・読み込みの検査は [06-state-file.md](06-state-file.md) にある。この文書の「ステートへの記録」は、06 の形式のファイルを書く前の、メモリ上のステート(以下「ステート値」)の更新を指す。定義ファイルの書式は [03-definition-format.md](03-definition-format.md)、コマンドラインは [04-cli.md](04-cli.md)、プロセス起動とシグナルは [07-process-and-signals.md](07-process-and-signals.md)、メッセージの文言は [08-output-and-i18n.md](08-output-and-i18n.md)、終了コードの総覧は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md) を参照。

## 確認の範囲(この文書の根拠)

この文書を書いた環境には PowerShell(`pwsh`)も .NET SDK も無く、実機で動かせていない。したがって、コードの挙動は実行して確かめていない。API と言語の挙動は、learn.microsoft.com の公式ドキュメント(PowerShell 7.4 版の `about_*` と各コマンドレットのページ)を取得して確認できたものだけを「確認(出典: …)」と書き、取得できなかったもの・記載が無かったものは「未確認」と書く(§17 に一覧と、実装時に最初に行う小さな試験)。算術の境界値の表は、任意精度整数で計算して確かめた値で、PowerShell で実行した結果ではない。実装時に、§15 の単体テストで固定する。

## 0. 構成の決定

### 0.1 置き場所と責務

「判断」(分類・次の動き・起動前の判断・リトライの判断・再開位置)と「記録」(ステート値の複製と更新)は、ファイルにもプロセスにも時計にも環境変数にも触れない**純粋な関数**にし、例外を投げず必ず結果を返す。ファイルは `src/Conductor/Private/` の下(全体の構成は [02-architecture.md](02-architecture.md))。

| 置き場所 | 置くもの(関数名は `Verb-ConductorNoun`) |
|---|---|
| `Private/Flow/` | 値の生成関数(`New-ConductorJobState`、`New-ConductorPipelineState`、`New-ConductorJobExecution`、`New-ConductorJobOutcome`、`New-ConductorNextStep`、`New-ConductorDecision`)、`Get-ConductorOutcome`(§2.3 の分類)、`Get-ConductorClassifiedStep`(分類と次の動きの組)、`Get-ConductorNextStep`、`Get-ConductorDecision`(起動前の判断)、`Get-ConductorRetryDecision`、`Get-ConductorNextWaitSecs`、`Get-ConductorResumePoint`、`ConvertTo-ConductorProcessExitCode`(§2.2 の整形)、`Get-ConductorSignalExitCode`、`Add-ConductorInt32Saturating`、飽和乗算(`Get-ConductorSaturatedProduct`)、記録の関数(`New-ConductorInitialState`、`Set-ConductorOutcomeRecord`、`Set-ConductorSkipRecord`、`Set-ConductorInterruptRecord`)、`Test-ConductorStateEqual`。`System.IO`・`System.Diagnostics`・`[Environment]`・`Get-Date`・`Start-Sleep` を使わない |
| `Private/RunId/` | run ID の検査(§6.1)と決定(§6.2。環境変数の読み取りは引数で渡された関数(スクリプトブロック)を通す) |
| `Private/Runner/` | 子プロセスの起動(`Invoke-ConductorJob`)、リトライの待機(`Wait-ConductorRetry`、§7.8)、中断の合図(07 章)。シグナル・時計・スレッドに触れる |
| `Private/State/` | 読み書き・整合性検査・`reset`([06](06-state-file.md)) |
| `Private/Cli/` | `Invoke-ConductorPrepare`、`run`・`pipeline` の流れ(§5)、リトライの実行ループ(§7.7) |

依存の向きは `Cli` → `Runner`/`State`/`Report` → `Flow`/`Definition`/`Convert`/`RunId` → `Json`。`Flow` は定義の読み込み結果(03 章)のうち判断に必要な部分(§13 の `FlowDefinition`)だけを受け取る。PowerShell には名前空間による依存の強制が無いので、`tests/` に、`Private/Flow/` の各ファイルを AST で走査し、禁止する呼び出し(上の表の「使わない」もの、および `Write-Host`・`Write-Output`・`Invoke-Expression`)が無いことを確かめるテストを置く([11-quality-and-build.md](11-quality-and-build.md))。

### 0.2 型の約束(この文書の全体)

| 値 | PowerShell の型 | 理由・注意 |
|---|---|---|
| 終了コード(ジョブ自身のもの、`ReportedExitCode`、成功・警告コードの一覧の値) | `[int]` | 32 ビット符号付き。整形(§2.2)は `-band 0xFF` を使う。2 の補数のビット列として下位 8 ビットが取れる(例: `-1 -band 0xFF` は 255、`-256 -band 0xFF` は 0)ことを §15 のテストで固定する(確認: `about_Arithmetic_Operators` に `-bnot 10` が -11 になる例があり、符号付き整数を 2 の補数で扱う。負数の `-band` の結果そのものを示す記載は無いので、結果は未確認) |
| 待ち時間の秒(`IntervalSecs`、`MaxIntervalSecs`、各回の待ち、`lastWaitSecs`) | `[uint64]`(値の範囲 0〜2^64-1) | 飽和演算(§7.6)は `[bigint]` で計算する。`[double]`・`[decimal]`・`[long]` に変換して計算しない |
| 倍率 `Multiplier` | `[int]`(2〜10) | 飽和乗算に渡すとき `[bigint]` に変換 |
| `Max`、`retriesDone`、知らせの回数 `n` | `[int]` | `Max` は 1〜100 |
| シグナル番号 | `[int]` | 終了コード `128 + n` の計算に使う。飽和加算(§2.2) |
| 時刻・経過 | `[long]` の `[System.Diagnostics.Stopwatch]::GetTimestamp()`(ティック) | 単調な時計だけを使う(§7.8)。`Get-Date` や `[DateTime]::UtcNow` は待機の計測に使わない |
| 真偽 | `[bool]` | 値の有無は `$null` で表す(下) |

- **「無し」は `$null`**。値のあるなしを持つ項目(`ExitCode`、`TerminatedBySignal`、`SignalNumber`、`SkipReason`、`ResumeJobId`、`RunId`)の変数・パラメーターは、`[int]` や `[string]` で型を固定しない(型付きの変数・パラメーターに `$null` を代入すると、`[int]` は 0、`[string]` は空文字列に変換される。確認: 公式ドキュメントでは取得できなかった。一般に知られた挙動として扱い、§17 の試験 T-4 で固定する)。パラメーターは `[AllowNull()] [object]` とし、値の有無は `$null -ne $x`(`$null` を左辺に置く。§14.3)で判定する。有る値は `[int]` / `[string]` の値そのもの(ボックス化された値)。0 と無しを混ぜない: `if ($exitCode)` と書かない(0 が偽になる。§14.3)。ステートの JSON の `null` は、読み込みで型の誤りとして拒否する([06](06-state-file.md))ので、メモリ上の「無し」と JSON の `null` は混ざらない。
- **値の表現: `[pscustomobject]`**(`class` は使わない)。`PipelineState` など項目の多い値は、`[pscustomobject]@{ ... }`(リテラルのハッシュテーブルから作る。項目の並びは記述順)で作る。理由: (a) `class` は、モジュールの再読み込み(`Import-Module -Force`)で型が再定義され、古い型の値との不整合・`-is` の失敗が起こりうる(PowerShell のクラスに関する一般の注意。この文書では出典を取得していない: 未確認)。(b) `class` のメソッドは、`return` で指定した値だけを出力する(確認: `about_Return` の注)が、関数は文の結果すべてを出力する(§14.4)。この 2 つの規則が混在すると、戻り値の混入を検査しにくくなる。値の表現を `[pscustomobject]` と関数に統一して、検査を単純にする。(c) `Set-StrictMode -Version Latest` のもとで、存在しないプロパティの参照は終了エラーになる(確認: `Set-StrictMode`。バージョン 2.0 以上)ので、項目名の綴りの誤りは、型が無くても実行時に検出される。
  - 値は**不変**として扱う(更新は §3.3〜3.5 の記録の関数が、新しい値を作って返す。元の値のプロパティを書き換えない)。配列のプロパティも、更新時は新しい配列を作る(`+=` で元の配列を伸ばさない。§14.2)。
  - `[pscustomobject]` どうしを `-eq` で比べると参照の比較になる(確認: `about_Comparison_Operators`。同じ内容の 2 つのオブジェクトでも、別のオブジェクトなら `False`。`IEquatable` を実装すれば比較できる)。ステート値の全項目の比較(§4.3)は、`Test-ConductorStateEqual`(各項目を順に比べ、`JobOrder` と `Jobs` は要素ごとに、文字列は序数で比べる)で行う。§15 のテストで、要素の同じ別々の配列を持つ 2 つの値が等しい、1 要素だけ違えば等しくない、を固定する。
- **固定の集合は文字列定数**: ジョブの状態・スキップ理由・分類・判断の種類・実行方式・判断の理由は、次の文字列(ステートの文字列と同じ綴り。大文字小文字を含めて固定)で表す。
  - 状態: `pending` / `success` / `warning` / `error` / `skipped`
  - スキップ理由: `byTransition` / `byEnd`
  - 分類: `success` / `warning` / `error`(分類なしは `$null`)
  - 判断の種類: `run` / `skip` / `reject`、判断の理由: `alreadyExecuted` / `skipTarget` / `afterEnd` / `outOfOrder`
  - 実行方式: `individual` / `pipeline`
  - 文字列の定数の比較は、常に `-ceq`(大文字小文字を区別)で行う。`-eq`・`switch` の既定は大文字小文字を区別しない(確認: `about_Comparison_Operators`、`about_Switch`)ので、使わない。生成関数は、渡された文字列が集合の要素かを `-ceq` で確かめ、外れたら内部の誤り(例外。70 になる)にする(`[ValidateSet()]` は使わない。大文字小文字の扱いが既定で区別されるかは未確認で、エラー文もロケールで変わる)。
- 文字列の比較は常に序数: `-ceq`、または `[string]::Equals($a, $b, [System.StringComparison]::Ordinal)`。`-eq`・`-like`・`-match`・`-contains`・`-in`・`switch`、`[string]::Compare`、`Sort-Object` の既定は大文字小文字を区別しない、または文化の規則に依存する(確認: `about_Comparison_Operators`。「文字列の比較は、明示的に大文字小文字を区別する演算子を使わない限り区別しない。文字列の比較は InvariantCulture を使い、Unicode のコードポイントの比較で、文化固有の照合順は使わない」)ので、使わない。区別する版が必要な演算子は `-ceq`・`-cne`・`-ccontains`・`-cin` を使う。並べ替えは `[System.Array]::Sort($copy, [System.StringComparer]::Ordinal)` を明示する。数値の文字列化は `.ToString([System.Globalization.CultureInfo]::InvariantCulture)`。
- 列挙の文字列表現(ステートの `"pending"` など)は、列挙型を使わず、上の文字列定数を直接ステートに書く(対応表が要らない)。

### 0.3 整数演算の方針

- PowerShell の算術は、結果が型に収まらないとき、**例外ではなく、より広い型へ自動で昇格する**: 確認(出典: `about_Arithmetic_Operators`「TYPE CONVERSION TO ACCOMMODATE RESULT」)。「演算の結果が型に対して大きすぎるとき、結果の型は広がる」(例: `512MB * 512MB` は `Int32` どうしの積で、結果は `Double`)。`[ulong]::MaxValue + 1` は `Double`、`9223372036854775807 + 2` は `Double`(整数の `long` と `int` の和が `long` を超えたとき)。`[decimal]` を含む演算は `Decimal` で、収まらなければ例外(`Value was either too large or too small for a Decimal.`)。`Double` に昇格した値を `[ulong]` に戻すと精度が失われる(例: `[ulong](9223372036854775807 + 2)` は 9223372036854775808。正しい値は ...809)。
- したがって、**`[uint64]` の `*` と `+` の結果をそのまま使わない**(折り返さない代わりに、黙って `[double]` になり、丸められる)。待ち時間の乗算・加算・比較は、すべて `[bigint]` で計算し、`[uint64]::MaxValue`(18446744073709551615)と比べて飽和させる(§7.6)。`[uint64]` どうしの積が `Double` になる、という記載は、`+` の例(`[ulong]::MaxValue + 1`)に対するもので、`*` の `[uint64]` の例は文書に無い(未確認。ただし規則「結果が型に収まらなければ広げる」は演算子に共通)。この規定は、その挙動に依存しない。
- `[bigint]` の演算は、すべてのオペランドを `[bigint]` に変換する(確認: 同ページ「Bigint arithmetic」。`[double]` の `1.9` は `1` に切り捨てて変換される。この文書の演算では `[double]` を混ぜない)。`[bigint]` と `[bigint]` の演算の結果は `[bigint]`。割り算は `/` でなく `[System.Numerics.BigInteger]::Divide($a, $b)` を使う(整数の除算であることを名前で示すため)。
- 整数の割り算の `/` は、PowerShell では商が整数でなければ `[double]` になり、`[int]` へのキャストは最近接の偶数への丸め(銀行丸め)になる(確認: 同ページ「DIVISION AND ROUNDING」。`[int](5/2)` は 2、`[int](7/2)` は 4)。この文書の数値計算では、`/` に `[int]` / `[long]` / `[uint64]` を使わない(使う箇所は `[bigint]::Divide` と、桁の和)。
- 数値リテラルの型は値で決まる: 接尾辞が無い 10 進整数は、`[int]` → `[long]` → `[decimal]` → `[double]` の順で、最初に収まる型(確認: `about_Numeric_Literals`)。`18446744073709551615` と書くと `[decimal]` になる(`[ulong]` の範囲を超えるリテラルは、接尾辞 `ul` を付けない限り `[ulong]` にならない。確認: 同ページ「`ul` は unsigned long」「`[ulong]::MaxValue` と `[decimal]::MaxValue` のあいだの値は `D` 接尾辞を付けないと `[double]` として解析される」)。**この文書の `[uint64]` の定数は、`18446744073709551615ul` と書く**(または `[uint64]::MaxValue`)。`n` 接尾辞(`10n`)は `[bigint]`(PowerShell 7.0 以降。同ページ)。`kb`・`mb` などの乗数の接尾辞は、待ち時間の定数に使わない。
- 整数の字句(定義の `interval`、JSON の数値)の変換は、`[int]::Parse` / `[long]::Parse` / `[int]'...'` のキャストに頼らず、ASCII の `0`〜`9` を 1 文字ずつ走査して自前で数える(§7.3)。文字列から数値へのキャストは、`0x`・`&h`・`#`・`0b` の接頭辞を受理する(確認: `about_Numeric_Literals`「Numeric type conversion」。`[int] '0xF'` は 15)ので、検査の前に使うと不正な字句が通る。

---

## 1. 用語とデータ

### 1.1 ジョブの状態

| 値(ステート上の文字列) | 意味 |
|---|---|
| `pending` | まだ起動していない。記録がないジョブも pending と同じに扱う |
| `success` | 成功(成功コードで終わった) |
| `warning` | 警告(警告コードで終わった) |
| `error` | エラー |
| `skipped` | スキップ。理由 `skip_reason` が必ず付く |

- 「済んだ(settled)」状態 = `success` / `warning` / `skipped`。
- 「結果のある」状態 = `success` / `warning` / `error`(終了コードを持てる)。
- スキップ理由: `byTransition`(遷移によるスキップ)、`byEnd`(`@end` によるスキップ)。

メモリ上の名前は PascalCase、ステートファイルのキー名は snake_case(対応は [06](06-state-file.md) §2)。

ジョブ状態(`JobState`)とステート値(`PipelineState`)の項目(`[pscustomobject]` のプロパティ):

| 値 | プロパティ(型) |
|---|---|
| `JobState` | `Id`(`[string]`)、`Status`(`[string]`。上の 5 値)、`ExitCode`(`$null` または `[int]`)、`SkipReason`(`$null` または `[string]`) |
| `PipelineState` | `DefinitionKey`・`DefinitionPath`・`PipelineName`(`[string]`。`DefinitionKey` は `DefinitionPath` と同じ値)、`RunId`(`$null` または `[string]`)、`JobOrder`(`[string[]]`)、`Jobs`(`[object[]]`。要素は `JobState`。`JobOrder` と同じ順・同じ数)、`ResumeJobId`(`$null` または `[string]`)、`EndReached`(`[bool]`)、`Completed`(`[bool]`) |

### 1.2 1 回の実行結果(`JobExecution`)

ジョブを 1 回起動して終わるまでの観測結果。

| プロパティ | 型 | 意味 |
|---|---|---|
| `JobId` | `[string]` | 定義のジョブID |
| `Started` | `[bool]` | 起動した(子プロセスの生成に成功した)か。「中断の合図が起動の前に立っていたため起動しなかった」ときだけ `$false`。ほかの起動失敗でも `Started` は `$true` のまま(`StartFailure` を見る) |
| `ExitCode` | `$null` または `[int]` | プロセスの終了コード。取れないとき(シグナル終了、起動失敗)は `$null` |
| `TerminatedBySignal` | `$null` または `[int]` | ジョブがシグナルで終了したときの番号(UNIX 系のみ。得かたは [07](07-process-and-signals.md)) |
| `StartFailure` | `[bool]` | 起動に失敗した。command を引数に分けられない、プロセス生成の失敗(`Process.Start` の例外)、終了状態を取得できない、のいずれか |
| `Interrupted` | `[bool]` | conductor がシグナルを受け取り、中断の合図が立っている(ジョブの終了を観測した直後に立っていた場合を含む) |
| `SignalNumber` | `$null` または `[int]` | `Interrupted` のとき、最初に受け取ったシグナルの番号 |

ジョブの起動手順(`Invoke-ConductorJob`。子プロセスの API と詳細は [07](07-process-and-signals.md)):

1. command を引数に分ける(規則は 07)。失敗なら `StartFailure = $true`(`Started = $true`)で終わる。
2. 中断の合図が立っていれば、`Started = $false, Interrupted = $true, SignalNumber = 番号` で終わる(起動しない)。
3. 子プロセスを生成する(`System.Diagnostics.Process`)。生成の失敗(`Win32Exception` など)なら `StartFailure = $true` で終わる。
4. 終了まで待つ。待機中に合図が立てば、子プロセスに同じシグナルを**1 回だけ**転送して、終了を待ち続ける。
5. 終了コードとシグナル終了を記録する(`Process.ExitCode`。シグナルで終わった場合の値の読み方は 07)。終了状態を取得できなければ `ExitCode`・`TerminatedBySignal` とも `$null` のまま(§2.3 の順 5 に当たり、`ConductorError` つきのエラー(終了コード 1)になる)。
6. 終了を観測した直後に合図が立っていれば `Interrupted = $true, SignalNumber = 番号`。

**`ExitCode` の範囲(Windows。未確認)**: `Process.ExitCode` は `[int]`(符号付き 32 ビット)。Windows のプロセスの終了コードは符号なし 32 ビットで、`0xC000013A` のような値が負の `[int]` として現れる、と理解しているが、この環境では確かめていない。`[int]` に収まる値はそのまま `ExitCode` に入れる。ステートの `exit_code` も 32 ビット符号付き整数で([06](06-state-file.md))、符号なしの値を足し引きして直す必要は無い。下位 8 ビットは同じなので、プロセスの終了コードに影響しない。

### 1.3 結果(`JobOutcome`)

| プロパティ | 意味 |
|---|---|
| `Classification` | `success` / `warning` / `error`、または**分類なし**(「起動しなかった中断」。値は `$null`) |
| `ReportedExitCode` | スケジューラへ返す終了コード(`[int]`)。後で 0〜255 に整える(§2.2) |
| `ConductorError` | 「成功コードに含まれない 0」など、conductor 自身が原因を標準エラーに表示するエラーのとき `$true` |

### 1.4 実行方式

`individual` は `conductor run <job-id>`、`pipeline` は `conductor pipeline <name>`。`@end` の扱いと完了の判定だけが違う(§3、§5)。

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

### 2.2 `ConvertTo-ConductorProcessExitCode`(`reported`, `isError`): 0〜255 への整形

プロセスの終了コードは 0〜255 に収める。

```text
if 0 <= reported <= 255:  return reported
low = reported & 0xFF              # 負数も含め、2 の補数表現の下位 8 ビット
if low == 0 and isError: return 1  # エラーを成功に見せない
return low
```

| reported | isError | 結果 |
|---|---|---|
| 3 | true | 3 |
| 256 | true | 1(下位 8 ビット 0 → 1) |
| 256 | false | 0 |
| 257 | true | 1 |
| 511 | true | 255 |
| -1 | true | 255 |
| -256 | true | 1 |

`isError` は `Classification` が `error`。分類なし(起動しなかった中断)と成功・警告は `$false`。

`Invoke-Conductor` が返す `[int]` は、この整形のあとの値(0〜255)にする。ランチャーの `exit` に範囲外の値を渡したときの OS ごとの切り詰めに頼らない(未確認)。

中断の終了コード `Get-ConductorSignalExitCode(n)` = `ConvertTo-ConductorProcessExitCode (Add-ConductorInt32Saturating 128 n) $true`(パイプラインのジョブのあいだの中断で使う)。

`Add-ConductorInt32Saturating`(`a`, `b`)(32 ビット符号付き整数の飽和加算): `[long]` に広げて足し、`[int]::MaxValue`(2147483647)を超えるなら `[int]::MaxValue`、`[int]::MinValue`(-2147483648)を下回るなら `[int]::MinValue` にする。2 つの `[int]` の和は `[long]` に収まるので、この加算は昇格に頼らない。オペランドは `[long]$a + [long]$b` と、両方の型を明示する(`[int]` 同士の `+` は、収まらないとき `[long]` ではなく `[double]` に昇格しうる。§0.3 の `512MB * 512MB` の例を参照)。結果を返す前に `[int]` へキャストするのは、範囲内であることを確かめた後だけ。

例: `Add-ConductorInt32Saturating 128 2147483647` = 2147483647。実在のシグナル番号(1〜64 程度)では起きない。

### 2.3 成功・警告・エラーの分類と返す値(`Get-ConductorOutcome`)

- **成功コードの集合** = 定義の `success_exit_codes` があればその値だけ(0 を書かなければ 0 でも成功にならない)、無ければ `[0]`。
- **警告コードの集合** = `warning_exit_codes`(0 を含まず、成功コードと重ならないことは定義の検証で保証済み)。
- 「`ExitCode` ∈ 集合」の判定は、集合の各要素と `[int]` の値を数値として比べる(`-eq` を `[int]` どうしで使う。`$set -contains $code` は、右辺が `$null` のとき挙動が紛らわしいので、`$null` でないことを先に確かめる。§14.3)。集合は空配列でありうる(`warning_exit_codes` が無いとき): `foreach` で走査し、`$set -contains` や `-in` の結果を、配列の単一化(§14.2)に頼って使わない。

判定は次の順に、最初に当てはまるものを採る(上ほど優先)。入力は 1 回の実行結果(§1.2)とジョブ定義。

| 順 | 条件 | Classification | ReportedExitCode | ConductorError |
|---|---|---|---|---|
| 1 | `Started = $false` かつ `Interrupted` | なし(`$null`) | `Add-ConductorInt32Saturating 128 (SignalNumber ?? 0)` | false |
| 1' | `Started = $false` かつ `Interrupted` でない(あり得ない組み合わせ) | error | 127 | false |
| 2 | `Interrupted`(起動済み) | error | `Add-ConductorInt32Saturating 128 (SignalNumber ?? 0)`。ジョブ自身の終了コードは見ない | false |
| 3 | `StartFailure` | error | 127 | false |
| 4 | `TerminatedBySignal` が有り(値 n) | error | `Add-ConductorInt32Saturating 128 n` | false |
| 5 | `ExitCode` が無い(起動して終わったのに終了コードがない) | error | 1 | **true** |
| 6 | `ExitCode` ∈ 成功コード | success | 0 | false |
| 7 | `ExitCode` ∈ 警告コード | warning | 0 | false |
| 8 | `ExitCode = 0`(成功コードに無い) | error | 1 | **true** |
| 9 | それ以外 | error | `ExitCode` そのまま(負数も、255 超もそのまま保持。整形は §2.2) | false |

`SignalNumber ?? 0` は「`SignalNumber` が `$null` なら 0」。PowerShell 7 の `??` 演算子は 7.0 以降で使えるが、この文書では `if ($null -eq $x) { 0 } else { $x }` と書く(`??` の優先順位と、`$null` 以外の偽の値(0、空文字列)の扱いを推測しない)。

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

- `ConductorError = $true`: `conductor: error: job '<id>': exit code 0 is not listed in success_exit_codes`(日本語ロケールでは日本語。文言は [08-output-and-i18n.md](08-output-and-i18n.md))。`ConductorError` が真なら、順 5(終了コードが無い)の場合も同じ文言を出す(実装は `ConductorError` が真なら、理由を区別せず同じ行を出す)。
- 起動失敗: `conductor: error: job '<id>': cannot start command '<program>'`。

---

## 3. 遷移と次の動き(`Get-ConductorClassifiedStep` / `Get-ConductorNextStep`)

ジョブ実行後に、分類と次の動き(`NextStep`)を決める。

### 3.1 `NextStep` のプロパティ

| プロパティ | 意味 |
|---|---|
| `NextJobId` | 次に実行するジョブ。完了・停止のとき `$null`(`pipeline` コマンドは、この値を使わない。§5.3) |
| `SkippedJobIds` | この時点でスキップとして記録するジョブ(`[string[]]`。空は空配列) |
| `SkipReasonKind` | その理由(`byTransition` / `byEnd` / `$null`) |
| `EndReached` | `@end` に到達した |
| `Completed` | 完了した |
| `ResumeJobId` | エラー等で停止したときの再開位置(そのジョブ)。`$null` もある |

### 3.2 判定表

入力: 分類、そのジョブの該当する遷移(`on = success` は成功のとき、`on = warning` は警告のときに参照する。`error` の遷移は書けない)、実行方式、そのジョブの位置(index。定義の記述順、0 始まり)、各ジョブの現在の状態。

「未完了のジョブ」= 指定した区間のジョブのうち、状態が success / warning / skipped **でない**もの(pending・error・記録なし)。区間は、`@end` なら「index より後ろ全部」、遷移先 `T` なら「index と T のあいだ(両端を含まない)」。

| 分類 | 遷移 | 実行方式 | NextJobId | SkippedJobIds / kind | EndReached | Completed | ResumeJobId |
|---|---|---|---|---|---|---|---|
| error | (見ない) | 両方 | null | 空 / null | false | false | **このジョブ** |
| 分類なし(起動しなかった中断) | (見ない) | 両方 | null | 空 / null | false | false | **このジョブ** |
| success / warning | `@end` | pipeline | null | 後ろの未完了ジョブ全部 / `byEnd`(空なら kind は null) | true | **true** | null |
| success / warning | `@end` | individual | null | **空** / kind = `byEnd`(後ろに未完了が 1 つでもあるとき。無ければ kind は null) | true | 後ろに未完了が無ければ true、あれば false | null |
| success / warning | ジョブ `T`(index より後ろ) | 両方 | `T` | index と T の間の未完了ジョブ / `byTransition`(空なら kind は null) | false | false | null |
| success / warning | 遷移なし(または遷移先が後ろのジョブでない。検証済み定義では起きない) | 両方 | 最後のジョブなら null、そうでなければ次のジョブ | 空 / null | false | 最後のジョブなら true、そうでなければ false | null |
| 定義にないジョブ | - | - | error と同じ(停止) | | | | |

要点:

- 成功・警告のあと再開位置は無し。エラー・分類なしのあとは必ずそのジョブ。
- individual の `@end` では、後ろのジョブを今は記録せず(`SkippedJobIds` は空)、`EndReached = $true` だけを記録する。後ろのジョブは、スケジューラが 1 つずつ `run` を呼んだときに `skip(afterEnd)` が返り、呼ばれたジョブだけ `skipped(byEnd)` と記録される(§4.2、§5.4)。
- 遷移先 `T` の遷移では、`T` へ飛ぶ。`T` が index+1 なら間は空で、スキップは無い。
- 自動のリトライ(ジョブ失敗の再実行)は、ジョブに `retry` を書かない限りしない(§7)。
- 表の「遷移」の値の比較(`@end` かジョブIDか)は、遷移の値の種類(03 章の型。`@end` を表す値か、ジョブIDの文字列か)で行い、ジョブIDが `@end` という文字列かどうかを `-eq` で調べない(ジョブIDは `@` で始まらないことが検証済み。03 章)。

### 3.3 `Set-ConductorOutcomeRecord`(結果の記録。メモリ上)

入力: 現在のステート値 `S`、ジョブID、実行結果、結果(`JobOutcome`)、`NextStep`。`S` を複製して更新し、返す。

1. 対象ジョブ(`S` のジョブ状態一覧から id で探す。見つからなければこの手順 1 を飛ばす):
   - `Classification` があれば `Status` を `success` / `warning` / `error` にし、`SkipReason` を消す。分類なし(起動前の中断)なら `Status` は**変えない**。
   - `ExitCode` = 実行結果の `ExitCode`(無ければ、以前の値も消す)。そのあと `Status` が success / warning / error のいずれでもなければ `ExitCode` を消す。
2. `SkippedJobIds` の各ジョブ(対象ジョブ自身は除く)を `Status = skipped`、`ExitCode` 無し、`SkipReason = SkipReasonKind`(`$null` なら `byTransition`)にする。
3. `ResumeJobId = NextStep.ResumeJobId`、`Completed = NextStep.Completed`。
4. `EndReached = S.EndReached -or NextStep.EndReached`(一度 true になったら戻さない)。

### 3.4 `Set-ConductorSkipRecord`(スキップの記録。メモリ上)

入力: ステート値 `S`、ジョブID、判断の理由、定義。

- 理由が `alreadyExecuted` / `outOfOrder` → 何も変えない(複製を返す)。
- `skipTarget` → そのジョブを `skipped(byTransition)`、`afterEnd` → `skipped(byEnd)` にし(`ExitCode` 無し)、`afterEnd` なら `EndReached = $true`。ジョブが `S` に無ければ、更新せず複製を返す。
- そのあと、`S` に `error` のジョブが 1 つでもあれば、`ResumeJobId` と `Completed` は**変えない**(エラーの再開位置を守る)。無ければ:
  - `firstPending` = 定義の記述順で最初の `pending`(記録なしを含む)のジョブ(無ければ `$null`)
  - `Completed` = 「いまスキップしたジョブが定義の最後のジョブ、かつ `firstPending` が `$null`」
  - `ResumeJobId = firstPending`

### 3.5 `Set-ConductorInterruptRecord`(中断の記録。メモリ上)

再開位置(§9)の値で更新する: `ResumeJobId = ResumePoint.ResumeJobId`、`Completed = ResumePoint.Completed`、`EndReached = S.EndReached -or ResumePoint.EndReached`。ジョブの状態は変えない(エラーとして記録しない)。

---

## 4. 起動前の判断(`Get-ConductorDecision`)

### 4.1 入力と出力

入力: ジョブID、定義、各ジョブの状態の一覧、`ResumeJobId`(ステートの同名の値)、`EndReached`。ステートが空(ファイルなし・長さ 0・完了のまま残ったもの)のときは、状態一覧は空配列、`ResumeJobId = $null`、`EndReached = $false` を渡す。

出力(`Decision`): `Kind` = `run`(実行する)/ `skip`(実行せず 0 で終わる。`Reason` は `skipTarget` / `afterEnd` / `alreadyExecuted`)/ `reject`(実行せず、ステートも変えず、66。`Reason` は `outOfOrder`)。`run` のとき `Reason` は `$null`。

### 4.2 判定表(上から最初に当てはまるものを採る)

状態 `st` = そのジョブの記録(無ければ `pending`)、`reason` = そのスキップ理由。

| 順 | 条件 | 結果 |
|---|---|---|
| 0 | ジョブIDが定義にない(`run` では、事前に 68 で止まるので到達しない) | `reject(outOfOrder)` |
| 1 | `st = skipped` かつ `reason = byEnd` | `skip(afterEnd)` |
| 1' | `st = skipped` かつ `reason` が `byTransition` または `$null` | `skip(skipTarget)` |
| 2 | `EndReached` かつ `st = pending` | `skip(afterEnd)` |
| 3 | `st` が `success` または `warning` | `skip(alreadyExecuted)` |
| 4 | `st = error`、または `ResumeJobId` がこのジョブID | `run` |
| 5 | 全ジョブが `pending`(または記録なし)で、このジョブが記述順で最初 | `run` |
| 6 | `st = pending` で、記述順で前のジョブがすべて `success` / `warning` / `skipped` | `run` |
| 7 | 上のどれでもない | `reject(outOfOrder)` |

例(4 ジョブ j0〜j3):

| 呼ぶジョブ | 状態 | resume | endReached | 結果 |
|---|---|---|---|---|
| j1 | j0=success, j1=skipped(byTransition) | - | false | skip(skipTarget) |
| j2 | j2=skipped(byEnd) | - | true | skip(afterEnd) |
| j3 | j0=success(j3 記録なし) | - | true | skip(afterEnd) |
| j0 | j0=success | - | false | skip(alreadyExecuted) |
| j0 | j0=warning | - | false | skip(alreadyExecuted) |
| j1 | j0=success, j1=error | j1 | false | run |
| j1 | j1=error | - | false | run |
| j2 | j0=success, j1=skipped | j2 | false | run |
| j0 | (空) | - | false | run |
| j0 | 全部 pending | - | false | run |
| j2 | j0=success, j1=warning | - | false | run |
| j3 | j0=success, j1=skipped, j2=skipped | - | false | run |
| j1 | (空) | - | false | reject |
| j2 | j0=success | - | false | reject |
| j2 | j0=success, j1=error | - | false | reject |

補足: 規則 1' は、理由が書かれていない `skipped`(ステートファイル上は読み込みで弾かれるが、関数としては入りうる)を `skipTarget` とする。規則 2 は `pending`(記録なしを含む)のときだけ。`error` のジョブは規則 4 で、`EndReached` でも `run` になる。規則 6 は「記述順で前のすべて」を見るので、後ろのジョブが済んでいても影響しない。

実装の注意(PowerShell): この表は `if` / `elseif` の連鎖で書く。`switch` は使わない(§14.6: 既定で文字列に変換し、大文字小文字を区別せず、一致した全ての節を実行し、`$_` を共有する)。「ジョブの状態を ID で引く」は、`Where-Object` のパイプラインでなく `foreach` で走査し、見つからなければ `pending` を使う(`Where-Object` の結果は 0 件で `$null`、1 件で単独の値、2 件以上で配列になる。§14.2)。

### 4.3 判断の結果とステート・終了コード(呼び出し側)

| 結果 | ステートの変更 | 標準エラー | 終了コード |
|---|---|---|---|
| `reject` | しない(初期ステートも書かない) | `conductor: error: job '<id>': cannot run this job now: earlier jobs have not finished (out of order)`(日本語: `このジョブは、いま実行できません: 前のジョブが終わっていません(順序外)`) | 66 |
| `skip` | `Set-ConductorSkipRecord` の結果が元と**等しくなければ**(§0.2 の `Test-ConductorStateEqual`)、耐久性ありで書く。`alreadyExecuted` は常に等しい(書かない) | `-v` のとき `[conductor] job=<id> skipped (<理由>)`。理由文: skipTarget = `skipped by a transition`、afterEnd = `skipped after @end`、alreadyExecuted = `already executed` | 0(pipeline では続行) |
| `run` | ステートが空なら先に初期ステートを、耐久性を省いて書く(§5) | - | ジョブの結果による |

「等しい」はステート値の全項目の比較。「耐久性あり(Durable)」「省く(Fast)」の定義は [06-state-file.md](06-state-file.md) の書き込み。

---

## 5. 実行の流れ

### 5.1 共通の前処理 `Invoke-ConductorPrepare`(`run`・`pipeline`)

順序は固定。最初に誤りがあった時点で、その終了コードで終わる(以降は行わない)。

| 順 | 処理 | 誤りのときの終了コード |
|---|---|---|
| 1 | 中断の合図(「立った/立っていない」と最初のシグナル番号を保持する共有の旗。§16、[07](07-process-and-signals.md))を用意し、シグナルのハンドラを登録(定義の検査より前)。登録の方式は 07 章。登録に使うオブジェクトは、プロセスの終了まで参照を保持する | 準備の失敗(登録が例外を出した): 70(`conductor: error: failed to set up signal handling`) |
| 2 | 定義ファイルの読み込みと検証。`pipeline` は、引数の名前と定義の `pipeline.name` を照合 | 64 |
| 3 | 全ジョブの `command` の検査(引用符の閉じ忘れ・空) | 64 |
| 4 | `run` のとき、ジョブIDが定義にあるか | 68(`unknown job`) |
| 5 | run ID の決定(§6) | 68 |
| 6 | ステートの読み込み([06-state-file.md](06-state-file.md) §6)。完了のまま残ったものは「空」 | 65 |
| 7 | 空でなければ、定義との照合(06 §7) | 65 |
| 8 | 実効の `remove_on_complete` = `--remove-on-complete` があれば true、なければ定義の `pipeline.remove_on_complete`(省略時 false)。環境変数は見ない | - |

- 定義ファイルは `--config <パス>`、なければ実行ディレクトリの `conductor.psd1`([03](03-definition-format.md)、[04](04-cli.md))。
- シグナルを受け取っても、1〜7 は最後まで進む(ハンドラは旗を立てるだけで、処理を中断しない)。64・65・66・スキップ(0)の終了コードが、中断の終了コードに優先する。
- `pipeline` で引数の名前が定義の名前と違う場合は 64(順 2)。引数の名前が無いのは使い方の誤り 68(コマンドライン解析。[04-cli.md](04-cli.md))。

### 5.2 `conductor run <job-id>` の擬似アルゴリズム

```text
session = Prepare(...)                      # 5.1。誤りなら終了
(states, resume, endReached) = session.State の Jobs / ResumeJobId / EndReached
                               (State が null(空)なら [], null, false)
d = DecideBeforeRun(jobId, def, states, resume, endReached)

match d:
  reject:  表示、return 66
  skip:
    base = session.State か InitialState(...)(空なら)
    rec  = ApplySkip(base, jobId, d.Reason, def)
    if !StateEqual(rec, base):
        Write(rec, Durable)                  # 失敗: 65、文脈=ジョブ起動前
        if rec.Completed and removeOnComplete:
            ResetState()                     # 失敗: 65
    -v なら skipped 表示
    return 0
  run:
    base = session.State か、(空なら) InitialState を Fast で書いたもの   # 書けなければ 65、ジョブは起動しない
    exec = RunJobWithRetry(job)              # §7。ステートは書かない
    (outcome, next) = ClassifyAndDecide(job, exec, base.Jobs, def, individual)
    表示(§5.5)
    rec = ApplyOutcome(base, jobId, exec, outcome, next)
    Write(rec, Durable)                      # 失敗: 65、文脈=exec.Started なら「ジョブは実行されました」、そうでなければ「起動していません」
    if next.Completed and removeOnComplete:
        ResetState()                         # 失敗: 65
    return ConvertToProcessExitCode(outcome.ReportedExitCode, outcome.Classification == "error")
```

注意:

- individual では、遷移でスキップが決まったジョブ(`byTransition`)は、結果の記録で記録済みなので、後で `run` されると `skip(skipTarget)`。
- 最後のジョブの `run` 完了、または `@end` 後の最後のジョブのスキップ記録で `Completed = $true`。
- 初期ステートは `run` と決まった後にだけ書く(reject・skip の前には書かない)。ただし skip のときは、ステートが空なら `InitialState`(06 §2.4)を土台に記録が書かれる。
- `ClassifyAndDecide(job, exec, states, def, mode)` は、分類(§2.3)で結果を作り、その分類を使って `NextStep`(§3.2)で次の動きを決めて、組で返す。PowerShell では、組を `[pscustomobject]@{ Outcome = ...; Next = ... }` で返す(複数の値を `return $a, $b` で返すと配列に展開され、受け取り側で取り違えやすいので使わない。§14.2)。

### 5.3 `conductor pipeline <name>` の擬似アルゴリズム

`pipeline` は、「次のジョブ」を `NextStep.NextJobId` で決めない。定義の記述順に全ジョブを 1 つずつ見て、`DecideBeforeRun` で決める(遷移先・スキップは、ステートが運ぶ)。

```text
session = Prepare(...)                       # 引数の名前が無ければ 68
current = session.State                      # 空なら null
for job in def.Jobs (記述順):
  d = DecideBeforeRun(job.Id, def, current の状態一覧 (null なら []), current?.ResumeJobId, current?.EndReached ?? false)
  match d:
    reject: 表示、return 66
    skip:
      base = current か InitialState(...)
      rec  = ApplySkip(base, job.Id, d.Reason, def)
      if !StateEqual(rec, base): Write(rec, Durable)    # 失敗: 65
      -v なら skipped 表示
      current = rec
      if rec.Completed: return Finish(...)         # 下記
    run:
      base = current、null なら InitialState を Fast で書いたもの   # 失敗: 65
      exec = RunJobWithRetry(job)
      (outcome, next) = ClassifyAndDecide(job, exec, base.Jobs, def, pipeline)
      表示
      rec = ApplyOutcome(base, job.Id, exec, outcome, next)
      Write(rec, Durable)                          # 失敗: 65
      if outcome は error、または分類なし:
          return ConvertToProcessExitCode(outcome.ReportedExitCode, outcome.Classification == "error")   # エラーで停止
      if next.Completed: return Finish(...)
      if 中断の合図が立っている(番号 n):            # ジョブのあいだの中断
          point = ResumePoint(def, rec.Jobs)
          Write(ApplyInterrupt(rec, point), Durable)        # 失敗: 65
          -v なら「interrupted by signal n」
          return SignalExitCode(n)                       # ConvertToProcessExitCode(AddInt32Saturating(128, n), true)
      current = rec
# ループを抜けた(完了していない。起こらない想定)
表示 "cannot determine whether the pipeline is complete"; return 65   # ステートは初期化しない

Finish: if !removeOnComplete: return 0
        ResetState() が成功 → 0、失敗 → 65
```

- 完了のステートは、`Finish` の前に必ず Durable で書き込み済み。`remove_on_complete` が真のときだけ削除する(06 §9)。
- 「完了を判定できない」は、全ジョブが `alreadyExecuted` などで通過し、どこでも `Completed` が真にならなかったとき。標準エラー `conductor: error: <ステートファイルのパス>: cannot determine whether the pipeline is complete`(日本語: `パイプラインが完了したかを判定できません`)、終了コード 65。
- ジョブ実行後、`next.Completed` の判定は、中断の合図の判定より**先**。完了していれば中断の記録はせず、`Finish` へ進む。
- エラーのジョブで止まったときの再実行: 同じコマンドを再度呼ぶと、`success`/`warning` のジョブは `skip(alreadyExecuted)`(`-v` に `already executed`)、`error` のジョブから `run`。
- `skip` で `rec.Completed` が真になるのは §3.4 の条件(スキップしたジョブが最後のジョブで、pending が無い)のとき。
- PowerShell の実装: ジョブの走査は `foreach ($job in $def.Jobs)`(`ForEach-Object` ではない。`return` や `break` の意味がパイプラインの中では変わる。§14.7)。中断の合図は、ジョブの実行後に 1 回だけ読み(`$n = 合図の番号`)、その値を使う(§16)。

#### 状態遷移の例(`pipeline`)

定義 `a → b → c`(遷移なし)。`b` が exit 3 でエラー。

| 時点 | job_states | resume_job_id | end_reached | completed | 終了コード |
|---|---|---|---|---|---|
| 初期(Fast で書く) | a=pending, b=pending, c=pending | なし | false | false | - |
| a 成功後 | a=success(0), b=pending, c=pending | なし | false | false | - |
| b エラー後 | a=success(0), b=error(3), c=pending | b | false | false | 3 |
| 再実行: a は skip(alreadyExecuted)、b 成功 | a=success, b=success(0), c=pending | なし | false | false | - |
| c 成功後 | 全部 success | なし | false | true | 0 |

定義 `a --success--> @end` のとき(pipeline): a 成功で b, c は `skipped(byEnd)`、`end_reached = true`、`completed = true`、0 で終わる([06-state-file.md](06-state-file.md) §4.4 の例 2 のステート)。

### 5.4 `@end` 後の `run`(individual)の流れ

`run a`(遷移 `@end`)→ `a=success`、`end_reached = true`、`completed = false`(後ろに未完了がある場合)。続けて `run b` → 規則 2 で `skip(afterEnd)` → `b=skipped(byEnd)`、`resume_job_id` = 最初の pending(`c`)。`run c` → `skip(afterEnd)` → `c=skipped(byEnd)`、最後のジョブで pending が無いので `completed = true`(`remove_on_complete` なら削除)。`@end` の後ろに未完了が無ければ、`run a` の時点で `completed = true`。

### 5.5 表示(`-v` と常時)

- 常に標準エラー: エラー行(`conductor: error: ...`)、起動失敗、成功コードに無い 0、リトライの知らせ(§7.9)、`reset --all` の警告。
- `-v` のときだけ標準エラー(先頭 `[conductor] `): `job=<id> result=<success|warning|error|interrupted> (...)`、`transition: <success|warning> -> <target>[ (skipped: <id>, ...)]`、`job=<id> skipped (<理由>)`、`interrupted by signal <n>`。括弧内の詳細の厳密な文言は [08-output-and-i18n.md](08-output-and-i18n.md) に従う。
- ジョブの結果の行と遷移の行は、**最終の結果について 1 回**だけ(リトライの途中の結果は出さない)。
- 言語: 環境変数 `LC_ALL`、`LC_MESSAGES`、`LANG` の順で、最初の空でない値が `ja` で始まれば日本語、それ以外は英語(どれも空・未設定なら英語)。トークン(`success` `warning` `error` `skipped` `pending` `job=` `result=`)は常に英語。「`ja` で始まる」の判定は `$value.StartsWith('ja', [System.StringComparison]::Ordinal)`(引数が 1 つの `StartsWith` は文化依存になりうる。`-like 'ja*'` は大文字小文字を区別しない(確認: `about_Comparison_Operators`)ので使わない)。表示は、画面への出力専用の関数だけが `[Console]::Error` に書く(製品コードで `Write-Host` / `Write-Output` / `Write-Error` を使わない。[02](02-architecture.md)、[08](08-output-and-i18n.md))。

---

## 6. run ID

run ID は、同じ定義を別々の実行単位で並行して使うときに、ステートファイルを分ける識別子。

### 6.1 値の規則(`Test-ConductorRunId` / `Get-ConductorRunIdError`)

次の順に検査し、最初に当てはまる誤りを返す(誤りは使い方の誤り 68。誤りの文に値は含めない)。

| 順 | 誤り | 条件 |
|---|---|---|
| 1 | `empty` | 長さ 0 |
| 2 | `tooLong` | UTF-8 で符号化したバイト数が 64 を超える |
| 3 | `invalidChar` | `A-Za-z0-9._-` 以外の文字がある |
| 4 | `leadingDot` | 先頭が `.`(`..` を含む) |

PowerShell での決定と落とし穴:

- **`-match`・`switch -Regex`・`[regex]` を使わず、手書きの走査にする。** 理由:
  - `-match` は大文字小文字を区別しない既定で、`$Matches` を書き換える(確認: `about_Comparison_Operators`)。`-cmatch` にしても、`\d`・`\w` は、既定で Unicode の数字・文字(全角の `１` など)に一致しうる(.NET の正規表現の一般の性質。この文書では取得して確認していない: 未確認)。`$` が文字列の末尾の `\n` の手前にも一致する(同じく一般に知られた .NET の挙動。未確認)ので、`^[A-Za-z0-9._-]+$` は `"a\n"` を受理しうる。
  - `[regex]` を使う場合は、`[regex]::new('\A[A-Za-z0-9._-]+\z', [System.Text.RegularExpressions.RegexOptions]::CultureInvariant)` のように、`\A`・`\z`・文字クラスの明示の範囲を使い、タイムアウトを渡す(確認: `about_Comparison_Operators` の「Regular expression safety」に、`[regex]::new($pattern, $regexOptions, $matchTimeout)` の例がある)ことで防げるが、この仕様は、確認の対象を増やさないため、手書きの走査を正本にする。`[regex]` による実装は、走査と同じ結果を返す限り、許す(§15 のテストで両方を同じ入力で比べてよい)。
- 順 3 の判定は、1 文字ずつ、**コードユニットの整数**に直して範囲比較する。`[char]` どうしの `-ge`・`-le`(例: `$c -ge 'a'`)は、文字列として、文化に依存しない(InvariantCulture)が大文字小文字を区別しない比較になりうる(確認: `about_Comparison_Operators`。`'macOS' -ilt 'MacOS'` が False、`-clt` が True の例。`-ge` の既定は大文字小文字を区別しない比較)ので、`'A' -ge 'a'` の判定を取り違える。`[int]` へのキャスト(`[int]$c` は `[char]` の UTF-16 コードユニットの値を返す。一般に知られた挙動。§17 の試験 T-6 で固定する)を使う:

```text
IsAllowed(c):  # c は [int](文字のコードユニット)
   (c >= 97 && c <= 122)  or  (c >= 65 && c <= 90)  or  (c >= 48 && c <= 57)  or  c == 46 ('.')  or  c == 95 ('_')  or  c == 45 ('-')
```

  - `[char]::IsLetterOrDigit`・`[char]::IsDigit` は Unicode の文字・数字に真を返すので使わない。
  - 文字列の走査は `for ($i = 0; $i -lt $value.Length; $i++)` と `$value[$i]`(`[string]` のインデックスは `[char]` を返す)。`foreach ($c in $value.ToCharArray())` でもよい。**`foreach ($c in $value)` と書かない**(`[string]` はパイプラインで 1 つの値で、列挙されない。ただし `foreach` の対象としての `[string]` の扱いは未確認なので、`ToCharArray()` か添字の走査に固定する)。
- 順 2 のバイト数は、`[string]` を UTF-16 のコードユニット(`[char]`)の列として走査して数える(正本):

| 走査する `char`(コードユニットの値 c) | 加えるバイト数 |
|---|---|
| U+0000〜U+007F | 1 |
| U+0080〜U+07FF | 2 |
| 高サロゲート(U+D800〜U+DBFF)の直後に低サロゲート(U+DC00〜U+DFFF)が続く対 | 4(2 つの `char` で 1 回。対で数える。`[char]::IsHighSurrogate($c)`・`[char]::IsLowSurrogate($next)` を使ってよい) |
| 対にならない孤立サロゲート(U+D800〜U+DFFF の 1 つ) | 3(U+FFFD に置き換わるとみなす) |
| それ以外(U+0800〜U+FFFF のサロゲート以外) | 3 |

  - `[System.Text.Encoding]::UTF8.GetByteCount($value)` は、同じ数を返すはずだが、実装を 1 つに固定するため、走査を正本にする。公式ドキュメントで確認できたこと(出典: `UTF8Encoding(Boolean, Boolean)` の例): 既定の `UTF8Encoding`(例外を投げない設定)は、不正なサロゲート(高サロゲートが 2 つ続く `U+D801 U+D802`)を `EF BF BD EF BF BD`(U+FFFD の 3 バイトを 2 つ)に符号化する。`[System.Text.Encoding]::UTF8` の `GetByteCount` が `GetBytes` と同じ数を返すことは、ドキュメントを取得して確認していない(未確認)。`GetByteCount` は、テストで走査と同じ結果になることの照合に使ってよい。
  - 走査の途中で合計が 64 を超えたら、その時点で `tooLong`(全体を数え切る必要はない。桁あふれもしない)。
- `[string]` の `.Length` は UTF-16 のコードユニット数で、コードポイントの数でも書記素の数でもない。`'あ'` は 1 つの `char` で 3 バイトなので、`('あ' * 30)` は `.Length` が 30 だが 90 バイトで `tooLong` になる(`'あ' * 30` の `*` は、文字列の複製。確認: `about_Arithmetic_Operators`)。`.Length` で順 2 を判定すると、順 2 と順 3 の判定が入れ替わる(`invalidChar` になる)。絵文字(U+1F600)は 2 つの `char`(サロゲートの対)で 4 バイト。許す文字は ASCII だけなので、検査を通る値では文字数とバイト数は同じ。
- 先頭の `.` の判定は `$value[0] -ceq [char]'.'`、または `[int]$value[0] -eq 46`(`$value.StartsWith('.')` は文化依存の比較になりうるので使わない)。空は順 1 で除かれているので、`$value[0]` は常に有効(`Set-StrictMode` の最新の版では範囲外の添字が例外になる。確認: `Set-StrictMode` の 3.0 以降)。
- Unix で、引数・環境変数に不正な UTF-8 のバイト列が入っていたとき、PowerShell がそれをどの文字列にするかは、公式ドキュメントで確認していない(未確認)。どの場合でも、許さない文字を含む(または長すぎる)ので、利用者に見える結果は 68 で変わらない(誤りの文に値と種類は含めないため)。

有効: `1`, `r1`, `12345`, `build-2026.11_a`, `a.b`, 64 文字ちょうど。無効: `""`(ただし §6.2 の規則で空は「なし」)、65 文字、`a/b`、`a b`、`.x`、`..`、`日本語`(30 文字(90 バイト)なら `tooLong`、21 文字(63 バイト)なら `invalidChar`)、`a` + LF(改行は許す文字に無いので `invalidChar`)、全角の `１`(U+FF11)、`A` と `a` の混在(大文字小文字はどちらも有効。大文字小文字を区別しない比較で `A-Z` と `a-z` を取り違えない)。

境界の例(`tooLong` との関係):

| 値 | バイト数 | 結果 |
|---|---|---|
| `a` × 64 | 64 | 有効 |
| `a` × 65 | 65 | `tooLong` |
| `あ` × 21 | 63 | `invalidChar` |
| `あ` × 22 | 66 | `tooLong` |
| `a` × 62 + `é` | 62 + 2 = 64 | `invalidChar`(バイト数は 64 以下) |
| `a` × 63 + `é` | 65 | `tooLong` |
| U+D800(孤立) 1 つ | 3 | `invalidChar` |

### 6.2 決定(`Resolve-ConductorRunId`、`Prepare` の 5)

```text
envName = 定義の pipeline.run_id_env があればそれ、なければ "CONDUCTOR_RUN_ID"
if --run-id が指定された(値は空でもよい):
    value = そのオプションの値(subject = "--run-id")      # 環境変数は見ない
else:
    value = lookupEnv(envName)(本番は [System.Environment]::GetEnvironmentVariable。未定義なら null)(subject = "run ID (environment variable)")
if value が null または "": run ID なし
else: 検査が失敗なら 68(`invalid value: '<subject>'`)、成功なら run ID
```

- `--run-id ""` は、環境変数があっても「run ID なし」に固定する。
- `run_id_env` の形は定義の検証で保証される(`A-Za-z_` で始まり `A-Za-z0-9_` が続く 1 文字以上)。
- 環境変数名は定義で決まるので、定義の検査(64)の**後**、ステートの読み込み(65)の**前**に run ID を決める。
- `watch`・`reset` は定義を検証しない: `--run-id` があれば定義を読まず(環境変数名は既定)、なければ定義の `pipeline.run_id_env` だけをゆるく読む。読めない・キーが無い・文字列でない・名前の形が違う、のどれでも「なし」= 既定の名前を使う。ほかの誤りは無視する。異常終了してはならない(定義の読み取りが投げた例外は、この場面では握りつぶして既定の名前にする。`try { ... } catch { ... }` の `catch` で `[System.Exception]` を受ける。ただし `PipelineStoppedException`(Ctrl+C。§16)は握りつぶさずに再送出する)。
- 環境変数の値が規則に合わなければ、`run`・`pipeline`・`watch`・`reset`(`--all` を除く)で 68。`reset --all` は環境変数を見ない。
- 環境変数は **`[System.Environment]::GetEnvironmentVariable($name)`** で読む(未定義は `$null`)。`$env:NAME` は使わない(未定義のとき `$null` を返すが、名前が変数のとき `$env:$name` と書けない・`Env:` ドライブ経由で、空の値と未定義の区別・名前の大文字小文字の扱いが、`Get-Item Env:` のプロバイダーに依存する。プロバイダーの挙動は公式ドキュメントを取得して確認していない: 未確認)。`GetEnvironmentVariable` が未定義で `null` を返すことは、このメソッドの契約として知られているが、この文書では取得して確認していない(未確認)。変数名の大文字小文字は、Unix では区別され、Windows では区別されない(Windows の環境変数の一般の性質。.NET の API としてどうなるかは未確認)。環境の読み取りは、差し替え可能な引数(`[scriptblock] $LookupEnv`)を通し、`Resolve-ConductorRunId` に渡す。`Private/Flow/` の関数からは読まない。

### 6.3 run ID とステートの関係

- run ID あり: ステートは `<定義ファイル名>.<run-id>.state`。中身に `run_id` = run ID を書く。run ID なし: `<定義ファイル名>.state`、`run_id` の項目無し。
- 読み込み時、中身の `run_id`(あるか、値)がファイル名側の run ID と一致しなければ壊れたステート(65)。詳細は 06 §6。
- run ID は同じ定義でも run ごとに独立したステート・再開位置を持つ。同じ run ID で呼び直すと、止まったジョブから再開する。

---

## 7. リトライ(`retry`、v0.8.0)

### 7.1 設定(検証済みの値。`RetryPolicy`)

ジョブ定義の `retry`([03](03-definition-format.md))。

| キー | 型・範囲 | 意味 |
|---|---|---|
| `exit_codes` | 整数の一覧。空でない。各値は 1〜255。昇順・重複なしに正規化済み | リトライの対象にするジョブ自身の終了コード。このジョブの成功コード・警告コードと重ならない |
| `max` | 整数 1〜100 | リトライの最大回数(最初の実行を除く)。実行は合計で最大 `max + 1` 回 |
| `interval` | 文字列 `<整数><s\|m\|h>`。結果が 1 秒〜24 時間(86400 秒) | 1 回目のリトライの前の待ち時間。数字は半角の数字だけで、整数は 1 以上。単位は小文字 1 文字(`s` 秒、`m` 分 = 60 秒、`h` 時間 = 3600 秒)。例 `"30s"`、`"2m"`、`"1h"` |
| `multiplier` | 整数 2〜10 | 待ち時間にかける倍率。`max_interval` と同時に指定 |
| `max_interval` | `interval` と同じ書式。`interval` 以上 | 待ち時間の上限。`multiplier` と同時に指定 |

- `multiplier` と `max_interval` は両方書くか両方書かない。検証の詳細は [03-definition-format.md](03-definition-format.md)。
- `retry` を書かないジョブは、1 回だけ実行する。

### 7.2 数値の型と骨格

§0.2・§0.3 の方針を、リトライに当てはめる。待ち時間は秒の `[uint64]`、倍率は `[int]`。`[double]`・`[decimal]`・`[timespan]`・`[long]` は、待ち時間の計算に使わない(丸めの余地を作らない)。計算は `[bigint]`(§7.6)。

`RetryPolicy`(検証済み。03 章の値を、判断に必要な形に取り出したもの。`[pscustomobject]`):

| プロパティ | 型 | 範囲 |
|---|---|---|
| `ExitCodes` | `[int[]]` | 昇順・重複なし・各 1〜255・空でない |
| `Max` | `[int]` | 1〜100 |
| `IntervalSecs` | `[uint64]` | 1〜86400 |
| `Backoff` | `$null` または `[pscustomobject]`(`Multiplier`: `[int]` 2〜10、`MaxIntervalSecs`: `[uint64]` で `IntervalSecs` 以上 86400 以下) | `$null` なら固定間隔 |

`Backoff` を 1 つの値にまとめるのは、「`multiplier` と `max_interval` は両方あるか両方ない」を値の構造で保証するための決定。定義の検証で片方だけは拒否される([03](03-definition-format.md))ので、部分的に存在する状態は作れない。`[uint64]` のプロパティへの代入は、`[uint64]` にキャストした値を明示して行う(`[pscustomobject]@{ IntervalSecs = [uint64]30 }`。リテラルの `30` は `[int]` で、プロパティの型は `[int]` のままになる。`[pscustomobject]` のプロパティは宣言した型を持たないので、代入した値の型がそのまま残る)。

### 7.3 `interval` / `max_interval` の文字列から秒への変換

文字列 → 秒(`ConvertFrom-ConductorInterval`。定義の検証で使う。置き場所は `Private/Definition/`。ここでは落とし穴を示すために再掲する):

```text
1. s の長さ(char の数)が 2 未満なら 拒否
2. 最後の 1 文字が 's' / 'm' / 'h' のどれでもなければ 拒否        # コードユニットの整数 115 / 109 / 104 で比べる(大文字小文字を区別)
3. 残り(digits)の全ての char が '0'〜'9'(整数 48〜57 の範囲比較)でなければ 拒否
   (符号・空白・小数点・'_'・全角数字・"0x" は、ここで弾かれる)
4. digits を 1 文字ずつ走査して n を作る。各桁で n = n * 10 + 桁。
   n は [long] で持つ。単位ごとの上限(下の 5)を超えた時点で拒否する(走査を続けない)。上限が十分小さいので、[long] で桁あふれしない
5. 単位ごとの上限: 's': 1 <= n <= 86400,  'm': 1 <= n <= 1440,  'h': 1 <= n <= 24。外れたら拒否
6. secs = [uint64](n * (1 / 60 / 3600))   # 単位ごと。[long] の乗算(上限内なので収まる)
```

PowerShell の落とし穴:

- **`[int]'...'` / `[long]'...'` / `[int]::Parse` / `[uint64]::Parse` / `[decimal]::Parse` を使わない**: 文字列から数値へのキャストは、先頭・末尾の空白、符号、`0x` などの接頭辞を受理する(確認: `about_Numeric_Literals`。`[int] '0xF'`、`'&hF'`、`'#F'`、`'0b1111'` が受理される)。`"+1s"`・`" 1s"`・`"0x10s"` の受理を避けるため、桁の判定は手順 3 の範囲比較にする。
- `-match '^\d+'` も使わない(§6.1)。`[char]::IsDigit` は Unicode の数字に真を返すので使わない。
- 手順 4 の `n = n * 10 + 桁` は、`[long]` の変数に対して、`[long]` で行う: `$n = $n * 10 + ([int]$c - 48)`。`$n` を `[long]` で初期化(`[long]$n = 0`)すれば、後続の演算で `[int]` のリテラル 10 と混ざっても、結果が `[long]` に収まる限り `[long]` のまま(上限で打ち切るので、収まる)。
- 先頭の `0` は読み飛ばす必要が無い(走査で `n` に 0 が積まれるだけ)。`"0000000000000000000000000001s"` は 1。
- 手順 2・3 の判定が `[char]` の比較でなく整数の比較であることを、§15 のテストで固定する(`"1S"` が拒否される、`"１s"`(全角の 1)が拒否される)。

例: `"1s"`→1、`"01s"`→1、`"0000000000000000000000000001s"`→1、`"1440m"`→86400、`"24h"`→86400、`"86400s"`→86400。拒否: `""`、`"s"`、`"10"`、`"1.5s"`、`"-1s"`、`"+1s"`、`" 1s"`、`"1s "`、`"1S"`、`"１s"`(全角 1)、`"1ms"`、`"0x10s"`、`"1_0s"`、`"0s"`、`"0h"`、`"86401s"`、`"25h"`、`"99999999999999999999999s"`、`"18446744073709551615h"`、`"1s"` + LF。

### 7.4 リトライの対象か(`Test-ConductorRetryTarget`)

次の**すべて**が成り立つときだけ真。

1. `Started = $true`
2. `Interrupted = $false`
3. `StartFailure = $false`
4. `TerminatedBySignal` が無い(`$null`)
5. `ExitCode` があり(`$null -ne`)、`ExitCodes` に含まれる(`[int]` どうしの数値の比較を `foreach` で行う)

分類(成功・警告・エラー)は見ない。対象の終了コードは成功・警告と重ならない(検証済み)ので、結果として対象はエラーの終了コードだけ。ジョブ自身が 127 で終わった場合は、`ExitCodes` に 127 を書いていれば対象(conductor が付けた起動失敗の 127 は、`ExitCode` を持たないので対象にならない)。

### 7.5 判断(`Get-ConductorRetryDecision`(job, execution, retriesDone, lastWaitSecs))

```text
policy = job.Retry か、無ければ return Stop
if !IsRetryTarget(policy, execution) or retriesDone >= policy.Max: return Stop
return Retry { WaitSecs: NextWaitSecs(policy, retriesDone, lastWaitSecs) }
```

戻り値は `[pscustomobject]`: `Kind`(`stop` / `retry`)と、`retry` のとき `WaitSecs`(`[uint64]`)。

### 7.6 待ち時間の式と飽和演算

```text
NextWaitSecs(policy, retriesDone: int, lastWaitSecs: uint64) -> uint64
  if policy.Backoff が有る:
      if retriesDone == 0: return min(policy.IntervalSecs, policy.Backoff.MaxIntervalSecs)   # = IntervalSecs(検証で interval <= max_interval)
      else:                return min(SaturatedProduct(lastWaitSecs, policy.Backoff.Multiplier), policy.Backoff.MaxIntervalSecs)
  else:
      return policy.IntervalSecs                                                              # 毎回同じ(固定間隔)
```

- k 回目のリトライ(k = `retriesDone + 1`)の前の待ち時間を `W(k)` とすると: 固定間隔なら `W(k) = interval`。倍率ありなら `W(1) = interval`、`W(k) = min(W(k-1) × multiplier, max_interval)`(k ≥ 2)。
- 倍率の掛け算は、直前の**実際の**待ち時間(上限で頭打ちされた後の値)に対して行う。
- **`min` と比較は `[bigint]` で行う**: `[uint64]` と `[uint64]` の `-lt` は、型が同じなので問題ないと思われるが、片方が `[int]`・`[long]` のリテラルのとき、数値の昇格の規則(§0.3)に依存する。この仕様は、比較する 2 つの値を、`[System.Numerics.BigInteger]` にキャストしてから `-lt` で比べ、小さい方を `[uint64]` にキャストして返す(`Math.Min` は、過負荷の選択が引数の型に依存するので使わない: 未確認)。
- `[uint64]` へのキャストは、値が `0`〜`[uint64]::MaxValue` の範囲であることを確かめた後にだけ行う(範囲外のキャストは例外になる、と理解している。未確認。`[bigint]` から `[uint64]` へのキャストが正しい値を返すことは、公式ドキュメントでは確認できていない: 未確認。§17 の試験 T-1 で固定する)。キャストが信用できない実装環境では、`[uint64]::Parse($big.ToString([System.Globalization.CultureInfo]::InvariantCulture), [System.Globalization.CultureInfo]::InvariantCulture)` でもよい(結果は同じになること)。

`Get-ConductorSaturatedProduct`(`a`: `[uint64]`, `b`: `[uint64]` または `[int]`)(64 ビット符号なし整数の飽和乗算):

```text
product = [bigint]a * [bigint]b                     # [bigint] どうしの積。昇格も折り返しも起きない
max     = [bigint]18446744073709551615              # [uint64]::MaxValue。[bigint]::Parse でなく、[bigint][uint64]::MaxValue と書く
if product > max: return [uint64]::MaxValue         # 飽和
return [uint64]product
```

- **折り返しも `[double]` への昇格も使わない**。`[uint64]` の `*` を直接使うと、型に収まらないとき `[double]` に昇格し、精度が失われる(§0.3)。たとえば、`(2^64-1) × 10` を `[double]` で計算すると 1.8446744073709552E+20 になり、`[uint64]` に戻せない(範囲外)。どちらでもなく、飽和が規定。
- 前提: `a` は `[uint64]` の任意の値でよい(`[bigint]` は全範囲で正しい)。`b` は 0 以上の `[int]` または `[uint64]`。
- `[bigint]` へのキャスト: `[bigint]$a` は `[uint64]` から正しく変換される(`[bigint]` が `[ulong]` を受け付ける、`about_Numeric_Literals` の型アクセラレーターの表に `[bigint]` がある)。`[uint64]` の変数に入れた値が、途中で `[double]` や `[long]` に変わっていないことが前提なので、`$a` が `[uint64]` であることを、関数の入口で `$a -is [uint64]` で確かめてよい(範囲外の値は内部の誤り)。

境界の例(関数単体の挙動。任意精度整数で計算した値で、PowerShell での実行結果ではない):

| a | b | 積(正確) | 結果 | 理由 |
|---|---|---|---|---|
| 0 | 10 | 0 | 0 | |
| 86400 | 10 | 864000 | 864000 | 検証済みの定義で起こりうる最大の積 |
| 1844674407370955161 | 10 | 18446744073709551610 | 18446744073709551610 | 飽和しない最大に近い値(`2^64-1` を 10 で割って切り捨てた値 × 10) |
| 1844674407370955162 | 10 | 18446744073709551620 | 18446744073709551615 | 積が `2^64-1` を超えるので飽和 |
| 18446744073709551615 | 2 | 36893488147419103230 | 18446744073709551615 | 飽和 |
| 9223372036854775807 | 10 | 92233720368547758070 | 18446744073709551615 | 飽和 |
| 9223372036854775808(2^63) | 2 | 18446744073709551616(2^64) | 18446744073709551615 | 積 2^64 が上限を 1 超えるので飽和。`[double]` に昇格すると 2^64 を表せてしまい、`[uint64]` に戻すとき範囲外になる点が罠 |
| 18446744073709551615 | 10 | 184467440737095516150 | 18446744073709551615 | 飽和 |
| 18446744073709551615 | 1 | 18446744073709551615 | 18446744073709551615 | 飽和ではない(積が上限と等しい。`>` で比べる。`>=` ではない) |

`Get-ConductorNextWaitSecs` の境界例(関数単体。検証済みの定義では起きない入力):

| IntervalSecs | Multiplier | MaxIntervalSecs | retriesDone | lastWaitSecs | 結果 |
|---|---|---|---|---|---|
| 10 | 2 | 60 | 0 | 0 | 10 |
| 10 | 2 | 60 | 3 | 40 | 60(80 を 60 で頭打ち) |
| 10 | 2 | 60 | 4 | 60 | 60 |
| 1 | 10 | 18446744073709551615 | 1 | 9223372036854775807 | 18446744073709551615(積が飽和し、上限で止まる) |
| 1 | 10 | 18446744073709551615 | 1 | 1844674407370955161 | 18446744073709551610(飽和しない最大) |
| 1 | 10 | 18446744073709551615 | 1 | 1844674407370955162 | 18446744073709551615 |
| 30 | (Backoff なし) | - | 5 | 30 | 30 |

検証済みの定義では `interval` と `max_interval` が 86400 秒以下なので、飽和する場面は無い(最大でも 86400 × 10)。再実装でこの極端な入力まで再現する必要は実用上ないが、乗算は折り返さず、`[double]` にもせず、例外にもしないこと。

待ち時間の列の例(計算した値):

| interval | multiplier | max_interval | max | 待ち時間の列(秒) | 合計 |
|---|---|---|---|---|---|
| 30s | なし | なし | 3 | 30, 30, 30 | 90 |
| 10s | 2 | 60s | 5 | 10, 20, 40, 60, 60 | 190 |
| 60s | 3 | 60s | 3 | 60, 60, 60 | 180 |
| 1s | 10 | 3600s(1h) | 6 | 1, 10, 100, 1000, 3600, 3600 | 8311 |
| 1m | 2 | 24h | 4 | 60, 120, 240, 480 | 900 |
| 1h | 10 | 24h | 4 | 3600, 36000, 86400, 86400 | 212400 |

### 7.7 実行ループ(`Invoke-ConductorJobWithRetry`)。`run` と `pipeline` の共通

```text
retriesDone = 0; lastWaitSecs = 0 (uint64)
loop:
    exec = RunJob(job)
    decision = RetryDecision(job, exec, retriesDone, lastWaitSecs)
    if decision.Kind == stop: return exec                 # 最後の結果
    # retry:
    表示: 常に 1 行(§7.9)。retryNumber = retriesDone + 1
    w = Wait(decision.WaitSecs)                           # 本番は §7.8。テストでは差し替える
    if w.Kind == elapsed:     retriesDone += 1; lastWaitSecs = decision.WaitSecs; continue
    if w.Kind == interrupted: return JobExecution(JobId: job.Id, Started: false, ExitCode: null, TerminatedBySignal: null,
                                                  StartFailure: false, Interrupted: true, SignalNumber: w.SignalNumber)
```

- 返された最後の結果だけが、分類・表示・ステート記録の対象(ステートは最終の結果で 1 回だけ書かれ、途中では書かれない)。リトライの回数・直前の待ち時間はローカルな値で、ステートファイルに書かない。
- したがって、conductor が落ちたあとの再開では、そのジョブを最初の実行から数え直す。
- 使い切った(`retriesDone == Max` で、まだ対象の終了コード): `stop` → 通常のエラーと同じ(最後の実行の終了コードで終わり、再開位置はそのジョブ)。再実行の結果が成功・警告なら通常どおり遷移へ進む。
- 総実行回数: 最大 `Max + 1`。知らせの回数: 実際に実行し直す(または待機に入る)回数。
- 起動前に中断された(合図が立っていた)ジョブはリトライせず、その場で `Started = $false` の結果を返す(判断は `Started = $false` で `stop`)。
- 待機は、テストで差し替えられるよう、`Invoke-ConductorJobWithRetry` の引数 `[scriptblock] $Waiter` として受ける(既定は §7.8 の関数)。スクリプトブロックを .NET のデリゲートとして別スレッドから呼ばせない(07 章。ここでは、同じスレッドから呼ぶ通常の呼び出しだけ)。

### 7.8 待機とシグナルによる中断(`Wait-ConductorRetry`)

**決定: 待機は同期で、中断の合図(C# ヘルパーの `ConductorNative.Interrupt`。[07](07-process-and-signals.md) 6 章)を 250 ミリ秒以下の刻みで繰り返し確かめ、刻みごとに合図と単調な時計の期限を確かめる。** 刻みの待ちは `$Interrupt.Wait(ms)`(ヘルパーの `ManualResetEventSlim`。合図が立つと直ちに戻る)で行い、`Start-Sleep` と `[System.Threading.Thread]::Sleep` は使わない(07 章 8 章の規定。コマンドレットは、PowerShell の停止の挙動に依存し、合図で起こせない)。シグナルのコールバックはヘルパー側で完結し、旗を立てるだけにする(07 章)。PowerShell は、先に `$Interrupt.IsRaised`(`[bool]`)を読み、`$true` のときだけ `$Interrupt.Number`(最初のシグナル番号の `[int]`)を読む(番号 0 を「立っていない」と取り違えない。1 つの判断のなかで、それぞれ 1 回だけ読む。§16)。`Start-Job`・`ForEach-Object -Parallel`・`Register-ObjectEvent` は使わない。スレッドを作らない。

手順(`Private/Runner/` の `Wait-ConductorRetry`。07 章 8 章と同じ。時計と待ちは差し替え可能にする: `[scriptblock] $Timestamp`(既定は `{ [System.Diagnostics.Stopwatch]::GetTimestamp() }`)、`[long] $Frequency`(既定は `[System.Diagnostics.Stopwatch]::Frequency`)、`[scriptblock] $WaitRaised`(既定は `{ param($ms) $Interrupt.Wait($ms) }`)):

```text
start  = Timestamp()                                       # [long]
target = SaturatedProduct(secs, frequency)                 # 期限までのティック数。[bigint] で計算して [uint64] に飽和(secs = 2^64-1 でも例外にならない)
loop:
    raised = Interrupt.IsRaised                            # 1 回だけ読む
    if raised: return Interrupted(Interrupt.Number)        # 待機の開始時にも確かめる(最初の番号)
    elapsed = [bigint](Timestamp() - start)                # 単調なので 0 以上。[long] の引き算(収まる)
    if elapsed >= [bigint]target: return Elapsed
    remaining   = [bigint]target - elapsed                 # ティック
    remainingMs = CeilDiv(remaining * 1000, frequency)     # ミリ秒に切り上げ。[bigint] で計算。(a + b - 1) を [bigint]::Divide(…, b)
    chunkMs     = [int]([bigint]::Min(remainingMs, 250))   # 1〜250(250 以下の [int] に収まる)
    WaitRaised(chunkMs)                                    # 戻り値は使わず、ループの先頭で再確認
```

- `[bigint]::Min` は、引数が 2 つの `[System.Numerics.BigInteger]` の静的メソッド(`[System.Numerics.BigInteger]::Min($a, $b)`)。PowerShell から静的メソッドを呼ぶ形は一般に使えるが、この文書では `BigInteger.Min` の API ページを取得していない(未確認)。使えなければ、`-lt` で比べる `if` に置き換える(結果は同じ)。
- 守るべきことは 2 つ: (a) 合図から 1 秒以内に打ち切る(この方式では、`Wait` が合図で直ちに戻るので、通常は即時。最大でも 250 ミリ秒の刻み)。(b) 待ち時間が `secs` 秒以上になるまで `Elapsed` を返さない(経過は単調な時計で毎回確かめる。ミリ秒への換算は**切り上げ**なので、早く戻っても、ループが期限を再確認する)。
- Ctrl+C の扱い: Ctrl+C は、ヘルパーの旗として受ける(07 章)。`$Interrupt.Wait(int)` は .NET のメソッドで、PowerShell の停止の要求で割り込まれない(確認していない。07 章 7.7 節。停止は、メソッドが戻ったあとの文の境目で届く)。この組み合わせで Ctrl+C が実際にどう届くかは未確認(試験 T-7)。

確認(出典: `Start-Sleep` のページ。PowerShell 7.4 版):

- `-Milliseconds` は `[Int32]`(`-Milliseconds <Int32>`)。上限は `[int]::MaxValue`。`-Seconds` は `[Double]`(PowerShell 6.2 以降は小数も受け付ける)で、`double` を経由するので、この仕様では使わない(§0.2 の丸めの方針に反する)。`-Duration` は `[TimeSpan]`(PowerShell 7.3 以降)で、「負でなく `[int]::MaxValue` ミリ秒を超えない」(ページの記載)ので、秒の `[uint64]` から作らない。
- 1 回の待機に渡す刻みは**常に 250 ミリ秒以下の `[int]`**なので、`[int]::MaxValue` ミリ秒(約 24.8 日)を超える待機は、刻みの繰り返しで自然に分割される。別の待機(例: `Process.WaitForExit(int)`、`Interrupt.Wait(int)`)に、ミリ秒の `[bigint]`(`[uint64]`)の値を渡すときの変換は、次の規則で行う:

```text
ToWaitMs(remainingMs: bigint) -> int:
   if remainingMs > [bigint][int]::MaxValue: return [int]::MaxValue
   return [int]remainingMs        # 0 は呼ばない前提(呼び側が先に期限を確かめる)
```

| 待ち(秒) | ミリ秒(正確) | `[int]` に収まるか | 扱い |
|---|---|---|---|
| 86,400(24 時間。定義の上限) | 86,400,000 | 収まる | この方式は 250 ミリ秒ずつ |
| 2,147,483 | 2,147,483,000 | 収まる(`[int]::MaxValue` = 2,147,483,647 以下) | 同上 |
| 2,147,484 | 2,147,484,000 | 収まらない | `ToWaitMs` で `[int]::MaxValue` に切り詰め、繰り返す(`[int]` に収まらない値を `Wait(int)` に渡すと変換が失敗する) |
| 18,446,744,073,709,551,615(`[uint64]::MaxValue`) | 18,446,744,073,709,551,615,000 | `[uint64]` にも収まらない | ミリ秒への換算は `[bigint]` で行う。事実上、終わらない |

- 単調な時計: `[System.Diagnostics.Stopwatch]::GetTimestamp()`(`[long]`)と `[System.Diagnostics.Stopwatch]::Frequency`(ティックの毎秒の数)を使う。ティックの差は `[long]` の引き算で、`Get-Date` の調整(時計合わせ・夏時間)に影響されない。`Stopwatch` が単調増加であること、システムの休止中にティックが進むかどうかは、公式ドキュメントを取得して確認していない(未確認)。分解能は `Frequency` で換算する(Linux・macOS は 10^9、Windows は環境による、と理解している。未確認)ので、秒への換算にティックの数の定数を埋め込まない。
- 旗の読み取りは、ヘルパー側が `Volatile` / `Interlocked` で書き込んだ値を、PowerShell が静的プロパティとして読む(§16)。PowerShell の変数(`$script:interrupted` など)に旗を持たない(別スレッドから書けず、書けても可視性の保証が無い)。

待機中に中断された場合は、次の実行をせず、分類なし(起動しなかった中断)として扱う: 終了コード `128 + 番号`、ジョブの状態は変えず(エラーとして記録しない)、再開位置はそのジョブ。`pipeline` でも、そこで止まって終わる(§5.3: 分類なしは停止)。再開すると、そのジョブを最初の実行から数え直す。

テストでは、`$Waiter` を偽の実装(待ち時間の `[uint64]` を記録するだけで実際には待たない)に、待機の実装自体のテストでは `$Timestamp`・`$Frequency`・`$WaitRaised` を偽の時計に差し替える。別のプロセスからシグナルを送る受け入れテストは [11-quality-and-build.md](11-quality-and-build.md)。

### 7.9 リトライの知らせ

待機の**前**に、`-v` の有無によらず、標準エラーに 1 行(エラーではない。`conductor: retry: ` で始まる)。待機が中断された場合も、その回の知らせは出ている。

- 英語: `conductor: retry: job '<id>' exited with code <last_exit_code>; retry <n> of <max> in <wait_secs> seconds`
- 日本語: `conductor: retry: ジョブ '<id>' が終了コード <last_exit_code> で終わりました。<wait_secs> 秒後にリトライします(<n>/<max> 回目)`
- `<last_exit_code>` は直前の実行の終了コード(`ExitCode` が無ければ 0)。`<n>` は 1 から。`<wait_secs>` は秒の整数(10 進、桁区切りなし。`$waitSecs.ToString([System.Globalization.CultureInfo]::InvariantCulture)`)で、1 のときも `seconds`(単数形の処理はない)。`<n>`・`<max>`・`<last_exit_code>` も同じく `.ToString([System.Globalization.CultureInfo]::InvariantCulture)`。`ToString('N0')` や、文字列補間(`"$n"`)・`-f` の既定は使わない(PowerShell の文字列補間が現在の文化を使うかは、公式ドキュメントを取得して確認していない: 未確認。`-replace` の左辺の変換が PowerShell 7.2 以降は文化に依存しないことは確認(出典: `about_Comparison_Operators`)だが、他の演算子には当てはまらない)。`<id>` は表示用にエスケープ・長さ制限される(規則は [08-output-and-i18n.md](08-output-and-i18n.md))。
- 例: `conductor: retry: job 'extract' exited with code 75; retry 2 of 5 in 20 seconds`

---

## 8. シグナルと中断(流れへの影響)

| 受信のタイミング | ジョブの結果 | ステートの記録 | 終了コード |
|---|---|---|---|
| ジョブの起動の前(`run`) | 起動しない。分類なし | ジョブの状態は変えない。`resume_job_id` = そのジョブ(初期ステートは、空だった場合に書かれている) | 128 + n |
| ジョブの実行中 | 子に同じシグナルを 1 回転送。終了を待つ。`Interrupted`。分類 error | そのジョブを `error` で記録(`exit_code` はジョブの値があればそれ) | 128 + n |
| リトライの待機中 | 分類なし(§7.8) | 状態は変えない。再開位置 = そのジョブ | 128 + n |
| `pipeline` で、ジョブを実行・記録した直後(まだ完了していない) | - | 再開位置(§9)を記録 | `Get-ConductorSignalExitCode(n)` = 128 + n |
| 定義の検査・ステートの確認の途中 | 検査は最後まで進む | 64・65・66・スキップ(0)が優先 | それぞれ |

- シグナルは SIGHUP・SIGINT・SIGQUIT・SIGTERM(UNIX 系)、Ctrl+C(Windows)。2 回目以降のシグナルは追加の動作をしない。終了コードに使う番号は、最初に受け取ったもの。詳細は [07-process-and-signals.md](07-process-and-signals.md)。
- 実行中の中断ではジョブ自身の終了コードは見ない(分類 error、返す値は 128 + n)。

---

## 9. 再開位置(`Get-ConductorResumePoint`)

入力: 定義、各ジョブの状態。

```text
ResumeJobId = 記述順で最初の「pending(または記録なし)または error」のジョブ(無ければ null)
Completed   = (ResumeJobId == null)
EndReached  = 「skipped(byEnd)」のジョブが 1 つでもあれば true
```

エラー・pending を区別せず、記述順で最初のものが再開位置。エラーのジョブが前にあれば、そのジョブ。この関数は記録を作らない(呼び出し側が `Set-ConductorInterruptRecord`(§3.5)で書く)。

---

## 10. 完了と完了後

- 完了(`Completed = $true`)になるのは: 最後のジョブが success / warning で終わった、`@end` に到達した(pipeline、または individual で後ろに未完了が無い)、スキップの記録で最後のジョブまで済んだ(§3.4)。
- 完了のステートは `completed = true` のまま**残る**(`remove_on_complete` が真でない限り)。書式は変わらない。
- 完了のステートは、読み込み(`run`・`pipeline` の `Prepare`)では「空」として扱う(定義との照合もしない)。次の `pipeline`・`run <最初のジョブ>` は先頭から始まり、初期ステートで上書きする。先頭以外のジョブを `run` で単独に呼ぶと順序外(66)。
- `remove_on_complete` が真のとき、完了のステートを書いた後に `ResetState` で削除する(失敗は 65。完了のステートは残る)。偽のときは削除を試みず、0 で終わる(`run` ではジョブの結果による終了コード)。
- `watch` は完了のステートも表示する(表の下に `completed`/`完了` の行)。

---

## 11. 保証の範囲(at-least-once)

- 結果の記録は、ジョブの終了の**後**。終了と書き込みのあいだに conductor が落ちると、ジョブは未実行として扱われ、再実行されうる(ジョブが 2 回動きうる)。
- 結果の書き込みに失敗したときは、ジョブの終了コードではなく 65 で終わり、「ジョブは実行されました」と表示する。
- `retry` のジョブは、最大 `Max + 1` 回動く。
- 時間の上限(タイムアウト)はない。ジョブが終わらなければ待ち続ける。
- 排他制御はしない([06-state-file.md](06-state-file.md) §11)。

---

## 12. 実装の特記事項(再実装で再現すること)

| # | 項目 | 挙動 |
|---|---|---|
| 1 | リトライの知らせの単数形 | 待ちが 1 秒のとき `in 1 seconds`(英語)。単数形の処理はない |
| 2 | エラーのジョブを再実行して、起動前・待機中に中断されたとき | 分類なしなので `status` は error のまま変わらないが、`exit_code` は消える(結果の記録が `ExitCode` に実行結果の値 = `$null` を書く) |
| 3 | 成功直後の `resume_job_id` | 成功・警告のあとの `NextStep.ResumeJobId` は `$null`。ただし後続のスキップ記録は、エラーが無いとき `ResumeJobId` を最初の pending にする。そのため、遷移で飛ばした後のステートには、成功直後でも `resume_job_id` が入りうる |
| 4 | 分類なし(起動前の中断)の終了コードの整形 | `isError = $false` で整形する。`128 + n` が 256 以上かつ下位 8 ビットが 0 になる番号(n = 128 など)は、0 になりうる(実在のシグナル番号では起きない) |
| 5 | `pipeline` の「完了を判定できない」 | 全ジョブを通過しても `Completed` が真にならなければ 65(ステートは初期化しない) |
| 6 | ステートの誤りの原因の文 | 種類の文はロケールに従うが、理由(例: `JSON として読めません`)は常に日本語の固定文(06 §6.4) |
| 7 | ステート値の全項目の比較 | 配列を要素ごとに比べる(`[pscustomobject]` の `-eq` は参照の比較。§0.2) |

---

## 13. `Flow` の純粋関数の署名の骨格

実装ではなく、名前・引数・戻り値の骨格。`Private/Flow/` の関数は、すべて例外を投げず(不正な入力の組み合わせは、表の規則どおりの値を返す。生成関数の集合外の文字列だけは内部の誤り)、画面に何も出さない。

```powershell
# 判断に必要な部分だけの定義の見え方(03 章の検証済みの定義から取り出す。[pscustomobject])
#   FlowJob        : Id [string], SuccessExitCodes [int[]](省略されていれば @(0)), WarningExitCodes [int[]],
#                    OnSuccess / OnWarning(遷移の値。$null、'@end' を表す値、またはジョブID。種類は 03 章), Retry(RetryPolicy か $null)
#   FlowDefinition : PipelineName [string], Jobs [object[]](FlowJob の列。記述順)
# 遷移の値の表現は 03 章が正本(ここでは、種類を値の形で区別できることだけを要求する)

function Get-ConductorOutcome {                    # §2.3
    [OutputType([pscustomobject])]
    param([Parameter(Mandatory)] [pscustomobject] $Job,
          [Parameter(Mandatory)] [pscustomobject] $Execution)
    # 戻り値: Classification ($null/'success'/'warning'/'error'), ReportedExitCode [int], ConductorError [bool]
}

function Get-ConductorClassifiedStep {             # §3。戻り値: [pscustomobject]@{ Outcome = ...; Next = ... }
    [OutputType([pscustomobject])]
    param([Parameter(Mandatory)] [pscustomobject] $Job,
          [Parameter(Mandatory)] [pscustomobject] $Execution,
          [Parameter(Mandatory)] [AllowEmptyCollection()] [object[]] $States,
          [Parameter(Mandatory)] [pscustomobject] $Definition,
          [Parameter(Mandatory)] [string] $Mode)         # 'individual' | 'pipeline'
}

function Get-ConductorNextStep {                   # §3.2
    [OutputType([pscustomobject])]
    param([Parameter(Mandatory)] [pscustomobject] $Definition, [Parameter(Mandatory)] [string] $JobId,
          [AllowNull()] [object] $Classification,       # $null = 分類なし
          [Parameter(Mandatory)] [AllowEmptyCollection()] [object[]] $States, [Parameter(Mandatory)] [string] $Mode)
}

function Get-ConductorDecision {                   # §4。戻り値: Kind ('run'|'skip'|'reject'), Reason ($null|'alreadyExecuted'|'skipTarget'|'afterEnd'|'outOfOrder')
    [OutputType([pscustomobject])]
    param([Parameter(Mandatory)] [string] $JobId, [Parameter(Mandatory)] [pscustomobject] $Definition,
          [Parameter(Mandatory)] [AllowEmptyCollection()] [object[]] $States,
          [AllowNull()] [object] $ResumeJobId, [Parameter(Mandatory)] [bool] $EndReached)
}

function Get-ConductorResumePoint {                # §9。戻り値: ResumeJobId, Completed, EndReached
    [OutputType([pscustomobject])]
    param([Parameter(Mandatory)] [pscustomobject] $Definition, [Parameter(Mandatory)] [AllowEmptyCollection()] [object[]] $States)
}

function Test-ConductorRetryTarget { [OutputType([bool])]            param([pscustomobject] $Policy, [pscustomobject] $Execution) }           # §7.4
function Get-ConductorRetryDecision { [OutputType([pscustomobject])] param([pscustomobject] $Job, [pscustomobject] $Execution,
                                                                            [int] $RetriesDone, [uint64] $LastWaitSecs) }                         # §7.5。Kind 'stop'|'retry', WaitSecs [uint64]
function Get-ConductorNextWaitSecs { [OutputType([uint64])]          param([pscustomobject] $Policy, [int] $RetriesDone, [uint64] $LastWaitSecs) }   # §7.6
function Get-ConductorSaturatedProduct { [OutputType([uint64])]      param([uint64] $A, [uint64] $B) }                                        # §7.6。B に [int] を渡すときは [uint64] に変換して渡す
function Add-ConductorInt32Saturating { [OutputType([int])]          param([int] $A, [int] $B) }                                              # §2.2
function ConvertTo-ConductorProcessExitCode { [OutputType([int])]    param([int] $Reported, [bool] $IsError) }                                # §2.2
function Get-ConductorSignalExitCode { [OutputType([int])]           param([int] $SignalNumber) }                                             # §2.2

function New-ConductorInitialState { [OutputType([pscustomobject])]  param([string] $DefinitionPath, [pscustomobject] $Definition, [AllowNull()] [object] $RunId) }   # 06 §2.4
function Set-ConductorOutcomeRecord { [OutputType([pscustomobject])] param([pscustomobject] $State, [string] $JobId, [pscustomobject] $Execution, [pscustomobject] $Outcome, [pscustomobject] $Next) }   # §3.3
function Set-ConductorSkipRecord { [OutputType([pscustomobject])]    param([pscustomobject] $State, [string] $JobId, [string] $Reason, [pscustomobject] $Definition) }   # §3.4
function Set-ConductorInterruptRecord { [OutputType([pscustomobject])] param([pscustomobject] $State, [pscustomobject] $Point) }                                  # §3.5
function Test-ConductorStateEqual { [OutputType([bool])]             param([pscustomobject] $Left, [pscustomobject] $Right) }                # §0.2
```

- `[OutputType()]` は**文書**で、実際の出力を検査も変換もしない(確認: `about_Functions_OutputTypeAttribute`「The OutputType attribute value isn't derived from the function code or compared to the actual function output. As such, the value might be inaccurate」)。型の誤りは、`tests/` の単体テストが、戻り値の `GetType()` や `-is` で確かめる(§15)。
- `Set-` で始まる関数(記録)は、承認された動詞の `Set` を使うが、副作用は無い(引数の値を変えず、新しい値を返す)。PSScriptAnalyzer の `PSUseShouldProcessForStateChangingFunctions` は、この名前に反応するので、`Private/Flow/` に限り、設定でこの規則を除外する([11](11-quality-and-build.md))。`SupportsShouldProcess` を付けて見せかけの対応をしない。
- 引数が配列のパラメーターに空配列を渡すには、`[AllowEmptyCollection()]` が要る(`[Parameter(Mandatory)]` は、空の配列を既定で拒否する。公式ドキュメントの確認はしていない: 未確認)。
- 関数の呼び出しは、**スペース区切りの引数**で書く(`Get-Foo -A 1 -B 2`)。`Get-Foo(1, 2)` や `Get-Foo(1)` のようなメソッド風の呼び出しは使わない(確認: `Set-StrictMode`: `add(3,4)` は、`(3,4)` が 1 つの配列として渡され、厳格モード 2.0 以降ではエラーになる)。

---

## 14. PowerShell 固有の注意(判定表の実装に影響する点)

### 14.1 `-eq` と文字列の比較

- `-eq`・`-ne`・`-lt`・`-gt`・`-like`・`-match`・`-contains`・`-in`・`switch` は、文字列を**既定で大文字小文字を区別せず**比べる(確認: `about_Comparison_Operators`、`about_Switch`)。ジョブID・状態・理由・run ID の比較は、すべて `-ceq` / `-cne` / `-ccontains` / `-cin`、または `[string]::Equals($a, $b, [System.StringComparison]::Ordinal)`。`ABC` と `abc` を同じジョブIDとして扱うと、ステートの照合(§4、06 §7)が壊れる。
- `-eq` は、**右辺の型に左辺を変換する**(確認: 同ページ。`1 -eq '1.0'` は True、`'1.0' -eq 1` は False)。値の型が混ざる比較(`[int]` と `[string]`)を書かない。比べる前に、両側を同じ型に明示的に直す。
- `-eq` の左辺が配列のとき、`-eq` は一致した要素を返す(真偽ではない。確認: 同ページ)。左辺がスカラーであることを確かめてから使う。
- 比較は InvariantCulture で、コードポイントの比較(確認: 同ページ)。文化依存の比較にはならないが、大文字小文字の区別が既定で無い点は上のとおり。

### 14.2 配列の単一化・展開

- 関数の出力として配列を返すと、PowerShell は**要素を 1 つずつ展開**して流す(確認: `about_Return`「When you return a collection from your scriptblock or function, PowerShell automatically unrolls the members」)。0 要素の配列は何も出力せず、受け取り側では `$null`、1 要素の配列は単独の値、2 要素以上は配列になる。この規則は、`Where-Object`・`ForEach-Object`・`Select-Object` のパイプラインの結果にも当てはまる(結果が 0 件・1 件・複数件で型が変わる。確認: `about_Comparison_Operators` の `-eq` の例: 一致が無ければ空の配列を返す)。
- この仕様の規則: **`Private/Flow/` の関数は、配列を直接返さない**。配列は、`[pscustomobject]` のプロパティとして返す(§3.1 の `SkippedJobIds` など)。配列を直接返す関数(`Get-ConductorSkippedJobIds` のような内部の補助)は、`return , $array`(単項のカンマ演算子。確認: `about_Return` の「To force a scriptblock or function to return collection as a single object」)で返し、呼び出し側は `$x = Get-...` と、**代入だけ**で受ける(`@(Get-...)` で包むと、1 要素の配列を入れた配列になる。パイプラインに流さない)。`[AllowEmptyCollection()]` と空配列の `return , @()` の受け取りが `$null` にならないことは、§17 の試験 T-3 で固定する。
- プロパティ `Jobs` や `SkippedJobIds` を `foreach` で走査する前に、`@()` で包む(`@($state.Jobs)`)必要があるかを、実装は判断しない: プロパティの値は、生成時に必ず配列(`[object[]]`・`[string[]]`)で作り、`foreach` は配列にも、(`$null` でない)単独の値にも使える。`$null` のプロパティを `foreach` に渡すと、`$null` を 1 要素として走査するか、0 要素として走査するかが版で異なりうる(未確認)ので、`$null` を配列の位置に入れない(空配列を使う)。
- 配列への要素の追加は、`+=` でなく、`[System.Collections.Generic.List[string]]`(`.Add()` は何も返さない)を使い、最後に `.ToArray()` する(`+=` は、毎回、新しい配列を作る。確認: `about_Arithmetic_Operators` の「When you concatenate collections ... a new object is created」。要素数が 100 件程度なら性能は問題ないが、ステートの `Jobs` の複製では、新しい配列を作ることが規定なので、`+=` を使わず、`List` から作る)。`System.Collections.ArrayList.Add` は挿入位置の整数を返す(出力に混ざる)ので使わない。
- 1 要素の配列を JSON に書くとき、`ConvertTo-Json` は使わない([06](06-state-file.md) §4.3: 配列の単一化を避けるため、自前の書き出しを使う)。

### 14.3 `$null` の比較

- `$null` との比較は、**`$null` を左辺**に置く: `$null -eq $x`、`$null -ne $x`(確認: `about_Comparison_Operators`「if you need to determine whether a variable is `$null`, you must put `$null` on the left-hand side」。`$a -ne $null` は、`$a` が配列のとき `$null` の要素を取り除いた配列を返す)。この文書のすべての「無し」の判定がこれに従う。
- `if ($x)` の真偽の判定は、`$null` だけでなく、`0`・空文字列・空配列・`$false` も偽にする(PowerShell の真偽の規則。`about_If` 等。この文書では取得していない: 一般に知られた挙動)。有る・無いの判定に `if ($x)` を使わない(`ExitCode` の 0、`SignalNumber` の 0、空のジョブIDが、「無し」と同じになる)。
- `Set-StrictMode -Version Latest` のもとでは、初期化されていない変数の参照と、存在しないプロパティの参照が、終了エラーになる(確認: `Set-StrictMode`。2.0 以降)。したがって、`$null` のプロパティは「存在して値が `$null`」と作る(`[pscustomobject]@{ ExitCode = $null }`)。プロパティを省略して「無し」を表さない。Latest は将来の版で厳しくなりうる(確認: 同ページの Caution「Using Latest ... isn't deterministic」)ので、`#requires` で版を固定したうえで、最小版と最新版の両方でテストする([11](11-quality-and-build.md))。

### 14.4 戻り値への出力の混入

- 関数は、`return` の式だけでなく、**文の結果のすべて**を出力する(確認: `about_Return`「the results of each statement are returned as output, even without a statement that contains the return keyword」。`"Please wait..."` の文字列が戻り値に混ざる例)。したがって、メソッドが値を返す呼び出し(`$list.Add($x)`(`ArrayList` のとき)、`$set.Add($x)`(`HashSet` は `bool` を返す)、`$sb.Append(...)`(`StringBuilder` を返す)、`[Environment]::...`、`$proc.Start()`(`bool`)、`$matches`)は、結果を `[void]`、または `$null = ...` で捨てる。**製品コードの関数は、戻り値以外のものを出力しない**。
- 画面への出力は、専用の関数(`Report` の `Write-ConductorStream`。`Send-Conductor*` 経由)だけが、標準出力・標準エラーのストリームを包んだ `StreamWriter` に書く(08 章 1.2.2。`[Console]::Out` / `[Console]::Error` は使わない)。`Write-Host`・`Write-Output`・`Write-Error`・`Write-Warning` は使わない(PSScriptAnalyzer で検査。[11](11-quality-and-build.md))。
- 戻り値を `return` で明示する(最後の式の値に頼らない)。関数の最後が `if` / `foreach` の文のとき、その中の式の結果が出力に混ざりうる。単体テストで、各関数の戻り値の個数(`@($result).Count -eq 1`、`$result -is [pscustomobject]`)を確かめる。

### 14.5 クラスを使わない理由と `Import-Module -Force`

§0.2 のとおり、値は `[pscustomobject]` で表す。`class` を使う場合は、(a) メソッドは `return` で指定した値だけを出力する(確認: `about_Return` の Note)、(b) `-eq` は `IEquatable` を実装しない限り参照の比較(確認: `about_Comparison_Operators`)、(c) 再読み込みで型が再定義されうる、の 3 点に注意が要る。この仕様では `class` を使わないので、これらは関係しない。

### 14.6 `switch` を使わない理由

`switch` は、(a) 入力と条件の値をすべて**文字列に変換**して比べる(確認: `about_Switch`「The switch statement converts all values to strings before comparison」。列挙型・数値が意図しない一致をする)、(b) 既定で**大文字小文字を区別しない**(同ページ)、(c) 一致した**すべての節を実行**する(`break` / `continue` が無い限り。同ページの `3` が 2 回一致する例)、(d) 入力が配列のとき、**要素ごと**に評価する、(e) `-Regex`・`-Wildcard` の指定が、他の指定と競合したとき、最後のものが勝つ、(f) 節の条件がスクリプトブロックのとき、`$_` を使って式として評価する、という挙動を持つ。この仕様の判定表(§2.3、§3.2、§4.2)は、`if` / `elseif` の連鎖で書く。例外として、`switch -CaseSensitive -Exact` で文字列定数の写像を書いてよいが、必ず `-CaseSensitive` を付け、全節の末尾に `break` を書き、`default` を持つ。

### 14.7 ループと制御

- `foreach` 文では `break` / `continue` はループに作用する。`ForEach-Object`(パイプライン)のスクリプトブロックの中の `return` は、そのブロックの反復を抜けるだけで、`break` / `continue` は外側のループやスクリプトの終了にまで作用しうる。`Private/Flow/` と `Cli/` の流れの制御には `foreach` 文と `for` 文だけを使い、`ForEach-Object` / `Where-Object` / `.ForEach()` / `.Where()` を使わない。
- `[int]` へのキャストは、小数を**最近接の偶数へ丸める**(確認: `about_Arithmetic_Operators`「DIVISION AND ROUNDING」)。切り捨てが必要な箇所は `[Math]::Floor`、切り上げは `[Math]::Ceiling`、`[bigint]::Divide`。この仕様の整数計算は、小数を作らない。
- `-split`・`-replace` は、既定で**正規表現**(確認: `about_Comparison_Operators`「-replace ... use regular expressions」)。文字列の分割・置換は、`.Split([char])`・`.Replace(string, string)` などの .NET のメソッドを使う(`-split` を使うときは、`[regex]::Escape` で特殊文字を無効にする)。

### 14.8 `$LASTEXITCODE`・`exit`・`return`

- この文書の流れの関数は、終了コードを `[int]` で**返す**(`return`)。`exit` を呼ぶのは、ランチャーだけ([02](02-architecture.md))。モジュールの中で `exit` を呼ぶと、呼び出し元のセッションごと終了する。
- `$LASTEXITCODE` は、ネイティブコマンドを `&` で呼んだときの終了コードを持つが、この仕様は子プロセスを `System.Diagnostics.Process` で起動し(07 章)、`Process.ExitCode` を使う。`$LASTEXITCODE` と `$?` は使わない。

---

## 15. 単体テストで固定する項目(この文書の範囲)

実機で動かせていない箇所を、実装時に次のテストで固定する(Pester。[11](11-quality-and-build.md))。

| 項目 | 内容 |
|---|---|
| `ConvertTo-ConductorProcessExitCode` | §2.2 の表の全行。`reported = [int]::MinValue` / `[int]::MaxValue`、`isError = $true`/`$false` |
| `Add-ConductorInt32Saturating` | `(128, [int]::MaxValue)` → `[int]::MaxValue`、`([int]::MinValue, -1)` → `[int]::MinValue`。戻り値の型が `[int]` |
| `Get-ConductorSaturatedProduct` | §7.6 の境界の表の全行。戻り値の型が `[uint64]`。特に 2^63 × 2、`[uint64]::MaxValue` × 1 |
| `Get-ConductorNextWaitSecs` | §7.6 の境界の表と待ち時間の列の表 |
| `Test-ConductorRunId` | §6.1 の有効・無効の例と境界の表。LF 付きの値、全角 `１`、孤立サロゲート、`'あ' * 30`、`'A'` と `'a'` を含む値。走査と `GetByteCount` の照合 |
| `ConvertFrom-ConductorInterval` | §7.3 の例(受理・拒否の全件)。`"1S"`、全角、`"0x10s"` |
| `Test-ConductorStateEqual` | §0.2 の 3 つの性質 |
| `Get-ConductorOutcome` / `Get-ConductorNextStep` / `Get-ConductorDecision` | §2.3・§3.2・§4.2 の表の全行。`ExitCode` が 0 のとき「有る」と扱う(`$null -ne` で判定)こと、状態の文字列が大文字小文字違い(`Success`)のとき内部の誤りになること、ジョブIDの `a` と `A` が別であること |
| 戻り値の個数と型 | `Flow` の全関数について、`@(& $fn ...).Count` が 1 であること(出力の混入が無い)と、`-is` による型の確認。配列のプロパティが、0 件・1 件で `[object[]]`・`[string[]]` のままであること(T-3) |
| `Wait-ConductorRetry` | 偽の時計で、`secs = 0`、`secs = 1`、`secs = [uint64]::MaxValue`(例外が出ず、`Elapsed` が返らず、旗で `Interrupted` が返る)、開始時にすでに旗が立っている場合、`remainingMs` が `[int]::MaxValue` を超える場合の分割 |
| AST の検査 | `Private/Flow/` に禁止の呼び出し(`Write-Host`、`Write-Output`、`Write-Error`、`Start-Sleep`、`Get-Date`、`[Environment]`、`switch`、`ForEach-Object`、`Where-Object`、`-match`)が無い |
| 受け入れ(別プロセスから送信) | 待機中の SIGTERM で 143、1 秒以内に終了 |

---

## 16. 共有状態とスレッド間の可視性

シグナルのコールバックは、メインスレッド以外(スレッドプールなど)で実行されうる([07](07-process-and-signals.md)。この文書では公式ドキュメントを取得して確認していない: 未確認)。したがって、メインスレッドとコールバックのあいだで共有する値は、次の表の方法だけで読み書きする。

| 共有する値 | 所有者 | 書き手 | 読み手 | 方法 | 終了条件 |
|---|---|---|---|---|---|
| 中断の合図(「立っているか」と最初のシグナル番号) | ヘルパー型 `ConductorNative.Interrupt`(07 章 6 章)のインスタンス。`Prepare` が作り、`SignalWatcher.Start` で登録する(登録はヘルパーが静的に保持する) | シグナルのコールバック(ヘルパー側。複数のコールバックが同時に呼ばれうる) | メインスレッド(待機のループ、ジョブの待機、`pipeline` のジョブのあいだ) | 書き込みはヘルパーが、「立っているか」と番号を 1 つの値にまとめ、`Interlocked.CompareExchange` で 1 度だけ書く(最初の 1 つだけが勝つ)。読み込みは PowerShell から `IsRaised`(`Volatile.Read`)を先に読み、`$true` のときだけ `Number` を読む | プロセスの終了まで。立ったら戻さない |
| 子への転送を済ませたか | `Invoke-ConductorJob`(07) | 07 の規定 | 同上 | 07 の規定 | ジョブの終了まで |
| ステート値・定義・実行結果 | メインスレッド | メインスレッドだけ | メインスレッドだけ | 共有しない(不変の値を引数で渡す) | - |

- **PowerShell のスクリプトブロックを .NET のデリゲート(シグナルのコールバック)にしない**。別スレッドで呼ばれると、そのスレッドに実行空間(runspace)が無く、失敗する問題が知られている(07 章 7.1 節。この文書では公式ドキュメントを取得していない: 未確認)。旗の更新はヘルパーの .NET のコード(`Add-Type` で与える)の中で完結させ、PowerShell は旗を読むだけにする。
- 旗を PowerShell の変数(`$script:`・`$global:`)に持たない。ヘルパーの `Interrupt` のメンバーを読む。
- 「立った」と番号は、ヘルパーの内部で 1 つの値にまとめて書かれる(`Interlocked.CompareExchange`)。読み手は、`IsRaised` が `$true` のときだけ `Number` を読み、「無し」か「完全な番号」のどちらかだけを見る。番号 0 や負の番号でも「立った」ことは記録される(07 章 6.1 節)。
- 1 つの判断のなかで、`IsRaised` と `Number` を、それぞれ 1 回だけ読み、その値を使う(例: `pipeline` のジョブのあいだの確認と、`Get-ConductorSignalExitCode` の引数 `n` は、同じ読み取りの結果)。
- ハンドラ(コールバック)の中で、ファイルの書き込み・画面への出力・`Process` のメンバーの呼び出し(子への転送を除く。転送は 07 の規定)をしない。旗を立てることだけを行う。
- `Start-Job`・`Start-ThreadJob`・`ForEach-Object -Parallel`・`Register-ObjectEvent`・`Register-EngineEvent` を使わない(07 章の決定)。`[System.Threading.Tasks.Task]` の `.Wait()`・`.Result` を使わない。

---

## 17. 確認の範囲と未確認

確認できたこと(出典は本文の該当箇所。いずれも learn.microsoft.com の PowerShell 7.4 版のドキュメントを取得して確認):

- `about_Numeric_Literals`: 10 進整数リテラルの型の選択順(`int` → `long` → `decimal` → `double`)、接尾辞 `u`・`ul`・`n`(`n` は 7.0 以降)、乗数、文字列から数値へのキャストが `0x` などを受理すること。
- `about_Arithmetic_Operators`: 結果が型に収まらないとき型が広がる(`Int32 * Int32` → `Double`、`[ulong]::MaxValue + 1` → `Double`)、`Double` に昇格した値を `[ulong]` に戻すと精度が失われる例、`[decimal]` の例外、`[bigint]` 演算は全オペランドを `[bigint]` にする、除算とキャストの偶数丸め、`-bnot`・ビット演算が 64 ビット整数。
- `about_Comparison_Operators`: 文字列の比較は既定で大文字小文字を区別しない(`c` 版で区別)、InvariantCulture で、コードポイントの比較、`-eq` は右辺の型に左辺を変換する、左辺が配列のとき一致した要素を返す、`$null` は左辺に置く、`-match` は `$Matches` を書き換える、`-replace` は正規表現、`[regex]::new(pattern, options, timeout)` の例、`IEquatable` を実装しないオブジェクトの `-eq` は参照の比較。
- `about_Switch`: 文字列への変換、既定で大文字小文字を区別しない、一致した全節を実行、`break` / `continue`、配列の要素ごとの評価。
- `about_Return`: 文の結果がすべて出力される、配列の展開、`return (, $array)`、クラスのメソッドは `return` の値だけを出力。
- `about_Functions_OutputTypeAttribute`: `OutputType` は実際の出力と照合されない。
- `Set-StrictMode`: 2.0 以降で存在しない変数・プロパティの参照が終了エラー、3.0 以降で範囲外の添字がエラー、`Latest` は将来厳しくなりうる、メソッド風の関数呼び出しの禁止。
- `Start-Sleep`: `-Milliseconds` は `[Int32]`、`-Seconds` は `[Double]`、`-Duration` は `[int]::MaxValue` ミリ秒以内(7.3 以降)、Ctrl+C は `Start-Sleep` を中断するが `[Threading.Thread]::Sleep` は中断しない。
- `about_Language_Modes`: ConstrainedLanguage では `[bigint]`・`[uint64]`・`[regex]`・`[pscustomobject]`・`[hashtable]`・`[string]` などの限られた型だけが使える(`[System.Diagnostics.Stopwatch]`・`[System.Numerics.BigInteger]` などは許可の一覧に無い。`[bigint]` は許可の一覧にある)。**ConstrainedLanguage のもとでは、この文書の `[System.Environment]::`・`[System.Diagnostics.Stopwatch]::`・`[System.Text.Encoding]::`・`[System.Numerics.BigInteger]::` などの .NET の静的メソッドの呼び出しが失敗する可能性が高い**(許可の一覧に無い型は使えない、という規定による。確認: 同ページ)。この仕様は、ConstrainedLanguage での動作を保証しない(動かないときは、起動時に検出して、70 と理由で終わる。[10](10-errors-and-exit-codes.md))。
- `File.Move(String, String, Boolean)`・`UTF8Encoding(Boolean, Boolean)`・`FileStream.Flush(Boolean)` などの .NET の API は、[06-state-file.md](06-state-file.md) に根拠を書く。

未確認(実装時に確認する。下の試験の番号は、最初に確かめる小さな試験):

- T-1: `[bigint]` から `[uint64]` へのキャストが、範囲内の全値(特に 2^63 以上)で正しいこと、範囲外で例外になること。`[uint64]` の `*` が `[double]` になること(使わないが、挙動の確認)。
- T-2: `-band 0xFF` が負の `[int]` で、2 の補数の下位 8 ビットを返すこと(§0.2、§2.2)。
- T-3: `return , @()` を `$x = f` で受けたとき `$x` が空配列(`$null` でない)になること。1 要素の配列が単一化されないこと。`[AllowEmptyCollection()]` と `[Parameter(Mandatory)]` の組み合わせ。
- T-4: `[int]$x = $null` が 0 になること(型付きのパラメーター・変数に `$null` を入れない規則の根拠)。`[AllowNull()] [object]` の `$null` が保たれること。
- T-5: `[bigint]` を `-lt`・`-gt` で比べたときの結果、`[System.Numerics.BigInteger]::Divide`・`Min` の呼び出し。
- T-6: `[int]$s[$i]`(`[char]` から `[int]`)がコードユニットの値を返すこと。`'A' -ge 'a'` が True になるか(大文字小文字を区別しない比較の確認)。
- T-7: ヘルパーが `CancelKeyPress` を `Cancel = $true` で処理するとき、`pwsh -File` 実行中のスクリプトが `PipelineStoppedException` で止まらないこと。`Interrupt.Wait(int)` の刻みの待機が、`PipelineStoppedException` の要求で途中で割り込まれず、合図で直ちに戻ること(07 章と合わせて確認)。
- `Stopwatch` の単調性・休止中の挙動・`Frequency`(§7.8)。
- `Process.ExitCode` が Windows で負数として現れること(§1.2)。
- Unix の引数・環境変数の不正な UTF-8 が `[string]` にどう入るか(§6.1)。
- `[System.Environment]::GetEnvironmentVariable` の未定義・空・名前の大文字小文字の扱い(§6.2)。`$env:NAME` と空の値の関係。
- PowerShell の文字列補間・`-f` が現在の文化を使うか(§7.9。使わない規定にしたので、出力の正しさには影響しない)。
- シグナルのコールバックが実際にどのスレッドで実行されるか(07)。複数のシグナルがほぼ同時に届いたときの「最初の番号」は、コールバックが呼ばれた順序に依存する。

実装に任せてよい点(外から見える挙動が同じなら自由):

- 判断関数・記録関数の内部構造、補助関数の分け方。
- 待機の刻みの具体値(250 ミリ秒以下。「合図から 1 秒以内」「`secs` 以上で `Elapsed`」を守る)。
- 標準エラーの出力のバッファリング(ただし行単位で、順序を保つ)。
