# 00. README(C# 版 仕様書の案内)

> この仕様書は v0.8.0 時点の挙動で凍結されている(v0.9.0 以降の環境変数の注入と `.env`、`graph`、`--dry-run`、`init` を含まない)。最新は [rust](../rust/00-README.md) と [language-agnostic](../language-agnostic/00-README.md)。

この仕様書は、コマンドラインツール `conductor` を、C#(.NET)でゼロから実装し直すための仕様である。`conductor` は、1 つの定義ファイルに書かれたジョブ(外部コマンド)の並びを、終了コードで分類しながら順に実行し、途中経過をステートファイルに残して、失敗したところから再開できるようにする、コマンドラインツールである。

この文書は、仕様書全体の入口である。範囲と版、目次、読む順序、規定の強さの語、用語集、この版が採った書式・実行環境の選択をまとめる。

## 1. 仕様書の範囲と版

| 項目 | 内容 |
|------|------|
| 対象 | `conductor` の利用者から見える挙動を、C#(.NET)で再実装するための仕様 |
| 版 | `conductor 0.8.0` と同じ利用者向けの挙動。`conductor --version` は `conductor 0.8.0` を出す。版の唯一の出所は `Directory.Build.props` の `<Version>` で、コードは、アセンブリの `AssemblyInformationalVersionAttribute` から実行時に読む(版の文字列のリテラルをコードに書かない。[02](02-architecture.md) 8 章、[11](11-quality-and-build.md) AC-476) |
| 実装言語・実行環境 | C#(`LangVersion` は `latest`)、.NET。**対象は .NET 10(`net10.0`)だけ**(実装時に確認)。作成時点(2026-11-20)で、サポート中の LTS は .NET 10 だけである(確認: .NET の公式のサポートポリシーのページ。.NET 10 は LTS で、サポート終了 2028-11-14。.NET 8 と .NET 9 は 2026-11-10 にサポートが終わった。.NET 11 は STS)。`Directory.Build.props` の `ConductorMinTfm` と `ConductorMaxTfm` は、どちらも `net10.0`。次の LTS が出たら、最小(1 つ前の LTS)と最新(最新の LTS)を分けて、テストを両方で通す。SDK は `global.json` で、`version` を `10.0.100` の形(完全な版。実装時に最新の安定版に合わせる)、`rollForward` を `latestFeature` にする |
| 言語機能の設定 | `Nullable=enable`、`ImplicitUsings=disable`(`using` を明示)、`TreatWarningsAsErrors=true`、アナライザー(`AnalysisLevel`、`EnforceCodeStyleInBuild`、`BannedApiAnalyzers`)。`InvariantGlobalization=true` |
| 外部ライブラリ | 実行時は YAML の読み取りの 1 つだけ(採用案は `YamlDotNet`。[03](03-definition-format.md) 7 章、[11](11-quality-and-build.md) 8 章)。ほかは BCL だけ(JSON の読み取りと書き出しは全面自前)。開発時の依存は、xUnit、`Microsoft.NET.Test.Sdk`、`coverlet.collector`、`Microsoft.CodeAnalysis.BannedApiAnalyzers`。版は実装時に確認して、`Directory.Packages.props` に正確な版で固定する |
| 対象 OS | Linux、macOS、Windows の 3 つ。配布物は、RID ごとの単一ファイル・自己完結の実行ファイル(`linux-x64`、`linux-arm64`、`osx-arm64`、`win-x64`) |
| 同じにするもの | サブコマンド、オプション、終了コード、実行の意味論(分類・分岐・再開・リトライ・run ID)、メッセージの文言(日本語・英語)、`watch` の表 |
| 含まないもの | 実装そのもの(コードは規定しない。骨格のコードは、型とシグネチャの例である)。各章が「実装に任せてよい」と書いた点 |

この仕様書だけで、同じ受理・拒否・出力・終了コードを返す実装を作れることを目標とする。

**確認の表記**: この仕様書を書いた環境には .NET SDK がなく、C# のコードを実際にビルドも実行もしていない。API と挙動は、公式ドキュメント(learn.microsoft.com、dotnet.microsoft.com、各ライブラリの公式リポジトリ)と、公開ソースの読み取りで確かめた範囲だけを「確認(出典: …)」と書き、確かめられなかった点は「未確認」と明記した。未確認の一覧は [12](12-rebuild-guide.md) 3 章にまとめた。この文書の版・サポートの記述は、各章の確認(02 章 1 章、11 章 1 章)に依る。

## 2. 目次

| ファイル | 内容(1 行要約) |
|----------|-----------------|
| [00-README.md](00-README.md) | この文書。範囲、目次、読む順序、規定の強さの語、用語集、書式・実行環境の選択 |
| [01-overview.md](01-overview.md) | 目的、ユースケース、設計方針、非目標、制約、利用者像、概念の概観 |
| [02-architecture.md](02-architecture.md) | ソリューションとプロジェクト、名前空間と依存の向き、型の骨格、差し替え口 `Deps`、実行のシーケンス、同期モデル、`Directory.Build.props`、版の出所、エラー処理の方針 |
| [03-definition-format.md](03-definition-format.md) | 定義ファイルの書式(JSON/YAML)、読み取りの段階、上限、YAML ライブラリの選定と補完、自前の JSON の読み取り、検査、エラーの種類(31 種)と位置の出し方 |
| [04-cli.md](04-cli.md) | コマンドライン(6 つのサブコマンド、オプション、自前の引数解析の理由、引数の解釈、非 UTF-8 の引数、run ID、環境変数) |
| [05-execution-flow.md](05-execution-flow.md) | 実行の意味論(分類、遷移、起動前の判断、リトライと `ulong` の飽和した待ち時間、再開、完了、`Conductor.Flow` の純粋関数) |
| [06-state-file.md](06-state-file.md) | ステートファイル(論理項目、JSON の形式、原子的な書き込み、読み込みの 14 段階の検査、整合性、`reset`) |
| [07-process-and-signals.md](07-process-and-signals.md) | `command` の分割、`System.Diagnostics.Process` による子プロセスの起動と待ち、中断の合図 `Interrupt`、シグナルの受信と転送、リトライの待機、OS 別の差、Windows の `.cmd` の起動、想定外の内部の誤り |
| [08-output-and-i18n.md](08-output-and-i18n.md) | 出力先、標準出力の flush と閉じたパイプ、`-v`、メッセージカタログ(日本語・英語)、`watch` の表、ロケール、エスケープ、`--help` の全文 |
| [09-converter.md](09-converter.md) | `convert`(JSON と YAML の相互変換)の読み取りの差分、書き出しの形式、`double` の書式、YAML の plain の条件、往復の保証、エラー |
| [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md) | 終了コードの全件と決定順序、エラー分類の一覧、C# でのエラー型、想定外の内部失敗(70) |
| [11-quality-and-build.md](11-quality-and-build.md) | 非機能要件、テスト構成、受け入れテスト一覧(AC-001〜550)、CI(GitHub Actions)、リリース(単一ファイル・自己完結)、デモ、依存、リポジトリのレイアウト |
| [12-rebuild-guide.md](12-rebuild-guide.md) | ゼロから実装する順序、.NET 特有の落とし穴、未確認事項の一覧、最初に確かめる小さな試験 |

## 3. 読む順序

| 目的 | 読む順序 |
|------|----------|
| 全体を理解する | 00 → 01 → 02 → 05 → 03 → 06 → 04 → 07 → 08 → 10 → 09 → 11 |
| 実装を始める | 00 → 01 → [12](12-rebuild-guide.md)(実装の順序と最初の試験)→ 12 が指す各章 |
| 定義の読み込みだけを作る | 03 → 09(読み取りを共有)→ 10(エラー) |
| 実行の流れだけを作る | 05 → 06 → 07 → 10 |
| 出力の文言を確かめる | 08 → 10 |
| 受け入れる | 11(各 AC は、該当章の字句が正本) |

各章の終わりに、その章の「実装に任せてよい点」と「未確認」がある。受け入れテスト(11 章)の期待は、定義・ステート・変換・終了コードの各章(03、06、09、10)の字句と食い違うときは、それらの章を正とする。章どうしの食い違いは、03(定義)、05(実行フロー)、06(ステート)、07(プロセスとシグナル)、09(変換)の定めを正とする。

## 4. 規定の強さの語

章によって、記号や語が少し違うが、意味は次の 3 段階に対応する。

| 強さ | 意味 | 章での表記 |
|------|------|------------|
| 必須 | 利用者から見える結果(受理・拒否、出力の文字列、終了コード、作られるファイルの内容)であり、同じにしなければならない。受け入れテストで確かめる | 「必須」「【規定】」「〜しなければならない」 |
| 目安 | 推奨する値や分け方。利用者から見える挙動を損なわない範囲で、別の値や分け方を採ってよい。基準を超えたときは警告として記録する | 「目安」「【目安】」「推奨」「仮」(暫定として置いた値) |
| 任意 | 利用者から見える結果が変わらない範囲で、実装が自由に選んでよい | 「任意」「【任意】」「実装に任せてよい」「裁量」 |

- 【.NET の設計】(07 章ほか)は、.NET の都合で選んだ設計で、利用者から見える結果が同じなら変えてよい(目安と同じ扱い)。ただし、各章が「規定」と明記したもの(シェルを介さない、転送は 1 回、OS の判定は `PlatformDetector` だけ、P/Invoke は `UnixNative` だけ、など)は必須である。
- 骨格のコード(型・シグネチャ)は、名前や細部が他の章の定めと食い違うときは、03・05・06・07・09 の定めを正とし、骨格を読み替える。
- 「確認(出典: …)」は、公式ドキュメントまたは公開ソースで確かめた点。「実装時に確認」は、版や道具の選定など、実装の時点で決める値で、この文書は決めない。「未確認」は、作成時に確かめられなかった点で、断定していない。実装時に確かめる。

## 5. 用語集

02〜11 章で使っている語を、次のように統一する。

| 語 | 意味 |
|----|------|
| 定義ファイル | パイプラインを書いたファイル。JSON(`.json`)か YAML(`.yaml`、`.yml`)。既定の名前は `conductor.json` |
| 定義 | 定義ファイルを検証して読み込んだ結果(`Definition`。`FormatVersion`、`Name`、`Jobs`)。誤りは種類・位置・固定の理由の値(`DefinitionError`)で返す |
| パイプライン | 名前をもち、ジョブの並びからなる 1 つの流れ。1 つの定義ファイルに 1 つ |
| ジョブ | 実行する外部コマンド 1 つと、その終了コードの扱い(成功・警告のコード、遷移、リトライ)。ID をもつ |
| ジョブID | ジョブの識別子(`id`)。定義の中で一意。空白だけでなく、`@` で始まらない |
| `command` | ジョブが実行するコマンド。文字列(分割して実行する)か、文字列の配列(そのまま渡す)。シェルは介さない |
| 成功コード・警告コード | ジョブの終了コードのうち、成功(`success_exit_codes`。省略時は `[0]`)・警告(`warning_exit_codes`)として扱う値 |
| 分類 | ジョブの結果を、成功・警告・エラー・分類なし(起動前の中断)に分けること(`Classifier`) |
| 遷移(`transitions`) | 成功・警告のとき、次に実行するジョブを指定すること。`@end` は、残りをスキップして終える |
| スキップ | 遷移などで、実行されないジョブ。理由は「遷移による」(`byTransition`)「`@end` による」(`byEnd`) |
| 拒否 | 順序外などで、`run` が実行を断ること(終了コード 66。`Reject(OutOfOrder)`)。スキップ対象・`@end` より後ろ・実行済みのジョブは、拒否ではなくスキップ(終了コード 0) |
| リトライ(`retry`) | 決めた終了コードで終わったジョブを、待ってから実行し直すこと。最大回数・間隔・倍率・上限を指定できる。待ち時間は `ulong` の秒(64 ビット符号なしの上限で飽和。`Saturating.MulU64`) |
| run ID | 実行単位を区別する文字列(`A-Za-z0-9._-`、UTF-8 で 1〜64 バイト、先頭は `.` でない)。ステートファイルを分ける |
| ステートファイル | 実行の途中経過を JSON で保存するファイル。名前は `<定義ファイル名>.state`(run ID があれば `<定義ファイル名>.<run ID>.state`) |
| 再開位置(`ResumeJobId`) | 次に実行する、または再開するジョブのID |
| 完了 | 全ジョブが終わった状態(`Completed = true`)。完了のステートは、読み込み(`Read`)では「空」とみなす |
| 空のステート | ステートが無いのと同じ扱い(ファイルがない、長さ 0、完了)。`StateReadOk(null)` |
| 耐久性(`Durability`) | ステートの書き込みで、内容の同期(`FileStream.Flush(true)`)まで行うか(`Durable`)、省くか(`Fast`) |
| 実行方式(`RunMode`) | `run`(1 ジョブ。`Individual`)と `pipeline`(全ジョブ。`Pipeline`) |
| at-least-once | ジョブは、少なくとも 1 回は実行される。結果の記録の前に落ちると、再実行されうる |
| 中断の合図(`Interrupt`) | シグナルを受けたことと、最初の番号を、シグナルのコールバックと待つ側(`JobRunner`、`RetryWaiter`、`Conductor.Cli` の流れ)のあいだで共有する値。コールバックは `Trigger` で旗を立てるだけ(別のスレッドで動く) |
| 起動失敗 | 子プロセスを作れなかったこと(終了コード 127)。終了の状態を取得できない場合も、安全側でこれに含める |
| シグナル終了の推定 | UNIX 系で、`Process.ExitCode` が 129〜159 で、そのジョブの成功・警告のコードに含まれないときだけ、シグナル(`ExitCode - 128`)で終了したとみなすこと(07 章 4.4 節) |
| プラットフォーム層 | OS 差を吸収する場所(`IPlatform`、`UnixPlatform`、`WindowsPlatform`、`CmdLauncher`、`PlatformDetector`、`SignalWatcher` の OS 別の部分)と、P/Invoke の `Conductor.Native.UnixNative`。OS の判定は `PlatformDetector.Detect()` だけ |
| 木(`TreeNode`) | YAML・JSON の読み取り結果の共通の木。`NullNode`、`BoolNode`、`IntNode`、`HugeIntNode`(範囲外の整数。値を持たない)、`FloatNode`、`StringNode`、`ListNode`、`MapNode`(`MapEntry` の並び)の 8 種 |
| マップ・リスト | キーと値の対応・値の並び(YAML のブロック/フロー、JSON のオブジェクト/配列) |
| 段階(Intake、Parse、Resolve、VersionCheck、Walk) | 定義の読み込みの処理の段階。前の段階で失敗したら、そこで打ち切る(03 章 2.2 節) |
| 種類(`ErrorKind`)・理由(`Reason`) | エラーの識別子(`syntax`、`typeMismatch` など)と、固定の日本語の原因の文 |
| 案内 | ステートの誤りに付ける、`conductor reset` で回復できるという文 |
| `Deps` | `Conductor.Cli` の差し替え口。環境(`CliEnvironment`)、時刻、標準出力・標準エラー(`OutputSinks`。`IOutputSink`)、定義、ステート、変換、プロセス、シグナルの `interface` の束。入口は `EntryPoint.Run(string[] args, Deps deps)` |
| `Reporter` | 書き込みの窓口(`Output`・`Error`・`Flush`)。標準出力の書き込みの失敗を `OutputException` にし、標準エラーの失敗は握りつぶす |
| flush | 標準出力・標準エラーの書き込みの完了(`Flush()`)。書くたびに行い、終了の前にも行う |
| `Escaping` | 表示用のエスケープと切り詰めの純粋な静的クラス(`Conductor.Text`) |
| TJ(テストジョブ) | 受け入れテスト用の実行ファイル `tests/Conductor.TestJob`(`tj`)。配布しない |
| AC | 受け入れテストの項目(`AC-xxx`)。11 章 4 章 |

## 6. この版が採った書式・実行環境の選択

| 項目 | 内容 | 章 |
|------|------|----|
| 定義ファイルの書式 | **JSON を主**、YAML を副。JSON(RFC 8259。コメントと末尾カンマは拒否)と YAML(1.2 のコアスキーマに沿う)。書式は拡張子で決める(`.json`、`.yaml`、`.yml`。ASCII の大文字小文字を区別しない。判定は自前で、`Path.GetExtension` に頼らない)。それ以外は `unsupportedFormat`(64)。人が書く定義にコメントが要るときは YAML を使う | 03 |
| 既定のファイル名 | `conductor.json`(YAML は `--config` で明示。`.yaml`・`.yml` を探索しない) | 03、04 |
| 定義のキー名 | `format_version`、`pipeline`(`name`、`description`、`run_id_env`、`remove_on_complete`)、`job`(`id`、`command`、`warning_exit_codes`、`success_exit_codes`、`transitions`、`retry`) | 03 |
| YAML の部分集合 | アンカー・エイリアス・マージキーは展開して受理。タグ(標準のタグも)、複数の文書、文字列でないキー、重複したキー、タブのインデントは拒否(`syntax`)。`null` は表せる | 03 |
| YAML ライブラリ | 採用案は YamlDotNet 18.1.0 の低水準の `Parser`・`Scanner`(公開ソースの読み取りで確認。動かしていない。採否は実装時の試験で決める)。位置は `Mark.Index` から自前で数える。型の判定、重複キー、アンカーの展開と上限、サロゲート・制御文字の拒否、深さの打ち切りなど、ライブラリの不足は自前で補う(追加の実装 14 項目) | 03、11 |
| JSON の読み取り・書き出し | **全面自前**(`System.Text.Json` の `Utf8JsonReader`・`Utf8JsonWriter`・`JsonSerializer`・`JsonDocument` を製品のコードで使わない)。定義・変換・ステートの 3 つが読み取りを共用する。整数は、符号を除いて 19 桁を超えれば範囲外(`HugeIntNode`)、それ以外は `long.TryParse`。`decimal` と `BigInteger` は使わない。`double.Parse` の前に字句を正規化する。列は UTF-16 のコードユニット | 03、06、09 |
| 文字コードと BOM | ファイルはバイトで読み、`UTF8Encoding(false, true)` で厳格に復号する。先頭の BOM は、定義と変換ではバイトで取り除き、ステートでは拒否する | 03、06 |
| 読み込みの上限 | ファイル 1 MiB、要素数 100,000(キーも数え、エイリアスは参照のたびに数える)、文字の大きさ 16 MiB、入れ子の深さ 32 段(根を数えず、33 段目を拒否)。深い入れ子を再帰で処理しない | 03 |
| 空白の判定 | Unicode の White_Space(25 文字)を明示した自前の表。`string.Trim()`・`char.IsWhiteSpace` を製品のコードで使わない | 03、07 |
| 文化の扱い | `InvariantGlobalization=true`。文化に依存する API を引数なしで使わず、`CultureInfo.InvariantCulture` と `StringComparison.Ordinal` を使う | 02、03、05 |
| 定義のエラーの種類 | 31 種(`unsupportedFormat` を含む)。誤りは位置の順に 1 行ずつ。理由は常に日本語の固定文で、入力の値を含めない。`unsupportedFormat` の理由は `拡張子は .yaml、.yml、.json のいずれかで書きます` | 03、08、10 |
| ステートファイル | JSON(UTF-8、BOM なし)。名前は `<定義ファイル名>.state`。キーは固定の順序、インデントは空白 2 つ、末尾は LF 1 つ。読み込みは 14 段階の検査。書き込みは、一時ファイル + `Flush(true)` + `File.Move(overwrite: true)` + ディレクトリの同期(UNIX 系) | 06 |
| ステートの理由文 | 全 23 件 | 06、10 |
| ディレクトリの同期 | UNIX 系だけ、`libc` の `open`・`fsync`・`close` を `Conductor.Native.UnixNative`(`Conductor.Runtime` の内部。`Runner` と `State` が共用)の P/Invoke で行う。`Conductor.Cli` が `StateStoreOptions.SyncDirectory` を決める | 06、07 |
| `convert` | JSON と YAML の相互変換(`--to json\|yaml`)。入力の形式は `--to` の反対とみなし、拡張子は見ない。出力のキーの順は入力の記述順、JSON はインデント 2 スペース・末尾 LF。書き出しは自前。`double` は往復できる最短の桁で書く | 09 |
| 変換のエラー | 6 種。終了コード 67 | 09、10 |
| 引数の解析 | 自前(`System.CommandLine` を使わない。値を取るオプション、省略形を受理しない点、エラーの優先順位と終了コードが合わないため) | 04 |
| 終了コード | 0、1〜255(ジョブの値)、64(定義)、65(ステート)、66(拒否)、67(変換)、68(使い方)、70(内部)、127(起動失敗)、128+番号(シグナル)。範囲合わせは `ExitCodes.For(int reported, bool isError)` | 10 |
| 想定外の内部失敗 | `Main` の `try`/`catch (Exception)`、`AppDomain.UnhandledException`、`TaskScheduler.UnobservedTaskException` で捕まえ、`conductor: error: internal error` を 1 行出して 70(`InternalFailure`)。`StackOverflowException` は捕まえられないので、再帰を使わない | 07、10 |
| 標準出力・終了 | BOM なし UTF-8 の `StreamWriter`(`Console.OpenStandardOutput()` から作る。LF 固定、書くたびに `Flush`)。`Main` は `int` を返して終わる。標準出力に書けない失敗は、`watch` が 65、`convert` が 67、`--help`・`--version` は無視して 0。**既知の差**: UNIX 系では、閉じたパイプへの書き込みが .NET の標準のストリームに捨てられ、`watch`・`convert` が 65・67 でなく 0 で終わりうる | 08、10 |
| 同期モデル | 同期(`async`・`Task`・`Thread` を使わない)。シグナルのコールバックは別のスレッドで動くので、旗を立てるだけ(`Interlocked`・`Volatile`・`ManualResetEventSlim`)。待機は `Process.WaitForExit(int)`(10 ms 以下の刻み)と `ManualResetEventSlim.Wait`(250 ms 以下の刻み) | 02、05、07 |
| シグナル | UNIX 系は SIGHUP・SIGINT・SIGQUIT・SIGTERM を `PosixSignalRegistration`(静的に保持)で受け、子の pid に `libc` の `kill` で 1 回転送する。Windows は `Console.CancelKeyPress` の `ControlC` だけを受け、転送しない。終了コードは 128+番号 | 07 |
| 子プロセス | `System.Diagnostics.Process`(`UseShellExecute=false`、標準入出力は引き継ぐ)。UNIX 系でも新しいプロセスグループ・セッションに置かない。実行ファイルは絶対パスに自前で解決する。ジョブの終了は、`ExitCode` が 129〜159 で成功・警告のコードに含まれないときだけシグナル終了とみなす。Windows の `.cmd`/`.bat` は、危険な文字を拒否したうえで、`cmd.exe /d /s /c` を `Arguments` 文字列で直接起動する | 07 |
| 待機の整数計算 | リトライの残り時間は `ulong` と `UInt128` の整数で求める(`double` を使わない)。刻みは 250 ms 以下 | 05、07 |
| ソースの構成 | `Conductor.sln`、`src/Conductor.Core`(`Text`、`Json`、`Definition`、`Convert`、`RunIds`、`Flow`)、`src/Conductor.Runtime`(`State`、`Runner`、`Native`)、`src/Conductor.Cli`(`Report`、`Cli`、`Program.cs`。`AssemblyName` は `conductor`)、`tests/Conductor.Tests`、`tests/Conductor.AcceptanceTests`、`tests/Conductor.TestJob`、`tools/Conductor.Checks`。依存の向きは `Cli` → `Runtime` → `Core` | 02、11 |
| 型とデータ | `sealed record`、`abstract record` の階層、`enum`、`interface`。失敗は値で返す。`PipelineState` だけ、手書きの `Equals` を持つ | 02、05、10 |
| ビルド・配布 | `dotnet publish -r <RID>`(単一ファイル・自己完結)。**トリミング、Native AOT、ReadyToRun は採らない**。`DebugType` は `none`。中央パッケージ管理(`Directory.Packages.props`)とロックファイル(`dotnet restore --locked-mode`) | 02、11 |
| 品質ゲート | `dotnet format --verify-no-changes`、`dotnet build -warnaserror`、`dotnet test`(3 OS)、カバレッジ(行 80% 以上。Linux で `coverlet.collector`)、依存の向きの点検(`ArchitectureTests`)、版の一致の検査(`Conductor.Checks`)、`dotnet list package --vulnerable`・`--deprecated`(定期。警告のみ)、起動時間の計測(目安) | 11 |
| リリース | `v*` のタグ(`main` 上で、`Directory.Build.props` の版と一致)で、RID ごとの成果物を zip/tar.gz にし、`SHA256SUMS` を付けて、下書きのリリースに添付する。NuGet には公開しない | 11 |

## 7. 表記のきまり

- 章への参照は、`03 章 6.3 節` のように書く。ほかの章へのリンクは、このディレクトリのファイルだけである。
- コードブロックの C# のコードは、型とシグネチャの骨格(実装ではない)である。
- メッセージの文言は、`LANG=C`(英語)と日本語の両方を、各章の表に原文のまま書いてある。理由の文(定義・ステート・変換)は、ロケールに関わらず日本語の固定文である。
- 確認できなかった点は「未確認」と書き、[12](12-rebuild-guide.md) 3 章にまとめる。
