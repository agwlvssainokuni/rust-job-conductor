# API Documentation

## CLI(外部 API)
```text
conductor run [-v] [--config <パス>] [--run-id <ID>] <ジョブID>
conductor pipeline [-v] [--config <パス>] [--run-id <ID>] <パイプライン名>
conductor watch [--config <パス>] [--run-id <ID>]
conductor reset [--config <パス>] [--run-id <ID> | --all]
conductor validate [-v] [--config <パス>] [<パイプライン名>]
conductor convert <ファイル> --to <yaml|toml>
conductor --help | --version
```
- 既定の定義は実行ディレクトリの `conductor.toml`。ステートは定義と同じディレクトリの `<定義名>.state`(run ID ありなら `<定義名>.<run-id>.state`)(`StateLocation::of(path, run_id)`。`crates/runtime/src/types.rs:114`)。
- 終了コード: 0 成功/警告/スキップ、ジョブの値(エラー)、1(成功コード外の 0)、64 定義(`command` の誤りと `run_id_env` の型・形の誤りを含む。`validate` の失敗も)、65 ステート(中身の `run_id` とファイル名の食い違いを含む)、66 拒否、67 変換、68 使い方(知らないジョブID、run ID の値の誤り、`--all` の誤用を含む)、70 内部、127 起動失敗、128+番号 シグナル。全 OS で 0〜255 に丸める。
- 出力: 既定では conductor 自身の出力なし。エラー時のみ標準エラー。`-v` で結果と遷移を標準エラーへ(`validate` は成功時に 1 行)。言語は `LC_ALL`/`LC_MESSAGES`/`LANG` の `ja` で日本語。色は UNIX 系の端末で `NO_COLOR` 未設定の `watch` だけ。

### 引数の文法(`crates/cli/src/args.rs`)
| コマンド | 位置引数 | `-v` | `--config` | `--run-id` | `--all` | `--to` |
|----------|----------|------|------------|------------|---------|--------|
| `run` | ジョブID(1) | 可 | 可 | 可 | 不可 | 不可 |
| `pipeline` | パイプライン名(1) | 可 | 可 | 可 | 不可 | 不可 |
| `watch` | なし | 不可 | 可 | 可 | 不可 | 不可 |
| `reset` | なし | 不可 | 可 | 可(`--all` と排他) | 可 | 不可 |
| `validate` | パイプライン名(0 か 1) | 可 | 可 | 不可 | 不可 | 不可 |
| `convert` | 入力ファイル(1) | 不可 | 不可 | 不可 | 不可 | 必須(`yaml` か `toml`) |

- `--config path` と `--config=path` を受け付ける(`--run-id` も同じ形)。`--` はオプションの終わり。`-v`、`--config`、`--to`、`--run-id`、`--all` の重複は `invalidValue`。`--all` と `--run-id` の併用も `invalidValue`(対象は `--all`)。使えないコマンドでの `--run-id`・`--all` は `optionNotAllowed`。`--help`・`-h`・`--version`・`-V` は、ほかの誤りより先に処理する(0 で終わる)。
- 使い方の誤りは 68。種類は 8 つ(`UnknownCommand`、`MissingArgument`、`UnexpectedArgument`、`UnknownOption`、`OptionNotAllowed`、`MissingValue`、`InvalidValue`、`UnknownJob`)。表示は、`conductor: error: <種類の文>: '<対象>'` の 1 行、空行、使い方の全文(標準エラー)。種類の全部は `args_grammar.rs` が網羅を固定する。
- 使い方の文言は `crates/cli/src/report/messages.rs` の `USAGE`(英語・日本語。`--run-id` と `--all` の行を含む)。`README.md` の「使い方」(`:75-83`)とオプション表(`:94-103`)にも同じ内容がある。
- 環境変数は、`LC_ALL`・`LC_MESSAGES`・`LANG`(言語)、`NO_COLOR`(色)、run ID の環境変数(定義の `run_id_env`、既定 `CONDUCTOR_RUN_ID`)を読む(`Env::var`。`crates/cli/src/deps.rs`)。

### run ID の決定(`resolve_run_id`。`crates/cli/src/args.rs:327`)
- `--run-id` があれば、環境変数は見ない。空の値(`--run-id ""`)は、環境変数があっても、run ID なしに固定する。
- なければ、環境変数(名前は定義の `[pipeline].run_id_env`、省略時 `CONDUCTOR_RUN_ID`)。未定義と空は run ID なし。
- 値の規則(`RunId::parse`): `A-Za-z0-9._-`、1〜64 文字、先頭が `.` でない(`..` も不可)。規則違反は、オプションでも環境変数でも `invalidValue`(68)。値は誤りの文に含めない。
- `run`・`pipeline` では、定義の検査(64)・ジョブIDの確認(68)のあと、ステートを読む前に決める。`watch`・`reset` では、`--run-id` があれば定義を読まず、なければ定義から `run_id_env` だけをゆるく読む(壊れた定義・キーなし・型や形の誤りは既定名)。`reset --all` は環境変数も定義も見ない。

### 利用者に見える run ID 関連の出力(実装どおり)
接頭辞: エラー `conductor: error: `、警告 `conductor: warning: `、`-v` の行 `[conductor] `。
| 場面 | 出力先 | 英語 | 日本語 |
|------|--------|------|--------|
| `watch`(run ID あり) | 標準出力 | 1 行目 `Pipeline: <名前>`、2 行目 `Run ID: <ID>`、続けて `JOB  STATUS  NOTE` の表 | `パイプライン: <名前>`、`Run ID: <ID>`(見出しは同じ)、`ジョブ  状態  補足` |
| `watch`(run ID なし) | 標準出力 | `Pipeline: <名前>` のあと表(`Run ID` の行なし。従来のまま) | 同左(日本語) |
| `watch`(ステートなし) | 標準出力 | `no state (nothing has run yet)` | `ステートはありません(まだ実行されていません)` |
| `reset --all`(消すだけ) | なし | 何も出さず 0 | 同左 |
| `reset --all`(読めないファイルを残した) | 標準エラー | `conductor: warning: kept state files that could not be read (remove one with `conductor reset --run-id <ID>`):` に続き、`  <ファイル名>` を 1 行ずつ。終了コード 0 | `conductor: warning: 読めなかったため、次のステートファイルは残しました(`conductor reset --run-id <ID>` で個別に消せます):` に続き、同じ |
| 規則違反の `--run-id` の値 | 標準エラー | `conductor: error: invalid value: '--run-id'` + 空行 + 使い方(68) | `conductor: error: 値が正しくありません: '--run-id'` + 使い方 |
| 規則違反の環境変数の値 | 標準エラー | `conductor: error: invalid value: 'run ID (environment variable)'` + 使い方(68) | `値が正しくありません: 'run ID (environment variable)'` |
| `--all` と `--run-id` の併用 | 標準エラー | `conductor: error: invalid value: '--all'` + 使い方(68) | `値が正しくありません: '--all'` |
| `reset` 以外への `--all`、`validate`・`convert` への `--run-id` | 標準エラー | `conductor: error: option not allowed for this command: '--all'`(`'--run-id'`) | `このコマンドでは使えないオプションです: '--all'` |
| 食い違い・壊れた run ID つきステート | 標準エラー | `…state: <理由>; run `conductor reset --run-id <ID>` to discard the state`(65) | `…; `conductor reset --run-id <ID>` でステートを初期化できます` |
| `validate -v`(成功) | 標準エラー | `[conductor] definition is valid (4 jobs)`(1 個は `1 job`) | `[conductor] 定義は正しいです(ジョブ 4 個)` |
- `pipeline -v`・`run -v` の行は、run ID を含まない。ジョブの標準出力は、そのまま流れる。

### 実行前の検査と終了コード(`run`・`pipeline`・`validate`)
`Ctx::prepare` の順序(architecture.md を参照)で、定義の誤りと `command` の誤りは 64(標準エラーに、誤りの一覧)、パイプライン名の不一致は 64(定義の誤りの 1 つ)、知らないジョブID(`run`)と run ID の値の誤りは 68、ステートの誤りと食い違いは 65。いずれも、ジョブは実行しない。`validate` は定義と `command` の検査(`check_definition`)だけを行い、ステートを読まない。検査が通ったときは、何も出さない(`-v` のときだけ 1 行)。

## 定義ファイル(TOML)
`format_version = 1`、`[pipeline] name`・`description`(省略可)・`run_id_env`(省略可)、`[[job]]`(`id`、`command`(文字列/配列)、`success_exit_codes`、`warning_exit_codes`、`[[job.transitions]] on/next`)。`next` は後ろのジョブID か `@end`。`[pipeline]` の許可キーは `name`・`description`・`run_id_env` で、それ以外は `unknownKey`(64。`crates/core/src/definition/walk.rs:34`)。`run_id_env` は文字列で、`A-Za-z_` で始まり `A-Za-z0-9_` が続く 1 文字以上。違えば定義の誤り(64。`validate` も同じ)。旧キー `[[jobs]]` は定義の誤り(`README.md`)。詳細は `README.md`。

## ステートファイル(TOML。`format_version = 1`)
最上位キー(10 個。`crates/runtime/src/state/reader.rs` の `TOP_KEYS`): `format_version`、`run_id`(run ID つきのときだけ書く)、`definition_key`、`definition_path`、`pipeline_name`、`job_order`、`job`(`[[job]]` の配列。キーは `id`・`status`・`exit_code`・`skip_reason`)、`resume_job_id`(省略可)、`end_reached`、`completed`。知らないキーは `corrupt`(65)。ファイル名の run ID と中身の `run_id` が食い違えば `corrupt`(65。ほかのキーの検査より先)。run ID なしのファイルに `run_id` があっても、食い違い(`corrupt`)。1 MiB を超えると `tooLarge`。`completed = true` のまま残ったファイルは、読むと空として扱われる。

## ライブラリ API(内部)
- conductor-core: `load_definition(Option<&Path>, Option<&str>)`(第 2 引数は、`pipeline` の名前の照合用。`None` なら照合しない)、`parse_definition`、`decide_before_run`、`classify_and_decide`、`resume_point_on_interrupt`、`RunId::parse`、`is_valid_env_name`、`peek_run_id_env`。型: `PipelineDefinition { name, description, run_id_env, format_version, job }`、`PipelineState`(9 フィールド。`run_id: Option<String>` を含む)、`JobDefinition`、`Command`(`Line(String)` か `Argv(Vec<String>)`)、`Transition`、`RunId`、`RunIdError`。
- conductor-runtime: `run_job`、`check_command`(起動せずに command を検査)、`read_state(&Path, Option<&RunId>)`、`write_state(&Path, Option<&RunId>, &PipelineState, Durability)`、`check_consistency`、`reset_state(&Path, Option<&RunId>)`、`reset_all_states(&Path) -> ResetAllReport { removed, kept }`、`initial_state`、`apply_outcome`、`apply_skip`、`apply_interrupt`。型: `StateLocation`、`StateError`・`StateErrorKind`。
- conductor-converter: `convert`、`convert_text`。
- conductor-cli: `run_cli`、`run_cli_with`、`parse_args`、`install_panic_hook`、`Runtime`(9 メソッド)・`Sink`・`Env`・`Signals` の各 trait と本番の実装、`CommandKind`・`CliCommand`(`run_id`・`all` を含む)・`ParsedArgs`・`UsageError`。
