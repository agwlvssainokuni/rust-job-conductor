# 10. エラーと終了コード

この章は、`conductor` の終了コードの全件、複数の条件が当てはまるときの優先順位、ジョブの結果から終了コードへの対応、想定外の内部失敗の扱いを規定する。エラーは、内部の型でなく、「エラー分類」「終了コード」「利用者に見えるメッセージ」で規定する。メッセージの全文は [08-output-and-i18n.md](08-output-and-i18n.md)、引数の誤りは [04-cli.md](04-cli.md)、流れは [05-execution-flow.md](05-execution-flow.md)、シグナルは [07-process-and-signals.md](07-process-and-signals.md) を参照する。

## 1. 終了コードの一覧(全件)

| 終了コード | 名前 | 意味 | 主な発生箇所 |
|------------|------|------|--------------|
| 0 | OK | 成功。ジョブの成功・警告、スキップ(実行済み・遷移・`@end`)、`watch`・`reset`・`validate`・`convert` の成功、`--help`・`--version`。`reset --all` の警告(読めなくて残したファイル)も 0。v0.9.0 以降: `graph`・`init` の成功、`--dry-run` の計画を出せたとき(模擬の結果がエラーでの停止でも 0)、`validate` の `.env` の重複キーの警告 | 全般 |
| 1 | UNLISTED_ZERO | ジョブが終了コード 0 で終わったが、`success_exit_codes` に 0 がない。または、ジョブの終了コードがエラーで、下位 8 ビットが 0 になる(例: 256)。または、起動して終わったのに終了コードが得られない(起こらない想定) | ジョブの結果の分類、終了コードの範囲合わせ |
| 1〜255 | (ジョブの値) | エラーになったジョブの終了コードを、そのまま。範囲外は下位 8 ビット(0 になるなら 1)。4 章 | 範囲合わせ |
| 64 | DEFINITION | 定義の誤り: ファイルがない・読めない・大きすぎる・文法・検証の誤り(パイプライン名の不一致、`run_id_env` や `remove_on_complete` の型・形の誤り、`[job.retry]` の誤りを含む)、`command` の誤り(引用符が閉じていない・空)。`validate` の失敗も 64。v0.9.0 以降: `env`・`env_file` の誤り(`reservedEnvName`・`envNameCaseConflict`・`invalidEnvValue` ほか)と、`.env` が無い・読めない・大きすぎる・UTF-8 でない・書式の誤り(13 種)。`graph`・`--dry-run` の定義と `.env` の誤りも 64 | `run`, `pipeline`, `validate`, `graph`, `--dry-run` |
| 65 | STATE | ステートの誤り: 読めない・大きすぎる・壊れている・版の不一致・定義との食い違い・中身の `run_id` とファイル名の食い違い・書き込みの失敗・削除(初期化)の失敗。`reset`・`reset --all` で消すのに失敗。`watch` が標準出力に書けない。`pipeline` が完了を判定できない。v0.9.0 以降: `graph`・`--dry-run` のステートの読み取りの誤りと、標準出力に書けないとき。`init` がファイルを作れない(親のディレクトリが無い・書く権限がない・作る先がディレクトリ・そのほかの入出力の誤り)・ひな形を標準出力に書けない | `run`, `pipeline`, `watch`, `reset`, `graph`, `--dry-run`, `init` |
| 66 | REJECTED | 拒否(順序外など。`run` で前のジョブが終わっていない、スキップ対象、`@end` より後ろ、実行済み)。v0.9.0 以降は、`--dry-run` で、本番なら拒否されるとき | `run`, `pipeline`, `--dry-run` |
| 67 | CONVERSION | 変換の誤り。`convert` が標準出力に書けない | `convert` |
| 68 | USAGE | 使い方の誤り(知らないコマンド、引数の不足、余分な引数、知らないオプション、使えないオプション、値の欠落・誤り、知らないジョブID、run ID の値の誤り〔引数・環境変数とも〕、重複、`--all` と `--run-id` の併用)。v0.9.0 以降: `--assume` の誤り(書式・範囲外・重複・定義にないジョブID)、`--dry-run` なしの `--assume`、`--format` の値の誤り、`graph` 以外の `--format`・`--state`、`--state` なしの `graph --run-id`、`init` の名前が空、`init` に許されないオプション、`init` 以外への `--force`、`run`・`pipeline` 以外の `--dry-run` | 引数の解釈、run ID の決定、`watch`/`reset`、`--dry-run` の準備 |
| 69 | EXISTS | (v0.9.0 以降)`init` の作る先に、ファイルが既にある(`--force` なし。何も変えない) | `init` |
| 70 | INTERNAL | 内部の誤り: 想定外の内部失敗(6 章)、シグナルの準備の失敗 | 全般、`run`/`pipeline` の準備 |
| 127 | START_FAILURE | ジョブを起動できなかった(実行ファイルがない・実行権限がない、など。終了の状態を取得できない場合も安全側で起動失敗扱い) | ジョブの結果の分類 |
| 128 + 番号 | SIGNAL_BASE | ジョブがシグナルで終了した、または `conductor` が中断された(例: SIGHUP = 129、SIGINT = 130、SIGQUIT = 131、SIGTERM = 143)。リトライの待機中の中断も含む | ジョブの結果の分類、中断 |

- 64〜70(69 を含む)は、ジョブ自身の終了コードと重なりうる。区別は、標準エラーの `conductor: error: ...` の行で行う(70 は固定の英語の文)。
- 終了コードの範囲(0〜255)への合わせ方は 4 章。

## 2. 終了コードの決定順序

`run` / `pipeline` は、次の順に検査し、最初に当てはまった時点で終わる(それ以降は行わない)。

| 順 | 検査 | 失敗時 |
|----|------|--------|
| 0 | 引数の解釈(オプションとして解釈された `-h`/`-V` は先に 0。値を取るオプションの値として取られた `--help` などは対象外。それ以外の使い方の誤り。04 章 4.1・4.2) | 68 |
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
- ステートの書き込みに失敗すると、ジョブの終了コードではなく 65 で終わり、「ジョブは実行されました」と表示する(少なくとも 1 回は実行された、という方針)。起動前の失敗(初期ステートの書き込みなど)は「ジョブは起動していません」。
- 想定外の内部失敗(6 章)は、どの時点でも 70 で終わる(上の順序の外)。

## 3. ジョブの結果から終了コードへ(分類)

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
| 10 | それ以外 | エラー | ジョブの終了コード | 範囲を 0〜255 に合わせる(4 章) |

プロセスの終了コード = `exitCodeFor(報告する終了コード, エラーか)`(4 章)。`run` は、この値で終わる。`pipeline` は、エラーまたは分類なしのジョブで止まるとき同じ式で、完了なら 0。

注: 「起動した」は、子プロセスを作れたことを指す。「中断」は、`conductor` がシグナル(07 章)を受け取り、中断の合図が立ったこと。ジョブがシグナルで終了した(順 5)のと区別する。

## 4. 終了コードの範囲合わせ

`exitCodeFor(reported, isError)`:

| 入力 | 出力 |
|------|------|
| `0 <= reported <= 255` | `reported` |
| 範囲外 | `reported` の下位 8 ビット(`reported & 0xFF`。負の値は 2 の補数表現の下位 8 ビット)。ただし、その結果が 0 で `isError` が真なら 1(エラーを成功に見せない) |

`reported` は、符号つき 32 ビット整数の範囲で扱う。例: 259 → 3、511 → 255、-1 → 255、-2 → 254、`0xC0000005`(Windows の例外コード。32 ビット符号つき整数にした値 -1073741819)→ 5。256・512・32 ビット符号つき整数の最小値・-256 は、エラーなら 1、エラーでなければ 0。

`signalExitCode(n)` = `exitCodeFor(128 + n, 真)`(加算はあふれない形で行う。32 ビットの範囲を超えるときは上限に止める)。15 → 143、2 → 130、1 → 129、3 → 131、300 → 172(428 の下位 8 ビット)。

`-v` の結果行には、元の値(`exit code <C>`)と、`conductor` の終了コードが異なるとき(`result=error` のみ)`, conductor exits with <E>` を示す(08 章 2.1)。

## 5. シグナルと中断

- UNIX 系: `SIGHUP`(1)・`SIGINT`(2)・`SIGQUIT`(3)・`SIGTERM`(15)を、専用の監視で受け取り、最初の番号を中断の合図に記録する。ほぼ同時に複数届くと、番号の小さい方が「最初」になりうる(順序は実装依存。必須ではない)。
- Windows: Ctrl+C(番号 2)だけ。
- `run`・`pipeline` だけがシグナルを準備する。`watch`・`reset`・`validate`・`convert` は、OS の既定の動作で終わる(v0.9.0 以降の `graph`・`init`、`--dry-run` も同じ)。
- 起動済みのジョブが中断されると、`128 + 番号`、エラーとして記録する。起動前、またはジョブのあいだ、リトライの待機中に中断されたときは、エラーとして記録せず、再開位置を記録して `128 + 番号` で終わる。
- 準備の失敗は、`conductor: error: failed to set up signal handling` を出して 70。同じプロセスで 2 回目の登録になる「すでに登録済み」は、成功と同じに扱う(実装が複数回呼ぶ構造のときだけ関係する)。
- 詳細は [07-process-and-signals.md](07-process-and-signals.md)。

## 6. 想定外の内部失敗(終了コード 70)

実装のバグなどで、設計が想定していない失敗(言語によって、例外・パニック・クラッシュ・表明の失敗などと呼ばれるもの)が起きたときの規定。

必須の挙動:

1. プログラムの起動の最初に、想定外の失敗を捕捉する仕組みを設定する(言語のグローバルな例外ハンドラ、トップレベルの try/catch、パニックのフックなど)。
2. 想定外の失敗が起きたら、標準エラーに、固定の英語の文を 1 行書く(ロケールを判定しない)。

   ```text
   conductor: error: internal error
   ```

   末尾は改行。書いたあと flush する。書き込みに失敗しても無視して続ける。
3. 終了コード 70 でプロセスを終了する。スタックトレースや失敗の詳細を、標準出力へ出してはならない(標準出力はジョブの出力と `watch`・`convert` の結果のためにある)。詳細を標準エラーへ出すかは、実装に任せてよい(出す場合も上の 1 行を、必ず含める)。
4. 外部の入力(引数、環境変数、ファイル、シグナル、ジョブの出力)に対して、想定外の失敗が起きない作りにする。これらは、すべて、規定された誤り(64〜68 など)として扱う。

ジョブの実行中に想定外の失敗が起きたとき:

- 子プロセスの終了を待たずにプロセスが終わり、ジョブが孤児として残りうる(待つことは必須ではない)。
- ステートは直前の完全な内容のままであること(書き込みは、途中の内容が見えない形で行う。[06-state-file.md](06-state-file.md))。次の実行は、そのジョブを再実行しうる(少なくとも 1 回の方針)。

実装に任せてよい点:

- 捕捉の仕組みの種類(例外ハンドラ、シグナル、トップレベルの try/catch)。
- 失敗の詳細の保存や標準エラーへの追加の出力。ただし、`conductor: error: internal error` の行と 70 は必須。
- 内部の失敗を起こすための専用のコマンドや、テスト用の差し替えの口は、規定しない(製品の実行ファイルに専用のコマンドはない)。

## 7. エラー分類の一覧

内部の型の分け方は規定しない。利用者から見える区分として、次の分類と対応を守ればよい。

### 7.1 定義の誤り(終了コード 64)

33 種(v0.9.0 で `reservedEnvName`・`envNameCaseConflict`・`invalidEnvValue` を追加)。種類の識別子と、英語・日本語の文は [08-output-and-i18n.md](08-output-and-i18n.md) 4.3 に全件ある。

識別子(参考の一覧): `fileMissing`, `unreadable`, `tooLarge`, `syntax`, `typeMismatch`, `unknownKey`, `versionMissing`, `versionUnsupported`, `unknownTarget`, `errorTransition`, `backwardOrSelfTarget`, `overlappingExitCodes`, `zeroAsWarning`, `pipelineNameMismatch`, `duplicateJobId`, `invalidJobId`, `noJobs`, `emptyField`, `duplicateTransition`, `invalidEnvName`, `retryNotTable`, `retryMissingKey`, `retryExitCodesEmpty`, `retryExitCodeInvalid`, `retryExitCodesOverlap`, `retryMaxInvalid`, `retryDurationInvalid`, `retryBackoffIncomplete`, `retryMultiplierInvalid`, `retryMaxIntervalTooSmall`, `reservedEnvName`, `envNameCaseConflict`, `invalidEnvValue`(最後の 3 つは v0.9.0 以降)。すべて終了コード 64。

誤りが持つ情報: ファイル(表示するパス)、行(あれば)、種類、原因(日本語の固定の文)。表示は `<ファイル>:<行>: <種類の文>: <原因>`(行がなければ `<ファイル>: <種類の文>: <原因>`)。原因には、入力の値を含めない(入力に由来するのはキー名だけで、`` `キー名` `` の形で入り、64 文字で切る)。原因の文の全件は [03-definition-format.md](03-definition-format.md) にある。

run ID の規則違反(`Empty`, `TooLong`, `InvalidChar`, `LeadingDot` の 4 通り)は、利用者には区別を見せず、すべて `invalid value`(68)にまとめ、値を含めない。

**`.env` の誤り(v0.9.0 以降。終了コード 64)**: 13 種(`fileMissing`, `unreadable`, `tooLarge`, `notUtf8`, `tooManyEntries`, `invalidName`, `reservedName`, `caseConflict`, `missingEquals`, `unterminatedQuote`, `invalidEscape`, `trailingGarbage`, `nulInValue`)。識別子・条件は 03 の 7.7、文言は 08 の 4.12。誤りが持つ情報: ファイル(解決済みのパス)、行(あれば)、種類、理由(**言語に従う**固定の文。値を含めない)。表示は `<ファイル>:<行>: <種類の文>: <理由>`(行がなければ `<ファイル>: <種類の文>: <理由>`)。重複キーの警告(ファイル、後ろの行、名前)は誤りではなく、`validate` だけが表示する。

**ファイルの作成の誤り(v0.9.0 以降)**: 5 種。`alreadyExists`(終了コード 69)、`parentMissing`, `isDirectory`, `permissionDenied`, `otherIo`(終了コード 65)。`init` が使う。文言は 08 の 4.13、作成の規則は 06 の 14 章。OS のエラーの文は持たない。

### 7.2 `command` の誤りとステートの誤り

`command` の誤り(終了コード 64): 2 種。`UnclosedQuote`(引用符が閉じていない)、`Empty`(コマンド名がない)。表示は 08 章 4.4 の種類の文だけ(原因は表示しない。内部の原因の文は、前者が `引用符が閉じていません`、後者が `コマンド名がありません`)。

ステートの誤り(終了コード 65): 6 種。`Unreadable`, `TooLarge`, `Corrupt`, `VersionMismatch`, `WriteFailed`, `RemoveFailed`。持つ情報は、パス、種類、原因。表示は 08 章 4.5。

定義との食い違い(終了コード 65): 3 種。`PipelineName`, `JobIds`, `JobOrder`。表示は 08 章 4.6。

ステートの原因の文(固定の日本語。言語を切り替えない)と、それを返す種類:

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

読み込みの検査の順序と、各原因の判定条件は [06-state-file.md](06-state-file.md)。複数の原因が当てはまるときにどれを出すかの順序は、この章では未確認(06 章に従う)。

### 7.3 変換の誤り(終了コード 67)

6 種: `fileMissing`, `unreadable`, `tooLarge`, `syntax`, `unrepresentable`, `rootNotMap`。持つ情報は、ファイル、行(あれば)、キーの位置(あれば)、種類、原因。表示は 08 章 4.8。

原因の文(固定の日本語。言語を切り替えない):

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
| `入れ子が深すぎます(上限 32)` | 入れ子 |
| `展開後の入れ子が深すぎます(上限 32)` | 入れ子 |
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

キーの位置の形式: キーは `.` で連結(各キーは 64 文字で切る)、配列の添字は `[i]`、文字列でないキーは `(非文字列キー)`。例: `job[2].transitions[0]`。

どの種類とどの原因が組になるか(例: `fileMissing` ↔ `指定の場所にファイルがありません`)は、場面の列から自然に決まる。変換の規則と判定の順序は [09-converter.md](09-converter.md) にあり、完全な対応表は未確認。

### 7.4 使い方の誤り(終了コード 68)

8 種: `UnknownCommand`, `MissingArgument`, `UnexpectedArgument`, `UnknownOption`, `OptionNotAllowed`, `MissingValue`, `InvalidValue`, `UnknownJob`。文言は 08 章 4.2。誤りは、種類と、対象(あれば)を持つ。対象は、誤りの対象の語(知らないコマンド、オプション名、ジョブID、`run ID (environment variable)` など)。入力の値は、それ以外には含めない(`--run-id` の値、環境変数の値は含めない)。

発生条件:

| 発生条件 | 種類 | 対象 |
|----------|------|------|
| 位置引数なし | `MissingArgument` | なし |
| コマンド名が未知(空文字も) | `UnknownCommand` | その語 |
| 文法にないオプション、`--bogus=x` | `UnknownOption` | `--bogus`、`-x` など |
| そのコマンドで使えないオプション | `OptionNotAllowed` | `-v`, `--config`, `--to`, `--run-id`, `--all`, `--remove-on-complete`, `--dry-run`, `--force`, `--assume`, `--format`, `--state`(後ろの 5 つは v0.9.0 以降) |
| 値のないオプション | `MissingValue` | `--config`, `--run-id`, `--to`, `--format`, `--assume` |
| 重複、`--verbose=x` など、`--all` と `--run-id` の併用、`--run-id` の値の規則違反 | `InvalidValue` | オプション名(`-v`, `--config`, …、併用は `--all`。v0.9.0 以降は `--dry-run`, `--force`, `--format`, `--state` も) |
| `--assume` の値の誤り(`=` がない、整数でない、範囲外、ジョブID の重複)、`--format` の値の誤り(v0.9.0 以降) | `InvalidValue` | `--assume`、`--format`(値は含めない) |
| `init` のパイプライン名が空・空白だけ(v0.9.0 以降) | `InvalidValue` | `pipeline name`(値は含めない) |
| `--to` の値が `yaml`/`toml` 以外 | `InvalidValue` | その値 |
| 値を取るオプションの直後が `-h`/`--help` | (値として取られるので help にならない。ほかに位置引数がなければ `MissingArgument`) | なし |
| 非 UTF-8 の位置引数 | `InvalidValue` | 読める形に変換した文字列 |
| 環境変数の run ID の規則違反 | `InvalidValue` | `run ID (environment variable)` |
| 余分な位置引数 | `UnexpectedArgument` | その語 |
| `convert` で `--to` なし | `MissingArgument` | `--to` |
| `run` のジョブIDが定義にない | `UnknownJob` | そのジョブID |
| `--assume` のジョブIDが定義にない(v0.9.0 以降) | `UnknownJob` | なし(ジョブIDを含めない) |

### 7.5 シグナルの準備の誤り

2 種。「すでに登録済み」(成功と同じ扱い)、「失敗」(`conductor: error: failed to set up signal handling`、70)。

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

## 9. 必須の挙動と、実装に任せてよい点

必須:

- 1 章の終了コードの全件と意味、2 章の決定順序、3 章の分類、4 章の範囲合わせ。
- 6 章の 70(固定の 1 行と終了コード)。
- 標準エラーの `conductor: error: ...` の行の形式と文言(08 章)。
- 少なくとも 1 回の方針: ステートの書き込みに失敗したとき、ジョブの終了コードより 65 を優先する。

実装に任せてよい点:

- エラーの内部表現(型、例外の階層、結果型)。
- 想定外の失敗を捕捉する仕組み。
- 同時に届いた複数のシグナルのうち、どれを「最初」とみなすか。

未確認:

- 定義の誤りの原因の文の全件(03 章に属する)。
- 変換の種類と原因の完全な対応表(上記)。
- ステートの読み込みで複数の原因が重なるときの優先(06 章)。
