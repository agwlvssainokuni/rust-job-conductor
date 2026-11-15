# API ドキュメント

利用者が見える面(CLI、定義ファイル、ステートファイル、終了コード)を先に、ライブラリの契約を後に書く。根拠は `crates/` のコード。

## 1. CLI(`conductor`)

構文(`report/messages.rs:59-100` の USAGE と同じ):

```text
conductor run [-v] [--config <path>] [--run-id <id>] <job-id>
conductor pipeline [-v] [--config <path>] [--run-id <id>] <pipeline-name>
conductor watch [--config <path>] [--run-id <id>]
conductor reset [--config <path>] [--run-id <id> | --all]
conductor validate [-v] [--config <path>] [<pipeline-name>]
conductor convert <file> --to <yaml|toml>
conductor --help | --version
```

### 文法(`crates/cli/src/args.rs`)

| コマンド | 位置引数 | `-v` | `--config` | `--to` | `--run-id` | `--all` |
|----------|----------|------|------------|--------|------------|---------|
| run | ジョブID(1) | 可 | 可 | 不可 | 可 | 不可 |
| pipeline | パイプライン名(1) | 可 | 可 | 不可 | 可 | 不可 |
| watch | なし | 不可 | 可 | 不可 | 可 | 不可 |
| reset | なし | 不可 | 可 | 不可 | 可 | 可(`--run-id` と併用不可) |
| validate | パイプライン名(0 か 1) | 可 | 可 | 不可 | 不可 | 不可 |
| convert | 入力ファイル(1) | 不可 | 不可 | 必須 | 不可 | 不可 |

- 許されない場所のオプション、`-v`・`--config`・`--to`・`--run-id`・`--all` の重複(`-vv` を含む)、`--all` と `--run-id` の併用、知らないコマンド・オプション、引数の過不足、定義にないジョブIDは、すべて使い方の誤りで、終了コード 68(`args.rs:171-307`、`ctx.rs:84-90`)。
- `--opt value` と `--opt=value` の両方を受け付ける。`--` で、オプションの終わり。`-h`/`--help`・`-V`/`--version` は、`--` より前にあれば、ほかの誤りより先に処理する(`args.rs:136-140`)。`--version` の表示は `conductor <版>`(言語に関わらず)。
- 既定の定義ファイルは、実行ディレクトリの `conductor.toml`(`ctx.rs:26,51-56`)。
- 既定では conductor 自身の出力はない。エラー(標準エラー)だけ。`-v` は、ジョブの結果・スキップ・遷移を、標準エラーに `[conductor] ...` の形で出す。`validate -v` は、通ったとき 1 行(`definition is valid (N jobs)` / `定義は正しいです(ジョブ N 個)`)。

### コマンドの動き

| コマンド | 動き | 根拠 |
|----------|------|------|
| `run <ジョブID>` | シグナルの準備 → 検査(下の順序)→ `decide_before_run`。`Reject` は 66。`Skip` は記録して 0。`Run` は実行して、結果をステートに書き、完了なら初期化。終了コードは `exit_code_for(...)` | `run.rs` |
| `pipeline <名前>` | 名前を定義と照合(不一致は 64)。記述順に 1 つずつ判断・実行。エラー・中断で止まる。完了で初期化して 0。ジョブのあいだにシグナルがあれば、再開位置を記録して 128 + 番号 | `pipeline.rs` |
| `watch` | ステートを読み(定義は検証しない)、表を標準出力に 1 回。空なら空の旨を 1 行。標準出力に書けなければ 65 | `aux_commands.rs:16-42` |
| `reset` | ステートを読まず・検査せず消す。ないときも 0。`--all` は全部 | `aux_commands.rs:44-71` |
| `validate` | `check_definition`(定義の検証 + 全ジョブの `command` の検査)だけ。誤りは 64 | `aux_commands.rs:73-95`、`ctx.rs:136-165` |
| `convert` | `--to` の反対の形式として読み、変換結果を標準出力に 1 回で書く。失敗は標準出力が空で 67 | `aux_commands.rs:97-123` |

`run`/`pipeline` の検査の順序(`ctx.rs:63-131`): シグナルの準備(失敗は 70)→ 定義の読み込み・検証(64)→ 全ジョブの `command` の検査(64)→ ジョブIDの確認(68。`run` のみ)→ run ID の決定(68)→ ステートの読み込み(65)→ 定義との照合(65)。`watch`・`reset`・`validate`・`convert` はシグナルの準備をしない(既定の動作)。

### run ID の規則(1 か所にまとめた事実)

- 値: `A-Za-z0-9._-`、1〜64 文字、先頭が `.` でない(`..` も不可)。大文字小文字は別の値(`core/src/run_id.rs:35-53`)。規則違反は、`--run-id` でも環境変数でも 68。誤りの文に値は含めない(`args.rs:309-342`)。
- 与え方: `--run-id <ID>`。なければ、環境変数。名前は、定義の `[pipeline].run_id_env`(省略時 `CONDUCTOR_RUN_ID`)。`--run-id` があれば環境変数を見ない。環境変数が未定義・空なら run ID なし。`--run-id ""` は、環境変数があっても、run ID なしに固定する(`args.rs:320-342`、`ctx.rs:92-103`)。
- 使えるコマンド: run・pipeline・watch・reset のみ。validate・convert は 68。
- ステートファイル: run ID ありは `<定義ファイル名>.<run-id>.state`、なしは `<定義ファイル名>.state`(`runtime/src/types.rs:StateLocation::of`)。run ID ありのステートの中身には `run_id = "<ID>"` を書く。名前と中身が食い違えば壊れたステート(65)(`runtime/src/state/reader.rs:233-244`)。
- `watch`・`reset`(`--all` なし): `--run-id` があれば定義を読まない。なければ、定義から `run_id_env` だけをゆるく読む(壊れている・キーなし・型や形が違うときは既定の名前。ほかの誤りは無視)(`ctx.rs:285-303`、`run_id.rs:peek_run_id_env`)。`watch` の表は、run ID があるとき、パイプライン名の次の行に `Run ID: <ID>`(`report/watch.rs:86-98`)。
- `reset --all`: 環境変数を見ない。定義と同じディレクトリで、(a) `<名>.state` は、通常のファイルなら、読まずに消す、(b) `<名>.<有効な run ID>.state` は、通常のファイルで、中身の `run_id` が名前と一致するものだけ消す。一時ファイル(`.<名>.state.tmp.*`)・中身に `run_id` がないか値が違うもの(黙って残す)・ディレクトリ・シンボリックリンクは消さない。読めない・壊れているものは残して、名前を警告(`conductor: warning: ...`)し、終了コードは 0。消すのに失敗すれば 65(`runtime/src/state/reset_all.rs`、`aux_commands.rs:44-62`)。
- 大文字小文字を区別しないファイルシステムでは、`A` と `a` が同じファイルになる(中身の照合で 65 になりうる)。
- 排他制御はしない。完了した run のステートは消える。放置された run のステートは、自動では消えない(`reset --all` で片づける)。

## 2. 定義ファイル(TOML。`crates/core/src/definition/walk.rs`)

- ルートのキー: `format_version`(必須。整数 1 のみ。ない/違うと `versionMissing`/`versionUnsupported`)、`pipeline`、`job`(配列のテーブル。1 つ以上)。それ以外のキーは `unknownKey`(`[[jobs]]` もここで拒否)。
- `[pipeline]`: `name`(必須)、`description`(省略可)、`run_id_env`(省略可。文字列。`A-Za-z_` で始まり、`A-Za-z0-9_` が続く)。
- `[[job]]`: `id`(空でなく、重複せず、`@` で始めない)、`command`(必須。文字列、または文字列の配列)、`success_exit_codes`(省略時 `[0]`)、`warning_exit_codes`(`0` を含まず、成功と重ならない)、`[[job.transitions]]`(`on` は `success` か `warning`、`next` は記述順で後ろのジョブID か `@end`。同じ `on` は 1 つ。エラーには書けない)。
- `command`(文字列): シェルを介さず、空白で分割。`'...'`(そのまま)、`"..."`(`\"` だけが `"` 1 つ)。`$`・`*`・`|`・`;`・`&&`・`>` は解釈しない。引用符が閉じていなければ `unclosedQuote`、名前がなければ `empty`(`runtime/src/split.rs`)。配列: 1 要素 = 1 引数。先頭が空・配列が空は定義の誤り。2 番目以降の `""` は有効(`core/src/types.rs` の `Command`)。
- 大きさの上限 1 MiB、入れ子 32 段(`intake.rs`、`depth.rs`)。誤りは、見つかったものをすべて集めて返す(ファイル・版・文法の誤りは 1 件)。

## 3. ステートファイル(TOML。`runtime/src/state/writer.rs:serialize`)

キー: `format_version = 1`、`definition_key`・`definition_path`(同じ文字列)、`pipeline_name`、`job_order`(配列)、`job`(配列のテーブル: `job_id`、`status`、`exit_code`、`skip_reason`)、`resume_job_id`(あれば)、`run_id`(run ID ありのときだけ)、`end_reached`、`completed`。1 MiB まで。一時ファイル `.<ステート名>.tmp.<pid>.<時刻>.<連番>` に書いて `rename`(`Durable` は同期)。`completed` のまま残ったものは、読むと空として返る。ステートには、command・引数・環境変数・標準出力は書かない。

## 4. 終了コード

| コード | 意味 | 根拠 |
|--------|------|------|
| 0 | 成功、警告、スキップ、watch・reset・validate・convert の成功 | `exit.rs:17`、`classify.rs` |
| ジョブの値 | エラーのジョブの終了コード(0〜255) | `exit.rs:35-42` |
| 1 | 成功コードに含まれない `0` で終わったジョブ(または、終了コードを取れなかったジョブ)。原因を標準エラーに表示 | `classify.rs:23,99-108` |
| 64 | 定義の誤り(`validate` を含む。パイプライン名の不一致、`command` の誤り、`run_id_env` の誤りを含む) | `exit.rs:21` |
| 65 | ステートの誤り(読めない・壊れ・版・食い違い・書き込み・初期化・削除の失敗)、`watch` の標準出力の失敗、完了を判定できない場合 | `exit.rs:23`、`pipeline.rs:157-162` |
| 66 | 拒否(順序外) | `exit.rs:25` |
| 67 | 変換の誤り、`convert` の標準出力の失敗 | `exit.rs:27` |
| 68 | 使い方の誤り、run ID の値の誤り | `exit.rs:29` |
| 70 | 内部の誤り(パニック、シグナルの準備の失敗) | `exit.rs:31`、`panic_hook.rs` |
| 127 | ジョブを起動できなかった | `classify.rs:20` |
| 128 + 番号 | ジョブがシグナルで終了、または conductor が中断された(SIGTERM 143、Ctrl+C 130) | `classify.rs:23-26`、`exit.rs:44-48` |

範囲外(255 超・負)は下位 8 ビット。エラーのジョブで 0 になるなら 1。元の値は `-v` に `exit code N, conductor exits with M` の形で示す(`report/mod.rs:474-482`)。64〜68・70 はジョブ自身の終了コードと重なりうる(区別は標準エラーの `conductor: error: ...` の行)。

## 5. シグナル(`cli/src/platform/`、`runtime/src/platform/unix.rs`)

- UNIX 系: `SIGHUP`・`SIGINT`・`SIGQUIT`・`SIGTERM` を受け取り、子プロセスの pid だけに、同じシグナルを 1 回だけ転送する(プロセスグループには送らない。ジョブは別のプロセスグループで起動)。結果はエラーとして記録し、128 + 番号。ほぼ同時の複数のシグナルは、`signal-hook` が小さい番号順に返すため、小さい番号が「最初」になりうる。
- Windows: Ctrl+C(番号 2、終了 130)のみ(`ctrlc`)。Ctrl+Break・コンソールを閉じた通知は対象外。
- ジョブの起動前・`pipeline` のジョブのあいだの合図は、次のジョブを起動せず、再開位置を記録して 128 + 番号(エラーとしては記録しない)。検査・ステートの確認の途中の合図は、最後まで進め、64・65・66・スキップの終了コードが優先する。
- シグナルの準備は、定義の検査より前(`run`/`pipeline` のみ)。

## 6. ロケール・色・エスケープ(`cli/src/report/`)

- 言語: `LC_ALL` → `LC_MESSAGES` → `LANG` の順に、最初の空でない値が `ja` で始まれば日本語、それ以外・どれもなければ英語。状態の語(`success`・`warning`・`skipped`・`error`・`pending`)と `-v` の `job=`・`result=` は英語のまま。`conductor: error: `・`conductor: warning: `・`[conductor] ` の接頭辞は言語に関わらず同じ。パニックのフックとシグナルの準備の失敗の文は、固定の英語。
- 色: `watch` の表のみ。UNIX 系、標準出力が端末、`NO_COLOR` が未設定または空のとき。状態は色に関わらず文字でも書く。Windows は色なし。
- エスケープ: ジョブID・名前は、制御文字・書式文字・双方向制御文字を `\u{..}` にし、64 文字で切る(`…` を付ける)。パスは切らない。

## 7. ライブラリの契約

| クレート | 契約 | 公開する関数・型 |
|----------|------|------------------|
| core(C2) | `lib.rs` | `load_definition`、`parse_definition`、`decide_before_run`、`classify_and_decide`、`resume_point_on_interrupt`、`RunId`・`peek_run_id_env`・`DEFAULT_RUN_ID_ENV`・`MAX_RUN_ID_LEN`、`PipelineDefinition`・`JobDefinition`・`Command`・`PipelineState` ほか |
| runtime(C3) | `lib.rs` | `run_job`、`check_command`、`read_state`、`write_state`、`check_consistency`、`reset_state`、`reset_all_states`、`initial_state`、`apply_outcome`・`apply_skip`・`apply_interrupt`、`InterruptSignal`・`StateLocation`・`Durability` |
| converter(C4) | `lib.rs` | `convert`、`convert_text`、`ConversionRequest`・`ConvertedText`・`TargetFormat`・`ConversionError` |
| cli | `lib.rs` | `run_cli`、`run_cli_with`、`parse_args`、`install_panic_hook`、`Deps` と trait 4、`exit` の定数 |

converter の上限: 入力 1 MiB、展開のノード数 10 万、スカラーの総バイト数 16 MiB、入れ子 32 段。コメントとキーの記述順は保持せず、YAML のアンカー・エイリアス・マージキーは展開、`null`・日時は失敗(`converter/src/lib.rs`)。

## 8. 出力先のまとめ

| 出力 | 先 |
|------|----|
| ジョブ本体の出力 | そのまま(加工しない) |
| エラー・警告・`-v` の行 | 標準エラー |
| `watch` の表、`convert` の結果、`--help`、`--version` | 標準出力 |
