# 10. エラーと終了コード

この章は、C#(.NET)で実装する `conductor` の終了コードの全件、複数の条件が当てはまるときの優先順位、ジョブの結果から終了コードへの対応、終了コードの整形、想定外の内部失敗の扱い、そして C# でのエラー型の設計を規定する。利用者から見えるものは「エラー分類」「終了コード」「メッセージ」で規定し、型は 8 章で規定する。メッセージの全文は [08-output-and-i18n.md](08-output-and-i18n.md)、引数の誤りは [04-cli.md](04-cli.md)、流れは [05-execution-flow.md](05-execution-flow.md)、シグナルは [07-process-and-signals.md](07-process-and-signals.md)、ステートは [06-state-file.md](06-state-file.md)、定義は [03-definition-format.md](03-definition-format.md)、変換は [09-converter.md](09-converter.md) を参照する。

「確認(出典: …)」は、公式ドキュメントまたは公開されているランタイムのソースを取得して確かめた内容、「未確認」は、この環境に .NET SDK がなく実機で動かせないこと、または出典を取得できなかったことで、確かめられていない内容である。

## 1. 終了コードの一覧(全件)

| 終了コード | 名前 | 意味 | 主な発生箇所 |
|------------|------|------|--------------|
| 0 | OK | 成功。ジョブの成功・警告、スキップ(実行済み・遷移・`@end`)、`watch`・`reset`・`validate`・`convert` の成功、`--help`・`--version`。`reset --all` の警告(読めなくて残したファイル)も 0 | 全般 |
| 1 | UNLISTED_ZERO | ジョブが終了コード 0 で終わったが、`success_exit_codes` に 0 がない。または、ジョブの終了コードがエラーで、下位 8 ビットが 0 になる(例: 256)。または、起動して終わったのに終了コードが得られない(起こらない想定) | ジョブの結果の分類、終了コードの範囲合わせ |
| 1〜255 | (ジョブの値) | エラーになったジョブの終了コードを、そのまま。範囲外は下位 8 ビット(0 になるなら 1)。4 章 | 範囲合わせ |
| 64 | DEFINITION | 定義の誤り: ファイルがない・読めない・大きすぎる・文法・検証の誤り(パイプライン名の不一致、`run_id_env` や `remove_on_complete` の型・形の誤り、`retry` の誤りを含む)、拡張子が `.json`・`.yaml`・`.yml` のどれでもない、`command` の誤り(引用符が閉じていない・空)。`validate` の失敗も 64 | `run`, `pipeline`, `validate` |
| 65 | STATE | ステートの誤り: 読めない・大きすぎる・壊れている・版の不一致・定義との食い違い・中身の `run_id` とファイル名の食い違い・書き込みの失敗・削除(初期化)の失敗。`reset`・`reset --all` で消すのに失敗。`watch` が標準出力に書けない。`pipeline` が完了を判定できない | `run`, `pipeline`, `watch`, `reset` |
| 66 | REJECTED | 拒否(順序外。`run` で、前のジョブがまだ済んでいない、など。05 章の `Reject(OutOfOrder)`)。スキップ対象・`@end` より後ろ・実行済みのジョブは、拒否ではなくスキップで、終了コード 0 | `run`, `pipeline` |
| 67 | CONVERSION | 変換の誤り。`convert` が標準出力に書けない | `convert` |
| 68 | USAGE | 使い方の誤り(知らないコマンド、引数の不足、余分な引数、知らないオプション、使えないオプション、値の欠落・誤り、知らないジョブID、run ID の値の誤り〔引数・環境変数とも〕、重複、`--all` と `--run-id` の併用、文字として正しくない引数) | 引数の解釈、run ID の決定、`watch`/`reset` |
| 70 | INTERNAL | 内部の誤り: 想定外の内部失敗(6 章)、シグナルの準備の失敗 | 全般、`run`/`pipeline` の準備 |
| 127 | START_FAILURE | ジョブを起動できなかった(実行ファイルがない・実行権限がない、など。終了の状態を取得できない場合も安全側で起動失敗扱い) | ジョブの結果の分類 |
| 128 + 番号 | SIGNAL_BASE | ジョブがシグナルで終了した、または `conductor` が中断された(例: SIGHUP = 129、SIGINT = 130、SIGQUIT = 131、SIGTERM = 143)。リトライの待機中の中断も含む | ジョブの結果の分類、中断 |

- 64〜68 と 70 は、ジョブ自身の終了コードと重なりうる。区別は、標準エラーの `conductor: error: ...` の行で行う(70 は固定の英語の文)。
- 終了コードの範囲(0〜255)への合わせ方は 4 章。
- .NET の実行環境が、`conductor` の規定にない終了コードで、プロセスを終わらせることがある(捕捉されなかった例外のときの、実行環境の既定の終了コード。値は OS によって違うとされるが、公式ドキュメントでは確認できていない。未確認)。実装は、これらが外へ出ないようにする(捕捉されなかった例外は 70: 6 章)。

## 2. 終了コードの決定順序

`run` / `pipeline` は、次の順に検査し、最初に当てはまった時点で終わる(それ以降は行わない)。

| 順 | 検査 | 失敗時 |
|----|------|--------|
| 0 | 引数の解釈(オプションとして解釈された `-h`/`-V` は先に 0。値を取るオプションの値として取られた `--help` などは対象外。それ以外の使い方の誤り。04 章 4.1・4.2) | 68 |
| 1 | シグナルの監視の準備(定義の検査より前)。「すでに登録済み」は成功と同じに扱う | 70 |
| 2 | 定義の読み込みと検証(パイプライン名の照合を含む。誤りは全部まとめて 1 行ずつ表示) | 64 |
| 3 | 全ジョブの `command` の検査(誤りのあるジョブごとに 1 行) | 64 |
| 4 | `run` のジョブIDが定義にあるか | 68 |
| 5 | run ID の決定(`--run-id` または環境変数。規則違反) | 68 |
| 6 | ステートの読み込み | 65 |
| 7 | ステートと定義の照合 | 65 |
| 8 | 起動前の判断 | 拒否は 66。スキップは 0(`run` は 0 で終わる。`pipeline` は続ける) |
| 9 | (実行のとき)初期ステートの書き込み | 65(ジョブは起動していない) |
| 10 | ジョブの実行(リトライ込み)、結果の分類 | 結果から決まる(3 章) |
| 11 | 結果のステートの書き込み | 65(ジョブは実行済み。ジョブの終了コードより優先) |
| 12 | 完了で、削除が求められていれば、ステートの削除 | 65 |

補足:

- 検査やステートの確認の途中でシグナルを受け取っても、検査は最後まで進む。定義の誤り(64)、ステートの誤り(65)、拒否(66)、スキップ(0)の終了コードが、中断の終了コードより優先する。
- `watch`: run ID の決定(68)→ ステートの読み込み(65)→ 表の出力(書けなければ 65)。成功は 0。定義は検証しない。シグナルの準備はしない。
- `reset`: run ID の決定(68。`--all` のときは決定しない)→ 削除(65)。成功は 0。
- `validate`: 定義(64)→ `command`(64)。成功は 0。ステートに触れない。シグナルの準備はしない。
- `convert`: 変換(67)→ 標準出力への書き込み(書けなければ 67)。成功は 0。
- `--help`・`--version`: 標準出力に書けなくても 0(書き込みの失敗は無視する。08 章 1.3.2)。標準出力に書けない失敗は、`watch` が 65、`convert` が 67、それ以外の経路で未捕捉になったものは 70(6 章)。**既知の差**: UNIX 系では、読み手が先に閉じたパイプへの書き込みは、.NET の標準のストリームが捨てるため検出できず、`watch`・`convert` が 65・67 でなく 0 で終わりうる(08 章 1.3.1)。Windows と、`/dev/full` などの `EPIPE` 以外の失敗では、65・67 になる(Windows は未確認)。
- `pipeline` の特有: 実行したジョブがエラー(または分類なし)なら、そのジョブの終了コードで終わる。ジョブのあいだに中断の合図があれば `128 + 番号`。完了なら 0。完了を判定できなければ 65。
- ステートの書き込みに失敗すると、ジョブの終了コードではなく 65 で終わり、「ジョブは実行されました」と表示する(少なくとも 1 回は実行された、という方針)。起動前の失敗(初期ステートの書き込みなど)は「ジョブは起動していません」。
- 想定外の内部失敗(6 章)は、どの時点でも 70 で終わる(上の順序の外)。

## 3. ジョブの結果から終了コードへ(分類)

最初に当てはまるものを採る。この判断は `Conductor.Flow`(`Conductor.Core`)の純粋なメソッド(ジョブの実行結果の値を入力に、分類と報告する終了コードを返す)として実装する([05-execution-flow.md](05-execution-flow.md))。

| 順 | 状況 | 分類 | 報告する終了コード | 備考 |
|----|------|------|--------------------|------|
| 1 | 起動しておらず、中断された | 分類なし | `128 + 番号` | エラーとして記録しない。再開位置はそのジョブ |
| 2 | 起動しておらず、中断でもない(起こらない想定) | エラー | 127 | 起動失敗と同じ扱い |
| 3 | 起動済みで中断された | エラー | `128 + 番号` | ジョブ自身の終了コードは見ない |
| 4 | 起動失敗 | エラー | 127 | |
| 5 | ジョブがシグナルで終了 | エラー | `128 + シグナル番号` | |
| 6 | 起動して終わったが、終了コードがない | エラー | 1 | `conductor` のエラー(`success_exit_codes` に含まれない 0 と同じ表示) |
| 7 | 成功コード(`success_exit_codes`、省略時は `[0]`)に含まれる | 成功 | 0 | |
| 8 | 警告コード(`warning_exit_codes`)に含まれる | 警告 | 0 | |
| 9 | 終了コードが 0 だが成功コードにない | エラー | 1 | `conductor` のエラー。`job '<ID>': exit code 0 is not listed in success_exit_codes` を常に表示 |
| 10 | それ以外 | エラー | ジョブの終了コード | 範囲を 0〜255 に合わせる(4 章) |

プロセスの終了コード = `ExitCodes.For(報告する終了コード, エラーか)`(4 章)。`run` は、この値で終わる。`pipeline` は、エラーまたは分類なしのジョブで止まるとき同じ式で、完了なら 0。

注: 「起動した」は、子プロセスを作れたこと(`Process.Start` が例外を送出せずに戻り、`Process` のオブジェクトが得られること)を指す。「中断」は、`conductor` がシグナル(07 章)を受け取り、中断の合図が立ったこと。ジョブがシグナルで終了した(順 5)のと区別する。

.NET での「終了の取り出し」(詳細は 07 章):

- 子の終了は、07 章 4.4 節の規則(`IPlatform.ReadExit` が、`Process.ExitCode` から `ExitCode` と `TerminatedBySignal` を決める)で `JobExecution`(05 章)の `ExitCode`・`TerminatedBySignal` にしてから、この章の分類に渡す。`Process.ExitCode`(`int`)から、シグナルで終了した場合の `128 + n` をどう取り出すか(UNIX 系: 終了コードが 129〜159 で、そのジョブの成功・警告のコードに含まれないときだけシグナル終了とみなす)は、07 章が規定する。この章の分類は、`JobExecution` の項目(`ExitCode`、`TerminatedBySignal`、`StartFailure`)だけを入力にする(`Process.ExitCode` の生の値の意味づけを、この章に持ち込まない)。順 5 は `TerminatedBySignal` が有るとき、順 6 は `ExitCode` も `TerminatedBySignal` も無いとき、順 7 以降は `ExitCode` が有るとき。
- Windows では、`Process.ExitCode` は符号つき 32 ビットの値で、例外コード(例: `0xC0000005`)が負の値として現れうる(値の符号の扱いは 07 章。ここでは未確認)。`ExitCode` は、`int` のまま 4 章の `ExitCodes.For` に渡す(下位 8 ビットを取る。符号つき・符号なしのどちらの解釈でも結果が同じ)。Windows では、ジョブの終了コードが 255 を超える値(例: 256)になりうるので、4 章の範囲合わせの対象になる。
- 起動の失敗: `Process.Start` は、起動に失敗したとき、`System.ComponentModel.Win32Exception` を送出する(ファイルがない・実行権限がない、など。`NativeErrorCode` に OS の誤りの番号が入る。例外の型の詳細は 07 章。ここでは公式ドキュメントを取得しておらず、未確認)。順 4(起動失敗)として扱う。`Win32Exception` の `NativeErrorCode` の値は、利用者には区別せず、すべて `cannot start command`(127)にまとめる。**捕捉するのは起動に関する例外の型(07 章が列挙するもの)だけ**とし、`Exception` 全体は捕捉しない(想定外の失敗を、起動失敗に化けさせないため)。
- 起動に失敗した場合は、`Process` の `WaitForExit` を呼ぶ対象がない(重複して分類しない。起動の例外の捕捉と、終了の待機は、1 つの `try` の中で、後者を前者の成功後にだけ行う)。

## 4. 終了コードの範囲合わせ

`ExitCodes.For(int reported, bool isError) -> int`(`Conductor.Flow`。05 章 2.2 節が正):

| 入力 | 出力 |
|------|------|
| `0 <= reported <= 255` | `reported` |
| 範囲外 | `reported` の下位 8 ビット(`reported & 0xFF`。負の値は 2 の補数表現の下位 8 ビット)。ただし、その結果が 0 で `isError` が真なら 1(エラーを成功に見せない) |

C# の `int` の `&` は、符号つき 64 ビットの 2 の補数表現のビット演算で、負の値も 2 の補数のまま下位 8 ビットを取れる(言語の仕様どおりの挙動。この環境では出典のページを取得しておらず、未確認)。入力の型は `int`(`Process.ExitCode` と `128 + n` の飽和加算の結果。05 章)。例(判定表。受け入れテストの入力に使う): 259 → 3、511 → 255、-1 → 255、-2 → 254、`0xC0000005`(符号なしで 3221225477)→ 5、符号つき 32 ビットの `-1073741819`(同じビット列)→ 5。256・512・`-256`・`int.MinValue` は、エラーなら 1、エラーでなければ 0。

- `128 + n` の計算は、`Saturating.AddI32(128, n)`(32 ビット符号つき整数の飽和加算。05 章)で行い、`checked` の文脈にしない(オーバーフローで例外にしない。プロジェクトで `<CheckForOverflowUnderflow>` を有効にしない)。`n` は小さな正の整数だけが渡る。
- `ExitCodes.SignalExitCode(int n) -> int` = `For(Saturating.AddI32(128, n), true)`。15 → 143、2 → 130、1 → 129、3 → 131、300 → 172(428 の下位 8 ビット)。`n` が負のときの規定はない(呼び出し側が正の番号だけを渡す)。
- `-v` の結果行には、元の値(`exit code <C>`)と、`conductor` の終了コードが異なるとき(`result=error` のみ)`, conductor exits with <E>` を示す(08 章 2.1)。

### 4.1 プロセスの終了(`Main` の戻り値)

`Program.Main` は、`EntryPoint.Run` が返した `int`(すでに 0〜255 に合わせたもの)を、08 章 1.4 の手順で出力を完了させたあと、**`Main` の戻り値として `return` する**。終了コードは、必ず 0〜255 の `int` にしてから返す。

確認(出典: 「Main() and command-line arguments」、「Environment.ExitCode」、「Environment.Exit」):

| 事項 | 内容 |
|------|------|
| `Main` の戻り値 | `int` を返す `Main` の値が、プロセスの終了コードとして呼び出し元に渡る。`Main` が戻ると、プログラムは終わる |
| `Environment.ExitCode` | 取得・設定できるプロセスの終了コード(符号つき 32 ビット整数。既定値は 0)。`Main` が `void` を返すときだけ使われ、「`Main` does not return `void`, this property is ignored」(`void` 以外を返す `Main` では無視される)。文書の警告: 負の終了コードを避けるため、`0x80000000` 以上の値を使わない |
| `Environment.Exit(int)` との違い | `Exit` は、常にアプリを終了させ、ほかのスレッドが動いていても直ちに終了する。`return` は、`Main` で使ったとき、フォアグラウンドのスレッドがすべて終わってから終了する。`try`/`catch` の中で `Exit` を呼ぶと、`finally` のコードは実行されない。`return` では実行される |

この仕様では、**`static int Main(string[] args)` が戻り値で終了コードを返す**(`Environment.ExitCode` に代入して `void` の `Main` を使う形にしない。値の出どころを 1 つにするため)。`Environment.Exit` は、6 章の最後の手段にだけ使う。

OS ごとの範囲(確認できた範囲):

- Windows: プロセスの終了コードは 32 ビットの値で、(取得した文書の記述の範囲では)`Environment.ExitCode` の値は符号つき 32 ビットで扱われる。
- Unix 系: 終了コードは、OS が下位 8 ビットだけを使う(0〜255)。`Environment.Exit(256)` が、Ubuntu で終了コード 0 になるという第三者の報告がある(Code Maze の記事。公式ドキュメントではなく、実機での確認もしていないので、参考にとどめる。未確認)。公式ドキュメントには、Unix での範囲外の値の挙動の記述を、取得したページの中では見つけられなかった。

したがって、この仕様では、`ExitCodes.For` で自分で 0〜255 に合わせた値だけを `return` し、OS の切り詰めや、Windows での符号つきの扱いに頼らない。この規定により、OS によらず、同じ値が終了コードになる(実機での 3 OS の確認は未確認)。

`Main` の戻り値の型は `int` であり、`void` や `Task` にしない(終了コードを持たない経路を作らない)。`async Main` を使わない(02 章の並行モデル)。

## 5. シグナルと中断

- UNIX 系: `SIGHUP`(1)・`SIGINT`(2)・`SIGQUIT`(3)・`SIGTERM`(15)を、`PosixSignalRegistration.Create`(07 章)で受け取り、コールバックは、最初の番号を中断の合図(フラグ)に記録するだけにする(コールバックの中で、複雑な処理・出力・ロックの取得をしない。`Volatile`/`Interlocked` で書く。02 章の並行モデル)。コールバックは、メインスレッドと別のスレッド(スレッドプールなど)で実行されうる。ほぼ同時に複数届くと、番号の小さい方が「最初」になりうる(順序は実装依存。必須ではない)。
- Windows: Ctrl+C(番号 2)だけ(`Console.CancelKeyPress` の `ControlC`。07 章 7.3 節)。Ctrl+Break とコンソールを閉じたときの通知は対象外(OS の既定の動作)。
- `run`・`pipeline` だけがシグナルを準備する。`watch`・`reset`・`validate`・`convert` は、シグナルを準備しない。これらのコマンドの実行中のシグナルは、実行環境の既定の動作に任せる(`Console.CancelKeyPress` や `PosixSignalRegistration` を登録しない)。その既定の動作(Ctrl+C でプロセスがどう終わるか、終了コード)は、公式ドキュメントを取得しておらず、未確認。実装時に確かめ、受け入れテストは、シェルから見て、シグナルで終わったこと(Unix で 130 になるか)だけを、確かめた結果に合わせて書く。
- 起動済みのジョブが中断されると、`128 + 番号`、エラーとして記録する。起動前、またはジョブのあいだ、リトライの待機中に中断されたときは、エラーとして記録せず、再開位置を記録して `128 + 番号` で終わる。
- 準備の失敗は、`conductor: error: failed to set up signal handling` を出して 70。同じプロセスで 2 回目の登録になる「すでに登録済み」は、成功と同じに扱う(実装が複数回呼ぶ構造のときだけ関係する)。`PosixSignalRegistration.Create` などの失敗は、シグナルの口(インターフェース)の実装が、準備できなかったときに例外(07 章が列挙する型)として返し、`run` がそれを準備の失敗(70)にする(07 章)。
- 詳細は [07-process-and-signals.md](07-process-and-signals.md)。

## 6. 想定外の内部失敗(終了コード 70)

実装のバグなどで、設計が想定していない失敗(C# では、規定されたエラー型以外の例外(`NullReferenceException`、`IndexOutOfRangeException`、`InvalidOperationException`、`InvalidCastException`、`OutOfMemoryException`、`ArgumentException` など)、`InvalidOperationException` を伴う列挙の変更、`Debug.Assert` が無効な Release での不変条件の破れ、`finally` や破棄の中の例外など)が起きたときの規定。

### 6.1 必須の挙動

1. 入口 `Program.Main` は、最初の実行の文で、次の 2 つのハンドラを登録する(ほかの処理より前)。両方が、同じ 1 つのメソッド `InternalFailure.ReportAndExit()` を呼ぶ。`Main` の本体は、`try`/`catch` で包む。

   ```csharp
   internal static class Program
   {
       private static int Main(string[] args)
       {
           AppDomain.CurrentDomain.UnhandledException += (_, _) => InternalFailure.ReportAndExit();
           TaskScheduler.UnobservedTaskException += (_, e) => { e.SetObserved(); InternalFailure.ReportAndExit(); };

           int code;
           try
           {
               code = EntryPoint.Run(args, SystemDeps.Create());
           }
           catch (Exception)                       // この 1 か所だけが、Exception 全体を捕捉する(8.1)
           {
               code = InternalFailure.Report();    // 70 を返す
           }
           return Finish(code);                    // 08 章 1.4(出力の完了)。0〜255 の int を返す
       }
   }
   ```

   (骨格。実装コードは規定しない。`Finish` は、出力を完了させ、終了コードを 0〜255 の `int` で返す。`Finish` の中の例外も、同じ `catch (Exception)` の規則で 70 にする。`try` を、`Finish` を含めて 1 つにまとめるか、二重にするかは、実装に任せる。)

2. `InternalFailure.Report()` は、標準エラーに、固定の英語の文を 1 行書く(ロケールを判定しない)。

   ```text
   conductor: error: internal error
   ```

   末尾は改行(LF)。書き込みは、**08 章の出力口(`IOutputSink`)を使わず**、`Console.OpenStandardError()` で得たストリームに、ASCII の定数のバイト列(`"conductor: error: internal error\n"` の UTF-8 バイト列)を直接 `Write` し、`Flush()` する。`IOException`・`ObjectDisposedException`・`NotSupportedException` を捕捉して無視する(出力口の状態(作れていない・書き込みに失敗済み)や、符号化の失敗に左右されないため)。この処理は、**1 回だけ**動く(`Interlocked.Exchange` のフラグで止める。2 回目以降の呼び出しは、何もしない。例外の連鎖で行が重複して出ないように)。
3. `AppDomain.CurrentDomain.UnhandledException` は、捕捉されなかった例外が、スレッドの最上位に届いたとき、システムの既定のハンドラが例外を報告してアプリを終了させる**前に**呼ばれる(確認(出典: 「AppDomain.UnhandledException」): 「provides notification of uncaught exceptions. It allows the application to log information about the exception before the system default handler reports the exception to the user and terminates the application」。複数のスレッドで例外が出れば、ハンドラは複数回呼ばれうる。ハンドラは、例外が投げられたときに保持していたロックを持ったまま動くので、ほかの資源を待たない)。このイベントの**ハンドラが戻ると、実行環境は、自分の既定の終了処理で終わる**(終了コードは 70 にならない)ので、`ReportAndExit()` は、**自分で 70 で終わる処理を行う**: 1 行を書き、`Flush()` し、`Environment.Exit(70)` を呼ぶ(確認(出典: 「Environment.Exit」): `Exit` は、ほかのスレッドが動いていても直ちに終了し、`finally` を実行しない)。**未確認**: `UnhandledException` のハンドラの中から呼んだ `Environment.Exit(70)` が、実行環境の既定の終了処理より優先して、終了コード 70 でプロセスを終わらせること(公式ドキュメントに、この組み合わせの記述がない)。実装時に、Windows・Linux・macOS で、別プロセスから終了コードを読んで確かめる。確かめて 70 にならなければ、別の手段(例外を `Main` の外へ出さない構造にする)を、この章に追記して決める。`ReportAndExit()` 自身が例外を送出しないよう、内部を `try`/`catch`(特定の例外型)で包み、`Environment.Exit` のあとに処理を続けない。
4. `TaskScheduler.UnobservedTaskException` は、誰にも観測されなかった、失敗した `Task` の例外が、ファイナライザの実行などで検出されたときに呼ばれる(確認(出典: 「TaskScheduler.UnobservedTaskException」): 「unobserved exceptions raise the UnobservedTaskException exception, but the process does not terminate by default」。実行環境は、イベントのあと、ハンドラの処理にかかわらず、この例外を処理し、既定ではプロセスを終了させない。構成の `ThrowUnobservedTaskExceptions` で終了させる設定にできる)。`conductor` は、`Task` を作らない設計(02 章。`async` を使わない)なので、通常の流れでは起こらない。起こったら、設計の破れ(バグ)なので、ハンドラは、`e.SetObserved()` のあと、上の 3 と同じ `ReportAndExit()` で 70 にする。呼ばれる時点が、ガベージコレクタ・ファイナライザの実行に依存し、非決定的である(いつ呼ばれるかは保証されない)。したがって、受け入れテストの対象にしない。
5. スタックトレースや失敗の詳細を、標準出力へ出してはならない(標準出力はジョブの出力と `watch`・`convert` の結果のためにある)。詳細(例外の `ToString()`)を標準エラーへ出すかは、実装に任せてよい(出す場合も上の 1 行を、必ず含める。1 行目に出す)。既定は出さない。
6. 外部の入力(引数、環境変数、ファイル、シグナル、ジョブの出力)に対して、想定外の失敗が起きない作りにする。これらは、すべて、規定された誤り(64〜68 など)として扱う。たとえば、配列の範囲外の参照(`IndexOutOfRangeException`)、`null` の参照(`NullReferenceException`)、辞書の欠けたキー(`KeyNotFoundException`)、型の誤り(`InvalidCastException`)、文字コードの誤り(`DecoderFallbackException`。UTF-8 の読み取りは `throwOnInvalidBytes: true` で、規定の誤りに変換する)、整数への変換の誤り(`FormatException`・`OverflowException`)、深い入れ子の再帰(下の 6.3)が、入力から起きないようにする。入力の検査に `Debug.Assert` を使わない(`Debug.Assert` は、Release のビルドで呼び出しごと取り除かれ、検査が消える)。

### 6.2 `Main` の `try`/`catch` と、スレッド

- `Main` の `catch (Exception)` は、メインスレッドで起きた例外を捕捉する。メインスレッド以外(`Process` のイベント、`PosixSignalRegistration` のコールバック、スレッドプールなど)で起きた捕捉されない例外は、`UnhandledException` に届く(6.1 の 3)。シグナルのコールバックは、フラグを立てるだけで、例外を投げない作りにする(07 章)ので、通常は起こらない。
- `catch (Exception)` は、`Exception` から派生する型をすべて捕捉する。`Exception` から派生しない例外(C++/CLI など、ほかの言語から投げられるもの)は、.NET では `RuntimeWrappedException` に包まれる(.NET の既知の挙動。この環境では出典を取得しておらず、未確認)。この仕様は、そのような経路を扱わない。
- `Environment.Exit` を `Main` の中で呼ぶコードを作らない(6.1 の 3 の最後の手段だけ)。`Main` から `return` すれば、`finally` が実行され、登録した解除(07 章の `PosixSignalRegistration` の `Dispose` など)が走る。
- ライブラリ(YAML の読み取りなど)が、`Environment.Exit` を呼ぶことは想定しない。

### 6.3 捕捉できないもの、確認した挙動

- **`StackOverflowException`(スタックの溢れ)**: 捕捉できない。確認(出典: 「AppDomain.UnhandledException」): このイベントは、スタックの溢れやアクセス違反のような、プロセスの状態を壊す例外については、呼ばれない(`HandleProcessCorruptedStateExceptionsAttribute` を付けたセキュリティ上重要なハンドラを除く)。したがって、`Main` の `catch` でも、上のイベントでも、70 にできず、プロセスは実行環境によって異常終了する(終了コードは OS によって違い、公式ドキュメントでは確認できていない。未確認)。**対策(必須)**: 入力に由来する入れ子(JSON/YAML の構造、定義の木)を、再帰で処理しない。再帰下降の解析器・書き出し器を作らず、**明示的なスタック(`Stack<T>`)で反復する**。入れ子の深さは、03 章・09 章の上限(32 段。根を数えず、33 段目を拒否する)で拒否する。YAML ライブラリが内部で再帰するかは、選定の基準に入れる(02 章)。
- **`OutOfMemoryException`**: 実際のメモリ不足では、投げられても、その後の処理(標準エラーへの書き込み、例外オブジェクトの作成)が失敗しうる。`Main` の `catch (Exception)` が捕捉できれば 70 にするが、保証しない(この仕様の対象外。`OutOfMemoryException` を握りつぶして続行しない)。入力の大きさの上限(ファイルサイズ 1 MiB、要素数 100,000、文字の大きさ 16 MiB)で、通常の入力では起きないようにする。
- そのほか、プロセスが実行環境の致命的な異常で終わる場合(実行環境のクラッシュ、OS による強制終了(`SIGKILL`)、`Environment.FailFast`)は、ハンドラが動かず、終了コードは 70 にならない。この場合は仕様の対象外とする。
- `Environment.FailFast` を、`conductor` は呼ばない。
- スレッドの中断(`Thread.Abort`)は、現在の .NET では使えない(使わない)。

ジョブの実行中に想定外の失敗が起きたとき:

- 子プロセスの終了を待たずにプロセスが終わり、ジョブが孤児として残りうる(待つことは必須ではない)。
- ステートは直前の完全な内容のままであること(書き込みは、一時ファイルに書いて置き換える形で、途中の内容が見えない形で行う。[06-state-file.md](06-state-file.md))。次の実行は、そのジョブを再実行しうる(少なくとも 1 回の方針)。

実装に任せてよい点:

- 失敗の詳細の保存や標準エラーへの追加の出力。ただし、`conductor: error: internal error` の行と 70 は必須。
- 内部の失敗を起こすための専用のコマンドや、テスト用の差し替えの口は、規定しない(製品の実行ファイルに専用のコマンドはない)。テストでは、`Deps` の差し替えで例外を投げる偽の実装を渡し、`Main` に相当するメソッド(`EntryPoint.Run` を呼ぶ入口)の戻り値が 70 で、標準出力が空であることを確認してよい。`TJ` の `inject-*` モード(11 章 3.4 節)は、実プロセスで同じことを確かめる。`UnhandledException` のハンドラ自体のテストは、別プロセスで入口を起動して、終了コードと標準エラーを確かめる(受け入れテスト。`Environment.Exit(70)` の確認は 6.1 の 3 の未確認の解消を兼ねる)。

## 7. エラー分類の一覧

内部の型の分け方は 8 章で規定する。利用者から見える区分として、次の分類と対応を守る。

### 7.1 定義の誤り(終了コード 64)

31 種。種類の識別子と、英語・日本語の文は [08-output-and-i18n.md](08-output-and-i18n.md) 4.3 に全件ある(03 章と同じ)。

識別子: `unsupportedFormat`, `fileMissing`, `unreadable`, `tooLarge`, `syntax`, `typeMismatch`, `unknownKey`, `versionMissing`, `versionUnsupported`, `unknownTarget`, `errorTransition`, `backwardOrSelfTarget`, `overlappingExitCodes`, `zeroAsWarning`, `pipelineNameMismatch`, `duplicateJobId`, `invalidJobId`, `noJobs`, `emptyField`, `duplicateTransition`, `invalidEnvName`, `retryNotTable`, `retryMissingKey`, `retryExitCodesEmpty`, `retryExitCodeInvalid`, `retryExitCodesOverlap`, `retryMaxInvalid`, `retryDurationInvalid`, `retryBackoffIncomplete`, `retryMultiplierInvalid`, `retryMaxIntervalTooSmall`。すべて終了コード 64。

`unsupportedFormat` は、定義ファイルの拡張子が `.json`・`.yaml`・`.yml` のどれでもないとき(拡張子の比較は大文字小文字を区別しない)の種類で、原因の文は 03 章が規定する(行はない)。

誤りが持つ情報: ファイル(表示するパス)、行(あれば)、種類、原因(日本語の固定の文)。表示は `<ファイル>:<行>: <種類の文>: <原因>`(行がなければ `<ファイル>: <種類の文>: <原因>`)。原因には、入力の値を含めない(入力に由来するのはキー名だけで、`` `キー名` `` の形で入り、64 文字で切る)。原因の文の全件は [03-definition-format.md](03-definition-format.md) にある。

run ID の規則違反(`Empty`, `TooLong`, `InvalidChar`, `LeadingDot` の 4 通り)は、利用者には区別を見せず、すべて `invalid value`(68)にまとめ、値を含めない。

### 7.2 `command` の誤りとステートの誤り

`command` の誤り(終了コード 64): 2 種。`unclosedQuote`(引用符が閉じていない)、`empty`(コマンド名がない)。表示は 08 章 4.4 の種類の文だけ(原因は表示しない。内部の原因の文は、前者が `引用符が閉じていません`、後者が `コマンド名がありません`)。

ステートの誤り(終了コード 65): 6 種。`unreadable`, `tooLarge`, `corrupt`, `versionMismatch`, `writeFailed`, `removeFailed`。持つ情報は、パス、種類、原因、`detail`(任意)。表示は 08 章 4.5。

定義との食い違い(終了コード 65): 3 種。`pipelineName`, `jobIds`, `jobOrder`。表示は 08 章 4.6。

ステートの原因の文(固定の日本語。言語を切り替えない)と、それを返す種類(全 23 件。06 章の表と同じ)。

| 原因の文 | 種類 | 場面 |
|----------|------|------|
| `開く、または読み取ることができません` | `unreadable` | 読み込み、`reset --all` で読めなかった |
| `大きさが上限(1 MiB)を超えています` | `tooLarge` | 読み込み |
| `UTF-8 の文字列ではありません` | `corrupt` | 読み込み |
| `JSON として読めません` | `corrupt` | 読み込み |
| `キーが重複しています` | `corrupt` | 読み込み |
| `format_version がありません` | `corrupt` | 読み込み |
| `format_version が整数ではありません` | `corrupt` | 読み込み |
| `対応していない版です(reset が必要です)` | `versionMismatch` | 読み込み |
| `知らないキーがあります` | `corrupt` | 読み込み |
| `必須のキーがありません` | `corrupt` | 読み込み |
| `値の型が誤っています` | `corrupt` | 読み込み |
| `status の値が誤っています` | `corrupt` | 読み込み |
| `skip_reason の値が誤っています` | `corrupt` | 読み込み |
| `skip_reason が status と合いません` | `corrupt` | 読み込み |
| `exit_code が status と合いません` | `corrupt` | 読み込み |
| `ジョブIDが重複しています` | `corrupt` | 読み込み |
| `job の並びが job_order と一致しません` | `corrupt` | 読み込み |
| `resume_job_id が job_order にありません` | `corrupt` | 読み込み |
| `completed と resume_job_id が両立しません` | `corrupt` | 読み込み |
| `run_id がファイル名の run ID と一致しません` | `corrupt` | 読み込み(`reset --all` では「別の定義のステート」として扱い、消さない) |
| `書き込みに失敗しました` | `writeFailed` | 書き込み |
| `内容を JSON にできません` | `writeFailed` | 書き込み |
| `削除に失敗しました` | `removeFailed` | `reset`、`reset --all`、完了時の削除 |

読み込みの検査の順序と、各原因の判定条件は [06-state-file.md](06-state-file.md)に従う。複数の原因が当てはまるときにどれを出すかの順序は、この章では未確認(06 章に従う)。

### 7.3 変換の誤り(終了コード 67)

6 種: `fileMissing`, `unreadable`, `tooLarge`, `syntax`, `unrepresentable`, `rootNotMap`。持つ情報は、ファイル、行(あれば)、キーの位置(あれば)、種類、原因。表示は 08 章 4.8。

原因の文(固定の日本語。言語を切り替えない)。変換の規則と、どの場面で使うかは [09-converter.md](09-converter.md)(JSON と YAML の相互変換)が規定する。次の表が全件で、文言は 09 章と完全に同じにする。

| 種類 | 原因の文 | 場面 |
|------|----------|------|
| `fileMissing` | `指定の場所にファイルがありません` | 入力なし |
| `unreadable` | `読み取りの権限がありません` | 権限 |
| `unreadable` | `読み取りに失敗しました` | 読み取りの失敗 |
| `unreadable` | `通常のファイルではありません` | ディレクトリなど |
| `unreadable` | `UTF-8 として読めません` | 文字コード |
| `tooLarge` | `入力が上限(1 MiB)を超えています` | 入力の大きさ |
| `tooLarge` | `要素数が上限(100000)を超えています` | 要素数の上限 |
| `tooLarge` | `文字の大きさが上限(16 MiB)を超えています` | 文字の大きさの上限 |
| `syntax` | `YAML の文法に誤りがあります` | YAML の文法 |
| `syntax` | `JSON の文法に誤りがあります` | JSON の文法 |
| `syntax` | `キーが重複しています` | キーの重複 |
| `syntax` | `キーは文字列で書きます` | 文字列でないキー(YAML) |
| `syntax` | `タグは使えません` | YAML のタグ |
| `syntax` | `複数の文書は書けません(1 ファイルに書ける文書は 1 つです)` | YAML の複数文書 |
| `syntax` | `アンカーが自分自身を含んでいます` | 再帰するアンカー |
| `syntax` | `マージキーの値がマップではありません` | `<<` |
| `syntax` | `入れ子が深すぎます(上限は 32 段)` | 入れ子の深さ |
| `unrepresentable` | `数値が表せる範囲を超えています` | 64 ビット整数・浮動小数の範囲外 |
| `unrepresentable` | `NaN と無限大は JSON で表せません` | YAML の `.nan`、`.inf` を JSON へ |
| `unrepresentable` | `キーが長すぎます(上限 1024 文字)` | 1,024 文字を超えるキーを JSON から YAML へ |
| `rootNotMap` | `最上位がマップではありません` | 最上位 |
| `rootNotMap` | `文書が空です` | YAML の文書が 0 個 |

- `null` は、どちらの形式でも表せる値であり、変換の誤りにならない。日時らしい書式は文字列として読むので、変換の誤りにならない。表の理由は、この 22 件で全件である(未定義のアンカーの参照は、YAML の文法の誤りとして `YAML の文法に誤りがあります` になる)。
- 「キーが長すぎます(上限 1024 文字)」の長さは、コードポイント単位で数える(C# では、サロゲートの対を 1 文字と数える。`string.Length` はコードユニット数なので使わない。04 章 4.0、08 章 11 章)。

キーの位置の形式: キーは `.` で連結(各キーは 64 文字で切る)、配列の添字は `[i]`、文字列でないキーは `(非文字列キー)`。例: `job[2].transitions[0]`。

種類と原因の組は、上の表のとおり(09 章と同じ)。

### 7.4 使い方の誤り(終了コード 68)

8 種: `UnknownCommand`, `MissingArgument`, `UnexpectedArgument`, `UnknownOption`, `OptionNotAllowed`, `MissingValue`, `InvalidValue`, `UnknownJob`。文言は 08 章 4.2。誤りは、種類と、対象(あれば)を持つ。対象は、誤りの対象の語(知らないコマンド、オプション名、ジョブID、`run ID (environment variable)` など)。入力の値は、それ以外には含めない(`--run-id` の値、環境変数の値は含めない)。

発生条件:

| 発生条件 | 種類 | 対象 |
|----------|------|------|
| 位置引数なし | `MissingArgument` | なし |
| コマンド名が未知(空文字も) | `UnknownCommand` | その語 |
| コマンド名が文字として正しくない(04 章 4.4.2) | `InvalidValue` | 読める形にした語(04 章 4.4.2) |
| 文法にないオプション、`--bogus=x` | `UnknownOption` | `--bogus`、`-x` など |
| そのコマンドで使えないオプション | `OptionNotAllowed` | `-v`, `--config`, `--to`, `--run-id`, `--all`, `--remove-on-complete` |
| 値のないオプション | `MissingValue` | `--config`, `--run-id`, `--to` |
| 重複、`--verbose=x` など、`--all` と `--run-id` の併用、`--run-id` の値の規則違反(文字として正しくない値を含む) | `InvalidValue` | オプション名(`-v`, `--config`, …、併用は `--all`) |
| `--to` の値が `json`/`yaml` 以外(文字として正しくない値を含む) | `InvalidValue` | その値(読める形) |
| ジョブID・パイプライン名が文字として正しくない | `InvalidValue` | 読める形にした語 |
| 値を取るオプションの直後が `-h`/`--help` | (値として取られるので help にならない。ほかに位置引数がなければ `MissingArgument`) | なし |
| 環境変数の run ID の規則違反 | `InvalidValue` | `run ID (environment variable)` |
| 余分な位置引数 | `UnexpectedArgument` | その語 |
| `convert` で `--to` なし | `MissingArgument` | `--to` |
| `run` のジョブIDが定義にない | `UnknownJob` | そのジョブID |

(環境変数の run ID が文字として正しくないときは、未設定と同じ扱いで、誤りにならない。04 章 5.1。)

### 7.5 シグナルの準備の誤り

2 種。「すでに登録済み」(成功と同じ扱い)、「失敗」(`conductor: error: failed to set up signal handling`、70)。

## 8. C# でのエラー型の設計

### 8.1 方針と、選択肢の比較

規定された誤りの伝え方には、3 つの選択肢がある。

| 案 | 内容 | 長所 | 短所 |
|----|------|------|------|
| A: すべて例外 | 規定された誤りを、`Exception` を継承した型で `throw` する | 書き方が簡単。呼び出しの途中の層で、伝え方を意識しなくてよい | 定義の誤りは、複数の誤りをまとめて 1 行ずつ出す必要があり(03 章)、1 つの例外では 1 件しか持てない(複数を持たせる専用の型が要る)。例外は、スタックトレースの取得などを伴い、通常の分岐の道具としては重い。どこで何が投げられるかが、型のシグネチャに表れず、取りこぼしが、想定外の失敗(70)に化ける |
| B: すべて結果型 | 値(`Result<T, TError>`)で返し、例外は想定外の失敗だけ | 失敗が型に出る。複数の誤りを値として持てる。`catch` の取りこぼしがない | ファイル・プロセスなど、OS の境界で例外を投げる API(`IOException`、`Win32Exception`)を、そのつど結果型に変える必要があり、冗長になる |
| C: 混合(採用) | 規定された誤りは、ステートの読み書き(`Conductor.State`)を含め、すべて**値で返す**(章ごとの `abstract record` の階層。`LoadResult`・`StateOutcome`・`CommandResult`・`ConvertResult`・`RunIdParse` など。`Result<T, TError>` は、その階層を使わない小さな内部の関数の補助で、任意)。OS の境界の BCL の例外は、その場で狭く捕まえて値に変える。**例外として送出する規定された型は、標準出力の書き込みの失敗の `OutputException` だけ**(呼び出し元が 65 か 67 にする) | 失敗が型に出る。複数の誤りを値として持てる。型の方針が 8.3 の表で一意に決まる | OS の境界で、BCL の例外を値に変える記述が要る(境界の内側に閉じ込める) |

採用は C。規定された誤りと、想定外の内部失敗(6 章)を、**型で区別する**: 規定された誤りは、8.3 の型の集合に属し、それ以外は、想定外とする。

- 各誤りの型は、「種別(`enum`。閉じた値の集合)」と「利用者に見える情報」を持つ。文言への変換(ロケール、エスケープ)は `Conductor.Report` の責務で、例外の `Message`(`Exception.Message`)は、ロケールを判定せず、デバッグ用の固定の英語の 1 行にする(利用者向けの出力には使わない)。
- 終了コードは、エラー型の値から、`Conductor.Cli` の 1 つのメソッドで決める(型ごとの `switch` 式)。終了コードの数値の定義は 1 か所(8.2 の `ExitCode`)に集める。
- 呼び出し側は、型の判定(`is`、`switch` 式、`OutputException` の `catch`)で扱う。**`catch (Exception)` と、`catch { }` を、規定された誤りの処理に使わない**(想定外の失敗を握りつぶさないため)。想定外の失敗を捕捉してよいのは、`Program.Main` の最上位(6 章)と、08 章 1.3.2 の書き込みの失敗の処理(`IOException` などの**列挙した型に限る**)だけ。OS の境界での例外の変換(`IOException` 系、`UnauthorizedAccessException`、`Win32Exception` などを、その境界の規定された種類に対応づける。例: 定義ファイルの `FileNotFoundException`・`DirectoryNotFoundException` は `fileMissing`、`UnauthorizedAccessException` は `unreadable`)は、`catch` する型を列挙して行い、列挙にない例外は伝える(70 になる)。どの操作がどの型の例外を投げるかの詳細は、公式ドキュメントを取得しておらず、未確認(実装時に確かめ、確かめた対応を 03・06・07・09 章に書く)。
- 誤りを例外で包む場合(`OutputException`)は、内部の例外を `InnerException` に入れる(`new OutputException(message, inner)`)。値の誤り(`StateError` など)は、下位の例外の文を持たない(固定の理由だけ)。包んでも、利用者向けの出力は、包んだ誤りの型の情報だけから作る(`InnerException` や `Message` は、利用者に出さない)。
- 種別の `enum` の値(`ErrorKind.FileMissing` など。7.1 の識別子の先頭を大文字にした名前)は、永続化も外部への公開もしない(利用者に見えるのは文言だけ)。名前を出力に使わない: `enum` の `ToString()` を、利用者向けの文言や、ステートの値に使わない(識別子は、文言の表のキーとして使う)。並びの追加・並べ替えで、互換性の問題は起きない。
- 種別の `switch` 式の網羅性: 種別に対する `switch` 式は、`_` の捨てのアームを置かずに書き、全メンバーを列挙する。コンパイラが、メンバーの取りこぼしや、名前のない値(`(Kind)99`)を、警告(`CS8509`・`CS8524` に相当するもの)にするかは、この環境では公式ドキュメントを取得しておらず、未確認(`TreatWarningsAsErrors` と組み合わせたときの挙動を含む)。したがって、網羅性は、単体テストで保証する(8 章 10 章のとおり、全メンバーを `Enum.GetValues<T>()` で列挙して、文言が全ロケールにあることを検査する)。
- 例外のクラスは、`[Serializable]` や、シリアライズ用のコンストラクタを持たせない(誤りを直列化・複製しない)。`Exception` の基底のコンストラクタに、`Message`(固定の英語の 1 行)を必ず渡す。

### 8.2 定数と共通の基底クラス

```csharp
namespace Conductor.Flow;

/// 終了コード(1 章)。値は 1 か所に集める。
public enum ExitCode : int
{
    Ok = 0,
    UnlistedZero = 1,
    Definition = 64,
    State = 65,
    Rejected = 66,
    Conversion = 67,
    Usage = 68,
    Internal = 70,
    StartFailure = 127,
    SignalBase = 128,
}

public static class ExitCodes
{
    public static int For(int reported, bool isError);      // 4 章(05 章 2.2 節)
    public static int ForSignal(int signal);                 // 4 章
}
```

```csharp
namespace Conductor;

/// 規定された誤りを例外として送出する型が必要になったときの基底クラス(任意)。
/// この版では、規定された誤りはすべて値で返し、例外は OutputException だけなので、これを継承する型はない。
public abstract class ConductorException : Exception
{
    protected ConductorException(string debugMessage, Exception? inner = null) : base(debugMessage, inner) { }

    /// この誤りの終了コード(ExitCode の値を int にしたもの)。
    public abstract int ExitCodeValue { get; }
}
```

- `ExitCode` は `enum : int` だが、`Main` の戻り値と `ExitCodes.For` には、`(int)ExitCode.X` で、`int` にして渡す(`enum` から `int` への変換は、明示的なキャストが要る)。
- 結果型(値で返す誤り): `DefinitionError`・`ConvertError`・`RunIdError`・`CommandError` は、`Exception` を継承**しない**、不変の `sealed record`(または `readonly record struct`)にする。理由: これらは `throw` されず、結果の型(`Result<T, TError>`)に入って返るだけで、`Exception` を継承すると、スタックトレースの保持など、不要な状態を持つため。

```csharp
namespace Conductor;

/// 成功 (Ok) または規定された誤り (Err) のどちらかを持つ。3 行以上の分岐には switch 式を使う。
public readonly struct Result<T, TError>
{
    public bool IsOk { get; }
    public T Value { get; }          // IsOk のときだけ有効
    public TError Error { get; }     // !IsOk のときだけ有効
    public static Result<T, TError> Ok(T value);
    public static Result<T, TError> Err(TError error);
}
```

(骨格。型の名前と分け方は、結果の型を表せれば変えてよい。成功・失敗を判別共用体で表す `abstract record` の階層でもよい。)

### 8.3 型の一覧(骨格)

プロジェクト(名前空間)ごとに、自分の誤りの型を持つ(構成は 02 章)。

| プロジェクト / 名前空間 | 型 | 種別の型 | 終了コード | 持つ情報(伝え方) |
|----------------------------|-----|----------|------------|----------|
| `Conductor.Core` / `Conductor.Definition` | `DefinitionError` | `ErrorKind`(`enum`。7.1 の 31 種。03 章) | 64 | ファイル、行・列(`int?`。`null` は「なし」)、種類、原因(日本語の固定の文)。例外にせず、`sealed record` の値として `Result` に入れて返す(03 章)。複数の誤りは `IReadOnlyList<DefinitionError>` で返す |
| `Conductor.Runtime` / `Conductor.Runner` | `CommandError` | `CommandErrorKind`(`enum`: `UnclosedQuote`, `Empty`。07 章) | 64 | 種類(ジョブIDは、呼び出し側が持つ)。値で返す(07 章) |
| `Conductor.Core` / `Conductor.RunIds` | `RunIdError` | `RunIdErrorKind`(`enum`: `Empty`, `TooLong`, `InvalidChar`, `LeadingDot`。05 章) | 68(利用者には `invalid value`) | 種類のみ(値は持たない)。結果の型で返す(05 章) |
| `Conductor.Runtime` / `Conductor.State` | `StateError`(`sealed record`)を持つ `StateFail`(`StateOutcome` の派生) | `StateErrorKind`(`enum`: `Unreadable`, `TooLarge`, `Corrupt`, `VersionMismatch`, `WriteFailed`, `RemoveFailed`。06 章) | 65 | パス、種類、原因(固定の日本語の文)、`Detail`(任意)。ステートの読み書きのメソッドは、例外を送出せず、`StateFail` を値で返す(06 章 13.1)。BCL の例外は、`IStateStore` の実装の内側で値に変える |
| `Conductor.Runtime` / `Conductor.State` | `ConsistencyMismatch`(`ConsistencyResult` の派生) | `MismatchWhat`(`enum`: `PipelineName`, `JobIds`, `JobOrder`。06 章) | 65 | 種類。値で返す(06 章) |
| `Conductor.Core` / `Conductor.Convert` | `ConvertError` | `ConvertErrorKind`(`enum`。7.3 の 6 種。09 章) | 67 | ファイル、行、キーの位置、種類、原因。値で返す(09 章) |
| `Conductor.Cli` / `Conductor.Cli` | `UsageError` | `UsageKind`(`enum`。7.4 の 8 種) | 68 | 種類、対象(`string?`)。値として持ち回る(04 章) |
| `Conductor.Core` / `Conductor.Flow` の `Reject` | `DecisionReason.OutOfOrder` | (1 種) | 66 | ジョブID。`Flow.DecideBeforeRun` が `Reject(OutOfOrder)` の値で返し、`Cli` が 66 にする(送出しない) |
| `Conductor.Cli` の `Conductor.Report` | `OutputException`(`Exception` を継承。`Conductor.Report`) | (なし) | 呼び出し元による(`watch` は 65、`convert` は 67) | 内部の例外を `InnerException` に持つ。標準出力への書き込みの失敗(08 章 1.3.2)。`ConductorException` を継承しない(終了コードが文脈で決まるため) |
| `Conductor.Core` / `Conductor.Flow` | (クラスにしない。`JobOutcome` の値) | (なし。分類の結果として扱う) | 3 章から決まる | 報告する終了コード、エラーか |

- 種別の `enum` は、すべて「値の集合が型で閉じている」。メンバー名は、7 章の識別子の先頭を大文字にした形(`unsupportedFormat` → `UnsupportedFormat`)。`enum` の値の数値(`= 0, 1, …`)を、永続化・出力に使わない(宣言の順序は、意味を持たない)。
- 「対象なし」と「対象が空文字」を区別する(`unknown command: ''` のように、対象があれば `'...'` で囲んで出す)。`string? Target` として、`null` を「対象なし」、`""` を「対象が空文字」とする(判定は `Target is null`。`string.IsNullOrEmpty(Target)` は、空文字を「なし」と誤るので使わない)。`<Nullable>enable</Nullable>`(01 章)で、`null` の可能性を型に表す。
- 送出する誤りは、必ず `Exception` を継承したクラスにする(`throw` できるのは `Exception` から派生した型だけ)。値で返す誤りと、送出する誤りを、同じ型で兼ねない。

```csharp
namespace Conductor.Cli;

public enum UsageKind
{
    UnknownCommand,
    MissingArgument,
    UnexpectedArgument,
    UnknownOption,
    OptionNotAllowed,
    MissingValue,
    InvalidValue,
    UnknownJob,
}

/// target: null は「対象なし」、"" は「対象が空文字」。
public sealed record UsageError(UsageKind Kind, string? Target = null);
```

### 8.4 終了コードへの対応

`Conductor.Cli` の 1 か所のメソッド(例: `ExitCodeMapper.From(...)`)が、すべての規定された誤りの型から、`int` の終了コードを決める(型ごとの `switch` 式。8.3 の表の「終了コード」の列)。`OutputException` は、呼び出し元(`watch` か `convert`)が、`ExitCode.State` か `ExitCode.Conversion` を決める。その他の例外は、このメソッドに届かず、`Main` の `catch (Exception)` が 70 にする(6 章)。

## 9. 必須の挙動と、実装に任せてよい点

必須:

- 1 章の終了コードの全件、2 章の決定順序、3 章の分類、4 章の範囲合わせ(`ExitCodes.For` の判定表)、`Main` の戻り値で終了コードを返すこと(4.1)。
- 6 章: `Main` の `try`/`catch (Exception)`、`UnhandledException`・`UnobservedTaskException` のハンドラ、`conductor: error: internal error` の 1 行(固定の英語。標準エラー)と 70、入力に由来する入れ子を再帰で処理しないこと(6.3)。
- 7 章の分類と、8 章の型の区別(規定された誤りと想定外の失敗を型で区別し、`Exception` の握りつぶしをしない)。

実装に任せてよい点:

- 内部の型の名前と分け方(8 章は目安。結果型を、判別共用体の `record` の階層にするなど)。
- 失敗の詳細の保存や、標準エラーへの追加の出力(6.1 の 5)。

## 10. 未確認

- `UnhandledException` のハンドラの中から呼んだ `Environment.Exit(70)` が、終了コード 70 でプロセスを終わらせること(6.1 の 3)。ハンドラが戻ったときの、実行環境の既定の終了コード(OS ごとの値)。
- スタックの溢れで、プロセスが終わるときの終了コード(6.3)。
- 子プロセスの起動の失敗で `Process.Start` が投げる例外の型の全件と、`Process.ExitCode` の、シグナルで終了した子に対する値(07 章が規定する。3 章)。
- Unix で、範囲外の終了コード(`256` など)が、OS により下位 8 ビットになること(4.1。第三者の報告のみ。公式ドキュメントでは確認していない)。Windows での 255 を超える値の扱い(`Environment.ExitCode` の符号つきの記述のみ)。
- `int` の `&` が負の値で 2 の補数の下位 8 ビットを返すこと(言語の仕様どおり。出典のページは取得していない。4 章)。
- シグナルの準備をしないコマンド(`watch`・`reset`・`validate`・`convert`)の実行中の Ctrl+C の、実行環境の既定の動作(5 章)。
- 種別の `switch` 式の網羅性に関するコンパイラの警告の挙動(8.1)。
- OS の境界で、どの操作がどの例外の型を投げるか(8.1)。
- `RuntimeWrappedException` の挙動(6.2)。
- サポートする最小の .NET の版(実装時に確認する。最新の LTS と、サポート中の 1 つ前の LTS)で、6 章の確認を、3 OS で繰り返すこと。
