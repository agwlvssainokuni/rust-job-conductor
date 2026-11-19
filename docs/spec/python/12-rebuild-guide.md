# 12. 再構築の手引き(Python でゼロから実装する)

この文書は、`conductor` を Python でゼロから実装する人のための、実装の順序、Python に特有の落とし穴、未確認事項、最初に確かめる小さな試験をまとめる。各規定の本文は、02〜11 章にある。この文書は、それらへの道案内であり、規定を足さない。食い違いがあれば、各章を正とする。

- 規定の強さの語は [00-README.md](00-README.md) 4 章。
- 受け入れテスト(AC)は [11-quality-and-build.md](11-quality-and-build.md) 4 章。ここで「AC-xxx〜yyy」と書くのは、その範囲の項目を指す。
- Python と各ライブラリの挙動で、作成時に実際に動かして確かめたものは、章ごとに「確認」「実測」「確認済み」と書いてある(macOS。CPython は 3.9.6・3.10.7・3.14.10・3.15.2 のうち章ごとに記した版。`PyYAML` は 6.0.3)。書いていないものは、公式の文書の知識によるもので、**実装時に確認する**。
- サポートする Python の版は、この文書の仮置き(最小 3.11、最新 3.15)で書いてある。実装時に、公式にセキュリティ修正が続いている最小と最新を確認して決める([00](00-README.md) 1 章)。

## 1. 実装の順序

依存の向き([02](02-architecture.md) 3 章)の下から上へ、かつ「純粋な部分を先に、入出力を後に」の順で作る。各段階は、完了条件を満たしてから次へ進む。段階ごとの確認は、その段階の単体テストと、11 章の受け入れテストのうち、その段階で動くものの範囲で行う。受け入れテストは、`python -m conductor` を外から動かすので、`cli` までそろった段階 8 から本格的に使える。それ以前は、単体テストと、`Deps` の偽の部品(02 章 4.4 節)で確かめる。

| 段階 | 作るもの | 読む章 | 完了条件 | 確認する方法 |
|------|----------|--------|----------|--------------|
| 0 | 準備: `pyproject.toml`(PEP 621、`requires-python`、`[project.scripts]`、`ruff`・`mypy`・`pytest`・`coverage` の設定)、`src/conductor/` のレイアウト、`version.py`、`scripts/`(版・依存・ビルドの検査)、`tests/inspect/test_imports.py`(依存の向きと循環)、最初の試験(4 章) | 02 の 8 章、11 の 5・8・9 章 | `ruff check`、`ruff format --check`、`mypy --strict`、`pytest`(空でも)、`python -m build` が通る。`conductor --version` が `conductor 0.8.0` を出す(版の定数が `pyproject.toml` と一致)。4 章の試験 T1〜T3 の結果で、YAML ライブラリの採否が決まっている | AC-476(版の定数)、AC-477(Python の版の記載)、AC-478(依存の整合)、AC-479(ビルド)、AC-480・483(静的検査の設定)、AC-507・508(起動と `pyproject.toml` の形)、AC-514(依存の向き)、AC-518(規則が有効であること) |
| 1 | 土台: 終了コードの定数と `exit_code_for`(`flow/`)、エラーの型(10 章 8 章)、`runid.py`、入口 `__main__.py`(`sys.excepthook` と `try/except BaseException` で 70、`os.write(2, …)` で 1 行、`KeyboardInterrupt` → `exit_by_interrupt`、`sys.exit` は 1 か所、出力の flush と `os.dup2`、起動時の `reconfigure`) | 10、05 の 2・6 章、04 の 5 章、07 の 7.6・10 章、08 の 1.2〜1.4 節 | 終了コードの範囲合わせ(例: 259→3、256→エラーなら 1)、run ID の有効・無効の例(30 文字の日本語が `TooLong`)が、単体テストで通る。偽の失敗(例外)で、固定の 1 行と 70 が出る。出力の失敗で終了コードが 120 にならない | 単体テスト(10 章 4 章の例、05 章 6.1 節の例)。AC-389〜396(終了コード全件。引数だけで出せる範囲)、AC-414、AC-490・491(内部の誤り)、AC-496(終了時の flush)、AC-500(終了の呼び出し) |
| 2 | 引数の解析(字句の段階 `lex` と検査の段階)、`--help`、`--version`、使い方の全文(`report` のうち、誤りの行と使い方だけ先に) | 04、08 の 4.2・8 章 | 04 章 4.3 の判定表の全行が、表駆動テストで通る。使い方の誤りは 68 で、誤りの行・空行・使い方の全文が出る。`argparse` を使っていない。非 UTF-8 の引数(`surrogateescape`)で落ちない | AC-325〜349、AC-447〜449、AC-405・406(ランダム・非 UTF-8 の引数)、AC-515(`--help` の標準出力が閉じたパイプ) |
| 3 | 定義の読み込み: 取り込み(拡張子、大きさ、UTF-8)、`read_json`(自前の JSON の読み取り)、YAML の読み取り(ライブラリ + 追加の実装)、展開と上限、版の確認、構造の検査(Walk)、ジョブ間の検査、位置の出し方、`loose_run_id_env` | 03、10 の 7.1 節 | 03 章 16 節の受理・拒否の例が、すべて表駆動テストで通る。誤りは位置の順に、種類・行・固定の理由で返る。深い入力でも異常終了しない(`sys.setrecursionlimit` を呼ばない) | AC-001〜075、AC-450〜475(`validate` を動かせる段階 8 以降でも確認)。単体テストでは 03 章 7.5 節の試験、AC-412(深さ 1,000)、AC-502(整数の検査)、AC-511・512(辞書のキー)、AC-522〜525、AC-531 |
| 4 | `command` の分割と検査(状態機械。コードポイントで走査。再帰を使わない。`shlex` を使わない) | 07 の 2〜3 章 | 07 章 3 章の受理・拒否の例が通る。約 30 万引数の入力が `RecursionError` にならず分割できる | 単体テスト。AC-076〜098(分割)、AC-409、AC-410 |
| 5 | `conductor/flow/`(純粋な判断): 分類、遷移と次の動き、起動前の判断、再開位置、リトライの判断と待ち時間(`int` の飽和)、記録の関数(`state` の純粋な部分) | 05 | 05 章の判定表と例の表が、すべて表駆動テストで通る。`os`・`sys`・`subprocess` などを import していない。`bool` を整数として受理しない | 単体テスト(05 章 13 章の「固定する入力」)。AC-268、AC-501(飽和演算)、AC-502 |
| 6 | `conductor/report/`: メッセージカタログ(英語・日本語)、ロケールの遅延判定、エスケープと切り詰め(コードポイント単位)、`watch` の表、色、使い方の全文、`Reporter` | 08 | 08 章の表の全文が、単体テストで文字単位で一致する。ロケールは最初に必要になったとき 1 回だけ判定する。孤立サロゲートが `UnicodeEncodeError` を起こさない | AC-350〜388(出力とロケール、エスケープ)、AC-170〜187 の表の部分、AC-519・520 |
| 7 | `conductor/state/`: ステートの JSON(固定のキー順)、原子的な書き込み(`os.open` の `O_EXCL`、`os.fsync`、`os.replace`、ディレクトリの同期)、読み込みの 14 段階の検査、整合性の検査、`reset`、`reset --all` | 06 | 06 章 6.6 節の単体テストの入力が、すべて期待どおり。読み込みは `read_json`(BOM の前処理と、厳格な UTF-8 の復号を含む)で行い、`json.loads` を使わない | 単体テスト(一時ディレクトリの実ファイル)。AC-302〜324(ステートの異常系)、AC-188〜205(`reset`)、AC-147、AC-407、AC-473・474 |
| 8 | `conductor/cli/`: `Deps`(02 章 4 章)、前処理 `prepare`、`run`・`pipeline` の流れ、リトライの実行ループ、`watch`・`reset`・`validate`・`convert` の配線、終了コードの決定。プロセスとシグナルは偽の部品で確かめる | 02 の 4・5 章、05 の 5〜11 章、04 の 7 章、10 の 2・8 章 | 偽の部品の上で、`run`・`pipeline` の全分岐(スキップ、拒否、再開、リトライ、中断、ステートの書き込みの失敗)が通る。ステートの書き込みの失敗は、ジョブの終了コードより優先して 65 | AC-117〜187(`run`・`pipeline`・`watch`)、AC-239〜279(run ID、リトライ)、AC-293〜300(注入)、AC-408、AC-485 |
| 9 | `conductor/runner/`: `run_job`(`Popen`、`poll`、転送の 1 回)、`Interrupt`、`wait_retry`、シグナル監視、`platform.py`(`platform_for`、`detect_platform`、`exit_by_interrupt`) | 07 | 実プロセスのテスト(UNIX 系)と、偽の `ChildLike` の単体テストが通る。Windows の `.cmd` の組み立ては、どの OS でも単体で通る。待機は、合図から 1 秒以内に打ち切れる。`signal.signal` の呼び出しが 1 か所で、終了時にハンドラを元へ戻す | AC-099〜116、AC-280〜292、AC-403、AC-484、AC-486、AC-499、AC-503〜506、AC-509、AC-510・513、AC-517、AC-532〜535。AC-301(Windows の Ctrl+C)は実機で手動 |
| 10 | `conductor/convert/`: YAML ⇔ JSON の変換、JSON の自前の書き出し、YAML の自前の書き出し、表せない値の検出 | 09、03 の 4〜7 章(読み取りを共有) | 09 章 11 節の例が通る。往復で木が等しい。整数は `int` のまま通る。YAML の plain の条件(YAML 1.1 の語を引用符で囲む)が守られる | AC-206〜238、AC-413、AC-418、AC-469〜472、AC-536 |
| 11 | 受け入れテストの全件、CI(GitHub Actions)、リリース(sdist・wheel・`SHA256SUMS`)、デモ、README | 11 | 11 章 4 章の「自動」が 3 OS で通る。品質ゲート(5 章)がすべて通る。リリースの手順(6 章)が、ドラフトのリリースまで通る | AC-397〜446、AC-476〜531 のうち残り(AC-488・489 の起動時間と import、AC-526〜530 の配布物) |

- 段階 7(`state`)と段階 9(`runner`)は、互いに依存しない(どちらも `flow` の型を使う)ので、別の人が並行して作れる。
- 段階 8 では、プロセスとシグナルを偽の部品(`ScriptProcess`、`FakeSignals`)にして、流れだけを確かめる。実プロセスとシグナルは、段階 9 で確かめる。
- 段階 3 の `read_json` は、段階 7 の読み込みと、段階 10 の変換が共用する(`conductor.definition` の公開名。03 章 14.2 節、06 章 6.5 節)。段階 3 を先に終える。
- 各段階の終わりに、`ruff check`、`mypy --strict`、`pytest` を通す。循環 import と依存の向きの検査(AC-514)は、段階 0 のうちに CI に入れておくと、後から崩れない。
- 段階 1 の入口と出力の完了(`reconfigure`、`flush`、`os.dup2`)は、早めに作る。標準出力のパイプが閉じたときの終了コード 120(落とし穴 2.1 の 1)は、後から直しにくい。

## 2. Python 特有の落とし穴

02〜11 章の「注意」「未確認」から集めた。各項目の末尾に、詳細の章を示す。

### 2.1 標準出力・終了・内部の失敗

| # | 落とし穴 | 対処 | 章 |
|---|----------|------|----|
| 1 | 読み手が閉じたパイプへ書いて、終了時の flush まで失敗が持ち越されると、`Exception ignored in: <_io.TextIOWrapper name='<stdout>'>` が出て、**終了コードが 120 になる**(`sys.exit(65)` の 65 が失われる)。標準エラーも同じ | 書くたびに `flush()` する。失敗を捕まえたら、そのストリームの記述子を `os.dup2(os.open(os.devnull, os.O_WRONLY), fd)` で `os.devnull` に差し替える | 08 の 1.3、10 の 1 |
| 2 | `--help`・`--version` の標準出力が書けないとき | 失敗を無視して 0。`watch` は 65、`convert` は 67。`BrokenPipeError` を 70 にしない | 08 の 1.3.2、04 の 8、10 の 2 |
| 3 | Python は起動時に `SIGPIPE` を無視する(`BrokenPipeError` になる)。`signal.signal(SIGPIPE, SIG_DFL)` を呼ぶと、書き込みでシグナルにより静かに死ぬ | `SIGPIPE` の設定を変えない。子には `restore_signals=True`(既定)で既定の動作に戻る | 08 の 1.3.2、07 の 4.2 |
| 4 | 標準出力・標準エラーの符号化は、環境(`PYTHONIOENCODING`、`PYTHONUTF8`、ロケール)で決まる。Windows の既定の `newline` は `\n` を `\r\n` にする | 起動時に `sys.stdout.reconfigure(encoding="utf-8", errors=…, newline="\n")`(標準エラーも)。`sys.stdout` が `None`(記述子が閉じている)なら、書き込みを失敗として扱う | 08 の 1.2、AC-520 |
| 5 | 標準エラーは行バッファ、パイプ・ファイルの先ではブロックバッファのことがある。親のバッファが残ったまま子を起動すると、出力の順序が入れ替わる | 書くたびに `flush()`。子を起動する前にも `flush()` する | 08 の 1.3.2、07 の 4.1、AC-521 |
| 6 | `sys.excepthook` のハンドラが戻ると、終了コードは 1 になる。ハンドラ自身が例外を送出すると `Error in sys.excepthook:` が出て 1 になる | ハンドラの中で 1 行を書き、`os._exit(70)`。内部で `try/except BaseException` して、例外を送出しない | 10 の 6.1・6.4 |
| 7 | `sys.stderr` が `None`、閉じている、符号化の誤りがあるとき、内部の誤りの 1 行が出ない | `os.write(2, b"conductor: error: internal error\n")` を `try/except OSError` で囲む | 07 の 10 |
| 8 | `KeyboardInterrupt` は `Exception` でなく `BaseException`。捕まえずに終わると、トレースバックが出る | 入口で捕まえて、何も出さずに終わる。UNIX 系は `SIG_DFL` に戻して自分に SIGINT を送る(シグナル終了。シェルから見て 130)。Windows は 130。`run`・`pipeline` は、ハンドラの登録後は `KeyboardInterrupt` が起きない | 07 の 7.6、10 の 6.2 |
| 9 | `except Exception:` を広く書くと、想定外の失敗(実装の不具合)が規定の誤りに化ける。`assert` は `python -O` で消える | 狭い型(`OSError`、`ValueError` など)だけ捕まえる。外部の入力の検査に `assert` を使わない(AC-530)。`BaseException` を捕まえるのは入口だけ | 02 の 9、10 の 8.1 |
| 10 | `sys.exit` に範囲外の整数や `bool` を渡すと、終了コードが化ける | 0〜255 の純粋な `int`(`bool` でない)にそろえて渡す。`IntEnum` は `int()` を通す | 10 の 4、8.2 |

### 2.2 整数・型・数値

| # | 落とし穴 | 対処 | 章 |
|---|----------|------|----|
| 1 | **`bool` は `int` のサブクラス**。`isinstance(True, int)` は真。JSON・YAML の `true` が整数として通る。`json.dumps({"exit_code": True})` は `true` を出す | 木で `BoolNode` と `IntNode` を別の型にする。値の判定は `type(x) is int`。ステートを書く前に `exit_code` が `type(x) is int` であることを確かめる | 05 の 0.3、03 の 1、06 の 4.3 |
| 2 | `1.0`・`1e0` は整数でない。`float.is_integer()` や `value == int(value)` で整数と見なさない。`nan`・`inf`・`1e400` は `float` | ノードの種別(`IntNode` か `FloatNode` か)で判定する | 03 の 5.1 |
| 3 | `int()` は寛容: `int("１２")`(全角)は 12、`int("1_0")` は 10、`int(" 7 ")` は 7、`int("+1")`・`int("1\n")` も通る。`float("infinity")` は `inf` | 正規表現(`[0-9]` だけ)で形を先に確かめてから渡す | 05 の 7.3、03 の 4.2、17 |
| 4 | 3.10.7 以降(確認: 3.10.7・3.14.10・3.15.2。3.9.6 には無い)、`int()` は 4,300 桁を超える文字列で `ValueError`。版で結果が変わる | 桁数で先に範囲外にする。巨大な字句で `int()` を呼ばない(`HugeIntNode`) | 03 の 4.2・5.1、05 の 7.3、06 の 6.5 |
| 5 | `int` は任意精度で、折り返さない。64 ビット符号なしの上限を超える待ち時間の計算 | `min(x, 2**64 - 1)` で飽和させる。`& 0xFFFFFFFFFFFFFFFF` で折り返さない。`float` に直して比べない | 05 の 7.6、AC-501 |
| 6 | `time.sleep` の引数の上限は版と OS で違い、超えると `OverflowError` や `OSError` になる。`Popen.wait(timeout=…)` も同様 | 残り時間の全体を渡さず、50 ms 以下の刻みに区切る | 05 の 7.8、07 の 5.1、AC-503 |
| 7 | `dict` は挿入順を保つ(整数に見えるキーでも)。`dict` に入れると重複が黙って落ちる | 木のマップは `tuple[MapEntry, ...]` で持つ。重複は自前で検出する | 03 の 6.1、AC-511 |
| 8 | `sys.getrecursionlimit()` は既定で 1,000。深い入れ子を再帰で読むと `RecursionError` | 明示のスタックで読む。`sys.setrecursionlimit` を呼ばない。深さの上限(32)は、解析の後の段階で効かせる | 03 の 5.3・6.4、AC-524 |
| 9 | `float` の等価: `nan != nan`、`0.0 == -0.0` | 木の等価はビット表現で比べる | 09 の 4.4、03 の 17 |
| 10 | `slots=True` の `dataclass` を `Exception` と多重継承すると、レイアウトの衝突の恐れ(未確認) | 例外クラスは通常のクラスにする | 10 の 8.2 |

### 2.3 文字列・正規表現

| # | 落とし穴 | 対処 | 章 |
|---|----------|------|----|
| 1 | **`re` の `$` は、末尾の改行の直前にも一致する**(`"a\n"` が通る)。`re.match` + `$` で検証すると、改行つきの値が通る | `re.fullmatch` を使う | 03 の 4.2・9、04 の 5.2、05 の 6.1 |
| 2 | **`\d` は Unicode の数字(全角・アラビア・インド数字)に一致する**。`\w`・`isalnum`・`isidentifier` も非 ASCII を受理する | `[0-9]`、`[A-Za-z0-9_]` を明示する(`re.ASCII` でもよい) | 05 の 6、03 の 9 |
| 3 | `re.IGNORECASE` と `str.lower()` は、非 ASCII を ASCII に化かす(`"K".lower()` は `"k"`) | `str.isascii()` のときだけ `lower()` を使う | 03 の 2.1、05 の 6 |
| 4 | **`str.strip()`・`isspace()` は、U+001C〜U+001F を空白として扱う**(Unicode の White_Space ではない)。U+FEFF は取り除かない | 列挙した表(White_Space の 25 文字)で判定する | 03 の 1、07 の 2、AC-523 |
| 5 | **`os.path.splitext(".yaml")` は拡張子を空にする** | 拡張子の判定は自前(最後の `.` から末尾。ASCII の大文字小文字を区別しない) | 03 の 2.1 |
| 6 | 引数・環境変数・ファイル名は、OS から `surrogateescape` で `str` に入る(不正なバイトは U+DC80〜U+DCFF の孤立サロゲート)。`str.encode("utf-8")` は孤立サロゲートで `UnicodeEncodeError` | 出力の前に、エスケープして表示する(`readable()`)。ステートの `definition_path` は U+FFFD に置き換える。木に孤立サロゲートを入れない | 04 の 4.4、08 の 9・11、06 の 4.3、AC-519 |
| 7 | `str` はコードポイントの列(`len("😀") == 1`)。見た目の 1 文字(書記素)ではない | 文字数・切り詰め・表示幅は、コードポイント単位で数える。run ID の長さは UTF-8 のバイト数(`len(value.encode("utf-8", "surrogateescape"))`) | 08 の 11、05 の 6.1 |
| 8 | ファイルをテキストモードで開く、`encoding=` を付けて読む、`utf-8-sig` で復号する、と、CR だけの改行が LF になる・BOM が消える | バイトで読み(上限 + 1 バイトまで)、`data.decode("utf-8")` で厳格に復号する | 03 の 3、06 の 6.1.2 |
| 9 | 書式文字列に入力の値を連結してから `.format` や `%` に渡すと、値が書式として解釈される | 固定の書式文字列に、値を引数として渡す。入力の値を書式文字列にしない | 08 の 10 |
| 10 | U+2028・U+2029 は、`json.dumps(ensure_ascii=False)` が生のまま出す | ステートの書き出しで ` `・` ` に置換する | 06 の 4.3 |

### 2.4 JSON・YAML・設定の読み書き

| # | 落とし穴 | 対処 | 章 |
|---|----------|------|----|
| 1 | **標準の `json.loads` の限界**: 重複キーが後勝ち、`NaN`・`Infinity` を受理、位置は文法の誤りにしかない、孤立サロゲートのエスケープを受理、深い入れ子の挙動が版で違う、`bytes` を渡すと BOM と UTF-16 を受け付ける | 自前の `read_json` を、定義・ステート・変換で共用する。`json.loads` は製品のコードで使わない(`ruff` の禁止、AC-421)。`json.dumps` は、ステートの書き出し(後処理つき)だけ | 03 の 5、06 の 6.5 |
| 2 | `json.dumps` は、浮動小数を `1e+16` と書き、NaN を `NaN` と書き、U+007F・U+2028・U+FEFF をエスケープしない | `convert` の書き出しは自前(`json.dumps` を使わない) | 09 の 6〜7 |
| 3 | `argparse` は、前方一致(`--verb`)を受理し、`--config --help` を「値が要る」誤りにし、誤りを終了コード 2 で終え、文言が版で違う | 引数の解析は自前(2 段階) | 04 の 1.1 |
| 4 | PyYAML の既定のローダーは YAML 1.1 の型(`yes`・`on`・60 進数・日付・`1_000`)を作る。`yaml.load` 系はオブジェクトを構築し、任意のコードが実行されうる。`yaml.compose` は再帰し、アンカーの再定義を拒否する | `yaml.parse(text, Loader=yaml.BaseLoader)` のイベントだけを使い、型は自前で決める。`yaml.load`・`yaml.safe_load`・`yaml.compose` を呼ばない(AC-421、AC-531) | 03 の 7 |
| 5 | PyYAML の `mark.line`・`mark.column` は、U+0085・U+2028・U+2029・CR だけの改行も改行として数える | `mark.index`(コードポイントの添字)から、LF の位置の表で行・列を求める。これらの文字は、ライブラリに渡す前に拒否する | 03 の 7.3・13.1 |
| 6 | PyYAML の C の拡張(`CBaseLoader`)は、環境で有無が違い、挙動も違う | 常に純粋な Python の実装(`BaseLoader`)を使う | 03 の 7.2 |
| 7 | YAML ライブラリは、タブの区切り、1,025 文字の単純キー、アンカー名などで、YAML 1.2 より狭く拒否する。フロー形式の深い入れ子で極端に遅い(超線形) | 03 章 4.1 節の「狭い」点を、そのまま規定とする。開いているコレクションが 1,000 を超えたら読み取りを打ち切る。時間は 03 章 7.5 節の敵対的な入力で測る | 03 の 4.1・6.4・7.4 |
| 8 | PyYAML は、範囲外のエスケープ(`"\U00110000"`)で `ValueError` を送出する(`YAMLError` ではない) | `YAMLError`・`ValueError`・`RecursionError`・`MemoryError` を読み取りの外側で捕まえて `syntax` にする | 03 の 7.4 |
| 9 | `import yaml` は約 12 ミリ秒かかる | YAML を読むときだけ、関数の内側で `import`(遅延)。`--help`・`--version` では読み込まない(`PLC0415` を、その 1 か所だけ無効にする) | 03 の 7.2、02 の 2.1、AC-489 |
| 10 | YAML の出力で、`yes`・`on`・`n`・`y` などの語を plain で書くと、YAML 1.1 の型の解決をするローダーが真偽値に読む | 条件 6 の 16 語を引用符で囲む(YAML 1.2 のローダーでは不要でも) | 09 の 8、AC-470 |

### 2.5 ファイル・ステート

| # | 落とし穴 | 対処 | 章 |
|---|----------|------|----|
| 1 | **`tempfile.mkstemp()` は `0600` でファイルを作る**。置き換え後のステートのパーミッションが `0600` になる | `tempfile` を使わず、`os.open(…, O_WRONLY | O_CREAT | O_EXCL, 0o666)` で一時ファイルを作る(権限は `0666` から umask を引いた OS 標準) | 06 の 5 |
| 2 | `os.replace` は同一ファイルシステム内で原子的(POSIX)。Windows で、他のプロセスが開いているときの `PermissionError`(未確認) | 一時ファイルは、同じディレクトリに作る。失敗は `writeFailed` | 06 の 5、13.4 |
| 3 | `os.fsync` は、ディレクトリにも要る(UNIX 系)。Windows では、ディレクトリを開けない | `StateStoreOptions.sync_directory`(`cli` が `PlatformKind` から決める)で切り替える。失敗は無視する | 06 の 5、02 の 3 |
| 4 | 名前つきパイプ(FIFO)を `open(path, "rb")` で開くと、書き手が現れるまでブロックする。`Path.read_bytes()` は上限を超える巨大なファイルを全部読む | `os.open` に `O_NONBLOCK | O_CLOEXEC | O_BINARY`(`getattr(os, …, 0)`)を付け、`fstat` で通常のファイルか調べ、上限 + 1 バイトまで読む | 09 の 3、03 の 3、AC-536 |
| 5 | Windows の `os.open` は、既定でテキストモード(CRLF の変換、Ctrl-Z で終端) | `O_BINARY` を付ける(未確認) | 09 の 3 |
| 6 | ディレクトリへの `os.unlink` は、OS で `IsADirectoryError` か `PermissionError` になる。`open(dir, "rb")` は `IsADirectoryError` | 「存在しない」以外の削除の失敗は `removeFailed`。ディレクトリを指すステートのパスは、消さない | 06 の 8.1、13.3 |
| 7 | `os.scandir` の `is_file()` は、既定でシンボリックリンクを辿る | `follow_symlinks=False` で、`reset --all` がリンクを消さないようにする | 06 の 8 |
| 8 | 大文字小文字を区別しないファイルシステムでは、run ID `A` と `a` のステートが衝突する | 仕様の記述に基づく(未確認)。run ID の規則(`A-Za-z0-9._-`)は変えない | 05 の 6、AC-260 |

### 2.6 プロセス・シグナル

| # | 落とし穴 | 対処 | 章 |
|---|----------|------|----|
| 1 | **PEP 475**: シグナルで中断された `time.sleep`・`Popen.wait`・`os.read` は、ハンドラが例外を出さない限り、自動で再試行される(`sleep` は残りの時間を眠り直す)。1 回の長い `sleep` では、合図の後も最後まで待ってしまう | ハンドラは旗を立てるだけにして、待つ側が、50 ms 以下の刻みの `sleep` と `time.monotonic_ns` の期限で、旗を確かめる | 05 の 7.8、07 の 5.2・6 |
| 2 | **`signal.signal` は、メインスレッドからだけ呼べる**(`ValueError`)。ハンドラは、メインスレッドのバイトコードの境界で実行される | スレッドを使わない。登録の失敗は 70。終了時にハンドラを元へ戻す(本番では `stop()` を呼ばない) | 07 の 7.1 |
| 3 | ハンドラの中でロックを取る・`print`・`Event.set()` は、自己デッドロックや再入の危険 | ハンドラは旗の代入だけ(`Interrupt.trigger`)。「立っていなければ保存」は、1 回の呼び出し(`dict.setdefault`)で行う | 07 の 6.2 |
| 4 | `Popen.wait()`(引数なし)は、ハンドラが走っても戻らず、転送が子の終了後になる | `poll()` と短い `sleep` で待つ | 07 の 5.2 |
| 5 | `Popen.send_signal` と `poll`/`wait` の競合(終了済みの子へ送る、pid の再利用) | `pid` を自分で保持して `os.kill` に渡さない。必ず `Popen.send_signal` を通す(内部で `poll()` を呼び、終了済みなら送らない)。回収は `run_job` のメインスレッドの `poll()` だけ | 07 の 5.2 |
| 6 | `Popen.returncode` は、UNIX 系でシグナル終了なら**負の値 `-n`**。Windows は符号なし 32 ビット(例: `3221225786`)で返ることがある | UNIX 系は `-returncode` をシグナル番号にする。Windows は `((rc + 2**31) % 2**32) - 2**31` で符号つきに直す(`platform.read_exit`) | 07 の 4.4、05 の 1.2 |
| 7 | 親が `SIGCHLD` を無視(`SIG_IGN`)していると、終了状態が捨てられ、`returncode` が 0 になる実装がある(Linux は未確認。macOS では正しい値) | UNIX 系で、`Popen` の前に `signal.getsignal(SIGCHLD)` が `SIG_IGN` なら `SIG_DFL` に直す | 07 の 4.4、AC-534 |
| 8 | **`start_new_session=True`**: 子は制御端末を持たず、`/dev/tty` を開けない(`ENXIO`)。一方、`process_group=0` は、ジョブが端末の標準入力を読むと **`SIGTTIN` で停止**し、`poll()` は停止を終了として報告しない | UNIX 系は `start_new_session=True`。Windows は指定しない。`/dev/tty` を直接開くジョブは動かない(受け入れる) | 07 の 4.7、AC-288・535 |
| 9 | `subprocess.run` は、`KeyboardInterrupt` や `timeout` で子を `kill` する。`communicate` はパイプを前提とする | `Popen` を直接使う(`stdin`・`stdout`・`stderr` は `None`)。`kill`・`terminate` を呼ばない | 07 の 4.2 |
| 10 | `Popen` が、`OSError`(見つからない・実行権限なし・実行できない形式・引数が長すぎる)、`ValueError`(NUL を含む)、`UnicodeEncodeError`(孤立サロゲート)を送出する | `except (OSError, ValueError)` で 1 つにまとめて起動失敗(127)。`shutil.which`・`os.access` で事前に調べない | 07 の 4.3、AC-522 |
| 11 | Windows の `.cmd` / `.bat`: `list2cmdline` は `cmd.exe` の規則に合わず、引数の中のコマンドが実行されうる | 禁止文字(`"`、`%`、`^`、`&`、`\|`、`<`、`>`、`!`、`(`、`)`、改行、復帰、NUL)を含む引数は起動しない。`cmd.exe /d /s /c` を `str` のコマンドラインで直接起動する(`shell=False`) | 07 の 4.8、AC-113〜115・486・510・513 |
| 12 | Windows は、シグナルを子に転送しない(`send_signal`・`terminate`・`kill` を呼ばない)。`signal.SIGBREAK` は UNIX 系に無い | Windows の分岐では、`getattr(signal, "SIGBREAK")` で読む。受けるのは SIGINT と SIGBREAK | 07 の 7.3・8 |
| 13 | シグナルで中断された `os.fsync` などの長い C の呼び出しは、ハンドラの実行が遅れうる | 待ちのループは、短い刻みで旗を見る(未確認の範囲は 3 章) | 07 の 13 |
| 14 | `sys.platform`・`os.name`・`platform.system()` を、あちこちに書くと、OS の分岐が散らばる | 読むのは `runner/platform.py` の `detect_platform()` だけ。ほかは `PlatformKind` を引数で受け取る(AC-420 の点検) | 07 の 8.2、02 の 3 |

### 2.7 テスト・CI・配布

| # | 落とし穴 | 対処 | 章 |
|---|----------|------|----|
| 1 | GitHub Actions の `strategy.matrix` の中では、`env` コンテキストを使えない | 版を `matrix` の値(`min`/`max`)にして、`env.PY_MIN`・`env.PY_MAX` を式で選ぶ | 11 の 5 章 |
| 2 | `pytest` の `filterwarnings = ["error"]` で、依存のライブラリの警告や `ResourceWarning` が失敗になる | `-X dev -W error` でも警告が出ないことを確かめる(AC-498) | 11 の 3.9 |
| 3 | `tests/` の `threading`・`sys.platform` の使用 | `ruff` の `per-file-ignores` で許す(製品のコードは禁止) | 02 の 8.3 |
| 4 | テストでスクリプトを書いてすぐ起動すると、Linux で `ETXTBSY`(text file busy)が出ることがある | スクリプトは、あらかじめ `tests/jobs/` に置く | 07 の 11 |
| 5 | wheel のファイル名の正規化、sdist の内容(`tests/`、`__pycache__`)、`Requires-Dist` | `scripts/check_dist.py` で点検する(AC-526・527) | 11 の 5.4 |
| 6 | `from __future__ import annotations` を付けないと、前方参照や型だけの import の循環で失敗する。`if TYPE_CHECKING:` の内側の import も、依存の向きの点検の対象 | 全モジュールの先頭に付ける。`tests/inspect/test_imports.py` が内側も数える | 02 の 1・3 |

## 3. 未確認事項の一覧

各章の「未確認」を、分類してまとめた。いずれも、断定していない。実装時に、採用する版と OS で確かめる。

### 3.1 Python の版と、ライブラリの版

| 項目 | 内容 | 章 |
|------|------|----|
| サポートする版 | 最小と最新の具体的な番号(この文書の仮置きは 3.11 と 3.15)。公式にセキュリティ修正が続いている版を、実装時に確認する | 02 の 10.2、11 の 10 |
| 確認した版の範囲 | 各章の確認は、macOS の 3.9.6・3.10.7・3.14.10・3.15.2 のうち、章ごとに記した版。`requires-python` の最小版・最新版そのものでの再確認(03・04・05・06・07・08・09・10 章の確認の全部) | 各章 |
| 3.11〜3.13 | 03・04・09 章の確認は 3.11〜3.13 を含まない | 03 の 18、04 の 11、09 の 12 |
| `PyYAML` の版 | 確認は 6.0.3。採用する版の確認(ライセンス、`pip-audit`、03 章 7.5 節の試験の再実行)。`ruamel.yaml` の全項目は未確認(代替の候補) | 03 の 7・18 |
| YAML 1.1 と 1.2 | PyYAML のスキャナと構文は YAML 1.1 に基づく。1.2 との文法の差が受け入れテストの入力に影響するか | 03 の 18、11 の 10 |
| 4,300 桁の上限 | 3.10.7・3.14.10・3.15.2 で確認。ほかの版は未確認。上限に達したとき `ValueError` が製品のコードで漏れないこと | 03、05、06、11 の 10 |
| `str.strip()` と White_Space の差 | 3.10.7・3.14.10・3.15.2 の全コードポイントで同じ。ほかの版(Unicode のデータが違いうる)は未確認 | 03 の 18 |
| `argparse` | 確認は 3.9.6・3.10.7・3.14.10・3.15.2。3.11〜3.13 は未確認(結論の「使わない」は版によらない) | 04 の 11 |
| `Popen.send_signal` | 終了済みの子に送らない作り(AC-289)の、各版での挙動。確認は 3.9・3.14・3.15 | 07 の 13、11 の 10 |

### 3.2 Windows

| 項目 | 内容 | 章 |
|------|------|----|
| `Popen.returncode` | 符号なし 32 ビットで返るか。負の終了コード(`0xFFFFFFFF`)の実機の値(AC-112) | 05 の 14、07 の 13 |
| Ctrl+C・Ctrl+Break | ジョブの終了の様子。`signal.SIGBREAK` の番号(21 と思われる)。`CTRL_BREAK_EVENT` による自動テストの方法(AC-533)。Ctrl+C の手動の確認の記録(AC-301) | 07 の 7.3・13、11 の 10 |
| `.cmd` / `.bat` の起動 | `cmd.exe` を `str` で直接起動する形が、最小・最新の Python で動くこと(AC-113〜115・486・510・513)。「バッチ ジョブを終了しますか」の問い合わせの影響。プログラム探索の詳細(カレントディレクトリ、拡張子の補完) | 07 の 4.8・13 |
| ステートの書き込み | `os.replace` の、既存ファイルへの置き換えの成否と、同時の読み手に対する原子性(AC-322)。他のプロセスが開いているときの `PermissionError`。`os.O_BINARY`。ディレクトリの `fsync` の可否 | 06 の 13.4、11 の 10 |
| 標準出力 | `reconfigure(encoding="utf-8", newline="\n")` が、コンソールとリダイレクト先の両方で意図どおりに働くか。`os.devnull`(`nul`)への `os.dup2` の差し替えが、終了コードの化けを防ぐか。標準出力の改行(CRLF への変換)。コンソールのコードページの表示 | 08 の 13、09 の 12 |
| 引数・環境 | `sys.argv`・`os.environ` での、対になっていないサロゲートの扱い。`os.environ` の名前の大文字小文字 | 04 の 11、08 の 13 |
| `KeyboardInterrupt` | 終了コード 130 を返す決定(OS の既定は `0xC000013A`)が、実機で動くこと | 07 の 7.6、10 の 12 |
| ほか | `OSError` の `errno`・`winerror`、`time.sleep` の分解能、`conductor.exe`(`pip` が作るラッパー)が終了コードを正しく返すこと、端末の判定(`isatty`)、`ntpath` でのパスの扱い | 03 の 18、07 の 13、02 の 10.2、06 の 13.4 |

### 3.3 Linux(ほかの UNIX 系を含む)

| 項目 | 内容 | 章 |
|------|------|----|
| 新しいセッション | `Popen(start_new_session=True)` の効果(macOS では `ps` で確認)。制御端末を持たないジョブの `/dev/tty`・標準入力への影響(擬似端末の手順は、この文書の作成時には実行していない。AC-182・183・535) | 07 の 13、11 の 10 |
| `SIGCHLD` | 無視して起動されたときの `returncode`(Python の実装についての知識によるもの。AC-534) | 07 の 4.4・13 |
| PEP 475 の表 | 05 章 7.8 節の表の値。`signal.signal` のメインスレッド制約、`send_signal` と終了済みの子 | 05 の 14、07 の 13 |
| ファイル | ディレクトリへの `os.unlink` が `IsADirectoryError` か `PermissionError` か。FIFO の `O_NONBLOCK`(AC-536)。非 UTF-8 のバイトを含むファイル名を実際に開けること(AC-349)。`/dev/full`(AC-495)。`/proc/<pid>/stat` の書式(AC-403) | 06 の 13.4、04 の 11、11 の 10 |
| テスト | `ETXTBSY`(text file busy)の発生と回避策の有効性 | 07 の 11 |

### 3.4 プロセス・シグナルの細部(OS を問わない)

| 項目 | 内容 | 章 |
|------|------|----|
| 同時のシグナル | ほぼ同時に届いた複数のシグナルの処理順序、ネストしたハンドラの動き | 05 の 14、07 の 13 |
| 長い C の呼び出し | `os.fsync` などの途中での、Python のハンドラの実行の遅れ | 07 の 13 |
| ハンドラの自己デッドロック | 一般則として避けるだけで、再現していない | 05 の 14 |
| `sys.excepthook` | `os._exit(70)` の経路の、Linux・Windows と最小の版での確認(macOS の 3.15.2・3.9.6 では確認済み) | 07 の 13、10 の 12 |
| メモリ不足 | 実際のメモリ不足での `MemoryError` が、捕まえられ、標準エラーに 1 行を書けること(明示的な `raise` の確認だけ) | 10 の 12 |
| 端末の Ctrl+C | 端末から押したとき、ジョブが SIGINT を 1 回だけ受けること(擬似端末の下での確認なし) | 07 の 13 |
| `time.monotonic_ns` | システムの休止中に進むか(OS による) | 07 の 13 |
| 公式文書による記述 | 07 章の【実測】の印のないものは、公式の文書と一般に知られた挙動による(動かして確かめていない) | 07 の 13 |

### 3.5 ステート・出力・文字列の細部

| 項目 | 内容 | 章 |
|------|------|----|
| ステート | 名前つきパイプ(FIFO)をステートのパスとして `open` したときの挙動(想定外の入力として扱わない)。`<時刻>` の一時ファイル名で、クロックが戻ったときの衝突 | 06 の 13.4 |
| 非 UTF-8 の引数 | 連結の短縮形の綴りに孤立サロゲートが入る場合の、実機との細かい一致。`=` が連結の途中にある場合の細部 | 04 の 11 |
| 標準入出力が閉じている | `None` のストリームの確認は 3.15.2 のみ | 08 の 13 |
| 端末 | 疑似端末で `isatty()` が真になること(実際の端末での確認が必要) | 08 の 13 |
| YAML の文法の誤りの行 | 採用するライブラリの検出位置による(表は確認した例に限る)。引用符なしのスカラー中の特殊な文字。タブの受理される場所の網羅。複数行のスカラーの文字の大きさの数え方の境界(16 MiB 付近のみ)。ライブラリの版で、深いフロー形式の遅さが改善されれば、1,000 段の打ち切りを見直せる | 03 の 18、09 の 12 |
| 変換 | マージで取り込まれる値の中の表せない値の場所(`a.<<.x`)は、この仕様の決定で、他の規定による裏づけがない。YAML の plain の条件 6 で、`y`・`n` を真偽値にする 1.1 のローダーがあるか | 09 の 12・8 |
| 大文字小文字を区別しないファイルシステム | run ID の衝突(AC-260)は仕様の記述による | 11 の 10 |
| macOS の APFS | 非 UTF-8 のバイトを含むファイル名を作れないこと(AC-349。Linux で作れること) | 11 の 10 |
| `-X dev -W error` | 依存のライブラリが `ResourceWarning` などを出さないこと(AC-498) | 11 の 10 |

### 3.6 道具・CI・配布・性能

| 項目 | 内容 | 章 |
|------|------|----|
| 道具を動かしていない | `ruff`、`mypy`、`pytest`、`pytest-cov`、`build`、`hatchling`、`pip-audit`、GitHub Actions の実行。版、設定のキー・書式、フラグ、終了コードの規則は、実装時に確認する(`actions/setup-python` の `cache`・`allow-prereleases`、`pytest` の `pythonpath`・`faulthandler_timeout`、`pytest-cov` の `--cov-fail-under` と `coverage.json`、`ruff` の `banned-api` の検出範囲、`mypy --python-version`・`--platform`、`pip-audit` のオプション、`python -m build` の分離環境、wheel のファイル名の正規化) | 11 の 10 |
| ビルドバックエンド | `hatchling` が採用案。版、PEP 639 の `license` の書き方、`[dependency-groups]`(PEP 735)を `pip` と `build` の環境が扱えるか | 02 の 10.2 |
| `ruff` の規則 | `select` の集合が、製品のコードに不自然な書き換えを強いないか | 02 の 10.2 |
| `types-PyYAML` | `PyYAML` の版に合う版。置き換える方法(型の情報のない `yaml` を 1 つの部品に閉じて、型つきの `Protocol` で包む)を採る場合は、`check_deps.py` を直す | 11 の 8.3・10 |
| コンソールスクリプト | `main()` の戻り値を `sys.exit` に渡す形が、3 OS の `conductor`(UNIX 系のスクリプト、Windows の `conductor.exe`)で終了コードを正しく返すこと | 02 の 10.2 |
| 性能 | 網羅率の実測値、各性能の計測値(起動時間、import の時間、`run` のオーバーヘッド)。`python -X importtime` の出力の書式は確認したが、`conductor` の import の時間は測っていない。巨大な入力・深い入れ子での時間(1 MiB の入力で 33 段目の打ち切りまで何秒か)。起動時間の測定を強制に切り替える予定 | 11 の 10 |
| ランナー・環境 | `macos-latest` の `shasum` / `sha256sum`、`ubuntu-24.04` の `ps` など、`windows-2022` の `bash` での wheel の展開と `$RUNNER_TEMP`、ブランチ保護(リポジトリの設定) | 11 の 10 |
| 配布 | 単一の実行ファイルの配布(任意。6.4 節)、`build-repro`(wheel が 3 OS で同じ結果になるか)、PyPI への公開の手順の細部(二要素認証、API トークン、信頼できる発行者、配布名の取得) | 11 の 10 |
| テストの補助 | `sleep` / `trap` モードの `TJ` が、シグナルで終了したと観測されること。SIGQUIT の既定の動作。`BrokenPipeError` を確実に起こすための出力の大きさ(AC-493・515・516) | 11 の 10 |

## 4. 最初に確かめる小さな試験

実装を始める前に、次の試験を、採用する Python の版(最小と最新)と、できれば 3 OS で行う。結果で、採否や設計を決める。試験は、リポジトリの外の作業用の場所で行い(仮想環境を作ってよい。リポジトリの中に作らない)、結果を記録する。

| # | 試験 | 手順 | 期待 | 結果の使い道 |
|---|------|------|------|--------------|
| T1 | **PyYAML の採否** | 03 章 7.5 節の 12 項目(受理の構文、タグ、重複キー、深い入力、敵対的な入力が 30 秒以内、型の判定、文法の誤りの行、1,024 文字のキー、位置、サロゲート、追加で拒否する入力、版の間の一致)を、`yaml.parse(…, Loader=yaml.BaseLoader)` のイベントだけで、小さな試作の読み取りに通す | 全項目が、03 章の期待どおり | 通らない版は採用しない。版を替える、03 章 7.4 節の方法で補う、`ruamel.yaml` に替える(03 章 7.5 節の末尾) |
| T2 | 最小版・最新版での再確認 | 03 章 4〜6 節の試験、06 章 6.6 節の入力、05 章 7.8 節の PEP 475 の表、07 章 5.2 節の `Popen.wait`・`send_signal` の挙動、08 章 1.3 節の終了コード 120、10 章 6.4 節の `sys.excepthook` の挙動を、最小の版と最新の版で実行する | 作成時の確認(3.9.6・3.10.7・3.14.10・3.15.2)と同じ | 食い違った項目は、その章の規定を見直す。版で挙動が違う点は、版ごとの分岐を作らず、両方で同じになる書き方にする |
| T3 | 標準出力のパイプが閉じたときの終了コード | `python -c` で、大きな出力(5 MB)を標準出力に書き、読み手(`head -c 1`)が先に閉じる。`write` のあとに `flush()` を呼び、`BrokenPipeError` を捕まえ、`os.dup2(os.open(os.devnull, os.O_WRONLY), 1)` のあとに `sys.exit(65)` する。差し替えなしの場合と比べる | 差し替えなしは 120、あり(差し替え後)は 65。メッセージが出ない | 08 章 1.3.2 の 5 の設計(入口の flush と `os.dup2`)を実装の最初に入れる。Windows では `nul` への差し替えも試す |
| T4 | 標準出力・標準エラーの符号化 | `PYTHONIOENCODING=ascii`、`PYTHONUTF8=0`、`LANG=C`、`LC_ALL=C` の各組み合わせで、日本語を `reconfigure(encoding="utf-8", errors=…, newline="\n")` のあとに出す | どの組み合わせでも、バイト列が同じ UTF-8。`UnicodeEncodeError` が出ない。Windows でも改行が LF のまま | 08 章 1.2 節の決定を確定する。AC-520 の下書き |
| T5 | シグナルと `sleep`(PEP 475) | 1 秒の `time.sleep` に、0.2 秒後に SIGTERM を送る。ハンドラは旗を立てるだけ。50 ms 刻みの `sleep` と `monotonic_ns` の期限で、旗を確かめるループと比べる | 1 回の `sleep` は 1.0 秒まで戻らない。刻みのループは、合図から約 50 ms 以内に戻る | 05 章 7.8 節の方式(刻みの長さ)を確定する。AC-275・503 の土台 |
| T6 | `Popen` の起動と終了の読み取り | `start_new_session=True` の子の `PGID`、`returncode` が負の値(`-15`)になること、`send_signal` が終了済みの子に送らないこと、`SIGCHLD` を `SIG_IGN` にした親での `returncode`(Linux)、孫プロセスを残すジョブを待たないこと | 07 章 4.4・4.7・5.2 節のとおり | 07 章の【実測】の Linux での再確認。AC-288・289・504・506・534 |
| T7 | 擬似端末と `start_new_session` | 標準ライブラリの `pty.openpty()` の子側を標準入出力にした `conductor` が、標準入力を読むジョブを起動する。`process_group=0` の場合と比べる。ジョブから `/dev/tty` を開く | `start_new_session=True` は標準入力を読めて止まらない。`process_group=0` は `SIGTTIN` で止まる。`/dev/tty` は開けない | 07 章 4.7 節の決定を確定する。AC-182・183・535 |
| T8 | `KeyboardInterrupt` の終了 | `os.kill(os.getpid(), SIGINT)` を、Python の既定のハンドラが働く区間(`signal.signal` の登録の前)で送り、`except KeyboardInterrupt` で、`SIG_DFL` に戻して再び自分に SIGINT を送る。Windows では 130 を返す | UNIX 系は `Popen.returncode` が `-2`(シェルから見て 130)、出力なし。Windows は 130 | 07 章 7.6 節の `exit_by_interrupt()` を確定する。AC-532 |
| T9 | Windows の最初の確認 | `Popen.returncode` が、Ctrl+C で終了した子について何を返すか。`signal.SIGBREAK` の値。`Popen(creationflags=CREATE_NEW_PROCESS_GROUP)` の子へ `CTRL_BREAK_EVENT` を送ったときの終了。`.cmd` を `cmd.exe /d /s /c "…"` の `str` で起動した結果(安全な引数、禁止文字)。`os.replace` が、他のプロセスの読み取り中に失敗するか。`os.open` の `O_BINARY`。ディレクトリの `os.open` | 07 章 4.4・4.8・7.3 節、06 章の想定のとおり | Windows の分岐(`read_exit`、`prepare_spawn`、`sync_directory`)を確定する。AC-112・301・322・486・510・533 |
| T10 | ステートの書き込みと読み取り | 一時ファイルの `os.open(…, O_EXCL, 0o666)` の権限(umask 022 で `0644`)、`os.fsync`、ディレクトリの `fsync`、`os.replace`、`/dev/zero` から上限 + 1 バイトで止まる読み取り、FIFO への `os.open(O_NONBLOCK)`(Linux) | 06 章 5 章・09 章 3 節のとおり。FIFO でブロックしない | AC-302〜324・536 の下書き |
| T11 | ビルド | 空のリポジトリで、`hatchling` の `pyproject.toml`(PEP 621、`license`、`[dependency-groups]`、`[tool.hatch.build.targets.wheel]`)を書き、`python -m build` で sdist と wheel を作る。wheel の内容、`METADATA`、`Requires-Dist`、`Tag: py3-none-any` を点検する。空の仮想環境に入れて `conductor --version` | 11 章 AC-526〜529 のとおり | バックエンドの採否(`setuptools` など)。`scripts/check_dist.py` の下書き |
| T12 | 型検査の骨格 | 02・03・05・07・09 章の骨格のコードを 1 つのファイルに連ね(本体は `...` か `raise NotImplementedError`)、`mypy --strict --python-version <最小の版>` を通す。`types-PyYAML` を入れた環境で `import yaml` を含める | エラーなし(本体の無い関数の `Missing return statement` を除く)。`typing.get_args(ErrorKind)` が 31 種 | 型の名前の食い違いを、実装の前に見つける。`types-PyYAML` の採否 |
| T13 | `ruff` の禁止の規則 | AC-518 の違反コードを置いて、`ruff check` が `S602`・`TID251`・`S307`・`ANN401`・`S101`・`T201`、`BLE001`、`PLC0415` を検出するか、`banned-api` が `json.loads`・`yaml.load`・`yaml.compose`・`threading` を検出するか | 非 0 で終わり、それぞれ指摘が出る | `[tool.ruff]` の設定を確定する(規則の番号は採用した版で確かめる) |
| T14 | 起動時間と import | `python -X importtime -m conductor --version` と `--help` で、`yaml` が読み込まれないこと。`python -X importtime` の累積の列で、`conductor` の import の時間 | `yaml` が出ない。起動が目安(11 章 2.1 節)に収まる | 遅延 import の設計(03 章 7.2 節、02 章 2.1 節)を確定する。AC-488・489 |
| T15 | 整数と文字列の基本 | `isinstance(True, int)`、`int("１２")`、`int("1" * 5000)`、`re.match(r"a$", "a\n")`、`re.fullmatch(r"\d+", "１２")`、`"\x1c".isspace()`、`os.path.splitext(".yaml")`、`min(2**70, 2**64 - 1)`、`os.fsdecode(b"\xff")`、`"\udc80".encode("utf-8")` | 本章 2.2・2.3 の表のとおり | 採用する版で、落とし穴の前提が成り立つことを確かめる |

- T1〜T3 は段階 0〜1 の前に、T5〜T8 は段階 9 の前に、T9 は Windows の CI を作る前に行う。
- 試験の結果が、この文書や 02〜11 章の記述と違うときは、その章を直す(この仕様書に、確認した版と OS を書き添える)。

## 5. 迷ったときの指針

- 利用者から見える結果(受理・拒否、終了コード、出力の文字列、作られるファイルの内容)は、必ず 03〜10 章の字句に合わせる。内部の構造は、02 章の依存の向きと、`Deps` の境界を守れば、自由である。
- 標準ライブラリの関数が、規定と違う挙動をするときは、まず本章 2 章の表にあるかを見る。ない場合は、実際に動かして確かめ、確かめた版と OS を、該当の章に書き足す。
- 確かめられなかったときは、断定せず、「未確認」と書いて、3 章の表に加える。
