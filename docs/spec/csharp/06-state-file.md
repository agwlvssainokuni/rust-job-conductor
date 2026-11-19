# 06. ステートファイル(論理項目・JSON の形式・読み書き・整合性・reset)

対象: C#(.NET)で実装する `conductor` v0.8.0 と同じ挙動の実行ツール。ステートファイルは、あるパイプライン定義の実行の途中経過(どのジョブがどう終わったか、どこから再開するか)を保存するファイル。実行の意味論(いつ何を記録するか)は [05-execution-flow.md](05-execution-flow.md) にある。この文書は、ステートが持つ**論理的な項目**、保存の**必須の性質**、**JSON の形式**、読み書きの手順、異常系の挙動を定める。

ファイル名は `<定義ファイル名>.state` / `<定義ファイル名>.<run ID>.state`(定義ファイルの拡張子を含む)で、中身は **JSON**(UTF-8、BOM なし)。実装は `Conductor.Runtime` の `Conductor.State`(読み書き・整合性検査・reset)。ステート値の型(`JobState`、`PipelineState` など)と、記録の純粋な関数は `Conductor.Core` の `Conductor.Flow` にある([05](05-execution-flow.md) §1.1、§13)。JSON の読み取りは、定義・変換と共用する自前の読み取り(`Conductor.Json`)を使う(§6.5)。

この文書の確認の範囲: この文書を書いた環境には .NET SDK が無く、C# のコードを実行して確かめていない。API の契約は、learn.microsoft.com の公式ドキュメントを取得して確認できた範囲だけを「確認(出典: …)」と書く。取得できなかった・記載が無かった点は「未確認」と書く(§13.4 に一覧)。§4.4 の実ファイルの例は、§4.2・§4.3 の規則から、別の言語のスクリプトで機械的に作った文字列で、C# の実装の出力そのものではない(実装時に、この例との完全一致を単体テストで固定する。§6.6)。サポートする版(最新の LTS と、サポート中の 1 つ前の LTS)は「実装時に確認」する([11-quality-and-build.md](11-quality-and-build.md))。

## 0. この文書の読み方

| 区分 | 内容 |
|---|---|
| 必須(どの実装も同じ) | §2 の論理項目・値の規則・矛盾の条件、§3 の保存の性質、§4 の JSON の形式(キー名・型・固定の順序・書き出しの規則)、§6 の読み込みの検査(順序・種類・理由文・案内文)、§7 の整合性検査、§8 の reset の挙動、ファイルの場所と名前(§1)、§5 のパーミッションの規定 |
| 実装に任せてよい点 | 一時ファイルの名前の細部(§1 の条件を満たす)、内部の構造、読み込みの内部の作り(§6.5 の規則を満たす) |

---

## 1. 場所と名前

ステートファイルは、**定義ファイルと同じディレクトリ**に置く。

| run ID | ファイル名 |
|---|---|
| なし | `<定義ファイル名>.state` |
| あり | `<定義ファイル名>.<run-id>.state` |

- `<定義ファイル名>` は、定義ファイルのパスの最後の要素(拡張子を含む)。例: `conductor.json` → `conductor.json.state`、`--config ci/nightly.yaml --run-id 12345` → `ci/nightly.yaml.12345.state`。
- 定義ファイルのパスは、`--config` で与えられた文字列のまま(正規化・絶対化しない。`Path.GetFullPath` を通さない)。ディレクトリ部分が空(`conductor.json` のように親が無い)なら、実行ディレクトリ。
- パスの組み立て(`StatePaths.Of(string definitionPath, string? runId)`)は、`Path.GetFileName(definitionPath)`(最後の要素)と `Path.GetDirectoryName(definitionPath)`(ディレクトリ部分)を使い、`Path.Join(directory, name)` で結ぶ。次の点を守る:
  - `Path.GetDirectoryName` は、パスにディレクトリ部分が無いとき(`conductor.json`)に空文字列、ルートのときに `null` を返す、と理解している(この文書では公式ドキュメントを取得して確認していない: 未確認)。`null` と空文字列は、どちらも「ディレクトリ部分なし」として空文字列に統一する。空のとき、ステートのパスは名前だけ(`Path.Join("", name)` は `name` になる、と理解している。未確認。実装は `directory.Length == 0 ? name : Path.Join(directory, name)` と書いて、この挙動に頼らない)。ディレクトリの操作(§5 の 9、§8.2 の一覧)には、空のとき `"."` を渡す。標準エラーに出す `<ステートファイルのパス>` は、この結果の文字列。
  - `Path.GetFileName` は、パスが区切り文字で終わるとき(`ci/`)に空文字列を返す(未確認)。空の名前になる定義ファイルのパスは、定義の読み込み(64)で先に弾かれる。ここでは来ない前提とするが、空なら `StateError`(writeFailed)でなく内部の誤り(70)にする。
  - 区切り文字は、Unix では `/` だけ、Windows では `/` と `\` の両方(`Path.DirectorySeparatorChar` と `Path.AltDirectorySeparatorChar`)。この扱いは `Path` に任せ、自前で `LastIndexOf('/')` を書かない。
  - 文化依存の API(`string.StartsWith(string)`・`EndsWith(string)`・`IndexOf(string)`)で名前を判定しない。序数(`StringComparison.Ordinal`)を使う。
- run ID は検査済み(`A-Za-z0-9._-`、1〜64 文字、先頭が `.` でない。[05](05-execution-flow.md) §6)なので、パス区切りや `..` は入らない。
- 一時ファイルの名前: `.<ステートファイル名>.tmp.<プロセス番号>.<時刻(ナノ秒)>.<連番>`(同じディレクトリ。§5)。名前の付け方は実装に任せてよいが、**ドットで始まり、`.tmp.` を含み、後始末(§5 の 9)で識別できる**こと。
- 大文字小文字を区別しないファイルシステム(macOS の既定、Windows)では、`A` と `a` の run ID は同じファイルになる。食い違いは、中身の `run_id` との照合で壊れたステート(65)として検出される(§6 の 9)。

---

## 2. ステートの論理項目

### 2.1 ステート全体の項目

メモリ上の名前(C#。`PipelineState` のプロパティ)と、ファイル上のキー名の対応を併記する。メモリ上の名前はキー名の PascalCase 版だが、`job` だけはプロパティ名が `Jobs`。

| 項目(ファイルのキー) | メモリ上の名前 | 型 | 必須 | 意味・値 |
|---|---|---|---|---|
| `format_version` | (書き出し時に固定値 `1` を出す) | 整数 | 必須 | 常に `1`。読み込みでは最初に検査される(§6) |
| `definition_key` | `DefinitionKey` | 文字列 | 必須 | 定義ファイルのパス(`definition_path` と**同じ文字列**を書く。読むときは別々に保持するが、どちらも照合に使わない) |
| `definition_path` | `DefinitionPath` | 文字列 | 必須 | 定義ファイルのパス(`--config` の文字列。文字列として表せない部分(孤立サロゲート)は U+FFFD に置き換える。§4.3、[04](04-cli.md))。空でもよい |
| `pipeline_name` | `PipelineName` | 文字列 | 必須 | 記録時の `pipeline.name`。照合に使う(§7) |
| `job_order` | `JobOrder` | 文字列の列(`IReadOnlyList<string>`) | 必須 | 記録時のジョブIDを、定義の記述順に並べたもの |
| `job` | `Jobs` | ジョブ状態の列(`IReadOnlyList<JobState>`) | 必須 | ジョブごとの状態(§2.2)。`job_order` と同じ順・同じ数 |
| `resume_job_id` | `ResumeJobId` | 文字列(`string?`) | 任意 | 次に実行する、または再開するジョブのID。無いときは項目自体を保存しない。`job_order` に含まれる値 |
| `end_reached` | `EndReached` | 真偽 | 必須 | `@end` に到達したか。一度 true になったら、その run のあいだ false に戻らない |
| `completed` | `Completed` | 真偽 | 必須 | 完了したか(§9) |
| `run_id` | `RunId` | 文字列(`string?`) | 任意 | run ID。run ID ありのステートだけが保存する。無いステートには項目を保存しない |

上の表にない項目は、ステート全体にも、ジョブ状態の中にも許さない(§6 の `corrupt`)。

### 2.2 ジョブ状態の項目

| 項目(ファイルのキー) | メモリ上の名前 | 型 | 必須 | 意味・値 |
|---|---|---|---|---|
| `id` | `Id` | 文字列 | 必須 | ジョブID |
| `status` | `Status` | 文字列 | 必須 | `"pending"` / `"success"` / `"warning"` / `"error"` / `"skipped"`(これ以外は `corrupt`)。メモリ上は `JobStatus` 列挙([05](05-execution-flow.md) §1.1 の対応表) |
| `exit_code` | `ExitCode`(`int?`) | 整数(`int` の範囲: -2147483648〜2147483647) | 任意 | ジョブの終了コード。`status` が success / warning / error のときだけ持てる。終了コードが無い結果(起動失敗・シグナル終了)の error には保存しない。負数も許す(例 `-1`)。範囲外の整数は `corrupt` |
| `skip_reason` | `SkipReason`(`SkipReason?`) | 文字列 | 条件付き | `"byTransition"`(遷移によるスキップ)/ `"byEnd"`(`@end` によるスキップ)。**`status = "skipped"` のときだけ必須、そのほかの `status` では保存してはならない** |

### 2.3 内容の矛盾(読み込み時に `corrupt` になる条件)

- ジョブ状態のジョブID に重複がある。
- ジョブ状態の並び(`id` の列)が `job_order` と、数も順序も完全には一致しない。
- `resume_job_id` が `job_order` にない。
- `resume_job_id` があるのに `completed = true`(完了と再開位置は両立しない)。
- `status` が `skipped` なのに `skip_reason` が無い、または `skipped` 以外なのに `skip_reason` がある。
- `status` が pending / skipped なのに `exit_code` がある。

### 2.4 初期ステート(`Recorder.InitialState`)

定義から作る、記録前のステート: 全ジョブ `pending`(`exit_code`・`skip_reason` なし)、`resume_job_id` 無し、`end_reached = false`、`completed = false`、`definition_key` = `definition_path` = 定義ファイルのパス文字列、`pipeline_name` = 定義の名前、`job_order` = 定義のジョブIDの記述順、`run_id` = 決定した run ID(あれば)。

### 2.5 不変条件(実装が保つもの)

[05](05-execution-flow.md) §3 の記録関数が作るステートは、次を満たす(読み込みの検査と対応する)。

- ジョブ状態と `job_order` は同じ順・同じID。
- `skipped` のジョブにだけ `skip_reason` がある。`exit_code` は結果のある状態(success / warning / error)にだけある。
- `end_reached` は単調(true になったら戻らない)。
- `completed = true` のとき `resume_job_id` は無い。
- ただし、エラーのジョブを再実行して「起動前の中断」([05](05-execution-flow.md) §12 の 2)になったときは、`status = error` のまま `exit_code` が消える。

---

## 3. 保存の必須の性質

| # | 性質 | 内容 |
|---|---|---|
| 1 | 原子的な置換 | 読み手は、直前の完全な内容か、新しい完全な内容のどちらかだけを見る。途中の内容は決して見えない。どの段階で失敗しても、ステートファイルは直前の完全な内容のまま(§5) |
| 2 | 再開に必要な情報 | 全ジョブの `id`・`status`・`exit_code`・`skip_reason`、`resume_job_id`、`end_reached`、`completed`、定義との照合用の `pipeline_name`・`job_order`、run ID を保存する。リトライの回数・待ち時間は保存しない(再開は最初の実行から数え直す) |
| 3 | 壊れたときの検出 | 読み込みで、構文・型・値・矛盾・版・run ID・大きさを検査し(§6)、壊れていれば**黙って続行せず**、種類と理由を表示して 65 で止まる。読み込みは、ファイルを消さず、書き換えない |
| 4 | 完了のステートの扱い | 完了のステートは書式を変えず残し、読み込み(`run`・`pipeline`)では「空」とみなす(§9) |
| 5 | 耐久性の 2 段階 | 結果・スキップ・中断の記録は、内容の同期まで行う(Durable)。最初の初期ステートだけは、同期を省いてよい(Fast)(§5) |
| 6 | 排他制御はしない | ファイルロック・PID ファイルを使わない(§11) |

---

## 4. JSON の形式(`format_version = 1`)

### 4.1 エンコードの基本

- UTF-8 の JSON(RFC 8259 の 1 個の値)。最上位は**オブジェクト**。BOM は書かない(読むときは拒否する。§6.1 の 5・6a)。
- 論理項目とのキーの対応は同名: §2.1 の項目は最上位のキー、§2.2 は `job` の配列の要素(オブジェクト)の中のキー。`job` は**オブジェクトの配列**(キー名は `job` のまま。論理項目の名前と一致させるための決定)、`job_order` は文字列の配列。
- 任意の項目(`resume_job_id`、`run_id`、`exit_code`、`skip_reason`)は、無いときキー自体を書かない(`null` を書かない)。
- 空の配列は `[]` と書く(`null` にしない)。

### 4.2 スキーマ(キー・型・固定の順序)

**書き出すときのキーの順序は固定**とする(辞書順にしない)。読み込みはキーの順序に依存しない。

最上位:

| 順 | キー | JSON の型 | 有無 |
|---|---|---|---|
| 1 | `format_version` | 数値(整数 `1`) | 必須 |
| 2 | `definition_key` | 文字列 | 必須 |
| 3 | `definition_path` | 文字列 | 必須 |
| 4 | `pipeline_name` | 文字列 | 必須 |
| 5 | `run_id` | 文字列 | run ID があるときだけ |
| 6 | `job_order` | 文字列の配列 | 必須 |
| 7 | `job` | オブジェクトの配列 | 必須 |
| 8 | `resume_job_id` | 文字列 | あるときだけ |
| 9 | `end_reached` | 真偽(`true`/`false`) | 必須 |
| 10 | `completed` | 真偽 | 必須 |

`job` の要素:

| 順 | キー | JSON の型 | 有無 |
|---|---|---|---|
| 1 | `id` | 文字列 | 必須 |
| 2 | `status` | 文字列(`"pending"`/`"success"`/`"warning"`/`"error"`/`"skipped"`) | 必須 |
| 3 | `exit_code` | 数値(整数。32 ビット符号付き) | 結果のある状態で、終了コードがあるときだけ |
| 4 | `skip_reason` | 文字列(`"byTransition"`/`"byEnd"`) | `status = "skipped"` のときだけ |

書き出し用の関数の署名(読み込みには使わない。理由は §6.5):

```csharp
public static class StateSerializer
{
    // §4.3 の規則で UTF-8 のバイト列にする(末尾に LF 1 つ、BOM なし)。失敗(孤立サロゲートを含む文字列、範囲外の exit_code)は null を返す(StateStore.Write が StateFail(WriteFailed、理由 §6.4 の 22)にする)
    public static byte[]? ToUtf8(PipelineState state);
}
```

- 出力は、`StringBuilder`(または `IBufferWriter<char>`)に、キーを §4.2 の順に 1 つずつ書いて作る(`run_id` などの省略可能なキーは、値があるときだけ、その順序の位置で書く)。順序は、コードに書いた順序そのもので決まる(`Dictionary<,>` などの列挙順に頼らない)。
- 「無い」値のキーは、`null` を書かず、**キーを書かない**。`ExitCode` は **`HasValue` で判定する**(`if (exitCode != 0)` や `if (exitCode)` のような真偽の判定を書くと、値 `0` のキーが落ちる。`int?` を `bool` として使うコードは C# では書けないが、`exitCode.GetValueOrDefault() != 0` のような書き方は誤り)。`SkipReason`・`ResumeJobId`・`RunId` も `HasValue` / `is not null` で判定する。
- 数値は `int.ToString(CultureInfo.InvariantCulture)`(文化の負号・桁区切りを使わない)。
- キーの順序を、単体テストで固定する(§4.4 の例との完全一致。§6.6)。

### 4.3 書き出しの規則

**決定: ステートの JSON の書き出しは、`Utf8JsonWriter` を使わず、自前の文字列の組み立てで行う。** 理由(公式ドキュメントで確認できた範囲):

| # | 事実 | 出典 | 帰結 |
|---|---|---|---|
| 1 | `JsonWriterOptions.NewLine` の既定は `Environment.NewLine`(Windows では CRLF) | `JsonWriterOptions` の API ページ | 設定し忘れると、Windows で CRLF になる。LF 固定の規定(下)と衝突する |
| 2 | `JsonWriterOptions.IndentSize` の既定は 2、`IndentCharacter` の既定は空白、`Indented` で整形 | 同上 | インデントの規定は満たせる |
| 3 | 既定のシリアライザーは、ASCII 以外の全部の文字を `\uxxxx` にエスケープする(エンコーダーの既定の範囲は BasicLatin) | `System.Text.Json` の文字エンコーディングの解説ページ | 日本語のジョブIDが `\u...` の列になり、§4.4 の例と食い違う |
| 4 | `JavaScriptEncoder.UnsafeRelaxedJsonEscaping` は `<` `>` `&` `'` をエスケープしない。ただし「グローバルのブロックリスト」(制御文字、未定義の符号位置、`U+0020` を除く空白類の `Zs` など)の文字は、許可リストに入れてもエスケープされる。このブロックリストは実装の詳細で、**.NET の版ごとに変わってきた**ので、特定の文字がブロックされるかどうかに依存してはならない、と書かれている | 同上 | 出力が .NET の版で変わりうる。同じ入力から同じバイト列を作る(往復・他の実装との比較・テスト)規定に反する |
| 5 | `Utf8JsonWriter` は、構造的に不正な JSON を書こうとすると例外を投げる(既定) | `JsonWriterOptions` の API ページ | 長所だが、決め手ではない |
| 6 | 孤立サロゲートを含む `string` を書いたとき、`Utf8JsonWriter` がどうするか(例外・置換) | (記載を確認できなかった) | 未確認 |
| 7 | 制御文字の `\u00XX` の 16 進の大文字・小文字、U+2028・U+2029 の扱い | (記載を確認できなかった) | 未確認 |

したがって、エスケープの規則を、この文書が全部定め、自前で実装する(下)。`Utf8JsonWriter` と `JavaScriptEncoder` の出力の細部に、この仕様を預けない。

規則:

- 文字列 = 下の規則で作った全体 + `"\n"`(LF 1 つ)。インデントは空白 2 つ、改行は LF のみ(`Environment.NewLine` を使わない。`StringBuilder.AppendLine()` も使わない。`Append('\n')` と書く)。ファイルの末尾は LF 1 つ。CR は出力に現れない(文字列の中の CR は `\r` にエスケープされる)。
- オブジェクトは `{` + LF、各メンバーを「インデント + `"キー"` + `: ` + 値」で、メンバーのあいだは `,` + LF、最後のメンバーの後は LF、`}`。配列は、要素ごとに 1 行(インデントされる)。空配列は `[]`(1 行)。キーと値のあいだは `": "`(コロンの後に空白 1 つ)。行末に空白を付けない。入れ子のインデントは、深さ 1 つにつき空白 2 つ(最上位のメンバーが 2、`job` の要素のオブジェクトが 4、その中のメンバーが 6、`job_order` の要素が 4)。
- 数値は 10 進の整数だけ(`1`、`0`、`-1`、`2147483647`)。小数点・指数・先頭の `+` は書かない。
- 文字列は `"` で囲み、文字ごとに次の表で書く。**表にない文字は、そのまま(エスケープせず)UTF-16 から UTF-8 に符号化して出す**(`<` `>` `&` `'` `/`、DEL(U+007F)、U+0085、ASCII 以外の全部の文字を含む)。

| 文字 | 出力 |
|---|---|
| `"`(U+0022) | `\"` |
| `\`(U+005C) | `\\` |
| U+0008 / U+000C / U+000A / U+000D / U+0009 | `\b` / `\f` / `\n` / `\r` / `\t` |
| その他の U+0000〜U+001F | `\u00xx`(`xx` は 2 桁の**小文字**の 16 進。例 `\u001f`、`\u0000`) |
| U+2028・U+2029 | ` `・` `(6 文字)。どちらも JSON としては生のまま書いても有効だが、これらを行の区切りとみなす構文解析との互換のための決定。読み込み側(§6.5)は同じ値を得る |
| 対になるサロゲート(U+10000 以上) | そのまま(UTF-8 の 4 バイトになる)。`😀` のようにエスケープしない |
| 孤立サロゲート(対にならない U+D800〜U+DFFF) | `definition_path`・`definition_key` では書く前に U+FFFD に置き換える(下)。ほかの文字列に現れたら書き出しの失敗(理由 `内容を JSON にできません`) |

- **孤立サロゲート**: .NET の `string` は UTF-16 のコードユニットの列で、孤立サロゲートを持ちうる(例: Windows のファイル名は孤立サロゲートを含みうる。未確認)。`new UTF8Encoding(false, throwOnInvalidBytes: true).GetBytes(...)` は、これを `EncoderFallbackException` で拒否する。確認(出典: `UTF8Encoding(Boolean, Boolean)` の API ページ): `throwOnInvalidBytes = true` のとき、不正なバイト列(符号化では不正な文字列)を検出すると例外を投げる(ページの記載は `ArgumentException`、例の出力は `EncoderFallbackException`)。`throwOnInvalidBytes = false`(`Encoding.UTF8` の既定)は例外を投げず、不正なサロゲートを `EF BF BD`(U+FFFD)に置き換える。この仕様は、**`definition_path` と `definition_key` だけ**、書き出す前に、孤立サロゲートの 1 つずつを U+FFFD に置き換える関数(`ReplaceLoneSurrogates`)を通す(対になるサロゲートは置き換えない)。定義から来る文字列(`pipeline_name`・ジョブID など)は孤立サロゲートを含まない([03](03-definition-format.md))ので、置き換えは 2 つのパスだけでよい。それでも書き出し(`UTF8Encoding(false, true).GetBytes`)が `EncoderFallbackException` を投げたら、書き出しの失敗(理由 `内容を JSON にできません`)にする。`Encoding.UTF8` を、書き出しの符号化に使わない(黙って置換してしまうため、検査にならない)。
- Unix で、引数に不正な UTF-8 のバイトを含むパスが、.NET の `string` にどう入るか(U+FFFD への置き換えなど)は、公式ドキュメントで確認していない(未確認)。置き換わっていた場合、その `string` でファイルを開くと別の名前になる。この問題は、ステートの書き出しの規定の外(引数の扱いは [04](04-cli.md))。
- 書き出した文字列は、`new UTF8Encoding(encoderShouldEmitUTF8Identifier: false, throwOnInvalidBytes: true).GetBytes(text)` でバイト列にして、バイトのまま書く(§5。`StreamWriter` に任せない。`StreamWriter` の既定の改行・BOM・フォールバックに依存しない)。BOM(`EF BB BF`)を付けない(確認(出典: 同ページ): `encoderShouldEmitUTF8Identifier = true` にしても `GetBytes` は BOM を前置しない。BOM を出すのは `GetPreamble()` を使う側。したがって `GetBytes` の結果をそのまま書けば BOM は付かない)。

### 4.4 実ファイルの例(全フィールドを含む)

以下は §4.2・§4.3 の規則で出力される内容。機械的に作った文字列で(上の確認の範囲を参照)、C# の実装の出力そのものではない。例 6・8・9 は、規則の確認のための入力。

(1)エラーで止まった run ID なしのステート(3 ジョブ。`b` が exit 3)。ファイル名 `c2.json.state`。

```json
{
  "format_version": 1,
  "definition_key": "c2.json",
  "definition_path": "c2.json",
  "pipeline_name": "n2",
  "job_order": [
    "a",
    "b",
    "c"
  ],
  "job": [
    {
      "id": "a",
      "status": "success",
      "exit_code": 0
    },
    {
      "id": "b",
      "status": "error",
      "exit_code": 3
    },
    {
      "id": "c",
      "status": "pending"
    }
  ],
  "resume_job_id": "b",
  "end_reached": false,
  "completed": false
}
```

(2)`@end` で完了した、run ID ありのステート。ファイル名 `conductor.json.r1.state`(a が成功し、遷移 `@end`。b・c が `byEnd`)。`resume_job_id` は無い。

```json
{
  "format_version": 1,
  "definition_key": "conductor.json",
  "definition_path": "conductor.json",
  "pipeline_name": "nightly",
  "run_id": "r1",
  "job_order": [
    "a",
    "b",
    "c"
  ],
  "job": [
    {
      "id": "a",
      "status": "success",
      "exit_code": 0
    },
    {
      "id": "b",
      "status": "skipped",
      "skip_reason": "byEnd"
    },
    {
      "id": "c",
      "status": "skipped",
      "skip_reason": "byEnd"
    }
  ],
  "end_reached": true,
  "completed": true
}
```

(3)全種類の状態を含む例(4 ジョブ。run ID なし、`end_reached = true`、エラーで停止、負の `exit_code`)。定義 `dir/conductor.json`、ファイル名は `dir/conductor.json.state`(ディレクトリ `dir` の中に置かれる)。

```json
{
  "format_version": 1,
  "definition_key": "dir/conductor.json",
  "definition_path": "dir/conductor.json",
  "pipeline_name": "nightly",
  "job_order": [
    "a",
    "b",
    "c",
    "d"
  ],
  "job": [
    {
      "id": "a",
      "status": "success",
      "exit_code": 0
    },
    {
      "id": "b",
      "status": "warning",
      "exit_code": 2
    },
    {
      "id": "c",
      "status": "skipped",
      "skip_reason": "byEnd"
    },
    {
      "id": "d",
      "status": "error",
      "exit_code": -1
    }
  ],
  "resume_job_id": "d",
  "end_reached": true,
  "completed": false
}
```

(4)`skip_reason = "byTransition"` の例(遷移 `a --success--> c` で b を飛ばした直後。a 成功、b は byTransition でスキップ、c が次)。定義が YAML のとき。ファイル名 `conductor.yaml.state`。

(`resume_job_id` が付くかどうかは書き込みの契機による。遷移のスキップは結果の記録([05](05-execution-flow.md) §3.3)で書かれ、その `resume_job_id` は無し。この例のステート値は規則から導いたもの。)

```json
{
  "format_version": 1,
  "definition_key": "conductor.yaml",
  "definition_path": "conductor.yaml",
  "pipeline_name": "nightly",
  "job_order": [
    "a",
    "b",
    "c"
  ],
  "job": [
    {
      "id": "a",
      "status": "success",
      "exit_code": 0
    },
    {
      "id": "b",
      "status": "skipped",
      "skip_reason": "byTransition"
    },
    {
      "id": "c",
      "status": "pending"
    }
  ],
  "end_reached": false,
  "completed": false
}
```

(5)**すべてのキーが現れる**例(run ID あり。ファイル名 `ci/nightly.json.12345.state`。定義 `ci/nightly.json`。`run_id`、`resume_job_id`、`exit_code`(0、正、負)、`skip_reason` が全部ある)。

```json
{
  "format_version": 1,
  "definition_key": "ci/nightly.json",
  "definition_path": "ci/nightly.json",
  "pipeline_name": "nightly",
  "run_id": "12345",
  "job_order": [
    "a",
    "b",
    "c",
    "d"
  ],
  "job": [
    {
      "id": "a",
      "status": "success",
      "exit_code": 0
    },
    {
      "id": "b",
      "status": "warning",
      "exit_code": 2
    },
    {
      "id": "c",
      "status": "skipped",
      "skip_reason": "byTransition"
    },
    {
      "id": "d",
      "status": "error",
      "exit_code": -1
    }
  ],
  "resume_job_id": "d",
  "end_reached": false,
  "completed": false
}
```

(6)ジョブIDに特殊な文字を含む例(`"`、`\`、日本語、タブ、前後の空白)。ジョブID は定義の検証で空でなく `@` で始まらない文字列であることだけが保証される(前後の空白もそのまま)。

```json
{
  "format_version": 1,
  "definition_key": "c.json",
  "definition_path": "c.json",
  "pipeline_name": "名前 \"x\"",
  "job_order": [
    " a ",
    "b\\c",
    "日本語\tタブ"
  ],
  "job": [
    {
      "id": " a ",
      "status": "pending"
    },
    {
      "id": "b\\c",
      "status": "pending"
    },
    {
      "id": "日本語\tタブ",
      "status": "pending"
    }
  ],
  "end_reached": false,
  "completed": false
}
```

(7)定義が JSON のとき(ファイル名 `pipelines/ci.json.2026-11-20_01.state`。run ID に `-`・`_`・`.` を含む。`exit_code` が 32 ビット符号付き整数の最小値)。

```json
{
  "format_version": 1,
  "definition_key": "pipelines/ci.json",
  "definition_path": "pipelines/ci.json",
  "pipeline_name": "ci",
  "run_id": "2026-11-20_01",
  "job_order": [
    "build",
    "test",
    "deploy"
  ],
  "job": [
    {
      "id": "build",
      "status": "success",
      "exit_code": 0
    },
    {
      "id": "test",
      "status": "error",
      "exit_code": -2147483648
    },
    {
      "id": "deploy",
      "status": "pending"
    }
  ],
  "resume_job_id": "test",
  "end_reached": false,
  "completed": false
}
```

(8)U+2028・U+2029 と `<` `&` `>` を含む例(§4.3 のエスケープの確認用。`pipeline_name` は `x` + U+2028 + `y`、ジョブID は `a` + U+2029 + `b <&>`)。U+2028・U+2029 は 6 文字の `\u2028`・`\u2029` になり、`<` `&` `>` は生のまま。

```json
{
  "format_version": 1,
  "definition_key": "c.json",
  "definition_path": "c.json",
  "pipeline_name": "x\u2028y",
  "job_order": [
    "a\u2029b <&>"
  ],
  "job": [
    {
      "id": "a\u2029b <&>",
      "status": "pending"
    }
  ],
  "end_reached": false,
  "completed": false
}
```

(9)`definition_path` に孤立サロゲートを含む入力(例: Windows で、ファイル名に孤立サロゲート U+D800 を含む `dir/` + U+D800 + `.json` を `--config` に渡した場合。Unix の不正なバイトの扱いは未確認。§4.3)。書き出す前に、孤立サロゲートが U+FFFD(1 文字)に置き換わる。置き換わるのは**中身の `definition_key` / `definition_path` だけ**で、ステートファイルの実際の名前は元のパスのまま。

```json
{
  "format_version": 1,
  "definition_key": "dir/�.json",
  "definition_path": "dir/�.json",
  "pipeline_name": "nightly",
  "job_order": [
    "a",
    "b"
  ],
  "job": [
    {
      "id": "a",
      "status": "success",
      "exit_code": 0
    },
    {
      "id": "b",
      "status": "pending"
    }
  ],
  "end_reached": false,
  "completed": false
}
```


---

## 5. 書き込み(`StateStore.Write`)の手順と原子性

入力: 定義ファイルのパス、run ID、ステート、耐久性(`Durability.Durable` / `Durability.Fast`)。同期の関数で、失敗は例外でなく値(`StateFail`。中身の `StateError` の Kind = `WriteFailed`)で返す(型は §13。[02](02-architecture.md))。

```text
1. data = StateSerializer.ToUtf8(state)          # §4.3。失敗(通常は起きない)なら WriteFailed(理由=「内容を JSON にできません」)
2. path = ステートファイルの場所(§1)
3. 一時ファイルを、同じディレクトリに、「既存なら失敗する」新規作成で作る
     fs = new FileStream(tmpPath, new FileStreamOptions {
              Mode = FileMode.CreateNew, Access = FileAccess.Write, Share = FileShare.None,
              Options = FileOptions.None,
              UnixCreateMode = (UnixFileMode)0b110_110_110 /* 0666。Windows では設定しない(下) */ })
     名前 = ".<ステートファイル名>.tmp.<Environment.ProcessId>.<UTC の Unix エポックからのナノ秒>.<連番>"
     名前が既存(IOException かつ File.Exists(tmpPath))なら、連番を進めて作り直す。最大 16 回。それでも作れなければ WriteFailed
     連番 = プロセス内の通し番号(Interlocked.Increment(ref s_seq))+ 試行回数(桁あふれは折り返してよい)
4. 全内容を書く: fs.Write(data)(FileStream.Write(ReadOnlySpan<byte>) は全部を書くか例外)。続けて fs.Flush(flushToDisk: durable)
5. Durable のときだけ、fs.Flush(true) が内容を記憶装置まで同期する(下の「確認」)。Fast のときは fs.Flush(false) までで、同期を省く
6. fs.Dispose()(一時ファイルを閉じる。Windows は、開いたままだと移せない・消せない。Dispose が投げた例外も失敗として扱う)
7. File.Move(tmpPath, path, overwrite: true) で置き換える
8. 3〜7 のどこかで失敗(IOException、UnauthorizedAccessException、その他の例外)したら: fs が開いたままなら Dispose(失敗は無視)、
     自分が作った一時ファイルを File.Delete(失敗は無視)し、WriteFailed(理由=「書き込みに失敗しました」)。下位の例外は InnerException に入れ、表示しない
9. 成功し、かつ Durable のとき:
     UNIX 系では、ディレクトリを同期する(下。失敗は無視。Windows では何もしない)
     同じ名前の型の一時ファイルのうち、更新から 24 時間以上たった通常のファイルを消す(失敗は無視)
```

API の選定と注意:

- **`Path.GetTempFileName()` / `File.Create` / `FileMode.Create` を使わない**: 一時ファイルは、ステートファイルと同じディレクトリに、この名前の規則で作る(別のディレクトリの一時ファイルは、別のファイルシステムになりうる。そうなると `File.Move` はコピーと削除になり、原子的でない。確認(出典: `File.Move` の API ページ): 「ディスクのボリュームをまたぐ移動は、コピーして、成功したらコピー元を削除するのと同じ」)。`FileMode.Create`・`Truncate` は既存のファイルを黙って上書きする。`FileMode.CreateNew` は、既存のとき `IOException` を投げる(`FileMode` の一般の契約。この文書では取得して確認していない: 未確認)。
- **`FileShare.None`**: 一時ファイルを、書いている最中にほかのプロセスが開けないようにする(Windows で意味を持つ。Unix では、助言のロックが掛かる程度、と理解している。未確認)。
- **`File.Move(string, string, bool overwrite)` を採る(`File.Replace` を採らない)**。確認(出典: `File.Move` の API ページ): `overwrite = true` なら、移動先が既にあれば置き換える。.NET Core 3.0 以降の版で使える(ページの記載)。移動先が既にあって `overwrite = false` なら `IOException`。`overwrite = true` で OS が移動先への排他アクセスを取れないときは `UnauthorizedAccessException`。コピー元と先が同じなら例外を投げない。ページには、この置き換えが**原子的である**という記載が無い(ページが原子的でないと書いているのは、`Copy` + `Delete` で代用する方法のほう)。実装(Unix では `rename(2)`、Windows では `MoveFileEx` と理解しているが、取得して確認していない)の原子性は未確認。`File.Replace` を採らない理由: (a) 置き換え先が無いと失敗する(最初の書き込みで使えない)と理解しており、分岐が要る(未確認)。(b) Windows の `ReplaceFile` は、置き換え先の属性・ACL を引き継ぐと理解している(この仕様は権限を引き継がない。下)。(c) バックアップのパラメーターなど、不要な機能を持つ。いずれも未確認。
- **`UnixCreateMode`**。確認(出典: `FileStreamOptions.UnixCreateMode` の API ページ): `UnixFileMode?` 型のプロパティで、「新しいファイルを作るときに使う Unix のファイルモード」。.NET 7 以降の版で使える。`set` は Windows で使えない(`UnsupportedOSPlatform("windows")`)ので、`OperatingSystem.IsWindows()` で分岐して、Windows ではこのプロパティに触れない(`CA1416` の警告を、警告をエラーにする設定のもとで避けるため。[11](11-quality-and-build.md))。ページには、**既定の値**(`null` のときのモード)と、**umask との関係**の記載が無い(未確認)。この仕様は、パーミッションを明示的に変更せず、OS 標準の新規作成の権限(`0666` から umask を引いた値)にする(下)ので、`0666`(`UserRead|UserWrite|GroupRead|GroupWrite|OtherRead|OtherWrite`)を明示する。umask が `open(2)` で適用されること(`open` の `mode` 引数は umask との論理積を取る)は POSIX の一般の規則で、.NET がこの値を `open(2)` にそのまま渡すなら、結果は `0666 & ~umask` になるはず。**この実装の挙動は未確認なので、受け入れテスト(Unix の 2 つの OS)で固定する**: umask 022 で書いたステートファイルの `File.GetUnixFileMode(path)` が `UserRead|UserWrite|GroupRead|OtherRead`(`0644`)、umask 077 なら `UserRead|UserWrite`(`0600`)になること。もし、`UnixCreateMode` を設定しても umask が掛からない(`0666` のまま)なら、設定せずに(`null` のまま)作った場合の結果を測り、どちらかが `0666 & ~umask` になる方を採る。
- **パーミッションは明示的に変更しない**(`File.SetUnixFileMode`・`chmod` を使わない)。既存のステートファイルを置き換えるときも、権限は引き継がず、新しい一時ファイルの権限になる(`File.Move` の上書きは、コピー元のファイルがそのまま移動先になる、と理解している。未確認)。所有者の変更・ACL の操作はしない。Windows では、`FileSecurity` を明示せず、親のディレクトリから継承される既定の ACL になる(未確認)。
- **`FileStream.Flush(bool flushToDisk)`**。確認(出典: `FileStream.Flush` の API ページ): `Flush(true)` は「バッファを消し、バッファのデータをファイルに書き、**すべての中間のファイルバッファも消す**」。「中間のファイルバッファのデータがディスクに書かれたことを確実にしたいときにこの過負荷を使う」。`Flush()` は、OS の I/O バッファも消す、と書かれている(同ページ)。ページには、Unix の `fsync(2)` や Windows の `FlushFileBuffers` を呼ぶと明記されていない(未確認。実装は、`flushToDisk: true` でそれらを呼ぶと理解している)。この仕様は、Durable のとき `Flush(true)` を、Fast のとき `Flush(false)`(または `Flush()`)を呼ぶ。
- 書き込みは `FileStream.Write(ReadOnlySpan<byte>)`(または `Write(byte[], int, int)`)。バッファを持つ `FileStream` なので、`Flush` を忘れると、`Dispose` まで内容が書かれない(`Dispose` は `Flush()` するが、同期はしない)。必ず `Flush(flushToDisk: durable)` を `Dispose` の前に呼ぶ。`StreamWriter` を重ねると、その内部のバッファの `Flush` も要る(使わない)。
- ファイル記述子・ハンドルは、子プロセスに引き継がれない(.NET の `FileStream` の既定。`FileOptions` に継承の指定が無い。この文書では取得して確認していない: 未確認)。
- 時刻は `StateStoreOptions.Now()`(`DateTimeOffset`)の `UtcTicks` から `DateTime.UnixEpoch.Ticks` を引いて 100 を掛けた値(ナノ秒単位、分解能 100 ナノ秒)でよい。クロックが戻っても、名前の衝突は連番の再試行(最大 16 回)で避ける。

**ディレクトリの同期(手順 9)**: .NET には、ディレクトリの `fsync` を行う管理 API が無い、と理解している(`File.OpenHandle` はディレクトリを開こうとすると `UnauthorizedAccessException` になる、と理解している。いずれも公式ドキュメントでは確認していない: 未確認)。この仕様は、UNIX 系で、`libc` の `open`・`fsync`・`close` を P/Invoke で呼ぶことに決める(`Conductor.Native.UnixNative` に置く。§13.1)。根拠: 手順 9 は「失敗は無視」の、耐久性のための最善の努力で、P/Invoke が使えない環境でも、他の挙動(ファイルの内容・原子性)は変わらない。

```csharp
// 置き場所は Conductor.Native.UnixNative(src/Conductor.Runtime/Native/UnixNative.cs。07 §7.2.1 の規定。P/Invoke は 1 か所)。ここでは、ディレクトリの同期のメソッドの形だけを示す(§13.1)
namespace Conductor.Native;
internal static partial class UnixNative   // 抜粋
{
    [LibraryImport("libc", EntryPoint = "open", StringMarshalling = StringMarshalling.Utf8)]
    private static partial int OpenDirectory(string path, int flags);        // 戻り値 -1 は失敗
    [LibraryImport("libc", EntryPoint = "fsync")]
    private static partial int FsyncFd(int fd);
    [LibraryImport("libc", EntryPoint = "close")]
    private static partial int CloseFd(int fd);

    private const int ORdOnly = 0;                                  // O_RDONLY は 0(Linux・macOS)

    // 失敗はすべて無視する(例外も握りつぶす: DllNotFoundException、EntryPointNotFoundException、その他)
    internal static void SyncDirectory(string directory) { /* Open → Fsync → Close。失敗は無視。Windows では呼ばない(呼び側が StateStoreOptions.SyncDirectory で決める) */ }
}
```

- `directory` が空のとき `"."`(§1)。`flags` は `O_RDONLY` だけ(`O_DIRECTORY`・`O_CLOEXEC` は値が OS で違うので使わない。ディレクトリを `O_RDONLY` で開いて `fsync` できることは、Linux・macOS で一般に使われる方法で、この文書では確認していない: 未確認)。`open` は可変長引数の関数だが、2 引数(`path`, `flags`)だけで呼ぶので、`mode` は渡さない。ARM64 の macOS などで、可変長引数の呼び出し規約が固定長と違う件は、可変長の部分(この呼び出しでは空)にだけ関わると理解している(未確認)。
- `LibraryImport` は .NET 7 以降の版で使える(ソース生成の P/Invoke。この文書では取得して確認していない: 未確認)。生成されたコードのために、プロジェクトで `<AllowUnsafeBlocks>true</AllowUnsafeBlocks>` が要る(未確認。07 の `kill` の P/Invoke と同じ設定)。ライブラリ名 `"libc"` の解決は、07 §7.2.1 の `DllImportResolver`(`libc.so.6`、macOS は `/usr/lib/libSystem.B.dylib`)に任せる(同じ型の中の別のメソッドも同じ解決を使う。未確認)。
- `SetLastError` は使わない(失敗を無視するので `errno` を読まない)。
- ディレクトリの同期は、トリミング・AOT の構成でも使える(`LibraryImport` はリフレクションを使わない。ただし未確認)。
- この P/Invoke を実装しない(省略する)場合の影響: 電源断・カーネルの異常終了で、置き換え(`rename`)がディスクに反映される前に失われうる(ファイルの内容は `Flush(true)` で同期済みなので、古い完全な内容か、新しい完全な内容のどちらかが残る)。「完全な内容のどちらか」という原子性は保たれる。したがって、省略は、この仕様の「必須の性質」(§3)の違反にはならず、「耐久性の強さ」だけを弱める。採用する(P/Invoke を行う)ことを標準とする。

性質:

- **原子的**: 読み手は、直前の完全な内容か、新しい完全な内容のどちらかだけを見る。途中の内容は見えない。どの段階で失敗しても、ステートファイルは直前の完全な内容のまま。UNIX 系の `rename(2)` が原子的であること、`File.Move(overwrite: true)` がそれを使うことは、この文書では確認していない(未確認)。Windows での原子性も未確認(§5.1)。
- 置き換え先がディレクトリなど、名前の付け替えが失敗するときは `WriteFailed`。ステートは変わらず、一時ファイルも残らない(手順 8 で消す。置き換え先がディレクトリのときの例外の型は、`IOException` か `UnauthorizedAccessException` と理解している。未確認。どちらでも `WriteFailed`)。書けないディレクトリ・存在しないディレクトリ(`DirectoryNotFoundException`)も `WriteFailed`。
- 残った一時ファイル(プロセスが落ちたとき)は、読み込みに影響しない。`Durable` の書き込みの成功後に、24 時間以上古いものだけ消える。`Fast` では消さない。
- 「同じ名前の型」= 名前が `.<ステートファイル名>.tmp.` で始まるもの。後始末は、`Directory.EnumerateFileSystemEntries(dir)` で名前を列挙し(`searchPattern` を使わない。`*`・`?` を含むファイル名で、パターンの解釈と食い違うため)、`Path.GetFileName(entry).StartsWith(prefix, StringComparison.Ordinal)` かつ、通常のファイル(ディレクトリでない: `FileInfo.Attributes` に `FileAttributes.Directory` が無い。シンボリックリンクでない: `FileSystemInfo.LinkTarget` が `null`。`LinkTarget` は .NET 6 以降。未確認)で、`File.GetLastWriteTimeUtc(entry)` が `Now() - 24 時間`(`StateStoreOptions.Now`) 以前のものを `File.Delete` する(エラーはすべて無視)。列挙は `foreach` の中で例外が出うる(列挙の途中でファイルが消える、など)ので、全体を `try/catch` で囲み、無視する。
- 排他制御(ファイルロック)はしない(§11)。

### 5.1 Windows での置き換えの注意

- `File.Move(overwrite: true)` は、移動先が既にあっても置き換える、と書かれている(確認(出典: `File.Move` の API ページ))。Windows での実機の確認はしていない(**未確認**。Windows の CI のテストで、既存のファイルへの置き換えが成功することと、置き換え中に読み手が「古い完全な内容」か「新しい完全な内容」だけを見ることを確認する)。
- 一時ファイルは、`File.Move` の前に `Dispose` する(開いたままだと移せない。`FileShare.None` で開いているので、自分の `FileStream` が生きているあいだは、自分でも移せない)。
- 置き換え先のステートファイルを、ほかのプロセス(例: 同時に走る `watch`、ウイルス対策ソフトの走査)が、`FileShare.Delete` を許さない設定で開いている間は、`File.Move(overwrite: true)` が `UnauthorizedAccessException`(OS が移動先への排他アクセスを取れない。ページの例外の記載)または `IOException` で失敗しうる(**未確認**)。失敗したら `WriteFailed`(65)にする。再試行はしない(仕様として、読み手との同時の実行は想定しない。§11)。conductor 自身の読み込みは、`FileShare.ReadWrite | FileShare.Delete` で開く(§6.1 の 1)ので、conductor どうし(`watch` を含む)は、置き換えを妨げない。
- ディレクトリの同期は行わない(Windows では、ディレクトリのハンドルを `FlushFileBuffers` で同期する方法が一般的でないと理解している。未確認)。
- 一時ファイルの後始末は、UNIX 系と同じ(24 時間以上古い通常のファイル。`FileInfo.LinkTarget` が `null`)。
- Windows の `UnixCreateMode` は使えない(`set` が `UnsupportedOSPlatform("windows")`)。ACL は親から継承される既定のまま。

### 5.2 いつ書くか(書き込みの契機)

| 契機 | 内容 | 耐久性 |
|---|---|---|
| ステートが空で、最初のジョブが `Run` と決まった直後(ジョブを起動する前) | 初期ステート。完了のまま残ったステートがあっても、同じ名前のファイルを初期ステートで**上書きする** | `Fast`(同期を省く) |
| ジョブの最終の結果が出た後(リトライを含めて 1 回だけ) | [05](05-execution-flow.md) §3.3 の結果の記録。起動前の中断・待機中の中断も、この契機で書く(ジョブの状態は変わらず、`resume_job_id` が付く) | `Durable` |
| スキップが決まったとき([05](05-execution-flow.md) §3.4 の記録が元と異なるときだけ) | スキップの記録 | `Durable` |
| `pipeline` のジョブのあいだの中断 | [05](05-execution-flow.md) §3.5 の中断の記録 | `Durable` |
| 完了(`completed = true`)になったとき | 上の契機の書き込みに含まれる(完了のステートも同じ手順) | `Durable` |

- `Reject`(66)では何も書かない(初期ステートも)。
- 書き込みの失敗(65)の表示の末尾に、ジョブの起動前なら `(ジョブは起動していません / the job was not started)`、起動後なら `(ジョブは実行されました / the job has already been executed)` が付く(ロケールに従う)。
- 結果を書けなかったときは、以降のジョブを起動せず、65 で終わる。

---

## 6. 読み込み(`StateStore.Read` / `ReadRaw`)の手順と異常系

### 6.1 手順

入力: 定義ファイルのパス、run ID(ファイル名側の期待値)。上から順に、最初に当てはまる結果で終わる。`Read` は完了のステートを「空」にし、`ReadRaw`(`watch` が使う)は 14 を行わない。どちらも同期の関数で、成功は `StateReadOk(PipelineState?)`(空のときは中身が `null`)、失敗は `StateFail(StateError)` を値で返す(§6.2、§13)。例外を送出しない(下位の `IOException` などは、この関数の内側で狭く捕まえて `StateFail` に変える)。

| 順 | 処理 | 結果 |
|---|---|---|
| 1 | ファイルを開く: `new FileStream(path, new FileStreamOptions { Mode = FileMode.Open, Access = FileAccess.Read, Share = FileShare.ReadWrite \| FileShare.Delete, Options = FileOptions.None })` | 存在しない(`FileNotFoundException`、または `DirectoryNotFoundException`。ただし、親のパスが通常のファイルとして存在するときは `unreadable`(下の注))→ **空**(ステート無し)。その他の開く失敗(`UnauthorizedAccessException`(権限、およびディレクトリを開こうとしたとき)、`IOException`、`PathTooLongException`、`NotSupportedException`、`ArgumentException`、`SecurityException`)→ `unreadable` |
| 2 | 開いた `FileStream` の大きさを確認(`CanSeek` なら `Length`。ハンドルへの問い合わせで、パスの再解決をしない) | `Length` の取得失敗(`IOException` など)→ `unreadable`。1 MiB(1,048,576 バイト)**超** → `tooLarge`。`CanSeek` が偽(パイプ・特殊ファイル)なら、大きさは不明として次へ(順 3 で止める) |
| 3 | 上限 + 1 バイトまで読む(§6.1.1)。終わりのない入力でも止まる | 読み取り失敗(`IOException`)→ `unreadable`。読めたバイト数が 1,048,576 を超える → `tooLarge`(ちょうど上限は受理) |
| 4 | 長さ 0 | **空** |
| 5 | UTF-8 として厳密に解釈(§6.1.2)。BOM を**取り除かない** | 失敗 → `corrupt`(理由 `UTF-8 の文字列ではありません`) |
| 6 | JSON として解析(下の 6a〜6c を順に) | 失敗 → `corrupt`(6a・6b は `JSON として読めません`、6c は `キーが重複しています`) |
| 7 | `format_version` の確認(**ほかの検査より先**。将来の版が、未知の項目で corrupt にならないように) | 項目無し → `corrupt`(`format_version がありません`)。整数以外 → `corrupt`(`format_version が整数ではありません`)。整数で 1 以外(0 や 2 や巨大な整数を含む)→ `versionMismatch`(`対応していない版です(reset が必要です)`) |
| 8 | 最上位に未知の項目がある | `corrupt`(`知らないキーがあります`) |
| 9 | `run_id` の取り出しと照合 | 型が文字列でない(`null` を含む)→ `corrupt`(`値の型が誤っています`)。中身の `run_id`(無ければ「なし」)がファイル名の run ID(無ければ「なし」)と一致しない → `corrupt`(`run_id がファイル名の run ID と一致しません`) |
| 10 | 必須項目の取り出し: `definition_key`、`definition_path`、`pipeline_name`、`end_reached`、`completed`、`job_order`、`job`(**この順に 1 項目ずつ**、無い/型が違うを判定する) | 無い → `corrupt`(`必須のキーがありません`)。型が違う → `corrupt`(`値の型が誤っています`)。`job_order` は文字列の配列(要素に文字列以外があれば型の誤り)、`job` は配列(要素がオブジェクトかは順 11 で見る) |
| 11 | `job` の各要素を、**配列の順に 1 つずつ**、次の順で検査する(下の表) | 下の表のとおり、すべて `corrupt` |
| 12 | `resume_job_id` の型 | 文字列でない(`null` を含む)→ `corrupt`(`値の型が誤っています`) |
| 13 | 内容の矛盾(この順): ジョブIDの重複 → `job` の並びと `job_order` の不一致 → `resume_job_id` が `job_order` に無い → `completed` かつ `resume_job_id` あり | いずれも `corrupt`(それぞれ `ジョブIDが重複しています`、`job の並びが job_order と一致しません`、`resume_job_id が job_order にありません`、`completed と resume_job_id が両立しません`) |
| 14 | `Read` のみ: `completed = true` | **空**(完了のまま残ったステート。ファイルは消さない) |

順 1 の注と補足:

- **`FileShare.ReadWrite | FileShare.Delete`**: 読んでいるあいだに、書き手が `File.Move(overwrite: true)` で置き換えても、Windows で書き手が失敗しないようにする(`FileShare.Delete` が無いと、読み手が開いている間、置き換えと削除が拒否される。Windows の一般の性質と理解している。未確認)。Unix では、開いているファイルを置き換えても、読み手は元の内容(古い版のファイルの実体)を読み続ける。
- 親のパスが通常のファイルのとき(`file/x` を開く。`ENOTDIR` に当たる)は、`DirectoryNotFoundException` になる、と理解している(未確認。公式ドキュメントには、`DirectoryNotFoundException` が「パスが無効」のときと書かれている)。`DirectoryNotFoundException` を「空」にする場合は、次の確認を足す: ステートファイルの親ディレクトリのパスを `File.Exists` した結果が真(親のパスが通常のファイル)なら、`unreadable` にする。親が存在しないだけ(`Directory.Exists` が偽)なら「空」(ステートの親のディレクトリが無い = 何も書かれていない)。この区別が、期待の挙動と一致するかは未確認で、受け入れテストで固定する。
- ディレクトリを指すパスを開く: Unix でも Windows でも `UnauthorizedAccessException` になる、と理解している(未確認)。どちらも `unreadable`(順 1 の「その他の開く失敗」)。ディレクトリでも、例外にならずに開けて、順 3 の読み取りで失敗する環境がありうる(その場合の例外は `IOException`/`UnauthorizedAccessException` と理解している。未確認)ので、順 3 の読み取り失敗も `unreadable` にする。いずれも、ファイルは変更しない。
- `File.ReadAllBytes` は使わない(ファイル全体を読む。上限を超える入力で止まらない)。`File.ReadAllText`・`StreamReader`・`File.OpenText` は使わない(BOM による自動のエンコーディング判定、UTF-16・UTF-32 の判定、不正バイトの黙った置換が起きうる。順 5・6a の規定と衝突する。`File.ReadAllText` が BOM でエンコーディングを判定することは一般に知られているが、この文書では取得して確認していない: 未確認)。

#### 6.1.1 順 3 の読み取り

```csharp
// buffer は byte[1048576 + 1]。Read(byte[], int, int) は、要求より少なく返すことがありうるので、ループで埋める
int total = 0;
while (total < buffer.Length)
{
    int n = fs.Read(buffer, total, buffer.Length - total);
    if (n == 0) break;          // 終端
    total += n;
}
// total > 1048576 なら tooLarge。そうでなければ buffer[0..total] が内容
```

- 上限 + 1 バイトまで読んで止める。終わりのない入力(`/dev/zero` など。`Length` が 0 または `CanSeek` が偽)でも、1 MiB + 1 バイトで止まる。`Stream.Read` が `0` を返したら終端(契約として知られているが、この文書では取得して確認していない: 未確認)。
- `FileStream` の `bufferSize` は既定でよい(読み取りの大きさの要求(最大 1 MiB + 1)が、内部のバッファより大きければ、直接読まれる)。要求より少なく返ることがあるのは、一般の `Stream.Read` の契約なので、ループで埋める。
- 名前付きパイプ(FIFO)を開くと、書き手が現れるまで止まる(`open(2)` の一般の性質。この仕様の確認はしていない: 未確認)。ステートのパスが FIFO であることは想定しない。

#### 6.1.2 順 5 のデコード

```csharp
private static readonly UTF8Encoding StrictUtf8 = new(encoderShouldEmitUTF8Identifier: false, throwOnInvalidBytes: true);

string text;
try { text = StrictUtf8.GetString(buffer, 0, total); }
catch (ArgumentException) { /* DecoderFallbackException も ArgumentException の派生型 */ → corrupt(UTF-8 の文字列ではありません) }
```

- **`new UTF8Encoding(false, true)` を使う**。確認(出典: `UTF8Encoding(Boolean, Boolean)` の API ページ): `throwOnInvalidBytes = true` なら、不正なバイト列の検出で例外を投げる(ページは `ArgumentException` と書く)。`false`(`Encoding.UTF8` と、引数なしの `new UTF8Encoding()` の既定)は、例外を投げず、不正な部分を置換する(ページ: 「セキュリティのため、`throwOnInvalidBytes` を持つ構造体で、エラーの検出を有効にすることを勧める」)。**`Encoding.UTF8.GetString` を使わない**(黙って U+FFFD に置換するので、不正な UTF-8 を検出できず、順 5 が働かない)。
- 復号の例外は、`DecoderFallbackException`(`ArgumentException` の派生型。復号の失敗で投げられる型は、ページの例では符号化側の `EncoderFallbackException` だけが示されている。復号側の型は未確認)。したがって、`catch (ArgumentException)` で受ける(`DecoderFallbackException` に限定しない)。例外の `Message` は表示に使わない。
- **BOM を取り除かない**。`GetString` は BOM(`EF BB BF`)を取り除かず、U+FEFF の 1 文字として残す、と理解している(`GetPreamble` の設定が `GetBytes` に影響しないことは、ページに書かれている(確認)。`GetString` が BOM を取り除かないことは、ページに記載が無い: 未確認)。このため、**`GetString` の結果の先頭が U+FEFF か**を、6a の前処理として確かめる(§6.5)。BOM を取り除く API(`StreamReader`、`File.ReadAllText`、`File.OpenText`、`JsonSerializer.Deserialize(Stream)`。後者が BOM を許す扱いは、確認していない: 未確認)を使わない。`GetString` が万一 BOM を取り除くなら、BOM 付きのファイルが受理されてしまうので、§6.6 の BOM のテストで検出する。
- 厳密な復号は、不正な UTF-8 を拒否する(`ED A0 80`(サロゲートの符号化)、`C0 80`(最短でない符号化)、`F4 90 80 80`(U+10FFFF 超)、途中で切れた `E3 81`)。.NET の復号がこれらを不正とみなすことは、一般に知られたこと(Unicode の標準の定義)と理解しているが、この環境では実行して確かめていない(未確認)。§6.6 のテストで固定する。
- NUL は UTF-8 としては有効で、JSON の構文の検査(6a)で拒否される(文字列の外に現れれば構文の誤り、文字列の中に現れれば制御文字の誤り)。

順 6 の細目(この順。最初の失敗で終わる):

| 細目 | 処理 | 失敗の理由 |
|---|---|---|
| 6a | §6.5 の読み取り(自前の `Conductor.Json` の読み取りと、深さの検査)で、全体を構文解析する。失敗するもの: 構文の誤り、BOM(先頭の U+FEFF)、値の後ろの余分な文字(2 つ目の値を含む)、空白だけの内容、制御文字(U+0000〜U+001F)の文字列内への直接の混入、`\u` エスケープの誤り、**対にならないサロゲートのエスケープ**(`"\ud800"`)、`NaN`・`Infinity`・`-Infinity`、先頭が `0` の整数(`01`)、入れ子が深すぎるもの(§6.5 の深さの上限 32 を超える) | `JSON として読めません` |
| 6b | 最上位の値がオブジェクトか(`null`、配列、数値、文字列、真偽の最上位を拒否する) | `JSON として読めません` |
| 6c | どのオブジェクトにも、同じキーが 2 回以上現れないか(最上位と `job` の各要素を含む全オブジェクト。エスケープを展開した後の文字列で、序数比較で比べる。`"id"` と `"id"` は同じキー) | `キーが重複しています` |

6a が 6c より先であることは、仕様として固定する(重複キーが先にあっても、後ろに構文の誤りがあれば、`JSON として読めません` になる)。空白の定義は JSON の空白(スペース、`\t`、`\n`、`\r`)。BOM・NBSP・垂直タブ・改ページは空白でないので、6a で拒否される。書き出しは BOM を付けない。他のツールが BOM 付きで保存した場合は壊れたステートとして 65 になり、`reset` で回復する。

順 7 の「整数」の定義: 値の字句が `-?(0|[1-9][0-9]*)` に一致するもの(小数点・指数を含まない)。`1.0`、`1e0`、`"1"`、`true`、`null`、配列、オブジェクトは**整数以外**(`format_version が整数ではありません`)。`-0` と `0`、`2`、`-1`、`99999999999999999999`、5000 桁の数字は整数で 1 ではないので `versionMismatch`。`1` だけが受理される(`01` は JSON の構文として誤り(6a))。整数の判定は、読み取りが返す値の種類(整数の字句 / それ以外の数 / 真偽 / 文字列 / `null` …。§6.5)で行い、受理は「整数の字句で、値が 1」のときだけとする。64 ビットを超える整数の字句でも `versionMismatch`(03 章の決定: 字句の桁数と `long.TryParse` の判定だけで足り、`BigInteger`・`decimal` を使わない。木では `HugeIntNode`)。

順 11 の `corrupt` の理由文(個別。いずれも種類は `corrupt`。1 つの要素の中の検査の順に上から):

| 順 | 違反 | 理由 |
|---|---|---|
| a | 要素がオブジェクトでない(`null` を含む) | `値の型が誤っています` |
| b | 未知のキー(大文字小文字も区別。`ID`、`Status` は未知。`__proto__` も未知のキー) | `知らないキーがあります` |
| c | `id` が無い | `必須のキーがありません` |
| d | `id` が文字列でない | `値の型が誤っています` |
| e | `status` が無い | `必須のキーがありません` |
| f | `status` が文字列でない | `値の型が誤っています` |
| g | `status` が 5 つの値以外 | `status の値が誤っています` |
| h | `exit_code` があり、整数でない(`1.0`、`1e2`、文字列、`null`、`true` / `false` を含む)、または 32 ビット符号付き整数の範囲外(§6.5 の字句による判定) | `値の型が誤っています` |
| i | `skip_reason` があり、文字列でない(`null` を含む) | `値の型が誤っています` |
| j | `skip_reason` が 2 つの値以外 | `skip_reason の値が誤っています` |
| k | `status = skipped` と `skip_reason` の有無の不整合 | `skip_reason が status と合いません` |
| l | `status` が pending / skipped なのに `exit_code` あり | `exit_code が status と合いません` |

- `ReadRaw`(`watch` が使う)は、14 を行わない(完了のステートもそのまま返す)。1〜13 は同じ。
- 検査の `run_id` 照合(9)は、`completed` の判定(14)より前なので、完了のステートでも run ID の食い違いは `corrupt`。
- 読み取りでは、ファイルを**消さない・書き換えない**。
- 「空」= ステートが無いのと同じ扱い。呼び出し側は、すべて pending のステートとして扱い、最初の `Run` の前に初期ステートを書く。
- 空のステートを返す関数の戻り値は `null`(`PipelineState?`)。
- 型の判定は、読み取りが返す木の値の種類(§6.5)で行う。`true`/`false` を整数として受理しない(C# では `bool` と `int` は別の型で取り違えは起きないが、「木の値の種類」を型の判定の唯一の根拠にする)。
- `FileStream` は、`using`(または `try/finally` の `Dispose`)で必ず閉じる。

### 6.2 失敗の表示と終了コード

すべて終了コード 65。標準エラーの 1 行(固定の文 + パス + 固定の理由。入力の値・部品のエラー文は含めない):

```text
conductor: error: <ステートファイルのパス>: <kind の文>: <理由>[; <案内>]
```

| kind | 英語の文 | 日本語の文 | 理由の例(常に日本語) | 案内を付けるか |
|---|---|---|---|---|
| `unreadable` | `cannot read state file` | `ステートファイルを読めません` | `開く、または読み取ることができません` | 付ける |
| `tooLarge` | `state file is too large` | `ステートファイルが大きすぎます` | `大きさが上限(1 MiB)を超えています` | 付ける |
| `corrupt` | `state file is corrupt` | `ステートファイルが壊れています` | §6.4 の各理由 | 付ける |
| `versionMismatch` | `state file version is not supported` | `ステートファイルの版が合いません` | `対応していない版です(reset が必要です)` | 付ける |
| `writeFailed` | `cannot write state file` | `ステートファイルを書き込めません` | `書き込みに失敗しました` / `内容を JSON にできません` | 付けない(§5.2 の `(ジョブは…)` が付く) |
| `removeFailed` | `cannot remove state file` | `ステートファイルを削除できません` | `削除に失敗しました` | 付けない |

案内(読み込みのときだけ): run ID なし → ``; run `conductor reset` to discard the state``(日本語 ``; `conductor reset` でステートを初期化できます``)、run ID あり → ``; run `conductor reset --run-id <ID>` to discard the state``(`<ID>` は文字どおりの `<ID>`。日本語は ``; `conductor reset --run-id <ID>` でステートを初期化できます``)。

実例(英語ロケール):

```text
conductor: error: c2.json.state: state file is corrupt: JSON として読めません; run `conductor reset` to discard the state
conductor: error: c2.json.state: state file is corrupt: キーが重複しています; run `conductor reset` to discard the state
conductor: error: c2.json.state: state file version is not supported: 対応していない版です(reset が必要です); run `conductor reset` to discard the state
conductor: error: c2.json.r3.state: state file is corrupt: run_id がファイル名の run ID と一致しません; run `conductor reset --run-id <ID>` to discard the state
```

表示に使うパス(`<ステートファイルのパス>`)の文字列化(孤立サロゲートのエスケープ表示など)は [08](08-output-and-i18n.md)。

### 6.3 異常系の挙動の一覧(`run` / `pipeline`)

| 状況 | 結果 | ジョブの起動 | ステート |
|---|---|---|---|
| ファイルが無い | 空として続行 | する | 最初の `Run` の前に初期ステートを書く |
| 長さ 0 | 空として続行 | する | 同上(上書き) |
| `completed = true` | 空として続行(定義との照合もしない) | する | 初期ステートで上書き |
| 読めない(権限等) | 65 | しない | 変更なし |
| 1 MiB 超 | 65 | しない | 変更なし |
| UTF-8 でない・JSON の構文の誤り・BOM・最上位がオブジェクトでない・深すぎる入れ子・キーの重複・未知のキー・必須項目欠落・型の誤り・内容の矛盾 | 65(`corrupt`) | しない | 変更なし |
| `format_version` が 1 以外の整数 | 65(`versionMismatch`) | しない | 変更なし |
| 中身の `run_id` とファイル名の run ID が不一致、または片方だけ有る | 65(`corrupt`) | しない | 変更なし |
| 定義と食い違い(§7) | 65 | しない | 変更なし |
| 書き込み・削除の失敗 | 65 | §5.2 の表のとおり | 直前の内容のまま(書き込み)/ 残る(削除) |

どの異常でも、回復は `conductor reset`(または `--run-id`、`--all`)。`reset` は読まず検査もしないため、どんなファイルでも消せる(§8)。

### 6.4 理由文の全件表

読み込みで出しうる `corrupt` / `versionMismatch` / `unreadable` / `tooLarge` の理由文と、書き込み・削除の理由文の**全件**(23 行)。表の文言は、ロケールに関わらず日本語の固定文である。

| # | kind | 理由文 | 出る順 | 条件 |
|---|---|---|---|---|
| 1 | unreadable | `開く、または読み取ることができません` | 1〜3 | 開く・大きさの取得・読み取りの失敗(存在しないを除く) |
| 2 | tooLarge | `大きさが上限(1 MiB)を超えています` | 2、3 | 1,048,576 バイト超 |
| 3 | corrupt | `UTF-8 の文字列ではありません` | 5 | UTF-8 として不正 |
| 4 | corrupt | `JSON として読めません` | 6a、6b | JSON の構文の誤り、BOM、余分な文字、最上位がオブジェクトでない、空白だけの内容、対にならないサロゲートのエスケープ、`NaN` / `Infinity`、入れ子の深さの超過 |
| 5 | corrupt | `キーが重複しています` | 6c | 同じキーが 2 回以上(最上位・`job` の要素を含む全オブジェクト) |
| 6 | corrupt | `format_version がありません` | 7 | キー無し |
| 7 | corrupt | `format_version が整数ではありません` | 7 | 整数の字句でない(`1.0`、`"1"`、`null`、`true` など。整数の定義は §6.1) |
| 8 | versionMismatch | `対応していない版です(reset が必要です)` | 7 | 整数で 1 以外 |
| 9 | corrupt | `知らないキーがあります` | 8、11b | 最上位、または `job` の要素に未知のキー(大文字小文字を区別) |
| 10 | corrupt | `値の型が誤っています` | 9、10、11、12 | 型の誤り(`null` を含む。任意の項目でも「無い」とは扱わない)、`exit_code` の範囲外・小数・真偽 |
| 11 | corrupt | `run_id がファイル名の run ID と一致しません` | 9 | 中身とファイル名の run ID の不一致(`StateError.Detail = RunIdMismatch`) |
| 12 | corrupt | `必須のキーがありません` | 10、11 | 必須キーが無い |
| 13 | corrupt | `status の値が誤っています` | 11 | 5 つの値以外 |
| 14 | corrupt | `skip_reason の値が誤っています` | 11 | 2 つの値以外 |
| 15 | corrupt | `skip_reason が status と合いません` | 11 | `skipped` と `skip_reason` の有無の不整合 |
| 16 | corrupt | `exit_code が status と合いません` | 11 | pending / skipped に `exit_code` |
| 17 | corrupt | `ジョブIDが重複しています` | 13 | `job` の `id` の重複 |
| 18 | corrupt | `job の並びが job_order と一致しません` | 13 | 数または順序の不一致 |
| 19 | corrupt | `resume_job_id が job_order にありません` | 13 | |
| 20 | corrupt | `completed と resume_job_id が両立しません` | 13 | |
| 21 | writeFailed | `書き込みに失敗しました` | 書き込み | 一時ファイル・書き込み・同期・閉じる・置き換えの失敗 |
| 22 | writeFailed | `内容を JSON にできません` | 書き込み | JSON への変換・UTF-8 への符号化の失敗(`exit_code` の範囲の確認の失敗、孤立サロゲートを含む文字列の符号化の失敗を含む) |
| 23 | removeFailed | `削除に失敗しました` | reset | 「存在しない」以外の削除の失敗 |

- キー名は大文字小文字を区別する(`Format_Version`、`ID` は未知のキー、または必須のキーが無い、として扱う)。キーの比較は序数(`StringComparison.Ordinal`)。
- 理由文の言語は、ロケールに関わらず日本語の固定文である(`conductor: error: ...` の行の中で、種類の文と案内だけがロケールに従う)。
- 例外の `Message`・`ToString()`・`InnerException` の文を、理由にも表示にも含めない(.NET の例外メッセージは、実行環境の言語・OS で変わる)。

### 6.5 JSON の読み込み: 定義と共用する自前の読み取り(選定)

目的: 順序が固定の 14 段階の検査と、理由文の区別を、確実に満たす。

**決定: ステートの JSON は、定義・変換と共用する自前の読み取り(`Conductor.Json.JsonReader.Read(text, deferDuplicateKey: true)`。[02](02-architecture.md)、[03-definition-format.md](03-definition-format.md) の規定が正本)で読み、続けて `Limits.Check(root, LimitsKind.DepthOnly)` で深さだけを確かめる。`System.Text.Json` の `Utf8JsonReader`・`JsonDocument`・`JsonSerializer` は、製品のコードのどこでも(定義、ステート、変換とも)読み取りに使わない。** 判断の根拠を、公式ドキュメントで確認できた範囲で次に示す。

`System.Text.Json` について、確認できたこと:

| # | 内容 | 出典 |
|---|---|---|
| 1 | `Utf8JsonReader` は、前方だけ・読み取り専用で UTF-8 の JSON を読み、既定で JSON の RFC(RFC 8259)に厳密に従う | `Utf8JsonReader` の API ページ |
| 2 | 不正な JSON に当たると `JsonException` を投げる(行番号とバイト位置の基本的な情報を持つ) | 同上 |
| 3 | 最大の深さとコメントの読み飛ばしは `JsonReaderOptions` で設定する | 同上 |
| 4 | `Utf8JsonReader` は、`TokenStartIndex`(トークンの開始位置)、`ValueSpan`(トークンの生のバイト列)、`ValueIsEscaped`、`TryGetInt64`・`TryGetInt32`・`TryGetDecimal`・`TryGetDouble` などを持つ | 同上 |
| 5 | `JsonDocumentOptions` は `AllowDuplicateProperties`(重複する名前を許すか)、`AllowTrailingCommas`、`CommentHandling`、`MaxDepth`(既定の 0 は最大の深さ 64)を持つ | `JsonDocumentOptions` の API ページ |
| 6 | `JsonWriterOptions.MaxDepth` の既定(0)は最大の深さ 1000 | `JsonWriterOptions` の API ページ |

確認できなかったこと(未確認。ページに記載が無い):

| # | 内容 | 帰結 |
|---|---|---|
| 1 | `Utf8JsonReader` が先頭の BOM(`EF BB BF`)をどうするか | 先頭の BOM を拒否する規定(6a)を、読み取り側に頼れない |
| 2 | 不正な UTF-8 の列を、読み取りが例外にするか、置換するか | 順 5 を自前の厳密な復号(§6.1.2)で先にやる(`Utf8JsonReader` に頼らない) |
| 3 | `\ud800`(対にならないサロゲートのエスケープ)を、読み取りが拒否するか、`GetString()` が例外を投げるか、置換するか | 6a で拒否する規定を、頼れない |
| 4 | `JsonDocument` の重複キーの既定の挙動(`AllowDuplicateProperties` の既定値と、導入された版) | 6c を頼れない。重複キーの検出と、構文の誤り(6a)が先の順序を、読み取りに任せられない |
| 5 | 数値の字句の厳密さ(先頭の `0`、`+`、`1.`、`.5`、`1e`、`-` だけ)、`NaN`・`Infinity` の扱い | 6a の規定(これらの拒否)を頼れない |
| 6 | `JsonException` の型・内容が、BOM・深さ・構文で、どう違うか | 理由文を区別できない(`JSON として読めません` だけでよいが、重複キーは別) |
| 7 | 深さの上限を超えたときの例外・位置 | 深さ 32 の上限(根を数えず、33 段目を拒否)は、読み取りのオプション(`MaxDepth`)の意味(根を数えるか。既定 64 の数え方)に頼れない |

帰結(決定の理由):

- ステートの検査は、**順序が固定の 14 段階**で、理由文が 23 件に分かれる。特に「構文の誤り(6a)が重複キー(6c)より先」「`format_version` が未知のキーより先」「BOM は構文の誤りではなく壊れたステート」は、読み取りの内部の順序・拒否の規則に、仕様として依存する。読み取りの BOM・不正 UTF-8・サロゲート・重複キー・数値の字句の挙動が、公式ドキュメントで確認できない以上、`System.Text.Json` の既定の動作に預けると、版で挙動が変わりうる。自前の読み取りなら、これらを 1 つの規則表で決められる。
- 定義・変換(03、09)が必要とする、位置(行・列)、重複キーの位置、整数の字句(`long` の範囲を超えるものの判定)、入れ子の深さ、要素数の上限も、同じ自前の読み取りで満たせる。読み取りが 1 つなので、同じ入力の受理・拒否が、定義・ステート・変換で食い違わない。
- `Utf8JsonReader` を土台にして、薄い自前の木の構築を載せる案(位置は `TokenStartIndex`、整数の字句は `ValueSpan`)は、確認できなかった点(上の表の 1〜7)を、読み取りの前後の追加の検査で補えば、実現できる。しかし、補う検査が読み取りの内部の拒否と二重になり、版による差(上の表の 4・5・7)を抱える。採らない。

自前の読み取りに、ステートが要求すること(木の型の名前(`MapNode`・`ListNode`・`StringNode`・`IntNode`・`FloatNode`・`BoolNode`・`NullNode` ほか)と署名は 02・03 の規定に従う。ここは必要な性質だけ。下の表の「整数の字句」は `IntNode`(範囲外の巨大な整数の字句を表せる形であること。型の詳細は 03)、「整数でない数」は `FloatNode`):

| # | 性質 |
|---|---|
| 1 | 入力は、`string`(順 5 で厳密に復号した UTF-16)。構文解析は、再帰でなく**明示のスタック**で行う(1 MiB の `[` の列でも、`StackOverflowException` を起こさない。`StackOverflowException` は捕捉できないので、再帰の深さの上限に頼らない。[10](10-errors-and-exit-codes.md)) |
| 2 | 返す値の種類: オブジェクト(メンバーの**並び**。キーと値の組。記述順。重複を落とさない)、配列、文字列、**整数の字句**、**整数でない数の字句**(小数点・指数を持つもの)、真偽、`null`。整数の字句は、`-?(0\|[1-9][0-9]*)` に一致するもの。`-0` は整数の字句(値 0) |
| 3 | 整数の字句から、`int` の範囲を判定できる(`TryGetInt32(out int value)`: 範囲内なら真。範囲外・64 ビットを超える字句は偽。03 章の決定: 符号を除いた桁数が 19 を超えれば `HugeIntNode`(範囲外)とし、それ以外は `long.TryParse` で判定する。`BigInteger`・`decimal`・`double` を経由しない) |
| 4 | 構文の誤り(BOM・余分な文字・空白だけ・制御文字・エスケープの誤り・対にならないサロゲートのエスケープ・`NaN`・`Infinity`・先頭が `0` の整数・末尾のカンマ・コメント)をすべて拒否する(失敗は 1 種類。位置はステートでは使わず捨てる) |
| 5 | 重複キーを**見つけても止めず**、「重複したキーがあった」ことを結果に持たせて、構文解析を続ける(後ろに構文の誤りがあれば、そちらを先に失敗にするため)。キーの比較は、エスケープを展開した後の文字列の序数比較 |
| 6 | 入れ子の深さの上限を、呼び出し側が指定できる。ステートでは 32(根の値を数えず、33 段目の `[`・`{` を拒否。03 の規定と同じ)。要素数・文字の大きさの上限は、ステートには適用しない(ファイルの大きさの上限 1 MiB が別にある) |
| 7 | 先頭の U+FEFF を無視しない(定義・変換の読み取りは、先頭の BOM を読み飛ばしてよいが、それは呼び出し側の前処理か、オプションで切り替える。ステートでは、読み取りに渡す前に、BOM を 6a として扱う) |

読み込みの手順(順 6〜13 を、この木に対して行う):

1. `text.Length > 0 && text[0] == '﻿'` なら 6a。そうでなければ、自前の読み取りを、深さの上限 32 で呼ぶ。失敗(構文の誤り・深さの超過)→ 6a。
2. 結果の最上位がオブジェクトでなければ 6b。
3. 「重複したキーがあった」が真なら 6c。
4. 順 7〜12 は、仕様の順に、最上位のオブジェクトから取り出す。未知のキーの検査(順 8)は、キーの全部が §2.1 の 10 個のどれかか、を見る(序数比較)。
5. 値の型の判定は、木の値の種類で行う(文字列・真偽・配列・オブジェクト・整数の字句・整数でない数・`null`)。整数の判定は「整数の字句」(`1.0`・`1e0` は整数でない数、`true`・`false` は真偽、`null` は `null`)。
6. `format_version`: 整数の字句で、値が 1 のときだけ受理。ほかの整数の字句(`0`、`-0`、`2`、巨大な整数)は `versionMismatch`。
7. `exit_code`: 整数の字句で、`TryGetInt32` が真のときだけ受理(`-2147483648` 〜 `2147483647`)。範囲外の整数、整数でない数、真偽、`null`、文字列は、型の誤り。`-0` は値 0。
8. `job_order` と `job` の要素も同様に取り出す(`job` の各要素がオブジェクトでなければ、順 11 の a)。

### 6.6 読み込みの単体テストで固定する入力(最低限)

この表の期待は、§6.1〜§6.5 の規則から導いた。C# で実行して確かめた結果ではない(実装時に、このテストで固定する)。

| 入力(要点) | 期待 |
|---|---|
| 正常(§4.4 の例 1〜8) | 受理。往復(読んで書く)で元と一致(例 8 の ` `・` ` のエスケープを含む。例 9 は `ReplaceLoneSurrogates` の入力で、往復では U+FFFD のまま) |
| 先頭に BOM(`EF BB BF`)+ 正常な内容 | `corrupt`(`JSON として読めません`)。`StreamReader`・`File.ReadAllText` で読むか、`GetString` が BOM を取り除くか、読み取りの前の BOM の前処理を省くと、受理されてしまう(このテストで検出する) |
| BOM + 不正な UTF-8 | `corrupt`(`UTF-8 の文字列ではありません`。順 5 が先) |
| 正常な内容の後ろに `{}` や `x` | `corrupt`(`JSON として読めません`) |
| `null`、`[]`、`1`、`"x"`、`true` | `corrupt`(`JSON として読めません`) |
| 空白だけ(`"\n"`) | `corrupt`(`JSON として読めません`) |
| 文字列の中に制御文字(生のタブ・NUL) | `corrupt`(`JSON として読めません`) |
| 入れ子の深さが 33(`{"a":[[[...]]]}` で 33 段目) | `corrupt`(`JSON として読めません`)。深さ 32 は受理(ほかの検査で別の理由になりうるので、深さだけを変えたテスト用の入力を使う) |
| 入れ子が極端に深い(1 MiB の `[` の列 + `]` の列) | `corrupt`(`JSON として読めません`)。プロセスが異常終了しない(`StackOverflowException` を起こさない) |
| `{"format_version": NaN}`、`[Infinity]` | `corrupt`(`JSON として読めません`) |
| 最上位に同じキーが 2 回(`"completed": false, "completed": true`) | `corrupt`(`キーが重複しています`) |
| `job` の要素に同じキーが 2 回 | `corrupt`(`キーが重複しています`) |
| `"id"` と `"id"` が同じ要素に | `corrupt`(`キーが重複しています`) |
| 重複キーがあり、かつ後ろに構文の誤り | `corrupt`(`JSON として読めません`。6a が先) |
| `{"format_version": 2, "unknown": 1}` | `versionMismatch`(未知のキーより版が先) |
| `{"format_version": 9007199254740993}`、`{"format_version": 100000000000000000000}`、5000 桁の整数 | `versionMismatch`(整数の字句の判定だけ。値を作らない。例外が出ない) |
| `{"format_version": -0}`、`{"format_version": 0}` | `versionMismatch` |
| `{"format_version": 1.0}`、`{"format_version": true}` | `corrupt`(`format_version が整数ではありません`) |
| `{"format_version": "1"}` | `corrupt`(`format_version が整数ではありません`) |
| `{"Format_Version": 1}` | `corrupt`(`format_version がありません`) |
| `{"format_version": 1, "__proto__": {}}` | `corrupt`(`知らないキーがあります`) |
| `{"format_version": 1, "__proto__": 1, "__proto__": 2}` | `corrupt`(`キーが重複しています`) |
| `exit_code` が `1.5`、`2147483648`、`9007199254740993`、`"3"`、`null`、`true`、`1e2` | `corrupt`(`値の型が誤っています`) |
| `exit_code` が `-2147483648`、`2147483647`、`-0` | 受理 |
| `resume_job_id` が `null` | `corrupt`(`値の型が誤っています`) |
| `run_id` が `null` | `corrupt`(`値の型が誤っています`) |
| 文字列に `"\ud800"`(孤立サロゲートのエスケープ) | `corrupt`(`JSON として読めません`。conductor が書いたステートには現れない) |
| 文字列に `"😀"`(正しい対。`😀` のエスケープでも同じ) | 受理(1 つの文字 U+1F600 になる) |
| 不正な UTF-8(`FF`)、サロゲートの符号化(`ED A0 80`)、最短でない符号化(`C0 80`)、途中で切れた(`E3 81`) | `corrupt`(`UTF-8 の文字列ではありません`) |
| ディレクトリを指すパス | `unreadable`(Unix・Windows の実機で確認する) |
| 1,048,576 バイトちょうど(正常な内容を空白で埋める)/ 1,048,577 バイト | 前者は受理、後者は `tooLarge` |
| 親が通常のファイルであるパス(`file/x`) | `unreadable`(§6.1 の順 1 の注。実機で確認する) |
| 長さ 0 のファイル | 空(`null`) |

---

## 7. 定義との整合性検査(`Consistency.Check`)

空でない(完了していない)ステートを読んだ後、`run`・`pipeline` の前処理([05](05-execution-flow.md) §5.1 の 7)で、定義と照合する。純粋な関数(`Conductor.State`。ファイル・時計に触れない)。

```text
if state.PipelineName != definition.Name:              Mismatch(PipelineName)
defined  = 定義のジョブIDの記述順の列
recorded = state.JobOrder
if defined の列と recorded の列が、要素ごとに(序数比較で)完全に等しい:   Ok
if 並べ替えた defined と並べ替えた recorded が、要素ごとに等しい:          Mismatch(JobOrder)    # 並びだけが違う
else:                                                                    Mismatch(JobIds)      # 追加・削除・改名
```

```csharp
// 型は §13.1(02 と同じ)
public abstract record ConsistencyResult;
public sealed record ConsistencyOk : ConsistencyResult;
public enum MismatchWhat { PipelineName, JobIds, JobOrder }
public sealed record ConsistencyMismatch(MismatchWhat What) : ConsistencyResult;

public static class Consistency
{
    public static ConsistencyResult Check(PipelineState state, FlowDefinition definition);
}
```

比較は文字列の完全一致(`string.Equals(a, b, StringComparison.Ordinal)`、または `==`。UTF-16 のコードユニットごとの比較で、大文字小文字・正規化を区別する。`string.Normalize()` を呼ばない。`CompareTo`・`string.Compare(a, b)`・`Array.Sort(string[])` の既定(いずれも現在の文化の規則)を使わない)。並べ替えは `Array.Sort(copy, StringComparer.Ordinal)`(または `List<string>.Sort(StringComparer.Ordinal)`。コピーに対して行い、入力の列を変えない)で、重複は取り除かず、並べ替えた列同士の一致を見る。ここでの並べ替えは「集合として同じか」を見るためだけなので、一貫した全順序なら何でもよい(UTF-16 の序数の順は、サロゲートの対の文字の順がコードポイントの順と一致しないことがあるが、全順序であることは変わらない)。読み込んだ文字列は、孤立サロゲートを含まない(§6.5。孤立サロゲートのエスケープは 6a で拒否される)。

| 状況 | 結果 |
|---|---|
| パイプライン名が違う | `PipelineName` |
| ジョブID の集合が違う(追加・削除・改名。重複の有無も含め、並べ替えた列が違う) | `JobIds` |
| 集合は同じで記述順だけが違う | `JobOrder` |
| すべて同じ | `Ok` |

- `definition_key` と `definition_path` は照合に使わない(定義ファイルの置き場所を動かしても食い違いにしないため)。
- ジョブのコマンド、遷移、成功コード、リトライ設定などの違いは検知しない(IDと並びだけを見る)。
- `completed = true` のステートは「空」なので、照合しない。
- 食い違いのとき、ジョブは起動せず、ステートも変えず、65。標準エラー:

```text
conductor: error: <ステートファイルのパス>: state does not match the definition: <種類>; <案内>
```

  種類: `pipeline name` / `job ids` / `job order`(日本語: `パイプライン名` / `ジョブID` / `ジョブの並び`)。日本語の文は `ステートが定義と食い違っています`。案内は §6.2 と同じ(run ID あり/なしで切り替わる)。

---

## 8. 初期化(`reset`)

### 8.1 `conductor reset [--config <パス>] [--run-id <ID>]`

- 読まず、検査せず(破損・食い違い・版違い・大きさに関係なく)ステートファイルを削除する。定義ファイルは検証しない。
- 対象のファイル: run ID の決定([05](05-execution-flow.md) §6.2。`--run-id` があれば定義を読まない。なければ定義の `pipeline.run_id_env` だけをゆるく読み、その環境変数を見る)で決まった 1 つ。
- 結果: ファイルが無ければ**成功**(0、何も表示しない)。削除が成功 → 0。失敗(ディレクトリ・権限等。「存在しない」以外)→ `removeFailed`、65。実装は **`File.Delete(path)`** を使う。`File.Delete` は、削除するファイルが存在しなくても例外を投げない、と理解している(`File.Delete` の一般の契約。この文書では取得して確認していない: 未確認)。そのため、「存在しない」を成功にする分岐を別に書かなくてよいが、念のため、`File.Delete` の前後で `File.Exists` を見て分岐する書き方は避ける(競合する。`File.Delete` の結果だけで決める)。`DirectoryNotFoundException`(親のディレクトリが無い)は、「存在しない」として成功にする。それ以外の例外(`UnauthorizedAccessException`、`IOException`、`PathTooLongException`、`NotSupportedException`、`ArgumentException`)は `removeFailed`。
- 完了のステートも消す。標準出力・標準エラーには何も出さない。
- パスがシンボリックリンクなら、リンク自体を削除する(`File.Delete` は、Unix では `unlink(2)` を使うので、リンクを辿らず、リンク自体を消す、と理解している。未確認。受け入れテストで、削除後もリンク先のファイルが残ることを確かめる)。
- **ディレクトリを指すパスは消さない**。`File.Delete` はディレクトリを消さず、`UnauthorizedAccessException`(または `IOException`)を投げる、と理解している(Unix・Windows とも。未確認)ので、`removeFailed`(65)になる。`Directory.Delete`・`FileSystem.DeleteDirectory`(再帰)を使わない(空のディレクトリや中身ごと消しうるため)。受け入れテストで、ディレクトリの名前が `conductor.json.state` であるとき、ディレクトリとその中身が残り、65 になることを確かめる。
- 親のパスが通常のファイルのとき(`ENOTDIR`)は、`DirectoryNotFoundException` になる、と理解している(未確認)ので、成功(0)になる。期待の挙動(`removeFailed`)と違うかもしれない。実装時に実機で確認し、違うなら、`File.Exists(parent)` の確認を足して `removeFailed` にする。
- run ID の値が規則に合わなければ 68。

### 8.2 `conductor reset --all [--config <パス>]`

定義の全ステートを、run ID の有無によらず消す。環境変数は見ない(値の誤りも 68 にならない)。`--run-id` との併用は 68。

対象の判定(厳密):

1. 一覧にするのは、定義ファイルと同じディレクトリだけ(再帰しない。`Directory.EnumerateFileSystemEntries(dir)` で名前を列挙する。順序は保証されないので、名前を `StringComparer.Ordinal` で並べ替える。run ID は ASCII だけで、候補の名前は同じ接頭辞を持つので、UTF-16 の序数の順と、コードポイントの順・UTF-8 のバイト列の順は一致する)。`dir` はディレクトリ部分が空のとき `"."`。`searchPattern` を使わない(§5)。
2. `<定義ファイル名>.state`(run ID なし): **通常のファイルのときだけ**(シンボリックリンクを辿らず、消さない。`new FileInfo(path)` の `Exists` が真、`Attributes` に `FileAttributes.Directory` が無く、`LinkTarget` が `null`。`File.Exists` はリンクを辿るので使わない。`LinkTarget` は .NET 6 以降で、リンクでなければ `null` と理解している。未確認)、中身を読まずに削除(`File.Delete`)。
3. `<定義ファイル名>.<run-id>.state`: 名前の `<run-id>` が有効な run ID(§1・[05](05-execution-flow.md) §6.1 の `RunIdRules.Parse` が成功)であるものだけが候補。名前は、定義ファイル名で始まり、続けて `.`、run ID、`.state`(`name.Length >= defName.Length + 1 + 1 + ".state".Length` で、`name.StartsWith(defName + ".", StringComparison.Ordinal)`、`name.EndsWith(".state", StringComparison.Ordinal)` で、その間 `name.Substring(defName.Length + 1, name.Length - defName.Length - 1 - ".state".Length)` が run ID。run ID なしの `<定義ファイル名>.state` は、この間が空になるので、候補にならない)。候補をファイル名の昇順(序数)に処理する。
   - 通常のファイルでない(ディレクトリ、シンボリックリンク)→ 黙って除外。取得中に消えた(`FileInfo.Exists` が偽)→ 除外。`FileInfo` の取得でその他の理由の例外(`IOException`、`UnauthorizedAccessException`)→ `kept` に入れる。
   - 中身を見分ける(`InspectRunState`。§6.1 の順 1〜13 を、ファイル名側の run ID を期待値として行う。`ReadRaw` と同じ手順):

| 中身 | 結果 |
|---|---|
| 開けない、1 MiB 超、読み取り失敗、UTF-8・JSON の構文・形式の誤り(`Detail = RunIdMismatch` 以外の `corrupt`。キーの重複・BOM・深すぎる入れ子を含む)、`versionMismatch` | **残して**、名前を `kept` に入れる |
| 長さ 0 | 残す。`kept` に**入れない**(誰のものか分からない) |
| 読めて、中身の `run_id` が名前の run ID と一致(完了のままのものを含む) | **削除** |
| 読めるが、中身の `run_id` が無い、または値が違う(別の定義のステートでありうる。`StateError` の `Detail = RunIdMismatch`) | 残す。`kept` に入れない(黙って残す) |

4. 一時ファイル(`.<ステート名>.tmp.*`)、ほかの名前のファイル、ほかのディレクトリは対象外。
5. 削除の失敗(「存在しない」以外)→ `removeFailed`、65(処理は途中で打ち切る)。ディレクトリ一覧の取得に失敗(ディレクトリが無い場合(`DirectoryNotFoundException`)を除く)→ `unreadable`、65。ディレクトリが無ければ成功。

結果の出力: `kept` が空でなければ、標準エラーに警告(終了コードは 0)。消すものがなければ何も出さず 0。

```text
conductor: warning: kept state files that could not be read (remove one with `conductor reset --run-id <ID>`):
  c2.json.r2.state
```

日本語: ``conductor: warning: 読めなかったため、次のステートファイルは残しました(`conductor reset --run-id <ID>` で個別に消せます):``、続けて 2 スペースで字下げしたファイル名を 1 行ずつ。

例(`c2.json` に対して `reset --all`): run ID なしの `c2.json.state` は削除、順序が壊れた(JSON として壊れた、または内容に矛盾のある)`c2.json.r2.state` は `kept`(警告)、`"run_id": "r2"` を持つが名前が `r3` の `c2.json.r3.state` は黙って残る、ほかの定義のステート `conductor.json.r1.state` は対象外で残る。

---

## 9. 完了と `--remove-on-complete`

### 9.1 完了のステートの扱い

- 完了(`completed = true`)の条件は [05](05-execution-flow.md) §10。完了のステートは、`completed = true` のまま**残る**(自動では消えない)。書式は変わらない(`format_version` は 1)。
- 読み込み(`Read`)は、完了のステートを「空」として返す(§6.1 の 14)。したがって次の `pipeline` は先頭から始まり、`run` は最初のジョブ以外が順序外(66)になる。
- `watch` は `ReadRaw` で完了のステートを読み、表の下に完了を示す行(英語 `completed`、日本語 `完了`)を足す。`no state (nothing has run yet)` を出すのは、ファイルが無いか長さ 0 のときだけ。壊れた完了のステートは 65。
- `reset`・`reset --all` は完了のステートも消す。

### 9.2 削除の指定

| 項目 | 値 |
|---|---|
| オプション | `--remove-on-complete`(`run`・`pipeline` だけ。値を取らない。ほかのコマンドに付けると 68) |
| 定義 | `pipeline.remove_on_complete`(真偽値。省略時 false。真偽値でなければ定義の誤り 64) |
| 実効値 | `--remove-on-complete` があれば true、なければ定義の値。環境変数では切り替えない。オプションで false にする手段はない |

完了時の手順: 完了のステートを `Durable` で書く → 実効値が真なら `ResetState`(run ID に応じたファイルを削除。存在しなくても成功)→ 成功なら 0(`run` ではジョブの終了コード)。削除に失敗したら 65(完了のステートは `completed = true` のまま残る。`watch` は完了として表示する)。

`run` では、完了の契機が「スキップの記録」のときも同様に、書き込みの後に削除する。

---

## 10. 書き込みと読み込みの併用シナリオ(確認用)

再実装の確認に使える、ファイルの状態遷移の例(定義 `a → b → c`、遷移なし、run ID なし、`conductor.json`)。

| 操作 | ファイル `conductor.json.state` | 終了コード |
|---|---|---|
| `pipeline`(b が exit 3 でエラー) | §4.4 の例(1)と同じ内容(`pipeline_name` は定義の名前) | 3 |
| `pipeline` を再実行(a は AlreadyExecuted、b・c 成功) | 全ジョブ success、`completed = true`、`resume_job_id` 無し(残る) | 0 |
| もう一度 `pipeline` | 完了のステートは空とみなされ、初期ステートで上書きされ、先頭から実行 | ジョブによる |
| ファイルの `format_version` を `2` に書き換えて `run a` | 変更なし。`state file version is not supported` | 65 |
| `reset` | ファイルを削除(無くても成功) | 0 |

---

## 11. 並行実行・排他・持続性

- **排他制御はしない**(ファイルロック、PID ファイルなし。`FileStream.Lock`・`FileShare.None` を長く保持するロックにしない。一時ファイルを書いている間だけの `FileShare.None` は、ロックではなく、書き込み中の保護)。同じ定義(同じ run ID)で複数の conductor を同時に動かすと、それぞれが読んで書き、後に書いた方が勝つ。書き込み自体は原子的なので、ファイルが壊れることはないが、記録は失われうる。同時の実行は避けること。
- 別の実行単位(run)で同じ定義を並行して使うときは、run ID でステートを分ける(§1)。run ID ごとに独立したファイルと再開位置を持つ。
- **at-least-once**: ジョブの終了と、結果の書き込みのあいだに conductor が落ちると、ジョブは未実行として扱われ、再実行されうる。
- **リトライの回数はステートに書かない**(書式に項目がない)。再開は、そのジョブを最初の実行から数え直す。
- ステートは、定義ファイルのあるディレクトリに作られる。書き込めないディレクトリでは、最初のジョブの前に 65 で止まる(初期ステートが書けない)。
- 古い run のステートは自動では消えない。`reset --all` か、`--remove-on-complete` で片づける。
- 書き込みは同期の関数で、1 つの `run`・`pipeline` の中では常にメインスレッドから直列に行う([05](05-execution-flow.md) §15)。シグナルのハンドラの中でステートを書かない。書き込みを、ジョブの終了の観測より前に行わない(「ジョブの終了後に書く」が at-least-once の前提)。`async`・`Task` を使わない。

---

## 12. 再現すべき特記事項

| # | 項目 | 挙動 |
|---|---|---|
| 1 | `reset --all` と長さ 0 のファイル | 長さ 0 の `<定義>.<run-id>.state` は、消さず、警告にも出さない |
| 2 | `reset`(個別)とシンボリックリンク | 個別の `reset` は、リンク自体を削除する。`reset --all` は、シンボリックリンクを消さない |
| 3 | 理由文の言語 | 理由(例: `JSON として読めません`)は、ロケールに関わらず日本語の固定文。種類の文と案内はロケールに従う |
| 4 | `definition_key` | 形式に残っているが、`definition_path` と同じ値で、照合に使わない |
| 5 | 版の先行検査 | `format_version` は、未知の項目の検査より先に調べる |
| 6 | キーの大文字小文字 | 区別する。`Format_Version` や `ID` は未知のキー(または必須のキーが無い)として扱う |
| 7 | `null` | どの項目の `null` も型の誤り。「無い」とは扱わない |
| 8 | BOM | 拒否する(`new UTF8Encoding(false, true)` で復号し、結果の先頭の U+FEFF を 6a にする。§6.1.2、§6.5) |
| 9 | 巨大な整数 | `format_version` と `exit_code` は、整数の字句の判定で行う。巨大な字句で `int.Parse`・`long.Parse` を呼ばない(`long.TryParse` は、桁数の検査のあとだけ)(`OverflowException` を起こさない) |
| 10 | ディレクトリを指すステートのパス | `reset` は消さず `removeFailed`(`File.Delete` を使う。§8.1) |
| 11 | 真偽 | `true` / `false` は整数でない。`exit_code` や `format_version` に受理しない |
| 12 | `NaN` / `Infinity` | 拒否する(自前の読み取りが拒否する) |
| 13 | 孤立サロゲート | 読み込みでは拒否する(6a)。書き出しでは、`definition_path` / `definition_key` の孤立サロゲートを U+FFFD に置き換える |
| 14 | `exit_code` の `0` | 「ある」。`HasValue` で判定する(§4.2) |
| 15 | 改行 | 書き出しは LF のみ。`Environment.NewLine` を使わない(Windows で CRLF になる。§4.3 の確認) |

---

## 13. 型の骨格・実装に任せてよい点・未確認

### 13.1 エラーと読み書きの型(骨格)

型の置き場所と名前は [02-architecture.md](02-architecture.md) の `Conductor.State` に合わせる。**失敗は例外でなく値で返す**(`StateFail`)。

```csharp
namespace Conductor.State;

public enum Durability { Durable, Fast }       // Durable は内容の同期(Flush(true))まで行う
public enum StateErrorKind { Unreadable, TooLarge, Corrupt, VersionMismatch, WriteFailed, RemoveFailed }
// 内部の判別用(表示に使わない)。reset --all が「中身の run_id が名前と合わない」を見分けるために使う(§8.2)。
public enum StateErrorDetail { None, RunIdMismatch }

public sealed record StateError(StateErrorKind Kind, string Path, string Reason, StateErrorDetail Detail = StateErrorDetail.None);
// Reason は §6.4 の固定文。入力の値や下位の例外の文を、Reason にも表示にも含めない。

public abstract record StateOutcome;                                         // すべての入出力の結果の共通の基底(02 章)。失敗を値で返す。例外を送出しない
public sealed record StateFail(StateError Error) : StateOutcome;             // 失敗(読み込み・書き込み・削除のすべてで共通)
public sealed record StateReadOk(PipelineState? State) : StateOutcome;       // null は「空」。Read は完了のものも null、ReadRaw は完了のものも返す
public sealed record StateWriteOk : StateOutcome;                            // Write・Reset の成功
public sealed record ResetAllOk(ResetAllOutcome Outcome) : StateOutcome;     // ResetAll の成功
public sealed record ResetAllOutcome(IReadOnlyList<string> Kept);            // 読めずに残したステートのファイル名(整列済み。空でなければ警告)
// 戻り値の型は StateOutcome とし、メソッドごとに、期待する派生型(Read・ReadRaw は StateReadOk、Write・Reset は StateWriteOk、ResetAll は ResetAllOk)と StateFail だけが返る、と規定する

public interface IStateStore
{
    StateOutcome Read(string definitionPath, string? runId);                // StateReadOk | StateFail。空は StateReadOk(null)。完了も StateReadOk(null)(14)
    StateOutcome ReadRaw(string definitionPath, string? runId);             // StateReadOk | StateFail。14 を行わない
    StateOutcome Write(string definitionPath, string? runId, PipelineState state, Durability durability);   // StateWriteOk | StateFail
    StateOutcome Reset(string definitionPath, string? runId);               // StateWriteOk | StateFail。無くても StateWriteOk
    StateOutcome ResetAll(string definitionPath);                           // ResetAllOk | StateFail。§8.2
}

public sealed record StateStoreOptions(Func<DateTimeOffset> Now, bool SyncDirectory);   // 一時ファイル名用の時刻 / UNIX 系 true・Windows false

// 整合性の検査(§7。純粋)
public abstract record ConsistencyResult;
public sealed record ConsistencyOk : ConsistencyResult;
public enum MismatchWhat { PipelineName, JobIds, JobOrder }
public sealed record ConsistencyMismatch(MismatchWhat What) : ConsistencyResult;
public static class Consistency { public static ConsistencyResult Check(PipelineState state, FlowDefinition definition); }
```

- 表示用の組み立て(種類の文・案内・ロケール)は `Conductor.Report` が行う。
- `IOException`・`UnauthorizedAccessException` などの BCL の例外は、`IStateStore` の実装の内側で、**狭く**(ファイル操作の `try` の中だけ)捕まえて `StateFail` に変える。それ以外の例外(`NullReferenceException`、`InvalidCastException` など。実装の誤り)は捕まえず、`Main` の最上位が 70 にする([10](10-errors-and-exit-codes.md))。
- 一時ファイル名の時刻(§5 の手順 3)は、`StateStoreOptions.Now` から得る(`DateTimeOffset` の UTC の Unix エポックからのナノ秒に直す。`UtcDateTime.Ticks` から 100 を掛ける)。`DateTime.UtcNow` を直接読まない(テストで時刻を固定するため)。
- `StateStoreOptions.SyncDirectory` が真のとき、手順 9 でディレクトリを同期する(決定)。P/Invoke の呼び出しは `Conductor.Native.UnixNative.SyncDirectory(string)`(中身は §5 の `open`・`fsync`・`close`。07 §7.2.1)の 1 か所に置く。`UnixNative` は `Conductor.Runtime` 内部の `internal` な型で、`Conductor.State` と `Conductor.Runner` の両方が、名前空間 `Conductor.Native` にだけ依存して使う(`State` と `Runner` は互いに依存しない。02 章 3 章)。OS の判定は `State` に持たせず、`Conductor.Cli` が `PlatformKind` から `StateStoreOptions.SyncDirectory` を決めて渡す(UNIX 系は真、Windows は偽)。

### 13.2 実装に任せてよい点

- 一時ファイルの名前の細部(§1 の条件を満たすこと)、再試行の回数の上限の具体値は 16 を推奨。
- 内部の構造(書き出しの関数の分け方、読み込みの内部の作りなど)。ただし、キーの順序・インデント・末尾の改行・文字のエスケープ(§4)、読み込みの規則(§6.5)は固定。
- ファイルの権限(§5。`0666` から umask を引いた OS 標準にする)。

### 13.3 確認できたこと(公式ドキュメントの取得による。出典は本文の該当箇所)

- この文書の範囲では、次の API の契約を確認した。
  - `FileStream.Flush(bool flushToDisk)`: 「中間のファイルバッファもすべて消す」。`Flush()` は OS の I/O バッファも消す(`FileStream.Flush` のページ)。
  - `File.Move(string, string, bool overwrite)`: .NET Core 3.0 以降。`overwrite = true` で置き換え。例外の種類。ボリュームをまたぐ移動はコピーと削除に等しい。原子性の記載は無い(`File.Move` のページ)。
  - `FileStreamOptions.UnixCreateMode`: .NET 7 以降。`UnixFileMode?`。`set` は Windows で使えない。既定値・umask の記載は無い(同ページ)。
  - `UTF8Encoding(bool, bool)`: `throwOnInvalidBytes` の働き。既定は不正を黙って置換する。BOM は `GetPreamble` の働きで、`GetBytes` は BOM を前置しない。不正なサロゲートは `EF BF BD` に置換される(同ページ)。
  - `JsonWriterOptions`: `NewLine` の既定は `Environment.NewLine`、`IndentSize` の既定は 2、`IndentCharacter` の既定は空白、`MaxDepth` の既定は 1000。`Utf8JsonWriter` は構造の誤りで例外を投げる(同ページ)。
  - `JsonDocumentOptions`: `AllowDuplicateProperties`・`AllowTrailingCommas`・`CommentHandling`・`MaxDepth`(既定 64)(同ページ)。
  - `Utf8JsonReader`: 既定で RFC 8259 に厳密、`JsonException`、`JsonReaderOptions`、`TokenStartIndex`・`ValueSpan` など(同ページ)。
  - `System.Text.Json` の文字エンコーディング: 既定のシリアライザーは ASCII 以外を `\uxxxx` にエスケープ。`UnsafeRelaxedJsonEscaping` は `<>&'` をエスケープしない。グローバルのブロックリストは版ごとに変わる実装の詳細(「文字エンコーディングのカスタマイズ」のページ)。

### 13.4 未確認(実装時に確認する)

- `FileMode.CreateNew` が既存のとき `IOException` を投げること(一般の契約と理解)、衝突と他の `IOException` を `File.Exists` で見分ける方法の十分さ。
- `UnixCreateMode` の既定値と umask の関係、`0666` を指定したときの実際の権限(§5。受け入れテストで固定)。
- `FileStream.Flush(true)` が Unix で `fsync(2)`、Windows で `FlushFileBuffers` を呼ぶこと。
- `File.Move(overwrite: true)` の原子性(Unix: `rename(2)`、Windows: `MoveFileEx`)と、Windows で他のプロセスが開いているときの失敗の型。
- `File.Replace` を採らない理由の各点(置き換え先が無いと失敗、Windows の属性・ACL の引き継ぎ)。
- ディレクトリの同期の P/Invoke の `LibraryImport("libc")` の名前解決(置き場所は `Conductor.Native.UnixNative` に決定済み。§13.1)、`open` を `O_RDONLY` でディレクトリに使えること、`AllowUnsafeBlocks` の要否)。
- `Path.GetDirectoryName`・`Path.GetFileName`・`Path.Join` の境界の挙動(§1)。
- `FileInfo.LinkTarget` が、リンクでないとき `null`、Unix のシンボリックリンクで非 `null` になること。
- ディレクトリ・親が通常のファイルのパスを `FileStream` で開いたとき、`File.Delete` で消そうとしたときの例外の型(Unix・Windows)。`File.Delete` が存在しないファイルで例外を投げないこと。
- `UTF8Encoding(false, true).GetString` が、§6.1.2 の不正な UTF-8 の 4 種を拒否し、BOM を取り除かないこと、投げる例外の型(`DecoderFallbackException`)。
- `Utf8JsonReader` の BOM・不正 UTF-8・サロゲートのエスケープ・重複キー・数値の字句・深さの超過の挙動(§6.5 の表。自前の読み取りを採るので、仕様の達成には影響しない)。
- 引数のパスに不正な UTF-8 のバイトがあるとき、Unix で .NET の `string` にどう入るか。Windows のパスの孤立サロゲート。
- Windows での、ファイルの権限(ACL)、ディレクトリを開いたときの `unreadable` の判定、`FileShare` の組み合わせの挙動。
- 一時ファイルの `<時刻>` は、`StateStoreOptions.Now` 由来のナノ秒。クロックが戻った場合の挙動は、名前の衝突の再試行(最大 16 回)に任せている。
- §4.4 の例は、規則から機械的に作った文字列で、C# の実装の出力としては確認していない。例 4 は、規則から導いた値(実行の流れの中で書かれた出力ではない)。
