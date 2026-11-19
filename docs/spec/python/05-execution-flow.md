# 05. 実行の意味論(終了コード・分岐・再開・run ID・リトライ・パイプライン)

対象: Python で実装する `conductor` v0.8.0 と同じ挙動の実行ツール。この文書は、`run`(ジョブを 1 つ実行)と `pipeline`(定義の全ジョブを順に実行)の実行の流れを、判定表・状態遷移・擬似コードで定める。数値の計算(待ち時間の倍率・上限・飽和・秒から待機への変換)は、Python の任意精度 `int` を使う方式として、式・境界の例・落とし穴まで定める。待機とシグナルによる中断は、同期の `time.sleep` を短い刻みで区切り、シグナルのフラグを毎回確かめる方式に決める(§7.8)。この文書だけで、Python でゼロから同じ挙動を実装できることを目標にする。

ステートファイルの論理項目・保存の性質・読み込みの検査は [06-state-file.md](06-state-file.md) にある。この文書の「ステートへの記録」は、06 の形式のファイルを書く前の、メモリ上のステート(以下「ステート値」)の更新を指す。定義ファイルの書式は [03-definition-format.md](03-definition-format.md)、コマンドラインは [04-cli.md](04-cli.md)、プロセス起動とシグナルは [07-process-and-signals.md](07-process-and-signals.md)、メッセージの文言は [08-output-and-i18n.md](08-output-and-i18n.md)、終了コードの総覧は [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md) を参照。

この文書の確認の範囲: 「確認済み」と明記した Python の挙動は、macOS(Darwin 27.2)上の CPython 3.9.6・3.14.10・3.15.2 で実際に動かして確かめた(型の骨格は mypy 2.4.1 の `--strict` で確かめた)。3.9 はこの仕様がサポートする最小の版より古く、`dataclass(slots=True)` が使えないことなど、差の確認だけに使った。Windows と、ほかの OS・版は未確認(§14)。サポートする版は「実装時に確認」する([11-quality-and-build.md](11-quality-and-build.md))。この文書の型の骨格は `dataclass(slots=True)`・`X | None`・`match` を使うので、`requires-python` は 3.10 以上が前提になる。

## 0. 構成の決定

### 0.1 モジュールの置き場所

「判断」(分類・次の動き・起動前の判断・リトライの判断・再開位置)は、ファイルにもプロセスにも触れない**純粋な関数**にし、例外を投げず必ず結果を返す。

| パッケージ | 置くもの |
|---|---|
| `conductor/flow/` | 共有の値の型(`JobStatus`、`SkipReason`、`JobState`、`PipelineState`、`JobExecution` ほか)、`classify`、`classify_and_decide`、`next_step`、`decide_before_run`、`retry_decision`、`next_wait_secs`、`sat_mul_u64`、`resume_point_on_interrupt`、`exit_code_for`、`signal_exit_code`、`sat_add_i32`。`os`・`sys`・`time`・`signal`・`subprocess`・`pathlib`・`threading` を import しない(ファイル・時計・環境変数・シグナルに触れない)。`definition` の型だけに依存してよい |
| `conductor/state/` | 記録の関数(`apply_outcome`、`apply_skip`、`apply_interrupt`、`initial_state`、`state_equals`。純粋。複製して更新)、読み書き([06](06-state-file.md)) |
| `conductor/runid.py` | run ID の検査と決定(§6) |
| `conductor/runner/` | `run_job`(子プロセスの起動と待機)、`wait_retry`(§7.8)、シグナルのフラグ([07](07-process-and-signals.md)) |
| `conductor/cli/` | `prepare`、`run`・`pipeline` の流れ(§5)、リトライの実行ループ(§7.7)。`Deps` を介して時計・環境・ファイル・プロセスに触れる |

依存の向きは `definition` ← `flow` ← (`state`、`runner`) ← `cli`。`state` と `runner` は互いに依存しない。循環させない([02-architecture.md](02-architecture.md))。

### 0.2 型と表記の約束(この文書の全体)

| 値 | Python の型 | 理由・注意 |
|---|---|---|
| 終了コード(ジョブ自身のもの、`reported_exit_code`、終了コードの一覧の値) | `int`(32 ビット符号付き整数の範囲 -2147483648〜2147483647 に収まる値) | 範囲外の値は、実行結果を作る側(runner)が範囲に直してから `flow` に渡す(§1.2 の注意)。整形(§2.2)は `reported & 0xFF` を使う。`int` の `&` は負数を無限の桁の 2 の補数として扱うので、負数でも下位 8 ビットが正しく取れる(確認済み: `(-1) & 0xFF` は 255、`(-256) & 0xFF` は 0、`(-2**31) & 0xFF` は 0) |
| 待ち時間(秒。`interval_secs`、`max_interval_secs`、各回の待ち、`last_wait_secs`、飽和の演算の値) | `int`(0〜2**64-1 の範囲で扱う) | 任意精度なので桁あふれしない。64 ビット符号なし整数の飽和は `min(x, 2**64 - 1)` で表す(§7.6)。**折り返し(`& (2**64 - 1)`)にしない** |
| 倍率 `multiplier`(2〜10)、`max`(1〜100)、`retries_done`、リトライの回数 `n` | `int` | 小さい値 |
| シグナル番号 | `int` | 終了コード `128 + n` の計算に使う |
| 時刻・経過時間(待機の計測) | `int` のナノ秒(`time.monotonic_ns()`) | 単調増加。実時計(`time.time()`)の調整に影響されない。浮動小数点の秒を持ち回らない |
| 真偽 | `bool` | **`bool` は `int` のサブクラス**(§0.3) |

- 「無し」(値がない)は `None` で表す。ステート値の「項目が無い」と `JobExecution` の「値が無い」は、どちらも `None`(ステートファイルの JSON の `null` は読み込みで拒否する。06)。
- ステート値と判断の入出力は、`@dataclass(frozen=True, slots=True)` の不変の値にし、配列は `tuple` で持つ(`list` は `frozen` でも変更できてしまう)。更新は `dataclasses.replace` か新しい値を作る(元は変えない)。dataclass の `==` は全フィールドの比較なので、「ステート値の全項目の比較」(§4.3)は `==` でよい(確認済み: 同じ値の 2 つは `==` が真、1 つでも違えば偽)。`None` と値は区別される。
- 文字列の値の集合(`JobStatus` など)は `typing.Literal` で表す。ステートの JSON から読んだ文字列と変換なしで比べられ、`match` と型検査(`mypy --strict`)で網羅を確かめられるため。判別共用体(`Decision` など)は、`kind: Literal[...]` を持つ `dataclass` の `|` の和で表す(確認済み: mypy `--strict` が `match` の網羅を通す)。`enum.IntEnum` は固定の終了コード(`ExitCode`)にだけ使う(§2.1)。
- 型の骨格は mypy `--strict` を通る形にする。骨格の `...` の本体は説明のためで、実装では値を返す。

### 0.3 `bool` が `int` である点と、`float` の扱い

| # | 事実(確認済み) | この仕様の規定 |
|---|---|---|
| 1 | `isinstance(True, int)` は真。`True + 1` は 2、`True * 2` は 2、`min(True, 5)` は `True`。`sys.exit(True)` の終了コードは 1 | 整数であることを確かめる箇所(定義の読み込み、ステートの読み込み)は、`type(x) is int` で判定する(`isinstance(x, int)` は使わない)。`true` / `false` を整数として受理してはならない |
| 2 | mypy `--strict` は、`int` の引数に `bool` を渡しても誤りにしない(確認済み: `f(True)` が型検査を通り、2 を返す) | 型検査では防げない。`flow` の関数は検証済みの値を受け取る前提とし、検証は入力の境界(`definition`、`state` の読み込み)で行う |
| 3 | `json.dumps({"exit_code": True})` は `{"exit_code": true}` を出す(確認済み) | ステートを書く前に、`exit_code` が `type(x) is int` であることを確かめる。偽の値を作った実装の誤りを、黙って `true` として保存しない(06 §4.3) |
| 4 | `float` の `nan`・`inf`・`1.0` は、`int` ではない | ステートの JSON では整数の字句だけを整数とみなす(`1.0`・`1e0` は整数でない。06 §6.5)。`flow` の関数は `float` を受け取らず、待ち時間にも `float` を使わない |

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

型の骨格(`conductor/flow/types.py`。メモリ上の名前は snake_case で、ステートファイルのキー名と同じ。対応は [06](06-state-file.md) §2):

```python
from __future__ import annotations
from dataclasses import dataclass
from typing import Literal, TypeAlias

JobStatus: TypeAlias = Literal["pending", "success", "warning", "error", "skipped"]
SkipReason: TypeAlias = Literal["byTransition", "byEnd"]

@dataclass(frozen=True, slots=True)
class JobState:
    id: str
    status: JobStatus
    exit_code: int | None = None        # 結果のある状態だけ。無しもある
    skip_reason: SkipReason | None = None  # status == "skipped" のときだけ

@dataclass(frozen=True, slots=True)
class PipelineState:
    definition_key: str                 # 定義ファイルのパス(definition_path と同じ値)
    definition_path: str
    pipeline_name: str
    run_id: str | None
    job_order: tuple[str, ...]
    jobs: tuple[JobState, ...]          # job_order と同じ順・同じ数
    resume_job_id: str | None
    end_reached: bool
    completed: bool
```

### 1.2 1 回の実行結果(`JobExecution`)

ジョブを 1 回起動して終わるまでの観測結果。

| フィールド | 型 | 意味 |
|---|---|---|
| `job_id` | `str` | 定義のジョブID |
| `started` | `bool` | 起動した(子プロセスの生成に成功した)か。「中断の合図が起動の前に立っていたため起動しなかった」ときだけ `False`。ほかの起動失敗でも `started` は `True` のまま(`start_failure` を見る) |
| `exit_code` | `int \| None`(32 ビット符号付き整数) | プロセスの終了コード。取れないとき(シグナル終了、起動失敗)は `None` |
| `terminated_by_signal` | `int \| None` | ジョブがシグナルで終了したときの番号(UNIX 系のみ) |
| `start_failure` | `bool` | 起動に失敗した。command を引数に分けられない、プロセス生成の失敗、終了状態を取得できない、のいずれか |
| `interrupted` | `bool` | conductor がシグナルを受け取り、中断の合図が立っている(ジョブの終了を観測した直後に立っていた場合を含む) |
| `signal_number` | `int \| None` | `interrupted` のとき、最初に受け取ったシグナルの番号 |

```python
@dataclass(frozen=True, slots=True)
class JobExecution:
    job_id: str
    started: bool
    exit_code: int | None
    terminated_by_signal: int | None
    start_failure: bool
    interrupted: bool
    signal_number: int | None
```

ジョブの起動手順(`run_job`。子プロセスの API と詳細は [07](07-process-and-signals.md)):

1. command を引数に分ける(規則は 07)。失敗なら `start_failure = True`(`started = True`)で終わる。
2. 中断の合図が立っていれば、`started = False, interrupted = True, signal_number = 番号` で終わる(起動しない)。
3. 子プロセスを生成する(`subprocess.Popen`)。生成の失敗(`OSError` の系統)なら `start_failure = True` で終わる。
4. 終了まで待つ。待機中に合図が立てば、子プロセスに同じシグナルを**1 回だけ**転送して、終了を待ち続ける。
5. 終了コードとシグナル終了を記録する。`Popen.returncode` が負(`-n`)なら `terminated_by_signal = n`、`exit_code = None`(UNIX 系)。終了状態を取得できなければ `exit_code`・`terminated_by_signal` とも `None` のまま(この場合は §2.3 の順 5 に当たり、`conductor_error` つきのエラー(終了コード 1)になる)。
6. 終了を観測した直後に合図が立っていれば `interrupted = True, signal_number = 番号`。

**`exit_code` の範囲に関する注意(Windows。未確認)**: この仕様の `exit_code` は 32 ビット符号付き整数で、06 の読み込みはその範囲外を壊れたステートとして拒否する。Windows では、プロセスの終了コードが符号なし 32 ビット整数(0〜4294967295)として `Popen.returncode` に入る、と理解している(例: Ctrl+C による終了が 3221225786 になる、と広く報告されている)が、この環境では確かめていない。`runner` は、`returncode >= 2**31` のとき `returncode - 2**32` に直してから `JobExecution.exit_code` に入れる(下位 8 ビットは同じなので、プロセスの終了コードには影響しない)。直さずにステートへ書くと、次の読み込みで `corrupt` になる。UNIX 系の `returncode` は 0〜255 か負のシグナル番号で、範囲内(確認済みではないが、`waitpid` の仕様から)。

### 1.3 結果(`JobOutcome`)

| フィールド | 意味 |
|---|---|
| `classification` | `success` / `warning` / `error`、または**分類なし**(「起動しなかった中断」。型では `None`) |
| `reported_exit_code` | スケジューラへ返す終了コード(`int`)。後で 0〜255 に整える(§2.2) |
| `conductor_error` | 「成功コードに含まれない 0」など、conductor 自身が原因を標準エラーに表示するエラーのとき `True` |

```python
Classification: TypeAlias = Literal["success", "warning", "error"]

@dataclass(frozen=True, slots=True)
class JobOutcome:
    classification: Classification | None   # None = 分類なし
    reported_exit_code: int
    conductor_error: bool
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
| 1 | - | 成功コードに含まれない 0 で終わったジョブ(`conductor_error`)。終了コードを取れなかったエラーも 1 |
| 64 | DEFINITION | 定義の誤り(ファイルがない・読めない・検証の誤り・パイプライン名の不一致・command の誤り) |
| 65 | STATE | ステートの誤り(読めない・壊れている・版の不一致・定義との食い違い・書き込み/初期化/削除の失敗)、完了を判定できない、`watch` の出力失敗 |
| 66 | REJECTED | 拒否(順序外) |
| 67 | CONVERSION | convert の失敗(この文書の対象外) |
| 68 | USAGE | 使い方の誤り(知らないジョブID、run ID の値の誤りを含む) |
| 70 | INTERNAL | 内部の誤り(想定外の異常終了、シグナル処理の準備の失敗) |
| 128 + 番号 | - | ジョブがシグナルで終了した、または conductor が中断された(SIGINT = 130、SIGTERM = 143) |

64〜68・70 は、ジョブ自身の終了コードと重なりうる。区別は標準エラーの `conductor: error: ...` の行で行う。

固定の値は `enum.IntEnum`(`ExitCode`: `OK = 0`、`DEFINITION = 64`、`STATE = 65`、`REJECTED = 66`、`CONVERSION = 67`、`USAGE = 68`、`INTERNAL = 70`)と、定数 `START_FAILURE_EXIT = 127` で持ってよい。`sys.exit` に `IntEnum` を渡すと、その整数が終了コードになる(確認済み: `ExitCode.STATE` で 65)。ただし、`flow` が返す値と `cli` が `SystemExit` に渡す値は、`int(...)` を通した素の `int` にそろえる。

### 2.2 `exit_code_for(reported, is_error)`: 0〜255 への整形

プロセスの終了コードは 0〜255 に収める。`sys.exit` に 256 以上や負数を渡したときの結果は OS 任せなので(確認済み(macOS): `sys.exit(256)` は 0、`sys.exit(-1)` は 255、`sys.exit(300)` は 44)、`SystemExit` に渡す前に必ず整形する。

```text
if 0 <= reported <= 255:  return reported
low = reported & 0xFF              # 負数も含め、2 の補数表現の下位 8 ビット(int の & は負数で正しく動く)
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

(この表は Python で実際に計算して確認済み: 3、1、0、1、255、255、1。)

`is_error` は `classification == "error"`。分類なし(起動しなかった中断)と成功・警告は `False`。

中断の終了コード `signal_exit_code(n)` = `exit_code_for(sat_add_i32(128, n), True)`(パイプラインのジョブのあいだの中断で使う)。

「32 ビット符号付き整数の飽和加算」(`sat_add_i32(a, b)` = `max(-2**31, min(2**31 - 1, a + b))`): 結果が 2147483647 を超えるなら 2147483647、-2147483648 を下回るなら -2147483648 にする(例: 128 + 2147483647 = 2147483647。確認済み)。実在のシグナル番号(1〜64 程度)では起きない。`int` は桁あふれしないので、足してから範囲に収める。

### 2.3 成功・警告・エラーの分類と返す値(`classify`)

- **成功コードの集合** = 定義の `success_exit_codes` があればその値だけ(0 を書かなければ 0 でも成功にならない)、無ければ `[0]`。
- **警告コードの集合** = `warning_exit_codes`(0 を含まず、成功コードと重ならないことは定義の検証で保証済み)。

判定は次の順に、最初に当てはまるものを採る(上ほど優先)。入力は 1 回の実行結果(§1.2)とジョブ定義。

| 順 | 条件 | classification | reported_exit_code | conductor_error |
|---|---|---|---|---|
| 1 | `started = False` かつ `interrupted` | なし | `128 + signal_number`(無ければ `128 + 0`)、飽和加算 | false |
| 1' | `started = False` かつ `interrupted` でない(あり得ない組み合わせ) | error | 127 | false |
| 2 | `interrupted`(起動済み) | error | `128 + signal_number`(無ければ 128)。ジョブ自身の終了コードは見ない | false |
| 3 | `start_failure` | error | 127 | false |
| 4 | `terminated_by_signal` が有り(値 n) | error | `128 + n` | false |
| 5 | `exit_code` が無い(起動して終わったのに終了コードがない) | error | 1 | **true** |
| 6 | `exit_code` ∈ 成功コード | success | 0 | false |
| 7 | `exit_code` ∈ 警告コード | warning | 0 | false |
| 8 | `exit_code = 0`(成功コードに無い) | error | 1 | **true** |
| 9 | それ以外 | error | `exit_code` そのまま(負数も、255 超もそのまま保持。整形は §2.2) | false |

注意(Python): 順 5 の「無い」は `exit_code is None` で判定する。`if not exit_code` と書くと、`0` が「無い」になり、順 6〜8 に進まない(誤り)。同様に、`terminated_by_signal` も `is not None` で判定する(`0` というシグナル番号は無いが、真偽値で判定しない習慣を守る)。「成功コードの集合に含まれる」は `exit_code in job.success_exit_codes`(`tuple` の `in`。値の比較)でよい。

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

- `conductor_error = True`: `conductor: error: job '<id>': exit code 0 is not listed in success_exit_codes`(日本語ロケールでは日本語。文言は [08-output-and-i18n.md](08-output-and-i18n.md))。`conductor_error = True` なら、順 5(終了コードが無い)の場合も同じ文言を出す(実装は `conductor_error` が真なら、理由を区別せず同じ行を出す)。
- 起動失敗: `conductor: error: job '<id>': cannot start command '<program>'`。

---

## 3. 遷移と次の動き(`classify_and_decide` / `next_step`)

ジョブ実行後に、分類と次の動き(`NextStep`)を決める。

### 3.1 `NextStep` のフィールド

| フィールド | 意味 |
|---|---|
| `next_job_id` | 次に実行するジョブ。完了・停止のとき `None`(`pipeline` コマンドは、この値を使わない。§5.3) |
| `skipped_job_ids` | この時点でスキップとして記録するジョブ(`tuple[str, ...]`) |
| `skip_reason_kind` | その理由(`byTransition` / `byEnd` / `None`) |
| `end_reached` | `@end` に到達した |
| `completed` | 完了した |
| `resume_job_id` | エラー等で停止したときの再開位置(そのジョブ)。`None` もある |

### 3.2 判定表

入力: 分類、そのジョブの該当する遷移(`on = success` は成功のとき、`on = warning` は警告のときに参照する。`error` の遷移は書けない)、`RunMode`、そのジョブの位置(index。定義の記述順、0 始まり)、各ジョブの現在の状態。

「未完了のジョブ」= 指定した区間のジョブのうち、状態が success / warning / skipped **でない**もの(pending・error・記録なし)。区間は、`@end` なら「index より後ろ全部」、遷移先 `T` なら「index と T のあいだ(両端を含まない)」。

| 分類 | 遷移 | RunMode | next_job_id | skipped_job_ids / kind | end_reached | completed | resume_job_id |
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
- individual の `@end` では、後ろのジョブを今は記録せず(`skipped_job_ids` は空)、`end_reached = True` だけを記録する。後ろのジョブは、スケジューラが 1 つずつ `run` を呼んだときに `Skip(AfterEnd)` が返り、呼ばれたジョブだけ `skipped(byEnd)` と記録される(§4.2、§5.4)。
- 遷移先 `T` の遷移では、`T` へ飛ぶ。`T` が index+1 なら間は空で、スキップは無い。
- 自動のリトライ(ジョブ失敗の再実行)は、`retry` を書かない限りしない(§7)。

### 3.3 `apply_outcome`(結果の記録。メモリ上)

入力: 現在のステート値 `S`、ジョブID、実行結果、結果(`JobOutcome`)、`NextStep`。`S` を複製して更新し、返す。

1. 対象ジョブ(`S` のジョブ状態一覧から id で探す。見つからなければこの手順 1 を飛ばす):
   - `classification` があれば `status` を success / warning / error にし、`skip_reason` を消す。分類なし(起動前の中断)なら `status` は**変えない**。
   - `exit_code` = 実行結果の `exit_code`(無ければ、以前の値も消す)。そのあと `status` が success / warning / error のいずれでもなければ `exit_code` を消す。
2. `skipped_job_ids` の各ジョブ(対象ジョブ自身は除く)を `status = skipped`、`exit_code` 無し、`skip_reason = skip_reason_kind`(無ければ `byTransition`)にする。
3. `resume_job_id = NextStep.resume_job_id`、`completed = NextStep.completed`。
4. `end_reached = S.end_reached or NextStep.end_reached`(一度 true になったら戻さない)。

### 3.4 `apply_skip`(スキップの記録。メモリ上)

入力: ステート値 `S`、ジョブID、判断の理由(`DecisionReason`: `AlreadyExecuted` / `SkipTarget` / `AfterEnd` / `OutOfOrder`)、定義。

- 理由が `AlreadyExecuted` / `OutOfOrder` → 何も変えない(複製を返す)。
- `SkipTarget` → そのジョブを `skipped(byTransition)`、`AfterEnd` → `skipped(byEnd)` にし(`exit_code` 無し)、`AfterEnd` なら `end_reached = True`。ジョブが `S` に無ければ、更新せず複製を返す。
- そのあと、`S` に `error` のジョブが 1 つでもあれば、`resume_job_id` と `completed` は**変えない**(エラーの再開位置を守る)。無ければ:
  - `first_pending` = 定義の記述順で最初の `pending`(記録なしを含む)のジョブ(無ければ無し)
  - `completed` = 「いまスキップしたジョブが定義の最後のジョブ、かつ `first_pending` が無い」
  - `resume_job_id = first_pending`

### 3.5 `apply_interrupt`(中断の記録。メモリ上)

再開位置(§9)の値で更新する: `resume_job_id = ResumePoint.resume_job_id`、`completed = ResumePoint.completed`、`end_reached = S.end_reached or ResumePoint.end_reached`。ジョブの状態は変えない(エラーとして記録しない)。

---

## 4. 起動前の判断(`decide_before_run`)

### 4.1 入力と出力

入力: ジョブID、定義、各ジョブの状態の一覧、`resume_job_id`(ステートの同名の値)、`end_reached`。ステートが空(ファイルなし・長さ 0・完了のまま残ったもの)のときは、状態一覧は空、`resume_job_id = None`、`end_reached = False` を渡す。

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

「等しい」はステート値の全項目の比較(§0.2: dataclass の `==`)。「耐久性あり(Durable)」「省く(Fast)」の定義は [06-state-file.md](06-state-file.md) の書き込み。

---

## 5. 実行の流れ

### 5.1 共通の前処理 `prepare`(`run`・`pipeline`)

順序は固定。最初に誤りがあった時点で、その終了コードで終わる(以降は行わない)。

| 順 | 処理 | 誤りのときの終了コード |
|---|---|---|
| 1 | 中断の合図(「立った/立っていない」と最初のシグナル番号を保持する共有のフラグ)を作り、シグナルのハンドラを登録(定義の検査より前)。`signal.signal` はメインスレッドからだけ呼べる(それ以外では `ValueError`)。登録は、すでに別のハンドラが入っていても置き換える(Python に「登録済みなら無視」はない)。終了時に元のハンドラへ戻す([07](07-process-and-signals.md)) | 準備の失敗(`signal.signal` が `ValueError` / `OSError` を出した): 70(`conductor: error: failed to set up signal handling`) |
| 2 | 定義ファイルの読み込みと検証。`pipeline` は、引数の名前と定義の `pipeline.name` を照合 | 64 |
| 3 | 全ジョブの `command` の検査(引用符の閉じ忘れ・空) | 64 |
| 4 | `run` のとき、ジョブIDが定義にあるか | 68(`unknown job`) |
| 5 | run ID の決定(§6) | 68 |
| 6 | ステートの読み込み([06-state-file.md](06-state-file.md) §6)。完了のまま残ったものは「空」 | 65 |
| 7 | 空でなければ、定義との照合(06 §7) | 65 |
| 8 | 実効の `remove_on_complete` = `--remove-on-complete` があれば True、なければ定義の `pipeline.remove_on_complete`(省略時 False)。環境変数は見ない | - |

- 定義ファイルは `--config <パス>`、なければ実行ディレクトリの `conductor.yaml`。
- シグナルを受け取っても、1〜7 は最後まで進む(ハンドラはフラグを立てるだけで、処理を中断しない)。64・65・66・スキップ(0)の終了コードが、中断の終了コードに優先する。
- `pipeline` で引数の名前が定義の名前と違う場合は 64(順 2)。引数の名前が無いのは使い方の誤り 68(コマンドライン解析。[04-cli.md](04-cli.md))。

### 5.2 `conductor run <job-id>` の擬似アルゴリズム

```text
session = prepare(...)                      # 5.1。誤りなら終了
(states, resume, end_reached) = session.state の jobs / resume_job_id / end_reached
                                (state が空なら (), None, False)
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
    (outcome, next) = classify_and_decide(job, exec, base.jobs, def, "individual")
    表示(§5.5)
    rec = apply_outcome(base, job_id, exec, outcome, next)
    write(rec, Durable)                      # 失敗: 65、文脈=exec.started なら「ジョブは実行されました」、そうでなければ「起動していません」
    if next.completed and remove_on_complete:
        reset_state()                        # 失敗: 65
    return exit_code_for(outcome.reported_exit_code, outcome.classification == "error")
```

注意:

- individual では、遷移でスキップが決まったジョブ(`byTransition`)は、`apply_outcome` で記録済みなので、後で `run` されると `Skip(SkipTarget)`。
- 最後のジョブの `run` 完了、または `@end` 後の最後のジョブのスキップ記録で `completed = True`。
- 初期ステートは `Run` と決まった後にだけ書く(Reject・Skip の前には書かない)。ただし Skip のときは、ステートが空なら `initial_state`(06 §2.4)を土台に記録が書かれる。
- `classify_and_decide(job, exec, states, def, mode)` は、`classify`(§2.3)で結果を作り、その分類を使って `next_step`(§3.2)で次の動きを決めて、組で返す。

### 5.3 `conductor pipeline <name>` の擬似アルゴリズム

`pipeline` は、「次のジョブ」を `NextStep.next_job_id` で決めない。定義の記述順に全ジョブを 1 つずつ見て、`decide_before_run` で決める(遷移先・スキップは、ステートが運ぶ)。

```text
session = prepare(...)                       # 引数の名前が無ければ 68
current = session.state                      # 空なら None
for job in def.jobs (記述順):
  d = decide_before_run(job.id, def, current の状態一覧 (None なら ()), current.resume_job_id, current.end_reached)
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
      base = current、None なら initial_state を Fast で書いたもの   # 失敗: 65
      exec = run_job_with_retry(job)
      (outcome, next) = classify_and_decide(job, exec, base.jobs, def, "pipeline")
      表示
      rec = apply_outcome(base, job.id, exec, outcome, next)
      write(rec, Durable)                    # 失敗: 65
      if outcome は error、または分類なし:
          return exit_code_for(outcome.reported_exit_code, outcome.classification == "error")   # エラーで停止
      if next.completed: return finish(...)
      if 中断の合図が立っている(番号 n):      # ジョブのあいだの中断
          point = resume_point_on_interrupt(def, rec.jobs)
          write(apply_interrupt(rec, point), Durable)     # 失敗: 65
          -v なら「interrupted by signal n」
          return signal_exit_code(n)           # exit_code_for(sat_add_i32(128, n), True)
      current = rec
# ループを抜けた(完了していない。起こらない想定)
表示 "cannot determine whether the pipeline is complete"; return 65   # ステートは初期化しない

finish: if not remove_on_complete: return 0
        reset_state() が成功 → 0、失敗 → 65
```

- 完了のステートは、`finish` の前に必ず Durable で書き込み済み。`remove_on_complete` が真のときだけ削除する(06 §9)。
- 「完了を判定できない」は、全ジョブが `AlreadyExecuted` などで通過し、どこでも `completed` が真にならなかったとき。標準エラー `conductor: error: <ステートファイルのパス>: cannot determine whether the pipeline is complete`(日本語: `パイプラインが完了したかを判定できません`)、終了コード 65。
- ジョブ実行後、`next.completed` の判定は、中断の合図の判定より**先**。完了していれば中断の記録はせず、`finish` へ進む。
- エラーのジョブで止まったときの再実行: 同じコマンドを再度呼ぶと、`success`/`warning` のジョブは `Skip(AlreadyExecuted)`(`-v` に `already executed`)、`error` のジョブから `Run`。
- `Skip` で `rec.completed` が真になるのは §3.4 の条件(スキップしたジョブが最後のジョブで、pending が無い)のとき。
- ループの途中で `current` を更新するとき、`current` は不変の値なので、`rec` に付け替えるだけでよい(コピーの共有で問題が起きない)。

#### 状態遷移の例(`pipeline`)

定義 `a → b → c`(遷移なし)。`b` が exit 3 でエラー。

| 時点 | jobs | resume_job_id | end_reached | completed | 終了コード |
|---|---|---|---|---|---|
| 初期(Fast で書く) | a=pending, b=pending, c=pending | なし | false | false | - |
| a 成功後 | a=success(0), b=pending, c=pending | なし | false | false | - |
| b エラー後 | a=success(0), b=error(3), c=pending | b | false | false | 3 |
| 再実行: a は Skip(AlreadyExecuted)、b 成功 | a=success, b=success(0), c=pending | なし | false | false | - |
| c 成功後 | 全部 success | なし | false | true | 0 |

定義 `a --success--> @end` のとき(pipeline): a 成功で b, c は `skipped(byEnd)`、`end_reached = True`、`completed = True`、0 で終わる([06-state-file.md](06-state-file.md) §4.4 の例 2 のステート)。

### 5.4 `@end` 後の `run`(individual)の流れ

`run a`(遷移 `@end`)→ `a=success`、`end_reached = True`、`completed = False`(後ろに未完了がある場合)。続けて `run b` → 規則 2 で `Skip(AfterEnd)` → `b=skipped(byEnd)`、`resume_job_id` = 最初の pending(`c`)。`run c` → `Skip(AfterEnd)` → `c=skipped(byEnd)`、最後のジョブで pending が無いので `completed = True`(`remove_on_complete` なら削除)。`@end` の後ろに未完了が無ければ、`run a` の時点で `completed = True`。

### 5.5 表示(`-v` と常時)

- 常に標準エラー: エラー行(`conductor: error: ...`)、起動失敗、成功コードに無い 0、リトライの知らせ(§7.9)、`reset --all` の警告。
- `-v` のときだけ標準エラー(先頭 `[conductor] `): `job=<id> result=<success|warning|error|interrupted> (...)`、`transition: <success|warning> -> <target>[ (skipped: <id>, ...)]`、`job=<id> skipped (<理由>)`、`interrupted by signal <n>`。括弧内の詳細の厳密な文言は [08-output-and-i18n.md](08-output-and-i18n.md) に従う。
- ジョブの結果の行と遷移の行は、**最終の結果について 1 回**だけ(リトライの途中の結果は出さない)。
- 言語: 環境変数 `LC_ALL`、`LC_MESSAGES`、`LANG` の順で、最初の空でない値が `ja` で始まれば日本語、それ以外は英語(どれも空・未設定なら英語)。トークン(`success` `warning` `error` `skipped` `pending` `job=` `result=`)は常に英語。
- 標準エラーへの書き込みは、行ごとに `flush` する(ジョブの出力との順序を保つため。[08](08-output-and-i18n.md))。

---

## 6. run ID

run ID は、同じ定義を別々の実行単位で並行して使うときに、ステートファイルを分ける識別子。

### 6.1 値の規則(`parse_run_id`)

次の順に検査し、最初に当てはまる誤りを返す(誤りは使い方の誤り 68。誤りの文に値は含めない)。

| 順 | 誤り | 条件 |
|---|---|---|
| 1 | `Empty` | 長さ 0 |
| 2 | `TooLong` | UTF-8 で符号化したバイト数が 64 を超える |
| 3 | `InvalidChar` | `A-Za-z0-9._-` 以外の文字がある |
| 4 | `LeadingDot` | 先頭が `.`(`..` を含む) |

Python での注意:

- 順 2 の長さは、`len(value)`(コードポイントの数)ではなく、UTF-8 で符号化したバイト数で数える。`"あ" * 30` は `len` が 30 だが UTF-8 では 90 バイトで、`TooLong` になる(確認済み: バイト数 90、`len` 30)。`len(value)` を使うと、順 2 と順 3 の判定が入れ替わる(`InvalidChar` になる)。許す文字は ASCII だけなので、検査を通る値では文字数とバイト数は同じ。
- バイト数の数え方: `len(value.encode("utf-8", "surrogateescape"))`。POSIX では、環境変数・引数の不正なバイトは、OS から `surrogateescape` で U+DC80〜U+DCFF の孤立サロゲートになって入る(確認済み: `CONDUCTOR_RUN_ID` に 2 バイト `FF FE` を入れると `'\udcff\udcfe'` になり、`surrogateescape` で符号化するとちょうど 2 バイト)。`surrogateescape` で符号化すると、その 1 文字が元の 1 バイトに戻るので、OS の元のバイト数になる。それ以外の孤立サロゲート(U+D800〜U+DB7F など。Windows の環境変数にはありうる、と理解している。未確認)は `surrogateescape` で `UnicodeEncodeError` になるので、その場合は `value.encode("utf-8", "surrogatepass")`(1 文字 3 バイト)で数える(確認済み: `"\ud800"` は `surrogatepass` で 3 バイト)。どちらの数え方でも、64 を超えれば `TooLong`、超えなければ順 3 で `InvalidChar` になり、利用者に見える結果(68)は同じ(誤りの文に値と種類は含めない)。
- 順 3 は、`re.fullmatch(r"[A-Za-z0-9._-]+", value)` で判定する。次の点を守る(いずれも確認済み)。
  - **`re.match(r"^...$", value)` は使わない**: Python の `$` は文字列の末尾の改行の手前にも一致するので、`"a\n"` が一致してしまう(確認済み: `re.match(r"^[A-Za-z0-9._-]+$", "a\n")` は一致、`re.fullmatch` は不一致)。
  - **`\d`・`\w`・`str.isalnum()`・`str.isdigit()` は使わない**: Unicode の数字・文字(全角の `１`、アラビア・インドの `٣`)も真になる(確認済み: `re.fullmatch(r"\d+", "１")` は一致、`"１".isalnum()` は真)。文字クラスは ASCII の範囲を明示する。
  - **`re.IGNORECASE` を付けない**: 付けると、ケルビン記号 U+212A が `[a-z]` に一致する(確認済み: `re.fullmatch(r"[a-z]+", "K", re.I)` は一致)。
- 空は順 1 で除かれている。

有効: `1`, `r1`, `12345`, `build-2026.11_a`, `a.b`, 64 文字ちょうど。無効: `""`(ただし §6.2 の規則で空は「なし」)、65 文字、`a/b`、`a b`、`.x`、`..`、`日本語`(30 文字でも `TooLong`)、`a\n`。

### 6.2 決定(`resolve_run_id`、`prepare` の 5)

```text
env_name = 定義の pipeline.run_id_env があればそれ、なければ "CONDUCTOR_RUN_ID"
if --run-id が指定された(値は空でもよい):
    value = そのオプションの値(subject = "--run-id")      # 環境変数は見ない
else:
    value = os.environ.get(env_name)(未定義なら None)(subject = "run ID (environment variable)")
if value が None または "": run ID なし
else: parse_run_id(value) が失敗なら 68(`invalid value: '<subject>'`)、成功なら run ID
```

- `--run-id ""` は、環境変数があっても「run ID なし」に固定する。
- `run_id_env` の形は定義の検証で保証される(`A-Za-z_` で始まり `A-Za-z0-9_` が続く 1 文字以上)。
- 環境変数名は定義で決まるので、定義の検査(64)の**後**、ステートの読み込み(65)の**前**に run ID を決める。
- `watch`・`reset` は定義を検証しない: `--run-id` があれば定義を読まず(環境変数名は既定)、なければ定義の `pipeline.run_id_env` だけをゆるく読む。読めない・キーが無い・文字列でない・名前の形が違う、のどれでも「なし」= 既定の名前を使う。ほかの誤りは無視する。異常終了してはならない(定義の読み込みが投げた例外は、この場面では握りつぶして既定の名前にする)。
- 環境変数の値が規則に合わなければ、`run`・`pipeline`・`watch`・`reset`(`--all` を除く)で 68。`reset --all` は環境変数を見ない。
- 環境変数は、`Deps` の環境(`os.environ` のコピーか読み取り関数)から読む。POSIX では名前の大文字小文字を区別する。Windows の `os.environ` は、名前を大文字にそろえて持つ、と文書にある(この仕様は確認していない。未確認)ので、`run_id_env` の大文字小文字の違いは Windows では同じ変数を指す。
- 環境変数が「空文字列で定義されている」場合は、「なし」として扱う(上の疑似コードの `""`。確認済み: `os.environ.get` は `''` を返し、未定義は `None`)。

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
| `interval` | 文字列 `<整数><s\|m\|h>`。結果が 1 秒〜24 時間(86400 秒) | 1 回目のリトライの前の待ち時間。数字は半角の数字だけで、整数は 1 以上。単位は小文字 1 文字(`s` 秒、`m` 分 = 60 秒、`h` 時間 = 3600 秒)。例 `"30s"`、`"2m"`、`"1h"` |
| `multiplier` | 整数 2〜10 | 待ち時間にかける倍率。`max_interval` と同時に指定 |
| `max_interval` | `interval` と同じ書式。`interval` 以上 | 待ち時間の上限。`multiplier` と同時に指定 |

- `multiplier` と `max_interval` は両方書くか両方書かない。検証の詳細は [03-definition-format.md](03-definition-format.md)。
- `retry` を書かないジョブは、1 回だけ実行する。

### 7.2 数値の型

§0.2 の方針を、リトライに当てはめる。

| 値 | 型 | 備考 |
|---|---|---|
| 待ち時間(秒。`interval_secs`、`max_interval_secs`、各回の待ち、`last_wait_secs`) | `int`(0〜2**64-1) | 計算は秒の `int` で完結し、浮動小数点への変換は待機の直前の 1 か所だけ(§7.3、§7.8)。ナノ秒への変換も `int` の乗算 |
| `multiplier` | `int`(2〜10) | `sat_mul_u64` にそのまま渡す |
| `max`、`retries_done`、知らせの回数 `n` | `int` | `max` は 1〜100 |
| `exit_codes` の各値 | `int`(1〜255) | ジョブの終了コードの型と同じ |

浮動小数点は使わない。丸めは無い(整数の秒だけ)。`/`(真の除算)は `float` を返すので、待ち時間の計算に使わない(整数の除算が要る場合は `//`。このツールの待ち時間の計算には除算は出てこない)。`bool` を渡さない(§0.3)。

`RetryPolicy` の骨格(検証済みの定義から作る。`flow` が受け取る形):

```python
@dataclass(frozen=True, slots=True)
class Backoff:
    multiplier: int          # 2〜10
    max_interval_secs: int   # interval_secs 以上、86400 以下

@dataclass(frozen=True, slots=True)
class RetryPolicy:
    exit_codes: tuple[int, ...]   # 昇順・重複なし・各 1〜255・空でない
    max: int                      # 1〜100
    interval_secs: int            # 1〜86400
    backoff: Backoff | None = None  # None なら固定間隔
```

`Backoff` を 1 つの値にまとめるのは、「`multiplier` と `max_interval` は両方あるか両方ない」を型で保証するための決定。定義の検証で片方だけは拒否される([03](03-definition-format.md))ので、部分的に存在する状態は作れない。

### 7.3 `interval` / `max_interval` の文字列から秒への変換と、待機への変換

文字列 → 秒(`parse_duration_secs(s: str) -> int | None`。定義の検証で使う。置き場所は `conductor/definition/`。ここでは Python の落とし穴を示すために再掲する):

```text
1. s の長さ(コードポイントの数)が 2 未満なら 無し(拒否)
2. 最後の 1 文字が 's' / 'm' / 'h' のどれでもなければ 拒否
3. 残り(digits)が、re.fullmatch(r"[0-9]+", digits) に一致しなければ 拒否
   (符号・空白・小数点・'_'・全角数字・"0x" は、ここで弾かれる。`int()` の解釈には頼らず、先に正規表現で検査する)
4. digits の先頭の '0' を取り除く(すべて '0' なら "0" を残す)。取り除いたあと:
     - 長さが 20 を超えるなら 拒否(2**64-1 は 20 桁。86400 秒を超えるので、どのみち拒否される)
     - n = int(digits)
5. 単位ごとの上限で、乗算の前に比べる:
     's': 1 <= n <= 86400
     'm': 1 <= n <= 1440
     'h': 1 <= n <= 24
   外れたら 拒否
6. secs = n * (1 / 60 / 3600)   # 単位ごと。整数の乗算
```

Python の落とし穴(いずれも確認済み):

- **`int()` は寛容**: `int("１")`(全角)は 1、`int("1_0")` は 10、`int(" 1")` は 1、`int("+1")` は 1、`int("1\n")` は 1、`int("٣")` は 3(3.9 と 3.14 で同じ)。手順 3 の正規表現を先に通さないと、`"１s"`・`"1_0s"`・`" 1s"` が受理されてしまう。
- **`\d` は Unicode の数字に一致する**(`re.fullmatch(r"\d+", "１")` は一致)ので、`[0-9]` と書く。
- **`$` は末尾の改行の手前に一致する**ので、`re.fullmatch` を使う(`re.match(r"[0-9]+$", "1\n")` は一致する)。
- **3.11 以降、`int()` は 4300 桁を超える文字列で `ValueError`**(確認済み(3.14): `int("1" * 5000)` が `Exceeds the limit (4300 digits) for integer string conversion`)。手順 4 の「20 桁を超えたら拒否」が、`int()` を呼ぶ前に働くので、この例外は出ない。

例: `"1s"`→1、`"01s"`→1、`"0000000000000000000000000001s"`→1、`"1440m"`→86400、`"24h"`→86400、`"86400s"`→86400。拒否: `""`、`"s"`、`"10"`、`"1.5s"`、`"-1s"`、`"+1s"`、`" 1s"`、`"1s "`、`"1S"`、`"１s"`(全角 1)、`"1ms"`、`"0x10s"`、`"1_0s"`、`"0s"`、`"0h"`、`"86401s"`、`"25h"`、`"99999999999999999999999s"`、`"18446744073709551615h"`、`"1s\n"`。

秒 → 待機(待機の直前だけ。§7.8):

```text
deadline_ns = start_ns + secs * 1_000_000_000       # int。桁あふれしない
# 1 回の time.sleep に渡す秒は、残りと 50 ミリ秒の小さい方だけ(浮動小数点は、この 1 か所だけ)
step_seconds = min(deadline_ns - now_ns, 50_000_000) / 1e9
```

- **長い値を `time.sleep` に渡さない**: `time.sleep` の引数の上限は版と OS で違い、超えると例外になる。確認済み(macOS): CPython 3.9.6 は 2147483648 秒で `OSError: [Errno 22] Invalid argument`、9300000000 秒で `OverflowError: timestamp too large to convert to C _PyTime_t`。3.14.10 は 9223372036.85 秒以上で `OverflowError: timestamp out of range for C PyTime_t`(2147483648 秒はそのまま待ちに入る)。`threading.Event.wait(timeout)` も同じ範囲で `OverflowError`(3.9 の 2**31 は待ちに入る)。したがって、**1 回に渡す値は常に 50 ミリ秒以下の秒**とし、全体の待ちは `int` のナノ秒の期限との比較で繰り返す(§7.8)。この規定により、`secs` が 2**64-1 でも例外にならず、「短くならない」(待機は事実上終わらないが、期限の計算は桁あふれしない)。この入力は検証済みの定義(86400 秒以下)では起きない。
- 検証済みの定義では、待ち時間は最大 86400 秒 = 86,400,000,000,000 ナノ秒で、`int` の演算に何の問題もない。

### 7.4 リトライの対象か(`is_retry_target`)

次の**すべて**が成り立つときだけ真。

1. `started = True`
2. `interrupted = False`
3. `start_failure = False`
4. `terminated_by_signal` が無い(`is None`)
5. `exit_code` があり(`is not None`)、`exit_codes` に含まれる

分類(成功・警告・エラー)は見ない。対象の終了コードは成功・警告と重ならない(検証済み)ので、結果として対象はエラーの終了コードだけ。ジョブ自身が 127 で終わった場合は、`exit_codes` に 127 を書いていれば対象(conductor が付けた起動失敗の 127 は、`exit_code` を持たないので対象にならない)。

### 7.5 判断(`retry_decision(job, execution, retries_done, last_wait_secs)`)

```text
policy = job.retry か、無ければ return Stop
if not is_retry_target(policy, execution) or retries_done >= policy.max: return Stop
return Retry(wait_secs = next_wait_secs(policy, retries_done, last_wait_secs))
```

```python
@dataclass(frozen=True, slots=True)
class Stop:
    kind: Literal["stop"] = "stop"

@dataclass(frozen=True, slots=True)
class Retry:
    wait_secs: int
    kind: Literal["retry"] = "retry"

RetryDecision: TypeAlias = Stop | Retry
```

### 7.6 待ち時間の式

```text
next_wait_secs(policy, retries_done: int, last_wait_secs: int) -> int
  if policy.backoff が有る:
      if retries_done == 0: return min(policy.interval_secs, policy.backoff.max_interval_secs)   # = interval_secs(検証で interval <= max_interval)
      else:                 return min(sat_mul_u64(last_wait_secs, policy.backoff.multiplier), policy.backoff.max_interval_secs)
  else:
      return policy.interval_secs                                                                 # 毎回同じ(固定間隔)
```

- k 回目のリトライ(k = `retries_done + 1`)の前の待ち時間を `W(k)` とすると: 固定間隔なら `W(k) = interval`。倍率ありなら `W(1) = interval`、`W(k) = min(W(k-1) × multiplier, max_interval)`(k ≥ 2)。
- 倍率の掛け算は、直前の**実際の**待ち時間(上限で頭打ちされた後の値)に対して行う。
- `min` は組み込みの `min`(`int` 同士の比較。Python では `int` と `float` が混ざっても動くが、混ぜない)。

`sat_mul_u64(a, b)`(64 ビット符号なし整数の飽和乗算):

```text
U64_MAX = 2**64 - 1                       # 18446744073709551615
return min(a * b, U64_MAX)                # int は桁あふれしないので、掛けてから上限で止める
```

`a × b` が 2^64 - 1(18446744073709551615)を超えるなら 18446744073709551615、そうでなければ `a × b`。**折り返し(`(a * b) & (2**64 - 1)`)にしてはならない**(確認済み: `((2**64 - 1) * 10) & (2**64 - 1)` は `18446744073709551606`)。前提として `a` は 0 以上 `U64_MAX` 以下(呼び出し側の保証。定義の検証と、`next_wait_secs` の出力が `max_interval_secs` 以下であることから成り立つ)。

境界の例(関数単体の挙動。いずれも Python で計算して確認済み):

| a | b | 結果 | 理由 |
|---|---|---|---|
| 0 | 10 | 0 | |
| 86400 | 10 | 864000 | 検証済みの定義で起こりうる最大の積 |
| 1844674407370955161 | 10 | 18446744073709551610 | 飽和しない最大に近い値(`2**64-1` を 10 で割って切り捨てた値 × 10) |
| 1844674407370955162 | 10 | 18446744073709551615 | 積 `18446744073709551620` が `2**64-1` を超えるので飽和 |
| 18446744073709551615 | 2 | 18446744073709551615 | 飽和(積は `36893488147419103230`) |
| 9223372036854775807 | 10 | 18446744073709551615 | 飽和(積は `92233720368547758070`) |
| 9223372036854775808(2**63) | 2 | 18446744073709551615 | 積 `2**64` が上限を 1 超えるので飽和 |

`next_wait_secs` の境界例(関数単体。検証済みの定義では起きない入力。確認済み):

| interval_secs | multiplier | max_interval_secs | retries_done | last_wait_secs | 結果 |
|---|---|---|---|---|---|
| 10 | 2 | 60 | 0 | 0 | 10 |
| 10 | 2 | 60 | 3 | 40 | 60(80 を 60 で頭打ち) |
| 10 | 2 | 60 | 4 | 60 | 60 |
| 1 | 10 | 18446744073709551615 | 1 | 9223372036854775807 | 18446744073709551615(積が飽和し、上限で止まる) |
| 1 | 10 | 18446744073709551615 | 1 | 1844674407370955161 | 18446744073709551610(飽和しない最大) |
| 1 | 10 | 18446744073709551615 | 1 | 1844674407370955162 | 18446744073709551615 |
| 30 | (backoff なし) | - | 5 | 30 | 30 |

検証済みの定義では `interval` と `max_interval` が 86400 秒以下なので、`sat_mul_u64` が飽和する場面は無い(最大でも 86400 × 10)。再実装でこの極端な入力まで再現する必要は実用上ないが、乗算は折り返さないこと。

待ち時間の列の例(Python で計算して確認済み):

| interval | multiplier | max_interval | max | 待ち時間の列(秒) | 合計 |
|---|---|---|---|---|---|
| 30s | なし | なし | 3 | 30, 30, 30 | 90 |
| 10s | 2 | 60s | 5 | 10, 20, 40, 60, 60 | 190 |
| 60s | 3 | 60s | 3 | 60, 60, 60 | 180 |
| 1s | 10 | 3600s(1h) | 6 | 1, 10, 100, 1000, 3600, 3600 | 8311 |
| 1m | 2 | 24h | 4 | 60, 120, 240, 480 | 900 |
| 1h | 10 | 24h | 4 | 3600, 36000, 86400, 86400 | 212400 |

### 7.7 実行ループ(`run_job_with_retry`)。`run` と `pipeline` の共通

```text
retries_done = 0; last_wait_secs = 0
loop:
    exec = run_job(job, sig)
    match retry_decision(job, exec, retries_done, last_wait_secs):
        Stop: return exec                          # 最後の結果
        Retry(wait_secs):
            表示: 常に 1 行(§7.9)。retry_number = retries_done + 1
            match wait_retry(wait_secs, sig):       # Deps の待機の差し替え点。本番は §7.8
                Elapsed:          retries_done += 1; last_wait_secs = wait_secs; continue
                Interrupted(n):   return JobExecution(job_id=job.id, started=False, interrupted=True, signal_number=n,
                                                       exit_code=None, start_failure=False, terminated_by_signal=None)
```

- 返された最後の結果だけが、分類・表示・ステート記録の対象(ステートは最終の結果で 1 回だけ書かれ、途中では書かれない)。リトライの回数・直前の待ち時間はローカルな値で、ステートファイルに書かない。
- したがって、conductor が落ちたあとの再開では、そのジョブを最初の実行から数え直す。
- 使い切った(`retries_done == max` で、まだ対象の終了コード): `Stop` → 通常のエラーと同じ(最後の実行の終了コードで終わり、再開位置はそのジョブ)。再実行の結果が成功・警告なら通常どおり遷移へ進む。
- 総実行回数: 最大 `max + 1`。知らせの回数: 実際に実行し直す(または待機に入る)回数。
- 起動前に中断された(合図が立っていた)ジョブはリトライせず、その場で `started = False` の結果を返す(`retry_decision` は `started = False` で Stop)。

### 7.8 待機とシグナルによる中断(`wait_retry`)

**決定: 待機は同期の `time.sleep` を 50 ミリ秒以下の刻みに区切り、刻みごとにシグナルのフラグと単調な時計の期限を確かめる。** スレッドを作らず、`threading.Event.wait(timeout)` も使わない。

手順(`wait_retry(secs, sig)`。`runner` に置く。時計と sleep は引数で差し替えられるようにしてよい):

```text
start_ns    = time.monotonic_ns()
deadline_ns = start_ns + secs * 1_000_000_000
loop:
    n = sig.number()
    if n is not None: return Interrupted(n)                        # 待機の開始時にも確かめる(最初の番号)
    now_ns = time.monotonic_ns()
    if now_ns >= deadline_ns: return Elapsed
    time.sleep(min(deadline_ns - now_ns, 50_000_000) / 1e9)
```

```python
# 型の正本は 07 章(6.2 節の Interrupt、7.5 節の Elapsed・Interrupted)。この章が必要とする形:
#   Interrupt.number() -> int | None     # 最初に受け取ったシグナルの番号。無ければ None(番号 0 と区別する)
@dataclass(frozen=True, slots=True)
class Elapsed:
    interrupted: Literal[False] = False

@dataclass(frozen=True, slots=True)
class Interrupted:
    number: int
    interrupted: Literal[True] = True

WaitOutcome: TypeAlias = Elapsed | Interrupted
```

守るべきことは 2 つ: (a) 合図から 1 秒以内に打ち切る(この方式では、合図から最大でも約 50 ミリ秒)。(b) 待ち時間が `secs` 秒以上になるまで `Elapsed` を返さない(経過は単調な時計 `time.monotonic_ns()` で毎回確かめる。`time.time()` は使わない)。

**この方式を選んだ理由と、確認した挙動**(CPython 3.9.6・3.14.10・3.15.2、macOS。子プロセスから `SIGTERM` を 0.3 秒後に送り、ハンドラは `Interrupt` を立てて時刻を記録するだけ。3 つの版で同じ結果):

| 待ち方 | ハンドラが走った時刻 | 待ちから戻った時刻 | 意味 |
|---|---|---|---|
| `time.sleep(1.0)` を 1 回 | 0.30 秒 | 1.00 秒 | ハンドラは**すぐ**走る(メインスレッドで、`sleep` が `EINTR` で中断されたとき)が、`sleep` は**残りの時間を待ち直す**(PEP 475)。1 回の長い `sleep` では、シグナルの後も最後まで待ってしまう |
| `Event.wait(1.0)`(ハンドラは何もしない) | 0.30 秒 | 1.00 秒 | 同じ。待ちは再開される |
| `Event.wait(1.0)`(ハンドラが `Event.set()` を呼ぶ) | 0.30 秒 | 0.30 秒 | すぐ戻る。ただし、ハンドラが `Event` の内部のロックを取るため、ハンドラの中でロックを取らない、という共通の方針(下)に反する |
| `time.sleep(0.05)` の刻みでフラグを確認 | 0.30〜0.31 秒 | 0.32〜0.33 秒 | シグナルから約 20 ミリ秒で戻る(刻みの途中で受けると、その刻みの残りを待つ。最大 50 ミリ秒) |

- **PEP 475**: シグナルで中断された `time.sleep`・`Popen.wait`・`os.read` などのシステムコールは、ハンドラが例外を出さない限り、自動で再試行される(`sleep` は残りの時間で)。ハンドラが走るのは、メインスレッドが次のバイトコードの境目に来たとき、または上のようにシステムコールが `EINTR` で戻ったとき。したがって、ハンドラは**フラグを立てるだけ**にし、「待ちを打ち切る」ことは待機のループの側が、刻みごとにフラグを見て行う。
- ハンドラは、複雑なこと(ロックを取る、`print`、`Event.set()`、ファイルへの書き込み)をしない。ロックを取る処理は、メインスレッドが同じロックを保持している最中にハンドラが走ると、自分自身を待って止まりうるため(この危険は、一般にシグナルハンドラで避けるべきこととして知られている。この仕様では再現していない。未確認)。フラグは単純な属性の代入(最初の 1 つだけ保持: `if self._number is None: self._number = signum`)にする。
- シグナルが、メインスレッド以外に配送される場合は、メインスレッドの `sleep` が中断されない(ハンドラはメインスレッドが次にバイトコードを実行するまで走らない)。この仕様の `conductor` はスレッドを作らないので、起きない。テストでは、シグナルを別のスレッドから `os.kill` で自分に送ると、この性質が現れうる(テストは別のプロセスから送る)。
- 刻みを 50 ミリ秒にしたのは、v0.8.0 の挙動(合図を 50 ミリ秒以下の間隔で確かめ、合図から 1 秒以内に打ち切る)に合わせるため。刻みが短いほど応答が速いが、起床の回数が増える(50 ミリ秒で毎秒 20 回で、負荷としては無視できる)。
- 経過時間は `time.monotonic_ns()` で毎回確かめるので、`sleep` が早く戻っても(または遅く戻っても)、`Elapsed` は期限以降にしか返らない。確認済み: `wait_retry(1, ...)` の所要時間は 1.002 秒(1 秒以上)。

待機中に中断された場合は、次の実行をせず、分類なし(起動しなかった中断)として扱う: 終了コード `128 + 番号`、ジョブの状態は変えず(エラーとして記録しない)、再開位置はそのジョブ。`pipeline` でも、そこで止まって終わる(§5.3: 分類なしは停止)。再開すると、そのジョブを最初の実行から数え直す。

テストでは、`Deps` の待機を偽の実装に差し替え、待ち時間(秒の `int`)を記録するだけで実際には待たない。待機の実装自体のテストは、時計と `sleep` を差し替えた単体テストと、別のプロセスからシグナルを送る受け入れテストで行う。

### 7.9 リトライの知らせ

待機の**前**に、`-v` の有無によらず、標準エラーに 1 行(エラーではない。`conductor: retry: ` で始まる)。待機が中断された場合も、その回の知らせは出ている。

- 英語: `conductor: retry: job '<id>' exited with code <last_exit_code>; retry <n> of <max> in <wait_secs> seconds`
- 日本語: `conductor: retry: ジョブ '<id>' が終了コード <last_exit_code> で終わりました。<wait_secs> 秒後にリトライします(<n>/<max> 回目)`
- `<last_exit_code>` は直前の実行の終了コード(`exit_code` が無ければ 0)。`<n>` は 1 から。`<wait_secs>` は秒の整数(10 進、桁区切りなし。`str(wait_secs)` か `f"{wait_secs}"`。`f"{wait_secs:,}"` や `locale` による整形は使わない)で、1 のときも `seconds`(単数形の処理はない)。`<id>` は表示用にエスケープ・長さ制限される(規則は [08-output-and-i18n.md](08-output-and-i18n.md))。
- 例: `conductor: retry: job 'extract' exited with code 75; retry 2 of 5 in 20 seconds`

---

## 8. シグナルと中断(流れへの影響)

| 受信のタイミング | ジョブの結果 | ステートの記録 | 終了コード |
|---|---|---|---|
| ジョブの起動の前(`run`) | 起動しない。分類なし | ジョブの状態は変えない。`resume_job_id` = そのジョブ(初期ステートは、空だった場合に書かれている) | 128 + n |
| ジョブの実行中 | 子に同じシグナルを 1 回転送。終了を待つ。`interrupted`。分類 error | そのジョブを `error` で記録(`exit_code` はジョブの値があればそれ) | 128 + n |
| リトライの待機中 | 分類なし(§7.8) | 状態は変えない。再開位置 = そのジョブ | 128 + n |
| `pipeline` で、ジョブを実行・記録した直後(まだ完了していない) | - | 再開位置(§9)を記録 | `signal_exit_code(n)` = 128 + n |
| 定義の検査・ステートの確認の途中 | 検査は最後まで進む | 64・65・66・スキップ(0)が優先 | それぞれ |

- シグナルは SIGHUP・SIGINT・SIGQUIT・SIGTERM(UNIX 系)、Ctrl+C と Ctrl+Break(Windows)。2 回目以降のシグナルは追加の動作をしない。終了コードに使う番号は、最初に受け取ったもの。詳細は [07-process-and-signals.md](07-process-and-signals.md)。
- 実行中の中断ではジョブ自身の終了コードは見ない(分類 error、返す値は 128 + n)。
- `prepare` の順 1 で SIGINT のハンドラを登録するため、`run`・`pipeline` の実行中は `KeyboardInterrupt` は発生しない(Python の既定の SIGINT のハンドラが、フラグを立てるハンドラに置き換わる)。登録の前(起動の直後)に Ctrl+C が来た場合の扱いは [02](02-architecture.md)・[07](07-process-and-signals.md)。

---

## 9. 再開位置(`resume_point_on_interrupt`)

入力: 定義、各ジョブの状態。

```text
resume_job_id = 記述順で最初の「pending(または記録なし)または error」のジョブ(無ければ None)
completed     = (resume_job_id is None)
end_reached   = 「skipped(byEnd)」のジョブが 1 つでもあれば True
```

```python
@dataclass(frozen=True, slots=True)
class ResumePoint:
    resume_job_id: str | None
    completed: bool
    end_reached: bool
```

エラー・pending を区別せず、記述順で最初のものが再開位置。エラーのジョブが前にあれば、そのジョブ。この関数は記録を作らない(呼び出し側が `apply_interrupt`(§3.5)で書く)。

---

## 10. 完了と完了後

- 完了(`completed = True`)になるのは: 最後のジョブが success / warning で終わった、`@end` に到達した(pipeline、または individual で後ろに未完了が無い)、スキップの記録で最後のジョブまで済んだ(§3.4)。
- 完了のステートは `completed = true` のまま**残る**(`remove_on_complete` が真でない限り)。書式は変わらない。
- 完了のステートは、読み込み(`run`・`pipeline` の `prepare`)では「空」として扱う(定義との照合もしない)。次の `pipeline`・`run <最初のジョブ>` は先頭から始まり、初期ステートで上書きする。先頭以外のジョブを `run` で単独に呼ぶと順序外(66)。
- `remove_on_complete` が真のとき、完了のステートを書いた後に `reset_state` で削除する(失敗は 65。完了のステートは残る)。偽のときは削除を試みず、0 で終わる(`run` ではジョブの結果による終了コード)。
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
| 2 | エラーのジョブを再実行して、起動前・待機中に中断されたとき | 分類なしなので `status` は error のまま変わらないが、`exit_code` は消える(`apply_outcome` が `exit_code` に実行結果の値 = 無しを書く) |
| 3 | 成功直後の `resume_job_id` | 成功・警告のあとの `NextStep.resume_job_id` は無し。ただし後続のスキップ記録(`apply_skip`)は、エラーが無いとき `resume_job_id` を最初の pending にする。そのため、遷移で飛ばした後のステートには、成功直後でも `resume_job_id` が入りうる |
| 4 | 分類なし(起動前の中断)の終了コードの整形 | `is_error = False` で整形する。`128 + n` が 256 以上かつ下位 8 ビットが 0 になる番号(n = 128 など)は、0 になりうる(実在のシグナル番号では起きない) |
| 5 | `pipeline` の「完了を判定できない」 | 全ジョブを通過しても `completed` が真にならなければ 65(ステートは初期化しない) |
| 6 | ステートの誤りの原因の文 | 種類の文はロケールに従うが、理由(例: `JSON として読めません`)は常に日本語の固定文(06 §6.2) |
| 7 | `0` と「無い」の区別 | `exit_code` の `0` は「ある」。`is None` で判定し、真偽値で判定しない(§2.3) |
| 8 | `bool` と `int` | JSON の `true` / `false` を整数として受理しない。ステートの `exit_code` に `bool` を書かない(§0.3) |

## 13. `conductor/flow/` の純粋関数の署名の骨格

実装は書かない。引数と戻り値の型だけを固定する(名前と型は、ほかの文書・テストの共通の語彙にする)。次の骨格は、本体を `...` にした状態で mypy 2.4.1 の `--strict` に通した(本体が `...` の関数について「Missing return statement」が出る以外に誤りは無い。実装では値を返す)。`FlowDefinition`・`FlowJob` は、`conductor/definition/` の型のうち判断に必要な部分だけを、`flow` が必要とする形で宣言したもの(`definition` の型が構造として満たせばよい)。

```python
from __future__ import annotations
from dataclasses import dataclass
from typing import Final, Literal, TypeAlias

# --- conductor/flow/types.py(1.1〜1.3、7.2、7.5、9 の型に加えて) ---
RunMode: TypeAlias = Literal["individual", "pipeline"]
SkipDecisionReason: TypeAlias = Literal["SkipTarget", "AfterEnd", "AlreadyExecuted"]
DecisionReason: TypeAlias = Literal["SkipTarget", "AfterEnd", "AlreadyExecuted", "OutOfOrder"]

@dataclass(frozen=True, slots=True)
class EndTarget:                       # "@end"
    kind: Literal["end"] = "end"

@dataclass(frozen=True, slots=True)
class JobTarget:
    id: str
    kind: Literal["job"] = "job"

TransitionTarget: TypeAlias = EndTarget | JobTarget

@dataclass(frozen=True, slots=True)
class FlowJob:
    id: str
    success_exit_codes: tuple[int, ...]    # 省略時は (0,) を入れておく
    warning_exit_codes: tuple[int, ...]
    on_success: TransitionTarget | None = None   # on = success の遷移
    on_warning: TransitionTarget | None = None   # on = warning の遷移
    retry: RetryPolicy | None = None

@dataclass(frozen=True, slots=True)
class FlowDefinition:
    name: str                              # pipeline.name
    jobs: tuple[FlowJob, ...]              # 定義の記述順

@dataclass(frozen=True, slots=True)
class NextStep:
    next_job_id: str | None
    skipped_job_ids: tuple[str, ...]
    skip_reason_kind: SkipReason | None
    end_reached: bool
    completed: bool
    resume_job_id: str | None

@dataclass(frozen=True, slots=True)
class Run:
    kind: Literal["run"] = "run"

@dataclass(frozen=True, slots=True)
class Skip:
    reason: SkipDecisionReason
    kind: Literal["skip"] = "skip"

@dataclass(frozen=True, slots=True)
class Reject:
    reason: Literal["OutOfOrder"] = "OutOfOrder"
    kind: Literal["reject"] = "reject"

Decision: TypeAlias = Run | Skip | Reject

U64_MAX: Final = 2**64 - 1
I32_MIN: Final = -(2**31)
I32_MAX: Final = 2**31 - 1

# 検証済みの定義(03 章の Definition)から、判断に必要な部分だけを取り出す(純粋)。
# 遷移を TransitionTarget に、success_exit_codes の省略を (0,) に、retry と backoff の None を保って直す。
# `Definition` は 03 章の型(骨格の検査では、中身の無い仮の型で置き換えた)。
def to_flow_definition(definition: "Definition") -> FlowDefinition: ...

# --- conductor/flow/classify.py ---
def classify(job: FlowJob, execution: JobExecution) -> JobOutcome: ...
def exit_code_for(reported: int, is_error: bool) -> int: ...
def signal_exit_code(signal_number: int) -> int: ...
def sat_add_i32(a: int, b: int) -> int: ...

# --- conductor/flow/next_step.py ---
def next_step(
    job: FlowJob, outcome: JobOutcome, states: tuple[JobState, ...],
    definition: FlowDefinition, mode: RunMode,
) -> NextStep: ...
def classify_and_decide(
    job: FlowJob, execution: JobExecution, states: tuple[JobState, ...],
    definition: FlowDefinition, mode: RunMode,
) -> tuple[JobOutcome, NextStep]: ...

# --- conductor/flow/decide.py ---
def decide_before_run(
    job_id: str, definition: FlowDefinition, states: tuple[JobState, ...],
    resume_job_id: str | None, end_reached: bool,
) -> Decision: ...

# --- conductor/flow/resume.py ---
def resume_point_on_interrupt(definition: FlowDefinition, states: tuple[JobState, ...]) -> ResumePoint: ...

# --- conductor/flow/retry.py ---
def sat_mul_u64(a: int, b: int) -> int: ...
def is_retry_target(policy: RetryPolicy, execution: JobExecution) -> bool: ...
def next_wait_secs(policy: RetryPolicy, retries_done: int, last_wait_secs: int) -> int: ...
def retry_decision(
    job: FlowJob, execution: JobExecution, retries_done: int, last_wait_secs: int,
) -> RetryDecision: ...
```

判別共用体は、`match` で分岐する(mypy `--strict` が、すべての場合を書いたことを確かめる):

```python
def describe(d: Decision) -> str:
    match d:
        case Run():
            return "run"
        case Skip(reason=r):
            return r
        case Reject():
            return "reject"
```

`conductor/state/` の記録関数の署名:

```python
def initial_state(
    definition_path: str, definition: FlowDefinition, run_id: str | None,
) -> PipelineState: ...
def apply_outcome(
    s: PipelineState, job_id: str, execution: JobExecution, outcome: JobOutcome, next_: NextStep,
) -> PipelineState: ...
def apply_skip(
    s: PipelineState, job_id: str, reason: DecisionReason, definition: FlowDefinition,
) -> PipelineState: ...
def apply_interrupt(s: PipelineState, point: ResumePoint) -> PipelineState: ...
def state_equals(a: PipelineState, b: PipelineState) -> bool: ...   # a == b でよい(dataclass の全項目の比較)
```

(`next` は組み込み関数の名前なので、引数名には `next_` を使う。)

単体テストで固定する入力(最低限): §2.2 の表、§2.3 の全組み合わせの表(特に `exit_code = 0` が「ある」として扱われること)、§4.2 の例の表、§7.6 の `sat_mul_u64`・`next_wait_secs` の表と待ち時間の列、§6.1 の有効・無効の例(特に 30 文字の日本語が `TooLong` になること、`"a\n"` が `InvalidChar` になること)、§7.3 の `parse_duration_secs` の受理・拒否の例(全角数字・アンダースコア・空白・改行を含む)、`bool` を整数として受理しないこと。

## 14. 実装に任せてよい点と未確認

実装に任せてよい点(外から見える挙動が同じなら自由):

- 判断関数・記録関数の内部構造、型の表現(`Literal` か `enum` か。この文書は `Literal` と `dataclass` を前提にしたシグネチャを示した)。
- シグナルのフラグの型は 07 章に従う。この章は `number` だけを要求する。`Deps` の `Clock` は時刻(`now()`)だけで、待機は `Deps` の待機の差し替え点(02 章)が担う。
- 標準エラーの出力のバッファリング(ただし行単位で、順序を保つ)。
- 待機の刻みの具体値(50 ミリ秒以下。「合図から 1 秒以内」「`secs` 以上で `Elapsed`」を守る)。

この文書の確認の範囲:

- **確認済み(CPython 3.9.6・3.14.10・3.15.2、macOS)**: §0.3 の `bool` の挙動と mypy の扱い(mypy 2.4.1)、`&` による整形、§7.6 の境界例と待ち時間の列(計算)、`dataclass` の `==`、`re` の `$`・`\d`・`IGNORECASE` の落とし穴、`int()` の寛容さと 4300 桁の上限(3.14)、`time.sleep` と `Event.wait` の上限の例外(版で違う)、シグナルのハンドラの実行のタイミングと PEP 475 による待ちの再開(§7.8 の表。3 つの版で同じ)、`sys.exit` に範囲外の値を渡したときの終了コード、環境変数の不正なバイトが `surrogateescape` で入ること、`os.environ.get` の空文字列と未定義の区別、`surrogateescape` / `surrogatepass` でのバイト数。`dataclass(slots=True)` は 3.9 で `TypeError`(3.10 以降で使える)。

未確認:

- サポートする最小の版と最新の版(実装時に確認)での、上記の挙動の再確認。この確認は上の 3 つの版だけで行った。
- Windows での、`Popen.returncode` が符号なし 32 ビットで返ること(§1.2)と負の終了コードの実機の値、`time.sleep` とシグナル(Ctrl+C・Ctrl+Break)の相互作用、`time.monotonic` の分解能、`os.environ` の名前の大文字化、実行中ジョブへのシグナル転送の細部(07)。
- ほかの UNIX 系(Linux)での §7.8 の表の値(PEP 475 は Python の仕様なので同じと考えられるが、確かめていない)。
- ハンドラの中でロックを取る処理が自己デッドロックを起こす場面の再現(§7.8。一般則として避けるだけ)。
- ジョブの標準入出力の扱い(07 が子の入出力を引き継ぐと定める)の、この文書の流れへの影響。
- 複数シグナルがほぼ同時に届いたときの「最初の番号」は、ハンドラが呼ばれた順序に依存する。再実装では「最初にハンドラが呼ばれた 1 つ」を保持すれば足りる。ほぼ同時の到着の再現は未確認。
