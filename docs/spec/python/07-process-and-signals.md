# 07 プロセス起動とシグナル(`command` の分割・起動・待ち・中断)

対象: `conductor` v0.8.0 と同じ挙動を Python で実装するための仕様。この文書は、次を定める。

- `command` の文字列を、コマンド名と引数に分ける規則(3 章)
- `subprocess.Popen` によるジョブ(子プロセス)の起動、待ち、結果の取得、起動失敗(4・5 章)
- 中断の合図と、OS ごと(UNIX 系と Windows)のシグナル処理(6・7 章)
- OS ごとの差と、OS 分岐を置く場所(8 章)
- 同期モデル(シグナルハンドラ + ポーリング)の規則と、順序の保証(9 章)
- 想定外の内部の誤り(10 章)

ディレクトリ構成と依存の向きは [02-architecture.md](02-architecture.md)、定義ファイルの検査は [03-definition-format.md](03-definition-format.md)、
ステートファイルは [06-state-file.md](06-state-file.md)、流れ全体は [05-execution-flow.md](05-execution-flow.md)、
表示の文言は [08-output-and-i18n.md](08-output-and-i18n.md)、終了コードの全件は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md)、
テストと品質ゲートは [11-quality-and-build.md](11-quality-and-build.md) にある。この文書は、必要な箇所だけ触れる。

## 規定の強さと確認の表記

| 表記 | 意味 |
|------|------|
| 【規定】 | どの実装も満たす。受け入れテストで確かめる |
| 【目安】 | 推奨の値。異なる値にしてよい。ただし、利用者から見える結果は変えない |
| 【任意】 | 実装に任せる。利用者から見える結果に影響しない |
| 【Python の設計】 | この Python 版が採る設計。利用者から見える挙動は変わらないが、実装者はこれに従う |
| 【実測】 | この文書の作成時に、実際に Python を動かして確かめた挙動(1.2 節に条件を書く) |
| 未確認 | 実測も、公式の文書による裏づけも得ていない。実装時に、採用する版と OS で確かめる(13 章) |

---

## 1. 構成要素と責務

### 1.1 構成要素とファイル

すべて `src/conductor/runner/` に置く(分割は純粋なので、同じパッケージの別のファイル)。ファイルの全体の表は [02-architecture.md](02-architecture.md) の 6 章。

| 構成要素 | 責務 | 置き場所(ファイル) |
|----------|------|---------------------|
| 分割器 | `command` の文字列を、コマンド名と引数に分ける。文字(コードポイント)を 1 つずつ左から読む状態機械。再帰を使わない。3 OS で同じ規則 | `split.py` |
| `command` の検査 | `command` を起動せずに検査する。成功なら分けた結果(`CommandLine`)を返す | `split.py` |
| ジョブの起動と待ち | 1 つのジョブをシェルを介さずに起動し、終了を待ち、実行結果(`JobExecution`)を返す。失敗は値で返し、例外を送出しない | `run_job.py` |
| 待機ループ | `poll()` と短い `sleep` を繰り返し、終了を観測する。転送の判断(1 回だけ、終了済みには送らない)もここで行う(5 章) | `run_job.py`(`run_job` の内側) |
| リトライの待機 | リトライの前に、短い `sleep` の繰り返しで待つ。待機中に中断の合図を確かめる(7.5 節) | `wait_retry.py` |
| 中断の合図 | 「中断が要求された」ことと、最初に受け取ったシグナルの番号を保持する値(6 章) | `interrupt.py` |
| プラットフォーム層 | OS の違いを吸収する唯一の場所。`sys.platform` を読む唯一の場所でもある(8 章) | `platform.py` |
| シグナル監視 | `signal.signal` でシグナルを受け取り、中断の合図を立てる | `watcher.py` |

### 1.2 実測の条件(【実測】の印の意味)

この文書の【実測】は、次の条件で得た。ほかの OS・版では未確認である。

| 項目 | 内容 |
|------|------|
| Python | 3.14.10 と 3.15.2(主な確認)、3.9.6(一部。`process_group` を除く 3.9 でも同じ結果になることの確認に使った) |
| OS | macOS(Darwin 27、arm64) |
| Linux、Windows | 実測していない。値と挙動は、Python の公式の文書と、OS の一般的な知識による(未確認) |
| 対象の版 | サポートする版は、実装時に確認する(`requires-python` の最小版と最新版。[02-architecture.md](02-architecture.md) の 8 章)。この文書の作成時点では、最小が 3.11、最新が 3.15 と思われるが、未確認。この文書の規定は、3.11 以降で使える機能だけに依存する |
| 確認に使ったもの | 実際に `subprocess.Popen`、`signal.signal`、`time.sleep` を動かす小さな確認用の手順書(作業用の場所に置き、リポジトリには含めていない)。分割の状態機械は、3.3・3.4 節の例のすべてを、手元の実装で通した(全件一致を確認) |

---

## 2. `command` の形式(定義の側の受理条件)

定義ファイルの `job` の `command` は、次の 2 つの形式のどちらか(YAML の例。JSON でも同じ型)。

| 形式 | 書き方 | 起動のときの扱い |
|------|--------|------------------|
| 文字列形式 | `command: "sh scripts/x.sh"` | 3 章の規則で、コマンド名と引数に分ける |
| 配列形式 | `command: ["sh", "-c", "echo hi"]` | 要素をそのまま使う(分割・引用符の処理・展開をしない) |

定義の読み込み時の検査(誤りは定義の誤り、終了コード 64。詳細は 03 章):

| 入力 | 判定 |
|------|------|
| `command` キーがない | 誤り(必須の値がない) |
| 文字列で、前後の空白を除くと空 | 誤り(`command` が空) |
| 配列が空(`[]`) | 誤り(配列が空) |
| 配列に文字列でない要素がある | 誤り(要素は文字列で書く)。要素ごとに報告する |
| 配列の先頭の要素が、前後の空白を除くと空 | 誤り(先頭の要素が空) |
| 配列の 2 番目以降が空文字列 `""` | 有効(空の引数として渡す) |
| 文字列でも配列でもない(数値・マップなど) | 誤り(型) |

「前後の空白を除く」の定義は 03 章に従う。注意: Python の `str.strip()` と `str.isspace()` は、Unicode の White_Space の全件に加えて、U+001C〜U+001F(制御文字 4 つ)も空白として扱う【実測。3.9、3.14 で `str.isspace()` と White_Space の差を、全コードポイントで比べた。差はこの 4 つだけ】。03 章の定義と合わない場合は、03 章の定義を実装した自前の関数を使うこと(`str.strip()` を `command` の検査に使わない。03 章と合わせる点)。

定義の検査とは別に、実行の前にもう一度、全ジョブの `command` を 3.6 節のとおり検査する。

---

## 3. `command` の分割規則

### 3.1 入力と出力

- 入力: `command` の文字列(Python の `str`。**コードポイントの列**)。
- 出力: コマンド名(`program`。空でない)と、引数の並び(`args`)。`args` の各要素が、そのまま 1 つの引数になる。
- シェルは使わない。変数展開(`$HOME`)、グロブ(`*`)、パイプ(`|`)、`;`、`&&`、リダイレクト(`>`)、バッククォート、`~`、
  括弧は **解釈しない**。普通の文字として引数に入る。パイプなどが要るときは、利用者が `sh -c '...'` を `command` に書く。
- 標準ライブラリの `shlex.split`、`str.split()`、`str.strip()` は使わない【規定】。`shlex` は、バックスラッシュの扱い(引用符の外で解釈する)、`#` のコメント、`posix=False` 時の引用符の残り方などが、この規則と異なる。この規則は、次の 3.2 節の表だけで決まる。

### 3.2 文法

`for ch in s:` で、コードポイントを、左から右へ **1 回だけ**読む(Python の `str` の走査は、コードポイント単位であり、UTF-16 の符号単位でも書記素でもない)。区切り文字は次の 4 つだけ。

```text
separator := ' ' (U+0020) | '\t' (U+0009) | '\n' (U+000A) | '\r' (U+000D)
```

(U+000B、U+000C、U+00A0、U+2028 などのほかの空白は区切りではなく、普通の文字。)

状態は 4 つ。これに加えて、`in_word`(語の途中か)と `current`(組み立て中の語)を持つ。

| 状態 | 入力の文字 | 動作 | 次の状態 |
|------|-----------|------|----------|
| Plain(引用符の外) | separator | `in_word` なら `current` を語として確定し、空にする。`in_word = false` | Plain |
| Plain | `'` | `in_word = true`(語を開始。まだ何も足さない) | Single |
| Plain | `"` | `in_word = true` | Double |
| Plain | 上記以外(`\` を含む) | `in_word = true`。その文字を `current` に足す | Plain |
| Single(`'...'` の中) | `'` | 何も足さない | Plain |
| Single | 上記以外(`\`、`"`、改行を含む) | その文字を足す | Single |
| Double(`"..."` の中) | `"` | 何も足さない | Plain |
| Double | `\` | 何も足さない(バックスラッシュを保留する) | DoubleBackslash |
| Double | 上記以外 | その文字を足す | Double |
| DoubleBackslash(`"..."` の中で `\` を 1 つ保留中) | `"` | `"` を 1 つ足す(保留の `\` は消える) | Double |
| DoubleBackslash | `\` | 保留していた `\` を 1 つ足す。今読んだ `\` が新しい保留になる | DoubleBackslash |
| DoubleBackslash | 上記以外の文字 c | `\` と c を足す | Double |

入力の終わりでの確定:

1. 状態が Plain でなければ(Single / Double / DoubleBackslash)、**閉じていない引用符**(誤りの種類 `unclosedQuote`)。
2. `in_word` なら `current` を最後の語として確定する。
3. 語の列の先頭がなければ、または先頭の語が空文字列なら、**コマンド名がない**(誤りの種類 `empty`)。それ以外は、先頭の語が `program`、
   残りが `args`。

要点:

- 引用符で囲んだ部分と囲まない部分は、隣り合っていればつなげて 1 つの語になる(`a"b c"'d e'f` は 1 語 `ab cd ef`)。
- `''` と `""` は、空の語(空の引数)になる(`in_word` が立つため)。ただし先頭の語が空なら `empty`。
- 二重引用符の中で解釈するバックスラッシュは、「直後が `"`」のときだけ(`\"` → `"`)。`\n`、`\t`、`\\` なども、
  **解釈せずそのまま**(`\n` は 2 文字のまま)。ただし `\\` は、1 つ目のバックスラッシュを出力し、2 つ目を新しい保留にする。
  したがって `\\"` は「`\` と、エスケープされた `"`」になる(引用符は閉じない)。`\` の個数の偶奇で数える規則ではなく、
  直前に保留している `\` が 1 つあるかどうかだけで決まるので、上の表を左から機械的に適用すること
  (`"` の直前に `\` が 1 つ以上並んでいれば、`"` は常にエスケープされ、引用符は閉じない)。
- 引用符の外のバックスラッシュは普通の文字(Windows のパス `C:\dir\x.exe` がそのまま書ける)。一重引用符の中も同じ。
- 末尾がバックスラッシュのパスを二重引用符で囲むと `\"` が引用符のエスケープになり、閉じない(誤り)。一重引用符で書く。
- 文字列の長さに上限はなく、入れ子もないため、再帰やスタックの深さ(`sys.getrecursionlimit()`)に依存しない(約 30 万引数の入力でも分割できる)。【規定】【実測: 30 万引数の分割は 0.03 秒(3.14)】

### 3.3 `str` を使う場合の注意【Python の設計】

| 論点 | 規定 |
|------|------|
| 走査の単位 | `for ch in s` による、コードポイント単位。添字の走査(`s[i]`)でも結果は同じ。`str` は UTF-16 ではないので、サロゲートペアを気にする必要はない(`𠮷` や絵文字は 1 コードポイント) |
| 孤立サロゲート | OS から `surrogateescape` で入った引数や環境変数の不正なバイトは、U+DC80〜U+DCFF の孤立サロゲートとして `str` に入る。分割器は、これを「上記以外の文字」として、そのまま語に入れる(検査も置換もしない)。区切り・引用符・バックスラッシュはすべて ASCII なので、サロゲートのどれとも等しくない |
| 起動のときの扱い | U+DC80〜U+DCFF は、`subprocess` が元のバイトに戻して子に渡す【実測: `"a\udcffb"` の引数が、子で `61 ff 62` のバイト列として届いた。環境変数も同じ】。それ以外の孤立サロゲート(U+D800〜U+DC7F、U+DD00〜U+DFFF)は、`UnicodeEncodeError`(`ValueError` の下位)になる【実測。`'\ud800'`】。この変換は、分割器が防ぐことではない。4.3 節で、起動失敗として扱う |
| 正規化・大文字小文字・照合 | しない。`unicodedata.normalize`、`str.lower`、`str.strip`、`str.split` を、語に対して呼ばない |
| 語の組み立て | `current` は、文字の `list` に `append` し、確定のときに `"".join(current)` で文字列にする。語の列(`words`)は `list` に `append` する。文字列の `+=` の繰り返しに頼らない(版によって速度が異なるため【目安】) |
| 戻り値 | `args` は新しい `tuple[str, ...]`(`tuple(words[1:])`。1 語だけのときは空の `tuple`) |

### 3.4 擬似コード(同じ結果になる実装であれば、書き方は自由)

```text
words = []; current = []; in_word = false; state = Plain
for c in s:                       # コードポイント
  switch state:
    Plain:  if c in separator: if in_word: words.append("".join(current)); current = []; in_word = false
            elif c == "'": in_word = true; state = Single
            elif c == '"': in_word = true; state = Double
            else: in_word = true; current.append(c)
    Single: if c == "'": state = Plain else current.append(c)
    Double: if c == '"': state = Plain
            elif c == '\\': state = DoubleBackslash
            else: current.append(c)
    DoubleBackslash:
            if c == '"': current.append('"'); state = Double
            elif c == '\\': current.append('\\')       # state は DoubleBackslash のまま
            else: current.append('\\'); current.append(c); state = Double
if state != Plain: return 誤り(unclosedQuote)
if in_word: words.append("".join(current))
if words が空 or words[0] == "": return 誤り(empty)
return (words[0], tuple(words[1:]))
```

### 3.5 受理の例

| 入力(command の文字列) | program | args |
|--------------------------|---------|------|
| `echo a b` | `echo` | `a`, `b` |
| `␣␣echo ⇥ a ⏎ b␣␣`(空白・タブ・改行の混在) | `echo` | `a`, `b` |
| `echo\r\na`(CR LF) | `echo` | `a` |
| `echo 'a b' 'c\d' '"x"'` | `echo` | `a b`, `c\d`, `"x"` |
| `echo "a \" b"` | `echo` | `a " b` |
| `echo "a\nb" "c\\d"` | `echo` | `a\nb`, `c\\d`(バックスラッシュは、そのまま 2 つ) |
| `echo "\\"x"`(`"` `\` `\` `"` `x` `"`) | `echo` | `\"x` |
| `C:\dir\x.exe a\ b` | `C:\dir\x.exe` | `a\`, `b` |
| `\\server\share` | `\\server\share` | (なし) |
| `prog 'C:\dir\'` | `prog` | `C:\dir\` |
| `echo --msg='a b'` | `echo` | `--msg=a b` |
| `echo a"b c"'d e'f` | `echo` | `ab cd ef` |
| `echo '' "" x` | `echo` | (空), (空), `x` |
| `echo ''` | `echo` | (空) |
| `echo a;b && c \| d > e $HOME * `` `x` `` (y)` | `echo` | `a;b`, `&&`, `c`, `\|`, `d`, `>`, `e`, `$HOME`, `*`, `` `x` ``, `(y)` |
| `echo 'あ い' う` | `echo` | `あ い`, `う` |
| `echo '𠮷 😀' x`(補助面の文字を含む) | `echo` | `𠮷 😀`, `x` |
| `echo a<U+000B>b<U+00A0>c`(U+000B と U+00A0 は区切りでない) | `echo` | `a<U+000B>b<U+00A0>c`(1 語) |
| `echo <U+DC80>x`(孤立サロゲート) | `echo` | `<U+DC80>x` |
| `a 'b c'` | `a` | `b c` |

### 3.6 拒否の例

| 入力 | 誤りの種類 |
|------|-----------|
| `echo 'a` | `unclosedQuote` |
| `echo "a` | `unclosedQuote` |
| `echo "a\`(末尾が二重引用符の中のバックスラッシュ) | `unclosedQuote` |
| `echo "a\\" b`(`\\` + `"` は、引用符のエスケープ) | `unclosedQuote` |
| `prog "C:\dir\"` | `unclosedQuote` |
| (空文字列) | `empty` |
| `␣⇥⏎`(区切りだけ) | `empty` |
| `''` | `empty`(先頭の語が空) |
| `'' x` | `empty`(先頭の語が空) |

### 3.7 配列形式の扱い

- 要素を一切加工せずに使う。先頭が `program`、残りが `args`。分割、引用符の解釈、展開、前後の空白の除去をしない
  (`"  lead and trail  "` も、そのまま 1 つの引数)。
- 先頭の要素が、前後の空白を除いて空(空文字列・空白だけ)、または要素が 0 個のときは `empty`(定義の検査の二重の備え)。
  先頭の要素が空白を含むが空でないとき(`"my prog"`)は、そのまま `program`(`my prog` というファイル名を起動する)。
- 2 番目以降は空文字列でもよい(空の引数)。

### 3.8 誤りの表示と検査のタイミング

| 項目 | 内容 |
|------|------|
| 誤りの内容 | 種類(`unclosedQuote` か `empty`)と、固定の文 |
| 固定の文 | `unclosedQuote` は `引用符が閉じていません`、`empty` は `コマンド名がありません`。**入力の値を含めない**(秘密の値が `command` に入っていても表示に出ない)【規定】。`CommandError` の `args[0]`(メッセージ)も、この固定の文だけにする |
| 利用者への表示(標準エラー) | `conductor: error: <定義ファイルのパス>: job '<ジョブID>': <文>`。文は表示言語に従う。英語は `command has an unclosed quote` / `command is empty`、日本語は `command の引用符が閉じていません` / `command が空です`(組み立ては `report`。08 章) |
| 検査の時点 | `run` / `pipeline` / `validate` は、定義の読み込みと検証のあと、**全ジョブ**の `command` を検査し、誤りのあるジョブをすべて報告して(1 ジョブ 1 行)、終了コード 64 で終わる。ジョブは 1 つも起動せず、ステートにも触れない |
| 順序 | シグナルの準備(`run`/`pipeline` のみ)→ 定義の読み込み(64)→ `command` の検査(64)→ ジョブID(68)→ run ID(68)→ ステートの読み込み(65)→ 照合(65) |
| 防御 | ジョブの起動の内側でも分割し直す。万一失敗したら、起動失敗の実行結果を返す(通常は到達しない) |

検査の関数の骨格(型は [02-architecture.md](02-architecture.md) の 2.1 節。実装コードではない):

```python
@dataclass(frozen=True, slots=True)
class CommandLine:
    program: str
    args: tuple[str, ...]

CommandErrorKind = Literal["unclosedQuote", "empty"]

class CommandError(Exception):
    kind: CommandErrorKind          # メッセージ(args[0])は固定の文だけ。入力の値を含めない

@dataclass(frozen=True, slots=True)
class CommandOk:
    line: CommandLine
    ok: Literal[True] = True

@dataclass(frozen=True, slots=True)
class CommandErr:
    error: CommandError
    ok: Literal[False] = False

CommandResult = CommandOk | CommandErr

def split_command(s: str) -> CommandResult: ...
def check_command(c: Command) -> CommandResult: ...   # 文字列形式は split、配列形式は 3.7 節
```

---

## 4. プロセスの起動(`subprocess.Popen`)

### 4.1 手順(時系列)

関数: `run_job(job: Job, sig: Interrupt, *, spawn: Spawner = ..., platform: Platform = ..., sleep: Callable[[float], None] = time.sleep) -> JobExecution`。失敗はすべて戻り値で返し、例外を送出しない【規定】(`Spawner`、`Platform` は 5.3 節と 8 章)。

1. 実行結果を初期化する: `started = True`、`exit_code = None`、`terminated_by_signal = None`、`start_failure = False`、
   `interrupted = False`、`signal_number = None`。
2. `command` を分割する(文字列は 3 章、配列は 3.7 節)。失敗なら `start_failure = True` で返す。
3. **起動の前**に合図が立っていれば(`sig.number()` が `None` でない)、起動せずに返す: `started = False`、`interrupted = True`、`signal_number = 番号`。
   (ジョブは実行されない。)
4. プラットフォーム層で、`Popen` に渡す第 1 引数とオプションを決める(8 章)。拒否された場合(Windows の `.cmd` の安全策、4.8 節)は、`start_failure = True` で返す。
5. `sys.stdout` と `sys.stderr` を `flush()` してから(9.2 節)、`Popen(...)` を呼ぶ。**`OSError` と `ValueError` を、その場で捕まえ**(4.3 節)、`start_failure = True` で返す(`started` は `True` のまま)。ほかの例外は捕まえない(実装の不具合。10 章)。
6. 返された `Popen` を、5 章の待機ループに渡す。時間切れ(タイムアウト)はない。自動のリトライもしない(リトライは `cli` の流れの制御で行う。7.5 節)。
7. 終了を観測したら(`poll()` が `None` 以外を返したら)、結果を取り出す(4.4 節):
   - `exit_code` = 子の終了コード(UNIX 系でシグナルで終了したときは `None`)。
   - `terminated_by_signal` = シグナル終了のときの番号(UNIX 系のみ。Windows は常に `None`)。
   - **この時点で** 合図が立っていれば(`sig.number()` を 1 回読む)、`interrupted = True`、`signal_number = 番号`。
8. `poll()` が `OSError` を送出した、または終了の状態を取得できないときは、`start_failure = True` で返す(分類で 127)。

### 4.2 起動の条件(`Popen` の引数)

| 項目 | 規定 |
|------|------|
| シェル | 使わない。`shell=False` を**明示する**【規定】。`sh -c` や `cmd /C` を挟まない。`Popen` に渡すのは、分割(または配列)で得た `program` と `args` だけ(第 1 引数は `[program, *args]` の `list[str]`)。唯一の例外は、Windows で `.cmd` / `.bat` を起動するときの `cmd.exe` 自身(4.8 節。`cmd.exe` を `program` として直接起動し、`shell=False` のまま) |
| 使う引数 | `stdin=None`、`stdout=None`、`stderr=None`(いずれも明示する)、`shell=False`、`close_fds=True`、`restore_signals=True`、`start_new_session`(UNIX 系は `True`、Windows は指定しない。4.7 節)、`creationflags`(Windows は 0 のまま。指定しない)。**これ以外の引数を渡さない** |
| 使わない引数 | `cwd`、`env`、`executable`、`user`、`group`、`extra_groups`、`umask`、`preexec_fn`、`pass_fds`、`process_group`、`text`、`encoding`、`bufsize`、`pipesize`。`preexec_fn` は、スレッドとの併用で安全でないため禁止する |
| 使わない関数 | `subprocess.run`、`subprocess.call`、`subprocess.check_output`、`Popen.communicate`、`os.system`、`os.popen`、`os.spawn*`、`os.fork`、`os.exec*`。`run`(と `call`)は、`KeyboardInterrupt` を受けると子を `kill` する、`timeout` で子を `kill` する、といった動作を持つため、強制終了しない規定に反する。`communicate` は、パイプを前提とする |
| プログラムの探索 | `Popen` の標準の規則に従う(`program` に区切り文字がなければ、UNIX 系では `PATH` の探索。Windows では `CreateProcess` の規則)。`conductor` 独自の探索(`shutil.which` など)はしない。Windows での探索の細部(拡張子の補完、カレントディレクトリを含むか)も、標準の規則に従い、見つからなければ起動失敗として扱う(13 章) |
| 作業ディレクトリ | `cwd` を渡さない(`conductor` のカレントディレクトリを引き継ぐ。定義ファイルの場所に移らない)。`conductor` は `os.chdir()` を呼ばない【規定】 |
| 環境変数 | `env` を渡さない(すべて引き継ぐ。`os.environ` が使われる)。追加・削除・上書きをしない。run ID も環境変数としては渡さない。`conductor` は `os.environ` を書き換えない【規定】 |
| 標準入力・標準出力・標準エラー | `None`(既定)を明示する【規定】。`None` のとき、子は親のファイル記述子 0・1・2 を引き継ぐ(リダイレクトしない)。`PIPE` を渡すと、子の出力が親に溜まり、引き継ぎにならない。`conductor` は、ジョブの出力を読まない・保存しない・ステートに書かない【規定】。`Popen` の `stdin`/`stdout`/`stderr` 属性は `None` になる |
| ファイル記述子 | `close_fds=True`(既定)。0・1・2 以外の記述子は子に渡さない。Python が開く記述子は、既定で継承されない(PEP 446) |
| 引数の渡し方 | 分割した各引数を、そのまま `list[str]` で渡す(メタ文字は注入にならない)【規定】。Windows で `.cmd` / `.bat` を起動するときは 4.8 節の安全策を適用する |
| 起動できなかった場合 | 原因(見つからない・実行権限がない・実行できない形式・ディレクトリを指定した、など)を区別せず `start_failure = True`。終了コードは 127(分類)。表示は `conductor: error: job '<ID>': cannot start command '<プログラム名>'`(英語。組み立ては `report`)。ジョブの終了コード・シグナルは `None`。原因の `errno` と例外の文は、表示にも、ステートにも出さない |
| 時間の上限 | ない。ジョブが終わらなければ、`conductor` は待ち続ける。`Popen` の `timeout` に相当するものを使わない |
| ジョブの強制終了 | しない(`kill()`、`terminate()`、`SIGKILL` の送信を、製品のコードに書かない) |

### 4.3 起動失敗の検出

`Popen(...)` は、起動に失敗すると、例外を送出する(子は残らない)。次を、すべて 1 つの「起動失敗」にまとめる。

| 例外 | 起こる場面 | `errno`(参考) | 実測 |
|------|-----------|-----------------|------|
| `FileNotFoundError`(`OSError`) | `program` が見つからない | `ENOENT`(2) | 実測(`/nonexistent/x`) |
| `PermissionError`(`OSError`) | 実行権限がない。ディレクトリを指定した。空の `program` | `EACCES`(13) | 実測(`/etc/passwd`、`/tmp`、`""`) |
| `NotADirectoryError`(`OSError`) | パスの途中がディレクトリでない | `ENOTDIR` | 未確認 |
| `OSError`(その他の下位型を含む) | 実行できない形式(シバンのないスクリプトなど)。引数の合計が長すぎる | `ENOEXEC`(8)、`E2BIG`(7) | 実測(macOS。Linux の `errno` の値は未確認) |
| `ValueError` | 引数に NUL(U+0000)を含む | なし | 実測(`embedded null byte`) |
| `UnicodeEncodeError`(`ValueError` の下位) | 引数に、元のバイトに戻せない孤立サロゲートを含む(3.3 節) | なし | 実測(`'\ud800'`) |

規定:

1. `except (OSError, ValueError)` の 1 つの節で捕まえる(`UnicodeEncodeError` は `ValueError` に含まれる)。例外の型と `errno` で分岐しない。**`FileNotFoundError` と `PermissionError` を、127 以外の終了コードに結びつけない**。
2. `Popen` が例外を送出したとき、子プロセスは存在しないので、待ちも回収もしない(`poll()` を呼ばない)。
3. 例外の文(`str(e)`)と `e.errno` を、利用者への表示にも、ステートにも出さない(入力の値や内部の情報が出るため。[10-errors-and-exit-codes.md](10-errors-and-exit-codes.md))。デバッグ用に残す場合も、利用者への出力に出さない。
4. `MemoryError`、`KeyboardInterrupt`、`SystemExit`、`RecursionError` などは捕まえない(10 章)。
5. 実行時に「見つからない」ことを事前に調べない(`shutil.which`、`os.access` で先に確かめない)。確かめた後に状態が変わる(競合)ことと、探索の規則が `Popen` と食い違うことを避けるため。

### 4.4 終了の取り出し(`returncode`)

`Popen.poll()` は、子が終わっていれば `returncode`(`int`)を返し、終わっていなければ `None` を返す。終わった子は、このとき OS から回収される(ゾンビが残らない)。

| 項目 | UNIX 系 | Windows |
|------|---------|---------|
| 終了した | `returncode` は 0〜255 の整数(`exit(300)` は 44【実測】)。`exit_code = returncode`、`terminated_by_signal = None` | `returncode` は子の終了コード。Python は、Windows の `DWORD`(符号なし 32 ビット)を、そのまま整数にする場合がある(例: Ctrl+C による終了の `0xC000013A` が 3221225786 になる)。`exit_code = ((rc + 2**31) % 2**32) - 2**31`(符号つき 32 ビットに直す。すでに符号つきの値は変わらない)。`terminated_by_signal` は常に `None` |
| シグナルで終了した | `returncode` は **負の値 `-n`**(`n` はシグナル番号)【実測: SIGTERM で -15、SIGKILL で -9、SIGUSR1 で -30(macOS の番号)】。`terminated_by_signal = -returncode`、`exit_code = None` | なし(終了はすべて終了コード。負の値の `returncode` を、シグナルとして読まない) |
| 終了の状態を取得できない | `poll()` が `OSError` を送出した場合。`start_failure = True`(手順 8) | 同左 |

- シグナル番号は、`-returncode` をそのまま使う。**固定の表を書かない**(OS により番号が異なる。`SIGUSR1` は macOS で 30 だが、Linux では 10 と思われる(未確認))。`SIGHUP`(1)、`SIGINT`(2)、`SIGQUIT`(3)、`SIGKILL`(9)、`SIGTERM`(15)は、両 OS で同じ。
- 終了コードの幅(UNIX 系は下位 8 ビット、Windows は 32 ビット符号つき)は、05 章と 10 章のとおり。`300` を返すと UNIX 系は 44、Windows は 300 のまま。
- `returncode` の符号で「シグナルか終了コードか」を判別できるのは UNIX 系だけ。この判別は、プラットフォーム層の `read_exit(returncode)` が 1 か所で行う(8 章)。
- **SIGCHLD が無視(`SIG_IGN`)されている環境**: 子の終了状態は OS が自動で破棄し、Python は、終了状態を取得できない子の `returncode` を 0 にする、と公式の実装にある(未確認。Linux の実機で確かめていない。macOS では、`SIGCHLD` を無視した状態で起動しても、`returncode` は正しい値(3)になった【実測】)。これは、ジョブの失敗を成功として扱う不具合になる。したがって、UNIX 系では、`Popen` を呼ぶ前に `signal.getsignal(signal.SIGCHLD)` を読み、`signal.SIG_IGN` なら `signal.SIG_DFL` に直す(プラットフォーム層の `prepare_child_reaping()`。メインスレッドで呼ぶ。`signal.signal` が `ValueError` なら何もしない)。親が `SIGCHLD` を無視して起動した場合に限った処置で、通常は何もしない【規定】。

実行結果(`JobExecution`)の各項目は 4.5 節。

### 4.5 実行結果(`JobExecution`)の各項目

| 項目 | 型 | 意味 |
|------|-----|------|
| `job_id`, `command` | `str`, `Command` | 定義のジョブIDと `command`(表示とステートには `job_id` だけを使う。`command` はステートに書かない) |
| `started` | `bool` | 起動を試みたか。`False` は、起動の前に合図が立っていたときだけ |
| `exit_code` | `int \| None` | 子の終了コード。UNIX 系でシグナルによって終了したときは `None`。Windows は常に値がある(起動失敗を除く) |
| `terminated_by_signal` | `int \| None` | UNIX 系のシグナル終了の番号。Windows は常に `None` |
| `start_failure` | `bool` | 分割の失敗、起動の失敗、終了の状態の取得の失敗のいずれか |
| `interrupted` | `bool` | 起動前、または、終了の観測の直後に、合図が立っていた |
| `signal_number` | `int \| None` | `interrupted` のとき、**最初に受け取った**シグナルの番号 |

型の全体は [02-architecture.md](02-architecture.md) の 2.1 節と 05 章に従う(`dataclass(frozen=True, slots=True)`)。`bool` は `int` の下位型なので、`exit_code` に `True`/`False` を入れない(`returncode` は `int` のみ)。

### 4.6 分類(終了コードの決定)

分類は `flow` の純粋な関数が行う。ここでは、規定の優先順位を再掲する。実行結果は、次の優先順位で、最初に当てはまるものを採る。

| 順 | 条件 | 分類 | 報告する終了コード |
|----|------|------|--------------------|
| 1 | `started = False` かつ `interrupted` | 分類なし(エラーとして記録しない) | `128 + signal_number` |
| 2 | `started = False` で中断でもない(起こらない) | エラー | 127 |
| 3 | `interrupted`(起動済み) | エラー | `128 + signal_number`(ジョブ自身の終了コードは見ない。ジョブが 0 で終わっていてもエラー) |
| 4 | `start_failure` | エラー | 127 |
| 5 | `terminated_by_signal = n` | エラー | `128 + n` |
| 6 | `exit_code = None`(起動して終わったのに値がない) | エラー(`conductor` 自身のエラー) | 1 |
| 7 | `exit_code` が成功コード(既定は `[0]`、`success_exit_codes` で指定) | 成功 | 0 |
| 8 | `exit_code` が `warning_exit_codes` | 警告 | 0 |
| 9 | `exit_code = 0` で成功コードに含まれない | エラー(`conductor` 自身のエラー) | 1 |
| 10 | 上記以外 | エラー | `exit_code` のまま |

`conductor` の終了コードは、報告する終了コードを 0〜255 に合わせたもの(範囲内はそのまま。範囲外は下位 8 ビット。エラーで
下位 8 ビットが 0 になるときは 1)。詳細は 10 章。

### 4.7 `start_new_session` を使う/使わない【Python の設計】

| 項目 | UNIX 系(Linux、macOS) | Windows |
|------|------------------------|---------|
| 指定 | **`start_new_session=True`** にする【規定: 新しいプロセスグループで動かす】。`process_group=0`(3.11 以降)は使わない | `start_new_session` も `creationflags` も指定しない |
| 効果 | 子は、新しいセッションのリーダーになり、プロセスグループ ID が子の pid になる(`setsid`)【実測: `ps` で `PGID = PID`。指定なしでは、親と同じ `PGID`】。端末の Ctrl+C(SIGINT)は、端末のフォアグラウンドのプロセスグループにだけ配られるため、ジョブには直接届かず、`conductor` が受けて 1 回だけ転送する(7.2 節)。指定なしのままだと、ジョブは SIGINT を直接受けたうえで、`conductor` の転送でもう 1 回受ける(2 回受ける) | 同じコンソールで動かし、Ctrl+C と Ctrl+Break を OS に配らせる規定(7.3 節)に合わせる。`creationflags=CREATE_NEW_PROCESS_GROUP` を指定すると、子の Ctrl+C が無効になり、Ctrl+Break だけが届く、とされるため使わない(未確認) |
| 副作用 | 子は**制御端末を持たない**。ジョブが `/dev/tty` を直接開く(`ssh`、`sudo` のパスワード入力など)と、`ENXIO`(`Device not configured`)で失敗する【実測: 擬似端末の下で、指定なしでは `/dev/tty` に書けたが、`start_new_session=True` では失敗した(macOS)】。標準入力(記述子 0)が端末のとき、ジョブがそこから読むことは**できる**【実測: 擬似端末の下で、`start_new_session=True` の子が標準入力から 1 行を読めた】 | なし |
| `process_group=0` を採らない理由 | 同じセッションの別のプロセスグループ(バックグラウンドのグループ)になるため、ジョブが端末の標準入力を読むと、`SIGTTIN` で停止する【実測: 擬似端末の下で、`process_group=0` の子が標準入力を読もうとして止まった(macOS)。`Popen.poll()` は停止を終了として報告しないので、`conductor` は待ち続ける】。`start_new_session=True` では、この問題は起きなかった | なし |
| 親が先に終わったとき | `conductor` は子の終了を待つ。親(`conductor`)が強制終了された場合、子は残って動き続ける(検知も停止もしない) | 同左 |

制御端末を持たないことは、この版の副作用であり、`/dev/tty` を直接開くジョブは動かない(受け入れる)。標準入力から読むジョブは動く。目的はプロセスグループの分離だけである。

### 4.8 Windows の `.cmd` / `.bat` の起動【Python の設計】

Windows の `cmd.exe` は、`.cmd` / `.bat` を実行するときに、引数を独自の規則で解釈する(`"`、`%`、`^`、`&`、`|`、`<`、`>` など)。`Popen` に引数の `list` を渡すと、Python は `subprocess.list2cmdline` で、1 本のコマンドラインに連結する。この連結は、`CreateProcess` の規則(`CommandLineToArgvW`)に合わせたもので、`cmd.exe` の規則には合わない。このため、`.cmd` / `.bat` の引数に、`cmd.exe` が解釈する文字が入ると、引数の中のコマンドが実行されうる(未確認。Python の版ごとに、この対策が入っているかは確認していない)。`conductor` は、引数の中のコマンドが実行されないことと、安全な引数で起動できること(終了コード 7 が取れる)を守る(受け入れテスト。11 章)。

決定(次のとおり、`cmd.exe` を直接起動する。`shell=False` のまま):

1. `platform.kind` が Windows のとき、かつ `program` の拡張子が(大文字小文字を区別せず)`.cmd` または `.bat` のときだけ、この規則を使う(`ntpath.splitext(program)[1].lower()` で判定する)。拡張子が付いていない `program`(例: `build`)は、`Popen` の標準の探索に任せる(`.cmd` は補完されない。見つからなければ起動失敗。未確認)。
2. 検査: `args` のどれかが、次の文字を含むなら、**起動せず**、起動失敗の実行結果を返す。`program` は、`(` と `)` を除く同じ集合を検査する(パス `C:\Program Files (x86)\...` を許す)。

```text
"  %  ^  &  |  <  >  !  (  )  改行(\n)  復帰(\r)  NUL
```

3. 検査を通ったら、`Popen` の第 1 引数に、**`list` ではなく 1 本の文字列**(コマンドライン)を渡す(Windows の `Popen` は、文字列を `CreateProcess` にそのまま渡し、再び引用しない。公式の文書による。未確認)。文字列は、`"<cmd.exe のパス>" /d /s /c "<全体>"` の形にする。`<cmd.exe のパス>` は、環境変数 `ComSpec` が絶対パスならその値、そうでなければ `cmd.exe`(環境変数は `Env.lookup_env` から受け取る。8 章)。`<全体>` は、`program` と各 `args` を、それぞれ `"` で囲み、空白 1 つで連結した文字列(空の引数は `""`)。
4. 配列形式の `command` も、同じ検査を通る。
5. 検査の関数と、コマンドラインの組み立ては、どの OS でも動く純粋な関数にする(Linux の 1 台から、Windows の分岐を単体テストできる。8 章)。

この禁止の文字の集合と、`cmd.exe` の起動の形は、この仕様の設計判断であり、Windows の実機で確かめていない(未確認。13 章)。`cmd.exe` が `.bat` の実行中に Ctrl+C を受けたとき「バッチ ジョブを終了しますか (Y/N)?」と尋ねる場合の影響も未確認。

---

## 5. 待ち(ポーリングによる待機)

### 5.1 規定

| 項目 | 内容 | 強さ |
|------|------|------|
| 待ち方 | `Popen.poll()` と `sleep` の繰り返し(待機ループ)。`Popen.wait()`(引数なし)を使わない。理由は 5.2 節 | 【Python の設計】 |
| 回収 | 子の終了の状態を読む(`poll()` を呼ぶ)のは、`run_job` の待機ループだけ(と、`send_signal` が内側で呼ぶ `poll()`) | 【規定】 |
| 転送の判断 | 「子がまだ終わっていない」と観測した時点で、**まだ転送していない**かつ合図が立っていれば、シグナルを子に **1 回だけ** 転送する(7.2 節) | 【規定】 |
| 終了済みへの送信の禁止 | 終了済み・回収済みの子には送らない。子が先に終わっていれば、合図が立っていても転送しない | 【規定】 |
| 転送は 1 回 | 2 回目以降の合図(同じシグナルでも別のシグナルでも)で、再度の転送や強制終了はしない | 【規定】 |
| 転送のあと | 子の終了を待ち続ける(子がシグナルを無視すれば、`conductor` はそのまま待ち続ける) | 【規定】 |
| 確認の間隔 | `poll()` と `poll()` のあいだに 1、2、4、8、10、10、…(ミリ秒。前回の 2 倍、上限 10 ms)眠る | 【目安】(間隔が長すぎて、合図からの転送が目に見えて遅れないこと。10 ms 以下を推奨。Windows の `sleep` の分解能は、この値より粗いことがあり、その場合は粗い間隔になる。未確認) |
| CPU の使用 | 子の実行中の `conductor` の CPU 時間の増分(待機の 1 秒後から 2.5 秒後まで)が 50 ms 以下であること(11 章 AC-403。Linux で `/proc` から計測する) | 【規定】【実測: 3 秒間、子が眠っている間のポーリングで、プロセス全体の CPU 時間は 0.024 秒(起動の分を含む。macOS)】 |
| 待機の専用スレッド | 使わない(呼び出したスレッドで待つ) | 【規定】 |

### 5.2 `poll` と `wait(timeout)` の使い分け

| 方法 | 動作【実測。3.14、3.15、3.9 で同じ】 | 採否 |
|------|---------------------------------------|------|
| `Popen.wait()`(引数なし) | シグナルのハンドラ(フラグを立てるだけ)が、待ちの途中で実行されても、待ちは**終わらず**、子が終わるまで戻らない(PEP 475。EINTR のあとの自動の再試行)。シグナルから 0.2 秒後に子が終わる設定で、`wait()` の戻りは子の終了(1.01 秒)で、ハンドラの実行は 0.21 秒だった | 使わない。ハンドラがフラグを立てても、待つ側が気づけず、転送が子の終了後になる(転送の規定に反する) |
| `Popen.wait(timeout=t)` | 内部で、短い間隔のポーリングを行う(実装の詳細。未確認)。ハンドラが実行されても、`timeout` が来るまで戻らない。`timeout` の長さで、フラグの確認の頻度が決まる | 使わない(自前の `poll` + `sleep` で、確認の間隔と、`sleep` の差し替えを制御できるため)。使うなら、`t` は 10 ms 以下 |
| `Popen.poll()` | 終了していれば `returncode`、そうでなければ `None`。ブロックしない。終了していれば、その場で回収する | **採用**。待機ループの中心 |
| `time.sleep(t)`(`t` ≤ 10 ms) | シグナルでハンドラが実行されても、残りの時間を眠る(PEP 475)。ハンドラが例外を送出したときだけ、`sleep` が中断される | **採用**。`t` が短いので、シグナルの検知の遅れは `t` 以下 |

待機ループの手順(骨格。実装コードではない):

```text
forwarded = False
interval = 0.001
loop:
    try: rc = child.poll()                       # 終了していれば回収される
    except OSError: 結果 = 起動失敗(手順 8); break
    if rc is not None: 結果 = 4.4 節で取り出す; break
    n = sig.number()                             # 1 回だけ読み、その値を使う(9.2 節)
    if n is not None and not forwarded:
        forwarded = True                         # 成否にかかわらず 1 回でやめる
        platform.forward(child, n)               # OSError は握りつぶす(7.2 節)
    sleep(interval)                              # time.sleep(差し替え可能)
    interval = min(interval * 2, 0.010)
```

- ループの 1 周目で、`poll()` の直後に合図を読む。したがって、`Popen` が返る前に立った合図は、1 周目で転送される。
- **子の終了と、`pid` の再利用**: 子は、`conductor`(メインスレッド)が `poll()` で回収するまで、ゾンビとして `pid` を保持する。回収は、`poll()` の戻りで 1 回だけ起こる。したがって、ループの中で `pid` が再利用されることはなく、「確かめてから送るまでのあいだに、別のプロセスに `pid` が移る」競合は起きない(単一のスレッドで回収し、`SIGCHLD` を無視しない場合。4.4 節)。転送の直前に子が終わっていても、ゾンビへの送信は無害で、何も起こらない。
- **`pid` を自分で保持して `os.kill(pid, ...)` に渡さない**【規定】。必ず `Popen.send_signal` を通す。`send_signal` は、内部で `poll()` を呼び、すでに終了していれば(`returncode` が確定していれば)何も送らない【実測: 終了したが未回収の子に `send_signal(SIGTERM)` を呼ぶと、`returncode` が確定して(0)、シグナルは送られなかった。回収済みの子への呼び出しも、例外にならず何も起きなかった。3.14、3.15、3.9】。
- `Popen.send_signal` が、内部の `poll()` で子を回収した場合も、次の周の `poll()` が、確定した `returncode` を返す(`returncode` は `Popen` が保持する)。結果の取り出しは、ループの `poll()` の戻りの 1 か所だけ。

### 5.3 テスト用の差し替え口【Python の設計】

`run_job` は、キーワード引数で、`Popen` に相当する関数、プラットフォーム層、`sleep` を差し替えられる。

```python
class ChildLike(Protocol):
    @property
    def pid(self) -> int: ...
    def poll(self) -> int | None: ...
    def send_signal(self, sig: int) -> None: ...

# 第 1 引数は list[str](UNIX 系と、Windows の通常の起動)または str(Windows の .cmd の起動)
Spawner = Callable[[Sequence[str] | str, SpawnOptions], ChildLike]

@dataclass(frozen=True, slots=True)
class SpawnOptions:
    start_new_session: bool            # UNIX 系 True、Windows False
    # stdin/stdout/stderr=None、shell=False、close_fds=True、restore_signals=True は、本番の Spawner が固定で渡す
```

偽の `ChildLike`(クラス)で、次を単体で検査できる(11 章): 転送が 1 回だけ、先に終了した子へ転送しない、2 回目以降の合図で追加の転送をしない、`Spawner` が起動前に合図が立っていた場合に起動しない(手順 3)、`poll()` が `OSError` を送出したとき起動失敗になる、`Spawner` が `OSError`/`ValueError` を送出したとき起動失敗になる、`sleep` に渡される間隔の列(0.001、0.002、0.004、0.008、0.010、0.010、…)。

---

## 6. 中断の合図(`Interrupt`)

「中断の合図」は、`conductor` がシグナルを受け取ったことを、ジョブを待つ側へ伝える値。

### 6.1 規定

| 項目 | 規定 |
|------|------|
| 内容 | 「立っているか」と、立っているなら「最初のシグナルの番号」 |
| 初期 | 立っていない |
| 立てる(番号 n) | 立っていなければ n を保存する。**すでに立っていれば何もしない**(最初の番号が残る)。番号は任意の整数でよく、負数や非常に大きい値、0 でも、「立った」ことは記録される |
| 性質 | 立ったら下りない。待つ側は読むだけ |
| 等価性 | 番号が等しければ等しい |

### 6.2 Python の型の骨格【Python の設計】

```python
class Interrupt:
    """中断の合図。Interrupt() で作る。スレッドを使わないので、ロックを持たない。"""

    def trigger(self, n: int) -> None:
        """立っていなければ n を保存する。立っていれば何もしない。シグナルハンドラから呼ばれる。"""

    def number(self) -> int | None:
        """立っていれば最初の番号、立っていなければ None。読むだけ。"""
```

- 内部は、保存先を 1 か所(例: 要素が高々 1 つの `dict` や `list`)にする。有無と番号を、別々の 2 つの変数に分けない(途中の状態を見せないため)。「立っていなければ保存する」は、`self._cell.setdefault("n", n)` のように、1 回の呼び出しで行うことを推奨する【目安】。Python のシグナルハンドラは、メインスレッドのバイトコードの境界で実行され、別のシグナルのハンドラの実行中にも、別のハンドラが割り込みうる(ネスト)。`if 未設定: 設定` の 2 命令に分けると、2 つのハンドラが互いに割り込んで、どちらの番号が残るかが定まらなくなるが、どちらが残っても「最初」の規定(7.2 節。ほぼ同時の到着は規定しない)に反しない。`setdefault` は、1 回の C の呼び出しであり、割り込まれない(CPython の実装による。未確認)。
- `number()` の戻りの `None` と、番号 0 を区別する(`if n is not None` で判定する。`if n:` と書かない。0 は偽である)【規定】。
- 同じ `Interrupt` を、`run_job`、`wait_retry`、`cli` の流れが共有する。`cli` が作り、`Signals.start` に渡す。
- 待つ側は、`threading.Event` も `Condition` も使わない(スレッドを使わないため。9 章)。

---

## 7. シグナル処理

### 7.1 準備(両 OS 共通)

- `run` と `pipeline` だけが、**定義の検査より前**に、シグナルの監視を始める【規定】。
  `watch`・`reset`・`validate`・`convert` は監視せず(`signal.signal` を呼ばず)、OS の既定の動作で終わる(7.6 節に、`KeyboardInterrupt` の扱いがある)。
- `cli` の流れが、`Interrupt()` で合図を 1 つ作り、`Deps.signals.start(sig)` に渡す。
- 監視の登録は **プロセスで 1 回だけ**。2 回目の呼び出しは「すでに登録済み」(`"alreadyRegistered"`)として、流れを成功と同様に続ける
  (2 回目に渡した合図には配線されない)。本番の実装は、モジュール内の変数(真偽値)で 1 回だけにする。
- **`signal.signal` は、メインスレッドからだけ呼べる**【実測: 別のスレッドから呼ぶと `ValueError: signal only works in main thread of the main interpreter`。3.14、3.15、3.9】。したがって、`conductor` 全体が、メインスレッドで動く(スレッドを使わない。9 章)。監視の開始が、メインスレッド以外で呼ばれたとき(組み込んだ場合など)は、`ValueError` を、登録の失敗として扱う。
- 登録は `signal.signal(番号, ハンドラ)` で行う。これが例外を送出したとき(`ValueError`: 範囲外の番号、メインスレッドでない。`OSError`(`errno` 22): `SIGKILL`・`SIGSTOP` のような登録できないシグナル【実測】)は、すでに登録したハンドラを元に戻し(下記)、「登録の失敗」を返す。このとき、標準エラーに
  `conductor: error: failed to set up signal handling` を書き、終了コード **70** で終わる(定義の検査もジョブの起動もしない)。
  登録に失敗した場合は、次の登録を許す(登録済みの印を戻す)。
- **復元**: `signal.signal` は、直前のハンドラを返す(`signal.SIG_DFL`、`signal.SIG_IGN`、呼び出し可能なもの、または Python 以外で設定されたときの `None`)【実測: `signal.getsignal(SIGINT)` は、起動直後は `signal.default_int_handler`】。監視は、登録のたびに、返された直前のハンドラを保存し、`stop()` で、保存した値を `signal.signal` で戻す。直前のハンドラが `None` のときは、`signal.SIG_DFL` に戻す(`signal.signal(n, None)` は `TypeError`【実測】)。
- `stop()` を返す。**本番では呼ばない**(プロセスの終了まで動き続ける)。理由: 最後のステートの書き込みの途中に、元のハンドラ(SIGINT の `KeyboardInterrupt` や、SIGTERM の既定の終了)に戻すと、書き込みが中断される。テストでは `stop()` を呼び、登録したハンドラを元に戻して、ハンドラを残さない。
- シグナルのハンドラの中では、`Interrupt.trigger` を呼ぶだけにする(出力、ステートの書き込み、`send_signal`、`logging`、例外の送出をしない)【規定】。ハンドラの中の処理が長いと、ほかのシグナルのハンドラが割り込み、ハンドラ自身が再入する(重い処理や入出力は再入に弱い)。
- ハンドラは、モジュールの関数(またはクロージャ)で、`(signum: int, frame: FrameType | None) -> None` の形にする。番号は、引数 `signum` をそのまま `trigger` に渡す(`int(signum)`。`signal.Signals` の `IntEnum` ではなく、素の `int` にして保存する)。

### 7.2 UNIX 系(Linux、macOS)

受け取るシグナルは **SIGHUP(1)、SIGINT(2)、SIGQUIT(3)、SIGTERM(15)** の 4 つ【規定】。受け取るたびに、`trigger` を、そのシグナルの番号で呼ぶ。
登録後、これらのシグナルでは、Python の既定の動作(SIGINT は `KeyboardInterrupt`、そのほかは OS の既定の終了)をしなくなる。

| 項目 | 規定 |
|------|------|
| 受け取る集合 | `(signal.SIGHUP, signal.SIGINT, signal.SIGQUIT, signal.SIGTERM)`(番号 1、2、3、15。両 OS で同じ値) |
| 登録 | 各シグナルに `signal.signal(シグナル, ハンドラ)` |
| 起動時に SIGHUP が無視されている場合 | `nohup` で起動すると、SIGHUP は `SIG_IGN` で始まる【実測: `signal.getsignal(SIGHUP)` が 1(`SIG_IGN`)】。`signal.signal(SIGHUP, ハンドラ)` は、これを上書きして、ハンドラを登録する【実測】。したがって、4 つのシグナルをいつも受け取る規定を満たす。子には、ハンドラは引き継がれず、既定の動作に戻る(`exec` で、ハンドラの登録は既定に戻る。無視の設定(`SIG_IGN`)だけが引き継がれる)【実測: 親が SIGHUP のハンドラを持つとき、子は SIGHUP で終了した(`returncode` -1)。親が `SIG_IGN` のままのときは、子も無視した】。`conductor` は、ハンドラを登録してから子を起動するので、前者(子は既定の動作)になる |
| 転送 | `Popen.send_signal(番号)`(4.2・5.2 節)。`os.kill(pid, ...)`、`os.killpg`、`Popen.kill()`、`Popen.terminate()` ではない |
| SIGPIPE | Python は、起動時に SIGPIPE を無視(`SIG_IGN`)にする【実測: `signal.getsignal(SIGPIPE)` が 1】。`Popen` の `restore_signals=True`(既定。明示する)により、子では SIGPIPE が既定に戻る【実測: `restore_signals=False` では、子の `yes | head -1` が `Broken pipe` の誤りを出し、`True` では出なかった】。ジョブの動作を、シェルから直接起動した場合と揃える |

時系列(ジョブの実行中に、利用者が `kill -TERM <conductor の pid>` したとき):

1. `conductor` が SIGTERM を受け取る。C のハンドラがフラグを立て、メインスレッドが、次のバイトコードの境界(`sleep` の中なら、`sleep` が EINTR で起きたとき)で、Python のハンドラを実行する。ハンドラが `trigger(15)` を呼ぶ。
2. 待機ループは、次の周(最大 10 ms 後)で `poll()` が `None` を返し、合図が立っているのを見る【実測: 子の実行中に、0.205 秒にハンドラが実行され、ポーリングが 0.213 秒に合図を見た。間隔は 1、2、4、8、10 ms】。
3. 子の pid に、**同じシグナルを 1 回**送る(`platform.forward`)。送る前に次を検査する。
   - `child.pid` が整数で 0 より大きい(0 や負にならない。プロセスグループへの送信にならない)。
   - シグナルが `{SIGHUP, SIGINT, SIGQUIT, SIGTERM}`(番号 `{1, 2, 3, 15}`)のどれか(`SIGKILL`、`SIGSTOP`、`SIGCONT`、`SIGUSR1`、0、負、9999 は送らない)。
   - 送信先は、**子プロセスの pid だけ**。子の子孫(`sh -c` が起こした孫など)には、`conductor` は送らない。`os.killpg` と、負の pid の `os.kill` を、製品のコードに書かない。
   - 送信の失敗(`ProcessLookupError`、`PermissionError` を含む `OSError`)は無視する。
4. 子は、受け取ったシグナルの既定の動作で終了するか、自分で処理する。`conductor` は待ち続ける。2 回目以降のシグナルでは、何も追加しない【実測: 同じジョブに SIGINT を 0.1 秒おいて 2 回送っても、子へ届いたのは 1 回(`forwards=1`)】。
5. 子の終了を観測する(`poll()` が値を返す)。`returncode` を読み(シグナル終了なら負の値)、直後に合図が立っているので
   `interrupted = True`、`signal_number = 15` を足す【実測: 子が TERM を捕まえて `exit 7` で終わったとき、`exit_code = 7` のまま、`signal_number = 15`。`conductor` の終了コードは 143】。
6. 分類で「エラー、`128 + 15 = 143`」となり、流れの制御はこの結果をステートに**エラーとして**記録する(再開位置はそのジョブ)。
7. `conductor` は **143** で終わる(`pipeline` は次のジョブを起動しない)。

その他:

- 子は、`start_new_session=True` で作る **新しいプロセスグループ**(グループ ID は子の pid)で動く【規定】(4.7 節)。端末の
  Ctrl+C(SIGINT)は、フォアグラウンドのプロセスグループに配られるため、ジョブには直接届かず、`conductor` が受けて 1 回転送する
  (SIGINT のとき、ジョブは `terminated_by_signal = 2` で終わるのが典型。【実測: 親に `kill -INT` を送ると、子が INT を 1 回受けて終了した】)。
  端末から Ctrl+C を押した場合の実機の確認は、未確認(擬似端末の下で、`\x03` を送る確認をしていない)。
- 転送の許可集合にない番号で合図が立った場合(`conductor` 本体の監視は 4 つだけを受けるため、通常は起こらない。`run_job` を単独でテストする場合のみ)は、
  送信せず、子が自分で終わるのを待ち、終わりを観測した後に `interrupted = True`、`signal_number = その番号` を足す
  (子の終了コードは `exit_code` に残るが、分類では見ない)。
- 時間を空けて複数のシグナルが届いたときは、到着順の最初の番号が残る(INT のあとの HUP でも 2)。
  ほぼ同時に届いたときに、どれが「最初」になるかは**規定しない**(Python は、保留中のシグナルを、番号の小さい順に処理する実装であるとされるが、未確認)。
- 終了コードの例: SIGHUP は 129、SIGINT は 130、SIGQUIT は 131、SIGTERM は 143。

### 7.3 Windows

受け取るのは **Ctrl+C(`signal.SIGINT`、番号 2)と Ctrl+Break(`signal.SIGBREAK`)** の 2 つ【規定。Python 版の取り決め】。`SIGBREAK` の番号は `int(signal.SIGBREAK)`(Windows の Python では 21 と思われる。未確認)。その番号で `trigger` を呼ぶ。したがって、Ctrl+Break の終了コードは `128 + int(signal.SIGBREAK)`(21 なら 149。未確認)。

| 項目 | 規定 |
|------|------|
| 受け取る集合 | `(signal.SIGINT, signal.SIGBREAK)`。`signal.SIGBREAK` は、Windows の Python だけにある(UNIX 系では `AttributeError`【実測: macOS で `hasattr(signal, "SIGBREAK")` が `False`】)。プラットフォーム層が、Windows の分岐の内側でだけ読む(`getattr` で、UNIX 系でも、Windows の分岐を単体テストできるようにする) |
| 登録 | `signal.signal(シグナル, ハンドラ)`。UNIX 系と同じ仕組み(1 回だけ。`trigger`)。メインスレッドだけ |
| 転送 | **何もしない**。`platform.forward` の本体は空にし、`Popen.send_signal`、`Popen.terminate`、`Popen.kill`、`os.kill` を**呼ばない**。Windows の `Popen.send_signal(SIGTERM)`、`terminate()`、`kill()` は、子を強制終了する(`TerminateProcess`)、とされるため、呼ぶと「ジョブを強制終了しない」規定に反する【規定】。`CTRL_C_EVENT`・`CTRL_BREAK_EVENT` を `send_signal` で送る方法は、プロセスグループの指定が要り、この版では使わない(下記のテストの用途を除く) |
| ジョブの起動 | `start_new_session`、`creationflags` を指定しない。新しいプロセスグループ・新しいコンソールを作らない(4.7 節) |
| `SIGTERM`、`SIGHUP`、`SIGQUIT` | 登録しない。Windows では、プロセスへ `SIGTERM` を配る仕組みが通常なく、`signal.signal` が受け付けても、イベントとして発生しない、とされる(未確認)。`SIGTERM` は、`Popen.terminate()` が使う番号であり、この版は登録しない |
| `time.sleep` | Ctrl+C を受けたときの `sleep` の中断の挙動は、UNIX 系と異なる可能性がある(未確認)。この版は、`sleep` を最大 50 ms に刻み、中断の有無に依らず、次の周で合図を読む(9.1 節)ので、挙動の差が結果に影響しない |

時系列(ジョブの実行中に、利用者が Ctrl+C を押したとき):

1. ジョブは `conductor` と **同じコンソール**で起動されている。Ctrl+C は、OS が、同じコンソールを共有するプロセス(`conductor` とジョブ)の両方に配る。
2. `conductor` のハンドラが `trigger(2)` を呼ぶ。
3. 待機ループは転送を試みるが、Windows の転送は **何もしない**【規定】。
4. ジョブは、OS から受け取った Ctrl+C で(既定の動作で)終了する。
5. 終了を観測する。シグナル終了の番号は常に `None`、`exit_code` は OS の終了コード(Ctrl+C による終了コード。4.4 節の符号つきへの変換を通したもの)。直後に合図が立っているので
   `interrupted = True`、`signal_number = 2`。
6. 分類で「エラー、`128 + 2 = 130`」。ステートに**エラーとして**記録し、**130** で終わる。

Ctrl+Break の時系列も同じで、`signal_number = int(signal.SIGBREAK)` になる。

その他:

- コンソールを閉じたときの通知(`CTRL_CLOSE_EVENT`)は、この版の対象外(OS の既定の動作)。
- **自動テストの方法(Ctrl+Break)**: Python の公式の文書では、`Popen(..., creationflags=subprocess.CREATE_NEW_PROCESS_GROUP)` で起動した子に、`Popen.send_signal(signal.CTRL_BREAK_EVENT)` で Ctrl+Break を送れる(未確認。実機で確かめていない)。受け入れテストは、`conductor` を `CREATE_NEW_PROCESS_GROUP` で起動し、`CTRL_BREAK_EVENT` を送って、`conductor` が SIGBREAK で終了コード `128 + int(signal.SIGBREAK)` を返すこと、ステートに中断が記録されること、ジョブの子が残らないこと、を確かめる。`conductor` が起動した子は、`conductor` のプロセスグループを引き継ぐので、イベントは `conductor` とジョブの両方に配られる(未確認)。これは、テストの起動の方法であり、製品のコードは `creationflags` を使わない(4.7 節)。
- **Ctrl+C**: 自動の試験環境(CI の Windows ランナー)には、対話的なコンソールがなく、特定のプロセスだけに Ctrl+C を送る方法がない(`CTRL_C_EVENT` は、新しいプロセスグループの子には効かない、とされる。未確認)。この経路は自動テストにしない。
  リリース前に実機で手動確認する(確認項目: 終了コード 130、ジョブの子プロセスが残らない、
  `watch` で中断したジョブが `error` で次が `pending`、`run -v` の表示に `interrupted by signal 2`)。手順は 11 章の手動の受け入れテストにある。

### 7.4 合図が立つ時点ごとの動作(流れの制御)

| 合図が立った時点 | 動作 | ステートの記録 | 終了コード |
|------------------|------|----------------|------------|
| `run_job` の呼び出しの前(`started = False`) | ジョブを起動しない | 分類なし。エラーとしては記録しない。再開位置はそのジョブ | `128 + 番号` |
| ジョブの実行中 | 1 回転送(UNIX 系)。ジョブの終了を待つ | そのジョブを**エラー**として記録(ジョブが 0 で終わっても) | `128 + 番号` |
| ジョブの終了の観測の直後(転送が間に合わなかった場合を含む) | 起動済みで `interrupted = True`(上の行と同じ扱い) | エラー | `128 + 番号` |
| `pipeline` のジョブとジョブのあいだ(ジョブの結果は成功か警告で、完了ではない) | 次のジョブを起動しない | 記述順で、まだ起動していない最初のジョブ(エラーのジョブが先にあればそれ)を再開位置にして、中断を記録。エラーとは記録しない。`-v` で `interrupted by signal N` | `128 + 番号` |
| `pipeline` の最後のジョブが完了した直後 | 完了の処理(`--remove-on-complete` ならステートを削除)をして終わる | 完了 | 0(合図は無視される) |
| リトライの待機中 | 待機を打ち切る。`started = False`、`interrupted = True`、`signal_number = 番号` の実行結果を作る。次の実行はしない | 分類なし、再開位置はそのジョブ。エラーとしては記録しない | `128 + 番号` |
| 定義の検査・ステートの読み込み・照合の途中 | 検査は最後まで進める | (定義の誤り 64、ステートの誤り 65、拒否 66、スキップ 0 の終了コードが、中断の終了コードより優先する) | 各検査の値 |

### 7.5 リトライの待機

関数: `wait_retry(secs: int, sig: Interrupt, *, sleep: Callable[[float], None] = time.sleep, monotonic_ns: Callable[[], int] = time.monotonic_ns) -> WaitOutcome`。

```python
@dataclass(frozen=True, slots=True)
class Elapsed:
    interrupted: Literal[False] = False        # 経過

@dataclass(frozen=True, slots=True)
class Interrupted:
    number: int                                # 最初の番号
    interrupted: Literal[True] = True          # 中断

WaitOutcome = Elapsed | Interrupted
```

OS 固有のコードを持たない。例外を送出しない(`secs` は 0 以上の `int` であること)。

手順:

1. 合図が立っていれば(`sig.number()` が `None` でない)、すぐ「中断(最初の番号)」を返す。
2. `secs == 0` なら、すぐ「経過」を返す(手順 1 で、立っていなければ)。
3. 待ち時間を **ナノ秒の `int`** で求める: `total_ns = secs * 1_000_000_000`。Python の `int` は任意の精度なので、あふれない【規定】。上限は、リトライの待ち時間の飽和(64 ビット符号なしの上限 `2**64 - 1`。05 章)が `flow` で済んでいるので、ここでは増やさない。
4. 期限を、単調な時計(`time.monotonic_ns()`)で求める: `deadline = monotonic_ns() + total_ns`。壁時計(`time.time()`)を使わない(時刻の調整の影響を受けない)。
5. 次を繰り返す。
   1. `n = sig.number()`。立っていれば、「中断(n)」を返す。
   2. `remaining_ns = deadline - monotonic_ns()`。`remaining_ns <= 0` なら「経過」を返す。
   3. `chunk_ns = min(remaining_ns, 50_000_000)`(50 ms)。`sleep(chunk_ns / 1_000_000_000)` を呼ぶ(`sleep` の引数は、高々 0.05 の `float`)。
6. 合図から返るまでは、50 ms と、`sleep` の分解能の範囲である。長い待ちでも、合図から **1 秒以内**に返る【規定】
   (例: 3600 秒の待機に、200 ms 後に合図を立てて、全体が 1.2 秒未満)。

`sleep` の引数に、残り時間の全体を渡さない【規定】。理由は 2 つある。(1) 合図がハンドラで立っても、`time.sleep` は、残りの時間を眠り続ける(PEP 475)【実測: 1.0 秒の `sleep` に、0.2 秒で SIGTERM を送ると、`sleep` は 1.00 秒後に戻り、ハンドラは 0.21 秒に実行された】。(2) `time.sleep` に大きな値を渡すと、`OverflowError` になる【実測: 3.14 と 3.15 で、9223372036.0 秒(約 292 年)までは受け付け、`2**63`、`2**64 - 1`、`1e300`、9223372037.0 は `OverflowError: timestamp out of range for C PyTime_t`。3.9 では 9223372036.0 が `OSError`(`EINVAL`)、`2**63` は `OverflowError`】。リトライの待ち時間は、`2**64 - 1` 秒まで飽和しうるので、1 回の `sleep` に渡す値を、50 ms 以下に刻む。

複数の合図は、最初の番号を返す。待機の時計は、`monotonic_ns` と `sleep` を差し替えれば、テストで実際には待たずに検査できる(11 章)。ただし `cli` のテストは、`Process.wait`(`Deps`)の偽の実装で、待ち時間を記録だけして実際には待たない([02-architecture.md](02-architecture.md))。システムの休止中に、単調な時計が進むかは、OS により異なる(未確認)。

### 7.6 `KeyboardInterrupt` の扱い【Python の設計】

`run` と `pipeline` では、SIGINT の `KeyboardInterrupt` は、監視の登録(7.1 節)により、起きない(SIGINT のハンドラが、Python の既定のもの(`signal.default_int_handler`)から置き換わる)。それ以外では、起きうる。

| 場面 | 動作 |
|------|------|
| `watch`・`reset`・`validate`・`convert`(監視しない) | SIGINT は Python の既定の動作で、`KeyboardInterrupt` が送出される。入口(`__main__.main`)が捕まえ、**何も出力せずに**、OS の既定の動作と同じになるよう終わる。UNIX 系: `signal.signal(SIGINT, SIG_DFL)` に戻してから、`os.kill(os.getpid(), SIGINT)` を自分に送り、シグナルで終了する(シェルから見て 130。親が `wait` で状態を読む場合も、シグナル終了として見える)。送信が戻ってきた場合(通常は起きない)は 130 を返す。Windows: 130 を返す(Windows の OS の既定の動作は、終了コード `0xC000013A`。この差は、利用者から見て小さいので、130 に揃える。未確認)。この動作は、プラットフォーム層の関数 1 つ(`exit_by_interrupt()`)に置く |
| `run`・`pipeline` の、監視の登録の前(起動の直後の短い区間) | 同じ(上の行)。ステートには、まだ触れていない |
| `run`・`pipeline` の、監視の登録の後 | `KeyboardInterrupt` は起きない(合図で伝わる) |
| 内部の誤りの捕捉(10 章)との関係 | `KeyboardInterrupt` は `Exception` の下位ではなく、`BaseException` の下位。入口の捕捉は、`KeyboardInterrupt` を、内部の誤り(70)として扱わない |

---

## 8. プラットフォーム差と OS 分岐の規則

### 8.1 差の一覧

| 項目 | UNIX 系(Linux、macOS) | Windows |
|------|------------------------|---------|
| ジョブのプロセスグループ | 新しいセッション・グループ(`start_new_session=True`) | 指定しない(同じコンソール) |
| 監視するシグナル | SIGHUP・SIGINT・SIGQUIT・SIGTERM | SIGINT(Ctrl+C)・SIGBREAK(Ctrl+Break) |
| ジョブへの転送 | 子の pid に、`Popen.send_signal` で 1 回(許可集合のみ) | しない(OS が Ctrl+C と Ctrl+Break を配る。`send_signal`・`terminate`・`kill` は呼ばない) |
| シグナル終了の読み取り | `returncode` が負なら、`-returncode` がシグナル番号 | なし(終了はすべて終了コード。`returncode` を符号つき 32 ビットに直す) |
| 終了コードの範囲 | 下位 8 ビット | 32 ビット符号つきのまま(`conductor` が報告する値は、0〜255 に合わせる) |
| `SIGCHLD` が無視されているとき | `SIG_DFL` に直す(4.4 節) | なし |
| ステート書き込み後のディレクトリの同期 | ディレクトリを開いて同期する。失敗は無視。親が空なら `.` | しない(06 章) |
| 端末の色 | 使える(標準出力が端末で `NO_COLOR` が未設定か空のとき) | 使わない(色なし。08 章) |
| `.cmd` / `.bat` の起動 | (なし) | 4.8 節の安全策 |
| `KeyboardInterrupt` の後始末 | SIGINT を自分に送って、シグナルで終わる | 130 を返す(7.6 節) |

### 8.2 分岐を置く場所の規則【規定】

`sys.platform`(`os.name`、`platform.system()`、`platform.platform()` を含む)を読んで分岐するのは、**`src/conductor/runner/platform.py` の 1 か所だけ**。ほかのファイルは、これらを読まず、OS 名の文字列(`"win32"`、`"linux"`、`"darwin"`)も書かない。

```python
PlatformKind = Literal["posix", "windows"]

class Platform(Protocol):
    kind: PlatformKind
    # Popen に渡す第 1 引数とオプションを決める。拒否は PrepareErr(起動失敗として扱う)。
    def prepare_spawn(self, line: CommandLine, lookup_env: Callable[[str], str | None]) -> PrepareResult: ...
    # returncode から、終了コードとシグナル番号を取り出す(4.4 節)。
    def read_exit(self, returncode: int) -> ExitReading: ...
    # 転送(UNIX 系は許可集合の検査のあと send_signal。Windows は何もしない)。
    def forward(self, child: ChildLike, n: int) -> None: ...
    # signal.signal に渡すシグナルの集合。
    watched_signals: tuple[int, ...]
    # Popen の前に、SIGCHLD が無視されていれば既定に直す(UNIX 系のみ。Windows は何もしない)。
    def prepare_child_reaping(self) -> None: ...

def detect_platform() -> PlatformKind: ...      # sys.platform を読む唯一の関数。"win32" なら windows、それ以外は posix
def platform_for(kind: PlatformKind) -> Platform: ...
def exit_by_interrupt() -> int: ...            # KeyboardInterrupt の後始末(7.6 節)
```

- `platform_for("windows")` と `platform_for("posix")` は、どの OS でも作れる(単体テストで、Linux の 1 台から Windows の分岐を検査できる。`prepare_spawn` の `.cmd` の組み立て、`read_exit` の符号つきへの変換、`forward` の空の動作)。実際の起動は、実行している OS のものだけが動く。ただし、Windows の分岐は、`signal.SIGBREAK` を `getattr` で読む(UNIX 系で `AttributeError` にならないように)。
- `detect_platform()` の `"win32"` 以外(`linux`、`darwin`、`freebsd`、`cygwin` など)は、すべて `posix` として扱う。対象の OS は Linux、macOS、Windows である。それ以外は、動作を確かめていない(未確認)。
- ステートの書き込み(ディレクトリの同期の有無)と、色の判定は、`cli` が `Deps.env.platform()` で得た `PlatformKind` を、必要な部品に引数で渡す(`state` と `report` は、`runner` の関数を呼ばず、`PlatformKind` の型だけを読む。[02-architecture.md](02-architecture.md))。
- 点検: `sys.platform`、`os.name`、`platform.system`、`"win32"`、`"linux"`、`"darwin"` が、`src/conductor/runner/platform.py` 以外に現れないことを、ソースの検索(または `ast` による走査)で確かめる(11 章 AC-420)。テスト(`tests/`)は対象外。
- ネットワークに接続しない(実装がネットワークの機能を持たない)。`socket`、`http`、`urllib`、`ssl` などを import しない。
- 製品のコードは、外部の入力(引数・環境・ファイル・OS の結果)に対して、例外を送出する書き方をしない(失敗は値で返す。[02-architecture.md](02-architecture.md) の 9 章)。`assert` を、外部の入力の検査に使わない(`python -O` で消える)。

---

## 9. 同期モデルの規則と、順序の保証

### 9.1 スレッドを使わない規則【規定】

`conductor` は、1 つのスレッド(メインスレッド)で動く。子プロセスの終了、シグナル、時間の経過は、すべて、メインスレッドの周期的な確認(`poll()` と `sleep`)と、シグナルハンドラ(フラグ)で伝わる。

| 規則 | 内容 |
|------|------|
| `asyncio` を使わない | イベントループ、`asyncio.subprocess`、子の監視器(`ChildWatcher`)を使わない。シグナルは `loop.add_signal_handler` ではなく、`signal.signal` で受ける |
| スレッドを使わない | `threading.Thread`、`threading.Timer`、`concurrent.futures`、`multiprocessing` を使わない(製品のコード。テストの補助は除く)。理由: `signal.signal` がメインスレッドだけに限られること、子の回収を 1 か所にして `pid` の再利用を避けること(5.2 節) |
| `fork` を直接呼ばない | `os.fork`、`os.forkpty` を使わない。子の起動は `Popen` だけ。`preexec_fn` を使わない(4.2 節) |
| 長い待ちの刻み | 待ち(`sleep`)は、1 回あたり 50 ms 以下(リトライの待機)、または 10 ms 以下(待機ループ)に刻む。合図の確認の周期を、これで保つ |
| ビジーループ | 時計を見て回るだけのループ(`sleep` なしの `while`)を書かない |
| 同期の子プロセスの API | `subprocess.run`、`Popen.wait()`(引数なし)、`Popen.communicate` を使わない(4.2 節、5.2 節) |
| 長い同期処理 | 子の実行中は、`run_job` の待機ループだけが動く(定義の読み込みと検査は、子を起動する前に終わる)。待機ループ以外の処理で、数十 ms を超える CPU の仕事を、子の実行中に行わない |
| シグナルハンドラ | `signal.signal` に登録する関数は、`Interrupt.trigger` を呼ぶだけ。中で出力、ファイルの書き込み、`send_signal`、ロック、`logging` を使わない。例外を送出しない |
| ハンドラが実行される時点 | Python のハンドラは、メインスレッドが、次のバイトコードの境界に達したとき、または EINTR で中断されたシステムコールを再試行する前に実行される【実測: `sleep`、`Popen.wait()` の中で、ハンドラが、シグナルの到着から 10 ms 以内に実行された(0.2 秒のタイマーで送ったシグナルに対し、0.205 〜 0.21 秒)】。長い C の呼び出し(ファイルの `fsync` など)の途中では、その呼び出しが戻る(または EINTR で中断される)まで、実行されない(未確認)。したがって、ハンドラの実行は、遅れうる前提で作る |
| EINTR | `time.sleep`、`Popen.poll`、`os.read`、`os.write`、`os.replace`、`os.fsync` などは、EINTR で中断されたとき、ハンドラが例外を送出しない限り、自動で再試行される(PEP 475)。自前で `InterruptedError` の再試行を書かない。ハンドラが例外を送出しない規則(上の行)により、`InterruptedError` が見えることはない |
| 合図の読み取り | `sig.number()` は、ハンドラが実行されるたびに変わりうる(バイトコードの境界で割り込まれる)。同じ判断のなかで 2 回読んで食い違いを起こさない。1 回読んで、その値を使う(5.2 節、7.5 節の手順) |

### 9.2 順序の保証【規定】

| 保証 | 内容 |
|------|------|
| 出力の順序 | 標準出力・標準エラーへの書き込みは、`cli` の 1 か所の関数(`Sink`)を通し、**書くたびに `flush()` する**(`sys.stdout.write(text)` のあとに `sys.stdout.flush()`、標準エラーも同じ)。標準出力と標準エラーの書き込みの順序が、呼び出しの順序と一致する(標準出力が端末でないとき、Python は標準出力をブロック単位でバッファリングするため、`flush()` を省くと、標準エラーより遅れる。標準エラーは、既定で行単位または即時に書き出される。詳細は 08 章) |
| 子の出力との順序 | `conductor` は、子を起動する前に、自分の出力を `flush()` で書き切る(4.1 節の手順 5)。子の実行中に、リトライの知らせ以外の出力を書かない。子の出力(子が直接ファイル記述子へ書く)と、`conductor` の出力が、混ざらない。リトライの知らせは、子の終了のあと、次の起動の前に、書いて `flush()` する |
| ステートの順序 | ステートの書き込み(一時ファイルへの書き込み、`fsync`、`os.replace`、必要ならディレクトリの同期)の完了を待ってから、次のジョブを起動する。「記録して(Durable)から次へ」を破らない(05・06 章) |
| 出力とステートの順序 | 05 章の順序に従う。ジョブごとに、`-v` の結果行などの表示を書き(`flush()` し)、そのあとステートを書く。順序を入れ替えない |
| 合図の読み取り | 9.1 節の「合図の読み取り」のとおり |
| 終了の前の flush | `main()` が終了コードを返す前に、標準出力と標準エラーを `flush()` する。`flush()` の失敗(`BrokenPipeError` など)は、出力の書き込みの失敗として扱う(08 章)。入口は `sys.exit(コード)` で終わる(インタープリターの終了時の `flush` の失敗は、別の終了コードを生みうる。08 章で、`BrokenPipeError` の扱いを決める) |
| ハンドラの副作用 | シグナルのハンドラは、出力もステートの書き込みもしない(7.1 節)。したがって、ハンドラが出力・ステートの順序を乱すことはない |

### 9.3 イベント源の一覧(誰が所有し、どう終わるか)【規定】

| イベント源 | 起こす者(所有者) | 数 | 読む | 書く | 終わる条件 |
|------------|-------------------|-----|------|------|------------|
| シグナルハンドラ(`signal.signal`) | `start_watcher`(`run`・`pipeline` のみ) | プロセスで 0 または 1 組(UNIX 系で最大 4 つ、Windows で最大 2 つ) | OS のシグナル | `Interrupt.trigger` だけ | プロセスの終了まで動き続ける(本番で `stop` は呼ばない)。テストでは `stop` で元に戻す |
| 子プロセス(`Popen`) | `run_job` | 実行中のジョブごとに 1(同時に最大 1) | 子の終了(`poll()`) | `run_job` の内側の状態だけ | `poll()` が値を返す(または起動失敗)まで。`run_job` が戻った時点で、子は回収済みである |
| 待ち(`time.sleep`) | `run_job` の待機ループと `wait_retry` | 同時に最大 1 | 時間の経過 | なし | 経過、または合図の確認で打ち切る |

- 出力(標準出力・標準エラー)とステートへの書き込みは、流れの制御(`cli`)の 1 本の呼び出しの連なりだけが行う。ハンドラは、それらに触れない。
- リソースのリークがないこと: テストの終わりに、シグナルハンドラが元に戻っている(`signal.getsignal` が、テストの前と同じ値)、`Popen` の `returncode` が確定している(ゾンビがない)、子プロセスが残っていない。

---

## 10. 想定外の内部の誤り

- `src/conductor/__main__.py` の入口に、想定外の内部の誤り(実装の不具合による例外)を捕まえる仕組みを、**最初に**入れる。具体的には、次の 2 つを、流れを呼ぶ前に設定する。
  1. `sys.excepthook` の置き換え(入口の外で起きた、捕まえられなかった例外のため。起動の途中の `import` の失敗など)。
  2. `main()` の内側の `try: ... except BaseException:`(`SystemExit` と `KeyboardInterrupt` を除く。`KeyboardInterrupt` は 7.6 節、`SystemExit` は、そのまま通す)。
- 両方とも、同じ関数(`fail_internal()`)を呼ぶ。内部の誤りが起きると、標準エラーに固定の英語の 1 行 `conductor: error: internal error`(ロケールを判定しない)を書き、終了コード **70** で終わる。標準エラーへの書き込みが失敗しても 70 で終わる。【規定】
  - 書き込みは、`os.write(2, b"conductor: error: internal error\n")` を、`try/except OSError` で囲んで行う【Python の設計】(`sys.stderr` が `None` や閉じている場合、エンコードの誤り、バッファの残りに左右されないため。`EAGAIN` などの失敗は無視する。書き込みが一部だけ成功した場合の再試行は、しない)。
  - `fail_internal()` は、繰り返し呼ばれても 1 行だけ出す(印を持つ)。
  - `main()` の内側では、`70` を返す(終了は、入口の `sys.exit` の 1 か所)。`sys.excepthook` の中では、`os._exit(70)` で終わる(インタープリターの通常の終了処理に戻ると、終了コードが 1 になるため。確認: 10 章 6.4 節。`sys.excepthook` に、標準エラーへ 1 行を書いて `os._exit(70)` を呼ぶ関数を設定すると、未捕捉の例外で終了コード 70 になった。Python 3.15.2・3.9.6、macOS)。
  - 例外の内容(メッセージ、スタックトレース)を、標準エラーにも標準出力にも出さない(`traceback.print_exc()` を呼ばない)。
- 入口は、流れの結果の終了コードで、プロセスを 1 回だけ終了させる(`sys.exit(main())`)。入口に、わざと例外を起こすコマンドを持たない。内部の誤りの検査は、`run` に差し替えた(例外を送出する)`Deps` を渡す単体テスト、または別のテスト用の起動(`tests/` の小さな手順書で、入口を差し替えて起動する)で行う([10-errors-and-exit-codes.md](10-errors-and-exit-codes.md))。
- ジョブの実行中に内部の誤りが起きると、子プロセスの終了を待たずに終わり、ジョブが孤児として残りうる(`Popen` の後始末は、子を終了させない。インタープリターの終了時に、`ResourceWarning` が出ることがあるが、既定では表示されない)。ステートは直前の完全な内容のままで、
  次の実行はそのジョブを再実行しうる(少なくとも 1 回の実行の範囲)。
- `conductor` が SIGKILL などで強制終了されたときも、ジョブは残って動き続けうる(検知も停止もしない)。
- 外部の入力(引数・環境・ファイル・OS の結果)に対して、例外を送出する書き方(添字の範囲外、`None` の属性の参照、`assert` による検査、`json` の例外の素通し)を製品のコードに書かない(失敗は値で返す)。標準ライブラリが例外を送出する場合(`Popen` の起動の失敗など)は、その場で `try/except` して値にする。

---

## 11. 検証の観点

| 確かめること | 検査の区分と置き場所の案 |
|--------------|--------------------------|
| 分割の全規則(3 章の表。受理・拒否。孤立サロゲートと補助面の文字を含む例) | `tests/runner/test_split.py` の表駆動の単体テスト(`pytest.mark.parametrize`) |
| ランダムな `command`(3 章の文字集合)で、成功なら `program` が空でない。約 30 万引数が分割できる(`RecursionError` にならない)。閉じない引用符は誤り | 同上(固定の種のランダム) |
| メタ文字が引数にそのまま渡る、配列の要素がそのまま渡る、環境変数と作業ディレクトリの引き継ぎ、起動失敗(見つからない・実行権限なし・NUL を含む引数・孤立サロゲートを含む引数)、終了コード(0・3・255・300) | `tests/jobs/` のテストジョブを起動する `runner` のテスト(実プロセス) |
| 標準入力・標準出力・標準エラーの引き継ぎ(`stdin=None` などであること) | 同上(ジョブが標準入力を読み、標準出力へ書く) |
| 起動前の合図で起動しない、範囲外の番号(負・0・9999)でも起動せず中断として扱う、終了の観測の直後の合図で `interrupted` | `run_job` の単体テスト(`trigger` してから呼ぶ) |
| `run_job` の転送: 転送が 1 回だけ、先に終了した子へ転送しない、2 回目以降の合図で追加の転送をしない、`poll()` と `Spawner` の例外が起動失敗になる、`sleep` の間隔の列 | 偽の `ChildLike` と `Spawner` による単体テスト(5.3 節) |
| `Popen.send_signal` が、終了済みの子に送らない(3.14、3.15、3.9 の実測と同じ) | 実プロセスのテスト(UNIX 系) |
| SIGTERM / SIGINT の転送、許可外の番号は送らない、新しいプロセスグループ(子の `PGID = PID`)、先に終わったジョブは転送なし(UNIX 系) | 実プロセスのテスト(UNIX 系) |
| 実バイナリ(`python -m conductor`)で SIGTERM → 143、ステートに再開位置、孤児なし。HUP のあとの INT → 129。`SIGCHLD` を無視して起動しても、ジョブの終了コードが正しい(Linux) | 受け入れテスト(UNIX 系。インタープリターの実行ファイル(`sys.executable`)を直接起動する) |
| `wait_retry` の 1 秒以内の打ち切り(3600 秒の待機に、200 ms 後の合図で、全体が 1.2 秒未満)、`secs` が `2**64 - 1` でも `OverflowError` にならない(合図で打ち切って検査する)、`monotonic_ns` と `sleep` の差し替え | `wait_retry` の単体テスト(合図は、`sleep` の偽が `trigger` を呼ぶ形で作る。スレッドを使わない) |
| 監視の登録(最初の番号が残る、2 回目の登録は `alreadyRegistered`、登録失敗で 70(メインスレッドでない、など)、`stop` でハンドラが元に戻る(直前が `None` のとき `SIG_DFL`)) | 単体テスト(登録失敗は、偽の `Signals`。メインスレッド以外の呼び出しは、テストの補助のスレッドで確かめる) |
| ジョブの実行中の CPU 時間の増分(1 秒後から 2.5 秒後)が 50 ms 以下(Linux) | 計測(強制。AC-403。実測する) |
| 標準出力が閉じたパイプでも、プロセスが未処理の例外で落ちない | 受け入れテスト(08 章と合わせる) |
| 内部の誤りで 70、固定の 1 行、`sys.excepthook` の経路でも 70 | 入口を差し替える単体テストと、別の起動のテスト |
| `KeyboardInterrupt` の扱い(7.6 節。`watch` などで SIGINT を受けたとき、出力なしでシグナル終了) | 実プロセスのテスト(UNIX 系) |
| Windows の Ctrl+Break(自動)、Ctrl+C(手動) | Windows の CI のテスト(7.3 節の方法。未確認)と、手動(11 章) |
| Windows の `.cmd` への危険な引数が注入にならない | Windows の CI のテスト。加えて `platform_for("windows").prepare_spawn` の単体テスト(どの OS でも動く) |
| `sys.platform` などが `platform.py` 以外にない | ソースの検索(AC-420) |

テストでスクリプトを書いてすぐに実行する場合、Linux で、実行ファイルを書いた直後の起動が「text file busy」(`ETXTBSY`)で失敗することがある(別の子プロセスの `fork` が、書き込み用のファイル記述子を一時的に持つため。一般に知られた事象)。テストのスクリプトは、あらかじめ作っておく(`tests/jobs/` に置く)か、起動前にファイルを閉じて待つ。Python での実際の発生と回避策の有効性は未確認。

---

## 12. 規定と実装の裁量の区別(まとめ)

| 項目 | 区別 |
|------|------|
| 分割の状態機械と、受理・拒否の例 | 【規定】 |
| シェルを介さない起動(`shell=False`)、作業ディレクトリ・環境変数・標準入出力の引き継ぎ(`stdin`/`stdout`/`stderr` を `None`、`cwd`・`env` を渡さない) | 【規定】 |
| 起動失敗(`OSError`・`ValueError`)の終了コード 127、分類の優先順位 | 【規定】 |
| 転送は 1 回だけ、許可する 4 つのシグナルだけ、子の pid だけ(`Popen.send_signal`)、終了済みには送らない | 【規定】 |
| UNIX 系でジョブを新しいプロセスグループで動かす | 【規定】(`start_new_session=True` は【Python の設計】) |
| Windows で転送をしない(`send_signal`・`terminate`・`kill` を呼ばない。OS が Ctrl+C と Ctrl+Break を配る) | 【規定】 |
| Windows で Ctrl+Break(`SIGBREAK`)も受け取る | 【規定】(Python 版の取り決め) |
| `run`/`pipeline` だけが、定義の検査より前にシグナルを準備する | 【規定】 |
| シグナルの準備の失敗は 70、内部の誤りは 70 | 【規定】 |
| リトライの待機が合図から 1 秒以内に返る | 【規定】 |
| `sleep` の引数を 50 ms 以下に刻む(`OverflowError` と PEP 475 への対処) | 【規定】 |
| スレッドを使わない、`asyncio` を使わない、ハンドラはフラグを立てるだけ | 【規定】 |
| `Popen.wait()`(引数なし)を使わず、`poll()` と `sleep` で待つ | 【Python の設計】 |
| 待機ループの確認の間隔(1・2・4・8・10 ms) | 【目安】 |
| リトライの待機の確認の間隔(50 ms 以下) | 【目安】 |
| 合図の内部の実装(`dict.setdefault` など) | 【任意】 |
| ほぼ同時に届いた複数のシグナルのどれが「最初」か | 規定しない |
| `KeyboardInterrupt` の後始末(UNIX 系は自分に SIGINT、Windows は 130) | 【Python の設計】 |
| Windows の `.cmd` の起動の形(`cmd.exe` の直接起動と禁止の文字の集合) | 【Python の設計】 |

---

## 13. 未確認の事項

- **Linux と Windows の実機の動作**: この文書の【実測】は、macOS(Darwin、arm64)だけで得た。Linux の `returncode` の符号、`send_signal` と終了済みの子、PEP 475 の `sleep`、`signal.signal` のメインスレッド制約、`start_new_session` の効果は、同じ結果になると考えられるが、Linux では確かめていない。実装時に、Linux で、11 章の検査を実行して確かめる。
- 対象の Python の版(最小と最新)。確認は 3.14.10、3.15.2、3.9.6 だけで、`requires-python` の最小版(3.11 と思われる)そのものでは動かしていない。
- **`SIGCHLD` が `SIG_IGN` のときの `returncode`**: macOS では正しい値だった。Linux で、終了状態を取得できず `returncode` が 0 になるか(4.4 節)は、Python の実装についての知識によるもので、確かめていない。
- Windows での `returncode` の型(符号なし 32 ビットで返るか)。`read_exit` の符号つきへの変換は、どちらでも正しい値になる。
- Windows で、Ctrl+C と Ctrl+Break を受け取ったとき、ジョブの終了の様子、`SIGBREAK` の番号(21 と思われる)、`CTRL_BREAK_EVENT` による自動テストの方法(7.3 節)、`.cmd` の起動(4.8 節の形、`cmd.exe` の「バッチ ジョブを終了しますか」の問い合わせ)、`time.sleep` の分解能(5.1 節)。
- Windows でのプログラム探索(カレントディレクトリを含むか、拡張子の補完)の詳細は、`CreateProcess` の標準の規則に依存し、確認していない。
- 端末から Ctrl+C を押した場合に、ジョブが SIGINT を 1 回だけ受けること(擬似端末の下での確認をしていない。親への `kill -INT` では確かめた)。
- 制御端末を持たない(`start_new_session=True`)ジョブが、`/dev/tty` を使えないことの影響範囲(`ssh`、`sudo` など。実際の道具での確認はしていない)。Linux での、標準入力の読み込みへの影響(macOS の擬似端末の下では、読めた)。
- ほぼ同時に届いた複数のシグナルの処理順序、ネストしたハンドラの動き。
- `sys.excepthook` の経路で 70 になること(`os._exit(70)`)の、Linux・Windows と、最小の Python の版での確認(macOS の 3.15.2・3.9.6 では確認した。10 章 6.4 節)。
- `time.monotonic_ns()` が、システムの休止中に進むかどうか(OS により異なる)。
- 長い C の呼び出し(`os.fsync` など)の途中での、Python のハンドラの実行の遅れ。
- この文書の、公式の文書や実装についての記述のうち、【実測】の印のないものは、Python の公式の文書(`subprocess`、`signal`、`time`、PEP 475)と、一般に知られた挙動による。コードを実行して確かめていない。
