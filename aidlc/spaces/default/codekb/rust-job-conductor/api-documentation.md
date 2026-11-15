# API Documentation

外部 API(HTTP など)は無い。公開面は、CLI、定義ファイルの書式、Rust ライブラリの公開関数である。

## CLI(`conductor`)

| コマンド | 内容 |
|----------|------|
| `conductor run [-v] [--config <パス>] <ジョブID>` | 個別実行 |
| `conductor pipeline [-v] [--config <パス>] <パイプライン名>` | 一括実行 |
| `conductor watch [--config <パス>]` | ステートの表を標準出力に出す。定義ファイルは読まない |
| `conductor reset [--config <パス>]` | ステートを消す |
| `conductor convert <ファイル> --to <yaml\|toml>` | 変換結果を標準出力に 1 回で出す |
| `--help`, `--version` | 使い方、版 |

- 既定の定義は実行ディレクトリの `conductor.toml`。`--` はオプションの終わり。
- `-v` は標準エラーに、結果(`job=... result ...`)、遷移、スキップ、中断を出す。command の文字列は出さない。エラーのときだけ標準エラーに原因を出す。起動失敗は常に `cannot start command '<プログラム名>'` を出す。
- 終了コード: 0 / ジョブの値 / 1(成功コード外の 0)/ 64 / 65 / 66 / 67 / 68 / 70 / 127 / 128+番号(`crates/cli/src/exit.rs`、`README.md`)。

## 定義ファイル(TOML。`format_version = 1`)

| キー | 型 | 検査(`core/src/definition/walk.rs`) |
|------|----|--------------------------------------|
| `format_version` | 整数 | 必須。1 だけ |
| `[pipeline].name` | 文字列 | 必須。空は `emptyField` |
| `[pipeline].description` | 文字列 | 省略可 |
| `[[job]].id` | 文字列 | 必須。空・`@` 始まり・重複は不可 |
| `[[job]].command` | 文字列 | 必須。型が文字列でなければ `typeMismatch`(「`command` は文字列で書きます」)。空白だけなら `emptyField`。分割の検査(引用符の閉じ、コマンド名の有無)は実行前に cli が行い、64 |
| `[[job]].warning_exit_codes` | 整数の配列 | 0 を含まず、成功コードと重ならない。要素ごとに型を報告 |
| `[[job]].success_exit_codes` | 整数の配列 | 省略時 `[0]` |
| `[[job.transitions]]` | テーブルの配列 | `on` は `success`/`warning`、`next` は後ろのジョブか `@end` |

- 未知のキーは `unknownKey`。入れ子の深さは 32 まで、ファイルは 1 MiB まで。
- 誤りの種類(`DefinitionErrorKind`): fileMissing, unreadable, tooLarge, syntax, typeMismatch, unknownKey, versionMissing, versionUnsupported, unknownTarget, errorTransition, backwardOrSelfTarget, overlappingExitCodes, zeroAsWarning, pipelineNameMismatch, duplicateJobId, invalidJobId, noJobs, emptyField, duplicateTransition。

## ステートファイル(`<定義ファイル名>.state`、TOML、`format_version = 1`)

`PipelineState`(パイプライン名、定義のパス、ジョブの順序、ジョブごとの状態・終了コード・スキップ理由、再開位置、`end_reached`、`completed`)を持つ。**command は含まない**。1 MiB まで。

## Rust ライブラリの公開面

| クレート | 公開 | 契約 |
|----------|------|------|
| conductor-core | `load_definition(path, expected_name)`, `parse_definition(text, label, expected_name)`, `decide_before_run`, `classify_and_decide`, `resume_point_on_interrupt`, 型一式(`JobDefinition.command: String` を含む) | C1/C2 |
| conductor-runtime | `run_job(&JobDefinition, &InterruptSignal) -> JobExecution`, `check_command(&str) -> Result<CommandLine, CommandError>`, `read_state`, `write_state`, `reset_state`, `check_consistency`, `initial_state`, `apply_outcome`, `apply_skip`, `apply_interrupt`, `InterruptSignal`, `StateLocation`, `Durability` | C3 |
| conductor-converter | `convert(&ConversionRequest)`, `convert_text(text, target, label)`, `TargetFormat`, `ConversionError(Kind)`(`fileMissing`, `unreadable`, `tooLarge`, `syntax`, `unrepresentable`, `rootNotMap`) | C4 |
| conductor-cli | `run_cli`, `run_cli_with`, `parse_args`, trait `Runtime`/`Sink`/`Env`/`Signals` | - |

command に関わる公開シグネチャ: `check_command(&str)`、`Runtime::check_command(&self, &str)`、`JobDefinition.command`、`JobExecution.command`、`CommandLine { program, args }`(`runtime/src/types.rs`)、`CommandError { kind, reason }`。
