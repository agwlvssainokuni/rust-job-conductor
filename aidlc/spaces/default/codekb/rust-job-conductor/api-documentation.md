# API Documentation

## CLI(外部 API)
```text
conductor run [-v] [--config <パス>] <ジョブID>
conductor pipeline [-v] [--config <パス>] <パイプライン名>
conductor watch [--config <パス>]
conductor reset [--config <パス>]
conductor validate [-v] [--config <パス>] [<パイプライン名>]
conductor convert <ファイル> --to <yaml|toml>
conductor --help | --version
```
- 既定の定義は実行ディレクトリの `conductor.toml`。ステートは定義と同じディレクトリの `<定義名>.state`(`StateLocation::of`。`crates/runtime/src/types.rs:109`)。
- 終了コード: 0 成功/警告/スキップ、ジョブの値(エラー)、1(成功コード外の 0)、64 定義(`command` の誤りを含む。`validate` の失敗も)、65 ステート、66 拒否、67 変換、68 使い方(知らないジョブIDを含む)、70 内部、127 起動失敗、128+番号 シグナル。全 OS で 0〜255 に丸める。
- 出力: 既定では conductor 自身の出力なし。エラー時のみ標準エラー。`-v` で結果と遷移を標準エラーへ(`validate` は成功時に 1 行)。言語は `LC_ALL`/`LC_MESSAGES`/`LANG` の `ja` で日本語。色は UNIX 系の端末で `NO_COLOR` 未設定の `watch` だけ。

### 引数の文法(`crates/cli/src/args.rs`)
| コマンド | 位置引数 | `-v` | `--config` | `--to` |
|----------|----------|------|------------|--------|
| `run` | ジョブID(1) | 可 | 可 | 不可 |
| `pipeline` | パイプライン名(1) | 可 | 可 | 不可 |
| `watch` | なし | 不可 | 可 | 不可 |
| `reset` | なし | 不可 | 可 | 不可 |
| `validate` | パイプライン名(0 か 1) | 可 | 可 | 不可 |
| `convert` | 入力ファイル(1) | 不可 | 不可 | 必須(`yaml` か `toml`) |

- `--config path` と `--config=path` を受け付ける(値を取るオプションは `lexopt` の `parser.value()`。`args.rs:98`)。`--` はオプションの終わり。`-v`、`--config`、`--to` の重複は `invalidValue`。`--help`・`-h`・`--version`・`-V` は、ほかの誤りより先に処理する(0 で終わる)。
- 使い方の誤りは 68。種類は 8 つ(`UnknownCommand`、`MissingArgument`、`UnexpectedArgument`、`UnknownOption`、`OptionNotAllowed`、`MissingValue`、`InvalidValue`、`UnknownJob`)。表示は `Usage` の固定文+使い方の全文(標準エラー)。種類の全部は `args_grammar.rs` が網羅を固定する。
- 使い方の文言は `crates/cli/src/report/messages.rs:57-92` の `USAGE`(英語・日本語)。`README.md` の「使い方」(`:75-83`)とオプション表(`:94-103`)にも同じ内容がある。
- 環境変数は、`LC_ALL`・`LC_MESSAGES`・`LANG`(言語)と `NO_COLOR`(色)だけを読む(`Env::var`。`crates/cli/src/deps.rs:37`)。

### 実行前の検査と終了コード(`run`・`pipeline`・`validate`)
`Ctx::prepare` の順序(architecture.md を参照)で、定義の誤りと `command` の誤りは 64(標準エラーに、誤りの一覧)、パイプライン名の不一致は 64(定義の誤りの 1 つ)、知らないジョブID(`run`)は 68、ステートの誤りと食い違いは 65。いずれも、ジョブは実行しない。`validate` は定義と `command` の検査(`check_definition`)だけを行い、ステートを読まない。検査が通ったときは、何も出さない(`-v` のときだけ 1 行)。

## 定義ファイル(TOML)
`format_version = 1`、`[pipeline] name`・`description`(省略可)、`[[job]]`(`id`、`command`(文字列/配列)、`success_exit_codes`、`warning_exit_codes`、`[[job.transitions]] on/next`)。`next` は後ろのジョブID か `@end`。`[pipeline]` の許可キーは `name` と `description` だけで、それ以外は `unknownKey`(64。`crates/core/src/definition/walk.rs:33`)。旧キー `[[jobs]]` は定義の誤り(`README.md`)。詳細は `README.md`。

## ステートファイル(TOML。`format_version = 1`)
最上位キー(9 個。`crates/runtime/src/state/reader.rs:128` の `TOP_KEYS`): `format_version`、`definition_key`、`definition_path`、`pipeline_name`、`job_order`、`job`(`[[job]]` の配列。キーは `id`・`status`・`exit_code`・`skip_reason`)、`resume_job_id`(省略可)、`end_reached`、`completed`。知らないキーは `corrupt`(65)。1 MiB を超えると `tooLarge`。`completed = true` のまま残ったファイルは、読むと空として扱われる。

## ライブラリ API(内部)
- conductor-core: `load_definition(Option<&Path>, Option<&str>)`(第 2 引数は、`pipeline` の名前の照合用。`None` なら照合しない)、`parse_definition`(文字列から検証。ファイルに触れない)、`decide_before_run`、`classify_and_decide`、`resume_point_on_interrupt`。型: `PipelineDefinition { name, description, format_version, job }`、`PipelineState`(8 フィールド)、`JobDefinition`、`Command`(`Line(String)` か `Argv(Vec<String>)`)、`Transition`。
- conductor-runtime: `run_job`、`check_command`(起動せずに command を検査)、`read_state(&Path)`、`write_state(&Path, &PipelineState, Durability)`、`check_consistency`、`reset_state(&Path)`、`initial_state`、`apply_outcome`、`apply_skip`、`apply_interrupt`。型: `StateLocation`、`StateError`・`StateErrorKind`。
- conductor-converter: `convert`、`convert_text`。
- conductor-cli: `run_cli`、`run_cli_with`、`parse_args`、`install_panic_hook`、`Runtime`・`Sink`・`Env`・`Signals` の各 trait と本番の実装、`CommandKind`・`CliCommand`・`ParsedArgs`・`UsageError`。
