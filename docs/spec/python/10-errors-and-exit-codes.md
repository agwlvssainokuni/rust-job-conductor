# 10. エラーと終了コード

この章は、Python で実装する `conductor` の終了コードの全件、複数の条件が当てはまるときの優先順位、ジョブの結果から終了コードへの対応、終了コードの整形、想定外の内部失敗の扱い、そして Python でのエラー型の設計を規定する。利用者から見えるものは「エラー分類」「終了コード」「メッセージ」で規定し、型は 8 章で規定する。メッセージの全文は [08-output-and-i18n.md](08-output-and-i18n.md)、引数の誤りは [04-cli.md](04-cli.md)、流れは [05-execution-flow.md](05-execution-flow.md)、シグナルは [07-process-and-signals.md](07-process-and-signals.md)、ステートは [06-state-file.md](06-state-file.md)、定義は [03-definition-format.md](03-definition-format.md)、変換は [09-converter.md](09-converter.md) を参照する。

## 1. 終了コードの一覧(全件)

| 終了コード | 名前 | 意味 | 主な発生箇所 |
|------------|------|------|--------------|
| 0 | OK | 成功。ジョブの成功・警告、スキップ(実行済み・遷移・`@end`)、`watch`・`reset`・`validate`・`convert` の成功、`--help`・`--version`。`reset --all` の警告(読めなくて残したファイル)も 0 | 全般 |
| 1 | UNLISTED_ZERO | ジョブが終了コード 0 で終わったが、`success_exit_codes` に 0 がない。または、ジョブの終了コードがエラーで、下位 8 ビットが 0 になる(例: 256)。または、起動して終わったのに終了コードが得られない(起こらない想定) | ジョブの結果の分類、終了コードの範囲合わせ |
| 1〜255 | (ジョブの値) | エラーになったジョブの終了コードを、そのまま。範囲外は下位 8 ビット(0 になるなら 1)。4 章 | 範囲合わせ |
| 64 | DEFINITION | 定義の誤り: ファイルがない・読めない・大きすぎる・文法・検証の誤り(パイプライン名の不一致、`run_id_env` や `remove_on_complete` の型・形の誤り、`retry` の誤りを含む)、拡張子が `.yaml`・`.yml`・`.json` のどれでもない、`command` の誤り(引用符が閉じていない・空)。`validate` の失敗も 64 | `run`, `pipeline`, `validate` |
| 65 | STATE | ステートの誤り: 読めない・大きすぎる・壊れている・版の不一致・定義との食い違い・中身の `run_id` とファイル名の食い違い・書き込みの失敗・削除(初期化)の失敗。`reset`・`reset --all` で消すのに失敗。`watch` が標準出力に書けない。`pipeline` が完了を判定できない | `run`, `pipeline`, `watch`, `reset` |
| 66 | REJECTED | 拒否(順序外など。`run` で前のジョブが終わっていない、スキップ対象、`@end` より後ろ、実行済み) | `run`, `pipeline` |
| 67 | CONVERSION | 変換の誤り。`convert` が標準出力に書けない | `convert` |
| 68 | USAGE | 使い方の誤り(知らないコマンド、引数の不足、余分な引数、知らないオプション、使えないオプション、値の欠落・誤り、知らないジョブID、run ID の値の誤り〔引数・環境変数とも〕、重複、`--all` と `--run-id` の併用、非 UTF-8 の引数) | 引数の解釈、run ID の決定、`watch`/`reset` |
| 70 | INTERNAL | 内部の誤り: 想定外の内部失敗(6 章)、シグナルの準備の失敗 | 全般、`run`/`pipeline` の準備 |
| 127 | START_FAILURE | ジョブを起動できなかった(実行ファイルがない・実行権限がない、など。終了の状態を取得できない場合も安全側で起動失敗扱い) | ジョブの結果の分類 |
| 128 + 番号 | SIGNAL_BASE | ジョブがシグナルで終了した、または `conductor` が中断された(例: SIGHUP = 129、SIGINT = 130、SIGQUIT = 131、SIGTERM = 143)。リトライの待機中の中断も含む | ジョブの結果の分類、中断 |

- 64〜68 と 70 は、ジョブ自身の終了コードと重なりうる。区別は、標準エラーの `conductor: error: ...` の行で行う(70 は固定の英語の文)。
- 終了コードの範囲(0〜255)への合わせ方は 4 章。
- Python の標準の終了コード(2: `argparse` の使い方の誤り、1: 捕捉されない例外、120: 標準出力のフラッシュの失敗。08 章 1.3)は、`conductor` の終了コードの規定に現れない。実装は、これらが外へ出ないようにする(`argparse` を使わない: 04 章 1.1。捕捉されない例外は 70: 6 章。120: 08 章 1.3.2 の 5)。

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
- `--help`・`--version`: 標準出力に書けなくても 0(書き込みの失敗は無視する。08 章 1.3.2)。標準出力に書けない失敗は、`watch` が 65、`convert` が 67、それ以外の経路で未捕捉になったものは 70(6 章)。
- `pipeline` の特有: 実行したジョブがエラー(または分類なし)なら、そのジョブの終了コードで終わる。ジョブのあいだに中断の合図があれば `128 + 番号`。完了なら 0。完了を判定できなければ 65。
- ステートの書き込みに失敗すると、ジョブの終了コードではなく 65 で終わり、「ジョブは実行されました」と表示する(少なくとも 1 回は実行された、という方針)。起動前の失敗(初期ステートの書き込みなど)は「ジョブは起動していません」。
- 想定外の内部失敗(6 章)は、どの時点でも 70 で終わる(上の順序の外)。

## 3. ジョブの結果から終了コードへ(分類)

最初に当てはまるものを採る。この判断は `conductor/flow/` の純粋な関数(ジョブの実行結果の値を入力に、分類と報告する終了コードを返す)として実装する([05-execution-flow.md](05-execution-flow.md))。

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

プロセスの終了コード = `exit_code_for(報告する終了コード, エラーか)`(4 章)。`run` は、この値で終わる。`pipeline` は、エラーまたは分類なしのジョブで止まるとき同じ式で、完了なら 0。

注: 「起動した」は、子プロセスを作れたこと(`subprocess.Popen(...)` の呼び出しが例外を送出せずに戻り、`Popen` のオブジェクトが得られること)を指す。「中断」は、`conductor` がシグナル(07 章)を受け取り、中断の合図が立ったこと。ジョブがシグナルで終了した(順 5)のと区別する。

Python での「終了コードの取り出し」(詳細は 07 章):

- 子の終了は、`Popen.wait()` または `Popen.poll()` が返す `returncode`(`int`)で受け取る。POSIX では、子が自分で終了したときは 0 以上の終了コード、子がシグナルで終了したときは**負の値 `-n`**(`n` はシグナル番号)になる(確認済み: Python 3.15.2、macOS。`SIGKILL` で殺した子の `returncode` は `-9`。`sys.exit(259)` で終わった子は、OS が下位 8 ビットに切り詰めるので `3`、`os._exit(255)` は `255`)。したがって、順 5 は `returncode < 0` のとき(シグナル番号は `-returncode`)、順 7 以降は `returncode >= 0` のとき、と判定する。順 6(終了コードがない)は、`wait()` が戻ったあとの `returncode` が `None` のとき(通常は起こらない)。
- Windows では、`returncode` は負にならず、符号なし 32 ビットの値(例外コード `0xC0000005` = 3221225477 など)になりうる(Python の文書の記述。Windows の実機では未確認)。この値は、そのまま `exit_code_for` に渡す(下位 8 ビットを取る: 4 章)。Windows では、ジョブの終了コードが 255 を超える値(例: 256)になりうるので、4 章の範囲合わせの対象になる。
- 起動の失敗: `Popen` は、子で `exec` に失敗すると、親で `OSError` の子クラスを送出する(確認済み: Python 3.15.2、macOS。実行ファイルがない `["nonexistent_prog_x"]` は `FileNotFoundError`(`errno` 2)、実行権限のないファイル `["./noexec.sh"]` と、ディレクトリ `["/tmp"]` は `PermissionError`(`errno` 13)、存在しないディレクトリを含むパス `["./nodir/x"]` は `FileNotFoundError`、空文字 `[""]` は `PermissionError`)。順 4(起動失敗)として扱う。`OSError` の種類(`FileNotFoundError`、`PermissionError`、`NotADirectoryError`、そのほかの `OSError`)は、利用者には区別せず、すべて `cannot start command`(127)にまとめる。**捕捉するのは `OSError` だけ**とし、`Exception` 全体や `BaseException` は捕捉しない(想定外の失敗を、起動失敗に化けさせないため)。
- 起動に失敗した `Popen` は、プロセスを作らないので、`wait()` を呼ぶ対象がない(`Popen(...)` が例外を送出するので、オブジェクトが得られない)。重複して分類しない(起動失敗の `OSError` の捕捉と、終了の待機は、1 つの `try` の中で、後者を前者の成功後にだけ行う)。

## 4. 終了コードの範囲合わせ

`exit_code_for(reported: int, is_error: bool) -> int`:

| 入力 | 出力 |
|------|------|
| `0 <= reported <= 255` | `reported` |
| 範囲外 | `reported` の下位 8 ビット(`reported & 0xFF`。負の値は 2 の補数表現の下位 8 ビット)。ただし、その結果が 0 で `is_error` が真なら 1(エラーを成功に見せない) |

Python の `int` は任意精度で、`&` は負の値も、無限の長さの 2 の補数として扱う。そのため、`reported & 0xFF` は、負の値でも、32 ビット・64 ビットの範囲を超える値でも、下位 8 ビットを返す(確認済み: Python 3.15.2。`-1 & 0xFF` は 255、`-2 & 0xFF` は 254、`259 & 0xFF` は 3、`(2**40 + 7) & 0xFF` は 7、`(-2**40) & 0xFF` は 0、`0xC0000005 & 0xFF` は 5、符号つき 32 ビット値にした `(0xC0000005 - 2**32) & 0xFF` も 5)。したがって、32 ビットへの前処理(`| 0` に相当するもの)は不要で、符号つき・符号なしのどちらで受け取っても結果が同じになる。例: 259 → 3、511 → 255、-1 → 255、-2 → 254、`0xC0000005` → 5。256・512・`-256`・`2**40` は、エラーなら 1、エラーでなければ 0。

- `reported` は、`int` であって `bool` でないこと(`bool` は `int` の子クラスで、`True & 0xFF` は 1 になる。型ヒントと `mypy --strict` で、呼び出し側が `int` を渡すことを保証する。`isinstance(x, bool)` の検査を入れてもよい)。`float` を渡さない(`1.0 & 0xFF` は `TypeError`)。
- `signal_exit_code(n: int) -> int` = `exit_code_for(128 + n, True)`(Python の `int` はあふれないので、加算を保護する必要はない)。`n` が負のときの規定はない(呼び出し側が正の番号だけを渡す)。15 → 143、2 → 130、1 → 129、3 → 131、300 → 172(428 の下位 8 ビット)。
- `-v` の結果行には、元の値(`exit code <C>`)と、`conductor` の終了コードが異なるとき(`result=error` のみ)`, conductor exits with <E>` を示す(08 章 2.1)。

### 4.1 プロセスの終了(`sys.exit` に渡せる値)

`__main__.main()` は、`conductor/cli/` の `run` が返した `int`(すでに 0〜255 に合わせたもの)を、08 章 1.4 の手順で出力を完了させたあと、`sys.exit(code)` に渡す。**終了コードは、必ず 0〜255 の `int`(`bool` でない)にしてから渡す**。次を、Python 3.15.2(macOS)で確かめた(`sys.exit(...)` を子プロセスで実行して、親が `returncode` を読んだ。3.9.6 では、0・255 の結果だけ確認した)。

| `sys.exit` に渡した値 | 結果(終了コード) |
|------------------------|--------------------|
| `0`、`255` | 0、255 |
| `256`、`259`、`300` | 0、3、44(OS が下位 8 ビットに切り詰める) |
| `-1`、`-2` | 255、254 |
| `2**31` | 0 |
| `2**32 + 5` | 5 |
| `2**63`、`2**64 + 3` | 255 |
| `-2**31` | 0 |
| `None` | 0(成功扱い) |
| `True` | 1(`bool` は `1` として扱われる) |
| `1.5`、`'abc'`、`[1]` | 1。値(`1.5`、`abc`、`[1]`)が標準エラーに出る(`sys.exit` に整数以外を渡すと、その文字列表現が標準エラーに出て、終了コード 1 になる) |

- 範囲外の整数は、OS(またはインタプリタの `C` の `long`・`int` への変換)の結果になり、`2**63` のような値で挙動が不規則になる(上の表の 255 など。3.15.2 のみの確認で、ほかの版・OS では未確認)。この仕様では、`exit_code_for` で自分で 0〜255 に合わせ、OS・インタプリタの切り詰めに頼らない。
- `None` を渡すと 0 になる。終了コードを持たない経路を、`sys.exit(None)` で成功にしてしまわないよう、`run` の戻り値の型は `int`(`None` を許さない。`mypy --strict` で検査する)。
- `os._exit(code)` は、`2**31` 以上の値で `OverflowError` を送出する(確認済み: 3.15.2)。0〜255 の整数を渡せば起こらない。
- `sys.exit()` は `SystemExit` を送出するので、`finally` と `atexit` が走る(`os._exit` との違い)。通常の終了は `sys.exit` を使い、`os._exit` は 6 章の最後の手段にだけ使う。
- Windows でこの切り詰めが行われるか(`sys.exit(256)` の終了コード)は未確認。上のとおり、`exit_code_for` で合わせた 0〜255 の値だけを渡すので、影響しない。

## 5. シグナルと中断

- UNIX 系: `SIGHUP`(1)・`SIGINT`(2)・`SIGQUIT`(3)・`SIGTERM`(15)を、`signal.signal(signum, handler)` で受け取り、ハンドラは、最初の番号を中断の合図(フラグ)に記録するだけにする(ハンドラの中で、複雑な処理・出力・ロックの取得をしない)。ほぼ同時に複数届くと、番号の小さい方が「最初」になりうる(順序は実装依存。必須ではない)。シグナル名と番号の対応は、`signal.Signals`(例: `signal.SIGINT.value` は 2)から得られるが、表を自前で持つ規定は 07 章に従う。
- Windows: Ctrl+C(番号 2。`signal.SIGINT`)と Ctrl+Break(`signal.SIGBREAK`。07 章)。
- `signal.signal` は、メインスレッドからだけ呼べる(メインスレッド以外から呼ぶと `ValueError`)。`conductor` はスレッドを使わず、メインスレッドで呼ぶ。終了時に、元のハンドラに戻す(07 章)。
- `run`・`pipeline` だけがシグナルを準備する。`watch`・`reset`・`validate`・`convert` は、シグナルを準備しない。ただし、Python は起動時に、`SIGINT` に、`KeyboardInterrupt` を送出する既定のハンドラ(`signal.default_int_handler`)を登録する。そのため、これらのコマンドの実行中の Ctrl+C は、`KeyboardInterrupt` になる(OS の既定の動作ではない)。`KeyboardInterrupt` の扱いは 6 章。
- 起動済みのジョブが中断されると、`128 + 番号`、エラーとして記録する。起動前、またはジョブのあいだ、リトライの待機中に中断されたときは、エラーとして記録せず、再開位置を記録して `128 + 番号` で終わる。
- 準備の失敗は、`conductor: error: failed to set up signal handling` を出して 70。同じプロセスで 2 回目の登録になる「すでに登録済み」は、成功と同じに扱う(実装が複数回呼ぶ構造のときだけ関係する)。`signal.signal` は、メインスレッド以外からの呼び出し(`ValueError`)、範囲外のシグナル番号(`ValueError`)、`SIGKILL` への登録(`OSError`、`errno` 22)などで失敗しうる(確認済み: Python 3.15.2、macOS。Python の起動時の `SIGINT` のハンドラは `signal.default_int_handler` だった)ので、準備の失敗は、`Deps` のシグナルの口(`Protocol`)の実装が、準備できなかったときに例外(`ValueError`・`OSError`)として返し、`run` がそれを準備の失敗(70)にする(07 章)。
- 詳細は [07-process-and-signals.md](07-process-and-signals.md)。

## 6. 想定外の内部失敗(終了コード 70)

実装のバグなどで、設計が想定していない失敗(Python では、規定されたエラー型以外の例外、`KeyboardInterrupt` と `SystemExit` 以外の `BaseException`(`GeneratorExit` など)、`assert` の失敗、`RecursionError`、`MemoryError`、`finally` や `atexit` の中の例外など)が起きたときの規定。

### 6.1 必須の挙動

1. 入口 `conductor/__main__.py` の `main()` は、最初の実行の文で、`sys.excepthook` に、次の 2 の関数を登録する(ほかの処理より前)。`main()` の本体は、`try`/`except` で包む。

   ```python
   def main() -> None:
       sys.excepthook = _excepthook          # 最初の文
       ...
       try:
           code = run(sys.argv[1:], production_deps())
       except KeyboardInterrupt:
           code = 130                         # 6.2
       except SystemExit:
           raise                              # 6.3
       except BaseException:
           code = _report_internal_error()    # 6.1 の 2。70 を返す
       sys.exit(_finish(code))                # 08 章 1.4(出力の完了)
   ```

   (骨格。実装コードは規定しない。`_finish` は、出力を完了させ、終了コードを 0〜255 の `int` で返す。`_finish` の中の例外も、同じ `except` の規則(`KeyboardInterrupt` → 130、`SystemExit` は通す、そのほかの `BaseException` → 70)で扱う。`try` を、`_finish` を含めて 1 つにまとめるか、二重にするかは、実装に任せる。)

2. `_report_internal_error()` と `_excepthook` は、同じ 1 つの関数(または、後者が前者を呼ぶ)に集める。この関数は、標準エラーに、固定の英語の文を 1 行書く(ロケールを判定しない)。

   ```text
   conductor: error: internal error
   ```

   末尾は改行。書き込みに失敗しても無視して続ける(08 章 1.3.2 の `write_all`、`flush` まで行い、失敗したストリームは記述子を `os.devnull` に差し替える)。この関数は、1 回だけ動く(2 回目以降の呼び出しは、何もしない。例外の連鎖で行が重複して出ないように、モジュールの変数のフラグで止める)。
3. `sys.excepthook` は、捕捉されなかった例外(`SystemExit` を除く)が、インタプリタの最上位に届いたときに呼ばれる。ハンドラが戻ると、インタプリタは終了コード 1 で終わる(確認済み: 下の表)ので、`_excepthook` は、**自分で 70 で終わる処理を行う**: 1 行を書き、`flush` し、`os._exit(70)`(または `sys.exit(70)`。どちらも 70 になることを確認した)を呼ぶ。`_excepthook` 自身が例外を送出すると、インタプリタは `Error in sys.excepthook:` と元の例外のトレースバックを標準エラーに出し、終了コード 1 で終わる(確認済み)ので、`_excepthook` の中は、例外を送出しない書き方にする(内部で `try`/`except BaseException` を使って握りつぶす)。
4. スタックトレースや失敗の詳細を、標準出力へ出してはならない(標準出力はジョブの出力と `watch`・`convert` の結果のためにある)。詳細(例外のトレースバック)を標準エラーへ出すかは、実装に任せてよい(出す場合も上の 1 行を、必ず含める。1 行目に出す。Python 既定のトレースバック表示 `traceback.print_exception` を、その 1 行のあとに足してよい)。
5. 外部の入力(引数、環境変数、ファイル、シグナル、ジョブの出力)に対して、想定外の失敗が起きない作りにする。これらは、すべて、規定された誤り(64〜68 など)として扱う。たとえば、リストの範囲外の参照(`IndexError`)、`None` の属性参照(`AttributeError`)、辞書の欠けたキー(`KeyError`)、型の誤り(`TypeError`)、ファイルの文字コードの誤り(`UnicodeDecodeError`)、整数への変換の誤り(`ValueError`)、深い入れ子の再帰(`RecursionError`)が、入力から起きないようにする。入力の検査に `assert` を使わない(`python -O` では `assert` 文が取り除かれ、検査が消える)。

### 6.2 `KeyboardInterrupt` の扱い

`KeyboardInterrupt` は、`Exception` ではなく `BaseException` の子クラスで、Python の既定の `SIGINT` ハンドラが、メインスレッドで送出する。この版の決定:

- `run`・`pipeline` では、シグナルの準備のあとは、`SIGINT` のハンドラが自前になるので、`KeyboardInterrupt` は送出されない。準備の前(引数の解釈、定義の読み込みなど)と、ハンドラを元へ戻したあとに届いた `SIGINT` は、`KeyboardInterrupt` になりうる。
- `watch`・`reset`・`validate`・`convert` でも、実行中の Ctrl+C は `KeyboardInterrupt` になる(5 章)。
- 決定: `main()` は、`KeyboardInterrupt` を捕捉したら、メッセージを出さず、終了コード 130(128 + 2)で終わる(`internal error` にしない。70 にしない)。`_finish`(出力の完了)の中で届いた場合も同じ。ステートファイルは、一時ファイルへ書いて置き換える方式(06 章)なので、途中の内容が残らない。
- 確認した挙動(Python 3.15.2・3.9.6、macOS): 捕捉されない `KeyboardInterrupt` は、インタプリタが、トレースバックを標準エラーに出したあと、自身に `SIGINT` を送って終わる(終了状態は「信号 2 で死亡」。`subprocess` の `returncode` は `-2`、シェルの `$?` は 130)。`sys.excepthook` を登録していても、`KeyboardInterrupt` ではそれが呼ばれたあとに、同じく `-2` で終わった。`except KeyboardInterrupt: sys.exit(130)` で捕捉すると、終了コード 130 の通常の終了になった(`returncode` は 130。トレースバックなし)。
- したがって、この版は、`except KeyboardInterrupt` で捕捉して `sys.exit(130)` に相当する終了にする(信号で死亡する形ではなく、終了コード 130 の通常の終了)。シェルから見た終了コードは同じ 130。`signal.signal(signal.SIGINT, signal.SIG_DFL)` で、OS の既定の動作(信号で死亡)に戻す方式は、この版では採らない(確認した範囲では、`SIG_DFL` に戻して自分に `SIGINT` を送ると `-2` で終わった。それでも、決定は上のとおり)。

### 6.3 `SystemExit` の扱い

- `SystemExit` は、`sys.exit()` が送出する。`conductor` が自分で終わるときの経路(`main()` の最後の `sys.exit(code)`)であり、「想定外の失敗」ではない。`main()` の `except BaseException` は、`SystemExit` を先に捕捉して**再送出する**(70 にしない。コードの値を変えない)。`sys.excepthook` は、`SystemExit` では呼ばれない(確認済み: `sys.excepthook` を登録して `sys.exit(5)` したとき、フックは呼ばれず、終了コード 5)。
- ライブラリ(YAML の読み取りなど)が `sys.exit` を呼ぶことは想定しない。呼ばれた場合は、その終了コードで終わる(想定外。実装は、ライブラリの呼び出しを、`sys.exit` を呼ばない使い方に限る)。

### 6.4 捕捉できないもの、確認した挙動

確認した挙動(Python 3.15.2・3.9.6、macOS。いずれも子プロセスで `python -c` を実行して、終了コードと標準エラーを読んだ):

| 確認 | 結果 |
|------|------|
| `sys.excepthook` を登録し、ハンドラが 1 行を書いて戻る。`raise RuntimeError('x')` | ハンドラが呼ばれ、終了コード **1**(ハンドラが戻ると 1) |
| ハンドラが 1 行を書き、`sys.exit(70)` を呼ぶ | 終了コード 70。標準エラーはその 1 行だけ(トレースバックなし) |
| ハンドラが 1 行を書き、`flush` して `os._exit(70)` を呼ぶ | 終了コード 70。1 行だけ |
| ハンドラ自身が例外を送出する | `Error in sys.excepthook:` と元の例外のトレースバックが標準エラーに出て、終了コード 1 |
| `sys.exit(5)`(`SystemExit`)で、ハンドラを登録 | ハンドラは呼ばれず、終了コード 5 |
| `main()` 相当の `try`/`except BaseException` で、`GeneratorExit`、`RecursionError`(無限再帰)、`MemoryError`(明示的に `raise`)を捕捉して `sys.exit(70)` | 捕捉でき、70 |
| `except KeyboardInterrupt: sys.exit(130)` | 130 |
| `__del__` の中の例外(送出されても伝わらない、`unraisable`) | 標準エラーに `Exception ignored ...` が出て、終了コードは 0(失敗にならない)。`sys.unraisablehook` を登録すると、そのフックが呼ばれ、終了コードは 0 のまま |

- 「`unraisable`」(`__del__`、ガベージコレクタ中の例外など、送出先がない例外)は、プロセスの終了コードに影響しない。この版は、`sys.unraisablehook` を登録しない(Python の既定の表示に任せる)。`conductor` は、`__del__` を持つ型を、設計上、作らない(リソースの解放は `with` と `try`/`finally` で行う)ので、通常の流れでは起こらない。
- `sys.excepthook` と `try`/`except BaseException` で捕捉できないもの(Python の制約。実装時に確認する): インタプリタの致命的な異常終了(セグメンテーション違反、メモリ不足で OS に殺される、`os.abort`)、`kill -9`、C 拡張の中のクラッシュは、ハンドラが動かず、終了コードは 70 にならない(信号による死亡)。この場合は仕様の対象外とする。ただし、`RecursionError`(スタックの溢れ)は例外として捕捉できる(上の表)。`MemoryError` は、実際のメモリ不足では捕捉できても、その後の処理(標準エラーへの書き込み)が失敗しうる(明示的な `raise MemoryError` の確認だけで、実際のメモリ不足は未確認)。
- スレッド: この版は `threading` を使わない(02 章)。スレッドの中の未捕捉の例外は、`sys.excepthook` ではなく `threading.excepthook` に届くが、スレッドを使わないので登録しない。
- `faulthandler` は有効にしない(既定は無効。`python -X faulthandler` や `PYTHONFAULTHANDLER` で有効にされた場合の挙動は、この仕様の対象外)。

ジョブの実行中に想定外の失敗が起きたとき:

- 子プロセスの終了を待たずにプロセスが終わり、ジョブが孤児として残りうる(待つことは必須ではない)。
- ステートは直前の完全な内容のままであること(書き込みは、一時ファイルに書いて `os.replace` で置き換える形で、途中の内容が見えない形で行う。[06-state-file.md](06-state-file.md))。次の実行は、そのジョブを再実行しうる(少なくとも 1 回の方針)。

実装に任せてよい点:

- 失敗の詳細の保存や標準エラーへの追加の出力。ただし、`conductor: error: internal error` の行と 70 は必須。
- 内部の失敗を起こすための専用のコマンドや、テスト用の差し替えの口は、規定しない(製品の実行ファイルに専用のコマンドはない)。テストでは、`Deps` の差し替えで例外を投げる実装を渡し、入口に相当する関数の戻り値が 70 で、標準出力が空であることを確認してよい。`sys.excepthook` 自体のテストは、別プロセスで入口を起動して、終了コードと標準エラーを確かめる(受け入れテスト)。

## 7. エラー分類の一覧

内部の型の分け方は 8 章で規定する。利用者から見える区分として、次の分類と対応を守る。

### 7.1 定義の誤り(終了コード 64)

31 種。種類の識別子と、英語・日本語の文は [08-output-and-i18n.md](08-output-and-i18n.md) 4.3 に全件ある(03 章と同じ)。

識別子: `unsupportedFormat`, `fileMissing`, `unreadable`, `tooLarge`, `syntax`, `typeMismatch`, `unknownKey`, `versionMissing`, `versionUnsupported`, `unknownTarget`, `errorTransition`, `backwardOrSelfTarget`, `overlappingExitCodes`, `zeroAsWarning`, `pipelineNameMismatch`, `duplicateJobId`, `invalidJobId`, `noJobs`, `emptyField`, `duplicateTransition`, `invalidEnvName`, `retryNotTable`, `retryMissingKey`, `retryExitCodesEmpty`, `retryExitCodeInvalid`, `retryExitCodesOverlap`, `retryMaxInvalid`, `retryDurationInvalid`, `retryBackoffIncomplete`, `retryMultiplierInvalid`, `retryMaxIntervalTooSmall`。すべて終了コード 64。

`unsupportedFormat` は、定義ファイルの拡張子が `.yaml`・`.yml`・`.json` のどれでもないとき(拡張子の比較は大文字小文字を区別しない)の種類で、原因の文は `拡張子は .yaml、.yml、.json のいずれかで書きます`、行はない(03 章)。

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

読み込みの検査の順序と、各原因の判定条件は [06-state-file.md](06-state-file.md)(JSON 版)に従う。複数の原因が当てはまるときにどれを出すかの順序は、この章では未確認(06 章に従う)。

### 7.3 変換の誤り(終了コード 67)

6 種: `fileMissing`, `unreadable`, `tooLarge`, `syntax`, `unrepresentable`, `rootNotMap`。持つ情報は、ファイル、行(あれば)、キーの位置(あれば)、種類、原因。表示は 08 章 4.8。

原因の文(固定の日本語。言語を切り替えない)。変換の規則と、どの場面で使うかは [09-converter.md](09-converter.md)(YAML と JSON の相互変換)が規定する。次の表が全件で、文言は 09 章と完全に同じにする。

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
- 「キーが長すぎます(上限 1024 文字)」の長さは、コードポイント単位で数える(Python では `len(key)`。08 章 11 章)。

キーの位置の形式: キーは `.` で連結(各キーは 64 文字で切る)、配列の添字は `[i]`、文字列でないキーは `(非文字列キー)`。例: `job[2].transitions[0]`。

種類と原因の組は、上の表のとおり(09 章と同じ)。

### 7.4 使い方の誤り(終了コード 68)

8 種: `UnknownCommand`, `MissingArgument`, `UnexpectedArgument`, `UnknownOption`, `OptionNotAllowed`, `MissingValue`, `InvalidValue`, `UnknownJob`。文言は 08 章 4.2。誤りは、種類と、対象(あれば)を持つ。対象は、誤りの対象の語(知らないコマンド、オプション名、ジョブID、`run ID (environment variable)` など)。入力の値は、それ以外には含めない(`--run-id` の値、環境変数の値は含めない)。

発生条件:

| 発生条件 | 種類 | 対象 |
|----------|------|------|
| 位置引数なし | `MissingArgument` | なし |
| コマンド名が未知(空文字も) | `UnknownCommand` | その語 |
| コマンド名が非 UTF-8(孤立サロゲートを含む) | `InvalidValue` | 読める形にした語(04 章 4.4) |
| 文法にないオプション、`--bogus=x` | `UnknownOption` | `--bogus`、`-x` など |
| そのコマンドで使えないオプション | `OptionNotAllowed` | `-v`, `--config`, `--to`, `--run-id`, `--all`, `--remove-on-complete` |
| 値のないオプション | `MissingValue` | `--config`, `--run-id`, `--to` |
| 重複、`--verbose=x` など、`--all` と `--run-id` の併用、`--run-id` の値の規則違反(非 UTF-8 を含む) | `InvalidValue` | オプション名(`-v`, `--config`, …、併用は `--all`) |
| `--to` の値が `yaml`/`json` 以外(非 UTF-8 を含む) | `InvalidValue` | その値(非 UTF-8 は読める形) |
| ジョブID・パイプライン名が非 UTF-8 | `InvalidValue` | 読める形にした語 |
| 値を取るオプションの直後が `-h`/`--help` | (値として取られるので help にならない。ほかに位置引数がなければ `MissingArgument`) | なし |
| 環境変数の run ID の規則違反 | `InvalidValue` | `run ID (environment variable)` |
| 余分な位置引数 | `UnexpectedArgument` | その語 |
| `convert` で `--to` なし | `MissingArgument` | `--to` |
| `run` のジョブIDが定義にない | `UnknownJob` | そのジョブID |

(環境変数の run ID が非 UTF-8 のときは、未設定と同じ扱いで、誤りにならない。04 章 5.1。)

### 7.5 シグナルの準備の誤り

2 種。「すでに登録済み」(成功と同じ扱い)、「失敗」(`conductor: error: failed to set up signal handling`、70)。

## 8. Python でのエラー型の設計

### 8.1 方針

- 規定された誤りの伝え方は、モジュールごとに次のとおり決まっている(8.3 節の表の「伝え方」の列)。**値で返す**ものは、結果の型(`Ok` と `Err` の判別共用体)で返し、例外を送出しない(`definition`、`convert`、`runner`、`flow`、`runid`)。**送出する**ものは、`Exception` を継承したクラスの値を `raise` する(`state` の読み書きの関数。06 章。`cli` の引数の解析の `UsageError` は、値として持ち回る)。`state` の入口 `StateStore`(02 章)は、送出された `StateError` を値に包んで `Deps` に返す。規定された誤りと、想定外の内部失敗(6 章)を、**型で区別する**。規定された誤りは、このあとの型の集合に属し、それ以外は想定外とする。
- 各誤りの型は、「種別(`enum.Enum`)」と「利用者に見える情報」を持つ。文言への変換(ロケール、エスケープ)は `conductor/report/` の責務で、例外の `str(e)`(`Exception.args` の文字列)は、ロケールを判定せず、デバッグ用の固定の英語の 1 行にする(利用者向けの出力には使わない)。
- 終了コードは、エラー型の値から、`conductor/cli/` の 1 つの関数で決める(クラス変数 `exit_code`、または型の振り分け)。終了コードの数値の定義は 1 か所(`ExitCode`)に集める。
- 呼び出し側は、`isinstance`(または `except XxxError`)で型を判定する。**`except Exception` と、裸の `except:` を、規定された誤りの処理に使わない**(想定外の失敗を握りつぶさないため)。想定外の失敗を捕捉してよいのは、`main()` の最上位(6 章)と、08 章 1.3.2 の書き込みの失敗の処理(`OSError`・`ValueError` に限る)だけ。`except OSError` は、ファイル・子プロセス・標準出力の境界で、`OSError` の子クラス(`FileNotFoundError`、`PermissionError`、`IsADirectoryError`、`NotADirectoryError`、`BrokenPipeError`)を、その境界の規定された種類に対応づけるために使う(例: 定義ファイルの `FileNotFoundError` は `fileMissing`、`PermissionError` は `unreadable`)。
- 誤りを包む場合は、`raise XxxError(...) from err` で、元の例外を `__cause__` に入れる。包んでも、利用者向けの出力は、包んだ誤りの型の情報だけから作る(`__cause__` や `str(e)` は、利用者に出さない)。
- `Exception` を継承したクラスは、コンストラクタの引数と `args` が一致しないと、`copy`・`pickle` で復元できなくなる。`conductor` は誤りを複製・直列化しないので、問題にならないが、`super().__init__(message)` を必ず呼び、`args` を空にしない(`Exception` のメッセージとして、固定の英語の 1 行を渡す)。
- 種別の値(`enum.Enum` の値。7.1 の `fileMissing` など)は、永続化も外部への公開もしない(利用者に見えるのは文言だけ)。並びの追加・並べ替えで、互換性の問題は起きない。

### 8.2 定数と共通の基底クラス

```python
import enum
from typing import ClassVar

class ExitCode(enum.IntEnum):
    """終了コード(1 章)。値は 1 か所に集める。"""
    OK = 0
    UNLISTED_ZERO = 1
    DEFINITION = 64
    STATE = 65
    REJECTED = 66
    CONVERSION = 67
    USAGE = 68
    INTERNAL = 70
    START_FAILURE = 127
    SIGNAL_BASE = 128

class ConductorError(Exception):
    """conductor の規定された誤りの基底クラス(送出して伝える型だけが継承する)。"""
    exit_code: ClassVar[int]   # サブクラスが ExitCode の値を定める
    kind_name: ClassVar[str]   # 利用者向けの文言を作るための判別子(固定の文字列)
```

- `ExitCode` は `IntEnum` だが、`sys.exit` と `exit_code_for` には、`int(ExitCode.X)` で、純粋な `int` にして渡す(`IntEnum` は `int` の子クラスなので動くが、`type(code) is int` の検査を通すため、境界で `int()` を通す)。
- 判別共用体: 値で返す誤り(`DefinitionError`、`ConvertError`、`CommandError`、`RunIdError`)は、`Exception` を継承しない `@dataclass(frozen=True, slots=True)` にする。理由: これらは `raise` されず、結果の型に入って返るだけで、`raise` を前提とする例外クラスにするとスタックの情報・`__traceback__` の保持など不要な状態を持つため。`slots=True` の `dataclass` を `Exception` と多重継承すると、レイアウトの衝突の恐れがある(確認していない。未確認)ので、例外クラスは通常のクラス(`dataclass` にしない)で書く。

### 8.3 型の一覧(骨格)

モジュールごとに、自分の誤りの型を持つ(ソース構成は 02 章)。

| モジュール | 型 | 種別の型 | 終了コード | 持つ情報(伝え方) |
|------------|-----|----------|------------|----------|
| `conductor/definition/` | `DefinitionError` | `ErrorKind`(`enum.Enum`。7.1 の 31 種。値は識別子の文字列。03 章) | 64 | ファイル、行・列(`None` は「なし」)、種類、原因(日本語の固定の文)。例外にせず、`@dataclass(frozen=True, slots=True)` の値として `LoadResult` に入れて返す(03 章) |
| `conductor/runner/` | `CommandError` | `CommandErrorKind`(`enum.Enum`。`UNCLOSED_QUOTE`・`EMPTY`。02 章) | 64 | 種類(ジョブIDは、呼び出し側が持つ)。結果の型で返す(07 章) |
| `conductor/runid.py` | `RunIdError` | `RunIdErrorKind`(`enum.Enum`。`EMPTY`・`TOO_LONG`・`INVALID_CHAR`・`LEADING_DOT`) | 68(利用者には `invalid value`) | 種類のみ(値は持たない)。結果の型で返す(05 章) |
| `conductor/state/` | `StateError` | `StateErrorKind`(`enum.Enum`。`unreadable`・`tooLarge`・`corrupt`・`versionMismatch`・`writeFailed`・`removeFailed`) | 65 | パス、種類、原因(固定の日本語の文)、`detail`(任意)。`state` の読み書きの関数は送出する(06 章) |
| `conductor/state/` | `ConsistencyResult` の `mismatch` | `what`(`Literal["pipelineName", "jobIds", "jobOrder"]`、または `enum.Enum`) | 65 | 種類。値で返す(06 章) |
| `conductor/convert/` | `ConvertError` | `ConvertErrorKind`(`enum.Enum`。7.3 の 6 種) | 67 | ファイル、行、キーの位置、種類、原因。値で返す(09 章) |
| `conductor/cli/` | `UsageError` | `UsageKind`(`enum.Enum`。7.4 の 8 種) | 68 | 種類、対象(あれば) |
| `conductor/cli/` | `RejectedError` | `RejectReason`(`enum.Enum`。`OutOfOrder`・`SkipTarget`・`AfterEnd`・`AlreadyExecuted`) | 66 | ジョブID、理由 |
| `conductor/report/` | `OutputError` | (なし) | 呼び出し元による(`watch` は 65、`convert` は 67) | なし。標準出力への書き込みの失敗(08 章 1.3.2)。`ConductorError` を継承しない(終了コードが文脈で決まるため) |
| `conductor/flow/` | (クラスにしない。`JobOutcome` の値) | (なし。分類の結果として扱う) | 3 章から決まる | 報告する終了コード、エラーか |

- 種別は、`enum.Enum`(各メンバの値は 7.1 などの識別子の文字列)にする。メンバの名前は UPPER_SNAKE_CASE。`match` 文と `typing.assert_never`(Python 3.11 以降。最小の版に応じて、自前の同等の関数を持つ)で、網羅性を `mypy --strict` が検査できる。種別を `Literal` の共用体にする代替案も許すが、同じ型の設計(種別の値の集合が型で閉じていること)を守る。
- 「対象なし」と「対象が空文字」を区別する(`unknown command: ''` のように、対象があれば `'...'` で囲んで出す)。`target: str | None = None` として、`None` を「対象なし」、`""` を「対象が空文字」とする(判定は `target is None`。`if e.target:` の真偽判定は、空文字を「なし」と誤るので使わない)。
- 送出する誤りは、必ず `Exception` を継承したクラスにする(`raise` できるのは `BaseException` の子クラスだけで、プレーンな値は送出できない)。値で返す誤りと、送出する誤りを、同じ型で兼ねない。

```python
class UsageKind(enum.Enum):
    UNKNOWN_COMMAND = "UnknownCommand"
    MISSING_ARGUMENT = "MissingArgument"
    UNEXPECTED_ARGUMENT = "UnexpectedArgument"
    UNKNOWN_OPTION = "UnknownOption"
    OPTION_NOT_ALLOWED = "OptionNotAllowed"
    MISSING_VALUE = "MissingValue"
    INVALID_VALUE = "InvalidValue"
    UNKNOWN_JOB = "UnknownJob"

class UsageError(ConductorError):
    exit_code: ClassVar[int] = int(ExitCode.USAGE)
    kind_name: ClassVar[str] = "usage"

    def __init__(self, kind: UsageKind, target: str | None = None) -> None:
        super().__init__(f"usage error: {kind.value}")  # デバッグ用の固定の英語
        self.kind = kind
        self.target = target  # None は「対象なし」。"" は「対象が空文字」。
```

結果の型(値で返す伝え方)の骨格:

```python
from dataclasses import dataclass
from typing import Generic, TypeVar

T = TypeVar("T")
E = TypeVar("E")

@dataclass(frozen=True, slots=True)
class Ok(Generic[T]):
    value: T

@dataclass(frozen=True, slots=True)
class Err(Generic[E]):
    error: E

# 例: LoadResult = Ok[Definition] | Err[tuple[DefinitionError, ...]]
```

(型パラメータの書き方は、サポートする最小の Python の版に合わせる(`TypeVar` と `Generic`、または 3.12 以降の `class Ok[T]`)。骨格は目安であり、`isinstance` または `match` で分岐できる判別共用体であればよい。)

### 8.4 ジョブの結果と終了コード

ジョブの結果(3 章)は、エラー型ではなく、`conductor/flow/` の値(分類、報告する終了コード)として表す。`run` / `pipeline` の処理は、結果を `run` の戻り値の終了コードに変換する。ステートの書き込みの失敗(`StateError`)は、ジョブの結果より優先して終了コード 65 にする(2 章の順 11)。

### 8.5 最上位での決定

`conductor/cli/` の `run` は、各処理が返した結果の型(値で返す誤り)と、送出された値を受け取り、次の順で終了コードと表示を決める(終了コードを決める関数は 1 つ)。

1. `UsageError` → 使い方の誤りの表示(誤りの行、空行、使い方全文)と 68。
2. `DefinitionError` / `CommandError` → 64。`StateError`、整合性の `mismatch` → 65。`RejectedError` → 66。`ConvertError` → 67。それぞれの表示は 08 章。
3. 上のどれでもない値(`ConductorError` のほかのクラス、`Exception` のほかのクラス、`BaseException` を含む。想定していない失敗)は、`run` の中では握らず、`main()` の最上位まで伝わらせて、6 章の `conductor: error: internal error` を標準エラーに 1 行出して 70 で終わる。想定外の値を握りつぶして成功にしてはならない。
4. 型の判定の順序は、同じ値が複数のクラスに当てはまらないように作る(`ConductorError` の直下のクラスは互いに独立で、継承関係を作らない)ので、実際には順序に依存しない。

`BaseException` を継承する値を、呼び出し側が `except` で受けるのは、`main()` の最上位だけ。

## 9. 標準エラーの出力と終了コードの例

```text
$ conductor validate            # command の誤り
conductor: error: d.yaml: job 'z': command has an unclosed quote
→ 64

$ conductor pipeline wrongname
conductor: error: c.yaml:3: pipeline name does not match the definition: <原因(日本語)>
→ 64

$ conductor run extract         # 壊れたステート
conductor: error: c.yaml.state: state file is corrupt: JSON として読めません; run `conductor reset` to discard the state
→ 65

$ conductor run load            # 順序外
conductor: error: job 'load': cannot run this job now: earlier jobs have not finished (out of order)
→ 66

$ conductor convert nofile --to json
conductor: error: nofile: input file does not exist: 指定の場所にファイルがありません
→ 67

$ conductor run nojob
conductor: error: job is not defined in the definition file: 'nojob'
(空行と使い方)
→ 68

$ conductor pipeline p          # step3 の実行ファイルがない
conductor: error: job 'step3': cannot start command 'nonexistent_prog_x'
→ 127

$ (ジョブが SIGKILL で終了)
→ 137
```

## 10. 必須の挙動と、実装に任せてよい点

必須:

- 1 章の終了コードの全件と意味、2 章の決定順序、3 章の分類、4 章の範囲合わせ(終了コードは、0〜255 の `int` にしてから `sys.exit` に渡す)。
- 6 章の 70(固定の 1 行と終了コード)と、`sys.excepthook`・`main()` の `try/except BaseException` の捕捉、`KeyboardInterrupt` → 130、`SystemExit` は再送出。
- 標準エラーの `conductor: error: ...` の行の形式と文言(08 章)。
- 少なくとも 1 回の方針: ステートの書き込みに失敗したとき、ジョブの終了コードより 65 を優先する。
- 8 章の、`Exception` を継承した型(送出する誤り)、値で返す誤りの型、`isinstance` での判定、`enum.Enum` の種別という方針(型の名前と分け方は目安)。
- 終了コードが 120(標準出力のフラッシュの失敗)・1(捕捉されない例外)・2(`argparse` の使い方の誤り)で外に出ないこと。

実装に任せてよい点:

- 型の名前、フィールドの名前、モジュール間の細かい分け方(8.3 は目安)。
- 同時に届いた複数のシグナルのうち、どれを「最初」とみなすか。
- 想定外の失敗の詳細を標準エラーへ追加で出すか(6.1 の 4)。
- 値で返す誤りを、`dataclass` にするか、通常のクラスにするか(8.2)。
- `os._exit(70)` と `sys.exit(70)` のどちらを `sys.excepthook` で使うか(6.1 の 3)。

## 11. この版が採った選択(まとめ)

| 項目 | 内容 |
|------|------|
| ステートの原因の文 | `JSON として読めません`、`キーが重複しています`、`内容を JSON にできません`(7.2) |
| 定義の誤り | 31 種。`unsupportedFormat`(拡張子が `.yaml`・`.yml`・`.json` のどれでもない)を含む(7.1) |
| `retryNotTable` の文言 | 08 章 4.3(`retry must be a map` / `retry はマップで書きます`) |
| 変換の誤り | YAML と JSON の相互変換。6 種の種類と 22 件の原因(7.3) |
| 想定外の内部失敗 | `sys.excepthook` と、入口の `try/except BaseException` で 70(6 章)。`SystemExit` は再送出(6.3) |
| `KeyboardInterrupt` | 捕捉して終了コード 130。メッセージなし(6.2) |
| 終了コードの型 | `int`(任意精度)。`bool` でない 0〜255 の値を `sys.exit` に渡す(4 章) |
| 終了コードの整形 | `reported & 0xFF`、エラーで 0 になれば 1(4 章) |
| エラー型 | `Exception` を継承したクラス(送出する誤り)+ 値で返す誤り(`dataclass`)+ `enum.Enum` の種別。`isinstance` で判定(8 章) |
| 起動の失敗 | `Popen` が送出する `OSError` の子クラスを捕捉して 127(3 章) |

## 12. 未確認

- 定義の誤りの YAML の文法の誤りの行番号(採用する YAML ライブラリの検出位置による。03 章)。
- ステートの読み込みの検査の順序は 06 章で固定してあり、この章に未確認はない。
- Windows で子プロセスの終了コードが `Popen.returncode` として渡るときの符号・値の範囲(3 章)、および、`sys.exit` に範囲外の整数を渡したときの Windows での切り詰め(4.1。POSIX(macOS)の 3.15.2 は確認済み)。
- `sys.exit` に `2**31` 以上・`-2**31` を渡したときの挙動の、3.15.2 以外の版・OS での一致(4.1。仕様は 0〜255 だけを渡すので影響しない)。
- 6 章の確認(ハンドラが 1 を返すこと、70 で終われること、`KeyboardInterrupt` の扱い)は、Python 3.15.2・3.9.6・macOS のみ。サポートする最小の版(実装時に確認する。見込みは 3.11)、3.11〜3.13、3.10.7、3.14.10、Linux・Windows での確認は、実装時に行う。
- 実際のメモリ不足での `MemoryError` が、捕捉でき、標準エラーに 1 行を書けること(6.4。明示的に `raise MemoryError` した場合の確認だけ)。
- `slots=True` の `dataclass` を `Exception` と多重継承したときのレイアウトの衝突(8.2)。
- Windows の子プロセスの `OSError` の種類(3 章。`FileNotFoundError` と `PermissionError` 以外が来うること。07 章と合わせて確認する)。
- 起動の失敗(3 章)のうち、`Popen` が送出する `OSError` 以外の例外(`ValueError`: 引数に NUL 文字を含む、など)の扱い。`command` の解釈の段階で NUL を含む引数が拒否される想定だが、確認していない(03 章・07 章)。
