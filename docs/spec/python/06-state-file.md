# 06. ステートファイル(論理項目・JSON の形式・読み書き・整合性・reset)

対象: Python で実装する `conductor` v0.8.0 と同じ挙動の実行ツール。ステートファイルは、あるパイプライン定義の実行の途中経過(どのジョブがどう終わったか、どこから再開するか)を保存するファイル。実行の意味論(いつ何を記録するか)は [05-execution-flow.md](05-execution-flow.md) にある。この文書は、ステートが持つ**論理的な項目**、保存の**必須の性質**、**JSON の形式**、読み書きの手順、異常系の挙動を定める。

ファイル名は `<定義ファイル名>.state` / `<定義ファイル名>.<run ID>.state`(定義ファイルの拡張子を含む)で、中身は **JSON**(UTF-8、BOM なし)。実装は `conductor/state/`(読み書き・整合性検査・reset)。ステート値の型(`JobState`、`PipelineState` など)は `conductor/flow/` と共有する([05](05-execution-flow.md) §1.1)。

この文書の確認の範囲: 「確認済み」と明記した Python の挙動は、macOS(Darwin 27.2)上の CPython 3.9.6・3.14.10・3.15.2 で実際に動かして確かめた。読み込み(§6.5〜§6.6)と書き出し(§4)と書き込み(§5)は、この文書の規則どおりに作った試作を動かして、表の入力・例の出力を確かめた(例 1〜9 の出力は、3 つの版で完全に一致した)。3.9 は、この仕様がサポートする最小の版より古く、差の確認に使った。Windows と、ほかの OS・版は未確認(§13)。サポートする版は「実装時に確認」する([11-quality-and-build.md](11-quality-and-build.md))。

## 0. この文書の読み方

| 区分 | 内容 |
|---|---|
| 必須(どの実装も同じ) | §2 の論理項目・値の規則・矛盾の条件、§3 の保存の性質、§4 の JSON の形式(キー名・型・固定の順序・書き出しの規則)、§6 の読み込みの検査(順序・種類・理由文・案内文)、§7 の整合性検査、§8 の reset の挙動、ファイルの場所と名前(§1)、§5 のパーミッションの規定 |
| 実装に任せてよい点 | 一時ファイルの名前の細部(§1 の条件を満たす)、内部の構造、読み込みの内部の作り(§6.5 の規則を満たす) |

---

## 1. 場所と名前

ステートファイルは、**定義ファイルと同じディレクトリ**に置く。

| run ID | ファイル名 |
|---|---|
| なし | `<定義ファイル名>.state` |
| あり | `<定義ファイル名>.<run-id>.state` |

- `<定義ファイル名>` は、定義ファイルのパスの最後の要素(拡張子を含む。`os.path.basename(def_path)`)。例: `conductor.yaml` → `conductor.yaml.state`、`--config ci/nightly.json --run-id 12345` → `ci/nightly.json.12345.state`。
- 定義ファイルのパスは、`--config` で与えられた文字列のまま(正規化・絶対化しない)。ディレクトリ部分は `os.path.dirname(def_path)`(親が無い `conductor.yaml` のようなパスでは `""`)。ステートのパスは `os.path.join(os.path.dirname(def_path), name)` で作る(確認済み: `os.path.join(os.path.dirname("c.yaml"), "c.yaml.state")` は `c.yaml.state`、`os.path.dirname("ci/n.json")` は `ci`、`os.path.basename("ci/n.json")` は `n.json`)。ディレクトリ部分が `""` のときは実行ディレクトリで、ディレクトリの操作(§5 の 9、§8.2 の一覧)には `"."` を渡す。標準エラーに出す `<ステートファイルのパス>` は、この結果の文字列。
- パスの `str` は、OS が渡した不正なバイトを `surrogateescape` の孤立サロゲート(U+DC80〜U+DCFF)として含みうる(確認済み: 引数の `\xff` は `'\udcff'` になる)。ファイルの操作(`open`・`os.open`・`os.replace` など)には、この `str` をそのまま渡す(Python が元のバイトに戻す)。**ステートの中身に書く `definition_path` / `definition_key` だけは、孤立サロゲートを U+FFFD に置き換えた文字列にする**(§4.3)。
- Windows では、`os.path` は `ntpath` の規則(`\` と `/` の両方を区切りとみなす)で動く。この仕様は、Windows でのパスの扱いを確認していない(§13)。
- run ID は検査済み(`A-Za-z0-9._-`、1〜64 文字、先頭が `.` でない。[05](05-execution-flow.md) §6)なので、パス区切りや `..` は入らない。
- 一時ファイルの名前: `.<ステートファイル名>.tmp.<プロセス番号>.<時刻(ナノ秒)>.<連番>`(同じディレクトリ。§5)。名前の付け方は実装に任せてよいが、**ドットで始まり、`.tmp.` を含み、後始末(§5 の 9)で識別できる**こと。
- 大文字小文字を区別しないファイルシステム(macOS の既定、Windows)では、`A` と `a` の run ID は同じファイルになる。食い違いは、中身の `run_id` との照合で壊れたステート(65)として検出される(§6 の 9)。

---

## 2. ステートの論理項目

### 2.1 ステート全体の項目

メモリ上の名前(Python。`PipelineState` の属性)と、ファイル上のキー名の対応を併記する。メモリ上の名前はキー名と同じ(snake_case)だが、`job` だけは属性名が `jobs`。

| 項目(ファイルのキー) | メモリ上の名前 | 型 | 必須 | 意味・値 |
|---|---|---|---|---|
| `format_version` | (書き出し時に固定値 `1` を出す) | 整数 | 必須 | 常に `1`。読み込みでは最初に検査される(§6) |
| `definition_key` | `definition_key` | 文字列 | 必須 | 定義ファイルのパス(`definition_path` と**同じ文字列**を書く。読むときは別々に保持するが、どちらも照合に使わない) |
| `definition_path` | `definition_path` | 文字列 | 必須 | 定義ファイルのパス(`--config` の文字列。文字列として表せない部分(不正な UTF-8 のバイト)は、孤立サロゲートから U+FFFD に置き換える。§4.3、[04](04-cli.md))。空でもよい |
| `pipeline_name` | `pipeline_name` | 文字列 | 必須 | 記録時の `pipeline.name`。照合に使う(§7) |
| `job_order` | `job_order` | 文字列の列(`tuple[str, ...]`) | 必須 | 記録時のジョブIDを、定義の記述順に並べたもの |
| `job` | `jobs` | ジョブ状態の列(`tuple[JobState, ...]`) | 必須 | ジョブごとの状態(§2.2)。`job_order` と同じ順・同じ数 |
| `resume_job_id` | `resume_job_id` | 文字列(`str \| None`) | 任意 | 次に実行する、または再開するジョブのID。無いときは項目自体を保存しない。`job_order` に含まれる値 |
| `end_reached` | `end_reached` | 真偽 | 必須 | `@end` に到達したか。一度 true になったら、その run のあいだ false に戻らない |
| `completed` | `completed` | 真偽 | 必須 | 完了したか(§9) |
| `run_id` | `run_id` | 文字列(`str \| None`) | 任意 | run ID。run ID ありのステートだけが保存する。無いステートには項目を保存しない |

上の表にない項目は、ステート全体にも、ジョブ状態の中にも許さない(§6 の `corrupt`)。

### 2.2 ジョブ状態の項目

| 項目(ファイルのキー) | メモリ上の名前 | 型 | 必須 | 意味・値 |
|---|---|---|---|---|
| `id` | `id` | 文字列 | 必須 | ジョブID |
| `status` | `status` | 文字列 | 必須 | `"pending"` / `"success"` / `"warning"` / `"error"` / `"skipped"`(これ以外は `corrupt`) |
| `exit_code` | `exit_code` | 整数(32 ビット符号付き整数の範囲: -2147483648〜2147483647。`type(x) is int`) | 任意 | ジョブの終了コード。`status` が success / warning / error のときだけ持てる。終了コードが無い結果(起動失敗・シグナル終了)の error には保存しない。負数も許す(例 `-1`)。範囲外の整数は `corrupt`。`bool` は整数でない |
| `skip_reason` | `skip_reason` | 文字列 | 条件付き | `"byTransition"`(遷移によるスキップ)/ `"byEnd"`(`@end` によるスキップ)。**`status = "skipped"` のときだけ必須、そのほかの `status` では保存してはならない** |

### 2.3 内容の矛盾(読み込み時に `corrupt` になる条件)

- ジョブ状態のジョブID に重複がある。
- ジョブ状態の並び(`id` の列)が `job_order` と、数も順序も完全には一致しない。
- `resume_job_id` が `job_order` にない。
- `resume_job_id` があるのに `completed = true`(完了と再開位置は両立しない)。
- `status` が `skipped` なのに `skip_reason` が無い、または `skipped` 以外なのに `skip_reason` がある。
- `status` が pending / skipped なのに `exit_code` がある。

### 2.4 初期ステート(`initial_state`)

定義から作る、記録前のステート: 全ジョブ `pending`(`exit_code`・`skip_reason` なし)、`resume_job_id` 無し、`end_reached = False`、`completed = False`、`definition_key` = `definition_path` = 定義ファイルのパス文字列、`pipeline_name` = 定義の名前、`job_order` = 定義のジョブIDの記述順、`run_id` = 決定した run ID(あれば)。

### 2.5 不変条件(実装が保つもの)

[05](05-execution-flow.md) §3 の記録関数が作るステートは、次を満たす(読み込みの検査と対応する)。

- ジョブ状態と `job_order` は同じ順・同じID。
- `skipped` のジョブにだけ `skip_reason` がある。`exit_code` は結果のある状態(success / warning / error)にだけある。
- `end_reached` は単調(true になったら戻らない)。
- `completed = true` のとき `resume_job_id` は無い。
- ただし、エラーのジョブを再実行して「起動前の中断」([05](05-execution-flow.md) §12 の 2)になったときは、`status = error` のまま `exit_code` が消える。

---

## 3. 保存の必須の性質

| # | 性質 | 内容 |
|---|---|---|
| 1 | 原子的な置換 | 読み手は、直前の完全な内容か、新しい完全な内容のどちらかだけを見る。途中の内容は決して見えない。どの段階で失敗しても、ステートファイルは直前の完全な内容のまま(§5) |
| 2 | 再開に必要な情報 | 全ジョブの `id`・`status`・`exit_code`・`skip_reason`、`resume_job_id`、`end_reached`、`completed`、定義との照合用の `pipeline_name`・`job_order`、run ID を保存する。リトライの回数・待ち時間は保存しない(再開は最初の実行から数え直す) |
| 3 | 壊れたときの検出 | 読み込みで、構文・型・値・矛盾・版・run ID・大きさを検査し(§6)、壊れていれば**黙って続行せず**、種類と理由を表示して 65 で止まる。読み込みは、ファイルを消さず、書き換えない |
| 4 | 完了のステートの扱い | 完了のステートは書式を変えず残し、読み込み(`run`・`pipeline`)では「空」とみなす(§9) |
| 5 | 耐久性の 2 段階 | 結果・スキップ・中断の記録は、内容の同期まで行う(Durable)。最初の初期ステートだけは、同期を省いてよい(Fast)(§5) |
| 6 | 排他制御はしない | ファイルロック・PID ファイルを使わない(§11) |

---

## 4. JSON の形式(`format_version = 1`)

### 4.1 エンコードの基本

- UTF-8 の JSON(RFC 8259 の 1 個の値)。最上位は**オブジェクト**。BOM は書かない(読むときは拒否する。§6.1 の 5・6a)。
- 論理項目とのキーの対応は同名: §2.1 の項目は最上位のキー、§2.2 は `job` の配列の要素(オブジェクト)の中のキー。`job` は**オブジェクトの配列**(キー名は `job` のまま。論理項目の名前と一致させるための決定)、`job_order` は文字列の配列。
- 任意の項目(`resume_job_id`、`run_id`、`exit_code`、`skip_reason`)は、無いときキー自体を書かない(`null` を書かない)。
- 空の配列は `[]` と書く(`null` にしない)。

### 4.2 スキーマ(キー・型・固定の順序)

**書き出すときのキーの順序は固定**とする(辞書順にしない)。読み込みはキーの順序に依存しない。

最上位:

| 順 | キー | JSON の型 | 有無 |
|---|---|---|---|
| 1 | `format_version` | 数値(整数 `1`) | 必須 |
| 2 | `definition_key` | 文字列 | 必須 |
| 3 | `definition_path` | 文字列 | 必須 |
| 4 | `pipeline_name` | 文字列 | 必須 |
| 5 | `run_id` | 文字列 | run ID があるときだけ |
| 6 | `job_order` | 文字列の配列 | 必須 |
| 7 | `job` | オブジェクトの配列 | 必須 |
| 8 | `resume_job_id` | 文字列 | あるときだけ |
| 9 | `end_reached` | 真偽(`true`/`false`) | 必須 |
| 10 | `completed` | 真偽 | 必須 |

`job` の要素:

| 順 | キー | JSON の型 | 有無 |
|---|---|---|---|
| 1 | `id` | 文字列 | 必須 |
| 2 | `status` | 文字列(`"pending"`/`"success"`/`"warning"`/`"error"`/`"skipped"`) | 必須 |
| 3 | `exit_code` | 数値(整数。32 ビット符号付き) | 結果のある状態で、終了コードがあるときだけ |
| 4 | `skip_reason` | 文字列(`"byTransition"`/`"byEnd"`) | `status = "skipped"` のときだけ |

書き出し用の関数の署名(読み込みには使わない。理由は §6.5):

```python
def serialize_state(state: PipelineState) -> str: ...   # §4.3 の規則で文字列にする(末尾に改行 1 つ)。失敗は WriteFailed 相当(§5 の 1)
```

- 出力のオブジェクトは、**`dict` に、キーを §4.2 の順に 1 つずつ代入して作る**(`run_id` などの省略可能なキーは、値があるときだけ、その順序の位置で代入する)。`dict` は挿入順を保ち(Python 3.7 以降の言語の仕様)、`json.dumps` は `sort_keys=False`(既定)のとき、その順で出力する(確認済み: `{"b":1,"a":2,"2":3,"1":4}` は、キーの順を変えずに出力される。整数に見えるキー名でも順序は変わらない)。
- 「無い」値のキーは、`None` を代入せず、**キーを代入しない**。`None` を代入すると `null` が出る(読み込みは `null` を型の誤りとして拒否する)。`exit_code` は **`is not None` で判定する**(`if exit_code:` と書くと、値 `0` のキーが落ちる)。`skip_reason`・`resume_job_id`・`run_id` も `is not None` で判定する。
- `exit_code` の値は、`type(x) is int` かつ -2147483648〜2147483647 であることを確かめてから出す(`json.dumps` は `True` を `true` として出してしまう。確認済み)。確かめに失敗したら、書き出しの失敗(理由 `内容を JSON にできません`。§5 の 1)にする。
- キーの順序を、単体テストで固定する(§4.4 の例との完全一致。§6.6)。

### 4.3 書き出しの規則

- 文字列 = `json.dumps(obj, ensure_ascii=False, indent=2, allow_nan=False)` の結果に、下の後処理をし、`"\n"` を 1 つ付けたもの。`separators` は指定しない(`indent` を指定したときの既定は `(",", ": ")` で、行末に空白が付かない。確認済み)。`sort_keys` は指定しない(`False`)。インデントは空白 2 つ、末尾の改行は LF 1 つ(CRLF にしない)。ファイルの末尾は改行 1 つ。
- **`ensure_ascii=False` にする**: 既定の `True` は、ASCII 以外のすべての文字を `\uXXXX`(astral は 2 つのサロゲート)にエスケープする。JSON としては有効で読み込みも同じ値を得るが、この仕様の実ファイルの例(日本語がそのまま現れる)と文字列の表記が変わるので、`False` に固定する。
- `json.dumps` は `<` `>` `&` をエスケープしない(確認済み。HTML 向けの特別扱いは無い)。`ensure_ascii=False` のとき、DEL(U+007F)と U+0085 もエスケープされない(確認済み)。
- 配列は、要素ごとに 1 行(インデントされる)。空配列は `[]`(確認済み)。
- 数値は 10 進の整数だけ(`1`、`0`、`-1`、`2147483647`)。小数点・指数は書かない。`allow_nan=False` は、万一の `float` の `nan`・`inf` を `ValueError` にして、不正な JSON を書かないための安全策(確認済み: `json.dumps(float("nan"), allow_nan=False)` は `ValueError`)。
- 文字列の表記は `json.dumps(ensure_ascii=False)` の出力のまま。ただし次の 1 点だけ後処理する: 出力に含まれる U+2028・U+2029 を、それぞれ 6 文字の文字列 ` `・` `(バックスラッシュ + `u` + 4 桁の 16 進)に置換する(`ensure_ascii=False` の `json.dumps` は、この 2 文字をそのまま(エスケープせず)出力する。確認済み)。どちらも JSON としては有効だが、改行と見なす構文解析との互換のための決定で、この置換をしても読み込み側(§6.5)は同じ値を得る。`str.replace(" ", "\\u2028")` と `str.replace(" ", "\\u2029")` でよい。
- 制御文字・`"`・`\` はエスケープされる(`\n`、`\t`、`\"`、`\\`、`\b`、`\f`、`\r`、その他の制御文字は小文字の 16 進で `\u001f` のように。確認済み)。読み込みは JSON として正しければどの表記も受け付ける。
- **孤立サロゲート**: `ensure_ascii=False` の `json.dumps` は、孤立サロゲートを `str` の中にそのまま残す(確認済み: `json.dumps({"a": "\ud800"}, ensure_ascii=False)` は `'{"a": "\ud800"}'`)ので、UTF-8 に符号化するとき(`str.encode("utf-8")`)に `UnicodeEncodeError` になる(確認済み)。引数のパスは、OS から不正なバイトを `surrogateescape` の U+DC80〜U+DCFF として含みうる(確認済み。§1)ので、**`definition_path` と `definition_key` は、書き出す前に次の関数で置き換える**: `s.encode("utf-8", "surrogateescape").decode("utf-8", "replace")`(元のバイトに戻してから、不正な部分を U+FFFD に置き換えて復号する)。確認済み: `"dir/\udce3\udc81\udcff.yaml"`(元のバイト `E3 81 FF`)は `dir/` + U+FFFD + U+FFFD + `.yaml` になる(途中で切れた `E3 81` が 1 つ、単独の `FF` が 1 つで、合計 2 つ。不正なバイト列を「最大の正しい接頭辞ごとに 1 つ」置き換える Python の `replace` の規則による)。`surrogateescape` で符号化できない孤立サロゲート(U+DC80〜U+DCFF 以外。OS からは来ない)が混ざるときは、`UnicodeEncodeError` になるので、1 文字ずつ U+FFFD に置き換える(`re.sub("[\ud800-\udfff]", "�", s)`)。定義から来る文字列(`pipeline_name`・ジョブID など)は、孤立サロゲートを含まない([03](03-definition-format.md))ので、置き換えは `definition_path` と `definition_key` だけでよい。それでも `str.encode("utf-8")` が失敗したら、書き出しの失敗(理由 `内容を JSON にできません`)にする。
- 書き出した `str` は、`str.encode("utf-8")` でバイト列にして、バイトのまま書く(§5。テキストモードの改行の変換を受けない)。

### 4.4 実ファイルの例(全フィールドを含む)

以下は §4.3 の規則で出力される内容。例 1〜9 は、§4.2 の順序と §4.3 の規則のシリアライザを Python で実際に動かした出力で、CPython 3.9.6・3.14.10・3.15.2 の 3 つで完全に一致した(例 9 は `definition_path` に孤立サロゲートを含む入力から作った)。

(1)エラーで止まった run ID なしのステート(3 ジョブ。`b` が exit 3)。ファイル名 `c2.yaml.state`。

```json
{
  "format_version": 1,
  "definition_key": "c2.yaml",
  "definition_path": "c2.yaml",
  "pipeline_name": "n2",
  "job_order": [
    "a",
    "b",
    "c"
  ],
  "job": [
    {
      "id": "a",
      "status": "success",
      "exit_code": 0
    },
    {
      "id": "b",
      "status": "error",
      "exit_code": 3
    },
    {
      "id": "c",
      "status": "pending"
    }
  ],
  "resume_job_id": "b",
  "end_reached": false,
  "completed": false
}
```

(2)`@end` で完了した、run ID ありのステート。ファイル名 `conductor.yaml.r1.state`(a が成功し、遷移 `@end`。b・c が `byEnd`)。`resume_job_id` は無い。

```json
{
  "format_version": 1,
  "definition_key": "conductor.yaml",
  "definition_path": "conductor.yaml",
  "pipeline_name": "nightly",
  "run_id": "r1",
  "job_order": [
    "a",
    "b",
    "c"
  ],
  "job": [
    {
      "id": "a",
      "status": "success",
      "exit_code": 0
    },
    {
      "id": "b",
      "status": "skipped",
      "skip_reason": "byEnd"
    },
    {
      "id": "c",
      "status": "skipped",
      "skip_reason": "byEnd"
    }
  ],
  "end_reached": true,
  "completed": true
}
```

(3)全種類の状態を含む例(4 ジョブ。run ID なし、`end_reached = true`、エラーで停止、負の `exit_code`)。定義 `dir/conductor.yaml`、ファイル名は `dir/conductor.yaml.state`(ディレクトリ `dir` の中に置かれる)。

```json
{
  "format_version": 1,
  "definition_key": "dir/conductor.yaml",
  "definition_path": "dir/conductor.yaml",
  "pipeline_name": "nightly",
  "job_order": [
    "a",
    "b",
    "c",
    "d"
  ],
  "job": [
    {
      "id": "a",
      "status": "success",
      "exit_code": 0
    },
    {
      "id": "b",
      "status": "warning",
      "exit_code": 2
    },
    {
      "id": "c",
      "status": "skipped",
      "skip_reason": "byEnd"
    },
    {
      "id": "d",
      "status": "error",
      "exit_code": -1
    }
  ],
  "resume_job_id": "d",
  "end_reached": true,
  "completed": false
}
```

(4)`skip_reason = "byTransition"` の例(遷移 `a --success--> c` で b を飛ばした直後。a 成功、b は byTransition でスキップ、c が次)。ファイル名 `conductor.yaml.state`。

```json
{
  "format_version": 1,
  "definition_key": "conductor.yaml",
  "definition_path": "conductor.yaml",
  "pipeline_name": "nightly",
  "job_order": [
    "a",
    "b",
    "c"
  ],
  "job": [
    {
      "id": "a",
      "status": "success",
      "exit_code": 0
    },
    {
      "id": "b",
      "status": "skipped",
      "skip_reason": "byTransition"
    },
    {
      "id": "c",
      "status": "pending"
    }
  ],
  "end_reached": false,
  "completed": false
}
```

(`resume_job_id` が付くかどうかは書き込みの契機による。遷移のスキップは結果の記録([05](05-execution-flow.md) §3.3)で書かれ、その `resume_job_id` は無し。この例のステート値は規則から導いたもので、書き出しの形式は動かして確かめたが、実行の流れの中で書かれた出力そのものを確かめたわけではない。)

(5)**すべてのキーが現れる**例(run ID あり。ファイル名 `ci/nightly.json.12345.state`。定義 `ci/nightly.json`。`run_id`、`resume_job_id`、`exit_code`(0、正、負)、`skip_reason` が全部ある)。

```json
{
  "format_version": 1,
  "definition_key": "ci/nightly.json",
  "definition_path": "ci/nightly.json",
  "pipeline_name": "nightly",
  "run_id": "12345",
  "job_order": [
    "a",
    "b",
    "c",
    "d"
  ],
  "job": [
    {
      "id": "a",
      "status": "success",
      "exit_code": 0
    },
    {
      "id": "b",
      "status": "warning",
      "exit_code": 2
    },
    {
      "id": "c",
      "status": "skipped",
      "skip_reason": "byTransition"
    },
    {
      "id": "d",
      "status": "error",
      "exit_code": -1
    }
  ],
  "resume_job_id": "d",
  "end_reached": false,
  "completed": false
}
```

(6)ジョブIDに特殊な文字を含む例(`"`、`\`、日本語、タブ、前後の空白)。ジョブID は定義の検証で空でなく `@` で始まらない文字列であることだけが保証される(前後の空白もそのまま)。

```json
{
  "format_version": 1,
  "definition_key": "c.yaml",
  "definition_path": "c.yaml",
  "pipeline_name": "名前 \"x\"",
  "job_order": [
    " a ",
    "b\\c",
    "日本語\tタブ"
  ],
  "job": [
    {
      "id": " a ",
      "status": "pending"
    },
    {
      "id": "b\\c",
      "status": "pending"
    },
    {
      "id": "日本語\tタブ",
      "status": "pending"
    }
  ],
  "end_reached": false,
  "completed": false
}
```

(7)定義が JSON のとき(ファイル名 `pipelines/ci.json.2026-11-20_01.state`。run ID に `-`・`_`・`.` を含む。`exit_code` が 32 ビット符号付き整数の最小値)。

```json
{
  "format_version": 1,
  "definition_key": "pipelines/ci.json",
  "definition_path": "pipelines/ci.json",
  "pipeline_name": "ci",
  "run_id": "2026-11-20_01",
  "job_order": [
    "build",
    "test",
    "deploy"
  ],
  "job": [
    {
      "id": "build",
      "status": "success",
      "exit_code": 0
    },
    {
      "id": "test",
      "status": "error",
      "exit_code": -2147483648
    },
    {
      "id": "deploy",
      "status": "pending"
    }
  ],
  "resume_job_id": "test",
  "end_reached": false,
  "completed": false
}
```

(8)U+2028・U+2029 と `<` `&` `>` を含む例(§4.3 の後処理の確認用。`pipeline_name` は `x` + U+2028 + `y`、ジョブID は `a` + U+2029 + `b <&>`)。

```json
{
  "format_version": 1,
  "definition_key": "c.yaml",
  "definition_path": "c.yaml",
  "pipeline_name": "x y",
  "job_order": [
    "a b <&>"
  ],
  "job": [
    {
      "id": "a b <&>",
      "status": "pending"
    }
  ],
  "end_reached": false,
  "completed": false
}
```

(9)`--config` のパスに UTF-8 として不正なバイトを含む例(`--config` のバイト列が `dir/` + `E3 81 FF` + `.yaml`。`str` では `'dir/\udce3\udc81\udcff.yaml'`)。ステートファイルの実際の名前は元のバイトのまま(`dir/` + `E3 81 FF` + `.yaml.state`)で、**中身の `definition_key` / `definition_path` だけ**が U+FFFD(下の `��`、2 文字)に置き換わる。

```json
{
  "format_version": 1,
  "definition_key": "dir/��.yaml",
  "definition_path": "dir/��.yaml",
  "pipeline_name": "nightly",
  "job_order": [
    "a",
    "b"
  ],
  "job": [
    {
      "id": "a",
      "status": "success",
      "exit_code": 0
    },
    {
      "id": "b",
      "status": "pending"
    }
  ],
  "end_reached": false,
  "completed": false
}
```

---

## 5. 書き込み(`write_state`)の手順と原子性

入力: 定義ファイルのパス、run ID、ステート、耐久性(Durable / Fast)。同期の関数で、失敗は `StateError`(kind = `writeFailed`)を送出する(型は §13)。

```text
1. data = serialize_state(state).encode("utf-8")   # §4.3。失敗(通常は起きない)なら WriteFailed(理由=「内容を JSON にできません」)
2. path = ステートファイルの場所(§1)
3. 一時ファイルを、同じディレクトリに、「既存なら失敗する」新規作成で作る
     fd = os.open(tmp_path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_BINARY", 0), 0o666)
     名前 = ".<ステートファイル名>.tmp.<os.getpid()>.<time.time_ns()>.<連番>"
     名前が既存(FileExistsError)なら、連番を進めて作り直す。最大 16 回。それでも作れなければ WriteFailed
     連番 = モジュール内の通し番号(itertools.count(1) の next())+ 試行回数(桁あふれは考えなくてよい)
4. 全内容を書く: 書けたバイト数(os.write の戻り値)だけ進めて、残りが無くなるまで繰り返す
5. Durable のときだけ、os.fsync(fd) で内容を記憶装置まで同期する
6. os.close(fd)(Windows は、開いたままだと移せない・消せない。close の失敗も失敗として扱う)
7. os.replace(tmp_path, path) で置き換える
8. 3〜7 のどこかで失敗(OSError)したら: fd が開いたままなら close(失敗は無視)、自分が作った一時ファイルを os.unlink(失敗は無視)し、
     WriteFailed(理由=「書き込みに失敗しました」)。下位の例外は `raise ... from e` で cause に入れ、表示しない
9. 成功し、かつ Durable のとき:
     UNIX 系では、ディレクトリを os.open(dir, os.O_RDONLY) で開いて os.fsync、close(失敗は無視。sync_directory が偽(Windows)では何もしない)
     同じ名前の型の一時ファイルのうち、更新から 24 時間以上たった通常のファイルを消す(失敗は無視)
```

API の選定と注意:

- **`tempfile` を使わない**: `tempfile.mkstemp()` は、ファイルを `0600`(所有者だけ)で作る(確認済み: `0o600`)。この仕様は、パーミッションを明示的に設定せず、OS 標準の新規作成の権限(`0666` から umask を引いた値)のままにする(下)ので、規定を満たせない。`os.chmod` で後から直す方法は、umask を読み取る標準の手段が無く(読むには `os.umask` で一度設定して戻す必要があり、競合しうる)、採らない。名前の規則(`.<ステートファイル名>.tmp.…`)も合わない。**`os.open(..., O_CREAT | O_EXCL, 0o666)` で、権限と名前を自分で制御する**。
- **パーミッションは明示的に設定しない**(`os.chmod` しない)。**確認済み(macOS、umask 022)**: `os.open(path, O_WRONLY | O_CREAT | O_EXCL, 0o666)` で作ったファイルの `stat.S_IMODE(os.fstat(fd).st_mode)` は `0o644`(`-rw-r--r--`)。3 つの版で同じ。既存のステートファイルを置き換えるときも、権限は引き継がず、新しい一時ファイルの権限になる(確認済み: `0o600` の既存ファイルへ `os.replace` したあと、`0o644`)。所有者の変更・ACL の操作はしない。Windows では、`mode` は読み取り専用の属性だけを意味する、と理解している(未確認)。
- **`os.O_BINARY`**: Windows では、`os.open` は既定でテキストモード(`\n` を `\r\n` に変換する)で開く。`getattr(os, "O_BINARY", 0)` を `flags` に足して、バイトのまま書く(UNIX 系にはこの定数が無い)。確認していない(Windows は未確認。文書にある挙動)。
- `os.open` が返す記述子は、子プロセスに引き継がれない(確認済み: `os.get_inheritable(fd)` は `False`。PEP 446)。
- 一時ファイルへの書き込みは `os.write` の戻り値で進める(1 回で書き切れないことがありうるため)。バッファを持つ `open()` / `os.fdopen()` を使う場合は、`flush()` してから `os.fsync(f.fileno())` する(`flush` を忘れると、同期の前にバッファの内容が書かれていない)。
- 時刻は `time.time_ns()`(実時計)でよい。クロックが戻っても、名前の衝突は連番の再試行(最大 16 回)で避ける。
- `os.fsync` は Windows でも使える(ファイルの記述子に対して)。ディレクトリの `os.open(dir, os.O_RDONLY)` は Windows では `PermissionError` になる、と理解している(未確認)ので、`StateStoreOptions.sync_directory`(`cli` が `PlatformKind` から決めて渡す。Windows は偽。02 章)が偽のとき何もしない。`state` は `sys.platform` を読まない。

性質:

- **原子的**: 読み手は、直前の完全な内容か、新しい完全な内容のどちらかだけを見る。途中の内容は見えない。どの段階で失敗しても、ステートファイルは直前の完全な内容のまま。POSIX の `rename` は原子的で、`os.replace` はそれを使う(文書にある。Windows の原子性は未確認)。
- 置き換え先がディレクトリなど、名前の付け替えが失敗するときは `WriteFailed`。ステートは変わらず、一時ファイルも残らない(確認済み: `os.replace(file, directory)` は `IsADirectoryError`(errno `EISDIR`)で失敗し、失敗後に一時ファイルを消す手順 8 で何も残らない)。書けないディレクトリ(確認済み: `PermissionError`)・存在しないディレクトリ(確認済み: `FileNotFoundError`)も `WriteFailed`。
- 残った一時ファイル(プロセスが落ちたとき)は、読み込みに影響しない。`Durable` の書き込みの成功後に、24 時間以上古いものだけ消える。`Fast` では消さない(確認済み: 25 時間前の更新時刻の一時ファイルは、`Durable` の書き込み後に消え、`Fast` の書き込みでは残った。ほかの名前の古いファイルは消えなかった)。
- 「同じ名前の型」= 名前が `.<ステートファイル名>.tmp.` で始まるもの。後始末は、`os.scandir(dir)` の各エントリのうち、`entry.name.startswith(prefix)` かつ `entry.is_file(follow_symlinks=False)` が真のもの(シンボリックリンク・ディレクトリを除く。リンクを辿らない。確認済み: シンボリックリンクは `is_file(follow_symlinks=False)` が偽、`is_symlink()` が真)で、`entry.stat(follow_symlinks=False).st_mtime` が `time.time() - 24 * 60 * 60` 以前のものを `os.unlink` する(エラーは無視)。`scandir` は `with` で閉じる。
- 排他制御(ファイルロック)はしない(§11)。

### 5.1 Windows での置き換えの注意

- `os.replace` は、置き換え先が既にあっても上書きする、と文書にある(「置き換え先がファイルなら、権限があれば黙って置き換える」)。Windows での実機の確認はしていない(**未確認**。Windows の CI のテストで、既存のファイルへの `os.replace` が成功することを確認する)。
- 一時ファイルは、`os.replace` の前に `os.close` する(開いたままだと移せない)。
- 置き換え先のステートファイルを、ほかのプロセス(例: 同時に走る `watch`)が開いている間は、Windows では `os.replace` が `PermissionError` で失敗しうる(**未確認**)。失敗したら `WriteFailed`(65)にする。再試行はしない(仕様として、読み手との同時の実行は想定しない。§11)。
- ディレクトリの同期は行わない(Windows ではディレクトリを開いて `fsync` できないと理解している。未確認)。
- 一時ファイルの後始末は、UNIX 系と同じ(24 時間以上古い通常のファイル)。

### 5.2 いつ書くか(書き込みの契機)

| 契機 | 内容 | 耐久性 |
|---|---|---|
| ステートが空で、最初のジョブが `Run` と決まった直後(ジョブを起動する前) | 初期ステート。完了のまま残ったステートがあっても、同じ名前のファイルを初期ステートで**上書きする** | `Fast`(同期を省く) |
| ジョブの最終の結果が出た後(リトライを含めて 1 回だけ) | [05](05-execution-flow.md) §3.3 の結果の記録。起動前の中断・待機中の中断も、この契機で書く(ジョブの状態は変わらず、`resume_job_id` が付く) | `Durable` |
| スキップが決まったとき([05](05-execution-flow.md) §3.4 の記録が元と異なるときだけ) | スキップの記録 | `Durable` |
| `pipeline` のジョブのあいだの中断 | [05](05-execution-flow.md) §3.5 の中断の記録 | `Durable` |
| 完了(`completed = true`)になったとき | 上の契機の書き込みに含まれる(完了のステートも同じ手順) | `Durable` |

- `Reject`(66)では何も書かない(初期ステートも)。
- 書き込みの失敗(65)の表示の末尾に、ジョブの起動前なら `(ジョブは起動していません / the job was not started)`、起動後なら `(ジョブは実行されました / the job has already been executed)` が付く(ロケールに従う)。
- 結果を書けなかったときは、以降のジョブを起動せず、65 で終わる。

---

## 6. 読み込み(`read_state` / `read_state_raw`)の手順と異常系

### 6.1 手順

入力: 定義ファイルのパス、run ID(ファイル名側の期待値)。上から順に、最初に当てはまる結果で終わる。`read_state` は完了のステートを「空」にし、`read_state_raw`(`watch` が使う)は 14 を行わない。どちらも同期の関数で、空のときは `None` を返し、失敗は `StateError` を送出する(§6.2)。

| 順 | 処理 | 結果 |
|---|---|---|
| 1 | `open(path, "rb")` | 存在しない(`FileNotFoundError`。errno `ENOENT` だけ。確認済み)→ **空**(ステート無し)。その他の開く失敗(権限など。`NotADirectoryError`(errno `ENOTDIR`)を含む。確認済み: 親が通常のファイルだと `NotADirectoryError`)→ `unreadable` |
| 2 | 開いたファイルの `os.fstat(f.fileno()).st_size` を確認 | `fstat` の失敗 → `unreadable`。1 MiB(1,048,576 バイト)**超** → `tooLarge` |
| 3 | 上限 + 1 バイトまで読む(§6.1.1)。終わりのない入力でも止まる | 読み取り失敗 → `unreadable`。読めたバイト数が 1,048,576 を超える → `tooLarge`(ちょうど上限は受理) |
| 4 | 長さ 0 | **空** |
| 5 | UTF-8 として解釈(§6.1.2)。BOM を**取り除かない** | 失敗 → `corrupt`(理由 `UTF-8 の文字列ではありません`) |
| 6 | JSON として解析(下の 6a〜6c を順に) | 失敗 → `corrupt`(6a・6b は `JSON として読めません`、6c は `キーが重複しています`) |
| 7 | `format_version` の確認(**ほかの検査より先**。将来の版が、未知の項目で corrupt にならないように) | 項目無し → `corrupt`(`format_version がありません`)。整数以外 → `corrupt`(`format_version が整数ではありません`)。整数で 1 以外(0 や 2 や巨大な整数を含む)→ `versionMismatch`(`対応していない版です(reset が必要です)`) |
| 8 | 最上位に未知の項目がある | `corrupt`(`知らないキーがあります`) |
| 9 | `run_id` の取り出しと照合 | 型が文字列でない(`null` を含む)→ `corrupt`(`値の型が誤っています`)。中身の `run_id`(無ければ「なし」)がファイル名の run ID(無ければ「なし」)と一致しない → `corrupt`(`run_id がファイル名の run ID と一致しません`) |
| 10 | 必須項目の取り出し: `definition_key`、`definition_path`、`pipeline_name`、`end_reached`、`completed`、`job_order`、`job`(**この順に 1 項目ずつ**、無い/型が違うを判定する) | 無い → `corrupt`(`必須のキーがありません`)。型が違う → `corrupt`(`値の型が誤っています`)。`job_order` は文字列の配列(要素に文字列以外があれば型の誤り)、`job` は配列(要素がオブジェクトかは順 11 で見る) |
| 11 | `job` の各要素を、**配列の順に 1 つずつ**、次の順で検査する(下の表) | 下の表のとおり、すべて `corrupt` |
| 12 | `resume_job_id` の型 | 文字列でない(`null` を含む)→ `corrupt`(`値の型が誤っています`) |
| 13 | 内容の矛盾(この順): ジョブIDの重複 → `job` の並びと `job_order` の不一致 → `resume_job_id` が `job_order` に無い → `completed` かつ `resume_job_id` あり | いずれも `corrupt`(それぞれ `ジョブIDが重複しています`、`job の並びが job_order と一致しません`、`resume_job_id が job_order にありません`、`completed と resume_job_id が両立しません`) |
| 14 | `read_state` のみ: `completed = true` | **空**(完了のまま残ったステート。ファイルは消さない) |

#### 6.1.1 順 3 の読み取り

`with open(path, "rb") as f:` の中で `f.read(1048576 + 1)` を 1 回呼ぶ(上限 + 1 バイトまで)。バッファ付きの読み取り(`open(..., "rb")` の既定)の `read(n)` は、終端に達しない限り `n` バイトを返す(終端なら、読めた分だけ。確認済み: パイプの 3 バイトを `read(10)` で読むと 3 バイト)。**確認済み(macOS)**: `/dev/zero`(無限に読める特殊ファイル)に対して `read(1048577)` は 1,048,577 バイトを返して止まる(`os.fstat` の `st_size` は 0 なので順 2 では止まらず、順 3 で `tooLarge` になる)。`pathlib.Path.read_bytes()` / `open(...).read()`(引数なし)は、上限なしで全部を読むので使わない。`buffering=0`(生の読み取り)は、1 回の `read(n)` が `n` バイト未満を返しうるので使わない。名前付きパイプ(FIFO)を `open` すると、書き手が現れるまで止まる(open の一般の性質。この仕様の確認はしていない。**未確認**)。ステートのパスが FIFO であることは想定しない。

#### 6.1.2 順 5 のデコード

```python
text = data.decode("utf-8")   # 厳密(errors="strict")。不正なバイト列なら UnicodeDecodeError → corrupt(UTF-8 の文字列ではありません)
```

- **`"utf-8"` を指定する(`"utf-8-sig"` にしない)**: `utf-8-sig` は先頭の BOM(`EF BB BF`)を**黙って取り除く**(確認済み: `b"\xef\xbb\xbf{}".decode("utf-8-sig")` は `'{}'`、`"utf-8"` なら `'﻿{}'`)。BOM を取り除くと、BOM 付きのファイルが正しい JSON として受理されてしまい、仕様(BOM は壊れたステート。§6.1 の 6a)と食い違う。`"utf-8"` なら先頭に U+FEFF が残り、6a で拒否される。
- **読み取りに、バイト列を渡さない**: 標準の `json.loads(bytes)` は、先頭の BOM を受け付け、UTF-16・UTF-32 を自動で判定して読む(確認済み(3.9・3.14))。この仕様のステートは UTF-8(BOM なし)だけなので、必ず上の厳密な復号で `str` にしてから `read_json` に渡す。
- 厳密な復号は、不正な UTF-8 を拒否する。**確認済み**: `ED A0 80`(サロゲートの符号化)、`C0 80`(最短でない符号化)、`F4 90 80 80`(U+10FFFF 超)、途中で切れた `E3 81` は、すべて `UnicodeDecodeError`。例外は握りつぶさず、`corrupt`(`UTF-8 の文字列ではありません`)に変換する。
- NUL は UTF-8 としては有効で、JSON の構文の検査(6a)で拒否される(文字列の外に現れれば構文の誤り(`Extra data` など)、文字列の中に現れれば制御文字の誤り。確認済み)。

順 6 の細目(この順。最初の失敗で終わる):

| 細目 | 処理 | 失敗の理由 |
|---|---|---|
| 6a | §6.5 の読み取り(自前の `read_json` と、深さの検査)で、全体を構文解析する。失敗するもの: 構文の誤り、BOM(先頭の U+FEFF)、値の後ろの余分な文字(2 つ目の値を含む)、空白だけの内容、制御文字(U+0000〜U+001F)の文字列内への直接の混入、`\u` エスケープの誤り、**対にならないサロゲートのエスケープ**(`"\ud800"`)、`NaN`・`Infinity`・`-Infinity`、入れ子が深すぎるもの(§6.5 の深さの上限 32 を超える) | `JSON として読めません` |
| 6b | 最上位の値がオブジェクト(`MapNode`)か(`null`、配列、数値、文字列、真偽の最上位を拒否する) | `JSON として読めません` |
| 6c | どのオブジェクトにも、同じキーが 2 回以上現れないか(最上位と `job` の各要素を含む全オブジェクト。エスケープを展開した後の文字列で比べる。`"id"` と `"id"` は同じキー) | `キーが重複しています` |

6a が 6c より先であることは、仕様として固定する(重複キーが先にあっても、後ろに構文の誤りがあれば、`JSON として読めません` になる)。空白の定義は JSON の空白(スペース、`\t`、`\n`、`\r`)。BOM・NBSP・垂直タブ・改ページは空白でないので、6a で拒否される(確認済み: `json.loads` は、先頭の ` `・`\x0b`・`\x0c` を拒否する)。書き出しは BOM を付けない。他のツールが BOM 付きで保存した場合は壊れたステートとして 65 になり、`reset` で回復する。

順 7 の「整数」の定義: 値の字句が `-?(0|[1-9][0-9]*)` に一致するもの(小数点・指数を含まない)。`1.0`、`1e0`、`"1"`、`true`、`null`、配列、オブジェクトは**整数以外**(`format_version が整数ではありません`)。`-0` と `0`、`2`、`-1`、`99999999999999999999`、5000 桁の数字は整数で 1 ではないので `versionMismatch`。`1` だけが受理される(`01` は JSON の構文として誤り(6a))。整数の判定は、木の型(`IntNode`・`HugeIntNode`。§6.5)で行い、受理は `IntNode` で値が 1 のときだけとする(`HugeIntNode` は 64 ビット符号つきを超える整数で、`int()` を経由しない)。

順 11 の `corrupt` の理由文(個別。いずれも種類は `corrupt`。1 つの要素の中の検査の順に上から):

| 順 | 違反 | 理由 |
|---|---|---|
| a | 要素がオブジェクトでない(`null` を含む) | `値の型が誤っています` |
| b | 未知のキー(大文字小文字も区別。`ID`、`Status` は未知。`__proto__` も未知のキー) | `知らないキーがあります` |
| c | `id` が無い | `必須のキーがありません` |
| d | `id` が文字列でない | `値の型が誤っています` |
| e | `status` が無い | `必須のキーがありません` |
| f | `status` が文字列でない | `値の型が誤っています` |
| g | `status` が 5 つの値以外 | `status の値が誤っています` |
| h | `exit_code` があり、整数でない(`1.0`、`1e2`、文字列、`null`、`true` / `false` を含む)、または 32 ビット符号付き整数の範囲外(§6.5 の字句による判定) | `値の型が誤っています` |
| i | `skip_reason` があり、文字列でない(`null` を含む) | `値の型が誤っています` |
| j | `skip_reason` が 2 つの値以外 | `skip_reason の値が誤っています` |
| k | `status = skipped` と `skip_reason` の有無の不整合 | `skip_reason が status と合いません` |
| l | `status` が pending / skipped なのに `exit_code` あり | `exit_code が status と合いません` |

- `read_state_raw`(`watch` が使う)は、14 を行わない(完了のステートもそのまま返す)。1〜13 は同じ。
- 検査の `run_id` 照合(9)は、`completed` の判定(14)より前なので、完了のステートでも run ID の食い違いは `corrupt`。
- 読み取りでは、ファイルを**消さない・書き換えない**。
- 「空」= ステートが無いのと同じ扱い。呼び出し側は、すべて pending のステートとして扱い、最初の `Run` の前に初期ステートを書く。
- 空のステートを返す関数の戻り値は `None`(`PipelineState | None`)。
- ディレクトリに対する `open(path, "rb")` は、UNIX 系では `open` の時点で `IsADirectoryError`(errno `EISDIR`)で失敗して `unreadable` になる(確認済み: macOS)。Windows と Linux は未確認(§13。Windows では `PermissionError` と理解している)。いずれも `OSError` の系統で、順 1 の「存在しない以外の開く失敗 → `unreadable`」に当たる。
- 型の判定は、`isinstance` でなく **`type(x) is str` / `type(x) is bool` / `type(x) is list` / `type(x) is dict`** で行う(§6.5)。`bool` が `int` のサブクラスであることに頼らず、`true` / `false` を整数として受理しない。

### 6.2 失敗の表示と終了コード

すべて終了コード 65。標準エラーの 1 行(固定の文 + パス + 固定の理由。入力の値・部品のエラー文は含めない):

```text
conductor: error: <ステートファイルのパス>: <kind の文>: <理由>[; <案内>]
```

| kind | 英語の文 | 日本語の文 | 理由の例(常に日本語) | 案内を付けるか |
|---|---|---|---|---|
| `unreadable` | `cannot read state file` | `ステートファイルを読めません` | `開く、または読み取ることができません` | 付ける |
| `tooLarge` | `state file is too large` | `ステートファイルが大きすぎます` | `大きさが上限(1 MiB)を超えています` | 付ける |
| `corrupt` | `state file is corrupt` | `ステートファイルが壊れています` | §6.4 の各理由 | 付ける |
| `versionMismatch` | `state file version is not supported` | `ステートファイルの版が合いません` | `対応していない版です(reset が必要です)` | 付ける |
| `writeFailed` | `cannot write state file` | `ステートファイルを書き込めません` | `書き込みに失敗しました` / `内容を JSON にできません` | 付けない(§5.2 の `(ジョブは…)` が付く) |
| `removeFailed` | `cannot remove state file` | `ステートファイルを削除できません` | `削除に失敗しました` | 付けない |

案内(読み込みのときだけ): run ID なし → ``; run `conductor reset` to discard the state``(日本語 ``; `conductor reset` でステートを初期化できます``)、run ID あり → ``; run `conductor reset --run-id <ID>` to discard the state``(`<ID>` は文字どおりの `<ID>`。日本語は ``; `conductor reset --run-id <ID>` でステートを初期化できます``)。

実例(英語ロケール):

```text
conductor: error: c2.yaml.state: state file is corrupt: JSON として読めません; run `conductor reset` to discard the state
conductor: error: c2.yaml.state: state file is corrupt: キーが重複しています; run `conductor reset` to discard the state
conductor: error: c2.yaml.state: state file version is not supported: 対応していない版です(reset が必要です); run `conductor reset` to discard the state
conductor: error: c2.yaml.r3.state: state file is corrupt: run_id がファイル名の run ID と一致しません; run `conductor reset --run-id <ID>` to discard the state
```

表示に使うパス(`<ステートファイルのパス>`)の文字列化(孤立サロゲートのエスケープ表示など)は [08](08-output-and-i18n.md)。

### 6.3 異常系の挙動の一覧(`run` / `pipeline`)

| 状況 | 結果 | ジョブの起動 | ステート |
|---|---|---|---|
| ファイルが無い | 空として続行 | する | 最初の `Run` の前に初期ステートを書く |
| 長さ 0 | 空として続行 | する | 同上(上書き) |
| `completed = true` | 空として続行(定義との照合もしない) | する | 初期ステートで上書き |
| 読めない(権限等) | 65 | しない | 変更なし |
| 1 MiB 超 | 65 | しない | 変更なし |
| UTF-8 でない・JSON の構文の誤り・BOM・最上位がオブジェクトでない・深すぎる入れ子・キーの重複・未知のキー・必須項目欠落・型の誤り・内容の矛盾 | 65(`corrupt`) | しない | 変更なし |
| `format_version` が 1 以外の整数 | 65(`versionMismatch`) | しない | 変更なし |
| 中身の `run_id` とファイル名の run ID が不一致、または片方だけ有る | 65(`corrupt`) | しない | 変更なし |
| 定義と食い違い(§7) | 65 | しない | 変更なし |
| 書き込み・削除の失敗 | 65 | §5.2 の表のとおり | 直前の内容のまま(書き込み)/ 残る(削除) |

どの異常でも、回復は `conductor reset`(または `--run-id`、`--all`)。`reset` は読まず検査もしないため、どんなファイルでも消せる(§8)。

### 6.4 理由文の全件表

読み込みで出しうる `corrupt` / `versionMismatch` / `unreadable` / `tooLarge` の理由文と、書き込み・削除の理由文の**全件**(23 行)。表の文言は、ロケールに関わらず日本語の固定文である。

| # | kind | 理由文 | 出る順 | 条件 |
|---|---|---|---|---|
| 1 | unreadable | `開く、または読み取ることができません` | 1〜3 | 開く・`fstat`・読み取りの失敗(存在しないを除く) |
| 2 | tooLarge | `大きさが上限(1 MiB)を超えています` | 2、3 | 1,048,576 バイト超 |
| 3 | corrupt | `UTF-8 の文字列ではありません` | 5 | UTF-8 として不正 |
| 4 | corrupt | `JSON として読めません` | 6a、6b | JSON の構文の誤り、BOM、余分な文字、最上位がオブジェクトでない、空白だけの内容、対にならないサロゲートのエスケープ、`NaN` / `Infinity`、入れ子の深さの超過・再帰の限界 |
| 5 | corrupt | `キーが重複しています` | 6c | 同じキーが 2 回以上(最上位・`job` の要素を含む全オブジェクト。標準の `json.loads` は重複を黙って許すので、独立した理由にした) |
| 6 | corrupt | `format_version がありません` | 7 | キー無し |
| 7 | corrupt | `format_version が整数ではありません` | 7 | 整数の字句でない(`1.0`、`"1"`、`null`、`true` など。整数の定義は §6.1) |
| 8 | versionMismatch | `対応していない版です(reset が必要です)` | 7 | 整数で 1 以外 |
| 9 | corrupt | `知らないキーがあります` | 8、11b | 最上位、または `job` の要素に未知のキー(大文字小文字を区別) |
| 10 | corrupt | `値の型が誤っています` | 9、10、11、12 | 型の誤り(`null` を含む。任意の項目でも「無い」とは扱わない)、`exit_code` の範囲外・小数・`bool` |
| 11 | corrupt | `run_id がファイル名の run ID と一致しません` | 9 | 中身とファイル名の run ID の不一致(`detail = "runIdMismatch"`) |
| 12 | corrupt | `必須のキーがありません` | 10、11 | 必須キーが無い |
| 13 | corrupt | `status の値が誤っています` | 11 | 5 つの値以外 |
| 14 | corrupt | `skip_reason の値が誤っています` | 11 | 2 つの値以外 |
| 15 | corrupt | `skip_reason が status と合いません` | 11 | `skipped` と `skip_reason` の有無の不整合 |
| 16 | corrupt | `exit_code が status と合いません` | 11 | pending / skipped に `exit_code` |
| 17 | corrupt | `ジョブIDが重複しています` | 13 | `job` の `id` の重複 |
| 18 | corrupt | `job の並びが job_order と一致しません` | 13 | 数または順序の不一致 |
| 19 | corrupt | `resume_job_id が job_order にありません` | 13 | |
| 20 | corrupt | `completed と resume_job_id が両立しません` | 13 | |
| 21 | writeFailed | `書き込みに失敗しました` | 書き込み | 一時ファイル・書き込み・同期・閉じる・置き換えの失敗 |
| 22 | writeFailed | `内容を JSON にできません` | 書き込み | JSON への変換・UTF-8 への符号化の失敗(`exit_code` の型・範囲の確認の失敗を含む) |
| 23 | removeFailed | `削除に失敗しました` | reset | 「存在しない」以外の削除の失敗 |

- キー名は大文字小文字を区別する(`Format_Version`、`ID` は未知のキー、または必須のキーが無い、として扱う)。
- 理由文の言語は、ロケールに関わらず日本語の固定文である(`conductor: error: ...` の行の中で、種類の文と案内だけがロケールに従う)。

### 6.5 JSON の読み込み: 定義と共用する自前の読み取り `read_json`(選定)

目的: 順序が固定の 14 段階の検査と、理由文の区別を、確実に満たす。

**決定: ステートの JSON は、定義・変換と同じ自前の読み取り `conductor.definition.read_json(text, defer_duplicate_key=True)` で読み([03-definition-format.md](03-definition-format.md) 第 5.3 節、第 14.2 節)、続けて `check_limits(root, "depth_only")` で深さだけを確かめる。標準ライブラリの `json.loads` は、製品のコードのどこでも(定義、ステート、変換とも)使わない。**

決めた理由:

- JSON の読み取りが 1 つなので、重複キー・整数の字句・`NaN`・サロゲート・深さ・空白・BOM などの挙動が、定義・ステート・変換で食い違わない(検査表は下)。同じ入力の受理・拒否を、2 つの実装で保つ負担と、版による差(下の「採らなかった案」)がなくなる。
- 03 章の読み取りは、重複キーを見つけても止めずに続ける指定(`defer_duplicate_key=True`)を持つ。これが、「構文の誤り(6a)が、重複キー(6c)より先」という規定の順序に、そのまま合う。
- ステートは位置を使わない(表示は固定の理由文だけ)ので、読み取りが返す位置は捨てる。木の型(`MapNode`、`SeqNode`、`StringNode`、`IntNode`、`HugeIntNode`、`FloatNode`、`BoolNode`、`NullNode`。03 章)で、型の判定が、`isinstance`(または `kind`)だけでできる。`bool` と `int` を取り違える余地がない(`BoolNode` と `IntNode` は別の型)。

**採らなかった案(標準の `json.loads` に、`object_pairs_hook`・`parse_int`・`parse_float`・`parse_constant` を渡して縛り、解析後に深さと孤立サロゲートを検査する)**: 次の限界を、フックと事後の検査で埋めれば、同じ結果にはできる。しかし、経路が 2 つになり、版による差を抱える。確認(CPython 3.9.6・3.14.10・3.15.2。特記のない限り 3 つで同じ):

| # | 標準の `json.loads` の挙動(確認済み) | 縛る方法(採らない) |
|---|---|---|
| 1 | 重複キーを検出せず、後のものが勝つ | `object_pairs_hook` で、キーの数を比べる |
| 2 | `NaN`・`Infinity`・`-Infinity` を受理する | `parse_constant` に例外を送出させる |
| 3 | `1.0` は `float`、`-0` は `int` の `0`。`true` は `bool`(`int` のサブクラス) | `parse_int`・`parse_float` に字句を持つ値を返させる |
| 4 | 3.11 以降は、4300 桁を超える整数の字句で、`int()` への変換が `ValueError`(3.9.6 には無い) | `parse_int` が `int()` を呼ばないようにする |
| 5 | 深い入れ子は、版で結果が違う(3.9 は約 1000 段で `RecursionError`。3.14・3.15 は 1 MiB の入力で、配列の入れ子 524,288 段は `RecursionError`、オブジェクトの入れ子 104,857 段は解析できた) | `RecursionError` を捕まえ、解析後に明示のスタックで深さを数える |
| 6 | `"\ud800"`(孤立サロゲートのエスケープ)を、孤立サロゲートを含む `str` にする | 解析後に、全文字列を検査する |
| 7 | `str` の先頭の U+FEFF を拒否する。`bytes` を渡すと BOM を受け付け、UTF-16・UTF-32 も自動で判定する | 厳密な UTF-8 の復号で `str` にしてから渡す |
| 8 | エラーの文言が版で変わる | 例外の文言を使わない |

**検査表(定義の読み取りと、ステートの読み込みで、挙動が同じであること)**: 次の入力は、どちらの経路でも同じ結果になる。ステートの欄は、§6.1 の順 6 の細目と、順 7〜12 の結果である。

| 入力 | 03 章の規定(定義・変換) | ステート(この章) |
|---|---|---|
| 重複キー(エスケープを展開した後の完全一致) | 構文の誤り(`キーが重複しています`)。ただし、後ろに文法の誤りがあれば、それが先 | 6c(`キーが重複しています`)。後ろに文法の誤りがあれば 6a が先 |
| 整数の字句 `-?(0\|[1-9][0-9]*)`、`1.0`・`1e0` は整数でない、`-0` は整数 0 | `IntNode`・`HugeIntNode`(64 ビット符号つきを超える)・`FloatNode`。整数だけの項目に `FloatNode` は型の誤り | `format_version` は `IntNode` で値が 1 のときだけ受理、ほかの `IntNode` と `HugeIntNode` は `versionMismatch`、`FloatNode`・`BoolNode`・文字列・`null` は「整数ではありません」。`exit_code` は `IntNode` で、-2147483648〜2147483647 だけ受理(`HugeIntNode` は型の誤り) |
| 桁数の多い整数(5000 桁など) | `int()` の桁数の上限に当たらない(19 桁を超えれば `int()` を呼ばない) | 同じ読み取りなので、3.9 でも 3.11 以降でも同じ結果 |
| `NaN`・`Infinity`・`-Infinity` | 構文の誤り | 6a |
| `true`・`false` | `BoolNode`(整数でない) | 整数でない。`exit_code`・`format_version` に受理しない |
| 孤立サロゲート(`\ud800` など) | 構文の誤り | 6a |
| 入れ子の深さ | 深さ 32 まで受理、33 段目を拒否(根を数えない)。文法の誤りが先 | 同じ(`check_limits(root, "depth_only")`)。拒否は 6a(`JSON として読めません`) |
| 1 MiB の `[` の列 | 明示のスタックで読み、文法の誤り(末尾)を返す。再帰で落ちない | 6a。プロセスが異常終了しない |
| BOM | 先頭の U+FEFF は無視して読む(定義・変換) | **拒否する**(6a)。唯一の意図した差で、読み取りの前の前処理による(下の注) |
| 空白 | スペース・タブ・LF・CR だけ | 同じ |
| 最上位が、オブジェクトでない | 型の誤り(第 6.7 節) | 6b |

- 注(BOM): 03 章は、定義と変換の入力の先頭の BOM(U+FEFF)を、取り除いて(無視して)読む。ステートは、BOM を壊れたものとして拒否しなければならない。したがって、ステートの読み込みは、`read_json` を呼ぶ**前に**、`text.startswith("\ufeff")` なら 6a にする(`read_json` が BOM を無視する実装でも、この前処理が先に働くので、結果は変わらない)。
- 木は、読み取り専用で、`dict` に入れて重複を落とさない。ステートの検査は、`MapNode` の `entries`(キーと値の並び)を、キー名で引く補助(最初の一致)で行う。重複は 6c で先に拒否されているので、引いた結果は一意である。

読み込みの手順(順 6〜13 を、この木に対して行う):

1. 先頭が U+FEFF なら 6a。そうでなければ `read_json(text, defer_duplicate_key=True)` を呼ぶ。失敗(`ReadJsonFailed`)→ 6a。
2. `check_limits(root, "depth_only")` が誤りを返したら 6a(深さ 32 まで受理、33 段目を拒否。根を数えない)。要素数・文字の大きさの上限は、ステートには適用しない(ファイルの大きさの上限 1 MiB が、別にある)。ステートの正しい内容の深さは 2(最上位のオブジェクト → `job` の配列 → 要素のオブジェクト)で、余裕がある。
3. `root` が `MapNode` でなければ 6b。
4. `duplicate_key` が `None` でなければ 6c。
5. 順 7〜12 は、仕様の順に、`root` から取り出す。未知のキーの検査(順 8)は、キーの全部が §2.1 の 10 個のどれかか、を見る。
6. 値の型の判定は、木の型(`isinstance`)で行う(`StringNode`・`BoolNode`・`SeqNode`・`MapNode`・`IntNode`・`HugeIntNode`・`FloatNode`・`NullNode`)。整数の判定は `IntNode` か `HugeIntNode`(`1.0`・`1e0` は `FloatNode`、`true`・`false` は `BoolNode`、`null` は `NullNode`)。
7. `format_version`: `IntNode` で `value == 1` のときだけ受理。ほかの `IntNode`(`0`、`-0`、`2` など)と `HugeIntNode` は `versionMismatch`。
8. `exit_code`: `IntNode` で、`-2147483648 <= value <= 2147483647` のときだけ受理。`HugeIntNode`、範囲外の `IntNode`、`FloatNode`、`BoolNode`、`NullNode`、文字列は、型の誤り。`-0` は値 0。
9. `job_order` と `job` の要素も同様に取り出す(`job` の各要素が `MapNode` でなければ、順 11 の a)。

### 6.6 読み込みの単体テストで固定する入力(最低限)

この表の入力は、§6.1〜§6.5 の手順どおりに作った試作の読み込みで実際に動かして、期待どおりになることを確かめた(CPython 3.14.10)。

| 入力(要点) | 期待 |
|---|---|
| 正常(§4.4 の例 1〜9) | 受理。往復(読んで書く)で元と一致(例 8 の U+2028・U+2029 のエスケープを含む) |
| 先頭に BOM(`EF BB BF`)+ 正常な内容 | `corrupt`(`JSON として読めません`)。`"utf-8-sig"` で復号するか、`read_json` の前に BOM の前処理を省くと、受理されてしまう(このテストで検出する) |
| 正常な内容の後ろに `{}` や `x` | `corrupt`(`JSON として読めません`) |
| `null`、`[]`、`1`、`"x"`、`true` | `corrupt`(`JSON として読めません`) |
| 空白だけ(`"\n"`) | `corrupt`(`JSON として読めません`) |
| 文字列の中に制御文字(生のタブ・NUL) | `corrupt`(`JSON として読めません`) |
| 入れ子の深さが 33(`{"a":[[[...]]]}` で 33 段目) | `corrupt`(`JSON として読めません`)。深さ 32 は受理(ほかの検査で別の理由になりうるので、深さだけを変えたテスト用の入力を使う) |
| 入れ子が極端に深い(1 MiB の `[` の列 + `]` の列) | `corrupt`(`JSON として読めません`)。プロセスが異常終了しない(`RecursionError` を 6a に変換する) |
| `{"format_version": NaN}`、`[Infinity]` | `corrupt`(`JSON として読めません`) |
| 最上位に同じキーが 2 回(`"completed": false, "completed": true`) | `corrupt`(`キーが重複しています`) |
| `job` の要素に同じキーが 2 回 | `corrupt`(`キーが重複しています`) |
| `"id"` と `"id"` が同じ要素に | `corrupt`(`キーが重複しています`) |
| 重複キーがあり、かつ後ろに構文の誤り | `corrupt`(`JSON として読めません`。6a が先) |
| `{"format_version": 2, "unknown": 1}` | `versionMismatch`(未知のキーより版が先) |
| `{"format_version": 9007199254740993}`、`{"format_version": 100000000000000000000}`、5000 桁の整数 | `versionMismatch`(`int` へ変換しない。3.9 でも 3.14 でも同じ結果になること) |
| `{"format_version": -0}`、`{"format_version": 0}` | `versionMismatch` |
| `{"format_version": 1.0}`、`{"format_version": true}` | `corrupt`(`format_version が整数ではありません`) |
| `{"format_version": "1"}` | `corrupt`(`format_version が整数ではありません`) |
| `{"Format_Version": 1}` | `corrupt`(`format_version がありません`) |
| `{"format_version": 1, "__proto__": {}}` | `corrupt`(`知らないキーがあります`) |
| `{"format_version": 1, "__proto__": 1, "__proto__": 2}` | `corrupt`(`キーが重複しています`) |
| `exit_code` が `1.5`、`2147483648`、`9007199254740993`、`"3"`、`null`、`true`、`1e2` | `corrupt`(`値の型が誤っています`) |
| `exit_code` が `-2147483648`、`2147483647`、`-0` | 受理 |
| `resume_job_id` が `null` | `corrupt`(`値の型が誤っています`) |
| `run_id` が `null` | `corrupt`(`値の型が誤っています`) |
| 文字列に `"\ud800"`(孤立サロゲートのエスケープ) | `corrupt`(`JSON として読めません`。conductor が書いたステートには現れない) |
| 文字列に `"😀"`(正しい対) | 受理(1 つの文字 U+1F600 になる) |
| 不正な UTF-8(`FF`)、サロゲートの符号化(`ED A0 80`) | `corrupt`(`UTF-8 の文字列ではありません`) |
| ディレクトリを指すパス | `unreadable`(macOS。ほかの OS は実機で確認する) |
| 1,048,576 バイトちょうど(正常な内容を空白で埋める)/ 1,048,577 バイト | 前者は受理、後者は `tooLarge` |
| 親が通常のファイルであるパス(`file/x`) | `unreadable`(`NotADirectoryError`) |
| 長さ 0 のファイル | 空(`None`) |

---

## 7. 定義との整合性検査(`check_consistency`)

空でない(完了していない)ステートを読んだ後、`run`・`pipeline` の前処理([05](05-execution-flow.md) §5.1 の 7)で、定義と照合する。純粋な関数(`conductor/state/`)。

```text
if state.pipeline_name != definition.name:              mismatch("pipelineName")
defined  = 定義のジョブIDの記述順の列(tuple)
recorded = state.job_order
if defined == recorded:                                  ok
if sorted(defined) == sorted(recorded):                  mismatch("jobOrder")    # 並びだけが違う
else:                                                    mismatch("jobIds")      # 追加・削除・改名
```

```python
@dataclass(frozen=True, slots=True)
class ConsistencyOk:
    kind: Literal["ok"] = "ok"

@dataclass(frozen=True, slots=True)
class ConsistencyMismatch:
    what: Literal["pipelineName", "jobIds", "jobOrder"]
    kind: Literal["mismatch"] = "mismatch"

ConsistencyResult: TypeAlias = ConsistencyOk | ConsistencyMismatch

def check_consistency(state: PipelineState, definition: FlowDefinition) -> ConsistencyResult: ...
```

比較は文字列の完全一致(`==`。コードポイントごとの比較で、大文字小文字・正規化を区別する。`unicodedata.normalize` をしない)。`sorted` は `sorted(ids)`(新しい `list` を返し、入力を変えない。`str` の比較はコードポイントの順で、UTF-8 のバイト列の順と一致する。確認済み: `sorted(["\U0001F600", "￿", "a"])` は `["a", "￿", "\U0001F600"]` で、UTF-8 のバイト列の比較と同じ順)で、重複は取り除かず、並べ替えた列同士の一致を見る。ここでの並べ替えは「集合として同じか」を見るためだけなので、一貫した全順序なら何でもよい。読み込んだ文字列は孤立サロゲートを含まない(§6.5)。

| 状況 | 結果 |
|---|---|
| パイプライン名が違う | `pipelineName` |
| ジョブID の集合が違う(追加・削除・改名。重複の有無も含め、並べ替えた列が違う) | `jobIds` |
| 集合は同じで記述順だけが違う | `jobOrder` |
| すべて同じ | `ok` |

- `definition_key` と `definition_path` は照合に使わない(定義ファイルの置き場所を動かしても食い違いにしないため)。
- ジョブのコマンド、遷移、成功コード、リトライ設定などの違いは検知しない(IDと並びだけを見る)。
- `completed = true` のステートは「空」なので、照合しない。
- 食い違いのとき、ジョブは起動せず、ステートも変えず、65。標準エラー:

```text
conductor: error: <ステートファイルのパス>: state does not match the definition: <種類>; <案内>
```

  種類: `pipeline name` / `job ids` / `job order`(日本語: `パイプライン名` / `ジョブID` / `ジョブの並び`)。日本語の文は `ステートが定義と食い違っています`。案内は §6.2 と同じ(run ID あり/なしで切り替わる)。

---

## 8. 初期化(`reset`)

### 8.1 `conductor reset [--config <パス>] [--run-id <ID>]`

- 読まず、検査せず(破損・食い違い・版違い・大きさに関係なく)ステートファイルを削除する。定義ファイルは検証しない。
- 対象のファイル: run ID の決定([05](05-execution-flow.md) §6.2。`--run-id` があれば定義を読まない。なければ定義の `pipeline.run_id_env` だけをゆるく読み、その環境変数を見る)で決まった 1 つ。
- 結果: ファイルが無ければ**成功**(0、何も表示しない)。削除が成功 → 0。失敗(ディレクトリ・権限等。「存在しない」以外)→ `removeFailed`、65。Python では **`os.unlink(path)`**(同じ働きの `os.remove`)を使い、`FileNotFoundError`(errno `ENOENT`)なら成功として扱う(確認済み: 存在しないパスの `os.remove` は `FileNotFoundError`)。
- 完了のステートも消す。標準出力・標準エラーには何も出さない。
- パスがシンボリックリンクなら、リンク自体を削除する(`os.unlink` はリンクを辿らず、リンク自体を消す。リンク先は消さない。確認済み: 削除後もリンク先のファイルが残った)。
- **ディレクトリを指すパスは消さない**。`os.unlink` はディレクトリに対して失敗する(確認済み: macOS では `PermissionError`(errno `EPERM`)。Linux では `IsADirectoryError`(`EISDIR`)と理解している。未確認)ので、`removeFailed`(65)になる。`os.rmdir`・`shutil.rmtree`・`pathlib.Path.rmdir` は、空のディレクトリや中身ごと消しうるため使わない。`NotADirectoryError`(親が通常のファイル)など、`FileNotFoundError` 以外の `OSError` も `removeFailed`。Windows でのディレクトリへの `os.unlink` の挙動は未確認(§13)。
- run ID の値が規則に合わなければ 68。

### 8.2 `conductor reset --all [--config <パス>]`

定義の全ステートを、run ID の有無によらず消す。環境変数は見ない(値の誤りも 68 にならない)。`--run-id` との併用は 68。

対象の判定(厳密):

1. 一覧にするのは、定義ファイルと同じディレクトリだけ(再帰しない。`os.listdir(dir)` か `os.scandir(dir)`。順序は保証されないので、名前を `sorted()` する。run ID は ASCII だけで、候補の名前は同じ接頭辞を持つので、`str` のコードポイントの順と UTF-8 のバイト列の順は一致する)。`dir` はディレクトリ部分が `""` のとき `"."`。
2. `<定義ファイル名>.state`(run ID なし): **通常のファイルのときだけ**(`os.lstat` の結果の `stat.S_ISREG(st.st_mode)` が真。シンボリックリンクは辿らず、消さない。`os.path.isfile` は辿るので使わない)、中身を読まずに削除(`os.unlink`)。
3. `<定義ファイル名>.<run-id>.state`: 名前の `<run-id>` が有効な run ID(§1・[05](05-execution-flow.md) §6.1 の `parse_run_id` が成功)であるものだけが候補。名前は、定義ファイル名で始まり、続けて `.`、run ID、`.state`(`len(name) >= len(def_name) + 1 + 1 + len(".state")` で、`name.startswith(def_name + ".")`、`name.endswith(".state")` で、その間 `name[len(def_name) + 1 : len(name) - len(".state")]` が run ID。run ID なしの `<定義ファイル名>.state` は、この間が空になるので、候補にならない)。候補をファイル名の昇順に処理する。
   - 通常のファイルでない(ディレクトリ、シンボリックリンク)→ 黙って除外。取得中に消えた(`os.lstat` が `FileNotFoundError`)→ 除外。`os.lstat` がその他の理由で失敗(`OSError`)→ `kept` に入れる。
   - 中身を見分ける(`inspect_run_state`。§6.1 の順 1〜13 を、ファイル名側の run ID を期待値として行う。`read_state_raw` と同じ手順):

| 中身 | 結果 |
|---|---|
| 開けない、1 MiB 超、読み取り失敗、UTF-8・JSON の構文・形式の誤り(`detail = "runIdMismatch"` 以外の `corrupt`。キーの重複・BOM・深すぎる入れ子を含む)、`versionMismatch` | **残して**、名前を `kept` に入れる |
| 長さ 0 | 残す。`kept` に**入れない**(誰のものか分からない) |
| 読めて、中身の `run_id` が名前の run ID と一致(完了のままのものを含む) | **削除** |
| 読めるが、中身の `run_id` が無い、または値が違う(別の定義のステートでありうる。`StateError` の `detail = "runIdMismatch"`) | 残す。`kept` に入れない(黙って残す) |

4. 一時ファイル(`.<ステート名>.tmp.*`)、ほかの名前のファイル、ほかのディレクトリは対象外。
5. 削除の失敗(「存在しない」以外)→ `removeFailed`、65(処理は途中で打ち切る)。ディレクトリ一覧の取得に失敗(ディレクトリが無い場合(`FileNotFoundError`)を除く)→ `unreadable`、65。ディレクトリが無ければ成功。

結果の出力: `kept` が空でなければ、標準エラーに警告(終了コードは 0)。消すものがなければ何も出さず 0。

```text
conductor: warning: kept state files that could not be read (remove one with `conductor reset --run-id <ID>`):
  c2.yaml.r2.state
```

日本語: ``conductor: warning: 読めなかったため、次のステートファイルは残しました(`conductor reset --run-id <ID>` で個別に消せます):``、続けて 2 スペースで字下げしたファイル名を 1 行ずつ。

例(`c2.yaml` に対して `reset --all`): run ID なしの `c2.yaml.state` は削除、順序が壊れた(JSON として壊れた、または内容に矛盾のある)`c2.yaml.r2.state` は `kept`(警告)、`"run_id": "r2"` を持つが名前が `r3` の `c2.yaml.r3.state` は黙って残る、ほかの定義のステート `conductor.yaml.r1.state` は対象外で残る。

---

## 9. 完了と `--remove-on-complete`

### 9.1 完了のステートの扱い

- 完了(`completed = true`)の条件は [05](05-execution-flow.md) §10。完了のステートは、`completed = true` のまま**残る**(自動では消えない)。書式は変わらない(`format_version` は 1)。
- 読み込み(`read_state`)は、完了のステートを「空」として返す(§6.1 の 14)。したがって次の `pipeline` は先頭から始まり、`run` は最初のジョブ以外が順序外(66)になる。
- `watch` は `read_state_raw` で完了のステートを読み、表の下に完了を示す行(英語 `completed`、日本語 `完了`)を足す。`no state (nothing has run yet)` を出すのは、ファイルが無いか長さ 0 のときだけ。壊れた完了のステートは 65。
- `reset`・`reset --all` は完了のステートも消す。

### 9.2 削除の指定

| 項目 | 値 |
|---|---|
| オプション | `--remove-on-complete`(`run`・`pipeline` だけ。値を取らない。ほかのコマンドに付けると 68) |
| 定義 | `pipeline.remove_on_complete`(真偽値。省略時 false。真偽値でなければ定義の誤り 64) |
| 実効値 | `--remove-on-complete` があれば true、なければ定義の値。環境変数では切り替えない。オプションで false にする手段はない |

完了時の手順: 完了のステートを `Durable` で書く → 実効値が真なら `reset_state`(run ID に応じたファイルを削除。存在しなくても成功)→ 成功なら 0(`run` ではジョブの終了コード)。削除に失敗したら 65(完了のステートは `completed = true` のまま残る。`watch` は完了として表示する)。

`run` では、完了の契機が「スキップの記録」のときも同様に、書き込みの後に削除する。

---

## 10. 書き込みと読み込みの併用シナリオ(確認用)

再実装の確認に使える、ファイルの状態遷移の例(定義 `a → b → c`、遷移なし、run ID なし、`conductor.yaml`)。

| 操作 | ファイル `conductor.yaml.state` | 終了コード |
|---|---|---|
| `pipeline`(b が exit 3 でエラー) | §4.4 の例(1)と同じ内容(`pipeline_name` は定義の名前) | 3 |
| `pipeline` を再実行(a は AlreadyExecuted、b・c 成功) | 全ジョブ success、`completed = true`、`resume_job_id` 無し(残る) | 0 |
| もう一度 `pipeline` | 完了のステートは空とみなされ、初期ステートで上書きされ、先頭から実行 | ジョブによる |
| ファイルの `format_version` を `2` に書き換えて `run a` | 変更なし。`state file version is not supported` | 65 |
| `reset` | ファイルを削除(無くても成功) | 0 |

---

## 11. 並行実行・排他・持続性

- **排他制御はしない**(ファイルロック、PID ファイルなし)。同じ定義(同じ run ID)で複数の conductor を同時に動かすと、それぞれが読んで書き、後に書いた方が勝つ。書き込み自体は原子的なので、ファイルが壊れることはないが、記録は失われうる。同時の実行は避けること。
- 別の実行単位(run)で同じ定義を並行して使うときは、run ID でステートを分ける(§1)。run ID ごとに独立したファイルと再開位置を持つ。
- **at-least-once**: ジョブの終了と、結果の書き込みのあいだに conductor が落ちると、ジョブは未実行として扱われ、再実行されうる。
- **リトライの回数はステートに書かない**(書式に項目がない)。再開は、そのジョブを最初の実行から数え直す。
- ステートは、定義ファイルのあるディレクトリに作られる。書き込めないディレクトリでは、最初のジョブの前に 65 で止まる(初期ステートが書けない)。
- 古い run のステートは自動では消えない。`reset --all` か、`--remove-on-complete` で片づける。
- 書き込みは同期の関数で、1 つの `run`・`pipeline` の中では常に直列に行う(スレッドを使わない。[05](05-execution-flow.md) §7.8)。書き込みを、ジョブの終了の観測より前に行わない(「ジョブの終了後に書く」が at-least-once の前提)。

---

## 12. 再現すべき特記事項

| # | 項目 | 挙動 |
|---|---|---|
| 1 | `reset --all` と長さ 0 のファイル | 長さ 0 の `<定義>.<run-id>.state` は、消さず、警告にも出さない |
| 2 | `reset`(個別)とシンボリックリンク | 個別の `reset` は、リンク自体を削除する。`reset --all` は、シンボリックリンクを消さない |
| 3 | 理由文の言語 | 理由(例: `JSON として読めません`)は、ロケールに関わらず日本語の固定文。種類の文と案内はロケールに従う |
| 4 | `definition_key` | 形式に残っているが、`definition_path` と同じ値で、照合に使わない |
| 5 | 版の先行検査 | `format_version` は、未知の項目の検査より先に調べる |
| 6 | キーの大文字小文字 | 区別する。`Format_Version` や `ID` は未知のキー(または必須のキーが無い)として扱う |
| 7 | `null` | どの項目の `null` も型の誤り。「無い」とは扱わない |
| 8 | BOM | 拒否する(`"utf-8"` で復号し、`read_json` に `str` を渡す。§6.1.2、§6.5) |
| 9 | 巨大な整数 | `format_version` と `exit_code` は、木の型(`IntNode`・`HugeIntNode`)で判定する。巨大な字句で `int()` を呼ばない(3.11 以降の 4300 桁の上限にも当たらない) |
| 10 | ディレクトリを指すステートのパス | `reset` は消さず `removeFailed`(`os.unlink` を使う。§8.1) |
| 11 | `bool` | `true` / `false` は整数でない。`exit_code` や `format_version` に受理しない。書き出しでも `exit_code` に `bool` を出さない |
| 12 | `NaN` / `Infinity` | 拒否する(自前の読み取りが拒否する。標準の `json` の既定は受理) |
| 13 | 孤立サロゲート | 読み込みでは拒否する(6a)。書き出しでは、`definition_path` / `definition_key` の孤立サロゲートを U+FFFD に置き換える |
| 14 | `exit_code` の `0` | 「ある」。`is not None` で判定する(§4.2) |

---

## 13. 型の骨格・実装に任せてよい点・未確認

### 13.1 エラーの型(骨格)

```python
StateErrorKind: TypeAlias = Literal[
    "unreadable", "tooLarge", "corrupt", "versionMismatch", "writeFailed", "removeFailed",
]

# 内部の判別用(表示に使わない)。reset --all が「中身の run_id が名前と合わない」を見分けるために使う(§8.2)。
StateErrorDetail: TypeAlias = Literal["runIdMismatch"]

class StateError(Exception):
    """パスと理由(固定の日本語文)を持つ。入力の値や下位の例外の文を、メッセージにも理由にも含めない。"""
    kind: StateErrorKind
    path: str
    reason: str                       # §6.4 の固定文
    detail: StateErrorDetail | None
    def __init__(
        self, kind: StateErrorKind, path: str, reason: str,
        detail: StateErrorDetail | None = None,
    ) -> None: ...                    # super().__init__(reason)。下位の例外は raise ... from e で __cause__ に入れる(表示しない)

# 表示用の組み立て(種類の文・案内・ロケール)は conductor/report/ が行う。
```

読み書きの関数の署名:

```python
def read_state(definition_path: str, run_id: str | None) -> PipelineState | None: ...       # 空は None。完了は None(14)
def read_state_raw(definition_path: str, run_id: str | None) -> PipelineState | None: ...   # 14 を行わない
Durability: TypeAlias = Literal["durable", "fast"]      # 本文の Durable / Fast。durable は内容の同期まで行う

def write_state(
    definition_path: str, run_id: str | None, state: PipelineState, durability: Durability,
) -> None: ...
def reset_state(definition_path: str, run_id: str | None) -> None: ...                        # 無くても成功
```

### 13.2 実装に任せてよい点

- 一時ファイルの名前の細部(§1 の条件を満たすこと)、再試行の回数の上限の具体値は 16 を推奨。
- 内部の構造(書き出しの関数の分け方、読み込みの内部の作りなど)。ただし、キーの順序・インデント・末尾の改行・文字のエスケープ(§4)、読み込みの規則(§6.5)は固定。
- ファイルの権限(§5。`0666` から umask を引いた OS 標準。`tempfile` の `0600` にしない)。

### 13.3 確認済み(CPython 3.9.6・3.14.10・3.15.2、macOS。特記のない限り 3 つで同じ)

- `json`(書き出しの `dumps` に使う分と、採らなかった案の根拠): 重複キーの後のものが勝つこと、`NaN`・`Infinity` の受理、整数・`float`・`bool` の区別(`parse_int` / `parse_float` のフックに字句の文字列が渡ること)、3.14 での 4300 桁の上限(3.9.6 には無い)、孤立サロゲートのエスケープの受理、BOM(`str`)の拒否と `bytes` での受理・UTF-16 の自動判定、空白・制御文字・数値の字句の拒否、深い入れ子の挙動(上の表 5)、`JSONDecodeError` の `pos`・`lineno`・`colno`、`dumps` の `ensure_ascii=False`(U+2028・U+0085・DEL・`<&>` が生のまま)、`indent=2` の形(空配列 `[]`、行末に空白なし)、挿入順のままのキー、`True` が `true` になること、`allow_nan=False` の `ValueError`、孤立サロゲートの生出力と `encode` の失敗。
- 文字列・復号: `surrogateescape` での復号と、`surrogateescape` + `decode("utf-8", "replace")` による U+FFFD への置き換え、`"utf-8"` の厳密な復号が `ED A0 80`・`C0 80`・`F4 90 80 80`・`E3 81` を拒否し BOM を残すこと(`"utf-8-sig"` は取り除く)、`str` の比較がコードポイントの順(UTF-8 の順と同じ)であること。
- ファイル: `os.open` の `O_EXCL`(`FileExistsError`)、権限 `0o666` から `0o644`(umask 022)、`tempfile.mkstemp` の `0o600`、置き換えで権限を引き継がないこと、`os.replace` のディレクトリへの `IsADirectoryError`、`os.unlink` のディレクトリへの `PermissionError`(macOS)、存在しないパスの `FileNotFoundError`、親がファイルのときの `NotADirectoryError`、ディレクトリを `open` したときの `IsADirectoryError`、ディレクトリの `os.fsync`、`os.get_inheritable` が `False`、`os.scandir` の `is_file(follow_symlinks=False)` でシンボリックリンクを除けること、シンボリックリンクの `os.unlink` がリンク先を残すこと、`/dev/zero` から `read(上限 + 1)` で止まること、書き込み失敗時に一時ファイルが残らないこと、24 時間以上古い一時ファイルが `Durable` の書き込みで消え `Fast` では残ること。

### 13.4 未確認

- サポートする最小の版と最新の版(実装時に確認)での、上記の挙動の再確認。この確認は上の 3 つの版だけで行った。特に、4300 桁の上限(3.11 以降)と、深い入れ子の挙動は、版で差があった。
- Windows での `os.replace` による置き換え(既存ファイルへの置き換えの成否と原子性、他のプロセスが開いているときの `PermissionError`)、`os.O_BINARY` が必要なこと、`ntpath` でのパスの扱い、ディレクトリへの `os.unlink` の挙動、ディレクトリを開いたときの `unreadable` の判定、ディレクトリの `fsync` の可否、ファイルの権限(ACL)。
- Linux で、ディレクトリへの `os.unlink` が `IsADirectoryError`(`EISDIR`)になること、ディレクトリを開いて読んだときの `unreadable` の判定。
- 名前付きパイプ(FIFO)を `open` したときの挙動(想定外の入力として扱わない)。
- 引数のパスに UTF-8 として不正なバイト列があるとき、`surrogateescape` で `str` に入ること(macOS で確認)。Linux と Windows(Windows は UTF-16 のパスで、孤立サロゲートが別の形で入りうる)は未確認。
- 一時ファイルの `<時刻>` は、`time.time_ns()`(実時計のナノ秒)。クロックが戻った場合の挙動は、名前の衝突の再試行(最大 16 回)に任せている。
- §4.4 の例 4 のステート値は、規則からの導出で、実行の流れの中で書かれた出力としては確認していない(書き出しの形式は動かして確認した)。
