# 04. コマンドライン(`conductor` v0.8.0)

実装: `crates/cli/src/{main,lib,args,types,ctx,run,pipeline,aux_commands}.rs`。
引数の解釈は `lexopt` で字句に分け、コマンドごとの文法の表で検査する。引数はシェルを介さず、解釈も展開もしない。

## 1. 全体像

- 実行ファイル名は `conductor`。プログラム名を除く引数を `std::env::args_os()` で受け取る。
- 入口 `main` は、(1) パニックのフックを設定し、(2) `run_cli` の返した終了コードで `std::process::exit` を 1 回だけ呼ぶ。
- サブコマンドは 6 つだけ: `run`, `pipeline`, `watch`, `reset`, `validate`, `convert`。
- 特別なオプション(`-h/--help`, `-V/--version`)は、コマンドの解釈より先に処理する。
- 使い方の誤りは、標準エラーに「1 行の誤り + 空行 + 使い方全文」を書いて終了コード 68(10 章の表を参照)。ジョブは実行しない。
- 既定では、`conductor` 自身は何も出力しない(ジョブ本体の出力がそのまま流れる)。エラーのとき、または `-v` のときだけ標準エラーに書く(`watch`・`--help`・`--version`・`convert` は標準出力に結果を書く)。

## 2. 構文

```text
conductor run [-v] [--config <path>] [--run-id <id>] [--remove-on-complete] <job-id>
conductor pipeline [-v] [--config <path>] [--run-id <id>] [--remove-on-complete] <pipeline-name>
conductor watch [--config <path>] [--run-id <id>]
conductor reset [--config <path>] [--run-id <id> | --all]
conductor validate [-v] [--config <path>] [<pipeline-name>]
conductor convert <file> --to <yaml|toml>
conductor --help | --version
```

## 3. オプションの一覧と、コマンドごとの可否

| オプション | 短縮形 | 値 | run | pipeline | watch | reset | validate | convert |
|------------|--------|----|-----|----------|-------|-------|----------|---------|
| `--verbose` | `-v` | なし | 可 | 可 | 不可 | 不可 | 可 | 不可 |
| `--config` | なし | パス(必須) | 可 | 可 | 可 | 可 | 可 | 不可 |
| `--run-id` | なし | 文字列(空も可) | 可 | 可 | 可 | 可 | 不可 | 不可 |
| `--remove-on-complete` | なし | なし | 可 | 可 | 不可 | 不可 | 不可 | 不可 |
| `--all` | なし | なし | 不可 | 不可 | 不可 | 可 | 不可 | 不可 |
| `--to` | なし | `yaml` か `toml`(必須) | 不可 | 不可 | 不可 | 不可 | 不可 | 必須 |
| `--help` | `-h` | なし | どこでも | | | | | |
| `--version` | `-V` | なし | どこでも | | | | | |

位置引数:

| コマンド | 位置引数 | 個数 |
|----------|----------|------|
| `run` | ジョブID | ちょうど 1 |
| `pipeline` | パイプライン名 | ちょうど 1 |
| `watch` | なし | 0(渡すと `unexpected argument`) |
| `reset` | なし | 0(渡すと `unexpected argument`) |
| `validate` | パイプライン名 | 0 または 1(省略時は名前を照合しない) |
| `convert` | 入力ファイル | ちょうど 1(`--to` は別に必須) |

コマンド名は位置引数の先頭(最初に現れた位置引数)である。コマンド名の前後どちらにもオプションを置ける(例: `conductor --config c.toml -v run x` は有効)。

各オプションの意味と既定値:

| オプション | 意味 | 既定 |
|------------|------|------|
| `-v`, `--verbose` | ジョブの結果・スキップ・遷移・中断を標準エラーに出す(`validate` は成功時 1 行)。1 段階のみ | 出さない |
| `--config <path>` | 定義ファイルのパス。`OsString` のまま保つ(UTF-8 でないパスも可) | 実行ディレクトリの `conductor.toml` |
| `--run-id <id>` | ステートファイルを分ける run ID。空文字は「run ID なしに固定」 | 未指定(環境変数を見る。5 章) |
| `--remove-on-complete` | 完了時にステートファイルを削除する | 指定なし(定義の `[pipeline].remove_on_complete`、さらに既定は `false`) |
| `--all` | 定義のステートを、run ID の有無によらず全部消す(`reset` のみ) | なし |
| `--to <yaml\|toml>` | `convert` の出力形式。大文字小文字を区別し、`yaml` と `toml` 以外は `invalid value`(対象は値そのもの) | なし(必須) |

## 4. 引数の字句規則

| 規則 | 挙動 |
|------|------|
| `--opt value` と `--opt=value` | `--config`, `--run-id`, `--to` の両方の形を受け付ける |
| 値の取り方 | 値を取るオプションは、次の引数を、先頭が `-` でもそのまま値にする(例: `--config --help` は `--help` というパスを指す)。次の引数がなければ `missing value` |
| 空の値 | `--config=`(空パス)は文字列として受理され、後段でファイルなしの誤りになる。`--run-id=` は有効(run ID なしに固定) |
| 短縮形の連結 | lexopt の標準どおり。`-vh` は `-v` と `-h`(`-h` が勝つ)。`-vv` は `-v` が 2 回 |
| `--` | オプションの終わり。以降はすべて位置引数(`-v` や `--help` も位置引数)。例: `run -- -weird` はジョブID `-weird`。`-- --help` は先頭の位置引数 `--help` がコマンド名として未知になり `unknown command` |
| 値を取らないオプションに `=値` | `--verbose=x`, `--all=x`, `--remove-on-complete=x` は `invalid value`(対象はオプション名)。文法にない `--bogus=x` は `unknown option`(対象 `--bogus`) |
| 非 UTF-8 | 位置引数(コマンド名、ジョブID、パイプライン名、`--to` の値)が UTF-8 でないと `invalid value`(対象は lossy 変換した文字列)。`--config` と convert の入力パスは `OsString` のまま通す。`--run-id` の値が非 UTF-8 だと `invalid value`(対象 `--run-id`) |
| 解釈の打ち切り | 値の欠落(`--config` 等が最後の引数)で字句の走査は止まる。それ以前に `-h`/`-V` があれば、そちらが先に処理される |

### 4.1 解釈の順序(誤りの優先順位)

1. `-h/--help` または `-V/--version` が(`--` より前に)1 つでもあれば、ほかの誤りに関係なく、先に現れた方を処理して終了コード 0。両方あるとき、先に現れた方が勝つ(`-V -h` は version、`-h -V` は help)。
2. 字句の誤り(値の欠落、`--verbose=x` のような値の取り残し)。
3. 文法にないオプション(`unknown option`。複数あれば最初のもの)。
4. 位置引数が 0 個: `missing argument`(対象なし)。例: 引数なし、`-v` だけ。
5. 先頭の位置引数が 6 コマンドのいずれでもない: `unknown command`(対象はその語。空文字も同じ)。
6. そのコマンドで使えないオプション: `option not allowed for this command`。検査の順は `-v`, `--config`, `--to`, `--run-id`, `--all`, `--remove-on-complete`。
7. 重複: `-v`, `--config`, `--to`, `--run-id`, `--all`, `--remove-on-complete` のどれかが 2 回以上 → `invalid value`(対象はそのオプション名。`-vv` は対象 `-v`)。
8. `--all` と `--run-id` の併用 → `invalid value`(対象 `--all`)。
9. `--run-id` の値の検査(下の 5.2)。違反は `invalid value`(対象 `--run-id`。値は含めない)。
10. 位置引数の過不足: `run`/`pipeline`/`convert` で入力なし → `missing argument`。`watch`/`reset` でコマンド名の後ろに位置引数あり → `unexpected argument`(対象はその語)。それ以外のコマンドで、コマンド名を除く 2 個目の位置引数があれば `unexpected argument`(対象は 2 個目の語。`run a b` なら `b`)。
11. `convert`: `--to` なし → `missing argument`(対象 `--to`)。`--to` の値が `yaml`/`toml` 以外 → `invalid value`(対象は値)。

使い方の誤りの種類と文言は 08 章のカタログ(`usage_kind`)を参照。表示形式は「`conductor: error: <文>: '<対象>'`(対象があるとき)、空行、使い方全文」。

## 5. run ID

### 5.1 決定の優先順位(`run`・`pipeline`)

1. `--run-id <値>` が指定されていれば、環境変数は見ない。値が空文字なら run ID なし。
2. 指定がなければ、環境変数を読む。環境変数の名前は、定義の `[pipeline].run_id_env`(なければ `CONDUCTOR_RUN_ID`)。未定義または空文字なら run ID なし。UTF-8 でない値は未定義扱い。
3. 得た値は `RunId` の規則で検査する。違反は `invalid value`(終了コード 68)。対象は、オプション由来なら `--run-id`、環境変数由来なら `run ID (environment variable)`(値は含めない)。

`run`・`pipeline` では、環境変数の名前を知るために定義を先に読む。検査の順序は 10 章の「終了コードの決定順序」を参照。

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
| `NO_COLOR` | `watch` の表を出すとき | 未設定または空以外の値なら色なし(08 章) |

## 7. 各コマンドの動作

### 7.1 `run <job-id>`

1. シグナルの監視を開始(定義の検査より前)。失敗は 70。
2. 定義の読み込みと検証(失敗は 64)。全ジョブの `command` を検査(失敗は 64)。
3. ジョブIDが定義にあるか(なければ `job is not defined in the definition file: '<ID>'`、68)。
4. run ID の決定(68)。
5. ステートの読み込み(65)と、定義との照合(65)。
6. 起動前の判断: 実行 / スキップ(0)/ 拒否(66)。
7. 実行なら、ステートが空のとき初期ステートを書き(`Fast`)、ジョブをリトライ込みで実行し、結果を分類してステートを 1 回書き(`Durable`)、完了かつ削除指定ならステートを消す。終了コードはジョブの結果(10 章)。
8. スキップなら、記録が変わるときだけ書く。完了し削除指定なら消す。0 で終わる。

`-v` のとき、結果行・遷移行・中断行を標準エラーに出す(08 章)。

### 7.2 `pipeline <pipeline-name>`

`run` と同じ検査(パイプライン名は定義の `name` と照合。不一致は 64)を行い、定義の記述順にジョブを 1 つずつ、起動前の判断で処理する。

- 実行 → 結果を記録。エラー(または分類なし=起動前の中断)で停止し、そのジョブの終了コードで終わる。自動の再開・再試行はしない(`[job.retry]` を除く)。
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

### 7.6 `convert <file> --to <yaml|toml>`

入力ファイルを、入力形式を `--to` の反対とみなして変換する(拡張子は見ない)。結果を全部作ってから標準出力に 1 回で書く。失敗したときは標準出力に何も書かず、標準エラーに原因を出して 67。標準出力に書けないときも 67(`cannot write to standard output`)。シグナルの準備はしない。変換の規則そのもの(コメントとキー順は保持しない、アンカーの展開、入力 1 MiB まで等)は README の記述を根拠とする。

## 8. ヘルプとバージョン

- `-h`/`--help`: 使い方全文を標準出力に出して 0。言語はロケールに従う。全文は 08 章に掲載。
- `-V`/`--version`: 標準出力に `conductor <版>\n`(版は `CARGO_PKG_VERSION`。v0.8.0 では `conductor 0.8.0`)。ロケールを判定しない。終了コード 0。

実際の出力:

```text
$ conductor --version
conductor 0.8.0
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
