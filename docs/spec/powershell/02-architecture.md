# 02 アーキテクチャ(ソース構成・責務・依存の向き・`Deps`・実行のシーケンス・モジュール構成)

対象: `conductor` v0.8.0 と同じ挙動を PowerShell 7(`pwsh`)で実装するための、論理的・物理的な構成。用語は [00-README.md](00-README.md) の用語集に従う。

この文書は、ソースのディレクトリとファイルの責務、関数の命名、依存の向きと、その強制、公開する関数の骨格、`Deps`(差し替え口)、`pipeline` 実行のシーケンス、モジュールの構成(`Conductor.psd1`、`Conductor.psm1`、ランチャー)、同期モデルとエラー処理の方針、版の唯一の出所を定める。外から見える挙動は 03〜10 章が正である。

## 規定の強さと確認の表記

| 表記 | 意味 |
|------|------|
| 【規定】 | どの実装も満たす。テスト([11-quality-and-build.md](11-quality-and-build.md))で確かめる |
| 【目安】 | 推奨。利用者から見える結果を変えない範囲で、変えてよい |
| 【PowerShell の設計】 | PowerShell で実装するために、この仕様が決めたこと(他の決め方もありうる) |
| 確認(出典: …) | 公式ドキュメント(learn.microsoft.com)、公開ソースを、この文書の作成時に読んで確かめた |
| 未確認 | 確かめられていない。この環境には `pwsh` も .NET SDK もなく、実機で動かしていない。実装時に最初に確かめる |

---

## 1. 設計の前提

| # | 前提 | 内容 |
|---|------|------|
| 1 | 形態 | PowerShell モジュール `Conductor`(`Conductor.psd1` + `Conductor.psm1`)と、ランチャー(`bin/conductor`、`bin/conductor.cmd`、`bin/conductor.ps1`)。公開する関数は `Invoke-Conductor` の 1 つ。利用者向けの文法は、現行の CLI のまま(`conductor pipeline -v --config x nightly`)で、PowerShell のコマンドレット(`-Parameter`、パイプライン)の形にしない |
| 2 | 対象 | PowerShell 7.x。最小の版は、実装時点でサポート中の LTS のうち最小のもの(`PowerShellVersion`。5.2 節)。Windows PowerShell 5.1 は対象外 |
| 3 | 実行時の依存 | なし(標準の PowerShell と .NET の BCL だけ)。C# の小さなヘルパー(`Add-Type`)は、ソースをモジュールが持つ([07-process-and-signals.md](07-process-and-signals.md)) |
| 4 | 開発時の依存 | Pester 5、PSScriptAnalyzer([11-quality-and-build.md](11-quality-and-build.md)) |
| 5 | 同期 | 本体は同期。コールバックは C# ヘルパーの中だけ(8 章) |
| 6 | 出力 | パイプラインに流さない。画面への出力は専用の関数が `[Console]::Out` / `[Console]::Error` に書く(9.2 節) |
| 7 | `class` を使わない | 値は `[pscustomobject]` と `[ordered]` のハッシュテーブル。理由は 7 章 |
| 8 | 終了 | `exit` を呼ぶのはランチャーだけ。`Invoke-Conductor` は `[int]` の終了コードを**返す** |

---

## 2. ソースのディレクトリ構成【規定】

```text
src/Conductor/
  Conductor.psd1                  マニフェスト(版の唯一の出所。5 章)
  Conductor.psm1                  ルートモジュール(読み込みの順序。5 章)
  Public/
    Invoke-Conductor.ps1          公開する唯一の関数(6.1 節)
  Private/
    Json/        ...              自前の JSON の読み取りと書き出し、上限の検査
    RunId/       ...              run ID の検査と決定
    Flow/        ...              純粋な判断(分類、起動前の判断、次の動き、再開位置、記録、リトライ、終了コード)
    Definition/  ...              定義の読み込みと検査(`.psd1` と JSON)
    Convert/     ...              変換(`.psd1` と JSON の相互変換)
    State/       ...              ステートの読み書き
    Runner/      ...              プロセス実行、シグナル、プラットフォーム層、C# ヘルパー
    Report/      ...              出力、ロケール、メッセージカタログ
    Cli/         ...              引数の解析と配線、サブコマンド、`Deps` の本番の実体
bin/
  conductor                       UNIX 系のランチャー(`sh` のスクリプト)
  conductor.ps1                   本体のランチャー(`pwsh` のスクリプト。両 OS が使う)
  conductor.cmd                   Windows のランチャー
tests/
  *.Tests.ps1                     Pester 5 のテスト
  jobs/                           受け入れテスト用のテストジョブ(11 章で定める)
  support/                        偽の `Deps`、補助
PSScriptAnalyzerSettings.psd1     静的検査の設定(リポジトリの直下)
build/                            ビルド・検査・パッケージのスクリプト(11 章で定める)
```

- 1 ファイルに 1 つの主な関数を置く。ファイル名は関数名と同じ(`Verb-ConductorNoun.ps1`)。補助の小さな関数は、同じディレクトリの同じファイルに置いてよい(ただし、ファイルの外から呼ぶ関数は、ファイル名の関数だけ)。
- ディレクトリの責務と関数の一覧は、3 章。
- `bin/conductor.ps1` は、固定のソース構成に対する追加のファイルである(Windows では、`pwsh -File` が `.ps1` だけを受け付けるため。確認。出典: `about_Pwsh` の `-File` の節「As of PowerShell 7.2, the File parameter only accepts .ps1 files on Windows」)。`bin/conductor`(UNIX 系)と `bin/conductor.cmd` は、この `.ps1` を呼ぶだけの薄い包みである(5.5 節)。

---

## 3. 各ディレクトリ・ファイルの責務と関数の一覧【規定】

### 3.1 命名の規則

- 関数名は `Verb-ConductorNoun`。`Verb` は PowerShell の承認された動詞(`Get-Verb` の一覧。PSScriptAnalyzer の `PSUseApprovedVerbs` が検査する)。`Noun` は単数形(`PSUseSingularNouns`)。モジュールの名前の接頭辞 `Conductor` を、名詞の先頭に付ける(利用者のセッションの他の関数と衝突しないため。非公開でも付ける)。
- 動詞の使い分け: `Get`(取り出す。純粋)、`Test`(真偽を返す、または検査して結果の値を返す)、`New`(値を作る。純粋)、`Read`(ファイルまたは文字列から読み取る)、`Write`(出力・ファイルへ書く)、`ConvertTo`/`ConvertFrom`(形の変換)、`Split`、`Resolve`、`Invoke`(実行する)、`Wait`、`Start`、`Send`、`Initialize`、`Format`(文を作る。純粋)、`Reset`(ステートの削除)。
- 純粋な関数に `New`/`Set`/`Reset` を使うため、PSScriptAnalyzer の `PSUseShouldProcessForStateChangingFunctions` を設定で無効にする【規定】(この規則は、`-WhatIf` を持たない状態変更の関数を警告する。製品の関数は、コマンドレットとして公開しないので、必要がない)。設定は `PSScriptAnalyzerSettings.psd1` の `ExcludeRules` に書く([11-quality-and-build.md](11-quality-and-build.md))。
- 引数は、位置ではなく、名前で渡す(`-Name value`)。引数を取る関数は、`param(...)` ブロックを持ち、型を明示する。`[CmdletBinding()]` は、内部の関数に付けない(共通パラメータ `-v`、`-Verbose`、`-Debug` などが付き、`-v` が `-Verbose` の短縮として解釈されるため。6.1 節の理由)。
- 値を返す関数の戻り値は、`return` で明示する。関数が返す値は 1 つだけ(1 つの `[pscustomobject]`、`[int]`、`[string]`、`[bool]`、またはハッシュテーブル)。配列を返す場合は、`return ,$array`(単項のカンマで包む)で、パイプラインによる展開を防ぐ。値を返さない関数は、何も出力しない(9.2 節)。

### 3.2 `Private/Json/`(依存の最下層)

| 関数 | 責務 |
|------|------|
| `ConvertFrom-ConductorJsonText` | JSON の文字列を、位置つきの木に読む(自前の走査。明示のスタック。重複キー、巨大整数、深さ上限)。`ConvertFrom-Json` を使わない |
| `ConvertTo-ConductorJsonText` | 木または値を、JSON の文字列(インデント 2、LF)に書き出す。`ConvertTo-Json` を使わない |
| `Test-ConductorTreeLimits` | 展開後の要素数、文字の大きさ、入れ子の深さの検査 |
| `New-ConductorTreeError` | 木の誤り(種類、位置、固定の文)を作る |

### 3.3 `Private/RunId/`

| 関数 | 責務 |
|------|------|
| `Test-ConductorRunId` | run ID の値の検査(空、長さ、文字、先頭のドット) |
| `Resolve-ConductorRunId` | `--run-id` と環境変数から run ID を決める |

### 3.4 `Private/Flow/`(純粋。入出力に触れない)

| 関数 | 責務 |
|------|------|
| `New-ConductorJobExecution` | 実行結果(`JobExecution`。[07-process-and-signals.md](07-process-and-signals.md) の 4.5 節)を作る |
| `Get-ConductorClassification` | 実行結果の分類と、報告する終了コード |
| `Get-ConductorDecision` | 起動前の判断(実行・スキップ・拒否) |
| `Get-ConductorNextStep` | 次の動き |
| `Get-ConductorResumePoint` | 中断時の再開位置 |
| `Get-ConductorRetryDecision`、`Get-ConductorRetryWait`、`Test-ConductorRetryTarget` | リトライの判断と待ち時間(`[bigint]` の飽和演算) |
| `New-ConductorInitialState`、`Add-ConductorOutcome`、`Add-ConductorSkip`、`Add-ConductorInterrupt` | 記録関数(元のステートを変えず、新しいステートを返す純粋な関数) |
| `Get-ConductorExitCode` | 終了コードの範囲合わせ(0〜255) |
| `Compare-ConductorState` | ステートの等価性(元と等しいとき書かない判断に使う) |
| `Add-ConductorSaturatedMultiply` | `[bigint]` を使う 64 ビット符号なしの飽和した積(折り返さない) |

### 3.5 `Private/Definition/`

| 関数 | 責務 |
|------|------|
| `Read-ConductorDefinitionFile` | 定義ファイルの場所の決定と読み取り(Intake)。拡張子の判定、大きさの上限。`Load` に相当 |
| `Read-ConductorDefinitionText` | 文字列に対する純粋な読み取りと検査(`Parse` に相当) |
| `Get-ConductorDefinitionFormat` | 拡張子から書式(`psd1` または `json`)を判定する(大文字小文字を区別しない) |
| `Read-ConductorPsd1Tree` | `.psd1` の文字列を、`[System.Management.Automation.Language.Parser]` で構文木にし、許可する構文だけを木に変換する |
| `Test-ConductorDefinitionDepth` | 入れ子の深さの検査 |
| `Test-ConductorDefinitionStructure` | 構造・型・遷移・終了コード・名前の検査(`Walk`) |
| `Read-ConductorLooseRunIdEnv` | `run_id_env` だけのゆるい読み取り(どの失敗でも `$null`) |
| `New-ConductorDefinitionError` | 定義の誤り(種類、ファイル、行、列、理由)を作る |

### 3.6 `Private/Convert/`

| 関数 | 責務 |
|------|------|
| `Invoke-ConductorConvertFile` | 変換の入口(入力ファイル 1 つを読み、`--to` の形式の文字列を返す) |
| `Convert-ConductorText` | 文字列の変換(純粋) |
| `ConvertTo-ConductorPsd1Text` | 木を `.psd1` の文字列に書き出す |

### 3.7 `Private/State/`

| 関数 | 責務 |
|------|------|
| `Get-ConductorStatePath` | ステートファイルの場所と名前 |
| `Read-ConductorState`、`Read-ConductorStateRaw` | 読み込み(14 段階の検査、異常系。`Raw` は完了のものも返す。`watch` 用) |
| `Write-ConductorState` | 書き込み(一時ファイル、同期、名前の付け替え、必要ならディレクトリの同期)。`-Durability`、`-SyncDirectory`(スクリプトブロック) |
| `Reset-ConductorState`、`Reset-ConductorStateAll` | `reset` と `reset --all` |
| `Test-ConductorStateConsistency` | 定義との整合性の検査(純粋) |
| `ConvertTo-ConductorStateJson`、`ConvertFrom-ConductorStateJson` | ステートの論理値と JSON の変換(`Json/` を使う) |
| `New-ConductorStateError` | ステートの誤り(種類、パス、理由)を作る |

### 3.8 `Private/Runner/`

| 関数 | 責務 |
|------|------|
| `Split-ConductorCommandLine` | `command` の分割(状態機械) |
| `Test-ConductorCommand` | `command` の検査 |
| `Invoke-ConductorJob` | ジョブの起動と待ち |
| `Wait-ConductorChild` | 待機ループ |
| `Wait-ConductorRetry` | リトライの待機 |
| `New-ConductorInterrupt` | 中断の合図(C# ヘルパーの `Interrupt`)を作る |
| `Start-ConductorSignalWatch` | シグナル監視の開始 |
| `Initialize-ConductorNativeHelper` | C# ヘルパーの読み込み(`Add-Type`。ここだけ) |
| `Get-ConductorPlatformKind` | OS の判定(ここだけ。`'Unix'` または `'Windows'`) |
| `Get-ConductorStartInfo` | `ProcessStartInfo` の組み立て(OS 固有の調整を含む) |
| `Resolve-ConductorUnixExecutable` | UNIX 系の実行ファイルの解決 |
| `ConvertTo-ConductorCmdArguments` | Windows の `.cmd`/`.bat` の起動の形(純粋) |
| `Send-ConductorSignal` | シグナルを子の pid に 1 回送る(`kill`) |
| `Read-ConductorExit` | 終了から `ExitCode` と `TerminatedBySignal` を決める |
| `Sync-ConductorDirectory` | ディレクトリの同期(C# ヘルパー経由) |

ファイル `NativeHelper.ps1` は、C# のソース(here-string)と、`Initialize-ConductorNativeHelper`、`Sync-ConductorDirectory`、`Send-ConductorSignal` の中の、ヘルパーの型の呼び出しを持つ(`Add-Type` と P/Invoke の許可箇所。[07-process-and-signals.md](07-process-and-signals.md) の 9.2 節)。

### 3.9 `Private/Report/`

| 関数 | 責務 |
|------|------|
| `Write-ConductorOutput`、`Write-ConductorErrorLine` | 標準出力・標準エラーへの書き込みの窓口(書くたびに flush。製品のコードが画面に書く唯一の経路) |
| `Get-ConductorLocale`、`Test-ConductorColor` | ロケールと色の判定(最初に必要になったとき 1 回) |
| `Get-ConductorMessage` | メッセージカタログ(英語・日本語) |
| `Format-ConductorVerboseResult`、`Format-ConductorDefinitionErrors`、`Format-ConductorCommandError`、`Format-ConductorStateError`、`Format-ConductorWatchTable`、`Get-ConductorUsageText` | 行・表・使い方を作る(純粋。書き込みはしない) |
| `ConvertTo-ConductorEscapedText` | 入力由来の文字列のエスケープと切り詰め |
| `Write-ConductorInternalFailure` | 想定外の内部の誤りの固定の 1 行([07-process-and-signals.md](07-process-and-signals.md) の 11 章) |

### 3.10 `Private/Cli/`(最上位)

| 関数 | 責務 |
|------|------|
| `Invoke-ConductorMain` | 引数と `Deps` を受け取り、終了コードを返す(`Invoke-Conductor` の本体。テストが `Deps` を渡して呼ぶ) |
| `ConvertFrom-ConductorArguments` | 引数の字句の解析と検査(誤りの優先順位) |
| `Invoke-ConductorPrepare` | `run`・`pipeline` の前処理(前処理の順序と失敗の終了コード) |
| `Invoke-ConductorRunCommand`、`Invoke-ConductorPipelineCommand` | `run`・`pipeline` の流れ |
| `Invoke-ConductorExecJob` | リトライ込みの 1 ジョブの実行、記録、表示(停止経路の `finally` を持つ) |
| `Invoke-ConductorWatchCommand`、`Invoke-ConductorResetCommand`、`Invoke-ConductorValidateCommand`、`Invoke-ConductorConvertCommand` | 補助のサブコマンド |
| `New-ConductorSystemDeps` | 本番の `Deps` を作る(4.3 節) |
| `Get-ConductorVersion` | 版の取り出し(10 章) |

### 3.11 `Public/`

| 関数 | 責務 |
|------|------|
| `Invoke-Conductor` | 公開する唯一の関数。`$args` を受け、本番の `Deps` で `Invoke-ConductorMain` を呼び、`[int]` を返す。想定外の内部の失敗を捕まえて 70 にする(6.1 節) |

### 3.12 ファイル単位の責務の補足【目安】

実装者は、ファイルの分け方を変えてよい。ただし、ディレクトリの責務と依存の向き(4 章)と、OS 分岐・`Add-Type` の置き場所は守る。`tests/` の対応: `tests/<ディレクトリ名>/<関数名>.Tests.ps1`(`src` と同じ構成)。

---

## 4. 依存の向き【規定】

矢印は「呼び出す(その関数を呼ぶ)」。**循環させない**。

```text
Public ─▶ Cli

Cli ─▶ Report, Runner, State, Flow, Definition, Convert, RunId, Json
Report ─▶ (なし。データを受け取って文を作るだけ)
Runner ─▶ Flow
State  ─▶ Flow, RunId, Json
Convert ─▶ Definition, Json
Definition ─▶ Json
Flow   ─▶ (なし)
RunId  ─▶ (なし)
Json   ─▶ (なし)
```

図(下へ向かって依存する):

```text
                       Public
                         │
                        Cli ───────────────┐
   ┌──────┬────────┬─────┴──┬─────────┐     │ (Cli が直接使う)
 Report  Runner   State   Convert   RunId   Definition
            │       │ ╲       │
            ▼       ▼  ╲      ▼
           Flow ◀───┘   ╲   Definition ─▶ Json
                         ╲──────────────▶ Json, RunId
```

規則:

1. **`Cli` が最上位**(`Public` を除く)。ほかのディレクトリは `Cli` の関数を呼ばない。`Report` も呼ばない。
2. **`Json`、`RunId`、`Flow` は最下層**。互いに呼ばず、ほかのディレクトリの関数を呼ばない。入出力に触れない(`Flow` は純粋)。
3. **`State` と `Runner` は、`Flow` の関数を使うが、互いに依存しない**。`State` は「ステートを読み書きする」だけで、ジョブを起動しない。`Runner` は「ジョブを起動して待つ」だけで、ステートに触れない。両者をつなぐのは `Cli`。ディレクトリの同期は、`Runner` の `Sync-ConductorDirectory` を、`Cli` が `Deps` の `SyncDirectory`(スクリプトブロック)として `State` に渡す(`State` は `Runner` を呼ばず、OS の判定も持たない)。
4. **`Convert` が呼ぶのは `Definition` と `Json` だけ**(木の読み取りを共有)。定義の検証(`Test-ConductorDefinitionStructure`)は呼ばない。
5. **`Report` は、データを受け取って文を作る**。判断をしない。何を表示するかは `Cli` が決める。環境変数と端末の判定は、`Deps` の `Env`(スクリプトブロック)を、引数で受け取る。
6. **OS ごとの分岐は、`Runner` の `Get-ConductorPlatformKind` と、プラットフォーム層の関数(`Get-ConductorStartInfo`、`Send-ConductorSignal`、`Read-ConductorExit`、`Sync-ConductorDirectory`)と、`NativeHelper.ps1` だけ**([07-process-and-signals.md](07-process-and-signals.md) の 9.2 節)。
7. **`Deps` を通した呼び出しは、`Cli` だけが行う**(`& $Deps.Proc.RunJob ...`)。下位の関数は `Deps` を知らず、必要なスクリプトブロックを引数で受け取る。

### 4.1 強制の方法【規定】

PowerShell は、名前空間の強制を持たない(関数は、読み込まれたすべてが、モジュールのスコープから呼べる)。そのため、テストで強制する。

`tests/Architecture.Tests.ps1`(点検テスト)が、次を行う。外部のモジュール(アーキテクチャを検査する専用のもの)を使わず、`[System.Management.Automation.Language.Parser]::ParseFile` と AST の走査だけで書く。

1. `src/Conductor/Private/**/*.ps1` と `Public/*.ps1` を、すべて `ParseFile` で構文木にする(`Parser` は静的メソッド。`out` 引数の `[ref]` で、トークンと構文エラーも受ける。構文エラーがあれば、点検テストの失敗)。
2. 各ファイルの `FunctionDefinitionAst`(`$ast.FindAll({ $args[0] -is [System.Management.Automation.Language.FunctionDefinitionAst] }, $true)`)から、「関数名 → ディレクトリ」の表を作る。
3. 各関数の本体の `CommandAst` のうち、最初の要素(`CommandElements[0]`)が定数の文字列(`StringConstantExpressionAst`)で、その名前が手順 2 の表にあるものを集め、「呼び出し元のディレクトリ → 呼び出し先のディレクトリ」の辺を作る。
4. 辺の集合が、上の 4 章の許可の表(`Cli` は全部、`Runner` は `Flow`、`State` は `Flow`/`RunId`/`Json`、`Convert` は `Definition`/`Json`、`Definition` は `Json`、そのほかは同じディレクトリの中だけ)に含まれることを検査する。同じディレクトリの中の呼び出しは許す。許されない辺があれば、失敗にする(呼び出し元の関数名と呼び出し先の関数名を失敗の文に書く)。
5. ディレクトリのあいだのグラフに、循環がないこと(深さ優先の走査)を検査する。
6. **動的な呼び出しの禁止**: `&` や `.` の呼び出しで、最初の要素が定数でない `CommandAst`(例: `& $name`、`& "Get-$x"`)は、`Cli` の `Deps` を通した呼び出し(最初の要素が、`$Deps` から始まるメンバーアクセス、または `Deps` 由来の変数)を除いて、失敗にする(静的な走査で追えない辺を作らないため)。`Invoke-Expression`、`Invoke-Command -ScriptBlock` の変数を取る形も、禁止の検査(11 章)で失敗にする。
7. 関数名の接頭辞の検査: すべての関数名が `Verb-ConductorNoun` の形で、`Verb` が `Get-Verb` の一覧にあること(PSScriptAnalyzer の規則と二重の備え)。

制約: ファイルの文字列の走査と違い、AST の走査は、別名(`alias`)、`& (Get-Command ...)`、パイプラインのスクリプトブロックの中の呼び出しを、辺として拾える(`FindAll` は、ネストしたスクリプトブロックも検索する)。`Set-Alias` による別名を、製品のコードで使わない(辺を追えないため。PSScriptAnalyzer の `PSAvoidUsingCmdletAliases` と合わせて禁止)。

---

## 5. モジュールの構成【規定】

### 5.1 `Conductor.psd1`(マニフェスト)

マニフェストは、キーと値のハッシュテーブルを持つ PowerShell データファイル(`.psd1`)である。`New-ModuleManifest` で作り、手で整える(確認。出典: 「How to Write a PowerShell Module Manifest」。`ModuleVersion` 以外のキーは、すべて任意)。

| キー | 値 | 規定 |
|------|----|------|
| `RootModule` | `'Conductor.psm1'` | ルートのスクリプトモジュール |
| `ModuleVersion` | `'0.8.0'` | **版の唯一の出所**(10 章) |
| `GUID` | 固定の GUID(最初に 1 回作り、変えない) | 識別子 |
| `Author`、`CompanyName`、`Copyright`、`Description` | 実装時に決める | 公開の情報 |
| `PowerShellVersion` | `'7.4'`(実装時点の最小の LTS に合わせて決める。サポート中かを実装時に確認する。7.4 のサポート期間が、この文書の作成時点(2026-11)で終わっているかは未確認) | 最小の版。`#requires -Version` と、ランチャーの検査に使う |
| `CompatiblePSEditions` | `@('Core')` | Core のエディション(`pwsh`)だけ(確認: マニフェストの標準のキー。この文書では個別の出典を確認していない。未確認) |
| `FunctionsToExport` | `@('Invoke-Conductor')` | 公開する関数。ワイルドカードを使わない(性能のため。確認。出典: 同上のサンプルのコメント「for best performance, do not use wildcards and do not delete the entry」) |
| `CmdletsToExport` | `@()` | 空の配列(キーを消さない) |
| `VariablesToExport` | `@()` | 空の配列(サンプルの既定は `'*'` だが、変数を公開しない) |
| `AliasesToExport` | `@()` | 空の配列 |
| `PrivateData` | `@{ PSData = @{ Tags = @('...'); ProjectUri = '...'; LicenseUri = '...' } }` | `PSData` の中に、タグ、URL。PowerShell Gallery への公開は、人が手で行う([11-quality-and-build.md](11-quality-and-build.md)) |

- `Test-ModuleManifest` が、品質ゲートで成功すること([11-quality-and-build.md](11-quality-and-build.md))。
- マニフェストは、データだけを持つ(コード、`$PSScriptRoot` 以外の変数を持たない。確認: マニフェストは `RestrictedLanguage` で処理され、許可される変数が限られる。出典: `about_Language_Modes` の RestrictedLanguage の節)。
- `RequiredModules`、`NestedModules`、`RequiredAssemblies`、`ScriptsToProcess`、`TypesToProcess`、`FormatsToProcess` を使わない(実行時の依存なし。起動の遅さを避ける)。`FileList` は、パッケージの検査用に、実装時に決めてよい。

### 5.2 `Conductor.psm1`(ルートモジュール)

骨格(本体の文を並べない。読み込みの順序と、規律だけ):

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# 版(10 章): マニフェストから 1 回だけ読む
$script:ConductorVersion = (Import-PowerShellDataFile -LiteralPath (Join-Path $PSScriptRoot 'Conductor.psd1')).ModuleVersion

# 読み込みの順序(4 章の依存の下層から。ドットソース)。固定の配列で、`Get-ChildItem` の並びに頼らない
$private = @('Json', 'RunId', 'Flow', 'Definition', 'Convert', 'State', 'Runner', 'Report', 'Cli')
foreach ($dir in $private) {
    foreach ($file in <# そのディレクトリの *.ps1 を、ファイル名の序数の昇順で #>) {
        . $file
    }
}
. (Join-Path $PSScriptRoot 'Public/Invoke-Conductor.ps1')

Export-ModuleMember -Function 'Invoke-Conductor'
```

規定:

| 項目 | 規定 |
|------|------|
| `Set-StrictMode -Version Latest` | ファイルの先頭。モジュールのスコープに効く(確認: `Set-StrictMode` は、設定したスコープとその子のスコープに効く。公式の文書の記述。この文書では個別の出典を確認していない。未確認)。`Latest` は、将来の `pwsh` の版で、厳しくなりうる。テストの行列(最小の版と最新の版)で検出する |
| `$ErrorActionPreference = 'Stop'` | `Set-StrictMode` の直後。非終了エラーも終了エラーにして、`catch` に届ける。モジュールの関数が、呼び出し元の設定ではなく、モジュールの値を使うこと(モジュールは、独立したセッション状態を持つ)は、公式の文書で確かめていない(未確認。T-S2) |
| ドットソースの順序 | 4 章の依存の下層から上へ(`Json`、`RunId`、`Flow`、`Definition`、`Convert`、`State`、`Runner`、`Report`、`Cli`、`Public`)。同じディレクトリの中は、ファイル名の序数の昇順。関数の定義は、呼び出されるより前に読み込まれる必要はない(関数は実行時に解決される)ので、順序は、`class` を使わない限り、動作に影響しない。順序は、検査の再現性と、起動時のエラーの再現のために固定する |
| 起動の速さ | ドットソースは、ファイルを 1 つずつ構文解析する。ファイル数が多いと、`Import-Module` が遅くなる。【目安】ビルドで、すべての `.ps1` を 1 つの `Conductor.psm1` に結合した配布物を作ってよい(`build/` のスクリプト。結合の順序は、上と同じ)。開発時は、ドットソースの形で動かす。どちらの形でも、テストを通す。起動時間の計測で、結合の要否を決める(11 章) |
| `Export-ModuleMember` | `Invoke-Conductor` だけ。マニフェストの `FunctionsToExport` と同じにする(二重の備え) |
| 出力 | ファイルの読み込み中に、何も出力しない(`Import-Module` の出力が混ざらない) |
| `$PSScriptRoot` | `.psm1` の中では、ファイルのあるディレクトリ(確認: PowerShell の自動変数。この文書では個別の出典を確認していない)。ドットソースされた `.ps1` の中の `$PSScriptRoot` は、その `.ps1` のあるディレクトリ(ドットソースのスコープでも、スクリプトのファイルの場所を指す。未確認。実装は、`$PSScriptRoot` を、`.psm1` の中で変数に取って、`$script:` で渡す) |

### 5.3 `class` の定義の順序の制約と、この版の決定

PowerShell の `class` には、次の制約がある(確認。出典: `about_Classes` の Limitations と関連の節)。

- クラスは、セッションの中で、アンロードも再読み込みもできない。`Import-Module -Force` は、ルートモジュールの関数の変更だけを読み直し、更新したクラスは読み込めない。新しいセッションが要る。
- モジュールの中で定義した `class` は、自動では、モジュールの外へ公開されない。`using module` 文(スクリプトの先頭)か、型アクセラレーターの登録が要る。`using module` は、ルートモジュールの直下で定義したクラスだけを、確実に読み込む。ルートモジュールにドットソースされたスクリプトの中のクラスは、一貫して読み込まれるとは限らない。
- `class` の型は、解析の時点で解決される。あるファイルのクラスを使う別のファイルは、そのクラスの定義より後に解析されなければならない。
- クラスのメソッドは、既定で、作った runspace に結び付く(別のスレッドから呼ぶと、状態の破損・デッドロックの恐れ。`[NoRunspaceAffinity()]` は 7.4 で追加)。
- PowerShell のスクリプトでは、インターフェースを定義できない(C# で定義して、アセンブリを参照する)。

**決定【規定】: 製品のコードで、PowerShell の `class` と `enum` を定義しない。** 理由: (1) 上の制約(再読み込みができない。ドットソースのファイルの分割と相性が悪い。テストが `InModuleScope` なしでは型を参照できない)。(2) `Import-Module -Force` を使う開発とテストの繰り返しで、古いクラスが残る。(3) 値は `[pscustomobject]`(`[ordered]` のハッシュテーブルから作る)で十分で、`Set-StrictMode` が、存在しないプロパティの参照を例外にする。(4) 列挙値は、文字列(`'Success'`、`'Warning'`、…)で表し、比較は `-ceq`(大文字小文字を区別する)で行う。

値の作り方の規定: 値を作る関数(`New-Conductor...`)が、プロパティ名と型を 1 か所で決める。`[pscustomobject]@{ ... }` の代わりに、`[pscustomobject][ordered]@{ ... }` で、プロパティの順序を保つ(表示と JSON の順序のため)。プロパティの値の型は、`[int]`、`[string]` などのキャストで明示する。`$null` を許すプロパティは、型を付けず、`$null -eq $x` で判定する。

将来、`class` が要る場合の規則(この版では使わない。ただし、記しておく): (1) `class` は、ルートモジュール `Conductor.psm1` の直下に、同じファイルで定義する。(2) 継承は、同じファイルの中で、基底を先に書く。(3) テストは `using module` を使うか、`InModuleScope` の中で使う。(4) `[NoRunspaceAffinity()]` を付ける(7.4 以降)。

### 5.4 `Conductor.psm1` の起動の遅延

- `Initialize-ConductorNativeHelper`(`Add-Type`)は、`run`/`pipeline` の準備まで呼ばない([07-process-and-signals.md](07-process-and-signals.md) の 7.3 節)。モジュールの読み込み時に、`Add-Type` を呼ばない【規定】。
- ロケールと色の判定も、最初に必要になったときまで行わない([08-output-and-i18n.md](08-output-and-i18n.md))。
- `--version`、`--help` は、定義・ステート・変換・ヘルパーに触れない。

### 5.5 ランチャー

引数の受け渡しの前提(確認。出典: `about_Pwsh` の `-File` の節): `-File` の値のあとのすべての文字は、スクリプトのファイルパスと、スクリプトへの引数として解釈される。引数は、現在のシェルの解釈を経たあとの、リテラルの文字列としてスクリプトに渡される。`cmd.exe` では、環境変数は `%VAR%` の形が展開される。`File` は、`pwsh` の最後のパラメータでなければならない。Windows では、`-File` は `.ps1` だけを受け付ける(7.2 以降)。`-NoProfile`、`-NonInteractive`、`-NoLogo` は、`pwsh` のパラメータ(確認。出典: `about_Pwsh`。`-NonInteractive` は、対話的な機能の使用を、ハングではなく、終了エラーにする)。

#### `bin/conductor.ps1`(本体のランチャー)【規定】

役割: モジュールを読み込み、`Invoke-Conductor` を呼び、終了コードで `exit` する。`param` ブロックを持たない(`$args` で、すべての引数を、文字列のまま受ける。`-v` などが、スクリプト自身のパラメータとして束縛されないため)。骨格:

```powershell
#requires -Version 7.4            # PowerShellVersion と同じ値(版の一致の検査。11 章)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$exitCode = 70
$returned = $false
try {
    Import-Module (Join-Path $PSScriptRoot '../src/Conductor/Conductor.psd1') -Force:$false
    $exitCode = Invoke-Conductor @args        # 配列のスプラット。引数は文字列のまま渡る
    $returned = $true
}
catch [System.Exception] {                    # 読み込みの失敗など。固定の 1 行を出して 70
    <# conductor: error: internal error を標準エラーに 1 回だけ(例外の内容は出さない) #>
}
finally {
    if (-not $returned) {
        <# 停止経路(Ctrl+C など): 07 章 7.7 節。予約された終了コード、なければ 130 で [System.Environment]::Exit(code) #>
    }
}
exit $exitCode
```

- 配置の前提: 配布物では、`bin/` の隣に `src/Conductor/`(または、配布の形に合わせた相対のパス)を置く。ランチャーは、`$PSScriptRoot` からの相対で、モジュールのマニフェストを読み込む(カレントディレクトリに依存しない)。`Import-Module` に、マニフェストの完全なパスを渡す。
- **`Invoke-Conductor @args` の注意**: 配列のスプラットは、要素を位置の引数として渡す。要素が `-v`、`--`、`-?`、`-Verbose` のような文字列でも、`Invoke-Conductor` の関数に `param` ブロックがなければ(6.1 節)、すべて `$args` に入る(確認していない。T-5 で確かめる。[07-process-and-signals.md](07-process-and-signals.md) の 12.2 節)。
- ランチャーの `exit $exitCode` は、`exit` を呼ぶ唯一の箇所(と、停止経路の `Environment.Exit`)である。`Invoke-Conductor` が返す値は `[int]` で、0〜255 に合わせ済みである。
- `$ErrorActionPreference` と、`trap`: ランチャーに `trap` を置かない。`catch [System.Exception]` は、`PipelineStoppedException` を捕まえない場合がある([07-process-and-signals.md](07-process-and-signals.md) の 7.7 節。未確認)。停止は `finally` で扱う。
- `pwsh` の異常終了(モジュールの構文エラー、`pwsh` 自身の失敗)は、ランチャーの外側になる場合があり、終了コードが 1 になる(確認。出典: `about_Pwsh` の `-File` の節の「when a script-terminating error occurs, the exit code is set to 1」)。70 にならない場合がある。許容する(11 章の受け入れテストは、ランチャーが読み込みに成功する前提)。

#### `bin/conductor`(UNIX 系)【規定】

`/bin/sh` のスクリプト(先頭 `#!/bin/sh`)。自分のディレクトリを求め、`exec pwsh -NoLogo -NoProfile -NonInteractive -File "$dir/conductor.ps1" "$@"` で、本体のランチャーに引き継ぐ。

- 引数 `"$@"` を、そのまま渡す(`sh` の引用で、空白と空文字列の引数が保たれる)。`--` を足さない(足すと、スクリプトの `$args` の先頭に `--` が入るおそれがある。未確認)。
- 自分のディレクトリの求め方: シンボリックリンク(`/usr/local/bin/conductor` からの)を、`readlink` で追うかは、実装時に決める。追わない場合は、配布の形(zip、tar.gz)が、リンクを要しない構成であること。
- `pwsh` が `PATH` にないとき、固定の英語の 1 行 `conductor: error: pwsh not found` を標準エラーに書き、終了コード 127 で終わる(コマンドが見つからない場合のシェルの慣習。メッセージの文言は、他の文書と合わせる)。
- `#!/usr/bin/env pwsh` を先頭に持つ `.ps1` の形(ランチャーを 1 つにする案)は、`-NoProfile` を渡す方法が、`env -S` に依存し(`env -S` が macOS と各 Linux で使えるかは未確認)、`-File` の引数の解釈も不確かなため、採らない(`pwsh` のプロファイルが読まれると、起動が遅くなり、`conductor` の出力が汚れる)。
- 実行ビットは、配布物で保つ(zip と tar.gz。11 章)。

#### `bin/conductor.cmd`(Windows)【規定】

`@echo off` のあと、`pwsh -NoLogo -NoProfile -NonInteractive -File "%~dp0conductor.ps1" %*` を呼び、`exit /b %ERRORLEVEL%` で、`pwsh` の終了コードをそのまま返す。

- `%~dp0` は、バッチファイルのあるディレクトリ(確認。出典: `about_Pwsh` の `-File` の節の `%~dp0` の例)。`.\conductor.ps1` と書かない(カレントディレクトリに依存し、見つからない。同じ節)。
- `-ExecutionPolicy Bypass` を付けない(実行ポリシーの回避を、この製品が勝手に行わない。ダウンロードした `.ps1` が `Zone.Identifier` でブロックされるときの案内は、利用者向けの文書に書く。[11-quality-and-build.md](11-quality-and-build.md))。
- `%*` の展開: `cmd.exe` は、`%*` を、引数を元のまま(引用符つきで)展開する。引数の中の `%` は、バッチファイルの中では、`%%` と書かない限り、展開されるが、`%*` の展開の結果は、再度は展開されない(確認していない。未確認)。`^`、`&`、`|`、`<`、`>` は、利用者が `cmd.exe` のコマンドラインで引用符で囲まずに書くと、`cmd.exe` が先に解釈する(ランチャーの責任の外。利用者向けの文書に、引用符で囲む注意を書く)。引用符で囲まれた引数の中の `"`(`\"`)の扱い、`!` の遅延展開は、確かめていない(未確認。11 章の Windows 専用の項目で確かめる)。
- `ERRORLEVEL` の値が、`pwsh` の終了コードと一致すること(確認していない。未確認)。
- `pwsh` が見つからないとき、`cmd.exe` が、`'pwsh' is not recognized ...` を出し、`ERRORLEVEL` は 9009 になる(確認していない。未確認。固有の処理はしない)。

---

## 6. 公開する関数の骨格

### 6.1 `Invoke-Conductor`(`Public/Invoke-Conductor.ps1`)【規定】

```powershell
# 引数 ($args): conductor の引数(プログラム名を除く)。文字列のまま。
# 戻り値: [int] の終了コード(0〜255)。exit は呼ばない。
function Invoke-Conductor {
    # param ブロックを持たない。[CmdletBinding()] を付けない。
    # 本体: 1) 内部の誤りの捕捉を、最初に設定する(try/catch [System.Exception])。
    #       2) New-ConductorSystemDeps で本番の Deps を作る。
    #       3) Invoke-ConductorMain -Arguments ([string[]]$args) -Deps $deps を呼び、結果の [int] を返す。
    #       4) 想定外の例外は、固定の 1 行を標準エラーに書き、70 を返す(07 章 11 章)。
}
```

- **`param` ブロックと `[CmdletBinding()]` を持たない理由**: `[CmdletBinding()]` を付けると、共通パラメータ(`-Verbose`、`-Debug`、`-ErrorAction` など)が付き、PowerShell のパラメータの束縛が、これらの名前の短縮(`-v` は `-Verbose` の短縮として解釈されうる)を先に解釈し、`conductor` の `-v` オプションを奪う。`param` ブロックのない関数は、すべての引数を `$args` に入れる(確認していない。T-5)。この規定を、実装時に最初に確かめる。確かめて、食い違うなら、`-ArgumentList` の名前つきの 1 つのパラメータ(`[Parameter(ValueFromRemainingArguments)] [string[]]$ArgumentList`)に変えて、ランチャーが `-ArgumentList $args` で渡す形にする(`-` で始まる引数が、パラメータ名として解釈されない保証は、この形でも未確認)。
- **`--` の扱い**: `conductor` の文法の `--`(以降を、オプションとして解釈しない)が、PowerShell のコマンド呼び出しの `--`(パラメータの終わり)として、`$args` に入る前に消えないことを、確かめる(T-5)。ランチャーが、配列を作って(`$args` をそのまま)スプラットする形なら、要素の `--` は、リテラルとして渡ると想定する(未確認)。消える場合の回避策: ランチャーが、`$args` を `[string[]]` の変数に入れ、`Invoke-ConductorMain` を直接呼ぶ(`Invoke-Conductor` を経由せず、`Invoke-ConductorMain` を呼べるよう、モジュールの内部の関数を、ランチャーから呼ぶ形に変える)。この回避策は、公開する関数を 1 つにする規定に反するため、確かめた結果で決める(実装時の判断として、この文書を直す)。
- `-h` と `--help`、`-V` と `--version` の解釈は、`conductor` の引数の解析([04-cli.md](04-cli.md))が行う。PowerShell は、`$args` の文字列としてしか見ない。

### 6.2 `Invoke-ConductorMain`(非公開。`Private/Cli/`)

```powershell
# arguments: [string[]]。deps: 4.2 節の Deps(ordered ハッシュテーブル)。
# 戻り値: [int] の終了コード。例外を送出しない(想定外の内部の失敗を除く)。Invoke-Conductor と、単体テストが呼ぶ。
function Invoke-ConductorMain {
    param([string[]]$Arguments, [System.Collections.IDictionary]$Deps)
}
```

---

## 7. `Deps`(差し替え口)とテスト用の偽実装【規定】

### 7.1 方針

`Invoke-ConductorMain` は、環境・時刻・標準出力/標準エラー・ファイル(定義・ステート・変換)・プロセス・シグナルを、**まとめて 1 つの `Deps`** で受け取る。本番の入口(`New-ConductorSystemDeps`)は本番の実体を渡し、テストは偽の実体を渡す。純粋な判断(`Flow` の関数、記録関数、`Report` の文の組み立て)は、`Deps` を通さず、`Cli` が直接呼ぶ(差し替え不要)。

### 7.2 インターフェースの表し方の決定

| 案 | 内容 | 採否 |
|----|------|------|
| (a) `class` の継承(基底クラス + 偽のクラス) | メソッドを持つクラスを定義し、テストで継承する | 不採用。5.3 節の制約(再読み込みができない。型がモジュールの外から見えない。runspace への結び付き) |
| (b) **名前つきのスクリプトブロックを持つ `[ordered]` のハッシュテーブル** | 各口をキー、実体をスクリプトブロックにする。呼び出しは `& $Deps.Proc.RunJob $job $sig` | **採用**。型の制約がない。偽の実体は、キーを置き換えるだけで作れる。スクリプトブロックは、デリゲートに変換せず、`&` で呼ぶ(同じスレッドの中だけ。[07-process-and-signals.md](07-process-and-signals.md) の 10.1 節) |
| (c) Pester の `Mock` だけに頼る | 関数を `Mock` で差し替える | 部分的に併用。`Mock` は、関数(コマンド)を差し替える仕組みで、モジュールの非公開の関数に使うには `-ModuleName Conductor` か `InModuleScope` が要る(確認していない。未確認。Pester の文書で、実装時に確認)。`Mock` は、呼び出しの記録と検証(`Should -Invoke`)に便利だが、`Deps` の口(スクリプトブロック)は、コマンドではないので、`Mock` できない。そのため、**口は (b) の `Deps` で差し替え、関数の呼び出しの回数の検査などが要る箇所だけ `Mock` を併用する** |

### 7.3 `Deps` の定義【規定】

`Deps` は、次のキーを持つ `[ordered]` のハッシュテーブル。値は、スクリプトブロックを持つハッシュテーブル(口ごとに 1 つ)。`Test-ConductorDeps`(内部。`Cli`)が、`Invoke-ConductorMain` の最初に、すべてのキーとスクリプトブロックの存在を検査する(足りなければ、想定外の内部の誤り。ハッシュテーブルの存在しないキーの参照が、`Set-StrictMode` で例外になるか `$null` を返すかは、公式の文書で確かめていない(未確認)ため、`ContainsKey` で明示的に検査する)。

| キー | 口(メンバー) | シグネチャ(引数 → 戻り値) | 本番の実体 |
|------|----------------|------------------------------|------------|
| `Env` | `GetEnvironmentVariable` | `name:[string]` → `[string]` または `$null` | `[System.Environment]::GetEnvironmentVariable($name)`(`$env:` の読み取りは、大文字小文字の扱いが OS で違うため、使わない) |
| | `IsStdoutTerminal` | → `[bool]` | `-not [Console]::IsOutputRedirected`(挙動は 3 OS で確かめる。未確認) |
| | `Platform` | → `'Unix'` / `'Windows'` | `Get-ConductorPlatformKind` |
| | `Cwd` | → `[string]` | `[System.IO.Directory]::GetCurrentDirectory()`(`$PWD` を使わない) |
| `Clock` | `Now` | → `[System.DateTimeOffset]` | `[System.DateTimeOffset]::UtcNow`(ステートの一時ファイル名に使う) |
| `Out` | `Stdout`、`Stderr`(口ごとに `WriteAll`) | `text:[string]` → なし。書いて flush する。失敗は、`[System.IO.IOException]` などで伝える(08 章) | `[Console]::OpenStandardOutput()` / `[Console]::OpenStandardError()` から作った UTF-8(BOM なし)のストリームに書く |
| `Defs` | `Load` | `path`、`expectedPipelineName` → 結果(`Ok` と定義、または失敗の列) | `Read-ConductorDefinitionFile` |
| | `LooseRunIdEnv` | `path` → `[string]` または `$null` | `Read-ConductorLooseRunIdEnv` |
| `States` | `Read`、`ReadRaw`、`Write`、`Reset`、`ResetAll` | 06 章が正。失敗は値で返す | `Read-ConductorState` ほか。`Write` は `-SyncDirectory`(下の `Proc` と別の口。`Cli` が `Sync-ConductorDirectory` を包んで渡す。`$kind -eq 'Unix'` のとき真) |
| `Converter` | `Convert` | `inputPath`、`to` → 結果 | `Invoke-ConductorConvertFile` |
| `Proc` | `CheckCommand` | `command` → 結果 | `Test-ConductorCommand` |
| | `RunJob` | `job`、`sig`、`progress` → `JobExecution` | `Invoke-ConductorJob` |
| | `Wait` | `secs:[uint64]`、`sig` → `Elapsed` / `Interrupted(番号)` | `Wait-ConductorRetry` |
| `Signals` | `Start` | `sig` → `'Started'` / `'AlreadyRegistered'` / `'Failed'` | `Start-ConductorSignalWatch` |

- 口の呼び出しは、`& $Deps.Proc.RunJob -Job $job ...` のように、`&` で行う。スクリプトブロックの引数は、`param(...)` で受ける。`$_` と `$args` に頼らない。
- ファイルシステムの差し替えは、`Defs`、`States`、`Converter` の 3 つの口が担う。本番の実体は、実ファイルを読み書きする。偽の実体は、メモリに持つ(ファイルなし)。ステートの書き込みの原子的な置換の失敗は、`State` の内側のテスト(一時ディレクトリの実ファイル)で行う。
- すべての口は、同期の関数(スクリプトブロック)である。`Proc`、`Defs`、`States`、`Converter` の口は、例外を送出しない(失敗は値で返す)。`Out` の `WriteAll` だけは、失敗を例外で伝える。**例外の扱いは、`Report` の窓口の 1 か所**(標準出力は専用の例外に変えて呼び出し元へ、標準エラーは握りつぶす。08 章)。
- 書き込みは `Report` の窓口の 1 か所を通す(順序を保つ。書くたびに flush する)。

### 7.4 偽の実装(`tests/support/`)【規定】

偽の実装は、次の振る舞いを持つ(名前は実装者が決める。関数 `New-FakeDeps` が、本番と同じキーのハッシュテーブルを返し、テストが必要な口だけを置き換える)。

| 偽の実装 | 振る舞い | 主な検査 |
|-----------|----------|----------|
| `Env` | 環境変数を表(ハッシュテーブル)で与える。`GetEnvironmentVariable` を呼んだ名前と回数を記録する。`IsStdoutTerminal`、`Platform`、`Cwd` の値を設定する | ロケール(`LC_ALL`・`LC_MESSAGES`・`LANG`)・`NO_COLOR`・run ID の環境変数。ロケールの判定を**遅延**して、最初に必要になったとき 1 回だけであること |
| `Clock` | 固定の時刻を返す。進める操作を持つ | 一時ファイル名の時刻 |
| `Out`(バッファ) | `WriteAll` で文字列に溜める。呼び出しの順序の記録も持つ。「n 回目の書き込みで失敗する」設定を持つ | 出力の中身、`cannot write to standard output`(`watch` の 65、`convert` の 67) |
| `Defs` | パスごとに、決めた定義や誤りの並びを返す。または、実ファイルを `Read-ConductorDefinitionFile` で読む | 定義の誤り(64)の経路 |
| `States`(メモリ) | ステートをメモリに持つ。読み込み・書き込み・削除の呼び出しを記録する。失敗の設定を持つ | 大きなジョブ数(1,000 個)の流れの測定、ステートの異常系(65) |
| `Converter` | 入力パスごとに、決めた文字列かエラーを返す | `convert` の 67 |
| `Proc` | ジョブIDごとに、台本の `JobExecution`(終了コード・シグナル終了・起動失敗・中断)を順に返す。呼び出しの記録を持つ。`Wait` は、待ち時間を記録するだけで実際には待たない。「`Wait` が中断を返す」設定を持つ | 分類・遷移・再開・リトライ(待ち時間の列)・中断の経路。実際の待ちをしない |
| `Signals` | `Start` の戻り値を設定する。「開始時に、すでに `$sig.Trigger(n)` を呼ぶ」設定を持つ(C# ヘルパーの `Interrupt` を、テストが作る) | 登録の失敗(70)、登録済み、開始時にすでに合図が立っている |
| プラットフォーム層の偽 | `Send-ConductorSignal`、`Read-ConductorExit` などを `Mock` する | 転送が 1 回だけ、先に終了した子へ転送しない、起動前の合図で起動しない |
| 待機ループの偽の子 | `WaitForExit` と転送の関数を、スクリプトブロックの引数で差し替える。`WaitForExit` の戻り値の列と、渡された値の列を記録する | 待ち時間の列 1、2、4、8、10、10、… |

- 実プロセスを使うテストは、`tests/jobs/` のテストジョブを使う(11 章)。
- 偽の実装は、スレッドを使わず、ロックを持たない。シグナルを使う偽は、`Trigger` を、偽の `WaitForExit` の中から呼ぶ形で作る(別のスレッドや本物のシグナルを使わない)。

---

## 8. 同期モデルの方針【規定】

| 項目 | 方針 |
|------|------|
| 基本 | **同期**。関数は、普通の関数。`Start-Job`、`Start-ThreadJob`、`ForEach-Object -Parallel`、`Start-Process`、`Register-ObjectEvent`、`Register-EngineEvent`、`[System.Threading.Tasks.Task]`、`[System.Threading.Thread]` を使わない |
| スレッド | メインのスクリプトのスレッドで、流れのすべてが動く。シグナルのコールバックだけが、ランタイムによって別のスレッドで動き、それは **C# ヘルパーの中で完結**する(PowerShell のスクリプトブロックを、別スレッドのデリゲートにしない。理由と決定は [07-process-and-signals.md](07-process-and-signals.md) の 7.1 節) |
| 並行の実行 | 流れの制御は、仕事を並行に走らせない。ジョブ、出力、ステートの書き込みは、すべて直列に呼ぶ |
| 子プロセス | `System.Diagnostics.Process`。待機は、`WaitForExit(int)` の繰り返し(07 章 5 章) |
| シグナル | C# ヘルパーの `PosixSignalRegistration`(UNIX 系)、`Console.CancelKeyPress`(Windows)。ハンドラは、フラグを立てる(`Cancel` と `Trigger`)だけ |
| 待ち方 | 待機ループ: 10 ms 以下の `WaitForExit(int)`。リトライの待機: 250 ms 以下の `Interrupt.Wait(int)` の繰り返しと、単調な時計(`Stopwatch.GetTimestamp`)。`Start-Sleep` を使わない |
| 共有する可変の値 | C# ヘルパーの `Interrupt`(`Interlocked` と `ManualResetEventSlim`)と、シグナル登録の印、予約された終了コードだけ。PowerShell の変数を、スレッド間で共有しない |
| キャンセル | 例外による中断、`CancellationToken` を使わない。中断は `Interrupt` で伝える。Ctrl+C による `PipelineStoppedException` は、7.7 節([07-process-and-signals.md](07-process-and-signals.md))の停止経路で扱う |
| 出力 | 標準出力・標準エラーへの書き込みは、`Report` の窓口を通し、書くたびに flush する |
| 排他制御 | 同じ定義への 2 つの `conductor` の同時の実行を、排他しない(両方が動き、ステートは、どちらも壊れた形では読めない。原子的な置換による) |
| 再帰 | 深い入れ子の入力(定義・ステート・変換)に対して、再帰で読まない(明示のスタック。03 章)。`command` の分割は、再帰を使わない(07 章 3 章) |
| スレッドの所有者・終了条件の表 | [07-process-and-signals.md](07-process-and-signals.md) の 10.3 節の表を、正本とする |

---

## 9. エラー処理の方針と、PowerShell の規律【規定】

### 9.1 エラー処理

| 項目 | 方針 |
|------|------|
| 基本 | 失敗は、戻り値(判別用の `Kind` または `Ok` を持つ `[pscustomobject]`)で返す。外部の入力(引数・環境・ファイル・OS の結果)に対して、例外を送出しない(`throw` しない) |
| 例外を許す場所 | (1) 実装の不具合(論理の矛盾。起こるはずのない状態)。`Invoke-Conductor` の `catch [System.Exception]` が、固定の 1 行を標準エラーに出して 70 を返す。(2) 標準出力への書き込みの失敗(`Report` が送出し、`watch`・`convert` の呼び出し元が捕まえる)。(3) .NET のメソッドが例外を送出する場合は、その場で、想定する型だけを狭く捕まえて、値にする(例: `Process.Start` の `Win32Exception`、ファイル入出力の `System.IO.IOException`・`System.UnauthorizedAccessException`、デコードの `System.Text.DecoderFallbackException`) |
| 捕まえる例外の範囲 | 想定する型だけを、狭く捕まえる(`catch [System.IO.IOException]`)。`catch { }`(型なし)と `catch [System.Exception]` を、`Invoke-Conductor` の 1 か所(とランチャー)以外に書かない。`try` の中身は、失敗しうる 1 つの呼び出しに絞る |
| 例外の内容 | 例外の `Message` と `ToString()` を、利用者への表示に使わない(入力の値、OS のメッセージが混ざらないよう、固定の文だけを表示する) |
| 終了 | `Invoke-Conductor` が `[int]` を返し、ランチャーが `exit` する。`exit` を、モジュールの中で呼ばない(モジュールの中の `exit` は、対話セッションごと終了する。確認していない。`pwsh` のプロセスの終了と同じ扱いになることは、PowerShell の既知の挙動)。例外: 停止経路のランチャー([07-process-and-signals.md](07-process-and-signals.md) の 7.7 節) |
| 非終了エラー | `$ErrorActionPreference = 'Stop'`(モジュールの先頭)で、非終了エラーも終了エラーとして `catch` に届く。コマンドレットに `-ErrorAction` を個別に付けない(例外: `Test-Path` のような、存在の確認は、使わず、`[System.IO.File]::Exists` を使う) |
| `$?`、`$LASTEXITCODE` | 使わない |
| 想定外の失敗の捕捉 | `Invoke-Conductor` の `try`/`catch [System.Exception]`(固定の 1 行 + 70)。捕まえられないもの(`StackOverflow`)と、捕まえて続行すべきでないもの(`OutOfMemoryException`)は、[07-process-and-signals.md](07-process-and-signals.md) の 11 章 |
| 停止の例外 | `PipelineStoppedException`(Ctrl+C)は、内部の誤りとして扱わない。再送出する([07-process-and-signals.md](07-process-and-signals.md) の 7.7 節・11 章) |

### 9.2 出力の規律(戻り値への出力の混入の防止)

PowerShell は、関数の中で、式の結果が変数に代入されずに残ると、それを関数の出力(戻り値の一部)にする。この版の規律:

- 値を返すメソッドや式を、結果を使わずに書くときは、`[void]` または `$null =` を付ける(例: `$null = $list.Add($x)` は不要(`List.Add` は戻り値なし)だが、`$set.Add($x)` は `bool` を返す。`StringBuilder.Append` は `StringBuilder` を返す)。
- `Write-Host`、`Write-Output`、`Write-Error`、`Write-Warning`、`Write-Verbose`、`Write-Debug`、`Write-Information`、`Out-Host`、`Out-String`、`Out-Null`(性能が遅い)を製品のコードで使わない(PSScriptAnalyzer の `PSAvoidUsingWriteHost` と、禁止コマンドの検査。[11-quality-and-build.md](11-quality-and-build.md))。画面への出力は、`Report` の `Write-ConductorOutput`/`Write-ConductorErrorLine` だけが、`[Console]::Out`/`[Console]::Error`(または、標準のストリーム)に書く。
- 関数は、戻り値を `return` で 1 つだけ明示する。配列を返すときは `return ,$array`。呼び出し側は、`@(...)` で包んで受ける(戻り値が 1 要素や 0 要素の配列のとき、単一の値・`$null` になるのを防ぐ)。
- 文字列の比較は、`-eq`、`-like`、`-match`、`switch` を使わない(既定で大文字小文字を区別しない)。区別が要る箇所は、`-ceq`、`[string]::Equals($a, $b, [System.StringComparison]::Ordinal)` を使う。`-match`/`-replace`/`-split`/`switch -Regex` を使わず、手書きの走査にする。`[regex]` を使うなら、`[System.Text.RegularExpressions.RegexOptions]` を明示する(`CultureInvariant`、`ECMAScript` か `None`)。
- `$null` の比較は、`$null -eq $x`(`$null` を左辺)。`if ($x)` で、`0`、空文字列、空配列を真偽として判定しない。
- 数値は、型を明示する(`[int]`、`[long]`、`[uint64]`、`[bigint]`)。

---

## 10. 版の唯一の出所【規定】

| 項目 | 規定 |
|------|------|
| 唯一の出所 | `src/Conductor/Conductor.psd1` の `ModuleVersion`(`'0.8.0'`)。版は、ここだけを書き換えて変える |
| 実行時の読み方 | `Conductor.psm1` の先頭で、`Import-PowerShellDataFile -LiteralPath (Join-Path $PSScriptRoot 'Conductor.psd1')` を 1 回だけ呼び、`.ModuleVersion` を `$script:ConductorVersion` に保つ。`Get-ConductorVersion`(`Cli`)が、これを返す。コードの中に、版の文字列のリテラルを書かない(二重の管理がない) |
| 選択肢の比較 | (a) `Import-PowerShellDataFile` でマニフェストを読む: モジュールの読み込み方(パス、名前)に依存しない。マニフェストはデータだけなので、安全に読める(確認: `Import-PowerShellDataFile` は、データファイルを、コードを実行せずに読む。出典: PowerShell の公式の文書の説明。この文書では個別の出典を確認していない。未確認)。**採用**。(b) `(Get-Module Conductor).Version`: モジュールが、`Conductor` という名前で読み込まれていることに依存し(別のパスから読み込んだ場合や、結合した `.psm1` では、名前や版の取り出しの挙動が違いうる)、`Get-Module` の探索のコストがある。不採用。(c) `$MyInvocation.MyCommand.Module.Version`、`$ExecutionContext.SessionState.Module.Version`: モジュールのスコープの中で取れるが、ドットソースされたスクリプトの中での値が確かめられていない。不採用(未確認) |
| 結合した配布物 | ビルドで `.psm1` を結合した場合も、`Conductor.psd1` は同じ場所に置かれ、同じ方法で読む |
| 表示 | `conductor --version` は `conductor <Get-ConductorVersion>`(`conductor 0.8.0`)。`--help` などに、版を書かない |
| 一致の検査 | (1) 単体テストが、`Test-ModuleManifest` の `Version` と、`Get-ConductorVersion` の値が等しいことを検査する。(2) 受け入れテストが、実際の `conductor --version` の出力が `conductor <マニフェストの ModuleVersion>` と等しいことを検査する。(3) リリースの CI が、タグ(`v0.8.0`)から `v` を除いた値と、`ModuleVersion` と、ビルドした配布物の `--version` の出力が、等しいことを検査する([11-quality-and-build.md](11-quality-and-build.md))。(4) ランチャー、テスト、ドキュメントの中の版の文字列(`#requires` の版は別。`PowerShellVersion` との一致を検査する)が、`ModuleVersion` と矛盾しないこと(ランチャーは版の文字列を持たない) |

---

## 11. 実行のシーケンス

### 11.1 入口

```text
bin/conductor.ps1:
    モジュールを読み込む
    code = Invoke-Conductor @args                 # 例外は 70(停止は finally で扱う)
    exit code                                     # 終了は、ここで 1 回だけ

Invoke-Conductor:
    内部の誤りの捕捉を、最初に設定する(try / catch [System.Exception])   # 固定の英語 1 行 + 70
    deps = New-ConductorSystemDeps
    return Invoke-ConductorMain -Arguments $args -Deps deps

Invoke-ConductorMain(arguments, deps):
    Test-ConductorDeps(deps)
    parsed = ConvertFrom-ConductorArguments(arguments)      # 04 章
      誤り      → 使い方の誤りを標準エラーに表示、return 68
      help      → 使い方全文を標準出力へ、return 0
      version   → `conductor 0.8.0` を標準出力へ、return 0     # 版は 10 章の唯一の出所から得る
      コマンド  → run / pipeline / watch / reset / validate / convert のどれかへ
```

### 11.2 `pipeline` の実行

前処理(`Invoke-ConductorPrepare`)の順序と、そこでの失敗の終了コードは、[05-execution-flow.md](05-execution-flow.md) と [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md) に固定されている。すべての呼び出しは直列で、並行に走らせない(8 章)。

```text
Cli: pipeline
 ├─ Invoke-ConductorPrepare
 │    ├─ sig = New-ConductorInterrupt                                  # C# ヘルパーを、ここで初めて読み込む
 │    ├─ r = & Deps.Signals.Start sig     → AlreadyRegistered は続行 / Failed は 70(定義の検査より前)
 │    ├─ & Deps.Defs.Load(...)                                          → 失敗 64(パイプライン名の照合を含む)
 │    ├─ & Deps.Proc.CheckCommand(全ジョブ)                              → 失敗 64(誤りのあるジョブすべてを 1 ジョブ 1 行で報告)
 │    ├─ run ID の決定(--run-id | 環境変数。Resolve-ConductorRunId)     → 失敗 68
 │    ├─ & Deps.States.Read(...)  (完了のものは「空」)                   → 失敗 65
 │    └─ Test-ConductorStateConsistency(...)                            → 食い違い 65
 └─ for job in 定義のジョブ(記述順):
      ├─ Get-ConductorDecision(...) → 実行 / スキップ / 拒否
      │    ├─ 拒否     → Report: 表示、return 66
      │    ├─ スキップ → Add-ConductorSkip → 変化があれば & Deps.States.Write(Durable)、続行(完了なら finish)
      │    └─ 実行:
      │         Invoke-ConductorExecJob    # 1 ジョブの try { ... } finally { 停止経路の後始末 }(07 章 7.7 節)
      │         ├─ ステートが空なら New-ConductorInitialState → & Deps.States.Write(Fast)
      │         ├─ リトライ込みの実行
      │         │    loop { r = & Deps.Proc.RunJob(job, sig, progress)
      │         │           dec = Get-ConductorRetryDecision(job, r, retriesDone, lastWait)
      │         │           リトライなら Report: 知らせ(常に標準エラー) → w = & Deps.Proc.Wait(secs, sig)
      │         │                    w が Interrupted なら「起動しなかった中断」の実行結果(Started=false, Interrupted=true, SignalNumber)を作って抜ける }
      │         ├─ Get-ConductorClassification(..., RunMode = 'Pipeline')
      │         ├─ Report: 表示(-v のときの結果行・遷移行。常時のエラー行)
      │         ├─ Add-ConductorOutcome → & Deps.States.Write(Durable)
      │         ├─ エラーまたは分類なし → return Get-ConductorExitCode(...)
      │         ├─ 完了 → finish(--remove-on-complete なら & Deps.States.Reset)、return 0
      │         └─ sig.IsRaised → Get-ConductorResumePoint → Add-ConductorInterrupt → & Deps.States.Write(Durable)、return 128 + 番号
 └─ ループを抜けても完了していない → "cannot determine whether the pipeline is complete"、return 65
```

- `run` は、前処理(ジョブIDの存在の検査を含む)のあと、指定した 1 ジョブについて同じ「判断 → 実行 → 分類 → 記録」を行う。分類と次の動きは `RunMode = 'Individual'`(`pipeline` は `'Pipeline'`。05 章)で求める。
- `watch`・`reset`・`validate`・`convert` は、シグナルを準備しない(C# ヘルパーを読み込まない)。それぞれ、ステートの読み込み(`ReadRaw`)・ステートの削除(`Reset`、`ResetAll`)・定義の検査(`Defs.Load` と `Proc.CheckCommand`)・変換(`Converter.Convert`)を呼ぶ。
- `pipeline` は、次のジョブを「次の動き」の戻り値で決めない。記述順に全ジョブを見て、起動前の判断で決める(スキップ・再開は、ステートが運ぶ)。
- 合図の確認は、`Cli` が `$sig.IsRaised` で行う(番号 0 を偽にしない。1 つの判断のなかで 2 回読まず、1 回読んだ値を使う。07 章 6 章)。`Invoke-ConductorJob` と `Wait-ConductorRetry` が、待機中の合図を扱う。
- 停止経路: Ctrl+C でメインのスクリプトが `PipelineStoppedException` で止まるとき、上のシーケンスの「記録 → 終了コード」は、`Invoke-ConductorExecJob` の `finally` が代わりに行う([07-process-and-signals.md](07-process-and-signals.md) の 7.7 節)。

### 11.3 責務の流れ(データ)

```text
定義ファイル ──Read-ConductorDefinitionFile──▶ Definition ─┐
ステートファイル ──Deps.States.Read──▶ PipelineState ───────┤
                                                             ▼
                       Get-ConductorDecision ──▶ 実行 / スキップ / 拒否
Invoke-ConductorJob ──▶ JobExecution ──Get-ConductorClassification──▶ (JobOutcome, NextStep)
                                          │
        Add-Conductor*(結果・スキップ・中断) ──▶ 新しい PipelineState ──Deps.States.Write──▶ ステートファイル
JobOutcome の報告する終了コード ──Get-ConductorExitCode──▶ conductor の終了コード
```

### 11.4 中断の伝わり方

- 中断の合図(C# ヘルパーの `ConductorNative.Interrupt`)は、「立った/立っていない」と「最初のシグナル番号」を持つ。シグナルのハンドラ(ランタイムが別のスレッドで動かす C# のコールバック)が `Trigger` で立て、待ち(`Invoke-ConductorJob`、`Wait-ConductorRetry`)と `Cli` の流れ(メインのスクリプトのスレッド)が読む。
- ハンドラは、`Cancel = true` と `Trigger` を呼ぶだけ。待つ側は、短い `WaitForExit(int)`(10 ms 以下の刻み)と `Interrupt.Wait(int)`(250 ms 以下の刻み。`Trigger` で即座に起こされる)の繰り返しの中で、次の周に合図を読む。詳細は [07-process-and-signals.md](07-process-and-signals.md) の 5〜7・10 章。

---

## 12. 未確認の事項

- この環境には `pwsh` も .NET SDK もなく、スクリプトの構文、`Add-Type`、マニフェストの検査、Pester の実行を、確かめていない。関数の骨格は、書き方の例であり、構文の検査を通していない。
- `param` ブロックのない関数と、配列のスプラットで、`-v`、`--`、`-?`、`-Verbose` が文字列のまま `$args` に入るか(T-5)。`pwsh -File` に、`--` を挟まず渡した引数が、リテラルのまま届くか。
- モジュールのスコープで、`Set-StrictMode` と `$ErrorActionPreference` が、呼び出し元の設定と独立に効くか。`Set-StrictMode -Version Latest` の下で、ハッシュテーブルの存在しないキーの参照が、例外になるか `$null` になるか。
- ドットソースされた `.ps1` の中の `$PSScriptRoot` の値。
- Pester 5 の `Mock` と、モジュールの非公開の関数の扱い(`-ModuleName`、`InModuleScope`)の細部。
- AST の走査(4.1 節)が、パイプラインのスクリプトブロックの中の呼び出し(`ForEach-Object { ... }`)も、関数の呼び出しとして拾えること(`FindAll` に、検索の対象にネストしたスクリプトブロックを含める第 2 引数 `$true` を渡す。公式の API の仕様のとおりと想定するが、確かめていない)。
- マニフェストの `PowerShellVersion` の最小の版(7.4 のサポート期間)、`CompatiblePSEditions` の扱い。
- 結合した `.psm1` の起動時間と、ドットソースの形との差(11 章の startup-bench で測る)。
- `bin/conductor` のシンボリックリンクの解決、`bin/conductor.cmd` の `%*` と `^`/`&`/`!` の扱い、`ERRORLEVEL` と `pwsh` の終了コードの一致、Windows での実機の動作。
- 確認できたこと(出典つき): `Add-Type` の挙動([07-process-and-signals.md](07-process-and-signals.md) の 7.3 節)、`about_Classes` の制約(5.3 節)、`about_Pwsh` の `-File` の終了コードと引数の受け渡し(5.5 節)、マニフェストのキー(5.1 節)、`ConsoleHost.cs` の Ctrl+C の処理(07 章の 7.5・7.6 節)。
