# 07 プロセス起動とシグナル(`command` の分割・起動・待ち・中断)

対象: `conductor` v0.8.0 と同じ挙動を Go で実装するための仕様。この文書は、次を定める。

- `command` の文字列を、コマンド名と引数に分ける規則(3 章)
- `os/exec` によるジョブ(子プロセス)の起動、待ち、結果の取得、起動失敗(4・5 章)
- 中断の合図と、OS ごと(UNIX 系と Windows)のシグナル処理(6・7 章)
- OS ごとの差とビルドタグ(8 章)
- goroutine とチャネルの設計、データ競合の避け方(9 章)
- 想定外の内部の誤り(10 章)

パッケージ構成と依存の向きは [02-architecture.md](02-architecture.md)、定義ファイルの検査は [03-definition-format.md](03-definition-format.md)、
ステートファイルは [06-state-file.md](06-state-file.md)、流れ全体は [05-execution-flow.md](05-execution-flow.md)、
表示の文言は [08-output-and-i18n.md](08-output-and-i18n.md)、終了コードの全件は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md)、
テストと品質ゲートは [11-quality-and-build.md](11-quality-and-build.md) にある。この文書は、必要な箇所だけ触れる。

## 規定の強さの表記

| 表記 | 意味 |
|------|------|
| 【規定】 | どの実装も満たす。受け入れテストで確かめる |
| 【目安】 | 推奨の値。異なる値にしてよい。ただし、利用者から見える結果は変えない |
| 【任意】 | 実装に任せる。利用者から見える結果に影響しない |
| 【Go の設計】 | この Go 版が採る設計。利用者から見える挙動は変わらないが、実装者はこれに従う |

「未確認」は、この文書を書いた時点で、Go の API の挙動を確かめていない点を指す。実装時に、採用する Go の版のドキュメントとテストで確かめる(12 章)。

---

## 1. 構成要素と責務(パッケージとファイル)

すべて `internal/runner` パッケージに置く(分割だけは、純粋なので同じパッケージの別ファイル)。

| 構成要素 | 責務 | 置き場所(ファイル) |
|----------|------|---------------------|
| 分割器 | `command` の文字列を、コマンド名と引数に分ける。`rune` を 1 つずつ左から読む状態機械。再帰を使わない。3 OS で同じ規則 | `split.go` |
| `command` の検査 | `command` を起動せずに検査する。成功なら分けた結果(`CommandLine`)を返す | `split.go` |
| ジョブの起動と待ち | 1 つのジョブをシェルを介さずに起動し、終了を待ち、実行結果(`flow.ExecResult`)を返す。失敗は値で返し、`panic` しない | `run.go` |
| 待機ループ | ジョブの終了を待つ。中断の合図の転送の判断もここで行う(5 章) | `wait_job.go` |
| リトライの待機 | リトライの前に待つ。待機中に中断の合図を見る(7.5 節) | `wait_retry.go` |
| 中断の合図 | 「中断が要求された」ことと、最初に受け取ったシグナルの番号を、goroutine の間で共有する値(6 章) | `interrupt.go` |
| プラットフォーム層 | 新しいプロセスグループの指定、シグナルの送信、シグナル終了の読み取り、受け取るシグナルの集合、`.cmd` の安全策。OS ごとに実装が違う | `platform_unix.go`、`platform_windows.go` |
| シグナル監視 | OS のシグナルを受け取り、中断の合図を立てる | `watcher.go`(共通)、`platform_*.go`(OS 別の部分) |

`flow.ExecResult`(実行結果)と、分類(実行結果 → 成功・警告・エラー・分類なし)は `internal/flow` にある([02-architecture.md](02-architecture.md) と [05-execution-flow.md](05-execution-flow.md))。`runner` は分類をしない。

OS ごとの分岐と OS 固有の呼び出し(`syscall` の OS 固有の名前を含む)は、`platform_unix.go` と `platform_windows.go` の外に置かない【規定】。
共通のファイルには `runtime.GOOS` による分岐も、`syscall.SIGHUP` のような UNIX 固有の名前も書かない。

---

## 2. `command` の形式(定義の側の受理条件)

定義ファイルのジョブの `command` は、次の 2 つの形式のどちらか(定義の書式は YAML または JSON。[03-definition-format.md](03-definition-format.md))。

| 形式 | YAML での書き方 | 起動のときの扱い |
|------|-----------------|------------------|
| 文字列形式 | `command: "sh scripts/x.sh"` | 3 章の規則で、コマンド名と引数に分ける |
| 配列形式 | `command: ["sh", "-c", "echo hi"]`(JSON では文字列の配列) | 要素をそのまま使う(分割・引用符の処理・展開をしない) |

定義の読み込み時の検査(誤りは定義の誤り、終了コード 64):

| 入力 | 判定 |
|------|------|
| `command` キーがない | 誤り(必須の値がない) |
| 文字列で、前後の空白(Unicode の空白)を除くと空 | 誤り(`command` が空) |
| 配列が空(`[]`) | 誤り(配列が空) |
| 配列に文字列でない要素がある | 誤り(要素は文字列で書く)。要素ごとに報告する |
| 配列の先頭の要素が、前後の空白を除くと空 | 誤り(先頭の要素が空) |
| 配列の 2 番目以降が空文字列 `""` | 有効(空の引数として渡す) |
| 文字列でも配列でもない(数値・マップなど) | 誤り(型) |

定義の検査とは別に、実行の前にもう一度、全ジョブの `command` を 3.6 節のとおり検査する。

Go の型(`definition.Command`)は、文字列形式と配列形式を区別して持つ([02-architecture.md](02-architecture.md) の骨格)。`runner` は、この型だけを見る。

---

## 3. `command` の分割規則

### 3.1 入力と出力

- 入力: `command` の文字列(`string`)。
- 出力: コマンド名(`Program`。空でない)と、引数の並び(`Args`)。`Args` の各要素が、そのまま 1 つの引数になる。
- シェルは使わない。変数展開(`$HOME`)、グロブ(`*`)、パイプ(`|`)、`;`、`&&`、リダイレクト(`>`)、バッククォート、`~`、
  括弧は **解釈しない**。普通の文字として引数に入る。パイプなどが要るときは、利用者が `sh -c '...'` を `command` に書く。

### 3.2 文法

文字は Unicode のコードポイント単位(Go の `rune`。UTF-16 の符号単位でも、書記素でもない)で、左から右へ 1 回だけ読む。
区切り文字は次の 4 つだけ。

```text
separator := ' ' (U+0020) | '\t' (U+0009) | '\n' (U+000A) | '\r' (U+000D)
```

(U+000B、U+000C、U+00A0 などのほかの空白は区切りではなく、普通の文字。`unicode.IsSpace` を使ってはならない。)

状態は 4 つ。これに加えて、`inWord`(語の途中か)と `current`(組み立て中の語)を持つ。

| 状態 | 入力の文字 | 動作 | 次の状態 |
|------|-----------|------|----------|
| Plain(引用符の外) | separator | `inWord` なら `current` を語として確定し、空にする。`inWord = false` | Plain |
| Plain | `'` | `inWord = true`(語を開始。まだ何も足さない) | Single |
| Plain | `"` | `inWord = true` | Double |
| Plain | 上記以外(`\` を含む) | `inWord = true`。その文字を `current` に足す | Plain |
| Single(`'...'` の中) | `'` | 何も足さない | Plain |
| Single | 上記以外(`\`、`"`、改行を含む) | その文字を足す | Single |
| Double(`"..."` の中) | `"` | 何も足さない | Plain |
| Double | `\` | 何も足さない(バックスラッシュを保留する) | DoubleBackslash |
| Double | 上記以外 | その文字を足す | Double |
| DoubleBackslash(`"..."` の中で `\` を 1 つ保留中) | `"` | `"` を 1 つ足す(保留の `\` は消える) | Double |
| DoubleBackslash | `\` | 保留していた `\` を 1 つ足す。今読んだ `\` が新しい保留になる | DoubleBackslash |
| DoubleBackslash | 上記以外の文字 c | `\` と c を足す | Double |

入力の終わりでの確定:

1. 状態が Plain でなければ(Single / Double / DoubleBackslash)、**閉じていない引用符**(誤りの種類 `UnclosedQuote`)。
2. `inWord` なら `current` を最後の語として確定する。
3. 語の列の先頭がなければ、または先頭の語が空文字列なら、**コマンド名がない**(誤りの種類 `Empty`)。それ以外は、先頭の語が `Program`、
   残りが `Args`。

要点:

- 引用符で囲んだ部分と囲まない部分は、隣り合っていればつなげて 1 つの語になる(`a"b c"'d e'f` は 1 語 `ab cd ef`)。
- `''` と `""` は、空の語(空の引数)になる(`inWord` が立つため)。ただし先頭の語が空なら `Empty`。
- 二重引用符の中で解釈するバックスラッシュは、「直後が `"`」のときだけ(`\"` → `"`)。`\n`、`\t`、`\\` なども、
  **解釈せずそのまま**(`\n` は 2 文字のまま)。ただし `\\` は、1 つ目のバックスラッシュを出力し、2 つ目を新しい保留にする。
  したがって `\\"` は「`\` と、エスケープされた `"`」になる(引用符は閉じない)。`\` の個数の偶奇で数える規則ではなく、
  直前に保留している `\` が 1 つあるかどうかだけで決まるので、上の表を左から機械的に適用すること
  (`"` の直前に `\` が 1 つ以上並んでいれば、`"` は常にエスケープされ、引用符は閉じない)。
- 引用符の外のバックスラッシュは普通の文字(Windows のパス `C:\dir\x.exe` がそのまま書ける)。一重引用符の中も同じ。
- 末尾がバックスラッシュのパスを二重引用符で囲むと `\"` が引用符のエスケープになり、閉じない(誤り)。一重引用符で書く。
- 文字列の長さに上限はなく、入れ子もないため、再帰やスタックの深さに依存しない(約 30 万引数の入力でも分割できる)。【規定】

### 3.3 Go での実装上の規定

| 項目 | 規定 |
|------|------|
| シグネチャ | `func Split(s string) (program string, args []string, err error)`。`err` は `*CommandError`(`Kind` が `UnclosedQuote` か `Empty`)。`Split` は純粋(入出力・環境・時刻に触れない) |
| 走査 | `for _, r := range s` で `rune` を 1 つずつ読む。`s[i]` のバイト単位の走査は使わない。入力は有効な UTF-8 を前提とする(`range` は不正なバイトを U+FFFD として読む。定義の読み込み側が不正な UTF-8 をどう扱うかは [03-definition-format.md](03-definition-format.md) に従う) |
| 組み立て | `current` は `strings.Builder`(または再利用する `[]rune`)。語を確定するたびに `Reset` する。語の列は `[]string`。計算量は入力の長さに比例させる(約 30 万引数で、2 乗の時間にならない) |
| 再帰・正規表現 | 使わない。`strings.Fields`、`strings.Split`、`shlex` 系のライブラリも使わない(規則が違う) |
| 空の `Args` | 引数がないときは、長さ 0 の `[]string`(`nil` でも `[]string{}` でもよいが、テストは `len` で比べる) |
| 状態 | `type splitState int` の定数 4 つ(`stPlain`、`stSingle`、`stDouble`、`stDoubleBackslash`)。`switch state` の中で `switch r` を入れ子にする |
| 誤りの文 | `CommandError` は、種類だけを持ち、入力の値を保持しない(3.6 節) |

擬似コード(同じ結果になる実装であれば、書き方は自由):

```text
words = []; current = ""; inWord = false; state = Plain
for r in runes(s):
  switch state:
    Plain:  if r in separator: if inWord: words.push(current); current = ""; inWord = false
            elif r == "'": inWord = true; state = Single
            elif r == '"': inWord = true; state = Double
            else: inWord = true; current += r
    Single: if r == "'": state = Plain else current += r
    Double: if r == '"': state = Plain
            elif r == '\\': state = DoubleBackslash
            else: current += r
    DoubleBackslash:
            if r == '"': current += '"'; state = Double
            elif r == '\\': current += '\\'            # state は DoubleBackslash のまま
            else: current += '\\' + r; state = Double
if state != Plain: return 誤り(UnclosedQuote)
if inWord: words.push(current)
if words が空 or words[0] == "": return 誤り(Empty)
return (words[0], words[1:])
```

### 3.4 受理の例

表の「入力」は、`command` に書く文字列そのもの(Go の文字列リテラルの引用符やエスケープではない)。`␣` は空白、`⇥` はタブ、`⏎` は改行。

| 入力(command の文字列) | Program | Args |
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
| `a 'b c'` | `a` | `b c` |

### 3.5 拒否の例

| 入力 | 誤りの種類 |
|------|-----------|
| `echo 'a` | `UnclosedQuote` |
| `echo "a` | `UnclosedQuote` |
| `echo "a\`(末尾が二重引用符の中のバックスラッシュ) | `UnclosedQuote` |
| `echo "a\\" b`(`\\` + `"` は、引用符のエスケープ) | `UnclosedQuote` |
| `prog "C:\dir\"` | `UnclosedQuote` |
| (空文字列) | `Empty` |
| `␣⇥⏎`(区切りだけ) | `Empty` |
| `''` | `Empty`(先頭の語が空) |
| `'' x` | `Empty`(先頭の語が空) |

### 3.6 配列形式の扱い

- 要素を一切加工せずに使う。先頭が `Program`、残りが `Args`。分割、引用符の解釈、展開、前後の空白の除去をしない
  (`"  lead and trail  "` も、そのまま 1 つの引数)。
- 先頭の要素が、前後の空白を除いて空(空文字列・空白だけ)、または要素が 0 個のときは `Empty`(定義の検査の二重の備え)。
  先頭の要素が空白を含むが空でないとき(`"my prog"`)は、そのまま `Program`(`my prog` というファイル名を起動する)。
- 2 番目以降は空文字列でもよい(空の引数)。

### 3.7 誤りの表示と検査のタイミング

| 項目 | 内容 |
|------|------|
| 誤りの内容 | 種類(`UnclosedQuote` か `Empty`)と、固定の文 |
| 固定の文 | `UnclosedQuote` は `引用符が閉じていません`、`Empty` は `コマンド名がありません`。**入力の値を含めない**(秘密の値が `command` に入っていても表示に出ない)【規定】。`CommandError.Error()` も、この固定の文だけを返す |
| 利用者への表示(標準エラー) | `conductor: error: <定義ファイルのパス>: job '<ジョブID>': <文>`。文は表示言語に従う。英語は `command has an unclosed quote` / `command is empty`、日本語は `command の引用符が閉じていません` / `command が空です`(組み立ては `report`。[08-output-and-i18n.md](08-output-and-i18n.md)) |
| 検査の時点 | `run` / `pipeline` / `validate` は、定義の読み込みと検証のあと、**全ジョブ**の `command` を検査し、誤りのあるジョブをすべて報告して(1 ジョブ 1 行)、終了コード 64 で終わる。ジョブは 1 つも起動せず、ステートにも触れない |
| 順序 | シグナルの準備(`run`/`pipeline` のみ)→ 定義の読み込み(64)→ `command` の検査(64)→ ジョブID(68)→ run ID(68)→ ステートの読み込み(65)→ 照合(65) |
| 防御 | ジョブの起動の内側でも分割し直す。万一失敗したら、起動失敗の実行結果を返す(通常は到達しない) |

検査の関数: `func CheckCommand(c definition.Command) (CommandLine, error)`。`CommandLine` は `Program string` と `Args []string` を持つ。

---

## 4. プロセスの起動(`os/exec`)

### 4.1 手順(時系列)

関数: `func RunJob(job definition.Job, sig *Interrupt) flow.ExecResult`(実際の起動は、`Process` の本番の実装が呼ぶ。[02-architecture.md](02-architecture.md))。
失敗はすべて戻り値で返し、`panic` しない。

1. 実行結果を初期化する: `Started = true`、`ExitCode = nil`、`TerminatedBySignal = nil`、`StartFailure = false`、
   `Interrupted = false`、`SignalNumber = nil`。
2. `command` を分割する(文字列は 3 章、配列は 3.6 節)。失敗なら `StartFailure = true` で返す。
3. **起動の前**に合図が立っていれば(`sig.Number()` が `ok`)、起動せずに返す: `Started = false`、`Interrupted = true`、`SignalNumber = 番号`。
   (ジョブは実行されない。)
4. `cmd := exec.Command(program, args...)` で起動の準備をし、4.2 節のとおり設定する。プラットフォーム層の設定を適用する
   (UNIX 系: 新しいプロセスグループ。Windows: `.cmd` / `.bat` の安全策。8 章)。
5. `cmd.Start()`。失敗(`error` が返る)なら `StartFailure = true` で返す(`Started` は `true` のまま)。原因は区別しない。
6. 待機用の goroutine を 1 つ起こして `cmd.Wait()` を呼び、結果をチャネルで受ける。5 章の待機ループで終了を待つ。
   時間切れ(タイムアウト)はない。自動のリトライもしない(リトライは `cli` の流れの制御で行う。7.5 節)。
7. 終了を観測したら(待機用の goroutine から結果を受け取ったら):
   - `cmd.ProcessState` から `ExitCode` と `TerminatedBySignal` を求める(4.3 節)。
   - **この時点で** 合図が立っていれば `Interrupted = true`、`SignalNumber = 番号`。
8. `Wait` が返した `error` が `nil` でも `*exec.ExitError` でもない、または `cmd.ProcessState` が `nil` のときは、終了の状態を取得できないので、`StartFailure = true` で返す(分類で 127)。

### 4.2 起動の条件(`exec.Cmd` の設定)

| 項目 | 規定 |
|------|------|
| シェル | 使わない。`sh -c` や `cmd /C` を挟まない【規定】。`exec.Command` に渡すのは、分割(または配列)で得た `Program` と `Args` だけ |
| 使う API | `exec.Command`(`exec.CommandContext` は使わない)。`CommandContext` は、コンテキストが終わると子を強制終了する(強制終了しない規定に反する)ため。`cmd.Cancel`、`cmd.WaitDelay` も設定しない |
| プログラムの探索 | `exec.Command` の標準の規則に従う(`Program` に区切り文字がなければ `PATH` で探索)。`conductor` 独自の探索はしない。`exec.ErrDot`(`PATH` の探索がカレントディレクトリの相対の位置に解決された場合のエラー。Go 1.19 以降)や、Windows での探索の細部も、標準の規則に従い、起動失敗として扱う(12 章) |
| 作業ディレクトリ | `cmd.Dir = ""`(`conductor` のカレントディレクトリを引き継ぐ。定義ファイルの場所に移らない)。変更しない【規定】 |
| 環境変数 | `cmd.Env = nil`(すべて引き継ぐ)。追加・削除・上書きをしない。run ID も環境変数としては渡さない【規定】 |
| 標準入力・標準出力・標準エラー | `cmd.Stdin = os.Stdin`、`cmd.Stdout = os.Stdout`、`cmd.Stderr = os.Stderr`と**明示する**【規定】。`nil` のままにすると、子のファイルは空のデバイス(`os.DevNull`)につながり、引き継がれない。**`*os.File` を直接渡す**こと(`io.Writer` に包む、`Deps` の `Sink` を渡す、とすると、`os/exec` がパイプとコピー用の goroutine を作り、引き継ぎにならない)。`conductor` は、ジョブの出力を読まない・保存しない・ステートに書かない【規定】 |
| 追加のファイル | `cmd.ExtraFiles` は使わない |
| 引数の渡し方 | 分割した各引数を、そのまま `exec.Command` に渡す(メタ文字は注入にならない)【規定】。Windows で `.cmd` / `.bat` を起動するときは 8.3 節の安全策を適用する |
| 起動できなかった場合 | `exec.Command` の探索の失敗(`cmd.Err`、`Start` が返す)、実行権限がない、実行できない形式、ディレクトリを指定した、などは、原因を区別せず `StartFailure = true`。終了コードは 127(分類)。表示は `conductor: error: job '<ID>': cannot start command '<プログラム名>'`(英語。組み立ては `report`)。ジョブの終了コード・シグナルは なし |
| 時間の上限 | ない。ジョブが終わらなければ、`conductor` は待ち続ける |
| ジョブの強制終了 | しない(`Process.Kill` を呼ばない。強制終了のシグナルを送る機能はない) |

### 4.3 実行結果の取り出し

`cmd.Wait()` が返したあと、`cmd.ProcessState`(`*os.ProcessState`)から次のように読む。

| 項目 | UNIX 系(`platform_unix.go`) | Windows(`platform_windows.go`) |
|------|----------------------------|--------------------------------|
| 終了した | `ps.Sys().(syscall.WaitStatus)` を取り、`ws.Signaled()` が偽なら、`ExitCode = ps.ExitCode()`(下位 8 ビットの 0〜255) | `ExitCode = int(int32(ps.ExitCode()))`(32 ビット符号つきとして読む。下記) |
| シグナルで終了した | `ws.Signaled()` が真なら、`TerminatedBySignal = int(ws.Signal())`、`ExitCode = nil`。`ps.ExitCode()` は `-1` を返す(使わない) | なし(`TerminatedBySignal` は常に `nil`) |

- 終了コードの値の幅(UNIX 系は下位 8 ビット、Windows は 32 ビット符号つき)は、[05-execution-flow.md](05-execution-flow.md) と [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md) のとおり(`300` を返すと UNIX 系は 44、Windows は 300 のまま。負の値は Windows で負のまま)。
- Windows で `ps.ExitCode()` が 32 ビットの符号なしの値を `int` に直した正の値を返す場合に備え(例: `-1` を返すプロセスが 4294967295 になる)、`int32` へ直してから使う。`ps.ExitCode()` の値の範囲は**未確認**(12 章)。`int32` への変換は、どちらの場合でも正しい値になる。
- `*exec.ExitError` は、終了コードが 0 でない、またはシグナルで終了したときに、`Wait` が返す。`errors.As(err, &exitErr)` で判定するが、結果の取り出しは `cmd.ProcessState`(`exitErr.ProcessState` と同じ)から行い、`err` の文字列は使わない(入力の値を表示に出さない。[10-errors-and-exit-codes.md](10-errors-and-exit-codes.md))。
- ジョブが、停止(`SIGSTOP`)や再開で `Wait` を戻らせることはない(`Wait` は終了だけを待つ。停止の通知を受けるフラグを使わない)。未確認の点は 12 章。

実行結果(`flow.ExecResult`)の各項目:

| 項目 | 意味 |
|------|------|
| `JobID`, `Command` | 定義のジョブIDと `command`(表示とステートには `JobID` だけを使う。`command` はステートに書かない) |
| `Started` | 起動を試みたか。`false` は、起動の前に合図が立っていたときだけ |
| `ExitCode` | `*int`。OS が報告する終了コード。UNIX 系でシグナルによって終了したときは `nil`。Windows は常に値がある |
| `TerminatedBySignal` | `*int`。UNIX 系のシグナル終了の番号。Windows は常に `nil` |
| `StartFailure` | 分割の失敗、起動の失敗、終了の状態の取得の失敗のいずれか |
| `Interrupted` | 起動前、または、終了の観測の直後に、合図が立っていた |
| `SignalNumber` | `*int`。`Interrupted` のとき、**最初に受け取った**シグナルの番号 |

### 4.4 分類(終了コードの決定)

分類は `flow` の純粋な関数が行う。ここでは、規定の優先順位を再掲する。実行結果は、次の優先順位で、最初に当てはまるものを採る。

| 順 | 条件 | 分類 | 報告する終了コード |
|----|------|------|--------------------|
| 1 | `Started = false` かつ `Interrupted` | 分類なし(エラーとして記録しない) | `128 + SignalNumber` |
| 2 | `Started = false` で中断でもない(起こらない) | エラー | 127 |
| 3 | `Interrupted`(起動済み) | エラー | `128 + SignalNumber`(ジョブ自身の終了コードは見ない。ジョブが 0 で終わっていてもエラー) |
| 4 | `StartFailure` | エラー | 127 |
| 5 | `TerminatedBySignal = n` | エラー | `128 + n` |
| 6 | `ExitCode = nil`(起動して終わったのに値がない) | エラー(`conductor` 自身のエラー) | 1 |
| 7 | `ExitCode` が成功コード(既定は `[0]`、`success_exit_codes` で指定) | 成功 | 0 |
| 8 | `ExitCode` が `warning_exit_codes` | 警告 | 0 |
| 9 | `ExitCode = 0` で成功コードに含まれない | エラー(`conductor` 自身のエラー) | 1 |
| 10 | 上記以外 | エラー | `ExitCode` のまま |

`conductor` の終了コードは、報告する終了コードを 0〜255 に合わせたもの(範囲内はそのまま。範囲外は下位 8 ビット。エラーで
下位 8 ビットが 0 になるときは 1)。詳細は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md)。

---

## 5. 待ち(待機ループ)

### 5.1 規定

| 項目 | 内容 | 強さ |
|------|------|------|
| 回収 | ジョブを回収する(`cmd.Wait` で終了の状態を読む)のは、待機用の goroutine 1 つだけ。`Wait` を 2 回以上呼ばない | 【規定】 |
| 転送の判断 | 「ジョブがまだ終わっていない」と観測した時点で、**まだ転送していない**かつ合図が立っていれば、シグナルをジョブに **1 回だけ** 転送する(7.2 節) | 【規定】 |
| 終了済みへの送信の禁止 | 終了済み・回収済みのジョブには送らない。ジョブが先に終わっていれば、合図が立っていても転送しない | 【規定】 |
| 転送は 1 回 | 2 回目以降の合図(同じシグナルでも別のシグナルでも)で、再度の転送や強制終了はしない | 【規定】 |
| 転送のあと | ジョブの終了を待ち続ける(ジョブがシグナルを無視すれば、`conductor` はそのまま待ち続ける) | 【規定】 |
| 待つ方式 | `select` で、「`Wait` の結果のチャネル」と「合図のチャネル(`Interrupt.Done()`)」を同時に待つ。確認の間隔(ポーリング)を持たない | 【Go の設計】 |
| CPU の使用 | ジョブの実行中の `conductor` の CPU 時間が、約 3 秒の待ちで 100 ms 以下であること(ポーリングの周期で起こさない) | 【規定】(強制。[11-quality-and-build.md](11-quality-and-build.md)) |
| 転送の遅れ | 合図が立ってから転送までは、`select` が直ちに起きるので、目に見える遅れがない(目安として 10 ms 以下) | 【目安】 |

### 5.2 待機ループの設計

待機ループは、`RunJob` から切り出した、次のシグネチャの関数とする(テストで、偽の「`Wait` の結果」と「転送の関数」を渡すため)。

```go
// done: 待機用の goroutine が、cmd.Wait() の結果を 1 回だけ送るチャネル(容量 1)。
// forward: 子の pid にシグナル n を 1 回送る(OS 別の実装。失敗は呼び出し先で無視する)。
// 戻り値: Wait が返した error。
func waitLoop(done <-chan error, sig *Interrupt, forward func(n int)) error
```

手順:

1. `sigC := sig.Done()` を取る(合図のチャネル。まだ転送していない間だけ有効)。
2. ループで `select` する。
   - `done` から値を受け取った → その `error` を返す(転送の機会は終わり)。
   - `sigC` が閉じた(合図が立った) →
     1. `sigC = nil` にする(`nil` のチャネルは `select` で選ばれないので、2 回目以降の合図で何もしない。転送は 1 回)。
     2. **まず `done` を非ブロッキングで確かめる**(`select` に `default` を付ける)。受け取れたら、ジョブはすでに終わっている → 転送せず、その `error` を返す。
        (`select` は、複数の準備ができているとき、乱数で選ぶ。この確認がないと、ジョブが先に終わったのに転送してしまう。【規定】)
     3. 終わっていなければ、`n, _ := sig.Number()` を取り、`forward(n)` を 1 回呼ぶ。
     4. ループに戻る(ジョブの終了を待ち続ける)。
3. `Wait` の結果を受け取ったあと、呼び出し元(`RunJob`)が `sig.Number()` を読み、4.1 節の手順 7 のとおり `Interrupted` を決める。

待機用の goroutine:

```text
done := make(chan error, 1)         # 容量 1: 受け取る側が先に抜けても、送信で詰まらず、goroutine が残らない
go func() { done <- cmd.Wait() }()
```

`RunJob` は、`waitLoop` が返る(= `done` から受け取る)まで戻らない。したがって待機用の goroutine は必ず終わり、リークしない(9 章)。

### 5.3 転送が「終了済み」に当たる競合

`waitLoop` が「まだ終わっていない」と判断してから `forward` が実際に送るまでの間に、ジョブが終わって回収される可能性は、ゼロにできない。
Go の `os.Process.Signal` は、回収済みのプロセスには `os.ErrProcessDone` を返し、送らない(プロセス番号の再利用を避ける作りになっている。版により内部の実装が違う)。
`forward` は、このエラーを含むすべての送信の失敗を無視する(7.2 節)。pid 番号を `syscall.Kill` に直接渡す実装にしてはならない
(回収後の pid の再利用で、無関係なプロセスにシグナルを送りうる)。`os.Process.Signal` と `Wait` の並行の呼び出しの扱いは、公式の文書で明記を確認できていない(**未確認**。12 章)。
この設計は、`Signal` を `waitLoop`(`RunJob` を呼んだ goroutine)から、`Wait` を別の goroutine から呼ぶ。`go test -race` と、転送のテスト(11 章)で、実装時に確かめる。

---

## 6. 中断の合図(`Interrupt`)

「中断の合図」は、`conductor` がシグナルを受け取ったことを、ジョブを待つ側へ伝える値。

### 6.1 規定

| 項目 | 規定 |
|------|------|
| 内容 | 「立っているか」と、立っているなら「最初のシグナルの番号」 |
| 初期 | 立っていない |
| 立てる(番号 n) | 立っていなければ n を保存する。**すでに立っていれば何もしない**(最初の番号が残る)。番号は任意の整数でよく、負数や非常に大きい値、0 でも、「立った」ことは記録される |
| 性質 | 立ったら下りない。シグナル監視の goroutine と待つ側の goroutine のあいだで、安全に共有できる(`go test -race` で競合が出ない)。待つ側は読むだけ |
| 等価性 | 番号が等しければ等しい |

### 6.2 Go の型の骨格【Go の設計】

```go
// Interrupt は、中断の合図。ゼロ値は使えない。NewInterrupt で作る。
type Interrupt struct {
    mu     sync.Mutex
    raised bool
    num    int
    done   chan struct{} // 最初の Raise で 1 回だけ閉じる
}

func NewInterrupt() *Interrupt
func (i *Interrupt) Raise(n int)                  // 立っていなければ n を保存し、done を閉じる。立っていれば何もしない
func (i *Interrupt) Number() (n int, raised bool) // 読むだけ
func (i *Interrupt) Done() <-chan struct{}        // 立つと閉じる。select で待てる
```

- 保護は `sync.Mutex` とする。「立っているか」と「番号」の 2 つの値を一緒に更新するため、`atomic` 単体では足りない(`atomic.Int64` 1 つに詰める場合は、「立っていない」を表す値が、任意の整数の番号と衝突しないことが必要で、この仕様の「任意の整数」に反する)。
- `Done()` のチャネルは、`Raise` の最初の呼び出しの中で、ロックを持ったまま閉じる(二重に閉じない)。
- `Interrupt` は、コピーしない(`sync.Mutex` を含む)。常に `*Interrupt` で渡す。`go vet` の copylocks が検査する。
- 2 つの値(合図の有無と番号)は、1 回のロックの中で読む(`Number()`)。立っているのに番号が古い、という中間の状態を見せない。

---

## 7. シグナル処理

### 7.1 準備(両 OS 共通)

- `run` と `pipeline` だけが、**定義の検査より前**に、シグナルの監視を始める【規定】。
  `watch`・`reset`・`validate`・`convert` は監視せず(`signal.Notify` を呼ばず)、OS の既定の動作で終わる。
- `cli` の流れが、`NewInterrupt()` で合図を 1 つ作り、`Deps.Signals.Start(sig)` に渡す。
- 監視の登録は **プロセスで 1 回だけ**。2 回目の呼び出しは「すでに登録済み」(`ErrAlreadyRegistered`)として、流れを成功と同様に続ける
  (2 回目に渡した合図には配線されない)。本番の実装は、パッケージ内の変数(`sync.Mutex` で保護した真偽値)で 1 回だけにする。
- `signal.Notify` は、エラーを返さない(失敗を表す戻り値がない)。したがって、本番の実装では、「登録の失敗」は起こらない。
  それでも、登録に失敗したとき(偽の `Signals` が返す)は、標準エラーに
  `conductor: error: failed to set up signal handling` を書き、終了コード **70** で終わる(定義の検査もジョブの起動もしない)。
  この経路は、偽の実装で検査する(11 章)。登録に失敗した場合は、次の登録を許す(登録済みの印を戻す)。
- 「最初の番号」は、合図を立てる操作(`Raise`)が呼ばれた順の最初。

### 7.2 UNIX 系(Linux、macOS)

受け取るシグナルは **SIGHUP(1)、SIGINT(2)、SIGQUIT(3)、SIGTERM(15)** の 4 つ【規定】。受け取るたびに、`Raise` を、そのシグナルの番号で呼ぶ。
登録後、これらのシグナルでは、Go の実行環境の既定の動作(終了。SIGQUIT は、すべての goroutine のスタックを出して終了)をしなくなる。

Go での対応(`platform_unix.go`):

| 項目 | 規定 |
|------|------|
| 受け取る集合 | `[]os.Signal{syscall.SIGHUP, syscall.SIGINT, syscall.SIGQUIT, syscall.SIGTERM}` |
| 番号への変換 | 受け取った `os.Signal` を `syscall.Signal` に型アサーションし、`int` に直す。アサーションに失敗した値は、無視する(起こらない) |
| 登録 | `ch := make(chan os.Signal, 8)`、`signal.Notify(ch, ...)`。チャネルは**容量を持つ**(`signal` パッケージは、送信でブロックせず、詰まったら捨てる。容量 0 だと取りこぼす) |
| 受信の goroutine | 1 つ。`ch` から読み、番号に直して `sig.Raise(n)` を呼ぶ。出力・ステート・子プロセスには触れない |
| 転送 | `cmd.Process.Signal(syscall.Signal(n))`(`syscall.Kill(pid, n)` ではない。5.3 節) |

時系列(ジョブの実行中に、利用者が `kill -TERM <conductor の pid>` したとき):

1. `conductor` が SIGTERM を受け取る。受信の goroutine が `Raise(15)` を呼ぶ。
2. 待機ループの `select` が、合図のチャネルが閉じたのを見て起きる。
3. ジョブの pid に、**同じシグナルを 1 回**送る。送る前に次を検査する(`forward` の中)。
   - pid が 0 より大きい(`cmd.Process.Pid > 0`)。0 や負にならない。プロセスグループへの送信にならない(`syscall.Kill(-pgid, ...)` や、`pid` に 0 を渡す呼び出しを使わない)。
   - シグナルが `{SIGHUP, SIGINT, SIGQUIT, SIGTERM}` のどれか(SIGKILL、SIGSTOP、SIGCONT、SIGUSR1、0、負、9999 は送らない)。
   - 送信先は、**ジョブの子プロセスの pid だけ**。ジョブの子孫(`sh -c` が起こした孫など)には、`conductor` は送らない。
   - 送信の失敗(`os.ErrProcessDone` を含む)は無視する。
4. ジョブは、受け取ったシグナルの既定の動作で終了するか、自分で処理する。`conductor` は待ち続ける。2 回目以降のシグナルでは、何も追加しない。
5. ジョブの終了を観測する(`Wait` が返る)。`ExitCode`(シグナル終了なら `nil`)と `TerminatedBySignal` を読み、直後に合図が立っているので
   `Interrupted = true`、`SignalNumber = 15` を足す。
6. 分類で「エラー、`128 + 15 = 143`」となり、流れの制御はこの結果をステートに**エラーとして**記録する(再開位置はそのジョブ)。
7. `conductor` は **143** で終わる(`pipeline` は次のジョブを起動しない)。

その他:

- ジョブは、`conductor` が起動時に作る **新しいプロセスグループ**(`cmd.SysProcAttr = &syscall.SysProcAttr{Setpgid: true}`。`Pgid` は 0 のまま = グループ ID は子の pid)で動く【規定】。端末の
  Ctrl+C(SIGINT)は、フォアグラウンドのプロセスグループに配られるため、ジョブには直接届かず、`conductor` が受けて 1 回転送する
  (SIGINT のとき、ジョブは `TerminatedBySignal = 2` で終わるのが典型)。
- 転送の許可集合にない番号で合図が立った場合(`conductor` 本体の監視は 4 つだけを受けるため、通常は起こらない。`runner` の部品を単独でテストする場合のみ)は、
  送信せず、ジョブが自分で終わるのを待ち、終わりを観測した後に `Interrupted = true`、`SignalNumber = その番号` を足す
  (ジョブの終了コードは `ExitCode` に残るが、分類では見ない)。
- 時間を空けて複数のシグナルが届いたときは、到着順の最初の番号が残る(INT のあとの HUP でも 2)。
  ほぼ同時に届いたときに、どれが「最初」になるかは**規定しない**(Go の `signal` パッケージが、どの順でチャネルへ送るかは確認していない。12 章)。
- 終了コードの例: SIGHUP は 129、SIGINT は 130、SIGQUIT は 131、SIGTERM は 143。
- `conductor` 自身が、起動時に SIGHUP や SIGINT を無視する設定(`nohup` など)で起動された場合に、`signal.Notify` がどう振る舞うかは、Go の `os/signal` の文書に記述がある(`Notify` を呼ぶと、ハンドラが入る旨)。この仕様は、4 つのシグナルをいつも受け取ることを要求する。実装時に、採用する版の文書で確かめる(未確認。12 章)。

### 7.3 Windows

受け取るのは **Ctrl+C(番号 2)だけ**【規定】。

Go での対応(`platform_windows.go`):

| 項目 | 規定 |
|------|------|
| 受け取る集合 | `[]os.Signal{os.Interrupt}` |
| 番号への変換 | 受け取ったシグナルが何であっても、番号は定数 `2` とする(`os.Interrupt` を受け取るのは Ctrl+C に対応する) |
| 登録 | UNIX 系と同じ(容量のあるチャネルと `signal.Notify`。受信の goroutine は 1 つ) |
| 転送 | `forward` は何もしない(関数は存在するが、本体は空) |
| ジョブの起動 | `cmd.SysProcAttr` を設定しない(新しいプロセスグループ・新しいコンソールを作らない) |

時系列(ジョブの実行中に、利用者が Ctrl+C を押したとき):

1. ジョブは `conductor` と **同じコンソール**で起動されている。Ctrl+C は、OS が、同じコンソールを共有するプロセス(`conductor` とジョブ)の両方に配る。
2. `conductor` の受信の goroutine が `Raise(2)` を呼ぶ。
3. 待機ループは転送を試みるが、Windows の転送は **何もしない**【規定】。
4. ジョブは、OS から受け取った Ctrl+C で(既定の動作で)終了する。
5. 終了を観測する。`TerminatedBySignal` は常に `nil`、`ExitCode` は OS の終了コード(Ctrl+C による終了コード)。直後に合図が立っているので
   `Interrupted = true`、`SignalNumber = 2`。
6. 分類で「エラー、`128 + 2 = 130`」。ステートに**エラーとして**記録し、**130** で終わる。

その他:

- Ctrl+Break とコンソールを閉じたときなどの通知は、この版の対象外とする。Go の実行環境は、Windows のコンソールのイベントを `SIGINT` / `SIGTERM` として配るが(という認識)、Ctrl+Break が `os.Interrupt` として届くかは**未確認**(届いても、番号は 2 として扱う。実装で `syscall.SIGTERM` を追加登録しない)。
- 自動の試験環境(CI の Windows ランナー)には、対話的なコンソールがなく、Ctrl+C を安定して送る方法がないため、この経路は自動テストにしない。
  リリース前に実機で手動確認する(確認項目: 終了コード 130、ジョブの子プロセスが残らない、
  `watch` で中断したジョブが `error` で次が `pending`、`run -v` の表示に `interrupted by signal 2`)。手順は [11-quality-and-build.md](11-quality-and-build.md) にある。

### 7.4 合図が立つ時点ごとの動作(流れの制御)

| 合図が立った時点 | 動作 | ステートの記録 | 終了コード |
|------------------|------|----------------|------------|
| `RunJob` の呼び出しの前(`Started = false`) | ジョブを起動しない | 分類なし。エラーとしては記録しない。再開位置はそのジョブ | `128 + 番号` |
| ジョブの実行中 | 1 回転送(UNIX 系)。ジョブの終了を待つ | そのジョブを**エラー**として記録(ジョブが 0 で終わっても) | `128 + 番号` |
| ジョブの終了の観測の直後(転送が間に合わなかった場合を含む) | 起動済みで `Interrupted = true`(上の行と同じ扱い) | エラー | `128 + 番号` |
| `pipeline` のジョブとジョブのあいだ(ジョブの結果は成功か警告で、完了ではない) | 次のジョブを起動しない | 記述順で、まだ起動していない最初のジョブ(エラーのジョブが先にあればそれ)を再開位置にして、中断を記録。エラーとは記録しない。`-v` で `interrupted by signal N` | `128 + 番号` |
| `pipeline` の最後のジョブが完了した直後 | 完了の処理(`--remove-on-complete` ならステートを削除)をして終わる | 完了 | 0(合図は無視される) |
| リトライの待機中 | 待機を打ち切る。`Started = false`、`Interrupted = true`、`SignalNumber = 番号` の実行結果を作る。次の実行はしない | 分類なし、再開位置はそのジョブ。エラーとしては記録しない | `128 + 番号` |
| 定義の検査・ステートの読み込み・照合の途中 | 検査は最後まで進める | (定義の誤り 64、ステートの誤り 65、拒否 66、スキップ 0 の終了コードが、中断の終了コードより優先する) | 各検査の値 |

### 7.5 リトライの待機

関数: `func WaitFor(secs uint64, sig *Interrupt) WaitOutcome`。`WaitOutcome` は `Interrupted bool` と `Number int` を持つ(`Interrupted = false` なら「経過」)。
OS 固有のコードを持たない。

手順(`time.Timer` と `select`):

1. 合図が立っていれば(`sig.Number()` が `ok`)、すぐ「中断(最初の番号)」を返す。
2. `secs == 0` なら、すぐ「経過」を返す(手順 1 で、立っていなければ)。
3. 待ち時間 `d` を求める。`secs` が `time.Duration` に収まらない大きさ(`math.MaxInt64 / int64(time.Second)` を超える)なら、`d = math.MaxInt64`(約 292 年)に止める。掛け算であふれさせない。
4. `t := time.NewTimer(d)`、`defer t.Stop()`。
5. `select` で `t.C` と `sig.Done()` を待つ。
   - `sig.Done()` が閉じた → 「中断(最初の番号)」。
   - `t.C` が来た → `sig.Number()` を再度確かめ、立っていれば「中断」、立っていなければ「経過」。
6. ポーリングをしないので、合図から返るまでは直ちにである。長い待ちでも、合図から **1 秒以内**に返る【規定】
   (例: 3600 秒の待機に、200 ms 後に合図を立てて、全体が 1.2 秒未満)。

複数の合図は、最初の番号を返す。待機の時計は、テストで差し替える必要がない(実時間の短い待ちで、1 秒以内の打ち切りを検査できる)。ただし `cli` のテストは、`Process.Wait`(`Deps`)の偽の実装で、待ち時間を記録だけして実際には待たない([02-architecture.md](02-architecture.md))。

---

## 8. プラットフォーム差とビルドタグ

### 8.1 差の一覧

| 項目 | UNIX 系(Linux、macOS) | Windows |
|------|------------------------|---------|
| ジョブのプロセスグループ | 新しいグループ(`Setpgid: true`) | 指定しない(同じコンソール) |
| 監視するシグナル | SIGHUP・SIGINT・SIGQUIT・SIGTERM | Ctrl+C のみ(番号 2。`os.Interrupt`) |
| ジョブへの転送 | 子の pid に、`Process.Signal` で 1 回(許可集合のみ) | しない(OS が Ctrl+C を配る) |
| シグナル終了の読み取り | `syscall.WaitStatus` から得る | なし(終了はすべて終了コード) |
| 終了コードの範囲 | 下位 8 ビット | 32 ビット符号つきのまま(`conductor` が報告する値は、0〜255 に合わせる) |
| ステート書き込み後のディレクトリの同期 | ディレクトリを開いて `Sync` する。失敗は無視。親が空なら `.` | しない(`internal/state` の OS 別のファイル。[06-state-file.md](06-state-file.md)) |
| 端末の色 | 使える(標準出力が端末で `NO_COLOR` が未設定か空のとき) | 使わない(色なし) |
| `.cmd` / `.bat` の起動 | (なし) | 8.3 節の安全策 |

### 8.2 ビルドタグとファイルの分け方【規定】

Go では、ファイル名の末尾の `_windows.go`、`_linux.go`、`_darwin.go` は、暗黙のビルド制約になる。しかし、**`_unix.go` は、暗黙の制約にならない**(`unix` は、ファイル名の末尾としては認識されない)。
そのため、次のようにする。

| ファイル | 1 行目付近の制約(明示する) | 内容 |
|----------|-----------------------------|------|
| `platform_unix.go` | `//go:build unix` | 上記の UNIX 系の実装。`unix` の制約は、Go 1.19 以降で使える。UNIX 系の OS 全般(linux、darwin、freebsd など)で成り立つ。対象の OS は linux と darwin だが、制約は `unix` でよい |
| `platform_windows.go` | `//go:build windows` | 上記の Windows の実装(ファイル名で暗黙にも制約される。明示も書く) |

- 2 つのファイルは、**同じ名前・同じシグネチャの関数**を定義する。共通のファイルは、それだけを呼ぶ。

| 関数(小文字。パッケージの内側だけ) | 役割 |
|--------------------------------------|------|
| `configureCmd(cmd *exec.Cmd) error` | OS 別の `exec.Cmd` の設定。UNIX 系は `SysProcAttr{Setpgid: true}`。Windows は `.cmd` / `.bat` の安全策(8.3 節)。拒否するなら `error`(起動失敗として扱う) |
| `readExit(ps *os.ProcessState) (exitCode *int, signal *int)` | 4.3 節の取り出し |
| `forwardSignal(p *os.Process, n int)` | 転送(UNIX 系は許可集合の検査のあと `p.Signal`。Windows は空) |
| `watchedSignals() []os.Signal` | `signal.Notify` に渡す集合 |
| `signalNumber(s os.Signal) (int, bool)` | 受け取ったシグナルを番号に直す |

- `syscall.SIGHUP`、`syscall.Setpgid`、`syscall.WaitStatus` のように、OS によって存在が違う名前は、`platform_unix.go` にだけ書く。共通のファイルが参照すると、Windows でのビルドが失敗する。
- 3 OS(Linux、macOS、Windows)の CI で `go build ./...` と `go vet ./...` を通すことで、制約の漏れを検出する。`GOOS=windows go vet ./...` と `GOOS=darwin go vet ./...` を、Linux の 1 台でも確かめられる(クロスコンパイルは、純粋な Go のコードなら外部の道具を要さない。cgo を使わない)。
- `//go:build unix` でも `windows` でもない OS(`js`、`plan9` など)は、対象外とする。ビルドが失敗してよい(専用のファイルを置かない)。
- 点検(AC-420 に相当): 共通のファイルに `runtime.GOOS` と OS 固有の `syscall` の名前がないことを、ソースの検索(`grep`)で確かめる。[11-quality-and-build.md](11-quality-and-build.md)。
- `internal/state` のディレクトリの同期と、`cmd/conductor` の入口の OS 別の部分(7.1 節のシグナル、未確認の項目の SIGPIPE の扱い)も、同様に `_unix.go` / `_windows.go` に置く。

### 8.3 Windows の `.cmd` / `.bat` への引数【規定】

Windows の `cmd.exe` は、`.cmd` / `.bat` を実行するときに、引数を独自の規則で解釈する(`"`、`%`、`^`、`&`、`|`、`<`、`>` など)。Go の `exec.Command` が組み立てる引数の引用は、`cmd.exe` の解釈と合わない場合がある。
引数の中のコマンドが実行されないこと(AC-113、AC-114)を守るため、この版は、**次のとおり拒否する**(起動を拒否することは、規定の 2 つの選択肢のひとつ)。

1. `exec.Command` のあと、`cmd.Err` が `nil` で、`cmd.Path` の拡張子が(大文字小文字を区別せず)`.cmd` または `.bat` のときだけ、検査する(`strings.EqualFold(filepath.Ext(cmd.Path), ".cmd")` など)。
2. `Args`(最初の `Program` を除く)のどれかが、次の文字を含むなら、起動せず、起動失敗の実行結果を返す(`configureCmd` が `error` を返す)。

```text
"  %  ^  &  |  <  >  !  (  )  改行(\n)  復帰(\r)  NUL
```

3. 含まなければ、そのまま起動する(空白を含む引数は、`exec.Command` の標準の引用に任せる。安全な引数 `plain` で、終了コード 7 が取れる。AC-115)。

- この禁止の文字の集合は、この仕様の設計判断である(Windows の実機で確かめていない)。`cmd.exe` の解釈の詳細と、採用する Go の版の `os/exec` が、`.cmd` / `.bat` に対して独自の安全策(特定の引数でエラーを返す)を持つかは、**未確認**(12 章)。Go 自身がエラーを返す場合も、起動失敗として同じに扱われる。
- 配列形式の `command` も、同じ検査を通る。

---

## 9. goroutine とチャネルの設計、データ競合の避け方

### 9.1 goroutine の一覧(誰が所有し、どう終わるか)【規定】

| goroutine | 起こす者(所有者) | 数 | 読む | 書く | 終わる条件 |
|-----------|-------------------|-----|------|------|------------|
| メイン(`main` から `cli` の流れまで) | 実行環境 | 1 | すべて | 出力、ステート、`Interrupt`(読むだけ) | `main` が `os.Exit` で終える |
| シグナル受信 | `Signals.Start`(本番は、パッケージ内の登録の関数) | プロセスで 0 または 1(`run`・`pipeline` のみ) | シグナルのチャネル | `Interrupt.Raise` だけ | プロセスの終了まで動き続ける(本番で停止の関数は呼ばない)。テスト用の監視は、停止の関数(`signal.Stop` と終了用チャネルの close)で終わらせる |
| `Wait` の待機 | `RunJob` | 実行中のジョブごとに 1(同時に最大 1) | `cmd.Wait` の結果 | 容量 1 のチャネルに 1 回送る | `cmd.Wait` が返り、送信し終えたとき。`RunJob` は、受け取るまで戻らない |

- それ以外の goroutine を起こさない。リトライの待機は、メインの goroutine が `select` で待つ(goroutine を起こさない)。
- **出力(標準出力・標準エラー)とステートへの書き込みは、メインの goroutine だけが行う**【規定】。受信の goroutine と `Wait` の goroutine は、それらに触れない。
- チャネルは、すべて所有者が作り、送る側が 1 か所に決まっている。受信側が先に抜けても詰まらない形にする(容量 1、または、閉じて知らせる)。
  `Interrupt.Done()` は閉じて知らせるだけで、値を送らない。
- `context.Context` は使わない。中断は `Interrupt` で伝える(`CommandContext` を使わない理由は 4.2 節)。
- goroutine のリークがないこと: テストの終わりに、起こした goroutine が残っていない(`RunJob` が戻った時点で、`Wait` の goroutine は終わっている)。検査は、実装者が選ぶ(`runtime.NumGoroutine` の前後比較、または `goleak` のような外部の道具。後者を使う場合は、外部ライブラリの選定を 11 章に従って記録する)。

### 9.2 データ競合の避け方(`go test -race` で通ること)【規定】

| 規則 | 内容 |
|------|------|
| 共有する値 | goroutine の間で共有する可変の値は、`Interrupt`(`sync.Mutex` で保護)と、シグナルの登録の印(パッケージ内の変数。`sync.Mutex` で保護)の 2 つだけ |
| 渡し方 | 実行結果(`flow.ExecResult`)は、値で返す(チャネルや戻り値で渡し、渡したあとは送る側が触れない)。ポインタの先を、2 つの goroutine が同時に読み書きしない |
| パッケージの変数 | 可変のパッケージの変数を、上記の登録の印以外に持たない(`init` で 1 回だけ作って読み取り専用にする表は可。`map` に並行の書き込みをしない) |
| 偽の実装 | `internal/testsupport` の偽の実装は、メインの goroutine からだけ呼ばれる設計。受信の goroutine を使うテストで、偽の実装に触れるときは、`sync.Mutex` で保護する |
| ループ変数 | `go` 文の中で、ループ変数や、あとで書き換える変数を直接参照しない(引数で渡す)。Go 1.22 以降の言語の版では、ループ変数は反復ごとの変数になるが、それに頼らない |
| テストの並列 | `t.Setenv` は、並列のテスト(`t.Parallel`)と同時に使えない(実行時に失敗する)。環境変数は、`Env` の偽の実装で与える。カレントディレクトリを変える(`os.Chdir`)テストは、並列にしない |
| `-race` の前提 | `-race` は、cgo が使える環境(C のコンパイラ)を要する場合が多い。CI の環境で有効にする(11 章) |
| 検査 | `go test -race ./...` が、`internal/runner` の次のテストで通ること: 合図を別の goroutine から立てる、複数の goroutine から `Raise` を同時に呼ぶ(最初の番号だけが残る)、転送とジョブの終了の競合(短いジョブへの繰り返し) |

---

## 10. 想定外の内部の誤り

- `cmd/conductor/main.go` の入口に、`defer` と `recover` で想定外の内部の誤り(実装の不具合による `panic`)を捕まえる仕組みを、**最初に**入れる。
  内部の誤りが起きると、標準エラーに固定の英語の 1 行 `conductor: error: internal error`(ロケールを判定しない)を書き、終了コード **70** で終わる。
  標準エラーへの書き込みが失敗しても 70 で終わる。【規定】
- 入口の形(骨格): `func main() { os.Exit(realMain()) }`。`realMain() (code int)` の中で、`defer func() { if r := recover(); r != nil { /* 1 行を書く(失敗は無視) */ code = 70 } }()` を最初に書く。
  `os.Exit` は `defer` を実行しないので、`defer` は `realMain` の中に置き、`os.Exit` は `main` で、戻り値で 1 回だけ呼ぶ(終了は 1 か所)。
- `recover` が捕まえられるのは、同じ goroutine の `panic` だけである。そのため、**メイン以外の goroutine は、`panic` を起こさない書き方にする**(9.1 節の goroutine は、`Interrupt.Raise` と `Wait` と送信だけで、失敗しうる処理を持たない)。
  メイン以外の goroutine で `panic` が起きると、`recover` を通らず、Go の実行環境の既定の動作(終了コード 2、スタックの出力)になる。これは避けるべき失敗であり、起こさない設計とする。
- 入口に、わざと `panic` を起こすコマンドを持たない。内部の誤りの検査は、`realMain` に差し替えた(`panic` する)`Deps` を渡す単体テスト、または別のテスト用の実行ファイルで行う([10-errors-and-exit-codes.md](10-errors-and-exit-codes.md))。
- ジョブの実行中に内部の誤りが起きると、子プロセスの終了を待たずに終わり、ジョブが孤児として残りうる。ステートは直前の完全な内容のままで、
  次の実行はそのジョブを再実行しうる(少なくとも 1 回の実行の範囲)。
- `conductor` が SIGKILL などで強制終了されたときも、ジョブは残って動き続けうる(検知も停止もしない)。
- 外部の入力(引数・環境・ファイル・OS の結果)に対して、`panic` する書き方(`nil` の参照外し、範囲外の添字、型アサーションの `ok` なしの使用、`must` 系の関数)を製品のコードに書かない(失敗は `error` か値で返す)。

---

## 11. 検証の観点

| 確かめること | 検査の区分と置き場所の案 |
|--------------|--------------------------|
| 分割の全規則(3 章の表。受理・拒否) | `internal/runner` の表駆動の単体テスト(`Split`) |
| ランダムな `command`(3 章の文字集合)で、成功なら `Program` が空でない。約 30 万引数が分割できる。閉じない引用符は誤り | 同上(固定の種のランダム。`testing.F` の fuzz に置く場合は、シード・コーパスを固定) |
| メタ文字が引数にそのまま渡る、配列の要素がそのまま渡る、環境変数と作業ディレクトリの引き継ぎ、起動失敗、終了コード(0・3・255・300) | `cmd/conductor-testjob` を起動する `runner` のテスト(実プロセス) |
| 標準入力・標準出力・標準エラーの引き継ぎ(`nil` ではなく `os.Std*` を渡していること) | 同上(ジョブが標準入力を読み、標準出力へ書く) |
| 起動前の合図で起動しない、範囲外の番号(負・0・9999)でも起動せず中断として扱う、終了の観測の直後の合図で `Interrupted` | `RunJob` の単体テスト(`Raise` してから呼ぶ) |
| `waitLoop`: 転送が 1 回だけ、先に終わったジョブへ転送しない、2 回目以降の合図で追加の転送をしない | `waitLoop` の単体テスト(偽の `done` と `forward`。`done` と合図を同時に準備して、`select` の乱数の選択に依らないこと。繰り返して確かめる) |
| SIGTERM / SIGINT の転送、許可外の番号は送らない、新しいプロセスグループ、先に終わったジョブは転送なし(UNIX 系) | 実プロセスのテスト(UNIX 系) |
| 実バイナリで SIGTERM → 143、ステートに再開位置、孤児なし。HUP のあとの INT → 129 | 受け入れテスト(UNIX 系) |
| `WaitFor` の 1 秒以内の打ち切り(3600 秒の待機に、200 ms 後の合図で、全体が 1.2 秒未満) | `WaitFor` の単体テスト |
| 監視の登録(最初の番号が残る、2 回目の登録は `ErrAlreadyRegistered`、登録失敗で 70) | 単体テスト(登録失敗は、偽の `Signals`) |
| ジョブの実行中(約 3 秒)の CPU 時間が 100 ms 以下(UNIX 系) | 計測(強制。ポーリングを持たないので満たす想定。実測する) |
| Windows の Ctrl+C | 手動(11 章) |
| Windows の `.cmd` への危険な引数が注入にならない(AC-113〜115) | Windows の CI のテスト |
| `go test -race ./...` が通る | CI |

テストでスクリプトを書いてすぐに実行する場合、Linux で、実行ファイルを書いた直後の起動が「text file busy」(`ETXTBSY`)で失敗することが、Go の既知の事象として知られている(別の goroutine の `fork` が、書き込み用のファイル記述子を一時的に持つため)。
テストのスクリプトは、あらかじめ作っておく(`cmd/conductor-testjob` をビルドして使う)か、起動前にファイルを閉じて待つ。実際の発生と、回避策の有効性は**未確認**(12 章)。

---

## 12. 規定と実装の裁量の区別、未確認の事項

### 12.1 規定と裁量(まとめ)

| 項目 | 区別 |
|------|------|
| 分割の状態機械と、受理・拒否の例 | 【規定】 |
| シェルを介さない起動、作業ディレクトリ・環境変数・標準入出力の引き継ぎ(`os.Std*` を明示) | 【規定】 |
| 起動失敗の終了コード 127、分類の優先順位 | 【規定】 |
| 転送は 1 回だけ、許可する 4 つのシグナルだけ、子の pid だけ(`Process.Signal`)、終了済みには送らない | 【規定】 |
| UNIX 系でジョブを新しいプロセスグループで動かす(`Setpgid`) | 【規定】 |
| Windows で転送をしない(OS が Ctrl+C を配る) | 【規定】 |
| `run`/`pipeline` だけが、定義の検査より前にシグナルを準備する | 【規定】 |
| シグナルの準備の失敗は 70、内部の誤りは 70 | 【規定】 |
| リトライの待機が合図から 1 秒以内に返る | 【規定】 |
| 出力とステートへの書き込みは、メインの goroutine だけ。`-race` で通る | 【規定】 |
| `select` による待機(ポーリングなし)、`Interrupt` の型の構造、goroutine の数 | 【Go の設計】 |
| `.cmd` / `.bat` の引数の禁止文字の集合 | 【Go の設計】(Windows の実機で未確認) |
| ほぼ同時に届いた複数のシグナルのどれが「最初」か | 規定しない |

### 12.2 未確認の事項

- Windows での実機の動作(Ctrl+C の終了コード 130、ジョブの終了の様子、`.cmd` の起動)は、ここでは確認していない。
- Windows で、Go が `os.Interrupt` として Ctrl+Break を配るか(対象外としている。確認していない)。
- `os.Process.Signal` と `cmd.Wait` を、別々の goroutine から同時に呼ぶことの保証(公式の文書の明記を確認できていない)。実装時に、採用する版の `os` のソースとドキュメントで確かめ、`go test -race` で検査する。
- Windows で `ProcessState.ExitCode()` が、32 ビットの符号なしの値を `int` に直した値を返すか、符号つきの値を返すか。この仕様は `int32` への変換を要求することで、どちらでも正しくなるようにしている。
- 採用する Go の版の `os/exec` が、Windows で `.cmd` / `.bat` の危険な引数に対して、独自にエラーを返すか。
- Windows でのプログラム探索(カレントディレクトリを含むか等)と `exec.ErrDot` の扱いの細部。標準の規則に従い、起動失敗として扱う、とだけ規定している。
- UNIX 系で、ジョブが新しいプロセスグループにあるため、ジョブが端末から読む(標準入力が端末のとき)場合に、背景のグループとして停止するかは、確認していない。受け入れテストは、標準入力を空にして実行する。
- `signal.Notify` に SIGHUP/SIGINT を渡したとき、起動時にそれらが無視されていた場合の挙動(Go の `os/signal` の文書に記述がある。この仕様では 4 つのシグナルを必ず受けることを要求する。実装時に確かめる)。
- 標準出力・標準エラーの書き込み先が壊れたパイプのとき、Go の実行環境は、通常は `SIGPIPE` でプロセスを終了させる(`os/signal` の文書の記述。文書は、`SIGPIPE` について `Notify` を呼んだ場合の例外を述べる)。この仕様は、書き込みの失敗を値で扱うこと(`watch`・`convert` で 65・67。[08-output-and-i18n.md](08-output-and-i18n.md))を求めるので、入口で `SIGPIPE` を `signal.Notify` で受けて捨てる(`signal.Ignore` は、無視の設定が子に引き継がれる可能性があるため使わない、という判断)ことを検討する。この引き継ぎの有無と、この方式の有効性は**未確認**。実装時に確かめる。
- ほぼ同時に届いた複数のシグナルの順序(Go の `signal` パッケージの配送の順序)。
- Linux での `ETXTBSY`(11 章の注意)の発生の条件と回避策。
- Go の API の挙動の記述は、公式のドキュメントの知識によるもので、この文書の作成時に、コードを実行して確かめていない。
