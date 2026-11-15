# API Documentation

外部 API(HTTP など)は無い。公開面は、CLI、定義ファイルの書式、ステートファイルの書式、Rust ライブラリの公開関数である。

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
| `[[job]].command` | 文字列、または文字列の配列 | 必須。文字列は空白だけなら `emptyField`、分割の検査(引用符の閉じ、コマンド名の有無)は実行前に cli が行い 64。配列は 1 要素 = 1 引数で、分割しない(先頭がプログラム)。そのほかの型は `typeMismatch` |
| `[[job]].warning_exit_codes` | 整数の配列 | 0 を含まず、成功コードと重ならない。要素ごとに型を報告 |
| `[[job]].success_exit_codes` | 整数の配列 | 省略時 `[0]` |
| `[[job.transitions]]` | テーブルの配列 | `on` は `success`/`warning`、`next` は後ろのジョブか `@end` |

- 旧書式の `[[jobs]]` は定義の誤り(64)になる(`README.md` L168、`crates/cli/tests/flow_run.rs`)。
- 未知のキーは `unknownKey`。入れ子の深さは 32 まで、ファイルは 1 MiB まで。
- 誤りの種類(`DefinitionErrorKind`): fileMissing, unreadable, tooLarge, syntax, typeMismatch, unknownKey, versionMissing, versionUnsupported, unknownTarget, errorTransition, backwardOrSelfTarget, overlappingExitCodes, zeroAsWarning, pipelineNameMismatch, duplicateJobId, invalidJobId, noJobs, emptyField, duplicateTransition。

## ステートファイル(`<定義ファイル名>.state`、TOML、`format_version = 1`)

`PipelineState`(パイプライン名、定義のパス、ジョブの順序、ジョブごとの状態・終了コード・スキップ理由、再開位置、`end_reached`、`completed`)を持つ。**command は含まない**。1 MiB まで。`README.md` には場所と動作だけが書かれ、キーの書式は書かれていない(書式の正は `crates/runtime/src/state/reader.rs`・`writer.rs`)。

### 現行の書式(`[[jobs]]` を使う)

```toml
format_version = 1
definition_key = "conductor.toml"
definition_path = "conductor.toml"
pipeline_name = "p"
job_order = ["a", "b"]
end_reached = false
completed = false
resume_job_id = "b"

[[jobs]]
id = "a"
status = "success"
exit_code = 0

[[jobs]]
id = "b"
status = "pending"
```

| キー | 型 | 必須 | 検査(`reader.rs`) |
|------|----|------|---------------------|
| `format_version` | 整数 | 必須 | 1 以外は `versionMismatch`、整数でなければ `corrupt`。最初に確認する |
| `definition_key`, `definition_path` | 文字列 | 必須 | 書き込みは同じ値(定義のパス)を両方に入れる。照合には使わない |
| `pipeline_name` | 文字列 | 必須 | 定義の名前と照合(`check_consistency`) |
| `job_order` | 文字列の配列 | 必須 | 定義の記述順と照合 |
| `jobs` | テーブルの配列 | 必須 | 各要素は `id`(必須)、`status`(必須。`pending/success/warning/error/skipped`)、`exit_code`(結果のあるジョブだけ)、`skip_reason`(`skipped` のときだけ。`byTransition/byEnd`)。ID の重複、`job_order` と並びの不一致は `corrupt` |
| `resume_job_id` | 文字列 | 省略可 | `job_order` に含まれること。`completed = true` と併存しない |
| `end_reached`, `completed` | 真偽値 | 必須 | `completed = true` のファイルは、空のステートとして読む |

- 未知のキー(トップとジョブの両方)は `corrupt`。失敗の種類(`StateErrorKind`): `unreadable`、`tooLarge`、`corrupt`、`versionMismatch`、`writeFailed`、`removeFailed`。表示は、種類ごとの固定の文とパスだけ。
- 書き込みは、一時ファイル `.<ステートファイル名>.tmp.<pid>.<時刻>.<連番>` に書いて `rename` で置き換える。

### 予定する変更(state-job-key。実装前)
`[[jobs]]` を `[[job]]` に改める。`format_version` は 1 のまま、旧形式は読まない(旧形式のファイルは、読み取りの未知のキー検査で `corrupt` になる見込み)。ほかのキーは変えない。

## Rust ライブラリの公開面

| クレート | 公開 | 契約 |
|----------|------|------|
| conductor-core | `load_definition(path, expected_name)`, `parse_definition(text, label, expected_name)`, `decide_before_run`, `classify_and_decide`, `resume_point_on_interrupt`, 型一式(`JobDefinition.command: Command` を含む) | C1/C2 |
| conductor-runtime | `run_job(&JobDefinition, &InterruptSignal) -> JobExecution`, `check_command(&Command) -> Result<CommandLine, CommandError>`, `read_state`, `write_state`, `reset_state`, `check_consistency`, `initial_state`, `apply_outcome`, `apply_skip`, `apply_interrupt`, `InterruptSignal`, `StateLocation`, `Durability` | C3 |
| conductor-converter | `convert(&ConversionRequest)`, `convert_text(text, target, label)`, `TargetFormat`, `ConversionError(Kind)`(`fileMissing`, `unreadable`, `tooLarge`, `syntax`, `unrepresentable`, `rootNotMap`) | C4 |
| conductor-cli | `run_cli`, `run_cli_with`, `parse_args`, trait `Runtime`/`Sink`/`Env`/`Signals` | - |

runtime のステート関連のシグネチャ(`lib.rs` の表):
- `read_state(&Path) -> Result<Option<PipelineState>, StateError>`(`None` は空。ファイルなし・長さ 0・`completed`)
- `write_state(&Path, &PipelineState, Durability) -> Result<(), StateError>`
- `reset_state(&Path) -> Result<(), StateError>`(読まず、検査しない)
- `check_consistency(&PipelineState, &PipelineDefinition) -> ConsistencyResult`
- `initial_state(&PipelineDefinition, &Path) -> PipelineState`、`apply_outcome`、`apply_skip`、`apply_interrupt`(純粋)

command に関わる公開シグネチャ: `check_command(&Command)`、`Runtime::check_command(&self, &Command)`(cli の trait)、`JobDefinition.command: Command`、`JobExecution.command`、`CommandLine { program, args }`(`runtime/src/types.rs`)、`CommandError { kind, reason }`、`Command::program_name`。
