# 10. エラーと終了コード

この章は、Go で実装する `conductor` の終了コードの全件、複数の条件が当てはまるときの優先順位、ジョブの結果から終了コードへの対応、想定外の内部失敗の扱い、そして Go でのエラー型の設計を規定する。利用者から見えるものは「エラー分類」「終了コード」「メッセージ」で規定し、Go の型は 8 章で規定する。メッセージの全文は [08-output-and-i18n.md](08-output-and-i18n.md)、引数の誤りは [04-cli.md](04-cli.md)、流れは [05-execution-flow.md](05-execution-flow.md)、シグナルは [07-process-and-signals.md](07-process-and-signals.md)、ステートは [06-state-file.md](06-state-file.md)、定義は [03-definition-format.md](03-definition-format.md)、変換は [09-converter.md](09-converter.md) を参照する。

## 1. 終了コードの一覧(全件)

| 終了コード | 名前 | 意味 | 主な発生箇所 |
|------------|------|------|--------------|
| 0 | OK | 成功。ジョブの成功・警告、スキップ(実行済み・遷移・`@end`)、`watch`・`reset`・`validate`・`convert` の成功、`--help`・`--version`。`reset --all` の警告(読めなくて残したファイル)も 0 | 全般 |
| 1 | UNLISTED_ZERO | ジョブが終了コード 0 で終わったが、`success_exit_codes` に 0 がない。または、ジョブの終了コードがエラーで、下位 8 ビットが 0 になる(例: 256)。または、起動して終わったのに終了コードが得られない(起こらない想定) | ジョブの結果の分類、終了コードの範囲合わせ |
| 1〜255 | (ジョブの値) | エラーになったジョブの終了コードを、そのまま。範囲外は下位 8 ビット(0 になるなら 1)。4 章 | 範囲合わせ |
| 64 | DEFINITION | 定義の誤り: ファイルがない・読めない・大きすぎる・文法・検証の誤り(パイプライン名の不一致、`run_id_env` や `remove_on_complete` の型・形の誤り、`retry` の誤りを含む)、拡張子が `.yaml`・`.yml`・`.json` のどれでもない、`command` の誤り(引用符が閉じていない・空)。`validate` の失敗も 64 | `run`, `pipeline`, `validate` |
| 65 | STATE | ステートの誤り: 読めない・大きすぎる・壊れている・版の不一致・定義との食い違い・中身の `run_id` とファイル名の食い違い・書き込みの失敗・削除(初期化)の失敗。`reset`・`reset --all` で消すのに失敗。`watch` が標準出力に書けない。`pipeline` が完了を判定できない | `run`, `pipeline`, `watch`, `reset` |
| 66 | REJECTED | 拒否(順序外など。`run` で前のジョブが終わっていない、スキップ対象、`@end` より後ろ、実行済み) | `run`, `pipeline` |
| 67 | CONVERSION | 変換の誤り。`convert` が標準出力に書けない | `convert` |
| 68 | USAGE | 使い方の誤り(知らないコマンド、引数の不足、余分な引数、知らないオプション、使えないオプション、値の欠落・誤り、知らないジョブID、run ID の値の誤り〔引数・環境変数とも〕、重複、`--all` と `--run-id` の併用) | 引数の解釈、run ID の決定、`watch`/`reset` |
| 70 | INTERNAL | 内部の誤り: 想定外の内部失敗(6 章)、シグナルの準備の失敗 | 全般、`run`/`pipeline` の準備 |
| 127 | START_FAILURE | ジョブを起動できなかった(実行ファイルがない・実行権限がない、など。終了の状態を取得できない場合も安全側で起動失敗扱い) | ジョブの結果の分類 |
| 128 + 番号 | SIGNAL_BASE | ジョブがシグナルで終了した、または `conductor` が中断された(例: SIGHUP = 129、SIGINT = 130、SIGQUIT = 131、SIGTERM = 143)。リトライの待機中の中断も含む | ジョブの結果の分類、中断 |

- 64〜68 と 70 は、ジョブ自身の終了コードと重なりうる。区別は、標準エラーの `conductor: error: ...` の行で行う(70 は固定の英語の文)。
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
- `validate`: 定義(64)→ `command`(64)。成功は 0。ステートに触れない。シグナルの準備はしない。
- `convert`: 変換(67)→ 標準出力への書き込み(書けなければ 67)。成功は 0。
- `pipeline` の特有: 実行したジョブがエラー(または分類なし)なら、そのジョブの終了コードで終わる。ジョブのあいだに中断の合図があれば `128 + 番号`。完了なら 0。完了を判定できなければ 65。
- ステートの書き込みに失敗すると、ジョブの終了コードではなく 65 で終わり、「ジョブは実行されました」と表示する(少なくとも 1 回は実行された、という方針)。起動前の失敗(初期ステートの書き込みなど)は「ジョブは起動していません」。
- 想定外の内部失敗(6 章)は、どの時点でも 70 で終わる(上の順序の外)。

## 3. ジョブの結果から終了コードへ(分類)

最初に当てはまるものを採る。この判断は `internal/flow` の純粋な関数(ジョブの実行結果の値を入力に、分類と報告する終了コードを返す)として実装する([05-execution-flow.md](05-execution-flow.md))。

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

プロセスの終了コード = `ExitCodeFor(報告する終了コード, エラーか)`(4 章)。`run` は、この値で終わる。`pipeline` は、エラーまたは分類なしのジョブで止まるとき同じ式で、完了なら 0。

注: 「起動した」は、子プロセスを作れたことを指す(Go では `exec.Cmd.Start` が成功したこと)。「中断」は、`conductor` がシグナル(07 章)を受け取り、中断の合図が立ったこと。ジョブがシグナルで終了した(順 5)のと区別する。

Go での「終了コードの取り出し」の注意(07 章と共通。実装時に確認する): `*exec.ExitError` の `ExitCode()` は、プロセスがシグナルで終了した場合(UNIX 系)に `-1` を返す、と文書に書かれている。その場合は、`ExitError.Sys()` を `syscall.WaitStatus` に型変換して、`Signaled()` と `Signal()` で番号を得る(順 5)。シグナルでも終了コードでも判定できないときは、順 6 の「終了コードがない」として扱う。Windows で、`ExitCode()` が、符号なし 32 ビットの値(例: `0xC0000005`)を、64 ビットの `int` の正の値(3221225477)で返す環境では、`ExitCodeFor` に渡す前に `int32(uint32(code))` で、符号つき 32 ビットの値(-1073741819)にそろえる(4 章の例の値になる)。この符号つきへの変換の要否は、実機(Windows)での確認が済んでいない(未確認)。

## 4. 終了コードの範囲合わせ

`ExitCodeFor(reported int32, isError bool) int`:

| 入力 | 出力 |
|------|------|
| `0 <= reported <= 255` | `reported` |
| 範囲外 | `reported` の下位 8 ビット(`reported & 0xFF`。負の値は 2 の補数表現の下位 8 ビット)。ただし、その結果が 0 で `isError` が真なら 1(エラーを成功に見せない) |

`reported` は、符号つき 32 ビット整数(Go の `int32`)の範囲で扱う。下位 8 ビットを取るときは、`int(uint32(reported) & 0xFF)` と書く(`int32` から `uint32` への変換は、実行時の値に対して 2 の補数のまま下位ビットを保つ。Go の言語仕様の整数変換の規則による。実装時にテストで確認する)。例: 259 → 3、511 → 255、-1 → 255、-2 → 254、`0xC0000005`(Windows の例外コード。32 ビット符号つき整数にした値 -1073741819)→ 5。256・512・32 ビット符号つき整数の最小値・-256 は、エラーなら 1、エラーでなければ 0。

`SignalExitCode(n int32) int` = `ExitCodeFor(128 + n, true)`(加算はあふれない形で行う。`n > math.MaxInt32 - 128` のときは、結果を `math.MaxInt32` に止める。`n` が負のときの規定はない。呼び出し側が正の番号だけを渡す)。15 → 143、2 → 130、1 → 129、3 → 131、300 → 172(428 の下位 8 ビット)。

`-v` の結果行には、元の値(`exit code <C>`)と、`conductor` の終了コードが異なるとき(`result=error` のみ)`, conductor exits with <E>` を示す(08 章 2.1)。

### 4.1 プロセスの終了

`cmd/conductor/main.go` は、`internal/cli` の `Run` が返した `int`(すでに 0〜255 に合わせたもの)を、`os.Exit` に渡す。`os.Exit` は、遅延実行(`defer`)を実行せずに終わるので、`recover` の `defer` は `Run` を呼ぶ関数の中に置き、`os.Exit` はその外側(`recover` が終わったあと)で 1 回だけ呼ぶ(6 章の骨格)。

## 5. シグナルと中断

- UNIX 系: `SIGHUP`(1)・`SIGINT`(2)・`SIGQUIT`(3)・`SIGTERM`(15)を、`os/signal` の `signal.Notify`(専用のチャネル)で受け取り、最初の番号を中断の合図に記録する。ほぼ同時に複数届くと、番号の小さい方が「最初」になりうる(順序は実装依存。必須ではない)。
- Windows: Ctrl+C(番号 2。`signal.Notify(ch, os.Interrupt)`)だけ。
- `run`・`pipeline` だけがシグナルを準備する。`watch`・`reset`・`validate`・`convert` は、OS の既定の動作で終わる(`signal.Notify` を呼ばない。ただし、`SIGPIPE` の扱いだけは 08 章 1.2 に従う)。
- 起動済みのジョブが中断されると、`128 + 番号`、エラーとして記録する。起動前、またはジョブのあいだ、リトライの待機中に中断されたときは、エラーとして記録せず、再開位置を記録して `128 + 番号` で終わる。
- 準備の失敗は、`conductor: error: failed to set up signal handling` を出して 70。同じプロセスで 2 回目の登録になる「すでに登録済み」は、成功と同じに扱う(実装が複数回呼ぶ構造のときだけ関係する)。`signal.Notify` 自体は失敗を返さないので、準備の失敗は、`Deps` のシグナルの口(インターフェース)の実装が、準備できなかったときに返すエラーとして規定する(07 章)。
- 詳細は [07-process-and-signals.md](07-process-and-signals.md)。

## 6. 想定外の内部失敗(終了コード 70)

実装のバグなどで、設計が想定していない失敗(Go では `panic`、およびランタイムの失敗のうち `recover` で捕捉できるもの)が起きたときの規定。

必須の挙動:

1. `cmd/conductor/main.go` の、`Run` を呼ぶ関数の最初に、`defer` と `recover()` で、`panic` を捕捉する仕組みを設定する。
2. 捕捉したら、標準エラーに、固定の英語の文を 1 行書く(ロケールを判定しない)。

   ```text
   conductor: error: internal error
   ```

   末尾は改行。書き込みに失敗しても無視して続ける(`os.Stderr` は書き込みを溜めないので、`Flush` は不要)。
3. 終了コード 70 でプロセスを終了する。スタックトレースや失敗の詳細を、標準出力へ出してはならない(標準出力はジョブの出力と `watch`・`convert` の結果のためにある)。詳細(`panic` の値、`runtime/debug.Stack()` の出力)を標準エラーへ出すかは、実装に任せてよい(出す場合も上の 1 行を、必ず含める。1 行目に出す)。
4. 外部の入力(引数、環境変数、ファイル、シグナル、ジョブの出力)に対して、想定外の失敗が起きない作りにする。これらは、すべて、規定された誤り(64〜68 など)として扱う。たとえば、配列の添字、`nil` の参照、型表明の失敗が、入力から起きないようにする。

骨格(例。実装コードではなく形の例):

```go
func main() {
    os.Exit(runMain())
}

func runMain() (code int) {
    defer func() {
        if r := recover(); r != nil {
            // 標準エラーに 1 行(失敗は無視)。標準出力には何も書かない
            _, _ = os.Stderr.WriteString("conductor: error: internal error\n")
            code = 70
        }
    }()
    return cli.Run(os.Args[1:], cli.DefaultDeps())
}
```

`recover` で捕捉できないもの(Go の制約。実装時に確認する): ほかのゴルーチンで起きた `panic` は、そのゴルーチン自身の `defer` で捕捉しない限り、プロセスを終了させる(終了コードは 2 で、70 にならない)。したがって、`go` で起動するすべてのゴルーチン(シグナルの受信、子プロセスの待ち受け、出力の転送など)の先頭にも、同じ形の `defer` + `recover` を置き、捕捉したら標準エラーに同じ 1 行を書いて、`os.Exit(70)` で終わる(この場合の `os.Exit` は、ゴルーチンの内側から呼んでよい)。メモリ不足や、メモリ領域の二重解放などの致命的なランタイムエラー(`fatal error`)は、`recover` で捕捉できない(70 にならない)。この場合は仕様の対象外とする(未確認ではなく、Go の既知の制約として明記する)。

ジョブの実行中に想定外の失敗が起きたとき:

- 子プロセスの終了を待たずにプロセスが終わり、ジョブが孤児として残りうる(待つことは必須ではない)。
- ステートは直前の完全な内容のままであること(書き込みは、一時ファイル + `os.Rename` で、途中の内容が見えない形で行う。[06-state-file.md](06-state-file.md))。次の実行は、そのジョブを再実行しうる(少なくとも 1 回の方針)。

実装に任せてよい点:

- 失敗の詳細の保存や標準エラーへの追加の出力。ただし、`conductor: error: internal error` の行と 70 は必須。
- 内部の失敗を起こすための専用のコマンドや、テスト用の差し替えの口は、規定しない(製品の実行ファイルに専用のコマンドはない)。テストでは、`Deps` の差し替えで `panic` を起こす実装を渡し、`runMain` に相当する関数の戻り値が 70 で、標準出力が空であることを確認してよい。

## 7. エラー分類の一覧

内部の型の分け方は 8 章で規定する。利用者から見える区分として、次の分類と対応を守る。

### 7.1 定義の誤り(終了コード 64)

30 種。種類の識別子と、英語・日本語の文は [08-output-and-i18n.md](08-output-and-i18n.md) 4.3 に全件ある。

識別子: `fileMissing`, `unreadable`, `tooLarge`, `syntax`, `typeMismatch`, `unknownKey`, `versionMissing`, `versionUnsupported`, `unknownTarget`, `errorTransition`, `backwardOrSelfTarget`, `overlappingExitCodes`, `zeroAsWarning`, `pipelineNameMismatch`, `duplicateJobId`, `invalidJobId`, `noJobs`, `emptyField`, `duplicateTransition`, `invalidEnvName`, `retryNotTable`, `retryMissingKey`, `retryExitCodesEmpty`, `retryExitCodeInvalid`, `retryExitCodesOverlap`, `retryMaxInvalid`, `retryDurationInvalid`, `retryBackoffIncomplete`, `retryMultiplierInvalid`, `retryMaxIntervalTooSmall`。すべて終了コード 64。

Go 版の定義の拡張子の誤り(`.yaml`・`.yml`・`.json` のどれでもない)の種類・原因の文は、[03-definition-format.md](03-definition-format.md) が規定する(この 30 種のどれに当てはめるか、新しい種類を足すかは、03 章に従う。終了コードは 64)。

誤りが持つ情報: ファイル(表示するパス)、行(あれば)、種類、原因(日本語の固定の文)。表示は `<ファイル>:<行>: <種類の文>: <原因>`(行がなければ `<ファイル>: <種類の文>: <原因>`)。原因には、入力の値を含めない(入力に由来するのはキー名だけで、`` `キー名` `` の形で入り、64 文字で切る)。原因の文の全件は [03-definition-format.md](03-definition-format.md) にある(YAML/JSON 版)。

run ID の規則違反(`Empty`, `TooLong`, `InvalidChar`, `LeadingDot` の 4 通り)は、利用者には区別を見せず、すべて `invalid value`(68)にまとめ、値を含めない。

### 7.2 `command` の誤りとステートの誤り

`command` の誤り(終了コード 64): 2 種。`UnclosedQuote`(引用符が閉じていない)、`Empty`(コマンド名がない)。表示は 08 章 4.4 の種類の文だけ(原因は表示しない。内部の原因の文は、前者が `引用符が閉じていません`、後者が `コマンド名がありません`)。

ステートの誤り(終了コード 65): 6 種。`Unreadable`, `TooLarge`, `Corrupt`, `VersionMismatch`, `WriteFailed`, `RemoveFailed`。持つ情報は、パス、種類、原因。表示は 08 章 4.5。

定義との食い違い(終了コード 65): 3 種。`PipelineName`, `JobIds`, `JobOrder`。表示は 08 章 4.6。

ステートの原因の文(固定の日本語。言語を切り替えない)と、それを返す種類。ステートファイルは JSON なので、TOML に触れていた 2 件を直してある(表の太字ではなく、「変更」の列で示す):

| 原因の文 | 種類 | 場面 | 変更 |
|----------|------|------|------|
| `開く、または読み取ることができません` | `Unreadable` | 読み込み、`reset --all` で読めなかった | |
| `大きさが上限(1 MiB)を超えています` | `TooLarge` | 読み込み | |
| `UTF-8 の文字列ではありません` | `Corrupt` | 読み込み | |
| `JSON として読めません` | `Corrupt` | 読み込み | 「TOML として読めません」から変更 |
| `format_version がありません` | `Corrupt` | 読み込み | |
| `format_version が整数ではありません` | `Corrupt` | 読み込み | |
| `対応していない版です(reset が必要です)` | `VersionMismatch` | 読み込み | |
| `知らないキーがあります` | `Corrupt` | 読み込み | |
| `必須のキーがありません` | `Corrupt` | 読み込み | |
| `値の型が誤っています` | `Corrupt` | 読み込み | |
| `status の値が誤っています` | `Corrupt` | 読み込み | |
| `skip_reason の値が誤っています` | `Corrupt` | 読み込み | |
| `skip_reason が status と合いません` | `Corrupt` | 読み込み | |
| `exit_code が status と合いません` | `Corrupt` | 読み込み | |
| `ジョブIDが重複しています` | `Corrupt` | 読み込み | |
| `job の並びが job_order と一致しません` | `Corrupt` | 読み込み | |
| `resume_job_id が job_order にありません` | `Corrupt` | 読み込み | |
| `completed と resume_job_id が両立しません` | `Corrupt` | 読み込み | |
| `run_id がファイル名の run ID と一致しません` | `Corrupt` | 読み込み(`reset --all` では「別の定義のステート」として扱い、消さない) | |
| `書き込みに失敗しました` | `WriteFailed` | 書き込み | |
| `内容を JSON にできません` | `WriteFailed` | 書き込み | 「内容を TOML にできません」から変更 |
| `削除に失敗しました` | `RemoveFailed` | `reset`、`reset --all`、完了時の削除 | |

読み込みの検査の順序と、各原因の判定条件は [06-state-file.md](06-state-file.md)(JSON 版)に従う。複数の原因が当てはまるときにどれを出すかの順序は、この章では未確認(06 章に従う)。

### 7.3 変換の誤り(終了コード 67)

6 種: `fileMissing`, `unreadable`, `tooLarge`, `syntax`, `unrepresentable`, `rootNotMap`。持つ情報は、ファイル、行(あれば)、キーの位置(あれば)、種類、原因。表示は 08 章 4.8。

原因の文(固定の日本語。言語を切り替えない)。変換の規則と、どの場面で使うかは [09-converter.md](09-converter.md)(YAML と JSON の相互変換)が規定する。次の表は、使ってよい文の全件で、文言はここと完全に同じにする。Go 版の変換の規則で起こりえない場面の文は、09 章が使わないと規定すれば、使わなくてよい。

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
| `null は表せません` | `null`(09 章が `null` を拒否すると規定するときだけ) |
| `文字列でないキーは表せません` | 非文字列キー |
| `タグつきの値は表せません` | YAML のタグ |
| `日時の値は表せません` | 日時(09 章が日時を拒否すると規定するときだけ) |
| `数値が表せる範囲を超えています` | 数値 |
| `マージキーの値がマップではありません` | `<<` |
| `最上位がマップではありません` | 最上位 |
| `文書が空です` | 空 |
| `変換後の形式として書き出せません` | 出力 |

この表の文のうち TOML に触れるものはない(元の仕様の一覧から、削除も変更もしていない)。`null`・`日時` の 2 件は、元の実装が TOML への変換を前提に拒否していた文であり、Go 版(YAML と JSON の相互変換)で使うかは 09 章が決める(この章では未確認)。

キーの位置の形式: キーは `.` で連結(各キーは 64 文字で切る)、配列の添字は `[i]`、文字列でないキーは `(非文字列キー)`。例: `job[2].transitions[0]`。

どの種類とどの原因が組になるか(例: `fileMissing` ↔ `指定の場所にファイルがありません`)は、場面の列から自然に決まる。完全な対応表は 09 章に従う(この章では未確認)。

### 7.4 使い方の誤り(終了コード 68)

8 種: `UnknownCommand`, `MissingArgument`, `UnexpectedArgument`, `UnknownOption`, `OptionNotAllowed`, `MissingValue`, `InvalidValue`, `UnknownJob`。文言は 08 章 4.2。誤りは、種類と、対象(あれば)を持つ。対象は、誤りの対象の語(知らないコマンド、オプション名、ジョブID、`run ID (environment variable)` など)。入力の値は、それ以外には含めない(`--run-id` の値、環境変数の値は含めない)。

発生条件:

| 発生条件 | 種類 | 対象 |
|----------|------|------|
| 位置引数なし | `MissingArgument` | なし |
| コマンド名が未知(空文字も) | `UnknownCommand` | その語 |
| 文法にないオプション、`--bogus=x` | `UnknownOption` | `--bogus`、`-x` など |
| そのコマンドで使えないオプション | `OptionNotAllowed` | `-v`, `--config`, `--to`, `--run-id`, `--all`, `--remove-on-complete` |
| 値のないオプション | `MissingValue` | `--config`, `--run-id`, `--to` |
| 重複、`--verbose=x` など、`--all` と `--run-id` の併用、`--run-id` の値の規則違反 | `InvalidValue` | オプション名(`-v`, `--config`, …、併用は `--all`) |
| `--to` の値が `yaml`/`json` 以外 | `InvalidValue` | その値 |
| 値を取るオプションの直後が `-h`/`--help` | (値として取られるので help にならない。ほかに位置引数がなければ `MissingArgument`) | なし |
| 非 UTF-8 の位置引数 | `InvalidValue` | 読める形に変換した文字列(04 章 4.4) |
| 環境変数の run ID の規則違反 | `InvalidValue` | `run ID (environment variable)` |
| 余分な位置引数 | `UnexpectedArgument` | その語 |
| `convert` で `--to` なし | `MissingArgument` | `--to` |
| `run` のジョブIDが定義にない | `UnknownJob` | そのジョブID |

### 7.5 シグナルの準備の誤り

2 種。「すでに登録済み」(成功と同じ扱い)、「失敗」(`conductor: error: failed to set up signal handling`、70)。

## 8. Go でのエラー型の設計

### 8.1 方針

- 失敗は、`error` インターフェースを実装した型の値として返す(例外的な制御の移動である `panic` は、想定外の内部失敗(6 章)だけに使い、規定された誤りには使わない)。
- 各誤りの型は、「種別(列挙)」と「利用者に見える情報」を持つ。文言への変換(ロケール、エスケープ)は `internal/report` の責務で、エラー型の `Error()` メソッドは、ロケールを判定せず、デバッグ用の固定の英語の 1 行を返す(利用者向けの出力には使わない)。
- 終了コードは、エラー型の値から、`internal/cli` の 1 つの関数で決める(型ごとの `ExitCode() int` メソッド、または型の振り分け)。終了コードの数値の定義は 1 か所(定数)に集める。
- 呼び出し側は、`errors.As` で型を判定する。誤りを包む(`fmt.Errorf("...: %w", err)`)場合も、`errors.As` で取り出せるようにする。包んでも、利用者向けの出力は、元の型の情報だけから作る(包んだ文脈の文字列は、利用者に出さない)。

### 8.2 定数と共通のインターフェース

```go
// 終了コード(1 章)。値は 1 か所の定数に集める。
const (
    ExitOK            = 0
    ExitUnlistedZero  = 1
    ExitDefinition    = 64
    ExitState         = 65
    ExitRejected      = 66
    ExitConversion    = 67
    ExitUsage         = 68
    ExitInternal      = 70
    ExitStartFailure  = 127
    ExitSignalBase    = 128
)

// すべての conductor の誤りが実装する。
type CodedError interface {
    error
    ExitCode() int
}
```

### 8.3 型の一覧(骨格)

パッケージごとに、自分の誤りの型を持つ(6 章のパッケージ構成)。

| パッケージ | 型 | 種別の列挙 | 終了コード | 持つ情報 |
|------------|----|------------|------------|----------|
| `internal/definition` | `*DefinitionError` | `DefinitionKind`(7.1 の 30 種 + 03 章が足すもの) | 64 | ファイル、行(0 は「なし」)、種類、原因(日本語の固定の文) |
| `internal/definition` | `*CommandError` | `CommandKind`(`UnclosedQuote`, `Empty`) | 64 | ジョブID、種類 |
| `internal/runid` | `*RunIDError` | `RunIDKind`(`Empty`, `TooLong`, `InvalidChar`, `LeadingDot`) | 68(利用者には `invalid value`) | 種類のみ(値は持たない) |
| `internal/state` | `*StateError` | `StateKind`(`Unreadable`, `TooLarge`, `Corrupt`, `VersionMismatch`, `WriteFailed`, `RemoveFailed`) | 65 | パス、種類、原因 |
| `internal/state` | `*MismatchError` | `MismatchKind`(`PipelineName`, `JobIds`, `JobOrder`) | 65 | パス、種類 |
| `internal/convert` | `*ConvertError` | `ConvertKind`(7.3 の 6 種) | 67 | ファイル、行、キーの位置、種類、原因 |
| `internal/cli` | `*UsageError` | `UsageKind`(7.4 の 8 種) | 68 | 種類、対象(あれば) |
| `internal/cli` | `*RejectedError` | `RejectReason`(`OutOfOrder`, `SkipTarget`, `AfterEnd`, `AlreadyExecuted`) | 66 | ジョブID、理由 |
| `internal/cli` | `*JobFailure` | (なし。分類の結果として扱う) | 3 章から決まる | 報告する終了コード、エラーか |

- 種別は、`iota` による整数型の列挙(`type StateKind int` と定数)にする。文字列の識別子(7.1 の `fileMissing` など)は、列挙の値から決まる固定の名前の表(`String()` メソッドなど)として持ち、テストで使ってよい。
- 種別の列挙の値は、永続化も外部への公開もしない。したがって、並びの追加・並べ替えで、互換性の問題は起きない。
- 「ゼロ値」で誤りを表さない(列挙の 0 番目は `kindUnknown` のような無効値にして、設定忘れをテストで検出できるようにしてもよい)。これは任意。

```go
type UsageError struct {
    Kind      UsageKind
    Target    string // 空文字は「対象なし」と区別したいので、次の HasTarget を使う
    HasTarget bool
}

func (e *UsageError) Error() string   { /* デバッグ用の固定の英語 */ }
func (e *UsageError) ExitCode() int   { return ExitUsage }
```

対象が空文字の誤り(例: `unknown command: ''`、空のコマンド名)があるので、「対象なし」と「対象が空文字」は、`HasTarget` で区別する(`conductor: error: unknown command: ''` のように、対象があれば `'...'` で囲んで出す)。

### 8.4 ジョブの結果と終了コード

ジョブの結果(3 章)は、エラー型ではなく、`internal/flow` の値(分類、報告する終了コード)として表す。`run` / `pipeline` の処理は、結果を `Run` の戻り値の終了コードに変換する。ステートの書き込みの失敗(`*StateError`)は、ジョブの結果より優先して終了コード 65 にする(2 章の順 11)。

### 8.5 最上位での決定

`internal/cli.Run` は、各処理が返した `error` を受け取り、次の順で終了コードと表示を決める。

1. `errors.As` で、`*UsageError` → 使い方の誤りの表示(誤りの行、空行、使い方全文)と 68。
2. `*DefinitionError` / `*CommandError` → 64。`*StateError` / `*MismatchError` → 65。`*RejectedError` → 66。`*ConvertError` → 67。それぞれの表示は 08 章。
3. 上のどれでもない `error`(想定していない誤り)は、`panic` と同じ扱いにはせず、`conductor: error: internal error` を標準エラーに 1 行出して 70 で終わる(6 章と同じ文)。想定外の `error` を握りつぶして成功にしてはならない。
4. 型の判定の順序は、同じ値が複数の型に当てはまらないように作る(各型は独立で、埋め込みで継承しない)ので、実際には順序に依存しない。

## 9. 標準エラーの出力と終了コードの例

```text
$ conductor validate            # command の誤り
conductor: error: d.yaml: job 'z': command has an unclosed quote
→ 64

$ conductor pipeline wrongname
conductor: error: c.yaml:3: pipeline name does not match the definition: <原因(日本語)>
→ 64

$ conductor run extract         # 壊れたステート
conductor: error: c.yaml.state: state file is corrupt: JSON として読めません; run `conductor reset` to discard the state
→ 65

$ conductor run load            # 順序外
conductor: error: job 'load': cannot run this job now: earlier jobs have not finished (out of order)
→ 66

$ conductor convert nofile --to json
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
```

## 10. 必須の挙動と、実装に任せてよい点

必須:

- 1 章の終了コードの全件と意味、2 章の決定順序、3 章の分類、4 章の範囲合わせ。
- 6 章の 70(固定の 1 行と終了コード)と、ゴルーチンでの `recover`。
- 標準エラーの `conductor: error: ...` の行の形式と文言(08 章)。
- 少なくとも 1 回の方針: ステートの書き込みに失敗したとき、ジョブの終了コードより 65 を優先する。
- 8 章の、`error` を実装した型、`errors.As` での判定、種別の列挙という方針(型の名前と分け方は目安)。

実装に任せてよい点:

- 型の名前、フィールドの名前、パッケージ間の細かい分け方(8.3 は目安)。
- 同時に届いた複数のシグナルのうち、どれを「最初」とみなすか。
- 想定外の失敗の詳細を標準エラーへ追加で出すか。

## 11. Go 版で変えた点

| 項目 | 変更 |
|------|------|
| ステートの原因の文 | `TOML として読めません` → `JSON として読めません`、`内容を TOML にできません` → `内容を JSON にできません` |
| 定義の誤りの識別子 `retryNotTable` の文言 | 08 章 4.3(`retry must be a mapping` / `retry はマップで書きます`) |
| 変換の誤り | YAML と JSON の相互変換。原因の文の使用は 09 章に従う |
| 定義の拡張子の誤り | 64。種類・原因は 03 章 |
| 想定外の内部失敗 | `defer` + `recover`(メインと、すべてのゴルーチン)で 70 |

## 12. 未確認

- 定義の誤りの原因の文の全件(03 章に属する)。拡張子の誤りの種類と原因の文(03 章に属する)。
- 変換の種類と原因の完全な対応表(09 章に属する)。`null`・日時の原因の文を Go 版で使うか。
- ステートの読み込みで複数の原因が重なるときの優先(06 章)。
- Windows で `ExitCode()` が返す値の符号(3 章の注)。
- `signal.Notify` と `SIGPIPE`・子プロセスへの影響(08 章 1.2)。
- Go の整数変換(`int32` → `uint32`)と `ExitError.ExitCode()` の、使用する Go の版ごとの文書の記述。実装時に、テストで確認する。
