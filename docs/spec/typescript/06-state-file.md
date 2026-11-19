# 06. ステートファイル(論理項目・JSON の形式・読み書き・整合性・reset)

対象: TypeScript(Node.js)で実装する `conductor` v0.8.0 と同じ挙動の実行ツール。ステートファイルは、あるパイプライン定義の実行の途中経過(どのジョブがどう終わったか、どこから再開するか)を保存するファイル。実行の意味論(いつ何を記録するか)は [05-execution-flow.md](05-execution-flow.md) にある。この文書は、ステートが持つ**論理的な項目**、保存の**必須の性質**、**JSON の形式**、読み書きの手順、異常系の挙動を定める。

ファイル名は `<定義ファイル名>.state` / `<定義ファイル名>.<run ID>.state`(定義ファイルの拡張子を含む)で、中身は **JSON**(UTF-8、BOM なし)。実装は `src/state/`(読み書き・整合性検査・reset)。ステート値の型(`JobState`、`PipelineState` など)は `src/flow/` と共有する([05](05-execution-flow.md) §1.1)。

この文書の確認の範囲: 「Node.js の挙動」と書いた点のうち「確認済み」と明記したものは、Node.js v26.13.0(macOS)で実際に動かして確かめた。Windows と、ほかの版は未確認(§13)。

## 0. この文書の読み方

| 区分 | 内容 |
|---|---|
| 必須(どの実装も同じ) | §2 の論理項目・値の規則・矛盾の条件、§3 の保存の性質、§4 の JSON の形式(キー名・型・固定の順序)、§6 の読み込みの検査(順序・種類・理由文・案内文)、§7 の整合性検査、§8 の reset の挙動、ファイルの場所と名前(§1) |
| 実装に任せてよい点 | 一時ファイルの名前の細部(§1 の条件を満たす)、内部の構造、パーサーの内部の作り(§6.5 の規則を満たす)。書き出しのインデントなどは §4.3 に固定する(読み込みは空白に依存しない) |

---

## 1. 場所と名前

ステートファイルは、**定義ファイルと同じディレクトリ**に置く。

| run ID | ファイル名 |
|---|---|
| なし | `<定義ファイル名>.state` |
| あり | `<定義ファイル名>.<run-id>.state` |

- `<定義ファイル名>` は、定義ファイルのパスの最後の要素(拡張子を含む。`path.basename(defPath)`)。例: `conductor.yaml` → `conductor.yaml.state`、`--config ci/nightly.json --run-id 12345` → `ci/nightly.json.12345.state`。
- 定義ファイルのパスは、`--config` で与えられた文字列のまま(正規化・絶対化しない)。ディレクトリ部分は `path.dirname(defPath)`(親が無い `conductor.yaml` のようなパスでは `"."`、つまり実行ディレクトリ)。ステートのパスは `path.join(path.dirname(defPath), name)`(確認済み: `path.join(path.dirname("c.yaml"), "c.yaml.state")` は `c.yaml.state`、`path.dirname("ci/n.json")` は `ci`)。標準エラーに出す `<ステートファイルのパス>` は、この結果の文字列。
- Windows では、`path` は `path.win32` の規則(`\` と `/` の両方を区切りとみなす)で動く。この仕様は、Windows でのパスの扱いを確認していない(§13)。
- run ID は検査済み(`A-Za-z0-9._-`、1〜64 文字、先頭が `.` でない。[05](05-execution-flow.md) §6)なので、パス区切りや `..` は入らない。
- 一時ファイルの名前: `.<ステートファイル名>.tmp.<プロセス番号>.<時刻>.<連番>`(同じディレクトリ。§5)。名前の付け方は実装に任せてよいが、**ドットで始まり、`.tmp.` を含み、後始末(§5 の 9)で識別できる**こと。
- 大文字小文字を区別しないファイルシステム(macOS の既定、Windows)では、`A` と `a` の run ID は同じファイルになる。食い違いは、中身の `run_id` との照合で壊れたステート(65)として検出される(§6 の 9)。

---

## 2. ステートの論理項目

### 2.1 ステート全体の項目

メモリ上の名前(TypeScript。`PipelineState` のプロパティ)と、ファイル上のキー名の対応を併記する。

| 項目(ファイルのキー) | メモリ上の名前 | 型 | 必須 | 意味・値 |
|---|---|---|---|---|
| `format_version` | (書き出し時に固定値 `1` を出す) | 整数 | 必須 | 常に `1`。読み込みでは最初に検査される(§6) |
| `definition_key` | `definitionKey` | 文字列 | 必須 | 定義ファイルのパス(`definition_path` と**同じ文字列**を書く。読むときは別々に保持するが、どちらも照合に使わない) |
| `definition_path` | `definitionPath` | 文字列 | 必須 | 定義ファイルのパス(`--config` の文字列。文字列として表せない部分(不正な UTF-8)は、Node.js が引数を文字列にする時点で代替文字 U+FFFD に置換される。未確認。§13)。空でもよい |
| `pipeline_name` | `pipelineName` | 文字列 | 必須 | 記録時の `pipeline.name`。照合に使う(§7) |
| `job_order` | `jobOrder` | 文字列の列 | 必須 | 記録時のジョブIDを、定義の記述順に並べたもの |
| `job` | `jobs` | ジョブ状態の列 | 必須 | ジョブごとの状態(§2.2)。`job_order` と同じ順・同じ数 |
| `resume_job_id` | `resumeJobId` | 文字列 | 任意 | 次に実行する、または再開するジョブのID。無いときは項目自体を保存しない。`job_order` に含まれる値 |
| `end_reached` | `endReached` | 真偽 | 必須 | `@end` に到達したか。一度 true になったら、その run のあいだ false に戻らない |
| `completed` | `completed` | 真偽 | 必須 | 完了したか(§9) |
| `run_id` | `runId` | 文字列 | 任意 | run ID。run ID ありのステートだけが保存する。無いステートには項目を保存しない |

上の表にない項目は、ステート全体にも、ジョブ状態の中にも許さない(§6 の `corrupt`)。

### 2.2 ジョブ状態の項目

| 項目(ファイルのキー) | メモリ上の名前 | 型 | 必須 | 意味・値 |
|---|---|---|---|---|
| `id` | `id` | 文字列 | 必須 | ジョブID |
| `status` | `status` | 文字列 | 必須 | `"pending"` / `"success"` / `"warning"` / `"error"` / `"skipped"`(これ以外は `corrupt`) |
| `exit_code` | `exitCode` | 整数(32 ビット符号付き整数の範囲: -2147483648〜2147483647) | 任意 | ジョブの終了コード。`status` が success / warning / error のときだけ持てる。終了コードが無い結果(起動失敗・シグナル終了)の error には保存しない。負数も許す(例 `-1`)。範囲外の整数は `corrupt` |
| `skip_reason` | `skipReason` | 文字列 | 条件付き | `"byTransition"`(遷移によるスキップ)/ `"byEnd"`(`@end` によるスキップ)。**`status = "skipped"` のときだけ必須、そのほかの `status` では保存してはならない** |

### 2.3 内容の矛盾(読み込み時に `corrupt` になる条件)

- ジョブ状態のジョブID に重複がある。
- ジョブ状態の並び(`id` の列)が `job_order` と、数も順序も完全には一致しない。
- `resume_job_id` が `job_order` にない。
- `resume_job_id` があるのに `completed = true`(完了と再開位置は両立しない)。
- `status` が `skipped` なのに `skip_reason` が無い、または `skipped` 以外なのに `skip_reason` がある。
- `status` が pending / skipped なのに `exit_code` がある。

### 2.4 初期ステート(`initialState`)

定義から作る、記録前のステート: 全ジョブ `pending`(`exitCode`・`skipReason` なし)、`resumeJobId` 無し、`endReached = false`、`completed = false`、`definitionKey` = `definitionPath` = 定義ファイルのパス文字列、`pipelineName` = 定義の名前、`jobOrder` = 定義のジョブIDの記述順、`runId` = 決定した run ID(あれば)。

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

書き出し用の型の骨格(読み込みには使わない。理由は §6.5):

```ts
// 書き出し専用。プロパティの「書く順」が、そのままキーの出力順になる(§4.2 の順序)。
interface JobFile {
  id: string;
  status: JobStatus;
  exit_code?: number;      // 値 0 も出力する(undefined のときだけ省く。`if (x !== undefined)` で判定し、真偽値の判定を使わない)
  skip_reason?: SkipReason;
}

interface StateFile {
  format_version: 1;
  definition_key: string;
  definition_path: string;
  pipeline_name: string;
  run_id?: string;
  job_order: string[];
  job: JobFile[];
  resume_job_id?: string;
  end_reached: boolean;
  completed: boolean;
}

export function serializeState(state: PipelineState): string; // §4.3 の規則で文字列にする(末尾に改行 1 つ)
```

- 出力のオブジェクトは、キーを §4.2 の順に 1 つずつ代入して作る(`run_id` などの省略可能なキーは、値があるときだけ、その順序の位置で代入する)。`JSON.stringify` は、**オブジェクトの挿入順**でキーを出力する(確認済み。ただし「整数として読めるキー名(`"1"`、`"2"` など)」は、挿入順に関わらず、先頭に昇順で並ぶ。確認済み: `{"b":1,"2":1,"a":1,"1":1}` のキーの順は `1, 2, b, a`)。このステートのキー名(`format_version` など)は整数として読めないので、挿入順どおりに出る。
- `undefined` の値を持つキーは、`JSON.stringify` が出力しない(省略される)が、この仕様は**キーを代入しない**書き方で省略を表す(`exit_code: undefined` のようなコードは書かない。`exactOptionalPropertyTypes` を有効にすると、型で防げる)。
- キーの順序を、単体テストで固定する(§4.4 の例との完全一致。§6.6)。

### 4.3 書き出しの規則

- 文字列 = `JSON.stringify(stateFile, null, 2) + "\n"`。インデントは空白 2 つ、末尾の改行は LF 1 つ(CRLF にしない)。ファイルの末尾は改行 1 つ。
- `JSON.stringify` は `<` `>` `&` をエスケープしない(確認済み。HTML 向けの特別扱いは無い)。
- 配列は、要素ごとに 1 行(インデントされる)。空配列は `[]`(確認済み)。
- 数値は 10 進の整数だけ(`1`、`0`、`-1`、`2147483647`)。小数点・指数は書かない(`exit_code` は 32 ビット符号付き整数で、`-0` は保持しない。`JSON.stringify({a: -0})` は `{"a":0}` を出す。確認済み)。
- 文字列の表記は `JSON.stringify` の出力のまま。ただし次の 1 点だけ後処理する: 出力に含まれる U+2028・U+2029 を、それぞれ 6 文字の文字列 `\u2028`・`\u2029`(バックスラッシュ + `u` + 4 桁の 16 進)に置換する(`JSON.stringify` は、この 2 文字をそのまま(エスケープせず)出力する。確認済み)。どちらも JSON としては有効だが、古い JavaScript の構文解析との互換のための決定で、この置換をしても読み込み側(§6.5 の自前のパーサー)は同じ値を得る。
- 単独のサロゲート(対のない `\uD800` など)は、`JSON.stringify` が `\ud800` の形にエスケープして出力する(確認済み。Node.js v26.13.0)。ステートの文字列は、引数(U+FFFD に置換済み)と定義の文字列から来るので、単独のサロゲートは通常は現れない。読み込み側は、このエスケープを U+FFFD に置き換える(§6.5)ので、往復で値が変わる。ステートを書くのは conductor だけなので、実害は無いと判断した。
- 制御文字・`"`・`\` はエスケープされる(`\n`、`\t`、`\"`、`\\`、その他の制御文字は `\u00XX`)。読み込みは JSON として正しければどの表記も受け付ける。

### 4.4 実ファイルの例(全フィールドを含む)

以下は §4.3 の規則で出力される内容。例 6・7・8 は、§4.2 の順序のシリアライザを Node.js v26.13.0 で実際に動かした出力。例 1〜5 は、同じ規則から書き起こしたもの(実装の出力と一致することを、実装時に単体テストで確認する。§13)。

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

(`resume_job_id` が付くかどうかは書き込みの契機による。遷移のスキップは結果の記録([05](05-execution-flow.md) §3.3)で書かれ、その `resume_job_id` は無し。この例は規則から導いたもので、実装の出力そのものを確認した例ではない。)

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
  "pipeline_name": "x\u2028y",
  "job_order": [
    "a\u2029b <&>"
  ],
  "job": [
    {
      "id": "a\u2029b <&>",
      "status": "pending"
    }
  ],
  "end_reached": false,
  "completed": false
}
```

---

## 5. 書き込み(`writeState`)の手順と原子性

入力: 定義ファイルのパス、run ID、ステート、耐久性(Durable / Fast)。`async` 関数で、失敗は `StateError`(kind = `writeFailed`)を投げる(型は §6.2)。

```text
1. data = serializeState(state)        # §4.3。失敗(通常は起きない)なら WriteFailed(理由=「内容を JSON にできません」)
2. path = ステートファイルの場所(§1)
3. 一時ファイルを、同じディレクトリに、「既存なら失敗する」新規作成で作る
     fh = await fs.open(tmpPath, "wx", 0o666)       # フラグ "wx" = 書き込み・新規作成・既存なら失敗(確認済み: 既存なら code = "EEXIST")
     名前 = ".<ステートファイル名>.tmp.<process.pid>.<時刻>.<連番>"
     <時刻> = BigInt(Date.now()) * 1000000n(ナノ秒単位の整数にそろえた 10 進の文字列。ミリ秒の精度でよい)
     名前が既存(err.code === "EEXIST")なら、連番を進めて作り直す。最大 16 回。それでも作れなければ WriteFailed
     連番 = モジュール内の通し番号(let seq = 0; seq += 1)+ 試行回数(桁あふれは考えなくてよい)
4. 全内容を書く: await fh.writeFile(Buffer.from(data, "utf8"))
     (FileHandle.writeFile は、1 回の書き込みで書き切れないときに続きを書く、と理解している。確認していない。§13。
      ユーザー空間のバッファは挟まないので、フラッシュの手順は要らない)
5. Durable のときだけ、await fh.sync() で内容を記憶装置まで同期する
6. await fh.close()(Windows は、開いたままだと移せない・消せない。close の失敗も失敗として扱う)
7. await fs.rename(tmpPath, path) で置き換える
8. 3〜7 のどこかで失敗したら: ハンドルが開いたままなら close(失敗は無視)、自分が作った一時ファイルを fs.unlink(失敗は無視)し、
     WriteFailed(理由=「書き込みに失敗しました」)。下位のエラーは StateError の cause に入れ、表示しない
9. 成功し、かつ Durable のとき:
     UNIX 系では、ディレクトリを fs.open(dir, "r") で開いて await dh.sync()、close(失敗は無視。Windows(process.platform === "win32")では何もしない)
     同じ名前の型の一時ファイルのうち、更新から 24 時間以上たった通常のファイルを消す(失敗は無視)
```

Node.js の API の選定:

- **一時ファイルの名前と権限を自分で制御する**ために、`fs.open` の `"wx"`(`O_EXCL`)と `0o666` を使う。標準の一時ファイル作成の関数(`fs.mkdtemp` など)は、ディレクトリを作る・名前の規則が合わない、という理由で使わない。
- パーミッションは明示的に設定しない(`fs.chmod` しない)。**確認済み(macOS)**: `fs.open(path, "wx", 0o666)` で作ったファイルの `stat().mode` の下位は 8 進の `100644`(`-rw-r--r--`。umask 022 の環境)。既存のステートファイルを置き換えるときも、権限は引き継がず、新しい一時ファイルの権限になる。所有者の変更・ACL の操作はしない。Windows では、`mode` はほぼ意味を持たない(読み取り専用の属性だけ)と理解している(未確認)。
- 時刻は `Date.now()`(ミリ秒。実時計)でよい。クロックが戻っても、名前の衝突は連番の再試行(最大 16 回)で避ける。

性質:

- **原子的**: 読み手は、直前の完全な内容か、新しい完全な内容のどちらかだけを見る。途中の内容は見えない。どの段階で失敗しても、ステートファイルは直前の完全な内容のまま。
- 置き換え先がディレクトリなど、名前の付け替えが失敗するときは `WriteFailed`。ステートは変わらず、一時ファイルも残らない(確認済み: `fs.rename(file, directory)` は `EISDIR` で失敗)。書けないディレクトリも `WriteFailed`。
- 残った一時ファイル(プロセスが落ちたとき)は、読み込みに影響しない。`Durable` の書き込みの成功後に、24 時間以上古いものだけ消える。`Fast` では消さない。
- 「同じ名前の型」= 名前が `.<ステートファイル名>.tmp.` で始まるもの。後始末は、`fs.readdir(dir, { withFileTypes: true })` の各エントリのうち、`entry.isFile()` が真のもの(シンボリックリンク・ディレクトリを除く。リンクを辿らない。確認済み: シンボリックリンクは `isFile()` が偽、`isSymbolicLink()` が真)で、`fs.lstat` の `mtimeMs` が `Date.now() - 24 * 60 * 60 * 1000` 以前のものを `fs.unlink` する(エラーは無視)。ファイルシステムによっては `readdir` が種別を返さない場合の挙動は未確認(§13)。
- 排他制御(ファイルロック)はしない(§11)。

### 5.1 Windows での置き換えの注意

- `fs.rename` が、置き換え先が既にあっても上書きするかは、Windows での確認をしていない(libuv が置き換えの指定つきの移動の API を使うと理解している。**未確認**。Windows の CI のテストで、既存のファイルへの `fs.rename` が成功することを確認する)。macOS では、既存のファイルへの置き換えが成功する前提で書いている(`fs.rename(file, directory)` が `EISDIR` で失敗することだけを確認した)。
- 一時ファイルは、`rename` の前に `close` する(開いたままだと移せない)。
- 置き換え先のステートファイルを、ほかのプロセス(例: 同時に走る `watch`)が開いている間は、Windows では `rename` が `EPERM`・`EBUSY`・`EACCES` で失敗しうる(**未確認**)。失敗したら `WriteFailed`(65)にする。再試行はしない(仕様として、読み手との同時の実行は想定しない。§11)。
- ディレクトリの同期は行わない(Windows ではディレクトリを開いて `sync` できないと理解している。未確認)。
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

## 6. 読み込み(`readState` / `readStateRaw`)の手順と異常系

### 6.1 手順

入力: 定義ファイルのパス、run ID(ファイル名側の期待値)。上から順に、最初に当てはまる結果で終わる。`readState` は完了のステートを「空」にし、`readStateRaw`(`watch` が使う)は 14 を行わない。どちらも `async` 関数で、空のときは `undefined` を返し、失敗は `StateError` を投げる(§6.2)。

| 順 | 処理 | 結果 |
|---|---|---|
| 1 | `fs.open(path, "r")` | 存在しない(`err.code === "ENOENT"`。確認済み)→ **空**(ステート無し)。その他の開く失敗(権限など。`ENOTDIR` を含む)→ `unreadable` |
| 2 | 開いたハンドルの `await fh.stat()` の `size` を確認 | `stat` の失敗 → `unreadable`。1 MiB(1,048,576 バイト)**超** → `tooLarge` |
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
| 14 | `readState` のみ: `completed = true` | **空**(完了のまま残ったステート。ファイルは消さない) |

#### 6.1.1 順 3 の読み取り

`Buffer.allocUnsafe(1048576 + 1)`(または `Buffer.alloc`)を用意し、`fh.read(buffer, offset, length, null)` を、`bytesRead` が 0 になる(終端)か、バッファが満杯(上限 + 1 バイト)になるまで繰り返す。位置の引数には **`null`**(現在位置から読む)を渡す。**確認済み(macOS)**: 位置に `0` を渡すと、名前付きパイプ(FIFO)では `ESPIPE`(`unreadable`)になるが、`null` なら、無限に読める特殊ファイル(`/dev/zero`)から読み取れ、上限で止められる。`fs.readFile` は、上限なしで全部を読むので使わない。読み終わったら、`finally` で `fh.close()` する。

#### 6.1.2 順 5 のデコード

```ts
const decoder = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true });
const text = decoder.decode(bytes); // 不正なバイト列なら TypeError(code = "ERR_ENCODING_INVALID_ENCODED_DATA")
```

- **`ignoreBOM: true` を必ず指定する**。既定(`false`)では、`TextDecoder` は先頭の BOM(`EF BB BF`)を**黙って取り除く**。**確認済み**: 既定では `EF BB BF 7B 7D` が `"{}"` になり、`ignoreBOM: true` なら先頭に U+FEFF が残る。BOM を取り除くと、BOM 付きのファイルが正しい JSON として受理されてしまい、仕様(BOM は壊れたステート。§6.1 の 6a)と食い違う。
- `fatal: true` により、不正な UTF-8 は例外になる。**確認済み**: `ED A0 80`(サロゲートの符号化)、`C0 80`(最短でない符号化)、`F4 90 80 80`(U+10FFFF 超)、途中で切れた `E3 81` は、すべて拒否された。例外は握りつぶさず、`corrupt`(`UTF-8 の文字列ではありません`)に変換する。
- `utf8.Valid` に当たる別の検査は要らない(上のデコードが兼ねる)。NUL は UTF-8 としては有効で、JSON の構文の検査(6a)で拒否される(文字列の外に現れれば構文の誤り、文字列の中に現れれば制御文字の誤り)。

順 6 の細目(この順。最初の失敗で終わる):

| 細目 | 処理 | 失敗の理由 |
|---|---|---|
| 6a | §6.5 の自前のパーサーで、全体を構文解析する(木を作る)。失敗するもの: 構文の誤り、BOM(先頭の U+FEFF)、値の後ろの余分な文字(2 つ目の値を含む)、空白だけの内容、NUL を含む制御文字(U+0000〜U+001F)の文字列内への直接の混入、`\u` エスケープの誤り、入れ子が深すぎるもの(§6.5 の深さの上限 32 を超える) | `JSON として読めません` |
| 6b | 最上位の値がオブジェクト(`{`)か(`null`、`[`、数値、文字列、真偽の最上位を拒否する) | `JSON として読めません` |
| 6c | どのオブジェクトにも、同じキーが 2 回以上現れないか(最上位と `job` の各要素を含む全オブジェクトを走査する。エスケープを展開した後の文字列で比べる。`"id"` と `"id"` は同じキー) | `キーが重複しています` |

6a が 6c より先であることは、仕様として固定する(重複キーが先にあっても、後ろに構文の誤りがあれば、`JSON として読めません` になる)。空白の定義は JSON の空白(スペース、`\t`、`\n`、`\r`)。BOM は空白でないので、6a で拒否される(書き出しは BOM を付けない。他のツールが BOM 付きで保存した場合は壊れたステートとして 65 になり、`reset` で回復する)。

順 7 の「整数」の定義: 値の字句が `-?(0|[1-9][0-9]*)` に一致するもの(小数点・指数を含まない)。`1.0`、`1e0`、`"1"`、`true`、`null`、配列、オブジェクトは**整数以外**(`format_version が整数ではありません`)。`-0` と `0`、`2`、`-1`、`99999999999999999999` は整数で 1 ではないので `versionMismatch`。`1` だけが受理される(`01` は JSON の構文として誤り(6a))。整数の判定と比較は**字句**に対して行い、`number` への変換(丸め)を経由しない(§6.5)。

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
| h | `exit_code` があり、整数でない(`1.0`、`1e2`、文字列、`null` を含む)、または 32 ビット符号付き整数の範囲外(§6.5 の字句による判定) | `値の型が誤っています` |
| i | `skip_reason` があり、文字列でない(`null` を含む) | `値の型が誤っています` |
| j | `skip_reason` が 2 つの値以外 | `skip_reason の値が誤っています` |
| k | `status = skipped` と `skip_reason` の有無の不整合 | `skip_reason が status と合いません` |
| l | `status` が pending / skipped なのに `exit_code` あり | `exit_code が status と合いません` |

- `readStateRaw`(`watch` が使う)は、14 を行わない(完了のステートもそのまま返す)。1〜13 は同じ。
- 検査の `run_id` 照合(9)は、`completed` の判定(14)より前なので、完了のステートでも run ID の食い違いは `corrupt`。
- 読み取りでは、ファイルを**消さない・書き換えない**。
- 「空」= ステートが無いのと同じ扱い。呼び出し側は、すべて pending のステートとして扱い、最初の `Run` の前に初期ステートを書く。
- 空のステートを返す関数の戻り値は `undefined`(`Promise<PipelineState | undefined>`)。
- ディレクトリに対する `fs.open(path, "r")` は、UNIX 系では成功し、順 3 の読み取りで `EISDIR` で失敗して `unreadable` になる(確認済み: macOS で、ディレクトリを開く・`stat` する(`isDirectory()` は真)・`read` すると `EISDIR`)。Linux と Windows は未確認(§13)。

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

TypeScript での表現(骨格):

```ts
export type StateErrorKind =
  | "unreadable" | "tooLarge" | "corrupt" | "versionMismatch" | "writeFailed" | "removeFailed";

// 内部の判別用(表示に使わない)。reset --all が「中身の run_id が名前と合わない」を見分けるために使う(§8.2)。
export type StateErrorDetail = "runIdMismatch";

// パスと理由(固定の日本語文)を持つ。入力の値や下位のエラー文を、message にも理由にも含めない。
export class StateError extends Error {
  constructor(
    readonly kind: StateErrorKind,
    readonly path: string,
    readonly reason: string,          // §6.4 の固定文
    readonly detail?: StateErrorDetail,
    options?: { cause?: unknown },    // 下位のエラー(NodeJS.ErrnoException など)。表示しない
  ) {
    super(reason, options);
    this.name = "StateError";
  }
}
// 表示用の組み立て(種類の文・案内・ロケール)は src/report/ が行う。
```

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
| 1 | unreadable | `開く、または読み取ることができません` | 1〜3 | 開く・`stat`・読み取りの失敗(存在しないを除く) |
| 2 | tooLarge | `大きさが上限(1 MiB)を超えています` | 2、3 | 1,048,576 バイト超 |
| 3 | corrupt | `UTF-8 の文字列ではありません` | 5 | UTF-8 として不正 |
| 4 | corrupt | `JSON として読めません` | 6a、6b | JSON の構文の誤り、BOM、余分な文字、最上位がオブジェクトでない、空白だけの内容、入れ子の深さの超過 |
| 5 | corrupt | `キーが重複しています` | 6c | 同じキーが 2 回以上(最上位・`job` の要素を含む全オブジェクト。`JSON.parse` は重複を検出しないため、独立した理由にした) |
| 6 | corrupt | `format_version がありません` | 7 | キー無し |
| 7 | corrupt | `format_version が整数ではありません` | 7 | 整数の字句でない(`1.0`、`"1"`、`null` など。整数の定義は §6.1) |
| 8 | versionMismatch | `対応していない版です(reset が必要です)` | 7 | 整数で 1 以外 |
| 9 | corrupt | `知らないキーがあります` | 8、11b | 最上位、または `job` の要素に未知のキー(大文字小文字を区別) |
| 10 | corrupt | `値の型が誤っています` | 9、10、11、12 | 型の誤り(`null` を含む。任意の項目でも「無い」とは扱わない)、`exit_code` の範囲外・小数 |
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
| 22 | writeFailed | `内容を JSON にできません` | 書き込み | JSON への変換の失敗 |
| 23 | removeFailed | `削除に失敗しました` | reset | 「存在しない」以外の削除の失敗 |

- キー名は大文字小文字を区別する(`Format_Version`、`ID` は未知のキー、または必須のキーが無い、として扱う)。
- 理由文の言語は、ロケールに関わらず日本語の固定文である(`conductor: error: ...` の行の中で、種類の文と案内だけがロケールに従う)。

### 6.5 JSON の読み込み: 自前のパーサー(選定)

目的: 順序が固定の 14 段階の検査と、理由文の区別を、確実に満たす。

**決定: ステートの JSON は、定義の読み込み(JSON 形式の定義ファイル)と共用する自前の字句・構文解析器(`parseJson`)で読む。`JSON.parse` は読み込みに使わない。** 共用するのは、字句・構文の規則と木の形(下の型)で、置き場所は `src/definition/` 配下の 1 モジュールとし、`src/state/` はそれを import する(`state` が `definition` の下位モジュールを使うのは、依存の向き `definition` ← `state` の範囲内)。

`JSON.parse` の限界(いずれも Node.js v26.13.0 で確認済み。確認した範囲):

| # | `JSON.parse` の挙動 | この仕様への影響 |
|---|---|---|
| 1 | 重複キーを検出せず、**後のものが勝つ**(`{"a":1,"a":2}` → `{ a: 2 }`) | 6c(`キーが重複しています`)を実現できない |
| 2 | 数値を倍精度に丸める(`9007199254740993` → `9007199254740992`、`99999999999999999999` → `100000000000000000000`)。`1.0` と `1` と `1e0` はどれも `1` になり、字句の区別が失われる | 順 7 の「整数の字句」の判定(`1.0` は整数以外)、`exit_code` の範囲外の判定(巨大な整数の丸め)ができない。`-0` は `-0`(`Object.is(x, -0)` が真)になる |
| 3 | `"__proto__"` というキーは、**自分自身のプロパティ**として作られる(確認済み: `Object.keys` に `__proto__` が含まれ、プロトタイプは変わらない)。ところが、通常のオブジェクトに `obj["__proto__"] = value` と**代入**すると、プロトタイプが書き換わる(確認済み: `r["__proto__"] = {x: 1}` の後、`Object.keys(r)` は空で `r.x` が `1`) | 解析した結果を別のオブジェクトへ代入でコピーする実装は、プロトタイプ汚染を起こしうる。木の中でオブジェクトを通常の `Object` として保持しない(下の方針) |
| 4 | 入れ子の深さの上限が実質なく、10 万段の配列でも解析できた(確認済み) | 深さの上限(§6.5 の 32)を適用できない |
| 5 | 構文の誤りの位置(行・列)を、安定した形で返さない(メッセージの文言は版で変わりうる。位置の有無は確認していない。**未確認**) | 定義ファイルの誤りの位置(行・列)を出せない。ステートでは位置は使わないが、パーサーを共用するため採用しない |
| 6 | `"\ud800"`(単独のサロゲートのエスケープ)を、そのまま単独のサロゲートを含む文字列にする(確認済み: `isWellFormed()` が偽) | 文字列の比較・再書き出しで値が変わりうる |
| 7 | 整数のように見えるキー(`"1"`、`"2"`)を、挿入順に関わらず先頭に昇順で並べる(確認済み) | ステートのキーでは影響しないが、変換(09)では記述順の保持が要る |
| 8 | BOM は `JSON.parse` 自身は拒否する(確認済み)。ただし、前段のデコード(`TextDecoder` の既定)が BOM を取り除くので、デコードを正しく設定しないと、BOM 付きのファイルが受理される | §6.1.2 の `ignoreBOM: true` が必須 |

`JSON.parse` の第 2 引数(reviver)に、ソースの字句を渡す機能がある版があるとの情報があるが、確認していない(**未確認**)。仮にあっても、重複キー(1)と位置(5)は解決しないので、採用しない。

パーサーが満たす規則:

| 項目 | 規則 |
|---|---|
| 入力 | UTF-8 としてデコード済みの文字列(§6.1.2。BOM が残っているので、先頭が U+FEFF なら構文の誤り) |
| 空白 | スペース、`\t`、`\n`、`\r` だけ |
| 値 | `{}`、`[]`、文字列、数値、`true`、`false`、`null`。リテラルは小文字で完全一致 |
| 文字列 | `"` で囲む。U+0000〜U+001F の直接の混入は構文の誤り。エスケープは `\"` `\\` `\/` `\b` `\f` `\n` `\r` `\t` `\uXXXX`(16 進は大文字小文字どちらでも。4 桁ちょうど)だけ。それ以外の `\x` は構文の誤り。`\uXXXX` のサロゲートの対は 1 つの文字に結合する。**対のないサロゲート(単独の上位・下位)は U+FFFD に置き換える**(決定。理由: 文字列の比較と再書き出しを安定させる。§4.3) |
| 数値 | `-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?`。先頭の `+`、先頭の `0` の連続(`01`)、`.5`、`5.` は構文の誤り。**字句(元の文字列)を保持**し、`number` に変換しない。「整数」= 小数部も指数部もない |
| 入れ子の深さ | 最上位の値を 0 として、その内側の配列・オブジェクトの入れ子を 1, 2, … と数え、**32 を超える段(33 段目)は構文の誤り**(根を数えない)。ステートの正しい内容の深さは 2(最上位のオブジェクト → `job` の配列 → 要素のオブジェクト)で、余裕がある |
| 値の後ろ | 空白以外が残っていれば構文の誤り(2 つ目の値を含む) |
| 木の形 | オブジェクトは**エントリの配列**(キーの文字列、位置、値)として保持し、`Map` や通常の `Object` に変換しない。重複キーも、エントリとして残す(6c で検出するため)。キー `__proto__` は、ほかのキーと同じエントリ(通常の文字列)にすぎない |
| 位置 | 各値・各キーに、1 始まりの行・列を付ける(定義ファイルの誤りの表示用。ステートでは使わない。数え方は [03](03-definition-format.md) に従う) |
| 失敗 | 例外を投げず、結果の型で返す(構文の誤りの種類と位置)。ステートは、失敗を `JSON として読めません` にまとめる(誤りの文を表示しない) |

型の骨格:

```ts
export type JsonValue =
  | { readonly kind: "string"; readonly value: string; readonly line: number; readonly column: number }
  | { readonly kind: "number"; readonly lexeme: string; readonly isInteger: boolean; readonly line: number; readonly column: number }
  | { readonly kind: "boolean"; readonly value: boolean; readonly line: number; readonly column: number }
  | { readonly kind: "null"; readonly line: number; readonly column: number }
  | { readonly kind: "array"; readonly items: readonly JsonValue[]; readonly line: number; readonly column: number }
  | { readonly kind: "object"; readonly entries: readonly JsonEntry[]; readonly line: number; readonly column: number };

export interface JsonEntry {
  readonly key: string;            // エスケープを展開した後の文字列
  readonly keyLine: number;
  readonly keyColumn: number;
  readonly value: JsonValue;
}

export type JsonParseResult =
  | { readonly ok: true; readonly value: JsonValue }
  | { readonly ok: false; readonly line: number; readonly column: number; readonly message: string };

export interface JsonParseOptions { readonly maxDepth: number } // 32
export function parseJson(text: string, options: JsonParseOptions): JsonParseResult;
export function findDuplicateKey(value: JsonValue): JsonEntry | undefined; // 最初に見つかった 2 回目以降のキー(全オブジェクトを走査)
```

読み込みの手順(順 6〜13 を、この木に対して行う):

1. `parseJson` が失敗 → 6a。`ok: true` でも最上位が `object` でない → 6b。
2. `findDuplicateKey` が見つかれば → 6c。
3. 最上位のオブジェクトのエントリから、キーの**完全一致**(大文字小文字を区別)で値を取り出す。取り出しのために、エントリの配列を、`Map<string, JsonValue>`(`__proto__` を普通のキーとして扱える)に変換してよい(重複は 6c で除かれているので、後のものが前のものを上書きすることはない)。通常のオブジェクト(`{}`)や `Object.create(null)` に変換しない。
4. 順 7〜12 は、仕様の順に、この `Map` から取り出す。未知のキーの検査(順 8)は、キーの全部が §2.1 の 10 個のどれかか、を見る。
5. 値の型の判定は、`kind` で行う(`string` / `boolean` / `array` / `object` / `number`。`null` は型の誤り)。`number` の整数の判定は `isInteger`(字句に小数部・指数部がない)。
6. `format_version`: 整数なら、字句を `1` と比べる(字句が `"1"` のときだけ受理。ほかの整数は `versionMismatch`)。`BigInt` や `Number` に変換する必要はない。
7. `exit_code`: 整数なら、符号を除いた桁数が 10 を超える場合は範囲外(型の誤り)。それ以外は `BigInt(lexeme)` で `-2147483648n` 以上 `2147483647n` 以下かを調べ、範囲内なら `Number(...)` に変換する(桁数の事前の確認は、極端に長い数字列を `BigInt` に渡さないため)。`-0` は、`0` として扱う(`Number(BigInt("-0"))` は `0`)。
8. `job_order` と `job` の要素も同様に取り出す(`job` の各要素がオブジェクトでなければ、順 11 の a)。

### 6.6 読み込みの単体テストで固定する入力(最低限)

| 入力(要点) | 期待 |
|---|---|
| 正常(§4.4 の例 1〜8) | 受理。往復(読んで書く)で元と一致(例 8 の U+2028・U+2029 のエスケープを含む) |
| 先頭に BOM(`EF BB BF`)+ 正常な内容 | `corrupt`(`JSON として読めません`)。`TextDecoder` に `ignoreBOM: true` を付け忘れると、受理されてしまう(このテストで検出する) |
| 正常な内容の後ろに `{}` や `x` | `corrupt`(`JSON として読めません`) |
| `null`、`[]`、`1`、`"x"`、`true` | `corrupt`(`JSON として読めません`) |
| 空白だけ(`"\n"`) | `corrupt`(`JSON として読めません`) |
| 文字列の中に制御文字(生のタブ・NUL) | `corrupt`(`JSON として読めません`) |
| 入れ子の深さが 33(`{"a":[[[...]]]}` で 33 段目) | `corrupt`(`JSON として読めません`)。深さ 32 は受理(ほかの検査で別の理由になりうるので、深さだけを変えたテスト用の入力を使う) |
| 最上位に同じキーが 2 回(`"completed": false, "completed": true`) | `corrupt`(`キーが重複しています`) |
| `job` の要素に同じキーが 2 回 | `corrupt`(`キーが重複しています`) |
| `"id"` と `"id"` が同じ要素に | `corrupt`(`キーが重複しています`) |
| 重複キーがあり、かつ後ろに構文の誤り | `corrupt`(`JSON として読めません`。6a が先) |
| `{"format_version": 2, "unknown": 1}` | `versionMismatch`(未知のキーより版が先) |
| `{"format_version": 9007199254740993}`、`{"format_version": 100000000000000000000}` | `versionMismatch`(丸めを経由しない) |
| `{"format_version": 1.0}` | `corrupt`(`format_version が整数ではありません`) |
| `{"format_version": "1"}` | `corrupt`(`format_version が整数ではありません`) |
| `{"Format_Version": 1}` | `corrupt`(`format_version がありません`) |
| `{"format_version": 1, "__proto__": {}}` | `corrupt`(`知らないキーがあります`)。プロトタイプは汚染されない(テストの後、`({}).polluted` が `undefined`) |
| `{"format_version": 1, "__proto__": 1, "__proto__": 2}` | `corrupt`(`キーが重複しています`) |
| `exit_code` が `1.5`、`2147483648`、`9007199254740993`、`"3"`、`null` | `corrupt`(`値の型が誤っています`) |
| `exit_code` が `-2147483648`、`2147483647` | 受理 |
| `resume_job_id` が `null` | `corrupt`(`値の型が誤っています`) |
| 文字列に `"\ud800"`(単独のサロゲートのエスケープ) | 受理され、値は U+FFFD(`pipeline_name` などで、その値のまま照合される) |
| ディレクトリを指すパス | `unreadable`(macOS。ほかの OS は実機で確認する) |
| 1,048,576 バイトちょうど(正常な内容を空白で埋める)/ 1,048,577 バイト | 前者は受理、後者は `tooLarge` |

---

## 7. 定義との整合性検査(`checkConsistency`)

空でない(完了していない)ステートを読んだ後、`run`・`pipeline` の前処理([05](05-execution-flow.md) §5.1 の 7)で、定義と照合する。純粋な関数(`src/state/`)。

```text
if state.pipelineName !== definition.name:              mismatch("pipelineName")
defined  = 定義のジョブIDの記述順の列
recorded = state.jobOrder
if defined と recorded が、長さも各要素も等しい:         ok
if sorted(defined) と sorted(recorded) が等しい:         mismatch("jobOrder")    # 並びだけが違う
else:                                                    mismatch("jobIds")      # 追加・削除・改名
```

```ts
export type ConsistencyResult =
  | { readonly kind: "ok" }
  | { readonly kind: "mismatch"; readonly what: "pipelineName" | "jobIds" | "jobOrder" };
export function checkConsistency(state: PipelineState, def: { readonly name: string; readonly jobs: readonly { readonly id: string }[] }): ConsistencyResult;
```

比較は文字列の完全一致(`===`。大文字小文字・正規化を区別する。単独のサロゲートは読み込みで U+FFFD に置き換えられているので、UTF-16 とバイト列の比較は同じ結果になる)。`sorted` は、複製した配列に対する `[...ids].sort()`(既定の比較。UTF-16 のコード単位の順)で、重複は取り除かず、並べ替えた列同士の一致を見る。ここでの並べ替えは「集合として同じか」を見るためだけなので、一貫した全順序なら何でもよい(UTF-8 のバイト列の順と、コード単位の順は、基本多言語面の外の文字と U+E000〜U+FFFF の文字の間で違うが、結果には影響しない。確認済み: `["\u{1F600}", "￿"].sort()` は `["😀", "￿"]`)。入力の配列を並べ替えない(複製してから並べ替える)。

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
- 結果: ファイルが無ければ**成功**(0、何も表示しない)。削除が成功 → 0。失敗(ディレクトリ・権限等。「存在しない」以外)→ `removeFailed`、65。TypeScript では **`fs.unlink(path)`** を使い、`err.code === "ENOENT"` なら成功として扱う。
- 完了のステートも消す。標準出力・標準エラーには何も出さない。
- パスがシンボリックリンクなら、リンク自体を削除する(`fs.unlink` はリンクを辿らず、リンク自体を消す。リンク先は消さない。確認済み)。
- **ディレクトリを指すパスは消さない**。`fs.unlink` はディレクトリに対して失敗する(確認済み: macOS では `EPERM`。Linux では `EISDIR` と理解している。未確認)ので、`removeFailed`(65)になる。`fs.rm` や `fs.rmdir` は、空のディレクトリを消しうるため使わない(`fs.rm` に `recursive` や `force` を付けると、ディレクトリや存在しないファイルの扱いが変わる)。Windows でのディレクトリへの `fs.unlink` の挙動は未確認(§13)。
- run ID の値が規則に合わなければ 68。

### 8.2 `conductor reset --all [--config <パス>]`

定義の全ステートを、run ID の有無によらず消す。環境変数は見ない(値の誤りも 68 にならない)。`--run-id` との併用は 68。

対象の判定(厳密):

1. 一覧にするのは、定義ファイルと同じディレクトリだけ(再帰しない。`fs.readdir(dir)`。順序は保証されないので、名前を `sort()` する。run ID は ASCII だけで、候補の名前は同じ接頭辞を持つので、UTF-16 のコード単位の順とバイト列の順は一致する)。
2. `<定義ファイル名>.state`(run ID なし): **通常のファイルのときだけ**(`fs.lstat` の結果が `isFile()`。シンボリックリンクは辿らず、消さない)、中身を読まずに削除(`fs.unlink`)。
3. `<定義ファイル名>.<run-id>.state`: 名前の `<run-id>` が有効な run ID(§1・[05](05-execution-flow.md) §6.1 の `parseRunId` が成功)であるものだけが候補。名前は、定義ファイル名で始まり、続けて `.`、run ID、`.state`(名前の長さが `定義ファイル名.length + 1 + 1 + ".state".length` 以上で、`name.startsWith(定義ファイル名 + ".")`、`name.endsWith(".state")` で、その間 `name.slice(定義ファイル名.length + 1, name.length - 6)` が run ID)。候補をファイル名の昇順に処理する。
   - 通常のファイルでない(ディレクトリ、シンボリックリンク)→ 黙って除外。取得中に消えた(`lstat` が `ENOENT`)→ 除外。`lstat` がその他の理由で失敗 → `kept` に入れる。
   - 中身を見分ける(`inspectRunState`。§6.1 の順 1〜13 を、ファイル名側の run ID を期待値として行う。`readStateRaw` と同じ手順):

| 中身 | 結果 |
|---|---|
| 開けない、1 MiB 超、読み取り失敗、UTF-8・JSON の構文・形式の誤り(`detail = "runIdMismatch"` 以外の `corrupt`。キーの重複・BOM・深すぎる入れ子を含む)、`versionMismatch` | **残して**、名前を `kept` に入れる |
| 長さ 0 | 残す。`kept` に**入れない**(誰のものか分からない) |
| 読めて、中身の `run_id` が名前の run ID と一致(完了のままのものを含む) | **削除** |
| 読めるが、中身の `run_id` が無い、または値が違う(別の定義のステートでありうる。`StateError` の `detail = "runIdMismatch"`) | 残す。`kept` に入れない(黙って残す) |

4. 一時ファイル(`.<ステート名>.tmp.*`)、ほかの名前のファイル、ほかのディレクトリは対象外。
5. 削除の失敗(「存在しない」以外)→ `removeFailed`、65(処理は途中で打ち切る)。ディレクトリ一覧の取得に失敗(ディレクトリが無い場合(`ENOENT`)を除く)→ `unreadable`、65。ディレクトリが無ければ成功。

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
- 読み込み(`readState`)は、完了のステートを「空」として返す(§6.1 の 14)。したがって次の `pipeline` は先頭から始まり、`run` は最初のジョブ以外が順序外(66)になる。
- `watch` は `readStateRaw` で完了のステートを読み、表の下に完了を示す行(英語 `completed`、日本語 `完了`)を足す。`no state (nothing has run yet)` を出すのは、ファイルが無いか長さ 0 のときだけ。壊れた完了のステートは 65。
- `reset`・`reset --all` は完了のステートも消す。

### 9.2 削除の指定

| 項目 | 値 |
|---|---|
| オプション | `--remove-on-complete`(`run`・`pipeline` だけ。値を取らない。ほかのコマンドに付けると 68) |
| 定義 | `pipeline.remove_on_complete`(真偽値。省略時 false。真偽値でなければ定義の誤り 64) |
| 実効値 | `--remove-on-complete` があれば true、なければ定義の値。環境変数では切り替えない。オプションで false にする手段はない |

完了時の手順: 完了のステートを `Durable` で書く → 実効値が真なら `resetState`(run ID に応じたファイルを削除。存在しなくても成功)→ 成功なら 0(`run` ではジョブの終了コード)。削除に失敗したら 65(完了のステートは `completed = true` のまま残る。`watch` は完了として表示する)。

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
- 書き込みは `async` で、1 つの `run`・`pipeline` の中では常に `await` して直列に行う。Node.js のイベントループは 1 つなので、1 つのプロセスの中でのデータ競合は起きないが、書き込みの完了を待たずに次の処理へ進むと、記録の順序と「ジョブの終了後に書く」(at-least-once の前提)が崩れる。

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
| 8 | BOM | 拒否する(`TextDecoder` の `ignoreBOM: true` が必要。§6.1.2) |
| 9 | 巨大な整数 | `format_version` と `exit_code` は、字句で判定し、`number` へ丸めない |
| 10 | ディレクトリを指すステートのパス | `reset` は消さず `removeFailed`(`fs.unlink` を使う。§8.1) |
| 11 | `__proto__` | 普通のキーとして扱う(未知のキーまたは重複)。通常のオブジェクトへ代入しない |

---

## 13. 実装に任せてよい点と未確認

実装に任せてよい点:

- 一時ファイルの名前の細部(§1 の条件を満たすこと)、再試行の回数の上限の具体値は 16 を推奨。
- 内部の構造(書き出しの型の名前、パーサーの内部の作りなど)。ただし、キーの順序・インデント・末尾の改行(§4)、パーサーの規則(§6.5)は固定。
- ファイルの権限(§5。`0o666` から umask を引いた OS 標準)。

確認済み(Node.js v26.13.0、macOS): `JSON.parse` が重複キーの後のものを採ること、巨大な整数・`1.0` の丸め、`__proto__` の扱い、深い入れ子を拒否しないこと、BOM を拒否すること、単独のサロゲートのエスケープをそのまま受けること。`JSON.stringify` がキーを挿入順に出すこと(整数様のキーを先頭に並べること)、`<` `>` `&` と U+2028・U+2029 をエスケープしないこと、単独のサロゲートを `\udXXX` にエスケープすること、インデント 2 の配列の形、`-0` を `0` と出すこと。`TextDecoder` の `fatal` と `ignoreBOM` の挙動(不正な UTF-8 の拒否、BOM の既定の除去)。`fs.open` の `"wx"` と `EEXIST`、`ENOENT`、権限 `0o666`(umask 022 で `100644`)、`FileHandle` の `sync`・`close`、ディレクトリの `fs.open` + `sync`、ディレクトリを開いたときの `EISDIR`(読み取り)、`fs.rename` が置き換え先のディレクトリに `EISDIR` で失敗すること、`fs.unlink` がディレクトリに `EPERM`、シンボリックリンクのリンク自体を消してリンク先を残すこと、`readdir` の `withFileTypes` で `isFile()`・`isSymbolicLink()`・`isDirectory()` が区別できること、`FileHandle.read` の位置に `null` を渡すと無限に読める特殊ファイルから読めて FIFO でも動くこと(`0` では `ESPIPE`)。

未確認:

- Node.js の最新の LTS 系列と 1 つ前の LTS 系列(実装時に確認)での、上記の挙動の再確認。この確認は v26.13.0 だけで行った。上記は、サポートする版の CI の単体テストで確認すること。
- Windows での `fs.rename` による置き換え(既存ファイルへの置き換えの成否、他のプロセスが開いているときの `EPERM`/`EBUSY`/`EACCES`)。Windows でのパスの扱い(`path.win32` の挙動)、ディレクトリへの `fs.unlink` の挙動、ディレクトリを開いたときの `unreadable` の判定、ディレクトリの `sync` の可否、ファイルの権限(ACL)。
- Linux で、ディレクトリへの `fs.unlink` が `EISDIR` になること、ディレクトリを開いて読んだときの `unreadable` の判定。
- `FileHandle.writeFile` が、部分的な書き込みを続けて書き切ること(この仕様はそう理解している)。
- `fs.readdir` の `withFileTypes` が、種別を返さないファイルシステムでの `isFile()` の結果。
- 引数のパスに UTF-8 として不正なバイト列があるときの、Node.js の文字列化(U+FFFD への置換と理解している)。
- `JSON.parse` の reviver がソースの字句を渡す機能の有無と、構文の誤りの位置の有無(いずれも、採用の判断に影響しない)。
- 一時ファイルの `<時刻>` は、`Date.now()` のミリ秒を 10^6 倍した値。クロックが戻った場合の挙動は、名前の衝突の再試行(最大 16 回)に任せている。
- §4.4 の例 1〜5 は、規則からの書き起こしで、実装の出力として確認していない(例 4 は、書き込みの契機から導いたもの)。例 6〜8 は、§4.2 の順序のシリアライザを動かして得た出力。
