# 10. エラーと終了コード

この章は、PowerShell 7.x(`pwsh`)で実装する `conductor` の終了コードの全件、複数の条件が当てはまるときの優先順位、ジョブの結果から終了コードへの対応、終了コードの整形、想定外の内部失敗の扱い、そして PowerShell でのエラーの設計を規定する。利用者から見えるものは「エラー分類」「終了コード」「メッセージ」で規定し、値の形は 8 章で規定する。メッセージの全文は [08-output-and-i18n.md](08-output-and-i18n.md)、引数の誤りは [04-cli.md](04-cli.md)、流れは [05-execution-flow.md](05-execution-flow.md)、シグナルは [07-process-and-signals.md](07-process-and-signals.md)、ステートは [06-state-file.md](06-state-file.md)、定義は [03-definition-format.md](03-definition-format.md)、変換は [09-converter.md](09-converter.md) を参照する。

「確認(出典: …)」は、公式ドキュメント(learn.microsoft.com の PowerShell 7.4 の about_*)または PowerShell の公式リポジトリのソースを取得して確かめた内容、「未確認」は、この環境に `pwsh` も .NET SDK もなく実機で動かせないこと、または出典を取得できなかったことで、確かめられていない内容である。未確認の事項は、実装の最初に、04 章 11.2 の試験と、この章 10 章の一覧で確かめる。

## 1. 終了コードの一覧(全件)

| 終了コード | 名前 | 意味 | 主な発生箇所 |
|------------|------|------|--------------|
| 0 | OK | 成功。ジョブの成功・警告、スキップ(実行済み・遷移・`@end`)、`watch`・`reset`・`validate`・`convert` の成功、`--help`・`--version`。`reset --all` の警告(読めなくて残したファイル)も 0 | 全般 |
| 1 | UNLISTED_ZERO | ジョブが終了コード 0 で終わったが、`success_exit_codes` に 0 がない。または、ジョブの終了コードがエラーで、下位 8 ビットが 0 になる(例: 256)。または、起動して終わったのに終了コードが得られない(起こらない想定) | ジョブの結果の分類、終了コードの範囲合わせ |
| 1〜255 | (ジョブの値) | エラーになったジョブの終了コードを、そのまま。範囲外は下位 8 ビット(0 になるなら 1)。4 章 | 範囲合わせ |
| 64 | DEFINITION | 定義の誤り: ファイルがない・読めない・大きすぎる・文法・検証の誤り(パイプライン名の不一致、`run_id_env` や `remove_on_complete` の型・形の誤り、`retry` の誤りを含む)、拡張子が `.psd1`・`.json` のどれでもない、`command` の誤り(引用符が閉じていない・空)。`validate` の失敗も 64 | `run`, `pipeline`, `validate` |
| 65 | STATE | ステートの誤り: 読めない・大きすぎる・壊れている・版の不一致・定義との食い違い・中身の `run_id` とファイル名の食い違い・書き込みの失敗・削除(初期化)の失敗。`reset`・`reset --all` で消すのに失敗。`watch` が標準出力に書けない。`pipeline` が完了を判定できない | `run`, `pipeline`, `watch`, `reset` |
| 66 | REJECTED | 拒否(順序外。`run` で、前のジョブがまだ済んでいない、など。05 章の `Reject(OutOfOrder)`)。スキップ対象・`@end` より後ろ・実行済みのジョブは、拒否ではなくスキップで、終了コード 0 | `run`, `pipeline` |
| 67 | CONVERSION | 変換の誤り。`convert` が標準出力に書けない | `convert` |
| 68 | USAGE | 使い方の誤り(知らないコマンド、引数の不足、余分な引数、知らないオプション、使えないオプション、値の欠落・誤り、知らないジョブID、run ID の値の誤り〔引数・環境変数とも〕、重複、`--all` と `--run-id` の併用、文字として正しくない引数) | 引数の解釈、run ID の決定、`watch`/`reset` |
| 70 | INTERNAL | 内部の誤り: 想定外の内部失敗(6 章)、シグナルの準備の失敗、ランチャーが `pwsh` を見つけられない(04 章 1.3) | 全般、`run`/`pipeline` の準備 |
| 127 | START_FAILURE | ジョブを起動できなかった(実行ファイルがない・実行権限がない、など。終了の状態を取得できない場合も安全側で起動失敗扱い) | ジョブの結果の分類 |
| 128 + 番号 | SIGNAL_BASE | ジョブがシグナルで終了した、または `conductor` が中断された(例: SIGHUP = 129、SIGINT = 130、SIGQUIT = 131、SIGTERM = 143)。リトライの待機中の中断も含む | ジョブの結果の分類、中断 |

- 64〜68 と 70 は、ジョブ自身の終了コードと重なりうる。区別は、標準エラーの `conductor: error: ...` の行で行う(70 は固定の英語の文)。
- **`pwsh` 自身が、起動の失敗に 64 と 70 を使う**。確認(出典: `ConsoleHost.cs` の定数): `ExitCodeBadCommandLineParameter = 64`、`ExitCodeInitFailure = 70`、`ExitCodeSuccess = 0`。ランチャーが `pwsh` に渡すコマンドラインは固定なので、64 は通常は現れない。70 は、`pwsh` の初期化の失敗と、`conductor` の内部の誤りで、同じ数値になる(標準エラーの行で区別する。`pwsh` 自身の失敗では、`conductor: error: internal error` の行が出ないことがある)。
- 終了コードの範囲(0〜255)への合わせ方は 4 章。
- `pwsh -File` は、スクリプトが例外で終わると終了コード 1 で終わる(確認(出典: about_Pwsh、about_Language_Keywords))。これは `conductor` の規定にない終了コードの出方であり、実装は、これらが外へ出ないようにする(想定外の失敗は 70。6 章)。ただし、入口スクリプト自身の構文の誤りなど、`try` の外で起きる失敗は、`pwsh` の終了コード 1 になりうる(未確認)。

## 2. 終了コードの決定順序

`run` / `pipeline` は、次の順に検査し、最初に当てはまった時点で終わる(それ以降は行わない)。

| 順 | 検査 | 失敗時 |
|----|------|--------|
| 0 | 引数の解釈(オプションとして解釈された `-h`/`-V` は先に 0。値を取るオプションの値として取られた `--help` などは対象外。それ以外の使い方の誤り。04 章 4.1・4.2) | 68 |
| 1 | シグナルの監視の準備(定義の検査より前)。「すでに登録済み」は成功と同じに扱う | 70 |
| 2 | 定義の読み込みと検証(パイプライン名の照合を含む。誤りは全部まとめて 1 行ずつ表示) | 64 |
| 3 | 全ジョブの `command` の検査(誤りのあるジョブごとに 1 行) | 64 |
| 4 | `run` のジョブIDが定義にあるか | 68 |
| 5 | run ID の決定(`--run-id` または環境変数。規則違反) | 68 |
| 6 | ステートの読み込み | 65 |
| 7 | ステートと定義の照合 | 65 |
| 8 | 起動前の判断 | 拒否は 66。スキップは 0(`run` は 0 で終わる。`pipeline` は続ける) |
| 9 | (実行のとき)初期ステートの書き込み | 65(ジョブは起動していない) |
| 10 | ジョブの実行(リトライ込み)、結果の分類 | 結果から決まる(3 章) |
| 11 | 結果のステートの書き込み | 65(ジョブは実行済み。ジョブの終了コードより優先) |
| 12 | 完了で、削除が求められていれば、ステートの削除 | 65 |

補足:

- 検査やステートの確認の途中でシグナルを受け取っても、検査は最後まで進む。定義の誤り(64)、ステートの誤り(65)、拒否(66)、スキップ(0)の終了コードが、中断の終了コードより優先する。
- `watch`: run ID の決定(68)→ ステートの読み込み(65)→ 表の出力(書けなければ 65)。成功は 0。定義は検証しない。シグナルの準備はしない。
- `reset`: run ID の決定(68。`--all` のときは決定しない)→ 削除(65)。成功は 0。
- `validate`: 定義(64)→ `command`(64)。成功は 0。ステートに触れない。シグナルの準備はしない。
- `convert`: 変換(67)→ 標準出力への書き込み(書けなければ 67)。成功は 0。
- `--help`・`--version`: 標準出力に書けなくても 0(書き込みの失敗は無視する。08 章 1.3.2)。標準出力に書けない失敗は、`watch` が 65、`convert` が 67、それ以外の経路で未捕捉になったものは 70(6 章)。**既知の差**: UNIX 系では、読み手が先に閉じたパイプへの書き込みは、.NET の標準のストリームが捨てるため検出できず、`watch`・`convert` が 65・67 でなく 0 で終わりうる(08 章 1.3.1)。Windows と、`/dev/full` などの `EPIPE` 以外の失敗では、65・67 になる(Windows は未確認)。
- `pipeline` の特有: 実行したジョブがエラー(または分類なし)なら、そのジョブの終了コードで終わる。ジョブのあいだに中断の合図があれば `128 + 番号`。完了なら 0。完了を判定できなければ 65。
- ステートの書き込みに失敗すると、ジョブの終了コードではなく 65 で終わり、「ジョブは実行されました」と表示する(少なくとも 1 回は実行された、という方針)。起動前の失敗(初期ステートの書き込みなど)は「ジョブは起動していません」。
- 想定外の内部失敗(6 章)は、どの時点でも 70 で終わる(上の順序の外)。

## 3. ジョブの結果から終了コードへ(分類)

最初に当てはまるものを採る。この判断は `Private/Flow/` の純粋な関数(ジョブの実行結果の値を入力に、分類と報告する終了コードを返す。`Get-ConductorOutcome`)として実装する([05-execution-flow.md](05-execution-flow.md))。

| 順 | 状況 | 分類 | 報告する終了コード | 備考 |
|----|------|------|--------------------|------|
| 1 | 起動しておらず、中断された | 分類なし | `128 + 番号` | エラーとして記録しない。再開位置はそのジョブ |
| 2 | 起動しておらず、中断でもない(起こらない想定) | エラー | 127 | 起動失敗と同じ扱い |
| 3 | 起動済みで中断された | エラー | `128 + 番号` | ジョブ自身の終了コードは見ない |
| 4 | 起動失敗 | エラー | 127 | |
| 5 | ジョブがシグナルで終了 | エラー | `128 + シグナル番号` | |
| 6 | 起動して終わったが、終了コードがない | エラー | 1 | `conductor` のエラー(`success_exit_codes` に含まれない 0 と同じ表示) |
| 7 | 成功コード(`success_exit_codes`、省略時は `[0]`)に含まれる | 成功 | 0 | |
| 8 | 警告コード(`warning_exit_codes`)に含まれる | 警告 | 0 | |
| 9 | 終了コードが 0 だが成功コードにない | エラー | 1 | `conductor` のエラー。`job '<ID>': exit code 0 is not listed in success_exit_codes` を常に表示 |
| 10 | それ以外 | エラー | ジョブの終了コード | 範囲を 0〜255 に合わせる(4 章) |

プロセスの終了コード = `ConvertTo-ConductorProcessExitCode -Reported <報告する終了コード> -IsError <エラーか>`(4 章)。`run` は、この値で終わる。`pipeline` は、エラーまたは分類なしのジョブで止まるとき同じ式で、完了なら 0。

注: 「起動した」は、子プロセスを作れたこと(`[System.Diagnostics.Process]::Start` が例外を送出せずに戻り、`Process` のオブジェクトが得られること)を指す。「中断」は、`conductor` がシグナル(07 章)を受け取り、中断の合図が立ったこと。ジョブがシグナルで終了した(順 5)のと区別する。

.NET の `Process` からの「終了の取り出し」(詳細は 07 章):

- 子の終了は、07 章の規則(`Process.ExitCode` から、終了コードとシグナル終了の有無を決める)で、ジョブの実行結果の値(05 章)の `ExitCode`・`TerminatedBySignal` にしてから、この章の分類に渡す。この章の分類は、その値の項目(`ExitCode`、`TerminatedBySignal`、`StartFailure`)だけを入力にする(`Process.ExitCode` の生の値の意味づけを、この章に持ち込まない)。順 5 は `TerminatedBySignal` が有るとき、順 6 は `ExitCode` も `TerminatedBySignal` も無いとき、順 7 以降は `ExitCode` が有るとき。
- Windows では、`Process.ExitCode` は符号つき 32 ビットの値で、例外コード(例: `0xC0000005`)が負の値として現れうる(値の符号の扱いは 07 章。ここでは未確認)。`ExitCode` は、`[int]` のまま 4 章の整形に渡す(下位 8 ビットを取る。符号つき・符号なしのどちらの解釈でも結果が同じ)。Windows では、ジョブの終了コードが 255 を超える値(例: 256)になりうるので、4 章の範囲合わせの対象になる。
- 起動の失敗: `Process.Start` は、起動に失敗したとき、`System.ComponentModel.Win32Exception` を送出する(ファイルがない・実行権限がない、など。例外の型の詳細は 07 章。ここでは公式ドキュメントを取得しておらず、未確認)。順 4(起動失敗)として扱う。**捕捉するのは起動に関する例外の型(07 章が列挙するもの)だけ**とし、`[System.Exception]` 全体は捕捉しない(想定外の失敗を、起動失敗に化けさせないため)。確認(出典: about_Try_Catch_Finally): `catch` に .NET の例外の型を書くと、その型と、派生した型を捕捉する(例の `catch [System.Net.WebException],[System.IO.IOException]` は、`DownloadFile` メソッドの呼び出しで起きる例外を、型で捕捉している)。PowerShell は、すべての例外を `RuntimeException` に包む(同)ので、型を指定しない `catch` は、あらゆる例外を捕捉する。
- 起動に失敗した場合は、`Process` の `WaitForExit` を呼ぶ対象がない(重複して分類しない。起動の例外の捕捉と、終了の待機は、1 つの `try` の中で、後者を前者の成功後にだけ行う)。

## 4. 終了コードの範囲合わせ

`ConvertTo-ConductorProcessExitCode -Reported [int] -IsError [bool]` → `[int]`(`Private/Flow/`。05 章が正):

| 入力 | 出力 |
|------|------|
| `0 <= reported <= 255` | `reported` |
| 範囲外 | `reported` の下位 8 ビット(負の値は 2 の補数表現の下位 8 ビット)。ただし、その結果が 0 で `isError` が真なら 1(エラーを成功に見せない) |

計算の書き方(PowerShell の演算の既定の挙動に頼らない。05 章 2.2 節が正本):

- 下位 8 ビットは、05 章 2.2 節のとおり `$reported -band 0xFF`(2 の補数表現の下位 8 ビット)で求める。負の値での `-band` の結果そのものは、取得した出典で確認できていない(未確認。05 章 §0.2)。そこで、単体テストで、この章の判定表(下の例)の全行を固定し、外れたときは、`$low = (([long]$reported % 256) + 256) % 256` に切り替える(`[long]` にしてから計算するので、`[int]::MinValue` でも桁あふれしない。`%` が負の被除数で返す値は未確認だが、この式は、剰余が負になる実装でも非負になる実装でも、同じ結果になる)。
- `128 + n`(シグナル番号 `n` は小さな正の整数)の計算は、`[long]` で行ってから、この関数に渡す。
- 入力の型は `[int]`(`Process.ExitCode` と `128 + n`)。`[int]` へのキャストは、小数を銀行丸めにするが、ここでは整数だけを扱う。

例(判定表。受け入れテストの入力に使う): 259 → 3、511 → 255、-1 → 255、-2 → 254、`0xC0000005`(符号なしで 3221225477。符号つき 32 ビットの `-1073741819` と同じビット列)→ 5。256・512・`-256`・`[int]::MinValue` は、エラーなら 1、エラーでなければ 0。

- `Get-ConductorSignalExitCode -Signal [int]` = `ConvertTo-ConductorProcessExitCode -Reported (Add-ConductorInt32Saturating 128 n) -IsError $true`(05 章 2.2 節が正本)。15 → 143、2 → 130、1 → 129、3 → 131、300 → 172(428 の下位 8 ビット)。`n` が負のときの規定はない(呼び出し側が正の番号だけを渡す)。
- `-v` の結果行には、元の値(`exit code <C>`)と、`conductor` の終了コードが異なるとき(`result=error` のみ)`, conductor exits with <E>` を示す(08 章 2.1)。

### 4.1 プロセスの終了(入口スクリプトの `exit`)

入口スクリプト `bin/conductor.ps1` が、`Invoke-Conductor` の戻り値(`[int]`。すでに 0〜255 に合わせたもの)を、08 章 1.4 の手順で出力を完了させたあと、**`exit` に渡す**(04 章 1.3)。モジュールの関数は `exit` を呼ばない(対話セッションごと終了してしまうため)。終了コードは、必ず 0〜255 の `[int]` にしてから渡す。

確認(出典: about_Language_Keywords の `exit`、about_Pwsh、about_Automatic_Variables):

| 事項 | 内容 |
|------|------|
| `exit` の値の範囲 | 「On Windows, any number between `[int]::MinValue` and `[int]::MaxValue` is allowed. On Unix, only positive numbers between `[byte]::MinValue` and `[byte]::MaxValue` are allowed. A negative number in the range of `-1` through `-255` is automatically translated into a positive number by adding 256. For example, `-2` is transformed to `254`.」 |
| 範囲外・数値でない値 | 「Any argument that is non-numeric or outside the platform-specific range is translated to the value of `0`.」。**UNIX 系で `exit 256` は 0 になる**(エラーを成功に見せる)。したがって、この仕様は、`exit` に 0〜255 の `[int]` だけを渡す(4 章の整形)。`[int]` でない値(`[uint64]`、`[double]`、文字列)を `exit` に渡さない |
| `pwsh -File` の終了コード | 「When you run `pwsh.exe -File <path to a script>` and the script file terminates with an `exit` command, the exit code is set to the numeric argument used with the `exit` command. If the script has no `exit` statement, the exit code is always `0` when the script completes without error or `1` when the script terminates from an unhandled exception.」 |
| `$LASTEXITCODE` | `-File` で呼ばれたスクリプトでは、例外で終われば 1、`exit` があればその値、正常終了なら 0 |
| `exit` と `finally` | 「A `finally` block also runs if an `exit` keyword stops the script from within a `catch` block.」(`catch` の中の `exit` では、`finally` が実行される) |
| `-Command` との違い | `-Command` では、最後のコマンドが 0・1 以外の終了コードを設定しても、プロセスの終了コードは 1 に変換される。「To preserve the specific exit code, add `exit $LASTEXITCODE`」。この仕様は `-Command` を使わない(04 章 1.6) |
| 終了コードを決めるホストの処理 | 確認(出典: `ConsoleHost.cs`): ファイルの実行が失敗すると、終了コードは 0x1 に設定される。`exit` による値があればそれが使われる。ホストは、最後に `$global:?` を調べ、偽なら終了コードを 1(`ExitCodeSuccess \| 0x1`)にする |

OS ごとの範囲(確認できた範囲): Windows は 32 ビットの値(`exit` は `[int]` の全範囲)。UNIX 系は、`exit` が 0〜255 だけを受ける(上の表)。この仕様では、`ConvertTo-ConductorProcessExitCode` で自分で 0〜255 に合わせた値だけを `exit` に渡し、OS の切り詰めや、`exit` の範囲外の値が 0 になる挙動に頼らない。**`pwsh` が `exit` の値をそのままプロセスの終了コードにすること、`$global:?` の検査が `exit` の値を上書きしないこと(たとえば `exit 66` のあとの `$?`)は、公式ドキュメントでは確認できていない**(ソースの断片から、`exit` による値が優先されるとみえるが、確定していない)。試験 T2(04 章 11.2)で、0、1、64、66、255 を確かめる。

`Invoke-Conductor` の戻り値の型は `[int]` で、`[void]` にしない(終了コードを持たない経路を作らない)。入口スクリプトは、`exit` の前に、戻り値が `[int]` で 0〜255 であることを確かめ、そうでなければ 70 にする。

## 5. シグナルと中断

- UNIX 系: `SIGHUP`(1)・`SIGINT`(2)・`SIGQUIT`(3)・`SIGTERM`(15)を、07 章が規定する方式(インラインの .NET コードの静的なフラグ。PowerShell のスクリプトブロックを、別のスレッドから呼ばれるコールバックにしない)で受け取り、最初の番号を中断の合図(フラグ)に記録するだけにする。コールバックは、メインスレッドと別のスレッドで実行されうる。ほぼ同時に複数届くと、番号の小さい方が「最初」になりうる(順序は実装依存。必須ではない)。
- Windows: Ctrl+C(番号 2)だけ(07 章)。Ctrl+Break とコンソールを閉じたときの通知は対象外(OS の既定の動作)。
- `run`・`pipeline` だけがシグナルを準備する。`watch`・`reset`・`validate`・`convert` は、シグナルを準備しない。これらのコマンドの実行中のシグナルは、`pwsh` の既定の動作に任せる。**既定の動作**: 確認(出典: about_Pwsh): `-File` で実行したスクリプトが Ctrl+C で中断されると、終了コードは 0 である(「unlike with `-Command`, when the execution is interrupted with Ctrl+C the exit code is `0`」)。確認(出典: `ConsoleHost.cs`): Ctrl+C は、ホストの割り込みの処理が、実行中のパイプラインを停止させる(`StopAsync`)。したがって、これらのコマンドの実行中の Ctrl+C は、終了コード 0 で終わりうる(**`run`・`pipeline` が 130 を返す場面との既知の差**。受け入れテストは、この場面の終了コードを検査しない)。SIGTERM などの、端末の Ctrl+C 以外のシグナルでの終了コードは未確認。
- 起動済みのジョブが中断されると、`128 + 番号`、エラーとして記録する。起動前、またはジョブのあいだ、リトライの待機中に中断されたときは、エラーとして記録せず、再開位置を記録して `128 + 番号` で終わる。
- **`run`・`pipeline` で、Ctrl+C に対して、終了コード `128 + 2 = 130` で終わること**は、`pwsh` が Ctrl+C でパイプラインを停止し(上)、`-File` の終了コードが 0 になる(上)ことと、両立させる必要がある。次の 3 点を、実装の最初に確かめる(試験 T4)。(1) 07 章の方式の登録が、`pwsh` 自身の Ctrl+C の処理(パイプラインの停止)を止められるか(止められるなら、`Invoke-Conductor` は通常の経路で 130 を返せる)。(2) 止められないとき、パイプラインの停止のあと、入口スクリプトの `finally` の中で、ヘルパーが予約した `128 + 番号`(なければ 130)を `[System.Environment]::Exit` に渡して、プロセスの終了コードを決められるか(07 章 7.7 節の規定)(`finally` の実行は確認(出典: about_Try_Catch_Finally): 「A `finally` block runs even if you use CTRL+C to stop the script」)。(3) どちらもできないとき、Ctrl+C の終了コードは 0 になる。この場合は、この章に追記して、既知の差として受け入れる。
- 準備の失敗は、`conductor: error: failed to set up signal handling` を出して 70。同じプロセスで 2 回目の登録になる「すでに登録済み」は、成功と同じに扱う(実装が複数回呼ぶ構造のときだけ関係する)。登録などの失敗は、シグナルの口の実装が、準備できなかったとき、値(失敗)として返し、`run` がそれを準備の失敗(70)にする(07 章)。
- 詳細は [07-process-and-signals.md](07-process-and-signals.md)。

## 6. 想定外の内部失敗(終了コード 70)

実装のバグなどで、設計が想定していない失敗(規定された値の形以外のあらゆる例外。`null` の参照、範囲外の添字、型の変換の誤り、メモリ不足、存在しないコマンドの呼び出し(`CommandNotFoundException`)、`Set-StrictMode` が検出する未定義の変数・プロパティの参照、`finally` の中の例外など)が起きたときの規定。

### 6.1 必須の挙動

1. `Invoke-Conductor` の本体は、全体を 1 つの `try`/`catch` で囲む。`catch` は、型を指定しない形、または `[System.Exception]` を指定する形で、**この 1 か所だけ**が `[System.Exception]` 全体を捕捉する(8.1)。`catch` の中で、10 章 6.1 の 2 の 1 行を書き、`70` を `return` する。入口スクリプト(04 章 1.3.4)にも、同じ規定の `try`/`catch` がある(モジュールの読み込みの失敗、`Invoke-Conductor` の戻り値の検査の失敗を捕捉する)。確認(出典: about_Try_Catch_Finally): 「statement-terminating errors that stop the current statement and script-terminating errors that unwind the entire call stack. Both kinds are caught by `try/catch`.」(どちらの種類の誤りも `try/catch` で捕捉できる)。

   ```powershell
   function Invoke-Conductor {
       [OutputType([int])]
       param([string[]] $ArgumentList = @())
       try {
           # 本体。戻り値は [int] を 1 つだけ(8.1)
       }
       catch {
           return (Write-ConductorInternalError)   # 70 を返す
       }
   }
   ```

   (骨格。実装コードは規定しない。`catch` の中の処理自体が例外を送出しないよう、`Write-ConductorInternalError` の内部は、特定の例外の型を捕捉して無視する。)
2. `Write-ConductorInternalError` は、標準エラーに、固定の英語の文を 1 行書く(ロケールを判定しない)。

   ```text
   conductor: error: internal error
   ```

   末尾は改行(LF)。書き込みは、**08 章の出力口を使わず**、`[System.Console]::OpenStandardError()` で得たストリームに、ASCII の定数のバイト列(`'conductor: error: internal error' + "`n"` の UTF-8 バイト列)を直接 `Write` し、`Flush()` する。`[System.IO.IOException]`・`[System.ObjectDisposedException]`・`[System.NotSupportedException]` を捕捉して無視する(出力口の状態(作れていない・書き込みに失敗済み)や、符号化の失敗に左右されないため)。この処理は、**1 回だけ**動く(`$script:InternalErrorReported` のフラグで止める。2 回目以降の呼び出しは、何もしない。例外の連鎖で行が重複して出ないように)。`Write-Host`・`Write-Error`・`Write-Output` を使わない(08 章 1.5)。
3. **`$ErrorActionPreference = 'Stop'` を、モジュールの先頭(`Conductor.psm1`)と、入口スクリプトの先頭で設定する**。確認(出典: about_Preference_Variables, `$ErrorActionPreference`): この変数は、「applies to **both** non-terminating and statement-terminating errors」で、`Stop` のとき、「it escalates non-terminating errors to script-terminating errors」。確認(出典: about_Try_Catch_Finally): 「A statement-terminating error stops the current statement from running, but PowerShell continues at the next statement unless the error is also script-terminating.」。**したがって、`Stop` を設定しないと、内側の関数で文が失敗しても、次の文から実行が続き、誤った状態のまま処理が進む**。また、確認(出典: about_Try_Catch_Finally の注): 「`$ErrorActionPreference` can suppress `throw`. When set to `SilentlyContinue` or `Ignore`, the error doesn't propagate」。呼び出し元(対話セッション)が `SilentlyContinue` を設定していても、モジュールの中では `Stop` が効くことを、試験で確かめる(モジュールの関数が、呼び出し元のスコープの設定を引き継ぐかは未確認。試験 T8)。**未確認**: `Stop` が、内側の関数の、.NET メソッドの例外(文を止める誤り)を、確実に外側の `try`/`catch` まで伝えること。確かめて伝わらなければ、境界の操作(ファイル入出力、プロセス)を、狭い `try`/`catch` で包む規則(8.1)を、すべての .NET メソッド呼び出しに広げる。
4. `Set-StrictMode -Version Latest` を、モジュールの先頭で設定する。未定義の変数やプロパティの参照は、実行時の誤りになり、6.1 の 1 の `catch` に届く(バグを、黙って `$null` として流さない)。
5. スタックトレースや失敗の詳細を、標準出力へ出してはならない(標準出力はジョブの出力と `watch`・`convert` の結果のためにある)。詳細(例外の `ToString()`)を標準エラーへ出すかは、実装に任せてよい(出す場合も上の 1 行を、必ず含める。1 行目に出す)。既定は出さない。
6. 外部の入力(引数、環境変数、ファイル、シグナル、ジョブの出力)に対して、想定外の失敗が起きない作りにする。これらは、すべて、規定された誤り(64〜68 など)として扱う。たとえば、配列の範囲外の参照、`$null` のメンバーの参照、ハッシュテーブルの欠けたキー、型の変換の誤り(`[int]` へのキャストの失敗)、文字コードの誤り(UTF-8 の読み取りは、不正な列で例外を投げる `[System.Text.UTF8Encoding]::new($false, $true)` で行い、規定の誤りに変換する)、整数への変換の誤り、深い入れ子の再帰(6.3)が、入力から起きないようにする。入力の検査に、`Assert` 系の仕組みに頼らない。

### 6.2 入口スクリプトと、スレッド

- 入口スクリプトの `try`/`catch` は、メインスレッドで起きた例外を捕捉する。メインスレッド以外(`Process` のイベント、シグナルのコールバックなど)で起きた捕捉されない例外は、`pwsh` のプロセスを終了させうる(終了コードは未確認)。シグナルのコールバックは、フラグを立てるだけで、例外を投げない作りにする(07 章。インラインの .NET コードで完結させる)ので、通常は起こらない。**PowerShell のスクリプトブロックを、.NET のデリゲート(イベントのコールバック)にしない**(別のスレッドで呼ばれたとき、実行空間(runspace)が無く失敗する。07 章)。
- `exit` を、`Invoke-Conductor` と、それが呼ぶ関数の中で呼ばない(対話セッションが終了する。入口スクリプトだけが呼ぶ)。`[System.Environment]::Exit` も、通常の経路で呼ばない(`finally` が実行されない)。
- モジュールが読み込む外部のコードは、`pwsh` を終了させる呼び出し(`exit`、`[System.Environment]::Exit`)を行わない(実行時の依存は無い。01 章)。

### 6.3 捕捉できないもの、確認した挙動

- **スタックの溢れ**: 捕捉できない(.NET の既知の挙動。取得した出典はなく、未確認)。プロセスは異常終了する(終了コードは OS によって違い、未確認)。**対策(必須)**: 入力に由来する入れ子(JSON の構造、`.psd1` の構文木、定義の木)を、再帰で処理しない。再帰下降の解析器・書き出し器を作らず、**明示的なスタック(`[System.Collections.Generic.Stack[object]]`)で反復する**。入れ子の深さは、03 章・09 章の上限(32 段。根を数えず、33 段目を拒否する)で拒否する。`[System.Management.Automation.Language.Parser]` が作る構文木の走査も、再帰でなく、明示のスタックで行う(構文木の深さの上限は、パーサーが持つか未確認。03 章が、深さの検査を先に行うことを規定する)。PowerShell の関数の再帰の深さに、実行環境が上限を持つか(「call depth」の誤り)は未確認である。
- **メモリ不足**: 実際のメモリ不足では、投げられても、その後の処理(標準エラーへの書き込み、例外オブジェクトの作成)が失敗しうる。6.1 の 1 の `catch` が捕捉できれば 70 にするが、保証しない(この仕様の対象外。メモリ不足を握りつぶして続行しない)。入力の大きさの上限(ファイルサイズ 1 MiB、要素数 100,000、文字の大きさ 16 MiB)で、通常の入力では起きないようにする。
- そのほか、プロセスが実行環境の致命的な異常で終わる場合(実行環境のクラッシュ、OS による強制終了(`SIGKILL`))は、ハンドラが動かず、終了コードは 70 にならない。この場合は仕様の対象外とする。
- 終了を止める例外(スレッドの中断など)は、現在の .NET では使えない(使わない)。

ジョブの実行中に想定外の失敗が起きたとき:

- 子プロセスの終了を待たずにプロセスが終わり、ジョブが孤児として残りうる(待つことは必須ではない)。
- ステートは直前の完全な内容のままであること(書き込みは、一時ファイルに書いて置き換える形で、途中の内容が見えない形で行う。[06-state-file.md](06-state-file.md))。次の実行は、そのジョブを再実行しうる(少なくとも 1 回の方針)。

### 6.4 Ctrl+C と、パイプラインの停止

確認(出典: about_Try_Catch_Finally): 「A `finally` block runs even if you use CTRL+C to stop the script.」。「Note that pressing CTRL+C stops the pipeline. Objects that are sent to the pipeline will not be displayed as output.」(Ctrl+C はパイプラインを止める。`finally` の中でパイプラインに出した値は表示されない。**だから、`finally` の中の出力は、パイプラインではなく、08 章の出力口(標準ストリームへの直接の書き込み)で行う**)。確認(出典: about_Language_Keywords): `finally` は、Ctrl+C で抜けるときにも、`exit` を使うときにも実行される。確認(出典: `ConsoleHost.cs`): Ctrl+C は、ホストの割り込みの処理が、実行中のパイプラインに停止を要求する(`StopAsync`)。

規定:

- **後始末は `finally` に書く**: 子プロセスの終了の待機、ステートの書き込み、シグナルの登録の解除、`[System.Console]::OutputEncoding` の復元は、`try`/`finally` の `finally` で行う。Ctrl+C でパイプラインが止められても、`finally` は実行される。
- 入口スクリプトと `Invoke-Conductor` の `catch` は、パイプラインの停止を表す例外(`System.Management.Automation.PipelineStoppedException`。型名は 07 章 7.7 節。以降、この例外と呼ぶ)に対して、`conductor: error: internal error` の行を**出さない**(Ctrl+C は内部の誤りではない)。そのため、型を指定した `catch` を、型を指定しない `catch` の前に置く(確認(出典: about_Try_Catch_Finally): 「If you specify an error class and one of its derived classes, place the `catch` block for the derived class before the `catch` block for the general class.」)。この例外が、`try`/`catch` の `catch` で捕捉できるのか、捕捉されずに `finally` だけが実行されて伝わるのかは、取得した出典に記述がなく**未確認**(試験 T4)。いずれの場合でも、「内部の誤りの行を出さない」ことと、「`finally` で後始末が行われる」ことを守る。
- 中断の終了コード(130 など)の決め方は、5 章の 3 点の確認の結果に従う。
- `trap` を使わない(確認(出典: about_Try_Catch_Finally): `try` の中に `trap` があると、一致する `catch` があっても `trap` が先に処理を取る。意図しない経路を作らない)。

## 7. エラー分類の一覧

内部の値の形は 8 章で規定する。利用者から見える区分として、次の分類と対応を守る。

### 7.1 定義の誤り(終了コード 64)

31 種。種類の識別子と、英語・日本語の文は [08-output-and-i18n.md](08-output-and-i18n.md) 4.3 に全件ある(03 章と同じ)。

識別子: `unsupportedFormat`, `fileMissing`, `unreadable`, `tooLarge`, `syntax`, `typeMismatch`, `unknownKey`, `versionMissing`, `versionUnsupported`, `unknownTarget`, `errorTransition`, `backwardOrSelfTarget`, `overlappingExitCodes`, `zeroAsWarning`, `pipelineNameMismatch`, `duplicateJobId`, `invalidJobId`, `noJobs`, `emptyField`, `duplicateTransition`, `invalidEnvName`, `retryNotTable`, `retryMissingKey`, `retryExitCodesEmpty`, `retryExitCodeInvalid`, `retryExitCodesOverlap`, `retryMaxInvalid`, `retryDurationInvalid`, `retryBackoffIncomplete`, `retryMultiplierInvalid`, `retryMaxIntervalTooSmall`。すべて終了コード 64。

`unsupportedFormat` は、定義ファイルの拡張子が `.psd1`・`.json` のどれでもないとき(拡張子の比較は大文字小文字を区別しない)の種類で、原因の文は 03 章が規定する(行はない。原因は、`拡張子は .psd1、.json のいずれかで書きます`)。

誤りが持つ情報: ファイル(表示するパス)、行(あれば)、種類、原因(日本語の固定の文)。表示は `<ファイル>:<行>: <種類の文>: <原因>`(行がなければ `<ファイル>: <種類の文>: <原因>`)。原因には、入力の値を含めない(入力に由来するのはキー名だけで、`` `キー名` `` の形で入り、64 文字で切る)。原因の文の全件は [03-definition-format.md](03-definition-format.md) にある。

run ID の規則違反(`Empty`, `TooLong`, `InvalidChar`, `LeadingDot` の 4 通り)は、利用者には区別を見せず、すべて `invalid value`(68)にまとめ、値を含めない。

### 7.2 `command` の誤りとステートの誤り

`command` の誤り(終了コード 64): 2 種。`UnclosedQuote`(引用符が閉じていない)、`empty`(コマンド名がない)。表示は 08 章 4.4 の種類の文だけ(原因は表示しない。内部の原因の文は、前者が `引用符が閉じていません`、後者が `コマンド名がありません`)。

ステートの誤り(終了コード 65): 6 種。`unreadable`, `tooLarge`, `corrupt`, `versionMismatch`, `writeFailed`, `removeFailed`。持つ情報は、パス、種類、原因、`detail`(任意)。表示は 08 章 4.5。

定義との食い違い(終了コード 65): 3 種。`pipelineName`, `jobIds`, `jobOrder`。表示は 08 章 4.6。

ステートの原因の文(固定の日本語。言語を切り替えない)と、それを返す種類(全 23 件。06 章の表と同じ)。

| 原因の文 | 種類 | 場面 |
|----------|------|------|
| `開く、または読み取ることができません` | `unreadable` | 読み込み、`reset --all` で読めなかった |
| `大きさが上限(1 MiB)を超えています` | `tooLarge` | 読み込み |
| `UTF-8 の文字列ではありません` | `corrupt` | 読み込み |
| `JSON として読めません` | `corrupt` | 読み込み |
| `キーが重複しています` | `corrupt` | 読み込み |
| `format_version がありません` | `corrupt` | 読み込み |
| `format_version が整数ではありません` | `corrupt` | 読み込み |
| `対応していない版です(reset が必要です)` | `versionMismatch` | 読み込み |
| `知らないキーがあります` | `corrupt` | 読み込み |
| `必須のキーがありません` | `corrupt` | 読み込み |
| `値の型が誤っています` | `corrupt` | 読み込み |
| `status の値が誤っています` | `corrupt` | 読み込み |
| `skip_reason の値が誤っています` | `corrupt` | 読み込み |
| `skip_reason が status と合いません` | `corrupt` | 読み込み |
| `exit_code が status と合いません` | `corrupt` | 読み込み |
| `ジョブIDが重複しています` | `corrupt` | 読み込み |
| `job の並びが job_order と一致しません` | `corrupt` | 読み込み |
| `resume_job_id が job_order にありません` | `corrupt` | 読み込み |
| `completed と resume_job_id が両立しません` | `corrupt` | 読み込み |
| `run_id がファイル名の run ID と一致しません` | `corrupt` | 読み込み(`reset --all` では「別の定義のステート」として扱い、消さない) |
| `書き込みに失敗しました` | `writeFailed` | 書き込み |
| `内容を JSON にできません` | `writeFailed` | 書き込み |
| `削除に失敗しました` | `removeFailed` | `reset`、`reset --all`、完了時の削除 |

読み込みの検査の順序と、各原因の判定条件は [06-state-file.md](06-state-file.md)に従う。複数の原因が当てはまるときにどれを出すかの順序は、この章では未確認(06 章に従う)。

### 7.3 変換の誤り(終了コード 67)

6 種: `fileMissing`, `unreadable`, `tooLarge`, `syntax`, `unrepresentable`, `rootNotMap`。持つ情報は、ファイル、行(あれば)、キーの位置(あれば)、種類、原因。表示は 08 章 4.8。

原因の文(固定の日本語。言語を切り替えない)。変換の規則と、どの場面で使うかは [09-converter.md](09-converter.md)(`.psd1` と JSON の相互変換)が規定する。**次の表は、09 章の全件と文言を完全に同じにする**(09 章が正。ここで違いが見つかったら、09 章に合わせてこの表を直す)。

| 種類 | 原因の文 | 場面 |
|------|----------|------|
| `fileMissing` | `指定の場所にファイルがありません` | 入力なし |
| `unreadable` | `読み取りの権限がありません` | 権限 |
| `unreadable` | `読み取りに失敗しました` | 読み取りの失敗 |
| `unreadable` | `通常のファイルではありません` | ディレクトリなど |
| `unreadable` | `UTF-8 として読めません` | 文字コード |
| `tooLarge` | `入力が上限(1 MiB)を超えています` | 入力の大きさ |
| `tooLarge` | `要素数が上限(100000)を超えています` | 要素数の上限 |
| `tooLarge` | `文字の大きさが上限(16 MiB)を超えています` | 文字の大きさの上限 |
| `syntax` | `.psd1 の文法に誤りがあります` | `.psd1` の文法(パーサーのエラー、前処理での拒否、文が 0 個、空の入力) |
| `syntax` | `.psd1 で使えない書き方です` | `.psd1` の、許可しない構文(コマンド、変数、式、演算子、スクリプトブロックなど) |
| `syntax` | `JSON の文法に誤りがあります` | JSON の文法(空の入力を含む) |
| `syntax` | `キーが重複しています` | キーの重複(`.psd1` は ASCII の大文字小文字を畳んで比べる) |
| `syntax` | `キーは文字列で書きます` | 文字列でないキー(`.psd1`) |
| `syntax` | `入れ子が深すぎます(上限は 32 段)` | 入れ子の深さ |
| `unrepresentable` | `数値が表せる範囲を超えています` | 64 ビット整数・浮動小数の範囲外 |
| `unrepresentable` | `.psd1 では、大文字小文字だけが違うキーを同時に書けません` | ASCII の英字の大文字小文字だけが違うキーを持つ JSON を `.psd1` へ(`--to psd1` のときだけ) |
| `rootNotMap` | `最上位がマップではありません` | 最上位がマップでない(`--to` のどちらでも) |

- `null` は、どちらの形式でも表せる値であり、変換の誤りにならない。日時らしい書式は文字列として読むので、変換の誤りにならない。表の理由は、この 17 件で全件である(09 章 10 章の一覧と同じ。理由が 09 章で増減したときは 09 章が正)。
- 空の入力(0 バイト、空白とコメントだけ)は `rootNotMap` ではなく `syntax`(09 章 4.3 節)。
- キーの長さの上限(1,024 文字)は、変換には無い(09 章 8.3 節)。

キーの位置の形式: キーは `.` で連結(各キーは 64 文字で切る)、配列の添字は `[i]`、文字列でないキーは `(非文字列キー)`。例: `job[2].transitions[0]`。

### 7.4 使い方の誤り(終了コード 68)

8 種: `UnknownCommand`, `MissingArgument`, `UnexpectedArgument`, `UnknownOption`, `OptionNotAllowed`, `MissingValue`, `InvalidValue`, `UnknownJob`。文言は 08 章 4.2。誤りは、種類と、対象(あれば)を持つ。対象は、誤りの対象の語(知らないコマンド、オプション名、ジョブID、`run ID (environment variable)` など)。入力の値は、それ以外には含めない(`--run-id` の値、環境変数の値は含めない)。

発生条件:

| 発生条件 | 種類 | 対象 |
|----------|------|------|
| 位置引数なし | `MissingArgument` | なし |
| コマンド名が未知(空文字も) | `UnknownCommand` | その語 |
| コマンド名が文字として正しくない(04 章 4.4.2) | `InvalidValue` | 読める形にした語(04 章 4.4.2) |
| 文法にないオプション、`--bogus=x` | `UnknownOption` | `--bogus`、`-x` など |
| そのコマンドで使えないオプション | `OptionNotAllowed` | `-v`, `--config`, `--to`, `--run-id`, `--all`, `--remove-on-complete` |
| 値のないオプション | `MissingValue` | `--config`, `--run-id`, `--to` |
| 重複、`--verbose=x` など、`--all` と `--run-id` の併用、`--run-id` の値の規則違反(文字として正しくない値を含む) | `InvalidValue` | オプション名(`-v`, `--config`, …、併用は `--all`) |
| `--to` の値が `psd1`/`json` 以外(文字として正しくない値を含む) | `InvalidValue` | その値(読める形) |
| ジョブID・パイプライン名が文字として正しくない | `InvalidValue` | 読める形にした語 |
| 値を取るオプションの直後が `-h`/`--help` | (値として取られるので help にならない。ほかに位置引数がなければ `MissingArgument`) | なし |
| 環境変数の run ID の規則違反 | `InvalidValue` | `run ID (environment variable)` |
| 余分な位置引数 | `UnexpectedArgument` | その語 |
| `convert` で `--to` なし | `MissingArgument` | `--to` |
| `run` のジョブIDが定義にない | `UnknownJob` | そのジョブID |

(環境変数の run ID が文字として正しくないときは、未設定と同じ扱いで、誤りにならない。04 章 5.1。)

### 7.5 シグナルの準備の誤り

2 種。「すでに登録済み」(成功と同じ扱い)、「失敗」(`conductor: error: failed to set up signal handling`、70)。

## 8. PowerShell でのエラーの設計

### 8.1 方針と、選択肢の比較

規定された誤りの伝え方には、3 つの選択肢がある。

| 案 | 内容 | 長所 | 短所 |
|----|------|------|------|
| A: すべて例外 | 規定された誤りを、`[System.Exception]` を継承した `class` の例外で `throw` する | 書き方が簡単。呼び出しの途中の層で、伝え方を意識しなくてよい | 定義の誤りは、複数の誤りをまとめて 1 行ずつ出す必要があり(03 章)、1 つの例外では 1 件しか持てない。確認(出典: about_Classes の制限): PowerShell の `class` は、モジュールを読み込んでも自動では取り込まれず、**ルートのモジュール(`.psm1`)にドットソースしたスクリプトで定義した `class` は、取り込みが一貫しない**。**`class` は、セッションの中で、再読み込み・アンロードができない**(`Import-Module -Force` で更新されない)。呼び出しの型が、署名に表れず、取りこぼしが、想定外の失敗(70)に化ける |
| B: すべて結果の値 | 値(成功か、規定された誤りか)で返し、例外は想定外の失敗だけ | 失敗が値に出る。複数の誤りを値として持てる。`catch` の取りこぼしがない | .NET の API(`IOException`、`Win32Exception`)が投げる例外を、そのつど値に変える必要があり、冗長になる |
| C: 混合(採用) | 規定された誤りは、ステートの読み書き、標準出力の書き込みの失敗を含め、すべて**値で返す**(`[pscustomobject]`)。.NET の境界の例外は、その場で狭く捕まえて値に変える。**`conductor` が独自の例外の型を定義して `throw` することはない**。想定外の失敗は、例外(任意の型)として伝え、`Invoke-Conductor` の最上位で 70 にする | 失敗が値に出る。複数の誤りを値として持てる。`class` の制限(上)を避けられる。`$ErrorActionPreference` が `throw` を抑える挙動(6.1 の 3)の影響を受けない | .NET の境界で、例外を値に変える記述が要る(境界の内側に閉じ込める) |

採用は C。規定された誤りと、想定外の内部失敗(6 章)を、**値と例外で区別する**: 規定された誤りは、8.3 の値の集合に属し、それ以外は(例外として)想定外とする。

- 各誤りの値は、「種別(文字列)」と「利用者に見える情報」を持つ。文言への変換(ロケール、エスケープ)は `Private/Report/` の責務で、例外の `Message` は、ロケールを判定せず、デバッグ用の固定の英語の 1 行にする(利用者向けの出力には使わない)。
- **種別は、`enum` にせず、文字列(7 章の識別子)で表す**。理由: 確認(出典: about_Classes): `enum` もモジュールの外へは自動で取り込まれず、アンロードもできない。ドットソースした複数のファイルの間での型の解決が、一貫しない(上の表)。種別の文字列は、7 章の識別子と同じ綴り(例: `'unsupportedFormat'`、`'UnknownCommand'`)にし、比較は、大文字小文字を区別する(`-ceq`、`switch -CaseSensitive -Exact`、または `[string]::Equals(..., [System.StringComparison]::Ordinal)`)。種別から文言への表は、`[System.Collections.Generic.Dictionary[string, string]]::new([System.StringComparer]::Ordinal)` に持つ(`@{}` のハッシュテーブルは、既定で大文字小文字を区別しない。08 章 10)。全種別が、全ロケールの文言の表にあることを、Pester のテストで検査する(08 章 10)。
- 終了コードは、値の種別から、`Private/Cli/` の 1 つの関数(`Get-ConductorExitCodeForError`。種別ごとの `switch`)で決める。終了コードの数値の定義は 1 か所(8.2)に集める。
- 呼び出し側は、値の判定(`$result.Ok`、`switch`)で扱う。**型を指定しない `catch` と、`catch { }`(空の本体)を、規定された誤りの処理に使わない**(想定外の失敗を握りつぶさないため)。想定外の失敗を捕捉してよいのは、`Invoke-Conductor` と入口スクリプトの最上位(6 章)と、08 章 1.3.2 の書き込みの失敗の処理(**列挙した型に限る**)だけ。.NET の境界での例外の変換(`IOException` 系、`UnauthorizedAccessException`、`Win32Exception` などを、その境界の規定された種類に対応づける。例: 定義ファイルの `FileNotFoundException`・`DirectoryNotFoundException` は `fileMissing`、`UnauthorizedAccessException` は `unreadable`)は、`catch` する型を列挙して行い、列挙にない例外は伝える(70 になる)。どの操作がどの型の例外を投げるかの詳細は、公式ドキュメントを取得しておらず、未確認(実装時に確かめ、確かめた対応を 03・06・07・09 章に書く)。
- 例外を値に変えるとき、下位の例外の文(`Exception.Message`)を、利用者向けの値に持たない(固定の理由だけ)。利用者向けの出力は、値の情報だけから作る。
- **終了エラーと非終了エラー**: 製品コードは、`Write-Error` と、コマンドレットの非終了エラーに頼らない(確認(出典: about_Automatic_Variables): `Write-Error` は、直後に `$?` を偽にするが、呼び出した関数の `$?` は変えない。`$ErrorActionPreference = 'Stop'` では、非終了エラーが終了エラーに格上げされる。08 章 1.5)。製品コードが使うコマンドレットは、`Stop` の下で、失敗が例外になるものだけ(`Join-Path` など、失敗しない純粋なもの)に限る。ファイルとプロセスの操作は、.NET の API で行う。
- **戻り値の汚染**: PowerShell の関数は、文の値をすべて出力に流す。値で返す関数は、`return` で 1 つの `[pscustomobject]` を返す。メソッドの戻り値を捨てるときは `[void]` か `$null =` を付ける(`StringBuilder.Append`、`List.Remove`、`HashSet.Add` など)。複数の誤りの配列は、値の `Errors` プロパティに入れて返し、関数の戻り値そのものを配列にしない(0 個・1 個の配列の展開を避ける)。
- 例外のクラスを定義しないので、`[Serializable]` や、例外の連鎖(`InnerException`)の規則は要らない。

### 8.2 定数

```powershell
# モジュール内の定数(読み取り専用の扱い。値を書き換えない)
$script:ExitCode = [pscustomobject]@{
    Ok = 0; UnlistedZero = 1; Definition = 64; State = 65; Rejected = 66
    Conversion = 67; Usage = 68; Internal = 70; StartFailure = 127; SignalBase = 128
}
```

- 値は、すべて `[int]`。`Main` に相当する入口スクリプトの `exit` には、`[int]` のまま渡す。
- 変数に `Constant` の属性を付けない(`Import-Module -Force` での再読み込みで、同じ名前の定数の再定義が誤りになりうる。確認していない挙動は未確認。値は書き換えないという規約と、テストで守る)。

結果の値(成功か規定された誤りか)は、次の形の `[pscustomobject]` で返す。

```powershell
function New-ConductorOk  { param($Value)  [pscustomobject]@{ Ok = $true;  Value = $Value; Error = $null } }
function New-ConductorErr { param($Error_) [pscustomobject]@{ Ok = $false; Value = $null;   Error = $Error_ } }
```

(骨格。名前と分け方は、成功と失敗を表せれば変えてよい。`$Error` は自動変数の名前なので、引数名に使わない。)

### 8.3 値の一覧(骨格)

ディレクトリ(関数の置き場所)ごとに、自分の誤りの値を作る関数を持つ(構成は 02 章)。値は、`Category`(下の表の「分類」)と `Kind`(種別の文字列)を持つ。

| 置き場所 | 分類 | 種別(文字列) | 終了コード | 持つ情報(伝え方) |
|----------|------|----------------|------------|----------|
| `Private/Definition/` | `Definition` | 7.1 の 31 種(03 章) | 64 | ファイル、行・列(`[Nullable[int]]` 相当。`$null` は「なし」)、種類、原因(日本語の固定の文)。複数の誤りは、`Errors` の配列で返す(03 章) |
| `Private/Runner/` | `Command` | `UnclosedQuote`, `Empty`(07 章) | 64 | 種類(ジョブIDは、呼び出し側が持つ)。値で返す(07 章) |
| `Private/RunId/` | `RunId` | `Empty`, `TooLong`, `InvalidChar`, `LeadingDot`(05 章) | 68(利用者には `invalid value`) | 種類のみ(値は持たない) |
| `Private/State/` | `State` | `unreadable`, `tooLarge`, `corrupt`, `versionMismatch`, `writeFailed`, `removeFailed`(06 章) | 65 | パス、種類、原因(固定の日本語の文)、`Detail`(任意)。ステートの読み書きの関数は、例外を送出せず、失敗の値を返す(06 章)。.NET の例外は、境界の内側で値に変える |
| `Private/State/` | `Consistency` | `pipelineName`, `jobIds`, `jobOrder`(06 章) | 65 | 種類。値で返す(06 章) |
| `Private/Convert/` | `Convert` | 7.3 の 6 種(09 章) | 67 | ファイル、行、キーの位置、種類、原因。値で返す(09 章) |
| `Private/Cli/` | `Usage` | 7.4 の 8 種 | 68 | 種類、対象(`[string]` または `$null`)。値として持ち回る(04 章) |
| `Private/Flow/` | `Reject` | `OutOfOrder` ほか(05 章) | 66 | ジョブID。起動前の判断が `Reject` の値で返し、`Cli` が 66 にする |
| `Private/Report/` | `Output` | (なし) | 呼び出し元による(`watch` は 65、`convert` は 67) | 標準出力への書き込みの失敗(08 章 1.3.2)。**例外にせず、`[bool]` の戻り値(偽が失敗)で返す** |
| `Private/Flow/` | (値にしない。ジョブの結果の値) | (なし。分類の結果) | 3 章から決まる | 報告する終了コード、エラーか |

- 種別は、すべて「値の集合が表で閉じている」。7 章の識別子と同じ綴り(大文字小文字を区別する)。
- 「対象なし」と「対象が空文字」を区別する(`unknown command: ''` のように、対象があれば `'...'` で囲んで出す)。`Target` として、`$null` を「対象なし」、`''` を「対象が空文字」とする。**判定は `$null -eq $Target`(`$null` を左辺に置く)**。`if ($Target)`・`[string]::IsNullOrEmpty($Target)` は、空文字を「なし」と誤るので使わない(確認していないが、PowerShell は `if` で空文字・`0`・空配列を偽とするのが一般に知られた挙動。未確認)。`$null` と比べるときは、`$null` を左辺に置く(右辺が配列のとき、`-eq` が要素の絞り込みになるため。未確認)。
- 使い方の誤りの値の例:

```powershell
# target: $null は「対象なし」、'' は「対象が空文字」。
[pscustomobject]@{ Category = 'Usage'; Kind = 'InvalidValue'; Target = '-v' }
```

### 8.4 終了コードへの対応

`Private/Cli/` の 1 か所の関数(`Get-ConductorExitCodeForError`)が、すべての規定された誤りの値から、`[int]` の終了コードを決める(`Category` と `Kind` ごとの `switch`。8.3 の表の「終了コード」の列)。標準出力の書き込みの失敗(`[bool]` が偽)は、呼び出し元(`watch` か `convert`)が、`$script:ExitCode.State` か `$script:ExitCode.Conversion` を決める。その他の例外は、この関数に届かず、`Invoke-Conductor` の最上位の `catch` が 70 にする(6 章)。

## 9. 必須の挙動と、実装に任せてよい点

必須:

- 1 章の終了コードの全件、2 章の決定順序、3 章の分類、4 章の範囲合わせ(判定表)、入口スクリプトの `exit` に 0〜255 の `[int]` を渡すこと(4.1)。
- 6 章: `Invoke-Conductor` と入口スクリプトの `try`/`catch`、`$ErrorActionPreference = 'Stop'` と `Set-StrictMode -Version Latest`、`conductor: error: internal error` の 1 行(固定の英語。標準エラー)と 70、入力に由来する入れ子を再帰で処理しないこと(6.3)、`finally` での後始末(6.4)。
- 7 章の分類と、8 章の値と例外の区別(規定された誤りは値、想定外の失敗は例外。型を指定しない `catch` での握りつぶしをしない)。

実装に任せてよい点:

- 内部の値の名前と分け方(8 章は目安)。
- 失敗の詳細の保存や、標準エラーへの追加の出力(6.1 の 5)。

## 10. 未確認

- `pwsh -File` が、`exit` の値を、そのままプロセスの終了コードにすること。ホストの `$global:?` の検査が、`exit` の値を上書きしないこと(4.1)。UNIX 系で、範囲外の値が実際に 0 になること(ドキュメントの記述のみ。実機で確認していない)。Windows での 255 を超える値の扱い。
- `pwsh -File` が、スクリプトの外の失敗(構文の誤り、入口スクリプトの `try` の外の例外)で返す終了コード(1.)。
- `$ErrorActionPreference = 'Stop'` が、内側の関数の .NET メソッドの例外を、外側の `try`/`catch` まで確実に伝えること。モジュールの関数が、呼び出し元の `$ErrorActionPreference` を引き継ぐか(6.1 の 3。試験 T8)。
- パイプラインの停止の例外(`PipelineStoppedException`)が、`catch` で捕捉できるか、`finally` だけが実行されるか。Ctrl+C の終了コード(`run`・`pipeline` で 130 にできるか)。`pwsh` 自身の Ctrl+C の処理を、07 章の方式の登録で止められるか。`finally` の中の `exit` が終了コードを決められるか(5 章、6.4。試験 T4)。
- シグナルの準備をしないコマンド(`watch`・`reset`・`validate`・`convert`)の実行中の SIGTERM などの終了コード(5 章)。
- スタックの溢れで、プロセスが終わるときの終了コード。PowerShell の関数の再帰の深さの上限(6.3)。
- 子プロセスの起動の失敗で `Process.Start` が投げる例外の型の全件と、`Process.ExitCode` の、シグナルで終了した子に対する値(07 章が規定する。3 章)。
- `-band 0xFF` が負の値で 2 の補数の下位 8 ビットを返すこと、`%`(剰余)が負の被除数で返す値(4 章。05 章 2.2 節。切り替え先の式は、どちらの実装でも同じ結果になる)。
- 「if」文の真偽の判定(空文字・`0`・空配列)と、`-eq` が配列に対して要素を絞り込む挙動(8.3)。この仕様は、それらに頼らない書き方を規定している。
- `Constant` の属性の変数の、`Import-Module -Force` での再定義の挙動(8.2)。
- OS の境界で、どの操作がどの例外の型を投げるか(8.1)。
- 変換の誤りの原因の文(7.3)が、09 章の最終版と一致すること(09 章が正)。
- サポートする最小の PowerShell の版(実装時に確認する)で、4・5・6 章の確認を、3 OS で繰り返すこと。
