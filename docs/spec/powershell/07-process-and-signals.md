# 07 プロセス起動とシグナル(`command` の分割・起動・待ち・中断)

対象: `conductor` v0.8.0 と同じ挙動を PowerShell 7(`pwsh`)で実装するための仕様。この文書は、次を定める。

- `command` の文字列を、コマンド名と引数に分ける規則(3 章)
- ジョブ(子プロセス)の起動、待ち、結果の取得、起動失敗(4・5 章。`System.Diagnostics.Process`)
- 中断の合図と、OS ごと(UNIX 系と Windows)のシグナル処理(6・7 章)。PowerShell のスクリプトブロックを別スレッドのコールバックにできない問題の対処として、小さな C# のヘルパー(`Add-Type`)を使う設計を含む
- リトライの待機(8 章)、OS ごとの差(9 章)、スレッドと順序(10 章)、想定外の内部の誤り(11 章)

構成は [02-architecture.md](02-architecture.md)、定義ファイルの検査は [03-definition-format.md](03-definition-format.md)、流れ全体は [05-execution-flow.md](05-execution-flow.md)、ステートは [06-state-file.md](06-state-file.md)、表示の文言は [08-output-and-i18n.md](08-output-and-i18n.md)、終了コードの全件は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md)、受け入れテストは [11-quality-and-build.md](11-quality-and-build.md) にある。この文書は、必要な箇所だけ触れる。

## 規定の強さと確認の表記

| 表記 | 意味 |
|------|------|
| 【規定】 | どの実装も満たす。受け入れテストまたは単体テストで確かめる |
| 【目安】 | 推奨の値。利用者から見える結果を変えない範囲で、変えてよい |
| 【任意】 | 実装に任せる。利用者から見える結果に影響しない |
| 【PowerShell の設計】 | PowerShell で実装するために、この仕様が決めたこと(他の決め方もありうる) |
| 確認(出典: …) | 公式ドキュメント(learn.microsoft.com)、PowerShell の公開ソース(github.com/PowerShell/PowerShell の `master`)、または、記事・回答を、この文書の作成時に読んで確かめた。出典の種類を括弧に書く。公式の文書でなく、記事・回答・ソースだけで確かめたものは、そのことを書く(版によって変わりうる) |
| 未確認 | 公式ドキュメントでも公開ソースでも確かめられていない。この環境には `pwsh` も .NET SDK もなく、実機で動かして確かめていない。12 章の試験で、実装時に最初に確かめる |

対象の PowerShell は 7.x(`pwsh`)。Windows PowerShell 5.1 は対象外(`PosixSignalRegistration` と `ProcessStartInfo.ArgumentList` の前提のため。`ArgumentList` は .NET Core 2.1 以降。確認: `ProcessStartInfo.ArgumentList` の API 文書)。この文書の .NET の API 名は、`pwsh` 7.4 以降が載せる .NET(7.4 は .NET 8。他の版の対応は実装時に確認)の範囲で書く。最小の対象版は [02-architecture.md](02-architecture.md) の `PowerShellVersion` で決める。

---

## 1. 構成要素と責務

関数は `Private/Runner/` に置く(`Verb-ConductorNoun` の命名。[02-architecture.md](02-architecture.md))。次の責務が、別々の関数として(単独で検査できる形で)ある。戻り値は、判別用の `Kind` または `Ok` を持つ `[pscustomobject]`(`class` は使わない。理由は [02-architecture.md](02-architecture.md) の 7 章)。

| 構成要素 | 関数・型(【PowerShell の設計】) | 責務 |
|----------|---------------------------------|------|
| 分割器 | `Split-ConductorCommandLine` | `command` の文字列を、コマンド名と引数に分ける。文字を 1 つずつ左から読む状態機械で、再帰を使わない。3 OS で同じ規則 |
| `command` の検査 | `Test-ConductorCommand` | `command`(文字列形式または配列形式)を起動せずに検査する。成功なら分けた結果(`Program` と `Args`)を返す |
| ジョブの起動と待ち | `Invoke-ConductorJob` | 1 つのジョブを、シェルを介さずに起動し、終了を待ち、「実行結果」(4.5 節)を返す。失敗は値で返し、例外を送出しない(想定外の内部の誤りを除く) |
| 待機ループ | `Wait-ConductorChild`(`Invoke-ConductorJob` が呼ぶ) | `Process.WaitForExit(int)` の繰り返しで、ジョブの終了を確かめ続ける。シグナルの転送の判断もここで行う(5 章) |
| リトライの待機 | `Wait-ConductorRetry` | リトライの前に待つ。待機中に中断の合図を確かめる(8 章) |
| 中断の合図 | C# ヘルパーの `ConductorNative.Interrupt`(6 章)。作る関数は `New-ConductorInterrupt` | 「中断が要求された」ことと、最初に受け取ったシグナルの番号を、スレッドの間で共有する値 |
| プラットフォーム層 | `Get-ConductorPlatformKind`(OS の判定はここだけ)、`Get-ConductorStartInfo`(起動の準備)、`Resolve-ConductorUnixExecutable`、`ConvertTo-ConductorCmdArguments`、`Send-ConductorSignal`、`Read-ConductorExit`、`Sync-ConductorDirectory` | OS ごとの差(4.7〜4.9 節、7 章、9 章)。OS ごとに分岐が違う |
| C# ヘルパー | `Initialize-ConductorNativeHelper`(`Add-Type` を呼ぶのはここだけ) | ヘルパーの型を、最初に必要になったときに 1 回だけ読み込む(7.1〜7.3 節) |
| シグナル監視 | `Start-ConductorSignalWatch` | OS のシグナル(UNIX 系)/ Ctrl+C(Windows)を受け取り、中断の合図を立てる登録を行う(7 章)。受け取りの本体は C# ヘルパーの中で完結する |
| 流れの制御 | `Private/Cli/` の関数(`Invoke-ConductorRunCommand` ほか) | 合図の準備、ジョブの起動の呼び出し、中断時のステートの記録、終了コード(128 + 番号) |
| 分類 | `Private/Flow/` の `Get-ConductorClassification`(純粋) | 実行結果を、成功・警告・エラー・分類なしと、報告する終了コードに変える(4.6 節) |

OS の分岐と OS 固有の呼び出し(`[System.OperatingSystem]` の判定、`kill`、`.cmd` の起動の形)は、プラットフォーム層(`Private/Runner/` の上の関数群と C# ヘルパー)の外に置かない(9.2 節)【規定】。

---

## 2. `command` の形式(定義の側の受理条件)

定義ファイルの各ジョブの `command` は、次の 2 つの形式のどちらか(`.psd1` と JSON のどちらでも、論理モデルは同じ)。

| 形式 | 書き方(`.psd1` の例) | 表す値(【PowerShell の設計】) | 起動のときの扱い |
|------|------------------------|-------------------------------|------------------|
| 文字列形式 | `command = 'sh scripts/x.sh'` | `Kind = 'Line'`、`Line`(`[string]`) | 3 章の規則で、コマンド名と引数に分ける |
| 配列形式 | `command = @('sh', '-c', 'echo hi')` | `Kind = 'Argv'`、`Argv`(`[string[]]`) | 要素をそのまま使う(分割・引用符の処理・展開をしない) |

定義の読み込み時の検査(誤りは定義の誤り、終了コード 64。検査の実装は [03-definition-format.md](03-definition-format.md)):

| 入力 | 判定 |
|------|------|
| `command` キーがない | 誤り(必須の値がない) |
| 文字列で、前後の空白(Unicode の空白)を除くと空 | 誤り(`command` が空) |
| 配列が空(`@()`) | 誤り(配列が空) |
| 配列に文字列でない要素がある | 誤り(要素は文字列で書く)。要素ごとに報告する |
| 配列の先頭の要素が、前後の空白を除くと空 | 誤り(先頭の要素が空) |
| 配列の 2 番目以降が空文字列 | 有効(空の引数として渡す) |
| 文字列でも配列でもない(数値・ハッシュテーブルなど) | 誤り(型) |

「前後の空白を除く」は、[03-definition-format.md](03-definition-format.md) の規定(Unicode の White_Space を明示した自前の表)で行う。`[string]::Trim()`、`[char]::IsWhiteSpace`、`-split '\s'`、`-match '\s'` を使わない。この文書の 3 章の区切り(4 文字だけ)とは別の規則である。

定義の検査とは別に、実行の前にもう一度、全ジョブの `command` を 3.8 節のとおり検査する。

---

## 3. `command` の分割規則

### 3.1 入力と出力

- 入力: `command` の文字列(.NET の `[string]`。UTF-16)。
- 出力: コマンド名(`Program`。空でない)と、引数の並び(`Args`)。`Args` の各要素が、そのまま 1 つの引数になる。
- シェルは使わない。変数展開(`$HOME`)、グロブ(`*`)、パイプ(`|`)、`;`、`&&`、リダイレクト(`>`)、バッククォート、`~`、括弧は **解釈しない**。普通の文字として引数に入る。パイプなどが要るときは、利用者が `sh -c '...'` を `command` に書く。
- PowerShell の構文(`$()`、`@()`、`&`、`` ` ``)も解釈しない。`Invoke-Expression` を使わない【規定】。

### 3.2 文法

文字は、左から右へ 1 回だけ読む。区切り文字は次の 4 つだけ。

```text
separator := ' ' (U+0020) | '\t' (U+0009) | '\n' (U+000A) | '\r' (U+000D)
```

(U+000B、U+000C、U+00A0、U+3000 などのほかの空白は区切りではなく、普通の文字。したがって `[char]::IsWhiteSpace`、`[string]::Split()`(区切りなし)、`[string]::Trim()`、`-split`(引数が正規表現。既定で `\s+`)、`\s` を使ってはならない。これらは、上の 4 文字以外も空白として扱う。)

状態は 4 つ。これに加えて、`inWord`(語の途中か)と `current`(組み立て中の語。`[System.Text.StringBuilder]`)を持つ。

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
- PowerShell の引用規則(二重引用符の中の `` ` `` がエスケープ、`""` が `"`、一重引用符の中の `''` が `'`)とは**別の規則**である。PowerShell の引用規則を持ち込まない。
- 引用符の外のバックスラッシュは普通の文字(Windows のパス `C:\dir\x.exe` がそのまま書ける)。一重引用符の中も同じ。
- 末尾がバックスラッシュのパスを二重引用符で囲むと `\"` が引用符のエスケープになり、閉じない(誤り)。一重引用符で書く。
- 文字列の長さに上限はなく、入れ子もないため、再帰やスタックの深さに依存しない。語の列は `[System.Collections.Generic.List[string]]` に溜める【規定】。配列への `+=` で語を足さない(そのたびに配列が作り直され、引数が約 30 万個の入力で現実的な時間に終わらない)【規定】。性能の目安(約 30 万引数の入力を数秒以内で分割する)は、実装時に計測して決める(未確認。PowerShell のループは、コンパイル言語より遅い。11 章の性能の項)。

### 3.3 UTF-16 のコードユニットと文字の走査【PowerShell の設計】

PowerShell の `[string]` は .NET の `string`(UTF-16)である。BMP 外の文字(例: U+1F600)は、2 つの `[char]`(上位サロゲート U+D800〜U+DBFF と、下位サロゲート U+DC00〜U+DFFF の組)で表される。`$line[$i]` は 1 つの `[char]`(コードユニット)を返す(確認: .NET の `string` のインデクサーの規則。PowerShell のインデックス演算子 `[]` が文字列に対して `[char]` を返すことは、PowerShell の既知の挙動で、この文書では個別の出典を確認していない)。

この文法の特別な文字(区切り 4 文字、`'`、`"`、`\`)は、すべて ASCII であり、1 つの `[char]` で表される。サロゲートの `[char]`(U+D800〜U+DFFF)は、これらのどれとも等しくない。したがって、`[char]` ごと(コードユニットごと)に走査しても、コードポイントごとに走査しても、結果は同じになる。サロゲートの組は「上記以外」として、2 つの `[char]` が順に `current` に足され、元の並びのまま残る。

**この仕様の採用: `[char]` ごと(コードユニットごと)に走査する**【規定: 結果が同じであれば、どちらでもよい】。理由は次のとおり。(1) 結果が同じ。(2) 対のないサロゲートを含む入力を、そのまま残せる(`[System.Text.Rune]` の列挙は、対のないサロゲートを U+FFFD に置き換えうる。確認: .NET の `Rune` の API 文書の存在。置き換えの詳細は未確認)。(3) `Rune` の生成のコストがない。

書記素(見た目の 1 文字。結合文字や絵文字の連なり)は、分割に関係しない。`current` に足すとき、サロゲートの組や結合文字の並びを、途中で切らない。表示の幅や文字数の数え方は、[08-output-and-i18n.md](08-output-and-i18n.md) の規定による(この章では使わない)。

走査の PowerShell 固有の規定【規定】:

| 項目 | 規定 |
|------|------|
| 走査の形 | `for ($i = 0; $i -lt $line.Length; $i++)`。`foreach ($c in $line.ToCharArray())` でもよい。`$line -split ''`、`[regex]::Matches` を使わない |
| 文字の比較 | `[char]` どうしの `-eq` は、大文字小文字の扱いが文書で確かめられていない(未確認)。**整数の符号位置で比べる**: `$u = [int]$line[$i]` のあと、`$u -eq 32`(空白)、`9`(タブ)、`10`(LF)、`13`(CR)、`39`(`'`)、`34`(`"`)、`92`(`\`)。`-eq` は、右辺の型に左辺を変換する(PowerShell の既知の挙動)ので、両辺とも `[int]` にそろえる |
| 状態の持ち方 | `$state` は `[int]`(0 = Plain、1 = Single、2 = Double、3 = DoubleBackslash)。`switch` を使わず、`if`/`elseif` で書く(`switch` は既定で大文字小文字を区別せず、複数の分岐に一致し、配列を展開する。`switch` のこれらの挙動に頼らない) |
| 戻り値の中の配列 | 戻り値は `[pscustomobject]`(`Ok`、`Program`、`Args`、`ErrorKind`、`Message`)。`Args` は `[string[]]` で、プロパティに入れて返す(関数の出力に配列を直接流すと、パイプラインが要素ごとに展開し、1 要素の配列が単一の文字列になる、0 要素が `$null` になる)。`Args` が 0 要素でも `[string[]]@()` のまま保つ |
| 出力の混入 | `StringBuilder.Append(...)` は `StringBuilder` を返し、`List.Add` は返さないが、`HashSet.Add` などは `bool` を返す。値を返すメソッドの呼び出しは、`[void]` または `$null =` で捨てる(捨てないと、関数の戻り値に混ざる) |

### 3.4 関数の骨格(実装コードではない)

```powershell
# Private/Runner/Split-ConductorCommandLine.ps1
# 戻り値(pscustomobject):
#   Ok = $true  : Program ([string])、Args ([string[]])
#   Ok = $false : ErrorKind ('UnclosedQuote' | 'Empty')、Message (固定の文。入力の値を含めない)
function Split-ConductorCommandLine {
    param([string]$Line)
}

# command(文字列形式 Kind='Line' または配列形式 Kind='Argv')の検査。起動しない。戻り値は上と同じ形。
function Test-ConductorCommand {
    param([System.Collections.IDictionary]$Command)
}
```

擬似コード(同じ結果になる実装であれば、書き方は自由):

```text
words = new List[string]; current = new StringBuilder; inWord = false; state = 0 (Plain)
for i in 0 .. line.Length-1:
  u = [int]line[i]
  if state == Plain:
      if u in {32, 9, 10, 13}: if inWord: words.Add(current.ToString()); current.Clear(); inWord = false
      elif u == 39: inWord = true; state = Single
      elif u == 34: inWord = true; state = Double
      else:         inWord = true; current.Append(line[i])
  elif state == Single:
      if u == 39: state = Plain else current.Append(line[i])
  elif state == Double:
      if u == 34: state = Plain
      elif u == 92: state = DoubleBackslash
      else: current.Append(line[i])
  else:  # DoubleBackslash
      if u == 34: current.Append('"'); state = Double
      elif u == 92: current.Append('\')          # state は DoubleBackslash のまま
      else: current.Append('\').Append(line[i]); state = Double
if state != Plain: return Err(UnclosedQuote)
if inWord: words.Add(current.ToString())
if words.Count == 0 or words[0].Length == 0: return Err(Empty)
return Ok(words[0], words.GetRange(1, words.Count-1).ToArray())
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
| `echo $x "$y" @(1)`(PowerShell の構文は解釈しない) | `echo` | `$x`, `$y`, `@(1)` |
| `echo "a`` b"`(`"` の中の `` ` `` は普通の文字) | `echo` | ``a`` b`` |
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

- 要素を一切加工せずに使う。先頭が `Program`、残りが `Args`。分割、引用符の解釈、展開、前後の空白の除去をしない(`'  lead and trail  '` も、そのまま 1 つの引数)。
- 先頭の要素が、前後の空白を除いて空(空文字列・空白だけ)、または要素が 0 個のときは `Empty`(定義の検査の二重の備え)。先頭の要素が空白を含むが空でないとき(`'my prog'`)は、そのまま `Program`(`my prog` というファイル名を起動する)。
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

`Start-Process` と `&`(呼び出し演算子)、`Invoke-Expression`、`cmd /c` を使わない【規定】。理由: 標準入出力の引き継ぎ、終了コード(`$LASTEXITCODE` は `&` の外部コマンドでだけ設定され、`Start-Process` は `-PassThru` を付けないと終了コードを返さない)、シグナルの転送、引数の厳密な受け渡しを、制御するため。

### 4.1 手順(時系列)

`Invoke-ConductorJob -Job $job -Interrupt $sig -Platform $kind -LookupEnv $fn`(ジョブ → 実行結果)。失敗はすべて戻り値で返し、例外を送出しない。

1. 実行結果を初期化する: `Started = $true`、`ExitCode = $null`、`TerminatedBySignal = $null`、`StartFailure = $false`、`Interrupted = $false`、`SignalNumber = $null`。
2. `command` を分割する(文字列は 3 章、配列は 3.7 節)。失敗なら `StartFailure = $true` で返す。
3. **起動の前**に合図が立っていれば、起動せずに返す: `Started = $false`、`Interrupted = $true`、`SignalNumber = 番号`。(ジョブは実行されない。)
4. `[System.Diagnostics.ProcessStartInfo]` を組み立てる(4.2 節)。プラットフォーム層の準備を適用する(UNIX 系: 実行ファイルの絶対パスへの解決。Windows: `.cmd` / `.bat` の起動の形。4.7・4.8 節)。解決に失敗したら `StartFailure = $true` で返す(4.3 節)。
5. 自分の出力を書き切ってから(flush 済みであること。10.2 節)、`[System.Diagnostics.Process]::Start($startInfo)` で起動する。例外(4.3 節の表)なら `StartFailure = $true` で返す(`Started` は `$true` のまま)。戻り値が `$null` のときも、`StartFailure = $true` で返す。
6. 5 章の待機ループで終了を待つ。時間切れ(タイムアウト)はない。自動のリトライもしない(リトライは流れの制御の層で行う。8 章)。
7. 終了を観測したら(4.4 節):
   - `ExitCode` と `TerminatedBySignal` を、4.4 節の規則で決める。
   - **この時点で** 合図が立っていれば `Interrupted = $true`、`SignalNumber = 番号`。
8. 終了の状態を取得できないとき(`ExitCode` の取得が例外)は、`StartFailure = $true` で返す(分類で 127)。
9. `Process` を `Dispose()` する(`finally` で)。起動後にハンドルを残さない。

### 4.2 起動の条件(`ProcessStartInfo`)

| 項目 | 規定 |
|------|------|
| シェル | 使わない。`UseShellExecute = $false` を必ず指定する【規定】。`sh -c` や `cmd /C` を挟まない(ただし、Windows の `.cmd` / `.bat` の起動だけは、4.8 節の決定により `cmd.exe` を直接起動する)。「シェル」は、`UseShellExecute` の文脈では、グラフィカルなシェル(関連づけによるファイルの起動)を指す(確認。出典: `ProcessStartInfo.UseShellExecute` の API 文書。既定は .NET では `false`)。既定に頼らず、明示する |
| 実行ファイル | `FileName`。UNIX 系では、4.9 節の決定により、絶対パスに解決して渡す。Windows では、`Program` をそのまま渡す(OS の標準の探索に任せる。4.8 節) |
| 引数 | **`ArgumentList` だけ**を使い、`Arguments` を使わない【規定】(例外: Windows の `.cmd` / `.bat`。4.8 節)。`ArgumentList` は `Collection[string]` で、`$startInfo.ArgumentList.Add($a)` で足す(戻り値が `void`)。追加した文字列は、前もってエスケープしなくてよい形で受け取られる(確認。出典: `ProcessStartInfo.ArgumentList` の API 文書)。UNIX 系では、`ArgumentList` の各要素が、そのまま `argv` の要素になる(確認。出典: .NET ランタイムのソース `Process.Unix.cs`。公式の API 文書には書かれていない)。メタ文字は注入にならず、空文字列の引数も保たれる(UNIX 系。Windows で空文字列の引数が `""` として渡されることは未確認)。**PowerShell 固有**: `ArgumentList` に `[string[]]` を一度に代入しない(読み取り専用のプロパティ)。`foreach` で 1 つずつ `Add` する。`$startInfo.ArgumentList.Add($a)` の `$a` は `[string]` に確定してから渡す(`$null` を渡さない) |
| 作業ディレクトリ | 引き継ぐ。`WorkingDirectory` を設定しない(既定の空文字列のまま)【規定】。`UseShellExecute = false` のとき、`WorkingDirectory` は、実行ファイルの探索には使われず、起動した側の作業ディレクトリになる(確認。出典: `ProcessStartInfo.UseShellExecute` の API 文書)。空のとき、親の作業ディレクトリを引き継ぐ(未確認。公式の文書に明記がない。受け入れテストで確かめる)。**PowerShell 固有**: プロセスの作業ディレクトリ(`[System.IO.Directory]::GetCurrentDirectory()`)と、PowerShell の現在の場所(`$PWD`、`Get-Location`)は別の値である(PowerShell は、`Set-Location` で前者を更新しない場合がある。確認: PowerShell の既知の挙動。この文書では出典を確認していない)。したがって、製品のコードで `Set-Location`、`Push-Location`、`cd` を呼ばない【規定】。相対パスを .NET の API に渡すときは、`[System.IO.Directory]::GetCurrentDirectory()`(起動したときの作業ディレクトリ)を基準に、自前で絶対パスにする(`Resolve-Path` と `Join-Path` は、存在しないパスの扱いと PowerShell の場所の基準に依存するので、ファイルの操作に使わない) |
| 環境変数 | すべて引き継ぐ。`ProcessStartInfo.Environment` に、追加・削除・上書きをしない。run ID も環境変数としては渡さない【規定】。`$env:` への代入を、製品のコードで行わない。`pwsh` 自身が起動時に環境変数を足すことがある(例: `PSModulePath` の調整。確認: PowerShell の既知の挙動。この文書では出典を確認していない)ため、子の環境が、`conductor` の起動前の環境と完全に同じとは言えない(受け入れテストは、利用者が設定した変数だけを検査する。未確認) |
| 標準入力・標準出力・標準エラー | すべて引き継ぐ。`RedirectStandardInput`、`RedirectStandardOutput`、`RedirectStandardError` を、すべて `$false` にする(明示する)【規定】。`UseShellExecute = false` で、これらが `false` のとき、新しいプロセスは、呼び出し側の標準入出力を引き継ぐ(確認。出典: `ProcessStartInfo` の API 文書の Remarks)。ジョブの出力は、`conductor` を介さず、そのまま流れる。`conductor` は、ジョブの出力を読まない・保存しない・ステートに書かない【規定】。**PowerShell 固有**: `pwsh` が標準出力を `$OutputEncoding` などで加工することは、子が直接ファイル記述子へ書く出力には起こらない(子はパイプラインを通らない) |
| 窓 | `CreateNoWindow` を設定しない。`WindowStyle` を設定しない |
| 資格情報 | `UserName`、`Password`、`Domain`、`LoadUserProfile` を設定しない |
| 起動できなかった場合 | 4.3 節 |
| 時間の上限 | ない。ジョブが終わらなければ、`conductor` は待ち続ける |
| ジョブの強制終了 | しない(`Process.Kill` を呼ばない。`Stop-Process` を使わない) |
| 新しいプロセスグループ | UNIX 系: 作らない(4.7 節の決定)。Windows: 作らない(`CreateNewProcessGroup` を使わない) |

### 4.3 起動の失敗の検出

`Process.Start(ProcessStartInfo)` が送出する例外のうち、公式の文書に書かれているもの(確認。出典: `Process.Start(ProcessStartInfo)` の API 文書。以下の表は、他の言語の実装と共通の .NET の挙動):

| 例外(PowerShell の型名) | 文書上の原因 |
|---------------------------|--------------|
| `System.InvalidOperationException` | `FileName` が指定されていない。または、`UseShellExecute = true` で標準入出力を横取りしている(この仕様では起こらない) |
| `System.ComponentModel.Win32Exception` | ファイルを開くときの誤り、または、`FileName` のファイルが見つからない。引数と実行ファイルのパスを合わせた長さが上限を超えた |
| `System.ArgumentNullException` | `startInfo` が `$null`(起こらない) |
| `System.PlatformNotSupportedException` | シェルのない OS(この仕様の対象外) |
| (戻り値 `$null`) | プロセスのリソースが起動されなかった |

UNIX 系の実装では、`FileName` が見つからない場合は `ENOENT` を表す `Win32Exception`、ディレクトリの場合も `Win32Exception`、`fork`/`exec` の失敗は、子が `errno` を親に伝え、親が `Win32Exception`(`NativeErrorCode` に `errno`)にする(確認。出典: .NET ランタイムのソース `Process.Unix.cs` と `pal_process.c`。公式の API 文書ではない。版によって変わりうる)。

**規定**: 次の例外はすべて「起動の失敗」として捕まえ、`StartFailure = $true` にする。原因(見つからない・実行権限がない・実行できない形式・ディレクトリを指定した、など)と、`NativeErrorCode` の値を区別しない【規定】。例外のメッセージも、表示に使わない(`NativeErrorCode` と `Message` を、標準出力にも標準エラーにも出さない。入力の値や OS の文が混ざらないようにするため)。したがって、`Win32Exception.NativeErrorCode` から 127 への対応は「**すべて 127**」であり、`NativeErrorCode` を読む必要はない(読まない)。

```powershell
try   { $child = [System.Diagnostics.Process]::Start($startInfo) }
catch [System.ComponentModel.Win32Exception]    { <# 起動の失敗 #> }
catch [System.InvalidOperationException]        { <# 起動の失敗 #> }
catch [System.ArgumentException]                { <# 起動の失敗(念のための備え。型は未確認) #> }
catch [System.PlatformNotSupportedException]    { <# 起動の失敗 #> }
```

- 上の 4 つ以外の例外は捕まえない(想定外の内部の誤りとして、11 章の経路で終了コード 70 にする)。ここで `catch { }`(型なし)、`catch [System.Exception]` を書かない【規定】。
- **PowerShell 固有**: .NET のメソッドが送出した例外は、PowerShell では `MethodInvocationException` に包まれて届くことがある。`catch [System.ComponentModel.Win32Exception]` が、包まれた例外の内側の型に一致するかは、公式の文書で確かめていない(未確認)。そのため、実装は、次のどちらかで書く(12 章の試験 T-6 で、どちらが要るかを実装時に確かめる): (1) `catch [System.Management.Automation.MethodInvocationException]` で捕まえ、`$_.Exception.InnerException` の型を、`-is [System.ComponentModel.Win32Exception]` などで見る。(2) `catch [System.ComponentModel.Win32Exception]` と、(1) の両方を書く。いずれでも、内側が 4 つ以外の型なら、再び `throw` する(`throw` の引数なしで、同じ例外を再送出する)。
- `$ErrorActionPreference = 'Stop'`(モジュールの先頭。[02-architecture.md](02-architecture.md))のもとで、.NET のメソッドの例外は終了エラーとして `catch` に届く。.NET のメソッドの呼び出しは、`-ErrorAction` の影響を受けない。

起動の失敗のときの終了コードは 127(分類。4.6 節)。表示は `conductor: error: job '<ID>': cannot start command '<プログラム名>'`(英語。日本語は [08-output-and-i18n.md](08-output-and-i18n.md))。ジョブの終了コード・シグナルは `$null`。

### 4.4 終了コードの取り出し(`Process.ExitCode`)

`ExitCode` は `[int]` である。プロセスが終了する前に取得すると `InvalidOperationException`(確認。出典: `Process.ExitCode` の API 文書)。`WaitForExit(int)` が `$true` を返したあとで取得する(5 章)。

**UNIX 系**

- 通常の終了(`exit(n)`)では、`ExitCode` は `n` の下位 8 ビット(0〜255)。
- シグナルで終了したとき、`ExitCode` は `128 + シグナル番号` になる(確認。出典: .NET ランタイムのソース `pal_process.c` の `SystemNative_WaitPidExitedNoHang`: `WIFEXITED` なら `WEXITSTATUS`、`WIFSIGNALED` なら `128 + WTERMSIG`。公式の API 文書には書かれていない。`pwsh` が載せる .NET の版が同じかは未確認)。
- したがって、`ExitCode` だけでは、「シグナルで終了した」と「自分で `exit(137)` した」を区別できない。`waitpid` を自分で呼ぶことは、ランタイムの子の回収(`SIGCHLD` の処理)と競合しうるため、採らない。

**【PowerShell の設計】決定(UNIX 系)**: 終了を観測したあと、`$code = $child.ExitCode` について、次のとおり決める(`Read-ConductorExit`)。

```text
if ($code -ge 129 -and $code -le 159          # = 128 + 1 から 128 + 31
    かつ $code が job の成功コードにも警告コードにも含まれない):
    TerminatedBySignal = $code - 128
    ExitCode = $null
else:
    ExitCode = $code
    TerminatedBySignal = $null
```

- 129〜159 は、標準のシグナル 1〜31 に対応する(Linux、macOS で、この範囲は共通。リアルタイムシグナルは対象外)。
- 成功・警告の終了コードに含まれるときは、利用者が意図した終了コードとして、シグナルとはみなさない。
- 帰結【規定: この版の挙動】: (1) 自分で `exit(130)` で終わったジョブは、「シグナル 2 で終了」として扱う。ステートに終了コードは残らず、`-v` の補足は `terminated by signal 2` になる。報告する終了コードは、どちらでも `128 + n`(= `exit` した値)で同じ。(2) シグナルで終了したジョブは、リトライの対象の終了コード(`retry.exit_codes`)に `128 + n` が含まれていても、リトライしない。自分で `exit(128 + n)` したジョブは、同じ理由で、リトライの対象外になる。成功・警告の指定には影響しない。
- 値の比較は `[int]` どうしで行う(`$code -ge 129`)。`$code` を文字列に変換しない。

**Windows**

- `ExitCode` は、OS が報告する 32 ビット符号つき整数のまま(例外による終了は負の値になりうる)。`TerminatedBySignal` は常に `$null`。`ExitCode` を、そのまま `[int]` として保つ(`[uint32]`、`[byte]` に変換しない。PowerShell の `[byte]` へのキャストは、範囲外で例外になる)【規定】。報告する終了コードの 0〜255 への合わせ方(下位 8 ビット)は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md)。負の値の下位 8 ビットは、`$value -band 0xFF`(`-band` は `[int]` どうしのビット積で、負の値でも下位 8 ビットが得られる。`0xFF` は `[int]` の 255)で取る。

**PowerShell の `$LASTEXITCODE` と `$?` は使わない**【規定】: `Process.ExitCode` を直接読む。`$LASTEXITCODE` は、`&` で起動した外部コマンドの終了コードを反映する変数であり、`System.Diagnostics.Process` では設定されない(PowerShell の既知の挙動)。

### 4.5 実行結果(`JobExecution`)の各項目

`Private/Flow/` の `New-ConductorJobExecution` が作る(05 章が正)。型は使わず、次のプロパティを持つ `[pscustomobject]`:

```text
JobId              [string]
Started            [bool]    false は、起動の前に合図が立っていたときだけ
ExitCode           [int] または $null   UNIX 系でシグナル終了とみなしたときは $null。Windows は常に値がある
TerminatedBySignal [int] または $null   UNIX 系のシグナル終了とみなした番号。Windows は常に $null
StartFailure       [bool]
Interrupted        [bool]
SignalNumber       [int] または $null   Interrupted のとき、最初に受け取ったシグナルの番号
```

| 項目 | 意味 |
|------|------|
| `JobId`、(`command`) | 定義のジョブID(と `command`)。`command` は、表示で使うときに定義から引く |
| `Started` | 起動を試みたか。`$false` は、起動の前に合図が立っていたときだけ |
| `ExitCode` | 4.4 節の規則で決めた終了コード。UNIX 系の通常の終了は 0〜255、Windows は 32 ビット符号つきのまま |
| `TerminatedBySignal` | UNIX 系のシグナル終了の番号(4.4 節の規則による推定) |
| `StartFailure` | 分割の失敗、起動の失敗、終了の状態の取得の失敗のいずれか |
| `Interrupted` | 起動前、または、終了の観測の直後に、合図が立っていた |
| `SignalNumber` | `Interrupted` のとき、**最初に受け取った**シグナルの番号 |

「値がない」を `$null` で表すので、`$null` の判定は `$null -eq $x`(`$null` を左辺に置く)で行う。`if ($x)` で判定しない(`0` が偽になる)【規定】。

### 4.6 分類(終了コードの決定)

実行結果は、次の優先順位で、最初に当てはまるものを採る。

| 順 | 条件 | 分類 | 報告する終了コード |
|----|------|------|--------------------|
| 1 | `Started = $false` かつ `Interrupted` | 分類なし(エラーとして記録しない) | `128 + SignalNumber` |
| 2 | `Started = $false` で中断でもない(起こらない) | エラー | 127 |
| 3 | `Interrupted`(起動済み) | エラー | `128 + SignalNumber`(ジョブ自身の終了コードは見ない。ジョブが 0 で終わっていてもエラー) |
| 4 | `StartFailure` | エラー | 127 |
| 5 | `TerminatedBySignal = n` | エラー | `128 + n` |
| 6 | `ExitCode = $null`(起動して終わったのに値がない) | エラー(`conductor` 自身のエラー) | 1 |
| 7 | `ExitCode` が成功コード(既定は `[0]`、`success_exit_codes` で指定) | 成功 | 0 |
| 8 | `ExitCode` が `warning_exit_codes` | 警告 | 0 |
| 9 | `ExitCode = 0` で成功コードに含まれない | エラー(`conductor` 自身のエラー) | 1 |
| 10 | 上記以外 | エラー | `ExitCode` のまま |

`conductor` の終了コードは、報告する終了コードを 0〜255 に合わせたもの(範囲内はそのまま。範囲外は下位 8 ビット。エラーで下位 8 ビットが 0 になるときは 1)。詳細は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md)。

### 4.7 新しいプロセスグループ/セッションに置くか【PowerShell の設計】

**決定: UNIX 系でも、ジョブを新しいプロセスグループにも新しいセッションにも置かない。`conductor` と同じプロセスグループで起動する。**

理由: `System.Diagnostics.Process` には、UNIX 系で、子を新しいプロセスグループに置く公開の手段がない。`ProcessStartInfo.CreateNewProcessGroup` は Windows 専用で、新しいプロセスの Ctrl+C を無効にする(確認。出典: `ProcessStartInfo.CreateNewProcessGroup` の API 文書)。`ProcessStartInfo.StartDetached` は .NET 11 以降だけで、標準入出力を既定でヌルデバイスにつなぐ(確認。出典: `ProcessStartInfo.StartDetached` の API 文書。`pwsh` 7.4 の .NET 8 では使えない)。C# ヘルパーから `setpgid`/`setsid` を起動のあとに呼ぶと、子が `exec` を終えたあとは失敗し、起動との競合があって信頼できない。`posix_spawn` の `POSIX_SPAWN_SETPGROUP` を呼んで `Process` を使わずに起動する案は、ランタイムの子の回収(`SIGCHLD`)と自前の `waitpid` が競合しない保証がなく、構造体と定数の OS 差があり、環境変数と `PATH` の探索を自前で持つことになるため、採らない(未確認の点を含む。将来採るときは、この章と受け入れテストを改める)。

帰結(**他の言語の実装との差。受け入れテストの項目に影響する。12 章の末尾の「合わせる点」にも書く**):

- 端末の Ctrl+C(SIGINT)は、端末のフォアグラウンドのプロセスグループ全体に配られるため、ジョブにも**直接**届き、そのあと `conductor` の転送(7.5 節)でもう 1 回届く(ジョブは SIGINT を 2 回受けうる)。`kill -INT <conductor の pid>` のように、`conductor` の pid だけに送った場合は、転送の 1 回だけである。
- 「ジョブが新しいプロセスグループにある」ことを確かめる試験(プロセスグループ ID が子の pid)は、この版では行わない。
- ジョブは、`conductor` と同じ制御端末を持つ(標準入力が端末のジョブが、背景のグループとして停止する問題も起こらない)。

### 4.8 Windows の `.cmd` / `.bat` の起動【PowerShell の設計】

`.cmd` / `.bat` の起動の方法と、引数の安全な渡し方は、公式の API 文書に書かれていない。次のとおり決める。

**問題**: Windows では、`.cmd` / `.bat` は `cmd.exe` が解釈する。`ArgumentList` は、引数を、一般のプログラムの規則(`CommandLineToArgvW` の規則。確認: `ArgumentList` の API 文書は「エスケープする」とだけ書く。規則の詳細は未確認)で 1 本の文字列にする。この文字列を `cmd.exe` が再解釈するとき、`"`、`%`、`^`、`&`、`|`、`<`、`>` などが、別の意味を持ち、引数の中のコマンドが実行されうる。この問題に、.NET が対策しているかは、未確認である。

**決定(`cmd.exe` を直接起動する。`UseShellExecute = $false` のまま)**:

1. 次のとき、かつ、そのときだけ、この規則を使う: 実行中の OS が Windows であり、`Program` が、大文字小文字を区別せず `.cmd` または `.bat` で終わる(`$program.EndsWith('.cmd', [System.StringComparison]::OrdinalIgnoreCase)`。`-like`、`-match` を使わない)。拡張子のない `Program`(例: `build`)は、OS の標準の探索に任せる(`.cmd` は補われない。見つからなければ起動の失敗)。
2. 検査: `Args` のどれかが、次の文字を含むなら、**起動せず**、起動の失敗の実行結果を返す。`Program` は、`(` と `)` を除く同じ集合を検査する(パス `C:\Program Files (x86)\...` を許す)。

```text
"  %  ^  &  |  <  >  !  (  )  改行(\n)  復帰(\r)  NUL(U+0000)
```

3. 検査を通ったら、`FileName` に `cmd.exe` のパス、`Arguments`(`ArgumentList` ではない)に次の 1 本の文字列を設定する(`ConvertTo-ConductorCmdArguments`)。
   - `cmd.exe` のパス: 環境変数 `ComSpec` が、絶対パス(`[System.IO.Path]::IsPathRooted`)ならその値、そうでなければ `cmd.exe`(環境変数は、`Deps` の `Env.GetEnvironmentVariable` から読む。[02-architecture.md](02-architecture.md))。
   - `Arguments`: `/d /s /c "` + `<全体>` + `"`。`<全体>` は、`Program` と各 `Args` を、それぞれ `"` で囲み、空白 1 つで連結した文字列(空の引数は `""`)。例: `Program = C:\x\build.cmd`、`Args = [a b, c]` → `/d /s /c ""C:\x\build.cmd" "a b" "c""`。
   - `/d`(AutoRun のコマンドを実行しない)、`/s` と `/c`(最初と最後の `"` を取り除いて、残りを実行する)の意味は、Windows の `cmd` の文書による(この文書では未確認)。
   - `Arguments` が、OS に、そのまま渡されること(.NET が再び引用しないこと)は未確認。
4. 配列形式の `command` も、同じ検査を通る。
5. 検査の関数と、`Arguments` の組み立ては、どの OS でも動く純粋な関数にする(Linux の 1 台から、Windows の分岐を単体テストできる)。**PowerShell 固有**: 禁止の文字の検査は、`$text.IndexOfAny([char[]]'"%^&|<>!()'.ToCharArray()...)` のように、文字の集合に対する `IndexOfAny` か、1 文字ずつの符号位置の比較で行う。`-match '[...]'` を使わない(正規表現のメタ文字の誤りと、大文字小文字の既定の扱いを避けるため)。

この禁止の文字の集合と、`cmd.exe` の起動の形は、この仕様の設計判断であり、Windows の実機で確かめていない(未確認)。`cmd.exe` が `.bat` の実行中に Ctrl+C を受けたとき「バッチ ジョブを終了しますか (Y/N)?」と尋ねる場合の影響も未確認(手動の確認項目。11 章)。

Windows で、拡張子のない `Program` や、`.exe` の探索(アプリケーションのあるディレクトリ、カレントディレクトリを含むか、など)は、`CreateProcess` の標準の規則に依存する(未確認)。PowerShell の `Get-Command` の探索(`PATHEXT` を補う、カレントディレクトリを含まない)とは**別**であり、`Get-Command` で `Program` を解決しない(解決の結果が、`Process.Start` と食い違うため)。

### 4.9 UNIX 系の実行ファイルの解決【PowerShell の設計】

`Process.Start` は、`UseShellExecute = false` のとき、`FileName` が絶対パスでなければ、次の順で探す(確認。出典: .NET ランタイムのソース `Process.Unix.cs` の `ResolvePath`。公式の API 文書には、この順序は書かれていない): (1) 実行中のアプリケーションの実行ファイルのあるディレクトリ(`pwsh` の場合は、`pwsh` のあるディレクトリ)、(2) カレントディレクトリ、(3) `PATH`。(1) と (2) は、ファイルがあれば(実行権限を見ずに)採る。これは、「`program` に区切り文字がなければ、OS の標準の探索(`PATH` だけ)」という規則に反する(カレントディレクトリにある同名のファイルが、先に起動されてしまう)。区切り文字を含む相対パス(`scripts/x.sh`)も、(1) が先に試される。

**決定(UNIX 系)**【規定】: `Resolve-ConductorUnixExecutable` が、`Process.Start` を呼ぶ前に、`FileName` を絶対パスに解決する。

| `Program` | 解決 |
|-----------|------|
| `/` を含む(絶対パスまたは相対パス) | `[System.IO.Path]::GetFullPath($program)`(プロセスの作業ディレクトリが基準。4.2 節の注意)。存在を確かめない(`exec` の失敗は、4.3 節の例外になる) |
| `/` を含まない | `PATH`(環境変数。`Deps` の `Env.GetEnvironmentVariable('PATH')` で読む)を `:` で分け、各ディレクトリで、`[System.IO.Path]::Combine($dir, $program)` のファイルを探す。空のエントリ(`PATH` の先頭、末尾、`::`)は、カレントディレクトリ。ファイルであり(ディレクトリでない。`[System.IO.File]::Exists`)、実行ビットのどれか(`[System.IO.File]::GetUnixFileMode` の `UserExecute`、`GroupExecute`、`OtherExecute`)が立っている最初のものを採る。`PATH` が未設定のときは、探索せず、起動の失敗(OS の既定の `PATH` は使わない)。見つからなければ、起動の失敗(127) |

`GetUnixFileMode` は .NET 7 以降の API である(確認: API の存在は既知。この文書では出典の確認を行っていない。未確認)。`FileName` に絶対パスを渡すと、`Process.Start` は、探索せず、そのまま使う(確認。出典: .NET ランタイムのソースの `ResolvePath` の「完全なパスなら、存在を問わず使う」。公式の API 文書ではない)。`Get-Command` と `which` を使わない。

---

## 5. 待ち(`WaitForExit(int)` の繰り返し)

### 5.1 規定

| 項目 | 内容 | 強さ |
|------|------|------|
| 待ち方 | `$child.WaitForExit([int]$ms)` の繰り返し(待機ループ)。引数なしの `WaitForExit()` を使わない。理由は 5.2 節 | 【PowerShell の設計】 |
| 回収 | 終了の状態を読む(`WaitForExit(int)` が `$true` を返し、`ExitCode` を取る)のは、`Wait-ConductorChild` と `Invoke-ConductorJob` だけ | 【規定】 |
| 転送の判断 | 「ジョブがまだ終わっていない」と観測した時点で、**まだ転送していない**かつ合図が立っていれば、シグナルをジョブに **1 回だけ** 転送する(7.5 節) | 【規定】 |
| 終了済みへの送信の禁止 | 終了済み・回収済みのジョブには送らない。ジョブが先に終わっていれば、合図が立っていても転送しない | 【規定】 |
| 転送は 1 回 | 2 回目以降の合図(同じシグナルでも別のシグナルでも)で、再度の転送や強制終了はしない | 【規定】 |
| 転送のあと | ジョブの終了を待ち続ける(ジョブがシグナルを無視すれば、`conductor` はそのまま待ち続ける) | 【規定】 |
| 確認の間隔 | `WaitForExit(int)` の待ち時間を、1、2、4、8、10、10、…(ミリ秒。前回の 2 倍、上限 10 ms)にする。ジョブが終わったときは、待ち時間の経過を待たずに戻る(UNIX 系の子プロセスでは、イベントによる待ち。確認: 出典: .NET ランタイムのソース `ProcessWaitState.Unix.cs`。公式の API 文書ではない)。Windows のタイマーの分解能は、これより粗いことがある(未確認) | 【目安】(間隔が長すぎて、合図からの転送が目に見えて遅れないこと。10 ms 以下を推奨) |
| 待機の専用スレッド | 使わない(呼び出したスレッドで待つ)。`Start-Job`、`Start-ThreadJob`、`ForEach-Object -Parallel` を使わない | 【規定】 |
| CPU の使用 | 待ち中の CPU 時間の増分が小さいこと(待機の 1 秒後から 2.5 秒後までで 50 ms 以下。11 章。Linux の `/proc` で計測) | 【目安】(PowerShell での実測は未確認。1 周ごとに PowerShell の文を数個実行するので、コンパイル言語より多い) |

### 5.2 `WaitForExit()` と `WaitForExit(int)` の使い分け

| 方法 | 動作 | 採否 |
|------|------|------|
| `WaitForExit()`(引数なし) | 終了まで無期限に待つ。さらに、標準出力が非同期のイベントハンドラにつながれているとき、そのイベントの処理が終わるまで待つ(確認。出典: `Process.WaitForExit` の API 文書の Remarks)。この仕様では、標準入出力をつながないので、イベントの処理待ちは起こらない。ただし、無期限に待つため、合図が立っても、呼び出しが戻らず、転送できない(転送の規定に反する)。**PowerShell 固有**: 無期限に待つ .NET の呼び出しの間は、Ctrl+C による停止(`PipelineStoppedException`。7.7 節)も、呼び出しが戻るまで効かない | 使わない。ただし、7.7 節の停止経路の `finally` の内側だけは、`WaitForExit(int)` を繰り返して同じ待ちをする(無期限の待ちを使わない) |
| `WaitForExit(int)` | 指定のミリ秒だけ待ち、終了していれば `$true`、そうでなければ `$false`。`0` は即時に戻り、終了済みのときだけ `$true`。`-1` は無期限。負の値(`-1` 以外)は `ArgumentOutOfRangeException`(確認。出典: `Process.WaitForExit(Int32)` の API 文書) | **採用**。待機ループの中心。待ち時間は、10 ms 以下(0 以上) |
| `Process.HasExited` | 終了しているか。ブロックしない | 転送の直前の確認(7.5 節)に使う |
| `Exited` イベント(`EnableRaisingEvents`)、`Register-ObjectEvent` | 終了を、別スレッドのコールバックで通知する。PowerShell のイベントは、メインのスレッドが戻らないと処理されない | 使わない(10 章) |
| `WaitForExitAsync` | 非同期 | 使わない(同期モデル。10 章) |
| `Wait-Process` | PowerShell のコマンドレット | 使わない(終了コードを返さず、転送の判断を挟めない) |

標準入出力を `Redirect*=$true` でつなぐと、子孫(`sh -c` が起こした孫など)が書き込み側を持ったまま動いているときに、`WaitForExit()` が戻らないことが起こりうる(詳細は公式の API 文書に書かれていない。未確認)。この仕様は、標準入出力をつながないので、この問題は起こらない。したがって、`Redirect*=$false` を**必ず**明示することが、この問題を避ける条件である。

### 5.3 待機ループの手順(骨格。実装コードではない)

```text
forwarded = $false
interval = 1                                          # ms
loop:
    try:
        if $child.WaitForExit(0): break               # すでに終了している(回収済み)
        if $sig.IsRaised and not forwarded:           # IsRaised を先に読み、true のときだけ Number を読む(6 章)
            n = $sig.Number
            forwarded = $true                         # 成否にかかわらず 1 回でやめる
            Send-ConductorSignal(child, n, platform)  # 例外を握りつぶす(7.5 節)。HasExited を確かめてから送る
        if $child.WaitForExit(interval): break
    catch (Win32Exception) / (InvalidOperationException):
        結果 = 起動の失敗(4.1 節の手順 8); return
    interval = min(interval * 2, 10)
結果 = 4.4 節で取り出す
```

- ループの 1 周目で、`WaitForExit(0)` の直後に合図を読む。したがって、`Process.Start` が返る前に立った合図は、1 周目で転送される。
- `Send-ConductorSignal` と `WaitForExit` の呼び出しを、差し替え可能にする(スクリプトブロックを引数に取る)と、偽の子プロセスで、待ち時間(`WaitForExit` に渡すミリ秒)の列 1、2、4、8、10、10、…、転送の回数を単体で検査できる(11 章)。
- **PowerShell 固有**: `$child.WaitForExit(0)` は `[bool]` を返す。`if` の条件に直接入れるか、変数に受ける(捨てた値は、関数の戻り値に混ざる)。
- **子の終了と `pid` の再利用**: .NET は、`SIGCHLD` を受けると、ランタイムの内部のスレッドが、子を回収する。子が終わった直後に、`kill` を呼ぶ前に、子が回収され、`pid` が別のプロセスに再利用される競合が、理論上はありうる(確認。出典: .NET ランタイムのソース `ProcessWaitState.Unix.cs`)。この競合を狭めるため、転送の直前に `HasExited` を確かめ、`$true` なら送らない(7.5 節)。確認から `kill` までの、ごくわずかな間は、残る。許容する(「終了済みの子には送らない」規定の、実装上の限界。未確認)。
- `Process.Id` を、転送の直前に 1 回だけ読む(`HasExited` が `$true` になったあとに `Id` を読むと、例外になりうる。未確認)。

---

## 6. 中断の合図(`ConductorNative.Interrupt`)

「中断の合図」は、`conductor` がシグナルを受け取ったことを、ジョブを待つ側へ伝える値。シグナルのコールバックは別スレッドで動くので、合図は **C# ヘルパーの型**として実装する(7.1 節の理由)。PowerShell の `[ref]`、`$script:` 変数、`[hashtable]::Synchronized` を、スレッド間の合図に使わない【規定】。

### 6.1 規定

| 項目 | 規定 |
|------|------|
| 内容 | 「立っているか」と、立っているなら「最初のシグナルの番号」 |
| 初期 | 立っていない |
| 立てる(番号 n) | 立っていなければ n を保存する。**すでに立っていれば何もしない**(最初の番号が残る)。番号は任意の整数でよく、0、負数、非常に大きい値でも、「立った」ことは記録される |
| 性質 | 立ったら下りない。シグナルのコールバックのスレッドと、待つ側のスレッドのあいだで、安全に共有できる(競合しない)。待つ側は読むだけ |
| 判定 | 「立っているか」は `IsRaised`(`bool`)で判定する(番号 0 を「立っていない」と取り違えない)。`Number` は、`IsRaised` が `$true` のときだけ読む。`Number` は、一度決まると変わらない |
| 読み取り | 1 つの判断のなかで、`IsRaised` と `Number` を、1 回ずつだけ読み、その値を使う(10.1 節) |

### 6.2 型の骨格【PowerShell の設計】

```csharp
namespace ConductorNative
{
    public sealed class Interrupt
    {
        // 立っていなければ n を保存して立てる。すでに立っていれば何もしない。例外を送出しない。
        // 実装: 0 を「立っていない」とする long に、(1L << 32) | (uint)n をまとめて保存し、
        //       Interlocked.CompareExchange(ref value, packed, 0) の成功だけが番号を決める。
        //       成功したあとに、ManualResetEventSlim.Set() で、待っている側を起こす。
        public void Trigger(int n) { /* ... */ }

        // Volatile.Read で読む。
        public bool IsRaised { get { /* ... */ } }

        // IsRaised が true のときだけ意味を持つ。false のときは 0 を返す(読まないこと)。
        public int Number { get { /* ... */ } }

        // 立つまで最大 milliseconds だけ待つ。立っていれば true(リトライの待機。8 章)。
        // milliseconds は 0 以上 int.MaxValue 以下。例外を送出しない。
        public bool Wait(int milliseconds) { /* ... */ }
    }
}
```

- `ManualResetEventSlim` は、`Set` の後に `Reset` しない(立ったら下りない)。`Wait(int)` は、`Set` されていれば直ちに `true` を返す。
- PowerShell 側は `New-ConductorInterrupt`(`[ConductorNative.Interrupt]::new()` を返す。`Initialize-ConductorNativeHelper` を先に呼ぶ)で作る。
- `Interrupt` は、プロセスの静的な状態を持たない(インスタンスごとの状態)。単体テストは、インスタンスを作り、`Trigger` を呼んで、合図を再現できる(実際のシグナルを使わない)。
- PowerShell から `$sig.IsRaised` を読むと、`[bool]` が返る(.NET の `bool` は、PowerShell の `[bool]`)。

---

## 7. シグナル処理

### 7.1 スクリプトブロックを別スレッドのコールバックにできない問題と、決定【PowerShell の設計】

**問題**: PowerShell のスクリプトブロックは、.NET のデリゲート(`Action`、`ConsoleCancelEventHandler`、`PosixSignalContext` を取るデリゲートなど)に変換できる。しかし、そのデリゲートが、PowerShell の実行空間(runspace)を持たないスレッド(シグナルのコールバックのスレッド、スレッドプールのスレッド)から呼ばれると、スクリプトブロックの実行に必要な runspace が、そのスレッドに見つからず、失敗する(例外 `There is no Runspace available to run scripts in this thread`)。スクリプトブロックの実行は、スレッドローカルの記憶から runspace を探すため。確認(出典: PowerShell チームのブログ記事「Invoking Scriptblocks from C#」と、Stack Overflow の回答。どちらも公式の API 文書ではない。回答は、デリゲートを包んで `Runspace.DefaultRunspace` を設定する回避策を示すが、runspace を使用中のスレッドと競合しうる)。さらに、PowerShell の `class` は、既定で、作った runspace に結び付き、別スレッドからのメソッド呼び出しは、元の runspace へ振り分けられ、状態の破損やデッドロックの恐れがある(確認。出典: `about_Classes` の `NoRunspaceAffinity` の節。7.4 で `[NoRunspaceAffinity()]` が追加された)。したがって、PowerShell の `class` のメソッドをコールバックにする案も採らない。

**選択肢の比較**:

| 案 | 内容 | 採否 | 理由 |
|----|------|------|------|
| (a) C# ヘルパー(`Add-Type`) | シグナルの受け取り(`PosixSignalRegistration`、`Console.CancelKeyPress`)と合図(`Interrupt`)を、C# のクラスの中で完結させる。PowerShell は、`IsRaised` と `Number` を読むだけ | **採用** | コールバックが、PowerShell を一切呼ばない。runspace に依存しない。`kill` と `fsync` の P/Invoke も同じ型に置ける |
| (b) PowerShell のイベント(`Register-ObjectEvent`、`Register-EngineEvent`) | イベントを PowerShell のイベントキューに入れ、`-Action` のスクリプトブロックまたは `Get-Event` で処理する | 不採用 | `Register-ObjectEvent` の `-Action` は、別の runspace(イベントのスクリプトを動かす専用)で動き、メインのスクリプトが実行中は処理されない場合がある(確認していない。未確認)。`PosixSignalRegistration` の `Create` は、デリゲートを取るので、スクリプトブロックを渡すと問題が再現する。`Cancel` の設定が、ハンドラの戻りまでに必要で、非同期の処理では間に合わない |
| (c) その他 | (1) スクリプトブロックを `Runspace.DefaultRunspace` を設定して包む回避策、(2) 専用の runspace、(3) `Console.TreatControlCAsInput` | 不採用 | (1) 使用中のスレッドの runspace と競合しうる(未確認)。(2) 複雑で、PowerShell のスクリプトブロックの実行の同期を要する。(3) キーとして読むだけで、`kill` で送られたシグナルを受けられず、標準入力が端末でないときは例外になる(未確認) |

**採用: (a)**。ヘルパーの C# のソースは、`Private/Runner/NativeHelper.ps1` の中の一重引用符の here-string(`@'...'@`)として持つ(エスケープが要らない)。`Add-Type` を呼ぶのは、`Initialize-ConductorNativeHelper` の 1 か所だけ【規定】。

### 7.2 ヘルパーの設計

名前空間 `ConductorNative`(`Add-Type` の既定の名前空間 `Microsoft.PowerShell.Commands.AddType.AutoGeneratedTypes` を使わず、`-TypeDefinition` のソースに、名前空間を書く。確認: `Add-Type` の API 文書は、`-TypeDefinition` に名前空間の宣言を含めることを勧める)。型は、次の 4 つ。

| 型 | 種別 | 責務 |
|----|------|------|
| `ConductorNative.HelperInfo` | `static class` | `public const int Revision`(ヘルパーのソースの改訂番号。7.3 節の再読み込みの検査に使う) |
| `ConductorNative.Interrupt` | `sealed class`(インスタンス) | 中断の合図(6 章) |
| `ConductorNative.SignalWatcher` | `static class` | シグナルの受け取りの登録(7.4・7.6 節)。登録の保持(静的なフィールド)。「プロセスで 1 回だけ」の印 |
| `ConductorNative.UnixNative` | `static class` | `libc` の `kill`、ディレクトリの `fsync`(7.5 節)。`ExitState`(7.7 節)も同じ型に持ってよい |

骨格(型・メソッドの署名と責務。本体は書かない):

```csharp
namespace ConductorNative
{
    public static class HelperInfo { public const int Revision = 1; }

    public sealed class Interrupt { /* 6.2 節 */ }

    public static class SignalWatcher
    {
        // 戻り値: 0 = Started、1 = AlreadyRegistered、2 = Failed。例外を送出しない。
        // isWindows: true なら Console.CancelKeyPress(Ctrl+C だけ)、false なら
        //            PosixSignalRegistration で SIGHUP/SIGINT/SIGQUIT/SIGTERM を登録する。
        // OS の判定は、ヘルパーの中で行わない(呼び出し側の Get-ConductorPlatformKind の結果を受け取る)。
        // ハンドラは、Cancel を true にして sig.Trigger(実際の番号) を呼ぶだけ(出力・ファイル・ロック・待ちをしない)。
        // 登録の戻り値(PosixSignalRegistration、デリゲート)は、静的なフィールドに保持する。
        public static int Start(Interrupt sig, bool isWindows) { /* ... */ }

        // 登録を元に戻す(Dispose / CancelKeyPress -=)。静的な印を戻す。本番では呼ばない(単体テスト用)。
        public static void Stop() { /* ... */ }
    }

    public static class UnixNative
    {
        // libc の kill(pid, sig) を呼ぶ。戻り値は成功なら true。例外を送出しない
        // (ライブラリが見つからない場合も false)。検査(pid > 0、許可集合)は呼び出し側(PowerShell)で行う。
        public static bool Kill(int pid, int sig) { /* ... */ }

        // ディレクトリを開いて fsync し、閉じる(06 章)。失敗はすべて無視。例外を送出しない。
        public static void SyncDirectory(string directory) { /* ... */ }

        // 終了コードの予約(7.7 節)。launcher が、停止経路で Environment.Exit に使う。
        public static void SetPendingExitCode(int code) { /* ... */ }
        public static bool HasPendingExitCode { get { /* ... */ } }
        public static int PendingExitCode { get { /* ... */ } }
    }
}
```

**P/Invoke の方法**: `[DllImport("libc")]` による宣言の代わりに、`NativeLibrary.TryLoad` と `NativeLibrary.TryGetExport` で `kill`、`open`、`fsync`、`close` の関数ポインターを実行時に求め、`Marshal.GetDelegateForFunctionPointer` でデリゲートにする【PowerShell の設計】。理由: (1) `Add-Type` は、Roslyn でコンパイルするだけで、`LibraryImport` のソース生成器を使えない。(2) `DllImport` の名前 `"libc"` が、Linux と macOS で解決される保証が、公式の文書にない(`libc.so` が、リンカースクリプトのテキストファイルの場合に、読み込みに失敗する。確認していない。未確認)。(3) 解決の候補を、自前で明示できる。候補の順序: Linux は `libc.so.6`、続けて `libc`。macOS は `/usr/lib/libSystem.B.dylib`、続けて `libc`。どれも失敗したら、`Kill` は `false`、`SyncDirectory` は何もしない(失敗の握りつぶし)。この名前の組が、実機で正しく解決されるかは、未確認(12 章の試験 T-3)。`open` は可変長引数の関数(`open(path, flags, ...)`)で、`O_RDONLY`(0)だけを渡し、モードを渡さない(ディレクトリを読み取りで開く。`O_DIRECTORY` は OS ごとに値が違うので使わない)。可変長引数の呼び出し規約が、arm64 の macOS で、引数を渡さなくても問題ないかは、未確認(T-3 で確かめる)。`pid_t` は `int`(Linux と macOS の通常の 64 ビットの ABI で 32 ビット。未確認)。ヘルパーが、Windows で `UnixNative` のメソッドを呼ばれたら、何もせず `false` を返す(呼び出し側が OS を分けるので、通常は呼ばれない)。

### 7.3 `Add-Type` の運用

**読み込み(`Initialize-ConductorNativeHelper`)**:

```text
1. 型が読み込み済みかを調べる:  $t = 'ConductorNative.HelperInfo' -as [type]
   - $t があり、$t::Revision が、ソースの Revision と等しい: 何もしない(再読み込みしない)。
   - $t があり、Revision が違う: 失敗(別の改訂のヘルパーが、同じセッションに残っている)。呼び出し側が「Failed」として扱う。
   - $t がない: Add-Type -TypeDefinition $script:ConductorNativeSource -Language CSharp
                 -ErrorAction Stop を呼ぶ(出力は捨てる: PassThru を付けない)。
2. 例外(コンパイルの失敗、Constrained Language Mode、AppLocker など)は、捕まえて「Failed」にする。
```

| 項目 | 規定・確認 |
|------|-----------|
| 再実行時の型の再定義 | `Add-Type` は、PowerShell 7 以降、同じ名前の型がすでにあれば、コンパイルしない(エラーにしない)(確認。出典: `Add-Type` の API 文書の説明の「Beginning in PowerShell 7, Add-Type doesn't compile a type if a type with the same name already exists」)。型の名前と名前空間は、セッション内で一意でなければならず、型を取り除くことも変えることもできない。コードを変えるには、名前を変えるか、新しいセッションを始める(確認。出典: 同文書)。したがって、`Import-Module -Force` を繰り返しても、再定義のエラーは起こらない。**ただし、ソースを書き換えて `-Force` で読み直しても、古いヘルパーが残る**(エラーなしで古い定義が使われる)。この事故を避けるため、`HelperInfo.Revision` を、ソースを変えるたびに増やし、読み込み済みの型の `Revision` と、モジュールが持つ期待の `Revision` を比べる(上の手順 1)。Windows PowerShell 5.1 では、再定義が例外になる(対象外) |
| 読み込みの遅延 | `Initialize-ConductorNativeHelper` は、`run` と `pipeline` の準備(シグナル監視の開始。7.4 節)で初めて呼ぶ。`Import-Module` の時点、`watch`・`reset`・`validate`・`convert`・`--help`・`--version` では呼ばない【規定】。`conductor 0.8.0` や `--help` の起動時間を、コンパイルのコストで遅らせないため |
| 起動のコスト | `Add-Type` は、Roslyn でソースをコンパイルし、メモリ上のアセンブリを作る(確認。出典: `Add-Type` の API 文書の説明の「compiles the specified source code and generates an in-memory assembly」)。コンパイルの所要時間(数百ミリ秒の可能性がある)は、公式の文書に書かれていない(未確認)。ヘルパーのソースは、小さく保つ(約 200 行以内を目安)。実装時に、`run` の起動の所要時間のうち `Add-Type` が占める値を計測する(12 章の試験 T-2。11 章の startup-bench に含める)。結果が、受け入れテストの時間の予算を超えるなら、7.8 節の別案を再検討する |
| 参照アセンブリ | `-ReferencedAssemblies` の既定の扱いは、PowerShell 6 以降で、既定の .NET のアセンブリを含まない、と書かれている(確認。出典: `Add-Type` の API 文書。この記述が、実際に必要な参照の指定を意味するかは、未確認)。ヘルパーが使う型(`Console.CancelKeyPress`、`PosixSignalRegistration`、`NativeLibrary`、`Marshal`、`ManualResetEventSlim`)の参照が、`-ReferencedAssemblies` なしで解決されるかは、未確認(T-2 で確かめる。解決しなければ、`System.Console`、`System.Runtime`、`System.Runtime.InteropServices`、`System.Threading` などを指定する) |
| コンパイルの警告 | `-IgnoreWarnings` を付けない(警告が出たらエラー扱いにして、ソースを直す。「警告をエラーとして扱う」と、API 文書は書く: `Add-Type` の `-IgnoreWarnings` の説明) |
| 名前空間 | ソースに明示する(`ConductorNative`)。`-Namespace`(`-MemberDefinition` 用)を使わない。`-TypeDefinition` を使う |
| コンパイラーの選択 | `-Language` は `CSharp` だけ(確認。出典: `Add-Type` の API 文書) |

**Constrained Language Mode と実行ポリシー**:

| 項目 | 内容 |
|------|------|
| Constrained Language Mode(CLM) | `Add-Type` は、署名されたアセンブリは読み込めるが、任意の C# のコードと Win32 API は読み込めない(確認。出典: `about_Language_Modes` の ConstrainedLanguage の節)。さらに、CLM では、許可された型の一覧にない .NET の型(`System.Diagnostics.ProcessStartInfo`、`System.IO.File` など)を使えない(確認。出典: 同文書の許可された型の一覧)。AppLocker または Windows Defender Application Control のポリシーのもとで、`pwsh` は自動で CLM で動く(確認。出典: 同文書)。ただし、ポリシーが許可したスクリプトとスクリプトモジュールは、FullLanguage で動く(確認。出典: 同文書)。**この版は CLM を対象外とする**: CLM で動く場合は、`Add-Type` が失敗し(`Initialize-ConductorNativeHelper` が「Failed」を返す)、`run`/`pipeline` は 7.4 節の規則で終了コード 70 になる。`watch`・`validate` なども、CLM では、`[System.IO.File]` を使えず、(上の想定外の例外の経路で)終了コード 70 の `internal error` になる(専用のメッセージは設けない。CLM の検出と専用の案内は、この版では実装しない) |
| 実行ポリシー | `pwsh` の実行ポリシーは、Windows だけで有効で、他の OS では無視される(確認。出典: `about_Pwsh` の `-ExecutionPolicy` の節)。実行ポリシーは、`.ps1` と `.psm1` の読み込みを制限する(リモートで署名されたものだけ、など)が、`Add-Type` の呼び出しそのものは制限しない(この点の公式の文書は確認していない。未確認)。ランチャーは、`-ExecutionPolicy Bypass` を付けない([02-architecture.md](02-architecture.md) の 9 章)。ダウンロードしたファイルの扱い(`Zone.Identifier`、`Unblock-File`)は、利用者向けの案内([11-quality-and-build.md](11-quality-and-build.md)) |
| AppLocker / WDAC | これらは、読み込む `.ps1` と、ネイティブのライブラリの読み込みを制限しうる。`libc` は OS のライブラリなので、通常は制限されない(未確認) |

### 7.4 シグナル監視の開始(`Start-ConductorSignalWatch`)

`Start-ConductorSignalWatch -Interrupt $sig -Platform $kind` は、`'Started'`、`'AlreadyRegistered'`、`'Failed'` のどれかの文字列を返す。

1. `Initialize-ConductorNativeHelper` を呼ぶ。「Failed」なら `'Failed'` を返す。
2. `[ConductorNative.SignalWatcher]::Start($sig, $kind -eq 'Windows')` を呼ぶ(戻り値 0、1、2 を、`'Started'`、`'AlreadyRegistered'`、`'Failed'` に写す)。
3. `Failed` のとき、流れの制御は、標準エラーに `conductor: error: failed to set up signal handling` を書き、終了コード **70** で終わる(定義の検査もジョブの起動もしない)。

- `run` と `pipeline` だけが、**定義の検査より前**に、この関数を呼ぶ【規定】。`watch`・`reset`・`validate`・`convert` は監視せず、OS の既定の動作で終わる(7.7 節の launcher の規則を除く)。
- 登録は **プロセスで 1 回だけ**(ヘルパーの静的な印を `Interlocked` で立てる)。2 回目の呼び出しは `'AlreadyRegistered'` として、流れを成功と同様に続ける(2 回目に渡した合図には配線されない)。登録に失敗したときは、すでに登録できた分を解除し、印を戻して、次の登録を許す。
- 「最初の番号」は、`Interrupt.Trigger` が、原子的な比較交換に成功した順の最初(6 章)。
- **この関数の呼び出しの PowerShell 固有の注意**: 戻り値の文字列以外の出力を出さない(`Add-Type` の出力は捨てる)。

### 7.5 UNIX 系(Linux、macOS)

**登録(ヘルパーの `SignalWatcher.Start(sig, isWindows: false)` の中)**: 次の 4 つのシグナルを、`PosixSignalRegistration.Create(signal, handler)` で受ける【規定】。ハンドラは、C# のデリゲート(PowerShell を呼ばない)。

| シグナル | `PosixSignal` の値 | 実際の番号(`128 +` に使う) |
|----------|--------------------|------------------------------|
| SIGHUP | `PosixSignal.SIGHUP`(-1) | 1 |
| SIGINT | `PosixSignal.SIGINT`(-2) | 2 |
| SIGQUIT | `PosixSignal.SIGQUIT`(-3) | 3 |
| SIGTERM | `PosixSignal.SIGTERM`(-4) | 15 |

`PosixSignal` の列挙値は、負の値(OS に依存しない識別子)であり、シグナル番号そのものではない(確認。出典: `PosixSignal` の API 文書)。`Trigger` に渡すのは、上の表の「実際の番号」(1、2、3、15)である【規定】。この 4 つの番号は、Linux と macOS で同じである(POSIX の規定。実機では未確認)。各シグナルごとに、番号を持った別のラムダを作る。

**ハンドラ**【規定】:

```text
handler(PosixSignalContext ctx):
    ctx.Cancel = true           // ランタイムの既定の動作(プロセスの終了)を打ち消す
    interrupt.Trigger(番号)      // これだけ。出力・ファイルの書き込み・ロック・ブロックをしない
```

- `PosixSignalContext.Cancel` の既定は `false` で、ハンドラのあと、`Cancel` が `false` のままなら、ランタイムが、そのシグナルの既定の動作(終了など)を実行する(確認。出典: `PosixSignalContext.Cancel` の API 文書)。4 つとも `Cancel = true` にする。SIGTERM を打ち消せるのは UNIX 系だけである(確認。出典: `PosixSignalRegistration.Create` の API 文書の Remarks)。.NET 10 以降、ランタイムが SIGTERM の独自のハンドラを入れなくなり、OS の既定の動作でプロセスが即座に終わる(確認。出典: .NET の互換性に関する文書「SIGTERM signal handler」。`pwsh` が載せる .NET の版と、その影響は未確認)。この仕様は、自前で `PosixSignalRegistration` を登録して打ち消すので、この変更の影響を受けない。
- ハンドラは、メインスレッドとは別のスレッドで動く(確認。出典: .NET ランタイムのソース `PosixSignalRegistration.Unix.cs`: SIGINT、SIGQUIT、SIGTERM は専用の背景スレッドで、SIGHUP などはスレッドプールで動く。公式の API 文書には、この記述はない)。複数のハンドラがあるときの順序は、.NET 10 以降、登録の逆順(確認。出典: `PosixSignalRegistration.Create` の API 文書)。この仕様では、1 つのシグナルに 1 つだけ登録する。
- **登録の寿命**: `PosixSignalRegistration` の `Finalize()` は、ハンドラを解除する(確認。出典: `PosixSignalRegistration` の API 文書)。したがって、`Create` の戻り値(4 つ)を、ヘルパーの**静的なフィールドに保持**して、ガベージコレクションで解除されないようにする【規定】。PowerShell の変数にだけ持つ設計にしない。
- `Create` の例外(`PlatformNotSupportedException`、`IOException`)は、`Failed`(2)にする(確認。出典: `PosixSignalRegistration.Create` の API 文書)。
- `PosixSignal.SIGCHLD` を登録しない【規定】(.NET のランタイムが、子の回収に使う)。
- **`pwsh` 自身の Ctrl+C の処理との関係(重要)**: `pwsh` は、UNIX 系で、`Console.CancelKeyPress` に自分のハンドラ(`MyBreakHandler`)を登録し、Ctrl+C(SIGINT)と Ctrl+Break(SIGQUIT)で、実行中のパイプラインを停止する(確認。出典: PowerShell のソース `ConsoleHost.cs` の `BindBreakHandler` と `HandleBreak`。ソースの先頭の 100,000 文字だけを読んだ。全体は読んでいない)。同じソースに、SIGTERM や SIGHUP の処理は見当たらなかった(読んだ範囲。未確認)。したがって、SIGHUP と SIGTERM は、ヘルパーの合図だけが立ち、`pwsh` のスクリプトは止まらない(想定どおりの経路)。SIGINT と SIGQUIT では、ヘルパーの合図が立つのに加えて、`pwsh` のハンドラが働き、**メインのスクリプトが `PipelineStoppedException` で止まりうる**(7.7 節)。`PosixSignalRegistration` で SIGINT に `Cancel = true` を返した場合に、.NET が `Console.CancelKeyPress` のハンドラを呼ぶか呼ばないかは、公式の文書で確かめていない(未確認。12 章の試験 T-1)。この仕様は、**どちらでも正しく動く**ように、7.7 節で、両方の経路を規定する。

時系列(ジョブの実行中に、利用者が `kill -TERM <conductor の pid>` したとき。`pwsh` が止まらない経路):

1. `pwsh` のプロセスが SIGTERM を受け取る。ランタイムが、背景スレッドで、ヘルパーのハンドラを呼ぶ。ハンドラが `Cancel = true` にして、`Trigger(15)` を呼ぶ(合図が立つ)。
2. メインスレッドの待機ループ(PowerShell)は、次の確認(最大 10 ms 後)で「まだ終わっていない」を観測し、`$sig.IsRaised` が `$true` なのを見る。
3. ジョブの pid に、**同じシグナルを 1 回**送る(`Send-ConductorSignal`)。送る前に次を検査する。
   - `$child.HasExited` が `$false`(`$true`、または `InvalidOperationException` なら、送らない)。
   - pid(`$child.Id`)が 0 より大きい(0 や負にならない。プロセスグループへの送信にならない)。
   - シグナルが `{1, 2, 3, 15}`(SIGHUP、SIGINT、SIGQUIT、SIGTERM)のどれか(SIGKILL(9)、SIGSTOP、SIGCONT、SIGUSR1、0、負、9999 は送らない)。
   - 送信先は、**ジョブの子プロセスの pid だけ**。ジョブの子孫(`sh -c` が起こした孫など)には、`conductor` は送らない。`kill` の第 1 引数に、負の値や 0 を渡さない。
   - 送信は、`[ConductorNative.UnixNative]::Kill($pid, $sig)` で行う(`UnixNative` を使えるのは、`Initialize-ConductorNativeHelper` が成功したあと)。戻り値(失敗)は無視する(`[void]` または `$null =` で捨てる)。例外は握りつぶす。ただし、`Kill` が黙って失敗し続けると、転送が行われないため、受け入れテスト(11 章)が、3 OS で、このことを検出する。
4. ジョブは、受け取ったシグナルの既定の動作で終了するか、自分で処理する。`conductor` は待ち続ける。2 回目以降のシグナルでは、何も追加しない。
5. ジョブの終了を観測する。`ExitCode` から、4.4 節の規則で `ExitCode` と `TerminatedBySignal` を決める。直後に合図が立っているので `Interrupted = $true`、`SignalNumber = 15` を足す。
6. 分類で「エラー、`128 + 15 = 143`」となり、流れの制御は、この結果をステートに**エラーとして**記録する(再開位置はそのジョブ)。
7. `Invoke-Conductor` は **143** を返す。launcher が `exit 143` で終わる(`pipeline` は次のジョブを起動しない)。

その他:

- ジョブは、`conductor` と同じプロセスグループで動く(4.7 節の決定)。端末の Ctrl+C(SIGINT)は、ジョブにも直接届き、`conductor` の転送でもう 1 回届く(4.7 節の帰結)。ジョブは `TerminatedBySignal = 2`(4.4 節の規則で推定)で終わるのが典型。
- 転送の許可集合にない番号で合図が立った場合(`conductor` 本体の監視は 4 つだけを受けるため、通常は起こらない。部品を単独で使う場合のみ)は、送信せず、ジョブが自分で終わるのを待ち、終わりを観測した後に `Interrupted = $true`、`SignalNumber = その番号` を足す(ジョブの終了コードは `ExitCode` に残るが、分類では見ない)。
- 時間を空けて複数のシグナルが届いたときは、到着順の最初の番号が残る(INT のあとの HUP でも 2)。ほぼ同時に届いたときに、どれが「最初」になるかは**規定しない**(シグナルごとに、別のスレッドでハンドラが動くため、順序は決まらない)。
- 終了コードの例: SIGHUP は 129、SIGINT は 130、SIGQUIT は 131、SIGTERM は 143。

### 7.6 Windows

受け取るのは **Ctrl+C(番号 2)だけ**【規定】。ヘルパーの `SignalWatcher.Start(sig, isWindows: true)` が、`Console.CancelKeyPress` で受ける【決定】。

```text
Console.CancelKeyPress += handler   // handler は C# のデリゲート(ヘルパーの静的メソッド)
handler(sender, ConsoleCancelEventArgs e):
    if (e.SpecialKey == ConsoleSpecialKey.ControlC):
        e.Cancel = true         // プロセスの終了を打ち消す
        interrupt.Trigger(2)    // これだけ
    // ControlBreak: 何もしない(e.Cancel を false のままにする。OS の既定の動作で終わる)
```

- `CancelKeyPress` は、Ctrl+C または Ctrl+Break で起こり、`SpecialKey` で、どちらかを区別できる。`Cancel` の既定は `false` で、ハンドラのあと、プロセスが終了する。`true` にすると、終了しない(確認。出典: `Console.CancelKeyPress` の API 文書)。ハンドラは、スレッドプールのスレッドで実行される(確認。出典: 同文書)。
- `PosixSignalRegistration` を Windows で使わない【決定】。Windows でどの OS のイベントがどの `PosixSignal` に対応するかは、公式の API 文書に書かれていない(未確認)。
- 登録は、プロセスで 1 回だけ(7.4 節の印)。解除(`-=`)のために、デリゲートを静的なフィールドに保持する(`Stop` で使う)。`+=` が例外を送出する場合は、`Failed`(2)にする。「すでに別のハンドラが登録されている」という失敗は、この API にはない(複数のハンドラが、すべて呼ばれる。`pwsh` 自身のハンドラも登録されている)。
- **`pwsh` 自身の Ctrl+C の処理**: Windows の `pwsh` は、コンソールの制御ハンドラ(`ConsoleControl.AddBreakHandler`)で、Ctrl+C を受け、実行中のパイプラインを停止する(確認。出典: PowerShell のソース `ConsoleHost.cs` の `BindBreakHandler`。UNIX 系は `Console.CancelKeyPress`、Windows は別の登録、と分岐していた)。したがって、7.5 節の SIGINT と同様に、ヘルパーの合図が立つのに加えて、**メインのスクリプトが `PipelineStoppedException` で止まりうる**(7.7 節)。

時系列(ジョブの実行中に、利用者が Ctrl+C を押したとき):

1. ジョブは `conductor` と **同じコンソール**で起動されている(`CreateNewProcessGroup` を使わない。4.2 節)。Ctrl+C は、OS が、同じコンソールを共有するプロセス(`conductor` とジョブ)の両方に配る(Windows のコンソールの規則。この文書では実機で未確認)。
2. ヘルパーのハンドラが `Cancel = true` にして、合図を立てる(番号 2)。`pwsh` のハンドラが、メインのスクリプトの停止を要求する(7.7 節)。
3. 待機ループ(または停止経路の `finally`)は転送を試みるが、Windows の転送は **何もしない**(`Send-ConductorSignal` は、`$kind -eq 'Windows'` のとき何もしない)【規定】。
4. ジョブは、OS から受け取った Ctrl+C で(既定の動作で)終了する。
5. 終了を観測する。`TerminatedBySignal` は常に `$null`、`ExitCode` は OS の終了コード(Ctrl+C による終了コード)。直後に合図が立っているので `Interrupted = $true`、`SignalNumber = 2`。
6. 分類で「エラー、`128 + 2 = 130`」。ステートに**エラーとして**記録し、**130** で終わる。

その他:

- Ctrl+Break とコンソールを閉じたときの通知は、この版の対象外(OS の既定の動作)。`ControlBreak` のとき `Cancel` を `false` のままにしても、実際にプロセスが終了することは、実機で未確認。
- 自動の試験環境(CI の Windows ランナー)には、対話的なコンソールがなく、Ctrl+C を安定して送る方法がないため、この経路は自動テストにしない。リリース前に実機で手動確認する(確認項目: 終了コード 130、ジョブの子プロセスが残らない、`watch` で中断したジョブが `error` で次が `pending`、`run -v` の表示に `interrupted by signal 2`)。手順は [11-quality-and-build.md](11-quality-and-build.md) の手動の受け入れテストにある。
- `Console.CancelKeyPress` の登録が、`Console` クラスの初期化(端末の設定の変更)を引き起こすかは、未確認。

### 7.7 `PipelineStoppedException` と Ctrl+C、`finally` での後始末

**事実**:

- `pwsh` は、Ctrl+C(UNIX 系では SIGINT と、Ctrl+Break の SIGQUIT)で、実行中のパイプラインの停止を要求する(確認。出典: PowerShell のソース `ConsoleHost.cs`。7.5・7.6 節)。停止は、実行中のスクリプトに、`System.Management.Automation.PipelineStoppedException` として届く(確認: 公式の `about_try_catch_finally` と、Stack Overflow の回答は、Ctrl+C がパイプラインを停止すると述べる。例外の型名そのものは、この文書では出典を確認していない。未確認)。.NET のメソッドの呼び出しの途中では割り込まれず、呼び出しが戻ったあとの、PowerShell の文の境目で届く(PowerShell の既知の挙動。この文書では出典を確認していない。未確認。`WaitForExit` を 10 ms 以下の刻みにする理由の 1 つ)。
- `try`/`finally` の `finally` は、Ctrl+C で停止したときも実行される(確認。出典: `about_try_catch_finally` と、検索で得た記事。`finally` の中で、パイプラインへの出力は表示されないことがある)。`catch` のブロックは、Ctrl+C の停止では実行されない、という報告がある(確認: フォーラムの報告のみ。公式の文書ではない。未確認)。7.0 の preview で、`finally` の途中で再び Ctrl+C を押すと、`finally` が中断される、という報告がある(確認: GitHub の issue の報告のみ。未確認)。
- `pwsh -File <script>` で、スクリプトが Ctrl+C で中断されたときの `pwsh` の終了コードは **0** である(確認。出典: `about_Pwsh` の `-File` の節の「when the execution is interrupted with Ctrl+C the exit code is `0`」)。公式の文書とソースの読み取り(`$global:?` に基づく)とで、記述が食い違う可能性がある(未確認)。したがって、**Ctrl+C のあとの終了コードを、`pwsh` の終了コードに頼らない**【規定】。

**規定(停止経路)**: Ctrl+C による停止と、ヘルパーの合図による(`pwsh` が止まらない)経路の、**両方**で、同じ結果(ジョブの終了を待ち、エラーとしてステートに記録し、`128 + 番号` で終わる)にする。

1. **待機ループの `finally`**(`Wait-ConductorChild`): 子の終了を観測せずに(`WaitForExit` が `$true` を返さずに)ループを抜けるとき(例外または停止)、`finally` で、次を行う。
   - 合図が立っていれば、転送が済んでいなければ、1 回だけ転送する(5.3 節)。合図がまだ立っていなければ、**最大 1 秒、10 ms ごとに** `$sig.IsRaised` を確かめて待つ(`pwsh` のハンドラとヘルパーのハンドラが別スレッドで動くため、停止が先に届き、合図が遅れて立つことがある)。1 秒待っても立たないとき、番号は 2(SIGINT / Ctrl+C)とみなす。
   - 子の終了を、`WaitForExit(10)` の繰り返しで待ち続ける(無期限の `WaitForExit()` を使わない)。
   - 実行結果(`Interrupted = $true`、4.4 節の `ExitCode`、`TerminatedBySignal`)を作り、呼び出し側が渡した進捗の入れ物(`-Progress`。`[ordered]` のハッシュテーブル)の `Execution` に入れる。
   - `Process` を `Dispose()` する。
   - この `finally` の中では、PowerShell のコマンドレット(`Write-Host`、`Start-Sleep` など)を呼ばず、.NET のメソッドと、この仕様の関数の呼び出しだけで済ませる(停止の状態で、コマンドレットが、再び `PipelineStoppedException` を出す可能性がある。未確認)。
   - 停止ではなく、通常の例外(`catch` で捕まえる想定外の内部の誤り)で抜けるときも、`finally` は同じ動作をする(子を待つ)。11 章の「ジョブが孤児として残りうる」の規定との関係: 通常の例外の経路では、`finally` が子を待たず(待たずに終わる)に、70 で終わる。**どちらにするか**: 停止(合図が立っている、または、`catch` で例外を記録していない)ときだけ待つ。`Invoke-ConductorJob` の内側の `catch [System.Exception]` で例外を受けたときは、`$stopped = $false` にして、待たずに再送出する。
2. **流れの制御の `finally`**(`Private/Cli/` の 1 ジョブの実行の関数。`ExecJob` に相当): 1 ジョブの「実行 → 記録」を `try { ... $completed = $true } finally { if (-not $completed) { ... } }` で囲む。`finally` で `$completed` が `$false` で、`$progress.Execution` があれば、通常の経路と同じ「分類 → 記録関数 → ステートの書き込み(Durable)」を行い、終了コードを `128 + 番号` として予約する(下の 3)。`$progress.Execution` がなければ(起動の前に停止したとき)、「起動しなかった中断」の記録(7.4 の表の最初の行と同じ)を行う。
3. **終了コードの予約と launcher**: 停止経路では、`Invoke-Conductor` から値が戻らない(例外が抜ける)ので、`pwsh` の終了コードは使えない(上の事実)。そこで、流れの制御の `finally` が、`[ConductorNative.UnixNative]::SetPendingExitCode(128 + 番号)` を呼び、launcher(`bin/conductor.ps1`。[02-architecture.md](02-architecture.md))の `finally` が、`HasPendingExitCode` が `$true` なら、`[System.Environment]::Exit(PendingExitCode)` で終わる。ヘルパーが読み込まれていない(`watch` など)場合は、launcher の `finally` は、`'ConductorNative.UnixNative' -as [type]` が `$null` のときに、130 で `[System.Environment]::Exit(130)` を呼ぶ。`Invoke-Conductor` が値を返して終わったとき(通常の経路)は、launcher は `exit $code` を呼ぶ。`[System.Environment]::Exit` を、モジュールの中では呼ばない(対話セッションごと終了するため)【規定】。launcher の `finally` の中で `Environment.Exit` が、停止の状態で働くかは、未確認(12 章の試験 T-4)。
4. **2 回目の Ctrl+C**: 後始末(`finally`)の途中で、利用者がもう一度 Ctrl+C を押すと、`finally` が中断されることがある(上の事実)。このとき、ステートは直前の完全な内容のままで、ジョブは孤児として残りうる(次の実行は、そのジョブを再実行しうる。at-least-once の範囲)。「転送は 1 回」の規定(2 回目のシグナルで追加の動作をしない)は、変わらない。この制約を、利用者向けの文書([11-quality-and-build.md](11-quality-and-build.md))に書く。

**正常な経路(`pwsh` が止まらない経路。SIGTERM、SIGHUP)**: 待機ループは、合図を読んで転送し、子の終了を観測して、通常どおり実行結果を返す。`finally` は、何もしない(子は回収済み)。

### 7.8 ヘルパーを使えない環境での縮退動作と、別案

| 状況 | 動作【規定】 |
|------|-------------|
| `Add-Type` が失敗する(CLM、コンパイルの失敗、別の改訂の型が残っている) | `run`/`pipeline` は、標準エラーに `conductor: error: failed to set up signal handling` を書き、終了コード 70(7.4 節)。ジョブは起動しない。**シグナルを受けずに、ジョブを起動することはしない**(中断の規定を満たせないため、縮退して続行する案は採らない) |
| `watch`、`reset`、`validate`、`convert`、`--help`、`--version` | ヘルパーを使わないので、影響を受けない(CLM では 7.3 節の別の理由で失敗しうる) |
| `libc` が見つからない(`Kill` が `false`、`SyncDirectory` が何もしない) | `Add-Type` は成功しているので、`run`/`pipeline` は続く。転送は行われず、ディレクトリの同期は行われない(06 章の「失敗は無視」の規定の範囲)。受け入れテストの「転送」の項目が、3 OS で、このことを検出する |

**別案(未採用。起動の所要時間が、T-2 の計測で許容できない場合に再検討)**: C# のソースを、ビルド時にコンパイルして `Conductor.Native.dll`(どの OS でも動く管理コードのアセンブリ)を作り、`bin` の隣に置いて、`Add-Type -LiteralPath` で読み込む。コンパイルの所要時間が要らなくなる。欠点: ビルドに .NET SDK(開発時の依存)が要る。CLM でも、署名されたアセンブリは読み込める(確認。出典: `about_Language_Modes`)が、署名が要る。この版は、実行時の依存を増やさず、ビルドを PowerShell だけで完結させるため、インラインの `Add-Type` を採る。

### 7.9 合図が立つ時点ごとの動作(流れの制御)

| 合図が立った時点 | 動作 | ステートの記録 | 終了コード |
|------------------|------|----------------|------------|
| ジョブの起動の呼び出しの前(`Started = $false`) | ジョブを起動しない | 分類なし。エラーとしては記録しない。再開位置はそのジョブ | `128 + 番号` |
| ジョブの実行中 | 1 回転送(UNIX 系)。ジョブの終了を待つ | そのジョブを**エラー**として記録(ジョブが 0 で終わっても) | `128 + 番号` |
| ジョブの終了の観測の直後(転送が間に合わなかった場合を含む) | 起動済みで `Interrupted = $true`(上の行と同じ扱い) | エラー | `128 + 番号` |
| `pipeline` のジョブとジョブのあいだ(ジョブの結果は成功か警告で、完了ではない) | 次のジョブを起動しない | 記述順で、まだ起動していない最初のジョブ(エラーのジョブが先にあればそれ)を再開位置にして、中断を記録。エラーとは記録しない。`-v` で `interrupted by signal N` | `128 + 番号` |
| `pipeline` の最後のジョブが完了した直後 | 完了の処理(`--remove-on-complete` ならステートを削除)をして終わる | 完了 | 0(合図は無視される) |
| リトライの待機中 | 待機を打ち切る。`Started = $false`、`Interrupted = $true`、`SignalNumber = 番号` の実行結果を作る。次の実行はしない | 分類なし、再開位置はそのジョブ。エラーとしては記録しない | `128 + 番号` |
| 定義の検査・ステートの読み込み・照合の途中 | 検査は最後まで進める | (定義の誤り 64、ステートの誤り 65、拒否 66、スキップ 0 の終了コードが、中断の終了コードより優先する) | 各検査の値 |

最後の行の補足(PowerShell 固有): この段階で Ctrl+C により `PipelineStoppedException` が届いたとき(7.7 節の停止経路)は、各検査を最後まで進められない(スクリプトが止まる)。ジョブは起動していないので、ステートにも触れず、launcher が 130 で終わる。この食い違いは、許容する(中断の終了コードが優先されない場合がある)。

---

## 8. リトライの待機(`Wait-ConductorRetry`)

「待機(秒数, 合図)」→ 「経過」または「中断(最初の番号)」。OS 固有のコードを持たない。

```text
# 戻り値(pscustomobject): Kind = 'Elapsed' | 'Interrupted'、Interrupted のとき SignalNumber ([int])
function Wait-ConductorRetry {
    param([uint64]$Seconds, $Interrupt,
          [scriptblock]$Timestamp,   # 既定: [System.Diagnostics.Stopwatch]::GetTimestamp()
          [scriptblock]$WaitRaised)  # 既定: $Interrupt.Wait(ms)
}
```

- 手順: 開始時刻を `[System.Diagnostics.Stopwatch]::GetTimestamp()` で取る(単調な時計。壁時計の `Get-Date`、`[DateTime]::UtcNow` は使わない。時刻の変更に左右されない)。ループで ① `$sig.IsRaised` が `$true` なら、すぐ `Interrupted(最初の番号)`。② 経過が `Seconds` 以上なら `Elapsed`。③ 残りの時間と、上限(250 ms)の小さい方だけ、`$sig.Wait(ms)` で待つ。待ちの途中で合図が立てば、`Wait` が直ちに戻る。`Start-Sleep` を使わない(コマンドレットで、PowerShell の停止の挙動に依存し、合図で起こせない)【規定】。
- 合図は、待機の開始時と、`Wait` が戻るたび(最大 250 ms ごと)に確かめる。長い待ちでも、合図から **1 秒以内**に返る【規定】(例: 3600 秒の待機に、200 ms 後に合図を立てて、全体が 1.2 秒未満)。`Trigger` が `ManualResetEventSlim` を起こすので、通常は即時に返る。上限 250 ms は、起こし損ねに備える保険である(【目安】)。**PowerShell 固有**: 1 周ごとに PowerShell の文を実行するが、250 ms 刻みなので、負荷は小さい。
- 残りの時間の計算(整数。`[double]` を使わない): 数値の変換は [05-execution-flow.md](05-execution-flow.md) の式を正とする。この文書の規定は、次のとおり。
  - `$target = [bigint]$Seconds * [bigint][System.Diagnostics.Stopwatch]::Frequency`。`[bigint]` で計算し、64 ビットの範囲を超えても、例外にしない(`[uint64]` の積は、`[uint64]` に収まらないとき、例外になるか `[double]` に昇格するかが、公式の文書で確かめられていない。未確認。したがって、`[uint64]` の演算に頼らない)。`$Seconds` は `[uint64]` で受け取る(最大値 `[uint64]::MaxValue` でも、`[bigint]` に変換して計算できる)。
  - `$elapsed = [bigint]([System.Diagnostics.Stopwatch]::GetTimestamp() - $start)`。
  - `$remaining = $target - $elapsed`(`[bigint]`)。0 以下なら `Elapsed`。
  - 残りミリ秒: `$remainingMs = ($remaining * 1000 + $frequency - 1) / $frequency`(`[bigint]` の切り上げ除算。`[bigint]` どうしの `/` は整数の商)。
  - 1 回の待ちは、`$chunkMs = [int][bigint]::Min($remainingMs, [bigint]250)`(1〜250)。`[int]` へのキャストは、`[bigint]` が 250 以下であることを確かめてから行う(PowerShell の `[int]` への数値のキャストは丸める。丸めを使わない整数なので影響しない)。
  - `[TimeSpan]::FromSeconds(...)` と、秒から `[TimeSpan]` を作る呼び出しは使わない(範囲外で例外になる)。
- 秒数が 0 のときは、合図が立っていなければ、すぐ `Elapsed`。合図が立っていれば `Interrupted`。
- 複数の合図は、最初の番号を返す。
- 待機の上限: 待機の合計は、`Seconds` で決まり、`conductor` は上限を持たない(利用者の指定が、飽和した 64 ビットの上限でも、待つ。ただし、1 つの待ちは、250 ms 以下に刻まれる)。時計(`Timestamp`)と待ち(`WaitRaised`)は、差し替え可能なスクリプトブロックで受ける(単体テストが、実際の待ちなしで、経過と中断を再現する)。

### 8.1 シグナル登録の寿命と復元

- 本番: 登録は、プロセスの終了まで有効(`Stop` を呼ばない)。ヘルパーの静的なフィールドが、参照を保つ。
- 単体テスト: `[ConductorNative.SignalWatcher]::Stop()` で、登録を元に戻し、静的な印を戻す。テストの終わりに、登録が残っていないこと(次のテストが、`Started` を得られること)を確かめる。
- テストで、実際のシグナルを使うとき、テストを動かす `pwsh` 自身に `kill` で送るのは、テストのプロセス(Pester)を巻き込むため、避ける。実際の `conductor` の実行(`bin/conductor` を別のプロセスとして起動)を使う受け入れテストで行う(11 章)。

---

## 9. プラットフォーム差と OS 分岐の規則

### 9.1 差の一覧

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
| `pwsh` の Ctrl+C の処理 | `Console.CancelKeyPress`(SIGINT、SIGQUIT)で、スクリプトを停止 | コンソールの制御ハンドラで、スクリプトを停止 |
| 自動の受け入れテスト | 11 章の「シグナルと中断」(UNIX 系) | Windows 専用の項目と、手動の確認 |

### 9.2 分岐を置く場所の規則

OS ごとの分岐と OS 固有の呼び出しは、プラットフォーム層(OS 固有の処理をまとめた 1 か所)の外にない【規定】。

- 「OS の判定」(`[System.OperatingSystem]::IsWindows()`、`IsLinux()`、`IsMacOS()`、`$IsWindows`、`$IsLinux`、`$IsMacOS`、`[System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform`、`[System.Environment]::OSVersion`、`[System.IO.Path]::DirectorySeparatorChar` を OS の判定に使うこと)は、`Get-ConductorPlatformKind` の 1 か所だけ(`'Unix'` または `'Windows'` を返す)。ほかの関数は、`$Platform`(`'Unix'`/`'Windows'`)の値を引数で受け取る。
- `Add-Type` と P/Invoke(`DllImport`、`NativeLibrary`、`Marshal.GetDelegateForFunctionPointer`)は、`Private/Runner/NativeHelper.ps1` の中の、1 か所だけ。ほかでは使わない。
- OS ごとの処理は、`Get-ConductorStartInfo`、`Send-ConductorSignal`、`Read-ConductorExit`、`Sync-ConductorDirectory` の中の `if ($Platform -eq 'Windows')` の分岐で行う(実行時の判定。3 OS のテストを、同じコードで行い、Linux の 1 台から Windows の純粋な部分を単体テストするため)。
- ネットワークに接続しない(実装がネットワークの機能を持たない)。`System.Net` の名前空間、`Invoke-WebRequest`、`Invoke-RestMethod` を使わない。
- 製品のコードは、外部の入力(引数・環境・ファイル・OS の結果)に対して、異常終了する書き方をしない(失敗は値で返す)。
- 点検: `tests/` の点検テストが、`src/` の `.ps1` を、`[System.Management.Automation.Language.Parser]::ParseFile` で構文木にして走査し、上の判定の API の名前が、`Get-ConductorPlatformKind` のファイル以外に、`Add-Type` と `DllImport` が `NativeHelper.ps1` 以外に、`Start-Process`、`Invoke-Expression`、`Write-Host` が、どこにも現れないことを確かめる(11 章。[02-architecture.md](02-architecture.md) の AST の走査と同じ道具)。

---

## 10. スレッドとコールバックの規則、順序の保証

### 10.1 同期モデルの規則【規定】

`conductor` の流れは、`pwsh` のメインのスクリプトのスレッド(`Invoke-Conductor` を呼んだスレッド)で、すべて**同期**で動く。子プロセスの終了、シグナル、時間の経過は、メインのスレッドの `WaitForExit(int)` と `Interrupt.Wait(int)` の待ちと、**ランタイムが別のスレッドで動かす C# のコールバック**(シグナルのハンドラ)が立てるフラグで伝わる。

| 規則 | 内容 |
|------|------|
| スレッドの作成 | `Start-Job`、`Start-ThreadJob`、`ForEach-Object -Parallel`、`[System.Threading.Thread]::new`、`[System.Threading.Tasks.Task]::Run`、`Start-Process`、`Register-ObjectEvent`、`Register-EngineEvent`、`New-Runspace` / `[runspacefactory]` を、製品のコードで使わない(子の回収と `pid` の扱いを、メインのスレッドの待機ループ 1 か所に集めるため。コールバックに PowerShell のスクリプトブロックを渡す問題は 7.1 節) |
| スクリプトブロックのデリゲート化 | `[System.Action]`、`[System.Func[...]]`、`[System.EventHandler]` などへの、スクリプトブロックの変換を、別スレッドから呼ばれうる場面で使わない。(同じスレッドの中でだけ呼ばれる `Deps` のスクリプトブロックは、デリゲートに変換しない: `& $scriptblock` で呼ぶ) |
| 子の終了の通知 | `Process.Exited` イベント(`EnableRaisingEvents`)を使わない(別スレッドで動くため) |
| シグナルのコールバック | C# ヘルパーの中のコールバック(`PosixSignalRegistration` のハンドラ、`Console.CancelKeyPress` のハンドラ)だけ。メインのスレッドとは別のスレッド(専用の背景スレッドまたはスレッドプール)で動く(確認。出典: 7.5 節と 7.6 節)。ハンドラの中は、`ctx.Cancel = true`(または `e.Cancel = true`)と `Interrupt.Trigger` を呼ぶだけ。出力、ファイルの書き込み、ロック、`Process` の操作、`kill`、ブロックする待ち、例外の送出をしない |
| 共有する値 | `Interrupt`(`Interlocked` と `ManualResetEventSlim`。6 章)と、シグナル登録の印(`Interlocked`)、予約された終了コード(7.7 節。`Interlocked` / `Volatile`)だけ。`lock` を使わない |
| 可視性 | 複数のスレッドから読み書きする値は、C# ヘルパーの中で、`Interlocked`、`Volatile.Read`、`Volatile.Write`、`ManualResetEventSlim` のいずれかを通す。PowerShell の変数を、スレッド間で共有しない |
| 待ちの刻み | 待機ループは 10 ms 以下、リトライの待機は 250 ms 以下で刻む。ビジーループ(待ちなしの `while`)を書かない |
| 合図の読み取り | 1 つの判断のなかで、`IsRaised` と `Number` を、1 回ずつだけ読み、その値を使う(6 章) |
| 例外 | コールバックの中で、例外を送出しない(未処理の例外は、プロセスを終了させる。確認: .NET の一般的な規則。この文書では出典を確認していない。未確認) |
| `Console` の入力 | `Console.In`、`Read-Host`、`[Console]::ReadKey` などを使わない(端末の設定を変えうる。未確認) |
| `$using:` と `$script:` | スレッドをまたいで使わない(スレッドを作らないので、起こらない) |

### 10.2 順序の保証【規定】

| 保証 | 内容 |
|------|------|
| 出力の順序 | 標準出力・標準エラーへの書き込みは、`Private/Report/` の窓口の 1 か所を通し、**書くたびに flush する**。標準出力と標準エラーの書き込みの順序が、呼び出しの順序と一致する。`Write-Host`、`Write-Output`、`Write-Error` を使わず、`[Console]::Out`、`[Console]::Error` に書く(詳細は [08-output-and-i18n.md](08-output-and-i18n.md)) |
| 子の出力との順序 | `conductor` は、子を起動する前に、自分の出力を flush して書き切る(4.1 節の手順 5)。子の実行中に、リトライの知らせ以外の出力を書かない。子の出力(子が直接ファイル記述子へ書く)と、`conductor` の出力が、混ざらない。リトライの知らせは、子の終了のあと、次の起動の前に、書いて flush する |
| ステートの順序 | ステートの書き込み(一時ファイルへの書き込み、同期、名前の付け替え、必要ならディレクトリの同期)の完了を待ってから、次のジョブを起動する。「記録して(Durable)から次へ」を破らない([05-execution-flow.md](05-execution-flow.md)、[06-state-file.md](06-state-file.md)) |
| 出力とステートの順序 | 05 章の順序に従う。ジョブごとに、`-v` の結果行などの表示を書き(flush し)、そのあとステートを書く。順序を入れ替えない |
| ハンドラの副作用 | シグナルのハンドラは、出力もステートの書き込みもしない(10.1 節)。したがって、ハンドラが出力・ステートの順序を乱すことはない |
| 終了の前の flush | `Invoke-Conductor` が終了コードを返す前に、標準出力と標準エラーを flush する。flush の失敗(`IOException`。閉じたパイプなど)は、出力の書き込みの失敗として扱う(08 章)。終了は、launcher の `exit` で行い、`Invoke-Conductor` の中で `exit` を呼ばない(7.7 節の停止経路の launcher の `Environment.Exit` を除く) |

### 10.3 スレッドの所有者・終了条件の表【規定】

| スレッド/コールバック | 起こす者(所有者) | 数 | 読む | 書く | 終わる条件 |
|-----------------------|-------------------|-----|------|------|------------|
| メインのスクリプトのスレッド | `pwsh`(`-File` で起動したスクリプトの実行) | 1 | すべて(定義、ステート、出力、`Interrupt`) | すべての出力とステートの書き込み | スクリプトが終わる(または停止する) |
| `pwsh` の Ctrl+C のハンドラのスレッド | `pwsh`(`HandleBreak`。確認: 7.5 節の出典) | Ctrl+C のたび | OS の Ctrl+C(SIGINT / SIGQUIT / Windows のコンソールイベント) | メインのスクリプトの停止の要求 | ハンドラが戻る。`conductor` は所有しない。停止の結果は 7.7 節で扱う |
| シグナルのコールバック(UNIX 系: SIGINT・SIGQUIT・SIGTERM は専用の背景スレッド、SIGHUP はスレッドプール) | ランタイム(`PosixSignalRegistration`。登録は C# ヘルパーの `SignalWatcher`) | シグナルごとに、到着のたびに(同時に複数ありうる) | OS のシグナル | `Interrupt.Trigger` だけ(と `ctx.Cancel`) | コールバックが戻る。登録は、プロセスの終了まで有効(本番で `Stop` は呼ばない。テストでは `Stop`) |
| `Console.CancelKeyPress` のコールバック(Windows) | ランタイム(スレッドプール。登録は `SignalWatcher`) | 到着のたびに | OS の Ctrl+C | `Interrupt.Trigger` だけ(と `e.Cancel`) | コールバックが戻る。登録は、プロセスの終了まで有効 |
| 子プロセス | `Invoke-ConductorJob` | 実行中のジョブごとに 1(同時に最大 1) | 子の終了(`WaitForExit(int)`、`ExitCode`) | `Invoke-ConductorJob` の内側の状態だけ | `WaitForExit` が `$true` を返す(または起動の失敗)まで。`Invoke-ConductorJob` が戻る時点で、子は回収済みである(例外: 7.7 節の 2 回目の Ctrl+C) |
| 待ち(`WaitForExit(int)`、`ManualResetEventSlim.Wait`) | `Wait-ConductorChild` と `Wait-ConductorRetry` | 同時に最大 1 | 時間の経過、`Interrupt` | なし | 経過、または合図の確認で打ち切る |
| ランタイム内部のスレッド(子の回収、スレッドプール) | .NET ランタイム | ランタイムが決める | `SIGCHLD` | ランタイムの内部の状態 | `conductor` は所有しない。直接使わない |

- 出力(標準出力・標準エラー)とステートへの書き込みは、流れの制御(`Private/Cli/`)の 1 本の呼び出しの連なり(メインのスレッド)だけが行う。シグナルのコールバックは、それらに触れない。
- リソースのリークがないこと: テストの終わりに、シグナルの登録が元に戻っている、`Process` が `Dispose` されている、子プロセスが残っていない。

---

## 11. 想定外の内部の誤り

- 実行の入口(`Invoke-Conductor`)で、想定外の内部の誤り(実装の不具合による例外)を捕まえる仕組みを、**最初に**入れる。内部の誤りが起きると、標準エラーに固定の英語の 1 行 `conductor: error: internal error`(ロケールを判定しない)を書き、終了コード **70** を返す。標準エラーへの書き込みが失敗しても 70 を返す。【規定】
- 仕組み: `Invoke-Conductor` 全体を `try { ... } catch [System.Exception] { <# 1 行を書いて 70 を返す #> }` で囲む。`catch` の内側で、例外の内容(メッセージ、スタックトレース)を、標準エラーにも標準出力にも出さない(`$_.Exception.Message`、`ToString()`、`$_ | Out-String` を呼ばない)。固定のバイト列(`conductor: error: internal error\n` の UTF-8)を、`[Console]::Error` ではなく、`[Console]::OpenStandardError()` のストリームに 1 回書く(`try { ... } catch [System.IO.IOException] { }` で囲み、失敗は無視する。エンコーディングと flush の設定に左右されないため)。繰り返し呼ばれても 1 行だけ出す(静的な印)。
- `catch [System.Exception]` は、`PipelineStoppedException` を捕まえない場合がある(7.7 節。未確認)。停止は、内部の誤りではなく、7.7 節の経路で扱う。`catch` が、停止の例外を捕まえてしまう場合に備え、`catch` の中で、例外の型が `System.Management.Automation.PipelineStoppedException` なら、`throw`(同じ例外の再送出)して、70 にしない。
- **`trap` を使わない**(`trap` は、スコープと `continue` の挙動が複雑で、停止の扱いと混ざるため)。**`$ErrorActionPreference = 'Stop'`** が、モジュールの先頭にある([02-architecture.md](02-architecture.md))ので、非終了エラーも終了エラーになり、`catch` に届く。
- `StackOverflowException` は、捕まえられない(プロセスが終了する)。したがって、深い入れ子の入力(定義・ステート・変換)を、再帰で処理しない(明示的なスタックを使う。03 章)。`command` の分割は、再帰を使わない(3 章)。PowerShell の関数の再帰は、深さの上限の規定がなく、深いと、`pwsh` のプロセスが終了する(未確認)。`OutOfMemoryException` は、捕まえて続行しない(`catch [System.Exception]` が捕まえた場合は 70 にする。回復を試みない)。
- 入口は、流れの結果の終了コードを**返す**(`exit` を呼ばない)。launcher が `exit` を呼ぶ。入口に、わざと例外を起こすコマンドを持たない。内部の誤りの検査は、`Invoke-ConductorMain` に、例外を送出する `Deps` を渡す単体テスト、または別のテスト用のスクリプト(`tests/jobs/`)で行う([10-errors-and-exit-codes.md](10-errors-and-exit-codes.md))。
- launcher 側の規定(詳細は [02-architecture.md](02-architecture.md)): launcher の最上位も `try`/`finally` で囲み、`Invoke-Conductor` が例外で抜けた(停止でもない)場合も、固定の 1 行を出して 70 で終わる。`pwsh` 自身の異常終了(読み込みの失敗など。`pwsh -File` の終了コードは、終了エラーのとき 1。確認: `about_Pwsh` の `-File` の節)は、launcher の外側のため、70 にならない。
- ジョブの実行中に内部の誤りが起きると、`Process` の `Dispose` が行われても、子プロセスの終了を待たずに終わり、ジョブが孤児として残りうる(`Process.Dispose` は子を終了させない)。ステートは直前の完全な内容のままで、次の実行はそのジョブを再実行しうる(少なくとも 1 回の実行の範囲)。
- `conductor` が SIGKILL などで強制終了されたときも、ジョブは残って動き続けうる(検知も停止もしない)。

---

## 12. 検証の観点と、実装時に最初に確かめる試験

### 12.1 受け入れテストとの対応

この文書の挙動を確かめる受け入れテストは、[11-quality-and-build.md](11-quality-and-build.md) にある。

| 確かめること | 11 章の節 |
|--------------|-----------|
| 分割の全規則(3 章の表)、UTF-16 のサロゲートの組、U+3000 などが区切りでないこと、PowerShell の構文を解釈しないこと | 「`command` の分割と起動」(受理・拒否の表) |
| メタ文字が引数にそのまま渡る、配列の要素がそのまま渡る(空の引数を含む)、環境変数と作業ディレクトリの引き継ぎ、起動失敗(見つからない・実行権限なし・ディレクトリ)、終了コード(0・3・255・300) | 同上 |
| UNIX 系で、カレントディレクトリの同名ファイルが、`PATH` より先に起動されないこと(4.9 節) | 同上(UNIX 系) |
| 起動前の合図で起動しない、終了の観測の直後の合図で `Interrupted` | 「シグナルと中断」(起動の部品の単体検査) |
| SIGTERM / SIGINT の転送、許可外の番号は送らない、転送は 1 回だけ、先に終わったジョブは転送なし | 「シグナルと中断」(UNIX 系) |
| 「新しいプロセスグループ」の試験は、この版では行わない(4.7 節) | 「シグナルと中断」に注記 |
| 待ちの間隔(1・2・4・8・10 ms)、リトライの待機の 1 秒以内の打ち切り | 「シグナルと中断」「リトライ」「非機能要件」(性能) |
| 実バイナリで SIGTERM → 143、ステートに再開位置、孤児なし。HUP のあとの INT → 129 | 「シグナルと中断」 |
| SIGINT(`kill -INT`)で `pwsh` が止まる経路でも、ステートにエラーが記録され、終了コードが 130 になる(7.7 節) | 「シグナルと中断」 |
| `kill -9 $$` のジョブ: `terminated by signal 9`、ステートに終了コードなし(4.4 節の推定の確認) | 「シグナルと中断」 |
| 監視の登録(最初の番号が残る、2 回目の登録は `AlreadyRegistered`、`Stop` で元に戻る) | 「シグナルと中断」(単体検査) |
| 内部の誤り → 70 | 「内部の誤り」 |
| `Add-Type` の失敗 → 70(`failed to set up signal handling`)。ヘルパーの失敗を再現する偽の `Deps` | 「シグナルと中断」(注入) |
| Windows の Ctrl+C | 「シグナルと中断」の手動の項目 |

### 12.2 実装時に最初に確かめる小さな試験(未確認事項の一覧)

実機の `pwsh`(最小の対象版と最新の安定版、3 OS)で、次を、最初に確かめる。結果で、この文書の該当の箇所を直す。

| # | 試験 | 確かめること | 結果で変わる箇所 |
|---|------|--------------|------------------|
| T-1 | `pwsh -File` のスクリプトが、ヘルパーの SIGINT のハンドラを登録して `Wait` で待つ。`kill -INT <pid>` を送る | (1) ヘルパーの `IsRaised` が立つか。(2) 同時に `PipelineStoppedException` で止まるか(`finally` が走るか)。`Cancel = true` で `pwsh` のハンドラが抑えられるか | 7.5・7.7 節(どちらでも動く設計だが、実際の経路を記録する) |
| T-2 | 空の C# ヘルパーと、実際のヘルパーの `Add-Type` の所要時間。`-ReferencedAssemblies` なしで、参照が解決するか | 起動のコスト、参照の指定の要否 | 7.3 節、11 章の startup-bench |
| T-3 | `NativeLibrary.TryLoad` の候補(`libc.so.6`、`libc`、`/usr/lib/libSystem.B.dylib`)で、`kill`、`open`、`fsync`、`close` が解決する。arm64 の macOS で、可変長引数の `open` を 2 引数で呼べる | P/Invoke の方法 | 7.2 節 |
| T-4 | `try { ... } finally { [System.Environment]::Exit(7) }` が、Ctrl+C による停止のあとに、終了コード 7 で終わるか | launcher の停止経路 | 7.7 節の 3 |
| T-5 | `pwsh -File x.ps1 -- a -v` と `pwsh -File x.ps1 a -v` で、`$args` に `--` と `-v` がそのまま入るか。配列のスプラット(`f @args`)で、`-v`、`--`、`-?`、`-Verbose` が、関数のパラメータとして解釈されず、文字列のままか | 引数の受け渡し | [02-architecture.md](02-architecture.md) の `Invoke-Conductor` |
| T-6 | `catch [System.ComponentModel.Win32Exception]` が、`Process.Start` の失敗を捕まえるか(`MethodInvocationException` に包まれるか) | 起動の失敗の捕まえ方 | 4.3 節 |
| T-7 | `catch [System.Exception]` が、Ctrl+C の停止を捕まえるか。`finally` の中の関数の呼び出しと .NET の呼び出しが、停止のあとに動くか | 停止経路の `finally` の規定 | 7.7 節、11 章 |
| T-8 | `$child.WaitForExit(10)` の最中の Ctrl+C が、呼び出しの戻りのあとに届くまでの遅れ | 刻みの値 | 5 章 |
| T-9 | 約 30 万引数の `command` の分割にかかる時間。`List[string]` への追加と `StringBuilder` の速度 | 性能の目安 | 3.2 節 |
| T-10 | 子の環境変数が、`conductor` の起動前の環境と等しいか(`pwsh` が足す変数の一覧) | 環境の引き継ぎ | 4.2 節 |
| T-11 | CLM でのヘルパーの失敗の様子(AppLocker または WDAC のある Windows) | 縮退の動作 | 7.3・7.8 節 |

### 12.3 他の担当と合わせる点

- 4.7 節: UNIX 系で、ジョブを**新しいプロセスグループに置かない**(論理モデルの「新しいグループで動く」とは異なる。`System.Diagnostics.Process` で実現できないため)。受け入れテストの「新しいプロセスグループ」を、この版は行わない。端末の Ctrl+C はジョブに直接届く。
- 7.2 節: `Add-Type` のヘルパーのソースは `Private/Runner/NativeHelper.ps1`。`kill` と、ステートのディレクトリの `fsync`(`SyncDirectory`)は、同じヘルパーに置く。`State` は、`Runner` に依存せず、`Deps` の `SyncDirectory` を、`Cli` が渡す(06 章の担当と合わせる)。
- 7.7 節: 停止経路で、`Invoke-Conductor` から値が戻らないため、launcher(`bin/conductor.ps1`)が `finally` で `[System.Environment]::Exit` を呼ぶ。`pwsh -File` の Ctrl+C のあとの終了コードは 0(公式の文書)。launcher の規定は 02 章で定める。
- 4.5 節: `JobExecution` は `[pscustomobject]`(`class` を使わない)。05 章の担当と合わせる。
- 1 ジョブの実行の関数が、`-Progress`(進捗の入れ物)を受け取り、停止経路で `Execution` を返す(05 章、02 章の流れと合わせる)。
- 8 章: `Wait-ConductorRetry` の数値の計算は `[bigint]` で行う(05 章の式を正とする)。
- 3.2 節: 文字の比較は符号位置の整数で行う(`[char]` の `-eq` に頼らない)。
- PSScriptAnalyzer の設定(11 章の担当): 純粋な関数に `New-`/`Set-`/`Reset-` の動詞を使うため、`PSUseShouldProcessForStateChangingFunctions` を無効にする([02-architecture.md](02-architecture.md))。`Add-Type` の許可箇所は `NativeHelper.ps1` だけ。

---

## 13. 未確認の事項(まとめ)

- この環境には `pwsh` も .NET SDK もなく、骨格のコンパイル、`Add-Type` の実行、シグナルの実機の動作を確かめていない。12.2 節の試験を、実装時に最初に行う。
- `pwsh` の SIGINT・SIGQUIT の停止と、ヘルパーの `PosixSignalRegistration` の `Cancel` の関係(T-1)。`pwsh` の SIGTERM・SIGHUP の処理(読んだソースの範囲では見当たらなかったが、ファイルの全体を読んでいない)。
- `PipelineStoppedException` の型名と、`catch` が停止を捕まえるか(フォーラムの報告のみ)。停止のあとの `finally` の中の関数の呼び出し、`Environment.Exit`(T-4、T-7)。`pwsh -File` の Ctrl+C のあとの終了コードの、文書とソースの食い違いの可能性。
- `Add-Type` のコンパイルの所要時間と、参照アセンブリの要否(T-2)。`NativeLibrary` の候補の名前と arm64 の macOS の可変長引数(T-3)。
- `Win32Exception` が `MethodInvocationException` に包まれるか(T-6)。
- `[char]` どうしの `-eq` の大文字小文字の扱い(符号位置の整数で比べるので影響しない)。
- 環境変数の引き継ぎで `pwsh` が足す変数、プロセスの作業ディレクトリと `$PWD` の関係(T-10)。
- Windows の `.cmd` / `.bat` の起動(`cmd.exe` の引数の再解釈、`Arguments` の再引用の有無)、Ctrl+C の終了コード 130、`Ctrl+Break`。実機で未確認。
- UNIX 系で、ジョブが `conductor` と同じプロセスグループにあるときの、端末の入力の扱いと、Ctrl+C の 2 回の配送の実機での確認。
- `File.GetUnixFileMode` の API の存在の出典の確認、`pid` の再利用の競合の実際の確率。
- 最小の対象版の `pwsh` が載せる .NET の版と、`PosixSignalRegistration` の動作の版ごとの差(7.4 は .NET 8。他は実装時に確認)。
- この仕様は、公式のドキュメントと、PowerShell・.NET の公開ソースの一部の読み取りに基づく。テストやビルドは実行していない。
