# 07 プロセス起動とシグナル(`command` の分割・起動・待ち・中断)

対象: `conductor` v0.8.0 と同じ挙動を C#(.NET)で実装するための仕様。この文書は、次を定める。

- `command` の文字列を、コマンド名と引数に分ける規則(3 章)
- ジョブ(子プロセス)の起動、待ち、結果の取得、起動失敗(4・5 章。`System.Diagnostics.Process`)
- 中断の合図と、OS ごと(UNIX 系と Windows)のシグナル処理(6・7 章。`PosixSignalRegistration`、`Console.CancelKeyPress`、`libc` の `kill`)
- OS ごとの差(8 章)、スレッドとコールバックの規則、出力とステートの順序(9 章)、想定外の内部の誤り(10 章)

ステートファイルの書式は [06-state-file.md](06-state-file.md)、定義ファイルの検査は [03-definition-format.md](03-definition-format.md)、流れ全体は [05-execution-flow.md](05-execution-flow.md)、表示の文言は [08-output-and-i18n.md](08-output-and-i18n.md)、終了コードの全件は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md)、構成は [02-architecture.md](02-architecture.md) にある。この文書は、必要な箇所だけ触れる。この文書の挙動を確かめる受け入れテストは [11-quality-and-build.md](11-quality-and-build.md) にある(11 章)。

## 規定の強さと確認の表記

| 表記 | 意味 |
|------|------|
| 【規定】 | どの実装も満たす。受け入れテストまたは単体テストで確かめる |
| 【目安】 | 推奨の値。利用者から見える結果を変えない範囲で、変えてよい |
| 【任意】 | 実装に任せる。利用者から見える結果に影響しない |
| 【.NET の設計】 | .NET で実装するために、この仕様が決めたこと(他の決め方もありうる) |
| 確認(出典: …) | 公式ドキュメント(learn.microsoft.com)または .NET ランタイムの公開ソース(github.com/dotnet/runtime の `main` ブランチ)を、この文書の作成時に読んで確かめた。出典の種類を括弧に書く。ソースだけで確かめたものは、公式の API 文書には書かれていない挙動であり、版によって変わりうる |
| 未確認 | 公式ドキュメントでも公開ソースでも確かめられていない。この環境には .NET SDK がなく、実機で動かして確かめていない。実装時に 3 OS(Linux、macOS、Windows)で確かめること |

この文書の作成時点(2026-11)で、サポート中の最新の LTS は .NET 10 である(確認。出典: .NET サポートポリシーのページ。.NET 8 と .NET 9 は 2026-11-10 にサポートが終了している)。この文書の API 名は、.NET 10 を前提にする。

---

## 1. 構成要素と責務

名前空間は `Conductor.Runner`(プロジェクト `Conductor.Runtime`。[02-architecture.md](02-architecture.md))。次の責務が、別々の型として(単独で検査できる形で)ある。

| 構成要素 | 型(【.NET の設計】) | 責務 |
|----------|---------------------|------|
| 分割器 | `CommandSplitter`(静的) | `command` の文字列を、コマンド名と引数に分ける。文字を 1 つずつ左から読む状態機械で、再帰を使わない。3 OS で同じ規則 |
| `command` の検査 | `CommandSplitter.Check(Command)` | `command` を起動せずに検査する。成功なら分けた結果(`CommandLine`)を返す |
| ジョブの起動と待ち | `JobRunner.RunJob(...)` | 1 つのジョブを、シェルを介さずに起動し、終了を待ち、「実行結果」(`JobExecution`。4.5 節)を返す。失敗は値で返し、例外を送出しない |
| 待機ループ | `JobRunner` の内側 | `Process.WaitForExit(int)` の繰り返しで、ジョブの終了を確かめ続ける。シグナルの転送の判断もここで行う(5 章) |
| リトライの待機 | `RetryWaiter.Wait(...)` | リトライの前に待つ。待機中に中断の合図を確かめる(7.5 節) |
| 中断の合図 | `Interrupt` | 「中断が要求された」ことと、最初に受け取ったシグナルの番号を、スレッドの間で共有する値(6 章) |
| プラットフォーム層 | `IPlatform` と実装 `UnixPlatform`、`WindowsPlatform`、検出 `PlatformDetector` | 起動前の準備(`ProcessStartInfo` の調整、`.cmd`/`.bat` の起動の形)、シグナルの送信(`libc` の `kill`)、終了の取り出しの OS 差。OS ごとに実装が違う |
| シグナル監視 | `SignalWatcher` | OS のシグナル(UNIX 系)/ Ctrl+C(Windows)を受け取り、中断の合図を立てる(7 章) |
| 流れの制御 | `Conductor.Cli`(`ExecJob` ほか) | 合図の準備、ジョブの起動の呼び出し、中断時のステートの記録、終了コード(128 + 番号) |
| 分類 | `Conductor.Flow.Classifier`(純粋) | 実行結果を、成功・警告・エラー・分類なしと、報告する終了コードに変える(4.6 節) |

OS の分岐と OS 固有の呼び出し(`OperatingSystem.IsWindows()` などの OS の判定、`LibraryImport`、`kill`)は、プラットフォーム層(`Platform` のファイル群と、`SignalWatcher` の OS 別の登録部分)の外に置かない(8.2 節)【規定】。

---

## 2. `command` の形式(定義の側の受理条件)

定義ファイルの各ジョブの `command` は、次の 2 つの形式のどちらか(JSON と YAML のどちらでも、論理モデルは同じ)。

| 形式 | 書き方(JSON の例) | 型(【.NET の設計】) | 起動のときの扱い |
|------|---------------------|-----------------------|------------------|
| 文字列形式 | `"command": "sh scripts/x.sh"` | `LineCommand(string Line)` | 3 章の規則で、コマンド名と引数に分ける |
| 配列形式 | `"command": ["sh", "-c", "echo hi"]` | `ArgvCommand(IReadOnlyList<string> Argv)` | 要素をそのまま使う(分割・引用符の処理・展開をしない) |

定義の読み込み時の検査(誤りは定義の誤り、終了コード 64。検査の実装は [03-definition-format.md](03-definition-format.md)):

| 入力 | 判定 |
|------|------|
| `command` キーがない | 誤り(必須の値がない) |
| 文字列で、前後の空白(Unicode の空白)を除くと空 | 誤り(`command` が空) |
| 配列が空(`[]`) | 誤り(配列が空) |
| 配列に文字列でない要素がある | 誤り(要素は文字列で書く)。要素ごとに報告する |
| 配列の先頭の要素が、前後の空白を除くと空 | 誤り(先頭の要素が空) |
| 配列の 2 番目以降が空文字列 `""` | 有効(空の引数として渡す) |
| 文字列でも配列でもない(数値・オブジェクトなど) | 誤り(型) |

「前後の空白を除く」は、`string.Trim()` に頼らず、Unicode の White_Space 特性に従う規則を 03 章に従って実装する(`string.Trim()` が除く文字の集合は、Unicode の White_Space と一致するとは限らない。未確認)。この文書の 3 章の区切り(4 文字だけ)とは別の規則である。

定義の検査とは別に、実行の前にもう一度、全ジョブの `command` を 3.8 節のとおり検査する。

---

## 3. `command` の分割規則

### 3.1 入力と出力

- 入力: `command` の文字列(.NET の `string`。UTF-16)。
- 出力: コマンド名(`Program`。空でない)と、引数の並び(`Args`)。`Args` の各要素が、そのまま 1 つの引数になる。
- シェルは使わない。変数展開(`$HOME`)、グロブ(`*`)、パイプ(`|`)、`;`、`&&`、リダイレクト(`>`)、バッククォート、`~`、括弧は **解釈しない**。普通の文字として引数に入る。パイプなどが要るときは、利用者が `sh -c '...'` を `command` に書く。

### 3.2 文法

文字は、左から右へ 1 回だけ読む。区切り文字は次の 4 つだけ。

```text
separator := ' ' (U+0020) | '\t' (U+0009) | '\n' (U+000A) | '\r' (U+000D)
```

(U+000B、U+000C、U+00A0、U+3000 などのほかの空白は区切りではなく、普通の文字。したがって `char.IsWhiteSpace`、`string.Split()`(引数なし)、`string.Trim()`、正規表現の `\s` を使ってはならない。これらは、上の 4 文字以外も空白として扱う。)

状態は 4 つ。これに加えて、`inWord`(語の途中か)と `current`(組み立て中の語。`StringBuilder`)を持つ。

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
3. 語の列の先頭がなければ、または先頭の語が空文字列なら、**コマンド名がない**(誤りの種類 `Empty`)。それ以外は、先頭の語が `Program`、残りが `Args`。

要点:

- 引用符で囲んだ部分と囲まない部分は、隣り合っていればつなげて 1 つの語になる(`a"b c"'d e'f` は 1 語 `ab cd ef`)。
- `''` と `""` は、空の語(空の引数)になる(`inWord` が立つため)。ただし先頭の語が空なら `Empty`。
- 二重引用符の中で解釈するバックスラッシュは、「直後が `"`」のときだけ(`\"` → `"`)。`\n`、`\t`、`\\` なども、**解釈せずそのまま**(`\n` は 2 文字のまま)。ただし `\\` は、1 つ目のバックスラッシュを出力し、2 つ目を新しい保留にする。したがって `\\"` は「`\` と、エスケープされた `"`」になる(引用符は閉じない)。`\` の個数の偶奇で数える規則ではなく、直前に保留している `\` が 1 つあるかどうかだけで決まるので、上の表を左から機械的に適用すること(`"` の直前に `\` が 1 つ以上並んでいれば、`"` は常にエスケープされ、引用符は閉じない)。
- 引用符の外のバックスラッシュは普通の文字(Windows のパス `C:\dir\x.exe` がそのまま書ける)。一重引用符の中も同じ。
- 末尾がバックスラッシュのパスを二重引用符で囲むと `\"` が引用符のエスケープになり、閉じない(誤り)。一重引用符で書く。
- 文字列の長さに上限はなく、入れ子もないため、再帰やスタックの深さに依存しない(約 30 万引数の入力でも分割できる)。語の列は `List<string>` に溜める(固定長の配列やスタック上の領域に溜めない)【規定】。

### 3.3 UTF-16 のコードユニットと `Rune`【.NET の設計】

.NET の `string` は UTF-16 の列である。BMP 外の文字(例: U+1F600)は、2 つの `char`(上位サロゲート U+D800〜U+DBFF と、下位サロゲート U+DC00〜U+DFFF の組)で表される。`System.Text.Rune` は 1 つのコードポイントを表す型で、`string.EnumerateRunes()` は、サロゲートの組を 1 つの `Rune` として返す(確認。出典: `Rune` と `string.EnumerateRunes` の API 文書の存在。不正なサロゲートの扱いの詳細は未確認)。

この文法の特別な文字(区切り 4 文字、`'`、`"`、`\`)は、すべて ASCII であり、1 つの `char` で表される。サロゲートの `char`(U+D800〜U+DFFF)は、これらのどれとも等しくない。したがって、次の 2 つは、同じ結果を返す。

| 走査の方法 | 内容 |
|------------|------|
| `char` ごと(コードユニットごと) | 上の表を、`string` の `char` に順に当てはめる。サロゲートの組は、「上記以外」として、2 つの `char` が順に `current` に足され、元の並びのまま残る |
| `Rune` ごと(コードポイントごと) | 上の表を、`Rune` に順に当てはめる。サロゲートの組は、1 つの `Rune` として「上記以外」になり、`current` に足される |

**この仕様の採用: `char` ごとに走査する**【規定: 結果が同じであれば、どちらでもよい】。理由は次のとおり。(1) 結果が同じ。(2) 不正なサロゲート(対のない `char`)を含む入力で、`EnumerateRunes` は U+FFFD に置き換えるため、元の内容が変わりうる。`char` ごとの走査は、そのまま残す(定義の読み込みが、不正なサロゲートを含む入力を、すでに拒否している場合は、到達しない。[03-definition-format.md](03-definition-format.md))。(3) `Rune` の生成のコストがない。

書記素(見た目の 1 文字。結合文字や絵文字の連なり)は、分割に関係しない。`current` に足すとき、サロゲートの組や結合文字の並びを、途中で切らない(`char` を 1 つずつ足しても、並びは変わらない)。表示の幅や文字数の数え方(エスケープと切り詰め)は、[08-output-and-i18n.md](08-output-and-i18n.md) の規定による(この章では使わない)。

### 3.4 型と関数の骨格(実装コードではない)

```csharp
namespace Conductor.Runner;

public sealed record CommandLine(string Program, IReadOnlyList<string> Args);

public enum CommandErrorKind { UnclosedQuote, Empty }

// 誤りは値。例外ではない。Message は固定の文(入力の値を含めない)。
public sealed record CommandError(CommandErrorKind Kind, string Message);

public abstract record CommandResult;
public sealed record CommandOk(CommandLine Line) : CommandResult;
public sealed record CommandErr(CommandError Error) : CommandResult;

public static class CommandSplitter
{
    // 文字列形式の分割(3.2 節)。例外を送出しない。入力が null のときは起こらない(Nullable で除く)。
    public static CommandResult Split(string line);

    // Command(文字列形式または配列形式)の検査。起動しない。
    public static CommandResult Check(Conductor.Definition.Command command);
}
```

擬似コード(同じ結果になる実装であれば、書き方は自由):

```text
words = new List<string>(); current = new StringBuilder(); inWord = false; state = Plain
foreach (char c in line)
  switch state:
    Plain:  if c is ' ' or '\t' or '\n' or '\r': if inWord: words.Add(current.ToString()); current.Clear(); inWord = false
            elif c == '\'': inWord = true; state = Single
            elif c == '"':  inWord = true; state = Double
            else: inWord = true; current.Append(c)
    Single: if c == '\'': state = Plain else current.Append(c)
    Double: if c == '"': state = Plain
            elif c == '\\': state = DoubleBackslash
            else: current.Append(c)
    DoubleBackslash:
            if c == '"': current.Append('"'); state = Double
            elif c == '\\': current.Append('\\')            // state は DoubleBackslash のまま
            else: current.Append('\\').Append(c); state = Double
if state != Plain: return Err(UnclosedQuote)
if inWord: words.Add(current.ToString())
if words.Count == 0 or words[0].Length == 0: return Err(Empty)
return Ok(words[0], words[1..])
```

### 3.5 受理の例

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
| `echo 😀 'x y'`(U+1F600 はサロゲートの組。3.3 節) | `echo` | `😀`, `x y` |
| `echo a　b`(U+3000 は区切りでない) | `echo` | `a　b`(1 語) |
| `a 'b c'` | `a` | `b c` |

### 3.6 拒否の例

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

### 3.7 配列形式の扱い

- 要素を一切加工せずに使う。先頭が `Program`、残りが `Args`。分割、引用符の解釈、展開、前後の空白の除去をしない(`"  lead and trail  "` も、そのまま 1 つの引数)。
- 先頭の要素が、前後の空白を除いて空(空文字列・空白だけ)、または要素が 0 個のときは `Empty`(定義の検査の二重の備え)。先頭の要素が空白を含むが空でないとき(`"my prog"`)は、そのまま `Program`(`my prog` というファイル名を起動する)。
- 2 番目以降は空文字列でもよい(空の引数)。

### 3.8 誤りの表示と検査のタイミング

| 項目 | 内容 |
|------|------|
| 誤りの内容 | 種類(`UnclosedQuote` か `Empty`)と、固定の文 |
| 固定の文 | `UnclosedQuote` は `引用符が閉じていません`、`Empty` は `コマンド名がありません`。**入力の値を含めない**(秘密の値が `command` に入っていても表示に出ない)【規定】 |
| 利用者への表示(標準エラー) | `conductor: error: <定義ファイルのパス>: job '<ジョブID>': <文>`。文は表示言語に従う。英語は `command has an unclosed quote` / `command is empty`、日本語は `command の引用符が閉じていません` / `command が空です` |
| 検査の時点 | `run` / `pipeline` / `validate` は、定義の読み込みと検証のあと、**全ジョブ**の `command` を検査し、誤りのあるジョブをすべて報告して(1 ジョブ 1 行)、終了コード 64 で終わる。ジョブは 1 つも起動せず、ステートにも触れない |
| 順序 | シグナルの準備(`run`/`pipeline` のみ)→ 定義の読み込み(64)→ `command` の検査(64)→ ジョブID(68)→ run ID(68)→ ステートの読み込み(65)→ 照合(65) |
| 防御 | ジョブの起動の内側でも分割し直す。万一失敗したら、起動失敗の実行結果を返す(通常は到達しない) |

---

## 4. プロセスの起動(`System.Diagnostics.Process`)

### 4.1 手順(時系列)

`JobRunner.RunJob(Job job, Interrupt sig, ...)`(ジョブ → 実行結果)。失敗はすべて戻り値で返し、例外を送出しない。

1. 実行結果を初期化する: `Started = true`、`ExitCode = null`、`TerminatedBySignal = null`、`StartFailure = false`、`Interrupted = false`、`SignalNumber = null`。
2. `command` を分割する(文字列は 3 章、配列は 3.7 節)。失敗なら `StartFailure = true` で返す。
3. **起動の前**に合図が立っていれば、起動せずに返す: `Started = false`、`Interrupted = true`、`SignalNumber = 番号`。(ジョブは実行されない。)
4. `ProcessStartInfo` を組み立てる(4.2 節)。プラットフォーム層の準備を適用する(UNIX 系: 実行ファイルの絶対パスへの解決。Windows: `.cmd` / `.bat` の起動の形。4.7・4.8 節)。解決に失敗したら `StartFailure = true` で返す(4.3 節)。
5. 自分の出力を書き切ってから(flush 済みであること。9.2 節)、`Process.Start(startInfo)` で起動する。例外(4.3 節の表)なら `StartFailure = true` で返す(`Started` は `true` のまま)。戻り値が `null` のときも、`StartFailure = true` で返す。
6. 5 章の待機ループで終了を待つ。時間切れ(タイムアウト)はない。自動のリトライもしない(リトライは流れの制御の層で行う。7.5 節)。
7. 終了を観測したら(4.4 節):
   - `ExitCode` と `TerminatedBySignal` を、4.4 節の規則で決める。
   - **この時点で** 合図が立っていれば `Interrupted = true`、`SignalNumber = 番号`。
8. 終了の状態を取得できないとき(`ExitCode` の取得が例外)は、`StartFailure = true` で返す(分類で 127)。
9. `Process` を `Dispose` する(`using`)。起動後にハンドルを残さない。

### 4.2 起動の条件(`ProcessStartInfo`)

| 項目 | 規定 |
|------|------|
| シェル | 使わない。`UseShellExecute = false` を必ず指定する【規定】。`sh -c` や `cmd /C` を挟まない(ただし、Windows の `.cmd` / `.bat` の起動だけは、4.8 節の決定により `cmd.exe` を直接起動する)。「シェル」は、`UseShellExecute` の文脈では、グラフィカルなシェル(関連づけによるファイルの起動)を指し、`bash` や `sh` のようなコマンドのシェルではない(確認。出典: `ProcessStartInfo.UseShellExecute` の API 文書。既定は .NET では `false`。.NET Framework では `true`)。既定に頼らず、明示する |
| 実行ファイル | `FileName`。UNIX 系では、4.7 節の決定により、絶対パスに解決して渡す。Windows では、`Program` をそのまま渡す(OS の標準の探索に任せる。4.8 節) |
| 引数 | **`ArgumentList` だけ**を使い、`Arguments` を使わない【規定】。`ArgumentList` と `Arguments` は互いに独立で、一度に使えるのは片方だけである。`ArgumentList` は、追加した文字列を、前もってエスケープしなくてよい形で受け取り、OS に渡す 1 本の文字列を内部で組み立てる(確認。出典: `ProcessStartInfo.ArgumentList` の API 文書)。UNIX 系では、`ArgumentList` の各要素が、そのまま `argv` の要素になる。`Arguments` は、空白と引用符で分割し直される(確認。出典: `Process.Unix.cs` の `ParseArgumentsIntoList` と、`Arguments` が空でなければ `Arguments` を使い、そうでなければ `ArgumentList` を使う分岐。公式の API 文書ではなく、ソース)。メタ文字は注入にならず、空文字列の引数も保たれる(UNIX 系。Windows で空文字列の引数が `""` として渡されることは未確認)。例外: Windows の `.cmd` / `.bat`(4.8 節)だけは、`Arguments` を使う |
| 作業ディレクトリ | 引き継ぐ。`WorkingDirectory` を設定しない(既定の空文字列のまま)【規定】。`UseShellExecute = false` のとき、`WorkingDirectory` は、実行ファイルの探索には使われず、起動した側の作業ディレクトリになる(確認。出典: `ProcessStartInfo.UseShellExecute` の API 文書)。空のとき、親の作業ディレクトリを引き継ぐ(未確認。公式の文書に明記がない。受け入れテストで確かめる) |
| 環境変数 | すべて引き継ぐ。`ProcessStartInfo.Environment` に、追加・削除・上書きをしない(`Environment` は、アクセスしたときに現在のプロセスの環境変数で初期化される。未確認)。run ID も環境変数としては渡さない【規定】 |
| 標準入力・標準出力・標準エラー | すべて引き継ぐ。`RedirectStandardInput`、`RedirectStandardOutput`、`RedirectStandardError` を、すべて `false` にする(明示する)【規定】。`UseShellExecute = false` で、これらが `false` のとき、新しいプロセスは、呼び出し側の標準入出力を引き継ぐ(確認。出典: `ProcessStartInfo` の API 文書の Remarks)。`StandardInputHandle`、`StandardOutputHandle`、`StandardErrorHandle`、`StartDetached`(.NET 11 だけ)を使わない。ジョブの出力は、`conductor` を介さず、そのまま流れる。`conductor` は、ジョブの出力を読まない・保存しない・ステートに書かない【規定】 |
| 窓 | `CreateNoWindow` を設定しない(既定のまま)。`WindowStyle` を設定しない |
| 資格情報 | `UserName`、`Password`、`Domain`、`LoadUserProfile` を設定しない |
| 起動できなかった場合 | 4.3 節 |
| 時間の上限 | ない。ジョブが終わらなければ、`conductor` は待ち続ける |
| ジョブの強制終了 | しない(`Process.Kill` を呼ばない。強制終了のシグナルを送る機能はない) |
| 新しいプロセスグループ | UNIX 系: 作らない(4.7 節の決定)。Windows: 作らない(`CreateNewProcessGroup` を使わない) |

### 4.3 起動の失敗の検出

`Process.Start(ProcessStartInfo)` が送出する例外のうち、公式の文書に書かれているもの(確認。出典: `Process.Start(ProcessStartInfo)` の API 文書):

| 例外 | 文書上の原因 |
|------|--------------|
| `InvalidOperationException` | `FileName` が指定されていない。または、`UseShellExecute = true` で標準入出力を横取りしている(この仕様では起こらない) |
| `Win32Exception` | ファイルを開くときの誤り、または、`FileName` のファイルが見つからない。引数と実行ファイルのパスを合わせた長さが上限を超えた(文書は、2080 を超えるとき、とする。環境によって上限が違うことは未確認) |
| `ArgumentNullException` | `startInfo` が `null`(起こらない) |
| `PlatformNotSupportedException` | シェルのない OS(Nano Server など。この仕様の対象外) |
| (戻り値 `null`) | プロセスのリソースが起動されなかった |

UNIX 系の実装(`main` ブランチのソース。確認。出典: `Process.Unix.cs`、`pal_process.c`)では、`FileName` が見つからない場合は `ENOENT` を表す `Win32Exception`、`FileName` がディレクトリの場合は `Win32Exception`(メッセージは「ディレクトリは入力として有効でない」)、`fork` または `exec` の失敗は、子が `errno` をパイプで親に伝え、親が `Win32Exception`(`NativeErrorCode` に `errno`)にする。

**規定**: 次の例外はすべて「起動の失敗」として捕まえ、`StartFailure = true` にする。原因(見つからない・実行権限がない・実行できない形式・ディレクトリを指定した、など)と、`NativeErrorCode` の値を区別しない【規定】。例外のメッセージも、表示に使わない(`NativeErrorCode` と `Message` を、標準出力にも標準エラーにも出さない。入力の値や OS の文が混ざらないようにするため)。

```text
catch (Win32Exception)                  → 起動の失敗
catch (InvalidOperationException)       → 起動の失敗
catch (ArgumentException)               → 起動の失敗(`FileName` や引数に、OS が受け付けない文字がある場合。例外の型は未確認。念のための備え)
catch (PlatformNotSupportedException)   → 起動の失敗
```

上の 4 つ以外の例外(`OutOfMemoryException` など)は捕まえない(想定外の内部の誤りとして、10 章の経路で終了コード 70 にする)。`catch (Exception)` を、ここに書かない【規定】。

起動の失敗のときの終了コードは 127(分類。4.6 節)。表示は `conductor: error: job '<ID>': cannot start command '<プログラム名>'`(英語。日本語は [08-output-and-i18n.md](08-output-and-i18n.md))。ジョブの終了コード・シグナルは `null`。

### 4.4 終了コードの取り出し(`Process.ExitCode`)

`ExitCode` は `int` である。プロセスが終了する前に取得すると `InvalidOperationException`(確認。出典: `Process.ExitCode` の API 文書)。`WaitForExit(int)` が `true` を返したあとで取得する(5 章)。

**UNIX 系**

- 通常の終了(`exit(n)`)では、`ExitCode` は `n` の下位 8 ビット(0〜255)。
- シグナルで終了したとき、`ExitCode` は `128 + シグナル番号` になる(確認。出典: `main` ブランチの `pal_process.c` の `SystemNative_WaitPidExitedNoHang`: `WIFEXITED` なら `WEXITSTATUS`、`WIFSIGNALED` なら `128 + WTERMSIG`。公式の API 文書には書かれていない。.NET 10 のリリース版が同じかは未確認)。
- したがって、`ExitCode` だけでは、「シグナルで終了した」と「自分で `exit(137)` した」を区別できない。ランタイムの内部には、終了したシグナルを別に保持する経路があるが、公開 API として使える形かは、この文書の作成時点で確かめられていない(.NET 11 以降の `ProcessExitStatus` に相当する型がソースにあるが、最新の LTS である .NET 10 で使えるかは未確認)。`waitpid` を自分で呼ぶことは、ランタイムの子の回収(`SIGCHLD` の処理)と競合しうるため、採らない(4.7 節の「採らなかった案」)。

**【.NET の設計】決定(UNIX 系)**: 終了を観測したあと、`code = process.ExitCode` について、次のとおり決める。

```text
if (code が 129 以上 159 以下(= 128 + 1 から 128 + 31)
    かつ code が job.SuccessExitCodes にも job.WarningExitCodes にも含まれない):
    TerminatedBySignal = code - 128
    ExitCode = null
else:
    ExitCode = code
    TerminatedBySignal = null
```

- 129〜159 は、標準のシグナル 1〜31 に対応する(Linux、macOS で、この範囲は共通。リアルタイムシグナルは対象外)。
- 成功・警告の終了コードに含まれるときは、利用者が意図した終了コードとして、シグナルとはみなさない。
- 帰結(他の実装との差)【規定: この版の挙動】: (1) 自分で `exit(130)`(例えば、シェルスクリプトが `SIGINT` を受けた子の状態を `exit $?` で伝える)で終わったジョブは、「シグナル 2 で終了」として扱う。ステートに終了コードは残らず、`-v` の補足は `terminated by signal 2` になる。報告する終了コードは、どちらでも `128 + n`(= `exit` した値)で同じ。(2) シグナルで終了したジョブは、リトライの対象の終了コード(`retry.exit_codes`)に `128 + n` が含まれていても、リトライしない(分類で、シグナル終了は終了コードを持たないため)。自分で `exit(128 + n)` したジョブは、同じ理由で、リトライの対象外になる(他の実装と異なる。成功・警告の指定には影響しない)。

**Windows**

- `ExitCode` は、OS が報告する 32 ビット符号つき整数のまま(例: 例外による終了は負の値になりうる)。`TerminatedBySignal` は常に `null`。`ExitCode` を、そのまま `int` として保つ(`uint` や `byte` に変換しない)【規定】。報告する終了コードの 0〜255 への合わせ方(下位 8 ビット)は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md)。

### 4.5 実行結果(`JobExecution`)の各項目

```csharp
namespace Conductor.Flow;

public sealed record JobExecution(
    string JobId,
    bool Started,              // false は、起動の前に合図が立っていたときだけ
    int? ExitCode,             // UNIX 系でシグナル終了とみなしたときは null。Windows は常に値がある
    int? TerminatedBySignal,   // UNIX 系のシグナル終了とみなした番号。Windows は常に null
    bool StartFailure,
    bool Interrupted,
    int? SignalNumber);        // Interrupted のとき、最初に受け取ったシグナルの番号
```

| 項目 | 意味 |
|------|------|
| `JobId`、(`command`) | 定義のジョブID(と `command`)。`command` は、表示で使うときに定義から引く |
| `Started` | 起動を試みたか。`false` は、起動の前に合図が立っていたときだけ |
| `ExitCode` | 4.4 節の規則で決めた終了コード。UNIX 系の通常の終了は 0〜255、Windows は 32 ビット符号つきのまま |
| `TerminatedBySignal` | UNIX 系のシグナル終了の番号(4.4 節の規則による推定) |
| `StartFailure` | 分割の失敗、起動の失敗、終了の状態の取得の失敗のいずれか |
| `Interrupted` | 起動前、または、終了の観測の直後に、合図が立っていた |
| `SignalNumber` | `Interrupted` のとき、**最初に受け取った**シグナルの番号 |

### 4.6 分類(終了コードの決定)

実行結果は、次の優先順位で、最初に当てはまるものを採る。

| 順 | 条件 | 分類 | 報告する終了コード |
|----|------|------|--------------------|
| 1 | `Started = false` かつ `Interrupted` | 分類なし(エラーとして記録しない) | `128 + SignalNumber` |
| 2 | `Started = false` で中断でもない(起こらない) | エラー | 127 |
| 3 | `Interrupted`(起動済み) | エラー | `128 + SignalNumber`(ジョブ自身の終了コードは見ない。ジョブが 0 で終わっていてもエラー) |
| 4 | `StartFailure` | エラー | 127 |
| 5 | `TerminatedBySignal = n` | エラー | `128 + n` |
| 6 | `ExitCode = null`(起動して終わったのに値がない) | エラー(`conductor` 自身のエラー) | 1 |
| 7 | `ExitCode` が成功コード(既定は `[0]`、`success_exit_codes` で指定) | 成功 | 0 |
| 8 | `ExitCode` が `warning_exit_codes` | 警告 | 0 |
| 9 | `ExitCode = 0` で成功コードに含まれない | エラー(`conductor` 自身のエラー) | 1 |
| 10 | 上記以外 | エラー | `ExitCode` のまま |

`conductor` の終了コードは、報告する終了コードを 0〜255 に合わせたもの(範囲内はそのまま。範囲外は下位 8 ビット。エラーで下位 8 ビットが 0 になるときは 1)。`int` の負の値は、`unchecked((byte)value)` と同じ下位 8 ビットを取る。詳細は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md)。

### 4.7 新しいプロセスグループ/セッションに置くか【.NET の設計】

**決定: UNIX 系でも、ジョブを新しいプロセスグループにも新しいセッションにも置かない。`conductor` と同じプロセスグループで起動する。**

| 項目 | 内容 |
|------|------|
| 公式の API で得られるもの | `ProcessStartInfo.CreateNewProcessGroup` は、**Windows 専用**(`SupportedOSPlatform("windows")`)で、.NET 10 以降にある(確認。出典: `ProcessStartInfo.CreateNewProcessGroup` の API 文書。Windows では、新しいプロセスの Ctrl+C が無効になる、と書かれている)。`ProcessStartInfo.StartDetached` は、UNIX 系で「新しいセッションのリーダー」にするが、**.NET 11 だけ**にあり、標準入出力を既定でヌルデバイスにつなぐ(確認。出典: `ProcessStartInfo.StartDetached` の API 文書)。この仕様は .NET 10(LTS)を対象とし、標準入出力を引き継ぐため、使わない |
| `setsid` / `setpgid` を P/Invoke で呼ぶ案 | 採らない。ライブラリ呼び出しは、親(`conductor`)の側でしか行えない。`Process.Start` の `fork` と `exec` のあいだに、子の側で `setsid` / `setpgid` を差し込む公開の手段がない(`pal_process.c` の `fork` 後の処理で `setsid` を呼ぶのは `startDetached` のときだけ。確認。出典: `main` ブランチのソース)。起動のあとに、親から `setpgid(pid, pid)` を呼ぶと、子が `exec` を終えたあとは、失敗する(POSIX の規定。実機で未確認)。起動と `exec` のあいだの競合があるため、信頼できない |
| `posix_spawn` の `POSIX_SPAWN_SETPGROUP` を P/Invoke で呼び、`Process` を使わずに起動する案 | 採らない。(1) ランタイムの `SIGCHLD` の処理と、自前の `waitpid` が競合しない保証が、公式のドキュメントにない(未確認)。(2) `posix_spawn_file_actions_t` などの構造体の大きさと、定数が、OS ごとに違う。(3) 環境変数の組み立てと、`PATH` の探索を、自前で持つことになる。(4) 得られるのは、シグナル終了の正確な区別(4.4 節)と、新しいグループであり、効果が実装の複雑さに見合わない。将来、この案を採るときは、この章と、受け入れテスト(「新しいプロセスグループ」と「シグナルで終了」)を、あわせて改める |

帰結(他の実装との差。受け入れテストの項目に影響する。他の担当と合わせる点):

- 端末の Ctrl+C(SIGINT)は、端末のフォアグラウンドのプロセスグループ全体に配られるため、ジョブにも**直接**届き、そのあと `conductor` の転送(7.2 節)でもう 1 回届く(ジョブは SIGINT を 2 回受けうる。1 回目で終わるジョブが多い)。`kill -INT <conductor の pid>` のように、`conductor` の pid だけに送った場合は、転送の 1 回だけである。
- 「ジョブが新しいプロセスグループにある」ことを確かめる試験(プロセスグループ ID が子の pid)は、この版では行わない。
- ジョブは、`conductor` と同じ制御端末を持つ(`/dev/tty` を開くジョブは動く。標準入力が端末のジョブが、背景のグループとして停止する問題も起こらない)。

### 4.8 Windows の `.cmd` / `.bat` の起動【.NET の設計】

`.cmd` / `.bat` の起動の方法と、引数の安全な渡し方は、公式の API 文書に書かれていない。次のとおり決める。

**問題**: Windows では、`.cmd` / `.bat` は `cmd.exe` が解釈する。`ArgumentList` は、引数を、一般のプログラムの規則(`CommandLineToArgvW` の規則。確認: `ArgumentList` の API 文書は「エスケープする」とだけ書く。規則の詳細は未確認)で 1 本の文字列にする。この文字列を `cmd.exe` が再解釈するとき、`"`、`%`、`^`、`&`、`|`、`<`、`>` などが、別の意味を持ち、引数の中のコマンドが実行されうる。この問題に、.NET が対策しているかは、未確認である。

**決定(`cmd.exe` を直接起動する。`UseShellExecute = false` のまま)**:

1. 次のとき、かつ、そのときだけ、この規則を使う: 実行中の OS が Windows であり、`Program` の拡張子が、大文字小文字を区別せず `.cmd` または `.bat`(`Path.GetExtension(program)` を序数の大文字小文字無視で比較する。`Path.GetExtension(".cmd")` のような、拡張子だけの名前の扱いは未確認なので、ファイル名の末尾の比較(`EndsWith(".cmd", StringComparison.OrdinalIgnoreCase)`)でもよい)。拡張子のない `Program`(例: `build`)は、OS の標準の探索に任せる(`.cmd` は補われない。見つからなければ起動の失敗)。
2. 検査: `Args` のどれかが、次の文字を含むなら、**起動せず**、起動の失敗の実行結果を返す。`Program` は、`(` と `)` を除く同じ集合を検査する(パス `C:\Program Files (x86)\...` を許す)。

```text
"  %  ^  &  |  <  >  !  (  )  改行(\n)  復帰(\r)  NUL(U+0000)
```

3. 検査を通ったら、`FileName` に `cmd.exe` のパス、`Arguments`(`ArgumentList` ではない)に次の 1 本の文字列を設定する。
   - `cmd.exe` のパス: 環境変数 `ComSpec` が、絶対パス(`Path.IsPathRooted`)ならその値、そうでなければ `cmd.exe`(環境変数は、`IEnv.LookupEnv` を通して受け取る。2 章 [02-architecture.md](02-architecture.md) の `Deps`)。
   - `Arguments`: `/d /s /c "` + `<全体>` + `"`。`<全体>` は、`Program` と各 `Args` を、それぞれ `"` で囲み、空白 1 つで連結した文字列(空の引数は `""`)。例: `Program = C:\x\build.cmd`、`Args = [a b, c]` → `/d /s /c ""C:\x\build.cmd" "a b" "c""`。
   - `/d`(AutoRun のコマンドを実行しない)、`/s` と `/c`(最初と最後の `"` を取り除いて、残りを実行する)の意味は、Windows の `cmd` の文書による(この文書では未確認)。
   - `Arguments` が、OS に、そのまま渡されること(.NET が再び引用しないこと)は未確認。
4. 配列形式の `command` も、同じ検査を通る。
5. 検査の関数と、`Arguments` の組み立ては、どの OS でも動く純粋な関数にする(`CmdLauncher.Build(program, args, lookupEnv)` など。Linux の 1 台から、Windows の分岐を単体テストできる)。

この禁止の文字の集合と、`cmd.exe` の起動の形は、この仕様の設計判断であり、Windows の実機で確かめていない(未確認)。`cmd.exe` が `.bat` の実行中に Ctrl+C を受けたとき「バッチ ジョブを終了しますか (Y/N)?」と尋ねる場合の影響も未確認(手動の確認項目。11 章)。

Windows で、拡張子のない `Program` や、`.exe` の探索(アプリケーションのあるディレクトリ、カレントディレクトリを含むか、など)は、`CreateProcess` の標準の規則に依存する(未確認)。

### 4.9 UNIX 系の実行ファイルの解決【.NET の設計】

`Process.Start` は、`UseShellExecute = false` のとき、`FileName` が絶対パスでなければ、次の順で探す(確認。出典: `main` ブランチの `Process.Unix.cs` の `ResolvePath`。公式の API 文書には、この順序は書かれていない): (1) `conductor` 自身の実行ファイルのあるディレクトリ、(2) カレントディレクトリ、(3) `PATH`。(1) と (2) は、ファイルがあれば(実行権限を見ずに)採る。これは、「`program` に区切り文字がなければ、OS の標準の探索(`PATH` だけ)」という規則に反する(`conductor` と同じディレクトリや、カレントディレクトリにある同名のファイルが、先に起動されてしまう)。区切り文字を含む相対パス(`scripts/x.sh`)も、(1) が先に試される。

**決定(UNIX 系)**【規定】: `UnixPlatform` が、`Process.Start` を呼ぶ前に、`FileName` を絶対パスに解決する。

| `Program` | 解決 |
|-----------|------|
| `/` を含む(絶対パスまたは相対パス) | `Path.GetFullPath(program)`(カレントディレクトリが基準)。存在を確かめない(`exec` の失敗は、4.3 節の例外になる) |
| `/` を含まない | `PATH`(環境変数。`LookupEnv("PATH")`)を `:` で分け、各ディレクトリで、`Path.Combine(dir, program)` のファイルを探す。空のエントリ(`PATH` の先頭、末尾、`::`)は、カレントディレクトリ。ファイルであり(ディレクトリでない)、実行ビットのどれか(`File.GetUnixFileMode` の `UserExecute`、`GroupExecute`、`OtherExecute`)が立っている最初のものを採る。`PATH` が未設定のときは、探索せず、起動の失敗(実機での OS の既定の `PATH` は使わない)。見つからなければ、起動の失敗(127) |

`File.GetUnixFileMode` は .NET 7 以降の API である(確認: API の存在は既知。この文書では出典の確認を行っていない。未確認)。`FileName` に絶対パスを渡すと、`Process.Start` は、探索せず、そのまま使う(確認。出典: `ResolvePath` の「完全なパスなら、存在を問わず使う」)。

---

## 5. 待ち(`WaitForExit(int)` の繰り返し)

### 5.1 規定

| 項目 | 内容 | 強さ |
|------|------|------|
| 待ち方 | `Process.WaitForExit(int milliseconds)` の繰り返し(待機ループ)。引数なしの `WaitForExit()` を使わない。理由は 5.2 節 | 【.NET の設計】 |
| 回収 | 終了の状態を読む(`WaitForExit(int)` が `true` を返し、`ExitCode` を取る)のは、`JobRunner` の待機ループだけ | 【規定】 |
| 転送の判断 | 「ジョブがまだ終わっていない」と観測した時点で、**まだ転送していない**かつ合図が立っていれば、シグナルをジョブに **1 回だけ** 転送する(7.2 節) | 【規定】 |
| 終了済みへの送信の禁止 | 終了済み・回収済みのジョブには送らない。ジョブが先に終わっていれば、合図が立っていても転送しない | 【規定】 |
| 転送は 1 回 | 2 回目以降の合図(同じシグナルでも別のシグナルでも)で、再度の転送や強制終了はしない | 【規定】 |
| 転送のあと | ジョブの終了を待ち続ける(ジョブがシグナルを無視すれば、`conductor` はそのまま待ち続ける) | 【規定】 |
| 確認の間隔 | `WaitForExit(int)` の待ち時間を、1、2、4、8、10、10、…(ミリ秒。前回の 2 倍、上限 10 ms)にする。ジョブが終わったときは、待ち時間の経過を待たずに戻る(UNIX 系の子プロセスでは、イベントによる待ち。確認: 出典: `ProcessWaitState.Unix.cs` の `WaitForExit`)。Windows のタイマーの分解能は、これより粗いことがある(未確認) | 【目安】(間隔が長すぎて、合図からの転送が目に見えて遅れないこと。10 ms 以下を推奨) |
| 待機の専用スレッド | 使わない(呼び出したスレッドで待つ) | 【規定】 |
| CPU の使用 | 待ち中の CPU 時間の増分が小さいこと(待機の 1 秒後から 2.5 秒後までで 50 ms 以下。11 章。Linux の `/proc` で計測) | 【目安】(.NET での実測は未確認) |

### 5.2 `WaitForExit()` と `WaitForExit(int)` の使い分け

| 方法 | 動作 | 採否 |
|------|------|------|
| `WaitForExit()`(引数なし) | 終了まで無期限に待つ。さらに、標準出力が非同期のイベントハンドラにつながれているとき、そのイベントの処理が終わるまで待つ(確認。出典: `Process.WaitForExit` の API 文書の Remarks)。この仕様では、標準入出力をつながないので、イベントの処理待ちは起こらない。ただし、無期限に待つため、合図が立っても、呼び出しが戻らず、転送できない(転送の規定に反する) | 使わない |
| `WaitForExit(int)` | 指定のミリ秒だけ待ち、終了していれば `true`、そうでなければ `false`。`0` は即時に戻り、終了済みのときだけ `true`。`-1` は無期限(`WaitForExit()` と同じ)。負の値(`-1` 以外)は `ArgumentOutOfRangeException`(確認。出典: `Process.WaitForExit(Int32)` の API 文書)。標準出力が非同期のイベントハンドラにつながれているときは、`true` が返っても、そのイベントの処理が終わっていないことがある(確認。同上)。この仕様では起こらない | **採用**。待機ループの中心。待ち時間は、10 ms 以下(0 以上) |
| `Process.HasExited` | 終了しているか。ブロックしない | 転送の直前の確認(7.2 節)に使う |
| `Exited` イベント(`EnableRaisingEvents`) | 終了を、スレッドプールのコールバックで通知する | 使わない(コールバックの順序の規則が増えるため。9 章) |
| `WaitForExitAsync` | 非同期 | 使わない(同期モデル。9 章) |

**`WaitForExit()`(引数なし)が、子孫の標準出力の待ちでブロックする条件**: 標準出力または標準エラーを `Redirect*=true` でつなぎ、`BeginOutputReadLine` などの非同期の読み取りを使うと、`WaitForExit()` は、その読み取りが終わる(パイプの EOF)まで戻らない。ジョブの子孫(`sh -c` が起こした孫など)が、そのパイプの書き込み側を持ったまま動いていると、ジョブ本体が終わっていても、EOF が来ないため、`WaitForExit()` が戻らない、ことが起こりうる(この「子孫が書き込み側を持つ」場合の詳細な挙動は、公式の API 文書に書かれていない。未確認)。この仕様は、標準入出力をつながないので、この問題は起こらない。したがって、`Redirect*=false` を**必ず**明示することが、この問題を避ける条件である。

### 5.3 待機ループの手順(骨格。実装コードではない)

```text
forwarded = false
interval = 1                                        // ms
loop:
    try:
        if p.WaitForExit(0): break                  // すでに終了している(回収済み)
        n = sig.Number                              // 1 回だけ読み、その値を使う(9.2 節)
        if n != null and not forwarded:
            forwarded = true                        // 成否にかかわらず 1 回でやめる
            platform.Forward(p, n.Value)            // 例外を握りつぶす(7.2 節)。HasExited を確かめてから送る
        if p.WaitForExit(interval): break
    catch (Win32Exception) / (InvalidOperationException):
        結果 = 起動の失敗(4.1 節の手順 8); return
    interval = min(interval * 2, 10)
結果 = 4.4 節で取り出す
```

- ループの 1 周目で、`WaitForExit(0)` の直後に合図を読む。したがって、`Process.Start` が返る前に立った合図は、1 周目で転送される。
- `IPlatform.Forward` を差し替え可能にする(単体テストで、転送の回数を数える)。待ち時間(`WaitForExit` に渡すミリ秒)の列を、差し替え可能な関数(`Func<Process, int, bool>`)で受け取ると、偽の子プロセスで、列 1、2、4、8、10、10、… を単体で検査できる(11 章)。
- **子の終了と `pid` の再利用**: .NET は、`SIGCHLD` を受けると、ランタイムの内部のスレッド(ネイティブのスレッド)が、子を回収する。つまり、子が終わった直後に、`conductor` のスレッドが `kill` を呼ぶ前に、子が回収され、`pid` が別のプロセスに再利用される競合が、理論上はありうる(`WaitForExit` はイベントで待つ。確認。出典: `ProcessWaitState.Unix.cs` の `WaitForExit` と `CheckChildren`)。この競合を狭めるため、転送の直前に `Process.HasExited` を確かめ、`true` なら送らない(7.2 節)。確認から `kill` までの、ごくわずかな間は、残る。`pid` は、ふつう順に割り当てられ、すぐには再利用されない(Linux、macOS の通常の挙動。この文書では確かめていない)ので、この残りの競合は許容する(「終了済みの子には送らない」規定の、実装上の限界。未確認)。
- `Process.Id` を、転送の直前に 1 回だけ読む(`HasExited` が `true` になったあとに `Id` を読むと、例外になりうる。未確認)。

---

## 6. 中断の合図(`Interrupt`)

「中断の合図」は、`conductor` がシグナルを受け取ったことを、ジョブを待つ側へ伝える値。

### 6.1 規定

| 項目 | 規定 |
|------|------|
| 内容 | 「立っているか」と、立っているなら「最初のシグナルの番号」 |
| 初期 | 立っていない |
| 立てる(番号 n) | 立っていなければ n を保存する。**すでに立っていれば何もしない**(最初の番号が残る)。番号は任意の整数でよく、0、負数、非常に大きい値でも、「立った」ことは記録される |
| 性質 | 立ったら下りない。シグナルのコールバックのスレッドと、待つ側のスレッドのあいだで、安全に共有できる(競合しない)。待つ側は読むだけ |
| 判定 | 「立っているか」は `Number != null` で判定する(番号 0 を「立っていない」と取り違えない) |

### 6.2 型の骨格【.NET の設計】

```csharp
namespace Conductor.Runner;

public sealed class Interrupt
{
    // 立っていなければ n を保存して立てる。すでに立っていれば何もしない。例外を送出しない。
    // 実装: 0 を「立っていない」とする long に、(1L << 32) | (uint)n をまとめて保存し、
    //       Interlocked.CompareExchange(ref value, packed, 0) の成功だけが番号を決める
    //       (「立っているか」と「番号」を 1 つの原子的な値にして、片方だけが見える状態を作らない)。
    //       成功したあとに、ManualResetEventSlim.Set() で、待っている側を起こす。
    public void Trigger(int n);

    // 立っていれば最初の番号。立っていなければ null。Volatile.Read で読む。
    public int? Number { get; }

    // 立つまで最大 timeout だけ待つ。立っていれば true。待機の起こし方(リトライの待機。7.5 節)。
    public bool WaitRaised(TimeSpan timeout);
}
```

- `ManualResetEventSlim` は、`Set` の後に `Reset` しない(立ったら下りない)。`Wait(TimeSpan)` は、`Set` されていれば直ちに `true` を返す。

---

## 7. シグナル処理

### 7.1 準備(両 OS 共通)

- `run` と `pipeline` だけが、**定義の検査より前**に、シグナルの監視を始める【規定】。`watch`・`reset`・`validate`・`convert` は監視せず、OS の既定の動作で終わる。
- 中断の合図(`Interrupt`)を 1 つ作り、`SignalWatcher.Start(Interrupt)` に渡す。
- 監視の登録は **プロセスで 1 回だけ**(静的な印を `Interlocked` で立てる)。2 回目の呼び出しは「すでに登録済み」(`SignalStart.AlreadyRegistered`)として、流れを成功と同様に続ける(2 回目に渡した合図には配線されない)。登録に失敗したとき(`SignalStart.Failed`)は、標準エラーに `conductor: error: failed to set up signal handling` を書き、終了コード **70** で終わる(定義の検査もジョブの起動もしない)。登録に失敗した場合は、すでに登録できた分を `Dispose` し、印を戻して、次の登録を許す。
- 「最初の番号」は、`Interrupt.Trigger` が、原子的な比較交換に成功した順の最初(6 章)。

```csharp
public enum SignalStart { Started, AlreadyRegistered, Failed }

public sealed class SignalWatcher
{
    // 本番: プロセスで 1 回だけ登録する。2 回目は AlreadyRegistered。
    public static SignalWatcher Start(Interrupt sig, IPlatform platform);
    public SignalStart Result { get; }
    // 登録を元に戻す(Dispose)。本番では呼ばない(終了まで有効)。単体テストが使う。
    public void Stop();
}
```

### 7.2 UNIX 系(Linux、macOS)

**登録**: 次の 4 つのシグナルを、`PosixSignalRegistration.Create(signal, handler)` で受ける【規定】。

| シグナル | `PosixSignal` の値 | 実際の番号(`128 +` に使う) |
|----------|--------------------|------------------------------|
| SIGHUP | `PosixSignal.SIGHUP`(-1) | 1 |
| SIGINT | `PosixSignal.SIGINT`(-2) | 2 |
| SIGQUIT | `PosixSignal.SIGQUIT`(-3) | 3 |
| SIGTERM | `PosixSignal.SIGTERM`(-4) | 15 |

`PosixSignal` の列挙値は、負の値(OS に依存しない識別子)であり、シグナル番号そのものではない(確認。出典: `PosixSignal` の API 文書。SIGKILL = -11、SIGTSTP = -10、SIGTTOU = -9、SIGTTIN = -8、SIGWINCH = -7、SIGCONT = -6、SIGCHLD = -5、SIGTERM = -4、SIGQUIT = -3、SIGINT = -2、SIGHUP = -1)。`Trigger` に渡すのは、上の表の「実際の番号」(1、2、3、15)である【規定】。この 4 つの番号は、Linux と macOS で同じである(POSIX の規定。実機では未確認)。変換は、ハンドラ(ラムダ)の外側で、定数として結びつける(各シグナルごとに、番号を持った別のラムダを作る)。

**ハンドラ**【規定】:

```text
handler(PosixSignalContext ctx):
    ctx.Cancel = true           // ランタイムの既定の動作(プロセスの終了)を打ち消す
    interrupt.Trigger(番号)      // これだけ。出力・ファイルの書き込み・ロック・ブロックをしない
```

- `PosixSignalContext.Cancel` の既定は `false` で、ハンドラのあと、`Cancel` が `false` のままなら、ランタイムが、そのシグナルの既定の動作(終了など)を実行する(確認。出典: `PosixSignalContext.Cancel` の API 文書と `PosixSignalRegistration.Unix.cs` の `if (!context.Cancel)` の分岐)。登録後、これらのシグナルで、OS の既定の動作(終了)をしなくなる、という規定(「受け取るたびに、合図を立てる」)を満たすため、4 つとも `Cancel = true` にする。SIGTERM を打ち消せるのは UNIX 系だけである(確認。出典: `PosixSignalRegistration.Create` の API 文書の Remarks: SIGINT と SIGQUIT は Windows と UNIX 系の両方で、SIGTERM は UNIX 系だけで、打ち消せる)。
- `Console.CancelKeyPress` は、UNIX 系では**併用しない**【決定】。SIGINT は `PosixSignalRegistration` で受けるため、重ねて登録すると、同じ 1 回の Ctrl+C に対して 2 つのコールバックが動きうる(2 つの関係は未確認)。`Trigger` は冪等なので、結果は変わらないが、不要な複雑さを避ける。
- ハンドラは、メインスレッドとは別のスレッドで動く(確認。出典: `main` ブランチの `PosixSignalRegistration.Unix.cs`: SIGINT、SIGQUIT、SIGTERM は、専用の背景スレッド(名前 ".NET Signal Handler")で、それ以外(SIGHUP を含む)は、スレッドプールで、ハンドラを動かす。公式の API 文書には、この記述はない)。ハンドラの数と並びは、1 つのシグナルにつき 1 つ。複数のハンドラがあるときの順序は、.NET 10 以降、登録の逆順(確認。出典: `PosixSignalRegistration.Create` の API 文書)。この仕様では、1 つのシグナルに 1 つだけ登録する。
- **登録の寿命**: `PosixSignalRegistration` の `Finalize()` は、ハンドラを解除する(確認。出典: `PosixSignalRegistration` の API 文書の「メソッド」の表)。したがって、`Create` の戻り値(4 つ)を、**静的なフィールドに保持**して、ガベージコレクションで解除されないようにする【規定】。戻り値を捨てる、またはローカル変数だけに持つと、実行の途中でハンドラが解除されうる。
- 例外: `Create` は、`PlatformNotSupportedException`(その OS で、このシグナルを使えない)、`IOException`(シグナルの設定に失敗した)を送出しうる(確認。出典: `PosixSignalRegistration.Create` の API 文書)。どちらも、`SignalStart.Failed`(終了コード 70)にする。`ArgumentNullException`(ハンドラが `null`)は、起こらない。
- `PosixSignal.SIGCHLD` を登録しない【規定】(.NET のランタイムが、子の回収に使う。`Cancel` で、端末の設定を打ち消すことができる、と書かれている。確認: `PosixSignalRegistration.Create` の API 文書。この仕様は、ランタイムの子の回収に干渉しない)。

時系列(ジョブの実行中に、利用者が `kill -TERM <conductor の pid>` したとき):

1. `conductor` が SIGTERM を受け取る。ランタイムが、背景スレッドで、ハンドラを呼ぶ。ハンドラが `Cancel = true` にして、`Trigger(15)` を呼ぶ(合図が立つ)。
2. メインスレッドの待機ループは、次の確認(最大 10 ms 後)で「まだ終わっていない」を観測し、合図が立っているのを見る。
3. ジョブの pid に、**同じシグナルを 1 回**送る。送る前に次を検査する(`UnixPlatform.Forward`)。
   - `Process.HasExited` が `false`(`true`、または `InvalidOperationException` なら、送らない)。
   - pid(`Process.Id`)が 0 より大きい(0 や負にならない。プロセスグループへの送信にならない)。
   - シグナルが `{1, 2, 3, 15}`(SIGHUP、SIGINT、SIGQUIT、SIGTERM)のどれか(SIGKILL(9)、SIGSTOP、SIGCONT、SIGUSR1、0、負、9999 は送らない)。
   - 送信先は、**ジョブの子プロセスの pid だけ**。ジョブの子孫(`sh -c` が起こした孫など)には、`conductor` は送らない。`kill` の第 1 引数に、負の値や 0 を渡さない。
   - 送信は、`libc` の `kill(int pid, int sig)` を P/Invoke で呼ぶ(7.2.1 節)。戻り値(失敗)は無視する。
4. ジョブは、受け取ったシグナルの既定の動作で終了するか、自分で処理する。`conductor` は待ち続ける。2 回目以降のシグナルでは、何も追加しない。
5. ジョブの終了を観測する。`ExitCode` から、4.4 節の規則で `ExitCode` と `TerminatedBySignal` を決める。直後に合図が立っているので `Interrupted = true`、`SignalNumber = 15` を足す。
6. 分類で「エラー、`128 + 15 = 143`」となり、流れの制御は、この結果をステートに**エラーとして**記録する(再開位置はそのジョブ)。
7. `conductor` は **143** で終わる(`pipeline` は次のジョブを起動しない)。

その他:

- ジョブは、`conductor` と同じプロセスグループで動く(4.7 節の決定)。端末の Ctrl+C(SIGINT)は、ジョブにも直接届き、`conductor` の転送でもう 1 回届く(4.7 節の帰結)。ジョブは `TerminatedBySignal = 2`(4.4 節の規則で推定)で終わるのが典型。
- 転送の許可集合にない番号で合図が立った場合(`conductor` 本体の監視は 4 つだけを受けるため、通常は起こらない。起動の部品を単独で使う場合のみ)は、送信せず、ジョブが自分で終わるのを待ち、終わりを観測した後に `Interrupted = true`、`SignalNumber = その番号` を足す(ジョブの終了コードは `ExitCode` に残るが、分類では見ない)。
- 時間を空けて複数のシグナルが届いたときは、到着順の最初の番号が残る(INT のあとの HUP でも 2)。ほぼ同時に届いたときに、どれが「最初」になるかは**規定しない**(シグナルごとに、別のスレッドでハンドラが動くため、順序は決まらない。確認: 上記のスレッドの規則)。
- 終了コードの例: SIGHUP は 129、SIGINT は 130、SIGQUIT は 131、SIGTERM は 143。

#### 7.2.1 `kill` の P/Invoke【.NET の設計】

.NET の管理 API には、任意のシグナルを子に送るものがない。`Process.Kill()` は強制終了(SIGKILL)である(確認。出典: `Process.ExitCode` の API 文書の「`Kill` で終了させられる」という記述と、`Process.Unix.cs` の `Interop.Sys.Kill(_processId, SIGKILL)`)。したがって、UNIX 系では、`libc` の `kill` を P/Invoke で呼ぶ。

```csharp
namespace Conductor.Runner;

internal static partial class UnixNative
{
    [LibraryImport("libc", EntryPoint = "kill")]
    internal static partial int Kill(int pid, int sig);
}
```

| 項目 | 内容 |
|------|------|
| `LibraryImport` | .NET 7 以降の P/Invoke のソース生成。`static partial` なメソッドに付ける。**プロジェクトに `AllowUnsafeBlocks` が要る**(確認。出典: 「P/Invoke source generation」の文書)。Native AOT やトリミングでも使える形(確認。同上。`DllImport` は実行時に IL を生成するため、AOT では使えない)。`Conductor.Runtime.csproj` に `<AllowUnsafeBlocks>true</AllowUnsafeBlocks>` を書く |
| 呼び出しの 1 か所 | この P/Invoke は、`UnixNative` の 1 か所だけにある【規定】(8.2 節。ほかのファイルで `LibraryImport` / `DllImport` を使わない) |
| ライブラリ名 | `"libc"`。.NET は、Linux と macOS で、名前に `.so` / `.dylib` を足した候補、`lib` を前につけた候補、そのままの候補を、順に探す(確認。出典: 「Native library loading」の文書)。Linux で、`libc.so.6` に解決されるか、macOS で、`libc` が解決されるかは、文書に書かれていない(未確認)。そのため、**`NativeLibrary.SetDllImportResolver` で、解決を明示する**【規定】 |
| 解決の明示 | `UnixNative` の静的コンストラクターで、`Conductor.Runtime` のアセンブリに、`DllImportResolver` を 1 つ設定する(1 つのアセンブリに 1 つだけ設定できる。確認。出典: 同文書)。`libraryName == "libc"` のとき、Linux では `NativeLibrary.TryLoad("libc.so.6", ...)` を、続けて `NativeLibrary.TryLoad("libc", ...)` を試す。macOS では、`NativeLibrary.TryLoad("/usr/lib/libSystem.B.dylib", ...)`(絶対パスは、そのまま使われ、変種を探さない。確認: 同文書)を、続けて `"libc"` を試す。どれも失敗したら `IntPtr.Zero` を返す(既定の探索に任せる)。この名前の組が、実機で正しく解決されるかは、未確認(3 OS で確かめる) |
| 引数の型 | `pid_t` を `int`、シグナル番号を `int` とする(Linux と macOS の通常の 64 ビットの ABI で、`pid_t` は 32 ビット。実機では未確認) |
| 戻り値と `errno` | 戻り値は無視する(失敗の原因を見ない)。`SetLastError` を使わない |
| ライブラリが見つからないとき | 最初の呼び出しで `DllNotFoundException` または `EntryPointNotFoundException` が起こりうる。この 2 つは、`Forward` の中で捕まえて握りつぶす(送信の失敗として無視する)。ただし、握りつぶすと、転送が黙って行われなくなるため、シグナルの転送の受け入れテスト(11 章)が、3 OS で、このことを検出する。Windows では、この型を呼ばない(`WindowsPlatform` は `Forward` で何もしない) |
| Windows での読み込み | `UnixNative` は、Windows では、型の静的コンストラクターも、メソッドも、実行されない(`UnixPlatform` からだけ参照するため)。単一ファイルの発行(`PublishSingleFile`)でも、`LibraryImport` の生成コードは、`libc` を実行時に探す(未確認) |

### 7.3 Windows

受け取るのは **Ctrl+C(番号 2)だけ**【規定】。`Console.CancelKeyPress` で受ける【決定】。

```text
Console.CancelKeyPress += (sender, e) =>
{
    if (e.SpecialKey == ConsoleSpecialKey.ControlC)
    {
        e.Cancel = true;            // プロセスの終了を打ち消す
        interrupt.Trigger(2);       // これだけ
    }
    // ControlBreak: 何もしない(e.Cancel を false のままにする。OS の既定の動作で終わる)
};
```

- `CancelKeyPress` は、Ctrl+C または Ctrl+Break で起こり、`ConsoleCancelEventArgs.SpecialKey` で、どちらかを区別できる。`Cancel` の既定は `false` で、ハンドラのあと、プロセスが終了する。`true` にすると、終了しない(確認。出典: `Console.CancelKeyPress` の API 文書)。ハンドラは、スレッドプールのスレッドで実行される(確認。出典: 同文書)。
- `PosixSignalRegistration` を Windows で使わない【決定】。Windows でどの OS のイベントがどの `PosixSignal` に対応するか(Ctrl+Break、コンソールを閉じる通知、ログオフとシャットダウン)は、公式の API 文書に書かれていない(未確認)。`CancelKeyPress` は、Ctrl+C と Ctrl+Break を `SpecialKey` で区別でき、この仕様の規定(「Ctrl+C だけ」)に合う。
- 登録(`Console.CancelKeyPress +=`)は、プロセスで 1 回だけ(7.1 節の印)。解除(`-=`)のために、ハンドラのデリゲートを保持する(`Stop` で使う)。`+=` が例外を送出する場合は、`SignalStart.Failed` にする(`PlatformNotSupportedException` などが起こりうる。未確認)。「すでに別のハンドラが登録されている」という失敗は、この API にはない(複数のハンドラが、すべて呼ばれる)。

時系列(ジョブの実行中に、利用者が Ctrl+C を押したとき):

1. ジョブは `conductor` と **同じコンソール**で起動されている(`CreateNewProcessGroup` を使わない。4.2 節)。Ctrl+C は、OS が、同じコンソールを共有するプロセス(`conductor` とジョブ)の両方に配る(Windows のコンソールの規則。この文書では実機で未確認)。
2. `conductor` のハンドラが `Cancel = true` にして、合図を立てる(番号 2)。
3. 待機ループは転送を試みるが、Windows の転送は **何もしない**(`WindowsPlatform.Forward` は空)【規定】。
4. ジョブは、OS から受け取った Ctrl+C で(既定の動作で)終了する。
5. 終了を観測する。`TerminatedBySignal` は常に `null`、`ExitCode` は OS の終了コード(Ctrl+C による終了コード)。直後に合図が立っているので `Interrupted = true`、`SignalNumber = 2`。
6. 分類で「エラー、`128 + 2 = 130`」。ステートに**エラーとして**記録し、**130** で終わる。

その他:

- Ctrl+Break とコンソールを閉じたときの通知は、この版の対象外(OS の既定の動作)。`ControlBreak` のとき `Cancel` を `false` のままにしても、実際にプロセスが終了することは、実機で未確認。
- 自動の試験環境(CI の Windows ランナー)には、対話的なコンソールがなく、Ctrl+C を安定して送る方法がないため、この経路は自動テストにしない。リリース前に実機で手動確認する(確認項目: 終了コード 130、ジョブの子プロセスが残らない、`watch` で中断したジョブが `error` で次が `pending`、`run -v` の表示に `interrupted by signal 2`)。手順は [11-quality-and-build.md](11-quality-and-build.md) の手動の受け入れテストにある。
- `Console.CancelKeyPress` の登録が、`Console` クラスの初期化(端末の設定の変更)を引き起こすかは、未確認。標準出力は、`Console.OpenStandardOutput()` から作ったストリームに書く(08 章)ため、`Console.Out` を使わない。

### 7.4 合図が立つ時点ごとの動作(流れの制御)

| 合図が立った時点 | 動作 | ステートの記録 | 終了コード |
|------------------|------|----------------|------------|
| ジョブの起動の呼び出しの前(`Started = false`) | ジョブを起動しない | 分類なし。エラーとしては記録しない。再開位置はそのジョブ | `128 + 番号` |
| ジョブの実行中 | 1 回転送(UNIX 系)。ジョブの終了を待つ | そのジョブを**エラー**として記録(ジョブが 0 で終わっても) | `128 + 番号` |
| ジョブの終了の観測の直後(転送が間に合わなかった場合を含む) | 起動済みで `Interrupted = true`(上の行と同じ扱い) | エラー | `128 + 番号` |
| `pipeline` のジョブとジョブのあいだ(ジョブの結果は成功か警告で、完了ではない) | 次のジョブを起動しない | 記述順で、まだ起動していない最初のジョブ(エラーのジョブが先にあればそれ)を再開位置にして、中断を記録。エラーとは記録しない。`-v` で `interrupted by signal N` | `128 + 番号` |
| `pipeline` の最後のジョブが完了した直後 | 完了の処理(`--remove-on-complete` ならステートを削除)をして終わる | 完了 | 0(合図は無視される) |
| リトライの待機中 | 待機を打ち切る。`Started = false`、`Interrupted = true`、`SignalNumber = 番号` の実行結果を作る。次の実行はしない | 分類なし、再開位置はそのジョブ。エラーとしては記録しない | `128 + 番号` |
| 定義の検査・ステートの読み込み・照合の途中 | 検査は最後まで進める | (定義の誤り 64、ステートの誤り 65、拒否 66、スキップ 0 の終了コードが、中断の終了コードより優先する) | 各検査の値 |

### 7.5 リトライの待機(`RetryWaiter.Wait`)

「待機(秒数, 合図)」→ 「経過」または「中断(最初の番号)」。OS 固有のコードを持たない。

```csharp
public abstract record WaitOutcome;
public sealed record Elapsed : WaitOutcome;
public sealed record Interrupted(int SignalNumber) : WaitOutcome;

public static class RetryWaiter
{
    // secs は 64 ビット符号なし(飽和した値でもよい)。例外を送出しない。
    // 時計と待ちは差し替え可能にする(単体テスト用): Func<long> timestamp(既定は Stopwatch.GetTimestamp)、
    // Func<Interrupt, TimeSpan, bool> waitRaised(既定は Interrupt.WaitRaised)
    public static WaitOutcome Wait(ulong secs, Interrupt sig);
}
```

- 手順: 開始時刻を `Stopwatch.GetTimestamp()` で取る(単調な時計。壁時計の `DateTime.UtcNow` は使わない。時刻の変更に左右されない)。ループで ① `sig.Number` が非 `null` なら、すぐ `Interrupted(最初の番号)`。② 経過(`Stopwatch.GetElapsedTime(start)` の秒)が `secs` 以上なら `Elapsed`。③ 残りの時間と、上限(250 ms)の小さい方だけ、`sig.WaitRaised(...)` で待つ。待ちの途中で合図が立てば、`WaitRaised` が直ちに戻る。
- 合図は、待機の開始時と、`WaitRaised` が戻るたび(最大 250 ms ごと)に確かめる。長い待ちでも、合図から **1 秒以内**に返る【規定】(例: 3600 秒の待機に、200 ms 後に合図を立てて、全体が 1.2 秒未満)。`Interrupt.Trigger` が `ManualResetEventSlim` を起こすので、通常は即時に返る。上限 250 ms は、起こし損ねに備える保険である(【目安】)。
- 残りの時間の計算: `secs` は `ulong`(最大 18,446,744,073,709,551,615)であり、ミリ秒に直すと `long` や `int` に収まらない。残りは、`double`(秒)で、`secs - 経過秒` として求め、`Math.Min(残り秒 * 1000, 250)` を `int` のミリ秒に直す(`ulong` から `double` への丸めは、経過が数秒のオーダーである範囲では、結果に影響しない)。`checked` で例外にしない。`TimeSpan.FromSeconds(ulong の値)` を、そのまま呼ばない(範囲外で例外になる)。
- 秒数が 0 のときは、合図が立っていなければ、すぐ `Elapsed`。合図が立っていれば `Interrupted`。
- 複数の合図は、最初の番号を返す。
- 待機の上限: 待機の合計は、`secs` で決まり、`conductor` は上限を持たない(利用者の指定が、飽和した 64 ビットの上限でも、待つ。ただし、1 つのイベントの待ちは、250 ms 以下に刻まれるので、`int` のミリ秒を超えない)。

### 7.6 シグナル登録の寿命と復元

- 本番: 登録は、プロセスの終了まで有効(`Stop` を呼ばない)。静的なフィールド(`PosixSignalRegistration` の配列、`CancelKeyPress` のデリゲート)が、参照を保つ。
- 単体テスト: `Stop` で `Dispose`(UNIX 系)/ `CancelKeyPress -=`(Windows)を行い、登録を元に戻し、静的な印を戻す。テストの終わりに、登録が残っていないこと(次のテストが、`Started` を得られること)を確かめる。
- テストで、実際のシグナルを使うとき、テストのプロセス自身に `kill` で送るのは、テストを動かすプロセスを巻き込むため、避ける。実際の `conductor` の実行ファイルを起動する受け入れテストで行う(11 章)。

---

## 8. プラットフォーム差と OS 分岐の規則

### 8.1 差の一覧

| 項目 | UNIX 系(Linux、macOS) | Windows |
|------|------------------------|---------|
| ジョブのプロセスグループ | 作らない(同じグループ。4.7 節) | 作らない(同じコンソール) |
| 監視するシグナル | SIGHUP・SIGINT・SIGQUIT・SIGTERM(`PosixSignalRegistration`) | Ctrl+C のみ(番号 2。`Console.CancelKeyPress`) |
| ジョブへの転送 | 子の pid に、シグナルを 1 回(許可集合のみ。`libc` の `kill`) | しない(OS が Ctrl+C を配る) |
| シグナル終了の読み取り | `ExitCode` が 129〜159 で、成功・警告の指定にないとき、シグナル終了と推定(4.4 節) | なし(終了はすべて終了コード) |
| 終了コードの範囲 | 下位 8 ビット | 32 ビット符号つきのまま(`conductor` が報告する値は、0〜255 に合わせる) |
| 実行ファイルの解決 | 自前で絶対パスに解決(4.9 節) | OS の標準(`CreateProcess`)に任せる。`.cmd` / `.bat` は `cmd.exe` を直接起動(4.8 節) |
| ステート書き込み後のディレクトリの同期 | ディレクトリを開いて同期する。失敗は無視。親が空なら `.`([06-state-file.md](06-state-file.md)) | しない |
| 端末の色 | 使える(標準出力が端末で `NO_COLOR` が未設定か空のとき) | 使わない(色なし) |
| 自動の受け入れテスト | 11 章の「シグナルと中断」(UNIX 系) | Windows 専用の項目と、手動の確認 |

### 8.2 分岐を置く場所の規則

OS ごとの分岐と OS 固有の呼び出しは、プラットフォーム層(OS 固有の処理をまとめた 1 か所)の外にない【規定】。

- 「OS の判定」(`OperatingSystem.IsWindows()`、`OperatingSystem.IsLinux()`、`OperatingSystem.IsMacOS()`、`RuntimeInformation.IsOSPlatform`、`Environment.OSVersion`、`Path.DirectorySeparatorChar` を OS の判定に使うこと)は、`PlatformDetector.Detect()` の 1 か所だけ。ほかの型は、`PlatformKind`(`Unix` または `Windows`)の値を引数で受け取る。
- `LibraryImport` / `DllImport`(P/Invoke)は、`UnixNative` の 1 か所だけ(シグナル送信)。ほかでは使わない。
- OS ごとの実装(`UnixPlatform`、`WindowsPlatform`)は、`IPlatform` を実装し、`PlatformDetector.Create(kind)` が、実行時に 1 つ選ぶ(`#if` の条件付きコンパイルは使わない。3 OS のテストを、同じビルドで行い、Linux の 1 台から `WindowsPlatform` の純粋な部分を単体テストするため)。
- ネットワークに接続しない(実装がネットワークの機能を持たない)。`System.Net` の名前空間を使わない。
- 製品のコードは、外部の入力(引数・環境・ファイル・OS の結果)に対して、異常終了する書き方をしない(失敗は値で返す)。
- 点検: ソースの検索(テストとして、`src/` の `.cs` を走査)で、上の判定の API の名前が、`PlatformDetector` のファイル以外に、`LibraryImport` が `UnixNative` のファイル以外に、現れないことを確かめる(11 章)。

```csharp
namespace Conductor.Runner;

public enum PlatformKind { Unix, Windows }

public static class PlatformDetector
{
    public static PlatformKind Detect();                  // OS の判定は、ここだけ
    public static IPlatform Create(PlatformKind kind);
}

public interface IPlatform
{
    PlatformKind Kind { get; }

    // ProcessStartInfo を組み立てる(4.2 節の共通の設定のあと、OS 固有の調整: 4.7〜4.9 節)。
    // 起動できない(解決失敗、禁止の文字)ときは null を返す(起動の失敗にする)。
    ProcessStartInfo? PrepareStart(CommandLine line, Func<string, string?> lookupEnv);

    // シグナルを子の pid に 1 回送る。UNIX 系: 4.2 節・7.2 節の検査のあと kill。Windows: 何もしない。例外を送出しない。
    void Forward(Process child, int signal);

    // 終了から (ExitCode, TerminatedBySignal) を決める(4.4 節)。
    (int? ExitCode, int? TerminatedBySignal) ReadExit(int rawExitCode, Job job);
}
```

---

## 9. スレッドとコールバックの規則、順序の保証

### 9.1 同期モデルの規則【規定】

`conductor` の流れは、メインスレッド(`Main` を呼んだスレッド)で、すべて**同期**で動く。子プロセスの終了、シグナル、時間の経過は、メインスレッドの `WaitForExit(int)` と `ManualResetEventSlim` の待ちと、**ランタイムが別のスレッドで動かすコールバック**(シグナルのハンドラ)が立てるフラグで伝わる。

| 規則 | 内容 |
|------|------|
| `async` / `await` | 使わない。`Task`、`Task.Run`、`Task.Wait`、`Task.Result`、`GetAwaiter().GetResult()`、`ThreadPool.QueueUserWorkItem` を使わない(デッドロックと、同期の文脈に起因する問題を作らないため。`Main` を `async` にしない) |
| スレッド | `new Thread`、`Timer`、`Parallel`、`PLINQ` を、製品のコードで使わない。理由: 子の回収と `pid` の扱いを、メインスレッドの待機ループ 1 か所に集めるため(5 章) |
| 子の終了の通知 | `Process.Exited` イベント(`EnableRaisingEvents`)を使わない(スレッドプールで動くため) |
| シグナルのコールバック | `PosixSignalRegistration` のハンドラと `Console.CancelKeyPress` のハンドラは、メインスレッドとは別のスレッド(専用の背景スレッドまたはスレッドプール)で動く(確認。出典: 7.2 節と 7.3 節)。ハンドラの中は、`ctx.Cancel = true`(または `e.Cancel = true`)と `Interrupt.Trigger` を呼ぶだけ。出力、ファイルの書き込み、ロック、`Process` の操作、`kill`、ブロックする待ち、例外の送出をしない |
| 共有する値 | `Interrupt`(`Interlocked` と `ManualResetEventSlim`。6 章)と、シグナル登録の印(`Interlocked`)だけ。`lock` を使わない(最小限。この仕様では、使う箇所がない) |
| 可視性 | 複数のスレッドから読み書きする値は、`Interlocked`、`Volatile.Read`、`Volatile.Write`、`ManualResetEventSlim` のいずれかを通す。`volatile` キーワードのフィールドと、通常のフィールドを混ぜない |
| 待ちの刻み | 待機ループは 10 ms 以下、リトライの待機は 250 ms 以下で刻む。ビジーループ(待ちなしの `while`)を書かない |
| 合図の読み取り | 1 つの判断のなかで、`Interrupt.Number` を 2 回読んで食い違いを起こさない。1 回読んで、その値を使う |
| 例外 | コールバックの中で、例外を送出しない(スレッドプールまたは背景スレッドでの未処理の例外は、プロセスを終了させる。確認: .NET の一般的な規則。この文書では出典を確認していない。未確認) |
| `Console` の入力 | `Console.In`、`Console.ReadKey` などを使わない(端末の設定を変えうる。未確認) |

### 9.2 順序の保証【規定】

| 保証 | 内容 |
|------|------|
| 出力の順序 | 標準出力・標準エラーへの書き込みは、`Reporter`(`Conductor.Report`。[02-architecture.md](02-architecture.md))の 1 か所を通し、**書くたびに `Flush()` する**(`Stream.Write` のあと `Flush`)。標準出力と標準エラーの書き込みの順序が、呼び出しの順序と一致する。.NET の `Console.Out` と `Console.Error` は使わない(エンコーディングと自動の flush の設定に依存しないよう、`Console.OpenStandardOutput()` と `Console.OpenStandardError()` から作ったストリームに、BOM なしの UTF-8 でバイト列を書く。詳細は [08-output-and-i18n.md](08-output-and-i18n.md)) |
| 子の出力との順序 | `conductor` は、子を起動する前に、自分の出力を `Flush()` で書き切る(4.1 節の手順 5)。子の実行中に、リトライの知らせ以外の出力を書かない。子の出力(子が直接ファイル記述子へ書く)と、`conductor` の出力が、混ざらない。リトライの知らせは、子の終了のあと、次の起動の前に、書いて `Flush()` する |
| ステートの順序 | ステートの書き込み(一時ファイルへの書き込み、同期、名前の付け替え、必要ならディレクトリの同期)の完了を待ってから、次のジョブを起動する。「記録して(Durable)から次へ」を破らない([05-execution-flow.md](05-execution-flow.md)、[06-state-file.md](06-state-file.md)) |
| 出力とステートの順序 | 05 章の順序に従う。ジョブごとに、`-v` の結果行などの表示を書き(`Flush()` し)、そのあとステートを書く。順序を入れ替えない |
| ハンドラの副作用 | シグナルのハンドラは、出力もステートの書き込みもしない(9.1 節)。したがって、ハンドラが出力・ステートの順序を乱すことはない |
| 終了の前の flush | `Main` が終了コードを返す前に、標準出力と標準エラーを `Flush()` する。`Flush()` の失敗(`IOException`。閉じたパイプなど)は、出力の書き込みの失敗として扱う(08 章)。終了は、`Main` が `int` を返すことで行い、`Environment.Exit` を、通常の経路で呼ばない(内部の誤りの経路だけ。10 章) |

### 9.3 スレッドの所有者・終了条件の表【規定】

| スレッド/コールバック | 起こす者(所有者) | 数 | 読む | 書く | 終わる条件 |
|-----------------------|-------------------|-----|------|------|------------|
| メインスレッド | ランタイム(`Main` の呼び出し) | 1 | すべて(定義、ステート、出力、`Interrupt`) | すべての出力とステートの書き込み | `Main` が戻る |
| シグナルのコールバック(UNIX 系: SIGINT・SIGQUIT・SIGTERM は専用の背景スレッド、SIGHUP はスレッドプール) | ランタイム(`PosixSignalRegistration`。登録は `SignalWatcher`) | シグナルごとに、到着のたびに(同時に複数ありうる) | OS のシグナル | `Interrupt.Trigger` だけ(と `ctx.Cancel`) | コールバックが戻る。登録は、プロセスの終了まで有効(本番で `Stop` は呼ばない。テストでは `Stop`) |
| `Console.CancelKeyPress` のコールバック(Windows) | ランタイム(スレッドプール。登録は `SignalWatcher`) | 到着のたびに | OS の Ctrl+C | `Interrupt.Trigger` だけ(と `e.Cancel`) | コールバックが戻る。登録は、プロセスの終了まで有効 |
| 子プロセス | `JobRunner` | 実行中のジョブごとに 1(同時に最大 1) | 子の終了(`WaitForExit(int)`、`ExitCode`) | `JobRunner` の内側の状態だけ | `WaitForExit` が `true` を返す(または起動の失敗)まで。`RunJob` が戻る時点で、子は回収済みである |
| 待ち(`WaitForExit(int)`、`ManualResetEventSlim.Wait`) | `JobRunner` の待機ループと `RetryWaiter` | 同時に最大 1 | 時間の経過、`Interrupt` | なし | 経過、または合図の確認で打ち切る |
| ランタイム内部のスレッド(子の回収、スレッドプール) | .NET ランタイム | ランタイムが決める | `SIGCHLD` | ランタイムの内部の状態 | `conductor` は所有しない。直接使わない |

- 出力(標準出力・標準エラー)とステートへの書き込みは、流れの制御(`Conductor.Cli`)の 1 本の呼び出しの連なり(メインスレッド)だけが行う。シグナルのコールバックは、それらに触れない。
- リソースのリークがないこと: テストの終わりに、シグナルの登録が元に戻っている、`Process` が `Dispose` されている、子プロセスが残っていない。

---

## 10. 想定外の内部の誤り

- 実行の入口(`Program.Main`)で、想定外の内部の誤り(実装の不具合による例外)を捕まえる仕組みを、**最初に**入れる。内部の誤りが起きると、標準エラーに固定の英語の 1 行 `conductor: error: internal error`(ロケールを判定しない)を書き、終了コード **70** で終わる。標準エラーへの書き込みが失敗しても 70 で終わる。【規定】
- 仕組みは次の 3 つ(同じ関数 `FailInternal()` を呼ぶ。【.NET の設計】):
  1. `Main` の内側の `try { ... } catch (Exception) { return FailInternal(); }`(`Main` を `try` で囲み、すべての実装の不具合を捕まえる)。
  2. `AppDomain.CurrentDomain.UnhandledException` のハンドラ(コールバックのスレッドなど、`Main` の外で起きた未処理の例外のため)。ハンドラの中で、`FailInternal()` を呼び、**`Environment.Exit(70)`** で終わる(ハンドラから戻ると、ランタイムがプロセスを異常終了させ、終了コードが 70 にならないため。未確認)。
  3. `TaskScheduler.UnobservedTaskException`(この仕様は `Task` を使わないため、通常は起こらない)。呼ばれたら、`FailInternal()` を呼び、`e.SetObserved()` のあと、`Environment.Exit(70)` で終わる。
- `FailInternal()` は、繰り返し呼ばれても 1 行だけ出す(`Interlocked` の印を持つ)。書き込みは、`Console.OpenStandardError()` のストリームに、固定のバイト列(`conductor: error: internal error\n` の UTF-8)を 1 回書く(`try { ... } catch (IOException) { }` で囲み、失敗は無視する。`Console.Error` を使わない。エンコーディングと flush の設定に左右されないため)。例外の内容(メッセージ、スタックトレース)を、標準エラーにも標準出力にも出さない(`ToString()` を呼ばない)。
- `StackOverflowException` は、捕まえられない(プロセスが終了する)。したがって、深い入れ子の入力(定義・ステート・変換)を、再帰で処理しない(明示的なスタックを使う。03 章)。`command` の分割は、再帰を使わない(3 章)。`OutOfMemoryException` は、捕まえて続行しない(`Main` の `catch (Exception)` が捕まえた場合は 70 にする。回復を試みない)。
- 入口は、流れの結果の終了コードで、プロセスを 1 回だけ終了させる(`Main` が `int` を返す)。入口に、わざと例外を起こすコマンドを持たない。内部の誤りの検査は、`Run` に差し替えた(例外を送出する)`Deps` を渡す単体テスト、または別のテスト用の実行ファイル(`tests/Conductor.TestJob/`)で行う([10-errors-and-exit-codes.md](10-errors-and-exit-codes.md))。
- ジョブの実行中に内部の誤りが起きると、`Process` の `Dispose` が行われても、子プロセスの終了を待たずに終わり、ジョブが孤児として残りうる(`Process.Dispose` は子を終了させない。確認: `Process.WaitForExit` の API 文書の「`Close` は、リソースを解放する」という記述。子を停止するとは書かれていない)。ステートは直前の完全な内容のままで、次の実行はそのジョブを再実行しうる(少なくとも 1 回の実行の範囲)。
- `conductor` が SIGKILL などで強制終了されたときも、ジョブは残って動き続けうる(検知も停止もしない)。

---

## 11. 検証の観点

この文書の挙動を確かめる受け入れテストは、[11-quality-and-build.md](11-quality-and-build.md) にある。

| 確かめること | 11 章の節 |
|--------------|-----------|
| 分割の全規則(3 章の表)、UTF-16 のサロゲートの組、U+3000 などが区切りでないこと | 「`command` の分割と起動」(受理・拒否の表) |
| メタ文字が引数にそのまま渡る、配列の要素がそのまま渡る(空の引数を含む)、環境変数と作業ディレクトリの引き継ぎ、起動失敗(見つからない・実行権限なし・ディレクトリ)、終了コード(0・3・255・300) | 同上 |
| UNIX 系で、`conductor` と同じディレクトリやカレントディレクトリの同名ファイルが、`PATH` より先に起動されないこと(4.9 節) | 同上(UNIX 系) |
| 起動前の合図で起動しない、終了の観測の直後の合図で `Interrupted` | 「シグナルと中断」(起動の部品の単体検査) |
| SIGTERM / SIGINT の転送、許可外の番号は送らない、転送は 1 回だけ、先に終わったジョブは転送なし | 「シグナルと中断」(UNIX 系) |
| 「新しいプロセスグループ」の試験は、この版では行わない(4.7 節) | 「シグナルと中断」に注記 |
| 待ちの間隔(1・2・4・8・10 ms)、リトライの待機の 1 秒以内の打ち切り | 「シグナルと中断」「リトライ」「非機能要件」(性能) |
| 実バイナリで SIGTERM → 143、ステートに再開位置、孤児なし。HUP のあとの INT → 129 | 「シグナルと中断」 |
| `kill -9 $$` のジョブ: `terminated by signal 9`、ステートに終了コードなし(4.4 節の推定の確認) | 「シグナルと中断」 |
| 監視の登録(最初の番号が残る、2 回目の登録は「すでに登録済み」、`Stop` で元に戻る) | 「シグナルと中断」(単体検査) |
| 内部の誤り → 70(`UnhandledException` の経路を含む) | 「内部の誤り」 |
| Windows の Ctrl+C | 「シグナルと中断」の手動の項目 |
| Windows の `.cmd` への危険な引数が注入にならない | 「`command` の分割と起動」(Windows 専用) |

---

## 12. 規定と実装の裁量の区別(まとめ)

| 項目 | 区別 |
|------|------|
| 分割の状態機械と、受理・拒否の例 | 【規定】 |
| シェルを介さない起動(`UseShellExecute = false`)、作業ディレクトリ・環境変数・標準入出力の引き継ぎ(`Redirect*` を `false`)、`ArgumentList` の使用 | 【規定】 |
| 起動失敗の終了コード 127、分類の優先順位 | 【規定】 |
| 転送は 1 回だけ、許可する 4 つのシグナルだけ、子の pid だけ、終了済みには送らない | 【規定】 |
| UNIX 系で、`PosixSignalRegistration` を静的に保持する、`Cancel = true`、`PosixSignal` の負の値を実際の番号に直す | 【規定】 |
| Windows で転送をしない(OS が Ctrl+C を配る) | 【規定】 |
| `run`/`pipeline` だけが、定義の検査より前にシグナルを準備する | 【規定】 |
| シグナルの準備の失敗は 70、内部の誤りは 70 | 【規定】 |
| リトライの待機が合図から 1 秒以内に返る | 【規定】 |
| ジョブを新しいプロセスグループに置かない、`ExitCode` からのシグナル終了の推定(4.4 節)、UNIX 系の実行ファイルの自前の解決 | 【.NET の設計】(この版の挙動) |
| 待機ループの確認の間隔(1・2・4・8・10 ms) | 【目安】 |
| リトライの待機の確認の間隔の上限(250 ms) | 【目安】 |
| `Interrupt` の内部の表現(1 つの `long` と `ManualResetEventSlim`) | 【任意】(原子的で、番号が一緒に見えること) |
| ほぼ同時に届いた複数のシグナルのどれが「最初」か | 規定しない |

---

## 13. 未確認の事項

- この環境には .NET SDK がなく、この文書の挙動を、実機で動かして確かめていない。API の名前、例外、既定値は、公式ドキュメントと、`dotnet/runtime` の `main` ブランチの公開ソースの読み取りに基づく。ソースだけで確かめたもの(下記)は、.NET 10 のリリース版と同じとは限らない。
- `Process.ExitCode` が、UNIX 系でシグナル終了のとき `128 + 番号` になること(`main` ブランチの `pal_process.c` で確認。API 文書には書かれていない。.NET 10 で同じか未確認)。シグナル終了と通常の `exit(128 + n)` の区別がつかないため、4.4 節の推定を採った。
- `Process.Start` の UNIX 系での、実行ファイルの探索の順序(`conductor` のディレクトリ、カレントディレクトリ、`PATH`)は、`main` ブランチのソースで確認した。API 文書には書かれていない。.NET 10 で同じか未確認(4.9 節は、これに依存せず、絶対パスを渡す)。
- `PosixSignalRegistration` のハンドラのスレッド(SIGINT・SIGQUIT・SIGTERM は専用の背景スレッド、それ以外はスレッドプール)は、`main` ブランチのソースで確認した。API 文書には書かれていない。
- `libc` の `kill` の P/Invoke で、ライブラリ名 `"libc"` と、`DllImportResolver` の名前の組(Linux の `libc.so.6`、macOS の `/usr/lib/libSystem.B.dylib`)が、3 OS・各アーキテクチャで解決されること。単一ファイルの発行での動作。
- UNIX 系で、`Process.Start` の子が、`conductor` のプロセスの `SIGPIPE` の扱い(ランタイムが無視にしている可能性)を引き継ぐかどうか(ジョブが、閉じたパイプへ書くときの挙動に影響しうる)。
- UNIX 系で、ジョブを `conductor` と同じプロセスグループに置くことの影響(端末の Ctrl+C が 2 回届くこと)は、実機で確かめていない。
- `WorkingDirectory` が空のとき、親の作業ディレクトリを引き継ぐこと。`ProcessStartInfo.Environment` の初期化の時点。
- Windows で、`ArgumentList` が空文字列の引数を `""` として渡すこと、`Arguments` が、そのまま `CreateProcess` に渡されること、`.cmd` の `cmd.exe` 経由の起動(4.8 節)、Ctrl+C が、同じコンソールのジョブにも届くこと、`ControlBreak` のとき `Cancel = false` で、プロセスが終了すること、Windows のタイマーの分解能(`WaitForExit` の短い待ち)、`Console.CancelKeyPress` の登録が `Console` の初期化を引き起こすか。
- Windows でのプログラム探索(カレントディレクトリを含むか等)の詳細。
- `PosixSignalRegistration` の Windows での、OS のイベントとシグナルの対応(この仕様は Windows で使わないため、影響しない)。
- UNIX 系で、ジョブが端末から読む(標準入力が端末のとき)場合の挙動(同じプロセスグループのため、背景グループとして停止する問題は、起こらない見込み。実機では未確認)。
- `string.Trim()` が除く空白の集合と、Unicode の White_Space の差(2 章。03 章の規定で、`Trim` を使わない)。
- ほぼ同時に届いた複数のシグナルの再現そのもの。
- 内部の誤りの経路で、`UnhandledException` のハンドラの中から `Environment.Exit(70)` を呼んだときに、終了コードが 70 になること(10 章)。
- `File.GetUnixFileMode` の使用(4.9 節。.NET 7 以降の API。出典を確認していない)。
- 他の担当と合わせる点: [02-architecture.md](02-architecture.md) の型の骨格(`Job`、`Command`、`JobExecution`、`Interrupt`、`SignalStart`、`IPlatform`)、[05-execution-flow.md](05-execution-flow.md) の分類(4.6 節)、[10-errors-and-exit-codes.md](10-errors-and-exit-codes.md) の終了コードと `FailInternal`、[11-quality-and-build.md](11-quality-and-build.md) の受け入れテスト(「新しいプロセスグループ」の試験を行わないこと、シグナル終了の推定、実行ファイルの解決の試験)。
