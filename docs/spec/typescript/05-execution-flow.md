# 05. 実行の意味論(終了コード・分岐・再開・run ID・リトライ・パイプライン)

対象: TypeScript(Node.js)で実装する `conductor` v0.8.0 と同じ挙動の実行ツール。この文書は、`run`(ジョブを 1 つ実行)と `pipeline`(定義の全ジョブを順に実行)の実行の流れを、判定表・状態遷移・擬似コードで定める。数値の計算(待ち時間の倍率・上限・飽和・変換)は、`bigint` と `number` の使い分け、式、境界の例まで定める。待機とシグナルによる中断は、07 章の `Interrupt` と `waitRetry` を使う方式に決める(§7.8)。この文書だけで、TypeScript でゼロから同じ挙動を実装できることを目標にする。

ステートファイルの論理項目・保存の性質・読み込みの検査は [06-state-file.md](06-state-file.md) にある。この文書の「ステートへの記録」は、06 の形式のファイルを書く前の、メモリ上のステート(以下「ステート値」)の更新を指す。定義ファイルの書式は [03-definition-format.md](03-definition-format.md)、コマンドラインは [04-cli.md](04-cli.md)、プロセス起動とシグナルは [07-process-and-signals.md](07-process-and-signals.md)、メッセージの文言は [08-output-and-i18n.md](08-output-and-i18n.md)、終了コードの総覧は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md) を参照。

この文書の確認の範囲: 「Node.js の挙動」と書いた点のうち「確認済み」と明記したものは、Node.js v26.13.0(macOS)で実際に動かして確かめた。Windows と、ほかの版は未確認(§14)。

## 0. 構成の決定

### 0.1 モジュールの置き場所

「判断」(分類・次の動き・起動前の判断・リトライの判断・再開位置)は、ファイルにもプロセスにも触れない**純粋な関数**にし、例外を投げず必ず結果を返す。

| ディレクトリ | 置くもの |
|---|---|
| `src/flow/` | 共有の値の型(`JobStatus`、`SkipReason`、`JobState`、`PipelineState`、`JobExecution` ほか)、`classify`、`classifyAndDecide`、`nextStep`、`decideBeforeRun`、`retryDecision`、`nextWaitSecs`、`satMulU64`、`resumePointOnInterrupt`、`exitCodeFor`、`signalExitCode`。`node:` のモジュールを一切 import しない(ファイル・時計・環境変数・シグナルに触れない) |
| `src/state/` | 記録の関数(`applyOutcome`、`applySkip`、`applyInterrupt`、`initialState`。純粋。複製して更新)、読み書き([06](06-state-file.md)) |
| `src/runid/` | run ID の検査と決定(§6) |
| `src/runner/` | `runJob`(子プロセスの起動と待機)、`waitRetry`(§7.8。07 章 7.5 節) |
| `src/cli/` | `prepare`、`run`・`pipeline` の流れ(§5)、リトライの実行ループ(§7.7)。`Deps` を介して時計・環境・ファイル・プロセスに触れる |

依存の向きは `definition` ← `flow` ← (`state`、`runner`) ← `cli`。`flow` は `definition` の型だけを使い、`state` と `runner` は `flow` の共有の型を使う。`state` と `runner` は互いに依存しない。循環させない([02-architecture.md](02-architecture.md))。

### 0.2 型と表記の約束(この文書の全体)

| 値 | TypeScript の型 | 理由 |
|---|---|---|
| 終了コード(ジョブ自身のもの、`reported_exit_code`、終了コードの一覧の値) | `number`(32 ビット符号付き整数の範囲に収まる整数) | 範囲が 2^31 未満で、`number`(倍精度)で誤差なく表せる。整形(§2.2)は `&` などのビット演算を使うが、`&` は被演算子を 32 ビット符号付き整数に変換するので、範囲内の値では正しく動く(確認済み: `(-1) & 0xFF` は 255、`(-256) & 0xFF` は 0) |
| 待ち時間(秒。`interval`、`max_interval`、各回の待ち、飽和の演算の値) | `bigint`(0〜2^64-1 の範囲で扱う) | 64 ビット符号なし整数の飽和演算を仕様どおりに行うため。`number` は 2^53 を超える整数を表せない(確認済み: `9007199254740993` を `JSON.parse` すると `9007199254740992` になる)。`bigint` の乗算は桁あふれしないので、結果を 2^64-1 と比べて飽和させる(§7.6) |
| 倍率 `multiplier`(2〜10)、`max`(1〜100)、`retriesDone`、リトライの回数 `n` | `number`(整数) | 小さい値で誤差がない。`bigint` との演算の直前に `BigInt(multiplier)` で変換する |
| シグナル番号 | `number` | 終了コード `128 + n` の計算に使う |
| 時刻・経過時間(待機の計測) | `bigint` のナノ秒(`process.hrtime.bigint()`) | 単調増加。実時計の調整に影響されない |

- 「無し」(値がない)は、`JobExecution` では `null` で表す(07 章 4.4 節)。ステート値(`JobState` の `exitCode` など)では、プロパティを書かない(または `undefined`)で表す(ステートの JSON の `null` は読み込みで拒否する。06)。比較(ステート値の等しさ)は、「無し」と `undefined` を同じとみなす項目ごとの比較で行う(参照の同一性ではない。§4.3)。
- ステート値と判断の入出力は読み取り専用(`readonly`)の型にし、更新は新しい値を作る(元は変えない)。
- tsconfig は `strict`。`noUncheckedIndexedAccess` を有効にすると、配列の添字アクセスで `undefined` の考慮が型で強制されるので推奨する(必須ではない)。
- 数値の飽和や変換は、`number` の浮動小数点に頼らない。`Number(bigintValue)` は、値が `Number.MAX_SAFE_INTEGER`(2^53-1)以下と分かっている場合だけ使う(確認済み: `Number(2n**63n)` は `9223372036854776000`、つまり丸められる)。

---

## 1. 用語とデータ

### 1.1 ジョブの状態(`JobStatus`)

| 値(ステート上の文字列) | 意味 |
|---|---|
| `pending` | まだ起動していない。記録がないジョブも pending と同じに扱う |
| `success` | 成功(成功コードで終わった) |
| `warning` | 警告(警告コードで終わった) |
| `error` | エラー |
| `skipped` | スキップ。理由 `skip_reason` が必ず付く |

- 「済んだ(settled)」状態 = `success` / `warning` / `skipped`。
- 「結果のある」状態 = `success` / `warning` / `error`(終了コードを持てる)。
- スキップ理由(`SkipReason`): `byTransition`(遷移によるスキップ)、`byEnd`(`@end` によるスキップ)。

型の骨格(`src/flow/types.ts`。メモリ上の名前は camelCase。ステートファイルのキー名は snake_case で、対応は [06](06-state-file.md) §2):

```ts
export type JobStatus = "pending" | "success" | "warning" | "error" | "skipped";
export type SkipReason = "byTransition" | "byEnd";

export interface JobState {
  readonly id: string;
  readonly status: JobStatus;
  readonly exitCode?: number;       // 結果のある状態だけ。無しもある
  readonly skipReason?: SkipReason; // status === "skipped" のときだけ
}

export interface PipelineState {
  readonly definitionKey: string;   // 定義ファイルのパス(definitionPath と同じ値)
  readonly definitionPath: string;
  readonly pipelineName: string;
  readonly runId?: string;
  readonly jobOrder: readonly string[];
  readonly jobs: readonly JobState[]; // jobOrder と同じ順・同じ数
  readonly resumeJobId?: string;
  readonly endReached: boolean;
  readonly completed: boolean;
}
```

### 1.2 1 回の実行結果(`JobExecution`)

ジョブを 1 回起動して終わるまでの観測結果。

| フィールド | 型 | 意味 |
|---|---|---|
| `jobId`、`command` | `string`、`Command` | 定義のジョブIDと `command`(表示とステートには `jobId` だけを使う) |
| `started` | `boolean` | 起動した(子プロセスの生成に成功した)か。「中断の合図が起動の前に立っていたため起動しなかった」ときだけ `false`。ほかの起動失敗でも `started` は `true` のまま(`startFailure` を見る) |
| `exitCode` | `number \| null`(32 ビット符号付き整数) | プロセスの終了コード。取れないとき(シグナル終了、起動失敗)は `null` |
| `terminatedBySignal` | `number \| null` | ジョブがシグナルで終了したときの番号(UNIX 系のみ) |
| `startFailure` | `boolean` | 起動に失敗した。command を引数に分けられない、プロセス生成の失敗、終了状態を取得できない、のいずれか |
| `interrupted` | `boolean` | conductor がシグナルを受け取り、中断の合図が立っている(ジョブの終了を観測した直後に立っていた場合を含む) |
| `signalNumber` | `number \| null` | `interrupted` のとき、最初に受け取ったシグナルの番号 |

```ts
// 型の正本は 07 章 4.4 節と 02 章 2.1 節(「無し」は null で表す)。
export interface JobExecution {
  readonly jobId: string;
  readonly command: Command;
  readonly started: boolean;
  readonly exitCode: number | null;
  readonly terminatedBySignal: number | null;
  readonly startFailure: boolean;
  readonly interrupted: boolean;
  readonly signalNumber: number | null;
}
```

ジョブの起動手順(`runJob`。子プロセスの API と詳細は [07](07-process-and-signals.md)):

1. command を引数に分ける(規則は 07)。失敗なら `startFailure = true`(`started = true`)で終わる。
2. 中断の合図が立っていれば、`started = false, interrupted = true, signalNumber = 番号` で終わる(起動しない)。
3. 子プロセスを生成する。生成の失敗(`spawn` が `error` イベントを出す、または例外を投げる)なら `startFailure = true` で終わる。
4. 終了まで待つ。待機中に合図が立てば、子プロセスに同じシグナルを**1 回だけ**転送して、終了を待ち続ける。
5. 終了コードとシグナル終了を記録する。終了状態を取得できなければ(例: `close` イベントの `code` も `signal` も無い、または `signal` の名前に対応する番号がない)`exitCode`・`terminatedBySignal` とも無しのままにする。この場合は §2.3 の順 5 に当たり、`conductor_error` つきのエラー(終了コード 1)になる。
6. 終了を観測した直後に合図が立っていれば `interrupted = true, signalNumber = 番号`。

### 1.3 結果(`JobOutcome`)

| フィールド | 意味 |
|---|---|
| `classification` | `success` / `warning` / `error`、または**分類なし**(「起動しなかった中断」。型では `undefined`) |
| `reportedExitCode` | スケジューラへ返す終了コード(`number`)。後で 0〜255 に整える(§2.2) |
| `conductorError` | 「成功コードに含まれない 0」など、conductor 自身が原因を標準エラーに表示するエラーのとき `true` |

```ts
export type Classification = "success" | "warning" | "error";
export interface JobOutcome {
  readonly classification?: Classification; // 無し = 分類なし
  readonly reportedExitCode: number;
  readonly conductorError: boolean;
}
```

### 1.4 実行方式(`RunMode`)

`"individual"`(`conductor run <job-id>`)と `"pipeline"`(`conductor pipeline <name>`)。`@end` の扱いと完了の判定だけが違う(§3、§5)。

---

## 2. 終了コード

### 2.1 conductor が返す終了コード(プロセスの終了コード)

| 値 | 名前 | 意味 |
|---|---|---|
| 0 | OK | 成功、警告、スキップ、正常終了 |
| ジョブの値(1〜255) | - | エラーになったジョブの終了コードをそのまま(範囲外は §2.2) |
| 1 | - | 成功コードに含まれない 0 で終わったジョブ(`conductorError`)。終了コードを取れなかったエラーも 1 |
| 64 | DEFINITION | 定義の誤り(ファイルがない・読めない・検証の誤り・パイプライン名の不一致・command の誤り) |
| 65 | STATE | ステートの誤り(読めない・壊れている・版の不一致・定義との食い違い・書き込み/初期化/削除の失敗)、完了を判定できない、`watch` の出力失敗 |
| 66 | REJECTED | 拒否(順序外) |
| 67 | CONVERSION | convert の失敗(この文書の対象外) |
| 68 | USAGE | 使い方の誤り(知らないジョブID、run ID の値の誤りを含む) |
| 70 | INTERNAL | 内部の誤り(想定外の異常終了、シグナル処理の準備の失敗) |
| 128 + 番号 | - | ジョブがシグナルで終了した、または conductor が中断された(SIGINT = 130、SIGTERM = 143) |

64〜68・70 は、ジョブ自身の終了コードと重なりうる。区別は標準エラーの `conductor: error: ...` の行で行う。

### 2.2 `exitCodeFor(reported, isError)`: 0〜255 への整形

プロセスの終了コードは 0〜255 に収める。

```text
if 0 <= reported <= 255:  return reported
low = reported & 0xFF              # 負数も含め、2 の補数表現の下位 8 ビット
if low == 0 and isError: return 1  # エラーを成功に見せない
return low
```

TypeScript では `reported & 0xFF` がそのまま使える(`reported` が 32 ビット符号付き整数の範囲内ならば。`&` は 32 ビット符号付き整数への変換を伴う)。範囲外の値(2^31 以上、非整数)は、入力の検査で排除済みのため渡さない。

| reported | isError | 結果 |
|---|---|---|
| 3 | true | 3 |
| 256 | true | 1(下位 8 ビット 0 → 1) |
| 256 | false | 0 |
| 257 | true | 1 |
| 511 | true | 255 |
| -1 | true | 255 |
| -256 | true | 1 |

`isError` は `classification === "error"`。分類なし(起動しなかった中断)と成功・警告は `false`。

中断の終了コード `signalExitCode(n)` = `exitCodeFor(satAddI32(128, n), true)`(パイプラインのジョブのあいだの中断で使う)。

「32 ビット符号付き整数の飽和加算」(`satAddI32`): 結果が 2147483647 を超えるなら 2147483647、-2147483648 を下回るなら -2147483648 にする(例: 128 + 2147483647 = 2147483647)。`number` は 2^53 まで正確なので、`Math.min(Math.max(a + b, -2147483648), 2147483647)` で足りる。実在のシグナル番号(1〜64 程度)では起きない。

### 2.3 成功・警告・エラーの分類と返す値(`classify`)

- **成功コードの集合** = 定義の `success_exit_codes` があればその値だけ(0 を書かなければ 0 でも成功にならない)、無ければ `[0]`。
- **警告コードの集合** = `warning_exit_codes`(0 を含まず、成功コードと重ならないことは定義の検証で保証済み)。

判定は次の順に、最初に当てはまるものを採る(上ほど優先)。入力は 1 回の実行結果(§1.2)とジョブ定義。

| 順 | 条件 | classification | reportedExitCode | conductorError |
|---|---|---|---|---|
| 1 | `started = false` かつ `interrupted` | なし | `128 + signalNumber`(無ければ `128 + 0`)、飽和加算 | false |
| 1' | `started = false` かつ `interrupted` でない(あり得ない組み合わせ) | error | 127 | false |
| 2 | `interrupted`(起動済み) | error | `128 + signalNumber`(無ければ 128)。ジョブ自身の終了コードは見ない | false |
| 3 | `startFailure` | error | 127 | false |
| 4 | `terminatedBySignal` が有り(値 n) | error | `128 + n` | false |
| 5 | `exitCode` が無い(起動して終わったのに終了コードがない) | error | 1 | **true** |
| 6 | `exitCode` ∈ 成功コード | success | 0 | false |
| 7 | `exitCode` ∈ 警告コード | warning | 0 | false |
| 8 | `exitCode = 0`(成功コードに無い) | error | 1 | **true** |
| 9 | それ以外 | error | `exitCode` そのまま(負数も、255 超もそのまま保持。整形は §2.2) | false |

全組み合わせの例(定義 `warning_exit_codes: [3, 4]`、`success_exit_codes` 省略):

| 実行結果 | 分類 | 返す値 | プロセス終了コード |
|---|---|---|---|
| exit 0 | success | 0 | 0 |
| exit 3 / 4 | warning | 0 | 0 |
| exit 1 / 2 / 255 | error | 1 / 2 / 255 | 同じ |
| exit -1(Windows 等) | error | -1 | 255 |
| 起動失敗 / command 不正 | error | 127 | 127 |
| SIGKILL(9)で終了 | error | 137 | 137 |
| 実行中に SIGTERM(15)を受信(ジョブが exit 0 でも) | error | 143 | 143 |
| 起動前に SIGINT(2)が立っていた | 分類なし | 130 | 130 |

`success_exit_codes: [2]` のとき: exit 2 → success(0)、exit 0 → error(1, `conductorError`)。`success_exit_codes: [0, 2]` のとき exit 0 も success。

標準エラーへの常時の表示(`-v` なしでも):

- `conductorError = true`: `conductor: error: job '<id>': exit code 0 is not listed in success_exit_codes`(日本語ロケールでは日本語。文言は [08-output-and-i18n.md](08-output-and-i18n.md))。`conductorError = true` なら、順 5(終了コードが無い)の場合も同じ文言を出す(実装は `conductorError` が真なら、理由を区別せず同じ行を出す)。
- 起動失敗: `conductor: error: job '<id>': cannot start command '<program>'`。

---

## 3. 遷移と次の動き(`classifyAndDecide` / `nextStep`)

ジョブ実行後に、分類と次の動き(`NextStep`)を決める。

### 3.1 `NextStep` のフィールド

| フィールド | 意味 |
|---|---|
| `nextJobId` | 次に実行するジョブ。完了・停止のとき無し(`pipeline` コマンドは、この値を使わない。§5.3) |
| `skippedJobIds` | この時点でスキップとして記録するジョブ |
| `skipReasonKind` | その理由(`byTransition` / `byEnd` / なし) |
| `endReached` | `@end` に到達した |
| `completed` | 完了した |
| `resumeJobId` | エラー等で停止したときの再開位置(そのジョブ)。無しもある |

### 3.2 判定表

入力: 分類、そのジョブの該当する遷移(`on = success` は成功のとき、`on = warning` は警告のときに参照する。`error` の遷移は書けない)、`RunMode`、そのジョブの位置(index。定義の記述順、0 始まり)、各ジョブの現在の状態。

「未完了のジョブ」= 指定した区間のジョブのうち、状態が success / warning / skipped **でない**もの(pending・error・記録なし)。区間は、`@end` なら「index より後ろ全部」、遷移先 `T` なら「index と T のあいだ(両端を含まない)」。

| 分類 | 遷移 | RunMode | nextJobId | skippedJobIds / kind | endReached | completed | resumeJobId |
|---|---|---|---|---|---|---|---|
| error | (見ない) | 両方 | なし | 空 / なし | false | false | **このジョブ** |
| 分類なし(起動しなかった中断) | (見ない) | 両方 | なし | 空 / なし | false | false | **このジョブ** |
| success / warning | `@end` | pipeline | なし | 後ろの未完了ジョブ全部 / `byEnd`(空なら kind なし) | true | **true** | なし |
| success / warning | `@end` | individual | なし | **空** / kind = `byEnd`(後ろに未完了が 1 つでもあるとき。無ければ kind なし) | true | 後ろに未完了が無ければ true、あれば false | なし |
| success / warning | ジョブ `T`(index より後ろ) | 両方 | `T` | index と T の間の未完了ジョブ / `byTransition`(空なら kind なし) | false | false | なし |
| success / warning | 遷移なし(または遷移先が後ろのジョブでない。検証済み定義では起きない) | 両方 | 最後のジョブなら なし、そうでなければ次のジョブ | 空 / なし | false | 最後のジョブなら true、そうでなければ false | なし |
| 定義にないジョブ | - | - | error と同じ(停止) | | | | |

要点:

- 成功・警告のあと再開位置は無し。エラー・分類なしのあとは必ずそのジョブ。
- individual の `@end` では、後ろのジョブを今は記録せず(`skippedJobIds` は空)、`endReached = true` だけを記録する。後ろのジョブは、スケジューラが 1 つずつ `run` を呼んだときに `Skip(AfterEnd)` が返り、呼ばれたジョブだけ `skipped(byEnd)` と記録される(§4.2、§5.4)。
- 遷移先 `T` の遷移では、`T` へ飛ぶ。`T` が index+1 なら間は空で、スキップは無い。
- 自動のリトライ(ジョブ失敗の再実行)は、`retry` を書かない限りしない(§7)。

### 3.3 `applyOutcome`(結果の記録。メモリ上)

入力: 現在のステート値 `S`、ジョブID、実行結果、結果(`JobOutcome`)、`NextStep`。`S` を複製して更新し、返す。

1. 対象ジョブ(`S` のジョブ状態一覧から id で探す。見つからなければこの手順 1 を飛ばす):
   - `classification` があれば `status` を success / warning / error にし、`skipReason` を消す。分類なし(起動前の中断)なら `status` は**変えない**。
   - `exitCode` = 実行結果の `exitCode`(無ければ、以前の値も消す)。そのあと `status` が success / warning / error のいずれでもなければ `exitCode` を消す。
2. `skippedJobIds` の各ジョブ(対象ジョブ自身は除く)を `status = skipped`、`exitCode` 無し、`skipReason = skipReasonKind`(無ければ `byTransition`)にする。
3. `resumeJobId = NextStep.resumeJobId`、`completed = NextStep.completed`。
4. `endReached = S.endReached || NextStep.endReached`(一度 true になったら戻さない)。

### 3.4 `applySkip`(スキップの記録。メモリ上)

入力: ステート値 `S`、ジョブID、判断の理由(`DecisionReason`: `AlreadyExecuted` / `SkipTarget` / `AfterEnd` / `OutOfOrder`)、定義。

- 理由が `AlreadyExecuted` / `OutOfOrder` → 何も変えない(複製を返す)。
- `SkipTarget` → そのジョブを `skipped(byTransition)`、`AfterEnd` → `skipped(byEnd)` にし(`exitCode` 無し)、`AfterEnd` なら `endReached = true`。ジョブが `S` に無ければ、更新せず複製を返す。
- そのあと、`S` に `error` のジョブが 1 つでもあれば、`resumeJobId` と `completed` は**変えない**(エラーの再開位置を守る)。無ければ:
  - `firstPending` = 定義の記述順で最初の `pending`(記録なしを含む)のジョブ(無ければ無し)
  - `completed` = 「いまスキップしたジョブが定義の最後のジョブ、かつ `firstPending` が無い」
  - `resumeJobId = firstPending`

### 3.5 `applyInterrupt`(中断の記録。メモリ上)

再開位置(§9)の値で更新する: `resumeJobId = ResumePoint.resumeJobId`、`completed = ResumePoint.completed`、`endReached = S.endReached || ResumePoint.endReached`。ジョブの状態は変えない(エラーとして記録しない)。

---

## 4. 起動前の判断(`decideBeforeRun`)

### 4.1 入力と出力

入力: ジョブID、定義、各ジョブの状態の一覧、`resumeJobId`(ステートの同名の値)、`endReached`。ステートが空(ファイルなし・長さ 0・完了のまま残ったもの)のときは、状態一覧は空、`resumeJobId` 無し、`endReached = false` を渡す。

出力: `Run`(実行する)/ `Skip(理由)`(実行せず 0 で終わる。理由は `SkipTarget` / `AfterEnd` / `AlreadyExecuted`)/ `Reject(OutOfOrder)`(実行せず、ステートも変えず、66)。

```ts
export type DecisionReason = "AlreadyExecuted" | "SkipTarget" | "AfterEnd" | "OutOfOrder";
export type Decision =
  | { readonly kind: "run" }
  | { readonly kind: "skip"; readonly reason: "SkipTarget" | "AfterEnd" | "AlreadyExecuted" }
  | { readonly kind: "reject"; readonly reason: "OutOfOrder" };
```

### 4.2 判定表(上から最初に当てはまるものを採る)

状態 `st` = そのジョブの記録(無ければ pending)、`reason` = そのスキップ理由。

| 順 | 条件 | 結果 |
|---|---|---|
| 0 | ジョブIDが定義にない(`run` では、事前に 68 で止まるので到達しない) | `Reject(OutOfOrder)` |
| 1 | `st = skipped` かつ `reason = byEnd` | `Skip(AfterEnd)` |
| 1' | `st = skipped` かつ `reason` が byTransition または無し | `Skip(SkipTarget)` |
| 2 | `endReached` かつ `st = pending` | `Skip(AfterEnd)` |
| 3 | `st` が success または warning | `Skip(AlreadyExecuted)` |
| 4 | `st = error`、または `resumeJobId` がこのジョブID | `Run` |
| 5 | 全ジョブが pending(または記録なし)で、このジョブが記述順で最初 | `Run` |
| 6 | `st = pending` で、記述順で前のジョブがすべて success / warning / skipped | `Run` |
| 7 | 上のどれでもない | `Reject(OutOfOrder)` |

例(4 ジョブ j0〜j3):

| 呼ぶジョブ | 状態 | resume | endReached | 結果 |
|---|---|---|---|---|
| j1 | j0=success, j1=skipped(byTransition) | - | false | Skip(SkipTarget) |
| j2 | j2=skipped(byEnd) | - | true | Skip(AfterEnd) |
| j3 | j0=success(j3 記録なし) | - | true | Skip(AfterEnd) |
| j0 | j0=success | - | false | Skip(AlreadyExecuted) |
| j0 | j0=warning | - | false | Skip(AlreadyExecuted) |
| j1 | j0=success, j1=error | j1 | false | Run |
| j1 | j1=error | - | false | Run |
| j2 | j0=success, j1=skipped | j2 | false | Run |
| j0 | (空) | - | false | Run |
| j0 | 全部 pending | - | false | Run |
| j2 | j0=success, j1=warning | - | false | Run |
| j3 | j0=success, j1=skipped, j2=skipped | - | false | Run |
| j1 | (空) | - | false | Reject |
| j2 | j0=success | - | false | Reject |
| j2 | j0=success, j1=error | - | false | Reject |

補足: 規則 1' は、理由が書かれていない `skipped`(ステートファイル上は読み込みで弾かれるが、関数としては入りうる)を SkipTarget とする。規則 2 は `pending`(記録なしを含む)のときだけ。`error` のジョブは規則 4 で、`endReached` でも `Run` になる。規則 6 は「記述順で前のすべて」を見るので、後ろのジョブが済んでいても影響しない。

### 4.3 判断の結果とステート・終了コード(呼び出し側)

| 結果 | ステートの変更 | 標準エラー | 終了コード |
|---|---|---|---|
| `Reject(r)` | しない(初期ステートも書かない) | `conductor: error: job '<id>': cannot run this job now: earlier jobs have not finished (out of order)`(日本語: `このジョブは、いま実行できません: 前のジョブが終わっていません(順序外)`) | 66 |
| `Skip(r)` | `applySkip` の結果が元と**等しくなければ**、耐久性ありで書く。`AlreadyExecuted` は常に等しい(書かない) | `-v` のとき `[conductor] job=<id> skipped (<理由>)`。理由文: SkipTarget = `skipped by a transition`、AfterEnd = `skipped after @end`、AlreadyExecuted = `already executed` | 0(pipeline では続行) |
| `Run` | ステートが空なら先に初期ステートを、耐久性を省いて書く(§5) | - | ジョブの結果による |

「等しい」はステート値の全項目の比較。`PipelineState` と `JobState` の全フィールドを、値で比べる関数 `stateEquals(a, b)` を `src/flow/` に置く(`===` は参照を比べるので使わない。`jobs` と `jobOrder` は要素ごとに比べる。省略可能な項目は、「無し」と `undefined` を等しいとみなす)。「耐久性あり(Durable)」「省く(Fast)」の定義は [06-state-file.md](06-state-file.md) の書き込み。

---

## 5. 実行の流れ

### 5.1 共通の前処理 `prepare`(`run`・`pipeline`)

順序は固定。最初に誤りがあった時点で、その終了コードで終わる(以降は行わない)。

| 順 | 処理 | 誤りのときの終了コード |
|---|---|---|
| 1 | 中断の合図(「立った/立っていない」と最初のシグナル番号を保持する共有の旗。§7.8 の `Interrupt`)を作り、シグナルの監視を開始(定義の検査より前)。すでに登録済みの場合は無視して続行(`Deps` のシグナルの登録は、2 回呼ばれても 2 重に登録しない) | 準備の失敗: 70(`conductor: error: failed to set up signal handling`) |
| 2 | 定義ファイルの読み込みと検証。`pipeline` は、引数の名前と定義の `pipeline.name` を照合 | 64 |
| 3 | 全ジョブの `command` の検査(引用符の閉じ忘れ・空) | 64 |
| 4 | `run` のとき、ジョブIDが定義にあるか | 68(`unknown job`) |
| 5 | run ID の決定(§6) | 68 |
| 6 | ステートの読み込み([06-state-file.md](06-state-file.md) §6)。完了のまま残ったものは「空」 | 65 |
| 7 | 空でなければ、定義との照合(06 §7) | 65 |
| 8 | 実効の `remove_on_complete` = `--remove-on-complete` があれば true、なければ定義の `pipeline.remove_on_complete`(省略時 false)。環境変数は見ない | - |

- 定義ファイルは `--config <パス>`、なければ実行ディレクトリの `conductor.yaml`(既定のファイル名と探し方は [04-cli.md](04-cli.md))。
- シグナルを受け取っても、1〜7 は最後まで進む。64・65・66・スキップ(0)の終了コードが、中断の終了コードに優先する。
- `pipeline` で引数の名前が定義の名前と違う場合は 64(順 2)。引数の名前が無いのは使い方の誤り 68(コマンドライン解析。[04-cli.md](04-cli.md))。
- すべての非同期処理(ファイル読み書き、子プロセス、待機)は `await` で直列に行い、1 つの `run`・`pipeline` の中で並行に走らせない。

### 5.2 `conductor run <job-id>` の擬似アルゴリズム

```text
session = await prepare(...)                 # 5.1。誤りなら終了
(states, resume, endReached) = session.state の jobs / resumeJobId / endReached
                               (state が空なら [], 無し, false)
d = decideBeforeRun(jobId, def, states, resume, endReached)

match d:
  reject(r):  表示、return 66
  skip(r):
    base = session.state か initialState(...)(空なら)
    rec  = applySkip(base, jobId, r, def)
    if not stateEquals(rec, base):
        await write(rec, Durable)            # 失敗: 65、文脈=ジョブ起動前
        if rec.completed and removeOnComplete:
            await resetState()               # 失敗: 65
    -v なら skipped 表示
    return 0
  run:
    base = session.state か、(空なら) initialState を Fast で書いたもの   # 書けなければ 65、ジョブは起動しない
    exec = await runJobWithRetry(job)        # §7。ステートは書かない
    (outcome, next) = classifyAndDecide(job, exec, base.jobs, def, "individual")
    表示(§5.5)
    rec = applyOutcome(base, jobId, exec, outcome, next)
    await write(rec, Durable)                # 失敗: 65、文脈=exec.started なら「ジョブは実行されました」、そうでなければ「起動していません」
    if next.completed and removeOnComplete:
        await resetState()                   # 失敗: 65
    return exitCodeFor(outcome.reportedExitCode, outcome.classification === "error")
```

注意:

- individual では、遷移でスキップが決まったジョブ(`byTransition`)は、`applyOutcome` で記録済みなので、後で `run` されると `Skip(SkipTarget)`。
- 最後のジョブの `run` 完了、または `@end` 後の最後のジョブのスキップ記録で `completed = true`。
- 初期ステートは `Run` と決まった後にだけ書く(Reject・Skip の前には書かない)。ただし Skip のときは、ステートが空なら `initialState`(06 §2.4)を土台に記録が書かれる。
- `classifyAndDecide(job, exec, states, def, mode)` は、`classify`(§2.3)で結果を作り、その分類を使って `nextStep`(§3.2)で次の動きを決めて、組で返す(同期の純粋な関数)。
- 終了コードは `process.exitCode` に設定して返し、`process.exit()` は標準出力の書き込みの完了を待ってから呼ぶ(または呼ばずにイベントループが空になるのを待つ)。詳細は [08](08-output-and-i18n.md)。

### 5.3 `conductor pipeline <name>` の擬似アルゴリズム

`pipeline` は、「次のジョブ」を `NextStep.nextJobId` で決めない。定義の記述順に全ジョブを 1 つずつ見て、`decideBeforeRun` で決める(遷移先・スキップは、ステートが運ぶ)。

```text
session = await prepare(...)                 # 引数の名前が無ければ 68
current = session.state                      # 空なら 無し
for job in def.jobs (記述順):
  d = decideBeforeRun(job.id, def, current の状態一覧 (無しなら []), current.resumeJobId, current.endReached)
  match d:
    reject(r): 表示、return 66
    skip(r):
      base = current か initialState(...)
      rec  = applySkip(base, job.id, r, def)
      if not stateEquals(rec, base): await write(rec, Durable)   # 失敗: 65
      -v なら skipped 表示
      current = rec
      if rec.completed: return await finish(...)   # 下記
    run:
      base = current、無ければ initialState を Fast で書いたもの   # 失敗: 65
      exec = await runJobWithRetry(job)
      (outcome, next) = classifyAndDecide(job, exec, base.jobs, def, "pipeline")
      表示
      rec = applyOutcome(base, job.id, exec, outcome, next)
      await write(rec, Durable)              # 失敗: 65
      if outcome は error、または分類なし:
          return exitCodeFor(outcome.reportedExitCode, outcome.classification === "error")   # エラーで停止
      if next.completed: return await finish(...)
      if 中断の合図が立っている(番号 n):      # ジョブのあいだの中断
          point = resumePointOnInterrupt(def, rec.jobs)
          await write(applyInterrupt(rec, point), Durable)     # 失敗: 65
          -v なら「interrupted by signal n」
          return signalExitCode(n)           # exitCodeFor(128 + n, true)
      current = rec
# ループを抜けた(完了していない。起こらない想定)
表示 "cannot determine whether the pipeline is complete"; return 65   # ステートは初期化しない

finish: if not removeOnComplete: return 0
        resetState() が成功 → 0、失敗 → 65
```

- 完了のステートは、`finish` の前に必ず Durable で書き込み済み。`remove_on_complete` が真のときだけ削除する(06 §9)。
- 「完了を判定できない」は、全ジョブが `AlreadyExecuted` などで通過し、どこでも `completed` が真にならなかったとき。標準エラー `conductor: error: <ステートファイルのパス>: cannot determine whether the pipeline is complete`(日本語: `パイプラインが完了したかを判定できません`)、終了コード 65。
- ジョブ実行後、`next.completed` の判定は、中断の合図の判定より**先**。完了していれば中断の記録はせず、`finish` へ進む。
- エラーのジョブで止まったときの再実行: 同じコマンドを再度呼ぶと、`success`/`warning` のジョブは `Skip(AlreadyExecuted)`(`-v` に `already executed`)、`error` のジョブから `Run`。
- `Skip` で `rec.completed` が真になるのは §3.4 の条件(スキップしたジョブが最後のジョブで、pending が無い)のとき。

#### 状態遷移の例(`pipeline`)

定義 `a → b → c`(遷移なし)。`b` が exit 3 でエラー。

| 時点 | jobs | resumeJobId | endReached | completed | 終了コード |
|---|---|---|---|---|---|
| 初期(Fast で書く) | a=pending, b=pending, c=pending | なし | false | false | - |
| a 成功後 | a=success(0), b=pending, c=pending | なし | false | false | - |
| b エラー後 | a=success(0), b=error(3), c=pending | b | false | false | 3 |
| 再実行: a は Skip(AlreadyExecuted)、b 成功 | a=success, b=success(0), c=pending | なし | false | false | - |
| c 成功後 | 全部 success | なし | false | true | 0 |

定義 `a --success--> @end` のとき(pipeline): a 成功で b, c は `skipped(byEnd)`、`endReached = true`、`completed = true`、0 で終わる([06-state-file.md](06-state-file.md) §4.4 の例 2 のステート)。

### 5.4 `@end` 後の `run`(individual)の流れ

`run a`(遷移 `@end`)→ `a=success`、`endReached = true`、`completed = false`(後ろに未完了がある場合)。続けて `run b` → 規則 2 で `Skip(AfterEnd)` → `b=skipped(byEnd)`、`resumeJobId` = 最初の pending(`c`)。`run c` → `Skip(AfterEnd)` → `c=skipped(byEnd)`、最後のジョブで pending が無いので `completed = true`(`remove_on_complete` なら削除)。`@end` の後ろに未完了が無ければ、`run a` の時点で `completed = true`。

### 5.5 表示(`-v` と常時)

- 常に標準エラー: エラー行(`conductor: error: ...`)、起動失敗、成功コードに無い 0、リトライの知らせ(§7.9)、`reset --all` の警告。
- `-v` のときだけ標準エラー(先頭 `[conductor] `): `job=<id> result=<success|warning|error|interrupted> (...)`、`transition: <success|warning> -> <target>[ (skipped: <id>, ...)]`、`job=<id> skipped (<理由>)`、`interrupted by signal <n>`。括弧内の詳細の厳密な文言は [08-output-and-i18n.md](08-output-and-i18n.md) に従う。
- ジョブの結果の行と遷移の行は、**最終の結果について 1 回**だけ(リトライの途中の結果は出さない)。
- 言語: 環境変数 `LC_ALL`、`LC_MESSAGES`、`LANG` の順で、最初の空でない値が `ja` で始まれば日本語、それ以外は英語(どれも空・未設定なら英語)。トークン(`success` `warning` `error` `skipped` `pending` `job=` `result=`)は常に英語。

---

## 6. run ID

run ID は、同じ定義を別々の実行単位で並行して使うときに、ステートファイルを分ける識別子。

### 6.1 値の規則(`parseRunId`)

次の順に検査し、最初に当てはまる誤りを返す(誤りは使い方の誤り 68。誤りの文に値は含めない)。

| 順 | 誤り | 条件 |
|---|---|---|
| 1 | `Empty` | 長さ 0 |
| 2 | `TooLong` | UTF-8 で符号化したバイト数が 64 を超える |
| 3 | `InvalidChar` | `A-Za-z0-9._-` 以外の文字がある |
| 4 | `LeadingDot` | 先頭が `.`(`..` を含む) |

TypeScript での注意:

- 順 2 の長さは、`value.length`(UTF-16 のコード単位の数)ではなく、`Buffer.byteLength(value, "utf8")` で数える。`"あ".repeat(30)` は `.length` が 30 だが UTF-8 では 90 バイトで、`TooLong` になる(確認済み: `Buffer.byteLength` が 90、`.length` が 30)。`.length` を使うと、順 2 と順 3 の判定が入れ替わる(`InvalidChar` になる)。許す文字は ASCII だけなので、検査を通る値では文字数とバイト数は同じ。
- 順 3 は、`/^[A-Za-z0-9._-]+$/` で判定してよい(`m` フラグを付けない。JavaScript の `$` は、`m` フラグなしでは文字列の最後にだけ一致し、末尾の改行の手前には一致しない)。空は順 1 で除かれている。
- 環境変数や引数に UTF-8 として不正なバイト列があると、Node.js は代替文字 U+FFFD に置換した文字列を渡す(確認済み。04 章 4.4 節。macOS・v26.13.0)。置換後の文字は ASCII でないので、いずれにせよ `InvalidChar`(または `TooLong`)になる。

有効: `1`, `r1`, `12345`, `build-2026.11_a`, `a.b`, 64 文字ちょうど。無効: `""`(ただし §6.2 の規則で空は「なし」)、65 文字、`a/b`、`a b`、`.x`、`..`、`日本語`。

### 6.2 決定(`resolveRunId`、`prepare` の 5)

```text
envName = 定義の pipeline.run_id_env があればそれ、なければ "CONDUCTOR_RUN_ID"
if --run-id が指定された(値は空でもよい):
    value = そのオプションの値(subject = "--run-id")      # 環境変数は見ない
else:
    value = env[envName](未定義なら 無し)(subject = "run ID (environment variable)")
if value が 無し または "": run ID なし
else: parseRunId(value) が失敗なら 68(`invalid value: '<subject>'`)、成功なら run ID
```

- `--run-id ""` は、環境変数があっても「run ID なし」に固定する。
- `run_id_env` の形は定義の検証で保証される(`A-Za-z_` で始まり `A-Za-z0-9_` が続く 1 文字以上)。
- 環境変数名は定義で決まるので、定義の検査(64)の**後**、ステートの読み込み(65)の**前**に run ID を決める。
- `watch`・`reset` は定義を検証しない: `--run-id` があれば定義を読まず(環境変数名は既定)、なければ定義の `run_id_env` だけをゆるく読む。読めない・キーが無い・文字列でない・名前の形が違う、のどれでも「なし」= 既定の名前を使う。ほかの誤りは無視する。異常終了してはならない(定義の読み込みが投げた例外は、この場面では握りつぶして既定の名前にする)。
- 環境変数の値が規則に合わなければ、`run`・`pipeline`・`watch`・`reset`(`--all` を除く)で 68。`reset --all` は環境変数を見ない。
- 環境変数は、`Deps` の環境(`process.env` のコピーまたは読み取り関数)から読む。Windows では `process.env` の名前の照合が大文字小文字を区別しない(Node.js の文書にある挙動。この仕様の確認はしていない)ので、`run_id_env` の大文字小文字の違いが Windows では同じ変数を指す。UNIX 系では区別する。
- 環境変数が「空文字列で定義されている」場合は、「なし」として扱う(上の疑似コードの `""`)。Windows で空の値の環境変数が `process.env` に現れるか、`undefined` になるかは未確認だが、どちらでも結果は「なし」で同じ。

### 6.3 run ID とステートの関係

- run ID あり: ステートは `<定義ファイル名>.<run-id>.state`。中身に `run_id` = run ID を書く。run ID なし: `<定義ファイル名>.state`、`run_id` の項目無し。
- 読み込み時、中身の `run_id`(あるか、値)がファイル名側の run ID と一致しなければ壊れたステート(65)。詳細は 06 §6。
- run ID は同じ定義でも run ごとに独立したステート・再開位置を持つ。同じ run ID で呼び直すと、止まったジョブから再開する。

---

## 7. リトライ(`retry`、v0.8.0)

### 7.1 設定(検証済みの値)

| キー | 型・範囲 | 意味 |
|---|---|---|
| `exit_codes` | 整数の一覧。空でない。各値は 1〜255。昇順・重複なしに正規化済み | リトライの対象にするジョブ自身の終了コード。このジョブの成功コード・警告コードと重ならない |
| `max` | 整数 1〜100 | リトライの最大回数(最初の実行を除く)。実行は合計で最大 `max + 1` 回 |
| `interval` | 文字列 `<整数><s|m|h>`。結果が 1 秒〜24 時間(86400 秒) | 1 回目のリトライの前の待ち時間。数字は半角の数字だけで、整数は 1 以上。単位は小文字 1 文字(`s` 秒、`m` 分 = 60 秒、`h` 時間 = 3600 秒)。例 `"30s"`、`"2m"`、`"1h"` |
| `multiplier` | 整数 2〜10 | 待ち時間にかける倍率。`max_interval` と同時に指定 |
| `max_interval` | `interval` と同じ書式。`interval` 以上 | 待ち時間の上限。`multiplier` と同時に指定 |

- `multiplier` と `max_interval` は両方書くか両方書かない。検証の詳細は [03-definition-format.md](03-definition-format.md)。
- `retry` を書かないジョブは、1 回だけ実行する。

### 7.2 数値の型

§0.2 の方針を、リトライに当てはめる。

| 値 | 型 | 備考 |
|---|---|---|
| 待ち時間(秒。`intervalSecs`、`maxIntervalSecs`、各回の待ち、`lastWaitSecs`) | `bigint`(0〜2^64-1) | 計算は秒の `bigint` で完結し、`number` への変換は待機の直前の 1 か所だけ(§7.3、§7.8) |
| `multiplier` | `number`(2〜10) | `satMulU64` の中で `BigInt(multiplier)` に変換する |
| `max`、`retriesDone`、知らせの回数 `n` | `number` | `max` は 1〜100 |
| `exitCodes` の各値 | `number`(1〜255) | ジョブの終了コードの型と同じ |

浮動小数点は使わない。丸めは無い(整数の秒だけ)。

`RetryPolicy` の骨格(検証済みの定義から作る。`src/flow/` が受け取る形):

```ts
export interface Backoff {
  readonly multiplier: number;          // 2〜10
  readonly maxIntervalSecs: bigint;     // intervalSecs 以上、86400 以下
}

export interface RetryPolicy {
  readonly exitCodes: readonly number[]; // 昇順・重複なし・各 1〜255・空でない
  readonly max: number;                  // 1〜100
  readonly intervalSecs: bigint;         // 1〜86400
  readonly backoff?: Backoff;            // 無しなら固定間隔
}
```

`Backoff` を 1 つのオブジェクトにまとめるのは、「`multiplier` と `max_interval` は両方あるか両方ない」を型で保証するための決定。定義の検証で片方だけは拒否される([03](03-definition-format.md))ので、部分的に存在する状態は作れない。

### 7.3 `interval` / `max_interval` の文字列から秒への変換と、ミリ秒・`number` への変換

文字列 → 秒(`parseDurationSecs(s: string): bigint | undefined`。定義の検証で使う。置き場所は `src/definition/`。ここでは数値の扱いを示すために再掲する):

```text
1. s の長さ(UTF-16 のコード単位)が 2 未満なら 無し(拒否)
2. 最後の 1 文字が 's' / 'm' / 'h' のどれでもなければ 拒否
3. 残り(digits)が、/^[0-9]+$/ に一致しなければ 拒否
   (符号・空白・小数点・'_'・全角数字・"0x" は、ここで弾かれる。`Number()`・`parseInt`・`BigInt()` の
    解釈には頼らず、先に正規表現で検査する)
4. digits の先頭の '0' を取り除く(すべて '0' なら "0" を残す)。取り除いたあと:
     - 長さが 20 を超えるなら 拒否(2^64-1 は 20 桁。86400 秒を超えるので、どのみち拒否される)
     - n = BigInt(digits)
5. 単位ごとの上限で、乗算の前に比べる:
     's': 1n <= n <= 86400n
     'm': 1n <= n <= 1440n
     'h': 1n <= n <= 24n
   外れたら 拒否
6. secs = n * (1n / 60n / 3600n)
```

`bigint` の乗算は桁あふれしないので、手順 5 の事前の比較は桁あふれ防止のためではなく、「結果が 86400 秒以下」という検証規則をそのまま実装するためのもの。手順 4 の桁数の制限は、極端に長い数字列を `BigInt` に渡さないためのもの。

例: `"1s"`→1n、`"01s"`→1n、`"0000000000000000000000000001s"`→1n、`"1440m"`→86400n、`"24h"`→86400n、`"86400s"`→86400n。拒否: `""`、`"s"`、`"10"`、`"1.5s"`、`"-1s"`、`"+1s"`、`" 1s"`、`"1s "`、`"1S"`、`"１s"`(全角 1)、`"1ms"`、`"0x10s"`、`"1_0s"`、`"0s"`、`"0h"`、`"86401s"`、`"25h"`、`"99999999999999999999999s"`、`"18446744073709551615h"`。

秒 → ミリ秒 → `number`(待機の直前だけ。§7.8、07 章 7.5 節):

```text
ms = secs * 1000n                       # bigint。桁あふれしない
# setTimeout に渡す 1 回分は 1〜2^30 ミリ秒(07 章 7.5 節。上限 2^31-1 より余裕を持たせた値)
chunkMs = min(remainingMs, 1073741824n)
Number(chunkMs)                          # 2^30 以下なので正確(2^53 未満)
```

- `setTimeout` の遅延は 32 ビット符号付き整数に収まる必要がある。**確認済み(Node.js v26.13.0)**: `setTimeout(fn, 2 ** 31)` は、`TimeoutOverflowWarning`(`2147483648 does not fit into a 32-bit signed integer. Timeout duration was set to 1.`)を出して**遅延 1 ミリ秒に置き換えられ、ほぼ即座に発火する**(実測 0 ミリ秒)。`timers/promises` の `setTimeout` も同じ。つまり、上限を超えた値を渡すと、長い待機が一瞬で終わってしまう。そのため、**1 回の `setTimeout` には 2^30(1073741824)ミリ秒以下の整数だけを渡し(07 章 7.5 節。32 ビット符号付きの上限 2147483647 を超えないための余裕を持たせた値)、それより長い待機は分割して繰り返す**(§7.8 の手順)。
- 検証済みの定義では、待ち時間は最大 86400 秒 = 86,400,000 ミリ秒で、上限 2,147,483,647 ミリ秒(約 24.86 日)の約 2.5% であり、分割は起きない。分割の分岐は、定義の検証を通らない入力(単体テスト)でだけ通る。
- 秒の値が 2^64-1 のとき、ミリ秒は約 1.8 × 10^22 で、分割すると約 8.6 × 10^12 回の繰り返しになり、実質的に終わらない。この入力は検証済みの定義では起きない。再現は不要で、「桁あふれして短くならない」ことだけを守る(`bigint` なので守られる)。
- `number` の `Number.MAX_SAFE_INTEGER`(2^53-1)は 9007199254740991。待ち時間の秒を `number` に落とすと、2^53 を超える値で丸められる。検証済みの定義(86400 以下)では問題ないが、この仕様では秒を `number` で持たない。

### 7.4 リトライの対象か(`isRetryTarget`)

次の**すべて**が成り立つときだけ真。

1. `started = true`
2. `interrupted = false`
3. `startFailure = false`
4. `terminatedBySignal` が無い
5. `exitCode` があり、`exitCodes` に含まれる

分類(成功・警告・エラー)は見ない。対象の終了コードは成功・警告と重ならない(検証済み)ので、結果として対象はエラーの終了コードだけ。ジョブ自身が 127 で終わった場合は、`exit_codes` に 127 を書いていれば対象(conductor が付けた起動失敗の 127 は、`exitCode` を持たないので対象にならない)。

### 7.5 判断(`retryDecision(job, execution, retriesDone, lastWaitSecs)`)

```text
policy = job.retry か、無ければ return stop
if not isRetryTarget(policy, execution) or retriesDone >= policy.max: return stop
return retry { waitSecs: nextWaitSecs(policy, retriesDone, lastWaitSecs) }
```

```ts
export type RetryDecision =
  | { readonly kind: "stop" }
  | { readonly kind: "retry"; readonly waitSecs: bigint };
```

### 7.6 待ち時間の式

```text
nextWaitSecs(policy, retriesDone: number, lastWaitSecs: bigint): bigint
  if policy.backoff が有る:
      if retriesDone == 0: return min(policy.intervalSecs, policy.backoff.maxIntervalSecs)   # = intervalSecs(検証で interval <= max_interval)
      else:                return min(satMulU64(lastWaitSecs, policy.backoff.multiplier), policy.backoff.maxIntervalSecs)
  else:
      return policy.intervalSecs                                                              # 毎回同じ(固定間隔)
```

- k 回目のリトライ(k = `retriesDone + 1`)の前の待ち時間を `W(k)` とすると: 固定間隔なら `W(k) = interval`。倍率ありなら `W(1) = interval`、`W(k) = min(W(k-1) × multiplier, max_interval)`(k ≥ 2)。
- 倍率の掛け算は、直前の**実際の**待ち時間(上限で頭打ちされた後の値)に対して行う。
- `min` は 2 つの `bigint` の小さい方。`Math.min` は `bigint` を受け付けない(`TypeError`)ので使わない。`a < b ? a : b` と書く。

`satMulU64(a: bigint, b: number): bigint`(64 ビット符号なし整数の飽和乗算):

```text
U64_MAX = (1n << 64n) - 1n               # 18446744073709551615n
p = a * BigInt(b)                         # bigint は桁あふれしない
return p > U64_MAX ? U64_MAX : p
```

`a × b` が 2^64 - 1(18446744073709551615)を超えるなら 18446744073709551615、そうでなければ `a × b`。**`BigInt.asUintN(64, a * BigInt(b))` で 64 ビットに切り詰めてはならない**(折り返しになる。確認済み: `BigInt.asUintN(64, (2n**64n - 1n) * 10n)` は `18446744073709551606n`)。前提として `a` は 0 以上 `U64_MAX` 以下(呼び出し側の保証。定義の検証と、`nextWaitSecs` の出力が `maxIntervalSecs` 以下であることから成り立つ)。

境界の例(関数単体の挙動):

| a | b | 結果 | 理由 |
|---|---|---|---|
| 0n | 10 | 0n | |
| 86400n | 10 | 864000n | 検証済みの定義で起こりうる最大の積 |
| 1844674407370955161n | 10 | 18446744073709551610n | 飽和しない最大に近い値(`2^64-1` を 10 で割って切り捨てた値 × 10。確認済み) |
| 1844674407370955162n | 10 | 18446744073709551615n | 積 `18446744073709551620n` が `2^64-1` を超えるので飽和 |
| 18446744073709551615n | 2 | 18446744073709551615n | 飽和 |
| 9223372036854775807n | 10 | 18446744073709551615n | 飽和 |

`nextWaitSecs` の境界例(関数単体。検証済みの定義では起きない入力):

| intervalSecs | multiplier | maxIntervalSecs | retriesDone | lastWaitSecs | 結果 |
|---|---|---|---|---|---|
| 10n | 2 | 60n | 0 | 0n | 10n |
| 10n | 2 | 60n | 3 | 40n | 60n(80 を 60 で頭打ち) |
| 10n | 2 | 60n | 4 | 60n | 60n |
| 1n | 10 | 18446744073709551615n | 1 | 9223372036854775807n | 18446744073709551615n(積が飽和し、上限で止まる) |
| 30n | (backoff なし) | - | 5 | 30n | 30n |

検証済みの定義では `interval` と `max_interval` が 86400 秒以下なので、`satMulU64` が飽和する場面は無い(最大でも 86400 × 10)。再実装でこの極端な入力まで再現する必要は実用上ないが、乗算は桁あふれで折り返さないこと。

待ち時間の列の例:

| interval | multiplier | max_interval | max | 待ち時間の列(秒) | 合計 |
|---|---|---|---|---|---|
| 30s | なし | なし | 3 | 30, 30, 30 | 90 |
| 10s | 2 | 60s | 5 | 10, 20, 40, 60, 60 | 190 |
| 60s | 3 | 60s | 3 | 60, 60, 60 | 180 |
| 1s | 10 | 3600s(1h) | 6 | 1, 10, 100, 1000, 3600, 3600 | 8311 |
| 1m | 2 | 24h | 4 | 60, 120, 240, 480 | 900 |
| 1h | 10 | 24h | 4 | 3600, 36000, 86400, 86400 | 212400 |

### 7.7 実行ループ(`runJobWithRetry`)。`run` と `pipeline` の共通

```text
retriesDone = 0; lastWaitSecs = 0n
loop:
    exec = await runJob(job, interrupt)
    match retryDecision(job, exec, retriesDone, lastWaitSecs):
        stop: return exec                          # 最後の結果
        retry{waitSecs}:
            表示: 常に 1 行(§7.9)。retryNumber = retriesDone + 1
            match await waitRetry(waitSecs, interrupt):   # Deps.proc.wait
                elapsed:        retriesDone += 1; lastWaitSecs = waitSecs; continue
                interrupted(n): return JobExecution{ jobId=job.id, command=job.command, started=false, interrupted=true, signalNumber=n,
                                                      exitCode=null, startFailure=false, terminatedBySignal=null }
```

- 返された最後の結果だけが、分類・表示・ステート記録の対象(ステートは最終の結果で 1 回だけ書かれ、途中では書かれない)。リトライの回数・直前の待ち時間はローカルな値で、ステートファイルに書かない。
- したがって、conductor が落ちたあとの再開では、そのジョブを最初の実行から数え直す。
- 使い切った(`retriesDone == max` で、まだ対象の終了コード): `stop` → 通常のエラーと同じ(最後の実行の終了コードで終わり、再開位置はそのジョブ)。再実行の結果が成功・警告なら通常どおり遷移へ進む。
- 総実行回数: 最大 `max + 1`。知らせの回数: 実際に実行し直す(または待機に入る)回数。
- 起動前に中断された(合図が立っていた)ジョブはリトライせず、その場で `started = false` の結果を返す(`retryDecision` は `started = false` で stop)。

### 7.8 待機とシグナルによる中断(`waitRetry`)

**決定: 中断の合図は 07 章の `Interrupt`(`raise`・`number`・`whenRaised`)で表し、待機は 07 章 7.5 節の `waitRetry(secs, sig)` で行う。** `AbortSignal` は使わない(07 章 4.2 節。`spawn` の `signal` は子を強制終了するため)。待機の手順(`setTimeout` と `Promise.race`、単調な時計での経過の再確認、1 回のタイマーを 2^30 ミリ秒以下に分けること、`finally` での `clearTimeout`)は、07 章 7.5 節が正本である。この節は、流れの側の規定だけを定める。

```ts
// 07 章の型(再掲。正本は 07 章)
export type WaitOutcome =
  | { readonly interrupted: false }                          // 経過
  | { readonly interrupted: true; readonly number: number }; // 中断(最初のシグナルの番号)
export function waitRetry(secs: bigint, sig: Interrupt): Promise<WaitOutcome>; // 拒否されない
```

- 7.7 の実行ループは、`Deps.proc.wait(secs, sig)`(02 章 4 節。本番は `waitRetry`)を呼ぶ。「経過」(`interrupted: false`)なら続け、「中断」(`interrupted: true`)なら、起動前の中断と同じ形の結果(`started = false`、`interrupted = true`、`signalNumber = number`)を返す。
- 守るべきことは 2 つ: (a) 合図から 1 秒以内に打ち切る(ポーリングをしない。07 章 7.5 節)。(b) 待ち時間が `secs` 秒以上になるまで「経過」を返さない(経過は単調な時計 `process.hrtime.bigint()` で再確認する。`Date.now()` は使わない)。
- 待機中に中断された場合は、次の実行をせず、分類なし(起動しなかった中断)として扱う: 終了コード `128 + 番号`、ジョブの状態は変えず(エラーとして記録しない)、再開位置はそのジョブ。`pipeline` でも、そこで止まって終わる(§5.3: 分類なしは停止)。
- 再開すると、そのジョブを最初の実行から数え直す。
- 待機の時計は、テストで差し替えない。`cli` のテストは、`Deps.proc.wait` の偽の実装で、待ち時間(秒の `bigint`)を記録だけして実際には待たない(02 章 4.4 節)。

### 7.9 リトライの知らせ

待機の**前**に、`-v` の有無によらず、標準エラーに 1 行(エラーではない。`conductor: retry: ` で始まる)。待機が中断された場合も、その回の知らせは出ている。

- 英語: `conductor: retry: job '<id>' exited with code <last_exit_code>; retry <n> of <max> in <wait_secs> seconds`
- 日本語: `conductor: retry: ジョブ '<id>' が終了コード <last_exit_code> で終わりました。<wait_secs> 秒後にリトライします(<n>/<max> 回目)`
- `<last_exit_code>` は直前の実行の終了コード(`exitCode` が無ければ 0)。`<n>` は 1 から。`<wait_secs>` は秒の整数(10 進、桁区切りなし。`waitSecs.toString()`。`toLocaleString` は使わない)で、1 のときも `seconds`(単数形の処理はない)。`<id>` は表示用にエスケープ・長さ制限される(規則は [08-output-and-i18n.md](08-output-and-i18n.md))。
- 例: `conductor: retry: job 'extract' exited with code 75; retry 2 of 5 in 20 seconds`

---

## 8. シグナルと中断(流れへの影響)

| 受信のタイミング | ジョブの結果 | ステートの記録 | 終了コード |
|---|---|---|---|
| ジョブの起動の前(`run`) | 起動しない。分類なし | ジョブの状態は変えない。`resumeJobId` = そのジョブ(初期ステートは、空だった場合に書かれている) | 128 + n |
| ジョブの実行中 | 子に同じシグナルを 1 回転送。終了を待つ。`interrupted`。分類 error | そのジョブを `error` で記録(`exitCode` はジョブの値があればそれ) | 128 + n |
| リトライの待機中 | 分類なし(§7.8) | 状態は変えない。再開位置 = そのジョブ | 128 + n |
| `pipeline` で、ジョブを実行・記録した直後(まだ完了していない) | - | 再開位置(§9)を記録 | `signalExitCode(n)` = 128 + n |
| 定義の検査・ステートの確認の途中 | 検査は最後まで進む | 64・65・66・スキップ(0)が優先 | それぞれ |

- シグナルは SIGHUP・SIGINT・SIGQUIT・SIGTERM(UNIX 系)、Ctrl+C(Windows)。実際に待ち受ける集合と `process.on` による登録は [07-process-and-signals.md](07-process-and-signals.md) に従う。2 回目以降のシグナルは追加の動作をしない(`Interrupt.raise` が番号を上書きしない)。終了コードに使う番号は、最初に受け取ったもの。
- 実行中の中断ではジョブ自身の終了コードは見ない(分類 error、返す値は 128 + n)。

---

## 9. 再開位置(`resumePointOnInterrupt`)

入力: 定義、各ジョブの状態。

```text
resumeJobId = 記述順で最初の「pending(または記録なし)または error」のジョブ(無ければ 無し)
completed   = (resumeJobId == 無し)
endReached  = 「skipped(byEnd)」のジョブが 1 つでもあれば true
```

```ts
export interface ResumePoint {
  readonly resumeJobId?: string;
  readonly completed: boolean;
  readonly endReached: boolean;
}
```

エラー・pending を区別せず、記述順で最初のものが再開位置。エラーのジョブが前にあれば、そのジョブ。この関数は記録を作らない(呼び出し側が `applyInterrupt`(§3.5)で書く)。

---

## 10. 完了と完了後

- 完了(`completed = true`)になるのは: 最後のジョブが success / warning で終わった、`@end` に到達した(pipeline、または individual で後ろに未完了が無い)、スキップの記録で最後のジョブまで済んだ(§3.4)。
- 完了のステートは `completed = true` のまま**残る**(`remove_on_complete` が真でない限り)。書式は変わらない。
- 完了のステートは、読み込み(`run`・`pipeline` の `prepare`)では「空」として扱う(定義との照合もしない)。次の `pipeline`・`run <最初のジョブ>` は先頭から始まり、初期ステートで上書きする。先頭以外のジョブを `run` で単独に呼ぶと順序外(66)。
- `remove_on_complete` が真のとき、完了のステートを書いた後に `resetState` で削除する(失敗は 65。完了のステートは残る)。偽のときは削除を試みず、0 で終わる(`run` ではジョブの結果による終了コード)。
- `watch` は完了のステートも表示する(表の下に `completed`/`完了` の行)。

---

## 11. 保証の範囲(at-least-once)

- 結果の記録は、ジョブの終了の**後**。終了と書き込みのあいだに conductor が落ちると、ジョブは未実行として扱われ、再実行されうる(ジョブが 2 回動きうる)。
- 結果の書き込みに失敗したときは、ジョブの終了コードではなく 65 で終わり、「ジョブは実行されました」と表示する。
- `retry` のジョブは、最大 `max + 1` 回動く。
- 時間の上限(タイムアウト)はない。ジョブが終わらなければ待ち続ける。
- 排他制御はしない([06-state-file.md](06-state-file.md) §11)。

---

## 12. 実装の特記事項(再実装で再現すること)

| # | 項目 | 挙動 |
|---|---|---|
| 1 | リトライの知らせの単数形 | 待ちが 1 秒のとき `in 1 seconds`(英語)。単数形の処理はない |
| 2 | エラーのジョブを再実行して、起動前・待機中に中断されたとき | 分類なしなので `status` は error のまま変わらないが、`exitCode` は消える(`applyOutcome` が `exitCode` に実行結果の値 = 無しを書く) |
| 3 | 成功直後の `resumeJobId` | 成功・警告のあとの `NextStep.resumeJobId` は無し。ただし後続のスキップ記録(`applySkip`)は、エラーが無いとき `resumeJobId` を最初の pending にする。そのため、遷移で飛ばした後のステートには、成功直後でも `resumeJobId` が入りうる |
| 4 | 分類なし(起動前の中断)の終了コードの整形 | `isError = false` で整形する。`128 + n` が 256 以上かつ下位 8 ビットが 0 になる番号(n = 128 など)は、0 になりうる(実在のシグナル番号では起きない) |
| 5 | `pipeline` の「完了を判定できない」 | 全ジョブを通過しても `completed` が真にならなければ 65(ステートは初期化しない) |
| 6 | ステートの誤りの原因の文 | 種類の文はロケールに従うが、理由(例: `JSON として読めません`)は常に日本語の固定文(06 §6.4) |
| 7 | run ID の長さ | 文字数ではなく UTF-8 のバイト数で数える(§6.1) |
| 8 | 待機の分割 | 1 回の `setTimeout` は 2^30 ミリ秒以下に分割する(§7.3、§7.8、07 章 7.5 節)。`setTimeout` の上限 2147483647 ミリ秒を超える値を渡すと、遅延 1 ミリ秒に置き換わる |

## 13. `src/flow/` の純粋関数の署名の骨格

実装は書かない。引数と戻り値の型だけを固定する(名前と型は、ほかの文書・テストの共通の語彙にする)。`FlowDefinition`・`FlowJob` は、`src/definition/` の型のうち判断に必要な部分だけを、`flow` が必要とする形で宣言したもの(`definition` の型が構造として満たせばよい。`flow` が `definition` を import してもよい)。

```ts
// src/flow/types.ts(続き。1.1〜1.3 の型を含む)
export type TransitionTarget =
  | { readonly kind: "end" }                       // "@end"
  | { readonly kind: "job"; readonly id: string };

export interface FlowJob {
  readonly id: string;
  readonly successExitCodes: readonly number[];    // 省略時は [0] を入れておく
  readonly warningExitCodes: readonly number[];
  readonly onSuccess?: TransitionTarget;           // on = success の遷移
  readonly onWarning?: TransitionTarget;           // on = warning の遷移
  readonly retry?: RetryPolicy;
}

export interface FlowDefinition {
  readonly name: string;                           // pipeline.name
  readonly jobs: readonly FlowJob[];               // 定義の記述順
}

// 検証済みの定義(03 章 14.1 節の Definition)から、判断に必要な部分だけを取り出す(純粋)。
// 遷移の `next: 'job' | 'end'` を TransitionTarget に、`successExitCodes` の省略を [0] に、
// `retry` と `backoff` の undefined を「プロパティなし」に直す(exactOptionalPropertyTypes のため)。
export function toFlowDefinition(def: Definition): FlowDefinition;

export type RunMode = "individual" | "pipeline";

export interface NextStep {
  readonly nextJobId?: string;
  readonly skippedJobIds: readonly string[];
  readonly skipReasonKind?: SkipReason;
  readonly endReached: boolean;
  readonly completed: boolean;
  readonly resumeJobId?: string;
}

// src/flow/classify.ts
export function classify(job: FlowJob, exec: JobExecution): JobOutcome;
export function exitCodeFor(reported: number, isError: boolean): number;
export function signalExitCode(signalNumber: number): number;
export function satAddI32(a: number, b: number): number;

// src/flow/next-step.ts
export function nextStep(
  job: FlowJob, outcome: JobOutcome, states: readonly JobState[],
  def: FlowDefinition, mode: RunMode,
): NextStep;
export function classifyAndDecide(
  job: FlowJob, exec: JobExecution, states: readonly JobState[],
  def: FlowDefinition, mode: RunMode,
): { readonly outcome: JobOutcome; readonly next: NextStep };

// src/flow/decide.ts
export function decideBeforeRun(
  jobId: string, def: FlowDefinition, states: readonly JobState[],
  resumeJobId: string | undefined, endReached: boolean,
): Decision;

// src/flow/resume.ts
export function resumePointOnInterrupt(def: FlowDefinition, states: readonly JobState[]): ResumePoint;

// src/flow/retry.ts
export const U64_MAX: bigint;                       // (1n << 64n) - 1n
export function satMulU64(a: bigint, b: number): bigint;
export function isRetryTarget(policy: RetryPolicy, exec: JobExecution): boolean;
export function nextWaitSecs(policy: RetryPolicy, retriesDone: number, lastWaitSecs: bigint): bigint;
export function retryDecision(
  job: FlowJob, exec: JobExecution, retriesDone: number, lastWaitSecs: bigint,
): RetryDecision;
```

`src/state/` の記録関数(`applyOutcome` など)の署名:

```ts
export function initialState(
  definitionPath: string, def: FlowDefinition, runId: string | undefined,
): PipelineState;
export function applyOutcome(
  s: PipelineState, jobId: string, exec: JobExecution, outcome: JobOutcome, next: NextStep,
): PipelineState;
export function applySkip(
  s: PipelineState, jobId: string, reason: DecisionReason, def: FlowDefinition,
): PipelineState;
export function applyInterrupt(s: PipelineState, point: ResumePoint): PipelineState;
export function stateEquals(a: PipelineState, b: PipelineState): boolean;
```

単体テストで固定する入力(最低限): §2.2 の表、§2.3 の全組み合わせの表、§4.2 の例の表、§7.6 の `satMulU64`・`nextWaitSecs` の表と待ち時間の列、§6.1 の有効・無効の例(特に 30 文字の日本語が `TooLong` になること)、§7.3 の `parseDurationSecs` の受理・拒否の例。

## 14. 実装に任せてよい点と未確認

実装に任せてよい点(外から見える挙動が同じなら自由):

- 判断関数・記録関数の内部構造、型の表現(文字列の共用体か `enum` か。この文書は文字列の共用体を前提にしたシグネチャを示した)。
- `Interrupt` は 07 章の型(`raise`・`number`・`whenRaised`)に従う。`Deps` の `Clock` は時刻(`now()`)だけで、待機は `Deps.proc.wait`(02 章)が担う。
- 標準エラーの出力のバッファリング(ただし行単位で、順序を保つ)。

この文書の確認の範囲:

- **確認済み(Node.js v26.13.0、macOS)**: `setTimeout`(`node:timers` と `node:timers/promises`。この仕様は後者を使わない)に 2^31 を渡すと `TimeoutOverflowWarning` が出て遅延が 1 ミリ秒になる。`timers/promises` の `setTimeout` は、`signal` の `abort()` で `AbortError`(`code` が `ABORT_ERR`)で拒否され、`abort` 済みの `signal` では待たずに拒否される。`BigInt` の乗算・比較、`BigInt.asUintN` が折り返す挙動。`Number(2n ** 63n)` が丸められること。`Buffer.byteLength` と `.length` の違い。`(-1) & 0xFF` などの 32 ビット符号付きの `&`。

未確認:

- Node.js の最新の LTS 系列と 1 つ前の LTS 系列(実装時に確認)での、上記の挙動の再確認。この確認は v26.13.0 だけで行った。上記は、サポートする版の CI の単体テストで確認すること。
- Windows での、実行中ジョブへのシグナル転送の細部(07)と、`exitCode` が負になる場合の実機の値。`process.env` の大文字小文字の扱いと、空の値の環境変数の見え方。
- タイマーが目標の時刻よりどれだけ早く発火しうるか(§7.8 は、毎回経過時間を再確認することで、この量に依存しないようにしている)。
- ジョブの標準入出力の扱い(07 が `stdio: "inherit"` と定める)の、この文書の流れへの影響は確認していない。
- 複数シグナルがほぼ同時に届いたときの「最初の番号」は、Node.js がシグナルのリスナーを呼ぶ順序に依存する。再実装では「最初に観測した 1 つ」を保持すれば足りる(`Interrupt.raise` の規則)。ほぼ同時の到着の再現は未確認。
