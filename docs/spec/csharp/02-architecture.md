# 02 アーキテクチャ(ソリューション・名前空間・依存方向・型の骨格・`Deps`・実行のシーケンス)

対象: `conductor` v0.8.0 と同じ挙動を C#(.NET)で実装するための、ソースの構成。用語は [00-README.md](00-README.md) の用語集に従う。

この章は、プロジェクトと名前空間の責務、依存の向き(循環させない)、各名前空間が公開する型とメソッドの骨格(シグネチャ)、差し替え口(`Deps`)、実行のシーケンス、並行モデル、エラー処理、`Directory.Build.props` / `Directory.Packages.props` / `global.json` / 各 `.csproj` の規定、版の唯一の出所を定める。骨格は、型の形を示すためのもので、実装コードではない。メソッドの本体は、各章(03〜10)の規定から書く。

## 規定の強さと確認の表記

| 表記 | 意味 |
|------|------|
| 【規定】 | どの実装も満たす。テストや点検で確かめる |
| 【目安】 | 推奨。異なる分け方にしてよい。ただし、プロジェクトの責務と依存の向き(2・3 章)は守る |
| 実装時に確認 | 版や道具の選定など、実装の時点で決める値。この文書は決めない |
| 確認(出典: …) | 公式ドキュメント(learn.microsoft.com、dotnet.microsoft.com)を、この文書の作成時に読んで確かめた |
| 未確認 | 公式ドキュメントでも、実機でも確かめていない。この環境には .NET SDK がなく、実機で動かしていない |

---

## 1. 設計の前提

| 項目 | 内容 |
|------|------|
| 言語・実行環境 | C#(コンパイラが対応する最新の言語版。`<LangVersion>latest</LangVersion>`)、.NET。対象は、サポート中の最新の LTS と、サポート中の 1 つ前の LTS(実装時に確認)。この文書の作成時点(2026-11)では、最新の LTS は .NET 10(2025-11-11 リリース、サポート終了 2028-11-14)。1 つ前の LTS の .NET 8 は、2026-11-10 にサポートが終了している(確認。出典: .NET サポートポリシーのページ。.NET 9 も同日に終了、.NET 11 は STS)。したがって、**この版の対象は .NET 10 だけ**とする(`net10.0`。最小の版 `ConductorMinTfm` と最新の版 `ConductorMaxTfm` が、どちらも `net10.0`)。実装時に、次の LTS の登場やサポートの状況を確認し、この行と `global.json`、`Directory.Build.props`、CI の版(11 章 2.5 節)を、いっしょに改める |
| SDK | `global.json` で固定する(7 章)。版は実装時に確認 |
| 暗黙の using | `<ImplicitUsings>disable</ImplicitUsings>`。すべての `using` を明示する。`using` は、ファイルの先頭に、名前空間の昇順に並べる(`dotnet format` が検査する。11 章) |
| 型検査 | `<Nullable>enable</Nullable>`。`null` の可能性のある参照は、`?` を付けて表す。`!`(null 許容の抑制)を、理由のコメントなしに使わない。`#nullable disable` を使わない |
| 警告 | `<TreatWarningsAsErrors>true</TreatWarningsAsErrors>`。アナライザーの警告もエラー(`AnalysisLevel`、`AnalysisMode`、`EnforceCodeStyleInBuild`。7 章)。警告の抑制(`#pragma warning disable`、`[SuppressMessage]`)は、理由のコメントつきの、狭い範囲に限る |
| 文化 | `<InvariantGlobalization>true</InvariantGlobalization>`(ICU に依存しない。確認。出典: MSBuild のプロパティの文書。文化に依存するデータにアクセスしないモード)。文化に依存する API(`ToString()`、`Parse`、`ToUpper`、`ToLower`、`string.Compare`、`StartsWith`、`EndsWith`、`IndexOf`(文字列)、`string.Format`)を、引数なしで使わない。数値や日時の変換には `CultureInfo.InvariantCulture`、文字列の比較には `StringComparison.Ordinal`(または `OrdinalIgnoreCase`)を必ず指定する【規定】。点検は、アナライザー(CA1305、CA1307、CA1309、CA1310、CA1311 など。規則の名前は実装時に確認)をエラーにして行う |
| データ型 | `sealed record`(不変)、`enum`、`interface`。判別共用体は、`abstract record` と、その `sealed` な派生 `record` で書き、`switch` 式のパターンで取り出す(全件を網羅し、既定の分岐は `throw new InvalidOperationException()` ではなく、網羅をアナライザー(CS8509)に確かめさせる) |
| 整数 | 終了コードは `int`(`ExitCodes.For(int, bool)`。05 章)。待ち時間(秒)は `ulong`(64 ビット符号なし)。64 ビット符号なしの飽和演算は `ulong` と `Math.BigMul`(または `UInt128`)で行い、`checked` で例外にせず、飽和させる(折り返さない)。`double`・`decimal`・`TimeSpan` を待ち時間の計算に使わない。`<CheckForOverflowUnderflow>` は設定しない(既定の `false`)で、飽和を明示的に書く([05-execution-flow.md](05-execution-flow.md))。JSON と YAML の整数は、字句の桁数(符号を除いて 19 桁を超えれば範囲外の整数 `HugeIntNode`)と `long.TryParse` で範囲を判定する(`decimal` と `BigInteger` は使わない。`double` を経由しない。[03-definition-format.md](03-definition-format.md)) |
| 実行時の依存 | YAML の読み取りのためのライブラリ 1 つだけ(採用案は YamlDotNet 18.1.0 の低水準の `Parser`・`Scanner`。選定は [03-definition-format.md](03-definition-format.md) と [11-quality-and-build.md](11-quality-and-build.md) 8 章)。JSON の読み取りと書き出しは**全面自前**(`System.Text.Json` の `Utf8JsonReader`・`Utf8JsonWriter` を使わない。理由は 03 章 4 章、06 章 4 章、09 章 6〜7 章)。引数の解析は自前(`System.CommandLine` などは使わない。理由は [04-cli.md](04-cli.md))。ほかは BCL だけ(`System.Diagnostics.Process`、`System.IO`、`System.Runtime.InteropServices`、`System.Threading`、`System.Text` など) |
| 開発時の依存 | テストの枠組み(xUnit)、`Microsoft.NET.Test.Sdk`、カバレッジ(`coverlet.collector`)、禁止の API の検査(`Microsoft.CodeAnalysis.BannedApiAnalyzers`。`BannedSymbols.txt`)。そのほかのアナライザーは、SDK に含まれるもの(.NET のアナライザー)だけを使う。版は実装時に確認([11-quality-and-build.md](11-quality-and-build.md) 8 章) |
| 公開の範囲 | ライブラリとして公開しない(NuGet に公開しない)。`Conductor.Core` と `Conductor.Runtime` の型は、プロジェクトをまたいで使うため `public`(`Conductor.Native` の `UnixNative` だけ `internal`)。`Conductor.Cli` の型(`Conductor.Report` と `Conductor.Cli` の名前空間の型)は `internal`(`Program` を除く。`EntryPoint`・`Deps` も `internal`)。テストプロジェクトには `InternalsVisibleTo` で見せる(SDK の `<InternalsVisibleTo Include="..."/>` 項目。未確認。使えなければ `AssemblyInfo.cs` の属性にする)。`Conductor.Cli` が `InternalsVisibleTo` で見せる相手は `Conductor.Tests` と `Conductor.TestJob` だけ(11 章 AC-422) |
| 配布物 | `dotnet publish` による、RID ごとの、単一ファイル・自己完結の実行ファイル(`conductor`、Windows は `conductor.exe`)。11 章 |
| トリミング・Native AOT・ReadyToRun | **採らない**【規定】。`PublishTrimmed`・`PublishAot`・`PublishReadyToRun` を `.csproj` に書かない。理由: (1) YAML のライブラリが、リフレクションを使う可能性があり、低水準の API が、トリミングの警告なしで使えるかを確かめていない(未確認)。(2) 利用者から見える挙動が同じであることを、最小の変更で保つ。(3) ReadyToRun の効果と、各 RID での可否は未確認。起動時間は計測し(11 章 AC-488)、必要なら、任意のジョブで差を測ってから採否を決める(11 章 6.2 節) |

---

## 2. ソリューションとプロジェクトの一覧【規定】

リポジトリの直下の構成:

```text
Conductor.sln
global.json
Directory.Build.props
Directory.Packages.props
NuGet.config
BannedSymbols.txt
.editorconfig
src/
  Conductor.Core/          クラスライブラリ(純粋。プロセスにもシグナルにも依存しない)
  Conductor.Runtime/       クラスライブラリ(ステートの入出力、子プロセス、シグナル、P/Invoke)
  Conductor.Cli/           実行ファイル(AssemblyName = conductor)
tests/
  Conductor.Tests/             単体テスト
  Conductor.AcceptanceTests/   受け入れテスト(実際に conductor を起動する)
  Conductor.TestJob/           受け入れテスト用のテストジョブ(実行ファイル。AssemblyName = tj。配布物に含めない。`inject-*` モードのために Conductor.Cli を参照する)
tools/
  Conductor.Checks/            検査プログラム(版・整合・依存・網羅率・発行物・README。BCL だけ。11 章 5.4 節。配布物に含めない)
```

| プロジェクト | 種類 | 名前空間(責務) | 依存 |
|--------------|------|------------------|------|
| `src/Conductor.Core` | クラスライブラリ | `Conductor.Text`(エスケープ `Escaping`。純粋)、`Conductor.Json`(木の型、自前の JSON の読み取りと書き出し、上限の検査)、`Conductor.Definition`(定義の型、読み込みと検査、YAML の読み取り)、`Conductor.Convert`(JSON ⇔ YAML の変換)、`Conductor.RunIds`(run ID の検査と決定)、`Conductor.Flow`(純粋な判断、実行時の値の型、記録の関数 `Recorder`) | YAML のライブラリ、BCL。ほかのプロジェクトに依存しない |
| `src/Conductor.Runtime` | クラスライブラリ | `Conductor.State`(ステートの読み書き、整合性の検査)、`Conductor.Runner`(`command` の分割、子プロセス、シグナル、中断の合図、リトライの待機、プラットフォーム層)、`Conductor.Native`(`libc` の P/Invoke `UnixNative`。`internal`) | `Conductor.Core`、BCL |
| `src/Conductor.Cli` | 実行ファイル | `Conductor.Report`(出力、メッセージ、ロケール。`IOutputSink`、`Reporter`)、`Conductor.Cli`(引数解析、サブコマンド、`EntryPoint`、`Deps`、`CliEnvironment`、本番の実装)、`Program`(入口) | `Conductor.Runtime`(と、それを通して `Conductor.Core`) |
| `tests/Conductor.Tests` | テスト | `src/` と同じ構成の単体テスト、`Support/`(偽の実装)、`Inspect/`(点検) | `Conductor.Cli`、`Conductor.Runtime`、`Conductor.Core` |
| `tests/Conductor.AcceptanceTests` | テスト | 実際の `conductor` の実行ファイルを起動する受け入れテスト | `Conductor.Cli` と `Conductor.TestJob`(ビルドの順序のためだけ。`ReferenceOutputAssembly="false"`。型を参照しない) |
| `tests/Conductor.TestJob` | 実行ファイル | テストジョブ(終了コード、スリープ、標準出力への書き込み、環境変数・引数・作業ディレクトリの出力など。`inject-*` モードは `Conductor.Cli.EntryPoint.Run` を呼ぶ。[11-quality-and-build.md](11-quality-and-build.md))。配布物に含めない | `Conductor.Cli`(プロジェクト参照。`inject-*` モードだけが使う)、BCL |
| `tools/Conductor.Checks` | 実行ファイル | 検査プログラム(11 章 5.4 節)。配布物に含めない | BCL だけ |

- `Conductor.Cli` が `Conductor.Core` を直接参照しなくても、型を使えるようにする(推移的なプロジェクト参照。SDK の既定)。依存の向きの**プロジェクトの単位**は、プロジェクト参照で強制される(`Conductor.Core` が `Conductor.Runtime` を参照しようとすると、循環の誤りでビルドが失敗する)。**名前空間の単位**の依存(3 章)は、テストで点検する(3 章の 9)。
- `src/` のプロジェクトは、`tests/` と `tools/` を参照しない(11 章 AC-422)。

### 2.1 型の置き場所の決定(依存の最下層に置く。循環を避ける)【規定】

| 型 | 置き場所 |
|----|----------|
| エスケープ(`Escaping`。名前用・パス用、孤立サロゲートの置き換え、64 文字での切り詰め。純粋な静的クラス) | `Conductor.Text`(`Conductor.Core`)。`Conductor.Report`、`Conductor.Definition`、`Conductor.Convert`、`Conductor.State` が共用する(08 章 9 章) |
| 木の型(`TreeNode`、`MapNode`、`MapEntry`、`ListNode`、`IntNode`、`HugeIntNode`、`FloatNode`、`StringNode`、`BoolNode`、`NullNode`、`Pos`)、自前の JSON の読み取り(`JsonReader`、`ReadJsonResult`)と書き出し、`Utf8Text`、上限の検査(`Limits`、`LimitsKind`)、`TreeError`・`TreeErrorKind` | `Conductor.Json`(`Conductor.Core`) |
| 定義の型(`Definition`、`Job`、`Command`、`RetryPolicy`、`Backoff`、`Transition`)、定義の誤り(`DefinitionError`、`ErrorKind`)、`DefinitionFormat`、`DefinitionLoader`、YAML の読み取りと、形式に応じて木を作る関数(`TreeReader.ReadTree`、`TreeResult`) | `Conductor.Definition` |
| `RunIdError`、`RunIdErrorKind`、`RunIdParse`、`RunIdRules` | `Conductor.RunIds` |
| 実行時の値の型(`JobExecution`、`JobStatus`、`SkipReason`、`JobState`、`PipelineState`(手書きの `Equals` を持つ `sealed record`)、`JobOutcome`、`NextStep`、`ResumePoint`、`RunMode`、`FlowDefinition` など)、終了コードの定数と `ExitCodes.For(int, bool)`、`Saturating`、`Retries`、`NextSteps`、`Decisions`、`Resume`、`Classifier`、記録の関数 `Recorder` | `Conductor.Flow`(`Conductor.Core`。05 章) |
| ステートの誤り(`StateError`)、`StateOutcome`・`StateFail` など、`Durability`、`IStateStore`、`StateStoreOptions`、`Consistency` | `Conductor.State`(06 章) |
| `CommandLine`、`CommandError`、`CommandResult`、`Interrupt`、`WaitOutcome`、`IPlatform`、`PlatformKind`、`SignalStart`、`SignalWatcher`、`RetryWaiter`、`JobRunner` | `Conductor.Runner`(07 章) |
| `UnixNative`(`libc` の `kill`、ステートのディレクトリの同期。P/Invoke はここだけ。`internal`) | `Conductor.Native`(`Conductor.Runtime` の内部。`Runner` と `State` の両方が使う。07 章 7.2.1 節) |
| `Locale`、`IReportEnv`、`IOutputSink`、`IReporter`、`Reporter`、`OutputException` | `Conductor.Report`(08 章) |
| `Deps`、`CliEnvironment`、`EntryPoint`、`IClock`、`IDefinitions`、`IStates`、`IConverter`、`IProcess`、`ISignals`、`OutputSinks` | `Conductor.Cli`(`IOutputSink` は `Conductor.Report`) |

木の型と JSON の読み取りを `Conductor.Json` に置くのは、`Conductor.Definition`、`Conductor.Convert`、`Conductor.State` の 3 つが共有し、`Conductor.Definition` が `Conductor.Json` を使う(逆ではない)ことで、循環を避けるためである。YAML の読み取りは `Conductor.Definition` に置き、`Conductor.Convert` が `Conductor.Definition` を使う。

### 2.2 各名前空間の公開の骨格【規定】

型の形の骨格を示す。詳細(項目の意味、検査、誤りの種類の全件)は、各章が正である。名前が章と異なる場合は、章に合わせる。「無し」は `null`(`?`)で表す。コレクションは `IReadOnlyList<T>`(不変として扱う)。

#### `Conductor.Definition`(03 章が正)

```csharp
namespace Conductor.Definition;

public sealed record Definition(
    string Name,                        // pipeline の名前
    string? Description,
    string? RunIdEnv,                   // null なら実行時に CONDUCTOR_RUN_ID
    bool RemoveOnComplete,
    int FormatVersion,                  // 1
    IReadOnlyList<Job> Jobs);           // 記述順。1 つ以上

public sealed record Job(
    string Id,
    Command Command,
    IReadOnlyList<int> WarningExitCodes,
    IReadOnlyList<int>? SuccessExitCodes,   // null なら既定の [0]
    IReadOnlyList<Transition> Transitions,  // on ごとに高々 1 つ
    RetryPolicy? Retry);

public abstract record Command;
public sealed record LineCommand(string Line) : Command;                 // 文字列形式
public sealed record ArgvCommand(IReadOnlyList<string> Argv) : Command;  // 配列形式。1 要素以上

public enum TransitionOn { Success, Warning }
public abstract record Transition(TransitionOn On);
public sealed record ToJob(TransitionOn On, string JobId) : Transition(On);
public sealed record ToEnd(TransitionOn On) : Transition(On);            // @end

public sealed record RetryPolicy(
    IReadOnlyList<int> ExitCodes,       // 1 要素以上
    int Max,                            // 1〜100
    ulong IntervalSecs,                 // 秒。64 ビット符号なし(上限の飽和は Flow で行う)
    Backoff? Backoff);

public sealed record Backoff(int Multiplier, ulong MaxIntervalSecs);     // 2〜10 / 秒

public enum DefinitionFormat { Json, Yaml }

// 誤りは値(例外ではない)。種類は 31 種(03 章)。reason は固定の日本語の文(入力の値や、ライブラリのエラー文を含めない)。
public enum ErrorKind { UnsupportedFormat, Syntax /* ... 03 章の 31 種 */ }
public sealed record DefinitionError(ErrorKind Kind, string File, int? Line, int? Column, string Reason);

public abstract record LoadResult;
public sealed record LoadOk(Definition Definition) : LoadResult;
public sealed record LoadFailed(IReadOnlyList<DefinitionError> Errors) : LoadResult;   // 1 件以上

public static class DefinitionLoader
{
    public static DefinitionFormat? FormatFromPath(string path);            // 拡張子(大文字小文字を区別しない)
    public static LoadResult Load(string path, string? expectedPipelineName);   // 例外を送出しない
    public static LoadResult Parse(string text, DefinitionFormat fmt, string fileLabel, string? expectedPipelineName);   // 純粋
    public static string? LooseRunIdEnv(string path);                       // どの失敗でも null(例外を送出しない)
}

public abstract record TreeResult;                                          // TreeOk | TreeFailed(03 章 14.2 節)

public static class TreeReader
{
    // JSON/YAML を 03 章の規定で木に読む(段階 2〜3)。Conductor.Convert が共有する。
    public static TreeResult ReadTree(string text, DefinitionFormat fmt, string fileLabel, DefinitionFormat? convertTo = null);
}
```

#### `Conductor.Json`(03 章が正)

```csharp
namespace Conductor.Json;

// 木の型(03 章 6.1 節): Pos、TreeNode、NullNode、BoolNode、IntNode、HugeIntNode(範囲外の整数。値を持たない)、
//                      FloatNode、StringNode、ListNode、MapEntry、MapNode
public enum TreeErrorKind { Syntax, TooLarge }
public sealed record TreeError(TreeErrorKind Kind, Pos? Pos, string Reason);   // Pos が null なら行なし。固定の日本語の文

public abstract record ReadJsonResult;                                          // ReadJsonOk | ReadJsonFailed
public sealed record ReadJsonOk(TreeNode Root, Pos? DuplicateKey) : ReadJsonResult;
public sealed record ReadJsonFailed(TreeError Error) : ReadJsonResult;

public static class JsonReader
{
    // 全面自前の JSON の読み取り(定義・変換・ステートの 3 つが共用する。System.Text.Json は使わない)。
    public static ReadJsonResult Read(string text, bool deferDuplicateKey = false);
}

public enum LimitsKind { All, DepthOnly }
public static class Limits
{
    public static TreeError? Check(TreeNode root, LimitsKind which);            // 深さ・要素数・文字の大きさ。State は DepthOnly だけ
}
```

(`Conductor.Json` が `DefinitionError` を使うと、`Conductor.Definition` への依存になり循環する。したがって、`Limits.Check` と `JsonReader.Read` の誤りは、`Conductor.Json` が持つ `TreeError`(種類、位置、固定の文)とし、`Conductor.Definition` が `DefinitionError` に変える。型の置き場所は 03 章に従い、この制約(`Conductor.Json` は `Conductor.Definition` を参照しない)を守る。)

#### `Conductor.RunIds`(05 章が正)

```csharp
namespace Conductor.RunIds;

public enum RunIdErrorKind { Empty, TooLong, InvalidChar, LeadingDot }
public sealed record RunIdError(RunIdErrorKind Kind);                      // 値は持たない(10 章)

public abstract record RunIdParse;
public sealed record RunIdOk(string RunId) : RunIdParse;
public sealed record RunIdErr(RunIdError Error) : RunIdParse;

public abstract record ResolveRunIdResult;
public sealed record ResolveOk(string? RunId) : ResolveRunIdResult;
public sealed record ResolveErr(RunIdError Error, string Subject) : ResolveRunIdResult;

public static class RunIdRules
{
    public static RunIdParse Parse(string value);                           // 値の検査(UTF-8 のバイト数は Encoding.UTF8.GetByteCount)
    public static ResolveRunIdResult Resolve(string? flagValue, string envName, Func<string, string?> lookupEnv);
}
```

#### `Conductor.Flow`(05 章が正)

```csharp
namespace Conductor.Flow;

public enum JobStatus { Pending, Success, Warning, Error, Skipped }
public enum SkipReason { ByTransition, ByEnd }
public readonly record struct JobState(string Id, JobStatus Status, int? ExitCode, SkipReason? SkipReason);

public sealed record JobExecution(
    string JobId, bool Started, int? ExitCode, int? TerminatedBySignal,
    bool StartFailure, bool Interrupted, int? SignalNumber);                // 07 章 4.5 節

// 手書きの Equals を持つ(列は要素ごと、文字列は序数で比べる。record の自動の等価は IReadOnlyList を参照で比べるため。05 章 1.1 節)
public sealed record PipelineState : IEquatable<PipelineState>
{
    public required string DefinitionKey { get; init; }     // DefinitionPath と同じ値
    public required string DefinitionPath { get; init; }
    public required string PipelineName { get; init; }
    public string? RunId { get; init; }
    public required IReadOnlyList<string> JobOrder { get; init; }
    public required IReadOnlyList<JobState> Jobs { get; init; }
    public string? ResumeJobId { get; init; }
    public bool EndReached { get; init; }
    public bool Completed { get; init; }
    public bool Equals(PipelineState? other);    // 05 章
    public override int GetHashCode();
}

// 型: Classification、JobOutcome、RunMode、DecisionKind、DecisionReason、Decision、EndTarget、JobTarget、FlowJob、FlowDefinition、
//     NextStep、ResumePoint、RetryDecision(Stop | Retry)
// 純粋な関数(静的クラス): Classifier.Classify、Classifier.ClassifyAndDecide、NextSteps.NextStep、Decisions.DecideBeforeRun、
//     Retries.IsRetryTarget、Retries.Decide、Retries.NextWaitSecs(ulong)、Saturating.MulU64、Saturating.AddI32、Resume.OnInterrupt、
//     ExitCodes.For(int reported, bool isError)、ExitCodes.SignalExitCode(int)
// 記録の関数(純粋。引数を変えず、with 式で新しい値を返す。署名の正本は 05 章): Recorder.InitialState、ApplyOutcome、ApplySkip、ApplyInterrupt
// 定数: ExitCode 列挙型(10 章 8.2 節の全件)

public static class Saturating
{
    public static ulong MulU64(ulong a, ulong b);   // 64 ビット符号なしの飽和した積。折り返さない。例外にしない。Math.BigMul(a, b, out ulong low) の上位が 0 でなければ ulong.MaxValue
    public static int AddI32(int a, int b);         // 32 ビット符号つきの飽和和
}
```

#### `Conductor.State`(06 章が正)

```csharp
namespace Conductor.State;

public enum Durability { Durable, Fast }                                   // Fast は起動前の初期ステート(同期を省く)

public enum StateErrorKind { Unreadable, TooLarge, Corrupt, VersionMismatch, WriteFailed, RemoveFailed }
public enum StateErrorDetail { None, RunIdMismatch }                       // 内部の判別用(reset --all)。表示に使わない

// 誤りは値。Reason は 06 章の固定の日本語文(入力の値や下位のエラー文を含めない)。
public sealed record StateError(StateErrorKind Kind, string Path, string Reason, StateErrorDetail Detail = StateErrorDetail.None);

// 整合性の検査(純粋。06 章が正)
public abstract record ConsistencyResult;
public sealed record ConsistencyOk : ConsistencyResult;
public enum MismatchWhat { PipelineName, JobIds, JobOrder }
public sealed record ConsistencyMismatch(MismatchWhat What) : ConsistencyResult;
public static class Consistency { public static ConsistencyResult Check(PipelineState state, FlowDefinition definition); }

public sealed record ResetAllOutcome(IReadOnlyList<string> Kept);          // 読めずに残したステートのファイル名(整列済み)

public abstract record StateOutcome;                                       // すべての入出力の結果の共通の基底。失敗を値で返す。例外を送出しない
public sealed record StateFail(StateError Error) : StateOutcome;           // 失敗(読み込み・書き込み・削除のすべてで共通)
public abstract record StateReadResult : StateOutcome;                     // StateReadOk | StateFail として扱う(型の絞り込みは StateOutcome で行う)
public sealed record StateReadOk(PipelineState? State) : StateReadResult;  // null は「空」。完了のものも「空」(Read)。ReadRaw は完了のものも返す

public sealed record StateWriteOk : StateOutcome;                          // 書き込み・削除の成功。結果は StateWriteOk | StateFail
public sealed record ResetAllOk(ResetAllOutcome Outcome) : StateOutcome;   // ResetAll の成功。結果は ResetAllOk | StateFail
// 実装メモ: 戻り値の型は StateOutcome とし、メソッドごとに期待する派生型(StateReadOk / StateWriteOk / ResetAllOk)と StateFail だけが返る、と規定する

public interface IStateStore
{
    StateOutcome Read(string definitionPath, string? runId);
    StateOutcome ReadRaw(string definitionPath, string? runId);         // watch 用
    StateOutcome Write(string definitionPath, string? runId, PipelineState s, Durability durability);
    StateOutcome Reset(string definitionPath, string? runId);
    StateOutcome ResetAll(string definitionPath);
}

public sealed record StateStoreOptions(Func<DateTimeOffset> Now, bool SyncDirectory);   // 一時ファイル名用の時刻 / UNIX 系 true・Windows false(Cli が PlatformKind から決める)
public static class StateStores { public static IStateStore Create(StateStoreOptions options); }
```

- ステートの JSON の表現(ファイル用の型、キーの順、整数の幅の検査、深さの制限)は [06-state-file.md](06-state-file.md)。`PipelineState` を、直接 `JsonSerializer` に渡さない(リフレクションに依存しない、固定の順序の書き出しを、自前で行う)。読み込みは、`Conductor.Json.JsonReader.Read(..., deferDuplicateKey: true)` と `Limits.Check(root, LimitsKind.DepthOnly)` を使う。
- 入出力は、同期の BCL(`FileStream`、`File.Move`、`FileStream.Flush(true)` など)で行う。`StateError` は、例外ではなく値(`StateFail`)。`IOException` などの BCL の例外は、`IStateStore` の実装の内側で、狭く捕まえて `StateFail` に変える。それ以外の例外は捕まえない(想定外の内部の誤り。10 章)。

#### `Conductor.Runner`(07 章が正)

```csharp
namespace Conductor.Runner;

public sealed record CommandLine(string Program, IReadOnlyList<string> Args);
public enum CommandErrorKind { UnclosedQuote, Empty }
public sealed record CommandError(CommandErrorKind Kind, string Message);   // Message は固定の文(入力の値を含めない)
public abstract record CommandResult;
public sealed record CommandOk(CommandLine Line) : CommandResult;
public sealed record CommandErr(CommandError Error) : CommandResult;

public static class CommandSplitter
{
    public static CommandResult Split(string line);
    public static CommandResult Check(Command command);
}

public sealed class Interrupt                                              // 07 章 6 章
{
    public void Trigger(int n);
    public int? Number { get; }
    public bool WaitRaised(TimeSpan timeout);
}

public static class JobRunner
{
    // 例外を送出しない(07 章 4.3 節の例外は、ここで値にする)。
    public static JobExecution RunJob(Job job, Interrupt sig, IPlatform platform, Func<string, string?> lookupEnv,
                                      Func<Process, int, bool>? waitForExit = null /* 差し替え用。既定は p.WaitForExit(ms) */);
}

public abstract record WaitOutcome;
public sealed record Elapsed : WaitOutcome;
public sealed record Interrupted(int SignalNumber) : WaitOutcome;
public static class RetryWaiter { public static WaitOutcome Wait(ulong secs, Interrupt sig); }   // 例外を送出しない

public enum SignalStart { Started, AlreadyRegistered, Failed }
public sealed class SignalWatcher
{
    public static SignalWatcher Start(Interrupt sig, IPlatform platform);  // 本番: プロセスで 1 回だけ。2 回目は AlreadyRegistered
    public SignalStart Result { get; }
    public void Stop();                                                    // 本番では呼ばない(テスト用)
}

public enum PlatformKind { Unix, Windows }
public static class PlatformDetector
{
    public static PlatformKind Detect();                                   // OS の判定は、ここだけ
    public static IPlatform Create(PlatformKind kind);
}
// IPlatform(PrepareStart、Forward、ReadExit)は 07 章 8.2 節。実装は UnixPlatform、WindowsPlatform(Windows の .cmd/.bat の起動の形は CmdLauncher)。
// UnixNative(kill、ディレクトリの同期)は Conductor.Native に置く(Runner と State が共用。07 章 7.2.1 節)
```

#### `Conductor.Report`(08 章が正)

```csharp
namespace Conductor.Report;

internal enum Locale { En, Ja }

internal interface IReportEnv
{
    string? GetEnvironmentVariable(string name);
    bool IsStdoutTerminal();
    PlatformKind Platform();                                               // 色の判定に使う(Windows は色なし)
}

// 標準出力・標準エラーへの書き込みの口(2 つ)。1 回の WriteAll のあとに Flush まで行う(08 章 1.3.2)。
// 失敗は、IOException(閉じたパイプを含む)・ObjectDisposedException・EncoderFallbackException・NotSupportedException で伝える。
internal interface IOutputSink { void WriteAll(string text); }

// 標準出力への書き込みの失敗(Output 水準)。呼び出し元が 65(watch)/ 67(convert)にする(08 章、10 章)。
internal sealed class OutputException : Exception { public OutputException(Exception inner); }

internal interface IReporter
{
    void Output(string text);   // 標準出力。失敗は OutputException を送出
    void Error(string text);    // 標準エラー(誤り・知らせ・-v)。失敗は握りつぶす
    void Flush();               // 終了前の完了(08 章 1.4)
}

internal sealed class Reporter : IReporter                                 // 書き込みの窓口(08 章)。IOutputSink を包む
{
    public Reporter(IReportEnv env, IOutputSink stdout, IOutputSink stderr);   // ロケールと色は、最初に必要になったとき 1 回だけ判定する(遅延評価)
}

// 文を作る関数群(静的クラス。純粋。戻り値は string。改行を含む。書き込みはしない。全件は 08 章)。例:
//   Formatter.VerboseResult(loc, outcome, execution)、Formatter.DefinitionErrors(loc, errors)、
//   Formatter.CommandError(loc, ...)、Formatter.StateError(loc, e)、Formatter.Watch(loc, color, s)、Usage.Text(loc) など
```

- `Conductor.Report` は、`Conductor.Definition`・`Conductor.Flow`・`Conductor.State`・`Conductor.Runner`・`Conductor.Text` の型を読むだけで、それらを変更しない。環境変数と端末の判定は、`IReportEnv`(`Conductor.Cli` の `CliEnvironment` が実装する)を通して受け取る(`Environment.GetEnvironmentVariable` と `Console.IsOutputRedirected` を直接読まない)。`IOutputSink` は `Conductor.Report` に置き、`Conductor.Cli` の `Deps` が使う(`Conductor.Report` は `Conductor.Cli` を参照しないため。3 章)。

#### `Conductor.Cli`(03〜05 章)

```csharp
namespace Conductor.Cli;

internal static class EntryPoint
{
    // 引数(プログラム名を除く)を解析し、サブコマンドを実行して、終了コード(0〜255)を返す。Environment.Exit を呼ばない。
    // 想定外の内部の失敗は、ここでは捕まえない(Program.Main の入口が捕まえる。10 章)。出力の flush は Program.Main が行う(08 章 1.4)。
    // TJ の inject-* モード(11 章 3.4 節)と、単体テストが、偽の Deps を渡して呼ぶ。
    public static int Run(string[] args, Deps deps);
}
internal static class SystemDeps { public static Deps Create(); }            // 本番の実体の束(CliEnvironment.CreateProduction を含む)
```

#### `Program`(入口)

```csharp
namespace Conductor.Cli;

public static class Program
{
    // 骨格(実装ではない): 最初に内部の誤りの捕捉を設定し、EntryPoint.Run を呼び、flush して、終了コードを返す(10 章 6.1 節)。
    public static int Main(string[] args);
}
```

- `Main` は、終了コードを `return` で返す(`Environment.Exit` を通常の経路で呼ばない。07 章 10 章の内部の誤りの経路だけ)。`Main` は `async` にしない。
- `Main` の `args` は、プログラム名を含まない(`Environment.GetCommandLineArgs()` は、先頭にプログラムの名前またはパスを含む。使い分けは [04-cli.md](04-cli.md))。

---

## 3. 依存の向き【規定】

矢印は「参照する(`using` する、または完全修飾名で使う)」。**循環させない**。

```text
Program ─▶ Conductor.Cli

Conductor.Cli ─▶ Conductor.Report, Conductor.Runner, Conductor.State, Conductor.Flow,
                 Conductor.Definition, Conductor.Convert, Conductor.RunIds, Conductor.Json, Conductor.Text

Conductor.Report ─▶ Conductor.State, Conductor.Runner, Conductor.Flow, Conductor.Definition, Conductor.RunIds, Conductor.Text
                    (型だけ。Conductor.Cli を参照しない)

Conductor.Runner ─▶ Conductor.Flow, Conductor.Definition, Conductor.Native
Conductor.State  ─▶ Conductor.Flow, Conductor.Definition, Conductor.RunIds, Conductor.Json, Conductor.Text, Conductor.Native
Conductor.Native ─▶ (BCL だけ)
Conductor.Flow   ─▶ Conductor.Definition
Conductor.Convert ─▶ Conductor.Definition, Conductor.Json, Conductor.Text    (木の読み取りを共有)
Conductor.RunIds ─▶ (BCL だけ)
Conductor.Definition ─▶ Conductor.Json, Conductor.Text, (YAML のライブラリ、BCL)
Conductor.Json   ─▶ Conductor.Text, (BCL)
Conductor.Text   ─▶ (BCL だけ)

tests/Conductor.Tests ─▶ 上のすべて(型だけ)
tests/Conductor.TestJob ─▶ Conductor.Cli(inject-* モードだけ)
```

図(下へ向かって依存する):

```text
                        Program
                           │
                     Conductor.Cli ──────────┬───────────────┐
        ┌───────┬────────┼────────┬───────┐   │               │
   Report   Runner    State   Convert  RunIds        (Cli が直接使う)
     │ ╲      │ ╲       │ ╲      │
     │  ╲     ▼  ╲      ▼  ╲     └─▶ Definition ──▶ Json ──▶ Text
     │   ╲──▶ Flow ◀────(Runner も State も Flow を使う)
     ▼          │
 (State, Runner, Flow, Definition の型を読む)
                ▼
           Definition ──▶ Json            Runner ─▶ Native ◀─ State
```

プロジェクト(アセンブリ)の単位では、`Conductor.Cli` → `Conductor.Runtime` → `Conductor.Core` の一方向である(`Conductor.Native` は `Conductor.Runtime` の内部)。

規則:

1. **`Conductor.Cli` が最上位**(`Program` を除く)。ほかの名前空間は `Conductor.Cli` を参照しない。`Conductor.Report` も参照しない(環境の判定は `IReportEnv` の形で受け取り、書き込みの `IOutputSink` は `Conductor.Report` が持つ)。
2. **`Conductor.Text`、`Conductor.Json`、`Conductor.Definition`、`Conductor.Flow` が下層**。`Conductor.Text` は、`Conductor` のどの名前空間にも依存しない。`Conductor.Json` は、`Conductor.Text` だけに依存する。`Conductor.Definition` は `Conductor.Json` と `Conductor.Text` だけに依存する。`Conductor.Flow` は `Conductor.Definition` だけに依存する。入出力に触れない(`DefinitionLoader.Load` のファイルの読み取りを除く)。
3. **`Conductor.State` と `Conductor.Runner` は、`Conductor.Flow` と `Conductor.Definition` の型を使うが、互いに依存しない**。`State` は「ステートを読み書きする」だけで、ジョブを起動しない。`Runner` は「ジョブを起動して待つ」だけで、ステートに触れない。両者をつなぐのは `Conductor.Cli`。**P/Invoke(`libc` の `kill`、ステートのディレクトリの `fsync`)は、`Conductor.Runtime` 内部の `Conductor.Native.UnixNative` の 1 か所にあり、`State` と `Runner` の両方が、この名前空間にだけ依存して使う**(互いには依存しない。決定。06 章 13.1 節、07 章 7.2.1 節)。`State` が OS の違い(ディレクトリの同期の有無)を知る必要があるときは、`Conductor.Cli` が `PlatformKind` から `StateStoreOptions.SyncDirectory` を決めて渡す(`State` は OS の判定を持たない)。
4. **`Conductor.Convert` が使う `Conductor` の名前空間は `Conductor.Definition`、`Conductor.Json`、`Conductor.Text` だけ**で、木の読み取りを共有するためである。定義の検証(`DefinitionLoader.Load` の段階 4・5)は呼ばない。
5. **`Conductor.Report` は、データを受け取って文を作る**。判断をしない。何を表示するかは `Conductor.Cli` が決める。
6. **OS ごとの分岐は、`Conductor.Runner` の `PlatformDetector` と、プラットフォーム層(`UnixPlatform`、`WindowsPlatform`、`CmdLauncher`、`SignalWatcher` の OS 別の登録部分)と、P/Invoke の `Conductor.Native.UnixNative` だけ**([07-process-and-signals.md](07-process-and-signals.md) の 8 章)。OS の判定の API を呼ぶのは `PlatformDetector.Detect()` だけ。`State`、`Report`、`Cli` は、`PlatformKind` を、`CliEnvironment` から得て、引数で受け取って使う。
7. **型の置き場所は、依存の最下層**(2.1 節)。型だけを使うために、上の層や横の層を参照しない。上の層の型が要るときは、型を下の層へ移す。`Report` が `Runner` の `PlatformKind` を型として読むのは、この規則の範囲内(`Runner` は `Report` を参照しないため、循環しない)。
8. **`Deps` の各 `interface` のメソッドのシグネチャは、下層の名前空間の型だけで書く**(4 章)。`IOutputSink` は `Conductor.Report`、ほかは `Conductor.Cli` に置くが、メソッドの引数と戻り値は `Conductor.Cli` の型を使わない(偽の実装が、構造的な形だけで、`interface` を満たせるようにする)。
9. 強制の方法(2 段): (a)(プロジェクトの単位)プロジェクト参照(`ProjectReference`)の向きで強制する。(b)(名前空間の単位)`tests/Conductor.Tests` の点検テスト(`ArchitectureTests`。11 章)が、`src/` の `.cs` ファイルを走査し、`using Conductor.X;` の行と、完全修飾の `Conductor.X.` の参照から、各ファイルの名前空間ごとの依存の集合を作り、上の表の許可されない組み合わせ(例: `Conductor.Definition` から `Conductor.Flow` への参照)と、名前空間のあいだの循環を、失敗にする。外部のパッケージを足さない(ファイルの文字列の走査と、グラフの循環の検出だけで書く)。専用のライブラリ(アーキテクチャを検査するパッケージ)を使う場合は、実装時に確認(未確認。この文書は、依存を増やさない走査を基本とする)。(c)(補助)製品のコードの禁止の API(`System.CommandLine`、`System.Net`、`Task.Run`、`Task.Wait`、`.Result`、`new Thread`、`Environment.Exit`(`InternalFailure` の内部の誤りの経路を除く)、`Console.Out`、`Console.Error`、`Console.WriteLine`)を、同じ走査で失敗にする。

---

## 4. `Deps`(差し替え口)とテスト用の偽実装【規定】

### 4.1 方針

`Conductor.Cli` の流れ(`EntryPoint.Run(string[] args, Deps deps)`)は、環境(`CliEnvironment`)・時刻・標準出力/標準エラー(`OutputSinks`)・ファイル(定義・ステート・変換)・プロセス・シグナルを、**まとめて 1 つの `Deps` オブジェクト**で受け取る。環境と出力以外のフィールドは `interface` で定義する。本番の入口(`SystemDeps.Create()`)は本番の実体を渡し、テストは偽の実体を渡す。純粋な判断(`Conductor.Flow` の関数、`Conductor.State` の記録関数、`Conductor.Report` の文の組み立て)は、`interface` を通さず、`Conductor.Cli` が直接呼ぶ(差し替え不要)。

### 4.2 `interface` の定義(`Deps.cs`)

```csharp
namespace Conductor.Cli;

// 環境(環境変数、標準出力が端末か、OS の種別、作業ディレクトリ)。差し替えのため、値ではなく関数を持つ(偽の実装は、表と設定値で作れる)。
// IReportEnv(Conductor.Report)を実装する。Environment.GetEnvironmentVariable と Console.IsOutputRedirected を呼ぶのは、本番の実装(CreateProduction)だけ(04 章 5.1 節、08 章 6.3 節)。
internal sealed class CliEnvironment : IReportEnv
{
    public CliEnvironment(Func<string, string?> getEnvironmentVariable, Func<bool> isStdoutTerminal, PlatformKind platform, string cwd);
    public string? GetEnvironmentVariable(string name);   // 未設定は null
    public bool IsStdoutTerminal();                       // 本番: !Console.IsOutputRedirected(端末の判定の挙動は 3 OS で確かめる。未確認)
    public PlatformKind Platform();                       // 本番: PlatformDetector.Detect()
    public string Cwd();                                  // 本番: Directory.GetCurrentDirectory()
    public static CliEnvironment CreateProduction();
}

internal interface IClock
{
    DateTimeOffset Now();                  // ステートの一時ファイル名に使う。本番: DateTimeOffset.UtcNow
}

// 標準出力・標準エラーへの書き込みの口(2 つ)。IOutputSink は Conductor.Report に置く(3 章の規則 1)。
// 1 回の WriteAll のあとに Flush まで行う(08 章 1.3.2)。失敗は、IOException(閉じたパイプを含む)・ObjectDisposedException・
// EncoderFallbackException・NotSupportedException で伝える。呼び出し側(Report の Reporter)が、水準ごとに扱う:
// 標準出力は OutputException にして呼び出し元へ、標準エラーは握りつぶす。
internal sealed record OutputSinks(IOutputSink Stdout, IOutputSink Stderr);

internal interface IDefinitions
{
    LoadResult Load(string path, string? expectedPipelineName);
    string? LooseRunIdEnv(string path);    // どの失敗でも null(既定の名前を使う)
}

internal interface IStates : IStateStore { }   // IStateStore と同じ形。例外を送出しない

internal interface IConverter
{
    ConvertResult Convert(string inputPath, DefinitionFormat to);      // ConvertOk(text) | ConvertFailed(error)(09 章)
}

internal interface IProcess
{
    CommandResult CheckCommand(Command c);
    JobExecution RunJob(Job job, Interrupt sig);
    WaitOutcome Wait(ulong secs, Interrupt sig);                       // リトライの待機
}

internal interface ISignals
{
    // 受け取った最初のシグナルの番号を sig に立てる監視を始める。
    // AlreadyRegistered なら、流れは成功と同様に続ける。Failed は、登録の失敗(終了コード 70)。
    SignalStart Start(Interrupt sig);
}

internal sealed record Deps(
    CliEnvironment Env,
    IClock Clock,
    OutputSinks Out,            // 標準出力と標準エラー
    IDefinitions Defs,
    IStates States,
    IConverter Converter,
    IProcess Proc,
    ISignals Signals);
```

- 引数 `args` は、`EntryPoint.Run` の引数で受け取る(`CliEnvironment` に入れない)。
- **ファイルシステムの差し替え**は、`IDefinitions`・`IStates`・`IConverter` の 3 つの `interface` が担う。本番の実装は、実ファイルを読み書きする。偽の実装は、メモリに持つ(ファイルなし)。ステートの書き込みの原子的な置換の失敗や、ディスクの異常の検査は、`Conductor.State` の内側のテスト(一時ディレクトリの実ファイル)で行う。
- `IOutputSink.WriteAll` は、失敗を例外で伝える。**例外の扱いは、`Reporter` の 1 か所**(標準出力は `OutputException` に変えて送出、標準エラーは握りつぶす)。ほかの箇所では、この例外を捕まえない。標準出力と標準エラーを混ぜない。**書き込みは `Reporter` の 1 か所を通す**(順序を保つ。書くたびに `Flush()` する。[07-process-and-signals.md](07-process-and-signals.md) の 9 章)。
- `IProcess.Wait` は `RetryWaiter.Wait` を包む。偽の実装は、待ち時間を記録するだけで実際には待たない。
- すべてのメソッドは、同期のメソッドである(`async`、`Task` を使わない)。`IProcess`、`IDefinitions`、`IStates`、`IConverter` のメソッドは、例外を送出しない(失敗は値で返す。9 章)。`IOutputSink.WriteAll` だけは、上のとおり例外で伝える。

### 4.3 本番の実装(`SystemDeps.cs`)【目安】

| 名前 | `interface` | 中身 |
|------|-------------|------|
| `CliEnvironment.CreateProduction()` | `CliEnvironment` | `getEnvironmentVariable` は `Environment.GetEnvironmentVariable(name)`、`isStdoutTerminal` は `!Console.IsOutputRedirected`、`platform` は `PlatformDetector.Detect()`、`cwd` は `Directory.GetCurrentDirectory()`。端末の判定の挙動(UNIX 系と Windows の違い)は、実装時に 3 OS で確かめる(未確認) |
| `SystemClock` | `IClock` | `DateTimeOffset.UtcNow` |
| `StdSink`(標準出力用と標準エラー用の 2 つ) | `IOutputSink` | `Console.OpenStandardOutput()`(標準エラー用は `Console.OpenStandardError()`)から得た `Stream` を、BOM なしの UTF-8 の `StreamWriter`(`AutoFlush = true`、`NewLine = "\n"`。標準出力は不正な文字で例外、標準エラーは U+FFFD に置換)で包み、`WriteAll` で 1 回の書き込みと `Flush()`。エンコーディング、改行、閉じたパイプの扱い(Unix では閉じたパイプへの書き込みが捨てられ、`IOException` にならない)は 08 章 1.2・1.3 |
| `SystemDefs` | `IDefinitions` | `DefinitionLoader.Load`、`DefinitionLoader.LooseRunIdEnv` |
| `SystemStates` | `IStates` | `StateStores.Create(new StateStoreOptions(Now: ..., SyncDirectory: kind == PlatformKind.Unix))`(`SyncDirectory` を決めるのは、ここ(`Conductor.Cli`)だけ) を持ち、メソッドを委譲 |
| `SystemConverter` | `IConverter` | `Converter.ConvertFile` |
| `SystemProcess` | `IProcess` | `CommandSplitter.Check`、`JobRunner.RunJob`、`RetryWaiter.Wait` |
| `SystemSignals` | `ISignals` | `SignalWatcher.Start(sig, platform).Result`(`Stop` は本番では呼ばない) |

### 4.4 偽の実装(`tests/Conductor.Tests/Support/`)【規定】

偽の実装は、次の振る舞いを持つ(名前は実装者が決める)。すべて `Deps.cs` の `interface` を満たす。

| 偽の実装 | 振る舞い | 主な検査 |
|-----------|----------|----------|
| `FakeEnv`(`CliEnvironment` を作る補助) | 環境変数を表(`Dictionary<string, string>`)で与える。`GetEnvironmentVariable` を呼んだ回数と名前を記録する。`IsStdoutTerminal`、`Platform`、`Cwd` の値を設定する | ロケール(`LC_ALL`・`LC_MESSAGES`・`LANG`)・`NO_COLOR`・run ID の環境変数。ロケールの判定を**遅延**して、最初に必要になったとき 1 回だけであること(成功・スキップで `-v` なしなら、読まない) |
| `FakeClock` | 固定の時刻を返す。進める操作を持つ | 一時ファイル名の時刻 |
| `BufSink`(`IOutputSink`。`OutputSinks` の標準出力用と標準エラー用の 2 つを作る) | `WriteAll` で文字列に溜める。呼び出しの順序の記録も持つ。「n 回目の書き込みで失敗する(`IOException` を送出する)」設定を持つ | 出力の中身、`cannot write to standard output`(`watch` の 65、`convert` の 67) |
| `FakeDefs` | パスごとに、決めた `Definition` や誤りの並びを返す。または、実ファイル(一時ディレクトリ)を `DefinitionLoader.Load` で読む版 | 定義の誤り(64)の経路 |
| `MemStates` | ステートをメモリに持つ(ファイルなし)。読み込み・書き込み・削除の呼び出しを記録する。「読み込みで `StateFail` を返す」「書き込みで失敗する」設定を持つ | 大きなジョブ数(1,000 個)の流れの測定、ステートの異常系(65) |
| `FakeConverter` | 入力パスごとに、決めた文字列かエラーを返す | `convert` の 67 |
| `ScriptProcess` | ジョブIDごとに、台本の `JobExecution`(終了コード・シグナル終了・起動失敗・中断)を順に返す。呼び出しの記録(何が、何回、どの順で実行されたか)を持つ。`CheckCommand` は、台本の誤り、または `CommandSplitter.Check` を呼ぶ。`Wait` は、待ち時間(秒の `ulong`)を記録するだけ。「`Wait` が中断を返す」設定を持つ | 分類・遷移・再開・リトライ(待ち時間の列 1・2・4・…)・中断の経路。実際の待ちをしない |
| `FakeSignals`(`ISignals`) | `Start` の戻り値を設定する(`Started`、`AlreadyRegistered`、`Failed`)。「開始時に、すでに `sig.Trigger(n)` を呼ぶ」設定を持つ | 登録の失敗(70)、登録済み、開始時にすでに合図が立っている |
| `FakePlatform` | `IPlatform` の偽。`Forward` の呼び出し(子の識別、シグナル)を記録する。`PrepareStart` の戻り値と、`ReadExit` の結果を設定する | 転送が 1 回だけ、先に終了した子へ転送しない、起動前の合図で起動しない |
| `FakeChild` | `Func<Process, int, bool>` の差し替えで、`WaitForExit(ms)` の戻り値の列と、渡された `ms` の列を記録する。実際の `Process` を使わない待機ループの検査用 | 待ち時間の列 1、2、4、8、10、10、…(07 章 5 章) |
| 補助 | 一時ディレクトリ(`Path.GetTempPath()` の下に作り、終了後に削除する)に、定義ファイルを書く。実行して、ステートファイルの内容を読み戻す | 実ファイルを使うテスト(受け入れテストの下位) |

- 実プロセスを使うテスト(`Conductor.Runtime` の内側と受け入れテスト)は、`Conductor.TestJob` のテストジョブを使う([11-quality-and-build.md](11-quality-and-build.md))。
- 偽の実装は、スレッドを使わず、ロックを持たない。シグナルを使う偽(`Interrupt` を使うもの)は、`Trigger` を、偽の `waitForExit` や偽の `IPlatform` の中から呼ぶ形で作る(別のスレッドや本物のシグナルを使わない)。

---

## 5. 実行のシーケンス

### 5.1 入口

```text
Program.Main(args):
    内部の誤りの捕捉を設定する                          # 最初に。UnhandledException / UnobservedTaskException(10 章 6.1 節)
    try:
        code = EntryPoint.Run(args, SystemDeps.Create())
        標準出力・標準エラーを Flush する                  # 08 章 1.4
        return code                                       # 終了は、Main が int を返すことで行う
    catch (Exception):
        return InternalFailure.Report()                   # 70(10 章 6.1 節)

EntryPoint.Run(args, deps):
    parsed = 引数を解析する(04 章)
      誤り           → 使い方の誤りを標準エラーに表示、return 68
      help           → 使い方全文を標準出力へ、return 0
      version        → `conductor 0.8.0` を標準出力へ、return 0   # 版は 8 章の唯一の出所から得る
      コマンド       → run / pipeline / watch / reset / validate / convert のどれかへ
```

### 5.2 `pipeline` の実行

`Prepare`(前処理)の順序と、そこでの失敗の終了コードは、[05-execution-flow.md](05-execution-flow.md) と [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md) に固定されている。すべての呼び出しは直列で、並行に走らせない(6 章)。

```text
Cli: pipeline
 ├─ Prepare
 │    ├─ sig = new Interrupt();  r = deps.Signals.Start(sig)          → AlreadyRegistered は続行 / Failed は 70(定義の検査より前)
 │    ├─ deps.Defs.Load(...)                                          → 失敗 64(パイプライン名の照合を含む)
 │    ├─ deps.Proc.CheckCommand(全ジョブ)                              → 失敗 64(誤りのあるジョブすべてを 1 ジョブ 1 行で報告)
 │    ├─ run ID の決定(--run-id | 環境変数。RunIdRules.Resolve)       → 失敗 68
 │    ├─ deps.States.Read(...)  (完了のものは「空」)                   → 失敗 65
 │    └─ Consistency.Check(...)                                        → 食い違い 65
 └─ for job in 定義のジョブ(記述順):
      ├─ Decisions.DecideBeforeRun(...) → 実行 / スキップ / 拒否
      │    ├─ 拒否     → Report: 表示、return 66
      │    ├─ スキップ → Recorder.ApplySkip → 変化があれば deps.States.Write(Durable)、続行(完了なら finish)
      │    └─ 実行:
      │         ├─ ステートが空なら Recorder.InitialState → deps.States.Write(Fast)
      │         ├─ リトライ込みの実行
      │         │    loop { r = deps.Proc.RunJob(job, sig)
      │         │           dec = Retries.Decide(job, r, retriesDone, lastWait)
      │         │           リトライなら Report: 知らせ(reporter.Error。常に標準エラー) → w = deps.Proc.Wait(secs, sig)
      │         │                    w が Interrupted なら「起動しなかった中断」の JobExecution(Started=false, Interrupted=true, SignalNumber)を作って抜ける }
      │         ├─ Classifier.ClassifyAndDecide(..., RunMode.Pipeline)
      │         ├─ Report: 表示(-v のときの結果行・遷移行。常時のエラー行)
      │         ├─ Recorder.ApplyOutcome → deps.States.Write(Durable)
      │         ├─ エラーまたは分類なし → return ExitCodes.For(...)
      │         ├─ 完了 → finish(--remove-on-complete なら deps.States.Reset)、return 0
      │         └─ sig.Number != null → Resume.OnInterrupt → Recorder.ApplyInterrupt → deps.States.Write(Durable)、return 128 + 番号
 └─ ループを抜けても完了していない → "cannot determine whether the pipeline is complete"、return 65
```

- `run` は、`Prepare`(ジョブIDの存在の検査を含む)のあと、指定した 1 ジョブについて同じ「判断 → 実行 → 分類 → 記録」を行う。分類と次の動きは `RunMode.Individual`(`pipeline` は `RunMode.Pipeline`。05 章)で求める。
- `watch`・`reset`・`validate`・`convert` は、シグナルを準備しない。それぞれ、ステートの読み込み(`ReadRaw`)・ステートの削除(`Reset`、`ResetAll`)・定義の検査(`Defs.Load` と `Proc.CheckCommand`)・変換(`Converter.Convert`)を呼ぶ。
- `pipeline` は、次のジョブを「次の動き」の戻り値で決めない。記述順に全ジョブを見て、起動前の判断で決める(スキップ・再開は、ステートが運ぶ)。
- 合図の確認は、`Conductor.Cli` が `sig.Number` で行う(`!= null` で判定する。番号 0 を偽にしない。1 つの判断のなかで 2 回読まず、1 回読んだ値を使う。07 章 6 章)。`JobRunner.RunJob` と `RetryWaiter.Wait` が、待機中の合図を扱う。

### 5.3 責務の流れ(データ)

```text
定義ファイル ──DefinitionLoader.Load──▶ Definition ─┐
ステートファイル ──IStates.Read──▶ PipelineState ───┤
                                                     ▼
                       Decisions.DecideBeforeRun ──▶ 実行 / スキップ / 拒否
JobRunner.RunJob ──▶ JobExecution ──Classifier.ClassifyAndDecide──▶ (JobOutcome, NextStep)
                                          │
        Recorder.Apply*(結果・スキップ・中断) ──▶ 新しい PipelineState ──IStates.Write──▶ ステートファイル
JobOutcome の報告する終了コード ──ExitCodes.For──▶ conductor の終了コード
```

### 5.4 中断の伝わり方

- 中断の合図(`Conductor.Runner.Interrupt`)は、「立った/立っていない」と「最初のシグナル番号」を持つ。シグナルのハンドラ(ランタイムが別のスレッドで動かすコールバック)が `Trigger` で立て、待ち(`JobRunner.RunJob`、`RetryWaiter.Wait`)と `Conductor.Cli` の流れ(メインスレッド)が読む。
- ハンドラは、`Cancel = true` と `Trigger` を呼ぶだけ。待つ側は、短い `WaitForExit(int)`(10 ms 以下の刻み)と `ManualResetEventSlim.Wait`(250 ms 以下の刻み。`Trigger` で即座に起こされる)の繰り返しの中で、次の周に合図を読む(最大 10 ms〜250 ms 後)。詳細は [07-process-and-signals.md](07-process-and-signals.md) の 5〜7・9 章。

---

## 6. 並行モデルの方針【規定】

| 項目 | 方針 |
|------|------|
| 基本 | **同期**。メソッドは、普通のメソッド。`async`、`await`、`Task`、`Task.Run`、`Task.Wait`、`.Result`、`GetAwaiter().GetResult()`、`Parallel`、`PLINQ` を使わない(`Task.Wait` と `.Result` によるデッドロックを作らない。`async` は必要な箇所だけ、という原則に対し、この仕様には、必要な箇所がない) |
| スレッド | 製品のコードは、`Thread`、`ThreadPool`、`Timer` を作らない。`Main` を呼んだスレッド(メインスレッド)で、流れのすべてが動く。シグナルのコールバックだけが、ランタイムによって別のスレッドで動く(07 章 9 章の表) |
| 並行の実行 | 流れの制御は、仕事を並行に走らせない。ジョブ、出力、ステートの書き込みは、すべて直列に呼ぶ |
| 子プロセス | `System.Diagnostics.Process`。待機は、`WaitForExit(int)` の繰り返し(07 章 5 章)。`WaitForExit()`(引数なし)、`WaitForExitAsync`、`Exited` イベント、標準入出力のリダイレクトを使わない |
| シグナル | `PosixSignalRegistration`(UNIX 系)、`Console.CancelKeyPress`(Windows)。ハンドラは、フラグを立てる(`Cancel` と `Trigger`)だけ。複雑なことをしない |
| 待ち方 | 待機ループ: 10 ms 以下の `WaitForExit(int)`。リトライの待機: 250 ms 以下の `ManualResetEventSlim.Wait` の繰り返しと、単調な時計(`Stopwatch.GetTimestamp`) |
| 共有する可変の値 | `Interrupt`(`Interlocked` と `ManualResetEventSlim`)と、シグナル登録の印(`Interlocked`)だけ。`lock` を使わない(使う箇所がない)。`volatile` と通常のフィールドを混ぜない(`Volatile.Read` / `Volatile.Write` を使う) |
| キャンセル | 例外による中断(ハンドラからの例外の送出)、`CancellationToken` を使わない。中断は `Interrupt` で伝える |
| 出力 | 標準出力・標準エラーへの書き込みは、`Reporter`(`IOutputSink` を包む)を通し、書くたびに `Flush()` する(07 章 9.2 節、08 章) |
| 排他制御 | 同じ定義への 2 つの `conductor` の同時の実行を、排他しない(両方が動き、ステートは、どちらも壊れた形では読めない。原子的な置換による) |
| 再帰 | 深い入れ子の入力(定義・ステート・変換)に対して、再帰で読まない(`StackOverflowException` は捕まえられず、プロセスが終了する)。深さの上限の検査を、解析より前に効かせる、または明示のスタックで読む(03 章)。`command` の分割は、再帰を使わない(07 章 3 章) |
| スレッドの所有者・終了条件の表 | 07 章 9.3 節の表を、正本とする |

---

## 7. `Directory.Build.props` / `Directory.Packages.props` / `global.json` / `.csproj` の規定

### 7.1 `global.json`

```json
{
  "sdk": {
    "version": "10.0.100",
    "rollForward": "latestFeature",
    "allowPrerelease": false
  }
}
```

- `version` は、完全な版(`10.0.100` の形。`10` や `10.0` や `10.0.x` は使えない)。範囲やワイルドカードは使えない(確認。出典: global.json の文書)。実装時に、最新の安定版の SDK の版に合わせる(例の値は、この文書の作成時点の、最初の機能バンドの版)。
- `rollForward`: `latestFeature`(指定の版以上の、同じ主版・副版の、最新の機能バンドとパッチ。確認。出典: 同文書)。完全に固定するなら `disable`(厳密な一致。確認: 同文書)。この仕様は、CI と手元の開発で、同じ 10.0.x の SDK を使うために、`latestFeature` とし(決定)、CI で `dotnet --version` が `global.json` の `version` 以上で、同じ `10.0` であることを検査する(11 章)。ロックファイル(11 章 8.4 節)を使うので、機能バンドの違いで、SDK が暗黙に足すパッケージの版が変わり、`dotnet restore --locked-mode` が失敗する場合がありうる(未確認)。その場合は、`rollForward` を `disable` にして SDK を固定し、SDK の更新を、`global.json`・README・ロックファイルの同じコミットで行う。
- `allowPrerelease: false`(既定は、Visual Studio の外では `true`。確認: 同文書。プレリリースを使わないよう、明示する)。
- `global.json` の `sdk.version` の主版・副版と、`TargetFramework`(`net10.0`)と、CI の版の一致を、11 章の検査で確かめる【規定】。

### 7.2 `Directory.Build.props`(リポジトリの直下。すべてのプロジェクトに適用)

```xml
<Project>
  <PropertyGroup>
    <!-- 版の唯一の出所(8 章) -->
    <Version>0.8.0</Version>
    <IncludeSourceRevisionInInformationalVersion>false</IncludeSourceRevisionInInformationalVersion>

    <!-- サポートする .NET の版(1 章。実装時に確認して決める。この版では、どちらも net10.0) -->
    <ConductorMinTfm>net10.0</ConductorMinTfm>
    <ConductorMaxTfm>net10.0</ConductorMaxTfm>

    <LangVersion>latest</LangVersion>
    <Nullable>enable</Nullable>
    <ImplicitUsings>disable</ImplicitUsings>
    <TreatWarningsAsErrors>true</TreatWarningsAsErrors>
    <AnalysisLevel>latest</AnalysisLevel>
    <AnalysisMode>Recommended</AnalysisMode>
    <EnforceCodeStyleInBuild>true</EnforceCodeStyleInBuild>
    <InvariantGlobalization>true</InvariantGlobalization>
    <Deterministic>true</Deterministic>
    <GenerateDocumentationFile>false</GenerateDocumentationFile>
    <RestorePackagesWithLockFile>true</RestorePackagesWithLockFile>
    <!-- NuGet の監査の警告の扱い(通常のビルドではエラーにしない)は、11 章 9.2 節 -->
  </PropertyGroup>

  <!-- CI のビルドの印。パスの正規化など、公式ビルドのための設定 -->
  <PropertyGroup Condition=" '$(CI)' == 'true' ">
    <ContinuousIntegrationBuild>true</ContinuousIntegrationBuild>
  </PropertyGroup>

  <!-- 製品のコード(src/)にだけ、禁止の API の一覧を適用する(11 章 9.3 節) -->
  <ItemGroup Condition=" $(MSBuildProjectDirectory.Replace('\', '/').Contains('/src/')) ">
    <AdditionalFiles Include="$(MSBuildThisFileDirectory)BannedSymbols.txt" />
  </ItemGroup>
</Project>
```

(11 章 9.2 節の完全な内容が正本。ここでは、アーキテクチャに関わる設定だけを示す。)

| 項目 | 規定 |
|------|------|
| `Version` | `0.8.0`。唯一の出所(8 章)。`IncludeSourceRevisionInInformationalVersion` を `false` にして、`AssemblyInformationalVersionAttribute` に `+<コミット>` が付かないようにする(8 章) |
| `ConductorMinTfm` / `ConductorMaxTfm` | 製品のプロジェクトの `TargetFramework` は `$(ConductorMinTfm)`、テストのプロジェクトの `TargetFrameworks` は `$(ConductorMinTfm);$(ConductorMaxTfm)`(同じなら 1 つ)。直書きしない(11 章 AC-477) |
| `LangVersion` | `latest`(コンパイラが対応する最新の言語版。確認: 既定は、対象のフレームワークに依存する。`latest` で最新を使う。実装時に確認) |
| `Nullable` / `ImplicitUsings` | `enable` / `disable`(1 章) |
| `TreatWarningsAsErrors` | `true`。警告を、すべてエラーにする |
| `AnalysisLevel` / `AnalysisMode` | `latest` / `Recommended`(`AnalysisMode` の値は、`None`、`Default`、`Minimum`、`Recommended`、`All`。確認。出典: MSBuild のプロパティの文書)。`AnalysisLevel` は、既定で、対象のフレームワークの版(最新のときは `latest`)になる(確認。同文書)。`AnalysisLevel` と `AnalysisMode` の両方を指定すると、`AnalysisLevel` が優先される(確認: 同文書。複合値 `latest-recommended` を使えば、`AnalysisMode` は省ける)。実装時に、規則の組を `.editorconfig` で調整してよい(緩める場合は理由を書く) |
| `EnforceCodeStyleInBuild` | `true`(コードスタイルの規則を、ビルドで実行する。既定は無効。確認: 同文書)。`.editorconfig` に、規則の重大度(`dotnet_diagnostic.<ID>.severity`)を書く。`dotnet format --verify-no-changes` が通る(11 章) |
| `InvariantGlobalization` | `true`(1 章) |
| `CheckForOverflowUnderflow` | **設定しない**(既定の `false`)。算術の例外を使わない。飽和は明示的に書く(1 章。11 章 AC-541) |
| `Deterministic` | `true`(既定。再現可能なビルド) |
| `RestorePackagesWithLockFile` | `true`。`packages.lock.json` をコミットし、CI は `dotnet restore --locked-mode`(11 章 8.4 節) |
| `ContinuousIntegrationBuild` | CI のとき `true`(確認。出典: MSBuild のプロパティの文書: CI のとき、パスの正規化などを有効にする) |

### 7.3 `Directory.Packages.props`(中央パッケージ管理)

```xml
<Project>
  <PropertyGroup>
    <ManagePackageVersionsCentrally>true</ManagePackageVersionsCentrally>
  </PropertyGroup>
  <ItemGroup>
    <!-- 実行時の依存: YAML のライブラリ 1 つだけ(採用案は YamlDotNet。選定は 03 章)。正確な版を書く。版は実装時に確認 -->
    <PackageVersion Include="YamlDotNet" Version="<実装時に確認>" />
    <!-- 開発時の依存。版は実装時に確認 -->
    <PackageVersion Include="Microsoft.NET.Test.Sdk" Version="<実装時に確認>" />
    <PackageVersion Include="xunit" Version="<実装時に確認>" />
    <PackageVersion Include="xunit.runner.visualstudio" Version="<実装時に確認>" />
    <PackageVersion Include="coverlet.collector" Version="<実装時に確認>" />
    <GlobalPackageReference Include="Microsoft.CodeAnalysis.BannedApiAnalyzers" Version="<実装時に確認>" />
  </ItemGroup>
</Project>
```

- `Directory.Packages.props` をリポジトリの直下に置き、`ManagePackageVersionsCentrally` を `true` にする。各 `.csproj` は、`<PackageReference Include="..." />` に `Version` 属性を書かない(確認。出典: 中央パッケージ管理の文書)。
- 開発時のパッケージ(`xunit` など)の選定(NUnit にしてもよい)と版は、実装時に確認する(11 章 8.3 節)。実行時の依存は、`PackageVersion` に、YAML のライブラリの 1 行だけ【規定】。それ以外の実行時の依存を足さない。`CentralPackageTransitivePinningEnabled` は設定しない(11 章 8.4 節)。
- `dotnet list package --vulnerable --include-transitive` と `--deprecated` を、定期的に実行する(11 章 5.2 節)。

### 7.4 `.csproj` の骨格

`src/Conductor.Core/Conductor.Core.csproj`:

```xml
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>$(ConductorMinTfm)</TargetFramework>
  </PropertyGroup>
  <ItemGroup>
    <PackageReference Include="YamlDotNet" />
  </ItemGroup>
</Project>
```

`src/Conductor.Runtime/Conductor.Runtime.csproj`:

```xml
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>$(ConductorMinTfm)</TargetFramework>
    <!-- LibraryImport(UnixNative)のソース生成が要る -->
    <AllowUnsafeBlocks>true</AllowUnsafeBlocks>
  </PropertyGroup>
  <ItemGroup>
    <ProjectReference Include="..\Conductor.Core\Conductor.Core.csproj" />
    <InternalsVisibleTo Include="Conductor.Tests" />
  </ItemGroup>
</Project>
```

`src/Conductor.Cli/Conductor.Cli.csproj`:

```xml
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>$(ConductorMinTfm)</TargetFramework>
    <AssemblyName>conductor</AssemblyName>
    <RootNamespace>Conductor.Cli</RootNamespace>
    <IsPackable>false</IsPackable>
    <RuntimeIdentifiers>linux-x64;linux-arm64;osx-arm64;win-x64</RuntimeIdentifiers>
    <DebugType>none</DebugType>
  </PropertyGroup>

  <!-- 発行(dotnet publish -r <RID>)のときだけ有効にする。RID は、コマンドラインで指定する -->
  <PropertyGroup Condition=" '$(RuntimeIdentifier)' != '' ">
    <SelfContained>true</SelfContained>
    <PublishSingleFile>true</PublishSingleFile>
  </PropertyGroup>

  <ItemGroup>
    <ProjectReference Include="..\Conductor.Runtime\Conductor.Runtime.csproj" />
    <InternalsVisibleTo Include="Conductor.Tests" />
    <InternalsVisibleTo Include="Conductor.TestJob" />
  </ItemGroup>
</Project>
```

(版は、SDK が `<Version>` から `AssemblyInformationalVersionAttribute` を生成する。コードに版の文字列を書かない。8 章。)

| 項目 | 規定 |
|------|------|
| `OutputType` / `AssemblyName` | `Exe` / `conductor`(Windows の出力は `conductor.exe`)。`RootNamespace` は名前空間の接頭辞に合わせる |
| `PublishSingleFile` | 単一ファイルの発行。`RuntimeIdentifier`(`-r`)が要る。`SelfContained` は、明示する(確認。出典: 単一ファイルの文書は、RID の指定で `SelfContained` が `true` になると書く。.NET の版によって異なりうるため、明示する)。単一ファイルのアプリは、OS とアーキテクチャごとに作る(確認。同文書) |
| `RuntimeIdentifiers` | 4 つの RID(`linux-x64`、`linux-arm64`、`osx-arm64`、`win-x64`)。ロックファイルが、4 つの RID の依存を含むようにする(11 章 6.2 節) |
| `DebugType` | `none`(発行物に `.pdb` を含めない。11 章 AC-526) |
| トリミング・Native AOT・ReadyToRun・圧縮・自己展開 | `PublishTrimmed`、`PublishAot`、`PublishReadyToRun`、`EnableCompressionInSingleFile` を書かない。`IncludeNativeLibrariesForSelfExtract` は、まず設定しない(発行の結果、実行ファイルのほかにファイルが残る場合だけ有効にする。11 章 6.2 節) |
| `InternalsVisibleTo` | 内部の型を、`Conductor.Tests`(単体テスト)と `Conductor.TestJob`(`inject-*` モード)に見せる(SDK の `InternalsVisibleTo` 項目。未確認)。`Conductor.Runtime` は `Conductor.Tests` にだけ見せる(`UnixNative` が `internal`)。ほかには見せない(11 章 AC-422) |

`tests/Conductor.Tests/Conductor.Tests.csproj`:

```xml
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFrameworks>$(ConductorMinTfm);$(ConductorMaxTfm)</TargetFrameworks>
    <IsPackable>false</IsPackable>
    <IsTestProject>true</IsTestProject>
  </PropertyGroup>
  <ItemGroup>
    <PackageReference Include="Microsoft.NET.Test.Sdk" />
    <PackageReference Include="xunit" />
    <PackageReference Include="xunit.runner.visualstudio" />
    <PackageReference Include="coverlet.collector" />
  </ItemGroup>
  <ItemGroup>
    <ProjectReference Include="..\..\src\Conductor.Cli\Conductor.Cli.csproj" />
  </ItemGroup>
</Project>
```

`tests/Conductor.AcceptanceTests/Conductor.AcceptanceTests.csproj` は、上と同じ構成で、`ProjectReference` に `Conductor.Cli` と `Conductor.TestJob` を置く(実行ファイルのビルドの順序のため。どちらも `ReferenceOutputAssembly="false"` を付けて、型を参照しない)。`tests/Conductor.TestJob/Conductor.TestJob.csproj` は、`OutputType` が `Exe`、`AssemblyName` が `tj`、`IsPackable` が `false`、`ProjectReference` は `Conductor.Cli` だけ(`inject-*` モードのため)。`tools/Conductor.Checks/Conductor.Checks.csproj` は、`OutputType` が `Exe`、BCL だけ。テストプロジェクト・テストジョブ・検査プログラムは、配布物(発行の成果物)に含めない【規定】。

---

## 8. 版の唯一の出所【規定】

| 項目 | 規定 |
|------|------|
| 唯一の出所 | `Directory.Build.props` の `<Version>`(`0.8.0`)。版は、ここだけを書き換えて変える |
| コード内の版 | `Conductor.Cli` の `VersionInfo.Current`(静的な読み取り専用のプロパティ)。**定数として書かず**、アセンブリの `AssemblyInformationalVersionAttribute`(SDK が `<Version>` から生成する)から、起動時に 1 回だけ取り出す(`Assembly.GetExecutingAssembly().GetCustomAttribute<AssemblyInformationalVersionAttribute>()` の `InformationalVersion`)。これにより、コードの中に、版の文字列のリテラルがなくなる(二重の管理がない)。ソースコントロールの識別子が `+<コミット>` として末尾に付かないよう、`Directory.Build.props` に `<IncludeSourceRevisionInInformationalVersion>false</IncludeSourceRevisionInInformationalVersion>` を書く(確認。出典: MSBuild のプロパティの文書は、この既定が `true` で、ソースリンクがあると `+<コミット>` が付くと書く。`false` で付かなくなることは、この文書では実機で確かめていない)。属性が見つからない、または値が空のときは、`VersionInfo.Current` を `"unknown"` にせず、想定外の内部の誤りとして扱う(70) |
| 表示 | `conductor --version` は `conductor <VersionInfo.Current>`(`conductor 0.8.0`)。`--help` などに、版を書かない |
| 一致の検査 | (1) 単体テスト(`VersionTests`)が、`Directory.Build.props` を、テストの実行ファイルのディレクトリから上へたどって見つけ、XML として読み(`System.Xml.Linq`)、`<Version>` の値が、`VersionInfo.Current` と等しいことを検査する。(2) 受け入れテストが、実際の `conductor --version` の出力が `conductor <Directory.Build.props の Version>` と等しいことを検査する(11 章 AC-476)。(3) リリースの CI が、タグ(`v0.8.0`)から `v` を除いた値と、`Directory.Build.props` の `<Version>` と、ビルドした実行ファイルの `--version` の出力が、等しいことを検査する(11 章。`Conductor.Checks version`) |
| 代替(未採用) | `AssemblyInformationalVersionAttribute` の機構が、使えない場合(例: `+<コミット>` が消せない)の代替: `VersionInfo.cs` に、定数 `public const string Current = "0.8.0";` を書き、上の 3 つの検査で、`Directory.Build.props` との一致を保証する(この場合の出所は 2 か所になるが、検査で一致を強制する)。採用するときは、この表と 11 章 AC-476 を改める |

---

## 9. エラー処理の方針【規定】

| 項目 | 方針 |
|------|------|
| 基本 | 失敗は、戻り値(判別可能な和の値。`abstract record` と `sealed` な派生 `record`、または `T?`)で返す。外部の入力(引数・環境・ファイル・OS の結果)に対して、例外を送出しない |
| 例外を許す場所 | (1) 実装の不具合(論理の矛盾。起こるはずのない状態)。入口(`Program`)の捕捉が、固定の 1 行を標準エラーに出して終了コード 70 にする(07 章 10 章)。`Conductor.Cli` の流れの内側で `catch (Exception)` して握りつぶさない。(2) `OutputException`(標準出力への書き込みの失敗。`Reporter` が送出し、`watch`・`convert` の呼び出し元が捕まえる)。(3) BCL が例外を送出する場合は、その場で、想定する型だけを狭く捕まえて、値にする(例: `Process.Start` の `Win32Exception`・`InvalidOperationException`、ファイル入出力の `IOException`・`UnauthorizedAccessException`、デコードの `DecoderFallbackException`) |
| 捕まえる例外の範囲 | 想定する型だけを、狭く捕まえる。`catch (Exception)` と、型を書かない `catch` を、入口の 1 か所以外に書かない。`try` の中身は、失敗しうる 1 つの呼び出しに絞る。`when` フィルターで、`IOException` の `HResult` などを見分けるときは、見分けの理由をコメントに書く |
| 例外の内容 | 例外の `Message` と `ToString()` を、利用者への表示に使わない(入力の値、OS のメッセージ、ライブラリの文が混ざらないよう、固定の文だけを表示する。03・06・10 章) |
| 終了 | `Program.Main` が `int` を返すことで終わる。`Environment.Exit` は、内部の誤りの経路(`UnhandledException`、`UnobservedTaskException`。`InternalFailure.ReportAndExit()`)だけ(07 章 10 章、10 章 6 章)。`Conductor.Cli` の流れは、終了コードを返す |
| 想定外の失敗の捕捉 | `Main` の `try` / `catch (Exception)`、`AppDomain.CurrentDomain.UnhandledException`、`TaskScheduler.UnobservedTaskException`(いずれも `InternalFailure`。10 章 6.1 節)。捕まえられないもの(`StackOverflowException`)と、捕まえて続行すべきでないもの(`OutOfMemoryException`)は、10 章 6.3 節、07 章 10 章 |
| 例外の型 | 規定された誤りはすべて値(各章の `abstract record` の階層)。例外は、標準出力の書き込みの失敗の `OutputException`(`Exception` を継承)だけ。`ConductorException`(10 章 8.2 節)は、規定された誤りを例外として送出する型が必要になったときの基底で、この版では、継承する型がない。`Result<T, TError>`(10 章 8.2 節)は、各章の階層を使わない小さな内部の関数の補助で、任意 |
| ガード | 入力の検査に `Debug.Assert` を使わない(Release では消える)。引数の検査は、`ArgumentNullException.ThrowIfNull` ではなく、公開の入口(外部の入力)では値で返す。内部の前提(`null` でないこと)は、`Nullable` の注釈で表す |

---

## 10. ファイル単位の責務表【目安】

実装者は、ファイルの分け方を変えてよい。ただし、プロジェクトと名前空間の責務と依存の向き(2・3 章)と、OS 分岐の置き場所(`PlatformDetector` とプラットフォーム層だけ)は守る。1 ファイルに 1 つの主な型を置く。

| パス | 責務 |
|------|------|
| `src/Conductor.Cli/Program.cs` | 入口(`Main`。内部の誤りの捕捉、`EntryPoint.Run` の呼び出し、flush、終了コードを返す) |
| `src/Conductor.Cli/InternalFailure.cs` | 想定外の内部の誤りの 1 行と 70(`Report`、`ReportAndExit`。10 章 6.1 節) |
| `src/Conductor.Cli/VersionInfo.cs` | 版の取り出し(8 章) |
| `src/Conductor.Cli/EntryPoint.cs` | `Run`(入口の分岐、終了コード) |
| `src/Conductor.Cli/Args.cs`、`ArgsParser.cs` | 引数の字句の解析と検査(04 章) |
| `src/Conductor.Cli/Deps.cs`、`CliEnvironment.cs`、`SystemDeps.cs` | `Deps` と `interface`、環境、本番の実装 |
| `src/Conductor.Cli/Prepare.cs` | 前処理(`run`・`pipeline` に共通) |
| `src/Conductor.Cli/RunCommand.cs`、`PipelineCommand.cs` | `run`・`pipeline` の流れ |
| `src/Conductor.Cli/ExecJob.cs` | リトライ込みの 1 ジョブの実行、記録、表示 |
| `src/Conductor.Cli/AuxCommands.cs` | `watch`・`reset`・`validate`・`convert` |
| `src/Conductor.Cli/Report/IOutputSink.cs`、`IReportEnv.cs`、`IReporter.cs` | 出力と環境の `interface`(名前空間 `Conductor.Report`) |
| `src/Conductor.Cli/Report/Reporter.cs`、`OutputException.cs` | 書き込みの窓口(遅延したロケールと色の判定、書き込みの失敗の扱い) |
| `src/Conductor.Cli/Report/LocaleResolver.cs` | ロケールと色の判定(遅延) |
| `src/Conductor.Cli/Report/Catalog.cs` | メッセージカタログ(英語・日本語) |
| `src/Conductor.Cli/Report/Formatter.cs` | 行の組み立て(`-v`、誤り) |
| `src/Conductor.Cli/Report/WatchTable.cs` | `watch` の表 |
| `src/Conductor.Cli/Report/Usage.cs` | 使い方 |
| `src/Conductor.Core/Text/Escaping.cs` | エスケープと切り詰め、`IsSuspectText`(純粋。名前空間 `Conductor.Text`) |
| `src/Conductor.Runtime/Runner/CommandSplitter.cs` | 分割と検査 |
| `src/Conductor.Runtime/Runner/JobRunner.cs` | ジョブの起動、待ち(待機ループ)、結果の取り出し、転送の判断 |
| `src/Conductor.Runtime/Runner/RetryWaiter.cs` | リトライの待機 |
| `src/Conductor.Runtime/Runner/Interrupt.cs` | 中断の合図 |
| `src/Conductor.Runtime/Runner/SignalWatcher.cs` | シグナル監視(登録は 1 回、ハンドラは `Trigger` だけ、登録の保持と解除) |
| `src/Conductor.Runtime/Runner/PlatformDetector.cs` | OS の判定と、`IPlatform` の選択(OS の判定は、このファイルだけ) |
| `src/Conductor.Runtime/Runner/UnixPlatform.cs`、`WindowsPlatform.cs` | OS ごとの `IPlatform` の実装(起動の準備、転送、終了の取り出し) |
| `src/Conductor.Runtime/Runner/CmdLauncher.cs` | Windows の `.cmd` / `.bat` の起動の形(純粋。どの OS でも動く。07 章 4.8 節) |
| `src/Conductor.Runtime/Native/UnixNative.cs` | `libc` の `kill`・`open`・`fsync`・`close` の P/Invoke(`LibraryImport`)と `DllImportResolver`(P/Invoke は、このファイルだけ。名前空間 `Conductor.Native`。`internal`) |
| `src/Conductor.Runtime/State/StateTypes.cs`、`StatePaths.cs` | ファイル用の型、場所と名前 |
| `src/Conductor.Runtime/State/Consistency.cs` | 整合性の検査(純粋) |
| `src/Conductor.Runtime/State/StateReader.cs` | 読み込み(14 段階の検査、異常系) |
| `src/Conductor.Runtime/State/StateWriter.cs` | 書き込み(一時ファイル + 同期 + 名前の付け替え)、`Reset` |
| `src/Conductor.Runtime/State/ResetAll.cs` | `ResetAll` |
| `src/Conductor.Runtime/State/DirectorySync.cs` | ディレクトリの同期(`SyncDirectory` の真偽で、`UnixNative.SyncDirectory` を呼ぶかを決める。OS の判定は読まない) |
| `src/Conductor.Core/Json/*.cs` | 木の型、自前の JSON の読み取りと書き出し、上限の検査 |
| `src/Conductor.Core/Definition/Types.cs` | 定義の型 |
| `src/Conductor.Core/Definition/Errors.cs` | 誤りの種類(`ErrorKind`)と `DefinitionError` |
| `src/Conductor.Core/Definition/DefinitionLoader.cs`、`Intake.cs` | `Load`(場所の決定、読み取り、書式の判定、段階の組み立て)、ファイルの読み取り、大きさの上限、拡張子の判定 |
| `src/Conductor.Core/Definition/TreeReader.cs`、`YamlTreeReader.cs` | 形式に応じた木の読み取り(YAML の位置つきの読み取り。展開と上限を含む) |
| `src/Conductor.Core/Definition/Depth.cs`、`Walk.cs`、`Loose.cs` | 入れ子の深さの検査、構造・型・遷移・終了コード・名前の検査、`run_id_env` のゆるい読み取り |
| `src/Conductor.Core/Convert/Converter.cs`、`YamlWriter.cs`、`JsonWriter.cs`、`ConvertErrors.cs` | 変換の入口(`ConvertFile`、`ConvertText`)、YAML の書き出し、JSON の書き出し、誤り |
| `src/Conductor.Core/RunIds/RunIdRules.cs` | run ID の型、検査、決定 |
| `src/Conductor.Core/Flow/FlowTypes.cs` | 実行時の値の型 |
| `src/Conductor.Core/Flow/Classifier.cs` | 分類 |
| `src/Conductor.Core/Flow/Decisions.cs` | 起動前の判断 |
| `src/Conductor.Core/Flow/NextSteps.cs` | 次の動き |
| `src/Conductor.Core/Flow/Resume.cs` | 中断時の再開位置 |
| `src/Conductor.Core/Flow/Recorder.cs` | 記録関数(純粋) |
| `src/Conductor.Core/Flow/Retries.cs`、`Saturating.cs` | リトライの判断と待ち時間(`ulong` の飽和) |
| `src/Conductor.Core/Flow/ExitCodes.cs` | 終了コードの定数と範囲合わせ |
| `tests/Conductor.Tests/<名前空間>/*Tests.cs` | 単体テスト(`src/` と同じ構成) |
| `tests/Conductor.Tests/Support/*.cs` | 4.4 節の偽の実装と補助 |
| `tests/Conductor.Tests/Inspect/ArchitectureTests.cs` | 依存の向き・循環・OS 分岐の置き場所・禁止の API の点検(3 章の 9、07 章 8 章) |
| `tests/Conductor.Tests/Inspect/VersionTests.cs` | 版の一致の検査(8 章) |
| `tests/Conductor.AcceptanceTests/*.cs` | 実際に `conductor` を起動する受け入れテスト |
| `tests/Conductor.TestJob/*.cs` | テストジョブ(`inject-*` モードは `EntryPoint.Run` を呼ぶ) |
| `tools/Conductor.Checks/*.cs` | 検査プログラム(11 章 5.4 節) |

単体テストの置き場所と受け入れテストの実行の方法は [11-quality-and-build.md](11-quality-and-build.md)。

---

## 11. 未確認の事項

- この環境には .NET SDK がなく、骨格のコンパイル、プロジェクトのビルド、`dotnet publish` を実行していない。型の骨格は、書き方の例であり、コンパイルを確かめていない(特に、`Conductor.Json` が `DefinitionError` に依存しないための `TreeError` の分離、`Reporter` の引数名、`internal` な型と `public` な型の可視性の整合)。
- `InternalsVisibleTo` 項目(SDK の機能)、`SatelliteResourceLanguages` の効果、`IncludeSourceRevisionInInformationalVersion` を `false` にしたときの `AssemblyInformationalVersionAttribute` の値(`+<コミット>` が付かないこと)、`RuntimeIdentifiers` とロックファイルの相性は、公式ドキュメントで確認していない。
- `rollForward = latestFeature` と `dotnet restore --locked-mode` の併用(SDK の機能バンドで、暗黙のパッケージの版が変わらないこと)。
- `AssemblyVersion`・`FileVersion` が `<Version>` から導かれる規則(8 章で、これらを表示に使わない)。
- アナライザーの規則の名前(CA1305 など)と、`AnalysisMode = Recommended` で有効になる規則の組。
- 文化に依存する API の、`InvariantGlobalization` 下での挙動の詳細(1 章の規定は、API の呼び出しの明示で、挙動に依存しない)。
- 依存の点検を、アーキテクチャ検査のパッケージなしで書く走査の網羅性(ファイルの文字列の走査では、`using` の別名や `global using` を見落としうる。この仕様は `ImplicitUsings` を無効にし、`global using` を使わない前提)。
- トリミング・Native AOT・ReadyToRun を採らない判断の前提(YAML ライブラリの低水準 API がリフレクションを使うか、ReadyToRun の効果)。
- 07 章の未確認の事項(`Process`、`PosixSignalRegistration`、`libc` の `kill`、Windows の挙動)は、07 章 13 章に集めている。
- 名前と形を合わせた章: 03 章(定義と木の型の置き場所: `Conductor.Json` と `Conductor.Definition`)、05 章(`Conductor.Flow` の型と関数、`Recorder` の置き場所、`RetryPolicy.IntervalSecs` が `ulong`、`RunMode`、`ExitCodes.For(int, bool)`)、06 章(`Conductor.State` の型、`StateOutcome`、`IStateStore`、`StateStoreOptions.SyncDirectory`)、07 章(`Conductor.Runner` と `Conductor.Native.UnixNative`)、08 章(`IOutputSink`、`Reporter`、`OutputException`、`Conductor.Text.Escaping`)、10 章(`InternalFailure` と終了コード)、11 章(`EntryPoint.Run`、`ArchitectureTests`、`VersionTests`、`tools/Conductor.Checks`、CI の版の一致の検査)。
