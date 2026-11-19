# 03. 定義ファイルの書式と検証

この文書は、`conductor` の定義ファイルの書式と、読み込み時の検証を定める。この文書だけで、C#(.NET)でゼロから、同じ受理・拒否・エラー(種類、文言、行番号)を返す読み込み処理を実装できることを目標とする。

- 定義ファイルの書式は **JSON(主)** と **YAML(副)** の 2 つ。書式は**拡張子**で決める(第 2 節)。既定のファイル名は `conductor.json`。人が書く定義にコメントが要るときは、YAML を使う(JSON はコメントを書けない)。
- 検証は、見つけた誤りを**まとめて**返す。ただし、早い段階の誤り(取り込み、文法、展開と上限、版)は 1 件だけで打ち切る(第 2 節)。
- 定義の内容は、読み込みでは実行も評価もしない。`command` の文字列は、そのまま保持する。
- 確認できなかった点は「未確認」と書く。公式ドキュメントや公開されたソースで確かめた点は「確認(出典: …)」と書く。この環境には .NET SDK が無く、実機では動かしていない。したがって「確認」は、**文書・ソースの記述の確認**であり、動作の確認ではない。実装時に、第 7.5 節の受け入れ試験(YAML)と第 4.4 節の差分試験(JSON)を、採用する版と最小・最新の LTS で、3 つの OS で行う。
- 「必須」と書いた挙動は、利用者から見える結果(受理/拒否、エラーの種類・文言・行番号、取り込み後の値)であり、同じにしなければならない。「任意」「実装に任せてよい」と書いた点は、利用者から見える結果が変わらない範囲で、実装が選んでよい。
- このツールの終了コード、エラー行の標準エラーへの出し方は [10. エラーと終了コード](10-errors-and-exit-codes.md)、メッセージの言語の選び方と標準出力・標準エラーの文字コードは [08. 出力と多言語対応](08-output-and-i18n.md) に従う。この文書は、エラーの内容(種類・位置・理由)までを定める。
- JSON と YAML の読み取り(第 4〜7 節)は、[09. 変換器](09-converter.md)と共有する。実装は、`src/Conductor.Core/` の中に「木の読み取り」の部分(名前空間 `Conductor.Json`(自前の JSON 読み書き)、`Conductor.Definition`(木の型、YAML の読み取り、上限、検査)。ファイル名は任意)を置き、名前空間 `Conductor.Convert` はその公開メソッドを使う([02. アーキテクチャ](02-architecture.md))。
- 実装は、`<Nullable>enable</Nullable>`、`<TreatWarningsAsErrors>true</TreatWarningsAsErrors>` で書く。読み込み処理は、利用者の入力の誤りを**例外ではなく戻り値**(結果の型)で返す。例外は、実装の不具合(想定外の内部失敗)にだけ使う。ただし、ライブラリや BCL が投げる例外は、第 3、7.4 節のとおり、読み込み処理の内側で捕まえて、定められた種類の戻り値にする。
- 文化に依存する API(`ToString()`、`Parse`、`ToUpper`、`string.Compare`、`StartsWith(string)` など)は、この文書の処理では使わない。数の書式化と解析は `CultureInfo.InvariantCulture`、文字列の比較は `StringComparison.Ordinal` を使う([02. アーキテクチャ](02-architecture.md)の `InvariantGlobalization=true` の決定)。

## 1. 記法と用語

| 語 | 意味 |
|----|------|
| 文字 | Unicode のコードポイント(スカラー値。サロゲートを除く)。「N 文字」と書いたものは、この単位で数える(UTF-8 のバイト数でも、書記素でも、`string.Length`(UTF-16 のコード単位)でもない)。C# では `Rune`(`System.Text.Rune`)がコードポイント 1 つを表す。文字列を `Rune` で走査するには `string.EnumerateRunes()`、コードポイントの数は、`EnumerateRunes()` の要素数で数える(下の注)。 |
| コード単位 | `char`(UTF-16 のコード単位。サロゲートの前半・後半は別の `char`)。位置の列(第 13.1 節)と、YAML ライブラリの位置の添字は、この単位である。 |
| 空白を取り除く(trim) | 文字列の先頭と末尾から、Unicode の White_Space 属性を持つ文字(U+0009〜U+000D、U+0020、U+0085、U+00A0、U+1680、U+2000〜U+200A、U+2028、U+2029、U+202F、U+205F、U+3000)を取り除く。「空白だけ」は、取り除いた結果が空文字列になること。**判定は、この列挙(25 文字)を明示した表(`Whitespace.IsWhiteSpace(char)` などの自前の関数)で行い、`string.Trim()`・`char.IsWhiteSpace`・`string.IsNullOrWhiteSpace` を製品のコードで使わない**(決定)。参考(確認(出典: `Char.IsWhiteSpace` の公式ドキュメントの列挙と、`String.Trim()` の説明(`Char.IsWhiteSpace` が真の文字を取り除く)): 公式ドキュメントの列挙は、この文書の列挙と一致し、U+200B と U+FEFF は含まれない。ただし、Unicode のデータの版で `SpaceSeparator` が増える可能性は、ドキュメントからは否定できないので、規定の集合をデータの版に依存させない)。単体テストは、全 `char`(U+0000〜U+FFFF)に対して、自前の関数が、この列挙(25 個)だけを真にすることを確かめる(参考として `char.IsWhiteSpace` との一致も記録する。未確認: 実機での結果)。 |
| 文化依存 | `ToString()`、`Parse`、`ToUpper`、`string.Compare`、`StartsWith(string)` の既定は、文化に依存する(この文書の作成時に、個別の公式ドキュメントでは確認していない。確認: `Double.ToString()` の出力が `NumberFormatInfo` に従うことは、標準の数値書式の公式ドキュメントの例(`en-US` と `fr-FR` で小数点が変わる)による) | 不変文化(`CultureInfo.InvariantCulture`)と序数比較(`StringComparison.Ordinal`)だけを使う(冒頭) |
| `\d` | 確認(出典: 正規表現の文字クラス): Unicode の 10 進数字(Nd)に当たる | `[0-9]` か自前の走査(第 5.2、9.5 節) |
| `$` | 確認(出典: 正規表現のアンカー): 末尾、または末尾の `\n` の直前に一致する。`\z` は末尾だけ | `\A…\z` か自前の走査(第 5.2、9.1 節) |
| `char.IsDigit`、`char.IsLetter` | 確認(出典: `Char.IsDigit`): Unicode の 10 進数字に分類される文字 | `c is >= '0' and <= '9'` で判定(第 4.6 節) |
| `string.Trim()` と White_Space | 確認(出典: `Char.IsWhiteSpace`、`String.Trim`): 公式ドキュメントの列挙は一致する。ただし、Unicode のデータの版による変化は未確認 | 自前の表で判定し、全 `char` の単体テストを置く。`Trim()` は使わない(第 1 節) |
| `int.Parse`・`long.Parse` | 既定の `NumberStyles` が、符号・前後の空白を受け付け、例外を投げる(未確認: 全角数字を受け付けるかは、確認していない) | 字句を自前で確かめてから `TryParse`(`NumberStyles` を明示、`InvariantCulture`)(第 4.2、5.2 節) |
| `double` の書式化 | 確認(出典: .NET ブログ、標準の数値書式): .NET Core 3.0 以降、`ToString()`、`"G"`、`"R"` は、往復できる最短の文字列を返す。.NET Framework と、3.0 より前の .NET Core では、`"R"` が往復に失敗することがある | 指数形・桁の組み立ては自前(桁と指数を取り出して、自分で並べる。[09. 変換器](09-converter.md)第 6.1 節) |
| `double.Parse` の桁あふれ・書式 | 確認(出典: .NET ブログ): .NET Core 3.0 以降の解析は IEEE 754 に準拠する。桁あふれの結果(例外か ∞)と、`1.`・`.5` の受理は未確認 | 字句を正規化し、指数が 10 桁以上のものは打ち切る(第 4.2、5.2 節) |
| `Path.GetExtension` | 確認(出典: 公式ドキュメント): 最後の `.` が最後の区切りより後ろにあれば、その後ろを返す | 使ってよい。大文字小文字は ASCII だけで自前で畳む(第 2.1 節) |
| テキストの復号 | 確認(出典: `UTF8Encoding`): 引数なしの構築は、不正なバイトで例外を投げない | `throwOnInvalidBytes: true` を使い、BOM はバイト列で取り除く(第 3 節) |
| `Utf8JsonReader` | 確認(出典: 公式ドキュメント): BOM を取り除いてから渡す必要がある。位置はバイトの添字 | 全面自前(第 4.4 節) |
| `StackOverflowException` | 確認(出典: 公式ドキュメント): `try`/`catch` で捕まえられず、プロセスが終了する | 再帰を使わない(第 4.6、6.1、6.4 節) |
| `record` の自動のメンバー | `Equals`、`GetHashCode`、`ToString` が、メンバーを再帰して呼ぶ(`record` の仕様。この文書の作成時に、個別には確認していない) | 木のノードは `sealed class`(第 6.1 節) |
| `List<T>.Sort` | 確認(出典: 公式ドキュメント): 不安定な並べ替え | `OrderBy` + `ThenBy`(安定。確認(出典: 公式ドキュメント))(第 10、14.2 節) |
| `Dictionary` の列挙の順 | 順序は保証されない(一般的な契約。未確認) | 順序は `Entries` の並びから得る(第 6.1 節) |
| `string.Normalize`、`StartsWith(string)` | 文化依存・正規化の可能性 | 呼ばない。`string.Equals(a, b, StringComparison.Ordinal)`、`StartsWith(char)` か `StartsWith(string, StringComparison.Ordinal)` |
| `Mark.Line`・`Mark.Column`(YamlDotNet) | 確認(ソース): NEL・LS・PS・CR だけも改行として数える | `Mark.Index` から LF の表で求める(第 13.1 節) |
| `Mark.Index` の単位 | 確認(ソース): `char` 1 個ごとに +1(UTF-16 のコード単位) | 添字として `string` にそのまま使える。コードポイントでは数えない(第 7.2 節) |
| YamlDotNet の `Scanner` の既定の単純キーの上限 | 確認(ソース): 既定 1,024。コンストラクターで変えられる | 明示して 1,024 で作る(第 5.1 節) |
| YamlDotNet が制御文字を拒否しない | 確認(ソース): `IsPrintable` は定義だけで、`Scanner` が呼ばない | 事前に自前で検査する(第 7.4 #13) |

## 18. 未確認事項

- 第 7 節: YamlDotNet の挙動は、**公開ソースを読んだ範囲**であり、動かしていない。実装時に、第 7.5 節の試験(特に #1〜#4、#7〜#9、#11)を、固定した版で、3 つの OS で行う。個別の未確認: (a) `!` 単独のタグが、`TagName` の構築で例外になるか、(b) アンカー・タグを前置したノードの `Start` が指す位置、(c) 値が省略された対のイベントの位置、(d) フロー形式の中の単純キーの 1,024 の制限、(e) 深いフロー形式の入れ子での速度、`Parser` が再帰を使うか、(f) 空の文書・`---` だけの文書のイベントの並び、(g) 複数の文書のときの 2 つ目の文書の根の位置、(h) `%TAG`・未知のディレクティブの受理。
- 第 3 節: `File.GetAttributes`、`FileInfo.Length` のシンボリックリンクでの結果、FIFO で `FileStream` がブロックするか、Windows など他の OS での `IOException` の `HResult`・権限の判定の細部、UTF-8 の復号の失敗になる入力(過長な符号化、サロゲートの符号化、範囲外、単独の `FF`)。パスの ENOTDIR が `DirectoryNotFoundException`(→ `fileMissing`)になる可能性(順 3 の `読み取りに失敗しました` との差)。
- 第 4 節: `Utf8JsonReader` の、不正な入力での詳細な挙動(`MaxDepth` の最大値、不正なサロゲートのエスケープ、重複キー、位置の規則)。差分試験(第 4.4 節)で確かめる。`long.TryParse` が `-9223372036854775808` を受理すること。`double.Parse` が、桁あふれで例外を投げないこと、指数が 9 桁の巨大な値を処理できること。
- 第 1・12.2 節: .NET が、UTF-8 として不正なバイトを含む引数・環境変数・ファイル名を、どう `string` にするか(U+FFFD か、単独のサロゲートか)。[08](08-output-and-i18n.md)で決める。`string.EnumerateRunes()` が、単独のサロゲートに対して返す値。`List<T>.BinarySearch` の補数の契約の、公式ドキュメントでの確認。
- 第 1 節: `char.IsWhiteSpace` が、規定の 25 文字と一致するか(参考。製品のコードは使わない。全 `char` の単体テストで記録する)。
- 第 6.3 節: 複数行にまたがる引用符なしのスカラーの、文字の大きさの数え方の境界(16 MiB 付近のみ)。
- 第 13.4 節: YAML の文法の誤りの行の厳密な規則(ライブラリの検出位置による。表は期待する値で、未確認)。
- 第 7.2 節: `VYaml`、`SharpYaml` の低水準の API(位置、タグ、スタイル)とライセンス。トリミング・NativeAOT での YamlDotNet の動作(`csproj` に、トリミング・AOT 対応の指定が見当たらない。使わない方針のため、未確認のまま)。
- 第 4.2 節: `1e999999999`(指数 9 桁)を `double.Parse` が処理する時間と結果。
