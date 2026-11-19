# 12. 再構築の手引き(PowerShell でゼロから実装する)

この文書は、`conductor` を PowerShell 7(`pwsh`)でゼロから実装する人のための、実装の順序、PowerShell に特有の落とし穴、未確認事項、最初に確かめる小さな試験をまとめる。各規定の本文は、02〜11 章にある。この文書は、それらへの道案内であり、規定を足さない。食い違いがあれば、各章を正とする。

- 規定の強さの語は [00-README.md](00-README.md) 4 章。
- 受け入れテスト(AC)は [11-quality-and-build.md](11-quality-and-build.md) 4 章。ここで「AC-xxx〜yyy」と書くのは、その範囲の項目を指す。
- この仕様書を書いた環境には `pwsh` も .NET SDK もなく、挙動は、公式ドキュメントと公開ソースの読み取りで確かめた範囲だけが「確認」である。実機で動かして確かめたものは無い。したがって、3 章の未確認事項と、4 章の試験は、実装の最初に行う。
- サポートする PowerShell は 7.6 LTS(作成日 2026-11-20 の時点。確認: 11 章 1 章。出典: 「PowerShell Support Lifecycle」)。実装時に、サポートの状況と次の LTS を確認して決める([00](00-README.md) 1 章)。版の出所は `build/Versions.psd1`。
- 試験の番号は、`<章>-<元の番号>` で書く(例: `04-T1` は 04 章 11.2 節の T1、`07-T-4` は 07 章 12.2 節の T-4、`11-T3` は 11 章 10.2 節の T3)。章ごとに番号を振っているので、章を付けないと重なる。

## 1. 実装の順序

依存の向き([02](02-architecture.md) 4 章。`Cli` → `Runner`/`State`/`Report` → `Flow`/`Definition`/`Convert`/`RunId` → `Json` → `Text`)の下から上へ、かつ「純粋な部分を先に、入出力を後に」の順で作る。各段階は、完了条件を満たしてから次へ進む。段階ごとの確認は、その段階の単体テストと、11 章の受け入れテストのうち、その段階で動くものの範囲で行う。受け入れテストは、配布物のランチャーを子の `pwsh` を含む外部プロセスとして動かすので、`Cli` とランチャーがそろった段階 11 から本格的に使える。それ以前は、単体テストと、偽の `Deps`(02 章 7 章)で確かめる。

PowerShell は、ファイルを書いてから動かすまでが速い一方、名前の綴り・型・戻り値の混入・大文字小文字の誤りが、実行するまで見つからない。このため、**先に `build/` のスクリプトと静的検査を作り(段階 0)、実機の試験で前提を確かめ(段階 1)、そのあとで部品を作る**。

| 段階 | 作るもの | 読む章 | 完了条件 | 確認する方法 |
|------|----------|--------|----------|--------------|
| 0 | 準備: リポジトリの骨組みと、`build/` の検査・テスト・パッケージのスクリプト。`src/Conductor/Conductor.psd1`(`ModuleVersion = '0.8.0'`、`PowerShellVersion = '7.6'`、`FunctionsToExport = @('Invoke-Conductor')`)、`Conductor.psm1`(読み込みの順序、`Set-StrictMode -Version Latest`、`$ErrorActionPreference = 'Stop'`、版の読み取り)、`Private/{Text,Json,RunId,Flow,Definition,Convert,State,Runner,Report,Cli}/`(空)、`Public/Invoke-Conductor.ps1`(仮の本体)、`build/Versions.psd1`・`Dependencies.psd1`・`Install-DevDependencies.ps1`・`Resolve-Pwsh.ps1`・`Invoke-Tests.ps1`・`Invoke-Checks.ps1`・`New-Package.ps1`(11 章 5.4 節の全文)、`PSScriptAnalyzerSettings.psd1`(11 章 9.2 節)、`.editorconfig`・`.gitattributes`・`.gitignore`(9.3 節)、`tests/Inspect/` の依存の向きの点検(AST の走査。02 章 4.1 節)、CI(`ci.yml`。11 章 5.1 節) | 02 の 2〜6・10 章、11 の 2.4・2.5・5・8・9 章 | `./build/Install-DevDependencies.ps1` が、固定の版とハッシュで Pester と PSScriptAnalyzer を入れる。`./build/Invoke-Checks.ps1 -Check lint`(PSScriptAnalyzer、禁止コマンド、書式、マニフェスト)と `-Check version`・`consistency`・`deps` が、骨組みに対して通る。`Import-Module ./src/Conductor` が通り、公開される関数が `Invoke-Conductor` だけ。Pester が(0 件でも)通る。CI が 3 OS で緑 | AC-476・477(版)、AC-478・481・573(依存)、AC-480・483・518(静的検査)、AC-508(マニフェスト)、AC-514(依存の向き)、AC-529(読み込み)、AC-562、AC-572(公開は 1 つ) |
| 1 | **最初に確かめる小さな試験**(4 章の表の「段階 1」)。`pwsh` の引数の経路、終了コード、Ctrl+C と `finally`、`Add-Type`、シグナルのコールバック、構文木の数・文字列・深い入れ子、数値の演算、`ConvertFrom-Json` との比較、閉じたパイプ、`Mock`・`CodeCoverage` の挙動など | 各章の「最初に確かめる試験」(04 の 11.2、03 の 16、05 の 17、06 の 13.4、07 の 12.2、09 の 12、11 の 10.2) | 4 章の「段階 1」の行がすべて実行され、結果(PowerShell の版と OS)が表の「結果」欄(各自の記録)に残っている。期待と違った行は、「直す章」を直し、この文書の 3 章・4 章にも反映した。最小と最新の PowerShell(分かれているとき)、3 OS で行った | 4 章の表。特に `04-T1`(生の引数)、`04-T4`(Ctrl+C)、`07-T-4`(`finally` の中の `Environment.Exit`)、`11-T4`・`07-T-2`(`Add-Type`)の結果で、ランチャーとヘルパーの設計(04 章 1.3〜1.5 節、07 章 7 章)が確定する |
| 2 | `Text`・`Json`: `Test-ConductorSuspectText`、`ConvertTo-ConductorReadableText`、`ConvertTo-ConductorEscapedText`、`Test-ConductorRunId`、`Read-ConductorJsonTree`(明示のスタック。重複キー、`hugeInt`、位置)、`ConvertTo-ConductorJsonText`、`Test-ConductorTreeLimits`、木のノードの生成、`Test-ConductorJsonInt32` | 03 の 1・5・6 章、04 の 4.0・4.4・5.2、06 の 6.5、08 の 9・11 | 03 の 14 章と 06 の 6.6 節の入力(受理・拒否)が単体で通る。深さ 100,000 の入れ子でも、異常終了せず、定められた誤りで返る(再帰を使っていない)。1 MiB の `[` の列が目標の時間(30 秒。`03-T-16`)で読める。文字の判定が整数の比較(`[int]$c`)で、`-eq`・`-match`・`Trim()` を使っていない(禁止コマンドの検査が通る) | 単体(11 章 3.2 節の `Text`・`Json` の行)。AC-370〜388(エスケープ)、AC-461〜468(JSON)、AC-524(深さ)、AC-554(配列の単一化)の単体の部分 |
| 3 | `Definition`: `Get-ConductorFormatFromPath`、`Read-ConductorPsd1Tree`(前処理、`ParseInput`、ノード型の許可リスト、`Extent.Text` による数の字句、重複キー)、`Read-ConductorTree`、`Read-ConductorDefinitionFile`・`ConvertFrom-ConductorDefinitionText`、検査(Walk)、`ConvertFrom-ConductorInterval`、`Get-ConductorRunIdEnv` | 03 の 2〜4・6〜13 章、10 の 7.1 | 03 の 14 章の受理・拒否の全例が通る。31 種のエラーの種類と、位置の順の出力(行・列)。**`.psd1` の中のコードが評価されない**(AC-454・531)。`Import-PowerShellDataFile` との比較(許可の範囲で同じ値)。`ConvertFrom-Json` を使っていない | 単体(`Definition` の行)。AC-001〜075、AC-450〜470(定義の読み取り)、AC-453・454・456・457・531・578〜580 |
| 4 | `Flow`(純粋): 値の生成、`Get-ConductorOutcome`、`Get-ConductorClassifiedStep`、`Get-ConductorNextStep`、`Get-ConductorDecision`、`Get-ConductorResumePoint`、リトライ(`Get-ConductorRetryDecision`、`Get-ConductorNextWaitSecs`。`[bigint]` の飽和)、`ConvertTo-ConductorProcessExitCode`、記録関数、`Test-ConductorStateEqual` | 05 の 0〜4・6・7・9・13〜15 | 05 の 15 章の「固定する項目」がすべて単体で通る。`[bigint]` の飽和(`[uint64]::MaxValue` でも例外・折り返しなし。AC-503 の入力)。比較が `-ceq`。`Private/Flow/` に禁止の呼び出しがない | 単体(`Flow` の行)。AC-239〜279 と AC-389〜396 の単体の部分 |
| 5 | `State`: パス、`ConvertTo-ConductorStateJsonBytes`、`Read-ConductorState`・`Read-ConductorStateRaw`(14 段階、23 の理由文)、`Write-ConductorState`(一時ファイル、`Flush($true)`、`File.Move`、ディレクトリの同期は `-SyncDirectory` のスクリプトブロックで受ける)、`Reset-ConductorState`・`Reset-ConductorAllState`、`Test-ConductorStateConsistency` | 06 の全章、10 の 7.2 | 06 の 6.6 節の入力と §4.4 の例が通る。書き込みの原子性(読み手が、直前の完全な内容か新しい完全な内容だけを見る)。`reset --all` の対象の選別。ステートの読み書きが例外を送出しない(失敗は値) | 単体(`State` の行。一時ディレクトリの実ファイル)。AC-302〜324、AC-473、AC-546 |
| 6 | `Convert`: `Convert-ConductorFile`・`Convert-ConductorText`、`ConvertTo-ConductorPsd1Text`(4 スペース、裸のキーの条件、単一引用符、LF のみ、BOM なし)、浮動小数の書き出し、表せない値 | 09 の全章、10 の 7.3 | 09 の 11 章の例が一致する。往復(JSON → `.psd1` → JSON、`.psd1` → JSON → `.psd1`)で木が変わらない(09 章 9 章)。書き出した `.psd1` を `Import-PowerShellDataFile` が同じ値で読める(テストの中だけ)。6 種・17 件の理由文 | 単体(`Convert` の行)。AC-206〜238、AC-469〜472、AC-482、AC-544、AC-584・585 |
| 7 | `Report`: `Write-ConductorStream`(`StreamWriter`、BOM なし UTF-8、LF、`AutoFlush`、`[bool]`)、`Send-Conductor*`、`Complete-ConductorOutput`、ロケール、メッセージカタログ(日本語・英語)、`-v` の行、`watch` の表(表示幅、色)、`--help` の全文、エスケープの適用、`Write-ConductorInternalError` | 08 の全章、10 の 6.1 | 08 章のすべての文言(日本語・英語)が一致する。`--help` の全文が一致する(08 章 8.4 節の検査)。`Write-Host`・`Write-Output`・`[Console]::Out` を使っていない。書き込みの失敗が `[bool]` の偽で返る(`/dev/full` で確かめる) | 単体(`Report` の行)。AC-350〜369、AC-370〜388、AC-493、AC-520 |
| 8 | `Cli` の引数解析と run ID: `Get-ConductorLexResult`、`ConvertFrom-ConductorArguments`(誤りの優先順位)、`Get-ConductorEnvironmentVariable`、`Resolve-ConductorRunId` | 04 の 2〜6 章、10 の 7.4 | 04 の 4.3 節の判定表の全行が単体で通る(`-Verbose` は version、`-?` は `unknown option: '-?'` など)。`$args` を使っていない | 単体(`Cli` の引数の行)。AC-239〜260、AC-325〜349 |
| 9 | `Runner`(ヘルパーなし): `Split-ConductorCommandLine`、`Test-ConductorCommand`、`Get-ConductorPlatformKind`、`Get-ConductorStartInfo`、`Resolve-ConductorUnixExecutable`、`ConvertTo-ConductorCmdArguments`、`Invoke-ConductorJob`、`Wait-ConductorChild`、`Read-ConductorExit`。テストジョブ `tests/jobs/tj.ps1`(11 章 3.4 節) | 07 の 1〜5・9 章、11 の 3.3・3.4 | `command` の分割の受理・拒否の表(07 章 3.5・3.6 節)が通る。起動失敗が値で返る(例外を送出しない)。待機が 10 ms 以下の刻み。UNIX 系で `ExitCode` 129〜159 の推定(AC-504)。OS の判定が `Get-ConductorPlatformKind` だけ(AC-420) | 単体(`Runner` の行)と、実プロセスの単体。AC-076〜116、AC-140、AC-288、AC-504、AC-522 |
| 10 | シグナルと C# ヘルパー: `NativeHelper.ps1`(`HelperInfo`・`Interrupt`・`SignalWatcher`・`UnixNative`)、`Initialize-ConductorNativeHelper`(`Revision` の検査)、`New-ConductorInterrupt`、`Start-ConductorSignalWatch`、`Wait-ConductorRetry`、`Send-ConductorSignal`、`Sync-ConductorDirectory`、停止経路(`Wait-ConductorChild` の `finally`) | 07 の 6〜8・10・11 章、05 の 7.8・16 | 合図が立ってから 1 秒以内に待機が打ち切られる(AC-275)。シグナルの登録が GC で解除されない(AC-537)。コールバックが PowerShell を呼ばない(AC-499・538)。`Add-Type` が、`--version`・`--help`・`validate` などで呼ばれない(AC-489)。再実行で型の再定義の誤りが出ない(AC-558)。ヘルパーを使えないとき `run`・`pipeline` が 70 | 単体(`Runner` の行)と、UNIX 系の実プロセス。AC-261〜279、AC-280〜301、AC-488・489、AC-499、AC-537・538・558 |
| 11 | `Cli` の配線とランチャー: `Deps`(`New-ConductorSystemDeps`、`Test-ConductorDeps`)、`Invoke-ConductorMain`、`Invoke-ConductorPrepare`、`run`・`pipeline`・`watch`・`reset`・`validate`・`convert`、`Invoke-ConductorExecJob`・`Invoke-ConductorJobWithRetry`(停止経路の `finally`)、`Invoke-Conductor`(`-ArgumentList`、作業ディレクトリの合わせ込み、内部の誤りの捕捉)、`bin/conductor`・`bin/conductor.cmd`・`bin/conductor.ps1` | 02 の 6〜7・11 章、04 の 1・7 章、05 の 5 章、10 の 2・6 章 | 11 章の受け入れテストが、ランチャー経由で、3 OS で通る(手動の項目を除く)。`conductor --version` が `conductor 0.8.0`。`Invoke-Conductor` の戻り値が `[int]` 1 つだけ(AC-552)。`exit` が入口スクリプトにしかない(AC-500)。`pwsh` が無いとき 70(AC-492)。Windows の Ctrl+C の手動の確認(AC-301。07 章 7.6 節) | AC-117〜205、AC-389〜396、AC-492・500・507・547・549・552・559・569。偽の `Deps` の単体(AC-293〜300、AC-324、AC-366〜368、AC-485、AC-491) |
| 12 | 品質ゲートの仕上げ・配布・デモ: カバレッジ(行 80%)、性能の計測(目安。起動時間は `pwsh` 自体を除いた値も)、`New-Package.ps1`、`release.yml`、デモ、README(実行ポリシーと `Zone.Identifier` の注意)、`audit.yml` | 11 の 2・5〜8 章 | 11 章 5.5 節の「ローカルでの再現手順」がすべて通る。成果物の `--version` の出力が `conductor <ModuleVersion>`。zip と tar.gz の中身・実行権限・`SHA256SUMS` が合う | AC-397〜423(非機能)、AC-424〜438(デモ)、AC-439〜446(配布)、AC-476〜487、AC-526〜529、AC-562〜573 |

- 段階 4〜6 は、互いに独立している(どれからでも作れる)。段階 7 と 8 も独立している。段階 9 と 10 は、この順。
- 各段階の終わりに、`./build/Invoke-Checks.ps1 -Check lint` と `./build/Invoke-Tests.ps1 -Kind Unit,Inspect` を通す。通ってから次へ進む(PowerShell は、構文の誤り・禁止コマンド・戻り値の混入を、実行する前に静的検査で見つけられる範囲が狭いので、検査を段階ごとに回す)。
- 段階 0 で作る `build/` のスクリプトは、11 章に全文がある(骨格ではなく、そのまま使える形。実機での確認は未確認なので、段階 1 で動かして直す)。
- 型の骨格(`[pscustomobject]` のプロパティの名前)を、先に 02・03・05・06・07・09 章から 1 か所にそろえて(`New-Conductor...` の生成関数)、後の段階が、同じ名前で値を作るようにする。プロパティの綴りの誤りは、`Set-StrictMode` で実行時にしか見つからない。

## 2. PowerShell 特有の落とし穴

この章は、02〜11 章の「注意」「未確認」を、テーマごとに集めたものである(規定は各章)。「対処」は、各章の規定を指す。

### 2.1 起動・引数・終了・内部の失敗

| 落とし穴 | 何が起きるか | 対処(規定) | 章 |
|----------|--------------|--------------|----|
| `pwsh -File` の引数の分割 | `-File` の後ろの引数は、スクリプトに届く前に PowerShell のパラメータの形に変換される。`-` で始まり `:` を含む語は `:` で名前と値に分かれ(`--config=C:\a.psd1` が `--config=C` と `\a.psd1` になりうる)、`-` で始まる語はスイッチ名になり、`--` は特別扱いされない。元の文字列が `$args` に残る保証がない | `$args` を使わない。`[System.Environment]::GetCommandLineArgs()` の、最初の `-File` の後ろを生の引数とする。ランチャーは、`-NoLogo -NoProfile -NonInteractive -File <ps1>` の固定の引数だけを前に置く(`-File` の前に利用者の引数を置くと、`-v` が `-Version`、`-h` が `-Help` として `pwsh` に取られる) | 04 の 1.3〜1.6 |
| `-Command` と `-File` | `-Command` は、後続を PowerShell のコードとして解釈し、終了コードを 0 か 1 に丸める | `-File` だけを使う | 04 の 1.6、11 の 1 |
| 終了コードの範囲 | `exit` は UNIX 系で 0〜255 だけを受け、範囲外は 0 になる(`exit 256` が成功に見える)。`-File` では、例外で終わると 1、Ctrl+C で中断されると 0 | `exit` に 0〜255 の `[int]` だけを渡す(`ConvertTo-ConductorProcessExitCode`)。`[uint64]`・`[double]`・文字列を渡さない | 10 の 4・4.1 |
| `pwsh` 自身の 64・70 | `pwsh` は起動の失敗に 64 と 70 を使うので、`conductor` の 64・70 と同じ数値になる | 標準エラーの `conductor: error: ...` の行で区別する。ランチャーが渡す固定の引数のせいで、64 は通常は現れない | 10 の 1 |
| モジュールの中の `exit` | `exit` は、対話セッションごとプロセスを終了させる | `Invoke-Conductor` は `[int]` を返す。`exit` は入口スクリプトだけ(と、停止経路の `Environment.Exit`) | 02 の 9.1、04 の 1.4 |
| `PipelineStoppedException`(Ctrl+C) | Ctrl+C で、`pwsh` が実行中のパイプラインを止める。`catch` に届くか、`finally` だけが実行されるかは未確認。`finally` の途中の 2 回目の Ctrl+C で、後始末が中断されうる。止まったあとの `-File` の終了コードは 0 | 後始末は `finally` に書く。`catch` は、停止の例外では内部の誤りの行を出さず再送出する。入口スクリプトの `finally` が、ヘルパーの予約した終了コード(なければ 130)で `[System.Environment]::Exit` を呼ぶ | 07 の 7.7、10 の 5・6.4 |
| `$ErrorActionPreference` と StrictMode | 設定しないと、非終了エラーのあとも次の文が実行され、誤った状態で進む。モジュールの関数が、呼び出し元の設定でなく、モジュールの値を使うかは未確認 | モジュールの先頭で `Set-StrictMode -Version Latest` と `$ErrorActionPreference = 'Stop'`。`trap` を使わない(`10-T8` で確かめる) | 02 の 5.2、10 の 6.1 |
| `cmd.exe` の再解釈(Windows) | `%`・`^`・`&`・`\|`・`<`・`>`・`!` と引用符が、`.cmd` の中で別の意味になる。`.\conductor.ps1` と書くと、カレントディレクトリに依存して見つからない | `"%~dp0conductor.ps1"`、`setlocal DisableDelayedExpansion`、`%*`、`endlocal` の前に終了コードを変数に取る。引数の注意を利用者向けに書く(ランチャーでは防げない) | 04 の 1.3.3 |
| 更新通知・テレメトリ | `pwsh` が起動時に出す更新通知が出力を汚す。設定した環境変数が、ジョブにも引き継がれる | `POWERSHELL_UPDATECHECK=Off`・`POWERSHELL_TELEMETRY_OPTOUT=1` を `pwsh` の起動の前に設定する。ジョブに引き継がれることは仕様として受け入れる | 04 の 1.3.2 |
| `$input` | `-File` のスクリプトに `$input` が現れると、ホストが標準入力を読む設定になり、ジョブが標準入力を読めなくなる | 入口スクリプトと、それが読み込むファイルで `$input` を使わない(`04-T7`) | 04 の 1.3.1 |
| 実行ポリシーと `Zone.Identifier` | Windows だけで有効。`RemoteSigned` で、ダウンロードした署名なしの `.ps1` が実行できない。`-ExecutionPolicy` は Windows 以外では無視される | `-ExecutionPolicy Bypass` を付けない。署名もしない。回避手順(`Unblock-File`、展開の方法)を利用者向けに書く | 11 の 6.5 |
| `pwsh` が無い・Windows PowerShell 5.1 | `pwsh` が無いと起動できない。5.1 では、モジュールが動かない | `pwsh` が無いときは、固定の 1 行と終了コード 70(127 ではない)。マニフェストの `PowerShellVersion` で 5.1 の読み込みを拒否する | 04 の 1.3、11 の AC-492・567 |

### 2.2 パイプライン・戻り値・比較・配列

| 落とし穴 | 何が起きるか | 対処(規定) | 章 |
|----------|--------------|--------------|----|
| パイプラインへの出力の混入 | 関数の中で、値を捨てていない文(`HashSet.Add` が返す `bool`、`ArrayList.Add` が返す位置、`StringBuilder.Append` が返す自分自身)の結果が、戻り値に混ざる。`Write-Output`・`Write-Host` の出力も混ざる | 値を返すメソッドは `[void]` か `$null =`。`Write-*` を製品のコードで使わない(PSScriptAnalyzer と構文木の検査)。戻り値は `return` で 1 つ。`Invoke-Conductor` は、戻り値が `[int]` 1 つだけであることを確かめる | 02 の 9.2、05 の 14.4、11 の AC-552 |
| 配列の単一化・展開 | 関数が返す 1 要素の配列は単一の値に、0 要素は `$null` になる。パイプラインは配列を展開する | 配列を返すときは `return ,$array`(または、値のプロパティに入れる)。呼び出し側は `@(...)` で受ける。リストは `[System.Collections.Generic.List[object]]` に `Add` する。`+=` で配列を伸ばさない(作り直しで遅い) | 02 の 3.1、05 の 14.2、04 の 4.0 |
| `-eq` の大文字小文字と型変換 | `-eq`・`-ne`・`-like`・`-match`・`switch` は、既定で大文字小文字を区別しない(`'-V' -eq '-v'` が真)。`-eq` は左辺を右辺の型に変換する。左辺が配列だと、要素の絞り込みになる | `-ceq`、`[string]::Equals($a, $b, [System.StringComparison]::Ordinal)`。`switch` は `-CaseSensitive -Exact` か、使わない。`$null` は左辺(`$null -eq $x`) | 04 の 4.0、05 の 14.1・14.3、11 の 3 |
| 正規表現が既定のもの | `-match`・`-replace`・`-split`・`switch -Regex` は正規表現。`-like` はワイルドカード。`$` が末尾の改行の直前にも一致し、`\d` が Unicode の数字に一致する | 使わず、手書きの走査(整数の範囲の比較)にする。使うなら `[regex]` に `RegexOptions.CultureInvariant` と `\A…\z`、`[0-9]` | 02 の 9.2、03 の 1.1、04 の 5.2 |
| `if ($x)` と `$null` | `0`・空文字列・空配列が偽になり、値の有無を取り違える。型付きの変数・パラメータに `$null` を入れると、`[int]` は 0、`[string]` は空文字列になる | 値の有無は `$null -ne $x`。「無し」を持つ項目は、型を付けず(`[AllowNull()] [object]`)、有る値は値そのもの | 05 の 0.2・14.3 |
| ハッシュテーブル `@{}` | キーの大文字小文字を区別しない(`@{a=1; A=2}` は重複)。順序を保たない。存在しないキーの参照が例外か `$null` かは未確認 | 木のマップは位置つきのエントリの `List`。キーの検索は `Dictionary[string, object]`(`StringComparer.Ordinal`)か `HashSet[string]`。順序が要るときは `[ordered]`。`ContainsKey` で明示的に検査する | 02 の 7.3、03 の 6.1、08 の 10 |
| `ForEach-Object` の `break`・`continue` | スクリプトブロックの中の `break`・`continue` は、ループでなくパイプラインを抜ける | 走査には `foreach` 文か `while` を使い、`ForEach-Object` を使わない | 03 の 5.3、05 の 14.7 |
| スコープ・`$global:` | 変数の可視性を取り違える。`$script:` は、ドットソースしたファイルでも、モジュールのスコープ | `$script:` だけを使い、モジュールの読み込み時に初期化する。`$global:` を使わない | 11 の AC-551 |
| `Set-Location` と .NET の現在のディレクトリ | PowerShell の現在の場所(`$PWD`)と、プロセスの現在のディレクトリは別。.NET の `[System.IO.File]` は、相対パスをプロセスの現在のディレクトリから解決する | `Set-Location` を使わない。`Invoke-Conductor` が、開始時に `[System.Environment]::CurrentDirectory` を `$PWD.ProviderPath` に合わせ、`finally` で戻す | 04 の 1.4 |
| `Join-Path`・`Resolve-Path`・`Test-Path` | `-Path` はワイルドカード(`[`・`*`・`?`)を解釈する。`Resolve-Path` は存在しないパスで失敗する。文字列を変える | ファイルの存在・結合は `[System.IO.File]`・`[System.IO.Path]` で行う。利用者に見せるパスは、元の文字列のまま | 03 の 3、04 の 3.3 |
| `Get-Content`・`Set-Content`・`Out-File` | 既定の文字コードと改行が版・OS で違う。行ごとの配列になる(`-Raw` が無いと改行が失われる) | `[System.IO.File]` と `[System.IO.FileStream]` で、バイト列として読み書きする | 03 の 1.1、06 の 4.3 |
| `$LASTEXITCODE`・`$?` | 取り違えやすい(関数・コマンドレット・外部コマンドで意味が違う) | 製品のコードで読まない(構文木の検査) | 02 の 9.1、11 の AC-421 |
| `Start-Sleep` | Ctrl+C で中断される。合図で起こせない。`-Seconds` は `[double]` | リトライの待機は `Interrupt.Wait(int)`(250 ms 以下の刻み)。`Start-Sleep` を使わない | 05 の 7.8、07 の 8 |

### 2.3 整数・型・数値

| 落とし穴 | 何が起きるか | 対処(規定) | 章 |
|----------|--------------|--------------|----|
| `[double]` への昇格 | 結果が型に収まらないとき、例外でなく、より広い型へ昇格する(`[int] * [int]` が `[double]`。`[ulong]::MaxValue + 1` が `[double]`)。`[double]` を `[ulong]` に戻すと精度が落ちる | リトライの待ち時間の乗算・加算・比較は `[bigint]` で行い、`[uint64]::MaxValue` と比べて飽和させる。`[uint64]` の `*`・`+` の結果をそのまま使わない | 05 の 0.3・7.6 |
| 数値リテラルの型の自動選択 | 接尾辞のない 10 進整数は `int` → `long` → `decimal` → `double` の順。`18446744073709551615` は `decimal` になる。`kb`・`mb` で値が変わる | `[uint64]` の定数は `18446744073709551615ul` か `[uint64]::MaxValue`。`.psd1` の数は、`Extent.Text` を自前で解析する(構文木の `.Value` に頼らない) | 03 の 4.3.1、05 の 0.3 |
| `/` と `[int]` のキャスト | `/` は商が整数でなければ `[double]`。`[int]` へのキャストは銀行丸め(`[int](5/2)` が 2) | `[bigint]::Divide`。整数は字句から自前で数える | 05 の 0.3 |
| 文字列から数値へのキャスト | `0x`・`&h`・`#`・`0b` の接頭辞を受理する。不正な字句が通る | 字句を、ASCII の `0`〜`9` の走査で検査してから変換する | 05 の 0.3・7.3 |
| `-band` と負の数 | 負の `[int]` で 2 の補数の下位 8 ビットを返すことが未確認 | `05-T-2` で確かめる。外れたら、`[long]` に広げた剰余の式(`(([long]$r % 256) + 256) % 256`)に切り替える | 05 の 2.2、10 の 4 |
| 文字の比較 | `[char]` どうしの `-eq`・`-ge` が、大文字小文字を区別しない文字列の比較になりうる | `[int]$c`(コードユニット)の整数の比較 | 05 の 6.1、06 の 4.3 |
| 整数の範囲検査 | JSON の整数の字句が `[long]` や `[int]` に収まらない | 字句の桁数で範囲外(`hugeInt`)を判定してから変換する。範囲を確かめてから `[int]` にキャストする | 03 の 5.1、06 の 6.5 |

### 2.4 文字列・文化・文字コード

| 落とし穴 | 何が起きるか | 対処(規定) | 章 |
|----------|--------------|--------------|----|
| UTF-16 とサロゲート | `$s[$i]`・`.Length` はコードユニット単位。U+10000 以上の文字は 2 つ。`foreach ($c in $s.ToCharArray())` で 1 文字ずつ見ると、対を壊す | 自前の走査で、サロゲートの対を 1 文字(コードポイント)として扱う。対になっていないサロゲートは、読める形(U+FFFD)にしてから表示する | 03 の 11、04 の 4.0・4.4、08 の 11 |
| 文化依存の文字列 API | `StartsWith('x')`・`IndexOf('=')`・`Trim()`・`ToUpper()`・`ToString()`・`-f`・文字列補間が、文化・Unicode のデータに依存しうる | `StringComparison.Ordinal`、`ToUpperInvariant`、`.ToString([System.Globalization.CultureInfo]::InvariantCulture)`。空白は自前の表(25 文字)。禁止コマンドの検査が強制する | 03 の 1・1.1、04 の 4.0、08 の 10 |
| ロケールの決め方 | `$PSCulture`・`CurrentUICulture` は環境変数 `LANG` などを反映するとは限らない | 環境変数 `LC_ALL`・`LC_MESSAGES`・`LANG` の最初の空でない値が `ja` で始まれば日本語。`$PSCulture` を使わない | 08 の 7 |
| 文字コードと BOM | `[System.Text.Encoding]::UTF8` が BOM を出す可能性がある。標準出力の既定のエンコーディングは、環境に依存する | `[System.Text.UTF8Encoding]::new($false, $true)`(BOM なし。不正な文字で例外)。標準出力の `StreamWriter` に明示する。ステートは先頭の BOM を拒否、定義・変換は読み飛ばす | 06 の 4.3・6.1.2、08 の 1.2 |
| 色 | `$PSStyle` が `NO_COLOR` や `TERM` で変わる | 自前の ANSI。`NO_COLOR` が空でなければ色なし。端末でないとき、Windows のとき色なし | 08 の 6.3 |
| 不正な UTF-8 の引数・環境変数 | `pwsh`(.NET)が `[string]` にどう変換するか(U+FFFD か、例外か、`pwsh` の起動前の失敗か)が未確認 | 対になっていないサロゲートか U+FFFD を含む文字列を「文字として正しくない」とみなして `invalid value`(68)。`--config` のパスは検査せず通す(開けない場合がある) | 04 の 4.4.2 |

### 2.5 `.psd1`・JSON・ファイル

| 落とし穴 | 何が起きるか | 対処(規定) | 章 |
|----------|--------------|--------------|----|
| `.psd1` を `Import-PowerShellDataFile` で読む | 位置(行・列)を返さない。誤りの種類を区別できない | `Parser::ParseInput` で構文木を作り、許可するノードの型だけを自前で木に変換する(コードを実行しない)。`Import-PowerShellDataFile` は比較用のテストだけ(と、`Conductor.psm1` が版を読む 1 か所) | 03 の 4・4.11 |
| 構文木の値 | `ConstantExpressionAst.Value` の型が値で変わる(`1L`・`1kb`・`0b1`)。`ParseError.Message` は翻訳され、入力の値を含む。U+2018・U+2019 が単一引用符として読まれる。`@(@(1,2))` が実行時に平坦化される。単項のカンマ。深い入れ子で、パーサーの再帰が溢れうる | `Extent.Text` を自前で解析する。理由は固定の文だけ。入れ子は、括弧の深さを事前走査して拒否する(1,000)。`@(@(...))` は拒否し、単項のカンマで 1 要素の入れ子を書く。木の走査も明示のスタック | 03 の 4・6.4、09 の 8 |
| `ConvertFrom-Json` | 重複キーは最後だけ残る。大文字小文字だけが違うキーが同じになる(`-AsHashtable` なしで)。時刻の形の文字列が `[datetime]` になる。コメントを受け付ける(6 以降)。1 要素の配列がパイプラインで展開される。64 ビットを超える整数・`NaN`・BOM の扱いが版で違う。位置情報がない | 使わない。自前の読み取り(明示のスタック、重複キーの位置、`hugeInt`、深さ)を、定義・変換・ステートで共用する | 03 の 5、06 の 6.5 |
| `ConvertTo-Json` | `-Depth` の既定が 2(深いと切り捨て)。キーの順序が記述順でない。LF 固定・末尾 LF・空配列の書き方・エスケープの細部が版で変わる | 使わない。自前の書き出し(インデント 2、LF、エスケープ表を固定) | 06 の 4.3、09 の 7 |
| `Utf8JsonReader` | `ref struct` で、PowerShell の変数に保持できない(未確認) | 土台にしない | 03 の 5、06 の 6.5 |
| ファイルの原子的な置き換え | `Set-Content` は、その場で書き換える。別のディレクトリの一時ファイルへの移動は、コピーと削除になる | 同じディレクトリに `FileMode.CreateNew` で一時ファイルを作り、`Flush($true)`、`Dispose()`、`[System.IO.File]::Move($src, $dst, $true)`。UNIX 系はディレクトリを `fsync`(C# ヘルパー。失敗は無視) | 06 の 5 |
| 例外の包み | .NET メソッドの例外が `MethodInvocationException` に包まれ、`catch [System.IO.IOException]` が一致しないことがありうる | `Get-ConductorBaseException` で元の例外の型を取り出して判定する(`06-T-8` で確かめる)。例外の `Message` を表示に使わない | 06 の 5 |
| 大きなファイル・特殊なファイル | 上限 1 MiB を超える入力、FIFO、デバイス、シンボリックリンク | 上限 + 1 バイトで読むのを止める。通常のファイルかを属性で確かめる(`03-T-14`、`09-T-C01`)。`reset --all` はシンボリックリンクを消さない | 03 の 3、06 の 8.2 |

### 2.6 `class`・モジュール・テスト

| 落とし穴 | 何が起きるか | 対処(規定) | 章 |
|----------|--------------|--------------|----|
| `class`・`enum` | セッションの中でアンロード・再読み込みできない(`Import-Module -Force` で更新されない)。ドットソースしたファイルの `class` は、モジュールの外に自動では公開されない。クラスのメソッドは、作った runspace に結び付く(別スレッドから呼ぶと危険) | 製品のコードで `class`・`enum` を定義しない。値は `[pscustomobject]`、種別は文字列(`-ceq`)。型は `Verb-ConductorNoun` の生成関数で作る | 02 の 5.3、05 の 0.2、10 の 8 |
| 依存の向きを言語が強制しない | 関数は、読み込まれたすべてが、モジュールのスコープから呼べる | `tests/Inspect/` の AST の走査(関数名 → ディレクトリの表、辺の許可、循環、動的な呼び出しの禁止)で強制する。`Set-Alias` を使わない | 02 の 4.1 |
| 読み込みの順序・`$PSScriptRoot` | ドットソースしたファイルの `$PSScriptRoot` の値が未確認。ファイルが多いと `Import-Module` が遅い | 読み込みの順序を、固定の配列と序数のソートで決める。必要なら、ビルドで 1 つの `.psm1` に結合する(起動時間の計測で決める)。`FunctionsToExport` にワイルドカードを使わない | 02 の 5.2 |
| `Import-Module -Force` と `Add-Type` | 型は、セッションの中で取り除けず、変えられない。ソースを書き換えて `-Force` しても、古い型が残る(エラーなし) | `HelperInfo.Revision` を、ソースを変えるたびに増やし、読み込み済みの型と比べる | 07 の 7.3 |
| Pester 5 | 発見(Discovery)と実行(Run)の段階が分かれ、変数の可視性が違う。`Mock` は、モジュールの非公開の関数に `-ModuleName` か `InModuleScope` が要る。`PesterConfiguration` のキーが 5.x にあるか、`-CI` の挙動、目標の率による失敗が未確認 | `Deps`(スクリプトブロックのハッシュテーブル)で口を差し替え、関数の呼び出しの検査が要る所だけ `Mock`。カバレッジは、JaCoCo の XML から自前で率を計算して判定する。`11-T10` で確かめる | 02 の 7.2、11 の 3.9・5.3 |
| PSScriptAnalyzer | 規則名の接頭辞、設定のキー、`Invoke-Formatter` の冪等性・CRLF の扱いが未確認。純粋な関数の `New-`・`Set-`・`Reset-` が `UseShouldProcessForStateChangingFunctions` に掛かる | 設定で `ExcludeRules` に置く。`Invoke-Formatter` は文字列を受けて文字列を返す(ファイルを直さない)。`11-T11` で確かめる | 02 の 3.1、11 の 9.2 |

### 2.7 プロセス・シグナル・`Add-Type`

| 落とし穴 | 何が起きるか | 対処(規定) | 章 |
|----------|--------------|--------------|----|
| スクリプトブロックを別スレッドのコールバックにする | シグナル・イベントのコールバックが、runspace の無いスレッドで呼ばれ、失敗する(`There is no Runspace available to run scripts in this thread`)。`Register-ObjectEvent` の `-Action` は、別の runspace で動き、メインのスクリプトの実行中は処理されない場合がある | コールバックは C# ヘルパーの中で完結させる(`PosixSignalRegistration`・`Console.CancelKeyPress`)。PowerShell は `IsRaised` と `Number` を読むだけ | 07 の 7.1 |
| シグナルの登録が GC で解除される | `PosixSignalRegistration` はファイナライザで解除される | ヘルパーの静的なフィールドに保持する。`SIGCHLD` を登録しない | 07 の 7.5 |
| `Add-Type` の起動の費用と制約 | Roslyn でコンパイルし、数百 ms かかりうる(未測定)。Constrained Language Mode・AppLocker で失敗する。警告をエラー扱いにする | 最初のジョブの起動までに遅延させる(`--version`・`--help`・`validate` などでは呼ばない)。失敗したら、`run`・`pipeline` は 70(シグナルを受けずにジョブを起動しない)。`-TypeDefinition` で、名前空間を明示する | 07 の 7.3・7.8、11 の 2.1 |
| Ctrl+C が二重に届く | `pwsh` 自身のハンドラ(UNIX 系は SIGINT・SIGQUIT、Windows は break ハンドラ)と、ヘルパーのハンドラの両方が働き、メインのスクリプトが `PipelineStoppedException` で止まりうる。.NET のメソッドの呼び出しの途中では割り込まれず、戻ったあとの文の境目で届く | 待機は 10 ms 以下の `WaitForExit(int)`。停止の有無にかかわらず、同じ結果(子の終了を待つ、エラーとして記録、`128 + 番号`)にする。`finally` の中で PowerShell のコマンドレットを呼ばない | 07 の 7.5〜7.7 |
| `Process.ExitCode` とシグナル | 子がシグナルで死んだときの `ExitCode`(`128 + n` と推定。未確認)。`exit 137` と区別できない。Windows では、例外コードが負の値になる | UNIX 系で `ExitCode` が 129〜159 で、成功・警告のコードに含まれないときだけ、シグナル終了とみなす。下位 8 ビットで整形する | 07 の 4.4、10 の 3 |
| 起動の失敗 | `Process.Start` が `Win32Exception` を投げる(見つからない・権限・ディレクトリ・形式が不正)。`Message` は環境で変わる | 起動に関する例外の型だけを狭く捕まえて、`StartFailure` にする(127)。原因を区別せず、`Message` を表示しない。`[System.Exception]` 全体を捕まえない | 07 の 4.3、10 の 3 |
| 実行ファイルの探索 | `Get-Command` の探索(別名・関数・`PATHEXT`)と、`Process.Start` の探索が食い違う | UNIX 系は `PATH` を自前で探索して絶対パスに解決する(空のエントリはカレントディレクトリ)。Windows は OS の標準に任せ、`Get-Command` を使わない。`.cmd`・`.bat` は、禁止の文字を拒否して `cmd.exe /d /s /c` を `Arguments` で直接起動する | 07 の 4.8・4.9 |
| 子の標準入出力 | 親のバッファが残ったまま子を起動すると、出力の順序が入れ替わる。`WaitForExit()`(無期限)は、リダイレクトした非同期の出力の完了も待つ | `Redirect*` をすべて `$false` にして引き継ぐ。子を起動する前に、自分の出力を `Flush()` する。待機は `WaitForExit(int)` | 07 の 4.2・5.2、08 の 1.3.2 |
| 同じプロセスグループ | UNIX 系で、ジョブを新しいグループに置く手段がない。端末の Ctrl+C が、ジョブに直接届き、転送でもう 1 回届く | 新しいグループ・セッションに置かない(仕様)。`kill` は子の pid だけ(正の値)、許可したシグナル(1・2・3・15)だけ、1 回だけ | 07 の 4.7・7.5 |
| 環境変数 | `pwsh` が起動時に環境変数を足すことがあり、子の環境が `conductor` の起動前と完全には同じでない。`$env:` に空文字列を代入すると変数が削除される | `$env:` に代入しない。環境の読み取りは `Get-ConductorEnvironmentVariable` の 1 か所 | 04 の 5.1、07 の 4.2 |

### 2.8 テスト・CI・配布

| 落とし穴 | 何が起きるか | 対処(規定) | 章 |
|----------|--------------|--------------|----|
| テストの遅さ | 受け入れテストは、1 件ごとに `pwsh` を起動する(起動が遅い) | プロセスを起動するテストは 60 秒の上限つき(`Slow` は 120 秒)。受け入れ(`Acceptance`)と単体(`Unit`)を分けて回す。起動時間は、`pwsh` 自体を除いた値も測る | 11 の 2.1・3.3 |
| GitHub のランナー | macOS・Windows のランナーの `pwsh`・Pester・PSScriptAnalyzer の版が未確認。`shell: pwsh` の複数行の失敗の扱い、ステップをまたぐ `PATH` の更新が未確認 | `build/Resolve-Pwsh.ps1` で、`MinPowerShell`・`MaxPowerShell` に合う `pwsh` を使う。依存は固定の版とハッシュで入れる | 11 の 5 |
| zip と tar.gz | zip は UNIX の実行権限を保てない。展開で `Zone.Identifier` が引き継がれるかが、道具によって違う | UNIX 系は tar.gz を使う。Windows は zip と `.cmd`。`SHA256SUMS` を付ける | 11 の 6.1・6.5 |
| `ModuleVersion` と先行版 | `ModuleVersion` は 3 つの数字で、先行版の接尾辞を持てない | 先行版は `PrivateData.PSData.Prerelease` に書き、リリースの注記に固定の 1 文を付ける | 11 の AC-445 |
| PowerShell Gallery | ギャラリーに出すのはモジュールだけで、`bin/` のランチャーは含まれない | ワークフローは公開しない(人が手で行う)。扱いは未検討 | 11 の 6.1 |

## 3. 未確認事項の一覧

この仕様書を書いた環境には `pwsh` がなく、次は確かめられていない。実装の最初に、4 章の試験で確かめ、結果で該当の章を直す。

### 3.1 起動・引数・終了コード・モジュール

| 内容 | 関係する章・試験 |
|------|------------------|
| `GetCommandLineArgs()` が、`pwsh` の受け取った引数を、変換や欠落なしに返すか。先頭の要素の中身(実行ファイル名か `pwsh.dll` のパスか)。ダメなら、接頭辞方式(04 章 1.5 節) | 04 の 1.5。`04-T1` |
| `Invoke-Conductor -ArgumentList` の配列の要素(`-v`・`--`・`-?`・`-Verbose`)が、パラメータとして解釈されないか。`[CmdletBinding()]` を付けたときの `-v` の束縛 | 04 の 1.4。`04-T5`、`11-T1`・`11-T2` |
| 終了コード: `exit` の値が、そのまま `pwsh -File` の終了コードになるか。`$global:?` の検査が上書きしないか。`exit 256`・負の値の Windows での値。例外で 1 | 10 の 4.1。`04-T2`・`04-T3` |
| `$input` を使わないとき、子が標準入力を読めるか | 04 の 1.3.1。`04-T7` |
| `Import-PowerShellDataFile` が、マニフェストを読めるか(`ModuleVersion` が文字列で返るか) | 02 の 10、04 の 8。`04-T6` |
| `$ErrorActionPreference = 'Stop'` と `Set-StrictMode` が、モジュールの関数の内側で、呼び出し元と独立に効くか。.NET メソッドの例外が外側の `try`/`catch` まで伝わるか。ハッシュテーブルの存在しないキーの参照が、例外か `$null` か | 02 の 5.2・9.1、10 の 6.1。`10-T8` |
| ドットソースした `.ps1` の中の `$PSScriptRoot` の値。結合した `.psm1` の起動時間 | 02 の 5.2。`11-T12` |
| `pwsh -File` の直後の、`$PWD` と `[System.Environment]::CurrentDirectory` の一致 | 04 の 1.4。`06-T-9` |
| 更新通知が、非対話の `-File` でも出るか。`readlink` によるシンボリックリンクの解決の移植性 | 04 の 1.3.2 |

### 3.2 Ctrl+C・シグナル・`Add-Type`・子プロセス

| 内容 | 関係する章・試験 |
|------|------------------|
| `PipelineStoppedException` が `catch` で捕まるか、`finally` だけが実行されるか。`finally` の中で、PowerShell のコマンドレット・.NET の呼び出し・`[System.Environment]::Exit` が動くか。停止のあとの `-File` の終了コード(文書は 0。ソースの読み取りと食い違う可能性) | 07 の 7.7、10 の 5。`04-T4`、`07-T-1`・`07-T-4`・`07-T-7`、`11-T3` |
| `PosixSignalRegistration` で SIGINT に `Cancel = true` を返したとき、.NET が `Console.CancelKeyPress` のハンドラを呼ぶか。`pwsh` が SIGTERM・SIGHUP を処理するか(読んだソースの範囲では見当たらない) | 07 の 7.5。`07-T-1` |
| `Add-Type` の所要時間、`-ReferencedAssemblies` なしで参照が解決するか、一時ファイルの有無、型の再定義(同名の型を再コンパイルしない)、Constrained Language Mode と AppLocker での失敗の様子 | 07 の 7.3。`07-T-2`・`07-T-11`、`11-T4`・`11-T5` |
| `NativeLibrary` の候補(`libc.so.6`・`libc`・`/usr/lib/libSystem.B.dylib`)で `kill`・`open`・`fsync`・`close` が解決するか。arm64 の macOS で可変長引数の `open` を 2 引数で呼べるか | 07 の 7.2。`07-T-3` |
| シグナルで死んだ子の `Process.ExitCode` が `128 + n` か。`SIGCHLD` を無視して起動された場合 | 07 の 4.4。`11-T6`、AC-140・504 |
| `catch [System.ComponentModel.Win32Exception]` が `Process.Start` の失敗を捕まえるか(`MethodInvocationException` に包まれるか)。NUL を含む引数の例外の型 | 07 の 4.3。`07-T-6`、`06-T-8` |
| `WaitForExit(10)` の最中の Ctrl+C が、戻りのあとに届くまでの遅れ。シグナルのコールバックの実際のスレッド。複数のシグナルが同時に届いたときの「最初の番号」 | 07 の 5・10。`07-T-8` |
| 環境変数の引き継ぎで `pwsh` が足す変数。子の作業ディレクトリの引き継ぎ(`WorkingDirectory` を設定しない場合) | 07 の 4.2。`07-T-10` |
| 約 30 万引数の `command` の分割の時間 | 07 の 3。`07-T-9` |

### 3.3 `.psd1`・構文木

| 内容 | 関係する章・試験 |
|------|------------------|
| `ParseInput` の `[ref]` の出力引数(`ParseError[]`・`Token[]`)。空・コメントだけの入力の `EndBlock` の状態。`StartOffset` の単位(UTF-16 の添字か)と、`StartLineNumber`・`StartColumnNumber` の一致。BMP 外の文字を含む行の列 | 03 の 4・11。`03-T-01`〜`03-T-03`、`03-T-18` |
| 深い入れ子(`@(` を 1,000・10,000・100,000 並べる)で、パーサーが異常終了しないか。事前走査の上限(1,000)の妥当性 | 03 の 4.8・6.4。`03-T-04`、`11-T7` |
| 数の字句: `1e999`・`1e-999`・範囲を超える 16 進・`1_000`・`.5`・`5.`・`01`・`1kb`・`-0.0`・`-9223372036854775808` の `ParseError` の有無・`Extent.Text`・負の数が `ConstantExpressionAst` か `UnaryExpressionAst` か | 03 の 4.3.1。`03-T-05`、`11-T7` |
| 文字列: `"a$"`・`"$5"`・`` "`$a" ``・`"${x}"` の分類。here-string の `Value`。`` `u{D800} ``・`` `u{110000} `` の扱い。U+2018・U+2019 の扱い。`name =` の次の行に値を書く場合 | 03 の 4.4・4.7。`03-T-06`〜`03-T-08`、`03-T-10`、`09-T-C05` |
| 裸のキー(`a-b`・`a.b`・`1abc`・`if`・`true`)の分類と予約語の一覧の十分さ。単項のカンマ(`@(,@(1))`) | 03 の 4.5・4.6、09 の 8.3。`03-T-09`、`09-T-C04`・`09-T-C09` |
| `.psd1` のハッシュテーブルがキーの大文字小文字を区別しないこと。`Import-PowerShellDataFile` との往復(`-9223372036854775808`・`1e16`・`-0.0`) | 03 の 4.5・4.11、09 の 5。`03-T-11`、`09-T-C03`、`11-T8` |
| Constrained Language Mode で `Parser` の型の呼び出しができるか | 03 の 15。`03-T-19`、`11-T5` |

### 3.4 JSON・文字列・数値・ファイル

| 内容 | 関係する章・試験 |
|------|------------------|
| 自前の JSON の読み取りの速度(1 MiB の `[` の列・長い文字列・多数の短い要素)。性能の基準値は仮 | 03 の 5.3、06 の 6.5。`03-T-16`、`06-T-12`、`09-T-C08` |
| `Utf8JsonReader`(`ref struct`)を PowerShell から使えないこと | 03 の 5。`03-T-13` |
| `FileInfo`・`File.GetAttributes` の、シンボリックリンク・FIFO・デバイスファイル・`ENOTDIR` での結果。FIFO を `FileStream` で開いたときのブロック。UTF-8 の厳格な復号が、過長・サロゲート・範囲外・単独の `FF` で例外を投げること | 03 の 3、09 の 3。`03-T-14`・`03-T-15`、`09-T-C01` |
| `FileStreamOptions` のプロパティを PowerShell から設定して `FileStream` を作れること(`UnixCreateMode`)。`FileStream.Write` の 3 引数の過負荷の選択。`FileMode.CreateNew` の `IOException`。`Flush($true)` が `fsync`・`FlushFileBuffers` を呼ぶこと。`File.Move(overwrite)` の原子性 | 06 の 5。`06-T-10`・`06-T-11` |
| `[bigint]` から `[uint64]` へのキャスト。`-band 0xFF` が負の `[int]` で 2 の補数の下位 8 ビットを返すこと。`return , @()` の単一化。`[int]$x = $null`。`[bigint]` の比較・`Divide`・`Min`。`[int]$s[$i]`。`[double]::Parse("1e999999999")` の結果と時間 | 05 の 17、03 の 4.3.1。`05-T-1`〜`05-T-6`、`03-T-12` |
| `[System.Array]::Sort` とスクリプトブロックの比較の安定性。全 `[char]` の空白の判定(25 文字) | 03 の 9・1。`03-T-17`・`03-T-20` |
| 浮動小数の書き出しの往復(最短の桁)。`.psd1` と JSON の往復の試験(ランダムな木) | 09 の 6.1・9。`09-T-C02`・`09-T-C07` |

### 3.5 出力・ロケール

| 内容 | 関係する章・試験 |
|------|------------------|
| 閉じたパイプへの書き込み: UNIX 系では `EPIPE` が捨てられ、`watch`・`convert` が 65・67 でなく 0 で終わりうる(既知の差)。Windows で `IOException` になるか。`/dev/full` で例外になるか。標準出力が閉じられている場合の `OpenStandardOutput()` | 08 の 1.3。`11-T9`、AC-493 |
| `StreamWriter` を `Dispose` しないときのバッファの流し直し。標準ストリームに書いた `\n` が、Windows で `\r\n` に変わらないか | 08 の 1.2・1.3、09 の 1。`09-T-C06` |
| Windows のコンソール・パイプ・ファイルへの出力の文字コード(`OutputEncoding` の設定と復元)。`IsOutputRedirected` の実装(特に Windows、疑似端末) | 08 の 1.2.2・6.3。AC-434・520 |
| 不正な UTF-8 の引数・環境変数・ファイル名が `[string]` にどう入るか | 04 の 4.4.2。`04-T1`(拡張)、AC-406・519 |

### 3.6 Windows

| 内容 | 関係する章・試験 |
|------|------------------|
| `.cmd` の `%*` の展開と、引用符・`&`・`^`・`!` の扱い。`ERRORLEVEL` と `pwsh` の終了コードの一致。バッチの Ctrl+C の確認。空文字の引数 `""` が空の要素として入るか | 04 の 1.3.3、`11-T13` |
| `cmd.exe /d /s /c` を `Arguments` 文字列で起動した結果(`.cmd`・`.bat`。禁止の文字の集合) | 07 の 4.8、AC-486・510 |
| Ctrl+C: ジョブと `conductor` の両方への配送、終了コード 130、子が残らないこと。Ctrl+Break とコンソールを閉じた通知(対象外) | 07 の 7.6、AC-301(手動) |
| `Process.ExitCode` の符号(例外コード)、ファイルの置き換え(`File.Move`)が他のプロセスの読み取り中に失敗するか、ACL | 06 の 5.1、07 の 4.4 |

### 3.7 道具・CI・配布・性能

| 内容 | 関係する章・試験 |
|------|------------------|
| Pester 5 の `PesterConfiguration` のキー、`-CI`、`-FullNameFilter`、`Mock -ModuleName`・`InModuleScope`、`Result` オブジェクトの名前、JaCoCo の出力の形、目標の率による失敗 | 11 の 1・5.3。`11-T10` |
| PSScriptAnalyzer の設定のキー、規則名の `PS` 接頭辞、`Invoke-Formatter` の冪等性・CRLF の扱い、`-EnableExit` | 11 の 9.2。`11-T11` |
| GitHub の macOS・Windows ランナーの `pwsh`・モジュールの版。`shell: pwsh` の失敗の扱い。ステップをまたぐ `PATH`。`pwsh` の公式の成果物の名前とハッシュ。`Save-Module` の保存の形とハッシュの再現性。`Test-ModuleManifest` の検査の範囲 | 11 の 5・8。`11-T15` |
| zip・tar.gz の展開で `Zone.Identifier` が引き継がれるか。macOS の Gatekeeper。zip の UNIX 権限 | 11 の 6.5。`11-T14` |
| 性能の基準値(起動 p95、定義の読み込み、待機中の CPU 時間など)は、すべて仮で、測っていない。`pwsh` 自体の起動と、`Import-Module`、`Add-Type` の費用 | 11 の 2.1。`11-T12`、AC-397〜403・488 |
| 公式ドキュメントの版: この仕様書の確認の多くは PowerShell 7.4 版の文書による。7.6 で同じかは、実装時に確かめる | 各章 |

## 4. 最初に確かめる小さな試験

実装の最初(段階 1)に、実機の `pwsh`(最小と最新の版。分かれていないときは 1 つ)で、3 OS で行う。結果(版・OS・実際の値)を、この表の「結果」の欄(各自の記録。この文書には書かない)に残し、期待と違ったら、「直す章」を直す。表は、章ごとの試験の番号を、テーマごとに並べ直したものである。同じ内容の試験が複数の章にあるときは、「元の番号」に並べた。

「段階」は、その試験を行う時期。「1」は段階 1(最初の日)、そのほかの数は、その段階の最初に行う。

### 4.1 起動・引数・終了コード・モジュール

| 元の番号 | 試験 | 期待 | 外れたときに直す章 | 段階 |
|----------|------|------|--------------------|------|
| `04-T1`、`07-T-5`、`11-T1`(前半) | ランチャー経由で、`-v`、`-vh`、`--`、`--config=C:\a.psd1`、`--config=a:b`、`--opt=value`、`-`、空文字、日本語、絵文字を渡し、入口スクリプトが取り出した配列(`GetCommandLineArgs()`)と `$args` の両方を、1 要素ずつ表示する | 取り出した配列が、渡した文字列と完全に一致する | 04 の 1.5(接頭辞方式に切り替える) | 1 |
| `04-T5`、`11-T1`(後半)、`11-T2`、`07-T-5` | `Invoke-Conductor -ArgumentList @('-v', '--', '-?', '-Verbose', 'x')` で、要素が保たれる。`$args` が空でないときの検出。`function f { $args }`(`CmdletBinding` なし)を `f -Verbose -v -?` で呼ぶ | 配列の要素が文字列のまま届く。共通パラメータに食われない | 04 の 1.4、02 の 6.1 | 1 |
| `04-T2` | 入口スクリプトから `exit 0`・`1`・`255`・`256`・`-1` を呼び、`pwsh -File` の終了コードをシェルで読む | UNIX 系は 0〜255(256 は 0、-1 は 255)、Windows は 32 ビットの値 | 10 の 4 | 1 |
| `04-T3` | 入口スクリプトの `try` の外で例外を投げ、終了コードを読む | 1 | 10 の 6 | 1 |
| `04-T7` | 入口スクリプトが `$input` を使わないとき、子プロセスが標準入力を読める(`cat` など) | 読める | 04 の 1.3.1 | 1 |
| `04-T6` | `Import-PowerShellDataFile` でマニフェストの `ModuleVersion` を読む | 文字列 `'0.8.0'` | 02 の 10、04 の 8(定数 + 検査に切り替える) | 0 |
| `10-T8` | モジュールの関数の内側で、`$ErrorActionPreference = 'Stop'` と `Set-StrictMode -Version Latest` が、呼び出し元(`SilentlyContinue` を設定したセッション)と独立に効く。.NET メソッドの例外が外側の `try`/`catch` まで伝わる。ハッシュテーブルの存在しないキーの参照が、例外か `$null` か | 効く。伝わる。結果を記録する | 02 の 5.2、10 の 6.1 | 1 |
| `11-T12` | `pwsh -NoProfile -Command exit` と `--version` の起動時間、`Import-Module` だけの時間を 3 OS で測る(結合した `.psm1` との差も) | 2.1 節の基準値(仮)の見直し | 11 の 2.1 | 12 |
| `11-T13` | `bin/conductor.cmd` に、`%`・`^`・`&`・`\|`・`"` を含む引数を渡す | AC-563 の期待の確定 | 04 の 1.3.3 | 11 |

### 4.2 Ctrl+C・シグナル・`Add-Type`・子プロセス

| 元の番号 | 試験 | 期待 | 外れたときに直す章 | 段階 |
|----------|------|------|--------------------|------|
| `04-T4`、`07-T-1`、`07-T-7`、`05-T-7`、`10-T4`、`11-T3` | `pwsh -File` の実行中のスクリプトで、ヘルパーのハンドラを登録して待つ。端末の Ctrl+C と `kill -INT`・`kill -TERM` を送る。終了コード、`finally` の実行、`PipelineStoppedException` の捕捉の可否、`IsRaised` が立つか、`Cancel = true` で `pwsh` のハンドラが抑えられるか、`Interrupt.Wait(int)` が停止の要求で割り込まれないか | 10 の 5・6.4、07 の 7.5〜7.7 の記述どおり。記録する | 10 の 5、07 の 7.7 | 1 |
| `07-T-4` | `try { ... } finally { [System.Environment]::Exit(7) }` が、Ctrl+C による停止のあとに、終了コード 7 で終わる | 7 | 07 の 7.7 の 3(入口スクリプトの停止経路) | 1 |
| `07-T-2`、`11-T4` | 空の C# ヘルパーと、実際のヘルパーの `Add-Type` の所要時間。`-ReferencedAssemblies` なしで参照が解決するか。`Import-Module -Force` の 2 回で、型の再定義の誤りが出ないか | コンパイルの費用の記録。参照が解決する。誤りなし(古い型が残る) | 07 の 7.3、11 の startup-bench | 1 |
| `11-T5`、`07-T-11`、`03-T-19` | `$ExecutionContext.SessionState.LanguageMode = 'ConstrainedLanguage'` のあとの `Add-Type`・`[Parser]::ParseInput`・.NET の静的メソッド。AppLocker・WDAC のある Windows | 失敗の様子の記録。`run`・`pipeline` が 70 | 07 の 7.3・7.8、03 の 15 | 1 |
| `07-T-3` | `NativeLibrary.TryLoad` の候補で、`kill`・`open`・`fsync`・`close` が解決する。arm64 の macOS で、可変長引数の `open` を 2 引数で呼べる | 解決できる | 07 の 7.2 | 10 |
| `11-T6`(AC-140) | `[System.Diagnostics.Process]` で `sh -c 'kill -9 $$'` を起動し、`ExitCode` を見る。`Redirect* = $false` の継承 | 137(`128 + 9`) | 07 の 4.4(推定の確定) | 9 |
| `07-T-6`、`06-T-8` | `catch [System.ComponentModel.Win32Exception]` が `Process.Start` の失敗を捕まえるか。`catch [System.IO.IOException]` が `MethodInvocationException` に包まれた例外にも一致するか。一致しないときの `InnerException` の取り出し | 結果を記録し、捕まえ方を決める | 07 の 4.3、06 の 5 | 5・9 |
| `07-T-8` | `$child.WaitForExit(10)` の最中の Ctrl+C が、呼び出しの戻りのあとに届くまでの遅れ | 刻みの値の確認 | 05、07 の 5 | 10 |
| `07-T-9` | 約 30 万引数の `command` の分割にかかる時間。`List[string]` への追加と `StringBuilder` の速度 | 性能の目安 | 07 の 3.2 | 9 |
| `07-T-10`、`06-T-9` | 子の環境変数が、起動前の環境と等しいか(`pwsh` が足す変数の一覧)。`Get-Location -PSProvider FileSystem` の `ProviderPath` が、PowerShell の現在の場所と一致するか。`Set-Location` が、プロセスの現在のディレクトリを変えるか | 結果を記録する | 04 の 1.4、07 の 4.2 | 9 |

### 4.3 `.psd1`(構文木)

| 元の番号 | 試験 | 期待 | 外れたときに直す章 | 段階 |
|----------|------|------|--------------------|------|
| `03-T-01`〜`03-T-03`、`03-T-18` | `ParseInput` を `[ref]` で呼び、`ParseError[]` と `Token[]` が得られる。空・コメントだけの入力の `EndBlock` と `Statements.Count`。`StartOffset` が入力の `string` の添字(UTF-16)であること。LF だけの入力で `StartLineNumber`・`StartColumnNumber` が `StartOffset` から求めた値と一致する。`.psd1` の文法の誤りの行 | 03 の 4・11 の記述どおり | 03 の 4・11.4 | 1 |
| `03-T-04`、`11-T7`(最後の入力) | `@(` を 1,000・10,000・100,000 並べた入力を `ParseInput` に渡して、異常終了しないか。しきい値 | 異常終了しない。1,000 段で余裕がある | 03 の 4.8・6.4(事前走査の上限を下げる) | 1 |
| `03-T-05`、`03-T-12`、`11-T7` | `1e999`・`1e-999`・範囲を超える 16 進・`1_000`・`.5`・`5.`・`01`・`1kb`・`-0.0`・`-9223372036854775808`・`@{ a = ,1 }`・`@{ a = 1L }` の、`ParseError` の有無・`Extent.Text`・負の数の AST の型。`[double]::Parse("1e999999999")` の結果と時間 | 03 の 4.3.1 の表どおり。落ちない | 03 の 4.3.1 | 1 |
| `03-T-06`〜`03-T-08`、`09-T-C05` | `"a$"`・`"$5"`・`` "`$a" ``・`"${x}"` の分類。here-string の改行の扱い。`` "`u{D800}" ``・`` "`u{110000}" ``。単一引用符の中の U+201C〜U+201E、U+2018・U+2019。二重引用符の `` `u{...} `` の値 | 03 の 4.4 の想定どおり | 03 の 4.4、09 の 8.4 | 3 |
| `03-T-09`、`03-T-10`、`09-T-C04`、`09-T-C09` | 裸のキー(`a-b`・`a.b`・`1abc`・`if`・`true`・`$true`)の分類と `Value`。予約語の一覧の十分さ(`Class`・`Enum` など)。`name =` の次の行に値を書く。`@(,@(1))`・`@(,@())`・`a = ,@(1, 2)` の木 | 03 の 4.5〜4.7、09 の 8.3 の想定どおり | 03 の 4.5〜4.7、09 の 8.3 | 3 |
| `03-T-11`、`09-T-C03`、`11-T8` | 書き出した `.psd1` を `Import-PowerShellDataFile` で読んで、値と型が同じか(`-9223372036854775808`・`1e16`・`-0.0`・`@(,@(1,2))`)。重複キー(`a`・`A`)の `.psd1` を渡したときの挙動 | 比較用に使える。重複の扱いを記録 | 03 の 4.11、09 の 6.3 | 3 |

### 4.4 JSON・ファイル・ステート・数値

| 元の番号 | 試験 | 期待 | 外れたときに直す章 | 段階 |
|----------|------|------|--------------------|------|
| `03-T-13` | `Utf8JsonReader` を PowerShell から使えるか | 使えない | 03 の 5 | 1 |
| `03-T-14`、`03-T-15`、`09-T-C01` | `File.GetAttributes`・`FileInfo.Length` の、シンボリックリンク・FIFO・デバイスファイル・`ENOTDIR` での結果。`UTF8Encoding(false, true).GetString` の BOM の扱いと、過長・サロゲート・範囲外・単独の `FF` の例外。FIFO を `FileStream` で開いたときのブロック | 03 の 3 の想定どおり。FIFO でブロックしない | 03 の 3、09 の 3 | 2 |
| `03-T-16`、`06-T-12`、`09-T-C08` | 自前の JSON の読み取り(1 MiB の `[` の列、空白で埋めた入力、長い文字列、多数の短い要素)と、1 MiB の `.psd1` の読み取り、100,000 要素の書き出しの時間 | 30 秒以内(目標) | 03 の 5.3(性能の規則)、11 の 2.1 | 2 |
| `03-T-17`、`03-T-20` | `[System.Array]::Sort` とスクリプトブロックの比較の安定性と実行空間。全 `[char]` の空白の判定(自前の 25 文字の表との差) | 25 文字だけが真 | 03 の 9・1 | 2・3 |
| `06-T-10`、`06-T-11` | `[System.IO.FileStreamOptions]::new()` のプロパティの設定(`Mode`・`UnixCreateMode`)と `FileStream` の作成。`Write($bytes, 0, $bytes.Length)` が `byte[]` の過負荷に解決される | 呼べる | 06 の 5 | 5 |
| `05-T-1`〜`05-T-6` | `[bigint]` から `[uint64]` へのキャスト(2^63 以上、範囲外で例外)。`-band 0xFF` が負の `[int]` で 2 の補数の下位 8 ビットを返す。`return , @()` が空配列になる。`[int]$x = $null` が 0 になる。`[bigint]` の比較・`Divide`・`Min`。`[int]$s[$i]` | 05 の 17 の想定どおり | 05 の 0.2・0.3・2.2、10 の 4(剰余の式に切り替える) | 4 |
| `09-T-C02`、`09-T-C07` | ランダムな `[double]` の書き出しを `[double]::Parse(..., Float, Invariant)` で読み戻して、ビット表現が一致する。ランダムな木 1 万個の往復 | 一致する | 09 の 6.1・9 | 6 |
| `09-T-C06` | Windows で、標準出力の `StreamWriter` に `"a`nb"` を書いても CRLF に変わらない | 変わらない | 08 の 1.2.2 | 7 |

### 4.5 出力・品質ゲート

| 元の番号 | 試験 | 期待 | 外れたときに直す章 | 段階 |
|----------|------|------|--------------------|------|
| `11-T9`(AC-493) | 標準出力を閉じたパイプ・`/dev/full` にして、`[Console]::OpenStandardOutput()` を包んだ `StreamWriter` に書く | 例外の型(`IOException` か、黙って捨てるか)を記録。UNIX 系の閉じたパイプで例外にならない(既知の差) | 08 の 1.3 | 1 |
| `11-T10` | Pester 5 の最小のテストで、`-Skip:$cond`(`BeforeDiscovery`)・`-TestCases`・`Mock -ModuleName`・`InModuleScope`・`$TestDrive`・`CodeCoverage`(JaCoCo) | キー名・挙動・XML の形の確認 | 11 の 3.9・5.3 | 0 |
| `11-T11` | `Invoke-ScriptAnalyzer -Settings` と `Invoke-Formatter -Settings` に 11 章 9.2 節の設定を渡す | キー名・規則名・整形の冪等性の確認 | 11 の 9.2 | 0 |
| `11-T14` | zip・tar.gz を `Zone.Identifier` つきで展開する(`tar.exe`・`Expand-Archive`・エクスプローラー・7-Zip) | AC-561 の期待の確定 | 11 の 6.5 | 12 |
| `11-T15` | `Save-Module -RequiredVersion` の保存の形とハッシュの再現性(3 OS) | 11 の 8.3 の前提 | 11 の 8.3 | 0 |

- 段階 0 で行う試験(`04-T6`、`11-T10`、`11-T11`、`11-T15`)は、`build/` のスクリプトを動かす前に行う。
- 試験の結果が、この文書や 02〜11 章の記述と違うときは、その章を直す(この仕様書に、確認した版と OS を書き添える)。
- 04 章の T1・T5、07 章の T-5、11 章の T1・T2 は、同じことを確かめている。1 度で足りる。

## 5. 迷ったときの指針

- 利用者から見える結果(受理・拒否、終了コード、出力の文字列、作られるファイルの内容)は、必ず 03〜10 章の字句に合わせる。内部の構造は、02 章の依存の向きと、`Deps` の境界を守れば、自由である。
- PowerShell のコマンドレット・演算子が、規定と違う挙動をするときは、まず本書 2 章の表にあるかを見る。ない場合は、実際に動かして確かめ、確かめた版と OS を、該当の章に書き足す。
- 確かめられなかったときは、断定せず、「未確認」と書いて、3 章の表に加える。
- コマンドレット(`Get-Content`・`ConvertFrom-Json`・`Start-Process`・`Test-Path` など)と便利な演算子(`-match`・`-eq`・`switch`)は、既定の挙動が、文化・大文字小文字・エンコーディング・配列の展開に依存する。この仕様は、`.NET` の API と、手書きの走査を選んでいる。迷ったら、まず禁止コマンドの検査(11 章 5.4 節)の一覧を見る。
