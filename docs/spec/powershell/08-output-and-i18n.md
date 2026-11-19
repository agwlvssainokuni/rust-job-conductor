# 08. 出力と多言語化

この章は、PowerShell 7.x(`pwsh`)で実装する `conductor` が出す文字列の全文、出力先、標準出力・標準エラーへの書き込みの規則(エンコーディング、BOM なし UTF-8、LF、フラッシュ、閉じたパイプ、終了前の出力の完了、`Write-Host`・`Write-Output`・`Write-Error` を使わない理由)、言語の決め方、`watch` の表、色、エスケープの規則(サロゲートを含む)、表示幅と埋め、文字列の単位の扱いを規定する。文言は、この章の表のとおりに(文字単位で)出力する。コマンドラインは [04-cli.md](04-cli.md)、終了コードとエラーの設計は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md)、定義の誤りの原因の文は [03-definition-format.md](03-definition-format.md)、ステートの形式は [06-state-file.md](06-state-file.md)、変換は [09-converter.md](09-converter.md)、子プロセスとシグナルは [07-process-and-signals.md](07-process-and-signals.md) を参照する。

「確認(出典: …)」は、公式ドキュメント(learn.microsoft.com の PowerShell 7.4 の about_*、.NET API リファレンス)、または公開されているソース(PowerShell、.NET のランタイム)を取得して確かめた内容、「未確認」は、この環境に `pwsh` も .NET SDK もなく実機で動かせないこと、または出典を取得できなかったことで、確かめられていない内容である。.NET の標準ストリームに関する確認(1.2.1、1.3.1)は、本仕様の作成の調査で .NET の公式ドキュメントとランタイムのソースから確かめた内容を引用したものであり、**PowerShell のホストの中で動かしたときの挙動は、すべて未確認**(13 章)。

実装は `src/Conductor/Private/Report/` に置く。処理の流れは、「表示する内容」(種類と値を持つ値)を、`Report` の関数に渡し、関数がロケールと水準に従って文言に直して書く、という分担にする。

## 0. この版の文言の決め(まとめ)

文言は、この章の表のとおりに(文字単位で)出力する。この版が採った書式(`.psd1` を主、JSON を副)に関わる文言は次のとおり。

| 場所 | 文言 |
|------|------|
| 使い方(英語)の `convert` の行 | `conductor convert <file> --to <psd1\|json>` |
| 使い方(日本語)の `convert` の行 | `conductor convert <ファイル> --to <psd1\|json>` |
| 使い方の `--config` の既定(英語) | `(default: conductor.psd1 in the current directory)` |
| 使い方の `--config` の既定(日本語) | `(省略時: 実行ディレクトリの conductor.psd1)` |
| 使い方の `--to`(英語) | `--to <psd1\|json>    target format of convert` |
| 使い方の `--to`(日本語) | `--to <psd1\|json>    convert の変換後の形式` |
| ステートの原因: 構文の誤り | `JSON として読めません` |
| ステートの原因: 書き出しの失敗 | `内容を JSON にできません` |
| 定義の誤りの種類 `retryNotTable`(英語 / 日本語) | `retry must be a map` / `retry はマップで書きます` |
| 定義の誤りの種類 `unsupportedFormat`(英語 / 日本語) | `unsupported definition file format` / `定義ファイルの形式に対応していません` |
| 例の中の定義・ステートのファイル名 | `c.psd1`、`c.psd1.state`(JSON の例は `c.json`、`c.json.state`)など |

(表の中の `\|` は、Markdown の表のための書き方で、実際の文言は `|` 1 文字。)

- ステートの原因 2 件は、ステートファイルが JSON であるため(10 章 7.2 に全件)。
- `retryNotTable` の識別子は内部の名前であり、利用者には見えない。文言は、`.psd1` のハッシュテーブルと JSON のオブジェクトの共通の語である「マップ(キーと値の対応)」で言う。
- 定義の誤りの「原因」の文(日本語の固定の文)は、[03-definition-format.md](03-definition-format.md) が規定する。この章は、種類の文(4.3)だけを規定する。
- 変換の誤りの「原因」の文は、[09-converter.md](09-converter.md) が規定する。10 章 7.3 に全件を再掲する。
- **直した箇所の一覧は 8.3 にある。**

## 1. 出力先の使い分け

表示は、処理の流れが作る「表示する内容」を、文言に直して、水準に従って出す。水準は 4 つある。

| 水準 | 出力先 | 出す条件 | 書き込み失敗 |
|------|--------|----------|--------------|
| `Error` | 標準エラー | 常に | 無視(本筋を止めない) |
| `Notice` | 標準エラー | 常に(エラーではない知らせ。リトライの前の知らせだけ) | 無視 |
| `Verbose` | 標準エラー | `-v` のときだけ | 無視 |
| `Output` | 標準出力 | 常に | 呼び出し元に失敗(`[bool]` の偽)を返す(`watch` は 65、`convert` は 67 で終わる) |

- どの文言も、最後に改行(LF)を含む。複数の誤りは 1 行ずつ。
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

#### 1.2.1 確認した挙動(.NET の標準ストリーム)

| 確認した事項 | 結果(出典) |
|--------------|--------------|
| `[System.Console]::OutputEncoding` の既定 | 確認(出典: 「Console.OutputEncoding」): コンソールは、出力エンコーディングで、アプリが書いた文字をコンソールに表示する文字へ変換する。既定のコードページは、システムのロケールで決まる。UNIX 系では、確認(出典: ランタイムのソース `ConsolePal.Unix.cs`): 環境変数の文字集合の指定から得たエンコーディングから、バイト順マークを除いたものを返し、得られなければ既定のエンコーディング。つまり、既定の出力エンコーディングは、環境(ロケール)に依存する |
| `OutputEncoding` にできるエンコーディング | 確認(出典: 同): UTF-8 と UTF-16 は使える。UTF-32 は使えず、設定すると `IOException` |
| `UTF8Encoding` の BOM と不正な文字 | 確認(出典: 「UTF8Encoding Constructor」): `[System.Text.UTF8Encoding]::new($false, $true)` の第 1 引数 `$false` は BOM なし(`GetPreamble()` が長さ 0 の配列)。第 2 引数が `$true` のとき、対になっていないサロゲートの符号化で例外(`EncoderFallbackException`)。`$false` のときは、対になっていないサロゲートを `EF BF BD`(U+FFFD の UTF-8)に置き換えて符号化する |
| `[System.Console]::OpenStandardOutput()` | 確認(出典: 「Console.OpenStandardOutput」): 標準出力のストリーム(`Stream`)を返す。文書の例は、`StreamWriter` で包み、`AutoFlush = true` を付けている。`OpenStandardOutput(int bufferSize)` の `bufferSize` は「効果がない」 |
| `[System.Console]::IsOutputRedirected` | 確認(出典: 「Console.IsOutputRedirected」): 標準出力がリダイレクトされているかを表す `bool`。UNIX 系では、確認(出典: ランタイムのソース): 標準出力(fd 1)が端末(`isatty`)でなければ真 |

PowerShell の側で確認した事項:

| 確認した事項 | 結果(出典) |
|--------------|--------------|
| `$Env:` 経由の環境変数 | 確認(出典: about_Environment_Variables): 空文字を代入すると、その環境変数が削除される。UNIX 系では名前の大文字小文字を区別する。`NO_COLOR` が存在すると、`$PSStyle.OutputRendering` が `PlainText` になる(**`conductor` は `$PSStyle` を使わない。6.3**)。`TERM` が `dumb` だと `$Host.UI.SupportsVirtualTerminal` が偽になる(**`conductor` は `TERM` を見ない**) |
| `-File` の実行と出力 | 確認(出典: `ConsoleHost.cs`): ファイルの実行は、出力用の仕組み(`AddOutputter`)を付けて行われる。`conductor` は、パイプラインに出力を流さない(1.5)ので、この仕組みは何も出力しない |
| `exit` の前の出力の完了 | 確認(出典: about_Language_Keywords): `finally` は `exit` でも実行される。ただし、`Write-Host` など、ホストを通す出力の完了と、自前のストリームの完了は別(1.4) |

#### 1.2.2 規定

1. **`Write-Host`・`Write-Output`・`Write-Error`・`Write-Verbose`・`Write-Warning`・`Write-Information`・`Out-Host`・`Out-String`・`[System.Console]::Out`・`[System.Console]::Error`・`[System.Console]::WriteLine` を、`conductor` の表示に使わない**(理由は 1.5)。次のとおり、標準ストリームを `[System.Console]::OpenStandardOutput()`・`[System.Console]::OpenStandardError()` で取り、自前の `StreamWriter` で包んで書く。

   ```powershell
   # Private/Report/ の初期化(骨格。実装コードは規定しない)
   $script:StrictUtf8  = [System.Text.UTF8Encoding]::new($false, $true)   # BOM なし。不正な文字で例外
   $script:LenientUtf8 = [System.Text.UTF8Encoding]::new($false, $false)  # BOM なし。不正な文字は U+FFFD
   # 標準出力: [System.IO.StreamWriter]::new([System.Console]::OpenStandardOutput(), $script:StrictUtf8)
   #            .AutoFlush = $true; .NewLine = "`n"
   # 標準エラー: [System.IO.StreamWriter]::new([System.Console]::OpenStandardError(), $script:LenientUtf8)
   #            .AutoFlush = $true; .NewLine = "`n"
   ```

   - 文字コードは UTF-8、**BOM なし**(`StreamWriter` は、エンコーディングの `GetPreamble()` が空なら、BOM を書かない。1.2.1 のとおり、`$false` を指定した `UTF8Encoding` は空の配列を返す)。ロケールにも `[System.Console]::OutputEncoding` にも `$OutputEncoding` にも依存しない。`[System.Text.Encoding]::UTF8` を使わない(BOM を出すインスタンスである可能性があり、取得した文書では確認していない。未確認)。
   - 改行は **LF だけ**。`WriteLine`・`[System.Environment]::NewLine` を使わず、文言の末尾の `"`n"`(LF 1 文字。`[char]10` と書いてもよい)を、文字列に含めて書く。`NewLine = "`n"` の設定は、うっかり `WriteLine` を呼んでも CRLF にならないための保険(Windows のコンソール・ファイル・パイプのどれでも LF だけにする)。
   - **`AutoFlush = $true`**: 1 回の `Write` ごとに、書き込みの末尾でストリームに流す。表示は、1 回の表示(1 行、または 1 かたまり)につき、組み立て済みの 1 つの文字列を 1 回の `Write` で書く(複数回の `Write` に分けない)。それに加えて、`AutoFlush` を過信せず、`Write` のあとに `Flush()` も明示して呼ぶ(多重の呼びで害はない)。`StreamWriter` はファイナライザでバッファを流さない(.NET の既知の挙動。取得した文書では確認していない。未確認)ので、`Flush()` を呼ばずに終わると出力が失われうる。1.4 の終了の手順で必ず流す。
   - 標準出力は `throwOnInvalidBytes: $true`(厳密): 対になっていないサロゲートが入り込んだら、黙って壊れた出力にせず、`EncoderFallbackException`(1.2.1 の例外の型)を、書き込みの失敗として扱う(1.3)。通常の流れでは、表示する文字列は 9 章のエスケープ(対になっていないサロゲートを U+FFFD にする)を通っているので、符号化の失敗は起こらない。標準エラーは `$false`(`conductor` の表示が、符号化の失敗で止まらないようにする。対になっていないサロゲートは U+FFFD になる。通常の流れでは到達しない)。
   - 2 つの `StreamWriter` は、`$script:` のスコープの変数に、起動時に 1 度だけ作る(最初の表示が必要になるまで作りを遅らせてよい。その場合も、作りの失敗は書き込みの失敗として扱う)。`OpenStandardOutput()`・`OpenStandardError()` を、表示のたびに呼ばない。
   - 作った `StreamWriter` と、その下のストリームを `Dispose` しない(`Dispose` が標準出力のハンドルを閉じるか、取得した文書では確認していない。未確認。プロセスの終了まで開けておく)。
   - ストリームの作り(`OpenStandardOutput()` など)が例外を投げる場合(標準出力が閉じられているとき。挙動は未確認)は、その出力口を「書き込みに失敗した」ものとして扱う(標準エラーは無視、標準出力は 1.3 の規則)。
2. `[System.Console]::OutputEncoding` を、UNIX 系では**設定しない**(読まない。上の自前の出力口が、環境に依存せず UTF-8 のバイト列を書くため)。`$OutputEncoding`(PowerShell が、外部プログラムへ文字列を送るときのエンコーディング。取得した出典はなく未確認)も設定しない。
3. Windows のコンソール(端末)への出力: 標準出力が端末のとき(`[System.Console]::IsOutputRedirected` が偽)は、起動時に、`[System.Console]::OutputEncoding` の現在の値を保存し、`[System.Text.UTF8Encoding]::new($false)`(BOM なし)を設定してから、出力口を作り、`Invoke-Conductor` の `finally` で保存した値に戻す(対話セッションのコンソールの設定を、後に残さない)。標準エラーが端末のときも同じ(`[System.Console]::IsErrorRedirected`)。設定の失敗(`IOException`、`UnauthorizedAccessException`)は無視する。コンソールに、UTF-8 のバイト列を、日本語を含めて正しく表示できることを意図する。**未確認**: 標準ストリームに UTF-8 のバイト列を書いたとき、Windows のコンソールが、コンソールの出力コードページで解釈するか(その場合、`OutputEncoding` の設定が必要)、`OutputEncoding` を設定すると、終了後もコンソールのコードページが変わったままになるか、`pwsh` 自身が起動時にコードページを設定しているか。標準出力・標準エラーをファイルやパイプにリダイレクトしたときは、この設定をせず、UTF-8 のバイト列がそのまま出る(未確認)。色は常に使わない(6.3)。
4. ジョブ本体の出力は `conductor` を通らないので、この設定の対象外(子のエンコーディングは子が決める)。

### 1.3 書き込みの方式と、標準出力が閉じている・書けないとき

#### 1.3.1 確認した挙動

| 確認した事項 | 結果(出典) |
|--------------|--------------|
| UNIX 系で、読み手が閉じたパイプへの書き込み(`EPIPE`) | 確認(出典: ランタイムのソース `ConsolePal.Unix.cs` の書き込み): 書き込みが `EPIPE` で失敗したとき、コードは、「Broken pipe... likely due to being redirected to a program that ended, so simply pretend we were successful.」というコメントのとおり、**例外を投げずに戻る(データは捨てられる)**。つまり、`OpenStandardOutput()` と `OpenStandardError()` のストリームでは、UNIX 系で、閉じたパイプへの書き込みを、`IOException` として検出できない |
| 上の以外の書き込みの失敗 | 確認(出典: 同): `EAGAIN` は、書き込み可能になるまで待って再試行する。それ以外の `errno` は、`IOException` 系の例外を投げる。例: 容量不足(`ENOSPC`)や、書き込みできないデバイスへの書き込み |
| `SIGPIPE` | 確認(出典: ランタイムのソース `signal.cpp`): 実行環境の初期化で `SIGPIPE` を無視にする。つまり、閉じたパイプへの書き込みで、プロセスが信号で死ぬことはない |
| Windows で、読み手が閉じたパイプへの書き込み | 取得できる出典がなく、**未確認**(`IOException` になることを想定する) |
| `[System.Environment]::Exit` と、バッファされた出力 | `Exit` は、ほかのスレッドが動いていても、アプリを直ちに終了させ、`finally` を実行しない(.NET のリファレンス)。**`conductor` は `exit`(入口スクリプトのみ)を使い、`Exit` を使わない**(10 章 6.2)。`StreamWriter` のバッファを流すかは、取得した文書に記述がない(未確認)ので、`exit` の前に必ず自分で `Flush()` する |
| .NET のメソッドの例外の型と `catch` | 確認(出典: about_Try_Catch_Finally): `catch [System.IO.IOException]` のように型を書くと、その型と派生した型の例外を捕捉する(例は `DownloadFile` メソッドの呼び出しで起きる例外を型で捕捉している)。PowerShell は、すべての例外を `RuntimeException` に包む(同) |

要するに、(1) 閉じたパイプへの書き込みで、プロセスが信号で死ぬことはない。(2) UNIX 系では、閉じたパイプへの書き込みが「成功したふり」になり、`IOException` にならない。**この版では、UNIX 系で `watch | head -0` のように読み手が先に閉じたとき、`watch` は 65 ではなく 0 で終わりうる**(`convert` も 67 ではなく 0)。この差は、.NET の標準のストリームの設計によるもので、受け入れる。閉じたパイプの検出を、標準のストリームを使わずに行う方法(標準出力のファイル記述子を直接 `FileStream` に包む方法など)は、動作を確認できておらず(未確認)、この版の必須にしない。(3) 容量不足などの、`EPIPE` 以外の失敗は、例外になる(1.3.2 で 65・67 に結びつける。Linux では、`/dev/full` への出力で再現できる想定。未確認)。

#### 1.3.2 規定

1. **書き込みは、`Private/Report/` の 1 つの関数 `Write-ConductorStream`(`-Stream 'Out'|'Err'`、`-Text <string>`。戻り値は `[bool]`。真が成功)を通す**。本番の実装は、次の形にする(骨格)。名前の `Write-` は、動詞の規約に合わせたもので、PowerShell の `Write-Output` などとは無関係。

   - 出力口が作れていない、または一度書き込みに失敗している(1.3.2 の 5)ときは、何も書かず、偽を返す。
   - `Write($Text)` のあと、必ず `Flush()` を呼ぶ。標準出力・標準エラーとも、行ごとではなく、1 回の表示(1 行、または 1 かたまり)につき 1 回の `Write` と 1 回の `Flush`。
   - 捕捉する例外は、`[System.IO.IOException]`(閉じたパイプ、容量不足などを含む)、`[System.ObjectDisposedException]`(閉じたストリームへの書き込み)、`[System.Text.EncoderFallbackException]`(符号化できない文字)、`[System.NotSupportedException]`(書き込みできないストリーム)に**限る**。これらを、書き込みの失敗として扱い、偽を返す(例外を外に出さない)。**型を指定しない `catch` で握りつぶさない**(想定外の失敗が、書き込みの失敗に化けないように。10 章 8.1)。
2. 水準ごとの書き込みの失敗の扱い:
   - `Output` 水準(標準出力): 関数の戻り値が偽のとき、呼び出し元(`watch`・`convert`)が、`cannot write to standard output` を標準エラーに出して、`watch` は 65、`convert` は 67 で終わる。**例外にしない**(10 章 8.3)。
   - `--help` と `--version` の出力: 書き込みの失敗は無視して、終了コード 0 で終わる(失敗を報告しない)。
   - `Error`・`Notice`・`Verbose` 水準(標準エラー): 失敗を握りつぶして続ける(本筋を止めない)。
3. **標準エラーの書き込みも、`Flush()` まで完了してから次へ進む**。パイプ・ファイルの先でも、`conductor` の行が、そのあとに起動する子プロセスの出力より前に出るようにするため(子は同じファイル記述子を直接使うので、親の書き込みが親のバッファに残ったまま子を起動すると、順序が入れ替わりうる)。特に、リトライの知らせ(`Notice`)と、ジョブの起動前に出す行は、`Flush()` を終えてから子を起動する。
4. 書き込みは、すべて同じスレッド(メインスレッド)で、呼び出しの順に直列に行う。シグナルのコールバック(別のスレッドで実行されうる。07 章)は、出力しない(フラグを立てるだけ)。
5. **終了時に、失敗したストリームのバッファを残さない**: 自前の出力口は `AutoFlush = $true` で、書き込みのたびに流し切るので、失敗した書き込みのデータが、プロセスの終了時に、もう一度失敗して終了コードを変える、という経路を持たない(取得した文書では、`StreamWriter`・`Console` のストリームが、終了時にバッファを流して失敗することは確認していない。ファイナライザや終了処理による流し直しの有無は未確認)。書き込みに一度失敗した出力口は、以降の書き込みを行わず(失敗を覚えて、何もしない)、1.4 の終了の流しも飛ばす。
6. `SIGPIPE` の設定を変えない(ランタイムが初期化時に無視にしている。1.3.1)。子プロセスの `SIGPIPE` の扱いは 07 章に従う。
7. 標準出力・標準エラーのバッファリングに頼らない(`Flush()` を呼ばずに終了しない)。

### 1.4 終了前に出力を完了させる規定

プロセスの終了の手順(入口スクリプトが行う。10 章 4.1・6 も参照):

1. `Invoke-Conductor` が返した終了コード(`[int]`)を受け取る。
2. 標準出力、続いて標準エラーを、`Flush()` する。`Flush()` の失敗(1.3.2 の 1 の例外)を捕捉して無視する。作れていない出力口と、書き込みに失敗した出力口は飛ばす。この処理は、`Invoke-Conductor` の `finally` の中で行ってよい(Ctrl+C でパイプラインが止められても、`finally` は実行される。10 章 6.4)。
3. そのあとで、**終了コードを 0〜255 の `[int]` にそろえて**(10 章 4)、入口スクリプトが `exit` に渡す(通常の終了の経路)。
4. `[System.Environment]::Exit` を使わない。想定外の内部失敗(10 章 6)の経路でも、標準エラーに 1 行を書き、`Flush()` を試みたあと、`return` で入口スクリプトに戻って `exit` する。
5. シグナルの登録は、終了する前に解除する(07 章。`finally`)。

### 1.5 `Write-Host`・`Write-Output`・`Write-Error` などを使わない理由

製品コードで、これらのコマンドレットを使わない(PSScriptAnalyzer の `PSAvoidUsingWriteHost` に加えて、構文木でコマンド名を走査するテストで、`Write-Host`・`Write-Output`・`Write-Error`・`Write-Verbose`・`Write-Warning`・`Write-Information`・`Out-Host`・`Out-String`・`Out-Default` の使用を禁止する。11 章)。

| コマンドレット | 使わない理由 |
|----------------|--------------|
| `Write-Output`(および、文の値がそのままパイプラインに出ること) | 出力は、関数の戻り値(成功ストリーム)に混ざる。確認(出典: about_Language_Keywords): `return` は、「writes the optional expression to the output」。値を返す関数の戻り値が、画面に出したい文字列と混ざり、`[int]` の終了コードの検証(10 章 6.1)が壊れる。画面への出力の書式・幅・エンコーディングは、ホストの出力の仕組み(`Out-Default`)に任され、LF 固定・BOM なし UTF-8 を保証できない |
| `Write-Host` | 出力先と改行が、ホストの実装に任される。PowerShell 5 以降は、情報ストリームを通すとされる(確認(出典: about_CommonParameters): `InformationAction` の設定で、`Write-Information` の値の表示が決まる。`Write-Host` が同じ経路か、リダイレクトの挙動は、取得した文書に記述がなく未確認)。標準出力・標準エラーのどちらに出るか、LF 固定か、エンコーディングは何かを、仕様で固定できない |
| `Write-Error` | `ErrorRecord` を書き、`$?` を偽にする(確認(出典: about_Automatic_Variables))。表示の形式は `$ErrorView` に従い(取得した出典はなく未確認)、`conductor: error: ...` の 1 行にならない。`$ErrorActionPreference = 'Stop'` では、終了エラーに格上げされる(確認(出典: about_CommonParameters、about_Preference_Variables))ので、表示のつもりの呼び出しが、処理を止める |
| `Write-Verbose`・`Write-Warning`・`Write-Information` | 表示は、`$VerbosePreference`・`$WarningPreference`・`$InformationPreference` などの設定変数に左右される(確認(出典: about_CommonParameters): `$VerbosePreference` の既定は `SilentlyContinue` で、冗長メッセージは既定では表示されない)。`-v` の出力は、この仕様の `[conductor] ` の形式(2 章)で、設定に関係なく出す必要がある。出力の接頭辞(`VERBOSE:` など。確認(出典: about_Environment_Variables の出力例): `VERBOSE: Performing the operation ...`)が付き、形式が合わない |
| `[System.Console]::Out`・`[System.Console]::Error`・`WriteLine` | `OutputEncoding` が環境(ロケール)に依存し(1.2.1)、`WriteLine` の改行が `[System.Environment]::NewLine`(Windows で CRLF)になりうる。`[System.Console]::OutputEncoding` を設定すると、その設定が `pwsh` のプロセス全体(対話セッションのコンソール)に残る。ホストと同じ書き込み口を共有するため、ホスト自身の出力との順序・バッファリングが未確認 |
| `Format-Table`・`Out-String`・`$PSStyle` | `watch` の表の幅と埋めを、仕様の表示幅の規則(6.2)に合わせられない。色は、`$PSStyle` を使わず、自前の ANSI の制御列で付ける(6.3) |

この仕様の書き込みは、1.2.2 の自前の `StreamWriter` に、組み立て済みの文字列を渡す 1 つの関数に集める。共通の設計判断どおり、画面への出力は専用の関数だけが行い、それ以外の関数は、値を返すだけにする。

## 2. `-v`(`--verbose`)

- `run`・`pipeline`・`validate` だけが受け付ける。1 回だけ(2 回は 68)。PowerShell の共通パラメータ `-Verbose`(`$VerbosePreference`)とは無関係(04 章 3.3)。
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
$ conductor pipeline -v nightly --config c.psd1 --run-id r1
[conductor] job=extract result=warning (exit code 1)
[conductor] transition: warning -> load (skipped: transform)
[conductor] job=transform skipped (skipped by a transition)
[conductor] job=load result=success (exit code 0)

$ conductor run -v a --config e.psd1        # success_exit_codes = @(5) で、ジョブが 0 で終了
[conductor] job=a result=error (exit code 0, conductor exits with 1)
conductor: error: job 'a': exit code 0 is not listed in success_exit_codes

$ conductor pipeline -v p --config d.psd1   # ジョブが SIGKILL で終了
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
| `PwshNotFound`(ランチャーが出す。04 章 1.3) | `conductor: error: pwsh (PowerShell 7) was not found in PATH` | 70 |
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

(識別子は、種別を表す文字列として、同じ綴りで使う。大文字小文字を区別する。10 章 8.1)

実出力(原因の文の例。`syntax` 以外の原因は 03 章の規定):

```text
$ conductor pipeline wrongname --config c.psd1        # LANG=C
conductor: error: c.psd1:3: pipeline name does not match the definition: 指定されたパイプライン名が、定義の `name` と一致しません
$ LANG=ja_JP.UTF-8 conductor validate --config nonexist.psd1
conductor: error: nonexist.psd1: 定義ファイルが存在しません: 指定の場所にファイルがありません
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
conductor: error: c.psd1.state: state file is corrupt: JSON として読めません; run `conductor reset` to discard the state
conductor: error: c.psd1.state: ステートファイルが壊れています: JSON として読めません; `conductor reset` でステートを初期化できます
```

(ステートのパスは、定義ファイル名に `.state` を足したもの。定義が `c.psd1` なら `c.psd1.state`。06 章。)

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
- 待ち時間の求め方は [05-execution-flow.md](05-execution-flow.md)(リトライ)に従う。`{秒}` と `{n}`・`{max}` は、整数を 10 進の整数で文字列にしたものを使う。**文化に依存しない書き方で作る**: `$n.ToString([System.Globalization.CultureInfo]::InvariantCulture)`。文言は、文字列の連結(`+`、または `[string]::Concat`)で組み立てる。`-f`(書式演算子)と、数値を含む二重引用符の文字列の展開に頼らない(`-f` は文化に依存しうる。確認していない挙動は未確認。文言のひな形には `{`・`}` を含むものがあり、`-f` の書式と衝突する)。待ち時間は 64 ビット符号なし整数の上限(18446744073709551615。20 桁)で飽和させる([05-execution-flow.md](05-execution-flow.md))ので、`[uint64]` をそのまま 10 進の文字列にする(`[double]`・`[decimal]` を経由しない。`[double]` の文字列化は、大きな値で `E+19` のような指数表記になりうる)。

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
- **表示幅**は、文字(コードポイント)ごとに、ASCII(U+0000〜U+007F)なら 1、それ以外は 2 として足す近似(全角を 2 とみなす)。実際の端末幅(結合文字、絵文字、ゼロ幅の文字など)は考慮しない。コードポイントの走査は、04 章 4.0 のサロゲートの対を 1 文字にする自前の走査と同じにする(`[string].Length` はコードユニット数なので、基本多言語面の外の文字を 2 と数えるが、この規定では 1 コードポイントなので、表示幅は 2)。**`.Length` を表示幅に使わず、`.PadRight`・`.PadLeft`、書式演算子の幅指定(`'{0,-10}' -f $s`)、`Format-Table`・`Out-String`、`[System.Globalization.StringInfo]`(書記素)、East Asian Width 相当の判定も使わない**(結果が、この近似と異なる)。
- 次の表の入力で、表示幅が決まる(この近似による。「実際の端末幅」ではない)。

  | 文字列 | `.Length`(コードユニット数) | コードポイント数 | 表示幅(この規定) | 備考 |
  |--------|-------------------------------|---------------------|-----------------------|------|
  | `step1` | 5 | 5 | 5 | ASCII |
  | `ジョブ` | 3 | 3 | 6 | 全角 |
  | `😀`(U+1F600) | 2 | 1 | 2 | サロゲートの対で 1 コードポイント |
  | `e` + U+0301(結合文字) | 2 | 2 | 3 | 結合文字を 2 と数える(端末の見た目の幅は 1) |
  | `🇯🇵`(U+1F1EF U+1F1F5) | 4 | 2 | 4 | 国旗は 2 コードポイント(端末の見た目の幅は 2) |
  | `日本語 abc` | 7 | 7 | 10 | |

- 見出し行: ジョブ見出しと状態見出しを、右に半角空白を足して列の幅にそろえ、補足見出しは埋めない。行末の空白は取らない(補足見出しは空でないので、実際には取る空白がない)。
- データ行: ジョブIDを埋め(列の表示幅にそろえる)、状態語を埋め(埋めは状態語の表示幅で数える。状態語は ASCII なので文字数と同じ)、補足を続ける。行全体の行末の空白を削る(補足が空なら、状態語のあとの埋めと区切りも削られる)。削る空白は ASCII の半角空白だけ(`$line.TrimEnd([char]0x20)`。引数なしの `TrimEnd()` は Unicode の空白(全角空白 U+3000、タブ、改行など)も削るので使わない。引数なしの `Trim` 系が、Unicode の空白を削る挙動は、この環境では出典を取得しておらず、未確認)。
- **埋めは、自前で `[string]::new([char]0x20, $n)` を足して行う**(`n = 列の幅 - その文字列の表示幅`。`n` が負にならないことは、列の幅が最大であることで保証される。`n` が 0 のときも、空文字を返す)。`.PadRight`・`.PadLeft` は、コードユニット数で数え、全角文字を 1 と数えるので、表示幅の規則と合わない(例: `'あ'.PadRight(4)` はコードユニット 4 になるが、表示幅の規定では `あ` は 2 なので、列の幅 4 にそろえるなら足す空白は 2)。
- 表の全体を 1 つの文字列(各行を `[System.Text.StringBuilder]` に `[void]$sb.Append($line).Append([char]10)` で連結する。`AppendLine`・`-join [System.Environment]::NewLine` は使わない。`Append` の戻り値は `StringBuilder` なので、`[void]` を付けて、パイプラインに流さない)に作ってから、`Write-ConductorStream`(1.3)で 1 回で書く(書き込みの失敗を 1 回で検出するため)。

### 6.3 色

- 色を使う条件(最初に必要になったとき一度だけ判定): 環境変数 `NO_COLOR` が未設定または空(値が `Test-ConductorSuspectText` の場合も未設定と同じ。04 章 4.4.2)、かつ標準出力が端末(`[System.Console]::IsOutputRedirected` が偽。確認: 1.2.1)、かつ実行環境が Windows でない。Windows は常に色なし(挙動をほかの OS の実装と揃えるための決定。Windows の端末の ANSI 対応の可否は、この規定のために確認していない)。OS の判定は、`Private/Runner/` の 1 か所(`Get-ConductorPlatform`。内部で `$IsWindows` を使ってよい。確認(出典: about_Automatic_Variables): `$IsWindows`・`$IsLinux`・`$IsMacOS` が PowerShell 7 にある)にまとめ、`Report` は `$IsWindows` などを直接参照しない(OS 差を 1 か所にまとめる規則)。端末判定は `Get-ConductorStdoutIsTerminal` が、OS を見ずに、`-not [System.Console]::IsOutputRedirected` だけを返す。
- **`$PSStyle`、`$Host.UI.SupportsVirtualTerminal`、`TERM` を使わない**。確認(出典: about_Environment_Variables): `NO_COLOR` が存在すると `$PSStyle.OutputRendering` が `PlainText` になる、`TERM=dumb` で `$Host.UI.SupportsVirtualTerminal` が偽になる。これらは、PowerShell 自身の出力の色の設定であり、`conductor` の条件は、この節の規則で固定する(`TERM=dumb` の端末でも、`NO_COLOR` が空で標準出力が端末なら、色を付ける。他の実装と同じ)。
- 端末の判定: `[System.Console]::IsOutputRedirected` は、標準出力が端末でないとき(パイプ・ファイル)真になる(確認: 1.2.1。UNIX 系では `isatty(1)` の否定。Windows の実装は出典を取得しておらず、未確認)。端末(疑似端末を含む)で偽になることは、この環境では確認できなかった(未確認。実装時に、実際の端末で確かめる)。`[System.Console]::IsOutputRedirected` に統一し、`IsInputRedirected` などを混ぜない。外部のモジュールは使わない。
- 色は、状態語だけに、ANSI の SGR で付ける: `ESC[<番号>m<語>ESC[0m`(ESC は U+001B。PowerShell の文字列では `[char]0x1B` で作る。`` "`e" `` は PowerShell 6 以降の書き方だが、`[char]0x1B` に統一する)のあとに埋めの空白。`success` = 32(緑)、`warning` = 33(黄)、`error` = 31(赤)、`skipped` = 36(シアン)、`pending` = 色なし(制御列を付けない)。
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

- 環境変数 `LC_ALL`, `LC_MESSAGES`, `LANG` を、この順に見る(`Get-ConductorEnvironmentVariable`。04 章 5.1)。最初に空でない値を採り、その値が `ja` で始まれば(`$value.StartsWith('ja', [System.StringComparison]::Ordinal)`)日本語、それ以外は英語。どれも未設定(`$null`)または空なら英語。値は大文字小文字を区別する(`JA_JP` は英語)。値が `Test-ConductorSuspectText`(04 章 4.4.2)のときは、その変数は未設定と同じ扱い(その変数は飛ばして、次の変数を見る)。
- 最初の空でない値で決まり、後の変数は見ない(`LC_ALL=C` かつ `LANG=ja_JP` は英語。`LC_ALL=` が空で `LANG=ja_JP.UTF-8` なら日本語)。
- 判定は、1 回の実行につき、最初に表示が必要になったときだけ行い、結果を保つ(`$script:Locale` が `$null` なら未判定)。
- ロケールを判定しない固定の文: `--version`、内部の誤りの文、シグナルの準備の失敗の文、ランチャーの `pwsh` が見つからない文(英語のみ)。
- 言語によらず英語: 状態語、`job=`・`result=`、`Run ID`、`conductor: error: `・`conductor: warning: `・`conductor: retry: `・`[conductor] ` の接頭辞、`--help` の中のオプション名。定義・ステート・変換の誤りの `<原因>` は日本語のまま。
- 必須: ロケールの判定に、`$PSCulture`・`$PSUICulture`・`Get-Culture`・`Get-UICulture`・`[System.Globalization.CultureInfo]::CurrentCulture`・`CurrentUICulture`・`InstalledUICulture`・`[System.Threading.Thread]::CurrentThread.CurrentCulture`・`Import-LocalizedData`・メッセージのリソースなどの、PowerShell と .NET の文化の仕組みを**使わず**、上の 3 つの環境変数だけで決めること。理由: 確認(出典: about_Automatic_Variables): `$PSCulture` は PowerShell 7 で実行空間の文化を反映し、`$PSUICulture` は OS の UI の文化(`CurrentUICulture.Name`)であって、環境変数 `LANG` などを反映するとは限らない。他の実装は環境変数だけで決めており、挙動を揃える必要がある。文字列の整形・比較にも、`[System.Globalization.CultureInfo]::InvariantCulture` と序数(`[System.StringComparison]::Ordinal`)を明示する(04 章 4.0、4.9)。
- PowerShell は、`switch`・`-eq`・`-like`・`-match` が既定で大文字小文字を区別しない。この章で書く比較(ロケールの先頭の `ja`、種別の識別子、状態語)は、序数の比較(`StartsWith(..., Ordinal)`、`[string]::Equals(..., Ordinal)`、`-ceq`、`switch -CaseSensitive -Exact`)で行う(04 章 4.0 の表)。

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
  conductor convert <file> --to <psd1|json>
  conductor --help | --version

Options:
  -v, --verbose       print job results, skips and transitions to standard error (run, pipeline; validate prints one line)
  --config <path>     definition file (default: conductor.psd1 in the current directory)
  --run-id <id>       run ID that separates the state file (run, pipeline, watch, reset; default: the environment variable
                      named by run_id_env in the definition, or CONDUCTOR_RUN_ID; an empty value means no run ID)
  --remove-on-complete
                      remove the state file when the run completes (run, pipeline; default: keep it, or the
                      remove_on_complete setting of the definition)
  --all               remove every state file of the definition, with or without a run ID (reset only)
  --to <psd1|json>    target format of convert
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
  conductor convert <ファイル> --to <psd1|json>
  conductor --help | --version

オプション:
  -v, --verbose       ジョブの結果、スキップ、遷移を標準エラーに表示する(run、pipeline。validate は 1 行)
  --config <パス>     定義ファイル(省略時: 実行ディレクトリの conductor.psd1)
  --run-id <ID>       ステートファイルを分ける run ID(run、pipeline、watch、reset。省略時: 定義の run_id_env の環境変数、
                      なければ CONDUCTOR_RUN_ID。空の値は run ID なし)
  --remove-on-complete
                      完了のときにステートファイルを消す(run、pipeline。省略時: 残す。定義の remove_on_complete に従う)
  --all               定義のステートファイルを、run ID の有無によらず、すべて消す(reset のみ)
  --to <psd1|json>    convert の変換後の形式
  -h, --help          この使い方を表示する
  -V, --version       版を表示する
```

### 8.3 この版で文言を直した箇所(一覧)

元の `conductor` の文言から、次の 3 点(英語・日本語とも。計 6 行)だけを、定義の主書式が `.psd1`(副書式が JSON)になったことに合わせて直した。それ以外の行は、文字単位で同じ。実装は「直した後」の文言だけを出す。

| 行 | 直した後 | 直した内容 |
|----|----------|------------|
| `convert` の使い方の行(英語) | `conductor convert <file> --to <psd1\|json>` | 変換先の書式の組を `psd1` と `json` にした |
| `convert` の使い方の行(日本語) | `conductor convert <ファイル> --to <psd1\|json>` | 同上 |
| `--config` の説明(英語) | `(default: conductor.psd1 in the current directory)` | 既定の定義ファイル名の拡張子を `psd1` にした |
| `--config` の説明(日本語) | `(省略時: 実行ディレクトリの conductor.psd1)` | 同上 |
| `--to` の説明(英語) | `--to <psd1\|json>    target format of convert` | 変換先の書式の組を `psd1` と `json` にした |
| `--to` の説明(日本語) | `--to <psd1\|json>    convert の変換後の形式` | 同上 |

(表の `\|` は Markdown の表のための書き方で、実際の文言は `|` 1 文字。)

あわせて、ステートの原因・定義の誤りの種類の文言は、0 章の表のとおり(`JSON として読めません`、`内容を JSON にできません`、`retry はマップで書きます`、`定義ファイルの形式に対応していません`)。変換の誤りの原因(10 章 7.3)は、`.psd1` に合わせて、次を置き換え・削除した(「元」は、`.psd1` の前提に直す前の文言)。元の文法の誤りの文(書式名が `.psd1` でないもの)を `.psd1 の文法に誤りがあります` に置き換えた。タグ・複数の文書・アンカー・マージキーに関する 4 件(`タグは使えません`、`複数の文書は書けません(…)`、`アンカーが自分自身を含んでいます`、`マージキーの値がマップではありません`)は、`.psd1` に対応する構文が無いので削除し、`この構文は使えません(データだけを書けます)` を追加した。`NaN と無限大は JSON で表せません` は、`.psd1` の許可する構文に `NaN`・無限大を表す書き方が無いので削除した。`文書が空です` は、`.psd1` の文書が空(式が 0 個)のとき。

### 8.4 全文の保持と検査

- 全文は、PowerShell のソースに、行の文字列の配列(`[string[]]`。各行は単一引用符の文字列。行の中の `'` は `''` と書く)として持つ(here-string(`@' ... '@`)で書くと、先頭と末尾の改行、行末の空白、ソースの改行コード(CRLF のチェックアウト)の影響を受けやすいため)。二重引用符の文字列を使わない(`$`・`` ` `` が展開・エスケープされるため)。「各行を `[string]::Join("`n", $lines)` で連結し、末尾にも `"`n"` を付ける」実装にして、受け入れテストで文字単位に比較する。
- 使い方の誤りの表示は、誤りの行、空行(`"`n"` だけの行)、全文、の順。

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
| 対になっていないサロゲート(U+D800〜U+DFFF の単独の `char`) | 走査の前に、04 章 4.4.2 の `ConvertTo-ConductorReadableText` で U+FFFD に置き換える(下記)。置き換え後の U+FFFD はそのまま出る |
| それ以外 | そのまま(日本語もそのまま。サロゲートの対で表す文字(絵文字など)もそのまま) |

PowerShell での実装の方針:

- **最初に、文字列全体を 04 章 4.4.2 の `ConvertTo-ConductorReadableText` に通す**。対になっていないサロゲートが U+FFFD になる。これにより、後続の処理と、標準出力・標準エラーの符号化(1.2)で、`EncoderFallbackException`(標準出力は厳密な符号化)が起きない。通さずに、単独のサロゲートの値を `\u{d800}` のように出すと、元のバイトとは無関係な値を見せてしまう。
- 走査は、04 章 4.0 の自前の走査(サロゲートの対を 1 コードポイントにする)で行い、コードポイントの値を `[int] $cp` で得る(対のとき `[char]::ConvertToUtf32($high, $low)`)。16 進は `$cp.ToString('x', [System.Globalization.CultureInfo]::InvariantCulture)`(小文字で、先頭の 0 が付かない書式。`0` は `0`)から `\u{` と `}` で囲んで作る。大文字になる `'X'`、桁を固定する `'x4'`・`'x8'` を使わない。出力の文字列は、単一引用符の文字列(`'\u{'`、`'}'`、`'\n'`)で書く(二重引用符の中では、`` ` `` が escape になり、`\` はそのままだが、`$` や `` ` `` の混入を避けるため)。
- Cc の判定は、範囲(`$cp -lt 0x20 -or ($cp -ge 0x7F -and $cp -le 0x9F)`)で書く(`[char]::IsControl` は同じ範囲だが、範囲を直接書く)。書式・双方向の文字は、上の表の範囲を列挙する(Unicode のカテゴリ `Cf` 全体ではない。表にない `Cf` の文字はそのまま出す)。
- `-replace`・`-match`・`[regex]::Escape`・`[System.Net.WebUtility]` など、正規表現と組み込みのエスケープは、規則の形(`\u{..}`、小文字の 16 進、先頭の 0 なし、対象とする範囲)と異なる、または大文字小文字を区別しないので使わない。
- 出力する文字列は、`[System.Text.StringBuilder]` に 1 かたまりずつ `[void]` を付けて足す。

切り詰め:

- 名前用(`ConvertTo-ConductorEscapedName`): 上限 64 文字。数えるのは、エスケープ後の文字数(コードポイント単位。エスケープ列(`\u{202e}` など)の各文字を 1 文字ずつ数える。基本多言語面の外の文字は 1 文字)。次の 1 文字(エスケープ列は 1 かたまり)を足すと 64 を超えるなら、そこで打ち切って末尾に `…`(U+2026)を 1 文字付ける。したがって出力は最大 65 文字。エスケープ列は途中で切らない。出力する文字列(`StringBuilder`)に、1 かたまりずつ足していく(エスケープ後の文字列に対して、`.Substring(0, 64)` のように位置を指定して切らない。エスケープ列やサロゲートの対を分断するため)。
- パス用(`ConvertTo-ConductorEscapedPath`): 上限なし(エスケープだけ)。
- 名前用を使うもの: 使い方の誤りの対象、ジョブID、遷移先、スキップのID、リトライのジョブID、パイプライン名、run ID、コマンド名。パス用を使うもの: パス(定義ファイル、ステートのパス、`reset --all` のファイル名、変換の入力ファイル、原因、キーの位置)。
- 文字に直せない部分(対になっていないサロゲート、実装が置き換えた U+FFFD)は、先頭の `ConvertTo-ConductorReadableText` が U+FFFD にしてから通す(U+FFFD は、制御文字でも書式文字でもないので、そのまま出る)。

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
| `😀` × 64(U+1F600。`.Length` は 128、コードポイントは 64) | 64 | `😀` × 64(切らない) |
| `😀` × 65 | 64 | `😀` × 64 + `…` |
| `a<U+D800>b`(単独の高位サロゲート) | 64 | `a<U+FFFD>b` |
| `a<U+DC00>b`(単独の低位サロゲート) | 64 | `a<U+FFFD>b` |
| `a<U+D83D>`(末尾の高位サロゲート) | 64 | `a<U+FFFD>` |
| `a<U+D83D><U+DE00>b`(対になっている。😀) | 64 | `a😀b`(そのまま) |

定義の検証・ステート・変換の各処理が、誤りの文を内部で作るときにも、同じ規則のエスケープを使う(エスケープだけの小さな部品を、`Private/Text/` に置き、`Report` と、定義・ステート・変換の各処理が、同じものを使う。依存の向きを守るため、エスケープは下位に置く。02 章)。その場合、定義とパスのファイル名には 256 文字の上限を使う実装がありうるが、`conductor` の表示では、ファイル名をパス用エスケープ(切らない)で出す。したがって、表示されるファイル名は切り詰められない(必須)。

## 10. 実装方針の選定(PowerShell)

| 項目 | 採用 | 理由・補足 |
|------|------|------------|
| メッセージの保持 | ロケールごとの読み取り専用の辞書(`[System.Collections.Generic.Dictionary[string, string]]::new([System.StringComparer]::Ordinal)`)を `Private/Report/` に持つ。キーは、この章の識別子(文字列)。**`@{}` のハッシュテーブルのリテラルは、キーの大文字小文字を区別しないので、辞書には使わない**(共通の設計判断 9・18)。全キーがすべてのロケールにあることを、Pester のテストで、キーの一覧との照合で確かめる | 表の全件を持てているかを、テストで検出できる。ただし、ロケールごとの語順の違いがあるメッセージは、組み立てる関数(下)で作る |
| プレースホルダの置換 | 固定の文字列に、値を連結する(`+`、または `[string]::Concat`)。**入力に由来する文字列を、書式演算子 `-f` の書式文字列にしない。文言のひな形にも `-f` を使わない**。値(ジョブIDなど)に `{n}` のような文字列が入っても、再度置換されないこと。**二重引用符の文字列の展開(`"... $x ..."`)に、入力に由来する値を、数値の書式つきで頼らない**(数値は `.ToString([System.Globalization.CultureInfo]::InvariantCulture)`) | 値の中の `{` `}` `$` が、書式や展開として再解釈されないことを、設計で保証する |
| 文言の組み立て | 種類ごとの関数(例: `Format-ConductorRetryNotice -Locale <string> -JobId <string> -ExitCode <int> -Attempt <int> -Max <int> -Seconds <uint64>`)。ロケールによる分岐は関数の中(`switch -CaseSensitive -Exact`) | 日本語・英語で語順が違う(リトライの知らせ)ため、単純な置換の表では足りない |
| 端末判定 | `[System.Console]::IsOutputRedirected`(6.3)。`Get-ConductorStdoutIsTerminal` の 1 か所を通す | 外部のモジュールは使わない |
| 色 | 自前の SGR の定数 | `$PSStyle` を使わない。`NO_COLOR` や端末判定の規則を、この規定に固定するため |
| 表示幅 | 自前(ASCII なら 1、それ以外は 2。コードポイント単位) | 仕様の近似に正確に合わせるため |
| エスケープ | 自前(9 章) | 仕様の表に正確に合わせるため |
| 書き込み | `Write-ConductorStream`(`Private/Report/`。標準出力・標準エラーの 2 つ。`-Text` を書いて流す。戻り値は `[bool]`)。本番の実装は `StreamWriter` を包んだもの(1.2、1.3)。テストでは、`$script:` の出力口の変数を、メモリ上のストリーム(`[System.IO.MemoryStream]`)を包んだ `StreamWriter` に差し替える | テストで差し替えるため |
| ログ(`Write-Host`、`Write-Verbose`、`Write-Information`、`Start-Transcript`、`Trace-Command`) | 使わない | 出力の形・出力先・水準が、この章で固定されているため(1.5) |

### 10.1 骨格(例)

次は、関数の署名と責務の一覧(実装コードではない)。

| 関数 | 責務 |
|------|------|
| `Send-ConductorOutput -Text <string>` → `[bool]` | `Output` 水準。標準出力に書く。偽が失敗 |
| `Send-ConductorError -Text <string>` → なし(`[void]`) | `Error`・`Notice` 水準。標準エラーに書く。失敗は握りつぶす。`Flush()` まで終えてから戻る(1.3.2 の 3) |
| `Send-ConductorVerbose -Text <string>` → なし | `Verbose` 水準。`-v` のときだけ標準エラーに書く |
| `Complete-ConductorOutput` → なし | 終了前の完了(1.4)。`Flush()` を試みる。`finally` から呼ぶ |
| `Get-ConductorLocale` → `[string]`(`'en'` か `'ja'`) | 7 章。最初に必要になったときに決めて、`$script:Locale` に保つ |

`Send-ConductorError` などの動詞 `Send` は、承認された動詞である(PSScriptAnalyzer の動詞の規則に合う。確認していない動詞の一覧は未確認で、実装時に `Get-Verb` で確かめる)。これらの関数は、戻り値が `[void]` のものも、パイプラインに値を出さない。

## 11. 文字列の単位(コードポイント)

PowerShell の `[string]` は、.NET の `System.String` で、UTF-16 のコードユニット(`[char]`)の列である。基本多言語面の外の文字(絵文字など。U+10000 以上)は、2 つの `[char]`(サロゲートの対)になり、`.Length` は 2 を返す。`conductor` が文字を数える・切る・並べる場面の単位は次のとおり(「コードポイント」は、サロゲートの対を 1 つと数える自前の走査。04 章 4.0)。

| 場面 | 単位 | 使う方法 | 使ってはならない方法 |
|------|------|----------|----------------------|
| 引数の連結の分解(04 章 4.1) | コードポイント | 04 章 4.0 の自前の走査 | `foreach ($c in $s.ToCharArray())` で 1 つずつ(サロゲートの対を分断する)、書記素の分割(`StringInfo`) |
| エスケープの走査(9 章) | コードポイント | 自前の走査と `[int] $cp`(走査の前に `ConvertTo-ConductorReadableText` で単独のサロゲートを除く) | `-replace`・`[regex]::Escape`・組み込みのエスケープ |
| 名前用の切り詰め(9 章) | コードポイント(エスケープ後) | 1 かたまりずつ足して数える | エスケープ後の文字列を位置指定で切る(`.Substring`。エスケープ列・サロゲートの対を分断する) |
| `watch` の表示幅(6.2) | コードポイントごとに 1 か 2 | `$cp -le 0x7F` なら 1、それ以外は 2、の合計 | `.Length`(基本多言語面の外の文字を 2 と数える)、`.PadRight`・`.PadLeft`、`-f` の幅指定、`Format-Table` |
| run ID の長さ(04 章 5.2) | (ASCII だけなので単位による差がない) | 範囲の比較のあと `.Length` | — |
| ファイルへ書く(ステート・変換の出力) | UTF-8 のバイト列(BOM なし) | `[System.Text.UTF8Encoding]::new($false, $true)` で符号化した `[byte[]]` を、`[System.IO.File]::WriteAllBytes`(または `FileStream`)で書く。UTF-8 のバイト数は `$enc.GetByteCount($s)` | `Set-Content`・`Out-File`・`>`(リダイレクト)・`Add-Content`(既定のエンコーディングと改行が、版と OS で違う。取得した文書では確認しておらず未確認。共通の設計判断 19)、`[System.IO.File]::WriteAllText($path, $text)`(エンコーディングを省略した形の既定は未確認)、`[System.Text.Encoding]::UTF8`(BOM の有無が未確認) |

注意:

- 「見た目の 1 文字」(書記素クラスタ。結合文字、絵文字の連結、国旗など)は、コードポイントとも一致しない。`conductor` は、書記素クラスタを単位にしない(表示幅は近似。6.2)。
- 文字列の比較(定義のキー、ジョブID、`--to` の値など)は、コードユニット列の厳密な一致(`[string]::Equals($a, $b, [System.StringComparison]::Ordinal)` または `-ceq`)で行う。Unicode の正規化(`.Normalize()`)や、大文字小文字の変換(`.ToLower()`・`.ToUpper()`・`.ToLowerInvariant()`・`.ToUpperInvariant()`)・大文字小文字を無視する比較(`OrdinalIgnoreCase`。拡張子の比較など、03 章が明示する箇所だけ)は、仕様が求める箇所でだけ行う。`-eq`・`-ne`・`-lt`・`-gt`・`-like`・`-match`・`-contains`・`-in`・`switch`(既定)は、文字列に対して、大文字小文字を区別しない(PowerShell の既定。04 章 4.0 の表)ので、識別子・オプション名・ID の比較に使わない。`string.Compare(a, b)`・`.CompareTo`・`.StartsWith(string)` の既定の形は、文化に依存する(取得した文書では確認しておらず、未確認)ので使わない。
- `.Trim()`・`.TrimEnd()`・`.Split()`(引数なし)・`[char]::IsWhiteSpace` は、Unicode の空白(U+3000、U+0085、U+00A0 など)に広く一致する(未確認)。ASCII の半角空白だけを扱う箇所(6.2 の行末の空白の除去など)では、引数を明示する(`.TrimEnd([char]0x20)`)。`-split`・`-replace` は、正規表現で、区切り文字列が正規表現として解釈される(未確認)ので、文字列の分割・置換には `.Split` の引数つきの形と `.Replace` を使う(ただし `.Replace(string, string)` の文化依存の挙動は未確認。`[System.StringComparison]::Ordinal` を取る形を使う)。
- 整数の最大値(リトライの待ち時間の飽和)は、`[bigint]` で計算し、`[uint64]::MaxValue` と比べて飽和させる([05-execution-flow.md](05-execution-flow.md))。`[uint64]` の演算が、オーバーフローで例外になるか、`[double]` に昇格するかは、取得した出典で確認しておらず未確認なので、頼らない。折り返さない。表示する数値は、`[uint64]`・`[int]` を 10 進の文字列にする(4.9)。

## 12. 必須の挙動と、実装に任せてよい点

必須:

- 4 章・5 章・6 章・8 章の全文(0 章の文言を含む)、`-v` の行の形式、出力先(標準出力か標準エラーか)と水準、行の改行(LF)。
- 言語の決め方(7 章。環境変数だけ)、エスケープの規則と切り詰め(9 章。単独のサロゲートを U+FFFD にする前処理を含む)、色の条件(6.3)、文字の単位と表示幅・埋め(6.2、11 章)。
- 日本語・英語ともに、固定の語とトークンの区別(2.1、7 章)。
- 書き込みの規則(1.2・1.3・1.5): `Write-Host`・`Write-Output`・`Write-Error` などと `[System.Console]::Out` を使わず、BOM なし UTF-8 の `StreamWriter`(`AutoFlush = $true`、`NewLine = "`n"`)を使うこと、`Write` のあとに `Flush` すること、標準出力の書き込みの失敗(`[bool]` の偽)を `watch` は 65、`convert` は 67 の終了コードに結びつけること、`SIGPIPE` の設定を変えないこと、終了の前に `Flush` を完了させること(1.4)。

実装に任せてよい点:

- 内部で「表示する内容」をどう表現するか、文言の表をどう保持するか(10 章は目安)。
- 言語・色の判定の遅延評価(観測できる差がない範囲で)。

## 13. 未確認

- 定義の誤りの `<原因>` は 03 章、変換の誤りの `<原因>` は 09 章が全件を規定する(この章の 8.3 に、変換の誤りの原因を `.psd1` に合わせて置き換えた点を挙げた。09 章の最終版と一致させる)。
- **PowerShell のホストの中で、`[System.Console]::OpenStandardOutput()` などが、.NET の通常のアプリと同じ挙動をすること**(すべて未確認。1.2.1・1.3.1 は .NET のドキュメントとソースからの引用)。ホスト自身の出力(エンジンの警告など)との順序。
- **UNIX 系で、閉じたパイプへの書き込みが、標準のコンソールのストリームでは検出できない**(ソースの記述で確認した。1.3.1)。この版は、その差(`watch`・`convert` が、閉じたパイプで 65・67 でなく 0 で終わりうる)を受け入れる。`FileStream` を標準出力のファイル記述子に直接被せる方法で検出できるかは未確認。Windows の閉じたパイプでの `IOException` の発生は未確認。Linux の `/dev/full` への出力が、65・67 になること(`ENOSPC` が例外になること)は、ソースの記述からの推測で、実機では未確認。
- Windows のコンソール・パイプ・ファイルへの出力の文字コード(`[System.Console]::OutputEncoding` を UTF-8 にしたうえで、UTF-8 のバイト列を標準ストリームに書いたとき、コンソールとリダイレクト先の両方で意図どおりに表示・保存されるか。`pwsh` が起動時にコードページを設定しているか。1.2.2 の 3)。`OutputEncoding` の復元が、`pwsh` の終了後のコンソールに影響しないこと。
- 標準入出力が閉じられている場合(`>&-`)の、`OpenStandardOutput()` の挙動(1.2.2)。
- `StreamWriter` を `Dispose` しないときの、バッファの流し直しと、ファイナライザの挙動(1.2.2、1.3.2 の 5)。
- 端末(疑似端末)で `[System.Console]::IsOutputRedirected` が偽になること(6.3)。Windows での `IsOutputRedirected` の実装。
- `Write-Host` が使う出力の経路(情報ストリームか、標準出力か)と、リダイレクトの挙動。`$ErrorView` の既定の表示(1.5)。
- 文化に依存する `string` の API の既定の挙動、`.ToString()` の既定の書式、PowerShell の文字列の展開が使う文化(4.9、11 章)。`.Trim` 系の挙動、`.Replace(string, string)` の文化依存(11 章)。いずれも、公式ドキュメントを取得しておらず未確認(この仕様は、それらを使わない書き方を規定する)。
- `switch`(既定)が、文字列の比較で大文字小文字を区別しないこと、複数の節に一致すると全部実行されること(04 章 4.0)。
- `[uint64]` の演算の、オーバーフローでの挙動(11 章)。
- `NO_COLOR` の値の扱い(未設定または空なら色あり、空でない値なら色なし)は規定どおりで、PowerShell の挙動に依存しない。
- 環境変数・引数に含まれる、不正な UTF-8 の置換のされ方(04 章 4.4.2)。
- `Send-ConductorError` などの関数名の動詞が、承認された動詞であること(10.1)。
- サポートする最小の PowerShell の版(実装時に確認する)で、1.2・1.3 の確認(閉じたパイプ、`/dev/full`、BOM なし UTF-8、LF)を、3 OS で行うこと。
