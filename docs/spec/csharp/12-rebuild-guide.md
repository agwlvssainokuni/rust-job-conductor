# 12. 再構築の手引き(C# でゼロから実装する)

この文書は、`conductor` を C#(.NET)でゼロから実装する人のための、実装の順序、.NET に特有の落とし穴、未確認事項、最初に確かめる小さな試験をまとめる。各規定の本文は、02〜11 章にある。この文書は、それらへの道案内であり、規定を足さない。食い違いがあれば、各章を正とする。

- 規定の強さの語は [00-README.md](00-README.md) 4 章。
- 受け入れテスト(AC)は [11-quality-and-build.md](11-quality-and-build.md) 4 章。ここで「AC-xxx〜yyy」と書くのは、その範囲の項目を指す。
- この仕様書を書いた環境には .NET SDK がなく、挙動は、公式ドキュメントと公開ソースの読み取りで確かめた範囲だけが「確認」である。実機で動かして確かめたものは無い。したがって、3 章の未確認事項と、4 章の試験は、実装の最初に行う。
- サポートする .NET は .NET 10(`net10.0`)だけ(作成時点。確認: .NET の公式のサポートポリシーのページ。.NET 10 は LTS でサポート終了 2028-11-14、.NET 8 と 9 は 2026-11-10 に終了)。実装時に、サポートの状況と次の LTS を確認して決める([00](00-README.md) 1 章)。

## 1. 実装の順序

依存の向き([02](02-architecture.md) 3 章)の下から上へ、かつ「純粋な部分を先に、入出力を後に」の順で作る。各段階は、完了条件を満たしてから次へ進む。段階ごとの確認は、その段階の単体テストと、11 章の受け入れテストのうち、その段階で動くものの範囲で行う。受け入れテストは、ビルドした `conductor` を `Process` で外から動かすので、`Conductor.Cli` までそろった段階 8 から本格的に使える。それ以前は、単体テストと、偽の `Deps`(02 章 4.4 節)で確かめる。

| 段階 | 作るもの | 読む章 | 完了条件 | 確認する方法 |
|------|----------|--------|----------|--------------|
| 0 | 準備: `Conductor.sln`、`global.json`(`10.0.100` + `latestFeature`)、`Directory.Build.props`(`<Version>0.8.0</Version>`、`ConductorMinTfm`/`ConductorMaxTfm`、アナライザー、ロックファイル)、`Directory.Packages.props`、`NuGet.config`、`BannedSymbols.txt`、`.editorconfig`、3 つの `src/` と 3 つの `tests/` と `tools/Conductor.Checks` のプロジェクト、`ArchitectureTests`(依存の向き・循環・OS 分岐・禁止の API)、`VersionTests`、最初の試験(4 章) | 02 の 2・3・7・8 章、11 の 5・8・9 章 | `dotnet format --verify-no-changes`、`dotnet build -warnaserror`、`dotnet test`(空でも)、`dotnet publish -r <RID>` が通る。`conductor --version` が `conductor 0.8.0` を出す(版が `AssemblyInformationalVersionAttribute` から読まれ、`+<コミット>` が付かない)。4 章の試験 T1〜T3 の結果で、YamlDotNet の採否が決まっている | AC-476(版)、AC-477(.NET の版の記載)、AC-478(依存の整合)、AC-479(ビルド)、AC-480・483(静的検査の設定)、AC-507・508(起動とプロジェクトの形)、AC-514(依存の向き)、AC-518(禁止の API の規則が有効であること) |
| 1 | 土台: `Conductor.Text.Escaping`、`ExitCodes.For(int, bool)`・`SignalExitCode`・`Saturating`(`Conductor.Flow`)、`Conductor.RunIds`、エラーの型(10 章 8 章)、入口 `Program.Main` と `InternalFailure`(`UnhandledException`・`UnobservedTaskException`・`catch (Exception)` で 70、`Environment.Exit` は `ReportAndExit` だけ)、`EntryPoint.Run` の空の骨格、出力の `IOutputSink`(`StreamWriter`、BOM なし、LF、`Flush`) | 10、05 の 2・6 章、08 の 1・9 章、07 の 10 章 | 終了コードの範囲合わせ(259→3、256→エラーなら 1、`int.MinValue`)、run ID の有効・無効の例(30 文字の日本語が `TooLong`)が、単体テストで通る。偽の失敗(例外)で、固定の 1 行と 70 が出る。出力の失敗で終了コードが変わらない | 単体テスト(10 章 4 章の例、05 章 6.1 節の例)。AC-389〜396(終了コード全件)、AC-414、AC-490・491・548(内部の誤り)、AC-496(終了時の flush)、AC-501(飽和演算) |
| 2 | 引数の解析(字句の段階と検査の段階。`IsSuspectText`)、`--help`、`--version`、使い方の全文 | 04、08 の 4・8 章 | 04 章の判定表の全行が、表駆動テストで通る。使い方の誤りは 68 で、誤りの行・空行・使い方の全文が出る。`System.CommandLine` を使っていない。サロゲートを含む引数で落ちない | AC-325〜349、AC-447〜449、AC-405・406(ランダム・不正な引数)、AC-515(`--help` の標準出力が閉じたパイプ) |
| 3 | 定義の読み込み: `Conductor.Json`(木の型、自前の `JsonReader`、`Limits`、`Utf8Text`)、取り込み(拡張子、大きさ、UTF-8、BOM をバイトで除く、空パスは `fileMissing`)、YAML の読み取り(YamlDotNet の `Parser`/`Scanner` + 追加の実装 14 項目)、展開と上限、版の確認、構造の検査(Walk)、ジョブ間の検査、位置の出し方、`LooseRunIdEnv` | 03、10 の 7.1 節 | 03 章の受理・拒否の例が、すべて表駆動テストで通る。誤りは位置の順に、種類・行・固定の理由で返る(安定な `OrderBy`)。深い入力でも異常終了しない(再帰を使わない) | AC-001〜075、AC-450〜475(`validate` を動かせる段階 8 以降でも確認)。単体テストでは 03 章 7.5 節の試験、`Utf8JsonReader` との差分試験(03 章 4.4 節)、AC-412(深い入れ子)、AC-502(整数の検査)、AC-511・512、AC-522〜525、AC-531、AC-545 |
| 4 | `command` の分割と検査(状態機械。UTF-16 を `Rune` ではなく、規定の走査で。再帰を使わない) | 07 の 2〜3 章 | 07 章 3 章の受理・拒否の例が通る。約 30 万引数の入力でスタックが溢れずに分割できる | 単体テスト。AC-076〜098(分割)、AC-409、AC-410 |
| 5 | `Conductor.Flow`(純粋な判断): 分類、遷移と次の動き、起動前の判断、再開位置、リトライの判断と待ち時間(`ulong` の飽和、`UInt128`)、記録の関数 `Recorder`、`PipelineState` の手書きの `Equals` | 05 | 05 章の判定表と例の表が、すべて表駆動テストで通る。入出力・`Process`・`DateTime` を参照していない。`checked` を有効にしたテストの構成でも例外にならない | 単体テスト(05 章 14 章の「固定する項目」)。AC-268、AC-501、AC-502 |
| 6 | `Conductor.Report`: メッセージカタログ(英語・日本語)、ロケールの遅延判定、エスケープと切り詰め(コードポイント単位)、`watch` の表、色、使い方の全文、`Reporter` | 08 | 08 章の表の全文が、単体テストで文字単位で一致する。ロケールは最初に必要になったとき 1 回だけ判定する。孤立サロゲートで例外が出ない | AC-350〜388(出力とロケール、エスケープ)、AC-170〜187 の表の部分、AC-519・520・521 |
| 7 | `Conductor.State`: ステートの JSON(自前の書き出し、固定のキー順)、原子的な書き込み(`FileMode.CreateNew`、`Flush(true)`、`File.Move(overwrite: true)`、`Conductor.Native.UnixNative.SyncDirectory`)、読み込みの 14 段階の検査、整合性の検査、`reset`、`reset --all` | 06、07 の 7.2.1 節 | 06 章 6.6 節の単体テストの入力が、すべて期待どおり。読み込みは自前の `JsonReader`(BOM の前処理と、厳格な UTF-8 の復号を含む)で行い、`System.Text.Json` を使わない | 単体テスト(一時ディレクトリの実ファイル)。AC-302〜324(ステートの異常系)、AC-188〜205(`reset`)、AC-147、AC-407、AC-473・474、AC-546 |
| 8 | `Conductor.Cli`: `Deps`・`CliEnvironment`(02 章 4 章)、前処理、`run`・`pipeline` の流れ、リトライの実行ループ、`watch`・`reset`・`validate`・`convert` の配線、終了コードの決定。プロセスとシグナルは偽の部品で確かめる | 02 の 4・5 章、05 の 5〜11 章、04 の 7 章、10 の 2・8 章 | 偽の部品の上で、`run`・`pipeline` の全分岐(スキップ、拒否、再開、リトライ、中断、ステートの書き込みの失敗)が通る。ステートの書き込みの失敗は、ジョブの終了コードより優先して 65 | AC-117〜187(`run`・`pipeline`・`watch`)、AC-239〜279(run ID、リトライ)、AC-293〜300(注入)、AC-408、AC-485 |
| 9 | `Conductor.Runner` と `Conductor.Native`: `JobRunner`(`Process`、`WaitForExit(int)`、転送の 1 回、`ExitCode` の読み取り)、`Interrupt`、`RetryWaiter`、`SignalWatcher`(`PosixSignalRegistration` の静的な保持)、プラットフォーム層(`PlatformDetector`、`UnixPlatform`、`WindowsPlatform`、`CmdLauncher`)、`UnixNative`(`kill` と `DllImportResolver`) | 07 | 実プロセスのテスト(UNIX 系)と、偽の子(`Func<Process, int, bool>`)の単体テストが通る。Windows の `.cmd` の組み立ては、どの OS でも単体で通る。待機は、合図から 1 秒以内に打ち切れる。実行ファイルを自前で絶対パスに解決する | AC-099〜116、AC-280〜292、AC-403、AC-484、AC-486、AC-499、AC-503〜506、AC-509、AC-510・513、AC-517、AC-532〜535、AC-537〜539、AC-550。AC-301(Windows の Ctrl+C)は実機で手動 |
| 10 | `Conductor.Convert`: JSON ⇔ YAML の変換、JSON の自前の書き出し、YAML の自前の書き出し、`double` の最短の書式、表せない値の検出 | 09、03 の 4〜7 章(読み取りを共有) | 09 章 11 節の例が通る。往復で木が等しい。整数は `long` のまま通り、`double` に化けない。YAML の plain の条件が守られる | AC-206〜238、AC-413、AC-418、AC-469〜472、AC-536、AC-544 |
| 11 | 受け入れテストの全件、CI(GitHub Actions)、リリース(RID ごとの発行物・`SHA256SUMS`)、デモ、README | 11 | 11 章 4 章の「自動」が 3 OS で通る。品質ゲート(5 章)がすべて通る。リリースの手順(6 章)が、ドラフトのリリースまで通る | AC-397〜446、AC-476〜550 のうち残り(AC-488・489 の起動時間と YAML ライブラリの遅延読み込み、AC-526〜530 の発行物) |

- 段階 7(`State`)と段階 9(`Runner`)は、互いに依存しない(どちらも `Flow` の型と `Native` を使う)ので、別の人が並行して作れる。`UnixNative` は、先に最小の形(`Kill` と `SyncDirectory`)で作っておく。
- 段階 8 では、プロセスとシグナルを偽の部品(`ScriptProcess`、`FakeSignals`)にして、流れだけを確かめる。実プロセスとシグナルは、段階 9 で確かめる。
- 段階 3 の `JsonReader` は、段階 7 の読み込みと、段階 10 の変換が共用する(`Conductor.Json`。03 章 14.2 節、06 章 6.5 節)。段階 3 を先に終える。
- 各段階の終わりに、`dotnet format --verify-no-changes`、`dotnet build -warnaserror`、`dotnet test` を通す。依存の向きの検査(AC-514)は、段階 0 のうちに CI に入れておくと、後から崩れない。
- 段階 1 の入口と出力の完了は、早めに作る。標準出力の失敗と flush の扱い(落とし穴 2.1 の 1〜3)は、後から直しにくい。

## 2. .NET 特有の落とし穴

02〜11 章の「注意」「未確認」から集めた。各項目の末尾に、詳細の章を示す。

### 2.1 標準出力・終了・内部の失敗

| # | 落とし穴 | 対処 | 章 |
|---|----------|------|----|
| 1 | **UNIX では、読み手が閉じたパイプへの書き込みが、標準のストリーム(`Console.OpenStandardOutput()`)で `EPIPE` を捨てられ、`IOException` にならない**(ランタイムのソースで確認)。`watch`・`convert` が 65・67 でなく 0 で終わりうる | この版の既知の差として受け入れる。容量不足(`/dev/full`)などの `EPIPE` 以外の失敗は例外になり、65・67 にする。Windows は未確認(`IOException` を想定) | 08 の 1.3.1、10 の 2、AC-493・495・516 |
| 2 | ランタイムは起動時に `SIGPIPE` を無視している(閉じたパイプでプロセスがシグナルで死なない)。`PosixSignalRegistration` で `SIGPIPE` を扱うと、この前提が崩れる | `SIGPIPE` の設定を変えない | 08 の 1.3.2 |
| 3 | `Console.Out`・`Console.WriteLine` は、符号化が環境で決まり、改行が `Environment.NewLine`(Windows で CRLF)になりうる。`StreamWriter` はファイナライザでバッファを流さない(未確認) | `Console.OpenStandardOutput()` から BOM なし UTF-8 の `StreamWriter`(`AutoFlush`、`NewLine = "\n"`)を作り、1 回の表示につき 1 回の書き込みと `Flush()`。`Console.Out` を使わない(禁止の API) | 08 の 1.2・1.3、AC-520 |
| 4 | **`Environment.Exit` は `finally` を実行せず、ほかのスレッドが動いていても直ちに終了する**。`Main` の `return` は `finally` が実行される | 通常は `Main` が `int` を返す。`Environment.Exit` は `InternalFailure.ReportAndExit()` だけ | 10 の 4.1・6、AC-549 |
| 5 | `AppDomain.UnhandledException` のハンドラが戻ると、ランタイムの既定の終了処理になり、終了コードは 70 にならない。ハンドラの中から呼んだ `Environment.Exit(70)` が 70 になるかは未確認 | ハンドラで 1 行を書き `Environment.Exit(70)`。実機で確かめる(AC-490) | 10 の 6.1 |
| 6 | **`StackOverflowException` は捕まえられず、プロセスが終了する**。`OutOfMemoryException` は捕まえて続行しない | 入力の入れ子を再帰で処理せず、明示的なスタックで反復する(JSON・YAML・変換・`command` の分割)。深さ 32 の上限 | 10 の 6.3、02 の 6、AC-412 |
| 7 | `catch (Exception)` や型なしの `catch` は、想定外の失敗(実装の不具合)を規定の誤りに化けさせる。`Debug.Assert` は Release で消える | 想定する型だけを狭く捕まえる(`IOException`、`UnauthorizedAccessException`、`Win32Exception`、`DecoderFallbackException` など)。`catch (Exception)` は入口の 1 か所。入力の検査に `Debug.Assert` を使わない | 02 の 9、10 の 8.1 |
| 8 | `TaskScheduler.UnobservedTaskException` は、ファイナライザの実行時に呼ばれ、非決定的 | `Task` を作らない。ハンドラは `SetObserved()` のあと `ReportAndExit()`。受け入れテストの対象は限る | 10 の 6.1 の 4、AC-491 |
| 9 | 例外の `Message`・`ToString()` に、入力の値や OS の文が混ざる | 利用者への表示に使わない。固定の文だけ | 02 の 9、10 の 8.1 |
| 10 | `Main` の戻り値が 0〜255 の外だと、OS で切り詰めが違う(UNIX 系は下位 8 ビット、Windows は 32 ビット) | `ExitCodes.For` で 0〜255 にそろえてから返す。`Environment.ExitCode` を使わない | 10 の 4.1 |

### 2.2 整数・型・数値

| # | 落とし穴 | 対処 | 章 |
|---|----------|------|----|
| 1 | `int.Parse`・`long.Parse` は、既定で符号・前後の空白を受け、例外を投げる(全角数字は未確認)。巨大な字句で `OverflowException` | 字句を自前で確かめ、桁数(符号を除いて 19 桁超は範囲外)で先に打ち切り、`long.TryParse(NumberStyles.None または Integer, InvariantCulture)` を使う。`BigInteger`・`decimal` を使わない | 03 の 4.2・4.3、06 の 6.5 |
| 2 | `double.Parse` は文化に依存し、`1e999` や指数が巨大な字句の結果が未確認 | 字句を正規化してから `InvariantCulture` で呼ぶ。`NaN`・`Infinity` は拒否。T4 で確かめる | 03 の 4.2、09 の 6 |
| 3 | `double.ToString()` は文化に依存し(`InvariantCulture` を指定する)、.NET Core 3.0 以降は往復できる最短の文字列を返す(確認)が、指数形への切り替えの閾値などは未確認 | 桁と指数を取り出して、自前で並べ直す(形に依存しない)。`-0` を区別する | 09 の 6.1、AC-234 |
| 4 | **`record` の自動の `Equals` は、`IReadOnlyList<T>`(配列・リスト)を参照で比べる**。`PipelineState`・`Definition` を `Equals` で比べると、中身が同じでも偽 | `PipelineState` は手書きの `Equals`(列は要素ごと、文字列は序数)。テストは `SequenceEqual` | 05 の 1.1、03 の 14.1 |
| 5 | `checked` の文脈や `<CheckForOverflowUnderflow>` で、飽和すべき演算が例外になる。`(int)` のキャストで `-1`(無期限)に化ける | 飽和を明示する(`Math.BigMul`・`UInt128`・`Saturating`)。プロジェクトで `CheckForOverflowUnderflow` を設定しない。待機に渡すミリ秒は `ToWaitMs` で `int` の範囲にする | 05 の 7.6・7.8、AC-503・541 |
| 6 | `TimeSpan.FromSeconds(double)`・`TimeSpan` を秒の `ulong` から作ると、丸めと範囲外の例外 | `TimeSpan` は 250 以下のミリ秒の `int` からだけ作る。残り時間は整数で求める | 05 の 7.8、07 の 7.5 |
| 7 | `Dictionary` の列挙順に頼ると、順序が変わりうる(キー順は入力の記述順が規定) | 木のマップは `MapEntry` の並び。`List<T>.Sort` は不安定なので、`OrderBy` + `ThenBy`(安定) | 03 の 6.1・10、AC-511 |
| 8 | `bool` を整数として受理しない、`1.0`・`1e0` は整数でない | 木の型(`BoolNode`・`IntNode`・`FloatNode`)で区別する。字句で判定する | 03 の 4・5 |

### 2.3 文字列・正規表現

| # | 落とし穴 | 対処 | 章 |
|---|----------|------|----|
| 1 | **文化依存の API**(`ToString()`、`Parse`、`ToUpper`、`ToLower`、`string.Compare`、`StartsWith(string)`、`EndsWith(string)`、`IndexOf(string)`)の既定は、文化に依存する | 常に `StringComparison.Ordinal`(または `OrdinalIgnoreCase`)と `CultureInfo.InvariantCulture` を指定する。`InvariantGlobalization=true`。CA1304・CA1305・CA1307・CA1309・CA1310 をエラーにし、`BannedSymbols.txt` で二重に守る(AC-518) | 02 の 1、03、AC-530 |
| 2 | **`string.Trim()`・`char.IsWhiteSpace`・`string.IsNullOrWhiteSpace`** の集合が、Unicode の White_Space と一致するかはデータの版に依る(公式の列挙は一致するが、未確認の余地) | 25 文字を明示した自前の表で判定する(`Trim()` を使わない)。行末の空白を削るのは `TrimEnd(' ')` と引数を明示する | 03 の 1、08 の 6.2、AC-523 |
| 3 | **`Regex` の `$` は、末尾の改行の直前にも一致する**。**`\d` は Unicode の 10 進数字(全角など)に一致する**。`RegexOptions.ECMAScript` や `\z` の挙動 | 正規表現を避け、1 文字ずつの比較で書く(`'0' <= c && c <= '9'`)。使うなら `\A…\z` と `[0-9]` | 03 の 5.2・9、04 の 5.2、05 の 6.1 |
| 4 | **`Path.GetExtension(".yaml")` などの拡張子の扱い**が、区切り文字や先頭の `.` で変わる | 拡張子の判定は自前(最後の区切りより後ろの、最後の `.` から末尾。ASCII の大文字小文字を区別しない)。`.cmd` の判定も末尾の序数比較 | 03 の 2.1、07 の 4.8、AC-545 |
| 5 | `string` は UTF-16。`Length` はコードユニット数で、`😀` は 2。`Rune` は孤立サロゲートを表せない。書記素は別 | 文字数・切り詰め・表示幅は、サロゲートの対を 1 と数える自前の走査(コードポイント)。run ID の長さは UTF-8 のバイト数(`Encoding.UTF8.GetByteCount`)。`foreach (char c in s)` でサロゲートの対を分断しない | 04 の 4.0、08 の 11 |
| 6 | 不正な UTF-8 の引数・環境変数・ファイル名が `string` にどう入るか(U+FFFD か孤立サロゲートか)は未確認 | `IsSuspectText`(孤立サロゲートまたは U+FFFD を含む)で近似して区別し、表示の前にエスケープする。木に孤立サロゲートを入れない | 04 の 4.4.2、08 の 9、AC-519 |
| 7 | `File.WriteAllText(path, text)`・`Encoding.UTF8` は、BOM の有無が環境で変わりうる(未確認)。`File.ReadAllText` は BOM を消し、CR だけの改行などを変える | バイトで読み(上限 + 1 バイトまで)、BOM をバイトで取り除き、`new UTF8Encoding(false, true)` で厳格に復号する。書き出しも同じエンコーディングで `byte[]` を書く | 03 の 3、06 の 6.1.2、08 の 11 |
| 8 | 複合書式(`string.Format`)に入力の値を渡すと、書式として解釈される | 固定の文字列に値を連結する(`string.Concat`、`InvariantCulture`)。`AppendLine`・`Environment.NewLine` を使わず `Append('\n')` | 08 の 10 |
| 9 | `Utf8JsonWriter` の既定は、改行が `Environment.NewLine`、`<`・`>`・`&`・`'`・`+` と非 ASCII をエスケープし、`UnsafeRelaxedJsonEscaping` のブロックリストは版で変わる | 書き出しは自前(`Utf8JsonWriter` を使わない)。U+2028・U+2029 のエスケープ、LF 固定、インデント 2 | 06 の 4、09 の 7、AC-469 |

### 2.4 JSON・YAML・ファイル

| # | 落とし穴 | 対処 | 章 |
|---|----------|------|----|
| 1 | **`Utf8JsonReader`/`JsonDocument` の限界**: 位置情報、重複キー、数値の字句(整数・小数・指数の区別、巨大整数)、`MaxDepth`、コメント・末尾カンマ、BOM、不正な UTF-8・サロゲートの扱いが、仕様の規則表と一致するかが未確認 | 読み取りは全面自前(`JsonReader`)。`Utf8JsonReader` は差分試験(T5)だけに使う。`JsonSerializer`・`JsonDocument.Parse` を製品で使わない(AC-421) | 03 の 4、06 の 6.5 |
| 2 | **YamlDotNet の高水準 API**(`Deserializer`、`DeserializerBuilder`)は、型を指定した復元で暗黙の型判定を行う。低水準の `Parser`/`Scanner` でも、タブ、単純キーの 1,024 文字、アンカー名、CR だけの改行などで YAML 1.2 より狭く拒否し、位置(`Mark.Index`)の数え方が違う | 低水準の `Parser`/`Scanner` のイベントだけを使い、型・重複キー・アンカーの展開・上限は自前。位置は `Mark.Index` から自前で数え、列は UTF-16 のコードユニット。ライブラリに渡す前に、CR だけの改行、サロゲート、制御文字を検出する | 03 の 7 |
| 3 | YamlDotNet は、深いフロー形式の入れ子で再帰・極端な遅さの恐れ(未確認) | 開いているコレクションの深さで読み取りを打ち切る。敵対的な入力で時間を測る(T1) | 03 の 6.4・7.4 |
| 4 | **`File.Move(overwrite: true)` の原子性**と、Windows で他のプロセスが開いているときの失敗(`UnauthorizedAccessException` か `IOException`)は未確認。`File.Replace` は置き換え先が無いと失敗する | 一時ファイルを `Dispose` してから `File.Move`。失敗は `WriteFailed`。一時ファイルは同じディレクトリに作る | 06 の 5・5.1 |
| 5 | **`FileStream.Flush()`(引数なし)は OS の同期までは保証しない**。`Flush(true)` が `fsync`/`FlushFileBuffers` を呼ぶこと(未確認)。**ディレクトリの `fsync` の管理 API は無い** | `Flush(flushToDisk: true)` を `Dispose` の前に呼ぶ。ディレクトリの同期は UNIX 系だけ、`UnixNative.SyncDirectory`(`open(O_RDONLY)`・`fsync`・`close` の P/Invoke。失敗は無視) | 06 の 5 |
| 6 | `FileMode.CreateNew` の `IOException` と他の `IOException` の見分け、`UnixCreateMode` の既定値と umask(`tempfile` に相当する 0600 にならないこと) | `UnixCreateMode` に `0666` を指定して OS 標準の権限にし、AC で固定する | 06 の 5 |
| 7 | 名前つきパイプ(FIFO)を `FileStream` で開くと、書き手が現れるまでブロックしうる。`File.ReadAllBytes` は上限を超える巨大なファイルを全部読む | 通常のファイルかを確かめ、上限 + 1 バイトまで読む。ブロックしない開き方を T7 で確かめる | 09 の 3、03 の 3、AC-536 |
| 8 | ディレクトリを指すパスを `File.Delete` すると、OS で例外の型が違う(`UnauthorizedAccessException` など)。`File.Delete` は存在しないファイルで例外を投げない | 「存在しない」以外の失敗は `removeFailed`。ディレクトリのステートのパスは消さない | 06 の 8.1 |
| 9 | `Directory.EnumerateFileSystemEntries` の途中でファイルが消えると例外。`searchPattern` は `*`・`?` を含むファイル名と食い違う | 列挙全体を `try/catch`(特定の例外型)。名前は `Path.GetFileName(entry).StartsWith(prefix, Ordinal)` で選ぶ。シンボリックリンクは `LinkTarget` で見分ける | 06 の 5・8 |

### 2.5 プロセス・シグナル

| # | 落とし穴 | 対処 | 章 |
|---|----------|------|----|
| 1 | **`Process.ExitCode` は、UNIX 系でシグナルで終了した子に `128 + n` を返す**(ランタイムのソースで確認。公式の API 文書に記述なし。.NET 10 のリリース版は未確認)。**`exit(137)` と区別できない**。公開 API で終了したシグナルは取れない。`waitpid` を自分で呼ぶとランタイムの回収と競合する | `ExitCode` が 129〜159 で、そのジョブの成功・警告のコードに含まれないときだけ、シグナル終了(`ExitCode = null`、`TerminatedBySignal = n`)とみなす。自分で `exit(128+n)` したジョブは、リトライの対象外になる(他の実装との差)。Windows は `ExitCode` を符号つき 32 ビットのまま保つ | 07 の 4.4、AC-140・504 |
| 2 | `Process.ExitCode` は、終了前に読むと `InvalidOperationException`。`WaitForExit()`(引数なし)は、リダイレクトした出力の EOF を待つ場合があり、孫プロセスが持つパイプで長く待つ | `WaitForExit(int)` を、10 ms 以下の刻みで繰り返す。標準入出力は `Redirect*=false`(引き継ぐ)。`WaitForExitAsync`・`Exited` を使わない | 07 の 5、AC-403・506 |
| 3 | **.NET の `ProcessStartInfo` では、UNIX 系でも子を新しいプロセスグループ・セッションに置けない**(`CreateNewProcessGroup` は Windows 専用、`StartDetached` は .NET 11 だけ。`fork` と `exec` の間に `setpgid` を差し込む手段がない) | 置かない(決定)。端末の Ctrl+C はジョブにも直接届き、転送でもう 1 回届く(ジョブは SIGINT を 2 回受けうる)。`/dev/tty` を開けるジョブは動く | 07 の 4.7、AC-288・535 |
| 3b | `Process.Kill()` は強制終了(SIGKILL)。任意のシグナルを子に送る管理 API が無い | `libc` の `kill` を `UnixNative` の P/Invoke(`LibraryImport`)で呼ぶ。許可集合(1・2・3・15)だけを 1 回送る。`Process.Kill` を使わない。`kill` を呼ぶ前に、終了済みの子へ送らない | 07 の 7.2.1、AC-289 |
| 4 | **`Process.Start` は、`UseShellExecute=false` で `FileName` が絶対パスでないとき、`conductor` 自身のディレクトリ、カレントディレクトリ、`PATH` の順に探す**(ソースで確認)。区切りのない名前でも、同名のファイルが先に起動される | UNIX 系では、`Process.Start` の前に、`FileName` を絶対パスに自前で解決する(`/` を含めば `Path.GetFullPath`、含まなければ `PATH` を探索し、実行ビットを確かめる。`PATH` 未設定は起動失敗)。Windows は OS の標準(`CreateProcess`)に任せる | 07 の 4.9、AC-550 |
| 5 | `Process.Start` の例外(`Win32Exception`、`InvalidOperationException`、`ArgumentException`、NUL を含む文字列、孤立サロゲートなど)が環境で違う。`NativeErrorCode` は OS で違う | 列挙した型だけを狭く捕まえて、起動失敗(127)にまとめる。メッセージ・`NativeErrorCode` を表示しない。`OutOfMemoryException` は捕まえない | 07 の 4.3、10 の 3、AC-548 |
| 6 | **`PosixSignalRegistration` はファイナライザでも登録を解除する**。保持しないと GC で解除される | 静的なフィールドに保持し、プロセスの終了まで `Dispose` しない(本番)。テストでは `Stop` で `Dispose` する | 07 の 7.1・7.6、AC-499・537 |
| 7 | **シグナルのコールバックは別のスレッドで動く**(SIGINT・SIGQUIT・SIGTERM は専用の背景スレッド、SIGHUP はスレッドプール。ソースで確認)。複数のシグナルの順序は保証されない。`PosixSignalContext.Cancel` の意味は未確認 | コールバックは `Cancel = true` と `Interrupt.Trigger`(`Interlocked.CompareExchange` と `ManualResetEventSlim.Set`)だけ。入出力・`lock`・`throw` を書かない。待つ側は `Volatile.Read` で読む | 07 の 6・7・9、AC-538・539 |
| 8 | `Thread.Sleep` は合図で起こせない。`ManualResetEventSlim.Wait(TimeSpan)` に秒から作った大きな `TimeSpan` を渡すと `ArgumentOutOfRangeException` | 待機は `WaitRaised` を 250 ms 以下の刻みで繰り返し、単調な時計(`Stopwatch.GetTimestamp`)で期限を確かめる。`DateTime.UtcNow` を使わない | 05 の 7.8、07 の 7.5 |
| 9 | Windows は Ctrl+C だけ受ける(`Console.CancelKeyPress` の `ControlC`)。`ControlBreak` は何もしない。`PosixSignalRegistration` を Windows で使わない。Ctrl+C は同じコンソールのジョブにも OS が配るので、転送しない | `e.Cancel = true` で `conductor` の終了を打ち消し、`Trigger(2)`。ジョブの終了を待つ | 07 の 7.3 |
| 10 | Windows の `.cmd`/`.bat` は `cmd.exe` が再解釈し、`ArgumentList` の引数の中のコマンドが実行されうる(`BatBadBut` 相当。.NET の対策は未確認) | 禁止文字(`"`、`%`、`^`、`&`、`\|`、`<`、`>`、`!`、`(`、`)`、改行、復帰、NUL)を含む引数は起動しない(127)。`cmd.exe /d /s /c "…"` を `Arguments` 文字列で直接起動する(`UseShellExecute=false` のまま) | 07 の 4.8、AC-113〜115・486・510・513 |
| 11 | 親が `SIGCHLD` を `SIG_IGN` で起動されたとき、子の回収と `ExitCode` が正しいか未確認 | AC-534 で実機で確かめる | 07 の 13、AC-534 |
| 12 | **`libc` の名前解決**: `"libc"` が Linux で `libc.so.6`、macOS で `libSystem` に解決されるか未確認。`DllImportResolver` は 1 アセンブリに 1 つだけ | `UnixNative` の静的コンストラクターで `NativeLibrary.SetDllImportResolver` を設定する(`libc.so.6`、`/usr/lib/libSystem.B.dylib`)。`DllNotFoundException`・`EntryPointNotFoundException` は握りつぶすが、転送の受け入れテストで 3 OS を確かめる。`AllowUnsafeBlocks` が要る | 07 の 7.2.1 |
| 13 | OS の判定が散らばる(`OperatingSystem.IsWindows()` など) | `PlatformDetector.Detect()` だけ。他は `PlatformKind` を引数で受ける。点検(AC-420) | 07 の 8.2、02 の 3 |

### 2.6 テスト・CI・配布

| # | 落とし穴 | 対処 | 章 |
|---|----------|------|----|
| 1 | GitHub Actions の `strategy.matrix` の中では `env` コンテキストを使えない(記憶による。未確認) | 最小と最新を別のジョブにする | 11 の 5 |
| 2 | `xUnit` は、テストクラスごとに並列実行する。環境変数や作業ディレクトリの変更が競合する | 受け入れテストは子プロセスに環境を渡す(`Environment.SetEnvironmentVariable` を使わない)。並列の設定は 11 章 3.9 節 | 11 の 3.1・3.9 |
| 3 | `coverlet.collector` はしきい値を持たず、MTP v2 とは併用できない | `Conductor.Checks coverage` が Cobertura の行の網羅率を検査する(80%) | 11 の 5.3 |
| 4 | `ロックファイル`(`--locked-mode`)と `rollForward = latestFeature` の相性、`RuntimeIdentifiers` との相性が未確認 | 失敗したら `rollForward` を `disable` にして SDK を固定する | 02 の 7.1、11 の 8.4 |
| 5 | 単一ファイルの発行物が、実行ファイル 1 つだけにならない(ランタイムのネイティブのファイルが残る)場合 | AC-526 で検出し、`IncludeNativeLibrariesForSelfExtract` を有効にする(`HOME` 未設定の環境では `DOTNET_BUNDLE_EXTRACT_BASE_DIR` が要る) | 11 の 6.2 |
| 6 | macOS arm64 の実行ファイルの署名(ad-hoc 署名が自動で付くか未確認)。署名・公証はしない | macOS のランナーで発行して、実機で動くことを確かめる | 11 の 6.2・6.5 |
| 7 | テストでスクリプトを書いてすぐ起動すると、Linux で `ETXTBSY` が出ることがある | スクリプトは、あらかじめ置く。`TJ` はアプリホストを直接起動する | 07 の 11、11 の 3.4 |
| 8 | 起動時間は、ランタイムの起動と JIT で、ネイティブより遅い。ReadyToRun・トリミング・AOT は採らない | 起動時間は目安として測る(AC-488)。YamlDotNet の読み込みは遅延し、`--version`・`--help` で読まない(AC-489) | 11 の 2.1 |

## 3. 未確認事項の一覧

各章の「未確認」を、分類してまとめた。いずれも、断定していない。実装時に、採用する版と OS で確かめる。

### 3.1 .NET の版と、ライブラリの版

| 項目 | 内容 | 章 |
|------|------|----|
| サポートする版 | .NET 10 のみ(作成時点)。次の LTS の登場、サポートの状況、`global.json` の SDK の版(`10.0.100` は書式の例) | 02 の 1・7、11 の 2.5 |
| ソースだけで確かめた挙動 | `dotnet/runtime` の `main` ブランチのソースの読み取り(`Process.ExitCode` と `128 + n`、実行ファイルの探索順序、`PosixSignalRegistration` のスレッド、`SIGPIPE` の無視、`EPIPE` を捨てること)が、.NET 10 のリリース版と同じか | 07 の 13、08 の 1.3.1 |
| YamlDotNet | 18.1.0 を公開ソースの読み取りで確認した範囲(動かしていない)。03 章 7.5 節の 12 項目の試験、トリミング・AOT での動作。`SharpYaml`・`Meziantou.Framework.Yaml` は未評価 | 03 の 7・18、11 の 8.2 |
| アナライザー | 規則の名前(CA1305 など)、`AnalysisMode = Recommended` の規則の組、`BannedApiAnalyzers` の検出範囲、`InternalsVisibleTo` 項目、`IncludeSourceRevisionInInformationalVersion = false` で `+<コミット>` が付かないこと | 02 の 11、11 の 10 |
| 版の導出 | `AssemblyVersion`・`FileVersion` が `<Version>` から導かれる規則(使わない) | 02 の 11 |

### 3.2 Windows

| 項目 | 内容 | 章 |
|------|------|----|
| `Process.ExitCode` | 負の値(`-1`)・`256`・符号なし 32 ビットの扱い(AC-112) | 05、07 の 4.4、11 の 10 |
| Ctrl+C | `Console.CancelKeyPress` の `ControlC` で届くこと、同じコンソールのジョブにも届くこと、`ControlBreak` で `Cancel = false` のときの終了。手動の確認の記録(AC-301)。Ctrl+Break は対象外(AC-533 は任意) | 07 の 7.3・13、11 の 10 |
| `.cmd` / `.bat` の起動 | `cmd.exe /d /s /c` を `Arguments` 文字列で起動する形が動くこと、`Arguments` が再び引用されないこと、「バッチ ジョブを終了しますか」の問い合わせの影響、`BatBadBut` 対策の版差(AC-113〜115・486・510・513)。プログラム探索の詳細 | 07 の 4.8・13 |
| ステートの書き込み | `File.Move(overwrite: true)` の原子性と、他のプロセスが開いているときの失敗の型(AC-322・546)。ディレクトリの同期はしない | 06 の 5.1・13.4 |
| 標準出力 | 閉じたパイプで `IOException` になること(AC-493・516)、`Console.OutputEncoding` を UTF-8 にしたときのコンソールとリダイレクト先の動作、改行(CRLF への変換が起きないこと) | 08 の 1.2・13 |
| 引数・環境 | 孤立サロゲート、`args` と `GetCommandLineArgs()` の規則、環境変数名の大文字小文字、`Console.IsOutputRedirected` の実装 | 04 の 11、08 の 13 |
| ほか | 端末の判定、`Stopwatch` の分解能、タイマーの分解能 | 07 の 13 |

### 3.3 Linux・macOS(UNIX 系)

| 項目 | 内容 | 章 |
|------|------|----|
| `libc` の P/Invoke | 名前の組(`libc.so.6`、`/usr/lib/libSystem.B.dylib`)の解決、`pid_t` が 32 ビット、単一ファイルの発行での動作。`open` を `O_RDONLY` でディレクトリに使えること、ARM64 での可変長引数の呼び出し規約 | 07 の 7.2.1、06 の 5 |
| 新しいセッションに置かないこと | 同じプロセスグループのジョブが、端末で SIGINT を 2 回受けること。標準入力が端末のジョブが停止しないこと(AC-182・183・535) | 07 の 4.7・13 |
| `SIGCHLD` | `SIG_IGN` で起動されたときの子の回収(AC-534)。ランタイムの内部処理 | 07 の 13 |
| `SIGPIPE` | 子が `conductor` の `SIGPIPE` の扱いを引き継ぐか | 07 の 13 |
| ファイル | ディレクトリへの `File.Delete` の例外の型、FIFO のブロック(AC-536)、`/dev/full`(AC-495)、`/proc/<pid>/stat`(AC-403)、不正な UTF-8 のファイル名(AC-349)、`FileInfo.LinkTarget` | 06 の 13.4、04 の 11、11 の 10 |
| 引数・環境 | 不正な UTF-8 のバイトが `string[] args`・`GetEnvironmentVariable` にどう入るか(置換か、孤立サロゲートか) | 04 の 4.4.2・11、03 の 18 |
| 未登録のシグナル | `watch`・`reset`・`validate`・`convert` の実行中の SIGINT の既定の動作(AC-532)、SIGQUIT の既定の動作 | 10 の 5・10 |

### 3.4 プロセス・シグナルの細部(OS を問わない)

| 項目 | 内容 | 章 |
|------|------|----|
| 同時のシグナル | ほぼ同時に届いた複数のシグナルの処理順序 | 07 の 13 |
| `PosixSignalContext.Cancel` | `true` にしたときの意味、同じシグナルへの複数の登録 | 11 の 10 |
| `Stopwatch` | 休止中にティックが進むか、`Frequency` | 05 の 7.8 |
| `ManualResetEventSlim` | `Set` と `Wait` の同時の呼び出しの安全性(クラス全体がスレッドセーフという記述の確認) | 05 の 7.8 |
| `UnhandledException` | ハンドラの中の `Environment.Exit(70)` が 70 で終わるか、ハンドラが戻ったときの既定の終了コード。`StackOverflowException` の終了コード。`RuntimeWrappedException` | 10 の 6・10 |
| メモリ不足 | 実際のメモリ不足での `OutOfMemoryException` が捕まえられるか | 10 の 6.3 |
| `Process.Start` の例外 | `Win32Exception` 以外の例外の型の全件(AC-548) | 07 の 4.3、10 の 10 |

### 3.5 JSON・YAML・文字列の細部

| 項目 | 内容 | 章 |
|------|------|----|
| `Utf8JsonReader` | 差分試験の対象(位置、重複キー、数値の字句、`MaxDepth`、BOM、不正な UTF-8、サロゲート) | 03 の 4.4・18 |
| 数値 | `long.TryParse` が `-9223372036854775808` を受理すること、`double.Parse` の桁あふれと 9 桁の指数、`double.ToString("R")` の書式、`double.IsNegative` | 03 の 4.2・18、09 の 6.1・12 |
| 文字クラス | `char.IsWhiteSpace` と規定の 25 文字との一致(参考)、`string.EnumerateRunes()` が孤立サロゲートに返す値、`Path` の各 API の細部 | 03 の 1・18、04 の 11 |
| YAML の文法の誤りの行 | 採用するライブラリの検出位置による(表は期待する値)。複数行のスカラーの文字の大きさの数え方の境界(16 MiB 付近) | 03 の 13.4・18、09 の 12 |
| 変換 | マージで取り込まれる値の中の表せない値の場所(`a.<<.x`)は、この仕様の決定で、他の規定による裏づけがない。1,024 文字を超えるキーの拒否、非 BMP の文字・引用符つきのキーの長さの既知の制限 | 09 の 12 |
| 出力のエスケープ | `System.Text.Json` の書き出しを使わないので確認していない。JSON の出力で U+007F〜U+009F などを `\u` でエスケープするのは、この仕様の決定 | 09 の 12 |
| 大文字小文字を区別しないファイルシステム | run ID `A` と `a` のステートの衝突(AC-260)。仕様の記述による | 11 の 10 |

### 3.6 道具・CI・配布・性能

| 項目 | 内容 | 章 |
|------|------|----|
| 道具を動かしていない | `dotnet format`、`dotnet build -warnaserror`、`dotnet test` の `--framework`・`--collect`・`--settings`・`--blame-hang-timeout`、`xunit.runner.json` のキー名、`Directory.Build.props` の `Condition` の式、GitHub Actions の実行。版、設定のキー・書式、フラグ、終了コードの規則は、実装時に確認する | 11 の 10 |
| ロックファイル | `RuntimeIdentifiers` を指定したプロジェクトの `--locked-mode` が、3 OS で同じロックファイルを受け入れるか。`latestFeature` との併用。`actions/setup-dotnet` の `cache-dependency-path` | 02 の 7.1、11 の 8.4 |
| 単一ファイル | 実行ファイル 1 つだけになるか。`IncludeNativeLibrariesForSelfExtract` の要否。macOS arm64 の ad-hoc 署名 | 11 の 6.2 |
| CI | `ubuntu-24.04-arm`・`macos-latest` のラベル、GitHub Actions の式の関数、`actions/*` の最新の版、`dotnet list package --vulnerable` の終了コード、ブランチ保護 | 11 の 5・6 |
| テスト | 疑似端末の `openpty` の P/Invoke、`TJ` のアプリホストの起動(`DOTNET_ROOT`)、FIFO・`SIGCHLD` の自動テスト、`System.Random` の列の安定 | 11 の 3・4.20 |
| 性能 | 起動時間と各 p95 の基準(すべて仮)、網羅率の実測値、トリミング・ReadyToRun・NativeAOT・圧縮の有無での差(採らない前提) | 11 の 2.1・10 |
| 手順 | macOS の署名・公証と Windows のコード署名は行わない(方針)。利用者向けの回避手順(`xattr`、`codesign`)の成否 | 11 の 6.5 |

## 4. 最初に確かめる小さな試験

実装を始める前に、次の試験を、.NET 10 の SDK で、できれば 3 OS で行う。結果で、採否や設計を決める。試験は、リポジトリの外の作業用の場所で行い、結果を記録する。

| # | 試験 | 手順 | 期待 | 結果の使い道 |
|---|------|------|------|--------------|
| T1 | **YamlDotNet の採否** | 03 章 7.5 節の 12 項目(受理の構文、タグ、重複キー、深い入力、敵対的な入力が 30 秒以内、型の判定、文法の誤りの行、1,024 文字のキー、位置、サロゲート、追加で拒否する入力、版の間の一致)を、固定した版(18.1.0)の `Parser`/`Scanner` のイベントだけで、小さな試作の読み取りに通す | 全項目が、03 章の期待どおり | 通らない版は採用しない。版を替える、03 章 7.4 節の方法で補う、代案(`SharpYaml` など)を評価する |
| T2 | `Utf8JsonReader` との差分試験 | 03 章 4.4 節: 自前の `JsonReader` と `Utf8JsonReader`/`JsonDocument` に、同じ入力(重複キー、巨大整数、BOM、不正な UTF-8、孤立サロゲートのエスケープ、深い入れ子、`NaN`)を通して差を記録する | 自前の読み取りが、03 章の規則表のとおり | 差の記録(製品は自前の規則を正とする)。`MaxDepth` などの未確認を埋める |
| T3 | 標準出力のパイプが閉じたとき | 5 MB を標準出力に書く小さな実行ファイルで、読み手(`head -c 1`)が先に閉じる。`Console.OpenStandardOutput()` の `Write` と `Flush` が `IOException` を投げるか、終了コードがどうなるかを、Linux・macOS・Windows で記録する。`/dev/full` への出力も試す | UNIX 系は例外なし(0)、`/dev/full` は `IOException`。Windows は `IOException`(想定) | 08 章 1.3.1 の既知の差の確定。AC-493・495・516 の期待 |
| T4 | `double` の往復 | ランダムな 20 万個以上の `double` について、`ToString("R", InvariantCulture)`、`double.Parse` の往復、指数形への切り替えの閾値、`-0`、`1e999`・`1e-400`・指数が 9 桁の字句の `double.Parse` の結果と時間を記録する | 往復が一致。巨大な指数で例外・停止がない | 09 章 6.1 節の書式の確定。03 章 4.2 節の字句の正規化 |
| T5 | `Process.ExitCode` の値 | UNIX 系で、`sh -c 'kill -9 $$'`、`exit 137`、`exit 130` の子の `ExitCode` を読む。`exit 256` と `exit -1`(Windows)の値。`SIGCHLD` を `SIG_IGN` にした親での `ExitCode` | シグナル終了が `128 + n`。`exit 137` と同じ値 | 07 章 4.4 節の推定の確定。AC-140・504・534 |
| T6 | P/Invoke の名前解決 | `LibraryImport("libc")` の `kill`、`open`、`fsync`、`close` を、`DllImportResolver` あり・なしで、Linux(x64・arm64)・macOS(arm64)・単一ファイルの発行物で呼ぶ。ディレクトリを `O_RDONLY` で開いて `fsync` | 解決でき、`kill(pid, 0)` が成功する | 07 章 7.2.1 節の名前の組の確定。06 章 5 章のディレクトリの同期 |
| T7 | ファイル・FIFO | `FileMode.CreateNew` の `IOException`、`UnixCreateMode` に `0666` を指定した権限(umask 022 で `0644`)、`Flush(true)`、`File.Move(overwrite: true)`、Windows で他のプロセスが開いているときの失敗の型、FIFO を `FileStream` で開いたときのブロック(Linux)、`/dev/zero` から上限 + 1 バイトで止まる読み取り | 06 章 5 章・09 章 3 節のとおり。FIFO でブロックしない | AC-302〜324・536 の下書き |
| T8 | シグナルのコールバック | `PosixSignalRegistration.Create` の保持あり・なし(GC を強制)、コールバックのスレッド ID、同じシグナルの複数の登録、`Cancel` の効果、SIGINT・SIGQUIT・SIGTERM・SIGHUP の順序。Windows の `Console.CancelKeyPress`(`ControlC`・`ControlBreak`)と `Cancel` | 保持しないと解除される。コールバックはメインスレッドと別 | 07 章 7 章の確定。AC-537〜539 |
| T9 | 新しいセッションに置かないことの影響 | 疑似端末(`openpty`)の子側を標準入出力にした `conductor` が、標準入力を読むジョブと `/dev/tty` を開くジョブを起動する。端末の Ctrl+C で、ジョブが SIGINT を何回受けるか | 標準入力を読めて止まらない。`/dev/tty` を開ける | 07 章 4.7 節の確定。AC-182・183・535 |
| T10 | Windows の最初の確認 | `Process.ExitCode` が、Ctrl+C で終了した子に何を返すか。`.cmd` を `cmd.exe /d /s /c "…"` の `Arguments` 文字列で起動した結果(安全な引数、禁止文字)。`ArgumentList` の空の引数。`File.Move` が、他のプロセスの読み取り中に失敗するか。ディレクトリのオープン | 07 章 4.4・4.8・7.3 節、06 章の想定のとおり | Windows の分岐の確定。AC-112・301・322・486・510 |
| T11 | `UnhandledException` | 別のスレッドで未処理の例外を投げ、ハンドラの中で 1 行を書いて `Environment.Exit(70)` を呼ぶ。終了コードを別プロセスから読む(Linux・macOS・Windows)。`UnobservedTaskException` も | 終了コード 70 | 10 章 6.1 節の確定。AC-490・491 |
| T12 | 文化・文字列の基本 | `InvariantGlobalization=true` の下で、`double.ToString()`・`ToUpper`・`string.Compare`・`StartsWith(string)` の結果。`char.IsWhiteSpace` を全 `char` に通して 25 文字と比べる。`Path.GetExtension(".yaml")`。不正な UTF-8 の引数・環境変数が `string` にどう入るか。`Regex` の `$` と `\d` | 本章 2.3 の表のとおり | 落とし穴の前提の確認。04 章 4.4.2、AC-523・530 |
| T13 | ビルド・配布 | `dotnet publish -r <RID>`(単一ファイル・自己完結)で、発行ディレクトリが実行ファイル 1 つだけか、`HOME` 未設定で動くか、`--locked-mode` と `rollForward = latestFeature` の併用、`+<コミット>` が付かない版、起動時間 | 11 章 AC-526〜529 のとおり | 11 章 6.2 節の確定(`IncludeNativeLibrariesForSelfExtract` の要否) |
| T14 | 禁止の API の規則 | AC-518 の違反コード(`Console.WriteLine`、`json` の `JsonSerializer`、`Task.Run`、`Environment.Exit`、`ToUpper()` など)を置いて、アナライザー(CA1304 など)と `BannedSymbols.txt` が検出するか | ビルドがエラーになる | `.editorconfig`・`BannedSymbols.txt` の確定 |
| T15 | 型の骨格のコンパイル | 02・03・05・06・07・08・09・10 章の骨格のコードを 1 つのソリューションに連ね(本体は `throw new NotImplementedException()`)、`dotnet build -warnaserror` を通す。`internal` と `public` の可視性、`InternalsVisibleTo` | エラーなし | 型の名前と可視性の食い違いを、実装の前に見つける |

- T1・T2 は段階 0〜3 の前に、T3 は段階 1 の前に、T5・T6・T8・T9 は段階 9 の前に、T10 は Windows の CI を作る前に、T11 は段階 1 の前に行う。
- 試験の結果が、この文書や 02〜11 章の記述と違うときは、その章を直す(この仕様書に、確認した版と OS を書き添える)。

## 5. 迷ったときの指針

- 利用者から見える結果(受理・拒否、終了コード、出力の文字列、作られるファイルの内容)は、必ず 03〜10 章の字句に合わせる。内部の構造は、02 章の依存の向きと、`Deps` の境界を守れば、自由である。
- BCL の API が、規定と違う挙動をするときは、まず本章 2 章の表にあるかを見る。ない場合は、実際に動かして確かめ、確かめた版と OS を、該当の章に書き足す。
- 確かめられなかったときは、断定せず、「未確認」と書いて、3 章の表に加える。
