# 10. エラーと終了コード

この章は、TypeScript(Node.js 上)で実装する `conductor` の終了コードの全件、複数の条件が当てはまるときの優先順位、ジョブの結果から終了コードへの対応、想定外の内部失敗の扱い、そして TypeScript でのエラー型の設計を規定する。利用者から見えるものは「エラー分類」「終了コード」「メッセージ」で規定し、型は 8 章で規定する。メッセージの全文は [08-output-and-i18n.md](08-output-and-i18n.md)、引数の誤りは [04-cli.md](04-cli.md)、流れは [05-execution-flow.md](05-execution-flow.md)、シグナルは [07-process-and-signals.md](07-process-and-signals.md)、ステートは [06-state-file.md](06-state-file.md)、定義は [03-definition-format.md](03-definition-format.md)、変換は [09-converter.md](09-converter.md) を参照する。

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
- `--help`・`--version`: 標準出力に書けなくても 0(書き込みの失敗は無視する。08 章 1.2 節)。標準出力に書けない失敗は、`watch` が 65、`convert` が 67、それ以外の経路で未捕捉になったものは 70(6 章)。
- `pipeline` の特有: 実行したジョブがエラー(または分類なし)なら、そのジョブの終了コードで終わる。ジョブのあいだに中断の合図があれば `128 + 番号`。完了なら 0。完了を判定できなければ 65。
- ステートの書き込みに失敗すると、ジョブの終了コードではなく 65 で終わり、「ジョブは実行されました」と表示する(少なくとも 1 回は実行された、という方針)。起動前の失敗(初期ステートの書き込みなど)は「ジョブは起動していません」。
- 想定外の内部失敗(6 章)は、どの時点でも 70 で終わる(上の順序の外)。

## 3. ジョブの結果から終了コードへ(分類)

最初に当てはまるものを採る。この判断は `src/flow/` の純粋な関数(ジョブの実行結果の値を入力に、分類と報告する終了コードを返す)として実装する([05-execution-flow.md](05-execution-flow.md))。

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

注: 「起動した」は、子プロセスを作れたこと(`node:child_process` の `spawn` が `'error'` イベントを出さずに、`child.pid` が数値になること)を指す。「中断」は、`conductor` がシグナル(07 章)を受け取り、中断の合図が立ったこと。ジョブがシグナルで終了した(順 5)のと区別する。

Node での「終了コードの取り出し」(詳細は 07 章。実装時に確認する): 子プロセスの `'close'`(または `'exit'`)イベントのリスナーは `(code: number | null, signal: NodeJS.Signals | null)` を受け取る。Node の文書によれば、子が自分で終了したときは `code` が終了コード(数値)で `signal` は `null`、子がシグナルで終了したときは `code` が `null` で `signal` がシグナル名(`"SIGKILL"` のような文字列)になる。したがって、順 5 は `code === null && signal !== null` のとき(`signal` の文字列名を、番号に変える表を持つ。07 章)、順 6 は `code === null && signal === null`(起こらない想定)のとき、順 7 以降は `code` が数値のとき、と判定する。`spawn` に失敗した(実行ファイルがない・権限がない)場合は、`'error'` イベント(`error.code` が `ENOENT`・`EACCES` など)が発行されるので、順 4(起動失敗)として扱う。`'error'` と `'close'` の両方が発行されうる(`'error'` のあとに `'close'` が来るかは、起動に失敗した場合の挙動として、実装時に確認する。重複して分類しないよう、先に決着した方だけを採る)。

Windows で子の終了コードが、符号なし 32 ビットの値(例: `0xC0000005` = 3221225477)で渡る場合に備え、`exitCodeFor` に渡す前に、`code | 0`(32 ビット符号つき整数への変換。3221225477 は -1073741819 になる)で符号つきにそろえる(4 章の例の値になる)。この変換の要否と、Node が Windows で返す値の符号は、実機(Windows)での確認が済んでいない(未確認)。UNIX 系では、シグナルによる終了は `code === null` で届くので、この変換の対象にならない。

## 4. 終了コードの範囲合わせ

`exitCodeFor(reported: number, isError: boolean): number`:

| 入力 | 出力 |
|------|------|
| `0 <= reported <= 255` | `reported` |
| 範囲外 | `reported` の下位 8 ビット(`reported & 0xFF`。負の値は 2 の補数表現の下位 8 ビット)。ただし、その結果が 0 で `isError` が真なら 1(エラーを成功に見せない) |

`reported` は、符号つき 32 ビット整数の範囲(`-2147483648`〜`2147483647`)の `number` として扱う(終了コードは `number` でよい。`bigint` は使わない。決定。リトライの待ち時間のような 64 ビットの量だけを `bigint` で扱う)。JavaScript のビット演算(`&`、`|`)は、オペランドを符号つき 32 ビット整数に変換して(2 の補数のまま)行うので、`reported & 0xFF` は、負の値でも下位 8 ビットを返す(`-1 & 0xFF` は 255。ECMAScript の規定による。実装時にテストで確認する)。範囲外の値が来うる場合は、先に `reported | 0` で 32 ビットに収める。例: 259 → 3、511 → 255、-1 → 255、-2 → 254、`0xC0000005`(Windows の例外コード。32 ビット符号つき整数にした値 -1073741819)→ 5。256・512・32 ビット符号つき整数の最小値・-256 は、エラーなら 1、エラーでなければ 0。

`signalExitCode(n: number): number` = `exitCodeFor(128 + n, true)`(加算はあふれない形で行う。`n > 2147483647 - 128` のときは、結果を `2147483647` に止める。`Math.min(128 + n, 2147483647)`。`n` が負のときの規定はない。呼び出し側が正の番号だけを渡す)。15 → 143、2 → 130、1 → 129、3 → 131、300 → 172(428 の下位 8 ビット)。

`-v` の結果行には、元の値(`exit code <C>`)と、`conductor` の終了コードが異なるとき(`result=error` のみ)`, conductor exits with <E>` を示す(08 章 2.1)。

### 4.1 プロセスの終了

`src/bin/conductor.ts` は、`src/cli/` の `run` が返した `number`(すでに 0〜255 に合わせたもの)を、`process.exitCode` に設定する。**終了コードは、必ず 0〜255 の整数にしてから渡す**。Node は、`process.exitCode` に整数でない値(`1.5` など)を設定すると `ERR_OUT_OF_RANGE` の `RangeError` を投げる(確認済み: v26.13.0)。整数であれば範囲外でも受け付け、POSIX では OS が下位 8 ビットに切り詰める(確認済み: macOS、v26.13.0。`256` → 0、`259` → 3、`-1` → 255、`300` → 44)。Windows でこの切り詰めが行われるかは未確認なので、この仕様では、`exitCodeFor` で自分で 0〜255 に合わせて、OS の切り詰めに頼らない。

終了の手順(出力の完了を待つこと、`process.exit()` を書き込みの完了前に呼ばないこと)は、08 章 1.3 に従う。まとめ: (1) `run` の戻り値を受け取る、(2) 標準出力・標準エラーの書き込みの完了を待つ、(3) `process.exitCode` を設定して終わる(または、完了を待ったあとに `process.exit(code)`)。

## 5. シグナルと中断

- UNIX 系: `SIGHUP`(1)・`SIGINT`(2)・`SIGQUIT`(3)・`SIGTERM`(15)を、`process.on(signal, listener)` で受け取り、最初の番号を中断の合図に記録する。ほぼ同時に複数届くと、番号の小さい方が「最初」になりうる(順序は実装依存。必須ではない)。シグナル名と番号の対応は、`node:os` の `os.constants.signals`(例: `os.constants.signals.SIGINT` は 2)から得られるが、表を自前で持つ規定は 07 章に従う。
- Windows: Ctrl+C(番号 2。`process.on('SIGINT', ...)`)だけ。
- `run`・`pipeline` だけがシグナルを準備する。`watch`・`reset`・`validate`・`convert` は、OS の既定の動作で終わる(`process.on` でシグナルを登録しない)。
- 起動済みのジョブが中断されると、`128 + 番号`、エラーとして記録する。起動前、またはジョブのあいだ、リトライの待機中に中断されたときは、エラーとして記録せず、再開位置を記録して `128 + 番号` で終わる。
- 準備の失敗は、`conductor: error: failed to set up signal handling` を出して 70。同じプロセスで 2 回目の登録になる「すでに登録済み」は、成功と同じに扱う(実装が複数回呼ぶ構造のときだけ関係する)。`process.on` 自体は、通常は失敗を投げないので、準備の失敗は、`Deps` のシグナルの口(interface)の実装が、準備できなかったときに例外(または `reject`)として返すエラーとして規定する(07 章)。
- 詳細は [07-process-and-signals.md](07-process-and-signals.md)。

## 6. 想定外の内部失敗(終了コード 70)

実装のバグなどで、設計が想定していない失敗(TypeScript では、規定されたエラー型以外の例外、`Error` でない値の `throw`、`Promise` の `reject`、イベントのリスナーの中の例外など)が起きたときの規定。

必須の挙動:

1. 入口 `src/bin/conductor.ts` の、最初の実行の文で、次の 2 つのハンドラを登録する(ほかの処理より前)。

   | 捕捉するもの | 登録 |
   |--------------|------|
   | 未捕捉の例外(同期の `throw`、タイマー・イベントのリスナー・コールバックの中の例外) | `process.on('uncaughtException', handler)` |
   | 未処理の `Promise` の拒否(`await` も `.catch` もされない `reject`) | `process.on('unhandledRejection', handler)` |

   加えて、`main()`(`run` を呼んで終了を扱う `async` 関数)の呼び出しを、`try/catch`(または `main().catch(handler)`)で包む。`catch` の変数は `unknown` として扱う(`strict` の既定)。
2. どのハンドラも、同じ 1 つの関数に集める。この関数は、標準エラーに、固定の英語の文を 1 行書く(ロケールを判定しない)。

   ```text
   conductor: error: internal error
   ```

   末尾は改行。書き込みに失敗しても無視して続ける。この関数は、1 回だけ動く(2 回目以降の呼び出しは、何もしない。例外の連鎖で行が重複して出ないように、フラグで止める)。
3. そのあと、終了コード 70 でプロセスを終了する。標準エラーへの書き込みは非同期になりうるので、書き込みの `callback` の中で `process.exit(70)` を呼び、書き込みが完了しない場合に備えて、短い時間(目安 1 秒)のあとに `process.exit(70)` を呼ぶ安全装置を併用する(08 章 1.3 の手順 4)。スタックトレースや失敗の詳細を、標準出力へ出してはならない(標準出力はジョブの出力と `watch`・`convert` の結果のためにある)。詳細(`Error` の `stack`、`cause`)を標準エラーへ出すかは、実装に任せてよい(出す場合も上の 1 行を、必ず含める。1 行目に出す)。
4. 外部の入力(引数、環境変数、ファイル、シグナル、ジョブの出力)に対して、想定外の失敗が起きない作りにする。これらは、すべて、規定された誤り(64〜68 など)として扱う。たとえば、配列の範囲外の参照、`undefined` や `null` へのプロパティ参照、型の絞り込みの省略(`as` による型の断言)、`JSON.parse` の例外(この仕様の JSON の読み取りは自前の解析なので使わない)が、入力から起きないようにする。

確認した挙動(Node v26.13.0、macOS): 

| 確認 | 結果 |
|------|------|
| `process.on('uncaughtException', ...)` を登録し、タイマーの中で例外を投げる。ハンドラが標準エラーに 1 行書いて `process.exit(70)` を呼ぶ | ハンドラが呼ばれ、プロセスの終了コードは 70 |
| `process.on('unhandledRejection', ...)` を登録し、`Promise.reject(new Error('x'))` を `await` せずに放置する。同じハンドラ | ハンドラが呼ばれ、終了コードは 70 |
| ハンドラがないときの未捕捉の例外(標準出力の `EPIPE` の `'error'` イベント) | スタックトレースを標準エラーに出し、終了コード 1 |

以上のとおり、ハンドラを登録すれば、既定の終了コード 1 が 70 に変わる。ハンドラの中で `process.exit()` を呼ばず、ハンドラが戻る場合、`uncaughtException` のハンドラが登録されていると、Node はプロセスを続行する(例外を処理済みとみなす)ので、**必ず 70 で終わる処理を、ハンドラ自身が行う**(この「続行する」挙動は Node の文書の記述に基づく。確認していない)。ハンドラ自身が例外を投げた場合、Node は終了コード 7 で終わる、と文書にある(未確認。ハンドラの中は、例外を投げない書き方にする)。

`uncaughtException` / `unhandledRejection` のハンドラで捕捉できないもの(Node・V8 の制約。実装時に確認する): メモリ不足(JavaScript のヒープの限界)による異常終了、V8 の致命的なエラー(プロセスが信号 `SIGABRT` などで落ちる)、`kill -9` のような外部からの強制終了は、ハンドラが動かず、終了コードは 70 にならない。この場合は仕様の対象外とする(Node の既知の制約として明記する)。スタックの溢れ(`RangeError: Maximum call stack size exceeded`)は例外として捕捉できる。

ジョブの実行中に想定外の失敗が起きたとき:

- 子プロセスの終了を待たずにプロセスが終わり、ジョブが孤児として残りうる(待つことは必須ではない)。
- ステートは直前の完全な内容のままであること(書き込みは、一時ファイルに書いて `rename` で置き換える形で、途中の内容が見えない形で行う。[06-state-file.md](06-state-file.md))。次の実行は、そのジョブを再実行しうる(少なくとも 1 回の方針)。

実装に任せてよい点:

- 失敗の詳細の保存や標準エラーへの追加の出力。ただし、`conductor: error: internal error` の行と 70 は必須。
- 内部の失敗を起こすための専用のコマンドや、テスト用の差し替えの口は、規定しない(製品の実行ファイルに専用のコマンドはない)。テストでは、`Deps` の差し替えで例外を投げる実装を渡し、入口に相当する関数の戻り値が 70 で、標準出力が空であることを確認してよい。`process.on` のハンドラ自体のテストは、別プロセスで入口を起動して、終了コードと標準エラーを確かめる(受け入れテスト)。

## 7. エラー分類の一覧

内部の型の分け方は 8 章で規定する。利用者から見える区分として、次の分類と対応を守る。

### 7.1 定義の誤り(終了コード 64)

31 種。種類の識別子と、英語・日本語の文は [08-output-and-i18n.md](08-output-and-i18n.md) 4.3 に全件ある(03 章と同じ)。

識別子: `unsupportedFormat`, `fileMissing`, `unreadable`, `tooLarge`, `syntax`, `typeMismatch`, `unknownKey`, `versionMissing`, `versionUnsupported`, `unknownTarget`, `errorTransition`, `backwardOrSelfTarget`, `overlappingExitCodes`, `zeroAsWarning`, `pipelineNameMismatch`, `duplicateJobId`, `invalidJobId`, `noJobs`, `emptyField`, `duplicateTransition`, `invalidEnvName`, `retryNotTable`, `retryMissingKey`, `retryExitCodesEmpty`, `retryExitCodeInvalid`, `retryExitCodesOverlap`, `retryMaxInvalid`, `retryDurationInvalid`, `retryBackoffIncomplete`, `retryMultiplierInvalid`, `retryMaxIntervalTooSmall`。すべて終了コード 64。

`unsupportedFormat` は、定義ファイルの拡張子が `.yaml`・`.yml`・`.json` のどれでもないとき(拡張子の比較は大文字小文字を区別しない)の種類で、原因の文は `拡張子は .yaml、.yml、.json のいずれかで書きます`、行はない(03 章)。

誤りが持つ情報: ファイル(表示するパス)、行(あれば)、種類、原因(日本語の固定の文)。表示は `<ファイル>:<行>: <種類の文>: <原因>`(行がなければ `<ファイル>: <種類の文>: <原因>`)。原因には、入力の値を含めない(入力に由来するのはキー名だけで、`` `キー名` `` の形で入り、64 文字で切る)。原因の文の全件は [03-definition-format.md](03-definition-format.md) にある。

run ID の規則違反(`Empty`, `TooLong`, `InvalidChar`, `LeadingDot` の 4 通り)は、利用者には区別を見せず、すべて `invalid value`(68)にまとめ、値を含めない。

### 7.2 `command` の誤りとステートの誤り

`command` の誤り(終了コード 64): 2 種。`unclosedQuote`(引用符が閉じていない)、`empty`(コマンド名がない)。表示は 08 章 4.4 の種類の文だけ(原因は表示しない。内部の原因の文は、前者が `引用符が閉じていません`、後者が `コマンド名がありません`)。

ステートの誤り(終了コード 65): 6 種。`unreadable`, `tooLarge`, `corrupt`, `versionMismatch`, `writeFailed`, `removeFailed`。持つ情報は、パス、種類、原因。表示は 08 章 4.5。

定義との食い違い(終了コード 65): 3 種。`pipelineName`, `jobIds`, `jobOrder`。表示は 08 章 4.6。

ステートの原因の文(固定の日本語。言語を切り替えない)と、それを返す種類(全 23 件。06 章の表と同じ)。

| 原因の文 | 種類 | 場面 |
|----------|------|------|
| `開く、または読み取ることができません` | `unreadable` | 読み込み、`reset --all` で読めなかった |
| `大きさが上限(1 MiB)を超えています` | `tooLarge` | 読み込み |
| `UTF-8 の文字列ではありません` | `corrupt` | 読み込み |
| `JSON として読めません` | `corrupt` | 読み込み |
| `キーが重複しています` | `corrupt` | 読み込み |
| `format_version がありません` | `corrupt` | 読み込み |
| `format_version が整数ではありません` | `corrupt` | 読み込み |
| `対応していない版です(reset が必要です)` | `versionMismatch` | 読み込み |
| `知らないキーがあります` | `corrupt` | 読み込み |
| `必須のキーがありません` | `corrupt` | 読み込み |
| `値の型が誤っています` | `corrupt` | 読み込み |
| `status の値が誤っています` | `corrupt` | 読み込み |
| `skip_reason の値が誤っています` | `corrupt` | 読み込み |
| `skip_reason が status と合いません` | `corrupt` | 読み込み |
| `exit_code が status と合いません` | `corrupt` | 読み込み |
| `ジョブIDが重複しています` | `corrupt` | 読み込み |
| `job の並びが job_order と一致しません` | `corrupt` | 読み込み |
| `resume_job_id が job_order にありません` | `corrupt` | 読み込み |
| `completed と resume_job_id が両立しません` | `corrupt` | 読み込み |
| `run_id がファイル名の run ID と一致しません` | `corrupt` | 読み込み(`reset --all` では「別の定義のステート」として扱い、消さない) |
| `書き込みに失敗しました` | `writeFailed` | 書き込み |
| `内容を JSON にできません` | `writeFailed` | 書き込み |
| `削除に失敗しました` | `removeFailed` | `reset`、`reset --all`、完了時の削除 |

読み込みの検査の順序と、各原因の判定条件は [06-state-file.md](06-state-file.md)(JSON 版)に従う。複数の原因が当てはまるときにどれを出すかの順序は、この章では未確認(06 章に従う)。

### 7.3 変換の誤り(終了コード 67)

6 種: `fileMissing`, `unreadable`, `tooLarge`, `syntax`, `unrepresentable`, `rootNotMap`。持つ情報は、ファイル、行(あれば)、キーの位置(あれば)、種類、原因。表示は 08 章 4.8。

原因の文(固定の日本語。言語を切り替えない)。変換の規則と、どの場面で使うかは [09-converter.md](09-converter.md)(YAML と JSON の相互変換)が規定する。次の表が全件で、文言は 09 章と完全に同じにする。

| 種類 | 原因の文 | 場面 |
|------|----------|------|
| `fileMissing` | `指定の場所にファイルがありません` | 入力なし |
| `unreadable` | `読み取りの権限がありません` | 権限 |
| `unreadable` | `読み取りに失敗しました` | 読み取りの失敗 |
| `unreadable` | `通常のファイルではありません` | ディレクトリなど |
| `unreadable` | `UTF-8 として読めません` | 文字コード |
| `tooLarge` | `入力が上限(1 MiB)を超えています` | 入力の大きさ |
| `tooLarge` | `要素数が上限(100000)を超えています` | 要素数の上限 |
| `tooLarge` | `文字の大きさが上限(16 MiB)を超えています` | 文字の大きさの上限 |
| `syntax` | `YAML の文法に誤りがあります` | YAML の文法 |
| `syntax` | `JSON の文法に誤りがあります` | JSON の文法 |
| `syntax` | `キーが重複しています` | キーの重複 |
| `syntax` | `キーは文字列で書きます` | 文字列でないキー(YAML) |
| `syntax` | `タグは使えません` | YAML のタグ |
| `syntax` | `複数の文書は書けません(1 ファイルに書ける文書は 1 つです)` | YAML の複数文書 |
| `syntax` | `アンカーが自分自身を含んでいます` | 再帰するアンカー |
| `syntax` | `マージキーの値がマップではありません` | `<<` |
| `syntax` | `入れ子が深すぎます(上限は 32 段)` | 入れ子の深さ |
| `unrepresentable` | `数値が表せる範囲を超えています` | 64 ビット整数・浮動小数の範囲外 |
| `unrepresentable` | `NaN と無限大は JSON で表せません` | YAML の `.nan`、`.inf` を JSON へ |
| `unrepresentable` | `キーが長すぎます(上限 1024 文字)` | 1,024 文字を超えるキーを JSON から YAML へ |
| `rootNotMap` | `最上位がマップではありません` | 最上位 |
| `rootNotMap` | `文書が空です` | YAML の文書が 0 個 |

- `null` は、どちらの形式でも表せる値であり、変換の誤りにならない。日時らしい書式は文字列として読むので、変換の誤りにならない。表の理由は、この 22 件で全件である(未定義のアンカーの参照は、YAML の文法の誤りとして `YAML の文法に誤りがあります` になる)。
- 「キーが長すぎます(上限 1024 文字)」の長さは、コードポイント単位で数える(08 章 11 章。`length` のコードユニット数ではない)。

キーの位置の形式: キーは `.` で連結(各キーは 64 文字で切る)、配列の添字は `[i]`、文字列でないキーは `(非文字列キー)`。例: `job[2].transitions[0]`。

種類と原因の組は、上の表のとおり(09 章と同じ)。

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
| 環境変数の run ID の規則違反 | `InvalidValue` | `run ID (environment variable)` |
| 余分な位置引数 | `UnexpectedArgument` | その語 |
| `convert` で `--to` なし | `MissingArgument` | `--to` |
| `run` のジョブIDが定義にない | `UnknownJob` | そのジョブID |

(引数が非 UTF-8 であることを理由にした `InvalidValue` は、この版には無い。Node が非 UTF-8 を U+FFFD に置き換えて渡し、区別できないため。04 章 4.4。)

### 7.5 シグナルの準備の誤り

2 種。「すでに登録済み」(成功と同じ扱い)、「失敗」(`conductor: error: failed to set up signal handling`、70)。

## 8. TypeScript でのエラー型の設計

### 8.1 方針

- 規定された誤りの伝え方は、モジュールごとに次のとおり決まっている(8.3 節の表の「伝え方」の列)。**値で返す**ものは、結果の型(`{ ok: true, ... } | { ok: false, ... }`)で返し、`Promise` を拒否しない(`definition`、`convert`、`runner`、`flow`、`runid`)。**投げる**ものは、`Error` を継承したクラスの値を `throw`(`async` 関数では `reject`)する(`state` の読み書きの関数。06 章。`cli` の引数の解析の `UsageError` は、値として持ち回る)。`state` の入口 `StateStore`(02 章)は、投げられた `StateError` を値に包んで `Deps` に返す。規定された誤りと、想定外の内部失敗(6 章)を、**型で区別する**。規定された誤りは、このあとの型の集合に属し、それ以外は想定外とする。
- 各誤りの型は、「種別(文字列リテラルの共用体型)」と「利用者に見える情報」を持つ。文言への変換(ロケール、エスケープ)は `src/report/` の責務で、`Error` の `message` は、ロケールを判定せず、デバッグ用の固定の英語の 1 行にする(利用者向けの出力には使わない)。
- 終了コードは、エラー型の値から、`src/cli/` の 1 つの関数で決める(クラスのプロパティ `exitCode`、または型の振り分け)。終了コードの数値の定義は 1 か所(定数)に集める。
- 呼び出し側は、`instanceof` で型を判定する(`catch (e)` の `e` は `unknown` なので、`e instanceof XxxError` で絞り込む。`as` による型の断言はしない)。誤りを包む場合は、標準の `Error` の `cause` オプション(`new XxxError(..., { cause: err })`)に元の誤りを入れる。包んでも、利用者向けの出力は、包んだ誤りの型の情報だけから作る(`cause` や `message` は、利用者に出さない)。
- `Error` を継承したクラスが `instanceof` で正しく判定されるように、`tsconfig.json` の `target` は `ES2022` 以上にする(`ES5` では、組み込みの `Error` を継承したクラスの `instanceof` が壊れる問題がある。クラス構文がそのまま実行時に使われる目標では起きない。この点は一般的な既知の制約で、実装時にテストで確かめる)。モジュールは ESM(`NodeNext`)の単一のコピーなので、同じクラスが複数のコピーに分かれて `instanceof` が失敗する問題は起きない(同じパッケージを別の場所から二重に読み込まない)。
- 文字列の識別子(7.1 の `fileMissing` など)は、種別の値そのもの(文字列リテラル)として持つ。種別の値は、永続化も外部への公開もしない(利用者に見えるのは文言だけ)。並びの追加・並べ替えで、互換性の問題は起きない。

### 8.2 定数と共通の基底クラス

```ts
// 終了コード(1 章)。値は 1 か所の定数に集める。
export const ExitCode = {
  OK: 0,
  UNLISTED_ZERO: 1,
  DEFINITION: 64,
  STATE: 65,
  REJECTED: 66,
  CONVERSION: 67,
  USAGE: 68,
  INTERNAL: 70,
  START_FAILURE: 127,
  SIGNAL_BASE: 128,
} as const;

// すべての conductor の規定された誤りが継承する。
export abstract class ConductorError extends Error {
  abstract readonly exitCode: number;
  // 利用者向けの文言を作るための判別子(文字列リテラル)。
  abstract readonly type: string;
  constructor(message: string, options?: { cause?: unknown }) {
    super(message, options);
    this.name = new.target.name;
  }
}
```

### 8.3 型の一覧(骨格)

モジュールごとに、自分の誤りの型を持つ(ソース構成は 02 章)。

| モジュール | 型 | 種別の型 | 終了コード | 持つ情報(伝え方) |
|------------|-----|----------|------------|----------|
| `src/definition/` | `DefinitionError` | `ErrorKind`(7.1 の 31 種の文字列リテラルの共用体。03 章 12.1 節) | 64 | ファイル、行・列(`undefined` は「なし」)、種類、原因(日本語の固定の文)。クラスにせず、普通の値で、`LoadResult` に入れて返す(03 章) |
| `src/runner/` | `CommandError` | `CommandErrorKind`(`"unclosedQuote" \| "empty"`。02 章) | 64 | 種類(ジョブIDは、呼び出し側が持つ)。`CommandResult` で返す(07 章) |
| `src/runid/` | `RunIdError` | `RunIdErrorKind`(`"Empty" \| "TooLong" \| "InvalidChar" \| "LeadingDot"`) | 68(利用者には `invalid value`) | 種類のみ(値は持たない)。`RunIdParse` で返す(05 章 6 節) |
| `src/state/` | `StateError` | `StateErrorKind`(`"unreadable" \| "tooLarge" \| "corrupt" \| "versionMismatch" \| "writeFailed" \| "removeFailed"`) | 65 | パス、種類、原因(固定の日本語の文)、`detail`(任意)。`state` の読み書きの関数は投げる(06 章 6.2 節) |
| `src/state/` | `ConsistencyResult` の `mismatch` | `what`(`"pipelineName" \| "jobIds" \| "jobOrder"`) | 65 | 種類。値で返す(06 章 7 節) |
| `src/convert/` | `ConvertError` | `ConvertErrorKind`(7.3 の 6 種) | 67 | ファイル、行、キーの位置、種類、原因 |
| `src/cli/` | `UsageError` | `UsageKind`(7.4 の 8 種) | 68 | 種類、対象(あれば) |
| `src/cli/` | `RejectedError` | `RejectReason`(`"OutOfOrder" \| "SkipTarget" \| "AfterEnd" \| "AlreadyExecuted"`) | 66 | ジョブID、理由 |
| `src/flow/` | (クラスにしない。`JobOutcome` の値) | (なし。分類の結果として扱う) | 3 章から決まる | 報告する終了コード、エラーか |

- 種別は、文字列リテラルの共用体型にする(`enum` は使わない。実行時の値と型の対応が単純で、`tsc` が `switch` の網羅性を検査できるため)。網羅性の検査には、`switch` の `default` で `never` 型の変数に代入する書き方を使う。
- 「対象なし」と「対象が空文字」を区別する(`unknown command: ''` のように、対象があれば `'...'` で囲んで出す)。`target?: string` として、`undefined` を「対象なし」、`""` を「対象が空文字」とする(`target` が `undefined` かどうかを `=== undefined` で判定する。`if (e.target)` の真偽判定は、空文字を「なし」と誤るので使わない)。
- 判別共用体を使う代替案: クラスにせず、`type UsageError = { type: "usage"; kind: UsageKind; target?: string }` のようなプレーンなオブジェクトを `throw` する方法も許す。ただし、`throw` する値が `Error` でないと、スタックの情報が取れず、`unhandledRejection` や `uncaughtException` のハンドラで型が `unknown` になるので、この仕様はクラス(`Error` の継承)を採る。代替案を採る場合も、外から見える挙動(終了コード、文言)は同じにする。

```ts
export type UsageKind =
  | "UnknownCommand" | "MissingArgument" | "UnexpectedArgument" | "UnknownOption"
  | "OptionNotAllowed" | "MissingValue" | "InvalidValue" | "UnknownJob";

export class UsageError extends ConductorError {
  readonly exitCode = ExitCode.USAGE;
  readonly type = "usage" as const;
  constructor(
    readonly kind: UsageKind,
    // undefined は「対象なし」。"" は「対象が空文字」。
    readonly target?: string,
  ) {
    super(`usage error: ${kind}`); // デバッグ用の固定の英語(利用者向けの出力には使わない)
  }
}
```

### 8.4 ジョブの結果と終了コード

ジョブの結果(3 章)は、エラー型ではなく、`src/flow/` の値(分類、報告する終了コード)として表す。`run` / `pipeline` の処理は、結果を `run` の戻り値の終了コードに変換する。ステートの書き込みの失敗(`StateError`)は、ジョブの結果より優先して終了コード 65 にする(2 章の順 11)。

### 8.5 最上位での決定

`src/cli/` の `run` は、各処理が返した結果の型(値で返す誤り)と、投げられた(または `reject` された)値を受け取り、次の順で終了コードと表示を決める(終了コードを決める関数は 1 つ)。

1. `instanceof UsageError` → 使い方の誤りの表示(誤りの行、空行、使い方全文)と 68。
2. `DefinitionError` / `CommandError` → 64。`StateError`、整合性の `mismatch` → 65。`RejectedError` → 66。`ConvertError` → 67。それぞれの表示は 08 章。
3. 上のどれでもない値(`ConductorError` のほかのクラスの `Error`、`Error` でない値を含む。想定していない失敗)は、`conductor: error: internal error` を標準エラーに 1 行出して 70 で終わる(6 章と同じ文。表示は 6 章の 1 つの関数に任せる)。想定外の値を握りつぶして成功にしてはならない。
4. 型の判定の順序は、同じ値が複数のクラスに当てはまらないように作る(各クラスは独立で、`ConductorError` 以外のクラスの継承関係を作らない)ので、実際には順序に依存しない。

非同期の処理の `await` を忘れた `Promise` の `reject` は、`unhandledRejection` になり、6 章で 70 になる。すべての `Promise` は、`await` するか、`.catch` を付ける(ESLint の `@typescript-eslint/no-floating-promises` を有効にして、機械的に検出する。11 章)。

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

- 1 章の終了コードの全件と意味、2 章の決定順序、3 章の分類、4 章の範囲合わせ(終了コードは、0〜255 の整数にしてから `process.exitCode` に渡す)。
- 6 章の 70(固定の 1 行と終了コード)と、`uncaughtException`・`unhandledRejection`・入口の `try/catch` の 3 つの捕捉。
- 標準エラーの `conductor: error: ...` の行の形式と文言(08 章)。
- 少なくとも 1 回の方針: ステートの書き込みに失敗したとき、ジョブの終了コードより 65 を優先する。
- 8 章の、`Error` を継承した型(または判別共用体)、`instanceof` での判定、種別の文字列リテラルの共用体型という方針(型の名前と分け方は目安)。

実装に任せてよい点:

- 型の名前、フィールドの名前、モジュール間の細かい分け方(8.3 は目安)。
- 同時に届いた複数のシグナルのうち、どれを「最初」とみなすか。
- 想定外の失敗の詳細を標準エラーへ追加で出すか。
- 判別共用体(クラスにしない)を採るか(8.3。ただし、既定はクラス)。

## 11. この版が採った選択(まとめ)

| 項目 | 内容 |
|------|------|
| ステートの原因の文 | `JSON として読めません`、`キーが重複しています`、`内容を JSON にできません`(7.2) |
| 定義の誤り | 31 種。`unsupportedFormat`(拡張子が `.yaml`・`.yml`・`.json` のどれでもない)を含む(7.1) |
| `retryNotTable` の文言 | 08 章 4.3(`retry must be a map` / `retry はマップで書きます`) |
| 変換の誤り | YAML と JSON の相互変換。6 種の種類と 22 件の原因(7.3) |
| 想定外の内部失敗 | `process.on('uncaughtException')`、`process.on('unhandledRejection')`、入口の `try/catch` で 70(6 章) |
| 終了コードの型 | `number`(32 ビット符号つき整数の範囲)。`bigint` は使わない(4 章) |
| 終了コードの整形 | `reported & 0xFF`、エラーで 0 になれば 1(4 章) |
| エラー型 | `Error` を継承したクラス + 文字列リテラルの共用体の種別。`instanceof` で判定(8 章) |

## 12. 未確認

- 定義の誤りの YAML の文法の誤りの行番号(採用する YAML ライブラリの検出位置による。03 章)。
- ステートの読み込みの検査の順序は 06 章で固定してあり、この章に未確認はない。
- Windows で子プロセスの終了コードが Node から渡るときの符号(3 章の注)、および、`process.exitCode` に範囲外の整数を渡したときの Windows での切り詰め(4.1。POSIX は確認済み)。
- `uncaughtException` のハンドラが戻った場合にプロセスが続行すること、ハンドラ自身が例外を投げたときの終了コード 7(6 章。Node の文書の記述に基づく。確認していない)。
- 6 章の確認(ハンドラが登録されていれば 70 で終わる)は、Node v26.13.0・macOS のみ。サポートする 2 つの LTS 系列・Linux・Windows での確認は、実装時に行う。
- `spawn` に失敗した場合の `'error'` と `'close'` の発行の順序と有無(3 章)。07 章と合わせて、実装時に確認する。
- メモリ不足や V8 の致命的な異常終了での終了コード(6 章。仕様の対象外)。
- `@typescript-eslint/no-floating-promises` を含む ESLint の規則の名前と、設定の詳細(実装時に確認する。11 章)。
