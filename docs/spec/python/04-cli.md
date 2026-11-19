# 04. コマンドライン

この章は、Python で実装する実行ファイル `conductor`(版 0.8.0)のコマンドラインを規定する。引数の解釈は、標準ライブラリの `argparse` を使わず、字句の段階(4.1)と検査の段階(4.2)を**自前の手順**で実装する。メッセージの全文は [08-output-and-i18n.md](08-output-and-i18n.md)、終了コードとエラー型は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md)、各コマンドの内部の流れは [05-execution-flow.md](05-execution-flow.md)、定義ファイルの書式は [03-definition-format.md](03-definition-format.md)、変換は [09-converter.md](09-converter.md)、シグナルと子プロセスは [07-process-and-signals.md](07-process-and-signals.md) を参照する。

## 1. 全体像

- 入口は 2 つ: コンソールスクリプト `conductor`(`conductor.__main__:main`)と、`python -m conductor`。どちらも引数は `sys.argv[1:]`(スクリプトまたはモジュールのパスを除いた文字列の並び)である。`python -m conductor` では、モジュール名より後ろの引数は、`-h`・`-V`・`-X`・`-c`・`-m`・`--` のような Python 自身のオプションと同じ綴りでも、すべてスクリプトに渡る(確認済み: Python 3.15.2 と 3.9.6、`python -m conductor --help -V --version -- -v -X dev -c x '' - -m` で、`sys.argv[1:]` が与えた並びのまま入った。`--` も除かれずに入る)。
- 引数はシェルを介さず、`conductor` 自身は展開(ワイルドカード、変数、チルダなど)も解釈もしない。
- サブコマンドは 6 つだけ: `run`, `pipeline`, `watch`, `reset`, `validate`, `convert`。
- 特別なオプション(`-h`/`--help`, `-V`/`--version`)は、コマンドの解釈より先に処理する(4.2 の 1)。
- 使い方の誤りのときは、標準エラーに「誤りの 1 行 + 空行 + 使い方全文」を書き、終了コード 68 で終わる。ジョブは実行しない。
- 既定では、`conductor` 自身は何も出力しない(ジョブ本体の出力がそのまま流れる)。エラーのとき、または `-v` のときだけ標準エラーに書く。例外として、`watch`・`--help`・`--version`・`convert` は結果を標準出力に書く。
- プロセスの終了コードを決める箇所は、`conductor/__main__.py` の `main()` の 1 か所だけにする。`conductor/cli/` の `run(argv, deps)` は終了コード(`int`)を戻り値として返し、`main()` がその値を、出力を完了させたうえで `sys.exit` に渡す(10 章 4.1、08 章 1.3)。

### 1.1 `argparse` を使わない理由

標準ライブラリの `argparse` は、この仕様の文法に合わないため使わず、自前で実装する。次の挙動は、Python 3.9.6・3.10.7(macOS)と 3.14.10・3.15.2(macOS)の実機で、`ArgumentParser(prog="conductor")` に `-v/--verbose`(`action="count"`)、`--config`、`--run-id`、`--all`(`store_true`)、`-V/--version`(`store_true`)、位置引数 `pos`(`nargs="*"`)を登録して確かめた。3.10.7 は 3.9.6 と、3.14.10 は 3.15.2 と、下の表の範囲で結果が同じだった(版による差は表に書く)。3.11〜3.13 は実行していない(未確認)。

| 仕様が要求すること | `argparse` で確認できた挙動 | 合わない点 |
|--------------------|------------------------------|------------|
| 長いオプションの前方一致による省略は受け付けない(`--verb` は知らないオプション) | 既定(`allow_abbrev=True`)では `['--verb','x']` が `--verbose` として受理され、`['--con','c.yaml']` が `--config` として受理された。`allow_abbrev=False` にすると `unrecognized arguments` の誤りになった | 既定は受理する。`allow_abbrev=False` にすれば合うが、下の別の点が合わない |
| 値を取る長いオプションは、次の引数を中身にかかわらず値にする(`--config --help` は `--help` が値) | `['--config','--help']`、`['--config','--version']`、`['--config','-x']`、`['--config','-v']`、`['--config','--']` はすべて `expected one argument`(終了コード 2)。`--config=--help` と書いたときだけ値になった | 「`-` で始まる次の引数」を値にしない。仕様の文法と逆 |
| `-h`/`--help`、`-V`/`--version` は、ほかの誤りより優先し、`=値` が付いても help/version(`--help=x` は help) | `['--help=x']`、`['--version=x']`、`['-V=1']` は `ignored explicit argument`(終了コード 2)。一方、`['--bogus','--help']`・`['-vh']` は `--help` の動作が走って終了コード 0 になった(組み込みの help は、見つけた時点で出力して終わる) | `=値` 付きを誤りにする。また、組み込みの help は「その時点までに見つけた誤りに関係なく」ではなく「見つけた時点」で動く(`--help` の前に別の誤りがあれば、その誤りが先に終了することがある。仕様は `-h`/`-V` を最優先にする)。`--version` は `store_true` なので出力と終了は自前になる |
| `-v=1` は `invalid value: '-v'`(`=` 以降を、`-v` に付いた値の取り残しとして扱う) | `['-v=1']` は `argument -v/--verbose: ignored explicit argument '1'`(終了コード 2) | 誤りの種類と文言がこの仕様と違う(対象は `-v` で近いが、文言と終了コードが違う) |
| 短縮形の連結で、知らない文字は `unknown option: '-x'`(`-vx run a`) | 3.9.6 と 3.10.7: `['-vx']` は `argument -v/--verbose: ignored explicit argument 'x'`。3.14.10 と 3.15.2: `unrecognized arguments: -x` | 版によって誤りの文言が違い、どちらもこの仕様の文言と違う |
| 値を取らないオプションに `=値`(`--verbose=x`、`--all=1`)は `invalid value`(対象 `--verbose`・`--all`) | `ignored explicit argument`(終了コード 2)。対象の綴りは `-v/--verbose` のような組になる | 文言が違う |
| 同じオプションの重複(`--all --all`、`-vv`、`-v --verbose`)は `invalid value` | `['--all','--all']` は誤りにならず `all=True`。`['-vv']` は `verbose=2`(`count`) | 重複を検出しない(`store_true` は上書き、`count` は加算するだけ) |
| 知らないオプションは、位置引数の個数などの検査より先でも後でもなく、4.2 の固定の順序で報告する | `parse_args` は、知らないオプションを `unrecognized arguments` としてまとめて最後に報告し、終了する(`parse_known_args` は、知らない引数を返すだけで誤りにしない) | 報告の順序を制御できない |
| `--=x` は `unknown option: '--'` | 既定では `ambiguous option: --=x could match --help, --verbose, ...`(3.9.6 で確認)。`allow_abbrev=False` では `unrecognized arguments: --=x` | 文言が違い、前方一致の機構が絡む |
| 使い方の誤りの終了コードは 68 | すべて 2。`argparse.ArgumentParser.error()` が、使い方(`usage:`)を標準エラーに出して `SystemExit(2)` を投げる | 上書き(`error()` のオーバーライド)で変えられるが、下の別の点で結局、字句の規則を自前でやり直すことになる |
| `exit_on_error=False` で、例外として受け取って自前で扱う | 3.9.6 では `['--config']`・`['-v=1']`・`['--config','--help']`・`['--help=x']` は `argparse.ArgumentError` として届いたが、`['--bogus']`(`unrecognized arguments`)は `exit_on_error=False` でも `SystemExit(2)` で終わった。3.15.2 では `['--bogus']` も `ArgumentError` になった | 版によって、例外になる誤りの範囲が違う(`unrecognized arguments` は 3.9.6 では例外にならない)。誤りの優先順位を、版を超えて安定させられない |
| サブコマンド名(`run` など)の前方一致は受け付けない | `add_subparsers` に `run`・`reset`・`validate` を登録したとき、`ru`・`res`・`val` は `invalid choice`(終了コード 2)で拒否された(3.15.2 では `maybe you meant 'run'?` の提案が付く。文言が版で違う) | サブコマンド名の前方一致は拒否する点は合う。ただし、誤りの文言が版で違い、終了コードが 2 |
| `--` のあとはすべて位置引数 | `['--','-v']` は位置引数 `-v`。`['run','--','a','--']` は `pos=['run','a','--']`(最初の `--` だけが除かれる)で、この仕様と一致した | 合う(サブコマンドの解析器を使うと挙動が変わりうるが、確認していない。未確認) |
| 単独の `-` は位置引数 | `['-']` は位置引数 `-` | 合う |
| 値を空文字にできる(`--run-id=`、`--run-id ''`) | どちらも `run_id=''` | 合う |

要するに、(1) 誤りの優先順位(`-h`/`-V` が最優先、字句の誤りが先、知らないオプションが次、…)を実装できない(`argparse` は見つけた順に動き、誤りの種類ごとの報告順を持たない)。(2) 値を取るオプションが次の引数を無条件に取る、`=` の位置の扱い、重複の検査が、仕様と違う。(3) 誤りの文言と終了コードが、この仕様の形(08 章 4.2、終了コード 68)と違い、しかも版によって変わる。(4) `allow_abbrev` と `exit_on_error` は、導入された版・修正された版が違い、サポートする版の全体で同じ挙動を保証しにくい。したがって、`conductor/cli/` に字句の段階の関数と検査の段階の関数を置き、`sys.argv` の文字列だけを入力に、標準ライブラリの引数解析器は使わずに(文字列の操作だけで)実装する。`optparse`・`getopt` も同様の理由で使わない(`getopt` は値を取るオプションの次の引数を無条件に取るが、`=` の扱い、重複、優先順位は同じく自前になる。実機では確認していない。未確認)。

`argparse` の挙動は Python の版で変わりうる。上の確認は、3.9.6・3.10.7・3.14.10・3.15.2 だけで行った。ただし、結論(自前で実装する)は、版によらない仕様側の要求(`-h`/`-V` の優先、`--config --help` が値、誤りの優先順位)から導いており、版が変わっても変わらない。

## 2. 構文

```text
conductor run [-v] [--config <path>] [--run-id <id>] [--remove-on-complete] <job-id>
conductor pipeline [-v] [--config <path>] [--run-id <id>] [--remove-on-complete] <pipeline-name>
conductor watch [--config <path>] [--run-id <id>]
conductor reset [--config <path>] [--run-id <id> | --all]
conductor validate [-v] [--config <path>] [<pipeline-name>]
conductor convert <file> --to <yaml|json>
conductor --help | --version
```

## 3. オプションと位置引数

### 3.1 オプションの一覧と、コマンドごとの可否

| オプション | 短縮形 | 値 | run | pipeline | watch | reset | validate | convert |
|------------|--------|----|-----|----------|-------|-------|----------|---------|
| `--verbose` | `-v` | なし | 可 | 可 | 不可 | 不可 | 可 | 不可 |
| `--config` | なし | パス(必須) | 可 | 可 | 可 | 可 | 可 | 不可 |
| `--run-id` | なし | 文字列(空も可) | 可 | 可 | 可 | 可 | 不可 | 不可 |
| `--remove-on-complete` | なし | なし | 可 | 可 | 不可 | 不可 | 不可 | 不可 |
| `--all` | なし | なし | 不可 | 不可 | 不可 | 可 | 不可 | 不可 |
| `--to` | なし | `yaml` か `json`(必須) | 不可 | 不可 | 不可 | 不可 | 不可 | 必須 |
| `--help` | `-h` | なし | どのコマンドでも(コマンド名がなくても)受け付ける |
| `--version` | `-V` | なし | どのコマンドでも(コマンド名がなくても)受け付ける |

- 長いオプション名・短縮形は、大文字小文字を区別する。長いオプションの前方一致による省略(`--verb` など)は受け付けない(知らないオプション)。
- 上表以外の短縮形は存在しない(`--config` に `-c` はない)。

### 3.2 位置引数

「コマンド名」は、最初に現れた位置引数である。位置引数の個数は、コマンド名を除いて数える。

| コマンド | 位置引数 | 個数 |
|----------|----------|------|
| `run` | ジョブID | ちょうど 1 |
| `pipeline` | パイプライン名 | ちょうど 1 |
| `watch` | なし | 0(渡すと `unexpected argument`) |
| `reset` | なし | 0(渡すと `unexpected argument`) |
| `validate` | パイプライン名 | 0 または 1(省略時は名前を照合しない) |
| `convert` | 入力ファイル | ちょうど 1(`--to` は別に必須) |

オプションは、コマンド名の前にも後ろにも置ける(例: `conductor --config c.yaml -v run x` は有効)。

### 3.3 各オプションの意味と既定値

| オプション | 意味 | 既定 |
|------------|------|------|
| `-v`, `--verbose` | ジョブの結果・スキップ・遷移・中断を標準エラーに出す(`validate` は成功時に 1 行)。段階は 1 つだけ(詳しさの度合いを選べない) | 出さない |
| `--config <path>` | 定義ファイルのパス。`str` のまま加工せず、そのままファイルシステムの API(`open()`、`os.stat()` など)に渡す(正規化・絶対化しない。相対パスは実行ディレクトリ(カレントディレクトリ)から解決される)。書式は拡張子で決まる(`.yaml`・`.yml` は YAML、`.json` は JSON。大文字小文字を区別しない。それ以外は拒否。[03-definition-format.md](03-definition-format.md)) | 実行ディレクトリ(カレントディレクトリ)の `conductor.yaml` |
| `--run-id <id>` | ステートファイルを分ける run ID。空文字は「run ID なしに固定」(環境変数を見ない) | 未指定(環境変数を見る。5 章) |
| `--remove-on-complete` | 完了時にステートファイルを削除する | 指定なし(その場合は定義の `pipeline.remove_on_complete`、それも無ければ `false`) |
| `--all` | 定義のステートファイルを、run ID の有無によらずすべて消す(`reset` のみ) | なし |
| `--to <yaml\|json>` | `convert` の出力形式。大文字小文字を区別し、`yaml` と `json` 以外は `invalid value`(対象は値そのもの) | なし(必須) |

既定の定義ファイル名 `conductor.yaml` は、`.yml`・`.json` を探索しない。既定の名前でファイルがなければ、通常の「ファイルなし」(64)になる。

`--config` のパスについての Python 固有の注意: `pathlib.Path` に通すと、文字列が変わる(確認済み: 3.15.2。`Path('a//b/./c.yaml')` は `a/b/c.yaml`、`Path('./c.yaml')` は `c.yaml`、`Path('c.yaml/')` は `c.yaml`、`Path('')` は `.`)。したがって、利用者に見せる文字列(誤りの行の場所、ステートのパスの元)には、`--config` に渡された**元の `str`** を使い、`Path` は内部でパスをつなぐときの道具にとどめる(ステートのパスの作り方は [06-state-file.md](06-state-file.md))。空文字の `--config ''` は、`open('')` と同じく `FileNotFoundError`(定義ファイルなし、64)になる(確認済み: 3.15.2)。

## 4. 引数の解釈

解釈は、(A)字句の段階、(B)コマンドの組み立てと検査、の 2 段で行う。(A)は引数を先頭から 1 つずつ見て、`special`(最初に現れた `-h`/`-V` の種別)、`lex_error`(最初の字句の誤り)、`tokens`(オプションと位置引数の並び)を作る。(B)は `tokens` を検査する。

### 4.0 内部の表現(骨格)

型の名前と分け方は目安であり、4.1・4.2 の挙動を満たせば変えてよい。

```python
from dataclasses import dataclass
from typing import Literal, Sequence

SpecialKind = Literal["none", "help", "version"]  # 最初に現れた方

TokenKind = Literal[
    "positional", "verbose", "config", "run_id", "to",
    "all", "remove_on_complete", "unknown",
]

@dataclass(frozen=True, slots=True)
class Token:
    kind: TokenKind
    value: str  # 位置引数・値を取るオプションの値。unknown では綴り(-x, --bogus)

@dataclass(frozen=True, slots=True)
class LexResult:
    special: SpecialKind
    lex_error: "UsageError | None"  # 最初の字句の誤り(10 章 8.3)
    tokens: tuple[Token, ...]

def lex(args: Sequence[str]) -> LexResult: ...
```

引数は Python の `str`(コードポイントの列)のまま扱う。`s[i]`・`len(s)`・`for ch in s` は、いずれもコードポイント単位なので、4.1 の連結の分解には `for ch in s[1:]` と書いてよい(サロゲートペアの問題は Python の `str` には無い)。ただし、「見た目の 1 文字」(書記素クラスタ。結合文字、国旗など)とは一致しない。連結の分解は、コードポイント 1 つずつで行う(`-é`(NFD で `e` + U+0301)は、`-e` と、知らない文字 U+0301 の 2 文字として分解される。実機での確認は不要で、`str` の定義による)。`str.split()` の引数なしの形、`str.strip()`、`str.isspace()`、`str.isalnum()`、`str.isdigit()` は、Unicode の空白・文字・数字に広く一致するため、字句の規則に使わない(`==`・`startswith`・`partition("=")` のような、文字列そのものの比較に使う。確認済み: 3.15.2、`"Ａ".isalnum()` は真、`"a　".rstrip()` は全角空白まで削る)。

### 4.1 字句の段階(A)の手順

`after_dash_dash = False` として、引数 `a` を先頭から順に処理する。

1. `after_dash_dash` が真なら、`a` は位置引数(`-` で始まっていても、`-v` や `--help` であっても位置引数)。
2. `a` が `--` ちょうどなら、`after_dash_dash = True` にする(`--` 自体は位置引数にならない)。2 個目以降の `--` は、`after_dash_dash` が真なので位置引数になる。
3. `a` が `--` で始まり(`--` より長い)、長いオプションとして扱う。`=` が含まれていれば、最初の `=` の前を名前(`--` を除く)、後ろを値(空でもよい)とする(`a[2:].partition("=")` で得られる)。含まれていなければ名前は `a` から `--` を除いた全体で、値なし。
   - 名前が `help`: `special` が未設定なら help を設定する。`=値` が付いていても、help が優先されるので誤りにならない(例: `--help=x` は help)。
   - 名前が `version`: 同様に `special` が未設定なら version を設定する(`--version=x` も version)。
   - 名前が `verbose`, `all`, `remove-on-complete`(値を取らない): `=値` が付いていれば(値が空でも)`lex_error`(最初の 1 つだけ保持)に `invalid value`(対象はそのオプションの正規の綴り、例 `--verbose`)を設定する。付いていなければ、そのオプションを `tokens` に追加する。
   - 名前が `config`, `run-id`, `to`(値を取る): `=値` があれば、それ(空文字も有効)を値として `tokens` に追加する。なければ、次の引数を、中身にかかわらず(`-` で始まっていても、`--` や `--help` であっても)値として取る(その引数は位置引数にもオプションにもならない)。次の引数が存在しないときは、`lex_error` に `option requires a value`(対象は `--config` / `--run-id` / `--to`)を設定し、字句の段階を打ち切る(以降の引数は見ない)。
   - それ以外の名前(空の名前を含む): 「知らないオプション `--<名前>`」を `tokens` に追加する。`=値` が付いていたときは、`lex_error` にも `unknown option`(対象 `--<名前>`、値は含めない)を設定する。名前が空のときの綴りは `--`(例: `--=x` は `unknown option: '--'`)。`---` は名前が `-` で、綴りは `---`。
4. `a` が `-` で始まり、長さが 2 以上(コードポイント数)で、`--` で始まらない: 短縮形の連結として、先頭の `-` を除く文字を、先頭から 1 文字(1 コードポイント)ずつ処理する。
   - `h`: help を設定(上と同じ。`special` が未設定のときだけ)。`V`: version を設定(同様)。`v`: `-v` を `tokens` に追加する。
   - ほかの文字 `c`: 「知らないオプション `-<c>`」を `tokens` に追加する(`-é` は `unknown option: '-é'`。孤立サロゲート(4.4)の文字も 1 文字として `-<c>` の綴りにする)。
   - 2 文字目以降の位置で、直前の 1 文字の直後に `=` が続いているとき、その `=` 以降の残りの文字列は**まとめて捨て**(残りの文字は解釈しない)、`lex_error` を設定する。直前の文字が `v` なら `invalid value`(対象 `-v`)、知らない文字 `c` なら `unknown option`(対象 `-<c>`)。直前の文字が `h`・`V` のときは `special` が優先されるので誤りにならない。連結の先頭の文字が `=` のとき(`-=x`)は、区切りではなく、`=` という文字そのものが知らないオプション(`unknown option: '-='`)。
   - 短縮形は値を取らない(値を取るのは長いオプションだけ)。`-vv` は `-v` が 2 回。`-vh` は `-v` と help(help が勝つ)。
5. それ以外(`-` ちょうど 1 文字、`-` で始まらない文字列、空文字)は位置引数。単独の `-` も位置引数である。

`lex_error` は最初の 1 つだけを保持する(後の字句の誤りは捨てる)。

補足(挙動の例。この文法の規則から導かれるもの。この章の文法は、実機の `conductor` で確認済みの文法と同じ):

- `--help=x`・`-V=1`・`--version=x` は、それぞれ help / version / version。`-vx run a` は `unknown option: '-x'`、`run -vh` は help、`-vV` は version、`--config --help run a` は `--help` という名前のパスを定義ファイルとして扱う(help ではない。拡張子が `.yaml`・`.yml`・`.json` のどれでもないので、定義の誤り `unsupportedFormat` として 64)。
- `--config --help`(ほかの引数なし)は `--help` を値に取り、位置引数が 0 個なので `missing argument`(68)。`run a --config --version` も `--version` を値(パス)として扱う。
- `--help --config` は、`--help` が先に解釈されるので、値の欠落(`--config`)より前に help(0)になる。
- `run a -- b` の `--` は、除かれて位置引数 `b` だけが残る。`run -- a --` は 2 個目の `--` が位置引数になり `unexpected argument: '--'`。`run -- -v` は、ジョブID `-v`。`run -` は、ジョブID `-`。
- `-v=1 run a` と `run -v=1h x` は、どちらも `invalid value: '-v'`(`=` 以降の `h` は解釈されず、help にならない)。`-v=` も `invalid value: '-v'`。`-vh=1` は help。`-x=1h` と `-vx=1` は `unknown option: '-x'`。
- `--=x`・`--=` は `unknown option: '--'`、`---` は `unknown option: '---'`、`--verbose=`(値が空)は `invalid value: '--verbose'`。
- `reset --all=1` は `invalid value: '--all'`。`run j --run-id`・`--to`(値なし)は `option requires a value`。

### 4.2 検査の順序(誤りの優先順位)

字句の段階のあと、次の順に検査する。最初に当てはまったもので終わる(それ以降は行わない)。

1. `special` が設定されていれば(`--` より前に、オプションとして解釈された `-h/--help` か `-V/--version` が 1 つでもあれば)、ほかの誤りに関係なく、先に現れた方を処理して終了コード 0。値を取るオプション(`--config`、`--run-id`、`--to`)の直後の引数は、`-h` や `--help` であっても値として取られ(4.1 の 3)、オプションとして解釈されないので、`special` にならない(`--config --help run a` は help ではない。`-h --config --version` は、`-h` が先に現れるので help で、`--version` は `--config` の値になる)。両方あるとき、先に現れた方が勝つ(`-V -h` は version、`-h -V` は help)。字句の段階が値の欠落で打ち切られた場合、それ以前に現れた `-h`/`-V` は有効、以降は存在しない扱い。
2. `lex_error` があれば、それを誤りとする。
3. 「知らないオプション」が `tokens` にあれば、最初のものを `unknown option`(対象はその綴り: `-x`, `--bogus`)とする。
4. 位置引数が 0 個: `missing argument`(対象なし)。例: 引数なし、`-v` だけ、`--` だけ。
5. 先頭の位置引数(コマンド名)が非 UTF-8 の引数(4.4 の判定)なら `invalid value`(対象は、4.4 の規則で読める形にした文字列)。6 コマンドのいずれでもなければ `unknown command`(対象はその語。空文字も同じ)。
6. そのコマンドで使えないオプション: `option not allowed for this command`。検査の順は `-v`, `--config`, `--to`, `--run-id`, `--all`, `--remove-on-complete`。対象はそのオプション名(`-v` の綴りは `-v`)。可否は 3.1 の表による。
7. 重複: `-v`, `--config`, `--to`, `--run-id`, `--all`, `--remove-on-complete` のどれかが 2 回以上 → `invalid value`(対象はそのオプション名。`-vv` は対象 `-v`)。検査の順は挙げた順。`-v` の重複は、`-v` と `--verbose` の合計で数える。
8. `--all` と `--run-id` の併用 → `invalid value`(対象 `--all`)。
9. `--run-id` の値の検査(5.2)。値が非 UTF-8(4.4)、または空でなく規則に違反 → `invalid value`(対象 `--run-id`。値は含めない)。空文字は有効。
10. 位置引数の過不足(コマンド名を除いて数える):
    - `run`・`pipeline`・`convert` で位置引数が 0 個 → `missing argument`(対象なし)。
    - `watch`・`reset` で位置引数が 1 個以上 → `unexpected argument`(対象は最初の位置引数)。
    - コマンドが取れる個数を超える(2 個目の位置引数がある)→ `unexpected argument`(対象は 2 個目の語。`run a b` なら `b`)。
    - 位置引数(ジョブID、パイプライン名)が非 UTF-8 → `invalid value`(対象は読める形にした文字列)。`convert` の入力ファイルは、非 UTF-8 でもパスとしてそのまま通す。
11. `convert`: `--to` なし → `missing argument`(対象 `--to`)。`--to` の値が `yaml`/`json` 以外(非 UTF-8 を含む)→ `invalid value`(対象は、読める形にした値)。

誤りの表示形式は、`conductor: error: <文>: '<対象>'`(対象があるとき)、空行、使い方全文。文は [08-output-and-i18n.md](08-output-and-i18n.md) 4.2、対象は名前用エスケープ(64 文字で切る。08 章 9 章)。

### 4.3 判定表(挙動の早見表)

| 引数 | 結果 |
|------|------|
| (引数なし)、`--`、`-v` | `missing argument`(68) |
| `-- run`、`run --` | `missing argument`(68)(`--` のあとの `run` はコマンド名。ジョブIDがない) |
| `-- run a` | `run` コマンドで、ジョブID `a` |
| `--config` | `option requires a value: '--config'`(68) |
| `run a --config` | `option requires a value: '--config'`(68) |
| `--config --help run a` | `--help` というパスを定義ファイルとして読む(拡張子が合わないので、定義の誤り `unsupportedFormat` で 64) |
| `--config --help` | `--help` を値に取り、位置引数が 0 個: `missing argument`(68) |
| `--help --config` | help(0)(`--help` が先に解釈され、値の欠落は検査されない) |
| `-h --config --version` | help(0)(`--version` は `--config` の値) |
| `--help=x`、`-h=x` | help(0) |
| `-V=1`、`--version=x` | version(0) |
| `-vh x` | help(0) |
| `-vx run a` | `unknown option: '-x'`(68) |
| `-v=1 run a` | `invalid value: '-v'`(68) |
| `run --verbose=x a` | `invalid value: '--verbose'`(68) |
| `--bogus=x` | `unknown option: '--bogus'`(68) |
| `--bogus --help` | help(0)(`special` が最優先) |
| `run a -x` | `unknown option: '-x'`(68) |
| `run a -- b` | `unexpected argument: 'b'`(68) |
| `--run-id= watch` | 有効(run ID なしに固定)。ステートなしなら `no state (nothing has run yet)`、0 |
| `run -- -v` | ジョブID `-v` |
| `run -` | ジョブID `-` |
| `run --verb a` | `unknown option: '--verb'`(68)(前方一致の省略は受け付けない) |
| `run a --to yaml` | `option not allowed for this command: '--to'`(68) |
| `convert --to yaml --to json f` | `invalid value: '--to'`(68) |
| `convert f --to xml` | `invalid value: 'xml'`(68)(`--to` の値は `yaml`・`json` だけ。大文字小文字も区別する) |
| `convert f` | `missing argument: '--to'`(68) |
| `reset --all --run-id x` | `invalid value: '--all'`(68) |
| `run -vv a` | `invalid value: '-v'`(68) |

### 4.4 `sys.argv` の文字列と、非 UTF-8 の引数

#### 4.4.1 Python が引数を `str` にする方法(確認した範囲)

POSIX(macOS で確認)では、Python はプロセスの引数のバイト列を、ファイルシステムのエンコーディング(`sys.getfilesystemencoding()`)と `surrogateescape` の誤りの扱い(`sys.getfilesystemencodeerrors()`)で復号して `sys.argv` に入れる。正しくない UTF-8 のバイトは、U+DC80〜U+DCFF の孤立サロゲート(バイト `0x80`〜`0xFF` に対し U+DC80〜U+DCFF。バイト 1 つにつき 1 コードポイント)になり、元のバイト列を取り戻せる。次の結果を、Python 3.9.6・3.15.2(macOS)で確かめた(`bash` の `$'...'` で作ったバイト列を引数に渡し、各文字のコードポイントを出した)。

| 入力(バイト列) | `sys.argv` の文字列(コードポイント) |
|------------------|--------------------------------------|
| `a` + `FF` + `b` | `a`, U+DCFF, `b` |
| `E3 81`(3 バイト列の途中で切れている) | U+DCE3, U+DC81(2 コードポイント) |
| `あ`(`E3 81 82`) | U+3042 |

- `sys.getfilesystemencoding()` は `utf-8`、`sys.getfilesystemencodeerrors()` は `surrogateescape`(確認済み: 3.9.6。`LC_ALL=C` で起動しても、UTF-8 モード(`sys.flags.utf8_mode` が 1)になり、同じ値だった)。環境変数 `os.environ` の値も同じ方式で復号される(確認済み: `X=$'a\xff'` で `os.environ['X']` が `a` + U+DCFF)。
- `s.encode("utf-8", "surrogateescape")` は、U+DC80〜U+DCFF の孤立サロゲートを元のバイトに戻す。`s.encode("utf-8")`(誤りの扱いが `strict`)は、孤立サロゲートがあると `UnicodeEncodeError`(`surrogates not allowed`)を投げる(確認済み)。
- `open(path)`・`os.stat(path)` などは、`str` のパスを `os.fsencode` で(`surrogateescape` で)バイト列に戻して OS に渡す(Python の文書の記述。元のバイト列を持つファイル名を実際に開けることは、確認していない: macOS のファイルシステム(APFS)は、非 UTF-8 のファイル名を作ること自体を拒否した(`illegal byte sequence`)ため、実機で確かめられなかった。未確認。Linux で確かめる)。
- Windows の `sys.argv` は、UTF-16 の引数から作られる。対になっていないサロゲートを含む引数が `str` にどう入るかは未確認(実装時に確認する)。入ってきた場合も、4.4.2 の判定(孤立サロゲートの有無)でそのまま扱える。

#### 4.4.2 この版の規定

`str` は、孤立サロゲート(U+D800〜U+DFFF のコードポイント)を持つか持たないかで、「非 UTF-8 の引数か」を区別できる。判定は `has_lone_surrogate(s) -> bool` とし、`s.encode("utf-8")` が `UnicodeEncodeError` を投げるかどうかで行う(`try`/`except` で包む。文字ごとの走査でもよい)。この版は、**非 UTF-8 の引数を区別し**、次のとおり扱う(4.2 の各段階で適用する)。

| 対象 | 扱い |
|------|------|
| コマンド名(最初の位置引数) | 孤立サロゲートを含むなら `invalid value`(4.2 の 5。対象は読める形) |
| ジョブID・パイプライン名(`validate` を含む) | 孤立サロゲートを含むなら `invalid value`(4.2 の 10。対象は読める形) |
| `--run-id` の値 | 孤立サロゲートを含むなら `invalid value`(4.2 の 9。対象は `--run-id`。値は含めない) |
| `--to` の値 | `yaml`/`json` 以外なので `invalid value`(4.2 の 11。対象は読める形) |
| `--config` の値、`convert` の入力ファイル | 孤立サロゲートを含んでいても、検査せず、`str` のままパスとして通す(`surrogateescape` により元のバイト列へ戻される。上のとおり、実際に開けることは未確認) |
| 知らないオプションの綴り(`-<c>`、`--<名前>`) | 孤立サロゲートを含んでいてもそのまま持つ。誤りの対象に出すとき、読める形にする |
| 環境変数の値(run ID、ロケール) | 孤立サロゲートを含むなら、未設定と同じ扱い(5.1、08 章 7 章) |

**読める形にする規則** `readable(s: str) -> str`(使い方の誤りの対象に、非 UTF-8 の引数を出すとき、および 08 章 9 章のエスケープの前処理で使う):

1. `s.encode("utf-8", "surrogateescape")` でバイト列にする。U+DC80〜U+DCFF 以外の孤立サロゲート(U+D800〜U+DC7F と U+DD00〜U+DFFF。Windows の引数などから入りうる)があって `UnicodeEncodeError` になるときは、先にそれらを U+FFFD に置き換えてからやり直す。
2. そのバイト列を `bytes.decode("utf-8", "replace")` で `str` に戻す。不正なバイト列は、Unicode の「最大の部分列」の規則(不正な列の、正しい列の先頭になりうる最長の部分を 1 つの U+FFFD にする)で U+FFFD に置き換わる。

確認した結果(3.9.6 と 3.15.2 で `readable` を試した。実機の `conductor` の表示と照合したのは、`a` + `FF` + `b` と `a` + `E3 81` の 2 件で、一致した。ほかの行は、`bytes.decode("utf-8", "replace")` の結果であり、実機の `conductor` との照合はしていない):

| 入力(バイト列) | `readable` の結果 |
|------------------|---------------------|
| `a` + `E3 81` | `a` + U+FFFD 1 個 |
| `E3 81 82` + `E3` | `あ` + U+FFFD 1 個 |
| `FF FE` | U+FFFD 2 個 |
| `F0 9F 98`(4 バイト列の途中) | U+FFFD 1 個 |
| `ED A0 80`(UTF-8 に符号化された孤立サロゲート。不正) | U+FFFD 3 個 |
| `C0 80`(冗長な符号化。不正) | U+FFFD 2 個 |

- ジョブID・パイプライン名: 孤立サロゲートを含む名前は、定義の中の名前と照合する前に、`invalid value`(68)で拒否される(定義の名前は、`str` として有効な文字の列であり、孤立サロゲートを含まないため、照合しても一致しない)。
- 連結の短縮形(`-` で始まる引数)に非 UTF-8 のバイトが含まれるときは、孤立サロゲート 1 つ(1 コードポイント)が、知らない文字 `c` 1 つとして扱われる(4.1 の 4)。例: `-` + `E3 81` は、U+DCE3 と U+DC81 の 2 つの知らないオプションになり、最初のものが `unknown option: '-�'`(U+DCE3 を `readable` に通すと U+FFFD 1 個)として報告される。引数全体を `readable` に通す場合(`E3 81` が U+FFFD 1 個)と、1 文字ずつの綴りに通す場合とで、置き換えの個数が変わりうる。この場合の実機の `conductor` との一致は確認していない(未確認)。
- 孤立サロゲートの引数・環境変数が標準出力・標準エラーに出る場合は、必ず 4.4.2 の `readable` と 08 章 9 章のエスケープを通した文字列にする。孤立サロゲートのまま `sys.stdout`/`sys.stderr` に書くと、エンコーディングの誤りの扱いによっては `UnicodeEncodeError` になる(08 章 1.1)。

### 4.5 実装に任せてよい点

- 内部の語・型の分け方(4.0 は目安)。
- 4.1 の挙動(値を取るオプションが次の引数を無条件に値にする、`=` の扱い、`--`、連結)と、4.2 の検査の順序は必須。
- 単独の `-` を位置引数とする挙動は必須(`run -` はジョブID `-`)。

## 5. run ID

### 5.1 決定の優先順位(`run`・`pipeline`)

1. `--run-id <値>` が指定されていれば、環境変数は見ない。値が空文字なら run ID なし。
2. 指定がなければ、環境変数を読む(`Deps` の `lookup_env`。実体は `os.environ.get(name)`)。環境変数の名前は、定義の `pipeline.run_id_env`(なければ `CONDUCTOR_RUN_ID`)。未定義(`None`)または空文字なら run ID なし。値が孤立サロゲートを含む(非 UTF-8)ときは、未定義と同じ扱い。
3. 得た値は 5.2 の規則で検査する。違反は `invalid value`(終了コード 68)。対象は、オプション由来なら `--run-id`、環境変数由来なら `run ID (environment variable)`(値は含めない)。

`run`・`pipeline` では、環境変数の名前を知るために、定義を先に読む。検査の順序は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md) 2 章を参照する。

環境変数の読み取りで注意する点: `os.environ` のキーは、POSIX では大文字小文字を区別し、Windows では区別しない(Python の文書の記述に基づく。Windows の実機では未確認。POSIX の区別は確認済み: `os.environ['Zz_Test']` を設定したとき、`os.environ.get('zz_test')` は `None`)。`os.environ.get(name)` が `None` のとき「未定義」、`""` のとき「空文字」(どちらも run ID なし)。

### 5.2 run ID の規則

- 文字は `A-Za-z0-9._-` だけ、長さは 1〜64 文字(ASCII だけなので、`len()` でも、コードポイント数でも、UTF-8 のバイト数でも同じ)、先頭が `.` でない(`..` も不可)。空文字は「run ID なし」を表し、値としては規則を満たさない(オプションの空文字だけが「なし」を意味する特別な扱い)。
- 正規表現で書く場合は `re.fullmatch(r"[A-Za-z0-9_-][A-Za-z0-9._-]{0,63}", value)`。次の Python 固有の点を守る(いずれも確認済み: 3.15.2)。
  - `re.match` と `^…$` の組では、`$` が文字列の末尾の改行 `\n` の直前にも一致するので、`"abc\n"` が通ってしまう(`rx.match("abc\n")` は真)。`re.fullmatch` を使う(`fullmatch(…, "abc\n")` は偽)。
  - `\d`・`\w` は、既定では Unicode の数字・語の文字(例: U+0663)にも一致する(`re.fullmatch(r"\d", "٣")` は真。`re.ASCII` を付けると偽)。上の形のように、`A-Za-z0-9` を文字クラスで明示し、`\d`・`\w` は使わない。
  - `str.isalnum()`・`str.isidentifier()` なども Unicode に広く一致するので、検査に使わない。
- ステートファイル名は、run ID なしで `<定義ファイル名>.state`、ありで `<定義ファイル名>.<run-id>.state`(定義ファイルと同じディレクトリ)。詳細は [06-state-file.md](06-state-file.md)。
- 定義の `run_id_env` は、`A-Za-z_` で始まり `A-Za-z0-9_` が続く 1 文字以上。違反は定義の誤り(64。[03-definition-format.md](03-definition-format.md))。
- 検査は `conductor/runid.py` に置く。理由の区別(空・長すぎ・不正な文字・先頭のドット)は内部では持ってよいが、利用者には区別を見せず、すべて `invalid value`(68)にまとめ、値を含めない。

### 5.3 `watch` / `reset` の run ID

- `--run-id` があれば、定義ファイルを読まない(環境変数は見ない)。
- なければ、定義から `pipeline.run_id_env` だけをゆるく読む。次のどれでも、既定名 `CONDUCTOR_RUN_ID` を使う: ファイルが読めない、拡張子が `.yaml`・`.yml`・`.json` のどれでもない、その書式として読めない(深さ・大きさの上限を超える場合を含む)、`pipeline` がマップでない、キーがない、文字列でない、名前の形が 5.2 の規則(`run_id_env` の形)と違う。ほかの誤りは無視するので、壊れた定義でも動く。得た名前の環境変数を見る。
  - 「ゆるく読む」ときに、定義の検査(未知のキー、版など)は行わない。読み方の細部(YAML の重複キー、アンカーなどが入っていた場合に既定名へ戻るか)は [03-definition-format.md](03-definition-format.md) の読み込み部品に従い、ここでは「読み込みに失敗すれば既定名」とだけ規定する。
- 環境変数の値が規則違反なら `invalid value`(68)。
- `reset --all` は、環境変数も `--run-id` も見ない。

## 6. 環境変数

| 名前 | 読む場所 | 効果 |
|------|----------|------|
| `CONDUCTOR_RUN_ID` | `run`, `pipeline`, `watch`, `reset`(`--run-id` なし、かつ定義の `run_id_env` なし) | run ID の既定の供給源(5 章) |
| `run_id_env` が指す名前(定義で指定) | 同上 | `CONDUCTOR_RUN_ID` の代わりに読む環境変数名 |
| `LC_ALL`, `LC_MESSAGES`, `LANG` | 表示が必要になった最初の時点 | この順に、最初の空でない値が `ja` で始まれば日本語、それ以外は英語([08-output-and-i18n.md](08-output-and-i18n.md) 7 章) |
| `NO_COLOR` | `watch` の表を出すとき | 未設定または空なら色を使える。空でない値が設定されていれば色なし(08 章 6.3) |

環境変数の読み取りは、すべて `Deps` の環境変数の口(`lookup_env(name: str) -> str | None`)を通す。`os.environ` を直接読むのは、`Deps` の本番の実装だけにする。これら以外の環境変数は、`conductor` 自身は読まない(子プロセスには `os.environ` をそのまま引き継ぐ。07 章)。Python 自身が読む環境変数(`PYTHONUTF8`、`PYTHONIOENCODING`、`PYTHONDONTWRITEBYTECODE` など)は、`conductor` の挙動の仕様には含めない。ただし、標準出力・標準エラーのエンコーディングは、これらに依存しないよう、起動時に固定する(08 章 1.1)。

## 7. 各コマンドの動作

ここでは入口の検査の順序と、コマンドごとの外から見える挙動を規定する。内部の流れの詳細は [05-execution-flow.md](05-execution-flow.md)。各サブコマンドの実装は `conductor/cli/` の同期関数(`int` を返す)とする。

### 7.1 `run <job-id>`

1. シグナルの監視を開始する(定義の検査より前)。失敗は終了コード 70([07-process-and-signals.md](07-process-and-signals.md))。
2. 定義の読み込みと検証(失敗は 64)。全ジョブの `command` を検査(失敗は 64)。
3. ジョブIDが定義にあるか(なければ `job is not defined in the definition file: '<ID>'`、68)。
4. run ID の決定(5 章。違反は 68)。
5. ステートの読み込み(65)と、定義との照合(65)。
6. 起動前の判断: 実行 / スキップ(0)/ 拒否(66)。
7. 実行なら、ステートが空のとき初期ステートを書き(`Fast` の耐久性)、ジョブをリトライ込みで実行し、結果を分類してステートを 1 回書き(`Durable` の耐久性)、完了かつ削除指定ならステートを消す。終了コードはジョブの結果(10 章)。
8. スキップなら、記録が変わるときだけ書く。完了し削除指定なら消す。0 で終わる。

`-v` のとき、結果行・遷移行・中断行を標準エラーに出す(08 章)。

### 7.2 `pipeline <pipeline-name>`

`run` と同じ検査(パイプライン名は定義の `name` と照合。不一致は 64)を行い、定義の記述順にジョブを 1 つずつ、起動前の判断で処理する。

- 実行 → 結果を記録。エラー(または分類なし=起動前の中断)で停止し、そのジョブの終了コードで終わる。自動の再開・リトライはしない(`retry` の設定を除く)。
- スキップ → 記録して続ける。
- 拒否 → 66 で終わる。
- 完了(最後のジョブ、`@end`、スキップによる完了)したら、完了のステートを残して 0 で終わる。`--remove-on-complete` または定義の `remove_on_complete: true` のときだけステートを消す(消せなければ 65)。
- ジョブのあいだに中断の合図が立っていれば、次のジョブを起動せず、再開位置を記録して `128 + 番号` で終わる(`-v` のとき `[conductor] interrupted by signal N`)。
- 記述順のジョブを使い切っても完了していない(通常は起こらない)と、`cannot determine whether the pipeline is complete` を出して 65。

### 7.3 `watch`

定義ファイルを検証しない。`--run-id`/環境変数で run ID を決め(5.3)、ステートを、完了のものも含めて読み(読み込みの検査は [06-state-file.md](06-state-file.md))、表を標準出力に 1 回出して 0 で終わる。ファイルなし・長さ 0 のときだけ `no state (nothing has run yet)`。読めない・壊れているは 65(`reset` の案内は run ID なしなら付き、run ID ありなら `--run-id` つきの案内が付く。08 章 4.5)。標準出力への書き込み失敗(`BrokenPipeError` を含む `OSError` など)は `cannot write to standard output` を標準エラーに出して 65。表の形式は 08 章 6 章。シグナルの準備はしない。

### 7.4 `reset`

ステートを読まず、中身の検査もしない(壊れていても動く)。ステートがなくても成功(0)。

- `--run-id` なし(かつ `--all` なし): 5.3 の規則で run ID を決め、そのステートファイルを消す。
- `--run-id <ID>`: その run ID のファイルだけを消す。
- `--all`: その定義のステートを、run ID の有無によらず全部消す。`--run-id` との併用は 68。環境変数は見ない。読めなくて残したファイルがあれば、標準エラーに警告を出す(終了コードは 0。08 章 4.10)。消すのに失敗したら 65。消すものがなければ何も出さず 0。

`--all` で消す対象(定義ファイル名を `<定義ファイル名>` とする。定義ファイルと同じディレクトリを、ファイル名の前方一致で走査する。走査には `os.scandir()`(`with` で閉じる)を使ってよい):

- `<定義ファイル名>.state`(run ID なし): 中身を読まず、通常のファイルなら消す。シンボリックリンクは消さない(`DirEntry.is_file(follow_symlinks=False)` が真で `is_symlink()` が偽、または `os.lstat` で判定する。`os.stat` と、既定の `DirEntry.is_file()` はリンクをたどるので判定に使わない)。
- `<定義ファイル名>.<run-id>.state`: 名前の `<run-id>` が 5.2 の規則に合い、かつ、中身の `run_id` の値が名前の run ID と一致するものだけ消す。中身が読めないものは消さずに残し、警告に挙げる。
- 一時ファイル、別の定義のステート、ディレクトリ、シンボリックリンクは消さない。

(この細かい規則は、利用者向けの説明を根拠にした記述で、細部は未確認。ステートファイルの形式は [06-state-file.md](06-state-file.md)。)

### 7.5 `validate [<pipeline-name>]`

定義を読んで検証し、全ジョブの `command` を検査するだけ。ジョブを起動せず、シグナルの準備もせず、ステートに触れない(読まず・書かず・消さない)。壊れたステートがあっても成功する。

- 成功: 何も出さず 0。`-v` のとき、標準エラーに `[conductor] definition is valid (<N> job|jobs)`(英語。`N == 1` のとき `job`、それ以外は `jobs`)/ `[conductor] 定義は正しいです(ジョブ <N> 個)`。
- 失敗: `run` と同じ表示で 64。
- 名前を渡すと定義の名前と照合する(不一致は 64)。省略すると照合しない。`--run-id` は 68(使えないオプション)。

### 7.6 `convert <file> --to <yaml|json>`

入力ファイルを、入力形式を `--to` の反対(`--to json` なら入力は YAML、`--to yaml` なら入力は JSON)とみなして変換する。この判定で、拡張子は見ない(定義ファイルの読み込みとは異なる)。結果を全部作ってから標準出力に 1 回で書く(書き込みの完了を待つ。08 章 1.3)。失敗したときは標準出力に何も書かず、標準エラーに原因を出して 67。標準出力に書けないときも 67(`cannot write to standard output`)。シグナルの準備はしない。変換の規則そのもの(コメントとキー順の扱い、アンカーの展開、入力の大きさの上限など)は [09-converter.md](09-converter.md)。

(「入力形式を `--to` の反対とみなし、拡張子を見ない」という判定の規定は、この章が持つ。`--to yaml` のときの入力は JSON として厳密に読む([09-converter.md](09-converter.md))ので、拡張子が `.yaml` でも、`--to yaml` で渡せば JSON の文法の誤り(67)になる。)

## 8. ヘルプとバージョン

- `-h`/`--help`: 使い方全文を標準出力に出して 0。言語はロケールに従う(全文は 08 章 8 章)。標準出力に書けなくても(閉じたパイプなど)、失敗は無視して 0 で終わる(08 章 1.3.2。`watch` は 65、`convert` は 67 で、ここだけが違う)。
- `-V`/`--version`: 標準出力に `conductor <版>` と改行 1 つ(`conductor 0.8.0`)。ロケールを判定しない(常にこの英語の形)。終了コード 0(標準出力に書けなくても、失敗は無視して 0。08 章 1.3.2)。版の文字列は `conductor/version.py` の定数 `VERSION = "0.8.0"` から作る(`pyproject.toml` の `version` と一致することをテストで検査する。実行時に `pyproject.toml` や、インストール済みの配布物のメタデータ(`importlib.metadata`)を読まない)。

```text
$ conductor --version
conductor 0.8.0
```

## 9. 実行例(`LANG=C`)

```text
$ conductor bogus
conductor: error: unknown command: 'bogus'

Usage:
  conductor run [-v] ...(使い方の全文が続く)
[終了コード 68]

$ conductor run -vv a
conductor: error: invalid value: '-v'
(空行と使い方)
[68]

$ conductor watch -v
conductor: error: option not allowed for this command: '-v'
[68]

$ conductor --run-id 'a b' watch
conductor: error: invalid value: '--run-id'
[68]

$ conductor reset --all --run-id x
conductor: error: invalid value: '--all'
[68]

$ conductor convert conductor.yaml
conductor: error: missing argument: '--to'
[68]

$ conductor convert conductor.yaml --to xml
conductor: error: invalid value: 'xml'
[68]

$ conductor run nojob          # 定義にないジョブID
conductor: error: job is not defined in the definition file: 'nojob'
[68]

$ CONDUCTOR_RUN_ID='bad id' conductor watch
conductor: error: invalid value: 'run ID (environment variable)'
[68]

$ conductor 'a<FF>b'           # バイト 0xFF を含む引数(POSIX)
conductor: error: invalid value: 'a�b'
[68]

$ conductor -vh        # help が勝つ。標準出力に使い方、0
```

## 10. この版が採った選択(まとめ)

| 項目 | 内容 |
|------|------|
| 既定の定義ファイル名 | `conductor.yaml`(`--config` の拡張子は `.yaml`・`.yml`・`.json`。それ以外は定義の誤り `unsupportedFormat`、終了コード 64) |
| `--to` の値 | `yaml` か `json`(それ以外は `invalid value`、68) |
| `convert` | YAML と JSON の相互変換(入力形式は `--to` の反対。拡張子を見ない。出力のキーの順は入力の記述順) |
| 定義の `run_id_env` の読み方(`watch`/`reset`) | YAML/JSON をゆるく読む(読めなければ既定名。03 章) |
| 引数の解析 | `argparse`(・`optparse`・`getopt`)を使わず、4 章の自前の手順(1.1) |
| 非 UTF-8 の引数 | `surrogateescape` の孤立サロゲートで区別し、コマンド名・ジョブID・パイプライン名・`--run-id`・`--to` の値は `invalid value`。`--config`・`convert` の入力のパスはそのまま通す(4.4) |
| 非 UTF-8 の環境変数 | 未設定と同じ扱い(5.1、08 章 7 章) |
| 誤りの表示用の変換 | `readable()`: `surrogateescape` でバイトに戻し、`decode("utf-8", "replace")`(最大の部分列。4.4.2) |

## 11. 未確認

- Windows の `sys.argv`・`os.environ` での、対になっていないサロゲートの扱い(4.4.1)。
- 非 UTF-8 のバイトを持つファイル名を、`surrogateescape` の `str` から実際に開けること(4.4.1。macOS の APFS は、そのファイル名の作成を拒否したため確認できない。Linux で確認する)。
- Python 3.11〜3.13 での `argparse` の挙動(1.1。確認したのは 3.9.6・3.10.7・3.14.10・3.15.2 だけ)。`optparse`・`getopt` の挙動(1.1)。サブコマンドの解析器を使った場合の `--` の扱い(1.1)。
- 連結の短縮形の綴り(`-<c>`)に孤立サロゲートが入る場合の、実機の `conductor` との細かい一致(4.4.2)。`=` が連結の途中にあり、その前後に知らない文字がある組み合わせの細部。
- Windows での `os.environ` の大文字小文字の扱い(5.1。Python の文書の記述に基づくのみ)。
- `reset --all` の対象の細部(7.4)。
- サポートする最小の Python の版(実装時に確認する。この文書を書いた時点の見込みは 3.11 だが、確認していない)で、1.1 と 4.4 の確認を繰り返すこと。
