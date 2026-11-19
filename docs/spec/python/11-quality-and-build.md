# 11 品質とビルド(非機能要件、テスト、受け入れテスト、CI、リリース、デモ、依存、レイアウト)

対象: `conductor` v0.8.0 の Python 版。この文書は、Python で `conductor` を再実装した人が、(1) 同じ品質の基準を満たし、(2) 同じテストを書き、(3) 同じ CI とリリースの仕組みを再現するための仕様である。
この文書の基準を満たし、受け入れテスト(4 章)がすべて通れば、利用者から見て同じ挙動のツールになる。

| 章 | 内容 |
|----|------|
| 1 | この文書の位置づけと、Python 版の前提 |
| 2 | 非機能要件(性能、堅牢性、セキュリティ、保守性)の基準値と、強制か目安かの区別 |
| 3 | テスト構成(単体テスト、受け入れテストの仕組み、テスト用ジョブ、偽の部品、pytest の設定、OS 別のテスト、略記、判定の書き方) |
| 4 | 受け入れテスト一覧(機能ごと。ID は `AC-xxx`) |
| 5 | CI(GitHub Actions)の全ジョブと、品質ゲートの再現手順 |
| 6 | リリース |
| 7 | デモ |
| 8 | 依存ライブラリと `pyproject.toml` / 制約ファイルの運用 |
| 9 | リポジトリのレイアウト |
| 10 | 未確認の事項と、他の章との整合が要る点 |

他章との関係: 振る舞いの規定は [03](03-definition-format.md)(定義)、[04](04-cli.md)(CLI)、[05](05-execution-flow.md)(流れ)、[06](06-state-file.md)(ステート)、[07](07-process-and-signals.md)(プロセスとシグナル)、[08](08-output-and-i18n.md)(出力)、[09](09-converter.md)(変換)、[10](10-errors-and-exit-codes.md)(終了コード)にある。
構成は [02](02-architecture.md)、再構築の手順は [12](12-rebuild-guide.md) にある。受け入れテストは、それらを「前提 → 実行 → 期待」の形で、外から確かめる。

---

## 1. この文書の位置づけと、Python 版の前提

- 受け入れテストは、**`python -m conductor`(または、配布物のコンソールスクリプト `conductor`)を、外から実行して**確かめる(終了コード、標準出力、標準エラー、作られるファイル)。外からは確かめにくい項目は、区分を「単体」「注入」「点検」「手動」「計測」として示す(3.6 節)。
- 定義ファイルは YAML(`.yaml` / `.yml`)か JSON(`.json`)。既定のファイル名は `conductor.yaml`。ステートファイルは JSON で、名前は `<定義ファイル名>.state`(run ID があれば `<定義ファイル名>.<run ID>.state`)。`convert` は YAML と JSON の相互変換(`--to yaml|json`)。
- 受け入れテストの ID は `AC-xxx`。001〜449 は、`conductor` が満たすべき基本の項目。450 以降は、YAML と JSON の扱いに固有の項目(450〜475)、品質・CI・版の項目(476〜487)、Python(CPython の実行モデル、標準入出力、`signal`、`subprocess`、配布物)に固有の項目(488 以降)。番号は、この文書の中で一意。
- 期待の標準エラーの文言は、特に記さない限り `LANG=C`(英語)。理由の文(日本語の固定の文)は、英語の表示でも日本語のまま([08](08-output-and-i18n.md)、[10](10-errors-and-exit-codes.md))。理由の文の正確な字句は、08・10 章が正本で、この文書の例は、それと同じ字句を使う。
- 実行環境は CPython。サポートする版は「実装時点で、公式にセキュリティ修正が続いている版のうち、最小と最新」(以下 `PY_MIN` と `PY_MAX`。具体的な版は**実装時に確認する**。この文書の例では、`PY_MIN` を 3.11、`PY_MAX` を 3.14 としたが、確認した値ではない)。`pyproject.toml` の `requires-python` に `PY_MIN` を書く(8 章)。
- 確認できなかった点は「未確認」と書き、10 章にまとめる。**この文書の作成時に実機で確かめられたのは、次の範囲だけである**(macOS、CPython 3.9.6 の標準ライブラリ。`ruff`、`mypy`、`pytest`、`build`、`pip-audit`、GitHub Actions は、動かして確かめていない)。
  - `json.loads` が `NaN`・`Infinity` を受理する(`json.loads('NaN')` が `nan` を返す)。
  - `bool` が `int` のサブクラスで、`isinstance(True, int)` が真。`min(2**70, 2**64 - 1)` が `18446744073709551615`。
  - 孤立サロゲート(例: `"\udc80"`)は、`str.encode("utf-8")` が `UnicodeEncodeError` になり、`errors="surrogateescape"` ならバイト `0x80`、`errors="backslashreplace"` なら `\udc80` の文字列になる。`os.fsdecode(b"\xff")` が `"\udcff"` を返す(ファイルシステムの符号化の誤りの扱いは `surrogateescape`)。
  - `signal.signal` をメインスレッドの外から呼ぶと `ValueError`(`signal only works in main thread of the main interpreter`)。
  - 標準出力が閉じたパイプへ書くと、`BrokenPipeError` になる。**書き込みが終了時の flush まで持ち越されると、インタプリタの終了時に `Exception ignored in: <_io.TextIOWrapper name='<stdout>' ...>` と `BrokenPipeError` が出て、終了コードが 120 になる**(読み手が閉じたあとで `sys.stdout.write('x')` だけ行う短いプログラムで再現した)。
  - `str.isspace()` は、`\x1c`・`\x85`・`\xa0` で真、`\u200b` で偽(Unicode の White_Space ではない `\x1c`〜`\x1f` も真になる)。
  - `argparse`(3.9.6)は、`--conf x` を `--config x` として受理し(省略形)、`-vv` を受理し、`--config=x --config=y` を後勝ちで受理し、エラーを終了コード 2 で終え、`--config --help` を「値が 1 つ要る」エラーにする。
  - `sys.excepthook` に、標準エラーへ 1 行書いて `os._exit(70)` を呼ぶ関数を設定すると、未捕捉の例外で終了コード 70 になる。
  - `python -X importtime -c pass` が使える。`subprocess.Popen.send_signal` のソースに、`poll()` を先に呼んで pid の再利用を避ける旨のコメント(bpo-38630)がある。
  - `shlex.split` の結果(8.1 節)、PyYAML 6.0.3 のイベントの API の挙動(8.2 節)、標準ライブラリ `json` の挙動(8.2 節)。
- 版の数字(Python、各ライブラリ、GitHub Actions の部品)は、例として書いたものを除き、「実装時に確認する」。

---

## 2. 非機能要件

「強制」は、基準を超えると受け入れテストが不合格になるもの。「目安」は、基準値で、超えたら警告として記録する(不合格にしない)。目安は、利用者から見える挙動を損なわない範囲で、別の値を採ってよい。「仮」は、暫定として置いた値で、実測してから見直す(この文書の作成時には測っていない)。

### 2.1 性能

p95 は、昇順に並べた測定値の `ceil(n × 0.95)` 番目(1 始まり)。測定値は、標準エラーに `[bench] <行>` として出し、環境変数 `GITHUB_STEP_SUMMARY` があれば、そのファイルに `- <行>` として追記する。基準を超えたときは、`警告: <名前>: <値> ms が基準 <限度> ms を超えた` を記録する。
環境変数 `STARTUP_BENCH_ENFORCE` が文字列 `true` のときだけ、目安の基準超えを、テストの失敗にする(CI の既定は `"false"`)。

Python は、起動のたびにインタプリタを立ち上げ、`site` と標準ライブラリの一部と、`conductor` の各モジュールを import する。したがって、`conductor` を 1 回起動する時間は、ネイティブの実行ファイルより長い。**起動時間は目安として測り、利用者から見える挙動の基準にはしない。** import の時間を抑えるため、`--version`・`--help`・使い方の誤り・`reset`・`watch` の経路では、YAML ライブラリを import しない(YAML を読む `validate`・`run`・`pipeline`・`convert` が、読む直前に import する。AC-489)。

| 項目 | 基準 | 扱い | 受け入れテスト |
|------|------|------|----------------|
| `python -m conductor --version` の起動から終了まで(インタプリタの起動と import を含む。20 回。`__pycache__` が作られたあとの値。最初の 5 回を捨てる) | p95 150 ms(Linux・macOS)、300 ms(Windows)(仮) | 目安 | AC-488 |
| `python -X importtime -m conductor --version` の累積の import 時間(`conductor` 配下のモジュールの合計) | 60 ms 以下(仮)。出力に YAML ライブラリのモジュールが現れない | 目安(YAML ライブラリが現れないことは、AC-489 で常に検査する) | AC-489 |
| 定義(ジョブ 10 個)の読み込みと検証(取り込みから結果まで。プロセス内で呼ぶ)を、最初の 5 回を捨てて 100 回 | p95 20 ms | 目安 | AC-397 |
| 起動前の処理(`command` の検査、ステートの読み、照合、速い書き込み)を、最初の 5 回を捨てて 100 回。ジョブ 10 個の定義 | p95 20 ms(仮)。`fsync` つきの耐久性のある書き込みは基準なし(記録のみ) | 目安 | AC-398 |
| `run` の `conductor` 自身のオーバーヘッド(ジョブ 10 個の定義、20 回。`conductor` 自身の Python の起動を含む) | p95 350 ms(仮) | 目安 | AC-399 |
| `pipeline` のジョブ 1 つあたりの増分(ジョブ 10 個と 1 個の `pipeline` の差から求める。5 回) | p95 50 ms | 目安 | AC-400 |
| ジョブ 1,000 個の定義で `run job0` が終わるまで(起動を含む) | 1,500 ms(仮) | 目安 | AC-401 |
| ジョブ 1,000 個の `pipeline` の流れの判断(偽の `Deps` を使い、プロセスとファイルなし) | 10,000 ms | 目安。ただし、完了のステートが残ることは常に検査する | AC-402 |
| ジョブの実行中の `conductor` の CPU 時間の増分(Linux。待機の 1 秒後から 2.5 秒後までの増分) | 50 ms 以下 | **強制**(`/proc` を読めない環境ではスキップ) | AC-403 |
| 同じ定義への 2 つの `run` を同時に | 両方が実行され、どちらもジョブの終了コードで終わり、ステートは読み戻せる(排他制御はしない) | **強制** | AC-404 |
| リトライの待機の中断 | 合図から 1 秒以内に返る。待ちは短い区間に分けて合図の旗を見る(区間の長さは 07 章の決定。この文書の基準は、1 秒以内の中断と、CPU 時間の上限だけ) | **強制**(1 秒) | AC-275 |
| ジョブの終了の待ち | `Popen.wait(timeout=…)` の短い区間(07 章)で待つ。合図から転送までの遅れは、区間の長さ以下 | 目安 | [07](07-process-and-signals.md) |
| ジョブが大量の出力(16 MiB)を標準出力に書いても、`conductor` がそれを溜め込まない | 子の標準入出力は引き継ぐ(`stdout=None`)ので、出力は `conductor` を通らない。出力の全体がそのまま読み手に届く | **強制** | AC-494 |

測定のしかた:

- 「`conductor` 自身のオーバーヘッド」は、`conductor` を起動してから終了するまでの時間から、ジョブ自身の時間を引いた値。ジョブ自身の時間は、テスト用ジョブ `timing`(3.4 節)が出す `start_ns`・`end_ns` の差と、テスト用ジョブ自身の起動の費用(基準。`TJ exit 0` を直接 20 回起動した所要時間の中央値)の和。
- 「ジョブ 10 個の定義」は、10 個とも同じ形の簡単なジョブ(テスト用ジョブの `exit 0` か `timing`)。
- 「`pipeline` のジョブ 1 つあたりの増分」は、`(10 個の pipeline の全体 − 1 個の pipeline の全体 − 9 × ジョブ 1 つの自身の時間) / 9`。
- 測定の対象は、リリースと同じ作り方の成果物(wheel を、空の仮想環境に入れたもの。6 章)。ソースの作業ツリーから `PYTHONPATH=src` で測ってもよいが、その場合は記録に明記する。
- バイトコードのキャッシュ(`__pycache__`)がない最初の起動は遅い。プロセス内の測定は、最初の 5 回を捨てる。プロセスの起動の測定も、最初の 5 回を捨てる。
- 性能のテストは、`tests/perf/` に置き、通常の実行では、環境変数 `CONDUCTOR_PERF=1` がなければスキップする(`tests/perf/conftest.py` が、その環境変数を見てモジュールごとスキップする)。測定は 3 OS(Linux・macOS・Windows)で行い、失敗しても全体は不合格にしない(目安)。

### 2.2 堅牢性

外部の入力(コマンドライン、`command` の文字列、ステートファイル、定義ファイル、変換の入力、OS の結果)に対して、**異常終了しない**(失敗は終了コードとメッセージで返す)。【強制】
Python では、捕まえそこねた例外は、トレースバックを標準エラーに出して終了コード 1 で終わる。再帰が深すぎると `RecursionError` になる(既定の上限は `sys.getrecursionlimit()` で、実機の CPython 3.9.6 では 1000)。YAML・JSON の読み取りと、定義・ステート・変換の走査は、入れ子の深さの上限(32 段)を、再帰に入る前に検査する(深さの数え上げは明示的なスタックか、字句の走査で行い、`sys.setrecursionlimit` を上げて逃げない。AC-412、AC-524、AC-232)。入口(`conductor.__main__`)は、`try`/`except BaseException`(`SystemExit` を除く)と、`sys.excepthook` で、残った失敗を 1 行(`conductor: error: internal error`)にして、終了コード 70 で終わる(AC-414、AC-490、AC-491)。
標準出力・標準エラーへの書き込みの失敗(`BrokenPipeError`、`OSError`、`UnicodeEncodeError`)も、入口で処理する。終了時の flush の失敗で、終了コードが 120 になる(Python の既定の動作)ことを許さない(AC-496)。

| 項目 | 基準 | 受け入れテスト |
|------|------|----------------|
| ランダムな引数 | 固定の種のランダムな引数列(語の表から 0〜6 個を選ぶ。乱数は 3.1 節の `xorshift32`)500 回で、終了コードが 0〜255、標準エラーに制御文字(改行を除く)と U+202E がない | AC-405 |
| UTF-8 でない引数(UNIX 系) | ジョブID、`--config`、オプション、`convert` の引数に、不正なバイト列を渡しても異常終了しない(Python は、`sys.argv` の不正なバイトを、`surrogateescape` で U+DC80〜U+DCFF の孤立サロゲートにする。08 章、04 章 4.4 節)。出力は、孤立サロゲートで `UnicodeEncodeError` にならない | AC-406、AC-519 |
| 壊れたステート | 壊れた JSON、ランダムなバイト列、状態・再開位置・フラグを乱したステート、巨大な値で、異常終了しない(終了コード 65) | AC-407 |
| 空・制御文字を含むジョブID | 異常終了しない | AC-408 |
| ランダムな `command` | 文字集合 `a b ⇥ ⏎ ' " \ ; \| & $ * あ` から長さ 0〜39 の文字列 2,000 件で、分割が成功したら `program` は空でない。成功か誤り(64)かのどちらか | AC-409 |
| 大きな `command` | 約 30 万引数が分割できる。閉じない引用符は誤り。再帰に依存しない | AC-410 |
| 起動できない `command` | 空文字列、`'`、`''`、存在しない名前、ディレクトリのパス、`/`、NUL を含む文字列は、すべて起動失敗(または定義の誤り)。`Popen` が投げる `OSError` の各種と `ValueError`(`embedded null byte`)を、すべて起動失敗として扱う。範囲外のシグナル番号の合図でも起動せず中断として扱う | AC-411、AC-522 |
| 入れ子の深い入力 | 深さ 1,000 の YAML・JSON(定義・ステート・変換)で、異常終了せず失敗を返す(`sys.setrecursionlimit` を既定のままで)。`RecursionError` が漏れて 70 にならない | AC-412、AC-524 |
| 整数の字句 | `bool`(`true`/`false`)、浮動小数(`1.0`)、`NaN`・`Infinity`、桁数が極端に多い整数(桁数の上限は `sys.set_int_max_str_digits` の値による。既定 4300 と記憶しているが、この文書の環境では未確認)で、異常終了せず、整数として受理しない | AC-502 |
| 製品のコードの書き方 | 外部の入力の経路に、異常終了を起こす書き方(`assert` による検証、`typing.cast`、`Any`、`# type: ignore`、`json.loads`、`yaml.load` の系列、入口以外での `sys.exit`、裸の `except:`、`BaseException` の捕捉の入口以外での使用)がない | AC-421 |
| 内部の誤り | 入口の `try`/`except` と `sys.excepthook` が、固定の英語 `conductor: error: internal error` を標準エラーに 1 行書き、終了コード 70。標準エラーが閉じていても 70 | AC-414、AC-490、AC-491 |
| 変換の入力 | 範囲外・空・壊れた入力、ランダム・切り詰めた入力で異常終了しない。成功したランダムな構造は往復で等しい | AC-413 |
| 標準出力の失敗 | 標準出力への書き込みの失敗(`BrokenPipeError`、`ENOSPC` など)を捕捉する。`watch` は 65、`convert` は 67 で、メッセージを標準エラーに出す | AC-186、AC-236、AC-493、AC-495 |
| 標準出力の取りこぼし | 標準出力がパイプで、読み手が遅くても、終了前に flush して書き込みの完了を待つ。出力が途中で切れない | AC-494 |
| 終了時の flush | 標準出力が閉じていても、インタプリタの終了時に `Exception ignored in` を出さず、終了コードが 120 にならない | AC-496 |
| 終了後に残らない | すべてのサブコマンドが、出力の完了から 1 秒以内に、プロセスとして終了する(タイマー・シグナルの登録・スレッドが残ってハングしない) | AC-497 |
| 警告を出さない | Python が標準エラーに出す警告(`DeprecationWarning`、`ResourceWarning`、`RuntimeWarning` など)が、通常の実行で出ない。`-X dev -W error` で実行しても、結果が変わらない | AC-498 |
| 時間の上限 | プロセスを起動するテストは、すべて 30 秒の上限つき(超えたら子プロセスを強制終了して不合格) | 3.3 節 |

### 2.3 セキュリティ

| 項目 | 基準 | 受け入れテスト |
|------|------|----------------|
| シェルを呼ばない | 起動は、`command` の分割(または配列)で得た `program` と引数を、`subprocess.Popen`(`shell=False`。引数は配列)に渡すだけ。`subprocess.run`・`call`・`check_call`・`check_output`・`getoutput`、`os.system`・`os.popen`・`os.exec*`・`os.spawn*`、`shell=True`、`sh`・`cmd`・`bash`・`powershell`・`-c`・`/C` を挟む経路がない(Windows の `.cmd` / `.bat` の安全策は 07 章) | AC-116(点検)と 4.2 節 |
| 引数にメタ文字が入っても注入にならない | `a;b`、`&&`、`\|`、`$HOME`、`*`、`>x` などが、そのまま引数として渡る。Windows の `.cmd` への危険な引数で、続くコマンドが実行されない | 4.2 節 |
| 検証は副作用を持たない | 定義の検証は、コマンドを解釈・展開・起動しない。定義ファイルのディレクトリは変わらない | AC-415 |
| 入力の値を表示に出さない | すべての種類のエラーの表示に、入力の値(秘密のつもりの文字列、`command`、ライブラリのエラー文)が現れない。キー名は制御文字をエスケープして表示。**YAML ライブラリが返すエラー文(`yaml.YAMLError` の `str()` は、入力の断片と位置を含みうる)と、例外のトレースバックを、そのまま表示に出さない**(行・列の位置だけを使う) | AC-416 |
| YAML から Python のオブジェクトを作らない | YAML の読み取りは、構文木(ノード)の段階まで(`compose` 系)で、**`yaml.load`・`yaml.unsafe_load`・`yaml.full_load` を呼ばない**(Python のオブジェクトの生成で任意のコードが実行されうる)。`!!python/object/apply:…` などのタグは、タグとして拒否する | AC-531 |
| ステートに機微な値を書かない | ステートファイルに、`command`、引数、環境変数の値、標準出力の内容が現れない | AC-417 |
| 表示のエスケープ | 制御文字・方向制御文字・孤立サロゲートを無効化する(4.14 節の表)。定義・ステート・変換の各部が、同じ入力に同じ結果を返す | 4.14 節、AC-519 |
| ネットワークに接続しない | 実行中に外向きの通信をしない。製品のコードが `socket`・`ssl`・`urllib`・`http`・`ftplib`・`smtplib`・`xmlrpc`・`asyncio`・`selectors` などを import しない。実行時の依存(YAML ライブラリ)も、実行中に通信しない | AC-419(点検) |
| 動的なコードの実行をしない | 製品のコードが、`eval`、`exec`、`compile`、`__import__`、`importlib` による動的な読み込み、`pickle`・`marshal`・`shelve`、`ctypes`・`cffi`、`multiprocessing`、`runpy` を使わない | AC-420(点検) |
| シグナルの送信 | ジョブの子プロセスの pid だけ(0 以下にならない)、許可したシグナルだけ、1 回だけ(プロセスグループには送らない) | 4.10 節の単体の項目 |
| テスト専用の経路を製品に持たない | 製品の入口に、わざと異常終了するコマンドや、テスト用の環境変数・隠しオプションがない。配布物(wheel)に `tests/` を含めない | AC-422(点検)、AC-526、6 章 |
| 変換の副作用 | 変換の部品(`src/conductor/convert/`)が `subprocess`・ネットワークのモジュールを import しない。ファイルを開くのは、入力ファイルを読む入口だけで、本体(文字列から文字列への変換)と書き出しは、ファイルにも環境変数にもカレントディレクトリにも触れない。`convert` は入力以外のファイルを作らず、カレントディレクトリの変更と環境変数に左右されない | AC-418 |
| 既知の脆弱性 | 依存の既知の脆弱性を、定期的に(毎週)と、変更のたびに `pip-audit` で検査する。結果は警告で、マージをブロックしない | 5 章 |
| 供給網(ビルドの再現) | 制約ファイル(`constraints.txt`)に、直接の依存の正確な版を書き、CI は `-c constraints.txt` を付けてだけ入れる(8.4 節)。実行時の依存は 1 つ(8 章) | 5 章、AC-478 |
| 配布物の完全性 | チェックサムのファイル(`SHA256SUMS`)を付ける(署名はしない。転送の破損の検知まで) | 6 章 |

### 2.4 保守性

| 項目 | 基準 | 受け入れテスト |
|------|------|----------------|
| OS 固有コードの隔離 | OS ごとの分岐(`sys.platform`、`os.name`、`platform.system()`)は、`src/conductor/runner/platform.py` の 1 か所だけ。ほかの部品は、そこが出す関数・値(UNIX 系か Windows かの判定、受け取るシグナルの一覧、ディレクトリの同期の可否)だけを使う | AC-420 |
| 外部の入力で異常終了しない書き方 | 2.2 節 | AC-421 |
| 型の検査 | `mypy --strict` が `src`・`tests`・`scripts` で通る。`Any`、`# type: ignore`、`typing.cast` を製品のコードに書かない(`ruff` の `ANN401` ほか。9.3・9.4 節) | AC-480、AC-483、AC-518 |
| 整形と静的検査 | `ruff format --check` の差分、`ruff check` の指摘が 1 つでもあればマージを止める | 5 章 |
| 命名 | PEP 8 の命名(関数・変数・モジュールは snake_case、型・クラスは PascalCase、定数は UPPER_SNAKE_CASE)に従う。独自の規則は設けない | (規約。`ruff` の `N` 規則) |
| 部品の分け方 | `definition/` と `flow/`(入出力なし。ファイルとプロセスに触れずに検査できる)、`runner/` と `state/`(入出力あり)、`convert/`、`report/`、`cli/`(引数・流れの組み立て・`Deps`)を分ける([02](02-architecture.md))。依存の向きと循環を `tests/inspect/` で検査する | AC-514 |
| ソースへの著作権表示 | 各ソースファイル(`.py`、`.yml`、`.sh`、`.cmd`)の先頭に、Apache License 2.0 のヘッダ(`Copyright 2026 agwlvssainokuni`)を置く | AC-423(点検) |

---

## 3. テスト構成

### 3.1 方針

- 方法論は test-after(実装してから、層ごとにテストを書いて実行する)。単体テストと、`python -m conductor`(または配布物)を `subprocess` で起動する受け入れテストを持つ。
- テストの枠組みは `pytest`。アサーションは素の `assert`(pytest が書き換える)。テスト用の別のライブラリ(`pytest-timeout`、`pytest-xdist`、`pytest-mock`、`hypothesis`、擬似端末など)は入れない(8 章。擬似端末は標準ライブラリの `pty`(UNIX 系)で足りる)。製品のコードでは `assert` を使わない(`python -O` で消えるため。AC-421)。
- 一時ディレクトリは、各テストが pytest の `tmp_path` で受け取る(以下 `T`)。**テストごとに別のディレクトリ**なので、ステートファイルの名前(`<定義>.state`)は、テストの間で衝突しない。環境変数は、テストのプロセスの `os.environ` を書き換えず、子プロセスの `env` に渡す(プロセス内の単体テストは、偽の `Deps` で環境を渡す)。`monkeypatch.setenv` を使う場合は、そのテストの中だけに閉じる。
- 並列実行: `pytest-xdist` は使わない(1 つの pytest プロセスが、テストを順に実行する)。理由は、(1) 依存を増やさない(8 章)、(2) 受け入れテストが子プロセスとシグナルを扱い、並列にすると CPU の取り合いで 30 秒の上限と時間の基準が不安定になるため。将来 `pytest-xdist` を入れても、各テストは自分の `tmp_path` だけを使うので、ステートファイルは衝突しない。同じ `T` を複数のテストで共有しない。
- 入力と期待の組が多いものは、**テーブル駆動**(`@pytest.mark.parametrize`)で書く。ID(`AC-xxx`)は、テストの名前(`ids=` か関数名)に含める(例: `def test_ac_092_single_quote_not_closed(...)`、パラメータの ID は `"AC-092"`)。
- テストの名前は、日本語でも英語でもよい(関数名は ASCII。説明は docstring に書く)。
- 行カバレッジの下限は **80%**(5 章)。
- プロセスを起動するテストは、30 秒の上限で、ハングだけを検知する(3.3 節)。
- 乱数が要るテストは、固定の種の擬似乱数 `xorshift32` を使う(`random` モジュールは、版による出力の違いを避けるため使わない。確認はしていない)。実装(32 ビット符号なしの演算。Python の `int` は任意精度なので、`& 0xFFFFFFFF` で切る):

```python
class Xorshift32:
    """固定の種で、版によらず同じ列を返す擬似乱数。"""

    def __init__(self, seed: int) -> None:
        self._state = (seed & 0xFFFFFFFF) or 0x9E3779B9

    def next_u32(self) -> int:
        x = self._state
        x ^= (x << 13) & 0xFFFFFFFF
        x ^= x >> 17
        x ^= (x << 5) & 0xFFFFFFFF
        self._state = x
        return x

    def below(self, n: int) -> int:
        """0 以上 n 未満の整数。(わずかな偏りは許す。試験の入力を作るだけ)"""
        return self.next_u32() % n
```

### 3.2 部品ごとの単体テスト

`src/conductor/<部品>/` の各部品は、`tests/unit/<部品>/` に単体テスト(`test_*.py`)を置く。

| 部品 | 主な試験対象 | 方法 | 主な AC |
|------|--------------|------|---------|
| `definition/` | 定義の読み込み・検査・型・エラーの種類と行番号。YAML と JSON の読み取り(タグ・複数文書・重複キー・非文字列キーの拒否、深さと大きさの上限)。`command` の分割。整数の字句検査(`bool`・浮動小数・範囲外) | テーブル駆動。`tests/fixtures/*.yaml`・`*.json` の入力例 | AC-001〜098、AC-450〜、AC-502、AC-522〜525 |
| `convert/` | YAML ⇔ JSON の変換、往復、表せない値、アンカーの展開、暴走する入力の上限 | テーブル駆動。往復の性質テスト(固定の種) | AC-206〜238 |
| `runid.py` | run ID の検査(文字種と長さの規定は [05](05-execution-flow.md))と決定の順序 | テーブル駆動 | AC-239〜260 |
| `flow/` | 純粋な判断: 分類 10 項目、遷移、再開、リトライの待ち時間(`min(x, 2**64 - 1)` の飽和)、終了コードの範囲合わせ | テーブル駆動。ファイル・プロセスなし | AC-268、AC-392〜394、AC-501 |
| `state/` | ステートの読み書き(実ファイル、`T`)、整合性検査(14 段階)、原子的な置換(`os.replace`)、reset(`--all`、シンボリックリンク) | 実ファイル。書き込みの失敗は、置き換え先をディレクトリにして起こす | AC-302〜324 |
| `runner/` | 子プロセスの起動と待ち、環境変数・作業ディレクトリの引き継ぎ、シグナルの転送(1 回だけ・許可した番号だけ)、シグナル名と番号の対応 | テスト用ジョブ `TJ` を実際に起動 | AC-099〜116、AC-280〜292、AC-504、AC-505 |
| `report/` | 表示の全種類の文言(英語・日本語)、ロケールの決定、色、エスケープ、`watch` の表 | テーブル駆動 | AC-350〜388 |
| `cli/` | 引数の文法、各サブコマンドの流れ、終了コードの決定 | **偽の `Deps`**(3.7 節)。ファイル・プロセス・シグナルなし | AC-125〜、AC-293〜300、AC-325〜349 |
| `tests/support/` | 偽の部品、一時ディレクトリの補助、30 秒の上限つきの実行、記録の読み取り | (試験の道具。単体試験は不要。カバレッジの対象は `src/` だけなので、自然に除かれる) | — |

### 3.3 受け入れテストの仕組み(`subprocess` の統合テスト)

受け入れテストは、`conductor` と、テスト用ジョブ `TJ`(3.4 節)を、`subprocess.Popen`(または、30 秒の上限つきの `subprocess.run`)で起動して確かめる。置き場所は **`tests/acceptance/`** の `test_*.py`。機能ごとにファイルを分ける。

| ファイル | 内容 |
|----------|------|
| `tests/conftest.py` | 共通の fixture(`T` を作業ディレクトリにする、既定の環境、`conductor` の起動方法の決定。下記) |
| `tests/support/harness.py` | 共通の補助(3.8 節): `run_conductor`、定義とステートの書き出し・読み取り、30 秒の上限 |
| `tests/acceptance/test_validate.py` | 4.1 節、4.2 節、450〜 の定義の検査 |
| `tests/acceptance/test_run.py` | 4.3 節、4.4 節(`run`、`pipeline`) |
| `tests/acceptance/test_aux.py` | 4.5 節、4.6 節(`watch`、`reset`)、4.8 節(run ID) |
| `tests/acceptance/test_convert.py` | 4.7 節 |
| `tests/acceptance/test_retry.py` | 4.9 節 |
| `tests/acceptance/test_signal.py` | 4.10 節(UNIX 系の項目は `skipif(sys.platform == "win32")`) |
| `tests/acceptance/test_state.py` | 4.11 節 |
| `tests/acceptance/test_args.py` | 4.12 節〜4.15 節 |
| `tests/acceptance/test_nfr.py` | 4.16 節(堅牢性) |
| `tests/acceptance/test_python.py` | 4.20 節(Python に固有の項目) |
| `tests/acceptance/test_demo.py` | 4.17 節(デモ。UNIX 系と Windows で `skipif`) |
| `tests/acceptance/test_dist.py` | 4.18 節(配布物。ビルド済みの sdist・wheel を、環境変数 `CONDUCTOR_DIST_DIR` で受ける。なければスキップ) |
| `tests/perf/test_*.py` | 2.1 節の計測(`CONDUCTOR_PERF=1` のときだけ) |
| `tests/inspect/test_*.py` | 点検(`ast` で、ソースを調べる。3.6 節) |

`conductor` の起動方法の決め方(`tests/conftest.py` の fixture `conductor_cmd`。セッションに 1 回):

1. 環境変数 `CONDUCTOR_BIN` があれば、そのパスを `conductor` として**直接実行する**(配布物のコンソールスクリプトを試験するため。AC-446)。拡張子が `.py` なら、`sys.executable` で実行する。
2. なければ、環境変数 `CONDUCTOR_PYTHON` があれば、そのインタプリタで `-m conductor` を実行する(配布物を入れた仮想環境の `python` を指定して、wheel の中身を試験するため)。
3. どちらもなければ、`[sys.executable, "-m", "conductor"]` で実行し、子の環境変数 `PYTHONPATH` に `<リポジトリの根>/src` を足す(ソースの作業ツリーを試験する。リポジトリの根は `tests/conftest.py` の場所から `..` と決める)。1・2 の場合は、`PYTHONPATH` を足さない(作業ツリーの `src/` が、配布物より先に読み込まれて、配布物の試験にならないことを避ける)。
4. テスト用ジョブのスクリプトは、環境変数 `CONDUCTOR_TESTJOB` があればそのパス、なければ `tests/jobs/tj.py`。

各テストは、`T` をカレントディレクトリにして、`subprocess.Popen([...conductor_cmd, *args], cwd=T, env=env, stdin=…, stdout=PIPE, stderr=PIPE, shell=False)` で実行する(`stdin` は、既定で `DEVNULL`)。`env` は 3.5 節の既定の環境変数だけ(親の環境を引き継がない)。
30 秒を超えたら、`conductor` を強制終了(UNIX 系は `SIGKILL`、Windows は `Popen.kill()`)し、記録(`pid=`)に書かれたジョブの pid があればそれも強制終了して、テストを失敗にする(`harness.run_conductor` の `communicate(timeout=30)`)。加えて、`pytest` の `faulthandler_timeout`(3.9 節)で、テストの全体のハングに備える(トレースバックを出すだけで、テストは止めない)。

### 3.4 テスト用ジョブ(`tests/jobs/tj.py`、以下 `TJ`)

受け入れテストで使う、小さな Python のスクリプト。配布物には含めない(6 章)。起動の費用を抑えるため、**標準ライブラリの `os`・`signal`・`sys`・`time` だけ**を import する(`json`・`pathlib`・`subprocess` などの、起動の遅くなるモジュールを読まない)。
定義の中では、`TJ` は、2 つの語 `"<sys.executable>" "<tj.py の絶対パス>"` を表す(各語を二重引用符で囲む。パスに空白があってもよい)。例: `command: "TJ exit 3"` は、定義の中で `"/usr/bin/python3" "/repo/tests/jobs/tj.py" exit 3` という `command` になる。
定義を書き出す補助(`harness.write_definition`)は、すべての文字列を `json.dumps(s, ensure_ascii=False)` で出す。JSON の文字列は、YAML の二重引用符の文字列としても同じ値を表すので、Windows のパス(`\`)も、引用符も、そのまま正しく書ける(ただし、U+2028・U+2029、孤立サロゲートを含む文字列は、この補助の対象外。必要な項目は、手で書く)。

```text
TJ [--out <ファイル>] <モード> [引数...]
```

- 1 回の `TJ` の起動は、Python の起動(数十 ms の見込み。実測は実装時に確認する)を伴う。起動の費用を抑えるため、(1) `tj.py` は、上記の 4 つの組み込みモジュール以外を読まない(`site` を省く `-S` を付けてよい。`TJ` の定義の中では `"<sys.executable>" -S "<tj.py>"` と書く。効果は未確認で、付けるかどうかは実測で決める)、(2) 大量のジョブを要するテスト(AC-402 など)は、`TJ` を使わず、偽の `Deps` で行う、(3) 性能の測定では、`TJ` 自身の起動の費用を基準として測り、差し引く(2.1 節)。
- `--out <ファイル>` が先頭にあれば、**記録の行**をそのファイルに追記する(`open(file, "a", encoding="utf-8", newline="\n")` で 1 行 1 件)。なければ標準出力に書く。ジョブの標準出力を被試験体越しに読めない場合があるため、結果はファイルで受け取る。
- 記録に出す文字列(値)は、`q(s)` で表す: 二重引用符で囲み、`"` と `\` の前に `\` を置き、改行は `\n`、復帰は `\r`、タブは `\t` と書く(`json.dumps` とは別。非 ASCII はそのまま。他の制御文字は変換しない)。
- 未知のモードは、終了コード 2。
- 終わり方は、`sys.exit(n)` で終わる(`say` は、書き込みの `flush` を済ませてから終わる)。Python の `sys.exit(n)` は、UNIX 系で `n & 0xFF` になる(`exit 300` が 44)。

| モード | 動作 |
|--------|------|
| `exit <n>` | 終了コード n で終わる(省略・整数でないときは 2) |
| `run <label> <n>` | 記録 `ran=<label>` を出し、終了コード n(省略・整数でないときは 0) |
| `echo <args...>` | 各引数を、記録 `arg[i]=q(引数)`(i は 0 から)として 1 行ずつ出し、終了コード 0 |
| `env <NAME>...` | 各環境変数を、設定されていれば `env[NAME]=set:q(値)`、なければ `env[NAME]=unset` として出す(`NAME in os.environ`)。終了コード 0 |
| `cwd` | カレントディレクトリ(`os.getcwd()`)を `cwd=q(パス)` として出す。終了コード 0 |
| `pid` | 自分のプロセス識別子を `pid=<N>`(`os.getpid()`)として出す |
| `say <out> <err> [n]` | `<out>` を標準出力に、`<err>` を標準エラーに、改行を付けずにそのまま書き(`flush` して完了を待つ)、終了コード n(省略は 0)。`--out` に従わない |
| `sleep [秒]` | `pid=<N>` と `ready` を記録し、眠る(`time.sleep(0.05)` の繰り返し)。秒があれば、その秒のあと終了コード 99。なければ眠り続ける。SIGTERM・SIGINT・SIGHUP を受けたら、ハンドラが自分のシグナルのハンドラを `SIG_DFL` に戻し、`os.kill(os.getpid(), <受けたシグナル>)` で自分に同じシグナルを送り直す(シグナルで終了したように観測されるようにする。確認は実装時。10 章)。Windows では、この 3 つのうち登録できるものだけを登録する |
| `trap <ms> <code>` | `pid=<N>` と `ready` を記録する。SIGTERM・SIGINT・SIGHUP を受けたら、記録 `trapped` を出し、`<ms>` ミリ秒待ってから、記録 `done` を出し、終了コード `<code>` で終わる(シグナルで中断された後も、ジョブが終わるまで `conductor` が待つことを確かめる。AC-505) |
| `flaky <counter-file> <fails> <code>` | `<counter-file>` の整数(なければ 0)に 1 を足して書き戻す。足した値が `<fails>` 以下なら終了コード `<code>`、超えたら 0 |
| `timing` | `start_ns=<time.time_ns()>`、`end_ns=<time.time_ns()>` を、標準出力に書く(性能の測定用。`start_ns` は最初の行で、`end_ns` は終了の直前で取る) |
| `burn <ms>` | 指定のミリ秒(既定 100)、`time.process_time()` で測って CPU を使い続けて終わる(CPU 時間の測定用) |
| `stdin` | 標準入力(`sys.stdin.buffer`)を終わりまで読み、読んだ内容を UTF-8 で復号して(`errors="replace"`)記録 `stdin=q(内容)` として出し、終了コード 0(標準入力の引き継ぎの確認。AC-509) |
| `big <MiB>` | 標準出力(`sys.stdout.buffer`)に、指定の MiB(既定 16)の `x` を、書き込みの完了を待ちながら書き、終了コード 0(AC-494) |

補足:

- `sleep` モードで SIGQUIT を受けると、Python の既定の動作(コアダンプを伴う終了)になる(実装時に確認する)。したがって、AC-282 の SIGQUIT の項目は、`conductor` の終了コード 131 だけを確かめ、ジョブの終わり方は確かめない。
- UNIX 系の受け入れテストでは、シグナルで終わるジョブを `["sh", "-c", "kill -9 $$"]`(配列形式)で作る(`TJ` にモードを足さない)。
- `TJ` の `env`・`cwd` の記録の値は、`q(s)` で出すので、OS によらず同じ書式になる。`TJ` は、標準出力・標準エラーを `sys.stdout.buffer`・`sys.stderr.buffer` に UTF-8 で書く(Windows のコンソールの符号化に左右されない)。

`tests/jobs/tj.py` の骨組み(全モードの実装は、上の表のとおり。ここでは、共通部分と、`exit`・`flaky`・`sleep` の 3 つを示す):

```python
# Copyright 2026 agwlvssainokuni (Apache License 2.0 のヘッダ。全文は省略)
from __future__ import annotations

import os
import signal
import sys
import time

OUT_FILE: str | None = None


def q(s: str) -> str:
    out = s.replace("\\", "\\\\").replace('"', '\\"')
    out = out.replace("\n", "\\n").replace("\r", "\\r").replace("\t", "\\t")
    return '"' + out + '"'


def record(line: str) -> None:
    if OUT_FILE is None:
        sys.stdout.buffer.write((line + "\n").encode("utf-8", "surrogateescape"))
        sys.stdout.buffer.flush()
    else:
        with open(OUT_FILE, "a", encoding="utf-8", newline="\n") as f:
            f.write(line + "\n")


def to_int(s: str | None, default: int) -> int:
    try:
        return int(s) if s is not None else default
    except ValueError:
        return default


def mode_flaky(args: list[str]) -> int:
    path, fails, code = args[0], to_int(args[1], 0), to_int(args[2], 1)
    try:
        with open(path, encoding="utf-8") as f:
            n = to_int(f.read().strip(), 0)
    except FileNotFoundError:
        n = 0
    n += 1
    with open(path, "w", encoding="utf-8") as f:
        f.write(str(n))
    return code if n <= fails else 0


def mode_sleep(args: list[str]) -> int:
    record(f"pid={os.getpid()}")
    record("ready")

    def resend(signum: int, _frame: object) -> None:
        signal.signal(signum, signal.SIG_DFL)
        os.kill(os.getpid(), signum)

    for name in ("SIGTERM", "SIGINT", "SIGHUP"):
        if hasattr(signal, name):
            signal.signal(getattr(signal, name), resend)
    deadline = time.monotonic() + float(args[0]) if args else None
    while deadline is None or time.monotonic() < deadline:
        time.sleep(0.05)
    return 99


def main(argv: list[str]) -> int:
    global OUT_FILE
    if len(argv) >= 2 and argv[0] == "--out":
        OUT_FILE, argv = argv[1], argv[2:]
    if not argv:
        return 2
    mode, args = argv[0], argv[1:]
    if mode == "exit":
        return to_int(args[0] if args else None, 2)
    if mode == "flaky":
        return mode_flaky(args)
    if mode == "sleep":
        return mode_sleep(args)
    # run / echo / env / cwd / pid / say / trap / timing / burn / stdin / big は、上の表のとおり
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
```

(`sys.argv` の不正なバイトは孤立サロゲートになるので、`echo` の `q()` の出力は、`encode("utf-8", "surrogateescape")` で元のバイト列に戻して書く。)

### 3.5 定義とステートの略記と、実行環境

**実行環境**

- 被試験体(`conductor` の起動方法。3.3 節)と `TJ` を用意する。
- 各テストは、空の一時ディレクトリ(`T`)で行う。`T` の中に定義ファイル(既定の名前 `conductor.yaml`)を作り、`T` をカレントディレクトリにして被試験体を起動する。ステートファイルは、定義ファイルと同じディレクトリにできる(例: `T/conductor.yaml.state`)。
- 環境変数は、特に記さない限り、次のとおり。`LANG=C`。`LC_ALL`・`LC_MESSAGES`・`NO_COLOR`・`CONDUCTOR_RUN_ID`・`PYTHONPATH`(3.3 節の 3 の場合を除く)・`PYTHONSTARTUP`・`PYTHONHOME`・`PYTHONIOENCODING`・`PYTHONUTF8`・`PYTHONWARNINGS`・`PYTHONDEVMODE` は未設定にする(表示を英語に固定し、利用者の環境の影響を除く)。`PATH` と、Windows の `SystemRoot`・`ComSpec`・`PATHEXT`・`TEMP`・`TMP` は、親から引き継ぐ。`VIRTUAL_ENV` は、親から引き継いでよい(`sys.executable` が仮想環境の `python` なら、標準ライブラリは正しく見つかる)。
- 標準入力は空(`stdin=subprocess.DEVNULL`)にする。標準入力を使う項目は、`PIPE` にして書き込む。
- パスに空白や非 ASCII を含めない(特に記す場合を除く)。

**定義の略記** — 前提の「定義」は、次の略記で書く。

```text
P(<パイプライン名> [run_id_env=<ENV>] [remove_on_complete=<true|false>]; <ジョブ>, <ジョブ>, ...)
```

| 略記 | 展開 |
|------|------|
| `P(p; ...)` | `format_version: 1`、`pipeline:` の `name: p`、続いて `job:` の列(`- id: ...`)を、書いた順に並べる |
| `<id>:<n>` | `job` の要素に `id: <id>`、`command: "TJ exit <n>"` |
| `<id>:<n>{...}` | 同上に、波括弧の中の修飾を足す |
| `w=[…]` | `warning_exit_codes: […]` |
| `s=[…]` | `success_exit_codes: […]` |
| `on=success->X` / `on=warning->X` | `transitions:` に `- on: success`(または `warning`)と `next: X`(X はジョブID か `@end`。**`@end` は YAML では `"@end"` と二重引用符で囲む**。`@` で始まる値は、引用符なしでは書けない) |
| `retry=(exit_codes=[…], max=N, interval="…", multiplier=M, max_interval="…")` | `retry:` の写像(書いたキーだけ) |
| `cmd=<文字列>` | `command` を、その文字列(YAML の文字列。`TJ` は 3.4 節の 2 つの語。必要なら二重引用符で囲む)にする。`id:n` の `n` は使わない |
| `argv=[…]` | `command` を、その文字列の列(YAML の流れ形式の列)にする。`TJ` を書くときは、`sys.executable` と `tj.py` のパスを、別々の要素にする |
| `O` | 記録の出力ファイル(`T/out.txt` など。`--out` に渡す) |

例: `P(p; a:0, b:7{w=[3]}, c:0{on=success->@end})` は、次の定義(`TJ` は、3.4 節の 2 つの語の略)。

```yaml
format_version: 1
pipeline:
  name: p
job:
  - id: a
    command: "TJ exit 0"
  - id: b
    command: "TJ exit 7"
    warning_exit_codes: [3]
  - id: c
    command: "TJ exit 0"
    transitions:
      - on: success
        next: "@end"
```

JSON の定義(`conductor.json`)で試験する項目は、同じ構造を JSON のオブジェクトで書く(例: `{"format_version": 1, "pipeline": {"name": "p"}, "job": [{"id": "a", "command": "TJ exit 0"}]}`)。キー名と構造は、YAML と同じ([03](03-definition-format.md))。

**ステートの略記** — ステートの期待は、次の略記で書く。`end` と `completed` は、書かなければ `false`。`resume` は、書かなければ「キーがない」。

```text
S[<id>=<状態>(<終了コード>), ... ; resume=<id>, end=<true|false>, completed=<true|false>, run_id=<id>]
```

状態は `pending`・`success`・`warning`・`error`・`skipped`。スキップは `skipped/byTransition` か `skipped/byEnd`。終了コードがなければ括弧を省く。例: `S[a=success(0), b=error(7), c=pending ; resume=b]`。ステートファイルの書式(JSON)は [06](06-state-file.md)。受け入れテストは、ステートファイルを(テストでは)`json.loads` で読み、論理項目(`pipeline_name`、`job_order`、`job` の各要素の `id`・`status`・`skip_reason`・`exit_code`、`resume_job_id`、`end_reached`、`completed`、`run_id` など)を比べる。`json.loads` を使ってよいのは、テストと開発用のスクリプトだけである(製品のコードは自前の読み取りを使う。AC-421。03 章が、製品のコードで `json` の一部の機能を使ってよいと定めた場合は、AC-421 の禁止の範囲を、その規定に合わせる。10.2 節)。

### 3.6 判定の書き方と区分

| 欄 | 意味 |
|----|------|
| 実行 | 被試験体の引数。`T` をカレントディレクトリにする。`--config` を省くと `T/conductor.yaml` |
| 終了 | 終了コード |
| OUT | 標準出力。「空」は 0 バイト。引用は完全一致。「含む」は部分一致 |
| ERR | 標準エラー。同上。複数行は `⏎` で区切って書く(行の終わりの改行は省く) |
| 前提 | 実行の前に作るもの(定義、ステート、環境変数) |

- どのテストも、**30 秒の上限**で実行する(ハングの検知)。
- 「ジョブが実行された」は、`TJ run <label> <n>` の記録 `ran=<label>` で確かめる(`O` を読む)。「実行されない」は、その記録がないこと。
- 「ステートが変わらない」は、ステートファイルの内容(バイト列)が、実行の前後で同じこと。「ステートがない」は、ファイルがないこと。
- 「UNIX 系」「Windows」「非 root」と書かれた項目は、その環境でだけ行う(3.10 節のマーク)。

| 区分 | 意味 |
|------|------|
| (自動) | 被試験体を外から実行して確かめる |
| 単体 | 内部の関数(分割、終了コードの範囲合わせ、エスケープ、判断の純粋な関数)を直接呼んで確かめる。外から再現しにくい入力のため |
| 注入 | 標準出力への書き込みの失敗、シグナル登録の失敗など、外から作れない失敗を、偽の `Deps`(3.7 節)で起こして確かめる。外から作れる範囲では、**テスト専用のラッパースクリプト**(`tests/support/inject_*.py`)を `python` の引数として起動し、ラッパーが失敗の仕掛けをしてから `runpy.run_module("conductor", run_name="__main__")` で被試験体を動かす(製品に試験用の経路を持たせない。AC-490) |
| 点検 | ソースや依存を、`ast` モジュール(`ast.parse` と構文木の走査)、`pyproject.toml` の文面の走査、ビルドした sdist・wheel の中身(`zipfile`・`tarfile`)で調べて確かめる(実行ではない。`tests/inspect/` に置く) |
| 手動 | 人が実機で確かめて、記録を残す |
| 計測 | 性能の測定(2.1 節) |

### 3.7 偽の部品(`Deps`)を使うテスト

`src/conductor/cli/` の `Deps` は、次の役割を `typing.Protocol` で持つ([02](02-architecture.md))。`tests/support/fakes.py` が、それぞれの偽物を提供する。

| 役割(`Deps` のフィールド) | 本物 | 偽物(`tests/support/fakes.py`。名前は 02 章 4.4 節) |
|-------------------------------|------|-------------------------------|
| 環境(`env`) | `os.environ`、`sys.stdout.isatty()`、OS の判定(`runner/platform.py`) | `FakeEnv`(環境変数の表。端末か・OS の種別を設定できる) |
| 時刻(`clock`) | `datetime.now(timezone.utc)` など | `FakeClock`(固定の時刻) |
| 定義(`defs`) | `definition` の読み込み、run ID の環境変数名の緩い読み取り | `FakeDefs`(決めた結果を返す。または一時ディレクトリの実ファイルを読む) |
| ステート(`states`) | `state/`(`os.replace` による原子的な書き込み) | `MemStates`(メモリ上のステート。読み込み・書き込み・削除の失敗を起こせる) |
| 変換(`converter`) | `convert` の変換の入口 | `FakeConverter` |
| プロセス実行・待ち(`proc`) | `runner/`(ジョブの実行、リトライの待ち、`command` の検査) | `ScriptProcess`(用意した結果を順に返す。起動の回数と引数を記録。待ちは、待ち時間(秒の `int`)を記録するだけで、すぐに返す。中断を返す設定がある) |
| シグナル(`signals`) | `signal.signal` による登録と、旗の読み取り | `FakeSignals`(任意の時点で番号を立てる。準備の失敗・「登録済み」を返せる) |
| 標準出力・標準エラー(`out`) | `sys.stdout` / `sys.stderr` | `BufSink`(文字列を溜める。「n 回目の書き込みで失敗する」設定で、書き込みが失敗する(`BrokenPipeError` や `UnicodeEncodeError` を投げる)場合を作る) |

偽の部品で確かめる項目(区分「注入」)は、AC-293〜300、AC-324、AC-366〜368、AC-485、AC-491 など。`cli/` の流れのテストは、偽の部品の上で、`run`・`pipeline`・`watch`・`reset`・`convert`・`validate` を呼び、終了コード・標準出力・標準エラー・`MemStates` のステートを比べる。
本物の部品との結合は、受け入れテスト(3.3 節)が確かめる。

### 3.8 テストの補助(`tests/support/harness.py`)

| 関数 | 役割 |
|------|------|
| `make_temp_dir()` | (`tmp_path` を使う fixture が、代わりに担う) |
| `default_env(extra=None)` | 3.5 節の既定の環境変数に `extra` を足した `env` を返す |
| `write_definition(T, name, source)` | 略記(3.5 節)の定義を、YAML か JSON で書き出す。文字列は `json.dumps(…, ensure_ascii=False)` で出す |
| `run_conductor(args, *, cwd, env, stdin=None, timeout=30.0)` | 被試験体を `Popen` し、`Result(code, stdout, stderr)`(`code` は `returncode`。シグナルで死ねば負の値。`stdout`・`stderr` は `bytes` を UTF-8 で復号した `str`、`errors="surrogateescape"`)を返す。30 秒で強制終了して失敗にする。`communicate()` で、終了まで読み切る |
| `start_conductor(args, ...)` | 実行中に操作する項目(シグナルを送る、標準エラーの行を待つ)のために、`Popen` と、行ごとの待ち受け(`wait_for_stderr(pattern, timeout)`。読み取りは、テスト側のスレッドを 1 つ使ってよい)を返す |
| `read_records(file)` | `TJ` の記録の行を読む |
| `read_state(T, name)` | ステートファイルを読んで、`json.loads` した値を返す |
| `list_dir(T)` | `T` のファイル名の一覧(ソート済み) |

### 3.9 pytest の設定

`pyproject.toml` の `[tool.pytest.ini_options]`(キー名は採用した版の文書で確かめる。**この文書の作成時には実行していない**):

```text
[tool.pytest.ini_options]
minversion = "8.0"
testpaths = ["tests"]
pythonpath = ["src", "."]
addopts = ["-ra", "--strict-markers", "--strict-config"]
markers = [
  "acceptance: 外から実行して確かめる",
  "unix: UNIX 系だけ",
  "windows: Windows だけ",
  "linux: Linux だけ",
  "non_root: root 以外",
  "perf: 性能の測定(CONDUCTOR_PERF=1)",
]
filterwarnings = ["error"]
faulthandler_timeout = 120
```

| 項目 | 決め | 理由 |
|------|------|------|
| `testpaths`・`pythonpath` | `tests`。`src` と `.`(`tests.support` を import するため。`tests/` と `tests/support/` に `__init__.py` を置く) | 編集可能なインストールがなくても、単体テストが動くようにする |
| `--strict-markers` | 未登録のマークを誤りにする | マークの綴りの誤りで、`skipif` が効かなくなることを防ぐ |
| `filterwarnings = ["error"]` | テストの中で出た警告を、すべて誤りにする | `DeprecationWarning`・`ResourceWarning` の見逃しを防ぐ(AC-498)。`ResourceWarning`(閉じていないファイル)は、ガベージコレクションの時に出るので、取りこぼす場合がある(未確認) |
| タイムアウト | `pytest-timeout` は使わない。子プロセスを起動するテストは、`run_conductor` の 30 秒で自分で止める。`faulthandler_timeout = 120` は、止まったテストのスタックを標準エラーに出すだけ | 依存を増やさない(8 章) |
| 並列 | しない(3.1 節) | 時間の基準を安定させる |
| `tmp_path` | テストごとの別のディレクトリ。pytest が直近の数回分を残す(基準の数は版による。実装時に確認) | ステートの衝突を避ける |
| カバレッジ | `pytest-cov`(`coverage`)。対象は `src/conductor/`。除外は `src/conductor/__main__.py`(入口。薄い。受け入れテストが通す)。**受け入れテストで起動した子プロセスの中のコードは、数えない**(子のプロセスのカバレッジの収集は、この仕様では行わない)。下限は、行 80% | 5.3 節 |
| スナップショット | 使わない(期待は、文言そのものをテストに書く) | 文言の変更に気づくため |
| 性能の試験 | `tests/perf/conftest.py` が、`CONDUCTOR_PERF` が `1` でなければ、モジュールごとスキップする | 通常の実行を遅くしない |

カバレッジの設定(`pyproject.toml`。キー名は採用した版で確かめる):

```text
[tool.coverage.run]
source = ["conductor"]
branch = false
omit = ["*/conductor/__main__.py"]

[tool.coverage.report]
fail_under = 80
show_missing = true
```

### 3.10 OS 別のテスト

- UNIX 系だけのテスト(シグナル、権限、シンボリックリンク、新しいプロセスグループ、`pty`)は、`@pytest.mark.skipif(sys.platform == "win32", reason="UNIX 系だけ")`。Windows だけのテスト(`.cmd` の安全策、Ctrl+C のチェックリストに対応する自動の部分)は、`skipif(sys.platform != "win32", ...)`。Linux だけのテスト(`/dev/full`、`/proc` による CPU 時間)は、`skipif(not sys.platform.startswith("linux"), ...)`。これらは `tests/support/marks.py` に、`unix_only`・`windows_only`・`linux_only` として定義して使う(`sys.platform` の判定が、テストの中に散らばらないように)。
- 非 root のテスト(権限の拒否を起こす項目)は、`non_root = pytest.mark.skipif(hasattr(os, "geteuid") and os.geteuid() == 0, reason="root では権限の拒否が起きない")` を加える。`os.geteuid` は Windows にない(属性がない)ので、`hasattr` で調べる。
- `skipif` でスキップされた項目は、結果に「skipped」として残る(`-ra` で理由を出す)。CI では、OS ごとにスキップされる件数が、想定の範囲であることを、実装時に確認する。
- OS 固有の製品のコードは、`src/conductor/runner/platform.py` に集める。他のファイルから見ると、OS の違いは見えない(AC-420)。
- どの OS でも、`mypy --strict` と `ruff` が通る(`sys.platform` の比較は、`mypy` が OS ごとに到達できないコードと見なす場合がある。`sys.platform == "win32"` の分岐の中でだけ使える属性(`signal.SIGBREAK`、`signal.CTRL_BREAK_EVENT`、`subprocess.CREATE_NEW_PROCESS_GROUP` など)は、`getattr` または `sys.platform` の分岐の形で書き、3 OS の CI の `mypy` が通ることを確かめる。実装時に確認)。

### 3.11 テストの実行

```sh
python -m venv .venv && . .venv/bin/activate      # Windows は .venv\Scripts\activate
python -m pip install -c constraints.txt -e ".[dev]"    # 制約ファイルのとおりに入れる
pytest                                            # 単体 + 受け入れ + 点検(3 OS で通る)
pytest --cov=conductor --cov-report=term-missing --cov-report=json   # 行 80%(Linux)
CONDUCTOR_PERF=1 pytest tests/perf                # 性能の測定(2.1 節)
CONDUCTOR_BIN=<配布物のコンソールスクリプト> pytest tests/acceptance      # 配布物の試験(AC-446)
CONDUCTOR_PYTHON=<配布物を入れた仮想環境の python> pytest tests/acceptance
```

`pyproject.toml` の `[project.optional-dependencies]` の `dev`(8.4 節)に、テストと検査の道具が入っている。

---

## 4. 受け入れテスト一覧

ID は、`AC-` と通し番号。表は、ID・機能・前提・実行・期待の順。期待の標準エラーの文言は、特に記さない限り `LANG=C`(英語)。理由の文(日本語の固定の文)は、英語の表示でも日本語のまま。
YAML と JSON に固有の項目は 4.19 節(AC-450〜487)、Python に固有の項目は 4.20 節(AC-488〜531)にまとめた。この節の定義は、特に記さない限り YAML の `conductor.yaml`。

### 4.1 定義の検査(`validate`、`run`、`pipeline` 共通)

特に記さない限り、実行は `validate`(`--config` を省く)。「64」の期待は、終了 64、OUT 空、ステートファイルを作らない、ジョブを起動しない、を含む。
ERR の期待は、**最初の行**のうち、`conductor: error: <ファイル>[:<行>]: ` に続く種類の文([08](08-output-and-i18n.md))を示す。ファイル名は、`--config` に渡した表記のまま。行は、誤りの位置の行(1 始まり)。YAML の誤りの行は、採用した YAML ライブラリが返す位置(多くは 0 始まり)を 1 始まりに直した値で、ライブラリの検出位置に依存する項目は、10 章に挙げる。
基本の定義 `B0` は、`P(p; a:0)` とする。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-001 | 正しい定義 | `B0` | `validate` | 終了 0。OUT・ERR 空。`T/conductor.yaml.state` を作らない |
| AC-002 | validate の -v(1 ジョブ) | `B0` | `validate -v` | 終了 0。ERR = `[conductor] definition is valid (1 job)`。OUT 空 |
| AC-003 | validate の -v(複数) | `P(p; a:0, b:0, c:0)` | `validate -v` | 終了 0。ERR = `[conductor] definition is valid (3 jobs)` |
| AC-004 | validate の -v(日本語) | `P(p; a:0, b:0, c:0)`、`LANG=ja_JP.UTF-8` | `validate -v` | 終了 0。ERR = `[conductor] 定義は正しいです(ジョブ 3 個)` |
| AC-005 | 定義ファイルがない | ファイルなし | `validate --config missing.yaml` | 64。ERR = `conductor: error: missing.yaml: definition file does not exist: 指定の場所にファイルがありません` |
| AC-006 | 定義ファイルがない(日本語) | 同上、`LANG=ja_JP.UTF-8` | 同上 | 64。ERR = `conductor: error: missing.yaml: 定義ファイルが存在しません: 指定の場所にファイルがありません` |
| AC-007 | 通常のファイルでない | `T/d.yaml` がディレクトリ | `validate --config d.yaml` | 64。ERR の種類の文 = `cannot read definition file`(理由 `通常のファイルではありません`) |
| AC-008 | 空のファイル(YAML) | 長さ 0 の `conductor.yaml`(コメントだけのファイルも同じ) | `validate` | 64。ERR = `conductor: error: conductor.yaml: format_version is missing: …`(行なし) |
| AC-009 | 文法の誤り(行つき) | `format_version: 1⏎pipeline: ]⏎` | `validate` | 64。ERR の最初の行が `conductor: error: conductor.yaml:2: syntax error: ` で始まる |
| AC-010 | キーの重複(YAML。文法の誤り) | `format_version: 1⏎format_version: 1⏎` | `validate` | 64。種類の文 = `syntax error`(行 2) |
| AC-011 | 書式の版が 1 でない | `format_version: 2` に `B0` の他を足す | `validate` | 64。`unsupported format_version` |
| AC-012 | 書式の版が整数でない | `format_version: "1"`、`format_version: 1.0`、`format_version: true`(Python では、`True` が `1` と等しいので、`bool` を整数として受理しないことの確認を兼ねる。AC-502) | `validate` | 64。`unsupported format_version` |
| AC-013 | 版の確認が先 | `format_version` なし、未知のキーと型の誤りを含む | `validate` | 64。ERR は 1 行だけ。種類の文 = `format_version is missing` |
| AC-014 | `jobs`(複数形)のキー | `B0` の `job:` を `jobs:` に | `validate` | 64。`unknown key` |
| AC-015 | 未知のキー(ジョブ) | `B0` の job に `colour: red` | `validate` | 64。`unknown key`(行はそのキーの行) |
| AC-016 | 未知のキー(パイプライン) | `pipeline:` に `owner: x`(または `remove_on_complet: true`) | `validate` | 64。`unknown key` |
| AC-017 | `pipeline` がない | `format_version: 1` と `job:` だけ | `validate` | 64。`wrong value type`(行 1) |
| AC-018 | `pipeline` が写像でなく列 | `pipeline: [{name: p}]` | `validate` | 64。`wrong value type` |
| AC-019 | パイプライン名が空 | `name: "  "` | `validate` | 64。`required value is empty` |
| AC-020 | ジョブがない | `job` を書かない(または `job: []`) | `validate` | 64。`no jobs are defined` |
| AC-021 | ジョブIDの重複 | `P(p; a:0, a:0)` | `validate` | 64。`duplicate job id` |
| AC-022 | ジョブIDが不正 | ジョブID が `""`、または `"@x"` | `validate` | 64。`invalid job id` |
| AC-023 | 遷移先がない | `P(p; a:0{on=success->nowhere})` | `validate` | 64。`transition target job is not defined` |
| AC-024 | エラー結果への遷移 | `transitions` の `on: error` | `validate` | 64。`no transition can be written for the error result` |
| AC-025 | `on` の値が不正 | `on: later`、または `on: Success` | `validate` | 64。`wrong value type` |
| AC-026 | 自分自身・前のジョブへの遷移 | `P(p; a:0{on=success->a}, b:0)`、`P(p; a:0, b:0{on=success->a})` | `validate` | 64。`transition target must be a later job or @end` |
| AC-027 | `@end` への遷移は正しい | `P(p; a:0{on=success->@end}, b:0)` | `validate` | 0 |
| AC-028 | 同じ結果の遷移の重複 | `on: success` の遷移を 2 つ | `validate` | 64。`duplicate transition for the same result` |
| AC-029 | 0 を警告にできない | `P(p; a:0{w=[0]})` | `validate` | 64。`0 cannot be a warning exit code` |
| AC-030 | 成功と警告の重なり | `P(p; a:0{w=[3], s=[0,3]})` | `validate` | 64。`success and warning exit codes overlap` |
| AC-031 | 終了コードの型・範囲の誤り | `w=["x"]`、`w=[1.5]`、`w=[true]`(`bool` は整数ではない)、`w=[1.0]`(`1.0` は整数ではない)、`w=[99999999999999999999]`、`w=[2147483648]`、`w=[-2147483649]` | `validate` | 64。`wrong value type`(要素ごとに 1 行)。範囲の検査は、整数の字句から行う(`float` に落とさない。`True`・`1.0` を `int` と見なさない。AC-502) |
| AC-032 | 負の終了コード・空の成功コード・範囲の端 | `w=[-1]`、`s=[]`、`w=[2147483647]`、`s=[-2147483648]` | `validate` | 0(受理する) |
| AC-033 | `command` の型・欠落・空 | `command` なし / `""` / `"   "` / `[]` / `[""]` / `["  ", "x"]` / `5` / `["a", 5]` / `true` | `validate` | 64。なし・`5`・`["a",5]`・`true` は `wrong value type`。ほかは `required value is empty` |
| AC-034 | 配列形式の `command` の受理 | `argv=["my prog", "a b", "$HOME", ""]`、`argv=["echo", ""]` | `validate` | 0 |
| AC-035 | `run_id_env` の名前の形 | `run_id_env` が `"1A"` / `"A-B"` / `""` / `"A B"` / `"A=B"` / `"A.B"` | `validate` | 64。`invalid environment variable name`。整数 `1` なら `wrong value type`。`_a1`・`GITHUB_RUN_ID`・`A` は 0 |
| AC-036 | `remove_on_complete` の型 | `"true"` / `1` / `[true]` / `yes` | `validate` | 64。`wrong value type`(`yes` は YAML の真偽値ではなく文字列)。`true`・`false`・省略は 0 |
| AC-037 | 誤りは位置の順に 1 行ずつ | `ME`(この節の末尾の定義) | `validate` | 64。ERR は 4 行。順に `…:2: unknown key`、`…:10: wrong value type`、`…:13: unknown key`、`…:16: transition target job is not defined` |
| AC-038 | 行番号は CRLF でも同じ | `ME` を CRLF 改行で | `validate` | 同上(行番号が同じ) |
| AC-039 | BOM つきの定義 | `B0` の先頭に UTF-8 の BOM(`b"\xef\xbb\xbf"`。Python の `utf-8` の復号器は BOM を取り除かない。`utf-8-sig` を使うか、自前で取り除く) | `validate` | 0 |
| AC-040 | 定義の大きさの上限(受理) | `B0` に、コメント行(`#` で始まる行)を足して、ちょうど 1,048,576 バイト | `validate` | 0 |
| AC-041 | 定義の大きさの上限(超過) | 1,048,577 バイト(UTF-8 でなくてもよい) | `validate` | 64。`definition file is too large`(読み込む前に、サイズを見る。全体をメモリに読んでから比べない) |
| AC-042 | UTF-8 でない定義 | 上限以内で、不正なバイト列を含む | `validate` | 64。`cannot read definition file`(復号は `bytes.decode("utf-8")` の厳格な既定(`errors="strict"`)で、不正なバイト列を置換せず拒否する。`surrogateescape` や `replace` を使わない) |
| AC-043 | 入れ子の深さ(受理) | `a: ` に `[` を 32 個、`]` を 32 個続けただけのファイル(`format_version` なし。根のマップは数えず、32 段まで受理。[03](03-definition-format.md)) | `validate` | 64。`format_version is missing`(深さの検査は通った) |
| AC-044 | 入れ子の深さ(超過) | `a: ` に `[` を 33 個・1,000 個・100,000 個(閉じも書く) | `validate` | 64。`syntax error`(理由 `入れ子が深すぎます(上限は 32 段)`。行は 33 段目の `[` の行)。`RecursionError` が漏れない(AC-524) |
| AC-045 | パイプライン名の照合 | `P(p; a:0)` | `validate p` / `validate wrong` / `validate` | 0 / 64(`pipeline name does not match the definition`)/ 0(照合しない) |
| AC-046 | 名前の不一致の誤りに値を含めない | `name: SECRET-NAME` | `validate other-name` | 64。ERR に `SECRET-NAME` も `other-name` も現れない |
| AC-047 | 誤りの文に入力の値を含めない | `format_version: 1`、`pipeline:` の `name: p`、`job:` の `id: a`、`command: 5`、`warning_exit_codes: ["SENTINEL-VALUE"]` | `validate` | 64。ERR に `SENTINEL-VALUE` が現れない |
| AC-048 | `command` の引用符が閉じていない | `P(p; a:0{cmd="TJ exit 0 'x"})` | `validate` | 64。ERR = `conductor: error: conductor.yaml: job 'a': command has an unclosed quote` |
| AC-049 | `command` が空になる | `cmd="''"` | `validate` | 64。`job 'a': command is empty` |
| AC-050 | `command` の誤りは全ジョブ報告 | 2 つのジョブがそれぞれ誤り | `validate` | 64。ERR は 2 行(ジョブごと) |
| AC-051 | `command` の誤り(日本語) | 同上(1 つ目の形)、`LANG=ja_JP.UTF-8` | `validate` | ERR = `conductor: error: conductor.yaml: job 'a': command の引用符が閉じていません` |
| AC-052 | `run` でも `command` の誤りは 64 でジョブは起動しない | 1 つ目が正しく、2 つ目が誤り | `run a` | 64。`ran=` の記録なし。ステートなし |
| AC-053 | 起動できないプログラムでも validate は 0 | `cmd="/no/such/program"` | `validate` | 0 |
| AC-054 | 壊れたステートでも validate は 0 | `B0`、`T/conductor.yaml.state` に `{garbage` | `validate p` | 0。ステートは変わらない |
| AC-055 | validate の引数の誤り | `B0` | `validate a b` / `validate --run-id x` / `validate --to yaml` | 68 |
| AC-056 | `run_id_env` が不正な定義は run でも 64 | `run_id_env: "1A"` | `run a` | 64 |
| AC-057 | validate は副作用を持たない | `B0` の `command` が「`TJ run a 0`」(`O` に記録) | `validate` | 0。`O` が作られない。`T` のファイル一覧が実行の前後で同じ |

#### `retry` の検査

`R0` は、`P(p; a:75{retry=(exit_codes=[75], max=3, interval="1s")})`。各行は、`R0` の `retry` を変えて `validate` する。誤りは終了 64、種類の文は ERR の最初の行。

| ID | 機能 | 変更後の `retry` | 期待 |
|----|------|------------------|------|
| AC-058 | 受理(固定の間隔) | `R0` のまま | 0 |
| AC-059 | 受理(倍率と上限) | `multiplier: 2`、`max_interval: 1m` を足す | 0 |
| AC-060 | 受理(境界) | `exit_codes: [1,255,127]`、`max: 1` と `max: 100`、`interval` が `1s`・`24h`・`86400s`・`1440m`、`multiplier` が 2 と 10、`max_interval` が `interval` と同じ | いずれも 0 |
| AC-061 | 受理(流れ形式の写像) | `retry: { exit_codes: [75], max: 3, interval: 1s }` | 0 |
| AC-062 | `retry` が写像でない | `retry: 5`、または `retry: [{exit_codes: [75], max: 3, interval: 1s}]` | 64。`retry must be a map` |
| AC-063 | 必須のキーがない | `retry: {}` | 64。`a required key of retry is missing` が 3 行(`exit_codes`、`max`、`interval` の順) |
| AC-064 | 未知のキー | `retries: 2` を足す | 64。`unknown key` |
| AC-065 | `exit_codes` が空 | `exit_codes: []` | 64。`retry exit_codes is empty` |
| AC-066 | `exit_codes` の要素の範囲 | `exit_codes: [0, 75, 256]` | 64。`retry exit_codes has an invalid exit code` が 2 行(0 と 256。要素ごと)。`-1`、`1.5`、`"75"`、`true`、`75.0`、`99999999999999999999` も同じ |
| AC-067 | 成功・警告の終了コードと重なる | `{s=[0,5]}` で `exit_codes: [5,75]`、`{w=[4]}` で `exit_codes: [4]` | 64。`retry exit_codes overlap the success or warning exit codes` |
| AC-068 | 最大回数の範囲 | `max: 0`、`101`、`-1`、`1.5`、`"3"`、`true`、`4294967296` | 64。`retry max is invalid` |
| AC-069 | 間隔の書式 | `interval` が `10`(整数)/ `1.5s` / `0s` / `25h` / `1S` / `86401s` / `1ms` / `" 1s"` / `"1 s"` | 64。`retry interval is invalid`(`max_interval` も同じ規則) |
| AC-070 | 倍率の範囲 | `multiplier` が `1`、`11`、`0`、`2.0`、`"2"`、`true`(`max_interval` を添える) | 64。`retry multiplier is invalid` |
| AC-071 | 倍率と上限は片方だけ不可 | `multiplier: 2` だけ、`max_interval: 1m` だけ | 64。`retry multiplier and max_interval must be given together or not at all` |
| AC-072 | 上限が間隔より小さい | `interval: 1m`、`multiplier: 2`、`max_interval: 30s` | 64。`retry max_interval is smaller than interval` |
| AC-073 | 誤りは位置の順、ほかの誤りと一緒に | `retry` の誤り 2 つと、別の定義の誤りを混ぜる | 64。ERR は位置の順に 1 行ずつ |
| AC-074 | `run` と同じ誤りの文 | 上のいずれかの誤りの定義で `run a` を実行 | `validate` と同じ文で 64 |
| AC-075 | 誤りの文に入力の値を含めない | `interval: SENTINEL-VALUE` で `validate` | 64。ERR に `SENTINEL-VALUE` が現れない |

`ME`(誤りが複数ある定義。行番号は、この並びのとおり):

```yaml
format_version: 1
unknown_top: true

pipeline:
  name: p

job:
  - id: a
    command: x
    success_exit_codes: ["zero"]
  - id: b
    command: y
    colour: red
    transitions:
      - on: success
        next: nowhere
```

JSON で同じ構造を書いた `ME`(`conductor.json`)では、誤りの行は、順に 3、11、16、20 になる(AC-475)。

### 4.2 `command` の分割と起動

「分割」の行は、`P(p; a:0{cmd="TJ --out O echo <X>"})` を作り、`run a` を実行する(`TJ`、`O` は 3.5 節)。`<X>` は、各行の「`command` の引数部」。
期待は、終了 0、OUT・ERR 空、`O` の記録(`arg[0]=…` 以降)。記録は `q(…)` の形(3.4 節)。ここでは、引数の値だけを書く(`arg[i]` の添字は先頭から 0)。YAML に書くときは、`json.dumps` で出した二重引用符の文字列(3.4 節)で書き、文字列として同じ値になるようにする。分割は、`shlex` を使わず自前で行う([05](05-execution-flow.md)、[07](07-process-and-signals.md)。`shlex` の規則とは、バックスラッシュの扱いが違う)。

| ID | 機能 | `command` の引数部(`TJ --out O echo` の後ろ) | 期待する引数の並び |
|----|------|------------------------------------------------|---------------------|
| AC-076 | 分割: 基本 | `a b` | `a`、`b` |
| AC-077 | 分割: 空白・タブ・改行の混在 | `␣␣a ⇥ b ⏎ c␣␣` | `a`、`b`、`c` |
| AC-078 | 分割: 一重引用符 | `'a b' 'c\d' '"x"'` | `a b`、`c\d`、`"x"` |
| AC-079 | 分割: 二重引用符と `\"` | `"a \" b"` | `a " b` |
| AC-080 | 分割: 二重引用符の中の `\n` `\\` | `"a\nb" "c\\d"` | `a\nb`、`c\\d`(バックスラッシュは、そのまま。`\n` は 2 文字) |
| AC-081 | 分割: `\\"` は引用符を閉じない | `"\\"x"` | `\"x` |
| AC-082 | 分割: 引用符の外のバックスラッシュ | `a\ b` | `a\`、`b` |
| AC-083 | 分割: 末尾が `\` のパスは一重引用符 | `'C:\dir\'` | `C:\dir\` |
| AC-084 | 分割: `--msg='a b'` | `--msg='a b'` | `--msg=a b` |
| AC-085 | 分割: 引用符のつなぎ | `a"b c"'d e'f` | `ab cd ef`(1 つ) |
| AC-086 | 分割: 空の引数 | `'' "" x` | (空)、(空)、`x` |
| AC-087 | 分割: 非 ASCII | `'あ い' う` | `あ い`、`う` |
| AC-088 | 分割: メタ文字は解釈しない | `a;b && c \| d > e $HOME * `` `x` `` (y)` | `a;b`、`&&`、`c`、`\|`、`d`、`>`、`e`、`$HOME`、`*`、`` `x` ``、`(y)`。`T` に `e` というファイルができない |
| AC-089 | 分割: 区切りは 4 文字だけ | `a<U+00A0>b`(U+00A0 は空白に見えるが区切りではない。Python の `str.split()` は U+00A0 でも分けるので、引数なしの `split()` で分割しない) | `a<U+00A0>b`(1 つ) |
| AC-090 | 配列形式: 要素はそのまま | `argv=[<TJ の 2 要素>, "--out", "O", "echo", "a b", "  lead and trail  ", "$HOME", "a;b", "*", ""]` | `a b`、`  lead and trail  `、`$HOME`、`a;b`、`*`、(空) |
| AC-091 | 文字列と配列は同じ結果 | `P(p; a:0{cmd="TJ --out O run a 0"}, b:0{cmd="TJ --out O run b 7"})` と、同じ内容の `argv` 形式 | それぞれ `run a` → 0、`pipeline p` → 7、OUT・ERR 空。`O` = `ran=a`⏎`ran=b` が同じ |

分割の誤り(拒否)。実行は `validate`。終了 64。ERR の最初の行は `conductor: error: conductor.yaml: job 'a': <文>`。

| ID | 機能 | `command`(文字列) | 期待する文 |
|----|------|---------------------|------------|
| AC-092 | 拒否: 一重引用符が閉じていない | `echo 'a` | `command has an unclosed quote` |
| AC-093 | 拒否: 二重引用符が閉じていない | `echo "a` | 同上 |
| AC-094 | 拒否: 末尾が二重引用符の中の `\` | `echo "a\` | 同上 |
| AC-095 | 拒否: `\\"` で閉じない | `echo "a\\" b` | 同上 |
| AC-096 | 拒否: パスの末尾の `\"` | `prog "C:\dir\"` | 同上 |
| AC-097 | 拒否: 空の語だけ | `''`、`'' x` | `command is empty` |
| AC-098 | 拒否: 区切りだけ | `"  \t\n"`(YAML の二重引用符の文字列)。定義の検査が、空白だけの `command` を先に拒否する | `required value is empty`(定義の検査。64) |

起動と実行環境。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-099 | 環境変数の引き継ぎ | `P(p; a:0{cmd="TJ --out O env CT_VAR CT_NO_SUCH"})`。環境に `CT_VAR=hello`、`CT_NO_SUCH` は未設定 | `run a` | 終了 0。`O` = `env[CT_VAR]=set:"hello"`⏎`env[CT_NO_SUCH]=unset` |
| AC-100 | 作業ディレクトリは引き継ぐ | `cmd="TJ --out O cwd"`。定義は `T/sub/x.yaml` | `T` で `run a --config sub/x.yaml` | `O` = `cwd=q(T のパス)`(`T/sub` ではない) |
| AC-101 | run ID を環境変数として渡さない | `cmd="TJ --out O env CONDUCTOR_RUN_ID"` | `run a --run-id r1` | `O` = `env[CONDUCTOR_RUN_ID]=unset` |
| AC-102 | 標準出力・標準エラーはそのまま | `cmd="TJ say 'OUT to stdout' 'ERR to stderr' 0"` | `run a` | 終了 0。OUT = `OUT to stdout`(改行なし、完全一致)。ERR = `ERR to stderr`(完全一致) |
| AC-103 | 存在しないプログラム | `cmd="/no/such/program-for-conductor-test"` | `run a` | 終了 127。ERR に `conductor: error: job 'a': cannot start command '/no/such/program-for-conductor-test'`。OUT 空。ステート `S[a=error ; resume=a]`(`Popen` が投げる `FileNotFoundError` を、起動失敗として扱う) |
| AC-104 | 配列形式の起動失敗は先頭の要素の名前 | `argv=["no-such-prog-zzz", "--flag"]` | `run a` | 127。ERR に `cannot start command 'no-such-prog-zzz'` |
| AC-105 | 空白を含む先頭の要素 | `argv=["my prog", "x"]`(`my prog` というファイルはない) | `run a` | 127。`cannot start command 'my prog'` |
| AC-106 | 実行できないファイル(UNIX 系) | `cmd` が実行権限のない通常のファイル(`T/data.txt`)のパス | `run a` | 127(`PermissionError`) |
| AC-107 | ディレクトリ・`/` を指定(UNIX 系) | `cmd="/"`、`cmd` がディレクトリのパス | `run a` | 127(`PermissionError`、`IsADirectoryError`、`NotADirectoryError` など `OSError` の各種) |
| AC-108 | 起動失敗は表示の言語に従う | `LANG=ja_JP.UTF-8`、存在しないプログラム | `run a` | 127。ERR に `コマンドを起動できません` |
| AC-109 | 終了コードの返り方 | `exit 0`、`exit 3`、`exit 255` のジョブ(警告の指定なし) | `run a` | 0 / 3 / 255。OUT・ERR 空 |
| AC-110 | 大きな終了コード | `TJ exit 300` | `run a` | 44(UNIX 系も Windows も `300 mod 256`。UNIX 系では、子の `sys.exit(300)` が OS により 44 になる) |
| AC-111 | 256 の終了コード(Windows) | `TJ exit 256`、`-v` | `run -v a` | 1。ERR に `exit code 256, conductor exits with 1` |
| AC-112 | 負の終了コード(Windows) | `TJ exit -1`、`-v` | `run -v a` | 255。ERR に `conductor exits with 255`(Windows で `Popen.returncode` が `-1` で返るか `4294967295` で返るかは未確認。どちらでも、32 ビット符号つきに直してから範囲合わせをして、255 になる。AC-392) |
| AC-113 | `.cmd` への危険な引数(文字列形式。Windows) | `T/echo-args.cmd`(内容 `@echo off` と `exit /b 0`)。`cmd` = そのパス + ` 'x" & echo injected > T\injected.txt'` | `run a` | 127(禁止文字を含む引数なので、起動を拒否する。[07](07-process-and-signals.md))。`T/injected.txt` ができない |
| AC-114 | `.cmd` への危険な引数(配列形式。Windows) | 同じスクリプト。`argv=[パス, "x\" & echo injected > T\injected.txt"]` | `run a` | 同上 |
| AC-115 | `.cmd` への安全な引数(Windows) | `T/ok.cmd`(`exit /b 7`)。`cmd` = そのパス + ` plain` | `run a` | 7 |
| AC-116 | シェルを使わないことの点検 | ソース | (`ast` の走査) | `subprocess` の import が `src/conductor/runner/` にだけあり、使う名前が `Popen`(と、定数・例外の型)だけ(`run`・`call`・`check_call`・`check_output`・`getoutput`・`getstatusoutput` を使わない)。`os.system`・`os.popen`・`os.exec*`・`os.spawn*`・`pty.spawn`・`commands` がどこにもない。`Popen` の呼び出しが `runner/` に 1 か所だけで、キーワード `shell` が、ないか `False`(リテラル)。`shell=True` がどこにもない。シェルの名前(`sh`・`bash`・`powershell`・`-c`)を `Popen` の引数列の先頭・次に渡す経路がない。**例外は、Windows の `.cmd` / `.bat` を起動する経路が、07 章で決められている場合の、その 1 か所だけである。この経路は、禁止文字(`"`、`%`、`^`、`&`、`\|`、`<`、`>`、`!`、`(`、`)`、改行、復帰、NUL)を含む引数を拒否したあとでだけ使い、`runner/platform.py` の 1 か所に集める(AC-486、AC-510、AC-513)**。区分: 点検 |

### 4.3 `run`(個別実行)

特に記さない限り、`P(…)` の `<id>:<n>` は、3.5 節のとおり `TJ exit <n>`。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-117 | 成功 | `P(p; a:0)` | `run a` | 終了 0。OUT・ERR 空。ステート: `S[a=success(0) ; completed=true]` が残る(最後のジョブで完了。完了のステートは残る) |
| AC-118 | 警告 | `P(p; warn:3{w=[3]}, fail:7, last:0)` | `run warn` | 0。OUT・ERR 空。ステート `warn=warning(3)` |
| AC-119 | 実行済みのスキップ | 上のあと | `run warn` | 0。OUT・ERR 空。ステートは変わらない |
| AC-120 | エラーはジョブの終了コード | 上のあと | `run fail` | 7。OUT・ERR 空。ステート `S[warn=warning(3), fail=error(7), last=pending ; resume=fail]` |
| AC-121 | エラーのあとの再実行 | 上のあと、定義の `fail` を `fail:0` に直す | `run fail` | 0。`fail=success(0)`。続けて `run last` → 0、`completed=true` |
| AC-122 | 自動の再実行はない | `P(p; a:7)` | `run a` | 7。ジョブは 1 回だけ実行(`TJ run` で確認) |
| AC-123 | 順序外は拒否(66) | `P(p; a:0, b:0, c:0)`、ステートなし | `run b` | 66。ERR = `conductor: error: job 'b': cannot run this job now: earlier jobs have not finished (out of order)`。OUT 空。ステートを作らない |
| AC-124 | 順序外でステートは変わらない | `S[a=success(0), b=error(1), c=pending ; resume=b]` | `run c` | 66。ステートは変わらない |
| AC-125 | 知らないジョブID | `B0` | `run nojob` | 68。ERR の最初の行 = `conductor: error: job is not defined in the definition file: 'nojob'`。続けて空行と使い方。OUT 空。ステートなし |
| AC-126 | 定義の誤りは知らないジョブIDより優先 | 定義に誤りあり | `run nojob` | 64 |
| AC-127 | ジョブIDなし | `B0` | `run` | 68。`missing argument` |
| AC-128 | 既定の定義ファイル | `T/conductor.yaml` | `run a`(`--config` なし) | 定義を読む。ステートは `T/conductor.yaml.state` |
| AC-129 | `--config` の場所 | `T/sub/x.yaml` | `run a --config sub/x.yaml` | ステートは `T/sub/x.yaml.state` |
| AC-130 | -v: 成功 | `cmd="TJ say hello ''"` | `run -v a` | 0。OUT = `hello`。ERR = `[conductor] job=a result=success (exit code 0)` |
| AC-131 | -v: 警告の遷移 | `P(p; a:1{w=[1], on=warning->c}, b:0, c:0)` | `run -v a` | 0。OUT 空。ERR = `[conductor] job=a result=warning (exit code 1)`⏎`[conductor] transition: warning -> c (skipped: b)` |
| AC-132 | -v: `@end` への遷移 | `P(p; a:0{on=success->@end}, b:0)` | `run -v a` | 0。ERR = `[conductor] job=a result=success (exit code 0)`⏎`[conductor] transition: success -> @end`。ステート `end=true`、`completed=false` |
| AC-133 | -v: 日本語 | `LANG=ja_JP.UTF-8`、`P(p; a:0, b:0)` | `run -v a`、続けて `run -v a` | ERR = `[conductor] job=a result=success (終了コード 0)`。2 回目に `[conductor] job=a スキップ (実行済み)` |
| AC-134 | -v: スキップの行 | `P(p; a:0, b:0)`、`a` を実行済み | `run -v a` | 0。ERR = `[conductor] job=a skipped (already executed)` |
| AC-135 | `@end` のあとの個別実行 | `P(p; a:0{on=success->@end}, b:0, c:0)` | `run a`、`run -v b`、`run -v c` | すべて 0。`b`・`c` は実行されない。`run b` の ERR = `[conductor] job=b skipped (skipped after @end)`。ステート: `b=skipped/byEnd`、`c=skipped/byEnd`、`end=true`。最後に `completed=true` |
| AC-136 | 遷移でスキップされたジョブの個別実行 | `P(p; a:1{w=[1], on=warning->c}, b:0, c:0)` | `run a`、`run -v b` | `run a` のあと `b=skipped/byTransition`。`run b` は 0、`b` は実行されず、ERR = `[conductor] job=b skipped (skipped by a transition)` |
| AC-137 | 成功コードに含まれない 0 | `P(p; a:0{s=[5]})` | `run a` | 1。ERR = `conductor: error: job 'a': exit code 0 is not listed in success_exit_codes`(-v なしでも出る)。ステート `a=error(0)`、`resume=a` |
| AC-138 | 成功コードに含まれない 0(-v) | 同上 | `run -v a` | 1。ERR = `[conductor] job=a result=error (exit code 0, conductor exits with 1)`⏎`conductor: error: job 'a': exit code 0 is not listed in success_exit_codes` |
| AC-139 | 成功コードの指定 | `P(p; a:5{s=[5]})`、`P(p; a:0{s=[0,2]})`、`P(p; a:2{s=[2]})` | `run a` | いずれも 0(成功) |
| AC-140 | シグナルで終了したジョブ(UNIX 系) | `argv=["sh", "-c", "kill -9 $$"]` | `run -v a` | 137。ERR = `[conductor] job=a result=error (terminated by signal 9)`。ステート `a=error`(終了コードなし)、`resume=a`(`Popen.returncode` が `-9` で、符号を反転して番号 9 に直す。AC-504) |
| AC-141 | 完了のステートが残っていても先頭から | `P(p; a:0, b:0)`。`run a`、`run b`(完了)のあと | `run a`(`TJ run` で記録) | 0。`a` が再び実行される。`completed` のステートは上書きされる |
| AC-142 | 完了のあと先頭以外は拒否 | 上の完了のステート | `run b` | 66 |
| AC-143 | `--remove-on-complete`(run) | `P(p; a:0)` | `run a --remove-on-complete` | 0。OUT・ERR 空。`T/conductor.yaml.state` がない |
| AC-144 | 定義の `remove_on_complete` | `P(p remove_on_complete=true; a:0)` | `run a` | 0。ステートがない |
| AC-145 | 完了でなければ消さない | `P(p; a:0, b:0)` | `run a --remove-on-complete` | 0。ステートが残る(`a=success`、`completed=false`) |
| AC-146 | ステートの形(エラーで停止) | 定義 `c2.yaml`(`P(n2; a:0, b:3, c:0)`) | `pipeline n2 --config c2.yaml` | 3。`T/c2.yaml.state` の内容は JSON のオブジェクトで、`completed=false`、`definition_key="c2.yaml"`、`definition_path="c2.yaml"`、`end_reached=false`、`format_version=1`、`job_order=["a","b","c"]`、`pipeline_name="n2"`、`resume_job_id="b"`、`job` は `a=success exit_code=0`、`b=error exit_code=3`、`c=pending`(`run_id` キーなし) |
| AC-147 | ステートの書式 | 上のステート | (ファイルを読む) | 1 つの JSON のオブジェクト(テストが `json.loads` で読める)。末尾は改行。インデントと、キーの並び(固定の順。実装が、キーの順を固定して出力する)は [06](06-state-file.md) のとおり。同じ内容のステートを書くと、毎回同じバイト列になる(`dict` の挿入順に頼るなら、その順を組み立てる箇所で固定する) |
| AC-148 | ステートに機微な値を書かない | `cmd="TJ say SECRET-OUT '' 3"`(終了コード 3 のエラーになる)。環境変数 `SECRET_ENV=SECRET-ENVVAL` を設定 | `run a` | ステートファイル(終了コード 3、`resume=a`)に、`command` の文字列、`SECRET-OUT`、`SECRET-ENVVAL` が現れない |
| AC-149 | 初期ステートの書き込みに失敗(UNIX 系、非 root) | `T` を書き込み不可にする(`os.chmod(T, 0o555)`。定義は読める。テストの後始末で権限を戻す) | `run a` | 65。ERR に `cannot write state file` と `(the job was not started)`。ジョブは実行されない |
| AC-150 | ジョブのあとの書き込みの失敗(UNIX 系、非 root) | 定義 `argv=["sh", "-c", "chmod a-w . ; exit 7"]`(ジョブが `T` を書き込み不可にする) | `run a` | 65(ジョブの 7 ではない)。ERR に `cannot write state file` と `(the job has already been executed)` |
| AC-151 | 定義の誤り(ファイルなし・誤り) | 定義なし / `format_version: 1` だけ | `run a` | 64。ERR は `conductor: error: ` で始まる。OUT 空 |


### 4.4 `pipeline`

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-152 | 全部成功: 記述順に実行 | `P(p; a:0{cmd="TJ --out O run a 0"}, b:0{cmd="TJ --out O run b 0"})`(2 ジョブ) | `pipeline p` | 0。OUT・ERR 空。`O` = `ran=a`⏎`ran=b`。ステート `S[a=success(0), b=success(0) ; completed=true]` が残る |
| AC-153 | 完了のステートの形 | 上のあと | (ステートを読む) | `resume_job_id` なし、`completed=true`、`end_reached=false`。形式は変わらない |
| AC-154 | 再実行は先頭から | 上のあと | `pipeline p` | 0。`a`、`b` が再び実行される(`O` に 2 組) |
| AC-155 | 警告の遷移で間をスキップ | `P(nightly; extract:1{w=[1], on=warning->load}, transform:0, load:0)` | `pipeline -v nightly --run-id r1` | 0。ERR = `[conductor] job=extract result=warning (exit code 1)`⏎`[conductor] transition: warning -> load (skipped: transform)`⏎`[conductor] job=transform skipped (skipped by a transition)`⏎`[conductor] job=load result=success (exit code 0)`。`transform` は実行されない |
| AC-156 | `@end` で完了 | `P(p; a:0{on=success->@end}, b:0, c:0)` | `pipeline p` | 0。`b`・`c` は実行されない。ステート `S[a=success(0), b=skipped/byEnd, c=skipped/byEnd ; end=true, completed=true]` |
| AC-157 | エラーで停止 | `P(p; a:0, b:7, c:0)`(各ジョブは `TJ run` 形式で記録) | `pipeline p` | 7。`c` は実行されない。ステート `S[a=success(0), b=error(7), c=pending ; resume=b]` |
| AC-158 | エラーからの再開 | 上のあと、定義の `b` を `b:0` に直す | `pipeline -v p` | 0。`a` は再実行されない(ERR に `[conductor] job=a skipped (already executed)`)。`b`・`c` が実行される。完了のステート |
| AC-159 | 個別実行の続きから一括実行 | `P(p; a:0, b:0, c:0)`。`run a` のあと | `pipeline p` | 0。`a` は再実行されない。`b`・`c` が実行される |
| AC-160 | 起動できないジョブで停止 | `P(p; a:0, b:0, c:0)` の `b` を存在しないプログラムに | `pipeline p` | 127。`c` は実行されない。ERR に `cannot start command` |
| AC-161 | パイプライン名の不一致 | `B0` | `pipeline wrongname` | 64。`pipeline name does not match the definition`。ジョブは実行されない |
| AC-162 | パイプライン名なし | `B0` | `pipeline` | 68。`missing argument` |
| AC-163 | `--remove-on-complete` | `P(p; a:0, b:0)` | `pipeline p --remove-on-complete` | 0。OUT・ERR 空。ステートがない。続く `watch` → `no state (nothing has run yet)` |
| AC-164 | 定義の `remove_on_complete: true` | `P(p remove_on_complete=true; a:0, b:0)` | `pipeline p` | 0。ステートがない |
| AC-165 | オプションと定義の組み合わせ | `remove_on_complete=false` の定義 | `pipeline p --remove-on-complete` | ステートがない(オプションが優先して消す) |
| AC-166 | 完了でないとき消さない | `P(p remove_on_complete=true; a:0, b:7)` | `pipeline p` | 7。ステートが残る(エラーで停止) |
| AC-167 | 完了のあと先頭以外の run は 66 | 完了のステート | `run b` | 66 |
| AC-168 | 初期ステートが書けない(UNIX 系、非 root) | `T` を書き込み不可 | `pipeline p` | 65。ジョブは起動されない |
| AC-169 | 完了のステートが残るとき定義の変更は照合されない | `pipeline p` で完了 → 定義のジョブIDを変える | `pipeline p` | 0(完了のステートは「空」として扱う。65 にならない) |

### 4.5 `watch`

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-170 | 表の形(英語) | `P(p; a:0, b:7, c:0)` で `pipeline p`(7 で終わる)のあと | `watch` | 終了 0。OUT = `Pipeline: p`⏎`JOB  STATUS   NOTE`⏎`a    success  exit code 0`⏎`b    error    exit code 7`⏎`c    pending`(行末の空白なし)。ERR 空 |
| AC-171 | ステートなし | ステートなし | `watch` | 0。OUT = `no state (nothing has run yet)`(1 行) |
| AC-172 | ステートなし(日本語) | `LANG=ja_JP.UTF-8` | `watch` | 0。OUT = `ステートはありません(まだ実行されていません)` |
| AC-173 | 長さ 0 のファイルも「なし」 | 長さ 0 のステートファイル | `watch` | 同上(`no state …`) |
| AC-174 | 完了の行 | `P(nightly; extract:1{w=[1], on=warning->load}, transform:0, load:0)`、`pipeline nightly --run-id r1` のあと | `watch --run-id r1` | OUT = `Pipeline: nightly`⏎`Run ID: r1`⏎`JOB        STATUS   NOTE`⏎`extract    warning  exit code 1`⏎`transform  skipped  skipped by a transition`⏎`load       success  exit code 0`⏎`completed` |
| AC-175 | 日本語の表 | 同上、`LANG=ja_JP.UTF-8` | `watch --run-id r1` | OUT = `パイプライン: nightly`⏎`Run ID: r1`⏎`ジョブ     状態     補足`⏎`extract    warning  終了コード 1`⏎`transform  skipped  遷移によりスキップ`⏎`load       success  終了コード 0`⏎`完了`(状態の語は英語のまま、`Run ID` は日本語でも `Run ID`) |
| AC-176 | `@end` のスキップの補足 | `@end` で完了したステート(4.4 節) | `watch` | `b`、`c` の行の NOTE = `skipped after @end` |
| AC-177 | 定義ファイルを読まない | 定義ファイルがない、または壊れている。ステートあり | `watch --config missing.yaml` | 0。ステートが表示される(定義がなくても動く) |
| AC-178 | ステートなし・定義なしでも 0 | 定義ファイルなし | `watch --config missing.yaml` | 0。OUT に `no state` |
| AC-179 | 壊れたステート | `{garbage` | `watch` | 65。ERR = ``conductor: error: conductor.yaml.state: state file is corrupt: JSON として読めません; run `conductor reset` to discard the state``(`` ` `` はバッククォート)。OUT 空 |
| AC-180 | 壊れた完了のステート | 完了のステートだが `run_id` がファイル名と食い違う | `watch` | 65 |
| AC-181 | ファイルへの出力に制御文字がない | `P(p; a:0, b:0)`、`run a` のあと | `watch`(標準出力をファイルに) | 0。ファイルに `\x1b` がなく、`a`、`success`、`pending` を含む |
| AC-182 | 端末では状態に色(UNIX 系。疑似端末) | 同上 | 標準出力を疑似端末(標準ライブラリの `pty.openpty()` の子側。`sys.stdout.isatty()` が真)にして `watch` | `success` が `\x1b[32m` と `\x1b[0m` で囲まれる(`warning` 33、`error` 31、`skipped` 36、`pending` は色なし)。列の幅は色の制御文字を除いた文字で決まる。区分: UNIX 系(疑似端末)。`pty` で作る手順は、この文書の作成時には実行していない(未確認。10 章)。併せて、端末かどうかの判定を `Deps` の関数にして、偽の `Deps` で「端末」を返す単体の試験を置く(区分: 注入) |
| AC-183 | `NO_COLOR` で色なし | 同上、`NO_COLOR=1` | 同上 | 制御文字なし。空の `NO_COLOR` では色あり。区分: UNIX 系(疑似端末か、注入) |
| AC-184 | Windows は色なし | Windows | `watch`(端末) | 制御文字なし。区分: Windows |
| AC-185 | ジョブIDの制御文字は表の中でもエスケープ | ジョブID に ESC を含む(YAML で `"a\u001bb"`)。実行済み | `watch` | 行に `a\u{1b}b`。生の ESC がない |
| AC-186 | 標準出力に書けない(注入) | ステートあり | `watch`(標準出力の `BufSink` を書き込み失敗にする) | 65。ERR = `conductor: error: cannot write to standard output`。Linux では `watch > /dev/full` でも起こせる(AC-495)。パイプの読み手が閉じた場合(`BrokenPipeError`)は AC-493、AC-516 |
| AC-187 | 使えないオプション | なし | `watch -v` / `watch --remove-on-complete` / `watch --all` / `watch extra` | 68。ERR = `option not allowed for this command: '-v'`(それぞれ対象は `-v`、`--remove-on-complete`、`--all`)。`watch extra` は `unexpected argument: 'extra'` |

### 4.6 `reset`

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-188 | ステートを消す | `P(p; a:0, b:7)`、`pipeline p` のあと | `reset` | 0。OUT・ERR 空。ステートがない。続く `watch` → `no state …` |
| AC-189 | 先頭から実行できる | 上のあと、`P(p; a:0, b:0)` に直す | `run a`(`TJ run` で記録) | 0。`a` が実行される |
| AC-190 | ステートがなくても成功 | ステートなし | `reset` | 0。OUT・ERR 空 |
| AC-191 | 定義がなくても動く | 定義なし | `reset --config missing.yaml` | 0 |
| AC-192 | 壊れたステート・壊れた定義を迂回 | 壊れたステートと壊れた定義 | `reset` | 0。ステートが消える |
| AC-193 | 完了のステートも消す | 完了のステート(run ID あり・なし) | `reset`、`reset --run-id r1` | 0。消える |
| AC-194 | `--run-id` は 1 つだけ消す | `conductor.yaml.state`、`.r1.state`、`.r2.state` がある | `reset --run-id r1` | `.r1.state` だけ消える。ほかは変わらない |
| AC-195 | `--all` で全部消す | `P(p; a:0, b:7)`。`pipeline p`(run ID なし)、`--run-id a`、`--run-id b` で実行。`T` に、別の定義のステート(`conductor.yaml.a.b.state`。中身はパイプライン名 `p`・ジョブ `a` の正常なステートで、`run_id` を持たない)、壊れたもの(`conductor.yaml.c.state` = `{garbage`)、一時ファイル(`.conductor.yaml.state.tmp.1.2.3`)、無関係なファイル(`conductor.yaml.bak`、`other.yaml.state`)を置く | `reset --all` | 終了 0。OUT 空。ERR は `conductor: warning: ` で始まり、`conductor.yaml.c.state` を含み、`conductor.yaml.a.b.state` を含まない。残るファイル: `.conductor.yaml.state.tmp.1.2.3`、`conductor.yaml`、`conductor.yaml.a.b.state`、`conductor.yaml.bak`、`conductor.yaml.c.state`、`other.yaml.state`。run ID なしと `.a.state`、`.b.state` は消える |
| AC-196 | 警告の文言 | 上と同じ | `reset --all` | ERR = ``conductor: warning: kept state files that could not be read (remove one with `conductor reset --run-id <ID>`):``⏎`  conductor.yaml.c.state`(ファイル名の行は先頭に空白 2 文字) |
| AC-197 | 壊れたものを個別に消す | 上のあと | `reset --run-id c` | 0。`conductor.yaml.c.state` が消える |
| AC-198 | 何もなくても 0 で出力なし | `T` に定義だけ | `reset --all` | 0。OUT・ERR 空 |
| AC-199 | `--all` と `--run-id` の併用 | なし | `reset --all --run-id x` | 68。`invalid value: '--all'` |
| AC-200 | `--all` は環境変数を見ない | `CONDUCTOR_RUN_ID='bad id'` | `reset --all` | 0 |
| AC-201 | シンボリックリンクと別の定義は消さない(UNIX 系) | `conductor.yaml.s.state` が通常のファイルへのシンボリックリンク。ディレクトリ `conductor.yaml.d.state`。`conductor.yaml.a.state` の中身の `run_id` がファイル名と食い違う | `reset --all` | リンク先のファイルは残る。リンクとディレクトリと、中身の `run_id` が食い違うファイルは消されない(`os.lstat`・`os.scandir` の `follow_symlinks=False` で種類を調べ、リンクをたどらない) |
| AC-202 | run ID なしのステートは中身を読まず消す | `conductor.yaml.state` が壊れている(通常のファイル) | `reset --all` | 消える。警告なし |
| AC-203 | 環境変数の値の違反 | `CONDUCTOR_RUN_ID='bad id'` | `reset` | 68。`invalid value: 'run ID (environment variable)'` |
| AC-204 | 引数の誤り | なし | `reset extra`、`reset -v`、`reset --remove-on-complete` | 68。`unexpected argument: 'extra'`、`option not allowed for this command: '-v'`、同 `'--remove-on-complete'` |
| AC-205 | 削除の失敗(UNIX 系、非 root) | `T/conductor.yaml.state` が空でないディレクトリ | `reset` | 65。ERR に `cannot remove state file`。OUT 空 |

### 4.7 `convert`(YAML と JSON の相互変換)

`convert <入力ファイル> --to yaml|json`。入力の形式は `--to` の反対(`--to json` なら YAML、`--to yaml` なら JSON)で、入力ファイルの拡張子は見ない([09](09-converter.md))。

変換の入力 `CV1`(YAML)、`CV1J`(`CV1` を JSON にしたもの)、`CV2`(YAML。アンカー、エイリアス、マージキーを含む)は、次のとおり。

```yaml
# CV1
format_version: 1
pipeline:
  name: nightly
  description: 夜間バッチ(抽出、変換、読み込み、通知)
job:
  - id: extract
    command: extract --all --since yesterday
  - id: transform
    command: 'transform "in dir" $HOME'
    warning_exit_codes: [3, 4]
    success_exit_codes: [0, 2]
    transitions:
      - on: warning
        next: notify
  - id: load
    command: load
    transitions:
      - on: success
        next: "@end"
  - id: notify
    command: notify --channel ops
```

```json
{"format_version": 1,
 "pipeline": {"name": "nightly", "description": "夜間バッチ(抽出、変換、読み込み、通知)"},
 "job": [
  {"id": "extract", "command": "extract --all --since yesterday"},
  {"id": "transform", "command": "transform \"in dir\" $HOME",
   "warning_exit_codes": [3, 4], "success_exit_codes": [0, 2],
   "transitions": [{"on": "warning", "next": "notify"}]},
  {"id": "load", "command": "load", "transitions": [{"on": "success", "next": "@end"}]},
  {"id": "notify", "command": "notify --channel ops"}]}
```

```yaml
# CV2(アンカー、エイリアス、マージキーを含む)
format_version: 1
pipeline:
  name: nightly
  description: 夜間バッチ(抽出、変換、読み込み、通知)
defaults: &defaults
  success_exit_codes: [0, 2]
  warning_exit_codes: [3, 4]
job:
  - id: extract
    command: extract --all --since yesterday
    <<: *defaults
  - id: transform
    command: 'transform "in dir" $HOME'
    <<: *defaults
    success_exit_codes: [0]
    transitions:
      - on: warning
        next: notify
      - on: error
        next: "@end"
  - id: load
    command: load
    transitions:
      - { on: success, next: "@end" }
  - id: notify
    command: notify --channel ops
```

特に記さない限り、失敗の期待は、終了 67、OUT 空、ERR の最初の行が `conductor: error: ` で始まる。「木」は、テストが `json.loads` と、YAML の読み取り(製品の読み取りの部品、またはテスト専用に用意した小さな読み取り)で得る、写像・列・文字列・整数・浮動小数点数・真偽値の構造。Python の `json.loads` は、`1` を `int`、`1.0` を `float` として区別する。ただし、`NaN`・`Infinity` を受理し、重複キーを黙って後勝ちにするので、**木の比較は、テスト用に、それらを検出する補助(`tests/support/tree.py`。`json.loads` の `parse_constant` と `object_pairs_hook` を使う)で行う**。`True == 1`・`1 == 1.0` が Python では真なので、木の比較は、型も比べる(`type(a) is type(b)`)。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-206 | YAML から JSON | `T/in.yaml` = `CV1` | `convert in.yaml --to json` | 終了 0。ERR 空。OUT は JSON で、`"format_version": 1` と `"id": "extract"` を含む。OUT を JSON として読んだ木が `CV1` の木と等しい。キーの順は入力の記述順、インデントは 2 スペース、末尾は改行(細部は [09](09-converter.md) のとおり) |
| AC-207 | YAML から JSON(アンカー展開) | `T/in.yaml` = `CV2` | `convert in.yaml --to json` | 0。OUT に `<<` も `&defaults` もない。OUT を JSON として読んだ木が、`CV2` を展開した木と等しい(`transform` の `success_exit_codes` は `[0]`、`extract` は `[0, 2]`) |
| AC-208 | 変換の出力は標準出力だけ | `T/in.yaml` = `format_version: 1⏎pipeline:⏎  name: p⏎job:⏎  - id: a⏎    command: echo a` | `convert in.yaml --to json` | 0。ERR 空。OUT に `"job": [` と `"format_version": 1` |
| AC-209 | 入力形式は `--to` の反対で、拡張子は見ない | `T/looks-like.json` = `a: 1⏎b: [x, y]`(YAML の内容) | `convert looks-like.json --to json` / `convert looks-like.json --to yaml` | 前者: 0、OUT を JSON として読むと `{"a": 1, "b": ["x","y"]}`。後者: 67(JSON として読んで `syntax error`) |
| AC-210 | 文法の誤りは種類と行つき | `T/broken.yaml` = `a: [unclosed` | `convert broken.yaml --to json` | 67。OUT 空。ERR の最初の行が `conductor: error: broken.yaml:` で始まり、種類の文が `syntax error`、理由が `YAML の文法に誤りがあります`(行は採用ライブラリの検出位置。未確認) |
| AC-211 | 入力がない | なし | `convert nofile --to json` | 67。ERR = `conductor: error: nofile: input file does not exist: 指定の場所にファイルがありません` |
| AC-212 | ディレクトリを入力に | `T/d` がディレクトリ | `convert d --to json` | 67。`cannot read input file` |
| AC-213 | 入力が大きすぎる | 1,048,577 バイト(中身が壊れていてもよい) | `convert big.yaml --to json` | 67。`input is too large` |
| AC-214 | ちょうど 1 MiB は受理 | `a: "xxx…"` で、ちょうど 1,048,576 バイト | `convert max.yaml --to json` | 0。OUT が `{⏎  "a": "xxx` で始まる |
| AC-215 | UTF-8 でない入力 | `a: ` のあとに不正なバイト列 | `convert bad.yaml --to json` | 67。`cannot read input file`(文法を見る前) |
| AC-216 | 表せない値は場所つき | `T/in.yaml` = `job:⏎  - id: x⏎  - id: y⏎    note: .nan` | `convert in.yaml --to json` | 67。ERR に `value cannot be represented in the target format: NaN と無限大は JSON で表せません` と `(at: job[1].note)` |
| AC-217 | 表せない値と、表せる値の境界 | YAML(`--to json`): `a: .inf` / `a: .nan` / `a: -.inf` / `a: [1, .inf]` / `a: 9223372036854775808` / `a: 0xFFFFFFFFFFFFFFFFFF` / `a: 1e999`。JSON(`--to yaml`): `{"a": 9223372036854775808}` / `{"a": 1e400}` / キーが 1,025 文字のオブジェクト。受理されるもの: YAML の `a: ~` / `a: null` / `a:`(値は `null`)、JSON の `{"a": null}`、`{"a": 9223372036854775807}`、キーが 1,024 文字のオブジェクト | `convert … --to <変換後>` | 表せないものは 67(種類 `unrepresentable`)。原因の文は、順に `NaN と無限大は JSON で表せません`(`.inf`・`.nan`・`-.inf`・`[1, .inf]` の 4 つ)、`数値が表せる範囲を超えています`(YAML の 3 つと JSON の 2 つ)、`キーが長すぎます(上限 1024 文字)`(1,025 文字のキー)。受理されるものは 0 で、`null` は `null` のまま出る。64 ビット符号つき整数の範囲の端(`9223372036854775807`)は、Python の `int` で桁を落とさずに扱う(`float` に直さない。範囲の検査は `-2**63 <= n <= 2**63 - 1` の比較)。`1e999` を `float()` に通すと `inf` になる点に注意する(AC-502) |
| AC-218 | 複数の文書・タグ・文字列でないキー | `a: 1⏎---⏎b: 2` / `a: !!str 1` / `1: x` / `true: x` | `convert in.yaml --to json` | 67。種類の文は `syntax error`。理由は順に `複数の文書は書けません(1 ファイルに書ける文書は 1 つです)`、`タグは使えません`、`キーは文字列で書きます`、`キーは文字列で書きます` |
| AC-219 | 最上位がマップでない | YAML `- 1⏎- 2`(`--to json`)、JSON `[1, 2]`(`--to yaml`) | `convert` | 67。`top level is not a map: 最上位がマップではありません`(行 1) |
| AC-220 | 空の入力 | 空の YAML ファイル(または `# c` だけ)を `--to json` で。空の JSON ファイル(0 バイト)を `--to yaml` で | `convert` | YAML: 67。`top level is not a map: 文書が空です`。JSON: 67。`syntax error`(JSON では空の入力は誤り) |
| AC-221 | コメントは捨てられる | `# 先頭⏎a: 1 # 末尾` | `convert in.yaml --to json` | 0。OUT = `{⏎  "a": 1⏎}`。OUT に `#` がない |
| AC-222 | キーの順序は入力の記述順 | `b: 1⏎a: 2` | `convert in.yaml --to json` | OUT = `{⏎  "b": 1,⏎  "a": 2⏎}`(辞書順に並べ替えない。`sorted()` や `json.dumps(sort_keys=True)` を使わない。マージキーで取り込まれたエントリは `<<` の位置に並ぶ。[09](09-converter.md))。`dict` は挿入順を保つので、整数に見えるキー(`"1"`、`"2"`)でも順序は変わらない(`b: 1⏎"1": 2` のような入力で、順序が保たれることも確かめる。AC-511) |
| AC-223 | 空のマップ | `{}`(YAML)、`{}`(JSON) | `convert in.yaml --to json` / `convert in.json --to yaml` | 0。JSON の出力は `{}`。YAML の出力は `{}` |
| AC-224 | アンカーとマージキー | `base: &b {x: 1, y: 2}⏎d:⏎  <<: *b⏎  y: 9` | `convert in.yaml --to json` | 0。`d` は `"x": 1`、`"y": 9`(自身のキーが優先) |
| AC-225 | マージキーの値がマップでない | `a:⏎  <<: 5` | `convert in.yaml --to json` | 67。種類の文は `syntax error`、理由は `マージキーの値がマップではありません`(行 2。場所 `(at: …)` は付かない) |
| AC-226 | 重複するキー | YAML `a: 1⏎b: 2⏎a: 3`(`--to json`)、JSON `{"a": 1, "b": 2, "a": 3}`(`--to yaml`) | `convert` | 67。`syntax error: キーが重複しています`(YAML は行 3) |
| AC-227 | 再帰するアンカー | `a: &r [*r]` | `convert in.yaml --to json` | 67。`アンカーが自分自身を含んでいます`(`RecursionError` が漏れない) |
| AC-228 | スカラーの解決(YAML 1.2 のコアスキーマ。03 章 4.2 節) | YAML: `a: yes`、`b: 0x1F`、`c: 007`、`d: '1'`、`e: 2001-01-01`、`f: 1_000`、`g: on`、`h: ~~`、`i: 'null'`、`j: 1:30`、`k: 0b101`、`l: 0o17`(YAML ライブラリの既定の型判定は、`yes`・`on` を真偽値に、`2001-01-01` を日付に、`1_000` や `1:30` を整数にする版があるので、構文木の段階までを使い、型は自前で決める。AC-525) | `convert in.yaml --to json` | `"a": "yes"`、`"b": 31`、`"c": 7`、`"d": "1"`、`"e": "2001-01-01"`、`"f": "1_000"`、`"g": "on"`、`"h": "~~"`、`"i": "null"`、`"j": "1:30"`、`"k": "0b101"`、`"l": 15`(キーは入力の記述順に並ぶ)。真偽値になるのは `true`/`false` の 3 通りの書き方(`true`、`True`、`TRUE` と `false` の同様の形)だけ。食い違うものは、03 章を正本とする |
| AC-229 | 展開後の要素数の上限 | `a: [0, 0, …]`(0 を n 個)。JSON は `{"a": [0, 0, …]}` | `convert in.yaml --to json` / `convert in.json --to yaml` | n = 99,997 は 0。n = 99,998 は 67(種類の文 `input is too large`。理由 `要素数が上限(100000)を超えています`)。JSON も同じ境界 |
| AC-230 | アンカーの爆発は上限で止まる | アンカーの入れ子で、指数的に増える YAML | `convert in.yaml --to json` | 30 秒以内に 67(`input is too large`。理由は `要素数が上限(100000)を超えています` または `文字の大きさが上限(16 MiB)を超えています`)。メモリを使い果たさない(`MemoryError` で落ちない) |
| AC-231 | 入れ子の深さの上限 | YAML: `a: ` に続けて `[` を 32 個 / 33 個(閉じも書く)。JSON: `{"a": ` に続けて `[` を 32 個 / 33 個(閉じも書く) | `convert` | 32 個: 0。33 個: 67(`syntax error: 入れ子が深すぎます(上限は 32 段)`)。最上位の写像(根)は数えず、その直下のコレクションを 1 段と数える([03](03-definition-format.md)) |
| AC-232 | 極端な深さでも異常終了しない | 1,000 段、100,000 段の入れ子(YAML・JSON) | `convert … ` | 67(`syntax error`)。異常終了しない(`RecursionError` が漏れて 70 にならない。2.2 節。`sys.setrecursionlimit` を上げて対処しない) |
| AC-233 | 往復で木が等しい | `CV1` | `convert in.yaml --to json` の OUT を `convert --to yaml` に通す | 結果の木が `CV1` の木と等しい。`command` が配列形式の定義、`retry` の写像も同じ。`CV1J` を先に `--to yaml`、次に `--to json` に通しても等しい |
| AC-234 | 数値の往復 | 整数の端(64 ビット符号つき整数の最小・最大)、浮動小数(`1.0`、`-0.0`、`0.5`、`1e21`)、空の配列・写像 | 往復(YAML → JSON → YAML、JSON → YAML → JSON) | 型が変わらない(`1` と `1.0` を区別する。`0.0` と `-0.0` は別。JSON の出力で、浮動小数は、小数点か指数を持つ書き方になる。`repr(float)` の `1e+21` と `1e21` の書き方の差は 09 章に従う)。整数の端は、桁を落とさない |
| AC-235 | ファイルを作らず環境に左右されない | `T` に入力のみ | `convert` を、別のカレントディレクトリ・異なる環境変数で実行 | `T` のファイル一覧が実行の前後で同じ。結果が同じ |
| AC-236 | 標準出力に書けない(注入) | 正しい入力 | `convert`(標準出力の `BufSink` を書き込み失敗にする) | 67。ERR = `conductor: error: cannot write to standard output` |
| AC-237 | 引数の誤り | なし | `convert f`(`--to` なし)/ `convert f --to xml` / `convert f --to YAML` / `convert f --to Json` / `convert f --to yaml --config c` | 68。`missing argument: '--to'` / `invalid value: 'xml'` / `invalid value: 'YAML'` / `invalid value: 'Json'` / `option not allowed for this command: '--config'` |
| AC-238 | 変換結果の全体を 1 回で書く | 失敗する入力 | `convert` | 失敗のとき、標準出力に 1 バイトも書かれない |

### 4.8 run ID

特に記さない限り、定義は `two_jobs(n)` = `P(p; a:0, b:<n>)`(`a` は成功、`b` は終了コード `n`)。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-239 | run ID ごとに別のステート | `two_jobs(7)` | `pipeline p --run-id r1` と `pipeline p --run-id r2` を同時に | どちらも 7。OUT 空。`T` のファイルは `conductor.yaml`、`conductor.yaml.r1.state`、`conductor.yaml.r2.state` の 3 つだけ。各ステートは `run_id` がその ID、`resume=b`、`a=success` |
| AC-240 | 同じ run ID で再開し、他は変わらない | 上のあと、定義を `two_jobs(0)` に直す | `pipeline p --run-id r1` | 0。`r1` は完了のステートが残る(`completed=true`)。`r2` のステートファイルはバイト列が変わらない |
| AC-241 | ステートの形(run ID あり・なし) | `two_jobs(7)` | `pipeline p --run-id r1`、`pipeline p` | `.r1.state` は `"run_id": "r1"` を持つ。run ID なしの `.state` は `run_id` キーを持たない |
| AC-242 | 環境変数 `CONDUCTOR_RUN_ID` | `two_jobs(7)`、`CONDUCTOR_RUN_ID=fromenv` | `pipeline p` | 7。`conductor.yaml.fromenv.state` ができる |
| AC-243 | 空のオプションは run ID なしに固定 | 同じ環境変数 | `pipeline p --run-id ""` | 7。`conductor.yaml.state`(run ID なし)ができる |
| AC-244 | オプションは環境変数に優先 | 同じ環境変数 | `pipeline p --run-id fromopt` | `conductor.yaml.fromopt.state` ができる(`fromenv` は使われない) |
| AC-245 | 環境変数が空なら run ID なし | `CONDUCTOR_RUN_ID=` | `pipeline p` | 7。`conductor.yaml.state` |
| AC-246 | 定義の `run_id_env` | `P(p run_id_env=MY_RUN; a:7)`、`MY_RUN=m1`、`CONDUCTOR_RUN_ID=ignored` | `run a` | 7。`conductor.yaml.m1.state` ができる(`ignored` は使われない) |
| AC-247 | `watch`・`reset` も `run_id_env` を読む | 上のあと、`MY_RUN=m1` | `watch`、`reset` | `watch` の OUT に `Run ID: m1`。`reset` で `conductor.yaml.m1.state` が消える |
| AC-248 | 有効な run ID | `two_jobs(0)` | `pipeline p --run-id <値>`(`1`、`r1`、`12345`、`build-2026.11_a`、`a.b`、`x` を 64 個) | いずれも 0 |
| AC-249 | 無効な run ID(オプション) | `two_jobs(0)` | `pipeline p --run-id <値>`(`a/b`、`..`、`x` を 65 個、`a b`、`.x`、`日本語`) | 68。ERR の最初の行 = `conductor: error: invalid value: '--run-id'`。ERR にその値が現れない。ステートが作られない |
| AC-250 | 無効な run ID(環境変数) | 同じ値を `CONDUCTOR_RUN_ID` に | `pipeline p` | 68。ERR の最初の行 = `conductor: error: invalid value: 'run ID (environment variable)'`。値が現れない |
| AC-251 | validate は run ID を受け付けない | `two_jobs(0)` | `validate --run-id x` | 68 |
| AC-252 | run ID の決定の順序 | 定義の誤り + `--run-id 'a b'` | `run a --run-id 'a b'` | 64(定義の検査が先。run ID は後) |
| AC-253 | run ID の決定はステートの読み込みの前 | 壊れたステート + `--run-id 'a b'` | `run a --run-id 'a b'` | 68(65 ではない) |
| AC-254 | `watch` は `--run-id` があれば定義を読まない | 定義が壊れている。`.r1.state` あり | `watch --run-id r1` | 0。表が出る |
| AC-255 | `watch`・`reset` は壊れた定義でも既定の環境変数名 | 定義が壊れている。`CONDUCTOR_RUN_ID=r1` | `reset` | 0。`.r1.state` だけが消える |
| AC-256 | run ID なしの `watch` に `Run ID` の行はない | run ID なしのステート | `watch` | OUT に `Run ID` の行がない |
| AC-257 | ファイル名と中身の `run_id` の食い違い | `conductor.yaml.r1.state` を `conductor.yaml.r2.state` に複製する | `run b --run-id r2` | 65。ERR = ``conductor: error: conductor.yaml.r2.state: state file is corrupt: run_id がファイル名の run ID と一致しません; run `conductor reset --run-id <ID>` to discard the state``(`<ID>` は文字どおり) |
| AC-258 | run ID ありでも `--remove-on-complete` で消える | `two_jobs(0)` | `pipeline p --run-id r1 --remove-on-complete` | 0。`conductor.yaml.r1.state` がない |
| AC-259 | `reset --run-id` は別の run に影響しない | `.r1.state`、`.r2.state`、`.state` | `reset --run-id r1` | `.r1.state` だけが消える |
| AC-260 | 大文字小文字を区別しないファイルシステム | macOS の既定 / Windows | `pipeline p --run-id A`、`pipeline p --run-id a` | 同じファイルになる。食い違いは、中身の `run_id` との照合で 65 として検出される(利用者は大文字小文字だけで run ID を区別しない)。区分: macOS / Windows。**未確認**(仕様の記述に基づく) |

### 4.9 リトライ

特に記さない限り、`RT` は `P(p; a:0{cmd="TJ flaky F <fails> 75", retry=(exit_codes=[75], max=3, interval="1s")})`(`F` は `T/flaky.count`。`<fails>` は失敗させる回数)。`flaky` は、実行のたびに `F` の回数を 1 増やす。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-261 | 2 回失敗して 3 回目に成功 | `RT`(`fails=2`) | `run a` | 0。OUT 空。ERR(-v なしでも)= `conductor: retry: job 'a' exited with code 75; retry 1 of 3 in 1 seconds`⏎`conductor: retry: job 'a' exited with code 75; retry 2 of 3 in 1 seconds`。`F` の値 = 3。ステート `a=success(0)` |
| AC-262 | -v で結果の行は 1 回だけ | 上と同じ | `run -v a` | ERR は上の 2 行のあとに `[conductor] job=a result=success (exit code 0)` の 1 行だけ(途中の失敗の結果行は出ない) |
| AC-263 | 使い切る | `RT`(`fails=99`) | `run a` | 75。ERR は retry の行 3 つ(`retry 1 of 3`、`2 of 3`、`3 of 3`)。`F` の値 = 4(最大 4 回の実行)。ステート `a=error(75)`、`resume=a` |
| AC-264 | 対象外の終了コードはリトライしない | `exit_codes: [75]` で、ジョブが 3 で失敗(`flaky F 99 3`) | `run a` | 3。retry の行なし。`F` の値 = 1 |
| AC-265 | `retry` がなければ 1 回だけ実行 | `P(p; a:0{cmd="TJ flaky F 99 75"})` | `run a` | 75。retry の行なし。`F` = 1 |
| AC-266 | 知らせは日本語でも | `RT`(`fails=1`)、`LANG=ja_JP.UTF-8` | `run a` | 0。ERR = `conductor: retry: ジョブ 'a' が終了コード 75 で終わりました。1 秒後にリトライします(1/3 回目)` |
| AC-267 | 倍率と上限の待ち時間 | `retry=(exit_codes=[75], max=3, interval="1s", multiplier=2, max_interval="2s")`、`fails=99` | `run a` | 75。retry の行の待ち時間は順に `1`、`2`、`2`(`in 1 seconds`、`in 2 seconds`、`in 2 seconds`)。実行全体の時間は 5 秒以上 |
| AC-268 | 待ち時間の列(関数の検査) | `interval` / `multiplier` / `max_interval` / `max` | 待ち時間を求める関数(`flow/`。結果は `int`) | `30s`、倍率なし、max 3 → 30、30、30。`10s`、2、`60s`、max 5 → 10、20、40、60、60。`60s`、3、`60s`、max 3 → 60、60、60。`1s`、10、`1h`、max 6 → 1、10、100、1000、3600、3600。`1h`、10、`24h`、max 4 → 3600、36000、86400、86400。極端に大きい値でも結果が `2**64 - 1` を超えない(`min(x, 2**64 - 1)` による飽和。AC-501)。区分: 単体 |
| AC-269 | 再実行の結果が成功なら遷移へ(pipeline) | `P(p; a:0{cmd="TJ flaky F 1 75", retry=(…)}, b:0)` | `pipeline p` | 0。`b` も実行される。完了のステート |
| AC-270 | 使い切ると停止(pipeline) | `RT` に `b:0` を足し、`fails=99` | `pipeline p` | 75。`b` は実行されない。`resume=a` |
| AC-271 | ジョブ自身の 127 は一覧にあればリトライ | `TJ exit 127`、`exit_codes: [127]`、`max: 1`、`interval: 1s` | `run a` | 127。retry の行が 1 つ(1 回リトライしてから終わる) |
| AC-272 | 起動の失敗はリトライしない | 存在しないプログラム、`exit_codes: [127]` | `run a` | 127。retry の行なし |
| AC-273 | シグナルで終了したジョブはリトライしない(UNIX 系) | `argv=["sh","-c","kill -9 $$"]`、`exit_codes: [137]` | `run a` | 137。retry の行なし |
| AC-274 | 待機中のシグナル(UNIX 系) | `retry=(exit_codes=[75], max=3, interval="60s")`、`fails=99` | `run a` を起動し、ERR に最初の retry の行が出てから SIGTERM を送る(`Popen.send_signal(signal.SIGTERM)`) | 143。ステート: `a=pending`(エラーとして記録しない)、`resume=a`。2 つ目の retry の行は出ない。ジョブは再び起動されない(`F` = 1) |
| AC-275 | 待機中の中断は 1 秒以内 | `interval="1h"`、`fails=99` | 最初の retry の行のあと SIGTERM | SIGTERM を送ってから 1 秒以内に、143 で終わる(待ちは、短い区間に分けて旗を見る。PEP 475 により、シグナルのハンドラが戻ったあとに `time.sleep` は残りの時間を眠り直すので、**長い 1 回の `time.sleep` で待ってはならない**。区間の長さは 07 章に従う)。単体では、3600 秒の待機に 200 ms 後の合図(別スレッドから `os.kill(os.getpid(), signal.SIGTERM)` か、`signal.raise_signal`)で、全体が 1.2 秒未満 |
| AC-276 | 再開は最初の実行から数え直す | 上のあと(`resume=a`) | 再度 `run a`(`interval="1s"`、`fails=1`) | `retry 1 of 3` の行から始まる。0 で完了 |
| AC-277 | 待機中のシグナル(pipeline) | 同様(pipeline) | `pipeline p`、待機中に SIGTERM | 143。以降のジョブを実行しない。`resume=a` |
| AC-278 | ステートは最終の結果で 1 回だけ | `RT`(`fails=2`) | `run a` | 実行の途中に、ステートファイルのジョブ `a` の状態が `success`/`error` にならない(最終の結果のときだけ変わる。途中の失敗はステートに書かれない)。リトライの回数はステートに書かれない。観測: 待機中にステートファイルを読む |
| AC-279 | README のリトライの説明 | README | (検査) | 見出し、5 つのキー名(`exit_codes`、`max`、`interval`、`multiplier`、`max_interval`)があり、載っている YAML の例が実際に読み取りを通る。区分: 点検 |


### 4.10 シグナルと中断

UNIX 系の「実行」は、被試験体を起動し(`start_conductor`)、`TJ sleep` の記録 `ready` が `O` に現れるのを待ってから、被試験体のプロセスにシグナルを送る(`Popen.send_signal(signal.SIGTERM)` など)。定義は `SL` = `P(p; sleeper:0{cmd="TJ --out O sleep"}, after:0)` を使う。
シグナル番号とジョブの終わり方の対応は、`Popen.returncode` が負の値 `-n` のとき、シグナル `n` で終了した、と読む(AC-504)。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-280 | 実行中の SIGTERM(UNIX 系) | `SL` | `run sleeper` を起動し、`ready` のあと SIGTERM | 143。OUT 空。ステート `S[sleeper=error, after=pending ; resume=sleeper]`。`pid=` に記録されたジョブのプロセスが存在しない(孤児なし。`os.kill(pid, 0)` が `ProcessLookupError`) |
| AC-281 | HUP のあとすぐ INT(UNIX 系) | `SL`(`sleeper` だけでもよい) | `ready` のあと SIGHUP、続けて SIGINT | 129(最初の番号 = 1 が残る)。ステート `sleeper=error` |
| AC-282 | SIGINT / SIGQUIT 単独(UNIX 系) | `SL` | SIGINT / SIGQUIT | 130 / 131(SIGQUIT では、`TJ` の終わり方は実装時に確認。3.4 節。確かめるのは `conductor` の終了コードだけ) |
| AC-283 | -v の表示(UNIX 系) | `SL` | `run -v sleeper`、SIGTERM | 143。ERR に `[conductor] job=sleeper result=error (interrupted by signal 15)` |
| AC-284 | 時間を空けた 2 つのシグナルは最初の番号が残る | シグナル監視の単体 | INT のあと、時間を空けて HUP | 最初の番号 2 が残る。区分: 単体(UNIX 系) |
| AC-285 | 最初の番号・登録は 1 回 | シグナル監視の単体 | 監視を登録し、HUP を受ける。再度の登録 | 最初の番号 1 が残る。2 回目の登録は「すでに登録済み」(失敗ではない)で、`signal.getsignal(SIGHUP)` が、1 回目の登録のハンドラのまま変わらない(ハンドラが入れ子にならず、2 回目の登録が、元のハンドラとして 1 回目のハンドラを保存して、復元時に自分自身へ戻す、ということがない)。区分: 単体(UNIX 系) |
| AC-286 | 転送は同じシグナルを 1 回だけ | 起動の部品(`runner/`)の単体 | ジョブの実行中に合図を立てる(SIGTERM と SIGINT で別々に) | ジョブは、そのシグナルで終了する(`Popen.returncode` が `-15` / `-2`、番号 15 / 2)。送信は 1 回(続けて合図を立てても追加で送らない)。区分: 単体(UNIX 系) |
| AC-287 | 許可外の番号は送らない | 起動の部品の単体 | 合図の番号が SIGKILL(9)、SIGSTOP、SIGCONT、SIGUSR1、0、-1、9999 | 送信しない。ジョブは自分で終わる。終わりの観測の直後に `interrupted = True`、`signal_number` がその番号。区分: 単体(UNIX 系) |
| AC-288 | ジョブのセッションとプロセスグループ | 起動の部品の単体。07 章の決定(新しいセッションにするか。`Popen(start_new_session=True)` は、UNIX 系で `setsid()` を呼び、子のプロセスグループ ID が子の pid になる。Windows の `creationflags` は別) | ジョブを起動して、プロセスグループ ID を調べる(`os.getpgid(pid)`) | 07 章が新しいセッションにする決定の場合: ジョブのプロセスグループ ID が、ジョブの pid に等しく、`conductor`(呼び出し側)のものと異なる。しない決定の場合: ジョブのプロセスグループ ID が、`conductor` のものと等しい(端末の Ctrl+C が、`conductor` とジョブの両方に届く。その場合の二重の転送の扱いは 07 章に従う)。どちらの場合も、`conductor` は、シグナルを子の pid にだけ送る(プロセスグループには送らない)。Windows では行わない。区分: 単体(UNIX 系) |
| AC-289 | 先に終わったジョブには転送しない | 起動の部品の単体(偽の子で、`poll()` が終了済みを返す) | ジョブが終わった後に合図が立つ | 送信しない(終了済みの pid に `send_signal` しない。`Popen.send_signal` は、先に `poll()` を呼んで、pid の再利用を避ける作りになっている(標準ライブラリのソースのコメント。確認した版は 1 章)が、版により異なりうるので、`conductor` も、送る直前に `poll()` で確かめる)。区分: 単体(UNIX 系) |
| AC-290 | 起動前の合図 | 起動の部品の単体 | 合図が立った状態で起動を呼ぶ | ジョブは起動されない(`started=False`、`interrupted=True`、`signal_number` = 番号)。分類は「分類なし」、終了コード `128 + 番号`。区分: 単体 |
| AC-291 | 終了の観測の直後の合図 | 起動の部品の単体 | ジョブの実行中に、転送されない番号の合図を立てる(ジョブは自分で終わる) | ジョブの終了コードを保ったまま `interrupted = True`、`signal_number` = その番号。分類は「エラー、`128 + 番号`」。区分: 単体 |
| AC-292 | 合図が立たなければ中断にならない | 起動の部品の単体 | 合図なしで `exit 1` | `interrupted = False`、`signal_number` なし。区分: 単体 |
| AC-293 | 起動前の中断の記録(注入) | 流れの制御(偽の `ScriptProcess`・`FakeSignals`) | 起動前に合図が立つ | ジョブを起動せず、`resume` = そのジョブ、エラーとして記録しない、終了 `128 + 番号`。区分: 注入 |
| AC-294 | 実行中の中断の記録(注入) | 同上 | 実行中に合図(2 つ来ても最初の番号) | ジョブを `error` として記録、終了 `128 + 最初の番号`。区分: 注入 |
| AC-295 | ジョブのあいだの中断(pipeline、注入) | 同上 | 1 つ目のジョブが成功した直後に合図 | 次のジョブを起動せず、再開位置を記録して `128 + 番号`。-v で `[conductor] interrupted by signal N`。区分: 注入 |
| AC-296 | 検査中の合図より誤りが優先 | 同上 | 定義の検査中に合図が立つ。定義の誤り / 順序外 / スキップ | 64 / 66 / 0(中断の終了コードにならない)。区分: 注入 |
| AC-297 | シグナルの準備は検査の前 | 同上 | `run` / `pipeline`、`validate`・`watch`・`reset`・`convert`・引数の誤り | `run`・`pipeline` だけが、定義の検査より前にシグナルを準備する。ほかは準備しない。区分: 注入 |
| AC-298 | シグナルの準備の失敗は 70 | 同上 | 準備が失敗する | 70。ERR = `conductor: error: failed to set up signal handling`。定義の検査もジョブの起動もしない。区分: 注入 |
| AC-299 | 登録済みでも続ける | 同上 | 準備が「すでに登録済み」を返す | 流れは成功と同様に続く。区分: 注入 |
| AC-300 | 標準エラーが閉じていても本来の終了コード | 同上 | 標準エラーへの書き込みが失敗する(`BufSink` の書き込み失敗。`BrokenPipeError`) | 本来の終了コードで終わる。区分: 注入 |
| AC-301 | Windows の Ctrl+C と Ctrl+Break(手動) | Windows の実機、コンソール。`SL` | `conductor run sleeper` の実行中に Ctrl+C を押す。別の回に Ctrl+Break を押す | (1) Ctrl+C: 終了コード 130。Ctrl+Break: 終了コード `128 + 21`(`signal.SIGBREAK` の値は 21 と記憶しているが、未確認。07 章に従う)。(2) ジョブの子プロセスが残らない。(3) 続く `watch` で `sleeper` が `error`、`after` が `pending`。(4) `run -v` の ERR に `interrupted by signal 2`(Ctrl+Break は、その番号)。結果を記録欄に残す(`tests/WINDOWS_CTRL_C_CHECKLIST.md`)。区分: 手動(Windows)。実施の記録は、この文書の作成時点では存在しない(未確認) |

### 4.11 ステートの異常系

ここの「実行」は、特に記さない限り `run a`(`P(p; a:0, b:0)`)。すべて、**ジョブは実行されず**、ステートファイルは**変わらず**、OUT は空。ERR の最初の行は `conductor: error: conductor.yaml.state: <種類の文>: <理由>; <案内>`。
案内は ``run `conductor reset` to discard the state``(run ID あり: ``run `conductor reset --run-id <ID>` to discard the state``)。ステートは JSON の 1 つのオブジェクト([06](06-state-file.md))。

| ID | 機能 | ステートの内容(前提) | 期待(終了 65) |
|----|------|------------------------|----------------|
| AC-302 | JSON として読めない | `{garbage` | `state file is corrupt: JSON として読めません` |
| AC-303 | UTF-8 でない | 不正なバイト列 | `state file is corrupt: UTF-8 の文字列ではありません` |
| AC-304 | 大きさの上限 | 正しいステートの JSON の末尾に、空白(スペース)を足して 1,048,577 バイト | `state file is too large: 大きさが上限(1 MiB)を超えています`。ちょうど 1,048,576 バイトは受理(実行が進む) |
| AC-305 | 書式の版が違う | `"format_version": 2`(ほかに未知のキーがあっても)/ `0` | `state file version is not supported: 対応していない版です(reset が必要です)` |
| AC-306 | 書式の版がない・整数でない | `format_version` なし / `"1"` / `1.0` / `true` | `state file is corrupt: format_version がありません` / `format_version が整数ではありません`(`1.0`・`true` も同じ。`True` を `1` と読まない。AC-502) |
| AC-307 | 未知のキー | 最上位に未知のキー。`jobs` というキー | `state file is corrupt: 知らないキーがあります` |
| AC-308 | 必須のキーがない | `pipeline_name` などを欠く | `state file is corrupt: 必須のキーがありません` |
| AC-309 | 値の型・状態の誤り | `status: "weird"`、`skip_reason` の誤り、`skipped` に `skip_reason` なし、`success` に `skip_reason` あり、`pending` に `exit_code` あり | `state file is corrupt: status の値が誤っています` / `skip_reason が status と合いません` / `exit_code が status と合いません` など([06](06-state-file.md) の矛盾) |
| AC-310 | ジョブの並びの矛盾 | ジョブID の重複 / `job_order` と `job` の不一致 / `resume_job_id` が `job_order` にない / `completed: true` と `resume_job_id` | `state file is corrupt: ジョブIDが重複しています` / `job の並びが job_order と一致しません` / `resume_job_id が job_order にありません` / `completed と resume_job_id が両立しません` |
| AC-311 | 読めないファイル(UNIX 系、非 root) | ステートの権限を 000 に | `cannot read state file: 開く、または読み取ることができません` |
| AC-312 | 定義との食い違い: パイプライン名 | `pipeline_name` が定義と違う(定義側の名前を変えて `run`) | `conductor: error: conductor.yaml.state: state does not match the definition: pipeline name; run `conductor reset` to discard the state` |
| AC-313 | 定義との食い違い: ジョブID | ジョブの追加・改名のあと | `state does not match the definition: job ids; …` |
| AC-314 | 定義との食い違い: ジョブの並び | ジョブの順序を入れ替えたあと | `state does not match the definition: job order; …` |
| AC-315 | 食い違いの案内は run ID ありで変わる | `--run-id r1` のステートで定義を変える | `…; run `conductor reset --run-id <ID>` to discard the state` |
| AC-316 | 日本語 | `{garbage`、`LANG=ja_JP.UTF-8` | `conductor: error: conductor.yaml.state: ステートファイルが壊れています: JSON として読めません; `conductor reset` でステートを初期化できます` |

ステートの異常のあとの回復と、正常な読み込み。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-317 | reset で回復する | 上のいずれかのあと | `reset`、続けて `run a` | 0。先頭から実行される |
| AC-318 | pipeline も同じ | 上のいずれか | `pipeline p` | 65。ジョブは起動しない |
| AC-319 | ファイルなし・長さ 0 は「空」 | ステートなし / 長さ 0 | `run a` | 通常どおり 0 |
| AC-320 | 完了のまま残ったステートは「空」 | 完了のステート(正しい) | `pipeline p` | 先頭から実行して 0。読み込みでファイルは消されない(実行の最初の書き込みで上書きされる) |
| AC-321 | 残った一時ファイルは読み込みに影響しない | `.conductor.yaml.state.tmp.1.2.3` がある | `run a` | 通常どおり 0 |
| AC-322 | 書き込みは原子的 | 書き込み前後 | (ステートを読み続ける) | 読み手は、直前の完全な内容か新しい完全な内容だけを見る(同じディレクトリの一時ファイルへ書き、`file.flush()` と `os.fsync(fd)` で同期し、`os.replace` で置き換える。`os.replace` は、置き換え先が既にあっても、UNIX 系・Windows の両方で置き換える。原子性は、UNIX 系では `rename(2)` に依る。Windows の同時の読み手に対する原子性は、未確認)。書き込みの失敗でも直前の内容が残る。置き換え先がディレクトリなら書き込みの失敗でステートは変わらず、一時ファイルも残らない。ディレクトリの `fsync` は、UNIX 系で行い、Windows では行わない(できない。07 章・06 章) |
| AC-323 | 排他制御はしない | 2 つの `run` を同時に | (2.1 節) | 両方が動く。AC-404 を参照 |
| AC-324 | 終了と書き込みの間の停止は再実行(at-least-once) | 終了の直後、書き込みの前に `conductor` が止まる | 再度 `run a` | ジョブが再実行されうる。**外からは再現できない**(偽の部品で検査する)。区分: 注入 |

### 4.12 引数の文法

特に記さない限り、定義は `B0`、OUT は空、ERR は `conductor: error: <文>`⏎(空行)⏎`Usage:` から始まる使い方の全文。「68」の行は、終了 68。

| ID | 機能 | 実行 | 期待 |
|----|------|------|------|
| AC-325 | 引数なし | `conductor` | 68。`missing argument` |
| AC-326 | オプションだけ | `-v` | 68。`missing argument` |
| AC-327 | 知らないコマンド | `bogus` / `''`(空文字) | 68。`unknown command: 'bogus'` / `unknown command: ''` |
| AC-328 | 必須の引数がない | `run` / `pipeline` / `convert` | 68。`missing argument` |
| AC-329 | 余分な引数 | `run a b` / `validate a b` / `watch x` / `reset x` | 68。`unexpected argument: 'b'` / `'b'` / `'x'` / `'x'` |
| AC-330 | 使えないオプション | `watch -v`、`reset -v`、`run --all a`、`run --to yaml a`、`validate --remove-on-complete`、`reset --remove-on-complete`、`convert f --to yaml -v` | 68。`option not allowed for this command: '<そのオプション>'` |
| AC-331 | `--run-id` が使えるコマンド | `run`、`pipeline`、`watch`、`reset` に `--run-id x` | 文法として受理される(4 コマンド)。`validate`・`convert` では 68 |
| AC-332 | 知らないオプション | `run --bogus a` / `run -x a` / `run --bogus=x a` | 68。`unknown option: '--bogus'` / `'-x'` / `'--bogus'`(`--conf` のような省略形は、`--config` として受理せず、`unknown option: '--conf'`。`argparse` の既定の動作と違う) |
| AC-333 | 値の欠落 | `run a --config` / `run a --run-id` / `convert f --to` | 68。`option requires a value: '--config'` / `'--run-id'` / `'--to'` |
| AC-334 | 重複 | `run -vv a`、`run -v -v a`、`run -v --verbose a`、`run a --config x --config y`、`run a --run-id x --run-id y`、`reset --all --all`、`run a --remove-on-complete --remove-on-complete`、`convert f --to yaml --to json` | 68。`invalid value: '<オプション名>'`(`-vv` は `'-v'`。`argparse` は、重複を後勝ちで黙って受理する) |
| AC-335 | `--all` と `--run-id` | `reset --all --run-id x` | 68。`invalid value: '--all'` |
| AC-336 | 値を取らないオプションに `=値` | `run --verbose=x a`、`reset --all=x`、`run a --remove-on-complete=x` | 68。`invalid value: '--verbose'` / `'--all'` / `'--remove-on-complete'` |
| AC-337 | `--to` の値 | `convert f --to xml` / `YAML` / `Json` | 68。`invalid value: 'xml'` など(値そのもの) |
| AC-338 | `--to` がない | `convert f` | 68。`missing argument: '--to'` |
| AC-339 | `--run-id` の値の検査 | `run a --run-id 'a b'` | 68。`invalid value: '--run-id'`(値を含めない) |
| AC-340 | 空の `--config` | `run a --config=` | 64。`definition file does not exist`(空のパスとして受理され、後で誤りになる) |
| AC-341 | オプションの書き方 | `--config c.yaml -v run a`、`run a -v`、`--config=c.yaml`、`--run-id=r1`、`convert f --to=yaml` | すべて文法として正しい(`run` は実行される。`--run-id=` は run ID なしに固定) |
| AC-342 | `--` のあとは位置引数 | `run -- -weird` / `-- --help` / `run a -- -v` | `job is not defined in the definition file: '-weird'`(68)/ `unknown command: '--help'`(68)/ `unexpected argument: '-v'`(68) |
| AC-343 | ヘルプ(英語) | `--help`、`-h` | 終了 0。OUT = [08](08-output-and-i18n.md) の使い方の全文(末尾は改行)。ERR 空 |
| AC-344 | ヘルプ(日本語) | `LANG=ja_JP.UTF-8` で `--help` | 0。OUT = 08 章の日本語の全文 |
| AC-345 | バージョン | `--version`、`-V` | 0。OUT = `conductor 0.8.0`(版は配布の版)。ERR 空。`LANG=ja_JP.UTF-8` でも英語・同じ |
| AC-346 | `-h`/`-V` は最優先 | `-vh`、`bogus --help`、`run -vv --help`、`-V -h`、`-h -V` | `-vh` は help(0)。`bogus --help` も help(0)。`-V -h` は version、`-h -V` は help(先に現れた方) |
| AC-347 | 使い方の誤りの形 | `bogus`(`LANG=C`)、`bogus`(`LANG=ja_JP.UTF-8`) | ERR の最初の行 = `conductor: error: unknown command: 'bogus'` / `conductor: error: 知らないコマンドです: 'bogus'`。2 行目は空行、3 行目以降は使い方の全文(英語 / 日本語)。OUT 空。ジョブは実行されない。**終了コードは 2 ではなく 68**(`argparse` の既定のエラーの終了コード 2 を使わない) |
| AC-348 | 使い方の誤りの全種類 | 上の各種類 | 8 種類(`UnknownCommand`、`MissingArgument`、`UnexpectedArgument`、`UnknownOption`、`OptionNotAllowed`、`MissingValue`、`InvalidValue`、`UnknownJob`)が、すべて 68 で、08 章の英語・日本語の文になる |
| AC-349 | UTF-8 でない引数(UNIX 系) | 定義あり。位置引数(ジョブID)・`--run-id` の値・`--config` と `convert` の入力パスに不正なバイト列(`b"\xff\xfe"`。AC-406 と同じ方法(`Popen` の引数列に `bytes` を渡す)で渡す)。実行: `run <不正なバイトのジョブID>`、`run a --run-id <不正なバイト>`、`--config <不正なバイトのパス>`、`convert <不正なバイトのパス> --to json` | Python は、不正なバイトを `surrogateescape` で孤立サロゲート(`"\udcff\udcfe"`)にして `sys.argv` に渡す(1 章で `os.fsdecode(b"\xff")` を確認)。ジョブID: 68、`unknown job`(定義にないため)。`--run-id`: 68、`invalid value`(孤立サロゲートが run ID の文字集合の外のため)。パス: そのパスのファイルが**存在すれば、元のバイト列のファイル名で開ける**(`open` が `os.fsencode` で元のバイトに戻す)。存在しなければ、`--config` は 64、`convert` は 67。ファイル名に任意のバイトを許さないファイルシステム(macOS の APFS など)では、ファイルを作れないので、存在する場合の項目は Linux だけで行う(未確認)。いずれも異常終了しない(`UnicodeEncodeError` で落ちない)。ERR の孤立サロゲートは、エスケープ規則(08 章)どおりに出る(AC-519)。区分: UNIX 系 |
| AC-447 | 値を取るオプションの値は `-h`/`--help` でも値 | `--config --help run a`、`--config -h run a`、`run a --config --version`(定義ファイルは存在しない) | 終了 64。help・version は出ない。ERR に、その語をパスとしたファイルの誤り(`--help: ...` など)(`argparse` は、`-` で始まる語を値にしないので、この文法に合わない) |
| AC-448 | 値を取るオプションが値を欠く | `--config --help` / `--help --config` | 前者は 68(`missing argument`。`--help` が値に取られて位置引数が 0 個)。後者は help(0) |
| AC-449 | `-h`/`-V`/`-` と `--` の細部 | `-h=x`、`-V=1`、`--help=x`、`-vh=x`、`run -`、`-- run`、`-- run a` | `-h=x`・`--help=x`・`-vh=x` は help(0)、`-V=1` は version(0)。`run -` はジョブID `-`(定義がなければ 64)。`-- run` は 68(`missing argument`。`run` がコマンド名でジョブIDなし)。`-- run a` は `run` のジョブID `a`(定義がなければ 64) |

引数は、標準ライブラリの `argparse` を使わず自前で解析する([04](04-cli.md))。理由(CPython 3.9.6 で確かめた範囲): `argparse` は、(1) オプションの省略形(`--conf`)を既定で受理する(`allow_abbrev`)、(2) 重複したオプション(`--config=x --config=y`)を後勝ちで黙って受理する、(3) `-vv` のような連結を受理し、重複の誤り(AC-334)にならない、(4) エラーを終了コード 2 で終える(この仕様では 68)、(5) `--config --help` を「値が 1 つ要る」エラーにする(この仕様では、`--help` を値にする。AC-447)。上の表の文法(`--opt=value`、短縮形の連結、`--`、`-h`/`-V` の最優先、誤りの優先順位)を、`argparse` の設定だけで満たせるかは、この文書の作成時には確かめていない(未確認。10 章)。

### 4.13 出力とロケール

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-350 | 言語の決定: `LC_ALL` が最優先 | `LC_ALL=ja_JP.UTF-8`、`LANG=C` | `bogus` | ERR に `知らないコマンドです`(日本語) |
| AC-351 | 最初の空でない値だけを見る | `LC_ALL=C`、`LANG=ja_JP.UTF-8` | `bogus` | 英語(`unknown command`) |
| AC-352 | `LC_MESSAGES`、`LANG` の順 | `LC_MESSAGES=ja_JP`、`LANG=C` / `LANG=ja_JP`(ほか未設定) | `bogus` | 日本語 / 日本語 |
| AC-353 | 空の値は飛ばす | `LC_ALL=`(空)、`LANG=ja_JP.UTF-8` | `bogus` | 日本語(空の `LC_ALL` は無視) |
| AC-354 | 大文字小文字を区別する | `LANG=JA` / 未設定 | `bogus` | 英語 |
| AC-355 | 標準出力と標準エラーの使い分け | `P(p; a:0)` | `run -v a` ほか | `run`・`pipeline`・`reset`・`validate` は、OUT に何も書かない。OUT に書くのは `watch`、`--help`、`--version`、`convert` だけ。ジョブ本体の出力は、そのまま流れる |
| AC-356 | 成功・スキップで表示なし | `P(p; a:0)`、`LANG` に不正な値 | `run a` | OUT・ERR 空(表示がなければ言語も色も判定しない) |
| AC-357 | -v のトークンは言語によらず英語 | `LANG=ja_JP.UTF-8`、`P(p; a:1{w=[1]})` | `run -v a` | ERR = `[conductor] job=a result=warning (終了コード 1)`(`job=`、`result=`、`warning` は英語) |
| AC-358 | -v の補足: 起動できない | 存在しないプログラム、`-v` | `run -v a` | ERR に `[conductor] job=a result=error (could not start)` と、続いて `conductor: error: job 'a': cannot start command '…'`。日本語は `起動できませんでした` |
| AC-359 | -v の補足: シグナルで終了(UNIX 系) | `kill -9 $$` のジョブ | `run -v a`(日本語も) | `terminated by signal 9` / `シグナルで終了 9` |
| AC-360 | -v の補足: 中断(UNIX 系) | `SL`、SIGTERM | `run -v sleeper` | `interrupted by signal 15`(日本語 `シグナルで中断 15`)。起動前の中断は `interrupted by signal N, not started` / `シグナルで中断 N, 起動せず`(注入) |
| AC-361 | 全種類のイベントの形 | [08](08-output-and-i18n.md) のイベント | (表示の部品を直接呼ぶ) | すべての文言が空でなく、改行で終わり、生の制御文字を含まない。誤りの行は `conductor: error: ` で始まる。英語の表示は日本語の文字を含まない(原因の文を除く)。区分: 単体 |
| AC-362 | 定義の誤りの形 | `P(…)` の誤り | `validate` | 1 行ずつ `conductor: error: <ファイル>[:<行>]: <種類の文>: <原因>`。原因は日本語の固定の文(英語のロケールでも) |
| AC-363 | 定義の誤りの種類の文(日本語) | `LANG=ja_JP.UTF-8` | 4.1 節の各誤り | 種類の文が 08 章の日本語(例: `定義にないキーです`、`ジョブが 1 つもありません`) |
| AC-364 | 定義の誤りの種類の文(英語・retry) | 4.1 節の retry の各誤り | `validate` | 08 章の英語の文(`retry` の 9 種) |
| AC-365 | ステートの誤りの補足 | 4.11 節の各誤り | (文脈ごと) | 読み込み: 案内 `; …`。書き込み: ` (the job was not started)` / ` (the job has already been executed)`。`reset`・`watch` 以外の読み込みは案内つき、`reset` の削除は補足なし |
| AC-366 | 拒否・完了判定不能・出力失敗の文言 | 08 章 | (表示の部品を直接呼ぶ) | `cannot run this job now: …`、`cannot determine whether the pipeline is complete`、`cannot write to standard output` の英語・日本語。区分: 単体 |
| AC-367 | 完了を判定できない | 全ジョブが「実行済み」で通過し、どこでも完了にならない状態(通常は起こらない) | `pipeline p` | 65。ERR = `conductor: error: conductor.yaml.state: cannot determine whether the pipeline is complete`。ステートを初期化しない。区分: 注入 |
| AC-368 | 内部の誤りの文は固定の英語 | `cli/` が例外(`raise RuntimeError(...)`)を投げる(偽の部品に投げさせる) | (入口の `main` の `try`/`except`) | ERR = `conductor: error: internal error`(ロケールによらず。例外のメッセージとトレースバックは出さない)、終了 70。標準エラーが閉じていても 70。区分: 注入 |
| AC-369 | 入口に異常終了を起こすコマンドがない | 製品の入口 | (点検) | 入口(引数の解釈の結果)に、わざと異常終了するコマンドがない。入口は 1 回だけ終了コードを返す。点検の項目は AC-421、AC-500 |

### 4.14 表示のエスケープ

入力に由来する文字列(ジョブID、名前、パス、キー名、コマンド名、使い方の誤りの対象)は、表示する前にエスケープする。**ジョブ本体の出力は通さない**(そのまま)。

規則: LF → `\n`、CR → `\r`、TAB → `\t`。そのほかの制御文字(U+0000〜U+001F、U+007F〜U+009F)と、書式・方向制御の文字(U+200B〜U+200F、U+2028、U+2029、U+202A〜U+202E、U+2060〜U+2064、U+2066〜U+2069、U+FEFF)は、`\u{<小文字の 16 進。先頭の 0 なし>}`。
孤立サロゲート(U+D800〜U+DFFF。非 UTF-8 の引数が `surrogateescape` で変わったもの)は、`\u{<小文字の 16 進>}`(例: `\u{dcff}`)とする(08 章に従う。この文書の例は、その字句に合わせた)。
そのほかの文字(日本語を含む)はそのまま。名前用は、エスケープ後 64 文字(**コードポイント**。Python の `len(str)` はコードポイントの数なので、そのまま数える)で切る(エスケープ列は 1 かたまり。次の 1 文字を足すと 64 を超えるなら、そこで打ち切り、末尾に `…`(U+2026)を 1 つ付ける。出力は最大 65 文字)。パスは切らない。サロゲートペアは、Python の `str` には現れない(1 コードポイント)ので、途中で切る問題は起きない。結合文字の列は、コードポイントごとに数える(書記素では数えない)。

外から確かめる方法: `B0` に対して `run '<入力>'`(定義にないジョブID)を実行し、ERR の最初の行が `conductor: error: job is not defined in the definition file: '<出力>'` になることを確かめる。終了 68。

| ID | 機能 | 入力 | 出力(期待) |
|----|------|------|--------------|
| AC-370 | LF と ESC | `a<LF>b<ESC>[0m` | `a\nb\u{1b}[0m` |
| AC-371 | TAB | `k<TAB>x` | `k\tx` |
| AC-372 | CR | `a<CR>b` | `a\rb` |
| AC-373 | 日本語と空白 | `日本語 abc` | `日本語 abc`(そのまま) |
| AC-374 | ゼロ幅スペース | `a<U+200B>b` | `a\u{200b}b` |
| AC-375 | 右から左への上書き | `a<U+202E>b` | `a\u{202e}b` |
| AC-376 | 方向の隔離 | `a<U+2066>b` | `a\u{2066}b` |
| AC-377 | BOM | `a<U+FEFF>b` | `a\u{feff}b` |
| AC-378 | 行・段落の区切り | `a<U+2028>b<U+2029>` | `a\u{2028}b\u{2029}` |
| AC-379 | その他の制御文字 | `<U+007F><U+0085>`(NUL は引数に渡せないため、単体で確かめる) | `\u{7f}\u{85}`(NUL は `\u{0}`。区分: 単体) |
| AC-380 | 64 文字ちょうど | `k` × 64 | `k` × 64(切らない) |
| AC-381 | 64 文字を超える | `k` × 200 | `k` × 64 + `…` |
| AC-382 | エスケープ後で数える | LF × 100 | `\n` × 32 + `…`(65 文字) |
| AC-383 | 上限 3(単体)とコードポイントの数え方 | `abcdef`、上限 3。`😀` × 64 と `😀` × 65(`😀` は U+1F600。Python では 1 コードポイント) | `abc…`。空の入力は空。`😀` × 64 は切らない、`😀` × 65 は `😀` × 64 + `…`。区分: 単体 |
| AC-384 | 全部の部品で同じ結果 | 上の表の入力 | 定義・ステート・変換の誤りを作る各部品が、この表と同じ結果を返す(共通の関数を使ってよい)。区分: 単体 |
| AC-385 | ジョブIDは 64 文字で切る | ジョブID が 200 文字の定義 | `watch` の行・`-v` の行・`run` の誤りで、ID が 64 文字 + `…` |

表示の上限と、各部の適用。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-386 | パスは切らない | 300 文字のファイル名の定義(存在しない) | `validate --config <パス>` | ERR にパスの全体が現れる |
| AC-387 | リトライの知らせのジョブID | ジョブID に制御文字・200 文字 | リトライの行 | エスケープされ、64 文字で切られる |
| AC-388 | 定義の誤りのキー名 | 定義のキー名に制御文字 | `validate` | キー名がエスケープされ、64 文字で切られる |

### 4.15 終了コード全件

「終了コードの全件」の確認。各コードを起こす受け入れテストを、表で示す。

| 終了コード | 意味 | 起こすテスト |
|------------|------|--------------|
| 0 | 成功・警告・スキップ・`watch`・`reset`・`validate`・`convert` の成功・`--help`・`--version`・`reset --all` の警告 | AC-389 |
| 1 | 成功コードに含まれない 0、エラーで下位 8 ビットが 0 | AC-390 |
| ジョブの値(1〜255) | エラーのジョブの終了コード | AC-391 |
| 64 | 定義の誤り・`command` の誤り | 4.1 節 |
| 65 | ステートの誤り・書き込み・削除の失敗・`watch` の出力失敗・完了判定不能 | 4.11 節、4.3 節、4.6 節、4.5 節 |
| 66 | 拒否(順序外) | 4.3 節 |
| 67 | 変換の誤り・`convert` の出力失敗 | 4.7 節 |
| 68 | 使い方の誤り(知らないジョブID、run ID の誤りを含む) | 4.3、4.8、4.12 節 |
| 70 | 内部の誤り(未捕捉の例外を含む)・シグナルの準備の失敗 | 4.10 節・4.13 節(注入)、AC-490、AC-491、AC-492 |
| 127 | ジョブを起動できなかった | 4.2 節 |
| 128 + 番号 | ジョブがシグナルで終了、または中断 | 4.3 節、4.9 節、4.10 節 |

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-389 | 終了 0 の全種類 | 成功 / 警告(`w=[3]` で `exit 3`)/ スキップ(実行済み)/ `watch`・`reset`・`validate`・`convert`・`--help`・`--version` の成功 / `reset --all` の警告 | 各実行 | すべて 0 |
| AC-390 | 終了 1 の 2 種類 | `P(p; a:0{s=[5]})`(成功コードにない 0)/ エラーで下位 8 ビットが 0 になる終了コード | `run a` | 1 / 1(`exit 256` は Windows で確かめる。UNIX 系では OS が 0 にするので成功。関数の検査は下) |
| AC-391 | ジョブの値 | `exit 1`、`exit 2`、`exit 7`、`exit 255` | `run a` | 同じ値 |
| AC-392 | 終了コードの範囲合わせ(関数の検査) | 報告する値 `reported`、`is_error` | 範囲合わせの関数(`flow/`) | `0..255` はそのまま。`259`→3、`511`→255、`-1`→255、`-2`→254、`0xC0000005`(32 ビット符号つき。符号なしの `3221225477` で渡されても同じ)→5、`4294967295`(符号なしで渡された -1)→255。`256`・`512`・`-256`・32 ビットの最小値は、エラーなら 1、エラーでなければ 0。Python の `%`(負の数でも非負になる)と `&` の違いに注意する(`-1 % 256 == 255`、`-1 & 0xFF == 255`。`int(x) % 256` と `x & 0xFF` は、負でも同じ結果になるが、「下位 8 ビットが 0 なら 1」の規則との組み合わせを検査する)。区分: 単体 |
| AC-393 | 中断の終了コード(関数の検査) | シグナル番号 n | `128 + n` を範囲合わせ(エラー扱い) | 15→143、2→130、1→129、3→131、300→172(範囲を超えない)。区分: 単体 |
| AC-394 | 分類の全 10 項目(関数の検査) | [07](07-process-and-signals.md) の分類の表の各行 | 分類の関数(`flow/`) | 順 1: 起動前の中断 → 分類なし、`128 + n`。順 2: 起動なし・中断でもない → エラー 127。順 3: 起動済みの中断 → `128 + n`(ジョブの終了コードを見ない)。順 4: 起動失敗 → 127。順 5: シグナル終了 n → `128 + n`。順 6: 終了コードなし → 1(`conductor` のエラー)。順 7: 成功コード → 0。順 8: 警告コード → 0。順 9: 0 で成功コードにない → 1(`conductor` のエラー)。順 10: そのまま。優先順位は上の行ほど高い。区分: 単体 |
| AC-395 | 終了コードの決定順序 | 複数の誤りを同時に持つ入力 | `run` | 検査の順(引数 68 → 準備 70 → 定義 64 → `command` 64 → ジョブID 68 → run ID 68 → ステートの読み込み 65 → 照合 65 → 起動前の判断 66/0 → 初期ステート 65 → ジョブ → 結果の書き込み 65 → 削除 65)で、最初の誤りだけが採られる。具体例は 4.3 節(定義の誤り > ジョブID)、4.8 節(定義 64 > run ID 68 > ステート 65) |
| AC-396 | 128 + 番号の例 | ジョブが SIGKILL / 実行中に SIGTERM(UNIX 系) | `run a` | 137 / 143 |

### 4.16 非機能要件の検証

2 章の各基準の受け入れテスト。「計測」は p95 などの測定、「強制」は基準超過で不合格(2 章の扱いの列)。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-397 | 定義の読み込みの性能 | ジョブ 10 個の定義(ファイルから読む) | 最初の 5 回を捨てて、読み込みと検証を 100 回(プロセス内) | p95 が 20 ms 以下(目安。超えたら警告を記録)。区分: 計測 |
| AC-398 | 起動前の処理の性能 | ジョブ 10 個の定義 | `command` の検査、ステートの読み、照合、速い書き込みを、最初の 5 回を捨てて 100 回 | p95 が 20 ms 以下(仮。目安)。区分: 計測 |
| AC-399 | `run` のオーバーヘッド | ジョブ 10 個の定義(各ジョブは `TJ timing`) | `run job0` を 20 回 | `conductor` 自身の時間(全体 − `TJ` の基準の起動の費用 − (`end_ns` − `start_ns`))が p95 350 ms 以下(仮。目安)。区分: 計測 |
| AC-400 | `pipeline` のジョブあたり | 同上 | 10 個と 1 個の `pipeline` を各 5 回 | ジョブ 1 つあたりの増分(2.1 節の式)が p95 50 ms 以下(目安)。区分: 計測 |
| AC-401 | 大きな定義 | ジョブ 1,000 個の定義 | `run job0` | 1,500 ms 以内(仮。目安)。区分: 計測 |
| AC-402 | 大きな流れ | ジョブ 1,000 個の `pipeline`(偽の部品で、プロセスもファイルも使わない) | (内部の流れ) | 10,000 ms 以内(目安)。完了のステートが残ることは常に検査。区分: 計測・単体 |
| AC-403 | 待機中の CPU 時間(Linux) | `TJ sleep 4`(約 4 秒)のジョブ | `run a` を起動し、`conductor` の pid の `/proc/<pid>/stat` の `utime + stime`(14・15 番目の値。単位は時計の刻み。`os.sysconf("SC_CLK_TCK")` で秒に直す)を、起動の 1 秒後と 2.5 秒後に読む | 増分が 50 ms 以下(待機中に、短い間隔の確認で CPU を使いすぎない)。強制(`/proc` が読めなければスキップ)。区分: Linux |
| AC-404 | 同時の 2 つの `run` | `P(p; a:0{cmd="TJ sleep 1"})`(終了コード 99) | 同じ定義に `run a` を 2 つ同時に | 両方が実行され(ジョブが 2 回始まる)、どちらも終了コード 99。ステートは正しい形で読み戻せる。強制 |
| AC-405 | ランダムな引数 | 固定の種(`Xorshift32(1)`) | 語の表から 0〜6 個を選んだ引数列で 500 回(`cli/` を偽の部品で、入口の関数を通して呼ぶ) | 終了コードが 0〜255。標準エラーに、制御文字(改行を除く)と U+202E がない。例外が漏れない |
| AC-406 | UTF-8 でない引数(UNIX 系) | `0xff 0xfe` を含むバイト列。`subprocess.Popen` は、UNIX 系で、引数列に `bytes` を受け付けるので、`Popen([*conductor_cmd, b"run", b"\xff\xfe"])` のように直接渡す(シェルを使わない) | ジョブID・`--config`・オプション・`convert` の引数に渡す | 異常終了しない(終了コードが 0〜255。70 にならない)。標準エラーが、有効な UTF-8 で読める。区分: UNIX 系 |
| AC-407 | 壊れたステート | 状態・再開位置・フラグを乱したステート、ランダムなバイト列、巨大な値 | `run` / `watch` | 異常終了しない。壊れていれば 65 |
| AC-408 | 空・制御文字のジョブID | ジョブID が空、制御文字を含む | `run <ID>` | 異常終了しない(68) |
| AC-409 | ランダムな `command` | 文字集合 `a b ⇥ ⏎ ' " \ ; \| & $ * あ` から長さ 0〜39 の文字列 2,000 件(定義の `command` に入れる) | `validate`(分割の単体も) | 0 か 64 のどちらか。0 のとき `program` は空でない |
| AC-410 | 大きな `command` | `prog ` + `'a b' "c\"d" e ` を 10 万回(約 30 万引数) | `validate` | 0。閉じない引用符を足したものは 64。30 秒以内。文字列の `+=` の繰り返しによる 2 乗の時間や、再帰で `RecursionError` にならない(リストに積み、最後に `join` する) |
| AC-411 | 起動できない `command` | `''`、`'`、`""`、存在しない名前、ディレクトリ、`/` | `validate` / `run a` | 前 3 つは 64、残りは 127。異常終了しない |
| AC-412 | 入れ子の深い入力 | 深さ 1,000 の YAML と JSON(定義、ステート) | `validate` / `run a` / `convert` | 64 / 65 / 67。異常終了せず(`RecursionError` が漏れて 70 にならず)、`sys.setrecursionlimit` を上げなくても動く |
| AC-413 | 変換の頑健性 | 範囲外・空・壊れた・切り詰めた入力、ランダムな構造 | `convert` | 異常終了しない。成功した構造は往復で等しい |
| AC-414 | 内部の誤り | 異常終了を起こす | (入口) | 4.13 節の「内部の誤りの文」と同じ(AC-368)。実際のプロセスでの検査は AC-490、AC-491 |
| AC-415 | 検証は副作用を持たない | 4.1 節の最後の項目 | — | AC-057 |
| AC-416 | 入力の値を表示に出さない | 各種類のエラー(定義・`command`・ステート・変換・使い方)に、秘密のつもりの文字列を含む入力。YAML ライブラリが、入力の断片を含むエラー文を返す入力(例: 不正な文字を含む行、閉じていない引用符の行)も含む | 各コマンド | ERR に、その値が現れない(キー名は、エスケープして現れてよい)。`command` の誤りの文に `command` の中身が現れない。例外のトレースバック(`Traceback (most recent call last):`、`  File "` で始まる行)が現れない |
| AC-417 | ステートに機微な値を書かない | 4.3 節の項目 | — | AC-148 |
| AC-418 | 変換に副作用がない | 4.7 節の項目 | — | AC-235。加えて、`src/conductor/convert/` が `subprocess`・ネットワークのモジュールを import せず、ファイルを開くのが入力ファイルの読み取りの入口だけであること(点検) |
| AC-419 | ネットワークに接続しない | 実装と依存 | 点検(`ast` で `import` と `from … import` を走査。インストールされた実行時の依存のパッケージの `.py` の文字列検索) | 製品のコードが `socket`・`ssl`・`urllib`・`http`・`ftplib`・`smtplib`・`poplib`・`imaplib`・`telnetlib`・`xmlrpc`・`socketserver`・`asyncio`・`selectors` を import せず、`requests`・`httpx` なども import しない。実行時の依存のソースにも、実行中に通信するコードがない(YAML ライブラリは未確認)。区分: 点検 |
| AC-420 | OS 固有コードの隔離と、動的実行・ネイティブの不使用 | ソース | 点検(`ast` の走査) | `sys.platform`、`os.name`、`platform.system()`、`platform.platform()` の参照が、`src/conductor/runner/platform.py` の外にない。`eval`、`exec`、`compile`、`__import__`、`importlib` の動的な読み込み、`pickle`・`marshal`・`shelve`、`ctypes`・`cffi`、`multiprocessing`、`runpy`、`threading` の import が、どこにもない(0 か所。`threading` を使う場合は、使う理由・所有者・終了条件を 02・07 章に表で示し、この項目の許可の一覧に加える)。区分: 点検 |
| AC-421 | 異常終了を起こす書き方がない | ソース、`ruff` の設定 | 点検(`ast` の走査と、`ruff` の規則) | 製品のコード(`src/`)に、(1) `sys.exit(`・`os._exit(`・`raise SystemExit`(`__main__.py` の 1 か所以下を除く。AC-500)、(2) `assert` 文(`ruff` の `S101`。`python -O` で消えるため、検証に使わない)、(3) `typing.cast`、(4) `Any` の型注釈(`ANN401`)と `# type: ignore`・`# noqa`・`# pyright: ignore` による無効化、(5) `json.loads`・`json.load`(自前の読み取りを使う。03 章が使ってよい範囲を定めた場合は、その範囲を除く)、(6) `yaml.load`・`yaml.unsafe_load`・`yaml.full_load`(AC-531)、(7) 裸の `except:`、`except Exception` / `except BaseException` で黙って捨てる書き方(`BLE001`、`S110`)、(8) `print(`(`T201`。出力は `report/` を通す)がない。`BaseException` の捕捉は `__main__.py` だけ。配列・辞書の添字は、`IndexError`・`KeyError` を起こしうる箇所で、先に検査する。区分: 点検 |
| AC-422 | テスト専用の経路を持たない | ソース、配布物 | 点検 | 入口に、異常終了を起こす経路や、テスト用の環境変数・隠しオプションがない。`tests` を、`src/` が import しない。wheel のファイル一覧に、`tests/`・`scripts/`・`demo/`・設定ファイルが含まれない(含まれるのは `conductor/` のパッケージと `*.dist-info/`。AC-526)。テスト用ジョブ `tj.py` が入っていない |
| AC-423 | 著作権表示 | ソース | 点検 | 各 `.py` ファイルの先頭(`#!` の行や、符号化の宣言があればその次)に Apache License 2.0 のヘッダ(`Copyright 2026 agwlvssainokuni`)がある。`.yml`・`.sh`・`.cmd` も同じ(コメントの書式は各言語に合わせる)。`.venv/`・`build/`・`dist/`・`*.egg-info/`・`tests/fixtures/` は対象外。区分: 点検 |

### 4.17 デモの受け入れテスト

デモの仕様は 7 章。実行は、`CONDUCTOR=<被試験体>`(`conductor` のコンソールスクリプトのパス、または `python -m conductor` を表すコマンド)、`DEMO_LANG=en_US.UTF-8`、`DEMO_SLEEP=0`、一時ディレクトリの親(UNIX 系は `TMPDIR`)を空のディレクトリ `D` にして行う。標準入力は空。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-424 | UNIX 系: スクリプトが全部通る | 上の設定 | `sh demo/run-demo.sh` | 終了 0。見出し `=== 0. ` から `=== 10. ` までが順に出る。区分: UNIX 系 |
| AC-425 | UNIX 系: 場面 7(検査) | 同上 | 同上 | 出力に `definition is valid`、`command has an unclosed quote`、`→ 終了コード 64` が現れる。誤りの定義の検査のあと、ステートの一覧が `(なし)` |
| AC-426 | UNIX 系: 場面 4(再開・完了) | 同上 | 同上 | `completed` の行、`skipped (already executed)` が現れる |
| AC-427 | UNIX 系: 場面 8(run ID) | 同上 | 同上 | `conductor.yaml.build-1.state` が一覧に現れる。再開のあと `completed`。`reset` のあと一覧は `(なし)` |
| AC-428 | UNIX 系: 場面 9(環境変数と後始末) | 同上 | 同上 | `conductor.yaml.opt-1.state` が現れ、最後の `watch` が `no state` |
| AC-429 | UNIX 系: 場面 10(リトライ) | 同上 | 同上 | `conductor: retry: job 'flaky' exited with code 75; retry 1 of 3 in 1 seconds` と `… retry 2 of 3 in 1 seconds` の 2 行、`[flaky] 3 回目の実行`、`job=flaky result=success` が現れる |
| AC-430 | UNIX 系: 利用者の環境に左右されない | `CONDUCTOR_RUN_ID=zzz`、`DEMO_RUN_ID=zzz`、`LC_ALL=ja_JP.UTF-8`、`LC_MESSAGES=ja_JP.UTF-8`、`NO_COLOR=1`、`PYTHONIOENCODING=ascii`、`PYTHONUTF8=0` を設定 | 同上 | 出力に `zzz` が現れない。英語の表示(`DEMO_LANG` のとおり)。結果が上と同じ |
| AC-431 | UNIX 系: 何も残さない | 同上 | 同上 | 終了後、一時ディレクトリの親 `D` に何も残らない。リポジトリの作業ツリーは変わらない(`__pycache__/`、`.pytest_cache/`、`build/` などを除く。`.gitignore` の対象) |
| AC-432 | Windows 版: 定義 | 定義 `demo\windows\conductor.yaml` | (検査) | 4 ジョブ(`extract`、`transform`、`load`、`notify`)と、`extract` の警告の遷移(`notify` へ)を持つ。`validate` が 0。区分: Windows |
| AC-433 | Windows 版: 場面 1〜2 | 一時ディレクトリに定義とスクリプトを写す | `pipeline nightly`(場面 1)、`DEMO_EXTRACT_EXIT=1` で `pipeline nightly`(場面 2) | 場面 1 は 0。場面 2 は `notify` へ進み 0 |
| AC-434 | Windows 版: 場面 3〜4 | 同上 | `DEMO_LOAD_EXIT=2` で `pipeline`(2)、続けて `pipeline`(`load` から再開) | 2 のあと、再実行で `load` から再開して 0 |
| AC-435 | Windows 版: 場面 5〜7 | 同上 | `run extract`、`reset`、`convert conductor.yaml --to json`、`validate`、誤りの定義の `validate` | `run`・`reset`・`convert`・`validate` は 0。誤りの定義は 64 |
| AC-436 | Windows 版: 場面 8〜9 | 同上 | run ID を使う一連(場面 8、9) | run ID ごとにステートが分かれる。`--run-id` が環境変数に勝つ。`reset --run-id` と `--all` が働く |
| AC-437 | Windows 版: 場面 10 | 同上 | `pipeline --config retry.yaml retry-demo` | 2 回失敗してから成功する |
| AC-438 | Windows 版: 利用者の run ID に影響されない | 利用者の run ID の環境変数を設定 | 同上 | 結果が変わらない |

### 4.18 リリース物の受け入れテスト

リリースの仕様は 6 章。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-439 | 成果物の名前と形 | リリース | (成果物の一覧) | `conductor-<版>.tar.gz`(sdist)、`conductor-<版>-py3-none-any.whl`(wheel)、`SHA256SUMS` の 3 つがある(配布名を置き換えた場合は、その名前。ファイル名の正規化の規則は、ビルドの道具に従う。実装時に確認)。区分: 点検 |
| AC-440 | 展開の構造 | sdist と wheel | 展開(`tarfile`、`zipfile`) | wheel: 最上位に `conductor/`(`__main__.py` を含む)と `conductor-<版>.dist-info/`(`METADATA`、`WHEEL`、`entry_points.txt`、`RECORD`、ライセンスのファイル)だけがある。`tests/` がない。テスト用ジョブがない。sdist: `pyproject.toml`、`LICENSE`、`README.md`、`src/conductor/`、`PKG-INFO` がある(`tests/` を含めてよいかは 6 章)。`.github/`・`demo/`・`docs/`・`.venv/` がない。区分: 点検 |
| AC-441 | チェックサム | `SHA256SUMS` | `sha256sum -c`(macOS は `shasum -a 256 -c`)の形式で検査 | sdist と wheel の行があり、検査が通る。区分: 点検 |
| AC-442 | 版の表示 | wheel を、空の仮想環境に `pip install` した結果 | `<venv>/bin/conductor --version`(Windows は `<venv>\Scripts\conductor.exe`)と、`<venv>/bin/python -m conductor --version` | 0。OUT = `conductor <版>`。区分: 点検(3 OS) |
| AC-443 | 実行時の依存が 1 つ | wheel の `METADATA` | 点検(`Requires-Dist` の行の走査) | 実行時の依存(`extra` の条件を持たない `Requires-Dist`)が YAML ライブラリ 1 つだけ。wheel にネイティブのファイル(`.so`、`.pyd`、`.dylib`、`.dll`)がない。`Requires-Python` が書かれている。区分: 点検 |
| AC-444 | タグの検査 | タグ `v<版>` | リリースの流れ | タグが `main` 上にない、または版が `src/conductor/version.py` の定数か `pyproject.toml` の `version` と違うと失敗する(タグが `main` 上にない旨 / タグと版の不一致の旨)。区分: 点検 |
| AC-445 | 下書きとプレリリース | リリースの作成 | (作成されたリリース) | 下書きである。版が先行版(PEP 440 の `a`・`b`・`rc`・`.dev` を含む。例: `0.9.0rc1`)ならプレリリース。注記に固定の 1 文がある。区分: 点検 |
| AC-446 | 3 OS で受け入れテストが通る | Linux・macOS・Windows の実機(または同等の実行環境) | 4 章の自動の項目(各 OS で行える項目)。`CONDUCTOR_PYTHON` に、AC-442 で wheel を入れた仮想環境の `python` を渡す形(または `CONDUCTOR_BIN` に、そのコンソールスクリプトを渡す形)でも通ること | 全項目が合格する(OS の区分の項目は、その OS で)。区分: 自動 |


### 4.19 YAML と JSON に固有の項目

定義の形式(YAML・JSON)、ステートの形式(JSON)、変換(YAML ⇔ JSON)の、形式固有の落とし穴を確かめる。特に記さない限り、「64」の期待は 4.1 節と同じ(終了 64、OUT 空、ステートなし、ジョブを起動しない。ERR の最初の行は `conductor: error: <ファイル>:<行>: ` で始まる(行がわかる誤り)。種類の文は [03](03-definition-format.md)・[08](08-output-and-i18n.md) のとおり)。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-450 | JSON の定義 | `P(p; a:0)` を JSON で書いた `T/conductor.json` | `validate --config conductor.json`、`run a --config conductor.json` | 0 / 0。ステートは `T/conductor.json.state`。YAML の定義と同じ結果 |
| AC-451 | `.yml` も受理 | `T/conductor.yml`(`B0`) | `run a --config conductor.yml` | 0。ステートは `T/conductor.yml.state` |
| AC-452 | 拡張子が違うファイルは拒否 | `B0` の内容を、`conductor.xml`、`conductor.txt`、拡張子なしの `conductor`、`conductor.yaml.bak` に置く(ファイルが存在しないパス `nofile.txt` も) | `validate --config <ファイル>` | 64。ERR の最初の行は `conductor: error: <ファイル>: unsupported definition file format: 拡張子は .yaml、.yml、.json のいずれかで書きます`(行なし。[03](03-definition-format.md))。大文字の拡張子 `conductor.YAML` は受理(0)。ステートなし。ジョブは起動されない(`convert` の入力は拡張子を見ない。AC-209) |
| AC-453 | YAML のタグを拒否 | `command: !!str "TJ exit 0"`、`id: !foo a`、`name: !!binary aGk=` | `validate` | 64(それぞれ別の定義)。行は、そのタグの行 |
| AC-454 | YAML の複数の文書を拒否 | `B0` の内容 + `---⏎` + もう 1 つの文書 | `validate` | 64。行は 2 つ目の文書の始まり。先頭に `---` が 1 つだけある `B0` は受理(0) |
| AC-455 | YAML の文字列でないキーを拒否 | `1: x`、`true: x`、`? [a, b]⏎: x`(トップレベルまたは `job` の中) | `validate` | 64 |
| AC-456 | YAML の重複キー(入れ子) | `job` の要素に `id` を 2 回、`pipeline` に `name` を 2 回 | `validate` | 64。`syntax error`。行は 2 回目のキーの行 |
| AC-457 | YAML のアンカー・エイリアス・マージキーは展開して受理 | 既知のキーの中に書く。(a) 1 つ目のジョブに `warning_exit_codes: &w [3]`、2 つ目に `warning_exit_codes: *w`。(b) 遷移 `- <<: {on: success, next: "@end"}`。(c) トップレベルに `defaults: &d {…}` という未知のキー | `validate`、`run` | (a)(b) は 0。展開した値で動く(2 つ目のジョブの警告コードは `[3]`、(b) は `@end` への遷移)。(c) は 64(`unknown key`。アンカーの置き場所のための未知のキーは作れない)。整合: 10 章 |
| AC-458 | 数値・真偽値に見えるジョブID | `id: 1`、`id: true`、`id: "1"`、`id: yes`、`id: on`、`id: 0x10` | `validate`(それぞれ別の定義) | `id: 1`・`id: true`・`id: 0x10` は 64(`wrong value type`)。`id: "1"`・`id: yes`・`id: on` は 0(文字列。YAML 1.2 のコアスキーマの規則。`on`・`yes` を真偽値として読まない。PyYAML の既定の型判定は YAML 1.1 で `yes`・`on` を真偽値にするので、型は自前で決める。AC-525) |
| AC-459 | `command` の書き方(YAML のブロック・折り返し・複数行) | `command: \|⏎  TJ --out O echo a⏎  b⏎`(ブロック)、`command: >-⏎  TJ --out O echo⏎  a b`(折りたたみ)、複数行の plain スカラー(`TJ` は、各書き方に合わせて 2 つの語を書く) | `run a` | 0。`O` の引数は、それぞれ `a`、`b` / `a`、`b` / 同様(改行・空白は区切り) |
| AC-460 | 定義の中の null の扱い | `name: ~`、`command: null`、`warning_exit_codes: ~`、`retry: null`、`remove_on_complete:`(値なし)。JSON: `"command": null` | `validate` | 64。`name`・`command`・`warning_exit_codes`・`remove_on_complete`・JSON の `command` の `null` は `wrong value type`。`retry: null` は `retry must be a map`。未知のキーの値が `null` なら `unknown key`([03](03-definition-format.md))。Python の `None` と「キーがない」を区別する(辞書の `.get()` が `None` を返す場合に、混同しない) |
| AC-461 | JSON の重複キー | `{"format_version": 1, "format_version": 1, ...}`(`conductor.json`)、`job` の要素の `id` の重複 | `validate --config conductor.json` | 64。`syntax error`(理由 `キーが重複しています`。`json.loads` は重複を黙って受理し、後の値が前の値を上書きする。`object_pairs_hook` で検出することはできるが、行・列の位置は得られないので、自前の読み取りで検出する。[03](03-definition-format.md)) |
| AC-462 | JSON のコメントを拒否 | 行コメント `// c`、ブロックコメント `/* c */` を含む定義 | `validate --config conductor.json` | 64。`syntax error` |
| AC-463 | JSON の末尾のカンマを拒否 | `[1,]`、`{"a": 1,}`(定義の中) | `validate --config conductor.json` | 64。`syntax error` |
| AC-464 | JSON の非標準の書き方を拒否 | 一重引用符の文字列、`NaN`、`Infinity`、`-Infinity`(`json.loads` は、この 3 つを既定で受理する。`parse_constant` で拒否するか、自前の読み取りで拒否する)、先頭が 0 の数(`01`)、引用符のないキー、値のあとの余分なデータ(`{...} {...}`、`{...} x`) | `validate --config conductor.json` | 64。`syntax error`(それぞれ別の定義) |
| AC-465 | JSON の最上位がオブジェクトでない | `[]`、`"x"`、`1`、`null`、`true` | `validate --config conductor.json` | 64。`wrong value type`(理由 `最上位はマップで書きます`。JSON の `null` も同じ。[03](03-definition-format.md)) |
| AC-466 | JSON の大きさの上限 | 正しい JSON の定義の末尾に、空白を足して 1,048,576 バイト / 1,048,577 バイト | `validate --config conductor.json` | 0 / 64(`definition file is too large`) |
| AC-467 | JSON の文字列のエスケープ | `"command": "TJ --out O echo \"a b\" c A 😀"`(`TJ` は 2 つの語)、不正なエスケープ `"\x"`、閉じていない文字列、制御文字(生の LF)を含む文字列 | `run a --config conductor.json`(正しいもの)、`validate`(誤り) | 正しいもの: `O` の引数 = `a b`、`c`、`A`、`😀`(サロゲートペアのエスケープが 1 つのコードポイントになる)。誤りは 64(`syntax error`)。孤立したサロゲートのエスケープ(`"\ud800"`)の扱いは 03 章に従う(未確認)。Python の `json.loads` は、これを孤立サロゲートの `str` として受理するので、受理する場合でも、出力で `UnicodeEncodeError` にならない(AC-519) |
| AC-468 | JSON の数値の型 | `"format_version": 1.0`、`1e0`、`"warning_exit_codes": [3.0]`、`[1e1]`、`[99999999999999999999]`、`[true]`、`[NaN]` | `validate --config conductor.json` | `format_version` は 64(`unsupported format_version`)。終了コードは 64(`wrong value type`)。整数のリテラルだけを整数と読む(`float` を経由して丸めた値を見ない。`True` を `1` と見なさない。AC-502) |
| AC-469 | 変換の JSON の出力の文字 | YAML: `a: "<&>"⏎b: 夜間⏎c: " x"` | `convert in.yaml --to json` | 0。OUT に `"<&>"` がそのまま(`&` にならない)と `夜間`(非 ASCII はそのまま。`ensure_ascii=False` 相当)がある。OUT を JSON として読んだ木が入力の木と等しい(`c` の U+2028 は、` ` の書き方でも直接でもよい) |
| AC-470 | 変換の YAML の出力: 予約語に見える文字列 | JSON: `{"a": "true", "b": "1", "c": "null", "d": "@end", "e": "", "f": "~", "g": "on", "h": "1e3", "i": "0x1F", "j": "yes", "k": "-", "l": "a: b", "m": "# c"}` | `convert in.json --to yaml` の OUT を `convert --to json` に通す | 往復の木が入力と等しい(すべて文字列のまま。YAML の出力は自前の書き出しで、必要なものを引用符で囲む) |
| AC-471 | 変換の YAML の出力: 特殊な文字列 | JSON: 複数行の文字列(`"a\nb\n"`)、先頭・末尾に空白のある文字列、タブ、`"\u0007"`(制御文字)、非 ASCII、絵文字、長い文字列(1 万文字)、空のキー `""`、`@` や `%` や `` ` `` で始まるキー | `convert in.json --to yaml` の OUT を `convert --to json` に通す | 往復の木が入力と等しい |
| AC-472 | 変換の JSON の入力の数値 | JSON: `{"a": 1, "b": 1.0, "c": 1e2, "d": -0, "e": 9223372036854775807, "f": -9223372036854775808}` | `convert in.json --to yaml` の OUT を `convert --to json` に通す | 型が変わらない(`a`・`d`・`e`・`f` は整数、`b`・`c` は浮動小数)。往復の木が入力と等しい。`e`・`f` は桁を落とさない |
| AC-473 | ステートの JSON 固有の異常 | ステートが、重複するキーを持つ JSON、値のあとに余分なデータ、最上位が配列 / `null` / 文字列、`{}` | `run a` | 65(ジョブは起動されない)。余分なデータ・最上位の型の誤りは `state file is corrupt: JSON として読めません`、キーの重複は `state file is corrupt: キーが重複しています`、`{}` は `state file is corrupt: format_version がありません` |
| AC-474 | ステートの数値の型 | `exit_code` が `3.0` / `"3"` / `1e1` / `256` / `-1` / `true` / `NaN`、`format_version` が `1e0` | `run a` | 65。`state file is corrupt: …`(型・範囲の理由は 06 章) |
| AC-475 | JSON の `ME` の誤りの行 | `ME` を JSON で書く(4.1 節。次のとおり) | `validate --config conductor.json` | 64。ERR は 4 行。順に `…:3: unknown key`、`…:11: wrong value type`、`…:16: unknown key`、`…:20: transition target job is not defined` |
| AC-476 | 版の表示と定数 | ソース、`pyproject.toml`、実行ファイル | 点検と `--version` | `src/conductor/version.py` の定数 `VERSION`(`VERSION = "0.8.0"` の 1 行)が、ソース中の版の唯一の定義。`pyproject.toml` の `[project]` の `version` と一致する(テストで検査。`scripts/check_version.py`)。`--version` の OUT = `conductor ` + `VERSION`。ビルドの道具の置換や、`importlib.metadata.version()` で版を引いていない(インストールの有無に左右されないため)。区分: 点検 |
| AC-477 | Python の版の記載の一致 | `pyproject.toml`、`README.md`、`.github/workflows/ci.yml` | `python scripts/check_python_version.py` | `pyproject.toml` の `requires-python` の最小(`>=3.11` なら 3.11)、`[tool.ruff]` の `target-version`(`py311`)、README の `Python <最小版> 以降`、`ci.yml` の `PY_MIN` が一致する。`ci.yml` の `PY_MAX`(最新の版)が、`pyproject.toml` の `Programming Language :: Python :: <版>` の分類子の最大と一致する。不一致なら失敗(終了 1)。区分: 点検 |
| AC-478 | 依存の整合 | `pyproject.toml`、`constraints.txt` | `pip install -c constraints.txt ".[dev]"`、`pip check`、`python scripts/check_deps.py` | インストールが成功し、`pip check` が「依存の衝突なし」で終わる。`dependencies` が YAML ライブラリ 1 つだけ、`dev` の依存が 8 章の許可の一覧の内。`constraints.txt` に、直接の依存(実行時と `dev`)のすべてが `==` の正確な版で書かれ、`pyproject.toml` に無い直接の依存の行がない。区分: 点検 |
| AC-479 | ビルドできる | ソース | `python -m build`(sdist と wheel)を、`PY_MIN` と `PY_MAX` で、Linux・Windows(`PY_MAX` は macOS も) | すべて成功する。`dist/` に sdist と wheel ができ、`scripts/check_dist.py` が通る。区分: 点検 |
| AC-480 | 型・整形・静的検査 | ソース | `mypy --strict`、`ruff check .`、`ruff format --check .` | すべて出力が「問題なし」(エラー・指摘・差分なし)。区分: 点検 |
| AC-481 | 既知の脆弱性の検査 | 依存 | `pip-audit`(`--skip-editable`) | 結果を記録する。脆弱性が見つかっても、マージはブロックしない(警告)。区分: 点検 |
| AC-482 | 変換した定義がそのまま使える | `CV1J` を `conductor.json` に | `convert conductor.json --to yaml > conductor.yaml`、`validate`、逆向きに `convert conductor.yaml --to json > c2.json`、`validate --config c2.json` | すべて 0。展開された木が同じ |
| AC-483 | `mypy` の厳格な設定と、`ruff` の規則 | `pyproject.toml` | 点検(`pyproject.toml` の `[tool.mypy]`・`[tool.ruff]` の文面の走査。および `mypy --strict` と `ruff check` の実行) | `[tool.mypy]` に `strict = true`、`warn_unreachable = true`、`disallow_any_explicit = true`(製品のコードの範囲)がある。`[tool.ruff.lint]` の `select` に、`E`・`F`・`W`・`I`・`N`・`UP`・`B`・`S`・`BLE`・`ANN`・`T20`・`TID`・`SIM`・`RUF`・`PT` を含む。`[tool.ruff.lint.flake8-tidy-imports.banned-api]` に、9.3 節の禁止の一覧がある。区分: 点検 |
| AC-484 | 合図とジョブの終了の順序と、シグナルのハンドラの復元 | `runner/` と `cli/` の単体(偽の `Deps` と、本物のシグナルの登録。UNIX 系) | 合図を、ジョブの終了の直前・直後・待機(リトライ)の直前に立てる。同じシグナルを続けて 2 回立てる。`run` / `pipeline` が終わったあとの `signal.getsignal(SIGHUP / SIGINT / SIGQUIT / SIGTERM)` を調べる | 最初の番号だけが残る。転送は 1 回。終了済みのジョブには送らない。`run` / `pipeline` の終了後(正常・失敗・例外のいずれでも)に、登録したシグナルのハンドラが、実行前のもの(`signal.getsignal` の値)に戻っている(ハンドラの漏れがない。復元は `finally` で行う)。区分: 単体 |
| AC-485 | 偽の `FakeSignals` で登録に失敗すると 70 | 偽の `FakeSignals`(登録が失敗を返す) | `run a`、`pipeline p`(定義は正しい / 誤りがある) | どちらも 70。ERR = `conductor: error: failed to set up signal handling`。定義の検査(定義が誤りでも 64 にならない)もジョブの起動もしない。区分: 注入。(AC-298 を、定義が誤りの場合まで広げたもの) |
| AC-486 | Windows の `.cmd` / `.bat` の禁止文字は起動しない | Windows。`T/echo-args.cmd`(内容 `@echo off` と `exit /b 0`)。`cmd` = そのパス + 引数 1 つ。引数に `"`、`%`、`^`、`&`、`\|`、`<`、`>`、`!`、`(`、`)`、改行のどれか 1 つを含める([07](07-process-and-signals.md)。1 文字ずつ別の定義)。配列形式でも同じ | `run a` | 127(起動失敗)。ERR = `conductor: error: job 'a': cannot start command '…'`。`T/injected.txt` ができない。含まない引数(`plain`)は起動される(AC-115)。区分: 自動(Windows の CI) |
| AC-487 | Windows の Ctrl+C のチェックリストの記録 | AC-301 の記録欄 | リリース前に、Windows の実機で AC-301 を行い、結果を `tests/WINDOWS_CTRL_C_CHECKLIST.md` の記録欄に書く | 記録欄に、実施日、OS の版、Python の版、(1)〜(4)の結果が書かれている。ない場合はリリースの前に実施する。区分: 手動(Windows) |

AC-475 の JSON の `ME`:

```json
{
  "format_version": 1,
  "unknown_top": true,
  "pipeline": {
    "name": "p"
  },
  "job": [
    {
      "id": "a",
      "command": "x",
      "success_exit_codes": ["zero"]
    },
    {
      "id": "b",
      "command": "y",
      "colour": "red",
      "transitions": [
        {
          "on": "success",
          "next": "nowhere"
        }
      ]
    }
  ]
}
```

JSON の定義の誤りの行は、自前の読み取りが、各キーと値の開始の位置(行・列)を記録して求める(LF の数で数える。CRLF でも同じ行番号。AC-038 と同様。[03](03-definition-format.md))。

### 4.20 Python に固有の項目

Python の実行モデル(インタプリタの起動と import、`signal` と `subprocess` の性質、標準入出力の符号化と flush、孤立サロゲート、`bool` が `int`、再帰の上限、`-O`、配布物の形)に固有の落とし穴を確かめる。注入のためのラッパー(`tests/support/inject_*.py`)は、`runpy.run_module("conductor", run_name="__main__")` で被試験体を動かす(3.6 節)。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-488 | 起動時間 | `__pycache__` が作られた状態(最初の 5 回を捨てる) | `python -m conductor --version` を 20 回(起動から終了まで)。コンソールスクリプトでも 20 回 | p95 が 150 ms(Linux・macOS)、300 ms(Windows)以下(仮。目安。2.1 節。コンソールスクリプトは、ランチャーの分の費用が加わるので、別に記録する)。区分: 計測 |
| AC-489 | import の時間と遅延 import | 同上 | `python -X importtime -m conductor --version`(標準エラーを読む) | 出力に、YAML ライブラリのモジュール(`yaml`)の行がない(`--version`・`--help`・使い方の誤り・`reset`・`watch` の経路で、YAML ライブラリを import しない)。`conductor` 配下のモジュールの累積が 60 ms 以下(仮。目安)。区分: 計測と点検(YAML ライブラリの不在は常に検査) |
| AC-490 | 未捕捉の例外は 70(UNIX 系) | `P(p; a:0{cmd="TJ sleep 1"})`。`python tests/support/inject_alarm.py run a` で起動する。`inject_alarm.py` は、`signal.signal(signal.SIGALRM, <RuntimeError('SECRET-INJECT') を投げるハンドラ>)` と `signal.setitimer(signal.ITIMER_REAL, 0.3)` を設定してから、`runpy.run_module("conductor", run_name="__main__")` を呼ぶ(製品に試験用の経路を持たせず、外から失敗を起こす) | `run a` | 終了 70。ERR = `conductor: error: internal error`(1 行だけ。末尾の改行 1 つ)。ERR に `SECRET-INJECT`・トレースバック(`Traceback`、`  File "`)が現れない。OUT 空。`__main__` の `except BaseException` が働く(ジョブの子プロセスが残る場合の扱いは 07 章に従う) |
| AC-491 | `sys.excepthook` 経由は 70。シグナルの旗を扱う部品の例外も 70 | (a) ドライバ `tests/support/inject_hook.py`: `sys.argv = ["conductor", "--version"]` にして `conductor.__main__.main()` を呼び(戻り値は捨てる。`SystemExit` は捕まえる)、そのあと `sys.excepthook(RuntimeError, RuntimeError("SECRET-INJECT"), None)` を直接呼ぶ。(b) 偽の `FakeSignals`(合図のコールバックが例外を投げる)と `run` / `pipeline`(注入) | (a) `python tests/support/inject_hook.py`。(b) 偽の部品の上で `run a` / `pipeline p` | (a) 終了 70。ERR = `conductor: error: internal error`(1 行だけ)。`SECRET-INJECT` が現れない。`sys.excepthook` は `main()` の中か `__main__` の import で設定されている(どちらでもよい)。(b) 70。ERR = `conductor: error: internal error`。ジョブのあとのステートの書き込みを、例外のために飛ばさない(例外が起きる前の確定したステートは残る)。区分: (a) 自動、(b) 注入 |
| AC-492 | シグナルの準備はメインスレッドの外では失敗し、70 になる | `runner/` のシグナルの準備の部品を、`threading.Thread` の中から呼ぶ(`signal.signal` がメインスレッドの外から `ValueError` を投げる。1 章) | 準備の部品の単体と、`cli/` の `run` を、スレッドの中から呼ぶ | 準備は「失敗」を返し(`ValueError` を漏らさない)、`run` は 70(ERR = `conductor: error: failed to set up signal handling`)。ジョブは起動されない。区分: 単体と注入 |
| AC-493 | 標準出力のパイプが閉じた(`BrokenPipeError`) | 入力 `big.json`(約 1 MiB。小さな要素の配列。`--to yaml` の出力が 64 KiB を大きく超える) | `convert big.json --to yaml` を起動し、標準出力の最初のデータを受けた時点で、読み手側(`proc.stdout.close()`)を閉じる | 終了 67。ERR に `conductor: error: cannot write to standard output`。`BrokenPipeError`・トレースバックが現れない。70 にならない(Python は、SIGPIPE を無視する設定で起動するので、`BrokenPipeError` として届く。確認は 1 章の範囲で、`SIGPIPE` の既定の扱いは未確認)。5 秒以内に終わる |
| AC-494 | 標準出力の取りこぼしがない | (a) `convert big.json --to yaml`(出力が 64 KiB を超える)。(b) `P(p; a:0{cmd="TJ big 16"})` | (a) 標準出力をパイプにし、読み手が最初の 500 ms 読まない。(b) `run a`(ジョブが 16 MiB を、`conductor` の標準出力に継承されたパイプへ書く) | (a) 終了 0。OUT の全体が、同じ変換を標準出力をファイルにして得た結果と、バイト単位で等しい(終了前に `flush` して書き込みの完了を待つ)。(b) 終了 0。OUT のバイト数がちょうど 16 MiB(`x` のみ)。`conductor` が出力を溜め込まない(子の標準入出力を `None` のままにして継承する) |
| AC-495 | 標準出力の書き込みの失敗(`/dev/full`。Linux) | Linux。標準出力を `/dev/full` を開いたファイルにして `watch`(ステートあり)、同様に `convert …`(`Popen(stdout=open("/dev/full", "wb"))`) | 実行 | `watch` は 65、`convert` は 67。ERR = `conductor: error: cannot write to standard output`。70 にならない。終了時の flush が再び失敗して、終了コードが 120 になったり `Exception ignored in` が出たりしない(AC-496)。区分: Linux |
| AC-496 | 終了時の flush の失敗で 120 にならない | UNIX 系。`python tests/support/inject_delay.py <引数>`(`inject_delay.py` は、`time.sleep(0.5)` のあとで `runpy.run_module("conductor", run_name="__main__")` を呼ぶ)。標準出力を `PIPE` にして、起動の直後に読み手を閉じる | `--version`、`--help`、`watch`(ステートあり)、`validate -v`(標準エラーが閉じた場合も) | 終了コードが 120 でない(`--version`・`--help` は 0(08 章 1.2 節に従う)、`watch` は 65)。ERR に `Exception ignored in` が現れない。1 章で、素の Python が、この条件で 120 になることを確認した(`sys.stdout.write` だけを行い、終了時の flush で失敗した場合)。区分: UNIX 系 |
| AC-497 | 終了後にプロセスが残らない | 各サブコマンド(`validate`、`run`、`pipeline`、`watch`、`reset`、`convert`、`--help`、`--version`、使い方の誤り) | 標準入力を開いたまま(`stdin=PIPE` で、閉じない)実行 | 出力の完了から 1 秒以内にプロセスが終わる(標準入力の読み取り、スレッド、`atexit` の登録、シグナルの旗の確認が残ってハングしない)。区分: 自動 |
| AC-498 | Python の警告が出ない | 4.1〜4.9 節の各種の実行(特に、YAML の誤り・`convert`・リトライの待機・ジョブの起動失敗) | `python -X dev -W error -m conductor …`(`CONDUCTOR_PYTHON` か既定の起動方法のとき。`CONDUCTOR_BIN` のときはスキップ)で実行。`-X dev` は `ResourceWarning` を有効にする | ERR に `Warning` を含む行(`DeprecationWarning`、`ResourceWarning`、`RuntimeWarning`、`SyntaxWarning`)が現れない。終了コードと出力が、通常の実行と同じ。`PY_MIN` と `PY_MAX` の両方で。区分: 自動 |
| AC-499 | `signal.signal` の点検 | ソース | 点検(`ast` の走査) | `signal.signal` の呼び出しが、`src/conductor/runner/` のシグナルの部品にしかない。登録するシグナルが、`platform.py` が返す一覧のシグナルだけ(UNIX 系は `SIGHUP`・`SIGINT`・`SIGQUIT`・`SIGTERM`、Windows は `SIGINT` と `SIGBREAK`。07 章)。登録のたびに、戻り値(元のハンドラ)を保存し、終了の処理(`finally`)で `signal.signal` に戻している。ハンドラの関数の本体が、旗の代入(`global` / `nonlocal` / 属性への代入)と `return` だけで、入出力・`print`・`subprocess`・ロックの取得・例外の送出を含まない。`signal.set_wakeup_fd`・`signal.alarm`・`signal.setitimer` を製品が使わない(使う場合は 07 章の決定に合わせて、許可の一覧に加える)。区分: 点検 |
| AC-500 | 終了の呼び出しの点検 | ソース | 点検(`ast` の走査) | `sys.exit`・`os._exit`・`raise SystemExit`・`exit(`・`quit(` の呼び出しが、`src/conductor/__main__.py` に 1 か所以下で、ほかのファイルにない。`__main__.py` の関数 `main` は、終了コードを**返す**(`int`)。`sys.exit(main())` は、`if __name__ == "__main__":` の中の 1 か所。標準出力・標準エラーの `flush` が、その前に済んでいる。`os._exit` を使うのは、`sys.excepthook` の中で、標準エラーへ書いて `flush` したあとだけ(使う場合)。区分: 点検 |
| AC-501 | 64 ビット符号なし整数の飽和演算 | `flow/` の単体 | 待ち時間を求める関数に、検査を通らない極端な値を直接渡す(`prev = 2**63`、`prev = 2**64 - 1`、`multiplier = 10`、`cap = 2**64 - 1`)。`max` = 100、`interval = 86400`、`multiplier = 10`、`max_interval = 86400` の列 | 結果が `int` で、`2**64 - 1` を超えず(`min(x, 2**64 - 1)` による飽和)、折り返し(`0` に戻る)がない(`2**64 - 1` のまま)。`& 0xFFFFFFFFFFFFFFFF` で切って折り返さない。`float` に直して比べない(`float` の精度で桁が落ちる)。列は 86400 のまま 100 回続く。待ちの秒数を、待ちの 1 区間の長さに直す関数は、AC-503 の範囲に収まる値だけを返す。区分: 単体 |
| AC-502 | 整数の検査: `bool`・浮動小数・`NaN`・巨大な整数 | 定義(YAML・JSON): `format_version: true`、`warning_exit_codes: [true]`、`[1.0]`、`[1e0]`、`max: true`、`max: 3.0`、JSON の `[NaN]`・`[Infinity]`、YAML の `[.nan]`・`[.inf]`、`[99999999999999999999]`、`max: 4294967296`、桁が 10,000 の整数(`format_version: 1000…0`。`int()` の文字数の上限(`sys.set_int_max_str_digits`。この文書の環境では未確認)で `ValueError` が起きる場合を含む)、YAML の `1e999`(`float()` が `inf`)、ステートの `exit_code: true`・`1.0`・`NaN`・`99999999999999999999` | `validate` / `run a` | すべて「範囲外・型の誤り」として拒否する(定義: 64、ステート: 65)。`True` を `1`、`1.0` を `1`、`inf` を整数として受理しない(`isinstance(x, bool)` を、`isinstance(x, int)` より先に検査する。`type(x) is int` で調べてもよい)。`ValueError`・`OverflowError` が漏れて 70 にならない。桁が多い入力で、ハングしない・異常終了しない。区分: 自動と単体 |
| AC-503 | 待ちの 1 区間の上限 | `runner/` の待ちの部品の単体(`time.sleep`・`Popen.wait` を、渡された時間を記録する関数に差し替える。合図で打ち切る) | 待機を 24 時間(86,400 秒)、`max_interval` の上限で呼ぶ。続けて、検査を通らない 30 日の待機を直接呼ぶ。さらに `2**40` 秒と、`float("inf")` に近い値 | 下の関数に渡される時間が、いつも 1 秒以下(07 章の区間の長さ以下)で、`threading.TIMEOUT_MAX` を超えない。`OverflowError`(`time.sleep` や `Popen.wait(timeout=…)` に大きすぎる値を渡したとき)が起きない。30 日の待機は、区間に分けて待つ(合図で打ち切って旗を検査する)。区分: 単体 |
| AC-504 | シグナルの番号と終了コードの対応 | `runner/` の単体 | `Popen.returncode`(`-9`、`-15`、`-2`、`-1`、`-3`、`-200`、`0`、`3`)を、「シグナルで終了」と終了コードに分ける関数 | 負の値 `-n` は、シグナル `n` で終了(番号 n)。0 以上は終了コード。`signal.Signals(n)` が `ValueError` になる番号(例: 200)でも、落ちない(名前なしで番号だけを使う)。`SIGHUP`=1、`SIGINT`=2、`SIGQUIT`=3、`SIGKILL`=9、`SIGTERM`=15(UNIX 系の共通の値。`SIGUSR1` など OS で違うものは、その OS の `signal.Signals` の値)。Windows の `Popen.returncode` は、符号なしの大きな値でも負の値でも、シグナルとして解釈しない(終了コードとして範囲合わせをする。07 章に従う。実機は未確認)。区分: 単体 |
| AC-505 | シグナルの送信で、ジョブの終わりを待たずに終わらない | `SL` の `sleeper` を `TJ --out O trap 500 3` にする(UNIX 系) | `run sleeper` を起動し、`ready` のあと SIGTERM | 終了 143。`O` に `trapped`、続いて `done` が記録されており、`conductor` が、ジョブの終了(`Popen.wait` / `poll` が終了を返すこと)を待ってから終わる(`send_signal` が戻った時点で終わらない)。`conductor` の終了は、SIGTERM の送信から 500 ms 以上あと。ステート `sleeper=error`(終了コード 3 は、中断の分類により `128 + 15` が採られる。ジョブの終了コード 3 は表示に使わない)。区分: UNIX 系 |
| AC-506 | ジョブが残した孫のプロセスを待たない | `argv=["sh", "-c", "sleep 5 & exit 3"]`(UNIX 系) | `run a`(標準出力・標準エラーを、パイプではなく一時ファイルに向けて起動する。パイプだと、孫が持ち続ける書き込み側のために、`communicate()` が EOF を 5 秒待つ。終了の時刻は `Popen.wait()` で測る) | 終了コード 3。`conductor` が、ジョブ(`sh`)の終了から 2 秒以内に終わる(孫の終了を待たない。子の標準入出力を `PIPE` にして `communicate()` で読むと、孫を待ってしまうので、`conductor` は、子の標準入出力を引き継ぐ。`None`)。区分: UNIX 系 |
| AC-507 | コンソールスクリプトと `python -m conductor` が同じ | wheel を入れた仮想環境 | `conductor` と `python -m conductor` で、`--version`、`--help`、`validate`(正しい定義・誤りの定義)、`run`、`watch`、使い方の誤りを実行 | 終了コード・OUT・ERR が同じ。`entry_points.txt` に `[console_scripts]` の `conductor = conductor.__main__:main` がある。コンソールスクリプトの終了コードは、`main()` の戻り値(`sys.exit(main())`)。区分: 自動(3 OS) |
| AC-508 | `pyproject.toml` の形 | `pyproject.toml` | 点検(文面の走査) | `[build-system]` の `requires`(ビルドバックエンドを、正確な版で固定)と `build-backend`。`[project]` に `name = "conductor"`、`version`、`description`、`readme`、`requires-python`、`license`(Apache-2.0)、`dependencies`(1 つ)、`[project.scripts]` の `conductor = "conductor.__main__:main"`、`[project.optional-dependencies]` の `dev`。`setup.py`・`setup.cfg`・`requirements.txt`(ビルドの設定として)がない。`[tool.pytest.ini_options]`・`[tool.ruff]`・`[tool.mypy]`・`[tool.coverage.*]` がある。区分: 点検 |
| AC-509 | 標準入力はジョブに引き継ぐ | `P(p; a:0{cmd="TJ --out O stdin"})` | `run a`(標準入力のパイプに `hello⏎` を書いて閉じる) | 終了 0。`O` = `stdin="hello\n"`。`conductor` が標準入力を読まない(`sys.stdin` を読み始めない。AC-497 と合わせて、読み始めると入力を奪うため。`Popen` の `stdin` は `None` のまま) |
| AC-510 | Windows の `.cmd` / `.bat` を、最小の Python でも最新の Python でも起動できる | Windows。`T/ok.cmd`(`exit /b 7`) | `PY_MIN` と `PY_MAX` で `run a`(`cmd` = そのパス + ` plain`) | どちらも 7(AC-115)。Python の `subprocess` は、Windows で `.cmd` / `.bat` を、`CreateProcess` 経由で暗黙の `cmd.exe` に渡す。引数の引用の扱いの版による違いと、その安全策は、07 章の決定に従い、この項目は、その方法が両方の Python で動くことを確かめる(実機で未確認)。区分: 自動(Windows の CI) |
| AC-511 | 辞書のキーの順序(整数に見えるキー) | YAML: `"2": a⏎"1": b⏎b: c`(キーは引用符つきの文字列)。JSON: `{"2": "a", "1": "b", "b": "c"}` | `convert in.yaml --to json` / `convert in.json --to yaml` | OUT のキーの順が、入力の記述順(`2`、`1`、`b`)。`sorted()`・`sort_keys=True`・`set` を経由して順序を失わない。区分: 自動 |
| AC-512 | 特別な名前のキーを、普通のキーとして扱う | JSON: `{"__proto__": 1, "__class__": 2, "__dict__": 3, "self": 4, "keys": 5, "items": 6, "constructor": 7}`(変換)。定義に `__class__: x`、`self: x`、`keys: x`(未知のキー) | `convert in.json --to yaml`、`validate` | 変換: 7 つのキーがそのまま出る(往復で木が等しい)。定義: `unknown key`(64)。`TypeError`(`dataclass` の `**kwargs` 展開で、キー名が仮引数と衝突する)や `AttributeError` で落ちない。区分: 自動と単体 |
| AC-513 | Windows の `.cmd` の起動引数の組み立ての単体(どの OS でも動く) | `runner/platform.py` の、起動引数を組み立てる関数(07 章。名前は 07 章に従う)に、Windows の分岐を表す引数を与える | `program` と `args` を与える: (a) `x.cmd` / `X.BAT`(拡張子の大文字小文字を区別しない)と引数 `plain`、(b) 禁止文字(`"`、`%`、`^`、`&`、`\|`、`<`、`>`、`!`、`(`、`)`、`\n`、`\r`、NUL)を 1 つ含む引数(1 文字ずつ)、(c) `program` が `C:\Program Files (x86)\t\x.cmd`(`(` と `)` は許す)、(d) 拡張子が `.exe` のもの、拡張子なしのもの、(e) 空の引数 | 07 章の決定のとおり。(a) `.cmd` の規則を使う(`shell=False` のまま。`shell=True` にならない)。(b) 拒否(起動しない)。(c) 受理。(d) `.cmd` の規則を使わず、そのまま。(e) 空の引数が失われない。UNIX 系の分岐は常に、そのまま(`start_new_session` の扱いは 07 章に従う)。区分: 単体(3 OS) |
| AC-514 | 依存の向きと循環の検査 | `src/`、`tests/inspect/test_imports.py` | 点検(`ast` で、各ファイルの `import` と `from … import` を集め、パッケージ(`definition`・`convert`・`runid`・`flow`・`state`・`runner`・`report`・`cli`)の依存の有向グラフを作り、`graphlib.TopologicalSorter` で循環を調べる) | 循環が 0 件(`graphlib.CycleError` が起きない)。02 章 3 章の許可されない組み合わせ(例: `definition/` から `flow/`・`runner/`・`state/`・`cli/`、`state/` と `runner/` の相互、`report/` から `cli/`)がない。ほかのパッケージからの import が、相手の `__init__.py` が公開した名前だけを通る(`conductor.flow.classify` のようにサブモジュールを直接読まず、`conductor.flow` から読む)。許可の表は、テストの中に書く。違反するコードを一時ファイルに置くと、テストが失敗する(テストの自己確認)。CI の `lint` ジョブで実行する。区分: 点検 |
| AC-515 | `--help` / `--version` の標準出力が閉じたパイプでも落ちない | `python -m conductor` | `conductor --help` と `conductor --version` を、標準出力を閉じた読み手(`proc.stdout.close()`、または `\| head -c 1`)で実行 | 終了 0(書き込みの失敗は無視する。08 章 1.2 節)。トレースバック・`Exception ignored in` が現れない。70 にも 120 にもならない。5 秒以内に終わる。区分: 自動(UNIX 系) |
| AC-516 | `watch` の標準出力が閉じたパイプ(`BrokenPipeError`) | 十分に大きな表になるステート(ジョブ 2,000 個)。UNIX 系 | `watch` を起動し、標準出力の最初のデータを受けた時点で、読み手側を閉じる | 終了 65。ERR に `conductor: error: cannot write to standard output`。`BrokenPipeError`・トレースバックが現れない。70 にも 120 にもならない。AC-493(`convert` は 67)と対にする。表が小さく、書き込みが閉じる前に終わるときは検査にならないので、表を大きくする(確実さは実装時に調整。10.1 節)。区分: 自動(UNIX 系) |
| AC-517 | シグナルの一覧と転送の単体(どの OS でも動く) | `runner/platform.py` の、UNIX 系と Windows の分岐を表す部品(07 章) | 受け取るシグナルの一覧、番号の対応、転送の関数(偽の子)、終了の読み取りを呼ぶ | UNIX 系: 一覧 = `SIGHUP`・`SIGINT`・`SIGQUIT`・`SIGTERM` の 4 つ、番号 1・2・3・15。転送は許可集合(1・2・3・15)だけ `send_signal` を 1 回呼ぶ。Windows: 一覧 = `SIGINT`・`SIGBREAK`(07 章に従う)。転送の規則は 07 章に従う。終了の読み取りは、`Popen.returncode` を、32 ビット符号つきに直して範囲合わせにかける。区分: 単体(3 OS) |
| AC-518 | 静的検査の規則が有効であることの確認 | `src/` の外の一時ディレクトリに、違反するコードのファイル(`subprocess.run(["x"], shell=True)`、`import socket`、`eval("1")`、`x: Any = 1`、`assert x`、`print("x")`)を置く(`ruff` の設定は、リポジトリの `pyproject.toml` を `--config` で指定し、`--force-exclude` と、`src/conductor/` 相当のパスを指定する。実装時に確認) | `ruff check <一時ファイル>` | 非 0 で終わり、それぞれの規則(`S602`・`TID251`・`S307`・`ANN401`・`S101`・`T201`)の指摘が出る(規則の番号は採用した版で確かめる)。`mypy --strict` が、型注釈のない関数を拒否する。製品のコードに `# type: ignore`・`# noqa` が 0 件。区分: 点検 |
| AC-519 | 孤立サロゲートの表示 | UNIX 系。ジョブID・`--run-id`・`--config` のパスに、不正なバイト(`b"\xff"`)を渡す(AC-406 と同じ方法)。YAML の `"\ud800"` を含む `command`・キー名(03 章が受理する場合) | `run <ID>`、`validate --config <パス>`、`watch`(ジョブID に孤立サロゲートを含むステートがある場合) | ERR・OUT の書き込みで `UnicodeEncodeError` が起きない。出力が、有効な UTF-8 として読める(`bytes.decode("utf-8")` が成功する)。孤立サロゲートは、08 章のエスケープ(例: `\u{dcff}`)で出る(生のバイト `0xff` を出さない)。区分: UNIX 系と単体 |
| AC-520 | 標準出力・標準エラーの符号化 | 日本語を含む出力を出す実行(`LANG=ja_JP.UTF-8` で `watch`・`validate`・`--help`) | 環境変数 `PYTHONIOENCODING=ascii`、`PYTHONUTF8=0`、`PYTHONUTF8=1`、`LANG=C`、`LC_ALL=C`、`PYTHONIOENCODING=latin-1` の各組み合わせで実行 | どの組み合わせでも、OUT・ERR のバイト列が同じ(UTF-8)。`UnicodeEncodeError` が起きない。起動時に `sys.stdout.reconfigure(encoding="utf-8", errors=<08 章の決定>)` と `sys.stderr.reconfigure(…)` を行う(08 章)。Windows のコンソールの表示(コードページ)は、別に手で確かめる(未確認)。区分: 自動 |
| AC-521 | 出力の順序(flush) | `P(p; a:0{cmd="TJ flaky F 1 75", retry=(exit_codes=[75], max=3, interval="1s")})`、標準出力と標準エラーを同じパイプ(`stderr=STDOUT`)にする | `run -v a` | 出力の行の順が、時間の順になる。`conductor` の `retry` の行が、2 回目のジョブの出力より前に現れる。`conductor` が自分の出力を、子を起動する前に `flush` している(標準エラーは行バッファでも、パイプでは、ブロックバッファのことがある。flush の漏れの確認)。区分: 自動 |
| AC-522 | NUL を含む `command` | `command: "a\u0000b"`、`argv=["a", "b\u0000c"]`(YAML の `\u0000` のエスケープ) | `validate` / `run a` | `ValueError: embedded null byte` が漏れて 70 にならない。定義の誤り(64)か、起動失敗(127、`cannot start command`)のどちらかとして扱う(03・07 章に規定があればそれに従う。未確認)。ステートが作られる場合は、`a=error ; resume=a`。区分: 自動と単体 |
| AC-523 | 空白の定義(`str.strip()` との差) | `command: "\u001c"`、`"\u0085"`、`"\u00a0"`、`"\u200b"`、`"\u3000"`、`name: "\u001c"`(別々の定義) | `validate` | 03 章の「空白だけ」の定義(Unicode の White_Space など)のとおり。Python の `str.strip()` / `str.isspace()` は、`\x1c`〜`\x1f` も空白として扱う(1 章で `"\x1c".isspace()` が真であることを確認)が、`\u200b` は扱わない。03 章が White_Space の集合を定めている場合は、`\x1c` を空白と見なさず(受理、または別の理由で拒否)、`\u200b` も空白と見なさない。実装は、03 章の集合を明示した表で判定する(`str.strip()` に頼らない)。区分: 単体 |
| AC-524 | 再帰の深さ | 深さ 1,000 / 100,000 の YAML・JSON(定義・ステート・変換)。`command` の分割(AC-410) | `validate` / `run a` / `convert` | `RecursionError` が漏れて 70 にならない(64 / 65 / 67)。`sys.setrecursionlimit` の呼び出しが製品のコードにない(点検)。`sys.getrecursionlimit()` を小さく設定した状態(テストが `sys.setrecursionlimit(200)` にして、単体を呼ぶ)でも、深さ 32 の入力が処理できる(再帰の深さが、入力の深さに比例して、上限に近づかない作りであること。32 段なら、読み取りの再帰の深さが 200 を超えない)。区分: 単体と自動 |
| AC-525 | YAML ライブラリの型判定に頼らない | YAML: AC-228 の値に加え、`a: 2001-12-14t21:59:43.10-05:00`(日時)、`b: 190:20:30`(60 進数)、`c: =`(値のタグ)、`d: <<`、`e: 0o17`、`f: +1`、`g: .5`、`h: 1.`、`i: -.inf`、`j: 0x_1F`、`k: 1e3` | `convert in.yaml --to json` | 日時・60 進数・`=` などは、文字列のまま(`datetime`・`date` が JSON の出力に漏れない。`TypeError: Object of type date is not JSON serializable` で落ちない)。数値は 03 章 4.2 節の規則(YAML 1.2 のコアスキーマ)に従う。ライブラリの `Loader`・`resolver` の暗黙の型判定(`yaml.SafeLoader` が `date`・`datetime`・`bool`(`yes`・`on`)・60 進数の整数にするもの)の結果を、そのまま使っていないこと。区分: 自動 |
| AC-526 | sdist と wheel の中身の検査 | ビルド済みの `dist/` | `python scripts/check_dist.py dist`(5.4 節) | wheel: 最上位が `conductor/` と `conductor-<版>.dist-info/` だけ。`conductor/` の中が `.py` だけ(`__pycache__`・テスト・テスト用ジョブ・`.so`・`.pyd` を含まない)。`conductor/__main__.py` と `conductor/version.py` がある。sdist: `pyproject.toml`・`LICENSE`・`README.md`・`PKG-INFO`・`src/conductor/` がある。`.github/`・`demo/`・`docs/`・`.venv/`・`constraints.txt` を含まない(含めるかどうかの決めは 6.2 節)。`__pycache__`・`*.pyc`・`.git*` を含まない。区分: 点検 |
| AC-527 | wheel のメタデータ | wheel の `*.dist-info/METADATA`・`WHEEL` | 点検(`zipfile` で読み、行を走査) | `Name: conductor`(配布名を置き換えた場合はその名前)、`Version` が `VERSION` と一致、`Requires-Python` が `pyproject.toml` の `requires-python` と一致、`Requires-Dist` が、`extra == "dev"` の条件つきのものを除いて 1 行だけ(YAML ライブラリ)、`Provides-Extra: dev`、`WHEEL` に `Root-Is-Purelib: true` と `Tag: py3-none-any`。`License` に Apache-2.0 の記載。区分: 点検 |
| AC-528 | 空の仮想環境に wheel を入れて動く | 配布された wheel | 空の仮想環境(`python -m venv`)に `pip install <wheel>`、`pip check`、`conductor --version`、`python -m conductor validate`(`B0`) | インストールが成功し(依存の YAML ライブラリを取得するため、ネットワークを使う)、`pip check` が衝突なし、`--version` の OUT = `conductor <版>`、`validate` が 0。作業ツリーの `src/` を `PYTHONPATH` に置かなくても動く。区分: 自動(3 OS) |
| AC-529 | sdist から入れて動く | 配布された sdist | 空の仮想環境に `pip install <sdist>`(ビルドの分離でバックエンドを取得する。ネットワークを使う)、`conductor --version` | 成功し、`--version` の OUT = `conductor <版>`。区分: 自動(Linux の CI) |
| AC-530 | `python -O` でも同じ | 4.1〜4.9 節の代表の実行(`validate`、`run`、`pipeline`、`watch`、`convert`、使い方の誤り) | `python -O -m conductor …` と `python -OO -m conductor …`(`CONDUCTOR_PYTHON` か既定の起動方法のとき) | 終了コード・OUT・ERR が、通常の実行と同じ(`assert` 文の除去と、docstring の除去に、挙動が依存しない。使い方の文言を docstring から作らない)。区分: 自動 |
| AC-531 | 危険な YAML のタグで任意のコードが実行されない | YAML: `command: !!python/object/apply:os.system ["touch pwned.txt"]`、`a: !!python/name:os.system`、`b: !!python/object/new:subprocess.Popen [["touch", "pwned2.txt"]]`(定義と変換の入力の両方) | `validate` / `convert in.yaml --to json` | 64 / 67(`syntax error`。理由 `タグは使えません`)。`T` に `pwned.txt`・`pwned2.txt` ができない。ライブラリの構築(`construct`)の段階に進まない(`yaml.load` の系列を呼ばない。AC-421)。区分: 自動 |


---

## 5. CI(GitHub Actions)と品質ゲート

ワークフローは `.github/workflows/` の 3 つ: `ci.yml`、`audit.yml`、`release.yml`。GitHub Actions の部品(`actions/checkout`、`actions/setup-python`、`actions/upload-artifact`、`actions/download-artifact`)の版は、`<版>` と書いた場所を、**実装時に、その時点の最新の安定版のメジャー版で確かめて決める**(この文書では確認していない)。
Python の版は、`PY_MIN`(最小。`pyproject.toml` の `requires-python` の最小と一致させる。5.4 節)と、`PY_MAX`(最新。公式にセキュリティ修正が続いている最新の版。分類子の最大と一致させる。5.4 節)で指定する。どちらも、ワークフローの `env` に書く(例: `PY_MIN: "3.11"`、`PY_MAX: "3.14"`。確認した値ではない)。
GitHub Actions では、`env` コンテキストを、ジョブの `strategy.matrix` の中では使えない(使えるコンテキストは `github`・`needs`・`vars`・`inputs` などで、`env` は含まれない。確認は文書の記憶による。実装時に確認)。そこで、最小と最新を行列に入れず、ジョブを分けるか、行列に `py: min` / `py: max` の印を入れて、**ステップの `with:` の中で** `${{ matrix.py == 'min' && env.PY_MIN || env.PY_MAX }}` と選ぶ。

### 5.1 `ci.yml`(名前 `CI`)

| 項目 | 内容 |
|------|------|
| 起動 | `push`(`main`)、`pull_request`(対象 `main`)、`workflow_call`(リリースから呼ばれる) |
| 権限 | `contents: read` |
| 同時実行 | `group: ci-${{ github.workflow }}-${{ github.ref }}`、`cancel-in-progress: true` |
| 環境変数 | `PY_MIN: "3.11"`(最小の Python。例。`pyproject.toml` の `requires-python` と一致させる)、`PY_MAX: "3.14"`(最新。例) |
| チェックアウト | `actions/checkout@<版>` |
| Python の用意 | `actions/setup-python@<版>`。`python-version` は `${{ env.PY_MAX }}`(最新)か `${{ env.PY_MIN }}`(最小)。`cache: pip`、`cache-dependency-path: constraints.txt`(pip のキャッシュ。実装時に確認) |
| 依存の入れ方 | `python -m pip install -c constraints.txt -e ".[dev]"`(制約ファイルのとおり。8.4 節)。Windows の `pip` の更新は、`python -m pip` の形で行う |

| ジョブ | 目的 | OS | Python | コマンド | 失敗の条件 |
|--------|------|----|--------|----------|------------|
| `lint` | 型・静的検査・整形・依存の向き | `ubuntu-24.04`(10 分) | 最新 | 依存を入れる → ① `ruff check .` ② `ruff format --check .` ③ `mypy --strict`(`pyproject.toml` の `files`)④ `mypy --strict --python-version "$PY_MIN"`(最小の版の意味での型の検査)⑤ `pytest tests/inspect`(依存の向きと循環、禁止の構文の点検。AC-514、AC-518、AC-420〜423) | `ruff` の指摘・差分が 1 つでも、`mypy` のエラーが 1 つでも、循環・許可されない import・禁止の構文がある(AC-480、AC-514、AC-518) |
| `consistency` | 依存と版の記載の整合 | `ubuntu-24.04`(5 分) | 最新 | ① `python scripts/check_version.py`(`version.py` と `pyproject.toml` の版の一致)② `python scripts/check_python_version.py`(5.4 節)③ `python scripts/check_deps.py`(8 章)④ 依存を入れる(制約ファイルと `pyproject.toml` の整合)⑤ `python -m pip check` | 版の不一致、依存の許可の違反、制約ファイルの食い違い、`pip check` の失敗(AC-476〜478) |
| `build-check` | 最小・最新の Python で、sdist と wheel がビルドでき、中身が正しく、入れて動くこと | `ubuntu-24.04`、`windows-2022`(`fail-fast: false`。15 分)。行列に `py: min`・`py: max` | 最小と最新 | `python -m pip install -c constraints.txt build` → `python -m build` → `python scripts/check_dist.py dist` → 空の仮想環境に wheel を入れて `conductor --version`・`python -m conductor --version`(5.1 節の骨組み) | ビルドの失敗、`--version` の失敗、配布物の中身の違反(AC-422、AC-440、AC-479、AC-526〜528) |
| `test` | 3 つの OS で、全テスト(単体・受け入れ・点検)が通ること | `ubuntu-24.04`、`macos-latest`、`windows-2022`(`fail-fast: false`。15 分) | 最新 | 依存を入れる → `pytest` | テストの失敗(30 秒の上限を含む) |
| `min-python-test` | 最小の Python でも、テストが通ること | `ubuntu-24.04`、`windows-2022`(`fail-fast: false`。15 分) | 最小 | 依存を入れる → `pytest` | テストの失敗。Windows で `.cmd` を最小の Python で起動できること(AC-510)を含む |
| `coverage` | 行カバレッジ 80% の維持 | `ubuntu-24.04`(20 分) | 最新 | 5.3 節のコマンド | カバレッジが 80% 未満 |
| `docs-check` | README に必要な語が残っていること | `ubuntu-24.04`(5 分) | 最新 | `python scripts/check_readme.py`(5.4 節。8 つの文字列) | 1 つでも欠けると `README.md is missing: <語>` を出して失敗 |
| `startup-bench` | 起動と性能の測定(2.1 節) | `ubuntu-24.04`、`macos-latest`、`windows-2022`(`fail-fast: false`。20 分) | 最新 | 依存を入れる → wheel をビルドして空の仮想環境に入れる → `CONDUCTOR_PERF=1 pytest tests/perf -s`(環境変数 `STARTUP_BENCH_ENFORCE: "false"`、`CONDUCTOR_PYTHON` に、その仮想環境の `python`)。測定値は `GITHUB_STEP_SUMMARY` に出る | `continue-on-error: true` のため、ジョブが失敗しても全体は失敗にならない。基準超えは警告のみ(強制するには環境変数を `"true"` にする) |

任意のジョブ(置くかどうかは、実装時に決める。ブランチ保護の必須にはしない): `next-python-test`(`actions/setup-python` の `allow-prereleases: true` で、次の版の Python の先行版に対して `pytest`。`continue-on-error: true`。オプション名は実装時に確認)、`build-repro`(`python -m build` を 2 回行い、wheel の SHA-256 が一致することを確かめる。環境変数 `SOURCE_DATE_EPOCH` を固定した場合に、ビルドバックエンドが同じ wheel を作るか、3 OS で同じになるかは未確認)。

`ci.yml` の骨組み(`<版>` は実装時に確認):

```yaml
name: CI
on:
  push: { branches: [main] }
  pull_request: { branches: [main] }
  workflow_call:
permissions:
  contents: read
concurrency:
  group: ci-${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
env:
  PY_MIN: "3.11"
  PY_MAX: "3.14"
jobs:
  lint:
    runs-on: ubuntu-24.04
    timeout-minutes: 10
    steps:
      - uses: actions/checkout@<版>
      - uses: actions/setup-python@<版>
        with: { python-version: "${{ env.PY_MAX }}", cache: pip, cache-dependency-path: constraints.txt }
      - run: python -m pip install -c constraints.txt -e ".[dev]"
      - run: ruff check .
      - run: ruff format --check .
      - run: mypy --strict
      - run: mypy --strict --python-version "${{ env.PY_MIN }}"
      - run: pytest tests/inspect
  consistency:
    runs-on: ubuntu-24.04
    timeout-minutes: 5
    steps:
      - uses: actions/checkout@<版>
      - uses: actions/setup-python@<版>
        with: { python-version: "${{ env.PY_MAX }}", cache: pip, cache-dependency-path: constraints.txt }
      - run: python scripts/check_version.py
      - run: python scripts/check_python_version.py
      - run: python scripts/check_deps.py
      - run: python -m pip install -c constraints.txt -e ".[dev]"
      - run: python -m pip check
  build-check:
    name: build-check (${{ matrix.os }}, ${{ matrix.py }})
    runs-on: ${{ matrix.os }}
    timeout-minutes: 15
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-24.04, windows-2022]
        py: [min, max]
    steps:
      - uses: actions/checkout@<版>
      - uses: actions/setup-python@<版>
        with:
          python-version: "${{ matrix.py == 'min' && env.PY_MIN || env.PY_MAX }}"
          cache: pip
          cache-dependency-path: constraints.txt
      - run: python -m pip install -c constraints.txt build
      - run: python -m build
      - run: python scripts/check_dist.py dist
      - name: install the wheel into an empty virtual environment
        shell: bash
        run: |
          python -m venv "$RUNNER_TEMP/venv"
          bin="$RUNNER_TEMP/venv/bin"; [ -d "$RUNNER_TEMP/venv/Scripts" ] && bin="$RUNNER_TEMP/venv/Scripts"
          "$bin/python" -m pip install dist/*.whl
          "$bin/python" -m pip check
          expected="conductor $(sed -n 's/^VERSION = "\(.*\)"$/\1/p' src/conductor/version.py)"
          out=$("$bin/python" -m conductor --version)
          [ "$out" = "$expected" ] || { echo "unexpected: $out"; exit 1; }
  test:
    name: test (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    timeout-minutes: 15
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-24.04, macos-latest, windows-2022]
    steps:
      - uses: actions/checkout@<版>
      - uses: actions/setup-python@<版>
        with: { python-version: "${{ env.PY_MAX }}", cache: pip, cache-dependency-path: constraints.txt }
      - run: python -m pip install -c constraints.txt -e ".[dev]"
      - run: pytest
  min-python-test:
    name: min-python-test (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    timeout-minutes: 15
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-24.04, windows-2022]
    steps:
      - uses: actions/checkout@<版>
      - uses: actions/setup-python@<版>
        with: { python-version: "${{ env.PY_MIN }}", cache: pip, cache-dependency-path: constraints.txt }
      - run: python -m pip install -c constraints.txt -e ".[dev]"
      - run: pytest
  coverage:
    runs-on: ubuntu-24.04
    timeout-minutes: 20
    steps:
      - uses: actions/checkout@<版>
      - uses: actions/setup-python@<版>
        with: { python-version: "${{ env.PY_MAX }}", cache: pip, cache-dependency-path: constraints.txt }
      - run: python -m pip install -c constraints.txt -e ".[dev]"
      - run: pytest --cov=conductor --cov-report=term-missing --cov-report=json
      - run: python scripts/check_coverage.py
  docs-check:
    runs-on: ubuntu-24.04
    timeout-minutes: 5
    steps:
      - uses: actions/checkout@<版>
      - uses: actions/setup-python@<版>
        with: { python-version: "${{ env.PY_MAX }}" }
      - run: python scripts/check_readme.py
  startup-bench:
    name: startup-bench (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    timeout-minutes: 20
    continue-on-error: true
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-24.04, macos-latest, windows-2022]
    env:
      STARTUP_BENCH_ENFORCE: "false"
      CONDUCTOR_PERF: "1"
    steps:
      - uses: actions/checkout@<版>
      - uses: actions/setup-python@<版>
        with: { python-version: "${{ env.PY_MAX }}", cache: pip, cache-dependency-path: constraints.txt }
      - run: python -m pip install -c constraints.txt -e ".[dev]"
      - run: python -m build --wheel
      - name: install the wheel into an empty virtual environment
        shell: bash
        run: |
          python -m venv "$RUNNER_TEMP/venv"
          bin="$RUNNER_TEMP/venv/bin"; [ -d "$RUNNER_TEMP/venv/Scripts" ] && bin="$RUNNER_TEMP/venv/Scripts"
          "$bin/python" -m pip install dist/*.whl
          echo "CONDUCTOR_PYTHON=$bin/python" >> "$GITHUB_ENV"
      - run: pytest tests/perf -s
```

(この文書の骨組みは、動作を確かめていない。版の取り出しの `sed` は、GNU 版と macOS 版のどちらでも通る書き方にしたが、確かめていない。`dist/*.whl` の `*` は、`shell: bash` のステップで展開される。Windows の `windows-2022` の `bash` は Git for Windows の `bash` で、`cygpath` の変換の要否は実装時に確認する。)

### 5.2 `audit.yml`(名前 `Audit`)

| 項目 | 内容 |
|------|------|
| 目的 | 依存の既知の脆弱性の検査(`pip-audit`)。結果は警告として出し、マージはブロックしない |
| 起動 | `pull_request`(対象 `main`)、`schedule`(cron `0 3 * * 1` = 毎週月曜 03:00 UTC) |
| 権限 | `contents: read` |
| ジョブ `audit`(名前 `pip-audit`) | `ubuntu-24.04`、10 分、**`continue-on-error: true`**。`actions/checkout@<版>` → `actions/setup-python@<版>`(`python-version: "3.x"` か `PY_MAX`)→ `python -m pip install -c constraints.txt -e ".[dev]"` → `pip-audit --skip-editable`(インストール済みの環境の全体。実行時の依存と開発時の依存の両方。オプション名と出力の書式は、採用した版の `pip-audit --help` で確かめる。この文書の作成時には実行していない) |
| 失敗の条件 | `pip-audit` が、脆弱性を報告すると、非 0 で終わり(終了コードの規則は版による。未確認)、ジョブは失敗するが、`continue-on-error` のため、ワークフロー全体は失敗にならない |

`pip-audit` は、脆弱性のデータベース(OSV など)へ問い合わせる(ネットワークを使う)。これは CI の検査であり、`conductor` の実行時の挙動ではない(AC-419 の対象は製品のコード)。実行時の依存だけを検査したい場合は、実行時の依存だけを入れた空の仮想環境で `pip-audit` を動かす(その仮想環境に `pip-audit` 自身が入るので、結果に `pip-audit` の依存が混じる点に注意。`--path` などで、別の環境を指定できるかは未確認)。

### 5.3 カバレッジ

| 項目 | 内容 |
|------|------|
| 計測 | `pytest --cov=conductor --cov-report=term-missing --cov-report=json`(`pytest-cov` が `coverage` を呼ぶ。`pyproject.toml` の `[tool.coverage.*]`。3.9 節)。`src/conductor/` の行の網羅率を測る。実行ファイルを `subprocess` で起動する受け入れテストの中のコードは、数えない |
| 除外 | `tests/`、`src/conductor/__main__.py`(入口)、`*/tests/*`。`source = ["conductor"]` にして、テスト用のコードが対象に入らないようにする |
| 基準 | 行の網羅率が **80.0% 未満なら失敗**。`pyproject.toml` の `fail_under = 80`(`coverage report` の失敗)と、`--cov-fail-under=80` の指定が、失敗(終了コード非 0)にし、加えて `scripts/check_coverage.py` が、`coverage.json` の `totals.percent_covered` を独立に検査する(設定の書式が版で変わるため。`coverage.json` のキー名は、実装時に確認) |
| 実行する OS | `ubuntu-24.04` のみ。したがって、Windows 専用のコード(`platform.py` の Windows の分岐)は、計測の対象に入っても、通らない行になる。この分は 80% の内で吸収できる大きさに留める(OS 別の分岐を、薄く 1 か所に集める。2.4 節) |
| 範囲 | 単体テストと、`Deps` を差し替えた流れのテストの全体。分岐の網羅(`branch = true`)は、下限に含めない(行だけ) |

```sh
pytest --cov=conductor --cov-report=term-missing --cov-report=json
python scripts/check_coverage.py
```

### 5.4 検査スクリプトの全文

すべて、標準ライブラリだけの Python のスクリプト(`from __future__ import annotations` を付け、`mypy --strict` を通す。3 OS で動く)。**`pyproject.toml` は、文面の走査(正規表現)で読む**(`[project]` などの表の見出しを追う最小の読み取り。複数行の値は、配列だけ扱う。標準ライブラリに、`pyproject.toml` の読み取りがあるかは版による(`PY_MIN` で使えるとは限らない)ので、使わない)。各ファイルの先頭に、Apache License 2.0 のヘッダ(AC-423)を置く(ここでは省略)。
この文書の作成時に、スクリプトは、小さな見本のリポジトリで、正常な場合と、版・制約・`PY_MAX` が食い違う場合の動作を、CPython 3.9.6 で確かめた(`check_dist.py` は、`zipfile`・`tarfile` で作った見本の wheel と sdist で、正常な場合だけ確かめた。本物のビルドバックエンドの出力では未確認)。

`scripts/check_version.py` — 版の一致(AC-476)。引数にタグの版(`v` なし)があれば、それとも比べる。

```python
"""src/conductor/version.py と pyproject.toml の版の一致を検査する(AC-476)。

引数にタグの版(v なし)があれば、それとも比べる。
"""

from __future__ import annotations

import re
import sys
from pathlib import Path


def project_field(text: str, key: str) -> str | None:
    """[project] 表の、`key = "値"` の 1 行の値を返す(文面の走査。複数行の値は扱わない)。"""
    section = ""
    for line in text.splitlines():
        header = re.match(r"^\[([^\]]+)\]\s*$", line)
        if header:
            section = header.group(1)
            continue
        if section == "project":
            m = re.match(rf'^{re.escape(key)}\s*=\s*"([^"]*)"\s*(#.*)?$', line)
            if m:
                return m.group(1)
    return None


def main(argv: list[str]) -> int:
    py = Path("pyproject.toml").read_text(encoding="utf-8")
    src = Path("src/conductor/version.py").read_text(encoding="utf-8")
    m = re.search(r'^VERSION = "([^"]+)"$', src, re.MULTILINE)
    if m is None:
        print('src/conductor/version.py: VERSION = "..." not found', file=sys.stderr)
        return 1
    versions = {
        "src/conductor/version.py": m.group(1),
        "pyproject.toml": project_field(py, "version"),
    }
    expected = argv[0] if argv else versions["pyproject.toml"]
    failed = False
    for where, v in versions.items():
        if v != expected:
            print(f"{where}: {v} != {expected}", file=sys.stderr)
            failed = True
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
```

`scripts/check_python_version.py` — Python の最小版・最新版の記載の一致(AC-477)。

```python
"""Python の最小版・最新版の記載の一致を検査する(AC-477)。

pyproject.toml の requires-python と ruff の target-version、README の「Python <最小版> 以降」、
ci.yml の PY_MIN、PY_MAX と分類子の最大が一致すること。
"""

from __future__ import annotations

import re
import sys
from pathlib import Path


def main() -> int:
    py = Path("pyproject.toml").read_text(encoding="utf-8")
    readme = Path("README.md").read_text(encoding="utf-8")
    ci = Path(".github/workflows/ci.yml").read_text(encoding="utf-8")
    failed = False

    m = re.search(r'^requires-python\s*=\s*">=\s*(3\.\d+)(?:\.\d+)?"\s*$', py, re.MULTILINE)
    if m is None:
        print('pyproject.toml: requires-python must look like ">=3.11"', file=sys.stderr)
        return 1
    minimum = m.group(1)

    t = re.search(r'^target-version\s*=\s*"py(\d)(\d+)"\s*$', py, re.MULTILINE)
    if t is None or f"{t.group(1)}.{t.group(2)}" != minimum:
        found = f"py{t.group(1)}{t.group(2)}" if t else "missing"
        print(f"pyproject.toml: ruff target-version ({found}) != requires-python ({minimum})", file=sys.stderr)
        failed = True

    if f"Python {minimum} 以降" not in readme:
        print(f"README.md: 'Python {minimum} 以降' not found", file=sys.stderr)
        failed = True

    c = re.search(r'^ {2}PY_MIN:\s*"?(3\.\d+)"?\s*$', ci, re.MULTILINE)
    if c is None or c.group(1) != minimum:
        print(f"ci.yml PY_MIN ({c.group(1) if c else 'missing'}) != requires-python ({minimum})", file=sys.stderr)
        failed = True

    classifiers = [int(x) for x in re.findall(r"Programming Language :: Python :: 3\.(\d+)\b", py)]
    d = re.search(r'^ {2}PY_MAX:\s*"?(3\.\d+)"?\s*$', ci, re.MULTILINE)
    if not classifiers or d is None or d.group(1) != f"3.{max(classifiers)}":
        newest = f"3.{max(classifiers)}" if classifiers else "missing"
        print(f"ci.yml PY_MAX ({d.group(1) if d else 'missing'}) != newest classifier ({newest})", file=sys.stderr)
        failed = True
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
```

`scripts/check_deps.py` — 依存の許可の一覧と制約ファイルの整合(8 章。AC-478)。

```python
"""依存の許可の一覧と、制約ファイルの整合を検査する(8 章、AC-478)。"""

from __future__ import annotations

import re
import sys
from pathlib import Path

RUNTIME_ALLOWED = {"pyyaml"}  # 8.2 節で採った YAML ライブラリの名前に合わせる
DEV_ALLOWED = {"pytest", "pytest-cov", "mypy", "ruff", "build", "pip-audit"}


def normalize(name: str) -> str:
    return re.sub(r"[-_.]+", "-", name).lower()


def requirement_name(spec: str) -> str:
    m = re.match(r"^\s*([A-Za-z0-9][A-Za-z0-9._-]*)", spec)
    return normalize(m.group(1)) if m else ""


def string_array(text: str, section: str, key: str) -> list[str]:
    """`[section]` 表の `key = [ "…", … ]`(複数行可)の文字列の並びを返す。"""
    current = ""
    items: list[str] = []
    collecting = False
    for line in text.splitlines():
        header = re.match(r"^\[([^\]]+)\]\s*$", line)
        if header and not collecting:
            current = header.group(1)
            continue
        if current != section:
            continue
        if not collecting and re.match(rf"^{re.escape(key)}\s*=\s*\[", line):
            collecting = True
            line = line.split("[", 1)[1]
        if collecting:
            body = line.split("]", 1)[0]
            items += re.findall(r'"([^"]*)"', body)
            if "]" in line:
                return items
    return items


def main() -> int:
    py = Path("pyproject.toml").read_text(encoding="utf-8")
    failed = False

    runtime = [requirement_name(s) for s in string_array(py, "project", "dependencies")]
    dev = [requirement_name(s) for s in string_array(py, "project.optional-dependencies", "dev")]
    if len(runtime) != 1 or runtime[0] not in RUNTIME_ALLOWED:
        print(f"dependencies must be exactly one allowed package: {runtime}", file=sys.stderr)
        failed = True
    for name in dev:
        if name not in DEV_ALLOWED:
            print(f"dev dependency not allowed: {name}", file=sys.stderr)
            failed = True

    pins: dict[str, str] = {}
    for raw in Path("constraints.txt").read_text(encoding="utf-8").splitlines():
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        m = re.match(r"^([A-Za-z0-9][A-Za-z0-9._-]*)==([^\s;]+)$", line)
        if m is None:
            print(f"constraints.txt: not an exact pin: {line}", file=sys.stderr)
            failed = True
            continue
        pins[normalize(m.group(1))] = m.group(2)

    direct = set(runtime) | set(dev)
    for name in sorted(direct - set(pins)):
        print(f"constraints.txt: missing pin for {name}", file=sys.stderr)
        failed = True
    for name in sorted(set(pins) - direct):
        print(f"constraints.txt: {name} is not a direct dependency", file=sys.stderr)
        failed = True

    backend = string_array(py, "build-system", "requires")
    if not any(re.match(r"^hatchling==[^\s;]+$", s) for s in backend):
        print('pyproject.toml: build-system requires must pin hatchling with "=="', file=sys.stderr)
        failed = True
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
```

`scripts/check_readme.py` — README の必須の語(`docs-check`)。

```python
"""README に必要な語が残っていることを検査する(docs-check)。"""

from __future__ import annotations

import sys
from pathlib import Path

WORDS = [
    "SHA-256",
    "終了コードの表",
    "ステートファイルの場所",
    "at-least-once",
    "シグナル",
    "convert",
    "Python ",
    "pip install",
]


def main() -> int:
    readme = Path("README.md").read_text(encoding="utf-8")
    missing = [w for w in WORDS if w not in readme]
    for w in missing:
        print(f"README.md is missing: {w}", file=sys.stderr)
    return 1 if missing else 0


if __name__ == "__main__":
    sys.exit(main())
```

`scripts/check_coverage.py` — カバレッジの独立の検査(5.3 節)。

```python
"""カバレッジの独立の検査(5.3 節)。`coverage.json`(`--cov-report=json`)の行の網羅率が 80% 以上。"""

from __future__ import annotations

import json
import sys
from pathlib import Path


def main() -> int:
    summary = json.loads(Path("coverage.json").read_text(encoding="utf-8"))
    pct = summary["totals"]["percent_covered"]
    print(f"total line coverage: {pct}%")
    if not isinstance(pct, (int, float)) or pct < 80:
        print("line coverage must be at least 80%", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
```

`scripts/check_dist.py` — sdist と wheel の中身の検査(AC-440、AC-526、AC-527)。

```python
"""sdist と wheel の中身を検査する(AC-440、AC-526、AC-527)。使い方: python scripts/check_dist.py dist"""

from __future__ import annotations

import re
import sys
import tarfile
import zipfile
from pathlib import Path

DIST_NAME = "conductor"  # 配布名を置き換えた場合は合わせる
SDIST_REQUIRED = ["PKG-INFO", "pyproject.toml", "LICENSE", "README.md", "src/conductor/__main__.py"]
SDIST_FORBIDDEN_PREFIXES = [".github/", "demo/", "docs/", ".venv/", "scripts/", "constraints.txt"]


def project_field(text: str, key: str) -> str | None:
    section = ""
    for line in text.splitlines():
        header = re.match(r"^\[([^\]]+)\]\s*$", line)
        if header:
            section = header.group(1)
            continue
        if section == "project":
            m = re.match(rf'^{re.escape(key)}\s*=\s*"([^"]*)"\s*(#.*)?$', line)
            if m:
                return m.group(1)
    return None


def check_wheel(path: Path, version: str, requires_python: str | None) -> list[str]:
    errors: list[str] = []
    with zipfile.ZipFile(path) as zf:
        names = zf.namelist()
        dist_info = f"{DIST_NAME}-{version}.dist-info"
        for n in names:
            top = n.split("/", 1)[0]
            if top not in (DIST_NAME, dist_info):
                errors.append(f"wheel: unexpected top-level entry: {n}")
            if top == DIST_NAME and not n.endswith("/") and not n.endswith(".py"):
                errors.append(f"wheel: non-.py file in the package: {n}")
            if "__pycache__" in n or n.endswith(".pyc"):
                errors.append(f"wheel: bytecode file: {n}")
        for required in (f"{DIST_NAME}/__main__.py", f"{DIST_NAME}/version.py"):
            if required not in names:
                errors.append(f"wheel: missing {required}")
        meta = zf.read(f"{dist_info}/METADATA").decode("utf-8") if f"{dist_info}/METADATA" in names else ""
        wheel = zf.read(f"{dist_info}/WHEEL").decode("utf-8") if f"{dist_info}/WHEEL" in names else ""
    if not meta:
        errors.append("wheel: METADATA is missing")
    if f"Name: {DIST_NAME}" not in meta.splitlines():
        errors.append(f"wheel: Name is not {DIST_NAME}")
    if f"Version: {version}" not in meta.splitlines():
        errors.append(f"wheel: Version is not {version}")
    if requires_python is not None and f"Requires-Python: {requires_python}" not in meta.splitlines():
        errors.append(f"wheel: Requires-Python is not {requires_python}")
    runtime = [
        line for line in meta.splitlines() if line.startswith("Requires-Dist:") and "extra ==" not in line
    ]
    if len(runtime) != 1:
        errors.append(f"wheel: expected exactly one runtime Requires-Dist, got {runtime}")
    if "Root-Is-Purelib: true" not in wheel.splitlines():
        errors.append("wheel: Root-Is-Purelib is not true")
    if "Tag: py3-none-any" not in wheel.splitlines():
        errors.append("wheel: Tag is not py3-none-any")
    return errors


def check_sdist(path: Path, version: str) -> list[str]:
    errors: list[str] = []
    root = f"{DIST_NAME}-{version}/"
    with tarfile.open(path, "r:gz") as tf:
        names = tf.getnames()
    rels: list[str] = []
    for n in names:
        if n == root.rstrip("/"):
            continue
        if not n.startswith(root):
            errors.append(f"sdist: entry outside {root}: {n}")
            continue
        rels.append(n[len(root):])
    for required in SDIST_REQUIRED:
        if required not in rels:
            errors.append(f"sdist: missing {required}")
    for rel in rels:
        if any(rel.startswith(p) for p in SDIST_FORBIDDEN_PREFIXES):
            errors.append(f"sdist: forbidden entry: {rel}")
        if "__pycache__" in rel or rel.endswith(".pyc"):
            errors.append(f"sdist: bytecode file: {rel}")
    return errors


def main(argv: list[str]) -> int:
    dist = Path(argv[0] if argv else "dist")
    pyproject = Path("pyproject.toml").read_text(encoding="utf-8")
    version = project_field(pyproject, "version")
    if version is None:
        print("pyproject.toml: version not found", file=sys.stderr)
        return 1
    wheels = sorted(dist.glob("*.whl"))
    sdists = sorted(dist.glob("*.tar.gz"))
    errors: list[str] = []
    if len(wheels) != 1:
        errors.append(f"expected exactly one wheel in {dist}, got {len(wheels)}")
    if len(sdists) != 1:
        errors.append(f"expected exactly one sdist in {dist}, got {len(sdists)}")
    if not errors:
        errors += check_wheel(wheels[0], version, project_field(pyproject, "requires-python"))
        errors += check_sdist(sdists[0], version)
    for e in errors:
        print(e, file=sys.stderr)
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
```

最小の Python を上げるときは、`pyproject.toml` の `requires-python`・`[tool.ruff]` の `target-version`・分類子、README の `Python <版> 以降`、`ci.yml` の `PY_MIN` を、同じコミットで更新する(`PY_MAX` を上げるときは、分類子と `PY_MAX`)。

### 5.5 品質ゲートの再現手順(ローカル)

マージの前に、次をすべて通す。

```sh
# 仮想環境と依存(制約ファイルのとおり)
python -m venv .venv && . .venv/bin/activate        # Windows は .venv\Scripts\activate
python -m pip install -c constraints.txt -e ".[dev]"

# 静的検査・整形・型
ruff check .
ruff format --check .
mypy --strict
mypy --strict --python-version 3.11                  # 最小の版(PY_MIN)の意味での型の検査

# 整合
python scripts/check_version.py
python scripts/check_python_version.py
python scripts/check_deps.py
python scripts/check_readme.py
python -m pip check

# ビルドと配布物の形
python -m pip install -c constraints.txt build
python -m build                                      # sdist と wheel が dist/ にできる
python scripts/check_dist.py dist

# テスト
pytest                                               # 単体 + 受け入れ + 点検

# カバレッジ(Linux)
pytest --cov=conductor --cov-report=term-missing --cov-report=json && python scripts/check_coverage.py

# 既知の脆弱性(警告のみ。マージはブロックしない)
pip-audit --skip-editable

# 起動時間(目安)
python -X importtime -m conductor --version 2>&1 | tail -n 20
CONDUCTOR_PERF=1 pytest tests/perf -s
```

加えて、CI で確かめる: Windows(`windows-2022`)と macOS(`macos-latest`)での `pytest`、最小の Python でのビルドとテスト(Linux・Windows)、README の必須の語、Python の版の記載の一致、起動の性能の測定(警告のみ)。
リリース前に、Windows の実機で Ctrl+C のチェックリスト(AC-301)を確かめ、結果を記録する。

ブランチ保護(どのジョブを必須にするか)の設定は、リポジトリの外にあり、未確認。想定は、`lint`・`consistency`・`build-check`・`test`・`min-python-test`・`coverage`・`docs-check` を必須にする。

---


## 6. リリース

### 6.1 配布の形

`conductor` は、Python のパッケージ(sdist と wheel)として配る。利用者は、Python(`requires-python` の最小版以上)を用意し、`pip install` で入れる。実行時の依存は YAML ライブラリ 1 つだけ(8 章)。OS ごとの実行ファイルは作らない(Python があれば、3 OS で同じ wheel が動く。wheel は、純 Python の `py3-none-any`)。

| 項目 | 内容 |
|------|------|
| 成果物 | `conductor-<版>.tar.gz`(sdist)、`conductor-<版>-py3-none-any.whl`(wheel)、`SHA256SUMS`(`<版>` は `v` なし。例: `0.8.0`。配布名を置き換えた場合は、その名前。ファイル名の正規化の規則は、ビルドの道具による) |
| 中身 | wheel は、`conductor/`(`.py` だけ)と `*.dist-info/` だけ。sdist は、`src/`、`tests/`、`pyproject.toml`、`LICENSE`、`README.md`、`PKG-INFO` を含む。`.github/`、`demo/`、`docs/`、`scripts/`、`constraints.txt`、仮想環境を含まない(`pyproject.toml` の `[tool.hatch.build.targets.*]` で絞る。AC-422、AC-440、AC-526) |
| 対応 OS | Linux、macOS、Windows(Python が動く環境) |
| チェックサム | 成果物の SHA-256 を、`sha256sum` の形式で `SHA256SUMS` に書き、一緒に配る。署名はしない(転送の破損の検知まで。配布元の改ざんは防げない) |
| リリースの注記 | 自動生成に、固定の 1 文を加える(`この成果物は Python のパッケージです。PyPI への公開は人が手で行います。SHA256SUMS で、ダウンロードしたファイルを確かめてください。`) |
| 公開 | GitHub のリリースは、**下書き(draft)として作り**、人が確認して公開する(自動では公開しない)。版が先行版(PEP 440 の `a`・`b`・`rc`・`.dev`。例: `0.9.0rc1`)なら、プレリリースにする |
| PyPI への公開 | **人が手で行う**(例: `python -m twine upload dist/*`。`twine` は、リポジトリの依存には入れず、人の手元の道具として使う)。ワークフローは、PyPI のトークンを持たず、公開をしない。公開の手順(二要素認証、API トークン、信頼できる発行者の設定、配布名の取得)の細部は、PyPI の運用による(未確認) |

### 6.2 ビルドの方法(決め)

```sh
python -m pip install -c constraints.txt build
python -m build        # sdist と wheel を dist/ に作る
python scripts/check_dist.py dist
```

| 項目 | 決め | 理由 |
|------|------|------|
| ビルドの道具 | PyPA の `build`(`python -m build`。分離した環境でビルドバックエンドを取得して動かす) | 標準的な入口。開発時の依存の許可の一覧に入れる(8.3 節) |
| ビルドバックエンド | `hatchling`(`[build-system]` の `requires` に、**正確な版**で固定する。`hatchling==<実装時に確認>`)。候補: `hatchling`、`setuptools`(+ `wheel`)、`flit-core`、`pdm-backend`。採る理由: `src/` 配置を設定が短く扱える、`setup.py` を要さない、sdist に含めるファイルを `pyproject.toml` で絞れる、純 Python のパッケージに十分。`setuptools` でも、同じ成果物を作れる(置き換えてよい。その場合は、`check_dist.py` の期待を実際の出力に合わせる)。この文書の作成時には、どのバックエンドも動かして確かめていない | 実行時の依存を増やさず、ビルドの設定を `pyproject.toml` に集める |
| パッケージの配置 | `src/conductor/`(wheel の `packages = ["src/conductor"]`)。wheel に入るのは、`.py` だけ | `src/` 配置は、作業ツリーの `conductor/` を誤って import することを防ぐ |
| 版 | `pyproject.toml` の `[project]` の `version` に、静的に書く(動的な版(`dynamic = ["version"]`)にしない)。`src/conductor/version.py` の `VERSION` と一致することを、ビルドの前に検査する(AC-476、6.3 節)。版を、`importlib.metadata` で実行時に引かない(インストールされていない作業ツリーでも `--version` が動くため) | 版の定義が 2 か所あるが、一致を機械で検査する |
| コンソールスクリプト | `[project.scripts]` の `conductor = "conductor.__main__:main"`。`main` は終了コード(`int`)を返し、インストーラが作るスクリプトが `sys.exit(main())` を行う(AC-507)。`python -m conductor` では、`__main__.py` の末尾で `sys.exit(main())` を行う | 2 つの起動方法で、終了コードを同じにする |
| ソースの先頭の行 | `.py` に、シバン(`#!`)を付けない(インストーラがランチャーを作る)。`# Copyright 2026 agwlvssainokuni` のヘッダ(AC-423) | — |
| 型の印 | `py.typed` を置かない(ライブラリとして型情報を公開しない) | `conductor` は CLI で、import して使うものではない |
| バイトコード | wheel に `__pycache__` を入れない。インストール時に、`pip` が必要に応じて作る | 内容の再現性。AC-526 |
| ビルドの再現 | 環境変数 `SOURCE_DATE_EPOCH` を固定したときに、同じ wheel が作れるかは未確認(`build-repro`。5.1 節) | — |
| 使う Python | ビルドとリリースは、最新の版(`PY_MAX`)。利用者の下限は `requires-python` | 最新の道具の修正を取り込むため。`build-check` が、最小の版でもビルドを確かめる |

### 6.3 リリースの手順と `release.yml`

人の操作:

1. `src/conductor/version.py` の `VERSION` と、`pyproject.toml` の `version` を、新しい版に上げる(README の例も)。版は、PEP 440 の**正規化された書き方**で書く(例: `0.9.0rc1`。`0.9.0-rc.1` と書かない。タグの版と、**文字列として完全に一致**させるため)。`main` に取り込む。
2. `main` のそのコミットに、タグ `v<版>`(例: `v0.8.0`)を付けて push する。タグのコミットが `main` の祖先であること。タグの `v` を除いた文字列が、`VERSION` と `pyproject.toml` の `version` と**完全に一致**すること。
3. ワークフローが、下書き(draft)のリリースを作る。人が内容を確認して、公開する(自動では公開しない)。
4. 人が、リリース前に Windows の実機で Ctrl+C のチェックリスト(AC-301、AC-487)を確かめる。
5. 人が、PyPI に公開する(任意。ワークフローは行わない)。

起動: `push` の `tags: ["v*"]`。権限は、ジョブごとに必要なものだけ(`draft-release` だけ `contents: write`)。

| ジョブ | 依存 | OS | 内容 | 失敗の条件 |
|--------|------|----|------|------------|
| `verify-tag` | なし | `ubuntu-24.04`(5 分) | `actions/checkout@<版>`(`fetch-depth: 0`)、`actions/setup-python@<版>`(`PY_MAX`)。bash: タグ名 `${GITHUB_REF_NAME}` から `v` を除いて `version`。① `git merge-base --is-ancestor "$GITHUB_SHA" origin/main`(タグのコミットが `main` の祖先)② `python scripts/check_version.py "$version"`(`version.py` と `pyproject.toml` の版が `version` と等しい)。出力: `version`、`prerelease`(`version` が先行版の印を持てば `true`) | タグが `main` 上にない(`tag is not on main`)、または版の不一致(`<場所>: <版> != <タグの版>`) |
| `checks` | `verify-tag` | — | `uses: ./.github/workflows/ci.yml`(5.1 節の全ジョブを再利用。マージの関門と同じ検査) | `startup-bench` は `continue-on-error` のため影響しない。それ以外のジョブの失敗で、以降が止まる |
| `build` | `verify-tag`、`checks` | `ubuntu-24.04`(15 分) | `actions/checkout@<版>`、`actions/setup-python@<版>`(`PY_MAX`、`cache: pip`)、`python -m pip install -c constraints.txt build`、`python -m build`、`python scripts/check_dist.py dist`、`actions/upload-artifact@<版>`(名前 `dist`、パス `dist/*`、`if-no-files-found: error`) | ビルド・検査・アップロードのどれかの失敗 |
| `accept-dist` | `verify-tag`、`build` | `ubuntu-24.04`、`macos-latest`、`windows-2022`(`fail-fast: true`。20 分) | 下記の手順(wheel を空の仮想環境に入れて、版の表示と受け入れテストを行う) | `--version` の不一致、受け入れテストの失敗(AC-442、AC-446、AC-528) |
| `draft-release` | `verify-tag`、`accept-dist` | `ubuntu-24.04`(10 分。`permissions: contents: write`) | `actions/download-artifact@<版>`(名前 `dist`、`path: dist-release`)。`cd dist-release && sha256sum * > SHA256SUMS`(`SHA256SUMS` 自身を含めないよう、先に `SHA256SUMS` を作ってからファイル一覧を取る順に注意する)。`gh release create "$TAG" dist-release/* --repo "$GITHUB_REPOSITORY" --title "$TAG" --draft --generate-notes [--prerelease] --notes "<注記>"`(トークンは `github.token`) | `gh` の失敗 |

`verify-tag` の検査(bash):

```sh
version="${GITHUB_REF_NAME#v}"
git merge-base --is-ancestor "$GITHUB_SHA" origin/main || { echo "tag is not on main"; exit 1; }
python scripts/check_version.py "$version" || exit 1
case "$version" in
  *a[0-9]*|*b[0-9]*|*rc[0-9]*|*.dev[0-9]*) echo "prerelease=true" >> "$GITHUB_OUTPUT" ;;
  *) echo "prerelease=false" >> "$GITHUB_OUTPUT" ;;
esac
echo "version=$version" >> "$GITHUB_OUTPUT"
```

`accept-dist` の手順(3 OS。bash(`shell: bash`)で書く。`${{ runner.temp }}` を使う):

```yaml
- uses: actions/checkout@<版>
- uses: actions/setup-python@<版>
  with: { python-version: "${{ env.PY_MAX }}", cache: pip, cache-dependency-path: constraints.txt }
- run: python -m pip install -c constraints.txt -e ".[dev]"
- uses: actions/download-artifact@<版>
  with: { name: dist, path: "${{ runner.temp }}/dist" }
- name: install the wheel into an empty virtual environment
  shell: bash
  run: |
    python -m venv "${{ runner.temp }}/venv"
    bin="${{ runner.temp }}/venv/bin"; [ -d "${{ runner.temp }}/venv/Scripts" ] && bin="${{ runner.temp }}/venv/Scripts"
    "$bin/python" -m pip install "${{ runner.temp }}"/dist/*.whl
    echo "BIN=$bin" >> "$GITHUB_ENV"
- name: version
  shell: bash
  run: |
    out=$("$BIN/python" -m conductor --version)
    [ "$out" = "conductor ${{ needs.verify-tag.outputs.version }}" ] || { echo "unexpected: $out"; exit 1; }
- name: acceptance tests against the installed wheel
  shell: bash
  env:
    CONDUCTOR_PYTHON: ${{ env.BIN }}/python
  run: pytest tests/acceptance
```

(`pip install <wheel>` は、パッケージの依存(YAML ライブラリ)をインデックスから取得する。このため、この手順はネットワークを使う。)
受け入れテストの `CONDUCTOR_PYTHON` の扱いは、3.3 節(作業ツリーの `src/` ではなく、仮想環境に入れた wheel を試験する)。デモ(4.17 節)は、リポジトリの作業ツリーを使うので、この手順では行わない。

### 6.4 単一の実行ファイルの配布(任意)

PyInstaller、`zipapp`(標準ライブラリの `python -m zipapp`)、Nuitka などで、OS ごとの単一の実行ファイル(または自己完結の 1 ファイル)を作ることは、**任意**とする。この仕様の受け入れ基準には含めない。確認できていない点:

- PyInstaller: 実行ファイルに Python の実行環境を同梱するので、OS ごとに別々に作る必要がある。大きさ(数 MB〜数十 MB の見込み。未確認)、起動時の展開の時間、ウイルス対策ソフトの誤検知、macOS の署名・公証、Windows の署名は、未確認。
- `zipapp`: 実行には Python が要る。YAML ライブラリを zip に同梱できるか(純 Python の部分だけで動くか。C の拡張がある場合の扱い)は未確認。
- 作った実行ファイルで、`sys.executable`、子プロセスの起動、`signal`、標準入出力の符号化の設定、`python -m conductor` を前提にしたテスト(AC-498、AC-520、AC-530)が、通常の Python と同じに動くか。
- 作った場合は、`CONDUCTOR_BIN` にその実行ファイルを渡して、受け入れテスト(AC-446)を通すこと。通らない項目は、結果を記録する。

---

## 7. デモ

### 7.1 構成

デモは、`conductor` の主な動きを、1 本のスクリプトで順に見せる。利用者は、リポジトリの根から実行する。スクリプトは、一時ディレクトリに定義とジョブを写して実行し、終了時にそのディレクトリを消す(リポジトリには何も残らない)。デモの定義は YAML。

```text
demo/README.md                 使い方と各場面の説明
demo/conductor.yaml            UNIX 系の定義(パイプライン nightly)
demo/retry.yaml                UNIX 系のリトライの定義(パイプライン retry-demo)。場面 10 だけが使う
demo/run-demo.sh               UNIX 系のスクリプト(シェル)
demo/scripts/{extract,transform,load,notify,flaky}.sh   UNIX 系のジョブ
demo/windows/conductor.yaml, retry.yaml, run-demo.cmd   Windows 版(コマンドプロンプトのバッチ)
demo/windows/scripts/{extract,transform,load,notify,flaky}.cmd   Windows 版のジョブ
```

### 7.2 実行の仕方(UNIX 系と Windows の差)

| 項目 | UNIX 系 | Windows |
|------|---------|---------|
| 実行 | `demo/run-demo.sh`(リポジトリの根から) | `demo\windows\run-demo.cmd` |
| 実行ファイルの選択 | 環境変数 `CONDUCTOR`(コンソールスクリプトのパス。直接実行する)。なければ、環境変数 `DEMO_PYTHON`(既定 `python3`)で、`PYTHONPATH=<リポジトリの根>/src` を付けて `-m conductor` を実行する。そのインタプリタで YAML ライブラリを import できなければ、`pip install -e .` を先に行うよう案内して、終了コード 1 で終わる(スクリプトは、依存を自動では入れない) | 同じ(`CONDUCTOR`、なければ `DEMO_PYTHON`(既定 `python`)で `-m conductor`) |
| 表示の言語 | `DEMO_LANG`(既定 `ja_JP.UTF-8`。英語は `en_US.UTF-8`)を `LANG` に設定。`LC_ALL`、`LC_MESSAGES` は外す | `DEMO_LANG`(既定 `ja_JP.UTF-8`。`en_US` のようにも書ける)。`LC_ALL`、`LC_MESSAGES` は空にする。コードページを実行中だけ 65001(UTF-8)にして、終わりに戻す |
| 一時停止 | `DEMO_PAUSE=1` のときだけ、各場面のあとで Enter を待つ | 同じ |
| ジョブの待ち時間 | `DEMO_SLEEP`(既定 0.5 秒。小数可) | `DEMO_SLEEP`(整数の秒。既定 1。`0` で待たない。小数は使えない) |
| ジョブの終了コード | 環境変数 `DEMO_<ジョブ名の大文字>_EXIT`(既定 0) | 同じ |
| 作業場所 | `mktemp -d` の一時ディレクトリに定義とスクリプトを写して実行。終了時に `trap` で消す | `%TEMP%` の下の一時ディレクトリ(`conductor-demo-<乱数>`)に写して実行し、終わりに消す |
| 事前の環境変数の処理 | `DEMO_RUN_ID`、`CONDUCTOR_RUN_ID` を消す(利用者の環境の影響を除く)。`PYTHONPATH`(デモが自分で付ける場合を除く)、`PYTHONIOENCODING`、`PYTHONUTF8` も消す | 同じ(空にする) |
| 色 | 見出し・コマンド・終了コードの行に ANSI の色。`watch` も色あり(標準出力が端末のとき) | 画面に色なし |
| 各コマンドの出力 | `$ conductor <引数>` の行、実行、`→ 終了コード <N>` | 同じ |
| ステートの一覧 | `ls conductor.yaml*.state`(なければ `(なし)`) | `dir /b conductor.yaml*.state`(なければ `(なし)`) |
| 場面の見出し | `=== <番号>. <題> ===` | 同じ |

### 7.3 定義とジョブ

`demo/conductor.yaml`(Windows 版は、`command` が `scripts\<名>.cmd`、`notify` は `[cmd, /c, 'scripts\notify.cmd']`。一重引用符の中の `\` は、そのまま `\`。**`.cmd` を起動する方法は 07 章に従う**。AC-510):

```yaml
format_version: 1

pipeline:
  name: nightly
  description: デモ用の夜間バッチ
  run_id_env: DEMO_RUN_ID

job:
  - id: extract
    command: sh scripts/extract.sh
    warning_exit_codes: [1]
    transitions:
      - on: warning
        next: notify
  - id: transform
    command: sh scripts/transform.sh
  - id: load
    command: sh scripts/load.sh
  - id: notify
    command: [sh, scripts/notify.sh]
```

`demo/retry.yaml`:

```yaml
format_version: 1

pipeline:
  name: retry-demo
  description: リトライのデモ

job:
  - id: flaky
    command: sh scripts/flaky.sh
    retry:
      exit_codes: [75]
      max: 3
      interval: 1s
```

ジョブのスクリプトの仕様(`extract`・`transform`・`load`・`notify`): `  [<名>] 実行中...` を表示する → `DEMO_SLEEP` の秒だけ待つ → `  [<名>] 終了コード <N> で終わります` を表示する → 環境変数 `DEMO_<名の大文字>_EXIT`(既定 0)の値 N で終わる。
`flaky` の仕様: カレントディレクトリの `flaky.count` の回数を 1 増やす(なければ 0 から)→ `  [flaky] <n> 回目の実行...` を表示する → 待つ → `n` が `DEMO_FLAKY_FAILS`(既定 2)以下なら `  [flaky] 終了コード 75 で終わります(一時的な失敗)` を表示して 75 で終わる。超えたら `  [flaky] 終了コード 0 で終わります` を表示して 0 で終わる。
(Windows 版のジョブは、待ちに `ping` を使うなど、実現の仕方が違ってよい。表示の文言は同じ。デモのジョブは、シェルのスクリプトであり、Python を使わない。`conductor` 自身だけが Python で動く。)

### 7.4 場面

| 場面 | 見せること | 実行するコマンド(UNIX 系。Windows も同じ引数) |
|------|-----------|--------------------------------------------------|
| 0 | 定義の中身 | `cat conductor.yaml`(Windows は `type`) |
| 1 | 全部成功。書いた順に 1 つずつ | `pipeline -v nightly` |
| 2 | `extract` が警告(`DEMO_EXTRACT_EXIT=1`)→ `transform`・`load` をスキップして `notify` へ | `pipeline -v nightly` |
| 3 | `load` のエラー(`DEMO_LOAD_EXIT=2`)で止まり、終了コード 2。止まった位置 | `pipeline -v nightly`、続けて `watch` |
| 4 | `load` から再開(`transform` は再実行しない)。完了してもステートは残り、`watch` が完了の行を出す。`--remove-on-complete` で完了時にステートを消す | `pipeline -v nightly`、`watch`、ステートの一覧、`pipeline -v --remove-on-complete nightly`、一覧、`watch` |
| 5 | 個別実行と初期化 | `run -v extract`、`reset` |
| 6 | YAML から JSON への変換 | `convert conductor.yaml --to json` |
| 7 | 検査。正しい定義は 0(`definition is valid`)、引用符が閉じていない定義は 64(`command has an unclosed quote`)。ジョブは実行されず、ステートも作られない | `validate -v`、誤りの定義 `bad.yaml`(`command: "sh -c 'echo hi"`)を作って `validate --config bad.yaml`、一覧 |
| 8 | run ID ごとに別のステートファイル(`conductor.yaml.<run ID>.state`)。同じ run ID で再開。完了後もステートは残り、`reset --run-id` で片づける | `pipeline -v --run-id build-1 nightly`(`load` で失敗)、環境変数 `DEMO_RUN_ID=build-2` で `pipeline -v nightly`、一覧、`watch --run-id build-1`、`pipeline -v --run-id build-1 nightly`(再開)、一覧、`watch --run-id build-1`、`reset --run-id build-1`、`reset --run-id build-2`、一覧 |
| 9 | `--run-id` は環境変数に勝つ。`--run-id ""` は run ID なしに固定。`reset --run-id` は 1 つ、`reset --all` は全部。最後の `watch` は `no state` | `DEMO_RUN_ID=env-1` で `pipeline`(`load` で失敗)、`DEMO_RUN_ID=env-1` と `--run-id opt-1`、`--run-id ""`、`--run-id opt-2`、一覧、`reset --run-id opt-1`、一覧、`reset --all`、一覧、`watch` |
| 10 | `flaky` が 2 回、終了コード 75 で失敗し、`conductor` は 1 秒待つたびに標準エラーへ `conductor: retry: job 'flaky' exited with code 75; retry <n> of 3 in 1 seconds` を出して実行し直し、3 回目で成功(`job=flaky result=success`) | `rm -f flaky.count`(Windows は `del`)、`pipeline -v --config retry.yaml retry-demo` |

最後に `デモは以上です。`(Windows 版も同じ)を出す。デモの受け入れテストは 4.17 節。

---

## 8. 依存ライブラリと `pyproject.toml` / 制約ファイルの運用

### 8.1 方針

- 依存は最小にする。**実行時の外部ライブラリは、YAML の読み取り 1 つだけ**を認める。ほかは、標準ライブラリを使う(引数の解析は自前、シグナルは `signal`、プロセスは `subprocess`、ファイルは `os`・`pathlib`、JSON の読み取りは自前、`dataclasses`・`enum`・`typing`・`tempfile`・`time`)。YAML の書き出し(`convert --to yaml`)も自前([09](09-converter.md))。
- 実行時の依存に、ネイティブの拡張(コンパイルを要するもの)を必須としない(AC-443)。YAML ライブラリが、任意の C の拡張を持つ場合は、使わなくても動くこと(純 Python の経路で動かす。8.2 節)。インストール時にコードを実行する仕組み(`setup.py` の実行)に頼らない(wheel で配るライブラリを選ぶ)。
- 開発時の依存は、`pytest`、`pytest-cov`、`mypy`、`ruff`、`build`、`pip-audit`、`types-PyYAML`(YAML ライブラリの型の情報。8.3 節)に限る。テスト用のタイムアウト・並列・モック・擬似端末のライブラリは入れない(擬似端末が要る項目 AC-182・AC-183 は、標準ライブラリの `pty` で行う。10 章)。
- `argparse` は、この仕様の引数の文法に合わないため使わない([04](04-cli.md)。理由は 4.12 節の末尾。CPython 3.9.6 で確かめた範囲: 省略形の受理、重複の後勝ち、`-vv` の受理、エラーの終了コード 2、`--config --help` を値にしない)。ほかの版の挙動(3.12 以降で、省略形や連結の扱いが変わった可能性)は未確認。
- `shlex`(`shlex.split`)は、`command` の分割に使わない([05](05-execution-flow.md) の規則と違うため。CPython 3.9.6 の `shlex.split` で確かめた: `"c\\d"` を `c\d`(バックスラッシュ 1 つ)にする(AC-080 の期待は、バックスラッシュ 2 つのまま)、`a\ b` を `a b`(1 語)にする(AC-082 の期待は `a\`・`b` の 2 語)、`"\\"x"` を `ValueError: No closing quotation` にする(AC-081 の期待は `\"x`)。分割は自前で行う)。

### 8.2 YAML ライブラリの選定

選定の基準(すべて満たすこと):

| 基準 | 理由 | 確かめる受け入れテスト |
|------|------|------------------------|
| (1) 誤りと、各キー・値の行番号(できれば列番号)を、入力から取り出せる(構文木かイベントの API で) | 定義の誤りを、行つきで出す([03](03-definition-format.md)) | AC-009、AC-037 |
| (2) 重複キーを検出できる(ライブラリの機能、またはイベント・構文木の走査で) | 重複キーを構文の誤りとして拒否する | AC-010、AC-226、AC-456 |
| (3) タグ(標準のタグも)・複数の文書・文字列でないキーを検出して拒否できる(明示のタグと、暗黙の解決を区別できる) | 定義と変換で拒否する | AC-217、AC-218、AC-453〜455、AC-531 |
| (4) 保守が続いている(未解決の重大な不具合や脆弱性の対応が止まっていない) | 長く使う | `pip-audit`、実装時の調査 |
| (5) アンカー・エイリアス・マージキーの展開を、自前で、上限つきで実装できる(エイリアスが、展開されずに取り出せる) | 爆発する入力に上限を付ける | AC-224〜230 |
| (6) 入れ子が深い入力で、再帰の上限を使い果たさず失敗を返せる(または、深さを数えて早く打ち切れる) | 2.2 節 | AC-232、AC-412、AC-524 |
| (7) 実行時の依存が少ない(望ましくはなし)。ネットワークを使わない。Python のオブジェクトを作らずに読める(`yaml.load` の系列を呼ばなくてよい)。型の判定を、自前で補える(YAML 1.2 のコアスキーマ) | 8.1 節。AC-458、AC-228、AC-525 | AC-419、AC-443、AC-458、AC-525、AC-531 |
| (8) 標準エラーに警告を出さない(または止められる) | AC-498 | AC-498 |
| (9) `PY_MIN` でも `PY_MAX` でも動く。wheel があり、純 Python でも動く | 6 章 | AC-479 |

候補(版と保守の状況は、実装時に再確認する):

| 候補 | 特徴 | 評価 |
|------|------|------|
| `PyYAML`(配布名 `PyYAML`、import 名 `yaml`) | **この文書の作成時に、版 6.0.3(CPython 3.9.6、macOS)で、次を確かめた**。(a) `yaml.parse(text, Loader=yaml.SafeLoader)` が、イベント(`MappingStartEvent`・`SequenceStartEvent`・`ScalarEvent`・`AliasEvent`・`DocumentStartEvent` など)を**遅延して**返し、各イベントが `start_mark.line`(0 始まり)・`start_mark.column` を持つ。(b) `ScalarEvent.tag` は、**明示のタグがあるときだけ**非 `None`(`!!str 1` で `tag:yaml.org,2002:str`。暗黙の場合は `None`)で、`implicit` の組と `value`(生の文字列。`yes`・`1:30`・`2001-01-01` が、そのまま文字列)も取れる。(c) アンカーは `anchor`、エイリアスは `AliasEvent` で、展開されずに取り出せる。(d) 文書は `DocumentStartEvent` の数で数えられる(`a: 1⏎---⏎b: 2` で 2 文書)。(e) タブのインデント(`a:⏎\t- 1`)が `ScannerError`、閉じていない `[` が `ParserError` で、`problem_mark.line`・`column` を持つ。(f) **イベントの API は再帰しない**(深さ 1,000 の `a: [[[…]]]` を `RecursionError` なしで読めた。ただし、時間は入れ子の深さに対して超線形(深さ 1,000 で 1.1 秒、3,000 で 8.4 秒)だった)ので、**イベントを読みながら入れ子の深さを数え、33 段目で打ち切る**(深さ 100,000・約 200 KB の入力で、打ち切りまで 0.38 秒)。構文木を作る `yaml.compose` は再帰し、深さ 2,000 で `RecursionError` になった(使わない)。(g) `yaml.SafeLoader`(純 Python)を使えば、ライブラリの C の拡張(`libyaml`)の有無に左右されない(`__with_libyaml__` は、この環境で真だった。`CSafeLoader` の位置や誤りの文言が、純 Python 版と同じかは未確認)。重複キーは、イベントの列から自前で検出する(`yaml.load` は後勝ちで黙って受理する。確認は記憶による)。型は暗黙のタグの解決(YAML 1.1: `yes`・`on`・`1_000`・60 進数・日付)に頼らず、生の `value` から、03 章 4.2 節の規則で自前で決める。実行時の依存はなし | **採用案**。版は、実装時に最新の安定版を確認して、`dependencies` に範囲で書き、`constraints.txt` に正確な版を固定する。基準(1)〜(3)・(5)〜(7)は、6.0.3 の上の(a)〜(g)で確認した(03 章 7 節の試験も参照)。(4)(8)(9)は、実装時に確認する。**スキャナと構文は、YAML 1.1 に基づく。1.2 との文法の差が受け入れテストの入力に影響するかは、実装時に確認する(未確認)** |
| `ruamel.yaml` | YAML 1.2 を扱い、位置情報(行・列)と、ラウンドトリップの機能を持つとされる。C の拡張を、任意の追加の依存(`ruamel.yaml.clib`)として持つとされる | 採らない(この文書の作成時に動かして確かめていない。基準(1)〜(6)を確認できていない。実装時に、PyYAML が基準を満たさない場合の代案として再評価してよい) |
| 自前の YAML 読み取り | — | 採らない(YAML の文法が大きく、誤りが入りやすい) |

**採用案**: `PyYAML`(上のとおり版を固定)。YAML を読む **定義と変換の入口**でだけ使い、`import yaml` は、その入口の関数の中で行う(遅延 import。`--version` などの起動を速くする。AC-489)。ステートは JSON(自前の読み取り)。ライブラリのイベント(`yaml.parse`)を、03 章 6.1 節の「木」(順序つきの `entries`・`seq`・文字列・整数(`int`)・浮動小数・真偽値・`None`)に変換する層を自前で書き、その層で、重複キー・タグ・複数文書・文字列でないキー・深さ・要素数・エイリアスの展開を扱う(`null` は拒否しない)。**`yaml.load`・`yaml.unsafe_load`・`yaml.full_load`・`yaml.safe_load` は、呼ばない**(`safe_load` も、暗黙の型判定と構築を行うため。AC-421、AC-531)。ライブラリのエラー文(`str(YAMLError)`)と例外は、表示に出さない(2.3 節)。必要なら、位置(`problem_mark`)だけを使う。ライブラリの警告(`warnings.warn`)は、標準エラーに出さない(`filterwarnings`。AC-498)。
実装時は、まず AC-010、AC-217〜232、AC-453〜460、AC-498、AC-525、AC-531 の入力を、小さな試験で候補に通し、基準(1)〜(9)を確かめてから決める。採用したライブラリと版は、`pyproject.toml`、`constraints.txt`、README に記録する。

**JSON の読み取り**は、自前の小さな字句・構文解析で行う([03](03-definition-format.md))。標準ライブラリの `json` は、CPython 3.9.6 で確かめた範囲で、次の理由から、定義・ステート・変換の入力の読み取りに使わない。(a) 構文の誤りの位置(`JSONDecodeError` の `lineno`・`colno`・`pos`)は取れるが、**成功した値には位置がない**(定義の誤りの行を出せない。AC-037、AC-475)。(b) 重複キーは、既定で後勝ちで黙って受理する(`object_pairs_hook` で検出はできる)。(c) `NaN`・`Infinity` を受理する(`parse_constant` で拒否はできる)。(d) `1` は `int`、`1.0` と `1e0` は `float` で区別されるが、`1e999` は `inf` になる。(e) `"\ud800"`(孤立サロゲート)を `str` として受理する。(f) 深い入れ子(100,000 段)で `RecursionError` になる(深さの上限を、解析の前に効かせられない)。(g) 整数の桁数の上限(`int()` の変換の上限。この文書の環境では未確認)で `ValueError` が起きうる。したがって、位置・重複・深さ・字句の厳格さが要る部分を、自前で読む。`json.loads` を使ってよいのは、テストと開発用のスクリプトだけ(AC-421。03 章が、製品のコードで `json` の一部を使う方針を定めた場合は、その範囲に合わせる。10.2 節)。JSON の出力(`json.dumps` の使用の可否と、`indent=2`・`ensure_ascii=False`・浮動小数の書式の細部)は、09 章に従う。

### 8.3 開発時の依存の一覧

版は、すべて「実装時に確認する」。`PY_MIN` と `PY_MAX` の両方で動く版を選ぶ(各ツールの `Requires-Python` を確かめる)。`constraints.txt` には、正確な版(`==`)を書く(8.4 節)。

| パッケージ | 用途 | 選定の基準 |
|------------|------|------------|
| `pytest` | テストの枠組み(`tmp_path`、`parametrize`、`skipif`、`faulthandler_timeout`) | `PY_MIN` で動く。設定のキー(`pythonpath`、`filterwarnings`)を使える版(`minversion` で下限を書く) |
| `pytest-cov` | カバレッジ(`coverage` を呼ぶ) | `pytest` と矛盾しない版 |
| `mypy` | 型の検査(`--strict`) | `PY_MIN` と `PY_MAX` の両方の構文を扱える版。`--python-version` で最小の版の意味の検査ができる |
| `ruff` | 静的検査(`check`)と整形(`format`)。規則の集合(`S`、`BLE`、`TID`、`ANN` など)を使える版 | 版を固定して、整形の差・規則の追加で、突然失敗しないようにする |
| `build` | sdist と wheel のビルド(`python -m build`) | PyPA の標準の入口 |
| `pip-audit` | 既知の脆弱性の検査 | 実行環境に依存しない(オンラインのデータベースを使う)。定期の検査(5.2 節) |
| `types-PyYAML` | `PyYAML` の型の情報(スタブ)。`mypy --strict` が、`import yaml` を型つきで通すため | `PyYAML` の版に合う版。`mypy` の設定で `ignore_missing_imports` に逃げない(AC-518)。この依存の追加は、共通の設計判断の一覧(`pytest`・`pytest-cov`・`mypy`・`ruff`・`build`・`pip-audit`)への追加であり、10.2 節で合意を取る |

上記以外の開発時の依存(`pytest-timeout`、`pytest-xdist`、`pytest-mock`、`hypothesis`、`tox`、`nox`、`black`、`isort`、`flake8`、`pylint`、`pip-tools`、`twine`(手元の道具としては使ってよいが、リポジトリの依存には入れない)など)は入れない(`scripts/check_deps.py` が検査する。5.4 節)。

### 8.4 `pyproject.toml` と制約ファイルの運用

| 項目 | 決め |
|------|------|
| 配布名・import 名 | `conductor`(置き換えてよい。置き換える場合は、README、リリースの成果物の名前、`CONDUCTOR_BIN`・`CONDUCTOR_PYTHON` の例、`scripts/check_*.py` の名前の参照、`src/` の下のディレクトリ名をすべて合わせる) |
| `requires-python` | `">=<最小版>"`(例: `>=3.11`)。実装時点で、公式にセキュリティ修正が続いている版のうち、最小と最新をサポートする。**実装時に確認して決める**。5.4 節の検査が、`target-version`・README・`ci.yml` の `PY_MIN` との一致を保つ |
| `dependencies` | 8.2 節で採った YAML ライブラリ 1 つだけ。**範囲**で書く(例: `PyYAML>=<確認した版>,<<次の主版>`)。利用者の環境の他のパッケージと衝突しないよう、下限を、確認した最小の版にし、上限を、次の主版の手前にする |
| `[project.optional-dependencies]` | `dev` だけ(8.3 節の一覧。範囲で書く)。ほかの extra を作らない |
| `constraints.txt` | コミットする。**直接の依存(実行時と `dev`)のすべてだけ**を、正確な版(`==`)で書く(行は `名前==版`。コメントは `#`)。間接の依存の固定はしない(OS と Python の版で、必要な間接の依存が違い(例: Windows でだけ要る `colorama`、古い版でだけ要る `exceptiongroup` など。確認は記憶による)、1 つのファイルで固定すると、版や OS により入らない行ができるため)。**この方針の弱点**: 間接の依存の版は、インストールの時点の最新で決まり、ビルドの再現が完全ではない。弱点を許容しない場合は、OS・Python の版ごとの完全な固定のファイル(`pip freeze` の出力。ロックを作る別の道具の採用は、依存の許可の一覧を変える)を、別途検討する(未確認) |
| 入れ方 | CI とリリースは、`python -m pip install -c constraints.txt …`(制約として使う)だけで入れる。`pip install` に、制約なしで依存を入れない |
| 更新 | 直接の依存の版を上げるときは、`pyproject.toml` の範囲と `constraints.txt` を同じコミットで更新し、`pytest`(3 OS)、`pip-audit` を通す。自動の更新ツール(Dependabot など)の導入は任意(未確認) |
| 検査 | CI の `consistency`(5.1 節): `check_deps.py`(許可の一覧、`constraints.txt` との整合、ビルドバックエンドの固定)、依存を入れて `pip check` が通る |
| `build-system.requires` | ビルドバックエンドを正確な版(`==`)で固定する(例: `hatchling==<実装時に確認>`)。`python -m build` の分離した環境は、`constraints.txt` の対象外なので、`requires` の中で固定する |
| `[tool.*]` | `pytest`・`coverage`・`ruff`・`mypy`・`hatch` の設定を、`pyproject.toml` に集める(9.2〜9.4 節)。別の設定ファイル(`pytest.ini`・`setup.cfg`・`tox.ini`・`.coveragerc`・`mypy.ini`・`ruff.toml`)を置かない |
| インストール | `pip` の更新、`--user`、グローバルな環境への入れ方は、リポジトリの運用に含めない(仮想環境で行う)。CI は、`actions/setup-python` の環境に入れる |

---

## 9. リポジトリのレイアウト

### 9.1 ファイルの構成

```text
pyproject.toml                  8.4 節、9.2 節(ビルド・依存・pytest・coverage・ruff・mypy の設定)
constraints.txt                 直接の依存の正確な版(8.4 節)。コミットする
LICENSE                         Apache License 2.0(配布物に入れる)
README.md                       利用者向けの文書(docs-check が必須の語を検査する。5.1 節)
.gitignore                      9.5 節
.github/workflows/ci.yml        5.1 節
.github/workflows/audit.yml     5.2 節
.github/workflows/release.yml   6.3 節
scripts/                        check_version.py、check_python_version.py、check_deps.py、check_readme.py、check_coverage.py、check_dist.py(5.4 節)
src/conductor/__init__.py       空(または版以外の何も import しない)
src/conductor/__main__.py       入口(薄い。try/except BaseException と sys.excepthook で終了コード 70。sys.exit(main()))
src/conductor/version.py        版の定数 VERSION(6.2 節)
src/conductor/runid.py          run ID
src/conductor/definition/       定義の読み込み・検査・型・エラー
src/conductor/convert/          YAML ⇔ JSON
src/conductor/flow/             純粋な判断
src/conductor/state/            ステートの読み書き・照合・reset
src/conductor/runner/           子プロセスの起動・シグナル(platform.py に OS の分岐を集める)
src/conductor/report/           出力・メッセージ・ロケール・エスケープ・watch の表
src/conductor/cli/              引数・各サブコマンド・Deps・終了コード
tests/__init__.py, tests/conftest.py     共通の fixture(3.3 節)
tests/unit/<部品>/              各部品の単体テスト(3.2 節)
tests/acceptance/test_*.py      受け入れテスト(3.3 節)
tests/inspect/test_*.py         点検のテスト(ast の走査、依存の向き)
tests/perf/test_*.py            性能の計測(2.1 節。CONDUCTOR_PERF=1 のときだけ)
tests/jobs/tj.py                テスト用ジョブ(3.4 節。配布しない)
tests/support/                  __init__.py、harness.py、fakes.py、marks.py、tree.py、xorshift.py、inject_alarm.py、inject_hook.py、inject_delay.py
tests/fixtures/                 入力例(YAML・JSON。壊れたものを含む。ruff の対象外)
tests/WINDOWS_CTRL_C_CHECKLIST.md   Windows の Ctrl+C と Ctrl+Break の手動確認の手順と記録欄(AC-301、AC-487)
demo/                           7 章
docs/spec/python/               この仕様書
```

パッケージ(ディレクトリ)の責務と依存の向きは、[02](02-architecture.md) のとおり。`src/conductor/version.py` は、`__main__.py` と `cli/`(`--version` の表示)から import される。`src/` は `tests/` を import しない(AC-422)。

### 9.2 `pyproject.toml`

(書式は、`pyproject.toml` の標準の書き方。`<実装時に確認>` は、版を実装時に決める印。**この文書の作成時には、ここに挙げた設定のキーを、実際のツールで動かして確かめていない**。キー名・規則の番号は、採用した版の文書で確かめる。)

```text
[build-system]
requires = ["hatchling==<実装時に確認>"]
build-backend = "hatchling.build"

[project]
name = "conductor"
version = "0.8.0"
description = "A CLI that runs jobs in order and records where it stopped"
readme = "README.md"
requires-python = ">=3.11"
license = { text = "Apache-2.0" }
classifiers = [
  "Programming Language :: Python :: 3",
  "Programming Language :: Python :: 3.11",
  "Programming Language :: Python :: 3.12",
  "Programming Language :: Python :: 3.13",
  "Programming Language :: Python :: 3.14",
  "Operating System :: OS Independent",
  "License :: OSI Approved :: Apache Software License",
]
dependencies = [
  "PyYAML>=<実装時に確認>,<<次の主版>",
]

[project.optional-dependencies]
dev = [
  "pytest>=<実装時に確認>",
  "pytest-cov>=<実装時に確認>",
  "mypy>=<実装時に確認>",
  "ruff>=<実装時に確認>",
  "build>=<実装時に確認>",
  "pip-audit>=<実装時に確認>",
  "types-PyYAML>=<実装時に確認>",
]

[project.scripts]
conductor = "conductor.__main__:main"

[tool.hatch.build.targets.wheel]
packages = ["src/conductor"]

[tool.hatch.build.targets.sdist]
include = ["/src", "/tests", "/pyproject.toml", "/LICENSE", "/README.md"]
```

(`license = { text = "Apache-2.0" }` は、古い書式。新しい書式(SPDX の式の文字列と `license-files`)を使えるかは、ビルドバックエンドの版による(未確認)。`[tool.hatch.build.targets.sdist]` の `include` の書式(先頭の `/` の意味)は、実装時に確認する。`PKG-INFO` は、ビルドバックエンドが自動で入れる。)

pytest と coverage の設定は 3.9 節。`ruff` と `mypy` の設定は 9.3・9.4 節。

### 9.3 `ruff` の設定(`pyproject.toml`)

```text
[tool.ruff]
target-version = "py311"
line-length = 100
src = ["src", "tests"]
extend-exclude = ["tests/fixtures"]

[tool.ruff.lint]
select = ["E", "F", "W", "I", "N", "UP", "B", "S", "BLE", "ANN", "T20", "TID", "SIM", "RUF", "PT", "C4"]

[tool.ruff.lint.per-file-ignores]
"src/conductor/runner/*.py" = ["S404", "S603", "TID251"]
"src/conductor/__main__.py" = ["TID251", "BLE001"]
"tests/**" = ["S101", "S404", "S603", "S607", "S105", "S106", "T201", "TID251", "ANN401"]
"scripts/**" = ["T201", "TID251"]

[tool.ruff.lint.flake8-tidy-imports.banned-api]
"subprocess".msg = "only src/conductor/runner may start processes"
"os.system".msg = "no shell"
"os.popen".msg = "no shell"
"sys.exit".msg = "only src/conductor/__main__.py may exit"
"os._exit".msg = "only src/conductor/__main__.py may exit"
"json.loads".msg = "use the own JSON reader"
"json.load".msg = "use the own JSON reader"
"yaml.load".msg = "never construct Python objects from YAML"
"yaml.unsafe_load".msg = "never construct Python objects from YAML"
"yaml.full_load".msg = "never construct Python objects from YAML"
"yaml.safe_load".msg = "use yaml.parse events and resolve types by hand"
"typing.cast".msg = "no casts"
"socket".msg = "no network"
"ssl".msg = "no network"
"urllib".msg = "no network"
"http".msg = "no network"
"ftplib".msg = "no network"
"smtplib".msg = "no network"
"xmlrpc".msg = "no network"
"asyncio".msg = "synchronous model"
"threading".msg = "synchronous model"
"multiprocessing".msg = "no extra processes"
"pickle".msg = "no object serialization"
"marshal".msg = "no object serialization"
"shelve".msg = "no object serialization"
"ctypes".msg = "no native code"
"runpy".msg = "tests only"
"importlib".msg = "no dynamic import"
```

- **この文書の作成時には、`ruff` を動かしていない**。`select` の各規則の番号(`S`・`BLE`・`ANN401`・`T201`・`TID251` など)と、`banned-api` の書式(`"名前".msg = "…"`)、`banned-api` が、`import sys` の後の `sys.exit(…)` のような属性の参照を検出するか、`per-file-ignores` でファイルごとに許可できるか、は、採用した版の文書で確かめる(未確認)。検出されない形は、AC-116・AC-420・AC-421・AC-499・AC-500 の `ast` の点検(`tests/inspect/`)が補う。
- `yaml` の import を、YAML の入口の 1 モジュール(例: `definition/` の YAML の読み取りの部品と、`convert/` の入口)だけに限る(遅延 import のため、`import yaml` は関数の中。`ruff` の `PLC0415`(関数の中の import)は選ばない)。
- `# noqa`・`# type: ignore` は、製品のコード(`src/`)に書かない(AC-421)。テストでの使用は、理由を同じ行に書く。
- `ruff format` の既定の整形を使う(`line-length` 以外は既定)。`ruff format --check .` が差分を出さないこと(AC-480)。この文書のコードの断片は、整形の規則に合わせて書いた見込みだが、`ruff format --check` を通ることは確かめていない。実装時に、`ruff format .` で整形してからコミットする。

### 9.4 `mypy` の設定(`pyproject.toml`)

```text
[tool.mypy]
files = ["src", "tests", "scripts"]
strict = true
warn_unreachable = true
disallow_any_explicit = true
explicit_package_bases = true
mypy_path = "src"

[[tool.mypy.overrides]]
module = ["tests.*"]
disallow_any_explicit = false
```

- `strict = true` は、`mypy --strict` と同じ(個別の `disallow_untyped_defs`・`warn_return_any` などを含む)。コマンド行の `mypy --strict` と、設定の `strict = true` の両方が、同じ結果になること(AC-483)。
- `disallow_any_explicit` は、製品のコード(`src/`)の `Any` の明示を禁じる(AC-421、AC-518)。テストは、`json.loads` の結果などで `Any` が要る場合があるので、緩める(上の上書き)。`scripts/` は、緩めない(`dict[str, str]` などで書く)。
- `sys.platform` の分岐は、`mypy` が、実行する OS だけを検査する(他の OS の分岐は到達できないコードとして飛ばす)。3 OS の CI で `mypy` を通す必要はないが、`lint` は Linux だけで動く。**Windows の分岐の型の誤りを、Linux の `mypy` は見逃す**ことがある(`mypy --platform win32` で確かめられるか、実装時に確認)。`lint` に `mypy --strict --platform win32` を足すかは、実装時に決める(未確認)。

### 9.5 `.gitignore`

少なくとも、次を除く: `.venv/`、`venv/`、`__pycache__/`、`*.pyc`、`dist/`、`build/`、`*.egg-info/`、`.pytest_cache/`、`.mypy_cache/`、`.ruff_cache/`、`.coverage`、`coverage.json`、`htmlcov/`、各 OS のエディタの一時ファイル。`constraints.txt` と `LICENSE` は除かない。

---

## 10. 未確認の事項と、他の章との整合が要る点

### 10.1 未確認の事項

この文書の作成時に確かめていない点だけを挙げる。他の章で確認済み、または決定済みの点は、挙げない。**この文書の作成時に実機で確かめた範囲は、1 章と、8.2 節の PyYAML・`json`・`argparse` の項に書いた**(macOS、CPython 3.9.6、PyYAML 6.0.3。`PY_MIN`・`PY_MAX` の版では、どれも確かめていない)。

- ブランチ保護(どのジョブを必須にするか)は、リポジトリの設定であり、リポジトリ内のファイルからは確認できない。
- 現在の行の網羅率の実測値、各性能の計測値(起動時間、import の時間、`run` のオーバーヘッドなど)は、この文書の作成時には測れていない(基準だけを記した。「仮」は、実測して見直す)。`python -X importtime` の出力の書式(累積の列)は、実機で確認した(`import time: <自己> | <累積> | <名前>`)が、`conductor` の import の時間は測っていない。
- `ruff`、`mypy`、`pytest`、`pytest-cov`、`build`、`hatchling`、`pip-audit`、GitHub Actions を実際に動かしては確かめていない。版、設定のキー・書式、フラグ、終了コードの規則は、**実装時に確認する**。特に: `actions/setup-python` の `cache: pip`・`allow-prereleases`、`strategy.matrix` で `env` を使えないこと(5 章の冒頭)、`${{ matrix.py == 'min' && env.PY_MIN || env.PY_MAX }}` の式、`pytest` の `pythonpath`・`faulthandler_timeout`・`filterwarnings` の挙動、`pytest-cov` の `--cov-fail-under` と `coverage.json` の `totals.percent_covered`、`ruff` の `banned-api` の検出範囲、`mypy --python-version`・`--platform`、`pip-audit` のオプションと終了コード、`python -m build` の分離環境と `hatchling` の sdist の `include` の書式、wheel のファイル名の正規化。
- YAML ライブラリ(PyYAML)の挙動のうち、`CSafeLoader` との差、YAML 1.1 と 1.2 の文法の差(受け入れテストの入力への影響)、`PY_MIN`・`PY_MAX` での動作、警告の有無、巨大な入力・深い入れ子での時間(超線形だった。1 MiB の入力で、33 段目の打ち切りまで何秒かは未測定)。
- Windows の実機: `.cmd` / `.bat` の起動の方法が、最小の Python でも最新の Python でも動くこと(AC-113〜115、AC-486、AC-510、AC-513。07 章の決定)。子プロセスの終了コードが `-1`(0xFFFFFFFF)のときの `Popen.returncode` の値(AC-112)。`signal.SIGBREAK` の番号(AC-301)。Ctrl+C の手動の確認(AC-301。記録は作成時点で存在しない)。`os.replace` の、同時の読み手に対する原子性(AC-322)。
- Linux の実機: `Popen(start_new_session=True)` が新しいセッションを作ること(AC-288。仕様の記述に基づく。確認していない)。ディレクトリへの `os.unlink` が `IsADirectoryError`(または `PermissionError`)になること(06 章)。`/proc/<pid>/stat` の書式(AC-403)。
- 最小の Python と最新の Python での、07・08・06・05 章の【実測】の再確認(確認した版は、各章に従う)。
- `sleep` / `trap` モードの `TJ` が、SIGTERM・SIGINT・SIGHUP で「シグナルで終了した」と観測されること(自分に同じシグナルを送り直す設計。3.4 節)。SIGQUIT の既定の動作。
- 疑似端末(AC-182、AC-183)を、標準ライブラリの `pty.openpty()` で作る手順は、この文書の作成時には実行していない。
- 大文字小文字を区別しないファイルシステムでの run ID の衝突(AC-260)は、仕様の記述に基づく。
- 非 UTF-8 のバイトを含むファイル名を、macOS の APFS で作れないこと(AC-349)。Linux で作れること(未確認)。
- `Popen.send_signal` が、終了済みの子に送らない作り(AC-289)の、各版(`PY_MIN`〜`PY_MAX`)での挙動。1 章のソースのコメント(bpo-38630)は、3.9.6 のもの。
- `sys.set_int_max_str_digits` の既定の値(4300 と記憶している)と、その上限に達したときの `ValueError`(AC-502)は、この文書の環境(3.9.6)に機能がなく、確認していない。
- `macos-latest` のランナーに `shasum` / `sha256sum` があること、`ubuntu-24.04` で `ps` などが使えること。`windows-2022` の `bash` での、`dist/*.whl` の展開と、`$RUNNER_TEMP` の形式。
- 単一の実行ファイルの配布(6.4 節)は、任意で、この文書では確かめていない。
- `build-repro`(5.1 節の任意のジョブ)で、wheel が、3 OS で同じ結果になること。
- PyPI への公開の手順の細部(二要素認証、API トークン、信頼できる発行者、配布名の取得)。
- 起動の性能の測定を強制に切り替える予定の有無は、決まっていない。
- 標準出力への書き込みの失敗を、実際のプロセスで確かめる方法のうち、`BrokenPipeError`(AC-493、AC-515、AC-516)は、読み手が閉じるタイミングに依存する。確実に起こすための調整(出力の大きさ)は、実装時に確認する。`SIGPIPE` が既定で無視されること(AC-493)。
- `-X dev -W error` で、依存のライブラリが `ResourceWarning` などを出さないこと(AC-498)。
- `argparse` が、3.9.6 より新しい版で、この仕様の文法のどこまでを満たすか(4.12 節の末尾)。
- 受け入れテストの期待の値の一部は、定義・ステート・変換の他章([03](03-definition-format.md)、[06](06-state-file.md)、[09](09-converter.md)、[10](10-errors-and-exit-codes.md))の字句に依存する。他章と食い違う場合は、他章を正とし、この文書の期待を直す。

### 10.2 他の章との整合が要る点

この文書が、他の章の決定に依存している、または、他の章に確認が要る点。

- **開発時の依存に `types-PyYAML` を足している**(8.3 節)。共通の設計判断の一覧(`pytest`・`pytest-cov`・`mypy`・`ruff`・`build`・`pip-audit`)を超える。`mypy --strict` が、型の情報のない `import yaml` を通さないための追加である。置き換える方法(型の情報のない `yaml` を、1 つの部品に閉じて、型つきの `Protocol` で包む)を採る場合は、8.3 節と `check_deps.py` を直す。
- **JSON の読み取り**(8.2 節): 製品のコードで `json.loads` を使わない(自前の読み取り)前提で、AC-421 と `ruff` の禁止の一覧を書いた。03 章が、`json` の一部の機能(`object_pairs_hook` など)の使用を定めた場合は、AC-421・AC-461〜464・9.3 節を合わせる。`json.dumps` の使用は、09 章に従う。
- **`.cmd` / `.bat` の起動**(AC-113〜115、AC-116、AC-486、AC-510、AC-513): 禁止文字を含む引数の拒否と、起動の方法は、07 章の決定に従う。この文書は、拒否(127)と、安全な引数(`plain`)の成功(7)だけを期待として書いた。
- **新しいセッションにするか**(AC-288): 07 章の決定に従う(両方の場合の期待を書いた)。
- **待ちの区間の長さ**(AC-275、AC-403、AC-503): 07 章の決定に従う。この文書は、1 秒以内の中断、CPU 時間 50 ms 以下、1 区間が 1 秒以下、だけを要求する。
- **終了時の flush・`--help` / `--version` の書き込み失敗**(AC-496、AC-515): 08 章 1.2 節の決定(書き込みの失敗を無視して 0)に従う。`watch` は 65、`convert` は 67(10 章)。
- **孤立サロゲートの表示**(AC-519、4.14 節): 08 章のエスケープの規則に従う。この文書は、`\u{dcff}` の形を例として書いた。
- **標準出力・標準エラーの符号化の設定**(AC-520): 08 章の決定(起動時に `reconfigure(encoding="utf-8", errors=…)`)に従う。`errors` の値は 08 章。
- **`KeyboardInterrupt` の扱い**: 共通の設計判断では、`SystemExit` を除く `BaseException` を入口で捕捉し(終了コード 70)、`KeyboardInterrupt` の扱いを別に決める、とされている。`run`・`pipeline` では、`SIGINT` のハンドラが旗を立てるので `KeyboardInterrupt` は起きない。`validate`・`watch`・`reset`・`convert`・使い方の誤りの経路で、Ctrl+C を受けたときの終了コード(例: 130)と出力の有無は、07 章と 10 章に従う。この文書に、この項目の受け入れテストはない(決まり次第、4.10 節に足す)。
- **NUL を含む `command`**(AC-522)と、**空白の定義**(AC-523): 03 章の規定に従う(未確認)。
- **YAML の型判定**(AC-228、AC-458、AC-525): 03 章 4.2 節の規則が正本。
- **`threading` の使用**(AC-420): 共通の設計判断のとおり、使わないのが原則。使う場合は、02・07 章に、理由・所有者・終了条件の表を置き、この文書の AC-420 の許可の一覧に加える。
- 受け入れテストの ID は、この文書の中で一意(001〜531)。他の章の ID は、この文書と独立である。
