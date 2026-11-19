# 11 品質とビルド(非機能要件、テスト、受け入れテスト、CI、リリース、デモ、依存、レイアウト)

対象: `conductor` v0.8.0 の PowerShell 版。この文書は、PowerShell で `conductor` を再実装した人が、(1) 同じ品質の基準を満たし、(2) 同じテストを書き、(3) 同じ CI とリリースの仕組みを再現するための仕様である。
この文書の基準を満たし、受け入れテスト(4 章)がすべて通れば、利用者から見て同じ挙動のツールになる。

| 章 | 内容 |
|----|------|
| 1 | この文書の位置づけと、PowerShell 版の前提、確認の範囲 |
| 2 | 非機能要件(性能、堅牢性、セキュリティ、保守性)の基準値と、強制か目安かの区別 |
| 3 | テスト構成(Pester 5 の単体テスト、受け入れテストの仕組み、テスト用ジョブ、偽の部品、OS 別のテスト、略記、判定の書き方) |
| 4 | 受け入れテスト一覧(機能ごと。ID は `AC-xxx`) |
| 5 | CI(GitHub Actions)と品質ゲートの再現手順、検査スクリプトの全文 |
| 6 | リリース |
| 7 | デモ |
| 8 | 依存(実行時: 無し。開発時: Pester、PSScriptAnalyzer)と、版の固定 |
| 9 | リポジトリのレイアウト |
| 10 | 未確認の事項と、実装時に最初に確かめる小さな試験、他の章との整合が要る点 |

他章との関係: 振る舞いの規定は [03](03-definition-format.md)(定義)、[04](04-cli.md)(CLI)、[05](05-execution-flow.md)(流れ)、[06](06-state-file.md)(ステート)、[07](07-process-and-signals.md)(プロセスとシグナル)、[08](08-output-and-i18n.md)(出力)、[09](09-converter.md)(変換)、[10](10-errors-and-exit-codes.md)(終了コード)にある。
構成は [02](02-architecture.md)、再構築の手順は [12](12-rebuild-guide.md) にある。受け入れテストは、それらを「前提 → 実行 → 期待」の形で、外から確かめる。

---

## 1. この文書の位置づけと、PowerShell 版の前提

- 受け入れテストは、**配布物の起動用ランチャー(UNIX 系は `bin/conductor`、Windows は `bin/conductor.cmd`。どちらも本体の入口 `bin/conductor.ps1` を `pwsh -NoLogo -NoProfile -NonInteractive -File` で呼ぶ。[04](04-cli.md) 1.3 節)を、子の `pwsh` プロセスを含む外部プロセスとして実行して**確かめる(終了コード、標準出力、標準エラー、作られるファイル)。外からは確かめにくい項目は、区分を「単体」「注入」「点検」「手動」「計測」として示す(3.6 節)。
- 定義ファイルは **`.psd1`(PowerShell データファイル)を主**とし、JSON(`.json`)も受け付ける。それ以外の形式は扱わない。既定のファイル名は `conductor.psd1`。書式は拡張子で決める(大文字小文字を区別しない)。ステートファイルは JSON で、名前は `<定義ファイル名>.state`(run ID があれば `<定義ファイル名>.<run ID>.state`)。`convert` は `.psd1` と JSON の相互変換(`--to psd1|json`。入力の形式は `--to` の反対とみなし、拡張子は見ない)。人が書く定義にコメントが要るときは `.psd1` を使う(`#` と `<# #>` が書ける)。`.psd1` は**コードを実行しない**(構文木を作って、許す構文だけを自前で値にする。[03](03-definition-format.md))。
- 受け入れテストの ID は `AC-xxx`。001〜449 は、`conductor` が満たすべき基本の項目(定義の形式を `.psd1` に直したもの)。450〜487 は、`.psd1` と JSON の扱いと、品質・CI・版の項目。488 以降は、PowerShell(実行空間、パイプライン、`pwsh` の起動、`Add-Type`、`-File` の終了コード、実行ポリシー、型の自動変換、大文字小文字の規則)に固有の項目。番号は、この文書の中で一意。
- 期待の標準エラーの文言は、特に記さない限り `LANG=C`(英語)。理由の文(日本語の固定の文)は、英語の表示でも日本語のまま([08](08-output-and-i18n.md)、[10](10-errors-and-exit-codes.md))。理由の文の正確な字句は、03・08・10 章が正本で、この文書の例は、それと同じ字句を使う。`.psd1` に固有の理由の字句が、03 章に定まっていない場合は、「03 章に従う」と書いた。
- 実行環境は PowerShell 7.x(`pwsh`。Linux、macOS、Windows)。Windows PowerShell 5.1 は対象外。サポートする版は「実装時点でサポート中の LTS のうち、最小と、最新の安定版」(以下 `PS_MIN` と `PS_MAX`。例は 2.5 節)。
- **この環境には PowerShell(`pwsh`)がなく、実機で動かせない。** この文書の作成時に確かめたのは、公式ドキュメントの記述の範囲だけである。確認できた点には「確認(出典: …)」を付け、確認できなかった点は「未確認」と書いて、10 章にまとめる。性能の基準値は、すべて仮で、測っていない。

確認できた範囲(いずれも learn.microsoft.com、pester.dev、GitHub の runner-images の文書。この文書の作成日 2026-11-20 に取得した版):

| 項目 | 確認の内容 | 出典 |
|------|-----------|------|
| サポート中の PowerShell | PowerShell 7.6(LTS。サポート終了 2028-11-14。.NET 10)がサポート中。7.5(2026-11-10 終了)と 7.4(LTS。2026-11-10 終了)は、作成日の時点でサポートが終わった。7.7 はプレビュー。サポートの終わりは、土台の .NET の方針に従う | 「PowerShell Support Lifecycle」 |
| `pwsh -File` | `-File <path> [args]` は、`-File` の後ろの全部を、スクリプトのパスと、スクリプトへの引数として扱う(`-File` は最後に書く)。引数は、**文字列のまま**スクリプトに渡る(呼び出し元のシェルが解釈した後)。**正常終了の終了コードは 0。`exit N` で終わると N。スクリプトを止める誤り(throw など)で 1。Ctrl+C で中断されると 0**(`-Command` では 1)。Windows では、7.2 以降、`-File` は `.ps1` だけ受け付ける | 「about_Pwsh」 |
| `pwsh -Command` | 終了コードは、最後に実行したものの `$?` から 0 か 1 になる。外部プログラムやスクリプトが 0・1 以外を返しても 1 になる。具体的な値を保つには `exit $LASTEXITCODE` を足す。**したがって、ランチャーは `-Command` でなく `-File` を使う** | 「about_Pwsh」 |
| `pwsh` の起動のオプション | `-NoProfile`(プロファイルを読まない)、`-NonInteractive`(`Read-Host` や確認の入力を求めると、待たずに終端エラー)、`-NoLogo`(対話の起動時の表示を隠す)、`-ExecutionPolicy`(**Windows だけに効く。Windows 以外では、引数も値も無視される**)。`-CommandWithArgs` は 7.5 で正式になった | 「about_Pwsh」 |
| 実行ポリシー | 強制は Windows だけ。Windows 以外の既定は `Unrestricted`(変更不可。実際の動作は `Bypass` と同じ)。Windows の既定は `RemoteSigned`(クライアントもサーバーも)。どの範囲にも設定がなければ、クライアントは `Restricted`、サーバーは `RemoteSigned`。`RemoteSigned` は、インターネットから取得した(代替データストリームが付いた)署名なしのスクリプトを実行しない。`Unblock-File` で解除すると実行できる。`curl.exe`・`Invoke-WebRequest`・`Invoke-RestMethod` で取得したファイルには、印が付かない場合がある。`Restricted` は、`.psm1` も止める。実行ポリシーはセキュリティの境界ではない | 「about_Execution_Policies」 |
| Pester の設定 | `PesterConfiguration` の `Run.Path`(既定 `.`)、`Run.Exit`(失敗時に 0 以外で終わる。既定 `$false`)、`Run.PassThru`、`Run.Container`、`Output.Verbosity`(`None`・`Normal`・`Detailed`・`Diagnostic`)、`TestResult.OutputFormat`(`NUnitXml`・`NUnit2.5`・`NUnit3`・`JUnitXml`)、`TestResult.OutputPath`(既定 `testResults.xml`)、`Filter.Tag`・`Filter.ExcludeTag`、`CodeCoverage` の `Enabled`・`OutputFormat`・`OutputPath`(既定 `coverage.xml`)・`Path`・`CoveragePercentTarget`(既定 75)など。**取得した文書は Pester 6 を扱っており、5.x でこれらのキーが同じ名前で存在することは、実装時に確認する**。カバレッジの形式は、`JaCoCo` が既定。`Cobertura` は Pester 6 で使えると書かれている。目標の率に満たないときに実行が失敗するかは、文書に記述がない(**未確認**。この文書は、自前の検査(5.4 節)で閾値を判定する)。`Invoke-Pester` の `-CI` スイッチの挙動は、取得した文書に記述がなく**未確認**(この文書は、`-CI` に頼らず、`PesterConfiguration` を使う) | pester.dev の「Configuration」「Code Coverage」 |
| PSScriptAnalyzer | `Invoke-ScriptAnalyzer` は `-Settings`(`.psd1` のパス、ハッシュテーブル、組み込みの設定名)を取る。設定の主なキーは `Severity`・`IncludeRules`・`ExcludeRules`・`Rules`(規則ごとの `Enable` など)・`CustomRulePath`。プロジェクトの根に `PSScriptAnalyzerSettings.psd1` があると、根を `-Path` に渡したときに暗黙に使われる(明示の指定が優先)。構文の誤りは、重大度 `ParseError` の診断として出る(1.18.0 以降)。規則の抑止は `SuppressMessageAttribute`(構文の誤りは抑止できない)。`UseCompatibleSyntax` は既定で無効で、`TargetVersions` を取る。取得した文書の版は 1.25.0 | 「Using PSScriptAnalyzer」「List of PSScriptAnalyzer rules」「UseCompatibleSyntax」 |
| PSScriptAnalyzer の規則(この仕様が使うもの) | 常に有効: `AvoidUsingInvokeExpression`(Warning)、`AvoidUsingWriteHost`(Warning)、`AvoidUsingCmdletAliases`(Warning)、`AvoidUsingEmptyCatchBlock`(Warning)、`AvoidGlobalVars`(Warning)、`AvoidUsingBrokenHashAlgorithms`(Warning)、`UseApprovedVerbs`(Warning)、`UseDeclaredVarsMoreThanAssignments`(Warning)、`PossibleIncorrectComparisonWithNull`(Warning)、`ReviewUnusedParameter`(Warning)、`UseShouldProcessForStateChangingFunctions`(Warning)、`MissingModuleManifestField`(Warning)、`UseToExportFieldsInManifest`(Warning)、`UseBOMForUnicodeEncodedFile`(Warning)、`AvoidAssignmentToAutomaticVariable`(Warning)、`AvoidUsingComputerNameHardcoded`(Error)など。既定で無効(設定で有効にする): `UseConsistentIndentation`・`UseConsistentWhitespace`・`PlaceOpenBrace`・`PlaceCloseBrace`・`AlignAssignmentStatement`・`UseCorrectCasing`・`AvoidLongLines`・`UseCompatibleSyntax`・`AvoidSemicolonsAsLineTerminators`・`UseConstrainedLanguageMode`。既定で有効: `AvoidUsingPositionalParameters`(Information)・`UseSingularNouns`(Warning)・`ProvideCommentHelp`(Information)・`AvoidOverwritingBuiltInCmdlets`(Warning) | 「List of PSScriptAnalyzer rules」 |
| `Invoke-Formatter` | `Invoke-Formatter -ScriptDefinition <string> [-Settings <hashtable または .psd1 のパス>] [-Range <int[4]>]`。結果は整形後の文字列(`[string]`)。`-Settings` を省くと `CodeFormatting` の設定(`Settings/CodeFormatting.psd1`)を使う。ファイルを直接直す引数はない(文字列を受け取って、文字列を返す) | 「Invoke-Formatter」 |
| `Install-Module` | `-RequiredVersion`(完全一致。`-MinimumVersion`・`-MaximumVersion` とは併用できない)、`-Scope`(PowerShellGet 2.0.0 以降で、`pwsh` では、昇格していなければ既定 `CurrentUser`)、`-Repository`、`-Force`。同じ版が入っていると、何もせず続ける。PowerShell Gallery は TLS 1.2 以上が要る。取得した文書は PowerShellGet 2.x の版 | 「Install-Module」 |
| GitHub のランナー | `ubuntu-24.04` のイメージ(版 20261111.370.1)に、PowerShell 7.6.10、モジュール Pester 5.9.0、PSScriptAnalyzer 1.25.0 が入っている。`ubuntu-latest` は 2026-11 に Ubuntu 26.04 を使う予告がある。**macOS・Windows のイメージの内容は、この文書の作成時には取得しておらず、未確認** | actions/runner-images の `Ubuntu2404-Readme.md` |

- 版の数字(PowerShell、Pester、PSScriptAnalyzer、GitHub Actions の部品)は、例として書いたものを除き、「実装時に確認する」。
- 取得できなかったもの(未確認): `Test-ModuleManifest` の検査の範囲、GitHub の macOS・Windows のランナーの `pwsh` と各モジュールの版、PowerShell を指定の版で入れる公式の `actions/setup-*`(存在しない、とは言い切れない)、`PowerShell/PowerShell` のリリースの成果物の名前と、ハッシュのファイルの有無、`Add-Type` の内部の動作(一時ファイルの有無)、`PSResourceGet`(`Install-PSResource`)の仕様。

---

## 2. 非機能要件

「強制」は、基準を超えると受け入れテストが不合格になるもの。「目安」は、基準値で、超えたら警告として記録する(不合格にしない)。目安は、利用者から見える挙動を損なわない範囲で、別の値を採ってよい。「仮」は、暫定として置いた値で、**この文書の作成時には何も測っていない**。実測してから見直す。

### 2.1 性能

p95 は、昇順に並べた測定値の `ceil(n × 0.95)` 番目(1 始まり)。測定値は、標準エラーに `[bench] <行>` として出し、環境変数 `GITHUB_STEP_SUMMARY` があれば、そのファイルに `- <行>` として追記する。基準を超えたときは、`警告: <名前>: <値> ms が基準 <限度> ms を超えた` を記録する。
環境変数 `STARTUP_BENCH_ENFORCE` が文字列 `true` のときだけ、目安の基準超えを、テストの失敗にする(CI の既定は `"false"`)。

PowerShell 版は、次の理由で、**起動と、1 文字ずつの走査が遅い**と見込まれる。いずれも測っておらず、基準値は仮である。

1. **`pwsh` の起動**: 起動のたびに、ランタイムの起動、PowerShell のエンジンの初期化(`-NoProfile` でも)が起きる。ランチャーの `--version` は、この費用を必ず払う。
2. **モジュールの読み込み**: `Conductor.psm1` が、`Private/` と `Public/` の多数のファイルをドットソースする。ファイル数に比例して、構文解析の費用が増える。(対策の案: リリース物を作るときに、全関数を 1 つの `Conductor.psm1` に結合する。採否は、実測してから決める。この文書では、結合しないことを基本とし、案として残す。)
3. **`Add-Type`(C# の補助の部品のコンパイル)**: 数百 ms の可能性がある(未測定)。`run`・`pipeline` の最初のジョブの起動までに遅延させ、`--version`・`--help`・使い方の誤り・`validate`・`reset`・`watch`・`convert` の経路では行わない(AC-489)。
4. **自前の JSON・`.psd1` の読み取り**: 文字列を 1 文字ずつ走査する PowerShell のループは、コンパイル済みのコードより桁違いに遅い(未測定)。大きさの上限は 1 MiB(定義・ステート・変換)。実装は、空白の読み飛ばし・文字列の本体の走査に、`String.IndexOfAny` や、位置を指定した `[regex]`(`\G` で始める。`RegexOptions` を明示)のような、**まとめて走査する .NET のメソッド**を使い、1 文字ごとの PowerShell のループを減らす。通常の定義は数 KiB なので、上限の大きさの入力は、遅くても完了することだけを要件にする(AC-040、AC-304、AC-466。これらは `Slow` のタグを付け、120 秒の上限で行う)。

| 項目 | 基準 | 扱い | 受け入れテスト |
|------|------|------|----------------|
| `pwsh -NoProfile -NonInteractive -Command exit` だけの起動から終了まで(20 回。最初の 5 回を捨てる) | 記録のみ(基準なし。他の基準の「`pwsh` を除いた値」の差し引きに使う) | 目安 | AC-488 |
| ランチャーの `conductor --version` の起動から終了まで(同) | p95 1,500 ms(Linux・macOS)、3,000 ms(Windows)(仮) | 目安 | AC-488 |
| 同じ値から、上の `pwsh` だけの起動を引いた値 | p95 1,000 ms(Linux・macOS)、2,000 ms(Windows)(仮) | 目安 | AC-488 |
| モジュールの読み込み(`Import-Module` を新しい `pwsh` で。`Measure-Command`。20 回。最初の 5 回を捨てる) | p95 800 ms(仮) | 目安 | AC-488 |
| `Add-Type` による補助の部品の用意(新しい `pwsh` の中で 1 回目。20 回、最初の 5 回を捨てる) | 記録のみ(基準なし) | 目安 | AC-489 |
| 定義(ジョブ 10 個、約 2 KiB)の読み込みと検証(プロセス内。最初の 5 回を捨てて 50 回) | p95 150 ms(仮) | 目安 | AC-397 |
| 定義(ジョブ 100 個、約 20 KiB)の同様の測定 | p95 1,500 ms(仮) | 目安 | AC-397 |
| 起動前の処理(`command` の検査、ステートの読み、照合、速い書き込み)。最初の 5 回を捨てて 50 回。ジョブ 10 個の定義 | p95 150 ms(仮)。`fsync` つきの耐久性のある書き込みは基準なし(記録のみ) | 目安 | AC-398 |
| `run` の `conductor` 自身のオーバーヘッド(ジョブ 10 個の定義、10 回。ランチャーの `pwsh` の起動を含む) | p95 5,000 ms(仮) | 目安 | AC-399 |
| `pipeline` のジョブ 1 つあたりの増分(ジョブ 10 個と 1 個の `pipeline` の差から求める。5 回) | p95 700 ms(仮) | 目安 | AC-400 |
| ジョブ 1,000 個の定義(約 200 KiB)で `run job0` が終わるまで(起動を含む) | 20,000 ms(仮) | 目安 | AC-401 |
| ジョブ 1,000 個の `pipeline` の流れの判断(偽の `Deps` を使い、プロセスとファイルなし) | 60,000 ms(仮) | 目安。ただし、完了のステートが残ることは常に検査する | AC-402 |
| ジョブの実行中の `conductor` の CPU 時間の増分(Linux。待機の 1 秒後から 2.5 秒後までの増分) | 150 ms 以下(仮。実測してから確定) | **強制**(`/proc` を読めない環境ではスキップ) | AC-403 |
| 同じ定義への 2 つの `run` を同時に | 両方が実行され、どちらもジョブの終了コードで終わり、ステートは読み戻せる(排他制御はしない) | **強制** | AC-404 |
| リトライの待機の中断 | 合図から 1 秒以内に返る。待ちは、250 ms 以下の `Interrupt.Wait(int)` の繰り返しで、合図が立てば直ちに起こされる([07](07-process-and-signals.md) 8 章の決定に従う)。`Start-Sleep` の長い 1 回で待たない | **強制**(1 秒) | AC-275 |
| ジョブの終了の待ち | `Process.WaitForExit(int)` の短い区間([07](07-process-and-signals.md))で待つ。合図から転送までの遅れは、区間の長さ以下 | 目安 | [07](07-process-and-signals.md) |
| ジョブが大量の出力(16 MiB)を標準出力に書いても、`conductor` がそれを溜め込まない | 子の標準入出力は引き継ぐ(`RedirectStandardOutput = $false`)ので、出力は `conductor` を通らない。出力の全体がそのまま読み手に届く | **強制** | AC-494 |
| 配布物の大きさ(zip と tar.gz) | 記録のみ(基準なし。測っていない) | 目安 | AC-526 |

測定のしかた:

- 「`conductor` 自身のオーバーヘッド」は、`conductor` を起動してから終了するまでの時間から、ジョブ自身の時間を引いた値。ジョブ自身の時間は、テスト用ジョブ `timing`(3.4 節)が出す `start_ns`・`end_ns` の差と、テスト用ジョブ自身の起動の費用(基準。`TJ exit 0` を直接 20 回起動した所要時間の中央値。**テスト用ジョブも `pwsh` なので、この費用は大きい**)の和。
- 「ジョブ 10 個の定義」は、10 個とも同じ形の簡単なジョブ(テスト用ジョブの `exit 0` か `timing`)。
- 「`pipeline` のジョブ 1 つあたりの増分」は、`(10 個の pipeline の全体 − 1 個の pipeline の全体 − 9 × ジョブ 1 つの自身の時間) / 9`。
- 測定の対象は、リリースと同じ作り方の成果物(展開したアーカイブのランチャー。6 章)。リポジトリの `bin/conductor` で測る場合は、記録に明記する。
- 暖機のため、プロセス内の測定は、最初の 5 回を捨てる。プロセスの起動の測定も、最初の 5 回を捨てる(ファイルのキャッシュの影響を除くため)。
- 性能のテストは、`tests/Perf/` に置き、タグ `Perf` を付け、環境変数 `CONDUCTOR_PERF=1` がなければスキップする(`BeforeDiscovery` で `$env:CONDUCTOR_PERF` を見て `-Skip` を決める)。測定は 3 OS で行い、失敗しても全体は不合格にしない(目安)。

### 2.2 堅牢性

外部の入力(コマンドライン、`command` の文字列、ステートファイル、定義ファイル、変換の入力、OS の結果)に対して、**異常終了しない**(失敗は終了コードとメッセージで返す)。【強制】
PowerShell では、捕まえそこねた終端エラーは、既定で、エラーのレコード(`Exception:`、`At line:` などの複数行)を標準エラーに出して、`pwsh -File` の終了コード 1 になる(確認: 1 章の `-File` の記述。スクリプトを止める誤りで 1)。これは、`conductor` の誤りの表示でも終了コードでもない。したがって、`Invoke-Conductor` の全体を `try`/`catch` で囲み、**捕捉できる失敗はすべて**、1 行 `conductor: error: internal error` と、終了コード 70 にする(AC-414、AC-490、AC-491)。ランチャーも、`Invoke-Conductor` が値を返さずに例外を漏らした場合の保険として、`try`/`catch` で 70 にする(AC-492)。
捕捉できないもの: スタックの溢れ(`StackOverflow`)は捕捉できず、プロセスが終了する。PowerShell のスクリプトの関数の再帰の深さの上限は、**確認できていない(未確認)**。したがって、JSON・`.psd1` の読み取りと、定義・ステート・変換の走査は、入れ子の深さの上限(32 段)を、**再帰に入る前に**検査する(深さの数え上げは、明示のスタック(`[System.Collections.Generic.Stack[object]]` か配列)か、字句の走査で行い、関数の再帰に頼らない。AC-412、AC-524、AC-232)。
`Ctrl+C`・端末の切断で起きる `PipelineStoppedException` は、`catch [System.Exception]` では確実に捕捉できると限らず、後始末は `finally` に置く(AC-491)。ただし、`pwsh -File` は、Ctrl+C で中断されると終了コード 0 になる(確認: 1 章)ので、`conductor` は自分でシグナルを受け(07 章の補助の部品)、`130` などを `Invoke-Conductor` の戻り値で返し、ランチャーが `exit` する(AC-301、AC-532)。
標準出力・標準エラーへの書き込みの失敗(`IOException`(閉じたパイプ・`ENOSPC`)、`ObjectDisposedException`、`UnauthorizedAccessException`)も、`Invoke-Conductor` の中で処理する。終了時の flush の失敗で、終了コードが変わったり、例外が出たりすることを許さない(AC-496)。
**出力の混入**: 関数の戻り値にパイプラインの出力が混ざる(例: `.Add()` の戻り値、`New-Item` の出力)事故で、終了コードの型が `[int]` でなくなることを許さない。`Invoke-Conductor` の戻り値は、ちょうど 1 つの `[int]`(AC-552)。

| 項目 | 基準 | 受け入れテスト |
|------|------|----------------|
| ランダムな引数 | 固定の種のランダムな引数列(語の表から 0〜6 個を選ぶ。乱数は 3.1 節の `Xorshift32`)500 回で、終了コードが 0〜255、標準エラーに制御文字(改行を除く)と U+202E がない | AC-405 |
| UTF-8 でない引数(UNIX 系) | ジョブID、`--config`、オプション、`convert` の引数に、不正なバイト列を渡しても異常終了しない。**`pwsh` が、不正な UTF-8 のコマンドライン引数を、入口スクリプトが取り出す引数の配列(`GetCommandLineArgs()`。04 章 1.5 節)にどう渡すか(U+FFFD に置き換える見込みだが)は、公式の文書で確認できていない(未確認)**。実装は、不正なバイト列が、何かの文字列になっても、異常終了せず、出力が有効な UTF-8 になることを保証する(AC-406、AC-519) | AC-406、AC-519 |
| 壊れたステート | 壊れた JSON、ランダムなバイト列、状態・再開位置・フラグを乱したステート、巨大な値で、異常終了しない(終了コード 65) | AC-407 |
| 空・制御文字を含むジョブID | 異常終了しない | AC-408 |
| ランダムな `command` | 文字集合 `a b ⇥ ⏎ ' " \ ; \| & $ * あ` から長さ 0〜39 の文字列 500 件で、分割が成功したら `program` は空でない。成功か誤り(64)かのどちらか | AC-409 |
| 大きな `command` | 約 3 万引数が分割できる(PowerShell のループの遅さを考え、小さくした。仮)。閉じない引用符は誤り。再帰に依存しない | AC-410 |
| 起動できない `command` | 空文字列、`'`、`''`、存在しない名前、ディレクトリのパス、`/`、NUL を含む文字列は、すべて起動失敗(または定義の誤り)。`Process.Start` が投げる例外(`Win32Exception`・`InvalidOperationException`・`ArgumentException`・`PlatformNotSupportedException` など)を、すべて起動失敗として扱う。範囲外のシグナル番号の合図でも起動せず中断として扱う | AC-411、AC-522、AC-548 |
| 入れ子の深い入力 | 深さ 1,000 と 100,000 の `.psd1`・JSON(定義・ステート・変換)で、異常終了せず失敗を返す(スタックの溢れでプロセスが落ちない) | AC-412、AC-524 |
| 整数の字句 | `$true`/`$false`、小数(`1.0`)、指数(`1e0`)、`NaN`・`Infinity`、型の接尾辞(`1L`)、乗数(`1kb`)、桁数が極端に多い整数で、異常終了せず、整数として受理しない。`[long]` を超える値、`[double]` の丸めで別の整数に化ける値を、範囲内として受理しない | AC-502 |
| 製品のコードの書き方 | 外部の入力の経路に、異常終了や取り違えを起こす書き方(`Invoke-Expression`、`Write-Host`・`Write-Output`・`Write-Error`、`Start-Process`、許可箇所以外の `Add-Type`、`-eq` などの大文字小文字を区別しない文字列の比較、`-match`・`-split`・`-replace`・`-like` の濫用、`exit` の使用(ランチャー以外)、`Exception` を黙って捨てる `catch`、`Set-StrictMode` の欠落)がない | AC-421、AC-551 |
| 内部の誤り | `Invoke-Conductor` の `try`/`catch` が、固定の英語 `conductor: error: internal error` を標準エラーに 1 行書き、終了コード 70。標準エラーが閉じていても 70 | AC-414、AC-490、AC-491 |
| 変換の入力 | 範囲外・空・壊れた入力、ランダム・切り詰めた入力で異常終了しない。成功したランダムな構造は往復で等しい | AC-413 |
| 標準出力の失敗 | 標準出力への書き込みの失敗(`IOException`(閉じたパイプ)、`ENOSPC` など)を捕捉する。`watch` は 65、`convert` は 67 で、メッセージを標準エラーに出す | AC-186、AC-236、AC-493、AC-495 |
| 標準出力の取りこぼし | 標準出力がパイプで、読み手が遅くても、終了前に flush して書き込みの完了を待つ。出力が途中で切れない | AC-494 |
| 終了時の flush | 標準出力が閉じていても、終了時に例外が出ず、終了コードが変わらない | AC-496 |
| 終了後に残らない | すべてのサブコマンドが、出力の完了から 1 秒以内に、プロセスとして終了する(標準入力が開いたままでも待たない。補助の部品のシグナルの登録・スレッドが残ってハングしない) | AC-497 |
| 警告を出さない | PSScriptAnalyzer の指摘が 0 件(Error と Warning)。通常の実行で、PowerShell の警告・冗長・エラーのレコードの表示(`WARNING:`・`VERBOSE:`・`At line:`)が標準エラーに出ない | AC-498 |
| 時間の上限 | プロセスを起動するテストは、すべて 60 秒の上限つき(`pwsh` の起動が遅いため、長めにした。超えたら子プロセスを強制終了して不合格。`Slow` タグは 120 秒) | 3.3 節 |

### 2.3 セキュリティ

| 項目 | 基準 | 受け入れテスト |
|------|------|----------------|
| シェルを呼ばない | 起動は、`command` の分割(または配列)で得た `program` と引数を、`System.Diagnostics.ProcessStartInfo`(`UseShellExecute = $false`。引数は `ArgumentList`)に渡すだけ。`Start-Process`、`&` 演算子や `Invoke-Expression` で `command` を実行する経路、`sh`・`cmd`・`bash`・`powershell`・`pwsh`・`-c`・`/C` を挟む経路がない(Windows の `.cmd` / `.bat` の安全策は 07 章) | AC-116(点検)と 4.2 節 |
| 引数にメタ文字が入っても注入にならない | `a;b`、`&&`、`\|`、`$HOME`、`*`、`>x`、`` `x` ``、`$(x)` などが、そのまま引数として渡る(PowerShell が解釈しない)。Windows の `.cmd` への危険な引数で、続くコマンドが実行されない | 4.2 節 |
| 検証は副作用を持たない | 定義の検証は、コマンドを解釈・展開・起動しない。定義ファイルのディレクトリは変わらない。**`.psd1` の読み取りは、コードを実行しない**(構文木の検査だけ。`$(...)`・コマンド・変数・スクリプトブロックを含む `.psd1` を、評価せずに拒否する) | AC-415、AC-453、AC-531 |
| 入力の値を表示に出さない | すべての種類のエラーの表示に、入力の値(秘密のつもりの文字列、`command`、パーサーのエラー文)が現れない。キー名は制御文字をエスケープして表示。**PowerShell のパーサーのエラー(`ParseError.Message`。入力の断片を含みうる)と、例外のスタックトレースを、そのまま表示に出さない**(行・列の位置だけを使う) | AC-416 |
| データからコードを作らない | `.psd1` の読み取りは、構文木の段階まで(`System.Management.Automation.Language.Parser`)で、`Invoke-Expression`・`[scriptblock]::Create`・`ConvertFrom-Json` を製品のコードで使わない。`Import-PowerShellDataFile` も、定義・変換の読み取りには使わない(位置を返さない。比較用のテストでだけ使う)。使ってよい 1 か所は、`Conductor.psm1` が、マニフェスト `Conductor.psd1`(自分のモジュールのデータ)から版を読む呼び出しだけ(02 章 10 章) | AC-531、AC-421 |
| ステートに機微な値を書かない | ステートファイルに、`command`、引数、環境変数の値、標準出力の内容が現れない | AC-417 |
| 表示のエスケープ | 制御文字・方向制御文字・孤立サロゲートを無効化する(4.14 節の表)。定義・ステート・変換の各部が、同じ入力に同じ結果を返す | 4.14 節、AC-519 |
| ネットワークに接続しない | 実行中に外向きの通信をしない。製品のコードが `Invoke-WebRequest`・`Invoke-RestMethod`・`Test-Connection`・`System.Net.*` の型・`Install-Module`・`Update-Help` を使わない。実行時の依存は無い | AC-419(点検) |
| 動的なコードの実行をしない | 製品のコードが、`Invoke-Expression`(別名 `iex` を含む)、`[scriptblock]::Create`、`Add-Type`(補助の部品の 1 か所を除く。その C# のソースは、リポジトリに置いた定数の文字列で、入力からは作らない)、`[System.Reflection.Assembly]::Load*`、`Import-Module`(自分のモジュール以外)、`New-Object -ComObject`、データのパスをドットソースする `. $path` を使わない。ネイティブの呼び出し(P/Invoke)は、補助の部品に置く `kill` などの、許可の一覧にある関数だけ | AC-420(点検) |
| シグナルの送信 | ジョブの子プロセスの pid だけ(0 以下にならない)、許可したシグナルだけ、1 回だけ(プロセスグループには送らない) | 4.10 節の単体の項目 |
| テスト専用の経路を製品に持たない | 製品の入口に、わざと異常終了するコマンドや、テスト用の環境変数・隠しオプションがない。配布物に `tests/` と `build/` の成果物を含めない | AC-422(点検)、AC-526、6 章 |
| 変換の副作用 | 変換の部品(`Convert/`)が `System.Diagnostics.Process`・ネットワークの型を使わない。ファイルを開くのは、入力ファイルを読む入口だけで、本体(文字列から文字列への変換)と書き出しは、ファイルにも環境変数にもカレントディレクトリにも触れない。`convert` は入力以外のファイルを作らず、カレントディレクトリの変更と環境変数に左右されない | AC-418 |
| 供給網(開発時の依存) | 実行時の依存は無い。開発時の依存(Pester、PSScriptAnalyzer)は、版を固定し、取得したファイルのハッシュを検査する(8 章)。インストールのたびに最新を取らない | 5 章、AC-478 |
| 配布物の完全性 | チェックサムのファイル(`SHA256SUMS`)を付ける(署名はしない。転送の破損の検知まで。Authenticode 署名は行わない。6.5 節) | 6 章 |
| 実行ポリシーの回避を製品が行わない | 製品のコードが `Set-ExecutionPolicy` を呼ばない。ランチャーが `-ExecutionPolicy` を渡すかどうかは 07 章に従い、Windows の利用者向けの注意(6.5 節)と矛盾しない | AC-561 |

### 2.4 保守性

| 項目 | 基準 | 受け入れテスト |
|------|------|----------------|
| OS 固有コードの隔離 | OS ごとの分岐(`$IsWindows`・`$IsLinux`・`$IsMacOS`、`[System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform`、`[System.Environment]::OSVersion`)は、`src/Conductor/Private/Runner/Get-ConductorPlatformKind.ps1` の 1 か所だけ(OS 別の起動の準備・転送・終了の読み取りは、プラットフォームの層。07 章)。ほかの部品は、その関数の戻り値(UNIX 系か Windows か)と、`Deps` の `SyncDirectory`(`-SyncDirectory` のスクリプトブロック。06 章)だけを使う | AC-420 |
| 外部の入力で異常終了しない書き方 | 2.2 節 | AC-421 |
| 静的検査 | PSScriptAnalyzer(`PSScriptAnalyzerSettings.psd1`)で、Error と Warning が 0 件。禁止コマンドの検査(5.4 節の `forbidden`)で 0 件。`Invoke-Formatter` の差分が 0。`Test-ModuleManifest` が成功 | AC-480、AC-483、AC-518 |
| 命名 | 関数は承認された動詞を使う `Verb-ConductorNoun`(PSScriptAnalyzer の `UseApprovedVerbs`)。パラメーターは PascalCase、ローカル変数は camelCase、スクリプト・モジュールの変数は `$script:PascalCase`、定数は PascalCase。独自の規則は設けない | (規約。`PSScriptAnalyzerSettings.psd1`) |
| 部品の分け方 | `Json`、`Definition`、`Convert`、`RunId`、`Flow`、`State`、`Runner`、`Report`、`Cli` の各ディレクトリ(`src/Conductor/Private/`。[02](02-architecture.md))。依存の向き `Cli` → `Runner`/`State`/`Report` → `Flow`/`Definition`/`Convert`/`RunId` → `Json` を、`tests/Inspect/` の検査(構文木で、関数の呼び出しを走査)で強制する(PowerShell には名前空間の強制がない) | AC-514 |
| ソースへの著作権表示 | 各ソースファイル(`.ps1`、`.psm1`、`.psd1`、`.yml`、`.sh`、`.cmd`)の先頭に、Apache License 2.0 のヘッダ(`Copyright 2026 agwlvssainokuni`)を置く | AC-423(点検) |

### 2.5 サポートする PowerShell の版

- 実装時点(この文書の作成日 2026-11-20)で、サポート中の LTS は PowerShell 7.6 だけである(7.4 は 2026-11-10 にサポートが終わった。1 章の確認)。したがって、**この文書の作成時点の例では、`PS_MIN` と `PS_MAX` は、どちらも `7.6`(最小と最新が同じ)になる**。7.7 はプレビューで、対象にしない。次の LTS が出ると、「最新の安定版と、サポート中の最小の LTS」が分かれる。**具体的な版は実装時に確認して決める。**
- `PS_MIN` と `PS_MAX` は、`build/Versions.psd1`(`MinPowerShell`、`MaxPowerShell`。例は `'7.6'`)に書く(版の出所)。モジュールのマニフェストの `PowerShellVersion` は `MinPowerShell` と等しくする。ソースの先頭の `#requires -Version <MinPowerShell>` も同じ。CI は `build/Versions.psd1` を読んで、使う `pwsh` の版を決める(5.1 節)。一致を `Invoke-Checks consistency`(5.4 節)が検査する。
- 配布物は、PowerShell を同梱しない。利用者は `pwsh`(`PS_MIN` 以上)を入れる。Windows PowerShell 5.1(`powershell.exe`)では動かない(ランチャーは `pwsh` を呼ぶ。07 章)。

---

## 3. テスト構成

### 3.1 方針

- 方法論は test-after(実装してから、層ごとにテストを書いて実行する)。単体テストと、ランチャーを外部のプロセスとして起動する受け入れテストを持つ。
- テストの枠組みは **Pester 5.x**(`Describe`/`Context`/`It`、`BeforeAll`/`BeforeEach`/`AfterEach`/`AfterAll`、`BeforeDiscovery`、`-TestCases`、`-Tag`、`-Skip`、`Mock`、`InModuleScope`、`$TestDrive`)。Pester 6 は、取得した文書(1 章)が存在を示すが、5.x との互換と、5.x からの移行の費用を確かめていないので、採らない(版は 8 章で固定)。アサーションは `Should`。**文字列の比較は `Should -BeExactly`(大文字小文字を区別する)を使う。`Should -Be` は、文字列を大文字小文字を区別せず比べる**(Pester の挙動の記憶による。実装時に確認)ので、期待の文言には使わない。
- テスト用の別のライブラリ(モックのライブラリ、`PSFramework`、`Assert` モジュールなど)は入れない(8 章)。Pester 自身の `Mock` は使ってよい。偽の `Deps`(3.7 節)は手で書く。
- Pester 5 は、**検出(Discovery)の段階**で `Describe`/`Context`/`It` の構造と `-Skip`・`-TestCases` の式を評価し、**実行(Run)の段階**で `BeforeAll`/`It` の本体を実行する。`-Skip` の条件の変数や、`-TestCases` の表は、`BeforeDiscovery`(またはファイルの先頭)で作る。`BeforeAll` で作った変数は、検出では見えない(Pester の挙動の記憶による。実装時に確認)。
- 一時ディレクトリは、各テストが `New-TestTemp`(`tests/support/Harness.ps1`)で作る(`[System.IO.Path]::Combine([System.IO.Path]::GetTempPath(), 'conductor-test-' + [guid]::NewGuid().ToString('N'))` を作成し、`AfterEach` で消す。以下 `T`)。**テストごとに別のディレクトリ**なので、ステートファイルの名前(`<定義>.state`)は、テストの間で衝突しない。受け入れテストで子プロセスに渡すパスは、`T` の実際のパス(文字列)を使う。**`TestDrive:`(PowerShell のドライブ)の相対のパスは、子プロセスや .NET の `System.IO` のメソッドからは解決できない**(`TestDrive:` は PowerShell の中だけのドライブ。`$TestDrive` 変数は実際のパスの文字列を持つ。記憶による。実装時に確認)ので、受け入れテストでは使わない。プロセス内の単体テストでは、`$TestDrive` が示す実際のパスを使ってよい。環境変数は、テストのプロセスの `$env:` を直接書き換えず(`finally` で戻す補助 `Use-TestEnv` を使う)、子プロセスの `ProcessStartInfo.Environment` に渡す。
- 並列実行: Pester 5.x は、ファイルを 1 つのプロセスで**逐次**に実行する(Pester 5 に、組み込みの並列実行がないことの記憶による。**未確認**)。したがって、受け入れテストは、1 つのテストが 1 つの子プロセス群を扱い、テストの間で CPU を取り合わない。CI の速さは、OS ごと・種別ごとのジョブの並列で稼ぐ(5 章)。テストを並列にする場合(`Start-ThreadJob`・`ForEach-Object -Parallel` で、複数の `Invoke-Pester` を走らせるなど)は、(1) `T` が別なのでステートの衝突は起きない、(2) 時間の基準(2.1 節)の測定は並列にしない、(3) プロセス全体に作用するもの(環境変数、カレントディレクトリ、シグナルの登録)は並列にしない、を守る。
- 入力と期待の組が多いものは、**テーブル駆動**(`It '<name>' -TestCases @(...)`)で書く。ID(`AC-xxx`)は、`It` の名前に含める(例: `It 'AC-092 一重引用符が閉じていない'`)。`-TestCases` の各ケースの表示名にも `AC-092` を含める。`Invoke-Pester -FullNameFilter '*AC-092*'` のように、名前で選べる(`-FullNameFilter` の存在は、実装時に確認。**未確認**)。
- 行カバレッジの下限は **80%**(5 章)。
- プロセスを起動するテストは、60 秒の上限で、ハングだけを検知する(3.3 節)。
- 乱数が要るテストは、固定の種の擬似乱数 `Xorshift32` を使う(`Get-Random -SetSeed` は、版による出力の違いを避けるため使わない)。PowerShell の 16 進のリテラル `0xFFFFFFFF` は、**`[int]` の -1 になる**(32 ビットの符号つき整数のリテラルの規則。記憶による。実装時に確認)ので、32 ビットの全ビットが 1 のマスクは `[uint64]4294967295` と書く。実装(`tests/support/Xorshift32.ps1`。クラスは、Pester の検出と実行の分離で、定義が見えなくなる問題を避けるため使わず、ハッシュテーブルと関数で書く):

```powershell
Set-StrictMode -Version Latest

function New-Xorshift32 {
    param([Parameter(Mandatory)][uint64]$Seed)
    $state = if ($Seed -ne [uint64]0) { $Seed } else { [uint64]2654435769 }   # 0x9E3779B9
    return @{ State = [uint64]$state }
}

function Get-Xorshift32Next {
    param([Parameter(Mandatory)][hashtable]$Rng)
    $mask = [uint64]4294967295
    [uint64]$x = $Rng.State
    $x = $x -bxor (($x -shl 13) -band $mask)
    $x = $x -bxor ($x -shr 17)
    $x = $x -bxor (($x -shl 5) -band $mask)
    $Rng.State = $x
    return [uint64]$x
}

# 0 以上 n 未満の整数(わずかな偏りは許す。試験の入力を作るだけ)
function Get-Xorshift32Below {
    param([Parameter(Mandatory)][hashtable]$Rng, [Parameter(Mandatory)][int]$N)
    $v = Get-Xorshift32Next -Rng $Rng
    return [int]($v % [uint64]$N)
}
```

### 3.2 部品ごとの単体テスト

`src/Conductor/Private/<部品>/` の各ディレクトリは、`tests/Unit/<部品>/` に単体テスト(`*.Tests.ps1`。タグ `Unit`)を置く。非公開の関数をテストするため、テストは `BeforeAll` でモジュールを読み込み(`Import-Module <src/Conductor> -Force`)、`InModuleScope Conductor { ... }` の中で呼ぶ(または `Mock -ModuleName Conductor`)。

| 部品 | 主な試験対象 | 方法 | 主な AC |
|------|--------------|------|---------|
| `Json`・`Definition` | 定義の読み込み・検査・型・エラーの種類と行番号。`.psd1` と JSON の読み取り(構文木の検査、重複キー、非文字列キー、深さと大きさの上限)。`command` の分割。整数の字句検査(真偽値・小数・範囲外) | テーブル駆動。`tests/fixtures/*.psd1`・`*.json` の入力例 | AC-001〜098、AC-450〜、AC-502、AC-522〜525 |
| `Convert` | `.psd1` ⇔ JSON の変換、往復、表せない値、暴走する入力の上限 | テーブル駆動。往復の性質テスト(固定の種) | AC-206〜238 |
| `RunId` | run ID の検査(文字種と長さの規定は [05](05-execution-flow.md))と決定の順序 | テーブル駆動 | AC-239〜260 |
| `Flow` | 純粋な判断: 分類 10 項目、遷移、再開、リトライの待ち時間(`[uint64]` の飽和)、終了コードの範囲合わせ | テーブル駆動。ファイル・プロセスなし | AC-268、AC-392〜394、AC-501 |
| `State` | ステートの読み書き(実ファイル、`T`)、整合性検査(14 段階)、原子的な置換(06 章の決定)、reset(`--all`、シンボリックリンク) | 実ファイル。書き込みの失敗は、置き換え先をディレクトリにして起こす | AC-302〜324 |
| `Runner` | 子プロセスの起動と待ち、環境変数・作業ディレクトリの引き継ぎ、シグナルの転送(1 回だけ・許可した番号だけ)、シグナル名と番号の対応、補助の部品(`Add-Type`)の用意 | テスト用ジョブ `TJ` を実際に起動 | AC-099〜116、AC-280〜292、AC-504、AC-505 |
| `Report` | 表示の全種類の文言(英語・日本語)、ロケールの決定、色、エスケープ、`watch` の表 | テーブル駆動 | AC-350〜388 |
| `Cli` | 引数の文法、各サブコマンドの流れ、終了コードの決定 | **偽の `Deps`**(3.7 節)。ファイル・プロセス・シグナルなし | AC-125〜、AC-293〜300、AC-325〜349 |
| `tests/support/` | 偽の部品、一時ディレクトリの補助、記録の読み取り、`Xorshift32` | (試験の道具。単体試験は不要。カバレッジの対象は `src/` だけなので、自然に除かれる) | — |

### 3.3 受け入れテストの仕組み(外部プロセスの統合テスト)

受け入れテストは、ランチャーと、テスト用ジョブ `TJ`(3.4 節)を、`System.Diagnostics.Process`(`ProcessStartInfo`。`UseShellExecute = $false`)で起動して確かめる。置き場所は **`tests/Acceptance/`** の `*.Tests.ps1`(タグ `Acceptance`)。機能ごとにファイルを分ける。

| ファイル | 内容 |
|----------|------|
| `Validate.Tests.ps1` | 4.1 節、4.2 節、450〜 の定義の検査 |
| `Run.Tests.ps1` | 4.3 節、4.4 節(`run`、`pipeline`) |
| `Aux.Tests.ps1` | 4.5 節、4.6 節(`watch`、`reset`)、4.8 節(run ID) |
| `Convert.Tests.ps1` | 4.7 節 |
| `Retry.Tests.ps1` | 4.9 節 |
| `Signal.Tests.ps1` | 4.10 節(UNIX 系の項目は `-Tag Unix`、`-Skip:$script:NotUnix`) |
| `StateErrors.Tests.ps1` | 4.11 節 |
| `Args.Tests.ps1` | 4.12 節〜4.15 節 |
| `Nfr.Tests.ps1` | 4.16 節(堅牢性) |
| `PowerShell.Tests.ps1` | 4.20 節(PowerShell に固有の項目) |
| `Demo.Tests.ps1` | 4.17 節(デモ。UNIX 系と Windows) |
| `Dist.Tests.ps1` | 4.18 節(配布物。展開した成果物を、環境変数 `CONDUCTOR_DIST_DIR` で受ける。なければスキップ) |
| `tests/Perf/*.Tests.ps1` | 2.1 節の計測(`CONDUCTOR_PERF=1` のときだけ) |
| `tests/Inspect/*.Tests.ps1` | 点検(構文木の走査、マニフェストの検査。3.6 節) |

`conductor` の起動方法の決め方(`Get-ConductorLauncher`。`tests/support/Harness.ps1`。ファイルごとに 1 回):

1. 環境変数 `CONDUCTOR_BIN` があれば、そのパスを**ランチャーとして直接実行する**(配布物のランチャーを試験するため。AC-446)。UNIX 系は `bin/conductor`(実行権限つき)、Windows は `bin\conductor.cmd`。
2. なければ、リポジトリの `bin/conductor`(Windows は `bin\conductor.cmd`)を実行する。
3. 使う `pwsh` は、`PATH` の先頭にあるもの。最小の版と最新の版を別々に試すときは、環境変数 `CONDUCTOR_PWSH_DIR`(`pwsh` のあるディレクトリ)があれば、子の `PATH` の先頭に足す(ランチャーが `pwsh` を `PATH` から探すことが前提。[07](07-process-and-signals.md))。
4. テスト用ジョブは、`tests/jobs/tj.ps1`(環境変数 `CONDUCTOR_TESTJOB` で別のパスを指定できる)。

各テストは、`T` をカレントディレクトリにして、`$psi = [System.Diagnostics.ProcessStartInfo]::new($exe)` に、`ArgumentList.Add()` で引数を足し、`WorkingDirectory = $T`、`RedirectStandardOutput = $true`、`RedirectStandardError = $true`、`RedirectStandardInput = $true`、`UseShellExecute = $false` で実行する(`stdin` は、既定で、すぐ閉じる(空))。`Environment` は、いったん `Clear()` して 3.5 節の既定の環境変数だけにする(親の環境を引き継がない)。**標準出力と標準エラーは、2 つとも非同期に読み切る**(片方を読まずに他方を待つとデッドロックするため。`$p.StandardOutput.ReadToEndAsync()` と `$p.StandardError.ReadToEndAsync()` の 2 つの `Task` を作ってから `WaitForExit` する)。バイト列で比べる項目(AC-494 など)は、`BaseStream` から読む。
60 秒を超えたら、`$p.Kill($true)`(プロセスの木ごと)で強制終了し、記録(`pid=`)に書かれたジョブの pid があればそれも強制終了して、テストを失敗にする(`Invoke-ConductorProcess` の `WaitForExit(60000)`)。加えて、CI のジョブに `timeout-minutes` を置き、全体のハングに備える(5 章)。

### 3.4 テスト用ジョブ(`tests/jobs/tj.ps1`、以下 `TJ`)

受け入れテストで使う、小さな PowerShell スクリプト。配布物には含めない(6 章)。**起動の費用の扱い**: `TJ` は `pwsh` で動くので、1 回の起動が、`pwsh` の起動を伴う(数百 ms の可能性。未測定)。次のように抑える。

1. `TJ` は、`pwsh -NoProfile -NonInteractive -NoLogo -File <tj.ps1> <引数...>` の形で起動する(`-File` なので、スクリプトの `exit N` が終了コードになる。確認: 1 章)。モジュール `Conductor` を読み込まない。`Add-Type` を使うのは、シグナルを扱うモード(`sleep`、`trap`)だけ。
2. 定義の中で、`TJ` は、**5 つの語**を表す: `"<pwsh の絶対パス>" -NoProfile -NonInteractive -NoLogo -File "<tj.ps1 の絶対パス>"`(パスは二重引用符で囲む。パスに空白があってもよい)。配列形式のときは、5 つの要素。例: `command = 'TJ exit 3'` は、定義の中で `command = '"/usr/bin/pwsh" -NoProfile -NonInteractive -NoLogo -File "/repo/tests/jobs/tj.ps1" exit 3'` になる(Windows のパスの末尾に `\` を置かない)。
3. 大量のジョブを要するテスト(AC-402 など)は、`TJ` を使わず、偽の `Deps` で行う。
4. 性能の測定では、`TJ` 自身の起動の費用を基準として測り、差し引く(2.1 節)。
5. 環境変数 `POWERSHELL_TELEMETRY_OPTOUT=1` を、子の既定の環境に足す(テレメトリの無効化。変数名は実装時に確認。**未確認**)。
6. 起動の費用が問題になる項目(終了コードの値だけを見る項目)の速い版として、UNIX 系の `["sh", "-c", "exit <n>"]`、Windows の `["cmd.exe", "/c", "exit <n>"]` を使ってよい(AC に明記した項目だけ)。

定義を書き出す補助(`Write-TestDefinition`)は、すべての文字列を `.psd1` の単一引用符の文字列(`'` は `''`)で出す。JSON の定義は、文字列を JSON のエスケープで出す(テストの補助なので `ConvertTo-Json` を使ってよい)。

```text
tj.ps1 [--out <ファイル>] <モード> [引数...]
```

- `--out <ファイル>` が先頭にあれば、**記録の行**をそのファイルに追記する(`[System.IO.File]::AppendAllText($file, $line + "`n", [System.Text.UTF8Encoding]::new($false))`。1 行 1 件)。なければ標準出力に書く。ジョブの標準出力を被試験体越しに読めない場合があるため、結果はファイルで受け取る。
- 引数は、`$args` で受ける(`param()` ブロックを書かない。書くと、`-File` の後ろの `-v` や `--out` のような語が、パラメーターに束縛される可能性がある。確認: 1 章の `-File` の引数の記述は、リテラルの文字列として渡す、までで、スクリプト側の束縛の細部は**未確認**)。
- 記録に出す文字列(値)は、`Q(s)` で表す: 二重引用符で囲み、`"` と `\` の前に `\` を置き、改行は `\n`、復帰は `\r`、タブは `\t` と書く(JSON のエスケープとは別。非 ASCII はそのまま。他の制御文字は変換しない)。
- 未知のモードは、終了コード 2。
- 終わり方は、`exit <n>`。`say` は、書き込みの `Flush()` を済ませてから終わる。UNIX 系で、終了コードは下位 8 ビットだけが親に届く(`exit 300` が 44)。Windows の `pwsh -File` で、`exit 256` や負の値が、プロセスの終了コードとしてどう届くかは**未確認**(AC-111、AC-112)。

| モード | 動作 |
|--------|------|
| `exit <n>` | 終了コード n で終わる(省略・整数でないときは 2) |
| `run <label> <n>` | 記録 `ran=<label>` を出し、終了コード n(省略・整数でないときは 0) |
| `echo <args...>` | 各引数を、記録 `arg[i]=Q(引数)`(i は 0 から)として 1 行ずつ出し、終了コード 0 |
| `env <NAME>...` | 各環境変数を、設定されていれば `env[NAME]=set:Q(値)`、なければ `env[NAME]=unset` として出す(`[System.Environment]::GetEnvironmentVariable($name) -ne $null`。`$null` は左辺に置く)。終了コード 0 |
| `cwd` | カレントディレクトリ(`[System.IO.Directory]::GetCurrentDirectory()`)を `cwd=Q(パス)` として出す。終了コード 0 |
| `pid` | 自分のプロセス識別子を `pid=<N>`(`$PID`)として出す |
| `say <out> <err> [n]` | `<out>` を標準出力に、`<err>` を標準エラーに、改行を付けずにそのまま書き(`[Console]::OpenStandardOutput()` に UTF-8 のバイトを書いて `Flush()`)、終了コード n(省略は 0)。`--out` に従わない |
| `sleep [秒]` | `pid=<N>` と `ready` を記録し、眠る(短い間隔の繰り返し)。秒があれば、その秒のあと終了コード 99。なければ眠り続ける。SIGTERM・SIGINT・SIGHUP を受けたら、**`Add-Type` で用意した C# の補助の部品**のハンドラが、自分のシグナルのハンドラを解除し、`kill(getpid(), 受けたシグナル)`(P/Invoke)で自分に同じシグナルを送り直す(シグナルで終了したように観測されるようにする。確認は実装時。10 章)。PowerShell のスクリプトブロックを、シグナルのコールバックにしない(別のスレッドには実行空間がなく、失敗する。07 章)。Windows では、`Console.CancelKeyPress` で登録できるものだけを登録する |
| `trap <ms> <code>` | `pid=<N>` と `ready` を記録する。SIGTERM・SIGINT・SIGHUP を受けたら(補助の部品。`Cancel = true`)、記録 `trapped` を出し、`<ms>` ミリ秒待ってから、記録 `done` を出し、終了コード `<code>` で終わる(待ちの本体は、PowerShell の側が、補助の部品の旗を読んで行う。シグナルで中断された後も、ジョブが終わるまで `conductor` が待つことを確かめる。AC-505) |
| `flaky <counter-file> <fails> <code>` | `<counter-file>` の整数(なければ 0)に 1 を足して書き戻す。足した値が `<fails>` 以下なら終了コード `<code>`、超えたら 0 |
| `timing` | `start_ns=<UTC のナノ秒>`、`end_ns=…` を、標準出力に書く(性能の測定用。`start_ns` は最初の行で、`end_ns` は終了の直前で取る。値は `[System.DateTime]::UtcNow.Ticks`(100 ns 単位)を 100 倍した数。`[long]` の範囲に収まる) |
| `burn <ms>` | 指定のミリ秒(既定 100)、`[System.Diagnostics.Process]::GetCurrentProcess().TotalProcessorTime` で測って CPU を使い続けて終わる(CPU 時間の測定用) |
| `stdin` | 標準入力(`[Console]::OpenStandardInput()`)を終わりまで読み、読んだ内容を UTF-8 で復号して(不正なバイトは U+FFFD)記録 `stdin=Q(内容)` として出し、終了コード 0(標準入力の引き継ぎの確認。AC-509) |
| `big <MiB>` | 標準出力(`[Console]::OpenStandardOutput()`)に、指定の MiB(既定 16)の `x` を、書き込みの完了を待ちながら書き、終了コード 0(AC-494) |
| `tty` | `/dev/tty` を開こうとして、開けなければ終了コード 9、開ければ 0(AC-535。UNIX 系) |
| `readline` | 標準入力から 1 行を読み、終了コード 0(AC-535) |

注入用の別のスクリプト `tests/jobs/inject.ps1 <名前> <引数...>`(3.6 節の「注入」): モジュール `Conductor` を読み込み、失敗の仕掛け(例外を投げる偽の `Deps`、先に登録するシグナルのハンドラ、起動の前の遅延)をしてから、モジュールの内側の入口(`& (Get-Module Conductor) { Invoke-ConductorMain ... }`。モジュールのスコープでコマンドを実行する、呼び出し演算子の形)を呼び、その戻り値で `exit` する。製品に試験用の経路を持たせずに、実プロセスで内部の誤りを起こせる。モジュールのスコープでの実行の書き方と、`Invoke-ConductorMain` の名前は、02 章に従う(**未確認**)。

補足:

- `sleep` モードで SIGQUIT を受けたときの、`pwsh` の既定の動作は、**公式の文書で確認できていない(未確認)**。したがって、AC-282 の SIGQUIT の項目は、`conductor` の終了コード 131 だけを確かめ、ジョブの終わり方は確かめない。
- UNIX 系の受け入れテストでは、シグナルで終わるジョブを `["sh", "-c", "kill -9 $$"]`(配列形式)で作る(`TJ` にモードを足さない)。
- `TJ` の `env`・`cwd` の記録の値は、`Q(s)` で出すので、OS によらず同じ書式になる。`TJ` は、標準出力・標準エラーを、`[Console]::OpenStandardOutput()`・`[Console]::OpenStandardError()` のストリームに UTF-8(BOM なし)で書く(`Write-Host`・`Write-Output` を使わない。コンソールの符号化に左右されない)。

`tests/jobs/tj.ps1` の骨組み(全モードの実装は、上の表のとおり。ここでは、共通部分と、`exit`・`flaky` の 2 つを示す):

```powershell
# Copyright 2026 agwlvssainokuni (Apache License 2.0 のヘッダ。全文は省略)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:OutFile = $null
$script:Utf8 = [System.Text.UTF8Encoding]::new($false)

function ConvertTo-TjQuoted([string]$s) {
    $t = $s.Replace('\', '\\').Replace('"', '\"').Replace("`n", '\n').Replace("`r", '\r').Replace("`t", '\t')
    return '"' + $t + '"'
}

function Write-TjRecord([string]$line) {
    if ($null -eq $script:OutFile) {
        $o = [Console]::OpenStandardOutput()
        $b = $script:Utf8.GetBytes($line + "`n")
        $o.Write($b, 0, $b.Length)
        $o.Flush()
    } else {
        [System.IO.File]::AppendAllText($script:OutFile, $line + "`n", $script:Utf8)
    }
}

function ConvertTo-TjInt([object]$s, [int]$fallback) {
    $v = 0
    if ($null -ne $s -and [int]::TryParse([string]$s, [System.Globalization.NumberStyles]::AllowLeadingSign,
            [System.Globalization.CultureInfo]::InvariantCulture, [ref]$v)) { return $v }
    return $fallback
}

function Invoke-TjFlaky([string[]]$a) {
    $path = $a[0]; $fails = ConvertTo-TjInt $a[1] 0; $code = ConvertTo-TjInt $a[2] 1
    $n = 0
    if ([System.IO.File]::Exists($path)) { $n = ConvertTo-TjInt ([System.IO.File]::ReadAllText($path, $script:Utf8).Trim()) 0 }
    $n++
    [System.IO.File]::WriteAllText($path, $n.ToString([System.Globalization.CultureInfo]::InvariantCulture), $script:Utf8)
    if ($n -le $fails) { return $code } else { return 0 }
}

$argv = @($args | ForEach-Object { [string]$_ })     # 単一要素でも配列にする
$start = 0
if ($argv.Count -ge 2 -and $argv[0] -ceq '--out') { $script:OutFile = $argv[1]; $start = 2 }
if ($argv.Count -le $start) { exit 2 }
$mode = $argv[$start]
$rest = if ($argv.Count -gt $start + 1) { @($argv[($start + 1)..($argv.Count - 1)]) } else { @() }
switch -CaseSensitive ($mode) {
    'exit'  { exit (ConvertTo-TjInt $(if ($rest.Count -gt 0) { $rest[0] } else { $null }) 2) }
    'flaky' { exit (Invoke-TjFlaky $rest) }
    # run / echo / env / cwd / pid / say / sleep / trap / timing / burn / stdin / big / tty / readline は、上の表のとおり
    default { exit 2 }
}
```

(`switch` は、既定で、大文字小文字を区別しないので `-CaseSensitive` を付ける。`$args` の要素を `[string]` にそろえるのは、`-File` の引数が文字列として渡る(確認: 1 章)ためで、型の自動変換に頼らないため。補助の部品(`sleep`・`trap` のシグナル)の C# のソースは、`tests/jobs/tj-signals.cs.txt` として、`Add-Type -TypeDefinition (ファイルの内容)` で使う。`PosixSignal.SIGHUP` などの列挙子の存在は、この文書では確認していない(未確認)。)

### 3.5 定義とステートの略記と、実行環境

**実行環境**

- 被試験体(ランチャー。3.3 節)と `TJ` を用意する。
- 各テストは、空の一時ディレクトリ(`T`)で行う。`T` の中に定義ファイル(既定の名前 `conductor.psd1`)を作り、`T` をカレントディレクトリにして被試験体を起動する。ステートファイルは、定義ファイルと同じディレクトリにできる(例: `T/conductor.psd1.state`)。
- 環境変数は、特に記さない限り、次のとおり。`LANG=C`。`LC_ALL`・`LC_MESSAGES`・`NO_COLOR`・`CONDUCTOR_RUN_ID`・`PSExecutionPolicyPreference`・`PSModulePath`(必要な場合を除く)・`POWERSHELL_UPDATECHECK` は未設定にする(`POWERSHELL_UPDATECHECK=Off` は、更新の確認を止める。変数名は実装時に確認。**未確認**)。`POWERSHELL_TELEMETRY_OPTOUT=1` を足す。`PATH` と、`HOME`、`TMPDIR`(UNIX 系)、Windows の `SystemRoot`・`ComSpec`・`PATHEXT`・`TEMP`・`TMP`・`USERPROFILE`・`APPDATA`・`LOCALAPPDATA` は、親から引き継ぐ(`pwsh` の起動に要る場合がある)。
- 標準入力は空(`StandardInput.Close()`)にする。標準入力を使う項目は、書き込んでから閉じる。
- パスに空白や非 ASCII を含めない(特に記す場合を除く)。

**定義の略記** — 前提の「定義」は、次の略記で書く。

```text
P(<パイプライン名> [run_id_env=<ENV>] [remove_on_complete=<true|false>]; <ジョブ>, <ジョブ>, ...)
```

| 略記 | 展開 |
|------|------|
| `P(p; ...)` | `.psd1` のハッシュテーブル: `format_version = 1`、`pipeline = @{ name = 'p' }`、続いて `job = @( ... )` に、ジョブのハッシュテーブル(`@{ id = ... }`)を、書いた順に並べる |
| `<id>:<n>` | `job` の要素に `id = '<id>'`、`command = 'TJ exit <n>'` |
| `<id>:<n>{...}` | 同上に、波括弧の中の修飾を足す |
| `w=[…]` | `warning_exit_codes = @(…)` |
| `s=[…]` | `success_exit_codes = @(…)` |
| `on=success->X` / `on=warning->X` | `transitions = @( @{ on = 'success'; next = 'X' } )`(X はジョブID か `@end`) |
| `retry=(exit_codes=[…], max=N, interval="…", multiplier=M, max_interval="…")` | `retry = @{ ... }` のハッシュテーブル(書いたキーだけ。`exit_codes = @(…)`) |
| `cmd=<文字列>` | `command` を、その文字列にする(`TJ` は 3.4 節の語。必要なら二重引用符で囲む)。`id:n` の `n` は使わない |
| `argv=[…]` | `command` を、その文字列の配列(`@('…', '…')`)にする。`TJ` を書くときは、3.4 節の 5 つの要素にする |
| `O` | 記録の出力ファイル(`T/out.txt` など。`--out` に渡す) |

例: `P(p; a:0, b:7{w=[3]}, c:0{on=success->@end})` は、次の定義(`TJ` は、3.4 節の語の略)。

```powershell
@{
    format_version = 1
    pipeline = @{ name = 'p' }
    job = @(
        @{ id = 'a'; command = 'TJ exit 0' }
        @{ id = 'b'; command = 'TJ exit 7'; warning_exit_codes = @(3) }
        @{ id = 'c'; command = 'TJ exit 0'; transitions = @( @{ on = 'success'; next = '@end' } ) }
    )
}
```

JSON の定義(`conductor.json`)で試験する項目は、同じ構造を JSON で書く。キー名と構造は、`.psd1` と同じ([03](03-definition-format.md))。**1 要素の配列は、`.psd1` では `@(3)` と書く(`3` だけだと、配列ではなく整数になり、`wrong value type`。AC-554)。** 単一引用符の中で `@` は特別でない(`'@end'`)。

**ステートの略記** — ステートの期待は、次の略記で書く。`end` と `completed` は、書かなければ `false`。`resume` は、書かなければ「キーがない」。

```text
S[<id>=<状態>(<終了コード>), ... ; resume=<id>, end=<true|false>, completed=<true|false>, run_id=<id>]
```

状態は `pending`・`success`・`warning`・`error`・`skipped`。スキップは `skipped/byTransition` か `skipped/byEnd`。終了コードがなければ括弧を省く。例: `S[a=success(0), b=error(7), c=pending ; resume=b]`。ステートファイルの書式(JSON)は [06](06-state-file.md)。受け入れテストは、ステートファイルを(テストでは)`ConvertFrom-Json -AsHashtable`(または `[System.Text.Json.JsonDocument]`)で読み、論理項目(`pipeline_name`、`job_order`、`job` の各要素の `id`・`status`・`skip_reason`・`exit_code`、`resume_job_id`、`end_reached`、`completed`、`run_id` など)を比べる。**`ConvertFrom-Json` は、重複キー・巨大整数・日時文字列などで要件に合わない(5 章の表。製品のコードでは使わない。AC-421)**ので、テストでも、整数の型(`[long]`)・配列の単一化(`-NoEnumerate` と `@()` で包む)に注意して使い、バイト列で比べる項目は文字列として比べる。

### 3.6 判定の書き方と区分

| 欄 | 意味 |
|----|------|
| 実行 | 被試験体の引数。`T` をカレントディレクトリにする。`--config` を省くと `T/conductor.psd1` |
| 終了 | 終了コード |
| OUT | 標準出力。「空」は 0 バイト。引用は完全一致。「含む」は部分一致 |
| ERR | 標準エラー。同上。複数行は `⏎` で区切って書く(行の終わりの改行は省く) |
| 前提 | 実行の前に作るもの(定義、ステート、環境変数) |

- どのテストも、**60 秒の上限**で実行する(ハングの検知)。
- 「ジョブが実行された」は、`TJ run <label> <n>` の記録 `ran=<label>` で確かめる(`O` を読む)。「実行されない」は、その記録がないこと。
- 「ステートが変わらない」は、ステートファイルの内容(バイト列)が、実行の前後で同じこと。「ステートがない」は、ファイルがないこと。
- 「UNIX 系」「Windows」「非 root」と書かれた項目は、その環境でだけ行う(3.10 節のタグ)。
- 文字列の比較は、`Should -BeExactly`。終了コードの比較は、`[int]` どうしで `Should -Be`(数値)。

| 区分 | 意味 |
|------|------|
| (自動) | 被試験体を外から実行して確かめる |
| 単体 | 内部の関数(分割、終了コードの範囲合わせ、エスケープ、判断の純粋な関数)を直接呼んで確かめる。外から再現しにくい入力のため |
| 注入 | 標準出力への書き込みの失敗、シグナル登録の失敗など、外から作れない失敗を、偽の `Deps`(3.7 節)で起こして確かめる。外から作れる範囲では、**テスト専用のスクリプト `tests/jobs/inject.ps1`**(3.4 節)を起動し、そのスクリプトが失敗の仕掛けをしてから、モジュールの内側の入口を呼ぶ(製品に試験用の経路を持たせない。AC-490) |
| 点検 | ソースや依存を、構文木の走査(`System.Management.Automation.Language.Parser`)、マニフェストの検査(`Import-PowerShellDataFile`。比較用)、ビルドした成果物の中身(`System.IO.Compression`)で調べて確かめる(実行ではない。`tests/Inspect/` に置く。検査の道具は、開発時の依存を増やさずに、PowerShell と .NET の標準の機能だけで書く) |
| 手動 | 人が実機で確かめて、記録を残す |
| 計測 | 性能の測定(2.1 節) |

### 3.7 偽の部品(`Deps`)を使うテスト

`Cli` の `Deps` は、次の役割を持つ([02](02-architecture.md)。値の形(ハッシュテーブルか、`[pscustomobject]` か、その中の `[scriptblock]`)は 02 章に従う)。`tests/support/Fakes.ps1` が、それぞれの偽物を作る関数を提供する(手書き。モックのライブラリは使わない)。

| 役割(`Deps` の項目) | 本物 | 偽物(`Fakes.ps1`。名前は 02 章) |
|---------------------|------|-------------------------------|
| 環境(`Env`) | `Get-ConductorEnvironmentVariable`(`[System.Environment]::GetEnvironmentVariable`)、`Get-ConductorStdoutIsTerminal`(`[Console]::IsOutputRedirected`)、OS の判定(`Get-ConductorPlatformKind`) | `New-FakeEnv`(環境変数の表。端末か・OS の種別を設定できる) |
| 時刻(`Clock`) | `[System.DateTimeOffset]::UtcNow` など | `New-FakeClock`(固定の時刻) |
| 定義(`Defs`) | `Definition/` の読み込み(`Read-ConductorDefinitionFile`)、run ID の環境変数名の緩い読み取り(`Get-ConductorRunIdEnv`) | `New-FakeDefs`(決めた結果を返す。または一時ディレクトリの実ファイルを読む) |
| ステート(`States`) | `State/`(一時ファイルと置き換えによる原子的な書き込み) | `New-MemStates`(メモリ上のステート。読み込み・書き込み・削除の失敗を起こせる) |
| 変換(`Converter`) | `Convert/` の変換の入口 | `New-FakeConverter` |
| プロセス実行・待ち(`Proc`) | `Runner/`(ジョブの実行、リトライの待ち、`command` の検査) | `New-ScriptProcess`(用意した結果を順に返す。起動の回数と引数を記録。待ちは、待ち時間(秒。`[uint64]`)を記録するだけで、すぐに返す。中断を返す設定がある) |
| シグナル(`Signals`) | 補助の部品(`Add-Type`)の旗の読み取りと登録 | `New-FakeSignals`(任意の時点で番号を立てる。準備の失敗・「登録済み」を返せる) |
| 標準出力・標準エラー(`Out`) | `Write-ConductorStream`(`[Console]::OpenStandardOutput()` と `[Console]::OpenStandardError()` から作った `Stream` を包む BOM なし UTF-8 の `StreamWriter`。戻り値は `[bool]`。08 章 1.3.2) | `New-BufSink`(文字列を溜め、`[bool]` を返す。「n 回目の書き込みで失敗する」設定で、書き込みが偽を返す場合を作る) |

偽の部品で確かめる項目(区分「注入」)は、AC-293〜300、AC-324、AC-366〜368、AC-485、AC-491 など。`Cli` の流れのテストは、偽の部品の上で、`run`・`pipeline`・`watch`・`reset`・`convert`・`validate` を呼び、終了コード・標準出力・標準エラー・`MemStates` のステートを比べる。
本物の部品との結合は、受け入れテスト(3.3 節)が確かめる。**Pester の `Mock` で製品の非公開の関数を置き換える場合**は、`Mock -ModuleName Conductor <関数名>` か `InModuleScope` の中で行う(モジュールの外から `Mock` しても、モジュールの内側の呼び出しは置き換わらない。Pester の挙動の記憶による。実装時に確認)。偽の `Deps` で足りるものは、`Mock` を使わない。

### 3.8 テストの補助(`tests/support/Harness.ps1`)

| 関数 | 役割 |
|------|------|
| `New-TestTemp` | `T` を作る(`AfterEach` で `Remove-TestTemp` が消す。消すのに失敗しても、テストを失敗にしない) |
| `Get-TestEnv -Extra` | 3.5 節の既定の環境変数に `Extra` を足したハッシュテーブルを返す |
| `Write-TestDefinition -Path -Source -Format` | 略記(3.5 節)の定義を、`.psd1` か JSON で書き出す。`.psd1` の文字列は単一引用符で出す |
| `Invoke-ConductorProcess -Arguments -Cwd -Env -Stdin -TimeoutSec 60` | 被試験体を `Process` で起動し、`[pscustomobject]@{ Code; Stdout; Stderr; StdoutBytes }`(`Code` は `ExitCode`。`Stdout`・`Stderr` は、バイト列を UTF-8 で復号した文字列。不正なバイトは U+FFFD。バイト列で比べる項目は `StdoutBytes`)を返す。60 秒で強制終了して失敗にする。標準出力・標準エラーは、並行して読み切る |
| `Start-ConductorProcess` | 実行中に操作する項目(シグナルを送る、標準エラーの行を待つ)のために、`Process` と、行ごとの待ち受け(`Wait-ConductorStderr -Pattern -TimeoutSec`)を返す |
| `Read-TestRecords -Path` | `TJ` の記録の行を読む(`[System.IO.File]::ReadAllLines`。空でも配列で返す: `@()`) |
| `Read-TestState -Dir -Name` | ステートファイルを読んで、ハッシュテーブルを返す |
| `Get-TestDirListing -Dir` | `T` のファイル名の一覧(序数でソート済み: `[System.Array]::Sort($a, [System.StringComparer]::Ordinal)`) |
| `Send-TestSignal -Process -Number` | UNIX 系で、シグナルを送る(`kill -<番号> <pid>` のコマンドを `/bin/kill` か `sh -c` で起動する。テスト側だけがシェルを使う。`Process.Kill` は強制終了なので使えない) |

### 3.9 Pester の設定と、実行の仕方

設定は、`build/Invoke-Tests.ps1`(5.4 節)が `New-PesterConfiguration` で作る。キー名は、1 章で確認した範囲(5.x での存在は実装時に確認):

| 項目 | 決め | 理由 |
|------|------|------|
| 実行する範囲 | `Run.Path` = `tests/Unit`・`tests/Acceptance`・`tests/Inspect`(種別ごとにジョブを分ける。`-Tag`/`ExcludeTag` で選ぶ)。`Run.Exit = $true`(失敗で 0 以外) | 5 章 |
| タグ | `Unit`、`Acceptance`、`Inspect`、`Perf`、`Slow`、`Unix`、`Windows`、`Linux`、`NonRoot`、`Manual` | `Filter.Tag`・`Filter.ExcludeTag` で選ぶ |
| 結果の出力 | `TestResult.Enabled = $true`、`OutputFormat = 'NUnitXml'`、`OutputPath = 'artifacts/test-<種別>-<OS>.xml'` | 失敗の記録をアーティファクトに残す |
| 出力の詳しさ | `Output.Verbosity = 'Normal'`(CI は `Detailed`) | |
| 条件付きのテスト | `It ... -Skip:$cond`。条件は、`BeforeDiscovery` で `tests/support/Marks.ps1` から読む(3.10 節) | 検出の段階で評価される |
| タイムアウト | Pester 5 に、個々のテストのタイムアウトの機能があるか、**未確認**。子プロセスを起動するテストは、`Invoke-ConductorProcess` の 60 秒で自分で止める。全体のハングは、CI の `timeout-minutes` で止める | |
| 警告 | テストのファイルも PSScriptAnalyzer の対象(`PSScriptAnalyzerSettings.psd1` の `ExcludeRules` で、テストに限って外すものを明示する) | 5 章 |
| カバレッジ | `CodeCoverage.Enabled = $true`、`Path = 'src/Conductor'`、`OutputFormat = 'JaCoCo'`、`OutputPath = 'artifacts/coverage.xml'`。対象は `src/` だけ(テスト・`build/`・ランチャーは除く)。**受け入れテストで起動した子プロセスの中のコードは、数えない**。下限は、行 80%(5.3 節) | 目標の率で失敗するかは未確認。検査は `Invoke-Checks coverage` |
| スナップショット | 使わない(期待は、文言そのものをテストに書く) | 文言の変更に気づくため |
| 性能の試験 | `tests/Perf` は、`CONDUCTOR_PERF` が `1` でなければ、`BeforeDiscovery` で `-Skip` | 通常の実行を遅くしない |

### 3.10 OS 別のテスト

- UNIX 系だけのテスト(シグナル、権限、シンボリックリンク、新しいセッション、疑似端末)は、`-Tag Unix` と `-Skip:$script:NotUnix`(`-not ($IsLinux -or $IsMacOS)`)。Windows だけのテスト(`.cmd` の安全策、Ctrl+C のチェックリストに対応する自動の部分)は、`-Tag Windows` と `-Skip:(-not $IsWindows)`。Linux だけのテスト(`/dev/full`、`/proc` による CPU 時間)は、`-Tag Linux` と `-Skip:(-not $IsLinux)`。これらの条件は `tests/support/Marks.ps1`(`BeforeDiscovery` でドットソースして、`$script:NotUnix` などに入れる)に定義して使う(OS の判定が、テストの中に散らばらないように)。`$IsWindows`・`$IsLinux`・`$IsMacOS` は、PowerShell 7 の自動変数(記憶による。実装時に確認)。
- 非 root のテスト(権限の拒否を起こす項目)は、`-Tag NonRoot` と、UNIX 系で `id -u` が `0` なら `-Skip`(`$script:IsRoot`。Windows では常に実行)を加える。
- スキップされた項目は、結果に「Skipped」として残る。CI では、OS ごとにスキップされる件数が、想定の範囲であることを、実装時に確認する。
- OS 固有の製品のコードは、プラットフォーム層(`Get-ConductorPlatformKind`、UNIX 系・Windows の起動の準備、`.cmd` の起動の組み立て、補助の部品の P/Invoke)に集める。他のファイルから見ると、OS の違いは見えない(AC-420)。
- どの OS でも、PSScriptAnalyzer・書式の検査・`Test-ModuleManifest` が通る(5 章)。

### 3.11 テストの実行

```powershell
./build/Install-DevDependencies.ps1                              # Pester・PSScriptAnalyzer を、固定の版で入れる(8 章)
./build/Invoke-Checks.ps1 -Check lint                            # PSScriptAnalyzer + 禁止コマンド + 書式 + マニフェスト(5.4 節)
./build/Invoke-Tests.ps1 -Kind Unit,Inspect                      # 単体 + 点検(3 OS で通る)
./build/Invoke-Tests.ps1 -Kind Acceptance                        # 受け入れ(ランチャーを起動)
./build/Invoke-Tests.ps1 -Kind Unit -Coverage                    # 行 80%(Linux。5.3 節)
$env:CONDUCTOR_PERF = '1'; ./build/Invoke-Tests.ps1 -Kind Perf   # 性能の測定(2.1 節)
$env:CONDUCTOR_BIN = '<展開した配布物のランチャー>'; ./build/Invoke-Tests.ps1 -Kind Acceptance   # 配布物の試験(AC-446)
```

`build/Versions.psd1` と `build/Dependencies.psd1` に、テストと検査の道具の版が入っている(8 章)。

---

## 4. 受け入れテスト一覧

ID は、`AC-` と通し番号。表は、ID・機能・前提・実行・期待の順。期待の標準エラーの文言は、特に記さない限り `LANG=C`(英語)。理由の文(日本語の固定の文)は、英語の表示でも日本語のまま。
`.psd1` と JSON に固有の項目は 4.19 節(AC-450〜487 と AC-576〜585)、PowerShell に固有の項目は 4.20 節(AC-488〜575)にまとめた。この節の定義は、特に記さない限り `.psd1` の `conductor.psd1`(略記は 3.5 節)。ID の番号は、この文書の中で一意で、表の中の並びは、ID の昇順とは限らない。

### 4.1 定義の検査(`validate`、`run`、`pipeline` 共通)

特に記さない限り、実行は `validate`(`--config` を省く)。「64」の期待は、終了 64、OUT 空、ステートファイルを作らない、ジョブを起動しない、を含む。
ERR の期待は、**最初の行**のうち、`conductor: error: <ファイル>[:<行>]: ` に続く種類の文([08](08-output-and-i18n.md))を示す。ファイル名は、`--config` に渡した表記のまま。行は、誤りの位置の行(1 始まり。LF の数で数える)。`.psd1` の行は、PowerShell の構文木の `IScriptExtent.StartLineNumber`、JSON の行は、自前の読み取りが数える。パーサーの検出位置に依存する項目は、10 章に挙げる。
基本の定義 `B0` は、`P(p; a:0)` とする。


| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-001 | 正しい定義 | `B0` | `validate` | 終了 0。OUT・ERR 空。`T/conductor.psd1.state` を作らない |
| AC-002 | validate の -v(1 ジョブ) | `B0` | `validate -v` | 終了 0。ERR = `[conductor] definition is valid (1 job)`。OUT 空 |
| AC-003 | validate の -v(複数) | `P(p; a:0, b:0, c:0)` | `validate -v` | 終了 0。ERR = `[conductor] definition is valid (3 jobs)` |
| AC-004 | validate の -v(日本語) | `P(p; a:0, b:0, c:0)`、`LANG=ja_JP.UTF-8` | `validate -v` | 終了 0。ERR = `[conductor] 定義は正しいです(ジョブ 3 個)` |
| AC-005 | 定義ファイルがない | ファイルなし | `validate --config missing.psd1` | 64。ERR = `conductor: error: missing.psd1: definition file does not exist: 指定の場所にファイルがありません` |
| AC-006 | 定義ファイルがない(日本語) | 同上、`LANG=ja_JP.UTF-8` | 同上 | 64。ERR = `conductor: error: missing.psd1: 定義ファイルが存在しません: 指定の場所にファイルがありません` |
| AC-007 | 通常のファイルでない | `T/d.psd1` がディレクトリ | `validate --config d.psd1` | 64。ERR の種類の文 = `cannot read definition file`(理由 `通常のファイルではありません`) |
| AC-008 | 空のファイル | (a) 長さ 0 の `conductor.psd1`(コメントだけのファイルも同じ)。(b) 長さ 0 の `conductor.json` | (a) `validate`。(b) `validate --config conductor.json` | 64。(a) `.psd1` では、空の入力(文が 0 個)は文法の誤り(`syntax error`。理由 `.psd1 の文法に誤りがあります`。位置は入力の末尾。03 章 4.2 節)。(b) JSON では空の入力は文法の誤り(`syntax error`。理由 `JSON の文法に誤りがあります`。位置は入力の末尾。03 章 5.3 節) |
| AC-009 | 文法の誤り(行つき) | `.psd1`(`conductor.psd1`): `@{`⏎`format_version = 1`⏎`pipeline = ]`⏎`}`。JSON(`conductor.json`): `{`⏎`"format_version": 1,`⏎`"pipeline": ]`⏎`}` | `validate`(JSON は `--config conductor.json`) | 64。ERR の最初の行が、`.psd1` は `conductor: error: conductor.psd1:3: syntax error: `、JSON は `conductor: error: conductor.json:3: syntax error: ` で始まる。ERR にパーサーのエラーの文が現れない |
| AC-010 | キーの重複(文法の誤り) | `.psd1`: `@{ format_version = 1`⏎`format_version = 1 }`、および大文字小文字だけ違う `format_version = 1`⏎`Format_Version = 1`。JSON: `{"format_version": 1,`⏎`"format_version": 1}` | `validate`(JSON は `--config conductor.json`) | 64。種類の文 = `syntax error`(行 2。AC-456) |
| AC-011 | 書式の版が 1 でない | `"format_version": 2` に `B0` の他を足す | `validate` | 64。`unsupported format_version` |
| AC-012 | 書式の版が整数でない | `"format_version": "1"`、`1.0`、`true`、`1e0` | `validate` | 64。`unsupported format_version`(整数のリテラルだけを整数と読む。AC-502) |
| AC-013 | 版の確認が先 | `format_version` なし、未知のキーと型の誤りを含む | `validate` | 64。ERR は 1 行だけ。種類の文 = `format_version is missing` |
| AC-014 | `jobs`(複数形)のキー | `B0` の `job` を `jobs` に | `validate` | 64。`unknown key` |
| AC-015 | 未知のキー(ジョブ) | `B0` の job に `"colour": "red"` | `validate` | 64。`unknown key`(行はそのキーの行) |
| AC-016 | 未知のキー(パイプライン) | `pipeline` に `"owner": "x"`(または `"remove_on_complet": true`) | `validate` | 64。`unknown key` |
| AC-017 | `pipeline` がない | `format_version` と `job` だけ | `validate` | 64。`wrong value type`(行 1) |
| AC-018 | `pipeline` がオブジェクトでなく配列 | `"pipeline": [{"name": "p"}]` | `validate` | 64。`wrong value type` |
| AC-019 | パイプライン名が空 | `"name": "  "` | `validate` | 64。`required value is empty` |
| AC-020 | ジョブがない | `job` を書かない(または `"job": []`) | `validate` | 64。`no jobs are defined` |
| AC-021 | ジョブIDの重複 | `P(p; a:0, a:0)` | `validate` | 64。`duplicate job id` |
| AC-022 | ジョブIDが不正 | ジョブID が `""`、または `"@x"` | `validate` | 64。`invalid job id` |
| AC-023 | 遷移先がない | `P(p; a:0{on=success->nowhere})` | `validate` | 64。`transition target job is not defined` |
| AC-024 | エラー結果への遷移 | `transitions` の `"on": "error"` | `validate` | 64。`no transition can be written for the error result` |
| AC-025 | `on` の値が不正 | `"on": "later"`、または `"on": "Success"` | `validate` | 64。`wrong value type` |
| AC-026 | 自分自身・前のジョブへの遷移 | `P(p; a:0{on=success->a}, b:0)`、`P(p; a:0, b:0{on=success->a})` | `validate` | 64。`transition target must be a later job or @end` |
| AC-027 | `@end` への遷移は正しい | `P(p; a:0{on=success->@end}, b:0)` | `validate` | 0 |
| AC-028 | 同じ結果の遷移の重複 | `"on": "success"` の遷移を 2 つ | `validate` | 64。`duplicate transition for the same result` |
| AC-029 | 0 を警告にできない | `P(p; a:0{w=[0]})` | `validate` | 64。`0 cannot be a warning exit code` |
| AC-030 | 成功と警告の重なり | `P(p; a:0{w=[3], s=[0,3]})` | `validate` | 64。`success and warning exit codes overlap` |
| AC-031 | 終了コードの型・範囲の誤り | `w=["x"]`、`w=[1.5]`、`w=[true]`(真偽値は整数ではない)、`w=[1.0]`(`1.0` は整数ではない)、`w=[99999999999999999999]`、`w=[2147483648]`、`w=[-2147483649]` | `validate` | 64。`wrong value type`(要素ごとに 1 行)。範囲の検査は、整数の字句から行う(`[double]` に落とさない。`[double]` に変換すると、`2147483648` 付近以外で別の整数に化ける値の見落としが起きる。AC-502) |
| AC-032 | 負の終了コード・空の成功コード・範囲の端 | `w=[-1]`、`s=[]`、`w=[2147483647]`、`s=[-2147483648]` | `validate` | 0(受理する) |
| AC-033 | `command` の型・欠落・空 | `command` なし / `""` / `"   "` / `[]` / `[""]` / `["  ", "x"]` / `5` / `["a", 5]` / `true` | `validate` | 64。なし・`5`・`["a",5]`・`true` は `wrong value type`。ほかは `required value is empty` |
| AC-034 | 配列形式の `command` の受理 | `argv=["my prog", "a b", "$HOME", ""]`、`argv=["echo", ""]` | `validate` | 0 |
| AC-035 | `run_id_env` の名前の形 | `run_id_env` が `"1A"` / `"A-B"` / `""` / `"A B"` / `"A=B"` / `"A.B"` | `validate` | 64。`invalid environment variable name`。整数 `1` なら `wrong value type`。`_a1`・`GITHUB_RUN_ID`・`A` は 0 |
| AC-036 | `remove_on_complete` の型 | `'true'` / `1` / `@($true)` / 裸の単語 `true` | `validate` | 64。`wrong value type`(裸の単語は、構文の誤りになる場合がある。03 章に従う)。`$true`・`$false`・省略は 0 |
| AC-037 | 誤りは位置の順に 1 行ずつ | `ME`(この節の末尾の定義。`.psd1`) | `validate` | 64。ERR は 4 行。順に `…:3: unknown key`、`…:13: wrong value type`、`…:18: unknown key`、`…:22: transition target job is not defined`。JSON で書いた `ME` では、行は 3、11、16、20(AC-475) |
| AC-038 | 行番号は CRLF でも同じ | `ME` を CRLF 改行で | `validate` | 同上(行番号が同じ。LF の数で数える) |
| AC-039 | BOM つきの定義 | `B0` の先頭に UTF-8 の BOM(`EF BB BF`)。`[System.IO.File]::ReadAllText` と `[System.Text.Encoding]::UTF8.GetString` は、BOM の扱いが異なる(前者は BOM を取り除くが、後者は U+FEFF を残す。記憶による)ので、バイト列から自前で取り除く。`Get-Content` は使わない | `validate` | 0 |
| AC-040 | 定義の大きさの上限(受理) | `B0` に空白を足して、ちょうど 1,048,576 バイト | `validate` | 0 |
| AC-041 | 定義の大きさの上限(超過) | 1,048,577 バイト(UTF-8 でなくてもよい) | `validate` | 64。`definition file is too large`(読み込む前に、`FileInfo.Length` でサイズを見る。全体をメモリに読んでから比べない) |
| AC-042 | UTF-8 でない定義 | 上限以内で、不正なバイト列を含む | `validate` | 64。`cannot read definition file`(復号は `[System.Text.UTF8Encoding]::new($false, $true)`(第 2 引数が `throwOnInvalidBytes`) で行い、不正なバイト列を U+FFFD に置換せず拒否する。`[System.Text.Encoding]::UTF8.GetString`・`[System.IO.File]::ReadAllText`・`Get-Content` の既定は、不正なバイトを U+FFFD にするので使わない) |
| AC-043 | 入れ子の深さ(受理) | `.psd1`: `@{ a = ` に `@(` を 32 個、`)` を 32 個、` }`(`format_version` なし。根のハッシュテーブルは数えず、32 段まで受理。[03](03-definition-format.md))。JSON: `{"a": ` に `[` を 32 個、`]` を 32 個、`}` | `validate`(JSON は `--config conductor.json`) | 64。`format_version is missing`(深さの検査は通った) |
| AC-044 | 入れ子の深さ(超過) | 同じ形で `@(`(JSON は `[`)を 33 個・1,000 個・100,000 個(閉じも書く。`Slow`) | `validate` | 64。`syntax error`(理由 `入れ子が深すぎます(上限は 32 段)`。行は 33 段目の開きの行)。スタックの溢れでプロセスが落ちない(AC-524) |
| AC-045 | パイプライン名の照合 | `P(p; a:0)` | `validate p` / `validate wrong` / `validate` | 0 / 64(`pipeline name does not match the definition`)/ 0(照合しない) |
| AC-046 | 名前の不一致の誤りに値を含めない | `"name": "SECRET-NAME"` | `validate other-name` | 64。ERR に `SECRET-NAME` も `other-name` も現れない |
| AC-047 | 誤りの文に入力の値を含めない | `format_version: 1`、`pipeline` の `name: p`、`job` の `id: a`、`"command": 5`、`"warning_exit_codes": ["SENTINEL-VALUE"]` | `validate` | 64。ERR に `SENTINEL-VALUE` が現れない |
| AC-048 | `command` の引用符が閉じていない | `P(p; a:0{cmd="TJ exit 0 'x"})` | `validate` | 64。ERR = `conductor: error: conductor.psd1: job 'a': command has an unclosed quote` |
| AC-049 | `command` が空になる | `cmd="''"` | `validate` | 64。`job 'a': command is empty` |
| AC-050 | `command` の誤りは全ジョブ報告 | 2 つのジョブがそれぞれ誤り | `validate` | 64。ERR は 2 行(ジョブごと) |
| AC-051 | `command` の誤り(日本語) | 同上(1 つ目の形)、`LANG=ja_JP.UTF-8` | `validate` | ERR = `conductor: error: conductor.psd1: job 'a': command の引用符が閉じていません` |
| AC-052 | `run` でも `command` の誤りは 64 でジョブは起動しない | 1 つ目が正しく、2 つ目が誤り | `run a` | 64。`ran=` の記録なし。ステートなし |
| AC-053 | 起動できないプログラムでも validate は 0 | `cmd="/no/such/program"` | `validate` | 0 |
| AC-054 | 壊れたステートでも validate は 0 | `B0`、`T/conductor.psd1.state` に `{garbage` | `validate p` | 0。ステートは変わらない |
| AC-055 | validate の引数の誤り | `B0` | `validate a b` / `validate --run-id x` / `validate --to psd1` | 68 |
| AC-056 | `run_id_env` が不正な定義は run でも 64 | `"run_id_env": "1A"` | `run a` | 64 |
| AC-057 | validate は副作用を持たない | `B0` の `command` が「`TJ run a 0`」(`O` に記録) | `validate` | 0。`O` が作られない。`T` のファイル一覧が実行の前後で同じ |

#### `retry` の検査

`R0` は、`P(p; a:75{retry=(exit_codes=[75], max=3, interval="1s")})`。各行は、`R0` の `retry` を変えて `validate` する。誤りは終了 64、種類の文は ERR の最初の行。

| ID | 機能 | 変更後の `retry` | 期待 |
|----|------|------------------|------|
| AC-058 | 受理(固定の間隔) | `R0` のまま | 0 |
| AC-059 | 受理(倍率と上限) | `"multiplier": 2`、`"max_interval": "1m"` を足す | 0 |
| AC-060 | 受理(境界) | `exit_codes: [1,255,127]`、`max: 1` と `max: 100`、`interval` が `1s`・`24h`・`86400s`・`1440m`、`multiplier` が 2 と 10、`max_interval` が `interval` と同じ | いずれも 0 |
| AC-061 | 受理(`.psd1` のカンマ区切りの配列) | `retry = @{ exit_codes = 75, 76; max = 3; interval = '1s' }`(配列を `@()` なしのカンマで書く)と、`exit_codes = @(75)` | 0 |
| AC-062 | `retry` がオブジェクトでない | `"retry": 5`、または `"retry": [{"exit_codes": [75], "max": 3, "interval": "1s"}]` | 64。`retry must be a map` |
| AC-063 | 必須のキーがない | `"retry": {}` | 64。`a required key of retry is missing` が 3 行(`exit_codes`、`max`、`interval` の順) |
| AC-064 | 未知のキー | `"retries": 2` を足す | 64。`unknown key` |
| AC-065 | `exit_codes` が空 | `"exit_codes": []` | 64。`retry exit_codes is empty` |
| AC-066 | `exit_codes` の要素の範囲 | `[0, 75, 256]` | 64。`retry exit_codes has an invalid exit code` が 2 行(0 と 256。要素ごと)。`-1`、`1.5`、`"75"`、`true`、`75.0`、`99999999999999999999` も同じ |
| AC-067 | 成功・警告の終了コードと重なる | `{s=[0,5]}` で `exit_codes: [5,75]`、`{w=[4]}` で `exit_codes: [4]` | 64。`retry exit_codes overlap the success or warning exit codes` |
| AC-068 | 最大回数の範囲 | `max` が `0`、`101`、`-1`、`1.5`、`"3"`、`true`、`4294967296` | 64。`retry max is invalid` |
| AC-069 | 間隔の書式 | `interval` が `10`(整数)/ `1.5s` / `0s` / `25h` / `1S` / `86401s` / `1ms` / `" 1s"` / `"1 s"` / `"１s"`(全角の数字)/ `"+1s"` / `"1_0s"`(AC-543) | 64。`retry interval is invalid`(`max_interval` も同じ規則) |
| AC-070 | 倍率の範囲 | `multiplier` が `1`、`11`、`0`、`2.0`、`"2"`、`true`(`max_interval` を添える) | 64。`retry multiplier is invalid` |
| AC-071 | 倍率と上限は片方だけ不可 | `multiplier: 2` だけ、`max_interval: 1m` だけ | 64。`retry multiplier and max_interval must be given together or not at all` |
| AC-072 | 上限が間隔より小さい | `interval: 1m`、`multiplier: 2`、`max_interval: 30s` | 64。`retry max_interval is smaller than interval` |
| AC-073 | 誤りは位置の順、ほかの誤りと一緒に | `retry` の誤り 2 つと、別の定義の誤りを混ぜる | 64。ERR は位置の順に 1 行ずつ |
| AC-074 | `run` と同じ誤りの文 | 上のいずれかの誤りの定義で `run a` を実行 | `validate` と同じ文で 64 |
| AC-075 | 誤りの文に入力の値を含めない | `"interval": "SENTINEL-VALUE"` で `validate` | 64。ERR に `SENTINEL-VALUE` が現れない |

`ME`(誤りが複数ある定義。`.psd1`。行番号は、この並びのとおり。JSON の `ME` は、その後ろ)。

```powershell
@{
    format_version = 1
    unknown_top = $true

    pipeline = @{
        name = 'p'
    }

    job = @(
        @{
            id = 'a'
            command = 'x'
            success_exit_codes = @('zero')
        }
        @{
            id = 'b'
            command = 'y'
            colour = 'red'
            transitions = @(
                @{
                    on = 'success'
                    next = 'nowhere'
                }
            )
        }
    )
}
```

JSON で同じ構造を書いた `ME`(`conductor.json`。行番号は、この並びのとおり):

```json
{
  "format_version": 1,
  "unknown_top": true,
  "pipeline": {
    "name": "p"
  },
  "job": [
    {
      "id": "a",
      "command": "x",
      "success_exit_codes": ["zero"]
    },
    {
      "id": "b",
      "command": "y",
      "colour": "red",
      "transitions": [
        {
          "on": "success",
          "next": "nowhere"
        }
      ]
    }
  ]
}
```

`.psd1` の `ME` の誤りの行は、順に 3、13、18、22。JSON の `ME` の誤りの行は、順に 3、11、16、20。


### 4.2 `command` の分割と起動

「分割」の行は、`P(p; a:0{cmd="TJ --out O echo <X>"})` を作り、`run a` を実行する(`TJ`、`O` は 3.4 節・3.5 節)。`<X>` は、各行の「`command` の引数部」。
期待は、終了 0、OUT・ERR 空、`O` の記録(`arg[0]=…` 以降)。記録は `Q(…)` の形(3.4 節)。ここでは、引数の値だけを書く(`arg[i]` の添字は先頭から 0)。定義に書くときは、`.psd1` の単一引用符の文字列(`'` は `''`)で書き、文字列として同じ値になるようにする。分割は、`-split`(正規表現による分割。空白の定義が合わない)や、`[System.Management.Automation.PSParser]::Tokenize`、`CommandLineToArgvW` 相当の既存の関数に頼らず、自前で行う([05](05-execution-flow.md)、[07](07-process-and-signals.md)。これらの規則とは、バックスラッシュの扱いなどが違う)。分割の走査は、区切り・引用符の判定に必要な ASCII の 6 文字だけを比べる(サロゲートペアを分割しない。04 章)。PowerShell は `command` の文字列を解釈しない(`$HOME`・`` `x` ``・`$(x)` が、そのまま引数として届く)。


| ID | 機能 | `command` の引数部(`TJ --out O echo` の後ろ) | 期待する引数の並び |
|----|------|------------------------------------------------|---------------------|
| AC-076 | 分割: 基本 | `a b` | `a`、`b` |
| AC-077 | 分割: 空白・タブ・改行の混在 | `␣␣a ⇥ b ⏎ c␣␣` | `a`、`b`、`c` |
| AC-078 | 分割: 一重引用符 | `'a b' 'c\d' '"x"'` | `a b`、`c\d`、`"x"` |
| AC-079 | 分割: 二重引用符と `\"` | `"a \" b"` | `a " b` |
| AC-080 | 分割: 二重引用符の中の `\n` `\\` | `"a\nb" "c\\d"` | `a\nb`、`c\\d`(バックスラッシュは、そのまま。`\n` は 2 文字) |
| AC-081 | 分割: `\\"` は引用符を閉じない | `"\\"x"` | `\"x` |
| AC-082 | 分割: 引用符の外のバックスラッシュ | `a\ b` | `a\`、`b` |
| AC-083 | 分割: 末尾が `\` のパスは一重引用符 | `'C:\dir\'` | `C:\dir\` |
| AC-084 | 分割: `--msg='a b'` | `--msg='a b'` | `--msg=a b` |
| AC-085 | 分割: 引用符のつなぎ | `a"b c"'d e'f` | `ab cd ef`(1 つ) |
| AC-086 | 分割: 空の引数 | `'' "" x` | (空)、(空)、`x` |
| AC-087 | 分割: 非 ASCII | `'あ い' う` | `あ い`、`う` |
| AC-088 | 分割: メタ文字は解釈しない | `a;b && c \| d > e $HOME * `` `x` `` (y)` | `a;b`、`&&`、`c`、`\|`、`d`、`>`、`e`、`$HOME`、`*`、`` `x` ``、`(y)`。`T` に `e` というファイルができない |
| AC-089 | 分割: 区切りは 4 文字だけ | `a<U+00A0>b`(U+00A0 は空白に見えるが区切りではない。`-split '\s+'`(正規表現の空白)・`.Split()` の引数なしの形・`[char]::IsWhiteSpace` は U+00A0 でも分けるので、これらで分割しない) | `a<U+00A0>b`(1 つ) |
| AC-090 | 配列形式: 要素はそのまま | `argv=[<TJ の要素>, "--out", "O", "echo", "a b", "  lead and trail  ", "$HOME", "a;b", "*", ""]` | `a b`、`  lead and trail  `、`$HOME`、`a;b`、`*`、(空) |
| AC-091 | 文字列と配列は同じ結果 | `P(p; a:0{cmd="TJ --out O run a 0"}, b:0{cmd="TJ --out O run b 7"})` と、同じ内容の `argv` 形式 | それぞれ `run a` → 0、`pipeline p` → 7、OUT・ERR 空。`O` = `ran=a`⏎`ran=b` が同じ |

分割の誤り(拒否)。実行は `validate`。終了 64。ERR の最初の行は `conductor: error: conductor.psd1: job 'a': <文>`。

| ID | 機能 | `command`(文字列) | 期待する文 |
|----|------|---------------------|------------|
| AC-092 | 拒否: 一重引用符が閉じていない | `echo 'a` | `command has an unclosed quote` |
| AC-093 | 拒否: 二重引用符が閉じていない | `echo "a` | 同上 |
| AC-094 | 拒否: 末尾が二重引用符の中の `\` | `echo "a\` | 同上 |
| AC-095 | 拒否: `\\"` で閉じない | `echo "a\\" b` | 同上 |
| AC-096 | 拒否: パスの末尾の `\"` | `prog "C:\dir\"` | 同上 |
| AC-097 | 拒否: 空の語だけ | `''`、`'' x` | `command is empty` |
| AC-098 | 拒否: 区切りだけ | `command = '   '`(空白だけ。JSON は `"  \t\n"`)。定義の検査が、空白だけの `command` を先に拒否する | `required value is empty`(定義の検査。64) |

起動と実行環境。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-099 | 環境変数の引き継ぎ | `P(p; a:0{cmd="TJ --out O env CT_VAR CT_NO_SUCH"})`。環境に `CT_VAR=hello`、`CT_NO_SUCH` は未設定 | `run a` | 終了 0。`O` = `env[CT_VAR]=set:"hello"`⏎`env[CT_NO_SUCH]=unset` |
| AC-100 | 作業ディレクトリは引き継ぐ | `cmd="TJ --out O cwd"`。定義は `T/sub/x.psd1` | `T` で `run a --config sub/x.psd1` | `O` = `cwd=Q(T のパス)`(`T/sub` ではない) |
| AC-101 | run ID を環境変数として渡さない | `cmd="TJ --out O env CONDUCTOR_RUN_ID"` | `run a --run-id r1` | `O` = `env[CONDUCTOR_RUN_ID]=unset` |
| AC-102 | 標準出力・標準エラーはそのまま | `cmd="TJ say 'OUT to stdout' 'ERR to stderr' 0"` | `run a` | 終了 0。OUT = `OUT to stdout`(改行なし、完全一致)。ERR = `ERR to stderr`(完全一致) |
| AC-103 | 存在しないプログラム | `cmd="/no/such/program-for-conductor-test"` | `run a` | 終了 127。ERR に `conductor: error: job 'a': cannot start command '/no/such/program-for-conductor-test'`。OUT 空。ステート `S[a=error ; resume=a]`(`$process.Start()` が投げる例外(PowerShell では `MethodInvocationException` に包まれ、内側が `Win32Exception`(`NativeErrorCode` は ENOENT 相当)になる場合がある。**未確認**)を、起動失敗として扱う。例外のメッセージは表示に出さない) |
| AC-104 | 配列形式の起動失敗は先頭の要素の名前 | `argv=["no-such-prog-zzz", "--flag"]` | `run a` | 127。ERR に `cannot start command 'no-such-prog-zzz'` |
| AC-105 | 空白を含む先頭の要素 | `argv=["my prog", "x"]`(`my prog` というファイルはない) | `run a` | 127。`cannot start command 'my prog'` |
| AC-106 | 実行できないファイル(UNIX 系) | `cmd` が実行権限のない通常のファイル(`T/data.txt`)のパス | `run a` | 127(起動の例外) |
| AC-107 | ディレクトリ・`/` を指定(UNIX 系) | `cmd="/"`、`cmd` がディレクトリのパス | `run a` | 127(`Process.Start` が投げる例外の各種) |
| AC-108 | 起動失敗は表示の言語に従う | `LANG=ja_JP.UTF-8`、存在しないプログラム | `run a` | 127。ERR に `コマンドを起動できません` |
| AC-109 | 終了コードの返り方 | `exit 0`、`exit 3`、`exit 255` のジョブ(警告の指定なし) | `run a` | 0 / 3 / 255。OUT・ERR 空 |
| AC-110 | 大きな終了コード | `TJ exit 300` | `run a` | 44(`300 mod 256`。UNIX 系では、子の `exit` の値が OS により 44 になる) |
| AC-111 | 256 の終了コード(Windows) | `TJ exit 256`、`-v` | `run -v a` | 1。ERR に `exit code 256, conductor exits with 1` |
| AC-112 | 負の終了コード(Windows) | `TJ exit -1`、`-v` | `run -v a` | 255。ERR に `conductor exits with 255`(Windows で `Process.ExitCode` が `-1` で返るか `4294967295` を `int` に直した値で返るかは未確認。どちらでも、32 ビット符号つきに直してから範囲合わせをして、255 になる。AC-392) |
| AC-113 | `.cmd` への危険な引数(文字列形式。Windows) | `T/echo-args.cmd`(内容 `@echo off` と `exit /b 0`)。`cmd` = そのパス + ` 'x" & echo injected > T\injected.txt'` | `run a` | 127(禁止文字を含む引数なので、起動を拒否する。[07](07-process-and-signals.md))。`T/injected.txt` ができない |
| AC-114 | `.cmd` への危険な引数(配列形式。Windows) | 同じスクリプト。`argv=[パス, "x\" & echo injected > T\injected.txt"]` | `run a` | 同上 |
| AC-115 | `.cmd` への安全な引数(Windows) | `T/ok.cmd`(`exit /b 7`)。`cmd` = そのパス + ` plain` | `run a` | 7 |
| AC-116 | シェルを使わないことの点検 | ソース | (構文木の走査) | `System.Diagnostics.Process` と `ProcessStartInfo` の使用が `src/Conductor/Private/Runner/` にだけある。`Start-Process`・`Invoke-Expression` がなく、コマンド名が変数・式になる呼び出し(`& $name`・`. $path`)がない(`Invoke-Checks forbidden` で禁止。AC-518)。`UseShellExecute` は、リテラル `$false` だけ。`FileName` に、シェルの名前(`sh`・`bash`・`powershell`・`pwsh`・`cmd`)を渡す経路がない。`Arguments` ではなく `ArgumentList` を使う。**例外は、Windows の `.cmd` / `.bat` を起動する経路(07 章の決定: `cmd.exe` の直接起動)の、その 1 か所だけである。この経路は、禁止文字(`"`、`%`、`^`、`&`、`\|`、`<`、`>`、`!`、`(`、`)`、改行、復帰、NUL)を含む引数を拒否したあとでだけ使い、起動の組み立て(純粋な関数)とプラットフォーム層の 1 か所に集める(AC-486、AC-510、AC-513)**。区分: 点検 |

### 4.3 `run`(個別実行)

特に記さない限り、`P(…)` の `<id>:<n>` は、3.5 節のとおり `TJ exit <n>`。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-117 | 成功 | `P(p; a:0)` | `run a` | 終了 0。OUT・ERR 空。ステート: `S[a=success(0) ; completed=true]` が残る(最後のジョブで完了。完了のステートは残る) |
| AC-118 | 警告 | `P(p; warn:3{w=[3]}, fail:7, last:0)` | `run warn` | 0。OUT・ERR 空。ステート `warn=warning(3)` |
| AC-119 | 実行済みのスキップ | 上のあと | `run warn` | 0。OUT・ERR 空。ステートは変わらない |
| AC-120 | エラーはジョブの終了コード | 上のあと | `run fail` | 7。OUT・ERR 空。ステート `S[warn=warning(3), fail=error(7), last=pending ; resume=fail]` |
| AC-121 | エラーのあとの再実行 | 上のあと、定義の `fail` を `fail:0` に直す | `run fail` | 0。`fail=success(0)`。続けて `run last` → 0、`completed=true` |
| AC-122 | 自動の再実行はない | `P(p; a:7)` | `run a` | 7。ジョブは 1 回だけ実行(`TJ run` で確認) |
| AC-123 | 順序外は拒否(66) | `P(p; a:0, b:0, c:0)`、ステートなし | `run b` | 66。ERR = `conductor: error: job 'b': cannot run this job now: earlier jobs have not finished (out of order)`。OUT 空。ステートを作らない |
| AC-124 | 順序外でステートは変わらない | `S[a=success(0), b=error(1), c=pending ; resume=b]` | `run c` | 66。ステートは変わらない |
| AC-125 | 知らないジョブID | `B0` | `run nojob` | 68。ERR の最初の行 = `conductor: error: job is not defined in the definition file: 'nojob'`。続けて空行と使い方。OUT 空。ステートなし |
| AC-126 | 定義の誤りは知らないジョブIDより優先 | 定義に誤りあり | `run nojob` | 64 |
| AC-127 | ジョブIDなし | `B0` | `run` | 68。`missing argument` |
| AC-128 | 既定の定義ファイル | `T/conductor.psd1` | `run a`(`--config` なし) | 定義を読む。ステートは `T/conductor.psd1.state` |
| AC-129 | `--config` の場所 | `T/sub/x.psd1` | `run a --config sub/x.psd1` | ステートは `T/sub/x.psd1.state` |
| AC-130 | -v: 成功 | `cmd="TJ say hello ''"` | `run -v a` | 0。OUT = `hello`。ERR = `[conductor] job=a result=success (exit code 0)` |
| AC-131 | -v: 警告の遷移 | `P(p; a:1{w=[1], on=warning->c}, b:0, c:0)` | `run -v a` | 0。OUT 空。ERR = `[conductor] job=a result=warning (exit code 1)`⏎`[conductor] transition: warning -> c (skipped: b)` |
| AC-132 | -v: `@end` への遷移 | `P(p; a:0{on=success->@end}, b:0)` | `run -v a` | 0。ERR = `[conductor] job=a result=success (exit code 0)`⏎`[conductor] transition: success -> @end`。ステート `end=true`、`completed=false` |
| AC-133 | -v: 日本語 | `LANG=ja_JP.UTF-8`、`P(p; a:0, b:0)` | `run -v a`、続けて `run -v a` | ERR = `[conductor] job=a result=success (終了コード 0)`。2 回目に `[conductor] job=a スキップ (実行済み)` |
| AC-134 | -v: スキップの行 | `P(p; a:0, b:0)`、`a` を実行済み | `run -v a` | 0。ERR = `[conductor] job=a skipped (already executed)` |
| AC-135 | `@end` のあとの個別実行 | `P(p; a:0{on=success->@end}, b:0, c:0)` | `run a`、`run -v b`、`run -v c` | すべて 0。`b`・`c` は実行されない。`run b` の ERR = `[conductor] job=b skipped (skipped after @end)`。ステート: `b=skipped/byEnd`、`c=skipped/byEnd`、`end=true`。最後に `completed=true` |
| AC-136 | 遷移でスキップされたジョブの個別実行 | `P(p; a:1{w=[1], on=warning->c}, b:0, c:0)` | `run a`、`run -v b` | `run a` のあと `b=skipped/byTransition`。`run b` は 0、`b` は実行されず、ERR = `[conductor] job=b skipped (skipped by a transition)` |
| AC-137 | 成功コードに含まれない 0 | `P(p; a:0{s=[5]})` | `run a` | 1。ERR = `conductor: error: job 'a': exit code 0 is not listed in success_exit_codes`(-v なしでも出る)。ステート `a=error(0)`、`resume=a` |
| AC-138 | 成功コードに含まれない 0(-v) | 同上 | `run -v a` | 1。ERR = `[conductor] job=a result=error (exit code 0, conductor exits with 1)`⏎`conductor: error: job 'a': exit code 0 is not listed in success_exit_codes` |
| AC-139 | 成功コードの指定 | `P(p; a:5{s=[5]})`、`P(p; a:0{s=[0,2]})`、`P(p; a:2{s=[2]})` | `run a` | いずれも 0(成功) |
| AC-140 | シグナルで終了したジョブ(UNIX 系) | `argv=["sh", "-c", "kill -9 $$"]` | `run -v a` | 137。ERR = `[conductor] job=a result=error (terminated by signal 9)`。ステート `a=error`(終了コードなし)、`resume=a`。07 章 4.4 節の決定: `Process.ExitCode` が 137(129〜159 で、そのジョブの成功・警告のコードに含まれない)なので、シグナル 9 で終了したと推定する(UNIX 系でシグナル終了が `128 + 9` で返ることは、ランタイムのソースで確認。PowerShell 7.6(.NET 10)のリリース版が同じかは未確認。この項目が実機での確認を兼ねる。AC-504) |
| AC-141 | 完了のステートが残っていても先頭から | `P(p; a:0, b:0)`。`run a`、`run b`(完了)のあと | `run a`(`TJ run` で記録) | 0。`a` が再び実行される。`completed` のステートは上書きされる |
| AC-142 | 完了のあと先頭以外は拒否 | 上の完了のステート | `run b` | 66 |
| AC-143 | `--remove-on-complete`(run) | `P(p; a:0)` | `run a --remove-on-complete` | 0。OUT・ERR 空。`T/conductor.psd1.state` がない |
| AC-144 | 定義の `remove_on_complete` | `P(p remove_on_complete=true; a:0)` | `run a` | 0。ステートがない |
| AC-145 | 完了でなければ消さない | `P(p; a:0, b:0)` | `run a --remove-on-complete` | 0。ステートが残る(`a=success`、`completed=false`) |
| AC-146 | ステートの形(エラーで停止) | 定義 `c2.psd1`(`P(n2; a:0, b:3, c:0)`) | `pipeline n2 --config c2.psd1` | 3。`T/c2.psd1.state` の内容は JSON のオブジェクトで、`completed=false`、`definition_key="c2.psd1"`、`definition_path="c2.psd1"`、`end_reached=false`、`format_version=1`、`job_order=["a","b","c"]`、`pipeline_name="n2"`、`resume_job_id="b"`、`job` は `a=success exit_code=0`、`b=error exit_code=3`、`c=pending`(`run_id` キーなし) |
| AC-147 | ステートの書式 | 上のステート | (ファイルを読む) | 1 つの JSON のオブジェクト(テストが読める)。末尾は改行。インデントと、キーの並び(固定の順)は [06](06-state-file.md) のとおり。同じ内容のステートを書くと、毎回同じバイト列になる(ハッシュテーブルの列挙順に頼らず(`[ordered]` か配列で)、キーを出力する順を、書き出しの部品の中で固定する) |
| AC-148 | ステートに機微な値を書かない | `cmd="TJ say SECRET-OUT '' 3"`(終了コード 3 のエラーになる)。環境変数 `SECRET_ENV=SECRET-ENVVAL` を設定 | `run a` | ステートファイル(終了コード 3、`resume=a`)に、`command` の文字列、`SECRET-OUT`、`SECRET-ENVVAL` が現れない |
| AC-149 | 初期ステートの書き込みに失敗(UNIX 系、非 root) | `T` を書き込み不可にする(`chmod 555 T`。テストから `chmod` コマンドを起動する。定義は読める。テストの後始末で権限を戻す) | `run a` | 65。ERR に `cannot write state file` と `(the job was not started)`。ジョブは実行されない |
| AC-150 | ジョブのあとの書き込みの失敗(UNIX 系、非 root) | 定義 `argv=["sh", "-c", "chmod a-w . ; exit 7"]`(ジョブが `T` を書き込み不可にする) | `run a` | 65(ジョブの 7 ではない)。ERR に `cannot write state file` と `(the job has already been executed)` |
| AC-151 | 定義の誤り(ファイルなし・誤り) | 定義なし / `{"format_version": 1}` だけ | `run a` | 64。ERR は `conductor: error: ` で始まる。OUT 空 |
### 4.4 `pipeline`

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-152 | 全部成功: 記述順に実行 | `P(p; a:0{cmd="TJ --out O run a 0"}, b:0{cmd="TJ --out O run b 0"})`(2 ジョブ) | `pipeline p` | 0。OUT・ERR 空。`O` = `ran=a`⏎`ran=b`。ステート `S[a=success(0), b=success(0) ; completed=true]` が残る |
| AC-153 | 完了のステートの形 | 上のあと | (ステートを読む) | `resume_job_id` なし、`completed=true`、`end_reached=false`。形式は変わらない |
| AC-154 | 再実行は先頭から | 上のあと | `pipeline p` | 0。`a`、`b` が再び実行される(`O` に 2 組) |
| AC-155 | 警告の遷移で間をスキップ | `P(nightly; extract:1{w=[1], on=warning->load}, transform:0, load:0)` | `pipeline -v nightly --run-id r1` | 0。ERR = `[conductor] job=extract result=warning (exit code 1)`⏎`[conductor] transition: warning -> load (skipped: transform)`⏎`[conductor] job=transform skipped (skipped by a transition)`⏎`[conductor] job=load result=success (exit code 0)`。`transform` は実行されない |
| AC-156 | `@end` で完了 | `P(p; a:0{on=success->@end}, b:0, c:0)` | `pipeline p` | 0。`b`・`c` は実行されない。ステート `S[a=success(0), b=skipped/byEnd, c=skipped/byEnd ; end=true, completed=true]` |
| AC-157 | エラーで停止 | `P(p; a:0, b:7, c:0)`(各ジョブは `TJ run` 形式で記録) | `pipeline p` | 7。`c` は実行されない。ステート `S[a=success(0), b=error(7), c=pending ; resume=b]` |
| AC-158 | エラーからの再開 | 上のあと、定義の `b` を `b:0` に直す | `pipeline -v p` | 0。`a` は再実行されない(ERR に `[conductor] job=a skipped (already executed)`)。`b`・`c` が実行される。完了のステート |
| AC-159 | 個別実行の続きから一括実行 | `P(p; a:0, b:0, c:0)`。`run a` のあと | `pipeline p` | 0。`a` は再実行されない。`b`・`c` が実行される |
| AC-160 | 起動できないジョブで停止 | `P(p; a:0, b:0, c:0)` の `b` を存在しないプログラムに | `pipeline p` | 127。`c` は実行されない。ERR に `cannot start command` |
| AC-161 | パイプライン名の不一致 | `B0` | `pipeline wrongname` | 64。`pipeline name does not match the definition`。ジョブは実行されない |
| AC-162 | パイプライン名なし | `B0` | `pipeline` | 68。`missing argument` |
| AC-163 | `--remove-on-complete` | `P(p; a:0, b:0)` | `pipeline p --remove-on-complete` | 0。OUT・ERR 空。ステートがない。続く `watch` → `no state (nothing has run yet)` |
| AC-164 | 定義の `remove_on_complete: true` | `P(p remove_on_complete=true; a:0, b:0)` | `pipeline p` | 0。ステートがない |
| AC-165 | オプションと定義の組み合わせ | `remove_on_complete=false` の定義 | `pipeline p --remove-on-complete` | ステートがない(オプションが優先して消す) |
| AC-166 | 完了でないとき消さない | `P(p remove_on_complete=true; a:0, b:7)` | `pipeline p` | 7。ステートが残る(エラーで停止) |
| AC-167 | 完了のあと先頭以外の run は 66 | 完了のステート | `run b` | 66 |
| AC-168 | 初期ステートが書けない(UNIX 系、非 root) | `T` を書き込み不可 | `pipeline p` | 65。ジョブは起動されない |
| AC-169 | 完了のステートが残るとき定義の変更は照合されない | `pipeline p` で完了 → 定義のジョブIDを変える | `pipeline p` | 0(完了のステートは「空」として扱う。65 にならない) |
### 4.5 `watch`

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-170 | 表の形(英語) | `P(p; a:0, b:7, c:0)` で `pipeline p`(7 で終わる)のあと | `watch` | 終了 0。OUT = `Pipeline: p`⏎`JOB  STATUS   NOTE`⏎`a    success  exit code 0`⏎`b    error    exit code 7`⏎`c    pending`(行末の空白なし)。ERR 空 |
| AC-171 | ステートなし | ステートなし | `watch` | 0。OUT = `no state (nothing has run yet)`(1 行) |
| AC-172 | ステートなし(日本語) | `LANG=ja_JP.UTF-8` | `watch` | 0。OUT = `ステートはありません(まだ実行されていません)` |
| AC-173 | 長さ 0 のファイルも「なし」 | 長さ 0 のステートファイル | `watch` | 同上(`no state …`) |
| AC-174 | 完了の行 | `P(nightly; extract:1{w=[1], on=warning->load}, transform:0, load:0)`、`pipeline nightly --run-id r1` のあと | `watch --run-id r1` | OUT = `Pipeline: nightly`⏎`Run ID: r1`⏎`JOB        STATUS   NOTE`⏎`extract    warning  exit code 1`⏎`transform  skipped  skipped by a transition`⏎`load       success  exit code 0`⏎`completed` |
| AC-175 | 日本語の表 | 同上、`LANG=ja_JP.UTF-8` | `watch --run-id r1` | OUT = `パイプライン: nightly`⏎`Run ID: r1`⏎`ジョブ     状態     補足`⏎`extract    warning  終了コード 1`⏎`transform  skipped  遷移によりスキップ`⏎`load       success  終了コード 0`⏎`完了`(状態の語は英語のまま、`Run ID` は日本語でも `Run ID`) |
| AC-176 | `@end` のスキップの補足 | `@end` で完了したステート(4.4 節) | `watch` | `b`、`c` の行の NOTE = `skipped after @end` |
| AC-177 | 定義ファイルを読まない | 定義ファイルがない、または壊れている。ステートあり | `watch --config missing.psd1` | 0。ステートが表示される(定義がなくても動く) |
| AC-178 | ステートなし・定義なしでも 0 | 定義ファイルなし | `watch --config missing.psd1` | 0。OUT に `no state` |
| AC-179 | 壊れたステート | `{garbage` | `watch` | 65。ERR = ``conductor: error: conductor.psd1.state: state file is corrupt: JSON として読めません; run `conductor reset` to discard the state``(`` ` `` はバッククォート)。OUT 空 |
| AC-180 | 壊れた完了のステート | 完了のステートだが `run_id` がファイル名と食い違う | `watch` | 65 |
| AC-181 | ファイルへの出力に制御文字がない | `P(p; a:0, b:0)`、`run a` のあと | `watch`(標準出力をファイルに) | 0。ファイルに `\x1b` がなく、`a`、`success`、`pending` を含む |
| AC-182 | 端末では状態に色(UNIX 系。疑似端末) | 同上 | 標準出力を疑似端末(テストの補助 `PtyHelper` が、P/Invoke の `openpty` で作った子側。`[Console]::IsOutputRedirected` が `false`)にして `watch` | `success` が `\x1b[32m` と `\x1b[0m` で囲まれる(`warning` 33、`error` 31、`skipped` 36、`pending` は色なし)。列の幅は色の制御文字を除いた文字で決まる。区分: UNIX 系(疑似端末)。`openpty` の P/Invoke(ライブラリ名は Linux の glibc の版で `libc` か `libutil`)は、この文書の作成時には確かめていない(未確認。10 章)。併せて、端末かどうかの判定を `Deps` の `Env` にして、偽の `Deps` で「端末」を返す単体の試験を置く(区分: 注入) |
| AC-183 | `NO_COLOR` で色なし | 同上、`NO_COLOR=1` | 同上 | 制御文字なし。空の `NO_COLOR` では色あり。区分: UNIX 系(疑似端末か、注入) |
| AC-184 | Windows は色なし | Windows | `watch`(端末) | 制御文字なし。区分: Windows |
| AC-185 | ジョブIDの制御文字は表の中でもエスケープ | ジョブID に ESC を含む(JSON の定義で `"a\u001bb"`。`.psd1` では ESC の文字そのものを単一引用符の中に入れる)。実行済み | `watch` | 行に `a\u{1b}b`。生の ESC がない |
| AC-186 | 標準出力に書けない(注入) | ステートあり | `watch`(標準出力の `BufSink` を書き込み失敗にする) | 65。ERR = `conductor: error: cannot write to standard output`。Linux では `watch > /dev/full` でも起こせる(AC-495)。パイプの読み手が閉じた場合(`IOException`)は AC-493、AC-516 |
| AC-187 | 使えないオプション | なし | `watch -v` / `watch --remove-on-complete` / `watch --all` / `watch extra` | 68。ERR = `option not allowed for this command: '-v'`(それぞれ対象は `-v`、`--remove-on-complete`、`--all`)。`watch extra` は `unexpected argument: 'extra'` |
### 4.6 `reset`

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-188 | ステートを消す | `P(p; a:0, b:7)`、`pipeline p` のあと | `reset` | 0。OUT・ERR 空。ステートがない。続く `watch` → `no state …` |
| AC-189 | 先頭から実行できる | 上のあと、`P(p; a:0, b:0)` に直す | `run a`(`TJ run` で記録) | 0。`a` が実行される |
| AC-190 | ステートがなくても成功 | ステートなし | `reset` | 0。OUT・ERR 空 |
| AC-191 | 定義がなくても動く | 定義なし | `reset --config missing.psd1` | 0 |
| AC-192 | 壊れたステート・壊れた定義を迂回 | 壊れたステートと壊れた定義 | `reset` | 0。ステートが消える |
| AC-193 | 完了のステートも消す | 完了のステート(run ID あり・なし) | `reset`、`reset --run-id r1` | 0。消える |
| AC-194 | `--run-id` は 1 つだけ消す | `conductor.psd1.state`、`.r1.state`、`.r2.state` がある | `reset --run-id r1` | `.r1.state` だけ消える。ほかは変わらない |
| AC-195 | `--all` で全部消す | `P(p; a:0, b:7)`。`pipeline p`(run ID なし)、`--run-id a`、`--run-id b` で実行。`T` に、別の定義のステート(`conductor.psd1.a.b.state`。中身はパイプライン名 `p`・ジョブ `a` の正常なステートで、`run_id` を持たない)、壊れたもの(`conductor.psd1.c.state` = `{garbage`)、一時ファイル(`.conductor.psd1.state.tmp.1.2.3`)、無関係なファイル(`conductor.psd1.bak`、`other.psd1.state`)を置く | `reset --all` | 終了 0。OUT 空。ERR は `conductor: warning: ` で始まり、`conductor.psd1.c.state` を含み、`conductor.psd1.a.b.state` を含まない。残るファイル: `.conductor.psd1.state.tmp.1.2.3`、`conductor.psd1`、`conductor.psd1.a.b.state`、`conductor.psd1.bak`、`conductor.psd1.c.state`、`other.psd1.state`。run ID なしと `.a.state`、`.b.state` は消える |
| AC-196 | 警告の文言 | 上と同じ | `reset --all` | ERR = ``conductor: warning: kept state files that could not be read (remove one with `conductor reset --run-id <ID>`):``⏎`  conductor.psd1.c.state`(ファイル名の行は先頭に空白 2 文字) |
| AC-197 | 壊れたものを個別に消す | 上のあと | `reset --run-id c` | 0。`conductor.psd1.c.state` が消える |
| AC-198 | 何もなくても 0 で出力なし | `T` に定義だけ | `reset --all` | 0。OUT・ERR 空 |
| AC-199 | `--all` と `--run-id` の併用 | なし | `reset --all --run-id x` | 68。`invalid value: '--all'` |
| AC-200 | `--all` は環境変数を見ない | `CONDUCTOR_RUN_ID='bad id'` | `reset --all` | 0 |
| AC-201 | シンボリックリンクと別の定義は消さない(UNIX 系) | `conductor.psd1.s.state` が通常のファイルへのシンボリックリンク。ディレクトリ `conductor.psd1.d.state`。`conductor.psd1.a.state` の中身の `run_id` がファイル名と食い違う | `reset --all` | リンク先のファイルは残る。リンクとディレクトリと、中身の `run_id` が食い違うファイルは消されない(`[System.IO.FileSystemInfo]` の `Attributes` の `ReparsePoint` か `LinkTarget` で種類を調べ、リンクをたどらない。`Get-ChildItem` の既定(リンクをたどる場合がある)に頼らない。挙動の細部は未確認。実装時に確認) |
| AC-202 | run ID なしのステートは中身を読まず消す | `conductor.psd1.state` が壊れている(通常のファイル) | `reset --all` | 消える。警告なし |
| AC-203 | 環境変数の値の違反 | `CONDUCTOR_RUN_ID='bad id'` | `reset` | 68。`invalid value: 'run ID (environment variable)'` |
| AC-204 | 引数の誤り | なし | `reset extra`、`reset -v`、`reset --remove-on-complete` | 68。`unexpected argument: 'extra'`、`option not allowed for this command: '-v'`、同 `'--remove-on-complete'` |
| AC-205 | 削除の失敗(UNIX 系、非 root) | `T/conductor.psd1.state` が空でないディレクトリ | `reset` | 65。ERR に `cannot remove state file`。OUT 空 |
### 4.7 `convert`(`.psd1` と JSON の相互変換)

`convert <入力ファイル> --to psd1|json`。入力の形式は `--to` の反対(`--to psd1` なら JSON、`--to json` なら `.psd1`)で、入力ファイルの拡張子は見ない([09](09-converter.md))。`.psd1` の入力は、**コードを実行せず**、構文木を検査して値にする([03](03-definition-format.md))。

変換の入力 `CV1P`(`.psd1`)、`CV1`(`CV1P` を JSON にしたもの)、`CV2P`(`.psd1`。コメント、引用符の書き方、区切りの書き方、here-string を含む)は、次のとおり。

```powershell
# CV1P
@{
    format_version = 1
    pipeline = @{
        name = 'nightly'
        description = '夜間バッチ(抽出、変換、読み込み、通知)'
    }
    job = @(
        @{ id = 'extract'; command = 'extract --all --since yesterday' }
        @{
            id = 'transform'
            command = 'transform "in dir" $HOME'
            warning_exit_codes = @(3, 4)
            success_exit_codes = @(0, 2)
            transitions = @( @{ on = 'warning'; next = 'notify' } )
        }
        @{ id = 'load'; command = 'load'; transitions = @( @{ on = 'success'; next = '@end' } ) }
        @{ id = 'notify'; command = 'notify --channel ops' }
    )
}
```

```json
{"format_version": 1,
 "pipeline": {"name": "nightly", "description": "夜間バッチ(抽出、変換、読み込み、通知)"},
 "job": [
  {"id": "extract", "command": "extract --all --since yesterday"},
  {"id": "transform", "command": "transform \"in dir\" $HOME",
   "warning_exit_codes": [3, 4], "success_exit_codes": [0, 2],
   "transitions": [{"on": "warning", "next": "notify"}]},
  {"id": "load", "command": "load", "transitions": [{"on": "success", "next": "@end"}]},
  {"id": "notify", "command": "notify --channel ops"}]}
```

```powershell
# CV2P(コメントは変換で失われる)
<# ブロックコメント
   2 行目 #>
@{
    'format_version' = 1                                                  # キーを引用符で
    "pipeline" = @{ name = "nightly"; description = '夜間バッチ' }       # `;` 区切りと、二重引用符
    job = @(
        @{
            id = 'extract'
            command = @'
extract --all
'@
            success_exit_codes = 0, 2                                     # `@()` なしのカンマ区切り
        },
        @{ id = 'notify'; command = 'notify' }
    )
}
```

特に記さない限り、失敗の期待は、終了 67、OUT 空、ERR の最初の行が `conductor: error: ` で始まる。「木」は、テストが `.psd1` を**テスト専用の独立した小さな読み取り**(`tests/support/Tree.ps1`。製品の `Json`・`Definition` とは別の、字句の走査。比較用に `Import-PowerShellDataFile` を使ってよい)で読んで得る、ハッシュテーブル・配列・文字列・整数・浮動小数点数・真偽値・`$null` の構造と、JSON を同様に読んで得る構造。**木の比較は、型も比べる**(整数 `[long]` と浮動小数点数 `[double]` と真偽値を、値が等しくても同じとしない。`1 -eq 1.0` や `1 -eq $true` が真になる PowerShell の比較演算子に頼らず、`-is` と `-ceq` を使う)。`ConvertFrom-Json` は、重複キー・巨大整数・日時文字列の自動変換などがあるため、木の比較には使わない。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-206 | `.psd1` から JSON | `T/in.psd1` = `CV1P` | `convert in.psd1 --to json` | 終了 0。ERR 空。OUT は JSON で、`"format_version": 1` と `"id": "extract"` を含む。OUT を JSON として読んだ木が `CV1` の木と等しい。キーの順は入力の記述順、インデントは 2 スペース、末尾は改行(細部は [09](09-converter.md) のとおり)。`$HOME`(単一引用符の中)は文字列 `$HOME` のまま |
| AC-207 | コメント・引用符・区切りの書き方 | `T/in.psd1` = `CV2P` | `convert in.psd1 --to json` | 0。OUT に `#` も `<#` もない。木は、`format_version` = 1、`pipeline` = {`name` = `nightly`、`description` = `夜間バッチ`}、`job[0]` = {`id` = `extract`、`command` = `extract --all`(here-string の末尾の改行は含まない)、`success_exit_codes` = [0, 2]}、`job[1]` = {`id` = `notify`、`command` = `notify`} |
| AC-208 | 変換の出力は標準出力だけ | `T/in.psd1` = `@{ format_version = 1; pipeline = @{ name = 'p' }; job = @( @{ id = 'a'; command = 'echo a' } ) }` | `convert in.psd1 --to json` | 0。ERR 空。OUT に `"job": [` と `"format_version": 1` |
| AC-209 | 入力形式は `--to` の反対で、拡張子は見ない | `T/looks-like.json` = `.psd1` の内容(`@{ a = 1; b = @('x', 'y') }`) | `convert looks-like.json --to json` / `convert looks-like.json --to psd1` | 前者: 0、OUT を JSON として読むと `{"a": 1, "b": ["x","y"]}`。後者: 67(JSON として読んで `syntax error`) |
| AC-210 | 文法の誤りは種類と行つき | `T/broken.psd1` = `@{`⏎`a = @(1, 2`⏎ | `convert broken.psd1 --to json` | 67。OUT 空。ERR の最初の行が `conductor: error: broken.psd1:` で始まり、種類の文が `syntax error`、理由が `.psd1` の文法の誤りの固定の文(03 章・09 章に従う)。ERR に、パーサーのエラーの文(`ParseError.Message`)の断片が現れない。行は、PowerShell のパーサーが返す位置(`ParseError.Extent.StartLineNumber`)。**どの行を返すか(閉じない `@(` の行か、ファイルの末尾か)は未確認** |
| AC-211 | 入力がない | なし | `convert nofile --to json` | 67。ERR = `conductor: error: nofile: input file does not exist: 指定の場所にファイルがありません` |
| AC-212 | ディレクトリを入力に | `T/d` がディレクトリ | `convert d --to json` | 67。`cannot read input file` |
| AC-213 | 入力が大きすぎる | 1,048,577 バイト(中身が壊れていてもよい) | `convert big.psd1 --to json` | 67。`input is too large`(読む前に `FileInfo.Length` で見る) |
| AC-214 | ちょうど 1 MiB は受理 | `@{ a = 'xxx…' }` で、ちょうど 1,048,576 バイト(`Slow`) | `convert max.psd1 --to json` | 0。OUT が `{⏎  "a": "xxx` で始まる |
| AC-215 | UTF-8 でない入力 | `a = '` のあとに不正なバイト列を含む `.psd1` | `convert bad.psd1 --to json` | 67。`cannot read input file`(文法を見る前) |
| AC-216 | 表せない値は場所つき | `T/in.psd1` = `@{ job = @( @{ id = 'x' }, @{ id = 'y'; note = 99999999999999999999 } ) }` | `convert in.psd1 --to json` | 67。ERR に `value cannot be represented in the target format: 数値が表せる範囲を超えています` と `(at: job[1].note)`。整数の字句は、構文木の定数の値(`[decimal]` や `[double]` に化ける)でなく、**元の字句(`Extent.Text`)**から判定する |
| AC-217 | 表せない値と、表せる値の境界 | `.psd1`(`--to json`): `a = 9223372036854775808` / `a = 0xFFFFFFFFFFFFFFFFFF` / `a = 1e999` / `a = @(1, 99999999999999999999)`。JSON(`--to psd1`): `{"a": 9223372036854775808}` / `{"a": 1e400}` / `{"a": 1, "A": 2}`。受理されるもの: `.psd1` の `a = $null`、JSON の `{"a": null}`、`{"a": 9223372036854775807}`、非常に長いキー(1,025 文字。変換にキーの長さの上限はない。09 章 8.3 節)、`.psd1` の `a = 9223372036854775807` と `a = -9223372036854775808` | `convert … --to <変換後>` | 表せないものは 67(種類 `unrepresentable`)。原因の文は、順に `数値が表せる範囲を超えています`(`.psd1` の 4 つと JSON の 2 つ。`1e999` が、PowerShell のパーサーで数値として読めず文法の誤りになる場合は `syntax error`。**どちらになるかは未確認。実装時に確認して、この行を直す**)、`.psd1 では、大文字小文字だけが違うキーを同時に書けません`(`{"a": 1, "A": 2}`。場所は `A`。`.psd1` のハッシュテーブルは、キーの大文字小文字を区別せず重複とみなす(AC-456)ので表せない。09 章 5 節が正本)。受理されるものは 0 で、`$null` は `null` のまま出る。64 ビット符号つき整数の端は、桁を落とさずに扱う(`[double]` に直さない。字句を `[bigint]` で読んで、`[long]::MinValue <= n <= [long]::MaxValue` の比較をしてから `[long]` にする) |
| AC-218 | 許されない構文・文字列でないキー・複数の式 | `.psd1`: `a = $x` / `a = Get-Date` / `a = $(1)` / `a = 1 + 1` / `a = [int]'1'` / `a = { 1 }` / `@{ 1 = 'x' }` / `@{ $true = 'x' }` / `@{ a = 1 }; @{ b = 2 }` / `a = 1`(`@{}` なし) | `convert in.psd1 --to json` | 67。種類の文は `syntax error`。理由は 03 章の固定の文(`.psd1 で使えない書き方です` / `キーは文字列で書きます`。複数の式も `.psd1 で使えない書き方です`)に従う。**どの入力も評価されない**(`T` にファイルができない。AC-454) |
| AC-219 | 最上位がハッシュテーブル・オブジェクトでない | `.psd1` `@(1, 2)` / `1` / `'x'` / `$null`(`--to json`)、JSON `[1, 2]`(`--to psd1`) | `convert` | 67。`top level is not a map: 最上位がマップではありません`(行 1) |
| AC-220 | 空の入力 | 空の `.psd1`(または `# c` だけ)を `--to json` で。空の JSON ファイル(0 バイト)を `--to psd1` で | `convert` | `.psd1`: 67。`syntax error: .psd1 の文法に誤りがあります`(空の入力は `rootNotMap` でなく `syntax`。09 章 4.3 節)。JSON: 67。`syntax error: JSON の文法に誤りがあります` |
| AC-221 | コメントは捨てられる | `# 先頭⏎@{ a = 1 # 末尾⏎}`、`<# ブロック #>` | `convert in.psd1 --to json` | 0。OUT = `{⏎  "a": 1⏎}`。OUT に `#` がない |
| AC-222 | キーの順序は入力の記述順 | `@{ b = 1; a = 2 }` | `convert in.psd1 --to json` | OUT = `{⏎  "b": 1,⏎  "a": 2⏎}`(辞書順にも、PowerShell の `@{}`(順序なしのハッシュテーブル)の列挙順にもしない。`Sort-Object`・`Group-Object`・`[hashtable]` の列挙に頼らず、順序つきの構造(`[System.Collections.Generic.List[object]]` に、キーと値の組を入れる)で持つ)。整数に見えるキー(`'1'`、`'2'`)でも順序は変わらない(`@{ b = 1; '1' = 2 }` で、順序が保たれることも確かめる。AC-511) |
| AC-223 | 空のハッシュテーブル | `@{}`(`.psd1`)、`{}`(JSON) | `convert in.psd1 --to json` / `convert in.json --to psd1` | 0。JSON の出力は `{}`。`.psd1` の出力は `@{}` |
| AC-224 | 配列の書き方の違い | `@{ a = @(1); b = @(); c = 1, 2; d = @(@(1)); e = @(@{ x = 1 }) }` | `convert in.psd1 --to json` | 0。`a` は `[1]`(1 要素の配列が、配列のまま)、`b` は `[]`、`c` は `[1, 2]`、`d` は `[[1]]`、`e` は `[{"x": 1}]`。1 要素の配列が整数やオブジェクトに化けない(AC-554) |
| AC-225 | 単項のカンマと、配列でない値 | `@{ a = ,1 }` / `@{ a = (1) }` / `@{ a = @(1) + @(2) }` | `convert in.psd1 --to json` | 67。`syntax error`(単項のカンマ・括弧・演算子は、許す構文の一覧にない。03 章) |
| AC-226 | 重複するキー | `.psd1` `a = 1`⏎`b = 2`⏎`a = 3`(1 つのハッシュテーブルの中。`--to json`)。大文字小文字だけ違う `a = 1; A = 2`。JSON `{"a": 1, "b": 2, "a": 3}`(`--to psd1`) | `convert` | 67。`syntax error: キーが重複しています`(`.psd1` の最初の例は行 3。2 番目も重複。JSON の `a` と `A` は別のキーで、重複でない。AC-217 の `unrepresentable` になる) |
| AC-227 | here-string | `@{ a = @'`⏎`line1`⏎`line2`⏎`'@ }` の形(改行を含む複数行)、二重引用符の here-string `@"`…`"@`(`$` を含まないもの / 含むもの) | `convert in.psd1 --to json` | `$` を含まないものは 0 で、`"a": "line1\nline2"`(here-string の最初の改行と、最後の `'@` の直前の改行は含まない)。`$` を含む二重引用符の here-string は 67(`syntax error`。変数の展開を含む文字列) |
| AC-228 | スカラーの扱い(PowerShell のリテラル。03 章) | `.psd1`: `a = 1`、`b = 0x1F`、`c = 0b101`、`d = '1'`、`e = '2001-01-01'`、`f = $true`、`g = $false`、`h = $null`、`i = 1.5`、`j = -5`、`k = 1.0`、`l = 'null'`、`m = 'yes'` | `convert in.psd1 --to json` | `"a": 1`、`"b": 31`、`"c": 5`、`"d": "1"`、`"e": "2001-01-01"`(日時に化けない)、`"f": true`、`"g": false`、`"h": null`、`"i": 1.5`、`"j": -5`、`"k": 1.0`、`"l": "null"`、`"m": "yes"`(キーは入力の記述順に並ぶ)。**整数・小数の字句の受理範囲は、03 章 4.3.1 節が正本**: 10 進の整数(先頭に 0 を並べない。`-` つきを含む)、`0x` の 16 進、`0b` の 2 進、小数点または指数をもつ浮動小数(`1.5`、`1e3`)。型の接尾辞(`1L`・`1d`・`1u`)、乗数(`1kb`)、数字の区切り(`1_000`)、先頭の `+`、先頭に 0 を並べた整数(`007`)は、67(`syntax error`。理由 `.psd1 で使えない書き方です`) |
| AC-229 | 展開後の要素数の上限 | `a = 0,0,…`(0 を n 個)。JSON は `{"a": [0, 0, …]}`(`Slow`) | `convert in.psd1 --to json` / `convert in.json --to psd1` | n = 99,997 は 0。n = 99,998 は 67(種類の文 `input is too large`。理由 `要素数が上限(100000)を超えています`)。JSON も同じ境界(要素の数え方は [03](03-definition-format.md)。キーも数える) |
| AC-230 | 文字の大きさの上限 | 変換の本体の関数(`Convert/`)を直接呼ぶ。メモリ上の木(1 MiB のファイルからは到達できない大きさ。16 MiB + 1 文字の文字列を持つ) | (単体) | 67 相当の失敗(種類の文 `input is too large`。理由 `文字の大きさが上限(16 MiB)を超えています`)。メモリを使い果たさない(出力を作る前に大きさを数える)。区分: 単体 |
| AC-231 | 入れ子の深さの上限 | `.psd1`: `a = ` に続けて `@(` を 32 個 / 33 個(閉じも書く)。JSON: `{"a": ` に続けて `[` を 32 個 / 33 個(閉じも書く) | `convert` | 32 個: 0。33 個: 67(`syntax error: 入れ子が深すぎます(上限は 32 段)`)。最上位のハッシュテーブル(根)は数えず、その直下のコレクションを 1 段と数える([03](03-definition-format.md)) |
| AC-232 | 極端な深さでも異常終了しない | 1,000 段、100,000 段の入れ子(`.psd1`・JSON。`Slow`) | `convert … ` | 67(`syntax error`)。異常終了しない(スタックの溢れでプロセスが落ちない。2.2 節。PowerShell のパーサー自体が深い入れ子で落ちないか、構文木を作る前に字句で深さを数えて拒否するかは**未確認**。後者が安全) |
| AC-233 | 往復で木が等しい | `CV1P` | `convert in.psd1 --to json` の OUT を `convert --to psd1` に通す | 結果の木が `CV1P` の木と等しい。`command` が配列形式の定義、`retry` のハッシュテーブルも同じ。`CV1` を先に `--to psd1`、次に `--to json` に通しても等しい |
| AC-234 | 数値の往復 | 整数の端(64 ビット符号つき整数の最小・最大)、浮動小数(`1.0`、`-0.0`、`0.5`)、空の配列・ハッシュテーブル | 往復(`.psd1` → JSON → `.psd1`、JSON → `.psd1` → JSON) | 型が変わらない(`1` と `1.0` を区別する。`0.0` と `-0.0` は別。JSON の出力で、浮動小数は、小数点か指数を持つ書き方になる。`[double]` の書式は 09 章に従う。AC-544)。整数の端は、桁を落とさない。`-0.0` の `.psd1` の字句(`-0.0`)が、PowerShell のパーサーで負のゼロを保つかは**未確認** |
| AC-235 | ファイルを作らず環境に左右されない | `T` に入力のみ | `convert` を、別のカレントディレクトリ・異なる環境変数(`LANG`、`PSModulePath`、`POWERSHELL_*`)で実行 | `T` のファイル一覧が実行の前後で同じ。結果が同じ |
| AC-236 | 標準出力に書けない(注入) | 正しい入力 | `convert`(標準出力の `BufSink` を書き込み失敗にする) | 67。ERR = `conductor: error: cannot write to standard output` |
| AC-237 | 引数の誤り | なし | `convert f`(`--to` なし)/ `convert f --to xml` / `convert f --to PSD1` / `convert f --to Json` / `convert f --to csv` / `convert f --to psd1 --config c` | 68。`missing argument: '--to'` / `invalid value: 'xml'` / `invalid value: 'PSD1'` / `invalid value: 'Json'` / `invalid value: 'csv'` / `option not allowed for this command: '--config'` |
| AC-238 | 変換結果の全体を 1 回で書く | 失敗する入力 | `convert` | 失敗のとき、標準出力に 1 バイトも書かれない |

---


### 4.8 run ID

特に記さない限り、定義は `two_jobs(n)` = `P(p; a:0, b:<n>)`(`a` は成功、`b` は終了コード `n`)。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-239 | run ID ごとに別のステート | `two_jobs(7)` | `pipeline p --run-id r1` と `pipeline p --run-id r2` を同時に | どちらも 7。OUT 空。`T` のファイルは `conductor.psd1`、`conductor.psd1.r1.state`、`conductor.psd1.r2.state` の 3 つだけ。各ステートは `run_id` がその ID、`resume=b`、`a=success` |
| AC-240 | 同じ run ID で再開し、他は変わらない | 上のあと、定義を `two_jobs(0)` に直す | `pipeline p --run-id r1` | 0。`r1` は完了のステートが残る(`completed=true`)。`r2` のステートファイルはバイト列が変わらない |
| AC-241 | ステートの形(run ID あり・なし) | `two_jobs(7)` | `pipeline p --run-id r1`、`pipeline p` | `.r1.state` は `"run_id": "r1"` を持つ。run ID なしの `.state` は `run_id` キーを持たない |
| AC-242 | 環境変数 `CONDUCTOR_RUN_ID` | `two_jobs(7)`、`CONDUCTOR_RUN_ID=fromenv` | `pipeline p` | 7。`conductor.psd1.fromenv.state` ができる |
| AC-243 | 空のオプションは run ID なしに固定 | 同じ環境変数 | `pipeline p --run-id ""` | 7。`conductor.psd1.state`(run ID なし)ができる |
| AC-244 | オプションは環境変数に優先 | 同じ環境変数 | `pipeline p --run-id fromopt` | `conductor.psd1.fromopt.state` ができる(`fromenv` は使われない) |
| AC-245 | 環境変数が空なら run ID なし | `CONDUCTOR_RUN_ID=` | `pipeline p` | 7。`conductor.psd1.state` |
| AC-246 | 定義の `run_id_env` | `P(p run_id_env=MY_RUN; a:7)`、`MY_RUN=m1`、`CONDUCTOR_RUN_ID=ignored` | `run a` | 7。`conductor.psd1.m1.state` ができる(`ignored` は使われない) |
| AC-247 | `watch`・`reset` も `run_id_env` を読む | 上のあと、`MY_RUN=m1` | `watch`、`reset` | `watch` の OUT に `Run ID: m1`。`reset` で `conductor.psd1.m1.state` が消える |
| AC-248 | 有効な run ID | `two_jobs(0)` | `pipeline p --run-id <値>`(`1`、`r1`、`12345`、`build-2026.11_a`、`a.b`、`x` を 64 個) | いずれも 0 |
| AC-249 | 無効な run ID(オプション) | `two_jobs(0)` | `pipeline p --run-id <値>`(`a/b`、`..`、`x` を 65 個、`a b`、`.x`、`日本語`、`r1<LF>`(末尾が改行。AC-542)) | 68。ERR の最初の行 = `conductor: error: invalid value: '--run-id'`。ERR にその値が現れない。ステートが作られない |
| AC-250 | 無効な run ID(環境変数) | 同じ値を `CONDUCTOR_RUN_ID` に | `pipeline p` | 68。ERR の最初の行 = `conductor: error: invalid value: 'run ID (environment variable)'`。値が現れない |
| AC-251 | validate は run ID を受け付けない | `two_jobs(0)` | `validate --run-id x` | 68 |
| AC-252 | run ID の決定の順序 | 定義の誤り + `--run-id 'a b'` | `run a --run-id 'a b'` | 64(定義の検査が先。run ID は後) |
| AC-253 | run ID の決定はステートの読み込みの前 | 壊れたステート + `--run-id 'a b'` | `run a --run-id 'a b'` | 68(65 ではない) |
| AC-254 | `watch` は `--run-id` があれば定義を読まない | 定義が壊れている。`.r1.state` あり | `watch --run-id r1` | 0。表が出る |
| AC-255 | `watch`・`reset` は壊れた定義でも既定の環境変数名 | 定義が壊れている。`CONDUCTOR_RUN_ID=r1` | `reset` | 0。`.r1.state` だけが消える |
| AC-256 | run ID なしの `watch` に `Run ID` の行はない | run ID なしのステート | `watch` | OUT に `Run ID` の行がない |
| AC-257 | ファイル名と中身の `run_id` の食い違い | `conductor.psd1.r1.state` を `conductor.psd1.r2.state` に複製する | `run b --run-id r2` | 65。ERR = ``conductor: error: conductor.psd1.r2.state: state file is corrupt: run_id がファイル名の run ID と一致しません; run `conductor reset --run-id <ID>` to discard the state``(`<ID>` は文字どおり) |
| AC-258 | run ID ありでも `--remove-on-complete` で消える | `two_jobs(0)` | `pipeline p --run-id r1 --remove-on-complete` | 0。`conductor.psd1.r1.state` がない |
| AC-259 | `reset --run-id` は別の run に影響しない | `.r1.state`、`.r2.state`、`.state` | `reset --run-id r1` | `.r1.state` だけが消える |
| AC-260 | 大文字小文字を区別しないファイルシステム | macOS の既定 / Windows | `pipeline p --run-id A`、`pipeline p --run-id a` | 同じファイルになる。食い違いは、中身の `run_id` との照合で 65 として検出される(利用者は大文字小文字だけで run ID を区別しない)。区分: macOS / Windows。**未確認**(仕様の記述に基づく) |
### 4.9 リトライ

特に記さない限り、`RT` は `P(p; a:0{cmd="TJ flaky F <fails> 75", retry=(exit_codes=[75], max=3, interval="1s")})`(`F` は `T/flaky.count`。`<fails>` は失敗させる回数)。`flaky` は、実行のたびに `F` の回数を 1 増やす。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-261 | 2 回失敗して 3 回目に成功 | `RT`(`fails=2`) | `run a` | 0。OUT 空。ERR(-v なしでも)= `conductor: retry: job 'a' exited with code 75; retry 1 of 3 in 1 seconds`⏎`conductor: retry: job 'a' exited with code 75; retry 2 of 3 in 1 seconds`。`F` の値 = 3。ステート `a=success(0)` |
| AC-262 | -v で結果の行は 1 回だけ | 上と同じ | `run -v a` | ERR は上の 2 行のあとに `[conductor] job=a result=success (exit code 0)` の 1 行だけ(途中の失敗の結果行は出ない) |
| AC-263 | 使い切る | `RT`(`fails=99`) | `run a` | 75。ERR は retry の行 3 つ(`retry 1 of 3`、`2 of 3`、`3 of 3`)。`F` の値 = 4(最大 4 回の実行)。ステート `a=error(75)`、`resume=a` |
| AC-264 | 対象外の終了コードはリトライしない | `exit_codes: [75]` で、ジョブが 3 で失敗(`flaky F 99 3`) | `run a` | 3。retry の行なし。`F` の値 = 1 |
| AC-265 | `retry` がなければ 1 回だけ実行 | `P(p; a:0{cmd="TJ flaky F 99 75"})` | `run a` | 75。retry の行なし。`F` = 1 |
| AC-266 | 知らせは日本語でも | `RT`(`fails=1`)、`LANG=ja_JP.UTF-8` | `run a` | 0。ERR = `conductor: retry: ジョブ 'a' が終了コード 75 で終わりました。1 秒後にリトライします(1/3 回目)` |
| AC-267 | 倍率と上限の待ち時間 | `retry=(exit_codes=[75], max=3, interval="1s", multiplier=2, max_interval="2s")`、`fails=99` | `run a` | 75。retry の行の待ち時間は順に `1`、`2`、`2`(`in 1 seconds`、`in 2 seconds`、`in 2 seconds`)。実行全体の時間は 5 秒以上 |
| AC-268 | 待ち時間の列(関数の検査) | `interval` / `multiplier` / `max_interval` / `max` | 待ち時間を求める関数(`Flow`。結果は `[uint64]`) | `30s`、倍率なし、max 3 → 30、30、30。`10s`、2、`60s`、max 5 → 10、20、40、60、60。`60s`、3、`60s`、max 3 → 60、60、60。`1s`、10、`1h`、max 6 → 1、10、100、1000、3600、3600。`1h`、10、`24h`、max 4 → 3600、36000、86400、86400。極端に大きい値でも結果が `[uint64]::MaxValue` を超えない(`[bigint]` で計算して上限と比べる飽和。例外にも、折り返しにも、`[double]` への昇格にもしない。AC-501)。区分: 単体 |
| AC-269 | 再実行の結果が成功なら遷移へ(pipeline) | `P(p; a:0{cmd="TJ flaky F 1 75", retry=(…)}, b:0)` | `pipeline p` | 0。`b` も実行される。完了のステート |
| AC-270 | 使い切ると停止(pipeline) | `RT` に `b:0` を足し、`fails=99` | `pipeline p` | 75。`b` は実行されない。`resume=a` |
| AC-271 | ジョブ自身の 127 は一覧にあればリトライ | `TJ exit 127`、`exit_codes: [127]`、`max: 1`、`interval: 1s` | `run a` | 127。retry の行が 1 つ(1 回リトライしてから終わる) |
| AC-272 | 起動の失敗はリトライしない | 存在しないプログラム、`exit_codes: [127]` | `run a` | 127。retry の行なし |
| AC-273 | シグナルで終了したジョブはリトライしない(UNIX 系) | `argv=["sh","-c","kill -9 $$"]`、`exit_codes: [137]` | `run a` | 137。retry の行なし |
| AC-274 | 待機中のシグナル(UNIX 系) | `retry=(exit_codes=[75], max=3, interval="60s")`、`fails=99` | `run a` を起動し、ERR に最初の retry の行が出てから SIGTERM を送る(`Harness.Signal(process, 15)`) | 143。ステート: `a=pending`(エラーとして記録しない)、`resume=a`。2 つ目の retry の行は出ない。ジョブは再び起動されない(`F` = 1) |
| AC-275 | 待機中の中断は 1 秒以内 | `interval="1h"`、`fails=99` | 最初の retry の行のあと SIGTERM | SIGTERM を送ってから 1 秒以内に、143 で終わる(待ちは、250 ms 以下の `Interrupt.Wait(int)` の区間に分け、合図で直ちに起こす(07 章 8 章)。**`Start-Sleep -Seconds 3600` や、1 回の長い `[System.Threading.Thread]::Sleep` で待ってはならない**)。単体では、3600 秒の待機に 200 ms 後の合図(別のスレッドから、旗を立てる関数を呼ぶ)で、全体が 1.2 秒未満 |
| AC-276 | 再開は最初の実行から数え直す | 上のあと(`resume=a`) | 再度 `run a`(`interval="1s"`、`fails=1`) | `retry 1 of 3` の行から始まる。0 で完了 |
| AC-277 | 待機中のシグナル(pipeline) | 同様(pipeline) | `pipeline p`、待機中に SIGTERM | 143。以降のジョブを実行しない。`resume=a` |
| AC-278 | ステートは最終の結果で 1 回だけ | `RT`(`fails=2`) | `run a` | 実行の途中に、ステートファイルのジョブ `a` の状態が `success`/`error` にならない(最終の結果のときだけ変わる。途中の失敗はステートに書かれない)。リトライの回数はステートに書かれない。観測: 待機中にステートファイルを読む |
| AC-279 | README のリトライの説明 | README | (検査) | 見出し、5 つのキー名(`exit_codes`、`max`、`interval`、`multiplier`、`max_interval`)があり、載っている `.psd1` と JSON の例が実際に読み取りを通る。区分: 点検 |
### 4.10 シグナルと中断


UNIX 系の「実行」は、被試験体を起動し(`Start-ConductorProcess`)、`TJ sleep` の記録 `ready` が `O` に現れるのを待ってから、被試験体のプロセスにシグナルを送る(`Send-TestSignal`。`kill` コマンド。`Process.Kill` は強制終了なので使えない)。定義は `SL` = `P(p; sleeper:0{cmd="TJ --out O sleep"}, after:0)` を使う。
ジョブの終わり方(終了コードの値)と、シグナル番号の対応は、07 章の規則に従う(AC-504)。**`Process.ExitCode` が、シグナルで終了した子に対して返す値は、公式の文書で確認できていない(未確認)。実機での確認は AC-140 が兼ねる。**
シグナルのハンドラ(補助の部品の C# のコールバック)は、メインスレッド以外で実行されうる(AC-538)。**PowerShell のスクリプトブロックを、そのコールバックにしてはならない**(別のスレッドには、実行空間がなく、失敗する。AC-499)。`pwsh -File` は、Ctrl+C で中断されると終了コード 0 になる(確認: 1 章)ので、この節の終了コード(130 など)は、`conductor` 自身が返す値である。


| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-280 | 実行中の SIGTERM(UNIX 系) | `SL` | `run sleeper` を起動し、`ready` のあと SIGTERM | 143。OUT 空。ステート `S[sleeper=error, after=pending ; resume=sleeper]`。`pid=` に記録されたジョブのプロセスが存在しない(孤児なし。`kill(pid, 0)` が `ESRCH`) |
| AC-281 | HUP のあとすぐ INT(UNIX 系) | `SL`(`sleeper` だけでもよい) | `ready` のあと SIGHUP、続けて SIGINT | 129(最初の番号 = 1 が残る)。ステート `sleeper=error` |
| AC-282 | SIGINT / SIGQUIT 単独(UNIX 系) | `SL` | SIGINT / SIGQUIT | 130 / 131(SIGQUIT では、`TJ` の終わり方は実装時に確認。3.4 節。確かめるのは `conductor` の終了コードだけ) |
| AC-283 | -v の表示(UNIX 系) | `SL` | `run -v sleeper`、SIGTERM | 143。ERR に `[conductor] job=sleeper result=error (interrupted by signal 15)` |
| AC-284 | 時間を空けた 2 つのシグナルは最初の番号が残る | シグナル監視の単体 | INT のあと、時間を空けて HUP | 最初の番号 2 が残る(`Interlocked.CompareExchange` で、0 のときだけ書く)。区分: 単体(UNIX 系) |
| AC-285 | 最初の番号・登録は 1 回 | シグナル監視の単体 | 監視を登録し、HUP を受ける。再度の登録 | 最初の番号 1 が残る。2 回目の登録は「すでに登録済み」(失敗ではない。製品の側の状態で判定する。`PosixSignalRegistration` 自体は、同じシグナルへの複数の登録を許す見込みだが、確認していない。未確認)で、1 回目の登録が、そのまま 1 つだけ有効(ハンドラが 2 回呼ばれない)。区分: 単体(UNIX 系) |
| AC-286 | 転送は同じシグナルを 1 回だけ | 起動の部品(`Runner`)の単体 | ジョブの実行中に合図を立てる(SIGTERM と SIGINT で別々に) | ジョブは、そのシグナルで終了する(07 章の規則で番号 15 / 2)。送信は 1 回(続けて合図を立てても追加で送らない)。区分: 単体(UNIX 系) |
| AC-287 | 許可外の番号は送らない | 起動の部品の単体 | 合図の番号が SIGKILL(9)、SIGSTOP、SIGCONT、SIGUSR1、0、-1、9999 | 送信しない。ジョブは自分で終わる。終わりの観測の直後に `Interrupted = true`、`SignalNumber` がその番号。区分: 単体(UNIX 系) |
| AC-288 | ジョブのセッションとプロセスグループ | 起動の部品の単体。07 章 4.7 節の決定: UNIX 系でも、ジョブを新しいプロセスグループにも新しいセッションにも置かない | ジョブを起動して、プロセスグループ ID を調べる(`getpgid(pid)` の P/Invoke。テストの補助) | ジョブのプロセスグループ ID が、`conductor` のものと同じ。`conductor` は、シグナルを子の pid にだけ送る(プロセスグループには送らない)。Windows では行わない。区分: 単体(UNIX 系) |
| AC-289 | 先に終わったジョブには転送しない | 起動の部品の単体(偽の子で、終了済みを返す) | ジョブが終わった後に合図が立つ | 送信しない(終了済みの pid に `kill` しない。pid の再利用を避けるため、送る直前に `Process.HasExited` で確かめる。確認と送信の間の競合は残る。07 章の規則に従う)。区分: 単体(UNIX 系) |
| AC-290 | 起動前の合図 | 起動の部品の単体 | 合図が立った状態で起動を呼ぶ | ジョブは起動されない(`Started = false`、`Interrupted = true`、`SignalNumber` = 番号)。分類は「分類なし」、終了コード `128 + 番号`。区分: 単体 |
| AC-291 | 終了の観測の直後の合図 | 起動の部品の単体 | ジョブの実行中に、転送されない番号の合図を立てる(ジョブは自分で終わる) | ジョブの終了コードを保ったまま `Interrupted = true`、`SignalNumber` = その番号。分類は「エラー、`128 + 番号`」。区分: 単体 |
| AC-292 | 合図が立たなければ中断にならない | 起動の部品の単体 | 合図なしで `exit 1` | `Interrupted = false`、`SignalNumber` なし。区分: 単体 |
| AC-293 | 起動前の中断の記録(注入) | 流れの制御(偽の `ScriptProcess`・`FakeSignals`) | 起動前に合図が立つ | ジョブを起動せず、`resume` = そのジョブ、エラーとして記録しない、終了 `128 + 番号`。区分: 注入 |
| AC-294 | 実行中の中断の記録(注入) | 同上 | 実行中に合図(2 つ来ても最初の番号) | ジョブを `error` として記録、終了 `128 + 最初の番号`。区分: 注入 |
| AC-295 | ジョブのあいだの中断(pipeline、注入) | 同上 | 1 つ目のジョブが成功した直後に合図 | 次のジョブを起動せず、再開位置を記録して `128 + 番号`。-v で `[conductor] interrupted by signal N`。区分: 注入 |
| AC-296 | 検査中の合図より誤りが優先 | 同上 | 定義の検査中に合図が立つ。定義の誤り / 順序外 / スキップ | 64 / 66 / 0(中断の終了コードにならない)。区分: 注入 |
| AC-297 | シグナルの準備は検査の前 | 同上 | `run` / `pipeline`、`validate`・`watch`・`reset`・`convert`・引数の誤り | `run`・`pipeline` だけが、定義の検査より前にシグナルを準備する。ほかは準備しない。区分: 注入 |
| AC-298 | シグナルの準備の失敗は 70 | 同上 | 準備が失敗する | 70。ERR = `conductor: error: failed to set up signal handling`。定義の検査もジョブの起動もしない。区分: 注入 |
| AC-299 | 登録済みでも続ける | 同上 | 準備が「すでに登録済み」を返す | 流れは成功と同様に続く。区分: 注入 |
| AC-300 | 標準エラーが閉じていても本来の終了コード | 同上 | 標準エラーへの書き込みが失敗する(`BufSink` の書き込み失敗。`IOException`) | 本来の終了コードで終わる。区分: 注入 |
| AC-301 | Windows の Ctrl+C(手動) | Windows の実機、コンソール。`SL` | `conductor run sleeper` の実行中に Ctrl+C を押す | (1) 終了コード 130(`pwsh -File` の Ctrl+C の既定の 0 ではない。確認: 1 章)。(2) ジョブの子プロセスが残らない。(3) 続く `watch` で `sleeper` が `error`、`after` が `pending`。(4) `run -v` の ERR に `interrupted by signal 2`。補助の部品では、Ctrl+C は `Console.CancelKeyPress`(`ConsoleSpecialKey.ControlC`)で届く(07 章 7.3 節。公式の API 文書で確認。実機は未確認)。Ctrl+Break は対象外(OS の既定の動作。07 章 7.3 節)。結果を記録欄に残す(`tests/WINDOWS_CTRL_C_CHECKLIST.md`)。区分: 手動(Windows)。実施の記録は、この文書の作成時点では存在しない(未確認) |
### 4.11 ステートの異常系

ここの「実行」は、特に記さない限り `run a`(`P(p; a:0, b:0)`)。すべて、**ジョブは実行されず**、ステートファイルは**変わらず**、OUT は空。ERR の最初の行は `conductor: error: conductor.psd1.state: <種類の文>: <理由>; <案内>`。
案内は ``run `conductor reset` to discard the state``(run ID あり: ``run `conductor reset --run-id <ID>` to discard the state``)。ステートは JSON の 1 つのオブジェクト([06](06-state-file.md))。

| ID | 機能 | ステートの内容(前提) | 期待(終了 65) |
|----|------|------------------------|----------------|
| AC-302 | JSON として読めない | `{garbage` | `state file is corrupt: JSON として読めません` |
| AC-303 | UTF-8 でない | 不正なバイト列 | `state file is corrupt: UTF-8 の文字列ではありません` |
| AC-304 | 大きさの上限 | 正しいステートの JSON の末尾に、空白(スペース)を足して 1,048,577 バイト | `state file is too large: 大きさが上限(1 MiB)を超えています`。ちょうど 1,048,576 バイトは受理(実行が進む) |
| AC-305 | 書式の版が違う | `"format_version": 2`(ほかに未知のキーがあっても)/ `0` | `state file version is not supported: 対応していない版です(reset が必要です)` |
| AC-306 | 書式の版がない・整数でない | `format_version` なし / `"1"` / `1.0` / `true` / `1e0` | `state file is corrupt: format_version がありません` / `format_version が整数ではありません`(`1.0`・`true`・`1e0` も同じ) |
| AC-307 | 未知のキー | 最上位に未知のキー。`jobs` というキー | `state file is corrupt: 知らないキーがあります` |
| AC-308 | 必須のキーがない | `pipeline_name` などを欠く | `state file is corrupt: 必須のキーがありません` |
| AC-309 | 値の型・状態の誤り | `status: "weird"`、`skip_reason` の誤り、`skipped` に `skip_reason` なし、`success` に `skip_reason` あり、`pending` に `exit_code` あり | `state file is corrupt: status の値が誤っています` / `skip_reason が status と合いません` / `exit_code が status と合いません` など([06](06-state-file.md) の矛盾) |
| AC-310 | ジョブの並びの矛盾 | ジョブID の重複 / `job_order` と `job` の不一致 / `resume_job_id` が `job_order` にない / `completed: true` と `resume_job_id` | `state file is corrupt: ジョブIDが重複しています` / `job の並びが job_order と一致しません` / `resume_job_id が job_order にありません` / `completed と resume_job_id が両立しません` |
| AC-311 | 読めないファイル(UNIX 系、非 root) | ステートの権限を 000 に | `cannot read state file: 開く、または読み取ることができません` |
| AC-312 | 定義との食い違い: パイプライン名 | `pipeline_name` が定義と違う(定義側の名前を変えて `run`) | `conductor: error: conductor.psd1.state: state does not match the definition: pipeline name; run `conductor reset` to discard the state` |
| AC-313 | 定義との食い違い: ジョブID | ジョブの追加・改名のあと | `state does not match the definition: job ids; …` |
| AC-314 | 定義との食い違い: ジョブの並び | ジョブの順序を入れ替えたあと | `state does not match the definition: job order; …` |
| AC-315 | 食い違いの案内は run ID ありで変わる | `--run-id r1` のステートで定義を変える | `…; run `conductor reset --run-id <ID>` to discard the state` |
| AC-316 | 日本語 | `{garbage`、`LANG=ja_JP.UTF-8` | `conductor: error: conductor.psd1.state: ステートファイルが壊れています: JSON として読めません; `conductor reset` でステートを初期化できます` |

ステートの異常のあとの回復と、正常な読み込み。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-317 | reset で回復する | 上のいずれかのあと | `reset`、続けて `run a` | 0。先頭から実行される |
| AC-318 | pipeline も同じ | 上のいずれか | `pipeline p` | 65。ジョブは起動しない |
| AC-319 | ファイルなし・長さ 0 は「空」 | ステートなし / 長さ 0 | `run a` | 通常どおり 0 |
| AC-320 | 完了のまま残ったステートは「空」 | 完了のステート(正しい) | `pipeline p` | 先頭から実行して 0。読み込みでファイルは消されない(実行の最初の書き込みで上書きされる) |
| AC-321 | 残った一時ファイルは読み込みに影響しない | `.conductor.psd1.state.tmp.1.2.3` がある | `run a` | 通常どおり 0 |
| AC-322 | 書き込みは原子的 | 書き込み前後 | (ステートを読み続ける) | 読み手は、直前の完全な内容か新しい完全な内容だけを見る(同じディレクトリの一時ファイルへ書き、`[System.IO.FileStream]::Flush($true)`(ディスクへの書き出し)で同期し、置き換える(`[System.IO.File]::Move($src, $dst, $true)` か `[System.IO.File]::Replace`。06 章の決定)。UNIX 系では `rename(2)` に依る原子性。**Windows で、置き換え先が他のプロセスに開かれているときの挙動と、同時の読み手に対する原子性は、未確認**)。書き込みの失敗でも直前の内容が残る。置き換え先がディレクトリなら書き込みの失敗でステートは変わらず、一時ファイルも残らない。ディレクトリの `fsync` は、UNIX 系で行い(P/Invoke の `fsync`)、Windows では行わない(できない。06 章) |
| AC-323 | 排他制御はしない | 2 つの `run` を同時に | (2.1 節) | 両方が動く。AC-404 を参照 |
| AC-324 | 終了と書き込みの間の停止は再実行(at-least-once) | 終了の直後、書き込みの前に `conductor` が止まる | 再度 `run a` | ジョブが再実行されうる。**外からは再現できない**(偽の部品で検査する)。区分: 注入 |
### 4.12 引数の文法


特に記さない限り、定義は `B0`、OUT は空、ERR は `conductor: error: <文>`⏎(空行)⏎`Usage:` から始まる使い方の全文。「68」の行は、終了 68。引数は、PowerShell のパラメーター束縛に任せず、`$args`(の文字列の配列)を自前で解析する([04](04-cli.md))。理由は、この文法(`--opt=value`、短縮形の連結、`--`、値を取るオプションが `-` で始まる語も値にする、オプションの省略形を受理しない、エラーの優先順位と終了コード 68)が、PowerShell のパラメーター束縛(`-Verbose` などの共通パラメーター、パラメーター名の前方一致、`-` で始まる語の解釈)と衝突するため。共通パラメーターと `pwsh` のオプションに引数が食われないことは AC-547。


| ID | 機能 | 実行 | 期待 |
|----|------|------|------|
| AC-325 | 引数なし | `conductor` | 68。`missing argument` |
| AC-326 | オプションだけ | `-v` | 68。`missing argument` |
| AC-327 | 知らないコマンド | `bogus` / `''`(空文字) | 68。`unknown command: 'bogus'` / `unknown command: ''` |
| AC-328 | 必須の引数がない | `run` / `pipeline` / `convert` | 68。`missing argument` |
| AC-329 | 余分な引数 | `run a b` / `validate a b` / `watch x` / `reset x` | 68。`unexpected argument: 'b'` / `'b'` / `'x'` / `'x'` |
| AC-330 | 使えないオプション | `watch -v`、`reset -v`、`run --all a`、`run --to psd1 a`、`validate --remove-on-complete`、`reset --remove-on-complete`、`convert f --to psd1 -v` | 68。`option not allowed for this command: '<そのオプション>'` |
| AC-331 | `--run-id` が使えるコマンド | `run`、`pipeline`、`watch`、`reset` に `--run-id x` | 文法として受理される(4 コマンド)。`validate`・`convert` では 68 |
| AC-332 | 知らないオプション | `run --bogus a` / `run -x a` / `run --bogus=x a` | 68。`unknown option: '--bogus'` / `'-x'` / `'--bogus'`(`--conf` のような省略形は、`--config` として受理せず、`unknown option: '--conf'`) |
| AC-333 | 値の欠落 | `run a --config` / `run a --run-id` / `convert f --to` | 68。`option requires a value: '--config'` / `'--run-id'` / `'--to'` |
| AC-334 | 重複 | `run -vv a`、`run -v -v a`、`run -v --verbose a`、`run a --config x --config y`、`run a --run-id x --run-id y`、`reset --all --all`、`run a --remove-on-complete --remove-on-complete`、`convert f --to psd1 --to json` | 68。`invalid value: '<オプション名>'`(`-vv` は `'-v'`。重複を後勝ちで黙って受理しない) |
| AC-335 | `--all` と `--run-id` | `reset --all --run-id x` | 68。`invalid value: '--all'` |
| AC-336 | 値を取らないオプションに `=値` | `run --verbose=x a`、`reset --all=x`、`run a --remove-on-complete=x` | 68。`invalid value: '--verbose'` / `'--all'` / `'--remove-on-complete'` |
| AC-337 | `--to` の値 | `convert f --to xml` / `PSD1` / `Json` | 68。`invalid value: 'xml'` など(値そのもの) |
| AC-338 | `--to` がない | `convert f` | 68。`missing argument: '--to'` |
| AC-339 | `--run-id` の値の検査 | `run a --run-id 'a b'` | 68。`invalid value: '--run-id'`(値を含めない) |
| AC-340 | 空の `--config` | `run a --config=` | 64。`definition file does not exist`(空のパスとして受理され、後で誤りになる) |
| AC-341 | オプションの書き方 | `--config c.psd1 -v run a`、`run a -v`、`--config=c.psd1`、`--run-id=r1`、`convert f --to=psd1` | すべて文法として正しい(`run` は実行される。`--run-id=` は run ID なしに固定) |
| AC-342 | `--` のあとは位置引数 | `run -- -weird` / `-- --help` / `run a -- -v` | `job is not defined in the definition file: '-weird'`(68)/ `unknown command: '--help'`(68)/ `unexpected argument: '-v'`(68) |
| AC-343 | ヘルプ(英語) | `--help`、`-h` | 終了 0。OUT = [08](08-output-and-i18n.md) の使い方の全文(末尾は改行)。ERR 空 |
| AC-344 | ヘルプ(日本語) | `LANG=ja_JP.UTF-8` で `--help` | 0。OUT = 08 章の日本語の全文 |
| AC-345 | バージョン | `--version`、`-V` | 0。OUT = `conductor 0.8.0`(版は配布の版)。ERR 空。`LANG=ja_JP.UTF-8` でも英語・同じ |
| AC-346 | `-h`/`-V` は最優先 | `-vh`、`bogus --help`、`run -vv --help`、`-V -h`、`-h -V` | `-vh` は help(0)。`bogus --help` も help(0)。`-V -h` は version、`-h -V` は help(先に現れた方) |
| AC-347 | 使い方の誤りの形 | `bogus`(`LANG=C`)、`bogus`(`LANG=ja_JP.UTF-8`) | ERR の最初の行 = `conductor: error: unknown command: 'bogus'` / `conductor: error: 知らないコマンドです: 'bogus'`。2 行目は空行、3 行目以降は使い方の全文(英語 / 日本語)。OUT 空。ジョブは実行されない。**終了コードは 68** |
| AC-348 | 使い方の誤りの全種類 | 上の各種類 | 8 種類(`UnknownCommand`、`MissingArgument`、`UnexpectedArgument`、`UnknownOption`、`OptionNotAllowed`、`MissingValue`、`InvalidValue`、`UnknownJob`)が、すべて 68 で、08 章の英語・日本語の文になる |
| AC-349 | UTF-8 でない引数(UNIX 系) | 定義あり。位置引数(ジョブID)・`--run-id` の値・`--config` と `convert` の入力パスに不正なバイト列(`FF FE`。AC-406 と同じ方法で渡す)。実行: `run <不正なバイトのジョブID>`、`run a --run-id <不正なバイト>`、`--config <不正なバイトのパス>`、`convert <不正なバイトのパス> --to json` | **`pwsh` が不正なバイトのコマンドライン引数を引数の配列(`GetCommandLineArgs()`)にどう渡すかは、公式の文書で確認できていない(未確認)**。確かめるのは、実装が何らかの文字列を受け取っても、異常終了しないことと、出力が有効な UTF-8 になること。ジョブID: 68、`unknown job`(定義にないため)。`--run-id`: 68、`invalid value`(run ID の文字集合の外のため)。`--config` は 64、`convert` は 67(その名前のファイルがないため)。元のバイト列の名前のファイルを開けることは、期待しない(開けるかは未確認)。ERR の不正な文字は、エスケープ規則(08 章)どおりに出る(AC-519)。区分: UNIX 系 |
| AC-447 | 値を取るオプションの値は `-h`/`--help` でも値 | `--config --help run a`、`--config -h run a`、`run a --config --version`(定義ファイルは存在しない) | 終了 64。help・version は出ない。ERR に、その語をパスとしたファイルの誤り(`--help: ...` など) |
| AC-448 | 値を取るオプションが値を欠く | `--config --help` / `--help --config` | 前者は 68(`missing argument`。`--help` が値に取られて位置引数が 0 個)。後者は help(0) |
| AC-449 | `-h`/`-V`/`-` と `--` の細部 | `-h=x`、`-V=1`、`--help=x`、`-vh=x`、`run -`、`-- run`、`-- run a` | `-h=x`・`--help=x`・`-vh=x` は help(0)、`-V=1` は version(0)。`run -` はジョブID `-`(定義がなければ 64)。`-- run` は 68(`missing argument`。`run` がコマンド名でジョブIDなし)。`-- run a` は `run` のジョブID `a`(定義がなければ 64) |
### 4.13 出力とロケール

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-350 | 言語の決定: `LC_ALL` が最優先 | `LC_ALL=ja_JP.UTF-8`、`LANG=C` | `bogus` | ERR に `知らないコマンドです`(日本語) |
| AC-351 | 最初の空でない値だけを見る | `LC_ALL=C`、`LANG=ja_JP.UTF-8` | `bogus` | 英語(`unknown command`) |
| AC-352 | `LC_MESSAGES`、`LANG` の順 | `LC_MESSAGES=ja_JP`、`LANG=C` / `LANG=ja_JP`(ほか未設定) | `bogus` | 日本語 / 日本語 |
| AC-353 | 空の値は飛ばす | `LC_ALL=`(空)、`LANG=ja_JP.UTF-8` | `bogus` | 日本語(空の `LC_ALL` は無視) |
| AC-354 | 大文字小文字を区別する | `LANG=JA` / 未設定 | `bogus` | 英語 |
| AC-355 | 標準出力と標準エラーの使い分け | `P(p; a:0)` | `run -v a` ほか | `run`・`pipeline`・`reset`・`validate` は、OUT に何も書かない。OUT に書くのは `watch`、`--help`、`--version`、`convert` だけ。ジョブ本体の出力は、そのまま流れる |
| AC-356 | 成功・スキップで表示なし | `P(p; a:0)`、`LANG` に不正な値 | `run a` | OUT・ERR 空(表示がなければ言語も色も判定しない) |
| AC-357 | -v のトークンは言語によらず英語 | `LANG=ja_JP.UTF-8`、`P(p; a:1{w=[1]})` | `run -v a` | ERR = `[conductor] job=a result=warning (終了コード 1)`(`job=`、`result=`、`warning` は英語) |
| AC-358 | -v の補足: 起動できない | 存在しないプログラム、`-v` | `run -v a` | ERR に `[conductor] job=a result=error (could not start)` と、続いて `conductor: error: job 'a': cannot start command '…'`。日本語は `起動できませんでした` |
| AC-359 | -v の補足: シグナルで終了(UNIX 系) | `kill -9 $$` のジョブ | `run -v a`(日本語も) | `terminated by signal 9` / `シグナルで終了 9` |
| AC-360 | -v の補足: 中断(UNIX 系) | `SL`、SIGTERM | `run -v sleeper` | `interrupted by signal 15`(日本語 `シグナルで中断 15`)。起動前の中断は `interrupted by signal N, not started` / `シグナルで中断 N, 起動せず`(注入) |
| AC-361 | 全種類のイベントの形 | [08](08-output-and-i18n.md) のイベント | (表示の部品を直接呼ぶ) | すべての文言が空でなく、改行で終わり、生の制御文字を含まない。誤りの行は `conductor: error: ` で始まる。英語の表示は日本語の文字を含まない(原因の文を除く)。区分: 単体 |
| AC-362 | 定義の誤りの形 | `P(…)` の誤り | `validate` | 1 行ずつ `conductor: error: <ファイル>[:<行>]: <種類の文>: <原因>`。原因は日本語の固定の文(英語のロケールでも) |
| AC-363 | 定義の誤りの種類の文(日本語) | `LANG=ja_JP.UTF-8` | 4.1 節の各誤り | 種類の文が 08 章の日本語(例: `定義にないキーです`、`ジョブが 1 つもありません`) |
| AC-364 | 定義の誤りの種類の文(英語・retry) | 4.1 節の retry の各誤り | `validate` | 08 章の英語の文(`retry` の 9 種) |
| AC-365 | ステートの誤りの補足 | 4.11 節の各誤り | (文脈ごと) | 読み込み: 案内 `; …`。書き込み: ` (the job was not started)` / ` (the job has already been executed)`。`reset`・`watch` 以外の読み込みは案内つき、`reset` の削除は補足なし |
| AC-366 | 拒否・完了判定不能・出力失敗の文言 | 08 章 | (表示の部品を直接呼ぶ) | `cannot run this job now: …`、`cannot determine whether the pipeline is complete`、`cannot write to standard output` の英語・日本語。区分: 単体 |
| AC-367 | 完了を判定できない | 全ジョブが「実行済み」で通過し、どこでも完了にならない状態(通常は起こらない) | `pipeline p` | 65。ERR = `conductor: error: conductor.psd1.state: cannot determine whether the pipeline is complete`。ステートを初期化しない。区分: 注入 |
| AC-368 | 内部の誤りの文は固定の英語 | `Cli` が例外(`throw 'boom'` や .NET の例外)を投げる(偽の部品に投げさせる) | (入口の `Invoke-Conductor` の `try`/`catch`) | ERR = `conductor: error: internal error`(ロケールによらず。例外のメッセージとスタックトレースは出さない)、終了 70。標準エラーが閉じていても 70。区分: 注入 |
| AC-369 | 入口に異常終了を起こすコマンドがない | 製品の入口 | (点検) | 入口(引数の解釈の結果)に、わざと異常終了するコマンドがない。入口は 1 回だけ終了コードを返す。点検の項目は AC-421、AC-500 |
### 4.14 表示のエスケープ


入力に由来する文字列(ジョブID、名前、パス、キー名、コマンド名、使い方の誤りの対象)は、表示する前にエスケープする。**ジョブ本体の出力は通さない**(そのまま)。

規則: LF → `\n`、CR → `\r`、TAB → `\t`。そのほかの制御文字(U+0000〜U+001F、U+007F〜U+009F)と、書式・方向制御の文字(U+200B〜U+200F、U+2028、U+2029、U+202A〜U+202E、U+2060〜U+2064、U+2066〜U+2069、U+FEFF)は、`\u{<小文字の 16 進。先頭の 0 なし>}`。
孤立サロゲート(U+D800〜U+DFFF。UTF-16 の文字列で、対になっていない `[char]`)は、`\u{<小文字の 16 進>}`(例: `\u{d800}`)とする(08 章に従う)。PowerShell の `[string]` は .NET の文字列(UTF-16)で、`[char]` はコードユニットなので、**走査は `[char]::IsHighSurrogate`・`[char]::IsLowSurrogate` で対を判定して行い、孤立サロゲートは、対にならない `[char]` として検出する**(`$s.EnumerateRunes()` は、孤立サロゲートを U+FFFD に置き換えて返すので、置換された値を見ただけでは、元が孤立サロゲートか本物の U+FFFD かを区別できない。区別する実装にする。AC-519)。`foreach ($c in $s.ToCharArray())` の `$c` は `[char]`、`$s[$i]` も `[char]` で、`"$c"` や `-eq` で `[string]` と比べると型が変わる点(`[char]` と `[string]` の違い)に注意する。
そのほかの文字(日本語を含む)はそのまま。名前用は、エスケープ後 64 文字(**コードポイント**。`$s.Length`(UTF-16 のコードユニット)ではなく、サロゲートペアを 1 つと数える。サロゲートペアを途中で切らない)で切る(エスケープ列は 1 かたまり。次の 1 文字を足すと 64 を超えるなら、そこで打ち切り、末尾に `…`(U+2026)を 1 つ付ける。出力は最大 65 文字)。パスは切らない。結合文字の列は、コードポイントごとに数える(書記素では数えない)。

外から確かめる方法: `B0` に対して `run '<入力>'`(定義にないジョブID)を実行し、ERR の最初の行が `conductor: error: job is not defined in the definition file: '<出力>'` になることを確かめる。終了 68。


| ID | 機能 | 入力 | 出力(期待) |
|----|------|------|--------------|
| AC-370 | LF と ESC | `a<LF>b<ESC>[0m` | `a\nb\u{1b}[0m` |
| AC-371 | TAB | `k<TAB>x` | `k\tx` |
| AC-372 | CR | `a<CR>b` | `a\rb` |
| AC-373 | 日本語と空白 | `日本語 abc` | `日本語 abc`(そのまま) |
| AC-374 | ゼロ幅スペース | `a<U+200B>b` | `a\u{200b}b` |
| AC-375 | 右から左への上書き | `a<U+202E>b` | `a\u{202e}b` |
| AC-376 | 方向の隔離 | `a<U+2066>b` | `a\u{2066}b` |
| AC-377 | BOM | `a<U+FEFF>b` | `a\u{feff}b` |
| AC-378 | 行・段落の区切り | `a<U+2028>b<U+2029>` | `a\u{2028}b\u{2029}` |
| AC-379 | その他の制御文字 | `<U+007F><U+0085>`(NUL は引数に渡せないため、単体で確かめる) | `\u{7f}\u{85}`(NUL は `\u{0}`。区分: 単体) |
| AC-380 | 64 文字ちょうど | `k` × 64 | `k` × 64(切らない) |
| AC-381 | 64 文字を超える | `k` × 200 | `k` × 64 + `…` |
| AC-382 | エスケープ後で数える | LF × 100 | `\n` × 32 + `…`(65 文字) |
| AC-383 | 上限 3(単体)とコードポイントの数え方 | `abcdef`、上限 3。`😀` × 64 と `😀` × 65(`😀` は U+1F600。UTF-16 では 2 つの `char` だが、1 コードポイント。`string.Length` は 128 と 130) | `abc…`。空の入力は空。`😀` × 64 は切らない、`😀` × 65 は `😀` × 64 + `…`。サロゲートペアの途中で切れない(切った結果に孤立サロゲートがない)。区分: 単体 |
| AC-384 | 全部の部品で同じ結果 | 上の表の入力 | 定義・ステート・変換の誤りを作る各部品が、この表と同じ結果を返す(共通の関数を使ってよい)。区分: 単体 |
| AC-385 | ジョブIDは 64 文字で切る | ジョブID が 200 文字の定義 | `watch` の行・`-v` の行・`run` の誤りで、ID が 64 文字 + `…` |

表示の上限と、各部の適用。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-386 | パスは切らない | 300 文字のファイル名の定義(存在しない) | `validate --config <パス>` | ERR にパスの全体が現れる |
| AC-387 | リトライの知らせのジョブID | ジョブID に制御文字・200 文字 | リトライの行 | エスケープされ、64 文字で切られる |
| AC-388 | 定義の誤りのキー名 | 定義のキー名に制御文字 | `validate` | キー名がエスケープされ、64 文字で切られる |
### 4.15 終了コード全件

「終了コードの全件」の確認。各コードを起こす受け入れテストを、表で示す。

| 終了コード | 意味 | 起こすテスト |
|------------|------|--------------|
| 0 | 成功・警告・スキップ・`watch`・`reset`・`validate`・`convert` の成功・`--help`・`--version`・`reset --all` の警告 | AC-389 |
| 1 | 成功コードに含まれない 0、エラーで下位 8 ビットが 0 | AC-390 |
| ジョブの値(1〜255) | エラーのジョブの終了コード | AC-391 |
| 64 | 定義の誤り・`command` の誤り | 4.1 節 |
| 65 | ステートの誤り・書き込み・削除の失敗・`watch` の出力失敗・完了判定不能 | 4.11 節、4.3 節、4.6 節、4.5 節 |
| 66 | 拒否(順序外) | 4.3 節 |
| 67 | 変換の誤り・`convert` の出力失敗 | 4.7 節 |
| 68 | 使い方の誤り(知らないジョブID、run ID の誤りを含む) | 4.3、4.8、4.12 節 |
| 70 | 内部の誤り(未捕捉の例外を含む)・シグナルの準備の失敗 | 4.10 節・4.13 節(注入)、AC-490、AC-491、AC-492 |
| 127 | ジョブを起動できなかった | 4.2 節 |
| 128 + 番号 | ジョブがシグナルで終了、または中断 | 4.3 節、4.9 節、4.10 節 |

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-389 | 終了 0 の全種類 | 成功 / 警告(`w=[3]` で `exit 3`)/ スキップ(実行済み)/ `watch`・`reset`・`validate`・`convert`・`--help`・`--version` の成功 / `reset --all` の警告 | 各実行 | すべて 0 |
| AC-390 | 終了 1 の 2 種類 | `P(p; a:0{s=[5]})`(成功コードにない 0)/ エラーで下位 8 ビットが 0 になる終了コード | `run a` | 1 / 1(`exit 256` は Windows で確かめる。UNIX 系では OS が 0 にするので成功。関数の検査は下) |
| AC-391 | ジョブの値 | `exit 1`、`exit 2`、`exit 7`、`exit 255` | `run a` | 同じ値 |
| AC-392 | 終了コードの範囲合わせ(関数の検査) | 報告する値 `reported`(`[int]`。32 ビットの符号なしで渡される場合は、`[uint32]` から符号つきに直した値。`[int]` へのキャストは、範囲外で例外になる(`[int][uint32]3221225477` は失敗する。記憶による。未確認)ので、`[System.BitConverter]::ToInt32([System.BitConverter]::GetBytes([uint32]$v), 0)` で直す)、`isError` | 範囲合わせの関数(`Flow`) | `0..255` はそのまま。`259`→3、`511`→255、`-1`→255、`-2`→254、`3221225477` を直した値(-1073741819)→5、`4294967295` を直した値(= -1)→255。`256`・`512`・`-256`・`[int]::MinValue` は、エラーなら 1、エラーでなければ 0。PowerShell の `%`(負の数で、負の結果を返す)と `-band` の違いに注意する(`-1 % 256` は `-1` だが、`-1 -band 255` は `255`。下位 8 ビットは `-band 255` で取る)。`[int]` へのキャストの丸め(銀行丸め)・例外に頼らない。区分: 単体 |
| AC-393 | 中断の終了コード(関数の検査) | シグナル番号 n | `128 + n` を範囲合わせ(エラー扱い) | 15→143、2→130、1→129、3→131、300→172(範囲を超えない)。区分: 単体 |
| AC-394 | 分類の全 10 項目(関数の検査) | [07](07-process-and-signals.md) の分類の表の各行 | 分類の関数(`Flow`) | 順 1: 起動前の中断 → 分類なし、`128 + n`。順 2: 起動なし・中断でもない → エラー 127。順 3: 起動済みの中断 → `128 + n`(ジョブの終了コードを見ない)。順 4: 起動失敗 → 127。順 5: シグナル終了 n → `128 + n`。順 6: 終了コードなし → 1(`conductor` のエラー)。順 7: 成功コード → 0。順 8: 警告コード → 0。順 9: 0 で成功コードにない → 1(`conductor` のエラー)。順 10: そのまま。優先順位は上の行ほど高い。区分: 単体 |
| AC-395 | 終了コードの決定順序 | 複数の誤りを同時に持つ入力 | `run` | 検査の順(引数 68 → 準備 70 → 定義 64 → `command` 64 → ジョブID 68 → run ID 68 → ステートの読み込み 65 → 照合 65 → 起動前の判断 66/0 → 初期ステート 65 → ジョブ → 結果の書き込み 65 → 削除 65)で、最初の誤りだけが採られる。具体例は 4.3 節(定義の誤り > ジョブID)、4.8 節(定義 64 > run ID 68 > ステート 65) |
| AC-396 | 128 + 番号の例 | ジョブが SIGKILL / 実行中に SIGTERM(UNIX 系) | `run a` | 137 / 143 |
### 4.16 非機能要件の検証

2 章の各基準の受け入れテスト。「計測」は p95 などの測定、「強制」は基準超過で不合格(2 章の扱いの列)。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-397 | 定義の読み込みの性能 | ジョブ 10 個(約 2 KiB)と 100 個(約 20 KiB)の定義(ファイルから読む) | 最初の 5 回を捨てて、読み込みと検証を 50 回(プロセス内。`Measure-Command`) | p95 が 150 ms 以下(10 個)、1,500 ms 以下(100 個)(仮。目安。超えたら警告を記録)。区分: 計測 |
| AC-398 | 起動前の処理の性能 | ジョブ 10 個の定義 | `command` の検査、ステートの読み、照合、速い書き込みを、最初の 5 回を捨てて 50 回 | p95 が 150 ms 以下(仮。目安)。区分: 計測 |
| AC-399 | `run` のオーバーヘッド | ジョブ 10 個の定義(各ジョブは `TJ timing`) | `run job0` を 10 回 | `conductor` 自身の時間(全体 − `TJ` の基準の起動の費用 − (`end_ns` − `start_ns`))が p95 5,000 ms 以下(仮。目安。ランチャーの `pwsh` の起動を含む)。区分: 計測 |
| AC-400 | `pipeline` のジョブあたり | 同上 | 10 個と 1 個の `pipeline` を各 5 回 | ジョブ 1 つあたりの増分(2.1 節の式)が p95 700 ms 以下(仮。目安)。区分: 計測 |
| AC-401 | 大きな定義 | ジョブ 1,000 個の定義(約 200 KiB) | `run job0` | 20,000 ms 以内(仮。目安)。区分: 計測 |
| AC-402 | 大きな流れ | ジョブ 1,000 個の `pipeline`(偽の部品で、プロセスもファイルも使わない) | (内部の流れ) | 60,000 ms 以内(仮。目安)。完了のステートが残ることは常に検査。区分: 計測・単体 |
| AC-403 | 待機中の CPU 時間(Linux) | `TJ sleep 4`(約 4 秒)のジョブ | `run a` を起動し、`conductor` の pid の `/proc/<pid>/stat` の `utime + stime`(14・15 番目の値。単位は時計の刻み。`getconf CLK_TCK` で秒に直す)を、起動の 1 秒後と 2.5 秒後に読む | 増分が 150 ms 以下(仮。実測してから確定)(待機中に、子の終了の確認(10 ms 以下の間隔。07 章)で CPU を使いすぎない)。強制(`/proc` が読めなければスキップ)。区分: Linux |
| AC-404 | 同時の 2 つの `run` | `P(p; a:0{cmd="TJ sleep 1"})`(終了コード 99) | 同じ定義に `run a` を 2 つ同時に | 両方が実行され(ジョブが 2 回始まる)、どちらも終了コード 99。ステートは正しい形で読み戻せる。強制 |
| AC-405 | ランダムな引数 | 固定の種(`new Xorshift32(1)`) | 語の表から 0〜6 個を選んだ引数列で 500 回(`Cli` を偽の部品で、モジュールの内側の入口(`Invoke-ConductorMain`)を通して呼ぶ) | 終了コードが 0〜255。標準エラーに、制御文字(改行を除く)と U+202E がない。例外が漏れない |
| AC-406 | UTF-8 でない引数(UNIX 系) | `0xff 0xfe` を含むバイト列。`ProcessStartInfo.ArgumentList` は文字列しか受け付けず、不正なバイト列をそのまま渡せない。そこで、テストの補助が、`/bin/sh -c 'exec "$0" "$@"' conductor run "$(printf '\377\376')"` の形で、シェルに引数を組み立てさせて起動する(テストの補助だけがシェルを使う。製品はシェルを使わない) | ジョブID・`--config`・オプション・`convert` の引数に渡す | 異常終了しない(終了コードが 0〜255。70 にならない)。標準エラーが、有効な UTF-8 で読める。区分: UNIX 系 |
| AC-407 | 壊れたステート | 状態・再開位置・フラグを乱したステート、ランダムなバイト列、巨大な値 | `run` / `watch` | 異常終了しない。壊れていれば 65 |
| AC-408 | 空・制御文字のジョブID | ジョブID が空、制御文字を含む | `run <ID>` | 異常終了しない(68) |
| AC-409 | ランダムな `command` | 文字集合 `a b ⇥ ⏎ ' " \ ; \| & $ * あ` から長さ 0〜39 の文字列 2,000 件(定義の `command` に入れる) | `validate`(分割の単体も) | 0 か 64 のどちらか。0 のとき `program` は空でない |
| AC-410 | 大きな `command` | `prog ` + `'a b' "c\"d" e ` を 1 万回(約 3 万引数) | `validate` | 0。閉じない引用符を足したものは 64。120 秒以内(`Slow`)。文字列・配列の `+=` の繰り返しによる 2 乗の時間(`[System.Text.StringBuilder]` と `[System.Collections.Generic.List[string]]` を使う)や、再帰でスタックが溢れることがない |
| AC-411 | 起動できない `command` | `''`、`'`、`""`、存在しない名前、ディレクトリ、`/` | `validate` / `run a` | 前 3 つは 64、残りは 127。異常終了しない |
| AC-412 | 入れ子の深い入力 | 深さ 1,000 の `.psd1` と JSON(定義、ステート) | `validate` / `run a` / `convert` | 64 / 65 / 67。異常終了せず(スタックの溢れでプロセスが落ちず)、関数の再帰に頼らずに動く |
| AC-413 | 変換の頑健性 | 範囲外・空・壊れた・切り詰めた入力、ランダムな構造 | `convert` | 異常終了しない。成功した構造は往復で等しい |
| AC-414 | 内部の誤り | 異常終了を起こす | (入口) | 4.13 節の「内部の誤りの文」と同じ(AC-368)。実際のプロセスでの検査は AC-490、AC-491 |
| AC-415 | 検証は副作用を持たない | 4.1 節の最後の項目 | — | AC-057 |
| AC-416 | 入力の値を表示に出さない | 各種類のエラー(定義・`command`・ステート・変換・使い方)に、秘密のつもりの文字列を含む入力。PowerShell のパーサーが、入力の断片を含むメッセージのエラー(`ParseError.Message`)を返す入力(例: 不正な文字を含む行、閉じていない引用符の行)も含む | 各コマンド | ERR に、その値が現れない(キー名は、エスケープして現れてよい)。`command` の誤りの文に `command` の中身が現れない。例外やエラーのレコードの表示(`   at ` で始まる行、`At line:`、`Exception:`、`+ CategoryInfo`、`System.` で始まる例外の型名)が現れない |
| AC-417 | ステートに機微な値を書かない | 4.3 節の項目 | — | AC-148 |
| AC-418 | 変換に副作用がない | 4.7 節の項目 | — | AC-235。加えて、`Convert` が `System.Diagnostics.Process`・ネットワークの型を使わず、ファイルを開くのが入力ファイルの読み取りの入口だけであること(点検) |
| AC-419 | ネットワークに接続しない | 実装と依存 | 点検(構文木の走査) | 製品のコードが `[System.Net.*]` の型(`System.Net.Http`・`System.Net.Sockets`・`System.Net.Security` など)、`Invoke-WebRequest`・`Invoke-RestMethod`・`Test-Connection`・`Install-Module`・`Update-Help` を使わない。実行時の依存はない(AC-443)。区分: 点検 |
| AC-420 | OS 固有コードの隔離と、動的実行・ネイティブの限定 | ソース | 点検(構文木の走査) | `$IsWindows`・`$IsLinux`・`$IsMacOS`・`[System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform`・`[System.Environment]::OSVersion` の参照が、`src/Conductor/Private/Runner/Get-ConductorPlatformKind.ps1` の外にない。`Invoke-Expression`・`iex`・`[scriptblock]::Create`・`[System.Reflection.Assembly]::Load*`・`New-Object -ComObject` が、どこにもない(0 か所)。`Add-Type` が、補助の部品のファイル(`src/Conductor/Private/Runner/NativeHelper.ps1`。関数 `Initialize-ConductorNativeHelper` を持つ。決定。02 章・07 章に従う)の 1 か所だけにあり、その C# のソースは、リポジトリに置いた定数の文字列。P/Invoke(`DllImport`)が、07 章が許す関数(`kill`、ステートのディレクトリの同期の `open`・`fsync`・`close`)の宣言だけで、その C# のソースの中にだけある。`Start-Job`・`Start-ThreadJob`・`ForEach-Object -Parallel`・`[System.Threading.Thread]`・`[System.Threading.Tasks.Task]` の使用は、07 章の所有者・終了条件の表にある箇所だけ(許可の一覧)。区分: 点検 |
| AC-421 | 異常終了・取り違えを起こす書き方がない | ソース、`PSScriptAnalyzerSettings.psd1` | 点検(構文木の走査と、PSScriptAnalyzer) | 製品のコード(`src/`)に、(1) `exit`・`[System.Environment]::Exit`・`[System.Environment]::FailFast`(ランチャー以外。AC-500)、(2) `Invoke-Expression`・`iex`(AC-420)、(3) `Write-Host`・`Write-Output`・`Write-Error`・`Write-Warning`・`Write-Verbose`・`Write-Information`・`Out-Host`・`Out-Default`・`Read-Host`(出力は `Report` と、標準出力・標準エラーの `Stream` を通す。AC-552)、(4) `Start-Process`・`Start-Sleep`(長い待ち。AC-503)・`Get-Content`・`Set-Content`・`Add-Content`・`Out-File`・`Remove-Item`・`Move-Item`・`New-Item`・`Test-Path`・`Resolve-Path`・`Join-Path`(ファイルの入出力は `[System.IO.File]`・`[System.IO.Directory]`。AC-556、AC-559)、(5) `ConvertFrom-Json`・`ConvertTo-Json`・`Import-PowerShellDataFile`(`Conductor.psm1` が版を読む 1 か所を除く)・`Import-Clixml`・`Export-Clixml`(自前の読み取り・書き出しを使う。決定は 06 章・03 章)、(6) 許可箇所以外の `Add-Type`、(7) `-match`・`-notmatch`・`-replace`・`-split`・`-like`・`-notlike`(AC-542)と、文字列リテラルを含む大文字小文字を区別しない比較(`-eq`・`-ne`・`-lt`・`-gt`。AC-553)、(8) 文化依存の API(AC-540)、(9) `$global:`・`$LASTEXITCODE`・`$?`(AC-557)、(10) `#pragma` 相当の抑止(`[Diagnostics.CodeAnalysis.SuppressMessageAttribute]`)(理由つきの許可の一覧にあるものを除く)、(11) `catch { }`(空の `catch`)・`catch` で `Exception` を黙って捨てる書き方、(12) `trap`(`try`/`catch`/`finally` を使う)、(13) `Set-StrictMode` を下げる呼び出し、(14) `Set-ExecutionPolicy`、(15) `Set-Location`・`Push-Location`・`Pop-Location`(利用者のセッションの現在の場所を変えない。AC-559)がない。`Exception` の捕捉は、`Invoke-Conductor` の入口と、07 章が許す箇所だけ。配列・ハッシュテーブルの添字は、`IndexOutOfRange`・`KeyNotFound` 相当を起こしうる箇所で、先に検査する。区分: 点検 |
| AC-422 | テスト専用の経路を持たない | ソース、配布物 | 点検 | 入口に、異常終了を起こす経路や、テスト用の環境変数・隠しオプションがない。`src/` が `tests/`・`build/` を参照しない(ドットソース・`Import-Module` のパスに現れない)。配布物のファイル一覧に、`tests/`・`build/`・`demo/`・設定ファイルが含まれない(含まれるのは、`bin/`、`Conductor/`、`LICENSE`、`README.md`。AC-440、AC-526)。テスト用ジョブ `tj.ps1`・`inject.ps1` が入っていない |
| AC-423 | 著作権表示 | ソース | 点検 | 各 `.ps1`・`.psm1`・`.psd1` ファイルの先頭(`#requires` の行があればその前)に Apache License 2.0 のヘッダ(`Copyright 2026 agwlvssainokuni`)がある。`.yml`・`.sh`・`.cmd` も同じ(コメントの書式は各言語に合わせる)。`artifacts/`・`tests/fixtures/` は対象外。区分: 点検 |

### 4.17 デモの受け入れテスト

デモの仕様は 7 章。デモのスクリプトは、UNIX 系と Windows で同じ `demo/run-demo.ps1`。実行は、`pwsh -NoProfile -NonInteractive -File demo/run-demo.ps1`(リポジトリの根から)に、環境変数 `CONDUCTOR=<被試験体>`(ランチャーのパス)、`DEMO_LANG=en_US.UTF-8`、`DEMO_SLEEP=0`、一時ディレクトリの親(UNIX 系は `TMPDIR`、Windows は `TEMP`・`TMP`)を空のディレクトリ `D` にして行う。標準入力は空。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-424 | スクリプトが全部通る(UNIX 系と Windows) | 上の設定 | `pwsh -NoProfile -NonInteractive -File demo/run-demo.ps1` | 終了 0。見出し `=== 0. ` から `=== 10. ` までが順に出る |
| AC-425 | 場面 7(検査) | 同上 | 同上 | 出力に `definition is valid`、`command has an unclosed quote`、`→ 終了コード 64` が現れる。誤りの定義の検査のあと、ステートの一覧が `(なし)` |
| AC-426 | 場面 4(再開・完了) | 同上 | 同上 | `completed` の行、`skipped (already executed)` が現れる |
| AC-427 | 場面 8(run ID) | 同上 | 同上 | `conductor.psd1.build-1.state` が一覧に現れる。再開のあと `completed`。`reset` のあと一覧は `(なし)` |
| AC-428 | 場面 9(環境変数と後始末) | 同上 | 同上 | `conductor.psd1.opt-1.state` が現れ、最後の `watch` が `no state` |
| AC-429 | 場面 10(リトライ) | 同上 | 同上 | `conductor: retry: job 'flaky' exited with code 75; retry 1 of 3 in 1 seconds` と `… retry 2 of 3 in 1 seconds` の 2 行、`[flaky] 3 回目の実行`、`job=flaky result=success` が現れる |
| AC-430 | 利用者の環境に左右されない | `CONDUCTOR_RUN_ID=zzz`、`DEMO_RUN_ID=zzz`、`LC_ALL=ja_JP.UTF-8`、`LC_MESSAGES=ja_JP.UTF-8`、`NO_COLOR=1`、`PSModulePath` を別の値、`POWERSHELL_TELEMETRY_OPTOUT` 未設定、`PSExecutionPolicyPreference=Restricted`(Windows) を設定 | 同上 | 出力に `zzz` が現れない。英語の表示(`DEMO_LANG` のとおり)。結果が上と同じ(実行ポリシーが `Restricted` の場合に動かないなら、`-ExecutionPolicy` の扱いを 07 章・6.5 節に従って直し、この行の期待を実機の結果に合わせる。**未確認**) |
| AC-431 | 何も残さない | 同上 | 同上 | 終了後、一時ディレクトリの親 `D` に何も残らない。リポジトリの作業ツリーは変わらない(`artifacts/` などの `.gitignore` の対象を除く) |
| AC-432 | デモの定義 | `demo/conductor.psd1` | `validate`(一時ディレクトリで) | 0。4 ジョブ(`extract`、`transform`、`load`、`notify`)と、`extract` の警告の遷移(`notify` へ)を持つ。UNIX 系と Windows で同じ定義(ジョブは `pwsh` のスクリプト)。`.psd1` として `Import-PowerShellDataFile` でも読める |
| AC-433 | Windows: `.cmd` のランチャーで全部通る | Windows。`CONDUCTOR=bin\conductor.cmd` | `demo\run-demo.ps1` | 0。AC-424〜429 と同じ出力。区分: Windows |
| AC-434 | Windows: 出力が UTF-8 で化けない | Windows。標準出力をファイルに | 同上 | 出力のバイト列が、BOM なしの UTF-8 で、日本語の行(`→ 終了コード`、`デモは以上です。`)を含む(`[Console]::OutputEncoding` を変えない。AC-520) |
| AC-435 | Windows: 一時ディレクトリ | Windows | 同上 | 一時ディレクトリは `%TEMP%` の下(`conductor-demo-<乱数>`)にでき、終了後に消える |
| AC-436 | Windows: 場面 8〜9 | 同上 | run ID を使う一連(場面 8、9) | run ID ごとにステートが分かれる。`--run-id` が環境変数に勝つ。`reset --run-id` と `--all` が働く |
| AC-437 | Windows: 場面 10 | 同上 | `pipeline --config retry.psd1 retry-demo` | 2 回失敗してから成功する |
| AC-438 | 利用者の run ID に影響されない | 利用者の run ID の環境変数を設定 | 同上(UNIX 系と Windows) | 結果が変わらない |

### 4.18 リリース物の受け入れテスト

リリースの仕様は 6 章。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-439 | 成果物の名前と形 | リリース | (成果物の一覧) | `conductor-<版>.zip`、`conductor-<版>.tar.gz`、`SHA256SUMS` の計 3 つがある(OS ごとに分けない。PowerShell のモジュールは OS に依らない)。区分: 点検 |
| AC-440 | 展開の構造 | 各アーカイブ | 展開(`tar`、`System.IO.Compression`) | 最上位に、`conductor-<版>/` の 1 つのディレクトリだけがあり、その中が、`bin/conductor`、`bin/conductor.cmd`、`Conductor/`(モジュール。`Conductor.psd1`・`Conductor.psm1`・`Public/`・`Private/`)、`LICENSE`、`README.md` だけ(`tests/`・`build/`・`demo/`・`artifacts/`・`.github/` を含まない)。tar.gz では `bin/conductor` の実行権限が保たれる。区分: 点検 |
| AC-441 | チェックサム | `SHA256SUMS` | `sha256sum -c`(macOS は `shasum -a 256 -c`)の形式で検査 | 2 つのアーカイブの行があり、検査が通る。区分: 点検 |
| AC-442 | 版の表示 | 各アーカイブを展開した結果(実行できる OS で) | `conductor --version` | 0。OUT = `conductor <版>`。区分: 点検(3 OS) |
| AC-443 | 実行時の依存が無い | `Conductor.psd1`、各ソース | 点検 | マニフェストの `RequiredModules`・`ExternalModuleDependencies`・`NestedModules` が空(または無い)。ソースに `#requires -Modules` と、自分のモジュール以外の `Import-Module`・`using module` がない。区分: 点検 |
| AC-444 | タグの検査 | タグ `v<版>` | リリースの流れ | タグが `main` 上にない、または版が `Conductor.psd1` の `ModuleVersion` と違うと失敗する(タグが `main` 上にない旨 / タグと版の不一致の旨)。区分: 点検 |
| AC-445 | 下書きとプレリリース | リリースの作成 | (作成されたリリース) | 下書きである。版が先行版(SemVer の `-` を含む。例: `0.9.0-rc.1`)ならプレリリース。注記に固定の 1 文がある(`ModuleVersion` は先行版の接尾辞を持てず、`PrivateData.PSData.Prerelease` に書く。**マニフェストの `Prerelease` と `ModuleVersion` の扱いは、実装時に確認。未確認**)。区分: 点検 |
| AC-446 | 3 OS で受け入れテストが通る | Linux・macOS・Windows の実機(または同等の実行環境) | 4 章の自動の項目(各 OS で行える項目)。`CONDUCTOR_BIN` に、AC-442 で展開したランチャーを渡す形でも通ること | 全項目が合格する(OS の区分の項目は、その OS で)。区分: 自動 |

### 4.19 `.psd1` と JSON に固有の項目

定義の形式(`.psd1`・JSON)、ステートの形式(JSON)、変換(`.psd1` ⇔ JSON)の、形式固有の落とし穴を確かめる。特に記さない限り、「64」の期待は 4.1 節と同じ(終了 64、OUT 空、ステートなし、ジョブを起動しない。ERR の最初の行は `conductor: error: <ファイル>:<行>: ` で始まる(行がわかる誤り)。種類の文は [03](03-definition-format.md)・[08](08-output-and-i18n.md) のとおり)。**`.psd1` の検査は、PowerShell の構文木で行い、コードを実行しない。位置は、`IScriptExtent` の `StartLineNumber`・`StartColumnNumber` から取る**([03](03-definition-format.md))。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-450 | `.psd1` の定義 | `P(p; a:0)` の `T/conductor.psd1` | `validate`、`run a`、`validate --config conductor.psd1` | 0 / 0 / 0。ステートは `T/conductor.psd1.state`。`--config` を省くと、`conductor.psd1` を探す(`conductor.json` は探さない) |
| AC-451 | JSON の定義 | `B0` を JSON で書いた `T/conductor.json` | `run a --config conductor.json` | 0。ステートは `T/conductor.json.state`。`.psd1` の定義と同じ結果 |
| AC-452 | 拡張子が違うファイルは拒否 | `B0` の内容を、`conductor.xml`、`conductor.txt`、`conductor.csv`、拡張子なしの `conductor`、`conductor.psd1.bak`、`conductor.ps1` に置く(ファイルが存在しないパス `nofile.txt` も) | `validate --config <ファイル>` | 64。ERR の最初の行は `conductor: error: <ファイル>: unsupported definition file format: 拡張子は .psd1、.json のいずれかで書きます`(行なし。理由の字句は [03](03-definition-format.md))。大文字の拡張子 `conductor.PSD1`・`conductor.JSON` は受理(0)。ステートなし。ジョブは起動されない(`convert` の入力は拡張子を見ない。AC-209。拡張子の判定の細部は AC-545) |
| AC-453 | `.psd1` の受理する構文 | 次の各書き方で、`B0` と同じ意味の定義(別々の定義): (a) キーを裸の単語 / 単一引用符 / 二重引用符(`$` と `` ` `` を含まない)で。(b) 区切りが改行 / `;` / 混在。(c) 配列が `@(…)` / カンマ区切り(`@()` なし)/ 要素の間が改行だけ。(d) `#` の行コメントと `<# … #>` のブロックコメントを、キーの前後・値のあとに。(e) 文字列が単一引用符(`''` で `'` を表す)/ here-string(`@'…'@`)。(f) `$true`・`$false`・`$null`。(g) 整数が 10 進と `0x` の 16 進 | `validate` | 0。同じ木になる。コメントの `#` と、文字列の中の `#` を取り違えない(`command = 'echo #x'` は文字列) |
| AC-454 | `.psd1` はコードを実行しない | `T` に空のファイル `pwned.txt` がない。定義の値に、(a) `$(New-Item pwned.txt)`、(b) `Set-Content pwned.txt x`、(c) `& { Set-Content pwned.txt x }`、(d) `[System.IO.File]::WriteAllText('pwned.txt', 'x')`、(e) `Invoke-Expression 'Set-Content pwned.txt x'`、(f) `"$(Set-Content pwned.txt x)"`、(g) `$env:COMSPEC`、(h) `(Get-Date)`、(i) `1 + 1`、(j) `-join 'a','b'`、(k) `'a' -f 1`、(l) `[ordered]@{ a = 1 }`、(m) `@{ a = 1 } \| Out-Null`(それぞれ別の定義) | `validate`、`run a` | 64。`syntax error`(03 章の固定の理由)。**`T` に `pwned.txt` ができない**(どれも評価されない)。ERR に、式の断片が現れない |
| AC-455 | `.psd1` の文字列でないキーを拒否 | `@{ 1 = 'x' }`、`@{ $true = 'x' }`、`@{ (1) = 'x' }`、`@{ "$x" = 'x' }`、`@{ [int]1 = 'x' }`(トップレベルまたは `job` の中) | `validate` | 64。理由は `キーは文字列で書きます`(または、構文の理由) |
| AC-456 | `.psd1` の重複キー(入れ子、大文字小文字) | `job` の要素に `id` を 2 回、`pipeline` に `name` を 2 回、`id` と `Id`(大文字小文字だけ違う)、`'id'` と `id`(引用符の有無だけ違う) | `validate` | 64。`syntax error`(`キーが重複しています`)。行は 2 回目のキーの行。**`.psd1` のキーは、大文字小文字を区別せず、重複とみなす**(PowerShell の `@{}` が、大文字小文字を区別しないハッシュテーブルのため。03 章 4.5 節が正本) |
| AC-457 | 最上位の構造 | (a) `@{…}` の前後に別の文(`$x = 1`、`Get-Date`、`param()`、`using namespace System`、`#requires -Version 7`)。(b) `@{…}` が 2 つ。(c) ファイルが `@{…}` を `(…)` で囲む。(d) `return @{…}`。(e) ファイルの先頭に通常のコメント(`# note`、`<# note #>`) | `validate` | (a) 64(`.psd1 で使えない書き方です`。`#requires` も `ScriptRequirements` として拒否する。03 章 4.2 節)。(b) 64(`.psd1 で使えない書き方です`)。(c)(d) 64。(e) 0。いずれも評価されない |
| AC-458 | 数値・真偽値に見えるジョブID | `id = 1`、`id = $true`、`id = '1'`、`id = 'yes'`、`id = 'on'`、`id = 0x10`、`id = 1.5`、`id = $null`、`id = true`(裸の単語)。JSON は `"id": 1`、`"id": true`、`"id": "1"` | `validate`(それぞれ別の定義) | `id = 1`・`$true`・`0x10`・`1.5`・`$null`・裸の `true`・JSON の `1`・`true` は 64(`wrong value type`。裸の単語は構文の誤りになる場合があり、03 章に従う)。`id = '1'`・`'yes'`・`'on'`・JSON の `"1"` は 0(文字列)。型は自前で決める(PowerShell の型変換・`-eq` の変換に頼らない。AC-553) |
| AC-459 | `command` の書き方 | `.psd1`: here-string(`@'`⏎`TJ --out O echo a`⏎`b`⏎`'@`)、二重引用符の文字列(`$` と `` ` `` を含まない。`"TJ --out O echo a b"`)、複数行の単一引用符の文字列、`@('TJ', …)` の配列形式(`TJ` は、各書き方に合わせて語を書く) | `run a` | 0。`O` の引数は、それぞれ `a`、`b`(改行・空白は区切り) / `a`、`b` / 同様 / 配列の要素どおり |
| AC-460 | 定義の中の null の扱い | `.psd1`: `name = $null`、`command = $null`、`warning_exit_codes = $null`、`retry = $null`、`remove_on_complete = $null`。JSON: `"command": null`、`"name": null` | `validate` | 64。`name`・`command`・`warning_exit_codes`・`remove_on_complete`・JSON の `null` は `wrong value type`。`retry = $null` は `retry must be a map`。未知のキーの値が `$null` なら `unknown key`([03](03-definition-format.md))。「キーがない」と「値が `$null`」を区別する(`$h.ContainsKey()` と値の比較を混同しない。`$null -eq $x` の向きに書く。AC-555) |
| AC-461 | JSON の重複キー | `{"format_version": 1, "format_version": 1, ...}`(`conductor.json`)、`job` の要素の `id` の重複 | `validate --config conductor.json` | 64。`syntax error`(理由 `キーが重複しています`)。重複の検出は、自前の読み取りで行う([03](03-definition-format.md))。`ConvertFrom-Json` は、重複キーを黙って後勝ちにするか、エラーにするかが公式の文書で確認できていない(未確認)ので、使わない。位置(行・列)は、自前の読み取りが、改行の数から求める(AC-475) |
| AC-462 | JSON のコメントを拒否 | 行コメント `// c`、ブロックコメント `/* c */` を含む定義 | `validate --config conductor.json` | 64。`syntax error`(`ConvertFrom-Json` がコメントを受理する場合があるかは未確認なので、使わない。自前の読み取りは、コメントを拒否する) |
| AC-463 | JSON の末尾のカンマを拒否 | `[1,]`、`{"a": 1,}`(定義の中) | `validate --config conductor.json` | 64。`syntax error` |
| AC-464 | JSON の非標準の書き方を拒否 | 一重引用符の文字列、`NaN`、`Infinity`、`-Infinity`、先頭が 0 の数(`01`)、引用符のないキー、値のあとの余分なデータ(`{...} {...}`、`{...} x`) | `validate --config conductor.json` | 64。`syntax error`(それぞれ別の定義。自前の読み取りは、値のあとの余分なデータを拒否する(03 章)) |
| AC-465 | JSON の最上位がオブジェクトでない | `[]`、`"x"`、`1`、`null`、`true` | `validate --config conductor.json` | 64。`wrong value type`(理由 `最上位はマップで書きます`。JSON の `null` も同じ。[03](03-definition-format.md)) |
| AC-466 | 大きさの上限(JSON と `.psd1`) | 正しい定義の末尾に、空白を足して 1,048,576 バイト / 1,048,577 バイト(JSON と `.psd1` の両方。`Slow`) | `validate` | 0 / 64(`definition file is too large`) |
| AC-467 | JSON の文字列のエスケープ | `"command": "TJ --out O echo \"a b\" c A 😀"`(`TJ` は 5 つの語)、不正なエスケープ `"\x"`、閉じていない文字列、制御文字(生の LF)を含む文字列 | `run a`(正しいもの)、`validate`(誤り) | 正しいもの: `O` の引数 = `a b`、`c`、`A`、`😀`(サロゲートペアのエスケープが 1 つのコードポイントになる)。誤りは 64(`syntax error`)。孤立したサロゲートのエスケープ(`"\ud800"`)の扱いは 03 章に従う。受理する場合でも、出力で U+FFFD に化けず、エスケープされる(AC-519) |
| AC-468 | JSON の数値の型 | `"format_version": 1.0`、`1e0`、`"warning_exit_codes": [3.0]`、`[1e1]`、`[99999999999999999999]`、`[true]`、`[NaN]` | `validate --config conductor.json` | `format_version` は 64(`unsupported format_version`)。終了コードは 64(`wrong value type`)。整数のリテラルだけを整数と読む(`[double]` を経由して丸めた値を見ない。字句の形(小数点・指数の有無)を検査する。AC-502) |
| AC-469 | 変換の JSON の出力の文字 | `.psd1`: `a = '<&>'`⏎`b = '夜間'`⏎`c = ' x'` | `convert in.psd1 --to json` | 0。OUT に `"<&>"` がそのまま(`\u0026` や `&` にならない。`ConvertTo-Json` は `<`・`>`・`&`・`'` をエスケープする場合があるため使わない。09 章)と `夜間`(非 ASCII はそのまま)がある。OUT を JSON として読んだ木が入力の木と等しい(U+2028 は、`\u2028` の書き方でも直接でもよい。09 章に従う) |
| AC-470 | 変換の `.psd1` の出力: 予約語・数に見える文字列 | JSON: `{"a": "true", "b": "1", "c": "null", "d": "@end", "e": "", "f": "~", "g": "$true", "h": "1e3", "i": "0x1F", "j": "yes", "k": "-", "l": "a: b", "m": "# c", "n": "$(x)", "o": "it's", "p": "@{}"}` | `convert in.json --to psd1` の OUT を `convert --to json` に通す。さらに OUT を `Import-PowerShellDataFile`(テストの中だけ)で読む | 往復の木が入力と等しい(すべて文字列のまま)。OUT で、値が**単一引用符**で書かれる(`$` や `#` を含む値が、変数や式に化けない)。`it's` は `'it''s'`。`Import-PowerShellDataFile` が読んだ木も等しい(PowerShell 自身が、書き出した `.psd1` を同じ値で読める)。区分: 自動 |
| AC-471 | 変換の `.psd1` の出力: キーと特殊な文字列 | JSON: 複数行の文字列(`"a\nb\n"`)、先頭・末尾に空白のある文字列、タブ、`"\u0007"`(制御文字)、非 ASCII、絵文字、長い文字列(1 万文字)、空のキー `""`、キー `a-b`・`1`・`$x`・`a b`・`@end`・`if`・`true`・`null`・`日本語`・`'q'` | `convert in.json --to psd1` の OUT を `convert --to json` に通す | 往復の木が入力と等しい。裸の単語で書けるキー(`^[A-Za-z_][A-Za-z0-9_]*$`)以外は、単一引用符で書く(`if`・`true`・`null` も、予約語として引用符つきにして、PowerShell の予約語・変数と取り違えない。予約語の一覧は 09 章 8.3 節) |
| AC-472 | 変換の JSON の入力の数値 | JSON: `{"a": 1, "b": 1.0, "c": 1e2, "d": -0, "e": 9223372036854775807, "f": -9223372036854775808}` | `convert in.json --to psd1` の OUT を `convert --to json` に通す | 型が変わらない(`a`・`d`・`e`・`f` は整数、`b`・`c` は浮動小数)。往復の木が入力と等しい。`e`・`f` は桁を落とさない。`.psd1` に出る `c`(`100.0` か `1e2` か)は、PowerShell のパーサーが `[double]` として読む書き方で、`-0` は整数の `0` になる(09 章。AC-544) |
| AC-473 | ステートの JSON 固有の異常 | ステートが、重複するキーを持つ JSON、値のあとに余分なデータ、最上位が配列 / `null` / 文字列、`{}` | `run a` | 65(ジョブは起動されない)。余分なデータ・最上位の型の誤りは `state file is corrupt: JSON として読めません`、キーの重複は `state file is corrupt: キーが重複しています`、`{}` は `state file is corrupt: format_version がありません` |
| AC-474 | ステートの数値の型 | `exit_code` が `3.0` / `"3"` / `1e1` / `256` / `-1` / `true` / `NaN`、`format_version` が `1e0` | `run a` | 65。`state file is corrupt: …`(型・範囲の理由は 06 章) |
| AC-475 | `ME` の誤りの行(`.psd1` と JSON) | `ME`(4.1 節。`.psd1` の `conductor.psd1` と、JSON の `conductor.json`) | `validate`、`validate --config conductor.json` | 64。ERR は 4 行。`.psd1` は順に `…:3: unknown key`、`…:13: wrong value type`、`…:18: unknown key`、`…:22: transition target job is not defined`。JSON は順に `…:3:`、`…:11:`、`…:16:`、`…:20:`(同じ種類の文)。位置は、`.psd1` は構文木の各キーと値の `Extent` の `StartLineNumber`、JSON は自前の読み取りが、各キーと値の開始の位置(行)を記録して求める(LF の数で数える。CRLF でも同じ行番号。AC-038 と同様。[03](03-definition-format.md)) |
| AC-476 | 版の表示と、版の唯一の出所 | ソース、`src/Conductor/Conductor.psd1`、ランチャー | 点検と `--version` | 版の唯一の出所は `Conductor.psd1` の `ModuleVersion`(例: `ModuleVersion = '0.8.0'`)。コードは、版の定数を持たず、`Conductor.psm1` の読み込み時に 1 回だけ、`(Import-PowerShellDataFile -LiteralPath <Conductor.psd1>).ModuleVersion` で読み、`$script:ConductorVersion` に保つ(`Get-ConductorVersion` が返す。02 章 10 章、04 章 8 章)。`Import-PowerShellDataFile` がマニフェストを読めない場合に限り、定数 `'0.8.0'` に切り替え、`ModuleVersion` と一致することを `Invoke-Checks version` と単体テストで検査する(04 章の試験 T6。**未確認**)。`--version` の OUT = `conductor ` + `ModuleVersion`(例 `conductor 0.8.0`)。ランチャー・テスト・README に、版の数字の直書きがない(README の記載は、5.4 節の `consistency` が `ModuleVersion` と比べる)。`Invoke-Checks version` が、`ModuleVersion` を、タグ(`v` なし)と比べる(5.4 節)。区分: 点検 |
| AC-477 | PowerShell の版の記載の一致 | `build/Versions.psd1`、`Conductor.psd1`、`README.md`、`.github/workflows/ci.yml`、ソースの `#requires` | `./build/Invoke-Checks.ps1 -Check consistency` | `Conductor.psd1` の `PowerShellVersion` が `build/Versions.psd1` の `MinPowerShell` と一致する。`MinPowerShell` ≦ `MaxPowerShell`(同じでもよい)。すべての `.ps1`・`.psm1` の `#requires -Version` が `MinPowerShell` 以上。README の「PowerShell <MinPowerShell> 以上」が一致する。ワークフローに、版の数字の直書きがない(`build/Versions.psd1` を読む)。不一致なら失敗(終了 1)。区分: 点検 |
| AC-478 | 開発時の依存の整合 | `build/Dependencies.psd1`、`PSScriptAnalyzerSettings.psd1`、ワークフロー | `./build/Invoke-Checks.ps1 -Check deps` | 製品に実行時の依存がない(AC-443)。開発時の依存が、`build/Dependencies.psd1` の許可の一覧(Pester、PSScriptAnalyzer)の内。各依存に、版(完全な版)と、パッケージのハッシュ(SHA-256)がある。ワークフローが、`build/Install-DevDependencies.ps1` だけで入れる(`Install-Module` の直書きがない)。区分: 点検 |
| AC-479 | モジュールの読み込みとパッケージの作成ができる | ソース | `Test-ModuleManifest src/Conductor/Conductor.psd1`、`Import-Module ./src/Conductor -Force`(新しい `pwsh` で)、`./build/New-Package.ps1` | すべて成功し、`Get-Command -Module Conductor` が `Invoke-Conductor` だけを返す(`FunctionsToExport`)。パッケージの中身が AC-440 のとおり。`PS_MIN` と `PS_MAX` の両方の `pwsh` で通る。区分: 点検 |
| AC-480 | 静的検査 | ソース | `./build/Invoke-Checks.ps1 -Check lint` | 終了 0(PSScriptAnalyzer の Error・Warning が 0、禁止コマンドが 0、`Invoke-Formatter` の差分が 0、`Test-ModuleManifest` が成功)。区分: 点検 |
| AC-481 | 開発時の依存の検査 | 依存 | `./build/Invoke-Checks.ps1 -Check deps-audit` | 固定した版が、PowerShell Gallery の最新と、どれだけ離れているかを記録する。固定の版のパッケージが、記録したハッシュと一致するかも検査する(一致しなければ失敗)。最新との差は、警告(マージはブロックしない)。区分: 点検 |
| AC-482 | 変換した定義がそのまま使える | `CV1` を `conductor.json` に | `convert conductor.json --to psd1 > conductor.psd1`、`validate`、逆向きに `convert conductor.psd1 --to json > c2.json`、`validate --config c2.json` | すべて 0。展開された木が同じ |
| AC-483 | 厳格な設定と、解析の規則 | `PSScriptAnalyzerSettings.psd1`、モジュールの先頭 | 点検(`Import-PowerShellDataFile` と構文木) | 設定に、`Severity = @('Error', 'Warning')`、有効にする規則(9.2 節の一覧)と `Enable = $true`、`PSUseCompatibleSyntax` の `TargetVersions`(`MinPowerShell` と `MaxPowerShell`)、`ExcludeRules` が理由つき(コメント)で空に近い。`Conductor.psm1` の先頭が `Set-StrictMode -Version Latest` と `$ErrorActionPreference = 'Stop'`。すべての製品の `.ps1` が、`#requires -Version`。区分: 点検 |
| AC-484 | 合図とジョブの終了の順序と、シグナルの登録の解除 | `Runner` と `Cli` の単体(偽の `Deps` と、本物の補助の部品の登録。UNIX 系) | 合図を、ジョブの終了の直前・直後・待機(リトライ)の直前に立てる。同じシグナルを続けて 2 回立てる。`run` / `pipeline` が終わったあと、登録が解除されていることを調べる | 最初の番号だけが残る。転送は 1 回。終了済みのジョブには送らない。`run` / `pipeline` の終了後(正常・失敗・例外のいずれでも)に、登録した補助の部品のシグナルの登録がすべて解除されている(`try`/`finally`。登録が漏れて、後続のテストに影響しない。偽の `FakeSignals` が、登録の数と解除の数を数える)。区分: 単体 |
| AC-485 | 偽の `FakeSignals` で登録に失敗すると 70 | 偽の `FakeSignals`(登録が失敗を返す) | `run a`、`pipeline p`(定義は正しい / 誤りがある) | どちらも 70。ERR = `conductor: error: failed to set up signal handling`。定義の検査(定義が誤りでも 64 にならない)もジョブの起動もしない。区分: 注入。(AC-298 を、定義が誤りの場合まで広げたもの) |
| AC-486 | Windows の `.cmd` / `.bat` の禁止文字は起動しない | Windows。`T/echo-args.cmd`(内容 `@echo off` と `exit /b 0`)。`cmd` = そのパス + 引数 1 つ。引数に `"`、`%`、`^`、`&`、`\|`、`<`、`>`、`!`、`(`、`)`、改行のどれか 1 つを含める([07](07-process-and-signals.md)。1 文字ずつ別の定義)。配列形式でも同じ | `run a` | 127(起動失敗)。ERR = `conductor: error: job 'a': cannot start command '…'`。`T/injected.txt` ができない。含まない引数(`plain`)は起動される(AC-115)。区分: 自動(Windows の CI) |
| AC-487 | Windows の Ctrl+C のチェックリストの記録 | AC-301 の記録欄 | リリース前に、Windows の実機で AC-301 を行い、結果を `tests/WINDOWS_CTRL_C_CHECKLIST.md` の記録欄に書く | 記録欄に、実施日、OS の版、PowerShell の版、(1)〜(4)の結果が書かれている。ない場合はリリースの前に実施する。区分: 手動(Windows) |
| AC-576 | `.psd1` の位置: 行と列 | `.psd1` を LF / CRLF で。キーと値が別の行。複数行の here-string・ブロックコメント(`<# … #>`)のあとの誤り。タブを含む行 | `validate`(誤りを 1 つ含む定義) | 行番号が、LF / CRLF で同じ。ブロックコメント・here-string の中の改行を数える(`StartLineNumber` は 1 始まり)。列(`StartColumnNumber`)は、表示に使わない(03 章が使う場合を除く) |
| AC-577 | `.psd1` の構文の誤り(パーサーのエラー) | `@{ a = }`、`@{ a = 'x }`(閉じない引用符)、`@{ a = @( }`、`@{ a = 1 b = 2 }`(区切りなし)、`}`、`@{ = 1 }` | `validate` | 64。`syntax error`。ERR にパーサーのエラーの文(`ParseError.Message`)が現れない。行は、最初の `ParseError` の `Extent.StartLineNumber`。構文の誤りが複数あっても、行の順に 1 行ずつ(または最初の 1 つだけ。03 章に従う) |
| AC-578 | `.psd1` の文字列の細部 | 単一引用符の `''`(1 つの `'`)、`'a''b'`、`'`(単独。誤り)、二重引用符の中の `` `n ``・`` `t ``・`` `" ``・`""`(`$` なし)、`'ａ'`(全角)、`'a'b'`、`'a' 'b'`(2 つの式)、U+2018/U+2019(「スマート」な単一引用符。PowerShell は、これを単一引用符として扱う場合がある) | `validate`(値が `command` などになる定義) | 03 章の規則どおり。**PowerShell のパーサーは、U+2018・U+2019 を単一引用符と同じに扱う(記憶による。未確認)ので、`'it’s'` のような入力の意味が、見た目と変わりうる**。実装は、構文木の値(`Value`)でなく、字句の書き方を 03 章の表で検査し、扱いを 03 章に従う。二重引用符の中のバッククォートのエスケープは、03 章が受理する場合だけ受理する |
| AC-579 | `.psd1` の整数の字句 | `format_version = 1`、`0x1`、`01`、`1L`、`1kb`、`1.0`、`1e0`、`1d`、`-1`、`+1`、`0b1`、`1_0`、`' 1'`、`$true`、`1/1`、`(1)` | `validate` | `1`・`0x1`・`0b1`(値が 1)は受理。`01`・`1L`・`1kb`・`1d`・`+1`・`1_0`・`1/1`・`(1)` は構文(64。`syntax error`。理由 `.psd1 で使えない書き方です`)。`1.0`・`1e0`・`' 1'`・`$true` は 64(`wrong value type`)、`-1` は 64(`unsupported format_version`)。**構文木の定数の値(`.Value`)は、`1L` を `[long]`、`1kb` を `1024`、`1d` を `[double]`、`0b1` を 1 にするので、`.Value` に頼らず、`Extent.Text` の字句を検査する**。AC-502 と同じ型の検査 |
| AC-580 | `.psd1` のキーの書き方と未知のキー | 未知のキー `Colour = 'x'`、`'colour' = 'x'`、`"colour" = 'x'`、既知のキーの大文字小文字違い `ID = 'a'`・`Command = 'x'`・`Format_Version = 1` | `validate` | 未知のキーは 64(`unknown key`)。**既知のキーの大文字小文字違い(`ID`)は、`unknown key`**(キー名は大文字小文字を区別して照合する。PowerShell のハッシュテーブルの照合(区別しない)に頼らない。03 章に従う。ただし重複の判定(AC-456)は区別しない) |
| AC-581 | `.psd1` の配列の要素の型 | `warning_exit_codes = @(3, 'x')`、`@(3, $null)`、`@(@(3))`、`@(3.5)`、`@(3, 4,)`(末尾のカンマ)、`@(,3)`、`@(3 4)` | `validate` | 64。`wrong value type`(または `syntax error`。03 章に従う)。要素ごとに 1 行 |
| AC-582 | `.psd1` から JSON への変換での `$null` と空 | `@{ a = $null; b = @(); c = @{}; d = '' ; e = @($null) }` | `convert in.psd1 --to json` | `"a": null`、`"b": []`、`"c": {}`、`"d": ""`、`"e": [null]`。`$null` を含む配列が、空の配列に化けない(PowerShell は、パイプラインで `$null` を捨てる場合がある。AC-555) |
| AC-583 | JSON から `.psd1` への変換で、入れ子の深さが 32 のとき | JSON の深さ 32 の入れ子 | `convert in.json --to psd1` | 0。出力の `.psd1` を `convert --to json` に通すと、深さ 32 で受理される(書き出した `.psd1` の括弧の数が、読み取りの深さの規則と一致する) |
| AC-584 | `.psd1` のインデントと改行 | `convert in.json --to psd1` の出力 | (バイト列を調べる) | 改行は LF だけ(CRLF にならない。Windows でも)。タブを含まない。BOM がない。末尾は LF 1 つ。行末に空白がない。インデントの幅は 4 スペース(09 章 8.1 節。タブを使わない) |
| AC-585 | PowerShell 自身が読めること | `convert` が出力した `.psd1`(CV1P と、AC-470・AC-471 の入力) | テストの中で `Import-PowerShellDataFile`、`Test-ModuleManifest` ではなく単独で `[System.Management.Automation.Language.Parser]::ParseInput` で構文の誤りが 0 | 誤りが 0。`Import-PowerShellDataFile` が例外なく読み、キーの数と値が入力と等しい(配列の 1 要素が単一化しない)。区分: 自動 |

### 4.20 PowerShell に固有の項目

PowerShell の実行モデル(`pwsh` の起動、モジュールの読み込み、パイプラインへの出力、`-File` の終了コード、実行空間(runspace)とシグナルのコールバック、`Add-Type`、型の自動変換、大文字小文字を区別しない比較、配列の単一化、`ErrorActionPreference`、実行ポリシーと Zone.Identifier、Constrained Language Mode、`Set-Location` と .NET の現在のディレクトリのずれ)に固有の落とし穴を確かめる。注入のための実行ファイル(`tests/jobs/inject.ps1`)は、モジュールの内側の入口を呼ぶ(3.4 節、3.6 節)。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-488 | 起動時間 | ランチャーの配布物(展開したもの) | `--version` を 20 回(起動から終了まで。最初の 5 回を捨てる)。比較のため `pwsh -NoProfile -NonInteractive -Command exit` を 20 回。`Import-Module` だけを新しい `pwsh` で 20 回 | ランチャーの `--version`: p95 が 1,500 ms(Linux・macOS)、3,000 ms(Windows)以下(仮。目安。2.1 節)。`pwsh` だけの起動を引いた値: 1,000 ms / 2,000 ms 以下(仮)。`Import-Module`: 800 ms 以下(仮)。すべて未測定。結合したモジュール(関数を 1 つの `psm1` にしたもの)との差は、未確認(測る場合は、記録に構成を明記する)。区分: 計測 |
| AC-489 | `Add-Type` を行わない経路 | ランチャー、または新しい `pwsh` の中の単体(プロセス内) | (単体)`--version`・`--help`・使い方の誤り・`validate`・`reset`・`watch`・`convert` を `Invoke-Conductor` で呼んだあと、`[System.AppDomain]::CurrentDomain.GetAssemblies()` に、補助の部品の型(アセンブリ)がない。`run` で 1 つ目のジョブを起動する直前までに、1 回だけ用意される | 補助の部品が読み込まれていない(`run`・`pipeline` がジョブを起動する直前にだけ読み込まれる)。`--version` が `Add-Type` の遅さに影響されない。単体のテストは、新しいプロセス(`tests/jobs/inject.ps1` の `assemblies` モード)で行う(同じテストのプロセスの中では、他のテストが先に読み込んでいるため)。区分: 単体と計測 |
| AC-490 | 未捕捉の例外は 70 | `P(p; a:0{cmd="TJ sleep 1"})`。`tests/jobs/inject.ps1 throw run a` で起動する。`throw` モードは、`Deps` の一部(例: ステートの読み込み)が `throw 'SECRET-INJECT'` と、.NET の例外(`[System.InvalidOperationException]::new('SECRET-INJECT')`)を投げる偽の `Deps` を作って、モジュールの内側の入口を呼ぶ | `run a` | 終了 70。ERR = `conductor: error: internal error`(1 行だけ。末尾の改行 1 つ)。ERR に `SECRET-INJECT`・`At line:`・`Exception:`・`+ CategoryInfo`・スタックトレースが現れない。OUT 空。`$ErrorActionPreference = 'Stop'` のもとで、非終端エラー(`Write-Error` 相当)も終端エラーに変わって、同じく 70 になる(別のモード `nonterminating`)。区分: 注入(実プロセス) |
| AC-491 | `PipelineStoppedException` と、別スレッドの例外 | (a) `tests/jobs/inject.ps1 stop`: モジュールの内側の入口の中で、`$PSCmdlet.ThrowTerminatingError` 相当や、`[System.Management.Automation.PipelineStoppedException]::new()` が投げられた状況(`Stop-Job`・`Ctrl+C` の模擬)。(b) 偽の `FakeSignals`(合図のコールバックが例外を投げる)と `run` / `pipeline`(注入) | (a) `inject.ps1 stop run a`。(b) 偽の部品の上で `run a` / `pipeline p` | (a) 70、または、`finally` の後始末(シグナルの登録の解除、ステートの一時ファイルの削除、標準出力の flush)が実行されてから、`pwsh` が終了する。**`PipelineStoppedException` が `catch [System.Exception]` で捕まるか、捕まらずにスクリプトが止まるかは、公式の文書で確認できていない(未確認)**。したがって、後始末を `finally` に置くことを必須とし、終了コードは、止まった場合の値(`pwsh -File` の 0 または 1)に頼らず、入口スクリプトの `finally` が、ヘルパーが予約した値(なければ 130)で `[System.Environment]::Exit` を呼んで決める(07 章 7.7 節。`Invoke-Conductor` が例外を漏らした通常の例外の経路は 70。AC-492)。(b) 70。ERR = `conductor: error: internal error`。ジョブのあとのステートの書き込みを、例外のために飛ばさない(例外が起きる前の確定したステートは残る)。区分: (a) 自動、(b) 注入 |
| AC-492 | ランチャーの終了コードの伝播 | ランチャー(`bin/conductor`、`bin/conductor.cmd`) | `Invoke-Conductor` が、0、1、64、65、66、67、68、70、127、128、143、255 を返す状況(入口スクリプトは、これらを加工せず `exit` に渡す。127 と 128 などは、ジョブの結果の値として返りうる)(偽の入口。`CONDUCTOR_TEST_*` のような環境変数を製品に作らない。`tests/jobs/inject.ps1` が、同じ形のランチャーのスクリプトを写して試す。または、実際の入口で作れるもの(`--version` の 0、使い方の誤りの 68、定義の誤りの 64、ジョブの 7、127、シグナルの 143)で確かめる) | ランチャーの終了コードが、`Invoke-Conductor` の戻り値と同じ(`pwsh -File` が `exit N` の N をそのまま返す。確認: 1 章)。`Invoke-Conductor` が例外を漏らす・値を返さない・複数の値を返す(AC-552)・`[int]` でない値を返す場合は、ランチャーが `conductor: error: internal error` を標準エラーに 1 行出して 70 にする(`pwsh -File` の 1 にしない)。`pwsh` が見つからないとき(`PATH` から `pwsh` を除いて起動する)は、`bin/conductor`・`bin/conductor.cmd` が、固定の英語の 1 行 `conductor: error: pwsh (PowerShell 7) was not found in PATH` を標準エラーに出して終了コード 70 にする([04](04-cli.md) 1.3 節、[08](08-output-and-i18n.md) 4.1 節、[10](10-errors-and-exit-codes.md) 1 章。127 はジョブの起動失敗の終了コードで、ランチャーは返さない)。区分: 自動と単体 |
| AC-493 | 標準出力のパイプが閉じた(UNIX 系と Windows の場合分け) | 入力 `big.json`(約 1 MiB。小さな要素の配列。`--to psd1` の出力が 64 KiB を大きく超える。`Slow`) | `convert big.json --to psd1` を起動し、標準出力の最初のデータを受けた時点で、読み手側(`$p.StandardOutput.Close()`)を閉じる | **UNIX 系**: 終了 0 になりうる(閉じたパイプへの書き込み `EPIPE` を、.NET の標準のストリームが成功のふりで捨てる場合がある。08 章 1.3.1。既知の差で、受け入れる)。終了 67 でもよい(`IOException` になる場合)。**どちらでも**、ERR に例外・スタックトレース・エラーのレコードが現れない、70 にならない、10 秒以内に終わる。67 のときは ERR に `conductor: error: cannot write to standard output`。**Windows**: 終了 67(`IOException` になると想定。実機は未確認)。`SIGPIPE` の扱いは 08 章 |
| AC-494 | 標準出力の取りこぼしがない | (a) `convert big.json --to psd1`(出力が 64 KiB を超える)。(b) `P(p; a:0{cmd="TJ big 16"})` | (a) 標準出力をパイプにし、読み手が最初の 500 ms 読まない。(b) `run a`(ジョブが 16 MiB を、`conductor` の標準出力に継承されたパイプへ書く) | (a) 終了 0。OUT の全体が、同じ変換を標準出力をファイルにして得た結果と、バイト単位で等しい(終了前に `Flush()` して書き込みの完了を待つ)。(b) 終了 0。OUT のバイト数がちょうど 16 MiB(`x` のみ)。`conductor` が出力を溜め込まない(子の標準入出力を `Redirect* = $false` のままにして継承する。PowerShell のパイプラインに出力を通さない) |
| AC-495 | 標準出力の書き込みの失敗(`/dev/full`。Linux) | Linux。標準出力を `/dev/full` を開いたファイルにして `watch`(ステートあり)、同様に `convert …`(`sh -c 'exec <ランチャー> … > /dev/full'` の形で起動する。テストの補助だけがシェルを使う) | 実行 | `watch` は 65、`convert` は 67。ERR = `conductor: error: cannot write to standard output`。70 にならない。終了時の flush が再び失敗して、終了コードが変わったり、例外が出たりしない(AC-496)。区分: Linux |
| AC-496 | 終了時の flush の失敗で終了コードが変わらない | UNIX 系。標準出力を `Process` で起動して、起動の直後に読み手を閉じる | `--version`、`--help`、`watch`(ステートあり)、`validate -v`(標準エラーが閉じた場合も) | 終了コードが、`--version`・`--help` は 0(08 章に従う)、`watch` は 0 または 65(08 章 1.3.1 の既知の差)。ERR に例外・エラーのレコードが現れない。標準出力の `Stream` を閉じる処理も `try`/`catch`。**`pwsh` 自身が、終了時に標準出力を flush して失敗する場合の挙動は未確認**。区分: UNIX 系 |
| AC-497 | 終了後にプロセスが残らない | 各サブコマンド(`validate`、`run`、`pipeline`、`watch`、`reset`、`convert`、`--help`、`--version`、使い方の誤り) | 標準入力を開いたまま(閉じない)実行 | 出力の完了から 1 秒以内にプロセスが終わる(標準入力の読み取り(`Read-Host`・`[Console]::In`)、前景スレッド、シグナルの登録、`PSReadLine` の初期化が残ってハングしない。ランチャーは `-NonInteractive` を渡す)。補助の部品が作るスレッドは、背景スレッドにする。区分: 自動 |
| AC-498 | 静的検査の指摘が 0 件、実行時に PowerShell の表示が出ない | 4.1〜4.9 節の各種の実行(特に、`.psd1` の誤り・`convert`・リトライの待機・ジョブの起動失敗) | `Invoke-ScriptAnalyzer -Path src -Recurse -Settings PSScriptAnalyzerSettings.psd1 -Severity Error,Warning`。実行は、通常の実行 | PSScriptAnalyzer の指摘(ParseError を含む)が 0。ERR に `WARNING:`・`VERBOSE:`・`DEBUG:`・`At line:`・`+ CategoryInfo`・`Exception:`・`Unhandled exception` を含む行が現れない。終了コードと出力が、通常の実行と同じ。区分: 自動と点検 |
| AC-499 | シグナルの登録の点検 | ソース | 点検(構文木と、C# のソースの文字列の走査) | シグナルの登録(`PosixSignalRegistration.Create`、`Console.CancelKeyPress`)が、補助の部品の C# のソース(`Add-Type` に渡す定数の文字列 1 つ)の中にしかない。PowerShell のコードに、`Register-ObjectEvent`・`Register-EngineEvent`・`[Console]::add_CancelKeyPress`・`Register-ArgumentCompleter` の使用がなく、**スクリプトブロックを .NET のデリゲート(イベントのハンドラ・コールバック)に渡す式がない**(`[System.Action]{ ... }`・`.add_*({ ... })`・`[scriptblock]` のキャスト。別のスレッドで呼ばれると実行空間がなく失敗する。AC-538)。登録するシグナルが、UNIX 系は `SIGHUP`・`SIGINT`・`SIGQUIT`・`SIGTERM` の 4 つ、Windows は Ctrl+C だけ(07 章)。登録の戻り値を、静的なフィールドに保持している(AC-537)。コールバックの本体が、旗の書き込み(`Volatile.Write`・`Interlocked.*`)だけで、入出力・`lock`・`throw` を含まない。区分: 点検 |
| AC-500 | 終了の呼び出しの点検 | ソース | 点検(構文木の走査) | `exit` 文(`ExitStatementAst`)と `[System.Environment]::Exit`・`[System.Environment]::FailFast`・`Stop-Process -Id $PID` の呼び出しが、入口スクリプト `bin/conductor.ps1` の中にだけある(通常の終了の `exit` が 1 か所、停止経路の `[System.Environment]::Exit` が 1 か所。`FailFast`・`Stop-Process -Id $PID` は 0 か所)。`src/Conductor/` にはない。`Invoke-Conductor` は、終了コードを**返す**(`[int]`)。標準出力・標準エラーの `Flush` が、その前に済んでいる(`exit` は、呼び出し元がモジュールの中なら対話セッションごと終了させるため、ここでは使わない)。区分: 点検 |
| AC-501 | 64 ビット符号なし整数の飽和演算 | `Flow` の単体 | 待ち時間を求める関数に、検査を通らない極端な値を直接渡す(`prev = [uint64]::Parse('9223372036854775808')`、`prev = [uint64]::MaxValue`、`multiplier = 10`、`cap = [uint64]::MaxValue`)。`max` = 100、`interval` = 86400、`multiplier` = 10、`max_interval` = 86400 の列 | 結果が `[uint64]` で、`[uint64]::MaxValue` を超えず(`[bigint]` で計算して `[bigint][uint64]::MaxValue` と比べ、超えたら `[uint64]::MaxValue` に飽和)、折り返し(0 に戻る)がなく、例外も `[double]` への昇格(`[uint64]` どうしの乗算が溢れたとき、PowerShell が `[double]` に昇格する場合がある。**挙動は未確認**)も起きない。`[double]` に直して比べない。列は 86400 のまま 100 回続く。待ちの秒数を、待ちの 1 区間の長さ(`[int]` のミリ秒)に直す関数は、AC-503 の範囲に収まる値だけを返す(`[int]` へのキャストの例外・丸めで、負の値や 0 に化けない)。区分: 単体 |
| AC-502 | 整数の検査: 真偽値・小数・指数・`NaN`・巨大な整数 | 定義(`.psd1`・JSON): `format_version = $true`、`warning_exit_codes = @($true)`、`@(1.0)`、`@(1e0)`、`max = $true`、`max = 3.0`、`@(1L)`、`@(1kb)`、`@(99999999999999999999)`、`max = 4294967296`、桁が 10,000 の整数(`format_version = 1000…0`)、`9007199254740993`(`[double]` に直すと `9007199254740992` に化ける整数。2^53 + 1)、JSON の `[NaN]`・`[Infinity]`、ステートの `exit_code: true`・`1.0`・`NaN`・`99999999999999999999` | `validate` / `run a` | すべて「範囲外・型の誤り」として拒否する(定義: 64、ステート: 65)。真偽値を `1` として、`1.0` を `1` として、`Infinity` を整数として受理しない(字句の形を、整数と、それ以外に分類してから、`[bigint]` で範囲を検査し、`[long]`・`[int]` に直す)。`[double]` を経由しない(`9007199254740993` は、`[long]` の範囲内の整数として、そのまま保たれる)。`.Value` の型(`[int]`・`[long]`・`[decimal]`・`[double]`)に頼らない(AC-579)。`[int]` へのキャストの例外(`InvalidCastException` 相当。PowerShell では `RuntimeException`)が漏れて 70 にならない。桁が多い入力で、ハングしない・異常終了しない(`[bigint]::Parse` に無制限の桁を渡さない。字句の長さの上限(ファイルの大きさの上限 1 MiB の範囲内)を超えないことで足りる。実装時に確認)。区分: 自動と単体 |
| AC-503 | 待ちの 1 区間の上限 | `Runner` の待ちの部品の単体(`WaitHandle.WaitOne`・`Process.WaitForExit(int)` を、渡された時間を記録する関数に差し替える。時計も差し替える。合図で打ち切る) | 待機を 24 時間(86,400 秒)、`max_interval` の上限で呼ぶ。続けて、検査を通らない 30 日の待機を直接呼ぶ。さらに `[uint64]1 -shl 40` 秒と、`[uint64]::MaxValue` 秒 | 下の関数に渡される時間(`[int]` のミリ秒)が、いつも 250 ms 以下(05 章・07 章の決定: リトライの待機は、250 ms 以下の刻みの待ちと、単調な時計(`[System.Diagnostics.Stopwatch]::GetTimestamp()`)の期限の確認で行い、残り時間は `[bigint]` で求める。ジョブの終了の待機は、10 ms 以下の `WaitForExit(int)`)で、`[int]::MaxValue` を超えない。`ArgumentOutOfRangeException` が起きない。**`-1`(= 無期限)を、計算の誤りで渡さない**(`[int]` のキャストで `-1` に化ける値がない)。`Start-Sleep` の `-Seconds` に巨大な値を渡さない。30 日の待機は、区間に分けて待つ(合図で打ち切って旗を検査する)。区分: 単体 |
| AC-504 | `Process.ExitCode` とシグナル番号の対応 | `Runner` の単体 | `ExitCode`(`137`、`143`、`130`、`129`、`131`、`128`、`255`、`256`、`0`、`3`、`-9`、`-1`、`[int]::MinValue`)と、「シグナルで終了した」かの判定(UNIX 系の子の終了の読み取り。07 章)を、シグナルで終了と終了コードに分ける関数 | 07 章の決定どおりに、UNIX 系では、`ExitCode` が 129〜159 で、ジョブの成功・警告のコードに含まれないときだけ「シグナルで終了 `ExitCode - 128`」とみなし(`ExitCode = $null`、`TerminatedBySignal = n`)、それ以外は終了コードとして扱う。例: `137`・`143`・`130`・`129`・`131` はシグナル(9・15・2・1・3)、`128`・`160`・`255`・`256`・`0`・`3`・`-9`・`-1`・`[int]::MinValue` は終了コード。成功・警告のコードに `137` を含めたジョブでは、`137` は終了コードのまま。子が自分で `exit 137` した場合との区別がつかない(この版の挙動。07 章)。UNIX 系でシグナル終了が `128 + n` で返ることは、ランタイムのソースの記述に基づく(PowerShell 7.6 の .NET 10 で同じかは未確認。AC-140 が実機の確認を兼ねる)。負の値・範囲外の番号でも、落ちない。Windows の `ExitCode` は、シグナルとして解釈しない(32 ビット符号つきのまま)。区分: 単体 |
| AC-505 | シグナルの送信で、ジョブの終わりを待たずに終わらない | `SL` の `sleeper` を `TJ --out O trap 500 3` にする(UNIX 系) | `run sleeper` を起動し、`ready` のあと SIGTERM | 終了 143。`O` に `trapped`、続いて `done` が記録されており、`conductor` が、ジョブの終了(`WaitForExit` が終了を返すこと)を待ってから終わる(シグナルの転送が戻った時点で終わらない)。`conductor` の終了は、SIGTERM の送信から 500 ms 以上あと。ステート `sleeper=error`(終了コード 3 は、中断の分類により `128 + 15` が採られる)。区分: UNIX 系 |
| AC-506 | ジョブが残した孫のプロセスを待たない | `argv=["sh", "-c", "sleep 5 & exit 3"]`(UNIX 系) | `run a`(標準出力・標準エラーを、パイプではなく一時ファイルに向けて起動する。終了の時刻は `WaitForExit` で測る) | 終了コード 3。`conductor` が、ジョブ(`sh`)の終了から 2 秒以内に終わる(孫の終了を待たない。子の標準入出力を `Redirect* = $true` にして、非同期の読み取りで EOF を待つと、孫を待ってしまうので、`conductor` は、子の標準入出力を引き継ぐ。引数なしの `WaitForExit()` は、リダイレクトした出力の EOF を待つ場合があるので、使うなら `WaitForExit(int)`) |
| AC-507 | ランチャー経由と、モジュールの直接の呼び出しが同じ | ランチャーと、`pwsh -NoProfile -Command "Import-Module ./src/Conductor; exit (Invoke-Conductor -ArgumentList $rawArgs)"`(`$rawArgs` は同じ引数の `[string[]]`) | 両方で、`--version`、`--help`、`validate`(正しい定義・誤りの定義)、`run`、`watch`、使い方の誤りを実行 | 終了コード・OUT・ERR が同じ。`-Command` の経路でも、引数が壊れない(`-ArgumentList` の配列の渡し方は、実装時に確かめる。**未確認**)。製品の中で、自分自身を再び `pwsh` で起動しない(子の起動に `[System.Diagnostics.Process]::GetCurrentProcess().MainModule.FileName`(`pwsh` を指す)を、ジョブの実行ファイルとして使わない)。区分: 自動(3 OS) |
| AC-508 | マニフェストとファイルの形 | `Conductor.psd1`、`Conductor.psm1`、`PSScriptAnalyzerSettings.psd1`、`bin/` | 点検(`Import-PowerShellDataFile` と構文木) | `Conductor.psd1` に、`RootModule = 'Conductor.psm1'`、`ModuleVersion`、`GUID`、`Author`、`CompanyName`、`Copyright`、`Description`、`PowerShellVersion`、`CompatiblePSEditions = @('Core')`、`FunctionsToExport = @('Invoke-Conductor')`、`CmdletsToExport = @()`、`VariablesToExport = @()`、`AliasesToExport = @()`(ワイルドカードを使わない。`UseToExportFieldsInManifest`)がある。`Conductor.psm1` が `Private/`・`Public/` の全ファイルを読み、`Export-ModuleMember -Function Invoke-Conductor` だけを公開する。`Private/` のサブディレクトリが 02 章の 9 つ(`Definition`・`Convert`・`Json`・`RunId`・`Flow`・`State`・`Runner`・`Report`・`Cli`)。`bin/conductor`・`bin/conductor.cmd` がある。依存の向き(AC-514)。`ModuleVersion` が 3 つの数字。区分: 点検 |
| AC-509 | 標準入力はジョブに引き継ぐ | `P(p; a:0{cmd="TJ --out O stdin"})` | `run a`(標準入力のパイプに `hello⏎` を書いて閉じる) | 終了 0。`O` = `stdin="hello\n"`。`conductor` が標準入力を読まない(`[Console]::In`・`[Console]::OpenStandardInput()`・`Read-Host`・`$input` を読み始めない。AC-497 と合わせて、読み始めると入力を奪うため。`RedirectStandardInput` は `$false` のまま。ランチャーの `pwsh` も、標準入力を、スクリプトとして読まない(`-File` を使う。`-File -` や `-Command -` を使わない)) |
| AC-510 | Windows の `.cmd` / `.bat` を、最小の版でも最新の版でも起動できる | Windows。`T/ok.cmd`(`exit /b 7`) | `PS_MIN` と `PS_MAX` で `run a`(`cmd` = そのパス + ` plain`) | どちらも 7(AC-115)。07 章の決定: `cmd.exe` を直接起動し(`UseShellExecute = $false` のまま。引数の組み立ては 07 章)、禁止文字を含む引数は起動しない(AC-486、AC-513)。この方法が、両方の版で動くことを確かめる(実機で未確認)。.NET は、`.cmd`/`.bat` を直接起動したときの引数の扱いが版により異なる可能性があるが、この文書では公式の文書を確認していない(未確認)。区分: 自動(Windows の CI) |
| AC-511 | ハッシュテーブルのキーの順序(整数に見えるキー) | `.psd1`: `@{ '2' = 'a'; '1' = 'b'; b = 'c' }`。JSON: `{"2": "a", "1": "b", "b": "c"}` | `convert in.psd1 --to json` / `convert in.json --to psd1` | OUT のキーの順が、入力の記述順(`2`、`1`、`b`)。ハッシュテーブル(`@{}`、`[hashtable]`)・`Sort-Object`・`Group-Object` を経由して順序を失わない(PowerShell の `@{}` は列挙順が保証されない)。`[ordered]` を使う場合も、整数に見えるキーを数値の添字として取り違えない(`[ordered]` の `$h[1]` は、キーでなく位置として扱われる場合がある。記憶による。未確認)。区分: 自動 |
| AC-512 | 特別な名前のキーを、普通のキーとして扱う | JSON: `{"__proto__": 1, "Keys": 2, "Count": 3, "Values": 4, "PSBase": 5, "PSObject": 6, "Item": 7, "Equals": 8, "GetType": 9, "ToString": 10}`(変換)。定義に `Keys = 'x'`、`Count = 'x'`、`PSObject = 'x'`(未知のキー) | `convert in.json --to psd1`、`validate` | 変換: 10 個のキーがそのまま出る(往復で木が等しい)。定義: `unknown key`(64)。`$h.Keys`・`$h.Count` が、キーとして保存した値でなくハッシュテーブルのプロパティを返す事故(`$h.Count` は、キー `Count` があればその値を返す場合がある。記憶による)で落ちない。値の取得は `$h['Count']` のインデクサー(またはキーと値の組のリスト)で行う。区分: 自動と単体 |
| AC-513 | Windows の `.cmd` の起動引数の組み立ての単体(どの OS でも動く) | `.cmd` の起動の組み立ての関数(07 章。純粋な関数で、どの OS でも動く)と、Windows・UNIX 系の起動の準備の関数(OS の種別を引数で指定して呼ぶ) | `program` と `args` を与える: (a) `x.cmd` / `X.BAT`(拡張子の大文字小文字を区別しない)と引数 `plain`、(b) 禁止文字(`"`、`%`、`^`、`&`、`\|`、`<`、`>`、`!`、`(`、`)`、`` `n ``、`` `r ``、NUL)を 1 つ含む引数(1 文字ずつ)、(c) `program` が `C:\Program Files (x86)\t\x.cmd`(`(` と `)` は許す)、(d) 拡張子が `.exe` のもの、拡張子なしのもの、(e) 空の引数 | 07 章の決定のとおり。(a) `.cmd` の規則を使う(`cmd.exe` の起動。`UseShellExecute = $false` のまま)。(b) 拒否(起動しない)。(c) 受理。(d) `.cmd` の規則を使わず、そのまま。(e) 空の引数が失われない。UNIX 系の分岐は常に、そのまま。拡張子の判定は、`[System.IO.Path]::GetExtension` に頼らず、末尾の文字列の序数比較(`EndsWith('.cmd', [System.StringComparison]::OrdinalIgnoreCase)`)で行う(AC-545)。区分: 単体(3 OS) |
| AC-514 | 依存の向きと循環の検査 | `src/`、`tests/Inspect/Architecture.Tests.ps1` | 点検(`src/Conductor/Private/<部品>/*.ps1` の全ファイルを構文木にし、関数の定義(`FunctionDefinitionAst`)と呼び出し(`CommandAst`)を集め、どの部品の関数が、どの部品の関数を呼ぶかの有向グラフを作り、深さ優先の探索(明示のスタック)で循環を調べる。テストの中に、許可の表を書く) | 循環が 0 件。許可されない向き(`Text` から他の部品、`Json` から `Text` 以外、`Definition`・`Convert`・`RunId`・`Flow` から `State`・`Runner`・`Report`・`Cli`、`State` と `Runner` の相互、`Report` から `Text` 以外、など。02 章 4 章の許可の表)がない(`State` は `Runner` を呼ばない。ディレクトリの同期は、`Cli` が `-SyncDirectory` のスクリプトブロックで渡す。決定)。違反するコードを一時ファイルに置くと、テストが失敗する(テストの自己確認)。呼び出し先が変数・式のコマンド(`& $name`)は、検査できないので、製品のコードで禁止する(AC-421)。CI の `lint` ジョブで実行する。区分: 点検 |
| AC-515 | `--help` / `--version` の標準出力が閉じたパイプでも落ちない | ランチャー | `conductor --help` と `conductor --version` を、標準出力を閉じた読み手(`$p.StandardOutput.Close()`、または `\| head -c 1`)で実行 | 終了 0(書き込みの失敗は無視する。08 章)。例外・エラーのレコードが現れない。70 にもならない。10 秒以内に終わる。区分: 自動(UNIX 系) |
| AC-516 | `watch` の標準出力が閉じたパイプ(UNIX 系と Windows の場合分け) | 十分に大きな表になるステート(ジョブ 2,000 個。`Slow`) | `watch` を起動し、標準出力の最初のデータを受けた時点で、読み手側を閉じる | **UNIX 系**: 終了 0 になりうる(08 章 1.3.1 の既知の差)。65 でもよい。どちらでも、例外・エラーのレコードが現れず、70 にならず、10 秒以内に終わる。65 のときは ERR に `conductor: error: cannot write to standard output`。**Windows**: 65(想定。未確認)。AC-493(`convert`)と対にする。確実に書き込みの失敗を起こすには AC-495(`/dev/full`。Linux)を使う。区分: 自動(UNIX 系と Windows) |
| AC-517 | シグナルの一覧と転送の単体(どの OS でも動く) | UNIX 系・Windows の起動の準備の関数(OS の種別を引数で指定して作る。07 章) | 受け取るシグナルの一覧、番号の対応、`Forward`(偽の子)、`ReadExit` を呼ぶ | UNIX 系: 一覧 = `SIGHUP`・`SIGINT`・`SIGQUIT`・`SIGTERM` の 4 つ、番号 1・2・3・15。転送は許可集合(1・2・3・15)だけ、`kill` の送信を 1 回呼ぶ。`ReadExit` は AC-504 のとおり。Windows: 一覧 = Ctrl+C(番号 2)だけ。`Forward` は何もしない。`ReadExit` は、`ExitCode`(`[int]`)をそのまま返す(符号つき 32 ビット)。区分: 単体(3 OS) |
| AC-518 | 静的検査の規則が有効であることの確認 | `src/` の外の一時ディレクトリに、違反するコードを書いた `.ps1`(`Invoke-Expression 'x'`、`iex 'x'`、`Write-Host 'x'`、`Write-Output 'x'`、`Start-Process x`、`Add-Type -TypeDefinition 'x'`、`exit 1`、`$a -eq 'A'`、`$a -match 'x'`、`'a'.ToUpper()`、`[int]::Parse('1')`、`Get-Content x`、`ConvertFrom-Json 'x'`、`$global:x = 1`、`catch { }`)を置く。リポジトリの `PSScriptAnalyzerSettings.psd1` と、禁止コマンドの検査を適用する | `Invoke-ScriptAnalyzer -Path <一時> -Settings PSScriptAnalyzerSettings.psd1` と `Invoke-Checks forbidden -Path <一時>` | それぞれが、PSScriptAnalyzer の規則(`PSAvoidUsingInvokeExpression`、`PSAvoidUsingWriteHost`、`PSAvoidGlobalVars`、`PSAvoidUsingEmptyCatchBlock` など)か、禁止コマンドの検査の指摘になる。診断の規則名は、`Invoke-ScriptAnalyzer` の出力の `RuleName`(`PS` の接頭辞が付く場合と付かない場合がある。**接頭辞の有無は、版により異なる可能性があり、未確認**)で確かめ、採用した版で固定する。製品のコードに `[Diagnostics.CodeAnalysis.SuppressMessageAttribute]` の抑止が 0 件(許可の一覧(理由つき)を除く)。区分: 点検 |
| AC-519 | 孤立サロゲートの表示 | UNIX 系。ジョブID・`--run-id`・`--config` のパスに、不正なバイト(`FF`)を渡す(AC-406 と同じ方法)。JSON の `"\ud800"` を含む `command`・キー名(03 章が受理する場合) | `run <ID>`、`validate --config <パス>`、`watch`(ジョブID に孤立サロゲートを含むステートがある場合) | ERR・OUT の書き込みで、**出力の符号化が、孤立サロゲートを U+FFFD に化かさず、例外も投げない**。出力が、有効な UTF-8 として読める(`[System.Text.UTF8Encoding]::new($false, $true).GetString(...)` が成功する)。孤立サロゲートは、08 章のエスケープ(例: `\u{d800}`)で出る(`UTF8Encoding` の既定の置換(U+FFFD)に任せない。生のバイトを出さない)。区分: UNIX 系と単体 |
| AC-520 | 標準出力・標準エラーの符号化 | 日本語を含む出力を出す実行(`LANG=ja_JP.UTF-8` で `watch`・`validate`・`--help`) | 環境変数 `LANG=C`、`LC_ALL=C`、`LC_ALL=ja_JP.eucJP`、`PSExecutionPolicyPreference`、`DOTNET_SYSTEM_GLOBALIZATION_INVARIANT=0`/`1` の各組み合わせで実行。Windows は、コンソールのコードページを `chcp 932` と `chcp 65001` にして(コンソールにつないだ場合と、パイプの場合) | どの組み合わせでも、OUT・ERR のバイト列が同じ(BOM なしの UTF-8)。`[Console]::OutputEncoding`・`$OutputEncoding` を書き換えず(利用者のセッションを変えない)、`[Console]::OpenStandardOutput()`・`[Console]::OpenStandardError()` から、BOM なしの UTF-8(`[System.Text.UTF8Encoding]::new($false)`)の書き出しを作る(08 章)。改行は LF 固定(`` `n ``。`[Environment]::NewLine` を使わない)。ランチャーの `pwsh` が、標準出力のバイト列に BOM や改行の変換を加えない。Windows のコンソールの表示(コードページ)は、別に手で確かめる(未確認)。区分: 自動 |
| AC-521 | 出力の順序(flush) | `P(p; a:0{cmd="TJ flaky F 1 75", retry=(exit_codes=[75], max=3, interval="1s")})`、標準出力と標準エラーを同じパイプ(`2>&1`。テストの補助がシェルで合流させる)にする | `run -v a` | 出力の行の順が、時間の順になる。`conductor` の `retry` の行が、2 回目のジョブの出力より前に現れる。`conductor` が自分の出力を、子を起動する前に `Flush()` している(書き込みのたびに `Flush()` する。08 章。flush の漏れの確認)。区分: 自動 |
| AC-522 | NUL を含む `command` | JSON の `"command": "a\u0000b"`、`"command": ["a", "b\u0000c"]`(JSON の `\u0000` のエスケープ。`.psd1` の文字列には NUL を書けないので、JSON の定義で試験する) | `validate --config conductor.json` / `run a --config conductor.json` | `validate` は 0(03 章は、`command` の中の NUL を定義の誤りにしない)。`run a` は、`Process.Start` が投げる例外(`InvalidOperationException`・`ArgumentException`・`Win32Exception` のいずれか。PowerShell では `MethodInvocationException` で包まれる。**NUL を含む `FileName`・引数に対する例外の型は、公式の文書で確認していない(未確認)**)を、07 章の規定で起動失敗にして、終了 127、ERR = `conductor: error: job 'a': cannot start command '…'`。70 にならない。ステートは `a=error ; resume=a`。区分: 自動と単体 |
| AC-523 | 空白の定義(`Trim()`・`-match '\s'` との差) | `command = "`u{1c}"`(PowerShell 7 の `` `u{…} `` の記法は、`.psd1` の二重引用符の文字列で使えるかを 03 章が決める。使えなければ JSON の `"\u001c"`)、`\u0085`、`\u00a0`、`\u200b`、`\u3000`、`name = U+001C`(別々の定義) | `validate` | 03 章の「空白だけ」の定義(Unicode の White_Space など)のとおり。`[string]::IsNullOrWhiteSpace`・`.Trim()`・`[char]::IsWhiteSpace`・正規表現の `\s` は、Unicode の White_Space の集合と、細部(U+001C〜U+001F、U+0085、U+180E など)が異なる可能性がある(Unicode のデータの版による変化を否定できない)。実装は、03 章の集合(25 文字)を明示した表で判定する(これらに頼らない)。`\u200b` は空白と見なさない(White_Space でない)。区分: 単体 |
| AC-524 | 再帰の深さ(スタックの大きさ) | 深さ 1,000 / 100,000 の `.psd1`・JSON(定義・ステート・変換。`Slow`)。`command` の分割(AC-410) | `validate` / `run a` / `convert` | スタックの溢れでプロセスが落ちない(64 / 65 / 67)。深さ 32 の入力を、**スタックを 256 KiB に制限したスレッド**(`[System.Threading.Thread]::new($start, 262144)`。PowerShell のスクリプトブロックを別のスレッドで動かすには実行空間が要るので、この試験は C# の補助でスレッドを作る。小さすぎて起動できない場合は、実装時に値を調整する)の中で処理できる(再帰の深さが、入力の深さに比例して大きくならない作りであること。製品が、スレッドのスタックの大きさを変えていない(点検))。PowerShell のパーサー(`Parser.ParseInput`)が、深い入れ子の入力で、どう振る舞うか(**未確認**)に注意し、構文木を作る前に、字句の走査で深さを数えて拒否する。区分: 単体と自動 |
| AC-525 | 型の自動変換に頼らない | `.psd1`: AC-228 の値に加え、`a = '2001-12-14t21:59:43.10-05:00'`(日時に見える文字列)、`b = '190:20:30'`、`c = '='`、`d = '<<'`、`e = '0o17'`、`f = '+1'`、`g = '.5'`、`h = '1.'`、`i = '-.inf'`、`j = '0x_1F'`、`k = '1e3'`(すべて単一引用符の文字列) | `convert in.psd1 --to json` | すべて文字列のまま(`[datetime]`・`[timespan]`・`[version]`・`[int]` に化けない。PowerShell の `-eq`・`-lt`・`+` は、右辺や左辺の型に合わせて、もう一方を**変換する**(例: `'1' + 1` は `'11'`、`1 + '1'` は `2`、`'10' -lt 9` は `$false`)ので、製品のコードの値の比較・結合は、型をそろえてから行う)。**数値になるのは、引用符のない整数・小数の字句だけ**。区分: 自動 |
| AC-526 | パッケージの中身の検査 | 作ったアーカイブ | `./build/Invoke-Checks.ps1 -Check dist -Path <ディレクトリ>`(5.4 節) | 展開した内容が AC-440 のとおり(`tests/`・`build/`・`demo/`・`artifacts/`・`.github/`・`*.Tests.ps1`・`PSScriptAnalyzerSettings.psd1`・`Dependencies.psd1` を含まない)。`bin/conductor` が空でなく、1 行目が `#!` で始まり、実行権限がある(tar.gz)。`bin/conductor.cmd` の改行が CRLF(Windows のバッチファイルの慣習。**LF だけの `.cmd` が正しく動かない場合があることは、記憶による。未確認**。`.gitattributes` で `*.cmd text eol=crlf` を指定する)。`Test-ModuleManifest` が、展開したモジュールで成功する。アーカイブのサイズを記録する。区分: 点検 |
| AC-527 | パッケージのメタデータ | 展開した成果物 | 点検(`conductor --version` と、ファイルの属性) | `--version` の OUT が `conductor <ModuleVersion>`。`bin/conductor` の UNIX 系の実行権限が付いている(`tar` に、権限が保たれる。zip は Windows 用だけ。zip は、権限を保たない場合がある)。モジュールのファイルが、BOM つきの UTF-8 か、BOM なしの UTF-8 かが、PSScriptAnalyzer の `UseBOMForUnicodeEncodedFile`(非 ASCII の文字を含むファイルに BOM を求める。常に有効)の指摘を出さない(`.ps1`・`.psm1` に非 ASCII の文字(日本語の固定の文)を含める場合の方針を、実装時に決める。**Windows PowerShell 5.1 は BOM なしの UTF-8 を誤読するが、本版は 7.x だけを対象にするので、BOM なしを基本とし、規則の指摘は設定で外す**。理由を設定ファイルのコメントに書く)。区分: 点検 |
| AC-528 | 展開した配布物が空の環境で動く | 配布されたアーカイブ | `pwsh` 以外の道具(.NET SDK、Pester、PSScriptAnalyzer、`PSModulePath` の追加)がない環境で展開して、`conductor --version`、`validate`(`B0`) | 動く(実行時の依存なし)。`HOME` が未設定でも動く(`pwsh` が `HOME` を要する場合は、そのこと(**未確認**)を README に書く)。`validate` が 0。区分: 自動(3 OS) |
| AC-529 | モジュールが読み込める | `src/Conductor` | `pwsh -NoProfile -Command "Import-Module ./src/Conductor -ErrorAction Stop; (Get-Module Conductor).Version.ToString()"` | 動き、OUT = `<ModuleVersion>`。`Import-Module` が警告(`WARNING:` の、承認されない動詞の名前の警告など)を出さない。区分: 自動(Linux の CI) |
| AC-530 | 文化(カルチャ)に依存しない | ランチャー | `--help` と `watch` を、`LANG`・`LC_ALL` を変えて実行(`ja_JP.UTF-8`、`tr_TR.UTF-8`(トルコ語。`I` の大文字小文字の規則が違う)、`C`)。`DOTNET_SYSTEM_GLOBALIZATION_INVARIANT=1` の有無でも。ICU のライブラリが無い環境(コンテナ)でも | どのロケールでも、OUT・ERR のバイト列が、決められた言語の選択(08 章)のとおりで、ICU の有無で変わらない。製品のコードが、文化依存の書き方(`.ToUpper()`・`.ToLower()`・`-f`・`[string]::Format`・`"$数値"` の展開・`[int]::Parse(<文字列>)`・`.ToString()` の引数なし(数値・日時)・`Get-Culture`・`$PSCulture`・`[System.Threading.Thread]::CurrentThread.CurrentCulture`)を使わない(AC-540)。**`LANG=tr_TR.UTF-8` で `'i'.ToUpper()` が `İ` になることは、この文書では確認していない(未確認)。ただし、文化に依らない比較にする規定は変えない** |
| AC-531 | 危険な構文のデータからコードが実行されない(定義・変換の両方) | `.psd1`: `command = $(Set-Content pwned.txt x)`、`a = (Start-Process touch pwned.txt)`、`b = [System.Diagnostics.Process]::Start('touch', 'pwned.txt')`、`c = { Set-Content pwned.txt x }`、`d = "$([System.IO.File]::WriteAllText('pwned.txt','x'))"`、`e = & 'touch' 'pwned.txt'`、`using module .\evil.psm1`、`#requires -Modules Evil`(コメントなので無害)、`$ExecutionContext.InvokeCommand.InvokeScript('…')` | `validate --config <.psd1>` / `convert in.psd1 --to json` | 64 / 67(`syntax error`。理由 `使えない構文です`。03 章)。`T` に `pwned.txt` ができない。モジュールの読み込み(`using module`)も `Import-Module` も起きない。`Import-PowerShellDataFile`・`Invoke-Expression`・`[scriptblock]::Create`・`ConvertFrom-Json` を製品が呼ばない(AC-421)。区分: 自動 |
| AC-532 | シグナルの登録の前の `SIGINT` は、出力なしで、終了する | UNIX 系。`tests/jobs/inject.ps1 sigint <引数>`(`sigint` は、入口を呼ぶ前に、自分に `kill -INT $PID` を送って眠る。製品に試験用の経路を持たせず、外から、シグナルの登録がない経路の `SIGINT` を再現する) | `watch` / `reset` / `validate` / `convert in.psd1 --to json` の各引数で起動 | OUT・ERR が空(例外・`internal error` が出ない)。終了は、シグナル 2 による終了、終了コード 130、または、`pwsh -File` の Ctrl+C の既定の 0(確認: 1 章)。70 にならない。ステートファイルが壊れず、一時ファイルが残らない。**`pwsh` の既定の `SIGINT` の動作(終了コード、`Console.CancelKeyPress` を登録していないときの挙動、`PipelineStoppedException`)は、この文書では確認していない(未確認)**。実機で確かめ、期待を 1 つに絞って 10 章に記録する。Windows の、シグナルの準備をしない経路の Ctrl+C は、OS の既定の動作に任せる(終了コードは未確認で、期待を決めない) |
| AC-533 | Windows の Ctrl+Break は受け取らない(任意) | Windows の CI または実機。`SL`(眠るジョブ)。テストが、ランチャーを、新しいプロセスグループで起動する(`CreateProcess` の `CREATE_NEW_PROCESS_GROUP`。`ProcessStartInfo` には、この指定の API がない(未確認)ので、補助の P/Invoke(`CreateProcessW`)でテストの側が起動する。製品のコードは指定しない) | ジョブが `ready` を記録したあと、`GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, pid)` を送る(P/Invoke) | 07 章の決定: Ctrl+Break は `conductor` の中断として扱わない。したがって、ステートに「中断」の記録(`interrupted by signal`)が残らない。`conductor` の終了の様子(OS の既定の動作)は未確認で、期待を決めない。実施した場合は、結果を 10 章に記録する。区分: 任意(Windows) |
| AC-534 | `SIGCHLD` を無視して起動されても、ジョブの終了コードが正しい | Linux。`sh -c 'trap "" CHLD; exec <ランチャー> run a'`(`SIG_IGN` は `exec` を越えて引き継がれる。テストの補助だけがシェルを使う)。`P(p; a:3)`(`TJ exit 3`) | `run a` | 終了コード 3(`0` や別の値にならない)。ステートは `a=error(3)`。ランタイムが、`SIGCHLD` の処理を内部で持つこと(子の回収を `waitpid` で行う)と、`SIG_IGN` を継承したときの扱いは、**公式の文書で確認していない(未確認)**。実機で確かめ、結果を 10 章に記録する。区分: 自動(Linux。macOS では、必須としない) |
| AC-535 | 疑似端末の下で、ジョブが標準入力から読める。ジョブは制御端末を持つか | UNIX 系。疑似端末の子側を、ランチャーの標準入力・標準出力・標準エラーにする(テストの補助が P/Invoke で行う)。`TJ` のジョブ 2 つ: (a) `TJ readline`、(b) `TJ tty` | (a) `run a`(親から 1 行を書く)、(b) `run b` | (a) 5 秒以内に 0 で終わる(`SIGTTIN` で止まらない)。(b) 0(07 章の決定: 子を新しいセッションに置かないので、制御端末を持ち、`/dev/tty` を開ける)。ランチャーの `pwsh` が、端末の設定(エコー・行編集)を変えて、ジョブの読み取りを妨げない(`-NonInteractive`)。区分: 自動(UNIX 系。実行は未確認) |
| AC-536 | `convert` の入力が名前つきパイプ(FIFO)でもブロックしない | UNIX 系。`mkfifo T/in.fifo`(書き込む側は現れない。テストの補助が `mkfifo` コマンドを起動する) | `convert in.fifo --to json`(タイムアウト 5 秒) | 5 秒以内に終わり、終了 67。ERR の最初の行の種類の文 = `cannot read input file`(理由 `通常のファイルではありません`)。09 章の決定: ファイルを開く前に、`[System.IO.FileInfo]` の `Attributes`・`UnixFileMode`(または `[System.IO.File]::GetAttributes`)で通常のファイルか調べる(FIFO を開くとブロックするため)。**`FileInfo` が FIFO をどう報告するかは、公式の文書で確認していない(未確認)**。区分: 自動(UNIX 系) |
| AC-537 | シグナルの登録が GC されない | UNIX 系。`Runner` のシグナルの準備の単体 | シグナルを登録したあと、`[System.GC]::Collect()`、`[System.GC]::WaitForPendingFinalizers()`、`[System.GC]::Collect()` を 3 回行い、そのあとシグナルを送る(自分への `kill`) | コールバックが呼ばれ、旗が立つ(ファイナライザが登録を解除していない。登録のオブジェクトを、静的なフィールド(補助の部品の C# の静的なクラス)に持つ。PowerShell の変数(`$script:` を含む)だけに持たない。1 章で確認した、`PosixSignalRegistration` のファイナライザが登録を解除する、という性質に対する、保持の確認)。区分: 単体(UNIX 系) |
| AC-538 | シグナルのコールバックが別のスレッドで動いても、旗が見え、PowerShell のスクリプトブロックが呼ばれない | 同上 | 別のスレッドから(メインスレッドが、旗を短い間隔で見ている間に)シグナルを送る。1,000 回繰り返す(1 回ごとに登録し直す) | 毎回、メインスレッドが旗を見て、待ちから起きる(旗の読み出しが `Volatile.Read` か `Interlocked`、起こす操作が `ManualResetEventSlim.Set`)。最初の番号が残る。**コールバックが PowerShell のスクリプトブロックだった場合に、別のスレッドで呼ばれて失敗する(実行空間がない)ことを、再現して確かめる**(試験用に、スクリプトブロックをコールバックにした版を作り、`There is no Runspace available to run scripts in this thread` 相当のエラーが出ることを確かめる。エラーの文言は未確認)。製品は、コールバックを C# の補助の部品の中で完結させる(AC-499)。区分: 単体(UNIX 系) |
| AC-539 | `PosixSignalContext.Cancel` の扱い | UNIX 系 | `run` の実行中に SIGTERM・SIGINT・SIGHUP・SIGQUIT を送る(AC-280〜282) | `conductor` のプロセスが、ランタイムの既定の動作(即座の終了)で終わらず、ジョブの終了を待ってから、`128 + n` で終わる(コールバックが `Cancel = true` を設定している。`Cancel` を設定しないと、ランタイムが、そのシグナルの既定の動作を続けるため。**この動作の正確な意味は、この文書では公式の文書を確認していない(未確認)**)。区分: 自動(UNIX 系) |
| AC-540 | 文化依存の API を使わない | ソース | 点検(構文木の走査) | 製品のコードが、文化依存の API(`.ToUpper()`・`.ToLower()`・`[string]::Compare(<2 つ>)`・`.CompareTo`・`.StartsWith(<文字列>)`・`.EndsWith(<文字列>)`・`.IndexOf(<文字列>)`・`[int]::Parse(<文字列>)`・`[long]::Parse`・`[double]::Parse`・`.ToString()`(数値・日時の、書式なし)・`[string]::Format`・`-f`・`"$数値"` の展開・`Get-Culture`・`$PSCulture`)を使わない(`build/Invoke-Checks.ps1 forbidden` で禁止)。使うのは、序数比較(`[System.StringComparison]::Ordinal`・`OrdinalIgnoreCase`)と、`[System.Globalization.CultureInfo]::InvariantCulture` を明示した形(`.ToString([System.Globalization.CultureInfo]::InvariantCulture)`、`[int]::TryParse($s, [System.Globalization.NumberStyles]::None, [System.Globalization.CultureInfo]::InvariantCulture, [ref]$v)`)だけ。`.StartsWith('x')` の文字列の引数の形は、文化依存なので、`.StartsWith('x', [System.StringComparison]::Ordinal)` と書く。区分: 点検 |
| AC-541 | 数値の型変換の落とし穴 | ソース | 点検(構文木の走査)と単体 | `[int]` へのキャストが、範囲外で例外になり、小数を銀行丸め(`[int]2.5` が 2、`[int]3.5` が 4)にする(確認できていない。記憶による。実装時に確認)ので、キャストの前に範囲と整数であることを検査する。`[int]'１'` や `[int]' 1 '`、`[int]'0x10'` のような、文字列からの変換は使わない(AC-543)。数値のリテラルの型の自動選択(`int` → `long` → `decimal` → `double`)に頼らず、`[int]`・`[long]`・`[uint64]`・`[bigint]` を明示する。`-bor`・`-band`・`-shl` の結果の型を明示する。`[uint64]` の演算の溢れに頼らない(AC-501)。区分: 点検と単体 |
| AC-542 | 正規表現の落とし穴(`$`、`\d`)と、演算子 | `RunId` の単体と、`--run-id` | run ID の検査に、`r1<LF>`(末尾が改行)、`１２３`(全角の数字)、`٣`(アラビア・インドの数字)、`a<U+0085>` を渡す | すべて無効(68)。**.NET の正規表現の `$` は、文字列の末尾の `\n` の前にも一致し、`\d` は Unicode の数字(Nd)に一致する**(この文書の作成時には確認していない。未確認)。したがって、製品は、run ID の検査に正規表現(`-match`・`[regex]`)を使わず、1 文字ずつ、ASCII の文字の範囲を比べる(正規表現を使う場所は、`\z` と `[System.Text.RegularExpressions.RegexOptions]::CultureInvariant` を、`[regex]::new` で明示する)。`-match`・`-notmatch`・`-replace`・`-split`・`-like`・`-notlike` は、大文字小文字を区別せず、右辺を正規表現・ワイルドカードとして解釈する(確認できていない。記憶による)ので、製品のコードで禁止する(`forbidden`)。区分: 単体と自動 |
| AC-543 | 数値の字句を、文化・書式に依存せず読む | `Definition` の間隔(`1s`・`2m`・`3h`)の単体 | `interval` に、`'１s'`(全角)、`'+1s'`、`'-1s'`、`'1_0s'`、`'1,0s'`、`'1.0s'`、`'0x1s'`、`' 1s'`、`'1 s'`、`'1s '`、`"1s`n"`(末尾が改行)、`'٣s'` | すべて無効(`retry interval is invalid`)。`[int]::Parse`・`[long]::Parse`・`[uint64]::Parse` の既定の `NumberStyles`(先頭・末尾の空白、符号、桁区切りを受理する場合がある)と、全角の数字の扱いに依存しない(`[char]` が `'0'`〜`'9'` の範囲かを自前で比べて、桁を積む。`[uint64]` の範囲の検査は先に行う)。PowerShell の文字列から数値への暗黙の変換(`'1' + 1`、`[int]'1'`)を使わない。区分: 単体 |
| AC-544 | `[double]` の書式 | `Convert` の単体 | JSON の浮動小数点数の出力に、`1e21`、`1e-7`、`0.1`、`-0.0`、`123456789012345678`(整数の字句は別)、`5e-324`、`1.7976931348623157e308`、`0.30000000000000004` を通す | 09 章の規則の書式(字句が、入力のまま保たれる場合は、入力の字句)で出る。`[double]::ToString()` の既定(文化依存。`E+21` の形)や、`'R'` の出力を、そのまま使うかは 09 章に従う(`.ToString('R', [System.Globalization.CultureInfo]::InvariantCulture)`)。往復で値が変わらない(最短の往復の書式。`'R'` が最短の往復の文字列を返す、と記憶している(未確認))。区分: 単体 |
| AC-545 | 拡張子の判定の落とし穴 | `Definition` の拡張子の判定の単体 | `--config` に、`.json`、`.psd1`、`a.`、`a.json.`、`a.JSON`、`dir.json/file`、`dir.json\file`(UNIX 系ではファイル名の一部)、`a.json `(末尾に空白)、`a.psd1.bak`、`conductor.psd1` を渡す | 03 章の規則どおり(最後の区切り文字のあとの、最後の `.` から末尾まで。大文字小文字を区別しない)。`[System.IO.Path]::GetExtension` や、`-like '*.json'`・`-match '\.json$'`(大文字小文字を区別しない。`$` が末尾の改行の前に一致する)に頼らず、03 章の規則を、序数比較で自前で実装する(**`GetExtension` の細部は、この文書では公式の文書を確認していない(未確認)**)。区分: 単体 |
| AC-546 | ファイルの置き換えと Windows | `State` の単体と、Windows の CI | ステートを 1,000 回書き換える。読み手のスレッドが、同時に読み続ける(Windows。読み手は C# の補助か、別の `pwsh`) | 読み手が、直前の完全な内容か新しい完全な内容だけを見る。置き換えに失敗(`IOException`、`UnauthorizedAccessException`)した場合は、短い再試行をするか、ステートの書き込みの失敗(65)にする(06 章の決定に従う)。**Windows で、置き換え先が他のプロセスに開かれている場合の `[System.IO.File]::Move($src, $dst, $true)`・`[System.IO.File]::Replace` の挙動は、公式の文書で確認していない(未確認)**。`Move-Item`・`Set-Content` を使わない。区分: 単体(3 OS) |
| AC-547 | `$args`・共通パラメーターと、引数の解析 | ソース、ランチャー、`Invoke-Conductor` | `conductor -v run a`、`conductor run -Verbose a`、`conductor run a -Debug`、`conductor run a -ErrorAction Stop`、`conductor -?`、`conductor /?`、`conductor -h`、`conductor run a -WhatIf`、`conductor -Command x`、`conductor -File x`、`conductor run a -- -v`、`conductor "run a"`(空白を含む 1 つの引数)、`conductor --config=C:\a.psd1 validate`(Windows の絶対パスの `:`) | PowerShell のパラメーター束縛と `pwsh -File` の引数の変換が、引数を食わない・変えない(入口スクリプトは `$args` でなく、`GetCommandLineArgs()` から生の引数を取り出す。[04](04-cli.md) 1.5 節。共通パラメーターの `-Verbose`・`-Debug`・`-ErrorAction`・`-WhatIf` も、`conductor` の引数の文字列のまま、`Invoke-Conductor -ArgumentList` に渡る)。`conductor` の文法の解釈は 04 章 4 章のとおり: `-v` は verbose(`-Verbose` ではない)。**`-Verbose` は、連結の分解で先頭の `-V` が version になり、`conductor 0.8.0` を出して 0**(`unknown option` にならない。04 章 4.2 の 1)。`-Debug` は `unknown option: '-D'`、`-ErrorAction` は `unknown option: '-E'`、`-WhatIf` は `unknown option: '-W'`(68)。`-?` は `unknown option: '-?'`(68)、`/?` は `unknown command: '/?'`(68)(PowerShell のヘルプとして処理しない)。`-h` は help(0)。`-Command x` は `unknown option: '-C'`、`-File x` は `unknown option: '-F'`(68)(`pwsh` のオプションとして解釈されない。`pwsh -File` の後ろの全部がスクリプトの引数。確認: 1 章)。`-- -v` は 04 章 4.1 の規則。`"run a"` は `unknown command: 'run a'`(68)。`--config=C:\a.psd1` は、`--config` の値が `C:\a.psd1` のまま届く(ファイルがなければ 64)。**`Invoke-Conductor` が `[CmdletBinding()]` を持つと、`-Verbose` などが共通パラメーターとして束縛されうる。持たず、`-ArgumentList [string[]]` の配列で受ける(04 章 1.4 節)。`$args` が空でないとき(ランチャーを経由しない呼び出しの誤り)は、70**。区分: 自動 |
| AC-548 | `Process.Start` の例外の網羅 | `Runner` の単体(`Start` を差し替えられる形にして、各例外を投げる偽の起動) | `Win32Exception`(`NativeErrorCode` の各値)、`InvalidOperationException`、`ArgumentException`、`PlatformNotSupportedException`、`IOException`、`UnauthorizedAccessException`、`OutOfMemoryException`(これは捕まえない)。PowerShell では `MethodInvocationException` に包まれた場合と、包まれない場合 | `OutOfMemoryException` 以外は、すべて起動失敗(分類 順 4)として、終了 127。例外のメッセージ・`NativeErrorCode` の値は、表示に出さない。包まれた例外は、`InnerException` をたどって種類を判定してもよいが、判定できなくても、`OutOfMemoryException` 以外は起動失敗にする。`OutOfMemoryException` は、入口の `catch` で、70(AC-414)。区分: 単体 |
| AC-549 | `finally` と終了 | ソース | 点検と自動 | `Invoke-Conductor` が戻り値で終わる経路では、`finally` の処理(シグナルの登録の解除、ステートの一時ファイルの削除、標準出力の `Flush`)がすべて実行される。`exit` はランチャーだけが呼ぶ(AC-500)。ランチャーが `exit` を呼ぶ前に、標準出力・標準エラーの `Flush` が済んでいる。区分: 点検と自動 |
| AC-550 | UNIX 系の実行ファイルの解決(絶対パスへの自前の解決) | UNIX 系。`T/bin/hello`(実行権限つきのスクリプト。`T/hello` と、ランチャーのディレクトリの `hello` にも、別の内容の同名の実行ファイルを置く)。`P(p; a:0{cmd="hello"})`(`command` は区切りなしの `hello`) | (a) `PATH=T/bin:/usr/bin:/bin` で、作業ディレクトリを `T` にして `run a`。(b) `PATH` を未設定にして `run a`。(c) `cmd="./hello"`(区切りあり)で `run a`(作業ディレクトリは `T`) | (a) `T/bin/hello` が起動される(カレントディレクトリや `bin/` の同名ファイルは起動されない。07 章の決定。**PowerShell の `Get-Command`・`&` 演算子のコマンド探索(カレントディレクトリを探さないが、エイリアス・関数・コマンドレットが先に一致する)に頼らない**)。(b) 起動の失敗で 127(`PATH` が未設定のときは探索しない)。(c) `[System.IO.Path]::GetFullPath` で、PowerShell の現在の場所(`$PWD.ProviderPath`)を基準に解決した `T/hello` が起動される。区分: 自動(UNIX 系) |
| AC-551 | 厳格モードと未定義の変数 | モジュールのソース、`tests/Inspect/` | 点検(構文木)と自動 | `Conductor.psm1` の先頭が `Set-StrictMode -Version Latest`、`$ErrorActionPreference = 'Stop'`。製品のすべての `.ps1` に、`Set-StrictMode -Off`・`Set-StrictMode -Version` の下げる呼び出しがない。テスト(`Unit`)も `Set-StrictMode -Version Latest` で行う。未定義の変数の参照・存在しないプロパティの参照(`$x.Missing`)が、70 で終わらず、単体の試験で失敗として検出される(`Set-StrictMode` の下で例外になる)。`$script:` の変数は、すべてモジュールの読み込み時に初期化される。`$global:` を使わない(`PSAvoidGlobalVars`)。区分: 点検と単体 |
| AC-552 | パイプラインへの出力の混入がない | `Invoke-Conductor` と、各部品の公開関数 | `$r = Invoke-Conductor -ArgumentList @('--version')`、`$r = Invoke-Conductor -ArgumentList @('run', 'a')`(成功・失敗・例外・中断の各経路)、`*>&1` で全ストリームを捕まえる | `$r` が、ちょうど 1 つの `[int]`(`$r -is [int]`、`@($r).Count -eq 1`)。成功ストリーム(出力)・エラー・警告・冗長・情報の各ストリームに、何も出ない(`*>&1` の結果が空)。画面への出力は、`Write-ConductorStream`(`[Console]::OpenStandardOutput()` / `OpenStandardError()` のストリームを包んだ `StreamWriter`。08 章 1.2.2)だけが行い、PowerShell のストリームを通らない。`.Add()`・`.Remove()`・`[void]`・`$null =` の戻り値の捨て忘れ(`[System.Collections.Generic.List[object]]::Add` は戻り値がないが、`ArrayList.Add` は戻り値を返す)がない。関数は `return` で戻り値を明示する。区分: 自動と単体 |
| AC-553 | 比較演算子の大文字小文字と型の変換 | `Definition`・`Flow`・`Cli` の単体 | ジョブID `A` と `a`(別のジョブ)、`run A` / `run a`、`on = 'Success'`、`'@END'`、`--to PSD1`、`-V` と `-v`、`LANG=JA`、ファイル名 `Conductor.psd1` と `conductor.psd1`、キー `Id`、状態 `Success`、`1` と `'1'` と `$true`(`-eq` の左右の型の変換) | すべて、大文字小文字を区別して比べる(**`-eq`・`-ne`・`-lt`・`-gt`・`-like`・`-match`・`-contains`・`-in`・`switch` は、既定で大文字小文字を区別しない**(確認できていない。記憶による。実装時に確認)ので、製品のコードは、`-ceq`・`-cne`・`-ccontains`・`-cin`、`[string]::Equals($a, $b, [System.StringComparison]::Ordinal)`、`switch -CaseSensitive` を使う)。`1 -eq '1'`・`'1' -eq 1`・`$true -eq 'x'`・`'0' -eq $false` のような、型をまたぐ `-eq` が、真になる(PowerShell が右辺の型に左辺を変換する)ことに頼らず、型をそろえる。`$null` は左辺に置く(`$null -eq $x`)。`if ($x)` の真偽判定で、`0`・`''`・`@()`・`$null`・`'0'` を取り違えない(`if ($x -ne $null)`・`$x.Count -gt 0` など、意図を書く)。区分: 単体と点検 |
| AC-554 | 配列の単一化 | `Definition`・`Json`・`Report` の単体 | `.psd1` `warning_exit_codes = @(3)`、`job = @(@{ id = 'a'; command = 'x' })`(1 ジョブ)、`transitions = @(@{ … })`(1 要素)、`argv = @('x')`(1 要素の `command` 配列)、`job = @()`、JSON の `[3]`・`[{"id": "a"}]`・`[]`。関数の戻り値(1 要素の配列・空の配列・`$null`)。`Where-Object`・`ForEach-Object`・パイプラインの結果が 0 個・1 個・複数のとき | 1 要素の配列が、配列のまま扱われる(`.Count`・インデクサー・`foreach` が正しく動く)。`warning_exit_codes = 3`(`@()` なし。1 つの整数)は `wrong value type`(AC-031)。`command = @('x')` は、配列形式の `command`(`argv`)で、文字列形式ではない。空の配列(`job = @()`)は `no jobs are defined`。関数が配列を返すときは、`return ,$array`(単項のカンマ)か、呼び出し側の `@(...)` で、パイプラインの展開と、0・1 要素での単一化(0 個は `$null`、1 個は要素そのもの)を避ける(使い分けは 02 章の規約に従う)。**1 要素の配列を JSON・`.psd1` に書き出すとき、`[` `]` と `@(` `)` を省かない**(AC-224)。区分: 単体 |
| AC-555 | `$null`・空・存在しないキーの区別 | `Definition`・`State` の単体 | キーがない / 値が `$null` / 値が空文字列 / 値が `@()` / 値が `0` / 値が `$false` の各場合の、`format_version`・`name`・`command`・`resume_job_id`・`run_id`・`exit_code`・`end_reached` | それぞれが区別される(`exit_code` の `0` が「ない」に化けない。`resume_job_id` がないことと、空文字列を区別する)。`if ($x)` や `$x -eq $null`(右辺に `$null` を置くと、配列に対して要素のフィルタになる)に頼らず、`$null -eq $x` と、`ContainsKey` を使う。`[ordered]`・`[hashtable]` のキーの有無は `ContainsKey`(`$h.key` は、キーがなくても、厳格モードの下では例外になる場合がある。**未確認**)。区分: 単体 |
| AC-556 | 終端エラーと非終端エラー | `Runner`・`State` の単体 | ファイルがない、権限がない、ディレクトリを開く、`Process.Start` の失敗の各場面で、.NET のメソッドが投げる例外と、コマンドレットの非終端エラー(`$ErrorActionPreference` が `Continue` のとき、続行して `$Error` に溜まる)を区別する | 製品のコードが、コマンドレットを呼ぶ場合は `-ErrorAction Stop` を明示するか、.NET のメソッドを使う(`Get-Content`・`Set-Content`・`Remove-Item`・`Move-Item`・`New-Item` を使わず、`[System.IO.File]`・`[System.IO.Directory]` を使う。5.4 節の `forbidden`)。`$ErrorActionPreference = 'Stop'` のもとで、非終端エラーが、握りつぶされず、70 または対応する終了コード(65・67 など)になる。`try`/`catch` が、非終端エラーを捕まえそこねて、表示が壊れることがない。区分: 単体 |
| AC-557 | `$LASTEXITCODE` と `$?` に頼らない | ソース | 点検(構文木の走査) | 製品のコードが、`$LASTEXITCODE`・`$?` を読まない(子プロセスの終了コードは `$process.ExitCode`)。ランチャーの終了コードは、`Invoke-Conductor` の戻り値の `exit`(`exit $code`)で決め、`$LASTEXITCODE` を引き継がない。区分: 点検 |
| AC-558 | `Add-Type` の再実行 | 同じ `pwsh` のセッションの中で | `Import-Module ./src/Conductor -Force` を 2 回行い、そのたびに `run` 相当(補助の部品の用意)を呼ぶ。別の版の補助の部品の C# のソースを、同じ型の名前で読み込む | 2 回目に、型の再定義のエラー(`Cannot add type. The type name '…' already exists.` 相当。**文言は未確認**)が出ない。補助の部品の型の名前に、ソースのハッシュ(または版)を含めるか、`($type -as [type])` で型の有無を確かめてから `Add-Type` する(`-Force` の `Import-Module` は、読み込み済みの .NET の型を取り除かない)。型の中の静的な状態(シグナルの旗、登録)が、2 回目の読み込みで二重にならない(AC-285)。区分: 単体 |
| AC-559 | 現在のディレクトリのずれ | `Invoke-Conductor` を、`pwsh` のセッションの中で、`Set-Location $T` のあとに呼ぶ(`[System.Environment]::CurrentDirectory` は、`Set-Location` では変わらない)。定義 `--config sub/x.psd1`(相対)、ジョブの `cwd` | `Set-Location $T; Invoke-Conductor -ArgumentList @('run', 'a', '--config', 'sub/x.psd1')`(プロセス内) | 相対のパスが、**PowerShell の現在の場所(`$PWD.ProviderPath`)**を基準に解決される。`Invoke-Conductor` が、開始時に `$PWD` が `FileSystem` プロバイダーの場所なら、`[System.Environment]::CurrentDirectory` を `$PWD.ProviderPath` に合わせ、`finally` で元に戻す(04 章 1.4 節)ので、`[System.IO.File]` のメソッドや、ジョブの `WorkingDirectory`(設定しない。親の作業ディレクトリの引き継ぎ。07 章 4.2 節)に相対のパスをそのまま渡しても、同じ場所を基準にする。呼び出しのあと、`[System.Environment]::CurrentDirectory` が元の値に戻っている。製品のコードで `Set-Location` を呼ばない。`Resolve-Path`・`Join-Path` は、存在しないパスで失敗する・PowerShell のドライブ(`TestDrive:`)を返す場合があるので、パスの検証に使わない。ランチャー経由では、2 つが一致するので、同じ結果になる。区分: 単体と自動 |
| AC-560 | Constrained Language Mode | 新しい `pwsh` で、`$ExecutionContext.SessionState.LanguageMode = 'ConstrainedLanguage'` にしてから(戻せない)、`Import-Module ./src/Conductor` | `Invoke-Conductor` で `--version`、`--help`、`validate`、`watch`、`reset`、`convert`、`run a`(定義は正しい) | `--version`・`--help`・`validate`・`watch`・`reset`・`convert` は、`Add-Type` を使わないので動く(0)。`run`・`pipeline` は、補助の部品(`Add-Type`)が使えないので、シグナルの準備の失敗として、70、ERR = `conductor: error: failed to set up signal handling`(AC-298)。ジョブは起動されない。**`Add-Type` が Constrained Language Mode で失敗すること、`LanguageMode` を代入できること、構文木の API(`[Parser]`)の型が、この言語モードで使えるか(使えないと `validate` も失敗する)は、いずれも確認していない(未確認)。実装時に最初に確かめ、使えなければ、この行の期待を実機に合わせて直す**。区分: 自動 |
| AC-561 | 実行ポリシーと Zone.Identifier(Windows) | Windows。配布の zip を、インターネットから取得した印(`Zone.Identifier` の代替データストリーム)が付いた状態で展開する(`Set-Content -Path <各ファイル> -Stream Zone.Identifier -Value "[ZoneTransfer]`nZoneId=3"` で付ける)。実行ポリシーが `RemoteSigned`(Windows の既定。確認: 1 章) | `bin\conductor.cmd --version`(印を付けたまま)。`Get-ChildItem <展開先> -Recurse -File \| Unblock-File` のあとに再度 | 印を付けたままでは、ランチャーが起動に失敗する場合がある(`.ps1`・`.psm1`・`.psd1` が、実行ポリシーで止まる)。その場合の終了コードと表示は、`pwsh` のもの(`conductor` の表示ではない)で、**期待を決めない(未確認)**。`Unblock-File` のあとは動く(0)。README の手順が、この手順と一致する(6.5 節)。ランチャーが `-ExecutionPolicy` を渡す場合は、`Bypass` などが、実行ポリシーを迂回するか(**未確認**)、07 章に従う。製品のコードは `Set-ExecutionPolicy` を呼ばない(AC-421)。UNIX 系では、実行ポリシーが強制されない(確認: 1 章)ので、この項目は対象外。区分: 手動(Windows) |
| AC-562 | `ModuleVersion` の一致 | `Conductor.psd1`、ランチャー、テスト、`README.md`、タグ | `./build/Invoke-Checks.ps1 -Check version`(タグがあるときは `-Tag v0.8.0`) | `ModuleVersion` が、`conductor --version` の OUT、README の版の記載、タグ(`v` を除く)、パッケージのディレクトリ名・アーカイブ名と一致する。ランチャーとテストに、版の数字の直書きがない。`ModuleVersion` が 3 つの数字(`0.8.0`)で、先行版の接尾辞は `PrivateData.PSData.Prerelease` に書く。不一致なら失敗(終了 1)。区分: 点検 |
| AC-563 | 起動の引数の壊れ(Windows のランチャー) | Windows。`bin\conductor.cmd`。引数 `run`、`a b`(空白を含むジョブID)、`--config`、`C:\path with space\x.psd1`、`100%`、`a^b`、`a&b`、`a\|b`、`a<b`、`"q"`、`!x!`、`(x)`、空の引数 `""` | 各引数を 1 つずつ含めて実行(ジョブIDとして渡す。`unknown job` の表示に、渡した値が(エスケープして)現れる) | 空白・`(`・`)`・`!`・`""` を含む引数は、そのまま届く(`unknown job: 'a b'` など)。`%`・`^`・`&`・`\|`・`<`・`>`・`"` を含む引数は、**届くか、起動を拒否する(07 章の決定に従う)かのどちらか**で、**続くコマンドが実行されない**(`T/injected.txt` ができない)。ランチャーが引数を壊して、別のジョブIDや別のサブコマンドとして実行することがない。**`cmd.exe` の引数の引用符と `%`・`^`・`&` の扱いは、公式の文書で確認していない(未確認)。07 章の決定に合わせて、この行の期待を直す**。区分: 自動(Windows の CI) |
| AC-564 | `pwsh` を起動するときの環境の独立 | ランチャー。`PSModulePath` に別の `Conductor` を置く、`$PROFILE` に出力するプロファイル(`HOME` を一時ディレクトリにして置く)、`POWERSHELL_*`・`PSExecutionPolicyPreference` の環境変数 | `--version`、`validate` | プロファイルを読まない(`-NoProfile`。プロファイルの出力が現れない)。`PSModulePath` の別の `Conductor` を読まず、ランチャーの相対のモジュールを、パスで読み込む(`Import-Module <パス>`)。**プロファイルの置き場所(`$PROFILE` の既定のパス)は、実装時に確認(未確認)**。区分: 自動 |
| AC-565 | Pester のテストの構造の点検 | `tests/**/*.Tests.ps1` | 点検(構文木) | すべてのテストのファイルに、`Describe` があり、`It` の名前が `AC-` で始まる(`tests/Unit` の `It` は、AC のない単体の項目を除く)。`-Skip` の式が `BeforeDiscovery` で作った変数だけを使う。`-Tag` が、3.9 節の一覧の中にある。`It` に `-Skip` が付いたまま放置された項目がない(スキップの理由が、AC の区分(OS・権限・`CONDUCTOR_PERF`)に対応する)。AC の ID が、この文書の一覧と重複・欠落なく対応する(一覧にあるが、テストがない ID を、`Manual`・`Inspect` 以外で 0 件にする。`build/Invoke-Checks.ps1 ac-coverage`)。区分: 点検 |
| AC-566 | テストの並列実行とステートの衝突 | 受け入れテストを、2 つの `pwsh` から同時に実行する(`Invoke-Pester` を 2 つ。`T` は別) | 同時に実行 | どちらも合格する(`T` が別なので、ステートファイルは衝突しない。`[System.IO.Path]::GetTempPath()` の下の GUID のディレクトリ)。`Slow`・`Perf` のタグは、並列にしない。区分: 自動 |
| AC-567 | Windows PowerShell 5.1 での拒否 | Windows。`powershell.exe -NoProfile -File bin\conductor.ps1`(ランチャーのスクリプトがある場合)か、`Import-Module ./src/Conductor` | 5.1 で実行 | マニフェストの `PowerShellVersion` により、読み込みが失敗する(Windows PowerShell 5.1 では動かない旨のエラー)。`conductor.cmd` は `pwsh` を使うので、5.1 にならない。区分: 手動(Windows) |
| AC-568 | PowerShell の版の行列 | `PS_MIN` と `PS_MAX` の `pwsh` | 3.11 節の `Unit`・`Inspect`・`Acceptance` を、それぞれの `pwsh` で | 両方で全項目が合格する(最小と最新が同じなら、重複して行う)。`$PSVersionTable.PSVersion` が、期待の版である(テストの最初の項目で確かめる)。区分: 自動 |
| AC-569 | 引数の文字列への取り込みと、`$args` の拒否 | `Invoke-Conductor` を、プロセス内で、(a) `-ArgumentList @('5', $null)`(`$null` の要素)、(b) `-ArgumentList $null`、(c) `-ArgumentList @()`、(d) `$args` に入る形(`Invoke-Conductor run a`。`-ArgumentList` なし)で呼ぶ | 呼ぶ | (a) 配列の `[string[]]` への束縛で、`$null` の要素は空文字列になり、`unknown command: '5'` の 68(2 要素目は位置引数)。(b) 引数なしと同じ(`missing argument`、68)。(c) 同じ。(d) `$args` は使わないので、空でない `$args` は内部の誤りとして `conductor: error: internal error` と 70(04 章 1.4 節)。落ちない。区分: 単体 |
| AC-570 | ログの出力の抑止(進捗など) | `run`・`pipeline` | `$ProgressPreference` が既定の状態で実行 | 進捗の表示(`Write-Progress`)が出ない(`Invoke-WebRequest` などを使わないので出ない。使わないことの点検)。`$VerbosePreference`・`$DebugPreference`・`$InformationPreference` が、利用者のセッションで有効でも、`conductor` の出力が変わらない(`-v` の出力は、`conductor` の自前の書き出し)。区分: 自動 |
| AC-571 | モジュールの読み込みで出力しない | `Import-Module ./src/Conductor` | 出力を捕まえる(`*>&1`) | 何も出力しない。`Conductor.psm1` のドットソースが、画面にも、パイプラインにも出力しない。承認されない動詞の警告(`Import-Module` が出す)がない(`Verb-ConductorNoun` はすべて承認された動詞。公開しない関数も、`Get-Verb` の一覧にある動詞。`PSUseApprovedVerbs`)。区分: 自動 |
| AC-572 | 公開されるものが 1 つだけ | `Import-Module ./src/Conductor` | `Get-Command -Module Conductor`、`Get-Alias`、`Get-Variable` | 公開される関数が `Invoke-Conductor` だけ。エイリアス・変数・コマンドレットを公開しない。非公開の関数(`Private/`)が、モジュールの外から見えない(`Get-Command Get-ConductorPlatformKind` が失敗する)。区分: 自動 |
| AC-573 | 開発時の依存の固定の検査 | `build/Dependencies.psd1`、`build/Install-DevDependencies.ps1` | `Install-DevDependencies.ps1` を、空の `PSModulePath` の一時ディレクトリで | 固定した版の Pester・PSScriptAnalyzer が入る(`Save-Module -RequiredVersion` で取得し、ハッシュが一致する)。ハッシュが一致しなければ、入れずに失敗する。最新の版を、黙って入れない。区分: 点検(ネットワークが要る。CI のジョブだけ) |
| AC-574 | 速い版のテスト用ジョブ | UNIX 系の `["sh", "-c", "exit 3"]`、Windows の `["cmd.exe", "/c", "exit 3"]` | `run a` | 終了コード 3(AC-109 と同じ)。`TJ`(`pwsh`)との起動時間の差を記録する(目安)。区分: 自動 |
| AC-575 | 製品のコードのスコープと状態 | ソース | 点検(構文木) | `$script:` の変数が、モジュールの読み込み時に初期化される定数と、`Add-Type` の状態だけ。関数の中で `$script:`・`$global:` に代入しない(状態は、引数と戻り値で渡す)。ドットソースされたファイルの先頭で、実行される文(関数の定義以外)がない。関数の `param()` の型指定が、すべてのパラメーターにある。区分: 点検 |

---

## 5. CI(GitHub Actions)と品質ゲート

ワークフローは `.github/workflows/` の 3 つ: `ci.yml`、`audit.yml`、`release.yml`。GitHub Actions の部品(`actions/checkout`、`actions/cache`、`actions/upload-artifact`、`actions/download-artifact`)の版は、`<版>` と書いた場所を、**実装時に、その時点の最新の安定版で確かめて決める**(供給網の注意として、タグでなく、確認した完全なコミットの ID(40 桁)で固定することを勧める)。
PowerShell の版は、`build/Versions.psd1` の `MinPowerShell`(最小)と `MaxPowerShell`(最新)で決める(2.5 節)。**PowerShell を指定の版で入れる公式の `actions/setup-*` の存在は、確認できていない(未確認)**。GitHub のランナーに `pwsh` が入っていることは、`ubuntu-24.04` のイメージで確認した(1 章。PowerShell 7.6.10)。macOS と Windows のイメージは、確認できていない(未確認。入っている見込み)。入っている `pwsh` の版が `Versions.psd1` の版と違うときだけ、`build/Resolve-Pwsh.ps1`(5.4 節)が、公式のリリースの成果物を取得して、`PATH` の先頭に足す。

### 5.1 `ci.yml`(名前 `CI`)

| 項目 | 内容 |
|------|------|
| 起動 | `push`(`main`)、`pull_request`(対象 `main`)、`workflow_call`(リリースから呼ばれる) |
| 権限 | `contents: read` |
| 同時実行 | `group: ci-${{ github.workflow }}-${{ github.ref }}`、`cancel-in-progress: true` |
| 環境変数 | `CI: "true"`、`POWERSHELL_TELEMETRY_OPTOUT: "1"`、`POWERSHELL_UPDATECHECK: "Off"`(変数名は実装時に確認。**未確認**)、`CONDUCTOR_PWSH_SLOT`(`min` か `max`。各ジョブで設定) |
| チェックアウト | `actions/checkout@<版>`(リリースの検査のジョブだけ `fetch-depth: 0`) |
| 開発時の依存 | `build/Install-DevDependencies.ps1 -Path ${{ runner.temp }}/devmodules`(固定の版の Pester・PSScriptAnalyzer を `Save-Module` で取得して、ハッシュを検査し、`PSModulePath` を `GITHUB_ENV` に足す。8 章)。`actions/cache` で `${{ runner.temp }}/devmodules` を、`hashFiles('build/Dependencies.psd1')` をキーにして保存してよい(復元しても、ハッシュの検査は毎回行う) |
| PowerShell の用意 | `pwsh ./build/Resolve-Pwsh.ps1 -Slot $env:CONDUCTOR_PWSH_SLOT`(ランナーの版が合っていれば何もしない) |

| ジョブ | 目的 | OS(時間の上限) | 版 | コマンド | 失敗の条件 |
|--------|------|-----------------|----|----------|------------|
| `lint` | 静的検査・整形・マニフェスト・禁止コマンド・依存の向き | `ubuntu-24.04`(15 分) | 最新 | 開発時の依存 → ① `./build/Invoke-Checks.ps1 -Check lint`(PSScriptAnalyzer、`forbidden`、`format`、`manifest`)② `./build/Invoke-Tests.ps1 -Kind Inspect`(依存の向きと循環、禁止の構文の点検。AC-514、AC-518、AC-420〜423) | PSScriptAnalyzer の Error・Warning(ParseError を含む)が 1 件でも、禁止コマンドが 1 件でも、整形の差分が 1 つでも、マニフェストの検査の失敗、循環・許可されない呼び出し(AC-480、AC-514、AC-518) |
| `consistency` | 版と依存の記載の整合 | `ubuntu-24.04`(5 分) | 最新 | ① `./build/Invoke-Checks.ps1 -Check version`(`ModuleVersion` が唯一の出所。ランチャーの `--version` との一致)② `-Check consistency`③ `-Check deps` | 版の不一致、依存の許可の違反(AC-476〜478、AC-562) |
| `package` | 配布用のアーカイブが作れ、中身が正しいこと | `ubuntu-24.04`(10 分) | 最新 | `./build/New-Package.ps1 -OutDir artifacts/dist` → `./build/Invoke-Checks.ps1 -Check dist -Path artifacts/dist/conductor-<版>`(展開した内容を検査)→ `actions/upload-artifact`(名前 `dist`) | アーカイブの作成の失敗、中身の違反、`Test-ModuleManifest` の失敗(AC-422、AC-479、AC-526、AC-527) |
| `test` | 3 つの OS で、単体・点検・受け入れテストが通ること | `ubuntu-24.04`、`macos-latest`、`windows-2022`(`fail-fast: false`。30 分) | 最新 | 開発時の依存 → `./build/Invoke-Tests.ps1 -Kind Unit,Inspect,Acceptance`(結果を `artifacts/test-*.xml` に出し、`actions/upload-artifact` で残す。`if: always()`) | テストの失敗(60 秒の上限を含む) |
| `min-test` | 最小の PowerShell でも、テストが通ること | `ubuntu-24.04`、`windows-2022`(`fail-fast: false`。30 分) | 最小 | `CONDUCTOR_PWSH_SLOT=min` で `Resolve-Pwsh.ps1` → `./build/Invoke-Tests.ps1 -Kind Unit,Acceptance`(最小と最新が同じなら、`test` と重複するが、そのまま行う) | テストの失敗。Windows で `.cmd` を最小の版で起動できること(AC-510)を含む |
| `dist-test` | 配布したアーカイブの中身で、受け入れテストが通ること | `ubuntu-24.04`、`macos-latest`、`windows-2022`(`fail-fast: false`。30 分) | 最新 | `package` の成果物を `actions/download-artifact` で取得 → 展開(Windows は zip、他は tar.gz) → `CONDUCTOR_BIN=<展開先>/bin/conductor`(Windows は `bin\conductor.cmd`)で `./build/Invoke-Tests.ps1 -Kind Acceptance`(AC-446)。`-Tag Dist` の項目(AC-442、AC-528) | テストの失敗 |
| `coverage` | 行カバレッジ 80% の維持 | `ubuntu-24.04`(30 分) | 最新 | 5.3 節のコマンド | カバレッジが 80% 未満 |
| `docs-check` | README に必要な語が残っていること、AC の一覧とテストの対応 | `ubuntu-24.04`(5 分) | 最新 | `./build/Invoke-Checks.ps1 -Check readme`(5.4 節。8 つの文字列)、`-Check ac-coverage`(AC-565) | 1 つでも欠けると `README.md is missing: <語>` を出して失敗。AC の対応の欠落 |
| `startup-bench` | 起動と性能の測定(2.1 節) | `ubuntu-24.04`、`macos-latest`、`windows-2022`(`fail-fast: false`。30 分) | 最新 | 開発時の依存 → `CONDUCTOR_PERF=1 STARTUP_BENCH_ENFORCE=false ./build/Invoke-Tests.ps1 -Kind Perf`。測定値は `GITHUB_STEP_SUMMARY` に出る | `continue-on-error: true` のため、ジョブが失敗しても全体は失敗にならない。基準超えは警告のみ(強制するには環境変数を `"true"` にする) |

任意のジョブ(置くかどうかは、実装時に決める。ブランチ保護の必須にはしない): `next-powershell-test`(次の LTS・プレビューの `pwsh` で `Invoke-Tests`。`continue-on-error: true`)、`merged-module`(リリース物を作るときに、関数を 1 つの `Conductor.psm1` に結合した版の起動時間を、通常の版と比べる。2.1 節の採否の判断のためだけの計測)。

`ci.yml` の骨組み(`<版>` は実装時に確認。長い定型の手順は `build/` のスクリプトに置き、ワークフローには置かない):

```
# Copyright 2026 agwlvssainokuni (Apache License 2.0 のヘッダ。全文は省略)
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  workflow_call:

permissions:
  contents: read

concurrency:
  group: ci-${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

env:
  CI: "true"
  POWERSHELL_TELEMETRY_OPTOUT: "1"
  POWERSHELL_UPDATECHECK: "Off"

defaults:
  run:
    shell: pwsh

jobs:
  lint:
    runs-on: ubuntu-24.04
    timeout-minutes: 15
    steps:
      - uses: actions/checkout@<版>
      - name: Development dependencies
        run: ./build/Install-DevDependencies.ps1 -Path "$env:RUNNER_TEMP/devmodules"
      - name: Lint
        run: ./build/Invoke-Checks.ps1 -Check lint
      - name: Inspect
        run: ./build/Invoke-Tests.ps1 -Kind Inspect

  consistency:
    runs-on: ubuntu-24.04
    timeout-minutes: 5
    steps:
      - uses: actions/checkout@<版>
      - run: ./build/Invoke-Checks.ps1 -Check version
      - run: ./build/Invoke-Checks.ps1 -Check consistency
      - run: ./build/Invoke-Checks.ps1 -Check deps

  package:
    runs-on: ubuntu-24.04
    timeout-minutes: 10
    steps:
      - uses: actions/checkout@<版>
      - name: Build package
        run: ./build/New-Package.ps1 -OutDir artifacts/dist
      - name: Verify package
        run: |
          $dir = (Get-ChildItem artifacts/dist -Directory | Select-Object -First 1).FullName
          ./build/Invoke-Checks.ps1 -Check dist -Path $dir
      - uses: actions/upload-artifact@<版>
        with:
          name: dist
          path: |
            artifacts/dist/*.zip
            artifacts/dist/*.tar.gz
            artifacts/dist/SHA256SUMS

  test:
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-24.04, macos-latest, windows-2022]
    runs-on: ${{ matrix.os }}
    timeout-minutes: 30
    env:
      CONDUCTOR_PWSH_SLOT: max
    steps:
      - uses: actions/checkout@<版>
      - name: Ensure PowerShell
        run: ./build/Resolve-Pwsh.ps1 -Slot $env:CONDUCTOR_PWSH_SLOT
      - name: Development dependencies
        run: ./build/Install-DevDependencies.ps1 -Path "$env:RUNNER_TEMP/devmodules"
      - name: Test
        run: ./build/Invoke-Tests.ps1 -Kind Unit,Inspect,Acceptance
      - uses: actions/upload-artifact@<版>
        if: always()
        with:
          name: test-${{ matrix.os }}
          path: artifacts/test-*.xml

  min-test:
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-24.04, windows-2022]
    runs-on: ${{ matrix.os }}
    timeout-minutes: 30
    env:
      CONDUCTOR_PWSH_SLOT: min
    steps:
      - uses: actions/checkout@<版>
      - run: ./build/Resolve-Pwsh.ps1 -Slot $env:CONDUCTOR_PWSH_SLOT
      - run: ./build/Install-DevDependencies.ps1 -Path "$env:RUNNER_TEMP/devmodules"
      - run: ./build/Invoke-Tests.ps1 -Kind Unit,Acceptance

  dist-test:
    needs: package
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-24.04, macos-latest, windows-2022]
    runs-on: ${{ matrix.os }}
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@<版>
      - uses: actions/download-artifact@<版>
        with:
          name: dist
          path: artifacts/dist
      - run: ./build/Install-DevDependencies.ps1 -Path "$env:RUNNER_TEMP/devmodules"
      - name: Acceptance tests against the package
        run: ./build/Invoke-Tests.ps1 -Kind Acceptance -Tag Dist -Package artifacts/dist

  coverage:
    runs-on: ubuntu-24.04
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@<版>
      - run: ./build/Install-DevDependencies.ps1 -Path "$env:RUNNER_TEMP/devmodules"
      - run: ./build/Invoke-Tests.ps1 -Kind Unit -Coverage
      - run: ./build/Invoke-Checks.ps1 -Check coverage -CoverageFile artifacts/coverage.xml -Threshold 80

  docs-check:
    runs-on: ubuntu-24.04
    timeout-minutes: 5
    steps:
      - uses: actions/checkout@<版>
      - run: ./build/Invoke-Checks.ps1 -Check readme
      - run: ./build/Invoke-Checks.ps1 -Check ac-coverage

  startup-bench:
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-24.04, macos-latest, windows-2022]
    runs-on: ${{ matrix.os }}
    timeout-minutes: 30
    continue-on-error: true
    env:
      CONDUCTOR_PERF: "1"
      STARTUP_BENCH_ENFORCE: "false"
    steps:
      - uses: actions/checkout@<版>
      - run: ./build/Install-DevDependencies.ps1 -Path "$env:RUNNER_TEMP/devmodules"
      - run: ./build/Invoke-Tests.ps1 -Kind Perf
```

(`defaults.run.shell: pwsh` は、すべての `run` を `pwsh` で動かす。`shell: pwsh` の `run` が、各コマンドの終了コードをどう扱うか(複数行のスクリプトで、途中のネイティブコマンドの失敗を見逃さないか)は、**公式の文書で確認していない(未確認)**。したがって、`run` には、`$ErrorActionPreference = 'Stop'` を前提にした 1 行の呼び出し(スクリプトが `exit` で失敗を返す)だけを書き、複数行の手順は書かない。`Install-DevDependencies.ps1` が `PSModulePath` を `GITHUB_ENV` に足すので、次のステップから Pester・PSScriptAnalyzer が使える。)

### 5.2 `audit.yml`(名前 `Audit`)

| 項目 | 内容 |
|------|------|
| 起動 | `schedule`(毎週 1 回。例: 月曜 03:00 UTC)、`workflow_dispatch`、`pull_request`(`build/Dependencies.psd1` を変えたときだけ。`paths` で絞る) |
| 権限 | `contents: read` |
| ジョブ `deps-audit` | `ubuntu-24.04`(15 分)。`./build/Install-DevDependencies.ps1`(固定の版を取得し、ハッシュを検査する。AC-573)→ `./build/Invoke-Checks.ps1 -Check deps-audit`(最新の版との差の記録。AC-481) |
| 失敗の扱い | ハッシュの不一致は失敗(供給網の異常の疑い)。最新の版との差は、警告で、マージをブロックしない。定期の実行の失敗は、通知(GitHub の標準)に任せる |

### 5.3 カバレッジ

- 測定は Linux(`coverage` ジョブ)で、`Unit` のテストだけで行う(受け入れテストで起動した子プロセスの中のコードは、数えない。3.9 節)。対象は `src/Conductor` だけ。
- 形式は JaCoCo(Pester 5 の既定。確認: 1 章。`Cobertura` は Pester 6 で使えると書かれており、この版では使わない)。出力は `artifacts/coverage.xml`。
- 閾値は、行 80%。Pester の `CoveragePercentTarget` で失敗するかは未確認なので、`Invoke-Checks coverage`(5.4 節)が、XML の `report` 直下の `counter` の `type="LINE"` の `covered`・`missed` から率を計算して判定する(`covered / (covered + missed) × 100 >= 80`)。`LINE` の `counter` がなければ、形式の想定が違うので失敗にし、メッセージで知らせる(**Pester 5 が出す JaCoCo の XML の、`report` 直下の `counter` の有無は、未確認**。ないときは、`package`・`class`・`method` の `counter` を足し合わせる実装に直す)。
- Pester 5 のカバレッジは、ブレークポイントで測る方式で(取得した文書は、Pester 6 の前の版が、ブレークポイントを使う方式だったと書く)、遅い。`Unit` のテストだけに絞る理由の 1 つ。
- 除外: `tests/`、`build/`、`bin/`、`demo/`。`Public/Invoke-Conductor.ps1` は含める。ランチャーはカバレッジの対象でない(PowerShell のコードでない場合がある)。

### 5.4 検査スクリプトの全文(`build/`)

#### `build/Versions.psd1`

```powershell
# Copyright 2026 agwlvssainokuni (Apache License 2.0 のヘッダ。全文は省略)
@{
    MinPowerShell = '7.6'
    MaxPowerShell = '7.6'
    # ランナーの pwsh の版が合わないときだけ使う、公式のリリースの成果物(版と SHA-256)。
    # 名前と ハッシュは、実装時に公式のリリースで確認して書く(この文書の作成時には確認していない)。
    PowerShellDownloads = @{
        '7.6' = @{
            Version = '7.6.6'
            Sha256  = @{
                'linux-x64' = '<実装時に確認>'
                'osx-arm64' = '<実装時に確認>'
                'win-x64'   = '<実装時に確認>'
            }
        }
    }
}
```

#### `build/Dependencies.psd1`

```powershell
# Copyright 2026 agwlvssainokuni (Apache License 2.0 のヘッダ。全文は省略)
@{
    Modules = @(
        @{ Name = 'Pester';           Version = '5.9.0';  TreeSha256 = '<実装時に確認>' }
        @{ Name = 'PSScriptAnalyzer'; Version = '1.25.0'; TreeSha256 = '<実装時に確認>' }
    )
}
```

(`Version` は、runner-images の文書(1 章)に載っていた版の例で、固定する版は実装時に確認して決める。`TreeSha256` は、`build/Update-Dependencies.ps1`(人が実行する。新しい版を `Save-Module` で取得して、ハッシュを計算して表示するだけ)で求め、人が確認して書く。)

#### `build/Install-DevDependencies.ps1`

```powershell
# Copyright 2026 agwlvssainokuni (Apache License 2.0 のヘッダ。全文は省略)
#requires -Version 7.6
[CmdletBinding()]
param(
    [string]$Path = (Join-Path $PSScriptRoot '..' 'artifacts' 'devmodules')
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Get-TreeHash {
    param([Parameter(Mandatory)][string]$Dir)
    $root = [System.IO.Path]::GetFullPath($Dir)
    $lines = [System.Collections.Generic.List[string]]::new()
    $files = [System.IO.Directory]::GetFiles($root, '*', [System.IO.SearchOption]::AllDirectories)
    [System.Array]::Sort($files, [System.StringComparer]::Ordinal)
    foreach ($f in $files) {
        if ([System.IO.Path]::GetFileName($f) -ceq 'PSGetModuleInfo.xml') { continue }   # インストール日時を含む
        $rel = [System.IO.Path]::GetRelativePath($root, $f).Replace('\', '/')
        $h = [System.Convert]::ToHexString([System.Security.Cryptography.SHA256]::HashData([System.IO.File]::ReadAllBytes($f)))
        $lines.Add($h.ToLowerInvariant() + '  ' + $rel)
    }
    $text = [string]::Join("`n", $lines) + "`n"
    $bytes = [System.Text.UTF8Encoding]::new($false).GetBytes($text)
    return [System.Convert]::ToHexString([System.Security.Cryptography.SHA256]::HashData($bytes)).ToLowerInvariant()
}

$deps = Import-PowerShellDataFile -LiteralPath (Join-Path $PSScriptRoot 'Dependencies.psd1')
$full = [System.IO.Path]::GetFullPath($Path)
[System.IO.Directory]::CreateDirectory($full) | Out-Null

foreach ($d in $deps.Modules) {
    $target = Join-Path $full $d.Name $d.Version
    if (-not [System.IO.Directory]::Exists($target)) {
        Save-Module -Name $d.Name -RequiredVersion $d.Version -Repository PSGallery -Path $full -Force
    }
    $actual = Get-TreeHash -Dir $target
    if ($actual -cne $d.TreeSha256) {
        throw "hash mismatch: $($d.Name) $($d.Version): expected $($d.TreeSha256), actual $actual"
    }
    [System.Console]::Error.WriteLine("ok: $($d.Name) $($d.Version)")
}

$sep = [System.IO.Path]::PathSeparator
$env:PSModulePath = $full + $sep + $env:PSModulePath
if ($env:GITHUB_ENV) {
    [System.IO.File]::AppendAllText($env:GITHUB_ENV, "PSModulePath=$($env:PSModulePath)`n")
}
```

(`Save-Module` は、`-Path` の下に `<名前>/<版>/` の形で保存する、と仮定している(PowerShellGet 2.x の挙動の記憶による。**未確認**。違えば `$target` の求め方を直す)。`Save-Module` の取得元は、PowerShell Gallery(TLS 1.2 以上。確認: 1 章)。`-Repository PSGallery` を明示する。)

#### `build/Resolve-Pwsh.ps1`

```powershell
# Copyright 2026 agwlvssainokuni (Apache License 2.0 のヘッダ。全文は省略)
#requires -Version 7.0
[CmdletBinding()]
param([Parameter(Mandatory)][ValidateSet('min', 'max')][string]$Slot)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$v = Import-PowerShellDataFile -LiteralPath (Join-Path $PSScriptRoot 'Versions.psd1')
$want = if ($Slot -ceq 'min') { $v.MinPowerShell } else { $v.MaxPowerShell }
$cur = $PSVersionTable.PSVersion
if (('{0}.{1}' -f $cur.Major, $cur.Minor) -ceq $want) {
    [System.Console]::Error.WriteLine("pwsh $cur already matches $want")
    return
}
$dl = $v.PowerShellDownloads[$want]
$ver = $dl.Version
$rid = if ($IsWindows) { 'win-x64' } elseif ($IsMacOS) { 'osx-arm64' } else { 'linux-x64' }
$ext = if ($IsWindows) { 'zip' } else { 'tar.gz' }
$name = if ($IsWindows) { "PowerShell-$ver-$rid.$ext" } else { "powershell-$ver-$rid.$ext" }   # 成果物の名前は、実装時に公式のリリースで確認(未確認)
$url = "https://github.com/PowerShell/PowerShell/releases/download/v$ver/$name"
$dest = Join-Path $env:RUNNER_TEMP "pwsh-$ver"
[System.IO.Directory]::CreateDirectory($dest) | Out-Null
$archive = Join-Path $dest $name
Invoke-WebRequest -Uri $url -OutFile $archive
$actual = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
if ($actual -cne $dl.Sha256[$rid]) { throw "hash mismatch: $name" }
if ($IsWindows) { Expand-Archive -LiteralPath $archive -DestinationPath $dest -Force }
else {
    tar -xzf $archive -C $dest
    if ($LASTEXITCODE -ne 0) { throw 'tar failed' }
    chmod +x (Join-Path $dest 'pwsh')
}
[System.IO.File]::AppendAllText($env:GITHUB_PATH, $dest + "`n")
[System.Console]::Error.WriteLine("installed pwsh $ver to $dest")
```

(これはビルド用のスクリプトで、製品のコードではないので、`Invoke-WebRequest` などを使ってよい。次のステップから、`PATH` の先頭の `pwsh` が新しい版になる。ステップをまたいで、`shell: pwsh` が新しい `pwsh` を使うか(ランナーが `pwsh` を `PATH` から探すか)は、**未確認**。使わない場合は、後続の `run` を `shell: bash` にして、`pwsh -NoProfile -File ...` を直接書く。)

#### `build/Invoke-Tests.ps1`

```powershell
# Copyright 2026 agwlvssainokuni (Apache License 2.0 のヘッダ。全文は省略)
#requires -Version 7.6
[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidateSet('Unit', 'Inspect', 'Acceptance', 'Perf')][string[]]$Kind,
    [string[]]$Tag = @(),
    [switch]$Coverage,
    [string]$Package
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$pin = (Import-PowerShellDataFile -LiteralPath (Join-Path $PSScriptRoot 'Dependencies.psd1')).Modules | Where-Object { $_.Name -ceq 'Pester' }
Import-Module Pester -RequiredVersion $pin.Version -ErrorAction Stop    # ランナーに入っている別の版を使わない

[System.IO.Directory]::CreateDirectory((Join-Path $root 'artifacts')) | Out-Null
$exit = 0
foreach ($k in $Kind) {
    $cfg = New-PesterConfiguration
    $cfg.Run.Path = [string[]]@(Join-Path $root 'tests' $(if ($k -ceq 'Perf') { 'Perf' } else { $k }))
    $cfg.Run.Exit = $false
    $cfg.Run.PassThru = $true
    $cfg.Output.Verbosity = if ($env:CI) { 'Detailed' } else { 'Normal' }
    $cfg.TestResult.Enabled = $true
    $cfg.TestResult.OutputFormat = 'NUnitXml'
    $cfg.TestResult.OutputPath = Join-Path $root 'artifacts' ('test-{0}-{1}.xml' -f $k, [System.Runtime.InteropServices.RuntimeInformation]::OSDescription.Split(' ')[0])
    $exclude = [System.Collections.Generic.List[string]]::new()
    if ($k -ceq 'Acceptance') { $exclude.Add('Perf') }
    if ($Tag.Count -gt 0) { $cfg.Filter.Tag = [string[]]$Tag }
    if ($exclude.Count -gt 0) { $cfg.Filter.ExcludeTag = $exclude.ToArray() }
    if ($Package) { $env:CONDUCTOR_DIST_DIR = [System.IO.Path]::GetFullPath($Package) }
    if ($Coverage -and $k -ceq 'Unit') {
        $cfg.CodeCoverage.Enabled = $true
        $cfg.CodeCoverage.Path = [string[]]@(Join-Path $root 'src' 'Conductor')
        $cfg.CodeCoverage.OutputFormat = 'JaCoCo'
        $cfg.CodeCoverage.OutputPath = Join-Path $root 'artifacts' 'coverage.xml'
    }
    $result = Invoke-Pester -Configuration $cfg
    if ($result.FailedCount -gt 0 -or $result.FailedBlocksCount -gt 0 -or $result.FailedContainersCount -gt 0) { $exit = 1 }
}
exit $exit
```

(`PesterConfiguration` のキー名(`Run.PassThru`・`Run.Exit`・`Output.Verbosity`・`TestResult.*`・`Filter.*`・`CodeCoverage.*`)は、1 章で確認した範囲。`Result` オブジェクトの `FailedCount` などの名前は、**未確認**(実装時に確認)。タグ `Dist` の項目は、`CONDUCTOR_DIST_DIR` があるときだけ実行し、なければ `-Skip`。種別ごとに結果ファイルを分ける。`-Tag Dist` を渡す呼び出しは、`Filter.Tag` で絞る。)

#### `build/Invoke-Checks.ps1`

```powershell
# Copyright 2026 agwlvssainokuni (Apache License 2.0 のヘッダ。全文は省略)
#requires -Version 7.6
[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [ValidateSet('lint', 'forbidden', 'format', 'manifest', 'version', 'consistency', 'deps', 'deps-audit', 'readme', 'coverage', 'dist', 'ac-coverage')]
    [string]$Check,
    [string]$Path,
    [string]$Tag,
    [string]$CoverageFile = 'artifacts/coverage.xml',
    [double]$Threshold = 80.0
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$Root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$Ast = [System.Management.Automation.Language.Parser]
$Failures = [System.Collections.Generic.List[string]]::new()
if ($Check -cin @('lint', 'format')) {
    $pin = (Import-PowerShellDataFile -LiteralPath (Join-Path $Root 'build' 'Dependencies.psd1')).Modules | Where-Object { $_.Name -ceq 'PSScriptAnalyzer' }
    Import-Module PSScriptAnalyzer -RequiredVersion $pin.Version -ErrorAction Stop
}

function Add-Failure([string]$Message) { $Failures.Add($Message) }
function Write-Info([string]$Message) { [System.Console]::Error.WriteLine($Message) }
function Get-SourceFiles([string[]]$Dirs, [string[]]$Patterns) {
    $list = [System.Collections.Generic.List[string]]::new()
    foreach ($d in $Dirs) {
        $full = Join-Path $Root $d
        if (-not [System.IO.Directory]::Exists($full)) { continue }
        foreach ($p in $Patterns) {
            foreach ($f in [System.IO.Directory]::GetFiles($full, $p, [System.IO.SearchOption]::AllDirectories)) { $list.Add($f) }
        }
    }
    return , $list.ToArray()
}
function Get-Manifest { return Import-PowerShellDataFile -LiteralPath (Join-Path $Root 'src' 'Conductor' 'Conductor.psd1') }
function Get-Versions { return Import-PowerShellDataFile -LiteralPath (Join-Path $Root 'build' 'Versions.psd1') }

# ---- forbidden: 禁止コマンド・禁止の書き方(構文木)---------------------------------
$BannedCommands = @(
    'invoke-expression', 'iex', 'write-host', 'write-output', 'echo', 'write', 'write-error', 'write-warning', 'write-verbose',
    'write-information', 'write-progress', 'out-host', 'out-default', 'read-host', 'start-process', 'saps', 'start', 'start-sleep', 'sleep',
    'get-content', 'gc', 'cat', 'type', 'set-content', 'sc', 'add-content', 'ac', 'out-file', 'remove-item', 'rm', 'del', 'ri', 'rd',
    'move-item', 'mv', 'move', 'mi', 'new-item', 'ni', 'test-path', 'resolve-path', 'rvpa', 'join-path', 'convertfrom-json', 'convertto-json',
    'import-powershelldatafile', 'import-clixml', 'export-clixml', 'invoke-webrequest', 'iwr', 'curl', 'wget', 'invoke-restmethod', 'irm',
    'test-connection', 'install-module', 'update-help', 'set-executionpolicy', 'set-location', 'cd', 'sl', 'chdir', 'push-location', 'pushd',
    'pop-location', 'popd', 'start-job', 'start-threadjob', 'register-objectevent', 'register-engineevent', 'new-object', 'import-module', 'ipmo'
)
$AddTypeAllowed = @('src/Conductor/Private/Runner/NativeHelper.ps1')
$DotSourceAllowed = @('src/Conductor/Conductor.psm1')   # モジュールの読み込みだけが、ファイルをドットソースしてよい
$ImportDataFileAllowed = @('src/Conductor/Conductor.psm1')   # 版をマニフェストから 1 回だけ読む 1 か所(02 章 10 章)
$BannedBinary = @('Imatch', 'Inotmatch', 'Cmatch', 'Cnotmatch', 'Ireplace', 'Creplace', 'Isplit', 'Csplit', 'Ilike', 'Inotlike', 'Clike', 'Cnotlike')
$CaseInsensitiveCompare = @('Ieq', 'Ine', 'Ilt', 'Igt', 'Ile', 'Ige')
$BannedStaticMembers = @('Parse', 'Format', 'Compare')
$BannedInstanceNoArg = @('ToUpper', 'ToLower', 'ToString')

function Test-Forbidden([string[]]$Files, [string]$Label) {
    foreach ($file in $Files) {
        $rel = [System.IO.Path]::GetRelativePath($Root, $file).Replace('\', '/')
        $tokens = $null; $errors = $null
        $tree = $Ast::ParseFile($file, [ref]$tokens, [ref]$errors)
        foreach ($e in $errors) { Add-Failure "${rel}:$($e.Extent.StartLineNumber): parse error ($($e.ErrorId))" }
        $nodes = $tree.FindAll({ param($n) $true }, $true)
        foreach ($n in $nodes) {
            $line = $n.Extent.StartLineNumber
            switch ($n.GetType().Name) {
                'CommandAst' {
                    $name = $n.GetCommandName()
                    if ($null -eq $name) {
                        if ($n.InvocationOperator -ne 'Unknown' -and $DotSourceAllowed -cnotcontains $rel) { Add-Failure "${rel}:${line}: dynamic command ('&' or '.' with an expression)" }
                        continue
                    }
                    $lower = $name.ToLowerInvariant()
                    if ($lower -ceq 'add-type') {
                        if ($AddTypeAllowed -cnotcontains $rel) { Add-Failure "${rel}:${line}: Add-Type is allowed only in $($AddTypeAllowed -join ', ')" }
                    } elseif ($lower -ceq 'import-powershelldatafile' -and $ImportDataFileAllowed -ccontains $rel) {
                        continue
                    } elseif ($BannedCommands -ccontains $lower) {
                        Add-Failure "${rel}:${line}: banned command '$name'"
                    }
                }
                'ExitStatementAst' { Add-Failure "${rel}:${line}: 'exit' is allowed only in the launcher" }
                'TrapStatementAst' { Add-Failure "${rel}:${line}: 'trap' is not allowed (use try/catch/finally)" }
                'UsingStatementAst' { Add-Failure "${rel}:${line}: 'using' statement is not allowed" }
                'BinaryExpressionAst' {
                    $op = $n.Operator.ToString()
                    if ($BannedBinary -ccontains $op) { Add-Failure "${rel}:${line}: banned operator ($op)" }
                    if ($CaseInsensitiveCompare -ccontains $op) {
                        if ($n.Left -is [System.Management.Automation.Language.StringConstantExpressionAst] -or
                            $n.Right -is [System.Management.Automation.Language.StringConstantExpressionAst] -or
                            $n.Left -is [System.Management.Automation.Language.ExpandableStringExpressionAst] -or
                            $n.Right -is [System.Management.Automation.Language.ExpandableStringExpressionAst]) {
                            Add-Failure "${rel}:${line}: case-insensitive comparison with a string (use -ceq / -cne or [string]::Equals(..., Ordinal))"
                        }
                    }
                }
                'VariableExpressionAst' {
                    $vp = $n.VariablePath
                    if ($vp.IsGlobal) { Add-Failure "${rel}:${line}: global variable" }
                    $un = $vp.UserPath
                    if ($un -ceq 'LASTEXITCODE' -or $un -ceq '?') { Add-Failure "${rel}:${line}: `$$un must not be read" }
                }
                'InvokeMemberExpressionAst' {
                    $member = $n.Member
                    if ($member -is [System.Management.Automation.Language.StringConstantExpressionAst]) {
                        $m = $member.Value
                        $argc = if ($null -eq $n.Arguments) { 0 } else { $n.Arguments.Count }
                        if ($n.Static -and ($n.Expression -is [System.Management.Automation.Language.TypeExpressionAst]) -and ($BannedStaticMembers -ccontains $m) -and $argc -le 2) {
                            Add-Failure "${rel}:${line}: culture-dependent static member '$m' (use the overload with CultureInfo / StringComparison)"
                        }
                        if ((-not $n.Static) -and ($BannedInstanceNoArg -ccontains $m) -and $argc -eq 0) {
                            Add-Failure "${rel}:${line}: culture-dependent member '$m()' (use ToUpperInvariant / ToString(CultureInfo.InvariantCulture))"
                        }
                    }
                }
                'TypeExpressionAst' {
                    $fn = $n.TypeName.FullName
                    if ($fn.StartsWith('System.Net.', [System.StringComparison]::OrdinalIgnoreCase) -or $fn -ieq 'System.Reflection.Assembly') {
                        Add-Failure "${rel}:${line}: banned type [$fn]"
                    }
                }
                'AttributeAst' {
                    if ($n.TypeName.Name -ieq 'SuppressMessage' -or $n.TypeName.Name -ieq 'SuppressMessageAttribute') {
                        Add-Failure "${rel}:${line}: rule suppression (list it in the allow-list with a reason)"
                    }
                }
            }
        }
    }
}

function Invoke-Forbidden {
    if ($Path) { $files = Get-SourceFiles @($Path) @('*.ps1', '*.psm1') } else { $files = Get-SourceFiles @('src') @('*.ps1', '*.psm1') }
    Test-Forbidden -Files $files -Label 'src'
}

# ---- lint: PSScriptAnalyzer + forbidden + format + manifest -------------------------
function Invoke-Lint {
    $settings = Join-Path $Root 'PSScriptAnalyzerSettings.psd1'
    foreach ($d in @('src', 'build', 'tests', 'demo')) {
        $dir = Join-Path $Root $d
        if (-not [System.IO.Directory]::Exists($dir)) { continue }
        $r = Invoke-ScriptAnalyzer -Path $dir -Recurse -Settings $settings
        foreach ($x in $r) {
            if ($x.Severity -in @('Error', 'Warning', 'ParseError')) {
                Add-Failure "$($x.ScriptPath):$($x.Line): [$($x.RuleName)] $($x.Message)"
            }
        }
    }
    Invoke-Forbidden
    Invoke-Format
    Invoke-Manifest
}

function Invoke-Format {
    $settings = Join-Path $Root 'PSScriptAnalyzerSettings.psd1'
    $files = Get-SourceFiles @('src', 'build', 'tests', 'demo') @('*.ps1', '*.psm1', '*.psd1')
    foreach ($f in $files) {
        $orig = [System.IO.File]::ReadAllText($f).Replace("`r`n", "`n")
        $fmt = (Invoke-Formatter -ScriptDefinition $orig -Settings $settings).Replace("`r`n", "`n")
        if ($orig -cne $fmt) { Add-Failure "$([System.IO.Path]::GetRelativePath($Root, $f)): formatting differs (run Invoke-Formatter)" }
    }
}

function Invoke-Manifest {
    $psd1 = Join-Path $Root 'src' 'Conductor' 'Conductor.psd1'
    $warn = $null
    $null = Test-ModuleManifest -Path $psd1 -ErrorAction Stop -WarningVariable warn
    if ($warn) { Add-Failure "Test-ModuleManifest warnings: $($warn -join '; ')" }
    $m = Get-Manifest
    if (@($m.FunctionsToExport).Count -ne 1 -or $m.FunctionsToExport[0] -cne 'Invoke-Conductor') { Add-Failure 'FunctionsToExport must be exactly @(''Invoke-Conductor'')' }
    foreach ($k in 'RequiredModules', 'ExternalModuleDependencies', 'NestedModules') {
        if ($m.ContainsKey($k) -and @($m[$k]).Count -gt 0) { Add-Failure "$k must be empty" }
    }
}

# ---- version / consistency / deps ------------------------------------------------------
function Get-LauncherVersionOutput {
    $launcher = if ($IsWindows) { Join-Path $Root 'bin' 'conductor.cmd' } else { Join-Path $Root 'bin' 'conductor' }
    $psi = [System.Diagnostics.ProcessStartInfo]::new($launcher)
    $psi.ArgumentList.Add('--version')
    $psi.RedirectStandardOutput = $true
    $psi.UseShellExecute = $false
    $p = [System.Diagnostics.Process]::Start($psi)
    $out = $p.StandardOutput.ReadToEnd()
    $p.WaitForExit()
    return $out.TrimEnd("`n")
}

function Invoke-Version {
    $ver = (Get-Manifest).ModuleVersion
    $parts = $ver.Split('.')
    $ok = ($parts.Length -eq 3)
    foreach ($p in $parts) { foreach ($c in $p.ToCharArray()) { if ($c -lt '0' -or $c -gt '9') { $ok = $false } } }
    if (-not $ok) { Add-Failure "ModuleVersion '$ver' must be three numbers" }
    if ($Tag) {
        $t = if ($Tag.StartsWith('v', [System.StringComparison]::Ordinal)) { $Tag.Substring(1) } else { $Tag }
        $core = $t.Split('-')[0]
        if ($core -cne $ver) { Add-Failure "tag '$Tag' does not match ModuleVersion '$ver'" }
    }
    $out = Get-LauncherVersionOutput
    if ($out -cne "conductor $ver") { Add-Failure "launcher --version printed '$out', expected 'conductor $ver'" }
}

function Invoke-Consistency {
    $v = Get-Versions
    $m = Get-Manifest
    $min = [version]$v.MinPowerShell; $max = [version]$v.MaxPowerShell
    if ($min -gt $max) { Add-Failure 'MinPowerShell must not exceed MaxPowerShell' }
    if ([version]$m.PowerShellVersion -ne $min) { Add-Failure "PowerShellVersion '$($m.PowerShellVersion)' must equal MinPowerShell '$($v.MinPowerShell)'" }
    foreach ($f in (Get-SourceFiles @('src', 'build') @('*.ps1', '*.psm1'))) {
        $tokens = $null; $errors = $null
        $tree = $Ast::ParseFile($f, [ref]$tokens, [ref]$errors)
        $req = $tree.ScriptRequirements
        if ($null -eq $req -or $null -eq $req.RequiredPSVersion) { Add-Failure "$([System.IO.Path]::GetRelativePath($Root, $f)): #requires -Version is missing"; continue }
        if ($req.RequiredPSVersion -lt $min) { Add-Failure "$([System.IO.Path]::GetRelativePath($Root, $f)): #requires -Version is lower than MinPowerShell" }
    }
    $readme = [System.IO.File]::ReadAllText((Join-Path $Root 'README.md'))
    if (-not $readme.Contains("PowerShell $($v.MinPowerShell)", [System.StringComparison]::Ordinal)) { Add-Failure "README.md must mention 'PowerShell $($v.MinPowerShell)'" }
    $ci = [System.IO.File]::ReadAllText((Join-Path $Root '.github' 'workflows' 'ci.yml'))
    if (-not $ci.Contains('Resolve-Pwsh.ps1', [System.StringComparison]::Ordinal)) { Add-Failure 'ci.yml must use build/Resolve-Pwsh.ps1' }
}

function Invoke-Deps {
    $d = Import-PowerShellDataFile -LiteralPath (Join-Path $Root 'build' 'Dependencies.psd1')
    $allowed = @('Pester', 'PSScriptAnalyzer')
    foreach ($x in $d.Modules) {
        if ($allowed -cnotcontains $x.Name) { Add-Failure "dependency '$($x.Name)' is not in the allow-list" }
        if ($x.Version -cnotmatch '^\d+\.\d+\.\d+$') { Add-Failure "dependency '$($x.Name)': Version must be exact (x.y.z)" }
        if ($x.TreeSha256 -cnotmatch '^[0-9a-f]{64}$') { Add-Failure "dependency '$($x.Name)': TreeSha256 must be 64 lowercase hex digits" }
    }
    foreach ($f in (Get-SourceFiles @('.github') @('*.yml'))) {
        $t = [System.IO.File]::ReadAllText($f)
        if ($t.Contains('Install-Module', [System.StringComparison]::Ordinal)) { Add-Failure "$([System.IO.Path]::GetRelativePath($Root, $f)): use build/Install-DevDependencies.ps1, not Install-Module" }
    }
    foreach ($f in (Get-SourceFiles @('src') @('*.ps1', '*.psm1'))) {
        $t = [System.IO.File]::ReadAllText($f)
        if ($t.Contains('#requires -Modules', [System.StringComparison]::OrdinalIgnoreCase)) { Add-Failure "$([System.IO.Path]::GetRelativePath($Root, $f)): #requires -Modules is not allowed" }
    }
}

function Invoke-DepsAudit {
    $d = Import-PowerShellDataFile -LiteralPath (Join-Path $Root 'build' 'Dependencies.psd1')
    foreach ($x in $d.Modules) {
        $latest = (Find-Module -Name $x.Name -Repository PSGallery).Version
        Write-Info "$($x.Name): pinned $($x.Version), latest $latest"
    }
}

# ---- readme / ac-coverage --------------------------------------------------------------
function Invoke-Readme {
    $readme = [System.IO.File]::ReadAllText((Join-Path $Root 'README.md'))
    $required = @('pwsh', 'Unblock-File', 'Get-ExecutionPolicy', 'conductor.psd1', '--run-id', 'exit_codes', 'SHA256SUMS', 'Authenticode')
    foreach ($w in $required) {
        if (-not $readme.Contains($w, [System.StringComparison]::Ordinal)) { Add-Failure "README.md is missing: $w" }
    }
}

function Invoke-AcCoverage {
    $doc = [System.IO.File]::ReadAllLines((Join-Path $Root 'docs' 'spec' 'powershell' '11-quality-and-build.md'))
    $need = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal)
    foreach ($l in $doc) {
        if ($l.StartsWith('| AC-', [System.StringComparison]::Ordinal) -and -not $l.Contains('区分: 手動', [System.StringComparison]::Ordinal)) {
            $null = $need.Add($l.Substring(2, $l.IndexOf(' ', 2, [System.StringComparison]::Ordinal) - 2))
        }
    }
    $have = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal)
    foreach ($f in (Get-SourceFiles @('tests') @('*.Tests.ps1'))) {
        $t = [System.IO.File]::ReadAllText($f)
        foreach ($m in [regex]::Matches($t, 'AC-\d{3}', [System.Text.RegularExpressions.RegexOptions]::CultureInvariant)) { $null = $have.Add($m.Value) }
    }
    foreach ($id in $need) { if (-not $have.Contains($id)) { Add-Failure "$id has no test" } }
}

# ---- coverage ----------------------------------------------------------------------------
function Invoke-Coverage {
    $file = [System.IO.Path]::GetFullPath((Join-Path $Root $CoverageFile))
    if (-not [System.IO.File]::Exists($file)) { Add-Failure "coverage file not found: $file"; return }
    $s = [System.Xml.XmlReaderSettings]::new()
    $s.DtdProcessing = [System.Xml.DtdProcessing]::Ignore
    $s.XmlResolver = $null
    $reader = [System.Xml.XmlReader]::Create($file, $s)
    try { $doc = [System.Xml.XmlDocument]::new(); $doc.Load($reader) } finally { $reader.Dispose() }
    $node = $doc.SelectSingleNode('/report/counter[@type="LINE"]')
    if ($null -eq $node) { Add-Failure 'no LINE counter under /report (the JaCoCo layout written by Pester is unconfirmed; adjust)'; return }
    $covered = [double]::Parse($node.GetAttribute('covered'), [System.Globalization.CultureInfo]::InvariantCulture)
    $missed = [double]::Parse($node.GetAttribute('missed'), [System.Globalization.CultureInfo]::InvariantCulture)
    $total = $covered + $missed
    if ($total -le 0) { Add-Failure 'no lines were measured'; return }
    $pct = $covered * 100.0 / $total
    Write-Info ('line coverage: {0:F2}% ({1} / {2}), threshold {3}%' -f $pct, $covered, $total, $Threshold)
    if ($pct -lt $Threshold) { Add-Failure ('line coverage {0:F2}% is below {1}%' -f $pct, $Threshold) }
}

# ---- dist ----------------------------------------------------------------------------------
function Invoke-Dist {
    if (-not $Path) { Add-Failure '-Path <extracted package directory> is required'; return }
    $dir = [System.IO.Path]::GetFullPath($Path)
    $allowedTop = @('bin', 'Conductor', 'LICENSE', 'README.md')
    foreach ($e in [System.IO.Directory]::GetFileSystemEntries($dir)) {
        $n = [System.IO.Path]::GetFileName($e)
        if ($allowedTop -cnotcontains $n) { Add-Failure "unexpected top-level entry: $n" }
    }
    foreach ($bad in '*.Tests.ps1', 'PSScriptAnalyzerSettings.psd1', 'Dependencies.psd1', 'Versions.psd1') {
        foreach ($f in [System.IO.Directory]::GetFiles($dir, $bad, [System.IO.SearchOption]::AllDirectories)) { Add-Failure "must not be packaged: $f" }
    }
    $launcher = Join-Path $dir 'bin' 'conductor'
    if (-not [System.IO.File]::Exists($launcher)) { Add-Failure 'bin/conductor is missing' }
    else {
        $head = [System.IO.File]::ReadAllBytes($launcher)
        if ($head.Length -lt 2 -or $head[0] -ne 0x23 -or $head[1] -ne 0x21) { Add-Failure 'bin/conductor must start with #!' }
        if (-not $IsWindows) {
            $mode = [System.IO.File]::GetUnixFileMode($launcher)
            if (($mode -band [System.IO.UnixFileMode]::UserExecute) -eq 0) { Add-Failure 'bin/conductor is not executable' }
        }
    }
    if (-not [System.IO.File]::Exists((Join-Path $dir 'bin' 'conductor.cmd'))) { Add-Failure 'bin/conductor.cmd is missing' }
    $null = Test-ModuleManifest -Path (Join-Path $dir 'Conductor' 'Conductor.psd1') -ErrorAction Stop
}

switch ($Check) {
    'lint' { Invoke-Lint }
    'forbidden' { Invoke-Forbidden }
    'format' { Invoke-Format }
    'manifest' { Invoke-Manifest }
    'version' { Invoke-Version }
    'consistency' { Invoke-Consistency }
    'deps' { Invoke-Deps }
    'deps-audit' { Invoke-DepsAudit }
    'readme' { Invoke-Readme }
    'coverage' { Invoke-Coverage }
    'dist' { Invoke-Dist }
    'ac-coverage' { Invoke-AcCoverage }
}
if ($Failures.Count -gt 0) {
    foreach ($f in $Failures) { Write-Info $f }
    Write-Info "$Check: $($Failures.Count) problem(s)"
    exit 1
}
Write-Info "$Check: ok"
exit 0
```

(これは検査の道具であり、製品のコードではないので、`Write-Host` の代わりに `[Console]::Error.WriteLine` を使うほかは、PowerShell の標準のコマンドレット(`Import-PowerShellDataFile`、`Invoke-ScriptAnalyzer`、`Invoke-Formatter`、`Test-ModuleManifest`、`Find-Module`)を使ってよい。**この文書の作成時には実行していない(`pwsh` がないため)。実装時に最初に実行して、構文・引数・戻り値の型・`TokenKind` の列挙子の名前(`Ieq` など)・`InvocationOperator` の比較(列挙子 `Unknown` の名前)を確かめる**。`Invoke-Formatter` の結果に末尾の改行の差が出るときは、比較の前に、末尾の改行をそろえる。`Invoke-ScriptAnalyzer -Settings` の `Severity` に `ParseError` を含める書き方は、1 章で確認した範囲(`Severity` に `ParseError` を含めないと、構文の誤りは抑止される)。`$x.Severity -in @(...)` は大文字小文字を区別しないが、ビルド用の検査では許す。)

#### `build/New-Package.ps1`

```powershell
# Copyright 2026 agwlvssainokuni (Apache License 2.0 のヘッダ。全文は省略)
#requires -Version 7.6
[CmdletBinding()]
param([Parameter(Mandatory)][string]$OutDir)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$ver = (Import-PowerShellDataFile -LiteralPath (Join-Path $root 'src' 'Conductor' 'Conductor.psd1')).ModuleVersion
$out = [System.IO.Path]::GetFullPath($OutDir)
$name = "conductor-$ver"
$stage = Join-Path $out $name
if ([System.IO.Directory]::Exists($stage)) { [System.IO.Directory]::Delete($stage, $true) }
foreach ($d in @('bin', 'Conductor')) { [System.IO.Directory]::CreateDirectory((Join-Path $stage $d)) | Out-Null }

Copy-Item -Path (Join-Path $root 'src' 'Conductor' '*') -Destination (Join-Path $stage 'Conductor') -Recurse
Copy-Item -LiteralPath (Join-Path $root 'bin' 'conductor') -Destination (Join-Path $stage 'bin' 'conductor')
Copy-Item -LiteralPath (Join-Path $root 'bin' 'conductor.cmd') -Destination (Join-Path $stage 'bin' 'conductor.cmd')
Copy-Item -LiteralPath (Join-Path $root 'LICENSE') -Destination $stage
Copy-Item -LiteralPath (Join-Path $root 'README.md') -Destination $stage
if (-not $IsWindows) {
    [System.IO.File]::SetUnixFileMode((Join-Path $stage 'bin' 'conductor'),
        [System.IO.UnixFileMode]'UserRead, UserWrite, UserExecute, GroupRead, GroupExecute, OtherRead, OtherExecute')
}

$zip = Join-Path $out "$name.zip"
$tgz = Join-Path $out "$name.tar.gz"
foreach ($f in @($zip, $tgz)) { if ([System.IO.File]::Exists($f)) { [System.IO.File]::Delete($f) } }
[System.IO.Compression.ZipFile]::CreateFromDirectory($stage, $zip, [System.IO.Compression.CompressionLevel]::Optimal, $true)
tar -czf $tgz -C $out $name
if ($LASTEXITCODE -ne 0) { throw 'tar failed' }

$lines = foreach ($f in @($zip, $tgz)) {
    $h = [System.Convert]::ToHexString([System.Security.Cryptography.SHA256]::HashData([System.IO.File]::ReadAllBytes($f))).ToLowerInvariant()
    $h + '  ' + [System.IO.Path]::GetFileName($f)
}
[System.IO.File]::WriteAllText((Join-Path $out 'SHA256SUMS'), ([string]::Join("`n", $lines) + "`n"), [System.Text.UTF8Encoding]::new($false))
```

(`[System.IO.Compression.ZipFile]` は、`Add-Type -AssemblyName System.IO.Compression.FileSystem` が要る場合がある(**未確認**)。`CreateFromDirectory` の第 4 引数 `includeBaseDirectory = $true` で、最上位が `conductor-<版>/` になる(記憶による。**未確認**)。zip は UNIX の実行権限を保たない場合があるので、UNIX 系の利用者には tar.gz を勧める(6.1 節)。`tar` は、ランナーに入っている(Windows のランナーには `tar.exe` がある。記憶による。未確認)。`bin/conductor.cmd` の改行は、`.gitattributes` の `*.cmd text eol=crlf` で CRLF になる。)

### 5.5 品質ゲートの再現手順(ローカル)

`pwsh`(`MinPowerShell` 以上)だけがあれば、次の順で、CI と同じ検査を再現できる。

```powershell
git clone <リポジトリ> ; cd <リポジトリ>
./build/Install-DevDependencies.ps1                                  # 固定の Pester・PSScriptAnalyzer(ハッシュ検査つき)
./build/Invoke-Checks.ps1 -Check lint                                # AC-480
./build/Invoke-Checks.ps1 -Check version                             # AC-476、AC-562
./build/Invoke-Checks.ps1 -Check consistency                         # AC-477
./build/Invoke-Checks.ps1 -Check deps                                # AC-478
./build/Invoke-Tests.ps1 -Kind Unit,Inspect,Acceptance               # AC-001〜、3 OS で
./build/Invoke-Tests.ps1 -Kind Unit -Coverage
./build/Invoke-Checks.ps1 -Check coverage                            # 行 80%
./build/New-Package.ps1 -OutDir artifacts/dist
./build/Invoke-Checks.ps1 -Check dist -Path artifacts/dist/conductor-<版>   # AC-526
./build/Invoke-Checks.ps1 -Check readme
./build/Invoke-Checks.ps1 -Check ac-coverage
$env:CONDUCTOR_PERF = '1'; ./build/Invoke-Tests.ps1 -Kind Perf       # 2.1 節
```

品質ゲートと AC の対応: 静的検査 = AC-480・AC-483・AC-518、依存の向き = AC-514、版 = AC-476・AC-477・AC-562、依存 = AC-478・AC-481・AC-573、パッケージ = AC-439〜445・AC-526・AC-527、3 OS = AC-446、性能 = AC-488 など(目安)。

---

## 6. リリース

### 6.1 配布の形

| 項目 | 内容 |
|------|------|
| 成果物 | `conductor-<版>.zip`(Windows 向け。UNIX 系でも展開できる)、`conductor-<版>.tar.gz`(UNIX 系向け。実行権限が保たれる)、`SHA256SUMS`(2 つのアーカイブの SHA-256。`sha256sum -c` の形式) |
| 中身 | 最上位に `conductor-<版>/` の 1 つのディレクトリ。その中: `bin/conductor`(UNIX 系のランチャー)、`bin/conductor.cmd`(Windows のランチャー)、`bin/conductor.ps1`(入口スクリプト)、`Conductor/`(モジュール。`Conductor.psd1`、`Conductor.psm1`、`Public/`、`Private/`)、`LICENSE`、`README.md`。入口スクリプトが、モジュールを、自分の位置からの相対(`../Conductor`。なければ、リポジトリの中の配置の `../src/Conductor`)で見つける([04](04-cli.md) 1.3.4 節) |
| OS ごとの分割 | しない(PowerShell のモジュールは、OS に依らない。OS ごとの違いは、実行時に判定する) |
| 同梱しないもの | PowerShell 自体、Pester・PSScriptAnalyzer、`tests/`・`build/`・`demo/`・`docs/`・`.github/` |
| 利用者が用意するもの | `pwsh`(`MinPowerShell` 以上。2.5 節)。`PATH` から `pwsh` が見つかること |
| PowerShell Gallery への公開 | **ワークフローは行わない。人が手で行う**(`Publish-Module`。API キーは人が持ち、ワークフローの秘密情報に置かない)。ギャラリーに出すのはモジュール(`Conductor/`)だけで、`bin/` のランチャー(コマンド名 `conductor`)は含まれず、`Invoke-Conductor` として使うことになる(この配布の形の扱いは未検討。10 章) |

### 6.2 ビルドの方法(決め)

- コンパイルはない。`build/New-Package.ps1`(5.4 節)が、`src/Conductor/` を `Conductor/` として、`bin/`・`LICENSE`・`README.md` と一緒に、`artifacts/dist/conductor-<版>/` に写し、zip と tar.gz にして、`SHA256SUMS` を作る。
- 版は、`src/Conductor/Conductor.psd1` の `ModuleVersion` から読む(唯一の出所。AC-476)。
- 再現可能なビルド(同じ入力から同じバイト列)は、要件にしない。zip・tar のファイルの更新時刻、並び順が、実行ごとに違ってよい(`SHA256SUMS` は、作ったアーカイブのハッシュ)。再現を求める場合の手段(`tar --sort=name --mtime=...`、zip の時刻の固定)は、未検討(10 章)。
- 関数を 1 つの `Conductor.psm1` に結合する案(起動時間の短縮。2.1 節)は、この版では採らない(採否は、測ってから決める)。
- Authenticode 署名は行わない(6.5 節)。

### 6.3 リリースの手順と `release.yml`

人が行う手順:

1. 作業ブランチで、`Conductor.psd1` の `ModuleVersion`(と README の版の記載)を新しい版にして、`main` に取り込む(ブランチの扱いは、チームの practices(`main` を統合ブランチとし、履歴をそのまま残して取り込む))。
2. `main` の先頭のコミットに、タグ `v<版>`(例: `v0.8.0`)を付けて、push する。
3. `release.yml` が、(a) タグが `main` 上にあること、タグと `ModuleVersion` の一致を検査し(AC-444)、(b) `ci.yml` を呼び出して全ジョブを通し、(c) アーカイブと `SHA256SUMS` を作り、(d) **下書きの**リリースに添付する。
4. 人が、下書きの内容(成果物、`SHA256SUMS`、注記)を確かめて、公開する。先行版(版に `-` を含む。例: `0.9.0-rc.1`)は、プレリリースにする(`ModuleVersion` は 3 つの数字なので、先行版の接尾辞は、マニフェストの `PrivateData.PSData.Prerelease` に書く。AC-445)。
5. (任意)人が手で、PowerShell Gallery に公開する(6.1 節)。

`release.yml` の骨組み:

```
# Copyright 2026 agwlvssainokuni (Apache License 2.0 のヘッダ。全文は省略)
name: Release

on:
  push:
    tags: ['v*']

permissions:
  contents: read

defaults:
  run:
    shell: pwsh

jobs:
  verify:
    runs-on: ubuntu-24.04
    timeout-minutes: 5
    steps:
      - uses: actions/checkout@<版>
        with:
          fetch-depth: 0
      - name: Tag must be on main
        shell: bash
        run: |
          git fetch origin main
          git merge-base --is-ancestor "$GITHUB_SHA" origin/main || { echo "tag is not on main"; exit 1; }
      - name: Tag must match ModuleVersion
        run: ./build/Invoke-Checks.ps1 -Check version -Tag $env:GITHUB_REF_NAME

  ci:
    needs: verify
    uses: ./.github/workflows/ci.yml

  release:
    needs: [verify, ci]
    runs-on: ubuntu-24.04
    timeout-minutes: 15
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@<版>
      - name: Build package
        run: ./build/New-Package.ps1 -OutDir artifacts/dist
      - name: Create draft release
        shell: bash
        env:
          GH_TOKEN: ${{ github.token }}
        run: |
          pre=""
          case "$GITHUB_REF_NAME" in *-*) pre="--prerelease" ;; esac
          gh release create "$GITHUB_REF_NAME" --draft --verify-tag $pre \
            --title "$GITHUB_REF_NAME" \
            --notes "Draft. Verify with SHA256SUMS. See README for execution policy and Unblock-File on Windows." \
            artifacts/dist/conductor-*.zip artifacts/dist/conductor-*.tar.gz artifacts/dist/SHA256SUMS
```

(`git merge-base --is-ancestor` による「タグが `main` 上」の判定、`gh release create` の `--draft`・`--verify-tag`・`--prerelease` の存在は、記憶による。**実装時に確認する(未確認)**。`ci.yml` は `workflow_call` で呼べるようにしてある(5.1 節)。`tag` の検査で、`GITHUB_REF_NAME` が `v0.9.0-rc.1` のとき、`-Check version` が、先行版の接尾辞を除いた `0.9.0` と `ModuleVersion` を比べる(5.4 節の実装)。)

### 6.4 利用者への注意と、検証の手順

```powershell
# 1. ダウンロードしたら、チェックサムを確かめる(UNIX 系)
sha256sum -c SHA256SUMS            # macOS: shasum -a 256 -c SHA256SUMS
# 2. 展開する(UNIX 系は tar.gz、Windows は zip)
tar -xzf conductor-0.8.0.tar.gz
# 3. 動作の確認
./conductor-0.8.0/bin/conductor --version      # conductor 0.8.0
```

Windows では、`Get-FileHash .\conductor-0.8.0.zip -Algorithm SHA256` の値を、`SHA256SUMS` の行と比べる。実行ポリシーの注意は 6.5 節。

### 6.5 署名をしないことと、実行ポリシー・Zone.Identifier の注意

- **Authenticode 署名は行わない**(コード署名の証明書を持たない)。`SHA256SUMS` は、転送の破損の検知までで、配布元の証明ではない。macOS の署名・公証も行わない。
- **実行ポリシー**(確認: 1 章): 強制されるのは Windows だけ。Windows の既定は `RemoteSigned`(クライアントもサーバーも)。`RemoteSigned` では、**インターネットから取得した(`Zone.Identifier` の代替データストリームが付いた)署名のないスクリプトは、実行されない**。署名のないローカルのスクリプトは実行される。`Restricted` は、スクリプトを一切実行せず、`.psm1` も止める。`AllSigned` は、署名のないスクリプトを実行しない(この配布物は署名がないので、動かない)。実行ポリシーは、セキュリティの境界ではない(確認: 1 章)。組織の Group Policy が設定している場合は、それが優先される。UNIX 系は、実行ポリシーが強制されない。
- 利用者向けの手順(README に書く。AC-561、`Invoke-Checks readme` が語を検査する):
  1. `Get-ExecutionPolicy -List` で、実効の実行ポリシーを確認する。
  2. ダウンロードした zip を、展開する前に `Unblock-File .\conductor-0.8.0.zip` で解除する(展開したファイルに、印が引き継がれないようにする)。展開したあとで印が付いている場合は、`Get-ChildItem -Recurse -File .\conductor-0.8.0 | Unblock-File`(`Unblock-File` は、ディレクトリを再帰しないので、ファイルを列挙して渡す)。
  3. 実行ポリシーを変えたくない場合は、そのセッションだけ `pwsh -ExecutionPolicy Bypass ...` を使える(`-ExecutionPolicy` は Windows だけに効く。確認: 1 章)。`Set-ExecutionPolicy -Scope CurrentUser RemoteSigned` で、利用者の範囲の方針を変えることもできる(どれを選ぶかは、利用者の組織の方針に従う。この配布物は変えない)。
- `curl.exe`・`Invoke-WebRequest` で取得したファイルには、印が付かない場合がある(確認: 1 章)ので、その場合は、解除が要らない。印の有無は `Get-Item <ファイル> -Stream *` で確かめられる(確認: 1 章の `Get-Item` の `-Stream` の記述)。
- 次は未確認: `tar.exe`・`Expand-Archive`・エクスプローラー・7-Zip が、zip・tar.gz の展開で、印を引き継ぐかどうか。macOS の Gatekeeper が、`pwsh` から読まれるスクリプトを止めるかどうか。Constrained Language Mode・AppLocker・WDAC の環境で、`Add-Type` が使えず `run`・`pipeline` が動かないこと(AC-560。実機で確かめる)。

---

## 7. デモ

### 7.1 構成

デモは、`conductor` の主な動きを、1 本のスクリプトで順に見せる。利用者は、リポジトリの根から実行する。スクリプトは、一時ディレクトリに定義とジョブを写して実行し、終了時にそのディレクトリを消す(リポジトリには何も残らない)。デモの定義は `.psd1`(場面 6 の `convert` で、JSON に変換して見せる)。ジョブは PowerShell のスクリプトで、UNIX 系と Windows で同じものを使う。

```text
demo/README.md                  使い方と各場面の説明
demo/conductor.psd1             定義(パイプライン nightly。UNIX 系と Windows 共通)
demo/retry.psd1                 リトライの定義(パイプライン retry-demo)。場面 10 だけが使う
demo/run-demo.ps1               デモのスクリプト(UNIX 系と Windows 共通)
demo/scripts/{extract,transform,load,notify,flaky}.ps1   ジョブ
```

### 7.2 実行の仕方(UNIX 系と Windows の差)

| 項目 | UNIX 系 | Windows |
|------|---------|---------|
| 実行 | `pwsh -NoProfile -NonInteractive -File demo/run-demo.ps1`(リポジトリの根から) | 同じ。コンソールから `pwsh -NoProfile -File demo\run-demo.ps1` |
| 実行ファイルの選択 | 環境変数 `CONDUCTOR`(ランチャーのパス。直接実行する)。なければ、リポジトリの `bin/conductor` | `CONDUCTOR`、なければ `bin\conductor.cmd` |
| 表示の言語 | `DEMO_LANG`(既定 `ja_JP.UTF-8`。英語は `en_US.UTF-8`)を、子の環境の `LANG` に設定。`LC_ALL`、`LC_MESSAGES` は外す | 同じ(環境変数は、子のプロセスにだけ渡し、利用者のセッションを書き換えない) |
| 一時停止 | `DEMO_PAUSE=1` のときだけ、各場面のあとで Enter を待つ | 同じ |
| ジョブの待ち時間 | `DEMO_SLEEP`(既定 0.5 秒。小数可。`0` で待たない) | 同じ |
| ジョブの終了コード | 環境変数 `DEMO_<ジョブ名の大文字>_EXIT`(既定 0) | 同じ |
| 作業場所 | `[System.IO.Path]::GetTempPath()` の下の `conductor-demo-<乱数>` に定義とスクリプトを写して実行。終了時に `finally` で消す | 同じ(`%TEMP%` の下) |
| 事前の環境変数の処理 | `DEMO_RUN_ID`、`CONDUCTOR_RUN_ID` を子の環境から外す(利用者の環境の影響を除く) | 同じ |
| 出力の符号化 | 日本語の固定の文を、`[Console]::OutputEncoding` を UTF-8(BOM なし)にして出す(デモのスクリプトだけが、実行中に設定して、終わりに戻す。`conductor` は自分で UTF-8 のバイトを書く。AC-520) | 同じ。**コンソールのフォントや、`pwsh` の `OutputEncoding` の既定により、日本語の表示が崩れる場合は、利用者の環境の問題(README に書く)** |
| 色 | 見出し・コマンド・終了コードの行に ANSI の色(`[Console]::IsOutputRedirected` が `$false` のときだけ。`NO_COLOR` があれば色なし)。`$PSStyle` に頼らず、自前の ANSI の文字列 | Windows の端末で、ANSI の表示が有効かは**未確認**(無効なら色なし)。パイプ・ファイルのときは色なし |
| 各コマンドの出力 | `$ conductor <引数>` の行、実行、`→ 終了コード <N>` | 同じ |
| ステートの一覧 | `conductor.psd1*.state` に当たるファイルを、`[System.IO.Directory]::GetFiles` で列挙(なければ `(なし)`)。`ls`・`dir` を使わない | 同じ |
| 場面の見出し | `=== <番号>. <題> ===` | 同じ |

### 7.3 定義とジョブ

`demo/conductor.psd1`:

```powershell
@{
    format_version = 1
    pipeline = @{
        name = 'nightly'
        description = 'デモ用の夜間バッチ'
        run_id_env = 'DEMO_RUN_ID'
    }
    job = @(
        @{
            id = 'extract'
            command = 'pwsh -NoProfile -NonInteractive -File scripts/extract.ps1'
            warning_exit_codes = @(1)
            transitions = @( @{ on = 'warning'; next = 'notify' } )
        }
        @{ id = 'transform'; command = 'pwsh -NoProfile -NonInteractive -File scripts/transform.ps1' }
        @{ id = 'load'; command = 'pwsh -NoProfile -NonInteractive -File scripts/load.ps1' }
        @{ id = 'notify'; command = @('pwsh', '-NoProfile', '-NonInteractive', '-File', 'scripts/notify.ps1') }
    )
}
```

`demo/retry.psd1`:

```powershell
@{
    format_version = 1
    pipeline = @{ name = 'retry-demo'; description = 'リトライのデモ' }
    job = @(
        @{
            id = 'flaky'
            command = 'pwsh -NoProfile -NonInteractive -File scripts/flaky.ps1'
            retry = @{ exit_codes = @(75); max = 3; interval = '1s' }
        }
    )
}
```

ジョブのスクリプトの仕様(`extract`・`transform`・`load`・`notify`): `  [<名>] 実行中...` を表示する → `DEMO_SLEEP` の秒だけ待つ → `  [<名>] 終了コード <N> で終わります` を表示する → 環境変数 `DEMO_<名の大文字>_EXIT`(既定 0)の値 N で終わる。`scripts/extract.ps1` の例(ほかは `$name` だけが違う):

```powershell
# Copyright 2026 agwlvssainokuni (Apache License 2.0 のヘッダ。全文は省略)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$name = 'extract'
$inv = [System.Globalization.CultureInfo]::InvariantCulture
$raw = [System.Environment]::GetEnvironmentVariable('DEMO_SLEEP')
$sleep = if ([string]::IsNullOrEmpty($raw)) { 0.5 } else { [double]::Parse($raw, $inv) }
$out = [System.Console]::Out
$out.WriteLine("  [$name] 実行中...")
if ($sleep -gt 0) { [System.Threading.Thread]::Sleep([int]($sleep * 1000)) }
$rawExit = [System.Environment]::GetEnvironmentVariable('DEMO_' + $name.ToUpperInvariant() + '_EXIT')
$code = if ([string]::IsNullOrEmpty($rawExit)) { 0 } else { [int]::Parse($rawExit, $inv) }
$out.WriteLine("  [$name] 終了コード $code で終わります")
$out.Flush()
exit $code
```

`flaky` の仕様: カレントディレクトリの `flaky.count` の回数を 1 増やす(なければ 0 から)→ `  [flaky] <n> 回目の実行...` を表示する → 待つ → `n` が `DEMO_FLAKY_FAILS`(既定 2)以下なら `  [flaky] 終了コード 75 で終わります(一時的な失敗)` を表示して 75 で終わる。超えたら `  [flaky] 終了コード 0 で終わります` を表示して 0 で終わる。
(デモのジョブは PowerShell のスクリプトであり、各ジョブの起動に `pwsh` の起動の費用がかかる。デモの全体の所要時間は、`DEMO_SLEEP=0` でも、`conductor` と各ジョブの `pwsh` の起動で、数十秒になりうる(未測定)。)

`demo/run-demo.ps1` の骨組み(場面の本体は、7.4 節の表のとおり):

```powershell
# Copyright 2026 agwlvssainokuni (Apache License 2.0 のヘッダ。全文は省略)
#requires -Version 7.6
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repo = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$conductor = if ($env:CONDUCTOR) { $env:CONDUCTOR }
             elseif ($IsWindows) { Join-Path $repo 'bin' 'conductor.cmd' }
             else { Join-Path $repo 'bin' 'conductor' }
$work = Join-Path ([System.IO.Path]::GetTempPath()) ('conductor-demo-' + [guid]::NewGuid().ToString('N'))
$prevEncoding = [System.Console]::OutputEncoding

function Write-Line([string]$Text) { [System.Console]::Out.WriteLine($Text) }

function Invoke-Demo {
    param([Parameter(Mandatory)][string[]]$Arguments, [hashtable]$Env = @{})
    Write-Line ('$ conductor ' + ($Arguments -join ' '))
    $psi = [System.Diagnostics.ProcessStartInfo]::new($conductor)
    foreach ($a in $Arguments) { $psi.ArgumentList.Add($a) }
    $psi.WorkingDirectory = $work
    $psi.UseShellExecute = $false
    foreach ($k in @('DEMO_RUN_ID', 'CONDUCTOR_RUN_ID', 'LC_ALL', 'LC_MESSAGES')) { $null = $psi.Environment.Remove($k) }
    $psi.Environment['LANG'] = if ($env:DEMO_LANG) { $env:DEMO_LANG } else { 'ja_JP.UTF-8' }
    foreach ($k in $Env.Keys) { $psi.Environment[$k] = [string]$Env[$k] }
    $p = [System.Diagnostics.Process]::Start($psi)
    $p.WaitForExit()
    Write-Line "→ 終了コード $($p.ExitCode)"
}

try {
    [System.IO.Directory]::CreateDirectory($work) | Out-Null
    Copy-Item -Path (Join-Path $PSScriptRoot '*') -Destination $work -Recurse
    [System.Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
    # 場面 0〜10(7.4 節)
    Write-Line 'デモは以上です。'
}
finally {
    [System.Console]::OutputEncoding = $prevEncoding
    if ([System.IO.Directory]::Exists($work)) { [System.IO.Directory]::Delete($work, $true) }
}
```

(デモのスクリプトは、製品のコードではないので、`Copy-Item` などを使ってよい。ただし、PSScriptAnalyzer と整形の検査の対象(5.4 節)。`Process.Start` で `bin/conductor`(シェルスクリプトとしての実行ファイル)を直接起動できるのは UNIX 系で、実行権限と `#!` が前提。)

### 7.4 場面

| 場面 | 見せること | 実行するコマンド(UNIX 系と Windows で同じ引数) |
|------|-----------|--------------------------------------------------|
| 0 | 定義の中身 | `conductor.psd1` を表示(`[System.IO.File]::ReadAllText`) |
| 1 | 全部成功。書いた順に 1 つずつ | `pipeline -v nightly` |
| 2 | `extract` が警告(`DEMO_EXTRACT_EXIT=1`)→ `transform`・`load` をスキップして `notify` へ | `pipeline -v nightly` |
| 3 | `load` のエラー(`DEMO_LOAD_EXIT=2`)で止まり、終了コード 2。止まった位置 | `pipeline -v nightly`、続けて `watch` |
| 4 | `load` から再開(`transform` は再実行しない)。完了してもステートは残り、`watch` が完了の行を出す。`--remove-on-complete` で完了時にステートを消す | `pipeline -v nightly`、`watch`、ステートの一覧、`pipeline -v --remove-on-complete nightly`、一覧、`watch` |
| 5 | 個別実行と初期化 | `run -v extract`、`reset` |
| 6 | `.psd1` から JSON への変換(`.psd1` はコメントが書ける、と案内する) | `convert conductor.psd1 --to json` |
| 7 | 検査。正しい定義は 0(`definition is valid`)、引用符が閉じていない定義は 64(`command has an unclosed quote`)。ジョブは実行されず、ステートも作られない | `validate -v`、誤りの定義 `bad.psd1`(`@{ format_version = 1; pipeline = @{ name = 'p' }; job = @( @{ id = 'a'; command = 'sh -c ''echo hi' } ) }`)を作って `validate --config bad.psd1`、一覧 |
| 8 | run ID ごとに別のステートファイル(`conductor.psd1.<run ID>.state`)。同じ run ID で再開。完了後もステートは残り、`reset --run-id` で片づける | `pipeline -v --run-id build-1 nightly`(`load` で失敗)、環境変数 `DEMO_RUN_ID=build-2` で `pipeline -v nightly`、一覧、`watch --run-id build-1`、`pipeline -v --run-id build-1 nightly`(再開)、一覧、`watch --run-id build-1`、`reset --run-id build-1`、`reset --run-id build-2`、一覧 |
| 9 | `--run-id` は環境変数に勝つ。`--run-id ""` は run ID なしに固定。`reset --run-id` は 1 つ、`reset --all` は全部。最後の `watch` は `no state` | `DEMO_RUN_ID=env-1` で `pipeline`(`load` で失敗)、`DEMO_RUN_ID=env-1` と `--run-id opt-1`、`--run-id ""`、`--run-id opt-2`、一覧、`reset --run-id opt-1`、一覧、`reset --all`、一覧、`watch` |
| 10 | `flaky` が 2 回、終了コード 75 で失敗し、`conductor` は 1 秒待つたびに標準エラーへ `conductor: retry: job 'flaky' exited with code 75; retry <n> of 3 in 1 seconds` を出して実行し直し、3 回目で成功(`job=flaky result=success`) | `flaky.count` を消し、`pipeline -v --config retry.psd1 retry-demo` |

最後に `デモは以上です。` を出す。デモの受け入れテストは 4.17 節。

---

## 8. 依存(実行時: 無し。開発時: Pester、PSScriptAnalyzer)

### 8.1 方針

- **実行時の依存は無い**。標準の PowerShell(`Microsoft.PowerShell.*` のモジュールと、組み込みのコマンドレット)と、.NET の標準のライブラリ(BCL)だけを使う。PowerShell Gallery のモジュールを、実行時に読み込まない(AC-443)。理由は、(1) 供給網の攻撃面を増やさない、(2) モジュールの読み込みが起動時間を増やす(2.1 節)、(3) 利用者が追加のインストールを要しない、(4) 製品のコードが、データからコードを作らない(2.3 節)方針と、外部のモジュールの機能(`ConvertFrom-Json` など)が合わない。
- 開発時の依存は、Pester 5 と PSScriptAnalyzer の 2 つだけ(8.2 節)。版を固定し、取得したファイルのハッシュを検査する(8.3 節)。
- 依存を足すときは、`build/Dependencies.psd1` の許可の一覧(`Invoke-Checks deps` が検査。AC-478)に、名前・版・ハッシュを足し、この章の選定の基準を満たすことを書く。

### 8.2 開発時の依存の一覧と選定

| 依存 | 用途 | 版(例。実装時に確認) | 選定の理由 |
|------|------|----------------------|------------|
| Pester | テストの枠組み(3 章)。5.x | 5.9.0(runner-images の文書に載っていた版。1 章) | PowerShell の標準的なテストの枠組みで、`Describe`/`It`/`Mock`/`TestDrive`/コードカバレッジを持つ。GitHub のランナーに入っている(ubuntu-24.04 で確認)。6.x は、移行の費用と互換が未確認なので採らない |
| PSScriptAnalyzer | 静的検査と整形(5 章) | 1.25.0(同上) | Microsoft の公式の静的検査で、構文の誤りの検出、規則の設定、`Invoke-Formatter` による整形を持つ。GitHub のランナーに入っている(ubuntu-24.04 で確認) |

選定の基準(新しい依存を足すときに、すべて満たすこと): (1) PowerShell 7.x の 3 OS(Linux・macOS・Windows)で動く、(2) 版を完全に指定できる(`Save-Module -RequiredVersion`)、(3) 実行時の依存を持たない(推移的な依存がない、または一覧に足す)、(4) 保守されている(直近 1 年以内にリリースがある。実装時に確認)、(5) ライセンスが、プロジェクトの Apache License 2.0 と両立する(実装時に確認。**この文書の作成時には、2 つのライセンスを確認していない(未確認)**)、(6) 開発時にだけ使い、配布物に含めない、(7) ネットワークからの取得が、固定の版とハッシュの検査を伴う。

採らなかったもの: Pester 6(上記)、テスト用のモックのライブラリ・アサーションの追加のモジュール(Pester の `Mock` と `Should` で足りる)、データファイルの形式を増やすモジュール(定義の形式は `.psd1` と JSON だけ)、独自の整形ツール(PSScriptAnalyzer の `Invoke-Formatter` で足りる)。

### 8.3 版の固定とハッシュ(`build/Dependencies.psd1`)

- `build/Dependencies.psd1` に、各依存の `Name`・`Version`(完全な版)・`TreeSha256`(取得した木のハッシュ。5.4 節の `Get-TreeHash`)を書く。
- 取得は `build/Install-DevDependencies.ps1`(`Save-Module -RequiredVersion`。`-MinimumVersion` や既定の「最新」を使わない。確認: 1 章の `Install-Module` の記述(`-RequiredVersion` は完全一致))だけが行う。ハッシュが違えば、入れずに失敗する。
- 版の更新は、人が行う: `build/Update-Dependencies.ps1 -Name Pester -Version <新しい版>` が、新しい版を取得してハッシュを表示する。人が、リリースの記録(変更の内容)を確かめて、`Dependencies.psd1` を書き換え、通常のプルリクエストにする。自動で最新に追従しない。
- `Save-Module` が、`<Path>/<名前>/<版>/` に保存するという前提と、`PSGetModuleInfo.xml`(インストール日時を含む)が木のハッシュから除かれていること(ほかに、取得のたびに変わるファイルがないこと)は、**未確認**。違えば、ハッシュの対象の規則を直す。

### 8.4 供給網の注意

- **GitHub のランナーに、別の版の Pester・PSScriptAnalyzer が入っている**(ubuntu-24.04 で確認)。固定の版の取得先は、`Save-Module -Path` で別のディレクトリにして、`PSModulePath` の先頭に足し、`Import-Module -RequiredVersion` で読み込む(5.4 節)。ランナーに入っている版を、黙って使わない。Windows PowerShell 5.1 には、古い Pester(3.x)が標準で入っている(記憶による。未確認)ので、`pwsh` を使い、版を明示して取り違えを避ける。
- PowerShell Gallery は、TLS 1.2 以上が要る(確認: 1 章)。取得元は `-Repository PSGallery` を明示する。初回に、信頼されないリポジトリの確認や、パッケージプロバイダーのインストールの確認が出る場合がある(CI で止まらないように、`-Force` を付ける。挙動は未確認)。
- Gallery のモジュールは、署名されている場合がある(Authenticode)。署名の検査(`Get-AuthenticodeSignature`)は、UNIX 系で使えるか、**未確認**。この仕様は、署名でなく、固定の版とハッシュで守る。
- 実行時の依存が無いので、脆弱性の検査(`deps-audit`。AC-481)の対象は、開発時の 2 つだけ。最新の版との差を、毎週記録する(5.2 節)。
- GitHub Actions の部品は、タグでなく、確認した完全なコミットの ID で固定することを勧める(5 章)。`release.yml` の権限は、`contents: write` を、リリースを作るジョブだけに限る。

---

## 9. リポジトリのレイアウト

### 9.1 ファイルの構成

```text
.
├── .editorconfig
├── .gitattributes
├── .gitignore
├── .github/workflows/{ci,audit,release}.yml
├── LICENSE                          Apache License 2.0
├── README.md                        使い方、実行ポリシーの注意(6.5 節)、リトライの例
├── PSScriptAnalyzerSettings.psd1    静的検査・整形の設定(9.2 節)
├── bin/
│   ├── conductor                    UNIX 系のランチャー(`sh`。[04](04-cli.md) 1.3.2 節)
│   ├── conductor.cmd                Windows のランチャー(CRLF。04 章 1.3.3 節)
│   └── conductor.ps1                入口スクリプト(`exit` を呼ぶ唯一のファイル。04 章 1.3.4 節)
├── build/
│   ├── Versions.psd1                PowerShell の版の出所(5.4 節)
│   ├── Dependencies.psd1            開発時の依存の版とハッシュ(8 章)
│   ├── Install-DevDependencies.ps1
│   ├── Update-Dependencies.ps1      人が実行する。新しい版のハッシュを表示する
│   ├── Resolve-Pwsh.ps1
│   ├── Invoke-Checks.ps1
│   ├── Invoke-Tests.ps1
│   └── New-Package.ps1
├── demo/                            7 章
├── docs/spec/powershell/            この仕様書
├── src/Conductor/
│   ├── Conductor.psd1               マニフェスト(`ModuleVersion` が版の唯一の出所)
│   ├── Conductor.psm1               Private/ と Public/ を読み込む
│   ├── Public/Invoke-Conductor.ps1
│   └── Private/{Text,Json,RunId,Flow,Definition,Convert,State,Runner,Report,Cli}/   ([02](02-architecture.md))
├── tests/
│   ├── Unit/<部品>/*.Tests.ps1
│   ├── Acceptance/*.Tests.ps1
│   ├── Inspect/*.Tests.ps1
│   ├── Perf/*.Tests.ps1
│   ├── jobs/{tj.ps1,inject.ps1,tj-signals.cs.txt}
│   ├── support/{Harness.ps1,Fakes.ps1,Marks.ps1,Tree.ps1,Xorshift32.ps1}
│   ├── fixtures/                    定義の入力例(.psd1 と JSON)
│   └── WINDOWS_CTRL_C_CHECKLIST.md  AC-301 の記録欄
└── artifacts/                       成果物(.gitignore の対象)
```

### 9.2 `PSScriptAnalyzerSettings.psd1`

```powershell
# Copyright 2026 agwlvssainokuni (Apache License 2.0 のヘッダ。全文は省略)
@{
    Severity     = @('Error', 'Warning', 'ParseError')
    ExcludeRules = @(
        'PSUseShouldProcessForStateChangingFunctions',   # 非公開の関数で、-WhatIf・-Confirm を持たない設計のため
        'PSUseSingularNouns'                             # 部品の名前の規約(Verb-ConductorNoun)を優先
    )
    Rules        = @{
        PSUseConsistentIndentation = @{ Enable = $true; Kind = 'space'; IndentationSize = 4; PipelineIndentation = 'IncreaseIndentationForFirstPipeline' }
        PSUseConsistentWhitespace  = @{ Enable = $true; CheckInnerBrace = $true; CheckOpenBrace = $true; CheckOpenParen = $true; CheckOperator = $true; CheckPipe = $true; CheckSeparator = $true }
        PSPlaceOpenBrace           = @{ Enable = $true; OnSameLine = $true; NewLineAfter = $true; IgnoreOneLineBlock = $true }
        PSPlaceCloseBrace          = @{ Enable = $true; NewLineAfter = $false; IgnoreOneLineBlock = $true; NoEmptyLineBefore = $false }
        PSAlignAssignmentStatement = @{ Enable = $true; CheckHashtable = $true }
        PSUseCorrectCasing         = @{ Enable = $true }
        PSUseCompatibleSyntax      = @{ Enable = $true; TargetVersions = @('7.6') }
    }
}
```

(1 章で確認したのは、設定の主なキー(`Severity`・`ExcludeRules`・`IncludeRules`・`Rules`・`Enable`)、規則の名前(`PSUseConsistentIndentation` など。接頭辞 `PS` の有無は、規則名と、設定のキーとで版により異なる場合がある。**未確認**)、`UseCompatibleSyntax` の `TargetVersions`。個々の規則の設定のキー(`IndentationSize`、`PipelineIndentation`、`CheckInnerBrace`、`OnSameLine`、`CheckHashtable` など)は、**この文書では確認していない(未確認)**。実装時に、PSScriptAnalyzer の `Settings/CodeFormatting.psd1`(1 章)を見て、キーの名前を確かめる。`TargetVersions` は、`MinPowerShell` と `MaxPowerShell`(重複を除く)にそろえる(AC-483)。この設定を `Invoke-Formatter -Settings` にも渡すので、PSScriptAnalyzer の指摘と、整形の差分が、同じ規則の組になる。)

### 9.3 `.gitattributes`、`.gitignore`、`.editorconfig`

```text
# .gitattributes
* text=auto eol=lf
*.cmd text eol=crlf
bin/conductor text eol=lf
*.zip binary
*.gz binary

# .gitignore
artifacts/
*.state
.vscode/
.idea/

# .editorconfig
root = true
[*]
charset = utf-8
end_of_line = lf
insert_final_newline = true
trim_trailing_whitespace = true
indent_style = space
indent_size = 4
[*.cmd]
end_of_line = crlf
[*.yml]
indent_size = 2
```

(`.ps1`・`.psm1`・`.psd1` の文字コードは、BOM なしの UTF-8 を基本とする。日本語の固定の文を含むため、PSScriptAnalyzer の `UseBOMForUnicodeEncodedFile` が指摘する場合は、設定で理由つきで外す(AC-527)。`bin/conductor` の実行権限は、`git update-index --chmod=+x` で付ける。)

### 9.4 `Conductor.psd1` と `Conductor.psm1`

```powershell
# Conductor.psd1(抜粋。全文は、AC-508 の項目を満たすこと)
@{
    RootModule           = 'Conductor.psm1'
    ModuleVersion        = '0.8.0'
    GUID                 = '<新しい GUID を 1 回だけ作って固定する>'
    Author               = 'agwlvssainokuni'
    CompanyName          = 'agwlvssainokuni'
    Copyright            = 'Copyright 2026 agwlvssainokuni'
    Description          = 'Run a pipeline of jobs defined in a data file, with retry, resume, and a state file.'
    PowerShellVersion    = '7.6'
    CompatiblePSEditions = @('Core')
    FunctionsToExport    = @('Invoke-Conductor')
    CmdletsToExport      = @()
    VariablesToExport    = @()
    AliasesToExport      = @()
}
```

```powershell
# Conductor.psm1
#requires -Version 7.6
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:ConductorVersion = (Import-PowerShellDataFile -LiteralPath ([System.IO.Path]::Combine($PSScriptRoot, 'Conductor.psd1'))).ModuleVersion

foreach ($dir in @('Text', 'Json', 'RunId', 'Flow', 'Definition', 'Convert', 'State', 'Runner', 'Report', 'Cli')) {
    $files = [System.IO.Directory]::GetFiles([System.IO.Path]::Combine($PSScriptRoot, 'Private', $dir), '*.ps1')
    [System.Array]::Sort($files, [System.StringComparer]::Ordinal)
    foreach ($f in $files) { . $f }
}
$pub = [System.IO.Directory]::GetFiles([System.IO.Path]::Combine($PSScriptRoot, 'Public'), '*.ps1')
[System.Array]::Sort($pub, [System.StringComparer]::Ordinal)
foreach ($f in $pub) { . $f }

Export-ModuleMember -Function 'Invoke-Conductor'
```

(`Conductor.psm1` が、ファイルをドットソースする唯一の場所で、`Invoke-Checks forbidden` の許可の一覧にある(5.4 節)。`Invoke-Conductor` は、`[CmdletBinding()]` を持たず、`-ArgumentList [string[]]` で受ける(04 章 1.4 節、AC-547)。`Export-ModuleMember` と `FunctionsToExport` の両方で、公開を 1 つに限る(AC-572)。)

---

## 10. 未確認の事項と、実装時に最初に確かめる小さな試験、他の章との整合が要る点

### 10.1 未確認の事項

この環境には PowerShell がなく、実機で動かせない。次は、公式の文書で確認できなかった、または確認していない点である。性能の基準値は、すべて仮で、測っていない。

| 区分 | 未確認の内容 | 関係する項目 |
|------|--------------|--------------|
| `pwsh` の挙動 | `-File` の後ろの `-v`・`-Verbose`・`-?`・`--`・`-h` が、`$args` にそのまま入るか。`param()` のない関数が、共通パラメーターを食わないか。不正な UTF-8 の引数の扱い。`exit 256`・負の値の、Windows での終了コード。SIGINT・SIGTERM を、登録なしで受けたときの既定の動作と終了コード。`PipelineStoppedException` が `catch` で捕まるか。標準出力を閉じたパイプへの書き込みの挙動 | AC-111、AC-112、AC-301、AC-406、AC-491、AC-493、AC-532、AC-547 |
| `Add-Type` | 起動の費用(数百 ms の見込み)。一時ファイルの有無。Constrained Language Mode での失敗。型の再定義のエラーの文言。`PosixSignal` の列挙子の存在。補助の部品のコールバックの、シグナルを受けるスレッド | AC-489、AC-538、AC-558、AC-560 |
| 子プロセス | シグナルで死んだ子の `Process.ExitCode`(`128 + n` か)。`SIGCHLD` を無視して起動された場合。NUL を含む引数の例外の型。`.cmd` の引数の扱い | AC-140、AC-504、AC-522、AC-534 |
| `.psd1` の読み取り | 構文木の定数の値(`.Value`)の型と、字句の違い。`1e999`・`-0.0`・単項のカンマ・U+2018/U+2019・バッククォートのエスケープ・`ParseError` の行の位置・深い入れ子での `Parser.ParseInput` の挙動 | AC-210、AC-217、AC-234、AC-524、AC-578、AC-579 |
| Pester 5 | `PesterConfiguration` のキーが 5.x にあるか。`-CI` の挙動。`-FullNameFilter`。`Mock -ModuleName`・`InModuleScope` の挙動。`Result` オブジェクトの名前。JaCoCo の出力の `report` 直下の `counter`。目標の率による失敗。個々のテストのタイムアウト | AC-565、5.3 節、5.4 節 |
| PSScriptAnalyzer | 個々の規則の設定のキー。規則名の `PS` 接頭辞。`Invoke-Formatter` が、末尾の改行・CRLF をどう扱うか。`-EnableExit` | AC-480、AC-518、9.2 節 |
| CI | macOS・Windows のランナーの `pwsh`・Pester・PSScriptAnalyzer の版。`shell: pwsh` の複数行の失敗の扱い。ステップをまたぐ `PATH` の更新が、`shell: pwsh` に効くか。`pwsh` の公式のリリースの成果物の名前とハッシュ。`Save-Module` の保存の形。`Test-ModuleManifest` の検査の範囲。`gh release create` と `git merge-base --is-ancestor` の細部。`POWERSHELL_TELEMETRY_OPTOUT`・`POWERSHELL_UPDATECHECK` の変数名 | 5 章、6.3 節 |
| 実行ポリシー | zip・tar.gz の展開で `Zone.Identifier` が引き継がれるか(`tar.exe`・`Expand-Archive`・エクスプローラー・7-Zip)。macOS の Gatekeeper。`Unblock-File` のあとの挙動 | AC-561、6.5 節 |
| 配布 | zip の UNIX 権限。`[System.IO.Compression.ZipFile]` の `Add-Type` の要否。`ModuleVersion` と `Prerelease` の扱い。PowerShell Gallery への公開の形(`bin/` が含まれない) | AC-445、AC-526、6.1 節 |
| 性能 | 2.1 節のすべての基準値(仮)。結合したモジュールの起動時間。`TJ` の起動の費用 | AC-397〜403、AC-488 |

### 10.2 実装時に最初に確かめる小さな試験

次を、実装の最初の日に実行して、結果を記録し、この文書の「未確認」を直す。

| 番号 | 試験 | 確かめること |
|------|------|--------------|
| T1 | `pwsh -NoProfile -File t.ps1 -v run a --help -Verbose -? -- x`(`t.ps1` は `$args` と `$PSBoundParameters` を出す) | 引数が文字列のまま `$args` に入る。終了コード(`exit 3` で 3、例外で 1) |
| T2 | モジュールの中の `function f { $args }`(`CmdletBinding` なし・`param()` なし)を `f -Verbose -v -?` で呼ぶ | 共通パラメーターが食われない |
| T3 | `Add-Type` の C# の補助(`PosixSignalRegistration`・`Console.CancelKeyPress`)を作り、`pwsh -File` の中で SIGINT・SIGTERM を送る | コールバックが C# の側で完結して動く。`-File` の終了コード(Ctrl+C で 0)と、`Invoke-Conductor` が返す 130 のとき。`PipelineStoppedException` の有無 |
| T4 | 同じ `Add-Type` を `Import-Module -Force` の 2 回で | 型の再定義のエラー。1 回目のコストを `Measure-Command` で測る |
| T5 | `$ExecutionContext.SessionState.LanguageMode = 'ConstrainedLanguage'` のあとの `Add-Type`・`[Parser]::ParseInput` | 使えるか |
| T6 | `[System.Diagnostics.Process]` で、`sh -c 'kill -9 $$'` を起動し、`ExitCode` を見る | `137` か(AC-140)。`Redirect* = $false` の継承 |
| T7 | `[Parser]::ParseInput` に、`@{ a = 1e999 }`・`@{ a = -0.0 }`・`@{ a = ,1 }`・`@{ a = 1L }`・`@{ a = 'it’s' }`・深さ 100,000 の入れ子を渡す | 構文木の型・`Extent.Text`・誤りの有無。落ちないか |
| T8 | `Import-PowerShellDataFile` に、重複キー(`a`・`A`)の `.psd1` を渡す | 比較用に使えるか。重複の扱い |
| T9 | 標準出力を閉じたパイプ・`/dev/full` にして、`[Console]::OpenStandardOutput()` に書く | 例外の型(`IOException` か、黙って捨てるか) |
| T10 | Pester 5 の最小のテストで、`-Skip:$cond`(`BeforeDiscovery`)、`-TestCases`、`Mock -ModuleName`、`InModuleScope`、`$TestDrive`、`CodeCoverage`(JaCoCo) | キー名・挙動・XML の形 |
| T11 | `Invoke-ScriptAnalyzer -Settings`、`Invoke-Formatter -Settings` に 9.2 節の設定を渡す | キー名、規則名、整形が冪等か |
| T12 | `pwsh -NoProfile -Command exit` と `--version` の起動時間を 3 OS で測る | 2.1 節の基準値の見直し |
| T13 | `bin/conductor.cmd` に、`%`・`^`・`&`・`\|`・`"` を含む引数を渡す | AC-563 の期待の確定(07 章) |
| T14 | zip・tar.gz を Zone.Identifier つきで展開 | AC-561 の期待 |
| T15 | `Save-Module -RequiredVersion` の保存の形とハッシュの再現性(3 OS) | 8.3 節の前提 |

### 10.3 他の章との整合(確認した結果と、残る未確認)

この文書が置いた前提のうち、02〜10 章と食い違っていたものは、02〜10 章の字句に合わせて、この文書の該当の項目(AC)を直した。直した主な点: ランチャーと入口(`bin/conductor`・`bin/conductor.cmd`・`bin/conductor.ps1`。`Invoke-Conductor -ArgumentList`。`pwsh` が無いときは 70)、補助の部品のファイル名(`Private/Runner/NativeHelper.ps1`)と OS の判定の関数(`Get-ConductorPlatformKind`)、`Text` ディレクトリ、`.psd1` の許す構文(10 進・`0x`・`0b`、`#requires` と複数の式の拒否、理由 `.psd1 で使えない書き方です`)、空の入力の理由、キーの大文字小文字(`.psd1` は重複とみなす。既知のキーの違いは `unknown key`)、`unrepresentable` の理由(`.psd1 では、大文字小文字だけが違うキーを同時に書けません`)、変換の理由文(17 件)、`.psd1` の書き出し(4 スペース、裸のキーは `^[A-Za-z_][A-Za-z0-9_]*$` かつ予約語でない)、標準出力の書き手(`Write-ConductorStream`)、リトライの待機の刻み(250 ms)、ジョブの終了の待機の刻み(10 ms)、版の出所(`Import-PowerShellDataFile`)。

残る未確認(実装時に確かめ、確かめた結果で、該当の章を直す):

| 章 | 未確認の前提 |
|----|--------------|
| [02](02-architecture.md) / [04](04-cli.md) | `GetCommandLineArgs()` が、`pwsh` の受け取った引数を変換や欠落なしに返すこと(04 章の T1)。`Invoke-Conductor -ArgumentList` の配列の要素が、パラメータとして解釈されないこと(T5)。`Import-PowerShellDataFile` が、マニフェストから `ModuleVersion` を読めること(T6) |
| [05](05-execution-flow.md) / [10](10-errors-and-exit-codes.md) | `-band 0xFF` が、負の `[int]` で、2 の補数の下位 8 ビットを返すこと(05 章の T-2。外れたら 10 章 4 章の剰余の式に切り替える) |
| [07](07-process-and-signals.md) | 入口スクリプトの `finally` の中の `[System.Environment]::Exit` が、Ctrl+C による停止のあとで終了コードを決められること(07 章の T-4)。シグナルで死んだ子の `Process.ExitCode`(AC-140) |
| [08](08-output-and-i18n.md) | 閉じたパイプへの書き込みの既知の差(UNIX 系で、`watch`・`convert` が 65・67 でなく 0 で終わりうる。AC-493 は、どちらでも合格にしてある) |
| [03](03-definition-format.md) / [09](09-converter.md) | 構文木の数・文字列の字句の細部(03 章の T-01〜T-20、09 章の T-C01〜T-C09) |
