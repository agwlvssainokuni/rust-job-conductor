# 07 プロセス起動とシグナル(`command` の分割・起動・待ち・中断)

対象: `conductor` v0.8.0 と同じ挙動を TypeScript(Node.js)で実装するための仕様。この文書は、次を定める。

- `command` の文字列を、コマンド名と引数に分ける規則(3 章)
- `node:child_process` の `spawn` によるジョブ(子プロセス)の起動、待ち、結果の取得、起動失敗(4・5 章)
- 中断の合図と、OS ごと(UNIX 系と Windows)のシグナル処理(6・7 章)
- OS ごとの差と、OS 分岐を置く場所(8 章)
- イベントループを塞がない規則、非同期の順序の保証(9 章)
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
| 【Node の設計】 | この TypeScript 版が採る設計。利用者から見える挙動は変わらないが、実装者はこれに従う |
| 【実測】 | この文書の作成時に、実際に Node を動かして確かめた挙動(1.2 節に条件を書く) |
| 未確認 | 実測も、公式の文書による裏づけも得ていない。実装時に、採用する Node の版で確かめる(12 章) |

---

## 1. 構成要素と責務

### 1.1 構成要素とファイル

すべて `src/runner/` に置く(分割だけは純粋なので同じディレクトリの別ファイル)。ファイルの全体の表は [02-architecture.md](02-architecture.md) の 6 章。

| 構成要素 | 責務 | 置き場所(ファイル) |
|----------|------|---------------------|
| 分割器 | `command` の文字列を、コマンド名と引数に分ける。文字列を 1 文字(UTF-16 の符号単位)ずつ左から読む状態機械。再帰を使わない。3 OS で同じ規則 | `split.ts` |
| `command` の検査 | `command` を起動せずに検査する。成功なら分けた結果(`CommandLine`)を返す | `split.ts` |
| ジョブの起動と待ち | 1 つのジョブをシェルを介さずに起動し、終了を待ち、実行結果(`JobExecution`)を返す。失敗は値で返し、`Promise` を拒否しない | `run-job.ts` |
| 待ちの制御 | 子プロセスのイベントと中断の合図を結び、転送の判断(1 回だけ、終了済みには送らない)を行う(5 章) | `run-job.ts`(`runJob` の内側) |
| リトライの待機 | リトライの前にタイマーで待つ。待機中に中断の合図で打ち切る(7.5 節) | `wait-retry.ts` |
| 中断の合図 | 「中断が要求された」ことと、最初に受け取ったシグナルの番号を保持する値(6 章) | `interrupt.ts` |
| プラットフォーム層 | OS の違いを吸収する唯一の場所。`process.platform` を読む唯一の場所でもある(8 章) | `platform.ts` |
| シグナル監視 | `process.on` でシグナルを受け取り、中断の合図を立てる | `watcher.ts` |

### 1.2 実測の条件(【実測】の印の意味)

この文書の【実測】は、次の条件で得た。ほかの OS・版では未確認である。

| 項目 | 内容 |
|------|------|
| Node | v26.13.0(一部は v24.22.2) |
| OS | macOS(Darwin) |
| Linux、Windows | 実測していない。Linux の値は、Node の公式の文書と、OS の一般的な知識による(未確認) |
| 注意 | 版管理の道具(Volta など)の「shim」を介して Node を起動すると、Node 自身がシグナルで死んだとき、シェルから見た終了コードが 143 などではなく 1 になる【実測】。シグナルの終了コードを検査するテストは、shim を介さず、Node の実行ファイルを直接起動すること |

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

「前後の空白を除く」の定義は 03 章に従う。注意: JavaScript の `String.prototype.trim()` は、Unicode の White_Space に加えて U+FEFF(BOM)を除き、U+0085(NEL)は除かない。03 章の定義と合わない場合は、03 章の定義を実装した自前の関数を使うこと(03 章と合わせる点)。

定義の検査とは別に、実行の前にもう一度、全ジョブの `command` を 3.6 節のとおり検査する。

---

## 3. `command` の分割規則

### 3.1 入力と出力

- 入力: `command` の文字列(JavaScript の文字列。UTF-16)。
- 出力: コマンド名(`program`。空でない)と、引数の並び(`args`)。`args` の各要素が、そのまま 1 つの引数になる。
- シェルは使わない。変数展開(`$HOME`)、グロブ(`*`)、パイプ(`|`)、`;`、`&&`、リダイレクト(`>`)、バッククォート、`~`、
  括弧は **解釈しない**。普通の文字として引数に入る。パイプなどが要るときは、利用者が `sh -c '...'` を `command` に書く。

### 3.2 文法

文字列を、左から右へ **1 回だけ**読む。区切り文字は次の 4 つだけ。

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
- 文字列の長さに上限はなく、入れ子もないため、再帰やスタックの深さに依存しない(約 30 万引数の入力でも分割できる)。【規定】

### 3.3 UTF-16 の符号単位で走査する場合の注意【Node の設計】

JavaScript の文字列は UTF-16 の符号単位の並びである。分割器は、**符号単位ごとに走査してよい**(`for (let i = 0; i < s.length; i++)` と `s.charCodeAt(i)` または `s[i]`)。理由は、区切り文字(4 つ)と引用符・バックスラッシュが、すべて BMP の ASCII の範囲(U+0000〜U+007F)にあり、サロゲート(U+D800〜U+DFFF)のどの符号単位とも等しくないからである。

| 論点 | 規定 |
|------|------|
| サロゲートペア(例: `𠮷`、絵文字) | 上位サロゲートと下位サロゲートは、どちらも「上記以外の文字」として、そのまま `current` に足される。ペアは 2 つの符号単位として並んだまま語に入り、壊れない。コードポイント単位(`for (const ch of s)`)で走査しても、結果は同じ。どちらでもよい |
| 単独のサロゲート(対のない `\uD800` など) | 分割器は、そのまま語に入れる(検査も置換もしない)。起動のとき、Node が引数を UTF-8 に変換する際に U+FFFD(置換文字)になる【実測】。この変換は、`conductor` が防ぐことではない |
| 正規化・大文字小文字・照合 | しない。`String.prototype.normalize`、`toLowerCase`、`trim` を、語に対して呼ばない |
| 語の組み立て | `current` は、文字列の連結(`+=`)で組み立ててよい。語の列(`words`)は配列に `push` する |
| 大きな配列の展開 | `words.push(...other)`、`Math.max(...words)`、`fn.apply(null, words)` のように、**語の配列を関数の引数に展開しない**(約 30 万要素で、引数の個数の上限により `RangeError` になりうる。上限の値は未確認)。`for` の添字でたどる |
| 戻り値 | `args` は新しい配列(`words.slice(1)` でよい。1 語だけのときは空配列) |

### 3.4 擬似コード(同じ結果になる実装であれば、書き方は自由)

```text
words = []; current = ""; in_word = false; state = Plain
for 符号単位 c を、先頭から末尾へ(添字で):
  switch state:
    Plain:  if c in separator: if in_word: words.push(current); current = ""; in_word = false
            elif c == "'": in_word = true; state = Single
            elif c == '"': in_word = true; state = Double
            else: in_word = true; current += c
    Single: if c == "'": state = Plain else current += c
    Double: if c == '"': state = Plain
            elif c == '\\': state = DoubleBackslash
            else: current += c
    DoubleBackslash:
            if c == '"': current += '"'; state = Double
            elif c == '\\': current += '\\'            # state は DoubleBackslash のまま
            else: current += '\\' + c; state = Double
if state != Plain: return 誤り(unclosedQuote)
if in_word: words.push(current)
if words が空 or words[0] == "": return 誤り(empty)
return (words[0], words[1:])
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
| `echo '𠮷 😀' x`(サロゲートペアを含む) | `echo` | `𠮷 😀`, `x` |
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
| 固定の文 | `unclosedQuote` は `引用符が閉じていません`、`empty` は `コマンド名がありません`。**入力の値を含めない**(秘密の値が `command` に入っていても表示に出ない)【規定】。`CommandError` の `message` も、この固定の文だけにする |
| 利用者への表示(標準エラー) | `conductor: error: <定義ファイルのパス>: job '<ジョブID>': <文>`。文は表示言語に従う。英語は `command has an unclosed quote` / `command is empty`、日本語は `command の引用符が閉じていません` / `command が空です`(組み立ては `report`。08 章) |
| 検査の時点 | `run` / `pipeline` / `validate` は、定義の読み込みと検証のあと、**全ジョブ**の `command` を検査し、誤りのあるジョブをすべて報告して(1 ジョブ 1 行)、終了コード 64 で終わる。ジョブは 1 つも起動せず、ステートにも触れない |
| 順序 | シグナルの準備(`run`/`pipeline` のみ)→ 定義の読み込み(64)→ `command` の検査(64)→ ジョブID(68)→ run ID(68)→ ステートの読み込み(65)→ 照合(65) |
| 防御 | ジョブの起動の内側でも分割し直す。万一失敗したら、起動失敗の実行結果を返す(通常は到達しない) |

検査の関数の骨格(型は [02-architecture.md](02-architecture.md) の 2.1 節):

```ts
export interface CommandLine {
  readonly program: string;
  readonly args: readonly string[];
}
export type CommandErrorKind = 'unclosedQuote' | 'empty';
export class CommandError extends Error {
  readonly kind: CommandErrorKind; // message は固定の文だけ
}
export type CommandResult =
  | { readonly ok: true; readonly line: CommandLine }
  | { readonly ok: false; readonly error: CommandError };

export function splitCommand(s: string): CommandResult;
export function checkCommand(c: Command): CommandResult; // 文字列形式は split、配列形式は 3.7 節
```

---

## 4. プロセスの起動(`node:child_process` の `spawn`)

### 4.1 手順(時系列)

関数: `runJob(job: Job, sig: Interrupt, opts?: RunJobOptions): Promise<JobExecution>`。失敗はすべて戻り値で返し、返す `Promise` は拒否されない【規定】。

1. 実行結果を初期化する: `started = true`、`exitCode = null`、`terminatedBySignal = null`、`startFailure = false`、
   `interrupted = false`、`signalNumber = null`。
2. `command` を分割する(文字列は 3 章、配列は 3.7 節)。失敗なら `startFailure = true` で返す。
3. **起動の前**に合図が立っていれば(`sig.number()` が `null` でない)、起動せずに返す: `started = false`、`interrupted = true`、`signalNumber = 番号`。
   (ジョブは実行されない。)
4. プラットフォーム層で、`spawn` に渡す実行ファイル・引数・オプションを決める(8 章)。拒否された場合(Windows の `.cmd` の安全策、4.7 節)は、`startFailure = true` で返す。
5. `spawn(file, args, options)` を呼ぶ。**呼び出しが同期的に例外を投げたら**(`try/catch` で受ける)、`startFailure = true` で返す。引数に NUL(U+0000)を含む、実行ファイル名が空、などで `ERR_INVALID_ARG_VALUE` の `TypeError` が投げられる【実測】。`started` は `true` のまま。
6. 返された `ChildProcess` に、次のイベントの購読を **同じ同期の区間の中で**(`spawn` の戻りの直後に)登録する。登録の漏れがあると、`error` が未処理の例外になり、プロセスが落ちる。
   - `error`: 5.2 節の規則。
   - `spawn`: 子が起動した。ここから 5 章の転送の対象になる。
   - `close`: 終了。7 の手順に進む。
7. `close` を受けたら(5 章の待ち)、起動に失敗していなければ、結果を取り出す(4.3 節):
   - `exitCode` = 子の終了コード(UNIX 系でシグナルで終了したときは `null`)。
   - `terminatedBySignal` = シグナル終了のときの番号(UNIX 系のみ。Windows は常に `null`)。
   - **この時点で** 合図が立っていれば `interrupted = true`、`signalNumber = 番号`。
8. 終了の状態を取得できないとき(終了コードもシグナル名も `null`、またはシグナル名が番号に変換できない、4.3 節)は、`startFailure = true` で返す(分類で 127)。

時間切れ(タイムアウト)はない。自動のリトライもしない(リトライは `cli` の流れの制御で行う。7.5 節)。

### 4.2 起動の条件(`spawn` のオプション)

| 項目 | 規定 |
|------|------|
| シェル | 使わない。`shell: false` を**明示する**【規定】。`sh -c` や `cmd /C` を挟まない。`spawn` に渡すのは、分割(または配列)で得た `program` と `args` だけ。唯一の例外は、Windows で `.cmd` / `.bat` を起動するときの `cmd.exe` 自身(4.7 節。`cmd.exe` を `program` として直接起動し、`shell: false` のまま) |
| 使うオプション | `stdio: 'inherit'`、`shell: false`、`detached`(UNIX 系は `true`、Windows は指定しない。4.6 節)。Windows の `.cmd` のときだけ `windowsVerbatimArguments: true` を足す。**これ以外のオプションを渡さない** |
| 使わないオプション | `cwd`、`env`、`argv0`、`uid`、`gid`、`timeout`、`killSignal`、`signal`(`AbortSignal`)、`serialization`、`windowsHide`(既定のまま)。`timeout` と `signal` は、子を強制終了する働きを持つため、強制終了しない規定に反する |
| プログラムの探索 | `spawn` の標準の規則に従う(`program` に区切り文字がなければ、OS の標準の探索。`PATH` など)。`conductor` 独自の探索はしない。Windows での探索の細部(拡張子の補完、カレントディレクトリを含むか)も、Node の標準の規則に従い、見つからなければ起動失敗として扱う(12 章) |
| 作業ディレクトリ | `cwd` を渡さない(`conductor` のカレントディレクトリを引き継ぐ。定義ファイルの場所に移らない)。`conductor` は `process.chdir()` を呼ばない【規定】 |
| 環境変数 | `env` を渡さない(すべて引き継ぐ。Node は `process.env` を使う)。追加・削除・上書きをしない。run ID も環境変数としては渡さない。`conductor` は `process.env` を書き換えない【規定】 |
| 標準入力・標準出力・標準エラー | `stdio: 'inherit'` を明示する【規定】。省略した場合の既定は `'pipe'` で、子の出力が `conductor` のストリームに溜まり、引き継ぎにならない。`conductor` は、ジョブの出力を読まない・保存しない・ステートに書かない【規定】。子の `stdin`/`stdout`/`stderr` のストリームは、`null` になる(`'inherit'` のため) |
| 引数の渡し方 | 分割した各引数を、そのまま `args` の配列で渡す(メタ文字は注入にならない)【規定】。Windows で `.cmd` / `.bat` を起動するときは 4.7 節の安全策を適用する |
| 起動できなかった場合 | 原因(見つからない・実行権限がない・実行できない形式・ディレクトリを指定した、など)を区別せず `startFailure = true`。終了コードは 127(分類)。表示は `conductor: error: job '<ID>': cannot start command '<プログラム名>'`(英語。組み立ては `report`)。ジョブの終了コード・シグナルは `null`。原因の `errno`(`ENOENT` など)は、表示にも、ステートにも出さない |
| 時間の上限 | ない。ジョブが終わらなければ、`conductor` は待ち続ける |
| ジョブの強制終了 | しない(`child.kill('SIGKILL')` を呼ばない。強制終了のシグナルを送る機能はない) |

### 4.3 起動失敗の検出と、終了の取り出し

#### 4.3.1 `error` イベント(起動失敗)

| 事実 | 内容 |
|------|------|
| `spawn` が失敗したとき | 同期的には例外を投げず(4.1 節の手順 5 の引数の誤りを除く)、**`error` イベント**を非同期に出す。`child.pid` は `undefined`。`spawn` イベントと `exit` イベントは出ない。**その後で `close` イベントが、負の数(`-errno`)の終了コードと `null` のシグナルで出る**【実測。`/nonexistent/x` で `error.code = 'ENOENT'`、`close` の値は -2】 |
| 実行権限がない、ディレクトリを指定した | `error.code = 'EACCES'`、`close` の値は -13【実測。macOS】 |
| 存在しない作業ディレクトリ | `ENOENT`(プログラムがない場合と区別できない)【実測】。`conductor` は `cwd` を渡さないので起こらない |
| `spawn` が成功したあとの `error` | 公式の文書では、プロセスを起動できなかった、`kill` が失敗した、`abort` した、メッセージの送信に失敗した、のいずれかで出る。`conductor` はこのうち `kill` の失敗だけが起こりうる(未確認。`child.kill` の失敗が `error` になるかは、実測では起きなかった) |

規定:

1. 購読する `error` のハンドラは、**起動が確認される前**(`spawn` イベントの前)に呼ばれたものを「起動失敗」として扱い、**待ちを終える**(`close` を待たない)。このとき `startFailure = true`、`started = true`、`exitCode = null`、`terminatedBySignal = null`。
2. 起動失敗のあとに届く `close`(負の終了コード)は、**無視する**。「結果を 1 回だけ確定する」ための印(`settled`)を持ち、2 回目の確定をしない。
3. `spawn` イベントの後に届いた `error` は、結果を確定させない(子はまだ動いている。`close` を待つ)。ただし、ハンドラは**必ず付けておく**(付けないと、未処理の例外として `conductor` が落ちる)。内容は捨ててよい。
4. 負の終了コード(`close` の値が `< 0` で、`spawn` イベントが来ていない)を、ジョブの終了コードとして読まない。

#### 4.3.2 `exit` と `close` の違い

| イベント | いつ | 引数 | 備考 |
|----------|------|------|------|
| `exit` | 子プロセスが終わったとき。標準入出力のストリームがまだ開いていてもよい | `(code: number \| null, signal: NodeJS.Signals \| null)` | 起動に失敗したときは出ない【実測】 |
| `close` | 子プロセスが終わり、**かつ**、子の標準入出力のストリームが閉じたあと | `(code, signal)`(`exit` と同じ値) | 起動に失敗したときも出る(負の値)【実測】 |

この版の規定【Node の設計】:

- 終了の観測には **`close`** を使う。`exit` と `close` を両方購読し、`exit` で「終了した」印(`exited`)を立て(転送の対象から外す。5.2 節)、結果の確定は `close` で行う。
- `stdio: 'inherit'` のとき、`conductor` は子の標準入出力のストリームを持たないので、`close` は `exit` の直後に出る【実測: 両方が同じミリ秒に出る】。ジョブが孫のプロセスを残して先に終わっても(`sh -c 'sleep 1 & exit 5'`)、`close` は孫の終了を待たない【実測。6 ms で出た】。
- 将来 `stdio` を `'pipe'` に変える場合は、孫のプロセスが出力を握っている間、`close` が遅れる。この版は変えない。

#### 4.3.3 終了コードとシグナル名の取り出し

`close`(`exit` も同じ)の引数 `(code, signal)` から、次のように読む。

| 項目 | UNIX 系 | Windows |
|------|---------|---------|
| 終了した | `code` は number(下位 8 ビットの 0〜255)、`signal` は `null`。`exitCode = code` | `code` は number。`exitCode = code \| 0`(32 ビット符号つきに直す。下記)、`terminatedBySignal` は常に `null` |
| シグナルで終了した | `code` は `null`、`signal` は `'SIGTERM'` のようなシグナル名【実測】。`terminatedBySignal = (シグナル名 → 番号の表)`、`exitCode = null` | なし(終了はすべて終了コード。`signal` を読まない) |
| 両方 `null` | 終了の状態を取得できない。`startFailure = true`(手順 8) | 同左 |

- 終了コードの幅(UNIX 系は下位 8 ビット、Windows は 32 ビット符号つき)は、05 章と 10 章のとおり。`300` を返すと UNIX 系は 44【実測】、Windows は 300 のまま。負の値は Windows で負のまま。
- Windows で Node が終了コードを符号なし 32 ビットで返す場合(例: Ctrl+C による終了の `0xC000013A` が 3221225786 になる)に備え、`code | 0`(`ToInt32`)で符号つきに直す。UNIX 系では、この変換をしない(0〜255 のため、変換しても同じ)。Windows で Node がどちらの形で返すかは未確認。`| 0` はどちらでも正しい値になる。
- シグナル名から番号への変換は、`node:os` の `os.constants.signals`(名前 → 番号のオブジェクト)を使う【Node の設計】。UNIX 系の番号は OS により異なる(Linux と macOS で `SIGUSR1`、`SIGBUS`、`SIGCHLD` などが違う)ので、**固定の表を書かず、実行している OS の値を使う**。`SIGHUP`(1)、`SIGINT`(2)、`SIGQUIT`(3)、`SIGTERM`(15)は、両 OS で同じ。変換できないシグナル名は、終了の状態を取得できないものとして扱う(`startFailure = true`、手順 8)。

シグナルで終了したときの `conductor` の終了コード(128 + n)の例:

| シグナル | 番号(Linux、macOS 共通) | 報告する終了コード |
|----------|---------------------------|--------------------|
| `SIGHUP` | 1 | 129 |
| `SIGINT` | 2 | 130 |
| `SIGQUIT` | 3 | 131 |
| `SIGKILL` | 9 | 137 |
| `SIGPIPE` | 13 | 141 |
| `SIGTERM` | 15 | 143 |

そのほかのシグナル(`SIGSEGV`、`SIGABRT`、`SIGUSR1` など)は、OS により番号が異なる場合があるため、`os.constants.signals` の値に 128 を足す。macOS の値は、`SIGABRT` 6、`SIGSEGV` 11、`SIGUSR1` 30、`SIGUSR2` 31、`SIGBUS` 10【実測】。Linux の値(`SIGUSR1` 10、`SIGBUS` 7 など)は、実測していない。

#### 4.3.4 `child.kill` の意味

| 事実 | 内容 |
|------|------|
| 戻り値 | `child.kill(signal)` が返す `boolean` は、**シグナルの送信が成功したか**(OS の `kill` が成功したか)である。**子が終了したことを意味しない**。送信直後の `child.exitCode` と `child.signalCode` は、まだ `null` で、`close` はあとで届く【実測】 |
| `child.killed` | 送信に成功すると `true` になる。「子が死んだ」意味ではない【実測】 |
| 終了済みの子 | `exit` のあと(`exitCode` または `signalCode` が `null` でなくなったあと)は、`false` を返し、何も送らない【実測】 |
| 2 回送ると | 子が終わるまでは、2 回とも `true` を返す(2 回送ってしまう)【実測】。**2 回目を送らないのは、`conductor` の責任**(5.3 節) |
| 不正なシグナル名 | `ERR_UNKNOWN_SIGNAL` の `TypeError` を同期的に投げる【実測】。許可集合(7.2 節)の名前だけを渡すので起こらない |
| 送信先 | `child.kill` は、子の pid だけに送る。プロセスグループには送らない(`process.kill(-pgid)` を `conductor` は使わない) |
| Windows | POSIX のシグナルがなく、`child.kill` は、子を強制終了する(Node の文書による記述。実測していない)。したがって、**Windows では `child.kill` を呼ばない**(7.3 節) |

### 4.4 実行結果(`JobExecution`)の各項目

| 項目 | 型 | 意味 |
|------|-----|------|
| `jobId`, `command` | `string`, `Command` | 定義のジョブIDと `command`(表示とステートには `jobId` だけを使う。`command` はステートに書かない) |
| `started` | `boolean` | 起動を試みたか。`false` は、起動の前に合図が立っていたときだけ |
| `exitCode` | `number \| null` | 子の終了コード。UNIX 系でシグナルによって終了したときは `null`。Windows は常に値がある(起動失敗を除く) |
| `terminatedBySignal` | `number \| null` | UNIX 系のシグナル終了の番号。Windows は常に `null` |
| `startFailure` | `boolean` | 分割の失敗、起動の失敗、終了の状態の取得の失敗のいずれか |
| `interrupted` | `boolean` | 起動前、または、終了の観測の直後に、合図が立っていた |
| `signalNumber` | `number \| null` | `interrupted` のとき、**最初に受け取った**シグナルの番号 |

型の全体は 02 章の 2.1 節と 05 章に従う。

### 4.5 分類(終了コードの決定)

分類は `flow` の純粋な関数が行う。ここでは、規定の優先順位を再掲する。実行結果は、次の優先順位で、最初に当てはまるものを採る。

| 順 | 条件 | 分類 | 報告する終了コード |
|----|------|------|--------------------|
| 1 | `started = false` かつ `interrupted` | 分類なし(エラーとして記録しない) | `128 + signalNumber` |
| 2 | `started = false` で中断でもない(起こらない) | エラー | 127 |
| 3 | `interrupted`(起動済み) | エラー | `128 + signalNumber`(ジョブ自身の終了コードは見ない。ジョブが 0 で終わっていてもエラー) |
| 4 | `startFailure` | エラー | 127 |
| 5 | `terminatedBySignal = n` | エラー | `128 + n` |
| 6 | `exitCode = null`(起動して終わったのに値がない) | エラー(`conductor` 自身のエラー) | 1 |
| 7 | `exitCode` が成功コード(既定は `[0]`、`success_exit_codes` で指定) | 成功 | 0 |
| 8 | `exitCode` が `warning_exit_codes` | 警告 | 0 |
| 9 | `exitCode = 0` で成功コードに含まれない | エラー(`conductor` 自身のエラー) | 1 |
| 10 | 上記以外 | エラー | `exitCode` のまま |

`conductor` の終了コードは、報告する終了コードを 0〜255 に合わせたもの(範囲内はそのまま。範囲外は下位 8 ビット。エラーで
下位 8 ビットが 0 になるときは 1)。詳細は 10 章。

### 4.6 `detached` を使う/使わない【Node の設計】

| 項目 | UNIX 系(Linux、macOS) | Windows |
|------|------------------------|---------|
| `detached` | **`true` にする**【規定: 新しいプロセスグループで動かす】 | **指定しない**(`false`) |
| 理由 | `detached: true` で、子は新しいセッションのリーダーになり、プロセスグループ ID が子の pid になる【実測: `ps` で `PGID = PID`、`STAT = Ss`。`false` では、親と同じ `PGID`】。端末の Ctrl+C(SIGINT)は、端末のフォアグラウンドのプロセスグループにだけ配られるため、ジョブには直接届かず、`conductor` が受けて 1 回だけ転送する(7.2 節)。`false` のままだと、ジョブは SIGINT を直接受けたうえで、`conductor` の転送でもう 1 回受ける(2 回受ける) | Node の文書では、Windows で `detached: true` を指定すると、子は**自分のコンソールのウィンドウ**を持つ(実測していない)。同じコンソールで動かし、Ctrl+C を OS に配らせる規定(7.3 節)に反する |
| 副作用 | `detached: true` は、新しいセッション(`setsid`)を作るため、子は**制御端末を持たない**。ジョブが `/dev/tty` を直接開く(`ssh`、`sudo` のパスワード入力など)場合に、端末を使えなくなりうる(未確認)。標準入力(`fd 0`)の読み込みへの影響は未確認(12 章) | なし |
| 親が先に終わったとき | `unref()` を呼ばないので、`conductor` は子の `close` を待つ。親(`conductor`)が強制終了された場合、子は残って動き続ける(検知も停止もしない) | 同左 |

`detached` は「親が終わっても子が生き続ける」ためのオプションであり、この版は、それを目的としない。目的はプロセスグループの分離だけである。`child.unref()` は呼ばない。

### 4.7 Windows の `.cmd` / `.bat` の起動【Node の設計】

Windows の `cmd.exe` は、`.cmd` / `.bat` を実行するときに、引数を独自の規則で解釈する(`"`、`%`、`^`、`&`、`|`、`<`、`>` など)。Node の `spawn` は、`.cmd` / `.bat` を `shell: true` なしで起動すると、安全のために `EINVAL` で失敗する場合がある(Node の公開された変更による。版ごとの境界と、`error.code` の値は未確認)。いずれにせよ、`conductor` は、引数の中のコマンドが実行されないこと(受け入れテスト AC-113、AC-114)と、安全な引数で起動できること(AC-115。終了コード 7 が取れる)を守る。

決定(次のとおり、`cmd.exe` を直接起動する。`shell: false` のまま):

1. `process.platform` が Windows のとき、かつ `program` の拡張子が(大文字小文字を区別せず)`.cmd` または `.bat` のときだけ、この規則を使う(`path.win32.extname(program).toLowerCase()` で判定)。拡張子が付いていない `program`(例: `build`)は、Node の標準の探索に任せる(`.cmd` は補完されない可能性がある。未確認)。
2. 検査: `args` のどれかが、次の文字を含むなら、**起動せず**、起動失敗の実行結果を返す。`program` は、`(` と `)` を除く同じ集合を検査する(パス `C:\Program Files (x86)\...` を許す)。

```text
"  %  ^  &  |  <  >  !  (  )  改行(\n)  復帰(\r)  NUL
```

3. 検査を通ったら、実行ファイルを `cmd.exe`(環境変数 `ComSpec` が絶対パスならその値、そうでなければ `cmd.exe`)、引数を `['/d', '/s', '/c', '"' + 全体 + '"']` にして `spawn` する。`全体` は、`program` と各 `args` を、それぞれ `"` で囲み、空白 1 つで連結した文字列(空の引数は `""`)。`windowsVerbatimArguments: true` を渡し、Node が引数を再び引用しないようにする。
4. 配列形式の `command` も、同じ検査を通る。

この禁止の文字の集合と、`cmd.exe` の起動の形は、この仕様の設計判断であり、Windows の実機で確かめていない(未確認。12 章)。`cmd.exe` が `.bat` の実行中に Ctrl+C を受けたとき「バッチ ジョブを終了しますか (Y/N)?」と尋ねる場合の影響も未確認。

---

## 5. 待ち(イベントによる待機)

### 5.1 規定

| 項目 | 内容 | 強さ |
|------|------|------|
| 待ち方 | `close` イベントを `Promise` で待つ。ポーリング(周期的な確認)をしない | 【Node の設計】 |
| 回収 | 子の終了の状態を読むのは、`runJob` の内側の `close` のハンドラ 1 か所だけ。結果の確定は 1 回だけ(`settled` の印) | 【規定】 |
| 転送の判断 | 「子がまだ終わっていない」と観測した時点で、**まだ転送していない**かつ合図が立っていれば、シグナルを子に **1 回だけ** 転送する(7.2 節) | 【規定】 |
| 終了済みへの送信の禁止 | 終了済み(`exit` を観測した)の子には送らない。子が先に終わっていれば、合図が立っていても転送しない | 【規定】 |
| 転送は 1 回 | 2 回目以降の合図(同じシグナルでも別のシグナルでも)で、再度の転送や強制終了はしない | 【規定】 |
| 転送のあと | 子の終了を待ち続ける(子がシグナルを無視すれば、`conductor` はそのまま待ち続ける) | 【規定】 |
| CPU の使用 | 子の実行中の `conductor` の CPU 時間が、約 3 秒の待ちで 100 ms 以下であること(イベント駆動なので満たす。UNIX 系で計測する。11 章) | 【規定】 |
| 転送の遅れ | 合図が立ってから転送まで、目に見える遅れがない(目安として 10 ms 以下) | 【目安】 |

### 5.2 待ちの制御の設計

`runJob` は、次の状態を、1 回の呼び出しの内側(クロージャ)に持つ。

| 状態 | 初期値 | 意味 |
|------|--------|------|
| `spawned` | `false` | `spawn` イベントを受けたか |
| `exited` | `false` | `exit` を受けたか(子が終わった) |
| `forwarded` | `false` | 転送を 1 回行ったか(行った/試みた) |
| `settled` | `false` | 結果を確定したか |

手順:

1. `spawn` の直後に、`child.once('error', ...)`、`child.once('spawn', ...)`、`child.once('exit', ...)`、`child.once('close', ...)` を購読する。購読は、`spawn` を呼んだ同じ同期の区間で行う。
2. `error`(`spawned` が偽): 4.3.1 節の規則 1。`settled` を立て、起動失敗の結果で `Promise` を解決する。`spawned` が真なら、何もしない(4.3.1 節の規則 3)。
3. `spawn`: `spawned = true`。**合図がすでに立っていれば、ここで転送の判断を行う**(`tryForward()`)。立っていなければ、`sig.whenRaised()` の `Promise` に `tryForward` を結ぶ(`then`)。
4. `tryForward()`(同期の関数):
   1. `settled` か `exited` か `forwarded` なら、何もしない。
   2. `forwarded = true`(送信の成否にかかわらず、1 回でやめる)。
   3. `n = sig.number()`。`n` が許可集合(7.2 節)になければ、送らず、待ち続ける。
   4. `platform.forward(child, n)`(UNIX 系: `child.kill(名前)`。Windows: 何もしない)。戻り値と例外は無視する(例外は `try/catch` で捕らえる)。
5. `exit`: `exited = true`。
6. `close`(`settled` が偽): `settled = true`。4.3.3 節で結果を取り出し、`sig.number()` を読んで `interrupted` を決める(手順 7)。`Promise` を解決する。
7. `close` のあとに合図が立っても、`tryForward` は `settled` で何もしない。

注意:

- JavaScript はイベントループが 1 つなので、`tryForward` と `exit`/`close` のハンドラは、同時には動かない。「`exited` を見てから `kill` するまで」の間に、別のハンドラが割り込むことはない。複数のスレッドで動く環境で要る「先に終了を確かめ直す」手順は不要である。
- 一方で、**子が終わってから `exit` イベントが配られるまでの間**に、シグナルが届いて `tryForward` が先に動く可能性は残る(OS が子の終了を `conductor` に通知し、Node が `exit` を出すまでの短い区間)。このとき `child.kill` は、まだ回収されていない子に送る(回収前の pid は再利用されないので、無関係なプロセスには送らない)。Node の `child.kill` は、Node が子の終了を処理したあとは `false` を返し、送らない【実測】。Node 内部で、子の回収と `exit` の配送が同じ区間で行われるかは、未確認(12 章)。実装は、**pid の番号を自分で保持して `process.kill(pid, ...)` に渡してはならない**(回収後の pid の再利用で、無関係なプロセスにシグナルを送りうる)。必ず `child.kill` を通す。
- `runJob` が返す `Promise` は、`close`(または起動失敗)で解決する。購読したハンドラは `once` で付け、解決後に残らない。

### 5.3 テスト用の差し替え口【Node の設計】

`runJob` は、第 3 引数のオプションで、`spawn` の関数とプラットフォーム層を差し替えられる。

```ts
export interface ChildLike {
  readonly pid: number | undefined;
  readonly exitCode: number | null;
  readonly signalCode: NodeJS.Signals | null;
  kill(signal: NodeJS.Signals): boolean;
  once(event: 'error', l: (e: Error) => void): this;
  once(event: 'spawn', l: () => void): this;
  once(event: 'exit' | 'close', l: (code: number | null, signal: NodeJS.Signals | null) => void): this;
}
export interface RunJobOptions {
  readonly spawn?: (file: string, args: readonly string[], options: SpawnOptions) => ChildLike;
  readonly platform?: Platform;
}
```

偽の `ChildLike`(`EventEmitter` で作る)で、次を単体で検査できる(11 章): 転送が 1 回だけ、先に `exit` した子へ転送しない、2 回目以降の合図で追加の転送をしない、`spawn` の前に合図が立った場合は `spawn` イベントで転送する、`error` のあとの `close`(負の値)を無視する、`close` の値が `(null, null)` のとき起動失敗になる。

---

## 6. 中断の合図(`Interrupt`)

「中断の合図」は、`conductor` がシグナルを受け取ったことを、ジョブを待つ側へ伝える値。

### 6.1 規定

| 項目 | 規定 |
|------|------|
| 内容 | 「立っているか」と、立っているなら「最初のシグナルの番号」 |
| 初期 | 立っていない |
| 立てる(番号 n) | 立っていなければ n を保存する。**すでに立っていれば何もしない**(最初の番号が残る)。番号は任意の整数でよく、負数や非常に大きい値、0 でも、「立った」ことは記録される |
| 性質 | 立ったら下りない。`raise` は同期的に動く。待つ側は読むだけ |
| 等価性 | 番号が等しければ等しい |

### 6.2 TypeScript の型の骨格【Node の設計】

```ts
// Interrupt は、中断の合図。new Interrupt() で作る。
export class Interrupt {
  // 立っていなければ n を保存し、whenRaised() の Promise を解決する。立っていれば何もしない。
  raise(n: number): void;
  // 立っていれば最初の番号、立っていなければ null。読むだけ。
  number(): number | null;
  // 立つと(または、すでに立っていれば直ちに)最初の番号で解決する。拒否されない。
  whenRaised(): Promise<number>;
}
```

- 実装は、フィールド 2 つ(`raised: boolean` と `value: number`、または `value: number | null`)と、最初の `raise` で解決する `Promise` を持つ。`null` で「立っていない」を表す場合、番号に `null` を使わない(番号は `number` 型なので衝突しない)。
- JavaScript は、ユーザーのコードが 1 つのスレッドで動く。ロックも原子的な演算も要らない。ただし `raise` は、シグナルのハンドラ(7 章)から、`await` の途中に割り込んで呼ばれうる。2 つの値(有無と番号)は、**1 つの同期的な関数の中で一緒に更新する**(途中の状態を `await` をまたいで見せない)。
- `whenRaised()` が返す `Promise` は、保留のあいだ、イベントループを生かさない(タイマーやハンドルと違い、保留の `Promise` はプロセスの終了を妨げない)。
- 同じ `Interrupt` を、`runJob`、`waitRetry`、`cli` の流れが共有する。`cli` が作り、`Signals.start` に渡す。

---

## 7. シグナル処理

### 7.1 準備(両 OS 共通)

- `run` と `pipeline` だけが、**定義の検査より前**に、シグナルの監視を始める【規定】。
  `watch`・`reset`・`validate`・`convert` は監視せず(`process.on` を呼ばず)、OS の既定の動作で終わる。
- `cli` の流れが、`new Interrupt()` で合図を 1 つ作り、`Deps.signals.start(sig)` に渡す。
- 監視の登録は **プロセスで 1 回だけ**。2 回目の呼び出しは「すでに登録済み」(`'alreadyRegistered'`)として、流れを成功と同様に続ける
  (2 回目に渡した合図には配線されない)。本番の実装は、モジュール内の変数(真偽値)で 1 回だけにする。
- 登録は `process.on(名前, ハンドラ)` で行う。これが例外を投げたとき(未確認。正常な環境では起きない想定)は、すでに登録したハンドラを `process.off` で外し、「登録の失敗」を返す。このとき、標準エラーに
  `conductor: error: failed to set up signal handling` を書き、終了コード **70** で終わる(定義の検査もジョブの起動もしない)。
  登録に失敗した場合は、次の登録を許す(登録済みの印を戻す)。
- 「最初の番号」は、合図を立てる操作(`raise`)が呼ばれた順の最初。
- 監視を止める関数(`stop()`)を返す。本番では呼ばない(プロセスの終了まで動き続ける)。テストでは、登録したハンドラを `process.off` で外して、リスナーを残さない。**すべてのリスナーを外すと、その信号の OS の既定の動作に戻る**【実測: `process.on` のあと `process.off` で外し、自分に SIGTERM を送ると、143 で終わった。Node v26.13.0 と v24.22.2】。
- シグナルのハンドラの中では、`raise` を呼ぶだけにする(出力、ステートの書き込み、`child.kill` を呼ばない)【規定】。

### 7.2 UNIX 系(Linux、macOS)

受け取るシグナルは **SIGHUP(1)、SIGINT(2)、SIGQUIT(3)、SIGTERM(15)** の 4 つ【規定】。受け取るたびに、`raise` を、そのシグナルの番号で呼ぶ。
登録後、これらのシグナルでは、Node の既定の動作(終了)をしなくなる。

Node での対応(`platform.ts` と `watcher.ts`):

| 項目 | 規定 |
|------|------|
| 受け取る集合 | `['SIGHUP', 'SIGINT', 'SIGQUIT', 'SIGTERM']` |
| 登録 | 各名前に `process.on(名前, ハンドラ)`。ハンドラは `(signal: string, number: number)` を受け取る【実測: `('SIGHUP', 1)`、`('SIGINT', 2)`、`('SIGQUIT', 3)`】。番号は、引数 `number` を使わず、**名前から固定の表(`SIGHUP` → 1、`SIGINT` → 2、`SIGQUIT` → 3、`SIGTERM` → 15)で引く**(両 OS で同じ。ハンドラの第 2 引数の保証は未確認のため) |
| イベントループ | シグナルのリスナーだけでは、プロセスの終了を妨げない【実測: `process.on('SIGTERM', ...)` だけのスクリプトは、直ちに終わる】。実行中の子プロセスのハンドルが、ループを生かす |
| 起動時に SIGHUP が無視されている場合 | `nohup` で起動しても、`process.on('SIGHUP', ...)` のリスナーは SIGHUP を受け取る【実測】。したがって、4 つのシグナルをいつも受け取る規定を満たす |
| 転送 | `child.kill(名前)`(4.3.4 節)。`process.kill(pid, ...)` ではない(5.2 節) |

時系列(ジョブの実行中に、利用者が `kill -TERM <conductor の pid>` したとき):

1. `conductor` が SIGTERM を受け取る。ハンドラが `raise(15)` を呼ぶ。`whenRaised()` の `Promise` が解決する。
2. `tryForward()`(5.2 節)が、マイクロタスクとして動く。子が `exited` でなく、`forwarded` でなければ、次へ。
3. 子の pid に、**同じシグナルを 1 回**送る。送る前に次を検査する(`platform.forward` の中)。
   - `child.pid` が整数で 0 より大きい(0 や負、`undefined` にならない。プロセスグループへの送信にならない)。
   - シグナルが `{SIGHUP, SIGINT, SIGQUIT, SIGTERM}`(番号 `{1, 2, 3, 15}`)のどれか(SIGKILL、SIGSTOP、SIGCONT、SIGUSR1、0、負、9999 は送らない)。
   - 送信先は、**子プロセスの pid だけ**。子の子孫(`sh -c` が起こした孫など)には、`conductor` は送らない。
   - 送信の失敗(`child.kill` が `false` を返す、または例外)は無視する。
4. 子は、受け取ったシグナルの既定の動作で終了するか、自分で処理する。`conductor` は待ち続ける。2 回目以降のシグナルでは、何も追加しない。
5. 子の終了を観測する(`close`)。`code`(シグナル終了なら `null`)と `signal` を読み、直後に合図が立っているので
   `interrupted = true`、`signalNumber = 15` を足す。
6. 分類で「エラー、`128 + 15 = 143`」となり、流れの制御はこの結果をステートに**エラーとして**記録する(再開位置はそのジョブ)。
7. `conductor` は **143** で終わる(`pipeline` は次のジョブを起動しない)。

その他:

- 子は、`detached: true` で作る **新しいプロセスグループ**(グループ ID は子の pid)で動く【規定】(4.6 節)。端末の
  Ctrl+C(SIGINT)は、フォアグラウンドのプロセスグループに配られるため、ジョブには直接届かず、`conductor` が受けて 1 回転送する
  (SIGINT のとき、ジョブは `terminatedBySignal = 2` で終わるのが典型)。
- 転送の許可集合にない番号で合図が立った場合(`conductor` 本体の監視は 4 つだけを受けるため、通常は起こらない。`runJob` を単独でテストする場合のみ)は、
  送信せず、子が自分で終わるのを待ち、終わりを観測した後に `interrupted = true`、`signalNumber = その番号` を足す
  (子の終了コードは `exitCode` に残るが、分類では見ない)。
- 時間を空けて複数のシグナルが届いたときは、到着順の最初の番号が残る(INT のあとの HUP でも 2)。
  ほぼ同時に届いたときに、どれが「最初」になるかは**規定しない**(Node が複数のシグナルを配る順序は確認していない。12 章)。
- 終了コードの例: SIGHUP は 129、SIGINT は 130、SIGQUIT は 131、SIGTERM は 143。

### 7.3 Windows

受け取るのは **Ctrl+C(番号 2)だけ**【規定】。

| 項目 | 規定 |
|------|------|
| 受け取る集合 | `['SIGINT']` だけ。Node は Windows で、Ctrl+C を `SIGINT` として配る(Node の文書による記述。実測していない) |
| 番号への変換 | 番号は定数 `2` |
| 登録 | `process.on('SIGINT', ハンドラ)`。UNIX 系と同じ仕組み(1 回だけ。`raise(2)`) |
| 転送 | **何もしない**。`platform.forward` の本体は空にし、`child.kill` を**呼ばない**。Windows の `child.kill` は子を強制終了するため(4.3.4 節)、呼ぶと「ジョブを強制終了しない」規定に反する【規定】 |
| ジョブの起動 | `detached` を指定しない。新しいプロセスグループ・新しいコンソールを作らない |
| `SIGTERM`、`SIGHUP`、`SIGQUIT`、`SIGBREAK` | 登録しない。Node は Windows で、`SIGHUP`(コンソールを閉じたとき)と `SIGBREAK`(Ctrl+Break)を配るとされるが、この版の対象外とする(OS の既定の動作)。`SIGTERM` は Windows でイベントとして発生しない、とされる(実測していない) |

時系列(ジョブの実行中に、利用者が Ctrl+C を押したとき):

1. ジョブは `conductor` と **同じコンソール**で起動されている。Ctrl+C は、OS が、同じコンソールを共有するプロセス(`conductor` とジョブ)の両方に配る。
2. `conductor` のハンドラが `raise(2)` を呼ぶ。
3. `tryForward()` は転送を試みるが、Windows の転送は **何もしない**【規定】。
4. ジョブは、OS から受け取った Ctrl+C で(既定の動作で)終了する。
5. 終了を観測する。シグナル終了の番号は常に `null`、`exitCode` は OS の終了コード(Ctrl+C による終了コード。`code | 0` で符号つきにしたもの)。直後に合図が立っているので
   `interrupted = true`、`signalNumber = 2`。
6. 分類で「エラー、`128 + 2 = 130`」。ステートに**エラーとして**記録し、**130** で終わる。

その他:

- Ctrl+Break とコンソールを閉じたときの通知は、この版の対象外(OS の既定の動作)。Node がそれらをどう配るか、`conductor` が受けるかは未確認。
- 自動の試験環境(CI の Windows ランナー)には、対話的なコンソールがなく、Ctrl+C を安定して送る方法がないため、この経路は自動テストにしない。
  リリース前に実機で手動確認する(確認項目: 終了コード 130、ジョブの子プロセスが残らない、
  `watch` で中断したジョブが `error` で次が `pending`、`run -v` の表示に `interrupted by signal 2`)。手順は 11 章の手動の受け入れテストにある。

### 7.4 合図が立つ時点ごとの動作(流れの制御)

| 合図が立った時点 | 動作 | ステートの記録 | 終了コード |
|------------------|------|----------------|------------|
| `runJob` の呼び出しの前(`started = false`) | ジョブを起動しない | 分類なし。エラーとしては記録しない。再開位置はそのジョブ | `128 + 番号` |
| ジョブの実行中 | 1 回転送(UNIX 系)。ジョブの終了を待つ | そのジョブを**エラー**として記録(ジョブが 0 で終わっても) | `128 + 番号` |
| ジョブの終了の観測の直後(転送が間に合わなかった場合を含む) | 起動済みで `interrupted = true`(上の行と同じ扱い) | エラー | `128 + 番号` |
| `pipeline` のジョブとジョブのあいだ(ジョブの結果は成功か警告で、完了ではない) | 次のジョブを起動しない | 記述順で、まだ起動していない最初のジョブ(エラーのジョブが先にあればそれ)を再開位置にして、中断を記録。エラーとは記録しない。`-v` で `interrupted by signal N` | `128 + 番号` |
| `pipeline` の最後のジョブが完了した直後 | 完了の処理(`--remove-on-complete` ならステートを削除)をして終わる | 完了 | 0(合図は無視される) |
| リトライの待機中 | 待機を打ち切る。`started = false`、`interrupted = true`、`signalNumber = 番号` の実行結果を作る。次の実行はしない | 分類なし、再開位置はそのジョブ。エラーとしては記録しない | `128 + 番号` |
| 定義の検査・ステートの読み込み・照合の途中 | 検査は最後まで進める | (定義の誤り 64、ステートの誤り 65、拒否 66、スキップ 0 の終了コードが、中断の終了コードより優先する) | 各検査の値 |

### 7.5 リトライの待機

関数: `waitRetry(secs: bigint, sig: Interrupt): Promise<WaitOutcome>`。

```ts
export type WaitOutcome =
  | { readonly interrupted: false }                       // 経過
  | { readonly interrupted: true; readonly number: number }; // 中断(最初の番号)
```

OS 固有のコードを持たない。拒否されない `Promise` を返す。

手順(`setTimeout` と `Promise.race`):

1. 合図が立っていれば(`sig.number()` が `null` でない)、すぐ「中断(最初の番号)」を返す。
2. `secs === 0n` なら、すぐ「経過」を返す(手順 1 で、立っていなければ)。
3. 待ち時間を **ミリ秒の `bigint`** で求める: `totalMs = secs * 1000n`。`bigint` の掛け算はあふれない(`number` に落とさない)【規定】。上限は、リトライの待ち時間の飽和(64 ビット符号なしの上限。05 章)が `flow` で済んでいるので、ここでは増やさない。
4. 開始時刻を、単調な時計(`process.hrtime.bigint()`、ナノ秒)で記録する。壁時計(`Date.now()`)を使わない(時刻の調整の影響を受けない)。
5. 次を繰り返す。
   1. `elapsedMs = (process.hrtime.bigint() - 開始) / 1_000_000n`。`elapsedMs >= totalMs` なら、合図を確かめて、立っていれば「中断」、立っていなければ「経過」を返す。
   2. `remainingMs = totalMs - elapsedMs`。1 回のタイマーの長さ `chunkMs = min(remainingMs, 2^30)` ミリ秒(約 12.4 日)を `number` に直す(`Number(chunkMs)`。2^30 以下なので安全)。
   3. `Promise.race` で「`setTimeout` のタイマー」と「`sig.whenRaised()`」を待つ。待ちが終わったら、**必ず `clearTimeout` でタイマーを解除する**(`finally`)。解除しないと、合図で打ち切ったあとも、タイマーがイベントループを生かし続け、プロセスの終了が遅れる。
   4. 合図が先に解決したら「中断(最初の番号)」を返す。タイマーが先なら、手順 5.1 に戻る(早めに発火した場合や、分割した場合の続き)。
6. ポーリングをしないので、合図から返るまでは直ちにである。長い待ちでも、合図から **1 秒以内**に返る【規定】
   (例: 3600 秒の待機に、200 ms 後に合図を立てて、全体が 1.2 秒未満)。

`setTimeout` の長さの上限【実測】: 2^31 − 1 ミリ秒(約 24.8 日)を超える値、`Infinity`、`NaN` を渡すと、Node は `TimeoutOverflowWarning` を標準エラーに出し、待ち時間を **1 ミリ秒**にする。これは、リトライの待ち時間が長い定義(`interval` が数年など)で、待たずに次の実行に進む不具合になり、標準エラーに警告が出る。したがって、**1 回のタイマーを 2^30 ミリ秒以下に分ける**(手順 5.2)規定は、省略してはならない【規定】。`timers/promises` の `setTimeout` も同じ上限を持つ(未確認。使うなら、同じ分割をする)。

複数の合図は、最初の番号を返す。待機の時計は、テストで差し替える必要がない(実時間の短い待ちで、1 秒以内の打ち切りを検査できる)。ただし `cli` のテストは、`Process.wait`(`Deps`)の偽の実装で、待ち時間を記録だけして実際には待たない([02-architecture.md](02-architecture.md))。

---

## 8. プラットフォーム差と OS 分岐の規則

### 8.1 差の一覧

| 項目 | UNIX 系(Linux、macOS) | Windows |
|------|------------------------|---------|
| ジョブのプロセスグループ | 新しいグループ(`detached: true`) | 指定しない(同じコンソール) |
| 監視するシグナル | SIGHUP・SIGINT・SIGQUIT・SIGTERM | Ctrl+C のみ(番号 2。`SIGINT`) |
| ジョブへの転送 | 子の pid に、`child.kill(名前)` で 1 回(許可集合のみ) | しない(OS が Ctrl+C を配る。`child.kill` は呼ばない) |
| シグナル終了の読み取り | `close` の `signal`(名前)を、`os.constants.signals` で番号に直す | なし(終了はすべて終了コード) |
| 終了コードの範囲 | 下位 8 ビット | 32 ビット符号つきのまま(`code \| 0`。`conductor` が報告する値は、0〜255 に合わせる) |
| ステート書き込み後のディレクトリの同期 | ディレクトリを開いて同期する。失敗は無視。親が空なら `.` | しない(06 章) |
| 端末の色 | 使える(標準出力が端末で `NO_COLOR` が未設定か空のとき) | 使わない(色なし。08 章) |
| `.cmd` / `.bat` の起動 | (なし) | 4.7 節の安全策 |

### 8.2 分岐を置く場所の規則【規定】

`process.platform` を読んで分岐するのは、**`src/runner/platform.ts` の 1 か所だけ**。ほかのファイルは、`process.platform` を読まず、OS 名の文字列(`'win32'`、`'linux'`、`'darwin'`)も書かない。

```ts
export type PlatformKind = 'posix' | 'windows';

export interface Platform {
  readonly kind: PlatformKind;
  // spawn に渡す実行ファイル・引数・オプションを決める。拒否は ok:false(起動失敗として扱う)。
  prepareSpawn(line: CommandLine): PrepareResult;
  // close の (code, signal) から、終了コードとシグナル番号を取り出す(4.3.3 節)。
  readExit(code: number | null, signal: NodeJS.Signals | null): ExitReading;
  // 転送(UNIX 系は許可集合の検査のあと child.kill。Windows は何もしない)。
  forward(child: ForwardTarget, n: number): void;
  // process.on に渡すシグナル名の集合と、名前から番号への変換。
  readonly watchedSignals: readonly NodeJS.Signals[];
  signalNumberOf(name: string): number | null;
}

// process.platform を読む唯一の関数。'win32' なら windows、それ以外は posix。
export function detectPlatform(): PlatformKind;
export function platformFor(kind: PlatformKind): Platform;
```

- `platformFor('windows')` と `platformFor('posix')` は、どの OS でも作れる(単体テストで、Linux の 1 台から Windows の分岐を検査できる。`prepareSpawn` の `.cmd` の組み立て、`readExit` の `| 0`、`forward` の空の動作)。実際の起動は、実行している OS のものだけが動く。
- `detectPlatform()` の `'win32'` 以外(`linux`、`darwin`、`freebsd`、`aix` など)は、すべて `posix` として扱う。対象の OS は Linux、macOS、Windows である。それ以外は、動作を確かめていない(未確認)。
- ステートの書き込み(ディレクトリの同期の有無)と、色の判定は、`cli` が `Deps.env.platform()` で得た `PlatformKind` を、必要な部品に引数で渡す(`state` と `report` は、`runner` を import しない。[02-architecture.md](02-architecture.md))。
- 点検: `process.platform` の文字列が、`src/runner/platform.ts` 以外に現れないことを、ソースの検索で確かめる(11 章)。`OS 名の文字列`(`'win32'` など)も同じ。テスト(`test/`)は対象外。
- ネットワークに接続しない(実装がネットワークの機能を持たない)。`node:net`、`node:http`、`node:dns` などを import しない。
- 製品のコードは、外部の入力(引数・環境・ファイル・OS の結果)に対して、例外を投げる書き方をしない(失敗は値で返す。[02-architecture.md](02-architecture.md) の 9 章)。

---

## 9. イベントループを塞がない規則と、順序の保証

### 9.1 イベントループを塞がない規則【規定】

Node は、ユーザーのコードをイベントループの 1 つのスレッドで動かす。子プロセスの終了、シグナル、タイマーの通知は、すべて、ループが回って初めて届く。ループを塞ぐと、シグナルの転送が遅れ、「合図から 1 秒以内に打ち切る」規定(7.5 節)と、「子の実行中に CPU を使わない」規定が破れる。

| 規則 | 内容 |
|------|------|
| 同期の子プロセスの API を使わない | `spawnSync`、`execSync`、`execFileSync` を使わない(子の実行中に、ループを止める) |
| 待ち | `Atomics.wait`、ビジーループ(時計を見て回る)、`while (Date.now() < ...)` を使わない。待ちは、`setTimeout` と `Promise` だけ |
| 長い同期処理 | 1 回の同期処理で、約 50 ms を超える CPU の仕事(巨大な入力の構文解析など)を、子の実行中に行わない。定義の読み込みと検査は、子を起動する前に終わる。子の実行中に動くのは、`runJob` の待ちだけ |
| ファイル入出力 | `node:fs/promises` の非同期の API を使う(06 章と合わせる点)。同期の API(`readFileSync` など)は、次の場所に限る: 起動直後の設定、内部の誤りを捕捉するハンドラの中(10 章) |
| 結果を待たない `Promise` を作らない | 「投げっぱなし」(`await` も `.then` もない `Promise`)を作らない。型情報つきの ESLint の規則 `@typescript-eslint/no-floating-promises` と `no-misused-promises` を、エラーにする(11 章) |
| イベントのハンドラ | `process.on` のシグナルのハンドラと、子のイベントのハンドラは、同期の関数にする。中で `await` する非同期の関数にしない。仕事は、値を立てる、`Promise` を解決するだけ |
| 並行の実行をしない | 流れの制御(`cli`)は、`Promise.all` などで複数の仕事を並行に走らせない。ジョブ、出力、ステートの書き込みは、すべて直列に `await` する。`worker_threads` と `cluster` を使わない |
| タイマーの解除 | 作ったタイマーは、必ず解除する(7.5 節)。`unref()` で隠さない |

### 9.2 順序の保証【規定】

| 保証 | 内容 |
|------|------|
| 出力の順序 | 標準出力・標準エラーへの書き込みは、`cli` の 1 か所の関数(`Sink`)を通し、**書き込みのコールバックの完了まで `await` してから**次に進む。標準出力と標準エラーへの書き込みの順序が、呼び出しの順序と一致する(2 つのストリームの書き込みが、非同期になりうるため。標準出力がパイプのとき、書き込みが同期か非同期かは、OS と Node の版による。未確認。Node の文書では、パイプは Linux で同期、Windows で非同期、とされる。実測していない) |
| 子の出力との順序 | `conductor` は、子を起動する前に、自分の出力の書き込みを完了させる(`await`)。子の実行中に、リトライの知らせ以外の出力を書かない。子の出力(子が直接ファイル記述子へ書く)と、`conductor` の出力が、混ざらない。リトライの知らせは、子の `close` のあと、次の起動の前に、書いて完了を待つ |
| ステートの順序 | ステートの書き込み(一時ファイルへの書き込み、同期、名前の付け替え、必要ならディレクトリの同期)の完了を `await` してから、次のジョブを起動する。「記録して(Durable)から次へ」を破らない(05・06 章) |
| 出力とステートの順序 | 05 章の順序に従う。ジョブごとに、`-v` の結果行などの表示を書き(完了を待ち)、そのあとステートを書く。順序を入れ替えない |
| 合図の読み取り | `sig.number()` は、`await` の前後で値が変わりうる(シグナルが割り込む)。同じ判断のなかで 2 回読んで食い違いを起こさない。1 回読んで、その値を使う |
| 終了の前の flush | `process.exit` を呼ぶ前に、標準出力と標準エラーの未完了の書き込みを完了させる(9.3 節)。書き込みの完了前に `process.exit()` を呼ぶと、出力が失われうる(Node の文書による) |
| ハンドラの副作用 | シグナルのハンドラは、出力もステートの書き込みもしない(7.1 節)。したがって、ハンドラが出力・ステートの順序を乱すことはない |

### 9.3 終了と flush【Node の設計】

`bin/conductor.ts` の入口が、終了を 1 か所で行う。

1. 最初に、内部の誤りの捕捉(10 章)を設定する。
2. `const code = await run(...)`(`cli.run`)。
3. 標準出力と標準エラーの、未完了の書き込みを待つ(`write('', callback)` の完了、または `Sink` が溜めている書き込みの完了を待つ)。待ちの失敗(`EPIPE` など)は無視する。
4. `process.exit(code)`(または `process.exitCode = code` を設定して終わる)。**どちらの場合も、`run` の戻り値で、1 回だけ**。

標準出力が閉じたパイプ(`conductor ... | head -1` など)のとき、Node は `SIGPIPE` でプロセスを終了させず、書き込みの失敗(`EPIPE`)を、書き込みのコールバックと、ストリームの `error` イベントの**両方**で通知する【実測】。`error` イベントのリスナーが無いと、未処理の例外になり、プロセスが落ちる(終了コード 70 になる)。したがって、`Sink` を作るとき、`process.stdout` と `process.stderr` に `'error'` のリスナーを**必ず付ける**(中身は、失敗を記録するだけ)。書き込みの失敗は値として返し、`watch` と `convert` で終了コード 65・67 にする(08 章)。

`run` の戻り値(流れの結果)以外の理由で `process.exit` を呼ぶのは、10 章の内部の誤りのハンドラだけ。

---

## 10. 想定外の内部の誤り

- `src/bin/conductor.ts` の入口に、想定外の内部の誤り(実装の不具合による例外・`Promise` の拒否)を捕まえる仕組みを、**最初に**入れる。具体的には、次の 3 つを、`run` を呼ぶ前に設定する。
  1. `process.on('uncaughtException', handler)`
  2. `process.on('unhandledRejection', handler)`
  3. 入口の `try { ... } catch { ... }`(`await run(...)` を囲む)
- `handler` と `catch` は、同じ関数(`failInternal()`)を呼ぶ。内部の誤りが起きると、標準エラーに固定の英語の 1 行 `conductor: error: internal error`(ロケールを判定しない)を書き、終了コード **70** で終わる。標準エラーへの書き込みが失敗しても 70 で終わる。【規定】
  - 書き込みは、**同期**の `fs.writeSync(2, ...)` を `try/catch` で囲んで行う(ハンドラの中で、非同期の書き込みの完了を待てないため。`EAGAIN` などの失敗は無視する)【Node の設計】。
  - `failInternal()` は、繰り返し呼ばれても 1 行だけ出す(印を持つ)。すでに終了を始めていたら、何もしない。
  - 終了は `process.exit(70)`。
  - 例外の内容(メッセージ、スタック)を、標準エラーにも標準出力にも出さない。
- 入口は、流れの結果の終了コードで、プロセスを 1 回だけ終了させる(9.3 節)。入口に、わざと例外を起こすコマンドを持たない。内部の誤りの検査は、`run` に差し替えた(例外を投げる)`Deps` を渡す単体テスト、または別のテスト用の実行ファイル(`test/jobs/` ではなく、入口を差し替えて起動する小さなスクリプト)で行う([10-errors-and-exit-codes.md](10-errors-and-exit-codes.md))。
- ジョブの実行中に内部の誤りが起きると、子プロセスの終了を待たずに `process.exit(70)` で終わり、ジョブが孤児として残りうる。ステートは直前の完全な内容のままで、
  次の実行はそのジョブを再実行しうる(少なくとも 1 回の実行の範囲)。
- `conductor` が SIGKILL などで強制終了されたときも、ジョブは残って動き続けうる(検知も停止もしない)。
- 外部の入力(引数・環境・ファイル・OS の結果)に対して、例外を投げる書き方(添字の範囲外、`null` の参照、非 `null` 表明 `!`、`as` による型の偽り、`JSON.parse` の例外の素通し)を製品のコードに書かない(失敗は値で返す)。Node の API が例外を投げる場合(`spawn` の引数の誤りなど)は、その場で `try/catch` して値にする。

---

## 11. 検証の観点

| 確かめること | 検査の区分と置き場所の案 |
|--------------|--------------------------|
| 分割の全規則(3 章の表。受理・拒否。サロゲートペアを含む例) | `test/runner/split.test.ts` の表駆動の単体テスト |
| ランダムな `command`(3 章の文字集合)で、成功なら `program` が空でない。約 30 万引数が分割できる(`RangeError` にならない)。閉じない引用符は誤り | 同上(固定の種のランダム) |
| メタ文字が引数にそのまま渡る、配列の要素がそのまま渡る、環境変数と作業ディレクトリの引き継ぎ、起動失敗(`ENOENT`・`EACCES`・NUL を含む引数)、終了コード(0・3・255・300) | `test/jobs/` のテストジョブを起動する `runner` のテスト(実プロセス) |
| 標準入力・標準出力・標準エラーの引き継ぎ(`stdio: 'inherit'` であること) | 同上(ジョブが標準入力を読み、標準出力へ書く) |
| 起動前の合図で起動しない、範囲外の番号(負・0・9999)でも起動せず中断として扱う、終了の観測の直後の合図で `interrupted` | `runJob` の単体テスト(`raise` してから呼ぶ) |
| `runJob` の転送: 転送が 1 回だけ、先に `exit` した子へ転送しない、2 回目以降の合図で追加の転送をしない、`error` のあとの `close`(負の値)を無視する | 偽の `ChildLike` による単体テスト(5.3 節) |
| SIGTERM / SIGINT の転送、許可外の番号は送らない、新しいプロセスグループ(子の `PGID = PID`)、先に終わったジョブは転送なし(UNIX 系) | 実プロセスのテスト(UNIX 系) |
| 実バイナリで SIGTERM → 143、ステートに再開位置、孤児なし。HUP のあとの INT → 129 | 受け入れテスト(UNIX 系。Node の実行ファイルを、shim を介さず直接起動する。1.2 節) |
| `waitRetry` の 1 秒以内の打ち切り(3600 秒の待機に、200 ms 後の合図で、全体が 1.2 秒未満)、打ち切ったあとにタイマーが残らない、2^31 ミリ秒を超える秒数で警告が出ず、待たずに返らない(`secs` が大きい `bigint`) | `waitRetry` の単体テスト(大きい値は、合図で打ち切って検査する) |
| 監視の登録(最初の番号が残る、2 回目の登録は `alreadyRegistered`、登録失敗で 70、`stop` でリスナーが残らない) | 単体テスト(登録失敗は、偽の `Signals`) |
| ジョブの実行中(約 3 秒)の CPU 時間が 100 ms 以下(UNIX 系) | 計測(強制。イベント駆動なので満たす想定。実測する) |
| 標準出力が閉じたパイプでも、プロセスが未処理の例外で落ちない | 受け入れテスト(`| head -c 1`) |
| 内部の誤りで 70、固定の 1 行、`unhandledRejection` でも 70 | 入口を差し替える単体テスト |
| Windows の Ctrl+C | 手動(11 章) |
| Windows の `.cmd` への危険な引数が注入にならない(AC-113〜115) | Windows の CI のテスト。加えて `platformFor('windows').prepareSpawn` の単体テスト(どの OS でも動く) |
| `process.platform` が `platform.ts` 以外にない | ソースの検索(11 章) |

テストでスクリプトを書いてすぐに実行する場合、Linux で、実行ファイルを書いた直後の起動が「text file busy」(`ETXTBSY`)で失敗することがある(別の子プロセスの `fork` が、書き込み用のファイル記述子を一時的に持つため。一般に知られた事象)。テストのスクリプトは、あらかじめ作っておく(`test/jobs/` に置く)か、起動前にファイルを閉じて待つ。Node での実際の発生と回避策の有効性は未確認。

---

## 12. 規定と実装の裁量の区別、未確認の事項

### 12.1 規定と裁量(まとめ)

| 項目 | 区別 |
|------|------|
| 分割の状態機械と、受理・拒否の例 | 【規定】 |
| シェルを介さない起動(`shell: false`)、作業ディレクトリ・環境変数・標準入出力の引き継ぎ(`stdio: 'inherit'`、`cwd`・`env` を渡さない) | 【規定】 |
| 起動失敗の終了コード 127、分類の優先順位 | 【規定】 |
| 転送は 1 回だけ、許可する 4 つのシグナルだけ、子の pid だけ(`child.kill`)、終了済みには送らない | 【規定】 |
| UNIX 系でジョブを新しいプロセスグループで動かす | 【規定】(`detached: true` は【Node の設計】) |
| Windows で転送をしない(`child.kill` を呼ばない。OS が Ctrl+C を配る) | 【規定】 |
| `run`/`pipeline` だけが、定義の検査より前にシグナルを準備する | 【規定】 |
| シグナルの準備の失敗は 70、内部の誤りは 70 | 【規定】 |
| リトライの待機が合図から 1 秒以内に返る、タイマーを 2^30 ミリ秒以下に分ける | 【規定】 |
| 出力とステートの書き込みは直列に `await` する。シグナルのハンドラは `raise` だけ | 【規定】 |
| `process.platform` を読むのは `platform.ts` の 1 か所 | 【規定】 |
| `close` による終了の観測、`exit` での終了済みの印、イベント駆動の待ち | 【Node の設計】 |
| `Interrupt` の型の構造、`ChildLike` の形 | 【Node の設計】 |
| `.cmd` / `.bat` の引数の禁止文字の集合と、`cmd.exe` の起動の形 | 【Node の設計】(Windows の実機で未確認) |
| ほぼ同時に届いた複数のシグナルのどれが「最初」か | 規定しない |

### 12.2 未確認の事項

- Windows での実機の動作(Ctrl+C の終了コード 130、ジョブの終了の様子、`.cmd` の起動、`cmd.exe` の起動の形、`ComSpec`)は、ここでは確認していない。
- Windows で `child.kill` が子を強制終了する、`detached: true` が新しいコンソールを作る、Ctrl+C が `SIGINT` として配られる、の 3 点は、Node の文書の記述による。実測していない。
- Windows で、`close` の終了コードが符号なし 32 ビットの値(例: 3221225786)で返るか、符号つきで返るか。この仕様は `| 0` で、どちらでも正しくなるようにしている。
- Windows で Ctrl+Break(`SIGBREAK`)とコンソールを閉じたときの通知(`SIGHUP`)を受け取るか(対象外としている。確認していない)。
- Windows でのプログラム探索(拡張子の補完、カレントディレクトリを含むか等)の詳細は、Node と OS の標準の規則に依存し、確認していない。`program` が拡張子なしの `.cmd` を指す場合、Node が補完するかも未確認。
- 採用する Node の版が、`.cmd` / `.bat` を `shell: true` なしで `spawn` したとき、どの版から、どの `error.code`(`EINVAL` かどうか)で失敗するか。
- Linux での実測は行っていない(macOS だけ)。特に、`EACCES` の `close` の値、`detached: true` による `PGID`、シグナル名と番号の対応(`SIGUSR1` など)は、Linux では未確認。
- Node v24 以外の版(特に、最小版として採る古い LTS)での、4.3 節と 7 章の【実測】の結果は未確認(`process.on` の解除の実測だけ v24.22.2 でも確認した)。
- UNIX 系で、`detached: true` による新しいセッション(制御端末なし)のために、ジョブが端末から読む(標準入力が端末のとき)場合、`/dev/tty` を開く場合の動作は、確認していない(受け入れテストは、標準入力を空にして実行する)。
- `child.kill` と、子の終了の回収・`exit` の配送の間に、`conductor` のコードが割り込めるか(5.2 節)。Node と libuv の内部の実装に依存し、確認していない。割り込めても、回収前の pid は再利用されない前提である。
- シグナルのハンドラの第 2 引数(番号)が、すべての OS・版で渡されるか(この仕様は、名前から固定の表で番号を引くので、依存しない)。
- `process.on` が例外を投げる条件(登録の失敗が実際に起こるか)。
- ほぼ同時に届いた複数のシグナルの順序(Node が配る順序)。
- `spawn` に約 30 万個の引数を渡したときの失敗(`E2BIG` などが `error` イベントで来るか。この仕様は、分割だけが 30 万引数で壊れないことを要求し、起動は要求しない)。語の配列を引数に展開したときの `RangeError` が出る個数。
- `timers/promises` の `setTimeout` が、`setTimeout` と同じ上限を持つか。
- 標準出力がパイプのとき、書き込みが同期か非同期か(OS と Node の版による。Node の文書の記述のみ)。
- 子プロセスでの `SIGPIPE` の扱い: Node は自身で `SIGPIPE` を無視するが、子の `SIGPIPE` は既定に戻される、と推測している。`yes | head -1` を子の中で動かして、警告が出ないことを見たが、決定的な検査ではない【実測の補足】。
- Linux での `ETXTBSY`(11 章の注意)の発生の条件と回避策。
- 本文書の Node の API の挙動の記述のうち【実測】の印がないものは、Node の公式のドキュメントの知識によるもので、コードを実行して確かめていない。
