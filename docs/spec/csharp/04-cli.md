# 04. コマンドライン

この章は、C#(.NET)で実装する実行ファイル `conductor`(版 0.8.0)のコマンドラインを規定する。引数の解釈は、`System.CommandLine` などの引数解析ライブラリを使わず、字句の段階(4.1)と検査の段階(4.2)を**自前の手順**で実装する。メッセージの全文は [08-output-and-i18n.md](08-output-and-i18n.md)、終了コードとエラー型は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md)、各コマンドの内部の流れは [05-execution-flow.md](05-execution-flow.md)、定義ファイルの書式は [03-definition-format.md](03-definition-format.md)、変換は [09-converter.md](09-converter.md)、シグナルと子プロセスは [07-process-and-signals.md](07-process-and-signals.md) を参照する。

この文書で「確認(出典: …)」と書いた事項は、公式ドキュメントまたは公開されているランタイムのソースを取得して確かめた内容である。「未確認」と書いた事項は、この環境に .NET SDK がなく実機で動かせないこと、または出典を取得できなかったことで、確かめられていない。実装時に、対象の OS と .NET の版で確かめる。

## 1. 全体像

- 入口は `src/Conductor.Cli/Program.cs` の `static int Main(string[] args)` の 1 つ(同期。`async` にしない。02 章の並行モデルに従う)。`Main` は終了コード(`int`)を戻り値として返す。確認(出典: 「Main() and command-line arguments」): `Main` は `void`・`int`・`Task`・`Task<int>` を返せ、`int` を返すと、その値が呼び出し元(シェルなど)に渡る。`string[] args` は null にならず、引数がなければ `Length` は 0。
- 引数はシェルを介さず、`conductor` 自身は展開(ワイルドカード、変数、チルダなど)も解釈もしない。
- サブコマンドは 6 つだけ: `run`, `pipeline`, `watch`, `reset`, `validate`, `convert`。
- 特別なオプション(`-h`/`--help`, `-V`/`--version`)は、コマンドの解釈より先に処理する(4.2 の 1)。
- 使い方の誤りのときは、標準エラーに「誤りの 1 行 + 空行 + 使い方全文」を書き、終了コード 68 で終わる。ジョブは実行しない。
- 既定では、`conductor` 自身は何も出力しない(ジョブ本体の出力がそのまま流れる)。エラーのとき、または `-v` のときだけ標準エラーに書く。例外として、`watch`・`--help`・`--version`・`convert` は結果を標準出力に書く。
- プロセスの終了コードを決める箇所は、`Main` の 1 か所だけにする。`Conductor.Cli` の `CliApp.Run(string[] args, CliEnvironment env)` は終了コード(`int`)を戻り値として返し、`Main` がその値を、出力を完了させたうえで `return` する(10 章 4.1、08 章 1.4)。`Environment.Exit` は通常の経路では使わない(10 章 4.1、6)。

### 1.1 `System.CommandLine` などを使わない理由

`System.CommandLine`(Microsoft の引数解析ライブラリ)は、この仕様の文法に合わないため使わず、自前で実装する。次の表は、公式ドキュメントの記述(確認(出典: 「Command-line syntax overview for System.CommandLine」、「System.CommandLine overview」。いずれも learn.microsoft.com))と、この仕様の文法の突き合わせである。ライブラリを実機で動かして確かめたわけではない(この環境に .NET SDK がない)ので、「ドキュメントの記述」の列は文書の記述であり、実際の挙動は未確認。

| 仕様が要求すること | `System.CommandLine` のドキュメントの記述 | 合わない点 |
|--------------------|---------------------------------------------|------------|
| 長いオプションの前方一致による省略は受け付けない(`--verb` は知らないオプション) | 「`System.CommandLine` doesn't support automatic aliases」(前方一致の自動の別名はない)。大文字小文字は区別する | 合う |
| 値の区切りは、空白と `=` だけ(`--config=x`)。`:` は区切りではない | 空白、`=`、`:` のどれも区切りとして使える(`-v:quiet`)。単一文字の別名では、区切りを省いて値をつなげられる(`-vquiet`) | `:` と、区切りなしの連結が、仕様にない解釈になる。仕様では `-vquiet` は、`-v` と知らない文字 `q`・`u`…の連結になる。 |
| 値を取らないオプション(`-v`、`--verbose`)の直後の引数は、位置引数 | 型が `bool` のオプションの引数の個数は `ZeroOrOne`(`--interactive false` のように、`true`/`false` を後ろに書ける) | `-v false` や `--all true` の `false`・`true` を、位置引数(ジョブIDなど)ではなくオプションの値と解釈しうる(仕様では位置引数) |
| 値を取る長いオプションは、次の引数を、中身にかかわらず値にする(`--config --help` は `--help` が値) | 値を取るオプションの次の引数が `-` で始まるときの扱いは、取得したページに記述がない | 未確認。仕様の文法を保証できない |
| `@` で始まる引数は、ただの位置引数 | 「To use a response file, enter the file name prefixed by an `@` sign wherever in the line」(応答ファイル `@ファイル名` を、既定で使える機能として説明している) | `@name` というジョブIDが、ファイルの読み込みとして扱われうる。無効にできるかは取得したページに記述がない(未確認) |
| `[` と `]` で囲んだ語は、ただの位置引数 | 「`[diagram]`」「`[suggest]`」のようなディレクティブを、アプリ名のあとに置ける。「An unrecognized directive is ignored without causing a parsing error」 | `[x]` という語が、無視される・別の動作になる |
| 同じオプションの重複(`-vv`、`-v --verbose`)は `invalid value` | 「the last instance of a repeated option overwrites any earlier instances」(最後の指定が前を上書きする設定が可能) | 重複を誤りとして報告する制御は、取得したページに記述がない(未確認) |
| `--` のあとはすべて位置引数 | 「`System.CommandLine` supports this double-dash functionality」 | 合う。ただし、2 個目以降の `--` の扱いは記述がない(未確認) |
| 誤りの優先順位(`-h`/`-V` が最優先、字句の誤り、知らないオプション…)と、終了コード 68、固定の文言 | 誤りは `ParseResult.Errors` に集められ、文言は `ParseError.Message`。ヘルプ・版は `RootCommand` が既定で持つ | 優先順位を、この仕様の順に制御する方法は、取得したページに記述がない(未確認)。既定のヘルプの文言・形が仕様と違う |

なお、`System.CommandLine` は、公式ドキュメントで「trim-friendly」(トリミングに適した)と説明されている(確認(出典: 「System.CommandLine overview」))。パッケージの正式版かどうかは、取得したページには記述がなく、未確認(NuGet のページは取得していない)。

要するに、(1) `:` での値の指定、区切りなしの短縮形の連結、応答ファイル、ディレクティブ、`bool` オプションの省略可能な値が、仕様にない文法として既定で働きうる。(2) 値を取るオプションが次の引数を無条件に取る規則、重複の検出、誤りの優先順位を、この仕様の形に制御できることを、公式ドキュメントから確認できない。(3) 誤りの文言と終了コードが固定されている。したがって、`Conductor.Cli` に字句の段階の関数と検査の段階の関数を置き、`string[] args` だけを入力に、文字列の操作だけで実装する。実行時の依存を YAML の読み取りライブラリ 1 つだけにする方針(01 章・02 章)にも合う。

## 2. 構文

```text
conductor run [-v] [--config <path>] [--run-id <id>] [--remove-on-complete] <job-id>
conductor pipeline [-v] [--config <path>] [--run-id <id>] [--remove-on-complete] <pipeline-name>
conductor watch [--config <path>] [--run-id <id>]
conductor reset [--config <path>] [--run-id <id> | --all]
conductor validate [-v] [--config <path>] [<pipeline-name>]
conductor convert <file> --to <json|yaml>
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
| `--to` | なし | `json` か `yaml`(必須) | 不可 | 不可 | 不可 | 不可 | 不可 | 必須 |
| `--help` | `-h` | なし | どのコマンドでも(コマンド名がなくても)受け付ける |
| `--version` | `-V` | なし | どのコマンドでも(コマンド名がなくても)受け付ける |

- 長いオプション名・短縮形は、大文字小文字を区別する。長いオプションの前方一致による省略(`--verb` など)は受け付けない(知らないオプション)。
- 上表以外の短縮形は存在しない(`--config` に `-c` はない)。
- Windows の `/h` や `/?` の形は、受け付けない(位置引数として扱う。`/` で始まる引数に特別な意味はない)。

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

オプションは、コマンド名の前にも後ろにも置ける(例: `conductor --config c.json -v run x` は有効)。

### 3.3 各オプションの意味と既定値

| オプション | 意味 | 既定 |
|------------|------|------|
| `-v`, `--verbose` | ジョブの結果・スキップ・遷移・中断を標準エラーに出す(`validate` は成功時に 1 行)。段階は 1 つだけ(詳しさの度合いを選べない) | 出さない |
| `--config <path>` | 定義ファイルのパス。`string` のまま加工せず、ファイルを開く API(`File.OpenRead`、`new FileStream(...)`、`File.Exists` など)にそのまま渡す(`Path.GetFullPath` や `Path.Combine` で正規化・絶対化した文字列を、利用者に見せる文字列(誤りの行の場所、ステートのパスの元)にしない。相対パスは実行ディレクトリ(カレントディレクトリ)から解決される)。書式は拡張子で決まる(`.json` は JSON、`.yaml`・`.yml` は YAML。大文字小文字を区別しない。それ以外は拒否。[03-definition-format.md](03-definition-format.md)) | 実行ディレクトリ(カレントディレクトリ)の `conductor.json` |
| `--run-id <id>` | ステートファイルを分ける run ID。空文字は「run ID なしに固定」(環境変数を見ない) | 未指定(環境変数を見る。5 章) |
| `--remove-on-complete` | 完了時にステートファイルを削除する | 指定なし(その場合は定義の `pipeline.remove_on_complete`、それも無ければ `false`) |
| `--all` | 定義のステートファイルを、run ID の有無によらずすべて消す(`reset` のみ) | なし |
| `--to <json\|yaml>` | `convert` の出力形式。大文字小文字を区別し、`json` と `yaml` 以外は `invalid value`(対象は値そのもの) | なし(必須) |

既定の定義ファイル名 `conductor.json` は、`.yaml`・`.yml` を探索しない。既定の名前でファイルがなければ、通常の「ファイルなし」(64)になる。YAML の定義を使うときは、`--config conductor.yaml` のように明示する。人が書く定義にコメントが要るときは、YAML を使う(JSON はコメントを書けない)、と利用者向けに案内する(使い方の文には入れない)。

`--config` のパスについての注意: `Path.GetFullPath`・`Path.Combine`・`new FileInfo(path).FullName` などは、文字列を変える(区切りの正規化、`.`・`..` の解決、相対パスの絶対化など。挙動の詳細は公式ドキュメントを取得していないため未確認)。利用者に見せる文字列と、ステートのパスの元には、`--config` に渡された**元の `string`** を使い、`Path` の API は、内部でパスをつなぐ(ステートのパスの作り方は [06-state-file.md](06-state-file.md))道具にとどめる。`Path.GetExtension` の戻り値(`.yaml` という名前のファイル、`c.yaml/` のような末尾の区切りなど)と、定義の書式の判定の関係は、03 章が規定する。空文字の `--config ''` は、ファイルを開けず、定義ファイルなし(64)になる(例外の型と文言は 03 章の読み込みの規定に従う。実機では未確認)。

## 4. 引数の解釈

解釈は、(A)字句の段階、(B)コマンドの組み立てと検査、の 2 段で行う。(A)は引数を先頭から 1 つずつ見て、`special`(最初に現れた `-h`/`-V` の種別)、`lexError`(最初の字句の誤り)、`tokens`(オプションと位置引数の並び)を作る。(B)は `tokens` を検査する。

### 4.0 内部の表現(骨格)

型の名前と分け方は目安であり、4.1・4.2 の挙動を満たせば変えてよい。

```csharp
namespace Conductor.Cli;

internal enum SpecialKind { None, Help, Version }   // 最初に現れた方

internal enum TokenKind
{
    Positional, Verbose, Config, RunId, To,
    All, RemoveOnComplete, Unknown,
}

// Value: 位置引数・値を取るオプションの値。Unknown では綴り(-x, --bogus)
internal readonly record struct Token(TokenKind Kind, string Value);

internal sealed record LexResult(
    SpecialKind Special,
    UsageError? LexError,            // 最初の字句の誤り(10 章 8.3)
    IReadOnlyList<Token> Tokens);

internal static class ArgumentLexer
{
    public static LexResult Lex(string[] args);
}
```

引数は .NET の `string`(UTF-16 のコードユニットの列)のまま扱う。`s[i]` と `s.Length` はコードユニット単位であり、基本多言語面の外の文字(絵文字など。U+10000 以上)は、2 つのコードユニット(高位サロゲート + 低位サロゲート)になる。4.1 の連結の分解では、次の**自前の走査**で「1 文字(1 コードポイント)」を切り出す(`foreach (char c in s)` で 1 つずつ見てはならない)。

1. 位置 `i` のコードユニットが高位サロゲート(`char.IsHighSurrogate`)で、次の位置があり、そのコードユニットが低位サロゲート(`char.IsLowSurrogate`)なら、その 2 つで 1 文字(コードポイントは `char.ConvertToUtf32(high, low)`)。
2. それ以外は、位置 `i` の 1 コードユニットを 1 文字として扱う(対になっていないサロゲートも 1 文字。4.4)。

「見た目の 1 文字」(書記素クラスタ。結合文字、国旗など)とは一致しない。連結の分解は、コードポイント 1 つずつで行う(`-é`(分解形で `e` + U+0301)は、`-e` と、知らない文字 U+0301 の 2 文字として分解される。`System.Globalization.StringInfo` や `TextElementEnumerator` で書記素単位に切らない)。`System.Text.Rune` を使う走査(例: `string.EnumerateRunes()`)は、対になっていないサロゲートの扱いが、上の規定と同じになるかを公式ドキュメントで確認していないため(未確認)、この仕様の規定は上の自前の走査とする(`Rune` を使うなら、孤立サロゲートを `Rune.ReplacementChar` に置き換える挙動が、4.4 の規定と同じ結果になることを実装時に確かめる)。

文字列の比較と検索は、すべて序数(`StringComparison.Ordinal`)で行う。`==`(`string` の等価比較は序数)、`string.StartsWith("--", StringComparison.Ordinal)`、`string.IndexOf('=')`(`char` を引数にする形)を使う。`string` を引数にする `StartsWith(string)`・`IndexOf(string)`・`string.Compare`・`Contains` の文化依存の形(`StringComparison` を省略した形。既定が文化依存である挙動は、この環境では公式ドキュメントを取得しておらず未確認)は使わない(`InvariantGlobalization=true` でも、`StringComparison` を明示する。01 章・02 章の方針)。`string.Trim()`・`string.Split()`(引数なし)・`char.IsWhiteSpace`・`char.IsLetterOrDigit` は、Unicode の空白・文字・数字に広く一致するため、字句の規則に使わない。

### 4.1 字句の段階(A)の手順

`afterDashDash = false` として、引数 `a` を先頭から順に処理する。

1. `afterDashDash` が真なら、`a` は位置引数(`-` で始まっていても、`-v` や `--help` であっても位置引数)。
2. `a` が `--` ちょうどなら、`afterDashDash = true` にする(`--` 自体は位置引数にならない)。2 個目以降の `--` は、`afterDashDash` が真なので位置引数になる。
3. `a` が `--` で始まり(`--` より長い)、長いオプションとして扱う。`=` が含まれていれば、最初の `=` の前を名前(`--` を除く)、後ろを値(空でもよい)とする(`a.IndexOf('=', 2)` で位置を得て、`a.Substring(2, p - 2)` と `a.Substring(p + 1)`)。含まれていなければ名前は `a` から `--` を除いた全体で、値なし。
   - 名前が `help`: `special` が未設定なら help を設定する。`=値` が付いていても、help が優先されるので誤りにならない(例: `--help=x` は help)。
   - 名前が `version`: 同様に `special` が未設定なら version を設定する(`--version=x` も version)。
   - 名前が `verbose`, `all`, `remove-on-complete`(値を取らない): `=値` が付いていれば(値が空でも)`lexError`(最初の 1 つだけ保持)に `invalid value`(対象はそのオプションの正規の綴り、例 `--verbose`)を設定する。付いていなければ、そのオプションを `tokens` に追加する。
   - 名前が `config`, `run-id`, `to`(値を取る): `=値` があれば、それ(空文字も有効)を値として `tokens` に追加する。なければ、次の引数を、中身にかかわらず(`-` で始まっていても、`--` や `--help` であっても)値として取る(その引数は位置引数にもオプションにもならない)。次の引数が存在しないときは、`lexError` に `option requires a value`(対象は `--config` / `--run-id` / `--to`)を設定し、字句の段階を打ち切る(以降の引数は見ない)。
   - それ以外の名前(空の名前を含む): 「知らないオプション `--<名前>`」を `tokens` に追加する。`=値` が付いていたときは、`lexError` にも `unknown option`(対象 `--<名前>`、値は含めない)を設定する。名前が空のときの綴りは `--`(例: `--=x` は `unknown option: '--'`)。`---` は名前が `-` で、綴りは `---`。
4. `a` が `-` で始まり、長さが 2 以上(コードポイント数。上の走査で数える)で、`--` で始まらない: 短縮形の連結として、先頭の `-` を除く文字を、先頭から 1 文字(1 コードポイント)ずつ処理する。
   - `h`: help を設定(上と同じ。`special` が未設定のときだけ)。`V`: version を設定(同様)。`v`: `-v` を `tokens` に追加する。
   - ほかの文字 `c`: 「知らないオプション `-<c>`」を `tokens` に追加する(`-é` は `unknown option: '-é'`。孤立サロゲート(4.4)の文字も 1 文字として `-<c>` の綴りにする)。
   - 2 文字目以降の位置で、直前の 1 文字の直後に `=` が続いているとき、その `=` 以降の残りの文字列は**まとめて捨て**(残りの文字は解釈しない)、`lexError` を設定する。直前の文字が `v` なら `invalid value`(対象 `-v`)、知らない文字 `c` なら `unknown option`(対象 `-<c>`)。直前の文字が `h`・`V` のときは `special` が優先されるので誤りにならない。連結の先頭の文字が `=` のとき(`-=x`)は、区切りではなく、`=` という文字そのものが知らないオプション(`unknown option: '-='`)。
   - 短縮形は値を取らない(値を取るのは長いオプションだけ)。`-vv` は `-v` が 2 回。`-vh` は `-v` と help(help が勝つ)。
5. それ以外(`-` ちょうど 1 文字、`-` で始まらない文字列、空文字)は位置引数。単独の `-` も位置引数である。

`lexError` は最初の 1 つだけを保持する(後の字句の誤りは捨てる)。

挙動の例(この文法の規則から導かれるもの):

- `--help=x`・`-V=1`・`--version=x` は、それぞれ help / version / version。`-vx run a` は `unknown option: '-x'`、`run -vh` は help、`-vV` は version、`--config --help run a` は `--help` という名前のパスを定義ファイルとして扱う(help ではない。拡張子が `.json`・`.yaml`・`.yml` のどれでもないので、定義の誤り `unsupportedFormat` として 64)。
- `--config --help`(ほかの引数なし)は `--help` を値に取り、位置引数が 0 個なので `missing argument`(68)。`run a --config --version` も `--version` を値(パス)として扱う。
- `--help --config` は、`--help` が先に解釈されるので、値の欠落(`--config`)より前に help(0)になる。
- `run a -- b` の `--` は、除かれて位置引数 `b` だけが残る(位置引数は `a` と `b` の 2 個なので `unexpected argument: 'b'`)。`run -- a --` は 2 個目の `--` が位置引数になり `unexpected argument: '--'`。`run -- -v` は、ジョブID `-v`。`run -` は、ジョブID `-`。
- `-v=1 run a` と `run -v=1h x` は、どちらも `invalid value: '-v'`(`=` 以降の `h` は解釈されず、help にならない)。`-v=` も `invalid value: '-v'`。`-vh=1` は help。`-x=1h` と `-vx=1` は `unknown option: '-x'`。
- `--=x`・`--=` は `unknown option: '--'`、`---` は `unknown option: '---'`、`--verbose=`(値が空)は `invalid value: '--verbose'`。
- `reset --all=1` は `invalid value: '--all'`。`run j --run-id`・`--to`(値なし)は `option requires a value`。

### 4.2 検査の順序(誤りの優先順位)

字句の段階のあと、次の順に検査する。最初に当てはまったもので終わる(それ以降は行わない)。

1. `special` が設定されていれば(`--` より前に、オプションとして解釈された `-h/--help` か `-V/--version` が 1 つでもあれば)、ほかの誤りに関係なく、先に現れた方を処理して終了コード 0。値を取るオプション(`--config`、`--run-id`、`--to`)の直後の引数は、`-h` や `--help` であっても値として取られ(4.1 の 3)、オプションとして解釈されないので、`special` にならない(`--config --help run a` は help ではない。`-h --config --version` は、`-h` が先に現れるので help で、`--version` は `--config` の値になる)。両方あるとき、先に現れた方が勝つ(`-V -h` は version、`-h -V` は help)。字句の段階が値の欠落で打ち切られた場合、それ以前に現れた `-h`/`-V` は有効、以降は存在しない扱い。
2. `lexError` があれば、それを誤りとする。
3. 「知らないオプション」が `tokens` にあれば、最初のものを `unknown option`(対象はその綴り: `-x`, `--bogus`)とする。
4. 位置引数が 0 個: `missing argument`(対象なし)。例: 引数なし、`-v` だけ、`--` だけ。
5. 先頭の位置引数(コマンド名)が「文字として正しくない引数」(4.4 の判定)なら `invalid value`(対象は、4.4 の規則で読める形にした文字列)。6 コマンドのいずれでもなければ `unknown command`(対象はその語。空文字も同じ)。
6. そのコマンドで使えないオプション: `option not allowed for this command`。検査の順は `-v`, `--config`, `--to`, `--run-id`, `--all`, `--remove-on-complete`。対象はそのオプション名(`-v` の綴りは `-v`)。可否は 3.1 の表による。
7. 重複: `-v`, `--config`, `--to`, `--run-id`, `--all`, `--remove-on-complete` のどれかが 2 回以上 → `invalid value`(対象はそのオプション名。`-vv` は対象 `-v`)。検査の順は挙げた順。`-v` の重複は、`-v` と `--verbose` の合計で数える。
8. `--all` と `--run-id` の併用 → `invalid value`(対象 `--all`)。
9. `--run-id` の値の検査(5.2)。値が文字として正しくない(4.4)、または空でなく規則に違反 → `invalid value`(対象 `--run-id`。値は含めない)。空文字は有効。
10. 位置引数の過不足(コマンド名を除いて数える):
    - `run`・`pipeline`・`convert` で位置引数が 0 個 → `missing argument`(対象なし)。
    - `watch`・`reset` で位置引数が 1 個以上 → `unexpected argument`(対象は最初の位置引数)。
    - コマンドが取れる個数を超える(2 個目の位置引数がある)→ `unexpected argument`(対象は 2 個目の語。`run a b` なら `b`)。
    - 位置引数(ジョブID、パイプライン名)が文字として正しくない(4.4)→ `invalid value`(対象は読める形にした文字列)。`convert` の入力ファイルは、4.4 の判定にかかっても検査せず、パスとしてそのまま通す。
11. `convert`: `--to` なし → `missing argument`(対象 `--to`)。`--to` の値が `json`/`yaml` 以外(文字として正しくない値を含む)→ `invalid value`(対象は、読める形にした値)。

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
| `run a --to json` | `option not allowed for this command: '--to'`(68) |
| `convert --to json --to yaml f` | `invalid value: '--to'`(68) |
| `convert f --to xml` | `invalid value: 'xml'`(68)(`--to` の値は `json`・`yaml` だけ。大文字小文字も区別する) |
| `convert f --to JSON` | `invalid value: 'JSON'`(68) |
| `convert f` | `missing argument: '--to'`(68) |
| `reset --all --run-id x` | `invalid value: '--all'`(68) |
| `run -vv a` | `invalid value: '-v'`(68) |
| `run -v:x a`、`run -vq a` | `unknown option: '-:'`・`unknown option: '-q'`(68)(`:` も区切りではなく、連結の中の知らない文字) |
| `run @a`、`run [x]`、`run /h` | それぞれジョブID `@a`・`[x]`・`/h`(位置引数。ファイルの読み込みなどはしない) |

### 4.4 `string[] args` の文字列と、文字として正しくない引数

#### 4.4.1 .NET が引数を `string` にする方法(確認した範囲)

確認(出典: 「Main() and command-line arguments」): `Main` の `string[] args` は、プログラム名を含まない(「Unlike C and C++, the name of the program isn't treated as the first command-line argument in the `args` array」)。確認(出典: 「Environment.GetCommandLineArgs」): `Environment.GetCommandLineArgs()` の最初の要素は、実行中のプログラムのファイル名(取得できないときは空文字)で、残りがそのほかの引数である。.NET 5 以降の単一ファイル発行では、最初の要素はホストの実行ファイルの名前。プログラムのファイル名は、パスを含みうるが、含むとは限らない。

この仕様では、**`string[] args` だけを使い、`Environment.GetCommandLineArgs()` と `Environment.CommandLine` を使わない**(先頭にプログラム名が入る差、単一ファイル発行での値の違いを避けるため)。`args` の要素と、`GetCommandLineArgs()` の 2 番目以降の要素が、すべての OS・起動方法で一致するかは、取得した文書に記述がなく、未確認。

**Windows のコマンドラインの区切り**: Windows は、コマンドラインを 1 本の文字列としてプロセスに渡す(一般的な Windows の仕組み。出典のページは取得しておらず、未確認)。.NET が引数に分ける規則は、次のとおり文書化されている。確認(出典: 「Environment.GetCommandLineArgs」の Remarks)。

- 引数は空白で区切る。二重引用符(`"`)で囲むと、空白を引数に含められる。一重引用符(`'`)は、この機能を持たない。
- 二重引用符が、偶数個のバックスラッシュのすぐ後ろにあるとき、バックスラッシュの対 1 つにつき、バックスラッシュ 1 つになり、二重引用符は取り除かれる。二重引用符が、奇数個(1 個を含む)のバックスラッシュのすぐ後ろにあるとき、対 1 つにつきバックスラッシュ 1 つになり、残りの 1 つのバックスラッシュは取り除かれるが、二重引用符は取り除かれない。

文書に載っている対応の例(アプリ名 `MyApp`。出力の先頭はプログラム名なので、`args` では 2 番目以降):

| コマンドラインの入力 | 引数(プログラム名を除く) |
|------------------------|------------------------------|
| `MyApp alpha beta` | `alpha`, `beta` |
| `MyApp "alpha with spaces" "beta with spaces"` | `alpha with spaces`, `beta with spaces` |
| `MyApp 'alpha with spaces' beta` | `'alpha`, `with`, `spaces'`, `beta` |
| `MyApp \\\alpha \\\\"beta` | `\\\alpha`, `\\beta` |
| `MyApp \\\\\"alpha \"beta` | `\\"alpha`, `"beta` |

これらは、Windows で `args` に入る値の例として、受け入れテストの入力に使ってよい(Windows の `args` が、`GetCommandLineArgs()` の文書と同じ規則であることは未確認)。Unix 系では、シェルが引数に分けたものが、そのまま `args` になる(ランタイムは、引用符やバックスラッシュを再解釈しない。取得した文書にこの記述はなく、未確認)。したがって、同じ見た目のコマンド行でも、OS によって `args` が変わりうる(例: Windows のコマンドプロンプトでの一重引用符、バックスラッシュと引用符の組)。`conductor` は、`args` に入ってきた文字列を、OS によらず同じ規則で解釈する(Windows 固有の再解析をしない)。

Windows の次の点は、取得した文書に記述がなく、未確認: 空文字の引数 `""` が `args` に空文字の要素として入るか(`--run-id ""` の形で必要になる。`--run-id=` の形は、この問題を避けられる)、ワイルドカード(`*`)が展開されないか、`conductor.exe` を `cmd.exe` と PowerShell から起動したときの引用符の扱いの差。

#### 4.4.2 Unix での不正な UTF-8(未確認の範囲と、この版の決定)

Unix の引数は、OS からはバイト列として渡される。そのバイト列が不正な UTF-8 を含むとき、`string[] args` にどう入るか(置換文字 U+FFFD になるのか、例外か、別の扱いか)は、公式ドキュメントでも、取得したランタイム(ホスト)のソースでも確認できなかった(ホストが引数を変換する箇所は確認したが、変換の定義と不正な列の扱いは取得できなかった)。**未確認**。ドキュメントの記述として確認できたのは、一般に `UTF8Encoding` が不正なバイト列を復号するとき、U+FFFD に置き換える設計であることだけである(確認(出典: 「UTF8Encoding Constructor」の例: 不正な符号化に対する既定の挙動は置換で、`throwOnInvalidBytes: true` のときだけ例外)。ただしこれは `UTF8Encoding` の挙動で、引数の変換が同じ経路を通る確証はない)。環境変数 `Environment.GetEnvironmentVariable` の値についても同じ(未確認)。

したがって、この版は、実装が不正な UTF-8 を区別できない前提で、次の規則を決める。

- **「文字として正しくない文字列」の判定** `IsSuspectText(string s) -> bool`: `s` が、(a) 対になっていないサロゲート(高位サロゲートの次が低位サロゲートでない、低位サロゲートの前が高位サロゲートでない、末尾の高位サロゲート)を含む、または (b) 置換文字 U+FFFD を含む、のどちらかなら真。走査は、4.0 の自前の走査(`char` を 1 つずつ見て、サロゲートの対を判定する)で行い、`string.IsNormalized`・`Encoding.GetBytes` の例外に頼らない。
  - (a) は、Windows の UTF-16 の引数(対になっていないサロゲートを含みうる)と、実装が不正な列をそのまま保持する場合のため。(b) は、実装が不正な UTF-8 を U+FFFD に置き換えて渡す場合のため。
  - 既知の差: 利用者が、U+FFFD を本当に含む(正しい UTF-8 の)文字列を、ジョブIDなどに指定した場合も、真になり、`invalid value` として拒否される。この差を受け入れる(ジョブIDは、定義の検査で許す文字が限られており([03-definition-format.md](03-definition-format.md))、U+FFFD を含まないため、定義済みのジョブIDを指定できなくなることはない)。
- 次のとおり扱う(4.2 の各段階で適用する)。

| 対象 | 扱い |
|------|------|
| コマンド名(最初の位置引数) | `IsSuspectText` が真なら `invalid value`(4.2 の 5。対象は読める形) |
| ジョブID・パイプライン名(`validate` を含む) | 真なら `invalid value`(4.2 の 10。対象は読める形) |
| `--run-id` の値 | 真なら `invalid value`(4.2 の 9。対象は `--run-id`。値は含めない) |
| `--to` の値 | `json`/`yaml` 以外なので `invalid value`(4.2 の 11。対象は読める形) |
| `--config` の値、`convert` の入力ファイル | 真でも検査せず、`string` のままパスとして通す。ただし、不正な UTF-8 のバイトを持つファイル名は、置換が行われた `string` からは、元のバイト列を取り戻せず、そのファイルを開けない(Unix のファイル名はバイト列であり、ここで元のバイト列を取り戻す .NET の公式の手段は確認できなかった。**制約として受け入れる**。実機での確認は未確認) |
| 知らないオプションの綴り(`-<c>`、`--<名前>`) | 真でもそのまま持つ。誤りの対象に出すとき、読める形にする |
| 環境変数の値(run ID、ロケール、`NO_COLOR`) | 真なら、未設定と同じ扱い(5.1、08 章 7 章) |

**読める形にする規則** `Readable(string s) -> string`(使い方の誤りの対象に、文字として正しくない引数を出すとき、および 08 章 9 章のエスケープの前処理で使う): 対になっていないサロゲートを、1 つにつき U+FFFD 1 個に置き換える(対になっているサロゲートは、そのまま。すでに U+FFFD の文字は、そのまま)。実装が不正な UTF-8 を U+FFFD に置き換えて渡す場合、不正なバイト列 1 つ(最大の部分列。Unicode の規定)あたり U+FFFD が何個になるかは、実装(ランタイム)に任せる(`Readable` はそれに手を加えない)。例: 実装が `a` + `FF` + `b` を `a`・U+FFFD・`b` として渡すなら、対象は `a` + U+FFFD + `b`(これは、仕様どおりの動作)。

## 5. run ID

### 5.1 決定の優先順位(`run`・`pipeline`)

1. `--run-id <値>` が指定されていれば、環境変数は見ない。値が空文字なら run ID なし。
2. 指定がなければ、環境変数を読む(`CliEnvironment` の `GetEnvironmentVariable(string name) -> string?`。実体は `Environment.GetEnvironmentVariable(name)`)。環境変数の名前は、定義の `pipeline.run_id_env`(なければ `CONDUCTOR_RUN_ID`)。未定義(`null`)または空文字なら run ID なし。値が `IsSuspectText`(4.4.2)なら、未定義と同じ扱い。
3. 得た値は 5.2 の規則で検査する。違反は `invalid value`(終了コード 68)。対象は、オプション由来なら `--run-id`、環境変数由来なら `run ID (environment variable)`(値は含めない)。

`run`・`pipeline` では、環境変数の名前を知るために、定義を先に読む。検査の順序は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md) 2 章を参照する。

環境変数の読み取りで注意する点: `Environment.GetEnvironmentVariable(name)` は、定義がないとき `null` を返す(.NET の API の説明。この環境では出典のページを取得しておらず、未確認)。`null` のとき「未定義」、`""` のとき「空文字」(どちらも run ID なし)として、同じに扱う。名前の大文字小文字の区別は OS に従う(Unix は区別し、Windows は区別しないとされるが、取得した文書の確認はなく未確認)。環境変数の読み取りは、すべて `CliEnvironment` の口を通す。`Environment.GetEnvironmentVariable` を直接呼ぶのは、その本番の実装だけにする。

### 5.2 run ID の規則

- 文字は `A-Za-z0-9._-` だけ、長さは 1〜64 文字、先頭が `.` でない(`..` も不可)。空文字は「run ID なし」を表し、値としては規則を満たさない(オプションの空文字だけが「なし」を意味する特別な扱い)。
- 検査は `Conductor.RunIds` の純粋な関数 `RunIdRules.IsValid(string value)` に置く。**正規表現を使わず、1 文字ずつの比較で書く**: 各文字が `'A' <= c && c <= 'Z'`、`'a' <= c && c <= 'z'`、`'0' <= c && c <= '9'`、`'.'`、`'_'`、`'-'` のどれかであること(`char.IsLetterOrDigit`・`char.IsAsciiLetterOrDigit` などの名前付きの判定関数ではなく、範囲の比較を書く。`char.IsLetterOrDigit` は Unicode の文字・数字に広く一致する)。文字がすべて ASCII であることを確かめてから、`value.Length`(コードユニット数)で長さを検査する(ASCII だけなので、コードユニット数、コードポイント数、UTF-8 のバイト数が同じになる)。正規表現を使う場合の注意: .NET の `Regex` の `$` は、文字列の末尾の改行の直前にも一致し、`\d` は既定で Unicode の数字に一致するとされる(出典のページは取得しておらず、未確認)ので、使うなら `\A`・`\z` と、文字クラスの明示、`RegexOptions.CultureInvariant` が要る。実装は正規表現を使わないのが既定。
- ステートファイル名は、run ID なしで `<定義ファイル名>.state`、ありで `<定義ファイル名>.<run-id>.state`(定義ファイルと同じディレクトリ)。詳細は [06-state-file.md](06-state-file.md)。
- 定義の `run_id_env` は、`A-Za-z_` で始まり `A-Za-z0-9_` が続く 1 文字以上(同じく範囲の比較で検査する)。違反は定義の誤り(64。[03-definition-format.md](03-definition-format.md))。
- 理由の区別(空・長すぎ・不正な文字・先頭のドット)は内部では持ってよい(`RunIdErrorKind`: `Empty`, `TooLong`, `InvalidChar`, `LeadingDot`)が、利用者には区別を見せず、すべて `invalid value`(68)にまとめ、値を含めない。

### 5.3 `watch` / `reset` の run ID

- `--run-id` があれば、定義ファイルを読まない(環境変数は見ない)。
- なければ、定義から `pipeline.run_id_env` だけをゆるく読む。次のどれでも、既定名 `CONDUCTOR_RUN_ID` を使う: ファイルが読めない、拡張子が `.json`・`.yaml`・`.yml` のどれでもない、その書式として読めない(深さ・大きさの上限を超える場合を含む)、`pipeline` がオブジェクト(マップ)でない、キーがない、文字列でない、名前の形が 5.2 の規則(`run_id_env` の形)と違う。ほかの誤りは無視するので、壊れた定義でも動く。得た名前の環境変数を見る。
  - 「ゆるく読む」ときに、定義の検査(未知のキー、版など)は行わない。読み方の細部(YAML の重複キー、アンカーなどが入っていた場合に既定名へ戻るか)は [03-definition-format.md](03-definition-format.md) の読み込み部品に従い、ここでは「読み込みに失敗すれば既定名」とだけ規定する。
- 環境変数の値が規則違反なら `invalid value`(68)。
- `reset --all` は、環境変数も `--run-id` も見ない。

## 6. 環境変数

| 名前 | 読む場所 | 効果 |
|------|----------|------|
| `CONDUCTOR_RUN_ID` | `run`, `pipeline`, `watch`, `reset`(`--run-id` なし、かつ定義の `run_id_env` なし) | run ID の既定の供給源(5 章) |
| `run_id_env` が指す名前(定義で指定) | 同上 | `CONDUCTOR_RUN_ID` の代わりに読む環境変数名 |
| `LC_ALL`, `LC_MESSAGES`, `LANG` | 表示が必要になった最初の時点 | この順に、最初の空でない値が `ja` で始まれば日本語、それ以外は英語([08-output-and-i18n.md](08-output-and-i18n.md) 7 章) |
| `NO_COLOR` | `watch` の表を出すとき | 未設定または空なら色を使える(ただし標準出力が端末でなく、Windows でないとき。08 章 6.3)。空でない値が設定されていれば色なし |

環境変数の読み取りは、すべて `CliEnvironment` の環境変数の口を通す。これら以外の環境変数は、`conductor` 自身は読まない(子プロセスには親の環境をそのまま引き継ぐ。07 章)。.NET のランタイム自身が読む環境変数(`DOTNET_*` など)は、`conductor` の挙動の仕様には含めない。ただし、`InvariantGlobalization` はプロジェクトの設定で固定する(02 章)ので、`DOTNET_SYSTEM_GLOBALIZATION_INVARIANT` などに依存しない。ロケールの判定に `LANG` などの環境変数を、.NET の `CultureInfo` を経由せずに自前で読む(08 章 7 章)。

## 7. 各コマンドの動作

ここでは入口の検査の順序と、コマンドごとの外から見える挙動を規定する。内部の流れの詳細は [05-execution-flow.md](05-execution-flow.md)。各サブコマンドの実装は `Conductor.Cli` の同期メソッド(`int` を返す)とする。

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
- 完了(最後のジョブ、`@end`、スキップによる完了)したら、完了のステートを残して 0 で終わる。`--remove-on-complete` または定義の `remove_on_complete: true`(YAML)/ `"remove_on_complete": true`(JSON)のときだけステートを消す(消せなければ 65)。
- ジョブのあいだに中断の合図が立っていれば、次のジョブを起動せず、再開位置を記録して `128 + 番号` で終わる(`-v` のとき `[conductor] interrupted by signal N`)。
- 記述順のジョブを使い切っても完了していない(通常は起こらない)と、`cannot determine whether the pipeline is complete` を出して 65。

### 7.3 `watch`

定義ファイルを検証しない。`--run-id`/環境変数で run ID を決め(5.3)、ステートを、完了のものも含めて読み(読み込みの検査は [06-state-file.md](06-state-file.md))、表を標準出力に 1 回出して 0 で終わる。ファイルなし・長さ 0 のときだけ `no state (nothing has run yet)`。読めない・壊れているは 65(`reset` の案内は run ID なしなら付き、run ID ありなら `--run-id` つきの案内が付く。08 章 4.5)。標準出力への書き込み失敗(`IOException` など。08 章 1.3)は `cannot write to standard output` を標準エラーに出して 65。表の形式は 08 章 6 章。シグナルの準備はしない。

### 7.4 `reset`

ステートを読まず、中身の検査もしない(壊れていても動く)。ステートがなくても成功(0)。

- `--run-id` なし(かつ `--all` なし): 5.3 の規則で run ID を決め、そのステートファイルを消す。
- `--run-id <ID>`: その run ID のファイルだけを消す。
- `--all`: その定義のステートを、run ID の有無によらず全部消す。`--run-id` との併用は 68。環境変数は見ない。読めなくて残したファイルがあれば、標準エラーに警告を出す(終了コードは 0。08 章 4.10)。消すのに失敗したら 65。消すものがなければ何も出さず 0。

`--all` で消す対象(定義ファイル名を `<定義ファイル名>` とする。定義ファイルと同じディレクトリを、ファイル名の前方一致で走査する。走査には `Directory.EnumerateFileSystemEntries` などの列挙 API を使い、名前の前方一致の比較は `StringComparison.Ordinal` で行う):

- `<定義ファイル名>.state`(run ID なし): 中身を読まず、通常のファイルなら消す。シンボリックリンクは消さない(リンクかどうかは `FileSystemInfo.LinkTarget` や `FileAttributes.ReparsePoint` で判定する。リンクをたどる `File.Exists` や `File.GetAttributes` の結果だけで判断しない。これらの挙動の詳細は公式ドキュメントを取得しておらず、未確認で、07・06 章の実装時に確かめる)。
- `<定義ファイル名>.<run-id>.state`: 名前の `<run-id>` が 5.2 の規則に合い、かつ、中身の `run_id` の値が名前の run ID と一致するものだけ消す。中身が読めないものは消さずに残し、警告に挙げる。
- 一時ファイル、別の定義のステート、ディレクトリ、シンボリックリンクは消さない。

(この細かい規則は、利用者向けの説明を根拠にした記述で、細部は未確認。ステートファイルの形式は [06-state-file.md](06-state-file.md)。)

### 7.5 `validate [<pipeline-name>]`

定義を読んで検証し、全ジョブの `command` を検査するだけ。ジョブを起動せず、シグナルの準備もせず、ステートに触れない(読まず・書かず・消さない)。壊れたステートがあっても成功する。

- 成功: 何も出さず 0。`-v` のとき、標準エラーに `[conductor] definition is valid (<N> job|jobs)`(英語。`N == 1` のとき `job`、それ以外は `jobs`)/ `[conductor] 定義は正しいです(ジョブ <N> 個)`。
- 失敗: `run` と同じ表示で 64。
- 名前を渡すと定義の名前と照合する(不一致は 64)。省略すると照合しない。`--run-id` は 68(使えないオプション)。

### 7.6 `convert <file> --to <json|yaml>`

入力ファイルを、入力形式を `--to` の反対(`--to json` なら入力は YAML、`--to yaml` なら入力は JSON)とみなして変換する。この判定で、拡張子は見ない(定義ファイルの読み込みとは異なる)。結果を全部作ってから標準出力に 1 回で書く(書き込みの完了を待つ。08 章 1.3)。失敗したときは標準出力に何も書かず、標準エラーに原因を出して 67。標準出力に書けないときも 67(`cannot write to standard output`)。シグナルの準備はしない。変換の規則そのもの(コメントとキー順の扱い、アンカーの展開、入力の大きさの上限など)は [09-converter.md](09-converter.md)。

(「入力形式を `--to` の反対とみなし、拡張子を見ない」という判定の規定は、この章が持つ。`--to yaml` のときの入力は JSON として厳密に読む([09-converter.md](09-converter.md))ので、拡張子が `.yaml` でも、`--to yaml` で渡せば JSON の文法の誤り(67)になる。)

## 8. ヘルプとバージョン

- `-h`/`--help`: 使い方全文を標準出力に出して 0。言語はロケールに従う(全文は 08 章 8 章)。標準出力に書けなくても(閉じたパイプなど)、失敗は無視して 0 で終わる(08 章 1.3.2。`watch` は 65、`convert` は 67 で、ここだけが違う)。
- `-V`/`--version`: 標準出力に `conductor <版>` と改行 1 つ(`conductor 0.8.0`)。ロケールを判定しない(常にこの英語の形)。終了コード 0(標準出力に書けなくても、失敗は無視して 0。08 章 1.3.2)。版の文字列は、`Conductor.Cli` の定数 `ProgramVersion.Value`(`"0.8.0"`)から作り、`Directory.Build.props` の `<Version>` と一致することをテストで検査する(11 章)。実行時に、`Assembly.GetName().Version`、`FileVersionInfo`、`AssemblyInformationalVersionAttribute`、設定ファイルを読まない。

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

$ conductor convert conductor.json
conductor: error: missing argument: '--to'
[68]

$ conductor convert conductor.json --to xml
conductor: error: invalid value: 'xml'
[68]

$ conductor run nojob          # 定義にないジョブID
conductor: error: job is not defined in the definition file: 'nojob'
[68]

$ CONDUCTOR_RUN_ID='bad id' conductor watch
conductor: error: invalid value: 'run ID (environment variable)'
[68]

$ conductor -vh        # help が勝つ。標準出力に使い方、0
```

## 10. この版が採った選択(まとめ)

| 項目 | 内容 |
|------|------|
| 入口 | `static int Main(string[] args)`(同期)。戻り値が終了コード。`Environment.GetCommandLineArgs()` と `Environment.CommandLine` は使わない |
| 既定の定義ファイル名 | `conductor.json`(`--config` の拡張子は `.json`・`.yaml`・`.yml`。それ以外は定義の誤り `unsupportedFormat`、終了コード 64。YAML は `--config` で明示) |
| `--to` の値 | `json` か `yaml`(それ以外は `invalid value`、68) |
| `convert` | JSON と YAML の相互変換(入力形式は `--to` の反対。拡張子を見ない。出力のキーの順は入力の記述順) |
| 定義の `run_id_env` の読み方(`watch`/`reset`) | JSON/YAML をゆるく読む(読めなければ既定名。03 章) |
| 引数の解析 | `System.CommandLine` などを使わず、4 章の自前の手順(1.1) |
| 文字の単位 | コードポイント(サロゲートの対を 1 文字として扱う自前の走査。`Rune` や書記素は使わない)。比較は序数(4.0) |
| 文字として正しくない引数 | 対になっていないサロゲートまたは U+FFFD を含む文字列(`IsSuspectText`)で近似して区別し、コマンド名・ジョブID・パイプライン名・`--run-id`・`--to` の値は `invalid value`。`--config`・`convert` の入力のパスはそのまま通す(4.4.2) |
| 文字として正しくない環境変数 | 未設定と同じ扱い(5.1、08 章 7 章) |
| 誤りの表示用の変換 | `Readable()`: 対になっていないサロゲートを U+FFFD に置き換える(4.4.2) |
| run ID の検査 | 正規表現を使わない、範囲の比較(5.2) |

## 11. 未確認

- Unix で、不正な UTF-8 のバイトを含む引数・環境変数の値が、`string` にどう入るか(置換か、例外か、別の扱いか)(4.4.2)。置換される場合の、不正なバイト列 1 つあたりの U+FFFD の個数。不正な UTF-8 のバイトを含むファイル名を、`string` から開けるか(4.4.2。制約として受け入れた)。
- Windows の `args` が、`GetCommandLineArgs()` の文書の規則(4.4.1)と同じ結果になるか。空文字の引数 `""`、ワイルドカード、`cmd.exe` と PowerShell の引用符の扱いの差。
- `args` の要素と、`GetCommandLineArgs()` の 2 番目以降の要素の一致(4.4.1)。
- `System.CommandLine` について、1.1 の表で「未確認」とした挙動(値を取るオプションの次が `-` で始まるときの扱い、応答ファイルとディレクティブの無効化、重複の検出、誤りの優先順位の制御、2 個目以降の `--`)と、パッケージが正式版であるか。実機での確認はしていない。
- `string.StartsWith(string)`・`IndexOf(string)` など、文化依存の形の既定の挙動(4.0)。`System.Text.Rune` を使う走査が、4.4.2 の規定と同じ結果になるか(4.0)。
- `Environment.GetEnvironmentVariable` の、定義がないときと空のときの値、名前の大文字小文字の区別(5.1)。
- `Path` の各 API の、文字列の変化の詳細(3.3)。`--config ''` を開こうとしたときの例外の型(3.3)。
- シンボリックリンクの判定 API の挙動(7.4)。`reset --all` の対象の細部(7.4)。
- サポートする最小の .NET の版(実装時に確認する。最新の LTS と、サポート中の 1 つ前の LTS。01 章)で、1.1・4.4・5.1 の確認を繰り返すこと。
