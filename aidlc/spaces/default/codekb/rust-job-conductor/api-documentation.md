# API の文書

`conductor` には、ネットワークの API(HTTP、gRPC など)は無い。外部の面は、(1) コマンドライン、(2) 定義ファイルの書式、(3) `.env` の書式、(4) ステートファイルの書式、(5) ジョブへ渡す環境変数、(6) 終了コードと標準出力・標準エラー、(7) ログのファイル、の 7 つ。内部の面は、4 クレートの公開の関数と型(8)と、cli の crate の中の口(9。この intent の対象)。この intent は外部の面を変えない。

## 1. コマンドライン(`crates/cli/src/args.rs`、`types.rs`)

```text
conductor run      [-v] [--config <パス>] [--run-id <ID>] [--remove-on-complete] [--dry-run [--assume <ジョブID>=<終了コード>]...] <ジョブID>
conductor pipeline [-v] [--config <パス>] [--run-id <ID>] [--remove-on-complete] [--dry-run [--assume <ジョブID>=<終了コード>]...] <パイプライン名>
conductor watch    [--config <パス>] [--run-id <ID>] [--follow [--interval <秒>]]
conductor reset    [--config <パス>] [--run-id <ID> | --all]
conductor validate [-v] [--config <パス>] [<パイプライン名>]
conductor graph    [-v] [--config <パス>] [--format <text|mermaid|dot>] [--state [--run-id <ID>]] [<パイプライン名>]
conductor init     [--config <パス>|-] [--force] [<パイプライン名>]
conductor convert  <ファイル> --to <yaml|toml>
conductor --help | -h | --version | -V
```

- コマンドは 8 つ(`CommandKind::ALL`、`types.rs`)。
- オプションは 13(`args.rs` の `Opt` と表 `OPTIONS`)。綴り、値の有無、許すコマンドは表の 1 か所に持つ。`--name value` と `--name=value` の両方を受け付ける。`--` 以降は位置引数。`-h`・`-V` は `--` より前ならほかの誤りより先に処理する。
- 「使えない」と「重複」の検査の順は表 `CHECKS`(25 行。仕様書 04 章 4.2 の 6・7 の順)。誤りは最初の 1 つだけを報告する。
- `--follow` は `watch` だけ。`--interval <秒>` は `--follow` と一緒のときだけで、1〜3600 の整数(数字だけ)、既定 2(`types.rs:215-219`)。
- `--format` は `text`(既定)・`mermaid`・`dot`。`--run-id` は空か `RunId` の規則。`--assume` は `--dry-run` と一緒のときだけ、0〜255、同じジョブIDの重複は誤り。
- 位置引数: `run`・`pipeline`・`convert` は必須、`validate`・`graph`・`init` は省略可、`watch`・`reset` は書けない(`operand_rule`)。
- 使い方の誤り(`UsageErrorKind`、8 種): `UnknownCommand`、`MissingArgument`、`UnexpectedArgument`、`UnknownOption`、`OptionNotAllowed`、`MissingValue`、`InvalidValue`、`UnknownJob`。表示に入力の値は含めない(対象はオプションの綴りか固定の語)。
- 使い方の全文(英日)は `report/messages.rs` の `USAGE`。
- `--version` は `conductor <版>`(今は `conductor 0.10.3`。期待値は `crates/cli/tests/binary_basic.rs:267`)。

### `watch --follow` の契約(`crates/cli/src/aux_commands.rs`、`crates/core/src/flow/activity.rs`)

| 項目 | 動き |
|------|------|
| 準備 | 最初にシグナルの準備(失敗は 70)。定義は検証せず、`run_id_env` だけをゆるく読む |
| 1 回分 | 合図を見る → `read_state_raw` → 有れば今の時刻で表を描く → 終わりの判定 → `--interval` 秒待つ |
| 描き方 | UNIX 系で標準出力が端末なら、1 回目から毎回 `ESC[H ESC[2J`。ほかは 2 回目から空の 1 行 |
| 終わり | 完了・止まった・消えた(2 回続けて無い)・中断で 0。`Unreadable` が 3 回続く、ほかの誤り、標準出力に書けないで 65 |

## 2. 定義ファイル(TOML。`crates/core/src/definition/`)

| 場所 | キー | 型と規則 |
|------|------|----------|
| 根 | `format_version` | 整数。1 だけ |
| 根 | `pipeline` | テーブル(1 つ) |
| 根 | `job` | テーブルの配列(1 つ以上) |
| `[pipeline]` | `name`、`description`、`run_id_env`、`remove_on_complete`、`env`、`env_file`、`retry`、`log_dir`、`unset_env` | 9 つ(`PIPELINE_KEYS`) |
| `[[job]]` | `id`、`command`、`warning_exit_codes`、`success_exit_codes`、`transitions`、`retry`、`env`、`env_file`、`unset_env` | 9 つ(`JOB_KEYS`) |
| 遷移 | `on`(`success`・`warning`)、`next`(後ろのジョブIDか `@end`) | 2 つ(`TRANSITION_KEYS`) |
| `[job.retry]`・`[pipeline.retry]` | `exit_codes`、`max`、`interval`、`multiplier`、`max_interval` | 5 つ(`RETRY_KEYS`。`walk/retry.rs`)。ジョブの `retry = false` で打ち消し |

- キーの表は private の `const`(`walk/mod.rs:47-70`、`walk/retry.rs:34`)。許されないキーは `unknownKey`。
- 誤りの種類は `DefinitionErrorKind` の 34 種(`crates/core/src/error.rs`)。表示は `<ファイル>:<行>: <種類>: <理由>`。誤りはすべて集め、位置の順に返す。
- 大きさの上限 1 MiB、入れ子 32 段。

## 3. `.env` の書式(`crates/core/src/envfile/parse.rs`)

- `KEY=VALUE` の行、`#` のコメント、空行、単引用符・二重引用符(エスケープは `\n`、`\r`、`\t`、`\"`、`\\` だけ)。展開しない。上限 1 MiB、10,000 項目。UTF-8 だけ。
- 誤りは `EnvFileErrorKind` の 14 種。表示は最大 20 件と残りの件数。値は表示しない。
- `required = false` のファイルが無いだけなら誤りでも警告でもない。

## 4. ステートファイル(TOML。`crates/runtime/src/state/`)

- 場所: 定義ファイルと同じディレクトリの `<定義ファイル名>.state`、run ID があれば `<定義ファイル名>.<run-id>.state`。
- 最上位のキー 10(`TOP_KEYS`)、ジョブのキー 5(`JOB_KEYS`)、試行のキー 2(`ATTEMPT_KEYS`。`started_at`、`ended_at`)。時刻は UTC の RFC 3339。表は `state/reader.rs` の private。
- 読み取りは厳格(知らないキー、型、状態と終了コードの不整合、試行の欠け・未完、時刻の書式、run ID の不一致は 65)。書き込みは原子的。
- `StateErrorKind`: `Unreadable`、`TooLarge`、`Corrupt`、`VersionMismatch`、`WriteFailed`、`RemoveFailed`。

## 5. ジョブへ渡す環境変数

- 親の環境を引き継ぎ、`unset_env` の名前を除いてから、`[pipeline].env_file` < `[pipeline.env]` < `[job].env_file` < `[job.env]` < 自動の変数、の順に足す。
- 自動の変数: `CONDUCTOR_PIPELINE`、`CONDUCTOR_JOB_ID`、`CONDUCTOR_RUN_ID`(run ID があるときだけ)、`CONDUCTOR_ATTEMPT`。

## 6. 終了コードと出力(`crates/cli/src/exit.rs`、`report/`)

| コード | 意味 |
|--------|------|
| 0 | 成功、警告、スキップ、補助のコマンドの成功、`watch --follow` の完了・止まった・消えた・中断 |
| ジョブの値 | エラーのジョブの終了コード(0〜255 に合わせる。エラーで 0 になるなら 1) |
| 1 | 成功コードに含まれない 0 |
| 64 | 定義の誤り(`.env`、command、`unset_env` の重なりを含む) |
| 65 | ステートの誤り、標準出力に書けない、`init` が作れない |
| 66 | 拒否(順序外) |
| 67 | 変換の誤り |
| 68 | 使い方の誤り |
| 69 | `init` の作る先が既にある |
| 70 | 内部の誤り(パニック、シグナルの準備の失敗) |
| 127 | 起動できなかった(ログのファイルを作れなかったときを含む) |
| 128 + 番号 | シグナルによる終了、または中断 |

- 表示は `DisplayEvent` の水準で出し先が決まる: Error(19 種)と Notice(5 種)は常に標準エラー、Verbose(5 種)は `-v` のときだけ標準エラー、Output(8 種)は標準出力(`watch` の表、`graph` の図、`--dry-run` の計画、`init -` のひな形、`--help`、`--version`)。`convert` の結果は流れが標準出力へ書く。
- 接頭辞: `conductor: error: `、`conductor: warning: `(`EnvWarnings`・`ResetAllKept`・ログの 2 つ。前の 2 つは水準が Error)、`conductor: retry: `、`[conductor] `(Verbose)。
- 色(ANSI)は `watch` の表だけで、UNIX 系の端末かつ `NO_COLOR` が未設定・空のとき。
- 表示の言語は `LC_ALL` → `LC_MESSAGES` → `LANG` の最初の空でない値で決める(`ja` で始まれば日本語)。`--version` とシグナルの準備の失敗は言語を判定しない。

## 7. ログのファイル(`crates/runtime/src/log.rs`)

- 名前: `<safe_log_stem(ジョブID)>.<format_compact(開始の時刻)>.<試行の番号>.log`。衝突は `-2`〜`-1000`。UNIX 系は 0600。`conductor` は消さない。
- 作れなければ起動せず 127。書き込みの失敗と終了後 2 秒の待ち切れは警告の 1 行。

## 8. ライブラリの公開の口(4 クレート)

| クレート | 公開の口 |
|----------|----------|
| conductor-core(`lib.rs`) | `load_definition`、`parse_definition`、判断(`decide_from_state`、`classify_and_decide`、`resume_point_on_interrupt`、`retry_decision`)、記録(`initial_state`、`apply_*`)、試行(`start_attempt`、`end_attempt`、`cancel_attempt`)、`pipeline_activity`、`check_consistency`、`.env`(`parse_env_file`、`merge_env`、`env_file_refs`、`summarize_errors`、`check_unset_conflicts`)、時刻、`graph`、`plan_pipeline`・`plan_run`、`RunId`、`escape_display`、誤りの種類 |
| conductor-runtime(`lib.rs`) | `check_command`、`run_job`、`now`、`open_log_file`・`safe_log_stem`、`wait_for`、`load_env_files`、ステートの読み書きと削除、`create_file` |
| conductor-converter(`lib.rs`) | `convert`、`convert_text` |
| conductor-cli(`lib.rs:107-111`) | `run_cli`、`run_cli_with`、`install_panic_hook`、trait `Runtime`・`Sink`・`Env`・`Signals` と本番の実装、`Deps`、`SignalError` |

`definition::walk` は core の private で、`walk::walk` だけを `definition/mod.rs:69` が呼ぶ。core の統合テストは `parse_definition`・`load_definition` を通して `walk` を試す。

## 9. cli の crate の中の口(この intent が保つもの)

cli の `args` と `report` は private のモジュール(`lib.rs:90`、`:99`)で、外のクレートと統合テストからは見えない。中の `pub`・`pub(crate)` は crate の中のパスで使われる。分けても、次のパスで今の名前が引けること(`pub use`)。

| パス | 項目 |
|------|------|
| `crate::args::parse_args` | `pub fn parse_args<I>(args: I) -> Result<ParsedArgs, UsageError>` |
| `crate::report::DisplayEvent` | 37 変種の列挙と `level()` |
| `crate::report::Reporter` | `new(env, verbose)`、`language()`、`color_enabled()`、`show(sink, event) -> bool`、`render(event) -> String` |
| `crate::report::StateContext` | 5 変種と `for_read(Option<&RunId>)` |
| `crate::report::ResultDetail` | 4 変種 |
| `crate::report::FramePrefix` | 3 変種と `for_frame(clear, n)`、`text()` |
| `crate::report::{escape_name, escape_full}` | `pub(crate)`。64 文字で切る/切らない |
| `crate::report::push_line`(子からは `super::push_line`) | `pub(crate)` |
| `crate::report::{messages, graph_text, graph_mermaid, graph_dot, plan_text, init_template}` | 子モジュール(`messages` は `pub(crate)`) |

使う場所の一覧は `component-inventory.md` の `cli::report`・`cli::args`。
