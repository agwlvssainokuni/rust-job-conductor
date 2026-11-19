# 11 品質とビルド(非機能要件、テスト、受け入れテスト、CI、リリース、デモ、依存、レイアウト)

対象: `conductor` v0.8.0 の C#(.NET)版。この文書は、C# で `conductor` を再実装した人が、(1) 同じ品質の基準を満たし、(2) 同じテストを書き、(3) 同じ CI とリリースの仕組みを再現するための仕様である。
この文書の基準を満たし、受け入れテスト(4 章)がすべて通れば、利用者から見て同じ挙動のツールになる。

| 章 | 内容 |
|----|------|
| 1 | この文書の位置づけと、C# 版の前提、確認の範囲 |
| 2 | 非機能要件(性能、堅牢性、セキュリティ、保守性)の基準値と、強制か目安かの区別 |
| 3 | テスト構成(単体テスト、受け入れテストの仕組み、テスト用ジョブ、偽の部品、xUnit の設定、OS 別のテスト、略記、判定の書き方) |
| 4 | 受け入れテスト一覧(機能ごと。ID は `AC-xxx`) |
| 5 | CI(GitHub Actions)の全ジョブと、品質ゲートの再現手順、検査プログラムの全文 |
| 6 | リリース |
| 7 | デモ |
| 8 | 依存ライブラリと、中央パッケージ管理・ロックファイルの運用 |
| 9 | リポジトリのレイアウト |
| 10 | 未確認の事項と、他の章との整合が要る点 |

他章との関係: 振る舞いの規定は [03](03-definition-format.md)(定義)、[04](04-cli.md)(CLI)、[05](05-execution-flow.md)(流れ)、[06](06-state-file.md)(ステート)、[07](07-process-and-signals.md)(プロセスとシグナル)、[08](08-output-and-i18n.md)(出力)、[09](09-converter.md)(変換)、[10](10-errors-and-exit-codes.md)(終了コード)にある。
構成は [02](02-architecture.md)、再構築の手順は [12](12-rebuild-guide.md) にある。受け入れテストは、それらを「前提 → 実行 → 期待」の形で、外から確かめる。

---

## 1. この文書の位置づけと、C# 版の前提

- 受け入れテストは、**ビルドした実行ファイル `conductor`(または、`dotnet conductor.dll`)を、`System.Diagnostics.Process` で外から実行して**確かめる(終了コード、標準出力、標準エラー、作られるファイル)。外からは確かめにくい項目は、区分を「単体」「注入」「点検」「手動」「計測」として示す(3.6 節)。
- 定義ファイルは **JSON(`.json`)を主**とし、YAML(`.yaml` / `.yml`)も受け付ける。既定のファイル名は `conductor.json`。書式は拡張子で決める(大文字小文字を区別しない)。ステートファイルは JSON で、名前は `<定義ファイル名>.state`(run ID があれば `<定義ファイル名>.<run ID>.state`)。`convert` は JSON と YAML の相互変換(`--to json|yaml`。入力の形式は `--to` の反対とみなし、拡張子は見ない)。人が書く定義にコメントが要るときは YAML を使う。
- 受け入れテストの ID は `AC-xxx`。001〜449 は、`conductor` が満たすべき基本の項目。450〜487 は、JSON と YAML の扱いと、品質・CI・版の項目。488 以降は、.NET(CLR の実行モデル、`string` が UTF-16、`System.Text.Json`、`Process`、`PosixSignalRegistration`、単一ファイルの配布物、文化依存の API)に固有の項目。番号は、この文書の中で一意。
- 期待の標準エラーの文言は、特に記さない限り `LANG=C`(英語)。理由の文(日本語の固定の文)は、英語の表示でも日本語のまま([08](08-output-and-i18n.md)、[10](10-errors-and-exit-codes.md))。理由の文の正確な字句は、03・08・10 章が正本で、この文書の例は、それと同じ字句を使う。JSON に固有の理由の字句が、03 章に定まっていない場合は、「03 章に従う」と書いた。
- 実行環境は .NET。サポートする版は「実装時点で公式のサポート中の LTS のうち、最小と最新」(以下 `MIN_TFM` と `MAX_TFM`。例は 2.5 節)。
- **この環境には .NET SDK が無く、実機で動かせない。** この文書の作成時に確かめたのは、公式ドキュメントの記述の範囲だけである。確認できた点には「確認(出典: …)」を付け、確認できなかった点は「未確認」と書いて、10 章にまとめる。

確認できた範囲(いずれも learn.microsoft.com、dotnet.microsoft.com、または各ツールの公式の文書。この文書の作成日に取得した版):

| 項目 | 確認の内容 | 出典 |
|------|-----------|------|
| サポート中の .NET | .NET 11(STS。サポート終了 2028-11-14)、.NET 10(LTS。サポート終了 2028-11-14)はサポート中。.NET 9(STS)と .NET 8(LTS)は 2026-11-10 にサポート終了 | .NET の公式のサポート方針の表(dotnet.microsoft.com の「.NET and .NET Core support policy」) |
| 単一ファイル | `PublishSingleFile`、`SelfContained`、`RuntimeIdentifier` で作る。RID ごとに別々に発行する。`*.runtimeconfig.json`・`*.deps.json` は同じファイルに入る。管理コードの DLL は束ねられ、実行時にメモリ上で読まれる。ランタイム自身のネイティブのファイルを 1 つにまとめるには `IncludeNativeLibrariesForSelfExtract`(自己展開を伴う。展開先は `DOTNET_BUNDLE_EXTRACT_BASE_DIR`、なければ Unix 系は `$HOME/.net`、Windows は `%TEMP%/.net`)。`EnableCompressionInSingleFile` は起動の費用を伴う。`Assembly.Location` は空文字列を返す。実行ファイルの場所は `Environment.ProcessPath`、同梱のファイルの場所は `AppContext.BaseDirectory` | 「Create a single file for application deployment」 |
| NativeAOT | `PublishAot`。動的な読み込み・`Reflection.Emit` は不可。トリミングが必須で、トリミングの非互換がある。生成した Linux のバイナリは、同じかより新しい Linux でだけ動く。ビルド側にネイティブのツールチェーン(Windows は C++ のワークロード、Linux は `clang`、macOS は Xcode のコマンドラインツール)が要る。不変グローバリゼーションなら ICU は要らない | 「Native AOT deployment overview」 |
| 中央パッケージ管理 | `Directory.Packages.props` に `ManagePackageVersionsCentrally=true` と `PackageVersion`。各プロジェクトの `PackageReference` は `Version` を書かない。最も近い `Directory.Packages.props` が使われる。推移的な依存の固定は `CentralPackageTransitivePinningEnabled`(版を下げる固定は不可)。複数のパッケージ元があると NU1507(パッケージ元の対応づけで回避) | 「Central Package Management」 |
| ロックファイル | `RestorePackagesWithLockFile=true` で `packages.lock.json` を作る。`dotnet restore --locked-mode`(`RestoreLockedMode=true`)で、ロックファイルと違えば失敗する。実行ファイルのプロジェクトは、ロックファイルをコミットする。SDK が暗黙に足すパッケージの版が SDK で変わるため、`global.json` の `rollForward` を `disable` にして SDK を固定することが、公式に勧められている | 「NuGet PackageReference in project files」 |
| `global.json` | `sdk.version` は完全な版(`10.0.100` の形。ワイルドカード不可)。`rollForward` の値は `patch`・`feature`・`minor`・`major`・`latestPatch`・`latestFeature`・`latestMinor`・`latestMajor`・`disable`。`version` があって `rollForward` がなければ `patch` | 「global.json overview」 |
| `dotnet format` | `--verify-no-changes`(整形が要るなら 0 以外で終わる)、`--no-restore`、`--severity`(`info`・`warn`・`error`。既定 `warn`)。`.editorconfig` の設定を読む。**対象のプロジェクトを復元・コンパイルし、アナライザーを動かすことがあるので、信頼できるコードにだけ使う** | 「dotnet format command」 |
| NuGet の監査 | `NuGetAudit`(既定 true)、`NuGetAuditMode`(`direct`・`all`。`net10.0` 以上では `all` が既定)、`NuGetAuditLevel`、警告 `NU1900`〜`NU1905`。`dotnet list package --vulnerable` は `--include-transitive` を付けないと推移的な依存を含まない。警告をエラーにするのは `WarningsAsErrors`。監査用の流れだけで失敗させる書き方が公式にある | 「Auditing package dependencies for security vulnerabilities」 |
| `dotnet-coverage`・`coverlet.collector` | `coverlet.collector` は `dotnet test --collect:"XPlat Code Coverage"` で使い、既定の出力は Cobertura(`TestResults/<GUID>/`)。**収集器は、しきい値の検査を持たない。Microsoft.Testing.Platform(MTP)の v2 とは併用できず、.NET 10 の `dotnet test` の標準の MTP 統合とも動かない**ことが、coverlet の文書に明記されている | coverlet の文書「VSTestIntegration」 |
| xUnit の並列実行 | 既定は、テストクラスごとに 1 つの収集(collection)で、収集の間は並列、収集の中は逐次。v2 と v3(4.0 より前)は `[assembly: CollectionBehavior(DisableTestParallelization = true)]` で無効にできる。v3 4.0 以降は `[assembly: Parallelization(Mode = ParallelMode.None)]` | xunit.net 「Running Tests in Parallel」 |
| `PosixSignalRegistration` | `Create(PosixSignal, Action<PosixSignalContext>)` で登録し、`Dispose()` で解除する。**ファイナライザ(`Finalize()`)も登録を解除する**(参照を保持し続けないと、GC で解除される) | 「PosixSignalRegistration Class」 |
| `Process.ExitCode` | 終了前に読むと `InvalidOperationException`。非同期の出力の処理の完了は、引数なしの `WaitForExit()` で待つ。**シグナルで終了した子の値については、公式の文書に記述がない(未確認)** | 「Process.ExitCode Property」 |
| BannedApiAnalyzers | `BannedSymbols.txt`(または `BannedSymbols.*.txt`)を `AdditionalFiles` に加える。1 行に、ドキュメント ID(例: `T:N.BannedType`)、任意で `;説明`。コメントは `//` | roslyn-analyzers の `BannedApiAnalyzers.Help.md` |
| `actions/setup-dotnet` | `dotnet-version`、`global-json-file`、`cache`(既定は無効。`packages.lock.json` を探す。ないと失敗する。`cache-dependency-path` で場所を指定)。`dotnet restore --locked-mode` と組み合わせる。例は `@v6` | 同 README |
| YamlDotNet | 対象は netstandard2.0・2.1、.NET 8.0、.NET 10.0、.NET Framework 4.7。MIT。低水準の解析と出力を持つと説明される。AOT 向けには、別パッケージの静的ジェネレーター(`StaticDeserializerBuilder`)がある | 同リポジトリの README、公開記事 |

- 版の数字(.NET の SDK、各ライブラリ、GitHub Actions の部品)は、例として書いたものを除き、「実装時に確認する」。

---

## 2. 非機能要件

「強制」は、基準を超えると受け入れテストが不合格になるもの。「目安」は、基準値で、超えたら警告として記録する(不合格にしない)。目安は、利用者から見える挙動を損なわない範囲で、別の値を採ってよい。「仮」は、暫定として置いた値で、実測してから見直す(この文書の作成時には測っていない)。

### 2.1 性能

p95 は、昇順に並べた測定値の `ceil(n × 0.95)` 番目(1 始まり)。測定値は、標準エラーに `[bench] <行>` として出し、環境変数 `GITHUB_STEP_SUMMARY` があれば、そのファイルに `- <行>` として追記する。基準を超えたときは、`警告: <名前>: <値> ms が基準 <限度> ms を超えた` を記録する。
環境変数 `STARTUP_BENCH_ENFORCE` が文字列 `true` のときだけ、目安の基準超えを、テストの失敗にする(CI の既定は `"false"`)。

.NET は、起動のたびにランタイムを立ち上げ、JIT でコードを機械語にする。単一ファイル・自己完結の配布物(6 章)は、ランタイム自身を同梱するが、**ReadyToRun も NativeAOT も使わない既定の構成では、起動の時間はネイティブの実行ファイルより長い**。ReadyToRun(`PublishReadyToRun`)・NativeAOT・トリミング・単一ファイルの圧縮(`EnableCompressionInSingleFile`)の有無で、起動時間がどう変わるかは、**この文書の作成時には測っておらず、未確認**。**起動時間は目安として測り、利用者から見える挙動の基準にはしない。** 起動時の費用を抑えるため、`--version`・`--help`・使い方の誤り・`reset`・`watch` の経路では、YAML ライブラリの型を読み込まない(YAML を読む `validate`・`run`・`pipeline`・`convert` が、読む直前に初めて使う。型の読み込みは、その型を使うメソッドが JIT される時点で起きるので、YAML の部品を別のクラスに分け、その入口を呼ぶまで触れない。AC-489)。

| 項目 | 基準 | 扱い | 受け入れテスト |
|------|------|------|----------------|
| 配布物(単一ファイル)の `conductor --version` の起動から終了まで(20 回。最初の 5 回を捨てる) | p95 250 ms(Linux・macOS)、500 ms(Windows)(仮) | 目安 | AC-488 |
| 同じ `--version` の、`dotnet conductor.dll`(フレームワーク依存)での起動 | 記録のみ(基準なし) | 目安 | AC-488 |
| 定義(ジョブ 10 個)の読み込みと検証(プロセス内)を、最初の 5 回(JIT の暖機)を捨てて 100 回 | p95 20 ms | 目安 | AC-397 |
| 起動前の処理(`command` の検査、ステートの読み、照合、速い書き込み)を、最初の 5 回を捨てて 100 回。ジョブ 10 個の定義 | p95 20 ms(仮)。`fsync` つきの耐久性のある書き込みは基準なし(記録のみ) | 目安 | AC-398 |
| `run` の `conductor` 自身のオーバーヘッド(ジョブ 10 個の定義、20 回。`conductor` 自身の起動を含む) | p95 600 ms(仮) | 目安 | AC-399 |
| `pipeline` のジョブ 1 つあたりの増分(ジョブ 10 個と 1 個の `pipeline` の差から求める。5 回) | p95 80 ms(仮) | 目安 | AC-400 |
| ジョブ 1,000 個の定義で `run job0` が終わるまで(起動を含む) | 2,500 ms(仮) | 目安 | AC-401 |
| ジョブ 1,000 個の `pipeline` の流れの判断(偽の `Deps` を使い、プロセスとファイルなし) | 10,000 ms | 目安。ただし、完了のステートが残ることは常に検査する | AC-402 |
| ジョブの実行中の `conductor` の CPU 時間の増分(Linux。待機の 1 秒後から 2.5 秒後までの増分) | 50 ms 以下 | **強制**(`/proc` を読めない環境ではスキップ) | AC-403 |
| 同じ定義への 2 つの `run` を同時に | 両方が実行され、どちらもジョブの終了コードで終わり、ステートは読み戻せる(排他制御はしない) | **強制** | AC-404 |
| リトライの待機の中断 | 合図から 1 秒以内に返る。待ちは、`ManualResetEventSlim.Wait` に短い期限(50 ms 以下)を与えて繰り返す、または合図で起こす(07 章の決定に従う) | **強制**(1 秒) | AC-275 |
| ジョブの終了の待ち | `Process.WaitForExit(int)` の短い区間(07 章)で待つ。合図から転送までの遅れは、区間の長さ以下 | 目安 | [07](07-process-and-signals.md) |
| ジョブが大量の出力(16 MiB)を標準出力に書いても、`conductor` がそれを溜め込まない | 子の標準入出力は引き継ぐ(`RedirectStandardOutput=false`)ので、出力は `conductor` を通らない。出力の全体がそのまま読み手に届く | **強制** | AC-494 |
| 配布物の大きさ(単一ファイル。トリミングなし) | 記録のみ(基準なし。大きさは測っていない。未確認) | 目安 | AC-526 |

測定のしかた:

- 「`conductor` 自身のオーバーヘッド」は、`conductor` を起動してから終了するまでの時間から、ジョブ自身の時間を引いた値。ジョブ自身の時間は、テスト用ジョブ `timing`(3.4 節)が出す `start_ns`・`end_ns` の差と、テスト用ジョブ自身の起動の費用(基準。`TJ exit 0` を直接 20 回起動した所要時間の中央値)の和。
- 「ジョブ 10 個の定義」は、10 個とも同じ形の簡単なジョブ(テスト用ジョブの `exit 0` か `timing`)。
- 「`pipeline` のジョブ 1 つあたりの増分」は、`(10 個の pipeline の全体 − 1 個の pipeline の全体 − 9 × ジョブ 1 つの自身の時間) / 9`。
- 測定の対象は、リリースと同じ作り方の成果物(`dotnet publish` した単一ファイル。6 章)。`dotnet run` や `dotnet conductor.dll` で測る場合は、記録に明記する。
- JIT の暖機があるため、プロセス内の測定は、最初の 5 回を捨てる。プロセスの起動の測定も、最初の 5 回を捨てる(ファイルのキャッシュの影響を除くため)。
- 性能のテストは、`tests/Conductor.Tests` の `Perf` 名前空間に置き、環境変数 `CONDUCTOR_PERF=1` がなければスキップする(`PerfFactAttribute` がその環境変数を見て `Skip` を設定する)。受け入れテストの性能(プロセスの起動を伴うもの)は、`Conductor.AcceptanceTests` の `Perf` 名前空間に置き、同じ規則にする。測定は 3 OS で行い、失敗しても全体は不合格にしない(目安)。

### 2.2 堅牢性

外部の入力(コマンドライン、`command` の文字列、ステートファイル、定義ファイル、変換の入力、OS の結果)に対して、**異常終了しない**(失敗は終了コードとメッセージで返す)。【強制】
.NET では、捕まえそこねた例外は、既定の動作として、メッセージとスタックトレースを標準エラーに出してプロセスを異常終了させる(UNIX 系では `SIGABRT` による終了になる場合がある。未確認)。`StackOverflowException` と `OutOfMemoryException` は、捕捉して続行すべきでない(`StackOverflowException` は捕捉できず、プロセスが終了する)。したがって、YAML・JSON の読み取りと、定義・ステート・変換の走査は、入れ子の深さの上限(32 段)を、**再帰に入る前に**検査する(深さの数え上げは、明示的なスタック(`Stack<T>` か配列)か、字句の走査で行い、スレッドのスタックの大きさを増やして逃げない。AC-412、AC-524、AC-232)。入口(`Program.Main`)は、`try`/`catch (Exception)` と、`AppDomain.CurrentDomain.UnhandledException`、`TaskScheduler.UnobservedTaskException` で、捕捉できる残りの失敗を 1 行(`conductor: error: internal error`)にして、終了コード 70 で終わる(AC-414、AC-490、AC-491)。
標準出力・標準エラーへの書き込みの失敗(`IOException`(閉じたパイプ・`ENOSPC`)、`ObjectDisposedException`、`UnauthorizedAccessException`)も、入口で処理する。終了時の flush の失敗で、終了コードが変わったり、例外が出たりすることを許さない(AC-496)。

| 項目 | 基準 | 受け入れテスト |
|------|------|----------------|
| ランダムな引数 | 固定の種のランダムな引数列(語の表から 0〜6 個を選ぶ。乱数は 3.1 節の `Xorshift32`)500 回で、終了コードが 0〜255、標準エラーに制御文字(改行を除く)と U+202E がない | AC-405 |
| UTF-8 でない引数(UNIX 系) | ジョブID、`--config`、オプション、`convert` の引数に、不正なバイト列を渡しても異常終了しない。**.NET が、不正な UTF-8 を `string[] args` でどう復号するか(U+FFFD に置き換える見込みだが)は、公式の文書で確認できていない(未確認)**。実装は、不正なバイト列が、何かの `string` になっても、異常終了せず、出力が有効な UTF-8 になることを保証する(AC-406、AC-519) | AC-406、AC-519 |
| 壊れたステート | 壊れた JSON、ランダムなバイト列、状態・再開位置・フラグを乱したステート、巨大な値で、異常終了しない(終了コード 65) | AC-407 |
| 空・制御文字を含むジョブID | 異常終了しない | AC-408 |
| ランダムな `command` | 文字集合 `a b ⇥ ⏎ ' " \ ; \| & $ * あ` から長さ 0〜39 の文字列 2,000 件で、分割が成功したら `program` は空でない。成功か誤り(64)かのどちらか | AC-409 |
| 大きな `command` | 約 30 万引数が分割できる。閉じない引用符は誤り。再帰に依存しない | AC-410 |
| 起動できない `command` | 空文字列、`'`、`''`、存在しない名前、ディレクトリのパス、`/`、NUL を含む文字列は、すべて起動失敗(または定義の誤り)。`Process.Start` が投げる `Win32Exception`・`InvalidOperationException`・`ArgumentException`・`PlatformNotSupportedException` などを、すべて起動失敗として扱う。範囲外のシグナル番号の合図でも起動せず中断として扱う | AC-411、AC-522、AC-548 |
| 入れ子の深い入力 | 深さ 1,000 と 100,000 の YAML・JSON(定義・ステート・変換)で、異常終了せず失敗を返す(`StackOverflowException` でプロセスが落ちない) | AC-412、AC-524 |
| 整数の字句 | `true`/`false`、小数(`1.0`)、指数(`1e0`)、`NaN`・`Infinity`、桁数が極端に多い整数で、異常終了せず、整数として受理しない。`long` を超える値、`double`・`decimal` の丸めで別の整数に化ける値を、範囲内として受理しない | AC-502 |
| 製品のコードの書き方 | 外部の入力の経路に、異常終了を起こす書き方(`Debug.Assert` による検証、`!`(null 許容の抑制)の濫用、`dynamic`、`checked` の例外の漏れ、入口以外での `Environment.Exit`・`Environment.FailFast`、`Exception` を黙って捨てる `catch`、`Task.Wait`・`.Result`)がない | AC-421 |
| 内部の誤り | 入口の `try`/`catch` と `UnhandledException` が、固定の英語 `conductor: error: internal error` を標準エラーに 1 行書き、終了コード 70。標準エラーが閉じていても 70 | AC-414、AC-490、AC-491 |
| 変換の入力 | 範囲外・空・壊れた入力、ランダム・切り詰めた入力で異常終了しない。成功したランダムな構造は往復で等しい | AC-413 |
| 標準出力の失敗 | 標準出力への書き込みの失敗(`IOException`(閉じたパイプ)、`ENOSPC` など)を捕捉する。`watch` は 65、`convert` は 67 で、メッセージを標準エラーに出す | AC-186、AC-236、AC-493、AC-495 |
| 標準出力の取りこぼし | 標準出力がパイプで、読み手が遅くても、終了前に flush して書き込みの完了を待つ。出力が途中で切れない | AC-494 |
| 終了時の flush | 標準出力が閉じていても、終了時に例外が出ず、終了コードが変わらない | AC-496 |
| 終了後に残らない | すべてのサブコマンドが、出力の完了から 1 秒以内に、プロセスとして終了する(前景スレッド・タイマー・シグナルの登録が残ってハングしない) | AC-497 |
| 警告を出さない | ビルドの警告が 0 件(`TreatWarningsAsErrors`。アナライザー込み)。通常の実行で、ランタイムの警告・未処理の例外の表示が標準エラーに出ない | AC-498 |
| 時間の上限 | プロセスを起動するテストは、すべて 30 秒の上限つき(超えたら子プロセスを強制終了して不合格) | 3.3 節 |

### 2.3 セキュリティ

| 項目 | 基準 | 受け入れテスト |
|------|------|----------------|
| シェルを呼ばない | 起動は、`command` の分割(または配列)で得た `program` と引数を、`ProcessStartInfo`(`UseShellExecute = false`。引数は `ArgumentList`)に渡すだけ。`Process.Start(string)`・`Process.Start(string, string)`、`UseShellExecute = true`、`sh`・`cmd`・`bash`・`powershell`・`-c`・`/C` を挟む経路がない(Windows の `.cmd` / `.bat` の安全策は 07 章) | AC-116(点検)と 4.2 節 |
| 引数にメタ文字が入っても注入にならない | `a;b`、`&&`、`\|`、`$HOME`、`*`、`>x` などが、そのまま引数として渡る。Windows の `.cmd` への危険な引数で、続くコマンドが実行されない | 4.2 節 |
| 検証は副作用を持たない | 定義の検証は、コマンドを解釈・展開・起動しない。定義ファイルのディレクトリは変わらない | AC-415 |
| 入力の値を表示に出さない | すべての種類のエラーの表示に、入力の値(秘密のつもりの文字列、`command`、ライブラリのエラー文)が現れない。キー名は制御文字をエスケープして表示。**YAML ライブラリが投げる例外のメッセージ(入力の断片と位置を含みうる)と、例外のスタックトレースを、そのまま表示に出さない**(行・列の位置だけを使う) | AC-416 |
| YAML からオブジェクトを作らない | YAML の読み取りは、構文要素(イベント)の段階まで(低水準の `Parser`)で、**型を指定した復元(`Deserializer`、`DeserializerBuilder`、`StaticDeserializerBuilder`)を使わない**(型名を持つタグで、任意の型が生成されうるため)。タグはタグとして拒否する | AC-531 |
| ステートに機微な値を書かない | ステートファイルに、`command`、引数、環境変数の値、標準出力の内容が現れない | AC-417 |
| 表示のエスケープ | 制御文字・方向制御文字・孤立サロゲートを無効化する(4.14 節の表)。定義・ステート・変換の各部が、同じ入力に同じ結果を返す | 4.14 節、AC-519 |
| ネットワークに接続しない | 実行中に外向きの通信をしない。製品のコードが `System.Net.*` の名前空間、`HttpClient`、`Socket` を使わない。実行時の依存(YAML ライブラリ)も、実行中に通信しない | AC-419(点検) |
| 動的なコードの実行をしない | 製品のコードが、`System.Reflection.Emit`、`Assembly.Load*`、`Activator.CreateInstance`(型名の文字列から)、`Type.GetType(string)`、`System.Linq.Expressions` の `Compile`、`BinaryFormatter`、`System.CodeDom`、`System.Runtime.Loader` を使わない。ネイティブの呼び出し(P/Invoke)は、07 章が決めた `kill` などの、許可の一覧にある関数だけ | AC-420(点検) |
| シグナルの送信 | ジョブの子プロセスの pid だけ(0 以下にならない)、許可したシグナルだけ、1 回だけ(プロセスグループには送らない) | 4.10 節の単体の項目 |
| テスト専用の経路を製品に持たない | 製品の入口に、わざと異常終了するコマンドや、テスト用の環境変数・隠しオプションがない。配布物に `tests/` の成果物(`Conductor.TestJob` など)を含めない | AC-422(点検)、AC-526、6 章 |
| 変換の副作用 | 変換の部品(`Conductor.Convert`)が `System.Diagnostics.Process`・ネットワークの型を使わない。ファイルを開くのは、入力ファイルを読む入口だけで、本体(文字列から文字列への変換)と書き出しは、ファイルにも環境変数にもカレントディレクトリにも触れない。`convert` は入力以外のファイルを作らず、カレントディレクトリの変更と環境変数に左右されない | AC-418 |
| 既知の脆弱性 | 依存の既知の脆弱性を、定期的に(毎週)と、変更のたびに、`dotnet list package --vulnerable --include-transitive` と `--deprecated` で検査する。結果は警告で、マージをブロックしない。復元時の `NuGetAudit` の警告(`NU1901`〜`NU1904`)は、通常の流れではエラーにしない(5.2 節) | 5 章 |
| 供給網(ビルドの再現) | `Directory.Packages.props` に、直接の依存の正確な版を書き、`packages.lock.json` をコミットし、CI は `dotnet restore --locked-mode` で入れる(8 章)。実行時の依存は 1 つ(8 章) | 5 章、AC-478 |
| 配布物の完全性 | チェックサムのファイル(`SHA256SUMS`)を付ける(署名はしない。転送の破損の検知まで。macOS の署名・公証と Windows のコード署名は行わない。6.5 節) | 6 章 |

### 2.4 保守性

| 項目 | 基準 | 受け入れテスト |
|------|------|----------------|
| OS 固有コードの隔離 | OS ごとの分岐(`OperatingSystem.IsWindows()`、`RuntimeInformation.IsOSPlatform`、`#if`)は、`src/Conductor.Runtime/Runner/PlatformInfo.cs`(名前空間 `Conductor.Runner`)の 1 か所だけ。ほかの部品は、そこが出す関数・値(UNIX 系か Windows か、受け取るシグナルの一覧、ディレクトリの同期の可否)だけを使う | AC-420 |
| 外部の入力で異常終了しない書き方 | 2.2 節 | AC-421 |
| 静的検査 | `dotnet build -warnaserror`(`Nullable=enable`、`TreatWarningsAsErrors=true`、`AnalysisLevel`、`EnforceCodeStyleInBuild`、BannedApiAnalyzers の `BannedSymbols.txt`)で、警告 0。`dotnet format --verify-no-changes` で、差分 0 | AC-480、AC-483、AC-518 |
| 整形と静的検査 | `dotnet format --verify-no-changes` の差分、ビルドの警告(アナライザーの指摘)が 1 つでもあればマージを止める | 5 章 |
| 命名 | .NET の命名の規約(型・メソッド・プロパティ・名前空間は PascalCase、ローカル変数・引数は camelCase、プライベートなフィールドは `_camelCase`、定数は PascalCase、インターフェイスは `I` で始める)に従う。`.editorconfig` の `dotnet_naming_rule` で、ビルドの警告にする。独自の規則は設けない | (規約。`.editorconfig`) |
| 部品の分け方 | `Conductor.Core`(`Definition`・`Convert`・`Json`・`RunIds`・`Flow`。入出力なし。ファイルとプロセスに触れずに検査できる)、`Conductor.Runtime`(`State`・`Runner`。入出力あり)、`Conductor.Cli`(`Report`・`Cli`・`Program.cs`。引数・流れの組み立て・`Deps`)を分ける([02](02-architecture.md))。依存の向き `Cli` → `Runtime` → `Core` は、プロジェクト参照で強制し、名前空間の間の依存と循環を `tests/Conductor.Tests` の点検で検査する | AC-514 |
| ソースへの著作権表示 | 各ソースファイル(`.cs`、`.csproj`、`.props`、`.yml`、`.sh`、`.cmd`)の先頭に、Apache License 2.0 のヘッダ(`Copyright 2026 agwlvssainokuni`)を置く | AC-423(点検) |

### 2.5 サポートする .NET の版

- 実装時点(この文書の作成日 2026-11-20)で、サポート中の LTS は .NET 10 だけである(.NET 8 は 2026-11-10 にサポートが終わった。1 章の確認)。したがって、**この文書の作成時点の例では、`MIN_TFM` と `MAX_TFM` は、どちらも `net10.0`(最小と最新が同じ)になる**。次の LTS(.NET 12 と見込まれるが、未確認)が出ると、「最新 LTS と、サポート中の 1 つ前の LTS」が 2 つに分かれ、`MIN_TFM = net10.0`、`MAX_TFM = net12.0`(例)となる。**具体的な版は実装時に確認して決める。**
- `MIN_TFM` と `MAX_TFM` は、`Directory.Build.props` の `ConductorMinTfm`・`ConductorMaxTfm` に書き、製品のプロジェクトの `TargetFramework` は `$(ConductorMinTfm)`、テストのプロジェクトの `TargetFrameworks` は、`$(ConductorMinTfm);$(ConductorMaxTfm)`(同じなら 1 つ)とする。`global.json` の SDK は `MAX_TFM` の SDK。CI の環境変数 `DOTNET_MIN_TFM`・`DOTNET_MAX_TFM` と、`global.json`・`Directory.Build.props` の一致を `Conductor.Checks consistency`(5.4 節)が検査する。
- 配布物(単一ファイル。6 章)は、ランタイムを同梱するので、利用者は .NET を入れなくてよい。フレームワーク依存の `conductor.dll` を `dotnet` で動かす場合の、利用者の側の最小は `MIN_TFM` のランタイム。

---

## 3. テスト構成

### 3.1 方針

- 方法論は test-after(実装してから、層ごとにテストを書いて実行する)。単体テストと、ビルドした実行ファイルを `Process` で起動する受け入れテストを持つ。
- テストの枠組みは **xUnit v2 系**(`xunit` 2.x、`xunit.runner.visualstudio`、`Microsoft.NET.Test.Sdk`)。アサーションは xUnit の `Assert`。**xUnit v3 を採らない理由**: v3 は Microsoft.Testing.Platform(MTP)で動く。coverlet の文書は、`coverlet.collector` が MTP と併用できないと明記している(1 章)。この仕様は、カバレッジを `coverlet.collector` で測るため、VSTest で動く xUnit v2 を使う。`global.json` に `"test": { "runner": "Microsoft.Testing.Platform" }` を書かない。xUnit v3 と `dotnet-coverage` の組み合わせは、**この文書の作成時には確かめていない(未確認)**。NUnit でも同等の構成にできる(その場合、並列の無効化は `[assembly: NonParallelizable]`・`[assembly: Parallelizable(ParallelScope.None)]` に読み替える。NUnit の記述は確認していない)。
- テスト用の別のライブラリ(モックのライブラリ(`Moq`・`NSubstitute`)、`FluentAssertions`、`Bogus`、`AutoFixture`、プロパティテストのライブラリなど)は入れない(8 章)。偽の部品は手で書く。
- 一時ディレクトリは、各テストが `Path.Combine(Path.GetTempPath(), "conductor-test-" + Guid.NewGuid().ToString("N"))` で作り(`TempDir` クラス。`IDisposable`)、終了時に消す(以下 `T`)。**テストごとに別のディレクトリ**なので、ステートファイルの名前(`<定義>.state`)は、テストの間で衝突しない。環境変数は、テストのプロセスの `Environment.SetEnvironmentVariable` を使わず、子プロセスの `ProcessStartInfo.Environment` に渡す(プロセス内の単体テストは、偽の `Deps` で環境を渡す)。
- 並列実行: **xUnit の並列実行を、受け入れテストのアセンブリで無効にする**(`[assembly: CollectionBehavior(DisableTestParallelization = true)]`。1 章で確認した v2 の方法)。理由は、(1) 受け入れテストが子プロセスとシグナルを扱い、並列にすると CPU の取り合いで 30 秒の上限と時間の基準が不安定になる、(2) 時間の測定の安定。単体テストのアセンブリ(`Conductor.Tests`)は、並列でよい(各テストが自分の `T` だけを使い、静的な可変の状態を持たないため)。ただし、`PosixSignalRegistration`・`Console.CancelKeyPress`・環境変数・カレントディレクトリなど、**プロセス全体に作用するものを触るテストは、1 つの収集 `[Collection("process-global")]` にまとめる**(収集の中は逐次)。同じ `T` を複数のテストで共有しない。
- 入力と期待の組が多いものは、**テーブル駆動**(`[Theory]` と `[MemberData]`・`[InlineData]`)で書く。ID(`AC-xxx`)は、テストの名前に含める(例: `public void AC_092_SingleQuoteNotClosed()`。`[Theory]` は、ケースの表示名に `"AC-092"` を含める)。`dotnet test --filter "FullyQualifiedName~AC_092"` のように、名前で選べる。
- 行カバレッジの下限は **80%**(5 章)。
- プロセスを起動するテストは、30 秒の上限で、ハングだけを検知する(3.3 節)。
- 乱数が要るテストは、固定の種の擬似乱数 `Xorshift32` を使う(`System.Random` は、版による出力の違いを避けるため使わない。`new Random(seed)` の列が .NET の版をまたいで同じかは、公式で確認していない。未確認)。実装(32 ビット符号なし。`uint` の演算は、既定で折り返す(`unchecked`)。プロジェクトで `CheckForOverflowUnderflow` を有効にしないこと。有効にする場合は `unchecked(...)` で囲む):

```csharp
namespace Conductor.Tests.Support;

/// <summary>固定の種で、版によらず同じ列を返す擬似乱数。</summary>
public sealed class Xorshift32
{
    private uint _state;

    public Xorshift32(uint seed)
    {
        _state = seed != 0 ? seed : 0x9E3779B9u;
    }

    public uint NextU32()
    {
        uint x = _state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        _state = x;
        return x;
    }

    /// <summary>0 以上 n 未満の整数。(わずかな偏りは許す。試験の入力を作るだけ)</summary>
    public int Below(int n)
    {
        return (int)(NextU32() % (uint)n);
    }
}
```

### 3.2 部品ごとの単体テスト

`src/Conductor.<部品>/` の各名前空間は、`tests/Conductor.Tests/<名前空間>/` に単体テスト(`*Tests.cs`)を置く。

| 部品 | 主な試験対象 | 方法 | 主な AC |
|------|--------------|------|---------|
| `Conductor.Definition`・`Conductor.Json` | 定義の読み込み・検査・型・エラーの種類と行番号。JSON と YAML の読み取り(タグ・複数文書・重複キー・非文字列キーの拒否、深さと大きさの上限)。`command` の分割。整数の字句検査(真偽値・小数・範囲外) | テーブル駆動。`tests/Conductor.Tests/Fixtures/*.json`・`*.yaml` の入力例(`CopyToOutputDirectory`) | AC-001〜098、AC-450〜、AC-502、AC-522〜525 |
| `Conductor.Convert` | JSON ⇔ YAML の変換、往復、表せない値、アンカーの展開、暴走する入力の上限 | テーブル駆動。往復の性質テスト(固定の種) | AC-206〜238 |
| `Conductor.RunIds` | run ID の検査(文字種と長さの規定は [05](05-execution-flow.md))と決定の順序 | テーブル駆動 | AC-239〜260 |
| `Conductor.Flow` | 純粋な判断: 分類 10 項目、遷移、再開、リトライの待ち時間(`ulong` の飽和)、終了コードの範囲合わせ | テーブル駆動。ファイル・プロセスなし | AC-268、AC-392〜394、AC-501 |
| `Conductor.State` | ステートの読み書き(実ファイル、`T`)、整合性検査(14 段階)、原子的な置換(`File.Move(…, overwrite: true)` または `File.Replace`。06 章の決定)、reset(`--all`、シンボリックリンク) | 実ファイル。書き込みの失敗は、置き換え先をディレクトリにして起こす | AC-302〜324 |
| `Conductor.Runner` | 子プロセスの起動と待ち、環境変数・作業ディレクトリの引き継ぎ、シグナルの転送(1 回だけ・許可した番号だけ)、シグナル名と番号の対応 | テスト用ジョブ `TJ` を実際に起動 | AC-099〜116、AC-280〜292、AC-504、AC-505 |
| `Conductor.Report` | 表示の全種類の文言(英語・日本語)、ロケールの決定、色、エスケープ、`watch` の表 | テーブル駆動 | AC-350〜388 |
| `Conductor.Cli` | 引数の文法、各サブコマンドの流れ、終了コードの決定 | **偽の `Deps`**(3.7 節)。ファイル・プロセス・シグナルなし | AC-125〜、AC-293〜300、AC-325〜349 |
| `tests/Conductor.Tests/Support/` | 偽の部品、一時ディレクトリの補助、記録の読み取り、`Xorshift32` | (試験の道具。単体試験は不要。カバレッジの対象は `src/` だけなので、自然に除かれる) | — |

### 3.3 受け入れテストの仕組み(`Process` の統合テスト)

受け入れテストは、`conductor` と、テスト用ジョブ `TJ`(3.4 節)を、`System.Diagnostics.Process`(`ProcessStartInfo`。`UseShellExecute = false`)で起動して確かめる。置き場所は **`tests/Conductor.AcceptanceTests/`** の `*Tests.cs`。機能ごとにファイルを分ける。

| ファイル | 内容 |
|----------|------|
| `Support/ConductorFixture.cs` | 共通の fixture(`conductor` の起動方法の決定。下記。`IClassFixture` で共有) |
| `Support/Harness.cs` | 共通の補助(3.8 節): `RunConductor`、定義とステートの書き出し・読み取り、30 秒の上限 |
| `ValidateTests.cs` | 4.1 節、4.2 節、450〜 の定義の検査 |
| `RunTests.cs` | 4.3 節、4.4 節(`run`、`pipeline`) |
| `AuxTests.cs` | 4.5 節、4.6 節(`watch`、`reset`)、4.8 節(run ID) |
| `ConvertTests.cs` | 4.7 節 |
| `RetryTests.cs` | 4.9 節 |
| `SignalTests.cs` | 4.10 節(UNIX 系の項目は `FactUnixAttribute`) |
| `StateTests.cs` | 4.11 節 |
| `ArgsTests.cs` | 4.12 節〜4.15 節 |
| `NfrTests.cs` | 4.16 節(堅牢性) |
| `DotnetTests.cs` | 4.20 節(.NET に固有の項目) |
| `DemoTests.cs` | 4.17 節(デモ。UNIX 系と Windows で `FactUnix`・`FactWindows`) |
| `DistTests.cs` | 4.18 節(配布物。ビルド済みの成果物を、環境変数 `CONDUCTOR_DIST_DIR` で受ける。なければスキップ) |
| `Perf/*.cs` | 2.1 節の計測(`CONDUCTOR_PERF=1` のときだけ) |
| `tests/Conductor.Tests/Inspect/*.cs` | 点検(ソースの文字列の走査とプロジェクトファイルの XML の走査。3.6 節) |

`conductor` の起動方法の決め方(`ConductorFixture`。クラスに 1 回):

1. 環境変数 `CONDUCTOR_BIN` があれば、そのパスを `conductor` として**直接実行する**(配布物の単一ファイルを試験するため。AC-446)。拡張子が `.dll` なら、`dotnet <パス>` で実行する。
2. なければ、環境変数 `CONDUCTOR_DLL` があれば、`dotnet <そのパス>` で実行する(フレームワーク依存のビルドを、別の版のランタイムで試験するため。`MAX_TFM` のランタイムで `MIN_TFM` のビルドを動かすには、環境変数 `DOTNET_ROLL_FORWARD=Major` を子に渡す。2.5 節)。
3. どちらもなければ、テストのビルドが出力ディレクトリに置いた `src/Conductor.Cli` のビルド(`ProjectReference` で、`ReferenceOutputAssembly="false"` と `Targets="Build"` を使って、ビルドの順序を保証し、出力を `$(OutDir)/conductor/` にコピーする)の `conductor`(Windows は `conductor.exe`)を実行する。コピーの方法は、実装時に決めてよい(受け入れテストのプロジェクトに、`conductor` の出力の場所を MSBuild のプロパティで渡す)。
4. テスト用ジョブの実行ファイルは、環境変数 `CONDUCTOR_TESTJOB` があればそのパス、なければ `Conductor.TestJob` のビルド出力(同様にコピー)。

各テストは、`T` をカレントディレクトリにして、`new ProcessStartInfo(exe) { WorkingDirectory = T, RedirectStandardOutput = true, RedirectStandardError = true, RedirectStandardInput = true, UseShellExecute = false }` に、`ArgumentList` で引数を足して実行する(`stdin` は、既定で、すぐ閉じる(空))。`Environment` は、いったん `Clear()` して 3.5 節の既定の環境変数だけにする(親の環境を引き継がない)。**標準出力と標準エラーは、2 つとも非同期に読み切る**(片方を読まずに他方を待つとデッドロックするため。`Process.BeginOutputReadLine` か、2 つの `Task` で `ReadToEndAsync`。受け入れテストの補助の中だけで `Task` を使い、製品のコードでは使わない)。バイト列で比べる項目(AC-494 など)は、`BaseStream` から読む。
30 秒を超えたら、`conductor` を `Process.Kill(entireProcessTree: true)` で強制終了し、記録(`pid=`)に書かれたジョブの pid があればそれも強制終了して、テストを失敗にする(`Harness.RunConductor` の `WaitForExit(30_000)`)。加えて、`dotnet test` の `--blame-hang-timeout 5m` で、テストの全体のハングに備える(`--blame-hang` の動作と書式は、採用した SDK の `dotnet test --help` で確かめる。未確認)。

### 3.4 テスト用ジョブ(`tests/Conductor.TestJob`、以下 `TJ`)

受け入れテストで使う、小さな .NET の実行ファイル(`OutputType=Exe`。`AssemblyName` は `tj`)。配布物には含めない(6 章)。起動の費用を抑えるため、**BCL の `System`・`System.IO`・`System.Text`・`System.Threading`・`System.Runtime.InteropServices` だけ**を使う(`System.Text.Json`・`System.Diagnostics.Process`・`System.Linq` を読み込まない。JIT の費用を避ける)。`InvariantGlobalization=true`、`TieredPGO` は既定。
**.NET の起動の費用の扱い**: 1 回の `TJ` の起動は、ランタイムの起動と JIT を伴い、数十〜百数十 ms の見込み(実測は実装時に確認する。未確認)。(1) `TJ` は、**フレームワーク依存の `tj.dll` を `dotnet tj.dll` で起動するのではなく、アプリホスト(`tj`、Windows は `tj.exe`)を直接起動する**(`dotnet` の起動の二重の費用を避ける。ただし、アプリホストは、`DOTNET_ROOT` か既定のインストール場所から、ランタイムを探す。CI では `actions/setup-dotnet` が `DOTNET_ROOT` を設定する見込みだが、未確認。見つからない環境では、`dotnet tj.dll` に切り替える)。(2) 起動を速める設定として、`TieredCompilation`(既定)、`TieredPGO=false`、`AutoreleasePoolSupport` は不要。単一ファイル・ReadyToRun で `TJ` を発行して使う案もあるが、効果は未確認で、採らない(ビルドの手順が増える)。(3) 大量のジョブを要するテスト(AC-402 など)は、`TJ` を使わず、偽の `Deps` で行う。(4) 性能の測定では、`TJ` 自身の起動の費用を基準として測り、差し引く(2.1 節)。
定義の中では、`TJ` は、1 つの語 `"<tj の絶対パス>"`(各語を二重引用符で囲む。パスに空白があってもよい)を表す。`dotnet tj.dll` 形式のときは、2 つの語 `"<dotnet の絶対パス>" "<tj.dll の絶対パス>"`。例: `"command": "TJ exit 3"` は、定義の中で `"/repo/tests/Conductor.TestJob/bin/Debug/net10.0/tj" exit 3` という `command` になる。
定義を書き出す補助(`Harness.WriteDefinition`)は、すべての文字列を JSON の文字列(`System.Text.Json` の `JsonEncoder`、または手書きのエスケープ。テストの補助なので `System.Text.Json` を使ってよい)で出す。YAML の二重引用符の文字列は、JSON の文字列と同じ値を表す(ただし、U+2028・U+2029、孤立サロゲートを含む文字列は、この補助の対象外。必要な項目は、手で書く)。

```text
TJ [--out <ファイル>] <モード> [引数...]
```

- `--out <ファイル>` が先頭にあれば、**記録の行**をそのファイルに追記する(`File.AppendAllText(file, line + "\n", new UTF8Encoding(false))`。1 行 1 件)。なければ標準出力に書く。ジョブの標準出力を被試験体越しに読めない場合があるため、結果はファイルで受け取る。
- 記録に出す文字列(値)は、`Q(s)` で表す: 二重引用符で囲み、`"` と `\` の前に `\` を置き、改行は `\n`、復帰は `\r`、タブは `\t` と書く(JSON のエスケープとは別。非 ASCII はそのまま。他の制御文字は変換しない)。
- 未知のモードは、終了コード 2。
- 終わり方は、`Main` が `int` を返す。`say` は、書き込みの `Flush()` を済ませてから終わる。UNIX 系で、終了コードは下位 8 ビットだけが親に届く(`exit 300` が 44)。

| モード | 動作 |
|--------|------|
| `exit <n>` | 終了コード n で終わる(省略・整数でないときは 2) |
| `run <label> <n>` | 記録 `ran=<label>` を出し、終了コード n(省略・整数でないときは 0) |
| `echo <args...>` | 各引数を、記録 `arg[i]=Q(引数)`(i は 0 から)として 1 行ずつ出し、終了コード 0 |
| `env <NAME>...` | 各環境変数を、設定されていれば `env[NAME]=set:Q(値)`、なければ `env[NAME]=unset` として出す(`Environment.GetEnvironmentVariable(name) is not null`)。終了コード 0 |
| `cwd` | カレントディレクトリ(`Directory.GetCurrentDirectory()`)を `cwd=Q(パス)` として出す。終了コード 0 |
| `pid` | 自分のプロセス識別子を `pid=<N>`(`Environment.ProcessId`)として出す |
| `say <out> <err> [n]` | `<out>` を標準出力に、`<err>` を標準エラーに、改行を付けずにそのまま書き(`Flush()` して完了を待つ)、終了コード n(省略は 0)。`--out` に従わない |
| `sleep [秒]` | `pid=<N>` と `ready` を記録し、眠る(`Thread.Sleep(50)` の繰り返し)。秒があれば、その秒のあと終了コード 99。なければ眠り続ける。SIGTERM・SIGINT・SIGHUP を受けたら、ハンドラが、自分のシグナルのハンドラを解除し(`PosixSignalRegistration.Dispose()`)、`kill(getpid(), 受けたシグナル)`(P/Invoke)で自分に同じシグナルを送り直す(シグナルで終了したように観測されるようにする。確認は実装時。10 章)。`Context.Cancel` は `false` のまま。Windows では、`Console.CancelKeyPress` で登録できるものだけを登録する |
| `trap <ms> <code>` | `pid=<N>` と `ready` を記録する。SIGTERM・SIGINT・SIGHUP を受けたら(`PosixSignalRegistration`。`Cancel = true`)、記録 `trapped` を出し、`<ms>` ミリ秒待ってから、記録 `done` を出し、終了コード `<code>` で終わる(シグナルで中断された後も、ジョブが終わるまで `conductor` が待つことを確かめる。AC-505) |
| `flaky <counter-file> <fails> <code>` | `<counter-file>` の整数(なければ 0)に 1 を足して書き戻す。足した値が `<fails>` 以下なら終了コード `<code>`、超えたら 0 |
| `timing` | `start_ns=<Stopwatch.GetTimestamp から求めた、UTC のナノ秒>`、`end_ns=…` を、標準出力に書く(性能の測定用。`start_ns` は最初の行で、`end_ns` は終了の直前で取る。値は `DateTime.UtcNow.Ticks`(100 ns 単位)を 100 倍した数) |
| `burn <ms>` | 指定のミリ秒(既定 100)、`Process.GetCurrentProcess().TotalProcessorTime` で測って CPU を使い続けて終わる(CPU 時間の測定用) |
| `stdin` | 標準入力(`Console.OpenStandardInput()`)を終わりまで読み、読んだ内容を UTF-8 で復号して(不正なバイトは U+FFFD)記録 `stdin=Q(内容)` として出し、終了コード 0(標準入力の引き継ぎの確認。AC-509) |
| `big <MiB>` | 標準出力(`Console.OpenStandardOutput()`)に、指定の MiB(既定 16)の `x` を、書き込みの完了を待ちながら書き、終了コード 0(AC-494) |
| `tty` | `/dev/tty` を開こうとして、開けなければ終了コード 9、開ければ 0(AC-535。UNIX 系) |
| `readline` | 標準入力から 1 行を読み、終了コード 0(AC-535) |
| `inject-<名前>` | 注入用の実行ファイルのモード(3.6 節の「注入」)。製品の `Conductor.Cli` の入口(`Conductor.Cli.EntryPoint.Run(string[] args, Deps deps)`)を、失敗の仕掛け(例外を投げる偽の `Deps`、先に登録するシグナルのハンドラ、起動の前の遅延)をしてから呼ぶ。`TJ` は `Conductor.Cli` を参照する(プロジェクト参照。これで、製品に試験用の経路を持たせずに、実プロセスで内部の誤りを起こせる。`inject-*` 以外のモードは、`Conductor.Cli` の型に触れないので、起動の費用は増えない見込み。未確認) |

補足:

- `sleep` モードで SIGQUIT を受けたときの、.NET ランタイムの既定の動作(コアダンプを伴う終了かなど)は、**公式の文書で確認できていない(未確認)**。したがって、AC-282 の SIGQUIT の項目は、`conductor` の終了コード 131 だけを確かめ、ジョブの終わり方は確かめない。
- UNIX 系の受け入れテストでは、シグナルで終わるジョブを `["sh", "-c", "kill -9 $$"]`(配列形式)で作る(`TJ` にモードを足さない)。
- `TJ` の `env`・`cwd` の記録の値は、`Q(s)` で出すので、OS によらず同じ書式になる。`TJ` は、標準出力・標準エラーを、`Console.OpenStandardOutput()`・`Console.OpenStandardError()` のストリームに UTF-8(BOM なし)で書く(Windows のコンソールの符号化に左右されない)。

`tests/Conductor.TestJob/Program.cs` の骨組み(全モードの実装は、上の表のとおり。ここでは、共通部分と、`exit`・`flaky`・`sleep` の 3 つを示す):

```csharp
// Copyright 2026 agwlvssainokuni (Apache License 2.0 のヘッダ。全文は省略)
using System;
using System.Globalization;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;

namespace Conductor.TestJob;

internal static class Program
{
    private static string? _outFile;
    private static readonly UTF8Encoding Utf8 = new(encoderShouldEmitUTF8Identifier: false);

    private static string Q(string s)
    {
        string t = s.Replace("\\", "\\\\", StringComparison.Ordinal).Replace("\"", "\\\"", StringComparison.Ordinal)
            .Replace("\n", "\\n", StringComparison.Ordinal).Replace("\r", "\\r", StringComparison.Ordinal)
            .Replace("\t", "\\t", StringComparison.Ordinal);
        return "\"" + t + "\"";
    }

    private static void Record(string line)
    {
        if (_outFile is null)
        {
            using Stream o = Console.OpenStandardOutput();
            byte[] b = Utf8.GetBytes(line + "\n");
            o.Write(b, 0, b.Length);
            o.Flush();
        }
        else
        {
            File.AppendAllText(_outFile, line + "\n", Utf8);
        }
    }

    private static int ToInt(string? s, int fallback)
    {
        return int.TryParse(s, NumberStyles.AllowLeadingSign, CultureInfo.InvariantCulture, out int v) ? v : fallback;
    }

    private static int ModeFlaky(string[] args)
    {
        string path = args[0];
        int fails = ToInt(args[1], 0);
        int code = ToInt(args[2], 1);
        int n = 0;
        if (File.Exists(path))
        {
            n = ToInt(File.ReadAllText(path, Utf8).Trim(), 0);
        }
        n++;
        File.WriteAllText(path, n.ToString(CultureInfo.InvariantCulture), Utf8);
        return n <= fails ? code : 0;
    }

    [DllImport("libc", EntryPoint = "kill")]
    private static extern int Kill(int pid, int sig);

    private static int ModeSleep(string[] args)
    {
        Record("pid=" + Environment.ProcessId.ToString(CultureInfo.InvariantCulture));
        Record("ready");
        // 登録は、変数に保持する(GC で解除されないように。1 章)
        var regs = new PosixSignalRegistration?[3];
        PosixSignal[] sigs = { PosixSignal.SIGTERM, PosixSignal.SIGINT, PosixSignal.SIGHUP };
        if (!OperatingSystem.IsWindows())
        {
            for (int i = 0; i < sigs.Length; i++)
            {
                PosixSignal sig = sigs[i];
                int idx = i;
                regs[idx] = PosixSignalRegistration.Create(sig, ctx =>
                {
                    // 登録を解除して、同じシグナルを自分に送り直す(シグナルで終了したように見せる)。
                    // ctx.Signal の番号は、ctx.Signal が PosixSignal の値(負の値もある)なので、番号へ直す表を使う。
                    regs[idx]?.Dispose();
                    _ = Kill(Environment.ProcessId, SignalNumber(sig));
                });
            }
        }
        DateTime? deadline = args.Length > 0
            ? DateTime.UtcNow.AddSeconds(double.Parse(args[0], CultureInfo.InvariantCulture))
            : null;
        while (deadline is null || DateTime.UtcNow < deadline)
        {
            Thread.Sleep(50);
        }
        GC.KeepAlive(regs);
        return 99;
    }

    private static int SignalNumber(PosixSignal s) => s switch
    {
        PosixSignal.SIGHUP => 1,
        PosixSignal.SIGINT => 2,
        PosixSignal.SIGQUIT => 3,
        PosixSignal.SIGTERM => 15,
        _ => 0,
    };

    private static int Main(string[] argv)
    {
        int start = 0;
        if (argv.Length >= 2 && argv[0] == "--out")
        {
            _outFile = argv[1];
            start = 2;
        }
        if (argv.Length <= start)
        {
            return 2;
        }
        string mode = argv[start];
        string[] args = argv[(start + 1)..];
        switch (mode)
        {
            case "exit": return ToInt(args.Length > 0 ? args[0] : null, 2);
            case "flaky": return ModeFlaky(args);
            case "sleep": return ModeSleep(args);
            // run / echo / env / cwd / pid / say / trap / timing / burn / stdin / big / tty / readline / inject-* は、上の表のとおり
            default: return 2;
        }
    }
}
```

(`PosixSignal.SIGHUP` などの列挙子の存在と、負の値の列挙子が使えるかは、この文書では確認していない(未確認)。`libc` の `kill` を `DllImport` する書き方も、確認できていない。製品のコードは `LibraryImport` を使う(決定 14)。`SignalNumber` の表は、07 章の番号の対応に合わせる。)

### 3.5 定義とステートの略記と、実行環境

**実行環境**

- 被試験体(`conductor` の起動方法。3.3 節)と `TJ` を用意する。
- 各テストは、空の一時ディレクトリ(`T`)で行う。`T` の中に定義ファイル(既定の名前 `conductor.json`)を作り、`T` をカレントディレクトリにして被試験体を起動する。ステートファイルは、定義ファイルと同じディレクトリにできる(例: `T/conductor.json.state`)。
- 環境変数は、特に記さない限り、次のとおり。`LANG=C`。`LC_ALL`・`LC_MESSAGES`・`NO_COLOR`・`CONDUCTOR_RUN_ID`・`DOTNET_gcServer`・`DOTNET_SYSTEM_GLOBALIZATION_INVARIANT`・`DOTNET_CLI_UI_LANGUAGE`・`DOTNET_ROLL_FORWARD`(3.3 節の 2 の場合を除く)・`DOTNET_STARTUP_HOOKS`・`DOTNET_TieredPGO` は未設定にする(表示を英語に固定し、利用者の環境の影響を除く)。`PATH` と、`HOME`(自己展開の場合の展開先に使われうる。1 章)、`DOTNET_ROOT`(`TJ` と `dotnet conductor.dll` の起動に要る)と、Windows の `SystemRoot`・`ComSpec`・`PATHEXT`・`TEMP`・`TMP` は、親から引き継ぐ。
- 標準入力は空(`StandardInput.Close()`)にする。標準入力を使う項目は、書き込んでから閉じる。
- パスに空白や非 ASCII を含めない(特に記す場合を除く)。

**定義の略記** — 前提の「定義」は、次の略記で書く。

```text
P(<パイプライン名> [run_id_env=<ENV>] [remove_on_complete=<true|false>]; <ジョブ>, <ジョブ>, ...)
```

| 略記 | 展開 |
|------|------|
| `P(p; ...)` | `"format_version": 1`、`"pipeline": {"name": "p"}`、続いて `"job": [...]` の列(`{"id": ...}`)を、書いた順に並べる |
| `<id>:<n>` | `job` の要素に `"id": "<id>"`、`"command": "TJ exit <n>"` |
| `<id>:<n>{...}` | 同上に、波括弧の中の修飾を足す |
| `w=[…]` | `"warning_exit_codes": […]` |
| `s=[…]` | `"success_exit_codes": […]` |
| `on=success->X` / `on=warning->X` | `"transitions": [{"on": "success", "next": "X"}]`(X はジョブID か `@end`) |
| `retry=(exit_codes=[…], max=N, interval="…", multiplier=M, max_interval="…")` | `"retry": {…}` のオブジェクト(書いたキーだけ) |
| `cmd=<文字列>` | `command` を、その文字列にする(`TJ` は 3.4 節の語。必要なら二重引用符で囲む)。`id:n` の `n` は使わない |
| `argv=[…]` | `command` を、その文字列の列(JSON の配列)にする。`TJ` を書くときは、`tj` の絶対パスを 1 つの要素にする |
| `O` | 記録の出力ファイル(`T/out.txt` など。`--out` に渡す) |

例: `P(p; a:0, b:7{w=[3]}, c:0{on=success->@end})` は、次の定義(`TJ` は、3.4 節の語の略)。

```json
{
  "format_version": 1,
  "pipeline": {"name": "p"},
  "job": [
    {"id": "a", "command": "TJ exit 0"},
    {"id": "b", "command": "TJ exit 7", "warning_exit_codes": [3]},
    {"id": "c", "command": "TJ exit 0", "transitions": [{"on": "success", "next": "@end"}]}
  ]
}
```

YAML の定義(`conductor.yaml`)で試験する項目は、同じ構造を YAML で書く(流れ形式でも、ブロック形式でもよい。**`@end` は YAML では `"@end"` と二重引用符で囲む**。`@` で始まる値は、引用符なしでは書けない)。キー名と構造は、JSON と同じ([03](03-definition-format.md))。

**ステートの略記** — ステートの期待は、次の略記で書く。`end` と `completed` は、書かなければ `false`。`resume` は、書かなければ「キーがない」。

```text
S[<id>=<状態>(<終了コード>), ... ; resume=<id>, end=<true|false>, completed=<true|false>, run_id=<id>]
```

状態は `pending`・`success`・`warning`・`error`・`skipped`。スキップは `skipped/byTransition` か `skipped/byEnd`。終了コードがなければ括弧を省く。例: `S[a=success(0), b=error(7), c=pending ; resume=b]`。ステートファイルの書式(JSON)は [06](06-state-file.md)。受け入れテストは、ステートファイルを(テストでは)`System.Text.Json.JsonDocument` で読み、論理項目(`pipeline_name`、`job_order`、`job` の各要素の `id`・`status`・`skip_reason`・`exit_code`、`resume_job_id`、`end_reached`、`completed`、`run_id` など)を比べる。`System.Text.Json` を直接使ってよいのは、テストと開発用の検査プログラムだけである(製品のコードは、自前の読み取り `Conductor.Json` を使う。AC-421。03 章が、製品のコードで `Utf8JsonReader` の一部を使ってよいと定めた場合は、AC-421 の禁止の範囲を、その規定に合わせる。10.2 節)。

### 3.6 判定の書き方と区分

| 欄 | 意味 |
|----|------|
| 実行 | 被試験体の引数。`T` をカレントディレクトリにする。`--config` を省くと `T/conductor.json` |
| 終了 | 終了コード |
| OUT | 標準出力。「空」は 0 バイト。引用は完全一致。「含む」は部分一致 |
| ERR | 標準エラー。同上。複数行は `⏎` で区切って書く(行の終わりの改行は省く) |
| 前提 | 実行の前に作るもの(定義、ステート、環境変数) |

- どのテストも、**30 秒の上限**で実行する(ハングの検知)。
- 「ジョブが実行された」は、`TJ run <label> <n>` の記録 `ran=<label>` で確かめる(`O` を読む)。「実行されない」は、その記録がないこと。
- 「ステートが変わらない」は、ステートファイルの内容(バイト列)が、実行の前後で同じこと。「ステートがない」は、ファイルがないこと。
- 「UNIX 系」「Windows」「非 root」と書かれた項目は、その環境でだけ行う(3.10 節の属性)。

| 区分 | 意味 |
|------|------|
| (自動) | 被試験体を外から実行して確かめる |
| 単体 | 内部の関数(分割、終了コードの範囲合わせ、エスケープ、判断の純粋な関数)を直接呼んで確かめる。外から再現しにくい入力のため |
| 注入 | 標準出力への書き込みの失敗、シグナル登録の失敗など、外から作れない失敗を、偽の `Deps`(3.7 節)で起こして確かめる。外から作れる範囲では、**テスト専用の実行ファイル `TJ` の `inject-*` モード**(3.4 節)を起動し、そのモードが失敗の仕掛けをしてから `Conductor.Cli.EntryPoint.Run` を呼ぶ(製品に試験用の経路を持たせない。AC-490) |
| 点検 | ソースや依存を、ソースの文字列の走査(正規表現と、`Microsoft.CodeAnalysis.CSharp` を使わず、BCL だけでできる範囲。字句の走査)、プロジェクトファイル・`Directory.*.props` の XML の走査(`System.Xml.Linq`)、ビルドした成果物の中身(`System.IO.Compression`)で調べて確かめる(実行ではない。`tests/Conductor.Tests/Inspect/` に置く。検査の道具は、開発時の依存を増やさずに、BCL だけで書く) |
| 手動 | 人が実機で確かめて、記録を残す |
| 計測 | 性能の測定(2.1 節) |

### 3.7 偽の部品(`Deps`)を使うテスト

`Conductor.Cli` の `Deps` は、次の役割を **インターフェイス**で持つ([02](02-architecture.md))。`tests/Conductor.Tests/Support/Fakes.cs` が、それぞれの偽物を提供する(手書き。モックのライブラリは使わない)。

| 役割(`Deps` のプロパティ) | 本物 | 偽物(`Fakes.cs`。名前は 02 章 4.4 節) |
|-------------------------------|------|-------------------------------|
| 環境(`Env`) | `Environment.GetEnvironmentVariable`、`Console.IsOutputRedirected`、OS の判定(`PlatformInfo`) | `FakeEnv`(環境変数の表。端末か・OS の種別を設定できる) |
| 時刻(`Clock`) | `DateTimeOffset.UtcNow` など | `FakeClock`(固定の時刻) |
| 定義(`Defs`) | `Conductor.Definition` の読み込み、run ID の環境変数名の緩い読み取り | `FakeDefs`(決めた結果を返す。または一時ディレクトリの実ファイルを読む) |
| ステート(`States`) | `Conductor.State`(一時ファイルと置き換えによる原子的な書き込み) | `MemStates`(メモリ上のステート。読み込み・書き込み・削除の失敗を起こせる) |
| 変換(`Converter`) | `Conductor.Convert` の変換の入口 | `FakeConverter` |
| プロセス実行・待ち(`Proc`) | `Conductor.Runner`(ジョブの実行、リトライの待ち、`command` の検査) | `ScriptProcess`(用意した結果を順に返す。起動の回数と引数を記録。待ちは、待ち時間(秒の `ulong`)を記録するだけで、すぐに返す。中断を返す設定がある) |
| シグナル(`Signals`) | `PosixSignalRegistration`・`Console.CancelKeyPress` による登録と、旗の読み取り | `FakeSignals`(任意の時点で番号を立てる。準備の失敗・「登録済み」を返せる) |
| 標準出力・標準エラー(`Out`) | `Console.OpenStandardOutput()` と `Console.OpenStandardError()` から作った `Stream` | `BufSink`(文字列を溜める。「n 回目の書き込みで失敗する」設定で、書き込みが失敗する(`IOException` や `ObjectDisposedException` を投げる)場合を作る) |

偽の部品で確かめる項目(区分「注入」)は、AC-293〜300、AC-324、AC-366〜368、AC-485、AC-491 など。`Conductor.Cli` の流れのテストは、偽の部品の上で、`run`・`pipeline`・`watch`・`reset`・`convert`・`validate` を呼び、終了コード・標準出力・標準エラー・`MemStates` のステートを比べる。
本物の部品との結合は、受け入れテスト(3.3 節)が確かめる。

### 3.8 テストの補助(`tests/Conductor.AcceptanceTests/Support/Harness.cs`)

| 関数 | 役割 |
|------|------|
| `TempDir` | `T` を作る `IDisposable`(`Dispose` で消す。消すのに失敗しても、テストを失敗にしない) |
| `DefaultEnv(extra)` | 3.5 節の既定の環境変数に `extra` を足した辞書を返す |
| `WriteDefinition(T, name, source)` | 略記(3.5 節)の定義を、JSON か YAML で書き出す。文字列は JSON のエスケープで出す |
| `RunConductor(args, cwd, env, stdin, timeout = 30 s)` | 被試験体を `Process` で起動し、`Result(Code, Stdout, Stderr)`(`Code` は `ExitCode`。`Stdout`・`Stderr` は、バイト列を UTF-8 で復号した `string`。不正なバイトは U+FFFD。バイト列で比べる項目は、別に `StdoutBytes` を持つ)を返す。30 秒で強制終了して失敗にする。標準出力・標準エラーは、並行して読み切る |
| `StartConductor(args, ...)` | 実行中に操作する項目(シグナルを送る、標準エラーの行を待つ)のために、`Process` と、行ごとの待ち受け(`WaitForStderr(pattern, timeout)`。読み取りは、テスト側のスレッドを 1 つ使ってよい)を返す |
| `ReadRecords(file)` | `TJ` の記録の行を読む |
| `ReadState(T, name)` | ステートファイルを読んで、`JsonDocument` を返す |
| `ListDir(T)` | `T` のファイル名の一覧(`StringComparer.Ordinal` でソート済み) |
| `Signal(Process, int)` | UNIX 系で、シグナルを送る(テスト側の P/Invoke の `kill`。`Process.Kill` は強制終了なので使えない) |

### 3.9 xUnit と `dotnet test` の設定

`tests/Directory.Build.props`(または各テストのプロジェクト)と `xunit.runner.json`(キー名は採用した版の文書で確かめる。**この文書の作成時には実行していない**):

```xml
<!-- tests/Conductor.AcceptanceTests/Conductor.AcceptanceTests.csproj の抜粋 -->
<PropertyGroup>
  <IsPackable>false</IsPackable>
  <IsTestProject>true</IsTestProject>
</PropertyGroup>
<ItemGroup>
  <PackageReference Include="Microsoft.NET.Test.Sdk" />
  <PackageReference Include="xunit" />
  <PackageReference Include="xunit.runner.visualstudio" />
  <PackageReference Include="coverlet.collector" />
  <None Include="xunit.runner.json" CopyToOutputDirectory="PreserveNewest" />
</ItemGroup>
```

```json
{
  "$schema": "https://xunit.net/schema/current/xunit.runner.schema.json",
  "parallelizeTestCollections": false,
  "diagnosticMessages": false,
  "longRunningTestSeconds": 60
}
```

(`parallelizeTestCollections`・`longRunningTestSeconds` のキー名は、**xUnit の設定の文書では確認していない(1 章で確認したのは、`[assembly: CollectionBehavior(DisableTestParallelization = true)]` だけ)**。確実な方法として、アセンブリの属性を、受け入れテストのアセンブリに必ず書く。)

| 項目 | 決め | 理由 |
|------|------|------|
| 並列 | 受け入れテスト: 無効(`[assembly: CollectionBehavior(DisableTestParallelization = true)]`)。単体テスト: 既定(収集ごとに並列)。プロセス全体に作用するテストは `[Collection("process-global")]` | 時間の基準を安定させる。ステートは `T` が別なので衝突しない |
| `T`(一時ディレクトリ) | テストごとに別。`Path.GetTempPath()` の下に GUID。終了時に消す | ステートの衝突を避ける。CI の一時領域を汚さない |
| タイムアウト | xUnit v2 には、同期のテストの個別のタイムアウトがない。子プロセスを起動するテストは、`Harness.RunConductor` の 30 秒で自分で止める。テスト全体のハングは `--blame-hang-timeout` で検知 | 依存を増やさない |
| 条件付きのテスト | `[Fact(Skip = "…")]` は固定の理由。OS・権限による条件は、`FactAttribute` を継承した属性(`FactUnixAttribute`・`FactWindowsAttribute`・`FactLinuxAttribute`・`FactNonRootAttribute`・`PerfFactAttribute`)が、コンストラクタで条件を調べて `Skip` を設定する(3.10 節) | xUnit v2 の標準の方法 |
| 警告 | テストのプロジェクトも `TreatWarningsAsErrors`(xUnit のアナライザー(`xunit.analyzers`)の指摘を含む) | 5 章 |
| カバレッジ | `coverlet.collector`(`dotnet test --collect:"XPlat Code Coverage" --settings coverlet.runsettings`)。対象は `Conductor.Core`・`Conductor.Runtime`・`Conductor.Cli`。除外は `Program.cs`(入口。薄い。受け入れテストが通す)、テストのアセンブリ。**受け入れテストで起動した子プロセスの中のコードは、数えない**。下限は、行 80%(5.3 節) | 収集器はしきい値を持たない(1 章)。検査は `Conductor.Checks coverage` |
| スナップショット | 使わない(期待は、文言そのものをテストに書く) | 文言の変更に気づくため |
| 性能の試験 | `PerfFactAttribute` が、`CONDUCTOR_PERF` が `1` でなければ、`Skip` を設定する | 通常の実行を遅くしない |

`coverlet.runsettings`(キー名は、1 章で確認した範囲):

```xml
<?xml version="1.0" encoding="utf-8"?>
<RunSettings>
  <DataCollectionRunSettings>
    <DataCollectors>
      <DataCollector friendlyName="XPlat Code Coverage">
        <Configuration>
          <Format>cobertura</Format>
          <Exclude>[Conductor.Tests]*,[Conductor.AcceptanceTests]*,[Conductor.TestJob]*,[xunit.*]*</Exclude>
          <ExcludeByFile>**/Program.cs</ExcludeByFile>
          <SingleHit>false</SingleHit>
        </Configuration>
      </DataCollector>
    </DataCollectors>
  </DataCollectionRunSettings>
</RunSettings>
```

### 3.10 OS 別のテスト

- UNIX 系だけのテスト(シグナル、権限、シンボリックリンク、新しいセッション、疑似端末)は、`[FactUnix]`(`!OperatingSystem.IsWindows()` でないなら `Skip = "UNIX 系だけ"`)。Windows だけのテスト(`.cmd` の安全策、Ctrl+C のチェックリストに対応する自動の部分)は、`[FactWindows]`。Linux だけのテスト(`/dev/full`、`/proc` による CPU 時間)は、`[FactLinux]`(`OperatingSystem.IsLinux()`)。これらは `tests/Conductor.Tests/Support/Marks.cs`(受け入れテストのプロジェクトからは、リンクで共有)に定義して使う(OS の判定が、テストの中に散らばらないように)。
- 非 root のテスト(権限の拒否を起こす項目)は、`[FactNonRoot]`(UNIX 系で `geteuid() == 0` なら `Skip = "root では権限の拒否が起きない"`。`geteuid` は P/Invoke。Windows では常に実行)を加える。
- `Skip` でスキップされた項目は、結果に「Skipped」として残る。CI では、OS ごとにスキップされる件数が、想定の範囲であることを、実装時に確認する。
- OS 固有の製品のコードは、`PlatformInfo.cs` に集める。他のファイルから見ると、OS の違いは見えない(AC-420)。
- どの OS でも、`dotnet build -warnaserror` と `dotnet format --verify-no-changes` が通る(`OperatingSystem.IsWindows()` などの、プラットフォームの互換性のアナライザー(`CA1416`)は、`[SupportedOSPlatform]`・ガードで満たす。Windows 専用の API は使わないので、見込みは少ない。3 OS の CI で通ることを確かめる。実装時に確認)。

### 3.11 テストの実行

```sh
dotnet restore --locked-mode                          # ロックファイルのとおりに入れる
dotnet build -c Release --no-restore -warnaserror
dotnet test -c Release --no-build                     # 単体 + 受け入れ + 点検(3 OS で通る)
dotnet test -c Release --no-build --collect:"XPlat Code Coverage" --settings coverlet.runsettings   # 行 80%(Linux)
CONDUCTOR_PERF=1 dotnet test -c Release --no-build --filter "FullyQualifiedName~Perf"             # 性能の測定(2.1 節)
CONDUCTOR_BIN=<配布物の単一ファイル> dotnet test -c Release --no-build tests/Conductor.AcceptanceTests   # 配布物の試験(AC-446)
CONDUCTOR_DLL=<フレームワーク依存の conductor.dll> DOTNET_ROLL_FORWARD=Major dotnet test …           # 別のランタイムでの試験
```

`Directory.Packages.props` に、テストと検査の道具が入っている(8.3 節)。

---

## 4. 受け入れテスト一覧

ID は、`AC-` と通し番号。表は、ID・機能・前提・実行・期待の順。期待の標準エラーの文言は、特に記さない限り `LANG=C`(英語)。理由の文(日本語の固定の文)は、英語の表示でも日本語のまま。
JSON と YAML に固有の項目は 4.19 節(AC-450〜487)、.NET に固有の項目は 4.20 節(AC-488 以降)にまとめた。この節の定義は、特に記さない限り JSON の `conductor.json`。

### 4.1 定義の検査(`validate`、`run`、`pipeline` 共通)

特に記さない限り、実行は `validate`(`--config` を省く)。「64」の期待は、終了 64、OUT 空、ステートファイルを作らない、ジョブを起動しない、を含む。
ERR の期待は、**最初の行**のうち、`conductor: error: <ファイル>[:<行>]: ` に続く種類の文([08](08-output-and-i18n.md))を示す。ファイル名は、`--config` に渡した表記のまま。行は、誤りの位置の行(1 始まり。LF の数で数える)。YAML の誤りの行は、採用した YAML ライブラリが返す位置(多くは 0 始まり)を 1 始まりに直した値で、ライブラリの検出位置に依存する項目は、10 章に挙げる。
基本の定義 `B0` は、`P(p; a:0)` とする。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-001 | 正しい定義 | `B0` | `validate` | 終了 0。OUT・ERR 空。`T/conductor.json.state` を作らない |
| AC-002 | validate の -v(1 ジョブ) | `B0` | `validate -v` | 終了 0。ERR = `[conductor] definition is valid (1 job)`。OUT 空 |
| AC-003 | validate の -v(複数) | `P(p; a:0, b:0, c:0)` | `validate -v` | 終了 0。ERR = `[conductor] definition is valid (3 jobs)` |
| AC-004 | validate の -v(日本語) | `P(p; a:0, b:0, c:0)`、`LANG=ja_JP.UTF-8` | `validate -v` | 終了 0。ERR = `[conductor] 定義は正しいです(ジョブ 3 個)` |
| AC-005 | 定義ファイルがない | ファイルなし | `validate --config missing.json` | 64。ERR = `conductor: error: missing.json: definition file does not exist: 指定の場所にファイルがありません` |
| AC-006 | 定義ファイルがない(日本語) | 同上、`LANG=ja_JP.UTF-8` | 同上 | 64。ERR = `conductor: error: missing.json: 定義ファイルが存在しません: 指定の場所にファイルがありません` |
| AC-007 | 通常のファイルでない | `T/d.json` がディレクトリ | `validate --config d.json` | 64。ERR の種類の文 = `cannot read definition file`(理由 `通常のファイルではありません`) |
| AC-008 | 空のファイル | (a) 長さ 0 の `conductor.json`。(b) 長さ 0 の `conductor.yaml`(コメントだけのファイルも同じ) | (a) `validate`。(b) `validate --config conductor.yaml` | 64。(a) JSON では空の入力は文法の誤り(`syntax error`。行は 03 章に従う)。(b) YAML では `conductor.yaml: format_version is missing: …`(行なし) |
| AC-009 | 文法の誤り(行つき) | JSON: `{`⏎`"format_version": 1,`⏎`"pipeline": ]`⏎`}`。YAML(`conductor.yaml`): `format_version: 1`⏎`pipeline: ]`⏎ | `validate`(YAML は `--config conductor.yaml`) | 64。ERR の最初の行が、JSON は `conductor: error: conductor.json:3: syntax error: `、YAML は `conductor: error: conductor.yaml:2: syntax error: ` で始まる |
| AC-010 | キーの重複(文法の誤り) | JSON: `{"format_version": 1,`⏎`"format_version": 1}`。YAML: `format_version: 1⏎format_version: 1⏎` | `validate` | 64。種類の文 = `syntax error`(行 2) |
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
| AC-031 | 終了コードの型・範囲の誤り | `w=["x"]`、`w=[1.5]`、`w=[true]`(真偽値は整数ではない)、`w=[1.0]`(`1.0` は整数ではない)、`w=[99999999999999999999]`、`w=[2147483648]`、`w=[-2147483649]` | `validate` | 64。`wrong value type`(要素ごとに 1 行)。範囲の検査は、整数の字句から行う(`double` に落とさない。`double` に変換すると、`2147483648` 付近以外で別の整数に化ける値の見落としが起きる。AC-502) |
| AC-032 | 負の終了コード・空の成功コード・範囲の端 | `w=[-1]`、`s=[]`、`w=[2147483647]`、`s=[-2147483648]` | `validate` | 0(受理する) |
| AC-033 | `command` の型・欠落・空 | `command` なし / `""` / `"   "` / `[]` / `[""]` / `["  ", "x"]` / `5` / `["a", 5]` / `true` | `validate` | 64。なし・`5`・`["a",5]`・`true` は `wrong value type`。ほかは `required value is empty` |
| AC-034 | 配列形式の `command` の受理 | `argv=["my prog", "a b", "$HOME", ""]`、`argv=["echo", ""]` | `validate` | 0 |
| AC-035 | `run_id_env` の名前の形 | `run_id_env` が `"1A"` / `"A-B"` / `""` / `"A B"` / `"A=B"` / `"A.B"` | `validate` | 64。`invalid environment variable name`。整数 `1` なら `wrong value type`。`_a1`・`GITHUB_RUN_ID`・`A` は 0 |
| AC-036 | `remove_on_complete` の型 | `"true"` / `1` / `[true]`(YAML の定義では、`yes` も。YAML 1.2 のコアスキーマでは文字列) | `validate` | 64。`wrong value type`。`true`・`false`・省略は 0 |
| AC-037 | 誤りは位置の順に 1 行ずつ | `ME`(この節の末尾の定義。JSON) | `validate` | 64。ERR は 4 行。順に `…:3: unknown key`、`…:11: wrong value type`、`…:16: unknown key`、`…:20: transition target job is not defined`。YAML で書いた `ME` では、行は 2、10、13、16(AC-475) |
| AC-038 | 行番号は CRLF でも同じ | `ME` を CRLF 改行で | `validate` | 同上(行番号が同じ。LF の数で数える) |
| AC-039 | BOM つきの定義 | `B0` の先頭に UTF-8 の BOM(`EF BB BF`)。.NET の `File.ReadAllText`・`Encoding.UTF8.GetString` は、既定で BOM の扱いが異なる(前者は BOM を取り除くが、後者は U+FEFF を残す)ので、バイト列から自前で取り除く | `validate` | 0 |
| AC-040 | 定義の大きさの上限(受理) | `B0` に空白を足して、ちょうど 1,048,576 バイト | `validate` | 0 |
| AC-041 | 定義の大きさの上限(超過) | 1,048,577 バイト(UTF-8 でなくてもよい) | `validate` | 64。`definition file is too large`(読み込む前に、`FileInfo.Length` でサイズを見る。全体をメモリに読んでから比べない) |
| AC-042 | UTF-8 でない定義 | 上限以内で、不正なバイト列を含む | `validate` | 64。`cannot read definition file`(復号は `new UTF8Encoding(false, throwOnInvalidBytes: true)` で行い、不正なバイト列を U+FFFD に置換せず拒否する。`Encoding.UTF8.GetString`・`File.ReadAllText` の既定は、不正なバイトを U+FFFD にするので使わない) |
| AC-043 | 入れ子の深さ(受理) | JSON: `{"a": ` に `[` を 32 個、`]` を 32 個、`}`(`format_version` なし。根のオブジェクトは数えず、32 段まで受理。[03](03-definition-format.md))。YAML: `a: ` に `[` を 32 個、`]` を 32 個 | `validate` | 64。`format_version is missing`(深さの検査は通った) |
| AC-044 | 入れ子の深さ(超過) | 同じ形で `[` を 33 個・1,000 個・100,000 個(閉じも書く) | `validate` | 64。`syntax error`(理由 `入れ子が深すぎます(上限は 32 段)`。行は 33 段目の `[` の行)。`StackOverflowException` でプロセスが落ちない(AC-524) |
| AC-045 | パイプライン名の照合 | `P(p; a:0)` | `validate p` / `validate wrong` / `validate` | 0 / 64(`pipeline name does not match the definition`)/ 0(照合しない) |
| AC-046 | 名前の不一致の誤りに値を含めない | `"name": "SECRET-NAME"` | `validate other-name` | 64。ERR に `SECRET-NAME` も `other-name` も現れない |
| AC-047 | 誤りの文に入力の値を含めない | `format_version: 1`、`pipeline` の `name: p`、`job` の `id: a`、`"command": 5`、`"warning_exit_codes": ["SENTINEL-VALUE"]` | `validate` | 64。ERR に `SENTINEL-VALUE` が現れない |
| AC-048 | `command` の引用符が閉じていない | `P(p; a:0{cmd="TJ exit 0 'x"})` | `validate` | 64。ERR = `conductor: error: conductor.json: job 'a': command has an unclosed quote` |
| AC-049 | `command` が空になる | `cmd="''"` | `validate` | 64。`job 'a': command is empty` |
| AC-050 | `command` の誤りは全ジョブ報告 | 2 つのジョブがそれぞれ誤り | `validate` | 64。ERR は 2 行(ジョブごと) |
| AC-051 | `command` の誤り(日本語) | 同上(1 つ目の形)、`LANG=ja_JP.UTF-8` | `validate` | ERR = `conductor: error: conductor.json: job 'a': command の引用符が閉じていません` |
| AC-052 | `run` でも `command` の誤りは 64 でジョブは起動しない | 1 つ目が正しく、2 つ目が誤り | `run a` | 64。`ran=` の記録なし。ステートなし |
| AC-053 | 起動できないプログラムでも validate は 0 | `cmd="/no/such/program"` | `validate` | 0 |
| AC-054 | 壊れたステートでも validate は 0 | `B0`、`T/conductor.json.state` に `{garbage` | `validate p` | 0。ステートは変わらない |
| AC-055 | validate の引数の誤り | `B0` | `validate a b` / `validate --run-id x` / `validate --to yaml` | 68 |
| AC-056 | `run_id_env` が不正な定義は run でも 64 | `"run_id_env": "1A"` | `run a` | 64 |
| AC-057 | validate は副作用を持たない | `B0` の `command` が「`TJ run a 0`」(`O` に記録) | `validate` | 0。`O` が作られない。`T` のファイル一覧が実行の前後で同じ |

#### `retry` の検査

`R0` は、`P(p; a:75{retry=(exit_codes=[75], max=3, interval="1s")})`。各行は、`R0` の `retry` を変えて `validate` する。誤りは終了 64、種類の文は ERR の最初の行。

| ID | 機能 | 変更後の `retry` | 期待 |
|----|------|------------------|------|
| AC-058 | 受理(固定の間隔) | `R0` のまま | 0 |
| AC-059 | 受理(倍率と上限) | `"multiplier": 2`、`"max_interval": "1m"` を足す | 0 |
| AC-060 | 受理(境界) | `exit_codes: [1,255,127]`、`max: 1` と `max: 100`、`interval` が `1s`・`24h`・`86400s`・`1440m`、`multiplier` が 2 と 10、`max_interval` が `interval` と同じ | いずれも 0 |
| AC-061 | 受理(YAML の流れ形式の写像) | YAML の `retry: { exit_codes: [75], max: 3, interval: 1s }` | 0 |
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

`ME`(誤りが複数ある定義。JSON。行番号は、この並びのとおり。YAML の `ME` は、4.19 節の AC-475 の後ろ)。

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

YAML で同じ構造を書いた `ME`(`conductor.yaml`。行番号は、この並びのとおり):

```yaml
format_version: 1
unknown_top: true

pipeline:
  name: p

job:
  - id: a
    command: x
    success_exit_codes: ["zero"]
  - id: b
    command: y
    colour: red
    transitions:
      - on: success
        next: nowhere
```

JSON の `ME` の誤りの行は、順に 3、11、16、20。YAML の `ME` の誤りの行は、順に 2、10、13、16。

### 4.2 `command` の分割と起動

「分割」の行は、`P(p; a:0{cmd="TJ --out O echo <X>"})` を作り、`run a` を実行する(`TJ`、`O` は 3.5 節)。`<X>` は、各行の「`command` の引数部」。
期待は、終了 0、OUT・ERR 空、`O` の記録(`arg[0]=…` 以降)。記録は `Q(…)` の形(3.4 節)。ここでは、引数の値だけを書く(`arg[i]` の添字は先頭から 0)。定義に書くときは、JSON の文字列で書き、文字列として同じ値になるようにする。分割は、`string.Split` や、`CommandLineToArgvW` 相当の既存の関数に頼らず、自前で行う([05](05-execution-flow.md)、[07](07-process-and-signals.md)。これらの規則とは、バックスラッシュの扱いなどが違う)。分割の走査は、UTF-16 の `char` でなく、区切り・引用符の判定に必要な ASCII の 6 文字だけを比べる(サロゲートペアを分割しない。決定 10)。

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
| AC-089 | 分割: 区切りは 4 文字だけ | `a<U+00A0>b`(U+00A0 は空白に見えるが区切りではない。`string.Split()` の引数なしの形・`char.IsWhiteSpace` は U+00A0 でも分けるので、これらで分割しない) | `a<U+00A0>b`(1 つ) |
| AC-090 | 配列形式: 要素はそのまま | `argv=[<TJ の要素>, "--out", "O", "echo", "a b", "  lead and trail  ", "$HOME", "a;b", "*", ""]` | `a b`、`  lead and trail  `、`$HOME`、`a;b`、`*`、(空) |
| AC-091 | 文字列と配列は同じ結果 | `P(p; a:0{cmd="TJ --out O run a 0"}, b:0{cmd="TJ --out O run b 7"})` と、同じ内容の `argv` 形式 | それぞれ `run a` → 0、`pipeline p` → 7、OUT・ERR 空。`O` = `ran=a`⏎`ran=b` が同じ |

分割の誤り(拒否)。実行は `validate`。終了 64。ERR の最初の行は `conductor: error: conductor.json: job 'a': <文>`。

| ID | 機能 | `command`(文字列) | 期待する文 |
|----|------|---------------------|------------|
| AC-092 | 拒否: 一重引用符が閉じていない | `echo 'a` | `command has an unclosed quote` |
| AC-093 | 拒否: 二重引用符が閉じていない | `echo "a` | 同上 |
| AC-094 | 拒否: 末尾が二重引用符の中の `\` | `echo "a\` | 同上 |
| AC-095 | 拒否: `\\"` で閉じない | `echo "a\\" b` | 同上 |
| AC-096 | 拒否: パスの末尾の `\"` | `prog "C:\dir\"` | 同上 |
| AC-097 | 拒否: 空の語だけ | `''`、`'' x` | `command is empty` |
| AC-098 | 拒否: 区切りだけ | `"  \t\n"`(JSON の文字列)。定義の検査が、空白だけの `command` を先に拒否する | `required value is empty`(定義の検査。64) |

起動と実行環境。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-099 | 環境変数の引き継ぎ | `P(p; a:0{cmd="TJ --out O env CT_VAR CT_NO_SUCH"})`。環境に `CT_VAR=hello`、`CT_NO_SUCH` は未設定 | `run a` | 終了 0。`O` = `env[CT_VAR]=set:"hello"`⏎`env[CT_NO_SUCH]=unset` |
| AC-100 | 作業ディレクトリは引き継ぐ | `cmd="TJ --out O cwd"`。定義は `T/sub/x.json` | `T` で `run a --config sub/x.json` | `O` = `cwd=Q(T のパス)`(`T/sub` ではない) |
| AC-101 | run ID を環境変数として渡さない | `cmd="TJ --out O env CONDUCTOR_RUN_ID"` | `run a --run-id r1` | `O` = `env[CONDUCTOR_RUN_ID]=unset` |
| AC-102 | 標準出力・標準エラーはそのまま | `cmd="TJ say 'OUT to stdout' 'ERR to stderr' 0"` | `run a` | 終了 0。OUT = `OUT to stdout`(改行なし、完全一致)。ERR = `ERR to stderr`(完全一致) |
| AC-103 | 存在しないプログラム | `cmd="/no/such/program-for-conductor-test"` | `run a` | 終了 127。ERR に `conductor: error: job 'a': cannot start command '/no/such/program-for-conductor-test'`。OUT 空。ステート `S[a=error ; resume=a]`(`Process.Start` が投げる `Win32Exception`(`NativeErrorCode` は ENOENT 相当)を、起動失敗として扱う。例外のメッセージは表示に出さない) |
| AC-104 | 配列形式の起動失敗は先頭の要素の名前 | `argv=["no-such-prog-zzz", "--flag"]` | `run a` | 127。ERR に `cannot start command 'no-such-prog-zzz'` |
| AC-105 | 空白を含む先頭の要素 | `argv=["my prog", "x"]`(`my prog` というファイルはない) | `run a` | 127。`cannot start command 'my prog'` |
| AC-106 | 実行できないファイル(UNIX 系) | `cmd` が実行権限のない通常のファイル(`T/data.txt`)のパス | `run a` | 127(`Win32Exception`) |
| AC-107 | ディレクトリ・`/` を指定(UNIX 系) | `cmd="/"`、`cmd` がディレクトリのパス | `run a` | 127(`Win32Exception` など、`Process.Start` が投げる例外の各種) |
| AC-108 | 起動失敗は表示の言語に従う | `LANG=ja_JP.UTF-8`、存在しないプログラム | `run a` | 127。ERR に `コマンドを起動できません` |
| AC-109 | 終了コードの返り方 | `exit 0`、`exit 3`、`exit 255` のジョブ(警告の指定なし) | `run a` | 0 / 3 / 255。OUT・ERR 空 |
| AC-110 | 大きな終了コード | `TJ exit 300` | `run a` | 44(`300 mod 256`。UNIX 系では、子の `exit` の値が OS により 44 になる) |
| AC-111 | 256 の終了コード(Windows) | `TJ exit 256`、`-v` | `run -v a` | 1。ERR に `exit code 256, conductor exits with 1` |
| AC-112 | 負の終了コード(Windows) | `TJ exit -1`、`-v` | `run -v a` | 255。ERR に `conductor exits with 255`(Windows で `Process.ExitCode` が `-1` で返るか `4294967295` を `int` に直した値で返るかは未確認。どちらでも、32 ビット符号つきに直してから範囲合わせをして、255 になる。AC-392) |
| AC-113 | `.cmd` への危険な引数(文字列形式。Windows) | `T/echo-args.cmd`(内容 `@echo off` と `exit /b 0`)。`cmd` = そのパス + ` 'x" & echo injected > T\injected.txt'` | `run a` | 127(禁止文字を含む引数なので、起動を拒否する。[07](07-process-and-signals.md))。`T/injected.txt` ができない |
| AC-114 | `.cmd` への危険な引数(配列形式。Windows) | 同じスクリプト。`argv=[パス, "x\" & echo injected > T\injected.txt"]` | `run a` | 同上 |
| AC-115 | `.cmd` への安全な引数(Windows) | `T/ok.cmd`(`exit /b 7`)。`cmd` = そのパス + ` plain` | `run a` | 7 |
| AC-116 | シェルを使わないことの点検 | ソース | (文字列の走査) | `System.Diagnostics.Process` と `ProcessStartInfo` の使用が `src/Conductor.Runtime/Runner/` にだけあり、`Process.Start(string)`・`Process.Start(string, string)` を使わない(`BannedSymbols.txt` で禁止。AC-518)。`UseShellExecute` は、リテラル `false` だけ。`ProcessStartInfo` の `FileName` に、シェルの名前(`sh`・`bash`・`powershell`・`cmd`)を渡す経路がない。`Arguments` ではなく `ArgumentList` を使う。**例外は、Windows の `.cmd` / `.bat` を起動する経路(07 章の決定: `cmd.exe` の直接起動)の、その 1 か所だけである。この経路は、禁止文字(`"`、`%`、`^`、`&`、`\|`、`<`、`>`、`!`、`(`、`)`、改行、復帰、NUL)を含む引数を拒否したあとでだけ使い、`PlatformInfo.cs` の 1 か所に集める(AC-486、AC-510、AC-513)**。区分: 点検 |

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
| AC-128 | 既定の定義ファイル | `T/conductor.json` | `run a`(`--config` なし) | 定義を読む。ステートは `T/conductor.json.state` |
| AC-129 | `--config` の場所 | `T/sub/x.json` | `run a --config sub/x.json` | ステートは `T/sub/x.json.state` |
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
| AC-140 | シグナルで終了したジョブ(UNIX 系) | `argv=["sh", "-c", "kill -9 $$"]` | `run -v a` | 137。ERR = `[conductor] job=a result=error (terminated by signal 9)`。ステート `a=error`(終了コードなし)、`resume=a`(.NET が `Process.ExitCode` として返す値を、07 章の規則で番号 9 に直す。UNIX 系でシグナル終了が `128 + 9` で返ると記憶しているが、公式の文書では確認できていない(未確認)。この項目が実機での確認を兼ねる。AC-504) |
| AC-141 | 完了のステートが残っていても先頭から | `P(p; a:0, b:0)`。`run a`、`run b`(完了)のあと | `run a`(`TJ run` で記録) | 0。`a` が再び実行される。`completed` のステートは上書きされる |
| AC-142 | 完了のあと先頭以外は拒否 | 上の完了のステート | `run b` | 66 |
| AC-143 | `--remove-on-complete`(run) | `P(p; a:0)` | `run a --remove-on-complete` | 0。OUT・ERR 空。`T/conductor.json.state` がない |
| AC-144 | 定義の `remove_on_complete` | `P(p remove_on_complete=true; a:0)` | `run a` | 0。ステートがない |
| AC-145 | 完了でなければ消さない | `P(p; a:0, b:0)` | `run a --remove-on-complete` | 0。ステートが残る(`a=success`、`completed=false`) |
| AC-146 | ステートの形(エラーで停止) | 定義 `c2.json`(`P(n2; a:0, b:3, c:0)`) | `pipeline n2 --config c2.json` | 3。`T/c2.json.state` の内容は JSON のオブジェクトで、`completed=false`、`definition_key="c2.json"`、`definition_path="c2.json"`、`end_reached=false`、`format_version=1`、`job_order=["a","b","c"]`、`pipeline_name="n2"`、`resume_job_id="b"`、`job` は `a=success exit_code=0`、`b=error exit_code=3`、`c=pending`(`run_id` キーなし) |
| AC-147 | ステートの書式 | 上のステート | (ファイルを読む) | 1 つの JSON のオブジェクト(テストが `JsonDocument` で読める)。末尾は改行。インデントと、キーの並び(固定の順)は [06](06-state-file.md) のとおり。同じ内容のステートを書くと、毎回同じバイト列になる(`Dictionary` の列挙順に頼らず、キーを出力する順を、書き出しの部品の中で固定する) |
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
| AC-177 | 定義ファイルを読まない | 定義ファイルがない、または壊れている。ステートあり | `watch --config missing.json` | 0。ステートが表示される(定義がなくても動く) |
| AC-178 | ステートなし・定義なしでも 0 | 定義ファイルなし | `watch --config missing.json` | 0。OUT に `no state` |
| AC-179 | 壊れたステート | `{garbage` | `watch` | 65。ERR = ``conductor: error: conductor.json.state: state file is corrupt: JSON として読めません; run `conductor reset` to discard the state``(`` ` `` はバッククォート)。OUT 空 |
| AC-180 | 壊れた完了のステート | 完了のステートだが `run_id` がファイル名と食い違う | `watch` | 65 |
| AC-181 | ファイルへの出力に制御文字がない | `P(p; a:0, b:0)`、`run a` のあと | `watch`(標準出力をファイルに) | 0。ファイルに `\x1b` がなく、`a`、`success`、`pending` を含む |
| AC-182 | 端末では状態に色(UNIX 系。疑似端末) | 同上 | 標準出力を疑似端末(テストの補助 `PtyHelper` が、P/Invoke の `openpty` で作った子側。`Console.IsOutputRedirected` が `false`)にして `watch` | `success` が `\x1b[32m` と `\x1b[0m` で囲まれる(`warning` 33、`error` 31、`skipped` 36、`pending` は色なし)。列の幅は色の制御文字を除いた文字で決まる。区分: UNIX 系(疑似端末)。`openpty` の P/Invoke(ライブラリ名は Linux の glibc の版で `libc` か `libutil`)は、この文書の作成時には確かめていない(未確認。10 章)。併せて、端末かどうかの判定を `Deps` の `Env` にして、偽の `Deps` で「端末」を返す単体の試験を置く(区分: 注入) |
| AC-183 | `NO_COLOR` で色なし | 同上、`NO_COLOR=1` | 同上 | 制御文字なし。空の `NO_COLOR` では色あり。区分: UNIX 系(疑似端末か、注入) |
| AC-184 | Windows は色なし | Windows | `watch`(端末) | 制御文字なし。区分: Windows |
| AC-185 | ジョブIDの制御文字は表の中でもエスケープ | ジョブID に ESC を含む(JSON で `"a\u001bb"`)。実行済み | `watch` | 行に `a\u{1b}b`。生の ESC がない |
| AC-186 | 標準出力に書けない(注入) | ステートあり | `watch`(標準出力の `BufSink` を書き込み失敗にする) | 65。ERR = `conductor: error: cannot write to standard output`。Linux では `watch > /dev/full` でも起こせる(AC-495)。パイプの読み手が閉じた場合(`IOException`)は AC-493、AC-516 |
| AC-187 | 使えないオプション | なし | `watch -v` / `watch --remove-on-complete` / `watch --all` / `watch extra` | 68。ERR = `option not allowed for this command: '-v'`(それぞれ対象は `-v`、`--remove-on-complete`、`--all`)。`watch extra` は `unexpected argument: 'extra'` |

### 4.6 `reset`

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-188 | ステートを消す | `P(p; a:0, b:7)`、`pipeline p` のあと | `reset` | 0。OUT・ERR 空。ステートがない。続く `watch` → `no state …` |
| AC-189 | 先頭から実行できる | 上のあと、`P(p; a:0, b:0)` に直す | `run a`(`TJ run` で記録) | 0。`a` が実行される |
| AC-190 | ステートがなくても成功 | ステートなし | `reset` | 0。OUT・ERR 空 |
| AC-191 | 定義がなくても動く | 定義なし | `reset --config missing.json` | 0 |
| AC-192 | 壊れたステート・壊れた定義を迂回 | 壊れたステートと壊れた定義 | `reset` | 0。ステートが消える |
| AC-193 | 完了のステートも消す | 完了のステート(run ID あり・なし) | `reset`、`reset --run-id r1` | 0。消える |
| AC-194 | `--run-id` は 1 つだけ消す | `conductor.json.state`、`.r1.state`、`.r2.state` がある | `reset --run-id r1` | `.r1.state` だけ消える。ほかは変わらない |
| AC-195 | `--all` で全部消す | `P(p; a:0, b:7)`。`pipeline p`(run ID なし)、`--run-id a`、`--run-id b` で実行。`T` に、別の定義のステート(`conductor.json.a.b.state`。中身はパイプライン名 `p`・ジョブ `a` の正常なステートで、`run_id` を持たない)、壊れたもの(`conductor.json.c.state` = `{garbage`)、一時ファイル(`.conductor.json.state.tmp.1.2.3`)、無関係なファイル(`conductor.json.bak`、`other.json.state`)を置く | `reset --all` | 終了 0。OUT 空。ERR は `conductor: warning: ` で始まり、`conductor.json.c.state` を含み、`conductor.json.a.b.state` を含まない。残るファイル: `.conductor.json.state.tmp.1.2.3`、`conductor.json`、`conductor.json.a.b.state`、`conductor.json.bak`、`conductor.json.c.state`、`other.json.state`。run ID なしと `.a.state`、`.b.state` は消える |
| AC-196 | 警告の文言 | 上と同じ | `reset --all` | ERR = ``conductor: warning: kept state files that could not be read (remove one with `conductor reset --run-id <ID>`):``⏎`  conductor.json.c.state`(ファイル名の行は先頭に空白 2 文字) |
| AC-197 | 壊れたものを個別に消す | 上のあと | `reset --run-id c` | 0。`conductor.json.c.state` が消える |
| AC-198 | 何もなくても 0 で出力なし | `T` に定義だけ | `reset --all` | 0。OUT・ERR 空 |
| AC-199 | `--all` と `--run-id` の併用 | なし | `reset --all --run-id x` | 68。`invalid value: '--all'` |
| AC-200 | `--all` は環境変数を見ない | `CONDUCTOR_RUN_ID='bad id'` | `reset --all` | 0 |
| AC-201 | シンボリックリンクと別の定義は消さない(UNIX 系) | `conductor.json.s.state` が通常のファイルへのシンボリックリンク。ディレクトリ `conductor.json.d.state`。`conductor.json.a.state` の中身の `run_id` がファイル名と食い違う | `reset --all` | リンク先のファイルは残る。リンクとディレクトリと、中身の `run_id` が食い違うファイルは消されない(`FileSystemInfo.Attributes` の `ReparsePoint` か `LinkTarget` で種類を調べ、リンクをたどらない。.NET の API の挙動の細部は未確認。実装時に確認) |
| AC-202 | run ID なしのステートは中身を読まず消す | `conductor.json.state` が壊れている(通常のファイル) | `reset --all` | 消える。警告なし |
| AC-203 | 環境変数の値の違反 | `CONDUCTOR_RUN_ID='bad id'` | `reset` | 68。`invalid value: 'run ID (environment variable)'` |
| AC-204 | 引数の誤り | なし | `reset extra`、`reset -v`、`reset --remove-on-complete` | 68。`unexpected argument: 'extra'`、`option not allowed for this command: '-v'`、同 `'--remove-on-complete'` |
| AC-205 | 削除の失敗(UNIX 系、非 root) | `T/conductor.json.state` が空でないディレクトリ | `reset` | 65。ERR に `cannot remove state file`。OUT 空 |

### 4.7 `convert`(JSON と YAML の相互変換)

`convert <入力ファイル> --to json|yaml`。入力の形式は `--to` の反対(`--to yaml` なら JSON、`--to json` なら YAML)で、入力ファイルの拡張子は見ない([09](09-converter.md))。

変換の入力 `CV1`(JSON)、`CV1Y`(`CV1` を YAML にしたもの)、`CV2`(YAML。アンカー、エイリアス、マージキーを含む)は、次のとおり。

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

```yaml
# CV1Y(CV1 の YAML 版。コメントは変換で失われる)
format_version: 1
pipeline:
  name: nightly
  description: 夜間バッチ(抽出、変換、読み込み、通知)
job:
  - id: extract
    command: extract --all --since yesterday
  - id: transform
    command: 'transform "in dir" $HOME'
    warning_exit_codes: [3, 4]
    success_exit_codes: [0, 2]
    transitions:
      - on: warning
        next: notify
  - id: load
    command: load
    transitions:
      - on: success
        next: "@end"
  - id: notify
    command: notify --channel ops
```

```yaml
# CV2(アンカー、エイリアス、マージキーを含む)
format_version: 1
pipeline:
  name: nightly
  description: 夜間バッチ(抽出、変換、読み込み、通知)
defaults: &defaults
  success_exit_codes: [0, 2]
  warning_exit_codes: [3, 4]
job:
  - id: extract
    command: extract --all --since yesterday
    <<: *defaults
  - id: transform
    command: 'transform "in dir" $HOME'
    <<: *defaults
    success_exit_codes: [0]
    transitions:
      - on: warning
        next: notify
      - on: error
        next: "@end"
  - id: load
    command: load
    transitions:
      - { on: success, next: "@end" }
  - id: notify
    command: notify --channel ops
```

特に記さない限り、失敗の期待は、終了 67、OUT 空、ERR の最初の行が `conductor: error: ` で始まる。「木」は、テストが `System.Text.Json.JsonDocument` と、YAML の読み取り(製品の読み取りの部品、またはテスト専用に用意した小さな読み取り)で得る、オブジェクト・配列・文字列・整数・浮動小数点数・真偽値・null の構造。`JsonDocument` は、整数と浮動小数点数を区別する(`TryGetInt64` の成否と、字句の `GetRawText()` の小数点・指数の有無で判定する)。ただし、**重複キーの扱いと、`NaN`・`Infinity` の扱いについて、公式の文書で確認できていない(未確認)ので、木の比較は、テスト用に、字句を調べる補助(`tests/Conductor.Tests/Support/Tree.cs`。製品の `Conductor.Json` とは別の、独立した小さな字句の走査)で行う**。木の比較は、型も比べる(整数と浮動小数点数と真偽値を、値が等しくても同じとしない)。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-206 | YAML から JSON | `T/in.yaml` = `CV1Y` | `convert in.yaml --to json` | 終了 0。ERR 空。OUT は JSON で、`"format_version": 1` と `"id": "extract"` を含む。OUT を JSON として読んだ木が `CV1` の木と等しい。キーの順は入力の記述順、インデントは 2 スペース、末尾は改行(細部は [09](09-converter.md) のとおり) |
| AC-207 | YAML から JSON(アンカー展開) | `T/in.yaml` = `CV2` | `convert in.yaml --to json` | 0。OUT に `<<` も `&defaults` もない。OUT を JSON として読んだ木が、`CV2` を展開した木と等しい(`transform` の `success_exit_codes` は `[0]`、`extract` は `[0, 2]`) |
| AC-208 | 変換の出力は標準出力だけ | `T/in.yaml` = `format_version: 1⏎pipeline:⏎  name: p⏎job:⏎  - id: a⏎    command: echo a` | `convert in.yaml --to json` | 0。ERR 空。OUT に `"job": [` と `"format_version": 1` |
| AC-209 | 入力形式は `--to` の反対で、拡張子は見ない | `T/looks-like.json` = `a: 1⏎b: [x, y]`(YAML の内容) | `convert looks-like.json --to json` / `convert looks-like.json --to yaml` | 前者: 0、OUT を JSON として読むと `{"a": 1, "b": ["x","y"]}`。後者: 67(JSON として読んで `syntax error`) |
| AC-210 | 文法の誤りは種類と行つき | `T/broken.yaml` = `a: [unclosed` | `convert broken.yaml --to json` | 67。OUT 空。ERR の最初の行が `conductor: error: broken.yaml:` で始まり、種類の文が `syntax error`、理由が `YAML の文法に誤りがあります`(行は採用ライブラリの検出位置。未確認) |
| AC-211 | 入力がない | なし | `convert nofile --to json` | 67。ERR = `conductor: error: nofile: input file does not exist: 指定の場所にファイルがありません` |
| AC-212 | ディレクトリを入力に | `T/d` がディレクトリ | `convert d --to json` | 67。`cannot read input file` |
| AC-213 | 入力が大きすぎる | 1,048,577 バイト(中身が壊れていてもよい) | `convert big.yaml --to json` | 67。`input is too large` |
| AC-214 | ちょうど 1 MiB は受理 | `a: "xxx…"` で、ちょうど 1,048,576 バイト | `convert max.yaml --to json` | 0。OUT が `{⏎  "a": "xxx` で始まる |
| AC-215 | UTF-8 でない入力 | `a: ` のあとに不正なバイト列 | `convert bad.yaml --to json` | 67。`cannot read input file`(文法を見る前) |
| AC-216 | 表せない値は場所つき | `T/in.yaml` = `job:⏎  - id: x⏎  - id: y⏎    note: .nan` | `convert in.yaml --to json` | 67。ERR に `value cannot be represented in the target format: NaN と無限大は JSON で表せません` と `(at: job[1].note)` |
| AC-217 | 表せない値と、表せる値の境界 | YAML(`--to json`): `a: .inf` / `a: .nan` / `a: -.inf` / `a: [1, .inf]` / `a: 9223372036854775808` / `a: 0xFFFFFFFFFFFFFFFFFF` / `a: 1e999`。JSON(`--to yaml`): `{"a": 9223372036854775808}` / `{"a": 1e400}` / キーが 1,025 文字のオブジェクト。受理されるもの: YAML の `a: ~` / `a: null` / `a:`(値は `null`)、JSON の `{"a": null}`、`{"a": 9223372036854775807}`、キーが 1,024 文字のオブジェクト | `convert … --to <変換後>` | 表せないものは 67(種類 `unrepresentable`)。原因の文は、順に `NaN と無限大は JSON で表せません`(`.inf`・`.nan`・`-.inf`・`[1, .inf]` の 4 つ)、`数値が表せる範囲を超えています`(YAML の 3 つと JSON の 2 つ)、`キーが長すぎます(上限 1024 文字)`(1,025 文字のキー。文字の数は、UTF-16 の `Length` ではなく、コードポイントの数。AC-383)。受理されるものは 0 で、`null` は `null` のまま出る。64 ビット符号つき整数の範囲の端(`9223372036854775807`)は、桁を落とさずに扱う(`double` に直さない。字句を `BigInteger` で読んで、`long.MinValue <= n <= long.MaxValue` の比較をしてから `long` にする)。`1e999` を `double.Parse` に通すと、版により `Infinity` を返すか `OverflowException` を投げる(.NET Core 3.0 以降は `Infinity` を返すと記憶している。未確認)ので、どちらでも「範囲外」として拒否する(AC-502) |
| AC-218 | 複数の文書・タグ・文字列でないキー | `a: 1⏎---⏎b: 2` / `a: !!str 1` / `1: x` / `true: x` | `convert in.yaml --to json` | 67。種類の文は `syntax error`。理由は順に `複数の文書は書けません(1 ファイルに書ける文書は 1 つです)`、`タグは使えません`、`キーは文字列で書きます`、`キーは文字列で書きます` |
| AC-219 | 最上位がオブジェクトでない | YAML `- 1⏎- 2`(`--to json`)、JSON `[1, 2]`(`--to yaml`) | `convert` | 67。`top level is not a map: 最上位がマップではありません`(行 1) |
| AC-220 | 空の入力 | 空の YAML ファイル(または `# c` だけ)を `--to json` で。空の JSON ファイル(0 バイト)を `--to yaml` で | `convert` | YAML: 67。`top level is not a map: 文書が空です`。JSON: 67。`syntax error`(JSON では空の入力は誤り) |
| AC-221 | コメントは捨てられる | `# 先頭⏎a: 1 # 末尾` | `convert in.yaml --to json` | 0。OUT = `{⏎  "a": 1⏎}`。OUT に `#` がない |
| AC-222 | キーの順序は入力の記述順 | `b: 1⏎a: 2` | `convert in.yaml --to json` | OUT = `{⏎  "b": 1,⏎  "a": 2⏎}`(辞書順に並べ替えない。`SortedDictionary`・`OrderBy` を使わない。マージキーで取り込まれたエントリは `<<` の位置に並ぶ。[09](09-converter.md))。`Dictionary<TKey, TValue>` の列挙順は、公式の文書では保証されない(未確認)ので、順序つきの構造(`List<KeyValuePair<…>>`)で持つ。整数に見えるキー(`"1"`、`"2"`)でも順序は変わらない(`b: 1⏎"1": 2` のような入力で、順序が保たれることも確かめる。AC-511) |
| AC-223 | 空のマップ | `{}`(YAML)、`{}`(JSON) | `convert in.yaml --to json` / `convert in.json --to yaml` | 0。JSON の出力は `{}`。YAML の出力は `{}` |
| AC-224 | アンカーとマージキー | `base: &b {x: 1, y: 2}⏎d:⏎  <<: *b⏎  y: 9` | `convert in.yaml --to json` | 0。`d` は `"x": 1`、`"y": 9`(自身のキーが優先) |
| AC-225 | マージキーの値がマップでない | `a:⏎  <<: 5` | `convert in.yaml --to json` | 67。種類の文は `syntax error`、理由は `マージキーの値がマップではありません`(行 2。場所 `(at: …)` は付かない) |
| AC-226 | 重複するキー | YAML `a: 1⏎b: 2⏎a: 3`(`--to json`)、JSON `{"a": 1, "b": 2, "a": 3}`(`--to yaml`) | `convert` | 67。`syntax error: キーが重複しています`(YAML は行 3) |
| AC-227 | 再帰するアンカー | `a: &r [*r]` | `convert in.yaml --to json` | 67。`アンカーが自分自身を含んでいます`(スタックが溢れない) |
| AC-228 | スカラーの解決(YAML 1.2 のコアスキーマ。03 章 4.2 節) | YAML: `a: yes`、`b: 0x1F`、`c: 007`、`d: '1'`、`e: 2001-01-01`、`f: 1_000`、`g: on`、`h: ~~`、`i: 'null'`、`j: 1:30`、`k: 0b101`、`l: 0o17`(YAML ライブラリの型を指定した復元は、`yes`・`on` を真偽値に、`2001-01-01` を日付にする場合があるので、構文要素の段階までを使い、型は自前で決める。AC-525) | `convert in.yaml --to json` | `"a": "yes"`、`"b": 31`、`"c": 7`、`"d": "1"`、`"e": "2001-01-01"`、`"f": "1_000"`、`"g": "on"`、`"h": "~~"`、`"i": "null"`、`"j": "1:30"`、`"k": "0b101"`、`"l": 15`(キーは入力の記述順に並ぶ)。真偽値になるのは `true`/`false` の 3 通りの書き方(`true`、`True`、`TRUE` と `false` の同様の形)だけ。食い違うものは、03 章を正本とする |
| AC-229 | 展開後の要素数の上限 | `a: [0, 0, …]`(0 を n 個)。JSON は `{"a": [0, 0, …]}` | `convert in.yaml --to json` / `convert in.json --to yaml` | n = 99,997 は 0。n = 99,998 は 67(種類の文 `input is too large`。理由 `要素数が上限(100000)を超えています`)。JSON も同じ境界 |
| AC-230 | アンカーの爆発は上限で止まる | アンカーの入れ子で、指数的に増える YAML | `convert in.yaml --to json` | 30 秒以内に 67(`input is too large`。理由は `要素数が上限(100000)を超えています` または `文字の大きさが上限(16 MiB)を超えています`)。メモリを使い果たさない(`OutOfMemoryException` で落ちない) |
| AC-231 | 入れ子の深さの上限 | YAML: `a: ` に続けて `[` を 32 個 / 33 個(閉じも書く)。JSON: `{"a": ` に続けて `[` を 32 個 / 33 個(閉じも書く) | `convert` | 32 個: 0。33 個: 67(`syntax error: 入れ子が深すぎます(上限は 32 段)`)。最上位のオブジェクト(根)は数えず、その直下のコレクションを 1 段と数える([03](03-definition-format.md)) |
| AC-232 | 極端な深さでも異常終了しない | 1,000 段、100,000 段の入れ子(YAML・JSON) | `convert … ` | 67(`syntax error`)。異常終了しない(`StackOverflowException` でプロセスが落ちない。2.2 節。スレッドのスタックを大きくして対処しない) |
| AC-233 | 往復で木が等しい | `CV1` | `convert in.json --to yaml` の OUT を `convert --to json` に通す | 結果の木が `CV1` の木と等しい。`command` が配列形式の定義、`retry` のオブジェクトも同じ。`CV1Y` を先に `--to json`、次に `--to yaml` に通しても等しい |
| AC-234 | 数値の往復 | 整数の端(64 ビット符号つき整数の最小・最大)、浮動小数(`1.0`、`-0.0`、`0.5`、`1e21`)、空の配列・オブジェクト | 往復(YAML → JSON → YAML、JSON → YAML → JSON) | 型が変わらない(`1` と `1.0` を区別する。`0.0` と `-0.0` は別。JSON の出力で、浮動小数は、小数点か指数を持つ書き方になる。`double.ToString("R")` の `1E+21` と、`1e21` の書き方の差は 09 章に従う。AC-544)。整数の端は、桁を落とさない |
| AC-235 | ファイルを作らず環境に左右されない | `T` に入力のみ | `convert` を、別のカレントディレクトリ・異なる環境変数で実行 | `T` のファイル一覧が実行の前後で同じ。結果が同じ |
| AC-236 | 標準出力に書けない(注入) | 正しい入力 | `convert`(標準出力の `BufSink` を書き込み失敗にする) | 67。ERR = `conductor: error: cannot write to standard output` |
| AC-237 | 引数の誤り | なし | `convert f`(`--to` なし)/ `convert f --to xml` / `convert f --to YAML` / `convert f --to Json` / `convert f --to yaml --config c` | 68。`missing argument: '--to'` / `invalid value: 'xml'` / `invalid value: 'YAML'` / `invalid value: 'Json'` / `option not allowed for this command: '--config'` |
| AC-238 | 変換結果の全体を 1 回で書く | 失敗する入力 | `convert` | 失敗のとき、標準出力に 1 バイトも書かれない |

### 4.8 run ID

特に記さない限り、定義は `two_jobs(n)` = `P(p; a:0, b:<n>)`(`a` は成功、`b` は終了コード `n`)。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-239 | run ID ごとに別のステート | `two_jobs(7)` | `pipeline p --run-id r1` と `pipeline p --run-id r2` を同時に | どちらも 7。OUT 空。`T` のファイルは `conductor.json`、`conductor.json.r1.state`、`conductor.json.r2.state` の 3 つだけ。各ステートは `run_id` がその ID、`resume=b`、`a=success` |
| AC-240 | 同じ run ID で再開し、他は変わらない | 上のあと、定義を `two_jobs(0)` に直す | `pipeline p --run-id r1` | 0。`r1` は完了のステートが残る(`completed=true`)。`r2` のステートファイルはバイト列が変わらない |
| AC-241 | ステートの形(run ID あり・なし) | `two_jobs(7)` | `pipeline p --run-id r1`、`pipeline p` | `.r1.state` は `"run_id": "r1"` を持つ。run ID なしの `.state` は `run_id` キーを持たない |
| AC-242 | 環境変数 `CONDUCTOR_RUN_ID` | `two_jobs(7)`、`CONDUCTOR_RUN_ID=fromenv` | `pipeline p` | 7。`conductor.json.fromenv.state` ができる |
| AC-243 | 空のオプションは run ID なしに固定 | 同じ環境変数 | `pipeline p --run-id ""` | 7。`conductor.json.state`(run ID なし)ができる |
| AC-244 | オプションは環境変数に優先 | 同じ環境変数 | `pipeline p --run-id fromopt` | `conductor.json.fromopt.state` ができる(`fromenv` は使われない) |
| AC-245 | 環境変数が空なら run ID なし | `CONDUCTOR_RUN_ID=` | `pipeline p` | 7。`conductor.json.state` |
| AC-246 | 定義の `run_id_env` | `P(p run_id_env=MY_RUN; a:7)`、`MY_RUN=m1`、`CONDUCTOR_RUN_ID=ignored` | `run a` | 7。`conductor.json.m1.state` ができる(`ignored` は使われない) |
| AC-247 | `watch`・`reset` も `run_id_env` を読む | 上のあと、`MY_RUN=m1` | `watch`、`reset` | `watch` の OUT に `Run ID: m1`。`reset` で `conductor.json.m1.state` が消える |
| AC-248 | 有効な run ID | `two_jobs(0)` | `pipeline p --run-id <値>`(`1`、`r1`、`12345`、`build-2026.11_a`、`a.b`、`x` を 64 個) | いずれも 0 |
| AC-249 | 無効な run ID(オプション) | `two_jobs(0)` | `pipeline p --run-id <値>`(`a/b`、`..`、`x` を 65 個、`a b`、`.x`、`日本語`、`r1<LF>`(末尾が改行。AC-542)) | 68。ERR の最初の行 = `conductor: error: invalid value: '--run-id'`。ERR にその値が現れない。ステートが作られない |
| AC-250 | 無効な run ID(環境変数) | 同じ値を `CONDUCTOR_RUN_ID` に | `pipeline p` | 68。ERR の最初の行 = `conductor: error: invalid value: 'run ID (environment variable)'`。値が現れない |
| AC-251 | validate は run ID を受け付けない | `two_jobs(0)` | `validate --run-id x` | 68 |
| AC-252 | run ID の決定の順序 | 定義の誤り + `--run-id 'a b'` | `run a --run-id 'a b'` | 64(定義の検査が先。run ID は後) |
| AC-253 | run ID の決定はステートの読み込みの前 | 壊れたステート + `--run-id 'a b'` | `run a --run-id 'a b'` | 68(65 ではない) |
| AC-254 | `watch` は `--run-id` があれば定義を読まない | 定義が壊れている。`.r1.state` あり | `watch --run-id r1` | 0。表が出る |
| AC-255 | `watch`・`reset` は壊れた定義でも既定の環境変数名 | 定義が壊れている。`CONDUCTOR_RUN_ID=r1` | `reset` | 0。`.r1.state` だけが消える |
| AC-256 | run ID なしの `watch` に `Run ID` の行はない | run ID なしのステート | `watch` | OUT に `Run ID` の行がない |
| AC-257 | ファイル名と中身の `run_id` の食い違い | `conductor.json.r1.state` を `conductor.json.r2.state` に複製する | `run b --run-id r2` | 65。ERR = ``conductor: error: conductor.json.r2.state: state file is corrupt: run_id がファイル名の run ID と一致しません; run `conductor reset --run-id <ID>` to discard the state``(`<ID>` は文字どおり) |
| AC-258 | run ID ありでも `--remove-on-complete` で消える | `two_jobs(0)` | `pipeline p --run-id r1 --remove-on-complete` | 0。`conductor.json.r1.state` がない |
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
| AC-268 | 待ち時間の列(関数の検査) | `interval` / `multiplier` / `max_interval` / `max` | 待ち時間を求める関数(`Conductor.Flow`。結果は `ulong`) | `30s`、倍率なし、max 3 → 30、30、30。`10s`、2、`60s`、max 5 → 10、20、40、60、60。`60s`、3、`60s`、max 3 → 60、60、60。`1s`、10、`1h`、max 6 → 1、10、100、1000、3600、3600。`1h`、10、`24h`、max 4 → 3600、36000、86400、86400。極端に大きい値でも結果が `ulong.MaxValue` を超えない(`Math.BigMul` か `UInt128` による飽和。`checked` の例外にも、折り返しにもしない。AC-501)。区分: 単体 |
| AC-269 | 再実行の結果が成功なら遷移へ(pipeline) | `P(p; a:0{cmd="TJ flaky F 1 75", retry=(…)}, b:0)` | `pipeline p` | 0。`b` も実行される。完了のステート |
| AC-270 | 使い切ると停止(pipeline) | `RT` に `b:0` を足し、`fails=99` | `pipeline p` | 75。`b` は実行されない。`resume=a` |
| AC-271 | ジョブ自身の 127 は一覧にあればリトライ | `TJ exit 127`、`exit_codes: [127]`、`max: 1`、`interval: 1s` | `run a` | 127。retry の行が 1 つ(1 回リトライしてから終わる) |
| AC-272 | 起動の失敗はリトライしない | 存在しないプログラム、`exit_codes: [127]` | `run a` | 127。retry の行なし |
| AC-273 | シグナルで終了したジョブはリトライしない(UNIX 系) | `argv=["sh","-c","kill -9 $$"]`、`exit_codes: [137]` | `run a` | 137。retry の行なし |
| AC-274 | 待機中のシグナル(UNIX 系) | `retry=(exit_codes=[75], max=3, interval="60s")`、`fails=99` | `run a` を起動し、ERR に最初の retry の行が出てから SIGTERM を送る(`Harness.Signal(process, 15)`) | 143。ステート: `a=pending`(エラーとして記録しない)、`resume=a`。2 つ目の retry の行は出ない。ジョブは再び起動されない(`F` = 1) |
| AC-275 | 待機中の中断は 1 秒以内 | `interval="1h"`、`fails=99` | 最初の retry の行のあと SIGTERM | SIGTERM を送ってから 1 秒以内に、143 で終わる(待ちは、シグナルの旗を見る短い区間に分けるか、合図で起こす(07 章)。**1 回の長い `Thread.Sleep` で待ってはならない**。区間の長さは 50 ms 以下)。単体では、3600 秒の待機に 200 ms 後の合図(別のスレッドから、旗を立てる関数を呼ぶ)で、全体が 1.2 秒未満 |
| AC-276 | 再開は最初の実行から数え直す | 上のあと(`resume=a`) | 再度 `run a`(`interval="1s"`、`fails=1`) | `retry 1 of 3` の行から始まる。0 で完了 |
| AC-277 | 待機中のシグナル(pipeline) | 同様(pipeline) | `pipeline p`、待機中に SIGTERM | 143。以降のジョブを実行しない。`resume=a` |
| AC-278 | ステートは最終の結果で 1 回だけ | `RT`(`fails=2`) | `run a` | 実行の途中に、ステートファイルのジョブ `a` の状態が `success`/`error` にならない(最終の結果のときだけ変わる。途中の失敗はステートに書かれない)。リトライの回数はステートに書かれない。観測: 待機中にステートファイルを読む |
| AC-279 | README のリトライの説明 | README | (検査) | 見出し、5 つのキー名(`exit_codes`、`max`、`interval`、`multiplier`、`max_interval`)があり、載っている JSON と YAML の例が実際に読み取りを通る。区分: 点検 |

### 4.10 シグナルと中断

UNIX 系の「実行」は、被試験体を起動し(`StartConductor`)、`TJ sleep` の記録 `ready` が `O` に現れるのを待ってから、被試験体のプロセスにシグナルを送る(`Harness.Signal(process, 15)` など。`kill(2)` の P/Invoke。`Process.Kill` は強制終了なので使えない)。定義は `SL` = `P(p; sleeper:0{cmd="TJ --out O sleep"}, after:0)` を使う。
ジョブの終わり方(終了コードの値)と、シグナル番号の対応は、07 章の規則に従う(AC-504)。**.NET の `Process.ExitCode` が、シグナルで終了した子に対して返す値は、公式の文書で確認できていない(未確認)。実機での確認は AC-140 が兼ねる。**
シグナルのハンドラ(`PosixSignalRegistration` のコールバック)は、メインスレッド以外(スレッドプールなど)で実行されうる(決定 8。AC-538)。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-280 | 実行中の SIGTERM(UNIX 系) | `SL` | `run sleeper` を起動し、`ready` のあと SIGTERM | 143。OUT 空。ステート `S[sleeper=error, after=pending ; resume=sleeper]`。`pid=` に記録されたジョブのプロセスが存在しない(孤児なし。`kill(pid, 0)` が `ESRCH`) |
| AC-281 | HUP のあとすぐ INT(UNIX 系) | `SL`(`sleeper` だけでもよい) | `ready` のあと SIGHUP、続けて SIGINT | 129(最初の番号 = 1 が残る)。ステート `sleeper=error` |
| AC-282 | SIGINT / SIGQUIT 単独(UNIX 系) | `SL` | SIGINT / SIGQUIT | 130 / 131(SIGQUIT では、`TJ` の終わり方は実装時に確認。3.4 節。確かめるのは `conductor` の終了コードだけ) |
| AC-283 | -v の表示(UNIX 系) | `SL` | `run -v sleeper`、SIGTERM | 143。ERR に `[conductor] job=sleeper result=error (interrupted by signal 15)` |
| AC-284 | 時間を空けた 2 つのシグナルは最初の番号が残る | シグナル監視の単体 | INT のあと、時間を空けて HUP | 最初の番号 2 が残る(`Interlocked.CompareExchange` で、0 のときだけ書く)。区分: 単体(UNIX 系) |
| AC-285 | 最初の番号・登録は 1 回 | シグナル監視の単体 | 監視を登録し、HUP を受ける。再度の登録 | 最初の番号 1 が残る。2 回目の登録は「すでに登録済み」(失敗ではない。製品の側の状態で判定する。`PosixSignalRegistration` 自体は、同じシグナルへの複数の登録を許す見込みだが、確認していない。未確認)で、1 回目の登録が、そのまま 1 つだけ有効(ハンドラが 2 回呼ばれない)。区分: 単体(UNIX 系) |
| AC-286 | 転送は同じシグナルを 1 回だけ | 起動の部品(`Conductor.Runner`)の単体 | ジョブの実行中に合図を立てる(SIGTERM と SIGINT で別々に) | ジョブは、そのシグナルで終了する(07 章の規則で番号 15 / 2)。送信は 1 回(続けて合図を立てても追加で送らない)。区分: 単体(UNIX 系) |
| AC-287 | 許可外の番号は送らない | 起動の部品の単体 | 合図の番号が SIGKILL(9)、SIGSTOP、SIGCONT、SIGUSR1、0、-1、9999 | 送信しない。ジョブは自分で終わる。終わりの観測の直後に `Interrupted = true`、`SignalNumber` がその番号。区分: 単体(UNIX 系) |
| AC-288 | ジョブのセッションとプロセスグループ | 起動の部品の単体。**07 章の決定に従う**(.NET の `ProcessStartInfo` には、子を新しいセッションやプロセスグループに置く指定がない(確認した範囲の記憶。未確認)ので、置く場合は、07 章が定める P/Invoke の方法による) | ジョブを起動して、プロセスグループ ID を調べる(`getpgid(pid)` の P/Invoke) | 07 章が「新しいグループに置く」と決めた場合: ジョブのプロセスグループ ID が、ジョブの pid に等しく、`conductor` のものと異なる。「置かない」と決めた場合: ジョブのプロセスグループ ID が、`conductor` と同じ。いずれの場合も、`conductor` は、シグナルを子の pid にだけ送る(プロセスグループには送らない)。Windows では行わない。区分: 単体(UNIX 系) |
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
| AC-301 | Windows の Ctrl+C と Ctrl+Break(手動) | Windows の実機、コンソール。`SL` | `conductor run sleeper` の実行中に Ctrl+C を押す。別の回に Ctrl+Break を押す | (1) Ctrl+C: 終了コード 130。Ctrl+Break: 終了コード `128 + <07 章が定める番号>`。(2) ジョブの子プロセスが残らない。(3) 続く `watch` で `sleeper` が `error`、`after` が `pending`。(4) `run -v` の ERR に `interrupted by signal 2`(Ctrl+Break は、その番号)。.NET では、Ctrl+C と Ctrl+Break は `Console.CancelKeyPress`(`ConsoleSpecialKey.ControlC` と `ConsoleSpecialKey.ControlBreak`)で届く(07 章に従う。この文書では、公式の文書を確認していない。未確認)。結果を記録欄に残す(`tests/WINDOWS_CTRL_C_CHECKLIST.md`)。区分: 手動(Windows)。実施の記録は、この文書の作成時点では存在しない(未確認) |

### 4.11 ステートの異常系

ここの「実行」は、特に記さない限り `run a`(`P(p; a:0, b:0)`)。すべて、**ジョブは実行されず**、ステートファイルは**変わらず**、OUT は空。ERR の最初の行は `conductor: error: conductor.json.state: <種類の文>: <理由>; <案内>`。
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
| AC-312 | 定義との食い違い: パイプライン名 | `pipeline_name` が定義と違う(定義側の名前を変えて `run`) | `conductor: error: conductor.json.state: state does not match the definition: pipeline name; run `conductor reset` to discard the state` |
| AC-313 | 定義との食い違い: ジョブID | ジョブの追加・改名のあと | `state does not match the definition: job ids; …` |
| AC-314 | 定義との食い違い: ジョブの並び | ジョブの順序を入れ替えたあと | `state does not match the definition: job order; …` |
| AC-315 | 食い違いの案内は run ID ありで変わる | `--run-id r1` のステートで定義を変える | `…; run `conductor reset --run-id <ID>` to discard the state` |
| AC-316 | 日本語 | `{garbage`、`LANG=ja_JP.UTF-8` | `conductor: error: conductor.json.state: ステートファイルが壊れています: JSON として読めません; `conductor reset` でステートを初期化できます` |

ステートの異常のあとの回復と、正常な読み込み。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-317 | reset で回復する | 上のいずれかのあと | `reset`、続けて `run a` | 0。先頭から実行される |
| AC-318 | pipeline も同じ | 上のいずれか | `pipeline p` | 65。ジョブは起動しない |
| AC-319 | ファイルなし・長さ 0 は「空」 | ステートなし / 長さ 0 | `run a` | 通常どおり 0 |
| AC-320 | 完了のまま残ったステートは「空」 | 完了のステート(正しい) | `pipeline p` | 先頭から実行して 0。読み込みでファイルは消されない(実行の最初の書き込みで上書きされる) |
| AC-321 | 残った一時ファイルは読み込みに影響しない | `.conductor.json.state.tmp.1.2.3` がある | `run a` | 通常どおり 0 |
| AC-322 | 書き込みは原子的 | 書き込み前後 | (ステートを読み続ける) | 読み手は、直前の完全な内容か新しい完全な内容だけを見る(同じディレクトリの一時ファイルへ書き、`FileStream.Flush(true)`(ディスクへの書き出し)で同期し、置き換える(`File.Move(src, dst, overwrite: true)` か `File.Replace`。06 章の決定)。UNIX 系では `rename(2)` に依る原子性。**Windows で、置き換え先が他のプロセスに開かれているときの挙動と、同時の読み手に対する原子性は、未確認**)。書き込みの失敗でも直前の内容が残る。置き換え先がディレクトリなら書き込みの失敗でステートは変わらず、一時ファイルも残らない。ディレクトリの `fsync` は、UNIX 系で行い(P/Invoke の `fsync`)、Windows では行わない(できない。06 章) |
| AC-323 | 排他制御はしない | 2 つの `run` を同時に | (2.1 節) | 両方が動く。AC-404 を参照 |
| AC-324 | 終了と書き込みの間の停止は再実行(at-least-once) | 終了の直後、書き込みの前に `conductor` が止まる | 再度 `run a` | ジョブが再実行されうる。**外からは再現できない**(偽の部品で検査する)。区分: 注入 |

### 4.12 引数の文法

特に記さない限り、定義は `B0`、OUT は空、ERR は `conductor: error: <文>`⏎(空行)⏎`Usage:` から始まる使い方の全文。「68」の行は、終了 68。引数は、`System.CommandLine` などのライブラリを使わず、自前で解析する([04](04-cli.md))。理由は、この文法(`--opt=value`、短縮形の連結、`--`、値を取るオプションが `-` で始まる語も値にする、オプションの省略形を受理しない、エラーの優先順位と終了コード 68)を、ライブラリの設定だけで満たせるかが確認できず、(ライブラリの文書を確認していない。未確認)、満たせない場合の回避が、自前の解析より複雑になるため。

| ID | 機能 | 実行 | 期待 |
|----|------|------|------|
| AC-325 | 引数なし | `conductor` | 68。`missing argument` |
| AC-326 | オプションだけ | `-v` | 68。`missing argument` |
| AC-327 | 知らないコマンド | `bogus` / `''`(空文字) | 68。`unknown command: 'bogus'` / `unknown command: ''` |
| AC-328 | 必須の引数がない | `run` / `pipeline` / `convert` | 68。`missing argument` |
| AC-329 | 余分な引数 | `run a b` / `validate a b` / `watch x` / `reset x` | 68。`unexpected argument: 'b'` / `'b'` / `'x'` / `'x'` |
| AC-330 | 使えないオプション | `watch -v`、`reset -v`、`run --all a`、`run --to yaml a`、`validate --remove-on-complete`、`reset --remove-on-complete`、`convert f --to yaml -v` | 68。`option not allowed for this command: '<そのオプション>'` |
| AC-331 | `--run-id` が使えるコマンド | `run`、`pipeline`、`watch`、`reset` に `--run-id x` | 文法として受理される(4 コマンド)。`validate`・`convert` では 68 |
| AC-332 | 知らないオプション | `run --bogus a` / `run -x a` / `run --bogus=x a` | 68。`unknown option: '--bogus'` / `'-x'` / `'--bogus'`(`--conf` のような省略形は、`--config` として受理せず、`unknown option: '--conf'`) |
| AC-333 | 値の欠落 | `run a --config` / `run a --run-id` / `convert f --to` | 68。`option requires a value: '--config'` / `'--run-id'` / `'--to'` |
| AC-334 | 重複 | `run -vv a`、`run -v -v a`、`run -v --verbose a`、`run a --config x --config y`、`run a --run-id x --run-id y`、`reset --all --all`、`run a --remove-on-complete --remove-on-complete`、`convert f --to yaml --to json` | 68。`invalid value: '<オプション名>'`(`-vv` は `'-v'`。重複を後勝ちで黙って受理しない) |
| AC-335 | `--all` と `--run-id` | `reset --all --run-id x` | 68。`invalid value: '--all'` |
| AC-336 | 値を取らないオプションに `=値` | `run --verbose=x a`、`reset --all=x`、`run a --remove-on-complete=x` | 68。`invalid value: '--verbose'` / `'--all'` / `'--remove-on-complete'` |
| AC-337 | `--to` の値 | `convert f --to xml` / `YAML` / `Json` | 68。`invalid value: 'xml'` など(値そのもの) |
| AC-338 | `--to` がない | `convert f` | 68。`missing argument: '--to'` |
| AC-339 | `--run-id` の値の検査 | `run a --run-id 'a b'` | 68。`invalid value: '--run-id'`(値を含めない) |
| AC-340 | 空の `--config` | `run a --config=` | 64。`definition file does not exist`(空のパスとして受理され、後で誤りになる) |
| AC-341 | オプションの書き方 | `--config c.json -v run a`、`run a -v`、`--config=c.json`、`--run-id=r1`、`convert f --to=yaml` | すべて文法として正しい(`run` は実行される。`--run-id=` は run ID なしに固定) |
| AC-342 | `--` のあとは位置引数 | `run -- -weird` / `-- --help` / `run a -- -v` | `job is not defined in the definition file: '-weird'`(68)/ `unknown command: '--help'`(68)/ `unexpected argument: '-v'`(68) |
| AC-343 | ヘルプ(英語) | `--help`、`-h` | 終了 0。OUT = [08](08-output-and-i18n.md) の使い方の全文(末尾は改行)。ERR 空 |
| AC-344 | ヘルプ(日本語) | `LANG=ja_JP.UTF-8` で `--help` | 0。OUT = 08 章の日本語の全文 |
| AC-345 | バージョン | `--version`、`-V` | 0。OUT = `conductor 0.8.0`(版は配布の版)。ERR 空。`LANG=ja_JP.UTF-8` でも英語・同じ |
| AC-346 | `-h`/`-V` は最優先 | `-vh`、`bogus --help`、`run -vv --help`、`-V -h`、`-h -V` | `-vh` は help(0)。`bogus --help` も help(0)。`-V -h` は version、`-h -V` は help(先に現れた方) |
| AC-347 | 使い方の誤りの形 | `bogus`(`LANG=C`)、`bogus`(`LANG=ja_JP.UTF-8`) | ERR の最初の行 = `conductor: error: unknown command: 'bogus'` / `conductor: error: 知らないコマンドです: 'bogus'`。2 行目は空行、3 行目以降は使い方の全文(英語 / 日本語)。OUT 空。ジョブは実行されない。**終了コードは 68** |
| AC-348 | 使い方の誤りの全種類 | 上の各種類 | 8 種類(`UnknownCommand`、`MissingArgument`、`UnexpectedArgument`、`UnknownOption`、`OptionNotAllowed`、`MissingValue`、`InvalidValue`、`UnknownJob`)が、すべて 68 で、08 章の英語・日本語の文になる |
| AC-349 | UTF-8 でない引数(UNIX 系) | 定義あり。位置引数(ジョブID)・`--run-id` の値・`--config` と `convert` の入力パスに不正なバイト列(`FF FE`。AC-406 と同じ方法で渡す)。実行: `run <不正なバイトのジョブID>`、`run a --run-id <不正なバイト>`、`--config <不正なバイトのパス>`、`convert <不正なバイトのパス> --to json` | **.NET が不正なバイトを `string[] args` にどう変換するかは、公式の文書で確認できていない(未確認)**。確かめるのは、実装が何らかの `string` を受け取っても、異常終了しないことと、出力が有効な UTF-8 になること。ジョブID: 68、`unknown job`(定義にないため)。`--run-id`: 68、`invalid value`(run ID の文字集合の外のため)。`--config` は 64、`convert` は 67(その名前のファイルがないため)。元のバイト列の名前のファイルを開けることは、期待しない(開けるかは未確認)。ERR の不正な文字は、エスケープ規則(08 章)どおりに出る(AC-519)。区分: UNIX 系 |
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
| AC-367 | 完了を判定できない | 全ジョブが「実行済み」で通過し、どこでも完了にならない状態(通常は起こらない) | `pipeline p` | 65。ERR = `conductor: error: conductor.json.state: cannot determine whether the pipeline is complete`。ステートを初期化しない。区分: 注入 |
| AC-368 | 内部の誤りの文は固定の英語 | `Conductor.Cli` が例外(`throw new InvalidOperationException(...)`)を投げる(偽の部品に投げさせる) | (入口の `Program.Main` の `try`/`catch`) | ERR = `conductor: error: internal error`(ロケールによらず。例外のメッセージとスタックトレースは出さない)、終了 70。標準エラーが閉じていても 70。区分: 注入 |
| AC-369 | 入口に異常終了を起こすコマンドがない | 製品の入口 | (点検) | 入口(引数の解釈の結果)に、わざと異常終了するコマンドがない。入口は 1 回だけ終了コードを返す。点検の項目は AC-421、AC-500 |

### 4.14 表示のエスケープ

入力に由来する文字列(ジョブID、名前、パス、キー名、コマンド名、使い方の誤りの対象)は、表示する前にエスケープする。**ジョブ本体の出力は通さない**(そのまま)。

規則: LF → `\n`、CR → `\r`、TAB → `\t`。そのほかの制御文字(U+0000〜U+001F、U+007F〜U+009F)と、書式・方向制御の文字(U+200B〜U+200F、U+2028、U+2029、U+202A〜U+202E、U+2060〜U+2064、U+2066〜U+2069、U+FEFF)は、`\u{<小文字の 16 進。先頭の 0 なし>}`。
孤立サロゲート(U+D800〜U+DFFF。UTF-16 の `string` で、対になっていない `char`)は、`\u{<小文字の 16 進>}`(例: `\u{d800}`)とする(08 章に従う)。.NET の `string` は UTF-16 で、`char` はコードユニットなので、**走査は `Rune`(`string.EnumerateRunes()`)で行い、孤立サロゲートは `Rune` ではなく `char` として検出する**(`EnumerateRunes()` は、孤立サロゲートを U+FFFD に置き換えて返すので、置換された `Rune` を見ただけでは、元が孤立サロゲートか本物の U+FFFD かを区別できない。区別する実装にする。AC-519)。
そのほかの文字(日本語を含む)はそのまま。名前用は、エスケープ後 64 文字(**コードポイント**。`string.Length`(UTF-16 のコードユニット)ではなく、`Rune` の数で数える。サロゲートペアを途中で切らない)で切る(エスケープ列は 1 かたまり。次の 1 文字を足すと 64 を超えるなら、そこで打ち切り、末尾に `…`(U+2026)を 1 つ付ける。出力は最大 65 文字)。パスは切らない。結合文字の列は、コードポイントごとに数える(書記素では数えない)。

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
| AC-392 | 終了コードの範囲合わせ(関数の検査) | 報告する値 `reported`(`int`。32 ビットの符号なしで渡される場合は `uint` から `unchecked((int)v)` で直した値)、`isError` | 範囲合わせの関数(`Conductor.Flow`) | `0..255` はそのまま。`259`→3、`511`→255、`-1`→255、`-2`→254、`unchecked((int)0xC0000005)`(`3221225477` を直した値)→5、`unchecked((int)4294967295u)`(= -1)→255。`256`・`512`・`-256`・`int.MinValue` は、エラーなら 1、エラーでなければ 0。C# の `%`(負の数で、負の結果を返す)と `&` の違いに注意する(`-1 % 256 == -1` だが、`-1 & 0xFF == 255`。下位 8 ビットは `& 0xFF` で取る)。`checked` のキャストで例外にしない。区分: 単体 |
| AC-393 | 中断の終了コード(関数の検査) | シグナル番号 n | `128 + n` を範囲合わせ(エラー扱い) | 15→143、2→130、1→129、3→131、300→172(範囲を超えない)。区分: 単体 |
| AC-394 | 分類の全 10 項目(関数の検査) | [07](07-process-and-signals.md) の分類の表の各行 | 分類の関数(`Conductor.Flow`) | 順 1: 起動前の中断 → 分類なし、`128 + n`。順 2: 起動なし・中断でもない → エラー 127。順 3: 起動済みの中断 → `128 + n`(ジョブの終了コードを見ない)。順 4: 起動失敗 → 127。順 5: シグナル終了 n → `128 + n`。順 6: 終了コードなし → 1(`conductor` のエラー)。順 7: 成功コード → 0。順 8: 警告コード → 0。順 9: 0 で成功コードにない → 1(`conductor` のエラー)。順 10: そのまま。優先順位は上の行ほど高い。区分: 単体 |
| AC-395 | 終了コードの決定順序 | 複数の誤りを同時に持つ入力 | `run` | 検査の順(引数 68 → 準備 70 → 定義 64 → `command` 64 → ジョブID 68 → run ID 68 → ステートの読み込み 65 → 照合 65 → 起動前の判断 66/0 → 初期ステート 65 → ジョブ → 結果の書き込み 65 → 削除 65)で、最初の誤りだけが採られる。具体例は 4.3 節(定義の誤り > ジョブID)、4.8 節(定義 64 > run ID 68 > ステート 65) |
| AC-396 | 128 + 番号の例 | ジョブが SIGKILL / 実行中に SIGTERM(UNIX 系) | `run a` | 137 / 143 |

### 4.16 非機能要件の検証

2 章の各基準の受け入れテスト。「計測」は p95 などの測定、「強制」は基準超過で不合格(2 章の扱いの列)。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-397 | 定義の読み込みの性能 | ジョブ 10 個の定義(ファイルから読む) | 最初の 5 回を捨てて、読み込みと検証を 100 回(プロセス内) | p95 が 20 ms 以下(目安。超えたら警告を記録)。区分: 計測 |
| AC-398 | 起動前の処理の性能 | ジョブ 10 個の定義 | `command` の検査、ステートの読み、照合、速い書き込みを、最初の 5 回を捨てて 100 回 | p95 が 20 ms 以下(仮。目安)。区分: 計測 |
| AC-399 | `run` のオーバーヘッド | ジョブ 10 個の定義(各ジョブは `TJ timing`) | `run job0` を 20 回 | `conductor` 自身の時間(全体 − `TJ` の基準の起動の費用 − (`end_ns` − `start_ns`))が p95 600 ms 以下(仮。目安)。区分: 計測 |
| AC-400 | `pipeline` のジョブあたり | 同上 | 10 個と 1 個の `pipeline` を各 5 回 | ジョブ 1 つあたりの増分(2.1 節の式)が p95 80 ms 以下(仮。目安)。区分: 計測 |
| AC-401 | 大きな定義 | ジョブ 1,000 個の定義 | `run job0` | 2,500 ms 以内(仮。目安)。区分: 計測 |
| AC-402 | 大きな流れ | ジョブ 1,000 個の `pipeline`(偽の部品で、プロセスもファイルも使わない) | (内部の流れ) | 10,000 ms 以内(目安)。完了のステートが残ることは常に検査。区分: 計測・単体 |
| AC-403 | 待機中の CPU 時間(Linux) | `TJ sleep 4`(約 4 秒)のジョブ | `run a` を起動し、`conductor` の pid の `/proc/<pid>/stat` の `utime + stime`(14・15 番目の値。単位は時計の刻み。`getconf CLK_TCK` で秒に直す)を、起動の 1 秒後と 2.5 秒後に読む | 増分が 50 ms 以下(待機中に、子の終了の確認(10 ms 以下の間隔。07 章)で CPU を使いすぎない)。強制(`/proc` が読めなければスキップ)。区分: Linux |
| AC-404 | 同時の 2 つの `run` | `P(p; a:0{cmd="TJ sleep 1"})`(終了コード 99) | 同じ定義に `run a` を 2 つ同時に | 両方が実行され(ジョブが 2 回始まる)、どちらも終了コード 99。ステートは正しい形で読み戻せる。強制 |
| AC-405 | ランダムな引数 | 固定の種(`new Xorshift32(1)`) | 語の表から 0〜6 個を選んだ引数列で 500 回(`Conductor.Cli` を偽の部品で、入口の関数 `EntryPoint.Run` を通して呼ぶ) | 終了コードが 0〜255。標準エラーに、制御文字(改行を除く)と U+202E がない。例外が漏れない |
| AC-406 | UTF-8 でない引数(UNIX 系) | `0xff 0xfe` を含むバイト列。.NET の `ProcessStartInfo.ArgumentList` は `string` しか受け付けず、不正なバイト列をそのまま渡せない。そこで、テストの補助が、`/bin/sh -c 'exec "$0" "$@"' conductor run "$(printf '\377\376')"` の形で、シェルに引数を組み立てさせて起動する(テストの補助だけがシェルを使う。製品はシェルを使わない) | ジョブID・`--config`・オプション・`convert` の引数に渡す | 異常終了しない(終了コードが 0〜255。70 にならない)。標準エラーが、有効な UTF-8 で読める。区分: UNIX 系 |
| AC-407 | 壊れたステート | 状態・再開位置・フラグを乱したステート、ランダムなバイト列、巨大な値 | `run` / `watch` | 異常終了しない。壊れていれば 65 |
| AC-408 | 空・制御文字のジョブID | ジョブID が空、制御文字を含む | `run <ID>` | 異常終了しない(68) |
| AC-409 | ランダムな `command` | 文字集合 `a b ⇥ ⏎ ' " \ ; \| & $ * あ` から長さ 0〜39 の文字列 2,000 件(定義の `command` に入れる) | `validate`(分割の単体も) | 0 か 64 のどちらか。0 のとき `program` は空でない |
| AC-410 | 大きな `command` | `prog ` + `'a b' "c\"d" e ` を 10 万回(約 30 万引数) | `validate` | 0。閉じない引用符を足したものは 64。30 秒以内。文字列の `+=` の繰り返しによる 2 乗の時間(`StringBuilder` と `List<string>` を使う)や、再帰でスタックが溢れることがない |
| AC-411 | 起動できない `command` | `''`、`'`、`""`、存在しない名前、ディレクトリ、`/` | `validate` / `run a` | 前 3 つは 64、残りは 127。異常終了しない |
| AC-412 | 入れ子の深い入力 | 深さ 1,000 の YAML と JSON(定義、ステート) | `validate` / `run a` / `convert` | 64 / 65 / 67。異常終了せず(`StackOverflowException` でプロセスが落ちず)、スレッドのスタックを大きくしなくても動く |
| AC-413 | 変換の頑健性 | 範囲外・空・壊れた・切り詰めた入力、ランダムな構造 | `convert` | 異常終了しない。成功した構造は往復で等しい |
| AC-414 | 内部の誤り | 異常終了を起こす | (入口) | 4.13 節の「内部の誤りの文」と同じ(AC-368)。実際のプロセスでの検査は AC-490、AC-491 |
| AC-415 | 検証は副作用を持たない | 4.1 節の最後の項目 | — | AC-057 |
| AC-416 | 入力の値を表示に出さない | 各種類のエラー(定義・`command`・ステート・変換・使い方)に、秘密のつもりの文字列を含む入力。YAML ライブラリが、入力の断片を含むメッセージの例外を投げる入力(例: 不正な文字を含む行、閉じていない引用符の行)も含む | 各コマンド | ERR に、その値が現れない(キー名は、エスケープして現れてよい)。`command` の誤りの文に `command` の中身が現れない。例外のスタックトレース(`   at ` で始まる行、`Unhandled exception.`、`System.` で始まる例外の型名)が現れない |
| AC-417 | ステートに機微な値を書かない | 4.3 節の項目 | — | AC-148 |
| AC-418 | 変換に副作用がない | 4.7 節の項目 | — | AC-235。加えて、`Conductor.Convert` が `System.Diagnostics.Process`・ネットワークの型を使わず、ファイルを開くのが入力ファイルの読み取りの入口だけであること(点検) |
| AC-419 | ネットワークに接続しない | 実装と依存 | 点検(ソースの `using` と完全修飾名の走査。依存のパッケージの、`System.Net` を使うかの調査) | 製品のコードが `System.Net`・`System.Net.Http`・`System.Net.Sockets`・`System.Net.Security` を使わず、`HttpClient`・`Socket`・`WebRequest` を使わない。実行時の依存のソースにも、実行中に通信するコードがない(YAML ライブラリは未確認)。区分: 点検 |
| AC-420 | OS 固有コードの隔離と、動的実行・ネイティブの限定 | ソース | 点検(文字列の走査) | `OperatingSystem.Is*`、`RuntimeInformation.IsOSPlatform`、`RuntimeInformation.OSDescription`、`#if WINDOWS`・`#if UNIX` の参照が、`src/Conductor.Runtime/Runner/PlatformInfo.cs` の外にない。`System.Reflection.Emit`、`Assembly.Load*`、`Activator.CreateInstance`、`Type.GetType(string)`、`AssemblyLoadContext`、`BinaryFormatter` が、どこにもない(0 か所)。`DllImport`・`LibraryImport` が、07 章が許す関数(`kill` など)の宣言だけで、`PlatformInfo.cs` と `Conductor.Runner` の限られたファイルにだけある。`System.Threading.Thread` の生成・`ThreadPool`・`Task` の使用は、07 章の所有者・終了条件の表にある箇所だけ(許可の一覧)。区分: 点検 |
| AC-421 | 異常終了を起こす書き方がない | ソース、`.editorconfig` と `BannedSymbols.txt` | 点検(文字列の走査と、アナライザー) | 製品のコード(`src/`)に、(1) `Environment.Exit(`・`Environment.FailFast(`(`Program.cs` の 1 か所以下を除く。AC-500)、(2) `Debug.Assert`・`Trace.Assert`(検証に使わない)、(3) 外部の入力の経路での `!`(null 許容の抑制演算子)と `dynamic`、(4) `#pragma warning disable`・`[SuppressMessage]`・`<NoWarn>` による無効化(理由つきの許可の一覧にあるものを除く)、(5) `System.Text.Json` の `JsonSerializer`・`JsonDocument.Parse`(自前の読み取り `Conductor.Json` を使う。決定は 06 章。ステートの書き出しは、自前の書き出し、または、06 章が許す範囲)、(6) YAML ライブラリの `Deserializer`・`DeserializerBuilder`・`StaticDeserializerBuilder`(AC-531)、(7) `catch { }`・`catch (Exception)` で黙って捨てる書き方、(8) `Console.Write*`・`Console.Out`・`Console.Error`(出力は `Conductor.Report` と、標準出力・標準エラーの `Stream` を通す。AC-520)、(9) `Task.Wait`・`.Result`・`.GetAwaiter().GetResult()`、(10) `Thread.Abort`、(11) `checked` の演算(`ulong` の飽和は `Math.BigMul`・`UInt128`)がない。`Exception` の捕捉は `Program.cs` と、07 章が許す箇所だけ。配列・辞書の添字は、`IndexOutOfRangeException`・`KeyNotFoundException` を起こしうる箇所で、先に検査する。区分: 点検 |
| AC-422 | テスト専用の経路を持たない | ソース、配布物 | 点検 | 入口に、異常終了を起こす経路や、テスト用の環境変数・隠しオプションがない。`src/` が `tests/` を参照しない。`Conductor.Cli` に `InternalsVisibleTo` を使う場合は、`Conductor.Tests` と `Conductor.TestJob` だけ。配布物のファイル一覧に、`tests/`・`tools/`・`demo/`・設定ファイルが含まれない(含まれるのは、実行ファイルと `LICENSE`。AC-526)。テスト用ジョブ `tj` が入っていない |
| AC-423 | 著作権表示 | ソース | 点検 | 各 `.cs` ファイルの先頭(`// <auto-generated>` の行があればその次)に Apache License 2.0 のヘッダ(`Copyright 2026 agwlvssainokuni`)がある。`.csproj`・`.props`・`.yml`・`.sh`・`.cmd` も同じ(コメントの書式は各言語に合わせる)。`bin/`・`obj/`・`tests/Conductor.Tests/Fixtures/` は対象外。区分: 点検 |

### 4.17 デモの受け入れテスト

デモの仕様は 7 章。実行は、`CONDUCTOR=<被試験体>`(`conductor` の実行ファイルのパス)、`DEMO_LANG=en_US.UTF-8`、`DEMO_SLEEP=0`、一時ディレクトリの親(UNIX 系は `TMPDIR`)を空のディレクトリ `D` にして行う。標準入力は空。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-424 | UNIX 系: スクリプトが全部通る | 上の設定 | `sh demo/run-demo.sh` | 終了 0。見出し `=== 0. ` から `=== 10. ` までが順に出る。区分: UNIX 系 |
| AC-425 | UNIX 系: 場面 7(検査) | 同上 | 同上 | 出力に `definition is valid`、`command has an unclosed quote`、`→ 終了コード 64` が現れる。誤りの定義の検査のあと、ステートの一覧が `(なし)` |
| AC-426 | UNIX 系: 場面 4(再開・完了) | 同上 | 同上 | `completed` の行、`skipped (already executed)` が現れる |
| AC-427 | UNIX 系: 場面 8(run ID) | 同上 | 同上 | `conductor.json.build-1.state` が一覧に現れる。再開のあと `completed`。`reset` のあと一覧は `(なし)` |
| AC-428 | UNIX 系: 場面 9(環境変数と後始末) | 同上 | 同上 | `conductor.json.opt-1.state` が現れ、最後の `watch` が `no state` |
| AC-429 | UNIX 系: 場面 10(リトライ) | 同上 | 同上 | `conductor: retry: job 'flaky' exited with code 75; retry 1 of 3 in 1 seconds` と `… retry 2 of 3 in 1 seconds` の 2 行、`[flaky] 3 回目の実行`、`job=flaky result=success` が現れる |
| AC-430 | UNIX 系: 利用者の環境に左右されない | `CONDUCTOR_RUN_ID=zzz`、`DEMO_RUN_ID=zzz`、`LC_ALL=ja_JP.UTF-8`、`LC_MESSAGES=ja_JP.UTF-8`、`NO_COLOR=1`、`DOTNET_SYSTEM_GLOBALIZATION_INVARIANT=0`、`DOTNET_gcServer=1` を設定 | 同上 | 出力に `zzz` が現れない。英語の表示(`DEMO_LANG` のとおり)。結果が上と同じ |
| AC-431 | UNIX 系: 何も残さない | 同上 | 同上 | 終了後、一時ディレクトリの親 `D` に何も残らない。リポジトリの作業ツリーは変わらない(`bin/`、`obj/` などを除く。`.gitignore` の対象) |
| AC-432 | Windows 版: 定義 | 定義 `demo\windows\conductor.json` | (検査) | 4 ジョブ(`extract`、`transform`、`load`、`notify`)と、`extract` の警告の遷移(`notify` へ)を持つ。`validate` が 0。区分: Windows |
| AC-433 | Windows 版: 場面 1〜2 | 一時ディレクトリに定義とスクリプトを写す | `pipeline nightly`(場面 1)、`DEMO_EXTRACT_EXIT=1` で `pipeline nightly`(場面 2) | 場面 1 は 0。場面 2 は `notify` へ進み 0 |
| AC-434 | Windows 版: 場面 3〜4 | 同上 | `DEMO_LOAD_EXIT=2` で `pipeline`(2)、続けて `pipeline`(`load` から再開) | 2 のあと、再実行で `load` から再開して 0 |
| AC-435 | Windows 版: 場面 5〜7 | 同上 | `run extract`、`reset`、`convert conductor.json --to yaml`、`validate`、誤りの定義の `validate` | `run`・`reset`・`convert`・`validate` は 0。誤りの定義は 64 |
| AC-436 | Windows 版: 場面 8〜9 | 同上 | run ID を使う一連(場面 8、9) | run ID ごとにステートが分かれる。`--run-id` が環境変数に勝つ。`reset --run-id` と `--all` が働く |
| AC-437 | Windows 版: 場面 10 | 同上 | `pipeline --config retry.json retry-demo` | 2 回失敗してから成功する |
| AC-438 | Windows 版: 利用者の run ID に影響されない | 利用者の run ID の環境変数を設定 | 同上 | 結果が変わらない |

### 4.18 リリース物の受け入れテスト

リリースの仕様は 6 章。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-439 | 成果物の名前と形 | リリース | (成果物の一覧) | RID ごとの 4 つ(`conductor-<版>-linux-x64.tar.gz`、`conductor-<版>-linux-arm64.tar.gz`、`conductor-<版>-osx-arm64.tar.gz`、`conductor-<版>-win-x64.zip`)と、`SHA256SUMS` の計 5 つがある。区分: 点検 |
| AC-440 | 展開の構造 | 各アーカイブ | 展開(`tar`、`System.IO.Compression`) | 最上位に、`conductor-<版>-<RID>/` の 1 つのディレクトリだけがあり、その中が、実行ファイル(`conductor`、Windows は `conductor.exe`)と `LICENSE` の 2 つだけ(`.pdb`・`.dll`・`*.json`・`tests/` を含まない)。区分: 点検 |
| AC-441 | チェックサム | `SHA256SUMS` | `sha256sum -c`(macOS は `shasum -a 256 -c`)の形式で検査 | 4 つのアーカイブの行があり、検査が通る。区分: 点検 |
| AC-442 | 版の表示 | 各アーカイブを展開した結果(実行できる OS で) | `conductor --version` | 0。OUT = `conductor <版>`。区分: 点検(3 OS。linux-arm64 は、実行できる環境がなければ、未実施を記録する。6.3 節) |
| AC-443 | 実行時の依存が 1 つ | `Directory.Packages.props`、各プロジェクトファイル、`packages.lock.json` | 点検 | 製品のプロジェクトの `PackageReference` が YAML ライブラリ 1 つだけ(開発時のアナライザーを除く。`PrivateAssets=all`)。ロックファイルの直接の依存と、推移的な依存の一覧に、ネイティブのライブラリのパッケージがない。区分: 点検 |
| AC-444 | タグの検査 | タグ `v<版>` | リリースの流れ | タグが `main` 上にない、または版が `Directory.Build.props` の `Version` と違うと失敗する(タグが `main` 上にない旨 / タグと版の不一致の旨)。区分: 点検 |
| AC-445 | 下書きとプレリリース | リリースの作成 | (作成されたリリース) | 下書きである。版が先行版(SemVer の `-` を含む。例: `0.9.0-rc.1`)ならプレリリース。注記に固定の 1 文がある。区分: 点検 |
| AC-446 | 3 OS で受け入れテストが通る | Linux・macOS・Windows の実機(または同等の実行環境) | 4 章の自動の項目(各 OS で行える項目)。`CONDUCTOR_BIN` に、AC-442 で展開した実行ファイルを渡す形でも通ること | 全項目が合格する(OS の区分の項目は、その OS で)。区分: 自動 |

### 4.19 JSON と YAML に固有の項目

定義の形式(JSON・YAML)、ステートの形式(JSON)、変換(JSON ⇔ YAML)の、形式固有の落とし穴を確かめる。特に記さない限り、「64」の期待は 4.1 節と同じ(終了 64、OUT 空、ステートなし、ジョブを起動しない。ERR の最初の行は `conductor: error: <ファイル>:<行>: ` で始まる(行がわかる誤り)。種類の文は [03](03-definition-format.md)・[08](08-output-and-i18n.md) のとおり)。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-450 | YAML の定義 | `P(p; a:0)` を YAML で書いた `T/conductor.yaml` | `validate --config conductor.yaml`、`run a --config conductor.yaml` | 0 / 0。ステートは `T/conductor.yaml.state`。JSON の定義と同じ結果。`--config` を省くと、`conductor.json` を探す(`conductor.yaml` は探さない) |
| AC-451 | `.yml` も受理 | `T/conductor.yml`(`B0` の YAML) | `run a --config conductor.yml` | 0。ステートは `T/conductor.yml.state` |
| AC-452 | 拡張子が違うファイルは拒否 | `B0` の内容を、`conductor.xml`、`conductor.txt`、拡張子なしの `conductor`、`conductor.json.bak` に置く(ファイルが存在しないパス `nofile.txt` も) | `validate --config <ファイル>` | 64。ERR の最初の行は `conductor: error: <ファイル>: unsupported definition file format: <拡張子の説明>`(行なし。理由の字句は [03](03-definition-format.md)。拡張子は `.json`、`.yaml`、`.yml`)。大文字の拡張子 `conductor.JSON`・`conductor.YAML` は受理(0)。ステートなし。ジョブは起動されない(`convert` の入力は拡張子を見ない。AC-209。`Path.GetExtension` の細部は AC-545) |
| AC-453 | YAML のタグを拒否 | `command: !!str "TJ exit 0"`、`id: !foo a`、`name: !!binary aGk=` | `validate --config conductor.yaml` | 64(それぞれ別の定義)。行は、そのタグの行。タグの有無は、構文要素のタグの属性で判定する(明示のタグがあるときだけ値を持つ。**採用したライブラリの属性の意味は、実装時に確認する。未確認**) |
| AC-454 | YAML の複数の文書を拒否 | `B0` の内容 + `---⏎` + もう 1 つの文書 | `validate --config conductor.yaml` | 64。行は 2 つ目の文書の始まり。先頭に `---` が 1 つだけある `B0` は受理(0) |
| AC-455 | YAML の文字列でないキーを拒否 | `1: x`、`true: x`、`? [a, b]⏎: x`(トップレベルまたは `job` の中) | `validate --config conductor.yaml` | 64 |
| AC-456 | YAML の重複キー(入れ子) | `job` の要素に `id` を 2 回、`pipeline` に `name` を 2 回 | `validate --config conductor.yaml` | 64。`syntax error`。行は 2 回目のキーの行 |
| AC-457 | YAML のアンカー・エイリアス・マージキーは展開して受理 | 既知のキーの中に書く。(a) 1 つ目のジョブに `warning_exit_codes: &w [3]`、2 つ目に `warning_exit_codes: *w`。(b) 遷移 `- <<: {on: success, next: "@end"}`。(c) トップレベルに `defaults: &d {…}` という未知のキー | `validate`、`run`(YAML の定義) | (a)(b) は 0。展開した値で動く(2 つ目のジョブの警告コードは `[3]`、(b) は `@end` への遷移)。(c) は 64(`unknown key`。アンカーの置き場所のための未知のキーは作れない)。整合: 10 章 |
| AC-458 | 数値・真偽値に見えるジョブID | `id: 1`、`id: true`、`id: "1"`、`id: yes`、`id: on`、`id: 0x10`(YAML)。JSON は `"id": 1`、`"id": true`、`"id": "1"` | `validate`(それぞれ別の定義) | `id: 1`・`id: true`・`id: 0x10`・JSON の `1`・`true` は 64(`wrong value type`)。`id: "1"`・`id: yes`・`id: on`・JSON の `"1"` は 0(文字列。YAML 1.2 のコアスキーマの規則。`on`・`yes` を真偽値として読まない。型は自前で決める。AC-525) |
| AC-459 | `command` の書き方(YAML のブロック・折り返し・複数行) | `command: \|⏎  TJ --out O echo a⏎  b⏎`(ブロック)、`command: >-⏎  TJ --out O echo⏎  a b`(折りたたみ)、複数行の plain スカラー(`TJ` は、各書き方に合わせて語を書く) | `run a --config conductor.yaml` | 0。`O` の引数は、それぞれ `a`、`b` / `a`、`b` / 同様(改行・空白は区切り) |
| AC-460 | 定義の中の null の扱い | YAML: `name: ~`、`command: null`、`warning_exit_codes: ~`、`retry: null`、`remove_on_complete:`(値なし)。JSON: `"command": null`、`"name": null` | `validate` | 64。`name`・`command`・`warning_exit_codes`・`remove_on_complete`・JSON の `null` は `wrong value type`。`retry: null` は `retry must be a map`。未知のキーの値が `null` なら `unknown key`([03](03-definition-format.md))。null 許容参照型の `null` と「キーがない」を区別する(`TryGetValue` が false の場合と、値が null の場合を、混同しない) |
| AC-461 | JSON の重複キー | `{"format_version": 1, "format_version": 1, ...}`(`conductor.json`)、`job` の要素の `id` の重複 | `validate` | 64。`syntax error`(理由 `キーが重複しています`)。重複の検出は、自前の読み取りで行う([03](03-definition-format.md))。`JsonDocument.Parse` の重複キーの扱いは、公式の文書で確認できていない(未確認)。位置(行・列)は、`Utf8JsonReader` の `TokenStartIndex` と、改行の数から求める(AC-475)。位置の取得の公式の記述は未確認 |
| AC-462 | JSON のコメントを拒否 | 行コメント `// c`、ブロックコメント `/* c */` を含む定義 | `validate` | 64。`syntax error`(`JsonReaderOptions` の `CommentHandling` を `Disallow` にする。既定の値かは未確認なので、明示する) |
| AC-463 | JSON の末尾のカンマを拒否 | `[1,]`、`{"a": 1,}`(定義の中) | `validate` | 64。`syntax error`(`AllowTrailingCommas = false` を明示する) |
| AC-464 | JSON の非標準の書き方を拒否 | 一重引用符の文字列、`NaN`、`Infinity`、`-Infinity`、先頭が 0 の数(`01`)、引用符のないキー、値のあとの余分なデータ(`{...} {...}`、`{...} x`) | `validate` | 64。`syntax error`(それぞれ別の定義。`Utf8JsonReader` は、値のあとの余分なデータをどう扱うか(`isFinalBlock` と、読み終えたあとの検査)を、自前で検査する。公式の記述は未確認) |
| AC-465 | JSON の最上位がオブジェクトでない | `[]`、`"x"`、`1`、`null`、`true` | `validate` | 64。`wrong value type`(理由 `最上位はマップで書きます`。JSON の `null` も同じ。[03](03-definition-format.md)) |
| AC-466 | JSON の大きさの上限 | 正しい JSON の定義の末尾に、空白を足して 1,048,576 バイト / 1,048,577 バイト | `validate` | 0 / 64(`definition file is too large`) |
| AC-467 | JSON の文字列のエスケープ | `"command": "TJ --out O echo \"a b\" c A 😀"`(`TJ` は 1 つの語)、不正なエスケープ `"\x"`、閉じていない文字列、制御文字(生の LF)を含む文字列 | `run a`(正しいもの)、`validate`(誤り) | 正しいもの: `O` の引数 = `a b`、`c`、`A`、`😀`(サロゲートペアのエスケープが 1 つのコードポイントになる)。誤りは 64(`syntax error`)。孤立したサロゲートのエスケープ(`"\ud800"`)の扱いは 03 章に従う(`Utf8JsonReader.GetString` が、これをどう扱うかは未確認)。受理する場合でも、出力で U+FFFD に化けず、エスケープされる(AC-519) |
| AC-468 | JSON の数値の型 | `"format_version": 1.0`、`1e0`、`"warning_exit_codes": [3.0]`、`[1e1]`、`[99999999999999999999]`、`[true]`、`[NaN]` | `validate` | `format_version` は 64(`unsupported format_version`)。終了コードは 64(`wrong value type`)。整数のリテラルだけを整数と読む(`double` を経由して丸めた値を見ない。`GetInt64`・`TryGetInt64` の成否だけに頼らず、字句の形(小数点・指数の有無)を検査する。AC-502) |
| AC-469 | 変換の JSON の出力の文字 | YAML: `a: "<&>"⏎b: 夜間⏎c: " x"` | `convert in.yaml --to json` | 0。OUT に `"<&>"` がそのまま(`\u0026` や `&` にならない。.NET の `JsonEncoder` の既定は、`<`・`>`・`&` を `\u003C` などにエスケープすると記憶している(未確認)ので、出力は自前の書き出しで行う(09 章))と `夜間`(非 ASCII はそのまま)がある。OUT を JSON として読んだ木が入力の木と等しい(`c` の U+2028 は、`\u2028` の書き方でも直接でもよい。09 章に従う) |
| AC-470 | 変換の YAML の出力: 予約語に見える文字列 | JSON: `{"a": "true", "b": "1", "c": "null", "d": "@end", "e": "", "f": "~", "g": "on", "h": "1e3", "i": "0x1F", "j": "yes", "k": "-", "l": "a: b", "m": "# c", "n": "n", "N": "N", "y": "y", "Y": "Yes", "o": "Off"}`(`y`・`n` は YAML 1.1 の型の表の真偽値の語。キー `n`・`y` も含む) | `convert in.json --to yaml` の OUT を `convert --to json` に通す。さらに OUT の各スカラーを、テスト用の YAML 1.1 の真偽値の語の表(`y`・`Y`・`yes`・`Yes`・`YES`・`n`・`N`・`no`・`No`・`NO`・`true`・`True`・`TRUE`・`false`・`False`・`FALSE`・`on`・`On`・`ON`・`off`・`Off`・`OFF`。テストの中に表を持つ)と照らす | 往復の木が入力と等しい(すべて文字列のまま。YAML の出力は自前の書き出しで、必要なものを引用符で囲む)。OUT で、値・キーの `y`・`n`・`N`・`Yes`・`Off`・`on`・`yes` が二重引用符で囲まれている(09 章の規定)。YAML のパーサーの構文要素で読むと、これらのスカラーの書式が `Plain` でない(引用符つき)。区分: 自動 |
| AC-471 | 変換の YAML の出力: 特殊な文字列 | JSON: 複数行の文字列(`"a\nb\n"`)、先頭・末尾に空白のある文字列、タブ、`"\u0007"`(制御文字)、非 ASCII、絵文字、長い文字列(1 万文字)、空のキー `""`、`@` や `%` や `` ` `` で始まるキー | `convert in.json --to yaml` の OUT を `convert --to json` に通す | 往復の木が入力と等しい |
| AC-472 | 変換の JSON の入力の数値 | JSON: `{"a": 1, "b": 1.0, "c": 1e2, "d": -0, "e": 9223372036854775807, "f": -9223372036854775808}` | `convert in.json --to yaml` の OUT を `convert --to json` に通す | 型が変わらない(`a`・`d`・`e`・`f` は整数、`b`・`c` は浮動小数)。往復の木が入力と等しい。`e`・`f` は桁を落とさない |
| AC-473 | ステートの JSON 固有の異常 | ステートが、重複するキーを持つ JSON、値のあとに余分なデータ、最上位が配列 / `null` / 文字列、`{}` | `run a` | 65(ジョブは起動されない)。余分なデータ・最上位の型の誤りは `state file is corrupt: JSON として読めません`、キーの重複は `state file is corrupt: キーが重複しています`、`{}` は `state file is corrupt: format_version がありません` |
| AC-474 | ステートの数値の型 | `exit_code` が `3.0` / `"3"` / `1e1` / `256` / `-1` / `true` / `NaN`、`format_version` が `1e0` | `run a` | 65。`state file is corrupt: …`(型・範囲の理由は 06 章) |
| AC-475 | `ME` の誤りの行(JSON と YAML) | `ME`(4.1 節。JSON の `conductor.json` と、YAML の `conductor.yaml`) | `validate`、`validate --config conductor.yaml` | 64。ERR は 4 行。JSON は順に `…:3: unknown key`、`…:11: wrong value type`、`…:16: unknown key`、`…:20: transition target job is not defined`。YAML は順に `…:2: …`、`…:10: …`、`…:13: …`、`…:16: …`(同じ種類の文)。JSON の定義の誤りの行は、自前の読み取りが、各キーと値の開始の位置(行)を記録して求める(LF の数で数える。CRLF でも同じ行番号。AC-038 と同様。[03](03-definition-format.md)) |
| AC-476 | 版の表示と、版の唯一の出所 | ソース、`Directory.Build.props`、実行ファイル | 点検と `--version` | 版の唯一の出所は `Directory.Build.props` の `<Version>`(例: `<Version>0.8.0</Version>`)。コードは、版の定数を持たず、実行時に `AssemblyInformationalVersionAttribute`(`Assembly.GetExecutingAssembly()` の `GetCustomAttribute`)から読む。ソースコントロールの識別子が版の文字列の末尾に `+<ハッシュ>` で付かないように、`Conductor.Cli.csproj` に `<IncludeSourceRevisionInInformationalVersion>false</IncludeSourceRevisionInInformationalVersion>` を書く(**このプロパティの名前と効果は、この文書では公式の文書を確認していない(未確認)**。`--version` の OUT が `conductor 0.8.0` と完全に一致することが、受け入れの条件)。`--version` の OUT = `conductor ` + `<Version>`。`Conductor.Checks version` が、`<Version>` を、タグ(`v` なし)と比べる(5.4 節)。区分: 点検 |
| AC-477 | .NET の版の記載の一致 | `global.json`、`Directory.Build.props`、`README.md`、`.github/workflows/ci.yml` | `dotnet run --project tools/Conductor.Checks -- consistency` | `global.json` の `sdk.version` の主版(例: `10.0.100` の `10`)が、`MAX_TFM`(`ConductorMaxTfm`。例: `net10.0`)の主版と一致する。`ConductorMinTfm`・`ConductorMaxTfm` が、ci.yml の `DOTNET_MIN_TFM`・`DOTNET_MAX_TFM` と一致する。README の「.NET SDK <`global.json` の版>」が一致する。`ConductorMinTfm` ≦ `ConductorMaxTfm`(同じでもよい)。製品のプロジェクトに、`TargetFramework` の直書きがない(`$(ConductorMinTfm)` を使う)。不一致なら失敗(終了 1)。区分: 点検 |
| AC-478 | 依存の整合 | `Directory.Packages.props`、各プロジェクトファイル、`packages.lock.json` | `dotnet restore --locked-mode`、`dotnet list package --include-transitive`、`dotnet run --project tools/Conductor.Checks -- deps` | 復元が成功し(ロックファイルと依存の定義が一致する)、`deps` が通る。製品のプロジェクトの実行時の依存が YAML ライブラリ 1 つだけ、開発時の依存が 8.3 節の許可の一覧の内。`PackageReference` に `Version` の属性がない(`Directory.Packages.props` だけで版を決める)。`ManagePackageVersionsCentrally` が `true`。すべてのプロジェクトに `packages.lock.json` がある。区分: 点検 |
| AC-479 | ビルドと発行ができる | ソース | `dotnet build -c Release`、`dotnet publish src/Conductor.Cli -c Release -r <RID> --self-contained -p:PublishSingleFile=true`(RID は、実行する OS に合うもの。Linux・macOS・Windows) | すべて成功し、発行の出力が `conductor`(Windows は `conductor.exe`)1 つだけ(AC-526)。`MIN_TFM` と `MAX_TFM` の両方で(別の SDK で)ビルドが通る。区分: 点検 |
| AC-480 | 静的検査 | ソース | `dotnet format --verify-no-changes --no-restore`、`dotnet build -c Release --no-restore -warnaserror` | どちらも終了 0(差分なし、警告なし。アナライザーの指摘を含む)。区分: 点検 |
| AC-481 | 既知の脆弱性の検査 | 依存 | `dotnet list package --vulnerable --include-transitive`、`dotnet list package --deprecated` | 結果を記録する。脆弱性・非推奨が見つかっても、マージはブロックしない(警告)。区分: 点検 |
| AC-482 | 変換した定義がそのまま使える | `CV1` を `conductor.json` に | `convert conductor.json --to yaml > conductor.yaml`、`validate --config conductor.yaml`、逆向きに `convert conductor.yaml --to json > c2.json`、`validate --config c2.json` | すべて 0。展開された木が同じ |
| AC-483 | 厳格な設定と、アナライザーの規則 | `Directory.Build.props`、`.editorconfig`、`BannedSymbols.txt` | 点検(XML の走査) | `Directory.Build.props` に `<Nullable>enable</Nullable>`、`<ImplicitUsings>disable</ImplicitUsings>`、`<TreatWarningsAsErrors>true</TreatWarningsAsErrors>`、`<EnforceCodeStyleInBuild>true</EnforceCodeStyleInBuild>`、`<AnalysisLevel>`(最新の値。実装時に確認)、`<AnalysisMode>`(`All` か、`Recommended` 以上。実装時に決める)、`<InvariantGlobalization>true</InvariantGlobalization>`(製品の実行ファイルのプロジェクト)、`<LangVersion>`(最新。`latest`)、`<Deterministic>true</Deterministic>`、`<ContinuousIntegrationBuild>`(CI で `true`)がある。`.editorconfig` に、命名の規則(2.4 節)と、`dotnet_diagnostic.<ID>.severity = error` の指定がある。`Directory.Packages.props` の `GlobalPackageReference` に `Microsoft.CodeAnalysis.BannedApiAnalyzers` があり、`BannedSymbols.txt` が `AdditionalFiles` として `src/` の全プロジェクトに入っている(9.2 節)。区分: 点検 |
| AC-484 | 合図とジョブの終了の順序と、シグナルの登録の解除 | `Conductor.Runner` と `Conductor.Cli` の単体(偽の `Deps` と、本物のシグナルの登録。UNIX 系) | 合図を、ジョブの終了の直前・直後・待機(リトライ)の直前に立てる。同じシグナルを続けて 2 回立てる。`run` / `pipeline` が終わったあと、登録が解除されていることを調べる | 最初の番号だけが残る。転送は 1 回。終了済みのジョブには送らない。`run` / `pipeline` の終了後(正常・失敗・例外のいずれでも)に、登録した `PosixSignalRegistration` がすべて `Dispose` されている(`using` か `try`/`finally`。登録が漏れて、後続のテストに影響しない。偽の `FakeSignals` が、登録の数と解除の数を数える)。区分: 単体 |
| AC-485 | 偽の `FakeSignals` で登録に失敗すると 70 | 偽の `FakeSignals`(登録が失敗を返す) | `run a`、`pipeline p`(定義は正しい / 誤りがある) | どちらも 70。ERR = `conductor: error: failed to set up signal handling`。定義の検査(定義が誤りでも 64 にならない)もジョブの起動もしない。区分: 注入。(AC-298 を、定義が誤りの場合まで広げたもの) |
| AC-486 | Windows の `.cmd` / `.bat` の禁止文字は起動しない | Windows。`T/echo-args.cmd`(内容 `@echo off` と `exit /b 0`)。`cmd` = そのパス + 引数 1 つ。引数に `"`、`%`、`^`、`&`、`\|`、`<`、`>`、`!`、`(`、`)`、改行のどれか 1 つを含める([07](07-process-and-signals.md)。1 文字ずつ別の定義)。配列形式でも同じ | `run a` | 127(起動失敗)。ERR = `conductor: error: job 'a': cannot start command '…'`。`T/injected.txt` ができない。含まない引数(`plain`)は起動される(AC-115)。区分: 自動(Windows の CI) |
| AC-487 | Windows の Ctrl+C のチェックリストの記録 | AC-301 の記録欄 | リリース前に、Windows の実機で AC-301 を行い、結果を `tests/WINDOWS_CTRL_C_CHECKLIST.md` の記録欄に書く | 記録欄に、実施日、OS の版、.NET の版、(1)〜(4)の結果が書かれている。ない場合はリリースの前に実施する。区分: 手動(Windows) |

### 4.20 .NET に固有の項目

.NET の実行モデル(ランタイムの起動と JIT、単一ファイルの配布物、`PosixSignalRegistration` と `Console.CancelKeyPress`、`Process` の性質、`string` が UTF-16 であること、文化依存の API、`InvariantGlobalization`、整数の型と `checked`、`StackOverflowException`、既定の `ExitCode` の扱い、`Environment.Exit` と `finally`)に固有の落とし穴を確かめる。注入のための実行ファイル(`TJ` の `inject-*` モード)は、`Conductor.Cli.EntryPoint.Run` を呼ぶ(3.4 節、3.6 節)。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-488 | 起動時間 | 配布物(単一ファイル)と、`dotnet conductor.dll`(フレームワーク依存) | `--version` を 20 回(起動から終了まで。最初の 5 回を捨てる) | 単一ファイル: p95 が 250 ms(Linux・macOS)、500 ms(Windows)以下(仮。目安。2.1 節)。`dotnet conductor.dll` は別に記録する。ReadyToRun・トリミング・圧縮の有無での差は、未確認(測る場合は、記録に構成を明記する)。区分: 計測 |
| AC-489 | YAML ライブラリを読み込まない経路 | 配布物と、単体(プロセス内) | (単体)`EntryPoint.Run` を `--version`・`--help`・使い方の誤り・`reset`・`watch` で呼んだあと、`AppDomain.CurrentDomain.GetAssemblies()` に YAML ライブラリのアセンブリがない。(外から)環境変数 `DOTNET_` の診断(アセンブリの読み込みのログ)は使わず、単体で確かめる | YAML のアセンブリが読み込まれていない(`validate`・`run`・`pipeline`・`convert` の YAML の入力を読むときだけ、読み込まれる)。型の読み込みが、JIT のタイミングで起きる点に注意する(2.1 節)。単体のテストは、`process-global` の収集の中で、新しい `AssemblyLoadContext` ではなく、別のプロセス(`TJ` の `inject-assemblies` モード)で行う(同じテストのプロセスの中では、他のテストが先に読み込んでいるため)。区分: 単体と計測 |
| AC-490 | 未捕捉の例外は 70(UNIX 系) | `P(p; a:0{cmd="TJ sleep 1"})`。`TJ inject-alarm run a` で起動する。`inject-alarm` は、`Conductor.Cli.EntryPoint.Run` の前に、タイマーで、別のスレッドから `InvalidOperationException("SECRET-INJECT")` を投げる(スレッドプールのタイマーのコールバックで、未処理の例外が起きる。`AppDomain.UnhandledException` に届く) | `run a` | 終了 70。ERR = `conductor: error: internal error`(1 行だけ。末尾の改行 1 つ)。ERR に `SECRET-INJECT`・スタックトレース(`   at `、`Unhandled exception`)が現れない。OUT 空。`AppDomain.CurrentDomain.UnhandledException` のハンドラが働く(`Environment.Exit(70)` ではなく、ハンドラが、標準エラーへ書いて flush したあと、プロセスを終える方法は、実装時に確認する。**ハンドラのあとで、ランタイムが続けて終了させる動作と、終了コードの扱いは、公式の文書で確認していない(未確認)**。ジョブの子プロセスが残る場合の扱いは 07 章に従う) |
| AC-491 | `UnobservedTaskException` と、シグナルのコールバックの例外も 70 | (a) `TJ inject-unobserved`: `Conductor.Cli` を呼ぶ前に、`Task` を作って未観測の例外で終えさせ、`GC.Collect()` と `GC.WaitForPendingFinalizers()` を行う(`TaskScheduler.UnobservedTaskException` は、ファイナライザで発火する)。(b) 偽の `FakeSignals`(合図のコールバックが例外を投げる)と `run` / `pipeline`(注入) | (a) `TJ inject-unobserved --version`。(b) 偽の部品の上で `run a` / `pipeline p` | (a) `conductor: error: internal error`(1 行だけ)。`SECRET-INJECT` が現れない。製品が `TaskScheduler.UnobservedTaskException` に登録していることが前提(登録は入口で行う。決定 13)。(b) 70。ERR = `conductor: error: internal error`。ジョブのあとのステートの書き込みを、例外のために飛ばさない(例外が起きる前の確定したステートは残る)。区分: (a) 自動、(b) 注入 |
| AC-492 | シグナルのコールバックはメインスレッド以外で実行される | `Conductor.Runner` のシグナルの準備の部品(UNIX 系) | 登録して、シグナルを送り、コールバックの中で `Environment.CurrentManagedThreadId` を記録する。メインスレッドの ID と比べる | コールバックのスレッドが、メインスレッドと異なる(確認できなければ、確認できた事実を 10 章に書く)。コールバックの本体は、旗の書き込み(`Volatile.Write`・`Interlocked`・`ManualResetEventSlim.Set`)だけで、ロックの取得・入出力・例外の送出を含まない(AC-499)。区分: 単体(UNIX 系)。**シグナルの準備に失敗(`PosixSignalRegistration.Create` が例外を投げる)すると、「失敗」を返し、`run` は 70(ERR = `conductor: error: failed to set up signal handling`)。ジョブは起動されない(AC-298、AC-485)** |
| AC-493 | 標準出力のパイプが閉じた(`IOException`) | 入力 `big.json`(約 1 MiB。小さな要素の配列。`--to yaml` の出力が 64 KiB を大きく超える) | `convert big.json --to yaml` を起動し、標準出力の最初のデータを受けた時点で、読み手側(`process.StandardOutput.Close()`)を閉じる | 終了 67。ERR に `conductor: error: cannot write to standard output`。`IOException`・スタックトレースが現れない。70 にならない(.NET が `SIGPIPE` を無視して、`IOException` として届ける、と記憶している。**公式の文書で確認していない(未確認)**)。5 秒以内に終わる |
| AC-494 | 標準出力の取りこぼしがない | (a) `convert big.json --to yaml`(出力が 64 KiB を超える)。(b) `P(p; a:0{cmd="TJ big 16"})` | (a) 標準出力をパイプにし、読み手が最初の 500 ms 読まない。(b) `run a`(ジョブが 16 MiB を、`conductor` の標準出力に継承されたパイプへ書く) | (a) 終了 0。OUT の全体が、同じ変換を標準出力をファイルにして得た結果と、バイト単位で等しい(終了前に `Flush()` して書き込みの完了を待つ)。(b) 終了 0。OUT のバイト数がちょうど 16 MiB(`x` のみ)。`conductor` が出力を溜め込まない(子の標準入出力を `Redirect*=false` のままにして継承する) |
| AC-495 | 標準出力の書き込みの失敗(`/dev/full`。Linux) | Linux。標準出力を `/dev/full` を開いたファイルにして `watch`(ステートあり)、同様に `convert …`(`sh -c 'exec conductor … > /dev/full'` の形で起動する。テストの補助だけがシェルを使う) | 実行 | `watch` は 65、`convert` は 67。ERR = `conductor: error: cannot write to standard output`。70 にならない。終了時の flush が再び失敗して、終了コードが変わったり、例外が出たりしない(AC-496)。区分: Linux |
| AC-496 | 終了時の flush の失敗で終了コードが変わらない | UNIX 系。`TJ inject-delay <引数>`(`inject-delay` は、`Thread.Sleep(500)` のあとで `EntryPoint.Run` を呼ぶ)。標準出力を `Process` で起動して、起動の直後に読み手を閉じる | `--version`、`--help`、`watch`(ステートあり)、`validate -v`(標準エラーが閉じた場合も) | 終了コードが、`--version`・`--help` は 0(08 章に従う)、`watch` は 65。ERR に例外・スタックトレースが現れない。標準出力の `Stream` を、`using`/`Dispose` や `AutoFlush` のファイナライザ経由で閉じるときに、例外が漏れない(閉じる処理も `try`/`catch`)。区分: UNIX 系 |
| AC-497 | 終了後にプロセスが残らない | 各サブコマンド(`validate`、`run`、`pipeline`、`watch`、`reset`、`convert`、`--help`、`--version`、使い方の誤り) | 標準入力を開いたまま(閉じない)実行 | 出力の完了から 1 秒以内にプロセスが終わる(標準入力の読み取り、前景スレッド、タイマー、シグナルの登録、`ManualResetEventSlim` の待ちが残ってハングしない)。`Thread` を作る場合は `IsBackground = true`。区分: 自動 |
| AC-498 | ビルドの警告が 0 件、実行時に未処理の表示が出ない | 4.1〜4.9 節の各種の実行(特に、YAML の誤り・`convert`・リトライの待機・ジョブの起動失敗) | `dotnet build -c Release -warnaserror`(`MIN_TFM` と `MAX_TFM` の両方。3 OS)。実行は、通常の実行 | ビルドの警告が 0(アナライザーの指摘を含む)。ERR に `Unhandled exception.`・`Warning`・`warn:` を含む行が現れない。終了コードと出力が、通常の実行と同じ。区分: 自動と点検 |
| AC-499 | シグナルの登録の点検 | ソース | 点検(文字列の走査) | `PosixSignalRegistration.Create`・`Console.CancelKeyPress` の使用が、`src/Conductor.Runtime/Runner/` のシグナルの部品にしかない。登録するシグナルが、`PlatformInfo` が返す一覧のシグナルだけ(UNIX 系は `SIGHUP`・`SIGINT`・`SIGQUIT`・`SIGTERM`、Windows は `Console.CancelKeyPress` の `ControlC`・`ControlBreak`。07 章)。登録の戻り値(`PosixSignalRegistration`)を、フィールドに保持し、終了の処理(`finally`)で `Dispose` している(保持しないと、ファイナライザが登録を解除する。1 章。AC-537)。コールバックの本体が、フラグの書き込み(`Volatile.Write`・`Interlocked.*`・`ManualResetEventSlim.Set`)と、`PosixSignalContext.Cancel` の設定だけで、入出力・`Console`・`Process`・`lock`・`throw` を含まない(AC-538、AC-539)。`System.Threading.Timer` を製品が使わない(使う場合は 07 章の決定に合わせて、許可の一覧に加える)。区分: 点検 |
| AC-500 | 終了の呼び出しの点検 | ソース | 点検(文字列の走査) | `Environment.Exit`・`Environment.FailFast` の呼び出しが、`src/Conductor.Cli/Program.cs` に 1 か所以下で、ほかのファイルにない。`Program.Main` は、終了コードを**返す**(`int`)。標準出力・標準エラーの `Flush` が、その前に済んでいる(`Environment.Exit` は `finally` を実行しないので、`Main` の戻り値で終わる経路を基本とする)。`Environment.Exit` を使うのは、`UnhandledException` のハンドラの中で、標準エラーへ書いて `Flush` したあとだけ(使う場合)。区分: 点検 |
| AC-501 | 64 ビット符号なし整数の飽和演算 | `Conductor.Flow` の単体 | 待ち時間を求める関数に、検査を通らない極端な値を直接渡す(`prev = 1UL << 63`、`prev = ulong.MaxValue`、`multiplier = 10`、`cap = ulong.MaxValue`)。`max` = 100、`interval` = 86400、`multiplier` = 10、`max_interval` = 86400 の列 | 結果が `ulong` で、`ulong.MaxValue` を超えず(`Math.BigMul(a, b, out low)` の上位が 0 でなければ `ulong.MaxValue` に飽和。または `UInt128` で計算して比べる)、折り返し(`0` に戻る)がなく、`checked` による `OverflowException` も起きない。`double` に直して比べない(`double` の精度で桁が落ちる)。列は 86400 のまま 100 回続く。待ちの秒数を、待ちの 1 区間の長さ(`int` のミリ秒)に直す関数は、AC-503 の範囲に収まる値だけを返す(`(int)` のキャストで負の値や 0 に化けない)。区分: 単体 |
| AC-502 | 整数の検査: 真偽値・小数・指数・`NaN`・巨大な整数 | 定義(YAML・JSON): `format_version: true`、`warning_exit_codes: [true]`、`[1.0]`、`[1e0]`、`max: true`、`max: 3.0`、JSON の `[NaN]`・`[Infinity]`、YAML の `[.nan]`・`[.inf]`、`[99999999999999999999]`、`max: 4294967296`、桁が 10,000 の整数(`format_version: 1000…0`)、YAML の `1e999`(`double.Parse` が `Infinity` を返すか `OverflowException` を投げる)、`9007199254740993`(`double` に直すと `9007199254740992` に化ける整数。2^53 + 1)、ステートの `exit_code: true`・`1.0`・`NaN`・`99999999999999999999` | `validate` / `run a` | すべて「範囲外・型の誤り」として拒否する(定義: 64、ステート: 65)。真偽値を `1` として、`1.0` を `1` として、`Infinity` を整数として受理しない(字句の形を、整数(符号と 10 進の数字、または YAML の 16 進・8 進)と、それ以外に分類してから、`BigInteger` で範囲を検査し、`long`・`int` に直す)。`double` を経由しない(`9007199254740993` は、整数の字句として、`long` の範囲内で受理される値が、そのまま保たれる)。`FormatException`・`OverflowException`・`InvalidCastException` が漏れて 70 にならない。桁が多い入力で、ハングしない・異常終了しない(`BigInteger.Parse` に、無制限の桁を渡して時間がかかる場合は、字句の長さの上限(ファイルの大きさの上限 1 MiB の範囲内)を超えないことで足りる。実装時に確認)。区分: 自動と単体 |
| AC-503 | 待ちの 1 区間の上限 | `Conductor.Runner` の待ちの部品の単体(`Thread.Sleep`・`ManualResetEventSlim.Wait`・`Process.WaitForExit(int)` を、渡された時間を記録する関数に差し替える。合図で打ち切る) | 待機を 24 時間(86,400 秒)、`max_interval` の上限で呼ぶ。続けて、検査を通らない 30 日の待機を直接呼ぶ。さらに `1UL << 40` 秒と、`ulong.MaxValue` 秒 | 下の関数に渡される時間(`int` のミリ秒)が、いつも 50 ms 以下(07 章の決定: リトライの待機は、50 ms 以下の刻みの待ちと、単調な時計(`Stopwatch.GetTimestamp`)の期限の確認で行う)で、`int.MaxValue` を超えない。`ArgumentOutOfRangeException`(`Thread.Sleep`・`Wait`・`WaitForExit` に、`-1` 以外の負の値や大きすぎる値を渡したとき)が起きない。**`-1`(= 無期限)を、計算の誤りで渡さない**(`(int)` のキャストで `-1` に化ける値がない)。30 日の待機は、区間に分けて待つ(合図で打ち切って旗を検査する)。区分: 単体 |
| AC-504 | `Process.ExitCode` とシグナル番号の対応 | `Conductor.Runner` の単体 | `Process.ExitCode`(`137`、`143`、`130`、`129`、`131`、`128`、`255`、`256`、`0`、`3`、`-9`、`-1`、`int.MinValue`)と、「シグナルで終了した」かの判定(UNIX 系の子の終了の読み取り。07 章)を、シグナルで終了と終了コードに分ける関数 | 07 章の規則どおりに、`128 + n`(許可の番号 n)をシグナルで終了 n と読むか、終了コードとして扱う(**.NET の `Process.ExitCode` は、UNIX 系で、シグナルで終了した子に 128 + n を返すと記憶しているが、公式の文書では確認できていない(未確認)。このため、子が自分で `exit(137)` した場合との区別がつかない。この曖昧さの扱いは 07 章に従う。AC-140 が実機の確認を兼ねる**)。負の値・範囲外の番号でも、落ちない(`PosixSignal` への変換で例外が出ない)。`SIGHUP`=1、`SIGINT`=2、`SIGQUIT`=3、`SIGKILL`=9、`SIGTERM`=15(UNIX 系の共通の値)。Windows の `Process.ExitCode` は、シグナルとして解釈しない(終了コードとして、32 ビット符号つきに直して範囲合わせをする。07 章に従う。実機は未確認)。区分: 単体 |
| AC-505 | シグナルの送信で、ジョブの終わりを待たずに終わらない | `SL` の `sleeper` を `TJ --out O trap 500 3` にする(UNIX 系) | `run sleeper` を起動し、`ready` のあと SIGTERM | 終了 143。`O` に `trapped`、続いて `done` が記録されており、`conductor` が、ジョブの終了(`Process.WaitForExit` が終了を返すこと)を待ってから終わる(シグナルの転送が戻った時点で終わらない)。`conductor` の終了は、SIGTERM の送信から 500 ms 以上あと。ステート `sleeper=error`(終了コード 3 は、中断の分類により `128 + 15` が採られる。ジョブの終了コード 3 は表示に使わない)。区分: UNIX 系 |
| AC-506 | ジョブが残した孫のプロセスを待たない | `argv=["sh", "-c", "sleep 5 & exit 3"]`(UNIX 系) | `run a`(標準出力・標準エラーを、パイプではなく一時ファイルに向けて起動する。パイプだと、孫が持ち続ける書き込み側のために、読み手が EOF を 5 秒待つ。終了の時刻は `WaitForExit` で測る) | 終了コード 3。`conductor` が、ジョブ(`sh`)の終了から 2 秒以内に終わる(孫の終了を待たない。子の標準入出力を `Redirect*=true` にして、非同期の読み取りで EOF を待つと、孫を待ってしまうので、`conductor` は、子の標準入出力を引き継ぐ。`Redirect*=false`。引数なしの `WaitForExit()` は、リダイレクトした出力の EOF を待つ場合があるので、使うなら `WaitForExit(int)`。AC-505 と矛盾しない範囲で) |
| AC-507 | 単一ファイルと `dotnet conductor.dll` が同じ | 発行した単一ファイルと、フレームワーク依存のビルドの `conductor.dll` | 両方で、`--version`、`--help`、`validate`(正しい定義・誤りの定義)、`run`、`watch`、使い方の誤りを実行 | 終了コード・OUT・ERR が同じ。`dotnet conductor.dll` では、ジョブの子の起動に `Environment.ProcessPath` を使わない(これは `dotnet` を指す。`ProcessPath` を使う場合の挙動は、1 章の確認の範囲では、単一ファイルで実行ファイルの場所を指す)。区分: 自動(3 OS) |
| AC-508 | プロジェクトファイルの形 | `Conductor.sln`、各 `.csproj`、`Directory.Build.props`、`Directory.Packages.props`、`global.json`、`NuGet.config` | 点検(XML と JSON の走査) | `Conductor.sln` に 3 つの `src/` のプロジェクトと 3 つの `tests/` のプロジェクトと `tools/Conductor.Checks` がある。`Conductor.Cli.csproj` の `AssemblyName` が `conductor`、`OutputType` が `Exe`。`Conductor.Core`・`Conductor.Runtime` が `Library`。プロジェクト参照が `Cli` → `Runtime` → `Core` の向きだけで、逆向き・循環がない(AC-514)。`Conductor.Cli.csproj` に、`RuntimeIdentifiers`(4 つの RID)、`SelfContained` と `PublishSingleFile` の発行時の設定(`Condition="'$(RuntimeIdentifier)' != ''"`)、`InvariantGlobalization`、`DebugType` が `none`、`PublishTrimmed`・`PublishAot`・`PublishReadyToRun` がない(6.2 節)。`NuGet.config` のパッケージ元が 1 つ(nuget.org)。区分: 点検 |
| AC-509 | 標準入力はジョブに引き継ぐ | `P(p; a:0{cmd="TJ --out O stdin"})` | `run a`(標準入力のパイプに `hello⏎` を書いて閉じる) | 終了 0。`O` = `stdin="hello\n"`。`conductor` が標準入力を読まない(`Console.In`・`Console.OpenStandardInput()` を読み始めない。AC-497 と合わせて、読み始めると入力を奪うため。`ProcessStartInfo` の `RedirectStandardInput` は `false` のまま) |
| AC-510 | Windows の `.cmd` / `.bat` を、最小の .NET でも最新の .NET でも起動できる | Windows。`T/ok.cmd`(`exit /b 7`) | `MIN_TFM` と `MAX_TFM` で `run a`(`cmd` = そのパス + ` plain`) | どちらも 7(AC-115)。07 章の決定: `cmd.exe` を直接起動し(`UseShellExecute = false` のまま。引数の組み立ては 07 章)、禁止文字を含む引数は起動しない(AC-486、AC-513)。この方法が、両方の版で動くことを確かめる(実機で未確認)。.NET は、`.cmd`/`.bat` を直接起動したときの引数の扱い(`BatBadBut` の対策を含む)が、版により異なる可能性があるが、この文書では公式の文書を確認していない(未確認)。区分: 自動(Windows の CI) |
| AC-511 | 辞書のキーの順序(整数に見えるキー) | YAML: `"2": a⏎"1": b⏎b: c`(キーは引用符つきの文字列)。JSON: `{"2": "a", "1": "b", "b": "c"}` | `convert in.yaml --to json` / `convert in.json --to yaml` | OUT のキーの順が、入力の記述順(`2`、`1`、`b`)。`SortedDictionary`・`OrderBy`・`HashSet` を経由して順序を失わない。区分: 自動 |
| AC-512 | 特別な名前のキーを、普通のキーとして扱う | JSON: `{"__proto__": 1, "__class__": 2, "Equals": 3, "GetType": 4, "ToString": 5, "Item": 6, "Count": 7}`(変換)。定義に `Equals: x`、`GetType: x`、`Item: x`(未知のキー) | `convert in.json --to yaml`、`validate` | 変換: 7 つのキーがそのまま出る(往復で木が等しい)。定義: `unknown key`(64)。`KeyNotFoundException`・`ArgumentException`(`Dictionary` の重複追加)・`NullReferenceException` で落ちない。区分: 自動と単体 |
| AC-513 | Windows の `.cmd` の起動引数の組み立ての単体(どの OS でも動く) | `PlatformInfo.cs` の、起動引数を組み立てる関数(07 章の `PlatformInfo.ForWindows().PrepareSpawn`。UNIX 系の分岐は `PlatformInfo.ForUnix()`。OS の判定を、引数で差し替えられる形にする)を呼ぶ | `program` と `args` を与える: (a) `x.cmd` / `X.BAT`(拡張子の大文字小文字を区別しない)と引数 `plain`、(b) 禁止文字(`"`、`%`、`^`、`&`、`\|`、`<`、`>`、`!`、`(`、`)`、`\n`、`\r`、NUL)を 1 つ含む引数(1 文字ずつ)、(c) `program` が `C:\Program Files (x86)\t\x.cmd`(`(` と `)` は許す)、(d) 拡張子が `.exe` のもの、拡張子なしのもの、(e) 空の引数 | 07 章の決定のとおり。(a) `.cmd` の規則を使う(`cmd.exe` の起動。`UseShellExecute = false` のまま)。(b) 拒否(起動しない)。(c) 受理。(d) `.cmd` の規則を使わず、そのまま。(e) 空の引数が失われない。UNIX 系の分岐は常に、そのまま。拡張子の判定は、`Path.GetExtension` に頼らず、末尾の文字列の序数比較(`StringComparison.OrdinalIgnoreCase`)で行う(AC-545)。区分: 単体(3 OS) |
| AC-514 | 依存の向きと循環の検査 | `src/`、`tests/Conductor.Tests/Inspect/DependencyTests.cs` | 点検(各プロジェクトの `ProjectReference` の XML と、各ソースの `using` と名前空間の宣言を集め、名前空間(`Definition`・`Convert`・`Json`・`RunIds`・`Flow`・`State`・`Runner`・`Report`・`Cli`)の依存の有向グラフを作り、深さ優先の探索で循環を調べる。テストの中に、許可の表を書く) | 循環が 0 件。`Conductor.Core` が `Conductor.Runtime`・`Conductor.Cli` を参照せず、`Conductor.Runtime` が `Conductor.Cli` を参照しない(プロジェクト参照で強制される)。02 章の許可されない組み合わせ(例: `Definition` から `Flow`・`Runner`・`State`・`Cli`、`State` と `Runner` の相互、`Report` から `Cli`)がない。違反するコードを一時ファイルに置くと、テストが失敗する(テストの自己確認)。CI の `lint` ジョブで実行する。区分: 点検 |
| AC-515 | `--help` / `--version` の標準出力が閉じたパイプでも落ちない | 発行した `conductor` | `conductor --help` と `conductor --version` を、標準出力を閉じた読み手(`Process.StandardOutput.Close()`、または `\| head -c 1`)で実行 | 終了 0(書き込みの失敗は無視する。08 章)。例外・スタックトレースが現れない。70 にもならない。5 秒以内に終わる。区分: 自動(UNIX 系) |
| AC-516 | `watch` の標準出力が閉じたパイプ(`IOException`) | 十分に大きな表になるステート(ジョブ 2,000 個)。UNIX 系 | `watch` を起動し、標準出力の最初のデータを受けた時点で、読み手側を閉じる | 終了 65。ERR に `conductor: error: cannot write to standard output`。`IOException`・スタックトレースが現れない。70 にもならない。AC-493(`convert` は 67)と対にする。表が小さく、書き込みが閉じる前に終わるときは検査にならないので、表を大きくする(確実さは実装時に調整。10.1 節)。区分: 自動(UNIX 系) |
| AC-517 | シグナルの一覧と転送の単体(どの OS でも動く) | `PlatformInfo.cs` の、UNIX 系と Windows の分岐を表す部品(07 章) | 受け取るシグナルの一覧、番号の対応、転送の関数(偽の子)、終了の読み取りを呼ぶ | UNIX 系: 一覧 = `SIGHUP`・`SIGINT`・`SIGQUIT`・`SIGTERM` の 4 つ、番号 1・2・3・15。転送は許可集合(1・2・3・15)だけ、`kill` の送信を 1 回呼ぶ。Windows: 一覧 = Ctrl+C・Ctrl+Break(07 章に従う)。転送の規則は 07 章に従う。終了の読み取りは、`Process.ExitCode`(`int`)を、32 ビット符号つきとして範囲合わせにかける。区分: 単体(3 OS) |
| AC-518 | 静的検査の規則が有効であることの確認 | `src/` の外の一時ディレクトリに、違反するコードを書いたプロジェクト(`Process.Start("x")`、`Thread.Sleep` ではなく `Environment.Exit(1)`、`"a".ToUpper()`、`int.Parse("1")`、`CultureInfo.CurrentCulture`、`string.Compare("a", "b")`、`Console.WriteLine("x")`)を置く。リポジトリの `Directory.Build.props`・`BannedSymbols.txt`・`.editorconfig` を取り込む(`Import`) | `dotnet build -warnaserror` | 非 0 で終わり、それぞれの規則(BannedApiAnalyzers の `RS0030` など。**診断 ID は、この文書では公式の文書で確認していない(未確認)ので、採用した版で確かめる**)の指摘が出る。製品のコードに `#pragma warning disable`・`[SuppressMessage]` が 0 件(許可の一覧を除く)。区分: 点検 |
| AC-519 | 孤立サロゲートの表示 | UNIX 系。ジョブID・`--run-id`・`--config` のパスに、不正なバイト(`FF`)を渡す(AC-406 と同じ方法)。JSON の `"\ud800"` を含む `command`・キー名(03 章が受理する場合) | `run <ID>`、`validate --config <パス>`、`watch`(ジョブID に孤立サロゲートを含むステートがある場合) | ERR・OUT の書き込みで、**出力の符号化が、孤立サロゲートを U+FFFD に化かさず、例外も投げない**。出力が、有効な UTF-8 として読める(`new UTF8Encoding(false, true).GetString` が成功する)。孤立サロゲートは、08 章のエスケープ(例: `\u{d800}`)で出る(`UTF8Encoding` の既定の置換(U+FFFD)に任せない。生のバイトを出さない)。区分: UNIX 系と単体 |
| AC-520 | 標準出力・標準エラーの符号化 | 日本語を含む出力を出す実行(`LANG=ja_JP.UTF-8` で `watch`・`validate`・`--help`) | 環境変数 `LANG=C`、`LC_ALL=C`、`LC_ALL=ja_JP.eucJP`、`DOTNET_SYSTEM_GLOBALIZATION_INVARIANT=0`/`1`、`DOTNET_CLI_UI_LANGUAGE=ja` の各組み合わせで実行。Windows は、コンソールのコードページを `chcp 932` と `chcp 65001` にして(コンソールにつないだ場合と、パイプの場合) | どの組み合わせでも、OUT・ERR のバイト列が同じ(BOM なしの UTF-8)。`Console.OutputEncoding` を使わず(Windows のコンソールの設定を変えない)、`Console.OpenStandardOutput()`・`Console.OpenStandardError()` から、BOM なしの UTF-8(`new UTF8Encoding(false)`)の書き出しを作る(決定 10)。改行は LF 固定(`Console.WriteLine` の `Environment.NewLine` を使わない)。Windows のコンソールの表示(コードページ)は、別に手で確かめる(未確認)。区分: 自動 |
| AC-521 | 出力の順序(flush) | `P(p; a:0{cmd="TJ flaky F 1 75", retry=(exit_codes=[75], max=3, interval="1s")})`、標準出力と標準エラーを同じパイプ(`2>&1`。テストの補助がシェルで合流させる)にする | `run -v a` | 出力の行の順が、時間の順になる。`conductor` の `retry` の行が、2 回目のジョブの出力より前に現れる。`conductor` が自分の出力を、子を起動する前に `Flush()` している(`AutoFlush` を有効にするか、書き込みのたびに `Flush()` する。決定 10。flush の漏れの確認)。区分: 自動 |
| AC-522 | NUL を含む `command` | `"command": "a\u0000b"`、`argv=["a", "b\u0000c"]`(JSON の `\u0000` のエスケープ) | `validate` / `run a` | `validate` は 0(03 章は、`command` の中の NUL を定義の誤りにしない)。`run a` は、`Process.Start` が投げる例外(`InvalidOperationException`・`ArgumentException`・`Win32Exception` のいずれか。**NUL を含む `FileName`・引数に対する例外の型は、公式の文書で確認していない(未確認)**)を、07 章の規定で起動失敗にして、終了 127、ERR = `conductor: error: job 'a': cannot start command '…'`。70 にならない。ステートは `a=error ; resume=a`。区分: 自動と単体(UNIX 系の実測は 07 章。Windows は未確認) |
| AC-523 | 空白の定義(`string.Trim()` との差) | `"command": "\u001c"`、`"\u0085"`、`"\u00a0"`、`"\u200b"`、`"\u3000"`、`"name": "\u001c"`(別々の定義) | `validate` | 03 章の「空白だけ」の定義(Unicode の White_Space など)のとおり。.NET の `string.Trim()`・`char.IsWhiteSpace`・`string.IsNullOrWhiteSpace` は、Unicode の White_Space の集合と、細部(U+001C〜U+001F、U+0085、U+180E など)が異なる可能性がある(**公式の文書で、各文字の扱いを確認していない(未確認)**)。実装は、03 章の集合を明示した表で判定する(`Trim()`・`char.IsWhiteSpace` に頼らない)。`\u200b` は空白と見なさない(White_Space でない)。区分: 単体 |
| AC-524 | 再帰の深さ(スタックの大きさ) | 深さ 1,000 / 100,000 の YAML・JSON(定義・ステート・変換)。`command` の分割(AC-410) | `validate` / `run a` / `convert` | `StackOverflowException` でプロセスが落ちない(64 / 65 / 67)。深さ 32 の入力を、**スタックを 256 KiB に制限したスレッド**(`new Thread(work, maxStackSize: 256 * 1024)`。小さすぎて起動できない場合は、実装時に値を調整する)の中で処理できる(再帰の深さが、入力の深さに比例して大きくならない作りであること。製品が、スレッドのスタックの大きさを変えていない(点検))。区分: 単体と自動 |
| AC-525 | YAML ライブラリの型判定に頼らない | YAML: AC-228 の値に加え、`a: 2001-12-14t21:59:43.10-05:00`(日時)、`b: 190:20:30`(60 進数)、`c: =`(値のタグ)、`d: <<`、`e: 0o17`、`f: +1`、`g: .5`、`h: 1.`、`i: -.inf`、`j: 0x_1F`、`k: 1e3` | `convert in.yaml --to json` | 日時・60 進数・`=` などは、文字列のまま(`DateTime`・`DateTimeOffset` が出力に漏れない)。数値は 03 章 4.2 節の規則(YAML 1.2 のコアスキーマ)に従う。ライブラリの型を指定した復元(`Deserializer`)や、スカラーの暗黙の型判定の結果を、そのまま使っていないこと。区分: 自動 |
| AC-526 | 発行物の中身の検査 | 発行した `publish/<RID>/` と、作ったアーカイブ | `dotnet run --project tools/Conductor.Checks -- dist <ディレクトリ> <RID>`(5.4 節) | 発行ディレクトリの内容が、`conductor`(Windows は `conductor.exe`)1 つだけ(`.pdb`・`.dll`・`*.json`・`*.config`・`createdump` などの余分なファイルがない。自己展開の補助ファイルがない。**単一ファイルで、ランタイムのネイティブのライブラリが別のファイルとして残るかは、公式の文書の記述では、`IncludeNativeLibrariesForSelfExtract` を使わないと分離する場合があると読め、確認できていない(未確認)。余分なファイルが残る場合は、6.2 節の決めに従って、そのプロパティを有効にするか、アーカイブにそれらを含める。AC-440 の期待を、実機の結果に合わせて直す**)。実行ファイルが空でない。アーカイブに、`tests/`・`tools/` の成果物が入っていない。区分: 点検 |
| AC-527 | 発行物のメタデータ | 発行した実行ファイル | 点検(`conductor --version` と、ファイルの属性) | `--version` の OUT が `conductor <Version>`。実行ファイルの UNIX 系の実行権限が付いている(`tar` に、権限が保たれる。zip は Windows 用だけ)。`runtimeconfig` の knob(`System.Globalization.Invariant`)が、`true` で埋め込まれている(AC-530 が実行で確かめる)。区分: 点検 |
| AC-528 | 展開した配布物が空の環境で動く | 配布されたアーカイブ | .NET を入れていない環境(`DOTNET_ROOT` を未設定、`PATH` から `dotnet` を除く)で展開して、`conductor --version`、`validate`(`B0`) | 動く(自己完結)。`HOME` が未設定でも動く(自己展開を使う構成では、`HOME` か `DOTNET_BUNDLE_EXTRACT_BASE_DIR` が要る。1 章)。`validate` が 0。区分: 自動(3 OS。linux-arm64 は実行できる環境のとき) |
| AC-529 | フレームワーク依存のビルドが動く | `dotnet build` の出力の `conductor.dll` | `dotnet conductor.dll --version` | 動き、OUT = `conductor <版>`。区分: 自動(Linux の CI) |
| AC-530 | `InvariantGlobalization` の確認 | 発行した `conductor` と、ビルドの `runtimeconfig.json` | `--help` と `watch` を、`LANG`・`LC_ALL` を変えて実行(`ja_JP.UTF-8`、`tr_TR.UTF-8`(トルコ語。`I` の大文字小文字の規則が違う)、`C`)。ICU のライブラリが無い環境(コンテナ)でも | どのロケールでも、OUT・ERR のバイト列が、決められた言語の選択(08 章)のとおりで、ICU の有無で変わらない。`runtimeconfig.json` に `"System.Globalization.Invariant": true` がある(点検)。製品のコードが、文化依存の API(`string.ToUpper()`・`ToLower()`・`Compare`・`StartsWith(string)`・`EndsWith(string)`・`IndexOf(string)`・`Parse`・`ToString()` の既定、`CultureInfo.CurrentCulture`)を使わない(AC-540)。**トルコ語のロケールでの `"i".ToUpper()` の結果が `İ` になることを、この文書では確認していない(未確認)。ただし、`InvariantGlobalization` 下でも、文化の規則に依らない比較にする規定は変えない** |
| AC-531 | 危険な YAML のタグでオブジェクトが作られない | YAML: `command: !<tag:example.com,2026:exec> ["touch pwned.txt"]`、`a: !System.Diagnostics.Process {}`、`b: !!binary aGk=`、`c: !MyApp.Evil {x: 1}`(定義と変換の入力の両方) | `validate --config <YAML>` / `convert in.yaml --to json` | 64 / 67(`syntax error`。理由 `タグは使えません`)。`T` に `pwned.txt` ができない。型を指定した復元(`Deserializer` など)を呼ばない(AC-421)。区分: 自動 |
| AC-532 | シグナルの登録の前の `SIGINT` は、出力なしで、終了する | UNIX 系。`TJ inject-sigint <引数>`(`inject-sigint` は、`EntryPoint.Run` を呼ぶ前に、自分に `kill(getpid(), SIGINT)` を送って眠る。製品に試験用の経路を持たせず、外から、シグナルの登録がない経路の `SIGINT` を再現する) | `watch` / `reset` / `validate` / `convert in.yaml --to json` の各引数で起動 | OUT・ERR が空(例外・`internal error` が出ない)。終了は、シグナル 2 による終了、または終了コード 130。70 にならない。ステートファイルが壊れず、一時ファイルが残らない。**.NET ランタイムの既定の `SIGINT` の動作(終了コード、`Console.CancelKeyPress` を登録していないときの挙動)は、この文書では公式の文書を確認していない(未確認)**。Windows は、`PlatformInfo.ForWindows()` の `ExitByInterrupt()` が 130 を返すことを単体で確かめる(区分: 単体、3 OS) |
| AC-533 | Windows の Ctrl+Break の自動テスト | Windows の CI。`SL`(眠るジョブ)。テストが、`conductor` を、新しいプロセスグループで起動する(`CreateProcess` の `CREATE_NEW_PROCESS_GROUP`。.NET の `ProcessStartInfo` に、この指定の API がない(未確認)ので、補助の P/Invoke(`CreateProcessW`)でテストの側が起動する。製品のコードは指定しない。07 章) | ジョブが `ready` を記録したあと、`GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, pid)` を送る(P/Invoke) | `conductor` の終了コードが `128 + <07 章の Ctrl+Break の番号>`。ステートに中断が記録され、`sleeper` は `error`、後ろは `pending`(AC-301 の確認項目と同じ)。ジョブの子プロセスが残らない。07 章の方法の実機での確認は未確認。確認できない場合は、このテストを外し、AC-301 の手動だけにする(その場合、10 章に記録)。区分: 自動(Windows の CI) |
| AC-534 | `SIGCHLD` を無視して起動されても、ジョブの終了コードが正しい | Linux。`sh -c 'trap "" CHLD; exec <conductor> run a'`(`SIG_IGN` は `exec` を越えて引き継がれる。テストの補助だけがシェルを使う)。`P(p; a:3)`(`TJ exit 3`) | `run a` | 終了コード 3(`0` や別の値にならない)。ステートは `a=error(3)`。.NET ランタイムが、`SIGCHLD` の処理を内部で持つ(子の回収を `waitpid` で行う)ことと、`SIG_IGN` を継承したときの扱いは、**公式の文書で確認していない(未確認)**。実機で確かめ、結果を 10 章に記録する。区分: 自動(Linux。macOS では、必須としない) |
| AC-535 | 疑似端末の下で、ジョブが標準入力から読める。ジョブは制御端末を持つか | UNIX 系。`PtyHelper` の疑似端末の子側を、`conductor` の標準入力・標準出力・標準エラーにする。`TJ` のジョブ 2 つ: (a) `TJ readline`(標準入力から 1 行を読んで終了コード 0)、(b) `TJ tty`(`/dev/tty` を開けるか) | (a) `run a`(親から 1 行を書く)、(b) `run b` | (a) 5 秒以内に 0 で終わる(`SIGTTIN` で止まらない)。(b) 07 章の決定に従う(子を新しいセッションに置くなら 9(`/dev/tty` を開けない。`ENXIO`)、置かないなら 0)。区分: 自動(UNIX 系。疑似端末の手順は P/Invoke で行う。実行は未確認) |
| AC-536 | `convert` の入力が名前つきパイプ(FIFO)でもブロックしない | UNIX 系。`mkfifo T/in.fifo`(書き込む側は現れない。テストの補助が `mkfifo` コマンドを起動する) | `convert in.fifo --to json`(タイムアウト 5 秒) | 5 秒以内に終わり、終了 67。ERR の最初の行の種類の文 = `cannot read input file`(理由 `通常のファイルではありません`)。09 章の決定: ファイルを開く前に、`File.GetAttributes`・`FileInfo`(`UnixFileMode`・`Attributes`)で通常のファイルか調べる(`FileStream` の `FileMode.Open` で、FIFO を開くとブロックするため)。**`FileInfo` が FIFO をどう報告するかは、公式の文書で確認していない(未確認)**。区分: 自動(UNIX 系) |
| AC-537 | `PosixSignalRegistration` が GC されない | UNIX 系。`Conductor.Runner` のシグナルの準備の単体(`process-global` の収集) | シグナルを登録したあと、`GC.Collect()`、`GC.WaitForPendingFinalizers()`、`GC.Collect()` を 3 回行い、そのあとシグナルを送る(自分への `kill`) | コールバックが呼ばれ、旗が立つ(ファイナライザが登録を解除していない。1 章で確認した、`PosixSignalRegistration` のファイナライザが登録を解除する、という性質に対する、保持の確認)。登録のオブジェクトを、ローカル変数ではなく、フィールド(静的、またはシグナルの準備の部品のインスタンスのフィールド)に持つ。`Release` ビルドで行う(`Debug` ビルドでは、ローカル変数が、メソッドの終わりまで生きる場合があり、検査にならないため)。区分: 単体(UNIX 系) |
| AC-538 | シグナルのコールバックが別のスレッドで動いても、旗が見える | 同上 | 別のスレッドから(メインスレッドが `ManualResetEventSlim.Wait(50 ms)` で待っている間に)シグナルを送る。1,000 回繰り返す(1 回ごとに登録し直す) | 毎回、メインスレッドが旗を見て、待ちから起きる(旗の読み出しが `Volatile.Read` か `Interlocked`、起こす操作が `ManualResetEventSlim.Set`。見落としと、起こしそびれがない)。最初の番号が残る。区分: 単体(UNIX 系) |
| AC-539 | `PosixSignalContext.Cancel` の扱い | UNIX 系 | `run` の実行中に SIGTERM・SIGINT・SIGHUP・SIGQUIT を送る(AC-280〜282) | `conductor` のプロセスが、ランタイムの既定の動作(即座の終了)で終わらず、ジョブの終了を待ってから、`128 + n` で終わる(コールバックが `Cancel = true` を設定している。`Cancel` を設定しないと、ランタイムが、そのシグナルの既定の動作を続けるため。**この動作の正確な意味は、この文書では公式の文書を確認していない(未確認)**)。区分: 自動(UNIX 系) |
| AC-540 | 文化依存の API を使わない | ソース、`BannedSymbols.txt` | 点検(文字列の走査と、アナライザー) | 製品のコードが、文化依存の API(`string.ToUpper()`・`ToLower()`・`string.Compare(string, string)`・`CompareTo`・`StartsWith(string)`・`EndsWith(string)`・`IndexOf(string)`・`int.Parse(string)`・`long.Parse`・`double.Parse`・`ToString()`(数値・日時の、書式なし)・`string.Format`・`$"{数値}"` の補間・`CultureInfo.CurrentCulture`・`CurrentUICulture`)を使わない(`BannedSymbols.txt` で禁止。9.3 節)。使うのは、序数比較(`StringComparison.Ordinal`・`OrdinalIgnoreCase`)と、`CultureInfo.InvariantCulture` を明示した形(`ToString(CultureInfo.InvariantCulture)`、`int.TryParse(s, NumberStyles.None, CultureInfo.InvariantCulture, out v)`)だけ。`InvariantGlobalization` が有効でも、コード上で明示する(`InvariantGlobalization` が文化依存の API の結果を、不変の文化にするのは、`CurrentCulture` が不変になるためで、コードの意図を残す。1 章では、この挙動を確認していない。未確認)。区分: 点検 |
| AC-541 | `checked` と `unchecked` の使い分け | ソース、`Directory.Build.props` | 点検 | `CheckForOverflowUnderflow` が `false`(既定)。`checked` の式が、製品のコードにない。符号なしの飽和は、`Math.BigMul`・`UInt128`(AC-501)。`(int)` のキャストで、範囲外の値が黙って化ける箇所には、先に範囲を検査する(`unchecked((int)v)` を使うのは、32 ビットの符号なしを符号つきに直す箇所(AC-392)だけ)。区分: 点検 |
| AC-542 | `Regex` の落とし穴(`$`、`\d`) | `Conductor.RunIds` の単体と、`--run-id` | run ID の検査に、`r1<LF>`(末尾が改行)、`１２３`(全角の数字)、`٣`(アラビア・インドの数字)、`a<U+0085>` を渡す | すべて無効(68)。**.NET の `Regex` の `$` は、文字列の末尾の `\n` の前にも一致し、`\d` は Unicode の数字(Nd)に一致する**(公式の文書の記述を、この文書の作成時には確認していない。未確認)。したがって、製品は、run ID の検査に `Regex` を使わず、1 文字ずつ、ASCII の文字の範囲を比べる(使う場合は、`\z` と `RegexOptions.ECMAScript` か `[0-9]` の明示、`RegexOptions.CultureInvariant`)。区分: 単体と自動 |
| AC-543 | 数値の字句を、文化・書式に依存せず読む | `Conductor.Definition` の間隔(`1s`・`2m`・`3h`)の単体 | `interval` に、`１s`(全角)、`+1s`、`-1s`、`1_0s`、`1,0s`、`1.0s`、`0x1s`、` 1s`、`1 s`、`1s `、`1s<LF>`、`٣s` | すべて無効(`retry interval is invalid`)。`int.Parse`・`long.Parse`・`ulong.Parse` の既定の `NumberStyles`(先頭・末尾の空白、符号、桁区切りを受理する場合がある)と、全角の数字の扱いに依存しない(`char` が `'0'`〜`'9'` の範囲かを自前で比べて、桁を積む。`ulong` の範囲の検査は先に行う)。区分: 単体 |
| AC-544 | `double` の書式 | `Conductor.Convert` の単体 | JSON の浮動小数点数の出力に、`1e21`、`1e-7`、`0.1`、`-0.0`、`123456789012345678`(整数の字句は別)、`5e-324`、`1.7976931348623157e308`、`0.30000000000000004` を通す | 09 章の規則の書式(字句が、入力のまま保たれる場合は、入力の字句)で出る。`double.ToString()` の既定(文化依存。`E+21` の形)や、`"R"` の出力を、そのまま使うかは 09 章に従う。往復で値が変わらない(最短の往復の書式。`"R"` が .NET Core 3.0 以降で最短の往復の文字列を返す、と記憶している(未確認))。区分: 単体 |
| AC-545 | `Path.GetExtension` の落とし穴 | `Conductor.Definition` の拡張子の判定の単体 | `--config` に、`.json`、`.yaml`、`a.`、`a.json.`、`a.JSON`、`dir.json/file`、`dir.json\file`(UNIX 系ではファイル名の一部)、`a.json `(末尾に空白)、`a.yml.bak`、`conductor.json` を渡す | 03 章の規則どおり(最後の区切り文字のあとの、最後の `.` から末尾まで。大文字小文字を区別しない)。`Path.GetExtension(".json")` が `.json` を返すこと、末尾の `.` で空文字列を返すこと、プラットフォームで区切り文字(`\`)の扱いが異なること、に依存せず、03 章の規則を、序数比較で自前で実装する(**`Path.GetExtension` の細部は、この文書では公式の文書を確認していない(未確認)**)。区分: 単体 |
| AC-546 | `File.Move`/`File.Replace` と Windows | `Conductor.State` の単体と、Windows の CI | ステートを 1,000 回書き換える。読み手のスレッドが、同時に読み続ける(Windows) | 読み手が、直前の完全な内容か新しい完全な内容だけを見る。置き換えに失敗(`IOException`、`UnauthorizedAccessException`)した場合は、短い再試行をするか、ステートの書き込みの失敗(65)にする(06 章の決定に従う)。**Windows で、置き換え先が他のプロセスに開かれている場合の `File.Move(…, overwrite: true)`・`File.Replace` の挙動は、公式の文書で確認していない(未確認)**。区分: 単体(3 OS) |
| AC-547 | `Environment.GetCommandLineArgs` と `args` | ソース、発行した実行ファイル | 点検と `--version`、`--help`(単一ファイル、`dotnet conductor.dll` の両方) | 製品が、`Main(string[] args)` の `args` だけを使い、`Environment.GetCommandLineArgs()` を使わない(`BannedSymbols.txt` で禁止)。`args` の先頭が、プログラム名でなく、最初の引数。`dotnet conductor.dll` と単一ファイルで、引数の並びが同じ。**単一ファイルの `Environment.GetCommandLineArgs()` の先頭の値の仕様は、1 章の確認の範囲では、.NET 11 から呼び出しの名前を保つように変わる、とある**(版による差を避けるため使わない)。区分: 点検と自動 |
| AC-548 | `Process.Start` の例外の網羅 | `Conductor.Runner` の単体(`Process.Start` を差し替えられる形にして、各例外を投げる偽の起動) | `Win32Exception`(`NativeErrorCode` の各値)、`InvalidOperationException`、`ArgumentException`、`PlatformNotSupportedException`、`IOException`、`UnauthorizedAccessException`、`OutOfMemoryException`(これは捕まえない) | `OutOfMemoryException` 以外は、すべて起動失敗(分類 順 4)として、終了 127。例外のメッセージ・`NativeErrorCode` の値は、表示に出さない。`OutOfMemoryException` は、入口の `catch (Exception)` で、70(AC-414)。区分: 単体 |
| AC-549 | `Environment.Exit` と `finally` | ソース | 点検と自動 | `Environment.Exit` が、`finally` の処理(シグナルの登録の `Dispose`、ステートの一時ファイルの削除、標準出力の `Flush`)を実行しないことを前提に、`Main` が戻り値で終わる経路では、それらがすべて実行される。`UnhandledException` のハンドラの中で `Environment.Exit` を使う場合は、その前に、標準エラーへ `Flush` する(AC-490)。区分: 点検 |

---

## 5. CI(GitHub Actions)と品質ゲート

ワークフローは `.github/workflows/` の 3 つ: `ci.yml`、`audit.yml`、`release.yml`。GitHub Actions の部品(`actions/checkout`、`actions/setup-dotnet`、`actions/upload-artifact`、`actions/download-artifact`)の版は、`<版>` と書いた場所を、**実装時に、その時点の最新の安定版のメジャー版で確かめて決める**(`actions/setup-dotnet` の README の例が `@v6` であることだけ、確認した。1 章)。
.NET の版は、`MIN_TFM`(最小。`Directory.Build.props` の `ConductorMinTfm`)と、`MAX_TFM`(最新)で指定する。ワークフローの `env` に、`DOTNET_MIN_TFM`(例: `net10.0`)、`DOTNET_MAX_TFM`、`DOTNET_MIN_CHANNEL`(例: `10.0.x`)、`DOTNET_MAX_CHANNEL` を書く(例は 2.5 節のとおり、確認した値ではなく、作成日の状況に基づく)。`global.json` の SDK は `MAX_TFM` の SDK(`actions/setup-dotnet` の `global-json-file`)。最小の版のランタイムは、`dotnet-version` に `${{ env.DOTNET_MIN_CHANNEL }}` を足して入れる(`global-json-file` と `dotnet-version` を両方指定すると、両方入る。1 章で確認)。
GitHub Actions の式では、`env` を `strategy.matrix` の中では使えない(使えるコンテキストは `github`・`needs`・`vars`・`inputs` など。確認は文書の記憶による。実装時に確認)ので、最小と最新を行列に入れず、ジョブを分ける(`test` と `min-test`)。式の関数に、文字列の切り出し(`substring` など)はない(記憶による。実装時に確認)ので、`net10.0` と `10.0.x` は、別々の `env` として持ち、`Conductor.Checks consistency` が一致を検査する。

### 5.1 `ci.yml`(名前 `CI`)

| 項目 | 内容 |
|------|------|
| 起動 | `push`(`main`)、`pull_request`(対象 `main`)、`workflow_call`(リリースから呼ばれる) |
| 権限 | `contents: read` |
| 同時実行 | `group: ci-${{ github.workflow }}-${{ github.ref }}`、`cancel-in-progress: true` |
| 環境変数 | `DOTNET_MIN_TFM`・`DOTNET_MAX_TFM`・`DOTNET_MIN_CHANNEL`・`DOTNET_MAX_CHANNEL`(上記)。`DOTNET_NOLOGO: "1"`、`DOTNET_CLI_TELEMETRY_OPTOUT: "1"`、`DOTNET_SKIP_FIRST_TIME_EXPERIENCE: "1"`、`CI: "true"`(`ContinuousIntegrationBuild` の条件に使う) |
| チェックアウト | `actions/checkout@<版>` |
| SDK の用意 | `actions/setup-dotnet@<版>`。`global-json-file: global.json`。`cache: true`、`cache-dependency-path: "**/packages.lock.json"`(キャッシュは既定で無効。`packages.lock.json` がないとキャッシュの設定が失敗する。1 章。複数のロックファイルを、ワイルドカードで指定できるかは未確認。できなければ、ファイルを列挙する) |
| 復元 | `dotnet restore --locked-mode`(ロックファイルのとおり。8.4 節)。以降のコマンドは `--no-restore` |

| ジョブ | 目的 | OS | 版 | コマンド | 失敗の条件 |
|--------|------|----|----|----------|------------|
| `lint` | 整形・静的検査・依存の向き | `ubuntu-24.04`(15 分) | 最新 | 復元 → ① `dotnet format --verify-no-changes --no-restore` ② `dotnet build -c Release --no-restore -warnaserror`(アナライザー、BannedApiAnalyzers 込み)③ `dotnet test -c Release --no-build --filter "FullyQualifiedName~Inspect"`(依存の向きと循環、禁止の構文の点検。AC-514、AC-518、AC-420〜423) | 整形の差分が 1 つでも、警告(アナライザーの指摘を含む)が 1 つでも、循環・許可されない参照・禁止の構文がある(AC-480、AC-514、AC-518) |
| `consistency` | 版と依存の記載の整合 | `ubuntu-24.04`(5 分) | 最新 | ① `dotnet run --project tools/Conductor.Checks -- version`(`Directory.Build.props` の版が唯一の出所)② `… -- consistency`(5.4 節)③ `… -- deps`(8 章)④ `dotnet restore --locked-mode`(ロックファイルと定義の整合)⑤ `dotnet list package --include-transitive`(一覧の記録) | 版の不一致、依存の許可の違反、ロックファイルの食い違い(AC-476〜478) |
| `build-check` | 発行(単一ファイル・自己完結)ができ、中身が正しく、動くこと | `ubuntu-24.04`、`macos-latest`、`windows-2022`(`fail-fast: false`。20 分)。行列の `rid` は `linux-x64`・`osx-arm64`・`win-x64`(それぞれの OS で実行できる RID) | 最新 | 復元 → `dotnet publish src/Conductor.Cli -c Release -r <RID> --no-restore -o publish` → `dotnet run --project tools/Conductor.Checks -- dist publish <RID>` → `publish/conductor`(Windows は `conductor.exe`)の `--version` の OUT が `conductor <Version>` | 発行の失敗、`--version` の失敗、発行物の中身の違反(AC-422、AC-479、AC-526〜528) |
| `test` | 3 つの OS で、全テスト(単体・受け入れ・点検)が通ること | `ubuntu-24.04`、`macos-latest`、`windows-2022`(`fail-fast: false`。20 分) | 最新 | 復元 → `dotnet build -c Release --no-restore` → `dotnet test -c Release --no-build --framework $DOTNET_MAX_TFM` | テストの失敗(30 秒の上限を含む) |
| `min-test` | 最小の .NET でも、テストが通ること | `ubuntu-24.04`、`windows-2022`(`fail-fast: false`。20 分) | 最小 | `dotnet-version` に `DOTNET_MIN_CHANNEL` を足して入れる → 復元 → `dotnet test -c Release --framework $DOTNET_MIN_TFM`(最小と最新が同じなら、`test` と重複するが、そのまま行う) | テストの失敗。Windows で `.cmd` を最小の版で起動できること(AC-510)を含む |
| `coverage` | 行カバレッジ 80% の維持 | `ubuntu-24.04`(25 分) | 最新 | 5.3 節のコマンド | カバレッジが 80% 未満 |
| `docs-check` | README に必要な語が残っていること | `ubuntu-24.04`(5 分) | 最新 | `dotnet run --project tools/Conductor.Checks -- readme`(5.4 節。8 つの文字列) | 1 つでも欠けると `README.md is missing: <語>` を出して失敗 |
| `startup-bench` | 起動と性能の測定(2.1 節) | `ubuntu-24.04`、`macos-latest`、`windows-2022`(`fail-fast: false`。25 分) | 最新 | 復元 → 発行(`build-check` と同じ。単一ファイル)→ `CONDUCTOR_PERF=1 CONDUCTOR_BIN=<発行した実行ファイル> dotnet test -c Release --filter "FullyQualifiedName~Perf"`(環境変数 `STARTUP_BENCH_ENFORCE: "false"`)。測定値は `GITHUB_STEP_SUMMARY` に出る | `continue-on-error: true` のため、ジョブが失敗しても全体は失敗にならない。基準超えは警告のみ(強制するには環境変数を `"true"` にする) |

任意のジョブ(置くかどうかは、実装時に決める。ブランチ保護の必須にはしない): `next-dotnet-test`(次の版の SDK の先行版で `dotnet test`。`continue-on-error: true`。`actions/setup-dotnet` の `dotnet-quality: preview` の指定は実装時に確認)、`publish-ready-to-run`(`-p:PublishReadyToRun=true` で発行した単一ファイルの起動時間を、通常の発行と比べる。6.2 節の採否の判断のためだけの計測)。

`ci.yml` の骨組み(`<版>` は実装時に確認):

```yaml
name: CI
on:
  push: { branches: [main] }
  pull_request: { branches: [main] }
  workflow_call:
permissions:
  contents: read
concurrency:
  group: ci-${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
env:
  DOTNET_MIN_TFM: "net10.0"
  DOTNET_MAX_TFM: "net10.0"
  DOTNET_MIN_CHANNEL: "10.0.x"
  DOTNET_MAX_CHANNEL: "10.0.x"
  DOTNET_NOLOGO: "1"
  DOTNET_CLI_TELEMETRY_OPTOUT: "1"
  DOTNET_SKIP_FIRST_TIME_EXPERIENCE: "1"
  CI: "true"
jobs:
  lint:
    runs-on: ubuntu-24.04
    timeout-minutes: 15
    steps:
      - uses: actions/checkout@<版>
      - uses: actions/setup-dotnet@<版>
        with: { global-json-file: global.json, cache: true, cache-dependency-path: "**/packages.lock.json" }
      - run: dotnet restore --locked-mode
      - run: dotnet format --verify-no-changes --no-restore
      - run: dotnet build -c Release --no-restore -warnaserror
      - run: dotnet test -c Release --no-build --filter "FullyQualifiedName~Inspect"
  consistency:
    runs-on: ubuntu-24.04
    timeout-minutes: 5
    steps:
      - uses: actions/checkout@<版>
      - uses: actions/setup-dotnet@<版>
        with: { global-json-file: global.json, cache: true, cache-dependency-path: "**/packages.lock.json" }
      - run: dotnet run --project tools/Conductor.Checks -- version
      - run: dotnet run --project tools/Conductor.Checks -- consistency
      - run: dotnet run --project tools/Conductor.Checks -- deps
      - run: dotnet restore --locked-mode
  build-check:
    name: build-check (${{ matrix.rid }})
    runs-on: ${{ matrix.os }}
    timeout-minutes: 20
    strategy:
      fail-fast: false
      matrix:
        include:
          - { os: ubuntu-24.04, rid: linux-x64, exe: conductor }
          - { os: macos-latest, rid: osx-arm64, exe: conductor }
          - { os: windows-2022, rid: win-x64, exe: conductor.exe }
    steps:
      - uses: actions/checkout@<版>
      - uses: actions/setup-dotnet@<版>
        with: { global-json-file: global.json, cache: true, cache-dependency-path: "**/packages.lock.json" }
      - run: dotnet restore --locked-mode
      - run: dotnet publish src/Conductor.Cli -c Release -r ${{ matrix.rid }} --no-restore -o publish
      - run: dotnet run --project tools/Conductor.Checks -- dist publish ${{ matrix.rid }}
      - name: version of the published binary
        shell: bash
        run: |
          expected="conductor $(dotnet run --project tools/Conductor.Checks -- version | tail -n 1)"
          out=$(./publish/${{ matrix.exe }} --version)
          [ "$out" = "$expected" ] || { echo "unexpected: $out"; exit 1; }
  test:
    name: test (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    timeout-minutes: 20
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-24.04, macos-latest, windows-2022]
    steps:
      - uses: actions/checkout@<版>
      - uses: actions/setup-dotnet@<版>
        with: { global-json-file: global.json, cache: true, cache-dependency-path: "**/packages.lock.json" }
      - run: dotnet restore --locked-mode
      - run: dotnet build -c Release --no-restore
      - run: dotnet test -c Release --no-build --framework ${{ env.DOTNET_MAX_TFM }}
  min-test:
    name: min-test (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    timeout-minutes: 20
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-24.04, windows-2022]
    steps:
      - uses: actions/checkout@<版>
      - uses: actions/setup-dotnet@<版>
        with:
          global-json-file: global.json
          dotnet-version: ${{ env.DOTNET_MIN_CHANNEL }}
          cache: true
          cache-dependency-path: "**/packages.lock.json"
      - run: dotnet restore --locked-mode
      - run: dotnet build -c Release --no-restore
      - run: dotnet test -c Release --no-build --framework ${{ env.DOTNET_MIN_TFM }}
  coverage:
    runs-on: ubuntu-24.04
    timeout-minutes: 25
    steps:
      - uses: actions/checkout@<版>
      - uses: actions/setup-dotnet@<版>
        with: { global-json-file: global.json, cache: true, cache-dependency-path: "**/packages.lock.json" }
      - run: dotnet restore --locked-mode
      - run: dotnet build -c Release --no-restore
      - run: >
          dotnet test -c Release --no-build --framework ${{ env.DOTNET_MAX_TFM }}
          --collect:"XPlat Code Coverage" --settings coverlet.runsettings --results-directory ./TestResults
      - run: dotnet run --project tools/Conductor.Checks -- coverage TestResults
  docs-check:
    runs-on: ubuntu-24.04
    timeout-minutes: 5
    steps:
      - uses: actions/checkout@<版>
      - uses: actions/setup-dotnet@<版>
        with: { global-json-file: global.json }
      - run: dotnet run --project tools/Conductor.Checks -- readme
  startup-bench:
    name: startup-bench (${{ matrix.rid }})
    runs-on: ${{ matrix.os }}
    timeout-minutes: 25
    continue-on-error: true
    strategy:
      fail-fast: false
      matrix:
        include:
          - { os: ubuntu-24.04, rid: linux-x64, exe: conductor }
          - { os: macos-latest, rid: osx-arm64, exe: conductor }
          - { os: windows-2022, rid: win-x64, exe: conductor.exe }
    env:
      STARTUP_BENCH_ENFORCE: "false"
      CONDUCTOR_PERF: "1"
    steps:
      - uses: actions/checkout@<版>
      - uses: actions/setup-dotnet@<版>
        with: { global-json-file: global.json, cache: true, cache-dependency-path: "**/packages.lock.json" }
      - run: dotnet restore --locked-mode
      - run: dotnet publish src/Conductor.Cli -c Release -r ${{ matrix.rid }} --no-restore -o publish
      - name: perf tests against the published binary
        shell: bash
        run: |
          export CONDUCTOR_BIN="$PWD/publish/${{ matrix.exe }}"
          dotnet test -c Release --filter "FullyQualifiedName~Perf" --framework "$DOTNET_MAX_TFM"
```

(この骨組みは、動作を確かめていない。`runs-on: macos-latest` が Apple Silicon であることは、`osx-arm64` の実行に必要で、実装時に確認する。`--framework` を `dotnet test` に渡す書式、`--collect` と `--settings` の併用、`--results-directory` は、`dotnet test --help` で確かめる。`dotnet run --project … -- version | tail -n 1` の、`dotnet run` の標準出力へのビルドの出力の混入は、`--no-build` か、検査プログラムを `dotnet publish` した実行ファイルで呼ぶ形に直す必要があるかもしれない。未確認。)

### 5.2 `audit.yml`(名前 `Audit`)

| 項目 | 内容 |
|------|------|
| 目的 | 依存の既知の脆弱性・非推奨の検査。結果は警告として出し、マージはブロックしない |
| 起動 | `pull_request`(対象 `main`)、`schedule`(cron `0 3 * * 1` = 毎週月曜 03:00 UTC)、`workflow_dispatch` |
| 権限 | `contents: read` |
| ジョブ `audit` | `ubuntu-24.04`、10 分、**`continue-on-error: true`**。`actions/checkout@<版>` → `actions/setup-dotnet@<版>`(`global-json-file`)→ ① `dotnet restore -p:AuditPipeline=true`(監査の警告 `NU1900`〜`NU1905` を、この流れだけエラーにする。`Directory.Build.props` の条件つきの設定。9.2 節。1 章で確認した、監査の流れだけを失敗にする書き方)② `dotnet list package --vulnerable --include-transitive` ③ `dotnet list package --deprecated`(②③は、一覧を記録する。`--include-transitive` が既定でないので付ける。1 章) |
| 失敗の条件 | ①が、`NuGetAuditLevel`(既定 `low`。実装時に `moderate` 以上にするか決める)以上の脆弱性で失敗する。②③の終了コードが、脆弱性・非推奨を見つけたときに 0 以外になるかは、**公式の文書で確認していない(未確認)**。確認できなければ、出力の文字列(`has the following vulnerable packages` など)を検索して、見つかれば失敗にする。いずれも、`continue-on-error` のため、ワークフロー全体は失敗にならない |

`dotnet restore` と `dotnet list package` は、脆弱性のデータベース(nuget.org の脆弱性の情報。GitHub Advisory Database が出典。1 章)へ問い合わせる(ネットワークを使う)。これは CI の検査であり、`conductor` の実行時の挙動ではない(AC-419 の対象は製品のコード)。通常のビルド(`lint`・`test` など)では、監査の警告をエラーにしない(`WarningsNotAsErrors` に `NU1901;NU1902;NU1903;NU1904` を入れる。9.2 節。新しい脆弱性の公表で、無関係な変更のビルドが突然失敗するのを避けるため。1 章で確認した、公式の勧め)。

### 5.3 カバレッジ

| 項目 | 内容 |
|------|------|
| 計測 | `dotnet test -c Release --no-build --collect:"XPlat Code Coverage" --settings coverlet.runsettings --results-directory ./TestResults`(`coverlet.collector`。3.9 節)。出力は、`TestResults/<GUID>/coverage.cobertura.xml`(Cobertura。1 章)。`Conductor.Core`・`Conductor.Runtime`・`Conductor.Cli` の行の網羅率を測る。実行ファイルを `Process` で起動する受け入れテストの中のコードは、数えない |
| 除外 | テストのアセンブリ、`tests/`、`Program.cs`(入口)。`coverlet.runsettings` の `Exclude` と `ExcludeByFile` |
| 基準 | 行の網羅率が **80.0% 未満なら失敗**。`coverlet.collector` は、しきい値の検査を持たない(1 章)ので、`Conductor.Checks coverage <ディレクトリ>` が、`TestResults` の下の Cobertura のファイル(複数あれば、行の合計を足して計算するか、最新の 1 つ。実装は、`lines-covered` と `lines-valid` の合計を使う)の網羅率を検査する(Cobertura のルート要素の `line-rate`・`lines-covered`・`lines-valid` の属性名は、この文書では確認していない(未確認)。実装時に実際のファイルで確かめる) |
| 実行する OS | `ubuntu-24.04` のみ。したがって、Windows 専用のコード(`PlatformInfo.cs` の Windows の分岐)は、計測の対象に入っても、通らない行になる。この分は 80% の内で吸収できる大きさに留める(OS 別の分岐を、薄く 1 か所に集める。2.4 節) |
| 範囲 | 単体テストと、`Deps` を差し替えた流れのテストの全体。分岐の網羅は、下限に含めない(行だけ) |

```sh
dotnet test -c Release --no-build --collect:"XPlat Code Coverage" --settings coverlet.runsettings --results-directory ./TestResults
dotnet run --project tools/Conductor.Checks -- coverage TestResults
```

### 5.4 検査プログラム(`tools/Conductor.Checks`)の全文

BCL だけの小さな実行ファイル(`OutputType=Exe`。`TargetFramework` は `$(ConductorMaxTfm)`。`Nullable=enable`、`TreatWarningsAsErrors=true`。`BannedSymbols.txt` は、製品用で、このプロジェクトには適用しない(`Regex` と `System.Text.Json` を使うため)。`dotnet format` と `-warnaserror` の対象には入る)。各ファイルの先頭に、Apache License 2.0 のヘッダ(AC-423)を置く(ここでは省略)。サブコマンド: `version [<タグの版>]`、`consistency`、`deps`、`coverage <ディレクトリ>`、`dist <ディレクトリ> <RID>`、`readme`。作業ディレクトリはリポジトリの根。失敗は、標準エラーに 1 行以上を出して終了コード 1(使い方の誤りは 2)。
**この文書の作成時には、このプログラムをビルドも実行もしていない**(.NET SDK が無い環境のため)。書いた内容は、公式の文書で確かめた API の範囲の記憶に基づく。実装時に、ビルドを通し、正常な場合と、版・依存・`MAX_TFM` が食い違う場合の動作を、見本のリポジトリで確かめる。

```csharp
// Copyright 2026 agwlvssainokuni (Apache License 2.0 のヘッダ。全文は省略)
using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Text.RegularExpressions;
using System.Xml.Linq;

namespace Conductor.Checks;

internal static class Program
{
    private static readonly string[] RuntimeAllowed = { "YamlDotNet" };
    private static readonly string[] DevAllowed =
    {
        "Microsoft.NET.Test.Sdk", "xunit", "xunit.runner.visualstudio", "coverlet.collector",
    };
    private static readonly string[] GlobalAllowed = { "Microsoft.CodeAnalysis.BannedApiAnalyzers" };
    private static readonly string[] ReadmeRequired =
    {
        "conductor.json", "--to json", "--to yaml", "exit_codes", "max_interval", "SHA256SUMS", ".NET SDK", "LICENSE",
    };

    private static int Main(string[] args)
    {
        string cmd = args.Length > 0 ? args[0] : string.Empty;
        string[] rest = args.Skip(1).ToArray();
        switch (cmd)
        {
            case "version": return CheckVersion(rest);
            case "consistency": return CheckConsistency();
            case "deps": return CheckDeps();
            case "coverage": return CheckCoverage(rest);
            case "dist": return CheckDist(rest);
            case "readme": return CheckReadme();
            default:
                Console.Error.WriteLine("usage: Conductor.Checks <version [tag] | consistency | deps | coverage <dir> | dist <dir> <rid> | readme>");
                return 2;
        }
    }

    private static int Fail(string message)
    {
        Console.Error.WriteLine(message);
        return 1;
    }

    private static IEnumerable<string> ProjectFiles()
    {
        return Directory.EnumerateFiles(".", "*.csproj", SearchOption.AllDirectories)
            .Where(p => !p.Contains($"{Path.DirectorySeparatorChar}obj{Path.DirectorySeparatorChar}", StringComparison.Ordinal)
                && !p.Contains($"{Path.DirectorySeparatorChar}bin{Path.DirectorySeparatorChar}", StringComparison.Ordinal))
            .OrderBy(p => p, StringComparer.Ordinal);
    }

    private static string? PropsValue(string name)
    {
        XDocument doc = XDocument.Load("Directory.Build.props");
        return doc.Descendants().Where(e => e.Name.LocalName == name).Select(e => e.Value.Trim()).FirstOrDefault();
    }

    // version [tag]: Directory.Build.props の <Version> が唯一の出所。標準出力の最後の行に版を出す。
    private static int CheckVersion(string[] rest)
    {
        string? version = PropsValue("Version");
        if (string.IsNullOrEmpty(version))
        {
            return Fail("Directory.Build.props: <Version> not found");
        }
        bool failed = false;
        foreach (string project in ProjectFiles())
        {
            XDocument doc = XDocument.Load(project);
            foreach (string name in new[] { "Version", "AssemblyVersion", "FileVersion", "InformationalVersion", "VersionPrefix" })
            {
                if (doc.Descendants().Any(e => e.Name.LocalName == name))
                {
                    Console.Error.WriteLine($"{project}: <{name}> must not be set here (single source: Directory.Build.props)");
                    failed = true;
                }
            }
        }
        if (rest.Length > 0 && !string.Equals(rest[0], version, StringComparison.Ordinal))
        {
            Console.Error.WriteLine($"Directory.Build.props: {version} != {rest[0]}");
            failed = true;
        }
        if (failed)
        {
            return 1;
        }
        Console.WriteLine(version);
        return 0;
    }

    private static bool ParseTfm(string? tfm, out int major, out int minor)
    {
        major = 0;
        minor = 0;
        if (tfm is null)
        {
            return false;
        }
        Match m = Regex.Match(tfm, @"^net([0-9]+)\.([0-9]+)$", RegexOptions.CultureInvariant);
        if (!m.Success)
        {
            return false;
        }
        major = int.Parse(m.Groups[1].Value, CultureInfo.InvariantCulture);
        minor = int.Parse(m.Groups[2].Value, CultureInfo.InvariantCulture);
        return true;
    }

    private static string? CiEnv(string ci, string key)
    {
        Match m = Regex.Match(ci, $@"^ {{2}}{Regex.Escape(key)}:\s*""?([^""\r\n]+?)""?\s*$", RegexOptions.Multiline | RegexOptions.CultureInvariant);
        return m.Success ? m.Groups[1].Value : null;
    }

    // consistency: global.json、Directory.Build.props、README、ci.yml の .NET の版の記載の一致(AC-477)
    private static int CheckConsistency()
    {
        bool failed = false;
        string? minTfm = PropsValue("ConductorMinTfm");
        string? maxTfm = PropsValue("ConductorMaxTfm");
        if (!ParseTfm(minTfm, out int minMajor, out int minMinor) || !ParseTfm(maxTfm, out int maxMajor, out int maxMinor))
        {
            return Fail("Directory.Build.props: ConductorMinTfm / ConductorMaxTfm must look like net10.0");
        }
        if (minMajor > maxMajor || (minMajor == maxMajor && minMinor > maxMinor))
        {
            Console.Error.WriteLine($"ConductorMinTfm ({minTfm}) is newer than ConductorMaxTfm ({maxTfm})");
            failed = true;
        }

        string sdkVersion = string.Empty;
        using (JsonDocument g = JsonDocument.Parse(File.ReadAllText("global.json"), new JsonDocumentOptions { CommentHandling = JsonCommentHandling.Skip }))
        {
            JsonElement sdk = g.RootElement.GetProperty("sdk");
            sdkVersion = sdk.GetProperty("version").GetString() ?? string.Empty;
            string roll = sdk.TryGetProperty("rollForward", out JsonElement r) ? r.GetString() ?? string.Empty : string.Empty;
            Match v = Regex.Match(sdkVersion, @"^([0-9]+)\.([0-9]+)\.([0-9]+)$", RegexOptions.CultureInvariant);
            if (!v.Success || int.Parse(v.Groups[1].Value, CultureInfo.InvariantCulture) != maxMajor)
            {
                Console.Error.WriteLine($"global.json: sdk.version ({sdkVersion}) major != {maxTfm}");
                failed = true;
            }
            if (!string.Equals(roll, "disable", StringComparison.Ordinal))
            {
                Console.Error.WriteLine("global.json: rollForward must be \"disable\" (lock files are used)");
                failed = true;
            }
        }

        string ci = File.ReadAllText(".github/workflows/ci.yml");
        string[][] pairs =
        {
            new[] { "DOTNET_MIN_TFM", minTfm! },
            new[] { "DOTNET_MAX_TFM", maxTfm! },
            new[] { "DOTNET_MIN_CHANNEL", $"{minMajor}.{minMinor}.x" },
            new[] { "DOTNET_MAX_CHANNEL", $"{maxMajor}.{maxMinor}.x" },
        };
        foreach (string[] p in pairs)
        {
            string? actual = CiEnv(ci, p[0]);
            if (!string.Equals(actual, p[1], StringComparison.Ordinal))
            {
                Console.Error.WriteLine($"ci.yml {p[0]} ({actual ?? "missing"}) != {p[1]}");
                failed = true;
            }
        }

        string readme = File.ReadAllText("README.md");
        if (!readme.Contains($".NET SDK {sdkVersion}", StringComparison.Ordinal))
        {
            Console.Error.WriteLine($"README.md: '.NET SDK {sdkVersion}' not found");
            failed = true;
        }

        foreach (string project in ProjectFiles())
        {
            XDocument doc = XDocument.Load(project);
            foreach (XElement e in doc.Descendants().Where(e => e.Name.LocalName is "TargetFramework" or "TargetFrameworks"))
            {
                string value = e.Value.Trim();
                if (!value.Contains("$(ConductorMinTfm)", StringComparison.Ordinal) && !value.Contains("$(ConductorMaxTfm)", StringComparison.Ordinal))
                {
                    Console.Error.WriteLine($"{project}: <{e.Name.LocalName}>{value}</...> must use $(ConductorMinTfm) / $(ConductorMaxTfm)");
                    failed = true;
                }
            }
        }
        return failed ? 1 : 0;
    }

    private static bool ExactVersion(string v)
    {
        return v.Length > 0 && v.IndexOfAny(new[] { '*', '[', ']', '(', ')', ',' }) < 0;
    }

    // deps: 依存の許可の一覧、中央パッケージ管理、ロックファイル、パッケージ元(AC-478)
    private static int CheckDeps()
    {
        bool failed = false;
        XDocument props = XDocument.Load("Directory.Packages.props");
        string? central = props.Descendants().Where(e => e.Name.LocalName == "ManagePackageVersionsCentrally").Select(e => e.Value.Trim()).FirstOrDefault();
        if (!string.Equals(central, "true", StringComparison.Ordinal))
        {
            Console.Error.WriteLine("Directory.Packages.props: ManagePackageVersionsCentrally must be true");
            failed = true;
        }
        string? lockProp = PropsValue("RestorePackagesWithLockFile");
        if (!string.Equals(lockProp, "true", StringComparison.Ordinal))
        {
            Console.Error.WriteLine("Directory.Build.props: RestorePackagesWithLockFile must be true");
            failed = true;
        }

        HashSet<string> allowed = new(RuntimeAllowed.Concat(DevAllowed).Concat(GlobalAllowed), StringComparer.Ordinal);
        HashSet<string> declared = new(StringComparer.Ordinal);
        foreach (XElement e in props.Descendants().Where(e => e.Name.LocalName is "PackageVersion" or "GlobalPackageReference"))
        {
            string name = e.Attribute("Include")?.Value ?? string.Empty;
            string version = e.Attribute("Version")?.Value ?? string.Empty;
            declared.Add(name);
            if (!allowed.Contains(name))
            {
                Console.Error.WriteLine($"Directory.Packages.props: {name} is not in the allowed list");
                failed = true;
            }
            if (!ExactVersion(version))
            {
                Console.Error.WriteLine($"Directory.Packages.props: {name} must have an exact version (got '{version}')");
                failed = true;
            }
        }

        foreach (string project in ProjectFiles())
        {
            XDocument doc = XDocument.Load(project);
            bool isSrc = project.Replace('\\', '/').Contains("/src/", StringComparison.Ordinal);
            foreach (XElement e in doc.Descendants().Where(e => e.Name.LocalName == "PackageReference"))
            {
                string name = e.Attribute("Include")?.Value ?? string.Empty;
                if (e.Attribute("Version") is not null || e.Attribute("VersionOverride") is not null)
                {
                    Console.Error.WriteLine($"{project}: {name} must not carry Version (use Directory.Packages.props)");
                    failed = true;
                }
                if (!declared.Contains(name))
                {
                    Console.Error.WriteLine($"{project}: {name} is not declared in Directory.Packages.props");
                    failed = true;
                }
                if (isSrc && !RuntimeAllowed.Contains(name, StringComparer.Ordinal))
                {
                    Console.Error.WriteLine($"{project}: runtime dependency {name} is not allowed in src/");
                    failed = true;
                }
            }
            string lockFile = Path.Combine(Path.GetDirectoryName(project) ?? ".", "packages.lock.json");
            if (!File.Exists(lockFile))
            {
                Console.Error.WriteLine($"{lockFile}: missing (run dotnet restore and commit it)");
                failed = true;
            }
        }

        if (!File.Exists("NuGet.config"))
        {
            Console.Error.WriteLine("NuGet.config: missing");
            failed = true;
        }
        else
        {
            int sources = XDocument.Load("NuGet.config").Descendants().Count(e => e.Name.LocalName == "add" && e.Parent?.Name.LocalName == "packageSources");
            if (sources != 1)
            {
                Console.Error.WriteLine($"NuGet.config: exactly one package source is required (found {sources})");
                failed = true;
            }
        }
        return failed ? 1 : 0;
    }

    // coverage <dir>: Cobertura の行の網羅率が 80% 以上(5.3 節)
    private static int CheckCoverage(string[] rest)
    {
        if (rest.Length < 1)
        {
            return Fail("usage: coverage <dir>");
        }
        long covered = 0;
        long valid = 0;
        string[] files = Directory.GetFiles(rest[0], "coverage.cobertura.xml", SearchOption.AllDirectories);
        if (files.Length == 0)
        {
            return Fail($"{rest[0]}: coverage.cobertura.xml not found");
        }
        foreach (string file in files)
        {
            XElement root = XDocument.Load(file).Root ?? throw new InvalidDataException(file);
            covered += long.Parse(root.Attribute("lines-covered")?.Value ?? "0", CultureInfo.InvariantCulture);
            valid += long.Parse(root.Attribute("lines-valid")?.Value ?? "0", CultureInfo.InvariantCulture);
        }
        if (valid == 0)
        {
            return Fail("coverage: lines-valid is 0");
        }
        double percent = 100.0 * covered / valid;
        Console.WriteLine(string.Create(CultureInfo.InvariantCulture, $"line coverage: {percent:F2}% ({covered}/{valid})"));
        if (percent < 80.0)
        {
            return Fail("line coverage is below 80%");
        }
        return 0;
    }

    // dist <dir> <rid>: 発行物の中身(AC-526)
    private static int CheckDist(string[] rest)
    {
        if (rest.Length < 2)
        {
            return Fail("usage: dist <dir> <rid>");
        }
        string expected = rest[1].StartsWith("win", StringComparison.Ordinal) ? "conductor.exe" : "conductor";
        string[] entries = Directory.GetFileSystemEntries(rest[0]).Select(p => Path.GetFileName(p)).OrderBy(n => n, StringComparer.Ordinal).ToArray();
        bool failed = false;
        foreach (string e in entries)
        {
            Console.WriteLine(e);
        }
        if (entries.Length != 1 || !string.Equals(entries[0], expected, StringComparison.Ordinal))
        {
            Console.Error.WriteLine($"{rest[0]}: expected exactly one file '{expected}', found: {string.Join(", ", entries)}");
            failed = true;
        }
        else if (new FileInfo(Path.Combine(rest[0], expected)).Length == 0)
        {
            Console.Error.WriteLine($"{expected}: empty file");
            failed = true;
        }
        return failed ? 1 : 0;
    }

    // readme: README に必要な語が残っている(docs-check)
    private static int CheckReadme()
    {
        string readme = File.ReadAllText("README.md");
        bool failed = false;
        foreach (string word in ReadmeRequired)
        {
            if (!readme.Contains(word, StringComparison.Ordinal))
            {
                Console.Error.WriteLine($"README.md is missing: {word}");
                failed = true;
            }
        }
        return failed ? 1 : 0;
    }
}
```

### 5.5 品質ゲートの再現手順(ローカル)

マージの前に、次をすべて通す。

```sh
# SDK の確認(global.json の版と一致すること。rollForward は disable)
dotnet --version

# 復元(ロックファイルのとおり)
dotnet restore --locked-mode

# 整形・静的検査(アナライザー、BannedSymbols.txt を含む)
dotnet format --verify-no-changes --no-restore
dotnet build -c Release --no-restore -warnaserror

# 整合
dotnet run --project tools/Conductor.Checks -- version
dotnet run --project tools/Conductor.Checks -- consistency
dotnet run --project tools/Conductor.Checks -- deps
dotnet run --project tools/Conductor.Checks -- readme

# テスト(単体 + 受け入れ + 点検)。最小・最新の版を分けて(同じなら 1 回)
dotnet test -c Release --no-build --framework net10.0

# カバレッジ(Linux)
dotnet test -c Release --no-build --collect:"XPlat Code Coverage" --settings coverlet.runsettings --results-directory ./TestResults
dotnet run --project tools/Conductor.Checks -- coverage TestResults

# 発行(単一ファイル・自己完結)と、発行物の形(RID は手元の OS に合わせる)
dotnet publish src/Conductor.Cli -c Release -r linux-x64 --no-restore -o publish
dotnet run --project tools/Conductor.Checks -- dist publish linux-x64
./publish/conductor --version

# 既知の脆弱性・非推奨(警告のみ。マージはブロックしない)
dotnet restore -p:AuditPipeline=true
dotnet list package --vulnerable --include-transitive
dotnet list package --deprecated

# 起動時間(目安)
CONDUCTOR_PERF=1 CONDUCTOR_BIN="$PWD/publish/conductor" dotnet test -c Release --no-build --filter "FullyQualifiedName~Perf"
```

加えて、CI で確かめる: Windows(`windows-2022`)と macOS(`macos-latest`)での `dotnet test`、最小の .NET でのテスト(Linux・Windows)、README の必須の語、.NET の版の記載の一致、発行物の形、起動の性能の測定(警告のみ)。
リリース前に、Windows の実機で Ctrl+C のチェックリスト(AC-301)を確かめ、結果を記録する。

ブランチ保護(どのジョブを必須にするか)の設定は、リポジトリの外にあり、未確認。想定は、`lint`・`consistency`・`build-check`・`test`・`min-test`・`coverage`・`docs-check` を必須にする。

---

## 6. リリース

### 6.1 配布の形

`conductor` は、RID ごとの **単一ファイル・自己完結** の実行ファイルとして配る。利用者は .NET を入れなくてよい。NuGet への公開は行わない。実行時の依存は YAML ライブラリ 1 つだけ(8 章)で、実行ファイルに同梱される。

| 項目 | 内容 |
|------|------|
| 対象の RID | `linux-x64`、`linux-arm64`、`osx-arm64`、`win-x64` の 4 つ |
| 成果物 | `conductor-<版>-<RID>.tar.gz`(`linux-x64`・`linux-arm64`・`osx-arm64`)、`conductor-<版>-win-x64.zip`、`SHA256SUMS`(`<版>` は `v` なし。例: `0.8.0`) |
| 中身 | アーカイブを展開すると、`conductor-<版>-<RID>/` の 1 つのディレクトリで、中が実行ファイル(`conductor`、Windows は `conductor.exe`)と `LICENSE` の 2 つだけ(AC-440、AC-526) |
| 対応 OS | Linux(x64・arm64。glibc か musl かは、RID `linux-x64`(glibc 前提)の発行物の動作する範囲で、**musl の環境(Alpine)で動くかは未確認**。musl 向けには別の RID(`linux-musl-x64`)がある、と記憶している(未確認)。この仕様の対象に含めない)、macOS(arm64。Intel の macOS は対象外)、Windows(x64) |
| チェックサム | 成果物の SHA-256 を、`sha256sum` の形式で `SHA256SUMS` に書き、一緒に配る。署名はしない(転送の破損の検知まで。配布元の改ざんは防げない) |
| リリースの注記 | 自動生成に、固定の 1 文を加える(`この成果物は署名されていません(macOS の署名・公証と Windows のコード署名は行っていません)。SHA256SUMS で、ダウンロードしたファイルを確かめてください。`)。利用者向けの注意は 6.5 節 |
| 公開 | GitHub のリリースは、**下書き(draft)として作り**、人が確認して公開する(自動では公開しない)。版が先行版(SemVer の `-` を含む。例: `0.9.0-rc.1`)なら、プレリリースにする |
| NuGet への公開 | **行わない**。ワークフローは、NuGet のトークンを持たない |

### 6.2 ビルドの方法(決め)

```sh
dotnet restore --locked-mode
dotnet publish src/Conductor.Cli -c Release -r <RID> --no-restore -o publish/<RID>
dotnet run --project tools/Conductor.Checks -- dist publish/<RID> <RID>
```

| 項目 | 決め | 理由 |
|------|------|------|
| 発行の方式 | 単一ファイル(`PublishSingleFile=true`)・自己完結(`SelfContained=true`)。`Conductor.Cli.csproj` に、`Condition="'$(RuntimeIdentifier)' != ''"` で書く(`RuntimeIdentifier` を指定した発行のときだけ有効。通常のビルドとテストは、フレームワーク依存のまま)。確認(1 章): 単一ファイルは `PublishSingleFile`・`SelfContained`・`RuntimeIdentifier` で作り、RID ごとに別々に作る | 利用者が .NET を入れなくてよい |
| RID ごとの発行 | 各 RID を、**その OS の実行環境で**発行する(linux-x64 は `ubuntu-24.04`、linux-arm64 は arm64 の Linux の環境(GitHub のホストの arm64 のランナーのラベルは、`ubuntu-24.04-arm` と記憶している。未確認。なければ、`ubuntu-24.04` で `-r linux-arm64` を指定して、交差して発行する。この場合、発行物を実行して確かめられないので、AC-442・AC-528 を linux-arm64 では「未実施」と記録する)、osx-arm64 は `macos-latest`(Apple Silicon と記憶している。未確認)、win-x64 は `windows-2022`)。理由: macOS の arm64 の実行ファイルは、署名(最低でも ad-hoc 署名)がないと実行できない、と記憶している。.NET SDK が、macOS の上でビルドしたときに ad-hoc 署名を付けるかは、**公式の文書で確認していない(未確認)**。macOS のランナーで発行して、実機で動くことを確かめる | 動作確認ができる環境で作る |
| `RuntimeIdentifiers` | `Conductor.Cli.csproj` に、4 つの RID を `RuntimeIdentifiers` として書く(ロックファイルが、4 つの RID の依存を含むようにする。ロックファイルと RID を指定した復元の相性は、**公式の文書では確認していない(未確認)**。`--locked-mode` が失敗する場合は、`dotnet restore -r <RID> --locked-mode` を RID ごとに行い、ロックファイルの扱いを 8.4 節の決めに合わせる) | ロックファイルを使った再現のため |
| トリミング | **採らない**(`PublishTrimmed` を使わない)。理由: (1) YAML ライブラリの低水準の解析・出力(`Parser`・`Scanner`・`Emitter` の類)が、リフレクションを使わないかを、**この文書では確かめていない(未確認)**。静的ジェネレーターは、型を指定した復元(この仕様では使わない)のためのもので、低水準の API が、トリミングの警告を出さずに使えるかは、実機で確かめていない。(2) 大きさの利点(測っていない)より、トリミングで壊れて、実行時にだけ発覚するリスクが大きい。(3) 製品のコードが、リフレクションを使わない規定(AC-420)なので、将来、トリミングを試す余地はある。試す場合は、`IsTrimmable`・`EnableTrimAnalyzer` の警告をエラーにして、AC-488〜、受け入れテスト全体を、トリミングした発行物で通す | 安全側 |
| NativeAOT | **採らない**(`PublishAot` を使わない)。理由: (1) NativeAOT は、トリミングが必須で、リフレクション・動的な読み込みに制限がある(1 章で確認)。YAML ライブラリの低水準の API が、これを満たすかが未確認。(2) ビルド側に、OS ごとのネイティブのツールチェーンが要る(1 章で確認)。(3) 生成した Linux のバイナリは、ビルドした環境と同じかより新しい Linux でだけ動く(1 章で確認)ので、配布の互換性の条件が厳しくなる。(4) 起動時間の利点(NativeAOT は起動が速いとされる。公式の文書の記述。この文書では測っていない)は、目安の項目(2.1 節)で、利用者から見える挙動の基準ではない | 安全側 |
| ReadyToRun | **採らない**(`PublishReadyToRun` を使わない)。理由: 起動時間が改善するとされるが、この仕様の作成時には、効果も、各 RID(特に交差した発行)での可否も確認していない(未確認)。`startup-bench` に、任意のジョブ `publish-ready-to-run`(5.1 節)を置き、差を測ってから、採るか決める | 未確認のため、既定を変えない |
| 単一ファイルの圧縮 | 使わない(`EnableCompressionInSingleFile` を設定しない)。公式の文書が、起動時の費用を伴うと説明している(1 章)ため | 起動時間 |
| ネイティブのライブラリの同梱 | `IncludeNativeLibrariesForSelfExtract` は、**まず設定しない**。1 章で確認したとおり、管理コードの DLL は束ねられるが、ランタイム自身のネイティブのファイルは、設定がないと別のファイルとして残る、と読める記述がある(記述の範囲が、自己完結の単一ファイルの全体なのか、確認できていない)。発行の結果、実行ファイルのほかにファイルが残る場合(AC-526 が検出する)は、このプロパティを有効にする。**有効にすると、実行時に `DOTNET_BUNDLE_EXTRACT_BASE_DIR`、なければ `$HOME/.net`(Windows は `%TEMP%/.net`)へ展開する**(1 章)ので、`HOME` が未設定の環境で失敗する場合がある。その場合の注意を README に書く(AC-528) | 単一の実行ファイルにするため |
| デバッグ情報 | `<DebugType>none</DebugType>`(発行物に `.pdb` を含めない。AC-526)。例外のスタックトレースは、利用者に表示しない(AC-416)ので、利用者側では不要 | 成果物を 1 つにする |
| 不変グローバリゼーション | `<InvariantGlobalization>true</InvariantGlobalization>`(ICU に依存しない。AC-530。ICU のライブラリのない環境でも動く。1 章で確認した、NativeAOT の文書の記述と同じ考え方) | 環境差を除く |
| 版 | `Directory.Build.props` の `<Version>` だけ(AC-476)。コードは `AssemblyInformationalVersionAttribute` から読む。`<IncludeSourceRevisionInInformationalVersion>false` で、`+<コミット>` の付加を避ける | 版の定義を 1 か所にする |
| 決定的なビルド | `<Deterministic>true</Deterministic>`(既定)。CI では `ContinuousIntegrationBuild=true`(`Condition="'$(CI)' == 'true'"`)。ビルドの再現(同じ入力で、同じバイト列の実行ファイルになるか)は、**未確認**。確認できても、必須にしない | — |
| 署名 | しない(6.5 節) | — |

### 6.3 リリースの手順と `release.yml`

人の操作:

1. `Directory.Build.props` の `<Version>` を、新しい版に上げる(README の例も)。版は、SemVer の書き方(例: `0.9.0-rc.1`)で書く。`main` に取り込む。
2. `main` のそのコミットに、タグ `v<版>`(例: `v0.8.0`)を付けて push する。タグのコミットが `main` の祖先であること。タグの `v` を除いた文字列が、`Directory.Build.props` の `<Version>` と**完全に一致**すること。
3. ワークフローが、下書き(draft)のリリースを作る。人が内容を確認して、公開する(自動では公開しない)。
4. 人が、リリース前に Windows の実機で Ctrl+C のチェックリスト(AC-301、AC-487)を確かめる。

起動: `push` の `tags: ["v*"]`。権限は、ジョブごとに必要なものだけ(`draft-release` だけ `contents: write`)。

| ジョブ | 依存 | OS | 内容 | 失敗の条件 |
|--------|------|----|------|------------|
| `verify-tag` | なし | `ubuntu-24.04`(5 分) | `actions/checkout@<版>`(`fetch-depth: 0`)、`actions/setup-dotnet@<版>`(`global-json-file`)。bash: タグ名 `${GITHUB_REF_NAME}` から `v` を除いて `version`。① `git merge-base --is-ancestor "$GITHUB_SHA" origin/main`(タグのコミットが `main` の祖先)② `dotnet run --project tools/Conductor.Checks -- version "$version"`(`Directory.Build.props` の版が `version` と等しい)。出力: `version`、`prerelease`(`version` が `-` を含めば `true`) | タグが `main` 上にない(`tag is not on main`)、または版の不一致(`<場所>: <版> != <タグの版>`) |
| `checks` | `verify-tag` | — | `uses: ./.github/workflows/ci.yml`(5.1 節の全ジョブを再利用。マージの関門と同じ検査) | `startup-bench` は `continue-on-error` のため影響しない。それ以外のジョブの失敗で、以降が止まる |
| `build` | `verify-tag`、`checks` | RID ごと(`fail-fast: true`。30 分) | 下記の手順(発行 → 検査 → アーカイブ → 展開して動作確認 → アップロード) | 発行・検査・動作確認・アップロードのどれかの失敗 |
| `draft-release` | `verify-tag`、`build` | `ubuntu-24.04`(10 分。`permissions: contents: write`) | `actions/download-artifact@<版>`(`path: dist`、`merge-multiple: true`)。`cd dist && sha256sum * > ../SHA256SUMS && mv ../SHA256SUMS .`(`SHA256SUMS` 自身を含めないよう、先にファイル一覧を取る)。`gh release create "$TAG" dist/* --repo "$GITHUB_REPOSITORY" --title "$TAG" --draft --generate-notes [--prerelease] --notes "<注記>"`(トークンは `github.token`) | `gh` の失敗 |

`build` の行列(`include`):

| `rid` | `os` | 拡張 | 実行ファイル | 備考 |
|-------|------|------|--------------|------|
| `linux-x64` | `ubuntu-24.04` | `tar.gz` | `conductor` | 展開して `--version` と `validate` を実行 |
| `linux-arm64` | arm64 の Linux のランナー(ラベルは実装時に確認。6.2 節) | `tar.gz` | `conductor` | ランナーがなければ、`ubuntu-24.04` で交差して発行し、動作確認(smoke)を省く(`if: matrix.smoke`) |
| `osx-arm64` | `macos-latest` | `tar.gz` | `conductor` | 展開して実行 |
| `win-x64` | `windows-2022` | `zip` | `conductor.exe` | `7z` でアーカイブ(Windows のランナーに入っている) |

`verify-tag` の検査(bash):

```sh
version="${GITHUB_REF_NAME#v}"
git merge-base --is-ancestor "$GITHUB_SHA" origin/main || { echo "tag is not on main"; exit 1; }
dotnet run --project tools/Conductor.Checks -- version "$version" || exit 1
case "$version" in
  *-*) echo "prerelease=true" >> "$GITHUB_OUTPUT" ;;
  *)   echo "prerelease=false" >> "$GITHUB_OUTPUT" ;;
esac
echo "version=$version" >> "$GITHUB_OUTPUT"
```

`build` の手順(各 RID。bash(`shell: bash`)で書く):

```yaml
- uses: actions/checkout@<版>
- uses: actions/setup-dotnet@<版>
  with: { global-json-file: global.json, cache: true, cache-dependency-path: "**/packages.lock.json" }
- run: dotnet restore --locked-mode
- run: dotnet publish src/Conductor.Cli -c Release -r ${{ matrix.rid }} --no-restore -o publish
- run: dotnet run --project tools/Conductor.Checks -- dist publish ${{ matrix.rid }}
- name: package
  shell: bash
  run: |
    name="conductor-${{ needs.verify-tag.outputs.version }}-${{ matrix.rid }}"
    mkdir "$name"
    cp "publish/${{ matrix.exe }}" LICENSE "$name/"
    if [ "${{ matrix.ext }}" = "zip" ]; then
      7z a "$name.zip" "$name" > /dev/null
    else
      tar czf "$name.tar.gz" "$name"
    fi
    echo "ASSET=$name.${{ matrix.ext }}" >> "$GITHUB_ENV"
- name: smoke test (the archive is extracted and run)
  if: matrix.smoke
  shell: bash
  run: |
    mkdir smoke
    if [ "${{ matrix.ext }}" = "zip" ]; then 7z x "$ASSET" -osmoke > /dev/null; else tar xzf "$ASSET" -C smoke; fi
    bin=$(ls smoke/*/${{ matrix.exe }})
    out=$("$bin" --version)
    [ "$out" = "conductor ${{ needs.verify-tag.outputs.version }}" ] || { echo "unexpected: $out"; exit 1; }
    printf '{"format_version":1,"pipeline":{"name":"p"},"job":[{"id":"a","command":"echo hi"}]}' > smoke/conductor.json
    (cd smoke && "../$bin" validate)
- uses: actions/upload-artifact@<版>
  with:
    name: ${{ matrix.rid }}
    path: ${{ env.ASSET }}
    if-no-files-found: error
```

(このワークフローは、動作を確かめていない。受け入れテストを、展開した実行ファイルで通す(`CONDUCTOR_BIN`。AC-446)かは、`build` の手順に加えるか、`checks`(CI の全ジョブ)に任せるかを、実装時に決める。`checks` は、リポジトリの作業ツリーのビルドを試験するので、アーカイブから展開した成果物の試験は、`smoke` だけになる。受け入れテストまで通すなら、`build` の最後に、`CONDUCTOR_BIN="$PWD/$bin" dotnet test -c Release --filter "FullyQualifiedName~AcceptanceTests"` を足す。)

### 6.4 利用者への注意と、検証の手順

- ダウンロードしたら、`SHA256SUMS` で確かめる(`sha256sum -c SHA256SUMS`、macOS は `shasum -a 256 -c SHA256SUMS`、Windows は `certutil -hashfile <ファイル> SHA256` の出力を目で比べる)。これは、転送の破損の検知であって、配布元の真正性の保証ではない。
- 展開して、`conductor --version` を実行する。UNIX 系は、`tar xzf` が実行権限を保つ。

### 6.5 署名・公証をしないことと、利用者への注意

| OS | この仕様の方針 | 利用者への注意 |
|----|----------------|----------------|
| macOS | コード署名と公証を、**行わない** | ブラウザなどでダウンロードしたファイルには、検疫の属性が付き、Gatekeeper が実行を止める場合がある。信頼できる場合は、`xattr -d com.apple.quarantine ./conductor` で属性を外す(`curl` や `gh release download` でのダウンロードは、検疫の属性が付かない場合が多い。未確認)。Apple Silicon で、署名のない実行ファイルが実行を拒否される場合は、`codesign --force --sign - ./conductor`(ad-hoc 署名)で、実行できるようにする(**この手順の必要性と成否は、この文書では確かめていない(未確認)**)。README にこの注意を書く(AC-446 の手動の確認の項目) |
| Windows | コード署名を、**行わない** | SmartScreen が警告を出す場合がある。`SHA256SUMS` で確かめたうえで、利用者が許可する。README にこの注意を書く |
| Linux | 署名しない | `SHA256SUMS` で確かめる |

署名・公証を行わない理由: 証明書・Apple の開発者アカウントの費用と運用が要り、この仕様の範囲を超えるため。将来、行う場合は、`PrepareForBundle` と `GenerateSingleFileBundle` の間に署名を挟む拡張点が、.NET SDK にある(1 章で確認した、「Post-processing binaries before bundling」)。この仕様の受け入れ基準には含めない。

---

## 7. デモ

### 7.1 構成

デモは、`conductor` の主な動きを、1 本のスクリプトで順に見せる。利用者は、リポジトリの根から実行する。スクリプトは、一時ディレクトリに定義とジョブを写して実行し、終了時にそのディレクトリを消す(リポジトリには何も残らない)。デモの定義は JSON(場面 6 の `convert` で、YAML に変換して見せる)。

```text
demo/README.md                 使い方と各場面の説明
demo/conductor.json            UNIX 系の定義(パイプライン nightly)
demo/retry.json                UNIX 系のリトライの定義(パイプライン retry-demo)。場面 10 だけが使う
demo/run-demo.sh               UNIX 系のスクリプト(シェル)
demo/scripts/{extract,transform,load,notify,flaky}.sh   UNIX 系のジョブ
demo/windows/conductor.json, retry.json, run-demo.cmd   Windows 版(コマンドプロンプトのバッチ)
demo/windows/scripts/{extract,transform,load,notify,flaky}.cmd   Windows 版のジョブ
```

### 7.2 実行の仕方(UNIX 系と Windows の差)

| 項目 | UNIX 系 | Windows |
|------|---------|---------|
| 実行 | `demo/run-demo.sh`(リポジトリの根から) | `demo\windows\run-demo.cmd` |
| 実行ファイルの選択 | 環境変数 `CONDUCTOR`(`conductor` の実行ファイルのパス。直接実行する)。なければ、`dotnet run --project src/Conductor.Cli -c Release --` で実行する(.NET の SDK が要る。起動が遅い。`dotnet run` は、引数・標準入出力・終了コードを、そのまま通すと記憶している(未確認)。スクリプトは、ビルドを自動では行わない旨を案内するだけにしてもよい)。`dotnet` がなければ、`CONDUCTOR` を指定するよう案内して、終了コード 1 で終わる | 同じ(`CONDUCTOR`、なければ `dotnet run …`) |
| 表示の言語 | `DEMO_LANG`(既定 `ja_JP.UTF-8`。英語は `en_US.UTF-8`)を `LANG` に設定。`LC_ALL`、`LC_MESSAGES` は外す | `DEMO_LANG`(既定 `ja_JP.UTF-8`。`en_US` のようにも書ける)。`LC_ALL`、`LC_MESSAGES` は空にする。コードページを実行中だけ 65001(UTF-8)にして、終わりに戻す |
| 一時停止 | `DEMO_PAUSE=1` のときだけ、各場面のあとで Enter を待つ | 同じ |
| ジョブの待ち時間 | `DEMO_SLEEP`(既定 0.5 秒。小数可) | `DEMO_SLEEP`(整数の秒。既定 1。`0` で待たない。小数は使えない) |
| ジョブの終了コード | 環境変数 `DEMO_<ジョブ名の大文字>_EXIT`(既定 0) | 同じ |
| 作業場所 | `mktemp -d` の一時ディレクトリに定義とスクリプトを写して実行。終了時に `trap` で消す | `%TEMP%` の下の一時ディレクトリ(`conductor-demo-<乱数>`)に写して実行し、終わりに消す |
| 事前の環境変数の処理 | `DEMO_RUN_ID`、`CONDUCTOR_RUN_ID` を消す(利用者の環境の影響を除く)。`DOTNET_*` のうち表示・挙動を変えるもの(`DOTNET_SYSTEM_GLOBALIZATION_INVARIANT` など)は、触らない | 同じ(空にする) |
| 色 | 見出し・コマンド・終了コードの行に ANSI の色。`watch` も色あり(標準出力が端末のとき) | 画面に色なし |
| 各コマンドの出力 | `$ conductor <引数>` の行、実行、`→ 終了コード <N>` | 同じ |
| ステートの一覧 | `ls conductor.json*.state`(なければ `(なし)`) | `dir /b conductor.json*.state`(なければ `(なし)`) |
| 場面の見出し | `=== <番号>. <題> ===` | 同じ |

### 7.3 定義とジョブ

`demo/conductor.json`(Windows 版は、`command` が `scripts\<名>.cmd`、`notify` は `["cmd", "/c", "scripts\\notify.cmd"]`。JSON の文字列の中の `\` は `\\` と書く。**`.cmd` を起動する方法は 07 章に従う**。AC-510):

```json
{
  "format_version": 1,
  "pipeline": {
    "name": "nightly",
    "description": "デモ用の夜間バッチ",
    "run_id_env": "DEMO_RUN_ID"
  },
  "job": [
    {
      "id": "extract",
      "command": "sh scripts/extract.sh",
      "warning_exit_codes": [1],
      "transitions": [{"on": "warning", "next": "notify"}]
    },
    {"id": "transform", "command": "sh scripts/transform.sh"},
    {"id": "load", "command": "sh scripts/load.sh"},
    {"id": "notify", "command": ["sh", "scripts/notify.sh"]}
  ]
}
```

`demo/retry.json`:

```json
{
  "format_version": 1,
  "pipeline": {"name": "retry-demo", "description": "リトライのデモ"},
  "job": [
    {
      "id": "flaky",
      "command": "sh scripts/flaky.sh",
      "retry": {"exit_codes": [75], "max": 3, "interval": "1s"}
    }
  ]
}
```

ジョブのスクリプトの仕様(`extract`・`transform`・`load`・`notify`): `  [<名>] 実行中...` を表示する → `DEMO_SLEEP` の秒だけ待つ → `  [<名>] 終了コード <N> で終わります` を表示する → 環境変数 `DEMO_<名の大文字>_EXIT`(既定 0)の値 N で終わる。
`flaky` の仕様: カレントディレクトリの `flaky.count` の回数を 1 増やす(なければ 0 から)→ `  [flaky] <n> 回目の実行...` を表示する → 待つ → `n` が `DEMO_FLAKY_FAILS`(既定 2)以下なら `  [flaky] 終了コード 75 で終わります(一時的な失敗)` を表示して 75 で終わる。超えたら `  [flaky] 終了コード 0 で終わります` を表示して 0 で終わる。
(Windows 版のジョブは、待ちに `ping` を使うなど、実現の仕方が違ってよい。表示の文言は同じ。デモのジョブは、シェルのスクリプトであり、.NET を使わない。`conductor` 自身だけが .NET で動く。)

### 7.4 場面

| 場面 | 見せること | 実行するコマンド(UNIX 系。Windows も同じ引数) |
|------|-----------|--------------------------------------------------|
| 0 | 定義の中身 | `cat conductor.json`(Windows は `type`) |
| 1 | 全部成功。書いた順に 1 つずつ | `pipeline -v nightly` |
| 2 | `extract` が警告(`DEMO_EXTRACT_EXIT=1`)→ `transform`・`load` をスキップして `notify` へ | `pipeline -v nightly` |
| 3 | `load` のエラー(`DEMO_LOAD_EXIT=2`)で止まり、終了コード 2。止まった位置 | `pipeline -v nightly`、続けて `watch` |
| 4 | `load` から再開(`transform` は再実行しない)。完了してもステートは残り、`watch` が完了の行を出す。`--remove-on-complete` で完了時にステートを消す | `pipeline -v nightly`、`watch`、ステートの一覧、`pipeline -v --remove-on-complete nightly`、一覧、`watch` |
| 5 | 個別実行と初期化 | `run -v extract`、`reset` |
| 6 | JSON から YAML への変換(人が書く定義にコメントが要るときは YAML を使える、と案内する) | `convert conductor.json --to yaml` |
| 7 | 検査。正しい定義は 0(`definition is valid`)、引用符が閉じていない定義は 64(`command has an unclosed quote`)。ジョブは実行されず、ステートも作られない | `validate -v`、誤りの定義 `bad.json`(`{"format_version":1,"pipeline":{"name":"p"},"job":[{"id":"a","command":"sh -c 'echo hi"}]}`)を作って `validate --config bad.json`、一覧 |
| 8 | run ID ごとに別のステートファイル(`conductor.json.<run ID>.state`)。同じ run ID で再開。完了後もステートは残り、`reset --run-id` で片づける | `pipeline -v --run-id build-1 nightly`(`load` で失敗)、環境変数 `DEMO_RUN_ID=build-2` で `pipeline -v nightly`、一覧、`watch --run-id build-1`、`pipeline -v --run-id build-1 nightly`(再開)、一覧、`watch --run-id build-1`、`reset --run-id build-1`、`reset --run-id build-2`、一覧 |
| 9 | `--run-id` は環境変数に勝つ。`--run-id ""` は run ID なしに固定。`reset --run-id` は 1 つ、`reset --all` は全部。最後の `watch` は `no state` | `DEMO_RUN_ID=env-1` で `pipeline`(`load` で失敗)、`DEMO_RUN_ID=env-1` と `--run-id opt-1`、`--run-id ""`、`--run-id opt-2`、一覧、`reset --run-id opt-1`、一覧、`reset --all`、一覧、`watch` |
| 10 | `flaky` が 2 回、終了コード 75 で失敗し、`conductor` は 1 秒待つたびに標準エラーへ `conductor: retry: job 'flaky' exited with code 75; retry <n> of 3 in 1 seconds` を出して実行し直し、3 回目で成功(`job=flaky result=success`) | `rm -f flaky.count`(Windows は `del`)、`pipeline -v --config retry.json retry-demo` |

最後に `デモは以上です。`(Windows 版も同じ)を出す。デモの受け入れテストは 4.17 節。

---

## 8. 依存ライブラリと、中央パッケージ管理・ロックファイルの運用

### 8.1 方針

- 依存は最小にする。**実行時の外部ライブラリは、YAML の読み取り 1 つだけ**を認める。ほかは、BCL を使う(引数の解析は自前、シグナルは `System.Runtime.InteropServices`、プロセスは `System.Diagnostics.Process`、ファイルは `System.IO`、`System.Threading`、JSON の読み取りは `System.Text.Json` の `Utf8JsonReader` を土台にした自前の木の構築(他章の決定。下記)、`System.Numerics.BigInteger`)。YAML の書き出し(`convert --to yaml`)も自前([09](09-converter.md))。
- `System.CommandLine` などの引数解析のライブラリは、使わない(4.12 節の理由。ライブラリの文書を、この文書の作成時には確認していない(未確認))。
- 実行時の依存に、ネイティブのライブラリを必須としない(AC-443)。インストール時にコードを実行する仕組み(パッケージの `install.ps1` など)に頼らない。
- 開発時の依存は、`Microsoft.NET.Test.Sdk`、`xunit`、`xunit.runner.visualstudio`、`coverlet.collector`、`Microsoft.CodeAnalysis.BannedApiAnalyzers`(`GlobalPackageReference`)に限る(8.3 節)。.NET SDK に同梱されるアナライザー(`AnalysisLevel`)と、`dotnet format` は、パッケージではない。モックのライブラリ・テスト用の表明のライブラリ・擬似端末のライブラリは入れない(擬似端末が要る項目 AC-182・AC-183・AC-535 は、P/Invoke の補助で行う。10 章)。

### 8.2 YAML ライブラリの選定

選定の基準(すべて満たすこと):

| 基準 | 理由 | 確かめる受け入れテスト |
|------|------|------------------------|
| (1) 誤りと、各キー・値の行番号(できれば列番号)を、入力から取り出せる(構文要素の API で) | 定義の誤りを、行つきで出す([03](03-definition-format.md)) | AC-009、AC-037 |
| (2) 重複キーを検出できる(ライブラリの機能、または構文要素の走査で) | 重複キーを構文の誤りとして拒否する | AC-010、AC-226、AC-456 |
| (3) タグ(標準のタグも)・複数の文書・文字列でないキーを検出して拒否できる(明示のタグと、暗黙の解決を区別できる) | 定義と変換で拒否する | AC-217、AC-218、AC-453〜455、AC-531 |
| (4) 保守が続いている | 長く使う | `dotnet list package --deprecated`・脆弱性の検査、実装時の調査 |
| (5) アンカー・エイリアス・マージキーの展開を、自前で、上限つきで実装できる(エイリアスが、展開されずに取り出せる) | 爆発する入力に上限を付ける | AC-224〜230 |
| (6) 入れ子が深い入力で、スタックを使い果たさず失敗を返せる(または、深さを数えて早く打ち切れる。構文要素の API が再帰しない) | 2.2 節 | AC-232、AC-412、AC-524 |
| (7) 実行時の依存が少ない(望ましくはなし)。ネットワークを使わない。型を指定した復元を使わずに読める。型の判定を、自前で補える(YAML 1.2 のコアスキーマ) | 8.1 節 | AC-419、AC-443、AC-458、AC-525、AC-531 |
| (8) 標準エラーに警告を出さない | AC-498 | AC-498 |
| (9) `MIN_TFM` でも `MAX_TFM` でも動く。ネイティブの部品を持たない | 6 章 | AC-479 |
| (10) 単一ファイル・自己完結の発行(トリミングなし)で動く。リフレクションを避けられる(将来のトリミング・NativeAOT のため。採否は 6.2 節) | 6 章 | AC-526、AC-528 |

候補(版と保守の状況は、実装時に再確認する):

| 候補 | 特徴 | 評価 |
|------|------|------|
| `YamlDotNet`(NuGet のパッケージ名 `YamlDotNet`) | 対象は netstandard2.0・2.1、.NET 8.0、.NET 10.0、.NET Framework 4.7。MIT。低水準の解析と出力を持つと説明される(1 章。リポジトリの README)。AOT 向けには、別パッケージの静的ジェネレーター(`Vecc.YamlDotNet.Analyzers.StaticGenerator`)と `StaticDeserializerBuilder` がある(公開記事)。**低水準の `Parser` が、各イベントの開始位置(行・列・添字)・アンカー・タグ・スカラーの書式(引用符の有無)・書かれた文字列のままの値を返すか、深い入れ子で再帰しないか、重複キー・タグ・複数文書を検出できるか、エラーの例外の種類(位置を持つか)は、この文書では確認していない(未確認)**。公式のリポジトリの文書を、実装時に読んで確かめる | **採用案**。版は、実装時に最新の安定版を確認して、`Directory.Packages.props` に正確な版で固定する(8.4 節)。基準(1)〜(3)・(5)〜(10)は、実装時に、AC-010、AC-217〜232、AC-453〜460、AC-525、AC-531 の入力を、小さな試験で通して確かめる。1 つでも満たせない場合は、次の代案を再評価する |
| `SharpYaml` | 検索で存在を確認した(NuGet の版 3.x がある)。特徴・基準の適合は、この文書では確認していない | 代案(未評価)。`YamlDotNet` が基準を満たさない場合に、再評価する |
| `Meziantou.Framework.Yaml` | 検索で存在を確認した(版 1.0.x)。特徴・基準の適合は、この文書では確認していない | 代案(未評価) |
| 自前の YAML 読み取り | — | 採らない(YAML の文法が大きく、誤りが入りやすい) |

**採用案**: `YamlDotNet`(上のとおり版を固定)。YAML を読む **定義と変換の入口**でだけ使い、YAML の部品を呼ぶ型を別のクラスに分けて(2.1 節。遅延して初めて読み込まれる。AC-489)、`--version` などの起動を速くする。ライブラリの構文要素(低水準の `Parser`)を、03 章の「木」(順序つきの `entries`・`seq`・文字列・整数・浮動小数・真偽値・null)に変換する層を自前で書き、その層で、重複キー・タグ・複数文書・文字列でないキー・深さ・要素数・エイリアスの展開を扱う(`null` は拒否しない)。**型を指定した復元(`Deserializer`・`DeserializerBuilder`・`StaticDeserializerBuilder`)は、使わない**(暗黙の型判定と、型の生成を行うため。AC-421、AC-531)。ライブラリの例外のメッセージと、例外の型の名前は、表示に出さない(2.3 節)。必要なら、位置だけを使う。
実装時は、まず AC-010、AC-217〜232、AC-453〜460、AC-498、AC-525、AC-531 の入力を、小さな試験で候補に通し、基準(1)〜(10)を確かめてから決める。採用したライブラリと版は、`Directory.Packages.props`、README に記録する。

**JSON の読み取り**は、自前の字句・構文解析で行う([03](03-definition-format.md)。定義・ステート・変換の 3 つが同じ読み取りを使う。決定と理由は [06](06-state-file.md))。`System.Text.Json` を、どの水準で使うかは、03・06 章の決定に従う。土台にしてよい `Utf8JsonReader` について、この文書の作成時に公式の文書で確認できた範囲はない(1 章の確認の表に含まれない)。**この文書の他の章が前提とする性質(位置情報(`TokenStartIndex`、`BytePositionInLine` など)の取得、重複キーの検出、整数・小数・指数の字句の区別、`MaxDepth` の上限、コメント・末尾カンマの既定、BOM、不正な UTF-8 とサロゲートの扱い、巨大な整数の字句)は、実装時に、公式の文書で確かめる(未確認)。** 足りない部分は、自前の検査で補うか、全面自前にする(推奨は、薄い自前の木の構築)。`JsonSerializer` と `JsonDocument.Parse` は、製品のコードで使わない(AC-421)。JSON の出力は、09 章(`convert`)と 06 章(ステート)の規則に従う自前の書き出しとする(AC-469)。

### 8.3 開発時の依存の一覧

版は、すべて「実装時に確認する」。`MIN_TFM` と `MAX_TFM` の両方で動く版を選ぶ。`Directory.Packages.props` には、正確な版を書く(8.4 節)。

| パッケージ | 用途 | 選定の基準 |
|------------|------|------------|
| `Microsoft.NET.Test.Sdk` | テストのホスト(`dotnet test`) | `coverlet.collector` が要求する最低の版(coverlet の文書の例は 18.4.0 以上。1 章)を満たす |
| `xunit` | テストの枠組み(`Fact`・`Theory`・`Assert`・`CollectionBehavior`) | **v2 系**(3.1 節の理由。v3 は MTP で動き、`coverlet.collector` と併用できない) |
| `xunit.runner.visualstudio` | VSTest のアダプター | `xunit` と矛盾しない版 |
| `coverlet.collector` | カバレッジ(`dotnet test --collect:"XPlat Code Coverage"`) | MTP と併用できない(1 章)ので、VSTest を使う。SDK 8.0.414 以上(1 章) |
| `Microsoft.CodeAnalysis.BannedApiAnalyzers` | `BannedSymbols.txt` による禁止の API の検出(`GlobalPackageReference`。`PrivateAssets` と `IncludeAssets` は、`GlobalPackageReference` の既定で開発時の依存になる。1 章) | 診断 ID は、採用した版で確かめる(未確認) |

上記以外の開発時の依存(`Moq`・`NSubstitute`・`FluentAssertions`・`Shouldly`・`AutoFixture`・`Bogus`・`FsCheck`・`NUnit`・`MSTest`・`StyleCop.Analyzers`・`Roslynator`・`SonarAnalyzer`・`dotnet-coverage`(ツール)・`ReportGenerator` など)は入れない(`Conductor.Checks deps` が検査する。5.4 節。レポートの見た目が要る場合は、手元の道具として使ってよいが、リポジトリの依存には入れない)。

### 8.4 中央パッケージ管理(`Directory.Packages.props`)とロックファイル(`packages.lock.json`)の運用

| 項目 | 決め |
|------|------|
| 中央パッケージ管理 | リポジトリの根の `Directory.Packages.props` に、`<ManagePackageVersionsCentrally>true</ManagePackageVersionsCentrally>` と、`PackageVersion`(許可の一覧のすべて)を書く。各プロジェクトの `PackageReference` は `Version` を書かない(1 章で確認)。最も近い `Directory.Packages.props` だけが使われる(`tests/` の下に別の `Directory.Packages.props` を置かない) |
| 版の書き方 | **正確な版**(範囲・`*` を使わない。`Conductor.Checks deps` が検査する)。`CentralPackageTransitivePinningEnabled` は、**設定しない**(既定のまま)。理由: 推移的な依存の固定は、便利だが、版を下げる固定は拒否される(1 章)。ロックファイルで、推移的な依存の版を固定するため、重ねて使わない。推移的な依存に脆弱性が見つかったときの、対処として使う場合は、その時点で有効にし、理由を記録する |
| ロックファイル | `Directory.Build.props` の `<RestorePackagesWithLockFile>true</RestorePackagesWithLockFile>` で、すべてのプロジェクト(`src/`・`tests/`・`tools/`)に `packages.lock.json` を作る。**コミットする**(実行ファイルのプロジェクトは、ロックファイルをコミットする。1 章。ライブラリのプロジェクト(`Conductor.Core`・`Conductor.Runtime`)は、実行ファイルから参照されるので、ロックファイルの用途は限られる(1 章の説明)が、一貫のため、すべてコミットする) |
| 入れ方 | CI とリリースは、`dotnet restore --locked-mode`(`RestoreLockedMode`)だけで入れる。ロックファイルと依存の定義が食い違うと、復元が失敗する(1 章)。ローカルで依存を変えたときは、`dotnet restore`(ロックモードでなく)で、ロックファイルを更新して、一緒にコミットする |
| `global.json` と SDK | `sdk.version` を完全な版で固定し、`rollForward` を `disable` にする(1 章: ロックファイルを使うときは、SDK が暗黙に足すパッケージの版が SDK で変わるため、`disable` が勧められている)。SDK を更新するときは、`global.json`、README、ロックファイルを、同じコミットで更新する(AC-477) |
| パッケージ元 | `NuGet.config` に、nuget.org だけ(`<clear />` のあと 1 つ)。複数あると、中央パッケージ管理が `NU1507` を出す(1 章)。`packageSourceMapping` で対応づけてもよい |
| 更新 | 直接の依存の版を上げるときは、`Directory.Packages.props` とロックファイルを同じコミットで更新し、`dotnet test`(3 OS)、監査(5.2 節)を通す。自動の更新ツール(Dependabot など)の導入は任意(未確認) |
| 検査 | CI の `consistency`(5.1 節): `Conductor.Checks deps`(許可の一覧、中央管理、ロックファイルの存在、パッケージ元)、`dotnet restore --locked-mode` |
| 弱点 | ロックファイルは、同じ SDK・同じ OS での復元の再現を保つ。OS・RID が違うと、必要な推移的な依存が違う場合がある(例: プラットフォーム固有のパッケージ)。`RuntimeIdentifiers` による復元が、3 つの OS で同じロックファイルを受け入れるかは、**未確認**(6.2 節)。受け入れない場合は、`--locked-mode` を OS ごとのロックファイルで運用する(`NuGetLockFilePath`。1 章)ことを検討する |
| `[tool.*]` に相当する設定 | ビルドの設定は、`Directory.Build.props`、`.editorconfig`、`xunit.runner.json`、`coverlet.runsettings` に集める(9 章)。別の設定ファイル(`stylecop.json`、`.globalconfig` など)は、置かない |

---

## 9. リポジトリのレイアウト

### 9.1 ファイルの構成

```text
Conductor.sln
global.json
Directory.Build.props
Directory.Packages.props
NuGet.config
.editorconfig
BannedSymbols.txt
coverlet.runsettings
.gitignore
LICENSE
README.md
.github/workflows/{ci,audit,release}.yml
src/Conductor.Core/             Conductor.Core.csproj、packages.lock.json
                                  Definition/ Convert/ Json/ RunIds/ Flow/
src/Conductor.Runtime/          Conductor.Runtime.csproj、packages.lock.json
                                  State/ Runner/(PlatformInfo.cs を含む)
src/Conductor.Cli/              Conductor.Cli.csproj(AssemblyName=conductor)、packages.lock.json
                                  Report/ Cli/ Program.cs
tests/Conductor.Tests/          単体。Support/ Inspect/ Fixtures/ Perf/ ほか
tests/Conductor.AcceptanceTests/  受け入れ。Support/ Perf/ xunit.runner.json
tests/Conductor.TestJob/        テスト用ジョブ(AssemblyName=tj)
tests/WINDOWS_CTRL_C_CHECKLIST.md
tools/Conductor.Checks/         5.4 節の検査プログラム
demo/                           7 章
docs/                           (この仕様書など。ビルドに関与しない)
```

- 各プロジェクトの `TargetFramework(s)` は、`$(ConductorMinTfm)` と `$(ConductorMaxTfm)` を使い、直書きしない(AC-477)。
- `ImplicitUsings` は `disable`(明示の `using`)。
- `bin/`・`obj/`・`TestResults/`・`publish/`・`.vs/`・`.idea/` は `.gitignore`(9.5 節)。`packages.lock.json` は、コミットする。

### 9.2 `Directory.Build.props`

```xml
<Project>
  <PropertyGroup>
    <!-- 版の唯一の出所(AC-476) -->
    <Version>0.8.0</Version>
    <IncludeSourceRevisionInInformationalVersion>false</IncludeSourceRevisionInInformationalVersion>

    <!-- サポートする .NET の版(2.5 節。実装時に確認して決める) -->
    <ConductorMinTfm>net10.0</ConductorMinTfm>
    <ConductorMaxTfm>net10.0</ConductorMaxTfm>

    <LangVersion>latest</LangVersion>
    <Nullable>enable</Nullable>
    <ImplicitUsings>disable</ImplicitUsings>
    <TreatWarningsAsErrors>true</TreatWarningsAsErrors>
    <EnforceCodeStyleInBuild>true</EnforceCodeStyleInBuild>
    <AnalysisLevel>latest</AnalysisLevel>
    <AnalysisMode>Recommended</AnalysisMode>
    <Deterministic>true</Deterministic>
    <InvariantGlobalization>true</InvariantGlobalization>
    <GenerateDocumentationFile>false</GenerateDocumentationFile>

    <!-- ロックファイル(8.4 節) -->
    <RestorePackagesWithLockFile>true</RestorePackagesWithLockFile>

    <!-- 監査の警告は、通常のビルドではエラーにしない。監査の流れ(AuditPipeline=true)だけエラーにする(5.2 節。1 章) -->
    <NuGetAuditCodes>NU1900;NU1901;NU1902;NU1903;NU1904;NU1905</NuGetAuditCodes>
    <WarningsAsErrors Condition=" '$(AuditPipeline)' == 'true' ">$(WarningsAsErrors);$(NuGetAuditCodes)</WarningsAsErrors>
    <WarningsNotAsErrors Condition=" '$(AuditPipeline)' != 'true' ">$(WarningsNotAsErrors);$(NuGetAuditCodes)</WarningsNotAsErrors>
  </PropertyGroup>

  <PropertyGroup Condition=" '$(CI)' == 'true' ">
    <ContinuousIntegrationBuild>true</ContinuousIntegrationBuild>
  </PropertyGroup>

  <!-- 製品のコード(src/)にだけ、禁止の API の一覧を適用する -->
  <ItemGroup Condition=" $(MSBuildProjectDirectory.Replace('\', '/').Contains('/src/')) ">
    <AdditionalFiles Include="$(MSBuildThisFileDirectory)BannedSymbols.txt" />
  </ItemGroup>
</Project>
```

(`AnalysisMode` の値、`AnalysisLevel` の書式、`Condition` の中の `Replace`・`Contains` の書き方は、確認していない。実装時に確認する。`InvariantGlobalization` は、`tests/` と `tools/` にも効くが、問題ない。`CheckForOverflowUnderflow` は設定しない(AC-541)。)

`src/Conductor.Cli/Conductor.Cli.csproj`:

```xml
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>$(ConductorMinTfm)</TargetFramework>
    <AssemblyName>conductor</AssemblyName>
    <RootNamespace>Conductor.Cli</RootNamespace>
    <RuntimeIdentifiers>linux-x64;linux-arm64;osx-arm64;win-x64</RuntimeIdentifiers>
    <DebugType>none</DebugType>
  </PropertyGroup>
  <!-- RID を指定した発行のときだけ、単一ファイル・自己完結(6.2 節) -->
  <PropertyGroup Condition=" '$(RuntimeIdentifier)' != '' ">
    <SelfContained>true</SelfContained>
    <PublishSingleFile>true</PublishSingleFile>
  </PropertyGroup>
  <ItemGroup>
    <ProjectReference Include="..\Conductor.Runtime\Conductor.Runtime.csproj" />
    <PackageReference Include="YamlDotNet" />
  </ItemGroup>
</Project>
```

(YAML ライブラリへの参照は、`Conductor.Core` の `Convert`・`Definition` が使うなら、`Conductor.Core.csproj` に置く。どのプロジェクトに置くかは 02 章の構成に従う。`Conductor.Checks deps` は、`src/` の下のどのプロジェクトでも、YAML ライブラリだけを許す。)

### 9.3 `BannedSymbols.txt` と `.editorconfig`

`BannedSymbols.txt`(1 行に、ドキュメント ID と、任意で `;説明`。コメントは `//`。1 章で確認した書式。ドキュメント ID の厳密な書き方(ジェネリックの型、引数の型の完全修飾)は、C# の仕様の「ID string format」に従う。**以下の ID は、この文書では、コンパイルして確かめていない(未確認)。実装時に、AC-518 の試験で、すべての行が効くことを確かめる**):

```text
// 文化依存(AC-540。InvariantCulture か序数を明示する)
M:System.String.ToUpper;use ToUpperInvariant or an ordinal operation
M:System.String.ToLower;use ToLowerInvariant or an ordinal operation
M:System.String.Compare(System.String,System.String);use string.CompareOrdinal
M:System.String.CompareTo(System.String);use string.CompareOrdinal
M:System.String.StartsWith(System.String);pass StringComparison.Ordinal
M:System.String.EndsWith(System.String);pass StringComparison.Ordinal
M:System.String.IndexOf(System.String);pass StringComparison.Ordinal
M:System.String.Format(System.String,System.Object);use invariant formatting
M:System.Int32.Parse(System.String);parse the lexeme by hand (AC-543)
M:System.Int64.Parse(System.String);parse the lexeme by hand (AC-543)
M:System.UInt64.Parse(System.String);parse the lexeme by hand (AC-543)
M:System.Double.Parse(System.String);numbers are read from lexemes (AC-502)
P:System.Globalization.CultureInfo.CurrentCulture;use CultureInfo.InvariantCulture
P:System.Globalization.CultureInfo.CurrentUICulture;use CultureInfo.InvariantCulture

// 異常終了・終了(AC-421、AC-500)
M:System.Environment.Exit(System.Int32);only Program.cs may exit (AC-500)
M:System.Environment.FailFast(System.String);do not fail fast
M:System.Environment.GetCommandLineArgs;use the args of Main (AC-547)

// シェル・プロセス(AC-116)
M:System.Diagnostics.Process.Start(System.String);use ProcessStartInfo with UseShellExecute=false
M:System.Diagnostics.Process.Start(System.String,System.String);use ProcessStartInfo with ArgumentList

// 出力は Report と Stream を通す(AC-421、AC-520)
P:System.Console.Out;write through the output sinks
P:System.Console.Error;write through the output sinks
P:System.Console.OutputEncoding;do not change the console encoding
M:System.Console.WriteLine;write through the output sinks
M:System.Console.Write;write through the output sinks

// 動的なコード・読み込み(AC-420)
M:System.Reflection.Assembly.Load(System.String);no dynamic loading
M:System.Reflection.Assembly.LoadFrom(System.String);no dynamic loading
M:System.Activator.CreateInstance(System.Type);no dynamic instantiation
M:System.Type.GetType(System.String);no type lookup by name
T:System.Runtime.Serialization.Formatters.Binary.BinaryFormatter;insecure

// 同期の待ち(決定 8。デッドロックを作らない)
M:System.Threading.Tasks.Task.Wait;do not block on tasks
P:System.Threading.Tasks.Task`1.Result;do not block on tasks
M:System.Threading.Thread.Abort;not supported
```

(`Console.WriteLine`・`Console.Write` は、オーバーロードが多い。ID に引数の型を付けずに書くと、すべてのオーバーロードに効くかは、確認していない(未確認)。)
`.editorconfig` の要点(キー名は、.NET の公式のコードスタイルの文書で、実装時に確かめる):

```ini
root = true

[*]
charset = utf-8
end_of_line = lf
insert_final_newline = true
indent_style = space
indent_size = 4

[*.{csproj,props,targets,json,yml,yaml}]
indent_size = 2

[*.cs]
# 命名(2.4 節)。違反はビルドの警告(= -warnaserror でエラー)
dotnet_naming_rule.private_fields_underscore.severity = warning
dotnet_naming_rule.private_fields_underscore.symbols = private_fields
dotnet_naming_rule.private_fields_underscore.style = underscore_camel
dotnet_naming_symbols.private_fields.applicable_kinds = field
dotnet_naming_symbols.private_fields.applicable_accessibilities = private
dotnet_naming_style.underscore_camel.required_prefix = _
dotnet_naming_style.underscore_camel.capitalization = camel_case
# 文化依存の API・比較(CA1304、CA1305、CA1307、CA1309、CA1310。BannedSymbols.txt と二重に守る)
dotnet_diagnostic.CA1304.severity = error
dotnet_diagnostic.CA1305.severity = error
dotnet_diagnostic.CA1307.severity = error
dotnet_diagnostic.CA1309.severity = error
dotnet_diagnostic.CA1310.severity = error
# using の整理、var の規則などは、実装時に決める
```

(`CA1304`・`CA1305`・`CA1307`・`CA1309`・`CA1310` は、文化・文字列比較に関する、.NET のコード分析の規則と記憶している。規則の番号と内容は、確認していない(未確認)。)

### 9.4 `global.json`、`Directory.Packages.props`、`NuGet.config`

```json
{
  "sdk": {
    "version": "10.0.100",
    "rollForward": "disable"
  }
}
```

(版 `10.0.100` は、書式の例で、確認した値ではない。1 章で確認した書式: `version` は完全な版、`rollForward` は `disable` など。)

```xml
<Project>
  <PropertyGroup>
    <ManagePackageVersionsCentrally>true</ManagePackageVersionsCentrally>
  </PropertyGroup>
  <ItemGroup>
    <!-- 実行時の依存(1 つ)。版は実装時に確認 -->
    <PackageVersion Include="YamlDotNet" Version="<実装時に確認>" />
    <!-- 開発時の依存(8.3 節) -->
    <PackageVersion Include="Microsoft.NET.Test.Sdk" Version="<実装時に確認>" />
    <PackageVersion Include="xunit" Version="<実装時に確認。2.x>" />
    <PackageVersion Include="xunit.runner.visualstudio" Version="<実装時に確認>" />
    <PackageVersion Include="coverlet.collector" Version="<実装時に確認>" />
    <GlobalPackageReference Include="Microsoft.CodeAnalysis.BannedApiAnalyzers" Version="<実装時に確認>" />
  </ItemGroup>
</Project>
```

```xml
<?xml version="1.0" encoding="utf-8"?>
<configuration>
  <packageSources>
    <clear />
    <add key="nuget.org" value="https://api.nuget.org/v3/index.json" />
  </packageSources>
</configuration>
```

### 9.5 `.gitignore`

```text
bin/
obj/
TestResults/
publish/
.vs/
.idea/
*.user
```

(`packages.lock.json` は、無視しない。)

---

## 10. 未確認の事項と、他の章との整合が要る点

### 10.1 未確認の事項

この文書の作成時には、.NET SDK のない環境で、公式の文書の記述だけを確かめた。次は、確認できなかった、または実機で動かして確かめていない項目である。実装時に、実機で確かめ、結果を、この文書の該当の項目に反映する。

| 区分 | 項目 | 関連 |
|------|------|------|
| 実行・ビルド | この文書の C# のコード(`Xorshift32`、`TJ` の骨組み、`Conductor.Checks`)、各 YAML・XML・設定のコードは、ビルドも実行もしていない。`dotnet test` の `--framework`・`--collect`・`--settings`・`--blame-hang-timeout` の併用、`xunit.runner.json` のキー名、`Directory.Build.props` の `Condition` の式、`AnalysisMode` の値は未確認 | 3.4、3.9、5.1、5.4、9.2 |
| 性能 | 起動時間と、各 p95 の基準は、すべて仮で、測っていない。ReadyToRun・トリミング・NativeAOT・圧縮の有無での差は未確認 | 2.1、AC-488 |
| .NET の挙動 | `Process.ExitCode` が、シグナルで終了した子に返す値(公式の文書に記述なし)。不正な UTF-8 の引数が `string[] args` になる形。`Process.Start` が NUL・不正な名前で投げる例外の型。SIGPIPE の扱いと、閉じたパイプへの書き込みの例外(`IOException`)。`Console.CancelKeyPress` を登録していないときの `SIGINT` の既定の動作。`SIGQUIT` の既定の動作。`SIGCHLD` が `SIG_IGN` で継承されたときの子の回収。`PosixSignalContext.Cancel` の意味。同じシグナルへの `PosixSignalRegistration` の複数の登録。`PosixSignal` の列挙子の値と、負の値の列挙子。`Utf8JsonReader` の位置情報・重複キー・コメントの既定・BOM・不正な UTF-8・孤立サロゲート・巨大な数値の扱い。`char.IsWhiteSpace`・`string.Trim()` の集合。`Path.GetExtension` の細部。`Regex` の `$`・`\d`。`FileInfo` の FIFO の報告。`Dictionary` の列挙順。`double.ToString("R")` の書式。`double.Parse("1e999")` の結果。`File.Move(overwrite)`・`File.Replace` の Windows での挙動。`FileSystemInfo.LinkTarget` の挙動 | 各 AC に併記 |
| 単一ファイル | 単一ファイル・自己完結の発行物が、実行ファイル 1 つだけになるか(ランタイムのネイティブのファイルが残るか)。`IncludeNativeLibrariesForSelfExtract` の要否。macOS の arm64 で、ad-hoc 署名が自動で付くか | 6.2、AC-526、AC-528 |
| YAML ライブラリ | `YamlDotNet` の低水準 API が、基準(1)〜(10)を満たすか。トリミング・NativeAOT での動作。SharpYaml、Meziantou.Framework.Yaml の評価 | 8.2 |
| ロックファイル | `RuntimeIdentifiers` を指定したプロジェクトの `--locked-mode` が、3 つの OS で同じロックファイルを受け入れるか。`actions/setup-dotnet` の `cache-dependency-path` が、ワイルドカードを受け付けるか | 5.1、6.2、8.4 |
| CI | `ubuntu-24.04-arm` と `macos-latest`(Apple Silicon)のラベル。GitHub Actions の式の関数(`substring` がないこと)。`actions/*` の最新の版。`dotnet list package --vulnerable` の終了コード。`dotnet run` の標準出力へのビルドの出力の混入。ブランチ保護の設定 | 5、6 |
| テスト | 疑似端末の `openpty` の P/Invoke。`TJ` のアプリホストの起動(`DOTNET_ROOT`)。FIFO・`SIGCHLD`・`Ctrl+Break` の自動テスト(`CreateProcessW` の `CREATE_NEW_PROCESS_GROUP`)。`TJ` の起動の費用。`System.Random` の列の安定。`[Theory]` の ID の表示 | 3、4.20 |
| Windows | `Process.ExitCode` の `-1`・`256` の扱い。Ctrl+C・Ctrl+Break の自動テストと手動の記録。`.cmd` の起動(`BatBadBut` 対策を含む)。置き換え先が開かれているときの `File.Move` | AC-112、AC-301、AC-510、AC-546 |
| 手順 | macOS の署名・公証と Windows のコード署名は、行わない(方針)。利用者向けの回避手順(`xattr`、`codesign`)の成否は未確認 | 6.5 |

### 10.2 他の章との整合が要る点

- **02 章(構成)**: (1) `Conductor.Cli` が、`Conductor.Cli.EntryPoint.Run(string[] args, Deps deps)`(終了コードを返す)を公開し、`Program.Main` は、これを呼んで、結果を返すだけの薄い入口にする(AC-405、AC-490〜491、AC-496。3.4 節の `inject-*` モードが使う)。`Conductor.TestJob` は `Conductor.Cli` を参照する。(2) `Deps` のフィールド(`Env`・`Clock`・`Defs`・`States`・`Converter`・`Proc`・`Signals`・`Out`)と、偽の部品の名前(3.7 節)は、02 章に合わせる。(3) `PlatformInfo.cs`(`ForWindows()`・`ForUnix()`・`PrepareSpawn`・`ExitByInterrupt()`)の名前と置き場所は、02・07 章に合わせる。(4) YAML ライブラリへの参照の置き場所(9.2 節)。(5) `tools/Conductor.Checks` を、ソリューションに加える(決定の固定の構成に、`tools/` を足した)。
- **03 章(定義)**: JSON の理由の字句(AC-008、AC-452、AC-465、AC-467)、拡張子の判定の規則(AC-545)、空白の集合(AC-523)、`command` の NUL の扱い(AC-522)、`Utf8JsonReader` を土台にした自前の木の位置情報(AC-461〜475)、YAML ライブラリのタグの属性(AC-453)の細部。
- **06 章(ステート)**: 原子的な置換の API(AC-322、AC-546)、ステートの書き出しの自前の書式と、キーの出力順(AC-147)、`Dictionary` の順序に頼らないこと。
- **07 章(プロセスとシグナル)**: `Process.ExitCode` とシグナル終了の対応と曖昧さ(AC-140、AC-504)、子の新しいセッション・プロセスグループの決定(AC-288、AC-535)、Windows の Ctrl+Break の番号(AC-301、AC-533)、`.cmd` の起動の方法(AC-113〜115、AC-486、AC-510、AC-513)、待ちの方式と区間の長さ(AC-275、AC-403、AC-503)、`PosixSignalRegistration` の保持と `Cancel` の扱い(AC-537〜539)、P/Invoke の許可の一覧(AC-420)、スレッド・タイマーの許可の一覧(AC-420、AC-497、AC-499)。
- **08 章(出力)**: 標準出力・標準エラーの作り方(`Console.OpenStandardOutput` から BOM なしの UTF-8、LF、`Flush`)(AC-520、AC-521)、孤立サロゲートの扱いと表示(AC-519)、使い方の誤りの文、`--help`・`--version` の書き込みの失敗の扱い(AC-515)。
- **09 章(変換)**: 浮動小数点数の書式(AC-234、AC-544)、JSON の出力の文字のエスケープ(AC-469)、YAML の出力で引用符を付ける条件(AC-470)、FIFO の入力の扱い(AC-536)、変換の上限(AC-229〜231)。
- **10 章(終了コード)**: 異常終了の経路(`UnhandledException` のハンドラ、`TaskScheduler.UnobservedTaskException`、`OutOfMemoryException`、`StackOverflowException`)と 70(AC-490、AC-491、AC-548)。
- **12 章(再構築の手順)**: 品質ゲートを再現する順序(5.5 節)、リリースの手順(6 章)、本章の未確認の事項(10.1 節)の、実装時の確認の順序。
