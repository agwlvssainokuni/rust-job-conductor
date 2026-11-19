# 03. 定義ファイルの書式と検証

この文書は、`conductor` の定義ファイルの書式と、読み込み時の検証を定める。この文書だけで、PowerShell でゼロから、同じ受理・拒否・エラー(種類、文言、行番号)を返す読み込み処理を実装できることを目標とする。

- 定義ファイルの書式は **`.psd1`(PowerShell データファイル。主)** と **JSON(副)** の 2 つ。書式は**拡張子**で決める(第 2 節)。既定のファイル名は `conductor.psd1`。人が書く定義にコメントが要るときは、`.psd1` を使う(JSON はコメントを書けない)。他の書式は扱わない。
- 検証は、見つけた誤りを**まとめて**返す。ただし、早い段階の誤り(取り込み、文法、展開と上限、版)は 1 件だけで打ち切る(第 2 節)。
- 定義の内容は、読み込みでは実行も評価もしない。`.psd1` はコードを**実行しない**: パーサーで構文木(AST)を作り、**許可する構文だけを自前で検査して値に変換する**(第 4 節)。`command` の文字列は、そのまま保持する。
- 確認できなかった点は「未確認」と書く。公式ドキュメントや公開されたソースの記述で確かめた点は「確認(出典: …)」と書く。この環境には PowerShell(`pwsh`)も .NET SDK も無く、実機では動かしていない。したがって「確認」は、**文書・ソースの記述の確認**であり、動作の確認ではない。実装時に最初に行う小さな試験を第 16 節に一覧にする。
- 「必須」と書いた挙動は、利用者から見える結果(受理/拒否、エラーの種類・文言・行番号、取り込み後の値)であり、同じにしなければならない。「任意」「実装に任せてよい」と書いた点は、利用者から見える結果が変わらない範囲で、実装が選んでよい。
- このツールの終了コード、エラー行の標準エラーへの出し方は [10. エラーと終了コード](10-errors-and-exit-codes.md)、メッセージの言語の選び方と標準出力・標準エラーの文字コードは [08. 出力と多言語対応](08-output-and-i18n.md) に従う。この文書は、エラーの内容(種類・位置・理由)までを定める。
- `.psd1` と JSON の読み取り(第 4〜6 節)は、[09. 変換器](09-converter.md)と共有する。実装は、`src/Conductor/Private/Json/`(自前の JSON の読み書き)と `src/Conductor/Private/Definition/`(木の型、`.psd1` の読み取り、上限、検査)に「木の読み取り」の部分を置き、`src/Conductor/Private/Convert/` はその関数を使う([02. アーキテクチャ](02-architecture.md))。ファイル名は任意。
- 実装は、モジュールの先頭で `Set-StrictMode -Version Latest` と `$ErrorActionPreference = 'Stop'` を設定する。読み込み処理は、利用者の入力の誤りを**例外ではなく戻り値**(結果のオブジェクト)で返す。例外は、実装の不具合(想定外の内部失敗)にだけ使う。ただし、.NET や PowerShell のパーサーが投げる例外は、第 3、4 節のとおり、読み込み処理の内側で捕まえて、定められた種類の戻り値にする。
- 画面への出力は、この文書の処理では行わない(`Write-Host`、`Write-Output`、`Write-Error` を使わない)。関数の戻り値に出力が混ざる事故を避けるため、戻り値は `return` で明示し、コマンドレットの出力を `$null = ...` か `[void]` で捨てる。
- 文化に依存する API(`ToString()` の引数なし、`Parse` の既定、`ToUpper()`、`-eq` などの比較演算子、`switch`)は、この文書の処理では使わない。数の書式化と解析は `[System.Globalization.CultureInfo]::InvariantCulture`、文字列の比較は `[string]::Equals(a, b, [System.StringComparison]::Ordinal)` か `-ceq` を使う。

## 1. 記法と用語

| 語 | 意味 |
|----|------|
| 文字 | Unicode のコードポイント(スカラー値。サロゲートを除く)。「N 文字」と書いたものは、この単位で数える(UTF-8 のバイト数でも、書記素でも、`string.Length`(UTF-16 のコード単位)でもない)。コードポイントの数は、`[System.Globalization.StringInfo]` ではなく、`char` を走査して上位サロゲート(U+D800〜U+DBFF)の直後の下位サロゲートを 1 つにまとめて数える自前の関数で求める。 |
| コード単位 | `[char]`(UTF-16 のコード単位。サロゲートの前半・後半は別の `[char]`)。位置の列(第 11.1 節)は、この単位で数える。 |
| 空白を取り除く(trim) | 文字列の先頭と末尾から、Unicode の White_Space 属性を持つ文字(U+0009〜U+000D、U+0020、U+0085、U+00A0、U+1680、U+2000〜U+200A、U+2028、U+2029、U+202F、U+205F、U+3000)を取り除く。「空白だけ」は、取り除いた結果が空文字列になること。**判定は、この列挙(25 文字)を持つ自前の表(`HashSet[int]`)で行い、`String.Trim()`・`Char.IsWhiteSpace`・`String.IsNullOrWhiteSpace` を製品のコードで使わない**(決定。Unicode のデータの版による集合の変化に依存させないため)。単体テストで、全 `[char]`(U+0000〜U+FFFF)に対して、自前の関数が、この列挙の 25 個だけを真にすることを確かめる(参考として `[char]::IsWhiteSpace` との一致も記録する。未確認: 実機での結果)。 |
| 位置 | 値や構文要素の、入力中の場所。行(1 始まり)と列(1 始まり)の組。取り方は第 11 節。 |
| 行 | 行番号。1 始まり。**LF(U+000A)の数**で数える(CRLF でも同じ。第 11.1 節)。 |
| 値の型 | 木の値は、null、真偽値(bool)、整数(int)、範囲外の整数(hugeInt)、浮動小数(float)、文字列(string)、リスト(seq)、マップ(map。キーと値の対応)の 8 種のいずれか(第 6 節)。`.psd1` では、マップはハッシュテーブル `@{}`、リストは配列。 |
| 範囲外の整数 | 64 ビット符号つき整数の範囲(-9,223,372,036,854,775,808〜9,223,372,036,854,775,807)に収まらない整数の書き方。木では種別 `hugeInt` で表し、値は持たない。エラーにはしない(使う側の検査が拒否する)。 |
| マップのリスト | 要素がすべてマップであるリスト(`job` や `transitions`)。 |
| 誤り | エラーの 1 件。種類(`kind`)、行(なしもある)、理由(`reason`)をもつ(第 10 節)。 |
| 受理 | その値や書き方を誤りなく読み込むこと。拒否はその逆。 |

### 1.1 この文書で避ける PowerShell の API と、その理由

| API・演算子 | 挙動(確認の別) | この仕様での扱い |
|----|------|------|
| `-eq`、`-ne`、`-like`、`-match`、`switch`(文字列) | 既定で大文字小文字を区別しない。左辺の型を右辺に合わせて変換する(未確認(公式ドキュメントを取得して確かめていない。広く知られた挙動として記す): 文字列の比較演算子は既定で大文字小文字を区別せず、`c` 接頭辞で区別する)。右辺が数のとき、左辺の文字列は数に変換される | 文字列の比較は `-ceq` か `[string]::Equals(…, Ordinal)`。`-match`、`switch -Regex` は使わず、手書きの走査か、`[regex]` に `RegexOptions` を明示して `\A…\z` で照合する(決定。`RegexOptions.CultureInvariant` を付け、`IgnoreCase` を付けない) |
| `$x -eq $null` | `$x` が配列のとき、要素の絞り込みになる | `$null -eq $x`(`$null` を左辺)で書く |
| `if ($x)` | `0`、空文字列、`$null`、空配列が偽。1 要素の配列は要素の真偽 | 真偽は `-eq $true` か `-is [bool]` で明示する。`if ($x)` を値の有無の判定に使わない |
| 関数の戻り値 | 出力されたすべての値が配列になる。1 要素の配列は単一の値に展開される | 戻り値は `return` で明示し、リストを返すときは `return ,$list`(単項のカンマ)で包む。受け手は `@(...)` で包まずに、`[System.Collections.Generic.List[object]]` を受け取る |
| `+=` による配列の連結 | 配列を作り直す(要素数の 2 乗の時間) | `[System.Collections.Generic.List[object]]` の `Add` か、`[System.Text.StringBuilder]` を使う |
| `[int]` へのキャスト | 小数を丸める(銀行丸め) | 丸めの要る場面はない。整数は字句から自前で求める(第 5.1 節) |
| `Get-Content`、`Set-Content`、`Out-File` | 既定の文字コードと改行の扱いが版で違う(未確認(公式ドキュメントを取得して確かめていない。広く知られた挙動として記す): `-Raw` が無いと行ごとの配列になり、改行が失われる) | 使わない。`[System.IO.File]` と `[System.IO.FileStream]`(第 3 節) |
| `ConvertFrom-Json`、`ConvertTo-Json`、`Import-PowerShellDataFile` | 第 4.11、5 節の比較表のとおり、要件に合わない | 使わない(`Import-PowerShellDataFile` は比較のためだけに第 4.11 節に書く) |
| `Invoke-Expression`、`[scriptblock]::Create`、`.psd1` の `Import-Module` による読み込み | `.psd1` の内容を実行する | 使わない(コードを実行しない) |

## 2. 書式の決定と処理の段階

### 2.1 書式の決定(拡張子)

- 定義ファイルのパスの指定(`--config`)があれば、それを使う。なければ、カレントディレクトリの `conductor.psd1`。
- 書式は、パスの**拡張子**で決める。拡張子は、パスの最後の要素(最後のパス区切りより後ろ。区切りは `/` と `\`(UNIX 系でも `\` を区切りとして扱う。決定。`[System.IO.Path]::DirectorySeparatorChar` の分岐を書かない))の、最後の `.` から末尾まで。ASCII の大文字小文字を区別しない(`.PSD1` は `.psd1`)。畳み込みは、A〜Z を a〜z にする自前の関数で行い、`ToLower()`、`ToLowerInvariant()` を使わない(非 ASCII の文字が ASCII に化ける可能性を避けるため)。
- **`[System.IO.Path]::GetExtension` は使わない**(決定。`.psd1` という名前のファイルの扱いを、この仕様の規定どおりに固定するため。未確認(公式ドキュメントを取得して確かめていない): 最後の `.` から末尾までを返し、`.` が無ければ空文字列。ファイル名が `.psd1` だけのとき、戻り値は `.psd1` だが、この仕様ではそれは拡張子 `.psd1` である(同じ結果)。ただし、プラットフォームによる区切り文字の差があるので、自前の規則を使う)。

| 拡張子 | 書式 |
|--------|------|
| `.psd1` | `.psd1` |
| `.json` | JSON |
| それ以外(拡張子なしを含む。`.txt`、`.xml`、末尾が `.` のものなど) | 拒否: `unsupportedFormat`、行なし、理由 ``拡張子は .psd1、.json のいずれかで書きます`` |

- この判定は、ファイルに触れる前に行う(ファイルが存在しなくても、拡張子が合わなければ `unsupportedFormat` になる)。
- 表示用のファイル名は、パスを文字列にしたもの(与えられた表記のまま。正規化しない。`Resolve-Path` や `[System.IO.Path]::GetFullPath` を通さない)である。

### 2.2 処理の段階と打ち切り

`load(path, expected_pipeline_name)` は次の順に処理する。前の段階で失敗したら、そこで打ち切り、**その 1 件だけ**を返す(後ろの段階は実行しない)。最後の Walk だけが、複数の誤りを返す。

| 段階 | 内容 | 失敗の種類(`kind`) | 返る誤りの数 |
|------|------|----------------------|--------------|
| 1. Intake | 書式の決定、ファイルの読み取り | `unsupportedFormat` / `fileMissing` / `unreadable` / `tooLarge` | 1 |
| 2. Parse | 文法の検査(`.psd1` はパーサー、JSON は自前の読み取り)と、`.psd1` の前処理 | `syntax` | 1 |
| 3. Resolve | 木の組み立て(`.psd1` は許可する構文の検査と値への変換)、キーの重複、上限(要素数・文字の大きさ・入れ子の深さ)、最上位の確認 | `syntax` / `tooLarge` / `typeMismatch`(最上位がマップでない) | 1 |
| 4. VersionCheck | `format_version` | `versionMissing` / `versionUnsupported` | 1 |
| 5. Walk | 構造、型、遷移、終了コード、名前 | 第 10.3 節の種類のうち該当するもの | 1 件以上(位置の順) |

`parse(text, fmt, file_label, expected_pipeline_name)` は、段階 2〜5 を文字列に対して行う(ファイルに触れない)。`file_label` は、エラーの表示に使うファイル名である。

- `expected_pipeline_name`(`pipeline` コマンドで渡されたパイプライン名)が無い(`$null`)ときは、名前の照合をしない。
- 検証に成功したときだけ、定義(第 12 節)を返す。誤りが 1 件でもあれば、定義は返さない。
- 版の確認(段階 4)に通らないと、構造の検査(段階 5)は一切行わない。たとえば `format_version` が無い定義に、未知のキーや型の誤りがあっても、返るのは `versionMissing` の 1 件だけである。
- 文法の誤り(段階 2)は版の確認より先である。
- **JSON の段階 2 は、入力を 1 回、先頭から読み、最初に見つかった問題(文法の誤り、重複キー)で打ち切る。** 入れ子の深さ・要素数・文字の大きさの上限は、段階 2 では確認しない(段階 3 で確認する。第 5.3 節)。
- **`.psd1` の段階 2 は、入力の全体を、前処理(第 4.8 節)と、パーサー(`ParseError` の有無。第 4.1 節)で、文法の誤りが無いか確かめ終えてから、段階 3 に進む。** したがって、後ろにある文法の誤りが、前にある段階 3 の誤り(許可外の構文、キーの重複、上限)より先に返る。
- 段階 3 は、木を、深さ優先・入力に現れる順に走査し、最初に見つかった問題で打ち切る。

## 3. 取り込み(Intake)

書式の決定(第 2.1 節)のあと、次の順に調べる。最初に当てはまったもので失敗する。

| 順 | 条件 | `kind` | 理由(日本語の原文) |
|----|------|--------|----------------------|
| 1 | ファイルの情報(種別・大きさ)の取得で「存在しない」 | `fileMissing` | `指定の場所にファイルがありません` |
| 2 | 同、権限がない | `unreadable` | `読み取りの権限がありません` |
| 3 | 同、その他の失敗 | `unreadable` | `読み取りに失敗しました` |
| 4 | 通常のファイルでない(ディレクトリなど) | `unreadable` | `通常のファイルではありません` |
| 5 | 取得した大きさが 1 MiB(1,048,576 バイト)を超える | `tooLarge` | `ファイルの大きさが上限(1 MiB)を超えています` |
| 6 | ファイルを開けない(権限) | `unreadable` | `読み取りの権限がありません` |
| 6' | ファイルを開けない(その他) | `unreadable` | `読み取りに失敗しました` |
| 7 | 読み取りの途中の失敗 | `unreadable` | 権限の問題なら `読み取りの権限がありません`、他は `読み取りに失敗しました` |
| 8 | 読み取った結果が上限を超える(順 5 のあとに大きくなった場合) | `tooLarge` | 順 5 と同じ |
| 9 | UTF-8 として解釈できない | `unreadable` | `文字コードが UTF-8 ではありません` |

実装(PowerShell + .NET)の対応:

- 順 1〜5: `[System.IO.FileInfo]::new($path)` を作り、`Attributes` と `Length` を読む。`FileInfo` の `Exists` は、ディレクトリでも偽になるので使わず、次の手順にする。(1) `[System.IO.File]::GetAttributes($path)` を `try` で呼ぶ。`FileNotFoundException`・`DirectoryNotFoundException` なら順 1、`UnauthorizedAccessException` なら順 2、それ以外の `IOException`・`ArgumentException`・`NotSupportedException`・`PathTooLongException`・`SecurityException`(パスに NUL を含む場合などを含む)は順 3。(2) 成功したら、属性に `Directory` が立っていれば順 4(デバイスファイルや名前つきパイプを順 4 にするかは、未確認: .NET が属性で表すかを確認していない。実装時の試験 T-14)。(3) `[System.IO.FileInfo]::new($path).Length` が 1,048,576 を超えれば順 5。シンボリックリンクは、リンク先の情報で判定する(.NET の `FileInfo.Length` が、リンクをたどるかは未確認。試験 T-14)。未確認(公式ドキュメントを取得して確かめていない。試験 T-14): 存在しないパスでは `FileNotFoundException`(親が無ければ `DirectoryNotFoundException`)。
- 順 6・7: `[System.IO.FileStream]::new($path, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::ReadWrite)`(`FileShare.ReadWrite` で、他の書き込み中でも開く)で開き、**上限 + 1 バイトまで**を `Read(byte[], int, int)` で読む(戻り値 0 か、上限 + 1 バイトに達するまで繰り返す。1 回の `Read` は、要求より少なく返すことがある)。`UnauthorizedAccessException` なら権限、他の `IOException` は失敗。`try`/`finally` で `Dispose()` する。**`[System.IO.File]::ReadAllBytes`、`ReadAllText`、`Get-Content` は使わない**(上限を超える巨大なファイルを、メモリに読み込まないため)。
- 順 8: 読み取ったバイト数が 1,048,576 を超えていれば。
- 順 9: **厳格な UTF-8 の復号**。`[System.Text.UTF8Encoding]::new($false, $true)`(第 1 引数 `encoderShouldEmitUTF8Identifier = $false`、第 2 引数 `throwOnInvalidBytes = $true`)の `GetString(byte[], 0, count)` を呼び、`DecoderFallbackException` なら順 9。未確認(公式ドキュメントを取得して確かめていない。試験 T-15): 第 2 引数が真だと不正なバイトで例外を投げる。引数なしの `[System.Text.Encoding]::UTF8` は、不正なバイトを U+FFFD に置き換えるので、**使わない**。**`GetString` は先頭の BOM(EF BB BF)を取り除かない**(未確認: BOM は文字 U+FEFF として復号されると推測される。試験 T-15)ので、BOM の除去は自前で行う(第 4.8、5 節)。過長な符号化(`C0 80`)、サロゲートの符号化(`ED A0 80`)、範囲外(`F4 90 80 80`)、単独の `FF` が、いずれも例外になることは、未確認(試験 T-15)。

- 上限は **1 MiB = 1,048,576 バイトちょうどまで受理**し、1 バイトでも超えると `tooLarge`。
- 空のファイル(0 バイト)は、空の文字列として読めて、次の段階へ進む(`.psd1`・JSON とも `syntax`。第 4.8、5 節)。
- 順 5 の大きさの確認は UTF-8 の確認(順 9)より先なので、1 MiB を超えて UTF-8 でないファイルは `tooLarge` である。
- 先頭の BOM(バイト列 EF BB BF)は不正ではない(順 9 を通る)。復号後の文字列の先頭の U+FEFF は、段階 2 が 1 つだけ取り除く(位置には数えない。第 11.1 節)。
- いずれも、行番号は付かない。
- OS による権限・種別の判定の細部(Windows での `HResult` の値など)は、実装に任せてよい(未確認)。必須なのは、上の表の条件の順序と、各理由の文言である。
- ファイルを読むコードは、`Get-Content`、`Test-Path`、`Resolve-Path` を使わない。`Test-Path` と `Resolve-Path` は、`[` `]` などをワイルドカードとして扱い、存在しないパスに失敗する(未確認(公式ドキュメントを取得して確かめていない。広く知られた挙動として記す): `-Path` はワイルドカードを解釈し、`-LiteralPath` は解釈しない)。.NET の API は PowerShell のカレントの場所を使わない(PowerShell の `Set-Location` は、.NET のプロセスの作業ディレクトリを変えない)ので、相対パスは、呼び出し側が、プロセスの作業ディレクトリ(`[System.IO.Directory]::GetCurrentDirectory()`)を基準に解決していることを前提にする。この仕様は、`Set-Location` を使わない(決定)ので、両者は一致する。

## 4. `.psd1` の受理する構文の部分集合

`.psd1` は、PowerShell のスクリプトとして構文解析し、構文木の**ノードの型**(`System.Management.Automation.Language` 名前空間の AST のクラス)を 1 つずつ検査して、許可するものだけを値に変換する。**スクリプトの実行はしない**(`Invoke-Expression`、`& $file`、`. $file`、`Import-Module`、`[scriptblock]::Create` を使わない)。許可する構文は、ハッシュテーブル、配列、文字列、数、`$true`/`$false`/`$null` だけである。

### 4.1 パーサーの呼び出し

- 使う API(確認(出典: Parser クラスの公式 API リファレンス)): `[System.Management.Automation.Language.Parser]::ParseInput([string] $input, [ref] $tokens, [ref] $errors)` と `ParseInput([string] $input, [string] $fileName, [ref] $tokens, [ref] $errors)`、`ParseFile([string] $fileName, [ref] $tokens, [ref] $errors)`。いずれも `ScriptBlockAst` を返し、`tokens`(`Token[]`)と `errors`(`ParseError[]`)を出力引数で返す。PowerShell からは `[ref]` で呼ぶ(`$tokens = $null; $errors = $null; $ast = [System.Management.Automation.Language.Parser]::ParseInput($text, [ref]$tokens, [ref]$errors)`)。
- **`ParseFile` は使わず、`ParseInput` を使う**(決定)。理由: (1) `ParseFile` は自分でファイルを読み、文字コードの判定(BOM の有無による)をパーサーに任せることになる(未確認: 判定の規則。公式の API リファレンスは、「指定したファイルから入力を解析する」としか書いていない)。この仕様は、第 3 節の上限、厳格な UTF-8 の復号、BOM の除去を自前で行う必要がある。(2) 前処理(第 4.8 節)を済ませた文字列を渡すため。`ParseFile` は、ファイルを `ParseInput` と同じに解析するはずだが(未確認)、この仕様の規定は `ParseInput` に対して書く。表示用のファイル名を AST の `Extent.File` に入れたいときは、第 2 引数つきの `ParseInput` に、表示用のファイル名を渡してよい(任意。この仕様は `Extent.File` を使わない)。
- 戻り値の扱い:
  - `errors`(`ParseError[]`)が 1 件以上あれば、**`syntax`**(理由 ``.psd1 の文法に誤りがあります``)で打ち切る。`ParseError` の `Message` と `ErrorId` は、ロケールで翻訳され、入力の値を含むので、**表示にも保持にも使わない**(固定の理由文だけを出す)。確認(出典: ParseError の公式 API リファレンス): プロパティは `ErrorId`、`Extent`、`IncompleteInput`、`Message`。行は、`errors` のうち、`Extent.StartOffset` が最小のもの(同じなら配列の先頭側)の `StartOffset` から、第 11.1 節の表で求める。エラーのときに返る `ScriptBlockAst` は、部分的な木なので、使わない。
  - `errors` が空なら、`ScriptBlockAst` を第 4.2 節の規則で検査する。`tokens` は使わない(任意。コメントの検出などには使わない)。
- `ParseInput` は構文解析だけを行い、スクリプトを**実行しない**(API リファレンスの説明は「ファイルに由来しない入力を解析する」。実行しないことの明示の記述は取得して確かめていない。未確認)。ただし、構文解析のときの、パーサーの内部の再帰の深さによる異常終了(捕捉できない `StackOverflowException`)の可能性は、未確認である(試験 T-04)。そこで、第 4.8 節の事前走査で、括弧の入れ子の深さを、パーサーに渡す前に制限する。
- `ParseInput` に渡す文字列は、第 4.8 節の前処理を済ませたもの(BOM を除き、CRLF を LF にしたもの)である。

### 4.2 AST の検査の手順

`ScriptBlockAst`(根)について、次を満たさなければ、**`syntax`**(理由 ``.psd1 で使えない書き方です``)。位置は、違反したノードの `Extent.StartOffset`(根の違反は、位置 (1, 1))。

| 条件 | 内容 |
|------|------|
| `ParamBlock` が `$null` | `param(...)` を許さない |
| `BeginBlock`、`ProcessBlock`、`DynamicParamBlock`、`CleanBlock` が `$null` | 名前つきのブロックを許さない |
| `Attributes.Count` が 0、`UsingStatements.Count` が 0、`ScriptRequirements` が `$null` | `[Attr()]`、`using`、`#requires` を許さない(確認(出典: ScriptBlockAst の API リファレンス): `ScriptRequirements` は `#requires` の情報で、使われなければ `$null`。`UsingStatements` と `Attributes` は null にならない) |
| `EndBlock` が `$null` でない | 空のファイル、コメントだけのファイルは `EndBlock` が空の文の並びになる(未確認: `$null` か、空のブロックか。どちらでも、「文が 0 個」として扱う) |
| `EndBlock.Statements.Count` が 1 | 0 個(空・コメントだけ)は `syntax`(理由 ``.psd1 の文法に誤りがあります``。位置は入力の末尾)。2 個以上(`@{} @{}`、`$a = 1; @{}`)は `syntax`(理由 ``.psd1 で使えない書き方です``。位置は 2 つ目の文) |
| `EndBlock.Traps` が空、`EndBlock.Unnamed` が真 | `trap {}` を許さない(未確認: 属性名。実装時に確かめる) |
| その 1 つの文が `PipelineAst` で、`PipelineElements.Count` が 1、その要素が `CommandExpressionAst`、`Redirections.Count` が 0、`PipelineAst.Background` が偽 | `a | b`、`@{} > file`、`@{} &`、コマンドを許さない |

`CommandExpressionAst.Expression` が、第 4.3 節の「値の式」であれば、その値が最上位の値になる。最上位が、マップ以外(配列、文字列、数など)でも、段階 3 では文法として受理する(最上位の確認は第 6.7 節)。

`PipelineChainAst`(`a && b`)、`AssignmentStatementAst`(`$a = 1`)、`IfStatementAst`、`FunctionDefinitionAst` などは、**文の型**が `PipelineAst` でないので、この表で拒否される。

### 4.3 値の式(ノードの型の表)

「値の式」の検査は、**明示のスタック**(`[System.Collections.Generic.Stack[object]]`)で行い、関数の再帰を使わない(第 6.4 節)。

| ノードの型(`System.Management.Automation.Language`) | 扱い | 条件・値 |
|----|----|----|
| `HashtableAst` | **受理**(マップ) | `KeyValuePairs`(確認(出典: HashtableAst の API リファレンス): `Tuple<ExpressionAst, StatementAst>` の並び)の各組が、第 4.5 節のキーと、第 4.3 節の値(下の「値の文」)を満たす。空の `@{}` も受理 |
| `ArrayExpressionAst`(`@(...)`) | **受理**(リスト) | 第 4.6 節 |
| `ArrayLiteralAst`(カンマ区切りの `1, 2, 3`) | **受理**(リスト) | `Elements` が、それぞれ値の式。第 4.6 節 |
| `UnaryExpressionAst`(単項のカンマ `,x`) | **受理**(1 要素のリスト) | `TokenKind` が `Comma` で、`Child` が値の式。第 4.6 節 |
| `StringConstantExpressionAst` | **受理**(文字列) | `StringConstantType` が `SingleQuoted`、`DoubleQuoted`、`SingleQuotedHereString`、`DoubleQuotedHereString`。`BareWord` は、**値の位置では拒否**(キーの位置では受理。第 4.5 節)。値は `Value`(第 4.4 節) |
| `ExpandableStringExpressionAst` | **拒否** | 変数の展開(`"$a"`)や部分式(`"$(1)"`)を含む二重引用符の文字列 |
| `ConstantExpressionAst`(`StringConstantExpressionAst` でないもの) | 条件つきで**受理**(整数・浮動小数) | 第 4.3.1 節。型は `Value` ではなく、**`Extent.Text` の字句**で決める |
| `UnaryExpressionAst`(`-` で始まる数) | 条件つきで**受理** | `Extent.Text` が第 4.3.1 節の負の数の字句に一致し、`Child` が `ConstantExpressionAst`(`StringConstantExpressionAst` でない)。パーサーが負の数を 1 つの `ConstantExpressionAst` として返す場合(確認(出典: tokenizer のソース): `AllowSignedNumbers` が真のとき、`+`/`-` に続く数字を 1 つの数のトークンとして読む。どの文脈で真になるかは未確認)も、`UnaryExpressionAst` を返す場合も、`Extent.Text` で判定するので、同じ結果になる |
| `VariableExpressionAst` | 条件つきで**受理** | `Extent.Text` が、ASCII の大文字小文字を区別せず、`$true`、`$false`、`$null` のいずれかと**完全に一致**するとき。真偽値と null。それ以外(`$x`、`$env:PATH`、`${true}`、`$global:true`、`@x`(スプラット))は拒否 |
| `ParenExpressionAst`(`(1)`) | **拒否** | |
| `SubExpressionAst`(`$(...)`) | **拒否** | |
| `ScriptBlockExpressionAst`(`{...}`) | **拒否** | |
| `ConvertExpressionAst`(`[int]1`、`[ordered]@{}`) | **拒否** | キャスト。`[ordered]` も拒否(キーの順序は、記述順を常に保つので不要) |
| `TypeExpressionAst`(`[int]`) | **拒否** | |
| `BinaryExpressionAst`(`1 + 2`、`'a' -f 1`) | **拒否** | |
| `UnaryExpressionAst`(上の 2 つ以外: `!$true`、`-not`、`++`、`+1`) | **拒否** | `+1` も拒否(符号は `-` だけ) |
| `TernaryExpressionAst`、`MemberExpressionAst`、`InvokeMemberExpressionAst`、`IndexExpressionAst`、`AttributedExpressionAst`、`UsingExpressionAst`、`ErrorExpressionAst` | **拒否** | |
| `CommandAst`(コマンドの呼び出し。`Get-Date`、`foo`) | **拒否** | `PipelineAst` の要素が `CommandExpressionAst` でないので、上の手順で拒否される |
| その他のすべてのノードの型 | **拒否** | 将来 PowerShell が追加するノードの型も、拒否する(許可リスト方式) |

- **許可リスト方式**: 検査は、型が上の「受理」の行にあるかを調べ、無ければ拒否する。「拒否の行」に書いていない型も拒否する。
- 値の文(`HashtableAst` の値側の `StatementAst`)は、第 4.2 節の最後の行と同じ条件(`PipelineAst`、要素 1 個、`CommandExpressionAst`、`Redirections` が空、`Background` が偽)を満たさなければ、`syntax`(理由 ``.psd1 で使えない書き方です``)。
- 拒否のときの位置は、そのノードの `Extent.StartOffset`。

#### 4.3.1 数の字句(`Extent.Text` による)

**数の値は、`ConstantExpressionAst.Value`(パーサーが選んだ `int`/`long`/`double`/`decimal`/`BigInteger` など)を使わず、`Extent.Text` を自前で解析して求める**(決定)。理由: (1) PowerShell は、接尾辞(`l`、`u`、`ul`、`d`、`n`、`y`、`uy`、`s`、`us`、`kb`、`mb`、`gb`、`tb`、`pb`)で型や値を変え、接頭辞なしの 10 進整数を `int`→`long`→`decimal`→`double` の順に選ぶ(確認(出典: tokenizer のソース): 型の接尾辞 `u`、`s`、`l`、`d`、`y`、`n`、倍数の接尾辞 `k`/`m`/`g`/`t`/`p` + `b`(1024 の累乗)。接尾辞のない 10 進整数は、int、long、(10 進の書式のとき)decimal、double の順に試す。16 進・2 進で大きすぎるものは、数として読めず失敗する)。(2) 16 進リテラルの型と符号は、版や桁数で変わる(確認(出典: tokenizer のソース): 16 桁(2 進は 64 桁)で接尾辞がないものは 64 ビットに昇格する)。(3) 全角のダッシュ(U+2013、U+2014、U+2015)が `-` として読まれる(確認(出典: tokenizer のソースと CharTraits のソース): `IsDash` が ASCII の `-`、EnDash、EmDash、HorizontalBar を真にし、数の先頭の符号は ASCII の `-` に正規化される)。この仕様は ASCII の `-` だけを受理する。

受理する字句(`Extent.Text` が、次のいずれかに**全体で**一致するとき。正規表現は説明用。実装は `\A…\z` で、`[0-9]` だけを使う(`\d` を使わない)):

| 種別 | 字句 | 値 |
|------|------|----|
| 10 進の整数 | `-?(0\|[1-9][0-9]*)` | 数字の列から求める。`-0` は 0。64 ビット符号つき整数の範囲を超えるものは、`hugeInt`(エラーにしない) |
| 16 進の整数 | `0[xX][0-9A-Fa-f]+` | 符号なし。16 進数字の大文字小文字は問わない。先頭の 0 を並べてよい(`0x00ff`)。範囲を超えるものは `hugeInt` |
| 2 進の整数 | `0[bB][01]+` | 同上 |
| 浮動小数 | `-?(0\|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?` で、`.` か指数部の少なくとも一方を含むもの | `[double]::Parse(text, NumberStyles.Float, InvariantCulture)`。結果が ±∞ になる(または `OverflowException`)ものは、`overflow` の印を付ける(エラーにしない)。極小(`1e-400`)は 0.0(エラーにしない) |

上のどれにも一致しない数(`01`、`+1`、`1_000`、`.5`、`5.`、`1e`、`1kb`、`1mb`、`10l`、`1d`、`1u`、`1n`、`0x`、`0xFFl`、`-0x1`、`１`(全角)、`–1`(U+2013 で始まる)、`1.0d`、`∞`)は、**`syntax`**(理由 ``.psd1 で使えない書き方です``、位置は式の先頭)。パーサーが数として読めず `ParseError` を返すもの(例: 範囲を超える 16 進、`0x` だけ)は、段階 2 の `syntax`(理由 ``.psd1 の文法に誤りがあります``)で先に返る。

- **整数だけの項目**(`format_version`、`max`、`multiplier`、各種の終了コード)に、浮動小数が来たら、型の誤りとして拒否する。`1.0`、`1e0`、`10e-1` のように値が整数になる書き方でも拒否する。判定は、木のノードの種別(`int` か `float` か)で行い、`[math]::Floor` などで整数と見なさない。
- 範囲外の整数(`hugeInt`)と、範囲を超える浮動小数は、読み取りでは拒否しない(変換器の `.psd1` の読み取りは、拒否する。[09. 変換器](09-converter.md))。定義のどのキーでも使われない場所にあるときは、そのキーが `unknownKey` になるだけである。
- `1e999` を、パーサーが数として読むか(`double.TryParse` が ∞ を返すと推測される)、`ParseError` にするかは未確認(確認(出典: tokenizer のソース): 範囲を超える浮動小数の検査は、トークナイザーにはなく、.NET の `double.TryParse` に任されている。`double.TryParse` が桁あふれで ∞ を返すか、失敗するかは、.NET Core 3.0 以降の挙動で、公式ドキュメントでの確認はしていない)。読めた場合は `overflow` の印を付ける。読めずに `ParseError` なら `syntax` になる。試験 T-05。

### 4.4 文字列

- `StringConstantExpressionAst.Value` を、文字列の値とする(パーサーが、引用符・エスケープ・here-string の外側を処理した後の値)。確認(出典: about_Quoting_Rules): 単一引用符の文字列は逐語的(中の `''` は `'` 1 個)。二重引用符の文字列では、バッククォート(`` ` ``)がエスケープ文字で、`` `0 ``、`` `a ``、`` `b ``、`` `e ``、`` `f ``、`` `n ``、`` `r ``、`` `t ``、`` `u{x} ``(1〜6 桁の 16 進、最大 10FFFF)、`` `v `` が使える(about_Special_Characters。`` `e `` と `` `u{x} `` は PowerShell 6 以降)。二重引用符の中の `""` は `"` 1 個。**スマートクォートの扱い**(確認(出典: about_Quoting_Rules と CharTraits のソース)): PowerShell は、U+2018、U+2019、U+201A、U+201B を単一引用符と、U+201C、U+201D、U+201E を二重引用符と見なす。したがって、文字列の中にこれらの文字を直接書くときは、引用符と同じ要領で 2 回並べる必要がある(例: `'a’’b'` の値は `a’b`)。書き出し(変換器)は、これらの文字を含む文字列を、`` `u{…} `` で書く(第 4.9、[09](09-converter.md))。
- **二重引用符の文字列は、展開のないものだけ受理**する。`"abc"` は `StringConstantExpressionAst`(`DoubleQuoted`)で受理。`"a$b"`、`"$(1)"`、`"${x}"` は `ExpandableStringExpressionAst` で拒否。`` "a`$b" ``(`$` をエスケープしたもの)は展開でないので受理し、値は `a$b`。`"a$"`(`$` の後に識別子の文字がない)は、展開でないと推測されるが未確認(試験 T-06)。
- **here-string**(`@'` … `'@`、`@"` … `"@`)は、`StringConstantExpressionAst` として得られるものを受理する。確認(出典: about_Quoting_Rules): 開きの印の直後の改行と、閉じの印の直前の改行は、値に含まれない。値は、`Value` のとおり(第 4.8 節で、改行を LF にそろえてから解析するので、値の改行は LF である)。二重引用符の here-string で変数などを含むものは、`ExpandableStringExpressionAst` で拒否。here-string の細部(行頭の空白、末尾の改行の数)の `Value` は、公式ドキュメントの記述に従い、この仕様は `Value` をそのまま使う(試験 T-07)。
- 複数行にまたがる通常の文字列(`'a` + 改行 + `b'`)も受理し、値に LF を含む(確認(出典: about_Quoting_Rules): 通常の文字列も複数行にまたがれる)。
- 文字列の長さの上限は、第 6.3 節の上限に含まれる(個別の上限はない)。
- 文字列の値は、対にならないサロゲートを含まない(入力が UTF-8 の復号を通った文字列で、エスケープの `` `u{D800} `` は、上のサロゲートの規則(下)で拒否する)。`` `u{…} `` で書かれた値が U+D800〜U+DFFF のとき、または U+10FFFF を超えるとき、パーサーが値として何を返すか(例外、置換文字、そのままの単独のサロゲート)は未確認(試験 T-08)。実装は、`Value` に対にならないサロゲートが含まれていたら、**`syntax`**(理由 ``.psd1 の文法に誤りがあります``、位置は文字列の先頭)で拒否する。

### 4.5 キー

`HashtableAst.KeyValuePairs` の各組の `Item1`(キーの式)について:

| ノードの型 | 扱い |
|----|----|
| `StringConstantExpressionAst`(`BareWord`、`SingleQuoted`、`DoubleQuoted`。`Value` が文字列) | **受理**。キーは `Value`。裸の単語のキー(`name = 'x'`)は `BareWord` |
| `StringConstantExpressionAst`(here-string) | **受理**(値と同じ) |
| `ExpandableStringExpressionAst` | **拒否**(`syntax`、理由 ``.psd1 で使えない書き方です``) |
| `ConstantExpressionAst`(数。`@{ 1 = 'a' }`) | **拒否**(`syntax`、理由 ``キーは文字列で書きます``、位置はキーの先頭) |
| `VariableExpressionAst`(`$true = 1`、`$x = 1`) | **拒否**(`syntax`、理由 ``キーは文字列で書きます``) |
| その他のノードの型(`ParenExpressionAst` など) | **拒否**(`syntax`、理由 ``キーは文字列で書きます``) |

- 裸のキーに使える文字は、パーサーが決める(`a-b`、`a.b`、`1abc` が、裸の単語として受理されるかは未確認。試験 T-09)。受理された場合は、`Value` がキーになる。キーに裸の単語で書けない文字(空白、`=`、`;`、`@`、`{`、`}`、`(`、`)`、`'`、`"`、`#`、`,`、`|`、`&`、`<`、`>`)がある場合は、引用符で囲む(パーサーが判定する)。空文字列のキー(`'' = 1`)は受理する。
- **キーの重複**: 同じマップの中で、キーを、**ASCII の英字の大文字小文字を区別せずに**比べて、同じになるものがあれば、**`syntax`**(理由 ``キーが重複しています``)。位置は 2 つ目のキーの先頭。判定は、キーの `Value` を、A〜Z を a〜z にした文字列(自前の関数)の完全一致(`[string]::Equals(a, b, Ordinal)`)で行う(**決定**。理由: ハッシュテーブル `@{}` は、キーの大文字小文字を区別しない(未確認(公式ドキュメントを取得して確かめていない): 既定で大文字小文字を区別しない。比較の文化も未確認。非 ASCII の英字の畳み込みを、この仕様は行わない)ので、`@{ a = 1; A = 2 }` を `.psd1` として読む他の道具(`Import-PowerShellDataFile` など)は、これを重複として扱うか、後勝ちにする。この仕様は、曖昧さを避けるため、拒否する)。キーの比較の対象は、エスケープ後の値(`'a'` と `a` と `"a"` は重複)。
- **キーの意味は、大文字小文字を区別する**: 定義のキー(`format_version`、`pipeline`、`Name` など)は、木のキーを**完全一致**(Ordinal)で照合する。したがって `@{ Format_Version = 1 }` は `unknownKey`(キー `Format_Version`)になる。JSON のキーも同様(完全一致)。
- 重複の検査は、段階 3 で、マップのキーを訪れた時点で行う(第 2.2 節)。

### 4.6 配列

PowerShell の配列は、カンマ区切りの式(`ArrayLiteralAst`)、`@(...)`(`ArrayExpressionAst`)、複数行で改行区切りの `@(...)` の 3 つの書き方がある。実行時の意味(要素の平坦化)と、この仕様の木の意味が食い違うと、他の道具との間でデータが変わるので、次のとおり**食い違う書き方を拒否**する(決定)。

**`@(...)` の中身**: `ArrayExpressionAst.SubExpression`(`StatementBlockAst`)の `Statements`(1 個以上、または 0 個で空の配列)。各文は、第 4.2 節の最後の行の条件(`PipelineAst`、要素 1 個、`CommandExpressionAst`)を満たす。各文の式 `E` について:

| `E` のノードの型 | 要素 |
|----|----|
| `ArrayLiteralAst`(カンマ区切り) | `Elements` のそれぞれが要素(それぞれ値の式) |
| `UnaryExpressionAst`(単項のカンマ `,x`) | `Child` を 1 つの要素とする |
| `ArrayExpressionAst`(`@(...)` が文として単独で現れる) | **拒否**(`syntax`、理由 ``.psd1 で使えない書き方です``)。実行時は、1 段平坦化される(`@(@(1,2))` は `1, 2`)ので、`[[1,2]]` と読むと食い違う |
| それ以外の値の式(スカラー、`HashtableAst`) | その式 1 つが要素 |

- `@()` は空のリスト。
- 文を複数書く(改行区切りの `@( 1 <改行> 2 )`)と、各文の要素を順に連ねたリストになる。例: `@(1 <改行> 2)` は `[1, 2]`。`@(1, 2 <改行> 3)` は `[1, 2, 3]`。要素が `HashtableAst` の文を並べる `@(@{a=1} <改行> @{b=2})` は 2 要素。
- カンマ区切りの途中の要素が `ArrayExpressionAst` のとき(`@(@(1,2), 3)`)は、その要素は入れ子のリスト(`[[1,2], 3]`)。実行時の意味と同じ。
- 1 要素のカンマリスト `,1` を値の位置に書く(`a = ,1`)と、1 要素のリスト `[1]`。
- **カンマ区切りの式が値の位置にある**(`a = 1, 2, 3`)と、リスト。要素が 1 個のカンマ区切りは文法上ない(それは単項のカンマ)。
- `@(...)` でも `,` でもない、単独の値(`a = 1`)は、スカラー。

**この規定が、`Import-PowerShellDataFile` と同じ結果になる範囲**: 書き出し(変換器)は、第 9.5 節の書式だけを出力する。その書式は、実行時の意味でも、この仕様の木と同じになる(第 4.11 節の比較)。

### 4.7 コメントと区切り

- `#` から行末までのコメントと、`<# ... #>` のブロックコメントを、どこにでも書ける(パーサーが処理する。値・キーの後ろにも書ける)。コメントは、読み込み時に捨てられる。
- キーと値の組は、改行または `;` で区切る。末尾の `;` を置いてよい。同じ行に複数の組を `;` で並べてよい。
- キーと値の間は `=`。`=` の前後の空白、改行(`name =` の次の行に値を書く)の可否は、パーサーが決める(未確認: 改行を許すか。試験 T-10)。
- インデントに意味はない。

### 4.8 前処理(段階 2 の最初。パーサーに渡す前)

復号後の文字列 `text` に、次を、この順に行う。いずれも、拒否のときは **`syntax`**、行は第 11.1 節の規則(拒否する文字の位置)で求める。

1. **先頭の BOM**: `text` の先頭が U+FEFF なら、1 文字取り除く(位置には数えない)。以降の `text` は、BOM を除いた文字列である。
2. **改行と文字の確認**: 全文字を走査して、次のいずれかを見つけたら、その位置で、理由 ``.psd1 の文法に誤りがあります`` で拒否する。(a) CR(U+000D)で、直後が LF(U+000A)でないもの(CR だけの改行)。(b) U+0085、U+2028、U+2029。(c) タブ(U+0009)、LF、CRLF の CR 以外の制御文字(U+0000〜U+0008、U+000B、U+000C、U+000E〜U+001F、U+007F〜U+009F)。(d) 2 文字目以降の U+FEFF。コメントの中、文字列の中でも拒否する(JSON のエスケープで書かれた制御文字は、`.psd1` に出力するとき、`` `u{…} `` で書く。第 9.5 節)。
3. **改行の正規化**: CRLF を LF に置き換える(すべての CR を除く)。以降、`ParseInput` に渡す文字列と、位置の計算に使う文字列は、この LF だけの文字列である。理由: 文字列の値と here-string の値に含まれる改行が、ファイルの改行の種類(LF か CRLF か)に依存しないようにするため。
4. **空の入力**: 正規化後の `text` が、空、または空白とコメントだけでも、パーサーが「文が 0 個」として返す(第 4.2 節の `Statements.Count` が 0)。空文字列は、`ParseInput` を呼ばずに、`syntax`(理由 ``.psd1 の文法に誤りがあります``、位置は (1, 1))にしてよい(結果は同じ)。
5. **括弧の入れ子の事前走査**: `text` を先頭から走査して、`(`、`{`、`[` で深さを 1 増やし、`)`、`}`、`]` で 1 減らす(0 未満にはしない)。文字列やコメントの中の括弧も数える(粗い走査。決定)。深さが 1,000 を超えた時点の、その文字の位置で、**`syntax`**(理由 ``入れ子が深すぎます(上限は 32 段)``)。理由: パーサーの内部の再帰の深さで、捕捉できない `StackOverflowException` が起きる可能性(未確認。試験 T-04)を避ける。33〜1,000 段の入れ子は、パーサーに渡す(段階 3 が、32 段の上限を、木の深さで判定する)。1 MiB の入力の中の括弧は高々 1,048,576 個なので、走査は 1 回で済む。
6. `ParseInput`(第 4.1 節)を呼ぶ。

### 4.9 受理する例

次はすべて受理する(値を確認する)。

```powershell
@{
    format_version = 1                         # 裸のキー
    'pipeline' = @{ name = 'p'; description = "夜間バッチ" }   # 引用符つきのキー、; 区切り
    job = @(
        @{
            id = 'a'
            command = 'echo ''x'''             # 単一引用符の中の '' は ' (値: echo 'x')
            warning_exit_codes = 4, 5          # カンマ区切り(リスト [4, 5])
            success_exit_codes = @(0, 0x05)    # 16 進(値 5)
            retry = @{ exit_codes = @(75); max = 3; interval = '5s' }
        },
        @{ id = 'b'; command = @('prog', 'a b', '$HOME', '') }   # 単一引用符の中の $ は展開されない
    )
}
```

| 書き方 | 値 |
|--------|----|
| `a = $true` / `$TRUE` / `$False` / `$null` | 真偽値 / 真偽値 / 真偽値 / null |
| `a = 'it''s'` | 文字列 `it's` |
| `a = "x`ty"`(バッククォート + t) | 文字列 `x` + タブ + `y` |
| `a = "q`"q"` | 文字列 `q"q` |
| `a = "`u{1F600}"` | 文字列 U+1F600(絵文字) |
| `a = @'` 改行 `line1` 改行 `line2` 改行 `'@` | 文字列 `line1` + LF + `line2` |
| `a = -5` | 整数 -5 |
| `a = 0xFF` / `0b101` | 整数 255 / 5 |
| `a = 1.5e-7` / `-0.0` / `1e21` | 浮動小数 |
| `a = 9223372036854775808` | 範囲外の整数(`hugeInt`) |
| `a = @()` / `@{}` | 空のリスト / 空のマップ |
| `a = ,@(1, 2)` | リスト `[[1, 2]]`(1 要素) |
| `a = @(,@(1, 2))` | リスト `[[1, 2]]` |
| `a = @(@(1, 2), 3)` | リスト `[[1, 2], 3]` |
| `a = @(` 改行 `1` 改行 `2` 改行 `)` | リスト `[1, 2]` |
| `# コメント` のあとに `@{ }` | 空のマップ(最上位。定義としては `versionMissing`) |

### 4.10 拒否する例

次は、すべて拒否する。種類は `syntax`(最上位がマップでないものは第 6.7 節)。**理由**は、「文法」= ``.psd1 の文法に誤りがあります``(`ParseError` など)、「構文」= ``.psd1 で使えない書き方です``(許可外のノード)。

| 入力 | 理由 | 備考 |
|------|------|------|
| `@{ a = 1 ` (閉じていない) | 文法 | `ParseError` |
| `@{ a = }` | 文法 | `ParseError` |
| `@{ a = 1 b = 2 }`(区切りなし) | 文法 | `ParseError` |
| `@{ a = $x }` | 構文 | `VariableExpressionAst` |
| `@{ a = $env:HOME }` | 構文 | `VariableExpressionAst` |
| `@{ a = "$x" }` | 構文 | `ExpandableStringExpressionAst` |
| `@{ a = "$(Get-Date)" }` | 構文 | `ExpandableStringExpressionAst` |
| `@{ a = Get-Date }` | 構文 | `CommandAst`(値の文が `CommandExpressionAst` でない) |
| `@{ a = (1) }` | 構文 | `ParenExpressionAst` |
| `@{ a = $(1) }` | 構文 | `SubExpressionAst` |
| `@{ a = { 1 } }` | 構文 | `ScriptBlockExpressionAst` |
| `@{ a = 1 + 2 }` | 構文 | `BinaryExpressionAst` |
| `@{ a = [int]1 }` | 構文 | `ConvertExpressionAst` |
| `@{ a = [System.IO.File] }` | 構文 | `TypeExpressionAst` |
| `[ordered]@{ a = 1 }` | 構文 | `ConvertExpressionAst` |
| `@{ a = 'x'.Length }` | 構文 | `MemberExpressionAst` |
| `@{ a = (Get-Date).Year }` | 構文 | `MemberExpressionAst` |
| `@{ a = 1 \| Out-Null }` | 構文 | パイプライン(要素 2 個) |
| `$a = @{ x = 1 }` | 構文 | `AssignmentStatementAst`(文が `PipelineAst` でない) |
| `@{ a = 1 }; @{ b = 2 }` | 構文 | 文が 2 個 |
| `Import-Module x` + `@{ a = 1 }` | 構文 | 文が 2 個 |
| `#requires -Version 7` + `@{}` | 構文 | `ScriptRequirements` |
| `using namespace System` + `@{}` | 構文 | `UsingStatements` |
| `param($a)` + `@{}` | 構文 | `ParamBlock` |
| `@{ a = 1kb }` | 構文 | 数の接尾辞 |
| `@{ a = 10l }` / `1d` / `1u` / `1n` | 構文 | 数の接尾辞 |
| `@{ a = 01 }` | 構文 | 先頭の 0 |
| `@{ a = +1 }` | 構文 | `+` の符号 |
| `@{ a = .5 }` / `5.` | 構文 | 数の字句 |
| `@{ a = 1_000 }` | 構文または文法 | 未確認(パーサーが数として読むか) |
| `@{ a = -0x1 }` | 構文 | 16 進に符号 |
| `@{ a = 0xFFFFFFFFFFFFFFFFFFFF }` | 文法 | 数として読めない(確認(出典: tokenizer のソース): 範囲を超える 16 進は失敗) |
| `@{ a = foo }` | 構文 | 値の位置の裸の単語(`CommandAst`) |
| `@{ 1 = 'a' }` | キーは文字列で書きます | 数のキー |
| `@{ $true = 1 }` | キーは文字列で書きます | |
| `@{ a = 1; a = 2 }` | キーが重複しています | |
| `@{ a = 1; A = 2 }` | キーが重複しています | 大文字小文字を区別しない |
| `@{ a = @(@(1, 2)) }` | 構文 | 文として単独の `@(...)` |
| `@{ a = @(` 改行 `@(1)` 改行 `@(2)` 改行 `)` `}` | 構文 | 同上 |
| `@{ a = 1 } \| ForEach-Object { }` | 構文 | パイプライン |
| `@{ a = 1 } > out.txt` | 構文 | リダイレクト(`Redirections`) |
| `@{ a = 1 } &` | 構文 | `Background` |
| `@{ a = $true && $false }` | 文法または構文 | `PipelineChainAst`。値の文が `PipelineAst` でない |
| `1` のみ | 最上位がマップでない(第 6.7 節) | `typeMismatch` |
| 空のファイル | 文法 | 文が 0 個 |
| `@{ a = 1 }` の全角ダッシュ `@{ a = –1 }` | 構文 | U+2013 で始まる |
| CR だけで改行したファイル | 文法 | 第 4.8 節 #2 |
| 文字列の中の U+2028 | 文法 | 第 4.8 節 #2 |

### 4.11 `Import-PowerShellDataFile` との比較(参考)

`Import-PowerShellDataFile` は、この仕様の検査には**使わない**。比較:

| 項目 | `Import-PowerShellDataFile`(確認(出典: 公式ドキュメント)) | この仕様 |
|------|------|------|
| 戻り値 | ハッシュテーブル(`Hashtable`)。位置の情報は無い | 位置(行・列)つきの木 |
| 上限 | 既定で 500 キー、5000 AST ノード(`-SkipLimitCheck` で解除) | 展開後の要素数 100,000(キーも数える)、ファイルの大きさ 1 MiB、深さ 32 段 |
| キーの大文字小文字 | ハッシュテーブルは区別しない | 重複の判定は ASCII の大文字小文字を区別せず拒否。照合は完全一致(第 4.5 節) |
| 重複キー | 未確認(ハッシュテーブルの作成時の挙動に従うと推測される) | 拒否(`syntax`) |
| 数の型 | パーサーが選ぶ型(`int`/`long`/`double` など) | 字句から自前で決める(第 4.3.1 節)。型の揺れがない |
| エラー | 例外(メッセージはロケールで変わり、入力の値を含む) | 固定の種類・理由・行 |
| `-0.0` | `double` | 浮動小数。ビット表現を保つ |
| `@(@(1,2))` | 実行時と同じに平坦化されて `1, 2` | 拒否(第 4.6 節) |
| 最上位 | ハッシュテーブルのみ(ドキュメントの記述: 「`.psd1` の中のハッシュテーブルのキーと値」)。他の型の挙動は未確認 | 任意の値の式を受理し、マップでなければ、後段で `typeMismatch` / `rootNotMap` |
| ファイルの読み取り | PowerShell の既定のファイル読み取り | 自前の厳格な UTF-8 の復号と上限 |
| 位置の取り出し | できない | `Extent.StartOffset` から求める(第 11 節) |

**往復の互換**: この仕様の書き出し(第 9.5 節)が出力する `.psd1` は、`Import-PowerShellDataFile` でも、同じ値のハッシュテーブルとして読める(未確認: 試験 T-11。特に、`-9223372036854775808` が、`long` として読まれるか `double` になるか、`1e16` の型)。

## 5. JSON の受理する形式

JSON は、RFC 8259 の厳格な形式で読む。読み取りは**自前の小さな字句・構文解析**(PowerShell で書く)で行う(第 5.3 節)。**`ConvertFrom-Json` は使わない**。`System.Text.Json` も使わない。理由:

| 手段 | 挙動(確認の別) | この仕様との関係 |
|------|------|------|
| `ConvertFrom-Json` の重複キー | 後勝ち(確認(出典: ConvertFrom-Json の公式ドキュメント): 「JSON の標準は重複キーを許すが、`PSObject` と `Hashtable` は禁止なので、重複があれば最後のキーだけが使われる」) | この仕様は拒否する |
| `ConvertFrom-Json` の位置 | 位置の情報を返さない(公式ドキュメントに位置の記述がない。未確認) | 値・キーの位置が必要(第 11 節) |
| `ConvertFrom-Json` のキーの大文字小文字 | `-AsHashtable` なしだと、大文字小文字だけが違うキーは同じキーと見なされ、最後のものだけが残る。空文字列のキーはエラー(確認(出典: ConvertFrom-Json の公式ドキュメント)) | この仕様は大文字小文字を区別する。空文字列のキーを受理する |
| `ConvertFrom-Json` の日時 | 日時の形式の文字列を `[datetime]` に変換する(確認(出典: ConvertFrom-Json の公式ドキュメントの注記): PowerShell 6 以降。`Kind` が `Unspecified`/`Utc`/`Local`) | 文字列は文字列のまま |
| `ConvertFrom-Json` のコメント | PowerShell 6 以降は、JSON のコメントを受け付ける(確認(出典: 同上)) | 拒否が規定 |
| `ConvertFrom-Json` の深さ | `-Depth` の既定は 1024(確認(出典: 同上)) | 上限は 32 段 |
| `ConvertFrom-Json` の 1 要素の配列 | 配列が列挙されて、1 要素だと要素そのものになる(`-NoEnumerate` で回避。確認(出典: 同上の例 5)) | 配列は配列のまま |
| `ConvertFrom-Json` の整数・小数 | 巨大整数、小数の扱いが、版と型(`int`/`long`/`BigInteger`/`double`)で変わる(公式ドキュメントに、型の選択の記述がない。未確認) | 整数と浮動小数を字句で区別し、範囲外の整数を `hugeInt` にする |
| `ConvertFrom-Json` の実装 | Newtonsoft Json.NET による(確認(出典: 同上の注記)) | 不正なサロゲート、`NaN` の扱いなどの挙動に依存したくない |
| `ConvertTo-Json` の深さ | `-Depth` の既定は 2、指定できる最大は 100。深さを超えると、警告を出して、その先を文字列に変換する(確認(出典: ConvertTo-Json の公式ドキュメント): PowerShell 7.1 以降は警告。「0〜100」) | 自前で書き出す(第 9 節、[09](09-converter.md)) |
| `ConvertTo-Json` の書式 | 既定のエスケープは制御文字だけ(`-EscapeHandling`)。インデントは既定で整形される(書式の細部は公式ドキュメントに規定がない)。`-Compress` で 1 行になる。浮動小数・巨大整数の書式は規定がない(確認(出典: 同上): PowerShell 7.5 で `BigInteger` を数としてシリアライズできる) | 出力の文字列を、この仕様に一致させるので使えない |
| `ConvertTo-Json` の順序 | `[ordered]` やオブジェクトのプロパティ順に従う。ハッシュテーブル(`@{}`)の順序は保証されない(未確認(公式ドキュメントを取得して確かめていない): 順序を保つには `[ordered]`) | 自前で書き出す |
| `System.Text.Json.Utf8JsonReader` | `ref struct`。PowerShell のスクリプトから、`ref struct` を、変数に保持し、メソッドに渡して使うことができない(確認(出典: PowerShell の言語仕様・.NET の `ref struct` の制約の一般的な記述。PowerShell での明示の記述は未確認)。試験 T-13)。位置はバイトの添字。重複キーは検出しない | 使わない |
| `System.Text.Json.JsonDocument`/`JsonNode` | 位置を返さない。重複キーの扱いは設定による(`JsonDocument.Parse` は、重複を許すか、未確認) | 使わない |

したがって、**自前で読む**(第 5.3 節)。

| 項目 | 規定 |
|------|------|
| 入力 | UTF-8 で復号済みの文字列(第 3 節)。先頭の BOM(U+FEFF)は無視する(位置には数えない。第 11.1 節)。 |
| 空白 | 半角スペース、タブ、LF、CR だけ。それ以外(垂直タブ、全角スペースなど)を、トークンの間に置くことは拒否。 |
| コメント(`//`、`/* */`、`#`) | **拒否**(`syntax`)。 |
| 末尾のカンマ(`[1,]`、`{"a":1,}`)、連続したカンマ、先頭のカンマ | **拒否**(`syntax`)。 |
| リテラル | `true`、`false`、`null`(小文字だけ)。`True`、`NaN`、`Infinity`、`undefined` は拒否。 |
| 文字列 | `"..."` だけ(単一引用符は拒否)。中にタブを含め、U+0000〜U+001F を直接書くことは拒否(`\t` などで書く)。U+007F 以上は直接書いてよい。エスケープは `\"`、`\\`、`\/`、`\b`、`\f`、`\n`、`\r`、`\t`、`\uXXXX`(16 進 4 桁。大文字小文字どちらでも)。それ以外のエスケープは拒否。`\u` でサロゲートの前半(U+D800〜U+DBFF)に、後半(U+DC00〜U+DFFF)の `\u` が続く対は、そのまま 2 つの `[char]` として文字列に入れる(UTF-16 の対になる)。**対にならないサロゲート**(単独の前半、単独の後半。対は直前直後の `\u` エスケープの組だけで判定し、直接書かれたサロゲートは、UTF-8 の復号が拒否済み)は拒否(`syntax`)。 |
| 数 | `-?(0\|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?` だけ。`+1`、`01`、`.5`、`5.`、`1e`、`0x10`、`1_000` は拒否。 |
| キー | 文字列だけ。 |
| 重複したキー | **拒否**(`syntax`、理由 ``キーが重複しています``、行は 2 つ目のキーの位置)。判定は、エスケープを解いた後の文字列の完全一致(Ordinal。**大文字小文字を区別する**。`"a"` と `"A"` は別のキー)。 |
| 最上位 | 任意の JSON の値を、文法としては受理する。最上位がオブジェクト(マップ)でなければ、第 6.7 節のとおり `typeMismatch`。JSON の `null` も(`.psd1` の `$null` と同じく)マップではないので `typeMismatch`。 |
| 空の入力、空白だけの入力 | 拒否(`syntax`)。 |
| 最上位の値の後ろ | 空白以外が続くことは拒否(`syntax`。複数の値、`{}{}`、後ろのゴミ)。 |

### 5.1 数の型

- 整数: `-?(0|[1-9][0-9]*)` だけ(小数部も指数部もない書き方)。`-0` は整数 0。値は**数字の列から**得る(`[System.Numerics.BigInteger]::Parse(text, [System.Globalization.NumberStyles]::AllowLeadingSign, [System.Globalization.CultureInfo]::InvariantCulture)` で得て、`[long]::MinValue`、`[long]::MaxValue` を `[System.Numerics.BigInteger]` にして比べる)。64 ビット符号つき整数の範囲を超えるものは `hugeInt`(エラーにはしない)。桁数が 19 を超える(符号を除く。JSON は先頭の 0 を持たないので、`0` 以外の先頭は 1〜9)ものは、`BigInteger.Parse` を呼ばずに `hugeInt`(巨大な桁数の字句で時間を使わないため)、19 桁以下は `BigInteger.Parse` で得て範囲を比べる。結果が範囲内なら `[long]` に変換する(`[long]$big` は、範囲内でだけ呼ぶ)。
- 浮動小数: 小数部または指数部を持つもの(`1.0`、`1e3`、`1E3`、`1e0`、`-0.0`)。値は `[double]::Parse(字句, [System.Globalization.NumberStyles]::Float, [System.Globalization.CultureInfo]::InvariantCulture)`(`NumberStyles.Float` は、先頭と末尾の空白、符号、小数点、指数を許す。字句は先に検査済みなので、他の書式は来ない)。結果が ±∞ のもの(`1e400`)、または `OverflowException` になるものは、`overflow` の印を付ける(エラーにはしない)。極小(`1e-400`)は 0.0(エラーにしない)。指数部が 9 桁以上の巨大な字句を `double.Parse` が処理する時間と結果は未確認(試験 T-12)。そこで、指数部の数字(符号を除く)が 5 桁を超える字句は、仮数が 0 でなければ、符号つきで `overflow`(指数が正)または 0.0(指数が負)として扱い、`double.Parse` を呼ばない(仮数が全部 0 なら 0.0。決定)。
- **整数だけの項目**に、浮動小数が来たら、型の誤りとして拒否する(第 4.3.1 節と同じ)。
- JSON の数のうち、値を使う検査の対象外の場所にある浮動小数や範囲外の整数は、エラーにならない(そのキーが `unknownKey` になるだけ)。

### 5.2 JSON の誤りの位置

文法の誤りの位置は、**入力の先頭から読んでいって、読み進められなくなった最初の文字**(その前までが、有効な JSON の先頭部分になっているもの)の位置。入力が途中で終わったとき(閉じていない配列・文字列など)は、入力の末尾の位置(末尾が改行なら、その次の空の行を 1 行と数える)。例:

| 入力(`\n` は改行) | 誤りの位置 |
|------|------------|
| `{"a": 1,}` | `}` の位置(1 行目) |
| `{"a" 1}` | `1` の位置 |
| `{\n  "a": 1\n  "b": 2\n}` | `"b"` の位置(3 行目) |
| `[1, 2` + 末尾の改行 | 末尾(2 行目) |
| `{"a": "x` + 改行 + `"}` | 改行の位置(1 行目。文字列の中に改行を直接書いたため) |
| `{"a":1}{"b":2}` | 2 つ目の `{` の位置 |
| `{"a":1,"a":2}`(重複) | 2 つ目の `"a"` の位置(1 行目) |

文字列の中の誤り(不正なエスケープ、対にならないサロゲート、直接書いた制御文字)の行は、その文字列の行(文字列は改行を含められないので、どの文字でも同じ行)。

### 5.3 JSON の読み取りの規定(自前の字句・構文解析)

読み取りの関数: `Read-ConductorJsonTree`(第 12.2 節。成功なら木、失敗なら 1 件の誤り)。ステートの読み込みも、同じ関数を `-DeferDuplicateKey` で呼ぶ(重複を見つけても止めず、後ろの文法の誤りを優先する。[06. ステートファイル](06-state-file.md))。**この読み取りは、定義・変換・ステートの 3 つで共用する(決定)。** ステートは、BOM を拒否するため、呼ぶ前に先頭の U+FEFF を検査する(06 章)。次は**必須**である。

1. **入力を、先頭から 1 回だけ読む**。文字列を `[string]` の添字(`$text[$i]`。`[char]` が得られる)で走査し、位置(行、列)を、第 11.1 節の表で求める(走査の途中で、LF を数えて行の先頭の添字を持つ方式でもよい)。ループは、`while ($i -lt $n)` の形で書き、`for`/`foreach` の `break`/`continue` の意味の違い(`ForEach-Object` のスクリプトブロックの中の `break`/`continue` は、ループでなくパイプラインを抜けるので、`ForEach-Object` を、走査に使わない)に注意する。
2. **入れ子は、明示のスタック**(`[System.Collections.Generic.Stack[object]]`)で読む(関数の再帰で読まない)。この段階では、入れ子の深さの上限を**適用しない**(段階 3 が適用する)。これは、深い入れ子の後ろに文法の誤りがあるとき、文法の誤りが先に返る(段階 2 が先)ことを守るためである。スタックの大きさは、入力の大きさ(1 MiB)で抑えられる。
3. **整数の字句**: 第 5 節の「数」の文法に従って、字句を先頭から読む(文字ごとに `'0'`〜`'9'` の範囲(`$c -ge [char]'0' -and $c -le [char]'9'`。`[char]` どうしの比較は、`-ge`/`-le` で序数。ただし、**`-ge` は `[char]` を文化依存で比較する可能性がある**ので、`[int]$c` の整数で比べる(決定))。読んだ字句に、小数部・指数部が無ければ整数、有れば浮動小数とする(第 5.1 節)。字句の直後の文字が、区切り(空白、`,`、`]`、`}`、入力の末尾)でなければ、その文字の位置で文法の誤り(`01` は 2 文字目の `1`、`1.` は `.` の次、`1e` は `e` の次、`+1` は `+`、`.5` は `.`、`0x10` は `x`)。
4. **リテラル**: `true`、`false`、`null` は、文字ごとに照合し(`-ceq` または `[int]` の比較)、最初に合わない文字(または入力の末尾)の位置で文法の誤り。リテラルの直後が区切りでなければ、その文字の位置で文法の誤り。
5. **文字列**: 開きの `"` から読む。U+0000〜U+001F の文字、改行、入力の末尾は、その位置で文法の誤り。エスケープは第 5 節の表のとおり。`\u` の 4 桁が 16 進でなければ文法の誤り(`[char]::IsLetterOrDigit` などでなく、`[0-9a-fA-F]` を、`[int]` の範囲比較で確かめる)。サロゲートの前半の `\uD800`〜`\uDBFF` の直後が、`\uDC00`〜`\uDFFF` の `\u` でなければ、文法の誤り(位置はそのエスケープの `\` の位置)。後半が単独で現れたときも同じ。読んだ文字列は、`[System.Text.StringBuilder]` に `[char]` を追加して作る(`+=` を使わない)。`\u` のコード単位は `[char][int]$code`(`[char]` への変換は 0〜65535 で成功する)。
6. **オブジェクト**: `{` の後ろは、空白を飛ばして `}`(空のオブジェクト)か文字列(キー)。キーの文字列を読み終えた**時点で**、重複を調べる(そのオブジェクトの `[System.Collections.Generic.HashSet[string]]`(`[System.StringComparer]::Ordinal` で作る。`@{}` や既定の `HashSet[string]` は、`@{}` が大文字小文字を区別しないので、使えない。`HashSet[string]` の既定は序数比較だが、明示する)に、エスケープを解いたキーが既にあれば、**そのキーの開きの `"` の位置で**重複のエラー。`:` 以降は読まない)。次に `:`、値、そして `,`(次のキー)か `}`。
7. **配列**: `[` の後ろは、空白を飛ばして `]`(空の配列)か値。値の後ろは `,`(次の値)か `]`。`,` の直後に `]` が来ること(末尾のカンマ)は、`]` の位置で文法の誤り。
8. **最上位**: 空白を飛ばして値を 1 つ読み、空白を飛ばして入力の末尾に達しなければ、その位置で文法の誤り。入力が空(空白だけを含む)なら、入力の末尾の位置で文法の誤り。
9. 木のノードの位置は、第 11.2 節(オブジェクトは `{`、配列は `[`、スカラーは字句の先頭、キーの位置は開きの `"`)。
10. 木は、`[System.Collections.Generic.List[object]]` の `Entries`(第 6.1 節。キーを `[string]` で持つ並び)として作る。**`@{}`(ハッシュテーブル)に入れて重複を落とすことをしない**(重複は拒否するので結果は同じだが、キーの順序と、重複の検出の時点を、この規定のとおりにするため)。
11. **性能**: PowerShell のループは、コンパイルされた言語より遅い(1 回の反復が、マイクロ秒の単位。確認: 公式ドキュメントでの数値の記述はなし。未確認)。1 MiB の入力を、字句の走査に 1 文字 1 反復で読むと、数秒から数十秒の可能性がある(未確認)。**目標**(実測で見直す。試験 T-16): 1 MiB の JSON(最悪: `[` を 1,048,576 個)を、30 秒以内に、規定の結果で返す。空白を飛ばす処理を、`[string]::IndexOf` や、`[System.Text.RegularExpressions.Regex]` の `Match(text, startat)` で、まとめて行ってよい(第 1.1 節の正規表現の規則: `\G`、`RegexOptions.CultureInvariant`、`[0-9]`、`\A`/`\z` を使い、`$` と `\d` を使わない)。

文法の誤りの理由は ``JSON の文法に誤りがあります``、重複は ``キーが重複しています``(いずれも `syntax`)。

## 6. 木への展開(Resolve)と上限

### 6.1 木の型

段階 3 の結果と、段階 4・5 の入力は、`.psd1` と JSON に共通の木である。**型の骨格は、`class` を使わず、`[pscustomobject]` と `[System.Collections.Generic.List[object]]` で作る**(決定)。理由(確認(出典: about_Classes の「制限」の節)): PowerShell のクラスは、セッションの中で、再読み込みも解放もできない(「PowerShell classes can't be unloaded or reloaded in a session」)。`Import-Module -Force` で、クラスの変更は反映されず、新しいセッションが要る。また、ドットソースしたファイルで定義したクラスは、モジュールの外へ自動では公開されず、`using module` の取り込みは、ドットソースしたクラスを一貫して取り込まない(「It doesn't consistently import classes defined in nested modules or classes defined in scripts that are dot-sourced into the root module」)。Pester(テスト)は、モジュールを何度も読み込み直すので、クラスは、型の不一致を起こす。また、クラスは、既定で作成した実行空間に結び付く(「By default, a PowerShell class is affiliated with the Runspace where it's created」)。したがって、型は、種別を持つ `[pscustomobject]` で表す(判別は `Kind` の文字列で行う)。

次は、構造の例(名前は実装が決めてよい。ファクトリー関数は `New-ConductorNode…`)。

```powershell
# 位置(1 始まりの行と列)
[pscustomobject]@{ Line = [int]1; Column = [int]1 }

# ノード: Kind は 'null' | 'bool' | 'int' | 'hugeInt' | 'float' | 'string' | 'seq' | 'map'
[pscustomobject]@{ Kind = 'null';    Pos = $pos }
[pscustomobject]@{ Kind = 'bool';    Pos = $pos; Value = [bool]$true }
[pscustomobject]@{ Kind = 'int';     Pos = $pos; Value = [long]5 }            # 64 ビット符号つき整数に収まる値
[pscustomobject]@{ Kind = 'hugeInt'; Pos = $pos }                              # 値は持たない
[pscustomobject]@{ Kind = 'float';   Pos = $pos; Value = [double]1.5; Overflow = [bool]$false }
[pscustomobject]@{ Kind = 'string';  Pos = $pos; Value = [string]'x' }
[pscustomobject]@{ Kind = 'seq';     Pos = $pos; Items = [System.Collections.Generic.List[object]]::new() }
[pscustomobject]@{ Kind = 'map';     Pos = $pos; Entries = [System.Collections.Generic.List[object]]::new() }
# マップのエントリ
[pscustomobject]@{ Key = [string]'a'; KeyPos = $pos; Value = $node }
```

- 木は、`.psd1` でも JSON でも、アンカーなどを含まない(展開済み)。マップのキーは `[string]` で重複せず(`.psd1` は大文字小文字を畳んで重複しない)、**記述順**を保つ。
- **マップを `@{}`(ハッシュテーブル)で持たない**(位置つきのエントリを順序どおりに持つため。`@{}` は順序を保たず、大文字小文字を区別しない。未確認(公式ドキュメントを取得して確かめていない))。`[ordered]@{}`(`OrderedDictionary`)も、キーの大文字小文字の区別が既定で無い(未確認: `[ordered]` の比較の文化)ので、木の本体には使わない。キーの検索が要る検査では、`[System.Collections.Generic.Dictionary[string, object]]`(`[System.StringComparer]::Ordinal` を渡して作る)を一時的に作る。「許されたキーか」の判定には、`[System.Collections.Generic.HashSet[string]]`(`Ordinal`)を使う。
- 整数の値は `[long]`、真偽値は `[bool]`、浮動小数は `[double]`。**整数かどうかの判定は、`Kind` の文字列で行い**(`-ceq 'int'`)、`$value -is [int]`・`[long]` の判定に頼らない(PowerShell は、`[int]` と `[long]` を相互に暗黙に変換する。`$null -eq` の注意は第 1.1 節)。`bool` を整数と見なさない(`Kind` が違う)。
- `Set-StrictMode -Version Latest` の下では、存在しないプロパティの参照が例外になるので、ノードの種別ごとに、持つプロパティだけを参照する。
- 木を走査する処理(段階 3 の走査、深さの数え、検査 Walk、変換器の書き出し)は、**関数の再帰を使わず、明示のスタック**で行う。必須なのは、深い入力でも、PowerShell の呼び出しの深さの上限(スクリプトの関数の呼び出しの深さの既定の上限は、版で変わる。未確認)や、捕捉できない `StackOverflowException` で異常終了せず、定められた誤りで返ること(第 6.4 節)。
- 木のノードを作る関数を、1 回の入力につき数十万回呼ぶので、`New-Object` と、パイプライン(`ForEach-Object`、`Where-Object`)は使わず、`[pscustomobject]@{...}` の直接の構築と `foreach` 文を使う(性能。`New-Object` が遅いことは、確認(出典: 一般的な記述。公式ドキュメントでの数値はなし。未確認))。

### 6.2 `.psd1` の展開の手順(段階 3)

`ScriptBlockAst` から、第 4.2 節の検査を通った最上位の式を、入力に現れる順に走査し、上の木に写す(明示のスタックで、木を組み立てる)。各ノードについて、次を行う(ここで「ノードの位置」は、第 11.2 節の位置)。

1. 第 4.3〜4.6 節の規則で、許可するノードか確かめる。拒否なら、第 4.3〜4.6 節の種類・理由・位置で打ち切る。
2. マップなら、第 4.5 節のとおり、キーの確認(文字列か)、重複の確認を、エントリを訪れる順に行う。
3. 第 6.3 節の上限を、ノードを訪れるごとに加算して確認する。確認の順は、深さ、要素数、文字の大きさ(同じノードで複数が超えるとき、先のものを返す。この仕様の決定)。
4. 範囲外の整数と、範囲を超える浮動小数は、木に記録する(`hugeInt`、`Overflow`)。エラーにはしない。

### 6.3 上限

| 上限 | 値 | 超過したときの失敗 |
|------|----|--------------------|
| 要素数 | 100,000 | `tooLarge`、理由 ``要素数が上限(100000)を超えています`` |
| 文字の大きさ | 16 MiB(16,777,216 バイト) | `tooLarge`、理由 ``文字の大きさが上限(16 MiB)を超えています`` |
| 入れ子の深さ | 32 | `syntax`、理由 ``入れ子が深すぎます(上限は 32 段)`` |

- **要素数の数え方**: 木で訪れたノードを 1 つずつ数える。**キーも数える**(キー 1 つで 1)。マップ・リストも 1。根のマップも 1。**100,000 ちょうどまで受理**し、100,001 個目で失敗する。`.psd1` でも JSON でも同じ数え方。例: `{"a":[0,0,...]}`(0 を n 個)は、根のマップ 1 + キー 1 + リスト 1 + 要素 n で、n = 99,997 まで受理、99,998 で失敗。
- **文字の大きさの数え方**: 訪れたスカラーの文字のバイト数(UTF-8)を足す。キーと値の両方。文字列は、**値(エスケープを解いたもの)のバイト数**(`[System.Text.Encoding]::UTF8.GetByteCount($value)`。単独のサロゲートが無いので失敗しない。`[System.Text.UTF8Encoding]::new($false, $true)` のインスタンスを使い回す)。数・真偽値・null は、**書かれた文字のバイト数**(`.psd1`: `123` は 3、`0x1F` は 4、`$null` は 5、`$true` は 5、`$false` は 6、`-1` は 2。JSON: `null` は 4、`true` は 4、`false` は 5)。**16 MiB ちょうどは受理**し、1 バイト超えると失敗する。なお、ファイルの大きさの上限(1 MiB)により、この上限が働くことは通常ない(`.psd1`・JSON の入力に、展開の仕組みが無いため)。第 3 節の上限を、変換器が緩めたときのための規定として残す。
- **入れ子の深さの数え方**: マップとリストは、それぞれ**1 段**と数える。スカラーは数えない。**根のマップそのものは数えない**。根の直下のコンテナの深さを 1 とし、その子のコンテナは 2、…と数える。**32 段まで受理**し、**33 段目を拒否する**。失敗の行は、33 段目になったコンテナの位置(第 11 節)。
  - 例: `.psd1` では、入れ子のリストを、`@(,@(,@(1)))` のように書く(第 4.6 節。`@(` の中の単項のカンマの文は、その子を 1 要素にするだけで、段を増やさない)。`@(` の数が n 個の `@(,@(, … @(1)))` は n 段(最も外側の `@(` が 1 段)。32 個は受理、33 個は拒否。値の位置の単項のカンマ(`a = ,@(1)`)は、1 要素のリストを作るので、1 段増える。JSON の `{"a": [[[...]]]}`(`[` が n 個)も n 段。`a = @{ b = @{ b = ... 1 } }`(`@{` の数が n 個)も n 段。
  - カンマ区切りのリスト `a = 1, 2` は 1 段。`@(1, 2)` も 1 段(`@()` の中のカンマ区切りは、1 つのリストを作るので、`ArrayExpressionAst` と `ArrayLiteralAst` を、まとめて 1 段とする)。
- 上限超過の失敗は、いずれも段階 3 の 1 件の打ち切り(`tooLarge`/`syntax`)で、複数は返さない。同じ入力に複数の上限の超過があれば、入力に現れる順で最初のもの。

### 6.4 深い入力と異常終了の禁止

- 数千〜数十万段の入れ子の入力(`@(` を並べる、`@{ a = ` を並べる、JSON の `[` を並べる)でも、プロセスが異常終了せず、`syntax` の 1 件で返らなければならない(必須)。理由が ``入れ子が深すぎます(上限は 32 段)`` か ``.psd1 の文法に誤りがあります`` かは、実装に任せてよい(`.psd1` の場合)。
- JSON は自前の読み取りなので、明示のスタックで読む(第 5.3 節)。
- `.psd1` は、パーサーの再帰を、実装が制御できないので、第 4.8 節 #5 の事前走査(括弧の深さ 1,000)で守る。事前走査を通った(深さ 1,000 以下の)入力を、`ParseInput` が、異常終了せずに解析できるか(スタックの消費が、1,000 段で十分に余裕があるか)は未確認(試験 T-04)。**試験で、1,000 段が危ういと分かったときは、事前走査の上限を下げる**(この仕様の変更として、本節と第 4.8 節を書き換える)。
- AST から木への変換は、明示のスタックで行う(第 6.1 節)。
- 例外の捕捉: `ParseInput` を `try`/`catch` で囲み、`[System.Exception]` を捕まえて、`syntax`(理由 ``.psd1 の文法に誤りがあります``)にする(防御。`StackOverflowException` は捕まえられない)。

### 6.5 変換器との違い

[09. 変換器](09-converter.md)は、この節の上限と数え方をそのまま使う。変換器だけが、範囲外の数(`hugeInt`、`Overflow` の浮動小数)を拒否する。定義の読み込みは、拒否しない。

### 6.6 JSON の段階 3

JSON には、展開の仕組みがないので、段階 3 は、第 6.3 節の上限の確認(木を深さ優先・入力に現れる順に走査して、第 6.2 節の 3 と同じ順で確認する)と、最上位の確認(次の 6.7)だけである。重複キーと文法の誤りは段階 2 で済んでいる。

### 6.7 最上位の確認

| 入力 | 結果 |
|------|------|
| 最上位がマップ | 次へ |
| それ以外(リスト、スカラー。JSON の `null`、`.psd1` の `$null` を含む) | `typeMismatch`、理由 ``最上位はマップで書きます``、行は最上位のノードの位置 |

- JSON も `.psd1` も、空の入力は、段階 2 の `syntax`(第 4.8、5 節)である。

## 7. 書式の版(VersionCheck)

根の `format_version` を確認する。

| 状況 | `kind` | 行 | 理由 |
|------|--------|----|------|
| キーが無い | `versionMissing` | なし | `` `format_version` が必要です(対応する版は 1 です) `` |
| 整数でない(文字列、浮動小数、真偽値、null、リスト、マップ。`1.0`、`1e0` を含む) | `versionUnsupported` | 値の位置の行 | `` `format_version` は整数で書きます(対応する版は 1 です) `` |
| 整数だが 1 でない(0、-1、2、9、範囲外の整数) | `versionUnsupported` | 値の位置の行 | `この版には対応していません(対応する版は 1 です)` |

- 対応する版は **1 だけ**。`format_version = 1` を受理する。`.psd1` の `0x1`、`0b1`(値が 1)でも受理する(JSON では `1` だけ)。判定は、ノードの種別が `int` で、値が `1` であること(`$true` は整数でないので、`format_version = $true` は「整数でない」の行になる)。
- 版が 1 のとき、取り込み後の定義の `format_version` は 1 になる。

## 8. 定義の構造(Walk)

以降の「マップ」「リスト」は、木の型(第 6.1 節)である。**キーの型**は、すべて `[string]`。位置の取り方は第 11 節。

根のマップは、次のキーだけを持てる。

| キー | 型 | 必須 | 説明 |
|------|----|------|------|
| `format_version` | 整数 | 必須 | 第 7 節 |
| `pipeline` | マップ | 必須 | 第 8.1 節 |
| `job` | マップのリスト | 必須(1 つ以上) | 第 8.2 節 |

許されていないキー(上の 3 つ以外)は、`unknownKey` である。`jobs`(複数形)も未知のキーとして拒否される(``キー `jobs` は定義にありません``)。`unknownKey` の理由は、`` キー `<キー名>` は定義にありません `` の形(`<キー名>` は整形したキー名)で、キー名の整形は第 10.2 節。位置はそのキーの位置。値の型は問わない(null でも同じ)。キーの照合は、完全一致(Ordinal。第 4.5 節)。

### 8.1 `pipeline`

| キー | 型 | 必須 | 既定値 | 検査 |
|------|----|------|--------|------|
| `name` | 文字列 | 必須 | なし | 空白だけ(trim して空)なら `emptyField` |
| `description` | 文字列 | 任意 | なし | 型だけ |
| `run_id_env` | 文字列 | 任意 | なし(実行時の既定は環境変数 `CONDUCTOR_RUN_ID`) | 名前の形(下記) |
| `remove_on_complete` | 真偽値 | 任意 | `false` | 型だけ |

- 許されたキー以外は `unknownKey`(例: `owner`、`remove_on_complet`)。
- `name` が無い: `typeMismatch`、行は `pipeline` のマップの位置、理由 ``必須の値 `name` がありません``。文字列でない(null を含む): `typeMismatch`、値の位置、理由 `` `name` は文字列で書きます ``。`description` が文字列でない: `typeMismatch`、値の位置、理由 `` `description` は文字列で書きます ``。`run_id_env` が文字列でない: `typeMismatch`、値の位置、理由 `` `run_id_env` は文字列で書きます ``。`remove_on_complete` が真偽値でない: `typeMismatch`、値の位置、理由 `` `remove_on_complete` は真偽値(`true` または `false`)で書きます ``。
- `name` が空白だけ: `emptyField`、`name` の値の位置、理由 `` `name` が空です ``(名前の照合はしない)。
- `run_id_env` の名前の形の誤り: `invalidEnvName`、値の位置、理由 `` `run_id_env` は、英字または `_` で始まり、英数字と `_` だけの名前で書きます ``。
- `run_id_env` の名前の形: 1 文字以上で、先頭が ASCII の英字(`A-Za-z`)または `_`、2 文字目以降が ASCII の英数字(`A-Za-z0-9`)または `_`。**自前の走査で判定する**(`[int]` の範囲比較)。正規表現で書くなら `\A[A-Za-z_][A-Za-z0-9_]*\z` を、`RegexOptions.CultureInvariant`(`IgnoreCase` なし)で `IsMatch` する(**`-match` を使わない**: 大文字小文字を区別しない(`[A-Za-z]` の範囲は、`IgnoreCase` が付くと、Kelvin 記号 U+212A などが一致しうる)。`$` を使わない: 末尾の改行の直前にも一致し、`"A`n"` が通ってしまう。`\w` や `[char]::IsLetterOrDigit` も使わない: 非 ASCII を受理するため)。受理: `A`、`_`、`_a1`、`GITHUB_RUN_ID`。拒否(`invalidEnvName`): `""`、`1A`、`A-B`、`A B`、`A=B`、`é`、`A.B`、NUL を含む文字列、`A` + 改行。文字列でない値(整数など)は `typeMismatch`。誤りの文に、値は含めない。
- `remove_on_complete` は真偽値だけ(`$true`/`$false`。大文字小文字は問わない。JSON は `true`/`false`)。文字列 `'true'`、整数 `1`、リスト `@($true)` は `typeMismatch`。判定は、ノードの種別が `bool` であること。型が違うときの値は、`false` として扱って続行する(誤りは報告済み)。
- `pipeline` の形:
  - 無い: `typeMismatch`、行は **1 行目**(位置 (1, 1))、理由 ``必須の値 `pipeline` がありません``。
  - リスト: `typeMismatch`、リストの位置、理由 `` `pipeline` は 1 つだけで、マップで書きます(1 ファイルに 1 パイプライン) ``。
  - その他(整数、文字列、null など): `typeMismatch`、値の位置、理由 `` `pipeline` はマップで書きます ``。
  - 上の誤りのとき、`pipeline` の中身の検査は行わない。
- 名前の照合: `expected_pipeline_name` が与えられ、定義の `name` と**完全に同じでない**(大文字小文字・空白も区別。`[string]::Equals(a, b, Ordinal)`)と、`pipelineNameMismatch`、行は `name` の値の位置、理由 ``指定されたパイプライン名が、定義の `name` と一致しません``。理由に両方の名前は含めない。`name` が空白だけのときは、`emptyField` だけを返し、照合はしない。`name` が文字列でない・無いときも、照合はしない。

### 8.2 `job`

`job` は、マップのリストでなければならない。

- 無い: `noJobs`、行は 1 行目(位置 (1, 1))、理由 ``ジョブ(`job`)が 1 つもありません``。
- 空のリスト(`job = @()`): `noJobs`、行は `job` の値の位置。
- リストでない(マップ、整数、null など): `typeMismatch`、値の位置、理由 `` `job` はマップのリストで書きます ``。
- リストの要素がマップでない: `typeMismatch`、その要素の位置、理由 `` `job` の要素はマップで書きます ``(その要素の検査は行わない)。

各ジョブ(マップ)のキー:

| キー | 型 | 必須 | 既定値 | 説明 |
|------|----|------|--------|------|
| `id` | 文字列 | 必須 | なし | ジョブID。第 9.1 節の検査 |
| `command` | 文字列、または文字列のリスト | 必須 | なし | 第 8.3 節 |
| `warning_exit_codes` | 整数のリスト | 任意 | 空(`@()`) | 警告にする終了コード |
| `success_exit_codes` | 整数のリスト | 任意 | 省略(意味は `[0]`) | 成功にする終了コード |
| `transitions` | マップのリスト | 任意 | 空 | 第 8.4 節 |
| `retry` | マップ | 任意 | なし | 第 8.5 節 |

許されたキー以外は `unknownKey`(例: `warn_exit_codes`)。

- `id` が無い: `typeMismatch`、行はジョブのマップの位置、理由 ``必須の値 `id` がありません``。文字列でない: `typeMismatch`、値の位置、理由 `` `id` は文字列で書きます ``。
- 終了コードのリスト(`warning_exit_codes` と `success_exit_codes`):
  - リストでない(null を含む): `typeMismatch`、値の位置、理由 `` `<キー名>` は整数のリストで書きます ``(`<キー名>` は実際のキー名)。
  - 要素が整数でない、または 32 ビット符号つき整数の範囲(-2,147,483,648〜2,147,483,647)に収まらない: 要素ごとに `typeMismatch`、その要素の位置、理由 `` `<キー名>` の要素は、整数(32 ビットの範囲)で書きます ``。1 つでも誤りがあれば、そのキー全体を「無効」として扱い、後続の重なりの検査では使わない。範囲の判定は、`Kind` が `int` のノードの値(`[long]`)で行う(`-2147483648 -le $v -and $v -le 2147483647`)。
  - 負の値(例: `-1`)は、その範囲なので**受理**する。重複した値も受理し、除かない(書いた順のまま保持する)。空のリストも受理する(`success_exit_codes = @()` は、成功にする終了コードが無い、の意味になる)。
  - 整数は、`.psd1` の `0x1F`、`0b101` でもよい(値で判定する)。JSON は 10 進の整数だけ。
- `success_exit_codes` を省略すると、成功の終了コードは `[0]`。書いたら、**書いた値だけ**が成功(`0` を書かなければ 0 でも成功にならない)。
- 要素が浮動小数(`1.0`、`1e0`)や、範囲外の整数(`99999999999999999999`、種別 `hugeInt`)、真偽値(`$true`)のときも、要素の型の誤り(`typeMismatch`)になる。

### 8.3 `command`

| 書き方 | 受理の条件 | 保持される値 |
|--------|-----------|---------------|
| 文字列 | trim して空でない | 1 本の文字列(そのまま。trim した値ではない) |
| 文字列のリスト | 空でないリストで、全要素が文字列で、**先頭の要素が trim して空でない**。2 番目以降の要素は空文字列でもよい | 要素の並び(そのまま) |

どちらの書き方かは、取り込み後も区別して保持する(実行側の解釈が異なる。[05. 実行フロー](05-execution-flow.md)・[07. プロセスとシグナル](07-process-and-signals.md))。

誤り:

| 条件 | `kind` | 行 | 理由 |
|------|--------|----|------|
| キーが無い | `typeMismatch` | ジョブのマップの位置 | ``必須の値 `command` がありません`` |
| 文字列が空または空白だけ | `emptyField` | 値の位置 | `` `command` が空です `` |
| リストが空 | `emptyField` | 値の位置 | `` `command` の配列が空です `` |
| リストに文字列でない要素がある | `typeMismatch`(要素ごと) | その要素の位置 | `` `command` の配列の要素は、文字列で書きます `` |
| リストの先頭の要素が空または空白だけ(全要素が文字列のとき) | `emptyField` | 先頭の要素の位置 | `` `command` の配列の先頭の要素が空です `` |
| 文字列でもリストでもない(整数・真偽値・マップ・null) | `typeMismatch` | 値の位置 | `` `command` は文字列、または文字列の配列で書きます `` |

- リストに型の誤りの要素があるときは、先頭要素の空の検査はしない。
- 受理の例(`.psd1`): `command = 'echo x'`、`command = @('my prog', 'a b', '$HOME', '')`、`command = @('echo', '')`。拒否の例: `command = 5`、`command = $true`、`command = @{ a = 1 }`、`command = @('a', 5)`、`command = ,@('a')`(`typeMismatch`)、`command = ''`、`command = '   '`、`command = @()`、`command = @('')`、`command = @('  ', 'x')`(`emptyField`)、`command = $null`(`typeMismatch`)。
- 文字列の `command` の中の引用符・`$` などは、この段階では解釈しない(実行時の解釈は実行側の仕様)。`.psd1` の単一引用符の文字列の中の `$HOME` は、展開されない(第 4.4 節)。
- 理由の文言(`の配列` という語)は、リストを指す(日本語の原文として固定する。`リスト` に言い換えない)。
- 誤りの文に、入力の値は含めない。

### 8.4 `transitions`

`transitions` は、マップのリスト(`.psd1` の `@( @{ on = 'success'; next = 'b' }, ... )`、JSON の配列)。

| キー | 型 | 必須 | 説明 |
|------|----|------|------|
| `on` | 文字列 | 必須 | `success` または `warning` |
| `next` | 文字列 | 必須 | 遷移先のジョブID、または `@end` |

- 許されたキー以外(`when` など)は `unknownKey`。
- `transitions` がリストでない(null を含む): `typeMismatch`、値の位置、理由 `` `transitions` はマップのリストで書きます ``(遷移は空として扱う)。要素がマップでない: `typeMismatch`、その要素の位置、理由 `` `transitions` の要素はマップで書きます ``。
- `on` が無い: `typeMismatch`、遷移のマップの位置、理由 ``必須の値 `on` がありません``。文字列でない: `typeMismatch`、値の位置、`` `on` は文字列で書きます ``。
- `on` の値: `success`、`warning` を受理(完全一致。大文字小文字を区別する)。`error` は `errorTransition`、値の位置、理由 ``エラー結果(`error`)には遷移先を書けません``。それ以外の文字列(`later`、大文字の `Success` など)は `typeMismatch`、値の位置、理由 `` `on` は `success` または `warning` で書きます ``。
- `next` が無い: `typeMismatch`、遷移のマップの位置、理由 ``必須の値 `next` がありません``。文字列でない: `typeMismatch`、値の位置、`` `next` は文字列で書きます ``。
- 同じ `on`(`success` どうし、`warning` どうし)の遷移が 2 つ以上: 2 つ目以降の遷移に `duplicateTransition`、行は**その遷移(マップ)の位置**、理由 ``同じ結果(`on`)の遷移が 2 つ以上あります``。`success` と `warning` を 1 つずつは受理。`on` が不正(`error` や未知の値、型の誤り)の遷移は、重複判定に入れない。
- 遷移先の存在・順序の検査は、第 9 節。

### 8.5 `retry`

ジョブが決めた終了コードで終わったときに、待って実行し直す設定。書き方は、`.psd1` のハッシュテーブル(`retry = @{ ... }`)でも、JSON のオブジェクトでもよい。

許されたキーは次の 5 つだけ。それ以外は `unknownKey`(例: `retries`)。

| キー | 型 | 必須 | 範囲・書式 | 取り込み後 |
|------|----|------|-----------|-------------|
| `exit_codes` | 整数のリスト | 必須 | 空でない。各要素は 1 以上 255 以下の整数 | 昇順に並べ、重複をまとめる(`@(75, 3, 75, 3, 12)` → `3, 12, 75`) |
| `max` | 整数 | 必須 | 1 以上 100 以下 | `[int]` |
| `interval` | 文字列 | 必須 | `<整数><s\|m\|h>`、1 秒以上 24 時間以下 | 秒(`[uint64]` で持ってよい。1〜86,400) |
| `multiplier` | 整数 | 任意 | 2 以上 10 以下 | `[int]`、または無し |
| `max_interval` | 文字列 | 任意 | `interval` と同じ書式、`interval` 以上 | 秒、または無し |

- 127 などの値も受理する(`@(127, 1)` → `1, 127`)。並べ替えは、数値の昇順(`[long]` の値で比べる。`Sort-Object` は使わず、重複を除いた `[long[]]` を `[System.Array]::Sort` する。要素が `[long]` なので、数値で並ぶ。文字列の並べ替えにならない)。
- `multiplier` と `max_interval` は、**両方を書くか、両方とも書かない**か。片方だけなら `retryBackoffIncomplete`。両方なしなら、待ち時間は毎回 `interval`。
- 取り込み後の値は、最大でも 86,400。**待ち時間の計算(倍率の累乗、上限、64 ビット符号なし整数の飽和)は、[05. 実行フロー](05-execution-flow.md)の規定に従う**(`[uint64]` の演算は、オーバーフローで例外か `double` への昇格かが版と書き方で変わるので、`[System.Numerics.BigInteger]` で計算して、`[uint64]::MaxValue` と比べて飽和させる。この文書の範囲外)。

#### 間隔の文字列の書式

- 文字列の最後の 1 文字が単位(`s`=秒、`m`=分=60 秒、`h`=時間=3600 秒。小文字だけ)。その前が、**ASCII の数字(0〜9)だけ**の 1 文字以上の列(符号・空白・小数点・`_`・全角数字・16 進(`0x`)は不可)。**自前の走査**(`[int]` の範囲比較)で判定する。正規表現で書くなら `\A[0-9]+[smh]\z` を、`RegexOptions.CultureInvariant`(`IgnoreCase` なし)で照合する(`\d` を使わない。全角数字に一致するため。`-match` を使わない。`$` を使わない)。
- 数値は 1 以上。秒に換算した値が 1 以上 86,400(24 時間)以下。桁あふれ(64 ビット符号なし整数に収まらない、または換算の乗算があふれる)は拒否。先頭の 0 は何個並べてもよい(`0000000000000000000000000001s` は値 1 で受理)。値が 86,400 秒を超えるものは、桁数にかかわらず拒否する。**判定は、桁数を先に見る**: 先頭の 0 を除いた数字の桁数が 5 を超えるものは、数値に変換せずに範囲外とする(6 桁以上の数は 100,000 以上で、単位が `s` でも 86,400 を超えるので、結果は同じ)。5 桁以下は、`[long]` にして、単位の倍率(1、60、3600)を掛けて、1 以上 86,400 以下かを調べる(`[int]::Parse` ではなく、字句を検査済みなので、`[long]::Parse(text, NumberStyles.None, InvariantCulture)`)。
- 受理: `1s`、`30s`、`5m`、`2h`、`24h`、`1440m`、`86400s`、`01s`(値 1)。拒否: `''`、`s`、`10`、`1.5s`、`-1s`、`+1s`、` 1s`、`1s `、`1 s`、`1d`、`1S`、`１s`、`1ms`、`0x10s`、`1_0s`、`0s`、`0h`、`86401s`、`25h`、`99999999999999999999999s`、`18446744073709551615h`。文字列でない値(整数・真偽値・null)も拒否。
- `.psd1` では、`interval = 30s` のように引用符なしで書くと、値の位置の裸の単語(`CommandAst`)なので、`syntax`(第 4.3 節)。`interval = 30` は整数なので `retryDurationInvalid`。

#### 誤りと検査

| 条件 | `kind` | 行 | 理由 |
|------|--------|----|------|
| `retry` がマップでない(整数、文字列、リスト、null) | `retryNotTable` | `retry` の値の位置(リストのときは、リストの位置) | `` `retry` はマップで書きます `` |
| 必須のキー(`exit_codes`、`max`、`interval`)が無い | `retryMissingKey`(欠けたキーごとに 1 件) | `retry` のマップの位置 | ``必須の値 `<キー名>` がありません`` |
| `exit_codes` がリストでない | `retryExitCodeInvalid` | 値の位置 | `` `exit_codes` は整数の一覧で書きます `` |
| `exit_codes` が空のリスト | `retryExitCodesEmpty` | 値の位置 | `` `exit_codes` が空です `` |
| `exit_codes` の要素が整数でない、32 ビットの範囲外、1〜255 の範囲外(0、256、-1、1.5、`'75'`、`$true` など) | `retryExitCodeInvalid`(要素ごと) | その要素の位置 | `` `exit_codes` の要素は、1 以上 255 以下の整数で書きます `` |
| `max` が整数でない、または 1〜100 の範囲外(0、101、-1、1.5、`'3'`、`$true`) | `retryMaxInvalid` | 値の位置 | `` `max` は、1 以上 100 以下の整数で書きます `` |
| `interval` / `max_interval` が書式・範囲に合わない | `retryDurationInvalid` | 値の位置 | `` `<キー名>` は、整数と単位(`s`、`m`、`h`)で、1 秒以上 24 時間以下に書きます(例: `30s`) ``(`<キー名>` は `interval` か `max_interval`) |
| `multiplier` が整数でない、または 2〜10 の範囲外(1、11、0、-2、`2.0`、`'2'`、`$true`) | `retryMultiplierInvalid` | 値の位置 | `` `multiplier` は、2 以上 10 以下の整数で書きます `` |
| `multiplier` と `max_interval` の片方だけがある | `retryBackoffIncomplete` | `retry` のマップの位置 | `` `multiplier` と `max_interval` は、両方を書くか、両方とも書きません `` |
| `exit_codes` に、そのジョブの成功または警告の終了コードと同じ値がある | `retryExitCodesOverlap` | `exit_codes` の**キー**の位置 | `` `exit_codes` に、成功または警告の終了コードと同じ値があります `` |
| `max_interval` が `interval` より小さい(秒で比べる。等しければ受理) | `retryMaxIntervalTooSmall` | `max_interval` の値の位置 | `` `max_interval` は、`interval` 以上で書きます `` |

- 整数の判定は、`Kind` が `int` のノードの値(`[long]`)で行う。範囲外の整数(`hugeInt`)と真偽値は、範囲外の誤りと同じ扱い。
- 空のマップ(`retry = @{}`)は、`retryMissingKey` が 3 件(`exit_codes`、`max`、`interval` の順)返る(行はすべて同じ。マップの位置は `@{` の位置)。
- `exit_codes` の複数の要素が不正なら、不正な要素ごとに 1 件(例: `@(0, 75, 256)` は 2 件)。
- **重なりの検査**(`retryExitCodesOverlap`)は、定義の他の値と照合する。`exit_codes` が有効(エラーなし)のときだけ行う。
  - 成功の終了コード: `success_exit_codes` を**書かなければ `[0]`**。書いて有効なら、その値。書いたが誤り(無効)のときは、成功との重なりを調べない(警告との重なりは調べる)。
  - 警告の終了コード: `warning_exit_codes` が有効なら、その値(無効なら調べない)。
  - どちらかと 1 つでも共通する値があれば、1 件(重なりの値の数にかかわらず 1 件)。
  - 例: `success_exit_codes = @(0, 5)` と `exit_codes = @(5, 75)` は誤り。`success_exit_codes = @(3)` と `exit_codes = @(3)` も誤り。`success_exit_codes = @(0, 5)`、`warning_exit_codes = @(4)`、`exit_codes = @(75)` は受理。
- `retry` の有無は、ほかのキーの読み取りの結果に影響しない。
- 誤りの文に、入力の値は含めない。

## 9. 検査の順序とエラーの集約・並び

Walk の中の処理順(誤りは、見つかった順に集める):

1. 根のキー検査(`unknownKey`)。
2. `pipeline`: キー検査 → `name` → `description` → `run_id_env`(型の誤り、続けて名前の形の誤り) → `remove_on_complete` → `name` の空の検査 → 名前の照合。
3. `job`: 無い/空/型の誤りの処理。リストなら、各ジョブを順に、次の順で検査する: キー検査 → `id` → `command` → `warning_exit_codes` → `success_exit_codes` → `transitions`(各遷移について、キー検査 → `on` → `next` → 重複) → `retry`(キー検査 → `exit_codes` → `max` → `interval` → `multiplier` → `max_interval` → 倍率と上限の片方だけの検査)。
4. 全ジョブにまたがる検査(第 9.1〜9.4 節): ジョブIDの検査 → 各ジョブについて、遷移先 → 終了コード(`zeroAsWarning`、`overlappingExitCodes`) → `retry` の重なり・上限と間隔の大小。
5. 誤りが 1 件もなく、`pipeline` とすべてのジョブが組み立てられるときだけ、定義を返す。

返す誤りの順序: すべての誤りを、**位置(行、同じ行なら列)の昇順**に並べて返す。同じ位置のものは、上の検査の順(集めた順)を保つ(**安定な並べ替え**)。**`Sort-Object` の安定性は、公式ドキュメントで確認していないので、頼らない**(未確認)。誤りに、集めた順の通し番号(`Seq`)を付け、`(Line, Column, Seq)` を比較する自前の比較で並べる(`[System.Array]::Sort` に `System.Comparison[object]` を渡してよいが、スクリプトブロックを .NET のデリゲートにするときの実行空間の問題(確認: スクリプトブロックを別スレッドから呼ぶと実行空間がなく失敗する。この並べ替えは同じスレッドで呼ばれるので、通常は問題にならない。未確認。試験 T-17)を避けるため、挿入ソートまたはマージソートの自前の実装でもよい)。位置の取り方は第 11 節。

- 型の誤りなどで検査できなくなった値は、その値に依存する後続の検査だけを省略する(他の検査は続ける)。
- 位置が (1, 1) の誤りは、行 1 として扱われる(`pipeline` が無い、`job` が無い)。

複数の誤りの例:

```powershell
@{
    format_version = 1
    unknown_top = $true          # 3 行目: unknownKey
    pipeline = @{
        name = 'p'
    }
    job = @(
        @{
            id = 'a'
            command = 'x'
            success_exit_codes = @('zero')   # 11 行目: typeMismatch
        },
        @{
            id = 'b'
            command = 'y'
            colour = 'red'                   # 16 行目: unknownKey
            transitions = @(
                @{ on = 'success'; next = 'nowhere' }   # 18 行目: unknownTarget(`next` の値の行)
            )
        }
    )
}
```

返る誤り: `unknownKey`(3)、`typeMismatch`(11)、`unknownKey`(16)、`unknownTarget`(18)。

### 9.1 ジョブ間の検査: ジョブID

`id` が文字列であるジョブごとに、記述順に検査する。

- trim して空、または先頭が `@` → `invalidJobId`、`id` の値の位置、理由 ``ジョブID(`id`)は、空でなく、`@` で始まらない文字列で書きます``。この ID は、遷移先の索引に入らない(後続のジョブから、遷移先として参照できない)。
- 既出の ID と完全に同じ(大文字小文字・前後の空白も区別。`' a '` と `'a'` は別) → `duplicateJobId`、2 つ目以降の `id` の値の位置、理由 ``同じジョブID(`id`)が、すでに定義されています``。(既出の ID は `[System.Collections.Generic.HashSet[string]]`(`Ordinal`)で調べる。`__proto__` や `constructor` という ID も、通常の文字列として扱う。)
- 長さの上限、使える文字(空白・記号)の制限は、これ以外にない(定義の検査では。実行時の状態ファイル名などへの影響は、[06. ステートファイル](06-state-file.md)による)。

### 9.2 遷移先

`on` と `next` がともに有効な遷移について、`next` を検査する(`on` が不正な遷移は、検査しない)。

- `next` が `@end`(完全一致): 受理(以降のジョブをスキップして終える)。
- ID の索引に無い(存在しない、または無効な ID): `unknownTarget`、`next` の値の位置、理由 ``遷移先(`next`)のジョブが、定義にありません``。
- 記述順で、自分自身または前のジョブ(位置 ≦ 自分の位置): `backwardOrSelfTarget`、`next` の値の位置、理由 ``遷移先(`next`)は、記述順で後ろのジョブまたは `@end` で書きます``。
- 後ろのジョブの ID は受理する。

### 9.3 終了コード

各ジョブについて(`warning_exit_codes` と `success_exit_codes` が有効なとき):

- `warning_exit_codes` に 0 を含む → `zeroAsWarning`、`warning_exit_codes` の**キー**の位置、理由 ``0 は警告の終了コード(`warning_exit_codes`)にできません``。(`success_exit_codes` の誤りの有無に関わらず検査する。)
- 両方を書き、有効で、共通する値がある → `overlappingExitCodes`、`warning_exit_codes` の**キー**の位置、理由 `` `warning_exit_codes` と `success_exit_codes` に、同じ値があります ``。ただし、共通の値が 0 だけで、すでに `zeroAsWarning` を報告した場合は、重ねて報告しない(例: `warning_exit_codes = @(0)` と `success_exit_codes = @(0)` は `zeroAsWarning` の 1 件だけ)。
- `success_exit_codes` を書かないとき、成功は `[0]` だが、`warning_exit_codes` との重なりは(書かれていない側は)調べない。警告に 0 を入れた場合は、`zeroAsWarning` で拒否される。

### 9.4 `retry` との関係

第 8.5 節(重なり、上限と間隔の大小)。

## 10. エラーの種類と表示

### 10.1 エラーの構造

型の骨格(例。`[pscustomobject]`。`class` は使わない。第 6.1 節):

```powershell
# 誤り 1 件
[pscustomobject]@{
    Kind   = [string]'syntax'      # 第 10.3 節の 31 種の識別子のいずれか
    File   = [string]'conductor.psd1'   # 表示用のファイル名(第 2.1 節)
    Line   = [Nullable[int]]1      # 1 始まり。$null は「行なし」
    Column = [Nullable[int]]1      # 1 始まり。$null は「列なし」。順序づけにだけ使い、表示には出さない
    Reason = [string]'…'           # 固定の文から組み立てた理由(日本語)。入力の値は含めない
    Seq    = [int]0                # 集めた順の通し番号(並べ替えの安定のため)
}

# 結果(成功)
[pscustomobject]@{ Ok = $true;  Definition = $definition }
# 結果(失敗)
[pscustomobject]@{ Ok = $false; Errors = [object[]]$errors }   # 1 件以上
```

- 種類の識別子の集合は、定数の配列 `$script:ConductorErrorKinds`(31 要素。第 10.3 節の表の順)で持つ。`[ValidateSet(...)]` を使ってよい(31 個を、表と同じに並べる)。単体テストで、集合の要素数が 31 であること、表(日本語・英語)の全項目が揃うことを検査する(PowerShell は、型でキーの網羅を検査できない)。
- `[Nullable[int]]` に `$null` を入れる、`[int]` を入れる、の扱い: `Set-StrictMode -Version Latest` の下でも、`[Nullable[int]]` は `$null` を保てる。比較は `$null -eq $e.Line`(`$null` を左辺)。

| フィールド | 内容 |
|------------|------|
| `File` | 表示用のファイル名 |
| `Line` | 行番号(1 始まり)。位置が決まらないときは無し |
| `Column` | 列番号(1 始まり)。同じ行の中での順序づけのために保持する。**表示には出さない** |
| `Kind` | 種類(第 10.3 節) |
| `Reason` | 固定の文から組み立てた理由。入力の値は含めない。入力由来はキー名だけ(`unknownKey`) |

ライブラリ(読み込み処理の呼び出し側)に返す表示の書式:

```
<file>:<line>: <種類の文>: <reason>      行があるとき
<file>: <種類の文>: <reason>             行が無いとき
```

例: ``conductor.psd1:7: 定義にないキーです: キー `x` は定義にありません``。ここで `<file>` は、制御文字・書式文字(第 10.2 節と同じ対象)をエスケープし、エスケープ後の文字数で 256 文字までに切る(超えるときは、256 文字までで切って末尾に `…` を付ける)。

`conductor` コマンドの標準エラーへの表示は、`conductor: error: <file>[:<line>]: <種類の文(言語による)>: <reason>` の 1 行(誤りごとに 1 行、位置の順)で、`reason` は日本語のまま(言語によらない)。種類の文は、メッセージの言語([08](08-output-and-i18n.md))が日本語のとき日本語(第 10.3 節の「日本語」)、それ以外は英語(「英語」)。`<file>` と `reason` は、改行・制御文字・書式文字をエスケープして表示する(`\n`、`\r`、`\t`、その他は `` \u{16進小文字} ``)。**ここでは文字数で切らない**。例:

```
conductor: error: bad.psd1:7: unknown key: キー `typo` は定義にありません
conductor: error: conductor.psd1: definition file does not exist: 指定の場所にファイルがありません
```

定義の誤りがあるときの終了コード(64)と、その標準出力・標準エラーの扱いは、[10. エラーと終了コード](10-errors-and-exit-codes.md)に従う。

### 10.2 キー名の表示

`unknownKey` の理由に入るキー名は、`` ` `` で囲み、次の整形をする。

- 改行を `\n`、復帰を `\r`、タブを `\t` に、その他の制御文字(U+0000〜U+001F、U+007F〜U+009F)と書式文字(U+200B〜U+200F、U+2028、U+2029、U+202A〜U+202E、U+2060〜U+2064、U+2066〜U+2069、U+FEFF)を `` \u{16進小文字} `` にエスケープする(例: U+0001 は `\u{1}`)。16 進は `$cp.ToString('x', [System.Globalization.CultureInfo]::InvariantCulture)`(先頭の 0 なし、小文字)。**対にならないサロゲート(U+D800〜U+DFFF)も、同じ形でエスケープする**(例: U+DC80 は `\u{dc80}`)。.NET が、OS から得た文字列(コマンドラインの引数、環境変数、ファイル名)の不正なバイトを、どのように `[string]` にするか(U+FFFD か、単独のサロゲートか)は未確認([08](08-output-and-i18n.md))なので、`<file>` に単独のサロゲートが入る可能性を、防御的に扱う。定義の中のキー名には、単独のサロゲートは含まれない(入力が厳格な UTF-8 の復号と、JSON のエスケープの検査を通るため)ので、この規定が働くのは、`<file>` だけである。
- エスケープ後の文字数(コードポイントの数)が **64 文字**を超えるときは、64 文字までで切って末尾に `…` を付ける(エスケープの途中では切らない。たとえば、残りが 2 文字でエスケープが 6 文字なら、そのエスケープは入れず、そこで切る)。ちょうど 64 文字は切らない。
- この整形は、`file`(256 文字)の整形と同じ手順で、上限だけが違う。
- 文字列は、コードポイントごとに走査する(上位サロゲートと下位サロゲートの対を 1 つとして数える。`[char]::IsHighSurrogate`、`[char]::IsLowSurrogate` は使ってよい。文化に依存しない)。`[System.Text.Rune]` と `EnumerateRunes()` は、不正なサロゲートで置換文字を返す(未確認)ので、使わない。

### 10.3 種類の一覧(31 種)

「識別子」は機械が読む名前、「日本語」は日本語の種類の文、「英語」は英語の種類の文。

| 識別子 | 日本語 | 英語 |
|--------|--------|------|
| `unsupportedFormat` | 定義ファイルの形式に対応していません | unsupported definition file format |
| `fileMissing` | 定義ファイルが存在しません | definition file does not exist |
| `unreadable` | 定義ファイルを読めません | cannot read definition file |
| `tooLarge` | 定義ファイルが大きすぎます | definition file is too large |
| `syntax` | 文法の誤りです | syntax error |
| `typeMismatch` | 値の型が正しくありません | wrong value type |
| `unknownKey` | 定義にないキーです | unknown key |
| `versionMissing` | 書式の版がありません | format_version is missing |
| `versionUnsupported` | 対応していない書式の版です | unsupported format_version |
| `unknownTarget` | 遷移先のジョブが定義にありません | transition target job is not defined |
| `errorTransition` | エラー結果には遷移先を書けません | no transition can be written for the error result |
| `backwardOrSelfTarget` | 遷移先は、記述順で後ろのジョブまたは @end のみです | transition target must be a later job or @end |
| `overlappingExitCodes` | 成功と警告の終了コードが重なっています | success and warning exit codes overlap |
| `zeroAsWarning` | 0 は警告の終了コードにできません | 0 cannot be a warning exit code |
| `pipelineNameMismatch` | パイプライン名が定義と一致しません | pipeline name does not match the definition |
| `duplicateJobId` | ジョブIDが重複しています | duplicate job id |
| `invalidJobId` | ジョブIDが正しくありません | invalid job id |
| `noJobs` | ジョブが 1 つもありません | no jobs are defined |
| `emptyField` | 必須の値が空です | required value is empty |
| `duplicateTransition` | 同じ結果の遷移が重複しています | duplicate transition for the same result |
| `invalidEnvName` | 環境変数の名前が正しくありません | invalid environment variable name |
| `retryNotTable` | retry はマップで書きます | retry must be a map |
| `retryMissingKey` | retry の必須のキーがありません | a required key of retry is missing |
| `retryExitCodesEmpty` | retry の終了コードの一覧が空です | retry exit_codes is empty |
| `retryExitCodeInvalid` | retry の終了コードが正しくありません | retry exit_codes has an invalid exit code |
| `retryExitCodesOverlap` | retry の終了コードが、成功または警告の終了コードと重なっています | retry exit_codes overlap the success or warning exit codes |
| `retryMaxInvalid` | retry の最大回数が正しくありません | retry max is invalid |
| `retryDurationInvalid` | retry の間隔が正しくありません | retry interval is invalid |
| `retryBackoffIncomplete` | retry の倍率と間隔の上限は、両方を書くか、両方とも書きません | retry multiplier and max_interval must be given together or not at all |
| `retryMultiplierInvalid` | retry の倍率が正しくありません | retry multiplier is invalid |
| `retryMaxIntervalTooSmall` | retry の間隔の上限が、間隔より小さくなっています | retry max_interval is smaller than interval |

(`retryDurationInvalid` の英語は、`max_interval` の誤りにも同じ文が使われる。`retryNotTable` の識別子は、論理的な名前のまま残し、文は「マップ」と言う。)

実装は、種類の文を、`[System.Collections.Generic.Dictionary[string, string[]]]`(識別子 → 日本語、英語)の定数の表で持つ(`[System.StringComparer]::Ordinal`)。

理由(`reason`)は、第 3〜9 節の各表の日本語の原文のとおり。英語の理由文は存在しない(理由は常に日本語)。理由の文中の `` ` `` は、文の一部としてそのまま出力する。`syntax`、`tooLarge`、`typeMismatch` の、読み取り(段階 2・3)の理由は、次のとおり。

| 種類 | 理由(日本語の原文) | 出す場所 |
|------|----------------------|----------|
| `syntax` | `.psd1 の文法に誤りがあります` | `.psd1` の文法の誤り(`ParseError` があるもの、前処理(第 4.8 節)での拒否、文が 0 個、サロゲートの不正、例外) |
| `syntax` | `.psd1 で使えない書き方です` | `.psd1` の、許可しない構文(第 4.2〜4.6 節) |
| `syntax` | `JSON の文法に誤りがあります` | JSON の文法の誤り(第 5 節) |
| `syntax` | `キーが重複しています` | 第 4.5、5 節 |
| `syntax` | `キーは文字列で書きます` | 第 4.5 節(`.psd1`) |
| `syntax` | `入れ子が深すぎます(上限は 32 段)` | 第 4.8、6.3、6.4 節 |
| `tooLarge` | `要素数が上限(100000)を超えています` | 第 6.3 節 |
| `tooLarge` | `文字の大きさが上限(16 MiB)を超えています` | 第 6.3 節 |
| `typeMismatch` | `最上位はマップで書きます` | 第 6.7 節 |

## 11. 位置(行番号と列番号)の出し方

### 11.1 位置の単位

- 位置は、行と列の組。**行は 1 始まりで、LF の数で数える**(CRLF でも、CR は直前の行に属し、LF のあとが次の行。`.psd1` は、第 4.8 節で CRLF を LF にそろえてから位置を求めるので、結果は同じ)。**列は 1 始まりで、その行の先頭からのコード単位(`[char]`)の数 + 1**。`.psd1` も JSON も、同じ単位である(入力の `[string]` の添字から、実装が求める)。**列は、同じ行の中の誤りの並びを決めるためだけに使い、行が違えば比べない**。同じ行の中で、前にあるものほど小さい列を返す限り、単位はどちらでもよい。
- **`.psd1` の行と列は、`IScriptExtent` の `StartOffset` から求める**(決定。確認(出典: IScriptExtent の API リファレンス): `StartLineNumber`、`StartColumnNumber` は 1 始まり(「1 being the first line / first column」)。`StartOffset` の単位は、API リファレンスに記述がない(未確認。入力の `string` の添字(コード単位)と推測される。試験 T-03))。理由: `StartLineNumber` が、LF 以外(CR だけ、U+0085 など)を行の区切りとして数える可能性があるので(確認(出典: tokenizer のソース): トークナイザーは、CR、LF、CRLF を改行として行の先頭を記録する)、規定は `StartOffset` から求める。第 4.8 節の前処理で、CR だけの改行と U+0085・U+2028・U+2029 を拒否し、CRLF を LF にしているので、`StartLineNumber` は、受理する入力では、LF の数に一致する(試験 T-03 で、`StartLineNumber`・`StartColumnNumber` との一致を確かめる)。
- 求め方: 入力の `[string]`(BOM を除き、CRLF を LF にしたもの)について、LF の添字の表(`$lineStarts`: `[System.Collections.Generic.List[int]]`。先頭 0 と、各 LF の添字 + 1)を作り、二分探索で、`$lineStarts[$k] -le $offset` を満たす最大の `$k` を求める。行は `$k + 1`、列は `$offset - $lineStarts[$k] + 1`。二分探索は、`[System.Collections.Generic.List[int]]::BinarySearch` の戻り値(見つからないとき、次に大きい要素の添字の補数)を扱う自前の関数にするか、手書きの二分探索にする。範囲外のオフセットは、最後の行。
- 先頭の BOM は、位置には数えない(1 行目の 1 列目は、BOM の次の文字)。
- 位置が決まらない誤りは、行なし(取り込み、`versionMissing`)。
- 位置が (1, 1)(入力の先頭)の誤りは、`pipeline` が無い、`job` が無い。

### 11.2 ノードの位置

`.psd1` のノードの位置は、AST のノードの `Extent.StartOffset` を、第 11.1 節の求め方で行・列にしたもの。

| ノード | 位置 |
|--------|------|
| スカラー(文字列、数、`$true`/`$false`/`$null`) | 値の先頭の文字(引用符つきなら、開く引用符。here-string なら `@` の位置。負の数なら `-` の位置) |
| マップ | `@{` の `@` の位置(`HashtableAst.Extent.StartOffset`) |
| `@(...)` のリスト | `@(` の `@` の位置(`ArrayExpressionAst.Extent.StartOffset`) |
| カンマ区切りのリスト(`a = 1, 2`) | 最初の要素の位置(`ArrayLiteralAst.Extent.StartOffset`) |
| 単項のカンマのリスト(`,x`) | `,` の位置(`UnaryExpressionAst.Extent.StartOffset`) |
| マップのキー(`KeyPos`) | キーの式の先頭(引用符つきなら、開く引用符。裸の単語なら、その先頭) |
| `@(...)` の中の要素 | 要素の式の先頭 |
| 値が省略された対 | `.psd1` には、値の省略がない(`a =` は `ParseError`) |

JSON のノードの位置は、オブジェクトは `{`、配列は `[`、スカラーは字句の先頭、キーは開きの `"`。

- 「値の位置」とは、そのキーの値のノードの位置。「キーの位置」とは、`KeyPos`。「マップの位置」とは、上の表のマップのノードの位置(必須キーの欠落、`retryMissingKey`、`retryBackoffIncomplete` で使う)。

### 11.3 誤りの位置の取り方(まとめ)

- 型の誤りや範囲の誤りは、**誤った値のノードの位置**。
- 未知のキーは、**キーの位置**。
- 必須のキーの欠落は、**それを含むマップの位置**。
- `pipeline` や `job` 自体が無いときは、(1, 1)。
- 終了コードの重なり(`overlappingExitCodes`、`zeroAsWarning`、`retryExitCodesOverlap`)は、対象の**キー**の位置(値ではない)。
- リストの誤りは、要素ごとに、その要素の位置。
- 取り込みの誤り、`unsupportedFormat`、`versionMissing` は、行なし。
- 段階 3 の誤りは、そのノードの位置(キーの誤り(重複、非文字列)はキーの位置、深さの誤りは 33 段目のコンテナの位置、上限(要素数、大きさ)の誤りは、超えたノードの位置)。

### 11.4 文法の誤りの行

- **JSON**: 第 5.2 節のとおり、必須。
- **`.psd1`**: 文法の誤り(`ParseError`)は、`errors` のうち `Extent.StartOffset` が最小のものの位置の行。検出位置は、パーサーの作りで変わる(原因の行より後ろになることも、前になることもある)。**必須なのは、種類(`syntax`)と理由(``.psd1 の文法に誤りがあります``)であり、行は、パーサーが返すものでよい**。次は、期待する行(試験 T-18 で確かめる。未確認):

| 入力(`\n` は改行) | 期待する行 |
|------|----|
| `@{ a = 1` + `\n`(閉じていない) | 入力の末尾の行(2) |
| `@{\n  a = 1\n  b = \n}` | 3(`b =` の行) |
| `@{ a = 'x` + `\n` + `}`(閉じていない文字列) | 文字列の開始の行(1)または末尾の行 |
| 前処理で拒否: `@{ a = 1 }\r` のように CR だけ | CR の位置の行 |

## 12. 取り込み後のデータモデルと検査の関数

### 12.1 データモデル

検証に成功すると、次のデータを返す(`[pscustomobject]`。変更しない値として扱う)。

```powershell
# Definition
[pscustomobject]@{
    Name             = [string]'nightly'         # 空白だけでない(trim していない元の値)
    Description      = [string]'…'               # 無ければ $null
    RunIdEnv         = [string]'GITHUB_RUN_ID'   # 名前の形は検査済み。$null なら実行時に CONDUCTOR_RUN_ID
    RemoveOnComplete = [bool]$false              # 省略時 $false
    FormatVersion    = [int]1
    Jobs             = [object[]]@(…)            # 記述順を保つ。1 つ以上
}
# Job
[pscustomobject]@{
    Id               = [string]'a'               # 空白だけでなく、@ で始まらず、定義内で一意
    Command          = $command                  # 下の Command
    WarningExitCodes = [int[]]@()                # 省略時は空。0 を含まない。書いた順のまま。32 ビット符号つき
    SuccessExitCodes = [int[]]@(0)               # 省略時 $null(意味は 0 だけ)。書いた順のまま
    Transitions      = [object[]]@()             # 書いた順。on ごとに高々 1 つ
    Retry            = $retry                    # 書かなければ $null
}
# Command(どちらの書き方か保つ)
[pscustomobject]@{ Form = 'line'; Line = [string]'echo x' }
[pscustomobject]@{ Form = 'argv'; Argv = [string[]]@('prog', 'a b') }   # 1 要素以上。先頭は空白だけでない
# Transition
[pscustomobject]@{ On = 'success'; Next = 'job'; JobId = 'b' }   # Next は 'job' | 'end'(end のとき JobId は持たない)
# RetryPolicy
[pscustomobject]@{
    ExitCodes       = [int[]]@(75)               # 昇順・重複なし・各 1〜255・空でない
    Max             = [int]3                     # 1〜100
    IntervalSecs    = [uint64]5                  # 1〜86400
    Backoff         = $backoff                   # $null なら固定間隔。{ Multiplier = [int](2〜10); MaxIntervalSecs = [uint64](interval 以上、86400 以下) }
}
```

- 成功として扱う終了コード = `SuccessExitCodes` があればそれ、なければ `@(0)`。
- ジョブIDから記述順の位置(0 始まり)を引く操作がある(遷移先の検査と、実行時の遷移の解決に使う)。`[System.Collections.Generic.Dictionary[string, int]]`(`Ordinal`)で持つ。
- **配列の単一化に注意**: `[object[]]` や `[int[]]` を返す関数は、`return ,$array`(単項のカンマ)で返す。1 要素の配列を、スカラーとして受け取ってしまうのを避けるため。受け側は、`[int[]]` の型を付けた変数に代入して使う。空の配列は、`$null` と区別する(`$null -eq $x`)。
- 取り込み後の整数(終了コード、`Max`、`Multiplier`)は、`Kind` が `int` のノードの `[long]` から、範囲を検査したうえで `[int]` に変換する(`[int]$long` は、範囲内でだけ呼ぶ)。`[uint64]` の秒数は、1〜86,400 の範囲に収まった `[long]` から作る。

### 12.2 公開する関数の署名(例)

`src/Conductor/Private/Definition/` が提供する関数(名前は規定。[02. アーキテクチャ](02-architecture.md)と一致させる。`Verb-ConductorNoun`。公開しない)。

```powershell
function Get-ConductorFormatFromPath {
    [CmdletBinding()]
    [OutputType([string])]   # 'psd1' | 'json' | $null(unsupportedFormat)
    param([Parameter(Mandatory)][string] $Path)
}

function Read-ConductorDefinitionFile {
    [CmdletBinding()]
    [OutputType([pscustomobject])]   # 結果(Ok / Errors)
    param(
        [Parameter(Mandatory)][string] $Path,
        [AllowNull()][string] $ExpectedPipelineName
    )
    # 段階 1〜5。ファイルに触れる。例外を投げない(想定外の不具合を除く)
}

function ConvertFrom-ConductorDefinitionText {
    [CmdletBinding()]
    [OutputType([pscustomobject])]
    param(
        [Parameter(Mandatory)][string] $Text,
        [Parameter(Mandatory)][ValidateSet('psd1', 'json')][string] $Format,
        [Parameter(Mandatory)][string] $FileLabel,
        [AllowNull()][string] $ExpectedPipelineName
    )
    # 段階 2〜5。ファイルに触れない
}

function Get-ConductorRunIdEnv {
    [CmdletBinding()]
    [OutputType([string])]   # watch・reset 用のゆるい読み取り(第 13 節)。どの失敗でも $null
    param([Parameter(Mandatory)][string] $Path)
}

function Read-ConductorTree {
    [CmdletBinding()]
    [OutputType([pscustomobject])]   # { Ok; Root; FirstUnrepresentable } または { Ok = $false; Error }
    param(
        [Parameter(Mandatory)][string] $Text,
        [Parameter(Mandatory)][ValidateSet('psd1', 'json')][string] $Format,
        [Parameter(Mandatory)][string] $FileLabel,
        [ValidateSet('psd1', 'json')][string] $ConvertTo   # 変換器のときだけ(09 章)
    )
    # 段階 2〜3(読み取りと展開)。変換器と共有する
}

function Read-ConductorJsonTree {
    [CmdletBinding()]
    [OutputType([pscustomobject])]   # { Ok; Root; DuplicateKey } または { Ok = $false; Error }
    param(
        [Parameter(Mandatory)][string] $Text,
        [switch] $DeferDuplicateKey
    )
    # JSON の読み取り(第 5.3 節)。ステートの読み込み(06 章)も使う。
    # -DeferDuplicateKey のとき、重複キーで止まらず、最初の重複の位置を DuplicateKey に付けて続ける。
    # 入れ子の深さ・要素数・文字の大きさの上限は、Test-ConductorTreeLimits が適用する。
}

function Read-ConductorPsd1Tree {
    [CmdletBinding()]
    [OutputType([pscustomobject])]   # { Ok; Root; FirstUnrepresentable } または { Ok = $false; Error }
    param(
        [Parameter(Mandatory)][string] $Text,
        [Parameter(Mandatory)][string] $FileLabel
    )
    # 第 4 節(前処理、ParseInput、AST の検査と変換、重複キー、上限)
}

function Test-ConductorTreeLimits {
    [CmdletBinding()]
    [OutputType([pscustomobject])]   # 誤り 1 件、または $null
    param(
        [Parameter(Mandatory)][pscustomobject] $Root,
        [Parameter(Mandatory)][ValidateSet('all', 'depth_only')][string] $Which
    )
}
```

内部の検査の関数(名前は任意。木と、誤りを集める入れ物を受け取り、定義の部品を返す。誤りは入れ物に足す。例外を投げない):

```powershell
function New-ConductorWalkContext {
    [OutputType([pscustomobject])]   # { File; Errors = List[object]; NextSeq }
    param([Parameter(Mandatory)][string] $File)
}
function Add-ConductorError {
    [OutputType([void])]
    param(
        [Parameter(Mandatory)][pscustomobject] $Context,
        [Parameter(Mandatory)][string] $Kind,
        [AllowNull()][pscustomobject] $Pos,
        [Parameter(Mandatory)][string] $Reason
    )
}
function Test-ConductorWalkRoot     { param($Root, $Context, [AllowNull()][string] $ExpectedName) }   # → Definition または $null
function Test-ConductorWalkPipeline { param($Node, $RootPos, $Context, [AllowNull()][string] $ExpectedName) }
function Test-ConductorWalkJob      { param($Node, $Context) }
function Read-ConductorExitCodes    { param($Entry, $Context) }   # → { Codes; Valid }
function ConvertFrom-ConductorInterval {
    [OutputType([Nullable[uint64]])]   # 秒。書式・範囲に合わなければ $null
    param([Parameter(Mandatory)][string] $Text)
}
function Test-ConductorJobSet       { param($Jobs, $Context) }   # 第 9 節(ジョブ間の検査)
```

- `[OutputType()]` は、ヒントであり、PowerShell は型を強制しない(未確認: 公式ドキュメントでは、`OutputType` は「出力の型を宣言するもの」と説明されると推測される。この仕様は、強制に頼らない)。戻り値の型は、`return` する値の型を、呼び出し側が確かめる(単体テストで検査する)。
- 誤りの並べ替え(位置の昇順、安定)は、`Read-ConductorDefinitionFile`/`ConvertFrom-ConductorDefinitionText` の最後に 1 回だけ行う(第 9 節)。Walk の誤りは、すべて位置を持つ。

## 13. `run_id_env` のゆるい読み取り

`watch` と `reset` は、`--run-id` が無いとき、定義全体を検証せず、`pipeline.run_id_env` だけを取り出す。

- 書式の決定(第 2.1 節)から段階 3(第 6 節の上限を含む)までは、定義の読み込みと同じに行う。段階 4・5 は行わない。
- 拡張子が対応していない、ファイルを読めない、大きさの上限を超える、`.psd1`・JSON として読めない(上限を含む)、最上位がマップでない、`pipeline` がマップでない、キーが無い、文字列でない、名前の形(第 8.1 節)に合わない、のどれでも「無し」(`$null`。実行時は `CONDUCTOR_RUN_ID`)。
- ほかの誤り(`format_version` の誤り、未知のキーなど)は無視する。壊れた入力でも異常終了しない(例外を投げず、`$null` を返す。`try`/`catch` で `[System.Exception]` を捕まえる)。
- ファイルの大きさの上限(第 3 節)は、この読み取りにも適用する(超えれば「無し」)。
- 例(`.psd1`): `@{ pipeline = @{ name = 'p'; run_id_env = 'X_ID' } }` → `X_ID`。`run_id_env = 5`、`'1A'`、`''` → 無し。`pipeline = 3` → 無し。空の入力 → 無し。

## 14. 受理・拒否の例

最小の定義(受理):

```powershell
@{
    format_version = 1
    pipeline = @{ name = 'p' }
    job = @(
        @{ id = 'a'; command = 'c' }
    )
}
```

全キーを使った定義(受理):

```powershell
@{
    format_version = 1

    pipeline = @{
        name = 'nightly'
        description = '夜間バッチ'
        run_id_env = 'GITHUB_RUN_ID'
        remove_on_complete = $true
    }

    job = @(
        @{
            id = 'extract'
            command = @('pwsh', 'scripts/extract.ps1', '')
            warning_exit_codes = @(4)
            success_exit_codes = @(0, 5)
            retry = @{
                exit_codes = @(75, 111)
                max = 5
                interval = '10s'
                multiplier = 2
                max_interval = '60s'
            }
            transitions = @(
                @{ on = 'warning'; next = 'notify' }
            )
        },
        @{
            id = 'load'
            command = 'load'
            transitions = @(
                @{ on = 'success'; next = '@end' }
            )
        },
        @{
            id = 'notify'
            command = 'notify --channel ops'
        }
    )
}
```

別の書き方でも同じ定義になる例(受理。引用符つきのキー、カンマ区切りのリスト、1 行のハッシュテーブル):

```powershell
@{
    'format_version' = 1
    pipeline = @{ name = 'p' }
    job = ,@{ id = 'a'; command = 'c'; retry = @{ exit_codes = ,75; max = 3; interval = '5s' } }
}
```

(`job = ,@{...}` は、1 要素のリスト。`exit_codes = ,75` も 1 要素のリスト `[75]`。`exit_codes = 75` と書くと、リストでないので `retryExitCodeInvalid`。)

拒否の例(行番号は、1 行目 `@{`、2 行目 `format_version = 1`、3 行目 `pipeline = @{ name = 'p' }`、4 行目 `job = @(`、5 行目 `@{`(ジョブの開始)のあと、6 行目 `id = ...`、7 行目 `command = ...`、8 行目 `transitions = ...` と続くものとしての値):

| 入力(ジョブの本体) | 結果 |
|------|------|
| `id = 'a'` 7 行目 `command = 'c'` 8 行目 `transitions = @(@{ on = 'success'; next = 'zzz' })` | `unknownTarget` 8 行目 |
| 同、`on = 'error'; next = '@end'` | `errorTransition` 8 行目 |
| ジョブ a の `next = 'a'`(自分自身) | `backwardOrSelfTarget` 8 行目 |
| `warning_exit_codes = @(2, 3)` と `success_exit_codes = @(0, 3)` | `overlappingExitCodes`(`warning_exit_codes` のキーの行) |
| `warning_exit_codes = @(0)` | `zeroAsWarning`(そのキーの行) |
| `warning_exit_codes = @('x')` | `typeMismatch`(その要素の行) |
| `id = 1` | `typeMismatch` 6 行目 |
| `command` が無い(`@{` と `id` だけ) | `typeMismatch` 5 行目(`@{` の行) |
| `warn_exit_codes = @(1)` | `unknownKey`(そのキーの行) |
| `id = ''` / `id = '@end'` | `invalidJobId` 6 行目 |
| 同じ `id` の 2 つ目 | `duplicateJobId`(2 つ目の `id` の行) |
| `command = '  '` | `emptyField`(`command` の値の行) |
| ヘッダのみ(ジョブなし: `job = @()`) | `noJobs`(`job` の値の行) |
| `retry = @{ exit_codes = @(0); max = 0; interval = '0s'; multiplier = 1; max_interval = '0s'; unknown = 1 }`(複数行で書く) | `retryExitCodeInvalid`、`retryMaxInvalid`、`retryDurationInvalid`、`retryMultiplierInvalid`、`retryDurationInvalid`、`unknownKey`(この順。行は各キーの値の行) |

そのほかの例: 行番号は CRLF の改行でも同じ(前処理で LF にそろえるので、LF の数)。`@{ format_version = 1 }` だけのファイル、`format_version = 1.0`、`format_version = 2`、`format_version = 0x01`(+ 有効な定義)は、順に `typeMismatch`(`pipeline` が無い)かつ `noJobs`(段階 4 を通ったあとの Walk で 2 件)、`versionUnsupported`(整数でない)、`versionUnsupported`(対応していない版)、受理。

## 15. PowerShell 固有の落とし穴(規定に入れたもの)

| 落とし穴 | 内容(確認の別) | この文書の規定 |
|----------|----------------|----------------|
| ハッシュテーブルの順序 | `@{}` は順序を保証しない。順序つきは `[ordered]`(未確認: 公式ドキュメントを取得して確かめていない) | 木にハッシュテーブルを使わない(第 6.1 節) |
| ハッシュテーブルのキーの大文字小文字 | 既定で区別しない(同上) | `.psd1` のキーの重複は、ASCII の大文字小文字を畳んで拒否(第 4.5 節) |
| 配列の単一化・平坦化 | 関数の出力は配列に集められ、1 要素なら要素そのもの。`@(@(1,2))` は `1, 2` | 戻り値は `return ,$x`(第 12.1 節)。`.psd1` の単独の入れ子 `@(@(...))` は拒否(第 4.6 節) |
| `$null` との比較 | 配列が左辺だと絞り込み | `$null -eq $x`(第 1.1 節) |
| `-eq` の文字列比較 | 大文字小文字を区別しない | `-ceq`、`Ordinal`(第 1.1 節) |
| スマートクォート | U+2018/2019/201A/201B/201C/201D/201E が引用符として扱われる(確認(出典: CharTraits のソース、about_Quoting_Rules)) | 書き出しは `` `u{…} `` で書く([09. 変換器](09-converter.md)の文字列の書式) |
| 全角ダッシュ | U+2013/2014/2015 が `-` として扱われる(確認(出典: CharTraits のソース)) | ASCII の `-` だけを受理(第 4.3.1 節) |
| 数の型の自動選択 | 接尾辞・桁数で型が変わる(確認(出典: tokenizer のソース)) | `Extent.Text` から自前で求める(第 4.3.1 節) |
| クラスの再読み込み | できない(確認(出典: about_Classes)) | `[pscustomobject]`(第 6.1 節) |
| `Sort-Object` の安定性 | 未確認 | 通し番号つきの自前の比較(第 9 節) |
| `String.Trim()` の空白 | 版で集合が変わりうる | 自前の 25 文字の表(第 1 節) |
| `ForEach-Object` の `break`/`continue` | ループでなくパイプラインを抜ける | 走査に `ForEach-Object` を使わない(第 5.3 節 #1) |
| `Set-StrictMode` | 未定義のプロパティの参照が例外になる | 種別ごとに、持つプロパティだけを参照する(第 6.1 節) |
| `+=` の連結 | 配列を作り直す | `List[object]`、`StringBuilder`(第 1.1 節) |
| メソッドの戻り値の漏れ | `StringBuilder.Append` は `StringBuilder` 自身を、`HashSet[string].Add` は `bool` を返し、文として書くと関数の出力に混ざる | `[void]` か `$null =` で捨てる(冒頭) |
| 実行ポリシー・Constrained Language Mode | `.psd1` を読むだけの処理は、`Parser`(.NET の型の呼び出し)を使うので、Constrained Language Mode では、`[System.Management.Automation.Language.Parser]` の型リテラルの呼び出しが制限される可能性(未確認。試験 T-19)。実行ポリシーは、この仕様のデータファイルの読み取り(`ParseInput` による構文解析だけ)には、影響しないと推測される(未確認) | 未確認を、第 17 節に記す |

## 16. 実装時に最初に確かめる小さな試験(実機が必要なもの)

この環境では動かせなかった。実装の最初に、最小・最新の PowerShell で、3 つの OS で行い、結果に合わせて、この文書(特に第 4 節、第 6.4 節、第 11.4 節)を更新する。

| 番号 | 試験 | 確かめること |
|------|------|--------------|
| T-01 | `ParseInput` の出力引数 | `[ref]` で呼び、`ParseError[]` と `Token[]` が得られること。`$errors.Count` で判定できること |
| T-02 | 空・コメントだけの入力 | `EndBlock` が `$null` か空か、`Statements.Count` が 0 か |
| T-03 | 位置 | `StartOffset` が、入力の `string` の添字(UTF-16)であること。`StartLineNumber`・`StartColumnNumber` が、LF だけの入力で、`StartOffset` から求めた値と一致すること。BMP 外の文字を含む行の列 |
| T-04 | 深い入れ子 | `@(` を 1,000、10,000、100,000 並べた入力を `ParseInput` に渡して、異常終了しないか。しきい値(第 4.8 節 #5、第 6.4 節) |
| T-05 | `1e999`、`1e-999`、範囲を超える 16 進、`1_000`、`.5`、`5.`、`01`、`1kb`、`-0.0`、`-9223372036854775808` | `ParseError` になるか、値は何か、`Extent.Text` は何か。`-` の負の数が `ConstantExpressionAst` か `UnaryExpressionAst` か |
| T-06 | `"a$"`、`"$5"`、`"`$a"`、`"${x}"` | `StringConstantExpressionAst` か `ExpandableStringExpressionAst` か |
| T-07 | here-string | `Value` の改行・先頭と末尾の扱い(LF だけの入力) |
| T-08 | `` "`u{D800}" ``、`` "`u{110000}" `` | `ParseError` か、`Value` の内容 |
| T-09 | 裸のキー | `a-b`、`a.b`、`1abc`、`if`、`true`、`$true` が、`StringConstantExpressionAst`(`BareWord`)か、その `Value` |
| T-10 | `name =` の次の行に値を書く | `ParseError` か |
| T-11 | `Import-PowerShellDataFile` との往復 | 書き出した `.psd1` を読んで、値と型が同じか(特に `-9223372036854775808`、`1e16`、`-0.0`、`@(,@(1,2))`) |
| T-12 | `[double]::Parse("1e999999999", Float, Invariant)` | 結果と時間 |
| T-13 | `Utf8JsonReader` | PowerShell から `ref struct` を使えないこと |
| T-14 | `File.GetAttributes`、`FileInfo.Length` | シンボリックリンク、FIFO、デバイスファイル、ENOTDIR の結果 |
| T-15 | `UTF8Encoding(false, true).GetString` | BOM の扱い、過長な符号化・サロゲートの符号化・範囲外・単独の `FF` の例外 |
| T-16 | JSON の読み取りの速度 | 1 MiB(`[` を並べた入力、長い文字列、多数の短い要素)の時間 |
| T-17 | `[System.Array]::Sort` とスクリプトブロックの比較 | 安定性・実行空間の問題 |
| T-18 | `.psd1` の文法の誤りの行 | 第 11.4 節の表 |
| T-19 | Constrained Language Mode | `Parser` の型の呼び出しができるか |
| T-20 | 全 `[char]` の空白の判定 | 第 1 節の 25 文字(`[char]::IsWhiteSpace` との差) |

## 17. 未確認事項

- 第 4.1 節: `ParseFile` が、ファイルの文字コードを、どう判定するか(この仕様は `ParseInput` を使う)。パーサーの内部の再帰の深さの上限(捕捉できない `StackOverflowException` の可能性)。`ParseError` の `Message` は使わない(翻訳され、入力の値を含む)。
- 第 4.2 節: 空のスクリプトの `EndBlock` の状態。`NamedBlockAst` の `Traps`、`Unnamed` の属性名。
- 第 4.3.1 節: 数のトークンの、パーサーの細部(`-` の符号の扱いの文脈、`1e999` の結果、`1_000`、範囲を超える 16 進の失敗の確認はソースの記述による)。
- 第 4.4 節: `"a$"` の分類、`` `u{D800} `` の扱い、here-string の `Value` の細部。
- 第 4.5 節: 裸のキーに使える文字(`a-b` など)。ハッシュテーブルの比較の文化。
- 第 4.7 節: `=` の後ろの改行。
- 第 3 節: シンボリックリンク、FIFO、デバイスファイルでの `FileInfo`/`FileStream` の挙動。UTF-8 の復号が、過長な符号化・サロゲートの符号化・範囲外・単独の `FF` で、`DecoderFallbackException` を投げること。
- 第 5.1 節: `double.Parse` が、桁あふれで例外を投げず ∞ を返すこと(この仕様は、どちらでも `overflow` にする)。指数が 9 桁の巨大な値の処理。
- 第 6.1 節: スクリプトの関数の呼び出しの深さの上限(この仕様は再帰を使わない)。`New-Object` の遅さの数値。
- 第 9 節: `Sort-Object` の安定性。
- 第 10.2 節: .NET が、UTF-8 として不正なバイトを含む引数・環境変数・ファイル名を、どう `[string]` にするか([08](08-output-and-i18n.md))。
- 第 11 節: `StartOffset` の単位(コード単位と推測)。
- 第 15 節: Constrained Language Mode と実行ポリシーの影響。
