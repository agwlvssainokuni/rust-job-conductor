# 10. エラーと終了コード

実装: `crates/core/src/error.rs`, `crates/runtime/src/error.rs`, `crates/converter/src/error.rs`, `crates/cli/src/{types,exit,ctx,run,pipeline,dry_run,aux_commands,panic_hook,deps}.rs`, `crates/core/src/flow/classify.rs`。

## 1. 終了コードの一覧(全件)

| 終了コード | 定数 | 意味 | 主な発生箇所 |
|------------|------|------|--------------|
| 0 | `EXIT_OK` | 成功。ジョブの成功・警告、スキップ(実行済み・遷移・`@end`)、`watch`・`reset`・`validate`・`convert` の成功、`--help`・`--version`。`reset --all` の警告(読めなくて残したファイル)も 0。v0.9.0 以降: `graph`・`init` の成功、`--dry-run` の計画を出せたとき(模擬の結果がエラーでの停止でも 0)、`validate` の `.env` の重複キーの警告 | 全般 |
| 1 | (`EXIT_UNLISTED_ZERO`) | ジョブが終了コード 0 で終わったが、`success_exit_codes` に 0 がない。または、ジョブの終了コードがエラーで、下位 8 ビットが 0 になる(例: 256) | `classify`, `exit_code_for` |
| ジョブの値(1〜255) | | エラーになったジョブの終了コードを、そのまま。範囲外は下位 8 ビット(0 になるなら 1) | `exit_code_for` |
| 64 | `EXIT_DEFINITION` | 定義の誤り: ファイルがない・読めない・大きすぎる・文法・検証の誤り(パイプライン名の不一致、`run_id_env` や `remove_on_complete` の型・形の誤り、`[job.retry]` の誤りを含む)、`command` の誤り(引用符が閉じていない・空)。`validate` の失敗も 64。v0.9.0 以降: `env`・`env_file` の誤り(`reservedEnvName`・`envNameCaseConflict`・`invalidEnvValue` ほか)と、`.env` が無い・読めない・大きすぎる・UTF-8 でない・書式の誤り(`EnvFileErrorKind` の 13 種)。`graph`・`--dry-run` の定義と `.env` の誤りも 64 | `Ctx::check_definition`, `Ctx::load_env_files` |
| 65 | `EXIT_STATE` | ステートの誤り: 読めない・大きすぎる・壊れている・版の不一致・定義との食い違い・中身の `run_id` とファイル名の食い違い・書き込みの失敗・削除(初期化)の失敗。`reset`・`reset --all` で消すのに失敗。`watch` が標準出力に書けない。`pipeline` が完了を判定できない。v0.9.0 以降: `graph`・`--dry-run` の `--state`・ステートの読み取りの誤りと、標準出力に書けないとき。`init` がファイルを作れない(親のディレクトリが無い・書く権限がない・作る先がディレクトリ・そのほかの入出力の誤り)・ひな形を標準出力に書けない | `state_error`, `watch`, `graph`, `dry_run`, `init` |
| 66 | `EXIT_REJECTED` | 拒否(順序外など。`run` で前のジョブが終わっていない、スキップ対象、`@end` より後ろ、など)。v0.9.0 以降は、`--dry-run` で、本番なら拒否されるとき | `run`, `pipeline`, `dry_run` |
| 67 | `EXIT_CONVERSION` | 変換の誤り。`convert` が標準出力に書けない | `convert` |
| 68 | `EXIT_USAGE` | 使い方の誤り(知らないコマンド、引数の不足、余分な引数、知らないオプション、使えないオプション、値の欠落・誤り、知らないジョブID、run ID の値の誤り〔引数・環境変数とも〕、重複、`--all` と `--run-id` の併用)。v0.9.0 以降: `--assume` の誤り(書式・範囲外・重複・定義にないジョブID)、`--dry-run` なしの `--assume`、`--format` の値の誤り、`graph` 以外の `--format`・`--state`、`--state` なしの `graph --run-id`、`init` の名前が空、`init` に許されないオプション、`init` 以外への `--force`、`run`・`pipeline` 以外の `--dry-run` | 引数の解釈、`enter_flow`、`prepare_run`・`prepare_pipeline`・`prepare_checked`(`require_job`・`require_known`・`definition_run_id`)、`watch`/`reset` |
| 69 | `EXIT_EXISTS` | (v0.9.0 以降)`init` の作る先に、ファイルが既にある(`--force` なし。何も変えない) | `init` |
| 70 | `EXIT_INTERNAL` | 内部の誤り: パニック、シグナルの準備の失敗 | パニックのフック、`start_signals`(`prepare_run`・`prepare_pipeline`) |
| 127 | `EXIT_START_FAILURE` | ジョブを起動できなかった(実行ファイルがない・実行権限がない、など。終了の状態を取得できない場合も安全側で起動失敗扱い) | `classify` |
| 128 + 番号 | `EXIT_SIGNAL_BASE` | ジョブがシグナルで終了した、または `conductor` が中断された(例: SIGHUP = 129、SIGINT = 130、SIGQUIT = 131、SIGTERM = 143)。リトライの待機中の中断も含む | `classify`, `signal_exit_code` |

- 64〜70(69 を含む)は、ジョブ自身の終了コードと重なりうる。区別は、標準エラーの `conductor: error: ...` の行で行う(70 は固定の英語の文)。
- 終了コードの範囲(0〜255)への合わせ方は 4 章。

## 2. 終了コードの決定順序

`run` / `pipeline` は、次の順に検査し、最初に当てはまった時点で終わる(それ以降は行わない)。

| 順 | 検査 | 失敗時 |
|----|------|--------|
| 0 | 引数の解釈(`-h`/`-V` は先に 0。それ以外の使い方の誤り) | 68 |
| 1 | シグナルの監視の準備(定義の検査より前)。「すでに登録済み」は成功と同じに扱う | 70 |
| 2 | 定義の読み込みと検証(パイプライン名の照合を含む。誤りは全部まとめて 1 行ずつ表示) | 64 |
| 3 | 全ジョブの `command` の検査(誤りのあるジョブごとに 1 行) | 64 |
| 4 | `run` のジョブIDが定義にあるか | 68 |
| 5 | run ID の決定(`--run-id` または環境変数。規則違反) | 68 |
| 5a | (v0.9.0 以降)`.env` の読み取り(`pipeline` は全ジョブ分、`run` はそのジョブ分と `[pipeline]` 分。誤りは全部集め、先頭 20 件と「ほか N 件」を表示) | 64(ジョブは起動せず、ステートに触れない) |
| 6 | ステートの読み込み | 65 |
| 7 | ステートと定義の照合 | 65 |
| 8 | 起動前の判断 | 拒否は 66。スキップは 0(`run` は 0 で終わる。`pipeline` は続ける) |
| 9 | (実行のとき)初期ステートの書き込み | 65(ジョブは起動していない) |
| 10 | ジョブの実行(リトライ込み)、結果の分類 | 結果から決まる(3 章) |
| 11 | 結果のステートの書き込み | 65(ジョブは実行済み。ジョブの終了コードより優先) |
| 12 | 完了で、削除が求められていれば、ステートの削除 | 65 |

補足:

- 検査やステートの確認の途中でシグナルを受け取っても、検査は最後まで進む。定義の誤り(64)、ステートの誤り(65)、拒否(66)、スキップ(0)の終了コードが、中断の終了コードより優先する。
- `watch`: run ID の決定(68)→ ステートの読み込み(65)→ 表の出力(書けなければ 65)。成功は 0。定義は検証しない。シグナルの準備はしない。
- `reset`: run ID の決定(68。`--all` のときは決定しない)→ 削除(65)。成功は 0。
- `validate`: 定義(64)→ `command`(64)→ (v0.9.0 以降)`.env`(64。重複キーの警告は 0)。成功は 0。ステートに触れない。シグナルの準備はしない。
- `graph`(v0.9.0 以降): 定義(64)→ `command`(64)→ `.env`(64)→(`--state` のとき)run ID の決定(68)→ ステートの読み取り(65)→ 照合(65)→ 標準出力への書き込み(書けなければ 65)。成功は 0。ステートに書かない。シグナルの準備はしない。
- `--dry-run`(v0.9.0 以降): 定義(64)→ `command`(64)→ `run` のジョブID(68)→ `--assume` のジョブID(68)→ run ID の決定(68)→ `.env`(64)→ ステートの読み込み(65)→ 照合(65)→ 計画(拒否は 66)→ 標準出力への書き込み(書けなければ 65)。成功は 0。ステートに書かない。シグナルの準備はしない。
- `init`(v0.9.0 以降): 引数の解釈(68)→ ひな形の作成: 既にある 69、作れない 65、標準出力に書けない 65。成功は 0。定義・`.env`・ステートを読まない。シグナルの準備はしない。
- `convert`: 変換(67)→ 標準出力への書き込み(書けなければ 67)。成功は 0。
- `pipeline` の特有: 実行したジョブがエラー(または分類なし)なら、そのジョブの終了コードで終わる。ジョブのあいだに中断の合図があれば `128 + 番号`。完了なら 0。完了を判定できなければ 65。
- ステートの書き込みに失敗すると、ジョブの終了コードではなく 65 で終わり、「ジョブは実行されました」と表示する(at-least-once)。起動前の失敗(初期ステートの書き込みなど)は「ジョブは起動していません」。

## 3. ジョブの結果から終了コードへ(`classify`)

最初に当てはまるものを採る。

| 順 | 状況 | 分類 | 報告する終了コード | 備考 |
|----|------|------|--------------------|------|
| 1 | 起動しておらず、中断された | 分類なし | `128 + 番号` | エラーとして記録しない。再開位置はそのジョブ |
| 2 | 起動しておらず、中断でもない(起こらない想定) | エラー | 127 | 起動失敗と同じ扱い |
| 3 | 起動済みで中断された | エラー | `128 + 番号` | ジョブ自身の終了コードは見ない |
| 4 | 起動失敗 | エラー | 127 | |
| 5 | ジョブがシグナルで終了 | エラー | `128 + シグナル番号` | |
| 6 | 起動して終わったが、終了コードがない | エラー | 1 | `conductor` のエラー(`success_exit_codes` に含まれない 0 と同じ表示) |
| 7 | 成功コード(`success_exit_codes`、省略時は `[0]`)に含まれる | 成功 | 0 | |
| 8 | 警告コード(`warning_exit_codes`)に含まれる | 警告 | 0 | |
| 9 | 終了コードが 0 だが成功コードにない | エラー | 1 | `conductor` のエラー。`job '<ID>': exit code 0 is not listed in success_exit_codes` を常に表示 |
| 10 | それ以外 | エラー | ジョブの終了コード | 範囲を 0〜255 に合わせる |

プロセスの終了コード = `exit_code_for(報告する終了コード, エラーか)`(`run`)。`pipeline` は、エラーまたは分類なしのジョブで止まるとき同じ式で、完了なら 0。

## 4. 終了コードの範囲合わせ(`exit.rs`)

`exit_code_for(reported, is_error)`:

| 入力 | 出力 |
|------|------|
| `0 <= reported <= 255` | `reported` |
| 範囲外 | `reported & 0xFF`(負の値も 2 の補数で下位 8 ビット)。ただし、その結果が 0 で `is_error` なら 1(エラーを成功に見せない) |

例: 259 → 3、511 → 255、-1 → 255、-2 → 254、`0xC0000005`(Windows の例外コードを i32 にした値)→ 5。256・512・`i32::MIN`・-256 は、エラーなら 1、エラーでなければ 0。

`signal_exit_code(n)` = `exit_code_for(128 + n, true)`(加算は飽和)。15 → 143、2 → 130、1 → 129、3 → 131、300 → 172。

`-v` の結果行には、元の値(`exit code <C>`)と、`conductor` の終了コードが異なるとき(`result=error` のみ)`, conductor exits with <E>` を示す。

## 5. シグナルと中断

- UNIX 系: `SIGHUP`(1)・`SIGINT`(2)・`SIGQUIT`(3)・`SIGTERM`(15)を、専用スレッドで監視し、最初の番号を中断の合図に書く。ほぼ同時に複数届くと、番号の小さい方が「最初」になりうる。
- Windows: Ctrl+C(番号 2)だけ。
- `run`・`pipeline` だけがシグナルを準備する。`watch`・`reset`・`validate`・`convert` は既定の動作で終わる(v0.9.0 以降の `graph`・`init`、`--dry-run` も同じ)。
- 起動済みのジョブが中断されると、`128 + 番号`、エラーとして記録する。起動前、またはジョブのあいだ、リトライの待機中に中断されたときは、エラーとして記録せず、再開位置を記録して `128 + 番号` で終わる。
- 準備の失敗(`SignalError::Failed`)は、`conductor: error: failed to set up signal handling` を出して 70。`AlreadyRegistered`(プロセスで 2 回目)は成功と同じに扱う。

## 6. パニックのフック

- `main` の最初に、パニックのフックを設定する。
- パニックすると、フックが標準エラーに、固定の英語の文(ロケールを判定しない)を 1 行書き、flush し、終了コード 70 で `std::process::exit` を呼ぶ。書き込みに失敗しても無視して 70 で終わる。

```text
conductor: error: internal error
```

- 外部の入力(引数、環境変数、ファイル、シグナル)に対してパニックしない設計(`#![forbid(unsafe_code)]`)。
- ジョブの実行中にパニックすると、子プロセスの終了を待たずに終わり、ジョブが孤児として残りうる。ステートは直前の完全な内容のままで、次の実行はそのジョブを再実行しうる。
- フックの `exit` を差し替えられる(`install_panic_hook_with`)のはテストのため。`exit` が戻ると、パニックの伝播は通常どおり続く。リリースの実行ファイルに、パニックを起こす専用のコマンドはない。

## 7. エラー型の一覧

### 7.1 `conductor-core`

| 型 | 内容 |
|----|------|
| `DefinitionErrorKind`(33 種) | 識別子(`code()`)・日本語の文(`Display`)・英語の文は 08 章 4.3 に全件 |
| `EnvFileErrorKind`(13 種。v0.9.0 以降) | `fileMissing`, `unreadable`, `tooLarge`, `notUtf8`, `tooManyEntries`, `invalidName`, `reservedName`, `caseConflict`, `missingEquals`, `unterminatedQuote`, `invalidEscape`, `trailingGarbage`, `nulInValue`。すべて終了コード 64。識別子・条件は 03 の 6.7、文言は 08 の 4.12。`ALL: [EnvFileErrorKind; 13]`、`code()`、`reason()` |
| `EnvFileError { file, line: Option<usize>, kind }` と `reason()`(同。理由は種類の固定の文。v0.9.6 までは `reason` をフィールドに持った) | `Display` は `<ファイル>:<行>: <種類の文>: <理由>`(行がなければ `<ファイル>: …`)。`file` は制御文字をエスケープして 256 文字で切る(ライブラリ側の表示のみ)。CLI は自前の形式で、言語に従った種類の文と理由を、`file` を切らずに表示する(08 の 4.12)。理由は固定の文で、値を含めない |
| `EnvWarning { file, line, name, kind: DuplicateKey }`(同) | 誤りではない(重複キー)。`validate` だけが表示する |
| `DefinitionError { file, line: Option<usize>, kind, reason }` | `Display` は `<ファイル>:<行>: <種類の文>: <原因>`(行がなければ `<ファイル>: <種類の文>: <原因>`)。`Display` では、`file` を制御文字のエスケープのうえ 256 文字で切る(ライブラリ側の表示のみ)。CLI は `Display` を使わず、自前の形式で、`file` をエスケープだけして切らずに表示する(08 章)。`reason` は core が作る日本語の固定の文。キー名は `` `キー名` `` の形で入り、64 文字で切る |
| `RunIdError`(4 種) | `Empty`, `TooLong`, `InvalidChar`, `LeadingDot`。CLI では、すべて `invalid value`(68)にまとめられ、値は含めない |

`DefinitionErrorKind` の一覧(識別子順):
`fileMissing`, `unreadable`, `tooLarge`, `syntax`, `typeMismatch`, `unknownKey`, `versionMissing`, `versionUnsupported`, `unknownTarget`, `errorTransition`, `backwardOrSelfTarget`, `overlappingExitCodes`, `zeroAsWarning`, `pipelineNameMismatch`, `duplicateJobId`, `invalidJobId`, `noJobs`, `emptyField`, `duplicateTransition`, `invalidEnvName`, `retryNotTable`, `retryMissingKey`, `retryExitCodesEmpty`, `retryExitCodeInvalid`, `retryExitCodesOverlap`, `retryMaxInvalid`, `retryDurationInvalid`, `retryBackoffIncomplete`, `retryMultiplierInvalid`, `retryMaxIntervalTooSmall`, `reservedEnvName`, `envNameCaseConflict`, `invalidEnvValue`(最後の 3 つは v0.9.0 以降)。すべて終了コード 64。

`reason`(原因の文)の全件は、定義の検証の仕様に属し、この文書では列挙しない(未確認)。

### 7.2 `conductor-runtime`

| 型 | 内容 |
|----|------|
| `CommandErrorKind`(2 種) | `UnclosedQuote`(原因: `引用符が閉じていません`)、`Empty`(原因: `コマンド名がありません`)。CLI の表示は、種類の文だけ(08 章 4.4)。終了コード 64 |
| `CommandError { kind, reason }` | `Display` は `reason` だけ |
| `StateErrorKind`(6 種) | `Unreadable`, `TooLarge`, `Corrupt`, `VersionMismatch`, `WriteFailed`, `RemoveFailed`。終了コード 65 |
| `StateError { path, kind, reason }` | `Display`(runtime 内)は `<種類の日本語の文>: <パス> (<原因>)`。CLI は自前の形式で表示する(08 章 4.5) |
| `MismatchKind`(3 種) | `PipelineName`, `JobIds`, `JobOrder`(定義との食い違い。終了コード 65)。定めるのは `conductor-core`(`consistency.rs`)で、`conductor-runtime` が再公開する |
| `CreateError`(5 種。v0.9.0 以降) | `AlreadyExists`(終了コード 69)、`ParentMissing`, `IsDirectory`, `PermissionDenied`, `OtherIo`(終了コード 65)。`ALL: [CreateError; 5]`。`init` が使う(`create_file`)。文言は 08 の 4.13、作成の規則は 06 の 13 章。OS のエラーの文は持たない |
| `CreateOutcome`(同) | `Created`, `Replaced`(成功。誤りではない) |

`StateError.reason`(固定の日本語。言語を切り替えない)と、それを返す種類:

| 原因の文 | 種類 | 場面 |
|----------|------|------|
| `開く、または読み取ることができません` | `Unreadable` | 読み込み、`reset --all` で読めなかった |
| `大きさが上限(1 MiB)を超えています` | `TooLarge` | 読み込み |
| `UTF-8 の文字列ではありません` | `Corrupt` | 読み込み |
| `TOML として読めません` | `Corrupt` | 読み込み |
| `format_version がありません` | `Corrupt` | 読み込み |
| `format_version が整数ではありません` | `Corrupt` | 読み込み |
| `対応していない版です(reset が必要です)` | `VersionMismatch` | 読み込み |
| `知らないキーがあります` | `Corrupt` | 読み込み |
| `必須のキーがありません` | `Corrupt` | 読み込み |
| `値の型が誤っています` | `Corrupt` | 読み込み |
| `status の値が誤っています` | `Corrupt` | 読み込み |
| `skip_reason の値が誤っています` | `Corrupt` | 読み込み |
| `skip_reason が status と合いません` | `Corrupt` | 読み込み |
| `exit_code が status と合いません` | `Corrupt` | 読み込み |
| `ジョブIDが重複しています` | `Corrupt` | 読み込み |
| `job の並びが job_order と一致しません` | `Corrupt` | 読み込み |
| `resume_job_id が job_order にありません` | `Corrupt` | 読み込み |
| `completed と resume_job_id が両立しません` | `Corrupt` | 読み込み |
| `run_id がファイル名の run ID と一致しません` | `Corrupt` | 読み込み(`reset --all` では「別の定義のステート」として扱い、消さない) |
| `書き込みに失敗しました` | `WriteFailed` | 書き込み |
| `内容を TOML にできません` | `WriteFailed` | 書き込み |
| `削除に失敗しました` | `RemoveFailed` | `reset`、`reset --all`、完了時の削除 |

(読み込みの順序と判定の詳細は runtime の実装。この表は `reader.rs`・`writer.rs`・`reset_all.rs` の対応による。)

### 7.3 `conductor-converter`

| 型 | 内容 |
|----|------|
| `ConversionErrorKind`(6 種) | `fileMissing`, `unreadable`, `tooLarge`, `syntax`, `unrepresentable`, `rootNotMap`。終了コード 67 |
| `ConversionError { file, line, key_path, kind, reason }` | `Display`(converter 内)は `<ファイル>:<行>: <種類の文>: <原因>`(行なしは `<ファイル>: …`)に、`key_path` があれば `(場所: <位置>)` を続ける。CLI は 08 章 4.8 の形式で表示する |

`reason`(固定の日本語。言語を切り替えない):

| 原因の文 | 場面 |
|----------|------|
| `指定の場所にファイルがありません` | 入力なし |
| `読み取りの権限がありません` | 権限 |
| `読み取りに失敗しました` | 読み取りの失敗 |
| `通常のファイルではありません` | ディレクトリなど |
| `UTF-8 として読めません` | 文字コード |
| `入力が上限(1 MiB)を超えています` | 入力の大きさ |
| `展開後の要素数が上限(100000)を超えています` | アンカーの展開 |
| `展開後の文字の大きさが上限(16 MiB)を超えています` | アンカーの展開 |
| `文法の誤りです` | 文法 |
| `キーが重複しています` | キーの重複 |
| `アンカーが自分自身を含んでいます` | 再帰するアンカー |
| `定義されていないアンカーを参照しています` | 未定義のアンカー |
| `入れ子が深すぎます(上限 32)` | 入れ子(数は `raw::MAX_DEPTH` と同じ。const の assert で縛る) |
| `展開後の入れ子が深すぎます(上限 32)` | 入れ子(同) |
| `複数の文書は表せません` | YAML の複数文書 |
| `null は表せません` | `null` |
| `文字列でないキーは表せません` | 非文字列キー |
| `タグつきの値は表せません` | YAML のタグ |
| `日時の値は表せません` | 日時 |
| `数値が表せる範囲を超えています` | 数値 |
| `マージキーの値がマップではありません` | `<<` |
| `最上位がマップではありません` | 最上位 |
| `文書が空です` | 空 |
| `変換後の形式として書き出せません` | 出力 |

`key_path` の形式: キーは `.` で連結(64 文字で切る)、添字は `[i]`、非文字列キーは `(非文字列キー)`。例: `job[2].transitions[0]`。

(どの `kind` とどの `reason` が組になるかの完全な対応は、converter の実装にある。未確認。)

### 7.4 `conductor-cli`

| 型 | 内容 |
|----|------|
| `UsageErrorKind`(8 種) | `UnknownCommand`, `MissingArgument`, `UnexpectedArgument`, `UnknownOption`, `OptionNotAllowed`, `MissingValue`, `InvalidValue`, `UnknownJob`。すべて終了コード 68。文言は 08 章 4.2 |
| `UsageError { kind, subject: Option<String> }` | `subject` は、誤りの対象の語(知らないコマンド、オプション名、ジョブID、`run ID (environment variable)` など)。入力の値は、それ以外には含めない(`--run-id` の値、環境変数の値は含めない) |
| `SignalError`(2 種) | `AlreadyRegistered`(成功と同じ扱い)、`Failed`(70) |

`UsageError` の例(`subject` と発生条件):

| 発生条件 | 種類 | `subject` |
|----------|------|-----------|
| 位置引数なし | `MissingArgument` | なし |
| コマンド名が未知(空文字も) | `UnknownCommand` | その語 |
| 文法にないオプション、`--bogus=x` | `UnknownOption` | `--bogus`、`-x` など |
| そのコマンドで使えないオプション | `OptionNotAllowed` | `-v`, `--config`, `--to`, `--run-id`, `--all`, `--remove-on-complete`, `--dry-run`, `--force`, `--assume`, `--format`, `--state`(後ろの 5 つは v0.9.0 以降) |
| 値のないオプション | `MissingValue` | `--config`, `--run-id`, `--to`, `--format`, `--assume` |
| 重複、`--verbose=x` など、`--all` と `--run-id` の併用、`--run-id` の値の規則違反 | `InvalidValue` | オプション名(`-v`, `--config`, …、併用は `--all`。v0.9.0 以降は `--dry-run`, `--force`, `--format`, `--state` も) |
| `--assume` の値の誤り(`=` がない、整数でない、範囲外、ジョブID の重複)、`--format` の値の誤り(v0.9.0 以降) | `InvalidValue` | `--assume`、`--format`(値は含めない) |
| `init` のパイプライン名が空・空白だけ(v0.9.0 以降) | `InvalidValue` | `pipeline name`(値は含めない) |
| `--to` の値が `yaml`/`toml` 以外(非 UTF-8 を含む) | `InvalidValue` | `--to`(値は含めない) |
| 非 UTF-8 の位置引数 | `InvalidValue` | lossy 変換した文字列 |
| 環境変数の run ID の規則違反 | `InvalidValue` | `run ID (environment variable)` |
| 余分な位置引数 | `UnexpectedArgument` | その語 |
| `convert` で `--to` なし | `MissingArgument` | `--to` |
| `run` のジョブIDが定義にない | `UnknownJob` | そのジョブID |
| `--assume` のジョブIDが定義にない(v0.9.0 以降) | `UnknownJob` | なし(ジョブIDを含めない) |

## 8. 標準エラーの出力と終了コードの例

```text
$ conductor validate            # command の誤り
conductor: error: d.toml: job 'z': command has an unclosed quote
→ 64

$ conductor pipeline wrongname
conductor: error: c.toml:3: pipeline name does not match the definition: <原因(日本語)>
→ 64

$ conductor run extract         # 壊れたステート
conductor: error: c.toml.state: state file is corrupt: TOML として読めません; run `conductor reset` to discard the state
→ 65

$ conductor run load            # 順序外
conductor: error: job 'load': cannot run this job now: earlier jobs have not finished (out of order)
→ 66

$ conductor convert nofile --to toml
conductor: error: nofile: input file does not exist: 指定の場所にファイルがありません
→ 67

$ conductor run nojob
conductor: error: job is not defined in the definition file: 'nojob'
(空行と使い方)
→ 68

$ conductor pipeline p          # step3 の実行ファイルがない
conductor: error: job 'step3': cannot start command 'nonexistent_prog_x'
→ 127

$ (ジョブが SIGKILL で終了)
→ 137

$ conductor pipeline p          # [pipeline].env_file の .env に 3 行目の誤り(v0.9.0 以降)
conductor: error: x.env:3: not in KEY=VALUE form: write a line as `KEY=VALUE`
→ 64(ジョブは 1 つも実行せず、ステートに触れない)

$ conductor init                # conductor.toml が既にある(v0.9.0 以降)
conductor: error: conductor.toml already exists (use --force to overwrite)
→ 69

$ conductor init --config /no/such/dir/c.toml
conductor: error: cannot create /no/such/dir/c.toml: the parent directory does not exist
→ 65
```

## 9. 実装と README の差

| 項目 | README | 実装 |
|------|--------|------|
| 終了コード 1 | 「成功コードに含まれない `0` で終わったジョブ」 | それに加え、終了コードがないまま終わったジョブ(起こらない想定)と、ジョブの終了コードがエラーで下位 8 ビットが 0 になる場合も 1 |
| 終了コード 70 | 「パニック、シグナルの準備の失敗」 | 一致 |
| 127 | 「ジョブを起動できなかった」 | 加えて、起動していないのに中断でもない組み合わせ、終了の状態を取得できない場合(runtime の `runner.rs` の記述)も 127 |
| 65 に含まれるもの | 一覧あり | 加えて、`pipeline` が完了を判定できないとき(起こらない想定) |
| 変換の入力・出力の誤り | 67 | 一致(標準出力に書けないときも 67) |
| `watch` の標準出力の失敗 | 65 | 一致 |
| `reset --all` の警告 | 終了コード 0 | 一致 |
| 69(v0.9.0 以降) | 「`init` の作る先に、ファイルが既にある」 | 一致 |
| `init` の 65(v0.9.0 以降) | 「作れないとき(親のディレクトリが無い、書く権限がない、作る先がディレクトリ、そのほかの入出力の誤り)と標準出力に書けないとき」 | 一致 |
