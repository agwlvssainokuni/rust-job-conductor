# 04. コマンドライン(`conductor` v0.9.1)

実装: `crates/cli/src/{main,lib,args,types,ctx,run,pipeline,aux_commands,graph,dry_run,init}.rs`。
引数の解釈は `lexopt` で字句に分け、コマンドごとの文法の表で検査する。引数はシェルを介さず、解釈も展開もしない。

## 1. 全体像

- 実行ファイル名は `conductor`。プログラム名を除く引数を `std::env::args_os()` で受け取る。
- 入口 `main` は、(1) パニックのフックを設定し、(2) `run_cli` の返した終了コードで `std::process::exit` を 1 回だけ呼ぶ。
- サブコマンドは 8 つだけ: `run`, `pipeline`, `watch`, `reset`, `validate`, `convert`, `graph`, `init`(後ろの 2 つは v0.9.0 以降)。v0.8.0 以前は 6 つ。
- 特別なオプション(`-h/--help`, `-V/--version`)は、コマンドの解釈より先に処理する。
- 使い方の誤りは、標準エラーに「1 行の誤り + 空行 + 使い方全文」を書いて終了コード 68(10 章の表を参照)。ジョブは実行しない。
- 既定では、`conductor` 自身は何も出力しない(ジョブ本体の出力がそのまま流れる)。エラーのとき、または `-v` のときだけ標準エラーに書く(`watch`・`--help`・`--version`・`convert` は標準出力に結果を書く。v0.9.0 以降は、`graph` の図、`--dry-run` の計画、`init --config -` のひな形も標準出力。`init` が作った・置き換えたの知らせは標準エラー)。

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

## 3. オプションの一覧と、コマンドごとの可否

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
| `--help` | `-h` | なし | どこでも | | | | | | | |
| `--version` | `-V` | なし | どこでも | | | | | | | |

位置引数:

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

コマンド名は位置引数の先頭(最初に現れた位置引数)である。コマンド名の前後どちらにもオプションを置ける(例: `conductor --config c.toml -v run x` は有効)。

各オプションの意味と既定値:

| オプション | 意味 | 既定 |
|------------|------|------|
| `-v`, `--verbose` | ジョブの結果・スキップ・遷移・中断を標準エラーに出す(`validate` は成功時 1 行)。`graph` では、ジョブごとの詳細(コマンド、リトライ、飛び越し)を標準出力の図に足す。1 段階のみ | 出さない |
| `--config <path>` | 定義ファイルのパス。`OsString` のまま保つ(UTF-8 でないパスも可)。`init` では、作る先のパス(`-` だけの値は、標準出力を意味する) | 実行ディレクトリの `conductor.toml` |
| `--run-id <id>` | ステートファイルを分ける run ID。空文字は「run ID なしに固定」 | 未指定(環境変数を見る。5 章) |
| `--remove-on-complete` | 完了時にステートファイルを削除する | 指定なし(定義の `[pipeline].remove_on_complete`、さらに既定は `false`) |
| `--all` | 定義のステートを、run ID の有無によらず全部消す(`reset` のみ) | なし |
| `--to <yaml\|toml>` | `convert` の出力形式。大文字小文字を区別し、`yaml` と `toml` 以外は `invalid value`(対象は値そのもの) | なし(必須) |
| `--format <text\|mermaid>`(v0.9.0 以降) | `graph` の出力形式。大文字小文字を区別し、それ以外は `invalid value`(対象 `--format`。値は含めない) | `text` |
| `--state`(同) | `graph` で、ステートを読み、状態を図に重ねる(読むだけ) | 重ねない |
| `--dry-run`(同) | `run`・`pipeline` で、ジョブを実行せず、ステートに書かず、シグナルも準備せずに、実行計画を標準出力に出す | 実行する |
| `--assume <ジョブID>=<終了コード>`(同) | 計画の中のジョブの終了コード(0〜255)を仮定する。最初の `=` で分ける。複数回指定できる(指定順を保つ) | 指定のないジョブは 0 |
| `--force`(同) | `init` で、既にあるファイルを上書きする(`--config -` のときは無視) | 上書きしない |

## 4. 引数の字句規則

| 規則 | 挙動 |
|------|------|
| `--opt value` と `--opt=value` | `--config`, `--run-id`, `--to`, `--format`, `--assume` の両方の形を受け付ける(`--assume=a=1` の値は `a=1`) |
| 値の取り方 | 値を取るオプションは、次の引数を、先頭が `-` でもそのまま値にする(例: `--config --help` は `--help` というパスを指す)。次の引数がなければ `option requires a value`(`MissingValue`。対象は `--config` など) |
| 空の値 | `--config=`(空パス)は文字列として受理され、後段でファイルなしの誤りになる。`--run-id=` は有効(run ID なしに固定) |
| 短縮形の連結 | lexopt の標準どおり。`-vh` は `-v` と `-h`(`-h` が勝つ)。`-vv` は `-v` が 2 回 |
| `--` | オプションの終わり。以降はすべて位置引数(`-v` や `--help` も位置引数)。例: `run -- -weird` はジョブID `-weird`。`-- --help` は先頭の位置引数 `--help` がコマンド名として未知になり `unknown command` |
| 値を取らないオプションに `=値` | `--verbose=x`, `--all=x`, `--remove-on-complete=x`, `--state=x`, `--dry-run=x`, `--force=x` は `invalid value`(対象はオプション名)。文法にない `--bogus=x` は `unknown option`(対象 `--bogus`) |
| 非 UTF-8 | 位置引数(コマンド名、ジョブID、パイプライン名、`--to` の値)が UTF-8 でないと `invalid value`(対象は lossy 変換した文字列)。`--config` と convert の入力パスは `OsString` のまま通す。`--run-id` の値が非 UTF-8 だと `invalid value`(対象 `--run-id`) |
| `-h`/`-V` に `=値` | `--help=x`、`--version=x`、`-h=x`、`-V=1`、`-vh=x` のように `=値` が付いていても、help / version が優先され、誤りにならない(`-v=1` は `invalid value`、対象 `-v`) |
| 単独の `-` | 位置引数。`run -` はジョブID `-` として扱われる |
| `--` の直後のコマンド名 | `-- run` は、`run` がコマンド名(先頭の位置引数)になり、ジョブIDがないので `missing argument`(68)。`-- run a` はジョブID `a` の `run` |
| 値として取られた `-h`/`--help` | `--config --help run a` は `--help` をパスとして扱うので、help にならない(ファイルがなければ 64)。`--config --help`(ほかの引数なし)は位置引数が 0 個で `missing argument`(68)。`--help --config` は `--help` が先に解釈されるので、値の欠落より前に help(0) |
| 解釈の打ち切り | 値の欠落(`--config` 等が最後の引数)で字句の走査は止まる。それ以前に `-h`/`-V` があれば、そちらが先に処理される |

### 4.1 解釈の順序(誤りの優先順位)

1. `-h/--help` または `-V/--version` が(`--` より前に、オプションとして解釈されて)1 つでもあれば(値を取るオプションの直後の引数は、値として取られるので数えない)、ほかの誤りに関係なく、先に現れた方を処理して終了コード 0。両方あるとき、先に現れた方が勝つ(`-V -h` は version、`-h -V` は help)。
2. 字句の誤り(値の欠落、`--verbose=x` のような値の取り残し)。
3. 文法にないオプション(`unknown option`。複数あれば最初のもの)。
4. 位置引数が 0 個: `missing argument`(対象なし)。例: 引数なし、`-v` だけ。
5. 先頭の位置引数が 8 コマンド(v0.8.0 以前は 6 コマンド)のいずれでもない: `unknown command`(対象はその語。空文字も同じ)。
6. そのコマンドで使えないオプション: `option not allowed for this command`。検査の順は `-v`, `--config`, `--to`, `--run-id`, `--all`, `--remove-on-complete`, `--dry-run`, `--force`(v0.9.0 以降。ここで `--force` が 2 回以上なら `invalid value`、対象 `--force`)、`--assume`(`--dry-run` なしの指定。同)、(`--dry-run` が 2 回以上なら `invalid value`、対象 `--dry-run`)、`--format`、`--state`(後ろの 2 つは `graph` 以外で不可)。`--run-id` は、`graph` では、ここでは許可し、`--state` なしの `--run-id` を、下の 8a の位置で拒否する。
7. 重複: `--format`, `--state`(v0.9.0 以降)、`-v`, `--config`, `--to`, `--run-id`, `--all`, `--remove-on-complete` のどれかが 2 回以上 → `invalid value`(対象はそのオプション名。`-vv` は対象 `-v`)。
8. `--all` と `--run-id` の併用 → `invalid value`(対象 `--all`)。
8a. (v0.9.0 以降)`graph` で `--run-id` があり `--state` がない → `option not allowed for this command`(対象 `--run-id`)。
8b. (v0.9.0 以降)値の検査。`--assume` の各値(指定順に): `=` がない、終了コードが整数でない、0〜255 の範囲外、同じジョブID の重複 → `invalid value`(対象 `--assume`。値は含めない)。`--format` の値が `text`・`mermaid` 以外 → `invalid value`(対象 `--format`)。ジョブID が定義にあるかは、ここでは調べない(定義を読んだあとに、流れが調べる。7.9)。
9. `--run-id` の値の検査(下の 5.2)。違反は `invalid value`(対象 `--run-id`。値は含めない)。
10. 位置引数の過不足: `run`/`pipeline`/`convert` で入力なし → `missing argument`。`watch`/`reset` でコマンド名の後ろに位置引数あり → `unexpected argument`(対象はその語)。それ以外のコマンドで、コマンド名を除く 2 個目の位置引数があれば `unexpected argument`(対象は 2 個目の語。`run a b` なら `b`)。`validate`・`graph`・`init` の位置引数(パイプライン名)は省略できる。`init` の名前が `trim` して空なら `invalid value`(対象 `pipeline name`。値は含めない)。
11. `convert`: `--to` なし → `missing argument`(対象 `--to`)。`--to` の値が `yaml`/`toml` 以外 → `invalid value`(対象は値)。

使い方の誤りの種類と文言は 08 章のカタログ(`usage_kind`)を参照。表示形式は「`conductor: error: <文>: '<対象>'`(対象があるとき)、空行、使い方全文」。

## 5. run ID

### 5.1 決定の優先順位(`run`・`pipeline`)

1. `--run-id <値>` が指定されていれば、環境変数は見ない。値が空文字なら run ID なし。
2. 指定がなければ、環境変数を読む。環境変数の名前は、定義の `[pipeline].run_id_env`(なければ `CONDUCTOR_RUN_ID`)。未定義または空文字なら run ID なし。UTF-8 でない値は未定義扱い。
3. 得た値は `RunId` の規則で検査する。違反は `invalid value`(終了コード 68)。対象は、オプション由来なら `--run-id`、環境変数由来なら `run ID (environment variable)`(値は含めない)。

`graph --state` も、同じ規則で run ID を決める(`--run-id` があればそれ、なければ定義の `run_id_env` の環境変数)。`run`・`pipeline` では、環境変数の名前を知るために定義を先に読む。検査の順序は 10 章の「終了コードの決定順序」を参照。

### 5.2 `RunId` の規則

- 文字は `A-Za-z0-9._-`、1〜64 文字、先頭が `.` でない(`..` も不可)。
- ステートファイル名は run ID なしで `<定義ファイル名>.state`、ありで `<定義ファイル名>.<run-id>.state`(定義ファイルと同じディレクトリ)。
- 定義の `run_id_env` は `A-Za-z_` で始まり `A-Za-z0-9_` が続く 1 文字以上。違反は定義の誤り(64)。

### 5.3 `watch` / `reset` の run ID

- `--run-id` があれば、定義ファイルを読まない(環境変数は見ない)。
- なければ、定義から `[pipeline].run_id_env` だけをゆるく読み(ファイルが読めない・TOML として読めない・キーがない・文字列でない・名前の形が違う、のどれでも既定名 `CONDUCTOR_RUN_ID`)、その環境変数を見る。ほかの誤りは無視するので、壊れた定義でも動く。
- 環境変数の値が規則違反なら `invalid value`(68)。
- `reset --all` は環境変数も `--run-id` も見ない。

## 6. 環境変数

| 名前 | 読む場所 | 効果 |
|------|----------|------|
| `CONDUCTOR_RUN_ID` | `run`, `pipeline`, `watch`, `reset`(`--run-id` なし、かつ定義の `run_id_env` なし) | run ID の既定の供給源(5 章) |
| `run_id_env` が指す名前(定義で指定) | 同上 | `CONDUCTOR_RUN_ID` の代わりに読む環境変数名 |
| `LC_ALL`, `LC_MESSAGES`, `LANG` | 表示が必要になった最初の時点 | この順に最初の空でない値が `ja` で始まれば日本語、それ以外は英語(08 章) |
| `NO_COLOR` | `watch` の表を出すとき | 未設定または空なら色を使える。空でない値が設定されていれば色なし(08 章。ほかに、UNIX 系で、標準出力が端末のときだけ色を使う) |

## 7. 各コマンドの動作

### 7.1 `run <job-id>`

1. シグナルの監視を開始(定義の検査より前)。失敗は 70。
2. 定義の読み込みと検証(失敗は 64)。全ジョブの `command` を検査(失敗は 64)。
3. ジョブIDが定義にあるか(なければ `job is not defined in the definition file: '<ID>'`、68)。
4. run ID の決定(68)。
4a. (v0.9.0 以降)`[pipeline].env_file` と、そのジョブの `env_file` の `.env` をすべて読む(誤りは 64。ステートに触れない。05 の 14 章)。使い方の誤り(3、4 の 68)と `.env` の誤り(64)が同時にあれば、先に検出する 68 が勝つ。
5. ステートの読み込み(65)と、定義との照合(65)。
6. 起動前の判断: 実行 / スキップ(0)/ 拒否(66)。
7. 実行なら、ステートが空のとき初期ステートを書き(`Fast`)、ジョブをリトライ込みで(試行ごとに環境変数を作り直して。v0.9.0 以降)実行し、結果を分類してステートを 1 回書き(`Durable`)、完了かつ削除指定ならステートを消す。終了コードはジョブの結果(10 章)。
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

定義ファイルを検証しない。`--run-id`/環境変数で run ID を決め(5.3)、ステートを完了のものも含めて読み(`read_state_raw`)、表を標準出力に 1 回出して 0 で終わる。ファイルなし・長さ 0 のときだけ `no state (nothing has run yet)`。読めない・壊れているは 65(`reset` の案内は付かない文脈。08 章)。標準出力への書き込み失敗は `cannot write to standard output` を標準エラーに出して 65。表の形式は 08 章。

### 7.4 `reset`

ステートを読まず、検査もしない(壊れていても動く)。ステートがなくても成功(0)。

- `--run-id` なし(かつ `--all` なし): 5.3 の規則で run ID を決め、そのステートファイルを消す。
- `--run-id <ID>`: その run ID のファイルだけを消す。
- `--all`: その定義のステートを run ID の有無によらず全部消す。`--run-id` との併用は 68。環境変数は見ない。読めなくて残したファイルがあれば、標準エラーに警告を出す(終了コードは 0。08 章)。消すのに失敗したら 65。消すものがなければ何も出さず 0。

`--all` で消すものの詳細(定義の README の規則と、`conductor-runtime` の実装): `<定義ファイル名>.state`(run ID なし。中身を読まず通常ファイルなら消し、シンボリックリンクは消さない)、`<定義ファイル名>.<run-id>.state`(名前の `<run-id>` が規則に合い、中身の `run_id` が一致するものだけ)。一時ファイル、別の定義のステート、ディレクトリ、シンボリックリンクは消さない。(実装の詳細は runtime クレート側にあり、この文書では README の記述を根拠とした。未確認。)

### 7.5 `validate [<pipeline-name>]`

定義を読んで検証し、全ジョブの `command` を検査するだけ。ジョブを起動せず、シグナルの準備もせず、ステートに触れない(読まず・書かず・消さない)。壊れたステートがあっても成功する。

- 成功: 何も出さず 0。`-v` のとき、標準エラーに `[conductor] definition is valid (<N> job|jobs)`(英語。`N == 1` のとき `job`)/ `[conductor] 定義は正しいです(ジョブ <N> 個)`。
- 失敗: `run` と同じ表示で 64。
- 名前を渡すと定義の名前と照合する(不一致は 64)。省略すると照合しない。`--run-id` は 68。
- (v0.9.0 以降)定義と `command` の検査が通ったあと、`pipeline` と同じ範囲の `.env`(`[pipeline].env_file` とすべてのジョブの `env_file`)も読んで検査する。誤りは `run` と同じ表示で 64。重複キーの警告(`conductor: warning: <ファイル>:<行>: duplicate key ...`)は標準エラーに出すが、誤りではないので、警告だけなら 0(`-v` の 1 行は、警告のあとに出る)。`pipeline`・`run` は、警告を表示しない。実装は `aux_commands::check_definition_and_env`(`graph` と共有)。

### 7.6 `convert <file> --to <yaml|toml>`

入力ファイルを、入力形式を `--to` の反対とみなして変換する(拡張子は見ない)。結果を全部作ってから標準出力に 1 回で書く。失敗したときは標準出力に何も書かず、標準エラーに原因を出して 67。標準出力に書けないときも 67(`cannot write to standard output`)。シグナルの準備はしない。変換の規則そのもの(コメントとキー順は保持しない、アンカーの展開、入力 1 MiB まで等)は README の記述を根拠とする。

### 7.7 `graph [<pipeline-name>]`(v0.9.0 以降)

定義の遷移(成功と警告の行き先、`@end`)を図にして、標準出力に 1 回で書く。ジョブを起動せず、シグナルの準備もせず、ステートを書かない・消さない(`--state` のときだけ読む)。実装は `crates/cli/src/graph.rs` の `run_graph`。

順序(誤りの終了コードを括弧に示す):

1. `check_definition_and_env`(`validate` と同じ検査): 定義の読み込みと検証(64)→ 全ジョブの `command` の検査(64)→ `pipeline` と同じ範囲の `.env` の読み取り(64)。名前を渡すと定義の名前と照合する(不一致は 64)。`.env` の重複キーの警告は標準エラーに出す(誤りではない)。`.env` の値は、図にも出力にも含めない(名前も出さない)。
2. `--state` のときだけ: run ID の決定(`--run-id`、なければ定義の `run_id_env` の環境変数。違反は 68)→ ステートを、**完了したものも含めて**読む(`read_state_raw`。読めない・壊れている・版の不一致は 65。ファイルなし・長さ 0 は、ステートなし)→ 定義との照合(食い違いは 65)。ステートがなければ、すべてのジョブが未実行(`[  ]`)。ステートがあれば `overlay_from_state` で重ね合わせを作る(05 の 15 章)。
3. 図の構造を作り(`build_graph`)、`--format text`(既定)なら配置を決めて(`layout_graph`)`render_text`、`--format mermaid` なら `render_mermaid` で、文字列を組み立てる。`-v` は text にだけ効く(Mermaid には影響しない)。
4. 組み立て終えた文字列を、標準出力に 1 回で書く。書けなければ(閉じたパイプなど)`cannot write to standard output` を標準エラーに出して 65(`watch` と同じ)。

終了コード: 成功 0、定義・`command`・`.env` の誤り 64、ステートの誤りと標準出力に書けないとき 65、使い方の誤り 68(`--format` の値の誤り、`--format`・`--state` の重複、`graph` 以外での `--format`・`--state`、`--state` なしの `--run-id`、`-v` の重複など)。

出力の形(text のレーンの図、辺の一覧、Mermaid、`--state`、`-v`)は 08 の 11 章に全例を載せ、図の構造と配置の規則は 05 の 15 章。

### 7.8 `init [<pipeline-name>]`(v0.9.0 以降)

定義のひな形を作る。定義、`.env`、ステートを読まず、ジョブを実行せず、シグナルの準備もしない。引数の誤り(68)は、`parse_args` が、ファイルに触れる前に返す。実装は `crates/cli/src/init.rs` の `run_init`。

1. オプション(`init_options()`): 出力先 = `--config` の値(省略時 `conductor.toml`。値が `-` だけなら標準出力)、`force` = `--force`、名前 = 位置引数(省略時 `my-pipeline`)。
2. ひな形の文字列を作る(`render_template(言語, 名前)`。言語は、表示が必要になった時点のロケール。08 の 12 章)。
3. 出力先が標準出力(`--config -`): ひな形だけを、標準出力に 1 回で書く(成功のメッセージは出さない。`--force` は無視する)。書けなければ `cannot write the template to standard output` を標準エラーに出して 65。成功は 0。
4. 出力先がファイル: `Runtime::create_file(path, ひな形のバイト列, force)` を呼ぶ(06 の 13 章に、作成の規則)。
   - `Created` → 標準エラーに `created <path>`(日本語は `<path> を作成しました`)、0。
   - `Replaced`(`--force` で置き換えた)→ `replaced <path>`(日本語は `<path> を置き換えました`)、0。
   - `AlreadyExists`(`--force` なし。何も変えない)→ `conductor: error: <path> already exists (use --force to overwrite)`、**69**。
   - そのほかの `CreateError`(`ParentMissing`、`IsDirectory`、`PermissionDenied`、`OtherIo`)→ `conductor: error: cannot create <path>: <理由>`、65。表示は種類ごとの固定の文とパスだけで、OS のエラーの文を含めない。
5. パスの表示は、制御文字をエスケープする(切らない)。成功のメッセージは、標準エラー(`Notice` の水準。常に出る)に出す。標準出力に出るのは、`--config -` のひな形だけ。

終了コード: 作れた・標準出力に出せた 0、既にある 69、作れない(親のディレクトリが無い、書く権限がない、作る先がディレクトリ、そのほかの入出力の誤り)と標準出力に書けない 65、使い方の誤り 68(名前が空・空白だけ、位置引数が 2 つ以上、`--config` と `--force` 以外のオプション、`init` 以外への `--force`)。

### 7.9 `run`・`pipeline` の `--dry-run`(v0.9.0 以降)

ジョブを実行せずに、実行計画を標準出力に出す。ジョブを起動せず、ステートに書かず・消さず(読むだけ)、シグナルの準備もしない。`--remove-on-complete` は、何も消さないので効果がない。実装は `crates/cli/src/dry_run.rs` の `run_dry_run`。`run_individual`・`run_pipeline` の先頭で、ジョブID(`run`)・パイプライン名(`pipeline`)を取り出したあとに呼ばれ、`--dry-run` なら、通常の準備(`prepare`)の代わりにこの流れに入る。

順序(誤りの終了コードを括弧に示す):

1. `Ctx::prepare_checked`(`prepare` の、シグナルの準備を除いた部分): 定義の読み込みと検証(64。`pipeline` は名前の照合)→ 全ジョブの `command` の検査(64)→ `run` の対象のジョブが定義にあるか(68)→ **`--assume` のジョブIDが定義にあるか**(なければ 68。誤りの表示は `job is not defined in the definition file` に固定で、ジョブIDを含めない)→ run ID の決定(68)→ `.env` の読み取り(64。`pipeline` は全ジョブ分、`run` は対象のジョブ分。**本番と同じ範囲**)→ ステートの読み込み(65。完了したステートは「ステートなし」)→ 定義との照合(65)。
2. 各ジョブの `command` を `check_command` で分割する(定義の検査で済んでいるので、誤りは起きない。起きれば本番と同じ表示で 64)。
3. `plan_pipeline` / `plan_run` で計画を作る(05 の 16 章)。**本番なら順序外として拒否される**(`Reject`)ときは、本番と同じ表示(`rejected: ...`。08 の 4.7)で 66。
4. `render_plan(言語, 計画)` で文字列を組み立て、標準出力に 1 回で書く。書けなければ `cannot write to standard output` を標準エラーに出して 65。

終了コード: 計画を出せたら 0(模擬の結果がエラーでの停止でも 0。実際の終了コードは示さない)、定義と `.env` の誤り 64、ステートの誤り・標準出力に書けない 65、拒否 66、使い方の誤り 68(`--assume` の書式、範囲外、重複、定義にないジョブID、`--dry-run` なしの `--assume`、`run`・`pipeline` 以外での `--dry-run`、`--dry-run` の重複)。`--assume` の誤りの表示に、入力の値を含めない(対象は `--assume`)。

出力の形は 08 の 11 章、模擬の規則は 05 の 16 章。

## 8. ヘルプとバージョン

- `-h`/`--help`: 使い方全文を標準出力に出して 0。言語はロケールに従う。全文は 08 章に掲載。
- `-V`/`--version`: 標準出力に `conductor <版>\n`(版は `CARGO_PKG_VERSION`。v0.9.1 では `conductor 0.9.1`。このリポジトリの 4 クレートの `Cargo.toml` の版は `0.9.1` である)。ロケールを判定しない。終了コード 0。

実際の出力:

```text
$ conductor --version
conductor 0.9.1
```

## 9. 実際の実行例(`LANG=C`)

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

## 10. 実装と README の差

| 項目 | README | 実装 |
|------|--------|------|
| 引数の置き場所 | 記述なし | オプションはコマンド名の前にも置ける。`--config` 等は次の引数を、`-` 始まりでも値にする |
| `--version` の形式 | 記述なし | `conductor <版>`(標準出力) |
| `-h`/`-V` の優先 | 記述なし | ほかの誤りより先。先に現れた方が勝つ。`--` の後ろでは効かない |
| 短縮形の連結 | 記述なし | `-vh`, `-vv`(後者は 68)が lexopt の規則で解釈される |
| 重複指定の誤り種別 | 「使い方の誤り」(68) | すべて `invalid value`(対象はオプション名) |
| `watch --all` | `reset` 以外への `--all` は 68 | 一致(`option not allowed for this command: '--all'`) |
| `--config=` の空値 | 記述なし | 空パスとして受理され、後段で `definition file does not exist` 等(64)になる |
| `--dry-run` の計画と実際の実行の、完了したステートの扱い(v0.9.0 以降) | 「計画は、完了したステートを、ステートなしとして扱う。`graph --state` は完了したものも読む」 | 一致(`--dry-run` は `read_state`、`graph --state` は `read_state_raw`。目的の違いによる、意図した違い) |
| `init` が作ったファイル(v0.9.0 以降) | 「新規の作成の短い間と強制終了のあとに、空のファイルが見えることがある」 | 一致(`create_new` で作ってから書くため。`--force` で直せる) |
