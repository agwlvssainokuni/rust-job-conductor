# 05. 実行の意味論(終了コード・分岐・再開・run ID・リトライ・パイプライン)

対象: C#(.NET)で実装する `conductor` v0.8.0 と同じ挙動の実行ツール。この文書は、`run`(ジョブを 1 つ実行)と `pipeline`(定義の全ジョブを順に実行)の実行の流れを、判定表・状態遷移・擬似コードで定める。数値の計算(待ち時間の倍率・上限・飽和・秒から待機への変換)は、`ulong` と `Math.BigMul` による飽和演算として、式・境界の例・落とし穴まで定める。待機は同期で、`ManualResetEventSlim` をシグナルのハンドラが `Set` して起こす方式に決める(§7.8)。この文書だけで、C# でゼロから同じ挙動を実装できることを目標にする。

ステートファイルの論理項目・保存の性質・読み込みの検査は [06-state-file.md](06-state-file.md) にある。この文書の「ステートへの記録」は、06 の形式のファイルを書く前の、メモリ上のステート(以下「ステート値」)の更新を指す。定義ファイルの書式は [03-definition-format.md](03-definition-format.md)、コマンドラインは [04-cli.md](04-cli.md)、プロセス起動とシグナルは [07-process-and-signals.md](07-process-and-signals.md)、メッセージの文言は [08-output-and-i18n.md](08-output-and-i18n.md)、終了コードの総覧は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md) を参照。

この文書の確認の範囲: この文書を書いた環境には .NET SDK が無く、実機で動かせていない。したがって、C# のコードの挙動は実行して確かめていない。API の存在と契約は、learn.microsoft.com の公式ドキュメントを取得して確認できたものだけを「確認(出典: …)」と書き、取得できなかったもの・文書に記載が無かったものは「未確認」と書く(§15 に一覧)。算術の境界値の表は、別の言語の任意精度整数で計算して確かめた値で、C# で実行した結果ではない。実装時に、§14 の単体テストで固定する。

## 0. 構成の決定

### 0.1 名前空間と置き場所

「判断」(分類・次の動き・起動前の判断・リトライの判断・再開位置)と「記録」(ステート値の複製と更新)は、ファイルにもプロセスにも時計にも触れない**純粋な関数**にし、例外を投げず必ず結果を返す。

| 置き場所 | 置くもの |
|---|---|
| `Conductor.Core` の `Conductor.Flow` | 値の型(`JobStatus`、`SkipReason`、`JobState`、`PipelineState`、`JobExecution`、`JobOutcome`、`NextStep`、`Decision` ほか。`RetryPolicy` と `Backoff` は 03 章の `Conductor.Definition` の型を使う)、`FlowDefinition`(判断に必要な部分だけの定義の見え方)、`Classifier.Classify`、`Classifier.ClassifyAndDecide`、`NextSteps.Next`、`Decisions.DecideBeforeRun`、`Retries.Decide`、`Retries.NextWaitSecs`、`Saturating.MulU64`、`Saturating.AddI32`、`Resume.OnInterrupt`、`ExitCodes.For`、`ExitCodes.SignalExitCode`、記録の関数(`Recorder.ApplyOutcome`、`ApplySkip`、`ApplyInterrupt`、`InitialState`)。`System.IO`・`System.Diagnostics`・`System.Threading`・`System.Environment`・`DateTime.Now`・`Stopwatch` を使わない |
| `Conductor.Core` の `Conductor.RunIds` | run ID の検査(§6.1)。純粋 |
| `Conductor.Runtime` の `Conductor.Runner` | `run_job`(子プロセス)、`RetryWaiter`(§7.8)、`Interrupt`(§15.1)。シグナル・時計・スレッドに触れる |
| `Conductor.Runtime` の `Conductor.State` | 読み書き・整合性検査・`reset`([06](06-state-file.md)) |
| `Conductor.Cli` | `prepare`、`run`・`pipeline` の流れ(§5)、リトライの実行ループ(§7.7) |

依存の向きは `Cli` → `Runtime` → `Core`。`Conductor.Flow` は `Conductor.Definition` の型だけに依存してよい([02-architecture.md](02-architecture.md))。

### 0.2 型の約束(この文書の全体)

| 値 | C# の型 | 理由・注意 |
|---|---|---|
| 終了コード(ジョブ自身のもの、`ReportedExitCode`、成功・警告コードの一覧の値) | `int` | 32 ビット符号付き。整形(§2.2)は `reported & 0xFF` を使う。`int` の `&` は負数を 2 の補数のビット列として扱うので、負数でも下位 8 ビットが正しく取れる(例: `(-1) & 0xFF` は 255、`(-256) & 0xFF` は 0)。これは C# の言語仕様の性質だが、この環境では実行して確かめていない(§14 のテストで固定する) |
| 待ち時間の秒(`IntervalSecs`、`MaxIntervalSecs`、各回の待ち、`lastWaitSecs`) | `ulong` | 0〜2^64-1。符号なし 64 ビットの飽和演算(§7.6)。`long` や `double` に変換して計算しない |
| 倍率 `Multiplier` | `int`(2〜10。03 章の `Backoff`) | `Saturating.MulU64` には `(ulong)multiplier` で渡す(検証済みなので 2 以上の正の値) |
| `Max`、`retriesDone`、知らせの回数 `n` | `int` | `Max` は 1〜100 |
| シグナル番号 | `int` | 終了コード `128 + n` の計算に使う。飽和加算(§2.2) |
| 時刻・経過 | `long` の `Stopwatch` のタイムスタンプ(ティック) | 単調増加のものだけを使う(§7.8)。`DateTime.UtcNow`(実時計)は待機の計測に使わない |
| 真偽 | `bool` | 値の有無は `int?`・`string?`・`SkipReason?`(`Nullable<T>` と参照型の null 許容)で表す。「0 が無い」を表すために 0 や -1 を番兵に使わない |

- 「無し」は `null` で表す。`<Nullable>enable</Nullable>` なので、null になりうる値は型で `?` を付ける。ステートの JSON の `null` は、読み込みで型の誤りとして拒否する([06](06-state-file.md))ので、メモリ上の「無し」と JSON の `null` は混ざらない。
- ステート値と判断の入出力は不変の値にする: 単純な値は `readonly record struct`、項目が多い・参照型を含むものは `sealed record`、固定の集合は `enum`。更新は `with` 式か新しい値を作る(元は変えない)。
- **`record` の `Equals` の落とし穴**: `record`(と `record struct`)が合成する `Equals` は、各フィールドを `EqualityComparer<T>.Default` で比べる。配列・`List<T>`・`IReadOnlyList<T>` のフィールドは**参照の同一性**で比べられ、同じ要素を持つ別の配列は等しくならない。したがって、「ステート値の全項目の比較」(§4.3)に使う `PipelineState`(位置引数の `sealed record` でも、この文書の骨格のような `init` プロパティの `sealed record` でもよい)は、`Equals(PipelineState?)` と `GetHashCode` を手で書き(`IEquatable<PipelineState>`)、`JobOrder` と `Jobs` を**要素ごとに(文字列は序数比較で)**比べる。`JobState`(配列を持たない)は合成の `Equals` でよい。これを単体テスト(§14)で固定する: 要素の同じ別々の配列を持つ 2 つの `PipelineState` が等しい、1 要素だけ違えば等しくない、`null` と値は区別される。
- 文字列の比較は常に序数(`StringComparison.Ordinal`、または `==` / `string.Equals(a, b)`。後者は序数)。`CompareTo`・`string.Compare(a, b)`・`Array.Sort(string[])` の既定・`StartsWith(string)`・`IndexOf(string)`・`ToUpper()`・`ToLower()` は文化に依存する(既定が文化依存)ので使わない。並べ替えは `StringComparer.Ordinal` を明示する。`InvariantGlobalization=true`([02](02-architecture.md))でも、書き方を序数に固定して意図を残す。数値の文字列化は `CultureInfo.InvariantCulture`。
- 列挙の文字列表現(ステートの `"pending"` など)は、`Enum.ToString()` / `Enum.Parse` に頼らず、`switch` 式の対応表で明示する(§1.1)。`Enum.ToString()` は名前の大文字小文字がステートの文字列と違い、トリミング・AOT でも使いにくい。

### 0.3 整数演算の方針(`checked` と飽和)

- プロジェクトは `<CheckForOverflowUnderflow>` を有効にしない(既定の unchecked のまま)。ただし、この文書の演算は、`checked` が有効でも例外にならないよう、`unchecked` に依存しない形で書く: `ulong` の乗算は `Math.BigMul`(§7.6)、`int` の加算は `long` に広げて飽和(§2.2)。`checked` による例外(`OverflowException`)で飽和を代用しない(「飽和」と「例外」は別の挙動で、ここは飽和が規定)。
- `UInt128` の `*` は、`checked` の文脈では `op_CheckedMultiply` が使われうる(未確認)ので、飽和の乗算に `UInt128` の `*` を使う場合は `unchecked(...)` で囲む。この文書の標準は `Math.BigMul` で、こちらは文脈に依存しない。
- 整数の字句(定義の `interval`、JSON の数値)の変換は、`int.Parse` / `Convert.ToInt32` / `double.Parse` に頼らず、ASCII の `0`〜`9` を 1 文字ずつ走査して自前で数える(§7.3)。

---

## 1. 用語とデータ

### 1.1 ジョブの状態(`JobStatus`)

| 値(ステート上の文字列) | 意味 |
|---|---|
| `pending` | まだ起動していない。記録がないジョブも pending と同じに扱う |
| `success` | 成功(成功コードで終わった) |
| `warning` | 警告(警告コードで終わった) |
| `error` | エラー |
| `skipped` | スキップ。理由 `skip_reason` が必ず付く |

- 「済んだ(settled)」状態 = `success` / `warning` / `skipped`。
- 「結果のある」状態 = `success` / `warning` / `error`(終了コードを持てる)。
- スキップ理由(`SkipReason`): `byTransition`(遷移によるスキップ)、`byEnd`(`@end` によるスキップ)。

型の骨格(`Conductor.Core/Flow/Types.cs`。メモリ上の名前は PascalCase、ステートファイルのキー名は snake_case。対応は [06](06-state-file.md) §2):

```csharp
namespace Conductor.Flow;

public enum JobStatus { Pending, Success, Warning, Error, Skipped }
public enum SkipReason { ByTransition, ByEnd }

public static class WireNames   // ステートの文字列との対応。reflection を使わない
{
    public static string ToWire(JobStatus s) => s switch
    {
        JobStatus.Pending => "pending", JobStatus.Success => "success",
        JobStatus.Warning => "warning", JobStatus.Error => "error",
        JobStatus.Skipped => "skipped",
        _ => throw new ArgumentOutOfRangeException(nameof(s)),
    };
    public static bool TryParse(string wire, out JobStatus status) { /* 序数の switch。大文字小文字を区別 */ throw null!; }
    public static string ToWire(SkipReason r) => r switch
        { SkipReason.ByTransition => "byTransition", SkipReason.ByEnd => "byEnd",
          _ => throw new ArgumentOutOfRangeException(nameof(r)) };
    public static bool TryParse(string wire, out SkipReason reason) { throw null!; }
}

public readonly record struct JobState(string Id, JobStatus Status, int? ExitCode, SkipReason? SkipReason);

public sealed record PipelineState : IEquatable<PipelineState>
{
    public required string DefinitionKey { get; init; }    // DefinitionPath と同じ値
    public required string DefinitionPath { get; init; }
    public required string PipelineName { get; init; }
    public string? RunId { get; init; }
    public required IReadOnlyList<string> JobOrder { get; init; }
    public required IReadOnlyList<JobState> Jobs { get; init; }   // JobOrder と同じ順・同じ数
    public string? ResumeJobId { get; init; }
    public bool EndReached { get; init; }
    public bool Completed { get; init; }

    public bool Equals(PipelineState? other) { /* §0.2: 全項目。列は要素ごと、文字列は序数 */ throw null!; }
    public override bool Equals(object? obj) => Equals(obj as PipelineState);
    public override int GetHashCode() { throw null!; }   // Equals と整合させる(使わなくても必須)
}
```

`throw null!` は骨格の本体の省略記号で、実装では値を返す。`JobState` の `SkipReason` という名前は型 `SkipReason` と同じだが、`record struct` の位置パラメーターとしては有効(型名と同名のプロパティは C# で許される。曖昧さが出る箇所は `Flow.SkipReason` と完全修飾する)。

### 1.2 1 回の実行結果(`JobExecution`)

ジョブを 1 回起動して終わるまでの観測結果。

| フィールド | 型 | 意味 |
|---|---|---|
| `JobId` | `string` | 定義のジョブID |
| `Started` | `bool` | 起動した(子プロセスの生成に成功した)か。「中断の合図が起動の前に立っていたため起動しなかった」ときだけ `false`。ほかの起動失敗でも `Started` は `true` のまま(`StartFailure` を見る) |
| `ExitCode` | `int?` | プロセスの終了コード。取れないとき(シグナル終了、起動失敗)は `null` |
| `TerminatedBySignal` | `int?` | ジョブがシグナルで終了したときの番号(UNIX 系のみ。得かたは [07](07-process-and-signals.md)) |
| `StartFailure` | `bool` | 起動に失敗した。command を引数に分けられない、プロセス生成の失敗(`Process.Start` の例外)、終了状態を取得できない、のいずれか |
| `Interrupted` | `bool` | conductor がシグナルを受け取り、中断の合図が立っている(ジョブの終了を観測した直後に立っていた場合を含む) |
| `SignalNumber` | `int?` | `Interrupted` のとき、最初に受け取ったシグナルの番号 |

```csharp
public sealed record JobExecution(
    string JobId, bool Started, int? ExitCode, int? TerminatedBySignal,
    bool StartFailure, bool Interrupted, int? SignalNumber);
```

ジョブの起動手順(`RunJob`。子プロセスの API と詳細は [07](07-process-and-signals.md)):

1. command を引数に分ける(規則は 07)。失敗なら `StartFailure = true`(`Started = true`)で終わる。
2. 中断の合図が立っていれば、`Started = false, Interrupted = true, SignalNumber = 番号` で終わる(起動しない)。
3. 子プロセスを生成する(`Process.Start`)。生成の失敗(`Win32Exception` など)なら `StartFailure = true` で終わる。
4. 終了まで待つ。待機中に合図が立てば、子プロセスに同じシグナルを**1 回だけ**転送して、終了を待ち続ける。
5. 終了コードとシグナル終了を記録する(`Process.ExitCode`。シグナルで終わった場合の値の読み方は 07)。終了状態を取得できなければ `ExitCode`・`TerminatedBySignal` とも `null` のまま(§2.3 の順 5 に当たり、`ConductorError` つきのエラー(終了コード 1)になる)。
6. 終了を観測した直後に合図が立っていれば `Interrupted = true, SignalNumber = 番号`。

**`ExitCode` の範囲(Windows。未確認)**: `Process.ExitCode` は `int`(符号付き 32 ビット)。Windows のプロセスの終了コードは符号なし 32 ビット(DWORD)で、`0xC000013A` のような値が負の `int` として現れる、と理解しているが、この環境では確かめていない。`int` に収まる値はそのまま `ExitCode` に入れる。ステートの `exit_code` も 32 ビット符号付き整数で([06](06-state-file.md))、符号なしの値を足し引きして直す必要は無い(`int` の範囲に収まっている以上、変換は起きない)。下位 8 ビットは同じなので、プロセスの終了コードに影響しない。

### 1.3 結果(`JobOutcome`)

| フィールド | 意味 |
|---|---|
| `Classification` | `Success` / `Warning` / `Error`、または**分類なし**(「起動しなかった中断」。型では `null`) |
| `ReportedExitCode` | スケジューラへ返す終了コード(`int`)。後で 0〜255 に整える(§2.2) |
| `ConductorError` | 「成功コードに含まれない 0」など、conductor 自身が原因を標準エラーに表示するエラーのとき `true` |

```csharp
public enum Classification { Success, Warning, Error }

public readonly record struct JobOutcome(Classification? Classification, int ReportedExitCode, bool ConductorError);
```

### 1.4 実行方式(`RunMode`)

`enum RunMode { Individual, Pipeline }`。`Individual` は `conductor run <job-id>`、`Pipeline` は `conductor pipeline <name>`。`@end` の扱いと完了の判定だけが違う(§3、§5)。

---

## 2. 終了コード

### 2.1 conductor が返す終了コード(プロセスの終了コード)

| 値 | 名前 | 意味 |
|---|---|---|
| 0 | OK | 成功、警告、スキップ、正常終了 |
| ジョブの値(1〜255) | - | エラーになったジョブの終了コードをそのまま(範囲外は §2.2) |
| 1 | - | 成功コードに含まれない 0 で終わったジョブ(`ConductorError`)。終了コードを取れなかったエラーも 1 |
| 64 | DEFINITION | 定義の誤り(ファイルがない・読めない・検証の誤り・パイプライン名の不一致・command の誤り) |
| 65 | STATE | ステートの誤り(読めない・壊れている・版の不一致・定義との食い違い・書き込み/初期化/削除の失敗)、完了を判定できない、`watch` の出力失敗 |
| 66 | REJECTED | 拒否(順序外) |
| 67 | CONVERSION | convert の失敗(この文書の対象外) |
| 68 | USAGE | 使い方の誤り(知らないジョブID、run ID の値の誤りを含む) |
| 70 | INTERNAL | 内部の誤り(想定外の異常終了、シグナル処理の準備の失敗) |
| 128 + 番号 | - | ジョブがシグナルで終了した、または conductor が中断された(SIGINT = 130、SIGTERM = 143) |

64〜68・70 は、ジョブ自身の終了コードと重なりうる。区別は標準エラーの `conductor: error: ...` の行で行う。

### 2.2 `ExitCodes.For(reported, isError)`: 0〜255 への整形

プロセスの終了コードは 0〜255 に収める。

```text
if 0 <= reported <= 255:  return reported
low = reported & 0xFF              # 負数も含め、2 の補数表現の下位 8 ビット
if low == 0 and isError: return 1  # エラーを成功に見せない
return low
```

| reported | isError | 結果 |
|---|---|---|
| 3 | true | 3 |
| 256 | true | 1(下位 8 ビット 0 → 1) |
| 256 | false | 0 |
| 257 | true | 1 |
| 511 | true | 255 |
| -1 | true | 255 |
| -256 | true | 1 |

`isError` は `Classification == Error`。分類なし(起動しなかった中断)と成功・警告は `false`。

`Main` が返す `int` は、この整形のあとの値(0〜255)にする。範囲外の値を返したときの OS ごとの切り詰めに頼らない(`Main` の戻り値が範囲外のとき、Unix 系では下位 8 ビットになると理解しているが、未確認)。

中断の終了コード `ExitCodes.SignalExitCode(n)` = `For(Saturating.AddI32(128, n), true)`(パイプラインのジョブのあいだの中断で使う)。

`Saturating.AddI32(a, b)`(32 ビット符号付き整数の飽和加算): `long` に広げて足し、`int.MaxValue`(2147483647)を超えるなら `int.MaxValue`、`int.MinValue`(-2147483648)を下回るなら `int.MinValue` にする。

```csharp
public static int AddI32(int a, int b)
{
    long sum = (long)a + b;                    // long は int 2 つの和で桁あふれしない
    return sum > int.MaxValue ? int.MaxValue : sum < int.MinValue ? int.MinValue : (int)sum;
}
```

例: `AddI32(128, 2147483647)` = 2147483647。実在のシグナル番号(1〜64 程度)では起きない。

### 2.3 成功・警告・エラーの分類と返す値(`Classify`)

- **成功コードの集合** = 定義の `success_exit_codes` があればその値だけ(0 を書かなければ 0 でも成功にならない)、無ければ `[0]`。
- **警告コードの集合** = `warning_exit_codes`(0 を含まず、成功コードと重ならないことは定義の検証で保証済み)。

判定は次の順に、最初に当てはまるものを採る(上ほど優先)。入力は 1 回の実行結果(§1.2)とジョブ定義。

| 順 | 条件 | Classification | ReportedExitCode | ConductorError |
|---|---|---|---|---|
| 1 | `Started = false` かつ `Interrupted` | なし(`null`) | `AddI32(128, SignalNumber ?? 0)` | false |
| 1' | `Started = false` かつ `Interrupted` でない(あり得ない組み合わせ) | Error | 127 | false |
| 2 | `Interrupted`(起動済み) | Error | `AddI32(128, SignalNumber ?? 0)`。ジョブ自身の終了コードは見ない | false |
| 3 | `StartFailure` | Error | 127 | false |
| 4 | `TerminatedBySignal` が有り(値 n) | Error | `AddI32(128, n)` | false |
| 5 | `ExitCode` が無い(起動して終わったのに終了コードがない) | Error | 1 | **true** |
| 6 | `ExitCode` ∈ 成功コード | Success | 0 | false |
| 7 | `ExitCode` ∈ 警告コード | Warning | 0 | false |
| 8 | `ExitCode = 0`(成功コードに無い) | Error | 1 | **true** |
| 9 | それ以外 | Error | `ExitCode` そのまま(負数も、255 超もそのまま保持。整形は §2.2) | false |

全組み合わせの例(定義 `warning_exit_codes = [3, 4]`、`success_exit_codes` 省略):

| 実行結果 | 分類 | 返す値 | プロセス終了コード |
|---|---|---|---|
| exit 0 | Success | 0 | 0 |
| exit 3 / 4 | Warning | 0 | 0 |
| exit 1 / 2 / 255 | Error | 1 / 2 / 255 | 同じ |
| exit -1(Windows 等) | Error | -1 | 255 |
| 起動失敗 / command 不正 | Error | 127 | 127 |
| SIGKILL(9)で終了 | Error | 137 | 137 |
| 実行中に SIGTERM(15)を受信(ジョブが exit 0 でも) | Error | 143 | 143 |
| 起動前に SIGINT(2)が立っていた | 分類なし | 130 | 130 |

`success_exit_codes = [2]` のとき: exit 2 → Success(0)、exit 0 → Error(1, `ConductorError`)。`success_exit_codes = [0, 2]` のとき exit 0 も Success。

標準エラーへの常時の表示(`-v` なしでも):

- `ConductorError = true`: `conductor: error: job '<id>': exit code 0 is not listed in success_exit_codes`(日本語ロケールでは日本語。文言は [08-output-and-i18n.md](08-output-and-i18n.md))。`ConductorError = true` なら、順 5(終了コードが無い)の場合も同じ文言を出す(実装は `ConductorError` が真なら、理由を区別せず同じ行を出す)。
- 起動失敗: `conductor: error: job '<id>': cannot start command '<program>'`。

---

## 3. 遷移と次の動き(`ClassifyAndDecide` / `NextStep`)

ジョブ実行後に、分類と次の動き(`NextStep`)を決める。

### 3.1 `NextStep` のフィールド

| フィールド | 意味 |
|---|---|
| `NextJobId` | 次に実行するジョブ。完了・停止のとき `null`(`pipeline` コマンドは、この値を使わない。§5.3) |
| `SkippedJobIds` | この時点でスキップとして記録するジョブ |
| `SkipReasonKind` | その理由(`ByTransition` / `ByEnd` / `null`) |
| `EndReached` | `@end` に到達した |
| `Completed` | 完了した |
| `ResumeJobId` | エラー等で停止したときの再開位置(そのジョブ)。`null` もある |

### 3.2 判定表

入力: 分類、そのジョブの該当する遷移(`on = success` は成功のとき、`on = warning` は警告のときに参照する。`error` の遷移は書けない)、`RunMode`、そのジョブの位置(index。定義の記述順、0 始まり)、各ジョブの現在の状態。

「未完了のジョブ」= 指定した区間のジョブのうち、状態が success / warning / skipped **でない**もの(pending・error・記録なし)。区間は、`@end` なら「index より後ろ全部」、遷移先 `T` なら「index と T のあいだ(両端を含まない)」。

| 分類 | 遷移 | RunMode | NextJobId | SkippedJobIds / kind | EndReached | Completed | ResumeJobId |
|---|---|---|---|---|---|---|---|
| Error | (見ない) | 両方 | null | 空 / null | false | false | **このジョブ** |
| 分類なし(起動しなかった中断) | (見ない) | 両方 | null | 空 / null | false | false | **このジョブ** |
| Success / Warning | `@end` | Pipeline | null | 後ろの未完了ジョブ全部 / `ByEnd`(空なら kind は null) | true | **true** | null |
| Success / Warning | `@end` | Individual | null | **空** / kind = `ByEnd`(後ろに未完了が 1 つでもあるとき。無ければ kind は null) | true | 後ろに未完了が無ければ true、あれば false | null |
| Success / Warning | ジョブ `T`(index より後ろ) | 両方 | `T` | index と T の間の未完了ジョブ / `ByTransition`(空なら kind は null) | false | false | null |
| Success / Warning | 遷移なし(または遷移先が後ろのジョブでない。検証済み定義では起きない) | 両方 | 最後のジョブなら null、そうでなければ次のジョブ | 空 / null | false | 最後のジョブなら true、そうでなければ false | null |
| 定義にないジョブ | - | - | Error と同じ(停止) | | | | |

要点:

- 成功・警告のあと再開位置は無し。エラー・分類なしのあとは必ずそのジョブ。
- Individual の `@end` では、後ろのジョブを今は記録せず(`SkippedJobIds` は空)、`EndReached = true` だけを記録する。後ろのジョブは、スケジューラが 1 つずつ `run` を呼んだときに `Skip(AfterEnd)` が返り、呼ばれたジョブだけ `skipped(byEnd)` と記録される(§4.2、§5.4)。
- 遷移先 `T` の遷移では、`T` へ飛ぶ。`T` が index+1 なら間は空で、スキップは無い。
- 自動のリトライ(ジョブ失敗の再実行)は、ジョブに `retry` を書かない限りしない(§7)。

### 3.3 `ApplyOutcome`(結果の記録。メモリ上)

入力: 現在のステート値 `S`、ジョブID、実行結果、結果(`JobOutcome`)、`NextStep`。`S` を複製して更新し、返す。

1. 対象ジョブ(`S` のジョブ状態一覧から id で探す。見つからなければこの手順 1 を飛ばす):
   - `Classification` があれば `Status` を Success / Warning / Error にし、`SkipReason` を消す。分類なし(起動前の中断)なら `Status` は**変えない**。
   - `ExitCode` = 実行結果の `ExitCode`(無ければ、以前の値も消す)。そのあと `Status` が Success / Warning / Error のいずれでもなければ `ExitCode` を消す。
2. `SkippedJobIds` の各ジョブ(対象ジョブ自身は除く)を `Status = Skipped`、`ExitCode` 無し、`SkipReason = SkipReasonKind`(`null` なら `ByTransition`)にする。
3. `ResumeJobId = NextStep.ResumeJobId`、`Completed = NextStep.Completed`。
4. `EndReached = S.EndReached || NextStep.EndReached`(一度 true になったら戻さない)。

### 3.4 `ApplySkip`(スキップの記録。メモリ上)

入力: ステート値 `S`、ジョブID、判断の理由(`DecisionReason`: `AlreadyExecuted` / `SkipTarget` / `AfterEnd` / `OutOfOrder`)、定義。

- 理由が `AlreadyExecuted` / `OutOfOrder` → 何も変えない(複製を返す)。
- `SkipTarget` → そのジョブを `Skipped(ByTransition)`、`AfterEnd` → `Skipped(ByEnd)` にし(`ExitCode` 無し)、`AfterEnd` なら `EndReached = true`。ジョブが `S` に無ければ、更新せず複製を返す。
- そのあと、`S` に `Error` のジョブが 1 つでもあれば、`ResumeJobId` と `Completed` は**変えない**(エラーの再開位置を守る)。無ければ:
  - `firstPending` = 定義の記述順で最初の `Pending`(記録なしを含む)のジョブ(無ければ `null`)
  - `Completed` = 「いまスキップしたジョブが定義の最後のジョブ、かつ `firstPending` が `null`」
  - `ResumeJobId = firstPending`

### 3.5 `ApplyInterrupt`(中断の記録。メモリ上)

再開位置(§9)の値で更新する: `ResumeJobId = ResumePoint.ResumeJobId`、`Completed = ResumePoint.Completed`、`EndReached = S.EndReached || ResumePoint.EndReached`。ジョブの状態は変えない(エラーとして記録しない)。

---

## 4. 起動前の判断(`DecideBeforeRun`)

### 4.1 入力と出力

入力: ジョブID、定義、各ジョブの状態の一覧、`ResumeJobId`(ステートの同名の値)、`EndReached`。ステートが空(ファイルなし・長さ 0・完了のまま残ったもの)のときは、状態一覧は空、`ResumeJobId = null`、`EndReached = false` を渡す。

出力: `Run`(実行する)/ `Skip(理由)`(実行せず 0 で終わる。理由は `SkipTarget` / `AfterEnd` / `AlreadyExecuted`)/ `Reject(OutOfOrder)`(実行せず、ステートも変えず、66)。

### 4.2 判定表(上から最初に当てはまるものを採る)

状態 `st` = そのジョブの記録(無ければ Pending)、`reason` = そのスキップ理由。

| 順 | 条件 | 結果 |
|---|---|---|
| 0 | ジョブIDが定義にない(`run` では、事前に 68 で止まるので到達しない) | `Reject(OutOfOrder)` |
| 1 | `st = Skipped` かつ `reason = ByEnd` | `Skip(AfterEnd)` |
| 1' | `st = Skipped` かつ `reason` が ByTransition または `null` | `Skip(SkipTarget)` |
| 2 | `EndReached` かつ `st = Pending` | `Skip(AfterEnd)` |
| 3 | `st` が Success または Warning | `Skip(AlreadyExecuted)` |
| 4 | `st = Error`、または `ResumeJobId` がこのジョブID | `Run` |
| 5 | 全ジョブが Pending(または記録なし)で、このジョブが記述順で最初 | `Run` |
| 6 | `st = Pending` で、記述順で前のジョブがすべて Success / Warning / Skipped | `Run` |
| 7 | 上のどれでもない | `Reject(OutOfOrder)` |

例(4 ジョブ j0〜j3):

| 呼ぶジョブ | 状態 | resume | endReached | 結果 |
|---|---|---|---|---|
| j1 | j0=success, j1=skipped(byTransition) | - | false | Skip(SkipTarget) |
| j2 | j2=skipped(byEnd) | - | true | Skip(AfterEnd) |
| j3 | j0=success(j3 記録なし) | - | true | Skip(AfterEnd) |
| j0 | j0=success | - | false | Skip(AlreadyExecuted) |
| j0 | j0=warning | - | false | Skip(AlreadyExecuted) |
| j1 | j0=success, j1=error | j1 | false | Run |
| j1 | j1=error | - | false | Run |
| j2 | j0=success, j1=skipped | j2 | false | Run |
| j0 | (空) | - | false | Run |
| j0 | 全部 pending | - | false | Run |
| j2 | j0=success, j1=warning | - | false | Run |
| j3 | j0=success, j1=skipped, j2=skipped | - | false | Run |
| j1 | (空) | - | false | Reject |
| j2 | j0=success | - | false | Reject |
| j2 | j0=success, j1=error | - | false | Reject |

補足: 規則 1' は、理由が書かれていない `Skipped`(ステートファイル上は読み込みで弾かれるが、関数としては入りうる)を SkipTarget とする。規則 2 は `Pending`(記録なしを含む)のときだけ。`Error` のジョブは規則 4 で、`EndReached` でも `Run` になる。規則 6 は「記述順で前のすべて」を見るので、後ろのジョブが済んでいても影響しない。

### 4.3 判断の結果とステート・終了コード(呼び出し側)

| 結果 | ステートの変更 | 標準エラー | 終了コード |
|---|---|---|---|
| `Reject(r)` | しない(初期ステートも書かない) | `conductor: error: job '<id>': cannot run this job now: earlier jobs have not finished (out of order)`(日本語: `このジョブは、いま実行できません: 前のジョブが終わっていません(順序外)`) | 66 |
| `Skip(r)` | `ApplySkip` の結果が元と**等しくなければ**(§0.2 の `Equals`)、耐久性ありで書く。`AlreadyExecuted` は常に等しい(書かない) | `-v` のとき `[conductor] job=<id> skipped (<理由>)`。理由文: SkipTarget = `skipped by a transition`、AfterEnd = `skipped after @end`、AlreadyExecuted = `already executed` | 0(pipeline では続行) |
| `Run` | ステートが空なら先に初期ステートを、耐久性を省いて書く(§5) | - | ジョブの結果による |

「等しい」はステート値の全項目の比較。「耐久性あり(Durable)」「省く(Fast)」の定義は [06-state-file.md](06-state-file.md) の書き込み。

---

## 5. 実行の流れ

### 5.1 共通の前処理 `Prepare`(`run`・`pipeline`)

順序は固定。最初に誤りがあった時点で、その終了コードで終わる(以降は行わない)。

| 順 | 処理 | 誤りのときの終了コード |
|---|---|---|
| 1 | 中断の合図(「立った/立っていない」と最初のシグナル番号を保持する共有の旗 `Interrupt`。§15.1)を作り、シグナルのハンドラを登録(定義の検査より前)。登録は `PosixSignalRegistration.Create`(UNIX 系)と `Console.CancelKeyPress`(Windows。詳細は [07](07-process-and-signals.md))。登録のオブジェクトは、プロセスの終了まで参照を保持する(GC されると登録が外れうる。07) | 準備の失敗(登録が例外を出した): 70(`conductor: error: failed to set up signal handling`) |
| 2 | 定義ファイルの読み込みと検証。`pipeline` は、引数の名前と定義の `pipeline.name` を照合 | 64 |
| 3 | 全ジョブの `command` の検査(引用符の閉じ忘れ・空) | 64 |
| 4 | `run` のとき、ジョブIDが定義にあるか | 68(`unknown job`) |
| 5 | run ID の決定(§6) | 68 |
| 6 | ステートの読み込み([06-state-file.md](06-state-file.md) §6)。完了のまま残ったものは「空」 | 65 |
| 7 | 空でなければ、定義との照合(06 §7) | 65 |
| 8 | 実効の `remove_on_complete` = `--remove-on-complete` があれば true、なければ定義の `pipeline.remove_on_complete`(省略時 false)。環境変数は見ない | - |

- 定義ファイルは `--config <パス>`、なければ実行ディレクトリの `conductor.json`。
- シグナルを受け取っても、1〜7 は最後まで進む(ハンドラは旗を立てるだけで、処理を中断しない)。64・65・66・スキップ(0)の終了コードが、中断の終了コードに優先する。
- `pipeline` で引数の名前が定義の名前と違う場合は 64(順 2)。引数の名前が無いのは使い方の誤り 68(コマンドライン解析。[04-cli.md](04-cli.md))。

### 5.2 `conductor run <job-id>` の擬似アルゴリズム

```text
session = Prepare(...)                      # 5.1。誤りなら終了
(states, resume, endReached) = session.State の Jobs / ResumeJobId / EndReached
                               (State が null(空)なら [], null, false)
d = DecideBeforeRun(jobId, def, states, resume, endReached)

match d:
  Reject(r):  表示、return 66
  Skip(r):
    base = session.State か InitialState(...)(空なら)
    rec  = ApplySkip(base, jobId, r, def)
    if !rec.Equals(base):
        Write(rec, Durable)                  # 失敗: 65、文脈=ジョブ起動前
        if rec.Completed and removeOnComplete:
            ResetState()                     # 失敗: 65
    -v なら skipped 表示
    return 0
  Run:
    base = session.State か、(空なら) InitialState を Fast で書いたもの   # 書けなければ 65、ジョブは起動しない
    exec = RunJobWithRetry(job)              # §7。ステートは書かない
    (outcome, next) = ClassifyAndDecide(job, exec, base.Jobs, def, Individual)
    表示(§5.5)
    rec = ApplyOutcome(base, jobId, exec, outcome, next)
    Write(rec, Durable)                      # 失敗: 65、文脈=exec.Started なら「ジョブは実行されました」、そうでなければ「起動していません」
    if next.Completed and removeOnComplete:
        ResetState()                         # 失敗: 65
    return ExitCodes.For(outcome.ReportedExitCode, outcome.Classification == Error)
```

注意:

- Individual では、遷移でスキップが決まったジョブ(`ByTransition`)は、`ApplyOutcome` で記録済みなので、後で `run` されると `Skip(SkipTarget)`。
- 最後のジョブの `run` 完了、または `@end` 後の最後のジョブのスキップ記録で `Completed = true`。
- 初期ステートは `Run` と決まった後にだけ書く(Reject・Skip の前には書かない)。ただし Skip のときは、ステートが空なら `InitialState`(06 §2.4)を土台に記録が書かれる。
- `ClassifyAndDecide(job, exec, states, def, mode)` は、`Classify`(§2.3)で結果を作り、その分類を使って `NextStep`(§3.2)で次の動きを決めて、組で返す。

### 5.3 `conductor pipeline <name>` の擬似アルゴリズム

`pipeline` は、「次のジョブ」を `NextStep.NextJobId` で決めない。定義の記述順に全ジョブを 1 つずつ見て、`DecideBeforeRun` で決める(遷移先・スキップは、ステートが運ぶ)。

```text
session = Prepare(...)                       # 引数の名前が無ければ 68
current = session.State                      # 空なら null
for job in def.Jobs (記述順):
  d = DecideBeforeRun(job.Id, def, current の状態一覧 (null なら []), current?.ResumeJobId, current?.EndReached ?? false)
  match d:
    Reject(r): 表示、return 66
    Skip(r):
      base = current か InitialState(...)
      rec  = ApplySkip(base, job.Id, r, def)
      if !rec.Equals(base): Write(rec, Durable)    # 失敗: 65
      -v なら skipped 表示
      current = rec
      if rec.Completed: return Finish(...)         # 下記
    Run:
      base = current、null なら InitialState を Fast で書いたもの   # 失敗: 65
      exec = RunJobWithRetry(job)
      (outcome, next) = ClassifyAndDecide(job, exec, base.Jobs, def, Pipeline)
      表示
      rec = ApplyOutcome(base, job.Id, exec, outcome, next)
      Write(rec, Durable)                          # 失敗: 65
      if outcome は Error、または分類なし:
          return ExitCodes.For(outcome.ReportedExitCode, outcome.Classification == Error)   # エラーで停止
      if next.Completed: return Finish(...)
      if 中断の合図が立っている(番号 n):            # ジョブのあいだの中断
          point = Resume.OnInterrupt(def, rec.Jobs)
          Write(ApplyInterrupt(rec, point), Durable)        # 失敗: 65
          -v なら「interrupted by signal n」
          return ExitCodes.SignalExitCode(n)             # For(AddI32(128, n), true)
      current = rec
# ループを抜けた(完了していない。起こらない想定)
表示 "cannot determine whether the pipeline is complete"; return 65   # ステートは初期化しない

Finish: if !removeOnComplete: return 0
        ResetState() が成功 → 0、失敗 → 65
```

- 完了のステートは、`Finish` の前に必ず Durable で書き込み済み。`remove_on_complete` が真のときだけ削除する(06 §9)。
- 「完了を判定できない」は、全ジョブが `AlreadyExecuted` などで通過し、どこでも `Completed` が真にならなかったとき。標準エラー `conductor: error: <ステートファイルのパス>: cannot determine whether the pipeline is complete`(日本語: `パイプラインが完了したかを判定できません`)、終了コード 65。
- ジョブ実行後、`next.Completed` の判定は、中断の合図の判定より**先**。完了していれば中断の記録はせず、`Finish` へ進む。
- エラーのジョブで止まったときの再実行: 同じコマンドを再度呼ぶと、`Success`/`Warning` のジョブは `Skip(AlreadyExecuted)`(`-v` に `already executed`)、`Error` のジョブから `Run`。
- `Skip` で `rec.Completed` が真になるのは §3.4 の条件(スキップしたジョブが最後のジョブで、pending が無い)のとき。

#### 状態遷移の例(`pipeline`)

定義 `a → b → c`(遷移なし)。`b` が exit 3 でエラー。

| 時点 | job_states | resume_job_id | end_reached | completed | 終了コード |
|---|---|---|---|---|---|
| 初期(Fast で書く) | a=pending, b=pending, c=pending | なし | false | false | - |
| a 成功後 | a=success(0), b=pending, c=pending | なし | false | false | - |
| b エラー後 | a=success(0), b=error(3), c=pending | b | false | false | 3 |
| 再実行: a は Skip(AlreadyExecuted)、b 成功 | a=success, b=success(0), c=pending | なし | false | false | - |
| c 成功後 | 全部 success | なし | false | true | 0 |

定義 `a --success--> @end` のとき(Pipeline): a 成功で b, c は `skipped(byEnd)`、`end_reached = true`、`completed = true`、0 で終わる([06-state-file.md](06-state-file.md) §4.4 の例 2 のステート)。

### 5.4 `@end` 後の `run`(Individual)の流れ

`run a`(遷移 `@end`)→ `a=success`、`end_reached = true`、`completed = false`(後ろに未完了がある場合)。続けて `run b` → 規則 2 で `Skip(AfterEnd)` → `b=skipped(byEnd)`、`resume_job_id` = 最初の pending(`c`)。`run c` → `Skip(AfterEnd)` → `c=skipped(byEnd)`、最後のジョブで pending が無いので `completed = true`(`remove_on_complete` なら削除)。`@end` の後ろに未完了が無ければ、`run a` の時点で `completed = true`。

### 5.5 表示(`-v` と常時)

- 常に標準エラー: エラー行(`conductor: error: ...`)、起動失敗、成功コードに無い 0、リトライの知らせ(§7.9)、`reset --all` の警告。
- `-v` のときだけ標準エラー(先頭 `[conductor] `): `job=<id> result=<success|warning|error|interrupted> (...)`、`transition: <success|warning> -> <target>[ (skipped: <id>, ...)]`、`job=<id> skipped (<理由>)`、`interrupted by signal <n>`。括弧内の詳細の厳密な文言は [08-output-and-i18n.md](08-output-and-i18n.md) に従う。
- ジョブの結果の行と遷移の行は、**最終の結果について 1 回**だけ(リトライの途中の結果は出さない)。
- 言語: 環境変数 `LC_ALL`、`LC_MESSAGES`、`LANG` の順で、最初の空でない値が `ja` で始まれば日本語、それ以外は英語(どれも空・未設定なら英語)。トークン(`success` `warning` `error` `skipped` `pending` `job=` `result=`)は常に英語。判定の `ja` で始まるかは、序数の `StartsWith("ja", StringComparison.Ordinal)`(文化依存の `StartsWith(string)` を使わない)。

---

## 6. run ID

run ID は、同じ定義を別々の実行単位で並行して使うときに、ステートファイルを分ける識別子。

### 6.1 値の規則(`RunIdRules.Parse`)

次の順に検査し、最初に当てはまる誤りを返す(誤りは使い方の誤り 68。誤りの文に値は含めない)。

| 順 | 誤り | 条件 |
|---|---|---|
| 1 | `Empty` | 長さ 0 |
| 2 | `TooLong` | UTF-8 で符号化したバイト数が 64 を超える |
| 3 | `InvalidChar` | `A-Za-z0-9._-` 以外の文字がある |
| 4 | `LeadingDot` | 先頭が `.`(`..` を含む) |

C# での決定と落とし穴:

- **`Regex` を使わず、手書きの走査にする。** 理由: `Regex` の `$` は、文字列の末尾の `\n` の手前にも一致する(一般に知られた .NET の挙動。この文書では公式ドキュメントを取得して確かめていない: 未確認)ので、`^[A-Za-z0-9._-]+$` は `"a\n"` を受理してしまう。`\d`・`\w` は既定で Unicode の数字・文字(全角の `１` など)に一致する(`RegexOptions.ECMAScript` で ASCII に限れるが、組み合わせられるオプションに制限がある。未確認)。`RegexOptions.IgnoreCase` は文化の大文字小文字の規則(ケルビン記号 U+212A が `k` に一致しうる。未確認)に関わる。`Regex` は、トリミング・起動時間にも負担になる。1 文字ずつ ASCII の範囲を比べる手書きの走査なら、これらを全部避けられる。
- 順 3 の判定は、`char` ごとに次の式で行う(`char.IsLetterOrDigit`・`char.IsDigit`・`char.IsAsciiLetterOrDigit` に頼らない。前 2 つは Unicode の文字・数字に真を返す。`IsAscii*` は .NET 7 以降の API で、最小サポートの版にあるかを確かめていない: 未確認。範囲比較を自分で書けば版に依らない):

```csharp
static bool IsAllowed(char c) =>
    (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9')
    || c == '.' || c == '_' || c == '-';
```

- 順 2 のバイト数は、`string` を UTF-16 のコードユニット(`char`)の列として走査して数える(正本)。`Encoding.UTF8.GetByteCount(value)` は同じ値になる(同じ規則)が、実装を 1 つに固定するため、走査の規則を規定する:

| 走査する `char` | 加えるバイト数 |
|---|---|
| U+0000〜U+007F | 1 |
| U+0080〜U+07FF | 2 |
| 高サロゲート(U+D800〜U+DBFF)の直後に低サロゲート(U+DC00〜U+DFFF)が続く対 | 4(2 つの `char` で 1 回。対で数える) |
| 対にならない孤立サロゲート(U+D800〜U+DFFF の 1 つ) | 3(U+FFFD に置き換わるとみなす) |
| それ以外(U+0800〜U+FFFF のサロゲート以外) | 3 |

  公式ドキュメントで確認できたこと(出典: `UTF8Encoding(Boolean, Boolean)` の例): 既定の `UTF8Encoding`(例外を投げない設定)は、不正なサロゲート(高サロゲートが 2 つ続く `U+D801 U+D802`)を `EF BF BD EF BF BD`(U+FFFD の 3 バイトを 2 つ)に符号化する。孤立サロゲート 1 つが 3 バイトになることは、この例から読み取れる。`Encoding.UTF8` の `GetByteCount` が同じ数を返すこと(`GetBytes` と整合すること)は、ドキュメントを取得して確認していない(未確認)ので、走査を正本とする。
  走査の途中で合計が 64 を超えたら、その時点で `TooLong`(全体を数え切る必要はない。桁あふれもしない)。
- `string` の `Length` は UTF-16 のコードユニット数で、コードポイントの数でも書記素の数でもない。`"あ"` は 1 つの `char` で 3 バイトなので、`new string('あ', 30)` は `Length` が 30 だが 90 バイトで `TooLong` になる。`Length` で順 2 を判定すると、順 2 と順 3 の判定が入れ替わる(`InvalidChar` になる)。絵文字(U+1F600)は 2 つの `char`(サロゲートの対)で 4 バイト。許す文字は ASCII だけなので、検査を通る値では文字数とバイト数は同じ。書記素(`StringInfo`)・`Rune` は使わない(順 2・3 はコードユニットの走査で足りる。`Rune` に分解して孤立サロゲートの扱いを `Rune` の API に任せない)。
- Unix で、引数・環境変数に不正な UTF-8 のバイト列が入っていたとき、.NET がそれをどの文字列にするか(U+FFFD に置き換える、など)は、公式ドキュメントで確認していない(未確認)。どの場合でも、許さない文字を含む(または長すぎる)ので、利用者に見える結果は 68 で変わらない(誤りの文に値と種類は含めないため)。
- 順 4 の「先頭が `.`」は、`value[0] == '.'`(`StartsWith(".")` は文化依存の比較になりうるので使わない)。
- 空は順 1 で除かれている。

有効: `1`, `r1`, `12345`, `build-2026.11_a`, `a.b`, 64 文字ちょうど。無効: `""`(ただし §6.2 の規則で空は「なし」)、65 文字、`a/b`、`a b`、`.x`、`..`、`日本語`(30 文字(90 バイト)なら `TooLong`、21 文字(63 バイト)なら `InvalidChar`)、`a\n`(改行は許す文字に無いので `InvalidChar`)、全角の `１`(U+FF11)。

境界の例(`TooLong` との関係):

| 値 | バイト数 | 結果 |
|---|---|---|
| `a` × 64 | 64 | 有効 |
| `a` × 65 | 65 | `TooLong` |
| `あ` × 21 | 63 | `InvalidChar` |
| `あ` × 22 | 66 | `TooLong` |
| `a` × 62 + `é` | 62 + 2 = 64 | `InvalidChar`(バイト数は 64 以下) |
| `a` × 63 + `é` | 65 | `TooLong` |
| U+D800(孤立) 1 つ | 3 | `InvalidChar` |

### 6.2 決定(`RunIdRules.Resolve`、`Prepare` の 5)

```text
envName = 定義の pipeline.run_id_env があればそれ、なければ "CONDUCTOR_RUN_ID"
if --run-id が指定された(値は空でもよい):
    value = そのオプションの値(subject = "--run-id")      # 環境変数は見ない
else:
    value = lookupEnv(envName)(本番は Environment.GetEnvironmentVariable。未定義なら null)(subject = "run ID (environment variable)")
if value が null または "": run ID なし
else: Parse(value) が失敗なら 68(`invalid value: '<subject>'`)、成功なら run ID
```

- `--run-id ""` は、環境変数があっても「run ID なし」に固定する。
- `run_id_env` の形は定義の検証で保証される(`A-Za-z_` で始まり `A-Za-z0-9_` が続く 1 文字以上)。
- 環境変数名は定義で決まるので、定義の検査(64)の**後**、ステートの読み込み(65)の**前**に run ID を決める。
- `watch`・`reset` は定義を検証しない: `--run-id` があれば定義を読まず(環境変数名は既定)、なければ定義の `pipeline.run_id_env` だけをゆるく読む。読めない・キーが無い・文字列でない・名前の形が違う、のどれでも「なし」= 既定の名前を使う。ほかの誤りは無視する。異常終了してはならない(定義の読み取りが投げた例外は、この場面では握りつぶして既定の名前にする)。
- 環境変数の値が規則に合わなければ、`run`・`pipeline`・`watch`・`reset`(`--all` を除く)で 68。`reset --all` は環境変数を見ない。
- 環境変数は `Environment.GetEnvironmentVariable(string)` で読む(未定義は `null`、空文字列で定義されていれば `""`。この区別が「環境変数が空文字列で定義されている場合も『なし』として扱う」規定と合う。`GetEnvironmentVariable` が未定義で `null` を返すことは、このメソッドの契約として知られているが、この文書では公式ドキュメントを取得して確認していない: 未確認)。変数名の大文字小文字は、Unix では区別され、Windows では区別されない(Windows の環境変数の一般の性質。.NET の API としてどうなるかは未確認)。そのため、`run_id_env` の大文字小文字の違いは、Windows では同じ変数を指すことがある。環境の読み取りは、差し替え可能な関数 `Func<string, string?> lookupEnv`(`IEnv.LookupEnv`。[02](02-architecture.md))を通し、`RunIdRules.Resolve(flagValue, envName, lookupEnv)` に渡す。`Conductor.Flow` の純粋な関数からは読まない。

### 6.3 run ID とステートの関係

- run ID あり: ステートは `<定義ファイル名>.<run-id>.state`。中身に `run_id` = run ID を書く。run ID なし: `<定義ファイル名>.state`、`run_id` の項目無し。
- 読み込み時、中身の `run_id`(あるか、値)がファイル名側の run ID と一致しなければ壊れたステート(65)。詳細は 06 §6。
- run ID は同じ定義でも run ごとに独立したステート・再開位置を持つ。同じ run ID で呼び直すと、止まったジョブから再開する。

---

## 7. リトライ(`retry`、v0.8.0)

### 7.1 設定(検証済みの値。`RetryPolicy`)

ジョブ定義の `retry` オブジェクト([03](03-definition-format.md))。

| キー | 型・範囲 | 意味 |
|---|---|---|
| `exit_codes` | 整数の一覧。空でない。各値は 1〜255。昇順・重複なしに正規化済み | リトライの対象にするジョブ自身の終了コード。このジョブの成功コード・警告コードと重ならない |
| `max` | 整数 1〜100 | リトライの最大回数(最初の実行を除く)。実行は合計で最大 `max + 1` 回 |
| `interval` | 文字列 `<整数><s|m|h>`。結果が 1 秒〜24 時間(86400 秒) | 1 回目のリトライの前の待ち時間。数字は半角の数字だけで、整数は 1 以上。単位は小文字 1 文字(`s` 秒、`m` 分 = 60 秒、`h` 時間 = 3600 秒)。例 `"30s"`、`"2m"`、`"1h"` |
| `multiplier` | 整数 2〜10 | 待ち時間にかける倍率。`max_interval` と同時に指定 |
| `max_interval` | `interval` と同じ書式。`interval` 以上 | 待ち時間の上限。`multiplier` と同時に指定 |

- `multiplier` と `max_interval` は両方書くか両方書かない。検証の詳細は [03-definition-format.md](03-definition-format.md)。
- `retry` を書かないジョブは、1 回だけ実行する。

### 7.2 数値の型と骨格

§0.2 の方針を、リトライに当てはめる。待ち時間は秒の `ulong`、倍率は `uint`。浮動小数点(`double`・`float`)・`decimal`・`TimeSpan`・`BigInteger` は、待ち時間の計算に使わない(丸めの余地を作らない)。

```csharp
// Backoff と RetryPolicy は 03 章の型(名前空間 Conductor.Definition。この章は再定義しない)
public sealed record Backoff(int Multiplier, ulong MaxIntervalSecs);   // 2〜10、IntervalSecs 以上 86400 以下
public sealed record RetryPolicy(
    IReadOnlyList<int> ExitCodes,    // 昇順・重複なし・各 1〜255・空でない
    int Max,                         // 1〜100
    ulong IntervalSecs,              // 1〜86400
    Backoff? Backoff);               // null なら固定間隔
```

`Backoff` を 1 つの値にまとめるのは、「`multiplier` と `max_interval` は両方あるか両方ない」を型で保証するための決定。定義の検証で片方だけは拒否される([03](03-definition-format.md))ので、部分的に存在する状態は作れない。`Backoff` は `sealed record`(参照型)で、`Backoff?` は null 許容の参照。

### 7.3 `interval` / `max_interval` の文字列から秒への変換

文字列 → 秒(`DurationParser.TryParseSecs(string s, out ulong secs)`。定義の検証で使う。置き場所は `Conductor.Definition`。ここでは落とし穴を示すために再掲する):

```text
1. s の長さ(char の数)が 2 未満なら 拒否
2. 最後の 1 文字が 's' / 'm' / 'h' のどれでもなければ 拒否
3. 残り(digits)の全ての char が '0'〜'9'(範囲比較)でなければ 拒否
   (符号・空白・小数点・'_'・全角数字・"0x" は、ここで弾かれる)
4. digits を 1 文字ずつ走査して n(ulong)を作る。各桁で n = n * 10 + 桁。
   単位ごとの上限(下の 5)を超えた時点で拒否する(走査を続けない)。桁あふれは、上限の方が十分小さいので起きない
5. 単位ごとの上限: 's': 1 <= n <= 86400,  'm': 1 <= n <= 1440,  'h': 1 <= n <= 24。外れたら拒否
6. secs = n * (1 / 60 / 3600)   # 単位ごと。ulong の乗算(上限内なので飽和しない)
```

C# の落とし穴:

- **`int.Parse` / `uint.Parse` / `ulong.Parse` / `Convert.ToInt32` は使わない**: 既定で先頭・末尾の空白、先頭の符号(`+`・`-`)を許し、`NumberStyles` と `IFormatProvider`(文化)で受理する形が変わる(`NumberStyles` の既定の許容は、公式ドキュメントを取得して確認していない: 未確認)。`"+1s"`・`" 1s"` の受理を避けるため、桁の判定は手順 3 の範囲比較にする。`char.IsDigit` は Unicode の数字(全角の `１`、アラビア・インドの数字)に真を返すので使わない(一般に知られた挙動。取得して確認していない: 未確認)。
- `int.Parse` は桁数が多いと `OverflowException` を投げる。この手順は自前の走査で、上限を超えた時点で拒否するので、例外は出ない。
- 先頭の `0` は読み飛ばす必要が無い(走査で `n` に 0 が積まれるだけ)。`"0000000000000000000000000001s"` は 1。

例: `"1s"`→1、`"01s"`→1、`"0000000000000000000000000001s"`→1、`"1440m"`→86400、`"24h"`→86400、`"86400s"`→86400。拒否: `""`、`"s"`、`"10"`、`"1.5s"`、`"-1s"`、`"+1s"`、`" 1s"`、`"1s "`、`"1S"`、`"１s"`(全角 1)、`"1ms"`、`"0x10s"`、`"1_0s"`、`"0s"`、`"0h"`、`"86401s"`、`"25h"`、`"99999999999999999999999s"`、`"18446744073709551615h"`、`"1s\n"`。

### 7.4 リトライの対象か(`IsRetryTarget`)

次の**すべて**が成り立つときだけ真。

1. `Started = true`
2. `Interrupted = false`
3. `StartFailure = false`
4. `TerminatedBySignal` が無い(`null`)
5. `ExitCode` があり、`ExitCodes` に含まれる

分類(成功・警告・エラー)は見ない。対象の終了コードは成功・警告と重ならない(検証済み)ので、結果として対象はエラーの終了コードだけ。ジョブ自身が 127 で終わった場合は、`ExitCodes` に 127 を書いていれば対象(conductor が付けた起動失敗の 127 は、`ExitCode` を持たないので対象にならない)。

### 7.5 判断(`Retries.Decide(job, execution, retriesDone, lastWaitSecs)`)

```text
policy = job.Retry か、無ければ return Stop
if !IsRetryTarget(policy, execution) or retriesDone >= policy.Max: return Stop
return Retry { WaitSecs: NextWaitSecs(policy, retriesDone, lastWaitSecs) }
```

### 7.6 待ち時間の式と飽和演算

```text
NextWaitSecs(policy, retriesDone: int, lastWaitSecs: ulong) -> ulong
  if policy.Backoff が有る:
      if retriesDone == 0: return min(policy.IntervalSecs, policy.Backoff.MaxIntervalSecs)   # = IntervalSecs(検証で interval <= max_interval)
      else:                return min(Saturating.MulU64(lastWaitSecs, (ulong)policy.Backoff.Multiplier), policy.Backoff.MaxIntervalSecs)
  else:
      return policy.IntervalSecs                                                              # 毎回同じ(固定間隔)
```

- k 回目のリトライ(k = `retriesDone + 1`)の前の待ち時間を `W(k)` とすると: 固定間隔なら `W(k) = interval`。倍率ありなら `W(1) = interval`、`W(k) = min(W(k-1) × multiplier, max_interval)`(k ≥ 2)。
- 倍率の掛け算は、直前の**実際の**待ち時間(上限で頭打ちされた後の値)に対して行う。
- `min` は `Math.Min(ulong, ulong)`(`ulong` 同士)。`long` や `int` にキャストして比べない(2^63 以上で負になる)。

`Saturating.MulU64(a, b)`(64 ビット符号なし整数の飽和乗算):

```csharp
public static ulong MulU64(ulong a, ulong b)
{
    ulong high = Math.BigMul(a, b, out ulong low);    // 128 ビットの積の上位 64 ビットが戻り値、下位が low
    return high != 0 ? ulong.MaxValue : low;          // 上位が 0 でなければ積は 2^64 以上 → 飽和
}
```

確認(出典: `Math.BigMul` の API ページ): `public static ulong BigMul(ulong a, ulong b, out ulong low)` は「2 つの符号なし 64 ビットの完全な積」を作り、戻り値は積の**上位 64 ビット**、`low` は**下位 64 ビット**。.NET 5 以降の版で使える(ページの対象の版に net-5.0 〜 net-11.0 が列挙されている。.NET Core 3.1 以前の版には列挙されていない)。CLS 準拠でない(ページの注記)ので、公開 API の署名に出さず、内部の実装で使う。別の過負荷 `Math.BigMul(ulong, ulong)`(戻り値 `UInt128`)は net-9.0 以降(ページの版の列挙による)で、最小サポートの版では使えない可能性があるので使わない。

- **折り返し(`unchecked(a * b)` の結果をそのまま使う)にしてはならない**。`ulong` の通常の乗算は、`unchecked` では黙って 2^64 を法とする剰余になる(例: `(2^64-1) × 10` は 18446744073709551606)。`checked` では `OverflowException` になる。どちらでもなく、飽和が規定。
- 前提: `a` は `ulong` の任意の値でよい(`BigMul` は全範囲で正しい)。`b` は `(ulong)uint`。

境界の例(関数単体の挙動。別の言語の任意精度整数で計算した値で、C# での実行結果ではない。表の `high` / `low` は `Math.BigMul` の戻り値と `low`):

| a | b | high | low | 結果 | 理由 |
|---|---|---|---|---|---|
| 0 | 10 | 0 | 0 | 0 | |
| 86400 | 10 | 0 | 864000 | 864000 | 検証済みの定義で起こりうる最大の積 |
| 1844674407370955161 | 10 | 0 | 18446744073709551610 | 18446744073709551610 | 飽和しない最大に近い値(`2^64-1` を 10 で割って切り捨てた値 × 10) |
| 1844674407370955162 | 10 | 1 | 4 | 18446744073709551615 | 積 18446744073709551620 が `2^64-1` を超えるので飽和(`low` は折り返した値 4) |
| 18446744073709551615 | 2 | 1 | 18446744073709551614 | 18446744073709551615 | 飽和 |
| 9223372036854775807 | 10 | 4 | 18446744073709551606 | 18446744073709551615 | 飽和(積は 92233720368547758070) |
| 9223372036854775808(2^63) | 2 | 1 | 0 | 18446744073709551615 | 積 2^64 が上限を 1 超えるので飽和(`low` が 0 になる点が罠: `low` を見て判定しない) |
| 18446744073709551615 | 10 | 9 | 18446744073709551606 | 18446744073709551615 | 飽和 |

`NextWaitSecs` の境界例(関数単体。検証済みの定義では起きない入力):

| IntervalSecs | Multiplier | MaxIntervalSecs | retriesDone | lastWaitSecs | 結果 |
|---|---|---|---|---|---|
| 10 | 2 | 60 | 0 | 0 | 10 |
| 10 | 2 | 60 | 3 | 40 | 60(80 を 60 で頭打ち) |
| 10 | 2 | 60 | 4 | 60 | 60 |
| 1 | 10 | 18446744073709551615 | 1 | 9223372036854775807 | 18446744073709551615(積が飽和し、上限で止まる) |
| 1 | 10 | 18446744073709551615 | 1 | 1844674407370955161 | 18446744073709551610(飽和しない最大) |
| 1 | 10 | 18446744073709551615 | 1 | 1844674407370955162 | 18446744073709551615 |
| 30 | (Backoff なし) | - | 5 | 30 | 30 |

検証済みの定義では `interval` と `max_interval` が 86400 秒以下なので、`MulU64` が飽和する場面は無い(最大でも 86400 × 10)。再実装でこの極端な入力まで再現する必要は実用上ないが、乗算は折り返さず、例外にもしないこと。

待ち時間の列の例(計算した値):

| interval | multiplier | max_interval | max | 待ち時間の列(秒) | 合計 |
|---|---|---|---|---|---|
| 30s | なし | なし | 3 | 30, 30, 30 | 90 |
| 10s | 2 | 60s | 5 | 10, 20, 40, 60, 60 | 190 |
| 60s | 3 | 60s | 3 | 60, 60, 60 | 180 |
| 1s | 10 | 3600s(1h) | 6 | 1, 10, 100, 1000, 3600, 3600 | 8311 |
| 1m | 2 | 24h | 4 | 60, 120, 240, 480 | 900 |
| 1h | 10 | 24h | 4 | 3600, 36000, 86400, 86400 | 212400 |

### 7.7 実行ループ(`RunJobWithRetry`)。`run` と `pipeline` の共通

```text
retriesDone = 0; lastWaitSecs = 0
loop:
    exec = RunJob(job, sig)
    match Retries.Decide(job, exec, retriesDone, lastWaitSecs):
        Stop: return exec                          # 最後の結果
        Retry(waitSecs):
            表示: 常に 1 行(§7.9)。retryNumber = retriesDone + 1
            match waiter.Wait(waitSecs, sig):     # 本番は §7.8。テストでは差し替える
                Elapsed:          retriesDone += 1; lastWaitSecs = waitSecs; continue
                Interrupted(n):   return new JobExecution(JobId: job.Id, Started: false, ExitCode: null, TerminatedBySignal: null,
                                                          StartFailure: false, Interrupted: true, SignalNumber: n)
```

- 返された最後の結果だけが、分類・表示・ステート記録の対象(ステートは最終の結果で 1 回だけ書かれ、途中では書かれない)。リトライの回数・直前の待ち時間はローカルな値で、ステートファイルに書かない。
- したがって、conductor が落ちたあとの再開では、そのジョブを最初の実行から数え直す。
- 使い切った(`retriesDone == Max` で、まだ対象の終了コード): `Stop` → 通常のエラーと同じ(最後の実行の終了コードで終わり、再開位置はそのジョブ)。再実行の結果が成功・警告なら通常どおり遷移へ進む。
- 総実行回数: 最大 `Max + 1`。知らせの回数: 実際に実行し直す(または待機に入る)回数。
- 起動前に中断された(合図が立っていた)ジョブはリトライせず、その場で `Started = false` の結果を返す(`Decide` は `Started = false` で Stop)。

### 7.8 待機とシグナルによる中断(`RetryWaiter.Wait`)

**決定: 待機は同期で、中断の合図 `Interrupt`([07](07-process-and-signals.md) §6)の `WaitRaised(TimeSpan)`(内部は `ManualResetEventSlim.Wait`)を、250 ミリ秒以下の刻みで繰り返し、刻みごとに合図と単調な時計の期限を確かめる。** シグナルのハンドラは `Interrupt.Trigger` を呼び、その中で `ManualResetEventSlim.Set()` が呼ばれるので、待機はハンドラの実行と同時に起きて戻る(刻みの上限は、起こし損ねの保険と、時計の確認のため)。`async`・`Task`・`Thread.Sleep`(合図で起こせない)・`Timer` は使わない。スレッドを作らない。型 `Interrupt`・`WaitOutcome`(`Elapsed` / `Interrupted(int SignalNumber)`)と `RetryWaiter.Wait(ulong secs, Interrupt sig)` の署名は 07 §7.5 が正本で、この節は、**秒から待機への数値の変換**(桁あふれ・丸め・上限の分割)を規定する。残りの時間は、`double` を使わず、この節の整数の式で求める(`secs` が `ulong` の全範囲でも、丸めと例外が起きない)。07 §7.5 も同じ式を使う(この節の式が正本)。

手順(`Conductor.Runner` に置く。時計は差し替え可能にする: `Func<long> timestamp`(既定は `Stopwatch.GetTimestamp`)と `long frequency`(既定は `Stopwatch.Frequency`)):

```text
start  = timestamp()                              # long
target = Saturating.MulU64(secs, (ulong)frequency)   # 期限までのティック数(ulong の飽和。secs = 2^64-1 でも例外にならない)
loop:
    if sig.Number が 有る(n): return Interrupted(n)            # 待機の開始時にも確かめる(最初の番号)
    elapsed = (ulong)(timestamp() - start)                      # 単調なので 0 以上
    if elapsed >= target: return Elapsed
    remaining   = target - elapsed                              # ティック(ulong)
    remainingMs = CeilDiv((UInt128)remaining * 1000, frequency) # ミリ秒に切り上げ。UInt128 で桁あふれを避け、結果は ulong に飽和
    chunkMs     = (int)Math.Min(remainingMs, 250)               # 1〜250
    sig.WaitRaised(TimeSpan.FromMilliseconds(chunkMs))          # 合図の Trigger で早く戻る。戻り値は使わず、ループの先頭で再確認
```

- `UInt128` の `*` は、`unchecked(...)` で囲むか、`remaining` が `ulong` で `1000` との積が `UInt128` に収まる(最大 2^64 × 1000 < 2^128)ことを根拠にして、`checked` の文脈でも例外にならないことを、単体テストで固定する(§0.3。`UInt128` の `checked` の挙動は未確認)。`UInt128` を避けるなら、`Math.BigMul` で積の上位・下位を得て、`frequency` で割る長除法で書いてよい(結果が同じなら自由)。
- 守るべきことは 2 つ: (a) 合図から 1 秒以内に打ち切る(この方式では、`Trigger` で即座に、保険として最大でも 250 ミリ秒)。(b) 待ち時間が `secs` 秒以上になるまで `Elapsed` を返さない(経過は単調な時計で毎回確かめる。ミリ秒への換算は**切り上げ**なので、早く戻っても、ループが期限を再確認する)。
- `TimeSpan.FromMilliseconds(chunkMs)` に渡すのは、250 以下の `int`(`double` に直しても丸めは起きない)。`TimeSpan` を、秒の `ulong` から作らない。

確認(出典: `ManualResetEventSlim.Wait` の API ページ):

- `Wait(int millisecondsTimeout)` は、イベントがセットされると `true`、タイムアウトなら `false` を返す。`-1`(`Timeout.Infinite`)で無期限。`-1` 以外の負数は `ArgumentOutOfRangeException`。上限は `int` の範囲(`int.MaxValue` ミリ秒 = 2,147,483,647 ミリ秒 ≒ 24.8 日)。
- `Wait(TimeSpan timeout)` は、`timeout` のミリ秒数が `Int32.MaxValue` を超えると `ArgumentOutOfRangeException`(ページの例外の記載)。したがって、**秒から作った大きな `TimeSpan` を渡さない**。`TimeSpan.FromSeconds(double)` は `double` を経由するので、丸めが入り、大きな値で範囲を超える(§0.2 の方針に反する)。
- `Wait()` は無期限で、セット済みなら即座に戻る(ページの記載)。`Set` の後で `Wait` に入っても、`Reset` しない限り戻る。この方式は `Reset` を呼ばないので、「確認してから `Wait` に入るまでのあいだに `Set` された」場合も、`Wait` が即座に戻り、取りこぼさない。
- 待機の上限と分割: 検証済みの定義の待ち時間は最大 86400 秒 = 86,400,000 ミリ秒で `int.MaxValue` より小さい。しかし、関数としては `secs` が `ulong` の全範囲を取りうる(2^64-1 秒の入力でも例外にならない、が規定)。1 回の `WaitRaised` に渡す刻みは**常に 250 ミリ秒以下**なので、`int.MaxValue` ミリ秒を超える待機は、刻みの繰り返しで自然に分割される。別の待機(例: `Process.WaitForExit(int)`)に、ミリ秒の `ulong` の値を `int` に直して渡す場合の変換は、次の関数で行う:

```csharp
// 残りミリ秒(ulong)を、Wait(int) に渡せる値に直す。0 は呼ばない前提(呼び側が先に期限を確かめる)。
static int ToWaitMs(ulong remainingMs) => remainingMs > int.MaxValue ? int.MaxValue : (int)remainingMs;
```

| 待ち(秒) | ミリ秒(正確) | `int` に収まるか | 扱い |
|---|---|---|---|
| 86,400(24 時間。定義の上限) | 86,400,000 | 収まる | この方式は 250 ミリ秒ずつ |
| 2,147,483 | 2,147,483,000 | 収まる(`int.MaxValue` = 2,147,483,647 以下) | 同上 |
| 2,147,484 | 2,147,484,000 | 収まらない | `ToWaitMs` で `int.MaxValue` に切り詰め、繰り返す(`TimeSpan` 過負荷は例外) |
| 18,446,744,073,709,551,615(`ulong.MaxValue`) | 18,446,744,073,709,551,615,000 | `ulong` にも収まらない | ミリ秒への換算は `UInt128`(または飽和)で行う。事実上、終わらない |

- 単調な時計: `Stopwatch.GetTimestamp()` と `Stopwatch.Frequency`(ティックの毎秒の数)を使う。ティックの差は `long` の引き算で、`DateTime.UtcNow` の調整(時計合わせ・夏時間)に影響されない。`Stopwatch` が単調増加であること、システムの休止中にティックが進むかどうかは、公式ドキュメントを取得して確認していない(未確認)。分解能は `Frequency` で換算する(Linux・macOS は 10^9、Windows は環境による、と理解している。未確認)ので、秒への換算にティックの数の定数を埋め込まない。
- `ManualResetEventSlim` の `Set` が、シグナルのハンドラのスレッドから、メインスレッドの `Wait` と同時に呼ばれても安全であること(クラスの全メンバーがスレッドセーフ、ただし `Dispose` と `Reset` を除く、という趣旨の記載)は、この文書では公式ドキュメントを取得して確認していない(未確認。実装時に確認する)。この方式は `Reset` と `Dispose` を呼ばない(イベントはプロセスの終了まで生かし、`using` で囲まない)ので、その例外の条件に当たらない。

待機中に中断された場合は、次の実行をせず、分類なし(起動しなかった中断)として扱う: 終了コード `128 + 番号`、ジョブの状態は変えず(エラーとして記録しない)、再開位置はそのジョブ。`pipeline` でも、そこで止まって終わる(§5.3: 分類なしは停止)。再開すると、そのジョブを最初の実行から数え直す。

テストでは、`RetryWaiter` を偽の実装(待ち時間の `ulong` を記録するだけで実際には待たない)に差し替える。待機の実装自体のテストは、偽の時計を渡した単体テストと、別のプロセスからシグナルを送る受け入れテストで行う。

### 7.9 リトライの知らせ

待機の**前**に、`-v` の有無によらず、標準エラーに 1 行(エラーではない。`conductor: retry: ` で始まる)。待機が中断された場合も、その回の知らせは出ている。

- 英語: `conductor: retry: job '<id>' exited with code <last_exit_code>; retry <n> of <max> in <wait_secs> seconds`
- 日本語: `conductor: retry: ジョブ '<id>' が終了コード <last_exit_code> で終わりました。<wait_secs> 秒後にリトライします(<n>/<max> 回目)`
- `<last_exit_code>` は直前の実行の終了コード(`ExitCode` が無ければ 0)。`<n>` は 1 から。`<wait_secs>` は秒の整数(10 進、桁区切りなし。`waitSecs.ToString(CultureInfo.InvariantCulture)`。`ToString("N0")` や `ToString()` の既定(現在の文化)は使わない)で、1 のときも `seconds`(単数形の処理はない)。`<n>`・`<max>`・`<last_exit_code>` も `int.ToString(CultureInfo.InvariantCulture)`。`<id>` は表示用にエスケープ・長さ制限される(規則は [08-output-and-i18n.md](08-output-and-i18n.md))。
- 例: `conductor: retry: job 'extract' exited with code 75; retry 2 of 5 in 20 seconds`

---

## 8. シグナルと中断(流れへの影響)

| 受信のタイミング | ジョブの結果 | ステートの記録 | 終了コード |
|---|---|---|---|
| ジョブの起動の前(`run`) | 起動しない。分類なし | ジョブの状態は変えない。`resume_job_id` = そのジョブ(初期ステートは、空だった場合に書かれている) | 128 + n |
| ジョブの実行中 | 子に同じシグナルを 1 回転送。終了を待つ。`Interrupted`。分類 Error | そのジョブを `error` で記録(`exit_code` はジョブの値があればそれ) | 128 + n |
| リトライの待機中 | 分類なし(§7.8) | 状態は変えない。再開位置 = そのジョブ | 128 + n |
| `pipeline` で、ジョブを実行・記録した直後(まだ完了していない) | - | 再開位置(§9)を記録 | `ExitCodes.SignalExitCode(n)` = 128 + n |
| 定義の検査・ステートの確認の途中 | 検査は最後まで進む | 64・65・66・スキップ(0)が優先 | それぞれ |

- シグナルは SIGHUP・SIGINT・SIGQUIT・SIGTERM(UNIX 系)、Ctrl+C(Windows)。2 回目以降のシグナルは追加の動作をしない。終了コードに使う番号は、最初に受け取ったもの。詳細は [07-process-and-signals.md](07-process-and-signals.md)。
- 実行中の中断ではジョブ自身の終了コードは見ない(分類 Error、返す値は 128 + n)。

---

## 9. 再開位置(`Resume.OnInterrupt`)

入力: 定義、各ジョブの状態。

```text
ResumeJobId = 記述順で最初の「Pending(または記録なし)または Error」のジョブ(無ければ null)
Completed   = (ResumeJobId == null)
EndReached  = 「Skipped(ByEnd)」のジョブが 1 つでもあれば true
```

エラー・pending を区別せず、記述順で最初のものが再開位置。エラーのジョブが前にあれば、そのジョブ。この関数は記録を作らない(呼び出し側が `ApplyInterrupt`(§3.5)で書く)。

---

## 10. 完了と完了後

- 完了(`Completed = true`)になるのは: 最後のジョブが success / warning で終わった、`@end` に到達した(Pipeline、または Individual で後ろに未完了が無い)、スキップの記録で最後のジョブまで済んだ(§3.4)。
- 完了のステートは `completed = true` のまま**残る**(`remove_on_complete` が真でない限り)。書式は変わらない。
- 完了のステートは、読み込み(`run`・`pipeline` の `Prepare`)では「空」として扱う(定義との照合もしない)。次の `pipeline`・`run <最初のジョブ>` は先頭から始まり、初期ステートで上書きする。先頭以外のジョブを `run` で単独に呼ぶと順序外(66)。
- `remove_on_complete` が真のとき、完了のステートを書いた後に `ResetState` で削除する(失敗は 65。完了のステートは残る)。偽のときは削除を試みず、0 で終わる(`run` ではジョブの結果による終了コード)。
- `watch` は完了のステートも表示する(表の下に `completed`/`完了` の行)。

---

## 11. 保証の範囲(at-least-once)

- 結果の記録は、ジョブの終了の**後**。終了と書き込みのあいだに conductor が落ちると、ジョブは未実行として扱われ、再実行されうる(ジョブが 2 回動きうる)。
- 結果の書き込みに失敗したときは、ジョブの終了コードではなく 65 で終わり、「ジョブは実行されました」と表示する。
- `retry` のジョブは、最大 `Max + 1` 回動く。
- 時間の上限(タイムアウト)はない。ジョブが終わらなければ待ち続ける。
- 排他制御はしない([06-state-file.md](06-state-file.md) §11)。

---

## 12. 実装の特記事項(再実装で再現すること)

| # | 項目 | 挙動 |
|---|---|---|
| 1 | リトライの知らせの単数形 | 待ちが 1 秒のとき `in 1 seconds`(英語)。単数形の処理はない |
| 2 | エラーのジョブを再実行して、起動前・待機中に中断されたとき | 分類なしなので `status` は error のまま変わらないが、`exit_code` は消える(`ApplyOutcome` が `ExitCode` に実行結果の値 = `null` を書く) |
| 3 | 成功直後の `resume_job_id` | 成功・警告のあとの `NextStep.ResumeJobId` は `null`。ただし後続のスキップ記録(`ApplySkip`)は、エラーが無いとき `ResumeJobId` を最初の pending にする。そのため、遷移で飛ばした後のステートには、成功直後でも `resume_job_id` が入りうる |
| 4 | 分類なし(起動前の中断)の終了コードの整形 | `isError = false` で整形する。`128 + n` が 256 以上かつ下位 8 ビットが 0 になる番号(n = 128 など)は、0 になりうる(実在のシグナル番号では起きない) |
| 5 | `pipeline` の「完了を判定できない」 | 全ジョブを通過しても `Completed` が真にならなければ 65(ステートは初期化しない) |
| 6 | ステートの誤りの原因の文 | 種類の文はロケールに従うが、理由(例: `JSON として読めません`)は常に日本語の固定文(06 §6.4) |
| 7 | `PipelineState` の `Equals` | 配列を要素ごとに比べる(`record` の合成の `Equals` は配列を参照で比べるので、手で実装する。§0.2) |

---

## 13. `Conductor.Flow` の純粋関数の署名の骨格

実装ではなく、名前・引数・戻り値の骨格。`...` の本体は説明のためで、実装では値を返す。すべて `public static`、例外を投げない(不正な入力の組み合わせは、表の規則どおりの値を返す)。

```csharp
namespace Conductor.Flow;

public enum RunMode { Individual, Pipeline }
public enum DecisionKind { Run, Skip, Reject }
public enum DecisionReason { AlreadyExecuted, SkipTarget, AfterEnd, OutOfOrder }

public readonly record struct Decision(DecisionKind Kind, DecisionReason? Reason);   // Run のとき Reason は null

public abstract record TransitionTarget;                       // EndTarget | JobTarget の 2 つだけ
public sealed record EndTarget : TransitionTarget;             // `@end`
public sealed record JobTarget(string JobId) : TransitionTarget;

// 検証済みの定義(03 の型)から、判断に必要な部分だけを取り出した見え方
public sealed record FlowJob(
    string Id,
    IReadOnlyList<int> SuccessExitCodes,       // 省略されていれば [0] にして渡す
    IReadOnlyList<int> WarningExitCodes,
    TransitionTarget? OnSuccess,
    TransitionTarget? OnWarning,
    RetryPolicy? Retry);

public sealed record FlowDefinition(string PipelineName, IReadOnlyList<FlowJob> Jobs);

public sealed record NextStep(
    string? NextJobId, IReadOnlyList<string> SkippedJobIds, SkipReason? SkipReasonKind,
    bool EndReached, bool Completed, string? ResumeJobId);

public readonly record struct ResumePoint(string? ResumeJobId, bool Completed, bool EndReached);

public abstract record RetryDecision;                       // Stop か Retry の 2 つだけ(sealed のサブタイプ)
public sealed record Stop : RetryDecision;
public sealed record Retry(ulong WaitSecs) : RetryDecision;

public static class ExitCodes
{
    public static int For(int reported, bool isError);       // §2.2
    public static int SignalExitCode(int signalNumber);      // For(Saturating.AddI32(128, n), true)
}

public static class Saturating
{
    public static ulong MulU64(ulong a, ulong b);            // §7.6
    public static int AddI32(int a, int b);                  // §2.2
}

public static class Classifier
{
    public static JobOutcome Classify(FlowJob job, JobExecution execution);                           // §2.3
    public static (JobOutcome Outcome, NextStep Next) ClassifyAndDecide(
        FlowJob job, JobExecution execution, IReadOnlyList<JobState> states, FlowDefinition def, RunMode mode);   // §3
}

public static class NextSteps
{
    public static NextStep NextStep(FlowDefinition def, string jobId, Classification? classification,
        IReadOnlyList<JobState> states, RunMode mode);                                                   // §3.2
}

public static class Decisions
{
    public static Decision DecideBeforeRun(string jobId, FlowDefinition def, IReadOnlyList<JobState> states,
        string? resumeJobId, bool endReached);                                                           // §4
}

public static class Resume
{
    public static ResumePoint OnInterrupt(FlowDefinition def, IReadOnlyList<JobState> states);          // §9
}

public static class Retries
{
    public static bool IsRetryTarget(RetryPolicy policy, JobExecution execution);                    // §7.4
    public static RetryDecision Decide(FlowJob job, JobExecution execution, int retriesDone, ulong lastWaitSecs);   // §7.5
    public static ulong NextWaitSecs(RetryPolicy policy, int retriesDone, ulong lastWaitSecs);           // §7.6
}

public static class Recorder
{
    public static PipelineState InitialState(string definitionPath, FlowDefinition def, string? runId);                // 06 §2.4
    public static PipelineState ApplyOutcome(PipelineState s, string jobId, JobExecution exec, JobOutcome outcome, NextStep next);   // §3.3
    public static PipelineState ApplySkip(PipelineState s, string jobId, DecisionReason reason, FlowDefinition def);   // §3.4
    public static PipelineState ApplyInterrupt(PipelineState s, ResumePoint point);                                    // §3.5
}
```

- 戻り値の `(JobOutcome Outcome, NextStep Next)` は名前付きタプル(`ValueTuple`)。静的クラスの名前(`NextSteps`・`Decisions`・`Retries`・`Resume`・`Saturating`・`Recorder`)は [02](02-architecture.md) の一覧に合わせた。名前は実装に任せてよい(引数と戻り値の意味が正本)。
- `JobExecution` は 02 に合わせて `sealed record` にした(`JobId` を持つ)。1 回の実行ごとに 1 つ作る小さな値なので、`readonly record struct` でもよい。
- `Recorder` の関数は、引数の `PipelineState` を変えず、`with` 式で新しい値を返す(列は新しい配列を作る)。

---

## 14. 単体テストで固定する項目(この文書の範囲)

実機で動かせていない箇所を、実装時に次のテストで固定する。

| 項目 | 内容 |
|---|---|
| `ExitCodes.For` | §2.2 の表の全行。`reported = int.MinValue` / `int.MaxValue`、`isError = true`/`false` |
| `Saturating.AddI32` | `(128, int.MaxValue)` → `int.MaxValue`、`(int.MinValue, -1)` → `int.MinValue` |
| `Saturating.MulU64` | §7.6 の境界の表の全行。特に 2^63 × 2(`low` が 0 になる)。`checked` を有効にしたテストの構成でも例外にならないこと |
| `NextWaitSecs` | §7.6 の境界の表と待ち時間の列の表 |
| `RunIdRules.Parse` | §6.1 の有効・無効の例と境界の表。`"a\n"`、全角 `１`、孤立サロゲート、`new string('あ', 30)` |
| `DurationParser.TryParseSecs` | §7.3 の例(受理・拒否の全件) |
| `PipelineState.Equals` | §0.2 の 3 つの性質 |
| `Classify` / `NextStep` / `DecideBeforeRun` | §2.3・§3.2・§4.2 の表の全行 |
| `RetryWaiter.Wait` | 偽の時計で、`secs = 0`、`secs = 1`、`secs = ulong.MaxValue`(例外が出ず、`Elapsed` が返らず、合図で `Interrupted` が返る)、開始時にすでに合図が立っている場合 |
| 受け入れ(別プロセスから送信) | 待機中の SIGTERM で 143、1 秒以内に終了 |

---

## 15. 共有状態とスレッド間の可視性

シグナルのハンドラ(`PosixSignalRegistration` のコールバック、`Console.CancelKeyPress`)は、メインスレッド以外(スレッドプールなど)で実行されうる([02](02-architecture.md) と [07](07-process-and-signals.md) の前提。この文書では公式ドキュメントを取得して確認していない: 未確認)。したがって、メインスレッドとハンドラのあいだで共有する値は、次の表の方法だけで読み書きする。

### 15.1 共有状態の表

型 `Interrupt`(中断の合図)の署名と規則は [07](07-process-and-signals.md) §6 が正本。この文書は、それを使う側の規則と、共有する値の一覧を定める。

| 共有する値 | 所有者 | 書き手 | 読み手 | 方法 | 終了条件 |
|---|---|---|---|---|---|
| 中断の合図(最初のシグナル番号) `Interrupt` | `Prepare` が 1 つ作り、`run`・`pipeline`・`RunJob`・`RetryWaiter` に渡す | シグナルのハンドラ(複数のハンドラが同時に呼ばれうる) | メインスレッド(待機のループ、`RunJob` の待機、`pipeline` のジョブのあいだ) | 書き込みは `Interrupt.Trigger(n)`(07 の実装: 「立っているか」と番号を 1 つの `long` にまとめ、`Interlocked.CompareExchange` で 1 度だけ書く。最初の 1 つだけが勝ち、負けた側は何もしない)、読み込みは `Interrupt.Number`(`Volatile.Read`) | プロセスの終了まで。立ったら戻さない |
| 起こすための `ManualResetEventSlim`(`Interrupt` の内部) | 同上 | `Trigger` の勝った 1 回だけ `Set()` | メインスレッド(`WaitRaised` → `Wait`) | `Set()` / `Wait` | プロセスの終了まで。`Reset`・`Dispose` を呼ばない |
| 転送を済ませたか(子への転送を 1 回だけにする旗) | `RunJob`(07) | ハンドラまたは待機のスレッド(07 の決定に従う) | 同上 | 07 の規定(`Interlocked.CompareExchange` による) | `RunJob` の終了まで |
| ステート値・定義・実行結果 | メインスレッド | メインスレッドだけ | メインスレッドだけ | 共有しない(不変の値を引数で渡す) | - |

規定の理由と、使う側の規則:

- 番号と「立った」の 2 つの値を別々に持つと、「立ったが番号がまだ見えない」瞬間を読み手が見うる。07 は、1 つの原子的な値にまとめ、`CompareExchange` で 1 度だけ書くので、読み手は「無し」か「完全な番号」のどちらかだけを見る。番号 0 と無しを区別できる(`Number` が `int?`)。
- 1 つの判断のなかで、`Interrupt.Number` を 2 回読んで食い違いを起こさない(1 回読んで、その値を使う。例: `pipeline` のジョブのあいだの確認と、`ForSignal(n)` の `n` は、同じ読み取りの結果)。
- `Volatile.Read` / `Interlocked.CompareExchange` は、`int`・`long` に対して .NET が提供する標準のメモリバリア付きの操作。読み書きの意図を呼び出しごとに明示する `Volatile` / `Interlocked` の API に統一し、`volatile` キーワードのフィールドと混ぜない。`Volatile` と `Interlocked` の厳密なメモリモデル上の保証を、公式ドキュメントを取得して確認していない(未確認。API の存在と用途は一般に知られているが、この文書では出典を取っていない)。
- `lock` は、ハンドラの中で使わない(メインスレッドが保持している最中にハンドラが走ると、待たされうる)。`lock` を使う箇所は、この文書の範囲には無い。
- `Task.Wait`・`.Result`・`GetAwaiter().GetResult()` を使わない(同期の本体にスレッドプールの待ちを持ち込まず、デッドロックを作らない)。
- ハンドラの中で、`Console.Error.WriteLine`・ファイルの書き込み・`Process` のメンバーの呼び出し(子への転送を除く。転送は 07 の規定)をしない。`Interrupt.Trigger`(と `Cancel` の設定)だけを行う。

### 15.2 確認の範囲と未確認

確認できたこと(出典は本文の該当箇所):

- `ManualResetEventSlim.Wait(int)` と `Wait(TimeSpan)` の契約(`-1`、負数の例外、`TimeSpan` が `Int32.MaxValue` ミリ秒を超えると `ArgumentOutOfRangeException`)。出典: `ManualResetEventSlim.Wait` の API ページ。
- `Math.BigMul(ulong, ulong, out ulong)` の署名・戻り値(上位)・`low`(下位)・対象の版。出典: `Math.BigMul` の API ページ。
- 既定の `UTF8Encoding` が不正なサロゲートを U+FFFD(`EF BF BD`)に置き換えること。出典: `UTF8Encoding(Boolean, Boolean)` の API ページの例。

未確認(実装時に確認する):

- `ExitCode` が Windows で負数として現れること(§1.2)、`Main` の戻り値が範囲外のときの OS ごとの扱い(§2.2)。
- `UInt128` の `*` と `checked` の関係(§0.3)。
- `Regex` の `$`・`\d`・`IgnoreCase` の挙動(§6.1。一般に知られた挙動を、この文書では取得して確認していない)。
- `Encoding.UTF8.GetByteCount` が §6.1 の走査と一致すること。
- `char.IsDigit`・`int.Parse` の受理する形(§7.3)。
- Unix の引数・環境変数の不正な UTF-8 が .NET の `string` にどう入るか(§6.1)。
- `Environment.GetEnvironmentVariable` の未定義・空・名前の大文字小文字の扱い(§6.2)。
- `Stopwatch` の単調性・休止中の挙動・`Frequency`(§7.8)。
- `ManualResetEventSlim` の `Set` と `Wait` の、複数スレッドからの同時の使用の安全性の記載(§7.8)。
- `Volatile`・`Interlocked` のメモリモデル上の保証の出典(§15.1)。
- シグナルのハンドラが実際にどのスレッドで実行されるか(07)。
- ジョブの標準入出力の扱い(子が conductor の入出力を引き継ぐか)は、07 が定める。この文書の流れへの影響は無い。
- 複数のシグナルがほぼ同時に届いたときの「最初の番号」は、ハンドラが呼ばれた順序(`CompareExchange` に勝った順)に依存する。再実装では「最初にハンドラが呼ばれた 1 つ」を保持すれば足りる。ほぼ同時の到着の再現は未確認。

実装に任せてよい点(外から見える挙動が同じなら自由):

- 判断関数・記録関数の内部構造、型の表現(`enum` か `record` の和か。この文書は `enum` と `record` を前提にした署名を示した)。
- 待機の刻みの具体値(250 ミリ秒以下。「合図から 1 秒以内」「`secs` 以上で `Elapsed`」を守る)。
- 標準エラーの出力のバッファリング(ただし行単位で、順序を保つ)。
