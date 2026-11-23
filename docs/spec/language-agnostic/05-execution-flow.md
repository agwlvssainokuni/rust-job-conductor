# 05. 実行の意味論(終了コード・分岐・再開・run ID・リトライ・環境変数・パイプライン・図と計画)

対象: `conductor` v0.9.6 までと同じ挙動の実行ツール。この文書は、`run`(ジョブを 1 つ実行)と `pipeline`(定義の全ジョブを順に実行)の実行の流れを、判定表・状態遷移・言語中立の擬似コードで定める。この文書だけで、任意の言語・ライブラリで同じ挙動を実装できることを目標にする。

ステートファイルの論理項目・保存の性質・読み込みの検査は [06-state-file.md](06-state-file.md) にある。この文書の「ステートへの記録」は、06 の形式のファイルを書く前の、メモリ上のステート(以下「ステート値」)の更新を指す。定義ファイルの書式は [03-definition-format.md](03-definition-format.md)、コマンドラインは [04-cli.md](04-cli.md)、プロセス起動とシグナルは [07-process-and-signals.md](07-process-and-signals.md)、終了コードの総覧は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md) を参照。

実装の構成(推奨): 「判断」(分類・次の動き・起動前の判断・リトライの判断・再開位置)は、ファイルにもプロセスにも触れない**純粋な関数**にし、失敗させず必ず結果を返す。「記録」はステート値の更新(複製して更新。元は変えない)。「流れ」が、ファイルとプロセスを扱う。

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

### 1.2 1 回の実行結果(`JobExecution`)

ジョブを 1 回起動して終わるまでの観測結果。

| フィールド | 型 | 意味 |
|---|---|---|
| `started` | 真偽 | 起動した(子プロセスの生成に成功した)か。「中断の合図が起動の前に立っていたため起動しなかった」ときだけ `false`。ほかの起動失敗でも `started` は `true` のまま(`start_failure` を見る) |
| `exit_code` | 32 ビット符号付き整数、または無し | プロセスの終了コード。取れないとき(シグナル終了、起動失敗)は無し |
| `terminated_by_signal` | 整数、または無し | ジョブがシグナルで終了したときの番号(UNIX 系のみ) |
| `start_failure` | 真偽 | 起動に失敗した。プログラム名が空、プロセス生成の失敗、終了状態を取得できない、のいずれか(分けられない command は実行の前の検査で 64 になり、起動に届かない) |
| `interrupted` | 真偽 | conductor がシグナルを受け取り、中断の合図が立っている(ジョブの終了を観測した直後に立っていた場合を含む) |
| `signal_number` | 整数、または無し | `interrupted` のとき、最初に受け取ったシグナルの番号 |

ジョブの起動手順(`run_job`):

1. 実行の前の検査で引数に分けたコマンド(規則は [07-process-and-signals.md](07-process-and-signals.md))を受け取る。ここでは分け直さない。プログラム名が空(空白だけを含む)なら `start_failure = true`(`started = true`)で終わる。v0.9.5 までは、ここで command を引数に分け直し、失敗なら `start_failure = true` で終わった。
2. 中断の合図が立っていれば、`started = false, interrupted = true, signal_number = 番号` で終わる(起動しない)。
3. 子プロセスを生成する。失敗なら `start_failure = true` で終わる。
4. 終了まで待つ。待機中に合図が立てば、子プロセスに同じシグナルを**1 回だけ**転送して、終了を待ち続ける。
5. 終了コードとシグナル終了を記録する。終了状態を取得できなければ `start_failure = true`。
6. 終了を観測した直後に合図が立っていれば `interrupted = true, signal_number = 番号`。

### 1.3 結果(`JobOutcome`)

| フィールド | 意味 |
|---|---|
| `classification` | `success` / `warning` / `error`、または**分類なし**(「起動しなかった中断」) |
| `reported_exit_code` | スケジューラへ返す終了コード(32 ビット符号付き整数)。後で 0〜255 に整える(§2.2) |
| `conductor_error` | 「成功コードに含まれない 0」など、conductor 自身が原因を標準エラーに表示するエラーのとき `true` |

### 1.4 実行方式(`RunMode`)

`Individual`(`conductor run <job-id>`)と `Pipeline`(`conductor pipeline <name>`)。`@end` の扱いと完了の判定だけが違う(§3、§5)。

---

## 2. 終了コード

### 2.1 conductor が返す終了コード(プロセスの終了コード)

| 値 | 名前 | 意味 |
|---|---|---|
| 0 | OK | 成功、警告、スキップ、正常終了 |
| ジョブの値(1〜255) | - | エラーになったジョブの終了コードをそのまま(範囲外は §2.2) |
| 1 | - | 成功コードに含まれない 0 で終わったジョブ(`conductor_error`)。終了コードを取れなかったエラーも 1 |
| 64 | DEFINITION | 定義の誤り(ファイルがない・読めない・検証の誤り・パイプライン名の不一致・command の誤り) |
| 65 | STATE | ステートの誤り(読めない・壊れている・版の不一致・定義との食い違い・書き込み/初期化/削除の失敗)、完了を判定できない、`watch` の出力失敗 |
| 66 | REJECTED | 拒否(順序外) |
| 67 | CONVERSION | convert の失敗(この文書の対象外) |
| 68 | USAGE | 使い方の誤り(知らないジョブID、run ID の値の誤りを含む) |
| 70 | INTERNAL | 内部の誤り(想定外の異常終了、シグナル処理の準備の失敗) |
| 128 + 番号 | - | ジョブがシグナルで終了した、または conductor が中断された(SIGINT = 130、SIGTERM = 143) |

64〜68・70 は、ジョブ自身の終了コードと重なりうる。区別は標準エラーの `conductor: error: ...` の行で行う。

### 2.2 `exit_code_for(reported, is_error)`: 0〜255 への整形

プロセスの終了コードは 0〜255 に収める。

```text
if 0 <= reported <= 255:  return reported
low = reported AND 0xFF            # 負数も含め、2 の補数表現の下位 8 ビット
if low == 0 and is_error: return 1 # エラーを成功に見せない
return low
```

| reported | is_error | 結果 |
|---|---|---|
| 3 | true | 3 |
| 256 | true | 1(下位 8 ビット 0 → 1) |
| 256 | false | 0 |
| 257 | true | 1 |
| 511 | true | 255 |
| -1 | true | 255 |
| -256 | true | 1 |

`is_error` は `classification == error`。分類なし(起動しなかった中断)と成功・警告は `false`。

中断の終了コード `signal_exit_code(n)` = `exit_code_for(128 + n(32 ビット符号付き整数の飽和加算), true)`(パイプラインのジョブのあいだの中断で使う)。

「32 ビット符号付き整数の飽和加算」: 結果が 2147483647 を超えるなら 2147483647、-2147483648 を下回るなら -2147483648 にする(例: 128 + 2147483647 = 2147483647)。実在のシグナル番号(1〜64 程度)では起きない。

### 2.3 成功・警告・エラーの分類と返す値(`classify`)

- **成功コードの集合** = 定義の `success_exit_codes` があればその値だけ(0 を書かなければ 0 でも成功にならない)、無ければ `[0]`。
- **警告コードの集合** = `warning_exit_codes`(0 を含まず、成功コードと重ならないことは定義の検証で保証済み)。

判定は次の順に、最初に当てはまるものを採る(上ほど優先)。入力は 1 回の実行結果(§1.2)とジョブ定義。

| 順 | 条件 | classification | reported_exit_code | conductor_error |
|---|---|---|---|---|
| 1 | `started = false` かつ `interrupted` | なし | `128 + signal_number`(無ければ `128 + 0`)、飽和加算 | false |
| 1' | `started = false` かつ `interrupted` でない(あり得ない組み合わせ) | error | 127 | false |
| 2 | `interrupted`(起動済み) | error | `128 + signal_number`(無ければ 128)。ジョブ自身の終了コードは見ない | false |
| 3 | `start_failure` | error | 127 | false |
| 4 | `terminated_by_signal` が有り(値 n) | error | `128 + n` | false |
| 5 | `exit_code` が無い(起動して終わったのに終了コードがない) | error | 1 | **true** |
| 6 | `exit_code` ∈ 成功コード | success | 0 | false |
| 7 | `exit_code` ∈ 警告コード | warning | 0 | false |
| 8 | `exit_code = 0`(成功コードに無い) | error | 1 | **true** |
| 9 | それ以外 | error | `exit_code` そのまま(負数も、255 超もそのまま保持。整形は §2.2) | false |

全組み合わせの例(定義 `warning_exit_codes = [3, 4]`、`success_exit_codes` 省略):

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

`success_exit_codes = [2]` のとき: exit 2 → success(0)、exit 0 → error(1, `conductor_error`)。`success_exit_codes = [0, 2]` のとき exit 0 も success。

標準エラーへの常時の表示(`-v` なしでも):

- `conductor_error = true`: `conductor: error: job '<id>': exit code 0 is not listed in success_exit_codes`(日本語ロケールでは日本語。文言は [08-output-and-i18n.md](08-output-and-i18n.md))。`conductor_error = true` なら、順 5(終了コードが無い)の場合も同じ文言を出す(実装は `conductor_error` が真なら、理由を区別せず同じ行を出す)。
- 起動失敗: `conductor: error: job '<id>': cannot start command '<program>'`。

---

## 3. 遷移と次の動き(`classify_and_decide` / `next_step`)

ジョブ実行後に、分類と次の動き(`NextStep`)を決める。

### 3.1 `NextStep` のフィールド

| フィールド | 意味 |
|---|---|
| `next_job_id` | 次に実行するジョブ。完了・停止のとき無し(`pipeline` コマンドは、この値を使わない。§5.3) |
| `skipped_job_ids` | この時点でスキップとして記録するジョブ |
| `skip_reason_kind` | その理由(`byTransition` / `byEnd` / なし) |
| `end_reached` | `@end` に到達した |
| `completed` | 完了した |
| `resume_job_id` | エラー等で停止したときの再開位置(そのジョブ)。無しもある |

### 3.2 判定表

入力: 分類、そのジョブの該当する遷移(`on = success` は成功のとき、`on = warning` は警告のときに参照する。`error` の遷移は書けない)、`RunMode`、そのジョブの位置(index。定義の記述順、0 始まり)、各ジョブの現在の状態。

「未完了のジョブ」= 指定した区間のジョブのうち、状態が success / warning / skipped **でない**もの(pending・error・記録なし)。区間は、`@end` なら「index より後ろ全部」、遷移先 `T` なら「index と T のあいだ(両端を含まない)」。

| 分類 | 遷移 | RunMode | next_job_id | skipped_job_ids / kind | end_reached | completed | resume_job_id |
|---|---|---|---|---|---|---|---|
| error | (見ない) | 両方 | なし | 空 / なし | false | false | **このジョブ** |
| 分類なし(起動しなかった中断) | (見ない) | 両方 | なし | 空 / なし | false | false | **このジョブ** |
| success / warning | `@end` | Pipeline | なし | 後ろの未完了ジョブ全部 / `byEnd`(空なら kind なし) | true | **true** | なし |
| success / warning | `@end` | Individual | なし | **空** / kind = `byEnd`(後ろに未完了が 1 つでもあるとき。無ければ kind なし) | true | 後ろに未完了が無ければ true、あれば false | なし |
| success / warning | ジョブ `T`(index より後ろ) | 両方 | `T` | index と T の間の未完了ジョブ / `byTransition`(空なら kind なし) | false | false | なし |
| success / warning | 遷移なし(または遷移先が後ろのジョブでない。検証済み定義では起きない) | 両方 | 最後のジョブなら なし、そうでなければ次のジョブ | 空 / なし | false | 最後のジョブなら true、そうでなければ false | なし |
| 定義にないジョブ | - | - | error と同じ(停止) | | | | |

要点:

- 成功・警告のあと再開位置は無し。エラー・分類なしのあとは必ずそのジョブ。
- Individual の `@end` では、後ろのジョブを今は記録せず(`skipped_job_ids` は空)、`end_reached = true` だけを記録する。後ろのジョブは、スケジューラが 1 つずつ `run` を呼んだときに `Skip(AfterEnd)` が返り、呼ばれたジョブだけ `skipped(byEnd)` と記録される(§4.2、§5.4)。
- 遷移先 `T` の遷移では、`T` へ飛ぶ。`T` が index+1 なら間は空で、スキップは無い。
- 自動のリトライ(ジョブ失敗の再実行)は、`[job.retry]` を書かない限りしない(§7)。
- (v0.9.0 以降)遷移の行き先の解決は、純粋な関数 `resolve_transition(job, on, definition, index)` に切り出す。`End`(`@end`)、`Job(t)`(記述順で index より後ろのジョブの位置 `t`)、`Default`(遷移なし、または行き先が後ろのジョブでない場合。記述順で次のジョブ、最後なら完了)の 3 つを返す。判定表の「遷移」の列は、この関数の結果で決まり、`graph` の辺の取り出し(§15)も、同じ関数を呼ぶ(図と実際の動きが食い違わない)。判定表の出力は、v0.8.0 と変わらない。

### 3.3 `apply_outcome`(結果の記録。メモリ上)

入力: 現在のステート値 `S`、ジョブID、実行結果、結果(`JobOutcome`)、`NextStep`。`S` を複製して更新し、返す。

1. 対象ジョブ(`S` のジョブ状態一覧から id で探す。見つからなければこの手順 1 を飛ばす):
   - `classification` があれば `status` を success / warning / error にし、`skip_reason` を消す。分類なし(起動前の中断)なら `status` は**変えない**。
   - `exit_code` = 実行結果の `exit_code`(無ければ、以前の値も消す)。そのあと `status` が success / warning / error のいずれでもなければ `exit_code` を消す。
2. `skipped_job_ids` の各ジョブ(対象ジョブ自身は除く)を `status = skipped`、`exit_code` 無し、`skip_reason = skip_reason_kind`(無ければ `byTransition`)にする。
3. `resume_job_id = NextStep.resume_job_id`、`completed = NextStep.completed`。
4. `end_reached = S.end_reached OR NextStep.end_reached`(一度 true になったら戻さない)。

### 3.4 `apply_skip`(スキップの記録。メモリ上)

入力: ステート値 `S`、ジョブID、判断の理由(`DecisionReason`: `AlreadyExecuted` / `SkipTarget` / `AfterEnd` / `OutOfOrder`)、定義。

- 理由が `AlreadyExecuted` / `OutOfOrder` → 何も変えない(複製を返す)。
- `SkipTarget` → そのジョブを `skipped(byTransition)`、`AfterEnd` → `skipped(byEnd)` にし(`exit_code` 無し)、`AfterEnd` なら `end_reached = true`。ジョブが `S` に無ければ、更新せず複製を返す。
- そのあと、`S` に `error` のジョブが 1 つでもあれば、`resume_job_id` と `completed` は**変えない**(エラーの再開位置を守る)。無ければ:
  - `first_pending` = 定義の記述順で最初の `pending`(記録なしを含む)のジョブ(無ければ無し)
  - `completed` = 「いまスキップしたジョブが定義の最後のジョブ、かつ `first_pending` が無い」
  - `resume_job_id = first_pending`

### 3.5 `apply_interrupt`(中断の記録。メモリ上)

再開位置(§9)の値で更新する: `resume_job_id = ResumePoint.resume_job_id`、`completed = ResumePoint.completed`、`end_reached = S.end_reached OR ResumePoint.end_reached`。ジョブの状態は変えない(エラーとして記録しない)。

---

## 4. 起動前の判断(`decide_before_run`)

### 4.1 入力と出力

入力: ジョブID、定義、各ジョブの状態の一覧、`resume_job_id`(ステートの同名の値)、`end_reached`。ステートが空(ファイルなし・長さ 0・完了のまま残ったもの)のときは、状態一覧は空、`resume_job_id = 無し`、`end_reached = false` を渡す。

出力: `Run`(実行する)/ `Skip(理由)`(実行せず 0 で終わる。理由は `SkipTarget` / `AfterEnd` / `AlreadyExecuted`)/ `Reject(OutOfOrder)`(実行せず、ステートも変えず、66)。

### 4.2 判定表(上から最初に当てはまるものを採る)

状態 `st` = そのジョブの記録(無ければ pending)、`reason` = そのスキップ理由。

| 順 | 条件 | 結果 |
|---|---|---|
| 0 | ジョブIDが定義にない(`run` では、事前に 68 で止まるので到達しない) | `Reject(OutOfOrder)` |
| 1 | `st = skipped` かつ `reason = byEnd` | `Skip(AfterEnd)` |
| 1' | `st = skipped` かつ `reason` が byTransition または無し | `Skip(SkipTarget)` |
| 2 | `end_reached` かつ `st = pending` | `Skip(AfterEnd)` |
| 3 | `st` が success または warning | `Skip(AlreadyExecuted)` |
| 4 | `st = error`、または `resume_job_id` がこのジョブID | `Run` |
| 5 | 全ジョブが pending(または記録なし)で、このジョブが記述順で最初 | `Run` |
| 6 | `st = pending` で、記述順で前のジョブがすべて success / warning / skipped | `Run` |
| 7 | 上のどれでもない | `Reject(OutOfOrder)` |

例(4 ジョブ j0〜j3):

| 呼ぶジョブ | 状態 | resume | end_reached | 結果 |
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

補足: 規則 1' は、理由が書かれていない `skipped`(ステートファイル上は読み込みで弾かれるが、関数としては入りうる)を SkipTarget とする。規則 2 は `pending`(記録なしを含む)のときだけ。`error` のジョブは規則 4 で、`end_reached` でも `Run` になる。規則 6 は「記述順で前のすべて」を見るので、後ろのジョブが済んでいても影響しない。

### 4.3 判断の結果とステート・終了コード(呼び出し側)

| 結果 | ステートの変更 | 標準エラー | 終了コード |
|---|---|---|---|
| `Reject(r)` | しない(初期ステートも書かない) | `conductor: error: job '<id>': cannot run this job now: earlier jobs have not finished (out of order)`(日本語: `このジョブは、いま実行できません: 前のジョブが終わっていません(順序外)`) | 66 |
| `Skip(r)` | `apply_skip` の結果が元と**等しくなければ**、耐久性ありで書く。`AlreadyExecuted` は常に等しい(書かない) | `-v` のとき `[conductor] job=<id> skipped (<理由>)`。理由文: SkipTarget = `skipped by a transition`、AfterEnd = `skipped after @end`、AlreadyExecuted = `already executed` | 0(pipeline では続行) |
| `Run` | ステートが空なら先に初期ステートを、耐久性を省いて書く(§5) | - | ジョブの結果による |

「等しい」はステート値の全項目の比較。「耐久性あり(Durable)」「省く(Fast)」の定義は [06-state-file.md](06-state-file.md) の書き込み。

---

## 5. 実行の流れ

### 5.1 共通の前処理 `prepare`(`run`・`pipeline`)

順序は固定。最初に誤りがあった時点で、その終了コードで終わる(以降は行わない)。

| 順 | 処理 | 誤りのときの終了コード |
|---|---|---|
| 1 | 中断の合図(「立った/立っていない」と最初のシグナル番号を保持する共有の旗)を作り、シグナルの監視を開始(定義の検査より前)。すでに登録済みの場合は無視して続行 | 準備の失敗: 70(`conductor: error: failed to set up signal handling`) |
| 2 | 定義ファイルの読み込みと検証。`pipeline` は、引数の名前と定義の `[pipeline].name` を照合 | 64 |
| 3 | 全ジョブの `command` の検査(引用符の閉じ忘れ・空) | 64 |
| 4 | `run` のとき、ジョブIDが定義にあるか | 68(`unknown job`) |
| 5 | run ID の決定(§6) | 68 |
| 5a | (v0.9.0 以降)`.env` の読み取り(§14)。`pipeline` は `[pipeline].env_file` とすべてのジョブの `env_file`、`run` は `[pipeline].env_file` とそのジョブの `env_file`。誤りが 1 つでもあれば、ジョブを実行せず、ステートに触れずに止まる | 64 |
| 6 | ステートの読み込み([06-state-file.md](06-state-file.md) §5)。完了のまま残ったものは「空」 | 65 |
| 7 | 空でなければ、定義との照合(06 §6) | 65 |
| 8 | 実効の `remove_on_complete` = `--remove-on-complete` があれば true、なければ定義の `[pipeline].remove_on_complete`(省略時 false)。環境変数は見ない | - |

- 定義ファイルは `--config <パス>`、なければ実行ディレクトリの `conductor.toml`。
- 使い方の誤り(4 と 5 の 68)と `.env` の誤り(5a の 64)が同時にあるときは、先に検出する 68 が勝つ(5a は 5 のあと)。`--dry-run` は、この `prepare` の、シグナルを除く部分(1 を除く 2〜7)を共有する(§16)。
- シグナルを受け取っても、1〜7 は最後まで進む。64・65・66・スキップ(0)の終了コードが、中断の終了コードに優先する。
- `pipeline` で引数の名前が定義の名前と違う場合は 64(順 2)。引数の名前が無いのは使い方の誤り 68(コマンドライン解析。[04-cli.md](04-cli.md))。

### 5.2 `conductor run <job-id>` の擬似アルゴリズム

```text
session = prepare(...)                      # 5.1。誤りなら終了
(states, resume, end_reached) = session.state の job_states / resume_job_id / end_reached
                                (state が空なら [], 無し, false)
d = decide_before_run(job_id, def, states, resume, end_reached)

match d:
  Reject(r):  表示、return 66
  Skip(r):
    base = session.state か initial_state(...)(空なら)
    rec  = apply_skip(base, job_id, r, def)
    if rec != base:
        write(rec, Durable)                  # 失敗: 65、文脈=ジョブ起動前
        if rec.completed and remove_on_complete:
            reset_state()                    # 失敗: 65
    -v なら skipped 表示
    return 0
  Run:
    base = session.state か、(空なら) initial_state を Fast で書いたもの   # 書けなければ 65、ジョブは起動しない
    exec = run_job_with_retry(job)           # §7。ステートは書かない
    (outcome, next) = classify_and_decide(job, exec, base.job_states, def, Individual)
    表示(§5.5)
    rec = apply_outcome(base, job_id, exec, outcome, next)
    write(rec, Durable)                      # 失敗: 65、文脈=exec.started なら「ジョブは実行されました」、そうでなければ「起動していません」
    if next.completed and remove_on_complete:
        reset_state()                        # 失敗: 65
    return exit_code_for(outcome.reported_exit_code, outcome.classification == error)
```

注意:

- Individual では、遷移でスキップが決まったジョブ(`byTransition`)は、`apply_outcome` で記録済みなので、後で `run` されると `Skip(SkipTarget)`。
- 最後のジョブの `run` 完了、または `@end` 後の最後のジョブのスキップ記録で `completed = true`。
- 初期ステートは `Run` と決まった後にだけ書く(Reject・Skip の前には書かない)。ただし Skip のときは、ステートが空なら `initial_state`(06 §2.6)を土台に記録が書かれる。
- `classify_and_decide(job, exec, states, def, mode)` は、`classify`(§2.3)で結果を作り、その分類を使って `next_step`(§3.2)で次の動きを決めて、組で返す。

### 5.3 `conductor pipeline <name>` の擬似アルゴリズム

`pipeline` は、「次のジョブ」を `NextStep.next_job_id` で決めない。定義の記述順に全ジョブを 1 つずつ見て、`decide_before_run` で決める(遷移先・スキップは、ステートが運ぶ)。

```text
session = prepare(...)                       # 引数の名前が無ければ 68
current = session.state                      # 空なら 無し
for job in def.job (記述順):
  d = decide_before_run(job.id, def, current の状態一覧 (無しなら []), current.resume_job_id, current.end_reached)
  match d:
    Reject(r): 表示、return 66
    Skip(r):
      base = current か initial_state(...)
      rec  = apply_skip(base, job.id, r, def)
      if rec != base: write(rec, Durable)    # 失敗: 65
      -v なら skipped 表示
      current = rec
      if rec.completed: return finish(...)   # 下記
    Run:
      base = current、無ければ initial_state を Fast で書いたもの   # 失敗: 65
      exec = run_job_with_retry(job)
      (outcome, next) = classify_and_decide(job, exec, base.job_states, def, Pipeline)
      表示
      rec = apply_outcome(base, job.id, exec, outcome, next)
      write(rec, Durable)                    # 失敗: 65
      if outcome は error、または分類なし:
          return exit_code_for(outcome.reported_exit_code, classification == error)   # エラーで停止
      if next.completed: return finish(...)
      if 中断の合図が立っている(番号 n):      # ジョブのあいだの中断
          point = resume_point_on_interrupt(def, rec.job_states)
          write(apply_interrupt(rec, point), Durable)     # 失敗: 65
          -v なら「interrupted by signal n」
          return signal_exit_code(n)           # exit_code_for(128 + n, true)
      current = rec
# ループを抜けた(完了していない。起こらない想定)
表示 "cannot determine whether the pipeline is complete"; return 65   # ステートは初期化しない

finish: if not remove_on_complete: return 0
        reset_state() が成功 → 0、失敗 → 65
```

- 完了のステートは、`finish` の前に必ず Durable で書き込み済み。`remove_on_complete` が真のときだけ削除する(06 §8)。
- 「完了を判定できない」は、全ジョブが `AlreadyExecuted` などで通過し、どこでも `completed` が真にならなかったとき。標準エラー `conductor: error: <ステートファイルのパス>: cannot determine whether the pipeline is complete`(日本語: `パイプラインが完了したかを判定できません`)、終了コード 65。
- ジョブ実行後、`next.completed` の判定は、中断の合図の判定より**先**。完了していれば中断の記録はせず、`finish` へ進む。
- エラーのジョブで止まったときの再実行: 同じコマンドを再度呼ぶと、`success`/`warning` のジョブは `Skip(AlreadyExecuted)`(`-v` に `already executed`)、`error` のジョブから `Run`。
- `Skip` で `rec.completed` が真になるのは §3.4 の条件(スキップしたジョブが最後のジョブで、pending が無い)のとき。

#### 状態遷移の例(`pipeline`)

定義 `a → b → c`(遷移なし)。`b` が exit 3 でエラー。

| 時点 | job_states | resume_job_id | end_reached | completed | 終了コード |
|---|---|---|---|---|---|
| 初期(Fast で書く) | a=pending, b=pending, c=pending | なし | false | false | - |
| a 成功後 | a=success(0), b=pending, c=pending | なし | false | false | - |
| b エラー後 | a=success(0), b=error(3), c=pending | b | false | false | 3 |
| 再実行: a は Skip(AlreadyExecuted)、b 成功 | a=success, b=success(0), c=pending | なし | false | false | - |
| c 成功後 | 全部 success | なし | false | true | 0 |

定義 `a --success--> @end` のとき(Pipeline): a 成功で b, c は `skipped(byEnd)`、`end_reached = true`、`completed = true`、0 で終わる([06-state-file.md](06-state-file.md) §2.5 の例 2 のステート)。

### 5.4 `@end` 後の `run`(Individual)の流れ

`run a`(遷移 `@end`)→ `a=success`、`end_reached = true`、`completed = false`(後ろに未完了がある場合)。続けて `run b` → 規則 2 で `Skip(AfterEnd)` → `b=skipped(byEnd)`、`resume_job_id` = 最初の pending(`c`)。`run c` → `Skip(AfterEnd)` → `c=skipped(byEnd)`、最後のジョブで pending が無いので `completed = true`(`remove_on_complete` なら削除)。`@end` の後ろに未完了が無ければ、`run a` の時点で `completed = true`。

### 5.5 表示(`-v` と常時)

- 常に標準エラー: エラー行(`conductor: error: ...`)、起動失敗、成功コードに無い 0、リトライの知らせ(§7.7)、`reset --all` の警告。
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
| 2 | `TooLong` | 64 バイトを超える(許す文字は ASCII だけなので、文字数と同じ) |
| 3 | `InvalidChar` | `A-Za-z0-9._-` 以外のバイトがある |
| 4 | `LeadingDot` | 先頭が `.`(`..` を含む) |

有効: `1`, `r1`, `12345`, `build-2026.11_a`, `a.b`, 64 文字ちょうど。無効: `""`(ただし §6.2 の規則で空は「なし」)、65 文字、`a/b`、`a b`、`.x`、`..`、`日本語`。

### 6.2 決定(`resolve_run_id`、`prepare` の 5)

```text
env_name = 定義の [pipeline].run_id_env があればそれ、なければ "CONDUCTOR_RUN_ID"
if --run-id が指定された(値は空でもよい):
    value = そのオプションの値(subject = "--run-id")      # 環境変数は見ない
else:
    value = 環境変数 env_name の値(未定義なら 無し)(subject = "run ID (environment variable)")
if value が 無し または "": run ID なし
else: parse(value) が失敗なら 68(`invalid value: '<subject>'`)、成功なら run ID
```

- `--run-id ""` は、環境変数があっても「run ID なし」に固定する。
- `run_id_env` の形は定義の検証で保証される(`A-Za-z_` で始まり `A-Za-z0-9_` が続く 1 文字以上)。
- 環境変数名は定義で決まるので、定義の検査(64)の**後**、ステートの読み込み(65)の**前**に run ID を決める。
- `watch`・`reset` は定義を検証しない: `--run-id` があれば定義を読まず(環境変数名は既定)、なければ定義の `run_id_env` だけをゆるく読む。読めない・キーが無い・文字列でない・名前の形が違う、のどれでも「なし」= 既定の名前を使う。ほかの誤りは無視する。異常終了してはならない。
- 環境変数の値が規則に合わなければ、`run`・`pipeline`・`watch`・`reset`(`--all` を除く)で 68。`reset --all` は環境変数を見ない。

### 6.3 run ID とステートの関係

- run ID あり: ステートは `<定義ファイル名>.<run-id>.state`。中身に `run_id` = run ID を書く。run ID なし: `<定義ファイル名>.state`、`run_id` の項目無し。
- 読み込み時、中身の `run_id`(あるか、値)がファイル名側の run ID と一致しなければ壊れたステート(65)。詳細は 06 §5、§6。
- run ID は同じ定義でも run ごとに独立したステート・再開位置を持つ。同じ run ID で呼び直すと、止まったジョブから再開する。

---

## 7. リトライ(`[job.retry]`、v0.8.0)

### 7.1 設定(検証済みの値。`RetryPolicy`)

| キー | 型・範囲 | 意味 |
|---|---|---|
| `exit_codes` | 整数の一覧。空でない。各値は 1〜255。昇順・重複なしに正規化済み | リトライの対象にするジョブ自身の終了コード。このジョブの成功コード・警告コードと重ならない |
| `max` | 整数 1〜100 | リトライの最大回数(最初の実行を除く)。実行は合計で最大 `max + 1` 回 |
| `interval` | 文字列 `<整数><s|m|h>`。結果が 1 秒〜24 時間(86400 秒) | 1 回目のリトライの前の待ち時間。数字は半角の数字だけで、整数は 1 以上。単位は小文字 1 文字(`s` 秒、`m` 分 = 60 秒、`h` 時間 = 3600 秒)。例 `"30s"`、`"2m"`、`"1h"` |
| `multiplier` | 整数 2〜10 | 待ち時間にかける倍率。`max_interval` と同時に指定 |
| `max_interval` | `interval` と同じ書式。`interval` 以上 | 待ち時間の上限。`multiplier` と同時に指定 |

- `multiplier` と `max_interval` は両方書くか両方書かない。検証の詳細は [03-definition-format.md](03-definition-format.md)。
- 内部の表現: 待ち時間は秒の**64 ビット符号なし整数**、倍率は 32 ビット符号なし整数。浮動小数点は使わない。
- `[job.retry]` を書かないジョブは、従来どおり 1 回だけ実行する。

### 7.2 リトライの対象か(`is_retry_target`)

次の**すべて**が成り立つときだけ真。

1. `started = true`
2. `interrupted = false`
3. `start_failure = false`
4. `terminated_by_signal` が無い
5. `exit_code` があり、`exit_codes` に含まれる

分類(成功・警告・エラー)は見ない。対象の終了コードは成功・警告と重ならない(検証済み)ので、結果として対象はエラーの終了コードだけ。ジョブ自身が 127 で終わった場合は、`exit_codes` に 127 を書いていれば対象(conductor が付けた起動失敗の 127 は、`exit_code` を持たないので対象にならない)。

### 7.3 判断(`retry_decision(job, execution, retries_done, last_wait_secs)`)

```text
policy = job.retry か、無ければ return Stop
if not is_retry_target(policy, execution) or retries_done >= policy.max: return Stop
return Retry { wait_secs: next_wait_secs(policy, retries_done, last_wait_secs) }
```

### 7.4 待ち時間

```text
next_wait_secs:
  if multiplier と max_interval が両方ある:
      if retries_done == 0: min(interval, max_interval)          # = interval(検証で interval <= max_interval)
      else:                 min(saturating_mul(last_wait_secs, multiplier), max_interval)
  else:
      interval                                                    # 毎回同じ(固定間隔)
```

- k 回目のリトライ(k = `retries_done + 1`)の前の待ち時間を `W(k)` とすると: 固定間隔なら `W(k) = interval`。倍率ありなら `W(1) = interval`、`W(k) = min(W(k-1) × multiplier, max_interval)`(k ≥ 2)。
- `saturating_mul(a, b)` は「64 ビット符号なし整数の飽和乗算」: `a × b` が 18446744073709551615(2^64 - 1)を超えるなら 18446744073709551615、そうでなければ `a × b`。丸めはない(整数の秒だけ)。
- ただし、`interval` と `max_interval` は 24 時間(86400 秒)以下に検証されるため、実際の定義では、乗算がオーバーフローすることはない(最大でも 86400 × 10)。境界の例(関数単体の挙動): `last_wait_secs = 9223372036854775807(= 18446744073709551615 を 2 で割って切り捨てた値)`、`multiplier = 10`、`max_interval = 18446744073709551615` のとき、積は飽和して 18446744073709551615 になり、上限で止まる。再実装でこの極端な入力まで再現する必要は実用上ないが、乗算は桁あふれで折り返さないこと。
- 倍率の掛け算は、直前の**実際の**待ち時間(上限で頭打ちされた後の値)に対して行う。

例:

| interval | multiplier | max_interval | max | 待ち時間の列(秒) | 合計 |
|---|---|---|---|---|---|
| 30s | なし | なし | 3 | 30, 30, 30 | 90 |
| 10s | 2 | 60s | 5 | 10, 20, 40, 60, 60 | 190 |
| 60s | 3 | 60s | 3 | 60, 60, 60 | 180 |
| 1s | 10 | 3600s(1h) | 6 | 1, 10, 100, 1000, 3600, 3600 | 8311 |
| 1m | 2 | 24h | 4 | 60, 120, 240, 480 | 900 |
| 1h | 10 | 24h | 4 | 3600, 36000, 86400, 86400 | 212400 |

### 7.5 実行ループ(`run_job_with_retry`)。`run` と `pipeline` の共通

```text
retries_done = 0; last_wait_secs = 0
loop:
    # v0.9.0 以降: 試行ごとに、足す環境を作り直す(§14)。attempt = retries_done + 1
    env  = merge_env(def, job, env_files, EnvContext{ pipeline_name, job.id, run_id, attempt: retries_done + 1 })
    exec = run_job(job, 分割したコマンド, env の名前と値の並び, signal)   # 分割したコマンドは検査で作ったもの(どの試行も同じ)。v0.9.5 までは run_job(job, env の名前と値の並び, signal) で、中で分け直した。v0.8.0 以前は run_job(job, signal)
    match retry_decision(job, exec, retries_done, last_wait_secs):
        Stop: return exec                          # 最後の結果
        Retry{wait_secs}:
            表示: 常に 1 行(§7.7)。retry_number = retries_done + 1
            match wait(wait_secs, signal):
                Elapsed:        retries_done += 1; last_wait_secs = wait_secs; continue
                Interrupted(n): return JobExecution{ started=false, interrupted=true, signal_number=n,
                                                      exit_code=無し, start_failure=false, terminated_by_signal=無し }
```

- 返された最後の結果だけが、分類・表示・ステート記録の対象(ステートは最終の結果で 1 回だけ書かれ、途中では書かれない)。リトライの回数・直前の待ち時間はローカルな値で、ステートファイルに書かない。
- したがって、conductor が落ちたあとの再開では、そのジョブを最初の実行から数え直す。
- 使い切った(`retries_done == max` で、まだ対象の終了コード): `Stop` → 通常のエラーと同じ(最後の実行の終了コードで終わり、再開位置はそのジョブ)。再実行の結果が成功・警告なら通常どおり遷移へ進む。
- 総実行回数: 最大 `max + 1`。知らせの回数: 実際に実行し直す(または待機に入る)回数。
- 起動前に中断された(合図が立っていた)ジョブはリトライせず、その場で `started = false` の結果を返す(`retry_decision` は `started = false` で Stop)。

### 7.6 待機とシグナル(`wait_for(secs, signal)`)

```text
start = 単調増加の時計の現在時刻
loop:
    if signal が立っている: return Interrupted(最初の番号)     # 待機の開始時にも確かめる
    elapsed = 現在時刻 - start
    if elapsed >= secs: return Elapsed
    sleep(min(secs - elapsed, 50ms))
```

- 合図は、50 ミリ秒以下の間隔で確かめる。シグナルから 1 秒以内に待機を打ち切る(長い待ちでも)。
- 待機中に中断された場合は、次の実行をせず、分類なし(起動しなかった中断)として扱う: 終了コード `128 + 番号`、ジョブの状態は変えず(エラーとして記録しない)、再開位置はそのジョブ。`pipeline` でも、そこで止まって終わる(§5.3: 分類なしは停止)。
- 再開すると、そのジョブを最初の実行から数え直す。
- 実装に任せてよい点: 待機の実現方法(sleep の刻み、条件変数など)。守るべきは「合図から 1 秒以内に打ち切る」「待ち時間が `secs` 秒以上になるまで Elapsed を返さない」。

### 7.7 リトライの知らせ

待機の**前**に、`-v` の有無によらず、標準エラーに 1 行(エラーではない。`conductor: retry: ` で始まる)。待機が中断された場合も、その回の知らせは出ている。

- 英語: `conductor: retry: job '<id>' exited with code <last_exit_code>; retry <n> of <max> in <wait_secs> seconds`
- 日本語: `conductor: retry: ジョブ '<id>' が終了コード <last_exit_code> で終わりました。<wait_secs> 秒後にリトライします(<n>/<max> 回目)`
- `<last_exit_code>` は直前の実行の終了コード(`exit_code` が無ければ 0)。`<n>` は 1 から。`<wait_secs>` は秒の整数(10 進、桁区切りなし)で、1 のときも `seconds`(単数形の処理はない)。`<id>` は表示用にエスケープ・長さ制限される(規則は [08-output-and-i18n.md](08-output-and-i18n.md))。
- 例: `conductor: retry: job 'extract' exited with code 75; retry 2 of 5 in 20 seconds`

---

## 8. シグナルと中断(流れへの影響)

| 受信のタイミング | ジョブの結果 | ステートの記録 | 終了コード |
|---|---|---|---|
| ジョブの起動の前(`run`) | 起動しない。分類なし | ジョブの状態は変えない。`resume_job_id` = そのジョブ(初期ステートは、空だった場合に書かれている) | 128 + n |
| ジョブの実行中 | 子に同じシグナルを 1 回転送。終了を待つ。`interrupted`。分類 error | そのジョブを `error` で記録(`exit_code` はジョブの値があればそれ) | 128 + n |
| リトライの待機中 | 分類なし(§7.6) | 状態は変えない。再開位置 = そのジョブ | 128 + n |
| `pipeline` で、ジョブを実行・記録した直後(まだ完了していない) | - | 再開位置(§9)を記録 | `signal_exit_code(n)` = 128 + n |
| 定義の検査・ステートの確認の途中 | 検査は最後まで進む | 64・65・66・スキップ(0)が優先 | それぞれ |

- シグナルは SIGHUP・SIGINT・SIGQUIT・SIGTERM(UNIX 系)、Ctrl+C(Windows)。2 回目以降のシグナルは追加の動作をしない。終了コードに使う番号は、最初に受け取ったもの。詳細は [07-process-and-signals.md](07-process-and-signals.md)。
- 実行中の中断ではジョブ自身の終了コードは見ない(分類 error、返す値は 128 + n)。

---

## 9. 再開位置(`resume_point_on_interrupt`)

入力: 定義、各ジョブの状態。

```text
resume_job_id = 記述順で最初の「pending(または記録なし)または error」のジョブ(無ければ 無し)
completed     = (resume_job_id == 無し)
end_reached   = 「skipped(byEnd)」のジョブが 1 つでもあれば true
```

エラー・pending を区別せず、記述順で最初のものが再開位置。エラーのジョブが前にあれば、そのジョブ。この関数は記録を作らない(呼び出し側が `apply_interrupt`(§3.5)で書く)。

---

## 10. 完了と完了後

- 完了(`completed = true`)になるのは: 最後のジョブが success / warning で終わった、`@end` に到達した(Pipeline、または Individual で後ろに未完了が無い)、スキップの記録で最後のジョブまで済んだ(§3.4)。
- 完了のステートは `completed = true` のまま**残る**(`remove_on_complete` が真でない限り)。書式は変わらない。
- 完了のステートは、読み込み(`run`・`pipeline` の `prepare`)では「空」として扱う(定義との照合もしない)。次の `pipeline`・`run <最初のジョブ>` は先頭から始まり、初期ステートで上書きする。先頭以外のジョブを `run` で単独に呼ぶと順序外(66)。
- `remove_on_complete` が真のとき、完了のステートを書いた後に `reset_state` で削除する(失敗は 65。完了のステートは残る)。偽のときは削除を試みず、0 で終わる(`run` ではジョブの結果による終了コード)。
- `watch` は完了のステートも表示する(表の下に `completed`/`完了` の行)。

---

## 11. 保証の範囲(at-least-once)

- 結果の記録は、ジョブの終了の**後**。終了と書き込みのあいだに conductor が落ちると、ジョブは未実行として扱われ、再実行されうる(ジョブが 2 回動きうる)。
- 結果の書き込みに失敗したときは、ジョブの終了コードではなく 65 で終わり、「ジョブは実行されました」と表示する。
- `[job.retry]` のジョブは、最大 `max + 1` 回動く。
- 時間の上限(タイムアウト)はない。ジョブが終わらなければ待ち続ける。
- 排他制御はしない([06-state-file.md](06-state-file.md) §10)。

---

## 12. 実装の特記事項(再実装で再現すること)

| # | 項目 | 挙動 |
|---|---|---|
| 1 | リトライの知らせの単数形 | 待ちが 1 秒のとき `in 1 seconds`(英語)。単数形の処理はない |
| 2 | エラーのジョブを再実行して、起動前・待機中に中断されたとき | 分類なしなので `status` は error のまま変わらないが、`exit_code` は消える(`apply_outcome` が `exit_code` に実行結果の値 = 無しを書く) |
| 3 | 成功直後の `resume_job_id` | 成功・警告のあとの `NextStep.resume_job_id` は無し。ただし後続のスキップ記録(`apply_skip`)は、エラーが無いとき `resume_job_id` を最初の pending にする。そのため、遷移で飛ばした後のステートには、成功直後でも `resume_job_id` が入りうる |
| 4 | 分類なし(起動前の中断)の終了コードの整形 | `is_error = false` で整形する。`128 + n` が 256 以上かつ下位 8 ビットが 0 になる番号(n = 128 など)は、0 になりうる(実在のシグナル番号では起きない) |
| 5 | `pipeline` の「完了を判定できない」 | 全ジョブを通過しても `completed` が真にならなければ 65(ステートは初期化しない) |
| 6 | ステートの誤りの原因の文 | 種類の文はロケールに従うが、理由(例: `TOML として読めません`)は常に日本語の固定文(06 §5.2) |

## 13. 実装に任せてよい点と未確認

実装に任せてよい点(外から見える挙動が同じなら自由):

- 判断関数・記録関数の内部構造、データ型の表現(列挙・文字列など)。
- 待機の実現方法(§7.6)、シグナル監視の仕組み。
- 標準エラーの出力のバッファリング(ただし行単位で、順序を保つ)。

未確認:

- Windows での、実行中ジョブへのシグナル転送の細部と、`exit_code` が負になる場合の実機の値。この文書は UNIX 系での確認と、コードの読み取りに基づく。
- ジョブの標準入出力の扱い(子プロセスが conductor の入出力を引き継ぐか)。この文書の対象外として確認していない。実装が採る方式は [07-process-and-signals.md](07-process-and-signals.md) を参照。
- 複数シグナルがほぼ同時に届いたときの「最初の番号」の決まり方は、元の実装が使うシグナル監視部品の返す順序(番号の小さい順)に依存する。ほぼ同時の到着そのものの再現は未確認。再実装では「最初に観測した 1 つ」を保持すれば足りる。

---

## 14. 環境変数の注入(`env`、`env_file`、`.env`。v0.9.0 以降)

ジョブを起動するとき、引き継いだ環境(`conductor` が親から受け取った環境変数)に、定義で決めた変数を**足す**。書式は 03 の 7.6(`env`、`env_file`)と 7.7(`.env`)。取り除く書き方(`unset`)はない。ステートには、環境変数(名前も値も)を書かない(ステートの論理項目は変わらない)。

### 14.1 データ

| 名前 | 内容 |
|------|------|
| `EnvEntry` | `.env` の 1 項目。名前、値、行。値は秘密として扱い、デバッグの出力・ログに出さない(あれば伏せ字) |
| `EnvFileContent` | 1 つの `.env` の解析結果。解決済みのパス、項目の並び(書いた順)、警告の並び |
| `LoadedEnvFiles` | 1 回の呼び出しで読んだ、すべての `.env`。「定義に書かれた `env_file` の記述 → 内容」を引ける。同じ解決済みのパスは、1 つの内容を共有する |
| `EnvContext` | `{ pipeline_name, job_id, run_id(無いことがある), attempt(1 から) }`。自動の変数の入力 |
| `EnvOrigin` | `pipelineEnvFile`、`pipelineEnv`、`jobEnvFile`、`jobEnv`、`builtin`(名前の出どころ。名前の要約に使う) |
| `EffectiveEnv` | 1 回のジョブの起動に足す環境。`{ job_id, entries: list<{ name, value, origin }> }`。「名前と値の並び」(起動に渡す)と、「名前と出どころの並び」(値を含まない要約)を取り出せる |

### 14.2 層と優先順位

ジョブの環境は、後ろが勝つ順に、次の層を重ねて決まる。

```text
引き継いだ環境  <  [pipeline].env_file  <  [pipeline.env]  <  [job].env_file  <  [job.env]  <  自動の変数
```

- `env_file` が配列のときは、書いた順に読み、後ろのファイルが勝つ。
- 引き継いだ環境は、統合の結果に**含めない**(子プロセスが OS の規則で引き継ぐ。結果は「足すもの」だけ)。引き継いだ環境が数百の名前を持ちうるため、名前の要約にも含めない。

### 14.3 統合 `merge_env(def, job, files, ctx) -> EffectiveEnv`

純粋な関数。次の手順で、結果の並び(`entries`)を作る。

```text
entries = []
put(name, value, origin):        # 同じ名前があれば取り除き、末尾に足す
    entries から name が完全に同じ項目を取り除く
    entries の末尾に { name, value, origin } を足す

for ref in def.env_file:  (files から引ける内容があれば) for e in その項目: put(e.name, e.value, pipelineEnvFile)
for v   in def.env:                                                      put(v.name, v.value, pipelineEnv)
for ref in job.env_file:  (files から引ける内容があれば) for e in その項目: put(e.name, e.value, jobEnvFile)
for v   in job.env:                                                      put(v.name, v.value, jobEnv)
put("CONDUCTOR_PIPELINE", ctx.pipeline_name, builtin)
put("CONDUCTOR_JOB_ID",   ctx.job_id,        builtin)
if ctx.run_id がある: put("CONDUCTOR_RUN_ID", ctx.run_id, builtin)
put("CONDUCTOR_ATTEMPT",  十進の整数の文字列(ctx.attempt),  builtin)
return EffectiveEnv { job_id: job.id, entries }
```

- 完全に同じ名前は、後ろの層が勝って 1 つになる(並びでは、後ろの層の位置に移る)。**大文字小文字だけが違う名前は、別の名前として**、層の順に並べる(`foo` と `FOO` は両方が残る)。
- 引けない記述は、何も足さない(読み取りは、実行の前に済んでいる前提。誤りは返さない)。
- 予約名の拒否は、`merge_env` では行わない(定義の読み取りと `.env` の解析が拒否済み。03 の 7.6、7.7)ので、`merge_env` は誤りを返さない。

#### 自動の変数

| 名前 | 値 | 備考 |
|------|----|------|
| `CONDUCTOR_PIPELINE` | パイプライン名(`[pipeline].name`) | 常に足す。引き継いだ値を上書きする |
| `CONDUCTOR_JOB_ID` | ジョブID | 常に足す |
| `CONDUCTOR_RUN_ID` | run ID | run ID が**あるときだけ**足す。無いときは足さず、引き継いだ環境に `CONDUCTOR_RUN_ID` があっても、そのまま(取り除かない) |
| `CONDUCTOR_ATTEMPT` | 試行の番号(十進。1 から) | 常に足す |

- `CONDUCTOR_ATTEMPT` = (その呼び出しの中での、これまでのリトライの回数) + 1。リトライのたびに環境を**作り直す**(1、2、3、…)。リトライの回数はローカルな値で、ステートに書かないので、`conductor` が落ちた(または中断された)あとの再開では、また 1 から数える。
- 利用者は `CONDUCTOR_` で始まる名前を書けない(予約名)ので、自動の変数が利用者の値に上書きされることはない。ただし、引き継いだ環境に同名があれば、上書きする。

### 14.4 起動への反映

ジョブの起動(1.2 の `run_job`)は、足す環境変数(名前と値の並び)を受け取る。子プロセスを生成するとき、**並びの順に**、各 `(name, value)` を、引き継いだ環境に足す(同じ名前は上書き。引き継いだ環境は、残る)。同じ名前が並びに 2 回あれば、後ろが勝つ(`merge_env` は完全に同じ名前を 1 つにするので、通常は起きない)。値の展開はしない。

**受け入れる OS の違い**: 層をまたぐ、大文字小文字だけが違う名前(例: `[pipeline.env]` の `foo` と `[job.env]` の `FOO`)は、Linux と macOS では、子に両方が見える。Windows は環境変数の名前の大文字小文字を区別せず、並びの後ろに足した `FOO` だけが見える。完全に同じ名前の優先順位は、どの OS でも同じ。統合した環境の全体の大きさには、上限を設けない(OS の上限を超えて起動できないときは、従来の「起動の失敗」(終了コード 127)として扱う)。

### 14.5 いつ読むか

| コマンド | 読む `env_file` | 位置 | 誤りのとき |
|----------|-----------------|------|-----------|
| `pipeline`(実行) | `[pipeline].env_file` と、**すべてのジョブ**の `env_file`(実行済みのジョブ、遷移でスキップされるジョブの分も) | 5.1 の 5a(run ID の決定のあと、ステートの読み込みの前) | 64。ジョブを 1 つも実行せず、ステートに触れない |
| `run`(実行) | `[pipeline].env_file` と、**そのジョブ**の `env_file` だけ | 同 | 同 |
| `--dry-run`(`pipeline`・`run`) | 上の 2 行と同じ範囲(本番と同じ読み取り) | 準備の中(16 章) | 64(計画は出ない) |
| `validate`、`graph` | `pipeline` と同じ範囲 | 定義と `command` の検査のあと | 64 |

- 範囲は、次の関数が決める: `env_file_refs(def, job_id?)` = `[pipeline].env_file` を先に、続けて(`job_id` が無ければ全ジョブ、あればそのジョブだけの)`env_file` を、定義の記述順に並べ、同じ記述の文字列は 1 つにまとめる(書いた順を保つ)。一覧が空(`env_file = []` を含む)なら、読み取りは何もしない。
- 再開のときも、呼び出しごとに、現在のファイルを読み直す(ステートに内容を残さない)。
- 誤りは、すべてのファイル分を集め、表示する(先頭の 20 件と「ほか N 件」。08 の 4.12)。終了コードは 64。
- 読み取った結果は、その呼び出しの中だけで持つ。

### 14.6 値の秘匿

環境変数の値は、標準出力、標準エラー(`-v` を含む)、ステート、エラーの表示、デバッグの出力のどこにも出さない。出してよいのは名前だけ。

- 値を持つデータ(`EnvEntry`、`EffectiveEnv` の項目)のデバッグの出力は、値を伏せ字にする(それがある言語では)。
- `.env` と `env` の誤りの理由は、固定の文で、入力の値を含めない。名前は、`env` の誤りでは整形して(9.2)示し、`.env` の誤りの表示は、ファイル名と行番号と種類の固定の文だけ(名前も出さない)。
- 計画(`--dry-run`)と図(`graph`)のデータは、値を持たない(計画は、名前と出どころだけ。図は、環境変数を持たない)。

---

## 15. `graph`: 図の構造と配置(v0.9.0 以降)

`conductor graph` が出す図の、構造と配置の規則(純粋。ファイルにもプロセスにも環境変数にも触れない。文字への変換は 15.6、15.7)。コマンドの流れは 04 の 7.7、出力の全例は 08 の 11 章。

### 15.1 データ

| 名前 | 内容 |
|------|------|
| `EdgeKind` | `success`、`warning`(エラーには辺がない) |
| `EdgeTarget` | `job(ID)`、`end`(終点 `@end`。最後のジョブが完了したときの行き先も `end`) |
| `FlowEdge` | `{ from_job, to: EdgeTarget, kinds: list<EdgeKind>, skipped_jobs: list<string>, explicit: bool }`。`kinds` は 1 つ以上(成功、警告の順)。`skipped_jobs` は、元と行き先のあいだのジョブの ID(記述順)。`explicit` は、定義に遷移として書かれた辺か(書かずに記述順の次へ進む辺は `false`) |
| `NodeState` | `ok`(成功と警告をまとめる)、`skipped`、`error`、`pending` |
| `StateOverlay` | `{ states: list<(job_id, NodeState)>, resume_job_id: string? }`。環境変数もコマンドも持たない |
| `GraphNode` | `{ job_id: string?(終点は無し), is_end, state: NodeState?(--state のときだけ), is_resume }` |
| `FlowGraph` | `{ nodes: list<GraphNode>(定義の記述順、最後に終点), edges: list<FlowEdge> }` |
| `LaneGlyph` | `empty`、`node`(`*`)、`endNode`(`o`)、`line`(`\|`)、`fork`(`\`)、`join`(`/`)、`underscore`(`_`。交差の行の横線) |
| `LaneRow` | `{ glyphs: list<LaneGlyph>, node_index: int? }`。ノードの行では、`glyphs` の長さが、ラベルを置く位置 |
| `GraphLayout` | `lanes(rows: list<LaneRow>, main_line: list<LaneGlyph>)`(対応する形)、`edgeList`(それ以外)。`main_line` は、ノードごとの主線の列 0 の記号(15.5)。`-v` の詳細の行の列 0 にも使う(15.6) |

### 15.2 辺の取り出し(`flow_edges(definition) -> list<FlowEdge>`)

ジョブ(位置 `i`、0 始まり)ごとに、成功と警告の行き先の位置を決める。位置は、ジョブなら `0..n-1`、終点なら `n`(`n` = ジョブの数)。

```text
target_position(i, on):
    match resolve_transition(job[i], on, definition, i):      # 3.2。実行と同じ関数
        End        → (n,   explicit = true)
        Job(t)     → (t,   explicit = true)
        Default    → (i+1, explicit = false)                  # 記述順の次。最後のジョブなら終点(n)
```

- 成功の行き先 `s` と警告の行き先 `w` が同じなら、1 つの辺にまとめ、`kinds = [success, warning]`、`explicit` = どちらかが `true`。違うなら、2 つの辺(`[success]` と `[warning]`)。
- 同じ元の辺は、行き先の位置の昇順に並べる(記述順で次のジョブへの辺が先、終点が最後)。辺の全体の並びは、元のジョブの記述順。
- `skipped_jobs` = 元と行き先のあいだのジョブ(`jobs[i+1 .. min(行き先, n)]`。終点なら、末尾まで)の ID。行き先が次のジョブなら空。
- エラーに辺はない(エラーでは、ジョブは止まる)。

### 15.3 状態の重ね合わせ

- `node_state_of(status)`: `success`・`warning` → `ok`、`skipped` → `skipped`、`error` → `error`、`pending` → `pending`。
- `overlay_from_state(state)`: ステートの各ジョブの状態を重ねる。再開位置 `resume_job_id` は、**完了していないステートのときだけ**使う(完了したステート、`completed = true`、は、状態の記号は重ねるが、再開位置は示さない)。
- `pending_overlay(definition)`: ステートがないとき。すべてのジョブが `pending`、再開位置なし。
- `build_graph(definition, overlay?)`: 節は、定義の記述順に、ジョブごとに 1 つ、最後に終点(`job_id` なし、`is_end = true`、状態なし)。`overlay` があれば、各ジョブの `state` に、重ね合わせの状態(重ね合わせにないジョブは `pending`)、`is_resume` に、再開位置と一致するか。辺は `flow_edges`。

### 15.4 「対応する形」の判定

レーンの図(`git log --graph` と同じ見た目)で描けるのは、ジョブが主線(左の列)に記述順に並び、飛び越しの辺が、主線の右隣(列 1)から始まる線になる形。関数 `skip_targets(graph)` が、配置の途中結果(内部の値。公開しない)を返す。途中結果は、ジョブごとの「次の節(位置 `i+1`)への辺を持つか」と、飛び越しの種類と、飛び越しの行き先(節の位置の昇順の並び。0〜2 個。終点は `n`)を持つ。

ジョブごとに、辺を、次の節への辺と、それ以外の辺(飛び越し)に分け(辺の行き先が、元より後ろでなければ「無し」)、次の表で種類を決める。

| 種類 | 次の節への辺 | 飛び越しの辺 |
|------|--------------|--------------|
| 飛び越しなし | あり | なし |
| 通常の飛び越し | あり | 1 本 |
| 「両方の飛び越し」 | なし | 1 本(種類が成功と警告の両方) |
| 分岐 | なし | 2 本(行き先が異なる。近い順に `t1 < t2`。成功と警告のどちらが近いかは問わない) |

すべてのジョブが、どれかの種類なら、途中結果を返す(全体が「対応する形」)。1 つでも、どれでもないジョブ(次の節への辺も飛び越しもない、次の節への辺がなく成功だけの辺が 1 本、辺が 3 本以上など)があれば「無し」(辺の一覧)。**交差と同じ行き先は、どの組み合わせでも、対応する形**である(「両方の飛び越し」や分岐と組み合わさっても同じ。v0.9.3 までは、交差・同じ行き先と、「両方の飛び越し」・分岐が組み合わさる形を、辺の一覧にしていた)。

(最後のジョブの、終点への辺は、次の節への辺なので、飛び越しではない。)

ジョブの結果は成功と警告の 2 つなので、定義から作れる図の構造では、各ジョブの辺は 1 本か 2 本で、必ず上の表のどれかになる。辺の一覧は、図の構造がこの範囲を外れるとき(後ろ向きの辺、辺が 3 本以上のジョブなど)のためだけに残る。

対応する形の例: 飛び越しが 1 本、入れ子の 2 本(`a→f`、`b→e`)、続く 2 本(`a→c`、`d→f`)、交差する 2 本(`a→d`、`b→e`。交差の行が入る)、交差に同じ行き先が混ざる(`a→e`、`b→d`、`c→e`。寄せの行が入る)、同じ行き先に 2 本(`a→d` と `b→d`。合流)、両方の飛び越し(`a` の成功と警告が `c` へ)、分岐(成功が `c`、警告が `e` など)、両方の飛び越しと通常の飛び越しの交差(N1)、分岐と通常の飛び越しの交差(N2)、分岐の線への同じ行き先の合流(N5)。

### 15.5 レーンの配置(`layout_graph`)

対応する形なら `lanes`、そうでなければ `edgeList`。配置は、**1 つの仕組み(線の並びと寄せ)だけ**で行う(v0.9.3 までの、「両方の飛び越し」と分岐を含むグラフのための別の経路と、その特例は、なくした)。開いている線 `lanes`(内側が先頭)は、同じ行き先の重複を許す。列は、主線が 0、`lanes[j]` が列 `j + 1`。`after_fork` = 直前の行が分岐の行か。

**実行されうるジョブと、主線の列 0 の記号**: 先頭のジョブ(位置 0)から、辺(次の節への辺と、飛び越しの辺)をたどって着ける節を、実行されうる節とする(各辺を 1 回ずつたどる。ジョブと辺の数に比例する)。`main_line[j]` = ジョブ `j` が実行されうる、かつ次の節への辺を持つなら `line`、それ以外(終点を含む)は `empty`。ノード `j` と次のノードのあいだの行(分岐の行、合流の行、寄せの行)の列 0 は、`main_line[j]`。節 `i` の手前の寄せの行の列 0 は、`main_line[i - 1]`。通常の飛び越しだけのグラフ(すべてのジョブが、飛び越しなしか通常の飛び越し)では、すべてのジョブが実行されうるので、列 0 は常に `line` で、v0.9.3 までの出力と 1 文字も変わらない。`main_line` は、配置の結果に入れる(15.6 の `-v` が使う)。

節 `i`(`0..=n`)ごとに、次の順で行を作る。

1. **ノードの手前**。`lanes` に `i` があれば(到着する線。最初の 1 本の位置 `k`)、(a) 到着する線の行き先 `i` の組と、位置 `k` をまたぐ組を除いて、同じ行き先の重複の組を寄せ(後述)、(b) 位置 `k` の線を主線へ寄せ(`MOVE(k + 1, 0)`)、行き先が `i` の線を、すべて取り除き、(c) 残りの重複の組を寄せる。到着する線がなくても、(c) の重複の寄せは行う。この手前の手順で寄せの行を 1 行でも作ったら、`after_fork = false`。終点(`i == n`)も、同じ手順を通り、ノードの行は `[endNode, empty]` で終える。
2. **重複の組の寄せ**。重複の組は、前の線 `a`、後ろの線 `b`(同じ行き先。`a < b`)で、`a` の小さい順、同じなら `b` の小さい順。後ろの線を前の線へ寄せる(`MOVE(b + 1, a + 1)`)。1 回寄せるたびに、組の探索を先頭からやり直す。探索は、各 `a` について、次の同じ行き先の線だけを見ればよい(それより後ろの同じ行き先の線は、到着する線をまたぐ場合も含めて、同じ判定になる)ので、線の数に比例する(行き先から次の位置への表を 1 回作る)。
3. **`MOVE(c, d)`**(列 `c` の線を、列 `d` へ寄せる。`m` = 寄せる前の線の数、`rest` = 寄せる線を除いた `lanes`)。`c - d = 1` は 1 行: `[line, empty]` を `c - 1` 回、`[line, join]`、`[empty, join]` を `m - c` 回(行末の空白は取る。`|/`、`| |/`、`|/ /`)。`c - d ≥ 2` は 2 行。行 1 = 列 0〜`c - 1` を `line` で、間の空白のうち、右の `c - d - 2` 個を `underscore`、残りを `empty` にし、続けて `join`、続けて `[empty, join]` を `m - c` 回(例: `| |/ /`、`| |_|/`)。行 2 = `[line, empty]` を `d` 回、`[line, join]`、続けて、`rest` の `d` 番目以降の各要素について、`[X, empty]`(行末の空白は取る。例: `|/| /`)。`X` は、`join` か `line`。`rest` の最初の重複(それ以前の要素と行き先が同じになる最初の添字)以降は `join`。`d ≥ 1` のときは、さらに、寄せた線より外側の線(`rest` の末尾 `m - c` 個)で、行き先が次のノード `i` と等しい最初の要素から先も `join`。それ以外は `line`。どの行も、最後に、列 0 を、上の「主線の列 0 の記号」(`main_line[i - 1]`)に置き換え、行末の空白を取る。
4. **ノードの行**。`existing` = `lanes` の数。ジョブ `i` が通常の飛び越しか「両方の飛び越し」で、飛び越し `t` が `lanes` の先頭と同じなら合流(新しい線なし)。それ以外で飛び越しがあれば新しい線(通常の飛び越しと「両方の飛び越し」は 1 本、分岐は 2 本)、なければ飛び越しなし。`new` = 新しい線の数(0〜2)。既存の線の記号は、新しい線を作るなら、`after_fork` なら `fork`、そうでなければ `line`。新しい線を作らないとき、(b) 直前に作った行が 1 行の寄せ(`c - d = 1`)で、次のノード `i + 1` も `lanes` にある(最初の位置 `q`)なら、位置 `q` 以降を `join`、それより内側を `line`。(c) 直前に作った行が 2 行の、同じ行き先の寄せ(`d ≥ 1`)で、ジョブ `i` に飛び越しがなく、次のノード `i + 1` の位置が 0 なら、位置 0 だけ `join`、ほかは `line`。(d) それ以外は `line`(到着の 2 行の寄せ(`d = 0`、`k ≥ 1`)の直後も `line`)。幅 = `2 × (1 + existing + new)`。行 = `[node]` に、`[empty, 既存の線の記号]` を `existing` 回足し、幅まで `empty` で埋めたもの。ラベルは、この幅の位置に置く。
5. **分岐の行、または合流の行**。新しい線を作るなら、`fork_row(existing, new, main_line[i])` の行を作り、`lanes` の先頭に、行き先を昇順で入れる(分岐は、近い行き先 `t1` が内側、遠い行き先 `t2` が外側。既存の線は、`new` だけ外側へずれる)。分岐の行き先が、既存の線と同じでも、2 本を入れ、次のノードの手前の寄せで 1 本にする。`fork_row(e, k, first)` = `[first, fork]` に、`[empty, fork]` を `e + k - 1` 回足したもの(`|\`、`|\ \`、` \ \`、…)。合流なら、`join_lane_row(existing - 1, main_line[i])` の行を作り、線は増やさない。`join_lane_row(o, first)` = `[first, fork, line]` に、`[empty, line]` を `o` 回足したもの(`|\|`、` \|`、`|\| |`、…)。そのあと、`after_fork` = 新しい線を作ったか。

全ノードが、ちょうど 1 回ずつ、記述順(最後が終点)に現れる。これらの規則は、レーンの配置の正解データと、1 文字も違わず一致する。正解データは、`git log --graph` の実際の出力(行き先がすべて違う形の 537 例、同じ行き先を含む形の 1,517 例、交差する形のジョブ 3〜7 個の 940 例、交差と同じ行き先が混ざる形のジョブ 4〜6 個の 487 例)と、**`git log --graph` の出力ではなく、規則(設計の参照実装)から導いた図**(分岐を含む形の 871 例。主線の列 0 の規則で作り直した第 2 版。「両方の飛び越し」か分岐を含む、ジョブ 3〜5 個のすべての形の 2,748 例。ジョブ 6〜7 個の 300 例)。git は、起点からたどれるコミットだけを描くので、次のジョブへの辺がないジョブの次のジョブが、git の図に出ない。そのため、後者は git では確かめられない。この配置の規則は、正解データの例から導いたもので、正解データが原本である(git のソースは読まない)。線が非常に多い形(1,000 ジョブなど)は、規則の一般化で、性能のテストで、1 秒以内に配置できることだけを確かめる。

### 15.6 text の整形

`lanes` ならレーンの図、`edgeList` なら辺の一覧。各行の末尾の空白は取り除き、最後に改行を付ける。

**レーンの図**: 各 `LaneRow` を、`empty`=空白、`node`=`*`、`endNode`=`o`、`line`=`|`、`fork`=`\`、`join`=`/`、`underscore`=`_` で文字にする。ノードの行は、そのあとに**ラベル**を付ける: 状態の記号(`--state` のとき。`[OK] `、`[--] `、`[NG] `、`[  ] `。末尾に空白 1 つ)、ジョブID(制御文字をエスケープ)、再開位置のジョブなら `  <- resume here`(日本語 `  <- 再開位置`。先頭に空白 2 つ)。終点は `@end`(状態の記号も注記もない)。ノードの行でない行(分岐、合流)は、記号だけ。

**`-v` の詳細**: ジョブのノードの行(終点を除く)の直後に、詳細の行を置く。ただし、ノードの行の直後が分岐の行(同じ行き先への合流の行 `|\|` を含む。`fork` を含む行)なら、その行を先に出してから、詳細の行を置く(合流の行の前)。詳細の行の接頭辞 = 列 0 の文字(配置の結果の `main_line` のそのノードの記号。`line` なら `|`、`empty` なら空白。15.5)と空白 1 つに、`"| "` を(そのノードの行の列の数 - 1)回繰り返して続けたもの。列の数 = `glyphs` の長さ ÷ 2(切り捨て)。そのため、実行されないジョブと、次のジョブへの辺がないジョブ(「両方の飛び越し」、分岐)の詳細の行の列 0 は、空白になる。詳細の文は、次の順(該当しない項目は出さない):

1. `command: <コマンド>`(日本語 `コマンド: `)。コマンドは、文字列ならそのまま(エスケープのみ)、配列なら要素ごとに(空、または空白を含む要素だけ二重引用符で囲み、エスケープ)空白でつなぐ。
2. `retry: <要約>`(日本語 `リトライ: `)。`[job.retry]` があるときだけ。要約は、英語 `exit codes 75, 76, up to 3 times, every 5 s`(倍率と上限があれば ` (x2, max 60 s)` を続ける)、日本語 `終了コード 75、76 を最大 3 回、間隔 5 秒`(あれば `(倍率 2、上限 60 秒)` を続ける)。
3. 飛び越しの注記。そのジョブから出る辺が 2 つ以上ある、または飛び越すジョブがあるとき。辺を、成功の辺を先、警告の辺を後に、`<種類> -> <行き先>`(日本語 `<種類> → <行き先>`)、飛び越すジョブがあれば `(skips b, c)`(日本語は `(b、c を飛び越す)`。区切りは英語 `, `、日本語 `、`)を続けたものを、英語 `, `、日本語 `、` でつなぐ。種類は、`success`/`warning`/`success, warning`(日本語 `成功`/`警告`/`成功・警告`)。

**辺の一覧**(対応しない形): 次の行を出す。

```text
<注記の行>                      注記: レーンの図で描けない形のため、レーンの図の代わりに、辺の一覧を出します。
ジョブ:                          (英語は Jobs:)
  <ラベル>                       ジョブごとに 1 行(2 つの空白の字下げ。ラベルは、状態の記号とジョブID、再開位置の注記)。終点は含めない
    <詳細の文>                   -v のときだけ。4 つの空白の字下げ。上の 1〜3
辺:                              (英語は Edges:)
  <元> -> <行き先>[ (<種類>)]    辺ごとに 1 行(2 つの空白の字下げ。`->` は日本語でも ASCII)
```

- 注記の行(英語): `note: the graph has a shape the lane diagram cannot draw, so it is replaced by a list of edges.`
- 辺の行の `(<種類>)` は、**主線の辺**(次のジョブへの、成功と警告の両方の辺。`kinds` が 2 つで `skipped_jobs` が空)以外に付ける。種類は日本語 `(成功)`/`(警告)`/`(成功・警告)`、英語 `(success)`/`(warning)`/`(success, warning)`。

### 15.7 Mermaid

形によらず常に出せる。`-v` は影響しない。次の順に、各行を 2 つの空白で字下げして出す(最初の行を除く)。

1. `flowchart TD`。
2. 節の定義: ジョブ `k`(0 始まりの位置)は `n<k>["<ラベル>"]`、終点は `nend(["@end"])`。ラベル = 状態の記号(`--state` のとき。`[OK] ` など)+ ジョブID(下のエスケープ)。**ジョブIDは、二重引用符つきのラベルに書く**(どのIDでも図が壊れない)。
3. 辺の定義(`flow_edges` の順): 主線の辺(`kinds` が 2 つで `skipped_jobs` が空)は `<元> --> <行き先>`、それ以外は `<元> -->|"<種類>"| <行き先>`(種類は text と同じ語。言語に従う)。
4. 再開位置があるときだけ: `classDef resume stroke-width:3px`、`class n<k> resume`(この 2 行)。

エスケープ(ラベルの中): `#` → `#35;`(**最初に**置き換える。置き換えで作った `#` は、さらに置き換えない)、`"` → `#quot;`、`<` → `#lt;`、`>` → `#gt;`、`&` → `#amp;`、`` ` `` → `#96;`、そのほかの制御文字 → `#<10 進>;`。

---

## 16. `--dry-run`: 実行計画(v0.9.0 以降)

`pipeline <名前> --dry-run` と `run <ジョブID> --dry-run` の計画の生成(純粋。ファイルにも、プロセスにも、環境変数にも触れない)。流れは 04 の 7.9、文字への整形は 08 の 11 章。実行と**同じ判断**(起動前の判断、分類と次の動き)を、ジョブを起動する代わりに、仮定した終了コード(指定がなければ 0)で進める。

### 16.1 データ

| 名前 | 内容 |
|------|------|
| `Assumption` | `{ job_id, exit_code: 整数 }`。終了コードの範囲は 0〜255 |
| `PlanDisposition` | `willRun`(実行する。模擬での結果 `SimulatedResult` を持つ)、`alreadyExecuted`(実行済み)、`skippedByTransition`(遷移でスキップ)、`skippedAfterEnd`(`@end` のあとでスキップ)、`notReached`(模擬でエラーの停止のあとのため、実行されない) |
| `SimulatedResult` | `{ exit_code, classification, assumed: bool(--assume で指定したか。false は既定の成功)、retryable: bool(終了コードがリトライの対象に含まれるか。注記のためだけで、模擬はリトライを繰り返さない) }`。`willRun` が持つ |
| `PlanEntry` | `{ job_id, disposition, is_resume, command_words: list<string>(分割後。先頭がプログラム), env_names: list<(name, origin)>(試行 1 で渡される名前と出どころ), retry: RetryPolicy? }`(模擬の結果は `disposition` の `willRun` が持つ)。`willRun` でないジョブにも、コマンドなどを持つ(表示は `willRun` だけ) |
| 計画の方式 | `pipeline`、`individual` |
| `ResumeInfo` | 3 つの場合のどれか: `fromStart`(ステートなし、または完了したステート)、`afterStop`(前回の停止で再開位置あり。再開位置のジョブ ID を持つ)、`individual`(`run` の対象。対象のジョブ ID を持つ) |
| `PlanEnd` | `completed`、`endReached`(`@end` に到達)、`stoppedAtError(job_id)` |
| `NextStepNote` | `{ next_job_id, skipped_job_ids, completed, end_reached, stopped }`(`run` の、仮定の結果による次の動き) |
| `ExecutionPlan` | `{ pipeline_name, run_id, mode, resume, assumptions, unused_assumptions: list<string>, entries, end, next_step_note? }` |
| `PlanRejection` | `{ job_id, reason }`。本番なら順序外として拒否されるとき |
| 計画の入力 | `{ definition, state?, commands: list<list<string>>(記述順。分割済み)、env_files, run_id?, assumptions }` |

### 16.2 `plan_pipeline(input) -> ExecutionPlan | PlanRejection`

```text
active = input.state があり completed でないもの          # 完了したステートは「ステートなし」(本番と同じ)
states = active があれば active.job_states の複製、なければ全ジョブ pending
initial_resume = active?.resume_job_id
resume = initial_resume; end_reached = active?.end_reached(なければ false); stopped = 無し
for (index, job) in def.job (記述順):
    is_resume = (initial_resume == job.id)
    if stopped: entries に notReached を足し、continue
    match decide_before_run(job.id, def, states, resume, end_reached):
        Reject(reason): return PlanRejection{ job.id, reason }
        Skip(reason):   entries に skip_disposition(reason) を足す          # 模擬の状態は変えない
        Run:
            (code, assumed) = job.id の仮定があれば (その終了コード, true)、なければ (0, false)
            (result, next) = simulate_job(job, def, states, code, assumed, Pipeline)
            apply_result(states, job.id, code, result.classification, next)
            resume = next.resume_job_id; end_reached = end_reached || next.end_reached
            if result.classification == error: stopped = job.id
            entries に willRun(result つき)を足す
end = stopped があれば stoppedAtError(その ID)、なければ end_reached なら endReached、なければ completed
resume 情報 = initial_resume があれば afterStop(job_id)、なければ fromStart
unused_assumptions = 仮定のうち、willRun の項目にならなかったジョブID(指定順)
```

- `skip_disposition(reason)`: `AlreadyExecuted` → `alreadyExecuted`、`AfterEnd` → `skippedAfterEnd`、`SkipTarget`・`OutOfOrder` → `skippedByTransition`。
- `simulate_job`: 合成した実行結果(`started = true`、`exit_code = code`、ほかは false/無し)を、分類と次の動き(3 章の `classify_and_decide`)に渡して、分類と次の動きを得る。分類は必ずある(無ければエラー扱い)。`retryable` = `job.retry` があり、その `exit_codes` に `code` を含む。
- `apply_result`(3.3 の `apply_outcome` と同じ結果にする): ジョブの状態を分類のとおり(success / warning / error)にし、`exit_code = code`、`skip_reason` を消す。次の動きの `skipped_job_ids`(自分自身を除く)を `skipped` にし、`exit_code` を消し、`skip_reason` = `skip_reason_kind`(無ければ `byTransition`)にする。再開位置と `end_reached` は、呼び出し側が次の動きから更新する。
- 項目: `command_words` = `input.commands[index]`(無ければ空)、`retry` = `job.retry`、`env_names` = `merge_env(def, job, env_files, EnvContext{ pipeline_name, job.id, run_id, attempt: 1 })` の名前と出どころ(値は持たない)。
- 状態の探索は、ジョブの数に比例する時間で済むこと(起動前の判断は、記録が同じ位置にあればそれを使い、なければ、ジョブIDで探す。1,000 ジョブの計画の性能のため。挙動は変わらない)。

### 16.3 `plan_run(job_id, input)`

```text
index = def のジョブ位置(job_id); なければ PlanRejection{ job_id, OutOfOrder }      # 通常は、68 で先に止まる
(states, resume, end_reached) = active があれば (active.job_states, active.resume_job_id, active.end_reached)、なければ ([], 無し, false)
match decide_before_run(job_id, def, states, resume, end_reached):
    Reject(reason): return PlanRejection
    Skip(reason):   項目 = skip_disposition(reason)、end = completed、note = 無し
    Run:
        (code, assumed) = 仮定または (0, false)
        (result, next) = simulate_job(..., Individual)
        stopped = (result.classification == error)
        note = NextStepNote{ next.next_job_id, next.skipped_job_ids, next.completed, next.end_reached, stopped }
        end = stopped なら stoppedAtError(job_id)、next.end_reached なら endReached、なければ completed
        項目 = willRun(result つき)
resume 情報 = individual(job_id)、entries = [項目]、unused_assumptions = 仮定のうち、job_id が willRun でないもの
```

- `run` では、`is_resume` は常に false。`next_step_note` は、対象が `willRun` のときだけ有り。`end` は参考の値で、表示しない。

### 16.4 使われなかった仮定

仮定したジョブが、遷移でスキップされる、実行済みである、停止のあとである、`run` の対象でない、のいずれかで `willRun` にならなければ、誤りにせず、計画の `unused_assumptions` に入れる(指定順)。定義にないジョブIDの仮定は、計画の前に 68 で拒否される(04 の 7.9)。

---

## 17. `init`(v0.9.0 以降)

`conductor init` は、定義のひな形を作る。ひな形の文面は 08 の 12 章、ファイルの作成の規則は 06 の 14 章、流れは 04 の 7.8。要点:

- ひな形は、`validate` に必ず通る定義(`[pipeline.env]` と `[job.retry]` の例を、有効な行として含む)。入力は、言語(ロケール)とパイプライン名だけで、利用者の環境変数や `.env`、既存のファイルの内容は入らない。
- 名前は、TOML の基本文字列として、`"`、`\`、制御文字(U+0000〜U+001F、U+007F は `\uXXXX`)をエスケープして `[pipeline].name` に入れる。名前は、コメントには書かない。
- 出力先が既にある(`--force` なし)と、何も変えず 69。`--force` は、同じディレクトリの一時ファイルに書いて、置き換える。`--config -` は標準出力。
