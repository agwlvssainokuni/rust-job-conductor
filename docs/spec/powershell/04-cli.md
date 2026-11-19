# 04. コマンドライン

この章は、PowerShell 7.x(`pwsh`)で実装する `conductor`(版 0.8.0)のコマンドラインを規定する。引数の解釈は、PowerShell のパラメータ束縛にも、引数解析用のモジュールにも任せず、字句の段階(4.1)と検査の段階(4.2)を**自前の手順**で実装する。メッセージの全文は [08-output-and-i18n.md](08-output-and-i18n.md)、終了コードとエラーの設計は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md)、各コマンドの内部の流れは [05-execution-flow.md](05-execution-flow.md)、定義ファイルの書式は [03-definition-format.md](03-definition-format.md)、変換は [09-converter.md](09-converter.md)、シグナルと子プロセスは [07-process-and-signals.md](07-process-and-signals.md)、全体構成は [02-architecture.md](02-architecture.md) を参照する。

この文書で「確認(出典: …)」と書いた事項は、公式ドキュメント(learn.microsoft.com の PowerShell 7.4 の about_* と .NET API リファレンス)、または PowerShell の公式リポジトリのソースを取得して確かめた内容である。「未確認」と書いた事項は、この環境に `pwsh` も .NET SDK もなく実機で動かせないこと、または出典を取得できなかったことで、確かめられていない。**未確認の事項は、実装の最初に 11 章の「最初に確かめる試験」で確かめ、結果に応じて本文を直す。**

## 1. 全体像

### 1.1 入口の構成

| 層 | ファイル | 役割 |
|----|----------|------|
| ランチャー(UNIX 系) | `bin/conductor`(POSIX の `sh` スクリプト) | `pwsh` を `-NoLogo -NoProfile -NonInteractive -File` で起動し、利用者の引数を**変えずに**渡し、`pwsh` の終了コードで終わる(1.3) |
| ランチャー(Windows) | `bin/conductor.cmd` | 同上(1.3) |
| 入口スクリプト | `bin/conductor.ps1` | モジュールを読み込み、生の引数を取り出し(1.2)、`Invoke-Conductor` を呼び、戻り値を `exit` に渡す。**`exit` を呼ぶのはこのスクリプトだけ**(と、停止経路の `[System.Environment]::Exit`。07 章 7.7 節) |
| 公開関数 | `Invoke-Conductor`(モジュール `Conductor`) | 引数の配列を受け、`[int]` の終了コードを**返す**(1.4)。`exit` を呼ばない(呼ぶと、対話セッションを丸ごと終了させるため) |

(ランチャーは、`pwsh` に `-NoProfile` などを確実に渡すため、UNIX 系を `sh` スクリプトにし、`pwsh` が実行する入口を `bin/conductor.ps1` として分けた。理由は 1.3。)

### 1.2 共通の規則

- サブコマンドは 6 つだけ: `run`, `pipeline`, `watch`, `reset`, `validate`, `convert`。
- 引数はシェルを介さず、`conductor` 自身は展開(ワイルドカード、変数、チルダなど)も解釈もしない。
- 特別なオプション(`-h`/`--help`, `-V`/`--version`)は、コマンドの解釈より先に処理する(4.2 の 1)。
- 使い方の誤りのときは、標準エラーに「誤りの 1 行 + 空行 + 使い方全文」を書き、終了コード 68 で終わる。ジョブは実行しない。
- 既定では、`conductor` 自身は何も出力しない(ジョブ本体の出力がそのまま流れる)。エラーのとき、または `-v` のときだけ標準エラーに書く。例外として、`watch`・`--help`・`--version`・`convert` は結果を標準出力に書く。
- プロセスの終了コードを決める箇所は、入口スクリプトの `exit` の 1 か所だけにする(停止経路の `Environment.Exit` を除く。07 章 7.7 節)。`Invoke-Conductor` は `[int]` を返し、入口スクリプトが、出力を完了させたうえで(08 章 1.4)その値を `exit` に渡す(10 章 4.1)。
- 引数は、入口スクリプトが `[System.Environment]::GetCommandLineArgs()` から取り出した**生の文字列の配列**(`[string[]]`)として `Invoke-Conductor` に渡す。取り出しの手順は 1.5。

### 1.3 ランチャーの仕様

#### 1.3.1 共通の方針

確認(出典: about_Pwsh, PowerShell 7.4):

| 事項 | 内容 |
|------|------|
| `-File` | 「`File` must be the last parameter in the command, because all characters typed after the File parameter name are interpreted as the script filepath followed by the script parameters」。`-File` の後ろはすべて、スクリプトのパスと、スクリプトへの引数として解釈される |
| 引数の渡し方 | 「Parameters passed to the script are passed as literal strings, after interpretation by the current shell」。つまり、`pwsh` を起動する側のシェル(`cmd.exe` など)が解釈したあとの文字列が、そのまま渡る。`cmd.exe` では `%windir%` が展開され、`$Env:windir` は展開されない |
| `-NoProfile` | 「Doesn't load the PowerShell profiles」 |
| `-NonInteractive` | 「Any attempts to use interactive features, like `Read-Host` or confirmation prompts, result in statement terminating errors rather than hanging」 |
| `-NoLogo` | 「Hides the banner at startup of interactive sessions」 |
| `-ExecutionPolicy` | 「This parameter only applies to Windows computers. On non-Windows platforms, the parameter and the value provided are ignored」(UNIX 系では実行ポリシーが適用されない) |
| `-File` の終了コード | 「When the script file terminates with an `exit` command, the process exit code is set to the numeric argument used with the `exit` command」。`exit` がなく正常に終われば 0、スクリプトが例外で終わるときは 1。「unlike with `-Command`, when the execution is interrupted with Ctrl+C the exit code is `0`」(`-File` では Ctrl+C で中断されたとき、終了コードは 0) |
| 非 Windows での `-File` | 「On other platforms, PowerShell attempts to run other file types」(Windows の 7.2 以降は `.ps1` だけ) |

確認(出典: PowerShell のソース `ConsoleHost.cs`): `-File` のファイルが `.ps1` で終わらないとき(UNIX 系)は、ファイルの中身を読んでスクリプトとして実行する(シェバンを使える)。`.ps1` で終わるときは、パスをそのままコマンドとして実行する。`-File` のスクリプトの中に `$input` が現れると(構文木の走査で検出)、ホストは標準入力を `$input` のために読む設定にする。**したがって、`bin/conductor.ps1` と、それが読み込むファイルでは `$input` を使わない**(子プロセスが標準入力を引き継ぐ。07 章)。

確認(出典: 同、`ConsoleHost.cs` の定数): `pwsh` 自身が、起動の失敗に終了コード 64(`ExitCodeBadCommandLineParameter`)と 70(`ExitCodeInitFailure`)を使う。これは `conductor` の終了コード 64(定義の誤り)・70(内部の誤り)と**同じ数値**である。ランチャーが `pwsh` を起動できても、`pwsh` 自身の起動の失敗(コマンドラインの誤りなど)は、`conductor` の誤りと区別できない。ランチャーは、固定の引数だけを前に置く(1.3.2・1.3.3)ので、64 は通常は現れない。

#### 1.3.2 `bin/conductor`(UNIX 系。`sh`)

先頭は `#!/bin/sh`。`#!/usr/bin/env pwsh` にしない理由: シェバンの行に `-NoProfile` などの引数を足す移植性のある方法がない(1 つの引数にまとめられる、など。実装系で挙動が違う。ここでは取得した出典がなく、未確認)ため、`sh` から `pwsh` を `exec` する。

手順(骨格。実装の細部は任せる):

1. 自分のディレクトリを求める。`$0` がシンボリックリンクなら、`readlink`(オプションなし)で、リンクの連鎖をたどる(`readlink -f` は使わない。古い macOS にないため。移植性は未確認)。
2. `pwsh` が `PATH` にあるかを `command -v pwsh` で確かめる。なければ、標準エラーに `conductor: error: pwsh (PowerShell 7) was not found in PATH` と LF を書き、終了コード 70 で終わる(固定の英語。ロケールを判定しない)。
3. 環境変数 `POWERSHELL_UPDATECHECK=Off` と `POWERSHELL_TELEMETRY_OPTOUT=1` を `export` する。確認(出典: about_Environment_Variables): 更新通知は `POWERSHELL_UPDATECHECK` の `Off` で止まり、テレメトリは `POWERSHELL_TELEMETRY_OPTOUT` が `true`・`yes`・`1` で止まる。いずれも「`pwsh` のプロセスを起動する前に設定しなければ効かない」。**これらは、ジョブ(子プロセス)にも引き継がれる。仕様として受け入れる**(`conductor` は環境を加工しない。07 章)。更新通知が、非対話の `-File` の実行でも出るかは未確認(出さないための保険)。
4. `exec pwsh -NoLogo -NoProfile -NonInteractive -File "<自分のディレクトリ>/conductor.ps1" "$@"`。**利用者の引数は、`-File` とスクリプトのパスのあとにだけ、`"$@"` で、1 つも変えず、足さず、並べ替えずに渡す。** 利用者の引数を、`pwsh` のオプション(`-NoProfile` など)の前に置かない。確認(出典: `CommandLineParameterParser.cs`): `pwsh` は、`-File` より前の `-v`(`-Version`)や `-h` を自分のオプションとして解釈する。
5. `exec` するので、`pwsh` の終了コードが、そのままランチャーの終了コードになる(ランチャーは、終了コードを加工しない)。

シグナル: UNIX 系で、端末の Ctrl+C は、前景のプロセスグループ全体(`sh` から `exec` した `pwsh`)に届く。`exec` で `sh` が置き換わるので、ランチャーが、シグナルを中継する層にならない。

#### 1.3.3 `bin/conductor.cmd`(Windows)

骨格:

```bat
@echo off
setlocal DisableDelayedExpansion
where pwsh >nul 2>nul
if errorlevel 1 (
  echo conductor: error: pwsh (PowerShell 7) was not found in PATH 1>&2
  exit /b 70
)
set "POWERSHELL_UPDATECHECK=Off"
set "POWERSHELL_TELEMETRY_OPTOUT=1"
pwsh -NoLogo -NoProfile -NonInteractive -File "%~dp0conductor.ps1" %*
set "CONDUCTOR_RC=%ERRORLEVEL%"
endlocal & exit /b %CONDUCTOR_RC%
```

規定:

- `-File` の値は、`"%~dp0conductor.ps1"`(バッチファイルのあるディレクトリからの絶対パス)にする。確認(出典: about_Pwsh): `.\test.ps1` のような相対パスは、バッチから実行すると見つからずに誤りになるので、`%~dp0` を使う。
- 利用者の引数は `%*` で、1 つも変えずに渡す。`%*` は、バッチに渡された引数の文字列全体(引用符を含む)に展開される(一般的なバッチの仕様。出典のページは取得しておらず、未確認)。
- `exit /b %ERRORLEVEL%` 相当で、`pwsh` の終了コードを伝える。`endlocal` の前に値を変数に取る形(上の骨格)にして、`endlocal` が終了コードを消さないようにする。
- `setlocal DisableDelayedExpansion` で、`!` を含む引数が遅延展開されないようにする。

Windows の `cmd.exe` での、引数の扱い(**利用者向けの注意として README に書く**):

| 文字 | `conductor.cmd` を使うときの扱い |
|------|----------------------------------|
| 空白を含む引数 | 二重引用符で囲む(`"a b"`)。`.NET` が、引数に分けるとき、二重引用符で囲まれた空白を引数に含める(確認(出典: `Environment.GetCommandLineArgs` の Remarks)。ただし `pwsh` が、自分のコマンドラインを同じ規則で分けるかは未確認) |
| `%` | `cmd.exe` が、`%名前%` を環境変数に展開する(確認(出典: about_Pwsh): `pwsh -File .\test.ps1 -TestParam %windir%` で値が展開される)。定義されていない名前は、そのまま残る(一般的な `cmd.exe` の挙動。未確認) |
| `&`、`\|`、`<`、`>`、`^` | 引用符の外にあると、`cmd.exe` が、コマンドの区切り・リダイレクト・エスケープとして解釈する。**引数にこれらを含めるときは、二重引用符で囲む**(例: `conductor run "a&b"`)。`%*` は引用符を保つので、引用符の中では文字そのものとして渡る(一般的な挙動。未確認)。引用符の外に置いた場合、バッチの行の中で `%*` が展開されたあとに再解釈され、意図しないコマンドが実行されうる。**この危険は、ランチャーでは防げない**(`.cmd` の仕組み上の制限)ので、README に明記する。ジョブID・パイプライン名・run ID は文字種が限られており、通常は問題にならない。危険が現れるのは `--config` のパスと `convert` の入力ファイル |
| 二重引用符そのものを含む引数 | 受け付ける保証はない。バックスラッシュと二重引用符の組の規則は 4.4.1。確認(出典: about_Parsing): バッチファイルへの引数は `cmd.exe` に生のコマンドライン文字列として渡され、「untrusted input should be passed another way」(信頼できない入力は別の方法で渡す) |
| 空文字の引数 `""` | `args` に空文字の要素として入るかは未確認(11 章)。`--run-id=`(値を `=` の後ろに書く形)を使えば避けられる |

Ctrl+C: `cmd.exe` は、バッチの実行中の Ctrl+C で「バッチ ジョブを終了しますか」と確認することがある(一般に知られた挙動。取得した出典はなく、未確認)。確認の答えによっては、`pwsh` の終了コードを伝えずにバッチが終わりうる。**この制限は、README に書く**(Windows の Ctrl+C の終了コードは、ランチャー経由では保証しない。10 章 5)。

#### 1.3.4 `bin/conductor.ps1`(入口スクリプト)

- 先頭に `param()` を書かない。`[CmdletBinding()]` を付けない。`$input` を使わない(1.3.1)。確認(出典: about_CommonParameters): 共通パラメータ(`-Verbose` など)が自動で足されるのは、コマンドレットと、`CmdletBinding` か `Parameter` の属性を使う高度な関数だけである。入口スクリプトは、パラメータを一切宣言しない。
- 骨格(実装コードは規定しない。順序と意味を規定する):

```powershell
#requires -Version 7.6            # PowerShellVersion と同じ値(11 章 2.5 節。版の一致の検査)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$exitCode = 70
$finished = $false
try {
    # 1. モジュールを読み込む(リリース: ../Conductor/Conductor.psd1、ソース: ../src/Conductor/Conductor.psd1)
    # 2. 生の引数を取り出す(1.5)
    # 3. $result = @(Invoke-Conductor -ArgumentList $rawArguments)
    #    $result が [int] 1 つで 0〜255 でなければ内部の誤り(10 章 4.1。固定の 1 行を出して 70)
    #    それ以外は $exitCode = $result[0]
    $finished = $true
}
catch [System.Management.Automation.PipelineStoppedException] {
    # Ctrl+C による停止。内部の誤りの 1 行を出さない(10 章 6.4)。finally で終了コードを決める
    throw
}
catch {
    # 想定外の失敗(モジュールの読み込みの失敗を含む)。10 章 6.1 の固定の 1 行を標準エラーに書き、70
    $exitCode = 70
    $finished = $true
}
finally {
    if (-not $finished) {
        # 停止経路(07 章 7.7 節): ヘルパーが予約した終了コード(HasPendingExitCode が真なら PendingExitCode。
        # なければ 130)で [System.Environment]::Exit を呼ぶ。出力の flush は Invoke-Conductor の finally が済ませている
    }
}
exit $exitCode
```


- モジュールの探索: 入口スクリプトのあるディレクトリ(`$PSScriptRoot`。確認(出典: about_Automatic_Variables): 「Contains the full path of the executing script's parent directory」。PowerShell 3.0 以降、すべてのスクリプトで有効)から、リリースの配置(`../Conductor/Conductor.psd1`)、なければソースの配置(`../src/Conductor/Conductor.psd1`)の順に探す。見つからなければ、固定の 1 行を出して 70(10 章 6)。
- `exit` に渡す値は `[int]`(0〜255。10 章 4)。

### 1.4 `Invoke-Conductor` の署名

```powershell
function Invoke-Conductor {
    [OutputType([int])]
    param(
        [Parameter(Position = 0)]
        [AllowNull()]
        [AllowEmptyCollection()]
        [AllowEmptyString()]
        [string[]] $ArgumentList = @()
    )
    # 戻り値は、0〜255 の [int] を 1 つだけ(return で明示)
}
```

規定:

- **`[CmdletBinding()]` を付けない。** 付けると、共通パラメータ(`-Verbose` など)が足され、対話セッションで `Invoke-Conductor -v` と書いたとき、`-v` が `-Verbose` の省略形として束縛されうる(確認(出典: about_CommonParameters): `Verbose` の別名は `vb` だが、共通パラメータは、名前の先頭の一部だけでも束縛される通常のパラメータ名の規則に従うかは、取得したページに記述がなく未確認。付けないことで、この問題を避ける)。
- 引数は、必ず `-ArgumentList` に配列(`[string[]]`)で渡す。呼び出しの例: `Invoke-Conductor -ArgumentList @('run', '-v', 'job1')`、引数なしは `Invoke-Conductor -ArgumentList @()`。配列の要素は、パラメータとして解釈されない(値として渡る)。
- 対話セッションで `Invoke-Conductor run -v job1` のように書くと、`run`・`-v`・`job1` は、宣言されていないパラメータとして `$args` に入る(確認(出典: about_Automatic_Variables): 「`$args` Contains an array of values for undeclared parameters that are passed to a function, script, or scriptblock」)。`$args` の中身が、`-v` や `--` を含む引数をそのまま保つかは未確認。したがって、この関数は `$args` を**使わない**。`$args` が空でないとき(呼び出し側の誤り)は、内部の誤りとして、固定の 1 行を出して 70 を返す。
- 戻り値は、`return` で明示した `[int]` 1 つだけ。関数の中で、パイプラインに値が漏れる文(戻り値を捨てていないメソッド呼び出しなど)を作らない(02 章 9.2 節)。公開関数は、内部の本体の出力をすべて配列に集め(`@(...)`)、要素が `[int]` 1 つだけであることを確かめる。そうでなければ、バグとして、固定の 1 行を出して 70 を返す。
- `Invoke-Conductor` 全体を `try`/`catch` で囲む(10 章 6.1)。`exit` を呼ばない。
- 作業ディレクトリ: 確認(出典: about_Automatic_Variables, `$PWD`): 「Each runspace has its own *current directory*. This isn't the same as the current directory of the process: `[System.Environment]::CurrentDirectory`」。.NET のファイル API(`[System.IO.File]` など)は、相対パスを、プロセスのカレントディレクトリから解決する。`pwsh -File` で起動された直後は、両者が一致する(未確認)。対話セッションから呼ばれたときのため、`Invoke-Conductor` は、開始時に `$PWD` が `FileSystem` プロバイダーの場所なら、`[System.Environment]::CurrentDirectory` を `$PWD.ProviderPath` に設定し、`finally` で元に戻す。設定が失敗したら、内部の誤り(70)にする。`Set-Location` を製品コードで使わない。

### 1.5 生の引数の取り出し(入口スクリプト)

`pwsh -File` に渡した引数は、スクリプトに届く前に、PowerShell のパラメータの形に変換される。確認(出典: `CommandLineParameterParser.cs`、`CollectPSArgs`):

| 入力の形 | 変換 |
|----------|------|
| `-` で始まり、長さが 2 以上で、`:` を含む(例 `-x:1`、`--config=C:\a.psd1`) | `:` で名前と値に分けられる。末尾が `:` のときは、次の引数が値になる |
| `-` で始まり、長さが 2 以上で、`:` を含まない(例 `-v`、`--help`、`--`、`--config=x`) | 値のない名前つきのパラメータ(スイッチ)になる |
| それ以外(`-` だけ、`-` で始まらない文字列) | 位置引数の値になる |

この変換では、`--config=C:\work\a.psd1` が、名前 `--config=C` と値 `\work\a.psd1` に分かれうる(Windows の絶対パスは `:` を含むので、現実の入力で起きる)。変換後の `$args` で、元の文字列を取り戻せる保証はない(未確認)。

したがって、この仕様は、`$args` を使わず、**`pwsh` プロセスが OS から受け取った引数の配列**を使う。

- 入口スクリプトは、`[System.Environment]::GetCommandLineArgs()` を呼ぶ。確認(出典: `Environment.GetCommandLineArgs` のリファレンス): 最初の要素は実行中のプログラムのファイル名で、以降の要素がコマンドラインの引数。
- その配列で、**最初に現れる `-File` に等しい要素**(`[string]::Equals(..., [System.StringComparison]::OrdinalIgnoreCase)`)を探し、その次の要素(スクリプトのパス)を飛ばして、**その後ろのすべて**を、生の引数とする。ランチャーが、`-File` の前に固定の引数だけを置く(1.3)ので、最初の `-File` は、必ずランチャーが置いたものである。利用者の引数に `-File` という値があっても、その前にランチャーの `-File` がある。
- `-File` が見つからないとき(ランチャーを経由しない起動)は、`$args` を、そのまま `[string[]]` にして使う(制限つき。上の変換の影響を受ける。サポート外)。
- `GetCommandLineArgs()` が、`pwsh` の受け取った引数を、変換や欠落なしに返すこと、ホストの実行ファイル(`pwsh` の起動の仕組み)による先頭の要素の違い(実行ファイルの名前か、`pwsh.dll` のパスか)は未確認。**11 章の試験 T1 で、最初に確かめる。** 期待どおりでなければ、この節の方式を、確かめた事実に合わせて直す(候補: 各引数に、`-` で始まらない 1 文字の接頭辞をランチャーが付け、入口スクリプトが外す方式。UNIX 系は `sh` の `for` で、Windows は `shift` の繰り返しで実装できるが、Windows では引数に含まれる二重引用符・`%` の扱いが別に難しい)。

### 1.6 PowerShell のパラメータ束縛と、この仕様の文法

確認できた範囲を表にする。出典は、about_Pwsh、about_Parsing、about_Automatic_Variables、about_CommonParameters(いずれも PowerShell 7.4)、`CommandLineParameterParser.cs`、`ConsoleHost.cs`。「`conductor` の入力」は、利用者が `conductor` に打つ引数。

| 入力 | PowerShell の挙動(確認できた範囲) | この仕様の対処 |
|------|--------------------------------------|----------------|
| `-v`(`conductor` の短縮形) | `pwsh` の `-v` は `-Version`。ただし、`-File` の後ろでは `pwsh` のオプションではなく、スクリプトの引数(確認: `-File` の後ろは全部スクリプトの引数)。スクリプトに宣言がなければ共通パラメータの `-Verbose` にならない(確認: about_CommonParameters。`CmdletBinding` なしの入口スクリプト) | 入口スクリプトは `param()` を持たない。生の引数の取り出し(1.5)で、変換を通らない |
| `-h` | `pwsh` の `-h` は `-Help`(確認: `MatchSwitch(switchKey, "help", "h")`)。`-File` の後ろでは、スクリプトの引数 | 同上。ランチャーは、利用者の引数を、`-File <パス>` の後ろにだけ置く(1.3) |
| `-vh`、`-vV`(連結) | 1 つの `-` つきの語として、スイッチ 1 つに変換される(確認: `CollectPSArgs`)。`$args` の中の形は未確認 | 生の引数をそのまま字句の段階(4.1)に渡す |
| `--opt=value`(`:` を含まない) | `-` で始まる語なので、名前つきのスイッチに変換される(確認: `CollectPSArgs`)。`$args` で `--opt=value` の形のまま届くかは未確認 | 同上 |
| `--config=C:\a.psd1`(`:` を含む) | `:` で名前と値に分けられる(確認: `CollectPSArgs`) | 同上。`$args` を使わない理由の 1 つ |
| `--`(単独) | `CollectPSArgs` には `--` の特別な処理がなく、長さが 1 より大きいので、`--` という名前のスイッチとして格納される(確認)。about_Parsing の「end-of-parameters token (`--`)」は、PowerShell のコマンド(コマンドレットなど)を名前で呼ぶときの構文であり、`pwsh -File` の引数の経路とは別 | 同上 |
| `-`(単独) | 長さが 1 なので、位置引数になる(確認: `arg.Length > 1` の検査) | 4.1 の 5 のとおり位置引数 |
| `--foo`(値を取る長いオプションの値が `-` で始まる) | `--config --help` の `--help` は、`pwsh` から見ると別のスイッチ(確認: 各語が独立に変換される)。`$args` で「値」として結び付けられる保証はない | 4.1 の 3 のとおり、値を取るオプションの次の語は、中身にかかわらず値。生の配列を、語の並びのまま字句の段階に渡す |
| `-Verbose`、`-Debug`、`-ErrorAction` など | 共通パラメータは、`CmdletBinding` などを使う高度な関数にだけ足される(確認: about_CommonParameters)。入口スクリプトと `Invoke-Conductor` は使わない(1.4) | `conductor` は、`-Verbose` などを知らないオプション(`unknown option: '-V'` など。`-V` は version)として扱う。`-Verbose` は、連結の分解で `-V`(version)が先に現れ、version になる |
| 空文字の引数 | 位置引数として、空文字の値になるか未確認 | `--run-id=` の形を案内する(3.3) |
| `$` や `%` を含む引数 | `pwsh -File` の引数は「literal strings」で、PowerShell は展開しない(確認: about_Pwsh)。`cmd.exe` は `%名前%` を展開する(確認: 同) | `conductor` は展開しない(1.2)。Windows の `%` は 1.3.3 |
| `pwsh -Command`・`-CommandWithArgs` | `-Command` は、後続のすべてを PowerShell のコードの一部として解釈する(確認: about_Pwsh)。`-CommandWithArgs` は 7.4 の実験的な機能(確認: 同) | **使わない** |
| `pwsh -File` の終了コード | `exit N` の `N`。例外で終われば 1。Ctrl+C では 0(確認: about_Pwsh、about_Language_Keywords) | 10 章 4・5 |

要するに、(1) `pwsh -File` の引数は、PowerShell のパラメータの形に変換される際に、`:` を含む語が分割され、`-` で始まる語がスイッチ名になり、`--` が特別扱いされない。元の文字列が `$args` に残る保証がない。(2) したがって、**`$args` と PowerShell のパラメータ束縛に頼らず、`pwsh` が OS から受け取った引数の配列を、入口スクリプトで直接取り出す**(1.5)。(3) 入口スクリプトと `Invoke-Conductor` は、共通パラメータが付かない形にする。(4) `Invoke-Conductor` には、必ず配列を `-ArgumentList` で渡す(1.4)。

## 2. 構文

```text
conductor run [-v] [--config <path>] [--run-id <id>] [--remove-on-complete] <job-id>
conductor pipeline [-v] [--config <path>] [--run-id <id>] [--remove-on-complete] <pipeline-name>
conductor watch [--config <path>] [--run-id <id>]
conductor reset [--config <path>] [--run-id <id> | --all]
conductor validate [-v] [--config <path>] [<pipeline-name>]
conductor convert <file> --to <psd1|json>
conductor --help | --version
```

## 3. オプションと位置引数

### 3.1 オプションの一覧と、コマンドごとの可否

| オプション | 短縮形 | 値 | run | pipeline | watch | reset | validate | convert |
|------------|--------|----|-----|----------|-------|-------|----------|---------|
| `--verbose` | `-v` | なし | 可 | 可 | 不可 | 不可 | 可 | 不可 |
| `--config` | なし | パス(必須) | 可 | 可 | 可 | 可 | 可 | 不可 |
| `--run-id` | なし | 文字列(空も可) | 可 | 可 | 可 | 可 | 不可 | 不可 |
| `--remove-on-complete` | なし | なし | 可 | 可 | 不可 | 不可 | 不可 | 不可 |
| `--all` | なし | なし | 不可 | 不可 | 不可 | 可 | 不可 | 不可 |
| `--to` | なし | `psd1` か `json`(必須) | 不可 | 不可 | 不可 | 不可 | 不可 | 必須 |
| `--help` | `-h` | なし | どのコマンドでも(コマンド名がなくても)受け付ける |
| `--version` | `-V` | なし | どのコマンドでも(コマンド名がなくても)受け付ける |

- 長いオプション名・短縮形は、大文字小文字を区別する(`-v` は verbose、`-V` は version。`-Verbose` は、連結として `-V`、`-e`、… に分解される。4.1 の 4)。長いオプションの前方一致による省略(`--verb` など)は受け付けない(知らないオプション)。
- 上表以外の短縮形は存在しない(`--config` に `-c` はない)。
- Windows の `/h` や `/?` の形は、受け付けない(位置引数として扱う。`/` で始まる引数に特別な意味はない)。`pwsh` 自身は `/?` を `-Help` として解釈する(確認: `GetSwitchKey` が `-` と `/` の両方を除く)が、それは `-File` より前の位置の話であり、`conductor` の引数は `-File` の後ろにある(1.3)。

### 3.2 位置引数

「コマンド名」は、最初に現れた位置引数である。位置引数の個数は、コマンド名を除いて数える。

| コマンド | 位置引数 | 個数 |
|----------|----------|------|
| `run` | ジョブID | ちょうど 1 |
| `pipeline` | パイプライン名 | ちょうど 1 |
| `watch` | なし | 0(渡すと `unexpected argument`) |
| `reset` | なし | 0(渡すと `unexpected argument`) |
| `validate` | パイプライン名 | 0 または 1(省略時は名前を照合しない) |
| `convert` | 入力ファイル | ちょうど 1(`--to` は別に必須) |

オプションは、コマンド名の前にも後ろにも置ける(例: `conductor --config c.psd1 -v run x` は有効)。

### 3.3 各オプションの意味と既定値

| オプション | 意味 | 既定 |
|------------|------|------|
| `-v`, `--verbose` | ジョブの結果・スキップ・遷移・中断を標準エラーに出す(`validate` は成功時に 1 行)。段階は 1 つだけ(詳しさの度合いを選べない)。PowerShell の `-Verbose` 共通パラメータとは無関係 | 出さない |
| `--config <path>` | 定義ファイルのパス。`[string]` のまま加工せず、ファイルを開く API(`[System.IO.File]::OpenRead`、`[System.IO.FileStream]::new`、`[System.IO.File]::Exists` など)にそのまま渡す(`Resolve-Path`、`Join-Path`、`[System.IO.Path]::GetFullPath` などで正規化・絶対化した文字列を、利用者に見せる文字列(誤りの行の場所、ステートのパスの元)にしない。相対パスは、プロセスのカレントディレクトリから解決される(1.4)。書式は拡張子で決まる(`.psd1` は PowerShell データファイル、`.json` は JSON。大文字小文字を区別しない。それ以外は拒否。[03-definition-format.md](03-definition-format.md)) | 実行ディレクトリ(カレントディレクトリ)の `conductor.psd1` |
| `--run-id <id>` | ステートファイルを分ける run ID。空文字は「run ID なしに固定」(環境変数を見ない) | 未指定(環境変数を見る。5 章) |
| `--remove-on-complete` | 完了時にステートファイルを削除する | 指定なし(その場合は定義の `pipeline.remove_on_complete`、それも無ければ `false`) |
| `--all` | 定義のステートファイルを、run ID の有無によらずすべて消す(`reset` のみ) | なし |
| `--to <psd1\|json>` | `convert` の出力形式。大文字小文字を区別し、`psd1` と `json` 以外は `invalid value`(対象は値そのもの) | なし(必須) |

既定の定義ファイル名 `conductor.psd1` は、`conductor.json` を探索しない。既定の名前でファイルがなければ、通常の「ファイルなし」(64)になる。JSON の定義を使うときは、`--config conductor.json` のように明示する。人が書く定義にコメントが要るときは、`.psd1` を使う(JSON はコメントを書けない)、と利用者向けに案内する(使い方の文には入れない)。

`--config` のパスについての注意: `Resolve-Path`・`Join-Path`・`[System.IO.Path]::GetFullPath`・`Get-Item` は、文字列を変える(区切りの正規化、`.`・`..` の解決、相対パスの絶対化。`Resolve-Path` は、存在しないパスで誤りになる。確認していない挙動は未確認)。利用者に見せる文字列と、ステートのパスの元には、`--config` に渡された**元の `[string]`** を使い、`[System.IO.Path]` の API は、内部でパスをつなぐ(ステートのパスの作り方は [06-state-file.md](06-state-file.md))道具にとどめる。`Get-Content`・`Set-Content` などのコマンドレットで定義・ステートを読み書きしない(`[System.IO.File]` を使う。02 章 9.2 節)。空文字の `--config ''` は、ファイルを開けず、定義ファイルなし(64)になる(例外の型と文言は 03 章の読み込みの規定に従う。実機では未確認)。

## 4. 引数の解釈

解釈は、(A)字句の段階、(B)コマンドの組み立てと検査、の 2 段で行う。(A)は引数を先頭から 1 つずつ見て、`special`(最初に現れた `-h`/`-V` の種別)、`lexError`(最初の字句の誤り)、`tokens`(オプションと位置引数の並び)を作る。(B)は `tokens` を検査する。

### 4.0 内部の表現(骨格)

関数の名前と形は目安であり、4.1・4.2 の挙動を満たせば変えてよい。種別は、文字列の定数(下の表)で表す(`enum` と `class` は定義しない。モジュール内の複数のファイルにまたがる型の解決に制限があるため。02 章 5.3 節、10 章 8.1)。

| 名前 | 型 | 内容 |
|------|----|------|
| `Get-ConductorLexResult -Arguments [string[]]` | `[pscustomobject]` を返す | `Special`(`'None'`、`'Help'`、`'Version'`)、`LexError`(`$null` または使い方の誤りの値。10 章 8.3)、`Tokens`(下の `Token` の配列) |
| `Token` | `[pscustomobject]` | `Kind`(`'Positional'`、`'Verbose'`、`'Config'`、`'RunId'`、`'To'`、`'All'`、`'RemoveOnComplete'`、`'Unknown'`)、`Value`(位置引数・値を取るオプションの値。`Unknown` では綴り `-x`・`--bogus`) |

**配列の扱い(PowerShell の落とし穴)**: トークンの配列を作るとき、`$tokens += $t` は、要素のたびに配列を作り直す(要素数が多いと遅い)。トークン列は、`[System.Collections.Generic.List[object]]` に `Add` で足す。`Add` は戻り値がない。`ArrayList.Add` は戻り値(位置)を返すので、使うなら `[void]` を付ける。関数からリストを返すとき、**1 要素や 0 要素のリストが展開されない**ように、`return` に `,$list`(単項のコンマ)を使うか、呼び出し側で `@(...)` で包む。この仕様は、呼び出し側で `@(...)` を使い、返すときは単項のコンマを使わない(`[pscustomobject]` に `Tokens` のプロパティとして入れて返す。プロパティの値は展開されない)。

引数は .NET の `[string]`(UTF-16 のコードユニットの列)のまま扱う。`$s[$i]` と `$s.Length` はコードユニット単位であり、基本多言語面の外の文字(絵文字など。U+10000 以上)は、2 つのコードユニット(高位サロゲート + 低位サロゲート)になる。4.1 の連結の分解では、次の**自前の走査**で「1 文字(1 コードポイント)」を切り出す(`foreach ($c in $s.ToCharArray())` で 1 つずつ見てはならない)。

1. 位置 `i` のコードユニットが高位サロゲート(`[char]::IsHighSurrogate($s, $i)`)で、次の位置があり、そのコードユニットが低位サロゲート(`[char]::IsLowSurrogate($s, $i + 1)`)なら、その 2 つで 1 文字(コードポイントは `[char]::ConvertToUtf32($s[$i], $s[$i + 1])`)。
2. それ以外は、位置 `i` の 1 コードユニットを 1 文字として扱う(対になっていないサロゲートも 1 文字。4.4)。

「見た目の 1 文字」(書記素クラスタ。結合文字、国旗など)とは一致しない。連結の分解は、コードポイント 1 つずつで行う(`-é`(分解形で `e` + U+0301)は、`-e` と、知らない文字 U+0301 の 2 文字として分解される。`[System.Globalization.StringInfo]` で書記素単位に切らない)。

**文字列の比較と検索は、すべて序数(`StringComparison.Ordinal`)で行う。PowerShell は、既定が文化にも大文字小文字にも依存しない形ではない**ので、次の表のとおり書く(02 章 9.2 節)。

| やりたいこと | 書き方 | 使ってはならない書き方と理由 |
|--------------|--------|--------------------------------|
| 等価の比較(大文字小文字を区別) | `[string]::Equals($a, $b, [System.StringComparison]::Ordinal)`、または `-ceq` | `-eq`(確認していないが、PowerShell の比較演算子は既定で大文字小文字を区別しない。`'-V' -eq '-v'` が真になり、version と verbose を取り違える)。`==` は PowerShell に無い |
| 前方一致 | `$a.StartsWith('--', [System.StringComparison]::Ordinal)` | `$a.StartsWith('--')`(文化依存の既定の形。未確認)、`-like '--*'`・`-match '^--'`(大文字小文字を区別せず、`-like` はワイルドカード、`-match` は正規表現) |
| 文字の検索 | `$a.IndexOf('=', 2, [System.StringComparison]::Ordinal)`、または `$a.IndexOf([char]'=', 2)` | `$a.IndexOf('=')`(`'='` は `[string]` なので、文化依存の `string` の形が呼ばれる) |
| 部分文字列 | `$a.Substring($start, $length)`、`$a.Substring($start)` | `-split`・`-replace`(正規表現) |
| 文字の範囲の比較 | `$code = [int]$s[$i]` を取り、`$code -ge 65 -and $code -le 90` | `$c -ge 'A'`(文字列の比較は大文字小文字を区別せず、文化の影響も受けうる。未確認)、`[char]::IsLetterOrDigit`(Unicode の文字・数字に広く一致する) |
| 空白の除去・分割 | 引数を明示(`$s.TrimEnd([char]0x20)`) | `.Trim()`・`.TrimEnd()`・`.Split()`(引数なし)・`char.IsWhiteSpace`(Unicode の空白に広く一致する) |
| 値による分岐 | `switch -CaseSensitive -Exact ($name) { ... }`(各節の末尾に `break`) | `switch ($name)`(確認(出典: about_Language_Keywords の構文): `-CaseSensitive`・`-Exact`・`-Regex`・`-Wildcard` の指定がある。指定しない既定の挙動は、取得したページに記述がなく未確認だが、大文字小文字を区別しない、複数の節に一致すると全部実行される、と一般に言われる) |

### 4.1 字句の段階(A)の手順

`afterDashDash = $false` として、引数 `a` を先頭から順に処理する。

1. `afterDashDash` が真なら、`a` は位置引数(`-` で始まっていても、`-v` や `--help` であっても位置引数)。
2. `a` が `--` ちょうどなら、`afterDashDash = $true` にする(`--` 自体は位置引数にならない)。2 個目以降の `--` は、`afterDashDash` が真なので位置引数になる。
3. `a` が `--` で始まり(`--` より長い)、長いオプションとして扱う。`=` が含まれていれば、最初の `=` の前を名前(`--` を除く)、後ろを値(空でもよい)とする(`$a.IndexOf('=', 2, [System.StringComparison]::Ordinal)` で位置 `p` を得て、`$a.Substring(2, $p - 2)` と `$a.Substring($p + 1)`)。含まれていなければ名前は `a` から `--` を除いた全体で、値なし。
   - 名前が `help`: `special` が未設定なら help を設定する。`=値` が付いていても、help が優先されるので誤りにならない(例: `--help=x` は help)。
   - 名前が `version`: 同様に `special` が未設定なら version を設定する(`--version=x` も version)。
   - 名前が `verbose`, `all`, `remove-on-complete`(値を取らない): `=値` が付いていれば(値が空でも)`lexError`(最初の 1 つだけ保持)に `invalid value`(対象はそのオプションの正規の綴り、例 `--verbose`)を設定する。付いていなければ、そのオプションを `tokens` に追加する。
   - 名前が `config`, `run-id`, `to`(値を取る): `=値` があれば、それ(空文字も有効)を値として `tokens` に追加する。なければ、次の引数を、中身にかかわらず(`-` で始まっていても、`--` や `--help` であっても)値として取る(その引数は位置引数にもオプションにもならない)。次の引数が存在しないときは、`lexError` に `option requires a value`(対象は `--config` / `--run-id` / `--to`)を設定し、字句の段階を打ち切る(以降の引数は見ない)。
   - それ以外の名前(空の名前を含む): 「知らないオプション `--<名前>`」を `tokens` に追加する。`=値` が付いていたときは、`lexError` にも `unknown option`(対象 `--<名前>`、値は含めない)を設定する。名前が空のときの綴りは `--`(例: `--=x` は `unknown option: '--'`)。`---` は名前が `-` で、綴りは `---`。
4. `a` が `-` で始まり、長さが 2 以上(コードポイント数。4.0 の走査で数える)で、`--` で始まらない: 短縮形の連結として、先頭の `-` を除く文字を、先頭から 1 文字(1 コードポイント)ずつ処理する。
   - `h`: help を設定(上と同じ。`special` が未設定のときだけ)。`V`: version を設定(同様)。`v`: `-v` を `tokens` に追加する。
   - ほかの文字 `c`: 「知らないオプション `-<c>`」を `tokens` に追加する(`-é` は `unknown option: '-é'`。孤立サロゲート(4.4)の文字も 1 文字として `-<c>` の綴りにする)。
   - 2 文字目以降の位置で、直前の 1 文字の直後に `=` が続いているとき、その `=` 以降の残りの文字列は**まとめて捨て**(残りの文字は解釈しない)、`lexError` を設定する。直前の文字が `v` なら `invalid value`(対象 `-v`)、知らない文字 `c` なら `unknown option`(対象 `-<c>`)。直前の文字が `h`・`V` のときは `special` が優先されるので誤りにならない。連結の先頭の文字が `=` のとき(`-=x`)は、区切りではなく、`=` という文字そのものが知らないオプション(`unknown option: '-='`)。
   - 短縮形は値を取らない(値を取るのは長いオプションだけ)。`-vv` は `-v` が 2 回。`-vh` は `-v` と help(help が勝つ)。
5. それ以外(`-` ちょうど 1 文字、`-` で始まらない文字列、空文字)は位置引数。単独の `-` も位置引数である。

`lexError` は最初の 1 つだけを保持する(後の字句の誤りは捨てる)。

挙動の例(この文法の規則から導かれるもの):

- `--help=x`・`-V=1`・`--version=x` は、それぞれ help / version / version。`-vx run a` は `unknown option: '-x'`、`run -vh` は help、`-vV` は version、`--config --help run a` は `--help` という名前のパスを定義ファイルとして扱う(help ではない。拡張子が `.psd1`・`.json` のどれでもないので、定義の誤り `unsupportedFormat` として 64)。
- `--config --help`(ほかの引数なし)は `--help` を値に取り、位置引数が 0 個なので `missing argument`(68)。`run a --config --version` も `--version` を値(パス)として扱う。
- `--help --config` は、`--help` が先に解釈されるので、値の欠落(`--config`)より前に help(0)になる。
- `run a -- b` の `--` は、除かれて位置引数 `b` だけが残る(位置引数は `a` と `b` の 2 個なので `unexpected argument: 'b'`)。`run -- a --` は 2 個目の `--` が位置引数になり `unexpected argument: '--'`。`run -- -v` は、ジョブID `-v`。`run -` は、ジョブID `-`。
- `-v=1 run a` と `run -v=1h x` は、どちらも `invalid value: '-v'`(`=` 以降の `h` は解釈されず、help にならない)。`-v=` も `invalid value: '-v'`。`-vh=1` は help。`-x=1h` と `-vx=1` は `unknown option: '-x'`。
- `--=x`・`--=` は `unknown option: '--'`、`---` は `unknown option: '---'`、`--verbose=`(値が空)は `invalid value: '--verbose'`。
- `reset --all=1` は `invalid value: '--all'`。`run j --run-id`・`--to`(値なし)は `option requires a value`。
- `-Verbose`・`-Debug` は、連結の分解で、`-V`(version)・`-e`・`-r`… の並びになり、先頭の `V` で version(0)になる。`-Debug` は `-D` が知らない文字なので `unknown option: '-D'`(68)。

### 4.2 検査の順序(誤りの優先順位)

字句の段階のあと、次の順に検査する。最初に当てはまったもので終わる(それ以降は行わない)。

1. `special` が設定されていれば(`--` より前に、オプションとして解釈された `-h/--help` か `-V/--version` が 1 つでもあれば)、ほかの誤りに関係なく、先に現れた方を処理して終了コード 0。値を取るオプション(`--config`、`--run-id`、`--to`)の直後の引数は、`-h` や `--help` であっても値として取られ(4.1 の 3)、オプションとして解釈されないので、`special` にならない(`--config --help run a` は help ではない。`-h --config --version` は、`-h` が先に現れるので help で、`--version` は `--config` の値になる)。両方あるとき、先に現れた方が勝つ(`-V -h` は version、`-h -V` は help)。字句の段階が値の欠落で打ち切られた場合、それ以前に現れた `-h`/`-V` は有効、以降は存在しない扱い。
2. `lexError` があれば、それを誤りとする。
3. 「知らないオプション」が `tokens` にあれば、最初のものを `unknown option`(対象はその綴り: `-x`, `--bogus`)とする。
4. 位置引数が 0 個: `missing argument`(対象なし)。例: 引数なし、`-v` だけ、`--` だけ。
5. 先頭の位置引数(コマンド名)が「文字として正しくない引数」(4.4 の判定)なら `invalid value`(対象は、4.4 の規則で読める形にした文字列)。6 コマンドのいずれでもなければ `unknown command`(対象はその語。空文字も同じ)。
6. そのコマンドで使えないオプション: `option not allowed for this command`。検査の順は `-v`, `--config`, `--to`, `--run-id`, `--all`, `--remove-on-complete`。対象はそのオプション名(`-v` の綴りは `-v`)。可否は 3.1 の表による。
7. 重複: `-v`, `--config`, `--to`, `--run-id`, `--all`, `--remove-on-complete` のどれかが 2 回以上 → `invalid value`(対象はそのオプション名。`-vv` は対象 `-v`)。検査の順は挙げた順。`-v` の重複は、`-v` と `--verbose` の合計で数える。
8. `--all` と `--run-id` の併用 → `invalid value`(対象 `--all`)。
9. `--run-id` の値の検査(5.2)。値が文字として正しくない(4.4)、または空でなく規則に違反 → `invalid value`(対象 `--run-id`。値は含めない)。空文字は有効。
10. 位置引数の過不足(コマンド名を除いて数える):
    - `run`・`pipeline`・`convert` で位置引数が 0 個 → `missing argument`(対象なし)。
    - `watch`・`reset` で位置引数が 1 個以上 → `unexpected argument`(対象は最初の位置引数)。
    - コマンドが取れる個数を超える(2 個目の位置引数がある)→ `unexpected argument`(対象は 2 個目の語。`run a b` なら `b`)。
    - 位置引数(ジョブID、パイプライン名)が文字として正しくない(4.4)→ `invalid value`(対象は読める形にした文字列)。`convert` の入力ファイルは、4.4 の判定にかかっても検査せず、パスとしてそのまま通す。
11. `convert`: `--to` なし → `missing argument`(対象 `--to`)。`--to` の値が `psd1`/`json` 以外(文字として正しくない値を含む)→ `invalid value`(対象は、読める形にした値)。

誤りの表示形式は、`conductor: error: <文>: '<対象>'`(対象があるとき)、空行、使い方全文。文は [08-output-and-i18n.md](08-output-and-i18n.md) 4.2、対象は名前用エスケープ(64 文字で切る。08 章 9 章)。

### 4.3 判定表(挙動の早見表)

| 引数 | 結果 |
|------|------|
| (引数なし)、`--`、`-v` | `missing argument`(68) |
| `-- run`、`run --` | `missing argument`(68)(`--` のあとの `run` はコマンド名。ジョブIDがない) |
| `-- run a` | `run` コマンドで、ジョブID `a` |
| `--config` | `option requires a value: '--config'`(68) |
| `run a --config` | `option requires a value: '--config'`(68) |
| `--config --help run a` | `--help` というパスを定義ファイルとして読む(拡張子が合わないので、定義の誤り `unsupportedFormat` で 64) |
| `--config --help` | `--help` を値に取り、位置引数が 0 個: `missing argument`(68) |
| `--help --config` | help(0)(`--help` が先に解釈され、値の欠落は検査されない) |
| `-h --config --version` | help(0)(`--version` は `--config` の値) |
| `--help=x`、`-h=x` | help(0) |
| `-V=1`、`--version=x` | version(0) |
| `-vh x` | help(0) |
| `-vx run a` | `unknown option: '-x'`(68) |
| `-v=1 run a` | `invalid value: '-v'`(68) |
| `run --verbose=x a` | `invalid value: '--verbose'`(68) |
| `--bogus=x` | `unknown option: '--bogus'`(68) |
| `--bogus --help` | help(0)(`special` が最優先) |
| `run a -x` | `unknown option: '-x'`(68) |
| `run a -- b` | `unexpected argument: 'b'`(68) |
| `--run-id= watch` | 有効(run ID なしに固定)。ステートなしなら `no state (nothing has run yet)`、0 |
| `run -- -v` | ジョブID `-v` |
| `run -` | ジョブID `-` |
| `run --verb a` | `unknown option: '--verb'`(68)(前方一致の省略は受け付けない) |
| `run a --to json` | `option not allowed for this command: '--to'`(68) |
| `convert --to json --to psd1 f` | `invalid value: '--to'`(68) |
| `convert f --to xml` | `invalid value: 'xml'`(68)(`--to` の値は `psd1`・`json` だけ。大文字小文字も区別する) |
| `convert f --to JSON`、`convert f --to PSD1` | `invalid value: 'JSON'`、`invalid value: 'PSD1'`(68) |
| `convert f` | `missing argument: '--to'`(68) |
| `reset --all --run-id x` | `invalid value: '--all'`(68) |
| `run -vv a` | `invalid value: '-v'`(68) |
| `run -v:x a`、`run -vq a` | `unknown option: '-:'`・`unknown option: '-q'`(68)(`:` も区切りではなく、連結の中の知らない文字) |
| `run @a`、`run [x]`、`run /h`、`run $x` | それぞれジョブID `@a`・`[x]`・`/h`・`$x`(位置引数。展開やファイルの読み込みなどはしない) |
| `run -Verbose a` | `-V` で version(0)(4.2 の 1) |

### 4.4 引数の文字列と、文字として正しくない引数

#### 4.4.1 引数が `[string]` になる経路(確認した範囲)

`pwsh` は .NET のプロセスで、`[System.Environment]::GetCommandLineArgs()` は、プロセスが受け取ったコマンドラインを、文字列の配列として返す(確認(出典: 同 API のリファレンス): 最初の要素は実行ファイル名、以降が引数。単一ファイル発行では最初の要素はホストの実行ファイル名)。引数の区切りの規則は、次のとおり文書化されている(確認(出典: 同、Remarks))。

- 引数は空白で区切る。二重引用符(`"`)で囲むと、空白を引数に含められる。一重引用符(`'`)は、この機能を持たない。
- 二重引用符が、偶数個のバックスラッシュのすぐ後ろにあるとき、バックスラッシュの対 1 つにつき、バックスラッシュ 1 つになり、二重引用符は取り除かれる。二重引用符が、奇数個(1 個を含む)のバックスラッシュのすぐ後ろにあるとき、対 1 つにつきバックスラッシュ 1 つになり、残りの 1 つのバックスラッシュは取り除かれるが、二重引用符は取り除かれない。

文書に載っている対応の例(アプリ名 `MyApp`。`GetCommandLineArgs()` の 2 番目以降):

| コマンドラインの入力 | 引数(プログラム名を除く) |
|------------------------|------------------------------|
| `MyApp alpha beta` | `alpha`, `beta` |
| `MyApp "alpha with spaces" "beta with spaces"` | `alpha with spaces`, `beta with spaces` |
| `MyApp 'alpha with spaces' beta` | `'alpha`, `with`, `spaces'`, `beta` |
| `MyApp \\\alpha \\\\"beta` | `\\\alpha`, `\\beta` |
| `MyApp \\\\\"alpha \"beta` | `\\"alpha`, `"beta` |

これらは、Windows で引数の配列に入る値の例として、受け入れテストの入力に使ってよい(`pwsh.exe` が起動されたときに、同じ規則であることは未確認)。UNIX 系では、シェル(`sh` のランチャー)が分けた引数が、そのままプロセスの引数になる(確認: `"$@"` が 1 つ 1 つの引数を保つのは `sh` の一般的な仕様。取得した出典はなく未確認)。同じ見た目のコマンド行でも、OS によって引数が変わりうる(例: Windows のコマンドプロンプトでの一重引用符、バックスラッシュと引用符の組)。`conductor` は、受け取った文字列を、OS によらず同じ規則で解釈する(Windows 固有の再解析をしない)。

Windows の次の点は、取得した文書に記述がなく、未確認: 空文字の引数 `""` が空文字の要素として入るか、ワイルドカード(`*`)が展開されないか。

#### 4.4.2 UNIX 系での不正な UTF-8(未確認の範囲と、この版の決定)

UNIX 系の引数は、OS からはバイト列として渡される。そのバイト列が不正な UTF-8 を含むとき、`pwsh`(.NET)が `[string]` にどう変換するか(置換文字 U+FFFD になるのか、例外か、別の扱いか)は、公式ドキュメントでも確認できなかった。**未確認**。環境変数の値(`[System.Environment]::GetEnvironmentVariable`)についても同じ。さらに、`pwsh` が、不正な列を含む引数で、スクリプトを起動する前に失敗するかも未確認である(その場合は、`pwsh` の終了コード 1 などが、利用者に見える。`conductor` の誤りとして整えられない)。

したがって、この版は、実装が不正な UTF-8 を区別できない前提で、次の規則を決める。

- **「文字として正しくない文字列」の判定** `Test-ConductorSuspectText -Text <string>`(`Private/Text/` に置く。`Cli` と `Report` が共用する): `s` が、(a) 対になっていないサロゲート(高位サロゲートの次が低位サロゲートでない、低位サロゲートの前が高位サロゲートでない、末尾の高位サロゲート)を含む、または (b) 置換文字 U+FFFD を含む、のどちらかなら真。走査は、4.0 の自前の走査で行い、`[string]::IsNormalized`・エンコーディングの例外に頼らない。
  - (a) は、Windows の UTF-16 の引数(対になっていないサロゲートを含みうる)と、実装が不正な列をそのまま保持する場合のため。(b) は、実装が不正な UTF-8 を U+FFFD に置き換えて渡す場合のため。
  - 既知の差: 利用者が、U+FFFD を本当に含む(正しい UTF-8 の)文字列を、ジョブIDなどに指定した場合も、真になり、`invalid value` として拒否される。この差を受け入れる(ジョブIDは、定義の検査で許す文字が限られており([03-definition-format.md](03-definition-format.md))、U+FFFD を含まないため、定義済みのジョブIDを指定できなくなることはない)。
- 次のとおり扱う(4.2 の各段階で適用する)。

| 対象 | 扱い |
|------|------|
| コマンド名(最初の位置引数) | `Test-ConductorSuspectText` が真なら `invalid value`(4.2 の 5。対象は読める形) |
| ジョブID・パイプライン名(`validate` を含む) | 真なら `invalid value`(4.2 の 10。対象は読める形) |
| `--run-id` の値 | 真なら `invalid value`(4.2 の 9。対象は `--run-id`。値は含めない) |
| `--to` の値 | `psd1`/`json` 以外なので `invalid value`(4.2 の 11。対象は読める形) |
| `--config` の値、`convert` の入力ファイル | 真でも検査せず、`[string]` のままパスとして通す。ただし、不正な UTF-8 のバイトを持つファイル名は、置換が行われた `[string]` からは、元のバイト列を取り戻せず、そのファイルを開けない(UNIX 系のファイル名はバイト列であり、ここで元のバイト列を取り戻す公式の手段は確認できなかった。**制約として受け入れる**。実機での確認は未確認) |
| 知らないオプションの綴り(`-<c>`、`--<名前>`) | 真でもそのまま持つ。誤りの対象に出すとき、読める形にする |
| 環境変数の値(run ID、ロケール、`NO_COLOR`) | 真なら、未設定と同じ扱い(5.1、08 章 7 章) |

**読める形にする規則** `ConvertTo-ConductorReadableText -Text <string>`(使い方の誤りの対象に、文字として正しくない引数を出すとき、および 08 章 9 章のエスケープの前処理で使う): 対になっていないサロゲートを、1 つにつき U+FFFD 1 個に置き換える(対になっているサロゲートは、そのまま。すでに U+FFFD の文字は、そのまま)。実装が不正な UTF-8 を U+FFFD に置き換えて渡す場合、不正なバイト列 1 つあたり U+FFFD が何個になるかは、実装(ランタイム)に任せる。

## 5. run ID

### 5.1 決定の優先順位(`run`・`pipeline`)

1. `--run-id <値>` が指定されていれば、環境変数は見ない。値が空文字なら run ID なし。
2. 指定がなければ、環境変数を読む(`Get-ConductorEnvironmentVariable -Name <string>`。戻り値は `[string]` または `$null`。実体は `[System.Environment]::GetEnvironmentVariable($Name)`)。環境変数の名前は、定義の `pipeline.run_id_env`(なければ `CONDUCTOR_RUN_ID`)。未定義(`$null`)または空文字なら run ID なし。値が `Test-ConductorSuspectText`(4.4.2)なら、未定義と同じ扱い。
3. 得た値は 5.2 の規則で検査する。違反は `invalid value`(終了コード 68)。対象は、オプション由来なら `--run-id`、環境変数由来なら `run ID (environment variable)`(値は含めない)。

`run`・`pipeline` では、環境変数の名前を知るために、定義を先に読む。検査の順序は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md) 2 章を参照する。

環境変数の読み取りで注意する点: `[System.Environment]::GetEnvironmentVariable($Name)` は、定義がないとき `$null` を返す(.NET の API の説明。出典のページは取得しておらず、未確認)。`$null` のとき「未定義」、`''` のとき「空文字」(どちらも run ID なし)として、同じに扱う。確認(出典: about_Environment_Variables): PowerShell の `$Env:` では、「an environment variable can't be set to an empty string」(空文字を代入すると、その変数が削除される)ので、製品コードは、`$Env:名前` への代入で環境を変えない(`conductor` は環境を変えない)。名前の大文字小文字の区別は OS に従う。確認(出典: 同): 「environment variable names on macOS and Linux are case-sensitive」(UNIX 系は区別し、Windows は区別しない)。環境変数の読み取りは、すべて `Get-ConductorEnvironmentVariable` を通す(`$Env:` を直接参照するのは、テストの差し替えの口である、この関数 1 つだけ)。

### 5.2 run ID の規則

- 文字は `A-Za-z0-9._-` だけ、長さは 1〜64 文字、先頭が `.` でない(`..` も不可)。空文字は「run ID なし」を表し、値としては規則を満たさない(オプションの空文字だけが「なし」を意味する特別な扱い)。
- 検査は `Test-ConductorRunId -Value <string>`(`Private/RunId/`。純粋な関数。戻り値は `[bool]`)に置く。**正規表現を使わず、1 文字ずつの比較で書く**: 各文字の値 `$code = [int]$value[$i]` が、`65..90`(`A`〜`Z`)、`97..122`(`a`〜`z`)、`48..57`(`0`〜`9`)、`46`(`.`)、`95`(`_`)、`45`(`-`)のどれかであること(4.0 のとおり、文字ではなく整数で比べる)。文字がすべて ASCII であることを確かめてから、`$value.Length`(コードユニット数)で長さを検査する(ASCII だけなので、コードユニット数、コードポイント数、UTF-8 のバイト数が同じになる)。正規表現を使う場合の注意: PowerShell の `-match` は大文字小文字を区別せず、.NET の正規表現の `$` は末尾の改行の直前にも一致し、`\d` は既定で Unicode の数字に一致する(いずれも出典のページは取得しておらず、未確認)ので、使うなら `[regex]` に `\A`・`\z`、文字クラスの明示、`RegexOptions.CultureInvariant` が要る。実装は正規表現を使わないのが既定。
- ステートファイル名は、run ID なしで `<定義ファイル名>.state`、ありで `<定義ファイル名>.<run-id>.state`(定義ファイルと同じディレクトリ)。詳細は [06-state-file.md](06-state-file.md)。
- 定義の `run_id_env` は、`A-Za-z_` で始まり `A-Za-z0-9_` が続く 1 文字以上(同じく整数の範囲の比較で検査する)。違反は定義の誤り(64。[03-definition-format.md](03-definition-format.md))。
- 理由の区別(空・長すぎ・不正な文字・先頭のドット)は内部では持ってよい(`'Empty'`、`'TooLong'`、`'InvalidChar'`、`'LeadingDot'` の文字列)が、利用者には区別を見せず、すべて `invalid value`(68)にまとめ、値を含めない。

### 5.3 `watch` / `reset` の run ID

- `--run-id` があれば、定義ファイルを読まない(環境変数は見ない)。
- なければ、定義から `pipeline.run_id_env` だけをゆるく読む。次のどれでも、既定名 `CONDUCTOR_RUN_ID` を使う: ファイルが読めない、拡張子が `.psd1`・`.json` のどれでもない、その書式として読めない(深さ・大きさの上限を超える場合、許可しない構文を含む場合を含む)、`pipeline` がマップ(`.psd1` ではハッシュテーブル)でない、キーがない、文字列でない、名前の形が 5.2 の規則(`run_id_env` の形)と違う。ほかの誤りは無視するので、壊れた定義でも動く。得た名前の環境変数を見る。
  - 「ゆるく読む」ときに、定義の検査(未知のキー、版など)は行わない。読み方の細部は [03-definition-format.md](03-definition-format.md) の読み込み部品に従い、ここでは「読み込みに失敗すれば既定名」とだけ規定する。`.psd1` を読むときも、コードを実行しない(03 章の許可する構文の検査を通す)。
- 環境変数の値が規則違反なら `invalid value`(68)。
- `reset --all` は、環境変数も `--run-id` も見ない。

## 6. 環境変数

| 名前 | 読む場所 | 効果 |
|------|----------|------|
| `CONDUCTOR_RUN_ID` | `run`, `pipeline`, `watch`, `reset`(`--run-id` なし、かつ定義の `run_id_env` なし) | run ID の既定の供給源(5 章) |
| `run_id_env` が指す名前(定義で指定) | 同上 | `CONDUCTOR_RUN_ID` の代わりに読む環境変数名 |
| `LC_ALL`, `LC_MESSAGES`, `LANG` | 表示が必要になった最初の時点 | この順に、最初の空でない値が `ja` で始まれば日本語、それ以外は英語([08-output-and-i18n.md](08-output-and-i18n.md) 7 章) |
| `NO_COLOR` | `watch` の表を出すとき | 未設定または空なら色を使える(ただし標準出力が端末でなく、Windows でないとき。08 章 6.3)。空でない値が設定されていれば色なし |

環境変数の読み取りは、すべて `Get-ConductorEnvironmentVariable` を通す。これら以外の環境変数は、`conductor` 自身は読まない(子プロセスには親の環境をそのまま引き継ぐ。07 章)。`pwsh` 自身が読む環境変数(`POWERSHELL_UPDATECHECK`、`POWERSHELL_TELEMETRY_OPTOUT`、`PSModulePath`、`TERM` など。確認(出典: about_Environment_Variables))は、`conductor` の挙動の仕様には含めない。ランチャーが、`POWERSHELL_UPDATECHECK` と `POWERSHELL_TELEMETRY_OPTOUT` を設定する(1.3)ことは、子プロセスに引き継がれる(仕様として受け入れる)。ロケールの判定に `$PSCulture`・`$PSUICulture`・`[System.Globalization.CultureInfo]` を使わず、環境変数を自前で読む(08 章 7 章)。

## 7. 各コマンドの動作

ここでは入口の検査の順序と、コマンドごとの外から見える挙動を規定する。内部の流れの詳細は [05-execution-flow.md](05-execution-flow.md)。各サブコマンドの実装は `Private/Cli/` の関数(`[int]` を `return` で返す)とする。

### 7.1 `run <job-id>`

1. シグナルの監視を開始する(定義の検査より前)。失敗は終了コード 70([07-process-and-signals.md](07-process-and-signals.md))。
2. 定義の読み込みと検証(失敗は 64)。全ジョブの `command` を検査(失敗は 64)。
3. ジョブIDが定義にあるか(なければ `job is not defined in the definition file: '<ID>'`、68)。
4. run ID の決定(5 章。違反は 68)。
5. ステートの読み込み(65)と、定義との照合(65)。
6. 起動前の判断: 実行 / スキップ(0)/ 拒否(66)。
7. 実行なら、ステートが空のとき初期ステートを書き(`Fast` の耐久性)、ジョブをリトライ込みで実行し、結果を分類してステートを 1 回書き(`Durable` の耐久性)、完了かつ削除指定ならステートを消す。終了コードはジョブの結果(10 章)。
8. スキップなら、記録が変わるときだけ書く。完了し削除指定なら消す。0 で終わる。

`-v` のとき、結果行・遷移行・中断行を標準エラーに出す(08 章)。

### 7.2 `pipeline <pipeline-name>`

`run` と同じ検査(パイプライン名は定義の `name` と照合。不一致は 64)を行い、定義の記述順にジョブを 1 つずつ、起動前の判断で処理する。

- 実行 → 結果を記録。エラー(または分類なし=起動前の中断)で停止し、そのジョブの終了コードで終わる。自動の再開・リトライはしない(`retry` の設定を除く)。
- スキップ → 記録して続ける。
- 拒否 → 66 で終わる。
- 完了(最後のジョブ、`@end`、スキップによる完了)したら、完了のステートを残して 0 で終わる。`--remove-on-complete` または定義の `remove_on_complete`(`.psd1` では `remove_on_complete = $true`、JSON では `"remove_on_complete": true`)のときだけステートを消す(消せなければ 65)。
- ジョブのあいだに中断の合図が立っていれば、次のジョブを起動せず、再開位置を記録して `128 + 番号` で終わる(`-v` のとき `[conductor] interrupted by signal N`)。
- 記述順のジョブを使い切っても完了していない(通常は起こらない)と、`cannot determine whether the pipeline is complete` を出して 65。

### 7.3 `watch`

定義ファイルを検証しない。`--run-id`/環境変数で run ID を決め(5.3)、ステートを、完了のものも含めて読み(読み込みの検査は [06-state-file.md](06-state-file.md))、表を標準出力に 1 回出して 0 で終わる。ファイルなし・長さ 0 のときだけ `no state (nothing has run yet)`。読めない・壊れているは 65(`reset` の案内は run ID なしなら付き、run ID ありなら `--run-id` つきの案内が付く。08 章 4.5)。標準出力への書き込み失敗(`IOException` など。08 章 1.3)は `cannot write to standard output` を標準エラーに出して 65(UNIX 系では、読み手が先に閉じたパイプへの書き込みは、.NET の標準のストリームが `EPIPE` を捨てるため検出できず、65 でなく 0 で終わりうる(08 章 1.3.1 の既知の差。Windows と、`/dev/full` などの `EPIPE` 以外の失敗では、65))。表の形式は 08 章 6 章。シグナルの準備はしない。

### 7.4 `reset`

ステートを読まず、中身の検査もしない(壊れていても動く)。ステートがなくても成功(0)。

- `--run-id` なし(かつ `--all` なし): 5.3 の規則で run ID を決め、そのステートファイルを消す。
- `--run-id <ID>`: その run ID のファイルだけを消す。
- `--all`: その定義のステートを、run ID の有無によらず全部消す。`--run-id` との併用は 68。環境変数は見ない。読めなくて残したファイルがあれば、標準エラーに警告を出す(終了コードは 0。08 章 4.10)。消すのに失敗したら 65。消すものがなければ何も出さず 0。

`--all` で消す対象(定義ファイル名を `<定義ファイル名>` とする。定義ファイルと同じディレクトリを、ファイル名の前方一致で走査する。走査には `[System.IO.Directory]::EnumerateFileSystemEntries` などの列挙 API を使い、`Get-ChildItem` を使わない。名前の前方一致の比較は `StartsWith(..., [System.StringComparison]::Ordinal)` で行う):

- `<定義ファイル名>.state`(run ID なし): 中身を読まず、通常のファイルなら消す。シンボリックリンクは消さない(リンクかどうかは `[System.IO.FileSystemInfo]` の `LinkTarget` や `Attributes` の `ReparsePoint` で判定する。リンクをたどる `[System.IO.File]::Exists` の結果だけで判断しない。これらの挙動の詳細は公式ドキュメントを取得しておらず、未確認で、06・07 章の実装時に確かめる)。
- `<定義ファイル名>.<run-id>.state`: 名前の `<run-id>` が 5.2 の規則に合い、かつ、中身の `run_id` の値が名前の run ID と一致するものだけ消す。中身が読めないものは消さずに残し、警告に挙げる。
- 一時ファイル、別の定義のステート、ディレクトリ、シンボリックリンクは消さない。

(この細かい規則は、利用者向けの説明を根拠にした記述で、細部は未確認。ステートファイルの形式は [06-state-file.md](06-state-file.md)。)

### 7.5 `validate [<pipeline-name>]`

定義を読んで検証し、全ジョブの `command` を検査するだけ。ジョブを起動せず、シグナルの準備もせず、ステートに触れない(読まず・書かず・消さない)。壊れたステートがあっても成功する。

- 成功: 何も出さず 0。`-v` のとき、標準エラーに `[conductor] definition is valid (<N> job|jobs)`(英語。`N == 1` のとき `job`、それ以外は `jobs`)/ `[conductor] 定義は正しいです(ジョブ <N> 個)`。
- 失敗: `run` と同じ表示で 64。
- 名前を渡すと定義の名前と照合する(不一致は 64)。省略すると照合しない。`--run-id` は 68(使えないオプション)。

### 7.6 `convert <file> --to <psd1|json>`

入力ファイルを、入力形式を `--to` の反対(`--to json` なら入力は `.psd1`、`--to psd1` なら入力は JSON)とみなして変換する。この判定で、拡張子は見ない(定義ファイルの読み込みとは異なる)。結果を全部作ってから標準出力に 1 回で書く(書き込みの完了を待つ。08 章 1.3)。失敗したときは標準出力に何も書かず、標準エラーに原因を出して 67。標準出力に書けないときも 67(`cannot write to standard output`。UNIX 系では、読み手が先に閉じたパイプへの書き込みは、.NET の標準のストリームが `EPIPE` を捨てるため検出できず、67 でなく 0 で終わりうる(08 章 1.3.1 の既知の差。Windows と、`/dev/full` などの `EPIPE` 以外の失敗では、67))。シグナルの準備はしない。変換の規則そのもの(コメントとキー順の扱い、許可する構文、入力の大きさの上限など)は [09-converter.md](09-converter.md)。

(「入力形式を `--to` の反対とみなし、拡張子を見ない」という判定の規定は、この章が持つ。`--to psd1` のときの入力は JSON として厳密に読む([09-converter.md](09-converter.md))ので、拡張子が `.psd1` でも、`--to psd1` で渡せば JSON の文法の誤り(67)になる。)

## 8. ヘルプとバージョン

- `-h`/`--help`: 使い方全文を標準出力に出して 0。言語はロケールに従う(全文は 08 章 8 章)。標準出力に書けなくても(閉じたパイプなど)、失敗は無視して 0 で終わる(08 章 1.3.2。`watch` は 65、`convert` は 67 で、ここだけが違う)。
- `-V`/`--version`: 標準出力に `conductor <版>` と改行 1 つ(`conductor 0.8.0`)。ロケールを判定しない(常にこの英語の形)。終了コード 0(標準出力に書けなくても、失敗は無視して 0。08 章 1.3.2)。版の文字列は、モジュールのマニフェスト `Conductor.psd1` の `ModuleVersion`(唯一の出所)を、モジュールの読み込み時に 1 度だけ、`Import-PowerShellDataFile`(マニフェストのパスは `Join-Path $PSScriptRoot 'Conductor.psd1'` で作る。`.psm1` の中の `$PSScriptRoot` は、確認(出典: about_Automatic_Variables)すべてのスクリプトで有効)で読んで、モジュール内の変数 `$script:ConductorVersion` に保ち、`Get-ConductorVersion`(`Cli`。02 章 10 章)が返す。`Test-ModuleManifest`(重い。未確認)、`Get-Module` の呼び出しを実行のたびに行わない。**`Import-PowerShellDataFile` が、マニフェストを読めること(`ModuleVersion` が文字列で返ること)は未確認**(試験 T6)。確かめて使えなければ、`$script:ConductorVersion` に定数 `'0.8.0'` を書き、マニフェストの値と一致することをテストで検査する(11 章)方式に直す。

```text
$ conductor --version
conductor 0.8.0
```

## 9. 実行例(`LANG=C`)

```text
$ conductor bogus
conductor: error: unknown command: 'bogus'

Usage:
  conductor run [-v] ...(使い方の全文が続く)
[終了コード 68]

$ conductor run -vv a
conductor: error: invalid value: '-v'
(空行と使い方)
[68]

$ conductor watch -v
conductor: error: option not allowed for this command: '-v'
[68]

$ conductor --run-id 'a b' watch
conductor: error: invalid value: '--run-id'
[68]

$ conductor reset --all --run-id x
conductor: error: invalid value: '--all'
[68]

$ conductor convert conductor.psd1
conductor: error: missing argument: '--to'
[68]

$ conductor convert conductor.psd1 --to xml
conductor: error: invalid value: 'xml'
[68]

$ conductor run nojob          # 定義にないジョブID
conductor: error: job is not defined in the definition file: 'nojob'
[68]

$ CONDUCTOR_RUN_ID='bad id' conductor watch
conductor: error: invalid value: 'run ID (environment variable)'
[68]

$ conductor -vh        # help が勝つ。標準出力に使い方、0
```

## 10. この版が採った選択(まとめ)

| 項目 | 内容 |
|------|------|
| ランチャー | UNIX 系は `sh` スクリプト、Windows は `.cmd`。`pwsh -NoLogo -NoProfile -NonInteractive -File <入口スクリプト> <利用者の引数>`。`-Command`・`-CommandWithArgs` は使わない |
| 入口スクリプト | `bin/conductor.ps1`。`param()`・`[CmdletBinding()]`・`$input` なし。`exit` を呼ぶのはここだけ |
| 引数の取り出し | `[System.Environment]::GetCommandLineArgs()` の、最初の `-File` の後ろ(1.5)。`$args` は、ランチャーを経由しない起動のときだけ |
| `Invoke-Conductor` | `-ArgumentList [string[]]`、戻り値は `[int]` 1 つ。`[CmdletBinding()]` なし、`exit` なし |
| 既定の定義ファイル名 | `conductor.psd1`(`--config` の拡張子は `.psd1`・`.json`。それ以外は定義の誤り `unsupportedFormat`、終了コード 64) |
| `--to` の値 | `psd1` か `json`(それ以外は `invalid value`、68) |
| `convert` | `.psd1` と JSON の相互変換(入力形式は `--to` の反対。拡張子を見ない。出力のキーの順は入力の記述順) |
| 定義の `run_id_env` の読み方(`watch`/`reset`) | `.psd1`/JSON をゆるく読む(読めなければ既定名。03 章) |
| 引数の解析 | PowerShell のパラメータ束縛と引数解析用のモジュールを使わず、4 章の自前の手順 |
| 文字の単位 | コードポイント(サロゲートの対を 1 文字として扱う自前の走査)。比較は序数(4.0 の表) |
| 文字として正しくない引数 | 対になっていないサロゲートまたは U+FFFD を含む文字列(`Test-ConductorSuspectText`)で近似して区別し、コマンド名・ジョブID・パイプライン名・`--run-id`・`--to` の値は `invalid value`。`--config`・`convert` の入力のパスはそのまま通す(4.4.2) |
| 文字として正しくない環境変数 | 未設定と同じ扱い(5.1、08 章 7 章) |
| 誤りの表示用の変換 | `ConvertTo-ConductorReadableText`: 対になっていないサロゲートを U+FFFD に置き換える(4.4.2) |
| run ID の検査 | 正規表現を使わない、整数の範囲の比較(5.2) |

### 設計の選択と理由

| 項目 | この章の選択 | 理由 |
|------|--------------|------|
| UNIX 系のランチャー | `bin/conductor`(`#!/bin/sh`)+ `bin/conductor.ps1`。先頭を `#!/usr/bin/env pwsh` にした 1 つの `.ps1` にしない | シェバンの行に `-NoProfile` などを足す移植性のある方法を確認できない(1.3.2) |
| 引数の受け方 | `Invoke-Conductor` は `-ArgumentList [string[]]`。生の引数は `GetCommandLineArgs()` から取り出す。`$args` を使わない | `pwsh -File` が、引数を PowerShell のパラメータの形に変換する(`:` で分割、`-` 始まりをスイッチ化)ため、`$args` で元の文字列が保たれる保証がない(1.5) |
| 版の出所 | マニフェスト `Conductor.psd1` の `ModuleVersion` が唯一の出所。読み込み時に 1 度、`Import-PowerShellDataFile` で読む(使えなければ定数とテスト) | 8 章 |

## 11. 未確認

### 11.1 未確認の一覧

- `GetCommandLineArgs()` が、`pwsh` の受け取った引数を、変換や欠落なしに返すか。先頭の要素の中身(1.5)。
- `pwsh -File` の引数が `$args` に届くときの形(`-v`、`--`、`--opt=value`、`-x:y`、空文字)(1.5、1.6)。
- Windows の引数の配列が、`GetCommandLineArgs()` の文書の規則(4.4.1)と同じ結果になるか。空文字の引数 `""`、ワイルドカード。`pwsh.exe` が自分のコマンドラインを同じ規則で分けるか(1.3.3)。
- UNIX 系で、不正な UTF-8 のバイトを含む引数・環境変数の値が、`[string]` にどう入るか(置換か、例外か、`pwsh` の起動前の失敗か)。不正な UTF-8 のバイトを含むファイル名を、`[string]` から開けるか(4.4.2)。
- `readlink` を使ったシンボリックリンクの解決の移植性(1.3.2)。更新通知が、非対話の `-File` でも出るか(1.3.2)。
- `.cmd` の `%*` の展開と、引用符・`&`・`^` の扱い。Ctrl+C のときの確認の挙動(1.3.3)。
- `Invoke-Conductor` に `[CmdletBinding()]` を付けたときの、`-v` と `-Verbose` の束縛(1.4)。
- `pwsh -File` の直後の、`$PWD` と `[System.Environment]::CurrentDirectory` の一致(1.4)。
- `Import-PowerShellDataFile` でのマニフェストの読み取り(8 章)。
- `[System.Environment]::GetEnvironmentVariable` の、定義がないときと空のときの値(5.1)。
- `[System.IO.Path]` の各 API の、文字列の変化の詳細。`--config ''` を開こうとしたときの例外の型(3.3)。
- シンボリックリンクの判定 API の挙動(7.4)。`reset --all` の対象の細部(7.4)。
- サポートする最小の PowerShell の版(実装時に確認する)で、1.3〜1.6・4.4・5.1 の確認を繰り返すこと。

### 11.2 最初に確かめる試験(実装の最初に、3 OS で行う)

| 番号 | 試験 | 期待 | 外れたときの対処 |
|------|------|------|------------------|
| T1 | 引数の経路: ランチャー経由で、`-v`、`-vh`、`--`、`--config=C:\a.psd1`、`--config=a:b`、`--opt=value`、`-`、空文字、日本語、絵文字を渡し、入口スクリプトが取り出した配列(1.5)と、`$args` の両方を、1 要素ずつ表示する | 取り出した配列が、渡した文字列と完全に一致する | 1.5 の接頭辞方式に切り替える |
| T2 | 終了コード: 入口スクリプトから `exit 0`、`exit 1`、`exit 255`、`exit 256`、`exit -1` を呼び、`pwsh -File` の終了コードをシェル(`$?`、`%ERRORLEVEL%`)で読む | UNIX 系は 0〜255(256 は 0、-1 は 255。確認(出典: about_Language_Keywords)の記述どおり)、Windows は 32 ビットの値 | 10 章 4 を直す |
| T3 | 未捕捉の例外: 入口スクリプトの `try` の外で例外を投げ、終了コードを読む | 1(確認(出典: about_Pwsh)の記述どおり) | 10 章 6 を直す |
| T4 | Ctrl+C(端末)と SIGINT(`kill -INT`)を、`pwsh -File` の実行中のスクリプトに送り、終了コード、`finally` の実行、`PipelineStoppedException` の捕捉可否を確かめる | 10 章 5・6.4 の記述どおり | 10 章 5 を直す |
| T5 | `Invoke-Conductor` に配列を渡したときの、`-v` などを含む配列要素の保持。`$args` が空でないときの検出 | 配列の要素が保たれる | 1.4 を直す |
| T6 | `Import-PowerShellDataFile` でマニフェストの `ModuleVersion` を読む | 文字列 `'0.8.0'` | 8 章の定数方式に切り替える |
| T7 | 入口スクリプトが `$input` を使わないとき、子プロセスが標準入力を読めること(`cat` など) | 読める | 1.3.1 を直す |
