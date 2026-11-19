# 00. README(Python 版 仕様書の案内)

この仕様書は、コマンドラインツール `conductor` を、Python でゼロから実装し直すための仕様である。`conductor` は、1 つの定義ファイルに書かれたジョブ(外部コマンド)の並びを、終了コードで分類しながら順に実行し、途中経過をステートファイルに残して、失敗したところから再開できるようにする、コマンドラインツールである。

この文書は、仕様書全体の入口である。範囲と版、目次、読む順序、規定の強さの語、用語集、この版が採った書式・実行環境の選択をまとめる。

## 1. 仕様書の範囲と版

| 項目 | 内容 |
|------|------|
| 対象 | `conductor` の利用者から見える挙動を、Python(CPython)で再実装するための仕様 |
| 版 | `conductor 0.8.0` と同じ利用者向けの挙動。`conductor --version` は `conductor 0.8.0` を出す(版の文字列は `src/conductor/version.py` の定数 `VERSION` が唯一の定義。`pyproject.toml` の `version` との一致をテストで検査する) |
| 実装言語・実行環境 | CPython。型ヒントを全面に付け、`mypy --strict` を通す。サポートする版は「実装時点で、公式にセキュリティ修正が続いている版のうち、最小と最新」で、具体的な版は**実装時に確認**して `pyproject.toml` の `requires-python` に最小版を書く。**この文書の仮置きは、最小 3.11、最新 3.15**(確認した値ではない。CI の `PY_MIN`・`PY_MAX`、`mypy` の `python_version`、`ruff` の `target-version`、分類子の例も、この仮置きで書いてある)。規定が使う言語機能の下限は 3.10(`dataclass(slots=True)`、`X \| None`、`match` 文)で、`typing.assert_never` と `enum.StrEnum` は 3.11 以降。実装時に最小版が 3.10 でなくなる(上がる)なら、規定は変わらない |
| 外部ライブラリ | 実行時は YAML の読み取りの 1 つだけ(採用案は `PyYAML`。版は固定する。選定の基準と候補は [03](03-definition-format.md) 7 章と [11](11-quality-and-build.md) 8 章)。ほかは標準ライブラリだけ。開発時の依存は、`pytest`、`pytest-cov`、`mypy`、`ruff`、`build`、`pip-audit` と、YAML ライブラリの型の情報 `types-PyYAML`(`mypy --strict` が `import yaml` を型つきで通すため。[11](11-quality-and-build.md) 8.3 節)の 7 つ |
| 対象 OS | Linux、macOS、Windows の 3 つ。配布物は、sdist と純粋な Python の wheel で、3 OS で同じものが動く |
| 同じにするもの | サブコマンド、オプション、終了コード、実行の意味論(分類・分岐・再開・リトライ・run ID)、メッセージの文言(日本語・英語)、`watch` の表 |
| 含まないもの | 実装そのもの(コードは規定しない。骨格のコードは、型とシグネチャの例である)。各章が「実装に任せてよい」と書いた点 |

この仕様書だけで、同じ受理・拒否・出力・終了コードを返す実装を作れることを目標とする。確認できなかった点は「未確認」と明記し、[12](12-rebuild-guide.md) 3 章にまとめた。Python と各ライブラリの挙動で、作成時に実際に動かして確かめたものは、「確認」「実測」「確認済み」と書いてある(macOS が中心。CPython は 3.9.6・3.10.7・3.14.10・3.15.2 のうち、章ごとに記した版。YAML ライブラリ `PyYAML` は 6.0.3)。書いていないものは、公式の文書の知識によるもので、「実装時に確認する」。最小の版と最新の版の Python、Linux、Windows での再確認は、全章に共通の未確認である。

## 2. 目次

| ファイル | 内容(1 行要約) |
|----------|-----------------|
| [00-README.md](00-README.md) | この文書。範囲、目次、読む順序、規定の強さの語、用語集、書式・実行環境の選択 |
| [01-overview.md](01-overview.md) | 目的、ユースケース、設計方針、非目標、制約、利用者像、概念の概観 |
| [02-architecture.md](02-architecture.md) | パッケージの構成と公開する名前、依存の向き、型の骨格、差し替え口 `Deps`、実行のシーケンス、同期モデル、`pyproject.toml`、エラー処理の方針 |
| [03-definition-format.md](03-definition-format.md) | 定義ファイルの書式(YAML/JSON)、読み取りの段階、上限、YAML ライブラリの選定と補完、検査、エラーの種類(31 種)と位置の出し方 |
| [04-cli.md](04-cli.md) | コマンドライン(6 つのサブコマンド、オプション、`argparse` を使わない理由、引数の解釈の手順、非 UTF-8 の引数、run ID、環境変数) |
| [05-execution-flow.md](05-execution-flow.md) | 実行の意味論(分類、遷移、起動前の判断、リトライと `int` の飽和した待ち時間、再開、完了、`conductor/flow/` の純粋関数) |
| [06-state-file.md](06-state-file.md) | ステートファイル(論理項目、JSON の形式、原子的な書き込み、読み込みの 14 段階の検査、JSON の読み取りの共用、整合性、`reset`) |
| [07-process-and-signals.md](07-process-and-signals.md) | `command` の分割、`subprocess.Popen` による子プロセスの起動と待ち、中断の合図 `Interrupt`、シグナルの受信と転送、リトライの待機、OS 別の差(`platform.py`)、Windows の `.cmd` の起動、`KeyboardInterrupt` |
| [08-output-and-i18n.md](08-output-and-i18n.md) | 出力先、標準出力の flush と `BrokenPipeError`、`-v`、メッセージカタログ(日本語・英語)、`watch` の表、ロケール、エスケープ、`--help` の全文 |
| [09-converter.md](09-converter.md) | `convert`(YAML と JSON の相互変換)の読み取りの差分、書き出しの形式、YAML の plain の条件と理由、往復の保証、エラー |
| [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md) | 終了コードの全件と決定順序、エラー分類の一覧、Python でのエラー型、想定外の内部失敗(70)、`KeyboardInterrupt` |
| [11-quality-and-build.md](11-quality-and-build.md) | 非機能要件、テスト構成、受け入れテスト一覧(AC-001〜536)、CI(GitHub Actions)、リリース(sdist と wheel)、デモ、依存、リポジトリのレイアウト |
| [12-rebuild-guide.md](12-rebuild-guide.md) | ゼロから実装する順序、Python 特有の落とし穴、未確認事項の一覧、最初に確かめる小さな試験 |

## 3. 読む順序

| 目的 | 読む順序 |
|------|----------|
| 全体を理解する | 00 → 01 → 02 → 05 → 03 → 06 → 04 → 07 → 08 → 10 → 09 → 11 |
| 実装を始める | 00 → 01 → [12](12-rebuild-guide.md)(実装の順序と最初の試験)→ 12 が指す各章 |
| 定義の読み込みだけを作る | 03 → 09(読み取りを共有)→ 10(エラー) |
| 実行の流れだけを作る | 05 → 06 → 07 → 10 |
| 出力の文言を確かめる | 08 → 10 |
| 受け入れる | 11(各 AC は、該当章の字句が正本) |

各章の終わりに、その章の「実装に任せてよい点」と「未確認」がある。受け入れテスト(11 章)の期待は、定義・ステート・変換・終了コードの各章(03、06、09、10)の字句と食い違うときは、それらの章を正とする。章どうしの食い違いは、03(定義)、05(実行フロー)、06(ステート)、07(プロセスとシグナル)、09(変換)の定めを正とする。

## 4. 規定の強さの語

章によって、記号や語が少し違うが、意味は次の 3 段階に対応する。

| 強さ | 意味 | 章での表記 |
|------|------|------------|
| 必須 | 利用者から見える結果(受理・拒否、出力の文字列、終了コード、作られるファイルの内容)であり、同じにしなければならない。受け入れテストで確かめる | 「必須」「【規定】」「〜しなければならない」 |
| 目安 | 推奨する値や分け方。利用者から見える挙動を損なわない範囲で、別の値や分け方を採ってよい。基準を超えたときは警告として記録する | 「目安」「【目安】」「推奨」「仮」(暫定として置いた値) |
| 任意 | 利用者から見える結果が変わらない範囲で、実装が自由に選んでよい | 「任意」「【任意】」「実装に任せてよい」「裁量」 |

- 【Python の設計】(07 章ほか)は、Python の都合で選んだ設計で、利用者から見える結果が同じなら変えてよい(目安と同じ扱い)。ただし、各章が「規定」と明記したもの(`shell=False`、転送は 1 回、`sys.platform` は `platform.py` だけ、など)は必須である。
- 【実測】(07 章ほか)は、作成時に実際に動かして確かめた挙動の印である。規定の強さではない。
- 骨格のコード(型・シグネチャ)は、名前や細部が他の章の定めと食い違うときは、03・05・06・07・09 の定めを正とし、骨格を読み替える。
- 「実装時に確認」は、版や道具の選定など、実装の時点で決める値で、この文書は決めない。「未確認」は、作成時に確かめられなかった点で、断定していない。実装時に確かめる。

## 5. 用語集

02〜11 章で使っている語を、次のように統一する。

| 語 | 意味 |
|----|------|
| 定義ファイル | パイプラインを書いたファイル。YAML(`.yaml`、`.yml`)か JSON(`.json`)。既定の名前は `conductor.yaml` |
| 定義 | 定義ファイルを検証して読み込んだ結果(`Definition`。`format_version`、`pipeline`、`job`) |
| パイプライン | 名前をもち、ジョブの並びからなる 1 つの流れ。1 つの定義ファイルに 1 つ |
| ジョブ | 実行する外部コマンド 1 つと、その終了コードの扱い(成功・警告のコード、遷移、リトライ)。ID をもつ |
| ジョブID | ジョブの識別子(`id`)。定義の中で一意。空でなく、`@` で始まらない |
| `command` | ジョブが実行するコマンド。文字列(分割して実行する)か、文字列のリスト(そのまま渡す)。シェルは介さない |
| 成功コード・警告コード | ジョブの終了コードのうち、成功(`success_exit_codes`。省略時は `[0]`)・警告(`warning_exit_codes`)として扱う値 |
| 分類 | ジョブの結果を、成功・警告・エラー・分類なし(起動前の中断)に分けること |
| 遷移(`transitions`) | 成功・警告のとき、次に実行するジョブを指定すること。`@end` は、残りをスキップして終える |
| スキップ | 遷移などで、実行されないジョブ。理由は「遷移による」(`byTransition`)「`@end` による」(`byEnd`) |
| 拒否 | 順序外などで、`run` が実行を断ること(終了コード 66。`Reject("OutOfOrder")`)。スキップ対象・`@end` より後ろ・実行済みのジョブは、拒否ではなくスキップ(終了コード 0) |
| リトライ(`retry`) | 決めた終了コードで終わったジョブを、待ってから実行し直すこと。最大回数・間隔・倍率・上限を指定できる。待ち時間は `int` の秒(64 ビット符号なしの上限で飽和) |
| run ID | 実行単位を区別する文字列(`A-Za-z0-9._-`、UTF-8 で 1〜64 バイト、先頭は `.` でない)。ステートファイルを分ける |
| ステートファイル | 実行の途中経過を JSON で保存するファイル。名前は `<定義ファイル名>.state`(run ID があれば `<定義ファイル名>.<run ID>.state`) |
| 再開位置(`resume_job_id`) | 次に実行する、または再開するジョブのID |
| 完了 | 全ジョブが終わった状態(`completed = true`)。完了のステートは、読み込みでは「空」とみなす |
| 空のステート | ステートが無いのと同じ扱い(ファイルがない、長さ 0、完了)。読み込みの戻り値は `None` |
| 耐久性(durable / fast) | ステートの書き込みで、内容の同期まで行うか(`durable`)、省くか(`fast`)。型は `Durability` |
| 実行方式 | `run`(1 ジョブ。`"individual"`)と `pipeline`(全ジョブ。`"pipeline"`)。型は `RunMode` |
| at-least-once | ジョブは、少なくとも 1 回は実行される。結果の記録の前に落ちると、再実行されうる |
| 中断の合図(`Interrupt`) | シグナルを受けたことと、最初の番号を、シグナルのハンドラと待つ側(`run_job`、`wait_retry`、`cli` の流れ)のあいだで共有する値(`trigger`・`number()`)。ハンドラは旗を立てるだけ |
| 起動失敗 | 子プロセスを作れなかったこと(終了コード 127)。終了の状態を取得できない場合も、安全側でこれに含める |
| プラットフォーム層 | OS 差を吸収する唯一の場所(`src/conductor/runner/platform.py`。`platform_for`、`detect_platform`、`exit_by_interrupt`)。`sys.platform` を読むのはここだけ |
| 木(`TreeNode`) | YAML・JSON の読み取り結果の共通の木。`NullNode`、`BoolNode`、`IntNode`、`HugeIntNode`(範囲外の整数)、`FloatNode`、`StringNode`、`SeqNode`、`MapNode` の 8 種。マップは `entries`(`tuple`)で持つ |
| マップ・リスト | キーと値の対応・値の並び(YAML のブロック/フロー、JSON のオブジェクト/配列) |
| 段階(Intake、Parse、Resolve、VersionCheck、Walk) | 定義の読み込みの処理の段階。前の段階で失敗したら、そこで打ち切る(03 章 2.2 節) |
| 種類(`kind`)・理由(`reason`) | エラーの識別子(`syntax`、`typeMismatch` など)と、固定の日本語の原因の文 |
| 案内 | ステートの誤りに付ける、`conductor reset` で回復できるという文 |
| `Deps` | `src/conductor/cli/` の差し替え口。環境(`Env`。`lookup_env`・`stdout_is_terminal`・`platform`)、時刻、標準出力・標準エラー(`Sink`)、定義、ステート、変換、プロセス、シグナルの `Protocol` の束 |
| `Reporter` | `report` が持つ書き込みの窓口(`output`・`error`・`flush`)。標準出力の書き込みの失敗を `OutputError` にし、標準エラーの失敗は握りつぶす |
| 公開窓口(`__init__.py`) | 各パッケージの公開の名前(`__all__`)を出す場所。他のパッケージからの import は、必ず相手の `__init__.py` を通す |
| flush | 標準出力・標準エラーの書き込みの完了(`flush()`)。書くたびに行い、終了の前にも行う。失敗したストリームの記述子を `os.devnull` に差し替えて、終了コードが 120 になるのを防ぐ(08 章 1.3 節) |
| TJ(テストジョブ) | 受け入れテスト用のスクリプト `tests/jobs/tj.py`。配布しない |
| AC | 受け入れテストの項目(`AC-xxx`)。11 章 4 章 |

## 6. この版が採った書式・実行環境の選択

この版が採った、書式・名前・文言・実行環境の選択を、一覧にする。

| 項目 | 内容 | 章 |
|------|------|----|
| 定義ファイルの書式 | YAML(YAML 1.2 のコアスキーマ)と JSON(RFC 8259)。書式は拡張子で決める(`.yaml`・`.yml` は YAML、`.json` は JSON。ASCII の大文字小文字を区別しない。判定は自前で、`os.path.splitext` を使わない)。それ以外は `unsupportedFormat`(64) | 03 |
| 既定のファイル名 | `conductor.yaml`(`--config` で指す) | 03、04 |
| 定義のキー名 | `format_version`、`pipeline`(`name`、`description`、`run_id_env`、`remove_on_complete`)、`job`(`id`、`command`、`warning_exit_codes`、`success_exit_codes`、`transitions`、`retry`) | 03 |
| YAML の部分集合 | アンカー・エイリアス・マージキーは展開して受理。タグ(標準のタグも)、複数の文書、文字列でないキー、重複したキー、タブのインデントは拒否。`yes`・`no`・`on`・`off`・`y`・`n` は文字列。スカラーの型は自前で決める | 03 |
| YAML ライブラリ | `PyYAML`(6.0.3 で確認。版は実装時に確認して固定)。`yaml.parse(text, Loader=yaml.BaseLoader)` のイベントだけを使い、位置は `mark.index` から自前で数える。C の拡張(`CBaseLoader`)、`yaml.load` 系、`yaml.compose` は使わない。重複キーの検査・アンカーの展開・サロゲートや制御文字の拒否・深さの打ち切り(開いているコレクション 1,000)は自前 | 03、11 |
| JSON の読み取り | 自前の字句・構文解析 `read_json`(定義・変換・ステートの 3 つが共用。標準の `json.loads` は製品のコードで使わない)。整数は字句から `IntNode`・`HugeIntNode`。ステートだけ、読む前に BOM を拒否する | 03、06 |
| 読み込みの上限 | ファイル 1 MiB、要素数 100,000、文字の大きさ 16 MiB、入れ子の深さ 32 段(根のマップを数えず、33 段目を拒否) | 03 |
| 空白の判定 | Unicode の White_Space(U+0009〜U+000D、U+0020、U+0085、U+00A0、U+1680、U+2000〜U+200A、U+2028、U+2029、U+202F、U+205F、U+3000)。`str.strip()`・`str.isspace()` は使わない(U+001C〜U+001F も空白として扱うため) | 03、07 |
| 定義のエラーの種類 | 31 種(`unsupportedFormat` を含む)。位置は行(と順序づけのための列)で出し、誤りは位置の順に 1 行ずつ。理由は常に日本語の固定文で、入力の値を含めない | 03、08、10 |
| ステートファイル | JSON(UTF-8、BOM なし)。名前は `<定義ファイル名>.state`。キーは固定の順序で書き、インデントは空白 2 つ、末尾は LF 1 つ。U+2028・U+2029 は ` `・` ` のエスケープにする。読み込みは 14 段階の検査で、キーの重複・BOM・最上位がオブジェクトでない値を拒否する。書き込みは、一時ファイル(`tempfile` を使わず、`os.open` の `O_EXCL`)+ `os.fsync` + `os.replace` | 06 |
| ステートの理由文 | 全 23 件。`JSON として読めません`、`キーが重複しています`、`内容を JSON にできません` を含む | 06、10 |
| `convert` | YAML と JSON の相互変換(`--to yaml\|json`)。入力の形式は `--to` の反対とみなし、拡張子は見ない。出力のキーの順は入力の記述順、JSON はインデント 2 スペース・末尾 LF。`null` は表せる。書き出しは自前(`json.dumps`・`yaml.dump` を使わない)。YAML の plain の条件に、YAML 1.1 の語(`y`・`n`・`yes`・`no`・`on`・`off` とその大文字の形)を含め、1.1 のローダーでも誤読されないようにする | 09 |
| 変換のエラー | 6 種(`fileMissing`、`unreadable`、`tooLarge`、`syntax`、`unrepresentable`、`rootNotMap`)。終了コード 67。入力は `O_NONBLOCK` つきで開く(FIFO でブロックしない) | 09、10 |
| `retryNotTable` の文言 | `retry must be a map` / `retry はマップで書きます` | 08 |
| 使い方(`--help`) | `convert <file> --to <yaml\|json>`、`--config` の既定は `conductor.yaml` | 08 |
| 引数の解析 | `argparse` を使わず、自前の字句・検査の 2 段階(誤りの優先順位、値を取るオプションの規則、前方一致を受け付けない点、誤りの終了コードが 2 になる点が合わないため。確認した範囲は 04 章 1.1 節) | 04 |
| 非 UTF-8 の引数 | OS から `surrogateescape` で `str` に入る(不正なバイトは U+DC80〜U+DCFF の孤立サロゲート)。表示は `readable()` で置き換え、エスケープする。非 UTF-8 を理由にした `invalid value` は、コマンド名・ジョブID・`--run-id`・`--to` の値 | 04、08、10 |
| 整数 | 64 ビット符号なしの飽和演算(リトライの待ち時間)は `min(x, 2**64 - 1)`。`bool` を整数にしない(`IntNode` と `BoolNode` を別の型にする。`type(x) is int`)。`1.0`・`nan`・`inf` は整数でない。巨大な字句で `int()` を呼ばない | 03、05 |
| 終了コード | 0、1〜255(ジョブの値)、64(定義)、65(ステート)、66(拒否)、67(変換)、68(使い方)、70(内部)、127(起動失敗)、128+番号(シグナル) | 10 |
| 想定外の内部失敗 | `sys.excepthook` と、入口 `main()` の `try/except BaseException`(`SystemExit` と `KeyboardInterrupt` を除く)で、`conductor: error: internal error` を `os.write(2, …)` で 1 行出して 70(`sys.excepthook` の中は `os._exit(70)`) | 07、10 |
| `KeyboardInterrupt` | 入口で捕まえ、何も出さずに終わる。UNIX 系は `SIG_DFL` に戻して自分に SIGINT を送りシグナル終了(シェルから見て 130)、Windows は 130 | 07、10 |
| 標準出力・終了 | 起動時に `reconfigure(encoding="utf-8", …, newline="\n")`。書き込みは 1 回の `write` のあとに `flush`。標準出力に書けない失敗は、`watch` が 65、`convert` が 67、`--help`・`--version` は無視して 0。失敗したストリームは、`os.dup2` で記述子を `os.devnull` に差し替えて、終了時の flush の失敗による終了コード 120 を防ぐ。`sys.exit` は入口の 1 か所 | 08、10 |
| 同期モデル | 同期(スレッドなし)。`asyncio`・`threading`・`multiprocessing` を使わない。ジョブ・出力・ステートの書き込みは直列。シグナルはメインスレッドで `signal.signal` のハンドラが旗を立てるだけ | 02、07 |
| シグナル | UNIX 系は SIGHUP・SIGINT・SIGQUIT・SIGTERM を受け、子プロセスの pid にだけ `Popen.send_signal` で 1 回転送(`start_new_session=True`)。Windows は Ctrl+C(SIGINT)と Ctrl+Break(SIGBREAK)を受け、転送しない(OS が配る)。中断の合図は自前の `Interrupt`。終了コードは 128+番号 | 07 |
| 子プロセス | `subprocess.Popen`(`shell=False`、`stdin`・`stdout`・`stderr` は `None` のまま引き継ぐ)。待機は `poll()` と短い `time.sleep`(1〜10 ms)。Windows の `.cmd` / `.bat` は、禁止文字を拒否したうえで、`cmd.exe /d /s /c` を `str` のコマンドラインで直接起動する(`shell=False` のまま) | 07 |
| 待機 | リトライの待機は、50 ms 以下の `time.sleep` の繰り返しと、`time.monotonic_ns` の期限の確認。合図から 1 秒以内に打ち切る(PEP 475 により、長い 1 回の `sleep` は残りを眠り直すため) | 05、07 |
| ソースの構成 | `src/conductor/{definition,convert,flow,state,runner,report,cli}/`、`runid.py`、`version.py`、`__main__.py`、`tests/`(`tests/support/`、`tests/jobs/`、`tests/inspect/`)。各パッケージの `__init__.py` が公開窓口。配布名・import 名とも `conductor`(置き換えてよい) | 02 |
| 型とデータ | `dataclass(frozen=True, slots=True)`、`Literal`、`typing.Protocol`、`match` と `assert_never`。誤りの種類は `Literal` の和(`UsageKind` だけ `enum.Enum`)。`mypy --strict`、`Any` を使わない | 02、10 |
| ビルド・配布 | `pyproject.toml`(PEP 621)、`src/` レイアウト、ビルドバックエンドは `hatchling` が採用案(実装時に確認)、PyPA の `build` で sdist と wheel。コンソールスクリプト `conductor = conductor.__main__:main` と `python -m conductor` | 02、11 |
| 品質ゲート | `ruff check`、`ruff format --check`、`mypy --strict`、依存の向きと循環の検査(`tests/inspect/test_imports.py`)、`pytest`(3 OS)、カバレッジ 80% 以上(Linux)、`pip-audit`(定期。警告のみ)、起動時間の計測(目安)、最小の Python と最新の Python でのテスト、`requires-python` と CI の版の一致の検査 | 11 |
| リリース | `v*` のタグ(`main` 上で、`version.py` および `pyproject.toml` の版と一致)で、sdist と wheel を作り、`SHA256SUMS` を付けて、下書きのリリースに添付する。PyPI への公開は人が手で行う(ワークフローは行わない)。単一の実行ファイルの配布は任意 | 11 |

## 7. 表記のきまり

- 章への参照は、`03 章 6.3 節` のように書く。ほかの章へのリンクは、このディレクトリのファイルだけである。
- コードブロックの Python のコードは、型とシグネチャの骨格(実装ではない)である。
- メッセージの文言は、`LANG=C`(英語)と日本語の両方を、各章の表に原文のまま書いてある。理由の文(定義・ステート・変換)は、ロケールに関わらず日本語の固定文である。
- 確認できなかった点は「未確認」と書き、[12](12-rebuild-guide.md) 3 章にまとめる。
