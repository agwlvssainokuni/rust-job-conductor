# 04. コマンドライン

この章は、Go で実装する実行ファイル `conductor`(版 0.8.0)のコマンドラインを規定する。引数の解釈は、標準ライブラリの `flag` パッケージを使わず、字句の段階(4.1)と検査の段階(4.2)を**自前の手順**で実装する。メッセージの全文は [08-output-and-i18n.md](08-output-and-i18n.md)、終了コードとエラー型は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md)、各コマンドの内部の流れは [05-execution-flow.md](05-execution-flow.md)、定義ファイルの書式は [03-definition-format.md](03-definition-format.md)、変換は [09-converter.md](09-converter.md) を参照する。

## 1. 全体像

- 実行ファイル名は `conductor`。引数は `os.Args[1:]`(プログラム名を除いた文字列の並び)である。引数はシェルを介さず、`conductor` 自身は展開(ワイルドカード、変数、チルダなど)も解釈もしない。
- サブコマンドは 6 つだけ: `run`, `pipeline`, `watch`, `reset`, `validate`, `convert`。
- 特別なオプション(`-h`/`--help`, `-V`/`--version`)は、コマンドの解釈より先に処理する(4.2 の 1)。
- 使い方の誤りのときは、標準エラーに「誤りの 1 行 + 空行 + 使い方全文」を書き、終了コード 68 で終わる。ジョブは実行しない。
- 既定では、`conductor` 自身は何も出力しない(ジョブ本体の出力がそのまま流れる)。エラーのとき、または `-v` のときだけ標準エラーに書く。例外として、`watch`・`--help`・`--version`・`convert` は結果を標準出力に書く。
- プロセスの終了コードを決める箇所は、`cmd/conductor/main.go` の 1 か所だけにする。`internal/cli` の `Run` は終了コード(`int`)を値として返し、`main` が `os.Exit` を 1 回だけ呼ぶ(10 章 8 章)。

### 1.1 `flag` パッケージを使わない理由

仕様の文法が、`flag` の既定の挙動と合わないため、自前で実装する。`flag` の挙動について、この仕様は「合わない」と判断した根拠を次の項目に限る(`flag` の細部の挙動の断定はしない。実装時に確認する)。

| 仕様が要求すること | `flag` との関係 |
|--------------------|-----------------|
| 短縮形の連結(`-vh`、`-vv`)を解釈する | `flag` は、1 つの `-` のあとの文字列全体を 1 つのフラグ名として扱う作りで、短縮形の連結を持たない |
| 長い形 `--opt` に対し、値を取るオプションは次の引数を中身にかかわらず値にする(`--config --help` は `--help` が値) | 真偽値のフラグと値を取るフラグの扱いが、`flag` 側の規則に縛られる |
| `-h`/`--help`/`-V`/`--version` を、ほかの誤りより先に、先に現れた方を採る | `flag` の `-h`/`-help` の扱いは `flag` が決める(`-V` と `--version` は定義が必要) |
| オプションをコマンド名の前にも後ろにも置ける。`--` のあとはすべて位置引数 | `flag` は最初の位置引数で解析を止める作り(位置引数のあとのオプションを解析しない) |
| 誤りの種類と対象(`unknown option: '-x'` など)を、固定の順序で返す | `flag` の誤りの文言と順序は `flag` が決める |
| `-v=1` を `invalid value: '-v'` にする | 短縮形の `=` の扱いが仕様と別 |

そのため、`internal/cli` に字句の段階の関数と検査の段階の関数を置き、標準ライブラリは `strings`・`unicode/utf8`・`os` だけを使う。

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
| `--config <path>` | 定義ファイルのパス。文字列として加工せず、そのままファイルシステムに渡す(`os.Open` などに渡す。正規化・絶対化しない)。書式は拡張子で決まる(`.yaml`・`.yml` は YAML、`.json` は JSON、それ以外は拒否。[03-definition-format.md](03-definition-format.md)) | 実行ディレクトリ(カレントディレクトリ)の `conductor.yaml` |
| `--run-id <id>` | ステートファイルを分ける run ID。空文字は「run ID なしに固定」(環境変数を見ない) | 未指定(環境変数を見る。5 章) |
| `--remove-on-complete` | 完了時にステートファイルを削除する | 指定なし(その場合は定義の `pipeline.remove_on_complete`、それも無ければ `false`) |
| `--all` | 定義のステートファイルを、run ID の有無によらずすべて消す(`reset` のみ) | なし |
| `--to <yaml\|json>` | `convert` の出力形式。大文字小文字を区別し、`yaml` と `json` 以外は `invalid value`(対象は値そのもの) | なし(必須) |

既定の定義ファイル名 `conductor.yaml` は、`.yml`・`.json` を探索しない。既定の名前でファイルがなければ、通常の「ファイルなし」(64)になる。

## 4. 引数の解釈

解釈は、(A)字句の段階、(B)コマンドの組み立てと検査、の 2 段で行う。(A)は引数を先頭から 1 つずつ見て、`special`(最初に現れた `-h`/`-V` の種別)、`lexError`(最初の字句の誤り)、`tokens`(オプションと位置引数の並び)を作る。(B)は `tokens` を検査する。

### 4.0 内部の表現(骨格)

型の名前と分け方は目安であり、4.1・4.2 の挙動を満たせば変えてよい。

```go
// 字句の段階の結果
type lexResult struct {
    special  specialKind // none / help / version(最初に現れた方)
    lexError *usageError // 最初の字句の誤り(なければ nil)
    tokens   []token
}

type tokenKind int // positional / verbose / config / runID / to / all / removeOnComplete / unknown

type token struct {
    kind  tokenKind
    value string // 位置引数・値を取るオプションの値。unknown では綴り(-x, --bogus)
}
```

引数は Go の `string` のまま扱う(非 UTF-8 のバイト列も保持できる)。`unicode/utf8` の `utf8.ValidString` で、UTF-8 として正しいかを判定する。

### 4.1 字句の段階(A)の手順

`afterDashDash = false` として、引数 `a` を先頭から順に処理する。

1. `afterDashDash` が真なら、`a` は位置引数(`-` で始まっていても、`-v` や `--help` であっても位置引数)。
2. `a` が `--` ちょうどなら、`afterDashDash = true` にする(`--` 自体は位置引数にならない)。2 個目以降の `--` は、`afterDashDash` が真なので位置引数になる。
3. `a` が `--` で始まり(`--` より長い)、長いオプションとして扱う。`=` が含まれていれば、最初の `=` の前を名前(`--` を除く)、後ろを値(空でもよい)とする。含まれていなければ名前は `a` から `--` を除いた全体で、値なし。
   - 名前が `help`: `special` が未設定なら help を設定する。`=値` が付いていても、help が優先されるので誤りにならない(例: `--help=x` は help)。
   - 名前が `version`: 同様に `special` が未設定なら version を設定する(`--version=x` も version)。
   - 名前が `verbose`, `all`, `remove-on-complete`(値を取らない): `=値` が付いていれば(値が空でも)`lexError`(最初の 1 つだけ保持)に `invalid value`(対象はそのオプションの正規の綴り、例 `--verbose`)を設定する。付いていなければ、そのオプションを `tokens` に追加する。
   - 名前が `config`, `run-id`, `to`(値を取る): `=値` があれば、それ(空文字も有効)を値として `tokens` に追加する。なければ、次の引数を、中身にかかわらず(`-` で始まっていても、`--` や `--help` であっても)値として取る(その引数は位置引数にもオプションにもならない)。次の引数が存在しないときは、`lexError` に `option requires a value`(対象は `--config` / `--run-id` / `--to`)を設定し、字句の段階を打ち切る(以降の引数は見ない)。
   - それ以外の名前(空の名前を含む): 「知らないオプション `--<名前>`」を `tokens` に追加する。`=値` が付いていたときは、`lexError` にも `unknown option`(対象 `--<名前>`、値は含めない)を設定する。名前が空のときの綴りは `--`(例: `--=x` は `unknown option: '--'`)。`---` は名前が `-` で、綴りは `---`。
4. `a` が `-` で始まり、長さが 2 以上で、`--` で始まらない: 短縮形の連結として、先頭の `-` を除く文字を、先頭から 1 文字(Unicode のコードポイント。`for _, r := range` の 1 回)ずつ処理する。
   - `h`: help を設定(上と同じ。`special` が未設定のときだけ)。`V`: version を設定(同様)。`v`: `-v` を `tokens` に追加する。
   - ほかの文字 `c`: 「知らないオプション `-<c>`」を `tokens` に追加する(`-é` は `unknown option: '-é'`)。
   - 2 文字目以降の位置で、直前の 1 文字の直後に `=` が続いているとき、その `=` 以降の残りの文字列は**まとめて捨て**(残りの文字は解釈しない)、`lexError` を設定する。直前の文字が `v` なら `invalid value`(対象 `-v`)、知らない文字 `c` なら `unknown option`(対象 `-<c>`)。直前の文字が `h`・`V` のときは `special` が優先されるので誤りにならない。連結の先頭の文字が `=` のとき(`-=x`)は、区切りではなく、`=` という文字そのものが知らないオプション(`unknown option: '-='`)。
   - 短縮形は値を取らない(値を取るのは長いオプションだけ)。`-vv` は `-v` が 2 回。`-vh` は `-v` と help(help が勝つ)。
5. それ以外(`-` ちょうど 1 文字、`-` で始まらない文字列、空文字)は位置引数。単独の `-` も位置引数である。

`lexError` は最初の 1 つだけを保持する(後の字句の誤りは捨てる)。

非 UTF-8 のバイトを含む引数を 4 の連結として分解するときの `-<c>` の綴りは未確認(`range` は不正なバイトを U+FFFD として 1 バイトずつ返す。そのまま `-�` の綴りにしてよいが、実機との一致は確認していない)。

補足(実機で確認した挙動):

- `--help=x`・`-V=1`・`--version=x` は、それぞれ help / version / version。`-vx run a` は `unknown option: '-x'`、`run -vh` は help、`-vV` は version、`--config --help run a` は `--help` という名前のパスを定義ファイルとして扱う(help ではない。拡張子が `.yaml`・`.yml`・`.json` のどれでもないので、定義の誤り `unsupportedFormat` として 64)。
- `--config --help`(ほかの引数なし)は `--help` を値に取り、位置引数が 0 個なので `missing argument`(68)。`run a --config --version` も `--version` を値(パス)として扱う。
- `--help --config` は、`--help` が先に解釈されるので、値の欠落(`--config`)より前に help(0)になる。
- `run -- a --` は 2 個目の `--` が位置引数になり `unexpected argument: '--'`。`run -- -v` は、ジョブID `-v`。`run -` は、ジョブID `-`。
- `-v=1 run a` と `run -v=1h x` は、どちらも `invalid value: '-v'`(`=` 以降の `h` は解釈されず、help にならない)。`-v=` も `invalid value: '-v'`。`-vh=1` は help。`-x=1h` と `-vx=1` は `unknown option: '-x'`。
- `--=x`・`--=` は `unknown option: '--'`、`---` は `unknown option: '---'`、`--verbose=`(値が空)は `invalid value: '--verbose'`。
- `reset --all=1` は `invalid value: '--all'`。`run j --run-id`・`--to`(値なし)は `option requires a value`。

### 4.2 検査の順序(誤りの優先順位)

字句の段階のあと、次の順に検査する。最初に当てはまったもので終わる(それ以降は行わない)。

1. `special` が設定されていれば(`--` より前に、オプションとして解釈された `-h/--help` か `-V/--version` が 1 つでもあれば)、ほかの誤りに関係なく、先に現れた方を処理して終了コード 0。値を取るオプション(`--config`、`--run-id`、`--to`)の直後の引数は、`-h` や `--help` であっても値として取られ(4.1 の 3)、オプションとして解釈されないので、`special` にならない(`--config --help run a` は help ではない。`-h --config --version` は、`-h` が先に現れるので help で、`--version` は `--config` の値になる)。両方あるとき、先に現れた方が勝つ(`-V -h` は version、`-h -V` は help)。字句の段階が値の欠落で打ち切られた場合、それ以前に現れた `-h`/`-V` は有効、以降は存在しない扱い。
2. `lexError` があれば、それを誤りとする。
3. 「知らないオプション」が `tokens` にあれば、最初のものを `unknown option`(対象はその綴り: `-x`, `--bogus`)とする。
4. 位置引数が 0 個: `missing argument`(対象なし)。例: 引数なし、`-v` だけ、`--` だけ。
5. 先頭の位置引数(コマンド名)が非 UTF-8 なら `invalid value`(対象は、読める形に変換した文字列。4.4)。6 コマンドのいずれでもなければ `unknown command`(対象はその語。空文字も同じ)。
6. そのコマンドで使えないオプション: `option not allowed for this command`。検査の順は `-v`, `--config`, `--to`, `--run-id`, `--all`, `--remove-on-complete`。対象はそのオプション名(`-v` の綴りは `-v`)。可否は 3.1 の表による。
7. 重複: `-v`, `--config`, `--to`, `--run-id`, `--all`, `--remove-on-complete` のどれかが 2 回以上 → `invalid value`(対象はそのオプション名。`-vv` は対象 `-v`)。検査の順は挙げた順。`-v` の重複は、`-v` と `--verbose` の合計で数える。
8. `--all` と `--run-id` の併用 → `invalid value`(対象 `--all`)。
9. `--run-id` の値の検査(5.2)。値が非 UTF-8、または空でなく規則に違反 → `invalid value`(対象 `--run-id`。値は含めない)。空文字は有効。
10. 位置引数の過不足(コマンド名を除いて数える):
    - `run`・`pipeline`・`convert` で位置引数が 0 個 → `missing argument`(対象なし)。
    - `watch`・`reset` で位置引数が 1 個以上 → `unexpected argument`(対象は最初の位置引数)。
    - コマンドが取れる個数を超える(2 個目の位置引数がある)→ `unexpected argument`(対象は 2 個目の語。`run a b` なら `b`)。
    - 位置引数(ジョブID、パイプライン名)が非 UTF-8 → `invalid value`(対象は読める形に変換した文字列)。`convert` の入力ファイルは、非 UTF-8 でもパスとしてそのまま通す。
11. `convert`: `--to` なし → `missing argument`(対象 `--to`)。`--to` の値が `yaml`/`json` 以外(非 UTF-8 を含む)→ `invalid value`(対象は値)。

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
| `run a -x` | `unknown option: '-x'`(68) |
| `run a -- b` | `unexpected argument: 'b'`(68) |
| `--run-id= watch` | 有効(run ID なしに固定)。ステートなしなら `no state (nothing has run yet)`、0 |
| `run -- -v` | ジョブID `-v` |
| `run a --to yaml` | `option not allowed for this command: '--to'`(68) |
| `convert --to yaml --to json f` | `invalid value: '--to'`(68) |
| `convert f --to xml` | `invalid value: 'xml'`(68)(`--to` の値は `yaml`・`json` だけ。大文字小文字も区別する) |
| `convert f` | `missing argument: '--to'`(68) |
| `reset --all --run-id x` | `invalid value: '--all'`(68) |

### 4.4 非 UTF-8 の引数を「読める形」にする規則

使い方の誤りの対象に、非 UTF-8 の引数を出すときは、不正なバイト列を U+FFFD(`�`)に置き換える。置き換えの単位は、Unicode の「最大の部分列」の規則(不正な列の、正しい列の先頭になりうる最長の部分を、1 つの U+FFFD にする)にそろえる。実機の確認結果:

| 入力(バイト列) | 出力 |
|------------------|------|
| `a` + `E3 81`(3 バイト列の途中で切れている) | `a` + U+FFFD 1 個 |
| `E3 81 82` + `E3` | `あ` + U+FFFD 1 個 |
| `FF FE` | U+FFFD 2 個 |

Go の標準ライブラリで注意する点(実装時に確認する): `strings.ToValidUTF8(s, "�")` は、連続する不正なバイトを**1 つ**の置換にまとめるので、`FF FE` が 1 個になり、上の表と合わない。`for range` で文字列を走査すると不正なバイトごとに U+FFFD が出るので、`E3 81` が 2 個になり、上の表と合わない。したがって、標準の関数をそのまま使わず、`utf8.DecodeRuneInString` を使って、不正な位置から、正しい列の先頭になりうる最長の部分(`E3 81` のような途中で切れた列)を 1 つの不正な単位として消費する関数を自前で書く。「正しい列の先頭になりうる」の判定は、UTF-8 の符号化規則(先頭バイトの範囲と、2 バイト目以降の範囲の制約。`E0` の次は `A0`〜`BF`、`ED` の次は `80`〜`9F`、`F0` の次は `90`〜`BF`、`F4` の次は `80`〜`8F`、それ以外の継続バイトは `80`〜`BF`)で行う。上の 3 件の表は、受け入れテストに使ってよい(確認済み)。それ以外の入力での一致は未確認。

Windows で `os.Args` が、不正な UTF-16 をどう文字列にするかは未確認(実装時に確認する。不正な UTF-16 を含む引数は、`invalid value` 扱いにしてよい)。

### 4.5 実装に任せてよい点

- 内部の語・型の分け方(4.0 は目安)。
- 4.1 の挙動(値を取るオプションが次の引数を無条件に値にする、`=` の扱い、`--`、連結)と、4.2 の検査の順序は必須。
- 単独の `-` を位置引数とする挙動は必須(`run -` はジョブID `-`)。

## 5. run ID

### 5.1 決定の優先順位(`run`・`pipeline`)

1. `--run-id <値>` が指定されていれば、環境変数は見ない。値が空文字なら run ID なし。
2. 指定がなければ、環境変数を読む(`os.LookupEnv`)。環境変数の名前は、定義の `pipeline.run_id_env`(なければ `CONDUCTOR_RUN_ID`)。未定義または空文字なら run ID なし。値が UTF-8 でない(`utf8.ValidString` が偽)ときは、未定義と同じ扱い。
3. 得た値は 5.2 の規則で検査する。違反は `invalid value`(終了コード 68)。対象は、オプション由来なら `--run-id`、環境変数由来なら `run ID (environment variable)`(値は含めない)。

`run`・`pipeline` では、環境変数の名前を知るために、定義を先に読む。検査の順序は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md) 2 章を参照。

### 5.2 run ID の規則

- 文字は `A-Za-z0-9._-` だけ、長さは 1〜64 文字(バイト数でも文字数でも同じ。ASCII だけなので)、先頭が `.` でない(`..` も不可)。空文字は「run ID なし」を表し、値としては規則を満たさない(オプションの空文字だけが「なし」を意味する特別な扱い)。
- ステートファイル名は、run ID なしで `<定義ファイル名>.state`、ありで `<定義ファイル名>.<run-id>.state`(定義ファイルと同じディレクトリ)。詳細は [06-state-file.md](06-state-file.md)。
- 定義の `run_id_env` は、`A-Za-z_` で始まり `A-Za-z0-9_` が続く 1 文字以上。違反は定義の誤り(64。[03-definition-format.md](03-definition-format.md))。
- 検査は `internal/runid` に置く。理由の区別(空・長すぎ・不正な文字・先頭のドット)は内部では持ってよいが、利用者には区別を見せず、すべて `invalid value`(68)にまとめ、値を含めない。

### 5.3 `watch` / `reset` の run ID

- `--run-id` があれば、定義ファイルを読まない(環境変数は見ない)。
- なければ、定義から `pipeline.run_id_env` だけをゆるく読む。次のどれでも、既定名 `CONDUCTOR_RUN_ID` を使う: ファイルが読めない、拡張子が `.yaml`・`.yml`・`.json` のどれでもない、その書式として読めない(深さ・大きさの上限を超える場合を含む)、`pipeline` がマップでない、キーがない、文字列でない、名前の形が 5.2 の規則と違う。ほかの誤りは無視するので、壊れた定義でも動く。得た名前の環境変数を見る。
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

環境変数の読み取りは、すべて `Deps` の環境変数の口(`LookupEnv(name string) (string, bool)`)を通す。

## 7. 各コマンドの動作

ここでは入口の検査の順序と、コマンドごとの外から見える挙動を規定する。内部の流れの詳細は [05-execution-flow.md](05-execution-flow.md)。

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

定義ファイルを検証しない。`--run-id`/環境変数で run ID を決め(5.3)、ステートを、完了のものも含めて読み(読み込みの検査は [06-state-file.md](06-state-file.md))、表を標準出力に 1 回出して 0 で終わる。ファイルなし・長さ 0 のときだけ `no state (nothing has run yet)`。読めない・壊れているは 65(`reset` の案内は run ID なしなら付き、run ID ありなら `--run-id` つきの案内が付く。08 章 4.5)。標準出力への書き込み失敗は `cannot write to standard output` を標準エラーに出して 65。表の形式は 08 章 6 章。シグナルの準備はしない。

### 7.4 `reset`

ステートを読まず、中身の検査もしない(壊れていても動く)。ステートがなくても成功(0)。

- `--run-id` なし(かつ `--all` なし): 5.3 の規則で run ID を決め、そのステートファイルを消す。
- `--run-id <ID>`: その run ID のファイルだけを消す。
- `--all`: その定義のステートを、run ID の有無によらず全部消す。`--run-id` との併用は 68。環境変数は見ない。読めなくて残したファイルがあれば、標準エラーに警告を出す(終了コードは 0。08 章 4.10)。消すのに失敗したら 65。消すものがなければ何も出さず 0。

`--all` で消す対象(定義ファイル名を `<定義ファイル名>` とする。定義ファイルと同じディレクトリを、ファイル名の前方一致で走査する。走査には `os.ReadDir` を使ってよい):

- `<定義ファイル名>.state`(run ID なし): 中身を読まず、通常のファイルなら消す。シンボリックリンクは消さない(`DirEntry` の種別、または `os.Lstat` で判定する)。
- `<定義ファイル名>.<run-id>.state`: 名前の `<run-id>` が 5.2 の規則に合い、かつ、中身の `run_id` の値が名前の run ID と一致するものだけ消す。中身が読めないものは消さずに残し、警告に挙げる。
- 一時ファイル、別の定義のステート、ディレクトリ、シンボリックリンクは消さない。

(この細かい規則は、利用者向けの説明を根拠にした記述で、細部は未確認。ステートファイルの形式は [06-state-file.md](06-state-file.md)。)

### 7.5 `validate [<pipeline-name>]`

定義を読んで検証し、全ジョブの `command` を検査するだけ。ジョブを起動せず、シグナルの準備もせず、ステートに触れない(読まず・書かず・消さない)。壊れたステートがあっても成功する。

- 成功: 何も出さず 0。`-v` のとき、標準エラーに `[conductor] definition is valid (<N> job|jobs)`(英語。`N == 1` のとき `job`、それ以外は `jobs`)/ `[conductor] 定義は正しいです(ジョブ <N> 個)`。
- 失敗: `run` と同じ表示で 64。
- 名前を渡すと定義の名前と照合する(不一致は 64)。省略すると照合しない。`--run-id` は 68(使えないオプション)。

### 7.6 `convert <file> --to <yaml|json>`

入力ファイルを、入力形式を `--to` の反対(`--to json` なら入力は YAML、`--to yaml` なら入力は JSON)とみなして変換する。この判定で、拡張子は見ない(定義ファイルの読み込みとは異なる)。結果を全部作ってから標準出力に 1 回で書く。失敗したときは標準出力に何も書かず、標準エラーに原因を出して 67。標準出力に書けないときも 67(`cannot write to standard output`)。シグナルの準備はしない。変換の規則そのもの(コメントとキー順の扱い、アンカーの展開、入力の大きさの上限など)は [09-converter.md](09-converter.md)。

(「入力形式を `--to` の反対とみなし、拡張子を見ない」という判定の規定は、この章が持つ。`--to yaml` のときの入力は JSON として厳密に読む(RFC 8259。[09-converter.md](09-converter.md) 4.1)ので、拡張子が `.yaml` でも、`--to yaml` で渡せば JSON の文法の誤り(67)になる。)

## 8. ヘルプとバージョン

- `-h`/`--help`: 使い方全文を標準出力に出して 0。言語はロケールに従う(全文は 08 章 8 章)。
- `-V`/`--version`: 標準出力に `conductor <版>` と改行 1 つ(`conductor 0.8.0`)。ロケールを判定しない(常にこの英語の形)。終了コード 0。版の文字列は `internal/version` の定数 `Version = "0.8.0"` から作る。

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

$ conductor -vh        # help が勝つ。標準出力に使い方、0
```

## 10. この版が採った選択(まとめ)

| 項目 | 内容 |
|------|------|
| 既定の定義ファイル名 | `conductor.yaml`(`--config` の拡張子は `.yaml`・`.yml`・`.json`。それ以外は定義の誤り `unsupportedFormat`、終了コード 64) |
| `--to` の値 | `yaml` か `json`(それ以外は `invalid value`、68) |
| `convert` | YAML と JSON の相互変換(入力形式は `--to` の反対。拡張子を見ない。出力のキーの順は入力の記述順) |
| 定義の `run_id_env` の読み方(`watch`/`reset`) | YAML/JSON をゆるく読む(読めなければ既定名。03 章 15 節) |
| 引数の解析 | `flag` を使わず、4 章の自前の手順 |

## 11. 未確認

- `=` を含み、かつ連結の途中に `=` がある短縮形(`-v=1h` の `h` が捨てられる以外の組み合わせ)の細部。
- 非 UTF-8 のバイトを含む連結の短縮形の綴り(4.1)。4.4 の表にない非 UTF-8 の入力の置換の一致。
- Windows の `os.Args` での不正な UTF-16 の扱い。
- `reset --all` の対象の細部(7.4)。
- 実際の Go のバージョンでの `flag` の細部の挙動(1.1 は設計判断の根拠であり、`flag` の挙動の断定ではない)。
