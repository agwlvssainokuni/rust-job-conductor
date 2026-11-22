# 04. コマンドライン

この章は、実行ファイル `conductor`(v0.9.0 の機能を含む)のコマンドラインを、実装の言語やライブラリに依存せずに規定する。引数の文法は、字句の分け方の手順(2 章)と、検査の順序(4 章)で決まる。メッセージの全文は [08-output-and-i18n.md](08-output-and-i18n.md)、終了コードは [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md)、各コマンドの内部の流れは [05-execution-flow.md](05-execution-flow.md) を参照する。

## 1. 全体像

- 実行ファイル名は `conductor`。引数は、プログラム名を除いた文字列の並びである。引数はシェルを介さず、`conductor` 自身は展開(ワイルドカード、変数、チルダなど)も解釈もしない。
- サブコマンドは 8 つだけ: `run`, `pipeline`, `watch`, `reset`, `validate`, `convert`, `graph`, `init`(後ろの 2 つは v0.9.0 以降。v0.8.0 以前は 6 つ)。
- 特別なオプション(`-h`/`--help`, `-V`/`--version`)は、コマンドの解釈より先に処理する(4 章)。
- 使い方の誤りのときは、標準エラーに「誤りの 1 行 + 空行 + 使い方全文」を書き、終了コード 68 で終わる。ジョブは実行しない。
- 既定では、`conductor` 自身は何も出力しない(ジョブ本体の出力がそのまま流れる)。エラーのとき、または `-v` のときだけ標準エラーに書く。例外として、`watch`・`--help`・`--version`・`convert` は結果を標準出力に書く(v0.9.0 以降は、`graph` の図、`--dry-run` の計画、`init --config -` のひな形も標準出力。`init` が作った・置き換えたの知らせは標準エラー)。
- プロセスの終了コードを決める箇所は、プログラムの最上位の 1 か所だけにする(各処理は終了コードを値として返し、最上位が 1 回だけプロセスを終了させる)。

## 2. 構文

```text
conductor run [-v] [--config <path>] [--run-id <id>] [--remove-on-complete] [--dry-run [--assume <job-id>=<code>]...] <job-id>
conductor pipeline [-v] [--config <path>] [--run-id <id>] [--remove-on-complete] [--dry-run [--assume <job-id>=<code>]...] <pipeline-name>
conductor watch [--config <path>] [--run-id <id>]
conductor reset [--config <path>] [--run-id <id> | --all]
conductor validate [-v] [--config <path>] [<pipeline-name>]
conductor graph [-v] [--config <path>] [--format <text|mermaid>] [--state [--run-id <id>]] [<pipeline-name>]
conductor init [--config <path>] [--force] [<pipeline-name>]
conductor convert <file> --to <yaml|toml>
conductor --help | --version
```

(`--dry-run`・`--assume`・`graph`・`init`・`--format`・`--state`・`--force` は v0.9.0 以降。)

## 3. オプションと位置引数

### 3.1 オプションの一覧と、コマンドごとの可否

| オプション | 短縮形 | 値 | run | pipeline | watch | reset | validate | convert | graph | init |
|------------|--------|----|-----|----------|-------|-------|----------|---------|-------|------|
| `--verbose` | `-v` | なし | 可 | 可 | 不可 | 不可 | 可 | 不可 | 可 | 不可 |
| `--config` | なし | パス(必須) | 可 | 可 | 可 | 可 | 可 | 不可 | 可 | 可(`-` は標準出力) |
| `--run-id` | なし | 文字列(空も可) | 可 | 可 | 可 | 可 | 不可 | 不可 | `--state` と一緒のときだけ | 不可 |
| `--remove-on-complete` | なし | なし | 可 | 可 | 不可 | 不可 | 不可 | 不可 | 不可 | 不可 |
| `--all` | なし | なし | 不可 | 不可 | 不可 | 可 | 不可 | 不可 | 不可 | 不可 |
| `--to` | なし | `yaml` か `toml`(必須) | 不可 | 不可 | 不可 | 不可 | 不可 | 必須 | 不可 | 不可 |
| `--format`(v0.9.0 以降) | なし | `text` か `mermaid` | 不可 | 不可 | 不可 | 不可 | 不可 | 不可 | 可 | 不可 |
| `--state`(同) | なし | なし | 不可 | 不可 | 不可 | 不可 | 不可 | 不可 | 可 | 不可 |
| `--dry-run`(同) | なし | なし | 可 | 可 | 不可 | 不可 | 不可 | 不可 | 不可 | 不可 |
| `--assume`(同) | なし | `<ジョブID>=<終了コード>`(複数回) | `--dry-run` と一緒のときだけ | `--dry-run` と一緒のときだけ | 不可 | 不可 | 不可 | 不可 | 不可 | 不可 |
| `--force`(同) | なし | なし | 不可 | 不可 | 不可 | 不可 | 不可 | 不可 | 不可 | 可 |
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
| `graph`(v0.9.0 以降) | パイプライン名 | 0 または 1(省略時は名前を照合しない) |
| `init`(v0.9.0 以降) | パイプライン名 | 0 または 1(省略時は `my-pipeline`。空・空白だけは `invalid value`、対象 `pipeline name`) |
| `convert` | 入力ファイル | ちょうど 1(`--to` は別に必須) |

オプションは、コマンド名の前にも後ろにも置ける(例: `conductor --config c.toml -v run x` は有効)。

### 3.3 各オプションの意味と既定値

| オプション | 意味 | 既定 |
|------------|------|------|
| `-v`, `--verbose` | ジョブの結果・スキップ・遷移・中断を標準エラーに出す(`validate` は成功時に 1 行)。`graph` では、ジョブごとの詳細(コマンド、リトライ、飛び越し)を標準出力の図に足す。段階は 1 つだけ(詳しさの度合いを選べない) | 出さない |
| `--config <path>` | 定義ファイルのパス。文字列として解釈せず、そのままファイルシステムに渡す(OS が許す任意のパスを受け付ける)。`init` では、作る先のパス(`-` だけの値は、標準出力を意味する) | 実行ディレクトリ(カレントディレクトリ)の `conductor.toml` |
| `--run-id <id>` | ステートファイルを分ける run ID。空文字は「run ID なしに固定」(環境変数を見ない) | 未指定(環境変数を見る。5 章) |
| `--remove-on-complete` | 完了時にステートファイルを削除する | 指定なし(その場合は定義の `[pipeline].remove_on_complete`、それも無ければ `false`) |
| `--all` | 定義のステートファイルを、run ID の有無によらずすべて消す(`reset` のみ) | なし |
| `--to <yaml\|toml>` | `convert` の出力形式。大文字小文字を区別し、`yaml` と `toml` 以外は `invalid value`(対象は値そのもの) | なし(必須) |
| `--format <text\|mermaid>`(v0.9.0 以降) | `graph` の出力形式。大文字小文字を区別し、それ以外は `invalid value`(対象 `--format`。値は含めない) | `text` |
| `--state`(同) | `graph` で、ステートを読み、状態を図に重ねる(読むだけ) | 重ねない |
| `--dry-run`(同) | `run`・`pipeline` で、ジョブを実行せず、ステートに書かず、シグナルも準備せずに、実行計画を標準出力に出す | 実行する |
| `--assume <ジョブID>=<終了コード>`(同) | 計画の中のジョブの終了コード(0〜255)を仮定する。最初の `=` で分ける。複数回指定できる(指定順を保つ) | 指定のないジョブは 0 |
| `--force`(同) | `init` で、既にあるファイルを上書きする(`--config -` のときは無視) | 上書きしない |

## 4. 引数の解釈

解釈は、(A)字句の段階、(B)コマンドの組み立てと検査、の 2 段で行う。(A)は引数を先頭から 1 つずつ見て、`special`(最初に現れた `-h`/`-V` の種別)、`lexError`(最初の字句の誤り)、`tokens`(オプションと位置引数の並び)を作る。(B)は `tokens` を検査する。

### 4.1 字句の段階(A)の手順

`afterDashDash = false` として、引数 `a` を先頭から順に処理する。

1. `afterDashDash` が真なら、`a` は位置引数(`-` で始まっていても、`-v` や `--help` であっても位置引数)。
2. `a` が `--` ちょうどなら、`afterDashDash = true` にする(`--` 自体は位置引数にならない)。2 個目以降の `--` は、`afterDashDash` が真なので位置引数になる。
3. `a` が `--` で始まり(`--` より長い)、長いオプションとして扱う。`=` が含まれていれば、最初の `=` の前を名前、後ろを値(空でもよい)とする。含まれていなければ名前は `a` 全体(`--` を除く)で、値なし。
   - 名前が `help`: `special` が未設定なら help を設定する。`=値` が付いていても、help が優先されるので誤りにならない(例: `--help=x` は help)。
   - 名前が `version`: 同様に `special` が未設定なら version を設定する(`--version=x` も version)。
   - 名前が `verbose`, `all`, `remove-on-complete`, `state`, `dry-run`, `force`(値を取らない。後ろの 3 つは v0.9.0 以降): `=値` が付いていれば `lexError`(最初の 1 つだけ保持)に `invalid value`(対象はそのオプションの正規の綴り、例 `--verbose`)を設定する。付いていなければ、そのオプションを `tokens` に追加する。
   - 名前が `config`, `run-id`, `to`, `format`, `assume`(値を取る。後ろの 2 つは v0.9.0 以降。`--assume=a=1` の値は `a=1`): `=値` があれば、それ(空文字も有効)を値として `tokens` に追加する。なければ、次の引数を、中身にかかわらず(`-` で始まっていても、`--` や `--help` であっても)値として取る。次の引数が存在しないときは、`lexError` に `option requires a value`(対象は `--config` / `--run-id` / `--to` / `--format` / `--assume`)を設定し、字句の段階を打ち切る(以降の引数は見ない)。
   - それ以外の名前: 「知らないオプション `--<名前>`」を `tokens` に追加する。`=値` が付いていたときは、`lexError` にも `unknown option`(対象 `--<名前>`、値は含めない)を設定する。
4. `a` が `-` で始まり、長さが 2 以上で、`--` で始まらない: 短縮形の連結として、先頭の `-` を除く文字を 1 文字ずつ処理する。
   - `h`: help を設定(上と同じ。`special` が未設定のときだけ)。`V`: version を設定(同様)。`v`: `-v` を `tokens` に追加する。
   - ほかの文字 `c`: 「知らないオプション `-<c>`」を `tokens` に追加する。
   - 1 文字の直後に `=` が続いているとき、`=` 以降は「値の取り残し」として扱い、`lexError` を設定する。文字が `v` なら `invalid value`(対象 `-v`)、知らない文字 `c` なら `unknown option`(対象 `-<c>`)。`h`・`V` のときは `special` が優先されるので誤りにならない。例: `-v=1` は `invalid value: '-v'`(確認済み)。`=` が連結の途中にある場合(`-v=1h` など)の細部は未確認。
   - 短縮形は値を取らない(値を取るのは長いオプションだけ)。`-vv` は `-v` が 2 回。`-vh` は `-v` と help(help が勝つ)。
5. それ以外(`-` ちょうど 1 文字、`-` で始まらない文字列、空文字)は位置引数。単独の `-` も位置引数である。

`lexError` は最初の 1 つだけを保持する(後の字句の誤りは捨てる)。

補足(確認済みの実機の挙動): `--help=x`・`-V=1`・`--version=x` はそれぞれ help / version / version。`-vx run a` は `unknown option: '-x'`、`run -vh` は help、`-vV` は version、`--config --help run a` は `--help` という名前のパスを定義ファイルとして扱う(help ではない。ファイルがなければ 64)。`--config --help`(ほかの引数なし)は `--help` を値に取り、位置引数が 0 個なので `missing argument`(68)。`run a --config --version` も `--version` を値(パス)として扱う。`--help --config` は、`--help` が先に解釈されるので、値の欠落(`--config`)より前に help(0)になる。`run -- a --` は 2 個目の `--` が位置引数になり `unexpected argument: '--'`。`run -- -v` は、ジョブID `-v`。`run -` は、ジョブID `-`。

### 4.2 検査の順序(誤りの優先順位)

字句の段階のあと、次の順に検査する。最初に当てはまったもので終わる(それ以降は行わない)。

1. `special` が設定されていれば(`--` より前に、オプションとして解釈された `-h/--help` か `-V/--version` が 1 つでもあれば)、ほかの誤りに関係なく、先に現れた方を処理して終了コード 0。値を取るオプション(`--config`、`--run-id`、`--to`)の直後の引数は、`-h` や `--help` であっても値として取られ(4.1 の 3)、オプションとして解釈されないので、`special` にならない(`--config --help run a` は help ではない。`-h --config --version` は、`-h` が先に現れるので help で、`--version` は `--config` の値になる)。両方あるとき、先に現れた方が勝つ(`-V -h` は version、`-h -V` は help)。字句の段階が値の欠落で打ち切られた場合、それ以前に現れた `-h`/`-V` は有効、以降は存在しない扱い。
2. `lexError` があれば、それを誤りとする。
3. 「知らないオプション」が `tokens` にあれば、最初のものを `unknown option`(対象はその綴り: `-x`, `--bogus`)とする。
4. 位置引数が 0 個: `missing argument`(対象なし)。例: 引数なし、`-v` だけ、`--` だけ。
5. 先頭の位置引数(コマンド名)が非 UTF-8 なら `invalid value`(対象は、可能な限り読める形に変換した文字列)。8 コマンド(v0.8.0 以前は 6 コマンド)のいずれでもなければ `unknown command`(対象はその語。空文字も同じ)。
6. そのコマンドで使えないオプション: `option not allowed for this command`。検査の順は `-v`, `--config`, `--to`, `--run-id`, `--all`, `--remove-on-complete`, `--dry-run`, `--force`(v0.9.0 以降。ここで `--force` が 2 回以上なら `invalid value`、対象 `--force`)、`--assume`(`--dry-run` なしの指定。同)、(`--dry-run` が 2 回以上なら `invalid value`、対象 `--dry-run`)、`--format`、`--state`(後ろの 2 つは `graph` 以外で不可)。対象はそのオプション名(`-v` の綴りは `-v`)。可否は 3.1 の表による。`--run-id` は、`graph` では、ここでは許可し、`--state` なしの `--run-id` を、下の 8a の位置で拒否する。
7. 重複: `--format`, `--state`(v0.9.0 以降)、`-v`, `--config`, `--to`, `--run-id`, `--all`, `--remove-on-complete` のどれかが 2 回以上 → `invalid value`(対象はそのオプション名。`-vv` は対象 `-v`)。検査の順は挙げた順。`-v` の重複は、`-v` と `--verbose` の合計で数える。
8. `--all` と `--run-id` の併用 → `invalid value`(対象 `--all`)。
   8a. (v0.9.0 以降)`graph` で `--run-id` があり `--state` がない → `option not allowed for this command`(対象 `--run-id`)。
   8b. (v0.9.0 以降)値の検査。`--assume` の各値(指定順に): `=` がない、終了コードが整数でない、0〜255 の範囲外、同じジョブID の重複 → `invalid value`(対象 `--assume`。値は含めない)。`--format` の値が `text`・`mermaid` 以外 → `invalid value`(対象 `--format`)。ジョブID が定義にあるかは、ここでは調べない(定義を読んだあとに、流れが調べる。7.9)。
9. `--run-id` の値の検査(5.2)。値が非 UTF-8、または空でなく規則に違反 → `invalid value`(対象 `--run-id`。値は含めない)。空文字は有効。
10. 位置引数の過不足(コマンド名を除いて数える):
    - `run`・`pipeline`・`convert` で位置引数が 0 個 → `missing argument`(対象なし)。
    - `watch`・`reset` で位置引数が 1 個以上 → `unexpected argument`(対象は最初の位置引数)。
    - コマンドが取れる個数を超える(2 個目の位置引数がある)→ `unexpected argument`(対象は 2 個目の語。`run a b` なら `b`)。
    - (v0.9.0 以降)`validate`・`graph`・`init` の位置引数(パイプライン名)は省略できる。`init` の名前が trim して空なら `invalid value`(対象 `pipeline name`。値は含めない)。
    - 位置引数(ジョブID、パイプライン名)が非 UTF-8 → `invalid value`(対象は可能な限り読める形に変換した文字列)。`convert` の入力ファイルは、非 UTF-8 でもパスとしてそのまま通す。
11. `convert`: `--to` なし → `missing argument`(対象 `--to`)。`--to` の値が `yaml`/`toml` 以外(非 UTF-8 を含む)→ `invalid value`(対象は値)。

誤りの文言は [08-output-and-i18n.md](08-output-and-i18n.md) 4.2、表示形式は「`conductor: error: <文>: '<対象>'`(対象があるとき)、空行、使い方全文」。対象は、制御文字をエスケープし、64 文字で切る(08 章 9 章)。

### 4.3 実装に任せてよい点

- 字句の段階の実現方法(自前の走査でもライブラリでもよい)。ただし 4.1 の挙動(値を取るオプションが次の引数を無条件に値にする、`=` の扱い、`--`、連結)は必須。
- 内部の語・型の分け方。
- 非 UTF-8 の引数を表現できない言語は、その引数を `invalid value` 扱いにしてよい(対象の文字列は、置換文字などで読める形にする)。`--config` や `convert` の入力のパスは、表現できる限り無加工で通す(できない言語は未確認の領域で、この仕様は規定しない)。
- 単独の `-` を位置引数とする挙動は、実機で確認済み(`run -` はジョブID `-` として扱われる)。

## 5. run ID

### 5.1 決定の優先順位(`run`・`pipeline`)

1. `--run-id <値>` が指定されていれば、環境変数は見ない。値が空文字なら run ID なし。
2. 指定がなければ、環境変数を読む。環境変数の名前は、定義の `[pipeline].run_id_env`(なければ `CONDUCTOR_RUN_ID`)。未定義または空文字なら run ID なし。値が文字として解釈できない(UTF-8 でない)ときは、未定義と同じ扱い。
3. 得た値は 5.2 の規則で検査する。違反は `invalid value`(終了コード 68)。対象は、オプション由来なら `--run-id`、環境変数由来なら `run ID (environment variable)`(値は含めない)。

`run`・`pipeline` では、環境変数の名前を知るために、定義を先に読む。検査の順序は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md) 2 章を参照。

### 5.2 run ID の規則

- 文字は `A-Za-z0-9._-` だけ、長さは 1〜64 文字、先頭が `.` でない(`..` も不可)。空文字は「run ID なし」を表し、値としては規則を満たさない(オプションの空文字だけが「なし」を意味する特別な扱い)。
- ステートファイル名は、run ID なしで `<定義ファイル名>.state`、ありで `<定義ファイル名>.<run-id>.state`(定義ファイルと同じディレクトリ)。詳細は [06-state-file.md](06-state-file.md)。
- 定義の `run_id_env` は、`A-Za-z_` で始まり `A-Za-z0-9_` が続く 1 文字以上。違反は定義の誤り(64。[03-definition-format.md](03-definition-format.md))。

### 5.3 `watch` / `reset` の run ID

- `--run-id` があれば、定義ファイルを読まない(環境変数は見ない)。
- なければ、定義から `[pipeline].run_id_env` だけをゆるく読む。ファイルが読めない・TOML として読めない・キーがない・文字列でない・名前の形が 5.2 の規則と違う、のどれでも、既定名 `CONDUCTOR_RUN_ID` を使う。ほかの誤りは無視するので、壊れた定義でも動く。得た名前の環境変数を見る。
- 環境変数の値が規則違反なら `invalid value`(68)。
- `reset --all` は、環境変数も `--run-id` も見ない。

## 6. 環境変数

| 名前 | 読む場所 | 効果 |
|------|----------|------|
| `CONDUCTOR_RUN_ID` | `run`, `pipeline`, `watch`, `reset`(`--run-id` なし、かつ定義の `run_id_env` なし) | run ID の既定の供給源(5 章) |
| `run_id_env` が指す名前(定義で指定) | 同上 | `CONDUCTOR_RUN_ID` の代わりに読む環境変数名 |
| `LC_ALL`, `LC_MESSAGES`, `LANG` | 表示が必要になった最初の時点 | この順に、最初の空でない値が `ja` で始まれば日本語、それ以外は英語([08-output-and-i18n.md](08-output-and-i18n.md) 7 章) |
| `NO_COLOR` | `watch` の表を出すとき | 未設定または空なら色を使える。空でない値が設定されていれば色なし(08 章 6.3) |

## 7. 各コマンドの動作

ここでは入口の検査の順序と、コマンドごとの外から見える挙動を規定する。内部の流れの詳細は [05-execution-flow.md](05-execution-flow.md)。

### 7.1 `run <job-id>`

1. シグナルの監視を開始する(定義の検査より前)。失敗は終了コード 70([07-process-and-signals.md](07-process-and-signals.md))。
2. 定義の読み込みと検証(失敗は 64)。全ジョブの `command` を検査(失敗は 64)。
3. ジョブIDが定義にあるか(なければ `job is not defined in the definition file: '<ID>'`、68)。
4. run ID の決定(5 章。違反は 68)。
4a. (v0.9.0 以降)`[pipeline].env_file` と、そのジョブの `env_file` の `.env` をすべて読む(誤りは 64。ステートに触れない。05 の 14 章)。使い方の誤り(3、4 の 68)と `.env` の誤り(64)が同時にあれば、先に検出する 68 が勝つ。
5. ステートの読み込み(65)と、定義との照合(65)。
6. 起動前の判断: 実行 / スキップ(0)/ 拒否(66)。
7. 実行なら、ステートが空のとき初期ステートを書き(`Fast` の耐久性)、ジョブをリトライ込みで(試行ごとに環境変数を作り直して。v0.9.0 以降)実行し、結果を分類してステートを 1 回書き(`Durable` の耐久性)、完了かつ削除指定ならステートを消す。終了コードはジョブの結果(10 章)。
8. スキップなら、記録が変わるときだけ書く。完了し削除指定なら消す。0 で終わる。

`-v` のとき、結果行・遷移行・中断行を標準エラーに出す(08 章)。

### 7.2 `pipeline <pipeline-name>`

`run` と同じ検査(パイプライン名は定義の `name` と照合。不一致は 64)を行い(`.env` は、`[pipeline].env_file` と**すべてのジョブ**の `env_file` を、最初のジョブを起動する前に読む。v0.9.0 以降)、定義の記述順にジョブを 1 つずつ、起動前の判断で処理する。

- 実行 → 結果を記録。エラー(または分類なし=起動前の中断)で停止し、そのジョブの終了コードで終わる。自動の再開・リトライはしない(`[job.retry]` を除く)。
- スキップ → 記録して続ける。
- 拒否 → 66 で終わる。
- 完了(最後のジョブ、`@end`、スキップによる完了)したら、完了のステートを残して 0 で終わる。`--remove-on-complete` または定義の `remove_on_complete = true` のときだけステートを消す(消せなければ 65)。
- ジョブのあいだに中断の合図が立っていれば、次のジョブを起動せず、再開位置を記録して `128 + 番号` で終わる(`-v` のとき `[conductor] interrupted by signal N`)。
- 記述順のジョブを使い切っても完了していない(通常は起こらない)と、`cannot determine whether the pipeline is complete` を出して 65。

### 7.3 `watch`

定義ファイルを検証しない。`--run-id`/環境変数で run ID を決め(5.3)、ステートを、完了のものも含めて読み(読み込みの検査は [06-state-file.md](06-state-file.md))、表を標準出力に 1 回出して 0 で終わる。ファイルなし・長さ 0 のときだけ `no state (nothing has run yet)`。読めない・壊れているは 65(`reset` の案内は付かない文脈。08 章 4.5)。標準出力への書き込み失敗は `cannot write to standard output` を標準エラーに出して 65。表の形式は 08 章 6 章。シグナルの準備はしない。

### 7.4 `reset`

ステートを読まず、中身の検査もしない(壊れていても動く)。ステートがなくても成功(0)。

- `--run-id` なし(かつ `--all` なし): 5.3 の規則で run ID を決め、そのステートファイルを消す。
- `--run-id <ID>`: その run ID のファイルだけを消す。
- `--all`: その定義のステートを、run ID の有無によらず全部消す。`--run-id` との併用は 68。環境変数は見ない。読めなくて残したファイルがあれば、標準エラーに警告を出す(終了コードは 0。08 章 4.10)。消すのに失敗したら 65。消すものがなければ何も出さず 0。

`--all` で消す対象(定義ファイル名を `<定義ファイル名>` とする):

- `<定義ファイル名>.state`(run ID なし): 中身を読まず、通常のファイルなら消す。シンボリックリンクは消さない。
- `<定義ファイル名>.<run-id>.state`: 名前の `<run-id>` が 5.2 の規則に合い、かつ、中身の `run_id` の値が名前の run ID と一致するものだけ消す。中身が読めないものは消さずに残し、警告に挙げる。
- 一時ファイル、別の定義のステート、ディレクトリ、シンボリックリンクは消さない。

(この細かい規則は、利用者向けの説明を根拠にした記述で、実装の細部は未確認。ステートファイルの形式は [06-state-file.md](06-state-file.md)。)

### 7.5 `validate [<pipeline-name>]`

定義を読んで検証し、全ジョブの `command` を検査するだけ。ジョブを起動せず、シグナルの準備もせず、ステートに触れない(読まず・書かず・消さない)。壊れたステートがあっても成功する。

- 成功: 何も出さず 0。`-v` のとき、標準エラーに `[conductor] definition is valid (<N> job|jobs)`(英語。`N == 1` のとき `job`、それ以外は `jobs`)/ `[conductor] 定義は正しいです(ジョブ <N> 個)`。
- 失敗: `run` と同じ表示で 64。
- 名前を渡すと定義の名前と照合する(不一致は 64)。省略すると照合しない。`--run-id` は 68(使えないオプション)。
- (v0.9.0 以降)定義と `command` の検査が通ったあと、`pipeline` と同じ範囲の `.env`(`[pipeline].env_file` とすべてのジョブの `env_file`)も読んで検査する。誤りは `run` と同じ表示で 64。重複キーの警告(`conductor: warning: <ファイル>:<行>: duplicate key ...`)は標準エラーに出すが、誤りではないので、警告だけなら 0(`-v` の 1 行は、警告のあとに出る)。`pipeline`・`run` は、警告を表示しない。`graph` も、同じ検査を共有する。

### 7.6 `convert <file> --to <yaml|toml>`

入力ファイルを、入力形式を `--to` の反対(`--to toml` なら入力は YAML、`--to yaml` なら入力は TOML)とみなして変換する(拡張子は見ない)。結果を全部作ってから標準出力に 1 回で書く。失敗したときは標準出力に何も書かず、標準エラーに原因を出して 67。標準出力に書けないときも 67(`cannot write to standard output`)。シグナルの準備はしない。変換の規則そのもの(コメントとキー順は保持しない、アンカーの展開、入力 1 MiB まで、など)は [09-converter.md](09-converter.md)。

### 7.7 `graph [<pipeline-name>]`(v0.9.0 以降)

定義の遷移(成功と警告の行き先、`@end`)を図にして、標準出力に 1 回で書く。ジョブを起動せず、シグナルの準備もせず、ステートを書かない・消さない(`--state` のときだけ読む)。

順序(誤りの終了コードを括弧に示す):

1. `validate` と同じ検査: 定義の読み込みと検証(64)→ 全ジョブの `command` の検査(64)→ `pipeline` と同じ範囲の `.env` の読み取り(64)。名前を渡すと定義の名前と照合する(不一致は 64)。`.env` の重複キーの警告は標準エラーに出す(誤りではない)。`.env` の値は、図にも出力にも含めない(名前も出さない)。
2. `--state` のときだけ: run ID の決定(`--run-id`、なければ定義の `run_id_env` の環境変数。違反は 68)→ ステートを、**完了したものも含めて**読む(読めない・壊れている・版の不一致は 65。ファイルなし・長さ 0 は、ステートなし)→ 定義との照合(食い違いは 65)。ステートがなければ、すべてのジョブが未実行(`[  ]`)。ステートがあれば、状態の重ね合わせを作る(05 の 15 章)。
3. 図の構造を作り、`--format text`(既定)なら配置を決めて text に、`--format mermaid` なら Mermaid に、文字列を組み立てる。`-v` は text にだけ効く(Mermaid には影響しない)。
4. 組み立て終えた文字列を、標準出力に 1 回で書く。書けなければ(閉じたパイプなど)`cannot write to standard output` を標準エラーに出して 65(`watch` と同じ)。

終了コード: 成功 0、定義・`command`・`.env` の誤り 64、ステートの誤りと標準出力に書けないとき 65、使い方の誤り 68(`--format` の値の誤り、`--format`・`--state` の重複、`graph` 以外での `--format`・`--state`、`--state` なしの `--run-id`、`-v` の重複など)。

出力の形(text のレーンの図、辺の一覧、Mermaid、`--state`、`-v`)は 08 の 11 章に全例を載せ、図の構造と配置の規則は 05 の 15 章。

### 7.8 `init [<pipeline-name>]`(v0.9.0 以降)

定義のひな形を作る。定義、`.env`、ステートを読まず、ジョブを実行せず、シグナルの準備もしない。引数の誤り(68)は、ファイルに触れる前に返す。

1. オプション: 出力先 = `--config` の値(省略時 `conductor.toml`。値が `-` だけなら標準出力)、`force` = `--force`、名前 = 位置引数(省略時 `my-pipeline`)。
2. ひな形の文字列を作る(言語はロケール。08 の 12 章)。
3. 出力先が標準出力(`--config -`): ひな形だけを、標準出力に 1 回で書く(成功のメッセージは出さない。`--force` は無視する)。書けなければ `cannot write the template to standard output` を標準エラーに出して 65。成功は 0。
4. 出力先がファイル: ファイルを作る(06 の 14 章に、作成の規則)。
   - 新規に作った → 標準エラーに `created <path>`(日本語は `<path> を作成しました`)、0。
   - `--force` で、既にあったファイルを置き換えた → `replaced <path>`(日本語は `<path> を置き換えました`)、0。
   - 既にある(`--force` なし。何も変えない)→ `conductor: error: <path> already exists (use --force to overwrite)`、**69**。
   - そのほか(親のディレクトリが無い、作る先がディレクトリ、権限がない、そのほかの入出力の誤り)→ `conductor: error: cannot create <path>: <理由>`、65。表示は種類ごとの固定の文とパスだけで、OS のエラーの文を含めない。
5. パスの表示は、制御文字をエスケープする(切らない)。成功のメッセージは、標準エラー(常に出る)に出す。標準出力に出るのは、`--config -` のひな形だけ。

終了コード: 作れた・標準出力に出せた 0、既にある 69、作れない(親のディレクトリが無い、書く権限がない、作る先がディレクトリ、そのほかの入出力の誤り)と標準出力に書けない 65、使い方の誤り 68(名前が空・空白だけ、位置引数が 2 つ以上、`--config` と `--force` 以外のオプション、`init` 以外への `--force`)。

### 7.9 `run`・`pipeline` の `--dry-run`(v0.9.0 以降)

ジョブを実行せずに、実行計画を標準出力に出す。ジョブを起動せず、ステートに書かず・消さず(読むだけ)、シグナルの準備もしない。`--remove-on-complete` は、何も消さないので効果がない。ジョブID(`run`)・パイプライン名(`pipeline`)を取り出したあとに、通常の準備の代わりにこの流れに入る。

順序(誤りの終了コードを括弧に示す):

1. 準備(通常の準備から、シグナルの準備を除いたもの): 定義の読み込みと検証(64。`pipeline` は名前の照合)→ 全ジョブの `command` の検査(64)→ `run` の対象のジョブが定義にあるか(68)→ **`--assume` のジョブIDが定義にあるか**(なければ 68。誤りの表示は `job is not defined in the definition file` に固定で、ジョブIDを含めない)→ run ID の決定(68)→ `.env` の読み取り(64。`pipeline` は全ジョブ分、`run` は対象のジョブ分。**本番と同じ範囲**)→ ステートの読み込み(65。完了したステートは「ステートなし」)→ 定義との照合(65)。
2. 各ジョブの `command` を分割する(定義の検査で済んでいるので、誤りは起きない。起きれば本番と同じ表示で 64)。
3. 計画を作る(05 の 16 章)。**本番なら順序外として拒否される**ときは、本番と同じ表示(08 の 4.7)で 66。
4. 計画の文字列を組み立て、標準出力に 1 回で書く。書けなければ `cannot write to standard output` を標準エラーに出して 65。

終了コード: 計画を出せたら 0(模擬の結果がエラーでの停止でも 0。実際の終了コードは示さない)、定義と `.env` の誤り 64、ステートの誤り・標準出力に書けない 65、拒否 66、使い方の誤り 68(`--assume` の書式、範囲外、重複、定義にないジョブID、`--dry-run` なしの `--assume`、`run`・`pipeline` 以外での `--dry-run`、`--dry-run` の重複)。`--assume` の誤りの表示に、入力の値を含めない(対象は `--assume`)。

出力の形は 08 の 11 章、模擬の規則は 05 の 16 章。

## 8. ヘルプとバージョン

- `-h`/`--help`: 使い方全文を標準出力に出して 0。言語はロケールに従う(全文は 08 章 8 章)。
- `-V`/`--version`: 標準出力に `conductor <版>` と改行 1 つ(v0.9.2 では `conductor 0.9.2`)。ロケールを判定しない(常にこの英語の形)。終了コード 0。

```text
$ conductor --version
conductor 0.9.2
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

$ conductor convert conductor.toml
conductor: error: missing argument: '--to'
[68]

$ conductor convert conductor.toml --to json
conductor: error: invalid value: 'json'
[68]

$ conductor run nojob          # 定義にないジョブID
conductor: error: job is not defined in the definition file: 'nojob'
[68]

$ CONDUCTOR_RUN_ID='bad id' conductor watch
conductor: error: invalid value: 'run ID (environment variable)'
[68]

$ conductor -vh        # help が勝つ。標準出力に使い方、0

$ conductor init -v                  # v0.9.0 以降。init は -v を取らない
conductor: error: option not allowed for this command: '-v'
[68]

$ conductor pipeline p --assume a=1  # --dry-run なしの --assume
conductor: error: option not allowed for this command: '--assume'
[68]

$ conductor pipeline p --dry-run --assume a=300
conductor: error: invalid value: '--assume'
[68]

$ conductor graph --run-id x         # --state なしの --run-id
conductor: error: option not allowed for this command: '--run-id'
[68]

$ conductor init                      # conductor.toml が既にある
conductor: error: conductor.toml already exists (use --force to overwrite)
[69]
```

## 10. 判定の早見表(確認済みの組み合わせ)

| 引数 | 結果 |
|------|------|
| (引数なし)、`--`、`-v` | `missing argument`(68) |
| `-- run`、`run --` | `missing argument`(68)(`--` のあとの `run` はコマンド名。ジョブIDがない) |
| `-- run a` | `run` コマンドで、ジョブID `a` |
| `--config` | `option requires a value: '--config'`(68) |
| `run a --config` | `option requires a value: '--config'`(68) |
| `--config --help run a` | `--help` というパスを定義ファイルとして読む(ファイルがなければ 64) |
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
| `pipeline p --assume a=1`(v0.9.0 以降) | `option not allowed for this command: '--assume'`(68)(`--dry-run` なし) |
| `pipeline p --dry-run --assume a=300`、`--assume a`、`--assume a=x`、`--assume a=1 --assume a=2` | `invalid value: '--assume'`(68)(範囲外、`=` なし、整数でない、重複) |
| `graph --run-id x`、`graph --state --run-id x` | 前者は `option not allowed for this command: '--run-id'`(68)。後者は文法として正しい |
| `graph --format json`、`graph --format text --format mermaid`、`watch --state` | `invalid value: '--format'`(68)、`invalid value: '--format'`(68)、`option not allowed for this command: '--state'`(68) |
| `init -v`、`init --run-id x`、`init a b`、`validate --force`、`init ''` | `option not allowed for this command: '-v'`、`'--run-id'`(68)、`unexpected argument: 'b'`(68)、`option not allowed for this command: '--force'`(68)、`invalid value: 'pipeline name'`(68) |
