# 08. 出力と多言語化

この章は、Python で実装する `conductor` が出す文字列の全文、出力先、標準出力・標準エラーへの書き込みの規則(エンコーディング、`reconfigure`、フラッシュ、`BrokenPipeError`、終了前の出力の完了)、言語の決め方、`watch` の表、色、エスケープの規則(孤立サロゲートを含む)、表示幅と埋め、文字列の単位(コードポイント)の扱いを規定する。文言は、この章の表のとおりに(文字単位で)出力する。コマンドラインは [04-cli.md](04-cli.md)、終了コードとエラー型は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md)、定義の誤りの原因の文は [03-definition-format.md](03-definition-format.md)、ステートの形式は [06-state-file.md](06-state-file.md)、変換は [09-converter.md](09-converter.md)、子プロセスとシグナルは [07-process-and-signals.md](07-process-and-signals.md) を参照する。

実装は `conductor/report/` に置く。処理の流れは、「表示する内容」(種類と値を持つ型)を `report` に渡し、`report` がロケールと水準に従って文言に直して書く、という分担にする。

## 0. この版の文言の決め(まとめ)

文言は、この章の表のとおりに(文字単位で)出力する。この版が採った書式に関わる文言は次のとおり。

| 場所 | 文言 |
|------|------|
| 使い方(英語)の `convert` の行 | `conductor convert <file> --to <yaml\|json>` |
| 使い方(日本語)の `convert` の行 | `conductor convert <ファイル> --to <yaml\|json>` |
| 使い方の `--config` の既定(英語) | `(default: conductor.yaml in the current directory)` |
| 使い方の `--config` の既定(日本語) | `(省略時: 実行ディレクトリの conductor.yaml)` |
| 使い方の `--to`(英語) | `--to <yaml\|json>    target format of convert` |
| 使い方の `--to`(日本語) | `--to <yaml\|json>    convert の変換後の形式` |
| ステートの原因: 構文の誤り | `JSON として読めません` |
| ステートの原因: 書き出しの失敗 | `内容を JSON にできません` |
| 定義の誤りの種類 `retryNotTable`(英語 / 日本語) | `retry must be a map` / `retry はマップで書きます` |
| 定義の誤りの種類 `unsupportedFormat`(英語 / 日本語) | `unsupported definition file format` / `定義ファイルの形式に対応していません` |
| 例の中の定義・ステートのファイル名 | `c.yaml`、`c.yaml.state` など |

- ステートの原因 2 件は、ステートファイルが JSON であるため(10 章 7.2 に全件)。
- `retryNotTable` の識別子は内部の名前であり、利用者には見えない。文言は、YAML/JSON の語である「マップ(キーと値の対応)」で言う。
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
- ジョブ本体の標準出力・標準エラーは `conductor` が加工せず、そのまま流れる(子プロセスの `stdin`・`stdout`・`stderr` を `None` のまま起動し、子が親と同じファイル記述子を直接使う。07 章)。`conductor` の表示は標準エラーなので、標準出力にはジョブの出力だけが残る。
- 成功・スキップで `-v` なしのときは、表示が一度もなく、言語も色も判定されない(必要になった最初の時点まで判定を遅らせる)。この遅延は、観測できる差(環境変数の読み取り順)を生まないなら、実装に任せてよい。

### 1.1 水準ごとのイベント

| 水準 | イベント |
|------|----------|
| Error | 定義の誤り、`command` の誤り、使い方の誤り(+使い方全文)、ステートの誤り、定義との食い違い、`reset --all` の警告(`conductor: warning:`)、拒否、起動失敗、成功コードに含まれない 0、変換の誤り、標準出力に書けない、完了を判定できない、シグナルの準備の失敗、内部の誤り |
| Notice | リトライの前の知らせ(`conductor: retry:`) |
| Verbose | ジョブの結果、スキップ、遷移、中断、`validate` が通った |
| Output | `watch` の表、`watch` の空、`--help`、`--version`、`convert` の変換結果 |

### 1.2 標準出力・標準エラーのエンコーディング

#### 1.2.1 確認した挙動

Python の `sys.stdout`・`sys.stderr` は、`io.TextIOWrapper` である。文字コード(`encoding`)と、符号化できない文字の扱い(`errors`)は、起動時に環境から決まる。次を、Python 3.9.6・3.15.2(macOS)で確かめた。

| 確認した事項 | 結果 |
|--------------|------|
| 既定の標準出力・標準エラーの `errors` | `LC_ALL=C` で起動した 3.9.6(UTF-8 モードになる)では、標準出力は `utf-8`/`surrogateescape`、標準エラーは `utf-8`/`backslashreplace`(確認済み)。UTF-8 のロケールでの標準出力は `strict` が既定(Python の文書の記述。この実機の確認は、`LC_ALL=C` の場合だけ。未確認) |
| 標準エラーの行バッファ | `line_buffering` は、標準エラーが端末でなくても真(3.9 以降。3.9.6 で確認) |
| `PYTHONIOENCODING=ascii` で起動 | 標準出力・標準エラーの `encoding` が `ascii` になる(3.15.2)。`PYTHONUTF8=0 LC_ALL=C` でも `ascii`(3.9.6) |
| `sys.stdout.reconfigure(encoding="utf-8", errors="strict", newline="\n")` | 上の 2 つの環境のどちらでも成功し、`encoding` が `utf-8` になった。以後の日本語の出力は UTF-8 のバイト列になった |
| 標準出力が `errors="strict"` のとき、孤立サロゲートを含む文字列を `write` する | `UnicodeEncodeError`(`surrogates not allowed`)。文字列は全体が符号化されてから書かれるので、その `write` の分は 1 バイトも出ない(確認済み) |
| `errors="surrogateescape"` にして、U+DCFF を含む文字列を書く | バイト `FF` として書かれる(UTF-8 として不正なバイトが出力に入る) |
| `errors="replace"` にして、孤立サロゲートを含む文字列を書く | `?` になる |
| 標準入出力のファイル記述子を閉じて起動(`<&- >&- 2>&-`) | `sys.stdin`・`sys.stdout`・`sys.stderr` が `None` になる(確認済み: 3.15.2) |
| `sys.stdout` を `io.StringIO` に差し替えたとき | `reconfigure` を持たない(`hasattr` が偽)。テストの差し替えで起こる |

#### 1.2.2 規定

1. `main()`(10 章 6)は、ほかの処理より先に、`sys.stdout` と `sys.stderr` を次のとおり設定する。

   ```python
   sys.stdout.reconfigure(encoding="utf-8", errors="strict", newline="\n")
   sys.stderr.reconfigure(encoding="utf-8", errors="replace", newline="\n")
   ```

   - 文字コードを UTF-8 に固定する(ロケール、`PYTHONIOENCODING`、`PYTHONUTF8` に依存しない。上の確認のとおり、`reconfigure` はそれらを上書きできる)。
   - `newline="\n"` で、`"\n"` が書き出されるときの変換を止める。Windows の既定(`newline=None`)は、`"\n"` を `"\r\n"` に変換するが、この仕様の出力は、すべての OS で LF だけにする(ファイルやパイプにリダイレクトしたときも)。Windows での確認は未確認。
   - 標準出力は `strict`。孤立サロゲートが入り込んだら、黙って壊れた出力にせず、書き込みの失敗として扱う(1.3)。通常の流れでは、表示する文字列は 9 章のエスケープ(孤立サロゲートを U+FFFD にする)を通っているので、符号化の失敗は起こらない。標準エラーは `replace`(`conductor` の表示が、符号化の失敗で止まらないようにする。孤立サロゲートは `?` になる。通常の流れでは到達しない)。
   - `reconfigure` を持たない、または `None` の場合(テストでの差し替え、記述子が閉じられている場合)は、何もしない(`AttributeError`・`ValueError`・`OSError` を握りつぶす)。
2. `sys.stdout`・`sys.stderr` が `None`(記述子が閉じられている)の場合、その出力は「書き込みに失敗した」ものとして扱う(標準エラーは無視、標準出力は 1.3 の規則)。`sys.stdout.write` を `None` に対して呼ぶと `AttributeError` になるので、本番の出力口の実装は、`None` を先に判定する。
3. `print()` を使わない。書き込みは、`sys.stdout.write(text)`・`sys.stderr.write(text)` に、組み立て済みの 1 つの文字列を渡す(`print` の `sep`・`end`・`file` の既定の挙動に依存しない)。
4. ジョブ本体の出力は `conductor` を通らないので、この設定の対象外(子のエンコーディングは子が決める)。

Windows のコンソール:

- Python は Windows のコンソール(端末)への標準出力・標準エラーを、UTF-16 の API で書く(PEP 528。Python の文書の記述)。コンソールでは、`reconfigure(encoding="utf-8")` を呼んでも、日本語が正しく表示されることを意図する。実機(Windows)での確認は未確認。
- 標準出力をファイルやパイプにリダイレクトしたとき、既定の文字コードはコンソールのコードページ(例: 日本語環境では cp932)になりうる(Python の文書の記述)。1.2.2 の `reconfigure` で UTF-8 に固定するので、リダイレクトしたときも UTF-8 のバイト列が出る(未確認)。
- Windows のコンソールのフォントや、ANSI の制御列の可否は、この仕様では確認しない。色は常に使わない(6.3)。

### 1.3 書き込みの方式と、標準出力が閉じている・書けないとき

#### 1.3.1 確認した挙動

次を、Python 3.9.6 と 3.15.2(macOS)で確かめた。

| 確認した事項 | 結果 |
|--------------|------|
| `SIGPIPE` の既定の扱い | Python は起動時に `SIGPIPE` を無視する設定(`SIG_IGN`)にする(`signal.getsignal(signal.SIGPIPE)` が `Handlers.SIG_IGN`)。読み手が先に閉じたパイプへの書き込みは、信号でプロセスを殺さず、`BrokenPipeError`(`OSError` の子)になる |
| 5 MB を標準出力(パイプ)に書き、読み手が先に閉じる(`python x.py \| true`) | `write` が `BrokenPipeError`。捕捉しなければスタックトレースが標準エラーに出て、終了コード 1 |
| 小さな出力(`"hello\n"`)を、読み手が先に閉じたパイプに `write` して `sys.exit(3)`(`flush()` を呼ばない) | `write` は成功する(バッファに溜まるだけ)。インタプリタの終了時のフラッシュが失敗し、`Exception ignored in: <_io.TextIOWrapper name='<stdout>' ...>` と `BrokenPipeError` が標準エラーに出て、**終了コードが 120 になる**(`sys.exit(3)` の 3 は失われる) |
| 同じ状況で `write` のあとに `flush()` を呼び、`BrokenPipeError` を捕捉して `sys.exit(65)` | 捕捉できる。しかし、バッファに残ったデータの終了時のフラッシュがもう一度失敗し、**終了コードは 120 になる**(65 にならない) |
| 同じ状況で、捕捉したあと、`os.dup2(os.open(os.devnull, os.O_WRONLY), sys.stdout.fileno())` で標準出力の記述子を `/dev/null` に差し替えてから `sys.exit(65)` | 終了コードは 65 になり、メッセージも出ない |
| 標準エラーが閉じたパイプのとき、`sys.stderr.write(...)` を `try`/`except OSError` で包んで `sys.exit(4)` | 標準エラーも同じで、**終了コードが 120 になる**。標準エラーの記述子も、`/dev/null` への差し替えで 4 になる |
| 標準出力・標準エラーとも閉じたパイプのとき、`flush()` が失敗したストリームの記述子を `/dev/null` に差し替えてから `sys.exit(4)` | 終了コードは 4(3.9.6 と 3.15.2 の両方) |
| 書き込みの後、`os._exit(3)` を、`flush()` を呼ばずに呼ぶ | 標準出力がファイル(`> o.txt`)のとき、`o.txt` は 0 バイト(バッファの内容が失われる)。`flush()` を呼んでから `os._exit(3)` なら、出力は届く |
| 標準出力がパイプのとき、`flush()` を呼ばずに `sys.exit()` | 通常の終了では、インタプリタの終了時のフラッシュで出力が届く(Python の文書の記述。読み手が生きている場合。この実機の確認は、読み手が先に閉じた場合の上の行だけ) |

要するに、(1) `SIGPIPE` で静かに死ぬことはなく、`BrokenPipeError` になる。(2) 失敗した書き込みのデータが、ストリームのバッファに残ると、終了時のフラッシュがもう一度失敗して、終了コードを 120 にする(`conductor` の終了コードが壊れる)。(3) 記述子を `/dev/null` に差し替えれば、これを避けられる。(4) `os._exit` はバッファをフラッシュしない。

#### 1.3.2 規定

1. **書き込みは、出力口(`Deps` の標準出力・標準エラーの口)の 1 つの関数を通す**。本番の実装は、次の形にする(骨格。実装コードは規定しない)。

   ```python
   def write_all(stream: TextIO | None, text: str) -> None:
       """text を 1 回の write で書き、flush する。失敗は OSError 系の例外で呼び出し側に伝える。"""
   ```

   - `stream` が `None` のとき、`OSError`(または専用の例外)を送出する。
   - `stream.write(text)` のあと、必ず `stream.flush()` を呼ぶ。標準出力・標準エラーとも、行ごとではなく、1 回の表示(1 行、または 1 かたまり)につき 1 回の `write` と 1 回の `flush`。
   - 捕捉する例外は `OSError`(`BrokenPipeError`・`ConnectionResetError` などを含む)、`ValueError`(閉じたファイルへの書き込み、`I/O operation on closed file`)、`UnicodeError`(`UnicodeEncodeError` は `ValueError` の子)、`AttributeError`(`None` の場合の保険)。これらを、書き込みの失敗として扱う。**`BaseException` や `Exception` 全体を握りつぶさない**(想定外の失敗が、書き込みの失敗に化けないように)。
2. 水準ごとの書き込みの失敗の扱い:
   - `Output` 水準(標準出力): 失敗を、`Reporter.output()` が `OutputError`(10 章 8.3)として呼び出し元に送出する。`watch` は `cannot write to standard output` を標準エラーに出して 65、`convert` は同じ文を出して 67。
   - `--help` と `--version` の出力: 書き込みの失敗は無視して、終了コード 0 で終わる(失敗を報告しない。AC-513)。
   - `Error`・`Notice`・`Verbose` 水準(標準エラー): 失敗を握りつぶして続ける(本筋を止めない)。
3. **標準エラーの書き込みも、`flush()` まで完了してから次へ進む**。パイプ・ファイルの先でも、`conductor` の行が、そのあとに起動する子プロセスの出力より前に出るようにするため(子は同じ記述子を直接使うので、親の書き込みが親のバッファに残ったまま子を起動すると、順序が入れ替わりうる)。特に、リトライの知らせ(`Notice`)と、ジョブの起動前に出す行は、`flush()` を終えてから子を起動する(標準エラーは行バッファだが、この規則は、バッファの設定に依存しないために `flush()` を明示する)。
4. 書き込みは、すべて同じスレッド(メインスレッド)で、呼び出しの順に直列に行う(スレッドを使わない。02 章)。
5. **失敗した書き込みのデータを残したまま終わらない**。標準出力・標準エラーの `write` または `flush` が失敗したストリームは、終了時にもう一度フラッシュに失敗して、終了コードが 120 に化ける(上の確認)。したがって、書き込みの失敗を捕捉したときは、そのストリームの記述子(`stream.fileno()`)を、`os.dup2(os.open(os.devnull, os.O_WRONLY), fd)` で `os.devnull`(Windows では `nul`)に差し替える。差し替えの失敗(`OSError`、`ValueError`、`fileno()` が使えない場合)は無視する。この処理は、出力口の中で、失敗を捕捉した直後に 1 回行ってよいし、1.4 の終了手順の中でまとめて行ってもよい(どちらでも、終了コードが 120 にならないこと)。標準エラーの記述子の差し替えも同様に行う。
6. `SIGPIPE` の設定を変えない(Python が起動時に設定する `SIG_IGN` のままにする。`signal.signal(signal.SIGPIPE, signal.SIG_DFL)` を呼ぶと、書き込みで信号により静かに死んでしまい、終了コードの規定(65・67)が守れない)。子プロセスの `SIGPIPE` の扱いは 07 章に従う(`subprocess.Popen` の `restore_signals=True`(既定)は、子で `SIGPIPE` を既定の動作へ戻す。Python の文書の記述。実機での確認は 07 章)。
7. 標準出力・標準エラーのバッファリングに頼らない(`sys.stdout.flush()` を呼ばずに終了しない)。

### 1.4 終了前に出力を完了させる規定

プロセスの終了の手順(`__main__.main()` が行う。10 章 4.1・6 も参照):

1. `run(argv, deps)` が返した終了コード(`int`)を受け取る。
2. 標準出力、続いて標準エラーを、`flush()` する。`flush()` の失敗(`OSError`・`ValueError` など)を捕捉して無視するが、失敗したストリームは 1.3.2 の 5 のとおり、記述子を `os.devnull` に差し替える。`None` のストリームは飛ばす。
3. そのあとで、**終了コードを 0〜255 の `int` にそろえて**(10 章 4)、`sys.exit(code)` を呼ぶ(関数を返して終わる、でもよい。通常の終了の経路で、インタプリタが終了処理を行う)。
4. `os._exit(code)` を使ってよいのは、想定外の内部失敗(10 章 6)で、標準エラーに 1 行を書き、`flush()` を試みたあとの最後の手段としてだけ(`os._exit` は、バッファのフラッシュ、`finally`、`atexit` を飛ばす)。通常の終了では使わない。
5. シグナルのハンドラ(`signal.signal(...)`)は、終了する前に元へ戻す(07 章)。

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
$ conductor pipeline -v nightly --config c.yaml --run-id r1
[conductor] job=extract result=warning (exit code 1)
[conductor] transition: warning -> load (skipped: transform)
[conductor] job=transform skipped (skipped by a transition)
[conductor] job=load result=success (exit code 0)

$ conductor run -v a --config e.yaml        # success_exit_codes: [5] で、ジョブが 0 で終了
[conductor] job=a result=error (exit code 0, conductor exits with 1)
conductor: error: job 'a': exit code 0 is not listed in success_exit_codes

$ conductor pipeline -v p --config d.yaml   # ジョブが SIGKILL で終了
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
| `INTERNAL_ERROR`(想定外の内部失敗) | `conductor: error: internal error` | 70 |
| `SIGNAL_SETUP_FAILED` | `conductor: error: failed to set up signal handling` | 70 |
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

実出力(原因の文の例。`syntax` 以外の原因は 03 章の規定):

```text
$ conductor pipeline wrongname --config c.yaml        # LANG=C
conductor: error: c.yaml:3: pipeline name does not match the definition: 指定されたパイプライン名が、定義の `name` と一致しません
$ LANG=ja_JP.UTF-8 conductor validate --config nonexist.yaml
conductor: error: nonexist.yaml: 定義ファイルが存在しません: 指定の場所にファイルがありません
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
conductor: error: c.yaml.state: state file is corrupt: JSON として読めません; run `conductor reset` to discard the state
conductor: error: c.yaml.state: ステートファイルが壊れています: JSON として読めません; `conductor reset` でステートを初期化できます
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
| `REJECTED` | `conductor: error: job '<ID>': <REJECTED>: <理由>` | `cannot run this job now` | `このジョブは、いま実行できません` | 66 |
| 拒否の理由 `OutOfOrder` | | `earlier jobs have not finished (out of order)` | `前のジョブが終わっていません(順序外)` | |
| 拒否の理由 `SkipTarget` | | `skip target` | `スキップの対象です` | |
| 拒否の理由 `AfterEnd` | | `after @end` | `@end より後ろです` | |
| 拒否の理由 `AlreadyExecuted` | | `already executed` | `実行済みです` | |
| `CANNOT_START` | `conductor: error: job '<ID>': <文> '<コマンド名>'` | `cannot start command` | `コマンドを起動できません` | 127 |
| `SUCCESS_CODE_NOT_LISTED` | `conductor: error: job '<ID>': <文>` | `exit code 0 is not listed in success_exit_codes` | `終了コード 0 は、success_exit_codes に含まれていません` | 1 |
| `OUTPUT_FAILED` | `conductor: error: <文>`(場所なし) | `cannot write to standard output` | `標準出力に書き込めません` | 65(`watch`)/ 67(`convert`) |
| `COMPLETION_UNDETERMINED` | `conductor: error: <ステートのパス>: <文>` | `cannot determine whether the pipeline is complete` | `パイプラインが完了したかを判定できません` | 65 |

`CANNOT_START` の `<コマンド名>` は、`command` を解釈して得た実行ファイル名(解釈できなければ `command` の先頭の語。それもなければ空)を、名前用エスケープしたもの。

実出力:

```text
conductor: error: job 'load': cannot run this job now: earlier jobs have not finished (out of order)
conductor: error: job 'step3': cannot start command 'nonexistent_prog_x'
```

### 4.8 変換の誤り

表示: `conductor: error: <ファイル>[:<行>]: <種類の文>: <原因>[ (<AT_KEY>: <キーの位置>)]`。`<原因>` は、変換が作る日本語の固定の文で、言語を切り替えない(一覧は 10 章 7.3)。`AT_KEY` は英語 `at`、日本語 `場所`。キーの位置があるときだけ付く(` (at: job[2].transitions[0])`)。終了コードはすべて 67。

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
- 待機の前に必ず出す(1.3.2 の 3 のとおり、`flush()` を終えてから待機に入る)。`{最後の終了コード}` は、直前の試行の終了コード。終了コードがなければ 0 と表示する。
- 待ち時間の求め方は [05-execution-flow.md](05-execution-flow.md)(リトライ)に従う。`{秒}` と `{n}`・`{max}` は、`int` を 10 進の整数で文字列にしたもの(`str(n)`、または f 文字列の `{n}`)を使う。待ち時間は 64 ビット符号なし整数の上限 `2**64 - 1`(20 桁)で飽和させる([05-execution-flow.md](05-execution-flow.md))ので、`int` から `str` への変換の桁数の上限(Python 3.11 以降の `sys.set_int_max_str_digits` の既定の 4300 桁)には届かない。`float` を経由しない(`str(float)` は `1e+20` のような指数表記になりうる。`int` のまま扱う)。

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
| `EXIT_CODE` | `exit code` | `終了コード` |
| `CONDUCTOR_EXITS_WITH` | `conductor exits with` | `conductor の終了コード` |
| `KILLED_BY_SIGNAL` | `terminated by signal` | `シグナルで終了` |
| `COULD_NOT_START` | `could not start` | `起動できませんでした` |
| `INTERRUPTED_BY_SIGNAL` | `interrupted by signal` | `シグナルで中断` |
| `NOT_STARTED` | `not started` | `起動せず` |
| `SKIPPED_WORD`(スキップ行の語) | `skipped` | `スキップ` |
| `SKIPPED_LIST`(遷移行の `(…: ID, …)`) | `skipped` | `スキップ` |

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
- **表示幅**は、文字(コードポイント)ごとに、ASCII(U+0000〜U+007F)なら 1、それ以外は 2 として足す近似(全角を 2 とみなす)。実際の端末幅(結合文字、絵文字、ゼロ幅の文字など)は考慮しない。Python では `sum(1 if ord(ch) <= 0x7F else 2 for ch in s)` と同じ。`len(s)` はコードポイント数であり、全角文字を 2 と数えないので、表示幅には使わない。`unicodedata.east_asian_width` や、外部のライブラリ(`wcwidth` など)も使わない(結果が、この近似と異なりうるため)。
- 次の表の入力で、表示幅が決まる(この近似による。「実際の端末幅」ではない)。

  | 文字列 | コードポイント数(`len`) | 表示幅(この規定) | 備考 |
  |--------|-----------------------------|-----------------------|------|
  | `step1` | 5 | 5 | ASCII |
  | `ジョブ` | 3 | 6 | 全角 |
  | `😀`(U+1F600) | 1 | 2 | Python の `str` では 1 コードポイント |
  | `e` + U+0301(結合文字) | 2 | 3 | 結合文字を 2 と数える(端末の見た目の幅は 1) |
  | `🇯🇵`(U+1F1EF U+1F1F5) | 2 | 4 | 国旗は 2 コードポイント(端末の見た目の幅は 2) |
  | `日本語 abc` | 7 | 10 | |

- 見出し行: ジョブ見出しと状態見出しを、右に半角空白を足して列の幅にそろえ、補足見出しは埋めない。行末の空白は取らない(補足見出しは空でないので、実際には取る空白がない)。
- データ行: ジョブIDを埋め(列の表示幅にそろえる)、状態語を埋め(埋めは状態語の表示幅で数える。状態語は ASCII なので文字数と同じ)、補足を続ける。行全体の行末の空白を削る(補足が空なら、状態語のあとの埋めと区切りも削られる)。削る空白は ASCII の半角空白だけ(`line.rstrip(" ")`。引数なしの `str.rstrip()` は全角空白(U+3000)・タブ・改行なども削るので使わない。確認済み: Python 3.15.2、`"a　".rstrip()` は `"a"`、`"a　 ".rstrip(" ")` は `"a　"`)。
- **埋めは、自前で `" " * n` を足して行う**(`n = 列の幅 - その文字列の表示幅`)。`str.ljust`・`str.rjust`・`str.center`・書式指定の幅(`f"{s:<10}"`)は、コードポイント数で数え、全角文字を 1 と数えるので、表示幅の規則と合わない(確認済み: `len("あ".ljust(4))` は 4。表示幅の規定では `あ` は 2 なので、列の幅 4 にそろえるなら足す空白は 2)。
- `completed`: ステートの `completed` が真のときだけ、表の下に 1 行足す(`completed` / `完了`)。
- 表の全体を 1 つの文字列(行のリストを `"\n".join(lines)` で連結し、末尾にも `"\n"` を付ける)に作ってから、`write_all`(1.3)で 1 回で書く(書き込みの失敗を 1 回で検出するため)。

### 6.3 色

- 色を使う条件(最初に必要になったとき一度だけ判定): 環境変数 `NO_COLOR` が未設定または空、かつ標準出力が端末(`Deps` の `stdout_is_terminal()` が真)、かつ `sys.platform` が `"win32"` で始まらない。Windows は常に色なし(挙動をほかの OS の実装と揃えるための決定。Windows の端末の ANSI 対応の可否は、この規定のために確認していない)。`report` は `sys.platform` を参照せず、`Deps` の `stdout_is_terminal()` の本番の実装が、Windows で常に `False` を返す(OS 差を `conductor/runner/` の 1 か所にまとめる規則のため、この判定も `sys.platform` による分岐を `Deps` の実装の 1 か所に置く)。
- 端末の判定: `Deps` の端末判定の口(`stdout_is_terminal() -> bool`)の本番の実装は、`sys.stdout` が `None` でなく、かつ `sys.stdout.isatty()` が真(`ValueError`(閉じたファイル)が出たら偽)。`isatty()` は、標準出力が端末のときだけ真で、パイプ・ファイルでは偽(確認済み: Python 3.15.2・3.9.6、パイプとファイル)。端末(疑似端末を含む)で真になることは、この環境(疑似端末を作れない実行環境)では確認できなかった(未確認。実装時に、実際の端末で確かめる)。`os.isatty(1)` と `sys.stdout.isatty()` は、標準出力が差し替えられたとき(テスト)に結果が分かれうるので、`sys.stdout.isatty()` に統一する。`curses`、`colorama` などの外部のライブラリは使わない。
- 色は、状態語だけに、ANSI の SGR で付ける: `ESC[<番号>m<語>ESC[0m`(ESC は U+001B)のあとに埋めの空白。`success` = 32(緑)、`warning` = 33(黄)、`error` = 31(赤)、`skipped` = 36(シアン)、`pending` = 色なし(制御列を付けない)。
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

- 環境変数 `LC_ALL`, `LC_MESSAGES`, `LANG` を、この順に見る(`Deps` の `lookup_env`)。最初に空でない値を採り、その値が `ja` で始まれば(`value.startswith("ja")`)日本語、それ以外は英語。どれも未設定(`None`)または空なら英語。値は大文字小文字を区別する(`JA_JP` は英語)。値が孤立サロゲートを含む(非 UTF-8。04 章 4.4)ときは、その変数は未設定と同じ扱い(その変数は飛ばして、次の変数を見る)。
- 最初の空でない値で決まり、後の変数は見ない(`LC_ALL=C` かつ `LANG=ja_JP` は英語。`LC_ALL=` が空で `LANG=ja_JP.UTF-8` なら日本語)。
- 判定は、1 回の実行につき、最初に表示が必要になったときだけ行い、結果を保つ(単純な「未判定」の状態を持つ)。
- ロケールを判定しない固定の文: `--version`、内部の誤りの文、シグナルの準備の失敗の文(英語のみ)。
- 言語によらず英語: 状態語、`job=`・`result=`、`Run ID`、`conductor: error: `・`conductor: warning: `・`conductor: retry: `・`[conductor] ` の接頭辞、`--help` の中のオプション名。定義・ステート・変換の誤りの `<原因>` は日本語のまま。
- 必須: ロケールの判定に、標準ライブラリの `locale`(`locale.getlocale()`、`locale.setlocale()`、`locale.getdefaultlocale()` など)、`gettext`、OS のロケール API、ロケール判定のライブラリを使わず、上の 3 つの環境変数だけで決めること(`locale` モジュールの既定の値は、環境変数とは別の経路(C ライブラリの設定、`PYTHONUTF8`、`LC_CTYPE` の強制など)で決まりうるため。実機では `LC_ALL=C` のとき UTF-8 モードに切り替わることを確認した(1.2.1)ので、ロケールと環境変数の対応は単純ではない)。

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
  conductor convert <file> --to <yaml|json>
  conductor --help | --version

Options:
  -v, --verbose       print job results, skips and transitions to standard error (run, pipeline; validate prints one line)
  --config <path>     definition file (default: conductor.yaml in the current directory)
  --run-id <id>       run ID that separates the state file (run, pipeline, watch, reset; default: the environment variable
                      named by run_id_env in the definition, or CONDUCTOR_RUN_ID; an empty value means no run ID)
  --remove-on-complete
                      remove the state file when the run completes (run, pipeline; default: keep it, or the
                      remove_on_complete setting of the definition)
  --all               remove every state file of the definition, with or without a run ID (reset only)
  --to <yaml|json>    target format of convert
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
  conductor convert <ファイル> --to <yaml|json>
  conductor --help | --version

オプション:
  -v, --verbose       ジョブの結果、スキップ、遷移を標準エラーに表示する(run、pipeline。validate は 1 行)
  --config <パス>     定義ファイル(省略時: 実行ディレクトリの conductor.yaml)
  --run-id <ID>       ステートファイルを分ける run ID(run、pipeline、watch、reset。省略時: 定義の run_id_env の環境変数、
                      なければ CONDUCTOR_RUN_ID。空の値は run ID なし)
  --remove-on-complete
                      完了のときにステートファイルを消す(run、pipeline。省略時: 残す。定義の remove_on_complete に従う)
  --all               定義のステートファイルを、run ID の有無によらず、すべて消す(reset のみ)
  --to <yaml|json>    convert の変換後の形式
  -h, --help          この使い方を表示する
  -V, --version       版を表示する
```

- 全文は、Python のソースに、行の文字列のタプルとして持つ(三重引用符の文字列で書くと、先頭と末尾の改行、行末の空白、バックスラッシュの扱いを間違えやすいため)。「各行を `"\n".join(...)` で連結し、末尾にも `"\n"` を付ける」実装にして、受け入れテストで文字単位に比較する。
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
| 孤立サロゲート(U+D800〜U+DFFF) | 走査の前に、4.4.2 の `readable` で U+FFFD に置き換える(下記)。置き換え後の U+FFFD はそのまま出る |
| それ以外 | そのまま(日本語もそのまま) |

Python での実装の方針:

- **最初に、文字列全体を 04 章 4.4.2 の `readable(s)` に通す**。孤立サロゲート(`surrogateescape` で入った U+DC80〜U+DCFF、およびそれ以外の U+D800〜U+DFFF)が U+FFFD になる(不正なバイト列は最大の部分列ごとに 1 つ)。これにより、後続の処理と、標準出力・標準エラーの符号化(1.2)で、`UnicodeEncodeError` が起きない。`readable` を通さずに、孤立サロゲートの `hex(ord(ch))` を出すと、`\u{dcff}` のような表示になり、元のバイトとは無関係な値を見せてしまう。
- 走査は `for ch in s`(コードポイント単位。Python の `str` はサロゲートペアを持たない)で行い、`cp = ord(ch)` で値を得る。16 進は `format(cp, "x")`(小文字で、先頭の 0 が付かない)から `\u{` と `}` で囲んで作る(`hex(cp)` は `0x` が付くので、`hex(cp)[2:]` でもよい)。
- Cc の判定は、範囲(`cp < 0x20 or 0x7F <= cp <= 0x9F`)で書く(`unicodedata.category(ch) == "Cc"` でもよいが、範囲で十分。確認済み: U+0085 と U+007F は `Cc`)。書式・双方向の文字は、上の表の範囲を列挙する(カテゴリ `Cf` 全体ではない。表にない `Cf` の文字はそのまま出す。確認済み: U+200B と U+FEFF は `Cf`、U+2028 は `Zl`)。
- `str.isprintable()`、`str.isidentifier()`、`repr()`、`ascii()`、`str.encode(..., "backslashreplace")`、`unicode_escape` などは、規則の形(`\u{..}`、小文字の 16 進、先頭の 0 なし、対象とする範囲)と異なるので使わない。

切り詰め:

- 名前用(`escape_name`): 上限 64 文字。数えるのは、エスケープ後の文字数(コードポイント単位。エスケープ列(`\u{202e}` など)の各文字を 1 文字ずつ数える)。次の 1 文字(エスケープ列は 1 かたまり)を足すと 64 を超えるなら、そこで打ち切って末尾に `…`(U+2026)を 1 文字付ける。したがって出力は最大 65 文字。エスケープ列は途中で切らない。出力する文字列に、1 かたまりずつ足していく(エスケープ後の文字列 `out` に対して、`out[:64]` のように位置を指定して切らない。エスケープ列を分断するため)。
- パス用(`escape_path`): 上限なし(エスケープだけ)。
- 名前用を使うもの: 使い方の誤りの対象、ジョブID、遷移先、スキップのID、リトライのジョブID、パイプライン名、run ID、コマンド名。パス用を使うもの: パス(定義ファイル、ステートのパス、`reset --all` のファイル名、変換の入力ファイル、原因、キーの位置)。
- 非 UTF-8 などで文字に直せない部分は、先頭の `readable` が U+FFFD にしてから通す(U+FFFD は、制御文字でも書式文字でもないので、そのまま出る)。

期待値の表(入力 → 上限 → 出力):

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
| `😀` × 64(U+1F600。`len` は 64) | 64 | `😀` × 64(切らない) |
| `😀` × 65 | 64 | `😀` × 64 + `…` |
| `a<U+D800>b`(孤立した高位サロゲート) | 64 | `a<U+FFFD>b` |
| `a<U+DCFF>b`(`surrogateescape` で入った、不正なバイト `FF`) | 64 | `a<U+FFFD>b` |
| `a<U+DCE3><U+DC81>b`(不正なバイト列 `E3 81`) | 64 | `a<U+FFFD>b`(最大の部分列なので U+FFFD は 1 個) |

定義の検証・ステート・変換の各処理が、誤りの文を内部で作るときにも、同じ規則のエスケープを使う(エスケープだけの小さな部品を `conductor/report/` から公開して、それらの処理が使う。循環参照を避けるため、エスケープの関数を別の小さなモジュール、例えば `conductor/report/escape.py` に置く)。その場合、定義とパスのファイル名には 256 文字の上限を使う実装がありうるが、`conductor` の表示では、ファイル名をパス用エスケープ(切らない)で出す。したがって、表示されるファイル名は切り詰められない(必須)。

## 10. 実装方針の選定(Python)

| 項目 | 採用 | 理由・補足 |
|------|------|------------|
| メッセージの保持 | ロケールごとの読み取り専用の対応表(`types.MappingProxyType` で包んだ `dict`、または `typing.Final` を付けたモジュールの定数)を `conductor/report/` に持つ。キーは `enum.Enum`(または `Literal` の共用体)。全キーがすべてのロケールにあることを、単体テストで検査する | 表の全件を持てているかを、テストと `mypy --strict` で検出できる。ただし、ロケールごとの語順の違いがあるメッセージは、10.1 の関数で組み立てる |
| プレースホルダの置換 | 固定の書式文字列(`str.format` または f 文字列)に、値を引数として渡す。**入力に由来する文字列を、書式文字列そのものとして `str.format`・`%` に渡さない**(値に `{0.__class__}` や `%s` が入っていても、値は書式として解釈されないので安全だが、値を連結してから `.format` を呼ぶと解釈される)。値(ジョブIDなど)に `{n}` のような文字列が入っても、再度置換されないこと | 値の中の `{` `}` `%` が、書式として再解釈されないことを、設計で保証する |
| 文言の組み立て | 種類ごとの関数(例: `format_retry(loc, job_id, code, n, max_, seconds) -> str`)。ロケールによる分岐は関数の中 | 日本語・英語で語順が違う(リトライの知らせ)ため、単純な置換の表では足りない |
| 端末判定 | `sys.stdout.isatty()`(6.3)。`Deps` の口を通す | 外部ライブラリは、YAML の読み取りのほかには追加しない |
| 色 | 自前の SGR の定数 | 外部の色のライブラリは使わない(`colorama`、`rich` などは使わない)。`NO_COLOR` や端末判定の規則を、この規定に固定するため |
| 表示幅 | 自前(ASCII なら 1、それ以外は 2。コードポイント単位) | 仕様の近似に正確に合わせるため。`wcwidth` などは、結果が異なりうるので使わない |
| エスケープ | 自前(9 章) | 仕様の表に正確に合わせるため |
| 書き込み | `Deps` の標準出力・標準エラーの口(`write(text: str) -> None`。失敗は例外)。本番の実装は `sys.stdout`・`sys.stderr` への `write` と `flush` を包んだもの(1.3) | テストで差し替えるため(`conductor/cli/` の `Deps`) |
| ログ(`logging`) | 使わない | 出力の形・出力先・水準が、この章で固定されているため。`logging` の既定の形式と衝突させない |

### 10.1 骨格(例)

```python
from enum import Enum
from typing import Literal, Protocol

Locale = Literal["en", "ja"]
Level = Literal["error", "notice", "verbose", "output"]

class OutputError(Exception):
    """標準出力への書き込みの失敗(Output 水準)。呼び出し元が 65(watch)/ 67(convert)にする。"""

class TextSink(Protocol):
    # 失敗は OSError / ValueError の例外で伝える(1.3.2)。write のあとに flush まで行う。
    def write(self, text: str) -> None: ...

class Reporter(Protocol):
    def output(self, text: str) -> None: ...   # 失敗は OutputError を送出
    def error(self, text: str) -> None: ...    # 失敗は握りつぶす(送出しない)
    def flush(self) -> None: ...               # 終了前の完了(1.4)
```

`Reporter` は、ロケールと色の判定を、最初に必要になったときに行い、結果を保持する。判定に使う環境変数と端末判定は `Deps` から受け取る。`error()` は、`flush()` まで終えてから戻る(1.3.2 の 3)。呼び出し側は、例外の捕捉を要しない(内部で握りつぶす)。

## 11. 文字列の単位(コードポイント)

Python の `str` は、Unicode のコードポイントの列である。基本多言語面の外の文字(絵文字など。U+10000 以上)も、1 コードポイントで、`len()` は 1 を返す(確認済み: Python 3.15.2、`len("\U0001f600")` は 1)。UTF-16 のコードユニットの区別はない。`surrogateescape` により、`str` は孤立サロゲートを持てる(04 章 4.4)。`conductor` が文字を数える・切る・並べる場面の単位は次のとおり。

| 場面 | 単位 | 使う方法 | 使ってはならない方法 |
|------|------|----------|----------------------|
| 引数の連結の分解(04 章 4.1) | コードポイント | `for ch in s`、`s[i]`、`len(s)` | 書記素クラスタの分割(外部ライブラリを含む) |
| エスケープの走査(9 章) | コードポイント | `for ch in s` と `ord(ch)`(走査の前に `readable` で孤立サロゲートを除く) | `repr`・`ascii`・`encode("unicode_escape")` |
| 名前用の切り詰め(9 章) | コードポイント(エスケープ後) | 1 かたまりずつ足して数える | エスケープ後の文字列を位置指定で切る(エスケープ列を分断する) |
| `watch` の表示幅(6.2) | コードポイントごとに 1 か 2 | `1 if ord(ch) <= 0x7F else 2` の合計 | `len(s)`(全角を 1 と数える)、`ljust`・`rjust`、書式指定の幅 |
| run ID の長さ(04 章 5.2) | (ASCII だけなので単位による差がない) | `re.fullmatch`、または `len` | — |
| ファイルへ書く(ステート・変換の出力) | UTF-8 のバイト列 | `open(path, "w", encoding="utf-8", newline="\n")`、または `str.encode("utf-8")` した `bytes` を書く(`encoding` を省略すると、ロケール依存の文字コードになる。Windows では cp932 などになりうる) | `encoding` なしの `open(path, "w")`、`io.open` の既定 |

注意:

- 「見た目の 1 文字」(書記素クラスタ。結合文字、絵文字の連結、国旗など)は、コードポイントとも一致しない(確認済み: `len("é")` は 2、`len("\U0001F1EF\U0001F1F5")` は 2)。`conductor` は、書記素クラスタを単位にしない(表示幅は近似。6.2)。
- 文字列の比較(定義のキー、ジョブID、`--to` の値など)は、コードポイント列の厳密な一致(`==`)で行う。Unicode の正規化(NFC/NFD)や、大文字小文字の変換(`str.lower`・`str.casefold`・`str.upper`)はしない(確認済み: `"é" == "é"` は偽。正規化しない)。`locale.strcoll` は使わない(ロケール依存で、仕様の規則と合わない)。
- `str.strip()`・`str.split()`(引数なし)・`str.isspace()` は、Unicode の空白(U+3000、U+0085、U+00A0 など)に広く一致する。ASCII の半角空白だけを扱う箇所(6.2 の行末の空白の除去など)では、引数を明示する(`rstrip(" ")`)。
- 整数の最大値(リトライの待ち時間の飽和)は、`min(x, 2**64 - 1)` で飽和させる([05-execution-flow.md](05-execution-flow.md))。折り返さない。`bool` は `int` の子クラスなので、`True` を `1` として表示しない(`str(True)` は `"True"`)。表示する数値は、`int` であって `bool` でないことを、呼び出し側が保証する。

## 12. 必須の挙動と、実装に任せてよい点

必須:

- 4 章・5 章・6 章・8 章の全文(0 章の文言を含む)、`-v` の行の形式、出力先(標準出力か標準エラーか)と水準、行の改行(LF)。
- 言語の決め方(7 章)、エスケープの規則と切り詰め(9 章。孤立サロゲートを U+FFFD にする前処理を含む)、色の条件(6.3)、文字の単位と表示幅・埋め(6.2、11 章)。
- 日本語・英語ともに、固定の語とトークンの区別(2.1、7 章)。
- 書き込みの規則(1.2・1.3): 起動の最初に `reconfigure` で UTF-8 に固定すること、`write` のあとに `flush` すること、標準出力の書き込みの失敗を `watch` は 65、`convert` は 67 の終了コードに結びつけること、失敗したストリームの記述子を `os.devnull` に差し替えて、終了コードが 120 に化けないようにすること、`SIGPIPE` の設定を変えないこと、終了の前に `flush` を完了させること(1.4)。

実装に任せてよい点:

- 内部で「表示する内容」をどう表現するか、文言の表をどう保持するか(10 章は目安)。
- 言語・色の判定の遅延評価(観測できる差がない範囲で)。
- 1.3.2 の 5 の記述子の差し替えを、出力口の中で行うか、終了手順の中でまとめて行うか。
- `sys.exit()` を呼ぶか、`main()` から戻って終わるか(どちらでも、通常の終了処理が走ること)。

## 13. 未確認

- 定義の誤りの `<原因>` は 03 章、変換の誤りの `<原因>` は 09 章が全件を規定する(この章に未確認はない)。
- 1.2・1.3 の確認結果は、Python 3.9.6・3.15.2(macOS)で確かめた(3.10.7・3.14.10 では、この章の確認は行っていない)。Linux・Windows、およびサポートする最小の Python の版(実装時に確認する。見込みは 3.11)、3.11〜3.13 では、実装時に同じ確認(5 MB のパイプ書き込みと `BrokenPipeError`、終了コード 120 の再現、記述子の差し替え、`reconfigure`)を行う。
- Windows でのコンソール・パイプ・ファイルへの出力の文字コード(`reconfigure(encoding="utf-8", newline="\n")` が、コンソールと、リダイレクト先の両方で意図どおりに働くか)。Windows の `os.devnull`(`nul`)への `os.dup2` の差し替えが、終了コードの化けを防ぐか。
- 端末(疑似端末)で `sys.stdout.isatty()` が真になること(6.3)。実際の端末での確認が必要。
- `NO_COLOR` の値の扱い(未設定または空なら色あり、空でない値なら色なし)は規定どおりで、Python 側の挙動に依存しない。
- 非 UTF-8 を含むパス・名前の細かい置き換えは、04 章 4.4.2 の `readable` に従う(`bytes.decode("utf-8", "replace")` の最大の部分列の規則。確認は 04 章 4.4.2 の表の範囲)。
- Windows の引数・環境変数に含まれる、対になっていないサロゲート(04 章 4.4.1)。
- 標準入出力が閉じられている(`None`)場合に、`reconfigure` を呼ばず、書き込みを失敗として扱う動作(1.2.2)の、実機の `conductor` との照合(`None` の確認は 3.15.2 のみ)。
