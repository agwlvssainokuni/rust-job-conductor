# 01. 概要

この文書は、`conductor`(PowerShell 版。挙動の版は 0.8.0)が何のためのツールか、どのような方針で作るか、何を作らないか、どのような制約の下で作るか、を述べる。詳細な規定は、02 章以降にある。用語は [00-README.md](00-README.md) の用語集に従う。

## 1. 目的

複数のコマンド(シェルスクリプトや実行ファイル)を決まった順に実行する一連の作業(バッチ処理、定期ジョブ、データの取り込みの手順など)を、次の 3 点で支える。

| 目的 | 内容 |
|------|------|
| 宣言的に書く | 実行するコマンドの並びと、終了コードごとの扱い(成功・警告・エラー、次に進むジョブ、リトライ)を、1 つの定義ファイル(`.psd1` が主、JSON も可)に書く |
| 途中経過を残す | どのジョブがどう終わったか、次にどこから再開するかを、ステートファイル(JSON)に保存する |
| 失敗から再開する | 失敗したジョブで止まり、原因を直したあとの再実行で、終わったジョブは飛ばして、失敗したところから続ける |

`conductor` は、PowerShell 7(`pwsh`)のモジュール `Conductor` と、それを起動するランチャー(`bin/conductor`、`bin/conductor.cmd`、`bin/conductor.ps1`)として作る。利用者の使い方は、PowerShell のコマンドレット(`Verb-Noun`、`-Parameter`、パイプライン)の形ではなく、コマンドラインツールの文法のまま(`conductor pipeline -v --config conductor.psd1 nightly`)である。利用者は、`pwsh`(7.6 LTS 以上。実装時に確認)を入れておく。PowerShell 自体は同梱しない([11](11-quality-and-build.md) 6 章)。

## 2. ユースケース

| ユースケース | コマンド | 内容 |
|--------------|----------|------|
| パイプラインを順に実行する | `conductor pipeline <名前>` | 定義の全ジョブを記述順に実行する。失敗したジョブで止まり、再度実行すると、そのジョブから再開する |
| 1 つのジョブだけ実行する | `conductor run <ジョブID>` | 外部のスケジューラが、ジョブを 1 つずつ呼ぶ。順序が守られているかをステートで確かめ、順序外なら拒否する |
| 状態を確かめる | `conductor watch` | ステートを表にして標準出力に出す(定義は検証しない) |
| ステートを捨てる | `conductor reset [--all]` | ステートを初期化する。壊れたステートからの回復にも使う |
| 定義を検査する | `conductor validate [<名前>]` | 定義とコマンドの書き方を、ジョブを起動せずに検査する(CI で定義の誤りを早く見つける) |
| 書式を変換する | `conductor convert <ファイル> --to psd1\|json` | 定義(など)を `.psd1` と JSON のあいだで変換する(人が書く定義にコメントが要るときは `.psd1` を使う) |
| 同じ定義を並行して使う | `--run-id <ID>` | run ID でステートを分け、別の実行単位として扱う(CI の実行番号など) |
| 警告を許す | `warning_exit_codes`、`transitions` | 一部の終了コードを「警告」として扱い、続行したり、別のジョブへ飛んだりする |
| 一時的な失敗を再試行する | `retry` | 決めた終了コードで終わったときに、待ってから実行し直す(間隔の倍率と上限を指定できる) |

## 3. 設計方針

| 方針 | 内容 |
|------|------|
| 同じ挙動 | 版 0.8.0 と、利用者から見て同じ挙動にする。サブコマンド、オプション、終了コード、メッセージの文言、`watch` の表、実行の意味論は同じ |
| データとして読む | `.psd1` は PowerShell のコードの形をしているが、**コードを実行しない**。構文木を作り、許可した構文だけを自前で値に変換する。コマンド・変数・式・スクリプトブロックを含む定義は拒否する(03 章 4 章) |
| 失敗を値で返す | 外部の入力(引数、環境、ファイル、シグナル、ジョブの出力)に対して異常終了しない。誤りは、種類・位置・固定の理由・終了コードを持つ `[pscustomobject]` の値で返す(`class` と `enum` を定義しない。10 章 8 章)。想定外の内部失敗だけが例外で、70 |
| 少なくとも 1 回 | ジョブは、少なくとも 1 回は実行される。結果の記録を書けなければ、ジョブの終了コードより 65 を優先する。記録の前に落ちると、再実行されうる |
| シェルを介さない | `command` は、決まった規則で分割した引数を、`System.Diagnostics.Process`(`UseShellExecute = $false`)に直接渡す。展開・パイプ・リダイレクトは解釈しない。`Start-Process` と `&` 演算子を使わない。メタ文字は、そのまま引数になる(Windows の `.cmd` / `.bat` だけは、07 章 4.8 節の安全策を通す) |
| 入力の値を表示に出さない | エラーの文に、入力の値や、パーサー・.NET のエラー文を含めない(入力に由来するのは、キー名だけ。制御文字と孤立サロゲートをエスケープして切り詰める) |
| 純粋な判断とふるまいの分離 | 分類・遷移・再開・リトライの判断は、入出力に触れない純粋な関数(`Private/Flow/`)にして、表駆動のテストで確かめる。ファイル・プロセス・シグナル・時刻・標準出力は、`Deps`(スクリプトブロックを持つハッシュテーブル)の向こうに置く |
| 原子的なステート | ステートは、一時ファイルに書いて、同期してから、`[System.IO.File]::Move($src, $dst, $true)` で置き換える。読み手は、直前の完全な内容か、新しい完全な内容のどちらかだけを見る |
| 同期モデル | 同期(ジョブ、ランナー、イベントを使わない)。ジョブ、出力、ステートの書き込みは、すべて直列。シグナルのコールバックは別のスレッドで動くので、C# ヘルパーの中で旗を立てるだけにして、待つ側が短い刻みで旗を見る。PowerShell のスクリプトブロックを別スレッドのコールバックにしない(実行空間が無く失敗する) |
| 出力を完了させてから終わる | 標準出力・標準エラーは、自前の `StreamWriter` に書き、書くたびに `Flush()` する。画面への出力は専用の関数だけが行い、関数の戻り値には何も混ぜない。終了は入口スクリプトの `exit` の 1 か所(と、Ctrl+C の停止経路の `Environment.Exit`) |
| 依存は最小 | 実行時の依存は無し(標準の PowerShell と BCL だけ)。引数の解析、JSON の読み取りと書き出し、`.psd1` の読み取りと書き出しは自前 |
| 文化・文字単位を明示する | `string` は UTF-16。文化に依存する演算子・API(`-eq`、`-match`、`switch`、`Trim()`、`ToString()` の既定)を使わず、序数比較(`-ceq`、`StringComparison.Ordinal`)と `InvariantCulture` を使う。文字を数える単位(コードユニット、コードポイント、UTF-8 のバイト数)は、場面ごとに規定する |
| 確かめられる仕様 | すべての規定は、配布物のランチャーを外から実行する受け入れテスト(11 章。AC-001〜585)で確かめられる形で書く |

## 4. 非目標

次のことは、この版では行わない。

| 非目標 | 補足 |
|--------|------|
| ジョブの並列実行 | ジョブは、記述順に 1 つずつ実行する。分岐は、遷移による前方へのスキップだけ(後戻り・ループはない) |
| 常駐とスケジューリング | デーモンも、時刻による起動もない。起動は、利用者か外部のスケジューラが行う |
| 排他制御 | ファイルロックや PID ファイルを使わない。同じ定義(同じ run ID)の同時実行は、避けること(run ID で分ければ並行できる) |
| ジョブの出力の加工 | ジョブの標準入出力・標準エラーは、そのまま流れる(`Redirect*` を `$false` のまま引き継ぐ)。ログの収集・保存はしない |
| シェルの機能 | 変数の展開、パイプ、リダイレクト、ワイルドカードの展開は、`conductor` は解釈しない(必要なら、ジョブがシェルを起動する) |
| 環境変数の展開 | 定義の値に環境変数を埋め込む機能はない。環境変数を読むのは、run ID、ロケール、色の判定と、実行ファイルの探索(`PATH`)・`cmd.exe` の場所(`ComSpec`)だけ |
| PowerShell の式 | `.psd1` に、変数・式・コマンド・スクリプトブロックを書く機能はない(データだけ)。`Import-PowerShellDataFile` の挙動(コードを実行しない範囲)とも、位置の扱いが違う(03 章 4.11 節) |
| ほかの書式 | 定義は `.psd1` と JSON だけ(拡張子が `.psd1`・`.json` 以外は拒否する) |
| PowerShell のコマンドレットとしての公開 | 公開する関数は `Invoke-Conductor` の 1 つで、引数は文字列の配列(`-ArgumentList`)。`-Verbose` などの共通パラメータや、パイプライン入力を持たない。PowerShell Gallery への公開は、人が手で行う(11 章 6.1 節) |
| リトライの回数の保存 | リトライの回数と待ち時間は、ステートに保存しない(再開は、最初の実行から数え直す) |
| 画面の対話 | 対話的な入力も、進捗の動的な表示もない(標準入力を読まない。ジョブに引き継ぐ。`Write-Progress` を使わない) |
| ジョブの強制終了 | 中断のとき、子プロセスにシグナルを 1 回転送するだけ(UNIX 系)。止まらないジョブを強制終了しない(`Process.Kill`・`Stop-Process` を使わない) |
| ジョブを新しいプロセスグループに置くこと | しない(`conductor` と同じグループ。`System.Diagnostics.Process` に、その手段がない。07 章 4.7 節) |
| ネットワーク | 外向きの通信をしない(`Invoke-WebRequest`・`System.Net` を使わない)。リモートの実行や、状態の共有もない |
| Windows PowerShell 5.1 | 対象外(`powershell.exe`。`PosixSignalRegistration` と `ArgumentList` の前提が無い。マニフェストの `PowerShellVersion` で読み込みを拒否する) |
| Constrained Language Mode | 対象外(`Add-Type` と .NET の型の呼び出しが制限される。`run`・`pipeline` は終了コード 70 になる。07 章 7.3 節) |
| 署名・公証 | Authenticode 署名と macOS の署名・公証は行わない。実行ポリシーと `Zone.Identifier` の注意は、利用者向けに書く(11 章 6.5 節) |

## 5. 制約

| 項目 | 内容 |
|------|------|
| PowerShell の版 | 対象は PowerShell 7.x。この版の決定は **7.6 LTS**(作成日 2026-11-20 の時点でサポート中の LTS は 7.6 だけ。確認: 11 章 1 章)。実装時に、サポートの状況と次の LTS を確認し、最小と最新の版を分ける。版の出所は `build/Versions.psd1`(`MinPowerShell`・`MaxPowerShell`)で、`Conductor.psd1` の `PowerShellVersion`、ソースの `#requires -Version`、CI の版との一致を検査する(11 章 AC-477) |
| 言語の設定 | `Set-StrictMode -Version Latest`、`$ErrorActionPreference = 'Stop'` をモジュールの先頭に置く。`class`・`enum`、`Write-Host`・`Write-Output`・`Write-Error`、`Invoke-Expression`、`trap`、型を指定しない `catch`(入口以外)、`Start-Job`・`Start-Process`・`Register-ObjectEvent` を製品のコードで使わない(PSScriptAnalyzer と構文木の検査で強制する) |
| 対象 OS | Linux、macOS、Windows の 3 つ。OS 差は、プラットフォーム層(`Get-ConductorPlatformKind`、`Get-ConductorStartInfo`、`Send-ConductorSignal`、`Read-ConductorExit`、`Sync-ConductorDirectory`)と C# ヘルパー(`NativeHelper.ps1`)に隔離する(OS の判定は `Get-ConductorPlatformKind` だけ) |
| 配布 | `Conductor/`(モジュール)と `bin/` を zip(Windows 向け)と tar.gz(UNIX 系向け。実行権限を保つ)にし、`SHA256SUMS` を付ける。タグ `v*` で、ドラフトのリリースに添付する |
| 依存の方針 | 実行時の依存は無し。開発時の依存は、Pester 5 と PSScriptAnalyzer(版とハッシュを `build/Dependencies.psd1` に固定。11 章 8 章) |
| ほかの依存 | ネイティブの部品を必須としない(P/Invoke は `libc` の `kill`・`open`・`fsync`・`close` だけで、失敗しても本体は動く)。ネットワークのライブラリ、`ConvertFrom-Json`・`ConvertTo-Json`、`Import-PowerShellDataFile`(定義・変換の読み取りには)を使わない |
| 性能の注意 | PowerShell は、`pwsh` の起動が遅く(数百 ms)、ループがコンパイルされた言語より遅い。`Add-Type` のコンパイルの費用(数百 ms の見込み)は、`run`・`pipeline` の最初のジョブの起動までに遅延させる。基準値(起動 p95 など)は、すべて仮で、測っていない(11 章 2.1 節)。1 MiB の入力の読み取りは、1 文字ずつの走査で数秒〜数十秒かかりうる |
| 入力の大きさ | 定義・ステート・変換の入力は 1 MiB まで。`.psd1`・JSON の展開後の要素数は 100,000、文字の大きさは 16 MiB、入れ子の深さは 32 段まで |
| 文字コード | 定義・ステート・変換の入力は UTF-8(先頭の BOM は、定義と変換では受理し、ステートでは拒否する)。引数と環境変数は、.NET の `string`(UTF-16)で受ける(不正な UTF-8 の扱いは 04 章 4.4 節。一部は未確認) |
| 言語 | メッセージは、日本語と英語(環境変数 `LC_ALL`、`LC_MESSAGES`、`LANG` で決める)。理由の文(定義・ステート・変換)は、常に日本語 |
| ライセンス | Apache License 2.0。ソースの先頭に著作権表示を置く(11 章 2.4 節) |
| 検証の限界 | 作成時には `pwsh` も .NET SDK もなく、コードのビルドも実行もしていない。確認できた点と未確認の点は、各章と 12 章に書いた |

## 6. 利用者像

| 利用者 | 求めるもの |
|--------|------------|
| 運用担当(バッチや定期ジョブの運用者) | 失敗したところから再開できること。状態を `watch` で確かめられること。壊れたステートを `reset` で回復できること。ダウンロードしたスクリプトが実行ポリシーでブロックされたときの回避手順(`Unblock-File` など。11 章 6.5 節) |
| 開発者(ジョブと定義を書く人) | 定義の誤りが、種類と行番号つきで分かること。`validate` を CI に入れられること。リトライと警告の扱いを宣言で書けること。定義にコメントが書けること(`.psd1`) |
| CI の管理者 | 実行番号などで run ID を分けて並行して使えること。終了コードで結果を判断できること。3 OS で動くこと |
| この仕様書の読者(再実装する人) | 02〜11 章を読んで、同じ挙動の実装を作り、受け入れテストで確かめられること。`pwsh -File` の引数の分割、パイプラインへの出力の混入、`-eq` の大文字小文字、配列の単一化、`[double]` への昇格、`ConvertFrom-Json` の限界、Ctrl+C と `PipelineStoppedException`、`Add-Type` の再実行と起動の費用などの落とし穴と、最初の試験が分かること(12 章) |

## 7. 概念の概観

### 7.1 定義ファイル

1 つのパイプラインと、ジョブの並びを書く。既定の名前は `conductor.psd1`。

```powershell
# 夜間バッチ
@{
    format_version = 1
    pipeline       = @{
        name        = 'nightly'
        description = '夜間バッチ'
    }
    job            = @(
        @{
            id                 = 'extract'
            command            = 'extract --all --since yesterday'
            warning_exit_codes = @(3, 4)
            transitions        = @(@{ on = 'warning'; next = 'notify' })
        },
        @{
            id      = 'load'
            command = 'load'
            retry   = @{ exit_codes = @(75); max = 3; interval = '30s' }
        },
        @{ id = 'notify'; command = 'notify --channel ops' }
    )
}
```

同じ定義は、JSON でも書ける(`conductor convert` で相互に変換できる)。

```json
{
  "format_version": 1,
  "pipeline": { "name": "nightly", "description": "夜間バッチ" },
  "job": [
    { "id": "extract", "command": "extract --all --since yesterday", "warning_exit_codes": [3, 4],
      "transitions": [{ "on": "warning", "next": "notify" }] },
    { "id": "load", "command": "load", "retry": { "exit_codes": [75], "max": 3, "interval": "30s" } },
    { "id": "notify", "command": "notify --channel ops" }
  ]
}
```

- 書式は拡張子で決まる(`.psd1`・`.json`。ASCII の大文字小文字を区別しない)。`.psd1` は、コードとして実行されず、許可した構文(ハッシュテーブル、配列、文字列、数、`$true`・`$false`・`$null`、コメント)だけが読まれる。
- ジョブは記述順に実行する。`transitions` で、成功・警告のときの次のジョブ(後ろのジョブか `@end`)を指定できる。飛ばされたジョブはスキップになる。
- 読み込みは、見つけた誤りをまとめて返す(種類、行、理由)。詳細は 03 章。

### 7.2 実行と分類

ジョブを起動し、終了コードで分類する。

| 結果 | 分類 | 終了コード |
|------|------|------------|
| 成功コード(省略時は 0)で終了 | 成功 | 0 |
| 警告コードで終了 | 警告 | 0 |
| それ以外の終了コード | エラー | ジョブの終了コード(0〜255 に合わせる) |
| シグナルで終了(UNIX 系) | エラー | 128 + 番号 |
| 起動できない | エラー | 127 |
| 起動前に中断された | 分類なし | 128 + 番号 |

UNIX 系の「シグナルで終了」は、`Process.ExitCode` が 129〜159 で、そのジョブの成功・警告のコードに含まれないときだけの推定である(07 章 4.4 節)。`pipeline` は、エラーで止まり、そのジョブの終了コードで終わる。リトライの対象なら、待って実行し直す。詳細は 05 章。

### 7.3 ステートと再開

実行の途中経過を、定義ファイルと同じ場所の `<定義ファイル名>.state` に JSON で保存する(run ID があれば `<定義ファイル名>.<run ID>.state`)。再実行では、終わったジョブを飛ばし、再開位置から続ける。`run` は、順序外のジョブを拒否する(66)。スキップ対象・`@end` より後ろ・実行済みのジョブは、拒否ではなく、スキップとして 0 で終わる。全ジョブが終わると完了になり、完了のステートは、次の実行では空とみなす。詳細は 05・06 章。

### 7.4 中断

UNIX 系では SIGHUP・SIGINT・SIGQUIT・SIGTERM を `PosixSignalRegistration` で、Windows では Ctrl+C を `Console.CancelKeyPress` で受ける。どちらも C# ヘルパーの中で完結し、コールバックは別のスレッドで動いて、中断の合図(`Interrupt`)に最初の番号を残すだけである。UNIX 系では、実行中のジョブにシグナルを 1 回転送し(`libc` の `kill`)、再開位置を記録して、128 + 番号で終わる。Windows は、OS が Ctrl+C を同じコンソールのジョブにも配るので、転送しない。リトライの待機は、合図から 1 秒以内に切り上げる。Ctrl+C では `pwsh` がスクリプトを `PipelineStoppedException` で止めうるので、後始末は `finally` に置き、終了コードは入口スクリプトの `finally` が決める。詳細は 07 章。

### 7.5 出力

- `conductor` 自身は、既定では何も出力しない(ジョブの出力がそのまま流れる)。エラーのとき、または `-v` のときだけ、標準エラーに書く。
- `watch`、`--help`、`--version`、`convert` は、結果を標準出力に書く。標準出力に書けないとき(容量不足など)は、`watch` が 65、`convert` が 67 で終わる(`--help`・`--version` は、失敗を無視して 0)。UNIX 系で、読み手が先に閉じたパイプへの書き込みは、.NET の標準のストリームに捨てられ、65・67 でなく 0 で終わりうる(既知の差。08 章 1.3.1)。
- メッセージは、環境変数で日本語か英語を選ぶ。`watch` の表は、端末なら状態語に色が付く(自前の ANSI。`$PSStyle` を使わない)。詳細は 08 章。

### 7.6 内部の構成

```text
bin/conductor(sh)/ bin/conductor.cmd ─▶ pwsh -File bin/conductor.ps1 ─▶ Invoke-Conductor -ArgumentList
                                                                               │
Public ─▶ Cli ─▶ Report / Runner / State / Convert / RunId / Flow / Definition / Json / Text
                  (Runner と State は C# ヘルパーを、Cli が仲立ちして使う)
```

`Cli` が引数を解き、各部を組み立てて終了コードを決める(`Invoke-ConductorMain`)。`Flow` は純粋な判断、`State` はステートの入出力、`Runner` は子プロセスとシグナル(OS 差はプラットフォーム層に隔離)、`Report` は文の組み立てと書き込みの窓口、`Definition` は定義の読み込みと検査(`.psd1` の読み取りを含む)、`Json` は自前の JSON の読み取りと書き出し(`Definition`・`State`・`Convert` が共用する)、`Convert` は書式の変換、`Text` は文字列の部品(エスケープなど)である。入口スクリプトは薄く、モジュールの読み込みと、内部の誤りの捕捉と、`exit` を受け持つ。依存の向きは、テスト(構文木の走査)で強制する(02 章 4 章)。

## 8. この仕様書の使い方

実装の順序は [12-rebuild-guide.md](12-rebuild-guide.md) にある。用語と範囲は [00-README.md](00-README.md)。
