# 08. 出力と多言語化

この章は、C#(.NET)で実装する `conductor` が出す文字列の全文、出力先、標準出力・標準エラーへの書き込みの規則(エンコーディング、BOM なし UTF-8、LF、フラッシュ、閉じたパイプ、終了前の出力の完了)、言語の決め方、`watch` の表、色、エスケープの規則(サロゲートを含む)、表示幅と埋め、文字列の単位の扱いを規定する。文言は、この章の表のとおりに(文字単位で)出力する。コマンドラインは [04-cli.md](04-cli.md)、終了コードとエラー型は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md)、定義の誤りの原因の文は [03-definition-format.md](03-definition-format.md)、ステートの形式は [06-state-file.md](06-state-file.md)、変換は [09-converter.md](09-converter.md)、子プロセスとシグナルは [07-process-and-signals.md](07-process-and-signals.md) を参照する。

「確認(出典: …)」は、公式ドキュメントまたは公開されているランタイムのソースを取得して確かめた内容、「未確認」は、この環境に .NET SDK がなく実機で動かせないこと、または出典を取得できなかったことで、確かめられていない内容である。

実装は `src/Conductor.Cli/Report/`(名前空間 `Conductor.Report`)に置く。処理の流れは、「表示する内容」(種類と値を持つ型)を `Reporter` に渡し、`Reporter` がロケールと水準に従って文言に直して書く、という分担にする。

## 0. この版の文言の決め(まとめ)

文言は、この章の表のとおりに(文字単位で)出力する。この版が採った書式(JSON を主、YAML を副)に関わる文言は次のとおり。

| 場所 | 文言 |
|------|------|
| 使い方(英語)の `convert` の行 | `conductor convert <file> --to <json\|yaml>` |
| 使い方(日本語)の `convert` の行 | `conductor convert <ファイル> --to <json\|yaml>` |
| 使い方の `--config` の既定(英語) | `(default: conductor.json in the current directory)` |
| 使い方の `--config` の既定(日本語) | `(省略時: 実行ディレクトリの conductor.json)` |
| 使い方の `--to`(英語) | `--to <json\|yaml>    target format of convert` |
| 使い方の `--to`(日本語) | `--to <json\|yaml>    convert の変換後の形式` |
| ステートの原因: 構文の誤り | `JSON として読めません` |
| ステートの原因: 書き出しの失敗 | `内容を JSON にできません` |
| 定義の誤りの種類 `retryNotTable`(英語 / 日本語) | `retry must be a map` / `retry はマップで書きます` |
| 定義の誤りの種類 `unsupportedFormat`(英語 / 日本語) | `unsupported definition file format` / `定義ファイルの形式に対応していません` |
| 例の中の定義・ステートのファイル名 | `c.json`、`c.json.state` など |

(表の中の `\|` は、Markdown の表のための書き方で、実際の文言は `|` 1 文字。)

- ステートの原因 2 件は、ステートファイルが JSON であるため(10 章 7.2 に全件)。
- `retryNotTable` の識別子は内部の名前であり、利用者には見えない。文言は、JSON/YAML の語である「マップ(キーと値の対応)」で言う。
- 定義の誤りの「原因」の文(日本語の固定の文)は、[03-definition-format.md](03-definition-format.md) が規定する。この章は、種類の文(4.3)だけを規定する。
- 変換の誤りの「原因」の文は、[09-converter.md](09-converter.md) が規定する。10 章 7.3 に全件を再掲する。

## 1. 出力先の使い分け

表示は、処理の流れが作る「表示する内容」を、文言に直して、水準に従って出す。水準は 4 つある。

| 水準 | 出力先 | 出す条件 | 書き込み失敗 |
|------|--------|----------|--------------|
| `Error` | 標準エラー | 常に | 無視(本筋を止めない) |
| `Notice` | 標準エラー | 常に(エラーではない知らせ。リトライの前の知らせだけ) | 無視 |
| `Verbose` | 標準エラー | `-v` のときだけ | 無視 |
| `Output` | 標準出力 | 常に | 呼び出し元に失敗を返す(`watch` は 65、`convert` は 67 で終わる) |

- どの文言も、最後に改行(LF、`\n`)を含む。複数の誤りは 1 行ずつ。
- 標準出力に出すのは、`watch` の表(と `no state`)、`--help`、`--version`、`convert` の変換結果だけ。
- ジョブ本体の標準出力・標準エラーは `conductor` が加工せず、そのまま流れる(子プロセスの標準入出力をリダイレクトせず、子が親と同じファイル記述子を直接使う。07 章)。`conductor` の表示は標準エラーなので、標準出力にはジョブの出力だけが残る。
- 成功・スキップで `-v` なしのときは、表示が一度もなく、言語も色も判定されない(必要になった最初の時点まで判定を遅らせる)。この遅延は、観測できる差(環境変数の読み取り順)を生まないなら、実装に任せてよい。

### 1.1 水準ごとのイベント

| 水準 | イベント |
|------|----------|
| Error | 定義の誤り、`command` の誤り、使い方の誤り(+使い方全文)、ステートの誤り、定義との食い違い、`reset --all` の警告(`conductor: warning:`)、拒否、起動失敗、成功コードに含まれない 0、変換の誤り、標準出力に書けない、完了を判定できない、シグナルの準備の失敗、内部の誤り |
| Notice | リトライの前の知らせ(`conductor: retry:`) |
| Verbose | ジョブの結果、スキップ、遷移、中断、`validate` が通った |
| Output | `watch` の表、`watch` の空、`--help`、`--version`、`convert` の変換結果 |

### 1.2 標準出力・標準エラーのエンコーディングと改行

#### 1.2.1 確認した挙動

| 確認した事項 | 結果(出典) |
|--------------|--------------|
| `Console.OutputEncoding` の既定 | 確認(出典: 「Console.OutputEncoding」): コンソールは、出力エンコーディングで、アプリが書いた文字をコンソールに表示する文字へ変換する。既定のコードページは、システムのロケールで決まる。Unix では、確認(出典: ランタイムのソース `ConsolePal.Unix.cs`): 環境変数の文字集合の指定(`EncodingHelper.GetEncodingFromCharset()`)から得たエンコーディングから、バイト順マークを除いたものを返し、得られなければ `Encoding.Default`。つまり、既定の出力エンコーディングは、環境(ロケール)に依存する |
| `Console.OutputEncoding` にできるエンコーディング | 確認(出典: 同): Unicode のうち、UTF-8(`UTF8Encoding`)と UTF-16(`UnicodeEncoding`)は使える。UTF-32 は使えず、設定すると `IOException`。コンソールに Unicode を表示するには、TrueType のフォントなどが要る |
| `UTF8Encoding` の BOM と不正な文字 | 確認(出典: 「UTF8Encoding Constructor」): `new UTF8Encoding(encoderShouldEmitUTF8Identifier: false, throwOnInvalidBytes: ...)` の `false` は `GetPreamble()` が長さ 0 の配列を返す(BOM なし)。`throwOnInvalidBytes` が `true` のとき、不正な列に出会うと例外(例の出力では、対になっていないサロゲート U+D801 の符号化で `EncoderFallbackException`)。`false` のときは、対になっていないサロゲートを `EF BF BD`(U+FFFD の UTF-8)に置き換えて符号化する(例の出力: `61 62 63 EF BF BD EF BF BD 64`) |
| `Console.OpenStandardOutput()` | 確認(出典: 「Console.OpenStandardOutput」): 標準出力のストリーム(`Stream`)を返す。`Console.SetOut` で変えたあとに、標準出力を取り直すのにも使える。文書の例は、`new StreamWriter(Console.OpenStandardOutput())` に `AutoFlush = true` を付けて `Console.SetOut` に渡している。`OpenStandardOutput(int bufferSize)` の `bufferSize` は「効果がない」 |
| `Console.IsOutputRedirected` | 確認(出典: 「Console.IsOutputRedirected」): 標準出力のストリームからリダイレクトされているかを表す `bool`。Unix では、確認(出典: ランタイムのソース): 標準出力(fd 1)が端末(`isatty`)でなければ真 |

#### 1.2.2 規定

1. `Console.Out`・`Console.Error`・`Console.WriteLine`・`Console.Write` を、`conductor` の表示に**使わない**(`Console.OutputEncoding` が環境依存であり、`WriteLine` の改行は `Environment.NewLine`(Windows で CRLF)になりうるため)。次のとおり、自前の出力口を作る。

   ```csharp
   namespace Conductor.Report;

   internal static class OutputStreams
   {
       // BOM なし。不正な文字(対になっていないサロゲート)で例外
       public static readonly UTF8Encoding StrictUtf8 = new(encoderShouldEmitUTF8Identifier: false, throwOnInvalidBytes: true);
       // BOM なし。不正な文字は U+FFFD(EF BF BD)にする
       public static readonly UTF8Encoding LenientUtf8 = new(encoderShouldEmitUTF8Identifier: false, throwOnInvalidBytes: false);
   }

   // 標準出力: new StreamWriter(Console.OpenStandardOutput(), OutputStreams.StrictUtf8)
   //             { AutoFlush = true, NewLine = "\n" }
   // 標準エラー: new StreamWriter(Console.OpenStandardError(), OutputStreams.LenientUtf8)
   //             { AutoFlush = true, NewLine = "\n" }
   ```

   - 文字コードは UTF-8、**BOM なし**(`StreamWriter` は、エンコーディングの `GetPreamble()` が空なら、BOM を書かない。上の確認のとおり、`encoderShouldEmitUTF8Identifier: false` の `UTF8Encoding` は空の配列を返す)。ロケールにも `Console.OutputEncoding` にも依存しない。
   - 改行は **LF だけ**。`WriteLine`・`Environment.NewLine` を使わず、文言の末尾の `"\n"` を、文字列に含めて書く。`NewLine = "\n"` の設定は、うっかり `WriteLine` を呼んでも CRLF にならないための保険(Windows のコンソール・ファイル・パイプのどれでも LF だけにする)。
   - **`AutoFlush = true`**: 1 回の `Write(string)` ごとに、書き込みの末尾でストリームに流す。表示は、1 回の表示(1 行、または 1 かたまり)につき、組み立て済みの 1 つの文字列を 1 回の `Write(string)` で書く(複数回の `Write` に分けない)。それに加えて、`AutoFlush` を過信せず、`Write` のあとに `Flush()` も明示して呼ぶ(多重の呼びで害はない)。`StreamWriter` はファイナライザでバッファを流さない(.NET の既知の挙動。取得した文書では確認していない。未確認)ので、`Flush()` を呼ばずに終わると出力が失われうる。1.4 の終了の手順で必ず流す。
   - 標準出力は `throwOnInvalidBytes: true`: 対になっていないサロゲートが入り込んだら、黙って壊れた出力にせず、`EncoderFallbackException`(上の確認のとおり、例外の型)を、書き込みの失敗として扱う(1.3)。通常の流れでは、表示する文字列は 9 章のエスケープ(対になっていないサロゲートを U+FFFD にする)を通っているので、符号化の失敗は起こらない。標準エラーは `throwOnInvalidBytes: false`(`conductor` の表示が、符号化の失敗で止まらないようにする。対になっていないサロゲートは U+FFFD になる。通常の流れでは到達しない)。
   - 2 つの `StreamWriter` は、起動時に 1 度だけ作る(最初の表示が必要になるまで作りを遅らせてよい。その場合も、作りの失敗は書き込みの失敗として扱う)。`Console.OpenStandardOutput()`・`Console.OpenStandardError()` を、表示のたびに呼ばない。
   - 作った `StreamWriter` と、その下のストリームを `Dispose` しない(`Dispose` が標準出力のハンドルを閉じるか、取得した文書では確認していない。未確認。プロセスの終了まで開けておく)。
   - ストリームの作り(`Console.OpenStandardOutput()` など)が例外を投げる場合(標準出力が閉じられているとき。挙動は未確認)は、その出力口を「書き込みに失敗した」ものとして扱う(標準エラーは無視、標準出力は 1.3 の規則)。
2. `Console.OutputEncoding` を、Unix では**設定しない**(読まない。上の自前の出力口が、環境に依存せず UTF-8 のバイト列を書くため)。
3. Windows のコンソール(端末)への出力: 標準出力が端末のとき(`Console.IsOutputRedirected` が偽)は、起動時に、`Console.OutputEncoding = new UTF8Encoding(false)`(BOM なし)を設定してから、出力口を作る。標準エラーが端末のときも同じ(`Console.IsErrorRedirected`)。設定の失敗(`IOException`、`UnauthorizedAccessException`)は無視する。コンソールに、UTF-8 のバイト列を、日本語を含めて正しく表示できることを意図する。**未確認**: 標準ストリームに UTF-8 のバイト列を書いたとき、Windows のコンソールが、コンソールの出力コードページで解釈するか(その場合、`OutputEncoding` の設定が必要)、`OutputEncoding` を設定すると、終了後もコンソールのコードページが変わったままになるか(公式ドキュメントに、SDK の版による差を扱う互換性の記述があるが、挙動の詳細は取得していない)。標準出力・標準エラーをファイルやパイプにリダイレクトしたときは、この設定をせず、UTF-8 のバイト列がそのまま出る(未確認)。Windows のコンソールのフォントや ANSI の制御列の可否は、この仕様では確認しない。色は常に使わない(6.3)。
4. ジョブ本体の出力は `conductor` を通らないので、この設定の対象外(子のエンコーディングは子が決める)。

### 1.3 書き込みの方式と、標準出力が閉じている・書けないとき

#### 1.3.1 確認した挙動

| 確認した事項 | 結果(出典) |
|--------------|--------------|
| Unix で、読み手が閉じたパイプへの書き込み(`EPIPE`) | 確認(出典: ランタイムのソース `ConsolePal.Unix.cs` の書き込み): 書き込みが `EPIPE` で失敗したとき、コードは、「Broken pipe... likely due to being redirected to a program that ended, so simply pretend we were successful.」(リダイレクト先のプログラムが終わったためと思われるので、成功したふりをする)というコメントのとおり、**例外を投げずに戻る(データは捨てられる)**。つまり、`Console.OpenStandardOutput()` と `Console.OpenStandardError()` のストリームでは、Unix で、閉じたパイプへの書き込みを、`IOException` として検出できない |
| 上の以外の書き込みの失敗 | 確認(出典: 同): `EAGAIN`(非ブロッキングのハンドル)は、書き込み可能になるまで待って再試行する。それ以外の `errno` は、例外(`Interop.GetExceptionForIoErrno` が作る例外。`IOException` 系)を投げる。例: 容量不足(`ENOSPC`)や、書き込みできないデバイスへの書き込み |
| `SIGPIPE` | 確認(出典: ランタイムのソース `src/coreclr/pal/src/exception/signal.cpp`): 実行環境の初期化で `signal(SIGPIPE, SIG_IGN)` を呼ぶ(コメントは、`SIGPIPE` の既定の動作がプロセスの終了であり、ソケットなどへの書き込みで `SIGPIPE` が発生しうることと、無視すると `errno` が `EPIPE` になることを述べている)。つまり、閉じたパイプへの書き込みで、プロセスが信号で死ぬことはない |
| Windows で、読み手が閉じたパイプへの書き込み | 取得できる出典がなく、**未確認**(`IOException` になることを想定する) |
| `Environment.Exit` と、バッファされた出力 | 確認(出典: 「Environment.Exit」): `Exit` は、ほかのスレッドが動いていても、アプリを直ちに終了させる。`try`/`catch` の中で呼ぶと `finally` は実行されない。`return`(`Main` から返る)は、フォアグラウンドのスレッドがすべて終わるのを待つ。`StreamWriter` のバッファを流すかは、取得した文書に記述がない(未確認)ので、`Environment.Exit` の前に必ず自分で `Flush()` する |

要するに、(1) 閉じたパイプへの書き込みで、プロセスが信号で死ぬことはない。(2) Unix では、閉じたパイプへの書き込みが「成功したふり」になり、`IOException` にならない。**この版では、Unix で `watch | head -0` のように読み手が先に閉じたとき、`watch` は 65 ではなく 0 で終わりうる**(`convert` も 67 ではなく 0)。この差は、.NET の標準のストリームの設計によるもので、受け入れる。閉じたパイプの検出を、標準のストリームを使わずに行う方法(標準出力のファイル記述子を直接 `FileStream` に包む方法など)は、動作を確認できておらず(未確認)、この版の必須にしない。(3) 容量不足などの、`EPIPE` 以外の失敗は、例外になる(1.3.2 で 65・67 に結びつける。Linux では、`/dev/full` への出力で再現できる想定。未確認)。

#### 1.3.2 規定

1. **書き込みは、出力口(`Conductor.Report` の `IOutputSink`)の 1 つのメソッドを通す**。本番の実装は、次の形にする(骨格。実装コードは規定しない)。

   ```csharp
   internal interface IOutputSink
   {
       /// text を 1 回の Write で書き、Flush する。失敗は例外(下の捕捉の対象)で伝える。
       void WriteAll(string text);
   }
   ```

   - ストリームが作れていないとき、`IOException` を送出する(本番の実装は、作りの失敗を `IOException` に包んで、書き込みの失敗として扱う)。
   - `Write(text)` のあと、必ず `Flush()` を呼ぶ。標準出力・標準エラーとも、行ごとではなく、1 回の表示(1 行、または 1 かたまり)につき 1 回の `Write` と 1 回の `Flush`。
   - 捕捉する例外は、`IOException`(閉じたパイプ、容量不足などを含む)、`ObjectDisposedException`(閉じたストリームへの書き込み)、`EncoderFallbackException`(符号化できない文字)、`NotSupportedException`(書き込みできないストリーム)に**限る**。これらを、書き込みの失敗として扱う。**`Exception` 全体を握りつぶさない**(想定外の失敗が、書き込みの失敗に化けないように)。
2. 水準ごとの書き込みの失敗の扱い:
   - `Output` 水準(標準出力): 失敗を、`Reporter.Output()` が `OutputException`(10 章 8.3)として呼び出し元に送出する。`watch` は `cannot write to standard output` を標準エラーに出して 65、`convert` は同じ文を出して 67。
   - `--help` と `--version` の出力: 書き込みの失敗は無視して、終了コード 0 で終わる(失敗を報告しない)。
   - `Error`・`Notice`・`Verbose` 水準(標準エラー): 失敗を握りつぶして続ける(本筋を止めない)。
3. **標準エラーの書き込みも、`Flush()` まで完了してから次へ進む**。パイプ・ファイルの先でも、`conductor` の行が、そのあとに起動する子プロセスの出力より前に出るようにするため(子は同じファイル記述子を直接使うので、親の書き込みが親のバッファに残ったまま子を起動すると、順序が入れ替わりうる)。特に、リトライの知らせ(`Notice`)と、ジョブの起動前に出す行は、`Flush()` を終えてから子を起動する。
4. 書き込みは、すべて同じスレッド(メインスレッド)で、呼び出しの順に直列に行う。シグナルのコールバック(スレッドプールなどで実行されうる。07 章)は、出力しない(フラグを立てるだけ)。
5. **終了時に、失敗したストリームのバッファを残さない**: 自前の出力口は `AutoFlush = true` で、書き込みのたびに流し切るので、失敗した書き込みのデータが、プロセスの終了時に、もう一度失敗して終了コードを変える、という経路を持たない(取得した文書では、`StreamWriter`・`Console` のストリームが、終了時にバッファを流して失敗することは確認していない。ファイナライザや終了処理による流し直しの有無は未確認)。書き込みに一度失敗した出力口は、以降の書き込みを行わず(失敗を覚えて、何もしない)、1.4 の終了の流しも飛ばす。
6. `SIGPIPE` の設定を変えない(ランタイムが初期化時に無視にしている。1.3.1)。`PosixSignalRegistration` で `SIGPIPE` を扱わない。子プロセスの `SIGPIPE` の扱いは 07 章に従う。
7. 標準出力・標準エラーのバッファリングに頼らない(`Flush()` を呼ばずに終了しない)。

### 1.4 終了前に出力を完了させる規定

プロセスの終了の手順(`Program.Main` が行う。10 章 4.1・6 も参照):

1. `EntryPoint.Run(args, deps)` が返した終了コード(`int`)を受け取る。
2. 標準出力、続いて標準エラーを、`Flush()` する。`Flush()` の失敗(1.3.2 の 1 の例外)を捕捉して無視する。作れていない出力口と、書き込みに失敗した出力口は飛ばす。
3. そのあとで、**終了コードを 0〜255 の `int` にそろえて**(10 章 4)、`Main` から `return` する(通常の終了の経路。`Environment.Exit` を使わない)。
4. `Environment.Exit(code)` を使ってよいのは、想定外の内部失敗(10 章 6)の経路で、標準エラーに 1 行を書き、`Flush()` を試みたあとの最後の手段としてだけ(`Exit` は、`finally` を実行せず、ほかのスレッドが動いていても直ちに終了する。1.3.1)。通常の終了では使わない。
5. シグナルの登録(`PosixSignalRegistration`、`Console.CancelKeyPress`)は、終了する前に解除する(07 章)。

## 2. `-v`(`--verbose`)

- `run`・`pipeline`・`validate` だけが受け付ける。1 回だけ(2 回は 68)。
- 常に表示するものは、`-v` に関係なく出る(エラー、起動失敗、成功コードに含まれない 0、リトライの知らせ)。
- `-v` の行は、先頭が `[conductor] `(角かっこ、`conductor`、角かっこ閉じ、半角空白)。

### 2.1 `-v` の行の形式

| 種類 | 形式(英語) | 形式(日本語) |
|------|-------------|----------------|
| ジョブの結果 | `[conductor] job=<ID> result=<success\|warning\|error\|interrupted> (<補足>)` | 同左(補足だけ日本語) |
| スキップ | `[conductor] job=<ID> skipped (<理由>)` | `[conductor] job=<ID> スキップ (<理由>)` |
| 遷移 | `[conductor] transition: <success\|warning> -> <遷移先ID または @end>`。スキップしたジョブがあれば、続けて ` (skipped: <ID>, <ID>)` | 同じ形で、` (スキップ: <ID>, <ID>)` |
| 中断 | `[conductor] interrupted by signal <N>` | `[conductor] シグナルで中断 <N>` |
| `validate` 成功 | `[conductor] definition is valid (<N> job)` / `(<N> jobs)`(`N == 1` のとき `job`、それ以外は `jobs`) | `[conductor] 定義は正しいです(ジョブ <N> 個)` |

- `job=`・`result=` と、`success`・`warning`・`error`・`interrupted` の状態語、`transition:`・`->`、遷移先の `@end` は、言語によらず英語(固定の語)。
- 遷移行の ` (skipped: ...)` の ID は、`, `(コンマ+半角空白)で区切る。ID は、名前用エスケープ(9 章)を通す。

結果行の補足 `(<補足>)`:

| 状況 | 英語 | 日本語 |
|------|------|--------|
| ジョブの終了コード | `exit code <C>` | `終了コード <C>` |
| `result=error` で、ジョブの終了コード `C` と `conductor` の終了コード `E` が異なるとき(例: `C = 0` で成功コードに含まれない、`C = 256`) | `exit code <C>, conductor exits with <E>` | `終了コード <C>, conductor の終了コード <E>` |
| ジョブがシグナルで終了 | `terminated by signal <N>` | `シグナルで終了 <N>` |
| 起動できなかった | `could not start` | `起動できませんでした` |
| 中断(ジョブは起動済み) | `interrupted by signal <N>` | `シグナルで中断 <N>` |
| 中断(起動前) | `interrupted by signal <N>, not started` | `シグナルで中断 <N>, 起動せず` |

- 区切りは、言語によらず ASCII の `, `(コンマ+半角空白)。日本語でも全角の読点は使わない。起動前の中断の `, not started` / `, 起動せず` も同じ。
- 補足が出る場面の決め方は、`result` が `error` のとき、`C` が conductor の終了コード `E` と同じなら 1 つ目の形、異なれば 2 つ目の形。

`result` の決め方: ジョブの分類が成功 → `success`、警告 → `warning`、エラー → `error`、分類なし(起動前の中断)→ `interrupted`。起動済みで中断された場合は、分類がエラーなので `error` として表示する。

スキップの理由(`-v` のスキップ行)と、拒否の理由は、別の表(5 章)。

実際の出力(`LANG=C`):

```text
$ conductor pipeline -v nightly --config c.json --run-id r1
[conductor] job=extract result=warning (exit code 1)
[conductor] transition: warning -> load (skipped: transform)
[conductor] job=transform skipped (skipped by a transition)
[conductor] job=load result=success (exit code 0)

$ conductor run -v a --config e.json        # success_exit_codes: [5] で、ジョブが 0 で終了
[conductor] job=a result=error (exit code 0, conductor exits with 1)
conductor: error: job 'a': exit code 0 is not listed in success_exit_codes

$ conductor pipeline -v p --config d.json   # ジョブが SIGKILL で終了
[conductor] job=a result=success (exit code 0)
[conductor] job=b result=error (terminated by signal 9)

$ conductor validate -v
[conductor] definition is valid (3 jobs)
```

出す順序: ジョブの結果行・遷移行・中断行は、ジョブの実行後(ステートの書き込みの前)に出す。起動の失敗と、成功コードに含まれない 0 は、結果行のあとに、常に出す誤り行。

## 3. 誤りの行の形式

```text
conductor: error: <場所>: <種類の文>[: <原因>]
```

- 先頭 `conductor: error: ` は言語によらず同じ。警告は `conductor: warning: `、リトライの知らせは `conductor: retry: `。
- `<場所>`: 定義の誤りは `<ファイル>` または `<ファイル>:<行>`、`command` の誤りは `<定義ファイル>: job '<ID>'`、ステート・食い違いは `<ステートのパス>`、拒否・起動失敗・成功コードに含まれない 0 は `job '<ID>'`、変換の誤りは `<ファイル>` または `<ファイル>:<行>`。
- 使い方の誤りだけは場所を持たず、`conductor: error: <文>[: '<対象>']` のあとに空行と使い方全文(8 章)を続ける。

## 4. メッセージカタログ

「英語」「日本語」の列は、原文のとおり出す。`{…}`・`<…>` は実行時に入る値(エスケープ後。9 章)。

### 4.1 固定の英語の文(ロケールを判定しない)

| キー | 文 | 終了コード |
|------|----|------------|
| `InternalError`(想定外の内部失敗) | `conductor: error: internal error` | 70 |
| `SignalSetupFailed` | `conductor: error: failed to set up signal handling` | 70 |
| `--version` | `conductor {版}`(`conductor 0.8.0`) | 0 |

### 4.2 使い方の誤り

| 種類 | 英語 | 日本語 |
|------|------|--------|
| `UnknownCommand` | `unknown command` | `知らないコマンドです` |
| `MissingArgument` | `missing argument` | `引数が足りません` |
| `UnexpectedArgument` | `unexpected argument` | `余分な引数があります` |
| `UnknownOption` | `unknown option` | `知らないオプションです` |
| `OptionNotAllowed` | `option not allowed for this command` | `このコマンドでは使えないオプションです` |
| `MissingValue` | `option requires a value` | `オプションに値がありません` |
| `InvalidValue` | `invalid value` | `値が正しくありません` |
| `UnknownJob` | `job is not defined in the definition file` | `定義にないジョブIDです` |

表示: 対象あり `conductor: error: <文>: '<対象>'`、なし `conductor: error: <文>`。対象は名前用エスケープ(64 文字で切る)。終了コードはすべて 68。

### 4.3 定義の誤り

表示は `conductor: error: <ファイル>[:<行>]: <種類の文>: <原因>`。`<原因>` は、定義の検証が作る日本語の固定の文で、言語を切り替えない(英語ロケールでも日本語のまま)。原因の文の一覧は [03-definition-format.md](03-definition-format.md) にある。この章では、種類の文だけを規定する。種類は 31 種。終了コードはすべて 64。

| 識別子 | 英語 | 日本語 |
|--------|------|--------|
| `unsupportedFormat` | `unsupported definition file format` | `定義ファイルの形式に対応していません` |
| `fileMissing` | `definition file does not exist` | `定義ファイルが存在しません` |
| `unreadable` | `cannot read definition file` | `定義ファイルを読めません` |
| `tooLarge` | `definition file is too large` | `定義ファイルが大きすぎます` |
| `syntax` | `syntax error` | `文法の誤りです` |
| `typeMismatch` | `wrong value type` | `値の型が正しくありません` |
| `unknownKey` | `unknown key` | `定義にないキーです` |
| `versionMissing` | `format_version is missing` | `書式の版がありません` |
| `versionUnsupported` | `unsupported format_version` | `対応していない書式の版です` |
| `unknownTarget` | `transition target job is not defined` | `遷移先のジョブが定義にありません` |
| `errorTransition` | `no transition can be written for the error result` | `エラー結果には遷移先を書けません` |
| `backwardOrSelfTarget` | `transition target must be a later job or @end` | `遷移先は、記述順で後ろのジョブまたは @end のみです` |
| `overlappingExitCodes` | `success and warning exit codes overlap` | `成功と警告の終了コードが重なっています` |
| `zeroAsWarning` | `0 cannot be a warning exit code` | `0 は警告の終了コードにできません` |
| `pipelineNameMismatch` | `pipeline name does not match the definition` | `パイプライン名が定義と一致しません` |
| `duplicateJobId` | `duplicate job id` | `ジョブIDが重複しています` |
| `invalidJobId` | `invalid job id` | `ジョブIDが正しくありません` |
| `noJobs` | `no jobs are defined` | `ジョブが 1 つもありません` |
| `emptyField` | `required value is empty` | `必須の値が空です` |
| `duplicateTransition` | `duplicate transition for the same result` | `同じ結果の遷移が重複しています` |
| `invalidEnvName` | `invalid environment variable name` | `環境変数の名前が正しくありません` |
| `retryNotTable` | `retry must be a map` | `retry はマップで書きます` |
| `retryMissingKey` | `a required key of retry is missing` | `retry の必須のキーがありません` |
| `retryExitCodesEmpty` | `retry exit_codes is empty` | `retry の終了コードの一覧が空です` |
| `retryExitCodeInvalid` | `retry exit_codes has an invalid exit code` | `retry の終了コードが正しくありません` |
| `retryExitCodesOverlap` | `retry exit_codes overlap the success or warning exit codes` | `retry の終了コードが、成功または警告の終了コードと重なっています` |
| `retryMaxInvalid` | `retry max is invalid` | `retry の最大回数が正しくありません` |
| `retryDurationInvalid` | `retry interval is invalid` | `retry の間隔が正しくありません` |
| `retryBackoffIncomplete` | `retry multiplier and max_interval must be given together or not at all` | `retry の倍率と間隔の上限は、両方を書くか、両方とも書きません` |
| `retryMultiplierInvalid` | `retry multiplier is invalid` | `retry の倍率が正しくありません` |
| `retryMaxIntervalTooSmall` | `retry max_interval is smaller than interval` | `retry の間隔の上限が、間隔より小さくなっています` |

(C# の列挙型のメンバー名は、識別子の先頭を大文字にした形。例: `UnsupportedFormat`。10 章 8.2)

実出力(原因の文の例。`syntax` 以外の原因は 03 章の規定):

```text
$ conductor pipeline wrongname --config c.json        # LANG=C
conductor: error: c.json:3: pipeline name does not match the definition: 指定されたパイプライン名が、定義の `name` と一致しません
$ LANG=ja_JP.UTF-8 conductor validate --config nonexist.json
conductor: error: nonexist.json: 定義ファイルが存在しません: 指定の場所にファイルがありません
```

定義に複数の誤りがあるとき、検出した誤りを 1 行ずつ出す(順序と検出の範囲は [03-definition-format.md](03-definition-format.md))。

### 4.4 `command` の誤り

表示: `conductor: error: <定義ファイル>: job '<ID>': <文>`(原因は付けない)。全ジョブを検査し、誤りのあるジョブごとに 1 行。終了コード 64。

| 種類 | 英語 | 日本語 |
|------|------|--------|
| `unclosedQuote` | `command has an unclosed quote` | `command の引用符が閉じていません` |
| `empty` | `command is empty` | `command が空です` |

`command` の解釈の規則は [03-definition-format.md](03-definition-format.md) と [05-execution-flow.md](05-execution-flow.md) にある。

### 4.5 ステートの誤り

表示: `conductor: error: <ステートのパス>: <種類の文>: <原因>[<補足>]`。`<原因>` は、ステートの読み書きが作る日本語の固定の文で、言語を切り替えない(一覧は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md) 7.2)。終了コードはすべて 65。

| 種類 | 英語 | 日本語 |
|------|------|--------|
| `unreadable` | `cannot read state file` | `ステートファイルを読めません` |
| `tooLarge` | `state file is too large` | `ステートファイルが大きすぎます` |
| `corrupt` | `state file is corrupt` | `ステートファイルが壊れています` |
| `versionMismatch` | `state file version is not supported` | `ステートファイルの版が合いません` |
| `writeFailed` | `cannot write state file` | `ステートファイルを書き込めません` |
| `removeFailed` | `cannot remove state file` | `ステートファイルを削除できません` |

`<補足>`(文脈による):

| 文脈 | 補足の付き方 | 英語 | 日本語 |
|------|----------------|------|--------|
| `Read`(run ID なしで読んだ) | 種類が `unreadable`/`tooLarge`/`corrupt`/`versionMismatch` のときだけ `; <案内>` | ``run `conductor reset` to discard the state`` | `` `conductor reset` でステートを初期化できます `` |
| `ReadRunId`(run ID つきで読んだ) | 同上 | ``run `conductor reset --run-id <ID>` to discard the state``(`<ID>` は、文字どおり `<ID>` という文字列) | `` `conductor reset --run-id <ID>` でステートを初期化できます `` |
| `JobNotStarted` | 常に ` (<文>)` | `the job was not started` | `ジョブは起動していません` |
| `JobExecuted` | 常に ` (<文>)` | `the job has already been executed` | `ジョブは実行されました` |
| `Plain`(`reset`・`watch` の一部) | なし | | |

文脈の使い分け:

- `watch`: run ID なしなら `Read`、ありなら `ReadRunId`(案内つき)。
- `reset`・`reset --all`: `Plain`(補足なし)。
- `run`・`pipeline`: 読み込みは `Read`/`ReadRunId`。書き込み・削除は、ジョブを起動する前の失敗なら `JobNotStarted`、ジョブの実行後の失敗なら `JobExecuted`。

実出力:

```text
conductor: error: c.json.state: state file is corrupt: JSON として読めません; run `conductor reset` to discard the state
conductor: error: c.json.state: ステートファイルが壊れています: JSON として読めません; `conductor reset` でステートを初期化できます
```

### 4.6 定義との食い違い

表示: `conductor: error: <ステートのパス>: <MISMATCH>: <食い違いの種類>; <案内>`。案内は run ID の有無で、4.5 の `Read` / `ReadRunId` の案内と同じ文。終了コード 65。

| 種類 | 英語 | 日本語 |
|------|------|--------|
| `MISMATCH` | `state does not match the definition` | `ステートが定義と食い違っています` |
| `pipelineName` | `pipeline name` | `パイプライン名` |
| `jobIds` | `job ids` | `ジョブID` |
| `jobOrder` | `job order` | `ジョブの並び` |

### 4.7 拒否・起動失敗・そのほか

| キー | 表示 | 英語 | 日本語 | 終了コード |
|------|------|------|--------|------------|
| `Rejected` | `conductor: error: job '<ID>': <Rejected>: <理由>` | `cannot run this job now` | `このジョブは、いま実行できません` | 66 |
| 拒否の理由 `OutOfOrder` | | `earlier jobs have not finished (out of order)` | `前のジョブが終わっていません(順序外)` | |
| 拒否の理由 `SkipTarget` | | `skip target` | `スキップの対象です` | |
| 拒否の理由 `AfterEnd` | | `after @end` | `@end より後ろです` | |
| 拒否の理由 `AlreadyExecuted` | | `already executed` | `実行済みです` | |
| `CannotStart` | `conductor: error: job '<ID>': <文> '<コマンド名>'` | `cannot start command` | `コマンドを起動できません` | 127 |
| `SuccessCodeNotListed` | `conductor: error: job '<ID>': <文>` | `exit code 0 is not listed in success_exit_codes` | `終了コード 0 は、success_exit_codes に含まれていません` | 1 |
| `OutputFailed` | `conductor: error: <文>`(場所なし) | `cannot write to standard output` | `標準出力に書き込めません` | 65(`watch`)/ 67(`convert`) |
| `CompletionUndetermined` | `conductor: error: <ステートのパス>: <文>` | `cannot determine whether the pipeline is complete` | `パイプラインが完了したかを判定できません` | 65 |

`CannotStart` の `<コマンド名>` は、`command` を解釈して得た実行ファイル名(解釈できなければ `command` の先頭の語。それもなければ空)を、名前用エスケープしたもの。

実出力:

```text
conductor: error: job 'load': cannot run this job now: earlier jobs have not finished (out of order)
conductor: error: job 'step3': cannot start command 'nonexistent_prog_x'
```

### 4.8 変換の誤り

表示: `conductor: error: <ファイル>[:<行>]: <種類の文>: <原因>[ (<AtKey>: <キーの位置>)]`。`<原因>` は、変換が作る日本語の固定の文で、言語を切り替えない(一覧は 10 章 7.3)。`AtKey` は英語 `at`、日本語 `場所`。キーの位置があるときだけ付く(` (at: job[2].transitions[0])`)。終了コードはすべて 67。

| 種類 | 英語 | 日本語 |
|------|------|--------|
| `fileMissing` | `input file does not exist` | `入力ファイルが存在しません` |
| `unreadable` | `cannot read input file` | `入力ファイルを読めません` |
| `tooLarge` | `input is too large` | `入力が大きすぎます` |
| `syntax` | `syntax error` | `文法の誤りです` |
| `unrepresentable` | `value cannot be represented in the target format` | `相手の形式で表せない値があります` |
| `rootNotMap` | `top level is not a map` | `最上位がキーと値の集まり(マップ)ではありません` |

実出力:

```text
$ conductor convert nofile --to json
conductor: error: nofile: input file does not exist: 指定の場所にファイルがありません
```

### 4.9 リトライの知らせ(`Notice`。常に標準エラー)

形式: `conductor: retry: <本文>`。

| 言語 | 本文 |
|------|------|
| 英語 | `job '{ID}' exited with code {最後の終了コード}; retry {n} of {max} in {秒} seconds` |
| 日本語 | `ジョブ '{ID}' が終了コード {最後の終了コード} で終わりました。{秒} 秒後にリトライします({n}/{max} 回目)` |

- `{n}` は、1 から始まる「何回目のリトライか」。`{max}` は最大回数。`{秒}` は、次の待ち時間(整数の秒。リトライの待ち時間は整数秒で扱う)。1 秒でも `seconds`(単複の区別はしない)。
- 待機の前に必ず出す(1.3.2 の 3 のとおり、`Flush()` を終えてから待機に入る)。`{最後の終了コード}` は、直前の試行の終了コード。終了コードがなければ 0 と表示する。
- 待ち時間の求め方は [05-execution-flow.md](05-execution-flow.md)(リトライ)に従う。`{秒}` と `{n}`・`{max}` は、整数を 10 進の整数で文字列にしたものを使う。**文化に依存しない書き方で作る**: `n.ToString(CultureInfo.InvariantCulture)`、または `string.Create(CultureInfo.InvariantCulture, $"...")`(文化に依存する `ToString()`・既定の補間は、桁区切りや符号の表記が環境で変わりうる。.NET の既知の注意で、この環境では出典を取得しておらず、未確認。`InvariantGlobalization=true` でも、書き方を固定する)。待ち時間は 64 ビット符号なし整数の上限 `ulong.MaxValue`(18446744073709551615。20 桁)で飽和させる([05-execution-flow.md](05-execution-flow.md))ので、`ulong` をそのまま 10 進の文字列にする(`double`・`decimal` を経由しない。`double` の `ToString()` は、大きな値で `E+19` のような指数表記になりうる)。

実出力:

```text
conductor: retry: job 'load' exited with code 75; retry 1 of 2 in 1 seconds
conductor: retry: ジョブ 'load' が終了コード 75 で終わりました。1 秒後にリトライします(1/2 回目)
```

### 4.10 `reset --all` の警告

形式(標準エラー。終了コードは 0):

```text
conductor: warning: <RESET_ALL_KEPT>
  <ファイル名>
  <ファイル名>
```

| 言語 | `RESET_ALL_KEPT` |
|------|------------------|
| 英語 | ``kept state files that could not be read (remove one with `conductor reset --run-id <ID>`):`` |
| 日本語 | ``読めなかったため、次のステートファイルは残しました(`conductor reset --run-id <ID>` で個別に消せます):`` |

ファイル名の行は、先頭に半角空白 2 文字を置き、パス用エスケープ(切らない)を通したファイル名(ディレクトリを含まない名前)を書く。1 行 1 ファイル。

### 4.11 `-v` の補助語

| キー | 英語 | 日本語 |
|------|------|--------|
| `ExitCode` | `exit code` | `終了コード` |
| `ConductorExitsWith` | `conductor exits with` | `conductor の終了コード` |
| `KilledBySignal` | `terminated by signal` | `シグナルで終了` |
| `CouldNotStart` | `could not start` | `起動できませんでした` |
| `InterruptedBySignal` | `interrupted by signal` | `シグナルで中断` |
| `NotStarted` | `not started` | `起動せず` |
| `SkippedWord`(スキップ行の語) | `skipped` | `スキップ` |
| `SkippedList`(遷移行の `(…: ID, …)`) | `skipped` | `スキップ` |

組み立ては、`<語> <数>`(半角空白 1 つで連結)。日本語でも数との連結は半角空白。

## 5. スキップ・拒否の理由の対応

| 理由 | `-v` のスキップ行 `(…)` 英語 | 日本語 | 拒否の理由(英語 / 日本語) |
|------|--------------------------------|--------|------------|
| `SkipTarget` | `skipped by a transition` | `遷移によりスキップ` | `skip target` / `スキップの対象です` |
| `AfterEnd` | `skipped after @end` | `@end の後ろのためスキップ` | `after @end` / `@end より後ろです` |
| `AlreadyExecuted` | `already executed` | `実行済み` | `already executed` / `実行済みです` |
| `OutOfOrder` | `out of order` | `順序外` | `earlier jobs have not finished (out of order)` / `前のジョブが終わっていません(順序外)` |

起動前の判断(実行・スキップ・拒否)の決め方は [05-execution-flow.md](05-execution-flow.md)。

## 6. `watch` の表

### 6.1 構成

出力(標準出力):

```text
<Pipeline|パイプライン>: <パイプライン名>
Run ID: <ID>                       ← run ID があるときだけ。見出しは言語によらず "Run ID"
<ジョブ見出し>  <状態見出し>  <補足見出し>
<ジョブID>  <状態>  <補足>
...
<完了|completed>                  ← ステートが完了のときだけ
```

| 要素 | 英語 | 日本語 |
|------|------|--------|
| パイプライン行の見出し | `Pipeline` | `パイプライン` |
| run ID 行の見出し | `Run ID` | `Run ID` |
| 列見出し(ジョブ) | `JOB` | `ジョブ` |
| 列見出し(状態) | `STATUS` | `状態` |
| 列見出し(補足) | `NOTE` | `補足` |
| 完了を示す行 | `completed` | `完了` |
| ステートなし(ファイルなし・長さ 0) | `no state (nothing has run yet)` | `ステートはありません(まだ実行されていません)` |

- パイプライン名・run ID・ジョブIDは、名前用エスケープ(9 章。64 文字で切り、制御文字を `\u{..}` に)を通す。
- パイプライン行は `<見出し>: <名前>`(コロンのあとに半角空白 1 つ)、run ID 行は `Run ID: <ID>`。
- ステートなしの文は、表の代わりにこの 1 行だけ(末尾改行つき)を標準出力に出す。
- 行はすべて LF で終わる。

### 6.2 列と整形

状態の語(言語によらず英語): `pending`, `success`, `warning`, `error`, `skipped`。

補足の決め方:

| 状態 | 補足 |
|------|------|
| `success` / `warning` / `error` | 終了コードがあれば `exit code <C>`(日本語 `終了コード <C>`)。なければ空 |
| `skipped` | `skip_reason` が `ByEnd` → `skipped after @end` / `@end の後ろのためスキップ`。`ByTransition` → `skipped by a transition` / `遷移によりスキップ`。なければ空 |
| `pending` | 空 |

(`skip_reason` の保存形式での値は [06-state-file.md](06-state-file.md) による。ここでは、内部の種別 `ByEnd` / `ByTransition` を指す。)

整形の規則:

- 列は、2 つの半角空白で区切る。
- ジョブの列の幅 = 見出しと、全ジョブID(エスケープ後)の表示幅の最大。状態の列の幅 = 見出しと、全状態語(5 語の固定集合ではなく、実際に現れる語と見出し)の表示幅の最大。
- **表示幅**は、文字(コードポイント)ごとに、ASCII(U+0000〜U+007F)なら 1、それ以外は 2 として足す近似(全角を 2 とみなす)。実際の端末幅(結合文字、絵文字、ゼロ幅の文字など)は考慮しない。コードポイントの走査は、4.0 のサロゲートの対を 1 文字にする自前の走査(04 章 4.0)と同じにする(`string.Length` はコードユニット数なので、基本多言語面の外の文字を 2 と数えるが、この規定では 1 コードポイントなので、表示幅は 2)。**`string.Length` を表示幅に使わず、`PadRight`・`PadLeft`、複合書式の幅指定(`{0,-10}`)、`StringInfo`(書記素)、`Rune.GetUnicodeCategory` などの East Asian Width 相当の判定も使わない**(結果が、この近似と異なる)。
- 次の表の入力で、表示幅が決まる(この近似による。「実際の端末幅」ではない)。

  | 文字列 | `Length`(コードユニット数) | コードポイント数 | 表示幅(この規定) | 備考 |
  |--------|-------------------------------|---------------------|-----------------------|------|
  | `step1` | 5 | 5 | 5 | ASCII |
  | `ジョブ` | 3 | 3 | 6 | 全角 |
  | `😀`(U+1F600) | 2 | 1 | 2 | サロゲートの対で 1 コードポイント |
  | `e` + U+0301(結合文字) | 2 | 2 | 3 | 結合文字を 2 と数える(端末の見た目の幅は 1) |
  | `🇯🇵`(U+1F1EF U+1F1F5) | 4 | 2 | 4 | 国旗は 2 コードポイント(端末の見た目の幅は 2) |
  | `日本語 abc` | 7 | 7 | 10 | |

- 見出し行: ジョブ見出しと状態見出しを、右に半角空白を足して列の幅にそろえ、補足見出しは埋めない。行末の空白は取らない(補足見出しは空でないので、実際には取る空白がない)。
- データ行: ジョブIDを埋め(列の表示幅にそろえる)、状態語を埋め(埋めは状態語の表示幅で数える。状態語は ASCII なので文字数と同じ)、補足を続ける。行全体の行末の空白を削る(補足が空なら、状態語のあとの埋めと区切りも削られる)。削る空白は ASCII の半角空白だけ(`line.TrimEnd(' ')`。引数なしの `string.TrimEnd()` は Unicode の空白(全角空白 U+3000、タブ、改行など)も削るので使わない。`Trim` 系の引数なしの形が、Unicode の空白を削る挙動は、この環境では出典を取得しておらず、未確認)。
- **埋めは、自前で `new string(' ', n)` を足して行う**(`n = 列の幅 - その文字列の表示幅`。`n` が負にならないことは、列の幅が最大であることで保証される)。`PadRight`・`PadLeft` は、コードユニット数で数え、全角文字を 1 と数えるので、表示幅の規則と合わない(例: `"あ".PadRight(4)` はコードユニット 4 になるが、表示幅の規定では `あ` は 2 なので、列の幅 4 にそろえるなら足す空白は 2)。
- `completed`: ステートの `completed` が真のときだけ、表の下に 1 行足す(`completed` / `完了`)。
- 表の全体を 1 つの文字列(各行を `StringBuilder` に `Append(line).Append('\n')` で連結する。`AppendLine`・`string.Join(Environment.NewLine, ...)` は使わない)に作ってから、`IOutputSink.WriteAll`(1.3)で 1 回で書く(書き込みの失敗を 1 回で検出するため)。

### 6.3 色

- 色を使う条件(最初に必要になったとき一度だけ判定): 環境変数 `NO_COLOR` が未設定または空(値が `IsSuspectText` の場合も未設定と同じ。04 章 4.4.2)、かつ標準出力が端末(`Console.IsOutputRedirected` が偽。確認: 1.2.1)、かつ実行環境が Windows でない。Windows は常に色なし(挙動をほかの OS の実装と揃えるための決定。Windows の端末の ANSI 対応の可否は、この規定のために確認していない)。OS の判定は、`Conductor.Runtime` の 1 か所(`PlatformKind Platform.Detect()`。内部で `OperatingSystem.IsWindows()` を使ってよい。07 章)にまとめ、`Conductor.Report` は `RuntimeInformation`・`OperatingSystem` を直接参照しない(OS 差を 1 か所にまとめる規則)。`CliEnvironment.IsStdoutTerminal()` は、OS を見ずに、`!Console.IsOutputRedirected` だけを返す。
- 端末の判定: `Console.IsOutputRedirected` は、標準出力が端末でないとき(パイプ・ファイル)真になる(確認: 1.2.1。Unix では `isatty(1)` の否定。Windows の実装は出典を取得しておらず、未確認)。端末(疑似端末を含む)で偽になることは、この環境では確認できなかった(未確認。実装時に、実際の端末で確かめる)。`Console.IsOutputRedirected` に統一し、`Console.IsInputRedirected` などを混ぜない。外部のライブラリは使わない。
- 色は、状態語だけに、ANSI の SGR で付ける: `ESC[<番号>m<語>ESC[0m`(ESC は U+001B。C# の文字列では `"\u001b"`。`"\e"` は言語の版に依存するので使わない)のあとに埋めの空白。`success` = 32(緑)、`warning` = 33(黄)、`error` = 31(赤)、`skipped` = 36(シアン)、`pending` = 色なし(制御列を付けない)。
- 列の幅は、色の制御文字を除いた文字で決める(状態語の埋めの空白は、制御列の外側に置く)。ファイルやパイプへの出力には制御文字を含めない。状態は色によらず文字でも表す。

実出力(`LANG=C`):

```text
$ conductor watch
Pipeline: hello
JOB    STATUS   NOTE
step1  success  exit code 0
step2  success  exit code 0
step3  error
```

run ID つき・完了(英語 / 日本語):

```text
Pipeline: nightly
Run ID: r1
JOB        STATUS   NOTE
extract    warning  exit code 1
transform  skipped  skipped by a transition
load       success  exit code 0
completed
```

```text
パイプライン: nightly
Run ID: r1
ジョブ     状態     補足
extract    warning  終了コード 1
transform  skipped  遷移によりスキップ
load       success  終了コード 0
完了
```

(日本語の表で `ジョブ` は表示幅 6、列の幅は `transform` の 9 なので、埋めは 3 空白、区切りの 2 空白と合わせて 5 空白になる。`状態` は幅 4、列の幅 7 で、同様に 5 空白。)

## 7. ロケールの決め方

- 環境変数 `LC_ALL`, `LC_MESSAGES`, `LANG` を、この順に見る(`CliEnvironment` の `GetEnvironmentVariable`)。最初に空でない値を採り、その値が `ja` で始まれば(`value.StartsWith("ja", StringComparison.Ordinal)`)日本語、それ以外は英語。どれも未設定(`null`)または空なら英語。値は大文字小文字を区別する(`JA_JP` は英語)。値が `IsSuspectText`(04 章 4.4.2)のときは、その変数は未設定と同じ扱い(その変数は飛ばして、次の変数を見る)。
- 最初の空でない値で決まり、後の変数は見ない(`LC_ALL=C` かつ `LANG=ja_JP` は英語。`LC_ALL=` が空で `LANG=ja_JP.UTF-8` なら日本語)。
- 判定は、1 回の実行につき、最初に表示が必要になったときだけ行い、結果を保つ(単純な「未判定」の状態を持つ)。
- ロケールを判定しない固定の文: `--version`、内部の誤りの文、シグナルの準備の失敗の文(英語のみ)。
- 言語によらず英語: 状態語、`job=`・`result=`、`Run ID`、`conductor: error: `・`conductor: warning: `・`conductor: retry: `・`[conductor] ` の接頭辞、`--help` の中のオプション名。定義・ステート・変換の誤りの `<原因>` は日本語のまま。
- 必須: ロケールの判定に、`CultureInfo.CurrentCulture`・`CultureInfo.CurrentUICulture`・`CultureInfo.InstalledUICulture`・`Thread.CurrentThread.CurrentCulture`・`RegionInfo`・リソース(`ResourceManager`、`.resx`)などの .NET の文化の API を**使わず**、上の 3 つの環境変数だけで決めること。理由: `InvariantGlobalization=true`(02 章)の下では、文化の値が環境のロケールを反映しない(インバリアントになる)うえ、環境変数の `LANG` などの扱いが、文化の API を通すと、OS・ランタイムの設定の経路(ICU の有無など)で変わりうるため。文字列の整形・比較にも、`CultureInfo.InvariantCulture` と序数(`StringComparison.Ordinal`)を明示する(04 章 4.0、4.9)。

## 8. 使い方(`--help`)全文

標準出力、終了コード 0。使い方の誤りのときは、誤りの行と空行のあとに、同じ全文を標準エラーに出す。言語は 7 章で決める。各全文の末尾は改行で終わる。行の先頭の空白は、半角空白 2 文字(オプションの説明の折り返しの行は 22 文字)。

### 8.1 英語

```text
Usage:
  conductor run [-v] [--config <path>] [--run-id <id>] [--remove-on-complete] <job-id>
  conductor pipeline [-v] [--config <path>] [--run-id <id>] [--remove-on-complete] <pipeline-name>
  conductor watch [--config <path>] [--run-id <id>]
  conductor reset [--config <path>] [--run-id <id> | --all]
  conductor validate [-v] [--config <path>] [<pipeline-name>]
  conductor convert <file> --to <json|yaml>
  conductor --help | --version

Options:
  -v, --verbose       print job results, skips and transitions to standard error (run, pipeline; validate prints one line)
  --config <path>     definition file (default: conductor.json in the current directory)
  --run-id <id>       run ID that separates the state file (run, pipeline, watch, reset; default: the environment variable
                      named by run_id_env in the definition, or CONDUCTOR_RUN_ID; an empty value means no run ID)
  --remove-on-complete
                      remove the state file when the run completes (run, pipeline; default: keep it, or the
                      remove_on_complete setting of the definition)
  --all               remove every state file of the definition, with or without a run ID (reset only)
  --to <json|yaml>    target format of convert
  -h, --help          print this help
  -V, --version       print the version
```

### 8.2 日本語

```text
使い方:
  conductor run [-v] [--config <パス>] [--run-id <ID>] [--remove-on-complete] <ジョブID>
  conductor pipeline [-v] [--config <パス>] [--run-id <ID>] [--remove-on-complete] <パイプライン名>
  conductor watch [--config <パス>] [--run-id <ID>]
  conductor reset [--config <パス>] [--run-id <ID> | --all]
  conductor validate [-v] [--config <パス>] [<パイプライン名>]
  conductor convert <ファイル> --to <json|yaml>
  conductor --help | --version

オプション:
  -v, --verbose       ジョブの結果、スキップ、遷移を標準エラーに表示する(run、pipeline。validate は 1 行)
  --config <パス>     定義ファイル(省略時: 実行ディレクトリの conductor.json)
  --run-id <ID>       ステートファイルを分ける run ID(run、pipeline、watch、reset。省略時: 定義の run_id_env の環境変数、
                      なければ CONDUCTOR_RUN_ID。空の値は run ID なし)
  --remove-on-complete
                      完了のときにステートファイルを消す(run、pipeline。省略時: 残す。定義の remove_on_complete に従う)
  --all               定義のステートファイルを、run ID の有無によらず、すべて消す(reset のみ)
  --to <json|yaml>    convert の変換後の形式
  -h, --help          この使い方を表示する
  -V, --version       版を表示する
```

### 8.3 この版で文言を直した箇所(一覧)

元の `conductor` の文言から、次の 3 点(英語・日本語とも。計 6 行)だけを、定義の主書式が JSON になったことに合わせて直した。それ以外の行は、文字単位で同じ。実装は「直した後」の文言だけを出す。

| 行 | 直した後 | 直した内容 |
|----|----------|------------|
| `convert` の使い方の行(英語) | `conductor convert <file> --to <json\|yaml>` | 変換先の書式の組を `json` と `yaml` にした |
| `convert` の使い方の行(日本語) | `conductor convert <ファイル> --to <json\|yaml>` | 同上 |
| `--config` の説明(英語) | `(default: conductor.json in the current directory)` | 既定の定義ファイル名の拡張子を `json` にした |
| `--config` の説明(日本語) | `(省略時: 実行ディレクトリの conductor.json)` | 同上 |
| `--to` の説明(英語) | `--to <json\|yaml>    target format of convert` | 変換先の書式の組を `json` と `yaml` にした |
| `--to` の説明(日本語) | `--to <json\|yaml>    convert の変換後の形式` | 同上 |

(表の `\|` は Markdown の表のための書き方で、実際の文言は `|` 1 文字。)

あわせて、ステートの原因・定義の誤りの種類の文言も、0 章の表のとおり、JSON/YAML に合わせてある(`JSON として読めません`、`内容を JSON にできません`、`retry はマップで書きます`、`定義ファイルの形式に対応していません`)。

### 8.4 全文の保持と検査

- 全文は、C# のソースに、行の文字列の配列(`static readonly string[]`)として持つ(`"""` の生文字列リテラルや `@""` の逐語的文字列で書くと、先頭と末尾の改行、行末の空白、ソースの改行コード(CRLF のチェックアウト)の影響を受けやすいため)。「各行を `string.Join("\n", lines)` で連結し、末尾にも `"\n"` を付ける」実装にして、受け入れテストで文字単位に比較する。
- 使い方の誤りの表示は、誤りの行、空行(`\n` だけの行)、全文、の順。

## 9. 入力に由来する文字列のエスケープ

表示するジョブID・名前・パス・キー名・コマンド名・使い方の誤りの対象は、エスケープを通してから出す(端末の制御列の注入や、表示の偽装を防ぐため)。ジョブ本体の出力は通さない(そのまま)。

規則(文字は Unicode のコードポイント単位で見る。11 章):

| 文字 | 出力 |
|------|------|
| LF(U+000A) | `\n`(バックスラッシュと `n`) |
| CR(U+000D) | `\r` |
| TAB(U+0009) | `\t` |
| 制御文字(Unicode の一般カテゴリ Cc: U+0000〜U+001F、U+007F〜U+009F。上の 3 つを除く) | `\u{<小文字の 16 進、先頭の 0 なし>}`(例: ESC は `\u{1b}`、NUL は `\u{0}`、U+0085 は `\u{85}`) |
| 書式・双方向の制御文字: U+200B〜U+200F、U+2028、U+2029、U+202A〜U+202E、U+2060〜U+2064、U+2066〜U+2069、U+FEFF | `\u{...}`(同じ形式。例: `\u{202e}`) |
| 対になっていないサロゲート(U+D800〜U+DFFF の単独の `char`) | 走査の前に、04 章 4.4.2 の `Readable` で U+FFFD に置き換える(下記)。置き換え後の U+FFFD はそのまま出る |
| それ以外 | そのまま(日本語もそのまま。サロゲートの対で表す文字(絵文字など)もそのまま) |

C# での実装の方針:

- **最初に、文字列全体を 04 章 4.4.2 の `Readable(s)` に通す**。対になっていないサロゲートが U+FFFD になる。これにより、後続の処理と、標準出力・標準エラーの符号化(1.2)で、`EncoderFallbackException`(標準出力は厳密な符号化)が起きない。`Readable` を通さずに、単独のサロゲートの値を `\u{d800}` のように出すと、元のバイトとは無関係な値を見せてしまう。
- 走査は、04 章 4.0 の自前の走査(サロゲートの対を 1 コードポイントにする)で行い、コードポイントの値を `int cp` で得る(対のとき `char.ConvertToUtf32(high, low)`)。16 進は `cp.ToString("x", CultureInfo.InvariantCulture)`(小文字で、先頭の 0 が付かない書式。`0` は `0`)から `\u{` と `}` で囲んで作る。大文字になる `"X"`、桁を固定する `"x4"`・`"x8"` を使わない。
- Cc の判定は、範囲(`cp < 0x20 || (0x7F <= cp && cp <= 0x9F)`)で書く(`char.IsControl`・`Rune.IsControl` は同じ範囲だが、範囲を直接書く)。書式・双方向の文字は、上の表の範囲を列挙する(Unicode のカテゴリ `Cf` 全体ではない。表にない `Cf` の文字はそのまま出す)。
- `string.Normalize`、`Regex.Escape`、`System.Net.WebUtility`・`System.Text.Encodings.Web.JavaScriptEncoder` など、組み込みのエスケープは、規則の形(`\u{..}`、小文字の 16 進、先頭の 0 なし、対象とする範囲)と異なるので使わない。

切り詰め:

- 名前用(`EscapeName`): 上限 64 文字。数えるのは、エスケープ後の文字数(コードポイント単位。エスケープ列(`\u{202e}` など)の各文字を 1 文字ずつ数える。基本多言語面の外の文字は 1 文字)。次の 1 文字(エスケープ列は 1 かたまり)を足すと 64 を超えるなら、そこで打ち切って末尾に `…`(U+2026)を 1 文字付ける。したがって出力は最大 65 文字。エスケープ列は途中で切らない。出力する文字列(`StringBuilder`)に、1 かたまりずつ足していく(エスケープ後の文字列に対して、`Substring(0, 64)` のように位置を指定して切らない。エスケープ列やサロゲートの対を分断するため)。
- パス用(`EscapePath`): 上限なし(エスケープだけ)。
- 名前用を使うもの: 使い方の誤りの対象、ジョブID、遷移先、スキップのID、リトライのジョブID、パイプライン名、run ID、コマンド名。パス用を使うもの: パス(定義ファイル、ステートのパス、`reset --all` のファイル名、変換の入力ファイル、原因、キーの位置)。
- 文字に直せない部分(対になっていないサロゲート、実装が置き換えた U+FFFD)は、先頭の `Readable` が U+FFFD にしてから通す(U+FFFD は、制御文字でも書式文字でもないので、そのまま出る)。

期待値の表(入力 → 上限 → 出力。`<LF>` などは実際の文字):

| 入力 | 上限 | 出力 |
|------|------|------|
| `a<LF>b<ESC>[0m` | 64 | `a\nb\u{1b}[0m` |
| `k<TAB>x` | 64 | `k\tx` |
| `a<CR>b` | 64 | `a\rb` |
| `日本語 abc` | 64 | `日本語 abc` |
| `a<U+200B>b` | 64 | `a\u{200b}b` |
| `a<U+202E>b` | 64 | `a\u{202e}b` |
| `a<U+2066>b` | 64 | `a\u{2066}b` |
| `a<U+FEFF>b` | 64 | `a\u{feff}b` |
| `a<U+2028>b<U+2029>` | 64 | `a\u{2028}b\u{2029}` |
| `<U+0000><U+007F><U+0085>` | 64 | `\u{0}\u{7f}\u{85}` |
| `k` × 64 | 64 | `k` × 64(切らない) |
| `k` × 200 | 64 | `k` × 64 + `…` |
| LF × 100 | 64 | `\n` × 32 + `…`(65 文字) |
| `abcdef` | 3 | `abc…` |
| (空) | 64 | (空) |
| `😀` × 64(U+1F600。`Length` は 128、コードポイントは 64) | 64 | `😀` × 64(切らない) |
| `😀` × 65 | 64 | `😀` × 64 + `…` |
| `a<U+D800>b`(単独の高位サロゲート) | 64 | `a<U+FFFD>b` |
| `a<U+DC00>b`(単独の低位サロゲート) | 64 | `a<U+FFFD>b` |
| `a<U+D83D>`(末尾の高位サロゲート) | 64 | `a<U+FFFD>` |
| `a<U+D83D><U+DE00>b`(対になっている。😀) | 64 | `a😀b`(そのまま) |

定義の検証・ステート・変換の各処理が、誤りの文を内部で作るときにも、同じ規則のエスケープを使う(エスケープだけの小さな部品を、`Conductor.Core` の `Conductor.Text.Escaping`(純粋な静的クラス)に置き、`Conductor.Report` と、定義・ステート・変換の各処理が、同じものを使う。依存の向き(`Cli` → `Runtime` → `Core`)を守るため、エスケープは `Core` に置く)。その場合、定義とパスのファイル名には 256 文字の上限を使う実装がありうるが、`conductor` の表示では、ファイル名をパス用エスケープ(切らない)で出す。したがって、表示されるファイル名は切り詰められない(必須)。

## 10. 実装方針の選定(C#)

| 項目 | 採用 | 理由・補足 |
|------|------|------------|
| メッセージの保持 | ロケールごとの読み取り専用の対応表(`IReadOnlyDictionary<MessageKey, string>`、または `switch` 式を持つ静的メソッド)を `Conductor.Report` に持つ。キーは `enum MessageKey`。全キーがすべてのロケールにあることを、単体テスト(`Enum.GetValues<MessageKey>()` の全メンバーで検査)で確かめる | 表の全件を持てているかを、テストで検出できる。ただし、ロケールごとの語順の違いがあるメッセージは、10.1 の関数で組み立てる |
| プレースホルダの置換 | 固定の文字列に、値を連結する(`string.Concat`、または補間文字列を `string.Create(CultureInfo.InvariantCulture, $"...")` に渡す)。**入力に由来する文字列を、複合書式の書式文字列(`string.Format(format, ...)` の `format`)に渡さない**(値に `{0}` が入っていても、値は書式として解釈されないが、値を連結してから `Format` を呼ぶと解釈される)。値(ジョブIDなど)に `{n}` のような文字列が入っても、再度置換されないこと | 値の中の `{` `}` が、書式として再解釈されないことを、設計で保証する |
| 文言の組み立て | 種類ごとのメソッド(例: `FormatRetry(Locale loc, string jobId, int code, int n, int max, ulong seconds) -> string`)。ロケールによる分岐はメソッドの中 | 日本語・英語で語順が違う(リトライの知らせ)ため、単純な置換の表では足りない |
| 端末判定 | `Console.IsOutputRedirected`(6.3)。`CliEnvironment` の口を通す | 外部ライブラリは、YAML の読み取りのほかには追加しない |
| 色 | 自前の SGR の定数 | 外部の色のライブラリは使わない。`NO_COLOR` や端末判定の規則を、この規定に固定するため |
| 表示幅 | 自前(ASCII なら 1、それ以外は 2。コードポイント単位) | 仕様の近似に正確に合わせるため |
| エスケープ | 自前(9 章) | 仕様の表に正確に合わせるため |
| 書き込み | `IOutputSink`(`Conductor.Report`。標準出力・標準エラーの 2 つ。`WriteAll(string)` で書いて流す。失敗は例外)。本番の実装は `StreamWriter` を包んだもの(1.2、1.3) | テストで差し替えるため(`Deps.Out`) |
| ログ(`Microsoft.Extensions.Logging`、`Console.WriteLine`、`Trace`/`Debug`) | 使わない | 出力の形・出力先・水準が、この章で固定されているため |

### 10.1 骨格(例)

```csharp
namespace Conductor.Report;

internal enum Locale { En, Ja }
internal enum Level { Error, Notice, Verbose, Output }

/// 標準出力への書き込みの失敗(Output 水準)。呼び出し元が 65(watch)/ 67(convert)にする。
internal sealed class OutputException : Exception
{
    public OutputException(Exception inner) : base("failed to write to standard output", inner) { }
}

internal interface IOutputSink
{
    // 失敗は IOException / ObjectDisposedException / EncoderFallbackException / NotSupportedException(1.3.2)。
    // Write のあとに Flush まで行う。
    void WriteAll(string text);
}

internal interface IReporter
{
    void Output(string text);   // 失敗は OutputException を送出
    void Error(string text);    // 失敗は握りつぶす(送出しない)
    void Flush();               // 終了前の完了(1.4)
}
```

`Reporter` は、`Deps.Out` の `IOutputSink`(標準出力・標準エラー)を包む。ロケールと色の判定を、最初に必要になったときに行い、結果を保持する。判定に使う環境変数(`GetEnvironmentVariable`)と端末判定(`IsStdoutTerminal`)は `CliEnvironment` から受け取る。`Error()` は、`Flush()` まで終えてから戻る(1.3.2 の 3)。呼び出し側は、例外の捕捉を要しない(内部で握りつぶす)。

## 11. 文字列の単位(コードポイント)

.NET の `string` は、UTF-16 のコードユニット(`char`)の列である。基本多言語面の外の文字(絵文字など。U+10000 以上)は、2 つの `char`(サロゲートの対)になり、`Length` は 2 を返す。`System.Text.Rune` は、1 つの Unicode スカラー値(対になっていないサロゲートを含まない)を表す型で、`string` は、対になっていないサロゲートを持てる。`conductor` が文字を数える・切る・並べる場面の単位は次のとおり(「コードポイント」は、サロゲートの対を 1 つと数える自前の走査。04 章 4.0)。

| 場面 | 単位 | 使う方法 | 使ってはならない方法 |
|------|------|----------|----------------------|
| 引数の連結の分解(04 章 4.1) | コードポイント | 04 章 4.0 の自前の走査 | `foreach (char c in s)` で 1 つずつ(サロゲートの対を分断する)、書記素の分割(`StringInfo`) |
| エスケープの走査(9 章) | コードポイント | 自前の走査と `int cp`(走査の前に `Readable` で単独のサロゲートを除く) | `Regex.Escape`・`JavaScriptEncoder` など、組み込みのエスケープ |
| 名前用の切り詰め(9 章) | コードポイント(エスケープ後) | 1 かたまりずつ足して数える | エスケープ後の文字列を位置指定で切る(`Substring`。エスケープ列・サロゲートの対を分断する) |
| `watch` の表示幅(6.2) | コードポイントごとに 1 か 2 | `cp <= 0x7F ? 1 : 2` の合計 | `Length`(基本多言語面の外の文字を 2 と数える)、`PadRight`・`PadLeft`、複合書式の幅指定 |
| run ID の長さ(04 章 5.2) | (ASCII だけなので単位による差がない) | 範囲の比較のあと `Length` | — |
| ファイルへ書く(ステート・変換の出力) | UTF-8 のバイト列(BOM なし) | `new UTF8Encoding(false, true)` で符号化した `byte[]` を書く、または、`StreamWriter` に同じエンコーディングを渡す。UTF-8 のバイト数は `Encoding.UTF8.GetByteCount(s)` | `File.WriteAllText(path, text)`(エンコーディングを省略した形の既定は、取得した文書で確認しておらず未確認。BOM の有無が環境で変わりうる)、`Encoding.UTF8`(BOM を出すインスタンスである可能性があり、`StreamWriter` に渡すと BOM が出うる。取得した文書では確認しておらず未確認。1.2.2 のとおり、BOM なしのインスタンスを使う) |

注意:

- 「見た目の 1 文字」(書記素クラスタ。結合文字、絵文字の連結、国旗など)は、コードポイントとも一致しない。`conductor` は、書記素クラスタを単位にしない(表示幅は近似。6.2)。
- 文字列の比較(定義のキー、ジョブID、`--to` の値など)は、コードユニット列の厳密な一致(`string.Equals(a, b, StringComparison.Ordinal)` または `==`)で行う。Unicode の正規化(`string.Normalize`)や、大文字小文字の変換(`ToLower`・`ToUpper`・`ToLowerInvariant`・`ToUpperInvariant`)・大文字小文字を無視する比較(`OrdinalIgnoreCase`。拡張子の比較など、03 章が明示する箇所だけ)は、仕様が求める箇所でだけ行う。`string.Compare(a, b)`・`CompareTo`・`string.StartsWith(string)` の既定の形は、文化に依存する(取得した文書では確認しておらず、未確認)ので使わない。
- `Trim()`・`TrimEnd()`・`Split()`(引数なし)・`char.IsWhiteSpace` は、Unicode の空白(U+3000、U+0085、U+00A0 など)に広く一致する(未確認)。ASCII の半角空白だけを扱う箇所(6.2 の行末の空白の除去など)では、引数を明示する(`TrimEnd(' ')`)。
- 整数の最大値(リトライの待ち時間の飽和)は、`ulong` と `Math.BigMul`(または `UInt128`)で行い、`checked` で例外にせず、飽和させる([05-execution-flow.md](05-execution-flow.md))。折り返さない。表示する数値は、`ulong`・`int` を 10 進の文字列にする(4.9)。

## 12. 必須の挙動と、実装に任せてよい点

必須:

- 4 章・5 章・6 章・8 章の全文(0 章の文言を含む)、`-v` の行の形式、出力先(標準出力か標準エラーか)と水準、行の改行(LF)。
- 言語の決め方(7 章。環境変数だけ)、エスケープの規則と切り詰め(9 章。単独のサロゲートを U+FFFD にする前処理を含む)、色の条件(6.3)、文字の単位と表示幅・埋め(6.2、11 章)。
- 日本語・英語ともに、固定の語とトークンの区別(2.1、7 章)。
- 書き込みの規則(1.2・1.3): `Console.Out`・`Console.WriteLine` を使わず、BOM なし UTF-8 の `StreamWriter`(`AutoFlush = true`、`NewLine = "\n"`)を使うこと、`Write` のあとに `Flush` すること、標準出力の書き込みの失敗(例外)を `watch` は 65、`convert` は 67 の終了コードに結びつけること、`SIGPIPE` の設定を変えないこと、終了の前に `Flush` を完了させること(1.4)。

実装に任せてよい点:

- 内部で「表示する内容」をどう表現するか、文言の表をどう保持するか(10 章は目安)。
- 言語・色の判定の遅延評価(観測できる差がない範囲で)。
- `Main` から `return` するか、(想定外の失敗の経路でだけ)`Environment.Exit` を呼ぶか(10 章 6)。

## 13. 未確認

- 定義の誤りの `<原因>` は 03 章、変換の誤りの `<原因>` は 09 章が全件を規定する(この章に未確認はない)。
- **Unix で、閉じたパイプへの書き込みが、標準のコンソールのストリームでは検出できない**(ソースの記述で確認した。1.3.1)。この版は、その差(`watch`・`convert` が、閉じたパイプで 65・67 でなく 0 で終わりうる)を受け入れる。`FileStream` を標準出力のファイル記述子に直接被せる方法で検出できるかは未確認。Windows の閉じたパイプでの `IOException` の発生は未確認。Linux の `/dev/full` への出力が、65・67 になること(`ENOSPC` が例外になること)は、ソースの記述からの推測で、実機では未確認。
- Windows のコンソール・パイプ・ファイルへの出力の文字コード(`Console.OutputEncoding` を UTF-8 にしたうえで、UTF-8 のバイト列を標準ストリームに書いたとき、コンソールとリダイレクト先の両方で意図どおりに表示・保存されるか。1.2.2 の 3)。
- 標準入出力が閉じられている場合(`>&-`)の、`Console.OpenStandardOutput()` の挙動(1.2.2)。
- `StreamWriter` を `Dispose` しないときの、バッファの流し直しと、ファイナライザの挙動(1.2.2、1.3.2 の 5)。
- 端末(疑似端末)で `Console.IsOutputRedirected` が偽になること(6.3)。Windows での `IsOutputRedirected` の実装。
- 文化に依存する `string` の API の既定の挙動、`ToString()` の既定の書式(4.9、11 章)。`Trim` 系の挙動(11 章)。いずれも、公式ドキュメントを取得しておらず未確認(この仕様は、それらを使わない書き方を規定する)。
- `NO_COLOR` の値の扱い(未設定または空なら色あり、空でない値なら色なし)は規定どおりで、.NET の挙動に依存しない。
- 環境変数・引数に含まれる、不正な UTF-8 の置換のされ方(04 章 4.4.2)。
- サポートする最小の .NET の版(実装時に確認する。最新の LTS と、サポート中の 1 つ前の LTS)で、1.2・1.3 の確認(閉じたパイプ、`/dev/full`、BOM なし UTF-8、LF)を、3 OS で行うこと。
