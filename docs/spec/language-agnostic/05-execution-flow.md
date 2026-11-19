# 05. 実行の意味論(終了コード・分岐・再開・run ID・リトライ・パイプライン)

対象: `conductor` v0.8.0 と同じ挙動の実行ツール。この文書は、`run`(ジョブを 1 つ実行)と `pipeline`(定義の全ジョブを順に実行)の実行の流れを、判定表・状態遷移・言語中立の擬似コードで定める。この文書だけで、任意の言語・ライブラリで同じ挙動を実装できることを目標にする。

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
| `start_failure` | 真偽 | 起動に失敗した。command を引数に分けられない、プロセス生成の失敗、終了状態を取得できない、のいずれか |
| `interrupted` | 真偽 | conductor がシグナルを受け取り、中断の合図が立っている(ジョブの終了を観測した直後に立っていた場合を含む) |
| `signal_number` | 整数、または無し | `interrupted` のとき、最初に受け取ったシグナルの番号 |

ジョブの起動手順(`run_job`):

1. command を引数に分ける(規則は [07-process-and-signals.md](07-process-and-signals.md))。失敗なら `start_failure = true`(`started = true`)で終わる。
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
| 6 | ステートの読み込み([06-state-file.md](06-state-file.md) §5)。完了のまま残ったものは「空」 | 65 |
| 7 | 空でなければ、定義との照合(06 §6) | 65 |
| 8 | 実効の `remove_on_complete` = `--remove-on-complete` があれば true、なければ定義の `[pipeline].remove_on_complete`(省略時 false)。環境変数は見ない | - |

- 定義ファイルは `--config <パス>`、なければ実行ディレクトリの `conductor.toml`。
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
    exec = run_job(job, signal)
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
