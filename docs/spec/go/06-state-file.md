# 06. ステートファイル(論理項目・JSON の形式・読み書き・整合性・reset)

対象: Go で実装する `conductor` v0.8.0 と同じ挙動の実行ツール。ステートファイルは、あるパイプライン定義の実行の途中経過(どのジョブがどう終わったか、どこから再開するか)を保存するファイル。実行の意味論(いつ何を記録するか)は [05-execution-flow.md](05-execution-flow.md) にある。この文書は、ステートが持つ**論理的な項目**、保存の**必須の性質**、**JSON の形式**、読み書きの手順、異常系の挙動を定める。

ファイル名は `<定義ファイル名>.state` / `<定義ファイル名>.<run ID>.state`(定義ファイルの拡張子を含む)で、中身は **JSON**(UTF-8、BOM なし)。実装は `internal/state`(読み書き・整合性検査・reset)。ステート値の型(`JobState` など)は `internal/flow` と共有する([05](05-execution-flow.md) §0)。

## 0. この文書の読み方

| 区分 | 内容 |
|---|---|
| 必須(どの実装も同じ) | §2 の論理項目・値の規則・矛盾の条件、§3 の保存の性質、§4 の JSON の形式(キー名・型・固定の順序)、§6 の読み込みの検査(順序・種類・理由文・案内文)、§7 の整合性検査、§8 の reset の挙動、ファイルの場所と名前(§1) |
| 実装に任せてよい点 | 一時ファイルの名前の細部(§1 の条件を満たす)、内部の構造、インデントの幅(読み込みは空白に依存しない。ただし書き出しは §4.3 の規則に固定する) |

---

## 1. 場所と名前

ステートファイルは、**定義ファイルと同じディレクトリ**に置く。

| run ID | ファイル名 |
|---|---|
| なし | `<定義ファイル名>.state` |
| あり | `<定義ファイル名>.<run-id>.state` |

- `<定義ファイル名>` は、定義ファイルのパスの最後の要素(拡張子を含む。Go では `filepath.Base`)。例: `conductor.yaml` → `conductor.yaml.state`、`--config ci/nightly.json --run-id 12345` → `ci/nightly.json.12345.state`。
- 定義ファイルのパスは、`--config` で与えられた文字列のまま(正規化・絶対化しない)。ディレクトリ部分が空(`conductor.yaml` のように親が無い。`filepath.Dir` が `"."` を返す)なら、実行ディレクトリ。ステートのパスは `filepath.Join(filepath.Dir(defPath), name)`。
- run ID は検査済み(`A-Za-z0-9._-`、1〜64 文字、先頭が `.` でない。[05](05-execution-flow.md) §6)なので、パス区切りや `..` は入らない。
- 一時ファイルの名前: `.<ステートファイル名>.tmp.<プロセス番号>.<時刻(ナノ秒)>.<連番>`(同じディレクトリ。§5)。名前の付け方は実装に任せてよいが、**ドットで始まり、`.tmp.` を含み、後始末(§5 の 9)で識別できる**こと。
- 大文字小文字を区別しないファイルシステム(macOS の既定、Windows)では、`A` と `a` の run ID は同じファイルになる。食い違いは、中身の `run_id` との照合で壊れたステート(65)として検出される(§6 の 9)。

---

## 2. ステートの論理項目

### 2.1 ステート全体の項目

| 項目 | 型 | 必須 | 意味・値 |
|---|---|---|---|
| `format_version` | 整数 | 必須 | 常に `1`。読み込みでは最初に検査される(§6) |
| `definition_key` | 文字列 | 必須 | 定義ファイルのパス(`definition_path` と**同じ文字列**を書く。読むときは別々に保持するが、どちらも照合に使わない) |
| `definition_path` | 文字列 | 必須 | 定義ファイルのパス(`--config` の文字列。文字列として表せない部分(不正な UTF-8)は代替文字 U+FFFD に置換される)。空でもよい |
| `pipeline_name` | 文字列 | 必須 | 記録時の `pipeline.name`。照合に使う(§7) |
| `job_order` | 文字列の列 | 必須 | 記録時のジョブIDを、定義の記述順に並べたもの |
| `job` | ジョブ状態の列 | 必須 | ジョブごとの状態(§2.2)。`job_order` と同じ順・同じ数 |
| `resume_job_id` | 文字列 | 任意 | 次に実行する、または再開するジョブのID。無いときは項目自体を保存しない。`job_order` に含まれる値 |
| `end_reached` | 真偽 | 必須 | `@end` に到達したか。一度 true になったら、その run のあいだ false に戻らない |
| `completed` | 真偽 | 必須 | 完了したか(§9) |
| `run_id` | 文字列 | 任意 | run ID。run ID ありのステートだけが保存する。無いステートには項目を保存しない |

上の表にない項目は、ステート全体にも、ジョブ状態の中にも許さない(§6 の `corrupt`)。

### 2.2 ジョブ状態の項目

| 項目 | 型 | 必須 | 意味・値 |
|---|---|---|---|
| `id` | 文字列 | 必須 | ジョブID |
| `status` | 文字列 | 必須 | `"pending"` / `"success"` / `"warning"` / `"error"` / `"skipped"`(これ以外は `corrupt`) |
| `exit_code` | 整数(32 ビット符号付き整数の範囲: -2147483648〜2147483647) | 任意 | ジョブの終了コード。`status` が success / warning / error のときだけ持てる。終了コードが無い結果(起動失敗・シグナル終了)の error には保存しない。負数も許す(例 `-1`)。範囲外の整数は `corrupt` |
| `skip_reason` | 文字列 | 条件付き | `"byTransition"`(遷移によるスキップ)/ `"byEnd"`(`@end` によるスキップ)。**`status = "skipped"` のときだけ必須、そのほかの `status` では保存してはならない** |

### 2.3 内容の矛盾(読み込み時に `corrupt` になる条件)

- ジョブ状態のジョブID に重複がある。
- ジョブ状態の並び(`id` の列)が `job_order` と、数も順序も完全には一致しない。
- `resume_job_id` が `job_order` にない。
- `resume_job_id` があるのに `completed = true`(完了と再開位置は両立しない)。
- `status` が `skipped` なのに `skip_reason` が無い、または `skipped` 以外なのに `skip_reason` がある。
- `status` が pending / skipped なのに `exit_code` がある。

### 2.4 初期ステート(`InitialState`)

定義から作る、記録前のステート: 全ジョブ `pending`(`exit_code`・`skip_reason` なし)、`resume_job_id` 無し、`end_reached = false`、`completed = false`、`definition_key` = `definition_path` = 定義ファイルのパス文字列、`pipeline_name` = 定義の名前、`job_order` = 定義のジョブIDの記述順、`run_id` = 決定した run ID(あれば)。

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

- UTF-8 の JSON(RFC 8259 の 1 個の値)。最上位は**オブジェクト**。BOM は書かない(読むときは拒否する。§6.1 の 6)。
- 論理項目とのキーの対応は同名: §2.1 の項目は最上位のキー、§2.2 は `job` の配列の要素(オブジェクト)の中のキー。`job` は**オブジェクトの配列**(キー名は `job` のまま。論理項目の名前と一致させるための決定)、`job_order` は文字列の配列。
- 任意の項目(`resume_job_id`、`run_id`、`exit_code`、`skip_reason`)は、無いときキー自体を書かない(`null` を書かない)。
- 空の配列は `[]` と書く(`null` にしない。Go では `nil` スライスが `null` になるので、書き出す前に空スライスにする)。

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

```go
// 書き出し専用。フィールドの宣言順がそのままキーの出力順になる(§4.2 の順序)。
type stateFile struct {
    FormatVersion  int       `json:"format_version"`
    DefinitionKey  string    `json:"definition_key"`
    DefinitionPath string    `json:"definition_path"`
    PipelineName   string    `json:"pipeline_name"`
    RunID          *string   `json:"run_id,omitempty"`
    JobOrder       []string  `json:"job_order"`
    Job            []jobFile `json:"job"`
    ResumeJobID    *string   `json:"resume_job_id,omitempty"`
    EndReached     bool      `json:"end_reached"`
    Completed      bool      `json:"completed"`
}

type jobFile struct {
    ID         string  `json:"id"`
    Status     string  `json:"status"`
    ExitCode   *int32  `json:"exit_code,omitempty"`   // *int32 なので、値 0 も出力される(nil のときだけ省く)
    SkipReason *string `json:"skip_reason,omitempty"`
}
```

- `omitempty` をポインタに付けると、`nil` のときだけ省かれ、指す値が `0` や `false` でも出力される(Go の仕様)。`ExitCode` を `int32`(ポインタでない)にして `omitempty` を付けると、`0` が省かれて誤りになるので、**必ずポインタにする**。
- Go の `encoding/json` は構造体のフィールドを宣言順に出力する。この順序に頼る場合は、キーの順序を検証する単体テスト(§4.4 の例との完全一致)を必ず置く。

### 4.3 書き出しの規則

- `json.NewEncoder(w)` に `SetEscapeHTML(false)`(`<` `>` `&` を `<` などに変えない)、`SetIndent("", "  ")`(インデントは空白 2 つ)を設定し、`Encode` で書く。`Encode` は末尾に `\n` を 1 つ付ける(LF。CRLF にしない)。ファイルの末尾は改行 1 つ。
- 文字列は、Go が出力する表記のまま(不正な UTF-8 は U+FFFD に置換され、制御文字・`"`・`\` はエスケープされる。U+2028/U+2029 は常に ` `/` ` になる)。読み込みは JSON として正しければどの表記も受け付ける。
- 数値は 10 進の整数だけ(`1`、`0`、`-1`、`2147483647`)。小数点・指数は書かない。
- 配列は、要素ごとに 1 行(インデントされる)。空配列は `[]`。

### 4.4 実ファイルの例(全フィールドを含む)

以下は §4.3 の規則で出力される内容(各例は規則から書き起こしたもの。実装の出力と一致することを、実装時に単体テストで確認する。§13)。

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

(3)全種類の状態を含む例(4 ジョブ。run ID なし、`end_reached = true`、エラーで停止、負の `exit_code`)。ファイル名 `conductor.yaml.state`(定義 `dir/conductor.yaml`)。

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

(ファイル名は、実際には `dir/conductor.yaml.state`。ディレクトリ `dir` の中に置かれる。)

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

(6)ジョブIDに特殊な文字を含む例(参考。`"`、`\`、日本語、タブ)。`definition_path` に不正な UTF-8 のバイトがあれば U+FFFD(`�`)になる。ジョブID は定義の検証で空でなく `@` で始まらない文字列であることだけが保証される(前後の空白もそのまま)。

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

---

## 5. 書き込み(`Write`)の手順と原子性

入力: 定義ファイルのパス、run ID、ステート、耐久性(Durable / Fast)。

```text
1. data = serialize(state)             # §4.3。失敗(通常は起きない)なら WriteFailed(理由=「内容を JSON にできません」)
2. path = ステートファイルの場所(§1)
3. 一時ファイルを、同じディレクトリに、「既存なら失敗する」新規作成で作る
     os.OpenFile(tmpPath, os.O_WRONLY|os.O_CREATE|os.O_EXCL, 0o666)
     名前 = ".<ステートファイル名>.tmp.<os.Getpid()>.<time.Now().UnixNano()>.<連番>"
     名前が既存(errors.Is(err, fs.ErrExist))なら、連番を進めて作り直す。最大 16 回。それでも作れなければ WriteFailed
     連番 = プロセス内の通し番号(atomic.Uint64 の Add) + 試行回数(桁あふれは折り返してよい)
4. 全内容を書き(File.Write の戻り値と error を確認)、フラッシュする(バッファを挟まないなら不要)
5. Durable のときだけ、File.Sync() で内容を記憶装置まで同期する
6. File.Close()(Windows は、開いたままだと移せない・消せない。Close の error も失敗として扱う)
7. os.Rename(tmpPath, path) で置き換える
8. 3〜7 のどこかで失敗したら: 自分が作った一時ファイルを os.Remove(失敗は無視)し、WriteFailed(理由=「書き込みに失敗しました」)
9. 成功し、かつ Durable のとき:
     UNIX 系では、ディレクトリを os.Open して Sync(失敗は無視。Windows では何もしない)
     同じ名前の型の一時ファイルのうち、更新から 24 時間以上たった通常のファイルを消す(失敗は無視)
```

Go 版の選定:

- **`os.CreateTemp` を使わない**。`os.CreateTemp` はパーミッションが 0600 で、名前の規則も本書と合わないため。`os.OpenFile` に `0o666` を渡し、UNIX 系では umask が引かれる(通常 `0644` = `-rw-r--r--`)ようにする。
- パーミッションは明示的に設定しない(`Chmod` しない)。既存のステートファイルを置き換えるときも、権限は引き継がず、新しい一時ファイルの権限になる。所有者の変更・ACL の操作はしない。
- 時刻は `time.Now().UnixNano()`(`int64`)。取得に失敗する概念は無い(0 を使う場面は無い)。クロックが戻っても、名前の衝突は連番の再試行(最大 16 回)で避ける。

性質:

- **原子的**: 読み手は、直前の完全な内容か、新しい完全な内容のどちらかだけを見る。途中の内容は見えない。どの段階で失敗しても、ステートファイルは直前の完全な内容のまま。
- 置き換え先がディレクトリなど、名前の付け替えが失敗するときは `WriteFailed`。ステートは変わらず、一時ファイルも残らない。書けないディレクトリも `WriteFailed`。
- 残った一時ファイル(プロセスが落ちたとき)は、読み込みに影響しない。`Durable` の書き込みの成功後に、24 時間以上古いものだけ消える。`Fast` では消さない。
- 「同じ名前の型」= 名前が `.<ステートファイル名>.tmp.` で始まるもの。後始末は、`os.ReadDir` の各エントリのうち `entry.Type().IsRegular()`(シンボリックリンク・ディレクトリを除く。リンクを辿らない)で、`entry.Info()` の `ModTime()` が `now - 24h` 以前のものを `os.Remove` する(エラーは無視)。
- 排他制御(ファイルロック)はしない(§11)。

### 5.1 Windows での置き換えの注意

- `os.Rename` は、Go の実装では置き換え先が既にあっても上書きする(Windows では `MoveFileEx` を置き換えの指定つきで呼ぶと理解している。**未確認**: 採用する Go の版で、既存のファイルへの `os.Rename` が成功することを、Windows の CI のテストで確認する)。
- 一時ファイルは、`Rename` の前に `Close` する(開いたままだと移せない)。
- 置き換え先のステートファイルを、ほかのプロセス(例: 同時に走る `watch`)が開いている間は、Windows では `Rename` が失敗しうる(Go の `os.Open` は、ほかのプロセスによる削除・名前変更を許す共有の指定を付けない、と理解している。**未確認**)。失敗したら `WriteFailed`(65)にする。再試行はしない(仕様として、読み手との同時の実行は想定しない。§11)。
- ディレクトリの同期は行わない(Windows ではディレクトリを `Sync` できない)。
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

## 6. 読み込み(`Read` / `ReadRaw`)の手順と異常系

### 6.1 手順

入力: 定義ファイルのパス、run ID(ファイル名側の期待値)。上から順に、最初に当てはまる結果で終わる。`Read` は完了のステートを「空」にし、`ReadRaw`(`watch` が使う)は 14 を行わない。

| 順 | 処理 | 結果 |
|---|---|---|
| 1 | `os.Open` | 存在しない(`errors.Is(err, fs.ErrNotExist)`)→ **空**(ステート無し)。その他の開く失敗(権限など)→ `unreadable` |
| 2 | 開いたハンドルの `Stat()` の `Size()` を確認 | `Stat` の失敗 → `unreadable`。1 MiB(1,048,576 バイト)**超** → `tooLarge` |
| 3 | `io.ReadAll(io.LimitReader(f, 1048576+1))`(上限 + 1 バイトまで。終わりのない入力でも止まる) | 読み取り失敗 → `unreadable`。読めたバイト数が 1,048,576 を超える → `tooLarge`(ちょうど上限は受理) |
| 4 | 長さ 0 | **空** |
| 5 | `utf8.Valid(data)` | 偽 → `corrupt`(理由 `UTF-8 の文字列ではありません`) |
| 6 | JSON として解析(下の 6a〜6c を順に) | 失敗 → `corrupt`(6a・6b は `JSON として読めません`、6c は `キーが重複しています`) |
| 7 | `format_version` の確認(**ほかの検査より先**。将来の版が、未知の項目で corrupt にならないように) | 項目無し → `corrupt`(`format_version がありません`)。整数以外 → `corrupt`(`format_version が整数ではありません`)。整数で 1 以外(0 や 2 や巨大な整数を含む)→ `versionMismatch`(`対応していない版です(reset が必要です)`) |
| 8 | 最上位に未知の項目がある | `corrupt`(`知らないキーがあります`) |
| 9 | `run_id` の取り出しと照合 | 型が文字列でない(`null` を含む)→ `corrupt`(`値の型が誤っています`)。中身の `run_id`(無ければ「なし」)がファイル名の run ID(無ければ「なし」)と一致しない → `corrupt`(`run_id がファイル名の run ID と一致しません`) |
| 10 | 必須項目の取り出し: `definition_key`、`definition_path`、`pipeline_name`、`end_reached`、`completed`、`job_order`、`job`(**この順に 1 項目ずつ**、無い/型が違うを判定する) | 無い → `corrupt`(`必須のキーがありません`)。型が違う → `corrupt`(`値の型が誤っています`)。`job_order` は文字列の配列(要素に文字列以外があれば型の誤り)、`job` はオブジェクトの配列 |
| 11 | `job` の各要素を、**配列の順に 1 つずつ**、次の順で検査する(§6.3 の表) | 下の表のとおり、すべて `corrupt` |
| 12 | `resume_job_id` の型 | 文字列でない(`null` を含む)→ `corrupt`(`値の型が誤っています`) |
| 13 | 内容の矛盾(この順): ジョブIDの重複 → `job` の並びと `job_order` の不一致 → `resume_job_id` が `job_order` に無い → `completed` かつ `resume_job_id` あり | いずれも `corrupt`(それぞれ `ジョブIDが重複しています`、`job の並びが job_order と一致しません`、`resume_job_id が job_order にありません`、`completed と resume_job_id が両立しません`) |
| 14 | `Read` のみ: `completed = true` | **空**(完了のまま残ったステート。ファイルは消さない) |

順 6 の細目(この順。最初の失敗で終わる):

| 細目 | 処理 | 失敗の理由 |
|---|---|---|
| 6a | `json.Valid(data)` が真か。偽になるもの: 構文の誤り、BOM(先頭の `EF BB BF`)、値の後ろの余分な文字(2 つ目の値を含む)、空白だけの内容、NUL を含む制御文字の誤用、入れ子が深すぎる(`encoding/json` の入れ子の上限を超える)もの | `JSON として読めません` |
| 6b | 最初の空白でないバイトが `{` か(`null`、`[`、数値、文字列、真偽の最上位を拒否する。`null` は、`map` への `Unmarshal` がエラーにならず空になるので、**先に自前で見る**) | `JSON として読めません` |
| 6c | どのオブジェクトにも、同じキーが 2 回以上現れないか(§6.5 の方法で、最上位と `job` の各要素を含む全オブジェクトを走査する。エスケープを展開した後の文字列で比べる。`"id"` と `"id"` は同じキー) | `キーが重複しています` |

空白の定義は JSON の空白(スペース、`\t`、`\n`、`\r`)。BOM は空白でないので、6a で拒否される(書き出しは BOM を付けない。他のツールが BOM 付きで保存した場合は壊れたステートとして 65 になり、`reset` で回復する)。

順 7 の「整数」の定義: 値の字句が `-?(0|[1-9][0-9]*)` に一致するもの(小数点・指数を含まない)。`1.0`、`1e0`、`"1"`、`true`、`null`、配列、オブジェクトは**整数以外**(`format_version が整数ではありません`)。`-0` と `0`、`2`、`-1`、`99999999999999999999` は整数で 1 ではないので `versionMismatch`。`1` だけが受理される。

順 11 の `corrupt` の理由文(個別。いずれも種類は `corrupt`。1 つの要素の中の検査の順に上から):

| 順 | 違反 | 理由 |
|---|---|---|
| a | 要素がオブジェクトでない(`null` を含む) | `値の型が誤っています` |
| b | 未知のキー(大文字小文字も区別。`ID`、`Status` は未知) | `知らないキーがあります` |
| c | `id` が無い | `必須のキーがありません` |
| d | `id` が文字列でない | `値の型が誤っています` |
| e | `status` が無い | `必須のキーがありません` |
| f | `status` が文字列でない | `値の型が誤っています` |
| g | `status` が 5 つの値以外 | `status の値が誤っています` |
| h | `exit_code` があり、整数でない(`1.0`、`1e2`、文字列、`null` を含む)、または 32 ビット符号付き整数の範囲外(`strconv.ParseInt(字句, 10, 32)` が失敗) | `値の型が誤っています` |
| i | `skip_reason` があり、文字列でない(`null` を含む) | `値の型が誤っています` |
| j | `skip_reason` が 2 つの値以外 | `skip_reason の値が誤っています` |
| k | `status = skipped` と `skip_reason` の有無の不整合 | `skip_reason が status と合いません` |
| l | `status` が pending / skipped なのに `exit_code` あり | `exit_code が status と合いません` |

- `ReadRaw`(`watch` が使う)は、14 を行わない(完了のステートもそのまま返す)。1〜13 は同じ。
- 検査の `run_id` 照合(9)は、`completed` の判定(14)より前なので、完了のステートでも run ID の食い違いは `corrupt`。
- 読み取りでは、ファイルを**消さない・書き換えない**。
- 「空」= ステートが無いのと同じ扱い。呼び出し側は、すべて pending のステートとして扱い、最初の `Run` の前に初期ステートを書く。
- 空のステートを返す関数の戻り値は `(nil, nil)`(`*PipelineState` が `nil`)。
- ディレクトリに対する `os.Open` は UNIX 系では成功し、順 3 の読み取りで失敗する想定(`unreadable`)。実機の確認はしていない(未確認。§13)。

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

Go での表現(骨格):

```go
package state

type ErrorKind int

const (
    KindUnreadable ErrorKind = iota
    KindTooLarge
    KindCorrupt
    KindVersionMismatch
    KindWriteFailed
    KindRemoveFailed
)

// Error は、パスと理由(固定の日本語文)を持つ。入力の値や下位のエラー文を含めない。
// 下位のエラー(*fs.PathError など)は Unwrap でだけ辿れる。
type Error struct {
    Kind   ErrorKind
    Path   string
    Reason string
    Err    error // 表示しない
}

func (e *Error) Error() string // 表示用の組み立ては internal/report が行う
func (e *Error) Unwrap() error
```

### 6.3 異常系の挙動の一覧(`run` / `pipeline`)

| 状況 | 結果 | ジョブの起動 | ステート |
|---|---|---|---|
| ファイルが無い | 空として続行 | する | 最初の `Run` の前に初期ステートを書く |
| 長さ 0 | 空として続行 | する | 同上(上書き) |
| `completed = true` | 空として続行(定義との照合もしない) | する | 初期ステートで上書き |
| 読めない(権限等) | 65 | しない | 変更なし |
| 1 MiB 超 | 65 | しない | 変更なし |
| UTF-8 でない・JSON の構文の誤り・BOM・最上位がオブジェクトでない・キーの重複・未知のキー・必須項目欠落・型の誤り・内容の矛盾 | 65(`corrupt`) | しない | 変更なし |
| `format_version` が 1 以外の整数 | 65(`versionMismatch`) | しない | 変更なし |
| 中身の `run_id` とファイル名の run ID が不一致、または片方だけ有る | 65(`corrupt`) | しない | 変更なし |
| 定義と食い違い(§7) | 65 | しない | 変更なし |
| 書き込み・削除の失敗 | 65 | §5.2 の表のとおり | 直前の内容のまま(書き込み)/ 残る(削除) |

どの異常でも、回復は `conductor reset`(または `--run-id`、`--all`)。`reset` は読まず検査もしないため、どんなファイルでも消せる(§8)。

### 6.4 JSON 向けの理由文の全件表(書き換えた箇所の一覧つき)

読み込みで出しうる `corrupt` / `versionMismatch` / `unreadable` / `tooLarge` の理由文と、書き込みの理由文の**全件**。「論理モデル上の文」は、書式を問わない検査の定義にある理由文。変更のある行だけ「書き換え」の欄に印を付ける。

| # | kind | 理由文(Go 版) | 出る順 | 条件 | 書き換え |
|---|---|---|---|---|---|
| 1 | unreadable | `開く、または読み取ることができません` | 1〜3 | 開く・`Stat`・読み取りの失敗(存在しないを除く) | なし |
| 2 | tooLarge | `大きさが上限(1 MiB)を超えています` | 2、3 | 1,048,576 バイト超 | なし |
| 3 | corrupt | `UTF-8 の文字列ではありません` | 5 | UTF-8 として不正 | なし |
| 4 | corrupt | `JSON として読めません` | 6a、6b | JSON の構文の誤り、BOM、余分な文字、最上位がオブジェクトでない | **書き換え**(論理モデル上の文: `TOML として読めません`)。BOM と最上位の型の条件は JSON 向けに追加 |
| 5 | corrupt | `キーが重複しています` | 6c | 同じキーが 2 回以上(最上位・`job` の要素を含む全オブジェクト) | **追加**(論理モデル上は構文の誤りに含まれる。Go の標準の読み込みは重複を検出しないため、独立した理由にした) |
| 6 | corrupt | `format_version がありません` | 7 | キー無し | なし |
| 7 | corrupt | `format_version が整数ではありません` | 7 | 整数の字句でない(`1.0`、`"1"`、`null` など) | なし(整数の定義を §6.1 に明記) |
| 8 | versionMismatch | `対応していない版です(reset が必要です)` | 7 | 整数で 1 以外 | なし |
| 9 | corrupt | `知らないキーがあります` | 8、11b | 最上位、または `job` の要素に未知のキー(大文字小文字を区別) | なし |
| 10 | corrupt | `値の型が誤っています` | 9、10、11、12 | 型の誤り(`null` を含む)、`exit_code` の範囲外・小数 | なし(`null` と小数の扱いを明記) |
| 11 | corrupt | `run_id がファイル名の run ID と一致しません` | 9 | 中身とファイル名の run ID の不一致 | なし |
| 12 | corrupt | `必須のキーがありません` | 10、11 | 必須キーが無い | なし |
| 13 | corrupt | `status の値が誤っています` | 11 | 5 つの値以外 | なし |
| 14 | corrupt | `skip_reason の値が誤っています` | 11 | 2 つの値以外 | なし |
| 15 | corrupt | `skip_reason が status と合いません` | 11 | `skipped` と `skip_reason` の有無の不整合 | なし |
| 16 | corrupt | `exit_code が status と合いません` | 11 | pending / skipped に `exit_code` | なし |
| 17 | corrupt | `ジョブIDが重複しています` | 13 | `job` の `id` の重複 | なし |
| 18 | corrupt | `job の並びが job_order と一致しません` | 13 | 数または順序の不一致 | なし |
| 19 | corrupt | `resume_job_id が job_order にありません` | 13 | | なし |
| 20 | corrupt | `completed と resume_job_id が両立しません` | 13 | | なし |
| 21 | writeFailed | `書き込みに失敗しました` | 書き込み | 一時ファイル・書き込み・同期・閉じる・置き換えの失敗 | なし |
| 22 | writeFailed | `内容を JSON にできません` | 書き込み | `json` への変換の失敗 | **書き換え**(論理モデル上の文: `内容を TOML にできません`) |
| 23 | removeFailed | `削除に失敗しました` | reset | 「存在しない」以外の削除の失敗 | なし |

**書き換えた箇所の一覧**(まとめ):

1. 構文の誤りの理由: `TOML として読めません` → `JSON として読めません`(#4)。
2. 書き込みの変換の失敗の理由: `内容を TOML にできません` → `内容を JSON にできません`(#22)。
3. 追加: `キーが重複しています`(#5)。
4. 条件の追加(理由文は #4 のまま): BOM、最上位がオブジェクトでない値(`null`・配列・数値・文字列・真偽)、値の後ろの余分な文字、空白だけの内容。
5. 条件の明記(理由文は変えない): `null` はどの項目でも型の誤り(任意の項目でも「無い」とは扱わない)、整数の字句の定義(小数点・指数は整数でない)、キー名は大文字小文字を区別する。
6. 理由文の言語: これまでどおり、ロケールに関わらず日本語の固定文(`conductor: error: ...` の行の中で、種類の文と案内だけがロケールに従う)。

### 6.5 `encoding/json` の厳格な読み込み(選定)

目的: 順序が固定の 14 段階の検査と、理由文の区別を、`encoding/json` の標準の機能だけで満たす。

| 候補 | 内容 | 判断 |
|---|---|---|
| A. 構造体へ `Unmarshal` + `Decoder.DisallowUnknownFields()` | 最も短い | **不採用**。(1) 未知のキーの検査が、版の確認(順 7)より先に走ってしまい、将来の版が `versionMismatch` にならず `corrupt` になる。(2) `encoding/json` は、構造体のキーの照合で大文字小文字を区別しない(`Format_Version` も `format_version` として受け付けてしまう)。(3) 型の誤り(`UnmarshalTypeError`)が、どのキーで起きたかを理由文に結び付けにくく、14 段階の順序を再現できない。(4) 重複キーは検出されない(後のものが勝つ) |
| B. `map[string]json.RawMessage` へ `Unmarshal` し、キーを手順どおりに取り出す | キーは文字列のまま(大文字小文字を区別して照合できる)。値は未解釈の字句で保持され、順 7 以降を自分で順序どおり検査できる | **採用** |
| C. 外部の JSON ライブラリ | 重複キーの検出などが付くものがある | 不採用(標準ライブラリで足りる。外部ライブラリは YAML だけ) |

採用案 B の手順:

1. `utf8.Valid`(順 5)→ `json.Valid`(6a)→ 最初の空白でないバイトが `{`(6b)。
2. **重複キーの検出(6c)**: `json.NewDecoder(bytes.NewReader(data))` で `Token()` を末尾まで読む。入れ子の各オブジェクトごとに「見たキーの集合」を持つスタックを管理する。`json.Delim('{')` で新しい集合を積み、オブジェクトの中では、トークンが「キー」と「値」で交互に来ることを使って、キー位置の `string` を集合に入れる(すでにあれば重複)。`json.Delim('}')` で降ろす。`Token()` が返すキーは、エスケープ展開後の文字列である。`Token()` の失敗は、6a を通っているので起きないはずだが、起きたら `JSON として読めません` にする。
3. `var top map[string]json.RawMessage` へ `json.Unmarshal(data, &top)`(失敗は `JSON として読めません`。6a・6b を通ったので起きない想定)。
4. 手順 7〜12 は `top` のキーを、仕様の順に、完全一致(大文字小文字を区別)で取り出す。未知のキーの検査(順 8)は、`top` の全キーが §2.1 の 10 個のどれかか、を見る。
5. 値の型の判定は、`RawMessage` の最初のバイトで行う: `"` → 文字列(`json.Unmarshal(raw, &s)` で取り出す)、`t`/`f` → 真偽(字句が `true`/`false`)、`n` → null(型の誤り)、`[` → 配列、`{` → オブジェクト、それ以外 → 数値(字句を保持。整数かの判定は §6.1 の定義、`exit_code` は `strconv.ParseInt(字句, 10, 32)`)。
6. `job` の要素と `job_order` も、`[]json.RawMessage` / `map[string]json.RawMessage` に分けて、同様に取り出す。
7. 文字列の値に、不正なサロゲートのエスケープ(`"\ud800"` のような単独のサロゲート)があっても `json.Valid` は通り、`json.Unmarshal` は U+FFFD に置き換える。ステートファイルを書くのは conductor なので起きないが、起きた場合は置換された値で照合を続ける(矛盾があれば、それぞれの検査で `corrupt`)。

`json.Valid` が BOM・余分な文字・深すぎる入れ子を偽にすること、`null` を `map` へ `Unmarshal` してもエラーにならないこと、`Token()` が展開後のキーを返すことは、Go の `encoding/json` の仕様・実装についての理解に基づく。採用する Go の版で、§6.6 の単体テストにより確認する(確認できない場合は、テストが失敗するので、実装を直す)。

### 6.6 読み込みの単体テストで固定する入力(最低限)

| 入力(要点) | 期待 |
|---|---|
| 正常(§4.4 の例 1〜6) | 受理。往復(読んで書く)で元と一致 |
| 先頭に BOM + 正常な内容 | `corrupt`(`JSON として読めません`) |
| 正常な内容の後ろに `{}` や `x` | `corrupt`(`JSON として読めません`) |
| `null`、`[]`、`1`、`"x"`、`true` | `corrupt`(`JSON として読めません`) |
| 空白だけ(`"\n"`) | `corrupt`(`JSON として読めません`) |
| 最上位に同じキーが 2 回(`"completed": false, "completed": true`) | `corrupt`(`キーが重複しています`) |
| `job` の要素に同じキーが 2 回 | `corrupt`(`キーが重複しています`) |
| `"id"` と `"id"` が同じ要素に | `corrupt`(`キーが重複しています`) |
| `{"format_version": 2, "unknown": 1}` | `versionMismatch`(未知のキーより版が先) |
| `{"format_version": 1.0}` | `corrupt`(`format_version が整数ではありません`) |
| `{"format_version": "1"}` | `corrupt`(`format_version が整数ではありません`) |
| `{"Format_Version": 1}` | `corrupt`(`format_version がありません`) |
| `exit_code` が `1.5`、`2147483648`、`"3"`、`null` | `corrupt`(`値の型が誤っています`) |
| `exit_code` が `-2147483648`、`2147483647` | 受理 |
| `resume_job_id` が `null` | `corrupt`(`値の型が誤っています`) |

---

## 7. 定義との整合性検査(`CheckConsistency`)

空でない(完了していない)ステートを読んだ後、`run`・`pipeline` の前処理([05](05-execution-flow.md) §5.1 の 7)で、定義と照合する。純粋な関数。

```text
if state.PipelineName != definition.Name:               Mismatch(PipelineName)
defined  = 定義のジョブIDの記述順の列
recorded = state.JobOrder
if defined == recorded:                                  Ok          # slices.Equal
if sorted(defined) == sorted(recorded):                  Mismatch(JobOrder)    # 並びだけが違う
else:                                                    Mismatch(JobIds)      # 追加・削除・改名
```

比較は文字列の完全一致(バイト列として。大文字小文字・正規化を区別する。Go の `==` と `slices.Equal`)。`sorted` は文字列の全順序(Go の `sort.Strings`・`slices.Sort` はバイト列の辞書順)で、重複は取り除かず、並べ替えた列同士の一致(`slices.Equal`)を見る。入力のスライスを並べ替えない(複製してから並べ替える)。

| 状況 | 結果 |
|---|---|
| パイプライン名が違う | `PipelineName` |
| ジョブID の集合が違う(追加・削除・改名。重複の有無も含め、並べ替えた列が違う) | `JobIds` |
| 集合は同じで記述順だけが違う | `JobOrder` |
| すべて同じ | `Ok` |

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
- 結果: ファイルが無ければ**成功**(0、何も表示しない)。削除が成功 → 0。失敗(ディレクトリ・権限等。「存在しない」以外)→ `removeFailed`、65。Go では `os.Remove(path)` の `error` が `errors.Is(err, fs.ErrNotExist)` なら成功として扱う。
- 完了のステートも消す。標準出力・標準エラーには何も出さない。
- パスがシンボリックリンクなら、リンク自体を削除する(`os.Remove` はリンクを辿らず、リンク自体を消す。リンク先は消さない)。
- ディレクトリを指すパス(空のディレクトリ)は、`os.Remove` が空のディレクトリを消せる挙動である点に注意: ステートのパスにディレクトリがある異常な場合に、空のディレクトリが消される可能性がある(確認していない。**未確認**。`os.Remove` を使うか、通常のファイルでないときに別の扱いにするかは、実装時に決めてテストで固定する。仕様としては「存在しない以外の失敗は `removeFailed`」とだけ定める)。
- run ID の値が規則に合わなければ 68。

### 8.2 `conductor reset --all [--config <パス>]`

定義の全ステートを、run ID の有無によらず消す。環境変数は見ない(値の誤りも 68 にならない)。`--run-id` との併用は 68。

対象の判定(厳密):

1. 一覧にするのは、定義ファイルと同じディレクトリだけ(再帰しない。`os.ReadDir`。結果はファイル名の昇順で返る)。
2. `<定義ファイル名>.state`(run ID なし): **通常のファイルのときだけ**(`os.Lstat` の `Mode().IsRegular()`。シンボリックリンクは辿らず、消さない)、中身を読まずに削除。
3. `<定義ファイル名>.<run-id>.state`: 名前の `<run-id>` が有効な run ID(§1・[05](05-execution-flow.md) §6.1 の `runid.Parse` が成功)であるものだけが候補。名前は、定義ファイル名で始まり、続けて `.`、run ID、`.state`(名前の長さが `len(定義ファイル名) + 1 + 1 + len(".state")` 以上で、接頭辞が `定義ファイル名 + "."`、接尾辞が `".state"` で、その間が run ID)。候補をファイル名の昇順に処理する。
   - 通常のファイルでない(ディレクトリ、シンボリックリンク)→ 黙って除外。取得中に消えた → 除外。メタデータの取得が失敗 → `kept` に入れる。
   - 中身を見分ける(`inspectRunState`):

| 中身 | 結果 |
|---|---|
| 開けない、1 MiB 超、読み取り失敗、UTF-8・JSON の構文・形式の誤り(`run_id` の不一致以外の `corrupt`。キーの重複・BOM を含む)、`versionMismatch` | **残して**、名前を `kept` に入れる |
| 長さ 0 | 残す。`kept` に**入れない**(誰のものか分からない) |
| 読めて、中身の `run_id` が名前の run ID と一致(完了のままのものを含む) | **削除** |
| 読めるが、中身の `run_id` が無い、または値が違う(別の定義のステートでありうる) | 残す。`kept` に入れない(黙って残す) |

4. 一時ファイル(`.<ステート名>.tmp.*`)、ほかの名前のファイル、ほかのディレクトリは対象外。
5. 削除の失敗(「存在しない」以外)→ `removeFailed`、65(処理は途中で打ち切る)。ディレクトリ一覧の取得に失敗(ディレクトリが無い場合を除く)→ `unreadable`、65。ディレクトリが無ければ成功。

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
- 読み込み(`Read`)は、完了のステートを「空」として返す(§6.1 の 14)。したがって次の `pipeline` は先頭から始まり、`run` は最初のジョブ以外が順序外(66)になる。
- `watch` は `ReadRaw` で完了のステートを読み、表の下に完了を示す行(英語 `completed`、日本語 `完了`)を足す。`no state (nothing has run yet)` を出すのは、ファイルが無いか長さ 0 のときだけ。壊れた完了のステートは 65。
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

---

## 12. 再現すべき特記事項

| # | 項目 | 挙動 |
|---|---|---|
| 1 | `reset --all` と長さ 0 のファイル | 長さ 0 の `<定義>.<run-id>.state` は、消さず、警告にも出さない |
| 2 | `reset`(個別)とシンボリックリンク | 個別の `reset` は、リンク自体を削除する。`reset --all` は、シンボリックリンクを消さない |
| 3 | 理由文の言語 | 理由(例: `JSON として読めません`)は、ロケールに関わらず日本語の固定文。種類の文と案内はロケールに従う |
| 4 | `definition_key` | 形式に残っているが、`definition_path` と同じ値で、照合に使わない |
| 5 | 版の先行検査 | `format_version` は、未知の項目の検査より先に調べる |
| 6 | キーの大文字小文字 | 区別する。`Format_Version` や `ID` は未知のキー(または必須のキーが無い)として扱う(`encoding/json` の構造体への読み込みの既定とは異なるので、§6.5 の方法で実装する) |
| 7 | `null` | どの項目の `null` も型の誤り。「無い」とは扱わない |

---

## 13. 実装に任せてよい点と未確認

実装に任せてよい点:

- 一時ファイルの名前の細部(§1 の条件を満たすこと)、再試行の回数の上限の具体値は 16 を推奨。
- 内部の構造(書き出しの型の名前など)。ただし、キーの順序・インデント・末尾の改行(§4)は固定。
- ファイルの権限(§5。`0o666` から umask を引いた OS 標準)。

未確認:

- Windows での `os.Rename` による置き換えの細部(既存ファイルが他のプロセスに開かれているとき)と、ディレクトリを開いたときの `unreadable` の判定(UNIX 系では、ディレクトリを `os.Open` でき、読み取りの失敗で `unreadable` になる想定。実機の確認はしていない)。
- ファイルの権限の挙動(Windows の ACL)。
- `json.Valid` が BOM・余分な文字・深すぎる入れ子を拒否すること、`Decoder.Token()` が返すキーがエスケープ展開後であること、`encoding/json` の構造体の出力順が宣言順であること、`os.Rename` が Windows で既存のファイルを置き換えること、`os.Remove` が空のディレクトリを消せること。いずれも Go の仕様・実装についての理解に基づく。実装時に、採用する Go の版で、単体テスト(§6.6、§4.2 の順序の完全一致)により確認する。
- 一時ファイルの `<時刻>` は、UNIX エポックからのナノ秒(`UnixNano`)。クロックが戻った場合の挙動は、名前の衝突の再試行(最大 16 回)に任せている。
- §4.4 の例(4)は、規則からの導出で、実装の出力として確認していない。例(5)(6)も、規則からの導出(Go の `encoding/json` の出力を実機で確認していない。実装時に、出力が各例と一致することを単体テストで確認する)。
