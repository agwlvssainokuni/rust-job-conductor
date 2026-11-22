# 00. README(PowerShell 版 仕様書の案内)

> この仕様書は v0.8.0 時点の挙動で凍結されている(v0.9.0 以降の環境変数の注入と `.env`、`graph`、`--dry-run`、`init` を含まない)。最新は [rust](../rust/00-README.md) と [language-agnostic](../language-agnostic/00-README.md)。

この仕様書は、コマンドラインツール `conductor` を、PowerShell 7(`pwsh`)でゼロから実装し直すための仕様である。`conductor` は、1 つの定義ファイルに書かれたジョブ(外部コマンド)の並びを、終了コードで分類しながら順に実行し、途中経過をステートファイルに残して、失敗したところから再開できるようにする、コマンドラインツールである。

この文書は、仕様書全体の入口である。範囲と版、目次、読む順序、規定の強さの語、用語集、この版が採った書式・実行環境の選択をまとめる。

## 1. 仕様書の範囲と版

| 項目 | 内容 |
|------|------|
| 対象 | `conductor` の利用者から見える挙動を、PowerShell で再実装するための仕様 |
| 版 | `conductor 0.8.0` と同じ挙動。`conductor --version` は `conductor 0.8.0` を出す。版の唯一の出所は `src/Conductor/Conductor.psd1` の `ModuleVersion` で、コードは、モジュールの読み込み時に `Import-PowerShellDataFile` で 1 回だけ読む(版の文字列のリテラルをコードに書かない。読めなければ、定数と一致の検査に切り替える。[02](02-architecture.md) 10 章、[04](04-cli.md) 8 章、[11](11-quality-and-build.md) AC-476) |
| 実装言語・実行環境 | PowerShell 7.x(`pwsh`。Linux、macOS、Windows)。**この版の決定は PowerShell 7.6 LTS**(実装時に確認)。作成日 2026-11-20 の時点で、サポート中の LTS は 7.6 だけである(確認: [11](11-quality-and-build.md) 1 章。出典: 「PowerShell Support Lifecycle」。7.6 のサポート終了は 2028-11-14。7.4 と 7.5 は 2026-11-10 に終了。7.7 はプレビュー)。最小と最新の版は `build/Versions.psd1` の `MinPowerShell`・`MaxPowerShell`(例は、どちらも `'7.6'`)に書き、マニフェストの `PowerShellVersion` と、ソースの `#requires -Version` は、`MinPowerShell` と同じ値にする([11](11-quality-and-build.md) 2.5 節)。次の LTS が出たら、最小と最新を分けて、テストを両方で通す。Windows PowerShell 5.1(`powershell.exe`)は対象外 |
| 実行時の依存 | **無し**(標準の PowerShell と .NET の BCL だけ)。C# の小さなヘルパー(`Add-Type`)は、モジュールがソースを持つ([07](07-process-and-signals.md) 7 章)。開発時の依存は、Pester 5 と PSScriptAnalyzer(版は実装時に確認して、`build/Dependencies.psd1` に正確な版とハッシュで固定する。[11](11-quality-and-build.md) 8 章) |
| 対象 OS | Linux、macOS、Windows の 3 つ |
| 同じにするもの | サブコマンド、オプション、終了コード、実行の意味論(分類・分岐・再開・リトライ・run ID)、メッセージの文言(日本語・英語)、`watch` の表 |
| 含まないもの | 実装そのもの(コードは規定しない。骨格のコードは、関数の署名と値の形の例である)。各章が「実装に任せてよい」と書いた点 |

この仕様書だけで、同じ受理・拒否・出力・終了コードを返す実装を作れることを目標とする。

**確認の表記**: この仕様書を書いた環境には `pwsh` も .NET SDK もなく、PowerShell のコードを実際に実行していない。API と挙動は、公式ドキュメント(learn.microsoft.com の PowerShell 7 の `about_*` と各コマンドレット、.NET の API リファレンス)と、PowerShell・.NET の公開ソース、Pester・PSScriptAnalyzer の公式ドキュメントの読み取りで確かめた範囲だけを「確認(出典: …)」と書き、確かめられなかった点は「未確認」と明記した。未確認の一覧と、実装時に最初に確かめる試験は、[12](12-rebuild-guide.md) の 3・4 章にまとめた。

## 2. 目次

| ファイル | 内容(1 行要約) |
|----------|-----------------|
| [00-README.md](00-README.md) | この文書。範囲、目次、読む順序、規定の強さの語、用語集、書式・実行環境の選択 |
| [01-overview.md](01-overview.md) | 目的、ユースケース、設計方針、非目標、制約(PowerShell の版、対応 OS、依存、性能の注意)、利用者像、概念の概観 |
| [02-architecture.md](02-architecture.md) | ソース構成(モジュールとディレクトリ)、関数の命名と一覧、依存の向きとその強制、モジュールの構成(マニフェスト、`.psm1`)、ランチャー、`Deps`(差し替え口)、同期モデル、エラー処理と出力の規律、版の出所、実行のシーケンス |
| [03-definition-format.md](03-definition-format.md) | 定義ファイルの書式(`.psd1` 主・JSON 副)、読み取りの段階、`.psd1` の許す構文(構文木のノードの型で定める)、JSON の自前の読み取り、木の型と上限、検査、エラーの種類(31 種)と位置の出し方 |
| [04-cli.md](04-cli.md) | コマンドライン(6 つのサブコマンド、オプション)、ランチャー 3 つと `Invoke-Conductor` の署名、生の引数の取り出し、自前の引数解析、非 UTF-8 の引数、run ID、環境変数 |
| [05-execution-flow.md](05-execution-flow.md) | 実行の意味論(分類、遷移、起動前の判断、リトライと `[bigint]` による飽和した待ち時間、再開、完了、`Flow` の純粋関数)、整数・比較・配列の PowerShell の落とし穴 |
| [06-state-file.md](06-state-file.md) | ステートファイル(論理項目、JSON の形式、原子的な書き込み、読み込みの 14 段階の検査、理由文 23 件、整合性、`reset`) |
| [07-process-and-signals.md](07-process-and-signals.md) | `command` の分割、`System.Diagnostics.Process` による子プロセスの起動と待ち、中断の合図、C# ヘルパーと `Add-Type`、シグナルの受信と転送、Ctrl+C と `PipelineStoppedException`、リトライの待機、OS 別の差、Windows の `.cmd` の起動、想定外の内部の誤り |
| [08-output-and-i18n.md](08-output-and-i18n.md) | 出力先、標準ストリームの書き込み・flush・閉じたパイプ、`-v`、メッセージカタログ(日本語・英語)、`watch` の表、ロケール、エスケープ、`--help` の全文 |
| [09-converter.md](09-converter.md) | `convert`(`.psd1` と JSON の相互変換)の読み取りの差分、書き出しの形式、浮動小数の書式、往復の保証、エラー(6 種、理由 17 件) |
| [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md) | 終了コードの全件と決定順序、分類、範囲合わせ、`pwsh -File` の終了コード、想定外の内部失敗(70)、エラー分類の一覧、PowerShell でのエラーの設計(値で返す) |
| [11-quality-and-build.md](11-quality-and-build.md) | 非機能要件、テスト構成、受け入れテスト一覧(AC-001〜585)、CI(GitHub Actions)、`build/` の検査スクリプトの全文、リリース、デモ、依存、リポジトリのレイアウト |
| [12-rebuild-guide.md](12-rebuild-guide.md) | ゼロから実装する順序、PowerShell 特有の落とし穴、未確認事項の一覧、最初に確かめる小さな試験 |

## 3. 読む順序

| 目的 | 読む順序 |
|------|----------|
| 全体を理解する | 00 → 01 → 02 → 05 → 03 → 06 → 04 → 07 → 08 → 10 → 09 → 11 |
| 実装を始める | 00 → 01 → [12](12-rebuild-guide.md)(実装の順序と最初の試験)→ 12 が指す各章 |
| 定義の読み込みだけを作る | 03 → 09(読み取りを共有)→ 10(エラー) |
| 実行の流れだけを作る | 05 → 06 → 07 → 10 |
| 出力の文言を確かめる | 08 → 10 |
| 受け入れる | 11(各 AC は、該当章の字句が正本) |

各章の終わりに、その章の「実装に任せてよい点」と「未確認」がある。受け入れテスト(11 章)の期待は、定義・ステート・変換・終了コードの各章(03、06、09、10)の字句と食い違うときは、それらの章を正とする。章どうしの食い違いは、03(定義)、04(引数の受け方、ランチャー)、05(実行フロー)、06(ステート)、07(プロセスとシグナル)、09(変換)の定めを正とする。

## 4. 規定の強さの語

章によって、記号や語が少し違うが、意味は次の 3 段階に対応する。

| 強さ | 意味 | 章での表記 |
|------|------|------------|
| 必須 | 利用者から見える結果(受理・拒否、出力の文字列、終了コード、作られるファイルの内容)であり、同じにしなければならない。受け入れテストで確かめる | 「必須」「【規定】」「〜しなければならない」「強制」(11 章の非機能要件) |
| 目安 | 推奨する値や分け方。利用者から見える挙動を損なわない範囲で、別の値や分け方を採ってよい。基準を超えたときは警告として記録する | 「目安」「【目安】」「推奨」「仮」(暫定として置いた値。測っていない) |
| 任意 | 利用者から見える結果が変わらない範囲で、実装が自由に選んでよい | 「任意」「実装に任せてよい」「裁量」 |

- 【PowerShell の設計】(02、07 章ほか)は、PowerShell で実装するために、この仕様が決めたことで、他の決め方もありうる(利用者から見える結果が同じなら変えてよい。目安と同じ扱い)。ただし、各章が「規定」と明記したもの(シェルを介さない、`Add-Type` と P/Invoke の許可箇所、OS の判定は `Get-ConductorPlatformKind` だけ、出力は `Write-ConductorStream` だけ、など)は必須である。
- 骨格のコード(関数の署名・値の形)は、名前や細部が他の章の定めと食い違うときは、03・04・05・06・07・09 の定めを正とし、骨格を読み替える。
- 「確認(出典: …)」は、公式ドキュメントまたは公開ソースで確かめた点。「実装時に確認」は、版や道具の選定など、実装の時点で決める値で、この文書は決めない。「未確認」は、作成時に確かめられなかった点で、断定していない。実装時に確かめる([12](12-rebuild-guide.md) 4 章)。

## 5. 用語集

02〜11 章で使っている語を、次のように統一する。

| 語 | 意味 |
|----|------|
| 定義ファイル | パイプラインを書いたファイル。`.psd1`(PowerShell データファイル)か JSON(`.json`)。既定の名前は `conductor.psd1` |
| 定義 | 定義ファイルを検証して読み込んだ結果(`format_version`、`pipeline`、`job` の並び)。誤りは種類・位置・固定の理由の値で返す |
| パイプライン | 名前をもち、ジョブの並びからなる 1 つの流れ。1 つの定義ファイルに 1 つ |
| ジョブ | 実行する外部コマンド 1 つと、その終了コードの扱い(成功・警告のコード、遷移、リトライ)。ID をもつ |
| ジョブID | ジョブの識別子(`id`)。定義の中で一意。空白だけでなく、`@` で始まらない |
| `command` | ジョブが実行するコマンド。文字列(分割して実行する)か、文字列の配列(そのまま渡す)。シェルは介さない |
| 成功コード・警告コード | ジョブの終了コードのうち、成功(`success_exit_codes`。省略時は `@(0)`)・警告(`warning_exit_codes`)として扱う値 |
| 分類 | ジョブの結果を、成功・警告・エラー・分類なし(起動前の中断)に分けること(`Get-ConductorOutcome`) |
| 遷移(`transitions`) | 成功・警告のとき、次に実行するジョブを指定すること。`@end` は、残りをスキップして終える |
| スキップ | 遷移などで、実行されないジョブ。理由は `byTransition`・`byEnd` |
| 拒否 | 順序外などで、`run` が実行を断ること(終了コード 66)。スキップ対象・`@end` より後ろ・実行済みのジョブは、拒否ではなくスキップ(終了コード 0) |
| リトライ(`retry`) | 決めた終了コードで終わったジョブを、待ってから実行し直すこと。最大回数・間隔・倍率・上限を指定できる。待ち時間は 64 ビット符号なしの秒で、`[bigint]` で計算して飽和させる(折り返さない) |
| run ID | 実行単位を区別する文字列(`A-Za-z0-9._-`、1〜64 文字、先頭は `.` でない)。ステートファイルを分ける |
| ステートファイル | 実行の途中経過を JSON で保存するファイル。名前は `<定義ファイル名>.state`(run ID があれば `<定義ファイル名>.<run ID>.state`) |
| 再開位置(`ResumeJobId`) | 次に実行する、または再開するジョブのID |
| 完了 | 全ジョブが終わった状態(`Completed = $true`)。完了のステートは、`Read-ConductorState` では「空」とみなす |
| 空のステート | ステートが無いのと同じ扱い(ファイルがない、長さ 0、完了) |
| 耐久性(`Durability`) | ステートの書き込みで、内容の同期(`FileStream.Flush($true)`)まで行うか(`durable`)、省くか(`fast`) |
| 実行方式 | `run`(1 ジョブ。`individual`)と `pipeline`(全ジョブ。`pipeline`) |
| at-least-once | ジョブは、少なくとも 1 回は実行される。結果の記録の前に落ちると、再実行されうる |
| 中断の合図(`Interrupt`) | シグナルを受けたことと、最初の番号を、シグナルのコールバックと待つ側のあいだで共有する値。C# ヘルパーの `ConductorNative.Interrupt`。コールバックは `Trigger` で旗を立てるだけ(別のスレッドで動く)。読む側は、先に `IsRaised` を読み、`$true` のときだけ `Number` を読む |
| 起動失敗 | 子プロセスを作れなかったこと(終了コード 127)。終了の状態を取得できない場合も、安全側でこれに含める |
| シグナル終了の推定 | UNIX 系で、`Process.ExitCode` が 129〜159 で、そのジョブの成功・警告のコードに含まれないときだけ、シグナル(`ExitCode - 128`)で終了したとみなすこと(07 章 4.4 節) |
| C# ヘルパー | `Add-Type` で読み込む小さな C# の型の組(名前空間 `ConductorNative`。`HelperInfo`・`Interrupt`・`SignalWatcher`・`UnixNative`)。シグナルのコールバックと、`kill`・ディレクトリの `fsync` の P/Invoke を持つ。ソースは `Private/Runner/NativeHelper.ps1`、読み込みは `Initialize-ConductorNativeHelper`(最初のジョブの起動までに遅延) |
| プラットフォーム層 | OS 差を吸収する場所(`Get-ConductorPlatformKind`、`Get-ConductorStartInfo`、`Send-ConductorSignal`、`Read-ConductorExit`、`Sync-ConductorDirectory`、`NativeHelper.ps1`)。OS の判定は `Get-ConductorPlatformKind` だけ |
| 木 | `.psd1`・JSON の読み取り結果の共通の木。ノードの `Kind` は `null`・`bool`・`int`・`hugeInt`(範囲外の整数。値を持たない)・`float`・`string`・`seq`・`map` の 8 種(マップは、位置つきのエントリの並び) |
| 段階(Intake、Parse、Resolve、VersionCheck、Walk) | 定義の読み込みの処理の段階。前の段階で失敗したら、そこで打ち切る(03 章 2.2 節) |
| 種類(`Kind`)・理由(`Reason`) | エラーの識別子(`syntax`、`typeMismatch` など。文字列で、`enum` を定義しない)と、固定の日本語の原因の文 |
| 案内 | ステートの誤りに付ける、`conductor reset` で回復できるという文 |
| `Deps` | `Cli` の差し替え口。環境(`Env`)、時刻(`Clock`)、標準出力・標準エラー(`Out`)、定義(`Defs`)、ステート(`States`)、変換(`Converter`)、プロセス(`Proc`)、シグナル(`Signals`)の、スクリプトブロックを持つ `[ordered]` のハッシュテーブル。内部の入口は `Invoke-ConductorMain -Arguments -Deps` |
| `Report` | 書き込みの窓口(`Write-ConductorStream` と `Send-Conductor*`)と、文の組み立て。標準出力の書き込みの失敗は `[bool]` の偽で返し、標準エラーの失敗は握りつぶす |
| flush | 標準出力・標準エラーの書き込みの完了(`Flush()`)。書くたびに行い、終了の前にも行う |
| 入口スクリプト | `bin/conductor.ps1`。モジュールを読み込み、生の引数を取り出し、`Invoke-Conductor -ArgumentList` を呼び、戻り値を `exit` に渡す。`exit` を呼ぶ唯一のファイル |
| TJ(テストジョブ) | 受け入れテスト用のスクリプト `tests/jobs/tj.ps1`。配布しない |
| AC | 受け入れテストの項目(`AC-xxx`)。11 章 4 章 |

## 6. この版が採った書式・実行環境の選択

| 項目 | 内容 | 章 |
|------|------|----|
| 定義ファイルの書式 | **`.psd1` を主、JSON を副**。書式は拡張子で決める(`.psd1`・`.json`。ASCII の大文字小文字を区別しない)。それ以外は `unsupportedFormat`(64。理由 `拡張子は .psd1、.json のいずれかで書きます`)。人が書く定義にコメントが要るときは `.psd1` を使う(`#` と `<# #>` が書ける) | 03 |
| 既定のファイル名 | `conductor.psd1`(JSON は `--config conductor.json` で明示。`conductor.json` を探索しない) | 03、04 |
| 定義のキー名 | `format_version`、`pipeline`(`name`、`description`、`run_id_env`、`remove_on_complete`)、`job`(`id`、`command`、`warning_exit_codes`、`success_exit_codes`、`transitions`、`retry`)。`.psd1` と JSON で同じ | 03 |
| `.psd1` の読み取り | **コードを実行しない**。`[System.Management.Automation.Language.Parser]::ParseInput` で構文木を作り、許可するノードの型(ハッシュテーブル、配列、単一引用符・展開のない二重引用符・here-string の文字列、整数(10 進・`0x`・`0b`)、浮動小数、`$true`・`$false`・`$null`)だけを自前で木に変換する。コマンド・変数・式・演算子・スクリプトブロック・キャストは拒否。数の字句は `Extent.Text` から自前で解析する。`Import-PowerShellDataFile` は位置を返さないので使わない(比較用のテストだけ)。`.psd1` のキーは、ASCII の大文字小文字を区別せず、重複とみなす。既知のキーの大文字小文字違いは `unknown key` | 03 |
| JSON の読み取り・書き出し | **全面自前**(`ConvertFrom-Json`・`ConvertTo-Json`・`System.Text.Json` を製品のコードで使わない)。定義・変換・ステートの 3 つが読み取りを共用する。明示のスタックで走査し(再帰しない)、位置・重複キー・整数の字句(範囲外は `hugeInt`)を扱う。書き出しはインデント 2、LF | 03、06、09 |
| 読み込みの上限 | ファイル 1 MiB、展開後の要素数 100,000(キーも数える)、文字の大きさ 16 MiB、入れ子の深さ 32 段(根を数えず、33 段目を拒否)。深い入れ子を再帰で処理しない | 03 |
| 空白の判定 | Unicode の White_Space(25 文字)を明示した自前の表。`.Trim()`・`[char]::IsWhiteSpace` を製品のコードで使わない | 03、07 |
| 文字列の比較 | 序数(`-ceq`、`StringComparison.Ordinal`)。`-eq`・`-like`・`-match`・`switch` の既定(大文字小文字を区別しない)を使わず、正規表現に頼らない手書きの走査にする。文字の単位はコードポイント(サロゲートの対を 1 文字) | 02、04、05 |
| 文化の扱い | ロケールは環境変数 `LC_ALL`・`LC_MESSAGES`・`LANG` だけで決める。`$PSCulture`・`CultureInfo` の現在の文化に依存せず、`InvariantCulture` を使う | 02、04、08 |
| 定義のエラーの種類 | 31 種(`unsupportedFormat` を含む)。誤りは位置の順に 1 行ずつ。理由は常に日本語の固定文で、入力の値を含めない | 03、08、10 |
| ステートファイル | JSON(UTF-8、BOM なし)。名前は `<定義ファイル名>.state`。キーは固定の順序、インデントは空白 2 つ、末尾は LF 1 つ。読み込みは 14 段階の検査。書き込みは、一時ファイル + `Flush($true)` + `[System.IO.File]::Move($src, $dst, $true)` + ディレクトリの同期(UNIX 系) | 06 |
| ステートの理由文 | 全 23 件 | 06、10 |
| ディレクトリの同期 | UNIX 系だけ、`libc` の `open`・`fsync`・`close` を C# ヘルパー(`UnixNative.SyncDirectory`)で行う。`Cli` が、`Write-ConductorState` の `-SyncDirectory`(スクリプトブロック。Windows では `$null`)に渡す | 06、07 |
| `convert` | `.psd1` と JSON の相互変換(`--to psd1\|json`)。入力の形式は `--to` の反対とみなし、拡張子は見ない。出力のキーの順は入力の記述順。JSON はインデント 2・末尾 LF。`.psd1` はインデント 4 スペース、裸のキーは `^[A-Za-z_][A-Za-z0-9_]*$` かつ予約語でないときだけ、文字列は単一引用符、LF のみ、BOM なし | 09 |
| 変換のエラー | 6 種(理由 17 件)。終了コード 67 | 09、10 |
| 引数の解析 | 自前(PowerShell のパラメータ束縛を使わない。値を取るオプション、省略形を受理しない点、エラーの優先順位と終了コードが合わないため) | 04 |
| ランチャーと入口 | 3 つ: `bin/conductor`(`#!/bin/sh`)、`bin/conductor.cmd`、`bin/conductor.ps1`(入口。`exit` を呼ぶのはここだけ)。`pwsh -NoLogo -NoProfile -NonInteractive -File <ps1> <引数>`。`POWERSHELL_UPDATECHECK=Off`、`POWERSHELL_TELEMETRY_OPTOUT=1`。`pwsh` が `PATH` に無いときは固定の 1 行と終了コード 70 | 02、04 |
| 公開関数 | `Invoke-Conductor` だけ。`-ArgumentList [string[]]` で生の引数を受け、`[int]` の終了コードを**返す**(`exit` を呼ばない)。`[CmdletBinding()]` を付けず、`$args` を使わない。生の引数は `[System.Environment]::GetCommandLineArgs()` の最初の `-File` の後ろから取り出す | 02、04 |
| 終了コード | 0、1〜255(ジョブの値)、64(定義)、65(ステート)、66(拒否)、67(変換)、68(使い方)、70(内部)、127(起動失敗)、128+番号(シグナル)。範囲合わせは `ConvertTo-ConductorProcessExitCode`(`exit` に 0〜255 の `[int]` だけを渡す) | 05、10 |
| 想定外の内部失敗 | `Invoke-Conductor` 全体を `try`/`catch` で囲み、`conductor: error: internal error` を `OpenStandardError` のストリームに 1 行出して 70。`PipelineStoppedException`(Ctrl+C)は内部の誤りにせず、`finally` で後始末する。深い入れ子は再帰でなく明示のスタックで処理する | 07、10 |
| 標準出力・標準エラー | `[Console]::OpenStandardOutput()`・`OpenStandardError()` のストリームを、BOM なし UTF-8 の `StreamWriter`(LF、`AutoFlush`、書くたびに `Flush`)で包んで書く(`Write-ConductorStream`)。`Write-Host`・`Write-Output`・`Write-Error`・`[Console]::Out` を使わない。書き込みの失敗は `[bool]` の偽で返し、`watch` が 65、`convert` が 67、`--help`・`--version` は無視して 0。**既知の差**: UNIX 系では、閉じたパイプへの書き込みが .NET の標準のストリームに捨てられ、`watch`・`convert` が 65・67 でなく 0 で終わりうる | 08、10 |
| パイプラインへの出力 | 関数の戻り値に出力を混ぜない(戻り値は `return` で 1 つ。配列は `return ,$array`)。`Set-StrictMode -Version Latest` と `$ErrorActionPreference = 'Stop'` をモジュールの先頭に置く | 02 |
| 型とデータ | `class` と `enum` を定義しない。値は `[pscustomobject]` と `[ordered]` のハッシュテーブル、種別は文字列(`-ceq` で比べる)。失敗は値で返し(`Ok` を持つ `[pscustomobject]`)、例外にするのは想定外だけ | 02、05、10 |
| 同期モデル | 同期(`Start-Job`・`Start-Process`・`Register-ObjectEvent`・`Task`・`Thread` を使わない)。待機は `Process.WaitForExit(int)`(1、2、4、8、10、10、… ms)と `Interrupt.Wait(int)`(250 ms 以下の刻み)。シグナルのコールバックは C# ヘルパーの中で完結し、旗を立てるだけ | 02、05、07 |
| シグナル | UNIX 系は SIGHUP・SIGINT・SIGQUIT・SIGTERM を `PosixSignalRegistration`(ヘルパーの静的なフィールドに保持)で受け、実行中の子の pid に `kill` で 1 回転送する。Windows は `Console.CancelKeyPress` の Ctrl+C だけを受け、転送しない。終了コードは 128+番号。Ctrl+C で `pwsh` がスクリプトを止めても、入口スクリプトの `finally` が、予約した終了コード(なければ 130)で `[System.Environment]::Exit` を呼ぶ。ヘルパーを使えないとき、`run`・`pipeline` は終了コード 70 | 07 |
| 子プロセス | `System.Diagnostics.Process`(`UseShellExecute = $false`、標準入出力は引き継ぐ。`ArgumentList` を使う)。UNIX 系でも新しいプロセスグループ・セッションに置かない。実行ファイルは絶対パスに自前で解決する。シグナル終了は `ExitCode` が 129〜159 で成功・警告のコードに含まれないときだけの推定。Windows の `.cmd`/`.bat` は、危険な文字を拒否したうえで `cmd.exe /d /s /c` を `Arguments` 文字列で直接起動する | 07 |
| 作業ディレクトリ | `Invoke-Conductor` が、開始時に `[System.Environment]::CurrentDirectory` を `$PWD.ProviderPath` に合わせ、終了時に戻す。`Set-Location` を使わない | 04 |
| ソースの構成 | `src/Conductor/`(`Conductor.psd1`、`Conductor.psm1`、`Public/Invoke-Conductor.ps1`、`Private/{Text,Json,RunId,Flow,Definition,Convert,State,Runner,Report,Cli}/`)、`bin/`、`tests/`(Pester 5。`Unit`・`Acceptance`・`Inspect`・`Perf`、`jobs/`、`support/`)、`build/`、`PSScriptAnalyzerSettings.psd1`。依存の向きは `Cli` → `Runner`/`State`/`Report` → `Flow`/`Definition`/`Convert`/`RunId` → `Json` → `Text`。AST の走査のテストで強制する | 02、11 |
| 品質ゲート | PSScriptAnalyzer(Error・Warning で失敗)、禁止コマンドの検査、書式(`Invoke-Formatter` の差分なし)、`Test-ModuleManifest`、Pester 5(3 OS、最小と最新の PowerShell)、カバレッジ(行 80% 以上。Linux)、版の一致の検査(`Invoke-Checks version`・`consistency`)、起動時間の計測(目安) | 11 |
| リリース | `v*` のタグ(`main` 上で、`ModuleVersion` と一致)で、`Conductor/` と `bin/` を zip(Windows 向け)と tar.gz(UNIX 系向け。実行権限を保つ)にし、`SHA256SUMS` を付けて、下書きのリリースに添付する。PowerShell Gallery へは、人が手で公開する(ワークフローは行わない)。Authenticode 署名は行わない(実行ポリシーと `Zone.Identifier` の注意と回避手順を、利用者向けに書く) | 11 |

## 7. 表記のきまり

- 章への参照は、`03 章 6.3 節` のように書く。ほかの章へのリンクは、このディレクトリのファイルだけである。
- コードブロックの PowerShell のコードは、関数の署名と値の形の骨格(実装ではない)である。C# のコードは、ヘルパーの型の署名の骨格(07 章)である。
- メッセージの文言は、`LANG=C`(英語)と日本語の両方を、各章の表に原文のまま書いてある。理由の文(定義・ステート・変換)は、ロケールに関わらず日本語の固定文である。
- 確認できなかった点は「未確認」と書き、[12](12-rebuild-guide.md) 3 章にまとめる。
